"""The smoke run from the command line (E05 T8, `docs/epics/E05-design.md` §8):

    uv run --group train python -m h3env.train --minutes 60 --out runs/smoke [--device cuda]

Every `Config` field is an option (`--eval-every 10`, `--kinds 0,0,1,0,0,0`, ...). Each update and
each evaluation prints one line; the run's files go into `--out` (`config.json`, `metrics.jsonl`,
`checkpoint.pt`).
"""

from __future__ import annotations

import argparse
import dataclasses
from pathlib import Path

from .ppo import Config, train


def parse(argv: list[str] | None = None) -> tuple[Config, Path | None]:
    parser = argparse.ArgumentParser(prog="python -m h3env.train", description=__doc__)
    defaults = Config()
    for f in dataclasses.fields(Config):
        name = "--" + f.name.replace("_", "-")
        value = getattr(defaults, f.name)
        if f.name == "kinds":
            parser.add_argument(
                name,
                type=lambda text: [int(x) for x in text.split(",")],
                default=None,
                help="six kind weights, comma-separated (default: the distribution's)",
            )
        elif isinstance(value, bool):
            parser.add_argument(name, action=argparse.BooleanOptionalAction, default=value)
        else:
            parser.add_argument(name, type=type(value), default=value, help=f"(default {value})")
    parser.add_argument("--out", type=Path, default=None, help="the run's directory")
    args = vars(parser.parse_args(argv))
    out = args.pop("out")
    return Config(**args), out


def line(record: dict) -> str:
    if record.get("eval"):
        g = record["sampled"]
        text = (
            f"eval update {record['update']:>4} {record['minutes']:6.1f} min: sampled "
            f"{g['win_rate']:.3f} (attacker {g['attacker']['win_rate']:.3f}, defender "
            f"{g['defender']['win_rate']:.3f})"
        )
        for name in ("greedy", "random"):
            if name in record:
                text += f", {name} {record[name]['win_rate']:.3f}"
        return text + f" [{record['eval_s']:.0f} s]"
    return (
        f"update {record['update']:>4} {record['minutes']:6.1f} min {record['samples']:>10} "
        f"samples {record['samples_per_s']:8.0f}/s episodes {record['episodes']:>7} "
        f"attacker {record['attacker_won']:.2f} trunc {record['truncated']:.2f} "
        f"pg {record['policy_loss']:+.4f} v {record['value_loss']:.4f} "
        f"ent {record['entropy']:.3f} kl {record['kl']:.4f} ev {record['explained_variance']:+.2f}"
    )


def main(argv: list[str] | None = None) -> None:
    config, out = parse(argv)
    train(config, out, log=lambda record: print(line(record), flush=True))


if __name__ == "__main__":
    main()
