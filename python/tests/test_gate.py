"""The evaluation gate (E06 T3b, `docs/epics/E06-design.md` §6): the SPRT's arithmetic and error
rates without torch, then the head-to-head of two networks on the gate set v1 (with torch)."""

from __future__ import annotations

import math

import numpy as np
import pytest

from h3env import _ffi
from h3env.train.gate import OUTCOMES, Bounds, Sprt, elo_score, game_score, score_elo


def test_the_scores_and_the_elo_differences() -> None:
    assert elo_score(0) == 0.5 and elo_score(20) == pytest.approx(0.52876, abs=1e-5)
    assert score_elo(elo_score(-150)) == pytest.approx(-150)
    assert score_elo(0.0) is None and score_elo(1.0) is None
    # A side wins by its own result or the other's escape; a draw or a truncation is half.
    assert [game_score(r, 0) for r in (_ffi.RESULT_ATTACKER, _ffi.RESULT_DEFENDER)] == [1.0, 0.0]
    assert game_score(_ffi.RESULT_ATTACKER_SURRENDERED, 1) == 1.0
    assert game_score(_ffi.RESULT_DRAW, 0) == game_score(_ffi.RESULT_NONE, 1) == 0.5


def test_the_sprt_has_its_bounds_and_decides_in_pair_order() -> None:
    sprt = Sprt(Bounds())
    assert (sprt.lower, sprt.upper) == pytest.approx((-2.944, 2.944), abs=1e-3)
    assert sprt.pairs == 0 and sprt.llr() == 0.0 and sprt.decision() is None
    # The candidate wins both games of every pair: promoted after a handful.
    while sprt.decision() is None:
        sprt.add(1.0)
    assert sprt.decision() == "promoted" and sprt.pairs < 20
    summary = sprt.summary()
    assert summary["counts"] == [0, 0, 0, 0, sprt.pairs] and summary["score"] == 1.0
    assert summary["elo"] is None and summary["llr"] >= sprt.upper
    # The formula: n (s1 − s0)(2 x̄ − s0 − s1) / (2 σ̂²), half a pair added to each outcome.
    other = Sprt(Bounds())
    for score, n in zip(OUTCOMES, (10, 20, 40, 20, 10), strict=True):
        for _ in range(n):
            other.add(score)
    w = np.array([10, 20, 40, 20, 10]) + 0.5
    x = np.array(OUTCOMES)
    mean = (w * x).sum() / w.sum()
    var = (w * (x - mean) ** 2).sum() / w.sum()
    s0, s1 = 0.5, elo_score(20)
    assert other.llr() == pytest.approx(100 * (s1 - s0) * (2 * mean - s0 - s1) / (2 * var))
    # At the cap without a decision: rejected as inconclusive.
    capped = Sprt(Bounds(max_pairs=3))
    for _ in range(3):
        capped.add(0.5)
    assert capped.decision() == "inconclusive"


def _simulate(elo: float, rng: np.random.Generator, draws: float = 0.3) -> tuple[str, int]:
    """One gate on simulated pairs: each game a draw with `draws`, else a win at the score
    `elo` gives."""
    sprt = Sprt(Bounds())
    p = elo_score(elo)
    while True:
        games = [
            0.5 if rng.random() < draws else float(rng.random() < (p - draws / 2) / (1 - draws))
            for _ in range(2)
        ]
        sprt.add(sum(games) / 2)
        decision = sprt.decision()
        if decision is not None:
            return decision, sprt.pairs


def test_the_sprt_keeps_its_error_rates_and_decides_large_gains_early() -> None:
    # Design §6's bounds: α = β = 0.05 at elo0 = 0 and elo1 = 20; +100 Elo in about 65 pairs.
    rng = np.random.default_rng(7)
    for elo, wanted in ((0.0, "rejected"), (20.0, "promoted")):
        runs = [_simulate(elo, rng) for _ in range(200)]
        right = sum(r[0] == wanted for r in runs)
        assert right >= 180, (elo, right)
        assert 300 < np.mean([r[1] for r in runs]) < 900
    large = [_simulate(100.0, rng) for _ in range(100)]
    assert all(r[0] == "promoted" for r in large) and np.mean([r[1] for r in large]) < 150


def test_a_network_against_itself_is_not_promoted() -> None:
    torch = pytest.importorskip("torch")
    from h3env import BattleBatch
    from h3env.train.gate import gate, line
    from h3env.train.policy import Policy

    probe = BattleBatch(1)
    torch.manual_seed(0)
    net = Policy.for_batch(probe, hidden=32, embed=8)
    probe.close()
    twin = net.frozen()
    bounds = Bounds(max_pairs=24)
    first = gate(net, twin, bounds, threads=2, max_steps=300)
    again = gate(net, twin, bounds, threads=2, max_steps=300)
    assert first["result"] in ("rejected", "inconclusive") and first["pairs"] <= 24
    assert first["games"] >= 2 * first["pairs"] and sum(first["counts"]) == first["pairs"]
    # The same networks and seeds: the same pairs (common random numbers), on the CPU.
    keep = ("result", "pairs", "counts", "llr")
    assert {k: first[k] for k in keep} == {k: again[k] for k in keep}
    assert net.training and not twin.training
    assert "gate: " in line(first, bounds) and math.isfinite(first["llr"])
    with pytest.raises(ValueError, match="max_pairs"):
        gate(net, twin, Bounds(max_pairs=4001))
