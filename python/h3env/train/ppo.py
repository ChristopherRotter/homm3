"""The PPO (E05 T8, `docs/epics/E05-log.md` §8.3): self-play over a `BattleBatch` of the
encounter distribution, one network for both sides, one trajectory per (env, side). A run's
settings, schedule and records: `run` (E06 T2a, ADR-0026).

Each call's sample is the acting side's decision (observation, masks, chosen heads, its
log-probability and value). `SeatBook` closes a side's transition at its next decision or at the
episode's end with the rewards in between (reward v1, terminal); the sample that opened it gets
that reward. GAE runs backwards over the rollout's calls with a carry per (env, side): a sample's
next value is its side's next sample's, 0 after a decided battle, the value of `final_obs[env,
side]` after a truncation, and the value of the batch's current observation for the side to move
when the rollout ends. A transition still open at the end (the side not to move) has no next value
yet and is left out of that update; the sample before it bootstraps from its value.

A run checkpoints its whole training state (`Trainer.state`, design §3.1) at update boundaries and
continues from it with `resume` in a new segment (§3.3): a fresh batch on the segment's own seed,
the episodes in flight at the stop discarded.
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
from .evaluate import evaluate, held_out_seeds
from .policy import Choice, Policy
from .run import (
    Cadence,
    Config,
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

#: The version of `checkpoint.pt`'s layout (`Trainer.state`).
CHECKPOINT_FORMAT = 1


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
        self.batch.add_encounters(kinds=c.kinds)
        #: The held-out episodes' seeds (the evaluation's, v0, and the report and gate sets v1):
        #: every training episode's is checked against them (design §5).
        self.held_out = HeldOut(held_out_seeds(c.eval_episodes, c.eval_envs, c.eval_seed))
        self.batch.reset()
        self.held_out.check(self.batch.episode_seed)
        self.policy = Policy.for_batch(self.batch, c.hidden, c.embed).to(self.device)
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
        self.best = -1.0
        #: The opponent pool (frozen snapshots) and, per env, the side its current episode's
        #: frozen network plays (-1: self-play) and that network.
        self.pool: list[Policy] = []
        self.pool_rng = np.random.default_rng(c.seed)
        self.frozen_side = np.full(c.envs, -1, np.int8)
        self.frozen_net: list[Policy | None] = [None] * c.envs
        self.snapshots = Cadence(c.pool_every if c.pool_share > 0 else 0)
        self.evaluations = Cadence(c.eval_every)

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
            "best": self.best,
            "next_snapshot": self.snapshots.next,
            "next_eval": self.evaluations.next,
            "policy": self.policy.state_dict(),
            "optimizer": self.optimizer.state_dict(),
            "pool": [net.state_dict() for net in self.pool],
            "rng": {
                "device": self.device.type,
                "sampling": self.generator.get_state(),
                "pool": self.pool_rng.bit_generator.state,
                "torch": torch.get_rng_state(),
                "cuda": cuda,
            },
            **run,
        }

    def restore(self, state: dict[str, Any]) -> None:
        """Takes a checkpoint's training state over (`state`'s config is this trainer's). The
        sampling generator's state fits only a device of the same type: resumed on another, the
        generator starts from the segment's seed instead."""
        c = self.config
        self.policy.load_state_dict(state["policy"])
        self.optimizer.load_state_dict(state["optimizer"])
        self.updates, self.samples = state["updates"], state["samples"]
        self.episodes, self.best = state["episodes"], state["best"]
        self.snapshots.next, self.evaluations.next = state["next_snapshot"], state["next_eval"]
        self.pool = []
        for weights in state["pool"]:
            net = Policy.for_batch(self.batch, c.hidden, c.embed).to(self.device)
            net.load_state_dict(weights)
            self.pool.append(net.frozen())
        rng = state["rng"]
        if rng["device"] == self.device.type:
            self.generator.set_state(rng["sampling"])
        else:
            self.generator.manual_seed(self.batch_seed)
        self.pool_rng.bit_generator.state = rng["pool"]
        torch.set_rng_state(rng["torch"])
        if rng["cuda"] and len(rng["cuda"]) == torch.cuda.device_count():
            torch.cuda.set_rng_state_all(rng["cuda"])

    def assign(self, envs: np.ndarray) -> None:
        """Draws the opponent of each of `envs`' new episodes: a frozen network for one side with
        `pool_share` once the pool holds one, else self-play."""
        rng, c = self.pool_rng, self.config
        for e in envs:
            if self.pool and rng.random() < c.pool_share:
                self.frozen_side[e] = rng.integers(2)
                self.frozen_net[e] = self.pool[rng.integers(len(self.pool))]
            else:
                self.frozen_side[e] = -1
                self.frozen_net[e] = None

    def snapshot(self) -> None:
        """Adds the current network to the pool (replacing a random one when it is full)."""
        twin = self.policy.frozen()
        if len(self.pool) < self.config.pool_size:
            self.pool.append(twin)
        else:
            self.pool[self.pool_rng.integers(len(self.pool))] = twin

    def act(self) -> tuple[Choice, np.ndarray]:
        """Every row's action, the frozen networks' rows overwritten by theirs; and each row's
        learning side (-1 for no side to move or a frozen network's move)."""
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
        pool_episodes = pool_won = 0
        for t in range(c.rollout):
            if stop is not None and stop.requested:
                return None
            choice, side = self.act()
            r.store(t, b, choice, side)
            live = side >= 0
            decisions += int(live.sum())
            self.opened[rows[live], side[live].astype(np.intp)] = t
            self.book.act(b)
            a = b.actions
            a["entry"], a["spell"] = choice.entry, choice.spell
            a["target"], a["second"] = choice.target, choice.second
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
            r.truncated[at, e] = trunc
            if trunc.any():
                final = torch.from_numpy(b.final_obs[e[trunc], s[trunc]]).to(self.device)
                with torch.no_grad():
                    v = self.policy.value(self.policy.features(final)).cpu().numpy()
                r.final_value[at[trunc], e[trunc]] = v
            self.opened[e, s] = -1
            done = np.nonzero(b.done)[0]
            if len(done):
                pooled = self.frozen_side[done] >= 0
                own = done[~pooled]
                res = b.result[own]
                for value in res[res >= 0]:
                    results[int(value)] += 1
                truncated += int(b.truncated[own].sum())
                # The learner's side is the one the frozen network does not play.
                learner = 1 - self.frozen_side[done[pooled]]
                pool_episodes += len(learner)
                pool_won += int((b.result[done[pooled]] == learner).sum())
                episodes += len(done)
                self.held_out.check(b.episode_seed[done])
                self.assign(done)
        self.episodes += episodes
        elapsed = time.perf_counter() - start
        with torch.no_grad():
            obs = torch.from_numpy(b.obs).to(self.device)
            self.last_value = self.policy.value(self.policy.features(obs)).cpu().numpy()
        # A frozen network to move: the learner's transition stays open (left out).
        frozen = (b.to_move >= 0) & (self.frozen_side == b.to_move)
        self.last_side = np.where(frozen, -1, b.to_move).astype(np.int8)
        ended = int(results.sum()) + truncated
        return {
            "decisions": decisions,
            "rollout_s": elapsed,
            # The self-play episodes that ended (the rates' denominator); not `episodes`, the
            # run's count in the update record.
            "self_play_episodes": ended,
            "attacker_won": int(results[_ffi.RESULT_ATTACKER]) / max(ended, 1),
            "truncated": truncated / max(ended, 1),
            "pool": len(self.pool),
            "pool_episodes": pool_episodes,
            "pool_won": pool_won / max(pool_episodes, 1),
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
        """The network alone (`best.pt`): its weights, the config that sizes it, `meta`."""
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
) -> Trainer:
    """Trains until `config.samples` (the first update boundary at or past it), with an
    evaluation every `config.eval_every` samples and at the end. Into `out` when given: the run's
    directory (`run.Run`: `config.toml` from `text`, `metrics.jsonl`, `record.json`),
    `checkpoint.pt` (the whole training state: at the start, every `checkpoint_minutes`, after
    each evaluation, at a stop and at the end) and `best.pt` (the best sampled rate's network).
    The final evaluation reuses the last one if no update came between. Ctrl+C (or `stop`'s
    request) stops the run at the next batch call or update boundary, saved (`run.Stop`):
    `resume` continues it."""
    config = resolve(config)
    trainer = Trainer(config)
    ident = identity(trainer)
    run = Run(out, config, ident, text, log)
    opening = {"type": "start", "name": run.record.name, "segment": 0, "seed": trainer.batch_seed}
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
    save, its batch starts new episodes on the segment's seed."""
    out = Path(out)
    path = out / "checkpoint.pt"
    if not path.exists():
        raise ResumeError(f"{out} holds no checkpoint.pt to resume")
    state = torch.load(path, map_location="cpu", weights_only=True)
    if state.get("format") != CHECKPOINT_FORMAT:
        raise ResumeError(
            f"{path} is not a checkpoint of format {CHECKPOINT_FORMAT} (E05's held the network "
            "and the optimizer only)"
        )
    config, changed = resumed(Config(**state["config"]), sets)
    if state["finished"] and state["samples"] >= config.samples:
        raise ResumeError(
            f"the run in {out} reached its budget ({config.samples} samples): extend it with "
            "--set samples=N"
        )
    trainer = Trainer(config, segment=state["segment"] + 1)
    trainer.restore(state)
    trainer.assign(np.arange(config.envs))
    run = Run(out, config, {}, log=log, resume=(state["metrics_length"], state["seconds"]))
    opening = {
        "type": "segment",
        "segment": trainer.segment,
        "seed": trainer.batch_seed,
        "dropped": run.dropped,
        "changed": changed,
    }
    return _go(trainer, run, stop, {**opening, "identity": identity(trainer)})


def _go(trainer: Trainer, run: Run, stop: Stop | None, opening: dict) -> Trainer:
    """The run's loop from its opening record (`start` or `segment`) to its end or a stop."""
    config = trainer.config
    stop = Stop() if stop is None else stop
    last_record: dict = {}
    last_save = time.monotonic()

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
        last_save = time.monotonic()

    def run_eval(final: bool, baseline: bool = False) -> None:
        nonlocal last_record
        t0 = time.perf_counter()
        result = evaluate(trainer.policy, config, baseline=baseline)
        rate = result["sampled"]["win_rate"]
        improved = rate > trainer.best
        trainer.best = max(trainer.best, rate)
        last_record = {
            "type": "eval",
            "final": final,
            **counters(),
            "eval_s": time.perf_counter() - t0,
            "best": trainer.best,
            **result,
        }
        run.emit(last_record)
        if run.out is not None and improved:
            save(run.out / "best.pt", trainer.network(win_rate=rate))

    with stop:
        try:
            run.emit({**opening, **counters()})
            if opening["type"] == "start" and config.eval_start:
                run_eval(final=False, baseline=True)
            checkpoint()
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
                    trainer.snapshot()
                    run.emit({"type": "snapshot", **counters(), "pool": len(trainer.pool)})
                if trainer.evaluations.due(trainer.samples):
                    run_eval(final=False)
                    checkpoint()
                elif time.monotonic() - last_save >= 60 * config.checkpoint_minutes:
                    checkpoint()
            if stop.requested:
                run.emit({"type": "stop", **counters()})
                checkpoint()
                return trainer
            if last_record.get("update") == trainer.updates:
                run.emit({**last_record, "final": True, "repeated": True})
            else:
                run_eval(final=True)
            run.emit({"type": "end", "reason": "budget", **counters()})
            checkpoint(finished=True)
        finally:
            run.close()
    return trainer
