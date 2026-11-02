"""The cloning bench (E25 T16, `h3env.train.clone`, the distillation bench's clone mode): the fair
system actor's episodes and their returns, the cross-entropy on its choices, a bench run end to
end whose student learns, saves as a run's network, starts a run and plays its ladder. They need
the group `train`."""

from __future__ import annotations

import json
from dataclasses import replace

import numpy as np
import pytest

from h3env.train.run import ConfigError, load_config

torch = pytest.importorskip("torch")

from h3env.train import distill  # noqa: E402
from h3env.train.clone import FIELDS, Episodes, cross_entropy, run  # noqa: E402
from h3env.train.distill import Bench, heads  # noqa: E402
from h3env.train.evaluate import load  # noqa: E402
from h3env.train.policy import Mlp  # noqa: E402
from h3env.train.ppo import Staging, Trainer  # noqa: E402

#: A small run on every kind (the neutral ones' defender the fair system actor inside the call),
#: with shaping and a tactics bound, as `e25-neutral.toml`.
RUN = """
seed = 1

[env]
envs = 8
threads = 2
max_steps = 300
kinds = [1, 1, 2, 1, 1, 1]

[network]
hidden = 32
embed = 8
tactics_rounds = 3

[ppo]
rollout = 16
shaping = true

[device]
device = "cpu"
"""


def test_the_teachers_episodes_are_whole_and_shaped_by_the_potential() -> None:
    # The same seed gives the same decisions with and without shaping (the teacher reads no
    # potential); a kept decision's return is its side's reward v1 at the end (the truncation
    # penalty for a cut battle) minus the potential at it, 0 at the side's first decision.
    shaped = load_config(RUN)
    plain = replace(shaped, shaping=False)
    kept = []
    for config in (shaped, plain):
        episodes = Episodes(config, 7, 8, 2)
        arrays, idx = episodes.take(300)
        assert len(idx) >= 300 and episodes.stats["lost"] == 0
        assert episodes.stats["decisions"] == episodes.size
        kept.append(({k: v[idx, 0].copy() for k, v in arrays.items()}, episodes))
    (s, shaped_episodes), (p, _) = kept
    n = min(len(s["returns"]), len(p["returns"]))
    assert (s["obs"][:n] == p["obs"][:n]).all() and (s["entry"][:n] == p["entry"][:n]).all()
    phi = shaped_episodes.arrays["phi"][np.sort(np.concatenate(shaped_episodes.complete))][:n]
    assert np.allclose(p["returns"][:n] - s["returns"][:n], phi, atol=1e-5)
    assert (phi == 0).sum() >= 2 and (phi != 0).any()
    stats = shaped_episodes.stats
    assert stats["episodes"] > 0 and set(stats["by_kind"]) <= {
        "wandering",
        "guards",
        "hero",
        "siege_hero",
        "siege_garrison",
        "siege_neutral",
    }
    # The teacher's choices are legal, the casts with their heads' masks.
    assert s["entry_mask"][np.arange(n), s["entry"][:n]].all()
    cast = s["cast"][:n]
    rows = np.nonzero(cast)[0]
    assert s["spell_mask"][rows, s["spell"][rows]].all()
    assert s["target_mask"][rows, s["target"][rows]].all()
    assert s["second_mask"][rows, s["second"][rows]].all()
    shaped_episodes.close()


def test_a_release_keeps_the_open_episodes_rows() -> None:
    episodes = Episodes(replace(load_config(RUN), shaping=False), 9, 8, 2)
    episodes.take(100)
    open_rows = [list(rows) for rows in episodes.open]
    before = [episodes.arrays["obs"][rows].copy() for rows in open_rows]
    episodes.release()
    assert episodes.completed == 0 and episodes.size == sum(len(r) for r in open_rows)
    for rows, obs in zip(episodes.open, before, strict=True):
        assert (episodes.arrays["obs"][rows] == obs).all()
    arrays, idx = episodes.take(100)
    assert len(idx) >= 100 and set(FIELDS) <= set(arrays)
    episodes.close()


def test_the_cross_entropy_is_the_students_log_probability_of_the_teachers_choices() -> None:
    episodes = Episodes(load_config(RUN), 11, 8, 2)
    arrays, idx = episodes.take(200)
    net = Mlp.for_batch(episodes.batch, hidden=16, embed=8)
    m = Staging(torch.device("cpu")).gather(arrays, idx)
    with torch.no_grad():
        mine = heads(net, m)
        loss, per = cross_entropy(mine, m)
    logp = mine[0][torch.arange(len(idx)), m["entry"]]
    assert torch.allclose(per["entry"], -logp)
    cast = int(m["cast"].sum())
    assert all(per[h].shape == (cast,) for h in ("spell", "target", "second"))
    total = per["entry"].mean() + sum(per[h].sum() for h in ("spell", "target", "second")) / len(
        idx
    )
    assert torch.allclose(loss, total) and torch.isfinite(loss)
    episodes.close()


def test_a_clone_bench_trains_saves_starts_a_run_and_plays_its_ladder(tmp_path) -> None:
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
        ladder=["system"],
        ladder_per_kind=2,
        students={"mlp": {"kind": "mlp"}},
    )
    with pytest.raises(ConfigError, match="needs `run`"):
        run(bench, tmp_path / "none")
    lines: list[str] = []
    report = run(bench, tmp_path / "bench", log=lines.append, config_text=RUN)
    assert report["ended"] == "budget" and report["samples"] >= 600
    evaluations = report["evaluations"]
    assert evaluations[0]["samples"] == 0 and evaluations[-1]["samples"] == report["samples"]
    first, last = evaluations[0]["students"]["mlp"], evaluations[-1]["students"]["mlp"]
    assert last["cross_entropy"]["entry"] < first["cross_entropy"]["entry"]
    assert last["agreement"] > first["agreement"] and last["seconds"] > 0
    assert sum(evaluations[-1]["categories"].values()) == evaluations[-1]["heldout"] == 150
    stats = report["teacher"]["stats"]
    assert stats["lost"] == 0 and stats["decisions"] > 600 and stats["episodes"] > 0
    assert report["teacher"]["config"]["tactics_rounds"] == 3
    assert report["identity"]["abi"] == 37 and report["teacher"]["system_policy"] > 0
    ladders = report["ladders"]["mlp"]
    assert set(ladders) == {"report-v1", "neutral-v1"}
    assert set(ladders["report-v1"]) == {"system"} and set(ladders["neutral-v1"]) == {"system"}
    assert 0.0 <= ladders["report-v1"]["system"]["win_rate"] <= 1.0
    saved = json.loads((tmp_path / "bench" / "report.json").read_text())
    assert saved["evaluations"] == json.loads(json.dumps(evaluations))
    assert Bench(**saved["bench"]) == bench
    path = tmp_path / "bench" / "students" / "mlp.pt"
    net = load(path)
    assert isinstance(net, Mlp) and net.tactics_rounds == 3
    assert torch.load(path, weights_only=True)["config"]["seed"] == bench.seed
    # The student starts a run of its config (`--init-from`).
    trainer = Trainer(load_config(RUN))
    parent = trainer.init_from(path)
    assert parent["samples"] == report["samples"]
    for a, b in zip(trainer.policy.parameters(), net.parameters(), strict=True):
        assert torch.equal(a, b)
    trainer.batch.close()
    assert any("held-out set" in line for line in lines)
    with pytest.raises(ConfigError, match="does not resume"):
        run(bench, tmp_path / "bench", config_text=RUN)


def test_the_distillation_bench_leaves_the_clone_modes_keys_to_it(tmp_path) -> None:
    with pytest.raises(ConfigError, match="clone mode"):
        distill.run(tmp_path / "teacher.pt", Bench(run="x.toml", students={"a": {}}), tmp_path)
    with pytest.raises(ConfigError, match="are no opponents"):
        distill.load_bench("ladder = ['nobody']\n[students.a]\n")
    with pytest.raises(SystemExit):
        distill.main(
            ["--teacher", "system-all-seeing", "--config", "x.toml", "--out", str(tmp_path)]
        )
