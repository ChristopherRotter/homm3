"""The training (E05 T8b, `docs/epics/E05-log.md` §8; E06 T2a, T2b1): the network's masked
choices, its log-probabilities, GAE over two sides, an update, the held-out evaluation, a run's
files and record, the checkpoint and the resume. They need the group `train` (`uv sync --group
train`) and are skipped without it; `test_run.py` covers the run's lifecycle without torch."""

from __future__ import annotations

import json
import signal
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest
from conftest import scenario

from h3env import BattleBatch, _ffi

torch = pytest.importorskip("torch")

from h3env.train import EVAL_SEED, Config, Policy, Rollout, gae, resume, train  # noqa: E402
from h3env.train.evaluate import episode_seeds, ladder, load, play, random_agent  # noqa: E402
from h3env.train.ppo import Trainer  # noqa: E402
from h3env.train.run import (  # noqa: E402
    ConfigError,
    HeldOutError,
    ResumeError,
    check_record,
    segment_seed,
)

PYTHON = Path(__file__).resolve().parents[1]

#: A run small enough for a test.
SMALL = dict(
    envs=8,
    threads=2,
    max_steps=300,
    rollout=16,
    epochs=2,
    minibatches=2,
    hidden=32,
    embed=8,
    eval_episodes=8,
    eval_envs=8,
    eval_greedy=False,
    device="cpu",
)
#: The most samples one update of `SMALL` holds (its rollout's rows).
ONE_UPDATE = 16 * 8


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
        rollout.store(t, batch, choice, batch.to_move)
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
    # A subset of rows chooses as the same rows of a whole-batch call (greedy, so no draws).
    rows = np.array([1, 4, 6])
    whole, part = policy.act(batch, greedy=True), policy.act(batch, greedy=True, rows=rows)
    for name in ("entry", "spell", "target", "second", "cast"):
        np.testing.assert_array_equal(getattr(part, name), getattr(whole, name)[rows])
    np.testing.assert_allclose(part.logprob, whole.logprob[rows], atol=1e-5)
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
    # A budget just past one update's samples: two updates (or more, if they hold few).
    config = Config(**SMALL, samples=ONE_UPDATE + 1, eval_every=0, pool_every=1)
    trainer = train(config, tmp_path, text="samples = 129\n")
    records = [json.loads(line) for line in (tmp_path / "metrics.jsonl").read_text().splitlines()]
    assert records[0]["type"] == "start" and records[-1]["type"] == "end"
    assert records[-1]["reason"] == "budget"
    updates = [r for r in records if r["type"] == "update"]
    evals = [r for r in records if r["type"] == "eval"]
    n = len(updates)
    # The budget in samples: the run stops at the first update boundary at or past it.
    assert n >= 2 and updates[-2]["samples"] < config.samples <= updates[-1]["samples"]
    # The untrained network with Random's own rates first, then the final evaluation.
    assert [e["update"] for e in evals] == [0, n]
    assert "random" in evals[0] and not evals[0]["final"] and evals[1]["final"]
    for r in updates:
        assert all(np.isfinite(r[k]) for k in ("policy_loss", "value_loss", "entropy", "kl"))
        assert r["update_samples"] > 0
    assert updates[1]["samples"] == updates[0]["samples"] + updates[1]["update_samples"]
    # The learning rate decays over the samples (the horizon: the budget).
    assert updates[0]["lr"] == config.lr > updates[1]["lr"]
    # A snapshot after each update (every sample is a multiple of 1): each rollout drew from
    # the pool the snapshots before it made.
    assert [r["pool"] for r in updates] == list(range(n)) and len(trainer.pool) == n
    assert sum(r["type"] == "snapshot" for r in records) == n
    assert (tmp_path / "best.pt").exists() and evals[-1]["best"] >= evals[0]["best"]
    assert (tmp_path / "config.toml").read_text() == "samples = 129\n"
    assert not list(tmp_path.glob("*.tmp"))
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == []
    assert record["identity"]["abi"] == _ffi.ABI_VERSION and record["identity"]["device"] == "cpu"
    assert record["config"]["network"]["hidden"] == 32
    assert record["config"]["ppo"]["lr_horizon"] == ONE_UPDATE + 1
    assert record["progress"]["ended"] == "budget" and record["progress"]["updates"] == n
    assert len(record["evaluations"]) == 2 and len(record["curve"]) == 1
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    restored = Policy.for_batch(trainer.batch, hidden=32, embed=8)
    restored.load_state_dict(saved["policy"])
    restored_state = restored.state_dict()
    for name, weights in trainer.policy.state_dict().items():
        assert torch.equal(weights, restored_state[name]), name
    assert saved["updates"] == n and saved["finished"] and len(saved["pool"]) == n
    assert record["segments"][0]["seed"] == config.seed == trainer.batch_seed


def _records(out: Path) -> list[dict]:
    return [json.loads(x) for x in (out / "metrics.jsonl").read_text().splitlines()]


def _same(a: dict, b: dict) -> None:
    """Two (nested) state dicts hold equal tensors and values."""
    assert a.keys() == b.keys()
    for k in a:
        if isinstance(a[k], dict):
            _same(a[k], b[k])
        elif torch.is_tensor(a[k]):
            assert torch.equal(a[k], b[k]), k
        else:
            assert a[k] == b[k], k


def test_a_stopped_run_resumes_to_its_budget(tmp_path) -> None:
    # Ctrl+C after the third update: the run saves and stops. A trainer of the next segment takes
    # the whole state over; a resume refuses a change to the training and continues to the budget.
    config = Config(**SMALL, samples=4 * ONE_UPDATE, eval_every=0, eval_start=False, pool_every=1)

    def interrupt(record: dict) -> None:
        if record["type"] == "update" and record["update"] == 3:
            signal.raise_signal(signal.SIGINT)

    stopped = train(config, tmp_path, log=interrupt)
    assert _records(tmp_path)[-1]["type"] == "stop" and stopped.updates == 3
    state = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    assert (state["format"], state["segment"], state["updates"], state["finished"]) == (
        1,
        0,
        3,
        False,
    )
    assert state["metrics_length"] == (tmp_path / "metrics.jsonl").stat().st_size
    _same(stopped.policy.state_dict(), state["policy"])
    again = Trainer(Config(**state["config"]), segment=1)
    again.restore(state)
    _same(again.policy.state_dict(), state["policy"])
    _same(again.optimizer.state_dict()["state"], state["optimizer"]["state"])
    assert len(again.pool) == len(state["pool"]) == 3
    for net, weights in zip(again.pool, state["pool"], strict=True):
        _same(net.state_dict(), weights)
    assert torch.equal(again.generator.get_state(), state["rng"]["sampling"])
    assert torch.equal(torch.get_rng_state(), state["rng"]["torch"])
    assert again.pool_rng.bit_generator.state == state["rng"]["pool"]
    assert (again.samples, again.episodes) == (stopped.samples, stopped.episodes)
    assert again.snapshots.next == stopped.snapshots.next
    assert again.batch_seed == segment_seed(config.seed, 1) != config.seed
    again.batch.close()

    before = (tmp_path / "metrics.jsonl").read_bytes()
    with pytest.raises(ConfigError, match=r"ppo\.lr would change the training"):
        resume(tmp_path, ["ppo.lr=1e-3"])
    assert (tmp_path / "metrics.jsonl").read_bytes() == before

    budget = 5 * ONE_UPDATE
    finished = resume(tmp_path, [f"samples={budget}"])
    records = _records(tmp_path)
    updates = [r["update"] for r in records if r["type"] == "update"]
    assert updates == list(range(1, len(updates) + 1)) and len(updates) > 3
    # The run's episodes count on across the segments.
    episodes = [r["episodes"] for r in records if r["type"] == "update"]
    assert episodes == sorted(episodes) and episodes[-1] == finished.episodes >= stopped.episodes
    segment = next(r for r in records if r["type"] == "segment")
    assert (segment["segment"], segment["update"], segment["dropped"]) == (1, 3, 0)
    assert segment["seed"] == segment_seed(config.seed, 1)
    assert segment["changed"] == {"samples": [4 * ONE_UPDATE, budget]}
    assert records[-1]["type"] == "end"
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == [] and record["progress"]["samples"] >= budget
    assert [(s["segment"], s["ended"]) for s in record["segments"]] == [(0, "stop"), (1, "budget")]
    assert record["config"]["samples"] == budget
    assert record["progress"]["episodes"] == finished.episodes
    # A finished run resumes only with a larger budget.
    with pytest.raises(ResumeError, match="extend it"):
        resume(tmp_path)


#: A run that dies hard (SIGKILL; TerminateProcess on Windows) while it writes its fourth
#: checkpoint, the one after its third update (`checkpoint_minutes` 0: one at every update).
KILLED = """
import json, os, signal, sys
from pathlib import Path
from h3env.train import ppo
from h3env.train.run import Config

saves = 0
write = ppo.atomic_write

def dying(path, data):
    global saves
    if path.name == "checkpoint.pt":
        saves += 1
        if saves == 4:
            path.with_name(path.name + ".tmp").write_bytes(b"half")
            os.kill(os.getpid(), getattr(signal, "SIGKILL", signal.SIGTERM))
    write(path, data)

ppo.atomic_write = dying
ppo.train(Config(**json.loads(sys.argv[2])), Path(sys.argv[1]))
"""


def test_a_run_killed_hard_resumes_from_its_last_checkpoint(tmp_path) -> None:
    settings = dict(SMALL, samples=6 * ONE_UPDATE, eval_every=0, eval_start=False)
    child = subprocess.run(
        [
            sys.executable,
            "-c",
            KILLED,
            str(tmp_path),
            json.dumps({**settings, "checkpoint_minutes": 0}),
        ],
        cwd=PYTHON,
        capture_output=True,
        timeout=600,
    )
    assert child.returncode != 0, child.stderr.decode()
    assert (tmp_path / "checkpoint.pt.tmp").exists()
    assert [r["update"] for r in _records(tmp_path) if r["type"] == "update"] == [1, 2, 3]
    assert torch.load(tmp_path / "checkpoint.pt", weights_only=True)["updates"] == 2
    resume(tmp_path, ["checkpoint.checkpoint_minutes=5"])
    assert not list(tmp_path.glob("*.tmp"))
    records = _records(tmp_path)
    updates = [r["update"] for r in records if r["type"] == "update"]
    assert updates == list(range(1, len(updates) + 1)) and len(updates) >= 6
    segment = next(r for r in records if r["type"] == "segment")
    assert (segment["update"], segment["dropped"]) == (2, 1)
    record = json.loads((tmp_path / "record.json").read_text())
    assert [(s["segment"], s["ended"], s["dropped"]) for s in record["segments"]] == [
        (0, "lost", 1),
        (1, "budget", 0),
    ]


def test_a_training_episode_on_the_held_out_set_stops_the_run() -> None:
    # The config refuses the evaluation's seed for training; past that check, the batch's first
    # episodes are the held-out ones and the trainer stops at its reset.
    with pytest.raises(HeldOutError):
        Trainer(Config(**SMALL, seed=5, eval_seed=5))


def test_a_frozen_network_plays_its_side_and_only_the_learner_trains() -> None:
    trainer = Trainer(Config(**{**SMALL, "max_steps": 10}, pool_share=1.0))
    trainer.snapshot()
    with torch.no_grad():
        # The learner now prefers Defend everywhere it is legal, the frozen copy does not.
        trainer.policy.entry_head.bias[1] = 50.0
    trainer.assign(np.arange(trainer.config.envs))
    assert (trainer.frozen_side >= 0).all() and all(
        n is trainer.pool[0] for n in trainer.frozen_net
    )
    batch = trainer.batch
    choice, side = trainer.act()
    frozen = (batch.to_move >= 0) & (batch.to_move == trainer.frozen_side)
    assert frozen.any() and (~frozen & (batch.to_move >= 0)).any()
    np.testing.assert_array_equal(side, np.where(frozen, -1, batch.to_move))
    defend = batch.entry_mask[:, 1] == 1
    assert (choice.entry[~frozen & defend & (side >= 0)] == 1).all()
    assert not choice.cast[frozen].any()
    # A rollout keeps only the learner's decisions (about half of the moves here); its update
    # runs on them.
    roll = trainer.collect()
    assert (trainer.rollout.side >= 0).sum() == roll["decisions"]
    assert 0 < roll["decisions"] < 0.8 * trainer.config.rollout * trainer.config.envs
    assert roll["pool_episodes"] > 0
    stats = trainer.update(1e-4)
    assert stats["update_samples"] > 0 and np.isfinite(stats["policy_loss"])


def test_the_final_evaluation_reuses_the_last_when_nothing_changed(tmp_path) -> None:
    # An evaluation after the update: the final one is the last one's record again.
    train(Config(**SMALL, samples=1, eval_every=1), tmp_path)
    records = [json.loads(line) for line in (tmp_path / "metrics.jsonl").read_text().splitlines()]
    evals = [r for r in records if r["type"] == "eval"]
    assert [(e["update"], e["final"], e.get("repeated", False)) for e in evals] == [
        (0, False, False),
        (1, False, False),
        (1, True, True),
    ]


def test_a_checkpoint_plays_the_ladder(tmp_path, capsys) -> None:
    # E05 T7: a saved network against Random and both bots, as `h3 ai eval`'s table's row.
    train(Config(**SMALL, samples=1, eval_every=0, eval_start=False), tmp_path)
    policy = load(tmp_path / "checkpoint.pt")
    rows = ladder(policy, episodes=4, envs=2, max_steps=300)
    assert list(rows) == ["random", "aggressor", "greedy"]
    for row in rows.values():
        assert row["attacker"]["episodes"] == row["defender"]["episodes"] == 4
        assert row["attacker"]["skipped"] == row["defender"]["skipped"] == 0
        assert 0.0 <= row["win_rate"] <= 1.0
    from h3env.train.ladder import main

    main(["--checkpoint", str(tmp_path / "checkpoint.pt"), "--episodes", "2", "--envs", "2"])
    lines = capsys.readouterr().out.splitlines()
    assert [line.split(":")[0].split()[-1] for line in lines] == ["random", "aggressor", "greedy"]
    with pytest.raises(ValueError, match="unknown opponent"):
        play(random_agent(np.random.default_rng(0)), episodes=2, envs=2, opponent="minimax")
    # E05 T9b: a network trained on the ladder's own episodes (its run's seed EVAL_SEED) is refused.
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    saved["config"]["seed"] = EVAL_SEED
    torch.save(saved, tmp_path / "same.pt")
    with pytest.raises(SystemExit):
        main(["--checkpoint", str(tmp_path / "same.pt")])
