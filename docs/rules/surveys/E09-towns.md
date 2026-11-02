# E09 towns survey (E09 T1): the census, the start, the town on the map

Date: 2026-11-02. Scope: E09 increment 1 (towns on the map; the epic's decision 5), input for
T2's ADR drafts, the design note and the cut. In three parts, T1a cut in two at its start (the
user, 2026-11-02): **T1a1** the census of the shipped maps and the players' faction choice with
the random towns (§0 to §3); **T1a2** the main town's generated hero and the pool it draws from,
the random hero, a hero placed on a town's blocked tile, the town's buildings at the start (§4 on,
appended); **T1b** the town on the map (the visit, capture and the siege's facts, the hall's
income, the views, batch TWN-1's rows, the checkpoint's `TWN-Q2`).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-11-02):
- `original-data`: the 160 shipped maps (`maps.tar`) read with `h3_map::read` and imported with
  `h3_map::import` (a throwaway example, not committed); a code reading of the user's
  `Heroes3.exe` (SoD 4.0 Complete, SHA-256 `aac39e8c…`); every address cited has an entry in
  `docs/exe/map.toml` or `structs.toml`. **Read whole:** `random_allowed_faction` 0x5132D0,
  `count_allowed_factions` 0x5132B0, `faction_choice_offered` 0x58CE70. **Read in part:**
  `load_town_body` 0x5019F0 (the record's identifier, owner and type, the SoD alignment byte, the
  random town's faction; the name, garrison, buildings, spells and events not followed),
  `load_object_body` 0x502E00 at its switch (classes 77 and 98 reach `load_town_body`),
  `scenario_dialog_begin` 0x588330 at its loop over the colours (0x5885CD to 0x588729),
  `setup_towns` 0x4CAA70 at the random town's conversion and the names (0x4CAC33 to 0x4CAD40).
  **Not followed:** the scenario dialog's drawing and its arrows (it shows a colour's choice; what
  the server must check is in §2.2), network play (the host's copy of the choices), the campaign's
  and the random map generator's paths, the computer player.
- VCMI 1.7.5 (`lib/mapping/MapFormatH3M.cpp`, `lib/mapping/CMapHeader.cpp`,
  `lib/mapObjects/CGTownInstance.cpp`, `lib/gameState/CGameState.cpp`).
- The wiki: [Map Editor - Objects](https://heroes.thelazy.net/index.php/Map_Editor_-_Objects)
  ("Random Towns"), [Map Editor](https://heroes.thelazy.net/index.php/Map_Editor) ("Player
  Specs"), [Horn of the Abyss (Changelog)](https://heroes.thelazy.net/index.php/Horn_of_the_Abyss_(Changelog))
  (1.6.0, 1.3.0), through the search's summaries.
- **T1a2** (retrieved 2026-11-02): a census of the same maps (a throwaway crate over
  `h3_map::read`, not committed); the code reading, every address in `docs/exe/map.toml`.
  **Read whole:** `hero_pool_init` 0x4C4E30, `hero_draw_of_faction` 0x4BB400, `main_town_heroes`
  0x4CA040, `town_hero_place` 0x5C13B0, `hero_place` 0x4D7900, `hero_show` 0x4D75D0,
  `placed_heroes_setup` 0x4CAE10, `scenario_hero_lists` 0x583890, `game_reset` 0x4BF1A0,
  `town_setup` 0x5C0670, `town_buildings_setup` 0x5C08C0, `tavern_hero_draw` 0x4BB5E0 (for the
  weights' role only; increment 3's). **Read in part:** `new_game_heroes` 0x4BFE70 (its steps'
  order and the chosen heroes, 0x4BFF4A to 0x4C012F), `new_game_bans` 0x4C2450 (its start),
  `map_header_read` 0x4C4390 (the allowed heroes, 0x4C47E4 to 0x4C498C), `load_hero_body`
  0x5021C0 (the type, the random hero's draw and the owner byte, 0x5021FA to 0x502374 and
  0x50294D to 0x50298F), `new_game_start` 0x58C570 (the choices, 0x58C680 to 0x58C6CE),
  `town_build` 0x5BE930 (its first lines). **Not followed:** the scenario dialog's drawing (what
  it shows in the hero column), network play's copy of the choices (0x4EF611), the campaign's
  paths (carried heroes, 0x4CA1A7's experience), the tavern's weekly heroes
  (`tavern_new_heroes` 0x4C8450), the horde part of `town_build`, the building tables' start-up
  (0x4EB730), a town's change of owner (0x4C61E0, T1b's), the guild's spells (0x5BE030, 0x5BE600)
  and the Shipyard's spot (0x5C0C90), increment 4's.
- VCMI 1.7.5 for T1a2: `lib/gameState/CGameState.cpp` (`pickNextHeroType`,
  `pickUnusedHeroTypeRandomly`, `getUnusedAllowedHeroes`, `placeStartingHeroes`,
  `placeHeroesInTowns`, `initVisitingAndGarrisonedHeroes`, `initTowns`),
  `lib/mapObjects/CGHeroInstance.cpp` (`pickRandomObject`), `lib/mapObjects/CGTownInstance.cpp`
  (`initializeNeutralTownGarrison`), `lib/mapping/MapFormatH3M.cpp` (the town's buildings),
  `config/gameConfig.json` (`startingDwellingChances`).
- The wiki and others for T1a2: [Town](https://heroes.thelazy.net/index.php/Town) ("Neutral Town
  Garrison"); the HotA team's [documentation](https://h3hota.com/en/documentation) (its own towns'
  default buildings); a GOG forum thread on the starting hero
  ([link](https://www.gog.com/forum/heroes_of_might_and_magic_series/homm3_is_there_a_way_to_pick_your_initial_hero_in_the_very_beginning_of_sp_play))
  and a Celestial Heavens thread on "Generate hero at main town"
  ([link](https://www.celestialheavens.com/forum/7/14260)), through the search's summaries.

## 0. Findings in short (T1a1, T1a2)

1. **Every shipped map has towns:** 1 468 on the 160 maps, 876 random towns (class 77, on 127
   maps) and 592 of a fixed faction (98); **no Conflux town is placed on any map** (98's subclass 8
   never occurs). 889 are owned at load, every one by a colour that plays; 579 are neutral. All
   twenty town templates share one footprint (§1.1).
2. **All 160 maps import once three things exist:** the hero generated at a main town (128 maps
   refused for it today), a hero standing in a town (3 maps) and **the random hero** (class 70: 21
   maps refused for it alone, 15 more behind a town refusal; 176 random heroes on 36 maps). The
   random hero is drawn from the same pool as the main town's hero, so it is a scope candidate for
   increment 1 (the checkpoint's `TWN-Q2`; T1a2 reads its draw beside the main town's).
3. **The header's main town is its entrance tile** (the town's anchor less two columns), as VCMI's
   loader converts it; all 553 main towns match a town of their own player; 538 generate a hero.
4. **The player's faction** (§2.2): the scenario dialog keeps one choice per colour, a faction or
   Random; when the game begins, Random becomes a faction drawn uniformly among the colour's
   allowed ones (the map's mask: eight bits in RoE, nine from AB on). The map's "random town" flag
   is read only by the dialog (whether it offers the choice) and changes no rule.
5. **A random town's faction** (§2.3; `load_town_body`, as the map's objects are read at the new
   game, in object order): a SoD alignment to a colour that plays gives that colour's faction;
   else an owned town takes its owner's; else (neutral, or aligned to a colour nobody plays) a
   uniform draw over **eight factions on an RoE map, nine (Conflux included) on an AB or SoD map**.
   The wiki agrees on each point; VCMI differs on two (TD-1, TD-2).
6. **Random stacks in town garrisons:** 12 stacks in 7 maps, all in random towns; the import
   keeps no garrison yet, so increment 1 resolves them by the town's faction or refuses them
   (T1a2 reads the draw; a random hero's army has the same kind, and HotA 1.3.0 fixed a SoD bug
   in it).
7. **Random dwellings tied to a town:** 1 397 on 87 maps (classes 216 and 217 by the town's
   identifier), **every one tied to a random town**: the random dwellings of E08's increment 10
   resolve after this lane's random towns (a note in the adventure lane's inbox).
8. **Disputes:** TD-1 (the alignment before the owner; no shipped map affected), TD-2 (Conflux on
   RoE maps). **Quirk candidate:** T-1 (an alignment byte from 8 to 254 reads past the eight
   colours' records; none on the shipped maps). Content for the harvest: none from T1a1 (factions
   are core content; the towns' name lists are text).

**T1a2 (§4 to §6):**

9. **The pool** (`hero_pool_init` 0x4C4E30, right after the map's header): a hero type is free
   when the map allows it, else out of the game; SoD's disposed heroes narrow who may take one (a
   colour mask per hero). An RoE map allows its 128 bits and **the 16 Conflux heroes besides**,
   never the 12 campaign heroes. On the shipped maps **no placed hero or prisoner is in its map's
   allowed set** (108 heroes, 89 prisoners).
10. **One draw for both starting heroes** (`hero_draw_of_faction` 0x4BB400): uniform over the free
    heroes of the colour's faction's two classes that the colour may take; with none, uniform over
    every free hero it may take. **No tavern weights:** they are the tavern's draw's (0x4BB5E0,
    increment 3), so increment 1 harvests none (the epic named them).
11. **The random hero** (class 70: 176 on 36 maps) is drawn while the map's objects are read, in
    the file's order, from its owner's faction; **a colour's starting-hero choice goes to its first
    random hero**. Every owner of one has the map's random-hero flag (140 players) and no other
    player has it.
12. **The main town's hero** (`main_town_heroes` 0x4CA040, after the placed heroes): per colour in
    colour order, with the generate flag: a human's chosen hero, else the same draw; it stands on
    the town's entrance as the **visiting hero** (`hero_show` 0x4D75D0 makes every hero on a
    town's entrance its visitor) and learns the guild's spells at once (increment 4).
13. **A hero in a town** (`placed_heroes_setup` 0x4CAE10): a hero (not a prisoner) whose tile is
    right of a town's entrance moves onto the entrance. All 23 on the shipped maps are such, in
    towns of their own owners, none in a main town that generates a hero.
14. **The buildings at the start** (§5.3): every town a Village Hall. The standard set: a Fort
    with the map's flag; **an owned town a Tavern, and only with a Fort a level-1 dwelling and 30
    times in 100 a level-2 one; a neutral town both dwellings**, fort or not. A custom set: the
    map's built buildings with the ones they supersede, less the forbidden ones.
15. **The garrison** (§5.2): a custom one as the map gives it, its random stacks by the town's
    faction (level and upgrade); without one, an owned town's is empty and **a neutral town's is
    drawn**: four rolls for levels 1 to 4 (33, 33, 20, 13 in 100); the wiki agrees.
16. **Disputes** TD-3 (the standard buildings: VCMI gives every one a level-1 dwelling and half
    the time a level-2), TD-4 (level 4's chance, VCMI 14), TD-5 (a neutral town's explicitly empty
    garrison: VCMI draws one), TD-6 (a hero on a town's other blocked tiles). **Quirk candidates**
    T-2 (the starting-hero choice is not reserved: one hero can stand twice on the map), T-3 (a
    random hero can take the type of a hero placed later in the file), T-4 (the draws' empty
    cases). **Content:** none new (the creatures by faction and level are in the packs; the
    neutral garrison's table and the 30 are constants of the executable).

## 1. The census

The 160 shipped maps (49 RoE, 55 AB, 56 SoD), read with `h3_map::read`; counts are towns or
players / maps. 744 colours play (human or computer).

### 1.1 Towns

| Class | Towns / maps | Owned (a colour that plays) | Neutral | Underground | RoE / AB / SoD |
|---|---|---|---|---|---|
| 77 random town | 876 / 127 | 562 / 118 | 314 / 78 | 114 / 33 | 139 / 378 / 359 |
| 98 Castle | 119 / 48 | | | | |
| 98 Rampart | 83 / 35 | | | | |
| 98 Tower | 69 / 37 | | | | |
| 98 Inferno | 80 / 42 | | | | |
| 98 Necropolis | 32 / 23 | | | | |
| 98 Dungeon | 89 / 39 | | | | |
| 98 Stronghold | 62 / 33 | | | | |
| 98 Fortress | 58 / 32 | | | | |
| 98 Conflux | 0 | | | | |
| 98 (all) | 592 / 93 | 327 / 78 | 265 / 59 | 94 / 43 | 210 / 177 / 205 |

- **No owner byte names a colour nobody plays.** No town of class 77 has a subclass other than 0.
  Neutral random towns by the map's format: RoE 28 (12 maps), AB 157 (34), SoD 129 (32).
- **Templates:** 20 DEFs (`AVCcast0`/`AVCcasx0` … `AVCrand0`/`AVCranx0`, the second of each pair
  drawn with a fort), all with one footprint: passable bytes `ff ff ff 8f 07 07`, visitable
  `00 00 00 00 00 20`: the bottom two rows blocked over five columns (the anchor's and the four to
  its left), the row above over three (columns x−3 to x−1), the entrance in the bottom row two
  columns left of the anchor, blocked and visitable (entered from below or the sides).
- **The SoD alignment byte** (random towns only use it): 255 ("the owner's or random") on 229 of
  230 owned random towns and on 83 neutral ones; 0 to 6 on 46 neutral random towns (9 maps) and
  on 1 owned one (`Battle of the Sexes`, aligned to its owner, colour 3); every aligned colour
  plays; no value from 7 to 254. Every fixed SoD town has 255.

### 1.2 Players

| A playing colour | Players / maps |
|---|---|
| with a main town | 553 (77: 369, 98: 184); 538 generate a hero (472 human-playable, 66 computer-only) |
| … not generating its hero | 15 (`Caught in the Middle`, `Faeries`, `For Sale`, `Just A Visit`, `Meeting in Muzgob` and its allied copy, `The Gauntlet`, `The Newcomers`) |
| without a main town | 191; 3 own no town (`Adventures of Jared Haret`, `Knee Deep in the Dead`, `Pirates`) |
| the "random town" flag set | 447 |

- **The main town** (`MainTown.pos`) is the entrance tile of a town owned by its player in all 553
  (the anchor less two columns). Its AB/SoD type byte equals the faction of a fixed main town (96
  players) and is 255 for a random one (276).
- **Allowed factions against the main town:** a fixed main town allows only its faction on 179 of
  184 players (more on `Unexpected Inheritance`, 2, and `Warmongers`, 3); a random main town
  allows all on 347 of 369 (all with the flag), 2 to 8 factions without the flag on 22 (`Chasing
  a Dream`, `Last Chance` and its allied copy, `Shadow Valleys`, `Caught in the Middle`,
  `Marshland Menace`). Without a main town: one faction on 53 players, more on 138.
- **Owned towns per player:** one random town 449 players, two to four 47; one fixed town 181,
  two to five 60; both kinds 4 (`Warmongers`, `The Gauntlet`); none 3.
- **SoD's byte before the allowed set** (VCMI: "faction is selectable", skipped): 1 on 56 players
  of 12 maps, 0 elsewhere; not read here.
- **The starting hero's flags** with a generating main town: the random-hero flag on 16 players
  (8 maps), a customised main hero on 5 (`Dragon Orb`, `Good Witch, Bad Witch`, `South of Hell`
  and its allied copy); T1a2's.

### 1.3 The towns' records

- **Buildings:** the standard set on 894 towns (700 with the has-fort flag, 194 without), custom
  sets on 574 (308 random, 266 fixed; 25 build nothing, 549 build something, 173 forbid
  something). Built at the start in the custom sets: a Town Hall on 370, a City Hall on 112, a
  **Capitol on 23**, a Fort on 460, a Citadel on 71, a Castle on 45; the most forbidden are the
  level-7 dwelling and its upgrade (bits 39, 40). Bits per `docs/formats/h3m.md`, "Town
  buildings".
- **Garrisons:** customised on 113 towns (53 of them empty), 12 random stacks in 7 maps (all in
  random towns: `One Bad Day - Allied`, `Peaceful Ending` and its allied copy, `The Great Race`,
  `Freedom`, `Pestilence Lake` and its allied copy). Formation tight on 3.
- **Names** customised on 252 towns; **required spells** on 32, **banned spells** on 396.
- **Town events:** 369 on 110 towns: 70 build, 56 add creatures, 105 give resources,
  189 carry only a message (3 maps); the increments that apply them read them (decision 5).

### 1.4 Heroes in and at towns

- **On a town's blocked tile:** 21 placed heroes (8 maps) and 2 random heroes (`Good to Go`),
  each in a town of its own owner and **each on the tile right of the entrance** (the entrance's
  column plus one, the same row). The import refuses 3 maps for it today (the 5 others stop
  earlier at a main town's hero). No hero stands on a town's entrance.
- **Random heroes (70):** 176 on 36 maps (1 to 8 a map, 14 on two; T1a1 wrote 166 and 1 to 9,
  recounted at T1a2).

### 1.5 The import (`MAP_IMPORT_VERSION` 15)

| Outcome today | Maps | With the main town's hero and a hero in a town allowed | With random heroes too |
|---|---|---|---|
| imports whole | 8 | 8 | 8 |
| a hero generated at a main town | 128 | 114 import, 14 stop at a random hero | all import |
| a hero standing in a town | 3 | 2 import, 1 stops at a random hero | all import |
| a random hero | 21 | 21 stop at it | all import |

Measured by clearing each map's generate-hero flags, removing its heroes on a town's blocked tile,
then its random heroes, and importing again; towns themselves stay blockers in all three runs.

### 1.6 Towns in the scenario

- **Victory:** capture a town on 14 maps, upgrade a town on 1 (`Hold the middle`, hall 2 and
  castle 2), build the Grail on 7; **loss:** lose a town on 2 (`For Sale`, `Fort Noxis`): E10's.
- **Random dwellings** (216, 217): 1 397 tied to a town by its identifier (216: 197 on 19 maps,
  217: 1 200 on 68), every one to a random town; 205 take a faction set instead.

## 2. The players' factions and the random towns (T1a1)

### 2.1 The order at the start

1. The scenario dialog holds a record per colour (0x7C bytes; H3API's `H3ScenarioPlayer`, its
   town at +0x24, −1 for Random). `scenario_dialog_begin` 0x588330 copies each colour's into the
   game's chosen factions (`game.chosen_factions`, +0x1F6B0, `structs.toml`), drawing a Random one
   (§2.2).
2. The new game (`new_game_heroes` 0x4BFE70) reads the map; `load_object_body` 0x502E00 sends
   classes 77 and 98 to `load_town_body` 0x5019F0, which appends a town setup record
   (`town_setup`, 0x88 bytes, `game.town_setups` +0x94) and resolves a random town's faction
   there (§2.3), in the objects' file order. The random dwellings resolve after all bodies are
   read (`random_dwellings_resolve` 0x502B60), a linked one from its town's record.
3. `setup_towns` 0x4CAA70 turns each random town's entrance tiles into class 98 of the record's
   faction (its picture redrawn) and sets every town up (§2.4).

### 2.2 The player's faction

- **The draw** (0x5885CD to 0x5886DB, per colour in colour order): the colour's dialog record's
  town when it is not −1; for −1, `random_allowed_faction` 0x5132D0 over the colour's allowed
  factions (`player_attributes` +0x8, game +0x1F914 + 0x44 × colour) without its "first" flag: the
  k-th set bit of the nine for k = `random_between(1, count)`, so **uniform over the allowed set**
  (a set without a bit gives faction 0). Not followed: a record that names another source of
  the choice (its +0x0 set: another copy of the choices, presumably network play's, 0x58869D)
  and a colour marked as not playing
  (+0x1F6D0 negative), which keeps its entry.
- **What the dialog offers** (not followed in full): the town shown is the record's when the
  map's "random town" flag (+0xA) is set or `faction_choice_offered` 0x58CE70 holds (the flag
  clear and more than one allowed faction, `count_allowed_factions` 0x5132B0); else the first
  allowed faction. So a colour with one allowed faction gets it; with more, the player picks one of
  them or Random. The flag's other readers are all in the dialog's functions; no rule reads it.
- **For the engine** (the setup's faction per player is the seam with E10's scenario setup,
  decision 6): a choice per playing colour, a faction of its allowed set or Random, validated
  against the map; Random drawn uniformly over the allowed set from the game's seed. The wiki
  agrees: the allowed alignments restrict the town available, and the random pool rolls only from
  them ("Player Specs").

### 2.3 A random town's faction (`load_town_body` 0x5019F0)

- The record's type (+0x68) starts as the template's subclass (0x501A93): a fixed town (98) keeps
  its faction. The identifier is 0 on an RoE map (0x501AA0); the owner is the file's byte.
- The SoD alignment byte is read after the events (0x502049: 0xFF before SoD). For class 77
  (0x502077):
  1. **an alignment other than 0xFF to a colour that plays** (its human or computer flag,
     `player_attributes` +0x0 or +0x1): that colour's chosen faction (0x5020AB);
  2. else **an owner other than 0xFF**: the owner's chosen faction (0x50215B);
  3. else `random_allowed_faction` of **0xFF** (the eight factions), or **0x1FF** (nine) when the
     edition number is 1 or 3 and `map_kind` (game +0x1F698) is at least 1 (0x5020C5); this build's
     edition is always 3 (`edition_pointer` 0x69923C), so **8 on an RoE map, 9 on an AB or SoD
     map**, Conflux included, whatever the players' allowed sets.
- The alignment is a signed index: a byte from 8 to 254 reads another record than a colour's
  (T-1). A colour that does not play falls through to the owner, then to the draw.

### 2.4 The random town becomes a town (`setup_towns` 0x4CAA70)

Per entrance tile of a class 77 or 98 object (tile +0xD bit 0x10), the town's position; a class 77
tile becomes class 98 with the record's type as its subclass and is redrawn (0x4CAC86 to
0x4CAC9B). A town without a custom name draws one from its faction's name list without repeats,
from a faction drawn by `random_between(0, 8)` once its own is used up (0x4CACC7 to 0x4CACFE);
presentation only (the names are text, not harvested). Then the town's setup and its buildings
(T1a2).

### 2.5 VCMI 1.7.5 and the wiki

- **VCMI's reader** (`MapFormatH3M.cpp`): the allowed set kept only when it is not "random and
  every faction allowed" (then the game's default, all nine); the main town's position converted
  to the object's (the H3M's is its active tile, as §1.2); the alignment: 255 neutral, below 8 a
  colour that can play, 8 to 15 "not as player" (HotA, not implemented, warned), the rest
  "corrupted" (warned), each of these kept as "the owner's or random".
- **VCMI's draws:** a player's Random uniform over its allowed set
  (`CGameState::initRandomFactionsForPlayers`), as the reading; a random town
  (`CGTownInstance::randomizeFaction`) takes **the owner's faction first, then the alignment's**,
  else a uniform draw over every faction with a town (nine, on every map's format) (TD-1, TD-2).
- **The wiki** ("Random Towns"): "Same as Owner or Random" gives the owner's faction, a random
  one when unowned; "Same as player" applies even when that player is not the owner, so an owned
  town can be "a differently aligned town" (the reading's order, against VCMI's); it falls back
  to random when the player has no faction. HotA's changelog 1.6.0 adds Conflux to RoE maps'
  random towns, which implies SoD had none there (the reading).

## 3. Disputes, quirk candidates and open points (T1a1)

| Id | What | Reading | VCMI | Wiki | Shipped maps |
|---|---|---|---|---|---|
| TD-1 | An owned random town aligned to another colour | the alignment's colour | the owner's | the alignment's | none (the one owned aligned town is aligned to its owner) |
| TD-2 | Conflux for a random town on an RoE map | never: a neutral one draws from eight, a player's set has eight bits | a neutral one draws from nine; a player with the flag and all eight allowed draws from nine | none on RoE maps (HotA 1.6.0 added it) | 28 neutral random towns (12 maps); 105 players with the flag and all eight allowed |

- **T-1 (quirk candidate):** an alignment byte from 8 to 254 indexes the playable flags past the
  eight colours (a signed byte: up to 127 forward, 128 to 254 backward into other fields) and may
  take a faction from another table entry. No shipped map has one; the editor writes 0 to 7 or
  255. Proposal: the import refuses such a byte (a typed error), as VCMI warns.
- **Open for T1a2:** the hero at the main town and its pool (the hero classes' tavern weights, the
  map's allowed heroes, the faction, the player's starting-hero choice and the random-hero flag),
  the random hero's draw (class 70; finding 2), a hero on a town's blocked tile (§1.4: always the
  tile right of the entrance), the standard buildings and the has-fort flag at the start
  (`setup_towns`' setup 0x5C0670, its buildings 0x5C08C0), the garrison's random stacks
  (finding 6).

## 4. The heroes at the start (T1a2)

### 4.1 The order at the start

1. `game_reset` 0x4BF1A0: every hero type free (`game.hero_owner` +0x4DF18, −1) and takeable by
   every colour (`game.hero_masks` +0x4DFB4, 0xFF); the choices (`chosen_heroes` 0x69FB24) −1.
2. The scenario dialog's start (`new_game_start` 0x58C570, 0x58C680 to 0x58C6CE): each colour's
   chosen hero (its record's list at +0x2C, the index at +0x20; −1 Random) into
   `chosen_heroes` by colour; network play fills it at 0x4EF611 (not followed).
3. `new_game_heroes` 0x4BFE70 (with the choices, 0x4C05E5): every hero type's starting army,
   experience and movement first (E08's readings); then for each human colour whose main town
   generates a hero, its chosen hero marked as the colour's (0x4BFF83) and stored as the colour's
   first hero (`game.first_heroes` +0x1F844, when still −1). **This mark does not last:** the map
   read that follows starts the pool again (step 4; T-2).
4. The map read (`new_game_bans` 0x4C2450): the header (`map_header_read` 0x4C4390, the allowed
   heroes), **the pool** (`hero_pool_init` 0x4C4E30, §4.2), the bans, then the objects
   (0x4FD690) in the file's order: a placed hero's or prisoner's body marks its type
   (`load_hero_body` 0x5021C0, 0x502973: a prisoner 0x41, a hero its owner), **a random hero is
   drawn there** (§4.4).
5. Then: the random objects (0x4C9DD0), `setup_towns` 0x4CAA70 (§5), the placed heroes
   (`placed_heroes_setup` 0x4CAE10: a hero in a town moved, §4.6; each hero put on its tile),
   a campaign's carried heroes, **the main towns' heroes** (`main_town_heroes` 0x4CA040, §4.5),
   the players with neither a hero nor a town out of the game (0x4C007A), the taverns' heroes
   (`tavern_new_heroes` 0x4C8450, increment 3).

So the wiki's order holds: the random heroes are drawn before the main towns' heroes, and both
after every hero placed earlier in the file.

### 4.2 The pool (`hero_pool_init` 0x4C4E30)

- Per hero type 0 to 155: free (−1) when the map's allowed set (`game.allowed_heroes` +0x1FB5C)
  has its bit, else 0x40, out of the game for every draw. Then each SoD disposed hero whose mask is
  not 0xFF sets the type's colour mask to it (0x4C4EC4): only those colours may take it. Last, a
  victory "acquire an artifact" (0, or 10) marks its artifact (artifacts, not heroes).
- **The allowed set by version** (`map_header_read`, 0x4C47E4 to 0x4C498C): an RoE map's 16
  bytes give types 0 to 127 and, **outside a campaign, types 128 to 143 (the Conflux heroes) are
  set besides** (0x4C48EF), 144 to 155 never; AB and SoD read 20 bytes, types 0 to 155.
- **The census:** no placed hero (108) or prisoner (89) has its type allowed on its map; RoE
  maps 49 (their Conflux heroes allowed so); AB/SoD maps: all 16
  Conflux heroes allowed on 94, none on 13, 8 on 3, 15 on 1; some of the 12 campaign heroes
  allowed on 99 (151 in all); SoD's disposed heroes: 832 on 9 maps, each with a mask other than
  0xFF.
- **What takes a hero out of the pool:** its owner's colour when it is placed, drawn or
  generated (`hero_place` 0x4D7900, `load_hero_body`), 0x41 for a prisoner; −1 again when a
  tavern slot's hero is released at the start (0x4C00F7) or a player is defeated (0x4F13F3).

### 4.3 The player's starting-hero choice

- **The offer** (`scenario_hero_lists` 0x583890, read whole; the dialog's other tests of the
  flags not followed in full): for a colour that plays with a faction (not Random), up to
  **16 heroes in type order**: free in the dialog's pool (the header and `hero_pool_init`
  only, so placed heroes are not marked, but none is allowed on the shipped maps), of the
  faction's two classes (the hero table's class, `hero_table` +0x8), takeable by the colour (a
  disposed hero's mask). The column offers the choice when the colour's main town generates a
  hero or the colour has the random-hero flag; with neither it shows the map's customised main
  hero (`player_attributes` +0x1C), which nothing outside the dialog reads.
- **Where it goes:** the colour's **first random hero in the file's order** takes it
  (`load_hero_body` 0x5022ED: then the choice is cleared, so a second random hero draws); else
  **a human colour's** main-town hero (`main_town_heroes` 0x4CA146 tests the human flag +0xE2; a
  computer colour's choice is honoured only by a random hero). Neither checks that the hero is
  still free (T-2).
- **The census:** the generate flag with no customised main hero on 533 players (16 with the
  random-hero flag too), the random-hero flag without the generate flag on 124 (3 with a
  customised main hero), a customised main hero on 62 (5 with a generating main town).

### 4.4 The draw and the random hero

- **`hero_draw_of_faction` 0x4BB400** (faction, colour; read whole): the faction's two classes
  (0 and 1 for Castle, then 3 and 2, 5 and 4, 6 and 7, …, 16 and 17 for Conflux; a faction
  outside 0 to 8 takes Castle's, T-4); the list of every type that is free, takeable by the colour
  and of one of the two classes (the game's hero record +0x30), in type order; when empty, every
  free type takeable by the colour; one of them by `random_between(1, n)`. **No weights:** the
  hero classes' tavern weights (HCTRAITS, `hero_class` +0x34, one byte per faction) are read only
  by the tavern's draw (`tavern_hero_draw` 0x4BB5E0, read for this: a class drawn by those
  weights, then a hero of it uniformly; on an SoD map a non-Conflux colour's tavern skips the
  Conflux classes while other heroes are free). That draw is increment 3's.
- **The random hero** (class 70, `load_hero_body`): with the owner's choice (§4.3), else
  `hero_draw_of_faction(chosen_factions[owner], owner)` at once (0x502352); the type is the
  owner's from then (0x502988), so the next draw excludes it. The rest of its body applies as a
  placed hero's (`placed_hero_setup`, E08's). A type placed later
  in the file is still free at this draw (T-3).
- **The census:** 176 random heroes on 36 maps (T1a1's 166 was wrong); their owners allow one
  faction on 10, three to six on 8, eight on 28, nine on 130; 2 stand in a town (§4.6); no hero's
  army on the shipped maps holds a random stack (the import refuses one; not read).

### 4.5 The main town's hero (`main_town_heroes` 0x4CA040, read whole)

- Per colour 0 to 7, when `player_attributes` +0xB (generate a hero at the main town; RoE: always
  with a main town) is set: the town whose record's position (its entrance, §2.4) equals the
  main town's (+0x14); the hero: a human colour's choice, else
  `hero_draw_of_faction(chosen_factions[colour], colour)`; the colour's first hero if none yet;
  the type the colour's; `town_hero_place` 0x5C13B0 puts it on the town's entrance
  (`hero_place` 0x4D7900: added to the player's heroes, the type's mask gains the colour, shown
  on its tile); then the town's guild spells are learned (0x5BE030, increment 4); one campaign
  scenario adds experience (not followed).
- **The faction is the colour's, not the town's:** on the 5 players whose fixed main town allows
  other factions too (`Unexpected Inheritance`, `Warmongers`, §1.2), a hero of another faction can
  stand in it; VCMI does the same.
- **The visiting hero:** `hero_show` 0x4D75D0 (read whole) remembers the tile's object and makes
  the hero the tile's; on a town's entrance it sets the town's visiting hero (+0x10, 0x4D76A9).
  So the generated hero, and every hero shown on an entrance, is its town's visitor.

### 4.6 A hero on a town's blocked tile (`placed_heroes_setup` 0x4CAE10)

- For each placed record with a position: when the tile one column left of the hero's is an
  entrance (+0xD bit 0x10) of a town (class 98; random towns are 98 by then) and the hero is not a
  prisoner, the hero's column is decreased by one (0x4CAE95 to 0x4CAEC2); the hero is then shown
  on its tile (the visitor, §4.5). Any owner's town; no other blocked tile.
- **The census:** 23 such heroes (21 placed, 2 random), all on the tile right of the entrance, in
  towns of their own owners, none in a main town that generates a hero (so no town starts with
  two visitors).

### 4.7 VCMI 1.7.5 and the wiki

- **VCMI:** `pickNextHeroType` takes the player's chosen hero while no hero of that type exists
  on the map, else `pickUnusedHeroTypeRandomly`: uniform over the allowed heroes of the player's
  faction that the player may take, **without the other players' choices and every hero on the
  map**, else over the other factions' (then any not allowed, then an error). A random hero
  draws so at the objects' randomisation (before the starting heroes), every object first read;
  `placeStartingHeroes` for each player with the generate flag and a main town, at the town's
  entrance, for computer players' choices too. `placeHeroesInTowns`: a hero on **any blocked tile
  of a town of its own owner** moves to its entrance (TD-6); `initVisitingAndGarrisonedHeroes`
  makes a hero on its own town's entrance the visitor. So VCMI reserves the choices (T-2) and the
  map's heroes (T-3).
- **The wiki and others:** "generate hero at main town" makes no difference against a random
  hero placed in the town, the player choosing among the town's heroes (Celestial Heavens); the
  starting hero is one of the chosen town's, unless the map sets the hero (GOG forum). Both
  agree with the reading.

## 5. The towns at the start (T1a2)

### 5.1 The town's setup (`town_setup` 0x5C0670, read whole)

From `setup_towns` 0x4CAA70, per town record (`town_setup`, §2.4): the faction (+0x68), owner −1
then the record's (`town_change_owner` 0x4C61E0, T1b's), the second army cleared, the garrison
(§5.2), the buildings (`town_buildings_setup`, §5.3), the "every built" set (+0x158: the built
set with each built building's superseded ones, 0x6A8BB8), the spells' bytes (+0x70 to the
town's +0xD4), then 0x5BE600 with the record (not read; the guild's spells, increment 4,
presumably).

### 5.2 The garrison

- **A custom garrison** (+0x19): the seven slots as the map gives them; a stack with a count above
  0 and a code from −2 down (**the random stacks**) becomes the town's faction's creature of level
  (−2 − code) / 2, upgraded when the code is odd (`town_creatures` 0x6747B4); a count of 0 leaves
  the slot empty, so a custom empty garrison stays empty.
- **No custom garrison:** empty; then **a neutral town** draws: for levels 1 to 4,
  `random_between(1, 100)` at most 33, 33, 20, 13 (0x688E84) adds the faction's unupgraded
  creature of that level, `random_between` of 8–15, 5–7, 3–5, 1–3 (0x688E94, 0x688EA4) of them
  (`army_add_stack` with slot −1). An owned town's stays empty.
- **The census:** neutral towns: 497 without a garrison (drawn), 41 custom, 41 custom and empty;
  owned towns: 858 without (empty), 19 custom, 12 custom and empty. The 12 random stacks (7
  maps, T1a1) are in custom garrisons of random towns.
- **VCMI** (`initializeNeutralTownGarrison`): the same four rows with level 4 at 14 in 100
  (TD-4), and only when the town has no stack, so it also fills an explicitly empty one (its own
  FIXME; TD-5). **The wiki** ("Neutral Town Garrison"): the four rows as read, 13 in 100, and
  only when the map defines no garrison.

### 5.3 The buildings (`town_buildings_setup` 0x5C08C0, read whole)

1. The recruits, the built and the every-built sets cleared; **the Village Hall** (10) built.
2. **Forbidden** from the start: the Grail building (26) on a map whose victory is to build the
   Grail, in a town that condition does not name (0x5F2160, not read); the Shipyard (6) when the
   faction has none or neither (column − 1, row + 2) nor (column + 1, row + 2) from the entrance
   is a boat's spot (0x5C0C90, not read).
3. **A custom set** (+0x5): the map's forbidden bits (+0x10) through `map_building_ids` 0x6888C0
   added to the forbidden; a forbidden horde (18, 24) forbids its upgrade (19, 25). The buildable
   set (+0x160) is the faction's (0x6976F0) less the forbidden. The built: each of the map's
   built bits (+0x8) with **the buildings it supersedes** (0x6A8BB8, computed at start-up by
   0x4EB730; named from its use, as `town_build` removes them from the built set: so presumably a
   Capitol brings the lower halls and a Castle the Citadel and Fort), less what is not buildable;
   each built in id order (`town_build` 0x5BE930).
4. **The standard set** (+0x5 clear): the buildable set as above; **a Fort** when the map's flag
   (+0x18) is set; **an owned town** a Tavern (5), and **only with a Fort** the level-1 dwelling
   (30) and, at `random_between(1, 100)` at most 30, the level-2 one (31); **a neutral town** both
   dwellings (30, 31), with a Fort or without.
- **The census:** standard sets: owned with a Fort 495, owned without 51, neutral with 205,
  neutral without 143; custom sets: none builds a forbidden building; the Shipyard built in 9
  towns, the Grail in 10; 7 maps' victory is to build the Grail.
- **VCMI** (`initTowns`, `startingDwellingChances` [100, 50]): the Village Hall; a Tavern when
  owned; **the level-1 dwelling always and the level-2 one half the time, for every standard
  town**, owned or neutral, Fort or not (TD-3); a built Shipyard without water removed.
- **Others:** the HotA team's documentation of its own towns: the first two dwellings are built in
  a neutral town or one with a Fort, a Tavern in a town owned at the start, and "towns with Fort
  always have level 2 dwelling" (HotA's own rule; the reading's 30 in 100 is SoD's); a fandom page
  on random maps: player towns with a Tavern, the level-1 dwelling and sometimes the level-2,
  neutral ones without a Tavern. The Lazy wiki's Town page says nothing on the buildings.

## 6. Disputes, quirk candidates and open points (T1a2)

| Id | What | Reading | VCMI | Wiki, others | Shipped maps |
|---|---|---|---|---|---|
| TD-3 | A standard town's dwellings | owned: none without a Fort; with one level 1, level 2 at 30 in 100; neutral: levels 1 and 2 | levels 1 always, 2 at 50 in 100, every town | HotA: levels 1 and 2 in a neutral town or one with a Fort (level 2 always, HotA's); fandom: level 1, sometimes 2 | 894 standard towns, 51 owned without a Fort |
| TD-4 | A neutral town's level-4 roll | 13 in 100 | 14 | 13 (Lazy wiki) | 497 towns draw |
| TD-5 | A neutral town's explicitly empty custom garrison | stays empty | drawn (VCMI's FIXME) | only an undefined garrison is drawn | 41 towns |
| TD-6 | A hero on a town's blocked tile | only right of any town's entrance, onto it | any blocked tile of its own owner's town, onto the entrance | — | 23, all right of their own town's entrance |

- **T-2 (quirk candidate): the starting-hero choice is not reserved.** `new_game_heroes` marks a
  human's chosen hero (0x4BFF83), the map read's `hero_pool_init` frees it again (0x4C4E6B), and
  neither `load_hero_body` nor `main_town_heroes` checks a choice before placing it. So another
  colour's random hero, or an earlier colour's main-town draw, can take the chosen hero, which is
  then placed a second time for its chooser: one hero type on two tiles and in two players' lists
  (state-corrupting). Exposure: any start where a colour chooses and another colour can draw its
  faction's heroes. Proposal: fixed (a state-corrupting bug, `AGENTS.md`): the choices are
  reserved before any draw, as VCMI's; a batch TWN-1 row only if a cheap way to force the draw
  exists.
- **T-3 (quirk candidate): a random hero can take a later hero's type.** A placed hero or
  prisoner marks its type only when its body is read, so a random hero earlier in the file can
  draw it (when allowed); the later body then overwrites the type's record (`hero_setup` by type).
  None on the shipped maps (no placed hero is allowed). Proposal: fixed: every placed hero's and
  prisoner's type leaves the pool before the random heroes are drawn (VCMI's order).
- **T-4 (quirk candidate): the draws' empty cases.** A generating main town whose position names
  no town (the town pointer stays null and is used); a draw with no free hero at all
  (`random_between(1, 0)`, then a read below the list); a faction outside 0 to 8 drawing
  Castle's classes. None on the shipped maps (all 553 main towns match; 156 heroes). Proposal:
  the import refuses an unmatched main town (typed), a draw with nothing free fails typed (VCMI
  throws), the faction is always one of the nine.
- **Open, for T1b's checkpoint and T2:** the random hero in increment 1 (`TWN-Q2`; it shares
  §4.4's draw, so it costs little more than the main town's hero); who may choose a starting
  hero and from which list (E10's setup seam; the original's offer is §4.3, a computer colour's
  choice honoured only by a random hero); the generated hero's spells from the guild at once and
  the Shipyard's spot (increment 4); the superseded buildings (0x6A8BB8) and the factions' sets
  are increment 2's content, increment 1 needing the halls' and forts' chains and the first two
  dwellings; the neutral garrison's table and the 30 are constants of the executable (ADR-0006
  tier 2, with this reading as their source). TD-3 and TD-5 are batch TWN-1 rows (an owned
  standard town without a Fort; a neutral town with an empty custom garrison).
