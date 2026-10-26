"""The PPO (E05 T8, `docs/epics/E05-log.md` §8.3): self-play over a `BattleBatch` of the
encounter distribution, one network for both sides, one trajectory per (env, side). A run's
settings, schedule and records: `run` (E06 T2a, ADR-0026).

Each call's sample is the acting side's decision (observation, masks, chosen heads, its
log-probability and value). `SeatBook` closes a side's transition at its next decision or at the
episode's end with the rewards in between (reward v1, terminal); the sample that opened it gets
that reward. GAE runs backwards over the rollout's calls with a carry per (env, side): a sample's
next value is its side's next sample's, 0 after a decided battle, after a truncation 0 with the
reward `-truncation_penalty` (E06 T5c; with 0 instead the value of `final_obs[env, side]`), and
the value of the batch's current observation for the side to move
when the rollout ends. A transition still open at the end (the side not to move) has no next value
yet and is left out of that update; the sample before it bootstraps from its value.

A run checkpoints its whole training state (`Trainer.state`, design §3.1) at update boundaries and
continues from it with `resume` in a new segment (§3.3): a fresh batch on the segment's own seed,
the episodes in flight at the stop discarded.

The gate (E06 T3b, design §6): the run starts with best #0 (the untrained network, or the parent's
with `--init-from`); every `gate_every` samples, and at the end if it trained since, the learner
plays the best on the gate set v1 and an SPRT decides (`gate`). A promoted learner becomes
`promoted/<n>.pt` and `best.pt`, and its ladder plays it against the engine's seats and the
promoted networks before it on the report set v1 (§7); at the end the final best's on the whole
set. A gate or ladder that began after the last checkpoint is pending: a resume runs it first.

The league (E06 T4a, design §8; `league`): a snapshot of the learner every `snapshot_every`
samples and every promoted network join it; an episode is self-play or against a live member,
which plays one side (only the learner's decisions train), and its result counts towards that
member's score. A member's network sits on the device and in its file (`league/<id>.pt`, or its
`promoted/<n>.pt`), outside the checkpoint, which names the members; a retired snapshot's file is
deleted after the next checkpoint, and a resume removes the files its checkpoint does not name.

The neutral share (E06 T7c, design §12): a neutral kind's episode (a wandering monster, a pile's or
a garrison's guards, a neutral town; `[env] kinds` draws them) has the system actor in the
defender's seat inside the batch, so the learner plays only its attacker and the league draws no
member for it. The update record splits the self-play episodes' rates by kind (`by_kind`), and
the ladder also plays the neutral set v1 (`[ladder] ladder_neutral`), the learner attacking the
system actor.

The system actor as an opponent (E06 T6b, decision 9 amended): with `[league] system_share` a
share of the player kinds' episodes has the system actor on a uniformly drawn side inside the
batch (`episode_entries`: the player kinds' own entry beside one per side), so the learner plays
the other and the league draws no member; the update record's `system` gives the learner's score
there by its side.
"""

from __future__ import annotations

import time
from collections.abc import Callable, Iterable
from dataclasses import asdict
from pathlib import Path
from typing import Any

import numpy as np
import torch

from .. import _ffi
from ..batch import BattleBatch
from ..seats import SeatBook
from .evaluate import (
    NEUTRAL_KINDS,
    PLAYER_KINDS,
    Stopped,
    compact,
    held_out_seeds,
    ladder,
    load,
)
from .gate import Bounds, game_score, gate
from .league import League, Member
from .policy import Choice, Policy, build
from .run import (
    Cadence,
    Config,
    ConfigError,
    HeldOut,
    ResumeError,
    Run,
    Stop,
    atomic_write,
    learning_rate,
    machine,
    resolve,
    resumed,
    segment_seed,
    source,
)

#: The version of `checkpoint.pt`'s layout (`Trainer.state`); 2: the gate's state instead of the
#: evaluations' (T3b); 3: the league instead of the pool, its networks outside (T4a).
CHECKPOINT_FORMAT = 3


class Rollout:
    """The samples of `calls` batch calls of `envs` envs, indexed [call, env]; a row without a
    side to move holds none (`side` -1)."""

    def __init__(self, calls: int, envs: int, batch: BattleBatch) -> None:
        t, n = calls, envs
        self.obs = np.zeros((t, n, batch.observation_len), np.float32)
        # The masks as bool, the dtype the network takes.
        self.entry_mask = np.zeros((t, n, batch.action_count), bool)
        self.spell_mask = np.zeros((t, n, batch.spell_count), bool)
        self.target_mask = np.zeros((t, n, batch.target_count), bool)
        self.second_mask = np.zeros((t, n, batch.target_count), bool)
        self.entry = np.zeros((t, n), np.int64)
        self.spell = np.zeros((t, n), np.int64)
        self.target = np.zeros((t, n), np.int64)
        self.second = np.zeros((t, n), np.int64)
        self.cast = np.zeros((t, n), bool)
        self.logprob = np.zeros((t, n), np.float32)
        self.value = np.zeros((t, n), np.float32)
        self.side = np.full((t, n), -1, np.int8)
        self.reward = np.zeros((t, n), np.float32)
        #: The transition closed in this rollout: at the side's next decision or the episode's end.
        self.closed = np.zeros((t, n), bool)
        self.terminal = np.zeros((t, n), bool)
        self.truncated = np.zeros((t, n), bool)
        self.final_value = np.zeros((t, n), np.float32)

    def clear(self) -> None:
        self.side.fill(-1)
        self.reward.fill(0.0)
        self.closed.fill(False)
        self.terminal.fill(False)
        self.truncated.fill(False)
        self.cast.fill(False)

    def store(self, t: int, batch: BattleBatch, choice: Choice, side: np.ndarray) -> None:
        """Call `t`'s samples: the batch's outputs before the step, the chosen actions and each
        row's learning side (-1 where none: no side to move, or a frozen network's move)."""
        self.obs[t] = batch.obs
        self.entry_mask[t] = batch.entry_mask
        self.side[t] = side
        self.entry[t] = choice.entry
        self.cast[t] = choice.cast
        if choice.cast.any():
            rows = choice.cast
            self.spell_mask[t, rows] = choice.spell_mask[rows]
            self.target_mask[t, rows] = choice.target_mask[rows]
            self.second_mask[t, rows] = choice.second_mask[rows]
        self.spell[t] = choice.spell
        self.target[t] = choice.target
        self.second[t] = choice.second
        self.logprob[t] = choice.logprob
        self.value[t] = choice.value


#: The rollout's fields an update reads per sample.
FIELDS = (
    "obs",
    "entry_mask",
    "entry",
    "cast",
    "spell_mask",
    "spell",
    "target_mask",
    "target",
    "second_mask",
    "second",
    "logprob",
)


class Staging:
    """Minibatches gathered into tensors allocated once and reused (a fresh array per minibatch,
    145 MB of observations at the default settings, costs its page faults every time); pinned
    for a GPU."""

    def __init__(self, device: torch.device) -> None:
        self.device = device
        self.pin = device.type == "cuda"
        self.buffers: dict[str, torch.Tensor] = {}

    def gather(self, arrays: dict[str, np.ndarray], idx: np.ndarray) -> dict[str, torch.Tensor]:
        """Rows `idx` of each [calls, envs, ...] array, on the device."""
        index = torch.from_numpy(idx)
        out = {}
        for name, array in arrays.items():
            source = torch.from_numpy(array.reshape(-1, *array.shape[2:]))
            buffer = self.buffers.get(name)
            if buffer is None or buffer.shape[0] < len(idx):
                shape = (len(idx), *source.shape[1:])
                buffer = torch.empty(shape, dtype=source.dtype, pin_memory=self.pin)
                self.buffers[name] = buffer
            part = buffer[: len(idx)]
            torch.index_select(source, 0, index, out=part)
            out[name] = part.to(self.device)
        return out


def gae(
    side: np.ndarray,
    value: np.ndarray,
    reward: np.ndarray,
    closed: np.ndarray,
    terminal: np.ndarray,
    truncated: np.ndarray,
    final_value: np.ndarray,
    last_side: np.ndarray,
    last_value: np.ndarray,
    gamma: float,
    lam: float,
) -> tuple[np.ndarray, np.ndarray]:
    """Generalized advantages of a two-sided rollout ([calls, envs] arrays, see the module's
    documentation); `last_side`, `last_value` [envs]: the side to move after the last call and
    the value of its observation. Returns the advantages and which samples the update uses."""
    calls, envs = side.shape
    adv = np.zeros((calls, envs), np.float32)
    carry_v = np.zeros((envs, 2), np.float32)
    carry_a = np.zeros((envs, 2), np.float32)
    rows = np.arange(envs)
    live = last_side >= 0
    carry_v[rows[live], last_side[live].astype(np.intp)] = last_value[live]
    for t in reversed(range(calls)):
        valid = side[t] >= 0
        e = rows[valid]
        s = side[t, e].astype(np.intp)
        v = value[t, e]
        term, trunc = terminal[t, e], truncated[t, e]
        next_v = np.where(trunc, final_value[t, e], np.where(term, 0.0, carry_v[e, s]))
        next_a = np.where(term, 0.0, carry_a[e, s])
        a = reward[t, e] + gamma * next_v - v + gamma * lam * next_a
        a = np.where(closed[t, e], a, 0.0).astype(np.float32)
        adv[t, e] = a
        carry_v[e, s] = v
        carry_a[e, s] = a
    return adv, (side >= 0) & closed


#: The seats (attacker, defender) of an entry where the system actor plays side 0 or 1.
_SEATS = (("system", "external"), ("external", "system"))


def episode_entries(config: Config) -> list[tuple[int, list[int] | None, int]]:
    """The batch's encounter entries (weight, kind weights, the side the system actor plays in
    their player kinds or -1): one, `[env] kinds` as given, without `system_share`; with it the
    neutral kinds' entry, the player kinds' own (their seats external) and one per side where
    the system actor plays it, weighed so that every kind keeps its share of the episodes and
    `system_share` (in thousandths) of the player kinds' have the system actor (E06 T6b)."""
    c = config
    share = round(c.system_share * 1000)
    if not share:
        return [(1, c.kinds, -1)]
    assert c.kinds is not None  # `resolve` checks it
    player = [w if k in PLAYER_KINDS else 0 for k, w in enumerate(c.kinds)]
    neutral = [0 if k in PLAYER_KINDS else w for k, w in enumerate(c.kinds)]
    p = sum(player)
    entries = [
        (2000 * sum(neutral), neutral, -1),
        ((2000 - 2 * share) * p, player, -1),
        (share * p, player, 0),
        (share * p, player, 1),
    ]
    return [e for e in entries if e[0]]


def device_of(name: str) -> torch.device:
    if name == "auto":
        return torch.device("cuda" if torch.cuda.is_available() else "cpu")
    return torch.device(name)


class Trainer:
    """One run's state: the batch, the network, the optimizer, the open transitions, the
    schedule. `segment` (0 for a new run) chooses the batch's seed (`run.segment_seed`)."""

    def __init__(self, config: Config, segment: int = 0) -> None:
        self.config = c = config
        torch.manual_seed(c.seed)
        if c.torch_threads:
            torch.set_num_threads(c.torch_threads)
        self.device = device_of(c.device)
        self.segment = segment
        self.batch_seed = segment_seed(c.seed, segment)
        self.batch = BattleBatch(
            c.envs,
            threads=c.threads,
            seed=self.batch_seed,
            max_steps=c.max_steps,
            currency=c.currency,
            mana_price=c.mana_price,
        )
        entries = episode_entries(c)
        for weight, kinds, side in entries:
            seats = {} if side < 0 else dict(zip(("attacker", "defender"), _SEATS[side]))
            self.batch.add_encounters(weight, kinds=kinds, **seats)
        #: Per batch entry (`batch.scenario`), the side the system actor plays in the player
        #: kinds (-1: none; the neutral kinds' defender is the encounters' own, E06 T6b).
        self.system_side = np.array([side for _, _, side in entries], np.int8)
        #: The held-out episodes' seeds (v0's, the report and gate sets v1's): every training
        #: episode's is checked against them (design §5).
        self.held_out = HeldOut(held_out_seeds())
        self.batch.reset()
        self.held_out.check(self.batch.episode_seed)
        self.policy = build(asdict(c), self.batch).to(self.device)
        self.optimizer = torch.optim.Adam(self.policy.parameters(), lr=c.lr, eps=1e-5)
        self.generator = torch.Generator(device=self.device).manual_seed(c.seed)
        self.book = SeatBook(c.envs)
        self.rollout = Rollout(c.rollout, c.envs, self.batch)
        self.staging = Staging(self.device)
        #: Where each (env, side)'s open transition sits in the rollout (-1: none, or earlier).
        self.opened = np.full((c.envs, 2), -1, np.int64)
        self.samples = 0
        self.updates = 0
        self.episodes = 0
        #: The league (`league`), its members' frozen networks by id, and per env the side its
        #: current episode's member plays (-1: self-play), that member's id (-1) and network.
        self.league = League(c, seed=c.seed)
        self.nets: dict[int, Policy] = {}
        self.frozen_side = np.full(c.envs, -1, np.int8)
        self.frozen_id = np.full(c.envs, -1, np.int64)
        self.frozen_net: list[Policy | None] = [None] * c.envs
        #: The frozen networks the last call passed (`act`).
        self.passes = 0
        self.snapshots = Cadence(c.snapshot_every)
        self.gates = Cadence(c.gate_every)
        #: The update of the last gate (-1: none yet).
        self.last_gate = -1
        #: The promoted network and update of the last final ladder (none yet).
        self.last_final: list[int] | None = None
        #: Each promoted network's entry (index, how, samples, update, minutes), in order.
        self.promoted: list[dict[str, Any]] = []
        #: The latest promoted networks (index, frozen network): the best last, before it the
        #: ones its ladder plays (`ladder_promoted`).
        self.lineage: list[tuple[int, Policy]] = []
        #: What runs before the training goes on (design §3): `{"kind": "gate"}` (the learner
        #: against the best), `{"kind": "ladder", "final": bool}` (the best's), or None.
        self.pending: dict[str, Any] | None = None

    def state(self, **run: Any) -> dict[str, Any]:
        """The whole training state at an update boundary (`checkpoint.pt`, design §3.1), with
        the run's own entries (`run`: its clock, its metrics' length, whether it finished). Not
        in it: the batch's battles and the open transitions (a resume starts new episodes). The
        update's minibatch order needs no generator (`default_rng(updates)`)."""
        cuda = torch.cuda.get_rng_state_all() if torch.cuda.is_available() else []
        return {
            "format": CHECKPOINT_FORMAT,
            "config": asdict(self.config),
            "segment": self.segment,
            "updates": self.updates,
            "samples": self.samples,
            "episodes": self.episodes,
            "next_snapshot": self.snapshots.next,
            "next_gate": self.gates.next,
            "last_gate": self.last_gate,
            "last_final": self.last_final,
            "promoted": self.promoted,
            "pending": self.pending,
            "policy": self.policy.state_dict(),
            "optimizer": self.optimizer.state_dict(),
            "league": self.league.state(),
            "rng": {
                "device": self.device.type,
                "sampling": self.generator.get_state(),
                "torch": torch.get_rng_state(),
                "cuda": cuda,
            },
            **run,
        }

    def restore(self, state: dict[str, Any], out: Path) -> None:
        """Takes a checkpoint's training state over (`state`'s config is this trainer's), and
        from the run's directory `out` the networks outside the checkpoint: the latest promoted
        ones (`promoted/<n>.pt`) and the league's members (their files). The sampling generator's
        state fits only a device of the same type: resumed on another, the generator starts from
        the segment's seed instead."""
        c, out = self.config, Path(out)
        self.policy.load_state_dict(state["policy"])
        self.optimizer.load_state_dict(state["optimizer"])
        self.updates, self.samples = state["updates"], state["samples"]
        self.episodes = state["episodes"]
        self.snapshots.next, self.gates.next = state["next_snapshot"], state["next_gate"]
        self.last_gate, self.last_final = state["last_gate"], state["last_final"]
        self.promoted, self.pending = list(state["promoted"]), state["pending"]
        self.lineage = [
            (e["index"], load(out / "promoted" / f"{e['index']}.pt", self.device).frozen())
            for e in self.promoted[-(c.ladder_promoted + 1) :]
        ]
        self.league.restore(state["league"])
        self.nets = {m.id: load(out / m.file, self.device).frozen() for m in self.league.members}
        rng = state["rng"]
        if rng["device"] == self.device.type:
            self.generator.set_state(rng["sampling"])
        else:
            self.generator.manual_seed(self.batch_seed)
        torch.set_rng_state(rng["torch"])
        if rng["cuda"] and len(rng["cuda"]) == torch.cuda.device_count():
            torch.cuda.set_rng_state_all(rng["cuda"])

    def assign(self, envs: np.ndarray) -> None:
        """Draws the opponent of each of `envs`' new episodes (`League.opponent`): a live member
        on a uniformly drawn side, or self-play. An episode with the system actor draws none (a
        neutral kind's defender, or a side of the player kinds with `system_share`): the learner
        plays its only external seat (design §12)."""
        kind, entry = self.batch.encounter, self.batch.scenario
        for e in envs:
            internal = int(kind[e]) in NEUTRAL_KINDS or self.system_side[entry[e]] >= 0
            member = None if internal else self.league.opponent()
            if member is None:
                self.frozen_side[e], self.frozen_id[e], self.frozen_net[e] = -1, -1, None
            else:
                self.frozen_side[e] = self.league.side()
                self.frozen_id[e] = member.id
                self.frozen_net[e] = self.nets[member.id]

    def init_from(self, path: Path) -> dict[str, Any]:
        """The learner starts from a saved network (`--init-from`: a run's `best.pt`,
        `promoted/<n>.pt` or `checkpoint.pt`, of this network's kind and size); returns the
        parent's entry for the record (its file, run, promoted index, samples and updates)."""
        saved = torch.load(path, map_location=self.device, weights_only=True)
        kind = saved.get("config", {}).get("kind", "mlp")
        if kind != self.config.kind:
            raise ConfigError(
                f"--init-from {path}: a network of kind {kind!r}, this run's is "
                f"{self.config.kind!r}"
            )
        try:
            self.policy.load_state_dict(saved["policy"])
        except (KeyError, RuntimeError) as e:
            raise ConfigError(
                f"--init-from {path}: not a network of this run's size ({e})"
            ) from None
        return {
            "path": str(path),
            **{k: saved.get(k) for k in ("run", "index", "samples", "updates")},
        }

    def promote(self) -> int:
        """The learner's network becomes the next promoted one and the best (frozen); returns
        its index. The caller records its entry (`promoted`)."""
        index = len(self.promoted)
        keep = self.config.ladder_promoted + 1
        self.lineage = [*self.lineage, (index, self.policy.frozen())][-keep:]
        return index

    def gate(self, stop: Stop | None = None) -> dict[str, Any]:
        """The learner against the best on the gate set v1 (`gate.gate`); raises `Stopped` on a
        stop."""
        c = self.config
        result = gate(
            self.policy,
            self.lineage[-1][1],
            Bounds(c.elo0, c.elo1, c.alpha, c.beta, c.max_pairs),
            threads=c.threads,
            max_steps=c.max_steps,
            stop=stop,
        )
        self.last_gate = self.updates
        return {"best": self.lineage[-1][0], **result}

    def ladder(self, final: bool, stop: Stop | None = None) -> dict[str, Any]:
        """The best's ladder (design §7): against the config's engine seats and the promoted
        networks before it, on the report set v1's first `ladder_episodes` per kind (the final
        one: `ladder_final_episodes`), and with `ladder_neutral` attacking the system actor on
        the neutral set v1's as many (`neutral`; design §12); its values at reward v1's settings
        (`evaluate.CURRENCY`, `MANA_PRICE`), whatever the run trains with; raises `Stopped` on a
        stop."""
        c = self.config
        index, best = self.lineage[-1]
        per_kind = c.ladder_final_episodes if final else c.ladder_episodes
        settings = dict(
            per_kind=per_kind,
            threads=c.threads,
            max_steps=c.max_steps,
            stop=stop,
        )
        rows = ladder(
            best,
            opponents=c.ladder_opponents,
            networks={f"promoted-{i}": net for i, net in self.lineage[:-1]},
            **settings,
        )
        neutral = None
        if c.ladder_neutral:
            neutral = compact(ladder(best, held_out="neutral-v1", **settings)["system"])
        if final:
            self.last_final = [index, self.updates]
        return {
            "index": index,
            "final": final,
            "per_kind": per_kind,
            "opponents": {name: compact(row) for name, row in rows.items()},
            "neutral": neutral,
            "seconds": sum(row["seconds"] for row in rows.values())
            + (neutral["seconds"] if neutral else 0.0),
        }

    def join(self, index: int | None = None) -> tuple[Member, Member | None]:
        """The learner's network joins the league: a snapshot (`league/<id>.pt`), or with
        `index` the promoted network it just became (`promoted/<index>.pt`, the lineage's copy).
        Returns the member and the one it retired (its network dropped). The caller writes the
        files."""
        if index is None:
            kind, file, net = "snapshot", f"league/{self.league.next_id}.pt", self.policy.frozen()
        else:
            kind, file, net = "promoted", f"promoted/{index}.pt", self.lineage[-1][1]
        member, retired = self.league.join(kind, file, self.samples, self.updates)
        self.nets[member.id] = net
        if retired is not None:
            del self.nets[retired.id]
        return member, retired

    def act(self) -> tuple[Choice, np.ndarray]:
        """Every row's action, the frozen networks' rows overwritten by theirs; and each row's
        learning side (-1 for no side to move or a frozen network's move). `passes`: the frozen
        networks this call passed."""
        b = self.batch
        choice = self.policy.act(b, self.generator)
        to_move = b.to_move
        frozen = (to_move >= 0) & (self.frozen_side == to_move)
        groups: dict[int, tuple[Policy, list[int]]] = {}
        for e in np.nonzero(frozen)[0]:
            net = self.frozen_net[e]
            groups.setdefault(id(net), (net, []))[1].append(int(e))
        for net, envs in groups.values():
            rows = np.array(envs)
            c = net.act(b, self.generator, rows=rows)
            for name in ("entry", "spell", "target", "second"):
                getattr(choice, name)[rows] = getattr(c, name)
            choice.cast[rows] = False
        self.passes = len(groups)
        return choice, np.where(frozen, -1, to_move).astype(np.int8)

    def collect(self, stop: Stop | None = None) -> dict | None:
        """Plays one rollout; returns its episodes' statistics, or None when `stop` was
        requested before one of its calls (the rollout is discarded: it changed no weights and
        no counter, only the generators moved on)."""
        b, r, c = self.batch, self.rollout, self.config
        r.clear()
        self.opened.fill(-1)
        rows = np.arange(c.envs)
        results = np.zeros(_ffi.RESULT_DEFENDER_SURRENDERED + 1, np.int64)
        truncated = 0
        decisions = 0
        episodes = 0
        start = time.perf_counter()
        drawn = self.league.draw(self.samples)
        #: Per member id: the learner's score sum and games in this rollout.
        tally: dict[int, list[float]] = {}
        #: Per encounter kind, the self-play episodes': the attacker's wins, truncations, games.
        by_kind: dict[int, list[int]] = {}
        #: Per side of the learner against the system actor in the player kinds: its score sum
        #: and games.
        against_system = [[0.0, 0], [0.0, 0]]
        passes = 0
        for t in range(c.rollout):
            if stop is not None and stop.requested:
                return None
            choice, side = self.act()
            passes += self.passes
            r.store(t, b, choice, side)
            live = side >= 0
            decisions += int(live.sum())
            self.opened[rows[live], side[live].astype(np.intp)] = t
            self.book.act(b)
            a = b.actions
            a["entry"], a["spell"] = choice.entry, choice.spell
            a["target"], a["second"] = choice.target, choice.second
            # The kind and entry of each env's episode before the step (an ended one's next comes
            # after).
            kinds, entries = b.encounter.copy(), b.scenario.copy()
            b.step()
            closed = self.book.settle(b)
            e, s = np.nonzero(closed.closed)
            at = self.opened[e, s]
            mine = at >= 0
            e, s, at = e[mine], s[mine], at[mine]
            r.reward[at, e] = closed.reward[e, s]
            r.closed[at, e] = True
            r.terminal[at, e] = closed.terminal[e, s]
            trunc = closed.truncated[e, s]
            if c.truncation_penalty:
                # A battle cut at the step limit ends there (it is terminal) at a loss for each
                # learning side (E06 T5c): no bootstrap, so stalling to the limit never pays.
                r.reward[at[trunc], e[trunc]] -= c.truncation_penalty
                trunc = np.zeros_like(trunc)
            r.truncated[at, e] = trunc
            if trunc.any():
                final = torch.from_numpy(b.final_obs[e[trunc], s[trunc]]).to(self.device)
                with torch.no_grad():
                    v = self.policy.values(final).cpu().numpy()
                r.final_value[at[trunc], e[trunc]] = v
            self.opened[e, s] = -1
            done = np.nonzero(b.done)[0]
            if len(done):
                against = self.frozen_side[done] >= 0
                system = self.system_side[entries[done]]
                own = done[~against & (system < 0)]
                # The learner's side is the one the system actor does not play.
                for env, side in zip(done[system >= 0], system[system >= 0]):
                    row = against_system[1 - int(side)]
                    row[0] += game_score(int(b.result[env]), 1 - int(side))
                    row[1] += 1
                res = b.result[own]
                for value in res[res >= 0]:
                    results[int(value)] += 1
                truncated += int(b.truncated[own].sum())
                for env in own:
                    row = by_kind.setdefault(int(kinds[env]), [0, 0, 0])
                    row[0] += int(b.result[env]) == _ffi.RESULT_ATTACKER
                    row[1] += int(b.truncated[env])
                    row[2] += 1
                # The learner's side is the one the member does not play.
                for env in done[against]:
                    entry = tally.setdefault(int(self.frozen_id[env]), [0.0, 0])
                    entry[0] += game_score(int(b.result[env]), 1 - int(self.frozen_side[env]))
                    entry[1] += 1
                episodes += len(done)
                self.held_out.check(b.episode_seed[done])
                self.assign(done)
        self.episodes += episodes
        # A retired member's episodes count no more (`settle` skips ids it lacks).
        self.league.settle(tally)
        elapsed = time.perf_counter() - start
        with torch.no_grad():
            obs = torch.from_numpy(b.obs).to(self.device)
            self.last_value = self.policy.values(obs).cpu().numpy()
        # A frozen network to move: the learner's transition stays open (left out).
        frozen = (b.to_move >= 0) & (self.frozen_side == b.to_move)
        self.last_side = np.where(frozen, -1, b.to_move).astype(np.int8)
        ended = int(results.sum()) + truncated
        games = sum(g for _, g in tally.values())
        return {
            "decisions": decisions,
            "rollout_s": elapsed,
            # The self-play episodes that ended (the rates' denominator); not `episodes`, the
            # run's count in the update record.
            "self_play_episodes": ended,
            "attacker_won": int(results[_ffi.RESULT_ATTACKER]) / max(ended, 1),
            "truncated": truncated / max(ended, 1),
            # The same per encounter kind (in a neutral kind the learner's attacker against the
            # system actor).
            "by_kind": {
                _ffi.ENCOUNTER_KINDS[k]: {
                    "episodes": n,
                    "attacker_won": won / n,
                    "truncated": cut / n,
                }
                for k, (won, cut, n) in sorted(by_kind.items())
            },
            "league": len(self.league.members),
            "live": list(self.league.live),
            "drawn": drawn,
            # The learner's episodes against members and its mean score in them (a draw or a
            # truncation ½), and the frozen networks a call passed on average.
            "league_episodes": int(games),
            "league_score": sum(s for s, _ in tally.values()) / max(games, 1),
            "frozen_passes": passes / c.rollout,
            # The learner's episodes against the system actor in the player kinds by its side,
            # and its mean score there (`system_share`; a side without one is left out).
            "system": {
                name: {"episodes": n, "score": s / n}
                for name, (s, n) in zip(("attacker", "defender"), against_system)
                if n
            },
        }

    def update(self, lr: float) -> dict:
        """PPO's update over the rollout's samples."""
        c, r = self.config, self.rollout
        adv, used = gae(
            r.side,
            r.value,
            r.reward,
            r.closed,
            r.terminal,
            r.truncated,
            r.final_value,
            self.last_side,
            self.last_value,
            c.gamma,
            c.lam,
        )
        flat = np.nonzero(used.reshape(-1))[0]
        returns = adv + r.value
        for g in self.optimizer.param_groups:
            g["lr"] = lr
        start = time.perf_counter()
        arrays = {name: getattr(r, name) for name in FIELDS}
        arrays["adv"], arrays["returns"] = adv, returns
        stats = {"policy_loss": 0.0, "value_loss": 0.0, "entropy": 0.0, "kl": 0.0, "clip": 0.0}
        steps = 0
        rng = np.random.default_rng(self.updates)
        size = max(1, len(flat) // c.minibatches)
        for _ in range(c.epochs):
            order = rng.permutation(flat)
            for k in range(0, len(order) - size + 1, size):
                m = self.staging.gather(arrays, np.sort(order[k : k + size]))
                logp, ent, value = self.policy.evaluate(
                    m["obs"],
                    m["entry_mask"],
                    m["entry"],
                    m["cast"],
                    tuple(m[n] for n in FIELDS[4:10]),
                )
                old = m["logprob"]
                a = m["adv"]
                a = (a - a.mean()) / (a.std() + 1e-8)
                ratio = torch.exp(logp - old)
                pg = torch.max(-a * ratio, -a * ratio.clamp(1 - c.clip, 1 + c.clip)).mean()
                v_loss = 0.5 * ((value - m["returns"]) ** 2).mean()
                loss = pg - c.entropy_coef * ent.mean() + c.value_coef * v_loss
                self.optimizer.zero_grad()
                loss.backward()
                torch.nn.utils.clip_grad_norm_(self.policy.parameters(), c.max_grad_norm)
                self.optimizer.step()
                with torch.no_grad():
                    log_ratio = logp - old
                    stats["kl"] += ((ratio - 1) - log_ratio).mean().item()
                    stats["clip"] += ((ratio - 1).abs() > c.clip).float().mean().item()
                stats["policy_loss"] += pg.item()
                stats["value_loss"] += v_loss.item()
                stats["entropy"] += ent.mean().item()
                steps += 1
        stats = {k: v / max(steps, 1) for k, v in stats.items()}
        y, p = returns.reshape(-1)[flat], r.value.reshape(-1)[flat]
        var = float(np.var(y))
        stats["explained_variance"] = float("nan") if var == 0 else 1 - float(np.var(y - p)) / var
        stats["update_samples"] = len(flat)
        stats["update_s"] = time.perf_counter() - start
        self.samples += len(flat)
        self.updates += 1
        return stats

    def network(self, **meta: Any) -> dict[str, Any]:
        """The network alone (`best.pt`, `promoted/<n>.pt`, a snapshot's `league/<id>.pt`): its
        weights, the config that sizes it, `meta`."""
        return {
            "format": CHECKPOINT_FORMAT,
            "config": asdict(self.config),
            "policy": self.policy.state_dict(),
            "updates": self.updates,
            "samples": self.samples,
            **meta,
        }


def save(path: Path, state: dict[str, Any]) -> None:
    atomic_write(path, lambda f: torch.save(state, f))


def identity(trainer: Trainer) -> dict:
    """The record's identity (design §4): the source; the engine's ABI, encodings' and encounter
    distribution's versions, sizes and the batch's entries with their rulesets' content hashes
    (C ABI v31, T2c); torch and the device; the machine."""
    device = trainer.device
    return {
        "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        **source(),
        **_ffi.versions(),
        "entries": trainer.batch.entries(),
        "observation_len": trainer.batch.observation_len,
        "action_count": trainer.batch.action_count,
        "parameters": sum(p.numel() for p in trainer.policy.parameters()),
        "torch": torch.__version__,
        "cuda": torch.version.cuda,
        "device": str(device),
        "gpu": torch.cuda.get_device_name(device) if device.type == "cuda" else None,
        **machine(),
    }


def train(
    config: Config,
    out: Path | None = None,
    log: Callable[[dict], None] | None = None,
    text: str = "",
    stop: Stop | None = None,
    init_from: Path | None = None,
) -> Trainer:
    """Trains until `config.samples` (the first update boundary at or past it), gating the learner
    every `config.gate_every` samples and at the end (see the module's documentation). Into `out`
    when given: the run's directory (`run.Run`: `config.toml` from `text`, `metrics.jsonl`,
    `record.json`), `checkpoint.pt` (the whole training state: at the start, every
    `checkpoint_minutes`, before and after each gate and ladder, at a stop and at the end),
    `promoted/<n>.pt` and `best.pt`. `init_from`: a saved network the learner and best #0 start
    from (a new run: fresh optimizer, counters and league), named in the record as its parent.
    Ctrl+C (or `stop`'s request) stops the run at the next batch call (of a rollout, a gate or a
    ladder) or update boundary, saved (`run.Stop`): `resume` continues it."""
    config = resolve(config)
    trainer = Trainer(config)
    parent = None if init_from is None else trainer.init_from(Path(init_from))
    ident = identity(trainer)
    run = Run(out, config, ident, text, log)
    opening = {
        "type": "start",
        "name": run.record.name,
        "segment": 0,
        "seed": trainer.batch_seed,
        "parent": parent,
    }
    return _go(trainer, run, stop, {**opening, "identity": ident})


def resume(
    out: Path,
    sets: Iterable[str] = (),
    log: Callable[[dict], None] | None = None,
    stop: Stop | None = None,
) -> Trainer:
    """Continues the run in `out` from its checkpoint (design §3.3) in a new segment, to its
    budget: `sets` (`--set table.key=value`) may change only `run.RESUMABLE`'s keys (extend or
    cut the budget, move it to other threads or another device). Its metrics are cut back to the
    save, its batch starts new episodes on the segment's seed; a pending gate or ladder runs
    first."""
    out = Path(out)
    path = out / "checkpoint.pt"
    if not path.exists():
        raise ResumeError(f"{out} holds no checkpoint.pt to resume")
    state = torch.load(path, map_location="cpu", weights_only=True)
    if state.get("format") != CHECKPOINT_FORMAT:
        raise ResumeError(
            f"{path} is not a checkpoint of format {CHECKPOINT_FORMAT} (an older one, before the "
            "league, E06 T4a: start a new run, --init-from its best.pt)"
        )
    config, changed = resumed(Config(**state["config"]), sets)
    if state["finished"] and state["samples"] >= config.samples:
        raise ResumeError(
            f"the run in {out} reached its budget ({config.samples} samples): extend it with "
            "--set samples=N"
        )
    trainer = Trainer(config, segment=state["segment"] + 1)
    trainer.restore(state, out)
    # The league's files the checkpoint does not name: snapshots taken after it, or retired
    # before it (deleted only after the next checkpoint).
    named = {m.file for m in trainer.league.members}
    for stale in (out / "league").glob("*.pt"):
        if f"league/{stale.name}" not in named:
            stale.unlink()
    trainer.assign(np.arange(config.envs))
    run = Run(out, config, {}, log=log, resume=(state["metrics_length"], state["seconds"]))
    opening = {
        "type": "segment",
        "segment": trainer.segment,
        "seed": trainer.batch_seed,
        "dropped": run.dropped,
        "changed": changed,
        "pending": trainer.pending,
    }
    return _go(trainer, run, stop, {**opening, "identity": identity(trainer)})


def _go(trainer: Trainer, run: Run, stop: Stop | None, opening: dict) -> Trainer:
    """The run's loop from its opening record (`start` or `segment`) to its end or a stop."""
    config = trainer.config
    stop = Stop() if stop is None else stop
    last_save = time.monotonic()
    #: Retired snapshots' files, deleted after the next checkpoint (the last one names them).
    doomed: list[Path] = []

    def counters() -> dict:
        return {
            "update": trainer.updates,
            "samples": trainer.samples,
            "episodes": trainer.episodes,
            "minutes": run.minutes(),
        }

    def checkpoint(finished: bool = False) -> None:
        """The training state and then the record; the state names the metrics' length."""
        nonlocal last_save
        if run.out is not None:
            state = trainer.state(
                seconds=run.seconds(), metrics_length=run.sync(), finished=finished
            )
            save(run.out / "checkpoint.pt", state)
            run.write_record()
            for path in doomed:
                path.unlink(missing_ok=True)
            doomed.clear()
        last_save = time.monotonic()

    def join(index: int | None = None) -> dict:
        """The learner's network joins the league (`Trainer.join`), a snapshot's file written;
        returns the record's fields."""
        member, retired = trainer.join(index)
        if run.out is not None:
            if member.kind == "snapshot":
                (run.out / "league").mkdir(exist_ok=True)
                save(run.out / member.file, trainer.network(member=member.id, run=run.record.name))
            if retired is not None and retired.kind == "snapshot":
                doomed.append(run.out / retired.file)
        return {
            "member": member.id,
            "retired": None if retired is None else retired.id,
            "league": len(trainer.league.members),
        }

    def promote(how: str) -> None:
        """The learner becomes the next promoted network, the best and a league member; its
        ladder is pending."""
        index = trainer.promote()
        record = {"type": "promotion", "index": index, "how": how, **counters()}
        trainer.promoted.append({k: record[k] for k in ("index", "how", "samples", "update")})
        if run.out is not None:
            network = trainer.network(
                index=index, how=how, run=run.record.name, parent=run.record.parent
            )
            (run.out / "promoted").mkdir(exist_ok=True)
            save(run.out / "promoted" / f"{index}.pt", network)
            save(run.out / "best.pt", network)
        run.emit({**record, **join(index)})
        trainer.pending = {"kind": "ladder", "final": False}

    def settle() -> None:
        """Runs what is pending, each saved before it starts and after it ends: a gate (the
        learner against the best; a promotion adds its ladder), a ladder."""
        while trainer.pending is not None:
            checkpoint()
            if trainer.pending["kind"] == "gate":
                result = trainer.gate(stop)
                run.emit({"type": "gate", **counters(), **result, "league": trainer.league.table()})
                trainer.pending = None
                if result["result"] == "promoted":
                    promote("gate")
            else:
                result = trainer.ladder(trainer.pending["final"], stop)
                run.emit({"type": "ladder", **counters(), **result})
                trainer.pending = None
        checkpoint()

    with stop:
        try:
            run.emit({**opening, **counters()})
            if opening["type"] == "start":
                promote("init" if run.record.parent else "start")
            settle()
            while trainer.samples < config.samples and not stop.requested:
                lr = learning_rate(config, trainer.samples)
                roll = trainer.collect(stop)
                if roll is None:
                    break
                stats = trainer.update(lr)
                run.emit(
                    {
                        "type": "update",
                        **counters(),
                        "lr": lr,
                        "samples_per_s": stats["update_samples"]
                        / (roll["rollout_s"] + stats["update_s"]),
                        **roll,
                        **stats,
                    }
                )
                if trainer.snapshots.due(trainer.samples):
                    run.emit({"type": "snapshot", **counters(), **join()})
                if trainer.gates.due(trainer.samples):
                    trainer.pending = {"kind": "gate"}
                    settle()
                elif time.monotonic() - last_save >= 60 * config.checkpoint_minutes:
                    checkpoint()
            if stop.requested:
                raise Stopped
            # The end: a gate if the learner trained since the last, then the best's final ladder.
            if trainer.last_gate != trainer.updates:
                trainer.pending = {"kind": "gate"}
                settle()
            best = trainer.lineage[-1][0]
            if config.ladder_final_episodes and trainer.last_final != [best, trainer.updates]:
                trainer.pending = {"kind": "ladder", "final": True}
                settle()
            run.emit({"type": "end", "reason": "budget", **counters()})
            checkpoint(finished=True)
        except Stopped:
            run.emit({"type": "stop", **counters()})
            checkpoint()
        finally:
            run.close()
    return trainer
