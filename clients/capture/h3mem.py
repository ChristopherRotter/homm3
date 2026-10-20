#!/usr/bin/env python3
"""Reads the running original's state from its memory: the battle's (E17 M2 T3) and the
adventure map's (M2 T6), *provisional*; clients/capture/README.md, "The original's state". Under
Wine the game is a Linux process that maps Heroes3.exe at its image base, so /proc/<pid>/mem
gives its globals and records at their addresses (docs/exe/map.toml, structs.toml). Read only:
nothing is written into the game.

  h3mem.py            print the battle's clocks and its stacks' animation state as JSON
  h3mem.py adventure  print the adventure map's state as JSON: the current player, the view,
                      each player's selected hero and resources, every hero on the map

Library use: `Mem` (globals, stacks, battle_heroes, battle_outcome; current_player, player, heroes,
all_heroes, hero, hero_named, view, mouse_tile, path, difficulty, tile, monster, creature_name,
message). Needs a running game (h3drive.py) and the rights to read its memory (root in a sandbox).
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
# bytes), the day's points (i32 +0x49) and the points left (i32 +0x4D); E17 H1: experience (u32
# +0x51), the hero's level (u16 +0x55), the army (seven creature types i32 +0x91, then seven
# counts), the secondary skills' levels by skill id (u8[28] +0xC9), the primary skills (i8[4]
# +0x476, with the worn artifacts' bonuses).
EXPERIENCE, HERO_LEVEL, ARMY, SECONDARY, PRIMARY = 0x51, 0x55, 0x91, 0xC9, 0x476
# The secondary skills by id (SSTRAITS.TXT's order; names as the packs harvest them).
SKILLS = ("Pathfinding", "Archery", "Logistics", "Scouting", "Diplomacy", "Navigation",
          "Leadership", "Wisdom", "Mysticism", "Luck", "Ballistics", "Eagle Eye", "Necromancy",
          "Estates", "Fire Magic", "Air Magic", "Water Magic", "Earth Magic", "Scholar", "Tactics",
          "Artillery", "Learning", "Offense", "Armorer", "Intelligence", "Sorcery", "Resistance",
          "First Aid")
# Game (E17 H1): the difficulty (i8 +0x1F6D8, 0 easy to 4 impossible), the tiles (a pointer
# +0x1FC40, level by level, row by row, 0x26 bytes each), two levels (u8 +0x1FC48).
DIFFICULTY, TILES, TWO_LEVELS, TILE_SIZE = 0x1F6D8, 0x1FC40, 0x1FC48, 0x26
# The date (E17 H2; H3API's H3Main date): the day of the week, the week and the month, u16 each.
DATE = 0x1F63E
DIFFICULTIES = ("easy", "normal", "hard", "expert", "impossible")
# A tile (map_item): the class's word (u32 +0x0; a monster's count bits 0-11, aggression 12-16
# signed, never flees 17, does not grow 18), class (i32 +0x1E), subclass (i16 +0x22).
MONSTER_CLASS = 54
# The creature table (a pointer at 0x6747B0; 0x74 bytes a record): the singular and plural names
# (char* +0x14, +0x18).
CREATURE_TABLE_POINTER, CREATURE_SIZE = 0x6747B0, 0x74
# The dialogs (E17 H1, observed; H3API's layouts, its addresses 0x50 higher in this executable's
# data): the window manager (a pointer at 0x699280) holds the topmost dialog at +0x54; a dialog
# lists its items from +0x2C, each item the next at +0x8, its id an i16 at +0x10; a text item
# holds its text as a string record at +0x30 (the characters' pointer +0x34, the length +0x38).
WINDOW_MANAGER_POINTER, TOP_DIALOG, FIRST_ITEM, NEXT_ITEM, ITEM_ID, ITEM_TEXT = (
    0x699280, 0x54, 0x2C, 0x8, 0x10, 0x34)
# Player: the selected hero (i32 +0x4, -1 none), the hero count (i8 +0x1) and the heroes (i32[8]
# +0x8, the hero list's order), the resources (i32[7] +0x9C).
RESOURCES = ("wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold")
# Adventure manager: the arrows of the selected hero's path (a pointer +0x48 to a u16 per tile,
# level by level: the underground's after the surface's, E17 H2), the view's top-left tile (+0xE4)
# and the tile under the mouse (+0xE8), both packed (unpack; the level in bit 26).
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
# The battle's outcome (E17 H1; structs.toml, battle_manager): each side retreated (u8 +0x132B0)
# or surrendered (u8 +0x132B2), the winner (i32 +0x13D48: 0 the attacker, 1 the defender).
RETREATED, SURRENDERED, WINNER = 0x132B0, 0x132B2, 0x13D48
# Stack fields (structs.toml, battle_stack): name -> offset (all i32).
FIELDS = {
    "draw_layer": 0x8,
    "creature": 0x34,
    "hex": 0x38,
    "group": 0x3C,
    "frame": 0x40,
    "facing": 0x44,
    "count": 0x4C,
    # The count the box shows while it is not -1 (E17 M3 T5b, design §4.3): the count before a
    # strike's kills, set by `stack_take_damage` (0x4440DB), -1 again at the end of
    # `strike_animation` (0x46935A).
    "shown": 0x50,
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

    def battle_outcome(self):
        """The battle just fought (E17 H1): the winning side and the side that retreated or
        surrendered (None: none)."""
        manager = self.manager()
        side = lambda raw: raw.index(1) if 1 in raw else None  # noqa: E731
        return {"winner": self.i32(manager + WINNER),
                "retreated": side(self.read(manager + RETREATED, 2)),
                "surrendered": side(self.read(manager + SURRENDERED, 2))}

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
        raw = self.read(self.game() + HEROES + HERO_SIZE * index, PRIMARY + 4)
        x, y, level = struct.unpack_from("<3h", raw, 0)
        types = struct.unpack_from("<7i", raw, ARMY)
        counts = struct.unpack_from("<7i", raw, ARMY + 28)
        return {
            "index": index,
            "name": raw[0x23:0x30].split(b"\0")[0].decode("latin-1"),
            "owner": struct.unpack_from("<b", raw, 0x22)[0],
            "tile": [x, y, level],
            "day_points": struct.unpack_from("<i", raw, 0x49)[0],
            "points": struct.unpack_from("<i", raw, 0x4D)[0],
            "experience": struct.unpack_from("<I", raw, EXPERIENCE)[0],
            "hero_level": struct.unpack_from("<H", raw, HERO_LEVEL)[0],
            "army": [[t, c] for t, c in zip(types, counts) if t >= 0],
            "skills": {SKILLS[i]: v for i, v in enumerate(raw[SECONDARY:SECONDARY + 28]) if v},
            "primary": list(struct.unpack_from("<4b", raw, PRIMARY)),
        }

    def heroes(self):
        """Every hero a player owns."""
        return [h for h in map(self.hero, range(HERO_COUNT)) if h["owner"] >= 0]

    def all_heroes(self):
        """The 156 heroes by type, on the map or not (E3-7: their experience at a new game)."""
        return [self.hero(i) for i in range(HERO_COUNT)]

    def hero_named(self, name):
        found = [h for h in self.heroes() if h["name"] == name]
        if len(found) != 1:
            raise KeyError(f"{len(found)} heroes named {name!r}")
        return found[0]

    def manager_field(self, offset):
        return self.u32(self.u32(ADVENTURE_MANAGER_POINTER) + offset)

    def view(self):
        """The view's top-left tile (x, y): the screen's map area starts there at (8, 8)."""
        return unpack(self.manager_field(VIEW))[:2]

    def view_level(self):
        """The level the view shows: 0 the surface, 1 the underground (E17 H2)."""
        return unpack(self.manager_field(VIEW))[2]

    def mouse_tile(self):
        """The tile under the mouse (x, y, level), as the game sees it."""
        return unpack(self.manager_field(MOUSE_TILE))

    def levels(self):
        return 2 if self.read(self.game() + TWO_LEVELS, 1)[0] else 1

    def path(self):
        """The selected hero's path as the map shows it: {(x, y, level): "green" | "red"}."""
        side = self.i32(self.game() + MAP_SIDE)
        tiles = side * side * self.levels()
        frames = struct.unpack(f"<{tiles}H", self.read(self.manager_field(PATH), 2 * tiles))
        return {(i % side, i // side % side, i // (side * side)):
                "green" if f in GREEN_FRAMES else "red" for i, f in enumerate(frames) if f}

    def date(self):
        """(month, week, day of the week), each from 1 (E17 H2)."""
        day, week, month = struct.unpack("<3H", self.read(self.game() + DATE, 6))
        return month, week, day


    def difficulty(self):
        """The game's difficulty by name (E17 H1)."""
        return DIFFICULTIES[struct.unpack("<b", self.read(self.game() + DIFFICULTY, 1))[0]]

    def tile(self, x, y, z=0):
        """A tile's word, class and subclass (map_item)."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        raw = self.read(self.u32(game + TILES) + TILE_SIZE * ((z * side + y) * side + x),
                        TILE_SIZE)
        return {"word": struct.unpack_from("<I", raw, 0)[0],
                "class": struct.unpack_from("<i", raw, 0x1E)[0],
                "subclass": struct.unpack_from("<h", raw, 0x22)[0]}

    def monster(self, x, y, z=0):
        """The wandering monster on a tile from its word (E17 H1; E3-3), None for none: its
        creature, count, aggression (-4 to 10, signed 5 bits), never flees and does not grow."""
        t = self.tile(x, y, z)
        if t["class"] != MONSTER_CLASS:
            return None
        word = t["word"]
        aggression = (word >> 12) & 0x1F
        return {"creature": t["subclass"], "count": word & 0xFFF,
                "aggression": aggression - 32 * (aggression >= 16),
                "never_flees": bool(word >> 17 & 1), "fixed_count": bool(word >> 18 & 1)}

    def creature_name(self, creature, count=1):
        """A creature's name from the game's creature table, plural unless count is 1."""
        record = self.u32(CREATURE_TABLE_POINTER) + CREATURE_SIZE * creature
        name = self.u32(record + (0x14 if count == 1 else 0x18))
        return self.read(name, 64).split(b"\0")[0].decode("cp1252")

    def dialog_text(self):
        """The topmost dialog's text: its text items' strings in item order, joined by a line
        break ("" for none). An item is a text item when its string record holds a pointer to
        exactly that many characters (other items hold pictures or numbers there)."""
        dialog = self.u32(self.u32(WINDOW_MANAGER_POINTER) + TOP_DIALOG)
        texts, item, seen = [], self.u32(dialog + FIRST_ITEM) if dialog else 0, set()
        while item and item not in seen and len(seen) < 256:
            seen.add(item)
            raw = self.read(item, ITEM_TEXT + 12)
            chars, length, capacity = struct.unpack_from("<3I", raw, ITEM_TEXT)
            if 0x10000 <= chars < 0x80000000 and 0 < length <= capacity < 0x10000:
                try:
                    data = self.read(chars, length + 1)
                except OSError:
                    data = b""
                if len(data) == length + 1 and data[length] == 0 and b"\0" not in data[:length]:
                    texts.append(data[:length].decode("cp1252"))
            item = struct.unpack_from("<I", raw, NEXT_ITEM)[0]
        return "\n".join(texts)


def unpack(packed):
    """A packed tile (x in bits 0-9, y in bits 16-25, each signed, the level in bit 26): (x, y,
    level). The bits between belong to something else."""
    x, y = packed & 0x3FF, (packed >> 16) & 0x3FF
    return x - 1024 * (x >= 512), y - 1024 * (y >= 512), (packed >> 26) & 1


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
