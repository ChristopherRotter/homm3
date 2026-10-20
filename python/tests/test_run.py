"""A run's lifecycle without torch (E06 T2a, T2b1; ADR-0026, `docs/epics/E06-design.md` §1 to
§5): the config file and `--set`, the schedule in samples, atomic writes, the held-out check,
the run's directory and record, what a resume needs (its allowed changes, the segments' seeds,
the metrics cut back, Ctrl+C), and every committed record and config."""

from __future__ import annotations

import json
import signal
from pathlib import Path

import pytest

from h3env.train import Config
from h3env.train.__main__ import line, parse
from h3env.train.evaluate import (
    EVAL_SEED,
    GATE_V1_SEED,
    HELD_OUT_BATCH_SEEDS,
    REPORT_V1_SEED,
    episode_seeds,
)
from h3env.train.run import (
    RECORD_SCHEMA,
    RESUMABLE,
    TABLES,
    Cadence,
    ConfigError,
    HeldOut,
    HeldOutError,
    ResumeError,
    Run,
    RunExistsError,
    Stop,
    atomic_write,
    atomic_write_text,
    by_table,
    check_record,
    learning_rate,
    load_config,
    resumed,
    segment_seed,
)

PYTHON = Path(__file__).resolve().parents[1]

#: A record's identity as `ppo.identity` gives it (C ABI v31), without the machine.
IDENTITY = {
    "commit": "abc",
    "dirty": False,
    "abi": 31,
    "observation_encoding": 21,
    "action_encoding": 13,
    "encounters": 1,
    "entries": [{"kind": "encounters", "weight": 1, "ruleset_hash": "00000000000000aa"}],
    "torch": "x",
    "device": "cpu",
}


def test_a_config_file_states_only_what_differs() -> None:
    text = """
seed = 3
samples = 2_000_000

[env]
envs = 64
kinds = [0, 0, 1, 0, 0, 0]

[ppo]
lr = 1e-4
lr_schedule = "constant"
"""
    sets = [
        "ppo.gamma=0.99",
        "device.device=cpu",
        "gate.elo1=10",
        'ladder.ladder_opponents=["greedy"]',
    ]
    c = load_config(text, sets)
    assert (c.seed, c.samples, c.envs, c.kinds) == (3, 2_000_000, 64, [0, 0, 1, 0, 0, 0])
    assert (c.lr, c.lr_schedule, c.gamma, c.device, c.elo1, c.ladder_opponents) == (
        1e-4,
        "constant",
        0.99,
        "cpu",
        10.0,
        ["greedy"],
    )
    # Everything else keeps its default; the horizon resolves to the budget at the start.
    assert c.hidden == Config().hidden and c.lr_horizon == 2_000_000
    assert load_config("").lr_horizon == Config().samples
    # An integer for a float, a top-level key without a table.
    assert load_config("", ["ppo.lr=1", "seed=9"]).lr == 1.0
    assert load_config("", ["seed=9"]).seed == 9


@pytest.mark.parametrize(
    ("text", "sets", "message"),
    [
        ("[ppo]\nlearning_rate = 1e-4\n", [], "unknown key `learning_rate` in \\[ppo\\]"),
        ("[env]\nlr = 1e-4\n", [], "it belongs to \\[ppo\\]"),
        ("[league]\nsize = 4\n", [], "unknown table \\[league\\]"),
        ("envs = 8\n", [], "it belongs to \\[env\\]"),
        ("", ["env.envs=many"], "`envs` takes an integer"),
        ("", ["env.envs=8.0"], "`envs` takes an integer"),
        ("", ["gate.max_pairs=1.5"], "`max_pairs` takes an integer"),
        ("[evaluation]\neval_every = 1\n", [], "unknown table \\[evaluation\\]"),
        ("", ['ladder.ladder_opponents=["minimax"]'], "distinct names"),
        ("", ['ladder.ladder_opponents=["random", "random"]'], "distinct names"),
        ("", ["gate.alpha=0.5"], "`alpha` and `beta` lie in"),
        ("", ["gate.elo1=0"], "`elo0` below `elo1`"),
        ("", ["gate.max_pairs=4001"], "`max_pairs` lies in"),
        ("", ["ladder.ladder_episodes=0"], "`ladder_episodes` lies in"),
        ("", ["ladder.ladder_final_episodes=501"], "`ladder_episodes` lies in"),
        ("", ["env.kinds=[1, 2]"], "six weights"),
        ("", ["ppo.lr"], "table.key=value"),
        ("seed = [", [], "not TOML"),
        ("", ["ppo.lr_schedule=cosine"], "linear or constant"),
        ("", ["samples=0"], "`samples` must be positive"),
        ("", [f"seed={EVAL_SEED}"], "held-out set's batch seed"),
        ("", [f"seed={REPORT_V1_SEED + 3}"], "held-out set's batch seed"),
        ("", [f"seed={GATE_V1_SEED}"], "held-out set's batch seed"),
    ],
)
def test_a_wrong_config_is_refused(text: str, sets: list[str], message: str) -> None:
    with pytest.raises(ConfigError, match=message):
        load_config(text, sets)


def test_every_key_has_one_table_and_the_record_shows_them() -> None:
    keys = [k for keys in TABLES.values() for k in keys]
    assert len(keys) == len(set(keys)) == len(Config.__dataclass_fields__)
    shown = by_table(Config())
    assert shown["seed"] == Config().seed and shown["ppo"]["lr"] == Config().lr
    assert set(shown) == set(TABLES[""]) | {t for t in TABLES if t}
    # The held-out set v0's seed (E05's, `h3 ai eval --set v0`'s) is refused for training.
    assert EVAL_SEED in HELD_OUT_BATCH_SEEDS and Config().seed not in HELD_OUT_BATCH_SEEDS


def test_the_command_line_reads_the_file_and_the_overrides(tmp_path: Path) -> None:
    path = tmp_path / "run.toml"
    path.write_text("samples = 5_000\n[ppo]\nlr = 3e-4\n", encoding="utf-8")
    command = parse(["--config", str(path), "--set", "ppo.lr=1e-4", "--out", str(tmp_path / "r")])
    config = command.config
    assert (config.samples, config.lr, command.out) == (5_000, 1e-4, tmp_path / "r")
    assert command.text == path.read_text(encoding="utf-8") and not command.resume
    with pytest.raises(SystemExit):
        parse(["--set", "ppo.lr=fast"])
    # A resume names the run's directory; its config is the checkpoint's, changed by --set only.
    command = parse(["--resume", str(tmp_path / "r"), "--set", "samples=9000"])
    assert command.resume and command.out == tmp_path / "r" and command.config is None
    assert command.sets == ["samples=9000"]
    for wrong in (["--config", str(path)], ["--out", str(tmp_path / "s")], ["--init-from", path]):
        with pytest.raises(SystemExit):
            parse(["--resume", str(tmp_path / "r"), *map(str, wrong)])
    # A new run from a parent network (E06 T3b): the file must exist.
    command = parse(["--out", str(tmp_path / "c"), "--init-from", str(path)])
    assert command.init_from == path and not command.resume
    with pytest.raises(SystemExit):
        parse(["--out", str(tmp_path / "c"), "--init-from", str(tmp_path / "none.pt")])


def test_the_gates_promotions_and_ladders_print_one_line_each() -> None:
    gate, promotion, ladder = _gate(4, 40, "promoted"), _promotion(4, 40, 2), _ladder(4, 40, 2)
    assert line(gate) == (
        "gate update    4          40 samples against best #0: promoted after 100 pairs, score "
        "0.600 (+70 Elo), LLR +3.00 [30 s]"
    )
    assert "Elo" not in line({**gate, "elo": None})
    assert line(promotion) == "promoted #2 (gate) at update 4"
    assert line(ladder) == "ladder #2 (250 per kind): random 0.800 ± 0.020 [90 s]"
    assert line({**ladder, "final": True}).startswith("ladder final #2")


def test_the_learning_rate_follows_samples() -> None:
    c = load_config("samples = 1000\n[ppo]\nlr = 1.0\nlr_floor = 0.1\n")
    assert [learning_rate(c, s) for s in (0, 500, 900, 1000, 5000)] == [1.0, 0.5, 0.1, 0.1, 0.1]
    # A horizon of its own; a constant rate.
    c = load_config("samples = 1000\n[ppo]\nlr = 1.0\nlr_horizon = 4000\n")
    assert learning_rate(c, 1000) == 0.75
    assert learning_rate(load_config("[ppo]\nlr_schedule = 'constant'\n"), 10**9) == Config().lr


def test_a_cadence_is_due_once_per_multiple() -> None:
    c = Cadence(100)
    assert [c.due(s) for s in (50, 100, 120, 199, 350, 399, 400)] == [
        False,
        True,
        False,
        False,
        True,
        False,
        True,
    ]
    assert not Cadence(0).due(10**9)


def test_an_atomic_write_replaces_whole_or_not_at_all(tmp_path: Path) -> None:
    path = tmp_path / "state.bin"
    atomic_write_text(path, "first")
    atomic_write_text(path, "second")
    assert path.read_text() == "second" and not list(tmp_path.glob("*.tmp"))

    def fails(f) -> None:
        f.write(b"half")
        raise RuntimeError("crash")

    with pytest.raises(RuntimeError):
        atomic_write(path, fails)
    # The old file stands; the partial one is a temporary, removed at the next start.
    assert path.read_text() == "second" and (tmp_path / "state.bin.tmp").exists()
    Run(tmp_path, Config(), {}).close()
    assert not list(tmp_path.glob("*.tmp"))


def test_a_refused_replace_keeps_the_old_file_and_goes_on(tmp_path: Path, monkeypatch, capsys):
    # Windows refuses a replace while another process holds the file open (a ladder reading
    # best.pt): five tries, then the old file stays and the run goes on (design §3).
    path = tmp_path / "best.pt"
    atomic_write_text(path, "old")
    tries = []

    def refused(src, dst) -> None:
        tries.append(dst)
        raise PermissionError("in use")

    monkeypatch.setattr("h3env.train.run.os.replace", refused)
    monkeypatch.setattr("h3env.train.run.time.sleep", lambda s: None)
    assert atomic_write(path, lambda f: f.write(b"new")) is False
    assert len(tries) == 5 and path.read_text() == "old" and not list(tmp_path.glob("*.tmp"))
    assert "best.pt kept" in capsys.readouterr().err


def test_a_resume_changes_only_what_leaves_the_training_as_it_is() -> None:
    saved = load_config("samples = 1000\n[ppo]\nlr = 1e-3\n")
    config, changed = resumed(
        saved,
        ["samples=3000", "env.threads=3", "device.device=cpu", "checkpoint.checkpoint_minutes=1"],
    )
    assert (config.samples, config.threads, config.device, config.checkpoint_minutes) == (
        3000,
        3,
        "cpu",
        1.0,
    )
    assert set(changed) <= set(RESUMABLE) and changed["samples"] == [1000, 3000]
    # The learning rate's horizon stays the start's budget.
    assert config.lr_horizon == 1000 and resumed(saved) == (saved, {})
    with pytest.raises(ConfigError, match=r"env\.envs, ppo\.lr would change the training"):
        resumed(saved, ["ppo.lr=1e-4", "env.envs=8", "samples=5"])
    with pytest.raises(ConfigError, match="unknown key"):
        resumed(saved, ["ppo.speed=2"])


def test_each_segment_has_its_own_batch_seed() -> None:
    assert segment_seed(7, 0) == 7
    # SplitMix64's reference outputs for the seed 0.
    assert [segment_seed(0, s) for s in (1, 2, 3)] == [
        0xE220A8397B1DCDAF,
        0x6E789E6AA1B965F4,
        0x06C45D188009454F,
    ]
    seeds = [segment_seed(Config().seed, s) for s in range(50)]
    assert len(set(seeds)) == 50 and all(0 <= s < 2**64 for s in seeds)
    assert not HELD_OUT_BATCH_SEEDS.intersection(seeds)


def test_the_first_ctrl_c_requests_a_stop_and_the_second_raises() -> None:
    before = signal.getsignal(signal.SIGINT)
    stop = Stop()
    with stop:
        signal.raise_signal(signal.SIGINT)
        assert stop.requested
        with pytest.raises(KeyboardInterrupt):
            signal.raise_signal(signal.SIGINT)
    assert signal.getsignal(signal.SIGINT) is before
    asked = Stop()
    asked.request()
    assert asked.requested


def test_a_resume_cuts_the_metrics_back_to_the_checkpoint(tmp_path: Path) -> None:
    out = tmp_path / "r"
    identity = {"started": "t0", **IDENTITY}
    config = load_config("samples = 40\n")
    run = Run(out, config, identity, "samples = 40\n")
    start = {"type": "start", "name": "r", "segment": 0, "seed": 1, "identity": identity}
    run.emit({**start, "update": 0, "samples": 0, "episodes": 0, "minutes": 0.0})
    run.emit(_update(1, 10, 1.0))
    run.emit(_update(2, 20, 3.0))
    length = run.sync()
    assert length == (out / "metrics.jsonl").stat().st_size
    run.emit(_update(3, 30, 5.0))
    run.close()
    kept = (out / "metrics.jsonl").read_bytes()[:length]
    # A kill in the middle of a line; a temporary of a checkpoint's write.
    with (out / "metrics.jsonl").open("ab") as f:
        f.write(b'{"type": "upd')
    (out / "checkpoint.pt.tmp").write_bytes(b"half")
    with pytest.raises(ResumeError, match="fewer than"):
        Run(out, config, {}, resume=(10**6, 0.0))
    again = Run(out, config, {}, resume=(length, 90.0))
    assert again.dropped == 2 and again.minutes() >= 1.5
    assert (out / "metrics.jsonl").read_bytes() == kept and not list(out.glob("*.tmp"))
    assert again.record.progress["updates"] == 2 and again.record.identity == identity
    segment = {"type": "segment", "segment": 1, "seed": 99, "dropped": again.dropped}
    resumed_at = {**segment, "changed": {"samples": [30, 40]}, "identity": {"commit": "def"}}
    again.emit({**resumed_at, "update": 2, "samples": 20, "episodes": 6, "minutes": 1.5})
    again.emit(_update(3, 30, 5.0))
    again.emit({"type": "stop", "update": 3, "samples": 30, "minutes": 2.0})
    again.write_record()
    again.close()
    lines = [json.loads(x) for x in (out / "metrics.jsonl").read_text().splitlines()]
    assert [x.get("update") for x in lines if x["type"] == "update"] == [1, 2, 3]
    record = json.loads((out / "record.json").read_text())
    assert check_record(record) == []
    first, second = record["segments"]
    assert (first["segment"], first["seed"], first["ended"], first["dropped"]) == (0, 1, "lost", 2)
    assert first["start"]["samples"] == 0 and first["end"]["samples"] == 20
    assert first["commit"] == "abc" and second["commit"] == "def"
    assert (second["ended"], second["changed"], second["end"]["updates"]) == (
        "stop",
        {"samples": [30, 40]},
        3,
    )
    assert record["progress"]["ended"] == "stop" and record["progress"]["minutes"] == 2.0
    assert "segment 1 from update 2" in line(lines[3]) and "samples 30 -> 40" in line(lines[3])
    assert "--resume" in line(lines[-1])


def test_the_held_out_check_needs_no_growing_set() -> None:
    held = HeldOut(episode_seeds(12, 4))
    assert len(held.seeds) == 12
    held.check([1, 2, 3])
    with pytest.raises(HeldOutError, match="held-out"):
        held.check([5, next(iter(held.seeds))])


def _gate(update: int, samples: int, result: str, seconds: float = 30.0) -> dict:
    return {
        "type": "gate",
        "update": update,
        "samples": samples,
        "episodes": 0,
        "minutes": update / 10,
        "best": 0,
        "result": result,
        "pairs": 100,
        "counts": [10, 20, 40, 20, 10],
        "llr": 3.0 if result == "promoted" else -3.0,
        "bounds": [-2.94, 2.94],
        "score": 0.6,
        "elo": 70.4,
        "games": 200,
        "seconds": seconds,
    }


def _promotion(update: int, samples: int, index: int, how: str = "gate") -> dict:
    return {
        "type": "promotion",
        "index": index,
        "how": how,
        "update": update,
        "samples": samples,
        "episodes": 0,
        "minutes": update / 10,
    }


def _ladder(update: int, samples: int, index: int, final: bool = False) -> dict:
    row = {"win_rate": 0.8, "interval": 0.02, "value": 0.2, "kinds": {"hero": 0.8}, "games": 6}
    return {
        "type": "ladder",
        "update": update,
        "samples": samples,
        "episodes": 0,
        "minutes": update / 10,
        "index": index,
        "final": final,
        "per_kind": 500 if final else 250,
        "opponents": {"random": row},
        "seconds": 90.0,
    }


def _update(update: int, samples: int, loss: float) -> dict:
    return {
        "type": "update",
        "update": update,
        "samples": samples,
        "episodes": 3 * update,
        "minutes": update / 10,
        "policy_loss": loss,
        "explained_variance": float("nan"),
    }


def test_a_run_writes_its_metrics_and_record(tmp_path: Path) -> None:
    shown: list[dict] = []
    identity = IDENTITY
    run = Run(
        tmp_path / "r", load_config("samples = 30\n"), identity, "samples = 30\n", shown.append
    )
    records = [
        {"type": "start", "name": "r", "update": 0, "samples": 0, "episodes": 0, "minutes": 0.0},
        _promotion(0, 0, 0, "start"),
        _ladder(0, 0, 0),
        _update(1, 10, 1.0),
        _update(2, 20, 3.0),
        _gate(2, 20, "promoted"),
        _promotion(2, 20, 1),
        _ladder(2, 20, 1),
        _update(3, 30, 5.0),
        _gate(3, 30, "rejected"),
        _ladder(3, 30, 1, final=True),
        {"type": "end", "reason": "budget", "update": 3, "samples": 30, "minutes": 0.5},
    ]
    for r in records:
        run.emit(r)
    run.write_record()
    run.close()
    out = tmp_path / "r"
    assert (out / "config.toml").read_text() == "samples = 30\n" and shown == records
    lines = [json.loads(x) for x in (out / "metrics.jsonl").read_text().splitlines()]
    assert [x["type"] for x in lines] == [r["type"] for r in records]
    record = json.loads((out / "record.json").read_text())
    assert check_record(record) == []
    assert record["id"] == "r" and record["config"]["samples"] == 30
    # The gates' and ladders' minutes: two gates of 30 s, three ladders of 90 s.
    assert record["progress"] == {
        "samples": 30,
        "updates": 3,
        "episodes": 9,
        "minutes": 0.5,
        "evaluation_minutes": 5.5,
        "ended": "budget",
    }
    # One curve point per gate interval, the means of its updates (a NaN counts nothing).
    assert [(p["samples"], p["policy_loss"]) for p in record["curve"]] == [(20, 2.0), (30, 5.0)]
    assert all("explained_variance" not in p for p in record["curve"])
    assert [(g["samples"], g["result"]) for g in record["gates"]] == [
        (20, "promoted"),
        (30, "rejected"),
    ]
    assert [(p["index"], p["how"], p["samples"]) for p in record["promotions"]] == [
        (0, "start", 0),
        (1, "gate", 20),
    ]
    assert [(x["index"], x["final"]) for x in record["ladders"]] == [
        (0, False),
        (1, False),
        (1, True),
    ]
    assert record["ladders"][0]["opponents"]["random"] == {
        "win_rate": 0.8,
        "interval": 0.02,
        "value": 0.2,
        "kinds": {"hero": 0.8},
    }
    assert record["best"] == {"index": 1, "samples": 20} and record["parent"] is None
    assert [(s["segment"], s["ended"]) for s in record["segments"]] == [(0, "budget")]
    assert line(records[0]) is None and "budget" in line(records[-1])
    # A new run never appends to another's records.
    with pytest.raises(RunExistsError):
        Run(out, load_config(""), identity)
    with pytest.raises(SystemExit):
        parse(["--out", str(out)])


def test_check_record_names_what_is_missing() -> None:
    assert check_record([]) == ["not a JSON object"]
    assert check_record({"schema": RECORD_SCHEMA + 1})[0].startswith("schema")
    problems = check_record(
        {
            "schema": RECORD_SCHEMA,
            "id": "",
            "identity": {},
            "config": {},
            "progress": {},
            "segments": [],
            "curve": [],
            "gates": [{}],
            "promotions": [],
            "ladders": [{"index": 0}],
        }
    )
    assert "`identity.commit` is missing" in problems and "`id` is empty" in problems
    assert "`identity.entries` is missing" in problems and "`parent` is missing" in problems
    assert "`segments` is empty" in problems and "`config.gate` is missing" in problems
    assert "`progress.evaluation_minutes` is missing" in problems
    assert problems[-2].startswith("gates 0 lacks") and problems[-1].startswith("ladders 0 lacks")


def test_every_committed_record_and_config_is_valid() -> None:
    # `bots-<set>.json` is the bots' table (`h3 ai eval --json`, `tests/test_evaluate.py`).
    for path in sorted((PYTHON / "records").glob("*.json")):
        if path.name.startswith("bots-"):
            continue
        assert check_record(json.loads(path.read_text(encoding="utf-8"))) == [], path.name
    configs = sorted((PYTHON / "configs").glob("*.toml"))
    assert configs
    for path in configs:
        load_config(path.read_text(encoding="utf-8"))
