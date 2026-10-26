"""The behaviour analysis without torch (E06 T6a, `h3env.train.inspect`): its battles are the
ladder's, its decisions add up, a clean strike's effects agree with the kill preview, and its
reading of the observation."""

from __future__ import annotations

import numpy as np
import pytest

from h3env import _ffi
from h3env.train import layout as L
from h3env.train.evaluate import play, random_agent, report
from h3env.train.inspect import (
    CATEGORIES,
    Recorder,
    category,
    inspect,
    lines,
    spell_names,
    view,
)


def _random(seed: int, side: str):
    return random_agent(np.random.default_rng(seed))


def test_the_analysis_plays_the_ladders_battles() -> None:
    # The same episodes and draws as `report`'s (the preview changes no play): each kind's and
    # side's battles and wins as its rates say, and every decision in one category.
    per_kind = 4
    result = inspect(_random, opponents=["greedy"], per_kind=per_kind, max_steps=300)["greedy"]
    plain = report(_random, opponent="greedy", per_kind=per_kind, max_steps=300)
    assert result["ladder"]["win_rate"] == plain["win_rate"]
    groups = result["groups"]
    assert set(groups) == {"hero", "siege_hero", "siege_garrison", "all"}
    for kind, sides in plain["by_kind"].items():
        for side, rates in sides.items():
            g = groups[kind][side]
            assert g["all"]["battles"] == rates["episodes"] == per_kind
            assert g["all"]["won"] == rates["win_rate"]
            assert g.get("won", {"battles": 0})["battles"] == round(rates["win_rate"] * per_kind)
            assert sum(v for v in g["all"]["mix"].values() if v) == pytest.approx(1.0)
    whole = groups["all"]
    assert whole["battles"] == 6 * per_kind and whole["decisions"] > 1
    # Random strikes rarely at the best target, shoots when a shot is legal at random.
    t = whole["targeting"]
    assert 0 < t["strikes"] <= t["strike_legal"] and 0.0 <= t["at_best"] <= 1.0
    assert t["kills"] is None or t["kills"] <= 1.0
    assert 0.0 <= whole["trades"]["first"] <= 1.0
    assert whole["trades"]["melee_avoidable"] <= whole["trades"]["melee_into_retaliation"]
    assert all(0.0 <= share <= 1.0 for share in whole["turns"].values())
    text = lines("greedy", result)
    assert text[0].startswith("against greedy: win rate") and text[-1].startswith("hero's spells")
    assert sum(line.startswith("all") for line in text) == 2


def test_a_clean_strike_kills_at_least_the_previewed_least() -> None:
    # The reading's check: the target of a strike seen alone (no opponent turn between) lost at
    # least the least kills the preview showed (luck, a second strike, an area may add more; a
    # Vampire Lords' retaliation drains some back). A commanded turret's shots are left out: the
    # preview does not follow BAT-SIE-018.
    result = inspect(_random, opponents=["aggressor"], per_kind=12, max_steps=300)
    trades = result["aggressor"]["groups"]["all"]["trades"]
    assert trades["melee"]["strikes"] + trades["shot"]["strikes"] > 50
    assert trades["least_killed"] >= 0.97, trades
    # A melee strike's retaliation costs the striker; a shot rarely does.
    assert trades["melee"]["taken"] > trades["shot"]["taken"] >= 0.0


def test_the_neutral_set_is_attacked_against_the_system_actor() -> None:
    result = inspect(_random, opponents=["greedy"], held_out="neutral-v1", per_kind=2)
    assert list(result) == ["system"]
    groups = result["system"]["groups"]
    assert set(groups) == {"wandering", "guards", "siege_neutral", "all"}
    assert all(list(sides) == ["attacker"] for k, sides in groups.items() if k != "all")
    assert groups["all"]["hero"]["battles"] <= groups["all"]["battles"] == 6
    with pytest.raises(ValueError, match="stratified"):
        inspect(_random, held_out="v0")


def test_the_recorder_needs_the_kill_preview() -> None:
    with pytest.raises(ValueError, match="kill preview"):
        play(Recorder(_random(0, "attacker"), "attacker"), episodes=1, envs=1, sides=("attacker",))


def test_the_categories_and_the_observations_reading() -> None:
    entries = np.array(
        [
            0,
            1,
            L.MOVE_BASE,
            L.ATTACK_BASE,
            L.SHOOT_BASE,
            _ffi.HERO_CAST,
            _ffi.CREATURE_CAST,
            L.HEAL_BASE,
            5,
        ]
    )
    rounds = np.array([1] * 8 + [0])
    assert [CATEGORIES[c] for c in category(entries, rounds)] == [
        "wait",
        "defend",
        "move",
        "melee",
        "shot",
        "hero_cast",
        "creature_cast",
        "other",
        "tactics",
    ]
    turret = np.arange(9) == 4
    assert CATEGORIES[category(entries, rounds, turret)[4]] == "turret"
    obs = np.zeros((1, L.OBSERVATION_LEN), np.float32)
    obs[0, L.ROUND] = 3
    hero = obs[0, L.HEROES : L.OWN_HERO].reshape(2, L.HERO_PUBLIC_FEATURES)
    hero[1, [0, L.HERO_MANA, L.HERO_MAX_MANA]] = 1, 4, 20
    stacks = obs[0, L.STACKS : L.ORDER].reshape(L.STACK_CAPACITY, L.STACK_FEATURES)
    # Own: 3 creatures of 10 hit points, the top one at 4 (24), 5 at the start; a war machine.
    # The opponent's: 2 of 7 (14), its turn started, it defended; a turret, the active stack; a
    # dead stack.
    for slot, values in enumerate(
        [
            {L.OWN: 1, L.COUNT: 3, L.TOP_HP: 4, L.MAX_HP: 10, L.BASE_COUNT: 5},
            {L.OWN: 1, L.COUNT: 1, L.TOP_HP: 250, L.MAX_HP: 250, L.WAR_MACHINE: 1},
            {L.COUNT: 2, L.TOP_HP: 7, L.MAX_HP: 7, L.STARTED: 1, L.DEFENDING: 1, L.BASE_COUNT: 2},
            {L.COUNT: 1, L.TOP_HP: 10, L.MAX_HP: 10, L.TURRET: 1, L.ACTIVE: 1},
            {L.COUNT: 0, L.TOP_HP: 0, L.MAX_HP: 9, L.BASE_COUNT: 4},
        ]
    ):
        stacks[slot, L.PRESENT] = 1
        for column, value in values.items():
            stacks[slot, column] = value
    v = view(obs)
    assert v["round"][0] == 3 and v["turret"][0]
    assert v["hp"][0].tolist() == [24, 14] and v["start"][0].tolist() == [50, 14 + 36]
    assert v["hero"][0].tolist() == [False, True] and v["mana"][0, 1] == 4
    # Turn flags: the opponent's stacks only (8 marks one), the started one's bit set; its
    # creature stacks' turns this round: one started, none waited, one defends.
    assert v["turns"][0, :5].tolist() == [0, 0, 12, 8, 8]
    assert v["opponent_turns"][0].tolist() == [1, 0, 1]


def test_the_spells_are_named_in_load_order() -> None:
    names = spell_names()
    assert names[0] == "Summon Boat" and names[-1] == "Titan's Lightning Bolt"
    assert "Magic Arrow" in names and len(names) == len(set(names))
