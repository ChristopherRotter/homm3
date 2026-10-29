# E08 design note: increment 8, boats and the sea

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 8's
amendment drafts of ADR-0013 (points 3, 13), ADR-0016 (points 1, 3, 4, 6) and ADR-0017 (points 1,
8, 10, 11) in `docs/lanes/adventure/adr-drafts/*-increment-8.md`, written by T66b from T66a's
survey and the answers at ADV-Q32 (the epic's decisions 162–168), accepted as written with the
cut at ADV-Q33 (decisions 170–173). The task that builds a section deletes it; the file goes
when the epic closes. Facts and sources: survey `docs/rules/surveys/E08-sea.md` (§ numbers are the
survey's unless they name this note); the proposed rule ids are the survey's §12.

Increment 7's note is in `E08-log.md` ("Increment 7's design note at its closure", moved by T65).

Written whole (decision 169): **part A**, the hero side (`h3-heroes`, the core content); **part
B**, the map (`h3-map`, `h3-adventure`); **part C**, the game, the env, batch E8 and the cut.
*Agent's call (change freely)* marks a choice the user accepted at the checkpoint (decision 173)
and may still change.

## Part A — the hero side (`h3-heroes`, the core content)

### 1. The data (the core content and packs: shared)

- **Three effect kinds** in `h3_content::HeroEffect` (ADR-0013 point 3; the parser in `h3-pack`):
  `navigation { level }` (1 to 3, as `pathfinding`'s), `sea_movement_points { points }` (from
  1) and `free_boarding` (no parameter). None reaches `BattleEffect`.
- **The packs** (`packs/base/core.toml`, `packs/sod/core.toml`): Navigation's basic, advanced
  and expert levels `navigation` 1, 2, 3; the Necklace of Ocean Guidance (71)
  `sea_movement_points` 1 000 and the Sea Captain's Hat (123) 500 beside its two spells, their
  `pending: E08-E10` marks dropped; the Admiral's Hat (136, SoD) `free_boarding`, its points by its
  parts. The Navigation specialists keep their `skill_specialty` (`bp_per_level` 500, Sylvia and
  Voy). Sources in the packs' comments: survey §4.2 and `E08-hero-movement.md` §1.1
  (`movement.txt`'s sea column, 0x698AEC, 0x698B04, 0x698B0C), the wiki (Navigation, Necklace of
  Ocean Guidance, Sea Captain's Hat, Admiral's Hat), VCMI (`config/skills.json`,
  `movementPointsSea`, `navigationSpecialty`, `FREE_SHIP_BOARDING`).
- The core ruleset hash and the pack format move (the `shared/` task, §18).

### 2. The day's sea points and free boarding (HER-ADV-009)

- `h3_heroes::sea_points(details, core, table: &[u32; 4]) -> u32`: `table` the adventure part's
  sea points by Navigation level (§5); the entry at the highest `navigation` level (0 without);
  with a level from a skill whose specialty the hero has, ⌊hero level × table[0] × `bp_per_level`
  ÷ 10 000⌋ added (75 a level: the original's ⌊level × 1 500 ÷ 20⌋); then each worn artifact
  type's `sea_movement_points` once (worn or a part of a worn combination: `Count::OncePerType`),
  saturating. Exact integers in both profiles.
- `h3_heroes::free_boarding(details, core) -> bool`: a worn type with `free_boarding` (the
  combination itself, not its parts worn apart: the original tests 136).
- Tests (HER-ADV-009): 1 500 without Navigation; 2 250, 3 000, 3 750; Sylvia (a Navigation
  specialist) at level 10 with Basic Navigation 3 000 (2 250 + 750), without Navigation 1 500;
  Advanced Navigation and the Necklace 4 000, the Sea Captain's Hat alone 2 000 (batch E8's
  E8-10); the Admiral's Hat 3 000 and free boarding, its two parts worn apart 3 000 and none.
- Not here: Lighthouses (+500 each, item 9) and the computer's bonus (E10): inputs then.

### 3. Rule ids, quirks, shared needs (the hero side)

HER-ADV-009 (the day's sea points and free boarding). No quirk. No hero change, no new
`HeroDetails` field: the details' hash keeps its version; a hero's boat, points and day's maximum
are the adventure's (§6).

## Part B — the map (`h3-map`, `h3-adventure`)

### 4. The import (`h3-map`; map import v14)

| Class | Becomes | Body |
|---|---|---|
| 8 boat | `AdventureSetup::boats`: a `BoatSetup { tile, kind }`, the kind the template's subclass (0 to 2), facing 2, no owner, nobody aboard | none (`load_object_body` case 8 reads nothing) |
| 29 Flotsam, 82 Sea Chest, 86 Shipwreck Survivor | typed objects `base:flotsam`, `base:sea_chest`, `base:shipwreck_survivor` | none |
| 59 Ocean Bottle | `base:ocean_bottle` | the message read and dropped (the client reads the map's text) |
| 124 hole | `base:hole`, lying on its picture's tiles | none |
| 225 Favorable Winds | `sod:favorable_winds`, lying on its picture's tiles (water only) | none |

- **The pictures' sizes** (codes.rs's table of the special grounds' `.msk`, ADV-GRD-001) gain the
  eight holes' (`AVLhlds0`, `AVLhlsn0`, `AVLhold0`, `AVLholg0`, `AVLholl0`, `AVLholr0`,
  `AVLhols0`, `AVLholx0`: passable everywhere, no visitable tile, land only, `Objects.txt`) and the
  eight Favorable Winds pictures (`AVXfw0` to `AVXfw7`: water only); read from the sprite
  archives by T67 (`H3sprite.lod`, as T20a2a read the grounds'): every hole 1 × 1, the winds 6 × 4,
  8 × 6, 6 × 6, 8 × 3, 3 × 6, 3 × 3, 5 × 2, 2 × 5 in that order (§19).
- **D-44** (decisions 163, 178: kept after E8-5): a hero object (34) on a water tile is `Unsupported::HeroOnWater(index)`
  (counted by reason in `examples/original_data.rs`: none of the 160 shipped maps, §1).
- **The boats' bound** (decision 164): the setup refuses a boat off water, two boats on one tile,
  a boat on a tile an object blocks or is visited from, or under a hero (`SetupError::Boat`); no
  count limit beyond one per water tile. The build's census confirms that no shipped map trips it
  (the census of §1 counted every boat on water, one visitable tile each; a boat sharing a tile
  with another object was not counted).
- The eight maps that import whole stay eight; Race for Ardintinny and Faeries gain their boats
  and every sea pickup typed (§1). Version: `MAP_IMPORT_VERSION` 14.

### 5. The kinds and the part (`h3-adventure`, `h3-pack-adventure`; adventure ruleset part v13)

- **The sea's three finds** (ADV-PIK-014 to -016) extend `Find` (increment 7's kind):
  - a content of **fixed resources with an optional artifact** beside them (Flotsam's bundles:
    nothing, 5 wood, 5 wood and 200 gold, 10 wood and 500 gold at 25 % each; the Sea Chest's 20 %
    nothing, 70 % 1 500 gold, 10 % 1 000 gold and a treasure artifact);
  - a full backpack's **turn replaces the whole entry**: the chest's `Give(1 500 gold)` instead of
    both its artifact and its 1 000 gold (survey §8.3); a new turn **`Lost`** (the Survivor's: the
    artifact lost, the object leaves anyway, §8.4);
  - **`leaves`**: the find leaves the map at its visit (the four sea pickups), not emptied in
    place (the land's finds).
  *Agent's call:* the find extended, not three new kinds: the draw at the build, the backpack's
  test and the turn are the find's (increment 7), only the content's shape and the leaving are new.
- **`Message { leaves }`** (ADV-PIK-017): its visit tells the visitor's owner which object's
  message it read (`MessageRead`); the Ocean Bottle leaves; the Sign (91, a later increment) will
  not. Every visitor reads it (A-27 not reproduced, decision 166).
- **`SeaGround`** (Favorable Winds, part of ADV-SEA-004): lies on its picture's tiles, never
  visited, no battlefield; the static map marks the tiles it tops (`StaticTile::winds`, as
  `no_native` for Cursed Ground: the tile's top ground decides). The price factor is the part's
  parameter `sea_ground = [2, 3]` (two thirds, truncated; survey §4.1 point 1). *Agent's call:* one
  factor in the part's parameters, the kind without parameters, as long as one such ground exists.
- **`Hole`** (part of ADV-SEA-003): lies on its picture's tiles, never visited; forbids a landing
  on them (`StaticTile::hole`). It blocks nothing else (`Objects.txt`: passable everywhere).
- **Object types** (`packs/base/adventure.toml`, `packs/sod/adventure.toml`): `flotsam`,
  `sea_chest`, `shipwreck_survivor`, `ocean_bottle` (visited from the neighbouring tile and from
  above: classes 29, 59, 82, 86 are on both of the original's lists, 0x63AA58 and 0x63AB00),
  `hole`, `favorable_winds` (entered, from above, as the grounds).
- **Parameters:** `sea_points = [1500, 2250, 3000, 3750]` (`movement.txt`'s sea column, the
  original's 0x698AEC); `sea_ground = [2, 3]`; `[params.battle]` gains `sea = "water"`, `deck =
  "deck"`, `boats = "boats"` (checked against the battle part when the game bundles the parts).

### 6. The state (`h3-adventure`; adventure state hash v16, game state hash v20)

- **`Boats`** (ADR-0016 point 1; ADV-SEA-001): `records: persistent::Table<Boat>` by boat index
  (the setup's order) and `empty: BitGrid`, the tiles an empty boat lies on; none for a map without
  boats, so today's forks keep their cost (`bench-check`). `Boat { tile, kind: u8, facing: u8,
  owner: Option<PlayerId>, captain: Option<HeroId>, aboard: bool, gone: bool }`: `tile` is where an
  empty boat lies (while aboard the hero's tile is the boat's); `facing` the original's 0 to 7 (a
  map's boat 2; a landing's the step's direction in the original's order: the mapping from
  `DIRECTIONS` is the build's, with a test against E8-15's record); `gone` once its hero left the
  map (the record stays, so indexes never move).
- **`HeroOnMap`** gains `boat: Option<u32>` (the boat aboard) and `max_movement: u32` (the day's
  maximum, ADR-0016 point 3), set by `start_game`, `start_turn`, a boarding and a landing.
- **Invariants** (`check_invariants` in the properties, and the setup's checks): every empty boat's
  tile is set in `empty` and no other tile is; a boat aboard has exactly one hero holding it, on a
  water tile, and no empty boat lies there; a hero on water is in a boat; no two boats on a tile.
- **The hash:** the boats in index order (tile, kind, facing, owner, captain, aboard, gone), each
  hero's boat and maximum: adventure state hash v16; the game state hash v20 (it hashes the
  adventure's).

### 7. Moving: the modes and the passage (ADV-MOV-007 amended; ADV-SEA-002 to -005, ADV-PIK-018)

`Mode::Sail` for a hero in a boat, before flying and water walking (a hero in a boat neither flies
nor water-walks, whatever it wears; survey §4.1). `Meets` gains **`Board(u32)`** and **`Land`**:
both end the move (a longer path is `PastVisit`), and the reachable-tiles query treats both as
leaves. The rows below extend increment 5's passage table (`state/passage.rs`; the walking hero's
in `check_step` and the walking search):

| Onto | Walking (on land) | Flying | Water walking | In a boat |
|---|---|---|---|---|
| an empty boat's tile | boards (a stop; no guard test there) | boards from a stop node; a flight over water passes it | boards from a stop node; never from water | never |
| a hero in a boat | visit from the tile (battle, meeting: the deck) | visit from a stop node; passes | visit from a stop node | visit from the neighbouring water tile (the two boats) |
| an object on water visited from the neighbour (the sea's pickups, a monster) | never | **never, and no pass** (D-45 reverted by E8-7; D-46, batch E8c) | never | visit from the neighbouring water tile |
| an entered object on water (a water Cartographer) | never | **no stop, no pass** (D-46, batch E8c; today it passes, ADV-MOV-009) | never | entered |
| plain water | never | passes | passes | a stop |
| a blocked water tile | never | passes | never | never |
| a free coastal tile | as land | as land | as land | **lands** (a stop, a leaf; no guard test at the step) |
| any other land tile (an object's, an event's, a hole's, a hero's, blocked) | as land | as land | as land | never |

- **The diagonal at sea:** water to water only when both side tiles are water (0x4B2703, 0x481D33);
  a diagonal landing has no side test (T66b's reading: the land branch 0x4B2867 and the move's
  0x481CDB test the sides only for a water destination); a water walker's diagonal has none either
  (unchanged).
- **The free coastal tile** (D-42, decision 162; the original's class 3): the map's coast flag
  (`StaticMap::is_coast`), and as it stands now no object blocking it or visited from it (the live
  occupancy), no event on it still on the map (`events_at`), no hole on it, no hero. A tile an
  object left becomes one again (0x5056A5).
- **Guards:** a boat's step onto a water tile a monster guards (water guards water, ADV-MON-005) is
  attacked at once; boarding tests no guard (0x480108); a landing tests none at the step, and the
  landing tile's guard visits after the landing (`guard_after`), at the monster's tile (§13).
- **Events on water** (ADV-OBJ-007, ADV-MOV-011 amended): a boat's step fires them as a walker's
  (`firing_event`) and ends the move there; their guards fight on the deck. `Reach` stays blind to
  them (decision 131's rule).

### 8. Prices, the drop and the conversion (ADV-SEA-002 to -004; D-41, D-43, A-28)

- **A boat's step:** water's straight price from the part (`Passage::Water`, 100) from the source
  tile; on a `winds` tile × 2 ÷ 3, truncated (66); the diagonal ⌊straight × √2⌋ (141, 93);
  neither roads, nor native terrain, nor Pathfinding, nor the lifts. **The diagonal exception**
  (ADV-MOV-005) compares the points left with water's price (100), never the winds' (D-43): on a
  winds tile with 66 to 92 points the diagonal (93) is refused, with 93 to 99 it costs 93, with 100
  to 141 the straight 66. `Pricing::sail` (one price function, no `LIFTED` branch).
- **The drop** after a step (ADV-MOV-006's rule): the cheapest step from the new tile as a boat
  prices it when the step ends at sea or boards (100; 66 on a winds tile), as a walker without
  lifts when it lands (the coast tile's road price, else the native price; Cursed Ground's
  terrain price under it), in both profiles: the hero is in a boat at a landing's step and is one
  after a boarding's (`ADV-QUIRK-17` concerns a walking hero's lifts and does not apply).
- **Boarding's step** is priced by the hero's mode on land (a walker's, a flyer's or a water
  walker's price from its land tile); **a landing's** by the boat's price from the water tile.
- **The conversion at the visit** (D-41, decision 162): the new maximum computed then (the sea
  points of §2 at a boarding; the land points, `day_points`, at a landing); with free boarding
  the points become ⌊points × new maximum ÷ old maximum⌋ (a 64-bit product; an old maximum of 0
  gives 0), the points being those after the step's drop; without it 0; the maximum becomes the
  new one in both cases (A-28, `ADV-QUIRK-31`, not reproduced: decision 166). E8-1 and E8-2
  confirm or revert the drop before the proportion (VCMI: (left − price) × new ÷ old).

### 9. The reachable-tiles query (`Reach`; ADV-MOV-007 amended)

- **A hero in a boat:** a third search beside the walking and the two-layer ones: one node per tile
  (every water tile a stop), Dijkstra by points left as the walker's, with §7's sail rows and §8's
  prices; a landing tile, a visited tile (an object or a hero) and a guarded water tile are
  destinations it goes no further from; the start in a boat. Its paths are ones `MoveHero` accepts
  (the property extended).
- **A walking hero:** a water neighbour (the terrain test fails) is tested for an empty boat
  (`Boats::empty`, one bit) and a hero, out of the hot loop (`#[cold]`, as `guarded`), so
  `reach-xl-144` stays within its gate; a boarding tile is a destination, a leaf.
- **A flying or water-walking hero:** §7's rows in `passage` (the boarding stop, the hero in a
  boat; D-46's water entrances closed to a flight).
- **Destinations** (the env's mask, `legal_moves`): the boarding and landing tiles, the visits at
  sea, every water tile a boat reaches. Today's reach only (the sea's next day is not the
  engine's, survey §9).

### 10. Visits at sea (ADV-SEA-005)

- From a boat, an object visited from the neighbouring tile is visited from the boat's water tile
  (the step's price paid, the hero stays; ADV-OBJ-003) and an entered one is entered (the boat's
  hero stands on its tile in its boat); another hero in a boat is attacked or met from the
  neighbouring water tile (GAM-BAT-004, GAM-MET-001); a boat never reaches a hero on land.
- The kinds already built become reachable by boat (§1's counts: 55 monsters on water, 57 water
  Cartographers, a few artifacts, resources and a chest on water); the creature generators and the
  Eye of the Magi on water stay blockers (later increments).

### 11. The sea's pickups (ADV-PIK-014 to -018)

- **The draws** when the game is built (`state/draws.rs`, ADR-0016 point 4), from the setup stream
  in object order with increment 7's finds: Flotsam one of its four entries, the Sea Chest's and
  the Survivor's 0–99 by their chances, their artifacts from increment 7's pool of the random
  artifacts (D-32: never one banned, placed or drawn before until the pool is exhausted). One draw
  of 0–99 each, not the original's `random_between(0, 3)` for Flotsam: the streams are the
  engine's, the chances the original's.
- **The visit:** the drawn content to the visitor (the resources to its owner, the artifact
  through `can_receive_artifact` and `HeroGains`); a hero that cannot receive it (`ADV-QUIRK-22`'s
  count under `faithful`, the placement first under `strict-fixed`) gets the chest's 1 500 gold
  instead of the 1 000 and the artifact, and nothing of the Survivor's; then the object leaves
  (`ObjectRemoved` where seen). The bottle: `MessageRead { hero, object }` to the visitor's owner
  alone, then it leaves. No question.
- **Who takes them** (ADV-PIK-018): a boat from the neighbouring water tile only; never a flyer, a
  walker or a water walker (§7; D-45 reverted by E8-7, survey §15).

### 12. Information (ADV-FOG-006 amended, ADV-FOG-008)

- **Views:** `AdventureView::boats` (the empty boats on explored tiles, in tile order: tile, kind,
  facing; never owner or captain); `HeroView::boat` (its boat's kind, public: the original draws
  the hero's flag on it); `OwnHeroView::max_movement` (the day's maximum, its owner's).
- **Events** (each `seen_by` the viewers of its tiles): `Boarded { hero, boat, at }` (the boat
  leaves the tile's view, the hero steps onto it), `Landed { hero, boat, from, to, facing }` (the
  boat left on `from`), `BoatRemoved { boat, at }` beside `HeroRemoved`, `MessageRead { hero,
  object }` (the visitor's owner only). A hero's points after a conversion are its owner's
  (`OwnHeroView`).
- **The fog:** unchanged (the scouting radius has no boat term, §4.4); `docs/06-security.md`'s
  viewer knowledge list gains the boats (the adventure epic's).

## Part C — the game, the env, batch E8 and the cut

### 13. The battle at sea (GAM-BAT-001, -003 amended; ADV-SEA-006)

- `battle_at` passes the two sides' boats: either hero in a boat → (`sea`, `deck`), both →
  (`sea`, `boats`), before the underground; then a special ground on the tile replaces the
  battlefield as on land (none lies on water in the shipped maps). A monster or an event's guards
  against a boat's hero: the deck. A landing's guards: the monster's tile's battlefield, as any
  guards' (E8-15: the shore only when that tile is coastal). No hero but a boat's reaches a
  monster on water (D-45 reverted), so no battle on water lacks a boat. The
  background (GAM-BAT-011) and the attacker's ground (GAM-BAT-013) unchanged. The battle lane's
  `deck` and `boats` battlefields exist (BAT-TER-002, BAT-GEO-004): no battle change, no seam
  change.
- **After the battle:** a hero in a boat that lost, retreated or surrendered, or both after a draw,
  leaves the map with its boat (`gone`, `BoatRemoved` where seen); a winner in a boat stays in it.

### 14. The game (`h3-game`)

- The turn start and the start of the game set the points and the maximum by the surface
  (`day_points` gains the sea's branch: `sea_points` with the part's table).
- No command, decision or answer is new; the outcomes' shapes are increment 7's; the new events
  are projected as §12 says; `GameObservation` carries the adventure's views. The adventure replay
  format keeps v7.

### 15. The env, replays, fixtures, properties, benchmarks

- **The env** (the AI lane's, ADR-0017 point 8): the observation encoding gains the empty boats on
  explored tiles and each seen hero's in-boat bit (an encoding version bump); the action encoding
  keeps v5 (a boarding or a landing is a tile). `cross-lane` in T74 with the user's consent
  (decision 172), as T63a's; the AI lane's inbox told (T66b).
- **The sea fixture** `fixtures/maps/e08-sea-36.h3m` (written by `h3_map::writer` in
  `tests/sea_map.rs`): two players; boats of the three kinds beside coasts; the four pickups (a
  hero with a full backpack); events on water with and without guards; a monster on water;
  Favorable Winds; a hole and a guarded tile on a coast; heroes with Navigation, the Necklace, the
  Sea Captain's Hat, the Admiral's Hat, Angel Wings beside a coastal Flotsam, Boots of Levitation.
- **Golden replays:** random play on the sea map (three seeds); scripted: boarding and landing with
  and without the Hat, a battle on the deck from the shore and against a monster on water, two
  boats, a landing into guards, an event on water, a flyer refused beside a pickup, a boat leaving with its beaten
  hero. Every golden re-recorded once (the state hash versions).
- **Properties** (`h3-map`'s `play.rs`, `view.rs`): generated maps with water regions and boats
  beside their coasts; `check_boats` (§6's invariants after every command); `Reach`'s paths
  accepted at sea; the conversion's bounds (without the Hat 0; with it at most the new maximum);
  no path of a hero not in a boat ends on a water entrance or, for a flyer, crosses one (D-46 as
  batch E8c leaves it); fixed cases on the sea map.
- **Benchmarks:** `reach-sea-xl-144` (a boat's search; a new XL fixture `e08-sea-144.h3m`, mostly
  sea, 15 boats as Myth and Legend's), `random-play-sea-36`, `fork-sea-xl-144`; the ten workloads
  at their baselines (`reach-xl-144` the walker's cold boat test).
- **`--with-original-data`:** the shipped maps import with boats and the pickups typed; the census
  by reason gains `HeroOnWater` (0) and the boats' refusals (0 expected).

### 16. Versions, rule ids, quirks

| Version | Change | Task |
|---|---|---|
| core ruleset hash, pack format | the three effect kinds; Navigation and the three artifacts | T70 (`shared/`) |
| adventure ruleset part v13 | the find's extension, `Message`, `SeaGround`, `Hole`; the sea's parameters | T72 |
| map import v14 | classes 8, 29, 59, 82, 86, 124, 225; `HeroOnWater` | T72 |
| adventure state hash v16, game state hash v20 | the boats, a hero's boat and maximum | T72 |
| env observation encoding (the AI lane's) | the boats, the in-boat bit | T74 (`cross-lane`) |

Rule ids: survey §12 (ADV-SEA-001 to -007, ADV-PIK-014 to -018, HER-ADV-009 and the amendments).
Quirks, recorded in `docs/rules/quirks-adventure.md` without flags (decisions 164, 166):
`ADV-QUIRK-30` A-27 (a computer's hero leaves a bottle), `ADV-QUIRK-31` A-28 (the day's maximum
kept without the Hat), `ADV-QUIRK-32` the 64 boats (a 65th dropped at load); all not reproduced.

### 17. Batch E8

Survey §11's 16 rows in two runs (decision 165), on the harness (`clients/capture/`, common) in
SoD on Easy, hot seat, from maps `h3-experiment` writes: **E8a** movement and the shore (E8-1 to
E8-5, E8-7, E8-10, E8-15, E8-16), **E8b** the pickups and the battles (E8-6, E8-8, E8-9, E8-11 to
E8-14).

- **The writer and the specs:** built by T67 (`E08-log.md`, T67): the writer's boat, pickups,
  holes and bottle text, the spec's `[[map.bottle]]` and a monster's own `at`, the specs
  `E08-batch-8a`, `-8a-water` (E8-5 on its own map) and `-8b` with their templates; the
  predictions correct two of survey §11's rows (E8-4: the drop to 0 after the straight winds step;
  E8-13: no path goes on past a guarded tile).
- **The harness** (common; CLT-Q18: this lane builds it with its check and tells the client and
  map-client lanes' inboxes): reads of the boats' records (game +0x4E3BC, `structs.toml` "boat"),
  a hero's in-boat bit (+0x105 bit 18) and day's maximum (+0x49), the battle's ship flag
  (`battle_manager` +0x53C6), terrain, special field and blocked hexes at the opening, an object's
  word; clicks at a coastal tile from a boat and at a boat from land; an arrow key toward a tile
  (E8-7).
- **The scoring:** survey §15 (new), each row against §11's prediction; a dispute a row contradicts
  reverts to the other side (decision 162) or becomes an `ADV-Q`; the consensus rows' sources
  `verified`. E8-5 decides whether D-44's refusal stays (decision 163). **Done** (T69, survey §15;
  decisions 177–179): D-41 to D-43 confirmed, D-44's refusal kept, D-45 reverted, E8-15's battle
  at the monster's tile; the re-read's **D-46** (a flight over a water tile holding an object)
  goes to a one-row **batch E8c** (T69b) before T70.

### 18. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T70's `shared/` task (the three ADRs applied with batch E8's outcome;
the three effect kinds in `h3-content` and `h3-pack` with the core packs; `docs/03-domain.md`'s
terms: boat, in a boat, boarding, landing, free coastal tile, ground of the sea); batch E8's spec
(`h3-experiment`, `cross-lane`); the harness (common); the env's observation encoding (`h3-sim`,
`cross-lane`, T74). Everything else is the lane's own.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut confirmed
at its start). **Batch E8 runs before the build and before the shared task** (decision 165, as
increment 7's decision 139), so the ADRs are applied once with the disputes' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T67** | Batch E8's maps: the writer's boat, pickups, hole, winds and a hero and monster on water (§17); the spec's fields (`cross-lane`); the two specs and templates |
| 3 | **T68** + **T69** (+ **T69b**) | T68: the harness's reads and clicks, the recipes, the runs E8a and E8b. T69: scored (survey §15); D-41 to D-45 settled. T69b: batch E8c, D-46 (decision 179) |
| 4 | **T70** | `shared/`: the three ADRs amended with T69's outcome, the three effect kinds and the packs, the glossary |
| 5 | **T71** + **T72** | T71: `h3-heroes` (§2; small). T72: `h3-map` and `h3-adventure`: the import (§4), the kinds and parameters (§5), the state (§6), the views (§12); map import v14, ruleset part v13, adventure state hash v16 |
| 6 | **T73** | `h3-adventure`: moving (§7), the prices, the drop and the conversion (§8), `Reach` (§9), visits and events at sea (§10); likely cut in two at its start (the boat's moves and prices; `Reach`) |
| 7 | **T74** | `h3-adventure` and `h3-game`: the pickups' visits (§11), the battle at sea and the boat leaving with its hero (§13), the game (§14), the env's observation (`cross-lane`, §15) |
| 8 | **T75** + **T76** | T75: the sea fixtures, replays, properties, benchmarks, `--with-original-data` (§15). T76: the audit against the exit criteria, increment 8 closed, increment 9 refined |

**Increment 8's exit criteria:**
1. Boats import without the original's cap and a hero placed on water is refused (D-44 as batch
   E8 left it); any hero boards any empty boat; the owner and captain are kept and never shown; a
   boat leaves the map with its hero (ADV-SEA-001, -006, -007, GAM-BAT-003).
2. Boarding and landing are the original's: the walk ends, the free coastal tile only (D-42), the
   Admiral's Hat's conversion after the step's drop (D-41 as batch E8 decided), the day's maximum
   switched (A-28 not reproduced), no guard at a boarding, the landing's guards at their tile
   (ADV-SEA-002, -003).
3. A boat sails at the sea's prices with Favorable Winds, the diagonal between two waters, never
   onto an empty boat, with the drop and the diagonal exception at sea (D-43 as decided); the day's
   sea points come from Navigation, its specialists, the Necklace and the two Hats at the turn
   start in a boat (ADV-SEA-004, HER-ADV-009, ADV-MOV-001, -002); every path `Reach` gives a boat
   is one `MoveHero` accepts.
4. At sea objects are visited from the neighbouring water tile, a boat's hero is met or attacked
   from the neighbour or from the shore and never attacks the shore, events on water fire and stop
   the move, a water monster attacks a boat at once (ADV-SEA-005, ADV-OBJ-007, ADV-MOV-011).
5. A battle with a hero in a boat is fought on the deck, with both in boats on the two boats
   (GAM-BAT-001).
6. Flotsam, the Sea Chest, the Shipwreck Survivor and the Ocean Bottle draw, give, turn, lose and
   leave as the original; only a boat takes one, never a flyer, a walker or a water walker (D-45
   reverted; ADV-PIK-014 to -018).
7. A viewer sees empty boats' tiles, kinds and facings and every seen hero's boat, never a boat's
   owner or captain nor a pickup's contents (ADV-FOG-006, ADV-FOG-008).
8. Every new rule has its id, source and tests; `ADV-QUIRK-30` to `-32` recorded as not
   reproduced; the gates pass (coverage, determinism with the re-recorded and new replays, the
   benchmarks at their baselines and the new ones baselined); the env plays on at sea.
9. Batch E8 run and scored (T69: D-41 to D-43 confirmed, D-44 refused, D-45 reverted) and batch
   E8c's D-46; the consensus rows `verified`.

### 19. Risks and open points

- **Boats are the first objects that move.** The boat's records, the empty boats' grid and the
  heroes' boats must agree after every path (a move, a battle's removal, a fork's write): §6's
  invariants are checked after every command in the properties, and the hash covers all three.
- **A third search** in `Reach`: its cost on the sea XL is new (`reach-sea-xl-144`); the walker's
  boat test must stay out of the hot loop (`bench-check` on `reach-xl-144`).
- **The pictures' sizes** (read by T67 from the sprite archives' `.msk`): every hole 1 × 1;
  Favorable Winds `AVXfw0` to `AVXfw7` 6 × 4, 8 × 6, 6 × 6, 8 × 3, 3 × 6, 3 × 3, 5 × 2, 2 × 5 (the
  Clover Field's eight sizes): T72 adds them to `GROUND_PICTURES`. A hole on a coastal tile was
  not counted in the shipped maps (§1), nor a boat sharing a tile with another object (§4): the
  build's census.
- **The facing's order:** the original's facing numbers against the engine's `DIRECTIONS` (§6),
  for the view only; E8-15 records a landing's facing.
- **D-45's flyer against a monster on water** (§13) followed the pickups' rule: E8-7 reverted
  both (T69). **D-46** (a flight over a water tile holding an object, survey §15) changes
  ADV-MOV-009, built and `verified` in increment 5, if batch E8c confirms the code: T73 then
  closes those tiles to a flight in `passage` and `Reach`.
- **The env's encoding** (`cross-lane`, §15): if the user prefers the AI lane's inbox, the env
  encodes no boat until the AI lane takes it, and its agents see a hero in a boat as on its tile.
- **Not in increment 8** (decision 168): Shipyards and Lighthouses (item 9), Whirlpools, Buoys,
  Mermaids, Sirens, the Derelict Ship and the Shipwreck (item 10), Summon Boat and Scuttle Boat, the
  computer player's boats (E10).
