"""The GPU's memory cap (E25 T28, `h3env.train.gpu`; the user, 2026-11-10: a run never fills more
than 90 % of the card). The sandbox has no GPU: the CUDA calls are stubbed, so the tests check
what the cap asks of torch; the user's runs show it at work (each update's `gpu_*` fields)."""

from __future__ import annotations

import pytest

torch = pytest.importorskip("torch")

from h3env.train import gpu  # noqa: E402

MB = 2**20


def test_the_cap_keeps_the_card_at_90_percent_with_what_it_held_outside_the_run() -> None:
    total = 8_192 * MB
    # Nothing reserved yet: the 512 MB outside the run (its context, the desktop) count.
    assert gpu.cap_bytes(total, total - 512 * MB, 0) == int(0.9 * total) - 512 * MB
    # Called again later: the run's own reserved memory is not outside it.
    assert gpu.cap_bytes(total, total - 5_632 * MB, 5_120 * MB) == int(0.9 * total) - 512 * MB
    # A card already past the fraction leaves nothing.
    assert gpu.cap_bytes(total, 256 * MB, 0) == 0
    assert gpu.cap_bytes(total, total, 0, fraction=0.5) == total // 2


def test_another_device_is_left_alone() -> None:
    assert gpu.cap(torch.device("cpu")) is None
    assert gpu.usage(torch.device("cpu")) == {}


@pytest.fixture
def card(monkeypatch):
    """A stubbed 8 GB card: its free bytes, the allocator's reserved and peak bytes, and the calls
    made to torch."""
    state = {
        "free": 7_680 * MB,
        "reserved": 0,
        "peak": 0,
        "fraction": None,
        "index": None,
        "resets": 0,
    }
    cuda = torch.cuda
    monkeypatch.setattr(cuda, "current_device", lambda: 0)
    monkeypatch.setattr(cuda, "mem_get_info", lambda device=None: (state["free"], 8_192 * MB))
    monkeypatch.setattr(cuda, "memory_reserved", lambda device=None: state["reserved"])
    monkeypatch.setattr(cuda, "max_memory_reserved", lambda device=None: state["peak"])

    def fraction(value, device=None):
        # torch's own refuses a device without an index (a plain "cuda", which `ppo.device_of`
        # gives): E25 T31, the first run of the cap on a card.
        if not isinstance(device, int):
            raise ValueError(f"Expected a torch.device with a specified index, got {device}")
        state["fraction"], state["index"] = value, device

    def reset(device=None):
        state["resets"] += 1

    monkeypatch.setattr(cuda, "set_per_process_memory_fraction", fraction)
    monkeypatch.setattr(cuda, "reset_peak_memory_stats", reset)
    return state


def test_the_cap_sets_the_allocators_fraction_and_refuses_a_full_card(card) -> None:
    m = gpu.cap(torch.device("cuda"))
    assert m == {"total_mb": 8_192.0, "outside_mb": 512.0, "cap_mb": pytest.approx(6_860.8, 1e-3)}
    assert card["fraction"] == pytest.approx(0.9 - 512 / 8_192, abs=1e-6)
    assert card["index"] == 0
    gpu.cap(torch.device("cuda", 1))
    assert card["index"] == 1
    card["free"] = 512 * MB
    with pytest.raises(gpu.GpuFull, match="7680 of its 8192 MB"):
        gpu.cap(torch.device("cuda"))


def test_the_usage_reports_the_reserved_memory_its_peak_and_the_card(card) -> None:
    card.update(free=2_048 * MB, reserved=5_120 * MB, peak=5_632 * MB)
    assert gpu.usage(torch.device("cuda")) == {
        "gpu_reserved_mb": 5_120.0,
        "gpu_peak_mb": 5_632.0,
        "gpu_card": 0.75,
    }
    # Each call starts the next peak.
    assert card["resets"] == 1
