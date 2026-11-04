"""The battlefield's attack positions as the networks read them (E25 T25): a port of
`h3_battle::geometry` (`crates/h3-battle/src/geometry.rs`: `Field::neighbor`, `Field::footprint`,
`Field::footprint_at_front`, `Footprint::position`, `Field::attack_positions`; BAT-GEO-001,
BAT-GEO-002) and of `Stack::footprint` (`crates/h3-battle/src/state.rs`), so that action
encoding v13's melee entry `ATTACK_BASE + 24 * s + k` (`h3_sim::action`: attack stack slot `s`
from the `k`-th canonical attack position of the active stack's shape) names the hex the attack
is made from.

The field's hexes are numbered `y * width + x`; even rows are shifted right (a hex's upper and
lower neighbours are at `x` and `x + 1` on an even row, at `x - 1` and `x` on an odd one). A
stack's position is its hex, a two-hex stack's rear hex; its other hex, the front hex, lies right
of it for the attacker's stack and left of it for the defender's. Sides are numbered as the env's:
0 the attacker, 1 the defender.

`attack_table` tabulates the attack positions of every attacker shape against every target
position and shape; `attack_hexes` reads, for each stack slot of an observation v22 row
(`layout`), the 24 hexes the active stack would attack it from. Neither considers occupancy,
reach or who may attack whom: the legal mask does.
"""

from __future__ import annotations

from typing import NamedTuple

import numpy as np

from . import layout as L

#: The base ruleset's battlefield (`packs/base/battle.toml`), as `hexnet.FIELD_WIDTH` and
#: `FIELD_HEIGHT`.
FIELD_WIDTH = 17
FIELD_HEIGHT = 11

#: The directions, clockwise from the right, in `Direction::ALL`'s order.
RIGHT, BOTTOM_RIGHT, BOTTOM_LEFT, LEFT, TOP_LEFT, TOP_RIGHT = range(6)
DIRECTIONS = 6

#: The sides, as the env numbers them.
ATTACKER, DEFENDER = 0, 1

#: Number of canonical attack positions around a target (`ATTACK_SLOTS`).
ATTACK_SLOTS = L.ATTACK_SLOTS


class Footprint(NamedTuple):
    """The hexes a stack covers (`h3_battle::geometry::Footprint`): its front hex, the one it
    faces the enemy with (a one-hex stack's position), and a two-hex stack's rear hex, its
    position (`None` for a one-hex stack)."""

    front: int
    rear: int | None = None

    @property
    def position(self) -> int:
        """The position: a one-hex stack's hex, a two-hex stack's rear hex."""
        return self.front if self.rear is None else self.rear

    def hexes(self) -> tuple[int, ...]:
        """The covered hexes, the front hex first."""
        return (self.front,) if self.rear is None else (self.front, self.rear)

    def covers(self, hex_: int) -> bool:
        """Whether the footprint covers `hex_`."""
        return hex_ == self.front or hex_ == self.rear


class Field:
    """The battlefield's shape (`h3_battle::geometry::Field`) without its side columns, which
    none of the ported functions reads."""

    def __init__(self, width: int = FIELD_WIDTH, height: int = FIELD_HEIGHT) -> None:
        self.width = width
        self.height = height

    @property
    def hex_count(self) -> int:
        return self.width * self.height

    def hex(self, x: int, y: int) -> int | None:
        """The hex at column `x`, row `y`, if inside the field (BAT-GEO-001)."""
        if x < 0 or y < 0 or x >= self.width or y >= self.height:
            return None
        return x + self.width * y

    def coords(self, hex_: int) -> tuple[int, int]:
        """Column and row of a hex."""
        return hex_ % self.width, hex_ // self.width

    def contains(self, hex_: int) -> bool:
        """Whether `hex_` is inside the field."""
        return 0 <= hex_ < self.hex_count

    def neighbor(self, hex_: int, direction: int) -> int | None:
        """The neighbour of `hex_` in `direction`, if inside the field (BAT-GEO-001)."""
        x, y = self.coords(hex_)
        # Even rows are shifted right: their upper/lower neighbours are at x and x + 1; odd rows'
        # are at x - 1 and x.
        shift = 0 if y % 2 == 0 else -1
        nx, ny = (
            (x + 1, y),
            (x + shift + 1, y + 1),
            (x + shift, y + 1),
            (x - 1, y),
            (x + shift, y - 1),
            (x + shift + 1, y - 1),
        )[direction]
        return self.hex(nx, ny)

    @staticmethod
    def front_direction(side: int) -> int:
        """The direction from a two-hex stack's position to its front hex: right for the
        attacker, left for the defender."""
        return RIGHT if side == ATTACKER else LEFT

    def footprint(self, position: int, two_hex: bool, side: int) -> Footprint | None:
        """The footprint of a stack of `side` whose position is `position`, or `None` if the
        position or a two-hex stack's front hex is off the field (BAT-GEO-002; only inside the
        field, not standable, as in the original)."""
        if not self.contains(position):
            return None
        if not two_hex:
            return Footprint(position)
        front = self.neighbor(position, self.front_direction(side))
        return None if front is None else Footprint(front, position)

    def footprint_at_front(self, front: int, two_hex: bool, side: int) -> Footprint | None:
        """The footprint of a stack of `side` whose front hex is `front`, or `None` if a hex of
        it is off the field."""
        if not self.contains(front):
            return None
        if not two_hex:
            return Footprint(front)
        # The position lies behind the front hex: the opposite direction, half a turn.
        rear = self.neighbor(front, (self.front_direction(side) + 3) % DIRECTIONS)
        return None if rear is None else Footprint(front, rear)

    def stack_footprint(self, position: int, two_hex: bool, side: int) -> Footprint:
        """A stack's footprint (`Stack::footprint`): `footprint`, or the position alone where
        that is `None`."""
        footprint = self.footprint(position, two_hex, side)
        return Footprint(position) if footprint is None else footprint

    def attack_positions(self, two_hex: bool, side: int, target: Footprint) -> list[int | None]:
        """The canonical attack positions of an attacker (`two_hex`, `side`) against the
        footprint `target` (BAT-GEO-002): positions from which the attacker's footprint touches
        the target. Slot `k = (i * 6 + d) * 2 + v`: `i` is the target hex (0 its front hex, 1 a
        two-hex target's rear hex), `d` the direction from it to the contact hex `n`, `v = 0` the
        attacker's front hex on `n`, `v = 1` its rear hex on `n` (two-hex attackers only). A slot
        is `None` where the position is off the field, overlaps the target, or repeats an earlier
        slot, so every position appears once. Occupancy and reach are not considered."""
        out: list[int | None] = [None] * ATTACK_SLOTS
        target_hexes = (target.front, target.rear)
        # Positions found so far (every position is kept in its first slot only).
        seen: list[int] = []
        for k in range(ATTACK_SLOTS):
            i, d, v = k // 12, (k // 2) % DIRECTIONS, k % 2
            if v == 1 and not two_hex:
                continue
            t = target_hexes[i]
            contact = None if t is None else self.neighbor(t, d)
            if contact is None:
                continue
            # The contact hex neighbours the target, so the footprint touches it unless they
            # overlap (for `v = 1` the footprint's rear hex, its position, is the contact hex).
            if v == 0:
                footprint = self.footprint_at_front(contact, two_hex, side)
            else:
                footprint = self.footprint(contact, two_hex, side)
            if footprint is None or any(target.covers(h) for h in footprint.hexes()):
                continue
            p = footprint.position
            if p not in seen:
                seen.append(p)
                out[k] = p
        return out


def attack_table(width: int = FIELD_WIDTH, height: int = FIELD_HEIGHT) -> np.ndarray:
    """Every attack position on a `width` x `height` field, int16 (-1 none), of shape
    `[width * height, 2, 2, 2, 2, 24]`, indexed `[target position, target two-hex, target side,
    attacker two-hex, attacker side, k]`: the hex of `Field::attack_positions(attacker two-hex,
    attacker side, the target's footprint)[k]`, the target's footprint `Stack::footprint` of its
    position (its rear hex if it is two-hex), shape and side (0 attacker, 1 defender)."""
    field = Field(width, height)
    table = np.full((field.hex_count, 2, 2, 2, 2, ATTACK_SLOTS), -1, np.int16)
    for position in range(field.hex_count):
        for target_two in (0, 1):
            for target_side in (ATTACKER, DEFENDER):
                target = field.stack_footprint(position, bool(target_two), target_side)
                for two in (0, 1):
                    for side in (ATTACKER, DEFENDER):
                        hexes = field.attack_positions(bool(two), side, target)
                        table[position, target_two, target_side, two, side] = [
                            -1 if h is None else h for h in hexes
                        ]
    return table


def attack_hexes(obs, table, width: int = FIELD_WIDTH):
    """Each stack slot's 24 attack positions as the active stack would attack it, from rows of
    observation v22 (`[rows, OBSERVATION_LEN]`) and `attack_table`'s table as a tensor on their
    device (its field `width` wide): a LongTensor `[rows, 42, 24]` of hexes (`y * width + x`,
    -1 none), entry `[s, k]` the hex action encoding v13's entry `ATTACK_BASE + 24 * s + k`
    attacks from. -1 for every `k` of a slot without a stack or off the field (a town's turret),
    and for every slot of a row whose active stack is missing or off the field. A stack's side is
    the viewer's when its own flag is set (its owner's: a hypnotized active stack keeps its
    owner's side, BAT-MAG-026, as `Field::attack_positions` takes it), else the other's; its
    position and shape are its slot's `X`, `Y` and `TWO_HEX` (as `hexnet.HexNet.draw` reads
    them)."""
    import torch

    rows = obs.shape[0]
    height = table.shape[0] // width
    raw = obs[:, L.STACKS : L.ORDER].reshape(rows, L.STACK_CAPACITY, L.STACK_FEATURES)
    present = raw[..., L.PRESENT] > 0.5
    sx, sy = raw[..., L.X].long(), raw[..., L.Y].long()
    inside = present & (sx >= 0) & (sx < width) & (sy >= 0) & (sy < height)
    viewer_attacks = obs[:, L.VIEWER_ATTACKER : L.VIEWER_ATTACKER + 1] > 0.5
    side = ((raw[..., L.OWN] > 0.5) != viewer_attacks).long()
    two = (raw[..., L.TWO_HEX] > 0.5).long()
    active = inside & (raw[..., L.ACTIVE] > 0.5)
    # The active stack's slot (any slot of a row without one: masked below).
    slot = active.long().argmax(1, keepdim=True)
    attacker = two.gather(1, slot) * 2 + side.gather(1, slot)
    position = torch.where(inside, sy * width + sx, torch.zeros_like(sx))
    index = ((position * 2 + two) * 2 + side) * 4 + attacker
    hexes = table.reshape(-1, ATTACK_SLOTS)[index].long()
    valid = inside & active.any(1, keepdim=True)
    return torch.where(valid.unsqueeze(2), hexes, torch.full_like(hexes, -1))
