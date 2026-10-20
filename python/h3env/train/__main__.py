"""A training run from the command line (E06 T2a; ADR-0026, `docs/epics/E06-design.md` §1.3):

    uv run --group train python -m h3env.train --config configs/pool-smoke.toml --out runs/smoke
    uv run --group train python -m h3env.train --out runs/x --set samples=2_000_000

The config file states what differs from the defaults (`h3env.train.run.Config`); each `--set
table.key=value` overrides one key. Each update and each evaluation prints one line; the run's
files go into `--out` (`config.toml`, `metrics.jsonl`, `record.json`, `checkpoint.pt`,
`best.pt`).
"""

from __future__ import annotations

import argparse
from pathlib import Path

from .run import Config, ConfigError, RunExistsError, check_new, load_config


def parse(argv: list[str] | None = None) -> tuple[Config, Path | None, str]:
    parser = argparse.ArgumentParser(prog="python -m h3env.train", description=__doc__)
    parser.add_argument("--config", type=Path, default=None, help="the run's TOML config file")
    parser.add_argument(
        "--set",
        action="append",
        default=[],
        metavar="TABLE.KEY=VALUE",
        help="override one key of the config (repeatable)",
    )
    parser.add_argument("--out", type=Path, default=None, help="the run's directory")
    args = parser.parse_args(argv)
    text = "" if args.config is None else args.config.read_text(encoding="utf-8")
    try:
        config = load_config(text, args.set)
        check_new(args.out)
    except (ConfigError, RunExistsError) as e:
        parser.error(str(e))
    return config, args.out, text


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
    return None


def show(record: dict) -> None:
    text = line(record)
    if text is not None:
        print(text, flush=True)


def main(argv: list[str] | None = None) -> None:
    config, out, text = parse(argv)
    from .ppo import train

    train(config, out, log=show, text=text)


if __name__ == "__main__":
    main()
