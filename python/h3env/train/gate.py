"""The evaluation gate (E06 T3b, `docs/epics/E06-log.md` §6; ADR-0026 point 6): a candidate
network against the current best on the gate set v1, promoted by a sequential test.

    uv run --group train python -m h3env.train.gate --candidate runs/b/best.pt --best runs/a/best.pt

Pair j is episode ⌊j / 500⌋ of env j mod 500 of the gate set (`GATE_V1_SEED`, the default kinds),
played twice: in one batch the candidate attacks, in the other the best; both seats external, each
network acting on its own rows, its draws seeded alike at every gate (common random numbers). A
pair's score is the candidate's mean over its two games (a win 1, a draw or a truncation ½, a loss
0), so the attacker's advantage cancels. The pairs feed, in order, a generalized SPRT (Fishtest's
normal approximation): `H0: elo = elo0`, `H1: elo = elo1`, the score `s = 1 / (1 + 10^(−elo /
400))`; after n pairs `LLR = n (s1 − s0)(2 x̄ − s0 − s1) / (2 σ̂²)`, the mean and variance from the
five outcomes' counts with half a pair added to each. The first n with `LLR ≥ ln((1 − β) / α)`
promotes, `LLR ≤ ln(β / (1 − α))` rejects, `max_pairs` without either rejects as inconclusive; the
pairs after the deciding one are not counted. The test's arithmetic is free of torch (`Sprt`).
"""

from __future__ import annotations

import argparse
import math
import os
import time
from dataclasses import dataclass

import numpy as np

from .. import _ffi
from ..batch import BattleBatch
from .evaluate import GATE_V1_ENVS, GATE_V1_EPISODES, GATE_V1_SEED, Stopped, duel

#: A pair's possible scores: the candidate's mean over its two games.
OUTCOMES = (0.0, 0.25, 0.5, 0.75, 1.0)

#: The results a side wins by (an escape is masked out of the batch, ADR-0025 point 7).
WINS = (
    (_ffi.RESULT_ATTACKER, _ffi.RESULT_DEFENDER_RETREATED, _ffi.RESULT_DEFENDER_SURRENDERED),
    (_ffi.RESULT_DEFENDER, _ffi.RESULT_ATTACKER_RETREATED, _ffi.RESULT_ATTACKER_SURRENDERED),
)


@dataclass(frozen=True)
class Bounds:
    """The test's hypotheses and error rates (the config's `[gate]`, recorded)."""

    elo0: float = 0.0
    elo1: float = 20.0
    alpha: float = 0.05
    beta: float = 0.05
    max_pairs: int = GATE_V1_ENVS * GATE_V1_EPISODES


def elo_score(elo: float) -> float:
    """The expected score at an Elo difference of `elo`."""
    return 1.0 / (1.0 + 10.0 ** (-elo / 400.0))


def score_elo(score: float) -> float | None:
    """The Elo difference of an expected `score` (None at 0 or 1)."""
    if not 0.0 < score < 1.0:
        return None
    return -400.0 * math.log10(1.0 / score - 1.0)


def game_score(result: int, side: int) -> float:
    """`side`'s score of a game that ended with `result`: a win 1, a loss 0, else (a draw, a
    truncation) ½."""
    if result in WINS[side]:
        return 1.0
    if result in WINS[1 - side]:
        return 0.0
    return 0.5


class Sprt:
    """The generalized SPRT over pair scores (see the module's documentation)."""

    def __init__(self, bounds: Bounds) -> None:
        self.bounds = bounds
        self.counts = [0] * len(OUTCOMES)
        self.lower = math.log(bounds.beta / (1.0 - bounds.alpha))
        self.upper = math.log((1.0 - bounds.beta) / bounds.alpha)

    @property
    def pairs(self) -> int:
        return sum(self.counts)

    def add(self, score: float) -> None:
        self.counts[round(score * 4)] += 1

    def llr(self) -> float:
        b = self.bounds
        s0, s1 = elo_score(b.elo0), elo_score(b.elo1)
        weights = [c + 0.5 for c in self.counts]
        total = sum(weights)
        mean = sum(w * x for w, x in zip(weights, OUTCOMES, strict=True)) / total
        variance = sum(w * (x - mean) ** 2 for w, x in zip(weights, OUTCOMES, strict=True)) / total
        return self.pairs * (s1 - s0) * (2.0 * mean - s0 - s1) / (2.0 * variance)

    def decision(self) -> str | None:
        """`promoted`, `rejected`, `inconclusive` (at `max_pairs`) or None (go on)."""
        llr = self.llr()
        if llr >= self.upper:
            return "promoted"
        if llr <= self.lower:
            return "rejected"
        if self.pairs >= self.bounds.max_pairs:
            return "inconclusive"
        return None

    def summary(self) -> dict:
        """The pairs, their counts per outcome, the LLR and its bounds, the candidate's mean pair
        score and its Elo difference."""
        n = self.pairs
        score = sum(c * x for c, x in zip(self.counts, OUTCOMES, strict=True)) / max(n, 1)
        return {
            "pairs": n,
            "counts": list(self.counts),
            "llr": self.llr(),
            "bounds": [self.lower, self.upper],
            "score": score,
            "elo": score_elo(score) if n else None,
        }


def play_gate(
    candidate,
    best,
    bounds: Bounds,
    *,
    threads: int = 0,
    max_steps: int = 2000,
    rng_seed: int = 0,
    stop=None,
) -> dict:
    """Plays the gate set's pairs in order until the test decides (see the module's
    documentation): `candidate` and `best` are networks (`Policy`). Returns the decision
    (`result`), `Sprt.summary`'s numbers, the games played and the seconds; it reads the games'
    results, never their rewards. Raises `Stopped` when `stop` was requested before one of its
    batch calls (the gate has no result)."""
    if not 1 <= bounds.max_pairs <= GATE_V1_ENVS * GATE_V1_EPISODES:
        raise ValueError(f"max_pairs lies in 1 to {GATE_V1_ENVS * GATE_V1_EPISODES}")
    start = time.monotonic()
    # Pair j needs episode j // 500 of env j % 500: the first min(500, max_pairs) envs suffice.
    envs = min(GATE_V1_ENVS, bounds.max_pairs)
    episodes = math.ceil(bounds.max_pairs / GATE_V1_ENVS)
    batches, agents = [], []
    for k, side in enumerate(("attacker", "defender")):
        batch = BattleBatch(
            envs,
            threads=threads,
            seed=GATE_V1_SEED,
            max_steps=max_steps,
            final_obs=False,
            # The kill preview for a network that reads it (E06 T6d).
            preview=candidate.reads_preview or best.reads_preview,
        )
        batch.add_encounters(attacker="external", defender="external")
        batch.reset()
        batches.append(batch)
        agents.append(duel(candidate, best, side, rng_seed + 2 * k))
    # The candidate's score of each (batch, env, episode); NaN until its game ended.
    scores = np.full((2, envs, episodes), np.nan)
    sprt = Sprt(bounds)
    decision = None
    games = 0
    try:
        while decision is None:
            if stop is not None and stop.requested:
                raise Stopped
            for k, (batch, agent) in enumerate(zip(batches, agents, strict=True)):
                if (batch.episode >= episodes).all():
                    continue
                before = batch.episode.astype(np.int64)
                agent(batch)
                batch.step()
                for e in np.nonzero(batch.done)[0]:
                    if before[e] < episodes:
                        scores[k, e, before[e]] = game_score(int(batch.result[e]), k)
                        games += 1
            while decision is None:
                j = sprt.pairs
                e, i = j % GATE_V1_ENVS, j // GATE_V1_ENVS
                pair = scores[:, e, i]
                if np.isnan(pair).any():
                    break
                sprt.add(float(pair.mean()))
                decision = sprt.decision()
    finally:
        for batch in batches:
            batch.close()
    return {
        "result": decision,
        **sprt.summary(),
        "games": games,
        "seconds": time.monotonic() - start,
    }


def gate(candidate, best, bounds: Bounds | None = None, **settings) -> dict:
    """`play_gate` (by default `Bounds()`) with both networks in evaluation mode (each restored
    after)."""
    bounds = Bounds() if bounds is None else bounds
    modes = [(net, net.training) for net in (candidate, best)]
    for net, _ in modes:
        net.eval()
    try:
        return play_gate(candidate, best, bounds, **settings)
    finally:
        for net, mode in modes:
            net.train(mode)


def main(argv: list[str] | None = None) -> None:
    """`python -m h3env.train.gate --candidate b.pt --best a.pt`: the gate between two saved
    networks (a run's `best.pt`, `promoted/<n>.pt` or `checkpoint.pt`), its decision and numbers."""
    from .evaluate import load

    defaults = Bounds()
    parser = argparse.ArgumentParser(prog="python -m h3env.train.gate", description=main.__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--best", required=True)
    for name in ("elo0", "elo1", "alpha", "beta"):
        parser.add_argument(f"--{name}", type=float, default=getattr(defaults, name))
    parser.add_argument("--max-pairs", type=int, default=defaults.max_pairs)
    parser.add_argument("--threads", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--max-steps", type=int, default=2000)
    parser.add_argument("--device", default="cpu")
    args = parser.parse_args(argv)
    bounds = Bounds(args.elo0, args.elo1, args.alpha, args.beta, args.max_pairs)
    candidate, best = load(args.candidate, args.device), load(args.best, args.device)
    r = gate(candidate, best, bounds, threads=args.threads, max_steps=args.max_steps)
    print(line(r, bounds), flush=True)


def line(r: dict, bounds: Bounds) -> str:
    """A gate's result in one line."""
    elo = "" if r["elo"] is None else f" ({r['elo']:+.0f} Elo)"
    return (
        f"gate: {r['result']} after {r['pairs']} pairs ({r['games']} games), score "
        f"{r['score']:.3f}{elo}, LLR {r['llr']:+.2f} in [{r['bounds'][0]:.2f}, "
        f"{r['bounds'][1]:.2f}] (elo0 {bounds.elo0:g}, elo1 {bounds.elo1:g}), {r['seconds']:.0f}s"
    )


if __name__ == "__main__":
    main()
