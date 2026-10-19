"""A trained network against the engine's seats (E05 T7, `docs/epics/E05-design.md` §9):

    uv run --group train python -m h3env.train.ladder --checkpoint runs/pool/best.pt

The sampled policy against Random and the baseline bots (C ABI v30) on the held-out episodes of
`h3env.train.evaluate`, the same as `h3 ai eval`'s by default: the network's row beside its table.
"""

from __future__ import annotations

import argparse
import os

from .evaluate import OPPONENTS, ladder, load


def main(argv: list[str] | None = None) -> None:
    """`python -m h3env.train.ladder --checkpoint runs/pool/best.pt`: a trained network against
    Random and the baseline bots on the held-out episodes, one line per opponent."""
    parser = argparse.ArgumentParser(prog="python -m h3env.train.ladder", description=main.__doc__)
    parser.add_argument("--checkpoint", required=True)
    parser.add_argument("--opponents", default=",".join(OPPONENTS))
    parser.add_argument("--episodes", type=int, default=1000)
    parser.add_argument("--envs", type=int, default=100)
    parser.add_argument("--threads", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--max-steps", type=int, default=2000)
    parser.add_argument("--device", default="cpu")
    args = parser.parse_args(argv)
    policy = load(args.checkpoint, args.device)
    rows = ladder(
        policy,
        opponents=args.opponents.split(","),
        episodes=args.episodes,
        envs=args.envs,
        threads=args.threads,
        max_steps=args.max_steps,
    )
    for opponent, r in rows.items():
        print(
            f"against {opponent:9}: win rate {r['win_rate']:.3f} (attacker "
            f"{r['attacker']['win_rate']:.3f}, defender {r['defender']['win_rate']:.3f}), "
            f"value {(r['attacker']['reward'] + r['defender']['reward']) / 2:+.3f}",
            flush=True,
        )


if __name__ == "__main__":
    main()
