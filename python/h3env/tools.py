"""Single environments: trusted tooling (E05 T6a; `docs/epics/E05-log.md` §6, ADR-0011).

`Env` wraps the single-env C functions (`h3_env_*`). Unlike the batch it reads the full state
(`state_hash`, `fork`) and keeps the escapes (Retreat, Surrender) and the scenario's own `human`
flags: for tests, tools and training infrastructure, never for an agent's input and never to be
forwarded to a client. The exception is a *world* (`BattleBatch.determinize`, C ABI v34, ADR-0025
point 12a): an env whose battle holds nothing its side may not know, a search agent's to fork,
step and observe.
"""

from __future__ import annotations

from ctypes import byref, c_float, c_uint8

import numpy as np

from . import _ffi
from ._ffi import H3Error


class Env:
    """One battle environment (`h3_env_*`). Sides: 0 attacker, 1 defender."""

    def __init__(self, handle) -> None:
        if not handle:
            raise H3Error(_ffi.ERR_NULL, "no env")
        self._lib = _ffi.lib()
        self._handle = handle
        self.action_count, self.spell_count, self.target_count, self.observation_len = _ffi.sizes()

    @classmethod
    def synthetic(cls, scenario: int, seed: int, max_steps: int = 0) -> Env:
        """A built-in synthetic scenario: 0 duel, 1 skirmish, 2 full armies."""
        handle = _ffi.lib().h3_env_create_synthetic(scenario, seed, max_steps)
        if not handle:
            raise H3Error(_ffi.ERR_ARGUMENT, f"no synthetic scenario {scenario}")
        return cls(handle)

    @classmethod
    def scenario(cls, toml: str | bytes, seed: int, max_steps: int = 0) -> Env:
        """Any battle from a scenario's text (its `difficulty` and `human` apply as written)."""
        text = toml.encode("utf-8") if isinstance(toml, str) else toml
        error = _ffi.error_buffer()
        handle = _ffi.lib().h3_env_create_scenario(
            text, len(text), seed, max_steps, error, len(error)
        )
        if not handle:
            raise H3Error(_ffi.ERR_ARGUMENT, _ffi.message(error))
        return cls(handle)

    def close(self) -> None:
        if getattr(self, "_handle", None):
            self._lib.h3_env_destroy(self._handle)
            self._handle = None

    def __enter__(self) -> Env:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        self.close()

    def fork(self) -> Env:
        """An independent copy of the env's full state."""
        return Env(self._lib.h3_env_fork(self._handle))

    def reset(self, seed: int) -> None:
        _ffi.check(self._lib.h3_env_reset(self._handle, seed))

    def reseed(self, seed: int) -> None:
        """Re-seeds the battle's stream (C ABI v34): a world's simulation draws its own rolls."""
        _ffi.check(self._lib.h3_env_reseed(self._handle, seed))

    def to_move(self) -> int:
        """The side to act, -1 once the episode is over."""
        return self._lib.h3_env_to_move(self._handle)

    def legal_mask(self, actor: int) -> np.ndarray:
        out = np.zeros(self.action_count, np.uint8)
        code = self._lib.h3_env_legal_mask(self._handle, actor, _ffi.ptr(out, c_uint8), out.size)
        _ffi.check(code)
        return out

    def spell_mask(self, actor: int, entry: int) -> np.ndarray:
        out = np.zeros(self.spell_count, np.uint8)
        code = self._lib.h3_env_spell_mask(
            self._handle, actor, entry, _ffi.ptr(out, c_uint8), out.size
        )
        _ffi.check(code)
        return out

    def target_mask(self, actor: int, entry: int, spell: int) -> np.ndarray:
        out = np.zeros(self.target_count, np.uint8)
        code = self._lib.h3_env_target_mask(
            self._handle, actor, entry, spell, _ffi.ptr(out, c_uint8), out.size
        )
        _ffi.check(code)
        return out

    def second_mask(self, actor: int, entry: int, spell: int, target: int) -> np.ndarray:
        out = np.zeros(self.target_count, np.uint8)
        code = self._lib.h3_env_second_mask(
            self._handle, actor, entry, spell, target, _ffi.ptr(out, c_uint8), out.size
        )
        _ffi.check(code)
        return out

    def observe(self, viewer: int) -> np.ndarray:
        """`viewer`'s projected observation (the flat encoding)."""
        out = np.zeros(self.observation_len, np.float32)
        code = self._lib.h3_env_observe(self._handle, viewer, _ffi.ptr(out, c_float), out.size)
        _ffi.check(code)
        return out

    def step(
        self, actor: int, entry: int, spell: int = 0, target: int = 0, second: int = 0
    ) -> _ffi.H3StepResult:
        """Plays an action for `actor`; raises if it is refused (the env is then unchanged)."""
        result = _ffi.H3StepResult()
        action = _ffi.H3Action(entry, spell, target, second)
        _ffi.check(self._lib.h3_env_step(self._handle, actor, action, byref(result)))
        return result

    def state_hash(self) -> int:
        """The canonical hash of the full state (trusted: never an agent's input)."""
        return self._lib.h3_env_state_hash(self._handle)
