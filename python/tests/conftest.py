"""Shared helpers of the binding's tests (E05 T6a). They need the release library
(`cargo build --release -p h3-ffi`) or `$H3_LIBRARY`."""

from __future__ import annotations

from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
SCENARIOS = REPO / "packs" / "synthetic" / "scenarios"


def scenario(name: str, human: bool = False) -> str:
    """The text of `packs/synthetic/scenarios/<name>.toml`; with `human`, both sides are human
    players', as the batch plays every scenario (ADR-0025 point 6), so that a single env from the
    text plays the batch's battle."""
    text = (SCENARIOS / f"{name}.toml").read_text(encoding="utf-8")
    if human:
        text = text.replace("[scenario]\n", '[scenario]\nhuman = ["attacker", "defender"]\n', 1)
    return text


def bits(values: np.ndarray) -> np.ndarray:
    """Floats as their bit patterns, for exact comparisons."""
    return np.ascontiguousarray(values, np.float32).view(np.uint32)
