"""The cloning bench (E25 T16, increment 5, AI-Q14; `docs/epics/E25-log.md`): the distillation
bench's clone mode, whose teacher is the fair system actor, a warm start for PPO (E25 decision 3).

    uv run --group train python -m h3env.train.distill --teacher system \\
        --config configs/distill/e25-clone.toml --out runs/clone-1

The fair system actor (`system`, `h3_battle_ai::choose_fair`) plays every external seat of the
training distribution of the bench's `run` config (its kinds: the player kinds' both sides, the
neutral kinds' attacker against the fair defender inside the call; its step limit and reward
settings), on the bench's `seed`: its choice for each side to move comes as the action an
external seat plays (`BattleBatch.choice`, C ABI v37), drawn from that side's seat stream, so the
episodes are the ones it plays inside the call. A choice no batch action plays (the count
`lost`; none expected) is replaced by a uniformly random legal action and left out.

The samples are whole episodes (`Episodes`): a decision's value target is its side's return
by Monte Carlo at the run's settings, PPO's targets with λ 1 (γ 1): reward v1 at the end, minus
`truncation_penalty` for a cut battle, and with `shaping` minus the potential at the decision
(0 at the side's first, as `ppo.Trainer.shape` sums). Every student (a `[students.<name>]`
table: `[network]` keys over the run config's) trains on the same samples, `epochs` passes in
minibatches of `minibatch` rows over each batch of at least `envs` × `rollout` decisions: the
cross-entropy of its entry head on the teacher's entry over the legal entries, plus for a cast
row its spell, target and second heads along the teacher's choices (summed over the cast rows
and divided by all rows), plus `value_coef` times half the squared error against the returns.
Adam, the learning rate falling linearly to `lr_floor` of `lr` over `samples`.

Every `eval_every` samples and at the end each student scores on a held-out set (`heldout`
decisions of whole episodes on the seed after `seed`, collected once): the cross-entropy per
head, the agreement of its most likely entry with the teacher's (overall and per category of the
teacher's entry, `distill.CATEGORIES`), of its cast heads' most likely choices along the
teacher's, and its value's mean squared error and explained variance against the returns, with
its training seconds. At the end each student plays its ladder (`ladder`: the opponents on the
report set v1, the neutral set v1 beside them; the sampled policy): how near the clone plays to
the fair actor, which scores 0.5 against itself. `report.json` (the distillation bench's layout,
the teacher's statistics and the ladders added), `metrics.jsonl` and `students/<name>.pt` (run
network files of the run config, the bench's seed as theirs: `--init-from` starts a run from one)
as the distillation bench writes them. A stopped bench (Ctrl-C) scores and saves what it has; it
does not resume.
"""

from __future__ import annotations

import dataclasses
import json
import time
from collections.abc import Callable
from dataclasses import asdict
from pathlib import Path
from typing import Any

import numpy as np
import torch

from .. import _ffi
from ..batch import BattleBatch
from . import layout as L
from .distill import CATEGORIES, NETWORK_KEYS, Bench, Student, category, heads, student_config
from .evaluate import HELD_OUT_BATCH_SEEDS, compact, held_out_seeds, ladder
from .policy import CASTS, Policy, build
from .ppo import _SEATS, CHECKPOINT_FORMAT, TEACHER, Staging, device_of, episode_entries, save
from .run import Config, ConfigError, HeldOut, atomic_write_text, load_config, machine, source

#: The per-sample arrays a student trains on, the teacher's choices among them.
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
    "returns",
)


class Episodes:
    """The fair system actor's decisions on a run config's training distribution (the module's
    documentation), kept by whole episodes: `take` plays until enough decisions of ended
    episodes have their returns, `release` drops them (the open episodes' decisions stay)."""

    def __init__(self, config: Config, seed: int, envs: int, threads: int, rows: int = 0) -> None:
        c = self.config = config
        if seed in HELD_OUT_BATCH_SEEDS:
            raise ConfigError(f"the seed {seed:#x} is a held-out set's")
        b = self.batch = BattleBatch(
            envs,
            threads=threads,
            seed=seed,
            max_steps=c.max_steps,
            currency=c.currency,
            mana_price=c.mana_price,
            potential=c.shaping,
        )
        for weight, kinds, side in episode_entries(c):
            b.add_encounters(weight, kinds=kinds, **({} if side < 0 else _SEATS[side]))
        self.held_out = HeldOut(held_out_seeds())
        b.reset()
        self.held_out.check(b.episode_seed)
        self.rng = np.random.default_rng(seed)
        self.arrays: dict[str, np.ndarray] = {}
        self._allocate(rows or envs * 64)
        #: Each env's rows of its current episode, in order.
        self.open: list[list[int]] = [[] for _ in range(envs)]
        #: Whether each (env, side) has decided in its current episode (its potential counts).
        self.seen = np.zeros((envs, 2), bool)
        self.size = 0
        self.complete: list[np.ndarray] = []
        self.completed = 0
        #: The teacher's statistics: decisions taken and lost, the decisions past the run's
        #: tactics bound (`policy.bounded` would end the phase there), episodes, truncations and
        #: the attacker's wins per encounter kind, and the seconds spent playing.
        self.stats: dict[str, Any] = {
            "decisions": 0,
            "lost": 0,
            "past_bound": 0,
            "episodes": 0,
            "truncated": 0,
            "by_kind": {},
            "seconds": 0.0,
        }

    def _allocate(self, rows: int) -> None:
        """Grows the arrays to `rows` rows, keeping what they hold."""
        b = self.batch
        shapes = {
            "obs": ((b.observation_len,), np.float32),
            "entry_mask": ((b.action_count,), bool),
            "entry": ((), np.int64),
            "cast": ((), bool),
            "spell_mask": ((b.spell_count,), bool),
            "spell": ((), np.int64),
            "target_mask": ((b.target_count,), bool),
            "target": ((), np.int64),
            "second_mask": ((b.target_count,), bool),
            "second": ((), np.int64),
            "returns": ((), np.float32),
            "side": ((), np.int8),
            "phi": ((), np.float32),
        }
        for name, (shape, dtype) in shapes.items():
            grown = np.zeros((rows, *shape), dtype)
            old = self.arrays.get(name)
            if old is not None:
                grown[: len(old)] = old
            self.arrays[name] = grown

    def step(self) -> None:
        """One batch call: every side to move plays the teacher's choice, kept as a row of its
        episode; an episode that ended sets its rows' returns."""
        b, c, a = self.batch, self.config, self.arrays
        start = time.perf_counter()
        to_move = b.to_move.copy()
        encoded, lost = b.choice(TEACHER)
        if lost:
            gone = (to_move >= 0) & (encoded == 0)
            chosen = b.actions.copy()
            b.random_actions(self.rng)
            chosen[gone] = b.actions[gone]
            b.actions[:] = chosen
        rows = np.nonzero((to_move >= 0) & (encoded == 1))[0]
        k = len(rows)
        if self.size + k > len(a["obs"]):
            self._allocate((self.size + k) * 5 // 4 + len(to_move))
        at = np.arange(self.size, self.size + k)
        side = to_move[rows].astype(np.intp)
        actions = b.actions[rows]
        a["obs"][at] = b.obs[rows]
        a["entry_mask"][at] = b.entry_mask[rows]
        for name in ("entry", "spell", "target", "second"):
            a[name][at] = actions[name]
        cast = np.isin(actions["entry"], CASTS)
        a["cast"][at] = cast
        if cast.any():
            full = b.actions
            entries = np.where(np.isin(full["entry"], CASTS), full["entry"], 0)
            spells = np.where(entries > 0, full["spell"], 0)
            targets = np.where(entries > 0, full["target"], 0)
            on = rows[cast]
            a["spell_mask"][at[cast]] = b.spell_mask(entries)[on]
            a["target_mask"][at[cast]] = b.target_mask(entries, spells)[on]
            a["second_mask"][at[cast]] = b.second_mask(entries, spells, targets)[on]
        a["side"][at] = side
        a["phi"][at] = 0.0
        if b.potential is not None:
            a["phi"][at] = np.where(self.seen[rows, side], b.potential[rows, side], 0.0)
        self.seen[rows, side] = True
        bound = c.tactics_rounds
        if bound:
            past = (b.tactics_round[rows] > bound) & (actions["entry"] != L.END_TACTICS)
            self.stats["past_bound"] += int(past.sum())
        for env, row in zip(rows.tolist(), at.tolist(), strict=True):
            self.open[env].append(row)
        self.size += k
        kinds = b.encounter.copy()
        b.step()
        self.stats["decisions"] += k
        self.stats["lost"] += lost
        done = np.nonzero(b.done)[0]
        for env in done.tolist():
            mine = np.array(self.open[env], np.int64)
            self.open[env] = []
            self.seen[env] = False
            reward = b.reward[env].astype(np.float32)
            truncated = bool(b.truncated[env])
            if truncated:
                reward = np.full(2, -c.truncation_penalty, np.float32)
            if len(mine):
                a["returns"][mine] = reward[a["side"][mine]] - a["phi"][mine]
                self.complete.append(mine)
                self.completed += len(mine)
            row = self.stats["by_kind"].setdefault(_ffi.ENCOUNTER_KINDS[int(kinds[env])], [0, 0, 0])
            row[0] += int(b.result[env]) == _ffi.RESULT_ATTACKER
            row[1] += truncated
            row[2] += 1
            self.stats["episodes"] += 1
            self.stats["truncated"] += truncated
        if len(done):
            self.held_out.check(b.episode_seed[done])
        self.stats["seconds"] += time.perf_counter() - start

    def take(self, minimum: int) -> tuple[dict[str, np.ndarray], np.ndarray]:
        """Plays until at least `minimum` decisions of ended episodes are kept; returns the
        arrays as [rows, 1, ...] (`Staging`'s [calls, envs, ...] shape) and those rows (valid
        until `release`)."""
        while self.completed < minimum:
            self.step()
        idx = np.sort(np.concatenate(self.complete))
        return {k: self.arrays[k][:, None] for k in FIELDS}, idx

    def release(self) -> None:
        """Drops the kept rows of ended episodes: the open episodes' rows move to the front."""
        keep = [np.array(o, np.int64) for o in self.open]
        moved = np.concatenate(keep) if keep else np.zeros(0, np.int64)
        for array in self.arrays.values():
            array[: len(moved)] = array[moved]
        offset = 0
        for env, rows in enumerate(keep):
            self.open[env] = list(range(offset, offset + len(rows)))
            offset += len(rows)
        self.size = len(moved)
        self.complete = []
        self.completed = 0

    def close(self) -> None:
        self.batch.close()


def cross_entropy(mine, m: dict[str, torch.Tensor]) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    """A student's cross-entropy on the teacher's choices of the minibatch `m` (its heads
    `mine`, `distill.heads`): the loss (the entry's mean plus each cast head's, summed over the
    cast rows and divided by all rows) and each head's per-row values."""
    s_entry, s_cast, _, _ = mine
    rows = s_entry.shape[0]
    per = {"entry": -s_entry.gather(1, m["entry"][:, None]).squeeze(1)}
    loss = per["entry"].mean()
    cast = torch.nonzero(m["cast"]).squeeze(1)
    for name, (logp, _) in zip(("spell", "target", "second"), s_cast, strict=True):
        per[name] = -logp.gather(1, m[name][cast][:, None]).squeeze(1)
        loss = loss + per[name].sum() / rows
    return loss, per


def evaluate(
    students: list[Student], data: dict[str, np.ndarray], device: torch.device, size: int = 512
) -> dict[str, Any]:
    """Every student's scores on the held-out `data` (the module's documentation)."""
    n = data["returns"].shape[0]
    heads_ = ("spell", "target", "second")
    sums = {
        s.name: {
            "ce": {},
            "agree": np.zeros(len(CATEGORIES)),
            "cast": dict.fromkeys(heads_, 0.0),
            "value": [],
        }
        for s in students
    }
    counts = np.zeros(len(CATEGORIES))
    staging = Staging(device)
    with torch.no_grad():
        for lo in range(0, n, size):
            m = staging.gather(data, np.arange(lo, min(n, lo + size)))
            cat = category(m["obs"], m["entry"]).cpu().numpy()
            counts += np.bincount(cat, minlength=len(CATEGORIES))
            rows = torch.nonzero(m["cast"]).squeeze(1)
            for s in students:
                mine = heads(s.net, m)
                _, per = cross_entropy(mine, m)
                row = sums[s.name]
                for head, values in per.items():
                    row["ce"][head] = row["ce"].get(head, 0.0) + float(values.sum())
                same = (mine[2].argmax(1) == m["entry"]).cpu().numpy()
                row["agree"] += np.bincount(cat, weights=same, minlength=len(CATEGORIES))
                for head, (logp, _) in zip(heads_, mine[1], strict=True):
                    row["cast"][head] += float((logp.argmax(1) == m[head][rows]).sum())
                row["value"].append(mine[3].cpu().numpy())
    y = data["returns"].reshape(-1)
    casts = int(data["cast"].sum())
    var = float(np.var(y))
    out: dict[str, Any] = {
        "heldout": n,
        "casts": casts,
        "categories": dict(zip(CATEGORIES, counts.astype(int).tolist(), strict=True)),
        "returns": {"mean": float(np.mean(y)), "variance": var},
        "students": {},
    }
    for s in students:
        row = sums[s.name]
        v = np.concatenate(row["value"])
        out["students"][s.name] = {
            "cross_entropy": {
                h: x / (n if h == "entry" else max(casts, 1)) for h, x in row["ce"].items()
            },
            "agreement": float(row["agree"].sum() / max(counts.sum(), 1)),
            "agreement_by_category": {
                c: (row["agree"][i] / counts[i] if counts[i] else None)
                for i, c in enumerate(CATEGORIES)
            },
            "cast_agreement": {h: (x / casts if casts else None) for h, x in row["cast"].items()},
            "value": {
                "mse": float(np.mean((v - y) ** 2)),
                "explained_variance": 1 - float(np.var(y - v)) / var if var else 0.0,
            },
            "seconds": s.seconds,
        }
    return out


def table(evaluation: dict[str, Any], samples: int) -> str:
    """An evaluation as a short text table."""
    lines = [
        f"at {samples:,} samples ({evaluation['heldout']:,} held out)",
        f"{'student':<14} {'CE entry':>9} {'CE cast':>8} {'agree':>6} {'move':>6} {'melee':>6} "
        f"{'shot':>6} {'value ev':>8} {'seconds':>8}",
    ]

    def pct(x):
        return "-" if x is None else f"{x:.3f}"

    for name, row in evaluation["students"].items():
        ce = row["cross_entropy"]
        cast = sum(ce.get(h, 0.0) for h in ("spell", "target", "second"))
        by = row["agreement_by_category"]
        lines.append(
            f"{name:<14} {ce['entry']:>9.4f} {cast:>8.4f} {row['agreement']:>6.3f} "
            f"{pct(by['move']):>6} {pct(by['melee']):>6} {pct(by['shot']):>6} "
            f"{row['value']['explained_variance']:>8.3f} {row['seconds']:>8.0f}"
        )
    return "\n".join(lines)


def run(
    bench: Bench,
    out: Path,
    log: Callable[[str], None] = print,
    config_text: str | None = None,
) -> dict[str, Any]:
    """Runs the clone mode of `bench` into `out` (the module's documentation; `config_text`: the
    run config's text, else read from `bench.run`); returns the report."""
    if not bench.run and config_text is None:
        raise ConfigError("the clone mode needs `run`, the run config whose distribution it plays")
    for seed in (bench.seed, bench.seed + 1):
        if seed in HELD_OUT_BATCH_SEEDS:
            raise ConfigError(f"the seed {seed:#x} is a held-out set's")
    if out.exists() and any(out.iterdir()):
        raise ConfigError(f"{out} is not empty: a bench does not resume")
    text = Path(bench.run).read_text(encoding="utf-8") if config_text is None else config_text
    base = dataclasses.replace(load_config(text), seed=bench.seed, system_share=0.0)
    (out / "students").mkdir(parents=True, exist_ok=True)
    device = device_of(bench.device)
    teacher = Episodes(
        base, bench.seed, bench.envs, bench.threads, bench.envs * (bench.rollout + 64)
    )
    torch.manual_seed(bench.seed)
    students = []
    for name, keys in bench.students.items():
        config = student_config(base, keys)
        net: Policy = build(asdict(config), teacher.batch).to(device)
        lr = float(keys.get("lr", bench.lr))
        students.append(
            Student(name, net, config, torch.optim.Adam(net.parameters(), lr=lr, eps=1e-5), lr)
        )
    versions = _ffi.versions()
    report: dict[str, Any] = {
        "teacher": {
            "system": TEACHER,
            "system_policy": versions["system_policy"],
            "run": bench.run,
            "config": {k: v for k, v in asdict(base).items() if k in NETWORK_KEYS},
            "kinds": base.kinds,
            "shaping": base.shaping,
            "truncation_penalty": base.truncation_penalty,
            "max_steps": base.max_steps,
        },
        "bench": asdict(bench),
        "identity": {
            "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            **source(),
            **versions,
            "torch": torch.__version__,
            "device": str(device),
            "gpu": torch.cuda.get_device_name(device) if device.type == "cuda" else None,
            **machine(),
        },
        "students": {
            s.name: {
                "network": {k: v for k, v in asdict(s.config).items() if k in NETWORK_KEYS},
                "lr": s.lr,
                "parameters": sum(p.numel() for p in s.net.parameters()),
            }
            for s in students
        },
        "evaluations": [],
        "ended": None,
    }
    log(f"held-out set: {bench.heldout:,} decisions of the fair actor on seed {bench.seed + 1}")
    held = Episodes(
        base, bench.seed + 1, bench.envs, bench.threads, bench.heldout + 64 * bench.envs
    )
    arrays, idx = held.take(bench.heldout)
    heldout = {
        k: v.reshape(-1, *v.shape[2:])[idx[: bench.heldout]][:, None] for k, v in arrays.items()
    }
    report["teacher"]["heldout"] = {k: v for k, v in held.stats.items() if k != "seconds"}
    held.close()
    del arrays
    staging = Staging(device)
    metrics = (out / "metrics.jsonl").open("a", encoding="utf-8")
    samples, next_eval = 0, 0

    def score() -> None:
        for s in students:
            s.net.eval()
        evaluation = {"samples": samples, **evaluate(students, heldout, device)}
        for s in students:
            s.net.train()
        report["evaluations"].append(evaluation)
        metrics.write(json.dumps(evaluation) + "\n")
        metrics.flush()
        log(table(evaluation, samples))

    stopped = False
    try:
        score()
        next_eval = bench.eval_every
        while samples < bench.samples:
            arrays, flat = teacher.take(bench.envs * bench.rollout)
            n = len(flat)
            frac = min(1.0, samples / bench.samples)
            rng = np.random.default_rng(samples)
            size = min(n, bench.minibatch)
            for _ in range(bench.epochs):
                order = rng.permutation(flat)
                for k in range(0, n - size + 1, size):
                    m = staging.gather(arrays, np.sort(order[k : k + size]))
                    for s in students:
                        if device.type == "cuda":
                            torch.cuda.synchronize(device)
                        start = time.perf_counter()
                        for g in s.optimizer.param_groups:
                            g["lr"] = s.lr * (1.0 - (1.0 - bench.lr_floor) * frac)
                        mine = heads(s.net, m)
                        loss, _ = cross_entropy(mine, m)
                        v_loss = 0.5 * ((mine[3] - m["returns"]) ** 2).mean()
                        s.optimizer.zero_grad()
                        (loss + bench.value_coef * v_loss).backward()
                        torch.nn.utils.clip_grad_norm_(s.net.parameters(), bench.max_grad_norm)
                        s.optimizer.step()
                        if device.type == "cuda":
                            torch.cuda.synchronize(device)
                        s.seconds += time.perf_counter() - start
            teacher.release()
            samples += n
            if samples >= next_eval:
                score()
                next_eval += bench.eval_every
        report["ended"] = "budget"
    except KeyboardInterrupt:
        stopped = True
        report["ended"] = "stop"
    finally:
        teacher.close()
    if stopped or not report["evaluations"] or report["evaluations"][-1]["samples"] != samples:
        score()
    metrics.close()
    stats = dict(teacher.stats)
    stats["by_kind"] = {
        kind: {"episodes": n, "attacker_won": won / n, "truncated": cut / n}
        for kind, (won, cut, n) in sorted(stats["by_kind"].items())
    }
    report["teacher"]["stats"] = stats
    for s in students:
        save(
            out / "students" / f"{s.name}.pt",
            {
                "format": CHECKPOINT_FORMAT,
                "config": asdict(s.config),
                "policy": s.net.state_dict(),
                "updates": 0,
                "samples": samples,
                "cloned_from": TEACHER,
            },
        )
    report["samples"] = samples
    if bench.ladder and not stopped:
        report["ladders"] = {}
        for s in students:
            s.net.eval()
            log(f"{s.name}: the ladder ({', '.join(bench.ladder)}; the neutral set)")
            rows = ladder(
                s.net,
                opponents=bench.ladder,
                per_kind=bench.ladder_per_kind,
                threads=bench.threads,
                max_steps=2000,
            )
            neutral = ladder(
                s.net,
                held_out="neutral-v1",
                per_kind=bench.ladder_per_kind,
                threads=bench.threads,
                max_steps=2000,
            )
            report["ladders"][s.name] = {
                "report-v1": {o: compact(r) for o, r in rows.items()},
                "neutral-v1": {o: compact(r) for o, r in neutral.items()},
            }
            for opponent, r in {**rows, "neutral": neutral["system"]}.items():
                log(
                    f"  against {opponent}: win rate {r['win_rate']:.3f} ± {r['interval']:.3f}, "
                    f"value {r['value']:+.3f}"
                )
    atomic_write_text(out / "report.json", json.dumps(report, indent=1) + "\n")
    return report
