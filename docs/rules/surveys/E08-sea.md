# E08 boats and sea survey (E08 T66a): boats, boarding, landing, sailing, the battle at sea, the sea's pickups

Date: 2026-10-28. Scope: E08 increment 8 (decisions 158, 159), input for T66b's ADR amendment
drafts, the design note and the cut. Cut in two (decision 160): **T66a1** the census, boats,
boarding, landing, sailing, the events on water and the writer (§0 to §6, this part); **T66a2**
the battle at sea, the four pickups (Sea Chest 82, Shipwreck Survivor 86, Flotsam 29, Ocean
Bottle 59), the commands, decisions, events and the observation, batch E8's plan, the rule ids, a
fresh agent's second reading and the checkpoint's `ADV-Q` (from §7). Input, not rules
(`README.md`): a fact becomes binding when a task writes it into the catalog with its sources and
tests.

**Sources** (retrieved 2026-10-28):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path: the boats' records (`create_boat` 0x4BB250, `hero_boat` 0x4CE900, the loader's
  case 8 in `load_object_body` 0x502E00), the tile's coast mark (`map_tiles_load` 0x4FE220,
  `tile_refresh_object` 0x505610), boarding and landing (`visit_object` 0x4A84F0 cases 3 and 8,
  `hero_boards_boat` 0x4A1010 and `hero_lands_from_boat` 0x49E670 whole, `hero_move_step`
  0x4805E0 at its eleven boat sites, `after_step_guard_check` 0x480090, `hero_move_loop`
  0x407B80 at its boat sites), the pathfinder's boat node (`pathfinder_expand` 0x4B2300, the human
  player's modes 0–2), the step's price at sea (`step_cost` 0x4B1740, read whole in T34a), the
  day's points at sea (`hero_max_land_movement` 0x4E4990, read whole in T34a), the hero sent back
  into the heroes' pool (0x4D9EC0, at its boat site). Not followed: the computer
  player's planning (the pathfinder's mode 4, its boat node from a walking node, `pathfinder` +0x14,
  0x4B2B01; 0x42FC50, 0x430F80), network play (0x482010, 0x49B800, 0x49B960, the move step's
  `network_game` branches), the message records the boat's steps build (0x49C560, 0x49C720,
  0x49C900, 0x49CB20: each fills a newly allocated record, for the network or the other players'
  screens; they write no hero, boat or tile field),
  saved games (`saved_game_load`'s boats), the adventure spells (Summon Boat's cast 0x41C8A0 and
  its choice 0x4E54A0), four more readers of the boats' records in a part of the code not read
  (two callers of `hero_boat`, two more of the boats' vector), the shipyard (E09, item 9), the drawing (E24's).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `server/CGameHandler.cpp`
  (`moveHero`), `lib/pathfinder/CPathfinder.cpp` (`getNeighbourTiles`, `getMovementCost`,
  `isLayerTransitionPossible`), `lib/pathfinder/PathfindingRules.cpp`,
  `lib/mapObjects/CGHeroInstance.cpp` (`movementPointsAfterEmbark`), `lib/mapObjects/MiscObjects.cpp`
  (`CGBoat`), `lib/gameState/CGameState.cpp` (`initHeroes`, `randomizeMapObjects`),
  `lib/gameState/GameStatePackVisitor.cpp`, `lib/mapping/TerrainTile.h`, `config/heroes/castle.json`,
  `config/skills.json`, `config/gameConfig.json`; links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Boat, Movement, Navigation, Admiral's Hat, Sea Captain's Hat. Re-check a quote before a rule
  cites it with `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §1;
  `movement.txt` (`extracted.tar`) for the sea's table.

## 0. Findings in short (T66a1)

- **Census** (§1): 118 boats on 41 maps (at most 15 on one), no hero placed on water, 9 events on
  water, 48 events on coastal land; Race for Ardintinny imports whole with 5 boats and 128 of the
  sea's pickups; Favorable Winds on 3 maps.
- **Boats** (§2): at most 64 records (the wiki's limit; VCMI none); a map's boat has no body, its
  type is the template's subclass and appearance only; any hero boards any empty boat; the boat
  remembers its owner and last captain (only Summon Boat reads them); it disappears with a hero
  sent back to the pool. The original gives a hero placed on water no boat (VCMI does: D-44).
- **Boarding** (§3.1): a step from land onto the boat's tile (priced by the land tile), no guard
  fight there, the walk ends, the visit boards: Fly and Water Walk cleared, the sea's maximum,
  the points 0, or with the Admiral's Hat (left after the step, dropped to 0 below 100) × sea ÷
  land (D-41: VCMI converts without the drop).
- **Landing** (§3.2): only onto a free coastal tile (class 3: the file's coast flag, no object,
  event or hole; D-42: VCMI lands onto events and enterable objects too); the boat stays on the
  water tile left, facing the hero's way; the walk ends, the visit lands (the land's maximum, 0 or
  the Hat's proportion), then the landing tile's guards attack.
- **Sailing** (§4): 100 a step, 141 diagonally, Favorable Winds 66 and 93 from a winds tile (boats
  only); a diagonal step at sea only between two water tiles; no boat onto another empty boat;
  the multi-day path takes the sea's maximum; the day's points at sea as T34a read them (1 500 by
  Navigation, specialists, the Necklace, the two Hats); D-43: the diagonal exception on winds.
  At sea objects are visited from the neighbouring tile, another boat's hero too; a hero on land
  visits a hero in a boat from the shore, never the reverse; events on water fire; the scouting
  radius does not change; an empty boat shows its place, type and facing, no owner.
- **The writer** (§5): an empty boat is a plain object of class 8; a hero "in a boat" is a hero
  that boards one in the game.

## 1. The census

`original-data` (the 160 shipped maps: 49 RoE, 55 AB, 56 SoD), read with `h3-map`'s reader by a
throwaway example (not committed). Five maps import whole today (Adventures of Jared Haret,
Faeries, Knee Deep in the Dead, Race for Ardintinny, the Tutorial; E08-rewards §1).

- **Water:** 131 maps have water (366 366 water tiles). The file's coast flag (0x40) is on every
  land tile with water among its eight neighbours and on no other tile (0 mismatches; as
  `docs/formats/h3m.md` says); 92 044 coastal land tiles. The tile flag 0x80 (VCMI's
  `hasFavorableWinds` bit) is set on no tile: VCMI sets it at run time from the winds' objects.
- **Boats (8):** 118 on 41 maps, every one on water, one visitable tile each; subclass 0: 20, 1:
  51, 2: 47. The most on one map: 15 (Myth and Legend), then 8 (Pandora's Box), 6 (Buried
  Treasure, Middletown), 5 (Thousand Islands): far below the 64 records (§2). Of the maps that
  import whole: Race for Ardintinny 5 boats, Faeries 1, the other three none.
- **Heroes on water:** none. No placed hero (34), prison (62) or random hero (70) has its tile on
  water: the shipped maps put every hero on land, so "a hero placed in a boat" (§2, §5) occurs in
  made maps only.
- **The coast** (where a boat may land, §3.2): of the 92 044 coastal land tiles 27 429 are free
  (class 3 at load), 50 948 are an object's blocked cell, 9 156 an object's entrance (most: random
  resources 76 2 278, treasure chests 101 1 482, resources 79 944, random monsters 72 to 75
  812, random artifacts 66 to 69 449, shipyards 87 220), 4 511 rock. **Events on coastal land:
  48 on 20 maps**, where the original lands no boat and VCMI does (D-42). Holes (124) on the coast
  were not counted.
- **Events on water: 9 on 5 maps** (Gelea's Champions and its allied copy, Myth and Legend,
  Pandora's Box, Race for Ardintinny): the queue inbox's count, now reachable (§4.3).
- **The sea's classes** (maps, objects): Flotsam 29 (90, 2 019), Sea Chest 82 (80, 826), Shipwreck
  Survivor 86 (72, 362), Ocean Bottle 59 (15, 29): as `E08-rewards.md` §1; Race for Ardintinny
  holds 87, 32, 7 and 2 of them and imports whole (T66a2's test map). **Favorable Winds** (225):
  154 objects on 3 maps (Carpe Diem and its allied copy, Just A Visit), none on a map that
  imports whole. Not in increment 8, for scale: Shipyards 87 (82 maps), Lighthouses 42 (54),
  Whirlpools 111 (62), Buoys 11 (56), Mermaids 55 (103), Sirens 92 (57), Derelict Ships 24 (58),
  Shipwrecks 85 (69).
- **The sea's artifacts and skill:** the Necklace of Ocean Guidance (71) placed 3 times on 2 maps,
  the Sea Captain's Hat (123) 3 times on 3 maps and worn or carried by 2 placed heroes (Loss of
  Innocence and its allied copy), the Admiral's Hat (136) once (Caught in the Middle);
  Navigation (skill 5) in a placed hero's custom skills 3 times (Loss of Innocence twice, Myth and
  Legend).
- **Land only the sea reaches** (an upper bound): on 95 maps some coastal land holds objects but
  no town and no placed hero by an 8-connected walk on its level over tiles no object blocks
  (5 463 entrances). Monoliths, subterranean gates, garrisons, quest guards and the other level
  were not followed, so this counts islands and land cut off by mountains alike.

## 2. Boats: their records and the import

**The records** (`structs.toml` "boat"; game +0x4E3BC, 0x28 bytes each): the tile (x, y, level at
+0, +2, +4), in use (+0x18), the record's index (+0x19), the type 0 to 2 (+0x1A), the facing
(+0x1B), the owner (+0x1C), the captain (+0x20, a hero's id) and aboard (+0x24). `create_boat`
(0x4BB250; the tile, an owner, a quiet flag, the type) takes the first record not in use, else
appends one, and fails (−1) when 64 records are in use; a new boat faces 2, has no owner (−1),
no captain and nobody aboard, and is put on its tile as an object of class 8 whose word is the
record's index (`hero_show` 0x4D75D0). The wiki: "There is a limit of 64 boats on the map." VCMI
1.7.5 keeps boats as map objects without a cap. The census's largest is §1's.

**The import.** A boat (class 8, templates `AVXboat0` to `AVXboat2`, water only) has no body:
`load_object_body`'s case 8 reads nothing from the file and calls `create_boat` with the
object's entrance tile, no owner and the template's subclass as the type (0x502F28 to 0x502F59;
the template record's +0x3C). The type is appearance only: the boat's pictures, its waves and the
flag of a hero aboard (`view_boat_piece` 0x410760, `view_hero_piece` 0x40FE30); no rule read
here tests it (the battle's backgrounds and obstacles test only whether a hero is in a boat,
`choose_battle_background` 0x4646D0, `battle_obstacles` 0x466290). VCMI: a boat's type carries
its layer (all SAIL in SoD's config) and its pictures.

**Owner and captain.** Boarding sets the owner to the hero's player and the captain to the
hero, aboard 1 (`hero_boards_boat` 0x4A10AD to 0x4A10BD); landing clears aboard and keeps both
(`hero_move_step` 0x480A79). Nothing on the human player's path reads them but Summon Boat's
choice (0x4E54A0: the caster's own last boat first, else the nearest empty boat of its player or
of none; the adventure spells, not placed) and the drawing of a hero aboard. **Any hero boards
any empty boat:** `visit_object`'s case 8 tests nothing before `hero_boards_boat`. An empty
boat shows no flag (`view_boat_piece`), so its owner is not seen.

**A boat leaves with its hero.** A hero sent back into the heroes' pool (0x4D9EC0, a hero that
lost a battle on the map: E08-battles §7.4) frees its boat's record (in use 0, 0x4DA009) and
clears its in-boat flag: the boat disappears. The wiki: "the boat disappears"; VCMI
(`GameStatePackVisitor`): "If hero on Boat is removed, the Boat disappears". **Consensus.**

**A hero placed on water.** Nothing on the paths read gives a placed hero a boat: `create_boat`'s
callers are the loader's case 8, Summon Boat (0x41C8A0), network play and the computer player's
planning; only `hero_boards_boat` sets the in-boat flag on a rule's path (two network handlers
copy it); `placed_hero_setup` (0x4D8B30) has no boat test, and its day's points are the land's
for a hero without the flag. VCMI (`CGameState::initHeroes`, "generate boats for all heroes on
water") gives every hero on water a boat of its class's faction's type, hidden under it. §1 counts
the shipped maps' cases; §5 what the writer does with it.

## 3. Boarding and landing

### 3.1 Boarding

A hero not in a boat steps onto an empty boat's tile (the boat's one visitable tile, which its
passability also blocks). The pathfinder takes an entrance of a boat (8), a hero (34) or class 85
on water as a normal step from land (survey `E08-hero-movement.md` §3.1 point 4); VCMI the same
three (`PathfindingRules.cpp`: from LAND to SAIL only a hero, a boat or a coast-visitable object,
the Shipwreck). The step's price is the source tile's walking price (`step_cost` prices the step
by its source, T34a §2.1).

1. **The step** (`hero_move_step` 0x480C19, its class switch): for a hero not in a boat message
   records (Sources), the hero lifted, `hero_step_allowed` (§4.1), then the walk; the price is
   charged after it and the points drop to 0 when fewer than the cheapest step from the boat's
   tile remain (0x481182; `cheapest_step_from` 0x4B1880: 100 on water, T34a). No guard attacks on a boat's tile (`after_step_guard_check` 0x480108: class 8
   skipped). The step returns the boat, which **ends the walk** (`hero_move_loop` 0x407EF9: a
   step that returns an object stops it; the rest of the path is rebuilt).
2. **The visit** (`visit_object` case 8, 0x4A8715) calls `hero_boards_boat` (read whole): the
   boat taken off its tile (0x4A1031), the Fly and Water Walk levels cleared (+0x112, +0x116 to
   −1), the in-boat flag (+0x105 bit 18); then the day's maximum becomes the sea's
   (`hero_max_land_movement` with 1) and the points left **left × the sea's maximum ÷ the land's
   maximum** (truncated) with the Admiral's Hat (136; `hero_wears_artifact`), else **0**; the
   boat's owner, captain and aboard (§2). A cheat's flag (+0x105 bit 24) keeps the points.

So with the Hat the points carried to sea are (left − the step's price), dropped to 0 below 100,
then × sea ÷ land. **VCMI** (`CGHeroInstance::movementPointsAfterEmbark`): 0 without free
boarding; with it (the Hat) (left − price) × sea ÷ land, with no drop between. **Wiki**
(Boat): boarding "takes the remainder of a hero's movement allowance for the current day";
(Admiral's Hat) "No boarding or unboarding penalty for boats", "Converts movement points between
land and water movement". **Consensus** on 0 without the Hat and the proportion with it; the drop
before the proportion is the reading's alone: **D-41** (§6).

### 3.2 Landing

**Where.** A hero in a boat steps onto land only onto a tile whose class is 3: the loader gives
class 3 to every land tile with the map file's coast flag (`map_tiles_load` 0x4FE3D6; the same
flag sets the map item's +0xD 0x02, the shore battlefield's coast), and `tile_refresh_object`
gives it back when such a tile's object goes (0x5056A5). Any object the tile records replaces
it: an object's entrance, an event (26, recorded though its tile is not visitable, E08 T58), a
hole (124, the fourth walk) and an object's blocked cell. So **a boat lands only on a free
coastal tile**: not on an object's entrance, an event or a hole (the pathfinder 0x4B2867 to
0x4B2886: from a boat node a land tile of another class is no step). The wiki (Boat): "Landing is
allowed on any clear land square, but not on a square with a hole." VCMI (`moveHero`):
disembarking onto any land tile that is not blocked (an event's tile or an enterable object's
too), with a visit at the destination: **D-42** (§6).

1. **The step** (`hero_move_step` 0x48095A, in a boat onto class 3): the hero's boat is put on the
   hero's water tile (`hero_boat` with aboard, its tile, `hero_show` with class 8), facing the
   hero's facing, aboard 0 (the captain kept); the walk, the price (the water source's: 100, 141
   diagonally, Favorable Winds below), the drop to 0 (the cheapest step from the coast tile: its
   road's price, else 100); no guard test (`after_step_guard_check` 0x4800E3: a hero in a boat
   onto class 3 skips it). The step returns the coast tile, which **ends the walk**.
2. **The visit** (`visit_object` case 3, 0x4A861A) calls `hero_lands_from_boat` (read whole): the
   in-boat flag cleared; with the Admiral's Hat the land's maximum and **left × the land's maximum
   ÷ the sea's maximum** (truncated), else **0** (the cheat's flag keeps them); the facing kept,
   a sound, then `guard_attack` (0x481900) for the landing tile: its guards attack now.

VCMI: the conversion (left − price) × land ÷ sea with the Hat, else 0, `CHECK_FOR_GUARDS` and the
destination's visit. Wiki: as boarding. **Consensus** but D-41's drop, which applies here too.

### 3.3 Guards and the shore

- **Boarding** never fights: the boat's tile is skipped by the guard test, and the next step
  away from it is tested as any step (VCMI: `EMBARK` with `IGNORE_GUARDS`; its pathfinder lets
  a boat leave a guarded tile: "Hero embarked a boat standing on a guarded tile").
- **Landing** on a guarded tile: the guards attack after the landing (`hero_lands_from_boat`'s
  `guard_attack`). VCMI: `DISEMBARK` with `CHECK_FOR_GUARDS`. **Consensus.**
- **At sea** a step onto a guarded water tile is attacked at once (`after_step_guard_check`: "a
  hero at sea at once", T20b3a; E08-battles §2.2).
- **A blocked or busy shore:** an object's blocked cell, entrance, an event or a hole on the
  coastal tile: no landing there (§3.2); the census counts what the shipped maps put on their
  coasts (§1).

## 4. Sailing

### 4.1 The boat node (`pathfinder_expand`, the human player's modes 0–2)

The search's node has a boat bit (flags bit 2, from the hero's in-boat flag at the start); it is
never cleared in modes 0–2 (only the computer's mode 4 clears it, 0x4B28B6).

1. **The price**: `step_cost` with no road, no native terrain, no flying or water walking, the
   hero's Pathfinding and Nomads (0x4B24B7): water's table price, 100 at every Pathfinding level
   (`step_cost_table` 0x63E510), 141 diagonally; **Favorable Winds** (225) on the source tile
   two thirds, truncated: 66, 93 diagonally (0x4B17B3; in the table's path for a water source,
   which a water walker's and a flyer's price replace: boats only). Wiki (Movement): "Favorable
   Winds at 66% (66 straight / 93 diagonal)", "Favorable winds do not affect movement under the
   Fly spell"; VCMI (`getMovementCost`): ×2/3 for the SAIL layer only, from the current tile.
   **Consensus.** VCMI marks the winds' tiles as the object's width × height rectangle
   (`randomizeMapObjects`); the original tests the tile's special ground (`tile_special_ground`
   0x4FCE20), the ground's picture's rectangle as the other grounds' (`docs/formats/h3m.md`,
   special grounds).
2. **The next day** in a multi-day path starts with the sea's maximum (`pathfinder` +0x10)
   instead of the land's (+0xC) for a boat node (0x4B2536).
3. **Water to water:** never onto a boat's entrance (an empty boat: 0x4B26EB; VCMI: "Hero in
   boat can't visit empty boats"); **a diagonal step needs both side tiles to be water**
   (0x4B2703 to 0x4B27F5; VCMI `getNeighbourTiles`: "avoid diagonal movement over coast when
   sailing", the same test; the move's own check too, below); a blocked water tile (an object's
   cell) never (only flying passes it); another entrance on water is a normal step (a visit, an
   attack: §4.3).
4. **Water to land:** onto class 3 only (§3.2).
5. **The drop to 0** (T34a) at sea: the cheapest step from a water tile is 100 (`cheapest_step_from`
   with the tile as native: 100).
6. **The diagonal exception** (ADV-MOV-005; A-2, A-3) compares the points left with the source
   terrain's table price, water's 100, never the winds' 66 (T34a §2.1 point 6): on a winds tile
   with 66 to 92 points left the diagonal (93) is refused, where VCMI's exception compares with
   the straight price after the winds (66) and lets it take the rest: **D-43** (§6).

**A landing ends the search's branch.** The boat bit stays, but `pathfinder_run` expands a
popped boat node only on water (0x56BBC1): a boat node on a class-3 tile is a leaf, so no path
goes on from a landing. **The move's own check** (`hero_step_allowed` 0x481BE0, before a step that
is no visit) repeats the node's rules: in a boat, land only of class 3 (0x481DF3); from water a
diagonal step only between two water side tiles, for a hero in a boat (and one neither flying nor
water-walking), not for a water walker (0x481D33 to 0x481DE6); a walking hero onto water only a
boat's or a class-85 entrance (0x481D0D; a hero on water is visited before this check, §4.3).

### 4.2 The day's points at sea

Read whole in T34a (`E08-hero-movement.md` §1.1), not repeated: `sea_movement_by_navigation`
(0x698AEC, `movement.txt`'s "Sea Movement" column) 1 500, 2 250, 3 000, 3 750 by Navigation's
level, the army's speeds not counted; a Navigation specialist with Navigation + trunc(level ×
1 500 ÷ 20) (75 a level: Sylvia and Voy in SoD); the Sea Captain's Hat (123) +500 (the
Lighthouse's amount, `movement.txt` "Lighthouse 500"); the Necklace of Ocean Guidance (71)
+1 000; the Admiral's Hat (136) both through its parts (`hero_wears_artifact` tests the parts of
a worn combination: +1 500); Lighthouses (item 9) and the computer's bonus (E10) left out. The
turn start sets the day's points for the hero's boat state (`hero_max_movement` 0x4E4D90, T6b);
boarding and landing switch the maximum (§3). **Wiki** (Navigation): "+50/100/150 %", the base
1 500 "fixed and does not depend on the slowest creature", the specialists' "75 extra movement
points at sea per level"; (Sea Captain's Hat) "+500 movement points on water"; (Admiral's Hat)
"+1500 movement points on water". **VCMI**: `movementPointsSea` 1 500, Navigation
`PERCENT_TO_BASE`, `navigationSpecialty` `valPer20` 1 500 with Navigation as its limiter.
**Consensus.**

### 4.3 What a hero in a boat visits

`hero_move_step`'s class switch (0x480C12) and its default case:

- **An object visited from the neighbouring tile** (`object_settings` +0 set, the "cannot be
  entered" list 0x63AA58: the sea's four pickups 29, 59, 82, 86 are on it; T66a2): at sea always from the neighbouring water tile (0x480F3A: a hero in a boat goes
  straight to `visit_from_neighbour` 0x480240, without the flying test).
- **Another hero** on water: from a boat, visited from the neighbouring tile when that hero is in
  a boat too (0x480D89 to 0x480E2D): a meeting or a battle (the two boats' battlefield, T66a2). A
  boat never reaches a hero on land (the pathfinder, §4.1). From land a hero visits a hero in a
  boat from the shore (34 on water is a normal step from land; the case's land branch: from the
  neighbour after the flying test): the deck's battlefield (T66a2).
- **An empty boat:** never from a boat (§4.1).
- **Events on water** (the queue's inbox note of 2026-10-27, confirmed): the step tests every
  tile it enters (`event_fires_for` 0x4FCDC0 at 0x4811A3, no boat test; only the network flag
  skips it), so an event on water fires for a hero in a boat that steps on it; the census's
  count is §1's; the engine's event at a step (`AdventureState::firing_event`) then takes the
  boat's step too (the inbox note, archived). An event on a coastal land tile blocks a landing
  there instead (§3.2).
- The sea's other objects (Buoys 11 and Sirens 92 on the list, Mermaids 55 and Whirlpools 111
  entered): not placed (decision 158).

### 4.4 Scouting at sea and what another player sees

- **Scouting:** the radius (`hero_scouting_radius` 0x4E4070, T8a) has no boat term, and the
  step's reveal (0x48103F) is the same at sea. VCMI the same.
- **What another player sees** (drawing, E24's readings): an empty boat on an explored tile: its
  tile, type and facing (waves, the boat; no flag, `view_boat_piece`); its owner and captain
  are not drawn. A hero in a boat: the hero's owner's flag on the boat (`boat_flag_def_names` by
  type × 8 + colour), the boat's type. The observation's fields are T66a2's.

## 5. The writer (CLT-Q5)

- **An empty boat:** a plain object (`writer::Plain`; class 8 joins `PLAIN_OBJECTS`), template
  `AVXboat0` to `AVXboat2` by type, no body (§2), its visitable tile on water.
- **A hero standing in one:** the original makes no boat for a hero placed on water (§2), and no
  shipped map places one (§1). So the map file has no way to put a hero in a boat that the
  original reads as one: a hero "in a boat" for the client lane's two situations (the deck: a
  hero in a boat against one on land; two boats: both heroes in boats) is a hero beside an empty
  boat that boards it in the game (a click: the boat's tile, §3.1), then sails or waits for the
  attack. The writer needs nothing more than the empty boat; a hero object on a water tile stays
  writable (the file allows it) for the experiment D-44 asks for (§6). Whether the import refuses
  a hero on water, gives it a boat as VCMI does, or leaves it on water without one is T66b's
  (D-44); the client lane's inbox gets the boat when it lands (CLT-Q5's answer, decision 158).

## 6. Disputes, quirk candidates and open points (T66a1)

The checkpoint's questions are T66a2's (§7 on); these are its inputs.

- **D-41. The drop to 0 before the Admiral's Hat's proportion.** The step onto a boat (and onto
  a coast) charges its price and drops the points to 0 when fewer than the cheapest step from the
  new tile remain (100 on water; a road's price or 100 on land) before the visit converts them
  (§3.1, §3.2): with the Hat and 1 to 99 points left after the step the original carries 0 to the
  sea (or the land), VCMI (left − price) × the new maximum ÷ the old. The wiki says only
  "converts". A conflict of the reading (the drop at 0x481182 runs in the step, the conversion in
  the visit after it) with VCMI, the wiki silent (`docs/04-rules-and-sources.md`, "Consensus before
  trust"), on the Hat alone (one shipped map places it, §1). **Proposed:** a batch E8 row decides
  (the Hat, a boat boarded with about 50 points left after the step: 0 or about 50 × sea ÷ land
  at sea); the checkpoint chooses the engine's side until then.
- **D-42. A landing onto an event's tile or an object's entrance.** The original lands only on a
  free coastal tile (class 3): 48 events and 9 156 entrances sit on the shipped maps' coasts
  (§1); VCMI disembarks onto any land tile that is not blocked (an event's, an enterable
  object's) and visits it. The wiki: "any clear land square, but not on a square with a hole".
  **Proposed:** the reading with the wiki (consensus of two); a batch E8 row checks an event on
  a coast (the step refused, the event not fired) and a hole.
- **D-43. The diagonal exception on Favorable Winds.** The exception compares the points left
  with water's table price (100), not the winds' 66 (§4.1 point 6): with 66 to 92 points left on a
  winds tile the original refuses the diagonal, VCMI lets it take the rest. Reading against
  VCMI; the wiki's exception section does not mention water: a conflict as D-41. Three shipped
  maps hold winds. **Proposed:** a batch E8 row on a winds tile decides; until then the reading's
  side (the comparison A-2 and A-3 already make on land, `ADV-QUIRK-2`, `-3`) if the checkpoint
  agrees.
- **D-44. A hero placed on water.** The original gives it no boat and the land's points (§2; no
  rule path sets the flag at a new game); VCMI gives it a boat of its faction's type. No shipped
  map has one (§1). **Options for T66b:** the import refuses such a hero (an `Unsupported` or an
  error: what the original does is untested), follows VCMI, or keeps it on water without a boat
  (what the reading says the original does, then its moves: from water a walking hero steps onto
  land or a boat, `hero_step_allowed`). A conflict of the reading with VCMI, the wiki silent.
  **Proposed:** refuse it, with a batch E8 row (a hero written on water: does the game show it in
  a boat, and what can it do) deciding later.
- **The 64 boats.** The reading and the wiki agree on the cap; VCMI has none. The shipped maps
  hold at most 15 (§1), and boats are made later only by shipyards and Summon Boat (not in
  increment 8): the cap can wait for them, or be a validated bound of the import now (T66b).
- **Not read** (for T66a2's second reading where it decides a rule): the computer player's
  boats (the pathfinder's mode 4, `pathfinder` +0x14 and its walking node's boat node 0x4B2B01;
  0x430F80), the network's and saved games' copies, the drawing, `hero_show` 0x4D75D0
  (inferred from its pair `hero_hide`), the message records (Sources).
