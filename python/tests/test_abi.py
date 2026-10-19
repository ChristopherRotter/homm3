"""The library and the binding agree: the ABI version, the encodings' sizes, the layouts."""

from __future__ import annotations

import ctypes

import pytest

import h3env
from h3env import _ffi


def test_the_library_speaks_the_bindings_abi() -> None:
    assert _ffi.lib().h3_abi_version() == h3env.ABI_VERSION == 28


def test_a_library_of_another_version_is_refused() -> None:
    with pytest.raises(ImportError, match="C ABI v28, this binding v27"):
        _ffi.check_version(_ffi.lib(), expected=27)


def test_the_encodings_sizes() -> None:
    actions, spells, targets, observation = _ffi.sizes()
    # Action encoding v13: the named entries sit after the stacks' and before the wall shots.
    assert actions > _ffi.CREATURE_CAST
    assert spells > 0 and targets > 256
    assert observation > 0


def test_the_layouts_match_h3_h() -> None:
    assert ctypes.sizeof(_ffi.H3Action) == h3env.ACTION_DTYPE.itemsize == 12
    for field in ("entry", "spell", "target", "second"):
        assert getattr(_ffi.H3Action, field).offset == h3env.ACTION_DTYPE.fields[field][1]
    assert ctypes.sizeof(_ffi.H3StepResult) == 16
    assert ctypes.sizeof(_ffi.H3BatchConfig) == 32
    assert ctypes.sizeof(_ffi.H3BatchBuffers) == 13 * ctypes.sizeof(ctypes.c_void_p)


def test_episode_seeds_come_from_the_engine() -> None:
    a = h3env.episode_seed(7, 0, 0)
    assert a == h3env.episode_seed(7, 0, 0)
    assert len({a, h3env.episode_seed(7, 1, 0), h3env.episode_seed(7, 0, 1)}) == 3
