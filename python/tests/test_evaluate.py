"""The held-out sets v1 without torch (E06 T3a, `docs/epics/E06-log.md` §5; the neutral set,
T7c, §12): the report and neutral sets' episodes and rates, every held-out seed in the trainer's
check, and the committed bots' tables."""

from __future__ import annotations

import math
import tomllib
from pathlib import Path

import numpy as np
import pytest

from h3env import BattleBatch, _ffi
from h3env.train.evaluate import (
    BOTS_TABLES,
    GATE_V1_SEED,
    HELD_OUT_BATCH_SEEDS,
    NEUTRAL_KINDS,
    NEUTRAL_V1_SEED,
    NEUTRAL_V1_WEIGHTS,
    REPORT_V1_ENVS,
    REPORT_V1_SEED,
    REPORT_V1_WEIGHTS,
    STRATIFIED,
    bots_table,
    episode_seeds,
    gate_seeds,
    held_out_seeds,
    random_agent,
    report,
    report_seeds,
    stratified,
)
from h3env.train.run import HeldOut, HeldOutError


def _one_hot(kind: int) -> list[int]:
    return [int(i == kind) for i in range(len(_ffi.ENCOUNTER_KINDS))]


def test_the_report_set_plays_each_kind_from_its_own_batch_seed() -> None:
    # Kind k's batch: seed REPORT_V1_SEED + k, the kind one-hot; its episodes are the first of each
    # env (`h3 ai eval --set report-v1`'s, its constants pinned by an h3-cli test).
    seeds = report_seeds(3)
    assert len(seeds) == 9
    for kind in REPORT_V1_WEIGHTS:
        batch = BattleBatch(3, seed=REPORT_V1_SEED + kind)
        batch.add_encounters(kinds=_one_hot(kind))
        batch.reset()
        assert (batch.encounter == kind).all()
        assert set(int(s) for s in batch.episode_seed) <= seeds
        batch.close()
    assert report_seeds(1) < report_seeds(2) and len(report_seeds()) == 3 * REPORT_V1_ENVS


def test_the_neutral_set_is_the_censuss_neutral_kinds() -> None:
    # The neutral kinds at their census shares (`share` in encounters-v1.toml), the system actor
    # defending each (design §12); their batches as the report set's, from NEUTRAL_V1_SEED + k.
    table = Path(__file__).resolve().parents[2] / "crates/h3-sim/data/encounters-v1.toml"
    shares = tomllib.loads(table.read_text())["kind"]
    assert {k: shares[_ffi.ENCOUNTER_KINDS[k]]["share"] for k in NEUTRAL_V1_WEIGHTS} == (
        NEUTRAL_V1_WEIGHTS
    )
    assert all(shares[_ffi.ENCOUNTER_KINDS[k]]["weight"] == 0 for k in NEUTRAL_KINDS)
    assert NEUTRAL_KINDS.isdisjoint(REPORT_V1_WEIGHTS)
    assert STRATIFIED["neutral-v1"][3] == ("attacker",)
    seeds = report_seeds(2, held_out="neutral-v1")
    for kind in NEUTRAL_V1_WEIGHTS:
        batch = BattleBatch(2, seed=NEUTRAL_V1_SEED + kind)
        batch.add_encounters(kinds=_one_hot(kind))
        batch.reset()
        # The system actor defends: the attacker is the only side the caller plays.
        assert (batch.encounter == kind).all() and (batch.to_move == 0).all()
        assert set(int(s) for s in batch.episode_seed) <= seeds
        batch.close()


def test_every_held_out_set_is_in_the_trainers_check() -> None:
    # v0's 12 here, the report and neutral sets' 1 500 each and the gate set's 4 000: all distinct.
    gate = gate_seeds()
    neutral = report_seeds(held_out="neutral-v1")
    held = HeldOut(held_out_seeds(12, 4))
    assert len(gate) == 4000 and len(held.seeds) == 12 + 1500 + 1500 + 4000
    assert episode_seeds(12, 4) | report_seeds() | neutral | gate == held.seeds
    held.check([1, 2, 3])
    for seeds in (report_seeds(1), {min(neutral)}, {min(gate)}):
        with pytest.raises(HeldOutError, match="held-out"):
            held.check(seeds)
    assert len(HELD_OUT_BATCH_SEEDS) == 8 and GATE_V1_SEED in HELD_OUT_BATCH_SEEDS
    assert NEUTRAL_V1_SEED + _ffi.ENCOUNTER_SIEGE_NEUTRAL in HELD_OUT_BATCH_SEEDS


def test_the_report_rates_are_stratified_by_kind_and_side() -> None:
    # Design §5: 0.75 on every kind and side, 500 games each, is ±1.8 points.
    side = {"win_rate": 0.75, "reward": 0.5, "episodes": 500}
    names = [_ffi.ENCOUNTER_KINDS[k] for k in REPORT_V1_WEIGHTS]
    rates = stratified({n: {"attacker": side, "defender": side} for n in names})
    assert rates["win_rate"] == pytest.approx(0.75) and rates["value"] == pytest.approx(0.5)
    assert rates["interval"] == pytest.approx(0.0176, abs=1e-4)
    # The kinds weigh 4 : 2 : 1, the sides equally.
    won = {n: {"attacker": dict(side, win_rate=0.0), "defender": side} for n in names}
    won["hero"]["attacker"] = dict(side, win_rate=1.0)
    rates = stratified(won)
    assert rates["kinds"]["hero"] == pytest.approx(0.875)
    assert rates["win_rate"] == pytest.approx((4 * 0.875 + 3 * 0.375) / 7)
    # The neutral set: the attacker alone, the kinds at 780 : 20 : 25.
    names = [_ffi.ENCOUNTER_KINDS[k] for k in NEUTRAL_V1_WEIGHTS]
    rates = stratified({n: {"attacker": side} for n in names}, "neutral-v1")
    shares = sum((w / 825) ** 2 for w in NEUTRAL_V1_WEIGHTS.values())
    assert rates["win_rate"] == pytest.approx(0.75)
    assert rates["interval"] == pytest.approx(1.96 * math.sqrt(shares * 0.75 * 0.25 / 500))
    won = {n: {"attacker": dict(side, win_rate=0.0)} for n in names}
    won["guards"]["attacker"] = dict(side, win_rate=1.0)
    rates = stratified(won, "neutral-v1")
    assert rates["kinds"]["guards"] == 1.0 and rates["win_rate"] == pytest.approx(20 / 825)


def test_random_plays_the_report_set_against_a_bot() -> None:
    def make(seed: int, side: str):
        return random_agent(np.random.default_rng(seed))

    first = report(make, opponent="aggressor", per_kind=2, max_steps=300)
    again = report(make, opponent="aggressor", per_kind=2, max_steps=300)
    assert first["games"] == 12 and first["skipped"] == 0
    assert list(first["kinds"]) == ["hero", "siege_hero", "siege_garrison"]
    for kind in first["by_kind"].values():
        assert kind["attacker"]["episodes"] == kind["defender"]["episodes"] == 2
    keep = ("win_rate", "interval", "value", "kinds", "by_kind")
    assert {k: first[k] for k in keep} == {k: again[k] for k in keep}
    assert 0.0 <= first["win_rate"] <= 1.0 and first["interval"] >= 0.0
    with pytest.raises(ValueError, match="episodes per kind"):
        report(make, per_kind=REPORT_V1_ENVS + 1)


def test_random_attacks_the_system_actor_on_the_neutral_set() -> None:
    def make(seed: int, side: str):
        assert side == "attacker"
        return random_agent(np.random.default_rng(seed))

    first = report(make, opponent="system", held_out="neutral-v1", per_kind=2, max_steps=300)
    assert first["games"] == 6 and list(first["kinds"]) == ["wandering", "guards", "siege_neutral"]
    assert all(list(kind) == ["attacker"] for kind in first["by_kind"].values())
    again = report(make, opponent="system", held_out="neutral-v1", per_kind=2, max_steps=300)
    assert first["by_kind"] == again["by_kind"] and first["win_rate"] == again["win_rate"]
    with pytest.raises(ValueError, match="system actor"):
        report(make, opponent="random", held_out="neutral-v1", per_kind=2)
    with pytest.raises(ValueError, match="not a stratified set"):
        report(make, held_out="v0")


@pytest.mark.parametrize("held_out", list(STRATIFIED))
def test_the_bots_tables_are_the_sets_and_the_engines(held_out: str) -> None:
    # `h3 ai eval --set <set> --json`, committed: its strata are the set's, its engine identity
    # this engine's (else regenerate it: `python/records/README.md`), and its rates are the Python
    # report's formula on its own pairs (the neutral set's against the system actor, attacking).
    table = bots_table(held_out)
    _, _, set_weights, sides = STRATIFIED[held_out]
    weights = [s["weight"] for s in table["strata"]]
    assert weights == [
        w * weights[0] // next(iter(set_weights.values())) for w in set_weights.values()
    ]
    batch = BattleBatch(1)
    batch.add_encounters()
    entry = batch.entries()[0]
    batch.close()
    engine = table["engine"]
    versions = _ffi.versions()
    assert engine["encounters"] == versions["encounters"]
    assert engine["system_policy"] == versions["system_policy"]
    assert (engine["core"], engine["battle"], engine["adventure"]) == (
        entry["ruleset_hash"],
        entry["battle_ruleset_hash"],
        entry["adventure_ruleset_hash"],
    )
    assert table["max_steps"] == 2000
    assert table["policies"] == ["random", "aggressor", "greedy", "system"]
    assert table["defender"] == (None if len(sides) == 2 else "system")
    games = {(p["attacker"], p["defender"], p["stratum"]): p for p in table["pairs"]}
    for (policy, opponent), rates in table["by_pair"].items():
        by_kind = {}
        for s in table["strata"]:
            att = games[(policy, opponent, s["name"])]
            by_kind[s["name"]] = {
                "attacker": {
                    "win_rate": att["wins"][0] / att["episodes"],
                    "reward": att["reward"][0],
                    "episodes": att["episodes"],
                },
            }
            if "defender" in sides:
                dfn = games[(opponent, policy, s["name"])]
                by_kind[s["name"]]["defender"] = {
                    "win_rate": dfn["wins"][1] / dfn["episodes"],
                    "reward": dfn["reward"][1],
                    "episodes": dfn["episodes"],
                }
        mine = stratified(by_kind, held_out)
        for key in ("win_rate", "interval", "value"):
            assert math.isclose(mine[key], rates[key], abs_tol=2e-4), (policy, opponent, key)
    with pytest.raises(ValueError, match=f"not the {held_out} set's table"):
        other = next(name for name in STRATIFIED if name != held_out)
        bots_table(held_out, BOTS_TABLES[other])
