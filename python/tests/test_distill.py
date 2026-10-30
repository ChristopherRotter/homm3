"""The distillation bench (E25 T7b, `h3env.train.distill`): its config, its scores (a student
that is the teacher scores KL 0 and full agreement), a bench run end to end whose students learn
and whose files load. They need the group `train`."""

from __future__ import annotations

import copy
import json
from dataclasses import asdict, replace
from pathlib import Path

import pytest

from h3env import _ffi
from h3env.train import layout as L
from h3env.train.run import Config, ConfigError

torch = pytest.importorskip("torch")

from h3env.train.distill import (  # noqa: E402
    CATEGORIES,
    Bench,
    Student,
    category,
    collect,
    copied,
    evaluate,
    load_bench,
    run,
    student_config,
)
from h3env.train.evaluate import load  # noqa: E402
from h3env.train.hexnet import HexConv  # noqa: E402
from h3env.train.policy import Mlp  # noqa: E402
from h3env.train.ppo import CHECKPOINT_FORMAT, Trainer  # noqa: E402

#: A small teacher's run (an MLP with shaping and a tactics bound, as `e25-scale.toml`).
TEACHER = Config(
    envs=8,
    threads=2,
    max_steps=300,
    rollout=16,
    hidden=32,
    embed=8,
    shaping=True,
    tactics_rounds=3,
    device="cpu",
)


def teacher_file(tmp_path):
    """A teacher's network file whose entry head prefers Wait and Defend (a policy to learn)."""
    torch.manual_seed(0)
    trainer = Trainer(TEACHER)
    with torch.no_grad():
        trainer.policy.entry_head.bias[[0, 1]] = 4.0
    path = tmp_path / "teacher.pt"
    torch.save(trainer.network(), path)
    trainer.batch.close()
    return path


def test_the_bench_config_refuses_what_it_does_not_know() -> None:
    bench = load_bench("samples = 10\n[students.a]\nkind = 'hexconv'\nwidth = 8\nlr = 1e-3\n")
    assert bench.samples == 10 and bench.students == {
        "a": {"kind": "hexconv", "width": 8, "lr": 1e-3}
    }
    assert load_bench("[students.a]\n", ["samples=5", "device='cpu'"]).samples == 5
    for text, match in (
        ("sample = 1\n[students.a]\n", "unknown bench keys sample"),
        ("samples = 1\n", "no `\\[students.<name>\\]` table"),
        ("[students.a]\ngamma = 0.9\n", "gamma are no `\\[network\\]` keys"),
        ("[students.a]\npreview = true\n", "kill preview"),
        ("epochs = 0\n[students.a]\n", "`epochs` must be positive"),
    ):
        with pytest.raises(ConfigError, match=match):
            load_bench(text)
    with pytest.raises(ConfigError, match="--set takes key=value"):
        load_bench("[students.a]\n", ["samples"])


def test_every_committed_bench_config_is_valid() -> None:
    # `configs/distill/` holds the benches (the run configs beside it are `test_run`'s).
    configs = sorted((Path(__file__).parents[1] / "configs" / "distill").glob("*.toml"))
    assert configs
    for path in configs:
        bench = load_bench(path.read_text(encoding="utf-8"))
        for keys in bench.students.values():
            student_config(Config(), keys)
    e25 = load_bench((configs[0].parent / "e25.toml").read_text(encoding="utf-8"))
    kinds = [keys["kind"] for keys in e25.students.values()]
    assert kinds == ["mlp", "mlp", "mlp", "hexconv", "hexformer", "entity"]


def test_every_committed_bench_report_is_complete() -> None:
    # `records/distill/<bench>.json` (E25 T7c): a bench's `report.json`, every evaluation scoring
    # every student, the bench's settings loadable.
    reports = sorted((Path(__file__).parents[1] / "records" / "distill").glob("*.json"))
    assert reports
    for path in reports:
        report = json.loads(path.read_text(encoding="utf-8"))
        assert {"teacher", "bench", "identity", "students", "evaluations", "ended"} <= set(report)
        assert report["ended"] in ("budget", "stop")
        bench = Bench(**report["bench"])
        assert list(report["students"]) == list(bench.students)
        assert report["evaluations"][0]["samples"] == 0
        for evaluation in report["evaluations"]:
            assert list(evaluation["students"]) == list(bench.students)
            assert set(evaluation["categories"]) == set(CATEGORIES)


def test_the_categories_follow_the_entries_and_the_tactics_phase() -> None:
    obs = torch.ones(9, L.OBSERVATION_LEN)
    obs[8, L.ROUND] = 0.0
    entry = torch.tensor(
        [0, L.MOVE_BASE + 3, L.ATTACK_BASE, L.SHOOT_BASE + 1, L.HEAL_BASE, _ffi.HERO_CAST]
        + [_ffi.CREATURE_CAST, L.END_TACTICS, L.MOVE_BASE]
    )
    names = [CATEGORIES[i] for i in category(obs, entry).tolist()]
    assert names == ["wait_defend", "move", "melee", "shot", "heal", "cast", "cast", "other"] + [
        "tactics"
    ]


def test_a_student_that_is_the_teacher_scores_no_divergence(tmp_path) -> None:
    trainer = Trainer(replace(TEACHER, seed=7, self_play=1.0))
    teacher = trainer.policy.eval()
    with torch.no_grad():
        teacher.entry_head.bias[[0, 1]] = 4.0
    data = copied(*collect(trainer))
    trainer.batch.close()
    twin = copy.deepcopy(teacher)
    other = Mlp.for_batch(trainer.batch, hidden=16, embed=8)
    students = [
        Student("twin", twin, TEACHER, torch.optim.Adam(twin.parameters()), 1e-3),
        Student("other", other, TEACHER, torch.optim.Adam(other.parameters()), 1e-3),
    ]
    out = evaluate(teacher, students, data, torch.device("cpu"), size=7)
    assert out["heldout"] == data["returns"].shape[0] > 0
    same = out["students"]["twin"]
    assert same["kl"]["entry"] == pytest.approx(0.0, abs=1e-6) and same["agreement"] == 1.0
    assert all(v == pytest.approx(0.0, abs=1e-5) for v in same["kl"].values())
    assert same["value"] == pytest.approx(out["teacher"])
    assert out["students"]["other"]["kl"]["entry"] > 0.01
    assert sum(out["categories"].values()) == out["heldout"]


def test_a_bench_trains_its_students_and_saves_them(tmp_path) -> None:
    path = teacher_file(tmp_path)
    bench = Bench(
        seed=7,
        samples=600,
        heldout=150,
        eval_every=300,
        envs=8,
        threads=2,
        rollout=16,
        epochs=2,
        minibatch=64,
        lr=3e-3,
        device="cpu",
        students={
            "mlp": {"kind": "mlp", "hidden": 32},
            "conv": {"kind": "hexconv", "width": 8, "layers": 1},
        },
    )
    lines: list[str] = []
    report = run(path, bench, tmp_path / "bench", log=lines.append)
    assert report["ended"] == "budget" and report["samples"] >= 600
    evaluations = report["evaluations"]
    assert evaluations[0]["samples"] == 0 and evaluations[-1]["samples"] == report["samples"]
    assert len(evaluations) == len({e["samples"] for e in evaluations}) >= 3
    first, last = evaluations[0]["students"]["mlp"], evaluations[-1]["students"]["mlp"]
    assert last["kl"]["entry"] < first["kl"]["entry"] and last["seconds"] > 0
    assert report["students"]["conv"]["network"]["kind"] == "hexconv"
    assert report["teacher"]["config"]["tactics_rounds"] == 3
    saved = json.loads((tmp_path / "bench" / "report.json").read_text())
    assert saved["evaluations"] == json.loads(json.dumps(evaluations))
    metrics = (tmp_path / "bench" / "metrics.jsonl").read_text().splitlines()
    assert len(metrics) == len(evaluations)
    conv = load(tmp_path / "bench" / "students" / "conv.pt")
    assert isinstance(conv, HexConv) and conv.tactics_rounds == 3
    file = torch.load(tmp_path / "bench" / "students" / "conv.pt", weights_only=True)
    assert file["format"] == CHECKPOINT_FORMAT and file["config"]["kind"] == "hexconv"
    assert asdict(Config(**file["config"])) == file["config"]
    assert any("held-out set" in line for line in lines)
    with pytest.raises(ConfigError, match="does not resume"):
        run(path, bench, tmp_path / "bench")
