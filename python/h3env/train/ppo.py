"""The smoke run's PPO (E05 T8, `docs/epics/E05-log.md` §8.3): self-play over a `BattleBatch`
of the encounter distribution, one network for both sides, one trajectory per (env, side).

Each call's sample is the acting side's decision (observation, masks, chosen heads, its
log-probability and value). `SeatBook` closes a side's transition at its next decision or at the
episode's end with the rewards in between (reward v1, terminal); the sample that opened it gets
that reward. GAE runs backwards over the rollout's calls with a carry per (env, side): a sample's
next value is its side's next sample's, 0 after a decided battle, the value of `final_obs[env,
side]` after a truncation, and the value of the batch's current observation for the side to move
when the rollout ends. A transition still open at the end (the side not to move) has no next value
yet and is left out of that update; the sample before it bootstraps from its value.
"""

from __future__ import annotations

import json
import os
import subprocess
import time
from collections.abc import Callable
from dataclasses import asdict, dataclass, field
from pathlib import Path

import numpy as np
import torch

from .. import _ffi
from ..batch import BattleBatch
from ..seats import SeatBook
from .evaluate import EVAL_SEED, episode_seeds, evaluate
from .policy import Choice, Policy


@dataclass
class Config:
    """Every setting of a run (§8.3; provisional), written into its `config.json`."""

    minutes: float = 60.0
    #: Stop after this many updates instead (0: the wall time decides).
    updates: int = 0
    envs: int = 256
    threads: int = field(default_factory=lambda: os.cpu_count() or 1)
    seed: int = 1
    max_steps: int = 2000
    #: The family's kind weights (`H3_ENCOUNTER_*` order); None: the distribution's defaults.
    kinds: list[int] | None = None
    currency: str = "ai_value"
    mana_price: int = 10
    rollout: int = 128
    gamma: float = 0.995
    lam: float = 0.95
    lr: float = 2.5e-4
    epochs: int = 4
    minibatches: int = 8
    clip: float = 0.2
    value_coef: float = 0.5
    entropy_coef: float = 0.01
    max_grad_norm: float = 0.5
    hidden: int = 512
    embed: int = 32
    #: Minutes between evaluations (and at the end); 0: only at the end.
    eval_every: float = 10.0
    eval_episodes: int = 1000
    eval_envs: int = 100
    eval_seed: int = EVAL_SEED
    #: Also evaluate greedy play (the criterion is the sampled policy's).
    eval_greedy: bool = True
    #: Evaluate the untrained network and Random itself on the same episodes first.
    eval_start: bool = True
    device: str = "auto"
    torch_threads: int = 0
    #: The share of episodes in which a frozen earlier network (from the pool) plays one side,
    #: drawn per episode (self-play otherwise; design §8.2, T8c).
    pool_share: float = 0.5
    #: Snapshots the pool holds; when full, a new one replaces a random one (ages stay spread).
    pool_size: int = 8
    #: Updates between snapshots (the first after this many).
    pool_every: int = 25


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


def commit() -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True, timeout=10
        )
        return out.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return "unknown"


class Trainer:
    """One run's state: the batch, the network, the optimizer, the open transitions."""

    def __init__(self, config: Config) -> None:
        self.config = c = config
        torch.manual_seed(c.seed)
        if c.torch_threads:
            torch.set_num_threads(c.torch_threads)
        self.device = device_of(c.device)
        self.batch = BattleBatch(
            c.envs,
            threads=c.threads,
            seed=c.seed,
            max_steps=c.max_steps,
            currency=c.currency,
            mana_price=c.mana_price,
        )
        self.batch.add_encounters(kinds=c.kinds)
        self.batch.reset()
        self.policy = Policy.for_batch(self.batch, c.hidden, c.embed).to(self.device)
        self.optimizer = torch.optim.Adam(self.policy.parameters(), lr=c.lr, eps=1e-5)
        self.generator = torch.Generator(device=self.device).manual_seed(c.seed)
        self.book = SeatBook(c.envs)
        self.rollout = Rollout(c.rollout, c.envs, self.batch)
        self.staging = Staging(self.device)
        #: Where each (env, side)'s open transition sits in the rollout (-1: none, or earlier).
        self.opened = np.full((c.envs, 2), -1, np.int64)
        self.train_seeds: set[int] = set(int(s) for s in self.batch.episode_seed)
        self.held_out = episode_seeds(c.eval_episodes, c.eval_envs, c.eval_seed)
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

    def collect(self) -> dict:
        """Plays one rollout; returns its episodes' statistics."""
        b, r, c = self.batch, self.rollout, self.config
        r.clear()
        self.opened.fill(-1)
        rows = np.arange(c.envs)
        results = np.zeros(_ffi.RESULT_DEFENDER_SURRENDERED + 1, np.int64)
        truncated = 0
        decisions = 0
        start = time.perf_counter()
        pool_episodes = pool_won = 0
        for t in range(c.rollout):
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
                self.episodes += len(done)
                self.train_seeds.update(int(x) for x in b.episode_seed[done])
                self.assign(done)
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
            "episodes": ended,
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

    def checkpoint(self, path: Path) -> None:
        torch.save(
            {
                "policy": self.policy.state_dict(),
                "optimizer": self.optimizer.state_dict(),
                "config": asdict(self.config),
                "updates": self.updates,
                "samples": self.samples,
            },
            path,
        )

    def held_out_check(self) -> bool:
        return self.held_out.isdisjoint(self.train_seeds)


def train(
    config: Config,
    out: Path | None = None,
    log: Callable[[dict], None] | None = None,
) -> Trainer:
    """Trains for `config.minutes` (or `config.updates`) with evaluations every
    `config.eval_every` minutes and at the end; writes `config.json`, `metrics.jsonl`,
    `checkpoint.pt` (the last evaluation's) and `best.pt` (the best sampled rate's) into `out`
    when given. The final evaluation reuses the last one if no update came between."""
    trainer = Trainer(config)
    if out is not None:
        out.mkdir(parents=True, exist_ok=True)
        meta = {
            "config": asdict(config),
            "commit": commit(),
            "abi": _ffi.ABI_VERSION,
            "torch": torch.__version__,
            "device": str(trainer.device),
            "observation_len": trainer.batch.observation_len,
            "parameters": sum(p.numel() for p in trainer.policy.parameters()),
        }
        (out / "config.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    metrics = None if out is None else (out / "metrics.jsonl").open("a", encoding="utf-8")

    def emit(record: dict) -> None:
        if metrics is not None:
            metrics.write(json.dumps(record) + "\n")
            metrics.flush()
        if log is not None:
            log(record)

    start = time.perf_counter()
    budget = config.minutes * 60.0
    last_eval = start
    last_record: dict = {}

    def run_eval(final: bool, baseline: bool = False) -> None:
        nonlocal last_eval, last_record
        held_out = trainer.held_out_check()
        if not held_out:
            raise RuntimeError("an evaluation episode's seed was played in training")
        t0 = time.perf_counter()
        result = evaluate(trainer.policy, config, baseline=baseline)
        rate = result["sampled"]["win_rate"]
        improved = rate > trainer.best
        trainer.best = max(trainer.best, rate)
        last_record = {
            "eval": True,
            "final": final,
            "update": trainer.updates,
            "samples": trainer.samples,
            "minutes": (time.perf_counter() - start) / 60,
            "held_out": held_out,
            "eval_s": time.perf_counter() - t0,
            "best": trainer.best,
            **result,
        }
        emit(last_record)
        if out is not None:
            trainer.checkpoint(out / "checkpoint.pt")
            if improved:
                trainer.checkpoint(out / "best.pt")
        last_eval = time.perf_counter()

    if config.eval_start:
        run_eval(final=False, baseline=True)
        start = last_eval = time.perf_counter()
    while True:
        elapsed = time.perf_counter() - start
        if config.updates:
            if trainer.updates >= config.updates:
                break
            progress = trainer.updates / config.updates
        else:
            if elapsed >= budget:
                break
            progress = elapsed / budget
        roll = trainer.collect()
        stats = trainer.update(config.lr * max(1.0 - progress, 0.05))
        now = time.perf_counter()
        emit(
            {
                "update": trainer.updates,
                "samples": trainer.samples,
                "episodes": trainer.episodes,
                "minutes": (now - start) / 60,
                "samples_per_s": stats["update_samples"] / (roll["rollout_s"] + stats["update_s"]),
                **roll,
                **stats,
            }
        )
        if config.pool_share > 0 and trainer.updates % config.pool_every == 0:
            trainer.snapshot()
        if config.eval_every and now - last_eval >= config.eval_every * 60:
            run_eval(final=False)
    if last_record.get("update") == trainer.updates:
        emit({**last_record, "final": True, "repeated": True})
    else:
        run_eval(final=True)
    if metrics is not None:
        metrics.close()
    return trainer
