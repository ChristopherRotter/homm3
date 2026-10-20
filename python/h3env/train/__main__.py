"""A training run from the command line (E06 T2a, T2b1; ADR-0026, `docs/epics/E06-design.md`
§1.3):

    uv run --group train python -m h3env.train --config configs/pool-smoke.toml --out runs/smoke
    uv run --group train python -m h3env.train --out runs/x --set samples=2_000_000
    uv run --group train python -m h3env.train --resume runs/smoke --set samples=40_000_000

The config file states what differs from the defaults (`h3env.train.run.Config`); each `--set
table.key=value` overrides one key. Each update and each evaluation prints one line; the run's
files go into `--out` (`config.toml`, `metrics.jsonl`, `record.json`, `checkpoint.pt`,
`best.pt`). Ctrl+C saves and stops the run (a second Ctrl+C aborts it, losing what the first would
have saved); `--resume` continues it from its checkpoint, the config the checkpoint's: its `--set`s
may change only the budget, the threads, the device and the checkpoints' cadence.
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass, field
from pathlib import Path

from .run import Config, ConfigError, ResumeError, RunExistsError, check_new, load_config


@dataclass
class Command:
    """What the command line asks for: a new run (`config`, from `text`) into `out`, or the
    resume of the run in `out` with `sets`."""

    out: Path | None
    config: Config | None = None
    text: str = ""
    resume: bool = False
    sets: list[str] = field(default_factory=list)


def parse(argv: list[str] | None = None) -> Command:
    parser = argparse.ArgumentParser(prog="python -m h3env.train", description=__doc__)
    parser.add_argument("--config", type=Path, default=None, help="the run's TOML config file")
    parser.add_argument(
        "--set",
        action="append",
        default=[],
        metavar="TABLE.KEY=VALUE",
        help="override one key of the config (repeatable)",
    )
    where = parser.add_mutually_exclusive_group()
    where.add_argument("--out", type=Path, default=None, help="the new run's directory")
    where.add_argument(
        "--resume", type=Path, default=None, metavar="DIR", help="continue the run in DIR"
    )
    args = parser.parse_args(argv)
    if args.resume is not None:
        if args.config is not None:
            parser.error("--resume takes the run's config from its checkpoint: change it by --set")
        return Command(args.resume, resume=True, sets=args.set)
    text = "" if args.config is None else args.config.read_text(encoding="utf-8")
    try:
        config = load_config(text, args.set)
        check_new(args.out)
    except (ConfigError, RunExistsError) as e:
        parser.error(str(e))
    return Command(args.out, config, text)


def line(record: dict) -> str | None:
    kind = record["type"]
    if kind == "eval":
        g = record["sampled"]
        text = (
            f"eval update {record['update']:>4} {record['samples']:>11} samples: sampled "
            f"{g['win_rate']:.3f} (attacker {g['attacker']['win_rate']:.3f}, defender "
            f"{g['defender']['win_rate']:.3f})"
        )
        for name in ("greedy", "random"):
            if name in record:
                text += f", {name} {record[name]['win_rate']:.3f}"
        if record.get("repeated"):
            return text + " [final: the last evaluation's]"
        return text + f" best {record['best']:.3f} [{record['eval_s']:.0f} s]"
    if kind == "update":
        return (
            f"update {record['update']:>4} {record['minutes']:6.1f} min {record['samples']:>11} "
            f"samples {record['samples_per_s']:8.0f}/s episodes {record['episodes']:>7} "
            f"attacker {record['attacker_won']:.2f} trunc {record['truncated']:.2f} "
            f"pool {record['pool']} won {record['pool_won']:.2f} "
            f"pg {record['policy_loss']:+.4f} v {record['value_loss']:.4f} "
            f"ent {record['entropy']:.3f} kl {record['kl']:.4f} "
            f"ev {record['explained_variance']:+.2f}"
        )
    if kind == "end":
        return f"end ({record['reason']}): {record['samples']} samples, {record['minutes']:.1f} min"
    if kind == "stop":
        return (
            f"stopped at update {record['update']}, {record['samples']} samples, "
            f"{record['minutes']:.1f} min: saved (continue with --resume)"
        )
    if kind == "segment":
        changed = "".join(f", {k} {old} -> {new}" for k, (old, new) in record["changed"].items())
        return (
            f"segment {record['segment']} from update {record['update']}, {record['samples']} "
            f"samples (batch seed {record['seed']:#x}, {record['dropped']} records dropped"
            f"{changed})"
        )
    return None


def show(record: dict) -> None:
    text = line(record)
    if text is not None:
        print(text, flush=True)


def main(argv: list[str] | None = None) -> None:
    command = parse(argv)
    from .ppo import resume, train

    if not command.resume:
        train(command.config, command.out, log=show, text=command.text)
        return
    try:
        resume(command.out, command.sets, log=show)
    except (ConfigError, ResumeError) as e:
        sys.exit(f"python -m h3env.train: error: {e}")


if __name__ == "__main__":
    main()
