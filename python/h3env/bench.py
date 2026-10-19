"""Python's steps/s through the binding (E05 T6a, `docs/epics/E05-design.md` §7), the counterpart
of `h3 ai bench`: a batch of N envs of one scenario or of the encounter distribution v1, both
seats external, K calls of uniformly random legal actions chosen from the batch's own masks (a
cast's heads through the head calls), observations and entry masks written.

    uv run python -m h3env.bench [--scenario FILE | --encounters] [--envs 64] [--threads 0]
                                 [--calls 1000] [--seed 1] [--max-steps 0]

It prints the steps per second inside `h3_batch_step` and with the head calls and the choice in
numpy; the choice draws from numpy's generator, so the episodes differ from `h3 ai bench`'s.
"""

from __future__ import annotations

import argparse
import time
from pathlib import Path

import numpy as np

from . import _ffi
from .batch import BattleBatch

#: The repository this package sits in.
REPO = Path(__file__).resolve().parents[2]


def run(
    *,
    scenario: Path | None,
    envs: int,
    threads: int,
    calls: int,
    seed: int,
    max_steps: int,
) -> dict[str, float]:
    """Plays the workload and returns its counts and wall times."""
    batch = BattleBatch(envs, threads=threads, seed=seed, max_steps=max_steps, final_obs=False)
    if scenario is None:
        batch.add_encounters()
    else:
        batch.add_scenario(scenario.read_text(encoding="utf-8"))
    rng = np.random.default_rng(seed)
    batch.reset()
    steps = episodes = refused = 0
    step_secs = 0.0
    start = time.perf_counter()
    for _ in range(calls):
        acting = batch.to_move >= 0
        batch.random_actions(rng)
        before = time.perf_counter()
        batch.step(check=False)
        step_secs += time.perf_counter() - before
        accepted = acting & (batch.status == _ffi.OK)
        steps += int(accepted.sum())
        refused += int((batch.status != _ffi.OK).sum())
        episodes += int(batch.done.sum())
    secs = time.perf_counter() - start
    batch.close()
    return {
        "steps": steps,
        "episodes": episodes,
        "refused": refused,
        "secs": secs,
        "step_secs": step_secs,
    }


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="python -m h3env.bench", description=__doc__.split("\n")[0]
    )
    source = parser.add_mutually_exclusive_group()
    source.add_argument(
        "--scenario", type=Path, default=REPO / "packs/synthetic/scenarios/full.toml"
    )
    source.add_argument("--encounters", action="store_true", help="the encounter distribution v1")
    parser.add_argument("--envs", type=int, default=64)
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument("--calls", type=int, default=1000)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--max-steps", type=int, default=0)
    args = parser.parse_args(argv)
    report = run(
        scenario=None if args.encounters else args.scenario,
        envs=args.envs,
        threads=args.threads,
        calls=args.calls,
        seed=args.seed,
        max_steps=args.max_steps,
    )
    print(
        f"envs {args.envs}, threads {args.threads}, calls {args.calls}: "
        f"{report['steps']} steps, {report['episodes']} episodes, {report['refused']} refused"
    )
    print(f"steps/s inside h3_batch_step: {report['steps'] / report['step_secs']:.0f}")
    print(
        f"steps/s with the head calls and the random choice: {report['steps'] / report['secs']:.0f}"
    )


if __name__ == "__main__":
    main()
