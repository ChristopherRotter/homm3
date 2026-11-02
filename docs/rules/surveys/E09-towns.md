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

## 0. Findings in short (T1a1)

1. **Every shipped map has towns:** 1 468 on the 160 maps, 876 random towns (class 77, on 127
   maps) and 592 of a fixed faction (98); **no Conflux town is placed on any map** (98's subclass 8
   never occurs). 889 are owned at load, every one by a colour that plays; 579 are neutral. All
   twenty town templates share one footprint (§1.1).
2. **All 160 maps import once three things exist:** the hero generated at a main town (128 maps
   refused for it today), a hero standing in a town (3 maps) and **the random hero** (class 70: 21
   maps refused for it alone, 15 more behind a town refusal; 166 random heroes on 36 maps). The
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
- **Random heroes (70):** 166 on 36 maps (1 to 9 a map).

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
