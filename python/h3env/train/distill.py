"""The distillation bench (E25 T7b, increment 3, AI-Q11; `docs/epics/E25-log.md`): how well
networks of other kinds and sizes learn a trained network's policy and value, on the same
samples, before any of them trains by RL.

    uv run --group train python -m h3env.train.distill --teacher runs/e25-scale-1/best.pt \\
        --config configs/distill/e25.toml --out runs/distill-1

The teacher (a run's network file) plays itself on the training distribution of its run's config
(its kinds, tactics bound and shaping; the sampled policy) in PPO's rollouts (`ppo.Trainer`'s,
self-play only, on the bench's `seed`). Every student (a `[students.<name>]` table: `[network]`
keys over the teacher's) trains on the same rollouts, `epochs` passes in minibatches of `minibatch`
rows (the hex networks' activations do not fit PPO's few thousand rows on an 8 GB GPU), the
teacher's distributions computed once per minibatch: the loss is the KL divergence from the
teacher's entry head over the legal entries, plus for a cast row its spell, target and second
heads along the teacher's choices, plus `value_coef` times half the squared error against the
teacher's λ-returns (PPO's value targets: `ppo.gae` with the teacher's values, γ and λ). AdamW
(`optimizer`; `weight_decay` 0, the default, is Adam's steps), the learning rate rising linearly
from 0 over the first `warmup` samples (0: none) and falling linearly to `lr_floor` of `lr` over
`samples` (`learning_rate`); a student's table may set its own `lr`, `warmup` and
`weight_decay` (E25 T24). On CUDA cuDNN picks its fastest kernels (`bench_device`). Every
`eval_every` samples and at the end each student scores on a held-out set (`heldout` samples of
rollouts on the seed after `seed`, collected once): the KL per head, the agreement of its most
likely entry with the teacher's, overall and per category of the teacher's (`CATEGORIES`), and
its value's mean squared error and explained variance against the returns (the teacher's own
beside them), with its training seconds and its cost (`cost`: the microseconds a sample-pass,
the peak CUDA memory of a step). `report.json` holds the settings,
the teacher, each student's parameters and every evaluation; `metrics.jsonl` the evaluations as
they come; `students/<name>.pt` each student as a run's network file (`evaluate.load` and
`--init-from` read it). A stopped bench (Ctrl-C) scores and saves what it has; it does not
resume. No network that reads the kill preview takes part.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import os
import time
import tomllib
from collections.abc import Callable
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

import numpy as np
import torch

from . import layout as L
from .evaluate import OPPONENTS, REPORT_V1_ENVS
from .policy import Policy, build, masked, rows_of
from .ppo import CHECKPOINT_FORMAT, FIELDS, Staging, Trainer, device_of, gae, save
from .run import Config, ConfigError, atomic_write_text, machine, resolve, source

#: The teacher's entry categories the agreement is split by (`category`): a decision of the
#: tactics phase (round 0), then by its entry.
CATEGORIES = ("tactics", "wait_defend", "move", "melee", "shot", "heal", "cast", "other")
#: The `[network]` keys a student's table may set.
NETWORK_KEYS = frozenset(
    f.name for f in dataclasses.fields(Config) if f.metadata.get("table") == "network"
)
#: The optimizer's keys a student's table may set over the bench's (E25 T24: `warmup` and
#: `weight_decay` beside `lr`).
STUDENT_KEYS = frozenset({"lr", "warmup", "weight_decay"})


@dataclass
class Bench:
    """A bench's settings (its TOML file; the module's documentation)."""

    #: The rollouts' seed (the held-out set's is the next).
    seed: int = 7
    #: The training samples every student sees.
    samples: int = 4_000_000
    #: The held-out set's samples.
    heldout: int = 16_384
    eval_every: int = 500_000
    envs: int = 256
    threads: int = field(default_factory=lambda: os.cpu_count() or 1)
    rollout: int = 128
    epochs: int = 2
    #: The rows of a minibatch (the last of an epoch's, smaller, is left out).
    minibatch: int = 512
    lr: float = 3e-4
    lr_floor: float = 0.1
    #: The samples over which the learning rate rises from 0 to `lr` (E25 T24; 0: none).
    warmup: int = 0
    #: AdamW's decoupled weight decay of the matrices and kernels (E25 T24; 0: Adam's steps).
    weight_decay: float = 0.0
    value_coef: float = 0.5
    max_grad_norm: float = 0.5
    device: str = "auto"
    #: The clone mode's run config (`h3env.train.clone`, E25 T16): its network the students'
    #: and its training distribution the fair system actor's; empty for a teacher's network file.
    run: str = ""
    #: The clone mode's ladder of each student at the end: its opponents on the report set v1
    #: (none: no ladder), the neutral set v1 beside it, `ladder_per_kind` episodes of each kind.
    ladder: list[str] = field(default_factory=list)
    ladder_per_kind: int = REPORT_V1_ENVS
    #: Each student's `[network]` keys (and `STUDENT_KEYS`) over the teacher's, by name, in order.
    students: dict[str, dict[str, Any]] = field(default_factory=dict)


def load_bench(text: str, sets: list[str] | tuple[str, ...] = ()) -> Bench:
    """A bench from its TOML `text` (unknown keys refused), then each `--set key=value` (a
    top-level key, the value as TOML writes it)."""
    try:
        data = tomllib.loads(text)
        for item in sets:
            key, sep, value = item.partition("=")
            if not sep:
                raise ConfigError(f"--set takes key=value, not {item!r}")
            data[key.strip()] = tomllib.loads(f"v = {value.strip()}")["v"]
    except tomllib.TOMLDecodeError as e:
        raise ConfigError(f"the bench file is not TOML: {e}") from None
    known = {f.name for f in dataclasses.fields(Bench)}
    unknown = sorted(set(data) - known)
    if unknown:
        raise ConfigError(f"unknown bench keys {', '.join(unknown)}")
    bench = Bench(**data)
    problems = []
    for key in (
        "samples",
        "heldout",
        "eval_every",
        "envs",
        "rollout",
        "epochs",
        "minibatch",
        "ladder_per_kind",
    ):
        if getattr(bench, key) <= 0:
            problems.append(f"`{key}` must be positive")
    for key in ("warmup", "weight_decay"):
        if getattr(bench, key) < 0:
            problems.append(f"`{key}` must not be negative")
    if not bench.students:
        problems.append("no `[students.<name>]` table")
    unknown = [o for o in bench.ladder if o not in OPPONENTS]
    if unknown:
        problems.append(f"`ladder`: {', '.join(unknown)} are no opponents ({', '.join(OPPONENTS)})")
    for name, keys in bench.students.items():
        bad = sorted(set(keys) - NETWORK_KEYS - STUDENT_KEYS)
        if bad:
            problems.append(f"student {name!r}: {', '.join(bad)} are no `[network]` keys")
        if keys.get("preview"):
            problems.append(f"student {name!r}: no network that reads the kill preview")
        for key in ("warmup", "weight_decay"):
            if keys.get(key, 0) < 0:
                problems.append(f"student {name!r}: `{key}` must not be negative")
    if problems:
        raise ConfigError("; ".join(problems))
    return bench


def student_config(teacher: Config, keys: dict[str, Any]) -> Config:
    """The teacher's run config with a student's `[network]` keys (its file's config)."""
    net = {k: v for k, v in keys.items() if k not in STUDENT_KEYS}
    return resolve(dataclasses.replace(teacher, **net))


def category(obs: torch.Tensor, entry: torch.Tensor) -> torch.Tensor:
    """Each row's `CATEGORIES` index from its observation and the teacher's most likely entry."""
    out = torch.full_like(entry, CATEGORIES.index("other"))
    for name, lo, hi in (
        ("wait_defend", 0, L.MOVE_BASE),
        ("move", L.MOVE_BASE, L.ATTACK_BASE),
        ("melee", L.ATTACK_BASE, L.SHOOT_BASE),
        ("shot", L.SHOOT_BASE, L.HEAL_BASE),
        ("heal", L.HEAL_BASE, L.END_TACTICS),
        ("cast", L.END_TACTICS + 3, L.END_TACTICS + 5),
    ):
        out = torch.where((entry >= lo) & (entry < hi), CATEGORIES.index(name), out)
    return torch.where(obs[:, L.ROUND] == 0, CATEGORIES.index("tactics"), out)


def heads(net: Policy, m: dict[str, torch.Tensor]):
    """`net`'s log-probabilities on a minibatch `m` (the rollout's fields, bool masks): the entry
    head's [rows, entries], the cast rows' spell, target and second heads' along the stored
    choices, the masked entry logits and the value."""
    h = net.encode(m["obs"])
    value = net.value(h)
    logits = masked(net.entry_logits(h), m["entry_mask"])
    rows = torch.nonzero(m["cast"]).squeeze(1)
    hc, c = rows_of(h, rows), net.cast_of(m["entry"][rows])
    s, t = m["spell"][rows], m["target"][rows]
    cast = []
    for head_logits, mask in (
        (net.spell_logits(hc, c), m["spell_mask"][rows]),
        (net.target_logits(hc, c, s), m["target_mask"][rows]),
        (net.second_logits(hc, c, s, t), m["second_mask"][rows]),
    ):
        cast.append((torch.log_softmax(masked(head_logits, mask), dim=1), mask))
    return torch.log_softmax(logits, dim=1), cast, logits, value


def kl(teacher: torch.Tensor, student: torch.Tensor, mask: torch.Tensor) -> torch.Tensor:
    """Each row's KL divergence from the teacher's log-probabilities to the student's over the
    legal entries `mask`."""
    p = teacher.exp()
    return (p * (teacher - student)).masked_fill(~mask, 0.0).sum(1)


def losses(target, mine, entry_mask: torch.Tensor) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    """The KL terms of a student's heads `mine` against the teacher's `target` (`heads`; the
    rows' entry mask `entry_mask`): the loss (the entry's mean plus each cast head's, summed over
    the cast rows and divided by all rows) and each head's per-row KL."""
    t_entry, t_cast, _, _ = target
    s_entry, s_cast, _, _ = mine
    per = {"entry": kl(t_entry, s_entry, entry_mask)}
    rows = t_entry.shape[0]
    loss = per["entry"].mean()
    for name, (t, mask), (s, _) in zip(("spell", "target", "second"), t_cast, s_cast, strict=True):
        per[name] = kl(t, s, mask)
        loss = loss + per[name].sum() / rows
    return loss, per


@dataclass
class Student:
    """A student: its name, network, config, optimizer, learning rate and training seconds, its
    warm-up and weight decay, the rows it trained on (each pass counted) and the most CUDA memory
    a step of it took above what was allocated before the step (bytes; 0 on the CPU)."""

    name: str
    net: Policy
    config: Config
    optimizer: torch.optim.Optimizer
    lr: float
    seconds: float = 0.0
    warmup: int = 0
    weight_decay: float = 0.0
    rows: int = 0
    peak: int = 0


def optimizer(net: Policy, lr: float, weight_decay: float = 0.0) -> torch.optim.Optimizer:
    """A student's AdamW (Adam's ε 1e-5, as every bench's): the decoupled `weight_decay` on its
    matrices and kernels (a parameter of two dimensions or more: the layers' weights, the
    embeddings, the attention's offset bias), none on its biases and norms' gains; at 0 Adam's
    steps exactly (every bench before E25 T24)."""
    if not weight_decay:
        return torch.optim.AdamW(net.parameters(), lr=lr, eps=1e-5, weight_decay=0.0)
    params = list(net.parameters())
    groups = [
        {"params": [p for p in params if p.ndim >= 2], "weight_decay": weight_decay},
        {"params": [p for p in params if p.ndim < 2], "weight_decay": 0.0},
    ]
    return torch.optim.AdamW(groups, lr=lr, eps=1e-5)


def make_student(
    name: str, keys: dict[str, Any], base: Config, bench: Bench, batch, device: torch.device
) -> Student:
    """The student of the table `keys` (its `[network]` keys over `base`, its optimizer's over
    the bench's), its network built for `batch` on `device`."""
    config = student_config(base, keys)
    net = build(asdict(config), batch).to(device)
    lr = float(keys.get("lr", bench.lr))
    decay = float(keys.get("weight_decay", bench.weight_decay))
    return Student(
        name,
        net,
        config,
        optimizer(net, lr, decay),
        lr,
        warmup=int(keys.get("warmup", bench.warmup)),
        weight_decay=decay,
    )


def learning_rate(s: Student, bench: Bench, samples: int, seen: float) -> float:
    """`s`'s learning rate at a minibatch: its `lr` falling linearly to `lr_floor` of it over the
    bench's `samples` by `samples`, the samples before the minibatch's batch (as before E25 T24),
    times its warm-up, rising linearly from 0 to 1 over its first `warmup` samples by `seen`, the
    samples into the training at the minibatch (a batch's counted pro rata over its minibatches)."""
    fall = s.lr * (1.0 - (1.0 - bench.lr_floor) * min(1.0, samples / bench.samples))
    return fall if seen >= s.warmup else fall * seen / s.warmup


def step(
    s: Student,
    loss: Callable[[Policy], torch.Tensor],
    rate: float,
    max_grad_norm: float,
    rows: int,
    device: torch.device,
) -> None:
    """One optimizer step of `s` at the learning rate `rate` on the `loss` of its network over a
    minibatch of `rows` rows, the gradient's norm clipped to `max_grad_norm`: its seconds and rows
    counted, on CUDA its peak memory above what was allocated before the step."""
    cuda = device.type == "cuda"
    if cuda:
        torch.cuda.synchronize(device)
        base = torch.cuda.memory_allocated(device)
        torch.cuda.reset_peak_memory_stats(device)
    start = time.perf_counter()
    for g in s.optimizer.param_groups:
        g["lr"] = rate
    value = loss(s.net)
    s.optimizer.zero_grad()
    value.backward()
    torch.nn.utils.clip_grad_norm_(s.net.parameters(), max_grad_norm)
    s.optimizer.step()
    if cuda:
        torch.cuda.synchronize(device)
        s.peak = max(s.peak, torch.cuda.max_memory_allocated(device) - base)
    s.seconds += time.perf_counter() - start
    s.rows += rows


def cost(s: Student) -> dict[str, float | None]:
    """`s`'s cost so far (E25 T24): the microseconds of a sample-pass (a row's share of a step:
    forward, backward and the optimizer; None before any) and the peak memory of a step on CUDA
    (MB; None on the CPU)."""
    return {
        "us_per_sample_pass": s.seconds * 1e6 / s.rows if s.rows else None,
        "peak_memory_mb": s.peak / 2**20 if s.peak else None,
    }


def described(s: Student) -> dict[str, Any]:
    """`s` in a report: its `[network]` keys, its optimizer's settings, its parameters."""
    return {
        "network": {k: v for k, v in asdict(s.config).items() if k in NETWORK_KEYS},
        "lr": s.lr,
        "warmup": s.warmup,
        "weight_decay": s.weight_decay,
        "parameters": sum(p.numel() for p in s.net.parameters()),
    }


def bench_device(name: str) -> torch.device:
    """The bench's device (`ppo.device_of`), with cuDNN's autotuner on CUDA (E25 T24: the
    minibatches keep their shape, so each convolution's fastest kernel is found once)."""
    device = device_of(name)
    if device.type == "cuda":
        torch.backends.cudnn.benchmark = True
    return device


def steps_of(n: int, size: int, epochs: int) -> int:
    """The minibatches of `epochs` passes over `n` rows in minibatches of `size` (the last,
    smaller one of each pass left out)."""
    return epochs * len(range(0, n - size + 1, size))


def collect(trainer: Trainer) -> tuple[dict[str, np.ndarray], np.ndarray]:
    """One rollout of the teacher (`trainer.policy`): the rollout's `FIELDS`, its values and
    λ-returns (`returns`), [calls, envs, ...] (the rollout's own arrays: the next rollout
    overwrites them), and the used samples' flat indices."""
    trainer.collect()
    c, r = trainer.config, trainer.rollout
    adv, used = gae(
        r.side,
        r.value,
        r.reward,
        r.closed,
        r.terminal,
        r.truncated,
        r.final_value,
        trainer.last_side,
        trainer.last_value,
        c.gamma,
        c.lam,
    )
    arrays = {name: getattr(r, name) for name in FIELDS}
    arrays["returns"] = adv + r.value
    return arrays, np.nonzero(used.reshape(-1))[0]


def copied(arrays: dict[str, np.ndarray], flat: np.ndarray) -> dict[str, np.ndarray]:
    """The samples `flat` of a rollout's `arrays`, copied, as [samples, 1, ...] (`Staging`'s
    [calls, envs, ...] shape)."""
    return {k: v.reshape(-1, *v.shape[2:])[flat][:, None] for k, v in arrays.items()}


def evaluate(
    teacher: Policy,
    students: list[Student],
    data: dict[str, np.ndarray],
    device: torch.device,
    size: int = 512,
) -> dict[str, Any]:
    """Every student's scores on the held-out `data` (the module's documentation) and the
    teacher's value's."""
    n = data["returns"].shape[0]
    sums = {s.name: {"kl": {}, "agree": np.zeros(len(CATEGORIES)), "value": []} for s in students}
    counts = np.zeros(len(CATEGORIES))
    teacher_value = []
    staging = Staging(device)
    arrays = {k: data[k] for k in FIELDS}
    with torch.no_grad():
        for lo in range(0, n, size):
            m = staging.gather(arrays, np.arange(lo, min(n, lo + size)))
            target = heads(teacher, m)
            best = target[2].argmax(1)
            cat = category(m["obs"], best)
            counts += np.bincount(cat.cpu().numpy(), minlength=len(CATEGORIES))
            teacher_value.append(target[3].cpu().numpy())
            for s in students:
                mine = heads(s.net, m)
                _, per = losses(target, mine, m["entry_mask"])
                row = sums[s.name]
                for head, values in per.items():
                    row["kl"][head] = row["kl"].get(head, 0.0) + float(values.sum())
                same = (mine[2].argmax(1) == best).cpu().numpy()
                row["agree"] += np.bincount(cat.cpu().numpy(), weights=same, minlength=len(counts))
                row["value"].append(mine[3].cpu().numpy())
    y = data["returns"].reshape(-1)
    casts = int(data["cast"].sum())

    def value_scores(v: np.ndarray) -> dict[str, float]:
        var = float(np.var(y))
        mse = float(np.mean((v - y) ** 2))
        return {"mse": mse, "explained_variance": 1 - float(np.var(y - v)) / var if var else 0.0}

    out: dict[str, Any] = {
        "heldout": n,
        "casts": casts,
        "categories": dict(zip(CATEGORIES, counts.astype(int).tolist(), strict=True)),
        "teacher": value_scores(np.concatenate(teacher_value)),
        "students": {},
    }
    for s in students:
        row = sums[s.name]
        kl_ = {h: v / (n if h == "entry" else max(casts, 1)) for h, v in row["kl"].items()}
        agree = {
            c: (row["agree"][i] / counts[i] if counts[i] else None)
            for i, c in enumerate(CATEGORIES)
        }
        out["students"][s.name] = {
            "kl": kl_,
            "agreement": float(row["agree"].sum() / max(counts.sum(), 1)),
            "agreement_by_category": agree,
            "value": value_scores(np.concatenate(row["value"])),
            "seconds": s.seconds,
            **cost(s),
        }
    return out


#: A table's cost columns (`cost_columns`): the microseconds a sample-pass, the peak MB.
COST_HEADER = f" {'µs':>6} {'MB':>6}"


def cost_columns(row: dict[str, Any]) -> str:
    """A student's evaluation's cost (`cost`) as a table's columns (`COST_HEADER`; "-" for none,
    and for a report written before E25 T24)."""
    out = ""
    for key in ("us_per_sample_pass", "peak_memory_mb"):
        x = row.get(key)
        out += f" {'-' if x is None else f'{x:.0f}':>6}"
    return out


def table(evaluation: dict[str, Any], samples: int) -> str:
    """An evaluation as a short text table."""
    lines = [
        f"at {samples:,} samples ({evaluation['heldout']:,} held out; the teacher's value: "
        f"explained variance {evaluation['teacher']['explained_variance']:.3f})",
        f"{'student':<14} {'KL entry':>9} {'KL cast':>8} {'agree':>6} {'move':>6} {'melee':>6} "
        f"{'value ev':>8} {'seconds':>8}{COST_HEADER}",
    ]
    for name, row in evaluation["students"].items():
        k = row["kl"]
        cast = sum(k.get(h, 0.0) for h in ("spell", "target", "second"))
        by = row["agreement_by_category"]

        def pct(x):
            return "-" if x is None else f"{x:.3f}"

        lines.append(
            f"{name:<14} {k['entry']:>9.4f} {cast:>8.4f} {row['agreement']:>6.3f} "
            f"{pct(by['move']):>6} {pct(by['melee']):>6} "
            f"{row['value']['explained_variance']:>8.3f} {row['seconds']:>8.0f}{cost_columns(row)}"
        )
    return "\n".join(lines)


def run(
    teacher_path: Path,
    bench: Bench,
    out: Path,
    log: Callable[[str], None] = print,
) -> dict[str, Any]:
    """Runs `bench` with the teacher of `teacher_path` into `out` (the module's documentation);
    returns the report."""
    if bench.run or bench.ladder:
        raise ConfigError("`run` and `ladder` are the clone mode's (--teacher system)")
    if out.exists() and any(out.iterdir()):
        raise ConfigError(f"{out} is not empty: a bench does not resume")
    (out / "students").mkdir(parents=True, exist_ok=True)
    saved = torch.load(teacher_path, map_location="cpu", weights_only=True)
    if saved["config"].get("preview"):
        raise ConfigError("the teacher reads the kill preview: no bench network does")
    base = resolve(Config(**saved["config"]))
    device = bench_device(bench.device)
    play = dataclasses.replace(
        base,
        seed=bench.seed,
        envs=bench.envs,
        threads=bench.threads,
        rollout=bench.rollout,
        self_play=1.0,
        system_share=0.0,
        device=str(device),
    )

    def teacher_trainer(seed: int) -> Trainer:
        t = Trainer(resolve(dataclasses.replace(play, seed=seed)))
        t.policy.load_state_dict(saved["policy"])
        t.policy.eval()
        return t

    trainer = teacher_trainer(bench.seed)
    teacher = trainer.policy
    torch.manual_seed(bench.seed)
    students = [
        make_student(name, keys, base, bench, trainer.batch, device)
        for name, keys in bench.students.items()
    ]
    report: dict[str, Any] = {
        "teacher": {
            "path": str(teacher_path),
            "config": {k: v for k, v in asdict(base).items() if k in NETWORK_KEYS},
            "samples": saved.get("samples"),
            "parameters": sum(p.numel() for p in teacher.parameters()),
        },
        "bench": asdict(bench),
        "identity": {
            "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            **source(),
            "torch": torch.__version__,
            "device": str(device),
            "gpu": torch.cuda.get_device_name(device) if device.type == "cuda" else None,
            **machine(),
        },
        "students": {s.name: described(s) for s in students},
        "evaluations": [],
        "ended": None,
    }
    log(f"held-out set: {bench.heldout:,} samples of the teacher on seed {bench.seed + 1}")
    held = teacher_trainer(bench.seed + 1)
    parts: list[dict[str, np.ndarray]] = []
    while sum(p["returns"].shape[0] for p in parts) < bench.heldout:
        parts.append(copied(*collect(held)))
    held.batch.close()
    heldout = {k: np.concatenate([p[k] for p in parts])[: bench.heldout] for k in parts[0]}
    del parts
    staging = Staging(device)
    metrics = (out / "metrics.jsonl").open("a", encoding="utf-8")
    samples, next_eval = 0, 0

    def score() -> None:
        for s in students:
            s.net.eval()
        evaluation = {"samples": samples, **evaluate(teacher, students, heldout, device)}
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
            arrays, flat = collect(trainer)
            n = len(flat)
            rng = np.random.default_rng(samples)
            size = min(n, bench.minibatch)
            done, steps = 0, steps_of(n, size, bench.epochs)
            for _ in range(bench.epochs):
                order = rng.permutation(flat)
                for k in range(0, n - size + 1, size):
                    m = staging.gather(arrays, np.sort(order[k : k + size]))
                    with torch.no_grad():
                        target = heads(teacher, m)

                    def loss(net: Policy, m=m, target=target) -> torch.Tensor:
                        mine = heads(net, m)
                        kl_, _ = losses(target, mine, m["entry_mask"])
                        v_loss = 0.5 * ((mine[3] - m["returns"]) ** 2).mean()
                        return kl_ + bench.value_coef * v_loss

                    done += 1
                    for s in students:
                        rate = learning_rate(s, bench, samples, samples + n * done / steps)
                        step(s, loss, rate, bench.max_grad_norm, size, device)
            samples += n
            if samples >= next_eval:
                score()
                next_eval += bench.eval_every
        report["ended"] = "budget"
    except KeyboardInterrupt:
        stopped = True
        report["ended"] = "stop"
    finally:
        trainer.batch.close()
    if stopped or not report["evaluations"] or report["evaluations"][-1]["samples"] != samples:
        score()
    metrics.close()
    for s in students:
        save(
            out / "students" / f"{s.name}.pt",
            {
                "format": CHECKPOINT_FORMAT,
                "config": asdict(s.config),
                "policy": s.net.state_dict(),
                "updates": 0,
                "samples": samples,
                "distilled_from": str(teacher_path),
            },
        )
    report["samples"] = samples
    atomic_write_text(out / "report.json", json.dumps(report, indent=1) + "\n")
    return report


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="python -m h3env.train.distill", description=__doc__)
    parser.add_argument(
        "--teacher",
        required=True,
        help="a run's network file, or `system`: the fair system actor (the clone mode, E25 T16)",
    )
    parser.add_argument("--config", type=Path, required=True, help="the bench's TOML file")
    parser.add_argument("--out", type=Path, required=True, help="a new directory")
    parser.add_argument("--set", action="append", default=[], metavar="KEY=VALUE")
    args = parser.parse_args(argv)
    if args.teacher == "system-all-seeing":
        parser.error("the all-seeing system actor is never imitated (E25 decision 3)")
    bench = load_bench(args.config.read_text(encoding="utf-8"), args.set)
    if args.teacher == "system":
        from . import clone

        clone.run(bench, args.out)
    else:
        run(Path(args.teacher), bench, args.out)


if __name__ == "__main__":
    main()
