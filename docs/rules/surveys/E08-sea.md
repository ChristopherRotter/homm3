# E08 boats and sea survey (E08 T66a): boats, boarding, landing, sailing, the battle at sea, the sea's pickups

Date: 2026-10-28. Scope: E08 increment 8 (decisions 158, 159), input for T66b's ADR amendment
drafts, the design note and the cut. Cut in two (decision 160): **T66a1** the census, boats,
boarding, landing, sailing, the events on water and the writer (§0 to §6); **T66a2**
the battle at sea, the four pickups (Sea Chest 82, Shipwreck Survivor 86, Flotsam 29, Ocean
Bottle 59), the commands, decisions, events and the observation, batch E8's plan, the rule ids, a
fresh agent's second reading and the checkpoint's `ADV-Q32` (§7 to §14; whole, decision 161).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

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
  **T66a2** read whole the battle's three boat tests (`battle_setup_battlefield` 0x4643B0,
  `choose_battle_background` 0x4646D0, `battle_obstacles` 0x466290 from 0x4666D7), the four
  pickups' visits (`visit_flotsam` 0x4A2230, `visit_object` 0x4A9AFA, `visit_sea_chest` 0x4A5030,
  `visit_shipwreck_survivor` 0x4A52A0) and draws (`setup_map_objects` 0x4C1466, 0x4C1B94,
  0x4C1BF2), the bottles' and signs' loader (`load_message_body` 0x501270), the move's default
  case for a neighbour-visited class (`hero_move_step` 0x480F27) and the check before it
  (`hero_step_or_meet_allowed` 0x481AD0), and listed every use of the in-boat bit (0x40000).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `server/CGameHandler.cpp`
  (`moveHero`), `lib/pathfinder/CPathfinder.cpp` (`getNeighbourTiles`, `getMovementCost`,
  `isLayerTransitionPossible`), `lib/pathfinder/PathfindingRules.cpp`,
  `lib/mapObjects/CGHeroInstance.cpp` (`movementPointsAfterEmbark`), `lib/mapObjects/MiscObjects.cpp`
  (`CGBoat`), `lib/gameState/CGameState.cpp` (`initHeroes`, `randomizeMapObjects`),
  `lib/gameState/GameStatePackVisitor.cpp`, `lib/mapping/TerrainTile.h`, `config/heroes/castle.json`,
  `config/skills.json`, `config/gameConfig.json`; T66a2: `server/battles/BattleProcessor.cpp`
  (`setupBattle`), `CGameState::battleGetBattlefieldType`, `config/terrains.json`,
  `config/battlefields.json`, `config/objects/rewardablePickable.json`,
  `lib/mapping/MapFormatH3M.cpp`, `lib/mapObjects/MiscObjects.cpp` (`CGSignBottle`),
  `lib/pathfinder/PathfindingRules.cpp` (`MovementToDestinationRule`, `LayerTransitionRule`); links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Boat, Movement, Navigation, Admiral's Hat, Sea Captain's Hat; T66a2: Flotsam, Sea Chest,
  Shipwreck Survivor, Ocean bottle (reached through the search tool's links). Re-check a quote
  before a rule cites it with `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §1;
  `movement.txt` (`extracted.tar`) for the sea's table.

## 0. Findings in short (T66a1, T66a2)

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

**T66a2** (§7 to §14):

- **The battle at sea** (§7): a hero in a boat on either side makes the battle the deck (water,
  the ship flag), both in boats the two boats (their 32 fixed hexes); nothing else in a battle
  tests a boat. A boat's hero is fought from the shore or from another boat, by a monster on
  water (55 on 28 maps) and by an event's guards on water (3); a landing's guards fight it on the
  shore, out of its boat. A hero that leaves the map takes its boat. VCMI (by the water tile) and
  the wiki agree.
- **The four pickups** (§8), visited from the neighbouring tile and gone after the visit, no
  question: Flotsam 25 % each of nothing, 5 wood, 5 wood and 200 gold, 10 wood and 500 gold; the
  Sea Chest 20 % nothing, 70 % 1 500 gold, 10 % 1 000 gold and a treasure artifact (1 500 gold and
  no artifact with a full backpack); the Shipwreck Survivor a treasure, minor, major or relic
  artifact at 55, 20, 20 and 5 % (lost with a full backpack); the Ocean Bottle its text or the
  default by its column, only for a human. VCMI and the wiki agree but for VCMI's unlimited
  backpack. A flyer on the coast takes one beside it with a click (the reading): **D-45**.
- **The interfaces** (§9): no new command or decision; `Reach` gains a boat layer; boats and the
  in-boat flag in the state, the observation and the env's encoding.
- **Batch E8** (§11): 16 rows in two runs. **Rule ids** (§12): the family `ADV-SEA`, ADV-PIK-014 to
  -018, HER-ADV-009, amendments. **The second reading** (§13). **The checkpoint** (§14, ADV-Q32).

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
- **T66a2's counts.** Objects whose entrance lies on water, by class (objects, maps): the four
  pickups as above; monsters (54) 55 on 28 (4 beside land), Derelict Ships (24) 170, Shipwrecks
  (85, visited from land) 401, water Cartographers (13) 57 on 46, artifacts and random artifacts
  (5, 66, 68) 4, resources and random resources (79, 76) 7, a treasure chest (101) and a sign (91)
  1 each, creature generators (17, 217) 11 and an Eye of the Magi (27) 3, besides the Buoys,
  Whirlpools, Mermaids and Sirens. So the kinds already built (monsters, the Cartographer, the
  pickups of increments 2 and 7) become reachable by boat. Of the 9 events on water 3 have guards.
  **Beside land** (any land terrain among the eight neighbours; an upper bound, the free coast not
  tested): 272 Flotsams on 36 maps, 113 Sea Chests on 28, 8 Survivors on 4, 2 Ocean Bottles on 2
  (395 of 3 236; D-45). **The bottles' texts:** 26 of the 29 have one, 3 are empty (Brave New World
  and its allied copy, Race for Ardintinny). Race for Ardintinny (imports whole) holds 5 boats, 5
  water Cartographers, 87 Flotsams, 32 Sea Chests, 7 Survivors, 2 Ocean Bottles and 1 event on water;
  Faeries 1 boat, 1 Flotsam, 2 Sea Chests, 1 Ocean Bottle; Knee Deep in the Dead 1 Flotsam and 1
  Sea Chest (no boat); Adventures of Jared Haret 3 Flotsams, 2 Sea Chests and 2 water monsters (no
  boat).

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
   −1), the in-boat flag (+0x105 bit 18); then, with the Admiral's Hat (136;
   `hero_wears_artifact`), the day's maximum becomes the sea's (`hero_max_land_movement` with 1)
   and the points left **left × the sea's maximum ÷ the old maximum** (a signed multiply first,
   then truncated); **without the Hat the points become 0 and the day's maximum stays the land's**
   until the next turn start (0x4A1098; the second reading, §13: A-28); the boat's owner, captain
   and aboard (§2). A cheat's flag (+0x105 bit 24) keeps the points.

So with the Hat the points carried to sea are (left − the step's price), dropped to 0 below 100
(66 on a Favorable Winds tile, §4.1 point 5), then × sea ÷ land. **VCMI**
(`CGHeroInstance::movementPointsAfterEmbark`): 0 without free boarding; with it (the Hat)
(left − price) × sea ÷ land, with no drop between. **Wiki**
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
   in-boat flag cleared; with the Admiral's Hat the day's maximum becomes the land's and the points
   **left × the land's maximum ÷ the old maximum** (truncated), else **0** with the maximum kept
   (A-28; the cheat's flag keeps them); the facing kept, a sound, then `guard_attack` (0x481900),
   which takes the current player's selected hero's tile: the landing tile's guards attack now.

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
5. **The drop to 0** (T34a) at sea: the cheapest step from the new water tile (`cheapest_step_from`
   0x4B1880: `step_cost` with the tile as both ends, the boat's flying and water walking −1) is
   100, **66 on a Favorable Winds tile** (the winds' two thirds, a boat's step: the second
   reading's correction, §13).
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

- **D-41. The drop to 0 before the Admiral's Hat's proportion.** The step onto a boat (and onto a
  coast) charges its price and drops the points to 0 when fewer than the cheapest step from the
  new tile remain (100 on water, 66 on Favorable Winds; a road's price or 100 on land) before the
  visit converts them (§3.1, §3.2): with the Hat and 1 to 99 points left after the step the
  original carries 0 to the sea (or the land), VCMI (left − price) × the new maximum ÷ the old.
  The wiki says only "converts". A conflict of the reading (the drop at 0x481182 runs in the step,
  the conversion in the visit after it) with VCMI, the wiki silent
  (`docs/04-rules-and-sources.md`, "Consensus before trust"), on the Hat alone (one shipped map
  places it, §1). **Proposed:** a batch E8 row decides (the Hat, a boat boarded with about 50
  points left after the step: 0 or about 50 × sea ÷ land at sea); the checkpoint chooses the
  engine's side until then.
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
  increment 8): the cap can wait for them, or be a validated bound of the import now (T66b). A
  map's 65th boat is dropped at load without a word (its object's word −1; the second reading,
  §13). The checkpoint asks (§14).
- **Not read** (for T66a2's second reading where it decides a rule): the computer player's
  boats (the pathfinder's mode 4, `pathfinder` +0x14 and its walking node's boat node 0x4B2B01;
  0x430F80), the network's and saved games' copies, the drawing, `hero_show` 0x4D75D0
  (inferred from its pair `hero_hide`), the message records (Sources).

## 7. The battle at sea (T66a2)

### 7.1 The battlefield

`original-data` (`battle_setup_battlefield` 0x4643B0, `choose_battle_background` 0x4646D0,
`battle_obstacles` 0x466290), read whole for their boat tests. At a battle's start the battlefield
is the first case of: a siege (the town's native terrain); **a hero in a boat on either side** (the
attacker's hero `battle_manager` +0x53CC or the defender's +0x53D0 with +0x105 bit 18), or the
battle's tile an entrance of a Shipwreck (85) or a Derelict Ship (24): **water** (terrain 8) and
the **ship flag** (+0x53C6); underground; a mine's; the coast; the tile's own (the order of
GAM-BAT-001, `E04c-battlefields.md` §4). The special ground's switch follows on the battle's tile
as for any battle; no ground of the switch lies on water (Favorable Winds, 225, is not in it).
The picture: a siege's, a special field's (1 to 9), then **both heroes in boats `CmBkBoat.pcx`**,
then **the ship flag `CmBkDeck.pcx`**, then the shore's or the terrain's (0x464721 to 0x46474D).
The obstacles: with water as the terrain and **both heroes in boats** the 32 hexes of the table
0x63D368 are blocked and nothing else (0x4666DC to 0x46673D); with one boat the terrain's draw on
water (only the water obstacle, type 55: `E04c-battlefields.md` §1). So **the deck** is a battle
with one hero in a boat, **the two boats** one with both; the battle lane has both battlefields
(`deck`, `boats` in `packs/base/battle.toml`; BAT-TER-002, BAT-GEO-004), and only the derivation
is the adventure lane's: GAM-BAT-001 gains the boat case before the underground's.

Every use of the in-boat bit's constant 0x40000 in the executable was listed (37 sites, any
register form): in the battle only these three functions test a hero's in-boat flag; the other
battle sites (`stack_base_morale`, `creature_spell_factor` and the like) test a creature's flags.
**Nothing else in a battle depends on a boat**: no test in retreat, surrender, the war machines
or the tactics phase.

### 7.2 Who fights a hero at sea

- **Another player's hero from the shore** (attacker on land, defender in a boat, §4.3): the
  deck, at the defender's water tile.
- **Another player's hero from a boat** (both in boats, from the neighbouring water tile, §4.3):
  the two boats.
- **A hero in a boat never attacks a hero on land** (§4.1, §4.3): no step from water reaches one.
- **A monster on water** guards its water neighbours (`E08-battles.md` §2.1: water monsters
  guard water tiles) and attacks a boat at once on such a tile (§3.3); a boat may also visit it
  from the neighbouring water tile: the deck. The census (§1): 55 monsters (class 54; no random
  monster) stand on water on 28 maps, 4 of them beside land; Adventures of Jared Haret, which
  imports whole, holds 2 and no boat.
- **An event's guards on water** (§4.3: events on water fire for a hero in a boat): the deck.
  The census (§1): 3 of the 9 events on water have guards (Gelea's Champions and its allied copy,
  Myth and Legend).
- **The landing's guards** attack after the landing (§3.2, §3.3): the hero is on land then, out
  of its boat: the coast's battlefield (sand and the shore), not the deck.
- The sea's creature banks (the Derelict Ship 24 from a boat, the Shipwreck 85 from land; both
  fought on the deck by their own class test) are item 10's; Sirens, Mermaids, Buoys and
  Whirlpools fight nothing (not placed, decision 158).

### 7.3 After the battle

A hero in a boat that loses, retreats or surrenders, or both heroes after a draw, leaves the map
as any (GAM-BAT-003); its boat's record is freed with it and the boat disappears (§2: 0x4D9EC0 at
0x4DA009). A winner in a boat stays in it on its tile; a winner on land that attacked from the
shore stays on the shore (the visit from the neighbouring tile). The winner takes the loser's
artifacts as on land (GAM-BAT-004). A monster's or an event's survivors stay on their water tile.

### 7.4 VCMI and the wiki

`vcmi` (`server/battles/BattleProcessor.cpp` `setupBattle`, `lib/gameState/CGameState.cpp`
`battleGetBattlefieldType`, `config/terrains.json`, `config/battlefields.json`): a battle on a
water tile takes water's one battlefield `ship` (the deck, `CMBKDECK`), both heroes in boats
`ship_to_ship` with the same 32 `impassableHexes`; a water tile is never "coastal"
(`CMap::isCoastalTile`). The tile decides in VCMI, the heroes' flag in the original; they differ
only where a battle on water has no hero in a boat or a hero in a boat fights on land. The second
never happens (the landing's battle comes after the landing); the first only on D-45's path (§8.1):
a flyer on the coast that attacks a monster on the water beside it with a click (4 such monsters,
§1) fights in the original on the water tile's coast mark (+0xD bit 0x02, a shore
picture on water: sand and the shore) or, without it, on water's own terrain, in VCMI on the
deck. "If hero on Boat is removed, the Boat disappears" (`GameStatePackVisitor`). `wiki`
(Combat; `E04c-battlefields.md` §1): the "Boat" battlefield when "both heroes are in boats", the
"Boat (Deck)" one for "a hero in a boat against one on land", monsters or locations on a boat;
(Boat) "the boat disappears" with its defeated hero. **Consensus** on every point.

## 8. The sea's four pickups (T66a2)

All four are visited from the neighbouring tile (the "cannot be entered" list, §4.3) and leave
the map at their visit; none asks a question. Their visits (`visit_object` cases 29, 59, 82, 86)
and their draws at a new game (`setup_map_objects` 0x4C0CC0, its switch 0x4C0DAD) were read whole.
The human flag (`visit_object`'s fourth argument, false for the computer player) only hides the
messages, except at the Ocean Bottle (§8.5). After the gains every one runs the same tail: the
object removed (`remove_map_object` 0x4AABB0) and, when the view is shown, one of the sounds
`pickup01` to `pickup07` and the dissolve (as the treasure chest's, `E08-rewards.md` §2.2).

### 8.1 Who visits them

- **A hero in a boat** from the neighbouring water tile: `hero_move_step`'s default case
  (0x480F27: a class whose `object_settings` +0 is set goes to `visit_from_neighbour` 0x480240,
  at once for a hero in a boat, 0x480F3A).
- **A walking hero on land never:** before the class switch `hero_step_or_meet_allowed`
  (0x481AD0: for a destination that is not a hero, `hero_step_allowed` 0x481BE0 with flags 1, 0)
  refuses a step onto water for a hero neither in a boat nor water-walking unless the tile is a
  boat's or a class-85 entrance (0x481D0D); the step then fails as any refused step. The
  pathfinder gives no path either (`E08-hero-movement.md` §3.1 point 4: another entrance on water
  is passable only flying).
- **A flying hero standing on land, by a click:** the search gives a flying node a path onto
  another entrance on water as a stop-only node (`pathfinder_expand` 0x4B27FB, 0x4B2D12); the move
  takes a flyer to the class switch only on its path's last step (0x480B44 to 0x480BD0), where
  `hero_step_or_meet_allowed` passes it (`hero_step_allowed` with the fourth argument 0: a flyer
  not in a boat at once, 0x481C66) and the default case visits from its land tile once
  `hero_flight_tile_test` (0x4E5CE0) passes on the ground under it: a free land tile. So **Angel
  Wings (and Fly, later) let a hero on the coast take a pickup on the water beside it with a
  click**: **D-45** (§10). The arrow keys do not: the key's check passes the fourth argument 1
  (0x408ED5), which treats a flyer as a walker. The census (§1) counts 395 of the 3 236 sea
  pickups beside land.
- **A water-walking hero on land never:** its search refuses a water tile holding an entrance
  other than a boat's, a hero's or class 85's (0x4B29AC to 0x4B29E4: only a flying node passes),
  and its arrow keys walk (the fourth argument 1). Only a step issued another way would reach the
  default case's visit (0x480F46: a water walker at once); none is (the second reading, §13).
- A flight or a water walk over water does not visit them on its way, and does not end beside
  them on water: a visit is made from a stop (`E08-hero-movement.md` §3.1 point 6; ADV-MOV-009).

### 8.2 Flotsam (29)

- **The draw** (0x4C1466): the word = `random_between(0, 3)`.
- **The visit** (0x4A2230; the word's switch 0x4A2470): 0 nothing; 1 five wood; 2 five wood and
  200 gold; 3 ten wood and 500 gold (`give_resource` 0x4E3600 to the hero's owner, wood first);
  a word above 3 gives nothing (none is made). The message (`advevent.txt` lines 51 to 54) shows
  the gains' pictures for a human. Then the tail.

### 8.3 Sea Chest (82)

- **The draw** (0x4C1B94): r = `random_between(0, 99)`; below 20 the kind 0 (nothing), below 90
  the kind 1 (1 500 gold), else the kind 2 with `random_artifact_of_level(2)` (a treasure
  artifact, the pool of `E08-rewards.md` §3.1) in bits 3 to 12 (the kind in bits 0 to 2).
- **The visit** (0x4A5030): the kind read as a signed 3-bit field; **the kind 2 with 64 or more
  artifacts in the backpack** (`hero_backpack_count` 0x4D90C0 with 1, before anything is
  placed) **becomes the kind 1**. Kind 0: a message; kind 1: 1 500 gold; kind 2: the artifact
  (`hero_give_artifact` 0x4E3070 with 1, 1) and 1 000 gold, a message naming the artifact (line
  117), the computer's `ai_artifact_received` for a computer player. Then the tail.
- So a full backpack gets **1 500 gold and no artifact** (not the 1 000 that came with it), as
  the treasure chest's turn (A-25, `ADV-QUIRK-22`: the backpack's count first, whatever the
  free positions).

### 8.4 Shipwreck Survivor (86)

- **The draw** (0x4C1BF2): r = `random_between(0, 99)`; below 55 a treasure artifact, below 75 a
  minor, below 95 a major, else a relic (`random_artifact_of_level` 2, 4, 8, 0x10), the
  artifact's number as the word.
- **The visit** (0x4A52A0): **with 64 or more in the backpack nothing is given** (a message,
  line 126: no room) **and the survivor still leaves**: its artifact is lost; else the artifact
  (`hero_give_artifact` with 1, 1; a message naming it, line 125). Then the tail.

### 8.5 Ocean Bottle (59)

- **The loader** (`load_object_body`'s case for 59 and 91, the Sign, 0x503219 → 0x501270): the
  message read from the file (a length-prefixed text, then 4 bytes not kept) is appended as a
  20-byte record to the game's messages (+0x4E378: a "has text" byte, set when the text has at
  least one character, and the text), and the object's word is the record's index. Every
  bottle and sign gets one; no new-game draw (the switch's default).
- **The visit** (`visit_object` 0x4A9AFA, the same code as the Sign's 0x4A9FC0 but the
  removal): **only for a human** (the fourth argument; the computer's visit returns at once and
  the bottle stays: A-27); a message box with the record's text or, when the record has none,
  **the default text by the bottle's column, x mod 25** (the table 0x696C74, the sign's
  default texts), then the tail: the bottle leaves. Nothing is given.

### 8.6 VCMI and the wiki

- `vcmi` (`config/objects/rewardablePickable.json`, `lib/mapping/MapFormatH3M.cpp`,
  `lib/mapObjects/MiscObjects.cpp` `CGSignBottle`, `lib/pathfinder/PathfindingRules.cpp`):
  Flotsam the same four outcomes at 25 % each; the Sea Chest 20 % empty, 10 % 1 000 gold and a
  treasure artifact, 70 % 1 500 gold (the same chances in another draw order); the Survivor 55,
  20, 20 and 5 % of treasure, minor, major and relic; the bottle shows its text, or a text drawn
  at random among `randsign` when it has none, and leaves (blocking visit). Its backpack has no
  limit by default (`gameConfig.json` `backpackSize` −1), so a full backpack never refuses. A
  walking hero on land reaches from the shore only a hero, a boat or a coast-visitable object
  (the Shipwreck: `MovementToDestinationRule`), and a water walker's layer enters only free water:
  neither visits a pickup from the shore; its air layer was not followed.
- `wiki` (Flotsam, Sea Chest, Shipwreck Survivor, Ocean bottle): the same chances (Flotsam 4 × 25 %;
  the chest 20, 70, 10 %; the Survivor 55, 20, 20, 5 %); "If your backpack is full, you will
  receive 1500 gold and no artifact" (Sea Chest); the Survivor's "no room in your backpack"
  text; "if no message is written on the field, a random text will appear", "ocean bottles will
  disappear once read"; every page: picked up by "heroes traveling in boats", "can only be found
  from water squares".
- **Consensus** on the draws and the visits; the full backpack is the reading's and the wiki's
  against VCMI's unlimited backpack (as the treasure chest's, A-25); the shore is **D-45**; the
  bottle's default text (by column, or at random) is a client's matter: the engine holds no text.

## 9. Commands, decisions, events, the observation, replays and benchmarks (T66a2)

What increment 8 changes in the game's interfaces (ADR-0016, ADR-0017; the design is T66b's):

- **Commands.** None new: boarding, sailing and landing are steps of `MoveHero`'s path (a path
  ends at a boat's tile or a coastal landing, §3), and a pickup's visit is a step toward it (the
  visit from the neighbouring tile, ADV-OBJ-003). No pickup asks, so no answer is new. The
  validation grows with the step's rules: in a boat only water and class-3 land, the diagonal
  between two waters, no empty boat from a boat; a walker onto water only a boat (§3, §4.1).
- **The reachable-tiles query** (`Reach`, ADV-MOV-007): a boat layer: a boat node from the
  start in a boat, water steps at the sea's prices with Favorable Winds, the landing as a leaf
  (§4.1: no path goes on from a landing), boarding as a leaf from land (the walk ends, §3.1);
  the multi-day path's next day at the sea's maximum is not the engine's (today's reach only).
- **Decisions.** None new: a battle at sea opens GAM-BAT-004's battle decision with the deck or
  the two boats (§7); a pickup gives at once.
- **Events.** Boarding and landing in the move's outcome (the hero's boat, its tile; the day's
  maximum and points after the conversion: its owner's), a boat left on the water tile at a
  landing and taken off at a boarding (public where seen), a boat removed with its hero (public
  where seen), the pickups' gains (the treasury's change, `HeroChanged` for an artifact) and
  `ObjectRemoved` where seen, a bottle's message to its visitor's owner (the map's text or the
  default by column: what the original shows; the engine names the bottle, the client the text).
- **The observation.** Boats on explored tiles (place and type, the type for appearance only:
  §2), the in-boat flag of every seen hero (drawn on its boat with its owner's flag: public),
  an own hero's day's maximum at sea; the sea's pickups as typed objects (type and anchor, never
  their draws, as the treasure chests); a boat's owner and captain never (§4.4).
- **Replays.** Boats in the state change the adventure state hash and the game state hash; the
  map import's version moves (boats, the sea's pickups typed); the replay format does not need to
  (no new command or answer). Golden replays: random play on a fixture map with the sea (boats,
  the four pickups, events on water, Favorable Winds, a coast with guards), scripted replays of
  boarding and landing with and without the Admiral's Hat, a battle on the deck and one between
  two boats, a landing into guards.
- **Benchmarks.** `Reach` gains a node layer (a boat's search on the sea maps; the land's reach
  unchanged when the hero is on land); the state a fork clones gains the boats (at most a few
  dozen on a shipped map, §1); random play on the sea fixture; `reach-*` on a sea map with many
  boats (Myth and Legend's 15).
- **The env** (the AI lane's): the action encoding's `MoveHero` targets are unchanged; the
  observation encoding gains the boats and the in-boat flag (an encoding version bump, a
  `cross-lane` change); T66b tells the AI lane's inbox.
- **The computer player** (E10): its boat planning (the pathfinder's mode 4, 0x430F80) and the
  bottle's skipped visit (A-27) wait for E10.

## 10. Disputes, quirk candidates and open points (T66a2)

T66a1's D-41 to D-44 and the 64 boats are in §6; the checkpoint's questions are §14.

- **D-45. A pickup on the water taken from the shore.** The reading lets a flying hero standing
  on land (Angel Wings; Fly later) take a sea pickup on a neighbouring water tile with a click
  (§8.1: its search gives a stop-only node there, and the path's last step visits from the
  hero's land tile); the arrow keys do not, and a water walker never does (the second reading's
  correction, §13). The same path lets a flyer attack a monster on the water from the shore (§7.4).
  The wiki's four pages say only boats pick them up ("can only be picked up by heroes traveling in
  boats"); VCMI's walking and water walking layers do not reach them from land (its air layer not
  followed). A conflict of the reading with the wiki (and VCMI for the water walker), on an
  artifact the engine has (Angel Wings, increment 5) and on coastal pickups (395 of the 3 236
  shipped ones lie beside land, §1). **Proposed:** a batch E8 row (a flyer by a click and by an
  arrow key, a water walker and a walker on the coast beside a Flotsam) decides; until then the
  reading's side, built and confirmed or reverted by the row, as decision 129 did for batch E7.
- **A full backpack at sea** (§8.3, §8.4): the Sea Chest's artifact becomes 1 500 gold and the
  Survivor's is lost, the backpack's count first (A-25, `ADV-QUIRK-22`); the wiki says the same,
  VCMI's backpack has no limit. Consensus of two, as the treasure chest's (ADV-PIK-005): no new
  dispute, a batch row confirms (E8-6).
- **The bottle's default text** (§8.5): by the bottle's column (the reading), at random (the wiki,
  VCMI). No rule reads a text (the engine holds none, `AGENTS.md`); the client lane shows the
  original's (by column): a note for its inbox when the bottle is built, no dispute.
- **A-27 (quirk candidate). A computer player's hero leaves an Ocean Bottle on the map.**
  `visit_object`'s cases 59 and 91 return at once when the visit is not a human's (0x4A9AFF,
  0x4A9FC5): a computer's hero that steps toward a bottle reads nothing and the bottle stays,
  where a human's leaves. Harmless (a bottle gives nothing) and unreachable until E10's computer
  players. **Proposed:** not reproduced, no flag (as A-17 and A-19), revisited with E10.
- **A-28 (quirk candidate). Boarding or landing without the Admiral's Hat keeps the day's
  maximum of the surface left.** `hero_boards_boat` and `hero_lands_from_boat` set the day's
  maximum (+0x49) only with the Hat (0x4A1088, 0x49E6C1); without it the points
  become 0 and the maximum stays until the next turn start (the second reading, §13). The Hat's
  conversion is the only rule that reads it: a hero that gains the Hat at sea (assembled from a
  Survivor's or a Sea Chest's part) lands with left × land ÷ land, its points unchanged, where the
  two maxima would give left × land ÷ sea; the movement bar shows the stale maximum. Rare and
  harmless (no division by 0: a day's maximum is never 0). **Proposed:** not reproduced, no flag:
  the engine converts by the two surfaces' maxima at the visit.
- **Not read** (for E10 or later): the computer player's answers to the pickups
  (`ai_artifact_received` 0x433B40), the dialogs' drawing, the sounds.

## 11. Batch E8's plan (T66a2)

Increment 8's batch, **batch E8**, runs on the client lane's harness in SoD on Easy, hot seat,
from maps `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E7 (decision
159's agent's call). Its spec and recipe are tasks of T66b's cut; this is the plan. **Two runs**:
**E8a** movement and the shore (E8-1 to E8-5, E8-7, E8-10, E8-15, E8-16), **E8b** the pickups and
the battles (E8-6, E8-8, E8-9, E8-11 to E8-14). The points of D-41 and D-43 need exact
starting points: the recipe walks a hero a computed number of steps first, as batch E2 did.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E8-1 | D-41 (boarding) | a hero with the Admiral's Hat boards a boat with about 50 points left after the step; another with about 500 (the control) | points, day's points and status after each boarding | 0 for the first (VCMI: about 50 × sea ÷ land); 500 × sea ÷ land, truncated, for the control (both) |
| E8-2 | D-41 (landing) | the first hero, in its boat, lands on a coast tile without a road with about 50 points left after the step; the control with about 500 | the same after each landing | 0 (VCMI: about 50 × land ÷ sea); 500 × land ÷ sea for the control |
| E8-3 | D-42 | a boat beside three coastal tiles: an event of the boat's owner giving 100 gold, a hole (124) and a free one | the path's marks on a click at each, the treasury, the hero's tile | no path to the event's or the hole's tile, the event not fired; the free tile lands (VCMI lands on the event and fires it) |
| E8-4 | D-43 | a boat on a Favorable Winds tile with 80 points left: a diagonal step onto water (93), then a straight one (66) | points and tile after each | the diagonal refused (no move), the straight taken: 14 left (VCMI: the diagonal taken, 0 left) |
| E8-5 | D-44 | a hero written on a water tile, no boat there | status (bit 18), day's points at its turn start; a step onto water, then one onto land | no boat, the land's points; the step onto water refused, the one onto land taken (VCMI: a boat under it and the sea's points) |
| E8-6 | §8.3, §8.4 (a full backpack) | a hero in a boat with 64 artifacts in the backpack visits 30 Sea Chests (about three hold an artifact) and 3 Survivors | each object's word before, the treasury, the backpack's count | an artifact chest gives 1 500 gold and no artifact, a Survivor nothing and leaves (VCMI: the artifacts given) |
| E8-7 | D-45 | four heroes on the coast, each beside a Flotsam on the water: two with Angel Wings, one with Boots of Levitation, a walker; the first flyer clicks its Flotsam, the second presses the arrow key toward it, the others click | the path's marks, the treasury, the Flotsam's word after each | the clicking flyer takes its Flotsam; the key's flyer, the water walker and the walker take nothing (no path, no step) (the wiki: only boats) |
| E8-8 | §8.2 to §8.4 (the draws) | 40 Flotsams, 40 Sea Chests and 40 Survivors | each word after the new game; one of each kind visited | Flotsam 0 to 3, about 10 each; the chests' kinds about 8, 28 and 4; the Survivors' classes about 22, 8, 8 and 2; the gains as §8.2 to §8.4 |
| E8-9 | §8.5 | four Ocean Bottles: three without a text at columns x, x + 25 and x + 1, one with a text | each message's text, the bottle after | the first two show the same default text, the third another; the fourth its text; every bottle gone |
| E8-10 | §4.2 (a check) | a hero with Advanced Navigation and the Necklace of Ocean Guidance and one with the Sea Captain's Hat end their turns in boats | day's points at the next turn start, then after a landing | 4 000 and 2 000 at sea (3 000 + 1 000; 1 500 + 500); the land's after the landing |
| E8-11 | §7.1, §7.3 (the deck) | Blue's hero in a boat beside the shore; Red's hero attacks it from land and wins | the opening's terrain, special field, ship flag and blocked hexes; Blue's boat's record and its tile after | water, no field, the ship flag, at most the water obstacle; the record freed and the tile empty |
| E8-12 | §7.1, §7.3 (two boats) | Red's and Blue's heroes in boats side by side; Red attacks, Blue retreats | the same at the opening; Red's status and tile after; Blue's boat | water, the ship flag, exactly the 32 hexes; Red still in its boat; Blue's boat gone |
| E8-13 | §7.2 (a monster on water) | a monster written on water beside a boat's way; the boat sails across a tile it guards that is not the path's end | where the battle starts, its terrain and ship flag | at the guarded tile at once; the deck |
| E8-14 | §7.2, §4.3 (events on water) | two events of Red's on water on a boat's way: one with 100 gold, one with guards | the tile and treasury after the first; the second's battle | the first fires and ends the move on its tile; the second's battle on the deck |
| E8-15 | §3.2, §3.3 (a guarded landing) | a boat lands on a free coastal tile a monster guards | the battle's terrain and field; the hero's status after; the boat's record (tile, facing, aboard) | sand and the shore (field 0); the in-boat flag clear; the boat on the water tile behind, facing the landing's way, nobody aboard |
| E8-16 | §2 (any hero boards) | Blue's hero boards the boat Red's hero left at a landing | the boat's owner and captain after | Blue's |

**Not in the batch:** the 64 boats (no shipped map comes near; the cap's question is §14), the
Admiral's Hat's proportion itself (consensus, E8-1's control checks it), the computer player
(A-27, E10), the sea's other objects (decision 158).

**What the harness lacks** (`clients/capture/` is a common path, CLT-Q18: this lane builds it
with the harness check and tells the client and map-client lanes' inboxes): reads of the boats'
records (game +0x4E3BC, `structs.toml` "boat": tile, in use, type, facing, owner, captain,
aboard), the battle's ship flag (`battle_manager` +0x53C6) and its blocked hexes at the opening;
a click at a coastal tile from a boat and at a boat from land (the walk's clicks exist). **What
the spec lacks** (`h3-experiment`, battle-owned: `cross-lane`; the writer `h3_map::writer` is
this lane's): boats (class 8 by type, CLT-Q5), the four pickups (29, 82 and 86 without a body, 59
with its text), holes (124), Favorable Winds (225, a ground), a hero and a monster on a water
tile.

## 12. The proposed rule ids (T66a2)

For T66b's cut; ids follow the catalogs' prefixes (the last ones: ADV-MAP-002, ADV-MOV-011,
ADV-OBJ-007, ADV-FOG-007, ADV-PIK-013, ADV-MON-012, GAM-BAT-014, HER-ADV-008), each written when
its rule is built. `ADV-SEA` (boats and the sea) is a new family.

| Id | Rule | Survey | Open points |
|---|---|---|---|
| ADV-SEA-001 | boats on the map: the import (class 8, no body, the type its subclass and appearance only), any hero boards any empty boat, the owner and captain kept and not read | §1, §2, §5 | the 64 boats |
| ADV-SEA-002 | boarding: a step from land onto an empty boat's tile, no guard test there, the walk ends; Fly and Water Walk cleared, the sea's maximum, the points 0 or the Admiral's Hat's proportion after the drop | §3.1 | D-41 |
| ADV-SEA-003 | landing: only onto a free coastal tile (no object, event or hole), the boat left on the water tile facing the hero's way, the walk ends; the land's maximum, 0 or the Hat's proportion; the landing tile's guards after | §3.2, §3.3 | D-41, D-42 |
| ADV-SEA-004 | sailing: the step's price at sea (100, 141), Favorable Winds (66, 93, a boat only), a diagonal only between two waters, no empty boat from a boat, the drop at sea, the diagonal exception's comparison | §4.1 | D-43 |
| ADV-SEA-005 | what a boat visits: objects visited from the neighbouring water tile, another boat's hero from the neighbour; a hero on land visits a boat's hero from the shore, never the reverse; events on water fire for a boat | §4.3 | — |
| ADV-SEA-006 | a boat leaves the map with its hero | §2, §7.3 | — |
| ADV-SEA-007 | a hero placed on water | §2, §5 | D-44 |
| HER-ADV-009 | a hero's day's sea points in `h3-heroes` (Navigation, its specialists, the Necklace, the two Hats; ADR-0013 point 2) | §4.2 | — |
| ADV-MOV-001, -002 (amended) | the day's points set at the turn start by the hero's boat state | §4.2 | — |
| ADV-MOV-007 (amended) | where a hero may step: water by boat, land from a boat only by a landing | §3, §4.1 | — |
| ADV-FOG-006 (amended), ADV-FOG-008 | what a player sees of a hero in a boat (its boat and flag) and of an empty boat (place and type, no owner) | §4.4, §9 | — |
| ADV-PIK-014 | Flotsam: its draw and the four outcomes | §8.2 | — |
| ADV-PIK-015 | the Sea Chest: its draw, the full backpack's 1 500 gold | §8.3 | — |
| ADV-PIK-016 | the Shipwreck Survivor: its draw by class, the full backpack's loss | §8.4 | — |
| ADV-PIK-017 | the Ocean Bottle: its message (the map's or the default by column), then it leaves | §8.5 | A-27 (E10) |
| ADV-PIK-018 | who takes a pickup on the water: a boat from the neighbouring tile; a flyer from the shore by a click | §8.1 | D-45 |
| GAM-BAT-001 (amended) | the battlefield: a hero in a boat on either side, the deck (water, the ship); both, the two boats | §7.1 | — |
| GAM-BAT-003 (amended) | a hero leaving the map takes its boat (ADV-SEA-006) | §7.3 | — |
| ADV-OBJ-007, ADV-MOV-011 (amended) | events on water: fired by a boat's step, the move's stop at sea | §4.3 | — |

## 13. The second reading (T66a2)

Two fresh agents, one after the other, re-read the deciding code without this survey or the
address map's summaries (2026-10-28), from questions per function: **A** the boats' records, the
loader, boarding, landing, the move step's boat sites, the guard test, the hero sent to the pool
and the coast's class 3 (nine questions); **B** the boat's search, the step's checks, the winds,
the battle's three boat tests, the four pickups' visits and draws and the bottles' loader (eight
questions).

**A confirmed** point by point: `create_boat`'s records (the first free one reused, else
appended, none past 64: a map's 65th boat is dropped and its object's word is −1), a new boat
facing 2 with no owner, captain or anyone aboard; the loader's boat without a body and with the
template's subclass as its type; `visit_object`'s cases 3 and 8 with no test of the hero or the
owner (§2); boarding's fields, the cheat's skip and the Hat's proportion as a signed multiply then
a truncating division by the old maximum (§3.1); landing's, with the facing from the step
(§3.2); the boat put back on the water tile left, facing the step's way, nobody aboard, the owner
and captain kept (§3.2); the class switch's boat and hero cases and the visit after the step
(§3.1, §4.3); the guard test's skips for a boat's step onto class 3 and onto a boat, a boat
attacked at once elsewhere (§3.3); the pool's freeing of the boat (§2); class 3 from the coast
flag on land, replaced by an entrance (an event's too), a blocked cell or a hole (§3.2).

**A corrected two points** (both checked in the code and written into the sections):

- **The day's maximum without the Hat** (§3.1, §3.2): boarding and landing set +0x49 only with
  the Admiral's Hat; without it the points become 0 and the maximum of the surface left stays
  until the next turn start: **A-28** (§10).
- **The drop at sea on Favorable Winds** (§4.1 point 5): the cheapest step from a winds tile is
  the winds' 66, not 100 (`cheapest_step_from` prices a step from the tile itself with the
  winds).

**A added:** landing leaves the Fly and Water Walk fields as boarding cleared them; the landing's
guard attack takes the current player's selected hero's tile (the hero that just landed); the
loader's 64 boats drop a later boat silently (§6, the 64 boats).

**B confirmed** point by point: the boat's search (the price by `step_cost` with no road, no
native terrain, no flying or water walking; the next day at the sea's maximum; no boat's tile from
a boat; the diagonal between two waters; land only of class 3, a leaf in the human player's mode
0) (§4.1); `hero_step_allowed`'s cases for a boat, a flyer, a water walker and a walker, and its
fourth argument (§4.1, §8.1); Favorable Winds' two thirds on a water source before the diagonal's
√2, for a boat only (§4.1); the battle's order (a siege, either hero in a boat or a ship's bank,
underground, a mine, the coast, the tile), the pictures (both in boats `CmBkBoat`, the ship
`CmBkDeck`) and the 32 hexes only with both in boats, the deck's random obstacles on water (§7.1);
the four pickups' visits and draws as §8.2 to §8.5, the human flag's skip at the bottle, the
loader's message records and the default text by the column.

**B corrected one point** (checked in the code and written into §8.1 and D-45): **a water walker
never takes a sea pickup from the shore**: its search refuses a water tile holding an entrance
other than a boat's, a hero's or class 85's (0x4B29AC to 0x4B29E4), and the arrow keys' check
passes the fourth argument 1 (0x408ED5), which treats flyers and water walkers as walkers. A
flyer takes one only by a click, on its path's last step (0x480B44 to 0x480BD0), from a free land
tile.

**B added:** a bottle's record with its flag set and no text shows an empty text; a
step `hero_move_step` itself refuses zeroes the hero's points when its fifth argument is set
(0x480CDF; its callers not followed: no path or key reaches a pickup this way, §8.1); the fly
price ignores roads and the diagonal's √2 is dropped between the price and its √2 (T34a's
readings, unchanged).

**Not read by either:** the computer player's boats and answers (E10), network play, saved games,
the drawing, the sounds.

## 14. For T66b: the checkpoint's questions (ADV-Q32)

1. **The disputes D-41 to D-43 and D-45** (§6, §10): (a) the code's side, built and confirmed or
   reverted by batch E8's row (as decision 129); (b) only once batch E8 confirms it; (c) VCMI's
   where the wiki agrees. D-42 has the wiki with the reading; D-41, D-43 and D-45 the reading
   alone (the wiki silent on D-41 and D-43, against it on D-45). Recommended (a).
2. **D-44, a hero placed on water:** (a) the import refuses it (an `Unsupported`; no shipped map
   has one) until batch E8's E8-5 shows what the original does; (b) VCMI's boat under it; (c) the
   reading: on water without a boat. Recommended (a).
3. **The 64 boats** (§2, §6): (a) no original limit, as decision 133 (a bound the import
   validates, as its others); (b) the original's 64. Recommended (a).
4. **Batch E8** (§11): (a) all 16 rows in two runs before the build; (b) beside the build; (c)
   the disputes' rows only (E8-1 to E8-5, E8-7); (d) none. Recommended (a), as batch E7.
5. **The quirk candidates** A-27 (a computer's hero leaves a bottle) and A-28 (the day's maximum
   kept without the Hat): not reproduced, no flag (A-27 revisited with E10). Recommended: agreed.
6. **The common rules as written** (§7 to §9): the battle at sea by the heroes' boats, the
   pickups' draws and visits (the full backpack's turn and loss with `ADV-QUIRK-22`), the bottle
   named in its event and its text the client's, no new command or decision. Recommended: as
   written.
7. **What stays out of increment 8:** decision 158's list, the computer player's boats (E10), the
   sea's creature banks' battles (item 10). Recommended: confirmed.
