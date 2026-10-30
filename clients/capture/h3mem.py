#!/usr/bin/env python3
"""Reads the running original's state from its memory: the battle's (E17 M2 T3) and the
adventure map's (M2 T6), *provisional*; clients/capture/README.md, "The original's state". Under
Wine the game is a Linux process that maps Heroes3.exe at its image base, so /proc/<pid>/mem
gives its globals and records at their addresses (docs/exe/map.toml, structs.toml). Read only
but for the harness's own timers, a siege's wall states before its first frame (E17 M4 T1b,
`write_walls`), a map still's explored tiles (E24 M1 T3b2, `write_explored`), its heroes' and
boats' facings (E24 M1 T5a, `write_facing`) and its boarded boats' flag colours (E24 M1 T5b2,
`write_boat_colour`), and a map sequence's set-up at its 0 (E24 M2: the speeds in the settings,
`write_u32`; the selected hero's movement points, `write_points`; every object's animation phase,
`write_phases`; the objects' frame steps, `write_frame_steps`; the palette tick's and the
redraw's next times).

  h3mem.py            print the battle's clocks, its stacks' animation state, its log and the
                      hovered dialog item as JSON
  h3mem.py adventure  print the adventure map's state as JSON: the current player, the view,
                      each player's selected hero and resources, every hero on the map

Library use: `Mem` (globals, stacks, battle_heroes, battle_outcome, battlefield; current_player,
player, heroes, all_heroes, hero, hero_named, view, mouse_tile, path, difficulty, tile, explored,
monster, creature_name, artifact_name, dialog_text, dialog_items, level_up_dialog; since E08 T47a
dialog, item_text, spell_name, held_artifact, tactics_phase; since E08 T57 object_at,
universities, dialog_pictures, map_kind, template_bytes; since E17 M4 T3b hovered_item; the map's
stills: write_explored, def_palette, map_due, frame_steps; since E24 M1 T5a boats, standing,
map_heroes, hero_on, write_facing, write_bytes). Needs a running game
(h3drive.py) and the rights to read its memory (root in a sandbox).
"""

import json
import os
import struct
import subprocess
import sys
from pathlib import Path

BATTLE_MANAGER_POINTER = 0x6993D0  # the battle manager while a battle runs
# A siege's walls (E17 M4 T1b; structs.toml, battle_manager): the parts' picture states and hit
# points (i32 by part), the gate's state.
WALL_STATES, WALL_HIT_POINTS, GATE_STATE = 0x13FA8, 0x13F60, 0x53A4
# A siege's fort level (i32 +0x132F4: 0 none, 1 Fort, 2 Citadel, 3 Castle) and its turret records
# (E17 M4 T2b; structs.toml, battle_manager's `turrets`, `siege_turret_records` 0x466C50): the
# keep's, the lower tower's and the upper tower's, 0x24 bytes each, their battle DEF (a pointer
# +0x4, 0: not set up), facing (+0x14), group (+0x18) and frame (+0x1C), i32 each.
FORT, TURRETS, TURRET_SIZE, TURRET_DEF, TURRET_POSE = 0x132F4, 0x13D78, 0x24, 0x4, 0x14
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
# The game's version (E08 T57; structs.toml game +0x1F698 map_kind: 0 RoE, 1 AB, 2 SoD).
MAP_KIND, MAP_KINDS = 0x1F698, ("RoE", "AB", "SoD")
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
# The adventure pathfinder (pathfinder_run 0x56B440; E24 M3 T6a1): its nodes at +0x24.
PATHFINDER_POINTER = 0x699284
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
# The map's object templates (E24 M1 T4b; design §2.7): a vector at map +4 of 0x44-byte records,
# the DEF's name a string at +0 (its characters' pointer at +4, its length at +8), the class at
# +0x38, the subclass at +0x3C. A new game appends a copy for each random object it draws
# (0x4C9990) and points the object's record at it.
MAP_TEMPLATES, TEMPLATE_RECORD, TEMPLATE_CLASS = 0x4, 0x44, 0x38
# The heroes and boats on the map (E24 M1 T5a; design §2.8): a tile's word (+0x0) is the index
# of the hero (class 34, among the 156 by type) or boat (class 8) standing there when the tile has
# the entrance bit (+0xD 0x10) and the word is not -1 (view_tile_heroes_boats 0x40FB30). A hero's
# class (i32 +0x30: its map DEF), facing (u8 +0x47) and status (u32 +0x105: bit 18 in a boat);
# its boat is the one in use (u8 +0x18) whose hero (i32 +0x20) is its id (i32 +0x1A) with a hero
# aboard (u8 +0x24; hero_boat 0x4CE900). The boats: a vector at game +0x4E3BC (begin, end), 0x28
# bytes each: tile x, y, level (i16 +0x0, +0x2, +0x4), type (i8 +0x1A), facing (i8 +0x1B), the
# flag's colour (i8 +0x1C).
HERO_CLASS_FIELD, HERO_ID, HERO_FACING, HERO_STATUS, IN_BOAT = 0x30, 0x1A, 0x47, 0x105, 1 << 18
BOATS, BOAT_RECORD, BOAT_FACING, BOAT_COLOUR = 0x4E3BC, 0x28, 0x1B, 0x1C
HERO_OBJECT, BOAT_OBJECT, ENTRANCE = 34, 8, 0x10
# The mines' and Lighthouses' records (E08 T79a, batch E9a; survey E08-economy §2.1, structs.toml
# "mine_record"): a vector at game +0x4E38C (begin, end), 0x40 bytes each: the owner (i8 +0x0, -1
# none), the resource (i8 +0x1: a mine's subclass 0-6, an abandoned mine's drawn one, 100 for a
# Lighthouse), the abandoned flag (u8 +0x2), the guards (creature types i32 +0x4 to +0x1F, counts
# i32 +0x20 to +0x3B), the entrance's column, row and level (u8 +0x3C to +0x3E). A hero's record
# keeps the class and word of the object under it (i32 +0xC, +0x14; hero_hide).
MINES, MINE_RECORD, MINE_GUARDS, MINE_ENTRANCE, LIGHTHOUSE = 0x4E38C, 0x40, 0x4, 0x3C, 100
HERO_UNDER_CLASS, HERO_UNDER_WORD = 0xC, 0x14
# A player's day's income (E08 T79b, batch E9b; survey E08-economy §8.1, structs.toml
# "player_record"): seven i32 at +0x108, wood to gold, computed at each day's start
# (player_daily_income 0x4B8AF0) and kept until the next; its Mystical Gardens' mask (u32 +0xB8,
# a bit per Garden by the index in its word's bits 0-4, §9.3). A wandering monster's word keeps
# the sixteenths of its count in bits 27-30 (§10.1; the count in bits 0-11).
PLAYER_INCOME, PLAYER_GARDENS, SIXTEENTHS_SHIFT = 0x108, 0xB8, 27
# The map hint's visit marks (E24 M3 T6a2a; E24 design §4.11, structs.toml): per class the kind
# whose bit of the viewing player in the game's kinds_visited (u8[28] +0x4E344) shows its bonus
# ("word": the object's word's bit 5 + the shown player, 0x69778C; None no bonus) and its mark's
# rule: ("flags", mask) the selected hero's status +0x105 has a bit of the mask; ("hero", offset)
# the bit (word & 0x1F) of the selected hero's u32 there; ("player", offset, bit) the bit (word &
# 0x1F) of the player's u32 there and, with a bit, that bit of the word clear; ("owner",) the
# word's bit 5 + the selected hero's owner; those with a selected hero only; ("garden",) as
# ("player", 0xB8, 10) without one; ("obelisk",) the viewing player's bit of the game's u8 at
# +0x4E3E9 + the word; ("current",) the word's bit 5 + the current player (always a mark);
# ("left", mask) that bit set, then visited when the word's bits of the mask are clear (none
# without the bit); None no mark.
KINDS_VISITED, OBELISKS_VISITED, SHOWN_PLAYER = 0x4E344, 0x4E3E9, 0x69778C
VISIT_RULES = {
    4: (None, ("hero", 0x73)), 11: (0, ("flags", 1 << 2)), 14: (1, ("flags", 1 << 3)),
    22: (None, ("player", 0xC0, None)), 23: (7, ("hero", 0x5B)), 28: (2, ("flags", 1 << 13)),
    30: ("word", ("flags", 1 << 5 | 1 << 27 | 1 << 28 | 1 << 29)), 31: (13, ("flags", 1 << 14)),
    32: (4, ("hero", 0x5F)), 35: (14, None), 38: (22, ("flags", 1 << 4 | 1 << 25)),
    39: (None, ("player", 0xC4, None)), 41: (6, ("hero", 0x6F)), 47: (9, ("hero", 0x77)),
    48: (15, ("player", 0xBC, 6)), 49: (25, ("flags", 1)), 51: (8, ("hero", 0x63)),
    52: (16, ("flags", 1 << 15)), 55: (None, ("garden",)), 56: (26, ("flags", 1 << 7)),
    57: (None, ("obelisk",)), 61: (11, ("hero", 0x67)), 63: (None, ("owner",)),
    64: (17, ("flags", 1 << 16)), 92: (None, ("flags", 1 << 20)), 94: (None, ("flags", 1 << 1)),
    96: (23, ("flags", 1 << 8 | 1 << 26)), 100: (5, ("hero", 0x57)), 104: (24, None),
    105: (None, ("current",)), 107: (10, ("hero", 0x7B)), 108: (None, ("current",)),
    109: (None, ("left", 0x1F)), 110: (27, ("flags", 1 << 6)), 112: (None, ("left", 0xF << 13)),
}
# The owners' and contents' hints (E24 M3 T6a2b; E24 design §4.12, structs.toml): the game's mines
# and Lighthouses (+0x4E38C, 0x40 bytes: owner i8 +0, resource +1, abandoned +2, the guards' army
# +4), creature banks (+0x4E3DC, 0x6C bytes: the guards' army +0; a bank's tile word: its index
# bits 13-24, looted bit 25, the visitors' bits 5 + colour), the kinds of a Witch Hut (12), a Tree
# (18) and the Shrines (19-21) in kinds_visited; the Keymaster's Tents' visitors by colour (u8[8]
# +0x4E364); a hero's spells (u8[70] +0x430) and Trees visited (u32 +0x6B); a quest: its vtable
# QUEST_VTABLE + 0x3C x its type (1-9, `quest_create` 0x573240), its texts' variant (i32 +0x38),
# its data from +0x40. The classes: a mine 53, the banks, the Shrines 88-90, a Tree 102, a Witch
# Hut 113, a Seer's Hut 83, a Quest Guard 215; a Keymaster's Tent 10 is a visit rule's.
MINES, MINE_SIZE, BANKS, BANK_SIZE, KEYMASTERS = 0x4E38C, 0x40, 0x4E3DC, 0x6C, 0x4E364
WITCH_KIND, TREE_KIND, SHRINE_KIND = 12, 18, 19
HERO_SPELLS, HERO_TREES, BANK_LOOTED = 0x430, 0x6B, 1 << 25
QUEST_VTABLE, QUEST_VTABLE_SIZE, QUEST_VARIANT = 0x64174C, 0x3C, 0x38
BANK_CLASSES = (16, 24, 25, 84, 85)
CONTENT_CLASSES = (10, 53, 83, 88, 89, 90, 102, 113, 215) + BANK_CLASSES
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
# Hero (E08 T57, batch E7; structs.toml, hero): the spell points (i16 +0x18), the status flags
# (u32 +0x105: bit 21 a Warrior's Tomb's penalty taken, bit 12 a looted Pyramid's), the morale
# and luck bonuses until the next battle's end (i8 +0x11A, +0x11B).
SPELL_POINTS, STATUS, MORALE_BONUS, LUCK_BONUS = 0x18, 0x105, 0x11A, 0x11B
# Game (E08 T57; structs.toml, game): the Universities' vector (begin +0x4E3CC, end +0x4E3D0),
# four i32 skills each, indexed by a University's tile word's bits 13-24 (university_skills
# 0x405E10).
UNIVERSITIES, UNIVERSITY_SIZE = 0x4E3CC, 16
# The signs' and Ocean Bottles' messages (E08 T68b; structs.toml, game +0x4E37C): a vector (begin,
# end), 20 bytes a record, indexed by the object's word: a "has text" byte (+0x0), then the text as
# the game's string, its characters' pointer at +0x8 and its length at +0xC (load_message_body
# 0x501270). A tile's terrain (map_item +0x4; 8 water).
MESSAGES, MESSAGE_RECORD, MESSAGE_TEXT, MESSAGE_LENGTH = 0x4E37C, 20, 0x8, 0xC
TILE_TERRAIN, WATER = 0x4, 8
# A message box's pictures (E08 T57, observed under Wine at a Treasure Chest; message_box_build
# 0x4F6990): a picture item (PICTURE_VTABLE) with its caption, a text item whose box starts at the
# picture's bottom; a choice's frames (vtable 0x63FE1C) are items 0x7809 + n (the left 0x7809,
# the right 0x780A), the window's result when chosen. The box's frame pieces are pictures of
# dialgbox.def.
CHOICE_VTABLE, CHOICE_FIRST, TEXT_VTABLE = 0x63FE1C, 0x7809, 0x642DB0
FRAME_DEFS = ("dialgbox.def",)
# Player: the selected hero (i32 +0x4, -1 none), the hero count (i8 +0x1) and the heroes (i32[8]
# +0x8, the hero list's order), the resources (i32[7] +0x9C).
RESOURCES = ("wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold")
# Adventure manager: the arrows of the selected hero's path (a pointer +0x48 to a u16 per tile,
# level by level: the underground's after the surface's, E17 H2), the view's top-left tile (+0xE4)
# and the tile under the mouse (+0xE8), both packed (unpack; the level in bit 26).
PATH, VIEW, MOUSE_TILE = 0x48, 0xE4, 0xE8
# Adventure manager +0x4C: a path shown (not 0: the view's passes 6 and 8 draw the arrows of +0x48,
# adventure_view_draw 0x40F5F2; E24 M1 T5b2, design §2.10).
PATH_SHOWN = 0x4C
# The adventure screen's panels (E24 M3 T2a; design §4.1 of E24): a player's towns (i8 +0x3E their
# number, i8 +0x3F the selected one, -1 none, i8[] +0x40 their indices in the town list's order); a
# hero asleep (u8 +0x11C); the adventure dialog (the manager's +0x44): the lists' first slots (i32
# +0x64 the heroes', +0x68 the towns'); the seer huts' and quest guards' quests (vectors of 19-byte
# and 5-byte records at game +0x1FBD4 and +0x1FBE4: the quest at +0, the visitors' bits at +4;
# quest_log_state 0x4039B0).
PLAYER_TOWNS, PLAYER_TOWN, PLAYER_TOWN_LIST, HERO_ASLEEP = 0x3E, 0x3F, 0x40, 0x11C
ADVENTURE_DIALOG, HERO_TOP, TOWN_TOP = 0x44, 0x64, 0x68
QUEST_VECTORS = ((0x1FBD4, 19), (0x1FBE4, 5))
# The lists' contents (E24 M3 T4a; design §4.5 of E24): a hero's portrait (u8 +0x34, an index into
# hero_info 0x679DD0); the towns (game +0x21614 their vector's first, 0x168 bytes a town): a town's
# day (u8 +0x2, a building built in it today), faction (i8 +0x4), place (u8 +0x5, +0x6, +0x7: its
# entrance, two columns left of its anchor, and level) and every built building (u64 +0x158; the Fort bit 7 kept by a Citadel and a
# Castle).
HERO_PORTRAIT = 0x34
TOWNS, TOWN_SIZE, TOWN_BUILT_TODAY, TOWN_FACTION, TOWN_PLACE, TOWN_EVERY_BUILT = (
    0x21614, 0x168, 0x2, 0x4, 0x5, 0x158)
FORT_BIT = 1 << 7
# The status window (E24 M3 T5b1; design §4.8 of E24): the kind shown (the manager's +0x394, 3 a
# hero's, 4 a town's; status_window_choose 0x415DE0) and its items, ids from 2000, in the
# adventure dialog's list: a hero's morale and luck (DEF items 2008 and 2009, frame the bonus + 3),
# a town's gold income (text 2006) and its silo's resources (DEF items 2007, a frame each). A
# town's garrison hero (i32 +0xC, -1 none), its own army (seven types i32 +0xE0, seven counts),
# its name (char* +0xC8) and its buildings standing (u64 +0x150: an upgrade clears what it
# replaces, unlike the every-built set +0x158).
STATUS_SHOWN, STATUS_ITEMS = 0x394, range(2000, 2100)
STATUS_MORALE, STATUS_LUCK, STATUS_SILO, STATUS_INCOME = 2008, 2009, 2007, 2006
TOWN_GARRISON_HERO, TOWN_ARMY, TOWN_NAME, TOWN_STANDING = 0xC, 0xE0, 0xC8, 0x150
# E24 M3 T5b2 (design §4.9): the kind asked (the manager's +0x398) and a timed kind's end (+0x39C,
# a clock time; status_window_set_kind 0x415D60), the kind none (8); the date's window's picture
# (DEF item 2001, its frame the window's step); the kingdom's: the viewer's towns by their halls
# (their every-built sets +0x158: the Capitol 13, the City Hall 12, the Town Hall 11, tested in
# that order, else a village), the players alive (game +0x1F636, a byte each: out of the game)
# by their teams (+0x1F879) against the shown player's (0x69778C).
STATUS_ASKED, STATUS_END, STATUS_NONE, STATUS_DATE_DEF = 0x398, 0x39C, 8, 2001
OUT_OF_GAME, TEAMS, SHOWN_PLAYER, HALL_BITS = 0x1F636, 0x1F879, 0x69778C, (13, 12, 11)
MOVE_HERO, NEXT_HERO, QUEST_LOG = 7, 11, 5
# A path arrow's frame: 0 none, 1 to 25 a move today (green), 26 to 50 a later one (red).
GREEN_FRAMES = range(1, 26)
# The battle animations' next frame time (0x4F82F0 waits for it); on the adventure map the
# view's redraw deadline (adventure_manager_procedure 0x4087B0; `map_due`, E24 M1 T3b2).
FRAME_DEADLINE = 0x698998
IDLE_STEP_TIME = 0x6989B8  # the idle step's next time (0x4960D0)
COMBAT_SPEED = 0x6987EC  # the setting, 0 to 2 (factors 0x63CF7C)
# The adventure map's speeds (E24 M2 T1a2, design §3.4; the registry's "Window Scroll Speed" and
# "Walk Speed", settings_read 0x50B7B0): the scroll's, 0 to 2 (the tiles a step: 1, 2, 3 by
# 0x63A66C), read at each scroll (view_scroll 0x4195C0); the hero's walk, 1 to 4 (M2 T1b). And
# the scroll's last time (0x691674: the edge scroll waits 70 ms from it, 0x419820).
SCROLL_SPEED, WALK_SPEED, SCROLL_TIME = 0x69878C, 0x69875C, 0x691674
# The walk (E24 M2 T1b, design §3.5): the manager's view offset (i32 +0xF4, +0xF8: added to every
# tile's place, written by the walk's animation 0x480380 only), the moving hero's group (i32
# +0x1FC: 0 to 4 standing, 5 to 9 walking) and frame (i32 +0x200: stepped by pass 9 after it drew
# it); a hero's movement points (i32 +0x4D of its record) and facing (u8 +0x47, HERO_FACING).
VIEW_OFFSET, WALK_GROUP, WALK_FRAME, HERO_POINTS = 0xF4, 0x1FC, 0x200, 0x4D
# Battle manager: records per side +0x54BC, the stacks from +0x54CC (21 per side, 0x6EE8 apart,
# 0x548 each), the idle step's counter +0x13FFC.
SIDE_COUNTS, STACKS, SIDE_STRIDE, STACK_SIZE, STEP_COUNTER = 0x54BC, 0x54CC, 0x6EE8, 0x548, 0x13FFC
# The battlefield (E08 T32b1; structs.toml, battle_manager): the terrain (i32 +0x5394, set by
# 0x4643B0) and the special field (i32 +0x53C0, -1 none); since E08 T68a the ship flag (u8
# +0x53C6: a hero in a boat, a Shipwreck or a Derelict Ship; the deck picture).
BATTLE_TERRAIN, SPECIAL_FIELD, ON_SHIP = 0x5394, 0x53C0, 0x53C6
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
SPELL_CURSOR_NEXT = 0x69CA1C  # its spell cursor's next frame (0x50D804, a 100 ms grid; E17 M5 T3b)
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
# Each side's owner and controller (E08 T79a, batch E9a's E9-2; battle_side_setup 0x4639F0,
# structs.toml): the owner (i32 +0x54A8: the attacking hero's +0x22, the defender's the setup's
# argument, -1 none), whether that player is human (u8 +0x54A4, player_id_is_human 0x4CE940) and
# human on this machine (u8 +0x54A6, player_id_is_human_here 0x4CE970), 0 for a side without one.
SIDE_OWNERS, SIDE_HUMAN, SIDE_HUMAN_HERE = 0x54A8, 0x54A4, 0x54A6
# Auto combat on (i32 +0x132C4; the button, control 2004, toggles it: structs.toml).
AUTO_COMBAT = 0x132C4
# The scenario list's dialog while it is open (E17 M5 T8b2, R7-8; `select_scenario_dialog_pointer`
# 0x69FBE8, map.toml, H3API's H3SelectScenarioDialog): two arrays of eight scenario player rows
# (+0x1064, +0x1444; 0x7C bytes each), each row's starting bonus an i32 at +0x6C, stepped by the
# advanced options' right arrow 3 (Random), 0 (Artifact), 1 (Gold); 2 (Resource) by H3API, not
# seen. Observed in the running game: on a two-player map the human's row 0 is the first array's
# entry 0, the computer's row 1 the second's entry 1; every other entry stays 3.
SCENARIO_DIALOG_POINTER, SCENARIO_ROWS, SCENARIO_ROW_SIZE, SCENARIO_BONUS = (
    0x69FBE8, (0x1064, 0x1444), 0x7C, 0x6C)
BONUS_RANDOM, BONUS_ARTIFACT, BONUS_GOLD, BONUS_RESOURCE = 3, 0, 1, 2
# The battle's log (E17 M3 T6b, design §9.8): the combat dialog (a pointer at the manager's
# +0x132FC) holds the entries (a vector of string pointers: begin +0x58, end +0x5C), the new
# entries' count +0x64, the first shown +0x68 and the time it last showed them +0x6C (the
# game's clock); its panel (+0x70) the log's text item (+0x34), whose string record at +0x30 is
# what the item shows, the status line's text included.
COMBAT_DIALOG, LOG_ENTRIES, LOG_FRESH, LOG_FIRST, LOG_TIME, LOG_PANEL, PANEL_LOG_ITEM = (
    0x132FC, 0x58, 0x64, 0x68, 0x6C, 0x70, 0x34)
# The dialog item the mouse last entered (`hovered_item` 0x6AAC68, map.toml; E17 M4 T2c-2: an
# item writes its hint only on entering, 0x5FE747; forgotten when it is destroyed): read per
# step since M4 T3b, read-only. Its dialog (+0x4) and hint (char* +0x20) by H3API's H3DlgItem
# (`parent`, `hint`), observed in the running game (the item's dialog lists it).
HOVERED_ITEM, ITEM_DIALOG, ITEM_HINT = 0x6AAC68, 0x4, 0x20
# Stack fields (structs.toml, battle_stack): name -> offset (all i32).
FIELDS = {
    "draw_layer": 0x8,
    "creature": 0x34,
    "hex": 0x38,
    "group": 0x3C,
    "frame": 0x40,
    "facing": 0x44,
    "count": 0x4C,
    "army_slot": 0x5C,  # E08 T57 (batch E7's openings): the army slot the stack came from
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
    """The Wine process that maps Heroes3.exe at its image base, 0x400000, in this instance's
    prefix (E17 M5 T8a: with two instances side by side the first game found was the other's,
    read and written; the runbook's "Two instances")."""
    prefix = os.environ.get("WINEPREFIX", str(Path.home() / ".wine-h3"))
    found = subprocess.run(["pgrep", "-f", "Heroes3.exe"], capture_output=True, text=True)
    for pid in found.stdout.split():
        try:
            with open(f"/proc/{pid}/environ", "rb") as environ:
                if f"WINEPREFIX={prefix}".encode() not in environ.read().split(b"\0"):
                    continue
            with open(f"/proc/{pid}/maps") as maps:
                if any(line.startswith("00400000-") for line in maps):
                    return int(pid)
        except OSError:
            continue
    raise RuntimeError(f"no running Heroes3.exe in the prefix {prefix}")


def starting_bonuses(mem):
    """The starting bonus of every scenario player row of the scenario list's dialog, both arrays
    (16 values; E17 M5 T8b2, R7-8); an error when the dialog is not open."""
    dialog = mem.u32(SCENARIO_DIALOG_POINTER)
    if not dialog:
        raise RuntimeError("the scenario list's dialog is not open: no starting bonus to read")
    return [mem.i32(dialog + rows + i * SCENARIO_ROW_SIZE + SCENARIO_BONUS)
            for rows in SCENARIO_ROWS for i in range(8)]


class Mem:
    def __init__(self, pid=None):
        self.pid = pid or game_pid()
        self.file = open(f"/proc/{self.pid}/mem", "rb", buffering=0)

    def read(self, address, size):
        self.file.seek(address)
        return self.file.read(size)

    def write_u32(self, address, value):
        """Writes a u32 into the game's memory: the harness's own timers (M3 T6c-2,
        `align_mouse_poll`; E24 M2 T1a1: the palette tick's and the redraw's next times), the
        game's settings (E24 M2 T1a2: a sequence's speeds) and, through `write_points` and
        `write_frame_steps`, a sequence's set-up at its 0 (E24 M2 T1b; M2 T6a, the reviewer's
        R3-8: game state, before the sequence's 0 only)."""
        with open(f"/proc/{self.pid}/mem", "r+b", buffering=0) as f:
            f.seek(address)
            f.write(struct.pack("<I", value))

    def align_mouse_poll(self, now):
        """The mouse manager reads the mouse on a 33 ms grid of the game's clock (`mouse_poll`
        0x50D680: its next time 0x69CA18), whose phase the menus before a battle leave at random:
        set to `now` (the opening), the grid runs from the opening in every run (M3 T6c-2,
        design §9.10), so the frames a sequence keeps where the cursor moves keep their times."""
        self.write_u32(MOUSE_POLL_NEXT, now)
        # The spell cursor's frames (`crspell.DEF`, the mouse manager's type 3) step on the poll's
        # second grid, 100 ms, from the same phase (E17 M5 T3b, design §7.18): only a type-3
        # cursor reads it.
        self.write_u32(SPELL_CURSOR_NEXT, now)

    def walls(self):
        """A siege's 18 parts' picture states (+0x13FA8, i32 each), their hit points (+0x13F60)
        and the gate's state (+0x53A4: 3 closed, 2 moving, 1 open, 0 destroyed; E17 design
        §10.1)."""
        m = self.manager()
        return (list(struct.unpack("<18i", self.read(m + WALL_STATES, 72))),
                list(struct.unpack("<18i", self.read(m + WALL_HIT_POINTS, 72))),
                self.i32(m + GATE_STATE))

    def siege(self):
        """A siege's moving parts (E17 M4 T2b, design §10.2): its turrets' `[facing, group,
        frame]` (None where the fort sets none up), its parts' picture states and the gate's;
        None outside a siege (no fort)."""
        m = self.manager()
        if not m or self.i32(m + FORT) <= 0:
            return None
        turrets = []
        for k in range(3):
            base = m + TURRETS + k * TURRET_SIZE
            built = self.u32(base + TURRET_DEF) != 0
            turrets.append(list(struct.unpack("<3i", self.read(base + TURRET_POSE, 12)))
                           if built else None)
        states, _, gate = self.walls()
        return {"turrets": turrets, "walls": states, "gate": gate}

    def write_walls(self, states, gate):
        """Writes a siege's wall states (E17 M4 T1b, the wall states as stills): each part's
        picture state and the gate's state, what the redraw reads (E17 design §10.1). Written
        after the siege's setup and before the battle's first redraw (`h3drive.into_battle`),
        the first frame shows them. The hit points stay: a tower's or the keep's written 0 makes
        the original skip its turn (its first frame then the attacker's, the record's the
        keep's)."""
        # A part's picture is read by its state (five each), the gate's by its four (E17 M4 T3a).
        if len(states) != 18 or not all(0 <= s <= 4 for s in states) or not 0 <= gate <= 3:
            raise ValueError(f"wall states {states}, gate {gate}: 18 of 0 to 4, the gate 0 to 3")
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
        none) and since E08 T68a its ship flag (0 or 1); None outside a battle."""
        manager = self.manager()
        if not manager:
            return None
        terrain, special = self.i32(manager + BATTLE_TERRAIN), self.i32(manager + SPECIAL_FIELD)
        return {"terrain": TERRAINS[terrain] if 0 <= terrain < len(TERRAINS) else str(terrain),
                "special": (SPECIAL_FIELDS[special] if 0 <= special < len(SPECIAL_FIELDS)
                            else None if special < 0 else str(special)),
                "ship": self.read(manager + ON_SHIP, 1)[0]}

    def battle_sides(self):
        """Each side's owner (a colour, -1 none) and whether it is a human player's and a human's
        on this machine (E08 T79a, batch E9a's E9-2: who plays an owned mine's guards); None
        outside a battle."""
        manager = self.manager()
        if not manager:
            return None
        owners = struct.unpack("<2i", self.read(manager + SIDE_OWNERS, 8))
        human, here = self.read(manager + SIDE_HUMAN, 2), self.read(manager + SIDE_HUMAN_HERE, 2)
        return [{"owner": owners[s], "human": human[s], "here": here[s]} for s in (0, 1)]

    def auto_combat_on(self):
        """Whether auto combat is on (E08 T79a; battle manager +0x132C4), False outside a
        battle."""
        manager = self.manager()
        return bool(manager) and self.i32(manager + AUTO_COMBAT) != 0

    def obstacle_hexes(self):
        """The hexes an obstacle blocks (E08 T68a, batch E8's openings): the hex records' blocked
        flag (0x2) on a hex no stack stands on, in the hexes' order."""
        return [h["hex"] for h in self.hexes() if h["flags"] & 0x2 and h["stack"] is None]

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

    def hovered_item(self):
        """The dialog item the mouse last entered (E17 M4 T3b; `hovered_item` 0x6AAC68): its
        address, id, vtable (its kind), its dialog's address and whether that is the battle's
        combat dialog, its box relative to the dialog (x, y, width, height), its state and its
        hint (None for none or unreadable); None while no item is hovered. Read-only."""
        item = self.u32(HOVERED_ITEM)
        if not item:
            return None
        raw = self.read(item, ITEM_TEXT)
        dialog = struct.unpack_from("<I", raw, ITEM_DIALOG)[0]
        manager = self.manager()
        hint = struct.unpack_from("<I", raw, ITEM_HINT)[0]
        try:
            text = self.c_string(hint, 256) if 0x10000 <= hint < 0x80000000 else None
        except (OSError, UnicodeDecodeError):
            text = None
        return {"at": item, "id": struct.unpack_from("<h", raw, ITEM_ID)[0],
                "vtable": struct.unpack_from("<I", raw, 0)[0], "dialog": dialog,
                "combat": bool(manager) and dialog == self.u32(manager + COMBAT_DIALOG),
                "box": list(struct.unpack_from("<2h2H", raw, ITEM_BOX)),
                "state": struct.unpack_from("<H", raw, ITEM_STATE)[0], "hint": text}

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
            # E08 T79b (batch E9b): the day's income and the Mystical Gardens' mask.
            "income": dict(zip(RESOURCES, struct.unpack_from("<7i", raw, PLAYER_INCOME))),
            "gardens": struct.unpack_from("<I", raw, PLAYER_GARDENS)[0],
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
            # E08 T57 (batch E7): the spell points, the status flags, the morale and luck bonuses.
            "spell_points": struct.unpack_from("<h", raw, SPELL_POINTS)[0],
            "status": struct.unpack_from("<I", raw, STATUS)[0],
            "morale_bonus": struct.unpack_from("<b", raw, MORALE_BONUS)[0],
            "luck_bonus": struct.unpack_from("<b", raw, LUCK_BONUS)[0],
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
        """The selected hero's path as the map shows it: {(x, y, level): "green" | "red"}; since
        E08 T68a {} while the manager's +0x4C shows no path (as path_arrows): a click that finds
        no path leaves the last path's arrows in the array (batch E8a's second run read E8-7c's
        path as E8-7d's)."""
        if not self.manager_field(PATH_SHOWN):
            return {}
        side = self.i32(self.game() + MAP_SIDE)
        tiles = side * side * self.levels()
        frames = struct.unpack(f"<{tiles}H", self.read(self.manager_field(PATH), 2 * tiles))
        return {(i % side, i // side % side, i // (side * side)):
                "green" if f in GREEN_FRAMES else "red" for i, f in enumerate(frames) if f}

    def path_arrows(self):
        """The path the view draws (E24 M1 T5b2; design §2.10), as `render --live` takes it:
        [x, y, level, arrow] for each tile with an arrow, level by level and rows then columns;
        none while the manager's +0x4C shows no path."""
        if not self.manager_field(PATH_SHOWN):
            return []
        side = self.i32(self.game() + MAP_SIDE)
        tiles = side * side * self.levels()
        arrows = struct.unpack(f"<{tiles}H", self.read(self.manager_field(PATH), 2 * tiles))
        return [[i % side, i // side % side, i // (side * side), a]
                for i, a in enumerate(arrows) if a]

    def open_tiles(self, colour, z=0, water=None):
        """The tiles of a level a hero of `colour` may walk to without visiting or meeting
        anything (E08 T57, the harness's approach to an unexplored target): explored by that
        player, no object's class (+0x1E 0), not closed (+0xD bit 0x01: rock, or an object
        blocks it) and outside every monster's guard (the vision word's bit 8). Since E08 T68b
        `water = True` keeps only water tiles (+0x4 terrain 8: a hero in a boat sails only
        there), `False` only land."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        count = side * side
        raw = self.read(self.u32(game + TILES) + TILE_SIZE * count * z, TILE_SIZE * count)
        words = struct.unpack(f"<{count}H", self.read(self.u32(TILE_VISION_POINTER) + 2 * count * z,
                                                      2 * count))
        return {(i % side, i // side) for i in range(count)
                if words[i] >> colour & 1 and not words[i] & 0x100
                and struct.unpack_from("<i", raw, TILE_SIZE * i + 0x1E)[0] == 0
                and not raw[TILE_SIZE * i + 0xD] & 1
                and (water is None or (raw[TILE_SIZE * i + TILE_TERRAIN] == WATER) == water)}

    def explored_rows(self, colour):
        """A player's explored bits of every tile (E24 M1 T5b2), as `render --live` takes them: a
        string per row of each level in turn, `1` explored, `0` not (tile_vision_word 0x4F79B0)."""
        side = self.i32(self.game() + MAP_SIDE)
        count = side * side * self.levels()
        words = struct.unpack(f"<{count}H", self.read(self.u32(TILE_VISION_POINTER), 2 * count))
        bits = "".join("1" if w >> colour & 1 else "0" for w in words)
        return [bits[i:i + side] for i in range(0, count, side)]

    def date(self):
        """(month, week, day of the week), each from 1 (E17 H2)."""
        day, week, month = struct.unpack("<3H", self.read(self.game() + DATE, 6))
        return month, week, day


    def map_kind(self):
        """The game's version as the rules test it (E08 T57; game +0x1F698): RoE, AB or SoD."""
        kind = self.i32(self.game() + MAP_KIND)
        return MAP_KINDS[kind] if 0 <= kind < len(MAP_KINDS) else str(kind)

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

    def walk(self):
        """The walk's drawing state (E24 M2 T1b, design §3.5): the moving hero's group and frame
        (the frame pass 9 draws next) and the view's pixel offset x, y."""
        manager = self.u32(ADVENTURE_MANAGER_POINTER)
        return [self.i32(manager + WALK_GROUP), self.i32(manager + WALK_FRAME),
                self.i32(manager + VIEW_OFFSET), self.i32(manager + VIEW_OFFSET + 4)]

    def selected(self):
        """The current player's selected hero (T1b): its index (-1 none), and its tile and facing
        [x, y, level, facing] (empty without one)."""
        index = self.player(self.current_player())["selected"]
        if index < 0:
            return index, []
        record = self.game() + HEROES + HERO_SIZE * index
        x, y, level = struct.unpack("<3h", self.read(record, 6))
        return index, [x, y, level, self.read(record + HERO_FACING, 1)[0]]

    def points(self, index):
        """A hero's movement points left (E24 M3 T4b: a sequence's timeline; i32 +0x4D)."""
        return self.i32(self.game() + HEROES + HERO_SIZE * index + HERO_POINTS)

    def write_points(self, index, points):
        """A hero's movement points (T1b: a sequence's `points`)."""
        self.write_u32(self.game() + HEROES + HERO_SIZE * index + HERO_POINTS, points)

    def write_mana(self, index, mana):
        """A hero's mana (E24 M3 T4a: a situation's hero's `mana`; i16 +0x18)."""
        record = self.game() + HEROES + HERO_SIZE * index
        self.write_bytes(record + SPELL_POINTS, struct.pack("<h", mana))

    def write_skills(self, index, skills):
        """A hero's primary skills (E24 M3 T5b1: a situation's hero's `skills`; i8[4] +0x476)."""
        record = self.game() + HEROES + HERO_SIZE * index
        self.write_bytes(record + PRIMARY, struct.pack("<4b", *skills))

    def write_bonuses(self, index, morale, luck):
        """A hero's morale and luck until its next battle (E24 M3 T5b1: a situation's hero's
        `morale` and `luck`; i8 +0x11A, +0x11B)."""
        record = self.game() + HEROES + HERO_SIZE * index
        self.write_bytes(record + MORALE_BONUS, struct.pack("<2b", morale, luck))

    def write_army(self, at, army):
        """Seven army slots at `at` (E24 M3 T5b1: a situation's hero's `army`, a hero's +0x91, a
        town's `garrison`, its +0xE0): the types (i32, -1 none), then the counts; `army` the
        slots' [type, count] in order, the rest left empty."""
        slots = list(army) + [[-1, 0]] * (7 - len(army))
        self.write_bytes(at, struct.pack("<14i", *[s[0] for s in slots], *[s[1] for s in slots]))

    def hero_record(self, index):
        """The address of the hero `index`'s record."""
        return self.game() + HEROES + HERO_SIZE * index

    def status_items(self):
        """The status window's items in the adventure dialog (E24 M3 T5b1; ids from 2000): by id,
        a list of each one's frame (a DEF item's, i32 +0x34) and text (None for an item that holds
        none), in the dialog's list's order."""
        dialog = self.manager_field(ADVENTURE_DIALOG)
        items, item, seen = {}, self.u32(dialog + FIRST_ITEM), set()
        while item and item not in seen and len(seen) < 512:
            seen.add(item)
            raw = self.read(item, ITEM_TEXT + 12)
            ident = struct.unpack_from("<H", raw, ITEM_ID)[0]
            if ident in STATUS_ITEMS:
                items.setdefault(ident, []).append(
                    (struct.unpack_from("<i", raw, PICTURE_FRAME)[0], self.item_text(raw)))
            item = struct.unpack_from("<I", raw, NEXT_ITEM)[0]
        return items

    def status(self, colour):
        """What the status window shows (E24 M3 T5b1; design §4.8), as `render --live`'s `status`
        takes it once the harness named the town's name: `{"kind": <the manager's +0x394>}`, and
        for a hero's (3) the hero (its record's index), its primary skills as stored, its morale
        and luck (the window's DEF items' frames less 3: the bonuses 0x4E39B0 and 0x4E36C0
        computed) and its army ([type, count] for each of the seven slots, type -1 empty); for a
        town's (4) its name (the record's text), its buildings standing (+0x150's bits), whether
        a hero is in its garrison (+0xC), the army the window shows (the garrison hero's or the
        town's own, 0x5C1460), its silo's resources (the frames of the items 2007, in the
        window's order) and its gold income (the item 2006's text, 0x5BF600 computed); since M3
        T5b2 for the date's (1) its picture's frame, for the kingdom's (2) the towns by their
        halls and the players alive of the shown player's team and the others."""
        kind = self.i32(self.u32(ADVENTURE_MANAGER_POINTER) + STATUS_SHOWN)
        raw = self.read(self.game() + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        items = self.status_items()

        def army(at):
            data = self.read(at, 56)
            return [list(s) for s in zip(struct.unpack_from("<7i", data, 0),
                                         struct.unpack_from("<7i", data, 28))]

        if kind == 3:
            index = struct.unpack_from("<i", raw, 0x4)[0]
            record = self.hero_record(index)
            return {"kind": kind, "hero": index,
                    "skills": list(struct.unpack("<4b", self.read(record + PRIMARY, 4))),
                    "morale": items[STATUS_MORALE][0][0] - 3,
                    "luck": items[STATUS_LUCK][0][0] - 3, "army": army(record + ARMY)}
        if kind == 4:
            index = struct.unpack_from("<b", raw, PLAYER_TOWN)[0]
            record = self.u32(self.game() + TOWNS) + TOWN_SIZE * index
            data = self.read(record, TOWN_SIZE)
            standing = struct.unpack_from("<Q", data, TOWN_STANDING)[0]
            hero = struct.unpack_from("<i", data, TOWN_GARRISON_HERO)[0]
            shown = army(self.hero_record(hero) + ARMY if hero >= 0 else record + TOWN_ARMY)
            silo = [frame for frame, _ in reversed(items.get(STATUS_SILO, []))]
            return {"kind": kind,
                    "name": self.c_string(struct.unpack_from("<I", data, TOWN_NAME)[0], 64),
                    "buildings": [b for b in range(64) if standing >> b & 1],
                    "garrison": int(hero >= 0), "army": shown, "silo": silo,
                    "income": int(items[STATUS_INCOME][0][1])}
        if kind == 1:
            # The date's (E24 M3 T5b2): its picture's frame, the window's step; the date and the
            # picture's kind the game's date gives.
            return {"kind": kind, "frame": items[STATUS_DATE_DEF][0][0]}
        if kind == 2:
            return {"kind": kind, **self.realm(colour)}
        return {"kind": kind}

    def realm(self, colour):
        """The kingdom's status window as the game would build it now (E24 M3 T5b2, since T6b2
        on its own: what a click on the window shows): the towns of `colour` by their halls
        (villages, Town Halls, City Halls, Capitols), the players alive of the shown player's team
        (itself among them) and the others, each by colour."""
        halls = [0, 0, 0, 0]
        for index in self.town_list(colour):
            record = self.u32(self.game() + TOWNS) + TOWN_SIZE * index
            built = struct.unpack("<Q", self.read(record + TOWN_EVERY_BUILT, 8))[0]
            hall = next((3 - i for i, b in enumerate(HALL_BITS) if built >> b & 1), 0)
            halls[hall] += 1
        out = self.read(self.game() + OUT_OF_GAME, 8)
        teams = struct.unpack("<8b", self.read(self.game() + TEAMS, 8))
        shown = self.i32(SHOWN_PLAYER)
        alive = [p for p in range(8) if out[p] == 0]
        allies = [p for p in alive if shown >= 0 and teams[p] == teams[shown]]
        return {"halls": halls, "allies": allies,
                "enemies": [p for p in alive if p not in allies]}

    def ask_status(self, kind, end, rebuild=False):
        """Asks the status window for a kind until the clock time `end` (E24 M3 T5b2; design
        §4.9: the manager's +0x398 and +0x39C, as status_window_set_kind 0x415D60 sets them); with
        `rebuild` the kind shown set to none, so that the next view draw builds the window afresh
        (the date's from its first picture) even when it shows that kind already."""
        manager = self.u32(ADVENTURE_MANAGER_POINTER)
        self.write_bytes(manager + STATUS_ASKED, struct.pack("<iI", kind, end % (1 << 32)))
        if rebuild:
            self.write_bytes(manager + STATUS_SHOWN, struct.pack("<i", STATUS_NONE))

    def status_shown(self):
        """The status window's kind shown (the manager's +0x394; E24 M3 T5b2)."""
        return self.i32(self.u32(ADVENTURE_MANAGER_POINTER) + STATUS_SHOWN)

    def town(self, index):
        """The town `index` of the game's towns (E24 M3 T4a): its record's address, faction,
        whether it has a Fort (its every-built set), whether it built today and its place [x, y,
        level]."""
        record = self.u32(self.game() + TOWNS) + TOWN_SIZE * index
        raw = self.read(record, TOWN_SIZE)
        built = struct.unpack_from("<Q", raw, TOWN_EVERY_BUILT)[0]
        return {"at": record, "faction": struct.unpack_from("<b", raw, TOWN_FACTION)[0],
                "fort": int(built & FORT_BIT != 0), "built": int(raw[TOWN_BUILT_TODAY] != 0),
                "place": list(raw[TOWN_PLACE:TOWN_PLACE + 3])}

    def town_list(self, colour):
        """A player's towns by their indices in the game's towns, in the town list's order."""
        raw = self.read(self.game() + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        towns = struct.unpack_from("<b", raw, PLAYER_TOWNS)[0]
        return list(struct.unpack_from(f"<{towns}b", raw, PLAYER_TOWN_LIST))

    def write_resources(self, colour, amounts):
        """A player's seven resources in the original's order (E24 M3 T5a: a situation's
        `resources`; i32[7] at the player's +0x9C)."""
        record = self.game() + PLAYERS + PLAYER_SIZE * colour
        self.write_bytes(record + 0x9C, struct.pack("<7i", *amounts))

    def write_date(self, month, week, day):
        """The game's date (E24 M3 T5a: a situation's `date`; the day of the week, the week and
        the month, u16 each from +0x1F63E)."""
        self.write_bytes(self.game() + DATE, struct.pack("<3H", day, week, month))

    def write_town_built(self, index, built=1):
        """A town's day (E24 M3 T4a: a situation's town's `built`; u8 +0x2)."""
        self.write_bytes(self.town(index)["at"] + TOWN_BUILT_TODAY, bytes([built]))

    def kingdom(self, colour):
        """What the adventure screen's panels show of a player's kingdom (E24 M3 T2a; design
        §4.1), as `render --live`'s `kingdom` takes it: its heroes in the hero list's order, each
        [asleep, movable, portrait, points, mana] (movable: points left; a step leaves 0 or at least
        the cheapest step from the new tile, hero_move_step 0x481182, so a hero with points left can
        move: *provisional*; the rest since M3 T4a, §4.5), its towns in the town list's order, each
        [faction, fort, built today] (T4a), the one selected ([1, the hero's place in the list],
        [2, the town's] or [0, 0]), the lists' first slots and whether a quest it visited is in
        its log (0x4039B0; the quest's text, which every quest has, not checked); since M3 T5a its
        seven resources in the original's order and the game's date [month, week, day] (design
        §4.7: the resource bar)."""
        raw = self.read(self.game() + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        count = struct.unpack_from("<b", raw, 0x1)[0]
        heroes = list(struct.unpack_from(f"<{count}i", raw, 0x8))
        selected = struct.unpack_from("<i", raw, 0x4)[0]
        towns = struct.unpack_from("<b", raw, PLAYER_TOWNS)[0]
        town_list = list(struct.unpack_from(f"<{towns}b", raw, PLAYER_TOWN_LIST))
        town = struct.unpack_from("<b", raw, PLAYER_TOWN)[0]
        rows = []
        for index in heroes:
            record = self.game() + HEROES + HERO_SIZE * index
            asleep = self.read(record + HERO_ASLEEP, 1)[0]
            points = self.i32(record + HERO_POINTS)
            portrait = self.read(record + HERO_PORTRAIT, 1)[0]
            mana = struct.unpack("<h", self.read(record + SPELL_POINTS, 2))[0]
            rows.append([int(asleep != 0), int(points > 0), portrait, points, mana])
        town_rows = [[t["faction"], t["fort"], t["built"]] for t in map(self.town, town_list)]
        if selected in heroes:
            chosen = [1, heroes.index(selected)]
        elif town in town_list:
            chosen = [2, town_list.index(town)]
        else:
            chosen = [0, 0]
        dialog = self.manager_field(ADVENTURE_DIALOG)
        tops = [self.i32(dialog + HERO_TOP), self.i32(dialog + TOWN_TOP)]
        # The Quest Log's state as shown (E24 M3 T6a2b): `quest_log_state` 0x4039B0 sets it at the
        # map's start and after a visit, not when the harness writes a quest's visitors.
        quests = self.item_states([QUEST_LOG])[QUEST_LOG] & 0x20 == 0
        resources = list(struct.unpack_from("<7i", raw, 0x9C))
        return {"heroes": rows, "towns": town_rows, "selected": chosen, "tops": tops,
                "quests": int(quests), "resources": resources, "date": list(self.date())}

    def buttons(self):
        """Move hero's and next hero's states on the adventure screen (E24 M3 T2a; design §4.1),
        `[move, next]`, 1 enabled: the items 7 and 11 of the adventure dialog (the manager's
        +0x44), a state with bit 8 or 0x20 disabled."""
        states = self.item_states([MOVE_HERO, NEXT_HERO])
        return [int(not states[i] & 0x28) for i in (MOVE_HERO, NEXT_HERO)]

    def item_states(self, ids):
        """The states (u16 +0x16) of the adventure dialog's items `ids` (the manager's +0x44),
        by id (E24 M3 T2a; since T6a2b the Quest Log's)."""
        dialog = self.manager_field(ADVENTURE_DIALOG)
        states, item, seen = {}, self.u32(dialog + FIRST_ITEM), set()
        while item and item not in seen and len(seen) < 512 and len(states) < len(ids):
            seen.add(item)
            raw = self.read(item, ITEM_STATE + 2)
            ident = struct.unpack_from("<H", raw, ITEM_ID)[0]
            if ident in ids:
                states[ident] = struct.unpack_from("<H", raw, ITEM_STATE)[0]
            item = struct.unpack_from("<I", raw, NEXT_ITEM)[0]
        return states

    def cursor(self):
        """The cursor (E24 M2 T1a2): its type (0 `crdeflt.DEF`, 1 `cradvntr.DEF`, ...), its
        frame (the edge scroll's 32 to 39 on the map, design §3.4) and the point it is drawn at
        (the mouse's at the last poll, its x even: design §9.10 of E17)."""
        pointer = self.u32(MOUSE_MANAGER)
        return [self.i32(pointer + CURSOR_TYPE), self.i32(pointer + CURSOR_FRAME),
                self.i32(pointer + CURSOR_X), self.i32(pointer + CURSOR_Y)]

    def map_objects(self):
        """The map's objects in its order (E24 M1 T4a): [(x, y, level, template, phase)] from
        their 12-byte records (the anchor, the template's index, the animation phase)."""
        begin, end = struct.unpack("<II", self.read(self.manager_field(MANAGER_MAP) + MAP_OBJECTS, 8))
        count = (end - begin) // OBJECT_RECORD
        raw = self.read(begin, OBJECT_RECORD * count) if count else b""
        return [(raw[i + 4], raw[i + 5], raw[i + 6]) + struct.unpack_from("<HB", raw, i + 8)
                for i in range(0, len(raw), OBJECT_RECORD)]

    def write_frame_steps(self, steps=0):
        """The objects' frame steps (adventure manager +0x100; E24 M2 T1b: `frames` sets them at
        a sequence's 0, the load's count depends on its real time)."""
        self.write_u32(self.u32(ADVENTURE_MANAGER_POINTER) + FRAME_STEPS, steps)

    def write_phases(self, phase=0):
        """Every map object's animation phase (its record's u8 +0xA; E24 M2 T1b: `frames` fixes
        them, the load's are random, design §1.6)."""
        begin, end = struct.unpack("<II", self.read(self.manager_field(MANAGER_MAP) + MAP_OBJECTS, 8))
        for record in range(begin, end, OBJECT_RECORD):
            self.write_bytes(record + 0xA, bytes([phase]))

    def map_templates(self):
        """The map's object templates in memory (E24 M1 T4b): [(DEF, class, subclass)] in order,
        the map's own and then the copies a new game made for its random objects."""
        begin, end = struct.unpack("<II", self.read(self.manager_field(MANAGER_MAP) + MAP_TEMPLATES, 8))
        out = []
        for record in range(begin, end, TEMPLATE_RECORD):
            chars, length = struct.unpack("<II", self.read(record + 4, 8))
            name = self.read(chars, length).decode("cp1252") if length else ""
            out.append((name,) + struct.unpack("<II", self.read(record + TEMPLATE_CLASS, 8)))
        return out

    def template_bytes(self):
        """The map's object templates' records as one byte string (E24 M1 T5b1; E08 T57)."""
        begin, end = struct.unpack("<II", self.read(self.manager_field(MANAGER_MAP)
                                                    + MAP_TEMPLATES, 8))
        return self.read(begin, end - begin) if end > begin else b""

    def object_at(self, x, y, z=0):
        """The object anchored at (x, y, z), read on its entrance tile (E08 T57, batch E7: an
        object's word lives on the tile it is visited from, object_entrance_tile 0x4FEC30, the
        anchor's for a one-tile object, a University's one left of it): its class, subclass,
        word and that tile; without an object anchored there, the tile's own. A class of 0 is
        none (a pickup taken, an emptied event: the tile's object cleared)."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        at = (x, y)
        for ox, oy, oz, template, _ in self.map_objects():
            if (ox, oy, oz) == (x, y, z):
                at = entrance_cell(self.template_bytes(), side, x, y, template) or at
                break
        found = self.tile(*at, z)
        found["tile"] = [at[0], at[1], z]
        return found

    def messages(self):
        """The signs' and Ocean Bottles' message records (E08 T68b; game +0x4E37C): [(has text,
        text)] by the object's word, the text as the map gave it (cp1252)."""
        begin, end = struct.unpack("<II", self.read(self.game() + MESSAGES, 8))
        out = []
        for at in range(begin, end - MESSAGE_RECORD + 1, MESSAGE_RECORD):
            raw = self.read(at, MESSAGE_RECORD)
            pointer, length = struct.unpack_from("<Ii", raw, MESSAGE_TEXT)
            text = self.read(pointer, length).decode("cp1252", errors="replace") \
                if pointer and 0 < length < 4096 else ""
            out.append((bool(raw[0]), text))
        return out

    def universities(self):
        """The game's Universities (E08 T57; game +0x4E3CC): each one's four skill ids, in the
        order a new game set them up (university_setup 0x4C06F0)."""
        begin, end = struct.unpack("<II", self.read(self.game() + UNIVERSITIES, 8))
        count = (end - begin) // UNIVERSITY_SIZE
        raw = self.read(begin, UNIVERSITY_SIZE * count) if count > 0 else b""
        return [list(struct.unpack_from("<4i", raw, UNIVERSITY_SIZE * i)) for i in range(count)]

    def dialog_pictures(self):
        """The topmost dialog's pictures with their captions (E08 T57: a message box's rewards, a
        choice's two sides): [{def, frame, text, box, choice}] in item order, the box's frame
        pieces left out; `choice` the id of the choice frame around the picture (0x7809 the
        left, 0x780A the right), None outside one."""
        items = self.dialog_items()
        frames = [i for i in items if i["vtable"] == CHOICE_VTABLE and i["id"] >= CHOICE_FIRST]
        texts = [i for i in items if i["vtable"] == TEXT_VTABLE and i["text"] is not None]
        out = []
        for item in items:
            if item["vtable"] != PICTURE_VTABLE or not item["def"]:
                continue
            name = self.c_string(item["def"] + DEF_NAME, 13).lower()
            if name in FRAME_DEFS:
                continue
            x, y, width, height = item["box"]
            caption = next((t["text"] for t in texts if abs(t["box"][1] - (y + height)) <= 4
                            and t["box"][0] < x + width and x < t["box"][0] + t["box"][2]), "")
            choice = next((f["id"] for f in frames if f["box"][0] <= x + width // 2
                           < f["box"][0] + f["box"][2] and f["box"][1] <= y + height // 2
                           < f["box"][1] + f["box"][3]), None)
            out.append({"def": name, "frame": item["frame"], "text": caption, "box": item["box"],
                        "choice": choice})
        return out

    def map_owners(self):
        """Every map object's owner as the view passes find it (E24 M1 T5b1, design §2.9), in the
        map's order: -1 none. tile_object_owner 0x40FD20 of the tile object_entrance_tile
        0x4FEC30 gives (the template's first visitable cell on the map, by row from the anchor's
        upwards, each row from the anchor's column leftwards; tile (0, 0) of the surface without
        one): its class and word, a hero's the object under it (the hero record's +0xC, +0x14),
        then the owner of the dwellings 17 and 20 (game +0x4E39C, 0x5C bytes, +0x57), a garrison
        33 (+0x4E3AC, 0x40 bytes), a lighthouse 42 or a mine 53 (+0x4E38C, 0x40 bytes), a town 77
        or 98 (+0x21614, 0x168 bytes, +0x1), a shipyard 87 (the word's low byte); -1 for every
        other class."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        tiles = self.u32(game + TILES)
        templates = self.template_bytes()

        def entrance(x, y, template):
            return entrance_cell(templates, side, x, y, template)

        def vector(offset, size, index, field, signed=True):
            first = self.u32(game + offset)
            value = self.read(first + size * index + field, 1)[0]
            return value - 256 * (signed and value > 127)

        out = []
        for x, y, z, template, _ in self.map_objects():
            at = entrance(x, y, template)
            tx, ty, tz = (at[0], at[1], z) if at else (0, 0, 0)
            raw = self.read(tiles + TILE_SIZE * ((tz * side + ty) * side + tx), TILE_SIZE)
            word, = struct.unpack_from("<i", raw, 0)
            kind, = struct.unpack_from("<i", raw, 0x1E)
            if kind == HERO_OBJECT:
                hero = self.read(game + HEROES + HERO_SIZE * word, HERO_SIZE)
                kind, word = struct.unpack_from("<i", hero, 0xC)[0], struct.unpack_from("<i", hero, 0x14)[0]
            if kind in (17, 20):
                owner = vector(0x4E39C, 0x5C, word, 0x57)
            elif kind == 33:
                owner = vector(0x4E3AC, 0x40, word, 0)
            elif kind in (42, 53):
                owner = vector(0x4E38C, 0x40, word, 0)
            elif kind in (77, 98):
                owner = vector(0x21614, 0x168, word, 1)
            elif kind == 87:
                owner = (word & 0xFF) - 256 * (word & 0x80 != 0)
            else:
                owner = -1
            out.append(owner)
        return out

    def node_cost(self, x, y, z=0):
        """The adventure pathfinder's kind-0 node of a tile (0x699284 +0x24, 30 bytes at ((y +
        2 z h) w + x); E24 M3 T6a1): the movement points the path there costs (+0x18), None when
        not reached (+4 bit 0)."""
        side = self.i32(self.game() + MAP_SIDE)
        nodes = self.u32(self.u32(PATHFINDER_POINTER) + 0x24)
        if not nodes:
            # No run yet (the pathfinder allocates its nodes at its first).
            return None
        raw = self.read(nodes + 30 * ((y + 2 * z * side) * side + x), 30)
        return struct.unpack_from("<H", raw, 0x18)[0] if raw[4] & 1 else None

    def hover(self, colour, hotraits, town_names, z=0, reach=None):
        """The game's facts the mouse-over reads (E24 M3 T6a1; design §4.10), as `render
        --live`'s `[hover]` takes them, for the viewing player `colour` and the view's level `z`:
        per map object the word of its entrance tile (with a hero or boat standing there, the word
        under it: their records' +0x14; -1 without an entrance on the map) and whether it holds
        an army that defends it (a town 98: `town_defended` 0x5BDFE0, a garrison 33: its army);
        the game's towns by index [name record of TownName.txt, type]; per hero of `map_heroes(z)`
        its record index and whether its name is its own (not its type's); the player's heroes in
        its list's order [index, class, own name] and its towns' indices; the teams; the tiles a
        monster guards (the vision words' bit 8, rows per level); `reach`, the selected hero's
        costs of the tiles the pointer looked at ({(x, y): cost}: the pathfinder stops once its
        destination is reached, so each tile's cost is the run's for it, `h3map.look_over_view`),
        and its day points (+0x49). `hotraits` and `town_names`:
        the hero types' names and TownName.txt's, in order (the harness keeps the records'
        numbers, never the text)."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        tiles = self.u32(game + TILES)
        templates = self.template_bytes()
        classes = self.map_templates()

        def tile(x, y, level):
            return self.read(tiles + TILE_SIZE * ((level * side + y) * side + x), TILE_SIZE)

        def has_creatures(address):
            return any(t != -1 for t in struct.unpack("<7i", self.read(address, 28)))

        def hero_record(index):
            return game + HEROES + HERO_SIZE * index

        def own_name(index):
            name = self.read(hero_record(index) + 0x23, 13).split(b"\0")[0].decode("latin-1")
            return int(index >= len(hotraits) or hotraits[index] != name)

        begin, end = struct.unpack("<II", self.read(game + TOWNS, 8))
        town_records = [begin + TOWN_SIZE * i for i in range((end - begin) // TOWN_SIZE)]
        words, armed, marks, subclasses = [], [], {}, {}
        for x, y, level, template, _ in self.map_objects():
            at = entrance_cell(templates, side, x, y, template)
            if at is None:
                words.append(-1)
                armed.append(0)
                continue
            raw = tile(at[0], at[1], level)
            word, = struct.unpack_from("<I", raw, 0)
            kind, = struct.unpack_from("<i", raw, 0x1E)
            if kind == HERO_OBJECT:
                word, = struct.unpack("<I", self.read(hero_record(word) + 0x14, 4))
            elif kind == BOAT_OBJECT:
                first = self.u32(game + BOATS)
                word, = struct.unpack("<I", self.read(first + BOAT_RECORD * word + 0x14, 4))
            words.append(word if word != 0xFFFFFFFF else -1)
            cls = classes[template][1]
            marks[len(words) - 1] = (cls, word)
            subclasses[len(words) - 1] = classes[template][2]
            defended = False
            if cls == 98 and word < len(town_records):
                town = town_records[word]
                garrison, visiting = self.i32(town + 0x10), self.i32(town + 0xC)
                defended = (garrison >= 0 or (has_creatures(hero_record(visiting) + 0x91)
                                              if visiting >= 0 else has_creatures(town + 0xE0)))
            elif cls == 33:
                first = self.u32(game + 0x4E3AC)
                defended = has_creatures(first + 0x40 * word + 4)
            armed.append(int(defended))
        towns = []
        for record in town_records:
            name = self.c_string(self.u32(record + TOWN_NAME), 64)
            if name not in town_names:
                raise RuntimeError("a town's name is not in TownName.txt")
            towns.append([town_names.index(name), struct.unpack("<b", self.read(record + TOWN_FACTION, 1))[0]])
        hero_ids = [[index, own_name(index)]
                    for (_, _), (kind, index) in sorted(self.standing(z).items(),
                                                        key=lambda t: t[0][::-1])
                    if kind == "hero"]
        player = self.read(game + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        count = struct.unpack_from("<b", player, 0x1)[0]
        listed = struct.unpack_from(f"<{count}i", player, 0x8)
        hero_list = [[i, self.i32(hero_record(i) + HERO_CLASS_FIELD), own_name(i)] for i in listed]
        towns_count = struct.unpack_from("<b", player, PLAYER_TOWNS)[0]
        town_list = list(struct.unpack_from(f"<{towns_count}b", player, PLAYER_TOWN_LIST))
        teams = list(struct.unpack("<8b", self.read(game + TEAMS, 8)))
        levels = 1 + (self.read(game + TWO_LEVELS, 1)[0] != 0)
        words_all = struct.unpack(f"<{side * side * levels}H",
                                  self.read(self.u32(TILE_VISION_POINTER), 2 * side * side * levels))
        guarded = ["".join("1" if words_all[(lv * side + y) * side + x] & 0x100 else "0"
                           for x in range(side))
                   for lv in range(levels) for y in range(side)]
        selected = struct.unpack_from("<i", player, 0x4)[0]
        day_points = self.i32(hero_record(selected) + 0x49) if selected >= 0 else 0
        reach = [[x, y, cost] for (x, y), cost in sorted((reach or {}).items())]
        state = self.visit_state(colour)
        visits = [self.visit(*marks[i], state) if i in marks else [0, -1]
                  for i in range(len(words))]
        # A Keymaster's Tent's mark (E24 M3 T6a2b): the viewer's bit of its colour's.
        for i, (cls, _) in marks.items():
            if cls == 10:
                colour_bits = self.read(game + KEYMASTERS + subclasses[i] % 8, 1)[0]
                visits[i] = [0, int(0 <= colour < 8 and colour_bits >> colour & 1)]
        return {"words": words, "armed": armed, "towns": towns, "hero_ids": hero_ids,
                "hero_list": hero_list, "town_list": town_list, "teams": teams,
                "guarded": guarded, "reach": reach, "day_points": day_points, "visits": visits,
                **self.contents(marks, state, own_name)}

    def visit_state(self, colour):
        """What the visit marks read for the viewing player `colour` (E24 M3 T6a2a; design
        §4.11): {"viewer", "shown", "current", "kinds", "player", "hero"} (the shown and the
        current player as their low 16 bits read signed, the player's record and the selected
        hero's, None without one)."""
        game = self.game()
        player = self.read(game + PLAYERS + PLAYER_SIZE * colour, PLAYER_SIZE)
        selected = struct.unpack_from("<i", player, 0x4)[0]
        return {"viewer": colour,
                "shown": struct.unpack("<h", self.read(SHOWN_PLAYER, 2))[0],
                "current": struct.unpack("<h", self.read(CURRENT_PLAYER, 2))[0],
                "kinds": self.read(game + KINDS_VISITED, 28), "player": player,
                "hero": (self.read(game + HEROES + HERO_SIZE * selected, HERO_SIZE)
                         if selected >= 0 else None)}

    def visit(self, cls, word, state):
        """[known, mark] of an object of class `cls` whose entrance tile's word is `word` (E24 M3
        T6a2a; design §4.11, `VISIT_RULES`): known 0 or 1, the mark -1 none, 0 not visited, 1
        visited; [0, -1] for a class without visit marks."""
        if cls not in VISIT_RULES:
            return [0, -1]
        kind, rule = VISIT_RULES[cls]

        def bit(n):
            return 0 <= n < 8 and (word >> 5 & 0xFF) >> n & 1

        def u32(raw, offset):
            return struct.unpack_from("<I", raw, offset)[0]

        viewer, hero, player = state["viewer"], state["hero"], state["player"]
        if kind == "word":
            known = bit(state["shown"])
        else:
            known = kind is not None and 0 <= viewer < 8 and state["kinds"][kind] >> viewer & 1
        mark = None
        if rule is None:
            pass
        elif rule[0] in ("flags", "hero", "player", "owner"):
            if hero is not None:
                if rule[0] == "flags":
                    mark = u32(hero, HERO_STATUS) & rule[1] != 0
                elif rule[0] == "hero":
                    mark = u32(hero, rule[1]) >> (word & 0x1F) & 1
                elif rule[0] == "player":
                    mark = (u32(player, rule[1]) >> (word & 0x1F) & 1
                            and (rule[2] is None or not word >> rule[2] & 1))
                else:
                    mark = bit(struct.unpack_from("<b", hero, 0x22)[0])
        elif rule[0] == "garden":
            mark = u32(player, 0xB8) >> (word & 0x1F) & 1 and not word >> 10 & 1
        elif rule[0] == "obelisk":
            raw = struct.unpack("<b", self.read(self.game() + OBELISKS_VISITED + word, 1))[0]
            mark = 0 <= viewer < 8 and raw >> viewer & 1
        elif rule[0] == "current":
            mark = bit(state["current"])
        elif bit(state["current"]):
            mark = word & rule[1] == 0
        return [int(bool(known)), -1 if mark is None else int(bool(mark))]

    def contents(self, marks, state, own_name):
        """The contents' lists of `[hover]` (E24 M3 T6a2b; design §4.12, the viewer's
        `hover::contents::Written`) of the objects `marks` gives ({object: (class, word)} of
        their entrance tiles), for the players of `state` (`visit_state`): the viewing one a Seer's
        Hut's visit reads, the shown one a bank's and a Quest Guard's, the current one a
        Shrine's, a Tree's and a Witch Hut's; the selected hero's spells, skills and Trees.
        `own_name(index)`: 1 when a hero's name is the map's own."""
        game = self.game()
        hero = state["hero"]
        out = {"mines": [], "banks": [], "shrines": [], "trees": [], "witch_huts": [],
               "quests": []}

        def bit(word, player):
            return int(0 <= player < 8 and word >> 5 + player & 1)

        def army(at):
            raw = struct.unpack("<14i", self.read(at, 56))
            return [[t, c] for t, c in zip(raw[:7], raw[7:]) if t != -1]

        def known(kind):
            current = state["current"]
            return int(0 <= current < 8 and state["kinds"][kind] >> current & 1)

        current = state["current"]
        for i, (cls, word) in sorted(marks.items()):
            if cls == 53:
                record = self.u32(game + MINES) + MINE_SIZE * word
                out["mines"].append([i, self.read(record + 1, 1)[0], army(record + 4)])
            elif cls in BANK_CLASSES:
                record = self.u32(game + BANKS) + BANK_SIZE * (word >> 13 & 0xFFF)
                out["banks"].append([i, bit(word, state["shown"]),
                                     [] if word & BANK_LOOTED else army(record)])
            elif cls in (88, 89, 90):
                spell = signed(word, 13, 10) if bit(word, current) else -1
                learned = int(hero is not None and 0 <= spell < 70
                              and hero[HERO_SPELLS + spell] != 0)
                out["shrines"].append([i, known(SHRINE_KIND + cls - 88), spell, learned])
            elif cls == 102:
                price = signed(word, 13, 3) if bit(word, current) else -1
                mark = (-1 if hero is None
                        else struct.unpack_from("<I", hero, HERO_TREES)[0] >> (word & 0x1F) & 1)
                out["trees"].append([i, known(TREE_KIND), price, mark])
            elif cls == 113:
                if word & 0xFE000 == 0xFE000:
                    out["witch_huts"].append([i, 0, -1, 0])
                    continue
                skill = signed(word, 13, 7) if bit(word, current) else -1
                learned = int(hero is not None and 0 <= skill < 28
                              and hero[SECONDARY + skill] != 0)
                out["witch_huts"].append([i, known(WITCH_KIND), skill, learned])
            elif cls in (83, 215):
                vector, size = QUEST_VECTORS[0 if cls == 83 else 1]
                record = self.read(self.u32(game + vector) + size * word, size)
                quest, visitors = struct.unpack_from("<IB", record, 0)
                player = state["viewer"] if cls == 83 else state["shown"]
                row = [i, int(0 <= player < 8 and visitors >> player & 1),
                       record[0x11] if cls == 83 else 0]
                out["quests"].append(row + (self.quest(quest, own_name) if quest
                                            else [0, 0, [], []]))
        return out

    def quest(self, address, own_name):
        """[type, variant, values, pairs] of the quest at `address` (E24 M3 T6a2b; design §4.12,
        the viewer's quest row): its type by its vtable, its texts' variant, and what its hint
        names: a level, the four primary skills, a hero [index, own name], a monster's creature,
        the artifacts, a creature quest's [type, count] pairs, the seven resources, a hero to
        be, a colour."""
        raw = self.read(address, 0x60)
        vtable, = struct.unpack_from("<I", raw, 0)
        kind, rest = divmod(vtable - QUEST_VTABLE, QUEST_VTABLE_SIZE)
        if rest or not 1 <= kind <= 9:
            raise RuntimeError(f"a quest at {address:#x} of vtable {vtable:#x}")
        variant, = struct.unpack_from("<i", raw, QUEST_VARIANT)

        def field(offset):
            return struct.unpack_from("<i", raw, offset)[0]

        def vector(offset):
            begin, end = struct.unpack_from("<II", raw, offset)
            count = (end - begin) // 4 if begin and end > begin else 0
            return list(struct.unpack(f"<{count}i", self.read(begin, 4 * count))) if count else []

        values, pairs = [], []
        if kind in (1, 4, 9):
            values = [field(0x48 if kind == 4 else 0x40)]
        elif kind == 2:
            values = list(struct.unpack_from("<4b", raw, 0x40))
        elif kind in (3, 8):
            index = field(0x44 if kind == 3 else 0x40)
            values = [index, own_name(index)]
        elif kind == 5:
            values = vector(0x44)
        elif kind == 6:
            pairs = [list(p) for p in zip(vector(0x54), vector(0x44))]
        else:
            values = list(struct.unpack_from("<7i", raw, 0x40))
        return [kind, variant, values, pairs]

    def write_content(self, cls, tile, word, subclass, colour, known, visited, hero, guards):
        """Writes a situation's visit of an object of the owners' and contents' hints (E24 M3
        T6a2b; design §4.12) whose entrance tile is at `tile` with `word` and `subclass`, for the
        viewing player `colour`, by what `contents` reads inverted: its kind known (a Shrine's, a
        Tree's, a Witch Hut's), visited or not (a Tent's colour bit, a bank's or a Shrine's, Tree's
        or Witch Hut's word bit, a Seer's Hut's or Quest Guard's visitor bit), the selected
        hero's own (`hero`: a Shrine's spell known, a Tree visited, a Witch Hut's skill at basic),
        the guards of a mine or a bank (`guards`, [creature, count] stacks; a bank's not looted)."""
        game = self.game()
        state = self.visit_state(colour)
        player = game + PLAYERS + PLAYER_SIZE * colour
        selected = self.i32(player + 0x4)
        hero_at = game + HEROES + HERO_SIZE * selected

        def set_bit(address, n, on=True):
            value = self.u32(address)
            self.write_u32(address, value | 1 << n if on else value & ~(1 << n))

        def word_bit(value, p, on):
            return value | 1 << 5 + p if on else value & ~(1 << 5 + p)

        kinds = {102: TREE_KIND, 113: WITCH_KIND, 88: SHRINE_KIND, 89: SHRINE_KIND + 1,
                 90: SHRINE_KIND + 2}
        if known and cls not in kinds:
            raise RuntimeError(f"class {cls} shows no bonus")
        if hero is not None and (cls not in kinds or selected < 0):
            raise RuntimeError(f"class {cls}: no hero's own, or no hero selected")
        if guards is not None and cls != 53 and cls not in BANK_CLASSES:
            raise RuntimeError(f"class {cls} has no guards")
        if known:
            set_bit(game + KINDS_VISITED + kinds[cls], state["current"])
        if cls == 10 and visited is not None:
            set_bit(game + KEYMASTERS + subclass % 8, colour, visited)
        elif cls in BANK_CLASSES:
            if visited is not None:
                word = word_bit(word, state["shown"], visited)
            if guards is not None:
                self.write_army(self.u32(game + BANKS) + BANK_SIZE * (word >> 13 & 0xFFF), guards)
                word &= ~BANK_LOOTED
        elif cls == 53 and guards is not None:
            self.write_army(self.u32(game + MINES) + MINE_SIZE * word + 4, guards)
        elif cls in kinds:
            if visited is not None:
                word = word_bit(word, state["current"], visited)
            if hero is not None and cls == 102:
                set_bit(hero_at + HERO_TREES, word & 0x1F, hero)
            elif hero is not None:
                at = (HERO_SPELLS + signed(word, 13, 10) if cls != 113
                      else SECONDARY + signed(word, 13, 7))
                self.write_bytes(hero_at + at, bytes([int(hero)]))
        elif cls in (83, 215) and visited is not None:
            vector, size = QUEST_VECTORS[0 if cls == 83 else 1]
            at = self.u32(game + vector) + size * word + 4
            p = state["viewer"] if cls == 83 else state["shown"]
            bits = self.read(at, 1)[0]
            self.write_bytes(at, bytes([bits | 1 << p if visited else bits & ~(1 << p) & 0xFF]))
        self.write_u32(tile, word)

    def write_visit(self, colour, x, y, known, visited, hero=None, guards=None):
        """Writes into the game that the viewing player `colour` knows the bonus of the object
        anchored on (x, y) of the surface (`known`) and that it was visited or not (`visited`,
        None leaves it), by `VISIT_RULES` inverted (E24 M3 T6a2a; design §4.11): its kind's bit
        or its word's, the selected hero's or the player's field, the word's bits (on its first
        entrance tile, the one the hint reads). A visited Water Wheel or Windmill has nothing
        left, one not visited 1 of the mask's lowest bits."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        templates = self.template_bytes()
        classes = self.map_templates()
        found = [(t, level) for ox, oy, level, t, _ in self.map_objects()
                 if (ox, oy, level) == (x, y, 0)]
        if len(found) != 1:
            raise RuntimeError(f"no single object anchored on {(x, y)}")
        template, level = found[0]
        cls = classes[template][1]
        at = entrance_cell(templates, side, x, y, template)
        tile = self.u32(game + TILES) + TILE_SIZE * ((level * side + at[1]) * side + at[0])
        word = self.u32(tile)
        if cls in CONTENT_CLASSES:
            return self.write_content(cls, tile, word, classes[template][2], colour, known,
                                      visited, hero, guards)
        if cls not in VISIT_RULES:
            raise RuntimeError(f"the object on {(x, y)} (class {cls}) has no visit marks")
        if hero is not None or guards is not None:
            raise RuntimeError(f"class {cls}: no hero's own or guards")
        kind, rule = VISIT_RULES[cls]
        state = self.visit_state(colour)

        def set_bit(address, n, on=True):
            value = self.u32(address)
            value = value | 1 << n if on else value & ~(1 << n)
            self.write_u32(address, value)

        player = game + PLAYERS + PLAYER_SIZE * colour
        selected = self.i32(player + 0x4)
        hero = game + HEROES + HERO_SIZE * selected
        if known and kind == "word":
            word |= 1 << 5 + state["shown"]
        elif known and kind is not None:
            set_bit(game + KINDS_VISITED + kind, colour)
        elif known:
            raise RuntimeError(f"class {cls} shows no bonus")
        if visited is not None:
            if rule is None:
                raise RuntimeError(f"class {cls} shows no mark")
            if rule[0] in ("flags", "hero", "player", "owner") and selected < 0:
                raise RuntimeError(f"class {cls}'s mark needs a selected hero")
            if rule[0] == "flags":
                status = self.u32(hero + HERO_STATUS)
                low = rule[1] & -rule[1]
                self.write_u32(hero + HERO_STATUS,
                               status | low if visited else status & ~rule[1])
            elif rule[0] == "hero":
                set_bit(hero + rule[1], word & 0x1F, visited)
            elif rule[0] in ("player", "garden"):
                offset, clear = (rule[1], rule[2]) if rule[0] == "player" else (0xB8, 10)
                set_bit(player + offset, word & 0x1F, visited)
                if clear is not None:
                    word &= ~(1 << clear)
            elif rule[0] == "owner":
                owner = struct.unpack("<b", self.read(hero + 0x22, 1))[0]
                word = word | 1 << 5 + owner if visited else word & ~(1 << 5 + owner)
            elif rule[0] == "obelisk":
                set_bit(game + OBELISKS_VISITED + word, colour, visited)
            elif rule[0] == "current":
                n = 5 + state["current"]
                word = word | 1 << n if visited else word & ~(1 << n)
            else:
                word |= 1 << 5 + state["current"]
                low = rule[1] & -rule[1]
                word = word & ~rule[1] if visited else (word & ~rule[1]) | low
        self.write_u32(tile, word)

    def write_bytes(self, address, data):
        """Writes `data` into the game (E24 M1 T5a: a hero's or boat's facing before a still)."""
        with open(f"/proc/{self.pid}/mem", "r+b", buffering=0) as f:
            f.seek(address)
            f.write(data)

    def boats(self):
        """Every boat record's address and fields (E24 M1 T5a): [{address, tile, used, type,
        facing, colour, hero, aboard}]."""
        begin, end = struct.unpack("<II", self.read(self.game() + BOATS, 8))
        raw = self.read(begin, end - begin) if end > begin else b""
        out = []
        for i in range(0, len(raw) - BOAT_RECORD + 1, BOAT_RECORD):
            x, y, z = struct.unpack_from("<3h", raw, i)
            kind, facing, colour = struct.unpack_from("<3b", raw, i + 0x1A)
            out.append({"address": begin + i, "tile": [x, y, z], "used": raw[i + 0x18],
                        "type": kind, "facing": facing, "colour": colour,
                        "hero": struct.unpack_from("<i", raw, i + 0x20)[0],
                        "aboard": raw[i + 0x24]})
        return out

    def mines(self):
        """The mines' and Lighthouses' records in the game's order (E08 T79a, batch E9a): [{index,
        owner, resource, abandoned, guards: [[creature, count]] by slot (an empty slot [-1, 0]),
        entrance: [x, y, level]}]."""
        begin, end = struct.unpack("<II", self.read(self.game() + MINES, 8))
        raw = self.read(begin, end - begin) if end > begin else b""
        out = []
        for i in range(0, len(raw) - MINE_RECORD + 1, MINE_RECORD):
            types = struct.unpack_from("<7i", raw, i + MINE_GUARDS)
            counts = struct.unpack_from("<7i", raw, i + MINE_GUARDS + 28)
            owner, resource = struct.unpack_from("<2b", raw, i)
            out.append({"index": i // MINE_RECORD, "owner": owner, "resource": resource,
                        "abandoned": raw[i + 2], "guards": [[t, c] for t, c in zip(types, counts)],
                        "entrance": list(raw[i + MINE_ENTRANCE:i + MINE_ENTRANCE + 3])})
        return out

    def tile_word(self, x, y, z=0):
        """A tile's own word, class and subclass (E08 T79a, batch E9a's Shipyard: each of its
        three tiles' words, which `object_at` would read on the entrance) and, when a hero stands
        there (class 34), the hero's index and the class and word of the object under it (its
        record's +0xC, +0x14)."""
        found = self.tile(x, y, z)
        if found["class"] == HERO_OBJECT:
            index = found["word"]
            raw = self.read(self.game() + HEROES + HERO_SIZE * index + HERO_UNDER_CLASS, 12)
            found["hero"] = index
            found["under_class"], = struct.unpack_from("<i", raw, 0)
            found["under_word"], = struct.unpack_from("<I", raw, HERO_UNDER_WORD - HERO_UNDER_CLASS)
        return found

    def standing(self, z=0):
        """The heroes and boats the view draws on a level (E24 M1 T5a; view_tile_heroes_boats
        0x40FB30): {(x, y): ("hero", index) | ("boat", index)} for each tile of class 34 or 8
        with the entrance bit and a word other than -1."""
        game = self.game()
        side = self.i32(game + MAP_SIDE)
        raw = self.read(self.u32(game + TILES) + TILE_SIZE * z * side * side,
                        TILE_SIZE * side * side)
        out = {}
        for i in range(side * side):
            at = TILE_SIZE * i
            word, = struct.unpack_from("<I", raw, at)
            kind, = struct.unpack_from("<i", raw, at + 0x1E)
            if kind in (HERO_OBJECT, BOAT_OBJECT) and raw[at + 0xD] & ENTRANCE and word != 0xFFFFFFFF:
                out[(i % side, i // side)] = ("hero" if kind == HERO_OBJECT else "boat", word)
        return out

    def map_heroes(self, z=0):
        """The heroes and boats the view draws on a level (E24 M1 T5a), as `render --live` takes
        them: heroes [x, y, z, class, facing, owner, boat type, boat colour] (the boat's -1 -1 on
        land) and boats [x, y, z, type, facing], each by its tile, rows then columns."""
        boats = self.boats()
        heroes, empty = [], []
        for (x, y), (kind, index) in sorted(self.standing(z).items(), key=lambda t: t[0][::-1]):
            if kind == "boat":
                b = boats[index]
                empty.append([x, y, z, b["type"], b["facing"]])
                continue
            raw = self.read(self.game() + HEROES + HERO_SIZE * index, HERO_SIZE)
            hero_id, = struct.unpack_from("<i", raw, HERO_ID)
            status, = struct.unpack_from("<I", raw, HERO_STATUS)
            boat = [-1, -1]
            if status & IN_BOAT:
                sailed = [b for b in boats if b["used"] and b["hero"] == hero_id and b["aboard"] == 1]
                boat = [sailed[0]["type"], sailed[0]["colour"]] if sailed else [-1, -1]
            heroes.append([x, y, z, struct.unpack_from("<i", raw, HERO_CLASS_FIELD)[0],
                           raw[HERO_FACING], struct.unpack_from("<b", raw, 0x22)[0]] + boat)
        return heroes, empty

    def hero_on(self, x, y, z=0):
        """The index of the hero a player owns whose record stands on (x, y, z), None for none
        (E24 M1 T5a). By the records, not the tiles: at a human player's turn start the original
        lifts its selected hero off its tile (adventure manager +0x1EC set) until the view moves."""
        found = [h["index"] for h in self.heroes() if h["tile"] == [x, y, z]]
        return found[0] if found else None

    def write_facing(self, x, y, z, facing):
        """Turns the hero, or else the empty boat, on (x, y, z) to `facing` (0-7; E24 M1 T5a):
        the hero record's +0x47 or the boat record's +0x1B. The view's next draw shows it."""
        index = self.hero_on(x, y, z)
        if index is not None:
            address = self.game() + HEROES + HERO_SIZE * index + HERO_FACING
        else:
            boats = [b for b in self.boats() if b["used"] and b["tile"] == [x, y, z]]
            if not boats:
                raise KeyError(f"no hero or boat on {(x, y, z)}")
            address = boats[0]["address"] + BOAT_FACING
        self.write_bytes(address, bytes([facing]))

    def write_boat_colour(self, x, y, z, colour):
        """Gives the boat of the hero on (x, y, z) the flag colour `colour` (0-7; E24 M1 T5b2):
        the boat record's +0x1C, which boarding sets to the hero's owner (design §2.8). A still's
        staging, as the facings: the view's next draw shows the boat's flag of that colour."""
        index = self.hero_on(x, y, z)
        hero_id = None if index is None else struct.unpack(
            "<i", self.read(self.game() + HEROES + HERO_SIZE * index + HERO_ID, 4))[0]
        boats = [b for b in self.boats() if b["used"] and b["aboard"] == 1 and b["hero"] == hero_id]
        if not boats:
            raise KeyError(f"no hero in a boat on {(x, y, z)}")
        self.write_bytes(boats[0]["address"] + BOAT_COLOUR, struct.pack("<b", colour))

    def monster(self, x, y, z=0):
        """The wandering monster on a tile from its word (E17 H1; E3-3), None for none: its
        creature, count, aggression (-4 to 10, signed 5 bits), never flees and does not grow;
        since E08 T79b the sixteenths of its count (bits 27-30, batch E9b's growth) and the word."""
        t = self.tile(x, y, z)
        if t["class"] != MONSTER_CLASS:
            return None
        word = t["word"]
        aggression = (word >> 12) & 0x1F
        return {"creature": t["subclass"], "count": word & 0xFFF,
                "aggression": aggression - 32 * (aggression >= 16),
                "never_flees": bool(word >> 17 & 1), "fixed_count": bool(word >> 18 & 1),
                "sixteenths": word >> SIXTEENTHS_SHIFT & 0xF, "word": word}

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


def signed(word, low, bits):
    """The signed field of `bits` bits from bit `low` of `word` (the hint's `shl`, `sar` pairs)."""
    value = word >> low & (1 << bits) - 1
    return value - (1 << bits) if value >> bits - 1 & 1 else value


def entrance_cell(templates, side, x, y, template):
    """An object's entrance tile (x, y) from its anchor and template (object_entrance_tile
    0x4FEC30): the template's first visitable cell, by row from the anchor's upwards, each row
    from the anchor's column leftwards; None without one (E24 M1 T5b1)."""
    t = TEMPLATE_RECORD * template
    width, height = templates[t + 0x10], templates[t + 0x11]
    mask = int.from_bytes(templates[t + 0x2C:t + 0x32], "little")
    for dy in range(height):
        for dx in range(width):
            tx, ty = x - dx, y - dy
            if 0 <= tx < side and 0 <= ty < side and mask >> (47 - 8 * dy - dx) & 1:
                return tx, ty
    return None


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
                          "log": mem.battle_log(), "hovered": mem.hovered_item()}, indent=1))
