"""The distillation bench (E25 T7b, `h3env.train.distill`): its config, its scores (a student
that is the teacher scores KL 0 and full agreement), a bench run end to end whose students learn
and whose files load; its optimizer (Adam's steps without weight decay, the decay on the matrices
only) and schedule (the warm-up, then the fall as before) and each student's cost (E25 T24); a
stopped bench resuming to its budget (E25 T26a). They need the group `train`."""

from __future__ import annotations

import copy
import json
import signal
from dataclasses import asdict, replace
from pathlib import Path

import pytest

from h3env import _ffi
from h3env.train import layout as L
from h3env.train.run import Config, ConfigError, ResumeError, Stop, segment_seed

torch = pytest.importorskip("torch")

from h3env.train.distill import (  # noqa: E402
    CATEGORIES,
    Bench,
    Student,
    category,
    collect,
    copied,
    cost,
    evaluate,
    learning_rate,
    load_bench,
    main,
    optimizer,
    resume,
    run,
    steps_of,
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
    # The optimizer's keys (E25 T24): the bench's, and a student's over them, never its network's.
    bench = load_bench(
        "warmup = 100\nweight_decay = 0.01\n[students.a]\nwarmup = 5\nweight_decay = 0.1\n"
    )
    assert (bench.warmup, bench.weight_decay) == (100, 0.01)
    assert student_config(Config(), bench.students["a"]) == student_config(Config(), {})
    for text, match in (
        ("warmup = -1\n[students.a]\n", "`warmup` must not be negative"),
        ("[students.a]\nweight_decay = -0.1\n", "student 'a': `weight_decay` must not be"),
    ):
        with pytest.raises(ConfigError, match=match):
            load_bench(text)
    # The committed benches keep T16's and T20's settings: no warm-up, Adam's steps.
    for name in ("e25-clone.toml", "e25-clone-2.toml"):
        path = Path(__file__).parents[1] / "configs" / "distill" / name
        clone = load_bench(path.read_text(encoding="utf-8"))
        assert (clone.warmup, clone.weight_decay) == (0, 0.0)
        assert all(set(keys) == {"kind"} for keys in clone.students.values())


def test_the_optimizer_is_adams_without_weight_decay_and_spares_the_biases() -> None:
    # At weight decay 0 (every bench before E25 T24) the steps are Adam's (ε 1e-5) bit for bit.
    torch.manual_seed(0)
    net = torch.nn.Sequential(torch.nn.Linear(6, 5), torch.nn.LayerNorm(5), torch.nn.Linear(5, 1))
    twin = copy.deepcopy(net)
    mine, adam = optimizer(net, 1e-2), torch.optim.Adam(twin.parameters(), lr=1e-2, eps=1e-5)
    x = torch.randn(32, 6)
    for _ in range(5):
        for model, opt in ((net, mine), (twin, adam)):
            opt.zero_grad()
            model(x).square().mean().backward()
            opt.step()
    for a, b in zip(net.parameters(), twin.parameters(), strict=True):
        assert torch.equal(a, b)
    # With decay: a step without gradient shrinks the matrices by (1 − lr · decay) and leaves
    # the biases and the norm's gains as they are.
    before = copy.deepcopy(net)
    decaying = optimizer(net, 0.1, weight_decay=0.5)
    for p in net.parameters():
        p.grad = torch.zeros_like(p)
    decaying.step()
    for (name, a), b in zip(net.named_parameters(), before.parameters(), strict=True):
        expected = b * (1 - 0.1 * 0.5) if a.ndim >= 2 else b
        assert torch.allclose(a, expected), name


def test_the_learning_rate_warms_up_then_falls_as_before() -> None:
    bench = Bench(samples=10_000, lr_floor=0.1)
    s = Student("a", None, Config(), None, 1e-3, warmup=1_000)
    assert learning_rate(s, bench, 0, 0) == 0.0
    assert learning_rate(s, bench, 0, 250) == pytest.approx(2.5e-4)
    assert learning_rate(s, bench, 0, 1_000) == pytest.approx(1e-3)
    assert learning_rate(s, bench, 5_000, 6_000) == pytest.approx(1e-3 * (1 - 0.9 * 0.5))
    # Without a warm-up the rate is T7b's, exactly: the fall by the batch's start.
    s.warmup = 0
    for samples in (0, 3_000, 10_000, 12_000):
        frac = min(1.0, samples / bench.samples)
        assert learning_rate(s, bench, samples, samples + 77) == 1e-3 * (1.0 - 0.9 * frac)
    # A batch of 1 000 rows in minibatches of 300, two passes: three steps a pass.
    assert steps_of(1_000, 300, 2) == 6 and steps_of(300, 300, 1) == 1
    assert cost(s) == {"us_per_sample_pass": None, "peak_memory_mb": None}
    s.seconds, s.rows = 2.0, 4_000
    assert cost(s)["us_per_sample_pass"] == pytest.approx(500.0)


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
            "conv": {
                "kind": "hexconv",
                "width": 8,
                "layers": 1,
                "norm": "layer",
                "warmup": 200,
                "weight_decay": 0.01,
            },
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
    # The cost (E25 T24): none before training, then the µs a sample-pass; no CUDA memory on
    # the CPU.
    assert first["us_per_sample_pass"] is None and last["us_per_sample_pass"] > 0
    assert last["peak_memory_mb"] is None and "µs" in lines[-1].splitlines()[1]
    conv = report["students"]["conv"]
    assert conv["network"]["kind"] == "hexconv" and conv["network"]["norm"] == "layer"
    assert (conv["warmup"], conv["weight_decay"]) == (200, 0.01)
    assert (report["students"]["mlp"]["warmup"], report["students"]["mlp"]["weight_decay"]) == (
        0,
        0.0,
    )
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
    with pytest.raises(ConfigError, match="not empty"):
        run(path, bench, tmp_path / "bench")


def same(a, b) -> None:
    """`a` and `b` (state dicts, nested) hold equal tensors and values."""
    assert a.keys() == b.keys()
    for k, v in a.items():
        if isinstance(v, dict):
            same(v, b[k])
        elif torch.is_tensor(v):
            assert torch.equal(v, b[k])
        else:
            assert v == b[k]


def test_a_stopped_bench_resumes_to_its_budget(tmp_path) -> None:
    # Ctrl+C at the first evaluation past the start (E25 T26a): the bench stops at that batch
    # boundary, saved. A resume refuses a change to the training; one stopped at once saves the
    # students as it restored them; the next continues to a larger budget, the report the whole
    # run's.
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
            "conv": {"kind": "hexconv", "width": 8, "layers": 1, "warmup": 200},
        },
    )
    out = tmp_path / "bench"

    def interrupt(line: str) -> None:
        if line.startswith("at ") and not line.startswith("at 0 "):
            signal.raise_signal(signal.SIGINT)

    stopped = run(path, bench, out, log=interrupt)
    assert stopped["ended"] == "stop" and 300 <= stopped["samples"] < 600
    assert [e["samples"] for e in stopped["evaluations"]] == [0, stopped["samples"]]
    state = torch.load(out / "checkpoint.pt", weights_only=True)
    assert (state["format"], state["mode"], state["segment"], state["finished"]) == (
        1,
        "distill",
        0,
        False,
    )
    assert (state["samples"], state["next_eval"], state["horizon"]) == (
        stopped["samples"],
        600,
        600,
    )
    assert state["metrics_length"] == (out / "metrics.jsonl").stat().st_size
    assert json.loads(state["report"]) == json.loads((out / "report.json").read_text())
    for name in bench.students:
        saved = torch.load(out / "students" / f"{name}.pt", weights_only=True)
        same(saved["policy"], state["students"][name]["policy"])
        assert state["students"][name]["rows"] > 0
    heldout = (out / "heldout.npz").read_bytes()

    before = (out / "metrics.jsonl").read_bytes()
    with pytest.raises(ConfigError, match="lr would change the training"):
        resume(out, ["lr=1e-3"])
    with pytest.raises(SystemExit):
        main(["--resume", str(out), "--config", "x.toml"])
    assert (out / "metrics.jsonl").read_bytes() == before
    assert torch.load(out / "checkpoint.pt", weights_only=True)["segment"] == 0

    # Stopped before its first batch: the students as restored, saved again in segment 1.
    halt = Stop()
    halt.request()
    again = resume(out, stop=halt)
    assert again["ended"] == "stop" and again["samples"] == stopped["samples"]
    restored = torch.load(out / "checkpoint.pt", weights_only=True)
    assert restored["segment"] == 1
    for name in bench.students:
        same(restored["students"][name], state["students"][name])
    assert (out / "metrics.jsonl").read_bytes() == before

    finished = resume(out, ["samples=900"])
    assert finished["ended"] == "budget" and finished["samples"] >= 900
    samples = [e["samples"] for e in finished["evaluations"]]
    assert samples[0] == 0 and samples == sorted(set(samples)) and samples[-1] >= 900
    assert [(s["segment"], s["ended"]) for s in finished["segments"]] == [
        (0, "stop"),
        (1, "stop"),
        (2, "budget"),
    ]
    third = finished["segments"][2]
    assert third["changed"] == {"samples": [600, 900]} and third["samples"] == stopped["samples"]
    assert third["seed"] == segment_seed(7, 2) != 7 and finished["bench"]["samples"] == 900
    saved = json.loads((out / "report.json").read_text())
    assert saved == json.loads(json.dumps(finished))
    lines = (out / "metrics.jsonl").read_text().splitlines()
    assert [json.loads(line) for line in lines] == saved["evaluations"]
    # The training counts on across the segments; the held-out set stays the one collected.
    first, last = (finished["evaluations"][i]["students"]["mlp"] for i in (0, -1))
    middle = stopped["evaluations"][-1]["students"]["mlp"]
    assert last["seconds"] > middle["seconds"] and last["kl"]["entry"] < first["kl"]["entry"]
    assert (out / "heldout.npz").read_bytes() == heldout
    end = torch.load(out / "checkpoint.pt", weights_only=True)
    assert end["finished"] and end["horizon"] == 600 and end["samples"] == finished["samples"]
    with pytest.raises(ResumeError, match="extend it"):
        resume(out)
    with pytest.raises(ResumeError, match="no checkpoint"):
        resume(tmp_path / "none")


def test_a_stop_inside_a_batch_counts_its_samples_pro_rata(tmp_path, monkeypatch) -> None:
    # E25 T26a: Ctrl+C during the third student step (the second minibatch's first student) of
    # the first batch: the second minibatch finishes, the bench stops before the third, and the
    # batch counts the share of its samples its steps took.
    from h3env.train import distill

    calls = 0
    real = distill.step

    def counted(*args, **kwargs) -> None:
        nonlocal calls
        calls += 1
        real(*args, **kwargs)
        if calls == 3:
            signal.raise_signal(signal.SIGINT)

    monkeypatch.setattr(distill, "step", counted)
    bench = Bench(
        seed=7,
        samples=600,
        heldout=64,
        envs=8,
        threads=2,
        rollout=16,
        epochs=2,
        minibatch=16,
        device="cpu",
        students={"a": {"kind": "mlp", "hidden": 16}, "b": {"kind": "mlp", "hidden": 8}},
    )
    report = run(teacher_file(tmp_path), bench, tmp_path / "bench")
    state = torch.load(tmp_path / "bench" / "checkpoint.pt", weights_only=True)
    assert calls == 4 and report["ended"] == "stop"
    assert [state["students"][s]["rows"] for s in ("a", "b")] == [32, 32]
    # Two of the batch's 2 × (n // 16) minibatches: its n samples' share, rounded down.
    assert 0 < state["samples"] == report["samples"] < 128
    assert [e["samples"] for e in report["evaluations"]] == [0, state["samples"]]
