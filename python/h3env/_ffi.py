"""The C ABI through ctypes (E05 T6a; `crates/h3-ffi/include/h3.h`, ADR-0011, ADR-0025).

Loads the engine's shared library once, declares every function's signature and refuses a library
whose `h3_abi_version()` is not the binding's. `ctypes.CDLL` releases the GIL during each call.
The layouts below mirror `h3.h` field for field; a change there is a new ABI version and a change
here.
"""

from __future__ import annotations

import ctypes
import os
import sys
from ctypes import (
    POINTER,
    c_char,
    c_char_p,
    c_float,
    c_int32,
    c_size_t,
    c_uint8,
    c_uint16,
    c_uint32,
    c_uint64,
    c_void_p,
)
from pathlib import Path

import numpy as np

#: The C ABI version this binding speaks (`h3_abi_version()`).
ABI_VERSION = 35

#: The most envs a batch holds (`H3_BATCH_MAX_ENVS`, v31): the engine refuses more.
BATCH_MAX_ENVS = 1 << 16

OK = 0
ERR_NULL = -1
ERR_ARGUMENT = -2
ERR_INVALID_ACTION = -3
ERR_REJECTED = -4
ERR_EPISODE_OVER = -5
ERR_NOT_READY = -6
ERR_PANIC = -99

ERROR_NAMES = {
    ERR_NULL: "H3_ERR_NULL",
    ERR_ARGUMENT: "H3_ERR_ARGUMENT",
    ERR_INVALID_ACTION: "H3_ERR_INVALID_ACTION",
    ERR_REJECTED: "H3_ERR_REJECTED",
    ERR_EPISODE_OVER: "H3_ERR_EPISODE_OVER",
    ERR_NOT_READY: "H3_ERR_NOT_READY",
    ERR_PANIC: "H3_ERR_PANIC",
}

RESULT_NONE = -1
RESULT_ATTACKER = 0
RESULT_DEFENDER = 1
RESULT_DRAW = 2
RESULT_ATTACKER_RETREATED = 3
RESULT_DEFENDER_RETREATED = 4
RESULT_ATTACKER_SURRENDERED = 5
RESULT_DEFENDER_SURRENDERED = 6

SEAT_EXTERNAL = 0
SEAT_RANDOM = 1
#: The baseline bots inside the call (v30, E05 T7; `h3_sim::bots`).
SEAT_AGGRESSOR = 2
SEAT_GREEDY = 3
#: The system actor inside the call, the original's computer (v32, E06 T7); a neutral kind's
#: defender in the encounter distribution.
SEAT_SYSTEM = 4

ENCOUNTER_NONE = -1
ENCOUNTER_WANDERING = 0
ENCOUNTER_GUARDS = 1
ENCOUNTER_HERO = 2
ENCOUNTER_SIEGE_HERO = 3
ENCOUNTER_SIEGE_GARRISON = 4
ENCOUNTER_SIEGE_NEUTRAL = 5
#: The kinds' names in `H3_ENCOUNTER_*` order (the weights of `h3_batch_add_encounters`).
ENCOUNTER_KINDS = ("wandering", "guards", "hero", "siege_hero", "siege_garrison", "siege_neutral")

CURRENCY_AI_VALUE = 0
CURRENCY_GOLD = 1
CURRENCIES = {"ai_value": CURRENCY_AI_VALUE, "gold": CURRENCY_GOLD}

#: Entries of action encoding v13 (`h3_sim::action`) the binding names: the batch never allows
#: Retreat and Surrender (ADR-0025 point 7); a cast's spell and targets come from the head calls.
RETREAT = 1392
SURRENDER = 1393
HERO_CAST = 1394
CREATURE_CAST = 1395

#: Room for an error message of the create and add calls.
ERROR_LEN = 1024


class H3Error(RuntimeError):
    """A call the engine refused: its code (`H3_ERR_*`) and the message of a call that writes
    one."""

    def __init__(self, code: int, message: str = "") -> None:
        self.code = code
        name = ERROR_NAMES.get(code, str(code))
        super().__init__(f"{name}: {message}" if message else name)


class H3Action(ctypes.Structure):
    _fields_ = [
        ("entry", c_uint32),
        ("spell", c_uint16),
        ("target", c_uint16),
        ("second", c_uint16),
    ]


#: `H3Action` as a numpy record: an array of it is passed to `h3_batch_step` as is.
ACTION_DTYPE = np.dtype(
    [("entry", "<u4"), ("spell", "<u2"), ("target", "<u2"), ("second", "<u2")], align=True
)


class H3StepResult(ctypes.Structure):
    _fields_ = [
        ("done", c_uint8),
        ("truncated", c_uint8),
        ("result", c_int32),
        ("reward_attacker", c_float),
        ("reward_defender", c_float),
    ]


class H3RewardConfig(ctypes.Structure):
    _fields_ = [("currency", c_uint32), ("mana_price", c_uint32)]


class H3BatchConfig(ctypes.Structure):
    _fields_ = [
        ("envs", c_uint32),
        ("threads", c_uint32),
        ("seed", c_uint64),
        ("max_steps", c_uint32),
        ("reward", H3RewardConfig),
    ]


class H3Seats(ctypes.Structure):
    _fields_ = [("attacker", c_int32), ("defender", c_int32)]


class H3ScenarioInfo(ctypes.Structure):
    """An episode source entry's engine identity (v31)."""

    _fields_ = [
        ("ruleset_hash", c_uint64),
        ("battle_ruleset_hash", c_uint64),
        ("adventure_ruleset_hash", c_uint64),
        ("weight", c_uint32),
        ("encounters", c_uint32),
    ]


class H3BatchBuffers(ctypes.Structure):
    _fields_ = [
        ("obs", c_void_p),
        ("entry_mask", c_void_p),
        ("to_move", c_void_p),
        ("reward", c_void_p),
        ("done", c_void_p),
        ("truncated", c_void_p),
        ("result", c_void_p),
        ("final_obs", c_void_p),
        ("episode", c_void_p),
        ("scenario", c_void_p),
        ("episode_seed", c_void_p),
        ("status", c_void_p),
        ("encounter", c_void_p),
        ("preview", c_void_p),
    ]


_env = c_void_p
_batch = c_void_p
_u8p = POINTER(c_uint8)
_u16p = POINTER(c_uint16)
_u32p = POINTER(c_uint32)
_f32p = POINTER(c_float)

_SIGNATURES = {
    "h3_abi_version": (c_uint32, []),
    "h3_action_count": (c_uint32, []),
    "h3_spell_count": (c_uint32, []),
    "h3_target_count": (c_uint32, []),
    "h3_observation_len": (c_uint32, []),
    "h3_preview_base": (c_uint32, []),
    "h3_preview_len": (c_uint32, []),
    "h3_observation_encoding_version": (c_uint32, []),
    "h3_action_encoding_version": (c_uint32, []),
    "h3_encounters_version": (c_uint32, []),
    "h3_system_policy_version": (c_uint32, []),
    "h3_env_create_synthetic": (_env, [c_uint32, c_uint64, c_uint32]),
    "h3_env_create_scenario": (
        _env,
        [c_char_p, c_size_t, c_uint64, c_uint32, POINTER(c_char), c_size_t],
    ),
    "h3_env_destroy": (None, [_env]),
    "h3_env_fork": (_env, [_env]),
    "h3_env_reset": (c_int32, [_env, c_uint64]),
    "h3_env_reseed": (c_int32, [_env, c_uint64]),
    "h3_env_to_move": (c_int32, [_env]),
    "h3_env_legal_mask": (c_int32, [_env, c_int32, _u8p, c_size_t]),
    "h3_env_spell_mask": (c_int32, [_env, c_int32, c_uint32, _u8p, c_size_t]),
    "h3_env_target_mask": (c_int32, [_env, c_int32, c_uint32, c_uint16, _u8p, c_size_t]),
    "h3_env_second_mask": (
        c_int32,
        [_env, c_int32, c_uint32, c_uint16, c_uint16, _u8p, c_size_t],
    ),
    "h3_env_observe": (c_int32, [_env, c_int32, _f32p, c_size_t]),
    "h3_env_step": (c_int32, [_env, c_int32, H3Action, POINTER(H3StepResult)]),
    "h3_env_state_hash": (c_uint64, [_env]),
    "h3_batch_create": (_batch, [POINTER(H3BatchConfig), POINTER(c_char), c_size_t]),
    "h3_batch_destroy": (None, [_batch]),
    "h3_batch_add_scenario": (
        c_int32,
        [_batch, c_char_p, c_size_t, c_uint32, H3Seats, POINTER(c_char), c_size_t],
    ),
    "h3_batch_add_encounters": (
        c_int32,
        [_batch, c_uint32, _u32p, POINTER(H3Seats), POINTER(c_char), c_size_t],
    ),
    "h3_batch_reset": (c_int32, [_batch, POINTER(H3BatchBuffers)]),
    "h3_batch_spell_mask": (c_int32, [_batch, _u32p, _u8p, c_size_t]),
    "h3_batch_target_mask": (c_int32, [_batch, _u32p, _u16p, _u8p, c_size_t]),
    "h3_batch_second_mask": (c_int32, [_batch, _u32p, _u16p, _u16p, _u8p, c_size_t]),
    "h3_batch_step": (c_int32, [_batch, c_void_p, POINTER(H3BatchBuffers)]),
    "h3_batch_episode_seed": (c_uint64, [c_uint64, c_uint32, c_uint32]),
    "h3_batch_scenario_count": (c_uint32, [_batch]),
    "h3_batch_scenario_info": (c_int32, [_batch, c_uint32, POINTER(H3ScenarioInfo)]),
    "h3_batch_determinize": (_env, [_batch, c_uint32, c_int32, c_uint64]),
    "h3_batch_true_world": (_env, [_batch, c_uint32, c_uint64]),
}


def library_name() -> str:
    """The file name of the engine's library on this platform (`h3-ffi`'s cdylib)."""
    if sys.platform == "win32":
        return "h3_ffi.dll"
    if sys.platform == "darwin":
        return "libh3_ffi.dylib"
    return "libh3_ffi.so"


def default_path() -> Path:
    """`$H3_LIBRARY`, else the release build of the repository this package sits in."""
    explicit = os.environ.get("H3_LIBRARY")
    if explicit:
        return Path(explicit)
    return Path(__file__).resolve().parents[2] / "target" / "release" / library_name()


def declare(lib: ctypes.CDLL) -> ctypes.CDLL:
    """Sets every function's signature on `lib`."""
    for name, (restype, argtypes) in _SIGNATURES.items():
        function = getattr(lib, name)
        function.restype = restype
        function.argtypes = argtypes
    return lib


def check_version(lib: ctypes.CDLL, expected: int = ABI_VERSION) -> None:
    """Refuses a library whose ABI version is not `expected`."""
    found = lib.h3_abi_version()
    if found != expected:
        raise ImportError(
            f"the h3 library speaks C ABI v{found}, this binding v{expected}: "
            "rebuild the library (cargo build --release -p h3-ffi) or update the binding"
        )


_LIB: ctypes.CDLL | None = None


def lib() -> ctypes.CDLL:
    """The engine's library, loaded and checked on first use."""
    global _LIB
    if _LIB is None:
        path = default_path()
        if not path.exists():
            raise ImportError(
                f"no h3 library at {path}: build it with `cargo build --release -p h3-ffi` "
                "or set H3_LIBRARY"
            )
        loaded = declare(ctypes.CDLL(str(path)))
        check_version(loaded)
        _LIB = loaded
    return _LIB


def error_buffer() -> ctypes.Array:
    """A buffer for the message of a create or add call."""
    return ctypes.create_string_buffer(ERROR_LEN)


def message(buffer: ctypes.Array) -> str:
    """The message a call wrote into `buffer` (empty if none)."""
    return buffer.value.decode("utf-8", errors="replace")


def check(code: int, error: ctypes.Array | None = None) -> int:
    """`code` if it is `H3_OK`, else raises it with the call's message."""
    if code != OK:
        raise H3Error(code, message(error) if error is not None else "")
    return code


def ptr(array: np.ndarray, ctype):
    """`array`'s data as a ctypes pointer to `ctype` (the array must stay alive meanwhile)."""
    return array.ctypes.data_as(POINTER(ctype))


def versions() -> dict[str, int]:
    """The versions a run's record names (v31): the C ABI's, the observation and action
    encodings' and the encounter distribution's; since v32 (E06 T7) the system actor's policy's
    (ADR-0027 point 12)."""
    h = lib()
    return {
        "abi": h.h3_abi_version(),
        "observation_encoding": h.h3_observation_encoding_version(),
        "action_encoding": h.h3_action_encoding_version(),
        "encounters": h.h3_encounters_version(),
        "system_policy": h.h3_system_policy_version(),
    }


def preview() -> tuple[int, int]:
    """The batch's kill preview (v33): its first entry and its entries (each holding the least
    and the greatest kills)."""
    h = lib()
    return h.h3_preview_base(), h.h3_preview_len() // 2


def sizes() -> tuple[int, int, int, int]:
    """The encodings' sizes: action entries, spells, targets, observation length."""
    h = lib()
    return (
        h.h3_action_count(),
        h.h3_spell_count(),
        h.h3_target_count(),
        h.h3_observation_len(),
    )
