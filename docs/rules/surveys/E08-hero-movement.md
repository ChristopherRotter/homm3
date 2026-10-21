# E08 hero movement survey (E08 T34a): the day's points, the step's price, flying, water walking, the scouting radius

Date: 2026-10-21. Scope: E08 increment 5 (decisions 71, 72; T34a whole and flying and water
walking in: decisions 73, 74), input for T34b's ADR amendment drafts, the design note and the cut.
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog
with its sources and tests. **The checkpoint** (ADV-Q20, 2026-10-22: the epic's decision 75) decided
the open points; §8 and §9 carry its answers.

**Sources** (retrieved 2026-10-21):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path: the day's points (`hero_max_land_movement` 0x4E4990, whole; its table loader
  `load_movement_table` 0x4D7240) and every write of a hero's points left; the step's price
  (`step_cost` 0x4B1740, whole) with its callers (`hero_step_cost` 0x4B18C0,
  `cheapest_step_from` 0x4B1880, the move's step `hero_move_step` 0x4805E0 around its calls);
  the pathfinder's set-up and loop (`pathfinder_run` 0x56B440) and its expansion
  (`pathfinder_expand` 0x4B2300, its branches for the human player's modes 0–2);
  `hero_flight_tile_test` 0x4E5CE0; the reveal's callers (`hero_scouting_radius` 0x4E4070,
  `reveal_disk` 0x49D160). Not followed: the computer's planning mode 4 (the Dimension Door,
  Fly and Water Walk options, `pathfinder` +0x15 to +0x17), boats (embarking, landing, sailing),
  network play, the click handler's own test of an empty path (§3.4).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `lib/pathfinder/TurnInfo.cpp`,
  `lib/pathfinder/CPathfinder.cpp` (`getMovementCost`, `getTileMovementCost`,
  `passOneTurnLimitCheck`), `lib/pathfinder/PathfindingRules.cpp`,
  `lib/pathfinder/PathfinderUtil.h`, `lib/mapObjects/CGHeroInstance.cpp`,
  `lib/entities/hero/CHeroHandler.cpp` (`createSecondarySkillSpecialty`), `lib/CSkillHandler.cpp`,
  `config/skills.json`, `config/artifacts.json`, `config/spells/adventure.json`,
  `config/gameConfig.json` (`originalFlyRules` true); links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Movement, Logistics, Pathfinding, Scouting, Scouting radius, Fly, Water Walk, Angel Wings, Boots
  of Levitation, Native terrain. The template that shows a HotA and a SoD value prints each value
  before its icon ("+400 [HotA] 600 [SoD]"); the summariser read the first as SoD twice and was
  corrected by the page's own Logistics sentence. Re-check a quote before a rule cites it with
  `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §7.
- `computed`: §1.3's table of the Logistics specialty comes from a throwaway script that follows
  the reading (not committed); test vectors to confirm, not facts.

## 0. Findings in short

- **The day's land points** (§1): the land table's entry for the slowest speed (a creature
  specialty's creature and its upgrade one faster), times 1 + the Logistics rate (10, 20, 30 %;
  a Logistics specialist's rate grown by 5 % of itself per level, in single precision), then
  +600 for Boots of Speed and +300 for Equestrian's Gloves (worn, or in a worn combination; no
  combination holds them), then +400 after Stables (an object: later). The percentage scales the
  table's points only. The wiki and VCMI agree; VCMI's integer bonus arithmetic rounds the
  specialty differently (D-22).
- **When** (§1.4): only at the owner's turn start (ADV-MOV-002). Nothing in the day changes the
  points left: a level-up's Logistics, an artifact equipped or received change only the day's
  maximum the hero screen shows (+0x49, written when the screen closes). VCMI and the wiki agree.
- **The step's price** (§2): Pathfinding is the table by level the adventure part has since T29
  (VCMI's max(100, cost − 25 × level) gives the same). Cursed Ground (class 21 on the step's
  source tile) takes away the native terrain's 100 (VCMI has no such rule, the wiki is silent:
  D-23). Nomads in the army price sand as dirt. **Flying** (Angel Wings, the Fly spell's level 3)
  replaces the price by 100 from every inland tile, roads included, and takes the lesser of 100 and
  the walking price only from a coastal land tile; the wiki's table and VCMI take the lesser
  everywhere (roads 75, 65, 50): **D-24, a conflict of sources**. **Water walking** (Boots of
  Levitation) prices a step *from* water at 100; VCMI and the wiki agree at level 3. The diagonal
  exception compares the points with the source's terrain price by Pathfinding even then.
- **The move's drop to 0** (ADV-MOV-006) prices the cheapest next step without the artifacts' flying
  and water walking (the hero's spell fields only): quirk candidate **A-12** (§2.5).
- **The pathfinder** (§3): a flying hero passes over blocked tiles, water, guarded tiles (the
  zone of control is ignored, the move attacks it only at its destination) and **unexplored tiles**
  (A-13, an information leak), never over rock; a water-walking hero passes over water that holds
  no object. Neither may stop on such a tile (a move's end and the next day's start need the
  tile's "may stop" node: the second reading, §3.4), nor visit a neighbour's object that is visited
  from the neighbouring tile, nor enter a town, from it: the wiki ("an unoccupied location (on
  land)") and VCMI agree (D-25 settled). A flying hero standing inland has its start node marked
  like a tile it flies over (`hero_flight_tile_test` 0x4E5CE0 passes only on coastal land), and the
  move's step refuses a flyer's visit from the neighbouring tile there: either a misreading or a
  quirk of the original (A-14: withdrawn at the checkpoint, the user: a flyer visits as a walker).
- **The scouting radius** (§4): 5, 6, 7, 8 by Scouting level, +1 for the Speculum and +1 for the
  Spyglass (worn or in a worn combination), as read in T8a. The Scouting specialty's 1 + 0.05 ×
  level is in the code, but **no SoD hero has the specialty** (the wiki: HotA's Beatrice, Miriam,
  Wynona): nothing to build. A radius that grows in the day reveals nothing until the hero's next
  step or the next day's reveal around every owned hero; VCMI reveals at once (D-26).
- **Content** (§5, §7): six artifacts (Boots of Speed, Equestrian's Gloves, Angel Wings, Boots of
  Levitation, Speculum, Spyglass), three Logistics specialists (Kyrre, Gunnar, Dessa), the creature
  specialists' speed (51 heroes), Nomads and Cursed Ground. In the shipped maps 1 of the 5 maps
  that import whole changes (`Tutorial.tut`: its hero wears Boots of Speed and Equestrian's Gloves
  with Expert Logistics and Pathfinding), Gunnar stands on another (`Adventures of Jared Haret`).
- **Not in increment 5** (§6): the sea's points (Navigation and its specialists, the Necklace of
  Ocean Guidance, the Sea Captain's Hat, the Admiral's Hat, Lighthouses), Stables, the spells Fly
  and Water Walk at levels 0 to 2, the computer players' bonus (+75 on Expert and Impossible, +50
  more for one kind of computer player, players 0 to 5 only).

## 1. The day's land points (`hero_max_land_movement` 0x4E4990)

### 1.1 The reading

`hero_max_land_movement` (thiscall, the hero, one argument: at sea) is read whole. On land:

1. A hero flag (+0x105 bit 24) returns 1 000 000 at once (not set by any rule read; a cheat's
   flag, H3API leaves it unnamed).
2. **The slowest speed**: from 20 down, the lowest speed (the creature record's +0x50) of the
   seven slots; a slot holding the hero's specialty creature (a `hero_specialty_record` of type 1)
   or its upgrade counts one faster (the Ballista specialist's creature 146 has no upgrade; with
   the game version flag `[game]+0x1F698` clear the four elementals have none either). Read in E08
   T4; the creature table's speed, so the speed artifacts (Ring of the Wayfarer, Necklace of
   Swiftness, Cape of Velocity) do not count.
3. **The table**: `land_movement_by_speed` (0x698A98) at that speed (ADV-MOV-001).
4. **Logistics**: the rate `skill_logistics` (0x63EA68: f32 0, 0.1, 0.2, 0.3) at the hero's level
   of skill 2 (+0xCB). With Logistics and a Logistics specialty (type 0, skill 2) the rate becomes
   f32(rate × (f32(level) × 0.05 + 1)) (`skill_specialty_growth` 0x63EAE4, the hero's level +0x55),
   the same growth `h3_heroes` applies to the battle's skill specialties (`Growth::single`,
   Q-0005). The points are trunc((rate + 1) × points) on the x87 (53 bits).
5. **Artifacts**: +600 (`movement_bonus_boots` 0x698B00) if Boots of Speed (98) is worn or part
   of a worn combination (`hero_wears_artifact` 0x4D91F0); +300 (`movement_bonus_gloves`
   0x698AFC) for Equestrian's Gloves (70).
6. **Stables**: +400 (`movement_bonus_stables` 0x698A94) while the hero's flag +0x105 bit 1 is
   set (the Stables object's visit sets it: objects, later).
7. **A computer player's bonus** (land and sea): for an owner 0 to 5 that is not human, on a
   difficulty above 2 (`game` +0x1F6D8: Expert, Impossible), +75, and +50 more when the player
   record's `ai_type` (+0x34) is 2. Players 6 and 7 (teal, pink) never get it. E10's.

At sea (argument 1, boats' increment): `sea_movement_by_navigation` (0x698AEC) at the
Navigation level (+0xCE); a Navigation specialist with Navigation adds trunc(level × the
table's first entry / 20) (+75 per level); +500 (`movement_bonus_lighthouse` 0x698B0C) per owned
Lighthouse object, per Castle town with its Lighthouse built (the loop over the towns tests no
owner: a lead for the sea's increment) and for the Sea Captain's Hat (123); +1 000
(`movement_bonus_necklace` 0x698B04) for the Necklace of Ocean Guidance (71); then the computer's
bonus.

**The table's file** (`movement.txt`, `load_movement_table` reads column 6 of rows 2 to 7 into
0x698AFC, 0x698B00, 0x698B04, 0x698B08, 0x698A94, 0x698B0C): 300 "Equestrian Gloves", 600 "Ring of
the Wayfarer" (the code gives it to Boots of Speed, 98), 1 000 "Necklace of Ocean Guidance", 600
"Angel Wings" (0x698B08: no reader in the day's points; one reference elsewhere, not read), 400
"Stables", 500 "Lighthouse". The amounts are data; the artifacts that get them are code.

### 1.2 VCMI and the wiki

`vcmi` (`TurnInfo::TurnInfo`): the land table by the lowest stack's speed *with its bonuses*
(`getLowestCreatureSpeed` uses `getInitiative`, marked "TODO? should speed modifiers (eg from
artifacts) affect hero movement?"), then the bonus system's `MOVEMENT` (land): Logistics
`PERCENT_TO_BASE` 10/20/30 (`config/skills.json`), Boots of Speed `ADDITIVE_VALUE` 600, Equestrian's
Gloves 300 (`config/artifacts.json`). A Logistics specialist: `createSecondarySkillSpecialty`
copies the skill's bonus as `PERCENT_TO_TARGET_TYPE` of the skill's bonuses with
`TimesHeroLevelUpdater`, 5 per level (`specialtySecondarySkillGrowth`): the skill's 10 % grown by
5 % per level, in integers.

`wiki` (Movement): "Basic Logistics increases base movement on land by 10% (5% in HotA)",
advanced 20 %, expert 30 %; "Boots of Speed give +400 [HotA] 600 [SoD] points on land",
"Equestrian's Gloves give +200 [HotA] 300 [SoD]", Stables "+300 [HotA] 400 [SoD] movement points
per day for a visiting hero for the rest of the week"; "Logistics and Navigation do not affect
bonuses from artifacts and adventure map locations"; "Artifacts increasing creatures' speed (like
Necklace of Swiftness), the hero speciality of Sir Mullich and native terrain bonuses don't affect
the base movement, but a hero's creature specialty does." (Logistics) a specialist's bonus grows
by 5 % of itself per level ("relative, not absolute"). The Logistics page's own SoD values came
back as 5/10/20 % from the summariser: HotA's (the Movement page and the code have 10/20/30).

**Consensus** on the order (the percentage on the table's points, the flat bonuses after) and the
amounts. **Speed artifacts**: the code and the wiki leave them out, VCMI counts them (its own
TODO): 2 sources to 1, no row needed. **D-22, the specialist's rounding**: the code's
single-precision rate; exact arithmetic (the strict profile without Q-0005) differs from it by one
point in 147 of the 2 664 cases (12 table entries × 3 levels × levels 1–74), VCMI's integer percent
of a percent in more. Q-0005's flag already chooses between the original's single precision and
exact rates for the battle's specialties; the same flag fits. Row E5-2.

### 1.3 The specialist's points, computed

| Table | Logistics | Hero level 1 | 2 | 3 | 6 | 12 | 20 |
|---|---|---|---|---|---|---|---|
| 1500 (speed 3) | Basic | 1657 | 1664 (exact 1665) | 1672 | 1694 (1695) | 1739 (1740) | 1800 |

| Table | Logistics | Hero level 1 | 2 | 3 | 6 | 9 | 12 |
|---|---|---|---|---|---|---|---|
| 2000 (speed 11+) | Basic | 2210 | 2219 (2220) | 2230 | 2259 (2260) | 2289 (2290) | 2319 (2320) |

Where they part, the rate's f32 rounding falls below the exact product. Among the table's entries
an army can reach (speed 3 and up) the code and exact arithmetic part for 1500, 1700, 1900 and 2000
at levels 2, 6, 12, 18, 24 (Basic; Advanced also 9, 15, 21, 27) and for 1560, 1760, 1960 at 15.
Without the specialty the f32 rates give exactly ⌊points × (10 + level) / 10⌋ for every entry.

### 1.4 When the points change

- **Set at the owner's turn start** (`next_player_turn` 0x4C6FE0, E08 T6b, ADV-MOV-002): the maximum
  (+0x49) and the points left (+0x4D) to `hero_max_movement` 0x4E4D90 (land or sea by the boat
  flag +0x105 bit 18). A placed hero's setup does the same.
- **Nothing in the day adds to the points left.** Every instruction that writes a dword at
  +0x4D was listed (57 sites, a hero's points left among them): the turn starts, the setups, the move's step and its visits from a neighbouring
  tile, objects' visits (Stables and other objects, later), boarding and landing (proportional with
  the Admiral's Hat 136, else 0: `hero_boards_boat` 0x4A1010, `hero_lands_from_boat` 0x49E670),
  battles, a cheat. `hero_level_up` 0x4DA720, `hero_learn_secondary_skill` 0x4E22D0,
  `hero_equip_artifact` 0x4E2A00, `hero_remove_artifact` 0x4E2BD0 and `hero_give_artifact`
  0x4E3070 call neither the day's points nor a reveal. The hero screen, when it closes, writes
  only the day's maximum (+0x49 ← 0x4E4990 for the hero's boat state, site 0x4E1A33): the bar
  shows the new maximum, the points left stay.
- `vcmi`: `movement` is set to the limit at the new day (`NewTurnProcessor`) and changed only by
  moves and objects; equipping an artifact does not touch it. `wiki`: "Base movement is calculated
  only at the beginning of each day"; "Heroes gain additional movement points when wearing certain
  artifacts at the beginning of a day." **Consensus**: a gain in the day takes effect at the
  owner's next turn start. Row E5-3 checks a level-up's Logistics.

## 2. The step's price (`step_cost` 0x4B1740)

### 2.1 The reading

`step_cost` (the source tile, the direction; on the stack the points left, the Pathfinding level,
the destination's road, the fly level, the water-walk level, the army's native terrain, whether it
holds Nomads) is read whole (E08 T7a read its walking part, §1.2 of `E08-adventure-core.md`):

1. **Nomads**: with a Nomad stack (creature 142, `army_count_of_type`) the source's sand (1)
   counts as dirt (0) for everything below, the diagonal's bound included.
2. **The walking price**: both tiles with a road → the source road's price (75, 65, 50, flat over
   Pathfinding); else the army's native terrain on a tile that is not Cursed Ground (the source
   tile's special ground 21, `tile_special_ground` 0x4FCE20) → 100; else `step_cost_table`
   (0x63E510) at the terrain and the Pathfinding level (the adventure part's `pathfinding` costs
   since T29).
3. **Favorable Winds** (225) on water: two thirds, truncated (the sea).
4. **Water walking** (level ≥ 0) on a water source: `fly_walk_step_cost` (0x6778AC: 140, 140, 120,
   100 by level) replaces the price.
5. **Flying** (level ≥ 0): from a coastal land tile (the tile's flags +0xC bit 0x40, the map file's
   coast flag) the lesser of the fly price and the price so far; from any other tile, inland land or
   water, the fly price replaces it.
6. **The diagonal**: the exception (A-2, A-3) compares the points left with the source terrain's
   `step_cost_table` price by Pathfinding (after Nomads; water 100), never the fly or water-walk
   price; the charge is the price of 1–5 straight or trunc(price × √2).

The levels come from `hero_step_cost` 0x4B18C0 for the move and from `pathfinder_run`'s
set-up for the search: the fly level is the hero's +0x112 (−1 unless a Fly spell is on), forced to 3
by Angel Wings (72); the water-walk level +0x116, forced to 3 by Boots of Levitation (90); both −1
in a boat (+0x105 bit 18). The Pathfinding level is the hero's skill 0 (+0xC9).

**With the artifacts** (level 3): a flying hero pays 100 for every straight step from an inland
tile, 141 diagonally; a road under it costs 100 as well, and swamp, sand, snow and rough cost 100;
from a coastal land tile it pays the lesser of 100 and the walking price (a road 75, 65, 50). A
water-walking hero pays 100 (141) for a step from water and the walking price for a step from land
onto water.

### 2.2 Pathfinding, Cursed Ground and Nomads against VCMI and the wiki

- **Pathfinding**: `vcmi` `getTileMovementCost`: max(100, terrain − 25 × level) (`ROUGH_TERRAIN_DISCOUNT`
  25/50/75), roads unaffected, the native terrain min(100, …); the same numbers as 0x63E510.
  `wiki` (Pathfinding): "reducing the cost of moving one tile by 25 movement points" per level,
  minimum 100 in SoD (95, 90, 85 in HotA), "Pathfinding is useless on roads". **Consensus.**
- **Cursed Ground** removes the native terrain's 100 on its tiles (the step's source), and
  ADV-MOV-006's cheapest step there is the terrain's price. `vcmi` 1.7.5 gives the object only its
  battlefield (`config/objects/generic.json`); `wiki`: Native terrain and Movement are silent.
  **D-23**: the code alone, against VCMI's silence; row E5-6.
- **Nomads**: `vcmi`: the creature's `NO_TERRAIN_PENALTY` for sand (min(100, …)); `wiki`: "a hero
  can eliminate the sand terrain penalty if he has at least one Nomad on his army." The code's
  sand-as-dirt gives 100 too, and dirt's 100 as the diagonal's bound. **Consensus** for the price;
  Nomads are AB content (no shipped hero carries them in a custom army, §7).

### 2.3 Flying and water walking against VCMI and the wiki

- `vcmi` `getMovementCost`: in the air layer (flying) the price is the lesser of the walking price
  and 100 + the bonus (Angel Wings 0, the Fly spell 40, 40, 20, 0 by level: `config/spells/
  adventure.json`), on every tile; in the water layer (a water tile destination) the walking price
  of the *source* tile × (100 + the bonus) / 100; the diagonal then × √2 with VCMI's own exception.
- `wiki` (Fly): "Allows the casting hero to move over water and any obstacles on the map with 40%
  penalty" (20 %, none at advanced and expert); "When flying over terrain that has its own penalty
  for the hero, the lesser of the penalties is applied"; its table gives roads 75, 65, 50 and dirt,
  grass, lava, subterranean 100 at every Fly level; "Favorable winds do not affect movement under
  the Fly spell". (Water Walk) "with 40% penalty … 20% … without penalty, provided the end
  destination is an unoccupied location on land". Angel Wings: "Permanent Expert Fly while
  equipped"; Boots of Levitation: "Permanent Expert Water Walk while equipped".
- **D-24, flying's price: a conflict of sources.** The code replaces the price by the fly price
  from every inland tile (a road costs 100 under Angel Wings) and takes the lesser only from coastal
  land; the wiki and VCMI take the lesser everywhere. Row E5-4 (an Angel Wings hero on an inland
  cobblestone road: 100 by the code, 50 by the others).
- **Water walking at level 3 agrees** (100 from water, the land tile's price onto water); at levels
  0 to 2 the code prices a step from water at 140 or 120 and a step from land onto water at the land
  price, VCMI a step onto water at the land price × 1.4 or 1.2 and a step from water at 100: the
  spells' levels, not increment 5's.

### 2.4 Rule candidates

The engine's `Pricing` (ADV-MOV-003 to -006) gains: the hero's Pathfinding level (the table it
already holds), Cursed Ground on the source tile, Nomads, and the fly and water-walk levels from the
hero's artifacts (3 or none in increment 5; a level field for the spells later), each with the
diagonal's bound as above.

### 2.5 Quirk candidate A-12: the drop to 0 forgets the artifacts

After each step the move sets the points left to 0 when they are below the cheapest straight step
from the new tile (ADV-MOV-006; `hero_move_step` 0x48084E → `cheapest_step_from` 0x4B1880, which
prices the tile against itself: its road, the native 100 unless Cursed Ground, the fly and
water-walk levels). The move passes the hero's own fields +0x112 and +0x116, not the artifacts'
level 3 that `hero_step_cost` forces. So for a hero with Angel Wings the drop prices a walking
step: on an inland road with 60 points left it keeps them, though every step costs 100 (harmless:
no step is affordable either way, the points shown differ); **on Cursed Ground** (swamp 175, sand
150) with fewer points than the terrain's price but at least 100 it **drops points an affordable
flying step needed**. With Boots of Levitation on water the walking cheapest is 100, the same.
Suggested class: **fix-optional** (faithful drops as the original; strict prices the drop with the
hero's flying and water walking). VCMI: its own last-step rule (D-3), no such case. Rare: no shipped
hero wears Angel Wings (§7).

## 3. The pathfinder with flying and water walking

### 3.1 The reading (`pathfinder_run` 0x56B440, `pathfinder_expand` 0x4B2300)

The search keeps one node per tile and kind (`pathfinder_cells_alloc`: two per tile) with a flags
word (+4): bit 2 in a boat, bit 4 flying, bit 5 water walking, bit 7 the computer's Dimension Door,
**bit 10 "may stop here"**: a node a move may end on and the next day may start from. A tile
has two nodes per level (`pathfinder_relax` stores a node at level × 2 + 1 when bit 10 is clear,
0x4B1C4B), so a tile flown over without stopping and the same tile stopped on are kept apart. The start node takes bit 4
and bit 5 from the pathfinder's fly and water-walk levels (`pathfinder` +0x1C, +0x18; set as in
§2.1) and bit 10, which it loses when the hero flies or water-walks and `hero_flight_tile_test`
fails (§3.3). Each neighbour of a popped node, in the eight directions (`pathfinder_expand`, the
human player's modes 0–2):

1. Off the map: none. **Rock** (terrain 9): none (also when flying).
2. The step is priced at the node's points (§2.1, with the node's flying and water walking). A
   step it cannot pay: only as the next day's first step, from a node with bit 10, priced *without*
   flying and water walking, the new node losing bits 4 and 5 (the multi-day path assumes a
   spell's end; the engine's reach is today's only: no rule).
3. **Blocked** (map item +0xD bit 0x01), **unexplored** for a human player (`pathfinder_fog_test`
   0x4B25D4), **a step up off an entrance** whose class is not visitable from above (E08-objects §2.3),
   and an entrance with an enemy hero standing in a Sanctuary (80): passable **only flying**,
   without bit 10.
4. **Water** (not in a boat): an entrance of a hero (34), a boat (8) or class 85 is a normal step
   (an attack, boarding: the sea's); another entrance on water as 3; plain water is passable
   **flying or water-walking**, without bit 10.
5. **A step down onto an entrance** not visitable from above: never (flying too).
6. From a node **without bit 10** (over a blocked tile, water, an unexplored tile, or a start node as
   in §3.3) no step enters an entrance of a class visited from the neighbouring tile ("cannot be
   entered", `object_settings` +0) or a town (98) as a stop: no visit from up there; a flying node
   still passes over it (point 7).
7. A flying node reaching an entrance is also stored without bit 10 (a second relax, 0x4B2D3D):
   the flight may go on over it without the visit.

`pathfinder_run` skips the guard test for a popped flying node (0x56BBCE: flags bits 4 or 7): a
flying hero's path goes on past guarded tiles, where a walking hero's ends (E08-battles §2.2); the
move attacks a flying hero only at its destination (`hero_move_step`, read in T19), and visits an
object only there: a flying hero not in a boat takes no object on a tile that is not its
destination (0x48147D to 0x4814E1, the second reading). A walking hero's step onto plain water
whose map item has +0xD bit 0x08 makes a separate boat node (0x4B2B01): boats', not read.

### 3.2 VCMI and the wiki

`vcmi` (`originalFlyRules` true, the default): layers land, sail, water (water walking), air
(flying). The air layer's tiles are all `FLYABLE` (`PathfinderUtil.h`); rock is not entered
(`isPassable`); "zone of control is ignored when flying" (`PathfindingRules.cpp`); a flying node
on a tile that is not free may not pass the turn (`passOneTurnLimitCheck`), nor visit a
"blockvis" object from a blocked tile or across the land and water line; the water layer enters
only free water and lands on free or visitable tiles; it may never pass a turn on water. The
pathfinder works on the player's explored tiles: unexplored ones are blocked in every layer
(`evaluateAccessibility`, ADV-MOV-008's source). `wiki`: (Fly) "It allows casting hero to fly over
terrain (except rock) obstacles to an unoccupied map location"; "Cannot be used by a hero on a
boat"; HotA changed it so that "Interrupting the flight above a guarded tile will now trigger
combat" (so in SoD it does not); (Water Walk) the end destination is "an unoccupied location on
land".

- **Over what**: consensus for blocked tiles, water and rock, and for guarded tiles (the code and
  VCMI ignore the zone of control; the wiki's HotA note implies SoD does).
- **A-13, a flying path through unexplored tiles**: the code treats an unexplored tile as blocked,
  which a flying hero passes over; VCMI never paths through them. The original's path (and its
  cost on the coast) then depends on hidden tiles: whether they are rock, coast or water. An
  information leak in the legal moves (ADV-MOV-008: "the legal moves never depend on hidden tiles").
  Suggested class: **always fixed** (the engine's moves stay on explored tiles; ADR-0017 point 8's
  mask leaks nothing). Row E5-9 confirms the original's path.
- **D-25, where a move may end**: settled, §3.4.

### 3.3 `hero_flight_tile_test` (0x4E5CE0) and A-14

Read whole: true when the hero's own tile matches its boat state (on land without a boat, on
water in one), carries the coast flag (+0xC bit 0x40, so only land next to water) and is not the
entrance of a class visited from the neighbouring tile. H3API calls it `CanFlyOnTile`. Its callers
(both read around the call):

- `pathfinder_run` (0x56B88C, 0x56B8B5): a hero flying or water-walking whose tile fails it gets
  a start node without bit 10, so by §3.1 point 6 **its first step cannot visit a neighbour's
  object visited from the neighbouring tile, nor enter a town**, from where it stands inland.
- `hero_move_step` (0x481519): at the move's destination, a flying hero (not in a boat) whose tile
  fails it does not visit an object of such a class (a boat excepted).

Read as it stands, a hero wearing Angel Wings on an inland tile could not pick up a resource pile
beside it, a gameplay effect the wiki would hardly leave out; the reading may miss what makes the
test pass there. **A-14** was withdrawn at the checkpoint (decision 75): "definitely wrong", the
user says, a flying hero visits from its tile as a walking hero does. The engine builds the
walker's visits for a flyer; what the reading misses here (which callers reach these tests, or
what the tile's bit 0x40 holds in memory) stays open for a later re-read, and row E5-8 is dropped.

### 3.4 Where a move may end (D-25, settled)

Read by the second reading (§11): the click's path (`make_hero_path` 0x418DD0 →
`pathfinder_build_path` 0x56A0D0) looks up the destination tile's bit-10 node (level × 2 + 0,
0x56A12E) and needs it reached (bit 0, 0x56A275); without it the path is emptied (0x56A2FF). The
steps before follow each node's predecessor, whose bit 10 the node keeps in its bit 11 (0x4B1B15,
0x56A2D8). So **a move ends only on a tile a hero may stop on**: never over a blocked tile, an
unexplored tile or water (flying), nor on water (water walking); a tile that holds an entered
object or a free land tile can be the end. The wiki ("to an unoccupied map location", "an
unoccupied location on land") and VCMI (a flying hero lands on free tiles; never a turn on water)
agree. Not read: whether the click handler shows a refusal for the empty path (presentation).
Row E5-7 stays as a check.

## 4. The scouting radius and its reveal

### 4.1 The radius (`hero_scouting_radius` 0x4E4070, read in T8a)

`skill_scouting` (0x63E9D8: 5, 6, 7, 8) at the Scouting level (+0xCC); with Scouting and a
Scouting specialty the radius × (1 + 0.05 × level), f32, truncated; +1 for each of the Speculum
(52) and the Spyglass (53), worn or in a worn combination (none holds them). `vcmi`: `SIGHT_RADIUS`
5 + 1/2/3 (`config/skills.json`), +1 each artifact; a specialty would grow the skill's +1–+3 only.
`wiki` (Scouting): basic 1, advanced "3 [HotA] 2 [SoD]", expert "5 [HotA] 3 [SoD]"; (Scouting
radius) "it can be increased up to 10" (SoD: 5 + 3 + 2); "Increases Scouting radius by 1 for every 6
levels", a specialty "introduced" with HotA's heroes Beatrice, Miriam and Wynona. **No SoD hero has
the specialty** (the packs: none; VCMI's heroes: none): the code's path exists, no content reaches
it. **Consensus** on the radius; the specialty: nothing to build (its formula differs in all three
sources).

### 4.2 When a reveal happens

`reveal_disk` (0x49D160) is called with the radius at: the move's each step (the step's
destination, before the hero's position changes; T8a), a placed hero's setup, the new day's
`reveal_owned` (0x4C7C50: around every owned hero), teleports and some objects' visits
(`sub_41D930`'s callers, not read: later increments). A level-up, a skill learned, an artifact
equipped or received reveal nothing (§1.4's list): **a grown radius shows at the hero's next step
or the next day**. `vcmi`: `CGameHandler::changeSecSkill` reveals the hero's radius at once
("Our scouting range may have changed - update it"); an artifact's move reveals nothing (no
caller of `getSightRadius` there): **D-26**, the code against VCMI for a skill, the wiki silent. The engine: ADV-FOG-003's reveal
after each step already uses the radius then; the new day's reveal (0x4C7C50) is not built (it
"adds nothing while no rule hides a tile", T8a) and becomes the place a grown radius shows for a
hero that does not move. Row E5-10.

## 5. The artifacts and specialties of increment 5

| Original id | Content | Effect (the reading) | Pack today | Census (§7) |
|---|---|---|---|---|
| 98 | Boots of Speed | +600 land points (data: `movement.txt` row 3) | `pending` | worn by 2 heroes in 2 maps (one imports whole), 1 in a backpack, 1 on a map |
| 70 | Equestrian's Gloves | +300 land points (row 2) | `pending` | worn by 1 (the Tutorial's), 5 on maps |
| 72 | Angel Wings | flying at level 3 | `pending` | none worn; none on maps |
| 90 | Boots of Levitation | water walking at level 3 | `pending` | worn by 1 hero (a map that does not import whole), 5 on maps |
| 52 | Speculum | radius +1 | `pending` | 4 on maps |
| 53 | Spyglass | radius +1 | `pending` | 1 in a backpack, 4 on maps |
| skill 2 | Logistics | +10/20/30 % of the table's points | skill exists, no effect | 8 heroes and 1 prisoner have it |
| skill 0 | Pathfinding | the table by level | skill exists; costs in the part (T29) | 11 heroes, 3 prisoners |
| skill 3 | Scouting | radius 6/7/8 | skill exists, no effect | 8 heroes, 5 prisoners |
| heroes 23, 85, 106 | Kyrre, Gunnar, Dessa: Logistics specialty | the rate grown 5 % of itself per level (§1.1) | `skill_specialty` (logistics, 500 bp) | Kyrre 2, Gunnar 2 (one imports whole), Dessa 0 |
| 51 heroes | creature specialties | the creature and its upgrade one faster for the slowest speed (§1.1) | `creature_specialty` with `speed = 1` (the battle's) | every map's placed creature specialists |
| creature 142 | Nomads | sand priced as dirt | creature, no ability | no custom army holds them |
| class 21 | Cursed Ground | no native 100 on it | a ground (T20a2a) | 386 objects in 54 maps |

The packs mark the six artifacts `pending: E08-E10`; the hero effects each needs are T34b's
(ADR-0013's effect kinds: a land points bonus, flying and water walking, a radius bonus; the
specialty and the creature specialty exist). Necklace of Swiftness, Ring of the Wayfarer and Cape
of Velocity (creature speed) do not reach the day's points (§1.1).

## 6. Not in increment 5

- **The sea's points**: Navigation (+50/100/150 %, the table 0x698AEC) and its specialists
  (Sylvia, Voy), the Necklace of Ocean Guidance (+1 000), the Sea Captain's Hat and Lighthouses
  (+500 each), the Admiral's Hat (boarding keeps the points proportionally), Favorable Winds:
  boats' increment (decision 71). Boarding clears the Fly and Water Walk spells (`hero_boards_boat`
  0x4A1010) and a hero in a boat neither flies nor water-walks.
- **Stables** (+400 for the week, hero flag +0x105 bit 1): with the weekly objects (the sketch's
  increment 7).
- **The spells** Fly, Water Walk, Dimension Door, and their levels 0 to 2 (the price tables'
  other entries, D-24's table at 140 and 120): with the adventure spells. The pathfinder's
  computer mode 4 (casting them on the way) with E10.
- **The computer players' bonus** (+75, +50; §1.1 point 7, players 0 to 5 only: a quirk candidate
  for E10) and the cheat flag (+0x105 bit 24).

## 7. The census

Counted on 2026-10-21 over the 160 shipped maps with `h3-map`'s reader. Heroes (class 34) and
prisoners (62) with the hero object's own skills and artifacts, else the map's custom hero's (SoD),
else the hero type's (random heroes, class 70, not counted).

| What | Count | Maps (per kind) |
|---|---|---|
| a hero with Logistics: Basic, Expert; a prisoner with Advanced | 7, 1; 1 | 7, 1; 1 |
| with Pathfinding: Basic, Advanced, Expert; prisoners Advanced | 7, 3, 1; 3 | 7, 3, 1; 3 |
| with Scouting: Basic, Advanced; prisoners Basic, Advanced | 5, 3; 4, 1 | 5, 3; 4, 1 |
| a hero wearing Boots of Speed, Equestrian's Gloves, Boots of Levitation | 2, 1, 1 | 2, 1, 1 |
| Boots of Speed, Spyglass in a hero's backpack | 1, 1 | 1, 1 |
| a hero wearing Angel Wings, the Speculum or the Spyglass | 0 | 0 |
| artifacts on the map (class 5): Equestrian's Gloves, Boots of Levitation, Speculum, Spyglass, Necklace of Ocean Guidance, Boots of Speed | 5, 5, 4, 4, 3, 1 | 5, 2, 4, 2, 2, 1 |
| placed Logistics specialists: Kyrre, Gunnar (Dessa none) | 2, 2 | 2, 2 |
| a custom army with Nomads | 0 | 0 |
| Cursed Ground objects (class 21) | 386 | 54 |
| maps with water | — | 131 |

**The 5 maps that import whole** (decision 22's list since T29): `Tutorial.tut` (its hero:
Expert Logistics and Pathfinding, Boots of Speed and Equestrian's Gloves worn; 4 Cursed Ground
objects), `Adventures of Jared Haret` (Gunnar with Basic Logistics and Scouting), `Faeries`
(Pathfinding, Scouting; 3 Cursed Ground objects), `Race for Ardintinny` (Pathfinding, Scouting,
Boots of Speed in a backpack; 1 Cursed Ground object), `Knee Deep in the Dead` (Advanced
Pathfinding, the Spyglass in a backpack). Increment 5 changes their play wherever these heroes
move; their import is unchanged (`MAP_IMPORT_VERSION` stays unless the import gains a field).

## 8. Disputes and quirk candidates

| Id | What | Sources | Suggested |
|---|---|---|---|
| D-22 | a Logistics specialist's rate: single precision against exact or integer arithmetic | code (f32, Q-0005's kind), VCMI (integer percent), wiki (silent on rounding) | **decided**: Q-0005's flag (faithful the code, strict-fixed exact); row E5-2 |
| D-23 | Cursed Ground takes away the native terrain's 100 | code; VCMI none; wiki silent | **decided**: built only if row E5-6 confirms it |
| D-24 | flying's price: the fly price from inland tiles (roads 100), the lesser only from the coast | code against the wiki and VCMI (the lesser everywhere) | **decided**: the original's rule, a fix-optional quirk (strict-fixed the lesser everywhere); row E5-4 confirms |
| D-25 | where a flying or water-walking move may end | code (the second reading), wiki and VCMI: a tile a hero may stop on | settled; row E5-7 a check |
| D-26 | a radius grown in the day shows at the next step or day | code; VCMI at once for a skill; wiki silent | **decided**: the code, the new day's reveal built; row E5-10 |
| A-12 | the drop to 0 prices the cheapest step without the artifacts' flying and water walking | code | **decided**: fix-optional (it matters only on Cursed Ground, D-23) |
| A-13 | a flying path crosses unexplored tiles | code; VCMI never | **decided**: fixed (moves on explored tiles); a trade-off for wide obstacle ranges (routing to the nearest unexplored but free tile) to design later; row E5-9 shows the original |
| A-14 | a flying hero standing inland neither visits a neighbour's "cannot be entered" object nor enters a town from its tile | code, unconfirmed | **withdrawn** (the user: a flyer visits as a walker) |
| — | players 6 and 7 get no computer bonus | code | E10's |

## 9. Batch E5's plan

Increment 5's batch, **batch E5**, runs on the client lane's harness in SoD on Easy, hot seat, from
a map `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 and E4. Its spec and
recipe are T34b's cut's tasks; this is the plan. Each row names what it settles and what it records.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E5-1 | ADV-MOV-001's bonuses (consensus, a check) | heroes of speed-4 armies (1560): no Logistics; Basic, Advanced, Expert; Boots of Speed; Equestrian's Gloves; both with Expert Logistics; a creature specialist with its creature as the slowest stack | each hero's points at the turn start of day 2 | 1560, 1716, 1872, 2028; 2160; 1860; 2928; the next table entry |
| E5-2 | D-22 | Gunnar and Kyrre with Basic Logistics and a speed-3 army at levels 2, 6 and 12 (where the code and exact arithmetic part, §1.3), one at level 3 (where they agree) | points at day 2's turn start | 1664, 1694, 1739 (exact: one more each); 1672 |
| E5-3 | §1.4 (consensus, a check) | a hero without Logistics one battle short of a level with Logistics offered (a level-up answered with it), then a step; a second hero receives Boots of Speed from a pile (if the spec can place one; else a skip) | points before and after the battle and the step; at day 2 | today's points unchanged by the gain; day 2 with the bonus |
| E5-4 | D-24 | an Angel Wings hero on an inland cobblestone road, one straight step along it; on grass; on swamp; from a coastal tile along a road | points before and after each step | 100 each inland (faithful; the wiki, VCMI and strict-fixed: 50, 100, 100); the coast's 50 |
| E5-5 | §2.1 water walking (consensus, a check) | a Boots of Levitation hero: land → water, water → water, water → land, a diagonal on water | points after each step | the land tile's price, 100, 100, 141 |
| E5-6 | D-23 | a Fortress army (native swamp) on swamp: a straight step from plain swamp and one from swamp under Cursed Ground | points after each step | 100 off it, 175 from it (VCMI 100) |
| E5-7 | D-25 (a check) | an Angel Wings hero clicks a tree beyond a tree line; a Boots of Levitation hero clicks a water tile | the mark (green, red, none); arrived; the hero's tile | no path to either (§3.4) |
| E5-8 | — | dropped at the checkpoint (A-14 withdrawn) | — | — |
| E5-9 | A-13 | an Angel Wings hero facing a strip of unexplored land between it and an explored tile | the target's mark at the first click | the reading: a green path through the strip |
| E5-10 | D-26, ADV-FOG-003 | a hero that gains Scouting at a level-up (battle), then no step; a second hero with the Spyglass in the backpack (if the harness can equip it; else a skip) | the explored tiles around both after the battle and at day 2 | nothing new after the battle; at day 2 the radius 6 around the hero |

Not planned: the Speculum and the Spyglass worn (consensus: radius +1 each, a fixed test), Nomads
(consensus), the specialty's radius (no SoD content).

**What the harness lacks** (the client lane's inbox with T34b's batch task, or this lane builds it,
`cross-lane`, as in T32b1): the explored bit of a tile for a player (`explored:x,y`, the tile's vision
word against the player's bit: E5-9, E5-10); the hero's points at a turn start without a walk (a read
step's `points` may already give it: check); a step that ends the turn and reads at the next turn's
start (`end_turn` exists). **What the spec lacks** (`h3-experiment`, battle-owned: `cross-lane`):
special grounds under heroes (Cursed Ground on a hero's tile, E5-6: the writer places grounds since
E04c T3), unexplored strips (the spec's map is explored around the heroes only: a hero far enough
away), artifacts worn by placed heroes (`equipment`: since batch E4), a level-up offer the test can
force (E5-3, E5-10: the seed byte's prediction as in batch E4, `e4_score`'s draws).

## 10. The proposed rule ids

For T34b's cut; ids follow the catalogs' prefixes (`HER-` `heroes.toml`, `ADV-` `adventure.toml`),
each written when its rule is built.

| Id | Rule | Survey | Open points |
|---|---|---|---|
| HER-ADV-003 | A hero's land points bonus: the Logistics rate (with a Logistics specialty's growth, single precision under Q-0005) and the flat bonuses of its worn artifacts | §1.1, §1.3 | D-22 |
| HER-ADV-004 | A hero's slowest speed for the day's points: its creature specialty's creature and upgrade one faster | §1.1 | — |
| HER-ADV-005 | A hero's flying and water-walking levels from its worn artifacts (3 or none) | §2.1 | the spells' levels later |
| HER-ADV-006 | A hero's scouting radius: the skill's, +1 per radius artifact | §4.1 | the specialty (no content) |
| ADV-MOV-001 (amended) | the day's land points: the table at the slowest speed (HER-ADV-004) × (1 + the rate) + the flat bonuses | §1.1 | — |
| ADV-MOV-002 (amended) | the points are set at the owner's turn start only; a gain in the day changes nothing until then | §1.4 | — |
| ADV-MOV-003 (amended) | the step's price by the hero's Pathfinding level; Cursed Ground; Nomads | §2.1, §2.2 | D-23 |
| ADV-MOV-009 | flying: the price (§2.1 point 5), the passage (§3.1) and the end of a move (§3.4) | §2.1, §3.1, §3.4 | D-24, A-13, A-14 |
| ADV-MOV-010 | water walking: the price, the passage and the end of a move | §2.1, §3.1, §3.4 | — |
| ADV-MOV-006 (amended) | the drop to 0 after a step, with A-12's flag | §2.5 | A-12 |
| ADV-FOG-001, -003 (amended) | the radius from HER-ADV-006; a grown radius shows at the next step or the new day's reveal | §4 | D-26 |

## 11. The second reading

A fresh agent re-read the deciding code without this survey (2026-10-21), from nine questions,
and **confirmed §1 to §3 point by point**: 0x4E4990's order (the cheat flag, the slowest speed with
the specialty's creature and upgrade, the table, the rate grown on the x87 and rounded once to f32
at 0x4E4CBB, the truncated product, then the boots, the gloves and the Stables) and the computer's
bonus with its players 0 to 5 (§1.1); 0x4B1740 with flying at level 3 (100 inland on a road, the
lesser 50 from the coast, 100 on water), water walking by the source tile, the diagonal's bound
from the terrain's price after Nomads (§2.1); `hero_flight_tile_test`'s three conditions (§3.3);
the start node's bits and the guard test skipped for flying nodes (§3.1); the expansion's table for
blocked, unexplored, water and rock tiles by walking, flying and water walking, every new node
starting with bit 10 and losing it on those tiles, a new day clearing bits 4 and 5, no stop entry
from a node without bit 10 (§3.1); the move passing the raw spell fields to the drop's cheapest step
(A-12, §2.5); and that the level-up, the skill learned and the artifacts' changes neither reprice
nor reveal (§1.4).

**What it adds:** where a move may end (§3.4, D-25 settled), the two nodes per tile, the flyer's
visit only at its destination and the walking hero's boat node (§3.1). Not traced by it: the move's
class switch after 0x48152B, which can also drop the visit (0x48167B), and whether the click
handler refuses an empty path on screen.

## 12. For T34b

The checkpoint (ADV-Q20) is answered (the epic's decision 75; §8). T34b drafts ADR-0013's effect
kinds and properties (the land bonus, the speed, the flying and water-walking levels, the radius),
ADR-0016's or -0017's amendment where the points, the passage, the end of a move and the reveal
change (the reach's two kinds of node, the new day's reveal), the quirks (D-24's flying price and
A-12 fix-optional, A-13 fixed, Q-0005 for D-22), and the design note's increment 5 with the cut;
D-23 waits for row E5-6.
