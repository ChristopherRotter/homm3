"""The update's speed tool (E25 T31, `h3env.train.speed`): its variants update from one rollout as
the baseline does, the phases count each call, nothing stays patched. They need the group
`train` and the release library."""

from __future__ import annotations

import json

import pytest

torch = pytest.importorskip("torch")

from h3env.train import speed  # noqa: E402
from h3env.train.ppo import Trainer  # noqa: E402
from h3env.train.run import load_config  # noqa: E402

#: A small hex convolution run with the imitation loss's labels and accumulated chunks.
SMALL = """
seed = 3
[env]
envs = 4
threads = 2
max_steps = 300
[network]
kind = "hexconv"
norm = "layer"
width = 8
layers = 2
embed = 8
mask_input = true
order = true
attack_hex = true
[ppo]
rollout = 16
epochs = 2
minibatches = 2
accumulate = 2
imitation_coef = 0.5
imitation_horizon = 1_000_000
[device]
device = "cpu"
"""


def test_the_variants_update_from_one_rollout_as_the_baseline_does(tmp_path) -> None:
    # Each variant's first update starts from the same weights, optimizer state and counters:
    # its losses and weights are the baseline's up to rounding (the chunks' sums and pads), as
    # `test_an_accumulated_update_steps_as_its_whole_minibatches_do` and
    # `test_a_padded_update_steps_as_the_unpadded_one_does` bound them; the report holds every
    # part the command line writes.
    out = tmp_path / "speed.json"
    config = tmp_path / "small.toml"
    config.write_text(SMALL, encoding="utf-8")
    speed.main(
        [
            "--config",
            str(config),
            "--variants",
            "heuristics,accumulate=3+align32",
            "--repeats",
            "2",
            "--trace",
            "--sweep",
            "8,12",
            "--sweep-repeats",
            "1",
            "--json",
            str(out),
        ]
    )
    report = json.loads(out.read_text(encoding="utf-8"))
    # The sweep: each size's pass with the autotuner and without, the first size again at the end.
    assert [e["rows"] for e in report["sweep"]] == [8, 12, 8]
    for e in report["sweep"]:
        assert e["plain_us_per_row"] > 0 and e["autotune_us_per_row"] > 0, e
    variants = report["variants"]
    assert list(variants) == ["baseline", "heuristics", "accumulate=3+align32"]
    assert report["epochs"] == 1 and report["rows"] > 0 and report["teacher_rows"] > 0
    assert variants["baseline"]["autotune"] and not variants["heuristics"]["autotune"]
    assert variants["accumulate=3+align32"]["accumulate"] == 3
    assert variants["baseline"]["weights_max_diff"] == 0.0 and report["weights_moved"] > 0
    # The CPU's kernels are deterministic: the baseline's repeat is its first update exactly.
    assert report["weights_noise"] == 0.0
    assert variants["accumulate=3+align32"]["chunk_rows"] == [32]
    assert variants["baseline"]["chunk_rows"] != [32]
    base = variants["baseline"]["losses"]
    assert base["imitation_loss"] > 0
    for name, v in variants.items():
        assert len(v["runs"]) == 2 and v["us_per_sample_pass"] > 0, name
        assert v["weights_max_diff"] <= 1e-6, name
        for key, value in v["losses"].items():
            assert value == pytest.approx(base[key], rel=1e-4, abs=1e-7), (name, key)
    phases = report["phases"]["parts"]
    assert list(phases) == [*speed.PHASES, "other"]
    assert sum(p["share"] for p in phases.values()) == pytest.approx(1.0)
    assert all(p["seconds"] >= 0 for name, p in phases.items() if name != "other")
    # One update of one epoch: 2 minibatches of 2 chunks, a gather, a pass and a backward each.
    chunks = 2 * 2
    for name in ("gather", "encode", "trunk", "heads", "backward"):
        assert phases[name]["calls"] == chunks, name
    assert phases["entry and value"]["calls"] == 2 * chunks
    assert 0 < phases["teacher heads"]["calls"] <= chunks
    assert phases["step"]["calls"] == phases["clip"]["calls"] == 2
    assert "Self CPU" in report["trace"]


def test_the_phases_and_variants_leave_nothing_patched() -> None:
    backward, clip = torch.Tensor.backward, torch.nn.utils.clip_grad_norm_
    trainer = Trainer(load_config(SMALL))
    try:
        timer = speed.Phases(trainer.device)
        timer.install(trainer)
        assert torch.Tensor.backward is not backward
        timer.remove()
        config = trainer.config
        with pytest.raises(RuntimeError):
            with speed.variant_of(
                trainer, config, speed.VARIANTS["heuristics, accumulate=4+align32"]
            ):
                assert (trainer.config.accumulate, trainer.config.chunk_align) == (4, 32)
                assert not torch.backends.cudnn.benchmark and not torch.backends.cudnn.allow_tf32
                raise RuntimeError
        assert trainer.config is config and torch.backends.cudnn.allow_tf32
        assert (torch.Tensor.backward, torch.nn.utils.clip_grad_norm_) == (backward, clip)
        for owner in (trainer.policy, trainer.staging, trainer.optimizer):
            assert not {"encode", "trunk", "along", "gather", "step"} & set(vars(owner))
    finally:
        trainer.batch.close()


def test_a_variant_names_only_known_variants() -> None:
    with pytest.raises(ValueError, match="no variant fast"):
        speed.profile(load_config(SMALL), variants=["fast"])
    with pytest.raises(ValueError, match="at least 1"):
        speed.profile(load_config(SMALL), repeats=0)
