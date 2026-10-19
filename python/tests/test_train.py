"""The smoke training (E05 T8b, `docs/epics/E05-design.md` §8): the network's masked choices, its
log-probabilities, GAE over two sides, an update, the held-out evaluation and a checkpoint. They
need the group `train` (`uv sync --group train`) and are skipped without it."""

from __future__ import annotations

import json

import numpy as np
import pytest
from conftest import scenario

from h3env import BattleBatch, _ffi

torch = pytest.importorskip("torch")

from h3env.train import EVAL_SEED, Config, Policy, Rollout, gae, train  # noqa: E402
from h3env.train.evaluate import episode_seeds, play, random_agent  # noqa: E402


def casting_policy(batch: BattleBatch) -> Policy:
    """A small network that prefers the casts where they are legal, so that their heads are
    chosen too."""
    torch.manual_seed(0)
    policy = Policy.for_batch(batch, hidden=32, embed=8)
    with torch.no_grad():
        policy.entry_head.bias[[_ffi.HERO_CAST, _ffi.CREATURE_CAST]] = 30.0
    return policy


def mixed_batch() -> BattleBatch:
    batch = BattleBatch(8, threads=2, seed=3, max_steps=300)
    batch.add_scenario(scenario("full"))
    batch.add_scenario(scenario("siege"))
    batch.add_encounters()
    batch.reset()
    return batch


def test_choices_keep_to_the_masks_and_recompute_their_log_probabilities() -> None:
    batch = mixed_batch()
    policy = casting_policy(batch)
    generator = torch.Generator().manual_seed(1)
    calls = 60
    rollout = Rollout(calls, batch.envs, batch)
    casts = 0
    for t in range(calls):
        choice = policy.act(batch, generator)
        rollout.store(t, batch, choice)
        live = batch.to_move >= 0
        rows = np.nonzero(live)[0]
        assert (batch.entry_mask[rows, choice.entry[rows]] == 1).all()
        for row in np.nonzero(choice.cast & live)[0]:
            casts += 1
            assert choice.spell_mask[row, choice.spell[row]] == 1
            assert choice.target_mask[row, choice.target[row]] == 1
            assert choice.second_mask[row, choice.second[row]] == 1
        assert (choice.spell[~choice.cast] == 0).all()
        a = batch.actions
        a["entry"], a["spell"] = choice.entry, choice.spell
        a["target"], a["second"] = choice.target, choice.second
        batch.step()  # raises if an env with a side to move refuses its action
    assert casts > 0
    # The stored samples' log-probabilities under the same weights, recomputed for the update.
    valid = (rollout.side >= 0).reshape(-1)

    def flat(array, dtype=None):
        x = torch.from_numpy(array.reshape(-1, *array.shape[2:])[valid])
        return x if dtype is None else x.to(dtype)

    with torch.no_grad():
        logp, ent, value = policy.evaluate(
            flat(rollout.obs),
            flat(rollout.entry_mask),
            flat(rollout.entry),
            flat(rollout.cast),
            (
                flat(rollout.spell_mask),
                flat(rollout.spell),
                flat(rollout.target_mask),
                flat(rollout.target),
                flat(rollout.second_mask),
                flat(rollout.second),
            ),
        )
    np.testing.assert_allclose(logp.numpy(), flat(rollout.logprob).numpy(), atol=1e-4)
    np.testing.assert_allclose(value.numpy(), flat(rollout.value).numpy(), atol=1e-5)
    assert torch.isfinite(ent).all() and (ent >= -1e-5).all()


def test_gae_over_two_sides_by_hand() -> None:
    # Env 0: the attacker acts twice, the defender once, the attacker ends the battle (+0.6 for
    # it, -0.6 for the defender); the next episode's defender acts, still open at the end (the
    # attacker to move: its value, 9, is no ended battle's next value). Env 1: the
    # defender acts twice and the battle is truncated (bootstrap 0.25); the next episode's
    # attacker, the defender (open at the end) and the attacker again, which is to move after the
    # rollout (its value 0.7).
    side = np.array([[0, 1], [0, 1], [1, 0], [0, 1], [1, 0]], np.int8)
    value = np.array([[0.1, 0.3], [0.2, 0.1], [-0.3, 0.2], [0.4, -0.1], [0.5, 0.6]], np.float32)
    reward = np.zeros((5, 2), np.float32)
    reward[3, 0], reward[2, 0] = 0.6, -0.6
    closed = np.array([[1, 1], [1, 1], [1, 1], [1, 0], [0, 1]], bool)
    terminal = np.zeros((5, 2), bool)
    terminal[2, 0] = terminal[3, 0] = terminal[1, 1] = True
    truncated = np.zeros((5, 2), bool)
    truncated[1, 1] = True
    final_value = np.zeros((5, 2), np.float32)
    final_value[1, 1] = 0.25
    adv, used = gae(
        side,
        value,
        reward,
        closed,
        terminal,
        truncated,
        final_value,
        last_side=np.array([0, 0], np.int8),
        last_value=np.array([9.0, 0.7], np.float32),
        gamma=0.9,
        lam=0.8,
    )
    g, gl = 0.9, 0.9 * 0.8
    a3 = 0.6 - 0.4
    a1 = g * 0.4 - 0.2 + gl * a3
    e4 = g * 0.7 - 0.6
    e2 = g * 0.6 - 0.2 + gl * e4
    e1 = g * 0.25 - 0.1
    expected = np.array(
        [
            [g * 0.2 - 0.1 + gl * a1, g * 0.1 - 0.3 + gl * e1],
            [a1, e1],
            [-0.6 + 0.3, e2],
            [a3, 0.0],
            [0.0, e4],
        ],
        np.float32,
    )
    np.testing.assert_allclose(adv, expected, atol=1e-6)
    np.testing.assert_array_equal(used, closed)


def test_the_evaluation_plays_fixed_held_out_episodes() -> None:
    settings = dict(episodes=12, envs=4, max_steps=300)
    first = play(random_agent(np.random.default_rng(5)), **settings)
    again = play(random_agent(np.random.default_rng(5)), **settings)
    assert first == again
    for side in ("attacker", "defender"):
        assert first[side]["episodes"] == 12
        assert sum(k["episodes"] for k in first[side]["by_kind"].values()) == 12
    assert set(first["attacker"]["by_kind"]) <= {"hero", "siege_hero", "siege_garrison"}
    # Disjoint from a training batch's episodes (another batch seed).
    training = BattleBatch(4, seed=1)
    training.add_encounters()
    training.reset()
    seeds = episode_seeds(12, 4)
    assert len(seeds) == 12 and seeds.isdisjoint(int(s) for s in training.episode_seed)
    assert EVAL_SEED != Config().seed


def test_a_short_run_writes_its_files_and_round_trips_its_checkpoint(tmp_path) -> None:
    config = Config(
        updates=2,
        envs=8,
        threads=2,
        max_steps=300,
        rollout=16,
        epochs=2,
        minibatches=2,
        hidden=32,
        embed=8,
        eval_every=0,
        eval_episodes=8,
        eval_envs=8,
        eval_sampled=False,
        device="cpu",
    )
    trainer = train(config, tmp_path)
    records = [json.loads(line) for line in (tmp_path / "metrics.jsonl").read_text().splitlines()]
    updates = [r for r in records if not r.get("eval")]
    evals = [r for r in records if r.get("eval")]
    # The untrained network with Random's own rates first, then the final evaluation.
    assert len(updates) == 2 and [e["update"] for e in evals] == [0, 2]
    assert "random" in evals[0] and not evals[0]["final"] and evals[1]["final"]
    assert all(e["held_out"] for e in evals)
    for r in updates:
        assert all(np.isfinite(r[k]) for k in ("policy_loss", "value_loss", "entropy", "kl"))
        assert r["samples"] > 0
    meta = json.loads((tmp_path / "config.json").read_text())
    assert meta["abi"] == _ffi.ABI_VERSION and meta["config"]["hidden"] == 32
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    restored = Policy.for_batch(trainer.batch, hidden=32, embed=8)
    restored.load_state_dict(saved["policy"])
    restored_state = restored.state_dict()
    for name, weights in trainer.policy.state_dict().items():
        assert torch.equal(weights, restored_state[name]), name
    assert saved["updates"] == 2
