"""A trained network against the engine's seats (E05 T7, `docs/epics/E05-log.md` §9; the held-out
sets v1, E06 T3a):

    uv run --group train python -m h3env.train.ladder --checkpoint runs/t3-1/best.pt

The sampled policy against Random and the baseline bots (C ABI v30) on a held-out set: the report
set v1 by default (`--episodes`: the first n of its 500 per kind), or E05's v0 (`--set v0`). The
episodes are `h3 ai eval --set …`'s: on the whole report set each line ends with the bots' rates
against the same opponent from their table (`python/records/bots-report-v1.json`).
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

from .evaluate import (
    BOTS_TABLE,
    HELD_OUT_BATCH_SEEDS,
    OPPONENTS,
    REPORT_V1_ENVS,
    SETS,
    bots_table,
    ladder,
    load,
    trained_seed,
)


def main(argv: list[str] | None = None) -> None:
    """`python -m h3env.train.ladder --checkpoint runs/t3-1/best.pt`: a trained network against
    Random and the baseline bots on a held-out set, one line per opponent."""
    parser = argparse.ArgumentParser(prog="python -m h3env.train.ladder", description=main.__doc__)
    parser.add_argument("--checkpoint", required=True)
    parser.add_argument("--set", dest="held_out", choices=SETS, default="report-v1")
    parser.add_argument("--opponents", default=",".join(OPPONENTS))
    parser.add_argument(
        "--episodes",
        type=int,
        help=f"per kind for report-v1 ({REPORT_V1_ENVS}), in all for v0 (1000)",
    )
    parser.add_argument("--envs", type=int, help="v0 only (100)")
    parser.add_argument("--threads", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--max-steps", type=int, default=2000)
    parser.add_argument("--device", default="cpu")
    parser.add_argument("--json", help="also write the results to this file")
    args = parser.parse_args(argv)
    trained = trained_seed(args.checkpoint)
    if trained in HELD_OUT_BATCH_SEEDS:
        # The same batch seed gives the same episodes: the ladder's would be the run's own.
        parser.error(f"the checkpoint was trained on a held-out set's episodes (seed {trained:#x})")
    if args.held_out == "v0":
        settings = {"episodes": args.episodes or 1000, "envs": args.envs or 100}
    elif args.envs is not None:
        parser.error("--envs: the report set v1 fixes its envs (only --set v0 takes it)")
    else:
        settings = {"per_kind": args.episodes or REPORT_V1_ENVS}
    policy = load(args.checkpoint, args.device)
    rows = ladder(
        policy,
        opponents=args.opponents.split(","),
        held_out=args.held_out,
        threads=args.threads,
        max_steps=args.max_steps,
        **settings,
    )
    whole = settings.get("per_kind") == REPORT_V1_ENVS and args.max_steps == 2000
    bots = bots_table() if whole and BOTS_TABLE.exists() else None
    for opponent, r in rows.items():
        print(_line(opponent, r, bots), flush=True)
    if args.json:
        Path(args.json).write_text(json.dumps(rows, indent=1) + "\n", encoding="utf-8")


def _line(opponent: str, r: dict, bots: dict | None) -> str:
    """One opponent's line: the win rate (with its interval and per kind on the report set), the
    value efficiency, skipped episodes, and the bots' rates against the same opponent."""
    if "kinds" in r:
        kinds = ", ".join(f"{kind} {rate:.3f}" for kind, rate in r["kinds"].items())
        line = (
            f"against {opponent:9}: win rate {r['win_rate']:.3f} ± {r['interval']:.3f} ({kinds}), "
            f"value {r['value']:+.3f}, {r['games']} games in {r['seconds']:.0f}s"
        )
        skipped = r["skipped"]
    else:
        line = (
            f"against {opponent:9}: win rate {r['win_rate']:.3f} (attacker "
            f"{r['attacker']['win_rate']:.3f}, defender {r['defender']['win_rate']:.3f}), "
            f"value {(r['attacker']['reward'] + r['defender']['reward']) / 2:+.3f}"
        )
        skipped = r["attacker"]["skipped"] + r["defender"]["skipped"]
    if skipped:
        line += f", {skipped} episodes skipped (not h3 ai eval's)"
    if bots is not None:
        rates = [
            f"{policy} {bots['by_pair'][(policy, opponent)]['win_rate']:.3f}"
            for policy in bots["policies"]
            if (policy, opponent) in bots["by_pair"]
        ]
        line += "; the bots against it: " + ", ".join(rates)
    return line


if __name__ == "__main__":
    main()
