"""PyTorch with the batch (E05 T6b): the build `uv.lock` pins, tensors over the arrays the engine
writes, a network's masked choices the batch accepts, and the GPU where there is one. They need the
group `train` (`uv sync --group train`) and are skipped without it."""

from __future__ import annotations

import numpy as np
import pytest
from conftest import scenario

from h3env import BattleBatch, _ffi

torch = pytest.importorskip("torch")


def masked_choice(logits, mask: np.ndarray, generator):
    """One index per row drawn from `logits` among the set entries of `mask` (0/1 bytes, [N, K]);
    0 for a row with none."""
    allowed = torch.from_numpy(mask).bool()
    allowed[~allowed.any(dim=1), 0] = True
    probs = torch.softmax(logits.masked_fill(~allowed, float("-inf")), dim=1)
    return torch.multinomial(probs, 1, generator=generator).squeeze(1).numpy()


def test_the_locked_build() -> None:
    # PyTorch 2.14 with its CUDA 12.6 build, the last for the user's GTX 1070 (ADR-0025 point 11).
    assert torch.__version__.startswith("2.14.")
    assert torch.version.cuda == "12.6"


def test_tensors_share_the_arrays_the_engine_writes() -> None:
    batch = BattleBatch(4, seed=1)
    batch.add_scenario(scenario("full"))
    batch.reset()
    obs = torch.from_numpy(batch.obs)
    mask = torch.from_numpy(batch.entry_mask)
    assert obs.data_ptr() == batch.obs.ctypes.data
    assert mask.data_ptr() == batch.entry_mask.ctypes.data
    before = obs.clone()
    batch.random_actions(np.random.default_rng(1))
    batch.step()
    # The engine wrote the next observations into the same memory: no copy, no new tensor.
    assert not torch.equal(obs, before)


def test_a_networks_masked_choices_apply() -> None:
    # A small network chooses every head from the batch's masks, a cast's heads one call after
    # another, as a policy in training would; `step` raises if an env with a side to move refuses.
    torch.manual_seed(0)
    generator = torch.Generator().manual_seed(0)
    batch = BattleBatch(8, threads=2, seed=3)
    batch.add_scenario(scenario("full"))
    batch.add_scenario(scenario("siege"))
    batch.reset()
    a, s, t = batch.action_count, batch.spell_count, batch.target_count
    net = torch.nn.Linear(batch.observation_len, a + s + 2 * t)
    obs = torch.from_numpy(batch.obs)
    # Bounded logits (the observation is not normalised), the casts preferred where legal so
    # that their heads are chosen too.
    bonus = torch.zeros(a)
    bonus[[_ffi.HERO_CAST, _ffi.CREATURE_CAST]] = 4.0
    casts = steps = 0
    with torch.no_grad():
        for _ in range(200):
            entry, spell, target, second = torch.tanh(net(obs)).split([a, s, t, t], dim=1)
            entry = entry + bonus
            entries = masked_choice(entry, batch.entry_mask, generator).astype(np.uint32)
            spells = np.zeros(batch.envs, np.uint16)
            targets = np.zeros(batch.envs, np.uint16)
            seconds = np.zeros(batch.envs, np.uint16)
            cast = (entries == _ffi.HERO_CAST) | (entries == _ffi.CREATURE_CAST)
            if cast.any():
                casts += int(cast.sum())
                spells = masked_choice(spell, batch.spell_mask(entries), generator)
                spells = spells.astype(np.uint16)
                targets = masked_choice(target, batch.target_mask(entries, spells), generator)
                targets = targets.astype(np.uint16)
                seconds = masked_choice(
                    second, batch.second_mask(entries, spells, targets), generator
                )
            steps += int((batch.to_move >= 0).sum())
            batch.step(entries, spells, targets, seconds)
    assert casts > 0
    assert steps == 200 * batch.envs


@pytest.mark.skipif(not torch.cuda.is_available(), reason="no CUDA device")
def test_the_build_runs_on_this_gpu() -> None:
    # A build without kernels for the device (a Pascal GPU after PyTorch 2.14) fails at the first
    # launch: an elementwise kernel and a cuBLAS product.
    x = torch.arange(1024, device="cuda", dtype=torch.float32)
    assert float((x * 2).sum()) == 1023 * 1024
    ones = torch.ones(64, 64, device="cuda")
    assert torch.equal((ones @ ones).cpu(), torch.full((64, 64), 64.0))
