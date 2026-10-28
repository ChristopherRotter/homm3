"""The library and the binding agree: the ABI version, the encodings' sizes and versions, the
layouts."""

from __future__ import annotations

import ctypes

import pytest

import h3env
from h3env import _ffi


def test_the_library_speaks_the_bindings_abi() -> None:
    assert _ffi.lib().h3_abi_version() == h3env.ABI_VERSION == 35


def test_a_library_of_another_version_is_refused() -> None:
    with pytest.raises(ImportError, match="C ABI v35, this binding v31"):
        _ffi.check_version(_ffi.lib(), expected=31)


def test_the_versions_a_record_names() -> None:
    # v31 (E06 T2c): observation encoding v21, action encoding v13, the encounter distribution v1;
    # v32 (E06 T7): the system actor's policy's behaviour version (`h3_battle_ai::POLICY_VERSION`);
    # v34: observation encoding v22 (the turn order); v35 (E25 T1a): a world's reward v1.
    versions = _ffi.versions()
    assert versions["system_policy"] > 0
    del versions["system_policy"]
    assert versions == {
        "abi": 35,
        "observation_encoding": 22,
        "action_encoding": 13,
        "encounters": 1,
    }


def test_the_encodings_sizes() -> None:
    actions, spells, targets, observation = _ffi.sizes()
    # Action encoding v13: the named entries sit after the stacks' and before the wall shots.
    assert actions > _ffi.CREATURE_CAST
    assert spells > 0 and targets > 256
    assert observation > 0


def test_the_preview_covers_the_attack_and_shot_entries() -> None:
    # v33 (AI-Q7 (b)): from the first attack entry to the heals, as `layout.py` names them.
    from h3env.train import layout

    assert _ffi.preview() == (layout.ATTACK_BASE, layout.HEAL_BASE - layout.ATTACK_BASE)


def test_the_layouts_match_h3_h() -> None:
    assert ctypes.sizeof(_ffi.H3Action) == h3env.ACTION_DTYPE.itemsize == 12
    for field in ("entry", "spell", "target", "second"):
        assert getattr(_ffi.H3Action, field).offset == h3env.ACTION_DTYPE.fields[field][1]
    assert ctypes.sizeof(_ffi.H3StepResult) == 16
    assert ctypes.sizeof(_ffi.H3BatchConfig) == 32
    assert ctypes.sizeof(_ffi.H3BatchBuffers) == 14 * ctypes.sizeof(ctypes.c_void_p)
    assert ctypes.sizeof(_ffi.H3ScenarioInfo) == 32


def test_episode_seeds_come_from_the_engine() -> None:
    a = h3env.episode_seed(7, 0, 0)
    assert a == h3env.episode_seed(7, 0, 0)
    assert len({a, h3env.episode_seed(7, 1, 0), h3env.episode_seed(7, 0, 1)}) == 3
