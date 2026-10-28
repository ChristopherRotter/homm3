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


def test_the_preview_shows_the_kills_of_the_legal_strikes_only() -> None:
    # v33 (AI-Q7 (b)): the acting side's previewed kills, 0 for every entry not legal; writing
    # them changes no play.
    batches = [
        BattleBatch(8, threads=2, seed=5, max_steps=200, preview=preview)
        for preview in (False, True)
    ]
    plain, previewed = batches
    assert plain.preview is None
    assert previewed.preview.shape == (8, previewed.preview_entries, 2)
    for batch in batches:
        batch.add_scenario(scenario("full"))
        batch.reset()
    rng = [np.random.default_rng(2), np.random.default_rng(2)]
    base, entries = previewed.preview_base, previewed.preview_entries
    shown = 0
    for _ in range(150):
        kills = previewed.preview
        legal = previewed.entry_mask[:, base : base + entries].astype(bool)
        assert (kills[..., 0] <= kills[..., 1]).all()
        assert (kills[~legal] == 0).all()
        shown += int((kills[..., 1] > 0).sum())
        for batch, r in zip(batches, rng, strict=True):
            batch.random_actions(r)
            batch.step()
        np.testing.assert_array_equal(bits(plain.obs), bits(previewed.obs))
    assert shown > 0


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


def test_the_binding_refuses_what_the_engine_would_misread() -> None:
    # E05 T9b: the engine reads N values from each head array and writes into the arrays whose
    # addresses it holds; ctypes would wrap a negative or oversized number silently.
    batch = BattleBatch(3, seed=2)
    batch.add_scenario(scenario("full"))
    batch.reset()
    entries = np.zeros(3, np.uint32)
    with pytest.raises(ValueError, match="entries"):
        batch.spell_mask(entries[:1])
    with pytest.raises(ValueError, match="spells"):
        batch.target_mask(entries, np.zeros(4, np.uint16))
    with pytest.raises(ValueError, match="targets"):
        batch.second_mask(entries, np.zeros(3, np.uint16), np.zeros((3, 1), np.uint16))
    with pytest.raises(AttributeError):
        batch.obs = batch.obs.copy()
    batch.actions = batch.actions[:2]
    with pytest.raises(ValueError, match="actions"):
        batch.step()
    for options in ({"mana_price": -1}, {"seed": -1}, {"max_steps": 1 << 32}):
        with pytest.raises(ValueError, match="out of range"):
            BattleBatch(1, **options)
    with pytest.raises(ValueError, match="out of range"):
        BattleBatch(-1)
    # v31: the engine refuses an absurd N (it aborted on its allocation).
    with pytest.raises(H3Error, match=f"at most {_ffi.BATCH_MAX_ENVS}"):
        BattleBatch(_ffi.BATCH_MAX_ENVS + 1)
    with pytest.raises(ValueError, match="weight"):
        batch.add_scenario(scenario("duel"), weight=-1)


def test_encounters_are_an_episode_source() -> None:
    n = 16
    batch = BattleBatch(n, threads=2, seed=4, max_steps=400)
    batch.add_scenario(scenario("duel"))
    batch.add_encounters(weight=3)
    # v31: each entry's engine identity, for a run's record.
    duel, encounters = batch.entries()
    assert (duel["kind"], duel["weight"], duel["adventure_ruleset_hash"]) == ("scenario", 1, None)
    assert (encounters["kind"], encounters["weight"]) == ("encounters", 3)
    assert duel["battle_ruleset_hash"] != encounters["battle_ruleset_hash"]
    assert all(len(e["ruleset_hash"]) == 16 for e in (duel, encounters))
    assert int(encounters["adventure_ruleset_hash"], 16) != 0
    batch.reset()
    rng = np.random.default_rng(2)
    kinds: set[int] = set()
    for _ in range(200):
        assert ((batch.scenario == 0) == (batch.encounter == _ffi.ENCOUNTER_NONE)).all()
        kinds |= set(batch.encounter[batch.scenario == 1].tolist())
        batch.random_actions(rng)
        batch.step()
        assert (batch.reward[:, 0] == -batch.reward[:, 1]).all()
    # The default weights leave out the neutral kinds (A12, E06 T7).
    assert kinds and kinds <= {
        _ffi.ENCOUNTER_HERO,
        _ffi.ENCOUNTER_SIEGE_HERO,
        _ffi.ENCOUNTER_SIEGE_GARRISON,
    }


def test_the_system_actor_plays_inside_the_call() -> None:
    # E06 T7 (v32): the system actor, the original's computer, as a scenario's seat on either side.
    batch = BattleBatch(4, seed=7, max_steps=300)
    batch.add_scenario(scenario("full"), defender="system")
    batch.add_scenario(scenario("siege"), attacker="system")
    batch.reset()
    rng = np.random.default_rng(5)
    ended = 0
    for _ in range(150):
        assert (batch.to_move == batch.scenario).all()
        batch.random_actions(rng)
        batch.step()
        ended += int(batch.done.sum())
    assert ended > 0


def test_neutral_encounters_play_the_defender_inside_the_call() -> None:
    # The neutral kinds' defender is the system actor (A12, E06 T7).
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


def test_a_world_scores_a_decided_battle_by_the_batchs_reward_v1() -> None:
    # E25 T1a (C ABI v35): a world of a battle that hides nothing (the duel: no hero) on its
    # episode's seed is the env's battle; played as the batch plays the env, it ends at the same
    # step with the batch's reward v1 for each side (its currency and mana price), not ±1.
    text, n, seed = scenario("duel"), 6, 11
    batch = BattleBatch(n, seed=seed, mana_price=3)
    batch.add_scenario(text)
    batch.reset()
    worlds = {}
    for e in range(n):
        side = int(batch.to_move[e])
        world = batch.determinize(e, side, int(batch.episode_seed[e]))
        single = Env.scenario(scenario("duel", human=True), episode_seed(seed, e, 0))
        assert world.state_hash() == single.state_hash()
        assert np.array_equal(world.observe(side), batch.obs[e])
        single.close()
        worlds[e] = world
    rng = np.random.default_rng(3)
    for _ in range(5_000):
        if not worlds:
            break
        actions = batch.random_actions(rng)
        ends = {}
        for e, world in worlds.items():
            r = world.step(int(batch.to_move[e]), *heads(actions[e]))
            if r.done:
                ends[e] = np.array([r.reward_attacker, r.reward_defender], np.float32)
            else:
                assert (r.reward_attacker, r.reward_defender) == (0.0, 0.0)
        batch.step()
        for e in list(worlds):
            assert bool(batch.done[e]) == (e in ends)
            if e in ends:
                assert np.array_equal(bits(ends[e]), bits(batch.reward[e]))
                assert abs(ends[e][0]) < 1 and ends[e][0] == -ends[e][1]
                worlds.pop(e).close()
    assert not worlds


def test_a_true_world_is_the_battle_with_its_stream_reseeded() -> None:
    # E25 T1b (C ABI v35): a trusted diagnostic's world: the env's battle with nothing guessed
    # (the full armies' heroes as they are, where a determinized world guesses the other one),
    # its stream re-seeded; refused arguments raise.
    text, n, seed = scenario("full"), 3, 8
    batch = BattleBatch(n, seed=seed, max_steps=300)
    batch.add_scenario(text)
    with pytest.raises(H3Error, match="no true world"):
        batch.true_world(0, 5)
    batch.reset()
    for e in range(n):
        side = int(batch.to_move[e])
        single = Env.scenario(scenario("full", human=True), episode_seed(seed, e, 0), 300)
        world = batch.true_world(e, 5)
        assert world.state_hash() != single.state_hash()
        single.reseed(5)
        assert world.state_hash() == single.state_hash()
        for viewer in (0, 1):
            assert np.array_equal(world.observe(viewer), single.observe(viewer))
        guessed = batch.determinize(e, side, 5)
        assert guessed.state_hash() != world.state_hash()
        for env in (single, world, guessed):
            env.close()
    with pytest.raises(H3Error, match="no true world"):
        batch.true_world(n, 5)
    batch.close()


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
