"""The training (E05 T8b, `docs/epics/E05-log.md` §8; E06 T2a, T2b1): the network's masked
choices, its log-probabilities, GAE over two sides, an update, the held-out evaluation, a run's
files and record, the checkpoint and the resume. They need the group `train` (`uv sync --group
train`) and are skipped without it; `test_run.py` covers the run's lifecycle without torch."""

from __future__ import annotations

import json
import signal
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path

import numpy as np
import pytest
from conftest import scenario

from h3env import BattleBatch, _ffi

torch = pytest.importorskip("torch")

from h3env.train import EVAL_SEED, Config, Policy, Rollout, gae, resume, train  # noqa: E402
from h3env.train.entity import Entity  # noqa: E402
from h3env.train.evaluate import (  # noqa: E402
    GATE_V1_SEED,
    NEUTRAL_KINDS,
    NEUTRAL_V1_SEED,
    REPORT_V1_SEED,
    Stopped,
    episode_seeds,
    ladder,
    load,
    play,
    random_agent,
)
from h3env.train.policy import Mlp, build  # noqa: E402
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
    # Gates only at the end, of 4 pairs at most; ladders of one episode per kind against Random.
    gate_every=0,
    max_pairs=4,
    ladder_opponents=["random"],
    ladder_episodes=1,
    ladder_final_episodes=1,
    device="cpu",
)
#: The most samples one update of `SMALL` holds (its rollout's rows).
ONE_UPDATE = 16 * 8
#: A small entity network (E06 T5a) for `SMALL`'s runs.
ENTITY = dict(kind="entity", width=16, layers=1, heads=2)


def casting_policy(batch: BattleBatch, kind: str = "mlp") -> Policy:
    """A small network of `kind` that prefers the casts where they are legal, so that their
    heads are chosen too."""
    torch.manual_seed(0)
    if kind == "mlp":
        policy = Mlp.for_batch(batch, hidden=32, embed=8)
        with torch.no_grad():
            policy.entry_head.bias[[_ffi.HERO_CAST, _ffi.CREATURE_CAST]] = 30.0
        return policy
    policy = Entity.for_batch(batch, width=16, layers=1, heads=2, embed=8)
    with torch.no_grad():
        # The fixed entries: Wait, Defend, End tactics, Retreat, Surrender, the two casts, ….
        policy.fixed_head.bias[[5, 6]] = 30.0
    return policy


def mixed_batch() -> BattleBatch:
    batch = BattleBatch(8, threads=2, seed=3, max_steps=300)
    batch.add_scenario(scenario("full"))
    batch.add_scenario(scenario("siege"))
    batch.add_encounters()
    batch.reset()
    return batch


@pytest.mark.parametrize("kind", ["mlp", "entity"])
def test_choices_keep_to_the_masks_and_recompute_their_log_probabilities(kind) -> None:
    batch = mixed_batch()
    policy = casting_policy(batch, kind)
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


@pytest.mark.parametrize("penalty", [0.0, 1.0])
def test_a_truncation_costs_its_penalty_or_bootstraps(penalty) -> None:
    # E06 T5c: battles cut after 6 steps. With a penalty each learning side's last transition
    # gets -penalty and ends there (reward v1 gives 0 at a truncation); without one it
    # bootstraps from the final observation's value, as before.
    trainer = Trainer(Config(**{**SMALL, "max_steps": 6}, truncation_penalty=penalty))
    stats = trainer.collect()
    r = trainer.rollout
    assert stats["truncated"] > 0.5
    cut = r.closed & r.terminal & (r.reward == -penalty) if penalty else r.closed & r.truncated
    assert cut.sum() > 0 and r.terminal[cut].all()
    if penalty:
        assert not r.truncated.any() and (r.final_value[cut] == 0.0).all()
    else:
        assert (r.final_value[cut] != 0.0).any()
    trainer.batch.close()


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
    config = Config(**SMALL, samples=ONE_UPDATE + 1, snapshot_every=1)
    trainer = train(config, tmp_path, text="samples = 129\n")
    records = [json.loads(line) for line in (tmp_path / "metrics.jsonl").read_text().splitlines()]
    assert records[0]["type"] == "start" and records[-1]["type"] == "end"
    assert records[-1]["reason"] == "budget"
    updates = [r for r in records if r["type"] == "update"]
    n = len(updates)
    # The budget in samples: the run stops at the first update boundary at or past it.
    assert n >= 2 and updates[-2]["samples"] < config.samples <= updates[-1]["samples"]
    # Best #0, the untrained network, and its ladder first; at the end the learner's gate (and,
    # promoted, its ladder), then the best's final ladder.
    kinds = [r["type"] for r in records if r["type"] not in ("update", "snapshot")]
    promoted = [r for r in records if r["type"] == "gate"][0]["result"] == "promoted"
    tail = ["gate", "promotion", "ladder", "ladder"] if promoted else ["gate", "ladder"]
    assert kinds == ["start", "promotion", "ladder", *tail, "end"]
    ladders = [r for r in records if r["type"] == "ladder"]
    assert (ladders[0]["index"], ladders[0]["final"], ladders[-1]["final"]) == (0, False, True)
    assert ladders[0]["opponents"]["random"]["games"] == 6
    # E06 T7c: and the neutral set's, attacking the system actor (one episode per neutral kind).
    assert ladders[0]["neutral"]["games"] == 3
    assert list(ladders[0]["neutral"]["sides"]) == ["wandering", "guards", "siege_neutral"]
    assert (tmp_path / "promoted" / "0.pt").exists()
    for r in updates:
        assert all(np.isfinite(r[k]) for k in ("policy_loss", "value_loss", "entropy", "kl"))
        assert r["update_samples"] > 0
    assert updates[1]["samples"] == updates[0]["samples"] + updates[1]["update_samples"]
    # The learning rate decays over the samples (the horizon: the budget).
    assert updates[0]["lr"] == config.lr > updates[1]["lr"]
    # The league: best #0 from the start, a snapshot after each update (every sample is a
    # multiple of 1), so each rollout drew from #0 and the snapshots before it; a promotion at
    # the end joins too. Each snapshot's network is in its file.
    assert [r["league"] for r in updates] == list(range(1, n + 1))
    snapshots = [r for r in records if r["type"] == "snapshot"]
    assert [(r["member"], r["retired"]) for r in snapshots] == [(i, None) for i in range(1, n + 1)]
    members = trainer.league.members
    assert [m.kind for m in members] == ["promoted", *["snapshot"] * n, *["promoted"] * promoted]
    assert [m.file for m in members][:2] == ["promoted/0.pt", "league/1.pt"]
    assert len(trainer.nets) == len(members) and all(r["frozen_passes"] >= 0 for r in updates)
    _same(load(tmp_path / "league" / "1.pt").state_dict(), trainer.nets[1].state_dict())
    # The end's gate's member table: before its promotion joined.
    assert record_gate_league(tmp_path) == list(range(n + 1))
    best = torch.load(tmp_path / "best.pt", weights_only=True)
    assert best["index"] == (1 if promoted else 0) and best["run"] == tmp_path.name
    assert (tmp_path / "config.toml").read_text() == "samples = 129\n"
    assert not list(tmp_path.glob("*.tmp"))
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == []
    assert record["identity"]["abi"] == _ffi.ABI_VERSION and record["identity"]["device"] == "cpu"
    assert record["config"]["network"]["hidden"] == 32
    assert record["config"]["ppo"]["lr_horizon"] == ONE_UPDATE + 1
    assert record["progress"]["ended"] == "budget" and record["progress"]["updates"] == n
    assert len(record["gates"]) == 1 and len(record["curve"]) == 1
    assert record["progress"]["evaluation_minutes"] > 0
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    restored = build(asdict(trainer.config), trainer.batch)
    restored.load_state_dict(saved["policy"])
    restored_state = restored.state_dict()
    for name, weights in trainer.policy.state_dict().items():
        assert torch.equal(weights, restored_state[name]), name
    assert saved["updates"] == n and saved["finished"]
    assert saved["league"]["members"] == [asdict(m) for m in members]
    assert record["segments"][0]["seed"] == config.seed == trainer.batch_seed


def _records(out: Path) -> list[dict]:
    return [json.loads(x) for x in (out / "metrics.jsonl").read_text().splitlines()]


def record_gate_league(out: Path) -> list[int]:
    """The member ids of the record's last gate's table."""
    record = json.loads((out / "record.json").read_text())
    return [m["id"] for m in record["gates"][-1]["league"]]


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
    config = Config(**SMALL, samples=4 * ONE_UPDATE, snapshot_every=1)

    def interrupt(record: dict) -> None:
        if record["type"] == "update" and record["update"] == 3:
            signal.raise_signal(signal.SIGINT)

    stopped = train(config, tmp_path, log=interrupt)
    assert _records(tmp_path)[-1]["type"] == "stop" and stopped.updates == 3
    state = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    assert (state["format"], state["segment"], state["updates"], state["finished"]) == (
        3,
        0,
        3,
        False,
    )
    assert state["metrics_length"] == (tmp_path / "metrics.jsonl").stat().st_size
    _same(stopped.policy.state_dict(), state["policy"])
    again = Trainer(Config(**state["config"]), segment=1)
    again.restore(state, tmp_path)
    _same(again.policy.state_dict(), state["policy"])
    _same(again.optimizer.state_dict()["state"], state["optimizer"]["state"])
    # The league: best #0 and three snapshots, their networks from their files.
    assert again.league.state() == state["league"] and len(again.nets) == 4
    for member in stopped.league.members:
        _same(again.nets[member.id].state_dict(), stopped.nets[member.id].state_dict())
    assert torch.equal(again.generator.get_state(), state["rng"]["sampling"])
    assert torch.equal(torch.get_rng_state(), state["rng"]["torch"])
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


def test_the_league_retires_its_most_beaten_and_a_resume_keeps_its_files(tmp_path) -> None:
    # Two members at most, a snapshot after every update: each retires one (retirement needs no
    # games here). A retired snapshot's file goes after the next checkpoint; the resume removes
    # files its checkpoint does not name and plays on with the members it names.
    settings = dict(snapshot_every=1, max_members=2, retire_games=0, checkpoint_minutes=0)
    config = Config(**SMALL, samples=4 * ONE_UPDATE, **settings)

    def interrupt(record: dict) -> None:
        if record["type"] == "update" and record["update"] == 3:
            signal.raise_signal(signal.SIGINT)

    stopped = train(config, tmp_path, log=interrupt)
    records = _records(tmp_path)
    snapshots = [(r["member"], r["retired"]) for r in records if r["type"] == "snapshot"]
    assert len(snapshots) == 3 and all(retired is not None for _, retired in snapshots[1:])
    assert all(r["league"] <= 2 for r in records if r["type"] == "update")
    members = stopped.league.members
    assert len(members) == 2 and set(stopped.nets) == {m.id for m in members}
    named = {m.file for m in members}
    assert {f"league/{f.name}" for f in (tmp_path / "league").glob("*.pt")} <= named
    # A snapshot taken after the checkpoint (here: a stray file) is removed at the resume.
    (tmp_path / "league" / "99.pt").write_bytes(b"stray")
    finished = resume(tmp_path, [f"samples={5 * ONE_UPDATE}"])
    assert not (tmp_path / "league" / "99.pt").exists()
    assert {f"league/{f.name}" for f in (tmp_path / "league").glob("*.pt")} <= {
        m.file for m in finished.league.members
    }
    assert finished.league.next_id > stopped.league.next_id
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == [] and len(record["gates"][-1]["league"]) == 2


#: A run that dies hard (SIGKILL; TerminateProcess on Windows) while it writes its fifth
#: checkpoint, the one after its third update (`checkpoint_minutes` 0: one at every update; two
#: around best #0's ladder before the first).
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
        if saves == 5:
            path.with_name(path.name + ".tmp").write_bytes(b"half")
            os.kill(os.getpid(), getattr(signal, "SIGKILL", signal.SIGTERM))
    write(path, data)

ppo.atomic_write = dying
ppo.train(Config(**json.loads(sys.argv[2])), Path(sys.argv[1]))
"""


def test_a_run_killed_hard_resumes_from_its_last_checkpoint(tmp_path) -> None:
    settings = dict(SMALL, samples=6 * ONE_UPDATE)
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
    # The config refuses a held-out set's batch seed for training; past that check, the batch's
    # first episodes are the gate set's and the trainer stops at its reset.
    with pytest.raises(HeldOutError):
        Trainer(Config(**SMALL, seed=GATE_V1_SEED))


def test_a_frozen_network_plays_its_side_and_only_the_learner_trains() -> None:
    trainer = Trainer(Config(**{**SMALL, "max_steps": 10}, self_play=0.0))
    member, _ = trainer.join()
    assert trainer.league.draw(0) and trainer.league.live == [member.id]
    with torch.no_grad():
        # The learner now prefers Defend everywhere it is legal, the frozen copy does not.
        trainer.policy.entry_head.bias[1] = 50.0
    trainer.assign(np.arange(trainer.config.envs))
    assert (trainer.frozen_side >= 0).all() and (trainer.frozen_id == member.id).all()
    assert all(n is trainer.nets[member.id] for n in trainer.frozen_net)
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
    # Every episode against the member; its score counts the learner's results.
    assert roll["league_episodes"] > 0 and roll["self_play_episodes"] == 0
    assert 0.0 < roll["frozen_passes"] <= 1.0 and roll["live"] == [member.id]
    assert trainer.league.member(member.id).played == roll["league_episodes"]
    stats = trainer.update(1e-4)
    assert stats["update_samples"] > 0 and np.isfinite(stats["policy_loss"])


def test_a_neutral_episode_draws_no_member_and_the_rates_split_by_kind() -> None:
    # E06 T7c (design §12): wandering monsters beside hero battles, no self-play. A neutral
    # episode's defender is the system actor inside the batch, so it gets no league member and
    # the learner plays its attacker alone; a hero battle gets the member.
    trainer = Trainer(Config(**{**SMALL, "max_steps": 10}, self_play=0.0, kinds=[1, 0, 1, 0, 0, 0]))
    member, _ = trainer.join()
    assert trainer.league.draw(0)
    trainer.assign(np.arange(trainer.config.envs))
    batch = trainer.batch
    neutral = np.isin(batch.encounter, list(NEUTRAL_KINDS))
    assert neutral.any() and (~neutral).any()
    assert (trainer.frozen_side[neutral] == -1).all()
    assert (trainer.frozen_id[~neutral] == member.id).all()
    roll = trainer.collect()
    # Only the neutral episodes are the learner's own: their rates by kind, the attacker's.
    assert roll["self_play_episodes"] > 0 and roll["league_episodes"] > 0
    assert list(roll["by_kind"]) == ["wandering"]
    wandering = roll["by_kind"]["wandering"]
    assert wandering["episodes"] == roll["self_play_episodes"]
    assert wandering["attacker_won"] == roll["attacker_won"]
    # The episodes the rollout's ends started drew alike.
    for e in np.nonzero(trainer.frozen_side >= 0)[0]:
        assert int(batch.encounter[e]) not in NEUTRAL_KINDS


def test_the_ladder_scores_at_reward_v1s_settings_whatever_the_run_trains_with() -> None:
    # The mana-price experiment (decision 10): `[env] currency` and `mana_price` shape the
    # training's reward only; the ladders score at reward v1's, as the bots' tables do.
    trainer = Trainer(Config(**{**SMALL, "max_steps": 2000}, currency="gold", mana_price=1000))
    trainer.promote()
    row = trainer.ladder(final=False)["opponents"]["random"]
    best = trainer.lineage[-1][1]
    settings = dict(opponents=["random"], per_kind=1, threads=2, max_steps=2000)
    plain = ladder(best, **settings)["random"]
    trained = ladder(best, **settings, currency="gold", mana_price=1000)["random"]
    assert row["value"] == plain["value"] != trained["value"]


def _gate(result: str):
    """A stand-in for `gate.gate` that decides at once (`result`)."""

    def decided(candidate, best, bounds, **settings) -> dict:
        assert candidate.training and not best.training
        return {
            "result": result,
            "pairs": 10,
            "counts": [0, 0, 0, 0, 10],
            "llr": 3.0,
            "bounds": [-2.94, 2.94],
            "score": 1.0,
            "elo": None,
            "games": 20,
            "seconds": 0.0,
        }

    return decided


def test_the_end_gates_only_after_training_since_the_last_gate(tmp_path, monkeypatch) -> None:
    # A gate after every update: none more at the end, only the best's final ladder.
    monkeypatch.setattr("h3env.train.ppo.gate", _gate("rejected"))
    trainer = train(Config(**{**SMALL, "gate_every": 1}, samples=2 * ONE_UPDATE), tmp_path)
    records = _records(tmp_path)
    gates = [r["update"] for r in records if r["type"] == "gate"]
    assert gates == list(range(1, trainer.updates + 1))
    ladders = [(r["index"], r["final"]) for r in records if r["type"] == "ladder"]
    assert ladders == [(0, False), (0, True)] and records[-1]["type"] == "end"


def test_promotions_keep_their_networks_and_ladder_the_one_before(tmp_path, monkeypatch) -> None:
    # Every gate promotes: each promoted network is kept and becomes best.pt; its ladder plays
    # the engine's seats and the promoted network before it (both seats external).
    monkeypatch.setattr("h3env.train.ppo.gate", _gate("promoted"))
    trainer = train(Config(**{**SMALL, "gate_every": 1}, samples=2 * ONE_UPDATE), tmp_path)
    n = trainer.updates
    records = _records(tmp_path)
    promotions = [(r["index"], r["how"], r["update"]) for r in records if r["type"] == "promotion"]
    assert promotions == [(0, "start", 0), *((i, "gate", i) for i in range(1, n + 1))]
    for i in range(n + 1):
        assert torch.load(tmp_path / "promoted" / f"{i}.pt", weights_only=True)["index"] == i
    best = torch.load(tmp_path / "best.pt", weights_only=True)
    assert best["index"] == n and best["updates"] == n
    _same(best["policy"], trainer.policy.state_dict())
    ladders = [r for r in records if r["type"] == "ladder"]
    assert [(r["index"], r["final"]) for r in ladders] == [
        *((i, False) for i in range(n + 1)),
        (n, True),
    ]
    assert list(ladders[0]["opponents"]) == ["random"]
    for r in ladders[1:]:
        assert list(r["opponents"]) == ["random", f"promoted-{r['index'] - 1}"]
        assert r["opponents"][f"promoted-{r['index'] - 1}"]["games"] == 6
    assert [i for i, _ in trainer.lineage] == [n - 1, n]
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == [] and record["best"] == {"index": n, "samples": trainer.samples}


def test_a_gate_cut_short_runs_first_at_the_resume(tmp_path, monkeypatch) -> None:
    # Ctrl+C during the first gate: the run stops with the gate pending, which the resume plays
    # first, on the learner of the checkpoint; then its promotion's ladder.
    def cut(candidate, best, bounds, stop=None, **settings) -> dict:
        stop.request()
        raise Stopped

    monkeypatch.setattr("h3env.train.ppo.gate", cut)
    stopped = train(Config(**{**SMALL, "gate_every": 1}, samples=3 * ONE_UPDATE), tmp_path)
    records = _records(tmp_path)
    assert records[-1]["type"] == "stop" and not any(r["type"] == "gate" for r in records)
    state = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    assert state["pending"] == {"kind": "gate"} and state["updates"] == stopped.updates == 1
    monkeypatch.setattr("h3env.train.ppo.gate", _gate("promoted"))
    resume(tmp_path)
    records = _records(tmp_path)
    at = next(i for i, r in enumerate(records) if r["type"] == "segment")
    assert records[at]["pending"] == {"kind": "gate"}
    kinds = [(r["type"], r["update"]) for r in records[at + 1 : at + 4]]
    assert kinds == [("gate", 1), ("promotion", 1), ("ladder", 1)]
    _same(torch.load(tmp_path / "promoted" / "1.pt", weights_only=True)["policy"], state["policy"])
    # A ladder cut short resumes too: its promoted network's from its file.
    assert check_record(json.loads((tmp_path / "record.json").read_text())) == []


def test_a_ladder_cut_short_runs_first_at_the_resume(tmp_path, monkeypatch) -> None:
    import h3env.train.ppo as ppo

    real = ppo.ladder

    def cut(*args, stop=None, **kwargs):
        stop.request()
        raise Stopped

    monkeypatch.setattr("h3env.train.ppo.ladder", cut)
    train(Config(**SMALL, samples=ONE_UPDATE), tmp_path)
    state = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    assert state["pending"] == {"kind": "ladder", "final": False} and state["updates"] == 0
    monkeypatch.setattr("h3env.train.ppo.ladder", real)
    trainer = resume(tmp_path)
    records = _records(tmp_path)
    at = next(i for i, r in enumerate(records) if r["type"] == "segment")
    assert (records[at + 1]["type"], records[at + 1]["index"]) == ("ladder", 0)
    assert [i for i, _ in trainer.lineage][0] == 0 and records[-1]["type"] == "end"


def test_a_run_starts_from_its_parents_network(tmp_path) -> None:
    # `--init-from`: the learner and best #0 start from another run's best; the record names it.
    train(Config(**SMALL, samples=1), tmp_path / "a")
    parent = torch.load(tmp_path / "a" / "best.pt", weights_only=True)
    train(Config(**SMALL, samples=1, seed=2), tmp_path / "b", init_from=tmp_path / "a" / "best.pt")
    zero = torch.load(tmp_path / "b" / "promoted" / "0.pt", weights_only=True)
    _same(zero["policy"], parent["policy"])
    assert zero["how"] == "init" and zero["parent"]["run"] == "a"
    assert zero["parent"]["index"] == parent["index"]
    record = json.loads((tmp_path / "b" / "record.json").read_text())
    assert record["parent"]["path"] == str(tmp_path / "a" / "best.pt")
    assert record["promotions"][0]["how"] == "init"
    with pytest.raises(ConfigError, match="not a network of this run's size"):
        train(Config(**{**SMALL, "hidden": 16}, samples=1), init_from=tmp_path / "a" / "best.pt")
    # E06 T5a: a network of another kind is refused before its weights are read.
    with pytest.raises(ConfigError, match="a network of kind 'mlp', this run's is 'entity'"):
        train(Config(**SMALL, **ENTITY, samples=1), init_from=tmp_path / "a" / "best.pt")


def test_an_entity_run_stops_resumes_and_plays_its_ladder(tmp_path) -> None:
    # E06 T5a: the entity network through a run: stopped after its first update, resumed to the
    # budget; its files are entity networks (`load` builds the kind their config names).
    config = Config(**SMALL, **ENTITY, samples=2 * ONE_UPDATE, snapshot_every=1)

    def interrupt(record: dict) -> None:
        if record["type"] == "update" and record["update"] == 1:
            signal.raise_signal(signal.SIGINT)

    stopped = train(config, tmp_path, log=interrupt)
    assert isinstance(stopped.policy, Entity) and stopped.updates == 1
    finished = resume(tmp_path)
    assert finished.updates >= 2 and isinstance(finished.policy, Entity)
    _same(finished.nets[1].state_dict(), load(tmp_path / "league" / "1.pt").state_dict())
    record = json.loads((tmp_path / "record.json").read_text())
    assert check_record(record) == [] and record["config"]["network"]["kind"] == "entity"
    assert [s["ended"] for s in record["segments"]] == ["stop", "budget"]
    best = load(tmp_path / "best.pt")
    assert isinstance(best, Entity) and best.width == 16 and len(best.blocks) == 1
    rows = ladder(best, opponents=["random"], per_kind=1, max_steps=300)
    assert rows["random"]["games"] == 6


def test_a_checkpoint_plays_the_ladder(tmp_path, capsys) -> None:
    # E05 T7, E06 T3a: a saved network against Random and both bots, as `h3 ai eval`'s table's
    # row: on the report set v1 (by default) and on v0.
    train(Config(**SMALL, samples=1), tmp_path)
    policy = load(tmp_path / "checkpoint.pt")
    rows = ladder(policy, held_out="v0", episodes=4, envs=2, max_steps=300)
    assert list(rows) == ["random", "aggressor", "greedy", "system"]
    for row in rows.values():
        assert row["attacker"]["episodes"] == row["defender"]["episodes"] == 4
        assert row["attacker"]["skipped"] == row["defender"]["skipped"] == 0
        assert 0.0 <= row["win_rate"] <= 1.0
    rows = ladder(policy, opponents=["greedy"], per_kind=2, max_steps=300)
    row = rows["greedy"]
    assert row["games"] == 12 and list(row["kinds"]) == ["hero", "siege_hero", "siege_garrison"]
    assert 0.0 <= row["win_rate"] <= 1.0 and row["interval"] >= 0.0
    # The sampled policy draws alike on the same episodes (common random numbers).
    again = ladder(policy, opponents=["greedy"], per_kind=2, max_steps=300)["greedy"]
    assert again["by_kind"] == row["by_kind"]
    with pytest.raises(ValueError, match="held-out set"):
        ladder(policy, held_out="v7")
    # E06 T7c: the neutral set, attacking its one opponent, the system actor; no networks there.
    rows = ladder(policy, opponents=["greedy"], held_out="neutral-v1", per_kind=2, max_steps=300)
    assert list(rows) == ["system"] and rows["system"]["games"] == 6
    assert list(rows["system"]["kinds"]) == ["wandering", "guards", "siege_neutral"]
    with pytest.raises(ValueError, match="no network opponents"):
        ladder(policy, held_out="neutral-v1", networks={"me": policy})
    from h3env.train.ladder import main

    checkpoint = ["--checkpoint", str(tmp_path / "checkpoint.pt")]
    out = tmp_path / "ladder.json"
    main([*checkpoint, "--episodes", "1", "--max-steps", "300", "--json", str(out)])
    lines = capsys.readouterr().out.splitlines()
    opponents = ["random", "aggressor", "greedy", "system"]
    assert [line.split(":")[0].split()[-1] for line in lines] == opponents
    assert all("±" in line and "siege_garrison" in line for line in lines)
    assert set(json.loads(out.read_text())) == set(opponents)
    main([*checkpoint, "--set", "v0", "--episodes", "2", "--envs", "2"])
    assert len(capsys.readouterr().out.splitlines()) == 4
    main([*checkpoint, "--set", "neutral-v1", "--episodes", "1", "--max-steps", "300"])
    (line,) = capsys.readouterr().out.splitlines()
    assert line.startswith("against system") and "wandering" in line
    with pytest.raises(SystemExit):
        main([*checkpoint, "--envs", "2"])
    with pytest.raises(ValueError, match="unknown opponent"):
        play(random_agent(np.random.default_rng(0)), episodes=2, envs=2, opponent="minimax")
    # E05 T9b, E06 T3a: a network trained on a held-out set's episodes (its run's seed one of
    # theirs) is refused.
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    for seed in (EVAL_SEED, REPORT_V1_SEED + 4, NEUTRAL_V1_SEED + 5):
        saved["config"]["seed"] = seed
        torch.save(saved, tmp_path / "same.pt")
        with pytest.raises(SystemExit):
            main(["--checkpoint", str(tmp_path / "same.pt")])


def test_a_checkpoint_plays_the_behaviour_analysis(tmp_path, capsys) -> None:
    # E06 T6a: the network's battles in the analysis are its ladder's (the same draws), and the
    # command writes two tables per opponent and the JSON.
    from h3env.train.evaluate import sampled
    from h3env.train.inspect import inspect, main

    train(Config(**SMALL, samples=1), tmp_path)
    policy = load(tmp_path / "checkpoint.pt")
    policy.eval()
    result = inspect(sampled(policy), opponents=["greedy"], per_kind=2, max_steps=300)
    row = ladder(policy, opponents=["greedy"], per_kind=2, max_steps=300)["greedy"]
    assert result["greedy"]["ladder"]["sides"] == {
        kind: [side["win_rate"] for side in sides.values()]
        for kind, sides in row["by_kind"].items()
    }
    assert result["greedy"]["groups"]["all"]["battles"] == 12
    out = tmp_path / "inspect.json"
    checkpoint = ["--checkpoint", str(tmp_path / "checkpoint.pt")]
    main([*checkpoint, "--episodes", "1", "--max-steps", "300", "--json", str(out)])
    text = capsys.readouterr().out.splitlines()
    assert [line.split(":")[0] for line in text if line.startswith("against")] == [
        "against system",
        "against greedy",
    ]
    assert set(json.loads(out.read_text())) == {"system", "greedy"}
    main([*checkpoint, "--set", "neutral-v1", "--episodes", "1", "--max-steps", "300"])
    assert capsys.readouterr().out.startswith("against system")
    saved = torch.load(tmp_path / "checkpoint.pt", weights_only=True)
    saved["config"]["seed"] = REPORT_V1_SEED + 2
    torch.save(saved, tmp_path / "same.pt")
    with pytest.raises(SystemExit):
        main(["--checkpoint", str(tmp_path / "same.pt")])
