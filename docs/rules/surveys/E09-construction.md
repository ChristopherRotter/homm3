# E09 construction survey (E09 T12): the build command, the requirements and costs, the town events

Date: 2026-11-07. Scope: E09 increment 2 (construction with the town events; the epic's decisions
37 and 38), input for T13's ADR amendment drafts, the design note (Part D of `E09-design.md`) and
the cut. In two parts, cut at T12's start (the user, 2026-11-06): **T12a** the build command's
checks, what a build does, the cost and requirement tables, VCMI and the wiki compared (§0 to §7);
**T12b** the town events (their day, buildings, resources and creatures), the census of the
shipped maps (forbidden buildings, custom sets, the town events) and the checkpoint with the user
(§8 to §12, 2026-11-07); **T14c** scored batch TWN-2 (§13, 2026-11-07). Input, not rules (`README.md`): a fact becomes binding when a task writes it into the
catalog with its sources and tests. The building ids are the original's (`docs/formats/h3m.md`,
"Town buildings"; survey `E09-towns.md` §16): 0 to 4 the mage guilds, 5 the Tavern, 6 the
Shipyard, 7 to 9 the Fort, Citadel, Castle, 10 to 13 the Village Hall, Town Hall, City Hall,
Capitol, 14 the Marketplace, 15 the Resource Silo, 16 the Blacksmith, 17 to 25 the faction's
specials and hordes, 26 the Grail's, 30 to 36 the dwellings by level, 37 to 43 their upgrades.

**Sources** (retrieved 2026-11-06 and -07):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml` or `structs.toml`. The
  reading follows the human player's path (the town screen's hall) and names the computer's where
  it differs. **Read whole:** `town_buildable_now` 0x5C0F20, `town_build_possible` 0x5C0E60,
  `town_can_buy` 0x5C0D20, `town_buy` 0x5BF3C0, `town_build_effects` 0x5BEDE0, `player_has_capitol`
  0x4B9F40, `town_building_allowed` 0x5C12A0, `town_has_boat_spot` 0x5BF4E0, `town_boat_spot`
  0x5C0C90, `grail_victory_town` 0x5F2160, `building_costs_load` 0x5C14C0, `building_cost`
  0x5C1080, `building_cost_copy` 0x5C10F0, `building_cost_list` 0x5C1180, `town_grail_build`
  0x5D30D0, `build_dialog_requirements_text` 0x5D5BE0, `hero_has_artifact` 0x4D91B0. **Read in
  part:** `hall_dialog_fill` 0x461190 (the slots and their states), `hall_dialog_proc` 0x461AB0
  (a slot's click), `build_dialog_run` 0x5D5F30 (its Build button and the payment), `town_hall_run`
  0x5D27B0, `town_screen_show_build` 0x5D6A80 (its call of the build), `town_screen_proc`
  0x5D3240 (the hall's and the Marketplace's cases), `new_day` 0x4C7FE0 at its towns' loop,
  `town_change_owner` 0x4C61E0 at the day's flag, `town_buildings_setup` 0x5C08C0 at the
  Shipyard's spot, `town_ship_update` 0x5BF210 (the ship, 20), `ai_town_visit` 0x5253D0 at its
  Grail. **Tables dumped** (a throwaway script, not committed): the faction lists 0x63EC80 (again,
  for the direct requirements), the hall's slots 0x66CF98 with their counts 0x66D03C.
  **T12b read whole:** `town_timed_events` 0x4CDA10, `town_events_apply` 0x5BFEB0; **in part:**
  `load_town_body` 0x5019F0 at its events, `player_turn_start` 0x4CC7D0 at its start,
  `town_event_buildings_message` 0x5C0220 (its text and its human test); the table
  `map_building_ids` 0x6888C0 dumped. **T14c (§13) read whole:** `town_screen_pieces` 0x5C33F0;
  **in part:** `build_dialog_create` 0x5D55C0 at its end, `town_screen_init` 0x5C63C0 at its
  calls, the readers of the bits 22, 23 and 27 at their faction tests; the faction lists dumped
  again. **Not followed:** what the computer chooses to build (0x42AE00 and its siblings, beyond their
  calls of the tests), network play, a campaign's starting buildings (0x484650), the tutorial's branch (game +0x1F69D: the Castle's Griffin Tower
  without the Blacksmith), the town screen's drawing and animations.
- The text table `Building.txt` (both archives: byte-identical), its seven numeric columns only.
- VCMI 1.7.5 (tag `1.7.5`, a sparse clone; T12b's files in §9): `lib/callback/CGameInfoCallback.cpp`
  (`canBuildStructure`), `server/CGameHandler.cpp` (`buildStructure`), `server/NetPacksServer.cpp`
  (`visitBuildStructure`), `lib/gameState/GameStatePackVisitor.cpp` (the new day),
  `lib/entities/faction/CTownHandler.cpp` (`loadLegacyData`), `lib/mapObjects/IObjectInterface.cpp`
  and `CGTownInstance.cpp` (the Shipyard's tile), `config/factions/*.json` and
  `config/buildingsLibrary.json` (the requirements, compared by a throwaway script).
- The wiki and others: the game's manual (Heroes III, "Towns", the Hall;
  https://issuu.com/ngutri.com/docs/heroes3_manual/s/13274881); the Heroes 3 wiki's difficulty
  page as the shoutwiki copy has it (https://heroes.shoutwiki.com/wiki/Difficulty); the Might and
  Magic fandom wiki's Conflux page (https://mightandmagic.fandom.com/wiki/Conflux_(town)) and its
  Shipyard page (https://mightandmagic.fandom.com/wiki/Shipyard); the Heroes 3 wiki's dwelling
  page (https://heroes.thelazy.net/index.php/Dwelling); T12b: the shoutwiki copy of the Heroes 3
  wiki's map editor objects page (https://heroes.shoutwiki.com/wiki/Map_Editor_-_Objects).

## 0. Findings in short

**T14c** (§13, batch TWN-2 scored): **TD-9** as the reading (the upgraded Altar of Fire needs
the upgraded Altar of Air; VCMI and the wiki refuted); **TD-11** refused as VCMI does: the build
dialog's constructor greys Build when the town's owner is not the current player (0x5D5AE2, not
read by T12a), so only the owner builds and pays; decisions 41 and 42 stand. Not predicted: the
town screen builds a display piece into the standing set (Castle 23, Stronghold 27, Fortress 23,
Conflux 22), outside every faction's list; nothing for the engine.

**T12b** (§8 to §12):

- **The town events** (§8) are one list for the map, read at each player's turn start right after
  the map's timed events; a record fires as a map event does (its day, its players, its kind) and
  **only when its town's owner is the current player**: the resources and the message first, then
  the buildings, then the creatures.
- **The buildings** are the event's bits through the town's faction, less what stands, what the
  faction lacks, what the event's own buildings supersede and the Shipyard without its spot; built
  in id order **with no requirement, forbidden, Capitol, Grail, cost or day's-mark test**, with the
  heroes' visit. **The creatures** go to the level's upgraded dwelling, else its dwelling, else are
  lost. VCMI and the wiki agree (§9) but for **TD-12** (an every-day event: VCMI fires it from the
  day before its first) and two cases no shipped map has.
- **The census** (§10): 173 custom sets forbid something (39 maps; the level-7 dwelling most); 369
  town events on 110 towns of 29 maps: 70 build (27 builds without a requirement in three maps, six
  Capitols that can be a player's second: **T-8**), 56 add creatures, 105 give resources, 199 carry
  a message only; 98 repeat, 18 of them every day.
- **The checkpoint** (§11, §12, `TWN-Q8`): TD-9 the executable's list and TD-11 the owner, each
  with a row of batch TWN-2; one a town a day for every player; the town events as the original
  plays them (T-8 kept, TD-12 the original's day).

**T12a** (§1 to §7):

- **The human's build** is the hall's (§1): the town screen's hall opens a list of slots, each the
  next building of a chain; a slot's Build button is enabled when the building is not built, is in
  the town's **buildable-now set** (computed when the hall opens, empty once the town built that
  day) and the current player can pay it; Build pays the cost from **the current player's**
  treasury and builds with the day's mark and the heroes' visit. The buildable-now set
  (`town_buildable_now` 0x5C0F20): not built (the every-built set +0x158), the requirements all in
  the every-built set, not forbidden (+0x160), the Shipyard only with its boat spot, the Capitol
  only while the player has none.
- **One a town a day** (§3): the town's +0x2 is set at a build (not for the ship), cleared by every
  new day for every town; a computer-only team on Easy or Normal gets 2 and builds every other day
  (the wiki says the same). The Grail's building ignores the mark but sets it.
- **What a build does** (§4, `town_build_effects` 0x5BEDE0): increment 1's `town_build` (the
  standing set, the supersedes, the hordes, the dwelling's first creatures), the every-built set
  again, the guild's level and counts and the heroes' new spells, the map's picture at a first fort
  or a Capitol, the scenario's tests, the Tower's Lookout Tower (radius 20) and Skyship (the whole
  map), the garrison hero's and the visitor's visit of the town's buildings.
- **The costs** (§5) are `Building.txt`'s, read at start-up into three tables (neutral by id, the
  faction's 17 to 25, the dwellings); VCMI reads the same file. **The requirements** (§6) are the
  executable's lists (survey `E09-towns.md` §16.1), tested against the every-built set (so a
  Castle satisfies a Fort); VCMI's agree but for the upgraded hordes (bought in the original's hall,
  automatic in VCMI, the same end state) and **TD-9** (§7).
- **Who builds:** the reading finds no owner test on the hall's path (the Marketplace's has one),
  and the town screen opens for an ally's town at a visit, so the original may let a visitor build
  in an ally's town at its own player's cost; VCMI builds for the owner only, paid by the owner
  (**TD-11**, §7).
- No quirk candidate; open points for T12b's checkpoint in §7.

## 1. The hall and the build (the human's path)

### 1.1 Reaching the hall

The town screen (`town_screen_open` 0x5BE210, survey `E09-towns.md` §7.2) opens for a human at a
visit of an own or an allied town, and from the town list for an own town. A click on the hall's
building (10 to 13, `town_screen_proc` 0x5D3240's switch on the building) calls `town_grail_build`
0x5D30D0 (§2.6), which, unless it built the Grail's building, ends in `town_hall_run` 0x5D27B0:
the hall dialog (`hall_dialog_fill` 0x461190, its messages `hall_dialog_proc` 0x461AB0), then, if
a building was chosen (the town manager's +0x1B8, -1 none), `town_screen_show_build` 0x5D6A80
with it. No step of this path tests
the town's owner against the current player (the build dialog does: §13, T14c); the Marketplace's case of the same switch does
(0x5D388A: the town's owner must be the current player).

### 1.2 The hall's slots

Per faction a list of slots (`hall_slots` 0x66CF98, 18 bytes a faction, `hall_slot_counts`
0x66D03C: Castle 16, Rampart 17, Tower 18, Inferno 18, Necropolis 18, Dungeon 18, Stronghold 18,
Fortress 17, Conflux 17): the seven dwellings, the Mage Guild 0, the Tavern 5, the Shipyard 6 (the
four factions with one), the Marketplace 14, the Fort 7, the Village Hall 10, the Blacksmith 16,
the horde 18 (and 24 for Rampart and Inferno), the faction's specials. When the hall opens, each
slot shows the next building of its chain (`hall_dialog_fill`): a dwelling its upgrade once the
dwelling is built; 0 → 1 → … up to the faction's highest guild (Castle 4, Stronghold and Fortress
3); 7 → 8 → 9; 10 → 11 → 12 → 13; 14 → 15; the horde its upgrade (19, 25) once the level's
upgraded dwelling is built; the Castle's Tavern its Brotherhood (22) and its Shipyard the
Lighthouse (17); the Rampart's Mystic Pond (17) the Fountain of Fortune (21); the Fortress' 21
the 22. **Every building of the faction's list but the Grail's (26) is offered by some slot**
(the slots and chains against the lists of §6, a throwaway script); the Village Hall stands from
the start (survey `E09-towns.md` §5.3), so its slot shows the Town Hall.

### 1.3 A slot's state

`hall_dialog_fill` computes, once when the hall opens, the town's buildable-now set
(`town_buildable_now` 0x5C0F20, §2.1; **empty when the town's day mark +0x2 is set**, 0x461710)
and the slots' buildings the current player can pay (each of the seven resources of the player
record +0x9C at least the cost's, 0x46177A). A slot is then: built (its building in the every-built
set +0x158); buildable (in the set and paid); short of resources; or not now, drawn one way when
`town_build_possible` 0x5C0E60 (§2.2) says it can still be built and another when it never can.
Opening the hall again recomputes it.

### 1.4 The build

A slot's click (`hall_dialog_proc`) opens the building's dialog (`build_dialog_run` 0x5D5F30)
with its Build button disabled unless the building is not built, is in the buildable-now set and
is paid (0x462066), and (§13, T14c) the dialog's constructor greys it when the town's owner is
not the current player (0x5D5AE2); a right click shows the same dialog as a popup. The text under it lists the
missing requirements (`build_dialog_requirements_text` 0x5D5BE0: the requirement set less the
every-built set, less what another missing one supersedes but for the hordes 18 and 24, by name),
or the Capitol's or the Shipyard's own refusal, or the never-buildable text. **Build** (0x7802)
stores the building in the manager's +0x1B8 and takes each non-zero resource of the cost
(`building_cost_list` 0x5C1180) from **the current player's** record (0x5D679C), not the town
owner's. The hall closes; `town_screen_show_build` builds through `town_build_effects(b, mark 1,
visit 1)` (§4) and redraws the screen.

## 2. The tests

### 2.1 Buildable now (`town_buildable_now` 0x5C0F20, read whole)

A 64-bit set over the ids 0 to 43: each id not in the every-built set (+0x158) whose requirement
mask (`building_requirements` 0x697798, the transitive closure of the faction's list, §6) is all in
the every-built set (the tutorial lifts the Blacksmith from the Castle's Griffin Tower, 32: not
followed); then kept to the buildable mask (+0x160: the faction's list less the forbidden, survey
`E09-towns.md` §5.3), the Shipyard (6) out while the town has no boat spot (+0x8 = 0xFF, §2.5),
the Capitol (13) out while `player_has_capitol` 0x4B9F40 (§2.4). It does not read the day's mark:
its callers do (the hall, §1.3).

### 2.2 Ever buildable (`town_build_possible` 0x5C0E60, read whole)

The building in the buildable mask; the Shipyard: its boat spot, nothing else; the Capitol: false
while the player has one; then every requirement either built (every-built set) or still allowed
(the buildable mask). A building one of whose requirements is forbidden is never buildable. It does
not test "not built".

### 2.3 The computer's buy (`town_can_buy` 0x5C0D20 and `town_buy` 0x5BF3C0, read whole)

`town_can_buy`: false when the town's day mark +0x2 is set or the building is not in the buildable
mask; the Shipyard: its boat spot alone; the Capitol: no Capitol of the player alone; any other: not
in the every-built set and its requirements in it (the tutorial's exception as §2.1). So for 6 and
13 it tests neither the requirements nor (for 6) "not built". `town_buy`: a town without an owner
buys nothing; `town_can_buy`; the cost (§5); a computer owner's hook (0x526D20, not read) and the
day's mark again; every resource of **the owner's** record at least the cost's, then taken; then
`town_build_effects(b, 1, 1)`. Its callers are all the computer's (nine functions, among them
`ai_town_build_choice` 0x42AE00, `ai_boat_create` 0x430F80 and `ai_town_visit` 0x5253D0; the
others buy a building of their own choosing: the Marketplace, the Tavern, a mage guild, a
faction's special). The gap
for 6 and 13 has no effect found: the Shipyard requires nothing in any faction (§6),
`ai_boat_create` buys it only when it is not in the every-built set, and the general builder
(`ai_town_build_choice`) picks from `town_buildable_now`'s set.

### 2.4 One Capitol (`player_has_capitol` 0x4B9F40, read whole)

True when any of the player's towns (player record +0x3E, +0x40) has the Capitol (13) in its
standing set (+0x150). Read by §2.1 to §2.3 and the hall's text. A capture that gives a player a
second Capitol takes the captured one down to a City Hall (`capitol_limit` 0x5BEC60, survey
`E09-towns.md` §8.4).

### 2.5 The Shipyard's water (`town_boat_spot` 0x5C0C90, read whole)

Called only by `town_buildings_setup` at the start, for a faction whose list holds the Shipyard:
the tile (entrance column − 1, row + 2), then (column + 1, row + 2), on the entrance's level; the
first that is on the map, water (terrain 8), not blocked (the tile's +0xC bit 8) and either without
an object (bit 12 clear) or with a boat (class 8) becomes the town's boat spot (+0x8, +0x9); with
neither the Shipyard is forbidden from the start (survey `E09-towns.md` §5.3). The spot is never
computed again: the Shipyard's water is a fact of the start. `town_has_boat_spot` 0x5BF4E0 tests
+0x8 ≠ 0xFF.

### 2.6 The Grail's building (`town_grail_build` 0x5D30D0, read whole)

At the hall's click, before the hall: the town's visitor (+0x10), else its garrison hero (+0xC),
carrying the Grail (artifact 2, worn or in the backpack: `hero_has_artifact` 0x4D91B0), the Grail's
building (26) not standing, the current player human: with 26 in the buildable mask
(`town_building_allowed` 0x5C12A0) a question (GENRLTXT), and on yes the Grail taken from the hero
(`hero_take_artifact` 0x4E2DD0, not read) and 26 built through `town_screen_show_build` (so **with the day's mark**,
which it never tests), the scenario's test, and the click ends there (no hall); without it a
message once a game (game +0x90), then the hall. No cost, no requirement (every
faction lists 26 with none, §6). The computer builds it the same way at its visit (`ai_town_visit`
0x525412). The buildable mask lacks 26 in a town that a "build the Grail" victory does not name
(`grail_victory_town` 0x5F2160, read whole: the condition's tile is the town's entrance, or the
condition names none). Increment 5's.

## 3. One a town a day (the town's +0x2)

- **Set** by `town_build_effects` with its mark (every build of §1.4, §2.3, §2.6) unless the
  building is the ship (20, built when a boat lies at the spot: `town_ship_update` 0x5BF210, not a
  construction): 2 when the game's difficulty (game +0x1F6D8) is below 2 (Easy, Normal) and the
  owner's team has no human (`team_has_human`), else 1 (0x5BEE6C).
- **Cleared** by `new_day` for every town (0x4C8067): to 0, but a value above 0 of a town whose
  owner's team has no human on Easy or Normal is counted down by one. So a human builds once a
  town a day; a computer-only team on Easy or Normal once every two days.
- **A change of owner** keeps it, but clears it when the old owner's team has no human and the
  new owner's team has one (`town_change_owner` 0x4C63B8): a town built in today by its old owner
  cannot be built in again today by a human who took it from a human.
- The wiki: the manual's Hall section says only one structure a day may be built in each town;
  the difficulty page says the computer builds once every two days on Easy and Normal and every
  day from Hard. VCMI: the town's `built` against `TOWNS_BUILDINGS_PER_TURN_CAP` (1), the Grail's
  building counted too, every town's `built` reset at each new day; no difference by difficulty.

## 4. What a build does (`town_build_effects` 0x5BEDE0, read whole)

Arguments: the building, the day's mark, the visit. In order:

1. `town_build` 0x5BE930 (survey `E09-towns.md` §16.2: the standing set, the supersedes, the
   hordes and their upgrade, a dwelling's first creatures, the hall companions).
2. The day's mark (§3).
3. The every-built set (+0x158) recomputed: the standing set and what each standing building
   supersedes.
4. A mage guild (0 to 4): the guild's level (+0x14) the building + 1 and its per-level counts
   again; the Tower's Library (22) the counts again (one more a level); then for every building
   `town_heroes_learn_spells` 0x5BE030 (increment 4).
5. The map's picture redrawn (`object_template_redraw` 0x4C9990) when the Capitol or a first fort
   level (7, 8 or 9) stands now and did not before.
6. The scenario's tests (0x5F1D40, `scenario_condition_met`; E10's).
7. The Tower: the Lookout Tower (21) reveals a disk of radius 20 around the town for its owner;
   the Skyship (26) the whole map, on both levels when there are two (`reveal_disk` 0x49D160).
8. With the visit: the garrison hero's, then the visitor's visit of the town's buildings
   (`town_visit_buildings` 0x5BD8E0; increment 4's services).

VCMI's `buildStructure` adds the automatic buildings, the dwelling's creatures, the Portal of
Summoning's, the costs, the guild's spells, the Lookout's reveal, the visitors' visit (the garrison
hero first, as the original) and the victory tests: the same steps.

## 5. The costs (`building_costs_load` 0x5C14C0, read whole)

- **`Building.txt`** (seven columns: wood, mercury, ore, sulfur, crystal, gems, gold, the player
  record's resource order) read at start-up into three tables of seven i32: **the faction's
  buildings 17 to 25** (nine rows a faction from row 4, two rows between factions; 0x6A82DC), **the
  neutral buildings 0 to 16** (17 rows from row 104, the same for every faction; 0x6A80F8), **the
  dwellings 30 to 43** (14 rows a faction from row 125; 0x6A9818). `building_cost` 0x5C1080 and
  `building_cost_copy` 0x5C10F0 pick by id: below 17 the neutral row, below 30 the faction's row
  9 × faction + id − 17, else the dwelling's row 14 × faction + id − 30. The ids 26 to 29 fall
  past the faction's nine rows (into the next faction's); no path asks their cost.
- **The neutral costs** (wood, mercury, ore, sulfur, crystal, gems, gold): Mage Guild 1 (5, 0, 5, 0,
  0, 0, 2000), 2 (5, 4, 5, 4, 4, 4, 1000), 3 (5, 6, 5, 6, 6, 6, 1000), 4 (5, 8, 5, 8, 8, 8, 1000), 5
  (5, 10, 5, 10, 10, 10, 1000); Tavern (5, 0, 0, 0, 0, 0, 500); Shipyard (20, …, 2000); Fort (20,
  0, 20, …, 5000); Citadel (0, 0, 5, …, 2500); Castle (10, 0, 10, …, 5000); Village Hall (1 gold,
  never bought); Town Hall 2500, City Hall 5000, Capitol 10 000 gold; Marketplace (5 wood, 500);
  Resource Silo (5 ore, 5000); Blacksmith (5 wood, 1000).
- **The faction's rows** hold blanks (all zero) where the faction has no building: 20 in every
  faction (the ship), 23 where it is a hall companion or absent, 24 and 25 without a second horde.
  The upgraded hordes cost their row (the Castle's Royal Griffin Horde 1000 gold, as its horde).
- **VCMI** reads the same file the same way (`CTownHandler::loadLegacyData`): the faction's rows
  from 17 until a blank row, the neutral rows for every faction, 14 dwelling rows a faction. The
  fandom wiki's Conflux dwelling costs agree with the rows (spot checks: the upgraded Altar of Air 2
  wood, 2 mercury, 2 gems, 1500 gold; the upgraded Altars of Water and Fire 5 ore, 5 mercury, 2000;
  the upgraded Altar of Earth 5 sulfur, 1000). RoE's file is E11's.

## 6. The requirements

The faction lists 0x63EC80 (survey `E09-towns.md` §16.1), each building with its **direct**
requirements (the closure is `building_requirements`), dumped again:

- **Castle** (35): 0; 1 (0); 2 (1); 3 (2); 5; 6; 7; 8 (7); 9 (8); 11 (5); 12 (11, 16, 0, 14); 13 (12, 9); 14; 15 (14); 16; 17 (6); 18 (32); 19 (39); 21 (33); 22 (5); 26; 30 (7); 31 (30); 32 (33); 33 (30, 16); 34 (33, 0); 35 (21); 36 (34); 37 (30); 38 (31); 39 (32); 40 (33); 41 (34); 42 (35); 43 (36).
- **Rampart** (37): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 7; 8 (7); 9 (8); 11 (5); 12 (11, 0, 16, 14); 13 (12, 9); 14; 15 (14); 16; 17; 18 (31); 19 (38); 21 (17); 22 (18); 24 (34); 25 (41); 26; 30 (7); 31 (30); 32 (30); 33 (32); 34 (32); 35 (33, 34); 36 (35, 1); 37 (30); 38 (31); 39 (32); 40 (33); 41 (34); 42 (35); 43 (36, 2).
- **Tower** (36): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 7; 8 (7); 9 (8); 11 (5); 12 (11, 14, 16, 0); 13 (12, 9); 14; 15 (14); 16; 17 (14); 18 (31); 19 (38); 21 (7); 22 (0); 23 (0); 26; 30 (7); 31 (30); 32 (30); 33 (31, 32, 0); 34 (33); 35 (33); 36 (34, 35); 37 (30); 38 (31); 39 (32); 40 (33, 22); 41 (34); 42 (35); 43 (36).
- **Inferno** (37): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 7; 8 (7); 9 (8); 11 (5); 12 (11, 16, 14, 0); 13 (12, 9); 14; 15 (14); 16; 18 (30); 19 (37); 21 (7); 22 (8); 23 (0); 24 (32); 25 (39); 26; 30 (7); 31 (30); 32 (30); 33 (31); 34 (33); 35 (33, 0); 36 (34, 35); 37 (30); 38 (31); 39 (32); 40 (33); 41 (34, 1); 42 (35); 43 (36).
- **Necropolis** (36): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 6; 7; 8 (7); 9 (8); 11 (5); 12 (11, 14, 16, 0); 13 (12, 9); 14; 15 (14); 16; 17 (7); 18 (22); 19 (37, 22); 21 (0); 22 (30); 26; 30 (7); 31 (30); 32 (30); 33 (31); 34 (31, 0); 35 (33, 34); 36 (35); 37 (30); 38 (31); 39 (32); 40 (33, 21); 41 (34); 42 (35); 43 (36).
- **Dungeon** (36): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 7; 8 (7); 9 (8); 11 (5); 12 (11, 16, 14, 0); 13 (12, 9); 14; 15 (14); 16; 17 (14); 18 (30); 19 (37); 21 (0); 22; 23; 26; 30 (7); 31 (30); 32 (30); 33 (32, 31); 34 (33); 35 (33); 36 (34, 35, 1); 37 (30); 38 (31); 39 (32); 40 (33); 41 (34); 42 (35); 43 (36, 2).
- **Stronghold** (34): 0; 1 (0); 2 (1); 5; 7; 8 (7); 9 (8); 11 (5); 12 (11, 0, 14, 16); 13 (12, 9); 14; 15 (14); 16; 17 (7); 18 (30); 19 (37); 21 (14); 22 (16); 23 (7); 26; 30 (7); 31 (30); 32 (30); 33 (32); 34 (31); 35 (33); 36 (34); 37 (30); 38 (31, 37); 39 (32, 16); 40 (33, 0); 41 (34); 42 (35); 43 (36).
- **Fortress** (34): 0; 1 (0); 2 (1); 5; 6; 7; 8 (7); 9 (8); 11 (5); 12 (11, 14, 16, 0); 13 (12, 9); 14; 15 (14); 16; 17 (11, 21); 18 (30); 19 (37); 21 (7); 22 (21); 26; 30 (7); 31 (30); 32 (30); 33 (32); 34 (32, 31); 35 (31); 36 (35, 33); 37 (30, 5); 38 (31); 39 (32); 40 (33); 41 (34, 15); 42 (35); 43 (36).
- **Conflux** (35): 0; 1 (0); 2 (1); 3 (2); 4 (3); 5; 6; 7; 8 (7); 9 (8); 11 (5); 12 (11, 14, 16, 0); 13 (12, 9); 14; 15 (14); 16; 17 (14); 18 (30); 19 (37); 21 (0); 26; 30 (7); 31 (30, 0); 32 (30, 0); 33 (31); 34 (32); 35 (33, 34); 36 (35); 37 (30); 38 (31); 39 (32); 40 (33, 38); 41 (34); 42 (35, 1); 43 (36).

**Tested against the every-built set** (§2.1): a requirement holds while it or a building that
supersedes it stands (a Castle for a Fort, a Capitol for a Town Hall, an upgraded dwelling for its
dwelling, a horde's upgrade for the horde). The Village Hall is no one's requirement (it always
stands). **The upgraded hordes** (19, 25) need only the level's upgraded dwelling: the hall's
horde slot offers the upgrade once that dwelling stands (§1.2), at the upgrade's cost (§5), and
`town_build`'s horde rule (survey `E09-towns.md` §16.2) gives the same standing set whichever
of the horde, the upgraded dwelling or the upgraded horde comes first.

**VCMI 1.7.5 compared** (the closures, a throwaway script over `config/factions/*.json` with
`buildingsLibrary.json`, a building's `upgrades` counted as a requirement, the `null` entries
dropped): the same buildings in every faction, plus VCMI's automatic Village Hall (10), the ship
(20) where there is a Shipyard, and the hall companions 27 to 29 in Rampart, Necropolis and
Conflux; the same closures but (a) **the upgraded hordes**: VCMI's also need the horde (18, 24) and
are automatic (`BUILD_AUTO`, refused by `buildStructure`), built when both stand: the same end
state, another buildable id; (b) **the Conflux's upgraded Altar of Fire** (40): the reading 33 and
the upgraded Altar of Air 38, VCMI 33 and the base Altar of Air 31 (which 33 needs anyway): TD-9.
The fandom wiki's Conflux page gives the upgraded Altar of Fire the Altar of Fire alone.

## 7. Disputes, quirk candidates and open points (T12a)

| Id | What | Reading | VCMI | Others | Increment |
|---|---|---|---|---|---|
| TD-9 | The Conflux's upgraded Altar of Fire (40) | needs the Altar of Fire (33) and the upgraded Altar of Air (38): the executable's list, read by every test (§2) | the Altar of Fire (and the Altar of Air, which it needs) | the fandom wiki: the Altar of Fire | 2 |
| TD-11 | Who builds in a town, and who pays | the hall's path tests no owner (§1.1) and pays from the current player's treasury (§1.4): a visitor in an ally's town would build there at its own player's cost; the computer's buy pays from the owner's (§2.3) | the owner only (`visitBuildStructure`: `throwIfWrongOwner`), the owner pays | the dwelling page: an ally's town's creatures cannot be hired (recruitment, not building); no source on building | 2 |

- **Quirk candidates:** none in T12a (next `T-8`).
- **Open for T12b and its checkpoint:** TD-9 and TD-11 (each a row for an experiment batch
  TWN-2 if the user wants the original observed: a Conflux town with the Altar of Fire and the
  Altar of Air, the hall's state of the upgraded Altar of Fire; a human's hall in an ally's
  town); the computer's every other day on Easy and Normal (§3: the engine has no difficulty and
  no computer player yet, E10 and E13; the recommendation for T13: one a town a day for every
  player, the handicap left to the computer player's epics); the day's mark kept across a capture
  but for the computer-to-human case (§3); the Grail's building (§2.6) and the Lookout's and
  Skyship's reveals (§4) as increments 5 and 4 build them; the tutorial's exception (not
  followed, out of scope).
- **For T12b:** the town events build through `town_build_effects` too (`town_timed_events`
  0x4CDA10 calls `town_events_apply` 0x5BFEB0, which calls it): whether with the day's mark and
  the visit is T12b's reading.

## 8. The town events (T12b; `town_timed_events` 0x4CDA10 and `town_events_apply` 0x5BFEB0, read whole)

### 8.1 The record and its list (`load_town_body` 0x5019F0 at its events)

- **Read** for each town after its spells: a u32 count, then per event `load_timed_event` 0x4FC1A0
  (the map's timed event: the message, seven resources, the players' bits, the human flag, 1
  before SoD, the computer flag, the first day stored as the file's + 1, the interval; E08 survey
  `E08-economy.md` §11), six bytes of buildings (the map's town building bits, `h3m.md`'s "Town
  buildings" order), seven u16 creature counts (levels 1 to 7) and four bytes skipped.
- **The record** (0x50 bytes, H3API's H3CastleEvent; `structs.toml` `town_event`) adds the town's
  index (+0x34: the number of town records before this town's, so the index its record and its
  town get), the buildings (+0x38, the file's 48 bits in a 64-bit set) and the creatures (+0x40).
- **The list:** one for the whole map (game +0x1FC00, begin +0x1FC04): the towns in the map's
  object order, each town's events in its file order. A grep of its offsets finds no other reader
  than `town_timed_events`; a random town's bits are read through its faction only when an event
  fires (§8.3), so its drawn faction applies.

### 8.2 When (`town_timed_events`, from `player_turn_start` 0x4CC7D0)

- At each player's turn start, **right after the map's timed events** (`map_timed_events`
  0x4CD910; E08 survey §7.5): a computer's turn start runs the two and nothing else; a human's
  runs the scenario's tests first (and nothing when its record's second human flag +0xE1 is
  clear, network play's case, not followed).
- **The day number** from the date (game +0x1F63E): (4 × month + week − 5) × 7 + day, as the
  map's events (E08 survey §7.1).
- **Each record of the list, in order, fires** when all of these hold: the current player is human
  and the human flag is set, or a computer and the computer flag is set; the current player's bit
  is in the players' bits; the day is the first day, or the interval is not 0, the day is past the
  first and (day − first) is a multiple of the interval (the map's events' test, E08 survey §11);
  **and the town's owner (the town's +0x1) is the current player.** So an event fires only at its
  town's owner's turn and only for an owner it names: a neutral town's never; a captured town's for
  its new owner when the event names it.
- **Fired:** `apply_timed_event` 0x4CD710 first (the resources to the current player, each floored
  at 0; the message for a human), then `town_events_apply` with the town.
- Not followed: the call from `new_game_start_other` 0x4EE3E0 (network play's path).

### 8.3 What it builds (`town_events_apply`)

- **The bits** are mapped through the town's faction (`map_building_ids` 0x6888C0, faction × 41 +
  bit; a bit the faction has no building for is 44, none). The loop runs over 44 bits although the
  map has 41: bits 41 to 43 would read the next faction's row, past the table for the Conflux; no
  shipped event sets them (§10).
- **Taken out:** what the town has (the every-built set +0x158: the building or one that supersedes
  it); what the faction's list lacks (`faction_buildings` 0x6976F0); what another building of the
  same event supersedes (`superseded_buildings` 0x6A8BB8: an event with the Fort and the Castle
  builds the Castle); the Shipyard (6) when the town has no boat spot (+0x8 0xFF, §2.5).
- **Not tested:** the requirements (§6), the forbidden set (+0x160), one Capitol a player (§2.4),
  the Grail (§2.6), the cost (§5) and the day's mark (§3); the owner's turn is the only "who".
- **Each building left, in id order,** through `town_build_effects` 0x5BEDE0 **without the mark and
  with the visit** (its arguments 0 and 1, 0x5C0053): all of §4 (increment 1's `town_build` with
  the standing set, the supersedes, the hordes and a dwelling's first creatures; the guild and the
  heroes' spells; the map's picture; the scenario's tests; the Tower's reveals; the garrison hero's
  then the visitor's visit of the buildings). So an event's build leaves the day's build free.
- **Two Capitols:** the one-Capitol rule outside the hall is `capitol_limit` 0x5BEC60, run only
  after a capture (`town_visit` 0x4AAFD0: the captured town loses its Capitol when its new owner
  holds another). An event's Capitol beside the owner's other one stands until one of the two towns
  changes hands (T-8, §11).
- **For a human owner** a message names the buildings with their pictures, eight a box
  (`town_event_buildings_message` 0x5C0220, read in part: the general text's +0x92C with the town's
  name; shown only when the current player is human and owns the town).

### 8.4 What it adds (the creatures)

- **Per level 1 to 7 with a count:** added to the town's recruits (+0x16) of the upgraded dwelling
  (index 7 + level − 1) when that dwelling (37 + level − 1) is in the every-built set, else of the
  dwelling (index level − 1) when it (30 + level − 1) is, **else lost**. After the buildings, so a
  dwelling the same event builds takes them (on top of its first creatures from `town_build`). No
  cap; the week's growth is not involved (increment 3).
- **For a human owner** a message with the creatures' pictures (`town_event_creatures_message`
  0x5C0400, inferred: its calls are 0x5C0220's).

## 9. VCMI and the wiki compared (T12b)

- **VCMI 1.7.5** (`server/processors/NewTurnProcessor.cpp`, `onPlayerTurnStarted` and
  `handleTownEvents`; `lib/mapping/CMap.cpp`, `CMapEvent::occursToday` and `affectsPlayer`;
  `server/CGameHandler.cpp`, `buildStructure` with `force`; `lib/mapping/MapFormatH3M.cpp`, the
  town's events; `lib/gameState/CGameState.cpp`, the hordes' bits remapped at the start): at a
  player's turn start, after the map's events, each of the player's towns' events (a town's in file
  order; the towns in the player's order, not the map's). The same day test but for an interval of
  1, which fires one day early (`occursToday`: (day − first − 1) % 1 is 0 on the day before;
  **TD-12**); the same players and kinds, plus a difficulty filter of VCMI's own map format; the
  resources given as the map's events'; each building of the faction not yet built through
  `buildStructure(…, force)`: **no requirement, forbidden, Capitol, Grail or cost test, the day's
  count `built` unchanged**, no visit inside it (the heroes visit every town of the player at every
  turn start, after the events); a Shipyard is not skipped without water (the reading skips it; no
  shipped event builds one, §10). The creatures go to the level's one count when the level has a
  dwelling, else are lost. One window with the message, the resources, the buildings and the
  creatures.
- **The wiki** (the shoutwiki copy of the Heroes 3 wiki's "Map Editor - Objects", its town events;
  https://heroes.shoutwiki.com/wiki/Map_Editor_-_Objects): the events run down the list, each on
  its days; only for the town's owner, of the kind the event names; a take larger than the supply
  leaves 0; an event may build a building, a forbidden one included; one already built does
  nothing, though the message still shows; no creatures without the level's dwelling (the editor
  offers the basic creatures; an upgraded dwelling's owner can upgrade them at once). It is silent
  on the requirements.
- **Agreement:** the days, the players and kinds, the owner only, the resources, no forbidden test
  (all three), no requirement test (the reading and VCMI; the wiki silent), the creatures lost
  without the level's dwelling (all three), the day's mark left alone (the reading and VCMI).
  **Differences:** TD-12; the Shipyard without its spot (no shipped case: the reading's skip); the
  towns' order (the map's in the reading, the player's in VCMI: it decides only the messages' order
  and, for a take floored at 0 beside a gain of another town of the same player on the same day,
  the amount taken; the shipped takes sit in one town each, §10).

## 10. The census (T12b; the 160 shipped maps, a throwaway example over `h3_map::read`, not committed)

- **Custom sets and forbidden buildings:** 574 custom sets in 100 maps (survey `E09-towns.md`
  §1.3's), 173 forbid something, in 39 maps (112 fixed towns, 61 random). The forbidden h3m bits,
  most first: the level-7 dwelling (39) 95, the Tavern (6) 63, the level-7 upgrade (40) 51, the
  level-6 dwelling (37) 45, the level-5 one (34) 37, the level-4 one (31) 32, the Fort (3) 20, the
  level-6 upgrade (38) 16, the Shipyard (16) 12, the level-5 upgrade (35) 12, the Grail (17) 10,
  the Silo (9) and the level-4 upgrade (32) 9 each, the Capitol (2) and the second special (19) 8
  each, 22 other bits 7 times or fewer; none is built as well. In 83 fixed towns a forbidden building is the direct requirement
  of a building not forbidden (225 such buildings): the buildable-now test (§2.1) blocks those too
  unless the requirement stands, with no list of their own.
- **The town events:** 369 on 110 towns of 29 maps (RoE 193, AB 45, SoD 131); 49 of the towns are
  random, 17 neutral at the start (their events fire only after a capture). **By content:** 70
  build, 56 add creatures, 105 give resources (5 take some: one town of `Myth and Legend`, one of
  `Rebellion`), **199 carry a message only** (159 in `Myth and Legend`, 23 in `Overthrow Thy
  Neighbors`, 7 in `Twins`, 5 each in `Resource War` and its allied copy; survey `E09-towns.md`
  §1.3 said 189 in 3 maps: corrected); 85 have no message. **By kind:** 219 for humans only, 48
  for computers only, 102 for both, none for neither; every event names a player; 8 do not name
  their town's owner at the start and 11 name it but not its kind (they fire after a capture or
  never).
- **The days** (the file's + 1): 271 once (day 1 39 times, day 2 20, day 4 20, day 6 13, day 8 10,
  the rest spread to day 168), 98 repeating: every 7 days 45, every 14 days 33, every 5 days 2 and
  **every day 18** (daily gold, 1 000 to 4 000, from day 21, 28 or 29 in `Marshland Menace`,
  `Pestilence Lake` and its allied copy, `Shadow Valleys`: TD-12).
- **The buildings** by h3m bit: the Town Hall (0) 23, the level-6 dwelling (37) and its upgrade
  (38) 19 each, the Fort (3) 12, the Citadel (4) 10, the City Hall (1), the Capitol (2), the
  level-1 dwelling (22) and the level-1 horde slot (24) 6 each, the Marketplace (8), the level-2
  dwelling (25), the level-2 horde slot (27), the level-4 dwelling (31) and the level-5 dwelling
  (34) and upgrade (35) 4 each, the Blacksmith (7), the Mage Guild 1 (11) and the first special
  (18) 2 each. **None** builds the Shipyard or the Grail, sets a bit past 40 or builds a building
  its own town forbids. **In the fixed towns** (the reading's rules, the events in day order, no
  build of the owner's between): 57 buildings built, 30 skipped as standing, **27 built without a
  requirement** (the `Back For Revenge` pair's day-11 dwellings and day-140 Capitol, `Undead
  Unrest`'s day-14 level-6 dwelling): the missing requirement test decides those three maps.
  **Six Capitols** by event (the `Back For Revenge` pair, neutral towns, days 125 to 140): a player
  who takes such a town and holds a Capitol elsewhere gets a second one (T-8).
- **The creatures:** 56 events, naming the levels 55, 52, 48, 38, 39, 38 and 38 times; in the
  fixed towns 106 of those levels have no dwelling at the start nor from an earlier event (lost
  unless the owner has built it by then), 61 go to an upgraded dwelling.
- **Checked by play** (E09 T19c2, exit criterion 6; `h3-map`'s `original_data`, seed 0, the
  packs' closed requirement lists): the fixed towns' events in this census's order give its
  numbers again, **57 built, 30 skipped as standing, 27 without a requirement**, once its terms
  are read as T12b counted them: each of an event's buildings built one by one in id order, the
  one another of its own supersedes too (the game builds the superseding one alone, its
  `supersedes` bringing the other: 38 of the 57 told as builds, 19 folded into them), and the
  requirement test the executable's direct lists (the closed lists: 23 of the 38); the ever-built
  sets after each event are the same both ways (checked). **Played by `EndTurn` alone** (a week;
  the 29 maps with events on to the last first day of an owned town's event that fires, a
  repeating one to its second firing; 168 days at most): of the 369 events 299 fire, 23 are a
  neutral town's (none fires without a capture: the `Back For Revenge` pair's Capitols among
  them), 8 do not name their town's owner, 39 name it but not its kind (a colour a human may play
  counts as human, GAM-BAT-010); 496 firings (236 with resources, 38 with buildings, 59 with
  creatures, 190 a message alone); the fixed towns' build 18 (9 without a requirement: the `Back
  For Revenge` pair's Rampart day-11 dwellings and `Undead Unrest`'s day-14 dwelling), skip 18 as
  standing and fold 5; the random towns' build 16 and skip 4; no Shipyard is an event's. Every
  firing and build is the census's own walk of TWN-EVT-001 and -003 (by day, town, place and
  building; day 1's inside the build).

## 11. Disputes, quirk candidates and open points (T12b; both parts, for the checkpoint)

| Id | What | Reading | VCMI | Others | Increment |
|---|---|---|---|---|---|
| TD-9 | The Conflux's upgraded Altar of Fire (40) | needs the Altar of Fire (33) and the upgraded Altar of Air (38), §6 | the Altar of Fire (and the Altar of Air, which it needs) | the fandom wiki: the Altar of Fire | 2 |
| TD-11 | Who builds in a town, and who pays | no owner test on the hall's path, the current player pays (§1.1, §1.4): a visitor in an ally's town at its own cost | the owner only, the owner pays | silent | 2 |
| TD-12 | A town event repeating every day | from its first day on (§8.2; the map's events' test) | one day early, from the day before its first (`occursToday`) | the wiki: on the chosen date | 2 |

- **Quirk candidate T-8 (new):** an event's Capitol beside the owner's other one (§8.3): the
  one-Capitol rule is the hall's and the capture's, not the event's; VCMI's `force` skips it too.
  Six shipped events can give it (§10). Recommendation: not a quirk to fix: the event is the map's
  override, both sources agree; play it as read.
- **The town events' rules as read** (§8; for T13 to place): no requirement, forbidden, Capitol,
  Grail, cost or mark test; skip what stands, what the faction lacks, what the event's own
  buildings supersede and the Shipyard without its spot; creatures lost without the dwelling; the
  owner's turn only; in the map's order, after the map's events. The reading, VCMI and the wiki
  agree but for TD-12, the Shipyard (no shipped case) and the towns' order.
- **Open from T12a:** the computer's every other day on Easy and Normal (§3; the engine has no
  difficulty and no computer player yet; recommendation: one a town a day for every player, the
  handicap with E10's and E13's computer player); the day's mark across a capture (§3: kept, but
  cleared from a team without a human to one with a human; recommendation: as read); the Grail's
  building (§2.6, increment 5), the Lookout Tower's and the Skyship's reveals (§4, increment 4); the
  tutorial's exception (out of scope).
- **For an experiment batch TWN-2** (if the user wants the original observed): TD-9 (a Conflux
  town with the Altar of Fire and the Altar of Air: the hall's state of the upgraded Altar of Fire)
  and TD-11 (two human allies in hot seat: the hall's Build for the visitor in the ally's town, and
  whose treasury pays).

## 12. The checkpoint (T12b; `TWN-Q8`, the user, 2026-11-07)

Asked interactively (`AskUserQuestion`, one round of four, a recommendation each); each answer as
recommended, the epic's decisions 41 to 44:

- **TD-9** (decision 41): the executable's list (the upgraded Altar of Fire needs the Altar of Fire
  and the upgraded Altar of Air); a row of an experiment batch TWN-2 observes the original's hall,
  and its verdict overrides.
- **TD-11** (decision 42): the town's owner builds and pays (VCMI's rule); a TWN-2 row (two allied
  humans in hot seat, the visitor at the ally's hall) observes the original; revisited if the
  original lets the visitor build.
- **The day's build** (decision 43): one a town a day for every player; the computer's every
  other day on Easy and Normal comes with the computer player (E10, E13); the day's mark is kept
  across a capture and cleared when the town passes from a team without a human to a team with
  one, as read (§3).
- **The town events** (decision 44): as the original plays them (§8): T-8 is not a quirk to fix
  (an event's Capitol beside the owner's other one stands); TD-12 the original's day (an every-day
  event fires from its first day).

## 13. Batch TWN-2 scored (T14c, 2026-11-07)

Both runs of T14b (the capture harness under Wine, the Complete 4.0 executable in SoD on Easy, hot
seat, Tan a computer enemy; `experiments/results/E09-batch-2a.md`, the main record, and
`-run2.md`): every read the same in both but the treasuries' gold (the random starting bonus). The
predictions are the spec's (`experiments/specs/E09-batch-2a.toml`). **Row 1 as the reading in
every point; row 2 refused, against the reading's prediction, by an owner test T12a had not read;
one bit not predicted in every town whose screen opened: a town-screen piece.**

- **TWN-2-1 (TD-9, §6).** Before: C1's standing and every-built sets {0, 7, 10, 23, 30, 31, 33};
  C2's standing set {0, 7, 10, 23, 30, 33, 38} (38 supersedes 31), its every-built set the same
  with 31; both marks 0. **C1:** the level-4 slot the upgraded Altar of Fire, its look not now
  (the bar's frame 2, `town_build_possible` true); its dialog's Build greyed (state 0xE), the text
  GENRLTXT:53 naming the upgraded Altar of Air (38) alone; Build pressed: nothing (the dialog
  stays; the sets but for the piece below, the mark 0 and Red's resources unchanged). **C2:** the
  slot buildable, Build enabled (state 0x6, the text GENRLTXT:220 naming nothing); pressed: the
  town screen at once, the standing set 40 in and 33 out, the every-built set 40 added (33 stays),
  the mark 1, Red's 5 mercury, 5 ore and 2 000 gold less. **TD-9: the reading (the executable's
  list: 40 needs 33 and 38), confirmed**; VCMI's (33 and the Altar of Air 31, which C1 holds) and
  the fandom wiki's (33 alone) refuted: under either C1 builds. Decision 41 stands; no gap-fill
  override (§25 of the design note).
- **TWN-2-2 (TD-11, §1.1, §1.4).** Before: B1's sets {7, 10}, the mark 0. Red's hero's step into
  its ally's town opened the town screen (§1.1 confirmed), the hall opened, the Tavern's slot
  showed buildable (the hall's sets computed with Red's resources); its dialog's Build **greyed**
  (state 0x26), the text GENRLTXT:220 (no requirement missing); pressed: nothing; both
  treasuries, B1's sets but for the piece and its mark unchanged. **Refused:** the reading's
  prediction (built, paid by Red) and the third outcome (built, paid by Blue) refuted.
  **Where:** not the hall's test (here it passes `build_dialog_run` 0 for its disabled flag: the
  building not built, in the buildable-now set and paid, 0x462066) but the dialog's constructor
  `build_dialog_create` 0x5D55C0, which T12a did not read: after its buttons it greys Build (the
  button's method at vtable +0x24 with 0) when the town's owner (+0x1) is not the current player
  (0x5D5AE2) or, in a network game, when the current player's record lacks a human flag
  (`player_is_human_both`). The hall's refusal (`build_dialog_run` 0x5D6754) shows as 0x8 in the
  button's state (C1's 0xE), this one as 0x20 (0x26). So an ally's visitor sees the hall and a
  slot buildable, and only Build is refused; §1.4's "no owner test on the hall's path" holds for
  the hall alone. **TD-11: VCMI's rule is the original's**: only the town's owner builds, so the
  current player who pays (§1.4) is always the owner. Decision 42 stands, now as the original's
  rule; no question.
- **The town-screen pieces (not predicted).** In each town whose screen opened, a bit that no
  building of its faction has: 22 in C1 and C2 (Conflux), 23 in B1 (Castle), set as the screen
  opened and kept after it closed; C2's build copied it into the every-built set. **Where:**
  `town_screen_pieces` 0x5C33F0 (read whole; named `town_build_cheat` before), called by the town
  screen's init (`town_screen_init` 0x5C63C0, at 0x5C653E) and where the screen rebuilds its
  buildings (`town_screen_proc` 0x5D4310, 0x5D498D, 0x5D4A3C; the Castle Gate's 0x5D863A), each
  beside the ship's update: with the build-all cheat's flag 0x6AAA5C set (a typed cheat code
  toggles it, 0x40293D) every building of the faction's mask not yet built, through
  `town_build_effects` without the mark; else, by faction (the table 0x5C34A8), `town_build` of
  the faction's piece: **Castle 23, Stronghold 27, Fortress 23, Conflux 22**, nothing in the five
  others. `town_build` writes only the standing set (towns survey `E09-towns.md` §16.2); the
  every-built set takes the piece at the town's next build, when `town_build_effects` (§4)
  recomputes it from the standing set (C2), or when the ship leaves its spot (`town_ship_update`).
  None of the four is in its faction's list (the lists 0x63EC80 dumped again: Castle and Fortress
  list 21 and 22, the Conflux 21, the Stronghold 21 to 23), so none is in the town's mask +0x160:
  never offered by the hall, never a requirement, never counted by the keep (towns survey §16.4),
  in no supersede list. **Readers:** bit 27 none; bit 23 the two of towns survey §16.3, which
  test the faction first (2, 3, 5, 6); bit 22 in sixteen functions, all but two testing the
  faction first (the Castle's Brotherhood, the Rampart's Treasury, the Tower's Library, the
  Inferno's Castle Gate, the Dungeon's Portal of Summoning, the Fortress' Blood Obelisk and its
  hall chain, case 7 of `hall_dialog_fill`'s faction switch); of the two, one is the town list the
  Castle Gate fills with Inferno towns (`town_castle_gate` 0x5D852C; Town Portal's use of it not
  read), the other in the computer's pathfinder (0x56A947: a town's every-built bit 22, else
  `town_can_buy` of 22; its callers not read), the computer's own planning, no rule. They are display pieces, as the hall
  companions are (towns survey §16.3; there VCMI draws Castle's and Fortress's screen extras
  without a building, §15). **For the engine: nothing** (no quirk candidate, no `T-10`): the part
  leaves them out as decision 27 left the companions, so the engine's sets hold only the faction's
  buildings; a comparison of the original's sets with the engine's drops them (the lane's
  `TRAPS.md`).
