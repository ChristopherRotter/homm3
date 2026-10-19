"""h3env: the h3 engine's battle environment from Python (E05 T6a, ADR-0011, ADR-0025).

A thin ctypes layer over the engine's C ABI (`crates/h3-ffi/include/h3.h`): `BattleBatch` for
training (N envs per call, numpy arrays written in place), `SeatBook` for each seat's transitions
in self-play, and `h3env.tools.Env` (single envs, fork, state hash: trusted tooling only).
The library is `$H3_LIBRARY`, else the repository's `target/release` build of `h3-ffi`.
"""

from ._ffi import (
    ABI_VERSION,
    ACTION_DTYPE,
    ENCOUNTER_KINDS,
    H3Error,
)
from .batch import BattleBatch, episode_seed, sample
from .seats import Closed, SeatBook

__all__ = [
    "ABI_VERSION",
    "ACTION_DTYPE",
    "ENCOUNTER_KINDS",
    "BattleBatch",
    "Closed",
    "H3Error",
    "SeatBook",
    "episode_seed",
    "sample",
]
