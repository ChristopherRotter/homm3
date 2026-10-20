"""A run's lifecycle without torch (E06 T2a; ADR-0026, `docs/epics/E06-design.md` §1, §2, §4,
§5.3): the config file and `--set`, the schedule in samples, atomic writes, the held-out check,
the run's directory and record, and every committed record and config."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from h3env.train import Config
from h3env.train.__main__ import line, parse
from h3env.train.evaluate import EVAL_SEED, episode_seeds
from h3env.train.run import (
    RECORD_SCHEMA,
    TABLES,
    Cadence,
    ConfigError,
    HeldOut,
    HeldOutError,
    Run,
    RunExistsError,
    atomic_write,
    atomic_write_text,
    by_table,
    check_record,
    learning_rate,
    load_config,
)

PYTHON = Path(__file__).resolve().parents[1]


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
    c = load_config(text, ["ppo.gamma=0.99", "device.device=cpu", "evaluation.eval_greedy=false"])
    assert (c.seed, c.samples, c.envs, c.kinds) == (3, 2_000_000, 64, [0, 0, 1, 0, 0, 0])
    assert (c.lr, c.lr_schedule, c.gamma, c.device, c.eval_greedy) == (
        1e-4,
        "constant",
        0.99,
        "cpu",
        False,
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
        ("", ["evaluation.eval_start=1"], "`eval_start` takes true or false"),
        ("", ["env.kinds=[1, 2]"], "six weights"),
        ("", ["ppo.lr"], "table.key=value"),
        ("seed = [", [], "not TOML"),
        ("", ["ppo.lr_schedule=cosine"], "linear or constant"),
        ("", ["samples=0"], "`samples` must be positive"),
        ("", [f"seed={EVAL_SEED}"], "held-out episodes' batch seed"),
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
    # The held-out set v0's seed is E05's (`h3env.train.evaluate.EVAL_SEED`, `h3 ai eval`'s).
    assert Config().eval_seed == EVAL_SEED


def test_the_command_line_reads_the_file_and_the_overrides(tmp_path: Path) -> None:
    path = tmp_path / "run.toml"
    path.write_text("samples = 5_000\n[ppo]\nlr = 3e-4\n", encoding="utf-8")
    config, out, text = parse(
        ["--config", str(path), "--set", "ppo.lr=1e-4", "--out", str(tmp_path / "r")]
    )
    assert (config.samples, config.lr, out) == (5_000, 1e-4, tmp_path / "r")
    assert text == path.read_text(encoding="utf-8")
    with pytest.raises(SystemExit):
        parse(["--set", "ppo.lr=fast"])


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


def test_the_held_out_check_needs_no_growing_set() -> None:
    held = HeldOut(episode_seeds(12, 4))
    assert len(held.seeds) == 12
    held.check([1, 2, 3])
    with pytest.raises(HeldOutError, match="held-out"):
        held.check([5, next(iter(held.seeds))])


def _eval(samples: int, rate: float) -> dict:
    side = {"win_rate": rate}
    return {
        "type": "eval",
        "final": False,
        "update": samples // 10,
        "samples": samples,
        "episodes": 0,
        "minutes": 0.0,
        "sampled": {"win_rate": rate, "attacker": side, "defender": side},
        "random": {"win_rate": 0.5, "attacker": side, "defender": side},
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
    identity = {"commit": "abc", "dirty": False, "abi": 30, "torch": "x", "device": "cpu"}
    run = Run(
        tmp_path / "r", load_config("samples = 30\n"), identity, "samples = 30\n", shown.append
    )
    records = [
        {"type": "start", "name": "r", "update": 0, "samples": 0, "episodes": 0, "minutes": 0.0},
        _eval(0, 0.5),
        _update(1, 10, 1.0),
        _update(2, 20, 3.0),
        _eval(20, 0.7),
        _update(3, 30, 5.0),
        {**_eval(30, 0.6), "final": True},
        {**_eval(30, 0.6), "final": True, "repeated": True},
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
    assert record["progress"] == {
        "samples": 30,
        "updates": 3,
        "episodes": 9,
        "minutes": 0.5,
        "ended": "budget",
    }
    # One curve point per evaluation interval, the means of its updates (a NaN counts nothing).
    assert [(p["samples"], p["policy_loss"]) for p in record["curve"]] == [(20, 2.0), (30, 5.0)]
    assert all("explained_variance" not in p for p in record["curve"])
    # The repeated final evaluation is not a second one.
    assert [e["samples"] for e in record["evaluations"]] == [0, 20, 30]
    assert record["evaluations"][0]["random"]["win_rate"] == 0.5
    assert record["best"] == {"samples": 20, "win_rate": 0.7}
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
            "curve": [],
            "evaluations": [{}],
        }
    )
    assert "`identity.commit` is missing" in problems and "`id` is empty" in problems
    assert "`config.ppo` is missing" in problems and "evaluation 0 lacks" in problems[-1]


def test_every_committed_record_and_config_is_valid() -> None:
    for path in sorted((PYTHON / "records").glob("*.json")):
        assert check_record(json.loads(path.read_text(encoding="utf-8"))) == [], path.name
    configs = sorted((PYTHON / "configs").glob("*.toml"))
    assert configs
    for path in configs:
        load_config(path.read_text(encoding="utf-8"))
