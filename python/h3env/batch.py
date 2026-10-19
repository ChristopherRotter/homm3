"""The training batch (E05 T6a; C ABI `h3_batch_*`, ADR-0025, `docs/epics/E05-design.md` §1, §5).

`BattleBatch` owns an `H3Batch` handle and numpy arrays the engine writes in place at each call
(zero copy): the binding allocates them once and passes their data pointers. Agents see only the
projected observations of the acting side (ADR-0025 point 12); full-state access is in
`h3env.tools`.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from ctypes import byref, c_uint8, c_uint16, c_uint32

import numpy as np

from . import _ffi
from ._ffi import H3Error

#: Seat names for `add_scenario` (`H3_SEAT_*`).
SEATS = {
    "external": _ffi.SEAT_EXTERNAL,
    "random": _ffi.SEAT_RANDOM,
    "aggressor": _ffi.SEAT_AGGRESSOR,
    "greedy": _ffi.SEAT_GREEDY,
}


def episode_seed(batch_seed: int, env: int, episode: int) -> int:
    """The seed of episode `episode` of env `env`: a single env with it plays the same battle."""
    return _ffi.lib().h3_batch_episode_seed(batch_seed, env, episode)


def sample(mask: np.ndarray, rng: np.random.Generator) -> np.ndarray:
    """One uniformly random legal index per row of `mask` (0/1 bytes, [N, K]); 0 for a row with
    none. One draw per row, from the legal indices' flat positions (a cumulative sum over the
    whole mask costs eight times as much)."""
    n, k = mask.shape
    flat = np.flatnonzero(np.ascontiguousarray(mask, np.uint8).view(np.bool_))
    counts = np.bincount(flat // k, minlength=n)
    chosen = np.cumsum(counts) - counts + (rng.random(n) * counts).astype(np.int64)
    has = counts > 0
    out = np.zeros(n, np.int64)
    out[has] = flat[chosen[has]] % k
    return out


def _seat(seat: int | str) -> int:
    return SEATS[seat] if isinstance(seat, str) else int(seat)


class BattleBatch:
    """N battle environments stepped together (`h3_batch_*`).

    Add scenarios (`add_scenario`) or the encounter distribution (`add_encounters`), then `reset`
    and `step`. After each call these arrays hold its outputs, one row per env; every call rewrites
    them, so copy what you keep:

    - `obs` [N, `observation_len`] float32: the acting side's projected observation;
    - `entry_mask` [N, `action_count`] uint8: its legal entries (never Retreat or Surrender);
    - `to_move` [N] int8: the acting side (0 attacker, 1 defender), -1 for an env that gave up;
    - `reward` [N, 2] float32: this call's reward per side (reward v1 when a battle is decided);
    - `done`, `truncated` [N] uint8: an episode ended in this call (the row shows the next one);
    - `result` [N] int8: `H3_RESULT_*` of the episode that ended, else -1;
    - `final_obs` [N, 2, `observation_len`] float32: both sides' last observations, written only
      for the rows truncated in this call (None with `final_obs=False`);
    - `episode`, `scenario` [N] uint32, `episode_seed` [N] uint64: the current episode, its
      source's index (in the order added) and its seed;
    - `status` [N] int32: `H3_OK`, or why the env refused this call's action (it is unchanged);
    - `encounter` [N] int8: the episode's encounter kind (`H3_ENCOUNTER_*`, -1 for a scenario).

    A handle must not be used from two threads at once; the engine's own threads (`threads`) run
    inside each call, without the GIL.
    """

    def __init__(
        self,
        envs: int,
        *,
        threads: int = 0,
        seed: int = 0,
        max_steps: int = 0,
        currency: str = "ai_value",
        mana_price: int = 10,
        final_obs: bool = True,
    ) -> None:
        h = _ffi.lib()
        self._lib = h
        self._handle = None
        self.action_count, self.spell_count, self.target_count, self.observation_len = _ffi.sizes()
        if currency not in _ffi.CURRENCIES:
            raise ValueError(f"unknown currency {currency!r} (one of {sorted(_ffi.CURRENCIES)})")
        config = _ffi.H3BatchConfig(
            envs=envs,
            threads=threads,
            seed=seed,
            max_steps=max_steps,
            reward=_ffi.H3RewardConfig(_ffi.CURRENCIES[currency], mana_price),
        )
        error = _ffi.error_buffer()
        handle = h.h3_batch_create(byref(config), error, len(error))
        if not handle:
            raise H3Error(_ffi.ERR_ARGUMENT, _ffi.message(error))
        self._handle = handle
        self.envs = envs
        self.seed = seed
        n, obs = envs, self.observation_len
        self.obs = np.zeros((n, obs), np.float32)
        self.entry_mask = np.zeros((n, self.action_count), np.uint8)
        self.to_move = np.full(n, -1, np.int8)
        self.reward = np.zeros((n, 2), np.float32)
        self.done = np.zeros(n, np.uint8)
        self.truncated = np.zeros(n, np.uint8)
        self.result = np.full(n, _ffi.RESULT_NONE, np.int8)
        self.final_obs = np.zeros((n, 2, obs), np.float32) if final_obs else None
        self.episode = np.zeros(n, np.uint32)
        self.scenario = np.zeros(n, np.uint32)
        self.episode_seed = np.zeros(n, np.uint64)
        self.status = np.zeros(n, np.int32)
        self.encounter = np.full(n, _ffi.ENCOUNTER_NONE, np.int8)
        self._buffers = _ffi.H3BatchBuffers(
            *(
                None if a is None else a.ctypes.data
                for a in (
                    self.obs,
                    self.entry_mask,
                    self.to_move,
                    self.reward,
                    self.done,
                    self.truncated,
                    self.result,
                    self.final_obs,
                    self.episode,
                    self.scenario,
                    self.episode_seed,
                    self.status,
                    self.encounter,
                )
            )
        )
        self.actions = np.zeros(n, _ffi.ACTION_DTYPE)
        self._spell_mask = np.zeros((n, self.spell_count), np.uint8)
        self._target_mask = np.zeros((n, self.target_count), np.uint8)
        self._second_mask = np.zeros((n, self.target_count), np.uint8)

    # The handle's lifetime.

    def close(self) -> None:
        """Destroys the handle (its threads end); the arrays stay readable."""
        if getattr(self, "_handle", None):
            self._lib.h3_batch_destroy(self._handle)
            self._handle = None

    def __enter__(self) -> BattleBatch:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        self.close()

    def _live(self):
        if not self._handle:
            raise H3Error(_ffi.ERR_NULL, "the batch is closed")
        return self._handle

    # Episode sources.

    def add_scenario(
        self,
        toml: str | bytes,
        weight: int = 1,
        attacker: int | str = "external",
        defender: int | str = "external",
    ) -> None:
        """Adds a scenario's text (the format of `packs/synthetic/scenarios/`) with its seats.

        Seats: `"external"` (the caller acts), `"aggressor"` or `"greedy"` (the baseline bots inside
        the call, v30) or `"random"` (uniformly random legal commands inside
        the call; tests); at least one external. Every side plays as a human player's whatever the
        scenario says (ADR-0025 point 6).
        """
        text = toml.encode("utf-8") if isinstance(toml, str) else toml
        seats = _ffi.H3Seats(_seat(attacker), _seat(defender))
        error = _ffi.error_buffer()
        code = self._lib.h3_batch_add_scenario(
            self._live(), text, len(text), weight, seats, error, len(error)
        )
        _ffi.check(code, error)

    def add_encounters(
        self,
        weight: int = 1,
        kinds: Sequence[int] | Mapping[str, int] | None = None,
        attacker: int | str | None = None,
        defender: int | str | None = None,
    ) -> None:
        """Adds the encounter distribution v1 (on the original packs) with `weight`.

        `kinds`: None for the distribution's weights (the neutral kinds 0 until E23), six weights in
        `ENCOUNTER_KINDS` order, or a mapping from those names to weights (the others 0).
        `attacker`, `defender`: None for each encounter's own seats, or both given (as for
        `add_scenario`) to play every episode with them, e.g. against `"random"`.
        """
        if (attacker is None) != (defender is None):
            raise ValueError("give both seats or neither")
        seats = None if attacker is None else _ffi.H3Seats(_seat(attacker), _seat(defender))
        error = _ffi.error_buffer()
        if kinds is None:
            pointer = None
        else:
            if isinstance(kinds, Mapping):
                unknown = set(kinds) - set(_ffi.ENCOUNTER_KINDS)
                if unknown:
                    raise ValueError(f"unknown encounter kinds {sorted(unknown)}")
                kinds = [kinds.get(name, 0) for name in _ffi.ENCOUNTER_KINDS]
            if len(kinds) != len(_ffi.ENCOUNTER_KINDS):
                raise ValueError(f"six kind weights, not {len(kinds)}")
            pointer = (c_uint32 * 6)(*kinds)
        seats_pointer = None if seats is None else byref(seats)
        code = self._lib.h3_batch_add_encounters(
            self._live(), weight, pointer, seats_pointer, error, len(error)
        )
        _ffi.check(code, error)

    # Calls.

    def reset(self) -> None:
        """Starts every env's first episode (a later reset replays the same episodes)."""
        _ffi.check(self._lib.h3_batch_reset(self._live(), byref(self._buffers)))

    def spell_mask(self, entries: np.ndarray) -> np.ndarray:
        """[N, `spell_count`]: each env's legal spells after choosing `entries` ("0 only" for an
        entry that is no cast). The array is rewritten by the next spell-mask call."""
        e = np.ascontiguousarray(entries, np.uint32)
        out = self._spell_mask
        code = self._lib.h3_batch_spell_mask(
            self._live(), _ffi.ptr(e, c_uint32), _ffi.ptr(out, c_uint8), out.size
        )
        _ffi.check(code)
        return out

    def target_mask(self, entries: np.ndarray, spells: np.ndarray) -> np.ndarray:
        """[N, `target_count`]: each env's legal targets after choosing `entries` and `spells`."""
        e = np.ascontiguousarray(entries, np.uint32)
        s = np.ascontiguousarray(spells, np.uint16)
        out = self._target_mask
        code = self._lib.h3_batch_target_mask(
            self._live(),
            _ffi.ptr(e, c_uint32),
            _ffi.ptr(s, c_uint16),
            _ffi.ptr(out, c_uint8),
            out.size,
        )
        _ffi.check(code)
        return out

    def second_mask(
        self, entries: np.ndarray, spells: np.ndarray, targets: np.ndarray
    ) -> np.ndarray:
        """[N, `target_count`]: each env's legal second targets (Teleport's destination)."""
        e = np.ascontiguousarray(entries, np.uint32)
        s = np.ascontiguousarray(spells, np.uint16)
        t = np.ascontiguousarray(targets, np.uint16)
        out = self._second_mask
        code = self._lib.h3_batch_second_mask(
            self._live(),
            _ffi.ptr(e, c_uint32),
            _ffi.ptr(s, c_uint16),
            _ffi.ptr(t, c_uint16),
            _ffi.ptr(out, c_uint8),
            out.size,
        )
        _ffi.check(code)
        return out

    def step(
        self,
        entry: np.ndarray | None = None,
        spell: np.ndarray | None = None,
        target: np.ndarray | None = None,
        second: np.ndarray | None = None,
        *,
        check: bool = True,
    ) -> int:
        """Plays each env's action for its acting side, then its internal seats; an env whose
        episode ends starts its next one in the same call.

        The heads given are written into `actions` (a head left out stays as it is there; fill
        `actions` directly to skip the copy). Returns `H3_OK` or `H3_ERR_REJECTED` (an env refused
        its action and is unchanged: see `status`). With `check`, a refusal by an env that has a
        side to move raises; an env that gave up refuses every action silently.
        """
        a = self.actions
        for name, values in (
            ("entry", entry),
            ("spell", spell),
            ("target", target),
            ("second", second),
        ):
            if values is not None:
                a[name] = values
        code = self._lib.h3_batch_step(self._live(), a.ctypes.data, byref(self._buffers))
        if code == _ffi.ERR_REJECTED:
            if check:
                refused = np.nonzero((self.status != _ffi.OK) & (self.to_move >= 0))[0]
                if len(refused):
                    i = int(refused[0])
                    name = _ffi.ERROR_NAMES.get(int(self.status[i]), str(self.status[i]))
                    raise H3Error(
                        code, f"{len(refused)} env(s) refused their action; env {i}: {name}"
                    )
            return code
        return _ffi.check(code)

    def random_actions(self, rng: np.random.Generator) -> np.ndarray:
        """Fills `actions` with a uniformly random legal action per env from the batch's masks: the
        entry, then, if any env chose a cast, its heads one call after another (as `h3 ai bench`).
        Returns `actions`."""
        a = self.actions
        entries = sample(self.entry_mask, rng)
        a["entry"] = entries
        a["spell"] = 0
        a["target"] = 0
        a["second"] = 0
        if np.any((entries == _ffi.HERO_CAST) | (entries == _ffi.CREATURE_CAST)):
            e = a["entry"].copy()
            spells = sample(self.spell_mask(e), rng).astype(np.uint16)
            targets = sample(self.target_mask(e, spells), rng).astype(np.uint16)
            a["spell"] = spells
            a["target"] = targets
            a["second"] = sample(self.second_mask(e, spells, targets), rng)
        return a
