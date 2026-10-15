#!/usr/bin/env python3
"""Reads the running original's state from its memory: the battle's (E17 M2 T3) and the
adventure map's (M2 T6), *provisional*; clients/capture/README.md, "The original's state". Under
Wine the game is a Linux process that maps Heroes3.exe at its image base, so /proc/<pid>/mem
gives its globals and records at their addresses (docs/exe/map.toml, structs.toml). Read only:
nothing is written into the game.

  h3mem.py            print the battle's clocks and its stacks' animation state as JSON
  h3mem.py adventure  print the adventure map's state as JSON: the current player, the view,
                      each player's selected hero and resources, every hero on the map

Library use: `Mem` (globals, stacks, battle_heroes; current_player, player, heroes, hero, hero_named, view,
mouse_tile, path). Needs a running game (h3drive.py) and the rights to read its memory (root in a
sandbox).
"""

import json
import struct
import subprocess
import sys

BATTLE_MANAGER_POINTER = 0x6993D0  # the battle manager while a battle runs
# The adventure map (M2 T6; structs.toml: game, hero, player_record, adventure_manager).
GAME_POINTER = 0x6994E8
ADVENTURE_MANAGER_POINTER = 0x699268
CURRENT_PLAYER = 0x69CCA8  # the colour whose turn runs
# Game: the map's side +0x1FC44, the players +0x20AD0 (0x168 each), the heroes +0x21620 (156,
# 0x492 each, by hero type).
MAP_SIDE, PLAYERS, PLAYER_SIZE, HEROES, HERO_COUNT, HERO_SIZE = (
    0x1FC44, 0x20AD0, 0x168, 0x21620, 156, 0x492)
# Hero: tile x, y, level (i16 at +0x0, +0x2, +0x4), owner (i8 +0x22, -1 none), name (+0x23, 13
# bytes), the day's points (i32 +0x49) and the points left (i32 +0x4D).
# Player: the selected hero (i32 +0x4, -1 none), the hero count (i8 +0x1) and the heroes (i32[8]
# +0x8, the hero list's order), the resources (i32[7] +0x9C).
RESOURCES = ("wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold")
# Adventure manager: the arrows of the selected hero's path (u16 per tile of level 0, +0x48),
# the view's top-left tile (+0xE4) and the tile under the mouse (+0xE8), both packed (unpack).
PATH, VIEW, MOUSE_TILE = 0x48, 0xE4, 0xE8
# A path arrow's frame: 0 none, 1 to 25 a move today (green), 26 to 50 a later one (red).
GREEN_FRAMES = range(1, 26)
FRAME_DEADLINE = 0x698998  # battle animations' next frame time (0x4F82F0 waits for it)
IDLE_STEP_TIME = 0x6989B8  # the idle step's next time (0x4960D0)
COMBAT_SPEED = 0x6987EC  # the setting, 0 to 2 (factors 0x63CF7C)
# Battle manager: records per side +0x54BC, the stacks from +0x54CC (21 per side, 0x6EE8 apart,
# 0x548 each), the idle step's counter +0x13FFC.
SIDE_COUNTS, STACKS, SIDE_STRIDE, STACK_SIZE, STEP_COUNTER = 0x54BC, 0x54CC, 0x6EE8, 0x548, 0x13FFC
# The heroes (E17 M3 T3a, design §4.5, §7.14): each side's hero (a pointer, 0: none) +0x53CC, its
# DEF group +0x53E4 and frame +0x53EC, its flag's frame +0x5414 (i32 per side).
HERO_POINTERS, HERO_GROUPS, HERO_FRAMES, FLAG_FRAMES = 0x53CC, 0x53E4, 0x53EC, 0x5414
# The active stack (a pointer: outline 0x60) and the mouse's highlight (+0x132CC set: the stack on
# the hex +0x132D0 gets outline 0x70) (M2 T4b-1; structs.toml, design §4.2).
ACTIVE_STACK, MOUSE_HIGHLIGHT, MOUSE_HEX = 0x132C8, 0x132CC, 0x132D0
# The magic animation (E17 M3 T3b, design §7.15): its number in the table 0x641E08 (+0x132EC,
# the last loaded) and the frame drawn (+0x132F0); a stack shows it while its byte +0x20 is set
# (`PlayMagicAnimation` 0x496840, `draw_battle_stack`'s tail 0x43E795).
MAGIC_ANIMATION, MAGIC_FRAME, MAGIC_SHOWN = 0x132EC, 0x132F0, 0x20
# Stack fields (structs.toml, battle_stack): name -> offset (all i32).
FIELDS = {
    "draw_layer": 0x8,
    "creature": 0x34,
    "hex": 0x38,
    "group": 0x3C,
    "frame": 0x40,
    "facing": 0x44,
    "count": 0x4C,
    "y_offset": 0x100,
    "x_offset": 0x104,
    "fidget_clock": 0xFC,
    "fidget_interval": 0x154,
    "walk_time": 0x158,
    "attack_time": 0x15C,
}


def game_pid():
    """The Wine process that maps Heroes3.exe at its image base, 0x400000."""
    found = subprocess.run(["pgrep", "-f", "Heroes3.exe"], capture_output=True, text=True)
    for pid in found.stdout.split():
        try:
            with open(f"/proc/{pid}/maps") as maps:
                if any(line.startswith("00400000-") for line in maps):
                    return int(pid)
        except OSError:
            continue
    raise RuntimeError("no running Heroes3.exe")


class Mem:
    def __init__(self, pid=None):
        self.file = open(f"/proc/{pid or game_pid()}/mem", "rb", buffering=0)

    def read(self, address, size):
        self.file.seek(address)
        return self.file.read(size)

    def u32(self, address):
        return struct.unpack("<I", self.read(address, 4))[0]

    def i32(self, address):
        return struct.unpack("<i", self.read(address, 4))[0]

    def manager(self):
        return self.u32(BATTLE_MANAGER_POINTER)

    def globals(self):
        """The battle's clocks (absolute game times, ms), the combat speed, the idle steps
        taken, the active stack (`[side, record]` or None) and the hex the mouse highlights
        (None: none)."""
        manager = self.manager()
        found = {
            "frame_deadline": self.u32(FRAME_DEADLINE),
            "idle_step_time": self.u32(IDLE_STEP_TIME),
            "idle_steps": None,
            "combat_speed": self.i32(COMBAT_SPEED),
            "active": None,
            "mouse_hex": None,
            "magic": None,
        }
        if manager:
            found["idle_steps"] = self.i32(manager + STEP_COUNTER)
            found["active"] = self.stack_at(manager, self.u32(manager + ACTIVE_STACK))
            if self.read(manager + MOUSE_HIGHLIGHT, 1) != b"\0":
                found["mouse_hex"] = self.i32(manager + MOUSE_HEX)
            found["magic"] = [self.i32(manager + MAGIC_ANIMATION),
                              self.i32(manager + MAGIC_FRAME)]
        return found

    @staticmethod
    def stack_at(manager, pointer):
        """The `[side, record]` of a stack record's address, None for none."""
        offset = pointer - manager - STACKS
        if pointer == 0 or not 0 <= offset < 2 * SIDE_STRIDE:
            return None
        side, rest = divmod(offset, SIDE_STRIDE)
        return [side, rest // STACK_SIZE]

    def stacks(self):
        """Every stack record in use: side, record and FIELDS; [] outside a battle."""
        manager, result = self.manager(), []
        if not manager:
            return result
        for side in (0, 1):
            for record in range(self.i32(manager + SIDE_COUNTS + 4 * side)):
                base = manager + STACKS + side * SIDE_STRIDE + record * STACK_SIZE
                raw = self.read(base, STACK_SIZE)
                stack = {"side": side, "record": record}
                stack.update({k: struct.unpack_from("<i", raw, o)[0] for k, o in FIELDS.items()})
                stack["magic"] = raw[MAGIC_SHOWN]
                result.append(stack)
        return result

    def battle_heroes(self):
        """Each side's hero as drawn (E17 M3 T3a): side, DEF group, frame and the flag's frame;
        a side without a hero is left out; [] outside a battle."""
        manager = self.manager()
        if not manager:
            return []
        return [{"side": side, "group": self.i32(manager + HERO_GROUPS + 4 * side),
                 "frame": self.i32(manager + HERO_FRAMES + 4 * side),
                 "flag": self.i32(manager + FLAG_FRAMES + 4 * side)}
                for side in (0, 1) if self.u32(manager + HERO_POINTERS + 4 * side)]

    # -- the adventure map ---------------------------------------------------------------------
    def game(self):
        return self.u32(GAME_POINTER)

    def current_player(self):
        return self.i32(CURRENT_PLAYER)

    def player(self, colour):
        """A player's selected hero (-1 none), its heroes in the hero list's order and its
        resources by name."""
        raw = self.read(self.game() + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        count = struct.unpack_from("<b", raw, 0x1)[0]
        return {
            "colour": colour,
            "selected": struct.unpack_from("<i", raw, 0x4)[0],
            "heroes": list(struct.unpack_from(f"<{count}i", raw, 0x8)),
            "resources": dict(zip(RESOURCES, struct.unpack_from("<7i", raw, 0x9C))),
        }

    def hero(self, index):
        raw = self.read(self.game() + HEROES + HERO_SIZE * index, 0x51)
        x, y, level = struct.unpack_from("<3h", raw, 0)
        return {
            "index": index,
            "name": raw[0x23:0x30].split(b"\0")[0].decode("latin-1"),
            "owner": struct.unpack_from("<b", raw, 0x22)[0],
            "tile": [x, y, level],
            "day_points": struct.unpack_from("<i", raw, 0x49)[0],
            "points": struct.unpack_from("<i", raw, 0x4D)[0],
        }

    def heroes(self):
        """Every hero a player owns."""
        return [h for h in map(self.hero, range(HERO_COUNT)) if h["owner"] >= 0]

    def hero_named(self, name):
        found = [h for h in self.heroes() if h["name"] == name]
        if len(found) != 1:
            raise KeyError(f"{len(found)} heroes named {name!r}")
        return found[0]

    def manager_field(self, offset):
        return self.u32(self.u32(ADVENTURE_MANAGER_POINTER) + offset)

    def view(self):
        """The view's top-left tile (x, y): the screen's map area starts there at (8, 8)."""
        return unpack(self.manager_field(VIEW))

    def mouse_tile(self):
        """The tile under the mouse (x, y), as the game sees it."""
        return unpack(self.manager_field(MOUSE_TILE))

    def path(self):
        """The selected hero's path as the map shows it: {(x, y): "green" | "red"}, level 0."""
        side = self.i32(self.game() + MAP_SIDE)
        raw = self.read(self.manager_field(PATH), 2 * side * side)
        frames = struct.unpack(f"<{side * side}H", raw)
        return {(i % side, i // side): "green" if f in GREEN_FRAMES else "red"
                for i, f in enumerate(frames) if f}


def unpack(packed):
    """A packed tile (x in bits 0-9, y in bits 16-25, each signed): (x, y). The bits between
    belong to something else."""
    x, y = packed & 0x3FF, (packed >> 16) & 0x3FF
    return x - 1024 * (x >= 512), y - 1024 * (y >= 512)


def adventure_state(mem):
    return {
        "current_player": mem.current_player(),
        "view": mem.view(),
        "players": [mem.player(colour) for colour in range(8)
                    if mem.player(colour)["heroes"]],
        "heroes": mem.heroes(),
    }


if __name__ == "__main__":
    mem = Mem()
    if sys.argv[1:] == ["adventure"]:
        print(json.dumps(adventure_state(mem), indent=1))
    elif len(sys.argv) > 1:
        sys.exit(__doc__)
    else:
        print(json.dumps({"globals": mem.globals(), "stacks": mem.stacks()}, indent=1))
