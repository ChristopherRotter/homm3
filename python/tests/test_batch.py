"""The batch through the binding (E05 T6a): it steps as N single envs, its masks are the single
envs' without the escapes, every legal action applies, and the encounter source works."""

from __future__ import annotations

import numpy as np
import pytest
from conftest import bits, scenario

from h3env import BattleBatch, H3Error, _ffi, episode_seed
from h3env.tools import Env


def heads(action) -> tuple[int, int, int, int]:
    """An action record's heads as ints (entry, spell, target, second)."""
    return tuple(int(action[f]) for f in ("entry", "spell", "target", "second"))


@pytest.mark.parametrize("name", ["full", "siege"])
def test_the_batch_steps_as_single_envs(name: str) -> None:
    text = scenario(name, human=True)
    n, seed, max_steps = 6, 11, 120
    batch = BattleBatch(n, threads=3, seed=seed, max_steps=max_steps)
    batch.add_scenario(text)
    batch.reset()
    singles = [Env.scenario(text, episode_seed(seed, i, 0), max_steps) for i in range(n)]
    rng = np.random.default_rng(5)
    escapes = 0
    ended = 0

    def same(i: int) -> None:
        nonlocal escapes
        env = singles[i]
        side = env.to_move()
        assert batch.to_move[i] == side
        assert batch.episode_seed[i] == episode_seed(seed, i, int(batch.episode[i]))
        assert batch.encounter[i] == _ffi.ENCOUNTER_NONE
        np.testing.assert_array_equal(bits(batch.obs[i]), bits(env.observe(side)))
        mask = env.legal_mask(side)
        escapes += int(mask[_ffi.RETREAT]) + int(mask[_ffi.SURRENDER])
        mask[[_ffi.RETREAT, _ffi.SURRENDER]] = 0
        np.testing.assert_array_equal(batch.entry_mask[i], mask)

    for i in range(n):
        same(i)
    for _ in range(300):
        actions = batch.random_actions(rng).copy()
        # The head masks for the chosen entries are the single envs'.
        e = actions["entry"]
        spells = batch.spell_mask(e).copy()
        targets = batch.target_mask(e, actions["spell"]).copy()
        seconds = batch.second_mask(e, actions["spell"], actions["target"]).copy()
        for i in range(n):
            side = singles[i].to_move()
            entry, spell, target, _ = heads(actions[i])
            if entry in (_ffi.HERO_CAST, _ffi.CREATURE_CAST):
                np.testing.assert_array_equal(spells[i], singles[i].spell_mask(side, entry))
                np.testing.assert_array_equal(
                    targets[i], singles[i].target_mask(side, entry, spell)
                )
                np.testing.assert_array_equal(
                    seconds[i], singles[i].second_mask(side, entry, spell, target)
                )
        episodes = batch.episode.copy()
        assert batch.step() == _ffi.OK
        assert (batch.status == _ffi.OK).all()
        for i in range(n):
            env = singles[i]
            r = env.step(env.to_move(), *heads(actions[i]))
            assert batch.done[i] == r.done
            assert batch.reward[i, 0] == -batch.reward[i, 1]
            if not r.done:
                assert batch.result[i] == _ffi.RESULT_NONE
                assert (batch.reward[i] == 0).all()
            else:
                ended += 1
                assert (batch.truncated[i], batch.result[i]) == (r.truncated, r.result)
                if r.truncated:
                    assert (batch.reward[i] == 0).all()
                    for viewer in (0, 1):
                        np.testing.assert_array_equal(
                            bits(batch.final_obs[i, viewer]), bits(env.observe(viewer))
                        )
                assert batch.episode[i] == episodes[i] + 1
                singles[i] = Env.scenario(
                    text, episode_seed(seed, i, int(batch.episode[i])), max_steps
                )
            same(i)
    assert ended > 0
    # The batch masks out escapes the single envs allow: the constants name the right entries.
    assert escapes > 0


def test_the_thread_count_changes_nothing() -> None:
    outputs = []
    for threads in (0, 4):
        batch = BattleBatch(8, threads=threads, seed=3, max_steps=200)
        batch.add_scenario(scenario("full"))
        batch.add_scenario(scenario("siege"), weight=2, defender="random")
        batch.reset()
        rng = np.random.default_rng(1)
        seen = []
        for _ in range(150):
            batch.random_actions(rng)
            batch.step()
            seen.append(
                (
                    bits(batch.obs).copy(),
                    batch.reward.copy(),
                    batch.scenario.copy(),
                    batch.done.copy(),
                )
            )
        outputs.append(seen)
    for a, b in zip(*outputs, strict=True):
        for x, y in zip(a, b, strict=True):
            np.testing.assert_array_equal(x, y)


def test_a_reset_replays_the_same_episodes() -> None:
    batch = BattleBatch(4, seed=9)
    batch.add_scenario(scenario("skirmish"))
    batch.reset()
    first = bits(batch.obs).copy()
    batch.random_actions(np.random.default_rng(0))
    batch.step()
    batch.reset()
    np.testing.assert_array_equal(bits(batch.obs), first)
    assert (batch.episode == 0).all()


def test_escapes_are_refused_and_change_nothing() -> None:
    batch = BattleBatch(3, seed=2)
    batch.add_scenario(scenario("full"))
    batch.reset()
    obs = bits(batch.obs).copy()
    batch.random_actions(np.random.default_rng(0))
    batch.actions["entry"][1] = _ffi.RETREAT
    with pytest.raises(H3Error, match="env 1: H3_ERR_INVALID_ACTION"):
        batch.step()
    assert batch.status[1] == _ffi.ERR_INVALID_ACTION
    assert (batch.status[[0, 2]] == _ffi.OK).all()
    np.testing.assert_array_equal(bits(batch.obs[1]), obs[1])
    batch.random_actions(np.random.default_rng(1))
    batch.actions["entry"][1] = _ffi.SURRENDER
    assert batch.step(check=False) == _ffi.ERR_REJECTED
    np.testing.assert_array_equal(bits(batch.obs[1]), obs[1])


def test_encounters_are_an_episode_source() -> None:
    n = 16
    batch = BattleBatch(n, threads=2, seed=4, max_steps=400)
    batch.add_scenario(scenario("duel"))
    batch.add_encounters(weight=3)
    batch.reset()
    rng = np.random.default_rng(2)
    kinds: set[int] = set()
    for _ in range(200):
        assert ((batch.scenario == 0) == (batch.encounter == _ffi.ENCOUNTER_NONE)).all()
        kinds |= set(batch.encounter[batch.scenario == 1].tolist())
        batch.random_actions(rng)
        batch.step()
        assert (batch.reward[:, 0] == -batch.reward[:, 1]).all()
    # The default weights leave out the neutral kinds (A12, until E23).
    assert kinds and kinds <= {
        _ffi.ENCOUNTER_HERO,
        _ffi.ENCOUNTER_SIEGE_HERO,
        _ffi.ENCOUNTER_SIEGE_GARRISON,
    }


def test_neutral_encounters_play_the_defender_inside_the_call() -> None:
    batch = BattleBatch(8, seed=5, max_steps=300)
    batch.add_encounters(kinds={"wandering": 1, "guards": 1})
    batch.reset()
    rng = np.random.default_rng(3)
    for _ in range(100):
        assert set(batch.encounter.tolist()) <= {_ffi.ENCOUNTER_WANDERING, _ffi.ENCOUNTER_GUARDS}
        assert (batch.to_move == 0).all()
        batch.random_actions(rng)
        batch.step()


def test_encounter_seats_replace_the_encounters_own() -> None:
    # The default distribution against a random attacker: the caller always defends.
    batch = BattleBatch(8, seed=6, max_steps=300)
    batch.add_encounters(attacker="random", defender="external")
    batch.reset()
    rng = np.random.default_rng(4)
    for _ in range(100):
        assert (batch.to_move == 1).all()
        batch.random_actions(rng)
        batch.step()


def test_refused_calls_raise_with_the_engines_message() -> None:
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        BattleBatch(0)
    with pytest.raises(ValueError, match="currency"):
        BattleBatch(1, currency="mana")
    batch = BattleBatch(2)
    with pytest.raises(H3Error, match="H3_ERR_NOT_READY"):
        batch.reset()
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        batch.add_scenario("not a scenario")
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        batch.add_scenario(scenario("duel"), attacker="random", defender="random")
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        batch.add_encounters(kinds=[0] * 6)
    with pytest.raises(ValueError, match="unknown encounter kinds"):
        batch.add_encounters(kinds={"dragons": 1})
    with pytest.raises(ValueError, match="six kind weights"):
        batch.add_encounters(kinds=[1, 2])
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        batch.add_encounters(attacker="random", defender="random")
    with pytest.raises(ValueError, match="both seats"):
        batch.add_encounters(attacker="random")
    batch.add_scenario(scenario("duel"))
    with pytest.raises(H3Error, match="H3_ERR_NOT_READY"):
        batch.step()
    batch.close()
    with pytest.raises(H3Error, match="closed"):
        batch.reset()


def test_sample_draws_every_legal_index_and_only_those() -> None:
    from h3env import sample

    mask = np.array([[0, 1, 0, 1, 1], [0, 0, 0, 0, 0], [1, 0, 0, 0, 0]], np.uint8)
    rng = np.random.default_rng(0)
    drawn = np.array([sample(mask, rng) for _ in range(600)])
    assert set(drawn[:, 0].tolist()) == {1, 3, 4}
    assert (drawn[:, 1] == 0).all() and (drawn[:, 2] == 0).all()
