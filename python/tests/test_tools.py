"""Single envs, fork and the state hash: trusted tooling (E05 T6a)."""

from __future__ import annotations

import numpy as np
import pytest
from conftest import scenario

from h3env import H3Error, _ffi, sample
from h3env.tools import Env


def random_step(env: Env, rng: np.random.Generator) -> None:
    side = env.to_move()
    entry = int(sample(env.legal_mask(side)[None], rng)[0])
    spell = int(sample(env.spell_mask(side, entry)[None], rng)[0])
    target = int(sample(env.target_mask(side, entry, spell)[None], rng)[0])
    second = int(sample(env.second_mask(side, entry, spell, target)[None], rng)[0])
    env.step(side, entry, spell, target, second)


def test_a_fork_is_independent_and_a_reset_replays() -> None:
    env = Env.synthetic(2, seed=4)
    start = env.state_hash()
    rng = np.random.default_rng(0)
    for _ in range(5):
        random_step(env, rng)
    with env.fork() as fork:
        assert fork.state_hash() == env.state_hash()
        random_step(fork, rng)
        assert fork.state_hash() != env.state_hash()
    env.reset(4)
    assert env.state_hash() == start


def test_a_scenario_env_plays_to_the_end() -> None:
    env = Env.scenario(scenario("duel"), seed=1, max_steps=500)
    rng = np.random.default_rng(1)
    while env.to_move() >= 0:
        random_step(env, rng)
    with pytest.raises(H3Error, match="H3_ERR_EPISODE_OVER"):
        env.step(0, 0)


def test_refusals_raise() -> None:
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT: .+"):
        Env.scenario("[scenario]\n", seed=0)
    with pytest.raises(H3Error, match="no synthetic scenario 9"):
        Env.synthetic(9, seed=0)
    env = Env.synthetic(0, seed=0)
    with pytest.raises(H3Error, match="H3_ERR_ARGUMENT"):
        env.observe(2)
    side = env.to_move()
    with pytest.raises(H3Error, match="H3_ERR_INVALID_ACTION|H3_ERR_REJECTED"):
        env.step(side, _ffi.HERO_CAST, 0, 0, 0)
