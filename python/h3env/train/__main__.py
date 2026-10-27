"""A training run from the command line (E06 T2a, T2b1, T3b; ADR-0026, `docs/epics/E06-log.md`
§1.3):

    uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-1
    uv run --group train python -m h3env.train --out runs/x --set samples=2_000_000
    uv run --group train python -m h3env.train --resume runs/t4-1 --set samples=80_000_000
    uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-3 \
        --init-from runs/t4-1/best.pt

The config file states what differs from the defaults (`h3env.train.run.Config`); each `--set
table.key=value` overrides one key. Each update, gate, promotion and ladder prints one line; the
run's files go into `--out` (`config.toml`, `metrics.jsonl`, `record.json`, `checkpoint.pt`,
`promoted/<n>.pt`, `best.pt`, `league/<id>.pt`). `--init-from` starts a new run from a saved
network (its parent, best #0). Ctrl+C saves and stops the run (a second Ctrl+C aborts it,
losing what the first would have saved); `--resume` continues it from its checkpoint (a gate or
ladder the stop cut short first), the config the checkpoint's: its `--set`s may change only the
budget, the threads, the device and the checkpoints' cadence.
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
    init_from: Path | None = None


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
    parser.add_argument(
        "--init-from", type=Path, default=None, metavar="FILE", help="a new run's parent network"
    )
    args = parser.parse_args(argv)
    if args.resume is not None:
        if args.config is not None or args.init_from is not None:
            parser.error(
                "--resume takes the run's config and network from its checkpoint: change the "
                "config by --set"
            )
        return Command(args.resume, resume=True, sets=args.set)
    if args.init_from is not None and not args.init_from.exists():
        parser.error(f"--init-from: {args.init_from} does not exist")
    text = "" if args.config is None else args.config.read_text(encoding="utf-8")
    try:
        config = load_config(text, args.set)
        check_new(args.out)
    except (ConfigError, RunExistsError) as e:
        parser.error(str(e))
    return Command(args.out, config, text, init_from=args.init_from)


def line(record: dict) -> str | None:
    kind = record["type"]
    if kind == "gate":
        elo = "" if record["elo"] is None else f" ({record['elo']:+.0f} Elo)"
        return (
            f"gate update {record['update']:>4} {record['samples']:>11} samples against best "
            f"#{record['best']}: {record['result']} after {record['pairs']} pairs, score "
            f"{record['score']:.3f}{elo}, LLR {record['llr']:+.2f} [{record['seconds']:.0f} s]"
        )
    if kind == "promotion":
        return f"promoted #{record['index']} ({record['how']}) at update {record['update']}"
    if kind == "ladder":
        rows = ", ".join(
            f"{name} {r['win_rate']:.3f} ± {r['interval']:.3f}"
            for name, r in record["opponents"].items()
        )
        final = " final" if record["final"] else ""
        neutral = record.get("neutral")
        if neutral is not None:
            rows += (
                f"; neutral (attacking system) {neutral['win_rate']:.3f} ± "
                f"{neutral['interval']:.3f}, value {neutral['value']:+.3f}"
            )
        return (
            f"ladder{final} #{record['index']} ({record['per_kind']} per kind): {rows} "
            f"[{record['seconds']:.0f} s]"
        )
    if kind == "update":
        return (
            f"update {record['update']:>4} {record['minutes']:6.1f} min {record['samples']:>11} "
            f"samples {record['samples_per_s']:8.0f}/s episodes {record['episodes']:>7} "
            f"attacker {record['attacker_won']:.2f} trunc {record['truncated']:.2f} "
            f"league {record['league']} score {record['league_score']:.2f} "
            f"passes {record['frozen_passes']:.1f} "
            f"pg {record['policy_loss']:+.4f} v {record['value_loss']:.4f} "
            f"ent {record['entropy']:.3f} kl {record['kl']:.4f} "
            f"ev {record['explained_variance']:+.2f}"
            + "".join(
                f" {name} {row['attacker_won']:.2f}"
                for name, row in record.get("by_kind", {}).items()
            )
            + "".join(
                f" system-{name} {row['score']:.2f}"
                for name, row in record.get("system", {}).items()
            )
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
            + (f", the pending {record['pending']['kind']} first" if record.get("pending") else "")
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
        try:
            train(
                command.config,
                command.out,
                log=show,
                text=command.text,
                init_from=command.init_from,
            )
        except ConfigError as e:
            sys.exit(f"python -m h3env.train: error: {e}")
        return
    try:
        resume(command.out, command.sets, log=show)
    except (ConfigError, ResumeError) as e:
        sys.exit(f"python -m h3env.train: error: {e}")


if __name__ == "__main__":
    main()
