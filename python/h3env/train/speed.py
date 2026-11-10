"""The update's speed (E25 T31, increment 8's decision 3): one PPO update of a run's network,
timed by its phases and under candidate changes, on one rollout.

    uv run --group train python -m h3env.train.speed --config configs/e25-imitate-structure.toml \
        --init-from runs/e25-structure-1/best.pt --device cuda --json runs/t31-speed.json

The config's trainer (`--config`, `--set`, as a run's; the learner from `--init-from`) plays one
rollout, with the imitation loss's labels when the config's weight is above 0 at its start. Each
variant (`VARIANTS`; `--variants` names a subset, the baseline always first) then updates from
the same weights, optimizer state and counters, `--repeats` times, over `--epochs` of the
config's epochs (each epoch costs the same: 1 is a quarter of a run's update at 4): the seconds of
each update (the first a warm-up: the autotuner's search for its shapes, the allocator's growth;
the median of the others counts), the microseconds a sample-pass (an update's seconds over its
rows times its epochs), the GPU's peak memory, the losses, and how far the weights after the
first update lie from the baseline's, beside how far the baseline's update moved them (a variant
changes the speed, not the update: its difference is rounding's). A variant is a change of the
update's code or settings the tool makes for its updates alone (`Variant`): the code of a run is
the baseline.

Then (but with `--no-phases`) one more update of the baseline runs with the device synchronised
around each of its phases (`PHASES`: the minibatches' gather, the encoding with its trunk, the
entry head and the value, PPO's heads along the action, the teacher's, the backward pass, the
clipping, the optimizer's step; the rest is the update's own code): each phase's seconds and
share of that update. `--trace` runs torch's profiler over the baseline's first `--trace-steps`
optimizer steps and keeps its operators with the most self time on the device (the CPU's off
CUDA). The report is printed as Markdown and, with `--json`, written whole.
"""

from __future__ import annotations

import argparse
import contextlib
import copy
import dataclasses
import json
import time
from collections.abc import Callable, Iterator
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import numpy as np
import torch

from . import hexnet
from .ppo import Trainer
from .run import Config, load_config, resolve

#: The update's losses a variant must reproduce (the update's statistics, `Trainer.update`).
LOSSES = ("policy_loss", "value_loss", "entropy", "kl", "clip", "imitation_loss")


@dataclass(frozen=True)
class Variant:
    """A candidate change of the update: cuDNN's autotuner (`torch.backends.cudnn.benchmark`)
    during it, the hex convolution trunk's tensors in the contiguous (NCHW) layout instead of the
    channels-last one its drawing hands it (`nchw`: `contiguous_trunk`), and settings of the
    config that change no loss (`settings`, e.g. `[ppo] accumulate`)."""

    autotune: bool = False
    nchw: bool = False
    settings: dict[str, Any] = field(default_factory=dict)


#: The candidates, the baseline first (the run's code: no autotuner, the trunk as drawn).
VARIANTS: dict[str, Variant] = {
    "baseline": Variant(),
    "autotune": Variant(autotune=True),
    "nchw": Variant(nchw=True),
    "nchw+autotune": Variant(autotune=True, nchw=True),
    "autotune+accumulate/2": Variant(autotune=True, settings={"accumulate": 0.5}),
    "nchw+autotune+accumulate/2": Variant(autotune=True, nchw=True, settings={"accumulate": 0.5}),
}

#: The phases of the phases' update, by the callable each wraps (`Phases.install`).
PHASES = (
    "gather",
    "encode",
    "trunk",
    "entry and value",
    "heads",
    "teacher heads",
    "backward",
    "clip",
    "step",
)


@contextlib.contextmanager
def contiguous_trunk() -> Iterator[None]:
    """The hex convolution trunk in the contiguous layout: each residual block's input made
    contiguous (the drawing's grid, a transposed view, is channels-last) and each `CellNorm`'s
    output too (its permutes give a channels-last tensor); the convolutions then run on NCHW
    tensors."""
    block, norm = hexnet.ConvBlock.forward, hexnet.CellNorm.forward

    def block_forward(self, x, cells, real):
        return block(self, x.contiguous(), cells, real)

    def norm_forward(self, x):
        return norm(self, x).contiguous()

    hexnet.ConvBlock.forward, hexnet.CellNorm.forward = block_forward, norm_forward
    try:
        yield
    finally:
        hexnet.ConvBlock.forward, hexnet.CellNorm.forward = block, norm


def settings_of(config: Config, settings: dict[str, Any]) -> Config:
    """`config` with a variant's settings: a float multiplies the config's integer value (at
    least 1), anything else replaces it."""
    changes = {}
    for key, value in settings.items():
        old = getattr(config, key)
        changes[key] = max(1, int(old * value)) if isinstance(value, float) else value
    return dataclasses.replace(config, **changes)


def chunk_rows(rows: int, config: Config) -> list[int]:
    """The row counts of an update's chunks over `rows` rows (`Trainer.update`'s minibatches and
    their split): the shapes cuDNN's autotuner searches once each."""
    size = max(2, rows // config.minibatches)
    chunks = max(1, min(config.accumulate, size // 2))
    return sorted({len(part) for part in np.array_split(np.arange(size), chunks)})


class Phases:
    """The seconds and calls of each phase of `PHASES`, the device synchronised before and after
    each call (`install` wraps the callables, `remove` restores them)."""

    def __init__(self, device: torch.device) -> None:
        cuda = device.type == "cuda"
        self.sync: Callable[[], None] = torch.cuda.synchronize if cuda else (lambda: None)
        self.seconds: dict[str, float] = {name: 0.0 for name in PHASES}
        self.calls: dict[str, int] = {name: 0 for name in PHASES}
        self._undo: list[Callable[[], None]] = []

    def wrap(self, name: str | Callable[..., str], fn: Callable) -> Callable:
        def timed(*args, **kwargs):
            phase = name(*args, **kwargs) if callable(name) else name
            self.sync()
            start = time.perf_counter()
            try:
                return fn(*args, **kwargs)
            finally:
                self.sync()
                self.seconds[phase] += time.perf_counter() - start
                self.calls[phase] += 1

        return timed

    def install(self, trainer: Trainer) -> None:
        policy, staging, optimizer = trainer.policy, trainer.staging, trainer.optimizer

        def instance(owner: Any, attr: str, name: str | Callable[..., str]) -> None:
            setattr(owner, attr, self.wrap(name, getattr(owner, attr)))
            self._undo.append(lambda: delattr(owner, attr))

        def module(owner: Any, attr: str, name: str) -> None:
            old = getattr(owner, attr)
            setattr(owner, attr, self.wrap(name, old))
            self._undo.append(lambda: setattr(owner, attr, old))

        instance(staging, "gather", "gather")
        instance(policy, "encode", "encode")
        if hasattr(policy, "trunk"):
            instance(policy, "trunk", "trunk")
        instance(policy, "entry_logits", "entry and value")
        instance(policy, "value", "entry and value")

        def heads(*args, with_entropy: bool = True, **kwargs) -> str:
            return "heads" if with_entropy else "teacher heads"

        instance(policy, "along", heads)
        instance(optimizer, "step", "step")
        module(torch.Tensor, "backward", "backward")
        module(torch.nn.utils, "clip_grad_norm_", "clip")

    def remove(self) -> None:
        while self._undo:
            self._undo.pop()()

    def report(self, total: float) -> dict[str, Any]:
        """Each phase's seconds, calls and share of `total` (the update's seconds), the trunk
        inside the encoding (its share apart, the encoding's without it), and the rest."""
        s = dict(self.seconds)
        s["encode"] -= s["trunk"]
        top = sum(v for k, v in s.items())
        out = {
            name: {"seconds": s[name], "calls": self.calls[name], "share": s[name] / total}
            for name in PHASES
        }
        out["other"] = {"seconds": total - top, "calls": 0, "share": (total - top) / total}
        return out


class Enough(Exception):
    """Ends a traced update after its first steps."""


@contextlib.contextmanager
def variant_of(trainer: Trainer, base: Config, v: Variant) -> Iterator[None]:
    """The trainer under the variant `v` (its config `base` with the variant's settings)."""
    trainer.config = settings_of(base, v.settings)
    with contextlib.ExitStack() as stack:
        stack.enter_context(torch.backends.cudnn.flags(enabled=True, benchmark=v.autotune))
        if v.nchw:
            stack.enter_context(contiguous_trunk())
        try:
            yield
        finally:
            trainer.config = base


def weights(trainer: Trainer) -> torch.Tensor:
    return torch.cat([p.detach().reshape(-1) for p in trainer.policy.parameters()]).cpu()


def environment(device: torch.device) -> dict[str, Any]:
    """What the timings ran on."""
    out: dict[str, Any] = {
        "torch": torch.__version__,
        "device": str(device),
        "threads": torch.get_num_threads(),
    }
    if device.type == "cuda":
        out["gpu"] = torch.cuda.get_device_name(device)
        out["cuda"] = torch.version.cuda
        out["cudnn"] = torch.backends.cudnn.version()
    return out


def profile(
    config: Config,
    init_from: Path | None = None,
    variants: list[str] | None = None,
    repeats: int = 2,
    epochs: int = 1,
    phases: bool = True,
    trace: bool = False,
    trace_steps: int = 1,
    log: Callable[[str], None] = print,
) -> dict[str, Any]:
    """The report (module docstring) of `config`'s update from one rollout."""
    if repeats < 1 or epochs < 1 or trace_steps < 1:
        raise ValueError("the repeats, the epochs and the traced steps are at least 1")
    names = ["baseline"] + [n for n in (variants or list(VARIANTS)) if n != "baseline"]
    unknown = [n for n in names if n not in VARIANTS]
    if unknown:
        raise ValueError(f"no variant {', '.join(unknown)} (the variants: {', '.join(VARIANTS)})")
    config = resolve(config)
    trainer = Trainer(config)
    try:
        return _profile(trainer, init_from, names, repeats, epochs, phases, trace, trace_steps, log)
    finally:
        trainer.batch.close()


def _profile(
    trainer: Trainer,
    init_from: Path | None,
    names: list[str],
    repeats: int,
    epochs: int,
    phases: bool,
    trace: bool,
    trace_steps: int,
    log: Callable[[str], None],
) -> dict[str, Any]:
    device = trainer.device
    cuda = device.type == "cuda"
    sync = torch.cuda.synchronize if cuda else (lambda: None)
    parent = None if init_from is None else trainer.init_from(init_from)
    base = dataclasses.replace(trainer.config, epochs=min(epochs, trainer.config.epochs))
    start = time.perf_counter()
    rollout = trainer.collect()
    assert rollout is not None
    collect_s = time.perf_counter() - start
    policy = copy.deepcopy(trainer.policy.state_dict())
    optimizer = copy.deepcopy(trainer.optimizer.state_dict())
    counters = (trainer.samples, trainer.updates)
    w0 = weights(trainer)
    lr = trainer.config.lr

    def reset() -> None:
        trainer.policy.load_state_dict(policy)
        trainer.optimizer.load_state_dict(optimizer)
        trainer.samples, trainer.updates = counters
        trainer.policy.zero_grad(set_to_none=True)

    def update() -> tuple[dict, float, float | None]:
        reset()
        if cuda:
            torch.cuda.reset_peak_memory_stats(device)
        sync()
        start = time.perf_counter()
        stats = trainer.update(lr)
        sync()
        seconds = time.perf_counter() - start
        peak = torch.cuda.max_memory_allocated(device) / 2**20 if cuda else None
        return stats, seconds, peak

    report: dict[str, Any] = {
        "environment": environment(device),
        "config": {
            k: getattr(trainer.config, k)
            for k in ("kind", "width", "layers", "norm", "envs", "rollout", "epochs")
            + ("minibatches", "accumulate", "imitation_coef")
        },
        "parent": parent,
        "epochs": base.epochs,
        "collect_s": collect_s,
        "teacher_rows": rollout.get("teacher_rows"),
        "variants": {},
    }
    log(
        f"# The update's speed: {trainer.config.kind} on {device} "
        f"({report['environment'].get('gpu', f'{torch.get_num_threads()} threads')})\n"
    )
    log(f"The rollout: {collect_s:.1f} s; each update below runs {base.epochs} epoch(s).\n")
    rows = None
    reference: torch.Tensor | None = None
    moved = None
    for name in names:
        v = VARIANTS[name]
        runs = []
        try:
            with variant_of(trainer, base, v):
                for i in range(repeats):
                    stats, seconds, peak = update()
                    runs.append({"seconds": seconds, "peak_mb": peak})
                    if i == 0:
                        first = stats
                        w = weights(trainer)
        except torch.cuda.OutOfMemoryError as e:
            if name == "baseline":
                raise
            # Under the memory's cap (`gpu.cap`) the variant does not fit: recorded, the next
            # one goes on from the same state.
            trainer.policy.zero_grad(set_to_none=True)
            torch.cuda.empty_cache()
            report["variants"][name] = {"out_of_memory": str(e).splitlines()[0]}
            log(f"- {name}: out of memory under the cap")
            continue
        rows = first["update_samples"]
        if reference is None:
            reference = w
            moved = float((w - w0).abs().max())
        # The first update warms up (the autotuner's search for its shapes, the allocator's
        # growth): the median of the others counts, with one repeat the first.
        seconds = float(np.median([r["seconds"] for r in runs[1:] or runs]))
        entry = {
            "autotune": v.autotune,
            "nchw": v.nchw,
            "settings": dict(v.settings),
            "accumulate": settings_of(base, v.settings).accumulate,
            "chunk_rows": chunk_rows(rows, settings_of(base, v.settings)),
            "runs": runs,
            "seconds": seconds,
            "warm_up_s": runs[0]["seconds"] - seconds,
            "us_per_sample_pass": 1e6 * seconds / (rows * base.epochs),
            "losses": {k: first.get(k) for k in LOSSES},
            "weights_max_diff": float((w - reference).abs().max()),
        }
        report["variants"][name] = entry
        times = ", ".join(f"{r['seconds']:.2f}" for r in runs)
        log(f"- {name}: {times} s ({entry['us_per_sample_pass']:.0f} µs a sample-pass)")
    report["rows"] = rows
    report["weights_moved"] = moved
    log("")
    log(table(report))
    if phases:
        timer = Phases(device)
        timer.install(trainer)
        try:
            with variant_of(trainer, base, VARIANTS["baseline"]):
                _, seconds, _ = update()
        finally:
            timer.remove()
        report["phases"] = {"seconds": seconds, "parts": timer.report(seconds)}
        log("")
        log(phase_table(report["phases"]))
    if trace:
        try:
            report["trace"] = traced(trainer, base, update, trace_steps, device)
        except Exception as e:  # the profiler's failure costs only its table
            report["trace"] = f"the profiler failed: {type(e).__name__}: {e}"
        log("")
        log(f"The profiler over {trace_steps} optimizer step(s) of the baseline:\n")
        log(report["trace"])
    reset()
    return report


def traced(
    trainer: Trainer,
    base: Config,
    update: Callable[[], tuple[dict, float, float | None]],
    steps: int,
    device: torch.device,
) -> str:
    """torch's profiler over the baseline update's first `steps` optimizer steps: its operators
    with the most self time on the device (the CPU's off CUDA), as its table."""
    from torch.profiler import ProfilerActivity
    from torch.profiler import profile as torch_profile

    cuda = device.type == "cuda"
    activities = [ProfilerActivity.CPU] + ([ProfilerActivity.CUDA] if cuda else [])
    step = trainer.optimizer.step
    count = [0]

    def limited(*args, **kwargs):
        out = step(*args, **kwargs)
        count[0] += 1
        if count[0] >= steps:
            raise Enough
        return out

    trainer.optimizer.step = limited
    try:
        with variant_of(trainer, base, VARIANTS["baseline"]):
            with torch_profile(activities=activities) as prof:
                with contextlib.suppress(Enough):
                    update()
    finally:
        del trainer.optimizer.step
    key = "self_device_time_total" if cuda else "self_cpu_time_total"
    return prof.key_averages().table(sort_by=key, row_limit=30)


def table(report: dict[str, Any]) -> str:
    """The variants as a Markdown table."""
    lines = [
        f"{report['rows']:,} rows an epoch; the baseline's update moved a weight by at most "
        f"{report['weights_moved']:.3g}.",
        "",
        "| variant | accumulate (chunks) | seconds (each) | warm-up s | µs a sample-pass "
        "| against the baseline | peak MB | policy loss | value loss | entropy | imitation "
        "| weights' max difference |",
        "|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    base = report["variants"]["baseline"]["seconds"]
    for name, v in report["variants"].items():
        if "out_of_memory" in v:
            lines.append(f"| {name} | out of memory under the cap |" + " |" * 10)
            continue
        peaks = [r["peak_mb"] for r in v["runs"] if r["peak_mb"] is not None]
        loss = v["losses"]
        imitation = loss.get("imitation_loss")
        lines.append(
            f"| {name} | {v['accumulate']} ({', '.join(map(str, v['chunk_rows']))} rows) | "
            + ", ".join(f"{r['seconds']:.2f}" for r in v["runs"])
            + f" | {v['warm_up_s']:.2f} | {v['us_per_sample_pass']:.0f} | "
            + f"{v['seconds'] / base:.3f} | "
            + (f"{max(peaks):,.0f}" if peaks else "-")
            + f" | {loss['policy_loss']:.6f} | {loss['value_loss']:.6f} | {loss['entropy']:.6f} | "
            + ("-" if imitation is None else f"{imitation:.6f}")
            + f" | {v['weights_max_diff']:.3g} |"
        )
    return "\n".join(lines)


def phase_table(phases: dict[str, Any]) -> str:
    """The baseline's phases as a Markdown table."""
    lines = [
        f"The baseline's phases (one update, synchronised around each: {phases['seconds']:.2f} s):",
        "",
        "| phase | seconds | calls | share |",
        "|---|---:|---:|---:|",
    ]
    for name, p in phases["parts"].items():
        lines.append(f"| {name} | {p['seconds']:.3f} | {p['calls']} | {p['share']:.3f} |")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="python -m h3env.train.speed", description=__doc__)
    parser.add_argument("--config", type=Path, default=None, help="a run's TOML config file")
    parser.add_argument("--set", action="append", default=[], metavar="TABLE.KEY=VALUE")
    parser.add_argument("--init-from", type=Path, default=None, metavar="FILE")
    parser.add_argument("--device", default=None, help="the config's `device` (cuda, cpu)")
    parser.add_argument(
        "--variants", default=None, help=f"a comma-separated subset of: {', '.join(VARIANTS)}"
    )
    parser.add_argument("--repeats", type=int, default=2)
    parser.add_argument("--epochs", type=int, default=1, help="the epochs of each timed update")
    parser.add_argument("--no-phases", action="store_true", help="skip the phases' update")
    parser.add_argument("--trace", action="store_true", help="torch's profiler's operators")
    parser.add_argument("--trace-steps", type=int, default=1)
    parser.add_argument("--json", type=Path, default=None, metavar="FILE")
    args = parser.parse_args(argv)
    if args.init_from is not None and not args.init_from.exists():
        parser.error(f"--init-from: {args.init_from} does not exist")
    text = args.config.read_text(encoding="utf-8") if args.config else ""
    sets = list(args.set) + ([f"device.device={args.device}"] if args.device else [])
    config = load_config(text, sets)
    variants = None if args.variants is None else [v.strip() for v in args.variants.split(",")]
    report = profile(
        config,
        args.init_from,
        variants,
        args.repeats,
        args.epochs,
        not args.no_phases,
        args.trace,
        args.trace_steps,
    )
    if args.json is not None:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(report, indent=1) + "\n", encoding="utf-8")
        print(f"\nwrote {args.json}")


if __name__ == "__main__":
    main()
