#!/usr/bin/env python3
"""Reads the running original's state from its memory: the battle's (E17 M2 T3) and the
adventure map's (M2 T6), *provisional*; clients/capture/README.md, "The original's state". Under
Wine the game is a Linux process that maps Heroes3.exe at its image base, so /proc/<pid>/mem
gives its globals and records at their addresses (docs/exe/map.toml, structs.toml). Read only
but for the harness's own timers, a siege's wall states before its first frame (E17 M4 T1b,
`write_walls`) and a map still's explored tiles (E24 M1 T3b2, `write_explored`).

  h3mem.py            print the battle's clocks and its stacks' animation state as JSON
  h3mem.py adventure  print the adventure map's state as JSON: the current player, the view,
                      each player's selected hero and resources, every hero on the map

Library use: `Mem` (globals, stacks, battle_heroes, battle_outcome, battlefield; current_player,
player, heroes, all_heroes, hero, hero_named, view, mouse_tile, path, difficulty, tile, explored,
monster, creature_name, artifact_name, dialog_text, dialog_items, level_up_dialog; since E08 T47a
dialog, item_text, spell_name, held_artifact, tactics_phase; the map's
stills: write_explored, def_palette, map_due, frame_steps). Needs a running game
(h3drive.py) and the rights to read its memory (root in a sandbox).
"""

import json
import struct
import subprocess
import sys

BATTLE_MANAGER_POINTER = 0x6993D0  # the battle manager while a battle runs
# A siege's walls (E17 M4 T1b; structs.toml, battle_manager): the parts' picture states and hit
# points (i32 by part), the gate's state.
WALL_STATES, WALL_HIT_POINTS, GATE_STATE = 0x13FA8, 0x13F60, 0x53A4
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
# E08 T32b1 (batch E4; structs.toml, hero): the level seed (u8 +0x8F) and the levels a magic
# school (i16 +0x3F) and Wisdom (u8 +0x90) were last offered at; each secondary skill's slot in
# the hero's list (u8[28] +0xE5, 1-8 in the order learned); the worn artifacts by slot (19
# entries +0x12D) and the backpack (64 entries +0x1D4, their number u8 +0x3D4), 8 bytes an entry:
# the artifact's id (i32, -1 none), then its subtype (hero_give_backpack_artifact 0x4E2F90).
SEED, SCHOOL_OFFER, WISDOM_OFFER, SLOTS = 0x8F, 0x3F, 0x90, 0xE5
WORN, WORN_SLOTS, BACKPACK, BACKPACK_SLOTS, BACKPACK_COUNT = 0x12D, 19, 0x1D4, 64, 0x3D4
# The artifact setup table (a pointer at 0x660B68, 0x20 bytes a record, filled from ARTRAITS.TXT
# by 0x44CD50): the name (char* +0x0, H3API's H3ArtifactSetup).
ARTIFACT_TABLE_POINTER, ARTIFACT_SIZE = 0x660B68, 0x20
# The worn slots' names (h3-experiment's SLOT_NAMES: the map's order is the game's).
ARTIFACT_SLOTS = ("head", "shoulders", "neck", "right hand", "left hand", "torso", "right ring",
                  "left ring", "feet", "misc 1", "misc 2", "misc 3", "misc 4", "war machine 1",
                  "war machine 2", "war machine 3", "war machine 4", "spell book", "misc 5")
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
# The tiles' vision words (E08 T37a; map.toml tile_vision_pointer 0x6989F8, tile_vision_word
# 0x4F79B0): a pointer to a u16 per tile at (level x side + y) x side + x, its low byte one
# explored bit per player (bit n for colour n), its bit 8 the guard flag.
TILE_VISION_POINTER = 0x6989F8
# The adventure manager's DEFs of the view (E24 design §1.3, §1.6; adventure_tick 0x4ED490): a
# DEF's pointer at manager +0x60 + 4 x terrain (water +0x80, lava +0x7C) and +0x88 + 4 x river
# (clear +0x8C, mud +0x94, lava +0x98); a DEF's 16-bit palette (a pointer at +0x20), its 256
# colours from +0x1C (H3API's H3LoadedDef, H3Palette565; palette16_rotate 0x522950). The objects'
# frame steps: manager +0x100 (design §1.6).
TERRAIN_DEFS, RIVER_DEFS, DEF_PALETTE, PALETTE_COLOURS, FRAME_STEPS = 0x60, 0x88, 0x20, 0x1C, 0x100
# The map's objects (E24 M1 T4a; structs.toml adventure_manager +0x5C, map_object): the map at
# manager +0x5C, its object records' vector at map +0x14 (begin, end), 12 bytes each in the map's
# order: +0x4 x, y, level; +0x8 the template (u16); +0xA the animation phase (design §1.6).
MANAGER_MAP, MAP_OBJECTS, OBJECT_RECORD = 0x5C, 0x14, 12
# The palette tick's next time (palette_tick_time 0x6989B4: the tick 0x4ED490 rotates once the
# clock reaches it and sets it to now + 200 on the map). The adventure map's idle step redraws
# the view once the clock reaches FRAME_DEADLINE (0x408A88; design §1.6).
PALETTE_TICK_TIME = 0x6989B4
# The creature table (a pointer at 0x6747B0; 0x74 bytes a record): the singular and plural names
# (char* +0x14, +0x18).
CREATURE_TABLE_POINTER, CREATURE_SIZE = 0x6747B0, 0x74
# The dialogs (E17 H1, observed; H3API's layouts, its addresses 0x50 higher in this executable's
# data): the window manager (a pointer at 0x699280) holds the topmost dialog at +0x54; a dialog
# lists its items from +0x2C, each item the next at +0x8, its id an i16 at +0x10; a text item
# holds its text as a string record at +0x30 (the characters' pointer +0x34, the length +0x38).
WINDOW_MANAGER_POINTER, TOP_DIALOG, FIRST_ITEM, NEXT_ITEM, ITEM_ID, ITEM_TEXT = (
    0x699280, 0x54, 0x2C, 0x8, 0x10, 0x34)
# A picture item (H3API's H3DlgDef, 0x48 bytes; its constructor 0x4EA720): the vtable 0x63EC48 at
# +0x0, the loaded DEF (+0x30, its name at +0x4) and the frame shown (i32 +0x34, the same place
# as a text item's characters). The level-up dialog (`level_up_dialog` 0x4F8880, E08 T32b1) shows
# the primary skill raised as item 2009 (pskil42.def, frame 0-3: attack, defense, power,
# knowledge) and the offered skills as items 2010 (left, or the only one) and 2011 (right),
# secskill.def frame 3 x skill + 3 + the hero's level of it before (hero_level_up 0x4DAABD): the
# skill and the level it would reach; an item is missing for a missing offer.
PICTURE_VTABLE, PICTURE_DEF, PICTURE_FRAME, DEF_NAME = 0x63EC48, 0x30, 0x34, 0x4
LEVEL_UP_PRIMARY, LEVEL_UP_LEFT, LEVEL_UP_RIGHT = 2009, 2010, 2011
PRIMARY_SKILLS = ("attack", "defense", "power", "knowledge")
# A dialog's place (E08 T47a, observed; H3API's H3BaseDlg): x, y, width, height (i32 +0x18 to
# +0x24); an item's (H3DlgItem) relative to it (i16 x +0x18, y +0x1A, u16 width +0x1C, height
# +0x1E) and its state (u16 +0x16). So a click finds an item by its id wherever its dialog stands.
DIALOG_BOX, ITEM_STATE, ITEM_BOX = 0x18, 0x16, 0x18
# The artifact held on the cursor in the hero screen and the exchange (map.toml held_artifact
# 0x698A88: its id, -1 none; E08 T47a).
HELD_ARTIFACT = 0x698A88
# The spell table (map.toml spell_table_pointer 0x687F58, 0x88 bytes a record): the name (char*
# +0x10) and the level (i32 +0x18), observed (E08 T47a; H3API's H3Spell).
SPELL_TABLE_POINTER, SPELL_SIZE, SPELL_NAME = 0x687F58, 0x88, 0x10
# Hero (E08 T47a; structs.toml, hero): the flags (u8 +0x48: bit 0 tight, bit 1 the tactics
# formation) and the learned spells (u8[70] +0x3EA, one byte per spell id).
FLAGS, LEARNED_SPELLS, SPELL_COUNT = 0x48, 0x3EA, 70
# The battle manager's tactics phase (i32 +0x13D68, structs.toml battle_manager; E08 T47a).
TACTICS_PHASE = 0x13D68
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
# The battlefield (E08 T32b1; structs.toml, battle_manager): the terrain (i32 +0x5394, set by
# 0x4643B0) and the special field (i32 +0x53C0, -1 none).
BATTLE_TERRAIN, SPECIAL_FIELD = 0x5394, 0x53C0
# The hex records (structs.toml, hex_record; H3API's H3CombatSquare): 187 from +0x1C4, 0x70 bytes
# each: the flags (u8 +0x10, 0x2 blocked), the stack on it (+0x18: side, record; -1 none), the
# pathfinder's attack mark for the active stack (u8 +0x4A).
HEXES, HEX_COUNT, HEX_SIZE, HEX_FLAGS, HEX_STACK, HEX_ATTACKABLE = (
    0x1C4, 187, 0x70, 0x10, 0x18, 0x4A)
TERRAINS = ("dirt", "sand", "grass", "snow", "swamp", "rough", "subterranean", "lava", "water",
            "rock")
SPECIAL_FIELDS = ("shore", "Magic Plains", "Cursed Ground", "Holy Ground", "Evil Fog",
                  "Clover Field", "Lucid Pools", "Fiery Fields", "Rocklands", "Magic Clouds")
# The heroes (E17 M3 T3a, design §4.5, §7.14): each side's hero (a pointer, 0: none) +0x53CC, its
# DEF group +0x53E4 and frame +0x53EC, its flag's frame +0x5414 (i32 per side).
HERO_POINTERS, HERO_GROUPS, HERO_FRAMES, FLAG_FRAMES = 0x53CC, 0x53E4, 0x53EC, 0x5414
# The active stack (a pointer: outline 0x60) and the mouse's highlight (+0x132CC set: the stack on
# the hex +0x132D0 gets outline 0x70) (M2 T4b-1; structs.toml, design §4.2).
ACTIVE_STACK, MOUSE_HIGHLIGHT, MOUSE_HEX = 0x132C8, 0x132CC, 0x132D0
# The mouse over the battle (E17 M3 T6c-2, design §9.9, §9.10): the hex under it (i32 +0x132D4,
# -1 none) and the cursor's kind there (i32 +0x132E0, `battle_cursor`); the mouse manager (a
# pointer at 0x699260) holds the cursor's type (i32 +0x4C: 2 the combat cursor) and frame (+0x50)
# and the point it is drawn at (i32 +0x6C, +0x70: the mouse's, its x even, read on a 33 ms grid of
# the game's clock, `mouse_poll` 0x50D680; design §9.10).
MOUSE_AT, CURSOR_KIND, MOUSE_MANAGER, CURSOR_TYPE, CURSOR_FRAME, CURSOR_X, CURSOR_Y = (
    0x132D4, 0x132E0, 0x699260, 0x4C, 0x50, 0x6C, 0x70)
MOUSE_POLL_NEXT = 0x69CA18  # the mouse poll's next time (0x50D680, a 33 ms grid)
# The magic animation (E17 M3 T3b, design §7.15): its number in the table 0x641E08 (+0x132EC,
# the last loaded) and the frame drawn (+0x132F0); a stack shows it while its byte +0x20 is set
# (`PlayMagicAnimation` 0x496840, `draw_battle_stack`'s tail 0x43E795).
MAGIC_ANIMATION, MAGIC_FRAME, MAGIC_SHOWN = 0x132EC, 0x132F0, 0x20
# The battle decided (E17 M3 T8, design §8.2: i32 +0x132F8, 0 from the battle's start, 1 once
# `battle_winner` 0x4762F0 finds it over, right before `battle_end` 0x477470 plays the result's
# tune; no clock step between).
DECIDED = 0x132F8
# The battle's outcome (E17 H1; structs.toml, battle_manager): each side retreated (u8 +0x132B0)
# or surrendered (u8 +0x132B2), the winner (i32 +0x13D48: 0 the attacker, 1 the defender).
RETREATED, SURRENDERED, WINNER = 0x132B0, 0x132B2, 0x13D48
# The battle's log (E17 M3 T6b, design §9.8): the combat dialog (a pointer at the manager's
# +0x132FC) holds the entries (a vector of string pointers: begin +0x58, end +0x5C), the new
# entries' count +0x64, the first shown +0x68 and the time it last showed them +0x6C (the
# game's clock); its panel (+0x70) the log's text item (+0x34), whose string record at +0x30 is
# what the item shows, the status line's text included.
COMBAT_DIALOG, LOG_ENTRIES, LOG_FRESH, LOG_FIRST, LOG_TIME, LOG_PANEL, PANEL_LOG_ITEM = (
    0x132FC, 0x58, 0x64, 0x68, 0x6C, 0x70, 0x34)
# Stack fields (structs.toml, battle_stack): name -> offset (all i32).
FIELDS = {
    "draw_layer": 0x8,
    "creature": 0x34,
    "hex": 0x38,
    "group": 0x3C,
    "frame": 0x40,
    "facing": 0x44,
    "count": 0x4C,
    "speed": 0xC4,  # E08 T32b1: a plan's move in reach
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
    # The stack's morale and luck (structs.toml; E17 M3 T7b-2b-2a: a scenario's hero effects
    # given by its skills, checked), set at the battle's start and after every action.
    "morale": 0x4E8,
    "luck": 0x4EC,
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
        self.pid = pid or game_pid()
        self.file = open(f"/proc/{self.pid}/mem", "rb", buffering=0)

    def read(self, address, size):
        self.file.seek(address)
        return self.file.read(size)

    def write_u32(self, address, value):
        """Writes a u32 into the game's memory: only the harness's own timers (M3 T6c-2,
        `align_mouse_poll`), never the game's state."""
        with open(f"/proc/{self.pid}/mem", "r+b", buffering=0) as f:
            f.seek(address)
            f.write(struct.pack("<I", value))

    def align_mouse_poll(self, now):
        """The mouse manager reads the mouse on a 33 ms grid of the game's clock (`mouse_poll`
        0x50D680: its next time 0x69CA18), whose phase the menus before a battle leave at random:
        set to `now` (the opening), the grid runs from the opening in every run (M3 T6c-2,
        design §9.10), so the frames a sequence keeps where the cursor moves keep their times."""
        self.write_u32(MOUSE_POLL_NEXT, now)

    def walls(self):
        """A siege's 18 parts' picture states (+0x13FA8, i32 each), their hit points (+0x13F60)
        and the gate's state (+0x53A4: 3 closed, 2 moving, 1 open, 0 destroyed; E17 design
        §10.1)."""
        m = self.manager()
        return (list(struct.unpack("<18i", self.read(m + WALL_STATES, 72))),
                list(struct.unpack("<18i", self.read(m + WALL_HIT_POINTS, 72))),
                self.i32(m + GATE_STATE))

    def write_walls(self, states, gate):
        """Writes a siege's wall states (E17 M4 T1b, the wall states as stills): each part's
        picture state and the gate's state, what the redraw reads (E17 design §10.1). Written
        after the siege's setup and before the battle's first redraw (`h3drive.into_battle`),
        the first frame shows them. The hit points stay: a tower's or the keep's written 0 makes
        the original skip its turn (its first frame then the attacker's, the record's the
        keep's)."""
        m = self.manager()
        with open(f"/proc/{self.pid}/mem", "r+b", buffering=0) as f:
            f.seek(m + WALL_STATES)
            f.write(struct.pack("<18i", *states))
            f.seek(m + GATE_STATE)
            f.write(struct.pack("<i", gate))

    def u32(self, address):
        return struct.unpack("<I", self.read(address, 4))[0]

    def i32(self, address):
        return struct.unpack("<i", self.read(address, 4))[0]

    def manager(self):
        return self.u32(BATTLE_MANAGER_POINTER)

    def globals(self):
        """The battle's clocks (absolute game times, ms), the combat speed, the idle steps
        taken, the active stack (`[side, record]` or None), the hex the mouse highlights
        (None: none), the hex under the mouse and the cursor's kind there (M3 T6c-2) and the
        cursor's type, frame and the point it is drawn at; whether the battle is decided (M3
        T8)."""
        manager = self.manager()
        found = {
            "decided": None,
            "frame_deadline": self.u32(FRAME_DEADLINE),
            "idle_step_time": self.u32(IDLE_STEP_TIME),
            "idle_steps": None,
            "combat_speed": self.i32(COMBAT_SPEED),
            "active": None,
            "mouse_hex": None,
            "magic": None,
            "mouse": None,
            "kind": None,
            "cursor": None,
            "cursor_at": None,
        }
        pointer = self.u32(MOUSE_MANAGER)
        if pointer:
            found["cursor"] = [self.i32(pointer + CURSOR_TYPE), self.i32(pointer + CURSOR_FRAME)]
            found["cursor_at"] = [self.i32(pointer + CURSOR_X), self.i32(pointer + CURSOR_Y)]
        if manager:
            found["decided"] = self.i32(manager + DECIDED) != 0
            found["idle_steps"] = self.i32(manager + STEP_COUNTER)
            found["active"] = self.stack_at(manager, self.u32(manager + ACTIVE_STACK))
            if self.read(manager + MOUSE_HIGHLIGHT, 1) != b"\0":
                found["mouse_hex"] = self.i32(manager + MOUSE_HEX)
            found["mouse"] = self.i32(manager + MOUSE_AT)
            found["kind"] = self.i32(manager + CURSOR_KIND)
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

    def hexes(self):
        """Every hex record (E08 T32b1): its flags (0x2 blocked), the stack on it ([side, record]
        or None) and the attack mark the pathfinder sets for the active stack (+0x4A: the stack
        there can be attacked this turn)."""
        manager = self.manager()
        raw = self.read(manager + HEXES, HEX_COUNT * HEX_SIZE)
        found = []
        for h in range(HEX_COUNT):
            side, record = struct.unpack_from("<2b", raw, h * HEX_SIZE + HEX_STACK)
            found.append({"hex": h, "flags": raw[h * HEX_SIZE + HEX_FLAGS],
                          "stack": [side, record] if side >= 0 else None,
                          "attackable": bool(raw[h * HEX_SIZE + HEX_ATTACKABLE])})
        return found

    def stack_hexes(self, side, record):
        """The hexes a living stack covers (E08 T32b1): the hex records (battle manager +0x1C4,
        0x70 bytes each) whose stack reference (side i8 +0x18, record i8 +0x19) names it; a
        two-hex stack's two, whichever its own hex field holds (a war machine's is its left one)."""
        manager = self.manager()
        raw = self.read(manager + HEXES, HEX_COUNT * HEX_SIZE)
        return [h for h in range(HEX_COUNT)
                if struct.unpack_from("<2b", raw, h * HEX_SIZE + HEX_STACK) == (side, record)]

    def battlefield(self):
        """The battle's terrain and special field by name (E08 T32b1; the special field None for
        none); None outside a battle."""
        manager = self.manager()
        if not manager:
            return None
        terrain, special = self.i32(manager + BATTLE_TERRAIN), self.i32(manager + SPECIAL_FIELD)
        return {"terrain": TERRAINS[terrain] if 0 <= terrain < len(TERRAINS) else str(terrain),
                "special": (SPECIAL_FIELDS[special] if 0 <= special < len(SPECIAL_FIELDS)
                            else None if special < 0 else str(special))}

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

    def c_string(self, pointer, limit=1024):
        """The characters at `pointer` up to their terminator ("" for none)."""
        if not pointer:
            return ""
        return self.read(pointer, limit).split(b"\0")[0].decode("cp1252")

    def battle_log(self):
        """The battle's log (E17 M3 T6b): the text its item shows (the status line's too), its
        entries, the new entries' count, the first shown and the time it last showed them (the
        game's clock, ms); None before the battle's dialog exists."""
        manager = self.manager()
        dialog = self.u32(manager + COMBAT_DIALOG) if manager else 0
        panel = self.u32(dialog + LOG_PANEL) if dialog else 0
        item = self.u32(panel + PANEL_LOG_ITEM) if panel else 0
        if not item:
            return None
        begin, end = self.u32(dialog + LOG_ENTRIES), self.u32(dialog + LOG_ENTRIES + 4)
        entries = [self.c_string(self.u32(at)) for at in range(begin, end, 4)] if begin else []
        return {"text": self.c_string(self.u32(item + ITEM_TEXT)), "entries": entries,
                "fresh": self.i32(dialog + LOG_FRESH), "first": self.i32(dialog + LOG_FIRST),
                "at": self.u32(dialog + LOG_TIME)}

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
        raw = self.read(self.game() + HEROES + HERO_SIZE * index, HERO_SIZE)
        x, y, level = struct.unpack_from("<3h", raw, 0)
        types = struct.unpack_from("<7i", raw, ARMY)
        counts = struct.unpack_from("<7i", raw, ARMY + 28)
        levels, slots = raw[SECONDARY:SECONDARY + 28], raw[SLOTS:SLOTS + 28]
        worn = struct.unpack_from(f"<{2 * WORN_SLOTS}i", raw, WORN)[::2]
        packed = struct.unpack_from(f"<{2 * BACKPACK_SLOTS}i", raw, BACKPACK)[::2]
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
            "skills": {SKILLS[i]: v for i, v in enumerate(levels) if v},
            # E08 T32b1: the skills in the hero's list (by slot), the seed and offer counters, the
            # artifacts (worn: {slot name: id}; the backpack's ids in its order).
            "skill_order": [[SKILLS[i], levels[i]]
                            for i in sorted((i for i in range(28) if levels[i]),
                                            key=lambda i: (slots[i], i))],
            "seed": raw[SEED],
            "wisdom_offer": raw[WISDOM_OFFER],
            "school_offer": struct.unpack_from("<h", raw, SCHOOL_OFFER)[0],
            "worn": {ARTIFACT_SLOTS[s]: a for s, a in enumerate(worn) if a >= 0},
            "backpack": [a for a in packed if a >= 0],
            "backpack_count": raw[BACKPACK_COUNT],
            "primary": list(struct.unpack_from("<4b", raw, PRIMARY)),
            # E08 T47a (batch E6): the seven army slots as they stand ([creature, count], the
            # count signed: a 16-bit wrap shows as -25 536; an empty slot [-1, 0]), the worn
            # artifacts by position number (0-18), the flags and the learned spells' ids.
            "slots": [[t, c] for t, c in zip(types, counts)],
            "positions": {s: a for s, a in enumerate(worn) if a >= 0},
            "flags": raw[FLAGS],
            "spells": [s for s in range(SPELL_COUNT) if raw[LEARNED_SPELLS + s]],
        }

    def artifact_name(self, artifact):
        """An artifact's name from the game's artifact setup table (E08 T32b1)."""
        record = self.u32(ARTIFACT_TABLE_POINTER) + ARTIFACT_SIZE * artifact
        return self.c_string(self.u32(record), 64)

    def spell_name(self, spell):
        """A spell's name from the game's spell table (E08 T47a)."""
        record = self.u32(SPELL_TABLE_POINTER) + SPELL_SIZE * spell
        return self.c_string(self.u32(record + SPELL_NAME), 64)

    def held_artifact(self):
        """The artifact held on the cursor (E08 T47a): its id, -1 for none."""
        return self.i32(HELD_ARTIFACT)

    def tactics_phase(self):
        """Whether the battle is in its tactics phase (E08 T47a; battle manager +0x13D68)."""
        return self.i32(self.manager() + TACTICS_PHASE) != 0

    def dialog(self):
        """The topmost dialog (E08 T47a): its address, vtable (what kind of screen it is) and box
        (x, y, width, height); None for none."""
        dialog = self.u32(self.u32(WINDOW_MANAGER_POINTER) + TOP_DIALOG)
        if not dialog:
            return None
        return {"at": dialog, "vtable": self.u32(dialog),
                "box": list(struct.unpack("<4i", self.read(dialog + DIALOG_BOX, 16)))}

    def dialog_items(self):
        """The topmost dialog's items (E08 T32b1): id, vtable and the i32 at +0x34 (a picture's
        frame); since E08 T47a each item's box on the screen (x, y, width, height), its state
        and its text (None for an item that holds none); [] for none."""
        shown = self.dialog()
        dx, dy = shown["box"][:2] if shown else (0, 0)
        items, item = [], self.u32(shown["at"] + FIRST_ITEM) if shown else 0
        seen = set()
        while item and item not in seen and len(seen) < 512:
            seen.add(item)
            raw = self.read(item, ITEM_TEXT + 12)
            x, y, width, height = struct.unpack_from("<2h2H", raw, ITEM_BOX)
            items.append({"at": item, "id": struct.unpack_from("<H", raw, ITEM_ID)[0],
                          "vtable": struct.unpack_from("<I", raw, 0)[0],
                          "def": struct.unpack_from("<I", raw, PICTURE_DEF)[0],
                          "frame": struct.unpack_from("<i", raw, PICTURE_FRAME)[0],
                          "box": [dx + x, dy + y, width, height],
                          "state": struct.unpack_from("<H", raw, ITEM_STATE)[0],
                          "text": self.item_text(raw)})
            item = struct.unpack_from("<I", raw, NEXT_ITEM)[0]
        return items

    def item_text(self, raw):
        """A dialog item's text from its first bytes (`raw`, at least ITEM_TEXT + 12 of them), or
        None: an item is a text item when its string record holds a pointer to exactly that many
        characters (other items hold pictures or numbers there)."""
        chars, length, capacity = struct.unpack_from("<3I", raw, ITEM_TEXT)
        if not (0x10000 <= chars < 0x80000000 and 0 < length <= capacity < 0x10000):
            return None
        try:
            data = self.read(chars, length + 1)
        except OSError:
            return None
        if len(data) == length + 1 and data[length] == 0 and b"\0" not in data[:length]:
            return data[:length].decode("cp1252")
        return None

    def level_up_dialog(self):
        """The level-up dialog on top (E08 T32b1), None if another or none shows: the primary
        skill raised (its name) and the offers left (or the only one) and right, each
        [skill name, the level it would reach (1 basic to 3 expert)] or None."""
        pictures = {i["id"]: i for i in self.dialog_items() if i["vtable"] == PICTURE_VTABLE}
        primary = pictures.get(LEVEL_UP_PRIMARY)
        if primary is None or not primary["def"]:
            return None
        if self.c_string(primary["def"] + DEF_NAME, 13).lower() != "pskil42.def":
            return None

        def offer(item_id):
            item = pictures.get(item_id)
            if item is None or item["frame"] < 3:
                return None
            skill, level = divmod(item["frame"] - 3, 3)
            return [SKILLS[skill], level + 1] if skill < len(SKILLS) else None

        frame = primary["frame"]
        return {"primary": PRIMARY_SKILLS[frame] if 0 <= frame < 4 else str(frame),
                "left": offer(LEVEL_UP_LEFT), "right": offer(LEVEL_UP_RIGHT)}

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

    def explored(self, x, y, z=0, colour=None):
        """Whether a player (default the current one) has explored tile (x, y) of a level: its
        bit in the tile's vision word (E08 T37a; tile_vision_word 0x4F79B0)."""
        side = self.i32(self.game() + MAP_SIDE)
        if not (0 <= x < side and 0 <= y < side and 0 <= z < self.levels()):
            raise ValueError(f"tile {(x, y, z)} is outside the map")
        colour = self.current_player() if colour is None else colour
        word = struct.unpack("<H", self.read(self.u32(TILE_VISION_POINTER)
                                             + 2 * ((z * side + y) * side + x), 2))[0]
        return bool(word >> colour & 1)

    def write_explored(self, z, explored, colour=0):
        """Writes a player's explored bit of every tile of a level (E24 M1 T3b2, the explored
        tiles as stills): `explored(x, y)` says which; the other players' bits and the guard flag
        stay. The view's next draw shows them (the unexplored pass reads only these bits, design
        §1.5); nothing else of the game's state changes."""
        side = self.i32(self.game() + MAP_SIDE)
        base = self.u32(TILE_VISION_POINTER) + 2 * z * side * side
        words = list(struct.unpack(f"<{side * side}H", self.read(base, 2 * side * side)))
        bit = 1 << colour
        for i, word in enumerate(words):
            words[i] = word | bit if explored(i % side, i // side) else word & ~bit
        with open(f"/proc/{self.pid}/mem", "r+b", buffering=0) as f:
            f.seek(base)
            f.write(struct.pack(f"<{side * side}H", *words))

    def def_palette(self, field):
        """The 256 colours (5-6-5) of the 16-bit palette of the DEF at adventure manager +`field`
        (E24 M1 T3b2: the terrains' and rivers' palettes as the tick has rotated them)."""
        palette = self.u32(self.manager_field(field) + DEF_PALETTE)
        return list(struct.unpack("<256H", self.read(palette + PALETTE_COLOURS, 512)))

    def map_due(self):
        """The adventure map's next palette tick and next redraw (clock times, ms)."""
        return self.u32(PALETTE_TICK_TIME), self.u32(FRAME_DEADLINE)

    def frame_steps(self):
        """The objects' frame steps (adventure manager +0x100, design §1.6)."""
        return self.manager_field(FRAME_STEPS)

    def map_objects(self):
        """The map's objects in its order (E24 M1 T4a): [(x, y, level, template, phase)] from
        their 12-byte records (the anchor, the template's index, the animation phase)."""
        begin, end = struct.unpack("<II", self.read(self.manager_field(MANAGER_MAP) + MAP_OBJECTS, 8))
        count = (end - begin) // OBJECT_RECORD
        raw = self.read(begin, OBJECT_RECORD * count) if count else b""
        return [(raw[i + 4], raw[i + 5], raw[i + 6]) + struct.unpack_from("<HB", raw, i + 8)
                for i in range(0, len(raw), OBJECT_RECORD)]

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
            text = self.item_text(raw)
            if text is not None:
                texts.append(text)
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
        print(json.dumps({"globals": mem.globals(), "stacks": mem.stacks(),
                          "log": mem.battle_log()}, indent=1))
