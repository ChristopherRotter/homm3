"""The attack positions' port (E25 T25, `h3env.train.geometry`): `h3_battle::geometry`'s tests of
the neighbours, the footprints and the attack positions (BAT-GEO-001, BAT-GEO-002) as asserts,
the table and the observation's read-out, and real battles: every legal melee entry of action
encoding v13 names a hex its attacker may attack from, and stands on after the attack. They need
the group `train`."""

from __future__ import annotations

from collections import Counter

import numpy as np
import pytest
from conftest import scenario

from h3env import sample
from h3env.tools import Env
from h3env.train import layout as L
from h3env.train.geometry import (
    ATTACKER,
    BOTTOM_LEFT,
    BOTTOM_RIGHT,
    DEFENDER,
    LEFT,
    RIGHT,
    TOP_LEFT,
    TOP_RIGHT,
    Field,
    Footprint,
    attack_hexes,
    attack_table,
)

torch = pytest.importorskip("torch")

FIELD = Field(17, 11)
SIDES = (ATTACKER, DEFENDER)
DIRECTIONS = (RIGHT, BOTTOM_RIGHT, BOTTOM_LEFT, LEFT, TOP_LEFT, TOP_RIGHT)


def at(x: int, y: int) -> int:
    hex_ = FIELD.hex(x, y)
    assert hex_ is not None
    return hex_


def distance(a: int, b: int) -> int:
    """`Field::distance` (BAT-GEO-001): axial coordinates with `X = x + floor(y / 2)`."""
    (ax, ay), (bx, by) = FIELD.coords(a), FIELD.coords(b)
    dx, dy = (bx + by // 2) - (ax + ay // 2), by - ay
    return max(abs(dx), abs(dy)) if (dx >= 0) == (dy >= 0) else abs(dx) + abs(dy)


def touches(a: Footprint, b: Footprint) -> bool:
    """`Field::touches`: the smallest distance between their hexes is 1 (BAT-ATK-001)."""
    return min(distance(x, y) for x in a.hexes() for y in b.hexes()) == 1


# covers: BAT-GEO-001
def test_neighbours_on_even_and_odd_rows() -> None:
    assert FIELD.hex_count == 187
    assert at(3, 2) == 37 and FIELD.coords(37) == (3, 2)
    assert FIELD.hex(17, 0) is None and FIELD.hex(0, 11) is None
    assert FIELD.hex(-1, 0) is None and FIELD.hex(0, -1) is None
    assert FIELD.contains(186) and not FIELD.contains(187)

    def neighbours(hex_: int) -> list[int | None]:
        return [FIELD.neighbor(hex_, d) for d in DIRECTIONS]

    # Even row: upper/lower neighbours at x and x + 1.
    assert neighbours(at(5, 4)) == [at(6, 4), at(6, 5), at(5, 5), at(4, 4), at(5, 3), at(6, 3)]
    # Odd row: upper/lower neighbours at x - 1 and x.
    assert neighbours(at(5, 3)) == [at(6, 3), at(5, 4), at(4, 4), at(4, 3), at(4, 2), at(5, 2)]
    # Corners lose neighbours.
    assert neighbours(at(0, 0)) == [at(1, 0), at(1, 1), at(0, 1), None, None, None]
    assert FIELD.neighbor(at(16, 10), RIGHT) is None


# covers: BAT-GEO-002
def test_two_hex_footprints_extend_in_front_of_the_position() -> None:
    position = at(5, 5)
    assert FIELD.footprint(position, True, ATTACKER) == Footprint(at(6, 5), position)
    assert FIELD.footprint(position, True, DEFENDER) == Footprint(at(4, 5), position)
    assert FIELD.footprint(position, False, DEFENDER) == Footprint(position)
    # A position or other hex off the field has no footprint.
    assert FIELD.footprint(at(16, 3), True, ATTACKER) is None
    assert FIELD.footprint(at(0, 3), True, DEFENDER) is None
    assert FIELD.footprint(200, False, ATTACKER) is None
    fp = FIELD.footprint(position, True, ATTACKER)
    assert fp.covers(at(6, 5)) and fp.covers(position) and not fp.covers(at(4, 5))
    # The front hex first, then the rear hex (the position).
    assert fp.position == position and fp.hexes() == (at(6, 5), position)
    assert Footprint(position).position == position
    assert Footprint(position).hexes() == (position,)
    # A footprint from its front hex.
    for side in SIDES:
        for two_hex in (False, True):
            fp = FIELD.footprint(position, two_hex, side)
            assert FIELD.footprint_at_front(fp.front, two_hex, side) == fp
    assert FIELD.footprint_at_front(at(0, 3), True, ATTACKER) is None
    assert FIELD.footprint_at_front(200, False, ATTACKER) is None
    # The test's own touching (the brute force below): overlap is not touching.
    fp = FIELD.footprint(position, True, ATTACKER)
    assert not touches(fp, Footprint(at(9, 5))) and touches(fp, Footprint(at(7, 5)))
    assert not touches(fp, Footprint(position))
    # A stack whose footprint is off the field stands on its position alone (`Stack::footprint`).
    assert FIELD.stack_footprint(at(16, 3), True, ATTACKER) == Footprint(at(16, 3))
    assert FIELD.stack_footprint(position, True, DEFENDER) == Footprint(at(4, 5), position)


# covers: BAT-GEO-002, BAT-ATK-001
def test_attack_positions_cover_every_touching_position_once() -> None:
    def count(two_hex: bool, side: int, target: Footprint) -> int:
        return sum(p is not None for p in FIELD.attack_positions(two_hex, side, target))

    single = FIELD.footprint(at(8, 5), False, DEFENDER)
    double = FIELD.footprint(at(8, 5), True, DEFENDER)
    # One-hex attacker: the 6 neighbours of a one-hex target, the 8 of a two-hex one.
    assert count(False, ATTACKER, single) == 6
    assert count(False, ATTACKER, double) == 8
    # Two-hex attacker: also positions where only its rear hex touches (either side's).
    for side in SIDES:
        assert count(True, side, single) == 8
        assert count(True, side, double) == 10
    for side in SIDES:
        for two_hex in (False, True):
            for target in (single, double):
                positions = FIELD.attack_positions(two_hex, side, target)
                assert len(positions) == 24
                found = [p for p in positions if p is not None]
                assert len(set(found)) == len(found)
                for p in found:
                    assert touches(FIELD.footprint(p, two_hex, side), target)
                # Brute force: every touching position appears.
                every = [FIELD.footprint(h, two_hex, side) for h in range(187)]
                assert sum(fp is not None and touches(fp, target) for fp in every) == len(found)
    # In a corner, positions off the field are missing.
    corner = FIELD.footprint(at(0, 0), False, ATTACKER)
    assert count(False, DEFENDER, corner) == 3


def test_the_table_holds_every_shape_and_side() -> None:
    table = attack_table()
    assert table.shape == (187, 2, 2, 2, 2, 24) and table.dtype == np.int16
    for position in (at(8, 5), at(1, 0), at(15, 10), at(16, 3)):
        for target_two in (0, 1):
            for target_side in SIDES:
                target = FIELD.stack_footprint(position, bool(target_two), target_side)
                for two in (0, 1):
                    for side in SIDES:
                        want = FIELD.attack_positions(bool(two), side, target)
                        got = table[position, target_two, target_side, two, side]
                        assert got.tolist() == [-1 if p is None else p for p in want]
    # The slots keep their geometry: k = 0 is right of a one-hex target's hex for every shape.
    assert table[at(8, 5), 0, DEFENDER, 0, ATTACKER, 0] == at(9, 5)
    # A two-hex attacker's position is its rear hex: the attacker's lies left of its front hex,
    # on the target at k = 0, so it attacks from its rear hex on the contact hex (k = 1); the
    # defender's lies right of it.
    assert table[at(8, 5), 0, DEFENDER, 1, ATTACKER, 0] == -1
    assert table[at(8, 5), 0, DEFENDER, 1, ATTACKER, 1] == at(9, 5)
    assert table[at(8, 5), 0, DEFENDER, 1, DEFENDER, 0] == at(10, 5)
    assert table[at(8, 5), 0, DEFENDER, 1, DEFENDER, 1] == -1


def observation(stacks: dict[int, dict[int, float]], viewer_attacks: bool) -> np.ndarray:
    """An observation row holding only the viewer's seat and the given slots' columns."""
    obs = np.zeros(L.OBSERVATION_LEN, np.float32)
    obs[L.VIEWER_ATTACKER] = float(viewer_attacks)
    for s, columns in stacks.items():
        for column, value in {L.PRESENT: 1.0, **columns}.items():
            obs[L.STACKS + s * L.STACK_FEATURES + column] = value
    return obs


def test_the_read_out_takes_each_slot_s_place_and_the_active_stack_s_shape() -> None:
    table = attack_table()
    own_two = {L.X: 3, L.Y: 4, L.OWN: 1, L.TWO_HEX: 1, L.ACTIVE: 1}
    enemy_one = {L.X: 8, L.Y: 5}
    enemy_two = {L.X: 12, L.Y: 6, L.TWO_HEX: 1}
    turret = {L.X: 18, L.Y: 3, L.TURRET: 1}
    rows = [
        # The defender's two-hex stack acts.
        observation({0: own_two, 5: enemy_one, 9: enemy_two, 21: turret}, False),
        # The attacker's view, no stack acts (a tactics hand-over).
        observation({0: {**own_two, L.ACTIVE: 0}, 5: enemy_one}, True),
        # A hypnotized enemy two-hex stack acts for the attacker: it keeps its owner's side.
        observation({2: {**own_two, L.ACTIVE: 0}, 7: {**enemy_two, L.ACTIVE: 1}}, True),
        # The active stack off the field.
        observation({0: {**own_two, L.X: 17}, 5: enemy_one}, False),
    ]
    out = attack_hexes(torch.from_numpy(np.stack(rows)), torch.from_numpy(table))
    assert out.shape == (4, 42, 24) and out.dtype == torch.long
    out = out.numpy()
    assert (out[0, 5] == table[at(8, 5), 0, ATTACKER, 1, DEFENDER]).all()
    assert (out[0, 9] == table[at(12, 6), 1, ATTACKER, 1, DEFENDER]).all()
    assert (out[0, 0] == table[at(3, 4), 1, DEFENDER, 1, DEFENDER]).all()
    empty = np.ones(42, bool)
    empty[[0, 5, 9]] = False
    assert (out[0, empty] == -1).all() and (out[0, 21] == -1).all()
    assert (out[1] == -1).all() and (out[3] == -1).all()
    # Row 2: the active stack is the defender's (its own flag is clear in the attacker's view).
    assert (out[2, 2] == table[at(3, 4), 1, ATTACKER, 1, DEFENDER]).all()
    assert (out[2, 2] != table[at(3, 4), 1, ATTACKER, 1, ATTACKER]).any()


#: The hex planes of a Quicksand (observation v22, `h3_sim::observation`): the viewer's own spell
#: obstacles are planes 3 to 7 (Force Field, Fire Wall, Quicksand, Land Mine, the value), the
#: opponent's it sees planes 8 to 12.
OWN_QUICKSAND = 5
SEEN_QUICKSAND = 10


def planes(obs: np.ndarray) -> np.ndarray:
    return obs[L.FIELD : L.STACKS].reshape(L.HEX_PLANES, L.MAX_FIELD_HEXES)


def slots(obs: np.ndarray) -> np.ndarray:
    return obs[L.STACKS : L.ORDER].reshape(L.STACK_CAPACITY, L.STACK_FEATURES)


def hex_of(slot: np.ndarray) -> int:
    return int(slot[L.Y]) * 17 + int(slot[L.X])


def random_step(env: Env, rng: np.random.Generator) -> None:
    """A random legal action, never a retreat or a surrender (which end most battles early)."""
    side = env.to_move()
    mask = env.legal_mask(side)
    mask[L.END_TACTICS + 1 : L.END_TACTICS + 3] = 0
    entry = int(sample(mask[None], rng)[0])
    spell = int(sample(env.spell_mask(side, entry)[None], rng)[0])
    target = int(sample(env.target_mask(side, entry, spell)[None], rng)[0])
    second = int(sample(env.second_mask(side, entry, spell, target)[None], rng)[0])
    env.step(side, entry, spell, target, second)


def check_battle(env: Env, seed: int, table, counts: Counter, decisions: int = 400) -> None:
    """Plays `decisions` random decisions and checks every legal melee entry at each."""
    rng = np.random.default_rng(seed)
    for _ in range(decisions):
        side = env.to_move()
        if side < 0:
            return
        mask = env.legal_mask(side)
        obs = env.observe(side)
        hexes = attack_hexes(torch.from_numpy(obs[None]), table)[0].numpy()
        raw = slots(obs)
        attacks = np.flatnonzero(mask[L.ATTACK_BASE : L.SHOOT_BASE])
        if attacks.size:
            (active,) = np.flatnonzero((raw[:, L.ACTIVE] > 0.5) & (raw[:, L.PRESENT] > 0.5))
        for e in attacks:
            s, k = divmod(int(e), L.ATTACK_SLOTS)
            from_hex = int(hexes[s, k])
            assert from_hex >= 0, (seed, s, k)
            # The attacker attacks where it stands or moves first (BAT-MAG-032).
            assert from_hex == hex_of(raw[active]) or mask[L.MOVE_BASE + from_hex]
            with env.fork() as fork:
                fork.step(side, L.ATTACK_BASE + int(e))
                after = slots(fork.observe(side))[active]
                if after[L.ALIVE] < 0.5:
                    counts["attacker died"] += 1
                    continue
                stands = hex_of(after)
            if stands != from_hex:
                # The one exclusion: the walk stopped short on a pit the attacker's side did not
                # see (BAT-MAG-032), which the opponent's view shows as its own Quicksand.
                assert planes(env.observe(1 - side))[OWN_QUICKSAND, stands] != 0, (seed, s, k)
                assert planes(obs)[SEEN_QUICKSAND, stands] == 0, (seed, s, k)
                counts["stopped by an unseen pit"] += 1
                continue
            two = "two-hex" if raw[active, L.TWO_HEX] > 0.5 else "one-hex"
            counts["checked"] += 1
            counts[f"{two} attacker"] += 1
            counts["two-hex target" if raw[s, L.TWO_HEX] > 0.5 else "one-hex target"] += 1
            counts[f"side {side} attacks"] += 1
            # A hypnotized stack (BAT-MAG-026; all of these battles' are one-hex: the read-out's
            # test covers a two-hex one's side).
            counts["hypnotized attacker"] += int(raw[active, L.OWN] < 0.5)
        random_step(env, rng)


def test_every_legal_melee_entry_is_made_from_its_hex() -> None:
    """The key acceptance (E25 T25): on real battles (`field`: every shape on both sides; `full`:
    spell obstacles, unseen pits in seed 14, Hypnotize in seed 33; `siege`: walls, the moat,
    turrets off the field), each legal attack entry `(s, k)` of the side to move reads a hex, the
    attacker's or a legal move's, and the attacker stands on it after the attack (unless it died,
    or an unseen pit stopped it short)."""
    table = torch.from_numpy(attack_table())
    counts: Counter = Counter()
    for name, seeds in (("field", (0, 1, 2)), ("full", (0, 1, 14, 33)), ("siege", (0, 1, 2))):
        for seed in seeds:
            with Env.scenario(scenario(name, human=True), seed=seed, max_steps=2000) as env:
                check_battle(env, seed, table, counts)
    assert counts["checked"] >= 10000
    for category in ("one-hex attacker", "two-hex attacker", "one-hex target", "two-hex target"):
        assert counts[category] >= 2000, category
    assert counts["side 0 attacks"] >= 2000 and counts["side 1 attacks"] >= 2000
    assert counts["hypnotized attacker"] >= 1
    # The exclusion is exercised (seed 14) and stays rare: it never hides a wrong geometry.
    assert 1 <= counts["stopped by an unseen pit"] <= counts["checked"] // 200
