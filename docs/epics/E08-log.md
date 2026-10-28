# E08 log: done tasks in full

The full record of E08's finished tasks (what was built, refinements, agent decisions marked
"change freely", measurements). **Not start-of-session reading:** the epic file keeps one line per
done task; read the matching section here when a task builds on an earlier one or a decision is
questioned. Rules live in the catalog, quirks in `docs/rules/quirks-adventure.md`, decisions in
ADR-0015 to ADR-0017; this log only records how and why the tasks went.

## Refinement (2026-10-07/08, with E07)

The brief's five points settled with the user: real maps first (the reader fills E07), the crate
split per context after E04b, `h3-game` from increment 1, `h3-heroes` keeps only the hero's own
state and rules, cheap forks via a wide persistent grid (quadtree rejected), the fog fringe as a
quirk candidate (faithful: objects shown; strict-fixed: terrain only), simultaneous turns
designed for, not built (turn groups, per-player server-only RNG streams, no blocking prompts; the
commutation target captured, its decision deferred). Review with the user (PR #21): the hero crate
keeps `derive_battle_hero`/`war_machines`; T1 split into T1a and T1b; four crates (no
`h3-content-adventure`); added the import boundary and setup types, the leavable own tile, map
input bounds and the missing fuzz harness, the two copy-on-write properties, the viewer knowledge
list, adventure sub-paths in battle's crates, the versions rows, `docs/formats/h3m.md` to
adventure, the C ABI non-goal, E09/E10's residual scope, exit criteria, the fringe's confirmation.

## Done tasks

### T1a — Three ADR drafts (2026-10-09, PR #32; accepted 2026-10-10)

Written from decisions 2–9 as they stood; no code. A fresh reviewer agent checked them against the
epic, the ADRs and the server lane's draft; its findings were folded in. New beyond the epic, each
flagged for the review: per-object state in a persistent chunked table and `check-arch` tokens
against pointer identity; two visiting modes (entering, or from a neighbouring tile) and heroes
blocking in increment 1; visibility resolved at emission; the import's three stages with setup
types owned by their validators; `Actor::System` and `Viewer::Spectator` as additive shapes
(SRV-Q6), a setup stream, a battle with another player's hero owned by both players, an
inventory of state two players can write. Deferred to increment 3 with the server lane: where a
battle's seed and obstacles come from.

**Checkpoint (2026-10-10):** the user read a one-screen summary per draft (the new points marked)
and accepted all three as written: ADR-0015, ADR-0016, ADR-0017. ADV-Q1: ids in `h3-contracts`.
ADV-Q2: a replay names its map by content hash and path plus the import version.

### T1b — Survey of movement, fog of war, the day and simultaneous turns (2026-10-09, PR #33)

Sources: VCMI 1.7.5, the wiki's Movement and Scouting pages (approved for that session), the
executable's tables already mapped, the HD mod's page. No code reading: the disputes name the
function to read (CR-1 step cost, CR-2 a day's points, CR-3 the reveal), queued as the first step
of T7, T6/T7 and T8 (`ADV-Q3`–`ADV-Q5`). Findings that touch the ADRs: a path may use explored
tiles only (VCMI; question 4 of the user's map check), so the action mask cannot leak; every
increment-1 rule runs at the start of the day; increment 1 has no owned objects, so a game opens
with only the heroes' surroundings explored (a recorded simplification). VCMI's contact rule for
simultaneous turns (reachability at the start of the day) is the recommended model for later.

**Classification (2026-10-10):** the user classifies A-2 and A-3 after T7's code reading; A-1
waits for the user's look at a map (four questions in the survey's §4), needed by T8 only. Rule
ids of the game context use the existing prefix `GAM-` (`check-rules`), not `GAME-` as the survey
proposed.

### T2 — Shared skeleton (2026-10-10, `shared/adventure-skeleton`, `cross-lane`)

One pull request with the user's consent: the new adventure crates' skeletons must land with the
workspace members and `arch.toml`, and `check-lane` refuses lane and shared paths together
without the label.

- **Crates:** `h3-adventure` (domain) and `h3-game` (application) as logic crates with the lint
  header and their hash-version constants; `h3-map` (with `MAP_IMPORT_VERSION`) and
  `h3-pack-adventure` as adapters, doc-only. No code, so no coverage yet.
- **`arch.toml`:** the four crates' edges; `h3-sim` may depend on `h3-adventure` and `h3-game`
  (dev: `h3-map`, `h3-pack-adventure`), `h3-cli` on all four, so T9 needs no shared change;
  `h3-map`'s dev-dependencies on the two pack adapters build test rulesets; `ptr_eq` and `as_ptr`
  are forbidden in logic crates outside `crates/h3-adventure/src/persistent.rs` (T5's module;
  agent decision, change freely: renaming it is a shared change).
- **`h3-contracts`:** `PlayerId(u8)` (position in the game's player list, colour order) and
  `HeroId(u16)` (position in the game's store of heroes), opaque with `new`/`raw`/`index`; no
  canonical-hash impls (the state hashes feed `raw()`).
- **`h3-kernel`:** `StreamId::GAME` (4), `SETUP` (5) and `player(i)` = 0x100 + i; KER-RNG-001
  lists the ids. No existing sequence changes, so `RNG_VERSION` stays 1.
- **`lanes.toml`:** the new crates, `src/adventure/` of `h3-sim` and `h3-cli`,
  `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`,
  each pack's `adventure.toml`, `docs/formats/h3m.md` (moved from battle, user consent) and
  `docs/rules/catalog/game.toml` to adventure; the survey index is a common path.
- **`xtask`:** `verify --scope` runs the determinism step for a lane that owns `h3-cli` or a
  non-empty replay folder of its own; `cargo xtask determinism` re-runs every existing replay test
  target of `h3-cli` (`replay_fixtures`, and `adventure_replays` once T9 adds it).
- **Executable map:** `check-exe-map` had failed on `master` since T1b's PR #33 (docs only, so
  CI never ran it): the survey cites H3API's `GetMovementCost`, `GetScoutingRadius`,
  `MaxLandMovement` and `CalcMaxMovement` (0x4B18C0, 0x4E4070, 0x4E4990, 0x4E4D90). Added as
  `guess` entries with the sizes from `scripts/exe.py check --fix-sizes`; T6–T8 read them.
- **Fuzzing:** the user chose stable-only for now: parsers of untrusted input are property-tested
  on arbitrary bytes; `cargo-fuzz` needs nightly, which the sandbox cannot download (BACKLOG
  inbox).
- **Docs:** the versions table (game state hash, adventure state and ruleset-part hashes, map
  import, adventure replay format), lane ownership in `docs/08`, `BACKLOG.md` (E07 folded into
  E08, E08 in progress, E09/E10 restated, R01 the content split), the global HANDOFF's adventure
  row, ADR-0005/0006 amendments, `docs/02` §3 and `docs/03` (Game row, terms, the object question
  answered).

### T3a — Map reader up to the terrain (2026-10-10, `adventure/e08-t3`, after `shared/h3m-reader-gate`)

The seam agreed with the user at the start: T3 builds only the reader (T3a to the terrain, T3b the
templates, objects and events, then a checkpoint); the import moves to T5 (`AdventureSetup`:
terrain, roads, rivers, footprints) and T6 (`GameSetup`: players, placed heroes, `Unsupported`,
`MAP_IMPORT_VERSION` documented, and the arbitrary-bytes property over parse, import and
validation together), because the setup types are best shaped with the validators that own them.

- **Shared prerequisite** (`shared/h3m-reader-gate`, PR #48, merged first): `h3-map` may use
  `miniz_oxide` and, in tests, `h3-formats`; `verify --with-original-data` runs every crate's
  `examples/original_data.rs` with the installation path (a crate adds one without a shared
  change); the sandbox setup copies staged maps. The user had the installation's `Maps` copied
  into `original-data/install/Maps` (164 files, compared byte for byte; the installation itself
  only read).
- **API:** `read(file, &Limits)` (bounded gzip, then `parse`), `parse(bytes, &Limits)`,
  `gunzip`, `crc32`, `Limits` (defaults in `docs/formats/h3m.md`), `ReadError` (file too large,
  not gzip, map too large, checksum, unsupported version, truncated, limit exceeded, invalid
  value; field and offset each), `model::RawMap` and its parts. Agent decisions, change freely:
  codes stay raw numbers (terrain, hero types, artifacts) and only layout-deciding values are
  checked (condition kinds, the map size); strings stay bytes (`Text`; other language editions
  use other code pages); bit sets stay bytes (`Bits`); unused bytes are skipped, not kept.
- **gzip:** RFC 1952 header with its optional fields, `miniz_oxide`'s limited inflate, the
  trailer's CRC-32 (own table, check value `CBF43926`) and length must match; a multi-member file
  fails the checksum.
- **Bounds:** every count is checked against its limit and against the bytes left
  (`count × smallest record`), so a hostile count is `Truncated` before any allocation.
- **Tests:** a test-only encoder (the inverse of `parse`) and strategies for every model a version
  can hold; properties: round trip, every strict prefix is `Truncated`, arbitrary bytes after a
  version and mutated maps never panic, the gzip layer reads back and fails cleanly; unit tests for
  each error and limit, the gzip header fields, and a map of the experiment writer read back. The
  strategies are boxed: unboxed, the value tree overflowed the test thread's stack in debug.
- **Shipped maps:** all 164 read to the end of the terrain with every tile code in range on the
  first run (50 RoE, 55 AB, 59 SoD); the peaks set the bounds' table. `Battle of the Sexes
  Allied` customises 130 hero types and restricts 130.
- **Stub:** ADV-STUB-1, the reader stopped after the terrain (removed by T3b).

### T3b — Map reader: templates, objects, events (2026-10-10, `adventure/e08-t3`, after T3a)

- **Model:** `model::Template`, `Object` (position, template index, `Body`), and `Body` with one
  variant per body the format tells apart by class (and, for mines and the grail, subclass): event,
  hero (also prison and random hero), monster, sign, seer hut, witch hut, scholar, garrison,
  artifact, scroll, resource, town, mine, abandoned mine, owned (dwellings, lighthouse, shipyard),
  random dwelling, shrine, Pandora's box, grail, quest guard, hero placeholder, none. Parts:
  `Army`, `Stack`, `Message`, `Rewards`, `Quest`, `Mission`, `Reward`, `TownBody`, `Buildings`,
  `TimedEvent`, `TownEvent`, `RandomDwelling`. Agent decisions, change freely: RoE's seer hut
  quest is modelled as an artifact mission without deadline or texts (VCMI's reading); an empty army
  slot drops its stored count.
- **Findings on the shipped maps:** every map ends with 124 zero bytes after the timed events, so
  the reader accepts zero padding and rejects anything else (`TrailingBytes`); a town's banned-spell
  set is stored in every version (only the required set is AB's), which a filtered first reading of
  VCMI's reader missed and all 50 RoE maps exposed. All 164 maps then read to their padding;
  every body kind but the campaign hero placeholder occurs (peaks and counts in
  `docs/formats/h3m.md`).
- **Bounds:** templates 4 096, objects 65 536, timed events 1 024 (map or town); the string limit
  stays 64 KiB (peak 1 314).
- **Tests:** the encoder and strategies cover every body (strategies pick a template class per
  body, then a body that fits it); unit tests for an object naming a missing template, unknown
  quest and reward codes, the new limits, every mission and reward, and the experiment writer's
  map with a hero and a monster read back.
- **Not done:** `fixtures/maps/` waits for the user's editor maps (QUEUE inbox).

### Checkpoint after T3 (2026-10-10)

- T4–T10 refined into queue entries; T5 and T6 split into T5a/T5b and T6a/T6b. The user takes one
  or two tasks per round; this round: T4 alone. The branches `adventure/e08-t3a` and `-t3b` were
  to be deleted (the session's GitHub access cannot delete branches; left to the user).

### T4 — Adventure content (2026-10-10, `adventure/e08-t4`)

- **Ruleset part** (`h3-adventure::AdventureRuleset`): compiled on top of the core `Ruleset` from
  one `AdventurePack` per core pack, so it shares the core's load order and dependency closures
  (each pack carries its core manifest; an empty part for a pack without the file). Per core
  terrain a `Passage` (land or water with the straight cost, impassable), road and river kinds
  (handles `RoadId`, `RiverId`), and the parameters: native cost, the land table by the slowest
  speed (faster armies take the last entry), the base scouting radius. Checks: the pack set
  against the core's (id and version), namespaces, duplicates, the closure, every core terrain
  covered once, one pack with valid parameters. Canonical hash v1 covers the part's own content
  (terrain keys, not the core's hash; `core_hash()` keeps the pairing for T6's bundle). Agent
  decisions, change freely: costs are `NonZeroU16`; the table holds 1 to 256 entries.
- **Format** (`h3-pack-adventure`): `[params]`, `[[terrain]]` (`terrain`, `passage` default land,
  `cost`), `[[road]]`, `[[river]]`; a bare name means the own pack; `compile()` takes
  `(pack.toml, adventure.toml)` pairs; `ORIGINAL_PACKS` for RoE to SoD. The format lives in the
  crate's doc; `packs/README.md` got one paragraph (cross-lane, decision 10).
- **Data:** base terrains, roads and rivers in the `.h3m` code order (for T5's import), with
  sources in the file; AB and SoD files are empty (their parts hash equal to RoE's). Synthetic
  packs for tests.
- **D-4 settled by original data:** the original reads the land table from `movement.txt`
  (column 2, speeds 0–20; RoE's and AB's archives hold the same file) through 0x4D7240 into
  0x698A98; 0x4E4990 takes the lowest speed of the seven slots from 20 down, so the wiki's row
  holds (VCMI's 1500 below speed 3 does not). Found for increment 3: the creature of a creature
  specialty and its upgrade count one faster there; Logistics (0x63EA68, now read) multiplies the
  points; at sea the points come from column 4 by Navigation level (0x698AEC). 0x4E4D90 passes
  the hero's at-sea flag; its callers (when the points are set, D-1) stay for T6b.
- **Rules:** ADV-MOV-001, -003, -004, -007 and ADV-FOG-001 written with sources, `specified`
  until T7/T8 read the data; the data tests carry their `covers:` markers.
- **Not done:** a check of `adventure.toml` against `movement.txt` under `--with-original-data`
  (it needs `h3-formats` as a dev-dependency, a shared change; inbox).

### T5a, T6a — Static map, setups and the import (2026-10-10, `adventure/e08-t5a-t6a`)

The user took T5a and T6a as one round (one import, written once into `GameSetup`) and chose,
for placed heroes without stacks of their own (58 of 116 in the shipped maps), a starting-army
placeholder that T6b draws, over an `Unsupported` error.

- **Adventure setup and state** (`h3-adventure`): `AdventureSetup` (players, the static map, the
  heroes), `MapSetup` (width, height, levels; tiles with T4's handles; objects with an anchor,
  `TileMask`s for blocked and visitable tiles, kind `Blocker`), `HeroSetup` (id, owner, tile,
  army as `h3_contracts::ArmyStack`s). `AdventureState::new` validates in order (size bounds,
  tile count and handles, players, each object's level, each hero's id, owner, tile, army) and
  builds the static map behind an `Arc` with per-tile `blocked`/`visitable` flags, the heroes in
  a plain array by id (inserted by binary search: logic crates may not sort), stacks by slot (a
  slot table). `check_stand` (ADV-MAP-001) is the one check T7 reuses. State hash v1: version,
  the static map's hash (computed once at build), players, heroes. Agent decisions, change
  freely: `Pos` uses `u16` columns and rows; `TileMask` is a `u64` of 48 bits whose bit
  `8 × row + column` equals the `Objects.txt` character.
- **Ruleset part v2:** the setup bounds as data (ADR-0015 point 6): `max_map_size` 144,
  `max_levels` 2, `max_players` 8, `army_slots` 7, with sources in `packs/base/adventure.toml`
  (VCMI's constants and the format). `max_players` sits in the adventure part because the
  adventure keeps a layer per player; the game reads it there until a game part exists.
- **Game rules and setup** (`h3-game`): `GameRuleset` (core + adventure part, paired by content
  hash); `GameSetup` (players in colour order, the map, heroes with `HeroArmy::Custom` or
  `Starting`); `PlayerId` and `HeroId` are indices into its lists. `validate` checks the game's
  part (player and hero counts, colours strictly increasing below `max_players`, known hero
  types, starting armies with stacks; the core already checks each stack's counts), then builds
  the adventure setup and lets the adventure validate it. A starting army stands in as its hero
  type's whole army at the lowest counts in slots 0..: every draw takes stacks from the front at
  counts within their bounds (VCMI `CGHeroInstance::initArmy`), so it validates exactly when every
  draw does. ADR-0017 point 4 said "none in increment 1" for setup placeholders: noted in its
  revision history (shared PR).
- **Import** (`h3-map`, `MAP_IMPORT_VERSION` 1, `docs/formats/h3m.md` "Import"): `import` and
  `import_map` (the static layer). Codes to keys only there: terrain, road and river tables, and
  creatures and hero types by the packs' `# original id` (a test parses `packs/*/pack.toml` and
  compares). `Unsupported`: a hero generated at a main town, a random hero, a campaign
  placeholder, a random stack (the 14 codes below "none", VCMI `readCreature32`), a hero standing
  on a town's blocked tile (VCMI `placeHeroesInTowns` moves it into its own town). Other typed
  errors: map size, unknown code, content the ruleset lacks, a missing template, a hero of a colour
  nobody plays, a hero template without one visitable tile on the map. A hero's customisations
  are ignored (`ADV-STUB-1`).
- **Found: the masks' geometry was mirrored** in `h3m.md` and the reader's docs since T3: bit
  `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (VCMI
  `readMap`; `readTxt` reads `Objects.txt` anchor first). Shown by the data: in the old reading
  every town's gate sat on its template's top row and every hero five rows above its position.
  The reader never interpreted the masks, so nothing else changes; `h3_formats::h3m`'s writer
  copies text to bytes by the right relation, only its comment says "top left first" (battle
  lane; inbox for the writer's move).
- **Found: anchors past the edge** are common (decorations, heroes on the last column): up to 5
  columns right and 2 rows below in the shipped maps; VCMI skips off-map tiles. So only an
  object's level is checked (`SetupError::ObjectLevel`). **And a gate is blocked:** towns' masks
  mark their gate blocked and visitable; VCMI's `evaluateAccessibility` asks for visitable first,
  so ADV-MAP-001 lets a hero stand on a tile that is blocked only where it is also visitable.
- **Shipped maps** (`examples/original_data.rs`, run by `verify --with-original-data`): all 164
  static layers import and validate; 10 maps import and validate whole, 129 are unsupported for a
  hero at a main town, 22 for a random hero, 3 for a hero in a town (first reason found).
- **Rules:** ADV-OBJ-001 (the tiles an object covers) and ADV-MAP-001 (where a hero may stand),
  `implemented`.
- **Tests:** every setup and import error, the footprint off the edges, the stand rule, the hash
  (content, not sharing; the order of heroes and stacks in a setup does not matter), the code
  tables against the packs, and parse, import and validation together on arbitrary bytes and on
  the reader's model strategy with damaged encodings.

### Checkpoint after T5a/T6a and T6b + T7a (2026-10-10, `adventure/e08-t6b-t7a`)

At the checkpoint the user took T6b with T7's code reading (T7a) as this round, so the round ends
at the classification of A-2 and A-3, and moved T5b's gated fork benchmark to T9: T5b measures
with callgrind on an adventure-owned example, because the `h3` binary's dispatch and its
`Cargo.toml` are battle-owned (T9 wires the CLI's adventure commands anyway).

- **T6b readings** (survey §1.1, §3.1; map entries). *When the points are set (CR-2's callers,
  ADV-Q4):* the End Turn handler 0x409A70 calls the next-player step 0x4C6FE0, which calls the
  new day 0x4C7FE0 after the day's last player and then sets the new current player's heroes'
  maximum and current points (+0x49, +0x4D; on the map, in its tavern, in its towns); the new day
  sets none within two calls. The wiki and VCMI say the day's start: a conflict of sources; the
  user chose the reading until a confirmation experiment (ADV-Q4, kept open). *The starting
  army (0x4C9730, GAM-SET-001):* the first stack always, a roll of 1–100 at most 88 for the
  second slot (a war machine there becomes its artifact), a new roll at most 25 for the third;
  the wiki has the same chances per slot and 100 % for the nine war machines; VCMI one roll for
  all slots. The user expected the war machine to be guaranteed; it is not on the map hero
  before increment 3, so the round builds the stacks as read (the user, interim) and leaves the
  war machine's chance open (ADV-Q6). *Turn order:* the original plays human players before
  computer players each day (VCMI too); inbox.
- **T6b build.** `h3_game::GameState::new(rules, setup, seed)`: the starting armies drawn from the
  setup stream in hero order (the slots read as the first stack, the war machines, the other
  stacks; the chances `starting_stack_chances = [88, 25]` in the adventure part, ruleset hash
  v3); the game's own check that a hero type's stacks fit the army's slots (so validity never
  depends on the draw; `GameSetup::validate` builds with seed 0); a stream per player and the
  game's stream; sequential turn groups; day 1; the first group's turn started. `apply(state,
  rules, Actor::Player(p), GameCommand::EndTurn, sink)`: `NotYourTurn`/`UnknownPlayer` leave the
  state untouched; a group ends when every player in it has ended its turn; after the last group
  the day advances. Events `TurnEnded`, `DayStarted`, `TurnStarted` (public facts). Game state
  hash v1. `h3-adventure`: `HeroOnMap.movement`, `day_points` (ADV-MOV-001), `start_turn`
  (ADV-MOV-002), state hash v2. Rules GAM-SET-001, GAM-TRN-001, GAM-TRN-002, GAM-CAL-001 (new
  catalog `game.toml`), ADV-MOV-001/002 implemented. Tests: the draw's chances and slot layout,
  4000 seeds against the read rates (the third stack without the second about 3 %), the turns
  and a group of two (a test-only schedule), the rejections, the hash, a property test of
  `EndTurn` atomicity and determinism; the shipped packs keep the draw's slot layout (nine war
  machines, all second of three). Weeks and months wait for the first rule that reads them.
- **T7a reading (CR-1, ADV-Q3; survey §1.2–1.3).** 0x4B18C0 → 0x4B1740: only the source tile
  prices a step (its road's cost when both tiles have one, else 100 when native, else its terrain's
  by Pathfinding); D-2 goes VCMI's way, against the wiki. The diagonal compares the points left
  with the plain terrain cost T: below T unaffordable, from T to below T·√2 the straight cost,
  above trunc(cost·√2); A-2 and A-3 exist as the wiki says, confirmed by an independent re-read by
  a fresh agent (ledger `ADV-QUIRK-2`, `-3`, fix-optional pending the user). After each step the
  points drop to 0 when fewer than the cheapest straight step from the new tile remain (0x4B1880;
  D-3: not VCMI's rule). The f32 √2 truncated equals `isqrt(2·cost²)` for every cost below 5741.
  Decisions for the user in ADV-Q3; T7b builds on them.

### Checkpoint after T6b/T7a and T7b + T5b (2026-10-11, `adventure/e08-t7b-t5b`)

At the checkpoint the user took T7b and T5b as one round (T7b first), decided ADV-Q3 (D-2: the
source tile prices a step, the reading and VCMI; D-3: the original's last-step rule; A-2 and A-3
fix-optional) and, during T7b, classified a new quirk and allowed the quirk flags as a
`cross-lane` change in this round's pull request (`h3_content`'s registry, the harvester's and
`packs/base/pack.toml`'s profiles are battle-owned).

- **T7b readings.** *The exception's bound* (0x4B1740 from 0x4B181A, re-read): the points and T
  go to f32 exactly and T × 1.41421354 (x87, 53 bits, exact for these sizes and never an integer)
  is compared with the points, so the straight cost applies for T ≤ points ≤ `isqrt(2·T²)`
  (141 points on grass: the diagonal costs 100). The clean check keeps that bound and charge and
  only takes the step's own straight cost for T (VCMI takes every remaining point and stops one
  lower). *The army's native terrain* (0x44C590, called by 0x4B18C0): the slots in order, each
  creature's faction's terrain from 0x643698, a neutral creature reading the -1 before the table,
  which is also "none yet": neutral stacks before the first faction stack are skipped, any later
  difference gives none. The wiki ("all the creatures") and VCMI 1.7.5 (`getNativeTerrain`, any
  neutral stack: none) differ; a fresh agent's independent re-read agreed exactly: `ADV-QUIRK-4`,
  fix-optional (the user). The four elementals count as neutral there while a game flag is clear
  (RoE content); the packs already have them neutral in `base`, Conflux in `ab`.
- **T7b build.** `h3_content`: flags 19–21 `quirk.adventure.diagonal_check_ignores_roads`,
  `diagonal_check_ignores_native`, `native_skips_leading_neutrals` (on in `faithful`, off in
  `strict-fixed`; after the battle lane's T8e, which took 17 and 18 and merged first).
  `h3-adventure`: `native_terrain` (ADV-MOV-004), `diagonal_cost` (ADV-MOV-005, `isqrt`), `Pricing` (the step's price from the source tile alone, the exception's check under
  the two flags, the points left zeroed below the cheapest straight step from the new tile:
  ADV-MOV-003, -006), `AdventureCommand::MoveHero` (`AdventureState::apply`: the hero must be the
  player's, one error for unknown or foreign heroes; each step to a neighbour on the level, onto a
  tile the hero may stand on, in increment 1 not an object's visitable tile; priced at the points
  left; the whole path checked before anything changes; one `HeroStepped` event per step;
  ADV-MOV-007), and `Reach` (`AdventureState::reach`: Dijkstra by points left over a window as
  far as the cheapest step allows, one path per tile, ties to the first found in tile and
  direction order, the original's clockwise directions 0x678150). `h3-game`:
  `GameCommand::Adventure` after the turn check, `GameCommandError::Adventure`,
  `GameEvent::Adventure`. No hash version changes (no new state). Tests: each step rule from its
  source, the exception's bounds in both profiles, D-3's example from ADV-Q3, MoveHero's
  rejections leaving the state equal, the query's points, paths and window, a later better path
  replacing the first, and a property on random maps, armies and profiles (every reported path is
  accepted with the reported points; an accepted one-step move lands on a reported tile with at
  most its points; rejected moves change nothing), stressed with 20 000 cases. A first version of
  the property also claimed that a rejected one-step move's tile is never reported: false, a tile
  next to the hero may be reachable only around a corner (from a swamp road over the next road
  tile with 162 points); the regression seed stays in `proptest-regressions/`.
- **The query's limit** (followed up the same day at the user's request: review, measurements,
  the original's pathfinder). Dijkstra's, like A*, is exact when more points on a tile are never
  worse later; the diagonal exception breaks that (a diagonal from swamp costs 175 with 175–247
  points, 247 with more: 257 points on the swamp leave 10 after it, 245 leave 70). *Review:* with
  the exception switched off (everything else as built) the query matched an exhaustive search
  over (tile, points) exactly on 1 152 queries of random 16 × 16 maps (~124 000 tiles, both
  profiles), so the search is right and the exception alone causes the differences. *With it*
  (random 16 × 16 maps, up to four moves along reported paths each): faithful, 60 of 572 queries
  differ, 4 of 64 713 reachable tiles not reported (each reachable only with 0 points left), 117
  reported with fewer points than possible (on average 12.9); strict-fixed, 6 of 581 queries, 3
  of 64 203 tiles missed, 9 short (49.4 on average). The exhaustive search explored ~45 000 states
  per query against ~110 tiles (≈ 26 ms against microseconds): too slow for the env's mask.
  Regression test `more_points_on_a_tile_can_leave_fewer_after_the_next_diagonal` (strict-fixed:
  the query's path leaves 0, a path over sand 70). *The original* (0x4194A0 → 0x56B440, 0x4B2300,
  0x4B1A70; survey §1.3): one node per tile, each step priced at the node's points left, a node
  replaced only by a strictly better one: the same limit, so the path the game shows can leave
  fewer points than one clicked step by step. MoveHero accepts every payable path.
- **T5b build and measurements.** `BitGrid<WORDS>` (`src/persistent.rs`): an `Arc` table of
  `Arc` leaves of `64 × WORDS` tiles, an empty leaf none and a full one the grid's shared full
  leaf (one stored form per content); `get`, `set`, `set_run` (a row of a leaf is one masked
  word); equality, `Debug` and the hash see content only (the leaf size is part of the hash
  encoding). Properties: a write through a fork reaches neither parent nor sibling, and equal
  content compares and hashes equal, for 8×8 to 64×64 leaves (20 000 cases). `examples/
  grid_bench.rs` under callgrind, per operation on one player's layer with twenty revealed disks
  (instructions; flat = one bitset of the map copied by each fork):

  | 144×144×2 | flat | 8×8 | 16×16 | 32×32 | 64×64 |
  |---|---|---|---|---|---|
  | fork | 906 | 22 | 22 | 22 | 22 |
  | fork + first write | 955 | 11 498 | 3 585 | 1 550 | 1 052 |
  | reveal r 5 by rows | 1 194 | 3 889 | 2 824 | 2 443 | 2 190 |
  | reveal r 5 by tiles | 2 696 | 11 874 | 11 444 | 11 252 | 10 620 |
  | read every tile | 1 329 426 | 1 334 326 | 1 394 230 | 1 500 214 | 1 556 116 |
  | 1000 scattered reads | 61 009 | 58 081 | 61 625 | 65 318 | 63 439 |

  | 36×36 | flat | 8×8 | 16×16 | 32×32 | 64×64 |
  |---|---|---|---|---|---|
  | fork | 209 | 22 | 22 | 22 | 22 |
  | fork + first write | 258 | 1 128 | 808 | 722 | 725 |
  | reveal r 5 by rows | 1 165 | 2 167 | 1 947 | 1 814 | 1 627 |
  | read every tile | 41 772 | 52 322 | 52 469 | 52 469 | 49 696 |
  | 1000 scattered reads | 61 009 | 66 007 | 71 007 | 71 007 | 64 007 |

  The first write after a fork copies the table (one reference increment per leaf), so small
  leaves pay on large maps; the grid loses 5–25 % on per-tile reads and about half on a reveal.
  Taken back to the user (ADR-0016): **32 × 32 leaves** (the user, 2026-10-11), the default.
  For T8: build observation planes word by word, not tile by tile, and batch a reveal's rows per
  leaf (one copy-on-write check per leaf).

### Checkpoint after T7b/T5b and T8a (2026-10-11, `adventure/e08-t8a`)

The user merged T7b/T5b (#69, #70), kept the query's one path per tile (the original's way; no
exact search), answered A-1 in SoD (terrain, objects and creatures show on the fog's half-dark
border; no path onto a dark tile, even an empty one) and took **T8a alone** of T8's split
(T8a the reveal, T8b the projection). A-1 is confirmed as `ADV-QUIRK-1`, class fix-optional as
the survey suggested (Q30's rule; the user may object at this checkpoint); its flag comes with
T8b.

- **T8a readings** (CR-3, survey §2 and §1.4; map entries). *The radius* (0x4E4070, H3API's
  name confirmed): `skill_scouting` at the Scouting level; with Scouting and a Scouting specialty
  × (1 + 0.05 × level) in f32, truncated; +1 each for Speculum (52) and Spyglass (53), worn or in
  a worn combination. *The reveal* (0x49D160): the tiles of the map and level with
  `sqrt(dx² + dy²) ≤ r + 0.5` (f64; 0x617B94 is `sqrt`), the bits of every player of the
  revealer's team in each tile's vision word (u16 per tile at 0x6989F8, low byte one bit per
  colour): VCMI's disk `dx² + dy² ≤ r² + r` exactly (ADV-Q5 settled). *When:* the move's step
  reveals around its destination before the hero's position changes (0x48106B), a placed hero's
  setup around the hero (0x486E25), the new day around every owned hero and further owned objects
  (0x4C7C50, the hero loop read). *The pathfinder:* the expansion tests a neighbour's bit against
  the current player's (0x69CCC4, written with the current player): unexplored is inaccessible
  for a human player (0x4BADA0, record +0xE2) unless the search's mode argument is 3 (the
  mouse's search passes 0; 3 not read); a computer player may step one tile in from an explored
  tile, or anywhere while its record's +0x3E is above 0 (not read). Not read: what the reveal's
  helper call and last argument do (a changed tile is queued; a call after the loop).
- **T8a build.** `h3-adventure`: `fog::reveal` (a disk row by row, one `BitGrid::set_run` per
  row, integer `isqrt`; ADV-FOG-002); `AdventureState` keeps one `BitGrid` of explored tiles per
  player (`explored`, `is_explored`), each hero reveals around itself to its owner after the
  setup is checked (ADV-FOG-001, -003); `MoveHero` checks each step's tile for being explored
  (the tiles explored when the command is given) after the neighbour test and before every other
  tile test, a tile off the map still `Blocked { Outside }`: `AdventureCommandError::Unexplored`
  (ADV-MOV-008); after the move each step reveals around its tile and emits its event;
  `AdventureEvent` is now `{ kind: AdventureEventKind, seen_by: Viewers }` (as the battle's
  `Event`), `Viewers` a set of up to eight players, a step seen by its owner and by every player
  that explored the tile it leaves or enters (ADV-FOG-004); `Reach` skips unexplored tiles;
  `MAX_PLAYERS` = 8 bounds the part's `max_players` (a compile error above). State hash v3 (the
  explored layers). `h3-game`: the move's test sees both players (a 4 × 4 map). Tests: the
  shape against a tile-by-tile model (fixed cases and a property over map sizes, centres off the
  map, levels and radii 0–11; 97 tiles at radius 5), the start's and each step's reveals, a path
  past the explored tiles refused whole though its first steps would reveal them, the explored
  test before the rock test, the steps' viewers, the query's limit at the explored edge, the hash.
  The query's property on random maps now runs with fog on both sides. A test-only
  `explore_all` keeps two pricing tests whose paths run past radius 5. The ten shipped maps that
  import whole still build (`examples/original_data.rs`).
- **Deviation:** the queue's scope said "batched per leaf"; the reveal writes one run per row
  (T5b's measured path, about 2 400 instructions at radius 5 on 144 × 144). A per-leaf batch is
  an API on `BitGrid`; T9's benchmark decides whether a step's reveal is worth it (inbox).
- **For T8b:** a hero on a fringe tile was not asked (an object like the others, presumably);
  `viewers` then adds the fringe under `ADV-QUIRK-1` in `faithful`; the observation reads the
  grids word by word.

### Checkpoint after T8a and T8b1 (2026-10-11, `adventure/e08-t8b1`)

The user merged T8a (#75, #77), raised no objection to `ADV-QUIRK-1`'s class (fix-optional), and
split T8b at the agent's proposal: **T8b1** (what a player sees: the fringe, `observe`,
`project`, the viewer knowledge list) this round, **T8b2** (`legal_commands` and the property
tests) the next. Two answers for T8b1: a hero on a fringe tile shows as creatures do (under the
flag; not looked at in SoD); of another player's hero a player sees its tile, owner and identity,
its army and movement points stay private until the original's right-click view is surveyed
(increment 3, with the battle seam).

- **The flag** (a `cross-lane` change, as T7b's): `quirk.adventure.fringe_shows_objects`
  (`h3_content::FRINGE_SHOWS_OBJECTS`, index 22), on in the base pack's `faithful`, off in
  `strict-fixed` (`packs/base/pack.toml` and the harvester's profile text).
- **The fringe** (ADV-FOG-005, `fog.rs`): the tiles not explored with an explored one among the
  eight around them on their level. `on_fringe` tests one tile (nine reads); `Sight::of` builds
  the explored and fringe planes word by word: each grid row read once (`BitGrid::read_row`, one
  masked word per leaf the row crosses, any leaf size), the fringe as the explored rows above,
  at and below or-ed, spread one column either way with the carries between words, less the
  explored tiles, masked to the width. Both checked against a tile-by-tile model (fixed cases
  across words and edges, a property over widths 1–149 and two levels); `read_row` against the
  flat model for the four leaf sizes.
- **The steps' viewers** add, under the flag, every player with the step's tile left or entered
  on its fringe (it sees heroes there).
- **The observation** (`observe.rs`, `AdventureState::observe(core, viewer)`): the map's size,
  the explored and fringe planes (rows of `stride` words, bit `x % 64` of word `x / 64`), one
  `TileView` per tile (`Hidden`; `Ground` with terrain, road and river on the fringe without the
  flag; `Seen` with the static tile, what objects cover there included, on explored tiles and on
  the fringe under the flag), and the heroes the viewer sees by id (`HeroView`: tile, owner, id;
  `own` with army and points for its own). A viewer that is not a player of the game sees no tile
  and no hero. The tile views are built tile by tile from the planes (a branch per tile, no grid
  read); T9's benchmark measures `observe`.
- **The projection:** `AdventureEvent::project(viewer)` gives the event to the players who see it
  with the viewers reduced to the viewer alone (the full set would tell which tiles other players
  explored). `HeroStepped` now carries `from` and `to` as options with the players that see the
  hero on each (`seen_from`, `seen_to`; `seen_by` their union): a player that sees the hero on
  one tile only is told that one, the other `None` (the review found that T8a's event gave a
  player told of a step both tiles, a hidden one included). `h3-game`: `Viewer::Player`, `observe` (the day, each player's colour and whether it
  acts, the turn group, the adventure observation) and `project` (the game's own events public).
  The spectator viewer comes when the server needs it (additive, as the battle's).
- **Tests:** the planes and tile views against the model under both profiles (also on a map of
  70 columns and two levels), the heroes a player
  sees (fringe under the flag only, another player's without army or points), a viewer that is
  not a player, hidden state that never changes red's observation (a hidden tile's terrain, road
  or object, a hidden hero and so another explored layer, a seen hero's army), a fringe object
  that changes it under the flag only, steps onto and off the fringe seen under the flag only
  and what each is told, the projection; in `h3-game` the public facts, an observation equal for 40 seeds where the seed draws
  only what the viewer cannot see, the events' projection.
- **Docs:** ADV-FOG-005 and -006 in the catalog (ADV-FOG-004 updated), `ADV-QUIRK-1`'s flag and
  regression test, the survey's A-1 row; the viewer knowledge list in `docs/06-security.md` (a
  `shared/` pull request).
- **Review** (an independent agent): no bit or carry error; the one-ended step (fixed above), two
  test gaps (a map past one word with two levels, a step off the fringe; added). Not taken:
  `#[non_exhaustive]` on `Viewer` (a downstream wildcard arm would be dead code under the
  coverage gate; the spectator variant is added with its users, as ADR-0017 says).
- **Not done here:** `legal_commands` and the property tests (T8b2). Open: the fringe's eight
  neighbours are ADR-0016's "touching", not a reading of how the original draws the border
  (inbox).

### Checkpoint after T8b1 and T8b2a (2026-10-11, `adventure/e08-t8b2a`)

The user merged T8b1 (#81, #82) and asked for smaller rounds as the work grows. T8b2 was split
at the agent's proposal: **T8b2a** (`legal_commands` and the properties of what a command may do)
this round, **T8b2b** (the properties of what a viewer may learn: observation hygiene, the
observation against the events) the next. The user asked why an agent cannot make the three
editor maps T9 waits for: the agent explained (they are worth having because the original editor
wrote their bytes; a map our writer makes only tests the reader against itself; driving the
editor through screenshots costs many tokens and runs from the installation that must stay
untouched) and offered a generated map for T9's replays. The user then dropped the editor maps
(too little gain for the time; reconsider only if a bug shows up they would have caught): T9's
replays play a map T8b2a's `lay_out` generates, written once by a test and committed; the shipped
maps stay the reader's test, locally.

- **The legal commands and moves** (`h3-game`, `legal.rs`): `legal_commands(state, rules,
  actor)` lists `EndTurn`; `legal_moves(state, rules, actor, hero)` gives a hero's moves as its
  reachable-tiles query (`Reach`: the points left on each tile, the path built for the one tile
  chosen). Both empty outside the actor's turn, through `apply`'s own turn check
  (`check_acting`); `legal_moves` also for another player's hero. The first version listed one
  `MoveHero` per reachable tile with its path; the user asked whether that is not excessive (a
  hero with 3000 points, on roads at 50 a step, reaches tiles 60 steps away: thousands of
  commands per hero and call, each with a path of dozens of tiles, which the env would pay at
  every step and a server should never send). Reworked in the same pull request, as the battle
  gives casts by their targets (ADR-0014 point 9): the moves are factorized, the env's mask is
  the query's tiles (ADR-0017 point 8: the action is a destination, the env builds the path),
  and a client computes its own paths: `apply` checks every step of any path against the state
  and the actor (ADV-MOV-007, ADV-MOV-008) and rejects it whole. The query keeps one path per
  tile, so `apply` also accepts other payable paths (one back to the hero's tile, or one the
  query does not keep, `Reach`'s doc). Unit tests on the 4 × 4 map: the moves are the query,
  each path applies with its points, nothing outside the turn or for another player's hero.
- **Playable maps** (`h3-map`'s `tests/play.rs`): a `Plan` (version, side 5–16, one or two
  levels, mostly land with a twentieth each water and rock, roads and rivers, two to four colours,
  two to six heroes with distinct types and custom or starting armies of base creatures, up to
  eight blockers of three shapes, one visited from a tile outside its blocked ones) laid out as a
  raw map (heroes on distinct land tiles, owners round the players, blockers covering a hero's
  tile left out), written by the test encoder, read back and imported, so every case plays an
  imported file. At the end of a case players have explored a third to all of their map.
- **The driver:** steps resolved against the state: the acting player's `n`-th option (its legal
  commands, then each legal move with the query's path: the test lists moves one by one on its
  small maps, the game does not), an
  `EndTurn`, a legal move edited (last step dropped, one more step, one more step onto a hidden
  tile, a step turned, another hero, another player), or a free command (`EndTurn` or a move by
  the acting player, a player of the game or one past them; walks in given directions with a
  jump anywhere, or a straight line to a tile across explored and hidden tiles; the hero the
  actor's own or any id up to one past the heroes). Over 48 cases of up to 40 steps about three
  in four commands apply (a quarter of them `EndTurn`; a day passes about once per case) and
  every rejection kind occurs.
- **The properties** (48 cases each, both profiles in the first): at every step only the acting
  player has legal commands (`EndTurn`) and legal moves, which are the query of each of its own
  heroes and none for another player's; every move to a tile of the query along its path applies
  and leaves the hero on that tile with the query's points, one event per step (legality
  consistency); a rejected
  command changes neither the state nor its hash and emits nothing, and was not listed
  (atomicity); an accepted command was the acting player's, an unlisted accepted one is a move of
  its own hero, and an unlisted one-step move goes to a listed tile with at most the query's
  points; a command moves no other player's hero, changes no other player's army or explored
  tiles, and changes another player's heroes' points only when its turn starts (ownership
  isolation); a fork playing other commands leaves its parent's hash and content as a fresh replay
  of the prefix has them, and a fork and its parent evolve identically (clone equivalence); two
  plays of the same file, seed and steps give the same states, events and hashes, and the laid
  out map survives the encoder and reader unchanged (determinism).
- **Review** (an independent agent): no bug; taken: the fork test drew an empty suffix in about
  half its cases (the split is now an index into at least one step), the legal moves are
  compared with the query and an unlisted one-step move's tile looked up in it, the
  isolation check compares the hero counts before zipping, the edit onto a hidden tile tries all
  eight directions, the determinism test plays the second run on rules compiled again. Deferred:
  decision 9 says ownership isolation covers the players' streams; no command draws yet and the
  streams have no accessor (inbox).
- **Not done here:** observation hygiene and the observation against the events (T8b2b).

### Checkpoint after T8b2a and T8b2b + T9a (2026-10-11, `adventure/e08-t8b2b-t9a`)

The user merged T8b2a (#89, #90) and asked for a couple of tasks per round with the seam
confirmed first. The agent proposed, and the user took, **T8b2b** with **T9a**: T9 split into
T9a (the env) and T9b (the CLI, the replays and the benchmark gate, next round). The user allowed
the two battle-owned lines `h3-sim` needs (its `Cargo.toml` and `lib.rs`) as a `cross-lane`
change, and chose `h3_game`'s view as the env's observation for now (a flat encoding waits for
E13's macro-actions).

- **Observation hygiene** (`h3-map`'s `tests/view.rs`, 48 cases, both profiles): a game on a
  playable map and a second one that differs only in what one viewer never sees: another seed,
  other armies for the other players' heroes (so other points, natives and starting armies),
  other terrain, roads and rivers on tiles it never sees, blockers removed or added and other
  players' heroes moved where it never sees what stands. "Never sees" comes from its explored
  tiles at the end of the first game by the rules (ADV-FOG-004, -005: a tile next to an explored
  one shows its ground, and its objects and heroes only under `ADV-QUIRK-1`), not from the
  observation under test: a first version took it from the observation and missed a mutation
  that showed objects on the fringe in `strict-fixed`. The viewer's own heroes get armies of
  their own (a starting army is drawn from the seed, also for a hero whose slots are all empty).
  Played in lockstep with the same commands, the two games show the viewer the same observation,
  legal commands and legal moves of every hero it sees, give its own commands the same result
  and tell it the same projected events, as long as the other players' commands are accepted in
  both or rejected in both (a move one hero can pay and its double cannot ends where the viewer
  may see it). Over 200 cases about four steps in five were compared, the viewer's own commands
  applied about a thousand times and it was told other players' steps in two cases in three.
- **The observation against the events** (`views_move_as_the_events_say`, 48 cases): for every
  player at every step, its view after a command is its view before with the projected events
  applied as a client would: the day, the acting flags and the turn group follow `DayStarted`,
  `TurnEnded` and `TurnStarted`; a projected step names the viewer alone, its `seen_from`/`seen_to` match the
  tiles it tells and it tells at least one; the tile a hero leaves is where the viewer last saw
  it, an untold tile entered takes it out of sight; a hero shows untold only after the viewer's
  own command and only on a tile it did not see before; a tile's view only deepens (hidden,
  ground, everything) and keeps what it showed; the explored and fringe planes change only by
  the viewer's own command. Mutations each test caught: other heroes' armies shown (hygiene),
  a step's tiles not filtered (consistency), objects on the fringe in `strict-fixed` (both),
  every hero shown (both), the query over unexplored tiles (hygiene). A unit test checks that the
  variation changes each kind of thing where the viewer never sees and only the armies where it
  sees everything.
- **`AdventureEnv`** (`h3-sim`'s `src/adventure/`, ADR-0017 point 8): built from a game
  ruleset, a shared `GameSetup` and a seed with a day limit (`max_days`, 0 none; done and
  truncated once day `max_days + 1` starts: increment 1 has no victory, so no reward either);
  `reset`, `to_move` (the turn group's players still acting, none once done), `legal_mask`,
  `tile_mask`, `step` and `step_with` (the unprojected events to a sink), `fork` (a clone:
  shared rules and setup, the state's persistent parts), `state_hash`, `observation`
  (`h3_game::observe`). A step decodes the action, applies its command through `h3_game::apply`
  for the trusted player and leaves the env unchanged when it is refused (`InvalidAction`,
  `Rejected`, `EpisodeOver`).
- **The action encoding**, version 1 (`ACTION_ENCODING_VERSION`), factorized as the battle's
  casts: entry 0 `EndTurn` (tile head 0), entry `1 + s` the move of the player's hero in slot `s`
  (its heroes by id, eight slots: the original's most heroes per player on the map) to the tile
  in the tile head, `(level × height + row) × width + column` (the observation's order). The
  masks are conditioned on the entry; a move's tiles are the hero's `legal_moves` but its own
  tile, and its command takes the query's path. Decoding needs no turn check (a move of another
  turn is `apply`'s `NotYourTurn`).
- **The random agent** (`rollout.rs`): a uniform legal entry, then a uniform tile of the hero's
  reach, from the AGENT stream; `random_rollout` never inlined (T9b's workload counts inside it).
- **The duel map** (`fixtures/maps/e08-duel-36.h3m`, 756 bytes): decision 11's map, SoD, 36 × 36
  with an underground, red and blue with two heroes each (one per level, their types' starting
  armies), terrain in 6 × 6 patches of every land terrain with some water and rock, a road per
  level, rivers on some patches, 72 blockers; laid out by T8b2a's `lay_out` from a plan of a
  fixed SplitMix64 generator (`tests/fixtures.rs`, whose test reads the committed file, checks
  it is still its plan's map and imports it; `H3_WRITE_FIXTURES=1` rewrites it).
- **The env's tests** (on the duel map): the start (red to move, its two heroes free, blue's
  masks empty); the encoding's arithmetic; random play of legal and arbitrary actions of any
  player in both profiles (16 cases of up to 59 steps): for every player each entry is legal
  exactly when its tile mask has a tile, a move's tiles are the hero's legal moves and a sample
  of them decodes to the query's paths (that every such path applies is T8b2a's property); the
  action played is accepted when legal and does what `apply` does with its command, an accepted
  one was legal, a refused one leaves hash and step count and emits nothing, with the error
  `apply` gives; forks (12 cases); determinism, the day limit and `reset`. `random_rollout`
  asserts in debug builds that each of its steps is accepted; without a day limit it needs a step
  limit.
- **Review** (an independent agent): no bug in the env; found and fixed: the fork test drew a
  rollout without a step or day limit in about one run in seven (it never ended); the turn group
  was not checked against the events; nothing showed that the variation changes anything (the
  unit test above); the module doc said `reset` takes a setup. Noted for T9b: the battle
  workload's `collect = "*random_rollout*"` matches the adventure's `random_rollout` too (both
  in `h3_sim`): the adventure workload needs a qualified pattern (inbox).

### Checkpoint after T8b2b/T9a and T9b1 + T9b2 (2026-10-11, `adventure/e08-t9b`)

The user merged T8b2b and T9a (#97, #98) and took (ADV-Q7) **T9b in two tasks with one checkpoint**: T9b1
the replays, T9b2 the benchmarks (T10 next round). Asked with the cut: the CLI's wiring in
battle-owned `h3-cli` files (`Cargo.toml`, `lib.rs`, `main.rs`): **user: `cross-lane`**; what a
replay records, the game commands (the acting player and `EndTurn` or `MoveHero { hero, path }`)
or the env's actions (entry and tile): after the pros and cons (commands are what the game
accepts from any source, a server's or a client's path included, and survive action-encoding
bumps and the hero slots' shifts; actions are compact and pin the env's path choice), **user:
the game commands**; which benchmarks are gated: **user: all three** (random play, path
searches, forks). The agent's own call: the commands live under `h3 adventure …`, which parses
its own arguments (the battle's `Args` stays private to `main.rs`).

**T9b1 — replays.**
- **The model** (`h3_sim::adventure::replay`, `REPLAY_FORMAT_VERSION` 1): a header (packs,
  profile, the core ruleset's and the adventure part's content hashes as ADR-0015 point 2 asks,
  the RNG and hash algorithms, the game and adventure state-hash versions; the engine version
  informational), the map as `MapRef` (path, the file's content hash — the kernel's FNV-1a over
  its bytes — and `h3_map::MAP_IMPORT_VERSION`, ADV-Q2; the path is informational, the hash
  names the file), the seed, the commands with the acting player, a checkpoint hash every N
  commands, the final day and hash. `record_random` plays the env's random agent (the same
  commands as `random_rollout`, tested); `verify` checks the header, the map, the checkpoint
  count, then re-applies every command through `h3_game::apply` (not the env: a server's log
  has no day limit) and compares the hashes and the day. `h3-sim` reads no maps (`arch.toml`
  allows `h3-map` there only as a dev-dependency): the caller loads the file and passes its
  `MapRef` and setup.
- **The env** gained `step_decoded` (the command an action decoded to, for the record; one
  query, as a step) and `step_command` (a game command, so a path of the player's own choice:
  tested with a detour the query would not pick).
- **The CLI** (`crates/h3-cli/src/adventure/`): `load_rules` (each pack's `pack.toml` and, where
  it exists, `adventure.toml`; pack ids checked as keys; the same hashes as the embedded packs,
  tested), `load_map` (a relative path without `..` below `--root`: a replay file is untrusted),
  the TOML format v1 (`deny_unknown_fields`; commands as `<player> end-turn` and
  `<player> move <hero> x,y,level …`), `h3 adventure run` (random agents, `--days` 28 by default,
  `--record`) and `verify`. `main.rs` hands everything after `adventure` to
  `h3_cli::adventure::cli::main`.
- **Golden replays** (`fixtures/replays/adventure/`): `duel-s1-d28` (seed 1, 28 days, 136
  commands), `duel-s2-d7` (42), `duel-s5-strict-d7` (strict-fixed, 46); re-run by
  `crates/h3-cli/tests/adventure_replays.rs`, which `cargo xtask determinism` already listed
  (debug, release, `target-cpu=native`).

**T9b2 — benchmarks.**
- **`h3 adventure bench`** (`bench.rs`): `--workload play` (inside `random_rollout`), `reach`
  (every hero's reachable-tiles query of the acting player at each step of random play, inside
  `reach_queries`; the play around it is not counted), `fork` (forks after `--days` of random
  play, inside `fork_workload`); `--observe 1` and `--masks 1` add what an agent reads before
  each step (wall time only).
- **The XL map** (`fixtures/maps/e08-xl-144.h3m`, 8 883 bytes): 144 × 144 with an underground,
  four heroes per player (two per level), the duel's patchwork and density (1 152 blockers).
  The generator became `patchwork(side, mix, blockers)`; the duel's draws and bytes are
  unchanged (its replays name it by hash).
- **Gated workloads** (`benches/adventure/`, callgrind Ir, +2 %): `random-play-duel-36` (5 games
  of 28 days) 510 210 757; `reach-xl-144` (2 games of 28 days, 1 920 queries) 657 381 227;
  `fork-xl-144` (100 000 forks after 7 days) 343 600 056. The adventure's pattern is qualified
  (`*adventure*random_rollout*`); the battle's `*random_rollout*` is unaffected (its run never
  calls the adventure's), so tightening it is hygiene only.
- **Measured** (release, the sandbox's Xeon vCPU, one core; wall time indicative):

  | Measure | Duel 36 × 36 × 2 | XL 144 × 144 × 2 |
  |---|---|---|
  | random play, one step (agent, queries, `apply`) | 70.9 µs (14.1 k steps/s), ≈ 745 k Ir | 183 µs (5.5 k steps/s), ≈ 1.6 M Ir (four heroes per player) |
  | + the acting player's observation before each step | +15 µs | +99 µs |
  | + the entry and tile masks before each step | +70 µs (×2.0) | +131 µs (×1.7) |
  | one reachable-tiles query | 27 µs, ≈ 263 k Ir | 33 µs, ≈ 342 k Ir |
  | one fork | 0.22 µs, ≈ 2 200 Ir (4 heroes) | 0.37 µs, ≈ 3 400 Ir (8 heroes) |
  | `GameState` / `AdventureEnv` size | 168 / 200 bytes | same |

- **The inbox's performance items, decided from the numbers:**
  - The masks query twice (`legal_mask`, then `tile_mask`, then the step's decode): an agent
    reading both heads doubles the duel's step. Worth caching the current state's queries in the
    env, but the mask API belongs to E05's batch API (or E13's macro-actions): queued there with
    the numbers, not built.
  - `observe` rebuilding every `TileView`: +21 % of a duel step, +54 % on XL. Queued for E13's
    flat encoding (cached static planes), not built.
  - The reveal's per-row writes: `fog::reveal` is 0.8 % of a random step: **closed**, no per-leaf
    batch.
  - New: the reachable-tiles query is 96 % of a random step (of a query: `Reach::search` 56 % on its
    own, the step price 28 % with `diagonal_cost`'s 16 %, the heap 7 %; zeroing map-sized arrays
    5 % of a step):
    optimizing it is the first lever (inbox).
  - Forks grow with the heroes (each hero's army is a `Vec`), not with the map: decision 6 holds.
- **Review** (an independent agent, reading the diff): no bug in record or verify; checkpoint
  math, refused steps, malformed input, determinism, the collect patterns and the duel map's
  bytes checked. Fixed: a plain env step cloned its command (now only `step_decoded` does;
  baselines re-taken, −0.02 %); `verify_file` read and parsed the map before comparing its hash,
  with no size cap and no check that a link under the root stays below it (now: canonical path
  inside the root, the reader's size limit before reading, the hash before parsing; tested,
  the link case on Unix); `record_random` cut a record short on a refused action in release
  builds and looped forever without a limit (now `RecordError::Refused` and `NoLimit`); no test
  refused a command of the player's own through `step_command` (now: an empty path, a far step,
  the hero's own tile, another player's turn, after the episode: hash, step count and events
  unchanged); `patchwork` asserts whole patches; `*.h3m binary` in `.gitattributes` (shared PR:
  a map is named by its bytes' hash). Kept: `verify` does not know an env's day limit, since a
  replay certifies game commands (a server's log has none).

### Checkpoint after T9b and T10a + T10b (2026-10-12, `adventure/e08-t10a`)

The user merged T9b (#103, #104) and took (ADV-Q8) **T10 in three parts**: T10a (the movement
experiments' spec shape and a batch) with T10b (the exit-criteria audit and the closure docs)
now, T10c (scoring the batch, refining increment 2, the checkpoint) next round. Asked with the
cut: the spec shape, **user: extend the battle lane's `h3-experiment` and `h3_formats::h3m` as
`cross-lane`** (not an adventure-owned compiler on `h3-map`'s test encoder, not waiting for
R01); the batch, **user: ADV-Q4, ADV-Q6 and D-2** (D-3 left out). Corrected the same day and told
to the user: D-2 is disputed only for a step off a road (the wiki: "Road ⇄ Terrain move — terrain
tile"; terrain to terrain all sources price by the tile left), so the batch needs roads too.

**T10a — the spec shape and batch 1.**
- **Writer** (`h3_formats::h3m`): `Patch` (a rectangle of another terrain, later ones on top),
  `Road` (straight, west to east, at least two tiles; `RoadKind`), and interior pictures for every
  terrain. Both read from the game's 164 maps through `h3-map` (a throwaway program): interior
  tiles (eight neighbours of the same terrain) use dirt 21–44, sand 0–23, the six other land
  terrains 49–72, water 21–32, rock 0–7, the first eight of each range about ten times as often;
  a straight east–west road 15 (west end), 12/13, 15 with the road mirror flag `0x10` (east end),
  over about 4 300 such tiles, a handful of exceptions aside (`docs/formats/h3m.md`). Between two
  terrains the writer puts interior pictures too (a hard edge in the game; movement reads only
  the codes).
- **Spec** (`h3-experiment`, `docs/experiments/README.md`): `[[map.patch]]`, `[[map.road]]`,
  a hero's `at`, `[[encounter.also]]` (more heroes per encounter, named as their type), `defaults
  = true` (army, experience, skills, artifacts, spells and primary skills the type's own; the
  writer leaves those fields out, as the editor does), table rows with a filled first column
  (`labels`); two objects never share a tile. `primary` and `army` are required only without
  `defaults`, so earlier specs compile unchanged. Tests: both shapes, every refusal; `h3-map`'s
  read-back test reads a patch, a road and a hero of its type's defaults.
- **Batch 1a** (`experiments/specs/E08-batch-1a.toml`, 72 × 72: grass, swamp over columns
  20–32, two dirt roads; scouts, heroes that never move, explore each walker's row past the
  farthest prediction so one click shows a path's marks): Q4a and Q4b (ADV-Q4: a battle in the
  other player's turn takes the Pikemen out of a Pikemen-and-Angels army, then the hero walks; 20
  steps if points are set at the owner's turn start, 15 at the day's; day 1 and day 2), Q4c (a
  dismissed Pikeman: 15 unless points follow every change of the army), D2a and D2b (off a road on
  grass onto swamp, on swamp onto grass: 9 and 14 by the reading and VCMI, 8 and 15 by the wiki),
  C0 (the land table: 15 on grass). **Batch 1b** (`E08-batch-1b.toml`, 36 × 36): sixteen heroes
  of their type's defaults, the nine war-machine heroes and seven whose three starting creatures
  differ, over up to eight loads (128 armies): the reading gives 3 % of armies their third slot
  without the second (none in 128: 2 %), VCMI never; the wiki never leaves out a war machine
  (12 % by the reading and VCMI). ADV-Q6's design (eight players, three loads) was cut to two
  players: the same armies, fewer turns to pass.
- **Checked:** the compiled maps through `h3-map`'s reader and import and the engine's
  reachable-tiles query (a throwaway run, seeds 1 and 2): D2a 9, D2b 14, C0 15, every Q4 walker 15
  before its battle; batch 1b's armies drawn as GAM-SET-001 says. **Review** (an agent that had
  not written the specs): the arithmetic and the hypotheses held; from it Q4c (recomputing on any
  army change would also give Q4a/Q4b 20), a path's last mark is a cross and counts, each path's
  expected length so a short click shows, saves to recover a battle, batch 1b's slots told apart by
  name (a missing second slot lets the third creature move up), the Catapult every hero carries,
  a fallback when Restart Scenario is not offered.
- **Delivered:** the maps in the user's `experiments/out/` (gitignored); the templates in
  `experiments/results/`. Scored in T10c.

**T10b — increment 1's exit criteria** (the epic's list, checked on 2026-10-12):

| Criterion | State |
|---|---|
| ADR drafts accepted and numbered | ADR-0015, -0016, -0017 (2026-10-10) |
| Full verify green in CI, adventure crates at 100 % | `master` d4e3aae green (run 38213824876, after #103/#104); the logic crates `h3-adventure` and `h3-game` are in the coverage gate; `h3-map`, `h3-pack-adventure` and `h3-sim`'s adventure path are adapters (`arch.toml`, not gated), tested by their own suites |
| Seven property tests, two copy-on-write properties | atomicity, legality consistency, ownership isolation (`h3-map` `play.rs`, random play on imported maps, both profiles), **conservation added here** (no source or sink of creatures in increment 1: every hero's army unchanged by an accepted command; before, only the other players' armies were checked), observation hygiene and the view against the events (`view.rs`), clone equivalence and determinism (`play.rs`; the env's own in `h3-sim`); copy-on-write in `h3-adventure`'s `persistent/tests.rs` |
| Golden replays in the determinism matrix | three on the duel map (`fixtures/replays/adventure/`), debug, release and native CPU; passed in this session's baseline verify |
| Benchmark baseline and measured targets in 02 §8 | three gated workloads (T9b) with targets met |
| Every ADV-*/GAM-* rule implemented with sources | 16 ADV and 4 GAM rules, all `implemented`, each with sources; ADV-MOV-002 (ADV-Q4) and GAM-SET-001 (ADV-Q6) rest on interim choices until batch 1 is scored; D-2 (ADV-MOV-003) on the user's choice |
| HANDOFF, epic log, QUEUE current | this round |

Open at T10c: scoring batches 1a and 1b (ADV-Q4, ADV-Q6, D-2's confirmation), the refinement of
increment 2 (scenario loading and validation, pickups, the first visitable objects; the epic's
sketch) with the queue's inbox sorted into it, the checkpoint that closes increment 1.

### Checkpoint after T10a/T10b and T10c: batch 1 scored, increment 1 closed, increment 2 refined (2026-10-12, `adventure/e08-t10c`)

The user merged T10a/T10b (#107), played batches 1a and 1b in SoD 4.0 Complete with plain HD and
pushed the filled templates. The round (ADV-Q9): **user: all of T10c** (the scoring, the
refinement, the closing checkpoint; one docs-only pull request, `cross-lane` for the two rows of
`docs/experiments/README.md`).

**T10c — batch 1 scored** (the specs' predictions against `experiments/results/E08-batch-1a.md`,
`-1b.md`):

| Encounter | Observed | Reading (engine) | VCMI | Wiki | Verdict |
|---|---|---|---|---|---|
| Q4a, day 1 | 2000 points left at Blue's turn start | 2000 | 1560 | 1560 | the reading; no first-day case |
| Q4b, day 2 | 2000 | 2000 | 1560 | 1560 | the reading (the decisive one) |
| Q4c (control) | not on the map played | 1560 | 1560 | — | the user's statement instead (below) |
| D2a, road on grass → swamp | 9 green, 5 red | 9 | 9 | 8 | the reading and VCMI |
| D2b, road on swamp → grass | 14 green, 2 red | 14 | 14 | 15 | the reading and VCMI |
| C0, grass | 15 green, 1 red | 15 | 15 | 15 | the land table |
| SA, 16 armies (load 1) | 123 ×4, 12 ×10, 13 ×1 (Crag Hack), 1 ×1 (Torosar) | 3.5, 10.6, 0.5, 1.4 | 13 never | war machine always | the reading |

- The user read the hero's points directly (the points left, asked and confirmed) instead of
  counting marks; 2000 means the Pikemen were dead (any speed-4 stack gives 1560), so Q4b's
  unfilled battle line went as planned.
- **Q4c was missing**: the map in the user's `experiments/out/` is a build from before the
  review added Q4c (read back with `h3-map`: 12 heroes, no Rion, no scouts B3 and B4); the
  template was regenerated after the review, the map not copied again. The user's statement
  covers the control: dismissing or gaining a stack does not change the day's points. Hence
  decision 18's stamp (a hash of the spec in the map's description and the template, from batch
  2) and the lane trap.
- **ADV-Q4 answered** (ADV-MOV-002 as built); **D-2 confirmed** (ADV-MOV-003); **ADV-Q6
  answered**: independent rolls, the war machine rolled at 88 % with its slot (the reading and
  VCMI; the wiki's 100 % refuted; on the map hero with increment 3). ADV-MOV-001 to -003 and
  GAM-SET-001 carry `observation` sources and are `verified`; the survey's D-1 and D-2 rows and
  §3.1 say so. No code changed.

**Increment 1 closed** with T10b's audit (above) and the interim choices now confirmed: every
criterion holds.

**Increment 2 refined** (the user, 2026-10-12): the scope as proposed (decisions 13–18 in the
epic): object types in the import, the treasury in `h3-game` (towns spend from it without
depending on the adventure), pickups (resource piles, random resources, campfires), the
**Cartographer** as the first pending decision (treasure chests moved to increment 3, where
experience and artifacts exist), the Redwood Observatory, Pillar of Fire, Keymaster's tents with
border guards and gates; starting resources by a difficulty level, the human row for every
player (default, the survey may overrule). Not taken: a smaller increment without visitable
objects, the week pulled forward, E09 first. Tasks T11a–T16 in the queue; the first round T11a
with T11b. Inbox sorted: D-3 into T16, the stream watch into T13, the partly seen object into
T11b/T12.

### Checkpoint after T10c and T11a + T11b: the objects survey, the ADR amendment drafts (2026-10-12, `adventure/e08-t11`)

The user merged T10c (#110) and took T11a with T11b in one round (one docs-only pull request,
one checkpoint), over T11a alone or T11a in halves.

**T11a — survey** `docs/rules/surveys/E08-objects.md`, from a code reading of the human player's
path and VCMI 1.7.5; the wiki's fetches first waited for the user's approval and timed out, then
the user approved them and the wiki was read the same day: it agrees with the reading wherever it
speaks (pile amounts, campfires, the Cartographer by terrain, the radius 20, keys, guards for
everyone, gates, the starting rows) and is silent on visit modes and random resources.
Findings:
- **Visit modes are per class in the original** (0x41B500's settings: "cannot be entered" and
  "visitable from above" lists): piles, random resources, campfires and border guards are
  visited from the neighbouring tile at the step's price (0x480240), the others entered; the
  gate is entered by key holders only. Objects not visitable from above (tents, Cartographers,
  observatories) are neither entered from nor left toward the three tiles above. A visit ends
  the move; only gates (and garrisons) let a key holder's move pass. VCMI agrees on all.
- **All random contents are drawn at setup** (0x4C9DD0 random resources, then 0x4C0CC0 amounts
  and campfires): no visit of increment 2 draws (decision 17's stream watch not needed yet).
- **D-5**: a random pile's amount, the reading and the wiki 5–10 (wood, ore, gold × 100) and 3–6
  (rare) against VCMI's 6–10 and 3–5; a batch-2 experiment optional.
- Campfires (4–6 of one of six resources plus 100 × that gold), random resources (seven equal
  chances), the Cartographer (once per player and kind, 1 000 gold checked before a yes/no,
  reveal by terrain mask on both levels: the land one reveals underground dirt and lava too),
  the observatories (radius 20 around the visited tile), keys, guards (removed for everyone
  after a key holder's yes) and gates: the reading and VCMI agree.
- **The treasury**: i32 per resource, a gain wraps and a negative result becomes 0, no cap:
  candidate **A-5** (state-corrupting, unreachable now; suggested always fixed).
- **Starting resources**: the human and computer rows by the game's difficulty (0x678170,
  0x6781FC), identical to VCMI's `difficulty.json`; decision 16 stands.
- **A second pending decision**: the border guard's yes/no (D-7); guarded piles need battles (D-6).
- A census of the 164 shipped maps (counts per class, 39 guarded bodies, 185 fixed amounts).
- A fresh agent re-read the deciding code without the survey and confirmed it (one caveat on
  flying and boats, inc. 3, added).
- 24 address-map entries (and `new_game_heroes` extended) and four record layouts (game, player record, map item, object
  settings); `check-exe-map` clean.

**T11b — ADR amendment drafts** in `docs/lanes/adventure/adr-drafts/`: ADR-0016 (types and
kinds from the ruleset part with the two visit properties, visits, contents resolved at the
build, the occupancy layer, per-player object state, objects shown whole once a covered tile is
explored) and ADR-0017 (the treasury in `GameState`, gains and costs by the context's outcome
with the price checked by the game, the decision queue and `Answer`, the env's answer entries,
the versions). Detail in `docs/epics/E08-design.md`.

For the user (ADV-Q10): accept the drafts; D-5, D-6, D-7, A-5; the next round.

### Checkpoint after T11a + T11b and T12a + T12b: object types (2026-10-12, `adventure/e08-t12`)

The user merged #113 and answered ADV-Q10: both ADR amendments accepted as written (applied by
`shared/e08-inc2-adrs`, #115, with `BACKLOG.md`'s E08 row and the glossary); D-5 the reading
with the wiki; D-6 guarded piles `Unsupported`; D-7 the border guard's question a second offer;
A-5 always fixed; the round T12 in two parts, T12a and T12b, one pull request.

**T12a — object types as data and in the import.**
- **Ruleset part** (`h3-adventure`, `src/ruleset/objects.rs`; ruleset-part hash v4): `ObjectType`
  (key, name, `Visit::{Enter, Neighbour}`, `from_above`, `ObjectKind`), `ObjectTypeId`, by key.
  Kinds: `Pickup(Pickup::{Resource, OneOf, Paired})` (a pile: a resource, the bounds of a drawn
  amount and a multiplier; a random resource: one of these types; a campfire: an amount, one of
  these resources and the paired resource times a multiplier), `RevealTerrain { terrains,
  price }`, `RevealArea { radius }`, `KeyTent`, `KeyGuard`, `KeyGate` (a colour below
  `KEY_COLOURS` = 8). Compiled per pack in two passes (keys first, so a type may name a later
  type of its own pack); references stay in the pack's dependency closure; a check after every
  pack that `OneOf` names resource pickups only. Bounds: an amount from 1 to
  `MAX_PICKUP_AMOUNT` (0x7FFFF, the original's 19 bits), `MAX_PICKUP_AMOUNT` × the multiplier
  within `MAX_GAIN` (2³¹ − 1, the treasury's cap), prices from 1 to `MAX_GAIN`, radius ≥ 1.
- **Packs** (`h3-pack-adventure`: `[[object_type]]` with one kind table: `pickup`,
  `pickup_one_of`, `pickup_paired`, `reveal_terrain`, `reveal_area`, `key_tent`, `key_guard`,
  `key_gate`): the base pack's 30 types (seven piles, the random resource, the campfire, three
  Cartographers, the two observatories, eight tents and eight guards) and AB's eight border gates,
  each with its sources.
- **Setup** (`ObjectSetup`): `object_type: Option<ObjectTypeId>` (none: a blocker) and `amount`
  (a pile's or random resource's fixed amount, 1 to 0x7FFFF); `SetupError::{UnknownObjectType,
  ObjectAmount}`; the static map keeps both (adventure state hash v4).
- **Import** (`MAP_IMPORT_VERSION` 2, `h3m.md` "Import"): `codes::object_type(class,
  subclass)`, a subclass the class does not define is `ImportError::Subclass`; a resource body's
  amount (0 none); a pile or random resource with a guards record is
  `Unsupported::GuardedPile` (D-6; even an empty record, as the reading of the loader is
  missing), a blocker in `import_map`. The shipped maps: every static layer still imports and
  validates; 9 import whole (`Myth and Legend` now has a guarded pile), 5 are unsupported for a
  guarded pile.
- **Rules**: ADV-OBJ-002 `implemented`; ADV-PIK-001 to -003 and ADV-VIS-001 to -005 written
  `specified` with their sources (data in the packs, tests on the data).
- **Golden replays** re-recorded for the new ruleset-part hash, import version and state hash
  (the commands are unchanged; the record options are in the test's doc comment).
- The ADR drafts left `adr-drafts/` (applied by #115).

**T12b — objects in the state.**
- **Contents drawn at the build** (ADV-PIK-001 to -003 `implemented`): `AdventureState::new`
  takes the setup stream (`h3_game` passes it after drawing the starting armies, GAM-SET-001's
  order, tested in `h3-game`); the original's two walks in object order: each random resource's
  pile type (`below(n)`), then each pile's amount where the map fixes none and each paired
  pickup's amount then resource. `MapObject.contents` (resource, amount × multiplier) replaces
  T12a's amount; `MapObject` is no longer `Copy`. Every draw is valid, so validity does not depend
  on the seed. A test replays the draws by hand for 50 seeds.
- **Removed objects and the occupancy layer** (`src/state/occupancy.rs`, ADR-0016 point 1): a
  `BitGrid` of removed objects (32 a row; `SetupError::Objects` beyond 32 × 65 535) and two
  `BitGrid`s of freed tiles (blocked, visitable), held as `Option<Arc<Occupancy>>`, none until an
  object leaves: a fork costs one reference more at most, and a tile is read through it only
  where the static tile is marked. `check_tile` (ADV-MAP-001, so `MoveHero`), the reachable-tiles
  query and the observation's tiles read the live tile. Measured (`bench-check`, valgrind): read
  on every tile of the query it cost +4.1 % on random play and the query; as built +0.35 % (fork),
  +0.86 % (random play), +0.91 % (query), under the 2 % gate, so no re-baseline. `AdventureState::remove_object` frees each tile no other object still on the
  map covers so (tested with overlapping objects); its first caller is T13 (an `expect(dead_code)`
  outside tests until then). The state hash covers the removed objects (the freed tiles follow).
- **Observation**: `AdventureObservation.objects`, an `ObjectView` (type, anchor, masks) per typed
  object still on the map with a covered tile the viewer sees (explored, or the fringe under
  `ADV-QUIRK-1`), in map order. Not the object's index: it counts the objects before it, seen or
  not (the hygiene property found that a removal out of sight would shift it). Blockers are
  left out (the tiles show them).
- **Play properties** (`h3-map`'s `play.rs`): plans lay out typed objects too (a wood pile of a
  drawn amount, 300 gold, a campfire, a Redwood Observatory, a Keymaster's tent; no random
  resource, whose drawn type shows), so atomicity, legality, isolation, hygiene (other seeds draw
  other contents) and determinism run with them; a fixed test checks a pile and a campfire in sight
  show the same for 20 seeds whose contents differ. The fixtures' maps are unchanged.
- The static layer check of the shipped maps builds a one-player game (`GameSetup::validate`),
  so it draws contents as a game does; still every static layer validates.
- Golden replays re-recorded again (state hash v4's final encoding); commands unchanged.


### Checkpoint after T12a + T12b and T13a + T13b: the treasury and pickups (2026-10-12, `adventure/e08-t13`)

The user merged #123 and accepted T12b's two calls (no object index in what a player sees; any
guards record makes a pile `Unsupported`). The round: T13 in two parts on one pull request, the
entered visit and ADV-OBJ-004 moved to T14 (building them now would leave a Cartographer,
observatory or tent entered with no effect, or code no test reaches); the viewer rows of
`docs/06-security.md` in `shared/e08-inc2-viewer-rows` (#125, docs only; BACKLOG's E08 row was
already current).

**T13a — treasuries.**
- **Starting resources** (GAM-RES-002): `[[difficulty]]` in `adventure.toml`, a row per difficulty
  from 0 (easy), each a human and a computer map of amounts by resource (one left out is 0);
  exactly one enabled pack gives them (`MissingDifficulties`, `DuplicateDifficulties`,
  `InvalidDifficulty`: an amount above 2³¹ − 1, a resource twice; at most 256 rows). Compiled in
  `h3-adventure` (`ruleset/starting.rs`: `StartingResources`, dense by `ResourceId`) beside
  GAM-SET-001's chances, read by `h3-game` through the part `GameRuleset` bundles (agent's call,
  change freely: ADR-0017 point 1's "compiled into `GameRuleset`" without a pack part of its
  own). Ruleset-part hash v5. The base pack's ten rows from the code reading, VCMI and the wiki
  (a test pins them; the first draft had ore at the rare amount: "wood, ore" in the survey's table
  is both).
- **Treasury** (GAM-RES-001; `GameState::treasury(player)`, a slice by resource): never
  negative, capped at `MAX_GAIN` (2³¹ − 1; `A-5` fixed). Every player's in one block behind an
  `Arc` (`treasury.rs`, crate-private `Treasuries`): a block per player cost a fork an allocation
  per player (+13.6 % on `fork-xl-144`), the shared block costs it a reference count (+1.0 %); a
  gain copies it once. `GameSetup.difficulty` (`DEFAULT_DIFFICULTY` = 1, normal;
  `GameSetupError::Difficulty`, checked first); every player gets the human row (decision 16).
  Game state hash v2 (the treasuries); `GameState` 168 → 192 bytes. `GameObservation.treasury`:
  the viewer's own, empty for a viewer not in the game.
- **Import** (`MAP_IMPORT_VERSION` 3, `h3m.md`): the setup's difficulty is the default; the
  header's difficulty byte is the map's rating.
- Golden replays re-recorded (hashes only; the commands unchanged).

**T13b — neighbour visits and pickups.**
- **`MoveHero`** (ADV-OBJ-003, the neighbour half): a step onto a live visitable tile looks up
  the visited object (`StaticMap::visited_from`, a per-tile list built at setup; the first object
  in map order still on the map, VCMI's `blockingVisit` order; agent's call). A pickup visited
  from a neighbour: the step is priced as if taken (ADV-MOV-006's drop to 0 from that tile), the
  hero stays, and the step must be the path's last (`PastVisit`). Any other visitable tile (a
  blocker's, an entered object's, a guard's) keeps `Visit`, now "no visit is built".
- **Outcome** (ADR-0017 point 10): `AdventureState::apply` returns `AdventureOutcome { gains }`
  (the object's contents, in rule order); `h3_game::apply` adds them to the actor's treasury
  (`Treasury::gain`, saturating) and emits `GameEvent::ResourceGained { player, resource, amount }`
  per resource with the amount actually added, projected to its owner only.
- **Removal** (ADV-OBJ-005): `remove_object` is live; `AdventureEventKind::ObjectRemoved
  { object_type, anchor }`, seen by the players that see the object as the observation shows it
  (an explored covered tile, or the fringe under `ADV-QUIRK-1`).
- **Query**: a pickup's tile is a destination with the points kept after the visit, never
  expanded; the env's tile mask offers it (action encoding unchanged, v1).
- **Properties** (`h3-map`): moves that end in a visit (the hero on the tile before; one event per
  tile, the visit's last); conservation of treasuries (each changes by exactly the gains it was
  told, which come with a removal); isolation over treasuries; the views against the events with
  removals (a removed object leaves the view only when told; its tiles may thin) and gains. The
  hygiene property found that a pickup tells its visitor the hidden contents (survey §8): two
  games of other seeds then tell it other gains, so the comparison stops after the viewer's own
  pickup (the shrunk case kept as a regression). The query's property runs on worlds with piles;
  fixed tests for a pile, a fire, a visit from the hero's tile, the drop to 0, `PastVisit`, the
  untyped and not-yet-built tiles, a gain through the game and the cap. Random play visits a few
  pickups per run of 48 cases (counted once).
- Golden replays re-recorded again for the treasuries' place in the hash (the shared block); the
  commands unchanged (the duel map has no pickups).
- **Measured** (`bench-check`, valgrind): after the treasury block, the query still cost +4.3 %
  and random play +4.6 %. A profile against `master` showed `Reach::search` +1.6 % (the visit
  test; a bit per tile, `StaticMap::visited_from`, now turns a blocker's visitable tile down
  before the per-tile lookup) and `diagonal_cost` +14 % because `isqrt` was no longer inlined
  into it (16 codegen units, thin LTO: an artifact of the partition). `diagonal_cost` is now
  ⌊straight · ⌊√2 · 2⁶⁴⌋ / 2⁶⁴⌋, one 128-bit multiplication, equal to `isqrt(2 · straight²)` for
  every cost up to `u16::MAX` (an exhaustive test): results unchanged, and the query and random
  play run 13.8 % fewer instructions than before T13. Re-baselined: `reach-xl-144` 657 381 227 →
  566 626 021 Ir, `random-play-duel-36` 510 210 757 → 439 577 005 Ir; `fork-xl-144` +1.0 %
  (343 600 056 → 347 100 056, under the gate, not re-baselined). `docs/02-architecture.md` §8's
  numbers follow in #125 (a shared path).

### Checkpoint after T13a + T13b and T14a + T14b: the Cartographer (2026-10-12, `adventure/e08-t14`)

The user merged #125 and #132 and accepted T13's four calls: the starting rows in the adventure
part; the neighbour visit takes the first object in map order (T14a may revisit it: if the
original visits one order in both modes, the neighbour visit follows it, flagged at the next
checkpoint); the diagonal cost in one multiplication with the two re-baselines (asked first what
changed: only the computation, equal for every cost); every game at normal difficulty until T14b
adds a difficulty option with the replay format's bump (the user: yes). The round: T14 in two
parts on one pull request, T14a the entered visit and ADV-OBJ-004, T14b the Cartographer's offer
end to end with the env's action encoding v2 and the replay format v2.

**T14a — the entered visit and not from above.**
- **The entered visit** (ADV-OBJ-003's other half): a step onto the visitable tile of an object
  whose type is entered (`Visit::Enter`) and whose visit is built enters it: priced as a step
  (ADV-MOV-003 to -006, the drop to 0 included), its step event and reveal as any step's, the hero
  stands there, and the move ends (`PastVisit` after it). The mode follows the type, the effect
  the kind: `ObjectKind::visit_built` (pickups and, from T14a, `RevealTerrain`) is the one place
  T14b and T15 extend, and `visit` dispatches on the kind (the Cartographer: nothing yet, no
  outcome, no event). The observatories, tents, guards, gates and blockers keep `Visit`.
- **ADV-OBJ-004**: an object whose type is not `from_above` closes the three steps across its
  visitable tile's top, both ways (down onto it, up off it; walking: flying is increment 3, no
  flag). `AdventureCommandError::FromAbove { step }`, checked after `Blocked` and before `Visit`.
  Applied to every typed object by its setting, not only the entered ones (agent's call: the
  original tests the class's flag; every neighbour-visited class is on its list, so the packs
  change nothing); an untyped blocker has no setting and closes nothing (a town's gate stays
  leavable upward until towns, E09).
- **The query**: an entered object's tile is a destination with the points left on it, never
  expanded, never reached from above; from the hero's own tile, when it closes upward, the three
  steps up are skipped in the start's own expansion, peeled off the loop (`expand::<START>`,
  inlined) so the hot loop has no test for it.
- **Which object a tile visits** (T13's call 2 revisited with the user's leave; flagged for the
  checkpoint): the original keeps one object per map tile (class, subclass, index; 0x505610 takes
  the topmost object of the tile's draw list visited from it, the objects placed in map order at
  load, 0x4FD690, 0x505B20), and the step, the visit and the test from above read that one (survey
  §2.5). The engine takes the last object in map order still on the map, for both modes and for
  ADV-OBJ-004; T13b's neighbour visit took the first (VCMI's `blockingVisit`; VCMI's entered visit,
  `visitObjectOnTile`, takes the last). Not read: the draw order's equal-layer and lower-layer
  cases (0x505230), which can put a later object under an earlier one.
- **Setup** (the inbox's case from T13b's review; agent's call): `check_stand` refuses a tile
  whose object (the one a visit concerns) is visited from a neighbour, built or not
  (`StandError::Visited`): no move enters such a tile. A hero on an entered object's tile, or a
  blocker's (a town's gate), stays allowed; ADV-OBJ-004 applies when it leaves. The import is
  unchanged (such a map imports and fails validation; none does), `MAP_IMPORT_VERSION` stays 3.
- **Shipped maps** (`original_data` counts stacked visitable tiles now): none of the 164 has a
  tile visited from two or more objects (0 of 101 246 visitable tiles of the static layers, heroes
  left out); the import's outcomes are unchanged (9 imported and valid; 129, 18, 5 and 3
  unsupported).
- **Properties**: a land Cartographer is the play properties' seventh kind (atomicity, legality
  consistency, conservation, isolation, hygiene, clone equivalence and determinism hold with it);
  random play entered a Cartographer 3 times in a run of 48 cases (counted once), and every later
  step checks the query from its tile against `MoveHero`. The query property's maps have charts
  of both settings and no hero on a pile. Fixed tests: entering (price, drop to 0, reveal,
  `PastVisit`), not from above both ways, from-above types unaffected (an open chart, a pile), the
  order (a pile and a chart stacked both ways, a removal, a blocker on top, the test from above
  by the last object), the setup's refusal.
- **Versions**: none bumped (no state, import or ruleset-part change; the charts of the tests are
  test rules). Golden replays verify unchanged.
- **Measured** (`bench-check`): the first build cost the query +1.03 % and random play +1.09 %;
  the extra Ir were all in `Reach::search`'s own code, not the new calls (a variant with the old
  loop and the new visit test cost the same). The test from above in the visit branch kept the
  step's row (`dy`) live across the loop; testing the direction (3 to 5, live anyway) instead
  removed it: `reach-xl-144` 565 798 491 Ir (−0.15 %), `random-play-duel-36` 438 912 898 (−0.15 %),
  `fork-xl-144` +1.02 % as before T14a. No re-baseline.

**T14b, first half — the Cartographer's offer as the first pending decision.**
- **The visit** (ADV-VIS-001): entering a Cartographer whose type the visitor's owner has not
  bought makes an offer, `AdventureOutcome.offer`: `Offer { hero, object_type, anchor, price }`,
  the object by type and anchor (inbox T12b), so the offer is also what its owner sees. A type
  already bought offers nothing (no event: the original's message is not modelled).
- **The decision** (GAM-DEC-001, ADR-0017 points 6 and 10): `h3_game` opens the offer as the
  actor's pending decision if its treasury can pay (`OfferOpened`), else `OfferRefused`; with one
  open, `legal_commands` lists `Answer(Accept)` and `Answer(Decline)` only, `legal_moves` gives
  nothing, and `EndTurn` or a move is rejected (`DecisionOpen`, after `NotYourTurn`).
  `GameCommand::Answer` closes the actor's first decision, in its turn or not (`UnknownPlayer`,
  then `NoDecision`): `OfferAnswered { accepted }`, and accepted, `ResourcePaid` per resource and
  `AdventureState::accept_offer` (infallible): every tile of the type's terrains on every level
  explored for the owner, `TilesRevealed { object_type, count }`, the type bought. All of them are
  told to the owner alone. A decision blocks no one else (two players of one group, a decision
  each, each answers its own). Agent's call (the brief): ADR-0017 point 10's checks at the answer
  (the treasury can pay, the offer still stands) cannot fail in increment 2 and are not made, no
  dead branch: nothing of the owner's but its answer runs while the decision is open, and no other
  player's command reaches its treasury or purchases; the play properties apply every listed
  answer at every step. T15's guard makes the second check reachable.
- **State** (agent's calls): the purchases in the adventure state as the original's flags, a
  byte of player bits per type in type order; the decisions in the game state in opening order.
  Both are `None` until the first, then one block the forks share (never an empty list): a list
  per fork cost `fork-xl-144` +2.65 % over T14a (356 300 056 Ir), a decision per player +2.94 %
  (the players' clone and drop lost their plain copy); now 349 100 056 (+0.58 % over T14a, +1.60 %
  on the baseline, not re-baselined). Query and random play unchanged (−0.14 %, −0.05 %).
  Adventure state hash v5, game state hash v3; golden replays re-recorded (hashes only).
- **Observation**: `AdventureObservation.bought` (the viewer's own purchases) and
  `GameObservation.decision` (its first open decision, an `Offer`, price included); other
  players' never show.
- **Elsewhere**: the CLI's replay text reads and writes `<player> accept` and `<player> decline`
  (format v1 still: nothing records an answer yet). The env is unchanged: with a decision open,
  action encoding v1's masks are empty, and `random_rollout` would stop at an offer (no fixture map
  has a Cartographer); encoding v2, the replay format v2 with the difficulty option and the
  golden replays are the second half's.
- **Properties**: plans draw a visitable land Cartographer for a quarter of their objects and a
  difficulty (impossible, with no gold, in half the cases, so offers are refused until pickups
  pay; the tests take the map's difficulty); random play gains `Visit` (a legal move onto or
  toward a visitable tile) and arbitrary answers. Legality with decisions (the answers only, each
  applies, no moves), `check_decisions` (an offer opens exactly when affordable, closes by its
  answer, and only an accepted one buys), conservation with payments (exactly the answered
  offer's price), isolation over purchases and decisions, the views against the events
  (decision, purchases, payments, reveal counts). The hygiene property stops after the viewer's
  own accepted offer (a reveal by terrain shows hidden tiles' terrain); an offer opened, refused
  or declined shows nothing hidden, so the comparison goes on. With the visit steps it found
  another player's move toward a pile the viewer never sees: the other game lacks the pile, and
  the hero walks on in the viewer's sight where it stayed. Like a move one hero can pay and the
  other cannot, an outcome that differs between the games now stops the comparison (the shrunk
  case kept as a regression): after the review, the command's effect (accepted, each of the
  actor's heroes' displacement, the steps told, the decisions), not where the heroes stand, which
  the `moved` variation changes from the start (600 cases: 14 such stops, none on an `EndTurn`,
  against 26 by absolute tiles, 21 of them on tiles that differed before). Over 48 cases (four runs,
  counted once): 9 to 16 offers opened, 13 or 14 refused, 4 or 5 accepted, 4 to 10 declined.

**T14b, second half — the env's answers and the game's difficulty in replays.**
- **Action encoding v2** (ADR-0017 point 8): entries 0 `EndTurn`, 1–8 the hero slots, 9
  `Accept`, 10 `Decline` (`ACCEPT`, `DECLINE`; `ENTRY_COUNT` 11). After a legal entry that takes
  no tile (`EndTurn`, an answer) the tile head allows 0 alone, and such an entry decodes with tile
  0 only (v1's rule for `EndTurn`, the battle's for non-cast entries); an illegal entry's tile mask
  is empty. `legal_mask` sets the entries of `legal_commands`' commands and the slots with a legal
  move, so with a decision open the answers are the only legal entries, each with tile 0 alone.
  The review (runner's call, for the checkpoint): the first build ignored the tile head of these
  entries with an empty mask, as the design note said, which gives a policy that always samples
  the head a fully masked categorical (NaN) on exactly the entries a decision forces; reverted
  before release, still v2. `action::encode` names a command as an action (the hero's slot, the
  path's last tile); `decode` gives it back on the query's paths, another path to the same tile
  becomes the query's. No C ABI (E05).
- **The random agent** (`choose`): its entries are `legal_commands`' commands and the slots with
  a move, one drawn uniformly as before, so it answers an open decision at even odds from its own
  stream (agent's call: the uniform rule unchanged) and, without decisions, draws the same numbers
  as v1 (the golden replays' commands and hashes are unchanged). `random_rollout`, the record and
  the bench's rollouts play through offers.
- **The difficulty** (the user, 2026-10-12): `AdventureEnv::reset_with_difficulty(seed,
  difficulty)`, which later resets keep (`reset(seed)` keeps the last; a new env plays the
  setup's, a map's import `DEFAULT_DIFFICULTY`), and `difficulty()`; the setup is copied only
  when the difficulty changes. An unknown one is the game's build error,
  `GameSetupError::Difficulty`, and leaves the env as it was: the difficulty is setup input,
  checked as the rest of the setup. `h3 adventure run` and `bench` take `--difficulty N`
  (default 1; a `u8`, an unknown row the setup's error); `verify` plays at the replay's.
- **Replay format v2**: `Replay.difficulty`, beside the seed (agent's call: setup input like the
  seed, not part of `ReplayHeader`, which `check` compares with the build; in the file the
  `[replay]` table's `difficulty = n`, always written; the review: a file of format v2 or later
  without it is malformed, `Format`, not a game at normal difficulty). `record_random` records the setup's,
  `verify` builds the game at it whatever the map's import set (an unknown one is
  `ReplayError::Setup`). The answers were already written (`<player> accept|decline`). Agent's call
  (v1 files): as every battle format bump, a replay of another format version is refused,
  `Incompatible("format_version")`; a v1 file has no difficulty and parses as normal (the battle
  files' `serde(default)` for added fields), so the refusal is that typed one, given before the
  map is read.
- **Tests** (h3-sim): the duel with a land Cartographer near each hero, built in the tests (no
  fixture map has one): on the nearest tile the hero reaches on day 1 by a last step not from
  above. Fixed: the offer opened through `step`, the entry mask 9 and 10 only and no tile,
  `EndTurn` and moves refused (`DecisionOpen`), the other player's answer `NoDecision`; a fork
  during the decision declines while its parent accepts (payment, purchase, masks back); no
  decision at impossible difficulty; the agent answers both ways and a rollout plays on; an
  answer with another tile is refused (`InvalidAction`); encode
  and decode round-trip; the reset's difficulty; a replay with answers at easy difficulty
  verifies against the import at normal (normal is another game, 9 the setup's error); the
  random record keeps its difficulty. The env's property plays this map at any difficulty with a
  pick that enters a Cartographer (26 steps with an open decision over its 16 cases): the masks
  against `legal_commands` and `legal_moves`, every masked-in action steps as `apply` does with
  its command, every masked-out one is refused and changes nothing. h3-cli: the option and its
  errors, the difficulty and answers through the file, the v1 file's refusal, a v2 file without
  its difficulty. Line coverage of
  both sub-paths: everything but the record's refusal closure (a bug path since T9b).
- **Versions**: action encoding v2, adventure replay format v2. Increment 2's versions through
  T14: `MAP_IMPORT_VERSION` 3, adventure ruleset-part hash v5, adventure state hash v5, game state
  hash v3, action encoding v2, adventure replay format v2. Golden replays re-recorded (the format
  line and the difficulty only) and a fourth, `duel-s3-easy-d7` (`--difficulty 0`: the duel has
  no objects, so the difficulty shows in the treasuries).
- **Measured** (`bench-check`): `reach-xl-144` 565 763 181 Ir (−0.15 %), `random-play-duel-36`
  438 920 743 (−0.15 %; +0.002 % over T14a's), `fork-xl-144` 349 100 056 (+1.60 %, unchanged
  since the first half). No re-baseline.

**Review and closure of the round** (runner).
- **Independent review** (an agent that had not seen the work; read the diff, ran checks in a
  worktree): no bug in the engine. Checked and passed: atomicity of every rejection path, the
  masks and `legal_*` against `apply` (decision open, outside the turn, other players), the
  from-above test in the query against `MoveHero`, one visit order everywhere (`tile_object`),
  copy-on-write of the decisions and purchases (`None` and empty hash alike), the reveal against
  a brute-force reference on four two-level maps, replay v2's difficulty on verify. Three
  findings, all fixed: the hygiene stop compared absolute tiles, which the property's own
  `moved` variation makes differ (26 stops at 600 cases, 21 of them spurious; now 14, 2); the
  tile mask was empty after `EndTurn` and the answers, a NaN for a masked softmax (tile 0 again,
  runner's call); a v2 replay without its difficulty read as normal (now a format error).
  Residual risks to the inbox (`accept_offer` trusts its offer; T15's gate and the mode).
- **The full verify's coverage run** drew a case T13b's view property models wrongly (not the
  engine): two campfires of other shapes on one anchor share the name a viewer sees (type and
  anchor), so after the visited one left, told by that name, the other still showed it; plans now
  lay out one typed object per kind and anchor (the shrunk case kept as a regression).
- **The full local verify** (CI out of minutes) with `--with-original-data` passed before the
  review's fixes; again on the branch's head before the pull request (its result there).

### Checkpoint after T14a + T14b and T15a + T15b: observatories, keys, guards and gates (2026-10-13, `adventure/e08-t15`)

The user merged #142, #143 and #145 (the easy golden replay's ruleset hash after R01b) and
accepted all of T14's calls: one object per tile, the last live one in map order, for both modes,
and setup's refusal of a hero on a neighbour-visited object's tile; tile 0 for the entries without
a tile; v1 replays refused, v2 needing the difficulty; no `CannotPay` at the answer; no event for a
kind already bought; one typed object per type and anchor in the play maps. The round: T15 in two
parts on one pull request, **T15a** the observatories, Keymaster's tents and border guards with the
second offer, **T15b** the border gates alone (the only change to movement and the query, so its
benchmark check is its own). The merged branches `adventure/e08-t14` and
`adventure/e08-easy-replay-hash` stay on GitHub (the user). CI still cannot start (GitHub: failed
payments or the spending limit), so the full local verify stands in again.

**T15a — observatories, Keymaster's tents, border guards with the second offer** (a worker).
- **Built kinds** (ADR-0016 point 4): `ObjectKind::visit_built` became `ObjectKind::effect`, an
  `Effect` per kind whose visit is built (pickup, Cartographer, observatory, tent, guard; the gate
  `None`, T15b's), carried by the step's visit, so `visit` dispatches on it with no dead arm (agent's
  call). The data and the import were already T12a's (`reveal_area = { radius = 20 }`, the tents
  and guards by colour): no ruleset-part or import change.
- **Observatories** (ADV-VIS-002): entered, not from above; after the step's own reveal, the disk of
  the type's radius around the visited tile on its level (`fog::reveal`) for the owner and
  `TilesRevealed { object_type, count }` to it alone, at every visit, also with 0 new tiles (agent's
  call: every visit tells).
- **Keys** (ADV-VIS-003): entered, not from above; the colour's key for the owner, `KeyGained { key }`
  to it alone; a held key gives and tells nothing (as T14's call 5). State as the original's, a byte
  of player bits per colour (+0x4E364), in one block with the purchases (`state/flags.rs`: `Flags`
  behind one `Option<Arc>`, none until the first purchase or key; agent's call: one field instead of
  a second, the fork pays nothing more). `AdventureState::{has_key, keys}`,
  `AdventureObservation::keys` (the viewer's own, a bit per colour). Adventure state hash v6 (the
  keys after the purchases, eight bytes; equal content hashes equal before the first write).
- **Border guards** (ADV-VIS-004): visited from a neighbour; without the key nothing and no event
  (flagged: the original's message is not modelled); with it an offer, `Offer { hero, object_type,
  anchor, kind }` with `OfferKind::Reveal { price }` (the Cartographer's) or `OfferKind::Pass` (no
  price), `Offer::price()` (agent's call: the price moved into the kind). `h3_game` opens it as the
  owner's decision (no price, never refused). Accepted: `accept_offer` (now with the core, for the
  viewers) takes the guard off the map, `ObjectRemoved` to the players that see it, its tiles freed.
- **The offer still stands** (ADR-0017 point 10): `AdventureState::offer_stands`, its object still on
  the map, the last in map order with the offer's type and anchor (agent's call: one test for both
  kinds; a Cartographer never leaves; the 164 shipped maps have no two typed objects of one type on
  one anchor, below). The game checks it at an acceptance: otherwise `GameCommandError::OfferGone`,
  nothing changes, and `legal_commands` lists `Decline` alone (agent's call: the new variant). The
  stale case needs two players of one turn group, so only `h3-game`'s tests reach it (the
  test-only `set_schedule`): both take the key and visit the guard, the first passes, the second's
  acceptance is refused and its decline closes the decision. Game state hash v4 (a decision's offer
  kind, 0 a reveal with its price, 1 a passage).
- **Inventory** (ADR-0017 point 7) in the epic file: the objects two players' commands can remove (a
  pickup, a border guard); purchases and keys only by their owner's.
- **Shipped maps** (`original_data`, the inbox's count): typed objects of one type on one anchor,
  each after the first: **0** in the 164 maps (and 0 of 101 246 stacked visitable tiles, as before).
- **Properties** (`h3-map`): the red border guard is the plans' eighth kind; plans draw observatories,
  red tents and red guards of a visitable shape more often and, in three cases of four, put a red
  tent within three tiles of the first hero with a red guard visited from two columns east of it.
  `landing` reads a neighbour visit from the steps told (a step fewer, the last tile visitable, at
  most one visit event); `check_decisions` checks both kinds (a guard's offer only to a key holder,
  an accepted one removes the guard, an accepted Cartographer's buys) and the keys (only by
  `KeyGained`, each new); isolation over keys; the views against the events with keys and the kinds
  of accepted offers. Hygiene: the comparison goes on after the viewer's own observatory visit
  (agent's call, against the brief's stop: the disk is the same tiles in both games, and the
  property compared 3 to 5 such visits per run of 48 cases, none differing). Counted over 48 cases
  (temporary counters, three runs of the final plans): 17 to 28 keys taken, 8 to 15 observatory
  visits, 3 to 7 guard offers, 2 to 4 accepted, 0 to 4 declined; stale answers 0 (sequential play
  cannot make one). The
  query's property (`h3-adventure`) has towers, tents, guards and gates. Fixed tests: the disk and
  its count (both levels, a revisit with 0), the key once per player (its event, the observation,
  equal keys hashing equal in either order), the guard without and with the key, the acceptance and
  a stale acceptance's no-op, a gate still `Visit`; through the game: the guard's offer without a
  price, its decline and acceptance, the stale case.
- **Versions**: adventure state hash v6, game state hash v4; the ruleset-part hash (v5), the import
  (v3), the action encoding (v2) and the replay format (v2) unchanged. Golden replays re-recorded
  (hashes and versions only; their commands unchanged).
- **Measured** (`bench-check`): `fork-xl-144` 349 000 056 Ir (+1.57 % on the baseline, −0.03 % on
  T14b's: the flags' thin pointer), `random-play-duel-36` 437 705 073 (−0.43 %), `reach-xl-144`
  565 723 829 (−0.16 %). No re-baseline.

**T15b — border gates and the per-player visit mode** (a worker).
- **The gate** (ADV-VIS-005; `ObjectKind::effect` gives `Effect::Gate(key)`, so every typed kind's
  visit is built and `Visit` is a blocker's alone): a player holding the gate's key passes it, its
  tile crossed like a free tile (the step does not visit, the move goes on, a path may end there;
  the original's 0x480EE2, 0x48160A and the pathfinder's 0x56AB40, VCMI's
  `MovementAfterDestinationRule`); any other player visits it from the neighbouring tile as a
  pile (the step paid, D-3's drop to 0 included, the hero stays, the move ends) and nothing else
  happens, no event (flagged as T15a's guard without the key: the original's message is not
  modelled). Never removed; visited from above (no ADV-OBJ-004).
- **One per-player mode** (the T14 review's inbox line; agent's call on the shape): `visit_at`
  takes the acting player and sets `StepVisit::pass` for a gate whose key it holds (one test,
  read only on a typed object's tile); `check_step` turns a pass into no visit, and the query's
  object branch, which already asked `visit_at`, asks it with the hero's owner and pushes a passed
  tile as any other. So `MoveHero` and `Reach` read one rule for a key holder and not.
- **A key gate's type is visited from above** (agent's call): the query tests ADV-OBJ-004 on a
  path's first tile only (T14a's peeled start), and a path never went on from any other tile an
  object is visited from; a gate is the first such tile a path crosses, so the ruleset part now
  refuses a `key_gate` with `from_above = false` (`InvalidObjectType`, "a key gate is not visited
  from above"; the original's class 212 is on the "visitable from above" list, survey §2.3; the
  packs' gates are). No hash changes: valid packs compile as before.
- **Setup** (unchanged code, checked): `check_stand` refuses a hero on a gate's tile
  (`StandError::Visited`), as the gate type's mode is `neighbour`, the mode of a player without
  the key, and no player holds a key at setup. A key holder's hero that a move stopped on a gate
  stands on a live gate, a state setup refuses but play reaches by the rules: it leaves the gate
  as any tile (upward too), and another hero's step onto it is `Occupied` in `MoveHero` and the
  query alike (meeting heroes is increment 3's).
- **The inbox's second item** (an entered removable object leaving a hero on a tile whose next
  live object is neighbour-visited): T15 cannot reach it. The removable objects (pickups, guards)
  are visited from a neighbour, so no hero stands on their tiles when they leave; the entered
  ones (Cartographers, observatories, tents) and the gates never leave. The line stays in the
  inbox for the first entered removable object; `check_stand`'s refusal is a setup check only, and
  the gate shows such a state is legal in play.
- **Properties**: the h3-adventure query property gives the owner the gates' key from the start
  in half its cases (`give_key`, test only; tents give it in others) and reads the visit through
  the same per-player `visit_at`. Fixed tests: the gate without the key (stays, pays, drop to 0,
  `PastVisit`, no event), with it (crossed with one step event per tile, a stop on it, leaving it
  upward), another player blocked by a hero on it then visiting it, never removed; the query with
  and without the key (and with another colour's); setup's refusal; the compile refusal. In
  `h3-map` the red border gate is the plans' ninth kind (drawn more often with a visitable
  shape); in three plans of four its visitable tile is the one below the red tent's (the tent is
  not entered from above, so a hero leaving it downward meets the gate with the key, one coming
  from below meets it without), and random play's visit pick also takes moves across a visitable
  tile (agent's call: crossings were 1 to 4 per run before). Atomicity, legality consistency,
  conservation, isolation, hygiene, the views against the events, clone equivalence and
  determinism hold (also once at 600 cases in a release build). Counted over 48 cases (temporary
  counters, seven runs of the final plans): 2 to 12 crossings by key holders, 0 to 3 stops on a
  gate, 16 to 29 visits without the key.
- **Versions**: none bumped (no state, import, ruleset-part hash, encoding or replay change: the
  keys are T15a's); golden replays verify unchanged.
- **Measured** (`bench-check`, the only T15 change to movement and the query): `reach-xl-144`
  565 267 757 Ir (−0.24 % on the baseline, −0.08 % on T15a's), `random-play-duel-36` 437 273 411
  (−0.52 %, −0.10 %), `fork-xl-144` 349 000 056 (+1.57 %, unchanged). The first build cost
  +0.23 % and +0.28 % over T15a: the new parameter made the compiler call `visit_at` out of line
  from the search (786 660 Ir of its own over 58 248 calls, from the XL map's blockers, and
  +0.5 M in the search); `#[inline]` on it restores T15a's code. An out-of-line helper for the whole
  object branch cost more (+0.68 %, +0.89 %). No re-baseline.

**Review and closure of the round** (runner).
- **Independent review** (an agent that had not seen the work; read the diff, traced the cases,
  ran throwaway tests in a worktree; the query against `MoveHero` at 100 000 cases in release):
  no defect a shipped pack or map reaches. Checked and passed: atomicity of every rejection
  (moves onto and through gates and guards, the answers with `OfferGone`), `MoveHero` against the
  query for gates with and without the key (crossing, stopping, leaving upward, starting on a gate,
  a gate under another hero, two gates of other keys), the masks and `legal_*` against `apply` in
  the stale case, the viewers of `KeyGained`, `TilesRevealed`, the offers and a guard's
  `ObjectRemoved`, T15a's call to let the hygiene comparison go on after the viewer's own
  observatory (sound: every disk tile is explored in the property's reference run, so `vary`
  leaves it alone), the flags' copy-on-write and both hash bumps. Three findings, all fixed:
  - Two guards of one type on one anchor: accepting one's passage removed the other (`offer_stands`
    and `accept_offer` take the last object with the offer's type and anchor). Setup now refuses
    two typed objects of one type on one anchor, `SetupError::SharedAnchor` (runner's call, for the
    checkpoint: a player names a typed object by both, ADR-0016 point 6, so this also settles the
    T14 verify's naming case; none of the 164 shipped maps has one, T15a's count; the play plans
    already lay out one per kind and anchor). Random resources are checked by their own type,
    before the draw: two different random piles on one anchor could still draw one type (a name
    shared by pickups only, which no offer names).
  - The ruleset accepted an entered guard or gate, which a player without the key would stand on
    and walk past: a key guard or gate type must now be visited from a neighbour ("a key guard or
    gate is not visited from a neighbour"; every pack's are).
  - Wording: the env's mask and agent docs said "its two answers"; `check_masks` asserted both
    answers with any decision open, which a turn group with a stale offer would break: now
    `Decline`, and `Accept` while the offer stands.
- **The full local verify** (CI cannot start: billing) with `--with-original-data` on the branch's
  head before the pull request (its result there).

### Checkpoint after T15a + T15b and T16b + T16a: batch 2, maps with objects (2026-10-13, `adventure/e08-t16`)

The user accepted every T15 call. The round, the user's cut of T16: **T16b** (the spec stamp and
batch 2: D-3 and D-5's piles) first, so the user can play batch 2 while **T16a** (the maps with
objects, replays, benchmarks, the audit) is built; both on one pull request labelled
`cross-lane` (battle-owned `h3-experiment`, consented). T16c (scoring batch 2, the closure) next
round.

**T16b — batch 2 with the spec stamp** (a worker).
- **The stamp** (decision 18): `h3_experiment::stamp`, 8 hex digits of the CRC-32 of the spec's
  text with CRLF made LF (`h3_map::crc32`, no new dependency), so the user's Windows checkout and
  Linux agree; `parse` sets `Spec::stamp` (a `stamp` key in the file is refused), `build` starts
  the map's description with `Stamp <hex>.` (the scenario list shows it) and puts the stamp and
  the line to compare it at the template's top. Tests: the value (`stamp("123456789")` is the
  CRC's check value `cbf43926`), CRLF, the description, the template, a Windows copy of a spec
  building the same map and template.
- **Resource piles**: `h3_map::writer::Pile` (class 79, the resource as subclass, the template
  from `Objects.txt` as the monsters'; the body as the editor writes one without a message: flag,
  u32 amount, 4 unused; validated: class, resource below 7, tile); `[[map.pile]]` in the spec
  (`resource`, `row`, `from`, `to`, `amount`, 0 drawn at the game's start; one pile per tile,
  `Tables::piles`); `h3-map`'s read-back test reads a random mercury pile back and imports it as
  `base:pile_mercury` without an amount. Docs: `docs/experiments/README.md` (the stamp, the
  shape, the batch's row), `docs/formats/h3m.md` (what the writer writes).
- **Batch 2** (`experiments/specs/E08-batch-2.toml`, 72 × 72, sand with a grass band; hot seat,
  one day, no battle; stamp `59a4b43d`):

  | Encounter | Set-up | Reading (engine) | VCMI | Wiki |
  |---|---|---|---|---|
  | D3a | 12 sand steps (200 left), onto a cobblestone road | 50 left, a further road step, 0 | 0, no step | 50, a step |
  | D3b (control) | the same onto a dirt road (75) | 0, no step | 0, no step | 50, no step |
  | D3c | 1 grass and 12 sand steps onto a snow marker | 100 | 0 | 100 |
  | D3d | 13 sand steps onto a snow marker | 0 | 0 | 50 |
  | D5w, D5m | 16 random wood, 16 random mercury piles | 5–10, 3–6 | 6–10, 3–5 | 5–10, 3–6 |

  Red's four walkers (one Angel: 2000 points; not native to sand or snow; no skills or
  artifacts) walk 13 steps straight east, no diagonal; two scouts explore their rows beyond the
  destinations so each can be clicked from the start; the observation is the walk (where the
  hero stops, the points it shows, whether a further step follows), not the preview, which does
  not apply the drop (0x56B440). Each source has its own signature over the four. Blue's four
  collectors stand between two rows of four piles each and pick them up column by column; the
  template asks for the resource bar after each pickup (the amounts are the rises). A wood pile
  of 5 or a mercury pile of 6 refutes VCMI's ranges; neither in 16 + 16 happens under the reading
  with (5/6)^16 · (3/4)^16 = 0.054 %. About 70 clicks for D-5, 16 for D-3.
- **Checked** (a throwaway program, not committed): the compiled map read back through `h3-map`
  and imported, then played by the engine with seeds 1 to 3: the description carries the stamp;
  the reachable-tiles query's path to each D-3 destination is straight east; D3a 50 then 0 on the
  road's second tile, D3b 0 and the further step refused, D3c 100, D3d 0; 32 typed piles
  (16 `pile_wood`, 16 `pile_mercury`) without amounts, each collector spending 1246 of 2000 points
  in the procedure's order, every amount within 5–10 and 3–6. Compiled again after the last
  change to the spec: the map in `experiments/out/` and the template carry the same stamp.
- Not shown in the template, as before: the predictions (in the spec, for the evaluating agent).
- **Delivered** (runner) to the user's `experiments/out/` (the map, and a copy of the template to
  read while playing); the runner checked the VCMI column against VCMI 1.7.5's
  `CPathfinderHelper::getMovementCost` (`pointsLeft < getTileMovementCost(dst, src)`, the cost of
  the step just taken, then all points are taken), as the survey states it.
- **Review** (an agent that had not written it): D-3's and D-5's arithmetic, the pile body, the
  templates and the stamp confirmed. Fixed: the writer refuses a pile's fixed amount above
  `0x7FFFF` (setup refuses more; batch 2's piles are random, its map unchanged by the fix);
  `docs/experiments/README.md` says gold counts in hundreds and that the stamp covers the spec's
  text only (recompile staged batches after a compiler change). Left, for the scoring: the
  template says the collector takes "the first two piles without moving", but four are (the
  next column's two are diagonal neighbours; the engine's 1 246 points count them so); the
  original's pathfinder may spend up to 1 446 points on a collector's eight piles (both within
  2 000; the amounts are unaffected). Not changed in the spec, so the stamp of the map already
  delivered stays valid.

**T16a — the maps with objects, the replays, the benchmarks, the exit criteria** (a worker).
- **The maps** (`h3-map`'s `tests/fixtures.rs`): increment 2's typed objects drawn from each
  plan's generator after the terrain and the blockers, which so stayed as they were (terrain,
  blocker and hero records and the first four templates byte for byte, checked against the old
  files); in map order the objects follow the blockers. `Placer` draws an anchor until every
  tile of the object is free land off the roads (a guard's or a gate's visitable tile on its
  road) and its visitable tile keeps two free neighbours it is reached from not from above
  (ADV-OBJ-004), which later objects leave free: no object covers a hero or another object, so
  setup takes both maps (no hero on a visited tile or a gate, no shared anchor). The density is
  the shipped maps' (a throwaway count over the 164 staged maps: increment 2's classes cover
  3.7 % of the land tiles on 36 × 36 maps, 3.0 % on 72, 2.5 % on 108, 2.9 % on 144; random
  resources and piles nine in ten): the duel 66 objects (2.9 % of 2 263 land tiles), the XL
  16 times as many (1 056, 3.0 % of 34 903). The duel: within each surface hero's first reveal a
  pile, a random resource, a green Keymaster's tent and a land Cartographer, within each
  underground hero's a pile, a random resource, a red tent and a campfire (5–6 typed objects in
  each hero's day-1 reach of 45–81 tiles); a green guard and gate on the surface road, a red
  pair on the underground road; anywhere else 22 more piles (the seven resources, drawn amounts
  but one gold pile of a fixed 300), 20 random resources, 2 campfires, a Redwood Observatory
  (surface) and a Pillar of Fire (underground). The XL: the same beside its eight heroes, four
  guards and four gates of the level's colour on each road, the rest anywhere on its level
  (tents, guards and gates green on the surface, red underground; 28 more Cartographers, 16
  observatories of each kind) or the map. `play.rs`'s `KINDS` gains eleven kinds only the maps
  lay out (the strategy draws the first nine, `PLAYED`, as before); `lay_out`'s one typed object
  per anchor and type compares class and subclass (two kinds share the gold pile's type). The
  duel's file 756 → 1 273 bytes, the XL's 8 883 → 13 551; `fixtures/maps/README.md` updated.
- **What plays the duel**: `h3-sim`'s adventure tests and `h3-cli`'s pass on the new map
  unchanged (no expectation depended on the empty map); only the module doc of `h3-sim`'s
  tests changed (`chart_setup` still adds a Cartographer on the nearest tile beside each hero,
  underground too: the duel's own two are farther).
- **Golden replays**, re-recorded with their documented options (seeds, days, difficulty,
  profile): every hash changed with the map's; commands `duel-s1-d28` 136 → 130 (74 moves, 56
  ends), `duel-s2-d7` 42 → 42, `duel-s5-strict-d7` 46 → 43, `duel-s3-easy-d7` 34 → 34. They
  verify in debug and release and in the scoped verify's determinism step. What they meet:

  | Replay | Pickups (gains) | Removals | Keys | Offers opened / refused / accepted / declined | `OfferGone` | Reveals | Gates crossed |
  |---|---|---|---|---|---|---|---|
  | `duel-s1-d28` | 1 crystal pile (1) | 1 | 1 green | 0 / 0 / 0 / 0 | 0 | 0 | 0 |
  | `duel-s2-d7` | a campfire, a sulfur pile (3) | 2 | 0 | 0 | 0 | 0 | 0 |
  | `duel-s5-strict-d7` | 1 gold pile (1) | 1 | 1 green | 0 | 0 | 0 | 0 |
  | `duel-s3-easy-d7` | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

  **None of the four meets an offer.** Random play's moves seldom end on an object: the agent
  picks a destination uniformly in the hero's reach, which every step's reveal widens, so 74
  moves in 28 days meet two objects at a density of 3 % (`OfferGone` needs two players in one
  turn group, never in sequential play). **Two more replays (runner)**, chosen from a search over
  the same agent's seeds instead of a new agent (`--seed N --agent-seed N --days 28`, N up to
  1 200: about one game in four answers an offer, 2 of 927 meet a guard's pass): `duel-s892-d28`
  (154 commands) accepts a Cartographer's offer (1 000 gold paid, a level revealed), takes a key
  and accepts a border guard's pass (the guard removed); `duel-s988-d28` (138 commands) declines
  an offer, accepts one and visits an observatory. The matrix now replays both answers and both
  offer kinds; a gate crossing still has no event of its own.
- **Benchmarks** re-baselined (no engine code changed; the workloads' maps did):

  | Workload | Before (baseline) | After | Per unit |
  |---|---|---|---|
  | `random-play-duel-36` | 439 577 005 Ir (685 steps) | 391 603 057 (667), −10.9 % | 640 k → 587 k Ir per step |
  | `reach-xl-144` | 566 626 021 (1 920 queries) | 551 030 247 (1 904), −2.8 % | 294 k → 289 k per query |
  | `fork-xl-144` | 343 600 056 | 349 400 056, +1.7 % (+1.6 % since T15a's keys) | 3 490 → 3 494 per fork |

  Objects cover tiles and end paths, so a query searches less. Not gated (callgrind): a duel
  query 241 k → 224 k Ir, an XL step 1.395 M → 1.392 M, a duel fork 2 253 → 2 261. Wall time
  (release, best of nine, the old and the new maps alternating with one binary; runs of one
  measure vary by up to a third on the sandbox's shared vCPU): duel step 64.9 → 70.0 µs, + the
  observation +20 → +16 µs, + the masks +59 → +50 µs; XL step 147 → 161 µs, + the observation
  +256 → +543 µs, + the masks +119 → +144 µs; query 23.8 → 24.4 µs (duel), 27.2 → 29.7 µs (XL);
  fork 0.27 → 0.29 µs, 0.32 → 0.34 µs; `GameState` / `AdventureEnv` 216 / 248 bytes, unchanged.
  New: the XL observation's object list costs ≈ 1.6 M Ir per observation (every typed object's
  tiles tested for explored or fringe, the fringe test reading eight neighbours of each hidden
  tile; the observation of an XL step +75 %): a lever for E13's encoding (inbox). The rows of
  `docs/02-architecture.md` §8 follow in a `shared/` pull request.
- **Increment 2's exit criteria** (the epic's list, checked on 2026-10-13):

  | Criterion | State |
  |---|---|
  | ADR amendments accepted | ADR-0016 and ADR-0017 amendments (2026-10-12, #115) |
  | Full verify green in CI, logic crates at 100 % | CI cannot start (GitHub billing): the runner's full local verify with `--with-original-data` stands in, **green on `0c87d63`** (2026-10-13; every step, the 164 shipped maps read, the benchmarks at +0.00 %); `h3-adventure` and `h3-game` at 100 % |
  | Atomicity | `h3-map` `play.rs` `every_step_keeps_the_commands_invariants` (48 cases × up to 40 steps, both profiles) on plans with piles, campfires, a Redwood Observatory, a Cartographer, red tents, guards and gates; `h3-sim` `a_refused_command_changes_nothing` on the duel with objects |
  | Legality consistency, pending decisions included | `check_legal`/`check_unlisted` (answers alone while a decision is open, `Decline` alone when it no longer stands); `h3-sim` `steps_agree_with_the_masks_and_the_game` with a Cartographer beside each hero; `h3-game` `tests/decisions.rs` (a stale offer, `OfferGone`: only `set_schedule` builds two players in one group, the inbox) |
  | Ownership isolation: treasuries; streams if a command draws | `check_isolation`: the other players' treasuries, purchases, decisions, keys, explored tiles, heroes; no command draws (every draw is at setup, ADV-PIK-001 to -003 from the setup stream; a player's stream is only hashed in `h3-game`), so the streams' part is void; the hygiene property varies the seed |
  | Conservation | `check_conservation`: every army unchanged by every accepted command; a treasury changes by the told gains, which come with a removal, and an accepted offer's price; **added here**: the gains are exactly the drawn contents of the objects that left, in order (checked to fail on a gain off by one) |
  | Observation hygiene (typed objects, treasuries) | `view.rs` `a_viewer_learns_nothing_it_cannot_see` (seeds, armies, hidden tiles and objects' contents differ; the viewer's treasury in its observation) and `views_move_as_the_events_say` |
  | Clone equivalence, determinism | `play.rs` `forks_evolve_as_their_parent`, `replays_are_deterministic`; `h3-sim`'s env forks and rollouts on the duel with objects; the golden replays |
  | Objects the properties play | every kind of increment 2 by its kind (pickup, reveal by terrain, reveal around, key tent, key guard, key gate); not the random resource, whose drawn pile type shows (the hygiene property would see the seed): its draw is `h3-adventure`'s unit tests', and since T16a the env tests and the replays play 24 on the duel |
  | Golden replays on a duel with objects in the determinism matrix | six: the four re-recorded and two chosen by seed (above); pickups, keys, both answers, both offer kinds, an observatory |
  | Benchmarks re-baselined | the three, above; §8 in a `shared/` pull request |
  | Every new rule `implemented` with sources | 12 ADV (ADV-OBJ-002 to -005, ADV-PIK-001 to -003, ADV-VIS-001 to -005) and 3 GAM (GAM-RES-001, -002, GAM-DEC-001), each `implemented` with two to four sources (`check-rules` ok); ADV-PIK-001's ranges (D-5) wait for batch 2 |
  | Batch 2 staged and scored | staged (T16b), scored in T16c |
  | HANDOFF, epic log, `QUEUE.md` current | this log; the HANDOFF and the queue with the round |

### Checkpoint after T16b + T16a and T16c: batch 2 scored, increment 2 closed, increment 3 refined (2026-10-13, `adventure/e08-t16c`)

The user merged T16b + T16a (#158, after the shared #157), played batch 2 in SoD 4.0 Complete
with plain HD and pushed the filled template (stamp `59a4b43d`, the map's matched). T16c in one
pull request (`cross-lane` for `docs/experiments/README.md` and the client lane's inbox line, both
the user's).

**T16c — batch 2 scored** (the spec's predictions against `experiments/results/E08-batch-2.md`):

| Encounter | Observed | Reading (engine) | VCMI | Wiki | Verdict |
|---|---|---|---|---|---|
| D3a, sand onto a cobblestone road with 200 | 50, the next tile green, stepped, 0 | 50, a step, 0 | 0, no step | 50, a step | the reading (VCMI refuted) |
| D3b, the same onto a dirt road | 0, red, no step | 0, no step | 0, no step | 50, no step | the reading (the wiki refuted) |
| D3c, 100 left on snow from sand | 100 | 100 | 0 | 100 | the reading (VCMI refuted) |
| D3d, 50 left on snow from sand | 0 | 0 | 0 | 50 | the reading (the wiki refuted) |
| D5w, 16 random wood piles | 5, 6, 6, 5, 7, 5, 7, 8, 9, 9, 7, 9, 9, 7, 8, 6 | 5–10 | 6–10 | 5–10 | the reading and the wiki (VCMI refuted: three of 5) |
| D5m, 16 random mercury piles | 3, 5, 3, 5, 4, 6, 6, 4, 5, 5, 4, 3, 3, 3, 6, 6 | 3–6 | 3–5 | 3–6 | the reading and the wiki (VCMI refuted: four of 6) |

- The amounts are the rises of the resource bar after each pickup (start 39 wood, 15 mercury).
  Nothing odd was noted; the review's caveats (four piles taken without moving, more points in
  the original's order) did not show in the results.
- **D-3 confirmed** (ADV-MOV-006, now `verified`, an `observation` source): the points left after
  a step drop to 0 only below the cheapest straight step from the new tile (0x481182); VCMI 1.7.5
  (`pointsLeft < getTileMovementCost(dst, src)`) and the wiki (no drop) are refuted. The diagonal
  exception in the same rule rests on the reading. A regression test pins the four encounters in
  the engine's own terms (a Peasant's 1 300 points on the test world:
  `the_points_left_after_a_step_reproduce_batch_2`). **D-5 confirmed** (ADV-PIK-001, now
  `verified`). The surveys' D-3 and D-5 rows and `docs/experiments/README.md`'s row say so.
- **Experiments by agents** (the user): the client lane's capture harness is a good basis for
  agents to perform the experiments themselves. Noted in `docs/experiments/README.md` ("Agents
  running a batch themselves": what a batch needs before it relies on it, the cross-check with a
  batch the user played, the run's setup in the source), the global HANDOFF's sources and the
  `observation` source kind (`shared/experiments-by-agents`, #168); the harness's adventure part
  in the client lane's inbox at high priority (the user).

**Increment 2 closed**: T16a's audit held every criterion but batch 2's scoring, now done.

**Increment 3 refined** (the user, ADV-Q11): monsters and battles (decision 19: heroes' details
in the game, wandering monsters and guarded piles, battles through `h3-game`, applying outcomes;
the dispositions and a battle between two players' heroes sized by the survey and decided at its
checkpoint), the first round a survey and the ADR amendment drafts (decision 20), batches planned
for the harness (decision 21). Not taken: the whole sketch at once, heroes without battles, towns
first. Level-ups, the adventure profile and chests moved to increment 4, the week to increment 5.
Inbox sorted: the adventure profile's items (Scouting, the step price's omissions, flying before
a neighbour visit) to increment 4; the hero details', guarded piles' and outcome items to T17a.

### Checkpoint after T16c and T17a: the battles survey (2026-10-13, `adventure/e08-t17a`)

The user merged T16c (#172, after the shared #168) and took **T17a alone** for this round (over
T17a with T17b), T17b in the next round, drafted for the scope picked from the survey. CI could
not start (billing); the full local verify with the original data stood in, green on `master`
(`8dd6920`) before the change.

**T17a — survey** `docs/rules/surveys/E08-battles.md`, from a code reading of the human player's
path (a visit of a monster and a step onto a guarded tile, both into 0x4A7740), VCMI 1.7.5, the
wiki (Wandering creature, Diplomacy; through the summarising tool, which would not quote more than
125 characters) and a census of the 164 shipped maps. A fresh agent re-read the deciding code
without the survey and confirmed it. Findings:
- **The monster's answer** (§3): the hero's power (army value times
  √((1 + 0.05 D)(1 + 0.05 A))) over the monster's strength in single precision gives a step
  (11, ⌊2r − 2⌋, −1, −2, −3); with sympathy (0, 1, 2) and Diplomacy against the aggression:
  fight, join free, join for gold (offered only when affordable), else the fight at C = M or for a
  monster that never flees, else the offer to let it flee. VCMI ported it; three disputes: the
  boundary r = 0.5 (D-8), VCMI 1.7.5 not counting the monster's own type for sympathy (D-9), Easy
  shifting Diplomacy one level up (the reading and the wiki; VCMI lacks it, D-10).
- **The split** (§4.1): `srand(248071 x + 276589 y + 310111 z + 155303)` of the monster's tile,
  2–7 stacks by the army ratio, a first draw for ±1, a second for the upgraded middle stack:
  a function of the tile and the armies. VCMI reproduces the first draw exactly and approximates
  the second (41 of 41 472 tiles differ, D-11). The battle's obstacles are seeded from the tile as
  well (`battle_side_setup`, the same expression as VCMI's `RandGen`); the engine draws them from a
  kernel stream today.
- **After a battle the monster survives** its count is the survivors of its own type: the upgraded
  stack's survivors vanish (A-6, the wiki's bug; VCMI fixes it).
- **Guarded tiles** (§2): the 3 × 3 scan column by column, water to water, not from above; the
  first guarded tile ends a path and the move; the reading, VCMI and the wiki agree.
- **Guarded piles** (§5): an empty guards record is a plain pile (3 shipped piles); a message asks,
  no message fights at once; the guards fight unsplit; VCMI agrees.
- **Heroes' details** (§6): two layers in the map; defaults; the default experience 40–90 (VCMI
  40–89, D-12); 42 of 116 placed heroes have a custom experience, all of at least 1 000, which
  increment 3 must refuse or import at level 1 (the original's level at load not read).
- **Hero against hero** (§7): a battle at once, both players; a surrender's gold goes to the enemy
  player (VCMI only takes it, D-13); a beaten, retreated or surrendered hero leaves the map (the
  pool and defeat are E09's and E10's); no town condition on retreat or surrender found.
- The census: 14 021 monsters in 162 maps (13 239 random counts; the largest fixed 3 000), 36
  guarded piles, 832 map-wide hero type entries (15 with gameplay fields).
- 38 address-map entries (and six updated), fields of `creature_record`, `hero`,
  `battle_manager` and `map_item`'s word; `check-exe-map` clean.

For the user (ADV-Q12): joining and fleeing in increment 3, battles between two players' heroes,
custom experience, D-8 to D-13, A-6, the treasures' artifacts; then T17b.

### Checkpoint after T17a and T17b: the ADR amendment drafts for increment 3 (2026-10-13, `adventure/e08-t17b`)

The user merged #177 and answered ADV-Q12 (OPEN-QUESTIONS, answered; the epic's decisions 22–24):
monsters join and flee, the join offer skipped as declined while the group finds no room (a feature
to-do for increment 4's army management, `ADV-STUB-2` when built); heroes of two players fight, a
capture placed as VCMI places it; heroes with an experience of 1 000 or more and treasures with an
artifact `Unsupported`; D-8 to D-13 each as the reading (asked one by one); A-6 always fixed. On
D-12 the user asked whether `random_between`'s upper bound is exclusive: re-read, it is not
(0x50B230: a + rand() mod (b − a + 1); 0x4BFE94 passes 0 and 0x32, then adds 0x28: 40–90). The
round: T17b whole, with the client lane's inbox line in the same pull request (`cross-lane`). CI
could not start (billing): the local verify stood in.

**T17b — ADR amendment drafts** in `docs/lanes/adventure/adr-drafts/`: `0017-increment-3.md`
(points 1, 2, 4, 6, 8–10 extended, point 11 new: the store of heroes and the adventure profile,
the system actor for a battle side without an owner, the obstacles from the tile and the battle's
seed from the attacker's stream, chained offers and battles as decisions, the env's battle phase
and neutral seat, battles through the game and their outcome) and `0016-increment-3.md` (points
1, 3, 4, 6: a guarding object's army left, enemy heroes visited as a battle, the monster and ground
kinds, guarded tiles, the monster's answer, the split by the tile, guarded piles, what a viewer
sees). ADR-0013 unchanged (its points 1, 2 and 5 already foresee the store, the derivations and
the orchestrator's outcome). Detail and the proposed cut in `E08-design.md` (§1–§11): five rounds,
T18a + T18b (the shared groundwork; the store of heroes), T19a + T19b (battles with hero against
hero; the env), T20a + T20b (monsters; their answer and battle), T21 + T22a (guarded piles, the
audit; batch E3), T22b (the closure).
- **Found:** the creatures' adventure-map ranges are harvested already (`adventure_count`,
  `CRTRAITS.TXT` columns 21 and 22, `h3_content::CreatureInfo`): T17a's "harvest needed" was
  wrong; the survey (§1.5, §14) and the queue's inbox corrected. The survey's §12 counted 138
  compliant and friendly monsters; §1.1's census gives 174 (136 + 38): corrected.
- **New for the user (ADV-Q13):** accept the drafts; who plays an unowned battle side (the system
  actor recommended); the `h3-heroes` handover's timing (increment 4 recommended); the cut.
- Inbox: the no-room feature to-do; the client lane's inbox gets increment 3's harness needs.

**ADV-Q13, the user's review of #181 (the same evening):** everything as drafted but four points,
applied to the drafts before the merge (decisions 25–27). (1) *Does the hero split still have a
justification?* `HeroDetails` has no runtime user yet (staged battles and the CLI write
`BattleHero`; only its own tests and the pack tests use it). The battle seam and one record per
hero with properties derived once hold; a peer domain the adventure may not see does not (from
increment 4 the adventure reads and writes heroes most, towns follow): **`h3-heroes` becomes the
hero model in the `shared` layer**, the store stays in `h3-game`, contexts read by reference and
return typed `HeroChange`s; decision 5's profile withdrawn; a third draft
(`adr-drafts/0013-increment-3.md`, also ADR-0015 point 1). (2) Predictable obstacles are
exploitable: **fix-optional** (`ADV-QUIRK-5`), and the split likewise with its own flag
(`ADV-QUIRK-6`); the obstacles' tile seed re-read at 0x463A08 with the call site's arguments
(x and y from the position word, the level not used). (3) The system actor for unowned sides:
fine; **the original's battle AI must be recreated** (the battle lane's inbox). (4) **`h3-heroes`
moves as soon as the battle lane is done with it** (the battle lane's inbox asks when). T18a now
also applies the accepted ADR amendments.

### Round 1 of increment 3: T18a and T18b (2026-10-14, `shared/e08-t18a` #185, `adventure/e08-t18b`)

The user merged #181 and, asked for a seam before building, trimmed the round (decision 28): T18b
keeps the store of heroes, its defaults, validation, the import of the customisations and what a
player sees; `HeroChange` goes to T19a, the store passed to the adventure, `attack_defense`,
`join_bonus` and the difficulty to T20a. CI could not start (billing): the full local verify stood
in. `master` (7b20fad, after #183) was red before either branch: BAT-Q41 cites an address without an
address-map entry (`check-exe-map` and its test); the battle lane's `battle/e04c-t5` adds it.

**T18a — the shared groundwork** (#185, `shared/` and `cross-lane`):
- The accepted amendments applied: ADR-0013 points 1 and 2 (the hero model in the shared layer,
  the store, hero changes; decision 5 withdrawn), ADR-0015 point 1, ADR-0016 points 1, 3, 4, 6,
  ADR-0017 points 1, 2, 4, 6, 8–10 and the new point 11; ADR-0005's amendment; the index, the
  glossary, `02-architecture.md`.
- `arch.toml`: `h3-heroes` in the shared layer, the new edges; `check-arch` gains forbidden edges
  (`[[policy.forbidden_edge]]`): `h3-battle → h3-heroes` stays rejected, declared or used.
- `h3_content` kinds `necromancy`, `experience_gain`, `max_mana`, `join_bonus` (codes 38–41,
  pack format v37). The battle lane's syntax test wants every kind in its synthetic packs: they
  went into `synthetic-ext`'s Storm Regalia, not `synthetic`, whose ruleset hash the battle
  replays record (so `cross-lane`, and no battle replay moved). The core ruleset hash version
  stays 43: no existing ruleset hashes otherwise.

**T18b — the store of heroes** (`cross-lane`: `h3-heroes`, the harvester, the packs, the address
map, 06-security):
- **GAM-HER-001** (`h3_game`): `GameState.heroes: Vec<Arc<HeroDetails>>` by `HeroId` (`hero`,
  `heroes`), built from `GameHeroSetup.details` (`HeroDetailsSetup`: experience, primary,
  skills, artifacts, spells, each `Option`) or the defaults: level 1; experience 40 + a draw of
  0–50 from the setup stream, in hero order after every starting army and before the objects'
  contents; the class's primary skills; the type's skills and spells; the spell book worn when
  the type starts with one and the artifacts are not customised (VCMI's `SPELLBOOK_PRESET`);
  `draw_starting_army` now returns the war machines its roll gave, worn as their artifacts where
  their slot is free. Spell points full; `h3_heroes::validate` (`GameSetupError::Hero`).
  `GameObservation.heroes` (`HeroInfo`): every hero the viewer sees with its type, the details
  of its own. Game state hash v5 (the store after the game's stream).
- **HER-MAN-001** (`h3_heroes::max_mana`, `MANA_PER_KNOWLEDGE`): a code reading
  (`map_hero_setup` 0x486C5C to 0x486C98, the Intelligence getter 0x4E48B0, entered in the
  address map): trunc((1 + f32(r × (1 + 0.05 × level))) × f32(10 K)), K the clamped Knowledge;
  VCMI's `manaLimit` the same but for the specialist's single precision (under Q-0005 an
  Intelligence specialist at level 1 with Basic and Knowledge 8 has 100, not 101). Effective
  Knowledge is HER-BAT-001's, now a helper of both.
- The harvester reads Necromancy's (0x63E9B8), Learning's (0x63EA58) and Intelligence's
  (0x63EA88) tables into `necromancy` 10/20/30 %, `experience_gain` 5/10/15 %, `max_mana`
  25/50/100 %: the base pack's three skills gain levels (the core ruleset hash of `base` moved).
- `h3-adventure`: `HeroSetup.formation`, `HeroOnMap.formation`, `OwnHeroView.formation`;
  adventure state hash v7.
- `h3-map`: the import reads a placed hero's customisations, the body's field over its hero
  type's map-wide entry's (VCMI: "using the latter set"); code tables `ARTIFACTS` (141),
  `SPELLS` (81), `SKILLS` (28) checked against the packs; before SoD experience 0 and AB's spell
  byte `0xFE` are the type's, `0xFF` no spell, another spell beside the type's; primary skills
  clamped to the hero parameters (the original's load); `Unsupported::HeroExperience` from
  1 000; formation. `ADV-STUB-1` closed; import v4. The shipped maps: 5 import whole (4 fewer:
  `Adventures of Jared Haret`, `Faeries`, `Knee Deep in the Dead`, `Unholy Quest` have a hero of
  level 2 or more); no imported map fails validation. The reader's test map had an ammo cart in
  its map-wide hero's backpack (now invalid): an axe instead; the generated fixture maps
  rewritten (`H3_WRITE_FIXTURES=1`).
- Golden adventure replays re-recorded with their options: the commands unchanged, the hashes
  (core ruleset, game v5, adventure v7, import v4, the maps') new.
- Tests: the defaults, the war machines, the customisations, validation, hash and forks, the
  views (`h3-game`); the customisations, map-wide entries, the version differences, the refusals
  and the code tables (`h3-map`); the hygiene properties give the viewer's heroes an experience
  of their own (the viewer now sees its drawn experience).
- **Measured** (`bench-check`): `fork-xl-144` +0.52 % with the store (the list of heroes behind
  one `Arc`, an `Arc` per hero inside); a list per fork had cost +9.76 %. The other workloads
  within ±0.5 %.
- Full local verify (`--with-original-data`): every step passes but `check-exe-map` and its
  test, red on `master` too (above).
- Inbox: the four maps, +0x479 and artifacts, VCMI's catapult and skipped artifacts.

### Round 2 of increment 3: T19a1 (2026-10-14, `shared/e08-t19a`, `adventure/e08-t19a1`)

#185, #187 and #188 merged first: `master` (ac19351) green in the full local verify
(`--with-original-data`; CI could not start). Asked for a seam before building, the user split the
round (decision 29): T19a1 (battles through the game) and T19a2 (the heroes' outcome) this round,
T19b next; `Actor::System` with T20b's monsters.

**`shared/e08-t19a`** (shared paths only): `h3_content::OBSTACLES_FROM_TILE` (flag 23,
`quirk.adventure.obstacles_from_tile`; no profile set it there, so no ruleset hash moved);
`arch.toml` lets `h3-map`'s tests compile the battle part; `06-security.md`: what a battle's owners
and other players learn of it.

**T19a1 — battles through the game** (`cross-lane`: the flag on in `packs/base/core.toml`'s
`faithful` and in the harvester's profile lines):
- **The visit** (GAM-BAT-004, ADR-0016 point 4): `check_step` meets another player's hero
  (`Meets::Hero`) before the tile checks: visited from the tile before at the step's price, the
  move ends; only leaving the visitor's tile across its top is refused (ADV-OBJ-004); an own hero
  blocks (`Occupied`). `Reach` marks other heroes' tiles own (blocks) or enemy (a destination, not
  expanded). `AdventureOutcome.battle: Option<BattleStart>` (both sides as `Combatant`s: hero,
  owner, army, formation; the defender's tile; terrain and battlefield).
- **The ground** (GAM-BAT-001): the adventure part's `[params.battle]` (`BattleTerrainsDef`:
  `base` underground on subterranean, the coast on desert, the original's sand, with `base:shore`;
  part v6), `AdventureState::battle_ground` in the original's order; the coast flag `0x40` imported
  (`TileSetup.coast`, import v5) and kept as a bit per tile of the static map (`is_coast`, hashed:
  adventure state hash v8). `GameRuleset` bundles the battle part (`RulesetMismatch` also for a
  coast battlefield the battle part lacks).
- **The battle** (GAM-BAT-001): `Decision::{Offer, Battle(Arc<OpenBattle>)}` (game state hash v6),
  `GameState::start_battle`: `BattleSetup` from the `BattleStart`, the store (battle heroes with
  `max_mana`, war machines) and the treasuries (funds in the surrender's resource); the battle's
  seed the attacker's stream's next draw, the obstacles' the tile's (110 291 x + 167 801 y +
  81 569, 32-bit) under `ADV-QUIRK-5`, else the stream's next. `GameCommand::Battle` from the owner
  of the side to act (`NotYourSide`, `NoBattle`, `BattleOpen`, `Battle(CommandError)`): the list
  is copied on its first write, the battle once the command validates. Events `BattleStarted`,
  `Battle { event, seen_by }`, `BattleEnded` to the owners; `GameObservation.battle` (the viewer's
  side); `legal_commands` the battle's for the side to act's owner. A setup the battle refuses
  opens nothing (a test with a cart of no kind).
- **The outcome** (GAM-BAT-002, -003, -005): a hero stays only if its side won with army
  survivors (also the BAT-Q7 case), keeping them; the others leave the map (`HeroRemoved`, seen
  where the tile is); a surrender's price leaves the surrendering side's owner (saturating) and
  enters the other's (D-13). The heroes' part is `ADV-STUB-3` (T19a2).
- **The env** (encoding v2 has no battle): the masks, the decoder and the random agent leave out
  other players' heroes' tiles, a battle command encodes to nothing, replay format v2 writes it
  unreadable (`ADV-STUB-4`, T19b). The six golden replays re-recorded with their options: the
  commands unchanged (the agent's draw over the filtered tiles), the hashes new.
- **Properties** (`h3-map`): the driver acts as the owner of the side to act; legality with a
  battle open (every listed command applies; nobody moves), conservation and isolation by a
  battle's end (`check_battle_end`); the views told the battle's events and removals; hygiene stops
  at the viewer's battle. In 48 cases about 37 battles start and 9 end, all by a retreat or a
  surrender (T19b: more victories); victories are unit-tested (`h3-game`'s `battles.rs`).
- **Measured** (`bench-check`): a `coast` field in `StaticTile` cost `reach-xl-144` +2.03 %
  (over the 2 % gate): the bit set instead, and the reach tests other heroes off the hot path:
  reach +1.14 %, fork +1.32 %, random play +0.83 %.
- Rules: GAM-BAT-001 to -005 `implemented` (the code reading, VCMI 1.7.5 re-read for
  `onHeroVisit` and `doSurrenderAction`, E04c batch 3b); `ADV-QUIRK-5`'s ledger entry has its flag
  and tests.
- **The full verify** (round 2's second session, decision 31): the first complete run found
  three gaps, fixed on the branch: the design note cited 0x469F20, 0x4E3C60 and 0x4E3CD0 without
  map entries (now `necromancy_add_raised`, `hero_necromancy_creature`, `hero_necromancy_power`,
  re-read: the power's town bonus counts each of the owner's Necropolis towns with the buildings;
  the upgrade's elemental exception holds only while `[game]+0x1F698` is 0), and the four new
  `GameCommandError` messages were never formatted (`the_battle_errors_read_well`). Decision 30's
  two points settled; `shared/e08-t19a2` built.

### Round 2 of increment 3: T19a2a (2026-10-14, `adventure/e08-t19a2a`)

#192 to #194 merged first. The baseline full verify of `master` (0919729, `--with-original-data`)
passed every step but `deny`: four RustSec advisories against rustls 0.23.45 (RUSTSEC-2026-0346 to
-0349, through `h3-server`); #197 (`shared/rustls-advisories`) moves the lock file to 0.23.46.
Asked for a seam, the user split T19a2 (decision 32): T19a2a here, T19a2b (Necromancy, the
`raises` entries, Eagle Eye's learning) next, so the packs and the replays change once; Eagle
Eye moved to T19a2b because its Wisdom limit needs a new effect kind. The user confirmed #194's
tie rule as built.

**T19a2a — the heroes' part of a battle's outcome** (`cross-lane`: `h3-heroes`, `heroes.toml`):
- **The code reading** (address map entries, `battle_end` 0x477470 on): a winner with a hero gets
  `battle_experience` 0x46A350 (the loser's records without the summoned and war machine flags at
  the types' hit points; +500 for the loser's hero, taken off again when a side retreated or
  surrendered, 0x6985A3 and 0x697744 from `battle_winner`; times the Learning getter 0x4E4840 on
  f32 points, truncated), given by `hero_gain_experience` 0x4E33B0 (the scenario's level limit
  caps it; the levels it gives, increment 4); then the raised creatures, Eagle Eye, and
  `battle_capture_artifacts` 0x46A070: nothing after a retreat or a surrender; the loser's body
  positions in order skipping the spell book, the Grail and the war machines, then its backpack
  from the end, each through `hero_give_artifact` 0x4E3070 (equipped at the first body position
  that takes it, else appended to the backpack, at most 64, no war machine) and removed from the
  loser; it stops at the first artifact the winner cannot take. In SoD an equipped part of a
  combination may complete it (a question for the local human, built at once for the computer),
  and equipping Titan's Thunder adds a spell book (0x4E2A64): neither is built (inbox).
- **`h3-heroes`** (`change.rs`): `HeroChange` (`GainExperience`, `SetSpellPoints`,
  `AddArtifact`, `RemoveArtifact(ArtifactPlace)`), `check_change`/`apply_change` (HER-CHG-001:
  the details stay valid; applying a refused change does nothing), where a received artifact
  goes (HER-CHG-002: worn if it and its locks fit, else the backpack's end), `next_capture`
  (HER-ART-004), `lost_war_machine` (HER-WAR-002: not the catapult flag's); `gained_experience`
  (HER-EXP-001, under Q-0005 the getter's single-precision rate times f32 points: a level-2
  specialist with Basic Learning gains 210 of 200, not 211), sharing `max_mana`'s arithmetic.
- **`h3-game`**: `GameState::heroes_after_battle` after the adventure's part: each side's spell
  points (`HeroOutcome::mana`, when changed) and destroyed war machines' artifacts, whatever the
  result; the winner that stays on the map gains the experience (`battle_experience`: army and
  raised records, `DEFEATED_HERO_EXPERIENCE` 500 after a victory over a hero; Learning) and, after
  a victory over a hero, the loser's artifacts until one finds no room. Each change applies to the
  store (copied on its first write) and is told to the hero's owner (`GameEvent::HeroChanged`,
  projected to the owner; `HeroChange`, `ArtifactPlace` re-exported). `ADV-STUB-3` narrowed to
  Necromancy and Eagle Eye. No version moves: the store is hashed since v5, no replay ends a
  battle.
- **Properties** (`h3-map`): conservation keeps every hero's details outside a battle's end; at
  one (`check_heroes_after_battle`) changes go to the battle's heroes by their owners, a hero's
  experience, spell points and number of artifacts follow its changes, nothing else of it moves,
  only a staying winner gains, artifacts move between the two or leave with a war machine; the
  views see their heroes' details change only as told. Plans give heroes backpacks (codes 7 to
  40); the random plays end few battles and none by a victory, so captures wait for T19b's plans
  (unit-tested in `h3-game`'s `aftermath.rs`).
- Rules: HER-CHG-001, -002, HER-ART-004, HER-WAR-002, HER-EXP-001 (Q-0005) new; GAM-BAT-002 (the
  heroes' part) and -004 (+500, the capture) completed but for T19a2b, with the reading, batch 1b
  R5, the wiki and VCMI.

### Round 2 of increment 3: T19a2b (2026-10-14, `shared/e08-t19a2b` #205, `adventure/e08-t19a2b` #206)

#197 and #198 merged first; the baseline full verify of `master` (5550866,
`--with-original-data`) passed every step. Asked for a seam, the user took both pull requests in
one session and the four Necromancy artifacts now; told that a new field moves the ruleset hash's
encoding, they put the harvested data and the one re-recording into the shared pull request
(decision 33), and asked to investigate whether the Cloak's 30 % without the skill is a bug: the
code has its own branch for it (0x4E3CD0 without the skill: only +0.30 with artifact 130), the
wiki's Cloak of the Undead King page says "For heroes without Necromancy, the Cloak of the Undead
King raises killed enemy troops" as Skeletons at a fixed 30 % that "cannot be increased", and
VCMI's `calculateNecromancy` says "need skill or cloak of undead king - lesser artifacts don't
work without skill": a rule (HER-NEC-001), not a quirk.

**`shared/e08-t19a2b`** (#205, `cross-lane`: packs, the harvester, the replays):
- `h3-content`: the kind `learn_level { levels }` (code 44; the highest spell level learnable grows
  over `[hero_params]`'s new required `learn_level`; sources add, at most 5), the `necromancy`
  kind's optional `unless_skill` (the share counts only while the hero lacks that skill),
  ruleset hash v44 (the golden base hash moved); `h3-pack`: their syntax, pack format v40.
- `h3-kernel`: `X87::over`, the x87's `fdiv` rounded to 53 bits (the proptest compares it with
  IEEE doubles), for Necromancy's count.
- The harvester: `[[heroes.skill_extra]]` (effects added to a table skill's levels), a `creature`
  in skill effects, skill and creature-list references in artifact effects; the gap-fill entries
  with the code reading, the wiki and VCMI: Wisdom +1/+2/+3, Necromancy's `raises` Skeletons at
  every level, the three parts +5/+10/+15 % with the skill, the Cloak's own 30 % without it and
  its `raises` by the Necromancy level at priority 1 (no longer `pending`), `learn_level = 2`.
  `packs/README.md`, `synthetic-ext`'s Storm Regalia carries both kinds, `06-security.md` the
  heroes' changes after a battle. The ten battle and six adventure replays re-recorded with their
  options: only the core ruleset hash in their headers moved.

**`adventure/e08-t19a2b`** (`cross-lane`: `h3-heroes`, `heroes.toml`):
- **`h3-heroes`**: `necromancy` → `Necromancy { share: Rate, creature }` (HER-NEC-001: the
  `RateSum` of the `necromancy` values of the grown effects, a share with `unless_skill` only
  without that skill, at most 100 %, under Q-0005 the single-precision sum; HER-NEC-002: the
  `raises` of the highest priority, decision 30's tie rule, `by_level` by the hero's level in the
  skill); `learn_level` (HER-SPL-005) and `learns_spell` (HER-SPL-006: a worn spell book, a new
  hero spell, a level at most `learn_level`); `HeroChange::LearnSpell` (`ChangeError::Spell`,
  `KnownSpell`); the worn types' filter shared as `worn_types`.
- **`h3-adventure`**: `AdventureState::raise` (GAM-BAT-006): the hero's stack of the creature,
  else its first free slot, else (2c + 2) / 3 of the first upgrade into the upgrade's stack.
- **`h3-game`**: in `heroes_after_battle` (the share, the creature and the count taken before the
  experience is given, as the original's 0x4775A1 before 0x477747: an independent reviewer's
  note, for when experience gives levels), after the experience: `raised_count` over the loser's
  records (every record that lost creatures, the war machines and created stacks too; under Q-0005
  the original's x87 arithmetic per record: 10 imps at 70 % raise 6, not 7) into `raise`, told as
  `GameEvent::CreaturesRaised` to the owner; then each Eagle Eye pick the winner may learn as a
  `LearnSpell` change (GAM-BAT-007); then the capture. `ADV-STUB-3` closed, GAM-BAT-002 complete.
  No version moves: the details are hashed since v5, no golden replay ends a battle.
- **Properties** (`h3-map`): a battle hero's learned spells follow its `LearnSpell` changes, only
  a staying winner learns; raised creatures go to a battle hero that stays, told to its owner, and
  its army holds them; the views see `CreaturesRaised` as their own heroes' changes.
- Rules: HER-NEC-001 (Q-0005), HER-NEC-002, HER-SPL-005, HER-SPL-006, GAM-BAT-006 (Q-0005),
  GAM-BAT-007 new; HER-CHG-001 and GAM-BAT-002 completed.
- Not built (QUEUE inbox): the Necropolis buildings' +10/+20 % (towns, E09); the original's
  exception for the four elementals' upgrade (a game flag, no hero raises them); its 32-bit
  product, which wraps above 2^31; VCMI's different full-army rule (any upgrade in the army, its
  hit points) as a batch E3 row.

### Round 3 of increment 3: T19b1 (2026-10-14, `adventure/e08-t19b1` #211)

#205 and #206 had merged; the baseline full verify of `master` (837cef2, `--with-original-data`)
passed every step. Asked for a seam, the user split T19b (decision 34): T19b1, everything that
changes behaviour or a version, in one pull request; T19b2, the property plans to a victory, next.
The user put the two shared docs that go stale (`02-architecture.md` §8's table,
`07-simulation-and-ai.md`'s env line) into this pull request (`cross-lane`).

- **Action encoding v3** (`h3-sim/src/adventure/action.rs`; ADR-0017 point 8): `Action` is
  `Adventure(MapAction)` (v2's entry and tile) or `Battle(h3_sim::action::Action)` (the battle
  lane's encoding, version 13, for the side the player owns); `Phase` by `state.battle(player)`;
  `battle_mask`, `spell_mask`, `target_mask`, `second_mask` delegate to the battle encoding for
  the player's side (empty outside a battle and while the other side is to act); in the battle
  phase the map masks are empty (`legal_commands` lists the battle's commands, `legal_moves`
  nothing); `encode` takes the rules (a battle command needs the battle's state). `ADV-STUB-4`
  closed: the filter of other players' heroes' tiles is gone, such a move is a legal destination
  (GAM-BAT-004's visit).
- **The env**: `to_move` yields, in id order, the players whose turn runs and who are in no
  battle, and each battle's owner of the side to act (the attacked player acts outside its turn);
  `phase`, the four battle masks (empty once the episode is over). The neutral seat stays
  T20b's (decision 29).
- **The random agent** plays a battle with the battle's own agent (`h3_sim::rollout::choose`:
  uniform among the commands that stay in the battle and the casts; it never retreats or
  surrenders, so a battle ends by a victory or a draw).
- **Replay format v3** (`adventure::replay`, `h3-cli`'s file format): each command with its
  `Actor`; a battle command written `<player> battle <command>`, the battle replays' text without
  the side letter (the side is the actor's in its battle). A v2 file parses and is refused by its
  version.
- **`h3-game`**: `Sink::RECORDS` (true by default, false for `NoopSink`), passed to the battle's
  sink: a quiet sink no longer has the walkers' paths built hex by hex (a test plays a battle with
  both kinds of sink: the same states, the same events but the moves).
- **Golden replays**: the six re-recorded with their options (five changed only their format
  line; `duel-s892-d28`'s moves changed from its 75th command, where the agent's tile choice counts
  another player's hero, and it still accepts the Cartographer and the border guard's pass). Random
  play seldom fights: of `--seed N --agent-seed N --days 28` for N up to 1 500 on the duel, 9
  games have a battle (seeds 120, 521, 770, 792, 987, 1056, 1120, 1294, 1314; the same in
  `base:strict-fixed`). Two joined the matrix: `duel-s792-d28` (blue's hero attacks red's, wins
  after 242 battle commands and captures the backpack's artifact 7) and `duel-s120-strict-d28`
  (strict: the obstacles from the attacker's stream; red's underground hero attacks blue's and
  loses after 148).
- **Benchmarks** (`h3 adventure bench`, `--battle 1` for `fork`): the XL map's heroes never met
  in 200 days of random play (30 seeds), so with `--battle` the heroes seek each other (each step
  the acting player's move that ends nearest to another player's hero on its level, an attack
  nearest of all; `EndTurn` when none gets nearer; `Decline` an offer) until a battle opens: on
  the XL map after 76 steps, on day 4; on the duel after 22. Baselines updated once: `fork-xl-144`
  354 800 024 Ir for 100 000 forks with a battle open (3 548 Ir a fork; master's 7-day state
  counted 354 000 055: a battle open costs 0.2 %, it is shared until its first write);
  `random-play-duel-36` 396 362 609 (master 395 025 919, +0.34 %: the battle-phase checks; the
  same 667 steps), `reach-xl-144` 557 265 701 (master 557 262 308; the same 1 904 queries). The
  master counts carried T19a1's unbaselined +0.87 to +1.32 %. Wall time: a fork with a battle open
  ≈ 0.40 µs on the XL map, ≈ 0.33 µs on the duel.
- **Tests** (`h3-sim`): the property test of steps against masks and `apply` runs on the duel with
  Cartographers or with blue's hero two steps from red's (`battle_setup`), with picks that attack
  and arbitrary battle actions: in each state every player's map and battle masks agree with the
  game and the battle encoding (a sample of legal battle entries decodes, encodes and decodes
  again), a player to move has a legal action in its phase and no other player has one; a battle
  opens by the move, only the side to act's owner steps it (`NotYourSide`), the random agent plays
  it to its end, a fork with a battle open evolves as its parent, a replay records the attacked
  player's commands with its actor and refuses them under the other owner. `h3-cli`: battle
  commands round-trip through their text; the fork workload with `battle`.
- Versions: action encoding v3, adventure replay format v3; no hash moved.

### Round 3 of increment 3: T19b2 (2026-10-14, `shared/e08-t19b2` #213, `adventure/e08-t19b2` #214)

#211 had merged; the baseline full verify of `master` (f67aac8, `--with-original-data`) passed
every step. The user confirmed the seam T19b2 + T20a1, two pull requests (decision 35). Asked
whether Eagle Eye could hook the battle's events: Eagle Eye is the battle's (BAT-MAG-069 picks at
the enemy hero's casts, `HeroOutcome.learned`; GAM-BAT-007 and HER-SPL-006 learn), but the game's
legal commands list no casts (a separate action head, ADR-0014 point 9) and `h3-map`'s tests
could not name a cast, so nothing in the plans cast and nothing could be picked: `h3-battle`
became `h3-map`'s dev-dependency (`shared/e08-t19b2`, `arch.toml` alone).

- **The driver** (`crates/h3-map/src/tests/play.rs`): `Step::Fight(n)`, in runs of 16 to 95
  steps among the single steps (`steps()`, one run per 13 entries, at most 40 entries; every
  property of `play.rs` and `view.rs` draws it). On the map `seek`: an open offer declined, else
  the `n`-th of the acting player's moves that end nearest to another player's hero on its level,
  nearer than the hero stands (an attack is nearest of all), `EndTurn` when none gets nearer (the
  fork benchmark's seeking, T19b1). In a battle `fight`: in three steps of four its hero's cast
  while it may (`castable_spells`, `cast_targets`, `second_targets`, picked by `n`), else a strike
  (an attack or a shot) of the active stack, else a command that stays; in one step of 128 a
  retreat or a surrender if its side may.
- **The plans**: one hero in two has skills of its own, Eagle Eye and up to two of Wisdom and
  Necromancy at levels 1 to 3, and wears a Spell Book (in any slot: the import keeps no position);
  without the book a winner learns nothing (HER-SPL-006), and only 8 of the 128 hero types have
  Eagle Eye. Measured on 96 fixed cases: without skills of their own 0 spells learned (3 Eagle
  Eye winners of 109 victories, 71 hero casts); with them but no book 0, and 1 with casts in three
  steps of four instead of one; with the book 2 to 4; on the final 192 cases 11.
- **Legality consistency of a cast**: a cast `apply` accepted is not an option (the game lists
  none); `check_unlisted` checks that its spell was among the battle's castable spells for the
  actor's side and its target among the spell's targets.
- **`the_plans_play_battles_to_a_victory`** (proptest's deterministic runner, 192 cases of
  `(plan, strict, seed, steps)`): 321 battles started, 226 victories, 0 draws, 43 retreats, 15
  surrenders; 5 299 battle commands accepted; 206 hero casts; 235 artifacts captured, 51 raises,
  11 spells learned. It asserts each is above 0 and victories above the rest.
- **What it found**: nothing to fix. 600 cases of each of the three play properties and of the
  four view properties passed. The three play properties take 12.6 s in a debug build (about 6 s
  before), the whole `h3-map` suite 22 s.
- **T19's part of the exit criteria** (audited): the ADR amendments accepted (ADV-Q13, applied by
  T18a #185); the full verify green (f67aac8's baseline; this branch's below); the logic crates at
  100 % (the coverage step); the property tests hold with battles, now played to a victory:
  atomicity and legality consistency with a battle open (`check_legal`'s battle branch, every
  listed battle command applies; casts by `check_unlisted`), ownership isolation
  (`check_isolation`: the battle's partner, a surrender's treasury), conservation (`check_battle_end`:
  only the battle's heroes' armies change, by the outcome and Necromancy's raised stack; treasuries
  by a surrender's gold; `check_heroes_after_battle`: experience, spell points, captures, lost war
  machines, learned spells, each told to its owner), observation hygiene (`view.rs`: the
  comparison ends at the viewer's battle, the events and the views agree through battles),
  clone equivalence and determinism (`forks_evolve_as_their_parent`,
  `replays_are_deterministic`); golden replays with battles in the determinism matrix (T19b1:
  `duel-s792-d28`, `duel-s120-strict-d28`); the benchmarks re-baselined with a battle open (T19b1);
  GAM-HER-001 and GAM-BAT-001 to -007 and the HER rules they rest on `implemented` with sources;
  no adventure stub open. Left to the increment: monsters (T20, T21) and batch E3 (T22).
- No version moved. Round 3 closed.

### Round 4 of increment 3: T20a1 (2026-10-14, `adventure/e08-t20a1` #215, `cross-lane`)

Decision 35's second part, built on `adventure/e08-t19b2` (#214; #213 the shared change). T20a's
`h3-heroes` part, ahead of T20b's monster answer, its first reader; `cross-lane` (`h3-heroes`,
the harvester's gap-fill and `packs/` are the battle lane's).

- **The reading** (survey `E08-battles.md` §3.1 had asked whether +0x476 includes the artifacts):
  `hero_equip_artifact` (0x4E2A00) adds the equipped artifact's four bytes of
  `artifact_primary_skills` (0x63E758) to the hero record's +0x476 to +0x479 (0x4E2B7E to
  0x4E2B99) and, for a combination, each part's first (0x4E2AF7 to 0x4E2B0D);
  `hero_remove_artifact` (0x4E2BD0, new in the address map) subtracts them (0x4E2C72, 0x4E2D14).
  The bytes are signed and unclamped; their readers clamp (`hero_attack_defense_factor` 0x4E5400:
  0–99). `map_hero_setup` equips the map's artifacts (0x4866D5, 0x48675C) before it sets the
  spell points from +0x479, so HER-MAN-001's effective Knowledge is the original's (its open
  question closed). VCMI's `getValueForDiplomacy` reads the bonus system's totals, artifacts
  included. A base of 99 with 29 or more from artifacts would wrap in the original (the wiki:
  "At 129 the skill value is reset to 0"); HER-BAT-001 clamps. Asked, the user kept the clamp at
  99, without the wrap and without a flag (2026-10-15: the engine keeps the base and the
  artifacts apart and stores no sum, and the readers cap at 99 (0x4E5400; HER-BAT-001 by the
  wiki's "The upper limit is 99"), so a cap at 127 would change nothing a rule reads).
- **`h3_heroes::attack_defense`** (HER-ADV-001): `derive_battle_hero`'s effective Attack and
  Defense (HER-BAT-001's `effective_primary`, 0–99), the base values without hero parameters.
- **`h3_heroes::join_bonus`** (HER-ADV-002) and `MAX_JOIN_BONUS` (3): the sum of the `join_bonus`
  levels (skills at their level, each artifact type once, the specialty), at most 3; no skill read
  by name.
- **Diplomacy's values**: the gap-fill's `[[heroes.skill_extra]]` gives Diplomacy `join_bonus` 1,
  2, 3 beside the table's surrender discount (sources: the code reading, the wiki, VCMI 1.7.5's
  `WANDERING_CREATURES_JOIN_BONUS` 1/2/3); the harvester's run changes only those three lines of
  `packs/base/core.toml`; `h3-map`'s `diplomacy_gives_a_join_bonus_of_its_level` checks them.
- **Golden replays**: the eight re-recorded with their options; only the core ruleset hash in
  their headers moved (the battle replays use `synthetic`).
- Tests: `h3-heroes` (the battle hero's values, the bounds, the combination's parts, without hero
  parameters; the levels, an artifact type once, the cap). Versions: none (the core ruleset
  hash's value only).

### Round 4 of increment 3: T20a2a (2026-10-15, `adventure/e08-t20a2a` #220)

Decision 36 (the seam, confirmed before building): T20a2 in two pull requests, T20a2a with every
version bump, T20a2b (guarded tiles, the store passed through) without one. Baseline: the full
verify `--with-original-data` of `master` 0132a15 (#215 merged) green. The user, offline after the
seam, had the session stage the sprite archives (`H3sprite.lod`, `H3ab_spr.lod`) for the grounds'
`.msk` sizes.

- **The kinds** (part v7): `ObjectKind::Monster { levels }` (none: the map names the creature;
  else a random monster's levels), visited from a neighbour (the compiler checks it), and
  `ObjectKind::Ground { battlefield }` (a battle part's key, which `GameRuleset::new` checks as
  the coast's); neither has a visit yet (`effect` none: a step onto a monster is refused as an
  unbuilt visit, T20b builds its answer). `[params.monsters]`: the aggression by disposition and
  the count words' thresholds (data, ADR-0006); per pack `not_random_monsters` (AB's six, ids
  132–137). `AdventureRuleset::random_monsters(core, levels)`: the core's creatures of the levels,
  in core order, no war machine (the flag: the original's id range 0–144 leaves out 145–149), none
  a pack leaves out; a random type whose levels hold none does not compile. The base pack's nine
  monster types and two grounds, SoD's nine grounds (223 and 230 on Cursed Ground and Magic
  Plains, D-4; 225 Favorable Winds stays untyped: no battle effect, VCMI's "ship" is a boat
  battle). RoE alone draws among 118 creatures, the three packs among 135; the elementals by
  BAT-QUIRK-19's levels in faithful.
- **The setup and the state** (adventure state hash v9): `ObjectSetup.drawn` (the tiles a
  picture lies on beyond its masks: grounds only) and `.monster` (`MonsterSetup`: creature, count,
  disposition, never flees, does not grow, treasure), checked by `SetupError::Monster` (data
  exactly for a monster type; a fixed monster's creature known and no war machine, a random one's
  none; the count at most `MAX_MONSTER_COUNT` 4 095; the disposition the parameters'; the treasure's
  resources known and distinct, 1 to `MAX_GAIN`). `ObjectSetup` lost `Copy`. The static map's
  `MapObject.drawn`, `.monster` (`Monster`: the draws' result) and `lies_on`; `tiles` includes the
  drawn tiles (a ground shows where it lies). The draws after the pickups', three walks in object
  order: the random creatures (equal chances), the counts of 0 (the creature's adventure range, at
  least 1), the aggressions (no draw for a single value, as the original's compliant and savage).
  `AdventureSetup.difficulty` (the game's, a static input) and `AdventureState::difficulty`;
  `army_left` (armies battles leave on objects, none until T20b writes one; `monster_count` reads
  it) in the state and its hash.
- **Grounds** (ADV-GRD-001, GAM-BAT-001's last step): `battle_ground` takes the last ground in map
  order lying on the tile, which replaces the battlefield (the coast's too) and keeps the terrain.
  The reading (2026-10-15): `place_object_tiles` (0x505B20) puts an object on its template's width
  × height (H3API: from the picture's `.msk`; `msk_load` 0x514610, `default.msk` 1 × 1), clipped to
  the map; a flat template (grounds) has layer 0 on every tile (0x505060), and `tile_draw_insert`
  (0x505230, now read) keeps objects of equal layers in map order, going before an earlier one
  only where that one's layer is higher on a shared tile; `tile_special_ground` (0x4FCE20) walks
  the draw list from the top. The wiki agrees ("the topmost terrain effect applies"); VCMI takes
  the first ground in object order (`battleGetBattlefieldType`), a difference on 142 tiles of 9
  shipped maps. The sizes: `h3_map`'s `codes::GROUND_PICTURES`, the first two bytes of each
  ground picture's `.msk` (72 pictures: 6 × 4, 8 × 6, 6 × 6, 8 × 3, 3 × 6, 3 × 3, 5 × 2, 2 × 5).
  **Agent decision (provisional):** grounds are left out of `SharedAnchor` (5 shipped maps stack
  two grounds of one type on one anchor; nothing names a ground).
- **The view** (ADV-FOG-006): `ObjectView.drawn` and `.monster` (`MonsterView`: the creature and
  `MonsterParams::count_word` of the count as it stands); `observe` takes the part.
- **The import** (v6, ADV-MON-001): the nine monster classes (class 54's subclass the creature),
  the body's data, the treasure's resources above 0; a treasure with an artifact is
  `Unsupported::MonsterArtifact` (decision 22), kept without it in `import_map`; the grounds with
  their pictures' tiles. The census (`examples/original_data.rs`): 14 021 monsters, 357
  treasures (55 with an artifact, none below 0), no disposition beyond 4; 1 887 grounds, every
  picture in the table, 10 939 tiles under two or more. 4 maps import whole (`Pandora's Box `
  is now unsupported for a monster's artifact, as 7 more).
- **Replays and versions:** the eight golden replays re-recorded with their options (commands
  unchanged). Part v7, map import v6, adventure state hash v9; the game state hash v6 unchanged.
- Tests: the kinds, parameters and their checks and hashes (`ruleset/tests.rs`), the draws by
  hand over seeds, each setup check, the hashes, a ground's battlefield, the view, a monster not
  visited (`state/tests/monsters.rs`), the packs (`h3-pack-adventure`), the import, the parts'
  battlefields (`h3-game`).

### Round 4 of increment 3: T20a2b (2026-10-15, `adventure/e08-t20a2b` #223)

Decision 36's second part, built on `adventure/e08-t20a2a`; no version bump.

- **Guarded tiles** (ADV-MON-005): `AdventureState::guard_of(part, pos)`, the original's scan
  (0x41A460, survey §2.1): the 3 × 3 block clipped to the map, columns from the left, each from the
  top, the row above left out when `pos`'s object is not visited from above (`closed_above`); the
  first tile whose object (`tile_object`: the last still on the map visited from it) is a monster
  with `pos`'s water status (the part's `Passage::Water`). The tile itself is in the block, as
  the original's (no hero stands on a monster's tile). A removed monster guards nothing: the
  guard follows from the static map and the occupancy, no per-tile flag in the state. The static
  map's `near_monsters` bit per tile (within one step of a monster's visitable tile; derived, not
  hashed) lets the tiles near no monster skip the scan.
- **The step** (ADV-MON-006): `MoveHero` checks each entered tile; a guarded one is the path's
  last (`PastVisit` for a later step), the hero's start may be guarded. After the move and the
  tile's own visit, `visit_guard` (`ADV-STUB-5`: the monster's answer, T20b) returns the visit's
  outcome unchanged. A step toward another hero or a neighbour-visited object does not enter, so
  it is not tested (the original tests the hero's tile after a visit from the neighbour too:
  T20b, with the answer).
- **The query** (ADV-MON-006): `Reach` makes a guarded tile a destination (as a visit's), its
  own start excepted; the property `the_query_and_move_hero_agree` now places monsters (`M` in
  `testing::world`: five of the first creature, aggressive). `bench-check`: the test in the hot
  loop cost the reach workload 11.6 % (random play 12.3 %) on the fixtures, which hold no
  monster; asked only of a tile the search improves, on a map with a monster (`StaticMap`'s
  `any_monster`), out of line (`Search::guarded`, cold), it costs 0.3 %. The fixtures gain
  monsters in T20b, which measures the cost on a map with them.
- **The store**: `AdventureState::apply(core, part, heroes, player, command, sink)` and
  `reach(core, part, heroes, hero)` take the game's `&[Arc<HeroDetails>]` (`h3-heroes` a
  dependency of `h3-adventure`, which `arch.toml` allowed since T18a); `move_hero` hands it to
  `visit_guard`, T20b's first reader; `reach` reads it from increment 4 (movement skills).
  `h3-game` splits its borrow (`GameState::adventure_with_heroes`, replacing `adventure_mut`);
  `legal_moves`, the env's decoder and the tests pass `state.heroes()`.
- Tests: the scan's order, clipping, water, the row above, a removal; the step past and onto a
  guarded tile, leaving one; the query along a corridor and from a guarded start
  (`state/tests/monsters.rs`); the reach property with monsters.

### Round 4 of increment 3: T20b1 (2026-10-15, `adventure/e08-t20b1`)

#220 and #223 merged before the session; `master` 615286e verified already (#223's record:
the baseline ran the light gates only). Asked for a seam, the user split T20b in three (decision
37: T20b1 and T20b2 this session, T20b3 next) and confirmed T20a2a's provisional call (special
grounds out of the one-type-per-anchor check).

- **The answer** (ADV-MON-007, `answer.rs`): `Temper::new` from the visitor's army, its details
  (`h3_heroes::attack_defense`, `join_bonus`; none: 0) and the monster as it stands (`Wanderer`:
  creature, count with a battle's survivors, aggression, never flees) on the game's difficulty;
  `first` (fight above C, join up to Sy + L + 1, join for gold up to Sy + 2L + 1, else
  `after_join`) and `after_join` (insulted at M = C or never flees: fight; else the flight). The
  code reading checked again (0x4A7740, 0x427650, 0x4E5400, 0x4A75C0, 0x47B1A0, 0x47B220,
  0x44AC80, 0x44ACE0): the power is the army's value stored as a float times the factor stored
  as a float, truncated (5.95 as a float is 5.9499998: Attack and Defense 99 give 5 949 for a value
  of 1 000); the ratio's operands are stored as floats and divided at 53 bits, then stored; the
  comparisons are with doubles (0.333's double lies below 0.333 as a float, so r = 0.333 is −2).
  The related type in the content: the first upgrade, else the creature whose first upgrade it is,
  of its faction and level (the original's town table by the record's faction and level; under
  `BAT-QUIRK-19` the Ice, Magma and Energy Elementals have none, their bases keep them; the RoE
  exceptions at map kind 0 do not apply to a SoD game). Not modelled: the 32-bit army value's wrap
  above 2³¹ (about 27 000 Azure Dragons).
- **The offers** (ADV-MON-008): `OfferKind::{Join, JoinForGold { price }, Pursue}` (the price:
  the creature's cost in `[params.monsters] price_resource`, the base pack's gold, times the count,
  empty when it costs none: adventure ruleset part v8); `AdventureState::accept_offer` returns an
  outcome (a join: the group into its creature's stack or the first free slot through
  `add_stack`, shared with `raise`; `CreaturesJoined` to the owner; the treasure as gains; the
  monster removed; a pursuit: the fight); `decline_offer` (new, infallible; a declined join goes on
  to `after_join`, a declined flight removes the monster without its treasure). An offer to join
  is skipped as declined without room (`ADV-STUB-2` in `answer_to`). The fight is `ADV-STUB-6`
  (`state/monsters.rs::fight`): nothing happens until T20b2.
- **The visits**: `Effect::Monster` makes a monster visitable from a neighbour (a step onto its
  tile was refused); `Reach` lists it. The guard's visit after an entering step is the monster's
  answer (`ADV-STUB-5` closed); when the tile's own object made an offer (a Cartographer beside a
  monster), the outcome carries `guard_after` instead, the game keeps it on the decision and calls
  the new `AdventureState::visit_guard` after the answer, or at once when the offer does not open.
- **The game**: `GameState::settle_outcome` applies an outcome (gains, the offer, the battle) and
  what follows an offer that does not open: a join for gold the treasury cannot pay is declined
  without an event and its chain goes on; another refusal is told and the waiting guard visits.
  `answer` declines through the adventure too, then settles, then the waiting guard. Game state
  hash v7 (the offer kinds' tags 2 to 4 and `guard_after`).
- Tests: `answer/tests.rs` (the power's arithmetic, the step's thresholds, the sympathy, Easy, the
  tests' order), `state/tests/answers.rs` (visits, each answer, room, declines, joins, gone offers,
  the guard after a step and after an offer), `h3-game`'s `tests/monsters.rs` (the decisions
  through `apply`, the chain, the unpayable price, the waiting guard). The eight golden replays
  re-recorded with their options: commands unchanged (the duel holds no monster).

### Round 4 of increment 3: T20b2 (2026-10-15, `shared/e08-t20b2`, `adventure/e08-t20b2`)

Decision 37's second part, on T20b1's branch (#228) with `shared/e08-t20b2` (#230) merged in.

**`shared/e08-t20b2`** (#230, `cross-lane`): `h3_content::SPLIT_FROM_TILE` (flag 24,
`quirk.adventure.split_from_tile`), on in `faithful` (`packs/base/core.toml`, the harvester's
profile lines) and the six `faithful` golden replays re-recorded (the core ruleset hash);
`06-security.md`: what each viewer learns of a wandering monster (the inbox's item, survey §9).
The profile lines were to be T20b2's `cross-lane` part (decision 37); they came here because
`h3-pack-battle`'s `the_profiles_differ_exactly_by_the_quirk_flags` needs `faithful` to set every
registered flag, so T20b2 stays in its lane.

**T20b2 — the monster's battle** (`adventure/e08-t20b2`, in the lane):
- **The split** (ADV-MON-009, `split.rs`): `h3_adventure::split(core, visitor, (creature, count),
  percent)` and `TileRand` (the C runtime's `rand`, 214 013 and 2 531 011, bits 16 to 30, seeded
  with the tile's 248 071 x + 276 589 y + 310 111 z + 155 303). The code reading checked again
  (0x4AC580, the table 0x6776E0 = 3.0, 2.0, 1.5, 1.0, 0.67, 0.5 read from its fifth entry down,
  0x50B230, 0x617E5C, 0x47B120): the ratio in doubles (a strength of 0 gives 2 stacks, 7 for a
  worthless army: the FPU's infinity and NaN), the ±1 draw, the clamp, the even stacks, the second
  draw only for a town's unupgraded creature in at least two stacks (stack n / 2). The tile
  (12, 27) underground draws 44 then 50, the survey's D-11 example. The thresholds stay code
  constants with their addresses, as the answer's (T20b1).
- **The battle**: the fight (`state/monsters.rs::fight`, `ADV-STUB-6` closed) and an accepted
  pursuit return `BattleStart { defender: Defender::Monster { creature, count }, tile: the
  monster's }`. `GameState::start_battle` splits it (under `ADV-QUIRK-6` from `TileRand`, else
  from the attacker's stream after the battle's seed and before the obstacles', so a hero battle
  draws as before); the defending side has no hero, no owner, no war machines, the default
  formation, no funds. `Actor::System` (ADR-0017 point 2): `GameCommand::Battle` goes to the first
  battle whose side to act is unowned (`GameState::unowned_battle`, `OpenBattle::unowned_to_act`),
  any other command `NotForSystem`; `legal_commands(System)` lists that side's commands,
  `legal_moves(System)` none. The game state hash stays v7 (the battle's encoding has no new
  field: the monster is found at the battle's tile).
- **The outcome** (ADV-MON-010): `GameState::end_battle` (with the battle's tile), after the
  heroes' part, calls `AdventureState::end_monster_battle` for a battle whose defender has no
  hero: every surviving stack counts as the monster's creature (A-6 fixed, `ADV-QUIRK-7` without
  a flag), kept in `army_left`; none left removes the monster (and its `army_left`), with the
  treasure's resources for the visitor's owner after a victory.
- **The env and the replays**: the env yields no seat for the unowned side (`ADV-STUB-7` at
  `to_move`; the fixtures hold no monster). The replay writer must name the new actor, so the
  adventure replay format v4 (`system battle <command>`) came now, not with T20b3; the eight golden
  replays re-recorded with their options (headers only: the format).
- Tests: `split/tests.rs` (the generator, the table, the draws, even stacks, the upgraded middle
  stack, no strength), `state/tests/answers.rs` (the battle's start, the guard's at its tile, the
  outcome's three cases), `h3-game`'s `tests/monsters.rs` (the split by the tile in the battle,
  the system actor's commands and refusals, a won and a lost battle played out), `h3-cli`'s
  format v4.

### Round 4 of increment 3: T20b3a (2026-10-15, `adventure/e08-t20b3a` #234)

Decision 38: T20b3 in two, this session T20b3a alone. Baseline: master `ae6d517` (#228, #230, #231
merged), its code already verified (the setup's baseline ran the light gates only).

- **The neutral seat** (ADR-0017 point 8, decision 27; `ADV-STUB-7` closed): `h3_sim::adventure::Seat
  { Player(PlayerId), Neutral }` with `actor()` (the player, or `Actor::System`), `player()` and
  `From<PlayerId>`. `AdventureEnv::to_move` yields the players as before, then `Seat::Neutral`
  while `GameState::unowned_battle` has a battle (none once the episode is over). The env's and
  `action`'s functions take `impl Into<Seat>`, so a caller with a `PlayerId` is unchanged:
  `phase` (the battle phase, always for the neutral seat), `battle_of` (for the neutral seat the
  first battle whose side to act no player owns, and that side), the battle masks, `decode` and
  `encode` (a battle action for the neutral seat's battle; no map action decodes or encodes for
  it), `moves` (none), the map masks (empty: `legal_commands(System)` lists only battle commands),
  `step`, `step_with`, `step_decoded`, `step_command` (issued as the seat's actor).
  `neutral_observation` is the neutral seat's battle as its side sees it (`h3_battle::observe`);
  a player's `observation` is unchanged. `rollout::choose` plays the neutral seat with the
  battle's random agent (`EndTurn`, which the seat cannot issue, when no battle waits for it);
  `random_rollout` and `Replay::record_random` step every seat and record the neutral seat's
  commands as the system actor's (replay format v4, T20b2). Action encoding v3 is unchanged (the
  seat is not part of an action). `h3-cli`'s benchmarks drive seats (the seek walk stops at any
  battle; the reach workload queries players only; the agent rollout reads the neutral seat's
  observation).
- **No guard after a visit from the neighbouring tile** (ADV-MON-006): the inbox's item (from
  T20a2b) read the address map's summary of 0x480090 ("after a hero's step: hero_move_step,
  visit_from_neighbour"). The code: 0x480090's arguments are the hero, the step's object, a skip
  flag, the x and y the hero stood on before the step (hero_move_step saves them at 0x48063F to
  0x480652), the move's end and a flag; it skips the guard test when the hero's x and y equal
  them (0x4800B8 to 0x4800C9), and `visit_from_neighbour` (0x480240) passes the hero's own tile
  (0x48034F to 0x480365), so a visit from the neighbouring tile (a pickup, another monster, a
  hero) tests no guard, even from a guarded tile. The only other caller of `guard_attack`
  (0x481900) is the landing from a boat (0x49E670, increment 4). VCMI 1.7.5 agrees
  (`CGameHandler::moveHero`: a blocking visit moves with `IGNORE_GUARDS`, a landing with
  `CHECK_FOR_GUARDS`). The engine already tested the guard only after an entering step: a test
  pins it (`a_visit_from_a_guarded_tile_tests_no_guard`: a pile and a second monster beside a
  guarded hero); the catalog, the survey §2.2 and the map entries of 0x480090 and 0x480240
  corrected.
- Tests: `h3-sim`'s `adventure/tests/monsters.rs` (`monster_setup`: 30 savage pikemen that never
  flee beside red's hero; the seat's accessors; the neutral seat outside a battle; the battle
  played to its end with one seat at a time, each seat's masks and the other's refusals; the
  rollout through the battle to the day limit; a replay with the system actor's commands; a
  random recording that meets the monster), the property `steps_agree_with_the_masks_and_the_game`
  over every seat with the monster setup as a third kind (free picks may name the neutral seat).
  The eight golden replays re-record byte-identical with their options (the duel holds no
  monster).
- Not changed (shared): `docs/07-simulation-and-ai.md` still says `to_move` yields the owner of
  the side to act (T19b1); the neutral seat goes into it with the next `shared/` change (the
  QUEUE's inbox).

### Round 4 of increment 3: T20b3b1 (2026-10-15, `adventure/e08-t20b3b1` #236)

Decisions 39 and 40. Baseline: master `c76d3c02` (#234 merged after E04c T9), the full verify with
original data passed (code `cc3daf7d`).

- **The plans' monsters and grounds** (`h3-map`'s `tests/play.rs`): seven kinds appended to
  `KINDS` (indices 20 to 26, after the committed maps' kinds, so the fixtures keep their
  templates and bytes): five monsters of class 54 (`MONSTERS`: six compliant goblins, fourteen
  hostile gogs with 100 gold, five aggressive pikemen, eight savage wolf riders that never flee,
  four friendly centaurs with 50 gold; fixed creatures and counts, no random monster: a drawn
  creature or count's word would show, as a random resource's pile type), Cursed Ground and Magic
  Plains. `masks` gives a monster its one tile, blocked and visited, a ground no tile (it lies on
  its anchor, an unknown picture's one tile); `body` writes a monster's data (AB and SoD store an
  identifier). The strategy draws them among the objects, and in three plans of four puts a
  monster within two tiles of the first hero (on a ground in one case of two; in another one of two
  the hero has expert Diplomacy and the monster is the centaurs); the plans' skills gain
  Diplomacy. Every monster has one object type, so `lay_out`'s twin test keys class 54 by its
  class alone (the setup refused two monsters on an anchor as `SharedAnchor`).
- **The driver**: `actor` (the system while `unowned_battle` has a battle, else the acting
  player), `player_of`, `battle_of`; `options`, `resolve` (now `(Actor, GameCommand)`), `fight`
  and `advance` play the system actor's commands; a free command of a player or the system's
  refused ones go through as before.
- **The checks**: legality (the system's commands only while a monster's side is to act, never a
  move; a declined join goes on to the flight, the answer to a guarded tile's own offer to the
  guard's visit, which may offer too); unlisted commands (the system has none); decisions (a
  monster's offer is never refused, an accepted join tells the creatures joining and removes the
  monster, a declined flight removes it, an accepted pursuit starts its battle); conservation (a
  join adds the whole group to the visitor's army, into its creature's stack or the first free
  slot, and gives the treasure; a flight gives nothing); a monster battle's end
  (`check_monster_battle_end`: beaten, the monster leaves the map with its treasure for the
  visitor's owner, a draw without; else it keeps between one and the creatures it had; no
  surrender, nothing paid); the views (a monster's offer ends the hygiene comparison: its answer
  follows from its hidden aggression).
- **Findings**: (1) a monster that survives a battle with fewer creatures showed a new count word
  untold (`views_move_as_the_events_say`): `AdventureEventKind::MonsterChanged { object_type,
  anchor, count_word }`, seen by the players that see the monster, emitted by
  `end_monster_battle` only when the word changes (ADV-MON-010, ADV-FOG-006). (2) Under
  strict-fixed a monster hidden on the fringe guards the explored tiles beside it, so the
  viewer's legal moves revealed it (`a_viewer_learns_nothing_it_cannot_see`: 139 against 302
  points left on a tile, a detour around the guarded tiles): the user's decision 40, a monster
  shows on the fringe in either profile (`observe`, `object_viewers`; ADV-QUIRK-1's ledger entry,
  ADV-FOG-005). A probe first found the reachable tiles equal with and without such a monster
  (its guard zone covers frontier tiles, which end every path anyway); the paths' points are
  where it shows.
- **The census** (`the_plans_play_battles_to_a_victory`, 192 fixed cases): 309 battles, 26 of
  them against monsters (13 on a special ground, 194 commands of the system actor, 10 monsters
  beaten, 6 visitors lost), 17 offers to join, 2 to join for gold, 28 to flee, 6 groups joined;
  each now asserted. Ten more runs of the four random properties (480 cases each) passed.
- No version bump: the event is not in a hash, a replay or an encoding; the golden replays
  verify unchanged (the duel holds no monster).
- Not changed (shared): `docs/06-security.md`'s monsters (#230) say neither that the new word is
  told nor that strict shows a fringe monster: the QUEUE's inbox, for the next `shared/` change.


### Round 4 of increment 3: T21a (2026-10-15, `adventure/e08-t21a`)

Decision 41: T21 in two, its rule part before T20b3b2, so the fixture maps, replays and
benchmarks change once. Baseline: master `c1309983` (#236 merged), already verified in full
(code `1be73c61`; the sandbox ran the light gates).

- **The setup and the state** (`h3-adventure`): `Guards { army, question }` in `ObjectSetup` and
  `MapObject` (the stacks in slot order), checked as a hero's army (`check_army`) with no war
  machine, on a pile only (a random one included): `SetupError::Guards`. `guards_left`: the
  stacks a battle left (`army_left`; none after a draw), else the map's. The static map hashes the
  guards (adventure state hash v10).
- **The visit** (`state/piles.rs`, ADV-PIK-004): a pickup's visit goes through `visit_pile`:
  while guards stand, a question (`OfferKind::Fight`, no price; game state hash v8) when the
  message has a text, else the battle at once (`fight_or_pick`: `Defender::Guards`, unsplit, at the
  pile's tile, its first visitable tile); none standing, the pickup. Declined, nothing happens;
  accepted, the fight (or the pickup, if another battle left none). `end_unowned_battle` replaces
  the game's call of `end_monster_battle`: a monster's ends as before; a pile's after a win is
  picked up (its contents the gains, its removal told), else its survivors stay (unseen).
- **The game** (`h3-game`): `start_battle` takes the guards' stacks as the defending army, without
  a hero or an owner; `end_battle` calls `end_unowned_battle`.
- **The import** (`h3-map`, map import v7): a guards record's stacks become the pile's guards, the
  message's text the question's flag; a record without stacks is a plain pile; a random stack
  among the guards is `Unsupported::RandomStack` (its static layer keeps the pile unguarded);
  `Unsupported::GuardedPile` removed. The census (`examples/original_data.rs`): 39 piles with a
  guards record, 3 without a stack, 36 guarded (30 ask, 9 with two stacks), no random stack; the
  4 maps refused for a guarded pile before now count for a random hero (19); 4 import whole, as
  before.
- **The code reading** (Heroes3.exe): `visit_guarded_resource` 0x4A4780 passes the visited tile's
  packed position (`visit_object`'s, 0x4A84F0) and the record's army to `adventure_battle`
  0x4AD470 (0x4A4870 to 0x4A4888), so the battle is fought at the pile's tile, as a monster's at
  its own; a battle result of 0 (the attacker won) gives the pile (0x4A48A6). After a draw the
  guards' army has no stack (the survey's "updated by the battle", inferred), so the next visit's
  test (0x4A47E0) is the plain pickup's.
- **The plans** (`tests/play.rs`): two kinds appended (indices 27, 28: 500 gold whose eight
  gremlins and two gnolls ask; four crystals whose twenty skeletons fight at once; `GUARDED`,
  `GUARDED_OBJECTS`), drawn among the objects and, in half the plans, one beside the first hero;
  `check_unowned_battle_end` (the pile picked up after a win, the survivors by slot at most the
  stacks they were, none after a draw), a pile's question in `check_decisions`; the census
  (192 fixed cases): 20 questions, 31 battles against guards (24 won, 6 lost, one open at its
  case's end), beside 19 monster battles, 341 battles in all.
- **The replays**: the eight re-recorded with their options for the versions; their commands
  stayed (the duel holds no guarded pile).
- Not changed (shared): `docs/06-security.md` does not say what a viewer learns of a guarded pile
  (the QUEUE's inbox, with T20b3b1's note). The writer (`h3_map::writer::Pile`) writes no guards
  (T22a's, `cross-lane`: `h3-experiment` builds piles).

### Round 4 of increment 3: T20b3b2 (2026-10-15, `adventure/e08-t20b3b2`, on T21a's branch)

Decisions 39 and 41: the fixture maps, the golden replays and the benchmarks with monsters,
grounds and guarded piles, changed once, after T21a (#239).

- **The fixture maps** (`h3-map`'s `tests/fixtures.rs`): `Placer` takes an object's masks from
  `play.rs`'s `masks` (a monster's one tile), `place_object` places a fixed object (a monster),
  `place_ground` a ground on a land tile, `fits` holds the old checks and keeps a monster three
  tiles or more from every hero on its level. `encounters`, drawn after increment 2's objects (so
  those, the terrain and the heroes stay byte for byte): beside each hero a monster (the plans' five
  kinds in turn, on a ground for every other hero) and a guarded pile (the two kinds in turn), then
  `2 × scale` of each monster kind, guarded kind and ground anywhere. The duel: 14 monsters, 8
  guarded piles, 6 grounds; the 144 map: 168, 72, 68. A first layout with two tiles between a
  monster and a hero boxed the duel's first hero in (its day-1 query reached 3 tiles, every exit
  guarded by the goblins): the env's battle tests found it, so three.
- **The env's and the CLI's tests on the changed duel**: all of `h3-sim`'s held once the hero was
  free; `h3-cli`'s reach workload test now allows fewer than two queries a step (a hero lost, or
  the neutral seat playing a monster's or guards' side).
- **The replays** (nine): the four with fixed options re-recorded (`duel-s1-d28`, `duel-s2-d7`,
  `duel-s5-strict-d7`, `duel-s3-easy-d7`); the four searched ones (`duel-s792-d28`,
  `duel-s892-d28`, `duel-s988-d28`, `duel-s120-strict-d28`) no longer met what they had been
  chosen for and were replaced by five from a search (`--seed N --agent-seed N --days 28`, N up
  to 1 500, `strict-fixed` up to 800; a scratch tool replaying each game's commands and noting its
  offers, answers, joins, battles and their ends, not committed): `duel-s535-d28` (a Cartographer
  accepted, a pile fought at once and picked up, a question declined, a hero battle won with a
  capture), `duel-s1218-d28` (a pursuit won, a key, a question declined, a guard's passage),
  `duel-s6-d28` (a question accepted and its guards beaten, a group joining, a pile's guards
  beating the visitor), `duel-s1429-d28` (a declined join and flight, a join, a monster's battle
  lost, a Cartographer declined), `duel-s100-strict-d28` (a hero battle lost in strict-fixed, the
  winner capturing). Of the 1 500 faithful games: 10 with a hero battle, 3 with a guard's
  passage, 106 with a monster's battle lost, 58 with a pile's guards winning, none with an offer
  to join for gold (no hero of the duel has Diplomacy).
- **The benchmarks** re-baselined: `random-play-duel-36` 396.4 M → 549.2 M Ir (667 → 2 912
  steps, most new ones battle commands, cheaper than map steps), `reach-xl-144` 557.3 M → 489.7 M
  (1 904 → 4 032 queries; guarded tiles end paths), `fork-xl-144` 354.8 M → 355.8 M (the walk to a
  battle: 76 → 18 steps, a monster or pile on the way).
- No version bump: the maps are data; the replays name the duel by content hash.

### Round 5 of increment 3: T21b, the audit (2026-10-19, `shared/e08-t21b-docs`, `adventure/e08-t21b`)

Decision 42: T21b alone, with a `shared/` pull request for the queued notes; T22a next session.
#239 (T21a) and #240 (T20b3b2) had merged; the baseline was `master` 093a930f (the AI lane's
E05 T8 on top): the full verify with the original data, coverage over 101 gated files, passed.

- **The audit of the exit criteria** (increment 3's, as fixed at T17's checkpoint), against
  `master` and this branch:
  - *ADR amendments accepted:* ADV-Q13 (decisions 25–27), applied by T18a (#185); round 4 needed
    none beyond them (ADR-0017 point 8's seats came with the accepted draft).
  - *Full verify green, the logic crates at 100 %:* the baseline's full run above and this
    branch's below (CI cannot start: billing). Since a line only a seeded proptest reaches makes
    the gate flaky (the AI lane's run of 2026-10-18 missed `h3-game`'s `state.rs:569`, the state
    hash of a pending offer to pass), a coverage run of the lane's packages with all 58 proptests
    of the workspace skipped (`cargo llvm-cov -p h3-adventure -p h3-game -p h3-heroes -p h3-map
    -p h3-sim -p h3-cli -- --skip …`) listed every line of `h3-adventure`, `h3-game` and
    `h3-heroes` left unreached: four, all in `h3-adventure`'s `testing.rs` (the `M` monster of
    the `world` fixture, which only the reach property builds). Fixed tests: the offer to pass is
    in the hash (`a_guard_offers_its_key_s_holders_to_pass_without_a_price`), a monster on the
    other level guards nothing on the hero's (`the_query_stays_on_the_heros_level`).
  - *Properties with battles, monsters and guarded piles:* the play properties
    (`every_step_keeps_the_commands_invariants`: atomicity, legality consistency with a battle
    open and the system actor to act, ownership isolation, conservation with
    `check_battle_end`, `check_unowned_battle_end` and `check_heroes_after_battle`;
    `forks_evolve_as_their_parent`; `replays_are_deterministic`) and the view properties
    (hygiene, the views as the events say, a monster on the fringe, `MonsterChanged`) run on the
    plans with monsters (T20b3b1), grounds and guarded piles (T21a); the fixed census
    `the_plans_play_battles_to_a_victory` asserts that the 192 cases meet each (T21a's counts).
    A guarded pile's hiding had no test of its own (`ObjectView` has no guards, and the view
    properties vary no pile's guards): `a_guarded_pile_looks_like_any_pile_to_every_viewer`
    compares each player's observation with the same game's pile unguarded.
  - *Golden replays with battles in the determinism matrix:* nine (`cargo xtask determinism`:
    debug, release, `target-cpu=native`), re-recorded or replaced by E04c T10b: monster battles
    won and lost, joins, declined joins and flights, pursuits, guarded piles fought at once and
    after an accepted question, a pile's guards beating the visitor, hero battles in both
    profiles with captures, a border guard's passage, Cartographers accepted, the easy
    difficulty. Lost with T10b's search: a declined pile question and a declined Cartographer
    (both in `h3-game`'s unit tests and the plans; the queue's inbox).
  - *Benchmarks re-baselined:* T20b3b2 (`random-play-duel-36`, `reach-xl-144`, `fork-xl-144` on
    the maps with monsters and guarded piles), moved since by other lanes' re-baselines (now
    523.5 M, 525.7 M and 355.8 M Ir); `bench-check` passed on the baseline and on this branch
    (+0.02 %, 0.00 %, −0.14 %).
  - *Every new rule `implemented` with sources:* ADV-MON-001 to -010, ADV-GRD-001, ADV-PIK-004,
    GAM-HER-001, GAM-BAT-001 to -007 and the HER rules they rest on (`check-rules`); D-8 to D-13
    rest on the code reading until batch E3 confirms them. Open stubs: `ADV-STUB-2` alone (the
    army dialog, increment 4's feature to-do).
  - *The batch:* not yet: T22a (the spec) and T22b (scored). The harness lacks what E3 needs
    (Yes/No dialogs, battles started from the map, the underground, the memory reads; the client
    lane's inbox since T17b), so T22a plans for the user to play what it cannot.
  - *Docs:* `docs/06-security.md` gains what a viewer learns of a monster after its battle and on
    the fringe and of a guarded pile, `docs/07-simulation-and-ai.md` the seats
    (`shared/e08-t21b-docs`); the play properties' module comment names monsters and guarded
    piles.
- **The 160 shipped maps** (the AI lane's note of 2026-10-18: the user's `maps.tar` had held
  four maps of the user's own, `experiment`, `Untitled` and edited copies of `All for One` and
  `Terrible Rumor`): the baseline's `original_data` run on the 160 (49 RoE, 55 AB, 56 SoD; the
  input-bound peaks unchanged): 2 import whole (`Race for Ardintinny`, `Tutorial.tut`), 128 have
  a hero at a main town, 18 a random hero, 8 a monster with an artifact, 3 a hero's experience, 1
  a hero in a town; 13 863 monsters (355 treasures, 53 with an artifact), 1 861 special grounds
  (10 771 tiles under two or more), 0 of 100 311 visitable tiles visited from two objects, the
  guarded piles as before (36, 30 asking first). Corrected in `docs/formats/h3m.md`, ADV-OBJ-003's
  summary, the HANDOFF, the queue and two comments; the surveys' and the writer's counts
  from one-off censuses are marked as counted over the 164 (decision 42), with the work
  process's staging of `maps.tar` (`shared/e08-t21b-docs`).
- **This branch's full verify** (`scripts/verify-once.sh engine --base origin/master --
  --with-original-data`, 2026-10-19): every step passed on 7092c45 (`verify-once: verified
  (sandbox) code 6e2dc1c9`), coverage over 101 gated files.
- No version moved, no rule changed.

### Round 5 of increment 3: T22a1, the tools for batch E3 (2026-10-19, `adventure/e08-t22a1`, `cross-lane`)

Decision 43: T22a in three (T22a1 the tools, T22a2 the spec, T22a3 the run). The client lane's
E17 H1 and H2 (#272, merged the same evening) had built what T21b found the harness lacking:
Yes/No questions, battles started from the map, the difficulty, the underground, the hero list
scrolled, the start's reads. What still kept batch E3 from a map was the adventure lane's own
tools. The baseline was `master` 7fd92d2 (its engine code verified in full on a620966; the light
gates here).

- **`h3_map::writer`:** `Map.underground: Option<Underground>` (a base terrain, patches and
  objects; no roads): the header's flag, the second level's tiles after the first's, its objects
  after the surface's with z = 1, a player's heroes on it in its hero list after the surface's.
  `Pile.guards: Option<Guards>` (an army of one to seven stacks and a question): the format keeps
  them in the pile's message (the flag, the text, the guards' flag and army, 4 unused bytes), so
  a guarded pile always has a message, its text empty when the guards fight at once (the
  import's `Guards.question`, ADV-PIK-004). The armies of heroes, towns and guards share one
  check and one encoding. Tests: the bodies byte by byte and their mistakes; a read-back test
  writes a two-level map with both kinds of guarded pile, a hero and a monster below, reads it
  back and imports it (both piles guarded, one asking; the hero and the monster on level 1).
- **`h3-experiment`** (`cross-lane`, battle-owned): `[map] difficulty` (`easy` … `impossible`:
  the template's line "Choose it in the scenario list"; the map holds none, the harness's recipe
  names it), `[map.underground]` with `[[map.underground.patch]]`, `rock` patches, an
  encounter's `level` (its heroes and opponent below; the template says so), a monster's
  `disposition` (`compliant` … `savage`, savage the default) and `flees` (false the default: the
  old specs build the same map and template), a pile row's `level`, `guards` and `question`;
  two objects on one tile are checked per level. Tests build each and its mistakes; a spec of
  both levels compiled against the original tables imports and plays in the engine (`h3
  adventure run`). The spec format in `docs/experiments/README.md`, which also gains the
  harness's new reach.
- **The other lanes' code:** `underground: None` in the `Map` literals of h3-cli's AI tests and
  the client's `h3-view` (`situation.rs`), with the user's consent (one `cross-lane` pull
  request).
- No version moves: the writer and the spec compiler are tools (no map import, hash or format
  version); the committed specs' maps are unchanged (`flees` and `disposition` default to the
  old values).
- **For T22a2:** batch E3's rows (survey `E08-battles.md` §13) need two difficulties (E3-1 on
  Easy, E3-2 on Normal): two maps or two runs of one; E3-4 needs the monster on (12, 27) of a
  36 × 36 two-level map, its hero beside it (`level = 1`, `at = [11, 27]`). The client lane's
  harness check (`capture/checks/harness-check.toml`) can now place a monster's offers, a pile's
  question and a walk below (its inbox).


### Round 5 of increment 3: T22a2a, batch E3's monster rows (2026-10-20, `adventure/e08-t22a2a`, `cross-lane`)

Decision 44: T22a2 in two by rows, the monster rows first, the recipes written here. The
baseline was `master` 818dd7e (its engine code bc73ebe verified in full in the sandbox, without
the original data's steps). No code changes; the specs, templates and recipes only.

- **Two batches, two difficulties.** `experiments/specs/E08-batch-3a.toml` on Easy (E3-1,
  E3-1c, E3-2, E3-2c: savage groups of 10 Pikemen that may flee, against 52 Pikemen, 36
  Halberdiers, 56 Pikemen, 39 Halberdiers) and `-3b.toml` on Normal (the same four at the same
  tiles; E3-3 twice, Red's and Blue's; E3-4 below and its control E3-4c; E3-5). Each spec's
  header holds the formulas and a table of every source's outcome; every row's predictions name
  the reading (the engine), VCMI 1.7.5 and the mixed cases.
- **What each row tells.** E3-1c (Halberdiers, the Pikemen's upgrade, counted by every source)
  settles D-10 whatever D-9: a flee offer on Easy (C = 11) and a battle on Normal (C = 10 = M).
  E3-2 settles D-9 whatever D-10 (a flee offer under the reading on either difficulty, a battle
  under VCMI). E3-1 is the survey's row (both together), E3-2c the control (a flee offer for
  every source). E3-3 (D-8): 23 Pikemen against 32 friendly, never-fleeing Halberdiers, r = 0.5
  exactly, four heroes with Diplomacy 0 to 3 in turn, each retreating at once so the monster
  keeps 32; the reading offers to join at L >= M, VCMI at L >= M - 1, so a monster with M = 1 to
  4 tells it (the aggression is drawn at the start, read from the tile's word before the first
  visit and after the last); two monsters: 82 % under uniform draws. E3-4 (D-11): 30 Pikemen at
  (12, 27) below against 10 Griffins: 8, 8, 7 Halberdiers, 7 under the reading (d2 = 50), none
  under VCMI. E3-4c (the review's addition): (29, 10) on the surface, d2 = 50 too but VCMI's
  formula 49 there, so Halberdiers under both: with E3-4 it tells VCMI's formula from a misread
  threshold (d2 < 50: none at both). E3-5 (A-6): the same split on (12, 27) above (d2 = 25, every
  source upgrades), a retreat, the count read (23 under the reading and the wiki, 30 under the
  engine and VCMI), then 5 Archangels' visit (12 and 11, or 15 and 15) and a win. E3-4 and E3-4c
  are A-6 samples too.
- **The engine's outcomes.** A throwaway test in `crates/h3-cli/tests` (never committed) read
  both compiled maps through `h3_cli::adventure::load_map`, played each visit with
  `h3_game::apply` (offers declined, battles retreated, the system side defending) and gave the
  reading's predictions at every row; for E3-3 it searched seeds for each aggression 1 to 7.
- **The starting bonus.** A hot-seat player's random bonus may give a hero an artifact: up to +2
  Attack or Defense leaves every St above unchanged, but voids an E3-3 visit (r > 0.5); every
  answer's visit records the hero's primary skills before it (`before:hero:primary`, +0x476 with
  the artifacts' bonuses).
- **Feasibility.** Each player keeps a hero after its battles (hot seat ends when a player is
  out); Red has eight heroes, Blue eight on the map in 3b (the list scrolled, H2); every attacker
  stands beside its monster, no guard zones overlap, the retreating side is faster or out of
  reach in round 1.
- **The recipes** (`clients/capture/batches/E08-batch-3a.toml`, 4 steps; `-3b.toml`, 19 steps;
  `cross-lane`, the client lane's inbox told): `answer = "no"` for every question (E3-5 B pursues
  a flee offer that only a wrong never-flees bit could bring), `battle` auto or retreat, `expect`
  the reading's outcome where it is fixed, E3-3 without `expect` (it depends on M). A script
  checked every record key against the compiled templates' fields (all filled).
- **The review** (a second agent, the specs, templates and recipes against the survey, the
  packs' AI values, the harness's runbook and code): every number and outcome recomputed and
  matching (VCMI's values 24 and 50 at (12, 27) above and below), D-9 and the M = C fight
  confirmed in VCMI's source; one finding (E3-4's d2 = 50 alone cannot tell VCMI from a strict
  threshold: E3-4c added), two minor ones taken (E3-5 B's answer, E3-3's second aggression read),
  one left (no final `end_turn`: harmless).
- Stamps: E3a `786afe33`, E3b `a2586e5b`. **Next:** T22a2b (the hero rows E3-6 to E3-9 and the
  inbox's rows), then T22a3 runs E3a, E3b and E3c on the harness.

### Round 5 of increment 3: T22a2b, batch E3's hero rows (2026-10-20, `adventure/e08-t22a2b`, `cross-lane`)

Decision 45: the hero rows only, as batch E3c; the inbox's four rows go to increment 4's batch.
The baseline was `master` fee5b69 (its engine code 6f711e8b verified in full in the sandbox, with
the original data). No engine code changes: the spec, template and recipe, one address map entry.

- **One batch on Normal.** `experiments/specs/E08-batch-3c.toml`, three encounters in hot seat:
  E3-7 (a read at the start: the 152 heroes off the map's experience through the harness's
  `heroes:experience`, and E3-7's own, placed with its type's defaults), E3-6 (Clavius with 5 000
  experience, 2/3/1/1 and eight secondary skills at basic, so every level-up's secondary gain
  raises a named one: the harness reads skills by name only), E3-8 (Red's Octavia with five
  Castle stacks attacks Blue's Gelare with five Tower stacks in tight formation; Blue's stacks
  defend until Red's first acts, which surrenders). E3-9 is the defender's formation in E3-8's
  battle (five stacks: loose rows 0, 2, 5, 8, 10 against tight 2, 4, 5, 6, 8, so 83 and 117
  against 15 and 185 tell them apart; the harness check's tight hero had led one stack, which
  stands on row 5 in either).
- **What each row tells.** D-12: a 90 among the 152 confirms the reading's 40 to 90 and refutes
  VCMI's `rand.nextInt(40, 89)` (95.1 % under the reading). E3-6: level 5 for every source (the
  executable's i16 table at 0x679C88, a new map entry; VCMI's `expPerLevel`); VCMI levels up at
  the start (4 primary points, 4 secondary advances: sums 11 and 12), the reading never read
  whether the original does; the early sign (E17 H1, H2) says it does. D-13: the attacker pays
  1 200 (half of 2 400), Blue gains it under the reading, nothing under VCMI's
  `doSurrenderAction`; the early sign had a defender pay, this asks the other direction. E3-9:
  the map's tight flag kept (the reading, the engine) or reset to loose (VCMI's `initHero`).
- **The engine's outcomes.** A throwaway test in `crates/h3-cli/tests` (never committed) read
  the compiled map with E3-6's experience set to 0 (the import refuses 5 000:
  `Unsupported::HeroExperience`, decision 22) and played E3-8 over seeds 1 to 5: E3-7 at level 1
  with 43 to 73, a battle at once, the hexes above (Castle creatures +1 speed on grass, their
  native terrain: the attacker's Swordsmen first), Red 20 000 to 18 800, Blue 20 000 to 21 200,
  E3-8 off the map, E3-9's army whole and its experience 0.
- **The recipe** (`clients/capture/batches/E08-batch-3c.toml`, 4 steps; `cross-lane`, the client
  lane's inbox told): three reads without a walk, then `battle = "surrender"`; `expect` the
  reading's outcome. All 29 record keys fill the compiled template's fields (checked with the
  harness's own `fill`).
- **The review** (a second agent, the spec, template and recipe against the survey, the
  catalog, the packs, HCTRAITS, VCMI 1.7.5 and the harness's code): every number recomputed and
  matching (95.07 %, 1 200, the hexes from the base pack's and VCMI's layouts, level 5, the sums,
  the speed tie), every address and rule id found, the harness able to play it. Taken: E3-6's
  primary sum spoiled by a starting bonus's artifact (the hero's primary skills hold the worn
  artifacts' bonuses: the secondary sum decides); E3-9's experience read before the battle too
  (VCMI predicts 0 as well: a confirmation); D-13's third reading, the gold to the player whose
  turn it is, which E3-8 tells apart (both golds unchanged) where E17 H1's defender's surrender
  could not; E3-9's tight column labelled the engine's (the original's handling not read); a
  harness bug for the client lane's inbox (a winner of -1 prints `defender`, so E3-8 decides by
  `surrendered` and the gold). Left: no final `end_turn` (harmless).
- Stamp: E3c `b026743c`. **Next:** T22a3 runs E3a, E3b and E3c on the harness (its seam to confirm);
  T22b scores batch E3 and closes increment 3.

### Round 5 of increment 3: T22a3, batch E3's run (2026-10-20, `adventure/e08-t22a3`, `cross-lane`)

Decision 46: the three batches in one run, scored by T22b. The baseline was `master` dd89029
(its engine code b4487432 verified there; the light gates passed). The changes: the three filled
templates and one function of the harness (`clients/capture/h3adv.py`, the user's consent).

- **The setup.** The user's copy staged from `original-data/install` (`Heroes3.exe`, the five
  DLLs, the four LODs, `Heroes3.snd`, `H3ab_ahd.snd`, `H3ab_ahd.vid`; about 280 MB in five
  staging calls), `scripts/sandbox-setup.sh --lane adventure` (it copies only the code-reading
  files: the others copied by hand), `clients/capture/setup.sh`; `h3-experiment` built in release
  first. Each batch compiled its spec (the stamps matched the committed templates).
- **The runs.** E3a (Easy, 4 steps) and E3c (Normal, 4 steps) ran at once. E3b (Normal, 19
  steps) stalled at E3-1's battle in two runs: the battle showed the attacker's Pikemen active,
  waiting for orders, after `h3adv.py`'s auto-combat key (sent 0.5 s after the stacks appear),
  and an `a` sent by hand then started auto combat. The game's first battle takes orders later
  than the later ones (the hand-helped run's other battles took the harness's key). The fix
  (`auto_combat`): the key waits for an active stack, and goes again when the same stack stays
  active with every stack unchanged for 10 s (a second key during auto combat would stop it). The
  third E3b run went through by the harness alone. E3a and E3c never reach that path: their runs
  stand. The client lane's inbox is told, and the runbook's traps have the line.
- **Every recipe's `expect` held** (no "anything odd" line in the three templates). The starting
  bonus: E3-1 on Normal showed 1/1/1/1 (an artifact, within the spec's +2); every E3-3 visit 0/0.
- **The raw values T22b scores** (the predictions are in the specs; not judged here). E3a: all
  four a flee offer, declined, no battle, the tile empty. E3b: E3-1 and E3-1c a battle at once,
  E3-2 and E3-2c a flee offer; E3-3r and E3-3b both drew aggression 3, a join offer at L3 only,
  32 Halberdiers kept; E3-4, E3-4c and E3-5 A: the defender 8 Pikemen, 8 Pikemen, 7 Halberdiers,
  7 Pikemen, **23 Pikemen** on the tile after the retreat; E3-5 B: 12 Pikemen and 11
  Halberdiers, won, the tile empty. E3c: 152 heroes off the map at 40 to 90 (3 at 40, 1 at 90);
  E3-7 51, level 1; E3-6 at 5 000 and level 5 with 3/6/1/1 and Pathfinding advanced, Logistics
  expert, Scouting advanced (the other five basic); E3-8 surrendered at once, Red's gold 20 000
  to 18 800, Blue's 20 900 to 22 100, E3-8 off the map with its army whole, E3-9's experience 0
  before and after; the defender's hexes 15, 49, 100, 151, 185.
- **The hand-helped run's E3-3** (kept in the sandbox only, not the batch's record; its one hand
  key came at E3-1's battle, before E3-3): aggression 1 offered to join at L1, L2 and L3;
  aggression 6 at none; every hero 0/0. Its other 43 of 48 records matched the batch's.
- **Next:** T22b scores batch E3 (with the hand-helped run's E3-3 as two extra aggressions) and
  closes increment 3.

### Round 5 of increment 3: T22b, batch E3 scored; increment 3 closed (2026-10-20, `adventure/e08-t22b`, `cross-lane`)

Decision 47: the scoring, E3-9's code reading and the closure in one pull request; increment 4's
refinement next session; docs only (no test pins the batch). `cross-lane` for the status row in
`docs/experiments/README.md` (the user's consent). The baseline was `master` 7ac0b91 (engine code
b4487432, verified by #291's full run with the original data; the light gates passed here).

**T22b — batch E3 scored** (each spec's predictions against `experiments/results/E08-batch-3a.md`,
`-3b.md` and `-3c.md`, played by the harness at T22a3; the stamps matched):

| Row | Observed | Reading (engine) | VCMI 1.7.5 | Verdict |
|---|---|---|---|---|
| E3-1, 52 Pikemen against 10 savage Pikemen | Easy: flee offer, no battle; Normal: a battle at once | flee on Easy (C = 11), a battle on Normal (C = 10 = M) | a battle on both | the reading: D-10 (and D-9) |
| E3-1c, 36 Halberdiers, the same | Easy: flee offer; Normal: a battle at once | the same | a battle on both | the reading: D-10 alone |
| E3-2, 56 Pikemen, the same | flee offer on Easy and Normal | flee (Sy 2: the own type counts) | a battle (Sy 0) | the reading: D-9 |
| E3-2c, 39 Halberdiers (control) | flee offer on both | flee | flee | the shared part holds |
| E3-3r, E3-3b, r = 0.5 exactly against 32 friendly Halberdiers | aggression 3 twice: no question at L0 to L2, a join offer at L3; every visit a battle; 32 kept, aggression 3 after | offers at L ≥ M: L3 | offers at L ≥ M − 1: L2, L3 | the reading: D-8 |
| E3-4, (12, 27) underground, d2 = 50 | 8, 8, 7 Halberdiers, 7 Pikemen; 23 Pikemen after the retreat | Halberdiers; 30 kept (A-6 fixed) | no Halberdiers; 30 | the reading: D-11; A-6 the original's |
| E3-4c, (29, 10), d2 = 50 (control) | the same; 23 | Halberdiers; 30 | Halberdiers; 30 | VCMI's formula and a threshold below 50 refuted with E3-4 |
| E3-5 A, (12, 27), d2 = 25 | 8, 8, 7 Halberdiers, 7 Pikemen; 23 after the retreat | the same; 30 | the same; 30 | A-6 confirmed (the wiki's bug) |
| E3-5 B, 5 Archangels against what is left | 12 Pikemen, 11 Halberdiers; won, the tile empty | after 23: 12 + 11 (after 30: 15 + 15) | 15 + 15 | A-6 confirmed again |
| E3-7, the 152 heroes off the map | 40 to 90 (3 at 40, 1 at 90); E3-7 51, level 1 | 40 to 90 | 40 to 89 | the reading: D-12 |
| E3-6, 5 000 experience, 2/3/1/1, eight basic skills | level 5, 3/6/1/1, Logistics expert, Pathfinding and Scouting advanced | level 5; the gains not read (the import refuses the hero) | level 5 with four level-ups' gains | VCMI's: the gains at load (increment 4) |
| E3-8, the attacker's surrender on its turn | no question; Red 20 000 → 18 800, Blue 20 900 → 22 100; E3-8 off the map, its army whole; E3-9 0 experience after | Red −1 200, Blue +1 200; E3-8 off; 0 | Red −1 200, Blue unchanged | the reading: D-13 |
| E3-9, the defender tight in the map | hexes 15, 49, 100, 151, 185 (loose) | tight: 49, 83, 100, 117, 151 | reset to loose | VCMI's: `ADV-QUIRK-8`, fixed |

- **The extra draws** (the hand-helped E3b run of T22a3, not the batch's record): aggression 1
  offered to join at L1, L2 and L3 (VCMI: L0 too), aggression 6 at none: D-8 a third time.
- The starting bonus: E3-1 on Normal showed 1/1/1/1 (an artifact; St stays 8 at r ≈ 5.46: the
  row stands); Blue's 20 900 gold at the start is its bonus; every E3-3 visit 0/0. Nothing odd
  was noted.
- **The catalog:** ADV-MON-007, ADV-MON-009, GAM-HER-001 and GAM-BAT-005 now `verified` (an
  `observation` source each); observation sources for ADV-MON-008 (the flight, a declined
  join), ADV-MON-010 (A-6 observed, kept fixed), GAM-BAT-001 (the visit; `ADV-QUIRK-8`),
  GAM-BAT-002 (no +500 after a surrender) and GAM-BAT-003. The survey's D-8 to D-13 and A-6
  rows, §6.3 and §6.4 say so; `ADV-QUIRK-7` gained the observation; `docs/experiments/README.md`
  has the batch's row.
- **E3-9's reading** (decision 46's condition; survey §6.4): the formation is bit 0 of the
  hero's byte +0x48 (the hero screen's buttons clear and set it, 0x4DD577, 0x4DD587);
  `map_hero_setup` (0x486590) writes none of +0x48, so the map's flag reaches no hero, whoever
  owns it: **`ADV-QUIRK-8`**, fixed in every profile, no flag (the engine keeps the map's
  flag; its tests: the import's and the battle start's). Found with it:
  `battle_war_machine_stacks` (0x463600) places an army by `tight_formation_slots` (0x63D1DC)
  only when the hero has the bit **and** its side's player is human (+0x54A4), else by
  `loose_formation_slots` (0x63D118), on `battle_army_hexes` (0x63D0A8): a computer player's
  hero always fights loose. Both tables match BAT-SET-001's rows (the battle lane's inbox). Four
  map entries and a struct field (`hero` +0x48).
- **What the original refutes** (the task's call with the user): nothing that needs an engine
  change now. E3-6's gains at load go to increment 4's levels (its inbox line), E3-9 is fixed by
  decision 46, A-6 by decision 23.

**Increment 3 closed.** T21b's audit held every exit criterion but the batch; since then the
engine code changed only by T22a1 (the writer's underground and guards, `h3-experiment`) and
other lanes, and #291's full local verify with the original data passed on that code
(b4487432, all 16 steps). The batch is staged and scored; the HANDOFF, this log and `QUEUE.md`
are current. Increment 4's refinement is next (QUEUE: T23).

### Increment 4 refined: T23 (2026-10-20, `adventure/e08-t23`)

The user merged T22b (#293) and asked for small increments, the seam confirmed first (ADV-Q14):
increment 4 is **hero progression only** (decision 48), this session **T23 alone, docs only**
(decision 49). The baseline was `master` 6f7d601 (engine code 08410436, verified on that commit;
the light gates passed here).

- **Increment 4** (decision 48): experience to levels (the table 0x679C88, the steps × 1.2 beyond
  level 12, the level limit game +0x1F877), a level-up's primary draw and secondary offers as a
  pending decision of the owner, the levels' gains at load (E3-6) lifting the import's refusal of a
  custom experience (decision 22), the battle's experience reaching levels. The rest of the
  former sketch (the movement profile, the objects that give experience, artifacts, spells or
  skills, army management, treasures with an artifact) is the sketch's increment 5; the week's
  increment becomes 6; the order is decided at each refinement.
- **The first round** (decision 49): T24a the progression survey (the table and the level limit,
  the level-up `hero_gain_experience` calls, with the seed +0x8F, the levels at load, what reads the level, disputes and
  quirk candidates, the census, the batch's plan), T24b the ADR amendment drafts (expected
  ADR-0017 and ADR-0013), the design note and the cut, after T24a's checkpoint.
- **The batch** (decision 50): on the harness (`level_up`, `heroes:experience`), with decision
  45's four rows kept (the agent's call).
- **The inbox sorted:** taken by T24a and removed or struck: E3-6's gains at load (T22b), the four
  maps refused for a custom experience (T18b), the level limit (T19a2a), how a custom experience
  becomes a level (T17a), the E04 refinement's progression. Re-tagged for the sketch's increment
  5: the formation command (T22b), the grounds' movement effects (T20a2a), `Pandora's Box ` and the
  artifacts (T20a2a, T19a2a), the army dialog (T17b), flying before a neighbour's visit (T11a), the
  Scouting specialty (T8a), the step price's omissions (T7b); the computer players' daily bonus to
  E10 (T11a); the week's events to the week's increment (T6b). New: where "increment 4" still
  names what moved (two code comments, ADR-0013, ADR-0016).
- **Re-tagged in the lane's docs** (decision 48): `docs/formats/h3m.md` (a monster's artifact),
  `docs/stubs.toml` (`ADV-STUB-2`), `docs/rules/quirks-adventure.md` (the formation command),
  ADV-MON-001, GAM-CAL-001 and GAM-BAT-004's summaries, `E08-design.md` §2 and §4.

### Increment 4, round 1: T24a1, the progression survey's first part (2026-10-20, `adventure/e08-t24a1`)

The user asked for a small increment with the seam confirmed first and cut T24a in two (decision
51): **T24a1** the code reading of experience to levels and the level-up, compared with VCMI;
**T24a2** the rest. Baseline `master` c4fe6b1 (engine code 08410436, verified on 6f7d601; the
light gates passed here). Docs only: `docs/rules/surveys/E08-progression.md` §0–§2 (§3–§7 marked
for T24a2), the address map and the record layouts.

- **Experience to levels** (§1): the table to level 12, then steps trunc(1.2 × the last step)
  (f64, `crt_ftol`); VCMI's table equals it through level 75. The original's thresholds are 32-bit
  sums: level 74 is the last right level; from 1 810 034 207 experience the hero is level 88
  (75–87 skipped), more gives wrapped levels, 2³¹ − 1 hangs the game, a gain past it makes the
  experience negative and the level 0 (A-7). `crt_ftol` stores 64 bits and returns the low word
  (its map entry corrected). The level limit (game +0x1F877) caps the experience and lowers
  experience above it (VCMI keeps it, D-19); a limit above 75 caps at a wrapped threshold (A-8);
  its setter was not found (a block copy of the map header, probably; the census reads the maps'
  limits).
- **The level-up** (§2, `hero_level_up` 0x4DA720): one level at a time, each reseeding the C
  runtime's `rand` with 214 013 L + 156 823 S + 154 079 (S the hero's seed byte +0x8F, never
  advanced; D-14, the recommendation as decision 26). The primary skill by the class's chances,
  the second row by the new level ≥ 10 (VCMI the old, D-15), +1 unclamped (A-9). Two offers
  (`hero_secondary_offer` 0x4DAD00): an advance of a known skill, then a new skill, each with a
  fallback; Wisdom when due, then a school when due, then by the class's weights; due after 3/3
  levels for the magic classes and 6/4 for the might ones, the Elementalist counted as might
  (VCMI magic: D-16, A-10); weights of a due known school and a banned known skill differ from
  VCMI's (D-17, D-18). A human at its own computer answers each level before the next is drawn;
  a computer player's hero and any hero during the new game's setup (the global 0x698400) get
  `ai_level_up_choice` (0x52BBD0) without a dialog: the lead for §3's levels at load.
- Worked vectors (computed, to confirm with the batch): a Knight with basic Leadership reaching
  level 2 with S = 1: seed 738 928, Attack, Leadership or Tactics; with S = 200: Knowledge,
  Leadership or Sorcery.
- **Map**: 12 entries (the level-up, the offer, learning, the dialog, the computer's choice and its
  two value functions, two tables, three globals), three updated (`hero_gain_experience` read in
  full, `new_game_heroes`, `crt_ftol`); `structs.toml`: the hero's class, the two counters, the
  skill slots and count, experience and level signed; the game's level limit, banned skills and a
  campaign field; a new `hero_class` record. `exe.py check` and `cited` clean.
- **Not done here** (T24a2): the wiki, levels at load, what reads the level, the census, the
  batch's plan, the proposed rule ids, a fresh agent's second reading, the ADV-Q for the
  checkpoint.

### Increment 4, round 1: T24a2a, levels at load and what follows the level (2026-10-20, `adventure/e08-t24a2a`)

The user merged T24a1 (#301) and cut T24a2 again (decision 52): **T24a2a** the rest of the code
reading with VCMI; **T24a2b** the wiki, the census, the batch's plan, the rule ids, the second
reading and the ADV-Q. Baseline `master` 810c584 (engine code 08410436, verified on 6f7d601; the
light gates passed here). Docs only: survey `E08-progression.md` §3–§4, §0 updated.

- **Which setup is the placed hero's** (§3.1): `new_game_heroes` calls `placed_heroes_setup`
  (0x4CAE10: per hero type the game's `heroSetup` record, H3API's `H3SetupHero`, the map file's
  placed hero) and, only with a campaign's state, the function E08 T6b to T22b read as the placed
  hero's setup: 0x486590 tests the scenario header's `keepExperience` to `keepArtifacts` and walks
  `campaignInfo`'s `crossoverHeroes`, so it sets up a **campaign's carried-over hero**; renamed
  `carried_hero_setup` (0x487290 `carried_heroes_setup`). The conclusions drawn from it hold on
  the real path (the starting army, the reveal, HER-MAN-001's spell points, no formation): the
  stale citations are in the inbox; one difference, the placed hero's custom primary skills are
  copied unclamped (the import clamps them).
- **Levels at load** (§3.2, `placed_hero_setup` 0x4D8B30): custom skills, army, spells and
  artifacts first, then the experience gained from 0 with the levels given, during the new game's
  setup: the computer's evaluation (0x52BBD0) chooses each two-offer level, the level limit caps
  (VCMI: the first offer, D-20; the limit ignored, D-21).
- **E3-6 against the reading** (§3.3, computed): of the 255 seed bytes, 3 give its primary gains
  and only 75 offers pairs that allow its advances (chosen left, right, right, right):
  consistent, not a confirmation; a run that reads S makes it a prediction (T24a2b's plan).
- **What follows the level** (§4): the battle's winner gains its levels at once when its player
  is not a human at this computer (0x477766), else after the battle's screen
  (`adventure_battle` 0x4AE324, 0x4AE42C); `h3-heroes`' level readers need only the level's new
  source; VCMI's view of another player's hero has no level.
- **Map**: `placed_heroes_setup`, `placed_hero_setup` new; 0x486590 and 0x487290 renamed and
  rewritten; `structs.toml`: `hero_setup` (0x334), the game's `hero_setup` and `campaign_info`,
  +0x1F45C corrected to H3API's `bigCampaignIndex` (T24a1's guess was a scenario).

### Increment 4, round 1: T24a2b1, the wiki and the census (2026-10-20, `adventure/e08-t24a2b1`)

The user merged T24a2a (#303) and cut T24a2b (decision 53): **T24a2b1** the wiki on the survey's
§1–§4 and the census; **T24a2b2** the batch's plan, the rule ids, the second reading and the
ADV-Q. Baseline `master` 49df46d (engine code d95c95dd, the full verify green in the sandbox).
Docs only: survey `E08-progression.md` §5–§6, §0 and the dispute and quirk tables updated.

- **The wiki** (§5; Experience, Secondary skill, Hero class, Primary skill, Prison, Campaign):
  agrees on the thresholds and the levels past 74 (88, 100, 108, 868, 3 732, 5 920, 6 424, then
  a hang; the reading's 32-bit arithmetic gives the same records, and the next, 72 924, does
  not fit the 16-bit level), the fixed seed per hero (D-14), the second primary row from the
  level-up to 10 (D-15; the five rows checked equal the packs, every class's weights sum to
  112), the Wisdom and school intervals and the Elementalist counted as might (D-16, A-10 now
  high), a class-banned known skill at weight 1; silent on the level limit (D-19, D-21) and a
  known school's weight (D-17); differs on the primary bound (A-9: "At 129 the skill value is
  reset to 0") and on a map-banned known skill (ignored, D-18: three answers).
- **D-20 is a conflict of sources** (§5.2): the wiki says heroes starting above 1 000
  experience, "such as" prisoners, always took the left offer, as VCMI's first; E3-6's only
  fitting seed (75) needs left, right, right, right under the reading's draws. For T24a2b2: the
  second reading looks for a left-only branch on 0x4D8B30's path, the batch reads S at load.
- **The census** (§6; the 160 shipped maps): 103 heroes and prisoners with a custom experience
  (42 heroes in 18 maps, 61 prisoners in 28; all at least 1 000; levels 3 to 15, two prisoners
  at 42; 768 level-ups at load), none from a map-wide entry (`Marshland Menace`'s 15 reach only
  its taverns); no level limit on any map (A-8, D-21 unreachable on them); custom primary skills
  at most 9 (the import's clamp changes nothing); 19 SoD maps ban skills, no placed hero knows
  one; no Elementalist with experience; experience at most 5 000 000 (A-7 only from play).
  Without the refusal of an experience, 5 maps import whole (`Adventures of Jared Haret`,
  `Faeries`, `Knee Deep in the Dead` besides today's two; `Unholy Quest` is refused for a
  monster's artifact since T20a2a): 9 heroes, 41 level-ups at load.
- **Not done here** (T24a2b2): the batch's plan, the rule ids, the second reading, the ADV-Q.

### Increment 4, round 1: T24a2b2, the batch's plan, the rule ids, the second reading; T24a done (2026-10-20, `adventure/e08-t24a2b2`)

The user merged T24a2b1 (#309) and took T24a2b2 whole (decision 54). Baseline `master` 3bea0c6
(engine code d95c95dd, unchanged since the full verify on 49df46d). Docs only: survey
`E08-progression.md` §7–§9 and §0, ADV-Q15, the client lane's inbox.

- **The second reading** (§9): a fresh agent, from questions per function and without the
  survey, confirmed §1–§3 point by point (the steps, the limit's cap, the reseed with the new
  level and S, the primary row from level 10, the windows, the intervals with the Elementalist at
  6 and 4, the weights, the counters, the dialog's conditions, the setup flag, the placed hero's
  order, 0x486590 as the campaign's). It adds D-20's code side: 0x52BBD0 never takes the left
  offer by default; at load its clear flag only makes 0x524690 use the defaults 1 000 and 500
  for the army's value. Not settled: whether 0x524DD0 or 0x4E22D0 draw, the x87 precision.
- **Batch E4's plan** (§7): ten rows, most of them predictions from S read in memory (E4-1 levels
  at load, which settles D-20 even without S; E4-2 D-15; E4-3 D-16; E4-4 several levels after a
  battle; E4-5 D-17; E4-6 the level limit, D-19 and D-21; E4-7 to E4-10 decision 45's four rows).
  The harness's needs (S and the counters, the level-up dialogs' offers, `level_up` as a list,
  artifacts, the battlefield) went into the client lane's inbox; the spec's level limit is a
  `cross-lane` change with T24b's batch task.
- **The proposed rule ids** (§8): HER-EXP-002, -003; HER-LVL-001 to -005; GAM-HER-002; GAM-DEC-002;
  GAM-SET-002; GAM-BAT-008 (006 and 007 are taken).
- **ADV-Q15**: thirteen points for the checkpoint (D-14 to D-21, A-7 to A-10, T24b's scope, the
  batch's rows), each with options and a recommendation. **T24a is done**; T24b waits for the
  answers.

### Increment 4, round 1: T24b1, the `h3-heroes` side (2026-10-20, `adventure/e08-t24b1`)

#312 (T24a2b2) was merged. The user answered ADV-Q15: **every point as recommended** (decision
55); point 3 clarified on asking (the user's "assuming that Elementalists will be correctly
classified as magic heroes" means `strict-fixed` only; `faithful` keeps SoD's 6 and 4). T24b cut
by crate, the user's choice of four (decision 56). Baseline `master` c4fbe58 (engine code
c4faca92, verified on 6f84c54; light gates in the sandbox). CI on `master` still fails unstarted
(billing). Docs only.

- **ADR-0013's amendment draft** (`docs/lanes/adventure/adr-drafts/0013-increment-4.md`): points
  1, 2, 3, 11 amended, point 12 new: progression as `h3-heroes`' rules (experience to levels with
  the bound at level 74; the level limit an input; a level-up drawn through a source its caller
  gives, the original's reseeded generator provided; the offers by the original's procedure; the
  answer a hero change; the computer's evaluation ported as a rule, read first). The details gain
  the two counters; the seed byte stays out (server-only, the game's). Point 2's "from increment
  4 movement and scouting" becomes a later increment's.
- **`E08-design.md` rewritten for increment 4**: increment 3's sections pruned (all built; their
  records here, T18a to T22b); part A (§1–§10) the hero side: the data (hero parameters' table,
  highest level and forced skills; the class's `offer_intervals`; A-10 as the data quirk
  `quirk.content.elementalist_offer_intervals` through a new `[[hero_class_override]]`), the
  thresholds and their test vectors (step + step / 5 equals the original's × 1.2; checked by a
  script against §1.1), the cap min(experience + n, cap) (which also lowers, D-19, and caps at
  load, D-21), the draw source and the generator's vectors (§2.3's seeds 738 928 and 31 946 705,
  re-computed), the primary skill, the offers, `GainLevel`, the evaluation's place and inputs
  with its risk (the spell values), rule ids (HER-LVL-006 new: the original's choice) and quirk
  ids ADV-QUIRK-9 to -13. Part B (§11–§17) headings for T24b2, with increment 3's last versions.
- **Stale references** to the pruned sections repointed to this log: three refs in
  `docs/exe/map.toml`, a ref each in `heroes.toml` and `game.toml`; the battle lane's inbox has
  `gapfill.toml`'s source url (battle-owned) and the coming gap-fill of hero data.

### Increment 4, round 1: T24b2, the game side and the cut; T24b done (2026-10-20, `adventure/e08-t24b2`)

The user merged T24b1 (#318) and took T24b2 whole (decision 57). Baseline `master` ed4a213 (the
battle lane's E23 T1c merged since T24b1; this change is docs only). Docs only.

- **ADR-0017's amendment draft** (`docs/lanes/adventure/adr-drafts/0017-increment-4.md`): points
  1, 4, 5, 6, 8, 9, 11 amended. The store keeps each hero's level seed (setup stream, 1–255,
  server-only) and the game the scenario's limit and banned skills; the custom experience's
  refusal goes, the build gives levels at load (the original's choice with the setup's defaults);
  a level's draws from the seed and the level in `faithful` (`ADV-QUIRK-9`), the owner's stream in
  play and the setup stream at load in `strict-fixed`; a two-offer level is the owner's pending
  decision answered `First` or `Second`, levels one at a time, one or no offer given at once; the
  system actor's default the original's choice with the army; action encoding v4, replay format
  v5, game state hash v9, map import v8; the battle's winner levels after the outcome (the
  original's human path).
- **`E08-design.md` part B** (§11–§17): the seed's draw and where it lives; the setup's limit and
  banned skills, the import (v8, the refusal lifted, the stale citations with it), levels at
  load in `hero_details`, the shipped maps' 41 level-ups in `--with-original-data`; the decision,
  its answers, events and the observation's decision kinds; the env's two entries, replays,
  fixtures, properties, benchmarks; versions; batch E4's tasks; shared and `cross-lane` work.
  Part A's references to part B corrected (§12 for the map's inputs; no event for the cap).
- **The cut** (§17): five rounds, T25 + T26 (the evaluation's reading; the shared groundwork),
  T27 + T28, T29 + T30, T31 + T32a, T32b + T33, with increment 4's six exit criteria. T25 and T26
  are in the QUEUE, waiting for ADV-Q16. The battle lane's inbox has T26's and T32a's
  `cross-lane` parts.
- **ADV-Q16**: seven points (the two drafts, the answers' shape, the system actor's default,
  `strict-fixed` at load, the intervals as data, the cut). **T24b is done.**
- **ADV-Q16, the user's answers (the same evening; decision 58).** Asked one by one with context
  at the user's request: every point as recommended. On ADR-0013's draft the user asked why
  towns and map objects need the hero rules: they share what lies beneath the level-up (gains of
  experience, skills and primary skills from objects and buildings, levels at load of prisons and
  tavern heroes, the computer players' choices), so the draft's context and consequences were
  tightened to say so. Both drafts marked accepted (T26's `shared/` pull request applies them);
  T25 and T26 ready. Applied to #321 before its merge.
  The user then stated the expectation that experience sources and map objects do not
  manipulate a hero through its interface but are linked up by the orchestrator, so they do not
  get entangled: confirmed as the design (ADR-0013 point 1, ADR-0017 point 10: contexts read by
  reference, return typed hero changes, the game applies them), and said in the draft's context.

### Increment 4, round 1: T25, the computer's evaluation; T26a, the ADRs (2026-10-20, `adventure/e08-t25`, `shared/e08-t26a` #325)

The user merged T24b2 (#321) and cut round 1 (decision 59): T25 with T26a, both docs only; T26b
next. Baseline `master` 774ba0e.

- **T26a** (`shared/e08-t26a`, #325): the accepted amendments applied to ADR-0013 (points 1, 2,
  3, 11, new 12) and ADR-0017 (points 1, 4, 5, 6, 8, 9, 11), the index's "Read when"; after
  ADV-Q17 the system default and the choice's inputs (the map's shares). The drafts leave the
  lane folder in T25's pull request.
- **T25** (survey §10): the executable staged from the user's folder. 0x524690 read whole (a
  switch over the 28 skills; the flag counts the army and the computer's state), 0x524DD0 (a
  ranking of the unknown skills against the free slots), 0x52BBD0 re-read; **no draws**. Found
  on the way: Pathfinding's and Navigation's values read two globals that `ai_map_terrain_shares`
  (0x52B9C0) computes at the new game before the heroes' setups, over the coast-flagged tiles
  only (A-11: Navigation always 0, a map without a coast a NaN that `crt_ftol` makes 0); the file's
  1.0 in them had first looked like constants. With that, batch E3c's E3-6 (S = 75, a grass map)
  gets all four of its observed choices from the evaluation (always-left and the constants each
  miss one). The flag-set path reads the hero's AI effectiveness words, a per-player AI value and
  the schools through the caster record (0x524D20): E10's. VCMI has no counterpart (its
  Nullkiller scores by role). Ten map entries (two functions upgraded to read, `ai_school_value`,
  `ai_map_terrain_shares`, two tables, two globals, `ai_player_values`) and two hero fields.
- **ADV-Q17** (asked interactively with context; decision 60): A-11 reproduced in every profile
  (no flag, `ADV-QUIRK-14` with T28); the system default chooses with the setup's inputs until
  E10; the adventure part's terrains gain their cost by Pathfinding level now (T29). Applied to
  #325, the design note (§1, §8, §9, §12, §13, §15, §17) and the queue (T26b).

### Increment 4, round 1: T26b, the shared groundwork's data; round 1 closed (2026-10-20, `shared/e08-t26b`)

The user took T26b whole (decision 61) after T25's #327 merged; one `shared/` pull request,
`cross-lane` with the user's consent. Baseline `master` 2d6838c (its engine code verified before:
the light gates only, green).

- **`h3-content`** (part A §1): `HeroParamsDef`/`HeroParams` gain `experience` (the pack's first
  levels), `max_level`, `wisdom` and `schools` (keys resolved in the pack's closure, the schools
  distinct). `thresholds` continues the table to `max_level + 1` entries, each step + step / 5
  (in `u64`, every threshold checked into `u32`), and refuses a table with fewer than two levels,
  not starting at 0, not rising strictly, longer than `max_level + 1` or above 32 bits (78 is the
  highest `max_level` with the original's levels: level 79's threshold 3 753 284 634). The
  survey's vectors (§1.1: 13 → 24 320 … 75 → 1 810 034 207) are a test. `HeroParams` lost `Copy`.
  `HeroClassDef`/`HeroClass` gain `offer_intervals` (`[wisdom, school]`, each at least 1, checked
  with the class); `HeroClassOverride` with `HeroClassPatch { offer_intervals }`, applied in
  `add_overrides` after the hero overrides (a gated one resolved in every profile, validated with
  the class). The hash: the classes' intervals, the compiled table, `max_level`, Wisdom, the
  schools; **core ruleset hash v46** (the golden base hash moved). The registry gains
  `quirk.adventure.level_up_from_seed` (27, `LEVEL_UP_FROM_SEED`, no reader until T29) and
  `quirk.content.elementalist_offer_intervals` (28, `ELEMENTALIST_OFFER_INTERVALS`). No new error
  variant (`InvalidHeroParams`, `InvalidContent`). Coverage: every new branch has a fixed test.
- **`h3-pack`**: **pack format v44** (`[hero_params]`'s four fields, `[[hero_class]]`'s
  `offer_intervals`, `[[hero_class_override]]`), a syntax test, and a test on the harvested packs
  (`the_harvested_level_data_and_the_elementalists_intervals`: the table, Wisdom and the schools,
  eight classes on 3 and 3 in `faithful`, nine in `strict-fixed`, with and without SoD).
- **The harvester** (battle-owned): the gap-fill's `[heroes.params]` names Wisdom and the four
  schools (keys from `HeroPacks.skills`, a skill's key and name by id), `[heroes.offer_intervals]`
  lists the eight classes 0x4DAD74 puts on 3 and 3, `[[data_quirks.class_intervals]]` writes the
  Elementalist's gated `[[hero_class_override]]` after its class (comment from the entry's
  `quirk`); unknown class or skill names are errors. Five sources (`exe-experience`,
  `wiki-experience`, `exe-offers`, `wiki-secondary-skill`, `vcmi-level-up`). Both new flags in
  the harvested profiles (the strict profile's comment gained the BAT-QUIRK-24 it lacked). The
  packs re-harvested from the staged installation: `base` and `ab` changed only by the new lines;
  `synthetic` by hand (invented levels 1–4, `max_level` 20, `channeling` and `stormcraft`).
- **Elsewhere**: the test packs' new fields (`h3-battle`, `h3-content-battle`, `h3-game`,
  `h3-heroes`); `ADV-QUIRK-10`'s ledger entry (its data quirk complete; `ADV-QUIRK-9`'s with T29,
  design §11); ADR-0007 point 4's scope names the class override (decided with ADR-0013 point 12);
  `packs/README.md`. **Replays**: the 19 golden replays (10 battle, 9 adventure) keep commands and
  state hashes, only `ruleset_hash` moved (three values: synthetic faithful, the base packs in
  both profiles); `duel-s2-d7` and `duel-s5-strict-d7` re-recorded with their options differed
  in that line only. The AI lane's `bots-report-v1.json` regenerated: only `engine.core` moved.
- **Inboxes**: battle (what moved in its paths), AI (the bots' table), the lane's own (the flag
  without a reader until T29), server (the lockfile bump below).
- **Verify**: the full run with the original data passed on 6979105 (every step; benchmarks within
  0.2 %). After merging `master` (eight commits, docs and the AI lane's Python) the second run
  failed only `deny`: RUSTSEC-2026-0382 (`rustls-webpki` 0.103.15, `h3-server`'s TLS), published
  between the two runs. The user chose to bump it here: 0.103.16 in `Cargo.lock`.

### Increment 4, round 2: T27, `h3-heroes`' progression (2026-10-20, `adventure/e08-t27`)

The user took T27 whole (decision 62, after #335 merged) and the level check in `h3-heroes` as
designed; one pull request, `cross-lane` with the user's consent. Baseline `master` b989682.

- **`h3-heroes`** (design part A §2–§7; the sections replaced by a summary): `level.rs`:
  `experience_for` (the table's thresholds; level 0 as 1, past the table its last entry),
  `experience_bound` (level 75's threshold − 1, A-7), `level_of` (the thresholds at or below,
  at most `max_level`), `experience_cap` (a limit's threshold, 0 or above `max_level` none: A-8),
  `gained_under_limit` (the sum at most the cap, experience above it lowered by any gain: D-19;
  no lower than the hero's level's threshold, so the details stay valid; design §2–§7's note),
  `pending_levels`. `level_up.rs`: the `Draws` trait (`between`, inclusive), `impl Draws for
  Rng` (the owner's stream), `OriginalDraws` (the C runtime's generator seeded with
  214 013 L + 156 823 S + 154 079; `state`, `rand`), `draw_level_up(details, ruleset, banned,
  draws) -> Option<LevelUp>` (the primary skill by the class's rows, D-15; the two offers through
  `hero_secondary_offer`'s steps: Wisdom outright, a school with a known one weighing 1 (D-17),
  any candidate by weight with a banned one 0 or 1 if known (D-18); the school's level set
  between the offers, both levels after them), `LevelUp { level, primary, offers, offered }`,
  `Offers { None, One, Two }`, `Choice { First, Second }`, `LevelUp::learned`/`change`.
  `HeroChange::GainLevel { primary, learned, offered }` (checked: a level pending, the skill
  learnable, the offers' levels at most the new level; applied: level + 1, the primary + 1 at
  most `primary_max` (A-9), the skill, the levels), `apply_change_limited` (`apply_change` is it
  with limit 0). `HeroDetails.offered: OfferLevels { wisdom, school }`; the validation refuses
  `ExperienceAboveBound`, `LevelAboveExperience`, `OfferedAboveLevel`; `ChangeError` gains
  `NoLevelPending`, `CannotLearn`, `OfferedAboveLevel`. Tests (`tests/level.rs`): the survey's
  thresholds and seeds (738 928 → 3, 31 946 705 → 91), the Knight's offers (S = 1: Leadership and
  Tactics; S = 200: Leadership and Sorcery), batch E3c's E3-6 (the Death Knight, S = 75, levels 2
  to 5 chosen left, right, right, right: 3/6/1/1, Pathfinding and Scouting advanced, Logistics
  expert), each step of the offers with fixed draws; 100 % coverage. Specialty tests that ran at
  level 255 run at 74 (their expected values recomputed).
- **The other lanes' heroes** (decision 62): every `HeroDetails` literal gained `offered`; heroes
  above level 1 carry their level's experience (`experience_for`): the battle lane's
  `h3-pack-battle` tests (their random heroes at levels 1 to 74, was 255) and the AI lane's
  encounter sampler (`h3-sim/src/encounter/hero.rs`; its battles unchanged: the battle reads no
  experience). `h3-map`'s play property refuses a told `GainLevel`.
- **`h3-game`**: the details' hash gains the offers' levels: **game state hash v9**. The battle's
  experience still goes through `apply_change` (no limit) until T29 passes the map's.
- **Replays**: the nine adventure replays re-recorded with their options: commands unchanged,
  their checkpoints, final hashes and `game_state_hash_version` moved.
- **Rules and quirks**: HER-EXP-002, HER-EXP-003, HER-LVL-001 to -005 written (`implemented`;
  sources the code reading, the wiki, VCMI, E3-6), HER-DET-001 and HER-CHG-001 amended;
  `ADV-QUIRK-11` to `-13` written (exploit-fix, no flag), `ADV-QUIRK-10`'s regression test named.
- **Inboxes**: battle and AI (their tests' and sampler's heroes), the lane's own (T29's limit).

### Increment 4, round 2: T28, the original's choice ported; round 2 closed (2026-10-21, `adventure/e08-t28`)

T28 as queued (design part A §8, survey §10). Baseline `master` 79551a5 (#339 merged); the
lane's own paths only.

- **The re-reading** (the sandbox, `scripts/exe.py`): 0x52BBD0, 0x524690's 28 cases, 0x524DD0
  and 0x52B9C0 read whole; the survey's §10.2 table holds case by case. Settled beyond it: the
  ranking's slot count drops before its test (the offered skill must rank among the top
  8 − count unknown skills; the map entry says so), and every unknown skill takes part in the
  walk (one the class never offers, valued 0, still uses up a slot); the exchange sort's tie
  order varies with the whole array (one hero walks Offense before Armorer, another Armorer
  first: a stable sort would differ). The level's primary skill is raised (0x4DA8F7) before
  0x52BBD0 runs, so the evaluation sees it raised. The flag-set path keeps the flag in `dl`
  across the army loop (E10's).
- **`h3-heroes`** (`choice.rs`): `original_choice(details, ruleset, level_up, worth, shares) ->
  Choice` (HER-LVL-006): one offer or none `First`; both known or both new the higher value, the
  first on a tie; else `takes_new` for the new one. `ArmyWorth { army, shooters }` (`SETUP`:
  1 000 and 500, ADR-0013 point 12.6's argument), `TerrainShares { tiles, water, rough: [u32;
  3] }` (the map's counts, integers so the game can keep and hash them, T29). The values by the
  original's id = the ruleset's order (as the offers' draws assume; a skill past the 28th is
  worth 0, the switch's default), Wisdom known by position too; exact in `u64` (the original's
  32-bit products overflow only past an army worth 306 million, E10's). The shares through
  `h3_kernel::arith::X87` (`ratio` at 53 bits for the stored f64, `times`, `over`, `trunc`), 0
  without tiles; a scratch scan of every count ≤ tiles ≤ 41 472 (144 × 144 × 2) for both
  divisors, 1.72 billion cases, found it equal to IEEE doubles and to the exact integer floor.
  `derive::effective_primary_from` (the base given) for the raised primary skill.
- **Tests** (`tests/choice.rs`; vectors from a Python model of the listing written apart from
  the code): every skill's value unknown, with Wisdom, by level and at Expert, a full hero, a
  29th skill; the raised primary skill with a worn artifact and the clamps (Wisdom 75 → 100 flips
  a choice); the shares (none without tiles, near-integer counts, a scaled army); alike offers
  and ties; the survey's worked Knight (Tactics refused after seven slots), the tie-split
  boundary cases. E3-6 in `tests/level.rs` now asserts the evaluation's four choices on the
  batch's coastless map; with the file's 1.0 shares levels 2 and 4 would take Navigation (the
  survey named level 2 only; corrected). 100 % coverage.
- **Rules and quirks**: HER-LVL-006 written (`implemented`; sources the reading, E3-6, the wiki
  and VCMI against it); `ADV-QUIRK-14` written (preserve, no flag; its count's test with T29).
  No version moves (nothing calls the rule yet): T29 counts the shares and chooses at load, T30
  for the system actor.

### Increment 4, round 3: T29, the game's setup and build (2026-10-21, `adventure/e08-t29`)

Decision 63: T29 whole, one pull request, `cross-lane` with the user's consent (the AI lane's
`crates/h3-cli/src/ai/census.rs` sets the two new `GameSetup` fields to none). Baseline `master`
528b602 (#343 merged): the full verify with the original data green in the sandbox.

- **The adventure part** (`h3-adventure`, `h3-pack-adventure`, `packs/base/adventure.toml`):
  `TerrainMovementDef::pathfinding` (Basic, Advanced, Expert; land only, none above the one
  below: `AdventureCompileError::PathfindingCosts`), `AdventureRuleset::step_cost(terrain,
  level)` (water its cost at every level, none for an impassable terrain); the base pack's sand,
  snow, swamp and rough by the original's `step_cost_table` (the wiki and VCMI agree, survey
  `E04-skills.md` row 0); adventure ruleset part v9. The static map counts the terrain shares once
  (`StaticMap::terrain_shares`, ADV-MAP-002: the coast-flagged tiles not impassable, the water
  ones, per Pathfinding level 0-2 those above a native step); not hashed (they follow from the
  tiles and the part), so the adventure state hash stays v10 and a fork shares them with the map.
- **The game** (`h3-game`, `state/levels.rs`): `GameSetup.level_limit` and `.banned_skills`
  (`GameSetupError::BannedSkill`, checked after the players); `Progression` (the limit, the bans
  in id order once each, a seed per hero) behind one `Arc` in `GameState`, hashed (game state hash
  v10), never observed; `GameState::level_limit()`, `banned_skills()`. GAM-HER-002: the seeds
  (1-255) are the setup stream's **last** draws, after the map objects' contents (design §11 had
  each after its hero's defaults: drawn last, no earlier draw moved, so the nine golden replays
  were re-recorded with their commands unchanged); `Progression::draw` takes the original's
  generator from the seed and the new level with `ADV-QUIRK-9`'s flag on, the stream given off.
  GAM-SET-002: a custom experience is built at experience 0 (so the details' check passes for
  any amount), then gained under the limit with each pending level drawn and given (two offers by
  `original_choice` with `ArmyWorth::SETUP` and the shares; at most as many draws as levels
  pending), the spell points full after; in hero id order after the seeds, in `strict-fixed`
  from the setup stream. The battle's experience goes through `apply_change_limited` with the
  limit (GAM-BAT-002's summary).
- **The import** (v8): the header's level limit, SoD's banned skills by their codes (a bit past
  the 28 codes refused as an unknown code; none in the shipped maps), a custom experience passed
  as it is (`Unsupported::HeroExperience` gone). The original data (`original_data.rs`'s new
  census lines): 5 maps import whole, their 9 heroes levelled at load with 41 levels (survey §6.5
  exactly); no map has a level limit; 19 ban skills, none past the 28 codes.
- **Stale citations** (the inbox's 0x486590 entry, closed): re-pointed to `placed_hero_setup`
  0x4D8B30 or `placed_heroes_setup` 0x4CAE10 in ADV-MOV-002, ADV-FOG-003, GAM-SET-001,
  GAM-HER-001, HER-MAN-001, `ADV-QUIRK-8`, the two comments in `h3-adventure`'s `state.rs` and
  `packs/base/adventure.toml`; the import's clamp of custom primary skills stays, its comment
  now naming the carried-over hero's clamp.
- **Tests**: `h3-game`'s `tests/levels.rs` (the seeded levels equal an oracle of the hero model's
  functions for 40 games; strict-fixed's do not follow the seed byte; the limit, the bound and
  level 74 from `u32::MAX`; bans in any order; the hash and the forks), the limit in the battle's
  aftermath, the error; `h3-adventure`'s step costs, the compile check, the hash, the shares'
  regression test for `ADV-QUIRK-14`; the pack's parse and the base pack's costs; the import's
  limit and bans. The test pack's knight class gained skill weights and a `t:seeded` profile.
- **Rules and quirks**: GAM-HER-002, GAM-SET-002 (`implemented`), ADV-MAP-002 (`implemented`);
  `ADV-QUIRK-9` written (fix-optional, flag 27); `ADV-QUIRK-14`'s count test named.

### Increment 4, round 3: T30, the level-up as a pending decision (2026-10-21, `adventure/e08-t30`)

Decision 64: T30 whole, one pull request, with the env's answers and the CLI's words from T31;
decision 65 at its checkpoint: an open level-up holds the game (below).
Baseline `master` 2518a92 (#349 merged): its code verified by T29's run, the sandbox's light
gates green. Before the cut, the nine golden replays were played with a probe: none reaches a
level (the highest experience any of their heroes ends with is 788), so T30 needs no recorded
answers.

- **The game** (`h3-game`): `Decision::LevelUp { owner, view: LevelUpView, offered }` (game state
  hash v11: kind 2, the owner, hero, level, primary skill, two offers, the offers' levels);
  `state/levels.rs`' `give_levels` (GAM-DEC-002: each level drawn by `Progression::draw` with the
  owner's stream, one offer or none given at once as `LevelGained`, two a decision told as
  `LevelUpOpened`), `answer_level_up` (the level
  given, the next level-up in the answered one's place, else the decision closed),
  `update_hero` (shared with the battle's `change_hero`); `end_battle` gives the winner's levels
  last (GAM-BAT-008); `answer` split into the level-up's and the offer's (`answer_offer`), with
  `WrongAnswer` after `BattleOpen`; `GameState::level_up`; `observe`'s
  `decision: Option<DecisionView>`; `legal_commands` lists `First` and
  `Second`; `project` gives the two events to the owner only; `default_answer` (`Decline` for an
  offer, the original's choice with the setup's inputs for a level-up, `ADV-STUB-8`).
- **The level-up holds the game** (decision 65; ADR-0017 point 6 amended in #353): the first
  build let the other players act on, as the ADR then said, so a defender that won could be
  attacked again before answering (it needed a guard against drawing a level twice and a
  battle waiting behind the level-up, both dropped). Now `apply` checks `check_held` first:
  every other actor's command is `WaitingForLevelUp(owner)`, `legal_commands` and `legal_moves`
  give it nothing, and `GameState::waiting_for` names the owner for the hosting layer.
- **The env and the CLI**: action encoding v4 (`FIRST` 11, `SECOND` 12, `Action::first`,
  `second`); while the game waits `to_move` yields the level-up's owner alone, in its turn or
  not; the random agent answers at even odds through `legal_commands`; replay format v5
  (`<player> answer first|second`).
- **The properties** (`h3-map`): random play met level-ups at once (the generated maps' heroes
  gain levels in their battles: the first runs after the game's change failed on them), so the driver
  answers them (`seek`, `acting`: the owner while the game waits), `check_legal` lists a
  level-up's answers for its owner and nothing for the others, `check_isolation` keeps other
  players' level-ups, `check_conservation` and the battle's
  heroes check follow the levels told (`check_levels`), and the view's event model knows both
  events. The two failing cases were kept in the regression files. T31 adds the plans' checks
  of §14.
- **Tests**: `h3-game`'s `tests/level_ups.rs` (the open level-up and each answer against a draw
  of the hero model with the owner's stream, the observation, the legal commands, wrong answers;
  two levels one at a time, each drawn after the previous answer; one offer and none by banned
  skills; faithful's seed; a defender that wins holds the game, the player whose turn runs and
  the system actor refused until it answers outside its turn; the limit; a fork; the system
  actor's default taking either offer over 40 seeds);
  the offer's wrong answers and default, a battle's none; the env's two entries, their masks,
  decode and encode, the random agent; the CLI's words.
- **Rules**: GAM-DEC-002, GAM-BAT-008 (`implemented`); GAM-DEC-001, GAM-SET-002, ADV-MAP-002 and
  `ADV-QUIRK-9`'s context point to GAM-DEC-002. `docs/07-simulation-and-ai.md`'s env line (a shared doc) gained encoding v4 and replay format v5 in #353.
- **Versions**: game state hash v11, action encoding v4, replay format v5; the nine golden
  replays re-recorded with their options, their commands unchanged.
- **Housekeeping**: T29's merge of `master` had left conflict markers in the adventure and AI
  lanes' `QUEUE.md` inboxes (both sides kept; the adventure's T27-T29 notes for T30 consumed).

### Increment 4, round 4: T31, the replays, fixtures, properties and benchmarks (2026-10-21, `adventure/e08-t31`)

Decision 66: T31 whole, one pull request (the user's choice of four; T32a in its own session,
its recipe waiting for the harness's new reads). Baseline `master` 1284596 (#353 and #352
merged): its code verified by T30's run, the sandbox's light gates green.

- **The properties** (`h3-map`'s `play.rs`, `view.rs`): `check_progression` after every command
  of `every_step_keeps_the_commands_invariants` and of the plans' 192 fixed cases (a level at
  most the experience's by an oracle of the thresholds, the experience under the bound and the
  limit's cap, a pending level only for the open level-up's hero, at the level after its own, at
  most one level-up open and the game waiting for its owner alone); the first test's cases get
  a custom experience in half the cases (`experience()`: mostly just below a threshold of levels
  2 to 20, sometimes any amount), drawn last so the regression seeds keep their plans. Its first
  run found the battle heroes' check adding a gain past the bound (the game caps it, HER-EXP-002;
  the event tells the gain asked): the test model caps it now (`experience_cap`), the case kept
  in the regression file. The fixed cases count level-ups opened and levels gained.
- **The draws' source by profile** through the original packs and a map: in `faithful` a
  hero's levels at load stay the same whatever another hero gains at load before it, in
  `strict-fixed` they change with it for some seeds (RoE's experience 0 is the default, drawn
  from the setup stream: the first try used it and moved every seed, so the test uses 1). In
  play the source per profile stays `h3-game`'s fixed tests (T30): `h3-map` has no access to
  the level seeds, and `h3-heroes` is not its dependency (`arch.toml`).
- **Hygiene**: `Unseen.experience` gives the other players' heroes that have a custom experience
  (RoE's and AB's plans) another one, so their levels at load differ unseen (55 of 207 cases in
  a probe run); `outcome` compares whom the game waits for, which every player learns. A probe
  found random play never reaching the end of another player's battle out of sight while the
  comparison runs, so `another_players_level_up_shows_nothing_of_its_offers` plays one: red's
  hero beats savage wolf riders out of blue's sight, Scholar known in one game only, and blue
  sees and is told the same at every step while red's offers differ (most seeds in `faithful`,
  2 of 16 in `strict-fixed`). The full verify's coverage run then found a case of the test's own
  (`vary`): an unseen blocker added over the tile of a monster the viewer sees on its fringe
  took the tile's visits from it (ADV-OBJ-003), so the monster stopped guarding and the viewer's
  paths changed. `vary` keeps added and removed objects off the monsters the viewer sees (the
  case kept in the regression file); 10 000 cases each of the hygiene and invariant properties
  passed after it.
- **Fixtures**: the duel's heroes 970, 950, 4 570 (red's underground hero: levels 2 to 4 at load)
  and 930; the XL map's the duel's, then 980, 940, 960, 920 (`fixtures.rs`, `DUEL_EXPERIENCE`,
  `XL_EXPERIENCE`); both regenerated. `h3-sim`'s battle replay test answers the winner's
  level-up before its end of turn.
- **Replays**: every command changed (the setup stream no longer draws the heroes' default
  experience, and the levels change the battles). The four with fixed options re-recorded; the
  five searched ones replaced from a search of 5 000 games per profile with a scratch probe (not
  committed): `duel-s767-d28` (a hero battle won with a capture), `duel-s2626-d28` (a passage),
  `duel-s236-d28` (a pile's guards beating the visitor, a question declined), `duel-s12-d28` (a
  monster's battle lost, a Cartographer declined), `duel-s929-strict-d28` (a hero battle lost in
  strict-fixed, the winner capturing). Seven answer level-ups, five with both offers; in all nine
  red's underground hero levels at load. Of the 5 000 faithful games 32 have a hero battle (1 500
  of E04c T10b's had 11), 4 725 a level in play. `duel-s5-strict-d7` now meets no battle.
- **Benchmarks**: `random-play-duel-36` 523.5 M → 567.1 M Ir (2 956 → 2 858 steps),
  `reach-xl-144` 525.7 M → 477.5 M (4 608 → 4 728 queries), both re-baselined; `fork-xl-144`
  +0.22 %, kept (the progression behind its `Arc`, shared by the forks).
- No version bump: tests, data and replays only.

### Increment 4, round 4: T32a, batch E4's specs and scorer; round 4 closed (2026-10-21, `adventure/e08-t32a`)

Baseline: `master` at a618c3e (T31 merged, #357), its code verified by T31's full run (the
sandbox's baseline ran only the light gates). The user took T32a whole (decision 67; not chosen:
the E4a spec alone, the spec with the scorer, the scorer first) and agreed to `cross-lane` for
`h3-experiment` and, asked again when the writer's new field broke it, for one line of the AI
lane's test map. `shared/e08-t32a` (#358) first: `h3-heroes` among `h3-map`'s dev-dependencies
in `arch.toml`, for the scorer (its full verify passed every step but coverage, which failed on
`h3-server`'s ladder test timing out under the sandbox's load and passed when run again alone;
the server lane's inbox has it).

- **The writer** (`h3_map::writer`): `Map.level_limit` (the header's byte after the difficulty)
  and `MapObject::Plain` (a visitable object without a body; `PLAIN_OBJECTS`, the Cartographers,
  class 13). The read-back test reads both back and imports them: the land Cartographer typed
  `base:cartographer_land`, the setup's limit 5.
- **The spec** (`h3-experiment`, `cross-lane`): `[map] level_limit` and `[[map.object]]`
  (`picture`, `at`; `Tables.plains`); a plain object counts with every tile it covers (its block
  and visit masks) in the check that no two objects share a tile. Two grounds on one tile it had.
  `docs/experiments/README.md` documents both and lists batch E4.
- **Batch E4 split in three** (the level limit binds every hero on its map, so E4-6 could not
  share E4a's map: E4-2's level 10 and E4-1e's 5 000 would be capped; the agent's call, decision
  67 updated): `E08-batch-4a` (E4-1 to E4-5: four Knights and E3-6's Death Knight levelled at
  load, three Barbarians at level 10, three Elementalists at level 3, two Rangers through three
  dialogs after one battle, three Demoniacs knowing two schools at level 4; Red and Blue eight
  heroes each), `-4b` (E4-6: limit 5, 10 000 experience at load and a gain; 4 000 gaining 700),
  `-4c` (E4-7: two pairs of overlapping grounds in both orders; E4-8: a Necromancer's full army
  with Skeleton Warriors beating 90 Pikemen; E4-9: Blue's Thunderbirds destroying Red's Ballista
  by plan, Red winning; E4-10: a water row leaving the land Cartographer's entrance the only way
  between Red above and Blue below). Classes chosen for conclusive rows: the Barbarian's rows
  differ for 60 of 100 draws at level 10 (the Knight's 35); the Demoniac's known Fire and Water
  Magic weigh 4 and 1 under VCMI against 1 and 1 (half the draws). Each spec compiled and imported
  whole through the engine (`original_data` on the three maps: 16 heroes and 56 level-ups at load,
  E4-6a's four under the limit; 24 tiles under two grounds); E4-10 played in the engine by a
  throwaway example (not committed): Blue declines the Cartographer and stays on its entrance,
  Red's step at it from above starts a battle.
- **The scorer** (`crates/h3-map/examples/e4_score.rs`, `test = true`): a batch map and its filled
  template; per hero from its seed byte the levels at load (the reading: the evaluation under the
  limit; always-left; VCMI without the limit, experience and level only) and the dialogs in play
  (the reading; per row VCMI's row by the old level, intervals 3 and 3, a known school by its
  class weight, or every level drawn up front), the record beside them and which fit. Its draws
  are a copy of `h3_heroes::draw_level_up` with each rule a switch; its tests check the reading
  against the engine (400 random heroes of every class at 7 seed bytes each; the levels at load
  against `GameState::new` under limits 0 and 5) and a record of the reading fitting it. It
  reproduces batch E3c's E3-6 (3/6/1/1, Pathfinding and Scouting advanced, Logistics expert) with
  seed byte 75 alone of the 255, as survey §3.3 found.
- **Not built**: the recipes (`clients/capture/batches/E08-batch-4{a,b,c}.toml`): the harness
  lacks the reads (the client lane's inbox lists them: the seed byte and counters, the skills as a
  list, each dialog, `level_up` as a list, the artifacts, the battlefield, E4-9's battle by plan,
  E4-6's message). They and the run are T32b (round 5, with T33).

### Increment 4, round 5: T32b1, the harness's reads for batch E4 (2026-10-21, `adventure/e08-t32b1`)

Baseline: `master` at 3cc8ba0 (T32a merged, #360); the sandbox's full baseline died with the VM
(a restart), and the branch changes no engine code, so its verify is the light gates (exit 0)
and the client workspace's gate. Decision 68: the user cut T32b, this lane building the reads the
client lane had in its inbox (busy with E17 M3), `cross-lane` into `clients/capture/` with the
user's consent; T32b2 (the recipes and the run) next.

- **The reading** (`level_up_dialog` 0x4F8880, its items; map entry `read` for them): the dialog
  shows the primary skill raised as DEF picture 2009 (`pskil42.def`, frame 0-3) and the offers as
  pictures 2010 (left, or the only one) and 2011 (right), `secskill.def` frame 3 × skill + 3 +
  the hero's level of it before (`hero_level_up` 0x4DAABD); the DEF picture's constructor
  0x4EA720 (new entry, H3API's `H3DlgDef`) keeps the frame at +0x34. The hero's backpack (+0x1D4,
  64 entries, the count +0x3D4; 0x4E2F90) and the artifact setup record's name (+0x0, stride
  0x20) went into `structs.toml`.
- **`h3mem`**: the hero's seed byte, offer counters, skills in the hero's order (+0xE5), worn
  artifacts and backpack; artifact names; the dialog's items and the level-up dialog; the
  battlefield (+0x5394, +0x53C0); the hex records (stack, flags, the attack mark +0x4A); a stack's
  speed (+0xC4).
- **`h3adv`**: every level-up dialog recorded (hero, level, primary skill, offers, the one taken),
  one or none offered included, once per hero and level (the dialog is the same local for each
  level); `level_up` as a list; the other dialogs' records (`messages`); the observations
  `level_up:<n>:<part>`, `messages`, `battlefield`, `battlefield:terrain`, `destroyed:<creature>`
  and the hero details `skills`, `seed`, `wisdom_offer`, `school_offer`, `artifacts`, `backpack`;
  battles played by `attack` (every stack attacks the nearest enemy) or a plan (`{ attack =
  "Ballista", by = "defender", then = "attack" }`, E4-9). **Auto combat does not play a battle
  between two human players in hot seat** (its key and button did nothing on a human stack's turn;
  E4-9 and E4-10 are such battles): the plans click like a player, the mouse resting over the
  target until the combat cursor shows a melee attack (frames 7 to 14), and a target out of reach
  this turn (the attack mark) makes the stack fly to the free hex nearest it within its speed (the
  Thunderbirds, 13 hexes from the Ballista with speed 11, take two turns). Fixed on the way: a
  local named `value` in `value()` (E17 M3 T6c-2's winner fix) made every `before:` read fail;
  and the battle screen is known in its `battle_book` state too (the client lane's note of
  T7a-2a: a human hero with a spell book lights the probes' button; batch E4's magic heroes).
- **The harness check** (`capture/checks/harness-check.toml` and its recipe) gained Exp's skills
  in order, seed byte and counters; Lvl's start, dialog, skills after, battlefield and messages;
  and **Gear**, E4-9's shape (Orrin with a Ballista, a helm, a spell book and a backpack axe
  against Valeska's 30 Thunderbirds on a Clover Field). It passed in full under Wine (two full
  runs and four of Gear alone). **Checked against the engine** with `e4_score` on the filled
  templates: every recorded dialog (Lvl's one and Gear's two, seed bytes 242 and 117, then 109
  and 133 in an earlier game) fits the reading's draws, and Exp's levels at load are the ported
  evaluation's in both games (seed 239: 4/3/2/2, Diplomacy basic, Logistics expert, Ballistics
  expert, the counters 2 and 4; seed 216: 3/4/2/2, Diplomacy basic, Logistics advanced, Offense
  expert, Ballistics basic, 0 and 4), not always-left's. Gear's battle read the Clover
  Field and grass; its destroyed Ballista left war machine 1 empty (HER-WAR-002's way: E4-9 in
  T32b2 decides it); 2 300 experience, two dialogs answered left, then right.
- **Not built**: batch E4's recipes and run (T32b2). E4-6's "can gain no more" shows under
  `messages` as its record (GENRLTXT, not yet seen).

### Increment 4, round 5: T32b2, batch E4's recipes and run (2026-10-21, `adventure/e08-t32b2`)

Baseline: `master` at 3843b3f (T32b1 merged, #362); its code state had no run on record, so the
sandbox ran the full verify on it (every step passed, without `--with-original-data`); the branch
changes no engine code. Decision 69: the user took T32b2 whole (not chosen: E4a first, then E4b
and E4c; one batch per task). The original's files were staged from the user's PC at the start.

- **The recipes** (`clients/capture/batches/E08-batch-4{a,b,c}.toml`, `cross-lane` as T32b1): the
  specs' procedures as steps; E4a's heroes on `h3-experiment`'s grid (size 36: (3 + 6 (i mod 6),
  3 + 5 (i div 6)) by the encounter's index, the opponent a column east), E4c's at the spec's
  `at`. E4-1's five heroes are Blue's and never move: read by name (`hero[E4-1a]:seed`, …) while
  Red's E4-2a is selected. E4-4 answers right, left, right; E4-9 by plan, E4-10 by `attack`.
- **The harness** (`clients/capture/`, the same `cross-lane`): a step's `note` (read as `record`,
  added under "anything odd" unless `none`) and the hero detail `equipment` (the worn artifacts but
  the war machines and the spell book: every hero wears a Catapult, which E4a's first run noted for
  all sixteen), so a starting bonus's artifact shows; `h3lod.identify` takes the most specific
  record (the most literal text) of any run of the dialog's text items (it took the first match
  of the whole text: `%s the %s`, GENRLTXT 16, formats every text with " the ", and the
  Necromancy dialog's picture caption is a text item of its own, so GENRLTXT 146 never matched);
  `strike` tries its hover pass four times, a second apart (E4c's third run stopped when a
  Thunderbird's turn showed only the plain pointer, frame 6, over every position: a good morale's
  extra turn not yet taking input). Runbook updated.
- **Batch E4a** (four runs, 64 hero records; each run's template and `e4_score` report under
  `experiments/results/`, `E08-batch-4a.md` and `-run2` to `-run4`): **63 fit the reading**; the
  64th (run 2's E4-1a) wore Blue's starting bonus, a Centaur's Axe (the note), and recorded 4/4/1/2
  against the reading's 2/4/1/2, its skills the reading's. Conclusive heroes (the alternative
  predicted otherwise), all for the reading: **E4-1** (D-20) 16 of 20 refute always-left (none
  fits it alone; the other rows' levels at load 19 of 36 more); **E4-2** (D-15) 7 of 12 (run 4's three the same under both rows); **E4-3**
  (D-16, A-10) 12 of 12 (no Elementalist was offered Wisdom at level 3); **E4-4** (§2.1) 8 of 8;
  **E4-5** (D-17) **1 of 12** (run 1's E4-5c, Water Magic where VCMI's weights give Fire; the
  others had a school offered at load, or both weights picked the same school from the draw).
  Run 1's template had its Catapult notes removed by hand (its header says so).
- **Batch E4b** (one run): E4-6a (seed 71) **4 600, level 5 at load** (the reading; VCMI's 10 000,
  level 8 refuted), the Pikeman's gain **no dialog, 4 600 kept**, the messages GENRLTXT:2 ("can
  gain no more experience") and GENRLTXT:7 (the warning of a player without towns, after the
  battle); E4-6b (seed 107) 4 000 to **4 600, level 5, one dialog** (the reading's offers), then
  GENRLTXT:2 too; it wore Blue's starting bonus, a Pendant of Dispassion (no primary skills).
  Both fit the reading (`E08-batch-4b-score.md`).
- **Batch E4c** (four runs; the fourth committed, by the final harness; runs 1 and 2 the same but
  for the message's record, run 3 stopped in E4-9's battle): **E4-7a Evil Fog, E4-7b Clover
  Field** (the last ground written is the topmost: the reading); **E4-8**: 90 Pikemen split 30
  Pikemen, 30 Halberdiers, 30 Pikemen, the Skeleton Warriors 10 → **16** (+6 = (2c + 2) / 3 for
  c = 9: the reading; VCMI's +9 refuted), 900 experience, the dialog GENRLTXT:146; **E4-9**: the
  Ballista destroyed, Red won, **war machine 1 empty** after (HER-WAR-002), 2 300, level 3 (run
  1: a Pendant of Death on the neck after, Blue's starting bonus taken from E4-9b; Blue's gold and
  resources were the plain ones); **E4-10**: Blue's hero stepped onto the Cartographer's entrance
  (18, 18) from below, its question ADVEVENT:26 declined; **Red's click from (18, 17) found no
  path (no marks) and started no battle, in all three runs that reached it**: the spec's "other
  outcome", **against the engine's prediction** (a battle; ADV-OBJ-004, GAM-BAT-001). For T33.

### Increment 4, round 5: T33, batch E4 scored; increment 4 closed; increment 5 refined (2026-10-21, `adventure/e08-t33`, `cross-lane`)

Baseline: `master` at 9eb496b (T32b2 merged, #367); its engine code (79015dc2) was verified on
32061cc, so the sandbox ran the light gates (passed). The original's sixteen files were staged
from the user's PC at the start. Decision 70 (ADV-Q18): the user took T33 whole (not chosen:
T33a the scoring then T33b the closure and refinement, or the scoring alone), **E4-10 by the
original's rule in every profile, no flag**, and **D-17 settled by its one conclusive hero**.
`cross-lane` for the status row in `docs/experiments/README.md` (battle-owned), as T22b.

**Batch E4 scored** (each hero's predictions by `e4_score` from its seed byte, and E4c's from its
spec, against `experiments/results/E08-batch-4a.md`, `-run2` to `-run4`, `-4b.md` and `-4c.md`,
played by the harness in T32b2; the stamps matched; "conclusive": the other source predicted
otherwise):

| Row | Observed | Reading (engine) | Other source | Verdict |
|---|---|---|---|---|
| E4-1 (D-20, D-14), 20 heroes levelled at load (per run four Knights at 3 200 with one Basic skill, E3-6's Death Knight at 5 000) | each skill set the evaluation's; 16 conclusive (17 with run 2's E4-1a, its attack +2 from a Centaur's Axe), and 19 of the other rows' 36 heroes levelled at load | the computer's evaluation (0x52BBD0) | always-left (the wiki): none fits it alone | the reading: D-20 |
| E4-2 (D-15), level 10 in play, 12 Barbarians at 14 699 | the primary skill from the second row; 7 conclusive | the new level 10's row | VCMI: the old level 9's | the reading |
| E4-3 (D-16, A-10), level 3 in play, 12 Elementalists without Wisdom | no Wisdom offered outright; 12 conclusive | 6 and 4 (SoD) | VCMI and HotA: 3 and 3 | the reading (`ADV-QUIRK-10` observed) |
| E4-4 (§2.1), three levels after one battle, answered right, left, right, 8 heroes | each level's offers followed the previous answer; 8 conclusive | drawn after each answer | drawn up front | the reading |
| E4-5 (D-17), level 4 in play with a school due, 12 Demoniacs knowing Fire and Water Magic | 1 conclusive (run 1's E4-5c, seed byte 6: Water Magic), 11 the same under both | a known school weighs 1 | VCMI: its class weight (Fire Magic) | the reading; the user: one settles it |
| E4-6a (D-21, D-19), limit 5, a custom 10 000 | 4 600 and level 5 at load; a battle's gain at the cap: no dialog, 4 600 kept, GENRLTXT:2 (no more experience) | the same | VCMI: 10 000, level 8 | the reading |
| E4-6b (D-19), 4 000 + 700 under the limit 5 | 4 600, level 5, one dialog (the reading's offers) | the same | — | the reading |
| E4-7a, E4-7b, two grounds on one tile | Evil Fog, then Clover Field: the one written last | the topmost (the last in map order) | VCMI: the first | the reading (ADV-GRD-001) |
| E4-8, a full army with 10 Skeleton Warriors beats 90 Pikemen | split 30 Pikemen, 30 Halberdiers, 30 Pikemen; 10 → 16 Skeleton Warriors; 900 experience | 9 Skeletons raised, (2 × 9 + 2) / 3 = 6 join as the upgrade | VCMI: 9 | the reading (GAM-BAT-006) |
| E4-9, a destroyed Ballista, a victory | war machine 1 empty, the Catapult kept, the backpack empty; 2 300 experience (1 800 + 500), level 3 | the same (HER-WAR-002, VCMI's rule) | — | the engine (HER-WAR-002 observed) |
| E4-10, Red above Blue's hero on a Cartographer's entrance | no path, no battle, in three runs | a battle (T19a1: the visit is the hero's) | VCMI: a battle | **the original, against the engine: built in T33** |

- **E4-10's rule** (decision 70): the code reading had it since T14a: the pathfinder's test from
  above (0x4B26AC entering, 0x4B267D leaving) reads the tile's class through 0x4FCBD0, which for
  a hero (34) or boat (8) standing there returns the object under it. T19a1 checked only the
  visitor's own tile. Now `check_step` refuses a step down onto another player's hero on a tile
  closed above (`FromAbove`), and the reachable-tiles query skips it (directions 3 to 5, only at
  an enemy hero's tile, so the hot loop gains no test elsewhere); a hero there is attacked from
  the tiles the object is visited from. VCMI 1.7.5's `canMoveBetween` tests the tile's blocking
  objects (the hero, visited from every side) and attacks. The test
  `a_hero_visit_ends_the_move_and_an_own_hero_blocks` checks the three steps down, the query's
  path round and the battle from the side. ADV-OBJ-004 and GAM-BAT-004 say so.
- **The catalog:** `verified`, an `observation` source each: HER-EXP-002, -003, HER-LVL-001 to
  -006, HER-NEC-001, -002, HER-WAR-002, GAM-HER-002, GAM-SET-002, GAM-DEC-002, GAM-BAT-002,
  -004, -006, -008, ADV-GRD-001, ADV-OBJ-004; `ADV-QUIRK-9` and `-10` gained the observation.
  Nothing reopened. D-18 (a banned known skill) had no row (survey §7) and stays the reading's.
  The survey's dispute rows and E4-10's batch row say so; `docs/experiments/README.md`'s row is
  `evaluated`.
- **The scorer** `crates/h3-map/examples/e4_score.rs` is **kept** (design §16): it reproduces
  the reports beside the results, its tests keep checking the reading against the engine, and a
  later batch's scorer can start from it.
- **The inbox:** three items batch E4 settled removed (the topmost ground, T20a2a; HER-WAR-002's
  source, T19a2a; the attack on a hero on an object, T19a1); the items of the sketch's former
  increment 5 re-tagged (below).

**The audit of increment 4's exit criteria** (design §17), on this branch's code:

1. Levels from experience, at most 74, capped by the scenario's limit (HER-EXP-002, -003: T27);
   the import keeps the limit and the banned skills (GAM-SET-002, T29). Observed: E4-1 to E4-6.
2. A level's draws from the seed byte and the level in `faithful`, from the owner's stream in
   `strict-fixed` (HER-LVL-002, GAM-HER-002, `ADV-QUIRK-9`: T27, T29, T30); the Elementalist's
   intervals by profile (`ADV-QUIRK-10`: T26b, T27). Observed in SoD: 65 of 66 heroes whole.
3. Two offers a pending decision of the owner, levels in order, each drawn after the previous
   answer, the battle's winner levelling (GAM-DEC-002, GAM-BAT-008: T30); the env (encoding v4),
   the replays (format v5, seven answer level-ups) and the plans (`check_progression`) play them
   (T30, T31). Observed: E4-4 8 of 8.
4. Levels at load by the ported evaluation (HER-LVL-006, GAM-SET-002: T28, T29); the full verify
   with the original data (below) imports the 5 shipped maps whole, their 9 heroes 41 levels at
   load (`crates/h3-map/examples/original_data.rs`).
5. Every new rule has its id, source and tests (`check-rules`); `ADV-QUIRK-9` to `-14` are in
   the ledger; the gates pass (the full verify below: coverage, determinism with the golden
   replays, benchmarks).
6. Batch E4 run (T32b2) and scored (above): D-14 to D-17 and D-19 to D-21 `verified`, none
   reopened; D-18 had no row.

**The full local verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master -- --with-original-data` on engine code ad0034e3 (commit 332a806, 2026-10-21, the sandbox, Linux x86_64): all 16 steps passed, the golden replays unchanged by E4-10's rule; `original-data`: every shipped map's static layer imports and validates, 5 import whole (*Adventures of Jared Haret*, *Faeries*, *Knee Deep in the Dead*, *Race for Ardintinny*, `Tutorial.tut`), 9 heroes levelled at load with 41 level-ups.

**Increment 4 closed** (2026-10-21).

**Increment 5 refined** (ADV-Q19; decisions 71, 72): the user chose **the hero's movement and
scouting** of the sketch's item 5 cut in three (Logistics, Pathfinding and Scouting on the map,
the movement and scouting artifacts, the specialties; derived once in `h3-heroes`), not rewards
on the map, army management or the whole sketch; Navigation and the sea wait for boats. The
first round is a survey and the ADR amendment drafts with the design note and the cut (T34a,
T34b), not the survey alone. The rest of item 5 is the sketch's increment 6, the week's 7. The
inbox re-tagged: the Scouting specialty (T8a), the step price's omissions (T7b), Cursed Ground's
step cost (T20a2a) and flying's visit test (T11a) for T34a's survey; the artifacts' increment
(T19a2a, T20a2a), army management (T17b, T22b) and ADR-0016's line (T23) for increment 6; the
`reach.rs` comment updated.

### Increment 5, round 1: T34a, the movement and scouting survey (2026-10-21, `adventure/e08-t34a`)

**The cut** (decisions 73, 74; asked before starting, `AskUserQuestion`): T34a whole (not three
parts as recommended, two parts, or the code reading split), and flying and water walking in
(Angel Wings and Boots of Levitation, not out with the spells and the sea as recommended, flying
only, or decided at the checkpoint).

**Baseline:** `master` 5c5c3d0 (#370 merged), engine code e17ebcf6, the full verify with the
original data in the sandbox: green.

**The survey** (`docs/rules/surveys/E08-hero-movement.md`, docs only): the code reading of the
day's land points (0x4E4990 whole, the table loader's item bonuses, every write of a hero's points
left), the step's price (0x4B1740 whole, its callers), the pathfinder's set-up, loop and expansion
for the human player's modes, `hero_flight_tile_test` 0x4E5CE0 and the reveal's callers; VCMI
1.7.5 and the wiki for each; the census of the 160 shipped maps (a throwaway example, not
committed); batch E5's plan (ten rows); the proposed rule ids; a fresh agent's second reading
(below). The address map gained 9 entries and 6 rewritten summaries (`docs/exe/map.toml`), the
struct map the hero's status flags (+0x105) and water-walk field (+0x116), the tile's flags (+0xC)
and the pathfinder's adventure fields.

**Findings** (survey §0): Logistics scales the table's points only, the artifacts' +600 and +300
after it (consensus); a Logistics specialist's rate in single precision (D-22, Q-0005's kind); a
gain in the day changes nothing until the owner's next turn start (consensus); Pathfinding as the
part's table (consensus); Cursed Ground takes away the native 100 (D-23, the code alone); flying's
price 100 from every inland tile, roads included, the lesser only from the coast (D-24, the code
against the wiki and VCMI); water walking at level 3 agrees; the move's drop to 0 forgets the
artifacts (A-12); a flying path crosses unexplored tiles (A-13, an information leak); a flying
hero inland may not visit from its tile (A-14, unconfirmed); where a flying or water-walking move
may end not read (D-25); a grown radius shows at the next step or day (D-26, VCMI at once for a
skill); no SoD hero has the Scouting specialty (HotA's). The checkpoint's questions: ADV-Q20.

**The wiki** needed the user's approval per fetch: the first (an `index.php?title=…&action=raw`
URL) was withdrawn unanswered; page URLs that a search returned were fetched without a prompt.
The HotA/SoD template prints each value before its icon; the summariser took the first for SoD
twice (Logistics 5/10/20 %, Boots of Speed +400) and was corrected by the Movement page's own
Logistics sentence and the executable's `movement.txt`.

**The second reading** (survey §11; a fresh agent, nine questions, without the survey): §1 to §3 confirmed point by point; it settled D-25 (the click's path needs the destination's "may stop" node, 0x56A0D0: a move ends only on a tile a hero may stop on, as the wiki and VCMI have it), and added the two nodes per tile and the flyer's visit only at its destination. ADV-Q20's point on D-25 was dropped.

**The checkpoint** (ADV-Q20, 2026-10-22; decision 75): A-13 fixed (a trade-off for wide obstacle
ranges to design later), A-14 withdrawn (the user: a flyer visits as a walker; the reading's gap
open), D-24 the original's flying price as a fix-optional quirk, D-22 under Q-0005, D-23 built only
if row E5-6 confirms (the user took Cursed Ground for a battle-only effect: the code alone says
otherwise), A-12 fix-optional, D-26 the original's way, the scope and batch E5 as recommended.
Asked again with context: Cursed Ground's place in movement and A-12's tie to it. The survey's §8,
§9 and §12, the queue (T34b ready) and the handoff record it on the same branch (#372).

### Increment 5, round 1: T34b, the ADR amendment drafts, the design note and the cut (2026-10-22, `adventure/e08-t34b` #375)

**The cut** (decision 76; asked at the start, `AskUserQuestion`): T34b whole, one session and one
pull request (not by crate as T24b, recommended; not both drafts first; not the hero side with the
map's drafts first). Docs only.

**Baseline:** `master` 42d3ce6 (#372 merged; the battle lane's E23 cross-lane change on top),
engine code e421fb75 verified on 42d3ce6 (the sandbox ran only the light gates). The user's PC
was linked at the start for the original data (the executable, both LODs, `maps.tar`,
`extracted.tar`) and went offline during the session, as announced.

**A reading that changed a checkpoint answer** (survey §4.2's correction; asked at once with
context, decision 77): `new_day` (0x4C7FE0) calls `reveal_owned` (0x4C7C50) only when
`cover_of_darkness_hide` (0x4C7BA0) returns true, that is when a Necropolis with the Cover of
Darkness (building 17) hid a disk of radius 20 (`hide_disk` 0x49D3D0, inferred); its two other
callers follow hides too (0x4A14B0, an object's visit; 0x402450, the whole map), and
`next_player_turn` reaches no reveal within two calls but through `new_day`. So T8a's and T34a's
"the new day reveals around every owned hero" holds only under a Cover of Darkness, and D-26's
basis (a grown radius shows at the next day) fell: in SoD it shows only at the hero's next step.
**The user: a fix-optional quirk** (`faithful` the original's, `strict-fixed` also revealing at
each new day as decision 75 assumed; not the original's way alone, recommended; not the new-day
reveal in every profile). `new_day` also ends the day's spells (the fly and water-walk levels
+0x112, +0x116 back to −1). The address map gained 4 entries (`hide_disk`, `cover_of_darkness_hide`,
`hide_whole_map`, `visit_hide_area`) and 2 rewritten summaries (`new_day`, `reveal_owned`); the
survey's §0, §4.2, §8 (D-26) and §9 (E5-10's prediction) and ADV-FOG-003's summary say so.

**The drafts** (`docs/lanes/adventure/adr-drafts/`): **ADR-0013** (`0013-increment-5.md`: points 2,
3, 11 amended, point 13 new: movement and scouting as `h3-heroes`' derivations on bases the
adventure gives; six effect kinds; D-22 under Q-0005; no Scouting specialty), **ADR-0016**
(`0016-increment-5.md`: points 3 to 5: the points and radius through the store, set after the
levels at load; the passage of walking, flying and water-walking heroes, moves on explored tiles
only (A-13, `ADV-QUIRK-15`), a move ending only where a hero may stop (D-25), two kinds of node in
the reach for flyers and water walkers, the price with D-24 and A-12 fix-optional, Cursed Ground
after E5-6; the reveal with the radius, `ADV-QUIRK-18`), **ADR-0017** (`0017-increment-5.md`:
points 1, 3, 9: the build's order (points and reveal after the levels at load, as the original's
0x4D8B30 and 0x4CAF74), the start of a day's first rule in `strict-fixed`, the versions).

**The design note** (`E08-design.md`, rewritten whole for increment 5): part A §1–§4 (the six
kinds and their sources, the adventure part's fly and water-walk prices and terrain walker, the
three flags; the derivations HER-ADV-003 to -007 with §1.3's vectors; HER-ADV-007, the Pathfinding
level, new beside the survey's ids since the price reads no skill by name), part B §5–§13 (the
points, the radius, the build and the day; the price with the diagonal's check per profile; the
passage table, the reach's two layers, `MoveHero`'s `NoStop`; the env, plans, fixtures, replays,
benchmarks; rule ids and `ADV-QUIRK-15` to `-18`; versions; batch E5 with an added row E5-11, a
flyer over a border gate, the agent's call; shared work; the cut; the exit criteria; risks).
Increment 4's note moved below (four code comments and the gap-fill's `exe-necromancy` source
still cite its sections: the queue's inbox).

**The cut** (§12): five rounds, **batch E5 first** (rounds 1 and 2: T35 the shared groundwork
with T36 the batch's specs and scorer; T37 the harness's reads and the run with T38 the scoring,
D-23 decided), then T39 + T40 (the derivations; the points, radius and walking price), T41 (flying
and water walking), T42 + T43 (the env, fixtures, replays, benchmarks; the closure). T35 and T36
are in the queue, waiting for ADV-Q21 (six points: the three drafts, the strict diagonal check
with flying, E5-11, the cut).

**The checkpoint** (ADV-Q21, 2026-10-22, the next morning; decision 78): **all accepted** as
recommended (the three drafts, the strict diagonal check with flying, E5-11, the cut with batch E5
first). The user missed an input of ADV-Q20 in ADR-0016's draft, now restated in its point 4, in the
design note (§9, §13), the queue's inbox and the epic: A-13 "probably fix, but gameplay impacts have
to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like
allowing to route to the closest unexplored, but free hex; TBD later anyway" (the draft had it only
as a consequence). The drafts are marked accepted for T35's `shared/` pull request; T35 and T36
ready. `master` (E23 T4b2, E17 M3 T7b-2b-3) merged into the branch first, without conflicts.

#### Increment 4's design note at its closure (moved by T34b)

The note as T33 left it, every section built (T25 to T33); its § numbers are the ones code
comments cite as `E08-design.md` until T39 and T40 repoint them.

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 4's
amendments of ADR-0013 (point 12; drafted in T24b1) and ADR-0017 (drafted in T24b2), accepted
at ADV-Q16 (the epic's decision 58) and applied by T26a (#325), from the answers at ADV-Q15 and
ADV-Q17 (decisions 55, 60). The task that builds a section deletes it; the file goes when the epic
closes. Facts and sources: survey `docs/rules/surveys/E08-progression.md` (§ numbers are the
survey's unless they name this note).
Increment 3's note was pruned by T24b1: every section was built, its records are in `E08-log.md`
(T18a to T22b) and its last versions in §15 below.

Written in two parts (decision 56): **part A**, the hero side (`h3-heroes`), by T24b1; **part B**,
the game side and the cut, by T24b2 (decision 57).

##### Part A — the hero side (`h3-heroes`; T24b1)

Everything in part A is a rule of `h3-heroes`, the hero model beneath the contexts (ADR-0013
point 2): pure functions of a hero's details, the ruleset and explicit inputs (the scenario's
level limit and banned skills, a draw source). Who calls them, when, with which source, and what a
player sees is the game's (part B).

###### 1. The data

- **Built by T26b** (`E08-log.md`; `packs/README.md`, "Hero parameters"): `HeroParams` holds the
  experience of levels 1 to `max_level + 1` (the pack's twelve, then each step + step / 5; 75
  entries, the last the bound of §2), `max_level` 74, `wisdom` and `schools` (the forced skills,
  resolved keys in the original's order); `HeroClass::offer_intervals` (`[wisdom, school]`: 3 and
  3 for the eight classes 0x4DAD74 lists, 6 and 4 else); `[[hero_class_override]]`, with A-10's
  data quirk in the `ab` pack (`quirk.content.elementalist_offer_intervals`, `ADV-QUIRK-10`).
  The level of an experience is then a search of `experience` (§2).
- **The adventure part's terrains** (`packs/*/adventure.toml`, adventure-owned; decision 60) gain
  their straight step's cost by Pathfinding level 1–3 beside the cost without it (the original's
  `step_cost_table` 0x63E510: sand and snow 125, 100, 100; swamp 150, 125, 100; rough 100, 100,
  100; the others as without), for the evaluation's shares (§8, §12) now and increment 5's
  movement later. Built by T29 (adventure ruleset part v9; §11 and §12).
- **Not data**: the scenario's level limit and banned skills are the map's (`RawMap::level_limit`,
  `banned_skills`), inputs from the game (§12).

###### 2. to 7. Built by T27

Built by T27 (2026-10-20, `adventure/e08-t27`, decision 62; `E08-log.md`): `h3_heroes`'
`level` (`experience_for`, `experience_bound`, `level_of`, `experience_cap`,
`gained_under_limit`, `pending_levels`; HER-EXP-002, -003), `level_up` (`Draws`, the
`OriginalDraws` generator, `draw_level_up` → `LevelUp` with `Offers`, `Choice`; HER-LVL-001 to
-004) and `HeroChange::GainLevel` with `apply_change_limited` (HER-LVL-005); `HeroDetails.offered`
(`OfferLevels`: the original's +0x90 and +0x3F) and the details' checks (the experience at most
the bound, the level at most the experience's, the offers' levels at most the level, in
`h3-heroes`: decision 62). What the game side needs from it: a level-up is drawn by
`draw_level_up(details, ruleset, banned, draws)` while `pending_levels` > 0, answered by
`LevelUp::change(choice)` (the game checks the answer against its `LevelUp`), gains go through
`apply_change_limited` with the map's limit. One difference from §3: a gain under a limit lowers
the experience no further than the threshold of the hero's level (the details stay valid); the
original keeps such a hero at its level with less experience, a state the game's setup does not
make (D-21).

###### 8. Built by T28

Built by T28 (2026-10-21, `adventure/e08-t28`; `E08-log.md`): `h3_heroes::original_choice(details,
ruleset, level_up, worth, shares) -> Choice` (HER-LVL-006; the flag-clear path of 0x52BBD0,
0x524690, 0x524DD0), `ArmyWorth` (`SETUP`: 1 000 and 500) and `TerrainShares { tiles, water,
rough }` (the map's counts; the evaluation divides them in the original's f64 through
`h3_kernel::arith::X87`, 0 without tiles); `ADV-QUIRK-14`. What the game side needs from it:
**the shares** are counted once from the static map at the build (§12: over the coast-flagged
tiles that are not rock, the water tiles and the tiles whose straight step costs more than 100
by Pathfinding level 0–2, from the terrains' costs by Pathfinding level of §1) and passed in;
the choice takes the hero before the level's answer and the drawn `LevelUp` (it raises the
level's primary skill itself). The flag-set path (E10's computer players) reads
computer-player state: the army's AI values, the effectiveness words, `ai_school_value`
(survey §10.4).

- **The levels at load** seen from the hero (the game's sequence is GAM-SET-002, §12): after the
  custom skills, army, spells and artifacts, the experience set to 0 and the custom experience
  gained (§3), then each pending level drawn with the original's source and S (§4), two offers
  chosen by `original_choice` with `ArmyWorth::SETUP` and the map's shares, one learned, none
  the primary only, each applied as `GainLevel` (§7); the spell points after (HER-MAN-001). In
  `strict-fixed` the owner's stream draws (D-14's flag).

###### 9. Rule ids and quirks proposed

- Rules (survey §8, ids written when built): HER-EXP-002 (§2), HER-EXP-003 (§3), HER-LVL-001
  (§5), HER-LVL-002 (§4), HER-LVL-003 and -004 (§6), HER-LVL-005 (§7), and **HER-LVL-006** (new,
  §8: the original's choice between two offers). HER-DET-001 and HER-CHG-001 gain the checks of
  §7. The game's GAM-HER-002, GAM-DEC-002, GAM-SET-002, GAM-BAT-008: part B.
- Quirks for `docs/rules/quirks-adventure.md` (confirmed by the second reading, §9), each written
  with its rule (`ADV-QUIRK-11` to `-13` by T27): **ADV-QUIRK-9** a level's draws follow the hero's seed byte and the level (D-14;
  fix-optional, its flag the game's: §11); **ADV-QUIRK-10** the Elementalist's might intervals
  (A-10; fix-optional, the data quirk `quirk.content.elementalist_offer_intervals`, §1);
  **ADV-QUIRK-11** the levels past 74 wrap (A-7), **ADV-QUIRK-12** a limit above 75 caps at a
  wrapped threshold (A-8), **ADV-QUIRK-13** a primary skill wraps at 128 (A-9): fixed, no flag;
  **ADV-QUIRK-14** the computer's terrain shares count only coast-flagged tiles (A-11, survey
  §10.3): the original's in every profile, no flag (decision 60), written by T28.
  The two flags are registered (T26b): 27 `quirk.adventure.level_up_from_seed`, 28
  `quirk.content.elementalist_offer_intervals`; `ADV-QUIRK-10`'s entry is written (its data
  quirk is complete), `ADV-QUIRK-9`'s is written with its rule (GAM-HER-002, T29).

###### 10. The hero side's shared and `cross-lane` needs (for the cut, §17)

Built by T26b (`shared/e08-t26b`, `cross-lane`): the data model and the flags in `h3-content`,
the packs and the harvester's gap-fill, both lanes' replay headers. What remains is the lane's own
(`h3-heroes`, T27 and T28; the game state hash with the details, §15).

##### Part B — the game side (T24b2)

Rules of `h3-game` (and the import's, the env's) around part A's: what the game keeps, when it
calls `h3-heroes`, who answers, what a player sees. ADR-0017's draft holds the decisions.

###### 11. and 12. Built by T29

Built by T29 (2026-10-21, `adventure/e08-t29`, `cross-lane`, decision 63; `E08-log.md`):
`GameSetup.level_limit` and `.banned_skills` (validated: `GameSetupError::BannedSkill`), kept by
`GameState` with each hero's level seed in one shared `Progression` (`state/levels.rs`; game
state hash v10; `level_limit()`, `banned_skills()`); GAM-HER-002 (the seeds, the draws' source
by `ADV-QUIRK-9`'s flag) and GAM-SET-002 (the limit, the bans, the levels at load through
`original_choice` with `ArmyWorth::SETUP`); the battle's experience under the limit
(`apply_change_limited`); the terrains' costs by Pathfinding level (`TerrainMovementDef::
pathfinding`, `AdventureRuleset::step_cost`; adventure ruleset part v9) and the map's shares
counted once by the static map (`StaticMap::terrain_shares`, ADV-MAP-002, not hashed: they follow
from the tiles); the import (v8: the limit, the bans, the refusal lifted; the five shipped maps'
9 heroes get 41 levels at load). **One change from §11**: the seeds are the setup stream's
*last* draws (after the map objects' contents), not each after its hero's defaults, so no earlier
draw moved and the golden replays kept their commands. What T30 needs: `Progression::draw(core,
hero, details, stream)` draws a level from the seed or from the stream given (the owner's in
play); the shares are `state.adventure().map().terrain_shares()`; `Progression::level_limit()`
for every gain.

###### 13. Built by T30: a level-up as a pending decision (GAM-DEC-002); the battle's levels (GAM-BAT-008)

Built by T30 (2026-10-21, `adventure/e08-t30`, decision 64: whole, one pull request, with §14's
env answers and the CLI's words; `E08-log.md`) as planned: the winner's pending levels after the
whole outcome (`end_battle`, GAM-BAT-008), given one at a time (`state/levels.rs`'
`give_levels`): each drawn when given or opened (`Progression::draw` with the owner's stream), one
offer or none at once (`LevelGained`), two a `Decision::LevelUp { owner, view, offered }`
(`LevelUpOpened`); `Answer::First`/`Second`, `GameCommandError::WrongAnswer` (after
`BattleOpen`, before `OfferGone`); the observation's `decision` an `Option<DecisionView>`
(`Offer(Offer)`, `LevelUp(LevelUpView { hero, level, primary, offers })`; `GameState::level_up`);
`legal_commands` lists `First` and `Second`; the events to the owner only (`HeroChanged` tells no
level); the system actor's default `h3_game::default_answer` (a query the hosting layer gives as
the player's answer, the server lane's M5: `Decline` for an offer, the original's choice with
`ArmyWorth::SETUP` and the map's shares for a level-up, `ADV-STUB-8` until E10, none for a
battle); game state hash v11. **An open level-up holds the game** (decision 65, ADR-0017 point 6
amended by `shared/e08-t30-adr`): T30's first build let the other players act on, as the ADR
then said, which allowed a battle against a hero whose level-up was still open; now every
other actor's command is rejected (`WaitingForLevelUp(owner)`, `GameState::check_held`) and
lists no legal command, `GameState::waiting_for` names the owner, and the env's `to_move` yields
the owner alone, in its turn or not (a battle's defender that won answers before the player
whose turn runs goes on). The hot-seat handover is the client's, the paused turn timer, the
owner's time limit and the default answer online the server's (their inboxes). The answer's
next level-up takes the answered decision's place. A hero without an owner does not exist in
increment 4: `give_levels` takes the owner.

###### 14. Built by T30 and T31: the env, replays, fixtures, properties and benchmarks

The env's answers (action encoding v4) and the replay format v5 came with T30 (decision 64).
Built by T31 (2026-10-21, `adventure/e08-t31`, decision 66: whole, one pull request; `E08-log.md`)
as planned: the fixtures' heroes near a level and red's underground hero levelled at load, the
nine golden replays re-recorded (the five searched ones replaced; seven answer level-ups, five
with both offers), the plans' level checks after every command (`check_progression`), the
hygiene property's other heroes' experience and whom the game waits for, a fixed test of
another player's level-up, the levels at load by profile through the original packs, two
workloads re-baselined. The draws' source in play per profile stays `h3-game`'s fixed tests
(T30): `h3-map` reaches no level seed.

###### 15. Versions

Increment 3's last: core ruleset hash v45 (the battle lane's since), game state hash v8,
adventure state hash v10, adventure ruleset part v8, map import v7, action encoding v3, adventure
replay format v4. Increment 4: the core ruleset hash v46 and the pack format v44 (T26b, done:
hero parameters, the class field, the override kind, two flags), the game state hash v9 (T27,
done: the details' counters) and v10 (T29: the seeds, the limit and banned skills), the map
import v8 (T29), the game state hash v11, the action encoding v4 and the replay format v5 (T30:
the level-up decision, its answers), the adventure ruleset part v9 (T29: the terrains' costs by
Pathfinding level). The
adventure state hash does not change. The others bump once, in the task that first changes them.

###### 16. Batch E4

The plan is the survey's §7 (ten rows; E4-1 settles D-20 in SoD, most rows predict from the seed
byte). **Built by T32a** (decision 67): three specs, split by rows as batch E3 —
`experiments/specs/E08-batch-4a.toml` (E4-1 to E4-5, sixteen heroes, Red and Blue eight each),
`-4b.toml` (E4-6 on a map of its own: the level limit binds every hero on its map) and `-4c.toml`
(decision 45's E4-7 to E4-10) — with their results templates. The spec (`h3-experiment`,
battle-owned: `cross-lane`) gained the map's `level_limit` and `[[map.object]]` (the
Cartographers, `h3_map::writer`'s `Plain`, for E4-10); two grounds on one tile it had. The
**scorer** `crates/h3-map/examples/e4_score.rs` (on `h3-heroes`, `h3-map`'s dev-dependency since
`shared/e08-t32a`) reads a batch map and its filled template and predicts each hero's levels at
load and dialogs from its seed byte under each source's rule (the reading as `h3_heroes` draws
it, always-left, VCMI's row, intervals and school weight, the levels drawn up front); its tests
check the reading's draws and levels at load against the engine. **Kept** at the scoring (T33):
it reproduces the reports beside the results, its tests keep checking the reading against the
engine, and a later batch's scorer can start from it. The **recipes** (`clients/capture/batches/`, client-owned) use the harness's reads, built by
T32b1 (decision 68: the seed byte and counters, the skills as a list, every level-up dialog,
`level_up` as a list, the artifacts, the battlefield, the messages, battles between two human
players by `attack` or a plan for E4-9). The run is on the harness (decision 50), T32b2.

###### 17. Shared and `cross-lane` work; the building tasks

**Shared and `cross-lane`**: T26's `shared/` pull request (the accepted ADR amendments applied,
`h3-content`'s data model and registry, part A §10), with the packs and the harvester's gap-fill
(battle-owned) as `cross-lane` in it or beside it and the golden replays' headers of both lanes
re-recorded; T32a's spec (`h3-experiment`, `cross-lane`); the client lane's harness and recipe
(its inbox). Everything else is the lane's own (`h3-heroes`, `h3-game`, `h3-map`, the env's and
the CLI's adventure sub-paths, the fixtures, replays and benchmarks).

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T25** + **T26** | Done. Cut (decision 59): **T25** (2026-10-20: survey §10; ADV-Q17, decision 60), **T26a** (2026-10-20, #325: ADR-0013 and -0017 amended), **T26b** (2026-10-20, `shared/e08-t26b`, decision 61: the hero parameters' level data and offered skills, the class's `offer_intervals`, `[[hero_class_override]]`, the two flags, the packs and the gap-fill, the bumps and the replays' headers; part A §1, §10) |
| 2 | **T27** + **T28** | Done: T27 (2026-10-20, `adventure/e08-t27`, decision 62), T28 (2026-10-21, `adventure/e08-t28`). T27: `h3-heroes`' progression (part A §2–§7: levels, the bound, the limit's cap, the draw source and the original's generator, the primary skill, the offers, `GainLevel`, the counters, the checks; HER-EXP-002, -003, HER-LVL-001 to -005; `ADV-QUIRK-10` to `-13`). T28: the original's choice ported, its flag-clear path (part A §8, HER-LVL-006; survey §10; `ADV-QUIRK-14`) |
| 3 | **T29** + **T30** | Done: T29 (2026-10-21, `adventure/e08-t29`, `cross-lane`, decision 63: whole, one pull request; §11 and §12 built), T30 (2026-10-21, `adventure/e08-t30`, decision 64: whole, one pull request, with §14's env answers and replay format; §13 built). T29: the game's setup and build (§11, §12: the seeds, the limit and banned skills, the terrains' costs by Pathfinding level and the map's shares, levels at load, the import's refusal lifted, the 5 shipped maps; GAM-HER-002, GAM-SET-002; `ADV-QUIRK-9`). T30: the level-up as a decision (§13: the answers, the events, the observation, the system actor's default, the battle's levels; GAM-DEC-002, GAM-BAT-008) |
| 4 | **T31** + **T32a** | Done. T31 (2026-10-21, `adventure/e08-t31`, decision 66: whole, one pull request; §14 built): the replays, fixtures, properties and benchmarks. T32a (2026-10-21, `adventure/e08-t32a`, `cross-lane`, decision 67: whole): batch E4's three specs and the scorer (§16); the recipes wait for the harness's new reads |
| 5 | **T32b** + **T33** | Cut (decision 68): **T32b1** done (2026-10-21, `adventure/e08-t32b1`, `cross-lane`: the harness's reads, built by this lane); **T32b2** done (2026-10-21, `adventure/e08-t32b2`, decision 69): batch E4's recipes and run on the harness. **T33** done whole (2026-10-21, `adventure/e08-t33`, decision 70): the scoring (every rule it covers `verified`, none reopened; E4-10's rule built), the audit against the exit criteria (all six hold: `E08-log.md`), increment 4 closed, increment 5 refined (decisions 71, 72) |

**Increment 4's exit criteria**:
1. A hero's experience gives the original's levels, at most 74, capped by the scenario's limit
   (HER-EXP-002, -003); the import keeps the map's limit and banned skills.
2. A level-up draws the original's primary skill and offers from the seed byte and the level in
   `faithful` (the survey's vectors), from the owner's stream in `strict-fixed`; the
   Elementalist's intervals follow the profile.
3. Two offers are a pending decision of the owner, levels in order, each drawn after the previous
   answer; the battle's winner levels; the env, the replays and the plans play them.
4. Placed heroes with a custom experience get their levels at load, chosen by the ported
   evaluation; 5 shipped maps import whole and their 9 heroes get 41 level-ups at load
   (`verify --with-original-data`).
5. Every new rule has its id, source and tests; `ADV-QUIRK-9` to `-13` are in the ledger; the
   gates pass (coverage, determinism with the re-recorded replays, benchmarks).
6. Batch E4 run and scored; the disputes it covers (D-14 to D-21) `verified` or reopened.

### Increment 5, round 1: T35a and T35b, the shared groundwork (2026-10-22, `shared/e08-t35a` #377, `shared/e08-t35b` #380)

**The cut** (decision 79; asked at the start, `AskUserQuestion`): T35 in two parts, both in this
session, as T26 was: T35a the ADRs (docs only), then T35b the code groundwork; T36 in the next
session. Recommended was T35a with T36 (batch E5's chain T36, T37, T38 is the critical path, and
T35b is not needed before T39); not chosen either: T35a alone, T36 alone.

**Baseline:** `master` 25c01bd (#375 merged), engine code 22188e09 verified on 5a6e583 (the
sandbox ran only the light gates). The user's PC was linked at the start: the repository came from
GitHub, the original data from the PC (the executable, both LODs, `Objects.txt`, `maps.tar`,
`extracted.tar`). The sandbox's VM restarted once during the first question (the setup's
background job died; the tools it had installed stayed).

- **T35a** (#377, docs only): ADR-0013 points 2, 3 and 11 amended and point 13 added (six effect
  kinds the adventure's derivations read, `HER-ADV-`, the derivations), ADR-0016 points 3 to 5
  (the points and radius from the details, a hero's passage with A-13's caveat in the user's
  words, a move's end, the query's two kinds of node, the step's price, the reveals), ADR-0017
  points 1, 3 and 9 (the levels at load before the points and the first reveal, the start of a
  day's first rule, the versions); the index's three "Read when" cells. Each ADR's consequences
  and alternatives are the drafts' condensed. Light gates and `check-lane` passed.
- **`h3-content`** (part A §1): `HeroEffect` gains `Pathfinding { level }` (1..=3),
  `LandMovement { bp }` (≥ 1), `LandMovementPoints { points }` (≥ 1), `Flying { level }` and
  `WaterWalking { level }` (0..=3) and `ScoutingRadius { tiles }` (≥ 1), codes 45 to 50, with
  `NAMES`, `try_map`, the parameter checks and the hash; a list holds each kind once (the default
  collision). No match outside `h3-content` and `h3-pack` needed an arm: `derive_battle_hero`
  picks the kinds it reads, and `clients/` constructs effects only from `BattleEffect`. The
  registry's flags 30 to 32: `quirk.adventure.flying_inland_price` (`FLYING_INLAND_PRICE`,
  `ADV-QUIRK-16`, D-24), `drop_ignores_artifacts` (`DROP_IGNORES_ARTIFACTS`, `-17`, A-12),
  `radius_waits_for_a_step` (`RADIUS_WAITS_FOR_A_STEP`, `-18`, D-26), none read before T40 and
  T41. **Core ruleset hash v47** (the golden base hash moved).
- **`h3-pack`**: **pack format v46** (the six kinds' syntax), a syntax test (the values, an
  unknown field, every parameter error) and a harvested test
  (`the_harvested_movement_and_scouting_effects`: the three skills' and six artifacts' effects
  with and without the expansions, the flags by profile).
- **The harvester** (battle-owned, `cross-lane`): the gap-fill's `[[heroes.skill_effects]]` for
  Pathfinding (1 to 3), Logistics (1 000 to 3 000 bp) and Scouting (1 to 3 tiles: the code's
  table 5 to 8 less the base 5), `[[artifacts.effects]]` for Boots of Speed (600), Equestrian's
  Gloves (300), Angel Wings (flying 3), Boots of Levitation (water walking 3), the Speculum and
  the Spyglass (1 tile each), the six out of the `pending` list; eight sources (`exe-movement`,
  `exe-step-cost`, `exe-scouting`, `vcmi-movement` from VCMI 1.7.5's `config/skills.json` and
  `config/artifacts.json`, read again for this, and four wiki pages from the survey); the three
  flags in both harvested profiles; `exe-necromancy`'s url `docs/exe/map.toml` (the queue's
  inbox). Logistics and Scouting stay out of the harvester's value-table list: Scouting's table
  holds the radius with its base, and the amounts are written as data with their code reading,
  as Wisdom's were. `base` re-harvested from the staged installation: only those lines changed
  (`ab` and `sod` unchanged). `synthetic-ext` gains `wayfarer_sash` with the six kinds
  (`h3-pack-battle`'s `hero_content_syntax` wants every kind used once; not in `synthetic`, so
  the battle replays' hash stays; the synthetic profiles still list only the battle's flags).
  `packs/README.md`'s effect table.
- **Replays and records**: the 19 golden replays keep commands and state hashes; only
  `ruleset_hash` in their headers moved (three values: synthetic faithful 0xccc383cf72e6a6ab, the
  base packs 0x1259740d4579c6b3 faithful and 0x5c100ae884edfb0a strict-fixed); `duel-s2-d7` and
  `duel-s5-strict-d7` re-recorded with their options differed in that line only. The AI lane's
  `bots-report-v1.json` regenerated: besides `engine.core`, two pairs' rewards and two values
  moved in the fourth decimal, wins unchanged; a regeneration on `master` 25c01bd gives the same
  table, so the drift is `master`'s (the battle lane's creature-cast change of 2026-10-21), not
  T35b's.
- **Docs**: the three drafts deleted; the design note's part A §1 and §4 cut to what T39 still
  needs; decision 79 and the round's line in the epic; the queue (T35 out, an inbox note: the data
  without a reader until T39 to T41); the battle and AI lanes' inboxes; the battle HANDOFF's
  shared versions. The quirks' ledger entries come with their rules (T40, T41), Q-0005's line
  with HER-ADV-003 (T39).
- **Verify**: the first full run with the original data failed two steps: coverage
  (`h3-pack-battle`'s `hero_content_syntax`: the synthetic packs must use every kind, hence
  `wayfarer_sash`) and `check-docs` (the adventure HANDOFF 100 bytes over). The second, on
  d403160, passed all 16 steps, and again on 8c982b6 after `master` was merged in (#377, E23 T4c) (benchmarks within +0.7 %, the harvest check matching the
  installation). The clients' check (`clients/scripts/check.sh --no-coverage`) passed every gate
  once SDL's build dependencies were installed in the sandbox (the client lane's setup installs
  them, the adventure lane's does not). CI cannot run (billing).

### Increment 5, round 1: T36a, batch E5a and E5b's specs (2026-10-22, `adventure/e08-t36a`, `cross-lane`)

**The cut** (decision 80; asked at the start, `AskUserQuestion`): T36 by rows, T36a now with the
fixed predictions, T36b (the seed byte's rows E5-2, E5-3, E5-10 and their scorer) in the next
session. Not chosen: both parts as two pull requests this session, T36 whole, T36b first. Found
while sizing it: the writer and the spec placed only Cartographers, and E5-7 needs trees, E5-11 a
Border Gate.

**Baseline:** `master` eb1f35a (#380 merged); the sandbox's baseline ran the full gates on its
engine code (5a41242d): all 16 steps passed. The user's PC was linked at the start (the repository from GitHub, the
original data from the PC: the executable, both LODs, `Objects.txt`, `maps.tar`, `extracted.tar`);
it went offline later, after everything was copied.

- **The writer** (`h3_map::writer`): `PLAIN_OBJECTS` gains the trees (Oak Trees 135, Pine Trees
  137, Trees 155) and the Border Gates (212, the key's colour as subclass): no body in the file,
  as the reader's `Body::None` for them. A writer test writes a tree and a gate; the read-back test
  imports a tree as a blocker (no type) and the light blue gate as `ab:border_gate_light_blue`.
- **The spec** (`h3-experiment`, battle-owned, `cross-lane`): `[[map.object]]` takes their
  pictures (`Tables.plains` already filters by `PLAIN_OBJECTS`); a test checks a tree's one tile,
  a gate's three with the middle one its entrance, and the shared-tile errors for both.
  `docs/experiments/README.md` and `docs/formats/h3m.md` say so.
- **Batch E5a** (`experiments/specs/E08-batch-5a.toml`, 36 x 36, grass): Blue's eight E5-1 heroes
  never move (10 Pikemen, 1560: no Logistics, Basic, Advanced, Expert, Boots of Speed, Equestrian's
  Gloves, both with Expert Logistics, Valeska with Archers; 1560 to 2928, read at Blue's turn
  starts on days 1 and 2); Red's E5-4a flies along an inland cobblestone road, then over grass and
  swamp (the reading 100 per step, the wiki and VCMI 50 on the road), E5-4b along a coastal road
  (50, every source); E5-5 crosses a lake with Boots of Levitation from sand and back with a
  diagonal (450, 491: a water walker cannot stop on water, so each walk crosses whole); E5-6a's
  Gnolls step off the RoE Cursed Ground on swamp (the reading 175, VCMI 100) and **E5-6b** off the
  SoD one (class 223; the agent's call: `tile_special_ground` 0x4FCE20 does not list 223, the map
  loader was not read, and the engine types it apart as `sod:cursed_ground`).
- **Batch E5b** (`E08-batch-5b.toml`): E5-9 first, a flyer and a walker west of a strip of five
  unexplored columns, a scout east of it (the walker's click the control: no path; the reading:
  the flyer's green path of 13 marks, 260 left; VCMI and the engine: none); E5-7a a flyer clicks a
  tree beyond a tree line (no path), then the free tile before it (3 marks over the line); E5-7b a
  water walker clicks a lake tile (no path), then the far shore (4 marks); E5-11 a flyer two rows
  south of a light blue Border Gate clicks two rows north of it (the reading: straight over the
  entrance, 1160 left; over a blocked side 1078; stopped by the gate 1460 and its message).
- **Checked in the engine**: both maps compile, read, import and validate
  (`original_data` on a folder holding them; the tiles' coast flags, roads and grounds as
  designed; E5-9's strip unexplored for Red at the build, the targets explored) through a
  throwaway example (not committed).
- **Not built**: the recipes and the run (T37: the harness's explored-tile read for E5-9; a read
  of the points at a turn start without a walk for E5-1); T36b's specs and scorer.
- **Verify**: the full run with the original data on de9f71b (`master` a64631e merged in, #382)
  passed all 16 steps (engine code 4e17824f; benchmarks within +0.7 %). CI cannot run (billing).

### Increment 5, round 1: T36b, batch E5's seed-byte rows and their scorer; round 1 closed (2026-10-22, `adventure/e08-t36b`, `cross-lane`)

**The cut** (decision 81; asked at the start, `AskUserQuestion`): T36b whole in one pull request,
its level-ups forced by the map's banned skills. Not chosen: several heroes per row (batch E4's
way), T36b cut with E5-2 first.

**Baseline:** `master` c84370f (#383 merged); the sandbox's baseline ran the full gates with the
original data on its engine code (4e17824f, the code T36a verified): all 16 steps passed. The
branch starts from `master` 0f989da (#386, battle only). The user's PC was linked at the start (the
repository from GitHub; the original data from the PC: the executable, both LODs, `Objects.txt`,
`maps.tar`, `extracted.tar`); it went offline later, after everything was copied.

- **Banned skills** (decision 81): `h3_map::writer::Map::banned_skills` (the original's codes;
  SoD's header's four bytes, bit `code`; a code from 28 on is refused) and `h3-experiment`'s
  `[map] banned_skills` by `SSTRAITS.TXT` name (battle-owned, `cross-lane`); the writer's other
  maps (the smoke example, `h3-cli`'s AI tests, `h3-view`'s situation: `cross-lane`) say none.
  Tests: the writer refuses code 28; the read-back test writes three, reads the bits and imports
  Navigation, Eagle Eye and First Aid; the spec maps names in any case and refuses an unknown one.
- **Batch E5c and E5d** (E5-2, D-22): Gunnar, Kyrre and Dessa (the three Logistics specialists)
  with Basic Logistics and 10 Dwarves (speed 3, 1500), levelled at load from their experience:
  E5c Gunnar 2, Kyrre 6, Dessa 12; E5d Kyrre 2, Gunnar 3 (both arithmetics agree: the control),
  Dessa 6; a hero type once per map, Red's two and Blue's one never moving; the points read at
  both turn starts of days 1 and 2. **Measured in the engine first** (the levels at load over 400
  games): the computer's evaluation takes Logistics whenever offered, so a level-2 hero ends Basic
  or Advanced always (both part from exact there), a level-6 one with Basic Logistics alone ends
  Expert in 81 % (Kyrre) and 92 % (Dessa), where both agree; with seven more skills at Basic,
  filling the slots so that Logistics is one advance among eight, in 25 % and 59 %. So the level-6
  heroes know Archery, Armorer, Offense, Leadership, Luck, Resistance and First Aid too; level 12
  ends Expert in 99 % either way, so E5-2c checks the specialty's growth at a high level (2220
  under both) rather than D-22.
- **Batch E5e** (E5-3, E5-10): a map banning all but Logistics and Scouting; the heroes who level
  know no skill and have 999 experience, one Pikeman east (10 hit points: level 2, one dialog, the
  two offered in the order S gives; the scorer's test checks it for every seed byte and three
  classes). Red's E5-3a takes Logistics (1560, 1460 after the attack, 1360 after a step, 1716 at
  day 2), E5-3b puts Boots of Speed on from its backpack (1560, 1460, 2160): the backpack instead
  of the survey's pile (no artifact objects in the writer; `hero_equip_artifact` 0x4E2A00 is on
  §1.4's list as `hero_give_artifact` is). Blue's E5-10a takes Scouting, E5-10b puts the Spyglass
  on, neither steps on day 1; three probes each on their row (W 6 west: radius 6 around the
  standing tile only; E 7 east: radius 6 around the day-2 step's tile only; C 8 east: never),
  read at the start, after the gain, at day 2's start and after the step: the reading E only
  after the step, VCMI W at once for the skill, the engine's strict-fixed W at day 2. Red plays
  E5-3 and Blue E5-10, so each player's explored tiles come from its own heroes. Checked in the
  engine: every probe unexplored for Blue at the build, the tiles at distance 5 explored.
- **The scorer** (`e4_score`, extended rather than an `e5_score`: the levels at load and the
  dialogs are batch E4's): each map hero carries the land table's entry at its army's slowest
  speed; E5-2's records get the day's points for the reading's hero after its levels at load and
  for the hero recorded (or its level and skills alone, played by hand), single precision (Q-0005,
  the x87's arithmetic through `h3_kernel::arith::X87`, a new dev-dependency of `h3-map`) and
  exact, and which fit every recorded day; a row without a seed byte still scores its points;
  E5-3's and E5-10's dialogs are scored as E4's, under the map's bans. Tests: §1.3's vectors and
  E5-1's from the base pack, the forced offers, E5-2's report.
- **Templates**: E5c to E5e's, and E5a's and E5b's, which T36a had not committed (the README's
  flow commits them). Every map compiles, reads, imports and validates (a throwaway example, not
  committed).
- **Not built**: the recipes and the run (T37: the explored-tile read, a read of the points at a
  turn start without a walk, an equip from the backpack in the hero screen); the scoring (T38).
- **Verify**: the full run with the original data on 736a00f (`master` 0f989da) passed all 16
  steps (engine code 947c5b97; benchmarks within +0.7 %); `h3-view` checked in `clients/` (SDL's
  build packages installed for it). CI cannot run (billing).

### Increment 5, round 2: T37a, the harness's reads for batch E5 (2026-10-22, `adventure/e08-t37a` #389, `cross-lane`)

**The cut** (decision 82; asked at the start, `AskUserQuestion`): T37 in two, the reads first (as
T32b1, T32b2). Not chosen: T37 whole, the reads with batches E5a and E5b.

**Baseline:** `master` 1b280e0 (#387 merged); the sandbox's baseline ran the full gates with the
original data on its engine code (7feae1f3): all 16 steps passed. The user's PC was linked at the
start: the repository from GitHub, the original's sixteen files from the PC (the executable, the
five DLLs, the four LODs, the three sound and video archives, `maps.tar`, `extracted.tar`,
`Objects.txt`), so the harness ran under Wine in the sandbox. The branch changes no engine code.

- **A tile's explored bit** (`h3mem.Mem.explored`; `h3adv`'s `explored:x,y[,level][:colour]`,
  yes or no, the player the step's hero's owner unless named): the tile's vision word
  (`tile_vision_word` 0x4F79B0 read again: index (level × height + y) × width + x into
  `tile_vision_pointer` 0x6989F8, the map's side for both on the square maps), bit n for colour n.
  `before:explored:` is refused (the step's snapshot holds no tiles).
- **A hero's points at a turn start without a walk:** a read step's `points` already gave the step
  hero's +0x4D (the "check" of §11); added the hero details `points` and `day_points` (+0x49, the
  movement bar), so any hero's are read by name (`hero[E5-1a]:points`, batch E5a's table).
- **An artifact put on from the backpack** (the step `equip = { "<artifact>" = "<slot>" }`): the
  hero screen opened by a click on the selected hero on the map (new probes `hero_screen`, from the
  primary skills' labels, checked on two heroes' screens), a click on the artifact's place in the
  backpack (the cursor takes it; the backpack in memory loses it at once) and one on the slot,
  checked in memory, the screen closed with `Return`. Places checked under Wine: the backpack's
  (425 + 46k, 387), the feet (537, 317), misc 1 (405, 165); the other slots are VCMI's hero
  window's places, unchecked. Only the first five backpack places (no scrolling).
- **Wider tables** (batch E5a's, E5b's, E5e's: a row with several value columns): `fill` and
  `records` key a cell by `<row label> | <column>` (`"W (2, 10) | after the battle"`), a table of
  one value column by its label as before.
- **The harness check** gained Blue's Fog (Tyris at (28, 30): probes at its disk's edge, before
  and after a step east, and Red's bit on its tile) and Equip (Sorsha at (28, 3), Boots of Speed
  and the Spyglass in the backpack, put on in the hero screen; its points at Blue's day-2 start
  without a walk); Red already held eight heroes (the list's most). **Passed under Wine in two
  full runs** (about 2.5 minutes each; the runs differ only in the random starting gold, seed
  bytes and their draws): Fog's probes as ADV-FOG-002's disk of radius 5 (dx² + dy² ≤ 30): (33,
  30) and (33, 32) explored at the start, (34, 30) and (33, 33) not, all four after the step;
  Red's bit clear; points 1560, 1460 after the step. Equip: Boots of Speed (feet) and Spyglass
  (misc 1) worn, the backpack empty; **the points left 1560 after the screen, the day's points
  1560 → 2160 at its close** (as §1.4's reading says: the close writes only +0x49); day 2: 2160
  and 2160. Seen while exploring by hand: neither value changes while the screen is open.
- **Not built:** batch E5's recipes and the runs (T37b); the scoring (T38).

### Increment 5, round 2: T37b, batch E5's recipes and its run (2026-10-22, `adventure/e08-t37b`, `cross-lane`)

**The cut** (decision 83; asked at the start, `AskUserQuestion`): T37b whole, as T32b2. Not
chosen: by the specs' tasks (E5a and E5b first), or the seed-byte batches first.

**Baseline:** `master` f494cb1 (#389, #390 merged); its engine code (783c406f) had no run on
record, so the sandbox ran the full verify with the original data: all 16 steps passed. The
user's PC was linked at the start: the original's sixteen files staged from it as T37a's (the
sandbox setup copies only the engine's four into `original-data/install`: the harness's other
LODs, sound archives and DLLs copied by hand before `clients/capture/setup.sh`). The branch
changes no engine code.

- **The recipes** (`clients/capture/batches/E08-batch-5{a,b,c,d,e}.toml`, `cross-lane` as T37a):
  the specs' procedures as steps on T37a's reads; each hero's points read before its first walk
  (`hero:points`), a walk's cells by `<row> | <column>`, E5-1's eight heroes by name; `note`
  reads each hero's worn artifacts but the war machines (the spec's own, or a starting bonus's).
  Each recipe's records match its template's fields one for one (checked with `fill` before the
  runs).
- **The harness** (the same `cross-lane`): `level_up` may name a skill (`"Logistics"`), taken on
  whichever side it stands (the left if neither: E5e's forced offers, whose side the seed byte
  decides); `path_tiles[:<row>]`, the tiles of the path's marks at the first click (E5-11's
  "tile on row 14"). Runbook updated.
- **Every batch ran clean at its first run** (E5a 26 steps, E5b 12, E5c and E5d 9, E5e 19; no
  stop, no unplanned dialog, no first click that walked); E5c and E5d ran twice (new seed
  bytes). Templates, and `e4_score`'s reports beside E5c's to E5e's, in `experiments/results/`
  (`E08-batch-5c.md`, `-run2`, their `-score.md`).
- **Batch E5a:** **E5-1** 1560, 1716, 1872, 2028, 2160, 1860, 2928, 1630 on both days, every
  source's (no starting bonus's artifact on them). **E5-4a: the road steps cost 50 each (1510,
  1460)**, the grass and swamp steps 100 (1360, 1260, 1160, 1060; flying: the swamp's 175
  replaced): **against the reading's D-24** (100 from every inland tile), as the wiki and VCMI
  (the lesser of 100 and the walking price). The road (3–5, 4) lies 5 rows from the lake; the
  writer flags only land next to water as coastal (`h3m.md`). **E5-4b** 1510, 1460 (every
  source). **E5-5** 4 green marks, 1110; 4, 619 (every source). **E5-6a 1385, 1210, 1110: the
  reading's D-23** (on the Cursed Ground the native 100 is lost; VCMI's 1460, 1360, 1260
  refuted); **E5-6b the same**: the SoD Cursed Ground (223) acts as class 21.
- **Batch E5b:** **E5-9** (13, 24) unexplored and (18, 24) explored before; the walker's control
  no path (0 marks, no move); **the flyer a green path of 13 marks straight east through the
  strip (6, 24) to (18, 24), arrived with 260: the reading's A-13** (VCMI and the engine: no
  path). **E5-7a** the tree no path, the hero stays with 1560; (7, 5) 3 marks over the tree line
  ((7, 7), (7, 6)), 1260. **E5-7b** the water tile no path; the shore 4 marks, 1160 (every
  source). **E5-11: a green path straight over the gate's entrance (31, 14), 4 marks, arrived on
  (31, 12) with 1160, no message: the reading** (§3.1 point 7; neither closed at the entrance
  nor stopped at the gate). Blue's hero wore a Pendant of Free Will (the starting bonus).
- **Batches E5c and E5d (E5-2, D-22), twelve heroes over two runs:** every level at load the
  reading's (E5-2b, -2c, -2e, -2f against always-left), but run 2's E5-2c (seed 183): skills,
  offers and levels the reading's, attack 7 against 5, its starting bonus's Centaur's Axe (+2;
  the scorer's "fits: none", as batch E4a's run 2). **Six heroes conclusive, all for the
  reading's single precision (Q-0005)**: Advanced at level 2 **1829** (exact 1830) four times
  (E5-2a both runs, E5-2d both runs), Basic at level 6 **1694** (1695; run 2's E5-2b), Advanced
  at level 6 **1889** (1890; run 2's E5-2f). Inconclusive: Expert at levels 6 and 12 (2085,
  2220), level 3 (1845). Both days alike on every hero. Run 2's E5-2f wore Cards of Prophecy.
- **Batch E5e:** **E5-3a** (seed 222) 1560; the Pikeman fought (1009 experience), one dialog,
  level 2, knowledge, **Scouting | Logistics** (the scorer's), Logistics taken right; **1460 after
  the battle**, 1360 after the step, **day 2 1716**; it wore a Pendant of Dispassion (the starting
  bonus). **E5-3b** 1560, **1560 with the boots on** (the day's points, +0x49, 2160: the bar),
  1460 after the step, **day 2 2160**. **E5-10a** (seed 114) one dialog, level 2, defense,
  **Logistics | Scouting** (the scorer's), Scouting taken right, 1460 after the battle; **W, E,
  C unexplored after the battle and at day 2's start, E explored after the step** (W and C not).
  **E5-10b** the same with the Spyglass put on. Every row the reading's (§1.4, D-26): no gain
  shows in the day, no reveal at the new day (VCMI's reveal at the skill and strict-fixed's at the
  new day refuted for the original).
- **For T38:** E5-4a contradicts the reading's D-24 (decision 75 took it as the original's rule,
  a fix-optional quirk): an ADV-Q at the scoring, and the survey's §2.1 point 5 read again.

### Increment 5, round 2: T38, batch E5 scored; the tile bit read again (2026-10-22, `adventure/e08-t38`, `cross-lane`)

**The cut** (decision 84; asked at the start): T38 whole. **Baseline:** `master` 38f227f (#392
merged); its engine code (783c406f) was verified on that commit, so the sandbox ran the light
gates (passed). `Heroes3.exe` staged from the user's PC for the re-read. `cross-lane` for the
status row in `docs/experiments/README.md` (battle-owned), as T33.

**Batch E5 scored** (survey `E08-hero-movement.md` §13, from T37b's results and `e4_score`'s
reports): E5-1, E5-3, E5-5 and E5-7 as the consensus; **D-22** six conclusive heroes of six fit
single precision (Q-0005); **D-23** confirmed for both Cursed Grounds (21 and 223): built in T40;
**D-26** (E5-10) and **A-13** (E5-9: the original's flyer crosses the unexplored strip) as read;
**E5-11** a flyer passes over a Border Gate's entrance without the key, no message, as read.
**E5-4 refuted D-24 as read**: an inland road's steps cost 50 under Angel Wings.

**The re-read** (`step_cost` 0x4B1740 at 0x4B17EA, then the bit's writers): the reading itself was
right (the lesser from a tile with +0xC bit 0x40, the fly price from water or a tile without it);
the bit's meaning was not. The map's tile loader `map_tiles_load` (0x4FE220, from
`map_objects_place_all` 0x4FD690 before the objects) copies only the file's six mirror bits into
+0xC, turns the file's coast bit into +0xD bit 0x02 (the shore battlefield's test, 0x4643B0) and
sets +0xC bit 0x40 on every tile but rock; `tile_refresh_object` (0x505610), for each tile of an
object placed or removed, sets it again and clears it (with +0xD bit 0x01) where an object's
passability blocks the tile without an entrance there. So the bit means "no object blocks the
tile". Three readings rested on it as the coast:
- **D-24**: the lesser of the fly and walking price from every free land tile (an entrance too),
  the fly price from water and from blocked tiles. Every E5-4, E5-7, E5-9 and E5-11 price fits.
  The wiki's text agrees; VCMI takes the lesser over obstacles too (at level 3 only a blocked
  tile carrying a road differs).
- **A-14's gap** (`hero_flight_tile_test` 0x4E5CE0): the test passes on every free land tile, so a
  flyer visits as a walker, as the user said at ADV-Q20.
- **A-11** (`ai_map_terrain_shares` 0x52B9C0, built as ADV-MAP-002 under `ADV-QUIRK-14` in T29):
  the shares count every tile no object blocks but rock, water included, not the coast. The
  experiment maps with levels at load are all grass, where both counts give 0.

**ADV-Q22** (decision 85, asked interactively): D-24 the original's rule in every profile, no
flag (`ADV-QUIRK-16` withdrawn; flag 30 and ADR-0016 point 4's words in T41's shared part);
ADV-MAP-002 corrected now as **T38b** (`ADV-QUIRK-14` withdrawn).

- **Docs:** `docs/exe/map.toml` (0x4FE220 new; 0x4B1740, 0x4E5CE0, 0x4FD690, 0x505610, 0x52B9C0
  corrected), `structs.toml` (`map_item` +0xC, +0xD); survey `E08-hero-movement.md` §0, §2.1,
  §2.3, §2.5, §3.3, §8, §13 and `E08-progression.md` §10.3; `ADV-QUIRK-14` marked withdrawn (removed
  with its test in T38b); `E08-design.md` §6, §9, §11 to §13.
- **The catalog:** observation sources for ADV-MOV-002 (E5-1, E5-3), ADV-MOV-008 (E5-9's walker;
  `verified`), ADV-FOG-003 (E5-10; `verified`), GAM-SET-002 (E5-2's twelve levels at load); a
  note on ADV-MAP-002's source. The new rules (HER-ADV-003 to -007, ADV-MOV-009, -010) get
  batch E5's observations when T39 to T41 write them.
- **For T41's shared part:** `h3-content`'s `FLYING_INLAND_PRICE` (flag 30) and its packs'
  profiles, ADR-0016 point 4 ("the fly price from every inland tile"); the queue's inbox.

### Increment 5, round 2: T38b, ADV-MAP-002's count corrected; round 2 closed (2026-10-22, `adventure/e08-t38b`)

Decision 85 (ADV-Q22 point 2), right after T38 in the same session, on T38's branch (#393 open).

- **The rule:** `StaticMap::terrain_shares` (ADV-MAP-002) counts, after the objects are placed,
  every tile of every level that is not impassable and that no object blocks (a tile an object
  is visited from counts, as the original's entrance keeps +0xC bit 0x40), water included; the
  coast flag plays no part. `ADV-QUIRK-14` withdrawn (its ledger entry a stub, the id not
  reused; no longer in ADV-MAP-002's, HER-LVL-006's and GAM-SET-002's `quirks`); the test
  `the_terrain_shares_count_the_tiles_no_object_blocks` replaces the coast one; `h3-heroes`'
  `TerrainShares` docs and two test comments say so.
- **What moved:** the duel's red underground hero's levels at load in three of the nine golden
  replays (`duel-s1-d28`, `duel-s3-easy-d7`, `duel-s236-d28`; the duel has water and rough
  terrain): re-recorded with their options, the commands of all nine unchanged, only those
  three's checkpoints and final hashes moved. `e4_score`'s reports re-run for E4a and E5c from
  freshly compiled maps: the same predictions (the experiment maps with levels at load are all
  grass: both counts give 0 shares).
- **Round 2 closed** (T37a, T37b, T38, T38b). Round 3 queued: T39 + T40 (`E08-design.md` §12),
  the cut at its start.
- **The full local verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master
  -- --with-original-data` on engine code ebd76b02 (commit 97457a6, 2026-10-22, the sandbox,
  Linux x86_64): all 16 steps passed; `original-data`: 5 shipped maps import whole, 9 heroes
  levelled at load with 41 level-ups (the counts unchanged).

### Increment 5, round 3: T39, `h3-heroes`' movement and scouting derivations (2026-10-22, `adventure/e08-t39`, `cross-lane`)

Decision 86: T39 alone (T40 in a session of its own, cut at its start), Q-0005's line in the
battle lane's ledger in the same pull request (`cross-lane`, the user's consent).

- **The derivations** (`crates/h3-heroes/src/movement.rs`, `E08-design.md` part A, ADR-0013
  point 13): `land_points(details, core, base)` (HER-ADV-003: `derive.rs`' `grown`, now
  `pub(crate)`, over the `land_movement` rates as Learning's experience, then the
  `land_movement_points` of the worn types added, saturating); `movement_speed(details, core,
  creature)` (HER-ADV-004: the core's speed plus each creature specialty's `speed` for its
  creature and direct upgrades; `u8::MAX` for a creature the core lacks); `movement(details,
  core) -> Movement { pathfinding, flying, water_walking }` (HER-ADV-005, -007: the highest of
  each kind, the levels `None` without a source; one call instead of the note's three getters);
  `scouting_radius(details, core, base)` (HER-ADV-006: the tiles added, no specialty growth).
  Every kind once per artifact type (`effects`' `OncePerType`).
- **The vectors** (`crates/h3-heroes/src/tests/movement.rs`, a test pack built in code): the
  survey's §1.3 in both arithmetics (17 cases, Basic to Expert, the tables' 1500 and 2000),
  ⌊base × (10 + level) / 10⌋ without a specialty for every base to 10 000 in both, batch E5's
  E5-1 (2160, 1860, 2928) and E5-2 (1829, 1889 single, 1830, 1890 exact): all as predicted at
  the first run. The base pack's data through the derivations: `e4_score`'s
  `the_base_packs_movement_effects_derive_as_batch_e5_saw_them` (Boots of Speed, Equestrian's
  Gloves, Valeska's Archers 1630, Angel Wings, Boots of Levitation, Pathfinding, Scouting with
  the Speculum and the Spyglass 10).
- **The scorer** (`crates/h3-map/examples/e4_score.rs`): its copy of the arithmetic replaced by
  `h3_heroes::land_points` under `base:faithful` and `base:strict-fixed` (`strict_core`), the
  land table at the slowest `movement_speed`; its reports on E5c, E5c-run2, E5d, E5d-run2 and
  E5e from freshly compiled maps identical to the committed ones below their headers.
- **Docs:** `heroes.toml` HER-ADV-003 to -007 (the code reading, VCMI, the wiki, batch E5's
  observations); Q-0005's ledger entry names HER-ADV-003 in its rules, regression tests and
  decisions (battle-owned, `cross-lane`; the battle lane's inbox told); `h3-heroes`' three module
  comments repointed to increment 4's note in this log; `E08-design.md`'s part A reduced to what
  T40 builds (the adventure part's data).
- **The full local verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master
  -- --with-original-data` on engine code 2391924a (commit 7d98ced, 2026-10-22, the sandbox,
  Linux x86_64): all 16 steps passed; `original-data`: 9 heroes levelled at load with 41
  level-ups (unchanged). `check-lane` names only `docs/rules/quirks-ledger.md` (the label).

### Increment 5, round 3: T40, the day's points, the radius and the walking price; round 3 closed (2026-10-22, `adventure/e08-t40`)

Decision 87: T40 whole (the price and the day in one session and one pull request).

- **Ruleset part v10** (`h3-adventure`'s `ruleset.rs`, `h3-pack-adventure`): `[params]`' `fly_cost`
  and `water_walk_cost` (four prices by level each, required; the base pack's 140, 140, 120, 100
  from `fly_walk_step_cost` 0x6778AC; read by T41), `AdventureParams::fly_cost`,
  `water_walk_cost`; `[[terrain_walker]]` (`TerrainWalkerDef`/`TerrainWalker`: a creature, a
  terrain and the terrain it counts as; both land, one walker per terrain,
  `AdventureCompileError::TerrainWalker`), the `ab` pack's Nomads (sand as dirt); a ground's
  `no_native` (`ObjectKind::Ground`), on both Cursed Grounds. `ADVENTURE_RULESET_HASH_VERSION`
  10. The misplaced doc line of `pathfinding_fits` moved to `core_terrain`.
- **The walking price** (`movement.rs`): `Pricing::new(core, part, army, &Movement)` precomputes
  per terrain the walking price at the hero's Pathfinding level of the terrain a walker in the
  army makes it count as, and whether that terrain is the army's native one; `base` gives the
  native cost only off a `no_native` tile; `left_after` prices a `no_native` tile's terrain (the
  original's `cheapest_step_from` 0x4B1880, re-read: it hands its own arguments on as
  `step_cost`'s Pathfinding level and Nomads flag, the tile's terrain as native; its caller not
  read); `cheapest` over the hero's prices. The static
  map's `StaticTile::no_native` from the last ground in map order lying on each tile (as
  `battle_ground`), left out of the hash. `MoveHero` and `reach` take the hero's `Movement` from
  the store (`details_of`; an empty store: no skills, this crate's tests).
- **The day** (`state.rs`, `h3-game`'s `state.rs`): `AdventureState::new` places the heroes
  without points or reveals; `start_game(core, part, heroes)` gives the day's points
  (`day_points(core, part, details, army)`: `land_table_points` at the lowest
  `h3_heroes::movement_speed`, then `h3_heroes::land_points`) and reveals with
  `h3_heroes::scouting_radius`; `start_turn` takes the store; `start_day` reveals around every
  hero under `strict-fixed` (`ADV-QUIRK-18` off). `GameState::new` calls `start_game` after the
  levels at load, then `start_day` for day 1; `advance_day` calls `start_day` after the calendar.
  A move reveals with the hero's radius. `reveal_at_start` (and its stale doc) is gone.
- **Tests:** `movement/tests.rs` (the details' points, radius and levels; Pathfinding's prices and
  ADV-QUIRK-3's check at a level; a terrain walker in every price, the native test and the check;
  Cursed Ground in the price, the diagonal and the drop), `state/tests/day.rs` (the start of the
  game from the store, a gain in the day under both profiles, a move's and the reach's price by
  Pathfinding, the grounds' bit and a move over it), `ruleset/tests.rs` (walkers, prices, the
  hash), `h3-pack-adventure` (the syntax, both packs' grounds), `h3-game`'s `tests/day.rs` (a
  creature and Logistics specialist with the boots: 2257 at level 1; the points after the levels at
  load). `testing::started` builds a state as the game does; `hero_rules` adds the skills and a
  walker.
- **Replays:** every game's commands changed (the duel's heroes' points and prices): the nine
  re-recorded with their options (the master binary reproduces the committed ones byte for byte
  with the same script); what each meets is re-checked by T42 (the queue's inbox).
- **Benchmarks:** the changed games moved `reach-xl-144` by +4.13 % (4 784 queries) and
  `random-play-duel-36` by -2.04 % (2 839 steps); both re-baselined. The code itself: a scratch
  build with the details' effects held off (the day's points, the levels and the radius computed
  and dropped, no `no_native` bits) replays master's nine games byte for byte and measures
  -0.09 % (reach), +0.13 % (play), +0.17 % (fork, not re-baselined) against the old baselines.
- **Docs:** the catalog (ADV-MOV-001 to -004, -006, ADV-FOG-001, -003; -004 and ADV-FOG-001
  verified by E5-6 and E5-10), `ADV-QUIRK-18`'s ledger entry, `e4_score`'s `land_table` on
  `land_table_points`, `levels.rs`' citations repointed to increment 4's note in this log, the
  design note (§5 and §6's built points reduced), the epic, the queue (T41's entry), the HANDOFF.
- **The full local verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master
  -- --with-original-data` on engine code 8235b24f (commit 55d1642, 2026-10-22, the sandbox,
  Linux x86_64): all 16 steps passed; `original-data`: 9 heroes levelled at load with 41
  level-ups (unchanged). The baseline on `master` e4dd551 (code 7214d773): all passed.

### Increment 5, round 4: T41, flying and water walking; round 4 closed (2026-10-22, `shared/e08-t41`, `adventure/e08-t41`, `cross-lane`)

Decision 88: T41 whole (the user: "no cut"), its shared part in its own pull request.

- **The shared part** (`shared/e08-t41` #401, `cross-lane` with the user's consent): `h3-content`'s
  registry position 30 (`quirk.adventure.flying_inland_price`, `ADV-QUIRK-16`) `RETIRED` with a
  note, `FLYING_INLAND_PRICE` gone; `h3-pack`'s harvested test on the two flags left; both
  harvested profiles (`h3-harvest`) and `packs/base/core.toml` without the line, and
  `h3-pack-battle`'s profile test with position 30 among the retired ones (battle-owned);
  ADR-0016 point 4's price as built (one fix-optional quirk, `ADV-QUIRK-17`; Cursed Ground
  confirmed). `base:faithful`'s core ruleset hash moved (0x1259740d4579c6b3 →
  0xde583eb06ef4dd73; strict-fixed and the synthetic packs unchanged): the seven faithful
  adventure replays' headers (commands and hashes unchanged), `python/records/bots-report-v1.json`
  regenerated (AI-owned: `engine.core`, and `engine.adventure` catching up with T40's part v10,
  which #399 left at v9's; every rate unchanged). The battle and AI lanes' inboxes.
- **Two properties after T40** (in the shared pull request, and on the lane branch alike): the
  shared branch's full verifies found, in their coverage runs, a case each of `h3-map`'s
  `views_move_as_the_events_say` and `every_step_keeps_the_commands_invariants` where an end of
  turn that starts a day changed another player's explored tiles: in strict-fixed `start_day`
  reveals around each hero (`ADV-QUIRK-18` off, T40), so a radius grown since the hero's last step
  shows then. On a new day both now ask only that those tiles grow (the view's fringe follows
  them); both cases are in the regressions files. Master fails them rarely until this merges.
- **The shared verify** took four full runs: coverage is a full verify's only run of the tests
  (`test` then runs the doc tests alone) and stops at the first failing test binary, so each fix
  uncovered the next (the two properties, then the battle's profile test).
  `cargo test --workspace --exclude xtask --all-targets --no-fail-fast` lists every failure at
  once. The fourth, `scripts/verify-once.sh engine --base origin/master -- --with-original-data`
  on b154ce9 (code 8b89cb94, 2026-10-23, the sandbox): all 16 steps passed.
- **The price** (`movement.rs`): `Pricing` holds the hero's fly and water-walk prices (`Lifts`,
  from `Movement`'s levels by `AdventureParams::fly_cost`, `water_walk_cost`) and the ones the
  drop counts (none under `ADV-QUIRK-17`, flag 31); each terrain's `water` bit. `lifted`: from
  water the water-walk price, then the fly price from water or a live-blocked tile and the lesser
  from a free land tile (an entrance included); `step` applies it, its diagonal check C is the
  walking one under `ADV-QUIRK-2` or `-3` and the step's own straight price clean; `left_after`
  applies it with the drop's lifts; `cheapest` includes both prices. Callers pass the tiles as they
  stand now (`MoveHero`'s start tile through `live` too).
- **The passage** (`state/passage.rs`): `Mode` (walk, fly, water walk; a flyer that water-walks
  flies), `Node` (stop, pass), `Entry` (the live tile, the stop it allows: a free stand or a
  visit, and whether the hero may pass) and `AdventureState::passage`, the design note's §7 table
  for a flyer or a water walker, its rows in order: off the map, unexplored (`ADV-QUIRK-15`),
  down onto a closed top, up off one (a flight's pass), rock, water (a flight passes, a water walk
  plain water), heroes (own: a flight's pass; another player's: a battle from a stop, a flight's
  pass), blocked tiles, a visit not built, a gate with its key, an entered object (from any
  node), a neighbour's (from a stop). A walker keeps `check_step` and its own search. `Meets` is
  `pub(crate)`.
- **`MoveHero`** (`state.rs`): a flyer's or water walker's step goes through `passage` from the
  node the step before left it on; a step before the last continues (a pass when the hero may pass,
  else a stop), the last stops or visits, or `AdventureCommandError::NoStop { step }` (new). A
  flyer meets a guard only at its last step's tile and visits only there (0x48147D); a water walk
  meets guards as a walk. Every step, passes included, reveals and emits `HeroStepped`.
- **The reach** (`reach.rs`): for a flyer or water walker `search_layered`, Dijkstra's over
  (slot, kind) with a second layer `passing`/`passing_via`, each node replaced only by a strictly
  better one of its kind, the predecessor's kind in the direction byte's bit 3 (`FROM_PASS`); a
  visit's stop node ends there, a guard ends any node of a hero that does not fly; `path` follows
  the recorded kinds. The walker's search keeps its loop (`bench-check` below).
- **Tests:** `movement/tests.rs` (a flight's prices by level from free land, an entrance, an
  obstacle with a road, water; a water walk's, E5-5's 450 and 491; the diagonal check with and
  without the quirks; the drop under `ADV-QUIRK-17`; the levels from the details), the new
  `state/tests/flight.rs` (E5-7, E5-9 and E5-11 as cases: passes, `NoStop`, visits only at the end
  and from a stop, objects without a visit, own and other heroes, guards only at the end, closed
  tops, the query's stop tiles and paths), `the_query_and_move_hero_agree` over the three modes
  and levels (`hero_rules`, a store). The test rules' faithful profile has flag 31; `hero_skills`
  gains invented flying and water-walking skills (levels 1 to 3: 140, 120, 100).
- **Measured** (an ad-hoc test, not committed, as increment 1's): with the diagonal exception
  switched off the layered query matched an exhaustive search over (tile, kind, points) on all 600
  random maps up to 8 × 8 (both profiles, flyers, water walkers and both, levels 1 to 3): 12 316
  stop tiles, none differing; with the exception on, none differed either on these maps.
- **Replays:** none changed (no fixture hero flies or water-walks; T42 adds them).
- **Benchmarks:** the first lane verify measured `reach-xl-144` +7.32 % and `random-play-duel-36`
  +6.76 % on the same games (4 784 queries): callgrind showed the walking search's loop with the
  price inlined into it (master keeps `Pricing::step` a call). The walker now prices by
  `Pricing::walking_step` (`#[inline(never)]`, without the lifts' test) and
  `walking_left_after`, the same prices for a hero that neither flies nor water-walks: `reach`
  +0.89 %, `play` +0.83 %, `fork` +0.17 %, within the 2 % (not re-baselined). The rest is the
  search's set-up (the terrains' water bit, the mode).
- **The lane's full verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master
  -- --with-original-data` on fd4f16f (code d82c93aa, 2026-10-23, the sandbox, Linux x86_64): all
  16 steps passed (the first run on 04e16ff failed coverage, `NoStop`'s message, and the benchmarks
  above). Merges cleanly with `master` 2ad29e2 and with #401.

### Increment 5, round 5: T42, the env, the plans, the fixtures, the replays and the benchmarks (2026-10-23, `adventure/e08-t42`, `cross-lane`)

Decision 89: T42 whole (the user's choice; the cut at the committed fixtures not chosen).

- **The plans** (`h3-map`'s `play.rs`): `Kit` (worn artifacts of `MOVERS`, the six with their
  editor slots, and Pathfinding, Logistics, Scouting at a level), drawn last in the plan's
  strategy so the regressions files' cases keep their maps: in two heroes of five some of the
  Speculum, the Spyglass, Equestrian's Gloves and Boots of Speed, in two of five some of the
  skills; in three plans of four the first two heroes fly (Angel Wings) and walk on water (Boots
  of Levitation), in one of those of four the flyer both, with a strip of three water tiles two
  columns east of each; in half the AB and SoD plans a stack of Nomads (`Plan.nomads`). A kit's
  skills join the hero's own (the Spell Book with them). `every_step_keeps_the_commands_invariants`
  checks that every accepted move ends where a hero may stop (`stops_on`: land no live object
  blocks but where one is visited); the observation property compares the legal moves as before,
  now over flyers' (`ADV-QUIRK-15`). New fixed test `the_plans_fly_and_walk_on_water` (64 cases):
  the six artifacts worn in 21, 21, 21, 42, 42, 18 games, the three skills known in 33, 38, 35;
  36 flights over water, 33 over blocked tiles, 20 water walks over water, 57 moves of armies with
  Nomads, 6 paths refused with `NoStop`. All 71 of `h3-map`'s lib tests pass.
- **The fixtures** (`fixtures.rs`' `DUEL_KITS`, `XL_KITS`; terrain and objects unchanged): the
  duel's red surface hero flies with the Speculum and basic Logistics, eight tiles from the
  surface lake; blue's surface hero wears Boots of Speed and Equestrian's Gloves with advanced
  Pathfinding; red's underground hero walks on water with the Spyglass, seven tiles from a lake;
  blue's underground hero has expert Scouting and basic Pathfinding. Blue's underground hero was
  the water walker at first: the underground lakes nearest to it lie beyond rock, and in four
  random weeks it never reached water. The XL map: each player a flyer and a water walker, one per
  level, the other artifacts and the three skills among the others.
- **The env** (`h3-sim`): `a_flyer_and_a_water_walker_are_masked_by_their_stop_tiles` (four
  random games of two weeks: every tile a lifted hero's move head allows is one it may stop on,
  some reached over water or blocked tiles); `steps_agree_with_the_masks_and_the_game` plays the
  duel's flyer and water walker. The tampered-replay test prints the result it got.
- **`h3_game` re-exports `h3_heroes::{Movement, movement}`** (for the benchmarks: which heroes fly
  or walk on water).
- **The replays** (`adventure_replays.rs`, its list rewritten): the four fixed ones re-recorded
  with their options; a search over 5 000 seeds per profile (28 days, a scratch probe, not
  committed) replaced T31's five searched ones by the closest games (`duel-s692-d28`,
  `duel-s4893-d28`, `duel-s2517-d28`, `duel-s3127-d28`, `duel-s1308-strict-d28`) and added
  `duel-s2931-d28`, the only faithful game with an offer to join for gold accepted (the fixed
  `duel-s3-easy-d7` no longer meets one). Not met by any replay now: a strict-fixed monster's
  battle lost together with a hero battle (no game has both). The test asks for ten files.
- **The benchmarks:** the reach workload counts the walkers in `reach_queries` and the flyers and
  water walkers in `lifted_queries`, which hero is which sorted out of the counted part
  (`heroes_by_lift`); `queries` takes the kind so that the release build does not fold the two
  identical functions into one symbol (it did: both names at one address, so the pattern
  `*lifted_queries*` matched nothing). On master's maps the new code measures `reach-xl-144`
  +0.86 % (T41's own +0.89 %), `random-play-duel-36` +0.85 %, `fork-xl-144` +0.17 %. Re-baselined
  on the new maps: `reach-xl-144` 466 864 446 Ir (3 145 walkers' queries, about 148 k each),
  `reach-lifted-xl-144` new, 880 080 619 Ir (3 158 queries, about 279 k each, 1.9 times a
  walker's), `random-play-duel-36` 976 085 207 Ir (+75.7 %: 3 205 steps, +56 % a step, the duel's
  flyer and water walker search two kinds of node at every step), `fork-xl-144` 356 400 024 Ir
  (+0.17 %). `benches/README.md`'s two rows (a shared file) in the lane's pull request, `cross-lane`
  with the user's consent.
- **`--with-original-data`:** the Tutorial's red hero (Expert Logistics and Pathfinding, Boots of
  Speed and Equestrian's Gloves, a speed-4 army) starts with 2 928 points, its entry 1 560 × 1.3 +
  900, printed and checked (`original_data.rs`' `tutorial`).
- **Docs:** the design note's §8 reduced to what was built, §10 and §13's lines; the queue (T43's
  entry; the inbox's T40 note closed), the HANDOFF, the epic, `fixtures/maps/README.md`.
- **The baseline:** `scripts/verify-once.sh engine -- --with-original-data` on master 8330f55
  (code 70c8a38e, 2026-10-23, the sandbox): all 16 steps passed.
- **The lane's full verify** (CI cannot run): `scripts/verify-once.sh engine --base origin/master
  -- --with-original-data` on 678768f (code 68058c9c, 2026-10-23, the sandbox, Linux x86_64): all
  16 steps passed (`original-data`: the Tutorial's 2 928 points; 9 heroes levelled at load with 41
  level-ups, unchanged); `master`'s client-only 9cb5725 merged in after it (the code unchanged).

### Increment 5, round 5: T43, the audit; increment 5 closed; increment 6 refined (2026-10-23, `adventure/e08-t43`)

Baseline: `master` at afa36f8 (T42 merged, #406); its engine code (68058c9c) was verified on
678768f (T42's full verify with the original data), so the sandbox ran the light gates (passed).
No original files staged: the task changes docs only. T43 whole, one session and one pull request
(the user's instruction for the session: a seam only for a significantly big task).

**The audit of increment 5's exit criteria** (design §12, moved below), on `master`'s code:

1. The day's points: `h3_heroes::land_points` (HER-ADV-003: the Logistics rate, a Logistics
   specialist's growth under Q-0005, the worn Boots of Speed and Equestrian's Gloves) and
   `movement_speed` (HER-ADV-004: a creature specialty's speed; T39), read by
   `h3_adventure::day_points` (ADV-MOV-001, T40); set by `AdventureState::start_game` after the
   levels at load and by `start_turn` for its owner's heroes (ADV-MOV-002, T40). Observed: E5-1,
   E5-2 (D-22), E5-3; the Tutorial's red hero's 2 928 under `--with-original-data` (T42).
2. The price: `Pricing` with the Pathfinding level (HER-ADV-007), a terrain walker (the `ab`
   pack's Nomads) and Cursed Ground's `no_native` (T40); the fly and water-walk prices
   (`Pricing::lifted`, T41: the lesser from a free land tile, the fly price from water and a
   blocked tile, decision 85); the drop under `ADV-QUIRK-17` (`Pricing::left_after`, T41).
   Observed: E5-4, E5-5, E5-6 (both Cursed Grounds).
3. The passage: `AdventureState::passage` (ADV-MOV-009, -010; the unexplored tile's row under
   `ADV-QUIRK-15`), `MoveHero` with `NoStop` and a flight's guard and visit at its end,
   `Reach::search_layered`'s stop and pass nodes (T41); the env's masks
   (`a_flyer_and_a_water_walker_are_masked_by_their_stop_tiles`), the ten replays (the duel's
   flyer and water walker) and the plans (`the_plans_fly_and_walk_on_water`; every accepted move
   ends where its hero may stop) play them (T42). Observed: E5-7 (D-25), E5-9 (A-13's original
   path; the engine keeps to explored tiles), E5-11.
4. The radius: `h3_heroes::scouting_radius` (HER-ADV-006, T39), `hero_radius` at the start and
   at each step (ADV-FOG-001, -003, T40); `start_day` reveals around every hero in `strict-fixed`
   (`ADV-QUIRK-18` off). Observed: E5-10 (D-26).
5. `check-rules` passes (every rule an id, a source and a test); the ledger holds `ADV-QUIRK-15`,
   `-17` and `-18` with their regression tests (`a_flight_never_crosses_an_unexplored_tile`,
   `the_drop_forgets_flying_and_water_walking_under_adv_quirk_17`,
   `a_gain_in_the_day_shows_at_the_next_turn_start_and_step`), `-14` and `-16` withdrawn, flag 30
   retired (T41's shared part); Q-0005's entry names HER-ADV-003 (T39). The gates: T42's full
   verify with the original data passed all 16 steps on the code `master` holds (68058c9c): the
   replays re-recorded, `reach-xl-144` at the walkers' baseline (+0.86 % on the old maps),
   `reach-lifted-xl-144` baselined.
6. Batch E5 run (T37b) and scored (T38, survey §13): D-22 to D-26 decided and observed, A-13's
   original path observed (E5-9; the fix provisional), E5-11 the reading's; none reopened.

**The gap fixed:** HER-ADV-003, -004 and -006 cite batch E5's observations (E5-1, E5-2; E5-1h;
E5-10), but T39 wrote them `implemented` after T38's scoring: now `verified`. HER-ADV-005 gains
E5-4's and E5-5's observation (the steps from swamp and from water at 100: both artifacts at level
3, where level 2 gives 120) and is `verified`. HER-ADV-007 (the Pathfinding level) stays
`implemented`: no row of batch E5 had a Pathfinding hero. Nothing else was missing.

**The design note** moved below ("Increment 5's design note at its closure"); `E08-design.md`
keeps a pointer until T44b writes increment 6's note. Two code comments cite it by section
(`crates/h3-adventure/src/state/passage.rs`: §7's table, since T41 the module's doc comment;
`crates/h3-sim/src/adventure/tests.rs`: §8): kept, the next code change of the lane repoints them
(a comment alone would cost a full verify).

**Increment 5 closed** (2026-10-23).

**Increment 6 refined** (ADV-Q23; decisions 90, 91): the user chose **army and hero management**
of the sketch's item 6 cut in three (the army dialog of `ADV-STUB-2`, meetings and exchanges of a
player's own heroes, moving, splitting, merging and dismissing stacks, the formation command, the
hero's artifact commands), not rewards on the map, boats and the sea, or the whole item; the rest
of item 6 is the sketch's increment 7, the week's 8. The first round is a survey and the ADR
amendment drafts with the design note and the cut (T44a, T44b), not the survey alone. The inbox
re-tagged: the army dialog (T17a, T17b), the formation (T22b), ADR-0016's line (T23), a received
combination's part, Titan's Thunder's book and the Grail's capture (T19a2a), the backpack's
capacity (E04a T3b) and the hero commands (the E04 refinement) for increment 6; Favorable Winds
(T20a2a) and Pandora's Box's artifact (T20a2a) for the sketch's increment 7.

#### Increment 5's design note at its closure (moved by T43)

The note as T42 left it, every section built (T35 to T42); its § numbers are the ones the two
code comments above cite as "design note".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 5's amendments
of ADR-0013 (point 13), ADR-0016 (points 3–5) and ADR-0017 (points 1, 3, 9), drafted in T34b from
the answers at ADV-Q20 (the epic's decision 75) and at T34b (decision 77: D-26 revised), accepted
at ADV-Q21 with the cut (decision 78; ADR-0016's with A-13's caveat restated) and applied in T35a
(#377). The task that builds a section
deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-hero-movement.md` (§ numbers are the survey's unless they name this note).

Increment 4's note was moved by T34b into `E08-log.md` ("Increment 4's design note at its
closure"): every section was built (T25 to T33); the code's citations of it were repointed (T35b,
T39, T40).

Written whole (decision 76): **part A**, the hero side (`h3-heroes` and the core content); **part
B**, the map, the game, the batch and the cut.

##### Part A — the hero side (`h3-heroes`, the core content)

**Built** (the log): the core data in T35b (`shared/e08-t35b`: the six hero-effect kinds, the
gap-fill's effects, the flags 30 to 32; flag 30 to be retired in T41's shared part, `ADV-QUIRK-16`
withdrawn at ADV-Q22, decision 85), the derivations in T39 (`h3_heroes`' `movement.rs`:
`land_points`, `movement_speed`, `movement` with its `Movement` of the Pathfinding, flying and
water-walking levels, `scouting_radius`; HER-ADV-003 to -007 in `heroes.toml`; Q-0005's entry
names HER-ADV-003), and the adventure part in T40 (ruleset part v10: `[params]`' `fly_cost` and
`water_walk_cost`, the `ab` pack's `[[terrain_walker]]` for Nomads, both Cursed Grounds'
`no_native`; the log). Part B reads them: since T40 `day_points`, `Pricing`'s Pathfinding and the
reveals' radius (§5, §6); flying and water walking since T41 (§6, §7).

##### Part B — the map, the game, the batch and the cut

Rules of `h3-adventure` and `h3-game` around part A's: when a hero's properties are read, how
its steps are priced and where it may pass and stop. ADR-0016 and ADR-0017 hold the
decisions.

###### 5. The day's points and the radius; the build and the day (ADV-MOV-001, -002, ADV-FOG-001, -003)

**Built by T40** (the log): `day_points` from the store, `AdventureState::start_game` after the
levels at load, `start_turn` with the store, the radius per move, `ADV-QUIRK-18` and the strict
`AdventureState::start_day`, called in `advance_day` and for day 1. A step over a tile a flight
passes reveals as any step (T41's passage).

###### 6, 7. The step's price; the passage, the reach and `MoveHero` (ADV-MOV-005 to -007, -009, -010)

**Built by T40 and T41** (the log): `Pricing` with the hero's Pathfinding, a terrain walker and
Cursed Ground (T40), the fly and water-walk prices, the diagonal's check and the drop under
`ADV-QUIRK-17` (T41, `Pricing::lifted`); `AdventureState::passage` (this note's table, now in its
doc comment and ADR-0016 point 4; `ADV-QUIRK-15`), `MoveHero`'s passage with `NoStop` and a
flight's guard and visit at its end, the reach's pass nodes (`Reach::search_layered`). The
exhaustive search's equality without the diagonal exception was measured (the log), not added as
a test, as for the walker in increment 1.

###### 8. The env, replays, fixtures, properties and benchmarks

**Built by T42** (the log): the plans' kits, flyers and water walkers beside water and Nomads, a
move's end where its hero may stop, `the_plans_fly_and_walk_on_water`; the duel's and XL map's
kits; the env's test of a flyer's and a water walker's masks; the replays re-recorded (six
searched, one added); the reach workload's walkers and lifted heroes apart (`reach-xl-144`,
`reach-lifted-xl-144`); the Tutorial's hero's day's points checked under `--with-original-data`.

###### 9. Rule ids and quirks (the map side)

**Built** (T40, T41): ADV-MOV-001 to -007 and ADV-FOG-001, -003 amended, ADV-MOV-009 and -010 new
(`verified` by batch E5); the ledger holds `ADV-QUIRK-15`, `-17` and `-18` (`-16` withdrawn, flag
30 retired in T41's shared part).

###### 10. Versions

Increment 4's last: core ruleset hash v46, pack format v45 (the battle lane's E23 T3-0), game
state hash v11, adventure state hash v10, adventure ruleset part v9, map import v8, action
encoding v4, adventure replay format v5. Increment 5: the pack format v46 and the core ruleset hash
v47 (T35: the six kinds, the three flags), the adventure ruleset part v10 (the fly and water-walk
prices, the terrain walker; Cursed Ground's `no_native` in the same bump if E5-6 is scored first,
which the cut does; bumped by T40). The game and adventure state hashes, the map import, the action encoding and
the replay format do not change (no new state; a flyer's destinations are tiles of the same
mask): T41 and T42 bumped nothing. Each bumps once, in the task that first changes it.

###### 11. Batch E5

The plan is the survey's §9: rows E5-1 to E5-10, E5-8 dropped (A-14 withdrawn), E5-10's
prediction corrected by T34b (§4.2: nothing new after the battle nor at day 2; the radius 6 after
the first hero's step). *Agent's call (change freely):* **E5-11**, an Angel Wings hero facing a
border gate whose key it lacks, clicks a free tile beyond it; the reading predicts a path over the
gate (an entrance a flyer passes, §3.1 point 7), which no other source states (§13).

- **Spec and scorer: built** (T36a, T36b; decisions 80, 81; the log): batches E5a to E5e
  (`experiments/specs/E08-batch-5*.toml`, their templates in `experiments/results/`), E5-2 on two
  maps (a hero type once per map; the level-6 heroes with eight skills, since the evaluation at
  load takes Logistics whenever offered), E5-3 and E5-10 on a map banning all but Logistics and
  Scouting (the forced offers), the gains in the day from the backpack instead of a pile;
  `crates/h3-map/examples/e4_score.rs` scores E5-2's points (D-22) and E5-3's and E5-10's dialogs.
  Since T39 the scorer calls `h3_heroes::land_points` under `faithful` and `strict-fixed` (the
  same §1.3 vectors; its reports on batch E5 unchanged).
- **The harness's reads** (`clients/capture/`, client-owned; built by this lane, `cross-lane`, as
  T32b1): a tile's explored bit for a player (`explored:x,y`, the vision word against the player's
  bit; E5-9, E5-10, E5-11), a hero's points at a turn start without a walk (check whether a read
  step's `points` gives it), the next turn's start after `end_turn` (exists), the path's mark
  (exists); for E5-3b and E5-10b an artifact moved from the backpack to a slot in the hero screen
  (or the rows are skipped, as the survey's plan allows).
- **The run: done** (T37b, decision 83; the log): the recipes
  `clients/capture/batches/E08-batch-5*.toml`, every batch on the harness (decision 50) at its
  first run, E5c and E5d twice; the results and the scorer's reports in `experiments/results/`.
- **The scoring: done** (T38, decisions 84, 85; survey §13): every row the reading's but E5-4,
  whose inland road steps cost 50; the re-read found the tile bit the flying price tests means "no
  object blocks the tile", not the coast (§6 point 4). The same misreading had made A-14 and
  `ADV-QUIRK-14` (A-11, the computer's terrain shares): ADV-Q22 (decision 85) withdraws
  `ADV-QUIRK-16` and `-14`; ADV-MAP-002's count corrected by T38b (three golden replays' hashes moved).

###### 12. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T35's `shared/` pull request (the three accepted drafts applied;
`h3-content`'s kinds and flags, `h3-pack`'s syntax; part A §4) with the gap-fill and the packs
(battle-owned) and both lanes' replays' headers as `cross-lane` in it, as T26b; batch E5's spec
(`h3-experiment`, `docs/experiments/`; T36) and the harness's reads and recipes (`clients/capture/`;
T37), `cross-lane`; the client lane's inbox if `clients/` needs the new kinds mapped. Everything
else is the lane's own (`h3-heroes`, `h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, the
adventure packs, the env's and the CLI's adventure sub-paths, fixtures, replays, benchmarks).

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). *Confirmed at ADV-Q21 (decision 78):* batch E5 runs **before** the build (rounds 1 and 2), so
that its answers (D-23's Cursed Ground above all, and D-22, D-24) shape the rules once instead of
reopening them, and nothing in the batch needs the engine's increment 5 (it tests the original;
the scorer needs only increment 4's draws):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T35** + **T36** | T35: the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016 and -0017 amended from the accepted drafts; the six kinds, the three flags, their syntax; the packs' effects through the gap-fill; pack format v46, core ruleset hash v47; both lanes' replay headers; `clients/` checked (part A §1, §4). T36: batch E5's specs, results templates and scorer (§11; `cross-lane`) |
| 2 | **T37** + **T38** | T37: the harness's reads, the recipes and the run of batch E5 (`cross-lane`). T38: batch E5 scored, D-23 decided by E5-6; the survey's §8 and the rules' sources updated; an ADV-Q if a row contradicts the reading. *Added by decision 85:* **T38b** (done), ADV-MAP-002's count corrected (every tile no object blocks but rock, water included), `ADV-QUIRK-14` withdrawn, before round 3 |
| 3 | **T39** + **T40** | T39 (done alone, decision 86): `h3-heroes`' derivations (part A; HER-ADV-003 to -007, §1.3's vectors; Q-0005's line). T40 (done whole, decision 87): the day's points, the radius and the walking price (§5, §6 points 1, 2, 7): ADV-MOV-001 to -004 amended, the build's order, `start_turn` and the reveals with the store, `ADV-QUIRK-18` and the strict start-of-day reveal, Nomads, Cursed Ground (confirmed); adventure ruleset part v10 |
| 4 | **T41** (done whole, decision 88) | Flying and water walking (§6 points 3 to 6, §7): the price (the original's, decision 85; flag 30 retired and ADR-0016 point 4's words corrected in its shared part), the drop (`ADV-QUIRK-17`), the passage, the reach's two kinds of node, `MoveHero`'s passage, end, visits and guards; ADV-MOV-006, -007 amended, -009, -010; `ADV-QUIRK-15` and `-17`. The largest task: cut at its start if needed (the price and the drop; the reach and `MoveHero`) |
| 5 | **T42** (done whole, decision 89) + **T43** | T42: the env, the plans and properties, the fixtures and replays, the benchmarks, `--with-original-data` (§8). T43: the audit against the exit criteria, increment 5 closed, increment 6 refined |

*Not chosen* at ADV-Q21: the batch at the end as in increment 4 (T35, T39 to T42 first, then
the batch and its scoring: the order of increments 3 and 4, at the risk of reopening a built rule
and building Cursed Ground late).

**Increment 5's exit criteria**:
1. A hero's day's points follow its Logistics (a specialist's growth by Q-0005), its creature
   specialty's speed and its worn Boots of Speed and Equestrian's Gloves (HER-ADV-003, -004,
   ADV-MOV-001), set at the build after the levels at load and at each turn start of its owner
   (ADV-MOV-002).
2. A step's price follows the hero's Pathfinding, Nomads in its army, flying and water walking
   (the original's flying price, decision 85), and Cursed Ground (E5-6); the drop to 0
   follows `ADV-QUIRK-17`.
3. A flying or water-walking hero passes as §7's table, never over an unexplored tile
   (`ADV-QUIRK-15`), and ends a move only where it may stop; the reach reports those tiles with
   paths `MoveHero` accepts; the env, the replays and the plans play flyers and water walkers.
4. A hero's radius follows Scouting and its worn Speculum and Spyglass (HER-ADV-006); a grown
   radius shows at the hero's next step, in `strict-fixed` also at the next day's start
   (`ADV-QUIRK-18`).
5. Every new rule has its id, source and tests; `ADV-QUIRK-15`, `-17` and `-18` are in the ledger (`-16` withdrawn, flag 30 retired) and
   Q-0005's entry names HER-ADV-003; the gates pass (coverage, determinism with the re-recorded
   replays, the benchmarks with the walker's search at its baseline and a flyer's baselined).
6. Batch E5 run and scored; D-22 to D-26, A-13's original path and E5-11 `verified` or reopened.

###### 13. Risks and open points

- **The passage table rests on the code reading alone** where the wiki and VCMI are silent: a
  flyer passing over a border gate without its key, over another player's hero, and a step up
  off an object not visited from above landing only in flight. Fixed tests pin the reading
  (`crates/h3-adventure/src/state/tests/flight.rs`, T41);
  E5-11 checks the gate in SoD; the wiki's Border Gate and Border Guard pages are worth a look in
  T41. E5-11 confirmed the gate (a flyer passes over its entrance without the key, no message).
  A-14's gap (0x4E5CE0) was the tile bit misread as the coast (T38): the test passes on every free
  land tile, so a flyer's start node is a walker's, as built.
- **A-13's cost** (the user at ADV-Q20 and ADV-Q21: "probably fix, but gameplay impacts have to
  be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like
  allowing to route to the closest unexplored, but free hex; TBD later anyway"): a flight over a
  wide obstacle range stops at its unexplored edge. Increment 5 builds the fix; the compromise is
  designed later (the queue's inbox), with what a flight's end beyond hidden tiles may tell the
  player (`docs/06-security.md`). Keep the reach's passage rules in one place, so that the
  compromise changes the unexplored tile's row of §7's table and nothing else.
- **The reach's speed:** the second layer only for flyers and water walkers; the walker's hot
  loop must not gain a branch (`bench-check`'s +2 %). T41: `reach` +0.89 %, `play` +0.83 % with
  the walker's own unlifted price (the log); a flyer's and a water walker's reach baselined by
  T42 (`reach-lifted-xl-144`).
- **Not in increment 5** (survey §6): the sea's points (Navigation, its specialists, the
  Necklace of Ocean Guidance, the Sea Captain's Hat, the Admiral's Hat, Lighthouses, Favorable
  Winds), Stables, the spells Fly, Water Walk and Dimension Door with their levels 0 to 2 (the
  day's end of a spell, `new_day` 0x4C80CF), the computer players' bonus (E10), the cheat flag,
  the new-day reveal after a Cover of Darkness (E09).

### Increment 6, round 1: T44a, the army and hero management survey (2026-10-23, `adventure/e08-t44a`)

Baseline: `master` at 49b19b8 (T43 merged, #407); its engine code (68058c9c) is the code T42's
full verify passed, so the sandbox ran the light gates (passed). Staged from the user's repo
folder once the device link was back: `Heroes3.exe`, `extracted.tar` (the text tables),
`maps.tar` (the census); the wiki approved by the user for the session. **T44a whole** (decision
92, the user's choice of four at the session's start; the instruction for the session: a seam only
for a significantly big task, confirmed with the user). Docs only.

**The survey** `docs/rules/surveys/E08-army.md` (§0 to §12): the army dialog (§1), meetings with
Scholar and the exchange screen (§2), a hero's army in the hero screen and between two heroes, the
split dialog's bounds, the dismissal and `army_swap_slots`' 16-bit count (§3), the formation and
the tactics formation flag (§4), the artifacts: positions, the backpack, what never moves,
combinations, Titan's Thunder, the derived properties (§5), the commands, decisions, visibility,
env, replays and benchmarks (§6), the census (§7), disputes and quirk candidates (§8: D7, D-27,
D-28, A-15, A-16), batch E6's plan (§9: E6-1 to E6-11), the proposed rule ids (§10), the second
reading (§11), the checkpoint's questions (§12, ADV-Q24).

**Findings beyond the queue's list:** the hero screen's third flag button (the tactics formation,
+0x48 bit 1, default on; off clears a human side's tactics phase: a seam question for the battle);
Scholar at every meeting; the backpack's arrows rotate the stored array (the capture walks it
from the end); the original's 19 positions decide "the first position that takes it" and the
capture's order, which the engine's kinds-only `HeroDetails.worn` loses (question 3); the
assembly question once per player and combination (the player record's +0xE8), the right-click
always; disassembly from worn only (VCMI also from the backpack: D-28); a split oddity (A-16) and a
state-corrupting 16-bit count on a move or swap (A-15).

**The exe map:** 51 entries (the army dialog and its garrison exchange, the town screen's garrison
logic it reuses, the exchange screen's manager and clicks, the split and creature dialogs, the
combinations' helpers, Scholar, the hero screen's globals; H3API's `UpdateLuckMorale` guess at
0x5AE500 contradicted: the exchange manager's constructor), four updated (`army_full_dialog`,
`army_add_stack`, `hero_screen_proc`, `hero_can_replace_artifact`: now `read`), two struct fields
(the hero's +0x48 bit 1, the player record's +0xE8). The surveys' README gained this survey's row
and `E08-hero-movement.md`'s, missing since T34a.

**The second reading** (survey §11): a fresh agent, from ten questions and without the survey,
confirmed §1 to §5 point by point; its additions (the held artifact lost when the visitor has no
room, `hero_give_artifact`'s question for an equipped part only, +0x53C5's likely meaning, the
exchange's drops skipping a team-mate's owner check) are in the survey.

**The census** (survey §7): 373 placed heroes, 40 with custom artifacts, 11 with a backpack, none
with a combination, its parts or Titan's Thunder; 3 with seven stacks; one tight.

**The checkpoint** (ADV-Q24, asked interactively; decisions 93–100): the meeting a pending
decision per step; the army dialog a decision with the group as a small army; the original's 19
artifact positions in `h3-heroes`; the tactics flag passed as Tactics 0, with the user's condition
that the two heroes' Tactics keep cancelling (both go as 0 when the leading side's flag is off);
Scholar with meetings; A-16 the original's bounds (A-15 fixed); the adventure's RL encoding handed
to the AI lane (the user's own answer; the AI lane's inbox told); batch E6 whole. T44b is ready.

**Not read** (survey's sources): network and allied play, the computer player's join without room
and meeting (0x52BC60, 0x525DC0: E10), the town screen's own garrisons and the upgrade (E09), the
reward callers of the army dialog (increment 7), `battle_start`'s other tactics branches
(0x69877C, `game` +0x1F69D, network).

### Increment 6, round 1: T44b, the ADR amendment drafts, the design note and the cut (2026-10-23, `adventure/e08-t44b`)

Baseline: `master` at 1d9cda5 (T44a merged, #410); docs only, the engine code unchanged since
T42's full verify (the light gates). **T44b whole** (decision 101, the user's instruction), one
session and one pull request, as T34b.

**The drafts** (`docs/lanes/adventure/adr-drafts/*-increment-6.md`): ADR-0013 points 1, 3, 14
(worn artifacts on the original's positions, the locks counted per slot; the effect kinds
`meeting_spells` and `gives_spell_book`; moves, combinations by the edition, Scholar as
`h3-heroes`' rules); ADR-0016 points 3, 4, 6 (the army commands with the last stack's rules and
full-width counts, the tactics flag, the meeting's visit, the join without room reported, the
owner's visibility); ADR-0017 points 1, 6, 8, 9, 11 (the management command checked in every
context first, the edition as a static setup input, the meeting and the army dialog as
decisions closed by `Decline`, the encoding to the AI lane with the replay model kept, the
versions, the tactics flag in battle as both heroes' Tactics at 0).

**The design note** `E08-design.md` rewritten for increment 6: part A (§1 the data, §2 positions
and the import, §3 moves, §4 combinations and Titan's book, §5 Scholar, §6 ids and needs), part B
(§7 the army commands, §8 the meeting, §9 the army dialog, §10 the commands, `legal_commands`,
events, observation and the edition, §11 the tactics flag, §12 the env's move and the gates, §13
versions, §14 batch E6, §15 the cut and the exit criteria, §16 risks).

**Findings of the design:** (1) the engine's split keeping both stacks ([1, t − 1]) with the move
reproduces the original's dialog's outcomes, so A-16's extra bound needs no rule (decision 98's
"the original's bounds" holds); (2) the positions' order: within a slot the original's numbering,
across slots the war machines and the book sit after the fifth misc here and before it there,
which no walk tells apart (the capture skips them); (3) zeroing only the flagged hero's Tactics
would hand a lower side the phase, so both go to 0 (decision 96's condition); (4) `Decline`
closes the two new decisions: the env's v4 mask needs only to allow it, no new entry.

**The cut** (§15): five rounds, batch E6 first as increment 5's (T45 shared groundwork + T46 the
batch's spec; T47 the harness and the run + T48 the scoring; T49 `h3-heroes`; T50 `h3-adventure`
+ T51 `h3-game`; T52 the env's masks, replays v6, fixtures, plans, properties, benchmarks + T53
the audit). T45 and T46 queued. The AI lane's inbox told of the paths.

**The checkpoint** (ADV-Q25, asked interactively; decisions 102–105): the drafts and the design
accepted; the encoding's paths (`crates/h3-sim/src/adventure/` to the AI lane but `replay.rs`);
the expansions as the game's configuration, not the map's edition (the user's own answer: the
drafts' and the design's edition clauses rewritten: the packs the ruleset is compiled from
decide, no setup field); batch E6 first. Round 1 (T45, T46) is ready.

### Increment 6, round 1: T45, the shared groundwork (2026-10-23, `shared/e08-t45`, `cross-lane`)

**The cut** (decision 106; asked at the start, `AskUserQuestion`): T45 whole, in one session and
one `shared/` pull request (`cross-lane`); T46 in the next session. Recommended was T45 in two
pull requests (the ADRs and the move, then the kinds and packs); not chosen either: T45a alone,
T45a with T46. The summary the user pasted at the start was T36a's (#383), older than `master`;
the repository's HANDOFF and queue decided the next task.

**Baseline:** `master` 5c09687 (#415), engine code 33e32dfd verified on 7ab66ede (the sandbox ran
only the light gates). The user's PC was linked at the start: the repository from GitHub, the
original data from the PC (the executable, both LODs, `maps.tar`, `extracted.tar`). CI cannot run
(billing): the latest runs on `master` failed unstarted.

- **The ADRs** (the three accepted drafts applied, then deleted): ADR-0013 point 1 (the worn
  artifacts by position, the locks per slot, the walk's order), point 3 (the two kinds), point 14
  added (moves, combinations, Scholar, the hero changes); ADR-0016 point 3 (the army commands, the
  formation and the tactics flag), point 4 (an own hero met, a join without room offered and held
  as the army dialog, with the increment-3 sentences they replace pointed to it), point 6 (what
  the owner alone sees); ADR-0017 point 1 (the management commands, the expansions as the game's
  configuration), point 6 (the meeting and the army dialog), point 8 (the env's opening reworded
  for the move, the encoding to the AI lane, the masks' `Decline`), point 9 (the versions), point
  11 (the tactics flag zeroing both heroes' Tactics); each ADR's consequences, alternatives and
  history; the index's three Read-when cells.
- **The encoding's move** (decisions 99, 103): `lanes.toml` gives `crates/h3-sim/src/adventure/`
  to the AI lane (its `crates/h3-sim/`) and keeps `replay.rs` for the adventure lane by a file
  pattern; `xtask`'s ownership test follows (`replay.rs` the adventure's, `env.rs` and `tests/`
  the AI lane's). The versions table splits the row (the replay format the adventure lane's, the
  action and observation encodings the AI lane's); the global HANDOFF's adventure row (kept under
  its 6 000 bytes by two abbreviations); `docs/07`'s env line; the AI lane's inbox (its two notes
  of T44a and T44b folded into one: what moved, what stayed, the mask check due when T51 lands,
  likely a test only since `legal_mask` builds the answers from `legal_commands`).
- **`h3-content`** (part A §1): `HeroEffect::MeetingSpells { level }` (1..=5) and `GivesSpellBook`,
  codes 51 and 52, with `NAMES`, `try_map`, the parameter check and the hash (the book with the
  kinds without parameters); **core ruleset hash v48** (the golden base hash moved).
- **`h3-pack`**: **pack format v47** (`gives_spell_book` a struct variant `{}`, so an unknown key is
  a syntax error, a trap of the global HANDOFF); a syntax test
  (`the_meeting_and_spell_book_kinds_parse`) and a harvested test
  (`the_harvested_scholar_and_titans_thunder`: Scholar 2, 3, 4 with and without the expansions,
  Titan's Thunder's book with SoD).
- **The harvester** (battle-owned, `cross-lane`): the gap-fill's `[[heroes.skill_effects]]` for
  Scholar, `gives_spell_book` beside Titan's Thunder's spell, the Thunder out of the `pending`
  list (its entry now "spell points per day" for the four mana artifacts); six sources
  (`exe-scholar` 0x4A2940, `exe-titans-thunder` 0x4E2A00, VCMI 1.7.5's `config/skills.json`
  `LEARN_MEETING_SPELL_LIMIT` 2, 3, 4 and `CGameHandler.cpp`'s `useScholarSkill` and
  `checkSpellbookIsNeeded`, re-read for this in a sparse checkout, and the survey's two wiki
  pages). `base` and `sod` re-harvested from the staged installation: only those lines changed;
  the harvest check matches. `synthetic-ext`'s `wayfarer_sash` uses both kinds (`h3-pack-battle`'s
  every-kind check; no new artifact, so its artifact count stays). `packs/README.md`'s effect table.
- **Replays and records**: the 20 golden replays keep commands and state hashes; only
  `ruleset_hash` in their headers moved (synthetic faithful `0x3156c56f2211ba02`, base faithful
  `0x1ac6c50d158057de`, strict-fixed `0xc01284bbdbb004c1`); `duel-s2-d7` and `duel-s5-strict-d7`
  re-recorded with their options differed in that line only. The AI lane's `bots-report-v1.json`
  regenerated: only `engine.core` moved.
- **Docs**: two code comments repointed to increment 5's design note in this log (the design
  note's request); the design note's §1, §6, §12, §13 and §15 cut to what later tasks need;
  decision 106 and the task's line in the epic; the queue (T45 out, an inbox note for T49 and
  T51); the battle lane's inbox and its HANDOFF's shared versions; `docs/03`'s glossary (the
  tactics flag, positions from T49, the meeting and the army dialog).
- **Verify**: the full run with the original data on 00497ba (engine code b28276c9) passed all 16
  steps (benchmarks within +0.44 %, the harvest check matching the installation); a first run was
  stopped when two comments changed the code. `master` (#416, docs only) merged in after it, its
  adventure inbox note beside T45's; `check-lane`, `check-docs`, `check-exe-map` and `check-rules`
  passed again. CI cannot run (billing).

### Increment 6, round 1: T46, batch E6's spec and results template; round 1 closed (2026-10-23, `adventure/e08-t46`, `cross-lane`)

**The cut** (decision 107; asked at the start, `AskUserQuestion`): T46 whole on one map. Sizing
found the expected cut by rows unneeded: `h3-experiment` already writes two heroes of a player
side by side (`also` with `at`), learned spells with Scholar, the backpack in order, a 16-bit
count of 40 000, a compliant group beside a full army and worn parts, and one map holds the
eleven rows in 16 heroes, the original's eight per player. Not chosen: two maps (E6a, E6b), the
cut by rows anyway.

**Baseline:** `master` f3c8fab (#431), engine code fe96ce40 verified on f3c8fab (the sandbox ran
only the light gates). The session had **no GitHub access** to the repository (the proxy refused
it: "not enabled for this session"): the clone came from the user's PC as a bundle of `master`
(`original-data/sync/homm3-master.bundle`, the same commit as the PC's `master`), the original
data from the PC (the executable, both LODs, `Objects.txt`, `maps.tar`, `extracted.tar`).

- **The spec** `experiments/specs/E08-batch-6.toml` (36 x 36 grass, Easy, hot seat; no compiler
  change): Red plays E6-1 and E6-2 (seven stacks beside 10 compliant Wolf Riders: OK without a
  change; a stack dismissed in the dialog and the group taken into the freed slot), E6-3 (a last
  stack's move and merge refused, the swap allowed), E6-4 (the split into a one-stack hero's empty
  slot at the slider's maximum: A-16's 9 and 1 against VCMI's 10 and 0), E6-11 (a meeting's
  price: 1560 to 1460, the visitor staying); Blue plays E6-6a and E6-6b (Advanced Tactics against
  5 Pikemen with the flag toggled off, then left on), E6-7, E6-8, E6-10 with E6-9, and E6-5 last
  (after save 3). Each row's settles, the reading and the other sources head the file.
- **Where the plan changed**: **E6-5** as survey §9 wrote it (a move to an empty slot in the hero
  screen giving -25 536) contradicted §3.5's roles; re-read in T46 (`army_swap_slots` 0x44AD60:
  slot b receives a's count through 16 bits; the hero screen 0x4DDB53 passes a = the clicked
  slot, b = the selected; the exchange 0x5B0ED5 a = the source, b = the target), the hero screen's
  move keeps 40 000 and the exchange's move and the screen's swap onto a clicked big stack wrap.
  So the row moves 40 000 Peasants in the hero screen (the control), then to E6-5 B's empty slot
  in the exchange, and swaps a Pikeman onto 40 000 Gnolls (both -25 536 by the reading; VCMI 40
  000); the survey's §9 row carries the correction. **E6-7**: Expert Scholar and Expert Wisdom
  (Precision, Fireball, Prayer) meets no skills (Bless, Chain Lightning, Implosion), so both
  sides of the limit show: Bless and Chain Lightning learned, Implosion not (Scholar's side);
  Precision learned, Fireball and Prayer not (Wisdom's). **E6-8**: Wizard's Well's three misc
  parts with misc 1 (9) free, the Orb equipped on misc 4 (12) and right-clicked there: the
  reading's first position (9) against VCMI's clicked one (12); the question after the first
  equip only (VCMI: after both); then the Well right-clicked in the backpack (D-28, in exit
  criterion 7 but not in §9's plan: the reading shows its window only, VCMI asks to disassemble).
  **E6-9 and E6-10** share one book-less hero with six backpack
  artifacts (the Thunder third, visible after a rotation either way): the arrow first, then the
  Thunder on and off, which freed Blue's eighth hero for E6-5's second.
- **The template** `experiments/results/E08-batch-6.md` (stamp ddd4ee4e). Its instructions name
  the memory reads T47's recipe needs (army +0x91/+0xAD, worn +0x12D, backpack +0x1D4/+0x3D4,
  spells +0x3EA, flags +0x48, the battle's +0x13D68), the positions' numbers, gold as both
  players' starting bonus where the options allow (else an artifact the spec does not give goes
  under "anything odd"), and the order with three saves.
- **Checked in the engine**: the map reads, imports whole and validates (`original_data` on a
  folder holding only it: 16 heroes, 4 monsters, a backpack of 6, 8 heroes per player).
- **Not built**: the harness's clicks and reads and the recipe, the run (T47); the scoring (T48).
- **Verify**: no fingerprinted path changed (docs and experiments only), so
  `scripts/verify-once.sh engine --base origin/master -- --with-original-data` ran the light
  gates on efcbb36 (engine code fe96ce40, verified on `master` f3c8fab): check-rules,
  check-stubs, check-docs, check-exe-map passed; `check-lane --cross-lane` passed (the battle
  lane's `docs/experiments/README.md`). CI cannot run (billing).

### Increment 6, round 2: T47a, the harness's clicks and reads for batch E6 (2026-10-23, `adventure/e08-t47a`)

**The cut** (decision 108; asked at the start, `AskUserQuestion`): T47 in two, as T37: the clicks
and reads with the harness check first, the recipe and the run (T47b) next. Not chosen: three
parts (the recommendation: the reads and the hero screen, then the dialogs, then the run), T47
whole.

**Baseline:** `master` e3cfe7c; the session had GitHub access again (cloned from GitHub). Its
engine code (9a54eabd) had no run on record, so the sandbox's baseline ran the full verify with
the original data: all 16 steps passed (code 9a54eabd verified on fbbf53b). The user's PC was linked: the original's sixteen files
staged from it as T37a's. The branch changes no engine code (the harness, a common path since
CLT-Q18: no `cross-lane`; the client and map-client lanes' inboxes told).

- **Screens and items from memory, not coordinates.** The window manager's topmost dialog
  (`h3mem.dialog`: its vtable and box, H3API's H3BaseDlg +0x18) tells the screen by its vtable
  (`h3adv.SCREEN_KINDS`, observed under Wine: the map, the battle, the hero screen, the exchange,
  the army, creature and split dialogs, a message). Each item holds its box relative to the dialog (i16 x +0x18, y +0x1A,
  u16 width +0x1C, height +0x1E) and its state (+0x16, bit 0x4 while drawn), so a click goes to
  the centre of the item with the procedure's id (survey `E08-army.md`, all as read: the hero
  screen's army slots 68-74, worn positions 2 + p, backpack places 40-44, arrows 77 and 78,
  loose 122, tight 124, tactics 126, split 127, close 30720; the exchange's armies 13-19 and
  20-26, counts shown 65-78, split buttons 103 and 104; the army dialog's upper 115-121, lower
  140-146, counts shown 108-114 and 133-139, OK 30722; the creature dialog's dismiss 30723; the
  split dialog's slider 6, counts 4 and 5, OK 30722; a question's yes 30725 and no 30726; the
  battle's Start Combat 30722). T37a's fixed places of the worn slots lie within a pixel of
  their items' centres.
- **The recipe's language** (`clients/capture/README.md`, "Screens and their items"): a step's
  `screen = "hero"`, and `actions` played in the hero screen or in the exchange or army dialog a
  walk opens: `click = [items…]`, `right = "<item>"` (held while read: a question stays and is
  answered, a window goes with the release), `slider = "max"`, `answer`, and `record`/`note`
  read right after each action; a worn position or a backpack place may be named by its
  artifact. A screen left open is closed (no change). A tactics phase is ended at once in
  `fight` (its message, GENRLTXT 373, then Start).
- **The reads:** a hero's slots as memory holds them (signed counts, a count without a creature
  shown), worn positions by number, flags (+0x48), learned spells (+0x3EA, names from the spell
  table +0x10), the counts a screen shows, the hero screen's five backpack places, the split
  dialog's counts, the topmost dialog's record (a title), what a right click opened, the battle's
  tactics phase (+0x13D68 at its start); the held artifact (0x698A88, a library read).
- **The harness check** gained Kit (the hero screen: a swap, the split at the slider's maximum,
  a dismissal, both formation buttons and the tactics formation twice, the Orb put on misc 4 and
  the assembly declined, the right click's assembly, the Well into the backpack and right-clicked
  there, the right arrow; then a battle whose tactics phase is ended) and Full (seven stacks
  joined by 5 compliant Wolf Riders: the army dialog's title, a dismissal in it, the group into
  the freed slot; then the exchange with Full B: a move, a split, closing). Blue holds eight
  heroes. **Passed under Wine in two full runs** (about four minutes each; every record of Kit and Full
  the same in both, the runs differing only in the random starting gold, seed bytes and their
  draws): every row as the reading, no
  oddity (Kit: flags 2, 0, 2, 3, 2; the assembly's question GENRLTXT 733 after the equip and on
  the right click, the Well at position 9; the backpack's right click a window, no question; the
  arrow's rotation; the tactics phase opened and ended. Full: GENRLTXT 36, the dismissal's
  GENRLTXT 13, the group in slot 3).
- **Seen while exploring on batch E6's map** (by hand, not a run; for T47b and T48): **E6-5's
  stacks of 40 000 load as empty slots**: each slot reads creature -1 with the count -25 536 (the
  map's 16-bit count sign-extended, the creature dropped), so the hero screen shows only the
  Pikeman and the row cannot run as specified. T47b's spec change (proposed): two stacks of
  20 000 of each creature merged in the hero screen first (a merge adds 32-bit counts, survey
  §3.5), then the row's three moves. The exchange's split into a one-stack hero's empty slot
  at the slider's maximum showed 0 : 10 and gave 1 and 9 (A-16 as read); the Scholar meeting,
  the army dialog, the tactics flag and the Well's assembly as the spec's predictions. Not
  recorded as results: batch E6 is T47b's run.
- **Not built:** batch E6's recipe and its run (T47b); the scoring (T48).
- **Verify:** the baseline's full run (above); the branch changes no fingerprinted path, so the
  light gates ran on it (check-rules, check-stubs, check-docs, check-exe-map) with `check-lane`
  (an `adventure` change: no `cross-lane`): all passed. CI's recent runs fail unstarted.

### Increment 6, round 2: T47b1, batch E6's recipe; E6-5 dropped (2026-10-23, `adventure/e08-t47b1`, `cross-lane`)

**The start's two questions** (`AskUserQuestion`): E6-5's setup, after T47a found that the map's
count of 40 000 loads as an empty slot. Recommended was two stacks of 20 000 of each creature
merged in the hero screen first, then the row's three moves, with one untouched stack of 40 000
on E6-5 B recording the load (the engine's import reads the count as 40 000). The user's own
answer (decision 109): the original's behaviour here does not matter, its oddity being a
limitation or a bug; the engine supports a reasonable count and handles saturation gracefully,
a merge past the cap refused. So E6-5 left the batch, and the cap waits for T50's start (the
queue's inbox; `E08-design.md` §7 and §16). The cut (decision 110): T47b in two, the recipe now,
the run next (recommended was T47b whole).

**Baseline:** `master` 8ac0949 (#437 merged). GitHub reached through the session's repository
grant (the token of an earlier attempt was refused). The sandbox's baseline ran the full verify
with the original data on engine code 9a54eabd: all 16 steps passed (h3-map's step skipped the shipped maps, not staged; code 9a54eabd verified on 8ac0949). The edits were made in a `git
worktree` meanwhile, so the baseline tested `master`'s files.

- **The spec** (`experiments/specs/E08-batch-6.toml`): E6-5 and its two heroes removed (14 heroes:
  Red's eight, Blue's six; the other rows' places unchanged, encounter i at column 3 + 6 (i mod
  6), row 3 + 5 (i div 6)); its header, the map's description and the order without it. Four
  fields renamed so that one read fills each: E6-1's and E6-2's "Wolf Riders still on the map
  (yes/no)" became "Wolf Riders on (4, 3) after (count, or none)" (and (10, 3)), the tile's
  monster as memory holds it; E6-8's "worn and backpack at the start" and "… at the end" became
  "worn at the … (positions)" and "backpack at the …". Compiled again: stamp e17cb77c (was ddd4ee4e); the map reads, imports whole and validates in the engine (`original_data` on a folder holding only it: 14 heroes, 8 per player at most, 4 monsters, a backpack of 6).
- **The recipe** (`clients/capture/batches/E08-batch-6.toml`, 18 steps): each row's procedure as
  T47a's steps and actions. Red: E6-1 (the gold and slots, the join answered yes, the army
  dialog's title, OK), E6-2 (the Swordsmen dismissed in the dialog, the group into slot 3), E6-3
  (the three clicks in the exchange, each cell of the table read after its click: a message, both
  armies), E6-4 (the split from the left army into the right's slot 1, the slider at its maximum,
  OK), E6-11 (the points before and after the meeting, both tiles); Blue: E6-6a (the flags, the
  tactics button, the battle by auto combat with the tactics phase read), E6-6b (the control),
  E6-7 (the spells, Scholar's message from the walk's messages), E6-8 (the Orb on position 12
  twice answered no, the right click answered yes, the Well into the backpack and right-clicked
  there), E6-10 (the right arrow; the screen opened again for E6-9: the Thunder on the right hand
  and back into the backpack's first place). Fields left to a player by hand: E6-8's "on screen:
  the Well's slot and the locked slots" (no read of the screen's locks).
- **A code reading for the recipe** (`docs/exe/map.toml`: 0x5AE5B0 `swap_selection_reset`):
  `swap_army_click` calls it after every second click, a refused move included, and it clears
  the selection (+0x48 to +0x58), so E6-3's second and third clicks start from none and do not
  open the creature dialog.
- **The check with `fill`**: every record key of the recipe (61) is a field or table cell
  of the compiled template (h3adv's `fill` raises on any other); against the template of before
  only the six renamed fields failed. Left empty: E6-8's screen line only.
- **Not built:** the run (T47b2); the scoring (T48); the cap (T50's start).
- **Verify:** the branch changes no fingerprinted path (docs, experiments and the harness's
  recipe), so the light gates ran on it (check-rules, check-stubs, check-docs, check-exe-map):
  all passed. `check-lane` names one battle path, `docs/experiments/README.md`'s status row:
  `cross-lane` with the user's consent. CI's recent runs fail unstarted.

### Increment 6, round 2: T47b2, batch E6's run (2026-10-23, `adventure/e08-t47b2`, `cross-lane`)

**Baseline:** `master` 0fd0383 (#440 merged). Its engine code (e2785835) was verified on 17dd372,
so the sandbox's baseline ran only the light gates: all passed. The user's PC was linked: the
original's files staged from it as T47a's, `clients/capture/setup.sh` run.

- **The run** (`h3adv.py batch capture/batches/E08-batch-6.toml`, about three minutes): three
  runs, the 59 filled records the same in all three (`compare`). No step stopped and no screen
  differed from the recipe's plan. The only recipe fix: E6-7's walk expects its dialog
  (`expect = { dialogs = true }`), since run 1 noted Scholar's message under "anything odd".
  `experiments/results/E08-batch-6.md` is run 3's filled template.
- **Two lines filled by the agent** from run 3, through a wrapper kept in the sandbox (the
  harness unchanged; no text or picture committed). E6-7's message, which `h3lod` names `?`, is
  composed of GENRLTXT 140, 141, 142, 143, **148 and 149** (survey §2.2 names 140 to 143 only: for
  T48) with the names, and names the spells the memory reads show. E6-8's screen line is from a
  dump at the read after the assembly: the Well on misc 1 (9), **locks on misc 4 and misc 5 (12,
  18)**, misc 2 and 3 empty (the locks drawn from a count, not on the parts' old positions).
  E6-9's dumps: the Spell Book shows with the Thunder on and after it is off; locks on the head,
  torso and left hand while it is worn.
- **A first look** (the scoring is T48): every row as the spec's reading. E6-1 ADVEVENT 86, the
  dialog GENRLTXT 36, the army and gold unchanged, the group gone; E6-2 the dismissal's GENRLTXT
  13, the group in slot 3; E6-3 refused, refused, swapped, no message; E6-4 10 : 0, then 0 : 10,
  1 and 9 Pikemen (A-16 as read, not VCMI's); E6-11 1 560 to 1 460 points, both heroes on their
  tiles; E6-6 flags 2 then 0 and no phase, the control 2 with a phase, both won; E6-7 learns
  Bless and Chain Lightning (not Implosion), E6-7 B Precision only; E6-8 GENRLTXT 733 after the
  first equip only and on the right click, the Well at 9 (not VCMI's 12), its right click in the
  backpack a window; E6-10 the right arrow moves the first to the end; E6-9 the book (17) after
  both.
- **Not built:** the scoring (T48); the cap (T50's start).
- **Verify:** the branch changes no fingerprinted path (the recipe, the results, docs).
  `verify --scope adventure` passed every step but `check-lane`, which names one battle path,
  `docs/experiments/README.md`'s status row: `cross-lane` with the user's consent (asked at the
  end). CI's recent runs fail unstarted.

### Increment 6, round 2: T48, batch E6 scored; round 2 closed (2026-10-23, `adventure/e08-t48`, `cross-lane`)

**Baseline:** `master` b8c5172 (#443 merged); its engine code (c4721d3e) was verified on 3a21abe
(#441's record), so the sandbox ran only the light gates: all passed. T48 whole (the user: "Go
on"; a scoring of docs only).

- **Batch E6 scored** (survey `E08-army.md` §13, from T47b2's results): every row as the code
  reading. Conclusive against VCMI: **E6-4** (A-16, D-27: the split's maximum leaves 1 Pikeman and
  gives 9; VCMI 10 whole), **E6-8** (§5.3, D-28: GENRLTXT 733 after the first equip only and on
  the right click, the Well on position 9 with 10 to 12 empty; VCMI asks again, puts it on 12 and
  locks 10 and 11; in the backpack only the artifact's window, VCMI a disassembly question),
  **E6-10** (the right arrow rotates the array; VCMI only scrolls a view). Confirmed with every
  source: E6-1, E6-2, E6-3, E6-6, E6-7, E6-9; E6-11 as the reading (the spec named no other).
  No ADV-Q: no row contradicts the reading.
- **Survey corrections** (presentation): Scholar's message is composed of GENRLTXT 140 to 143,
  148 and 149 (§2.2 named 140 to 143); the hero screen draws a combination's locks apart from
  the positions (the Well's two on misc 4 and 5, §5.3); §8's D-27, D-28 and A-16 cite their rows.
- **The catalog:** HER-ART-001 (the locks) `verified` with E6-8's and E6-9's observation. The
  rules the other rows settle (§10's ids: ADV-ARM-*, GAM-MET-001, HER-ART-005 to -007,
  HER-SPL-007, GAM-BAT-009, ADV-MON-008's amendment) cite them when T49 to T51 write them;
  A-16's ledger entry (decision 98, no flag) takes E6-4.
- **The next round's cut** (decision 111, asked at the end): T49 in two, T49a the positions
  (design §2), T49b the moves, combinations and Scholar (§3 to §5); recommended so. Not chosen:
  three parts, T49 whole.
- **Docs:** `E08-design.md` §14 to §16; the experiments README's status row (**evaluated**;
  `cross-lane` with the user's consent, battle's path); the queue: T49a and T49b (round 3).
- **Verify:** docs only (no fingerprinted path). `verify --scope adventure` (below, in the pull
  request).

### Increment 6, round 3: T49a, the artifact positions (2026-10-24, `adventure/e08-t49a`, `cross-lane`)

**Baseline:** `master` a5fbe80 (#446 merged); its engine code (c4721d3e) was verified on 3a21abe,
so the sandbox ran only the light gates: all passed. T49a whole (the user's choice at its start,
`AskUserQuestion`: recommended was two parts, the model with every caller first and the import
v9 with `PlaceArtifact` after; not chosen either: a cross-lane constructor first).

- **The model** (`h3_heroes`, design §2, ADR-0013 point 1): `HeroDetails.worn` is
  `Vec<Option<ArtifactInstance>>`, one entry per position of `positions(ruleset)` (the slots in
  their order, each `count` times; SoD's 19, the test packs' 9 and 4); `Position(usize)`.
  `validate` checks the length (`DetailsError::Positions`), an artifact's slot against its
  position's (`WrongPosition`, `NotWearable` for the Grail) and the locks per slot (`SlotFull`,
  counted as before: `positions::full_slot`, shared with the first position and a placement).
- **The first position** (`first_position`, HER-CHG-002): the lowest empty position of the
  artifact's slot when the counts fit with it; `AddArtifact` wears it there, else the backpack's
  end as before. `wearing(ruleset, list)` places a list at first positions, one that finds none
  left out (the callers that list artifacts; a caller needing an invalid placement writes
  positions). The capture (HER-ART-004) and a lost machine (HER-WAR-002) walk the positions;
  `RemoveArtifact` at a position leaves it empty; `PlaceArtifact { artifact, at }` puts one at an
  empty position of its slot with the locks fitting, or at a backpack index up to the length
  while the backpack carries it and has room (`ChangeError::CannotPlace`); a battle never tells
  one (the plans' property).
- **The game** (`h3-game`): `HeroArtifactsSetup.worn` by position; the defaults' book and the
  starting army's machines at their first positions; the details' hash writes each position (0
  empty, 1 and the artifact): **game state hash v12** (T51's decisions become v13, design §13).
  `h3_game` re-exports `positions`, `takes`, `wearing`, `Position` (`h3-map` has no
  `h3-heroes` dependency, `arch.toml`).
- **The import v9** (`h3-map`): the map's worn entry `i` is `codes::WORN`'s slot and rank, placed
  at that rank of the slot's content positions (the map's 13 to 17 to 14 to 18, SoD's 18, the
  fifth misc, to 13), entry by entry in the map's order and only where the position takes it
  (`h3_heroes::takes`), else left out. Found by the full verify's shipped maps: the Tutorial's
  second hero stores the Shackles of War (125, misc) on its left hand, which the first version
  (each entry placed as stored) refused to validate. The original's loader drops such an entry
  (`placed_hero_setup` equips entry by entry through `hero_equip_artifact`, which returns
  without placing what `hero_can_place_artifact` refuses; 0x4D8D63, read this session) and
  VCMI's skips it (`loadArtifactToSlot`, its comment naming a campaign hero's Shackles on a left
  hand): two sources agreeing, built (HER-ART-001's sources). With RoE's rules an artifact on the
  fifth misc is dropped too (the original refuses position 18 before SoD). The backpack is as
  before (the original's 0x4E2F90 also refuses a war machine there, which validation refuses
  here; no shipped map has one). `MAP_IMPORT_VERSION` 9; `h3m.md`'s import paragraph. The plans put a hero's Spell
  Book on the map's book entry (17), until now the head's (0, "any slot"): the fixture maps
  regenerated (`H3_WRITE_FIXTURES`), both changed only there.
- **The other lanes' call sites** (`cross-lane`, ADR-0013's consequences): the battle lane's
  `h3-pack-battle` tests (`src/tests.rs`, eleven `tests/harvested_*.rs`: `wearing`, the random
  details through `first_position`; the Alliance's locked head and AB's four misc positions
  restated by position), the AI lane's `h3-sim` encounter (`wearing` before its `AddArtifact`s).
  The client lane's `h3-view` and the battle lane's `h3-experiment` build maps through the
  writer's 19-entry array, not details: nothing to change (the design note named them). Their
  inboxes told.
- **Replays:** the ten re-recorded with their options (`adventure_replays.rs`' history), their
  commands unchanged, every checkpoint moved (the version, the map's hash, the positions).
- **Catalog:** HER-ART-001 amended (positions; batch E6's source kept, `verified`), HER-CHG-002,
  HER-ART-004, HER-WAR-001, HER-WAR-002 restated by position. Design note §2 reduced to a pointer
  here, §13's versions and §16's ripple risk updated.

### Increment 6, round 3: T49b, moves, combinations, Titan's book, Scholar (2026-10-24, `adventure/e08-t49b`)

**Baseline:** `master` d0d7bb8 (#450 merged); its engine code (85f9c044) was verified on d0d7bb8,
so the sandbox ran only the light gates: all passed. T49b whole and the import's Titan's book in it
(decision 112, the user's choices at its start).

- **Titan's book** (HER-ART-007, design §3): `h3_heroes::equip(worn, ruleset, item, at)` wears an
  artifact where `takes` allows it and, for one with `gives_spell_book`, puts the ruleset's first
  artifact with the `spell_book` effect at its first position when one takes it; whether it
  wore it. `apply_change` equips every worn placement (`AddArtifact` at its first position,
  `PlaceArtifact` at a position), so a capture, a move and an assembly give the book; into the
  backpack nothing; a removal takes nothing back. `wearing` still only places (no book): its
  callers list artifacts. The original tests the artifact's id (135) at 0x4E2A64; the engine reads
  the effect of the artifact itself (a combination's parts' effects do not equip).
- **Moves** (HER-ART-005): `ArtifactMove { from, to }` (two `ArtifactPlace`s), `check_move(from,
  to: Option<&HeroDetails>, ruleset, move)` (`None`: within one hero) and `move_changes` →
  `MoveChanges { from, to }` (within one hero all in `from`): the removal, the target's removal,
  the placement, the target's artifact back at the source's place. The order of the checks: the
  source holds an artifact (`NoArtifact`); it moves (`Unmovable`: a slot not carried and no war
  machine or the Catapult's, so the Spell Book and the Catapult; the packs' Catapult places none,
  the test pack's places one with the `catapult` flag); not the same place within one hero
  (`SamePlace`, an engine choice: a pick-up dropped back changes nothing, the env's masks need no
  no-op); a position target of the artifact's slot (`CannotPlace`; a position past the last, which
  the property found taking the Grail as `None == None`, refused), the target's artifact movable
  (`Unmovable`), the locks with the target's artifact gone (within one hero a worn source's
  artifact gone and the target's back on it: a swap keeps the set); between two heroes the
  moving hero's locks with the returned artifact (`CannotReturn`); a backpack target carried, at an
  index up to the length (without the moved one within one backpack) and below the most. The
  property `a_checked_move_applies_change_by_change`: every move it accepts applies change by
  change (`check_change` passing each), validates and keeps both heroes' artifacts but the book.
- **Combinations** (HER-ART-006): `check_assemble(details, ruleset, at)` → the first combination
  in artifact order listing the part at `at` whose every part is worn on a position;
  `assemble_changes`: the removal at `at`, then for each other part (pack order) the highest
  position of its type not taken, emitted in descending position order, then `AddArtifact` of the
  combination (its first position: the test's Well lands on misc 4 when assembled from misc 5).
  `check_disassemble` / `disassemble_changes`: a worn combination's removal, then `AddArtifact` of
  each part in artifact order (no sort: logic crates have none). `CombineError::{NoArtifact,
  Incomplete, NotCombination}`. A part listed twice would take two copies after a check of one
  (the original's test is per part too); no pack has one.
- **Scholar** (HER-SPL-007): `meeting_spells(a, b, ruleset) -> (Vec, Vec)` in `spells.rs`, the
  spells by the ruleset's order; `wears_book` shared with `learns_spell`. VCMI's
  `useScholarSkill` read this session agrees (the greater level, both books, each limit).
- **The import v10** (`h3-map`, decision 112): each worn entry equipped through `h3_game::equip`
  (re-exported beside `takes`); the original's `placed_hero_setup` equips entry by entry through
  `hero_equip_artifact` (0x4D8D63), which adds the book (0x4E2A64), so the map's own book entry
  then finds the position taken. VCMI's `checkSpellbookIsNeeded` gives the book only on a move to
  the right hand or an assembly; its comment leaves a pickup and the game's start open
  (HER-ART-007's sources). A scratch scan (not committed) of the 160 shipped maps: no hero wears
  or carries the Thunder, so no import changes; the ten golden replays moved only their header's
  `import_version` (every hash the same; `adventure_replays.rs`' history).
- **Catalog:** HER-ART-005, -006, -007 and HER-SPL-007 `verified` (batch E6's E6-10, E6-8, E6-9,
  E6-7; VCMI 1.7.5 read for the three code links); HER-CHG-001, -002 and HER-ART-001 amended.
  Design note §3 to §5 reduced to a pointer here (§3), §13's versions. Coverage of `h3-heroes`
  100 % with and without its proptests.

### Increment 6, round 4: T50, the adventure's army management, the meeting's visit, the join without room (2026-10-24, `shared/e08-t50-cap` #455, `adventure/e08-t50`)

**Baseline:** `master` f89aa89 (#454 merged); its engine code (808b7ac3) was verified on f89aa89, so
the sandbox ran only the light gates: all passed. At its start the user chose T50 whole (decision
113), a cap of 1 000 000 (decision 114) and, for a group its stack cannot take, a free slot, else
the army dialog (decision 115). ADR-0016 point 3 amended first in `shared/e08-t50-cap` (#455,
docs only).

- **The cap** (ADV-ARM-007): `AdventureParamsDef::max_stack_count` (`NonZeroU32`; the pack format's
  `[params] max_stack_count`, base 1 000 000, synthetic 9 999; the compiler refuses a value below
  `MAX_MONSTER_COUNT`, so any monster's group fits a slot; the tests' rulesets use exactly that,
  `testing::MAX_STACK`), adventure ruleset part v11. A setup's hero or guards' stack above it:
  `ArmyError::TooManyCreatures`. `monsters::room` finds the army's first stack of the creature
  that takes the group within the cap (the original's first stack, which takes any), else the
  first free slot; `add_stack` (joins) and `raise` (Necromancy: at most the cap raised, the
  upgrade's stack likewise within it) use it; `has_room` (the join's offer) too.
- **The army commands** (`army.rs`, ADV-ARM-001 to -005): `ArmyRef::{Hero, Group}`, `ArmySlot`,
  `ArmyCommand::{Move, Split, Dismiss}`, `ArmyCommandError`; `plan` works on copies of the one or
  two armies and returns their new stacks (slot order kept by insertion: logic crates do not
  sort). Checks in order: the slots, the source's stack, the same slot (refused, as T49b's no-op
  artifact move), then the rule: a merge past the cap (`TooManyCreatures`), a hero's last stack
  leaving its army (`LastStack`: moved to an empty slot or merged into another army, dismissed),
  a split's other creature or count outside `split_range` (max(1, t − most) to min(t − 1, most)).
  A split that leaves the counts as they were is allowed (the original's OK does the same). The
  group has no protected stack. `state/manage.rs`: `manage_army(part, player, command, group:
  Option<&mut Vec<ArmyStack>>)` finds the heroes (the player's on the map: `NotYourHero`) and the
  group (`NoGroup` when none is given), plans and writes; `split_range` (the query, `None` when no
  split is possible); `set_formation`, `set_tactics` (ADV-ARM-006, any number of times). No
  events: the game tells the owner in T51.
- **The tactics flag**: `HeroOnMap::tactics`, true at the build (the original's 2), in the hash
  after the formation (adventure state hash v11) and `OwnHeroView::tactics`.
- **The meeting's visit** (GAM-MET-001): `check_step` and `passage` treat every other hero alike
  (another player's a battle, an own one a meeting, from a stop; a flight passes both), and the
  walking query's occupancy is one bit (the own-hero block gone); `move_hero` reports
  `AdventureOutcome::meeting` (`Meeting { visitor, visited }`). The game ignores it until T51.
- **The join without room**: `join` reports an accepted group that finds no room as
  `AdventureOutcome::no_room` (`NoRoom { hero, creature, count }`), its treasure given and the
  monster removed. **Moved to T51** (an agent's call, told in the summary): dropping the offer's
  `has_room` skip, so that no build between the two loses a group the game cannot hold yet
  (`ADV-STUB-2` updated).
- **Rules and ledger:** ADV-ARM-001 to -007 and GAM-MET-001 written (ADV-ARM-002, -003, -004,
  -006 `verified` by batch E6's E6-3, E6-4, E6-2, E6-6; GAM-MET-001 `implemented`, its decision in
  T51); ADV-MON-007, -008 and GAM-BAT-006 amended; ADV-QUIRK-19 (A-15, exploit-fix) and -20 (A-16,
  the original's bounds, no flag) in the ledger; `h3m.md`'s import: a count kept whole.
- **Tests:** `army/tests.rs` (the plan within one army, between two, with the group, the split's
  range, the errors, the state's command and query, the formation and flag; the property
  `commands_keep_the_armies_whole`: random commands keep every invariant and each creature's
  total, and a move that is no merge undone by the move back restores both armies); the room and
  the cap in the join's and the raise's tests; the meeting in the walk's, the flight's and the
  query's tests and properties; `h3-map`'s step property counts an own hero's visit.
- **Replays:** the ten re-recorded with their options, their commands unchanged (no game of the
  duel meets an own hero: each player's two heroes start on different levels); every hash moved.
- **Found and fixed:** the epic held decision 112's block five times (T49b's edit had put it
  before every increment's task list); the four extra copies removed.

### Increment 6, round 4: T51a, the game's management commands on one hero, their listing, the tactics flag in battle (2026-10-24, `shared/e08-t51a-legal`, `adventure/e08-t51a`)

**Baseline:** `master` e816f20 (#458 merged); its engine code (31a2e374) not verified yet, so the
sandbox ran the full verify with the original data first: every step passed. At its start the user
cut T51 in two (decision 116: T51a here, T51b the meeting and the army dialog) and consented to
`h3-sim`'s one arm (`cross-lane`). Reading the env's rollout before the build showed that
ADR-0017 point 6's flat list would change it: the user chose the listing per hero (decision 117),
ADR-0017 point 6 amended in `shared/e08-t51a-legal` (docs only, with `docs/06-security.md`'s line).

- **The command** (GAM-MAN-001): `GameCommand::Manage(Manage)`, `Manage::{Army(ArmyCommand),
  Formation, Tactics, Artifact { from: (HeroId, ArtifactPlace), to: (HeroId, ArtifactPlace) },
  Assemble, Disassemble}` (`manage.rs`), `GameCommandError::Manage(ManageError::{NotYourHero,
  NotTogether, Army, Artifact, Combine})`; the adventure's `NotYourHero` named as the game's
  (`ManageError::of_army`). `state/manage.rs`: `check_ready` (turn, decision, a level-up's hold),
  then `alone` (an army command on one hero's army: the group alone is `NoGroup`, two armies
  `NotTogether` until T51b), `manage_army` with no group, `set_formation`, `set_tactics`; an
  artifact command checks both heroes are the player's on the map (`own_details`), then
  `NotTogether` for two, then `move_changes` (its `from` changes), `assemble_changes`,
  `disassemble_changes`, applied in order through the battle's `change_hero` (now `pub(super)`).
- **Events**, the owner's alone (`project`): `StackChanged { player, at, stack }` for each slot an
  army command names, the stack's first, with what it holds after it (an agent's call against the
  design's `ArmyChanged` with the whole army: `GameEvent` stays `Copy`); `FormationSet`,
  `TacticsSet`; `HeroChanged` per artifact change. `ArmyCommand::slots` gives the slots;
  `ArmyCommand::armies` is public now.
- **The listing** (GAM-MAN-002; decision 117): `legal_management(state, rules, actor, hero)`: the
  army's commands (`AdventureState::army_commands(part, player, (from, to), group)`: each stack's
  moves onto the other slots of `from` and, with `to`, of `to`, then the dismissals, each checked
  by the plan without applying it, `plan_army`, which `manage_army` now shares), both formations,
  both flag values, each artifact's moves onto every place `check_move` takes (sources and targets
  by position, then by backpack index up to its length), the assemblies and disassemblies by
  position; empty for the system actor, outside the turn or with a decision open, another
  player's hero, a hero not on the map. `split_range(state, rules, actor, (from, to))`: one army's
  split counts (`None` for two armies). `legal_commands` unchanged.
- **The tactics flag in battle** (GAM-BAT-009): `Combatant::tactics` (built in one place,
  `HeroOnMap::combatant`, for the three battle starts), `deny_tactics` in `start_battle`: the side
  with the greater level's flag off zeroes both heroes' Tactics; equal levels cancel as before;
  a side without a hero counts as on.
- **The CLI and the replays**: `h3-cli`'s command text gains the management commands (`army
  move|split|dismiss <army>:<slot> …`, `formation`, `tactics`, `artifact <hero>:w<p>|b<i> …`,
  `assemble`, `disassemble`), adventure replay format v6 (moved from T52: the text's match is
  exhaustive); the ten replays' headers only (`format = 6`), their commands and hashes unchanged
  (no replay manages; every flag stays on). `h3-sim`'s `tileless_entry` maps `Manage` to no entry
  (the AI lane's inbox told); `h3-map`'s tests' matches gained the variant and the events.
- **Rules**: GAM-MAN-001 and -002 `implemented`, GAM-BAT-009 `verified` (batch E6's E6-6).
  Versions: game state hash, adventure state hash, ruleset parts and map import unchanged (the
  flag and the armies were hashed since T50).
- **Tests**: `h3-game`'s `tests/manage.rs` (the army commands and their refusals, the formation
  and flag with their projection, the artifacts' moves, assembly and disassembly against
  `h3-heroes`' changes, the listing against every candidate `apply` accepts, the split's query,
  six battles of the flag), `h3-adventure`'s listing against the state's command (within, toward
  another hero, the last stack, the group), the battle start's flag, the CLI's round trip and
  refusals.

### Increment 6, round 4: T51b, the meeting with Scholar, the army dialog, the commands between two armies (2026-10-26, `adventure/e08-t51b`)

**Baseline:** `master` f72f8c2 (#461, #463 merged); its engine code not verified yet, so the
sandbox ran the full verify with the original data first: every step passed. At its start the user
chose T51b whole (decision 118; recommended was two parts, the meeting first).

- **The decisions** (`state.rs`): `Decision::Meeting { owner, heroes: [visitor, visited] }`
  (GAM-DEC-003) and `Decision::ArmyDialog { owner, hero, group: Vec<ArmyStack> }` (GAM-DEC-004),
  opened by `settle_outcome` from the adventure's `AdventureOutcome::meeting` (`meet`) and
  `no_room` (`open_army_dialog`: the group as one stack in slot 0). Scholar first (HER-SPL-007):
  `meeting_spells` on both heroes' details, the visitor's `LearnSpell` changes, then the visited
  hero's, each `HeroChanged`, then `MeetingOpened`. `answer`: `Decline` closes either
  (`close_exchange`: `DecisionClosed`, then a `GroupLost` per stack left in the group, slot
  order), anything else is `WrongAnswer`. Queries `GameState::{meeting, army_dialog}`;
  `GameState::decision` stays the offer's alone. Game state hash v13: kinds 3 (owner, the two
  heroes) and 4 (owner, hero, the group's stacks).
- **The join offered without room** (ADV-MON-008): `answer_to`'s skip (`has_room`) gone, so the
  offer is made whatever room the army has; `ADV-STUB-2` closed (`docs/stubs.toml`). Accepted
  without room, the adventure's T50 outcome (`no_room`, the treasure and the monster's leaving)
  opens the dialog. No join offer waits for a guard (`guard_after` is set only for an object on an
  entered tile, never a monster's), so no battle starts while a dialog is open.
- **Management with a decision open** (`state/manage.rs`): `Together::{Alone, Meeting, Dialog}`
  (`GameState::together`, from the player's first decision; `None` for another kind),
  `check_managing` (the turn, that decision, the level-up's hold), `Together::admits` for an army
  command's armies (outside a decision as T51a: `NoGroup`, `NotTogether`; in a meeting its two
  heroes, the group `NoGroup`; in a dialog the hero and the group) and `admits_heroes` (a hero the
  open decision does not hold: `DecisionOpen`). An army command runs on a copy of the group,
  written back (`set_group`) only when the adventure accepts it; each slot told from its army
  (`stack_at`: the hero's on the map or the group). An artifact between the two heroes of a
  meeting applies `move_changes`' `from`, then its `to` changes; outside a meeting two heroes are
  `NotTogether` as before. The group's own commands (a move within it, its dismissal) are allowed:
  ADR-0016 point 3 (its last stack moves out, merges or is dismissed) and the original's upper bar
  (0x5C7400); ADR-0017 point 6's "between the hero and the group's small army" read as including
  them (an agent's reading).
- **The listing** (GAM-MAN-002): `legal_commands` lists `Decline` alone with either open (the
  env's entry 10); `default_answer` `Decline`; `legal_moves` none. `legal_management` for a hero
  the decision holds: its army's moves onto its own and the other army's slots (`army_commands`'
  `to`), its dismissals, with a dialog then the group's commands (from the group toward the hero),
  the formations and flags, each artifact's moves onto its own places and then the meeting's other
  hero's (`check_move` with `Some(other)`), the assemblies. `split_range` takes slots of two armies
  together (`admits`), with the dialog's group.
- **Events and views**: `GameEvent::{MeetingOpened, ArmyDialogOpened, DecisionClosed, GroupLost}`
  (the owner's alone, `project`; `GameEvent` stays `Copy`: the group is told per stack, as T51a's
  `StackChanged`, against the design's `ArmyDialogOpened { stack }` / `GroupLost { stacks }`, an
  agent's call; the design's `MeetingOpened { learned }` dropped: the learned spells are their
  `HeroChanged`). `DecisionView::{Meeting { heroes }, ArmyDialog { hero, group }}`.
- **Other crates**: `h3-map`'s view test follows the new decisions and the group's slots;
  `play.rs`' checks expect `Decline` alone with either open and its plans decline them (`seek`);
  `h3-cli`'s bench `seek` declines them too. `h3-sim` (the AI lane's) needed nothing: its masks
  come from `legal_commands`; its inbox told.
- **Replays**: the ten re-recorded with their options, their commands unchanged (no game meets an
  own hero or a join without room): every hash moved (`adventure_replays.rs`' history).
- **Rules**: GAM-DEC-003 and GAM-DEC-004 written, `verified` by batch E6's E6-1 to E6-4 and E6-7;
  ADV-MON-008, GAM-MET-001, GAM-MAN-001, -002, HER-SPL-007, ADV-ARM-002 and -007 restated.
- **Tests**: `h3-game`'s `tests/exchanges.rs` (the meeting's opening, listing, refusals, hash and
  closing; Scholar both ways, the visitor's first; stacks and artifacts between the two heroes and
  the refusals; the listing per hero and the split's query in a meeting and in a dialog against
  every candidate `apply` accepts; the join without room offered, its dialog's events, view,
  commands with the group and closing with the loss; a management command refused with an offer
  open), `level_ups.rs`' held game refusing one too, `h3-adventure`'s join offered whatever room.
- **Verify**: the scoped run without a failing step (its first coverage run found the two
  refusals untested and an implicit branch of the group's write-back, rewritten); the benchmarks
  passed `bench-check`.

### Increment 6, round 5: T52, the plans and properties, the scripted golden replays, the benchmarks (2026-10-26, `adventure/e08-t52`, `shared/e08-t52-benches` #469)

**Baseline:** `master` 7619a4a (#468 merged), its engine code verified (only the light gates
ran). At its start the user chose T52 whole (decision 119; recommended was two parts, the tests
first). The original game's files staged at the start, as the user's computer was about to go.

- **The plans** (`h3-map`'s `play.rs`; drawn last, so the regression files' cases keep their
  maps; the committed duel and XL maps unchanged, byte for byte): `Plan::exchange`, an
  `Exchange` in one plan of two: a mate of the first hero's owner on the tile beside it (a
  direction), Scholar and Wisdom at a level or none (its skills of its own), a Spell Book (the
  first hero wears one too), up to five spells of its own on a SoD map (`MATE_SPELLS`: levels 1
  to 5), Elixir of Life's three parts worn in one case of two (`ELIXIR_PARTS`, base artifacts
  every version holds; the combination is SoD's, which the original packs enable); in one case
  of two the first hero's army full (`FULL_ARMY`: seven creatures, none a plan's monster's) with
  six compliant goblins on the tile beside it opposite the mate's.
- **The driver**: `Step::Manage(n)` (the actor's management commands, `managing`: each hero's
  `legal_management`, then the splits of every pair of slots of the armies managed together to
  their range's ends and middle, `splits`; picked by kind first, `pick`, so an assembly among
  hundreds of moves is played as often as the others), `Step::Meet(n)` (a move ending nearest to
  another own hero; in a meeting or a dialog as `Manage`), `Step::Join(n)` (an offer to join
  accepted, else a move onto a wandering monster's tile, a full army's first), runs of a `Meet`
  and up to fifteen `Manage`s and of two `Join`s and `Manage`s; `Free::Manage` (an `Arbitrary`
  command: kind, heroes or the group, slots up to two past the army's, a count, places up to a
  few past the last). Agent's calls (change freely): the weights (2 single `Manage`s, 2 meeting
  runs, 1 join run in `steps`), the goblins opposite the mate (a random offset rarely led to a
  dialog: none in 96 cases), the join step preferring a full army's visit.
- **The checks** (every step of `every_step_keeps_the_commands_invariants`): `check_invariants`
  (every hero one to seven stacks in order, counts 1 to the stack cap, details `validate`d; a
  group's stacks alike, its hero the player's; a meeting's heroes the player's, neighbours on one
  level), `check_managing` (only the acting player lists management, for the heroes it may
  manage; both formations and flags, no command twice, a sample of 24 listed commands applies;
  a sample of 16 pairs of slots: a split range exactly where a split applies, its ends apply and
  one past each refuses; another player's range none), `check_unlisted` (an accepted management
  command is listed by a hero or a split within its pair's range), `check_conservation` (details
  change by the told hero changes applied in order: a meeting's spells, artifacts' moves,
  assemblies; a join without room holds its whole group, the army full; the told slots), the new
  `check_groups` (a dialog opens with the group whole in slot 0, its slots change as told, closed
  its stacks are lost and told in slot order), `check_decisions` (a meeting and a dialog open
  and close as told, one at a time), `check_isolation` (another player's meeting and dialog
  unchanged), `check_management` (no decision opens or closes, no hero moves; an army command
  keeps the named armies' creatures but a dismissed stack's; formation and flag only as told; an
  artifact's move keeps the two heroes' artifacts; an assembly turns the parts into the
  combination, a disassembly back), `check_inverse` (a stack moved onto an empty slot or swapped,
  an artifact moved onto an empty place: the inverse move applies and restores both armies and
  heroes; a merge and a swap out of a backpack have none). The main property passed 320 cases in
  a one-off run (48 committed). `the_plans_meet_and_manage` (96 fixed cases): 78 meetings, 10
  spells taught, 457 moves, 428 splits, 354 dismissals, 220 formations, 233 flags, 430 artifact
  moves, 24 assemblies, 18 disassemblies, 180 stacks and 98 artifacts between two heroes, 5 army
  dialogs, 2 group stacks taken, 14 lost, 38 refused, 841 moves undone by their inverse.
- **The army map** (`fixtures/maps/e08-army-12.h3m`, `fixtures.rs`' `army`): SoD, 12 × 12 of
  grass; red's Valeska (hero 0) at (3, 5), a full army, twelve backpack artifacts
  (`ARMY_BACKPACK`); Caitlin (hero 2, the mate) east of her: expert Scholar, advanced Wisdom,
  Magic Arrow, Ice Bolt, Lightning Bolt, Fireball, Bless, Haste, Elixir's parts; six compliant
  goblins west of her; blue's Rion (hero 1) at (9, 9). The fixture test counts a plan's mate
  among its heroes.
- **The scripted recording**: `Replay::record_commands` (`h3-sim`'s `replay.rs`: commands
  applied through `h3-game`, a rejected one `RecordError::Rejected`, nothing recorded),
  `h3 adventure run --script <file>` (`read_script`: one command per line as a replay writes it,
  `#` comments; a `.toml` path a replay's own commands, which re-record it byte for byte;
  `run_script`; the agents' options refused with it).
- **The golden replays** (`army-meeting-s1`, 17 commands, and `army-dialog-s1`, 8, checkpoints
  every 5; `adventure_replays.rs` lists what each does): the meeting (Scholar teaches Valeska five
  spells of level 2 at most, stacks moved, split, swapped and dismissed between the two, the axe
  to Caitlin's hand, the ring there and back, Elixir assembled, taken apart and assembled, a
  formation and a flag, the meeting closed, a walk on); the dialog (the join accepted without
  room, a stack dismissed, four goblins taken by a split from the group, the Monks moved into it,
  closed: two stacks lost).
- **The benchmarks**: `h3 adventure bench --meeting 1` (a player's heroes seek each other,
  `until_meeting` and `seek_own`; the random agent plays offers, level-ups, dialogs and battles on
  the way) and `--workload manage` (`management_queries`: both heroes' `legal_management` and
  the split range of every pair of the two armies' slots, `--queries` times). Gated:
  `fork-meeting-xl-144` 357 000 055 Ir (the meeting after 54 steps; +0.17 % over
  `fork-xl-144`'s battle: no copy per fork beyond the decision's) and `manage-army-12`
  399 997 833 Ir (250 times; Valeska lists 270 commands, 168 of them artifact moves, Caitlin 92;
  about 106 µs a listing in release). The four earlier workloads within 1 % of their baselines.
  The benches README's two rows are a shared path: `shared/e08-t52-benches` (#469).
- **The env** (the AI lane's): the CLI's test plays `random_rollout` to the day limit after a
  meeting and after an army dialog opened; the AI lane's inbox told (with the army map for its
  own test).
- **Versions**: none moved (the replay format stays v6; no hash).
- **Verify**: the full `cargo xtask verify --with-original-data` passed every step (coverage,
  determinism with the twelve replays, `bench-check` with the six adventure workloads) on code
  5a00f64 (commit 61c13d2); `check-lane` ok (lane 20 files, common 2).

### Increment 6, round 5: T53, the audit; increment 6 closed; increment 7 refined (2026-10-26, `adventure/e08-t53`)

Baseline: `master` at e8b9164 (#470 and #471 merged; CI's full verify green on it). No original
files staged: the task changes docs and six code comments only. At its start the user chose T53
whole (decision 120, `AskUserQuestion`; not chosen: T53a the audit and the closure, T53b the
refinement) and answered increment 7's three questions with it (ADV-Q26).

**The audit of increment 6's exit criteria** (design §15, moved below), on `master`'s code:

1. The army: `h3_adventure::{ArmyCommand, ArmyRef, ArmySlot, split_range}` and
   `AdventureState::{manage_army, split_range, set_formation, set_tactics}` (ADV-ARM-001 to
   -006, T50), a hero's last stack kept (`ArmyCommandError::LastStack`), counts 32-bit up to the
   cap (ADV-ARM-005, -007; A-15 fixed, `ADV-QUIRK-19`); the game's `Manage` on one hero
   (GAM-MAN-001, T51a; `tests/manage.rs`' `a_player_rearranges_a_heros_army_in_its_turn`,
   `the_formation_and_the_flag_are_set_and_told_to_the_owner_alone`). Observed: E6-2, E6-3,
   E6-4 (A-16, D-27), E6-6.
2. The meeting: a step toward an own hero visits it at the step's price (GAM-MET-001, T50),
   Scholar's spells learned before it opens, the visitor's first (HER-SPL-007, T49b, T51b),
   `Decision::Meeting` with `Manage` between the two heroes until the owner closes it
   (GAM-DEC-003, T51b; `tests/exchanges.rs`' `a_step_toward_an_own_hero_opens_a_meeting_its_decline_closes`,
   `scholar_teaches_both_heroes_before_the_meeting_opens_the_visitors_first`,
   `in_a_meeting_stacks_and_artifacts_move_between_its_two_heroes`). Observed: E6-7, E6-11.
3. The army dialog: the join offered whatever room the army has, a join without room opens
   `Decision::ArmyDialog`, what the player leaves in the group is lost (`GameEvent::GroupLost`;
   GAM-DEC-004, ADV-MON-008; `a_join_without_room_is_offered_and_opens_the_army_dialog`);
   `ADV-STUB-2` closed (T51b; no adventure stub but ADV-STUB-8, E10's). Observed: E6-1, E6-2.
4. The artifacts: `HeroDetails.worn` by the original's 19 positions with the locks
   (HER-ART-001, T49a), the first position that takes an artifact (HER-CHG-002), the capture in
   position order (HER-ART-004), moves (HER-ART-005: the book and the Catapult never, the machines
   only between machine positions, the backpack's 64 in order), combinations assembled and
   disassembled while worn with the SoD pack (HER-ART-006), Titan's Thunder's book (HER-ART-007,
   the import's too, map import v10); `h3-heroes`' tests (`an_artifact_moves_within_one_hero`,
   `what_never_moves_and_where_an_artifact_cannot_go`, `a_combination_assembles_from_its_worn_parts`,
   `equipping_titans_thunder_adds_a_spell_book_that_stays`). Observed: E6-8 (D-28), E6-9, E6-10.
5. The flag in battle: a side whose hero's flag is off gets no tactics phase, both heroes'
   Tactics reach the battle as 0 then (GAM-BAT-009, `Combatant::tactics`, T51a;
   `a_flag_off_denies_the_tactics_phase_its_side_would_get`). Observed: E6-6.
6. `check-rules` passes (every rule an id, a source and a test); the ledger holds `ADV-QUIRK-19`
   (A-15, exploit-fix, no flag) and `-20` (A-16, the original's bounds, no flag) with their
   regression tests (`between_two_armies_a_hero_keeps_its_last_stack_and_counts_stay_whole`,
   `a_split_leaves_both_stacks_within_one_and_the_most`). The gates: T52's full verify with the
   original data passed every step on code 5a00f64 (the ten replays re-recorded across the
   increment and the two new goldens `army-meeting-s1`, `army-dialog-s1`; the four earlier
   adventure workloads within 1 % of their baselines, `fork-meeting-xl-144` and `manage-army-12`
   baselined), CI green on #470 and on `master` after it. The env plays on through a meeting and
   an army dialog (`h3-cli`'s `the_env_plays_on_through_a_meeting_and_an_army_dialog`); the
   encoding is the AI lane's (`lanes.toml`: `crates/h3-sim/` but `replay.rs`, T45).
7. Batch E6 run (T47b2) and scored (T48, survey §13): every row as the reading, E6-4 (A-16, D-27),
   E6-8 (D-28) and E6-10 conclusive against VCMI; none reopened.

**The gaps fixed:** GAM-MET-001 cites E6-11's observation (the step's 100 points, the visitor on
its tile) but stayed `implemented` while its decision waited for T51b: now `verified`.
GAM-MAN-001 cites the hero screen's rows (E6-1 to E6-4, E6-6, E6-8 to E6-10), every one as
predicted: `verified`. ADV-MON-008's amendment (the join without room opens the army dialog) cites
E6-1 and E6-2 now; it stays `implemented`, as no row of any batch accepts a join with room.
ADV-ARM-001 (moves within one army; E6-5 dropped), ADV-ARM-005 (A-15, no row), ADV-ARM-007 and
GAM-MAN-002 (the cap and the listing, designs) stay `implemented`. Nothing else was missing.

**The design note** moved below ("Increment 6's design note at its closure"); `E08-design.md`
keeps a pointer until T55b writes increment 7's note. The six code comments that cited it by
section (`h3-adventure`'s `army.rs` and `state/manage.rs`, §7; `h3-game`'s `manage.rs` and
`state/manage.rs`, §10; `h3-map`'s `tests/play.rs`, §12 twice), `manage-army-12`'s description
(§16), GAM-MAN-001's design source (§10) and survey `E08-army.md` §13 (§14) now name the note in
this log; since the repository went public CI's full verify costs nothing, so the comments did
not wait for the next code change (T43 had deferred them). ADR-0017's citation of §16 (a shared
path) and the AI lane's inbox notes keep theirs: historical.

**Increment 6 closed** (2026-10-26); its scope, decisions 90 to 120 and task list moved to
`E08-archive.md`, the epic file from 37 KB to 18 KB.

**Increment 7 refined** (ADV-Q26; decisions 121 to 123): the user chose **rewards on the map**
of the sketch's item 7 cut in two (treasure chests and the objects that give experience,
artifacts, spells, skills or primary skills; artifacts as pickups; the monsters' treasures with
an artifact; the rewards that give creatures), not boats and the sea (now the sketch's item 8,
the week item 9), the whole item, or towns (E09) instead; the first round a survey and the ADR
amendment drafts with the design note and the cut (T55a, T55b), not the survey alone; and,
before it, a task of its own (T54) for the battle seam's facts the battle lane's notes ask
`h3-game` to set (difficulty, a player's human flag, colours, background, the attacker's ground,
a placed hero's sex; the retreat facts stubbed), not inside increment 7 or later with E10. The
inbox re-tagged: the battle lane's three notes of 2026-10-23 and -24 (the seam's facts) for T54;
the other entries keep their tags.

**Verify:** the scoped `cargo xtask verify --scope adventure --summary` passed every step it runs
(fmt, clippy, check-arch, check-rules, check-stubs, check-docs, check-lane, test, coverage,
determinism, bench-check, doc-drift; deny and ffi-smoke skipped by the scope; exit 2, its normal
result), the doc checks again after the last edits; no original data staged (no step needs them
for comments and docs). CI's full verify on the pull request is the record.

#### Increment 6's design note at its closure (moved by T53)

The note as T52 left it, every section built (T45 to T52; T53's round-5 row marked done); its §
numbers are the ones the code's comments cite as "increment 6's design note".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 6's
amendments of ADR-0013 (points 1, 3, 14), ADR-0016 (points 3, 4, 6) and ADR-0017 (points 1, 6, 8,
9, 11), drafted in T44b from the answers at ADV-Q24 (the epic's decisions 93–100), accepted at
ADV-Q25 with the cut (decisions 102–105; decision 104's expansions written in) and applied by T45.
The task that builds a section deletes it; the file goes when the epic closes. Facts and sources:
survey `docs/rules/surveys/E08-army.md` (§ numbers are the survey's unless they name this note).

Increment 5's note is in `E08-log.md` ("Increment 5's design note at its closure", moved by T43);
T45 repointed the two code comments that cited it.

Written whole (decision 101): **part A**, the hero side (`h3-heroes` and the core content); **part
B**, the map, the game, the batch and the cut.

##### Part A — the hero side (`h3-heroes`, the core content)

Everything in part A is core content (shared: `h3-content`, `h3-pack`, the packs) or a rule of
`h3-heroes`: pure functions of one or two heroes' details and the ruleset (ADR-0013 point 14 of
the draft). When the game applies them is part B's.

###### 1. The data

**Built in T45:** the kinds `meeting_spells { level }` (Scholar: 2, 3, 4) and `gives_spell_book`
(Titan's Thunder) in `h3-content` and `h3-pack` (codes 51, 52), their data in the packs; pack
format v47, core ruleset hash v48. T49b reads them (built: §3).

**No data for the positions**: they follow from the slots' order and counts (§2, built). **The expansions**
are the game's configuration (part B §10; decision 104).

###### 2. Positions (built in T49a)

Built in T49a (`E08-log.md`, "T49a"): `HeroDetails.worn` by position (`h3_heroes::positions`,
`Position`), the validation with the locks counted per slot, `first_position` (HER-CHG-002),
the capture and a lost machine's artifact in position order, `ArtifactPlace::Worn(Position)`,
`HeroChange::PlaceArtifact`, `wearing` for callers that list artifacts, the import v9.

###### 3. Moves, combinations, Titan's book, Scholar (built in T49b)

Built in T49b (`E08-log.md`, "T49b"): `h3_heroes::{ArtifactMove, check_move, move_changes}`
(HER-ART-005), `check_assemble`, `assemble_changes`, `check_disassemble`, `disassemble_changes`
(HER-ART-006), `equip` and the equip in `apply_change` (HER-ART-007), `meeting_spells`
(HER-SPL-007), the import v10 (an equipped Thunder's book). Part B applies them: a move's
changes on both heroes (§8, §10's `Manage`), the assembly and disassembly as commands (§10),
Scholar's spells at the meeting (§8).

###### 6. Rule ids and quirks (the hero side); its shared and `cross-lane` needs

| Id | Rule | § |
|---|---|---|
| HER-ART-001 (amended) | worn artifacts on positions; the locks counted per slot | 2 |
| HER-CHG-002 (amended) | a received artifact at its first position, else the backpack's end | 2 |
| HER-ART-004, HER-WAR-002 (restated) | the capture and a lost machine's artifact by position | 2 |
| HER-ART-005 | moves within a hero and between two (built) | 3 |
| HER-ART-006 | assembly and disassembly (the combinations the ruleset has; built) | 3 |
| HER-ART-007 | Titan's Thunder's Spell Book (built) | 3 |
| HER-SPL-007 | Scholar at a meeting (built) | 3 |

No quirk on the hero side (D-28 the original's; Titan's book a rule). The shared part (the two
kinds, the packs, the replay headers) was T45's. **`cross-lane` in the positions task:** the call sites of the battle, AI and client
lanes that build `HeroDetails` (§2's `wearing`), done in T49a with their inboxes told.

##### Part B — the map, the game, the batch and the cut

###### 7. A hero's army, formation and flag (built in T50)

Built: `E08-log.md`, T50 (ADV-ARM-001 to -007; the cap, decisions 109, 114, 115; adventure state
hash v11, ruleset part v11). For T51: `h3_adventure::{ArmyCommand, ArmyRef, ArmySlot}` and
`AdventureState::{manage_army, split_range, set_formation, set_tactics}` (the group passed in as
`Option<&mut Vec<ArmyStack>>`, by slot like a hero's army); they emit no event: the game tells the
owner (§10).

###### 8. The meeting (GAM-MET-001; GAM-DEC-003)

Built: the visit in T50, the decision with Scholar in T51b (`E08-log.md`, T51b): `Decision::Meeting
{ owner, heroes }`, `MeetingOpened` after Scholar's `LearnSpell` changes (the visitor's first),
`Decline` alone listed, `DecisionClosed`; `DecisionView::Meeting`; game state hash v13.

###### 9. The army dialog (ADV-MON-008 amended; GAM-DEC-004; `ADV-STUB-2` closed)

Built in T51b (`E08-log.md`, T51b): the join offered whatever room the army has, an accepted join
without room opening `Decision::ArmyDialog { owner, hero, group }` (the group as one stack in slot
0), `ArmyDialogOpened`, the group's slots told as `ArmyRef::Group`, `Decline` closing it with a
`GroupLost` per stack left; `DecisionView::ArmyDialog`.

###### 10. Commands, the listing, events, the observation, the expansions (GAM-MAN-001, -002)

Built in T51a and T51b (`E08-log.md`; decisions 116 to 118): `GameCommand::Manage`, the listing
per hero (`legal_management`, `split_range`; ADR-0017 point 6), the events per slot, and with a
meeting or an army dialog open the commands between its two armies (`Together` in `h3-game`'s
`state.rs`). **The expansions** (decision 104, the user at ADV-Q25: "it must depend on the game
configuration that is used to play, i.e. the enabled expansions"): the packs the game's ruleset is
compiled from; the combinations (the SoD pack's artifacts with `parts`) and the fifth misc
position (the SoD pack's slot override) follow them, so no setup field and no map version reach
the rules. A rule the packs cannot express yet (the moat, E09) gains data or a configuration input
then.

###### 11. The tactics flag in battle (GAM-BAT-009; ADR-0017 point 11)

Built in T51a (`deny_tactics` in `state/battle.rs`, `Combatant::tactics`): when the side with
the greater Tactics level is a hero whose flag is off, both sides' Tactics go as 0 (equal levels
still cancel, the difference still decides; survey §4). Every player's hero's flag counts (the
original reads a human side's only, decision 16); a side without a hero has no flag.

###### 12. The env, the encoding's move, replays, fixtures, properties, benchmarks

*Built in T52* (decision 119; the record in `E08-log.md`): the replays scripted with `h3 adventure
run --script` on a new map, `e08-army-12.h3m`; the plans' fixtures as an `Exchange` of the first
hero; the fork measured as `fork-meeting-xl-144` (+0.17 % over the battle's) and the listing as
`manage-army-12`; the env's test stays the AI lane's (its inbox).

- **The env is the AI lane's** since T45 (decisions 99, 103; ADR-0017 point 8); the replay model
  (`replay.rs`), `h3-cli`'s adventure commands, the golden replays and `benches/adventure/` stay
  this lane's. Increment 6 changes only the env's masks: `Decline` while a meeting or an army dialog is open
  (§8, §9), so the rollout and the replays' recorder never stall in a meeting; the masks are built
  from `legal_commands`, so this may need only the AI lane's test (its inbox; this lane tells it
  when T51 lands).
- **Replays** (format v6, built in T51a): the management commands (`Manage`); two new golden replays, scripted
  (the rollout does not manage): a meeting with Scholar and exchanges of stacks and artifacts, and
  a join into a full army through the army dialog; the ten re-recorded (the hashes move).
- **Fixtures and plans** (`h3-map`'s `play.rs`): two heroes of one player side by side, a full
  army next to a friendly group, artifacts in a backpack, a combination's parts, Scholar and a
  book; the plans meet, exchange, split, dismiss, assemble and fill an army; `check_progression`
  and a new invariant check (every army 1–7 stacks, counts > 0; every hero's details valid) after
  every command.
- **Properties**: a random sequence of legal management commands keeps the invariants and never
  changes another player's view; a rejected command leaves the state's hash; a move and its
  inverse restore the hero (where the original allows the inverse).
- **Benchmarks**: `fork-xl-144` with an open meeting (no new copy per fork beyond the decision's);
  `legal_management` during a meeting (a new workload, T52); the four gated workloads at their
  baselines (the walker's search untouched).

###### 13. Versions

| Version | Change | Task |
|---|---|---|
| core ruleset hash v48, pack format v47 | two effect kinds, the packs' data | T45 (done) |
| game state hash v12 | the details' positions | T49a (done) |
| adventure state hash v11, adventure ruleset part v11 | the tactics flag; a stack's most creatures | T50 (done) |
| game state hash v13 | the meeting and army dialog decisions | T51b (done) |
| map import v9 | positions | T49a (done) |
| map import v10 | Titan's Thunder's book at load | T49b (done) |
| adventure replay format v6 | `Manage` | T51a (done) |
| action encoding | unchanged (v4); the AI lane's from the move on | — |

###### 14. Batch E6

Ten of survey §9's eleven rows (decision 100; E6-5 dropped by decision 109), on the harness (`clients/capture/`: a common path
since CLT-Q18) in SoD on Easy, hot seat, from a map `h3-experiment` writes (battle-owned:
`cross-lane`). The spec (the rows' setups, `experiments/specs/E08-batch-6*.toml`) and the
results template are T46; the harness's new clicks and reads (survey §9's list), the recipes and
the run T47 (as T36 and T37 for batch E5); the scoring T48 (an ADV-Q if a
row contradicts the reading; A-16's E6-4 decides only the ledger's text, the rule being fixed by
decision 98). T47 is cut in T47a (the harness's clicks and reads), T47b1 (the recipe, checked
with `fill`) and T47b2 (the run; decisions 108, 110). Scored in T48 (survey §13): every row as
the reading, no question.

###### 15. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** round 1's `shared/` task done (T45: the ADRs, the two kinds, the
packs, the replay headers, the encoding's move); batch E6's spec (`h3-experiment`, `cross-lane`);
the positions' call sites in other lanes' code (T49, `cross-lane`). The harness is common. Everything
else is the lane's own.

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). As in increment 5, **batch E6 runs before the build**: it tests the
original only, and its answers (A-16's text, the assembly's questions, the tactics
flag) land once:

| Round | Tasks | Content |
|---|---|---|
| 1 | **T45** + **T46** | T45 (done): the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016, -0017 amended; `meeting_spells`, `gives_spell_book`; the packs; core ruleset hash v48; the encoding's move to the AI lane (`lanes.toml`, the versions table, the global HANDOFF, the AI lane's inbox). T46: batch E6's spec and results template (§14; `h3-experiment` `cross-lane`) |
| 2 | **T47** + **T48** | T47: the harness's clicks and reads (the hero, exchange, split, creature and army dialogs; the spells, backpack, flags and tactics reads), the recipes and the run. T48: scored; the survey's §8 and the rules' sources; an ADV-Q if a row contradicts the reading |
| 3 | **T49** | `h3-heroes`: positions (§2: the model, validation, the first position, the capture, the import v9, the constructor and the other lanes' call sites, `cross-lane`), moves, combinations and Titan's book, Scholar (§3). The largest task: cut in T49a (positions, §2; done) and T49b (§3; decision 111; done) |
| 4 | **T50** + **T51** | T50 (done whole, decision 113): `h3-adventure`: the army commands, the split's bounds, dismissal, the formation and the flag, the cap, the meeting's visit, the join without room's report (§7, §8 visit, §9); adventure state hash v11, ruleset part v11. T51 in two (decision 116): T51a (done): `h3-game`: `Manage` on one hero, its per-hero listing and `split_range` (decision 117), events, the tactics flag in battle, replay format v6 (§10, §11); T51b (done whole, decision 118): the meeting and army dialog decisions, Scholar's application, the commands between two armies, the join offered without room (§8 to §10); game state hash v13; `ADV-STUB-2` closed |
| 5 | **T52** + **T53** | T52 (done whole, decision 119): replays v6 and the two new goldens (scripted), fixtures, plans, properties, benchmarks, `--with-original-data` (§12); the env's masks the AI lane's. T53 (done whole, decision 120): the audit against the exit criteria, increment 6 closed, increment 7 refined |

**Increment 6's exit criteria:**
1. A player rearranges each hero's army (move, merge, swap, split within the original's bounds,
   dismiss), never leaving a hero without a stack, counts whole (A-15 fixed); sets each hero's
   formation and tactics flag (ADV-ARM-001 to -006).
2. A step toward an own hero meets it at the step's price: Scholar's spells are learned, and
   until the owner closes the meeting both heroes' stacks and artifacts move between them
   (GAM-MET-001, GAM-DEC-003, HER-SPL-007).
3. A join that finds no room opens the army dialog; what the player does not take is lost
   (GAM-DEC-004, ADV-MON-008; `ADV-STUB-2` closed).
4. Worn artifacts sit on the original's positions: moves, the first position that takes an
   artifact, the capture and a lost machine's artifact follow it; the Spell Book and the Catapult
   never move, the machines only between machine positions, the backpack holds 64 in order;
   combinations assemble and disassemble while worn (where the game's configuration enables them), Titan's
   Thunder gives its book (HER-ART-001 to -007).
5. A hero whose flag is off denies the tactics phase it would get, and the two heroes' Tactics
   still cancel as before (GAM-BAT-009).
6. Every new rule has its id, source and tests; A-15 is in the ledger as fixed; the gates pass
   (coverage, determinism with the re-recorded and the two new replays, the benchmarks at their
   baselines and the new one baselined); the env plays on through meetings and army dialogs; the
   encoding is the AI lane's in `lanes.toml`.
7. Batch E6 run and scored; D-27, D-28, A-16 and the consensus rows `verified` or reopened.

###### 16. Risks and open points

- **The listing's size** (§10): settled at T51's start by listing per hero (decision 117,
  ADR-0017 point 6), so `legal_commands` and the env's rollout stay as they were; a hero with
  seven stacks and a full backpack still lists a few thousand commands, which T52 measures.
  T52: in a meeting a hero with seven stacks and twelve backpack artifacts lists 270 commands,
  its mate 92; both listings and the 196 split ranges of the two armies cost about 800 000
  instructions (`manage-army-12`), about 0.1 ms in release.
- **The handover of the env**: the AI lane takes an env whose encoding lacks increment 6's
  commands; agents trained before E13's macro-actions manage nothing. The masks' `Decline` keeps
  the env playable; the encoding's growth is the AI lane's call (its inbox).
- **A stack's count cap**: settled at T50's start (decisions 109, 114, 115; ADR-0016 point 3) and
  built (ADV-ARM-007); recruiting and growth keep it when they come.
- **A-16 and the split's bounds** rest on the code reading; E6-4 confirmed them (T48: 1 and 9). The engine's
  [1, t − 1] with the move reproduces the original's reachable outcomes (§7): a row that showed
  the dialog allowing what the click refuses would reopen it.
- **The expansions** (decision 104): a game's packs decide; the import's map version does not
  reach the rules. The moat (E09) is the next rule that differs by expansion.
- **Not in increment 6** (survey §12 point 9, decision 100): allied heroes' meetings and one-way
  trades (teams, E10), the computer player's join without room and meetings (E10), hero
  dismissal and the creature dialog's upgrade (E09), the backpack's rotation as a command of its
  own, the other callers of the army dialog (rewards, increment 7).

### Increment 7, before round 1: T54, the battle seam's facts from the game (2026-10-26, `adventure/e08-t54`)

Baseline: `master` at f2fc414 (#472 merged; CI green; the sandbox's light gates on code a4fc709,
which CI had verified). Original files staged from the user's repo folder (the executable, both
LODs, `maps.tar`, `extracted.tar`). At its start the user answered ADV-Q27 (decision 124): the
human flag from the map's player kinds with a session's override, T54 whole, the `cross-lane`
lines.

**The code reading** (survey `E08-objects.md` §2.5; `map.toml`'s 0x41ADB0, 0x505610, the new
`class_is_ownable` 0x40FCA0). 0x41ADB0 counts the class each tile within 3 of the battle's map
position (battle manager +0x13FF0) records in its own record (`map_item` +0x1E), not an object
list. 0x505610 sets that class from the tile's draw list, topmost first, in four walks: an object
whose entrance is on the tile; else an ownable one (0x40FCA0's byte table, 0 true: 17, 20, 33,
42, 53, 77, 87, 98) whose passability blocks the tile; else any blocking it; else one of class
124; else the reset's 0 (3 on coastal land). The 485 decoration templates (classes 114–160) of
`Objects.txt` have no entrance tile; 44 block nothing. 0x40FCA0's table was first misread (class
77 left out); corrected before the code.

**Built.** `h3-adventure`: `Scenery` (other, ownable, tree, mountain, dead vegetation) on
`ObjectSetup` and `MapObject` (adventure state hash v12); `AdventureState::background` with
`tile_sceneries` (one reverse pass over the objects whose masks reach the 7 × 7 square, the
walks ranked per tile, removed objects skipped; the topmost is the last in map order, decision
124), `special_ground` (split out of `battle_ground`), `battle_at` building every `BattleStart`
(heroes, monsters, guards), which gains `background` and `attacker_ground`. `h3-game`:
`PlayerSetup::human` and `HeroDetailsSetup::sex`; the players keep the flag, `Progression` the
sexes (no extra reference count per fork); `GameState::human`, `hero_sex`; `start_battle` fills
`difficulty` (the adventure state's), `human` (false without a player), `colours`, each hero's
`sex`, `background`, `attacker_ground`, and leaves `retreat` at its default (`ADV-STUB-9`, E10).
Game state hash v14. `h3-map`'s import: the human flag from the map's player kinds, every
object's scenery by class (`codes::scenery`), a placed hero's sex (the body's code, else its
type's map-wide entry's; 0 male, 1 female, any other the type's); map import v11; the test map's
hero body takes the format's default sex code 0xFF (the model's `Default` gives 0, male).
The attacker's ground passes every special ground: the battle's `mastery_at` only raises, so a
ground the original's `tile_spell_field` leaves out (Cursed Ground and the others) values alike.

**Tests.** `the_background_counts_the_scenery_the_tiles_around_a_battle_record`,
`a_tile_records_its_entrance_then_an_ownable_block_then_the_topmost_block`,
`a_battle_from_the_map_carries_its_background_and_the_attackers_ground` (h3-adventure);
`a_battles_setup_carries_the_difficulty_the_human_sides_colours_and_sexes` and the extended
`expected_setup`, the monster fight's observation (h3-game);
`a_maps_player_kinds_and_object_classes_carry_over`,
`a_placed_heros_sex_is_the_bodys_else_the_map_wide_entrys` (h3-map). Rules GAM-BAT-010 to -013.

**Replays.** All twelve re-recorded with their options, their commands unchanged (the duel's
players a human may play now count as human; no game's battle met a bad-morale freeze of theirs):
every hash moved, the headers' versions.

**Cross-lane** (the user's consent, ADV-Q27): `crates/h3-cli/src/ai/census.rs`' `PlayerSetup`
literal, `crates/h3-sim/src/adventure/tests.rs`' and `tests/monsters.rs`' `ObjectSetup` literals
(`scenery: Default::default()`).

**Later readers of the human flag** (E10): a computer player's starting resources (GAM-RES-002's
computer row), its loose formation (ADV-QUIRK-8), its ignored tactics flag (GAM-BAT-009).

### Increment 7, round 1: T55a1, the rewards survey's census and pickups (2026-10-26, `adventure/e08-t55a1`)

Baseline: `master` at e1217a2 (#473 merged; CI green; the sandbox's scoped verify on its code
passed). Original files staged from the user's repo folder (the executable, `maps.tar`,
`extracted.tar`; not the LODs: nothing here touches the packs). At its start the user asked how
much code T55a reads: measured from `visit_object`'s 94 cases and their callees, about 8 000 to
10 000 instructions not read yet (the per-class handlers 80 to 330 each; the shared helpers mostly
read; Pandora's Box, the banks and the army dialog's callers share one large closure). The user
took the agent's cut in four (decision 125).

**The survey** (`docs/rules/surveys/E08-rewards.md` §0 to §5, §9 and §10 for T55a4):

- **The census** (§1): every class that may hand a visiting hero something, by the part that
  reads it, with objects, maps and the objects on the five maps that import whole; the bodies of
  T55a1's classes. 11 720 chests on 157 maps; 362 placed and 3 371 random artifacts; 100 scrolls;
  53 monsters with an artifact on 18 maps (8 refused by today's import for it).
- **The readings** (18 new `map.toml` entries, three amended): the chest's draw at a new game
  (`setup_map_objects` case 101; the tutorial's 1 500 gold) and its visit (0x4A6520, the gold or
  experience choice 0x4A6440, the computer's 0x527FE0); the artifact's body (0x4FF120, from the
  body switch 0x502E00), the given marks (0x4C0AA0) before the random artifacts and the chests,
  the visit (0x49F7E0) with the full backpack, the guarded artifact's question (0x49F070) and the
  six pickup conditions that no map reaches; the scroll (0x4FF2F0, 0x4A5EA0, 0x4A5A80: a guarded
  scroll without a text fights at once); the monster's treasure (0x4A6B30: the artifact lost on
  a full backpack, then the resources). VCMI 1.7.5 and the wiki compared for each.
- **Disputes** D-29 (a guarded artifact without a text: the wiki says it fights at once), D-30
  (where a reward's artifact goes: the wiki says the backpack), D-31 (class 65 without relics:
  VCMI rolls any class), D-32 (repeats); no quirk candidate; the tutorial's chests a question for
  the checkpoint.

*Agent's calls (change freely):* the census ran from a throwaway example in a `git worktree`
outside the clone (its own target directory), so the baseline verify running in the clone was not
disturbed; the wiki was read through pages a search returned (a direct fetch of other pages is
refused in this session); a general text is cited by its table offset, not a GENRLTXT line (the
line numbering differs between VCMI and the table); 0x49ED50, 0x49EA40 and 0x49E8F0 (the pickup
conditions) were read by their arguments and calls only: no map reaches them.

**Verified:** the scoped verify (`cargo xtask verify --scope adventure --summary`, without the
LODs) with no failing step; `scripts/exe.py check` and `cited` clean.

### Increment 7, round 1: T55a2, the objects that teach (2026-10-26, `adventure/e08-t55a2`)

Baseline: `master` at bef62ab (#474 merged; CI green; the sandbox's baseline verified the code).
Original files staged from the user's repo folder (the executable, `maps.tar`, `extracted.tar`).
At its start the agent measured the task from `visit_object`'s 20 cases (their handlers and
callees to depth 2, helpers with more than 40 callers and read functions left out): about 6 700
instructions not read on the human player's path in 54 functions (the computer's 0x528040, 1 922,
left out), the handlers 80 to 300 each, the University's dialog about 2 400. The user took the
whole task with the University's dialog read for its rules only (decision 126).

**The survey** (`docs/rules/surveys/E08-rewards.md` §6, with §0, §1, §9 and §10 for T55a4):

- **The marks** (§6.1): `setup_map_objects` numbers each once-per-hero class's objects; the hero
  keeps a u32 per class (hero +0x57 to +0x7B), so a 33rd object shares the first one's mark (A-17;
  at most 30 on a shipped map). The team's marks per kind (game +0x4E344), per tile (0x4FBF90) and
  per player (record +0xC0, +0xC4) are knowledge only, some set before a refusal.
- **The readings** (39 new `map.toml` entries, six amended; 18 `structs.toml` fields): the four +1 objects, the Arena (its
  Cancel taken as Attack), the Library (level + 2 × Diplomacy ≥ 10), the Schools (1 000 gold, a
  choice or leave), the Learning Stone (1 000 × Learning), the Tree (a price drawn at a new game,
  one level by `experience_to_next_level`, no Learning), the Shrines (`random_shrine_spell`: no
  repeat within a level until it is used up, the marks starting as the bans), the Witch Hut (the
  map's mask, all 28 without one, less the bans), the Scholar (drawn at the map's load, a fallback
  primary skill drawn at a new game, removed), the University (four drawn skills; known, class
  weight 0, eight skills or less than 2 000 gold refuse), the Corpse (20 %; full backpack 1 000
  gold), the Wagon (10 / 40 / 50 %; full backpack its resources), the Warrior's Tomb (30/50/15/5 %;
  −3 morale once until the battle's end, cleared by `hero_battle_bonuses_clear` 0x4DA510; full
  backpack: the artifact stays), the Lean To. VCMI 1.7.5 and the wiki compared for each.
- **The census** (§1): no class above 32 on a map; the bodies of the Shrines, Witch Huts and
  Scholars; no empty pool.
- **Disputes** D-33 (an RoE map's Witch Hut teaches Leadership and Necromancy in the code only),
  D-34 (the Shrines' repeats), D-35 (the Corpse's and Wagon's artifact class, the Wagon's amount),
  D-36 (Necromancy at a University in the code only); quirk candidates A-17 (the 32-bit marks) and
  A-18 (a spell of −1 not tested); rule ids proposed (ADV-TCH-001 to -007, ADV-PIK-009 to -012,
  HER-ADV-008); batch E7's rows listed.

*Agent's calls (change freely):* Lean To is in (decision 126); the census ran from a throwaway
example in the clone (excluded from git, deleted); the University's dialog was read in its three
rule functions (0x5F0790, 0x5F0AC0, 0x5F0F60), its drawing and 0x5D2DA0 not; the wiki was read
through pages a search returned (the Corpse's was not returned).

**Verified:** the scoped verify (`cargo xtask verify --scope adventure --summary`) with no failing
step; `scripts/exe.py check` and `cited` clean.

### Increment 7, round 1: T55a3, Pandora's Box, events and the Pyramid (2026-10-26, `adventure/e08-t55a3`)

Baseline: `master` at f91cdd5 (#476 merged; CI green on it). Original files staged from the user's
repo folder (the executable, `maps.tar`, `extracted.tar`). At the start the agent measured the
task. It took the classes' handlers and their callees to depth 2, and left out the drawing, the
battle itself and the computer's branches:

- Pandora's Box and events: about 2 300 instructions, most of them one shared reward routine,
  0x49FA90; their dialogs about 2 900 more.
- the Pyramid: about 420.
- the Refugee Camp: about 1 400, most of it the hire dialog of dwellings and towns.
- the five creature banks: about 3 000, plus the battle's bank flag (battle +0x53C5), which
  `BattleSetup` lacks.
- Seer's Huts and quest guards: about 1 400, plus a quest system of nine missions.

Decision 121's two other callers of the army dialog turned out to be the banks' reward and the
Seer's Hut's. The user took T55a3 whole with Pandora's Box, events and the Pyramid, and placed the
others in the sketch: the banks as item 10, the Refugee Camp in item 9, the quests as item 11 after
E09 and E10 (decision 127).

**The survey** (`docs/rules/surveys/E08-rewards.md` §7, with §0, §1, §9 and §10 for T55a4):

- **The reward record** (§7.1): boxes and events share one list of 228-byte records (H3API's
  H3PandorasBox, the layout confirmed). An event's word holds its colours (bits 10–17), the
  computer flag (18) and the removal flag (19). Past 400 records the index stays 0 (A-19;
  at most 99 on a shipped map).
- **Pandora's Box** (§7.2, `visit_pandora_box` 0x4A0C50): the map's text, then "open?"; guards with
  a warning and the guarded pile's battle; the rewards; empty: a message; removed.
- **Events** (§7.3): `event_fires_for` (0x4FCDC0) for a human of a colour in the mask, a computer
  only with its flag; **the move ends on the tile** (`move_interrupted` 0x69777C ends the move's
  loop); `event_fire` (0x4A0ED0): guards fought at once, the rewards, removal with the flag (else
  it fires at every entry), the end-of-game tests.
- **The rewards** (§7.4, `grant_rewards` 0x49FA90), in order:
  - the experience is computed first, with the Learning before the box's skills;
  - primary skills, positive values only;
  - secondary skills, raised to the level;
  - mana, the total held in 0 to 999;
  - morale and luck, to the bonuses until the next battle's end;
  - resources;
  - artifacts, lost when the backpack holds 64;
  - spells, which need a book and a level of 2 + Wisdom at most;
  - creatures; those without room go into one army dialog (0x5D12B0, read, renamed
    `army_dialog_with`);
  - the experience, given last.
- **The Pyramid** (§7.5):
  - At a new game it draws a level-5 spell with a school that the game does not ban. Titan's
    Lightning Bolt is always banned by its flag 0x2000.
  - At every visit it asks first.
  - A looted Pyramid costs −2 luck once until the next battle's end (status bit 12).
  - The guards are 40 Gold Golems, split by `monster_battle` with a second creature into 1 to 5
    stacks (the table 0x63DF94 orders the slots), and 20 Diamond Golems in 2 stacks.
  - The spell is lost without a book or without Expert Wisdom.
  - An empty pool would read outside its list (A-20, unreachable).
- **The census** (§1): 46 boxes, 486 events (48 on the whole maps), 60 Pyramids; the bodies' kinds
  and ranges, with nothing out of range.
- **Disputes**:
  - D-37, a known skill: raised to the level in the code, added in VCMI;
  - D-38, mana: up to 999 in the code, up to the maximum in VCMI;
  - D-39, the order of the parts and the Learning used;
  - D-40, the Pyramid's Gold Golem stacks.
- **Proposed** for T55a4: the rule ids (ADV-PIK-013, ADV-OBJ-007, ADV-RWD-001, ADV-TCH-008,
  ADV-MON-012, HER-ADV-008 widened) and batch E7's rows.

The readings went into `map.toml` (21 new entries, 10 amended, among them `monster_battle`'s second
and third creatures and `new_game_bans`' always-banned flag) and `structs.toml` (`reward_record`;
the spell's level and school; a player's second human flag). `E08-army.md` §1.1 point 5 is updated
for 0x5D12B0.

*Agent's calls (change freely):*

- The dialogs of Pandora's Box and events were read for their rules only (decision 127).
  `load_rewards` (0x4FF6B0) was skimmed, and the census checks what it reads.
- The end-of-game tests after an event (0x5F1B10, 0x5F1C80, 0x4F2CE0) are named for E10, not read.
- The census ran from a throwaway example in the clone, excluded from git and deleted.
- The wiki was read through pages a search returned (Pandora's Box, Event, Pyramid).

**Verified:** the scoped verify (`cargo xtask verify --scope adventure --summary`) on 7bf3a6b
passed every step but `check-exe-map`. The one address it flagged, 0x4A39A0, got its entry in
d1e4c1e, after which `check-exe-map`, `check-docs` and `check-lane` passed. `scripts/exe.py check`
and `cited` came back clean.

### Increment 7, round 1: T55a4, the common rules, batch E7's plan, the second reading and the checkpoint (2026-10-26, `adventure/e08-t55a4`)

Baseline: `master` at ef3c34b (#479 merged; CI's last code run on `master`, 9215205, green; the
scoped verify in the sandbox passed, `deny` skipped). The executable and `Objects.txt` staged from
the user's repo folder (no LODs: docs only). At the start the user took **T55a4 whole** (decision
128) over two or three parts.

**The survey** (`docs/rules/surveys/E08-rewards.md`):

- **§8, the common rules:** what a visit asks (§8.1: ten classes ask, the rest do not; a battle, the
  rewards, the army dialog, then the experience and its level-ups), refuses or converts (§8.2: a
  full backpack, no book, Wisdom, a known spell or skill, eight skills, gold, the level, a visit
  made), what changes when (§8.3: at once, at the next step, at the owner's next turn, at the next
  battle; a Knowledge gained leaves the spell points), what a player knows (§8.4), the hidden event
  and the move (§8.5), the commands, decisions, events, observation, replays, benchmarks and env
  (§8.6).
- **§10** rewritten as T55b's inheritance (the import's changes with one new refusal, the draws, the
  new hero state, the battles); **§11** batch E7's plan: 23 rows in two runs (E7a the pickups and
  the teachers, E7b the boxes, events and the Pyramid), with what the harness and the spec lack;
  **§12** the proposed rule ids (new families `ADV-TCH`, `ADV-RWD`; GAM-DEC-005, -006, ADV-MOV-011,
  ADV-FOG-007, ADV-VIS-006, HER-CHG-003, HER-ADV-008); **§14** the checkpoint's questions.

**The second reading** (§13): two fresh agents in turn, from questions per function and without the
survey: A, nine questions on the pickups and the objects that teach; B, seven on the reward routine,
the box, events, the pathfinder, the records and the Pyramid. Both confirmed the readings point by
point, but:

- **An RoE map's Witch Hut** gets the fixed mask 0xEFBF from the loader (0x5034BD, checked by T55a4):
  14 skills, never Leadership, Necromancy or Water Magic to First Aid. T55a2 had read all 28. D-33
  reshaped (VCMI's 26 against the code's 14), **A-21** (the editor's default cut to 16 bits).
- **The Wagon's draws** come as r, the amount, the resource (§6.9 corrected; not a rule).
- **The pathfinder makes an event's tile a stop for every colour** (`pathfinder_pass_through`'s
  mouse mode answers no for every class but 33 and 212, checked by T55a4): a human's path never
  crosses a hidden event and bends around it. §8.5 rewritten with two ways; **A-23**.
- Smaller: the pool's later-edition bans, the lost artifact when nothing takes it, the combination
  question's conditions, the 999 clamp that lowers a hero above it, **A-22** (a University's
  negative range), **A-24** (the loader's creature count past seven).

**The exe map:** four entries amended (`pathfinder_pass_through`, `load_object_body`, `load_rewards`,
and `adventure_mouse_move` 0x40E360, which the map-client lane's #480 had mapped meanwhile: a
merge conflict, its entry kept).

**The checkpoint** (ADV-Q28, asked interactively; decisions 129–136): the disputes the code's side,
confirmed or reverted by batch E7; A-21 a flag; A-23 a flag **only if row E7-23 confirms it** (the
user is skeptical); batch E7 whole before the build; no original limits on objects and their state
(the user: a map whose every tile is a box must work; A-17 and A-19 not reproduced and no smaller
bound in their place); the tutorial not modelled; §8 as written; decision 104 for the pool and the
scope confirmed. **T55a done**; T55b is ready.

### Increment 7, round 1: T55b, the ADR amendment drafts, the design note and the cut (2026-10-26, `adventure/e08-t55b`)

Baseline: `master` at b2d2653 (#482 merged; CI's last code run, on 7a8899c, green; the sandbox's
light gates passed on code 3c4cd19, which CI had verified). Docs only; no original data staged.
**T55b whole** (decision 137, the user's choice of four at the start: three parts recommended),
one session and one pull request, as T44b.

**The drafts** (`docs/lanes/adventure/adr-drafts/*-increment-7.md`): ADR-0013 points 1, 2 and a new
15 (the battle bonuses in the details with their once-marks, reaching the battle through the
hero's own morale and luck effects, no seam change; the hero changes `GainPrimary`, `RaiseSkill`,
`Bonus`, `ClearBonuses`; the reward record's hero half in the original's order; what a teacher may
teach; the backpack's count as the test for receiving an artifact); ADR-0016 points 1, 3, 4, 6 (the
visits' state without the original's bounds; 31 classes as kinds; random contents at the build,
the random artifacts first; refusals as outcomes; a won guards' battle's rest after its levels;
events as hidden objects that stop a move, A-23 in `Reach` only if E7-23 confirms it; the
visitor's owner's knowledge; events never observed); ADR-0017 points 1, 4, 6, 8–11 (the human flag
and the spell and artifact bans in the setup; increment 7's draws; the new decisions; the
University; the visit's rest; resources lost; T54's battle facts in point 11, decision 124's
line; the bonuses cleared at every battle's end).

**The design note** `E08-design.md` rewritten for increment 7: part A (§1 the data, §2 the hero
changes, §3 `grant`, §4 the teachers' functions, §5 the bonuses in battle, §6 ids), part B (§7 the
import, §8 the kinds, §9 the draws, §10 the state, §11 the visits, §12 events and the move, §13
information), part C (§14 the decisions and typed answers, §15 the application, §16 the env,
replays, fixtures, properties and benchmarks, §17 versions and ids, §18 batch E7, §19 the cut and
the exit criteria, §20 risks).

**Decision 133's measurement** (§10; a throwaway callgrind program in the sandbox on `BitGrid` and
stand-in tables, release build, not committed): on the XL map's 41 472 objects a fork costs 19 Ir
for the once-per-hero marks in one map by hero id (its first write 2 727, 7 973 and 26 233 Ir with
8, 48 and 156 heroes) against 453 Ir with a set on each of eight heroes (13 % of today's 3 564 Ir
`fork-xl-144` fork); the players' knowledge 20 Ir (2 238 on a write); today's army left, one
ordered map behind an `Arc`, copies every entry on a write: 11 086 Ir at 30 entries, 388 877 at
1 000, 16 459 140 at 41 472; a two-level table with shared armies 4 197 Ir at 41 472. Chosen (the
agent's calls of decision 137): one block behind one `Arc` for the marks, the knowledge and the
emptied finds; the army left a two-level table.

**Findings of the design:** (1) the battle bonuses need no seam change: `morale_and_luck` already
sums the own side's morale and luck, so the bonuses fold into it; (2) the original gives a won
battle's levels before a box's, an artifact's or the Pyramid's rest, which matters when a level-up
gives Wisdom or Learning: the game holds the rest (GAM-DEC-007); (3) an event's stop is the move's
outcome in both profiles, also for a client's explicit path; A-23 changes only `Reach` (and so the
legal moves); (4) an RoE hut's mask stays the map's fact (none), A-21 is the build's rule, so the
import reads no profile; (5) A-20 fixed as a Pyramid without a spell, A-22 as a University
offering what is left; (6) open for T62's start: the order of an event and a monster guarding its
tile on one step. New ids beyond survey §12: GAM-SET-003, GAM-DEC-007, GAM-BAT-014.

**The cut** (§19): eight rounds, batch E7 first and before the `shared/` task (T56 the maps and
spec; T57 the harness, recipes and runs + T58 the scoring; T59 `shared/`; T60 `h3-heroes`; T61 the
import, kinds, draws and state; T62 the visits; T63 `h3-game`; T64 the replays and gates + T65 the
audit). T56 queued. The AI lane's inbox told of the typed answers and A-23's masks.

**The checkpoint** (ADV-Q29, asked interactively; decisions 138–141): the drafts and the design
accepted, but the user, on the ADR-0017 draft's `Answer` gaining `Buy(u8)`: "I think there should
be specific types tailored to the choice to be made"; asked again, **every decision's answer typed
by its kind**, increments 2 to 6's migrated in T63 (`h3-sim`'s action mapping `cross-lane`); the
ADR-0017 draft's point 6 and the design's §14 rewritten. Batch E7, then shared, then the build;
the University's purchase an answer (`University(Buy(slot))`); the levels before the visit's rest.
**T55b done**; T56 is ready.

### Increment 7, round 1: T56, batch E7's maps and specs (2026-10-26, `adventure/e08-t56`)

**The cut** (decision 142): measured at the start (the writer's bodies, the bodiless classes, a
monster's treasure and the RoE format about 800 to 1 000 lines with tests; the spec's fields 500 to
700, `cross-lane`; the two specs about 1 000 lines of TOML), the user took T56 whole; the AI
lane's one line (`treasure: None` in `crates/h3-cli/src/ai/tests.rs`) consented when asked.

**The writer** (`h3_map::writer`, `writer/rewards.rs`): `RewardObject` with `RewardBody`
(`Artifact` for classes 5 and 65–69, `Scroll` 93, `Shrine` 88–90, `WitchHut` 113, `Pandora` 6,
`Event` 26), each with the body the reader reads: the message record (`Message`: text, guards),
the reward record (the reader's `model::Rewards`), an event's players, computer and removal flags;
`Monster::treasure` (`Treasure`: text, seven resources, the artifact); `PLAIN_OBJECTS` gains the
Corpse, the Pyramid, the Learning Stone, the Treasure Chest, the University, the Wagon and the
Warrior's Tomb; `write_in(map, Format::RoE)` writes version `0x0E` (no level limit, the RoE
player block and hero body, 16 bytes of allowed heroes, one-byte codes, no Witch Hut body) and
`Codes` per format refuses what RoE cannot hold (a level limit, banned skills, a town, a hero
type, artifact or creature past 128, 127, 118, a hero's spells, primary skills or misc 5, a
Witch Hut's skills, towns past eight factions). The SoD bytes are unchanged (extras' golden CRC
holds); a monster of class 54 now needs a creature the format names. Tests: the layout of each
body, every refusal, and `tests/rewards.rs` reading both formats back (every body equal to what
was written) and importing them: the static layers validate; the SoD map stops at
`Unsupported::MonsterArtifact` and imports whole without it; the RoE map imports whole with RoE's
rules alone (`import::ROE`). Coverage 100 %.

**The spec** (`h3-experiment`, battle-owned, `cross-lane`): `[[map.artifact]]` (`artifact` or
`random` by class), `[[map.scroll]]`, `[[map.shrine]]`, `[[map.hut]]`, `[[map.pandora]]`,
`[[map.event]]` with `text`, `guards` and `gives` (a reward record by names); `grid` on them and
on `[[map.object]]`; an opponent monster's `treasure`; a hero's `fill_backpack` (to 64); `[map]
format` (`map_format`, the binary writes with `write_in`); `Tables::rewards` (the reward classes'
templates). `docs/experiments/README.md` documents them and lists batch E7 as staged.

**The specs** (compiled with the staged tables; `original_data` on the three maps: every static
layer imports and validates, E7b and the RoE map whole, E7a stops at E7-6's treasure artifact):
`E08-batch-7a` (72 × 72; E7-1 to E7-6, E7-8 to E7-11; Red 8 heroes, three of them only naming
the draw rows, Blue 2), `-7a-roe` (36 × 36, RoE; E7-7's 60 Witch Huts, two `defaults` heroes),
`-7b` (36 × 36, every skill banned but Wisdom and Learning; E7-12 to E7-23; Red 7 heroes, Blue
6). The run E7a is two maps (the plan's "row 7 on an RoE map of its own"); E7-5 and E7-10 visit
only the objects whose word holds an artifact and a few controls; a new game without an artifact
chest, Corpse or Wagon is restarted before playing. Not needed: the treasury setting the queue
named ("gold": no row buys anything), the University dialog's reads (E7-9 reads +0x4E3CC).

**The review** (a fresh agent against survey §11 and the sections the rows cite; eight findings,
all taken): (1) D-40's prediction ran the wrong way: a weak army's ratio gives n = 7, so **E7-16
predicts 5 (or 4) Gold stacks**, E7-22's strong army one stack of 40, as VCMI's (E7-16 settles
D-40); (2) the Pyramid is entered (0x63AA58 does not list 63): E7-22 walks off after its win, E7-21
steps off between its two visits; (3) E7-4's hero could not reach the stone on day 1: the stone
moved to (18, 12), the hero given Expert Logistics, a next day allowed; (4) E7-11's hero could
wear a Tomb's morale artifact: its backpack filled (the Tomb's −3 applies either way); (5) a 13 %
chance of no artifact chest among 40: the restart rule; (6) a Witch Hut blocks and is visited on
the tile west of its picture's corner: the RoE spec's labels moved there; (7) 0x63AA58 lists class
26 among the "not entered" classes against §7.3's stop on the event's tile: E7-17, E7-18 and E7-23
predict both (the record tells them apart; T58 settles it); (8) the spec's and the writer's docs
said guards with an empty text fight at once, the wiki's side of D-29: reworded (by class).

T57 (the harness and the runs) and T58 (the scoring) queued.

### Increment 7, round 2: T57, batch E7's harness and runs (2026-10-26/27, `adventure/e08-t57`)

**The cut** (decision 143): the user took T57 whole (recommended was the reads first, then the
runs). The University dialog's slots and purchase button left out: no row needs them (E7-9 reads
the game's Universities, +0x4E3CC).

**The reads** (`clients/capture/`, common; README "Rewards on the map"): `h3mem.object_at` (the word,
class and subclass of the object anchored at a tile, read on its entrance tile by the template's
mask, `entrance_cell` shared with `map_owners`; a University's entrance is left of its corner),
`universities` (game +0x4E3CC, four i32 skills each), `dialog_pictures` (a message box's picture
items with the text item starting at their bottom as caption; the choice frames 0x7809 and
0x780A around them, items of their own vtable, `h3mem.CHOICE_VTABLE`, observed at a chest), `open_tiles` (explored, empty, not
closed, not guarded), `map_kind` (game +0x1F698); a hero's spell points (+0x18), status flags
(+0x105), morale and luck bonuses (+0x11A, +0x11B); a battle stack's army slot (+0x5C).
`h3adv`: `object:x,y` decodes the word by class as `map.toml`'s visit functions read it (chest,
Corpse, Wagon, Tomb, Shrine, Witch Hut, University, Pyramid, artifact by subclass); `word:x,y`;
`choice` (a message box whose OK waits for a picture: the side's frame, then the OK; it stopped
the batch before); `message_pictures`; `level_up = "not <skill>"`; `level_up_dialogs`,
`treasury`, `offering:<skill>`, `opening:rows` (a `rows` record fills a table's unlabelled rows),
`morale:start`, `luck:start`; `before:` read right before the step (any observation); a list of
observations per field, `+<field>`; `each`/`select` grids (steps expanded as they come, nearest
first), `once`, `days` (a red target waits for the hero's next turn), `near` (an unexplored target
approached over open tiles: the pathfinder finds no path onto an unexplored tile, as the first E7a
run showed: E7-5's and E7-10's visits past the first got no path), `[[require]]` (a new game
started again, at most 20 times). The walk's view centres on the target when both ends do not fit.
A walk is got ready before its step reads anything (`ready`: the approach and the day waits, the
route set anew each day), so its `before:` reads never cross a day's start; the walk's view
centres on the target when both ends do not fit; `hover` moves the mouse through the next cell
when the game's tile under it is stale (below). README: "Rewards on the map", the observations'
table, two traps.

**The recipes** (`clients/capture/batches/`): `E08-batch-7a.toml` (the draw rows and the words of
E7-5's and E7-10's objects first, then Red's E7-1, E7-2, E7-4, E7-6, E7-5 and Blue's E7-10, E7-11;
three `[[require]]` grids, the spec's restart rule; E7-5 and E7-10 by `select`, `near` and
`days`), `-7a-roe.toml` (E7-7's sixty huts read on their visit tiles; the game's version noted),
`-7b.toml` (E7-12 to E7-23; E7-23 by `once`). The specs' `opening.py` is the battle record's
`opening` (the recipes say so; the specs keep the word).

**The spec's change** (`E08-batch-7a`): E7-5 and E7-10 got Expert Logistics, as T56's review gave
E7-4 (for the points: without it the third run took them into a sixth day, where it stopped);
the predictions do not depend on it. The compiled map is in the user's `experiments/out/`.

**The runs** (all on Easy, hot seat, under Wine; the results filled in `experiments/results/`):
E7a four times (the first stopped where a target 18 columns away fell outside the view; the second
found no path to the chests and Corpses past the first, all unexplored, whence `near`; the third
ran its day waits into a sixth day and stopped at a turn's start; the fourth, recorded, took five
days, no restart), the RoE map once, E7b three times (the first stopped at E7-23's last click: the
game kept the tile under the mouse from before the view moved, whence `hover`'s retry; the second
was stopped for a caption fix; the third recorded). The harness check twice (the first stopped in
Gear's plan battle, no attack cursor; the second passed: Win's chest 1 500 gold or 1 000
experience, the experience taken, 50 → 1 050; the Tomb's question, −3 and 0x200000, its artifact
kept with the full backpack).

**What the runs showed** (for T58; not scored here): E7-1 a question first (GENRLTXT 421; No left
the axe and fought nothing; Yes the battle, the axe then worn in the right hand). E7-2 the axe in
the right hand, the scroll at misc 1 (position 9), no spell learned. E7-3 forty different
artifacts. E7-4 525, 1 050 and 1 575 for chests of 1 000, 1 500 and 2 000, the stone 1 050,
Learning still Basic. E7-5 four artifact chests of forty, each offering 1 000 gold or 500
experience, the backpack 64 throughout. E7-6 the treasure's 1 000 gold given, the axe lost. E7-7
the sixty huts drew exactly the fourteen skills of 0xEFBF; game +0x1F698 read 2 (SoD) on the RoE
map. E7-8 fifteen different spells. E7-9 Necromancy in 4 of 30 Universities (the second run's
game: 1 of 30). E7-10 5 of 20 Corpses held an artifact (the second run's game: 9 of 20), each
giving 1 000 gold; 7 of 10 Wagons (4 of 10), each giving its drawn resources; the Tombs kept
their artifacts, −3 morale after the first and after the second. E7-11 −3 after each Tomb (status
0x200000), −3 at the opening, 0 and 0x0 after the battle. E7-12 Advanced Wisdom. E7-13 600 spell
points. E7-14 1 000 experience (no Learning bonus), Wisdom taken at level 2. E7-15 Fireball not
learned, Wisdom then taken. E7-16 one Pikeman against five Gold Golem stacks of 8 and two Diamond
stacks of 10 (slots 0 to 6). E7-22 two Diamond stacks of 10 and one Gold stack of 40; Dimension
Door lost with Advanced Wisdom, the Pyramid looted (bit 0 cleared). E7-21 −2 luck after both
visits (status 0x1000), −2 at the opening, 0 after. E7-17 Red's event fired with the hero standing
on its tile (1 560 points → 1 160), and was removed; on Blue's-only event the hero stood on the
tile and nothing fired. E7-18 +100 at each entry. E7-19 the axe lost, the box removed. E7-20 the
army dialog (GENRLTXT 36) with the three stacks; the Monks dismissed, the Wood Elves taken, the
rest lost. E7-23 no path to (15, 30) nor to (14, 30) (Red's event in Blue's corridor); a straight
path to (3, 30) over Blue's own event at (4, 30), and one ending on it.

**Traps found** (README, "Traps"): the game finds the tile under the mouse again only when the
mouse enters another of the view's cells; days waited cost every player's turn (E7a's third run);
the check's Gear battle stopped once.

### Increment 7, round 2: T58, batch E7 scored; A-23 withdrawn (2026-10-27, `adventure/e08-t58`, `cross-lane`)

**The cut** (decision 144; asked at the start): T58 whole with a re-read. **Baseline:** `master`
828a703 (#490, T57, merged dd3d2d1f; CI on `master` green); the sandbox's light gates passed (the
code verified by CI's run). `Heroes3.exe` staged from the user's PC for the re-read. `cross-lane`
for the batch's row in `docs/experiments/README.md` (battle-owned), as T38.

**At the start:** E7-23's corridor half tested nothing. Its targets (14, 30) and (15, 30) lay 6 and
7 tiles east of a hero of radius 5 (ADV-FOG-002: dx² ≤ 30), no Blue hero came near, and the
original draws no path onto an unexplored tile (ADV-MOV-008, T57's own finding): "no path" there is
the fog's. Its open half was valid: a straight path over Blue's own event.

**Batch E7 scored** (survey `E08-rewards.md` §15, from T57's results): every dispute's row as the
code reading, conclusive against VCMI or the wiki: D-29 (E7-1), D-30 (E7-2), D-31 (E7-3: forty
artifacts, no relic), D-33 with A-21 (E7-7), D-34 (E7-8: the fifteen spells of the first circle),
D-36 (E7-9), D-37 to D-39 (E7-12 to E7-15), D-40 (E7-16: G G D G D G G as the table 0x63DF94; E7-22
D D – G); the full-backpack rows (E7-5, E7-6, E7-10, E7-19) and the consensus rows confirmed. None
reverts; no `ADV-Q`. Two findings: a full backpack is tested before a free position (E7-6's axe
lost with the right hand free), and a visit sets the visitor's colour bit in the word's bits 5–12
(Blue's bit 6 after E7-16's retreat: the content as before). Corpses 5 of 20 (the recorded game)
and 9 of 20 (T57's second game): 14 of 40 against 20 % (about 2 %); the threshold read again
(0x4C113D, below 20) and the harness's bit right (bit 16): chance. Wagons 5 of 10 in the recorded
game by its words (T57's entry says 7).

**The re-read** (decision 131: "contradicted, A-23 is withdrawn and the reading re-read"):
`place_object_tiles` gives an event's entrance tile its class (+0x1E 26) and word but clears +0xD
bits 0x01 and 0x10 (`and word [tile + 0xC], 0xEEFF` at 0x505C78; `tile_refresh_around_heroes`
0x505A10 leaves the tile so); `pathfinder_expand` hands the entered tile's +0xD bit 0x10 to the
relax (0x4B2D12 to 0x4B2D22), which stores it as the node's stop flag (node +4 bit 1, 0x4B1EF1 to
0x4B1F04); `pathfinder_run` asks `pathfinder_pass_through` only of a stop node (0x56BB7C), with the
mouse's mode 0 (0x41955B's sixth argument). An event's tile is no stop, so the second reading's
"no for class 26" is never asked, and nothing tests a colour: a path crosses any event.
`event_fires_for` (0x4FCDC0) reads the class and the word, so an event fires mid-path at the step
that enters it and the move ends there (§7.3). The second reading's error: "the tile carries the
visitable bit", inferred from the removal, which clears it again. **A-23 withdrawn**: §8.5's blind
way is the rule in every profile, no flag, no `ADV-QUIRK-22`, no leak for `docs/06-security.md`;
no re-run (the other colour rests on the code, which tests none).

- **Docs:** survey §0, §7.3, §8.5 (rewritten), §8.6, §9, §12 (ADV-MOV-011), §13, §14, §15 (new);
  `docs/exe/map.toml` (0x4B1A70, 0x4FCDC0, 0x505B20, 0x56AB40, 0x56B440), `structs.toml`
  (`map_item` +0xD); `E08-design.md` §12, §16, §17, §19, §20; the ADR-0016 draft (points 4, 6, the
  consequences and alternatives); `docs/experiments/README.md`'s row (`cross-lane`); the lane's
  `TRAPS.md` (a path row's targets must be explored); the AI lane's inbox (A-23 withdrawn: the
  masks reveal nothing of events).
- **The catalog:** observations for HER-EXP-001 (E7-4; `verified`), HER-CHG-002 (E7-1, E7-2;
  `verified`), ADV-MOV-008 (E7-23's corridor). The new rules (survey §12) cite their rows when T60
  to T63 write them (§15 lists them).
- **For T59:** the drafts carry A-23's outcome (ADR-0016 point 4: `Reach` never reads events); the
  mana ceiling and A-21's flag as design §17.

### Increment 7, round 2: T58b, `h3-map`'s view property's random failure; A-25 (2026-10-27, `adventure/e08-t58b`)

Decision 144 (next after T58, the user: "You can fix the random test failure now"), in the same
session after #494 merged (2969de0). **The failure** (the queue's inbox, T57's scoped verify):
`views_move_as_the_events_say` with the seed `cc debe5cd4…` reproduced at once. In `strict-fixed`,
player 3's end of turn started day 2; the new day's reveal (E08 T40, `ADV-QUIRK-18` off: each hero
reveals around it to its owner) widened player 2's hero's radius, grown to 8 by an Expert Scouting
taken since its last step, onto (6, 2), which went from the fringe to seen, with player 1's hero
standing on it. No event names a hero shown by a reveal, as for one the viewer's own move reveals.
**Not a leak** (the tile is explored, ADV-FOG-004), **the oracle's gap**: `check_told` let the
planes grow at a new day (T40) but let a hero show without a step only after the viewer's own
command. Now `own || after.day != before.day`, with the tile's not-seen-before test as it was;
the seed committed with a comment. `PROPTEST_CASES=1000` of the property passed.

**A-25** (ADV-Q30, decision 145; the user on T58's report: "sounds like a bug to me"): a full
backpack tested before a free position is a quirk with a flag, `faithful` the original's,
`strict-fixed` the placement first; survey `E08-rewards.md` §15, design §17 (`ADV-QUIRK-22`, flag
38 if free), T59's queue entry.

### Increment 7, round 3: T59, increment 7's shared part (2026-10-27, `shared/e08-t59`, `cross-lane`)

**The cut** (decision 146; asked at the start, `AskUserQuestion`): T59 whole, one `shared/` pull
request, as T45; not chosen: the docs half or the code half alone (a session's shared changes go
in one pull request, so a split takes two sessions). **Baseline:** `master` 18f3cb1 (#496
merged; CI green on it; the sandbox's light gates passed, the code verified by CI). The user's PC
was linked: the original data staged from the repository's `original-data` (the executable, both
LODs, `maps.tar`, `extracted.tar`).

- **The ADRs** (the three accepted drafts applied, then deleted with their folder): ADR-0013
  points 1 (the battle bonuses and once-marks), 2 (four hero changes; rewards among what is
  derived once) and 15 added (the bonuses as the hero's own effects, the reward record's hero
  half with the mana ceiling, what a teacher may teach, receiving an artifact); ADR-0016 points 1
  (increment 7's block: emptied, the army left, the once-per-hero marks by hero id, the players'
  knowledge), 3, 4 (the 31 classes' kinds, events blind to the reachable-tiles query, visits'
  refusals, the draws at the build, A-21) and 6 (knowledge, events never observed); ADR-0017
  points 1 (the human flag, the bans), 4, 6 (answers typed by decision, pointed to from
  increment 2's sentence), 8, 9, 10 and 11 (T54's setup facts, increment 7's guards, the bonuses
  cleared); each ADR's consequences, alternatives and history; the index's three Read-when
  cells. **A-25's flag written in** (decision 145 came after the drafts): ADR-0013 point 15.4 (the
  count first with the flag on, the placement first with it off), point 15.2 and ADR-0016 point
  4's refusals point to it. A-23's withdrawal was already in the drafts (T58).
- **The mana ceiling**: `HeroParams::mana_ceiling` and `HeroParamsDef::mana_ceiling` (`u16`, as
  the details' mana) in `h3-content`, hashed after the schools: **core ruleset hash v49** (the
  golden base hash `0xc62d83f6389d8ef4`); `h3-pack`'s `[hero_params]` requires it: **pack format
  v48** (`the_mana_ceiling_parses_and_is_required`: 999 parses, missing or 65 536 is a syntax
  error, the harvested base holds 999); the synthetic pack 450 (above its 40 × 10 maximum, as the
  original's 999 above 990). The harvester (battle-owned, `cross-lane`): the gap-fill's
  `[heroes.params]` `mana_ceiling = 999` with the source `exe-mana-ceiling` (`grant_rewards`
  0x49FA90, survey §7.4, §13, batch E7's E7-13), written into `[hero_params]`.
- **The flags**: `quirk.adventure.roe_witch_hut_mask` (37, `ROE_WITCH_HUT_MASK`, `ADV-QUIRK-21`)
  and `quirk.adventure.full_backpack_refuses` (38, `FULL_BACKPACK_REFUSES`, `ADV-QUIRK-22`) in
  the registry, exported, on in the harvested `base:faithful` and off in `strict-fixed`
  (`the_harvested_flags_of_increment_7`; the synthetic packs name no adventure flag, so both are
  off there); the two entries in `docs/rules/quirks-adventure.md` (regression tests with T60 to
  T62, which read them). `base` re-harvested from the staged installation: three lines changed
  (the ceiling, the two profiles).
- **Replays and records**: the 22 golden replays keep commands and state hashes; only
  `ruleset_hash` in their headers moved (synthetic faithful `0xf957d87efe3a32c6`, base faithful
  `0x026155eb7308da0f`, strict-fixed `0x242432fb9f5bf302`), each verified by `h3 verify` and
  `h3 adventure verify`. The AI lane's `bots-report-v1.json` and `bots-neutral-v1.json`
  regenerated (`h3 ai eval --json`, release). The run records (`t6b-1`, `t6d-1`) keep the hash
  they ran on.
- **Docs**: `docs/03-domain.md`'s glossary (battle bonuses and once-marks, the mana ceiling, reward
  record, map event, once-per-hero mark, knowledge); `packs/README.md` (v48's field, the base
  pack's 999); the design note's §1, §3, §4, §6 and §17 (what T59 built removed, A-25's flag in
  `can_receive_artifact`, the drafts' points now the ADRs'); the epic file trimmed under its
  35 KB aim (decisions 125–128, 137, 142, 143, the cuts of done tasks, moved verbatim to
  `E08-archive.md`); decision 146 and the task's line; the queue (T59 out, T60 queued from design
  §19); the battle lane's inbox and HANDOFF versions, the AI lane's inbox.

### Increment 7, round 4: T60a, the battle bonuses and the four changes (2026-10-27, `adventure/e08-t60a`, `cross-lane`)

**The cut** (decision 147; asked at the start, `AskUserQuestion`): T60 in two, T60a here (what
reaches outside `h3-heroes`: the field, the hash, the replays, the other lanes' literals), T60b
next (`grant` and the teachers, inside the crate, no version); not chosen: T60 whole, or three
parts. The other lanes' literals with the user's consent (`cross-lane`). **Baseline:** `master`
46f7d62 (#497 and #498 merged; CI green on 1c99f41, the code's last run; the sandbox's light
gates passed, the code verified by CI). The user's repo folder fast-forwarded to `master` (one
stale lock of the session's own fetch removed). No original data staged (none needed).

- **The data** (design §1; ADR-0013 point 1): `h3_heroes::BattleBonuses { morale: i8, luck: i8,
  marks: u8 }` (`Default` none; `MARKS` 8 kinds; `has_mark`, false for a mark of no kind),
  `HeroDetails::bonuses`; `validate` accepts any value. The map's import (`h3-game`'s setup)
  gives none.
- **The changes** (design §2; ADR-0013 point 2; HER-CHG-001): `GainPrimary([u8; 4])` (each
  added, at most `primary_max`, saturating), `RaiseSkill { skill, level }` (a known skill below
  the level rises to it in its place, one at or above it stays; a new one at the level after the
  others; checked: the ruleset's skill, else `CannotLearn`; Basic to Expert, else the new
  `ChangeError::SkillLevel`; a new one only below eight, else `CannotLearn`), `Bonus { morale,
  luck, mark: Option<u8> }` (both added, saturating; with a mark only while it is clear, then
  set; a mark of 8 or more the new `ChangeError::Mark`), `ClearBonuses` (all to none). The
  emitter leaves out a change that would do nothing (the design's rule; the check passes it).
- **In battle** (design §5; HER-ADV-008): `morale_and_luck` takes the bonuses and adds them to
  the own side's morale and luck sums, so the battle gets one effect of each as from any source
  (a sum of 0 left out, as before). The clearing at a battle's end is the game's (GAM-BAT-014,
  T63).
- **The hash**: `h3-game`'s `hash_details` adds morale and luck (`write_i32`) and the marks;
  `GAME_STATE_HASH_VERSION` 15. A test-only `GameState::apply_hero_change`.
- **Replays**: the twelve adventure goldens re-recorded with their options (the ten duels with
  their seeds, days, profile and difficulty; the two scripted with `--script` of themselves),
  their commands unchanged, every checkpoint and final hash moved, the header's version 15;
  `adventure_replays.rs`' history. The battle replays are untouched (no battle hash changes:
  every bonus is 0).
- **Cross-lane** (the user's consent, decision 147): `bonuses: Default::default()` in the battle
  lane's `h3-pack-battle` tests (`src/tests.rs`, eleven `tests/harvested_*.rs`) and the AI lane's
  `h3-sim` encounter (`encounter/hero.rs`); both inboxes told. The lane's own: `h3-adventure`'s
  `testing.rs` and two tests, `h3-map`'s `e4_score` example and `play.rs` (a battle tells none of
  the new changes).
- **Tests**: `primary_skills_gained_saturate_at_the_limit`,
  `a_skill_rises_to_its_level_and_never_falls`,
  `bonuses_add_up_saturate_and_once_marks_gate_them` (E7-11's two Tombs −3, E7-21's two looted
  Pyramids −2, saturation both ways, the bad mark, the clearing, any value valid),
  `the_new_change_errors_have_texts` (`tests/rewards.rs`);
  `the_battle_bonuses_join_the_own_sides_morale_and_luck` (−3 with Leadership's +1 and the
  circle's +2: left out; design §5's −2; the bonuses alone; cleared, none) and the derivation
  property with random bonuses (`tests/derive.rs`); the hash test covers morale, luck and a mark
  (`h3-game`'s `the_store_is_in_the_hash_and_shared_by_forks`).
- **Rules**: HER-ADV-008 (new, implemented: the code reading, batch E7's E7-11 and E7-21, the
  wiki, VCMI); HER-CHG-001's summary (the four changes).
- **Docs**: the design note's §1, §2 and §5 (what T60a built, removed; §5's game half stays for
  T63), §6, §17; decision 147 and the tasks' lines; the queue (T60b); the HANDOFF; the battle and
  AI lanes' inboxes.
- **Verify** (scoped, no original data staged): every step passed but clippy (an unused import in
  the new tests, fixed, clippy re-run clean over the lane's crates and `h3-pack-battle`, `h3-sim`,
  `h3-cli`) and `check-lane` (`cross-lane` by design; `check-lane --cross-lane` ok); coverage,
  determinism and bench-check passed.

### Increment 7, round 4: T60b, the reward record's hero half and the teachers (2026-10-27, `adventure/e08-t60b`)

**The cut** (decision 147): T60b whole, inside `h3-heroes`, no version; nothing to split (the
previous session's estimate held). **Baseline:** `master` c637652 (#500 merged as 40151d3, CI
green on it; c637652 a client change); the sandbox's light gates passed (the code verified by
CI). No original data staged (none needed).

- **`grant`** (design §3; ADR-0013 point 15.2; HER-CHG-003): `h3_heroes::grant(details, ruleset,
  &HeroReward) -> Granted` on a scratch copy, each change checked and applied before the next
  part is tested. `HeroReward` holds the map's values (`experience: u32`, read as signed;
  `primary: [i8; 4]`; `skills`; `mana: i32`; `morale`, `luck: i8`; `artifacts`; `spells`),
  `Granted { changes, experience, lost, skipped }`. The experience through `learning_experience`
  of the details before the record's skills (D-39), not a change. The engine's readings where the
  design was silent: a skill pair that changes nothing (known at or above its level) or that the
  ruleset refuses gives no change; a mana of 0 gives none (a hero above the ceiling keeps its
  points then); an artifact the ruleset refuses as received (a Spell Scroll without a spell, one
  it lacks) is lost.
- **`can_receive_artifact(details, ruleset, ArtifactInstance)`** (ADR-0013 point 15.4): the
  `AddArtifact` check (HER-CHG-002), and with `FULL_BACKPACK_REFUSES` the backpack below its
  most too. Under HER-CHG-003 (every giver calls it); `change::has_space` made `pub(crate)`.
- **The teachers** (design §4; ADR-0013 point 15.3; **HER-CHG-004**, an id beyond survey §12, so
  that HER-CHG-003 stays the record's): `can_learn_spell(details, ruleset, spell, known_counts)`
  is the design's "`learns_spell`'s known-counts form" under its own name, so `learns_spell`
  (Eagle Eye, HER-SPL-006) and its caller in `h3-game` stay as they are (it delegates with
  `true`); `hut_teaches`; `university_slot -> SlotState { Known, NoRoom, Barred, Open }` in the
  design's order (no room before the class's weight; a hero type or skill the ruleset lacks:
  barred); `scholar_reward(details, ruleset, ScholarGift, fallback)` with `ScholarGift {
  Primary, Skill, Spell }` for the design's kind and id (a primary gift +1 to itself, the
  fallback for a spell or skill the hero cannot take); `library_admits(details, ruleset, level)`
  (the Diplomacy level read as `join_bonus`, which only Diplomacy gives; no skill by name);
  `learning_experience` (= `gained_experience`, HER-EXP-001's single computation);
  `next_level_experience` (the gap between the level's and the next level's thresholds; 0
  without the hero parameters).
- **Tests** (`src/tests/grant.rs`, a pack with Wisdom, Learning and Diplomacy stand-ins, a worn
  book, spells of levels 3 and 5, class weights and both profiles):
  `a_record_is_granted_in_the_original_order` (E7-13's 600 points, E7-14's 1 000),
  `the_experience_is_signed_and_counts_the_learning_known_before`,
  `skills_rise_never_fall_and_a_new_one_needs_room` (E7-12),
  `the_mana_is_held_between_zero_and_the_ceiling`,
  `the_bonuses_and_primary_skills_are_given_only_when_set`,
  `a_spell_needs_the_book_the_record_may_give`,
  **`a_full_backpack_refuses_an_artifact_in_faithful_only`** (`ADV-QUIRK-22`'s regression test in
  both profiles: E7-5, E7-6, E7-10, E7-19's situation), and the teachers' five.
- **Rules**: HER-CHG-003 and HER-CHG-004 (new, implemented: the code reading, batch E7, VCMI, the
  wiki); the ledger's `ADV-QUIRK-22` names its test.
- **Docs**: the design note's §3, §4 (built, shortened), §6, §11's Pyramid; the tasks' lines;
  the queue (T61); the HANDOFF; `OVERVIEW.md`.
- **Verify** (scoped, no original data staged): every step passed but check-rules, which ran
  before the catalog had HER-CHG-003 and -004 (the markers' ids unknown); re-run after the
  catalog's entries, ok, with check-docs, check-exe-map, check-stubs and doc-drift; coverage,
  determinism and bench-check passed.

### Increment 7, round 5: T61a, the import and the kinds (2026-10-27, `adventure/e08-t61a`)

**The cut** (decision 148, asked at the start): T61 in two as the design proposed, T61a the
import and the kinds; the agent's recommendation was three (the kinds alone first). **Baseline:**
`master` 94a5039 (#504 merged, CI green on it; 94a5039 a map-client change); the sandbox's light
gates passed (the code verified by CI). No original data staged: the user's computer was not
reachable when the census was due (below).

- **The kinds** (design §8; ADR-0016 point 4; ruleset part v12): `ObjectKind::Chest(Chest)`,
  `Artifact { classes: Option<Vec<ArtifactClass>> }` (none: the map names it), `Scroll`,
  `Gift(Gift)` (`Gives::Primary`, `Choice { skills, amount }`, `Experience`, `Level`; `prices`,
  none free, several drawn; `admits`, the Library's level for `library_admits`; `leave`),
  `Shrine { level }`, `WitchHut { roe, roe_fixed }` (A-21's two lists, flag 37 picks one at the
  draw), `Scholar`, `University { offers, price }`, `Find(Find)` (`draws` by chance:
  `FindContent::Nothing`, `Artifact(classes)`, `Resources { amount, resources }`; the full
  backpack's `Turn::Give`, `Drawn`, `Stays`; `question`; `bonus: OnceBonus`), `PandoraBox`,
  `Event`, `Pyramid(Pyramid)` (`split`, `fixed` stacks, `level`, `looted`). Their defs with keys
  in `ruleset/rewards.rs`, every parameter checked (chances, amounts against `MAX_GAIN`, classes
  distinct and never Special, marks below `BattleBonuses::MARKS`, prices of distinct resources);
  `AdventureCompileError::UnknownSkill`; a Pyramid's fixed stacks leave the split creature a slot,
  checked once the parameters are in (`check_object_types` now runs after them). The hash's tags
  8 to 19. `ObjectKind::effect` gives none for them: their visits are T62's, so a step onto one
  is refused as a blocker's was. The pack format (`h3-pack-adventure`): one table per kind
  (`chest`, `artifact`, `scroll`, `gift` with `gives` as `{ primary = [...] }`, `{ choice = {...}
  }`, `{ experience = n }` or `"level"`, `shrine`, `witch_hut`, `scholar`, `university`, `find`
  with `holds`, `pandora_box`, `event`, `pyramid`), primary skills and classes by name. The base
  pack's 31 types with their sources (survey §2 to §7), the visit modes from the original's two
  lists (0x63AA58, 0x63AB00).
- **The import** (design §7; map import v12): `codes.rs` types the 31 classes (5 by any subclass,
  its artifact). `ObjectSetup::holds` (`Holds::Nothing`, `Artifact(ArtifactInstance)`,
  `Spell(Option<SpellId>)`, `Skills(Option<Vec<SkillId>>)`, `Gift(Option<ScholarGift>)`,
  `Record(Box<RewardRecord>)`, `Event(Box<EventSetup>)`), `RewardRecord { hero: HeroReward,
  resources: Vec<(ResourceId, i32)>, creatures: Vec<ArmyStack> }`, `EventSetup { record,
  colours, computer, remove }`, `MonsterSetup::artifact`. Guards from the message records of
  artifacts (always asking, D-29), scrolls (asking with a text), boxes and events (never asking);
  a random stack among them unsupported, kept unguarded by `import_map`. A record's creatures with
  a random code or more stacks than the army's slots unsupported (`Unsupported::RewardArmy`, A-24),
  kept without them; `Unsupported::MonsterArtifact` gone (the 8 maps move on).
- **The setup** (`state/holds.rs`; `SetupError::Holds`): each holding against its kind; a placed
  artifact and a scroll one a hero may hold (`h3_heroes::validate_artifact`, new: HER-ART-003's
  test of one item), the scroll's with a spell, the artifact's without; a Shrine's spell and a
  Scholar's a hero spell; a Witch Hut's skills distinct; a record's skills at Basic to Expert,
  artifacts known (a Spell Scroll without a spell is a valid gift: `grant` loses it), spells hero
  spells, resources distinct and not 0, creatures an army's without the "no stack" error and no
  war machine. Guards on artifacts, scrolls, boxes and events too (ADV-PIK-004). A monster's
  treasure artifact one a hero may hold. `MapObject::holds`, `Monster::artifact`; the static
  map's hash adds both (adventure state hash v13).
- *Agent's calls (change freely):* one `Holds` enum on `ObjectSetup` rather than a field per
  kind (40 struct literals took one line each); a scroll holds its `ArtifactInstance` (the import,
  which alone knows code 1, resolves the Spell Scroll), so `ObjectKind::Scroll` has no parameter;
  an empty Witch Hut mask imports as every skill (the original's draw reads it so, 0x4C207D), an
  RoE map's as none; a Scholar kind other than −1 to 2 and a primary code above 3 are unknown
  codes; a record's stack of 0 creatures stays for the setup to refuse (as an army's); the hero
  model's types the setup names (`ArtifactInstance`, `HeroReward`, `PrimarySkill`, `ScholarGift`)
  re-exported by `h3-adventure`, since `arch.toml` gives `h3-map` and `h3-pack-adventure` no edge
  to `h3-heroes`; the bans moved to T61b (decision 148); a typed event still blocks a walk until
  T62's event tile (design §12), as the blocker it was.
- **Tests**: `ruleset/tests/rewards.rs` (every kind compiled, every check, the references, the
  hash per parameter), `state/tests/holds.rs` (each holding against each kind, the records'
  checks, guards by kind, a monster's artifact, the hash), `h3-pack-adventure`'s
  `parses_the_reward_kinds` and `the_base_pack_types_the_rewards_of_increment_7`, `h3-map`'s
  `increment_7_bodies_import_with_their_guards_and_records` and the writer's reward maps (T56's)
  now importing whole, `h3-heroes`' `validate_artifact` cases.
- **Replays**: the twelve re-recorded with their options, their commands unchanged (neither map
  holds increment 7's classes): every hash moved; the headers' import v12, adventure state hash
  v13, the ruleset part's hash.
- **Rules**: ADV-OBJ-002, ADV-PIK-004 and ADV-MON-001 amended; the visits' ids (ADV-PIK-005 and
  on) stay for T62. `docs/formats/h3m.md`, "Import" (v12).
- **For other lanes**: the AI lane's inbox: `h3 ai census` counts every object with guards as a
  guarded pile; since T61a artifacts, scrolls, boxes and events carry guards too. **Cross-lane**
  (the user's consent, decision 148): `holds: Default::default()` and `artifact: None` in the AI
  lane's `h3-sim` adventure tests (`src/adventure/tests.rs`, `tests/monsters.rs`; three lines).
- **Not run**: `--with-original-data` (the user's computer was not reachable to stage
  `maps.tar`): the census of what imports whole after T61a and whether every static layer still
  validates with the 31 classes' bodies (survey §1 found nothing out of range) are the next
  session's with the original data, before T61b builds on it.
- **Verify** (scoped, no original data staged): every step passed but check-lane, which flags the
  two AI-lane test files (the consented `cross-lane` lines); coverage, determinism and bench-check
  passed; check-docs, check-rules, check-stubs and doc-drift re-run after the docs, ok.

### Increment 7, round 5: T61b, the bans, the draws and the state (2026-10-27, `adventure/e08-t61b`, `cross-lane`)

**The cut** (decision 149, asked at the start with the census's result): whole; the agent's
recommendation was two parts (the state apart). **Baseline:** `master` c94a5ab (#507 merged, CI
green on it); the sandbox's light gates passed (the code verified by CI); the original data staged
from the user's computer (`maps.tar`, `extracted.tar`, the executable and the two LODs).

- **The census first** (T61a's import v12, then v13): every map reads, every static layer imports
  and validates; 8 import whole (`Myth and Legend`, `Pandora's Box `, `Unholy Quest` join T49a's
  5); the 8 refused for a monster's artifact before: 3 whole, 3 a random hero, 2 a hero in a town;
  128 a hero generated at a main town, 21 a random hero, 3 a hero in a town. The example's
  "guarded piles" counted every guarded object since T61a (171): now by kind (36 piles as T21a's,
  33 artifacts, 3 scrolls, 12 boxes, 87 events), and the bans (26 maps ban spells, 111
  artifacts). Recorded in `h3m.md` "Import".
- **The bans** (GAM-SET-003, map import v13): `GameSetup::banned_spells`, `banned_artifacts`
  (`GameSetupError::BannedSpell`, `BannedArtifact`, after the banned skills); `h3_adventure::Bans`
  on `AdventureSetup` (skills, spells, artifacts), which only the build's draws read; the import's
  spell set and artifact set, the latter within the edition's artifacts the codes name: the SoD
  editor's default bans the unused records 141 to 143 (123 bits on 41 shipped maps), which an
  error would have refused on every such map. `cross-lane` (consented): the AI lane's one full
  `GameSetup` literal (`census.rs`), two default lines; the queue named `ai/tests.rs` too, whose
  literal is the experiment writer's `Map` (a first edit there broke its build, which the
  workspace check caught before the pull request); the AI lane's inbox told.
- **The draws** (design §9; `state/draws.rs`; adventure state hash v14): `MapObject::resolved:
  Resolved` (`Nothing`, `Artifact`, `Price`, `Spell`, `Skill`, `Scholar(gift, fallback)`,
  `Skills`), a chest's gold and a find's resources in `contents`; after the piles and the monsters,
  in two walks in object order: the random artifacts (`ArtifactPool`: marks from the bans and the
  placed artifacts; a class out restarts the marks from the bans, then widens to every class but
  Special, so widened draws may repeat, as 0x4C94D0), then chests, a gift's price (the Tree),
  Shrines (`ShrineMarks`, per level), Witch Huts (the mask less the bans in id order; an RoE hut by
  flag 37), Scholars (an empty pool gives a primary skill), Universities, finds (the Wagon's
  resources beside its artifact) and the Pyramid. A hero spell always has a school (the core's
  compiler), so the pools test the category, the bans and `granted_by_name_only`. Rules
  ADV-OBJ-006, ADV-PIK-005, ADV-PIK-009 to -012, ADV-TCH-003 to -008 (`implemented`; their visits
  T62); `ADV-QUIRK-21`'s regression test; A-18, A-20, A-22 and A-24 as `ADV-QUIRK-23` to `-26`
  (exploit-fix, no flag; design §17 had them in the ledger, which T59 did not write).
- **The state** (design §10): the army left as `persistent::Table` (a top table of mid tables of
  64 leaves of 16 entries, each value shared; a part without an entry is none; the hash's
  encoding unchanged); `Visits` (each hero's marks by hero id, each player's knowledge, the emptied
  objects, a bit per object index in `Occupancy`'s grid shape), a public value whose methods are
  its API, since the lints refuse crate-private code without a caller (T62's visits write it);
  `AdventureState::visits` none until then, hashed after the army left (0 or 1 and its encoding).
  `fork-xl-144`: 3 587 Ir a fork against 3 564 (+0.65 %); `random-play-duel-36` −0.50 %.
- **Replays**: the twelve re-recorded with their options, their commands unchanged (neither map
  has an object that draws, nor bans): every hash moved.
- **Gates**: the scoped verify with the original data (`check-lane` flags the AI lane's file, the
  label `cross-lane`), then `cargo check --workspace --all-targets` and `h3-cli`'s and `h3-sim`'s
  tests (the scoped verify builds neither the AI lane's tests nor every target); coverage closed three gaps on the way (a test helper's early returns,
  the pool's out-of-range bans, the two new errors' texts); `check-arch` refused a `sort_unstable`
  (the hut's candidates now come in id order from the unbanned skills).

### Increment 7, round 6: T62a, the pickups, the teachers and the finds (2026-10-27, `adventure/e08-t62a`)

**The cut** (decision 150, asked at the start): T62 in two as designed, T62a the pickups, teachers
and finds; and `h3-game` applies the simple parts now (the agent's recommendation both times).
**Baseline:** `master` 54b9264 (#511 merged, CI green on it); the sandbox's light gates passed (the
code verified by CI); the original data staged from the user's computer.

- **The outcome** (`command.rs`; ADR-0017 point 10's increment 7): `AdventureOutcome::hero`
  (`HeroGains`: the changes in order, then the experience the game gives last), `choice`
  (`Choice` with `ChoiceKind::Chest { gold, experience }` or `ChoiceKind::Primary { skills,
  amount, price, leave }`; `allows`, `default_pick`, `price`), `university` (`UniversityVisit`);
  `Pick` (first, second, leave), the adventure's answer that T63's typed answers map onto;
  `OfferKind::Gift { price }` and `OfferKind::Search`.
- **The visits** (`state/rewards.rs`, dispatched by `ObjectKind::effect`'s new `Effect`s; the
  boxes, events and the Pyramid still none): a chest (an artifact the hero receives, else the
  choice; removed with the visit); an artifact or a scroll (refused while the hero cannot receive
  it, then its guards as a guarded pile's, then `pick_up`, which a won battle's end also calls);
  a once-per-hero gift (the mark, the Library's condition, the Tree's drawn price known, a choice,
  an offer to pay or the gift at once; `give_gift`, `accept_gift`; `choose` marks a gift's pick);
  the Shrines, Witch Huts, Scholars and Universities; the finds (`visit_find`, `accept_search`,
  `search`: the knowledge, the artifact or its turn, emptied, the Tomb's bonus whatever it held). A
  monster's treasure artifact (`treasure_artifact`) after a win (`end_monster_battle`) or a join.
  `accept_offer`, `end_unowned_battle` and `end_monster_battle` take the store of heroes (and the
  latter two the hero) and the battles' ends return an outcome. `Resolved::artifact`, `skills`.
- **The state:** the visits' block written through `visits_mut` (built on its first write, copied
  when a fork shares it), `learn` writing only what the player did not know (a fork's block stays
  shared on a second visit of a Shrine: tested); `AdventureState::is_marked`, `knows`,
  `is_emptied`. The adventure state hash's encoding unchanged (v14 hashed the block already).
- **The views** (ADV-VIS-006): `AdventureObservation::known` (`KnownObject`: type, anchor,
  `Known::Spell`, `Skill`, `Price`, `Skills`, `Emptied`), `OwnHeroView::marks`.
- **The game** (decision 150; part of design §15 moved from T63): `settle_outcome` gives the
  hero's gains (`give_hero`: each change through `change_hero`, then the experience with
  `give_levels`) and answers a choice with its default (`choose_by_default`: the chest's gold, the
  Arena's first skill, a School left; a priced default counts as left); the University is left at
  once; a battle's end against an object or a monster settles its outcome (a won guarded artifact's
  pickup before the winner's levels). All three `ADV-STUB-10` until T63. Game state hash v16 (the
  two offer kinds, as v8 for the pile's question).
- **Rules:** ADV-PIK-006, -007, -008, ADV-MON-011 (`verified`: E7-1, E7-2, E7-6), ADV-TCH-001,
  -002, ADV-VIS-006 (new, `implemented`); the visit halves of ADV-OBJ-006, ADV-PIK-005, -009 to
  -012, ADV-TCH-003 to -007; `ADV-QUIRK-22` linked from the rules that read
  `can_receive_artifact` and tested in both profiles (the chest, the artifact and scroll, the
  monster's artifact); A-17 as `ADV-QUIRK-27` (exploit-fix, no flag); `ADV-QUIRK-23`'s visit test.
- *Agent's calls (change freely):* a chest whose artifact draw found none offers the 1 000 gold's
  choice, as for an artifact the hero cannot receive; a random artifact whose draw found none
  leaves the map at its visit, giving nothing; a Tomb whose artifact stays (a full backpack) is
  not emptied, an empty find is; the knowledge is written only where a visit tells something
  drawn (the Shrines, the Witch Hut, a gift's drawn price, the University, the finds; not a
  School's, whose skills and price are its type's, nor a gift of one price); a find's emptying
  shows as it stands now to a player who knows it; the University's purchases (the slot's state,
  the skill at Basic) are left to T63 with `h3_heroes::university_slot`; the offers named
  generically (`Gift`, `Search`) rather than the design's `PayTree`, `SearchTomb`; a hero without
  details in the store (the adventure's own tests) receives nothing and gains no experience.
- **Tests:** `h3-adventure`'s `state/tests/rewards.rs` (18: each kind through `MoveHero`, the
  choices, fabricated offers naming no gift or find, the marks and knowledge in the views, the
  block's sharing); `draws.rs`' rules builder shared (`rules_with`); `h3-game`'s
  `tests/rewards.rs` (the experience and its level, the defaults, the gift's offer paid and
  refused, the Tomb's question and morale, a won guarded artifact and a beaten monster's artifact
  through `end_battle`); `h3-map`'s play property's answered offers (no plan lays out the new
  kinds yet: T64).
- **Replays**: the twelve re-recorded with their options, their commands unchanged (neither map
  has a gift or a find): every hash moved.
- **Gates**: the scoped verify with the original data passed every step but two, both fixed and
  passing alone since: `check-rules` (the catalog was written after the run started) and coverage
  (the two new offer kinds' arms of the game's hash: the open offers hashed in the game's tests);
  `bench-check` at its baselines (no workload's map holds the new kinds). Then `cargo check
  --workspace --all-targets` and `h3-sim`'s, `h3-cli`'s and `h3-map`'s tests. A smoke run, not
  committed: random play on the eight shipped maps that import whole (three seeds each, 14 days,
  at most 3 000 steps), no failure.

### Increment 7, round 6: T62b1, the reward record, Pandora's Box and the Pyramid (2026-10-27, `adventure/e08-t62b1`)

**The opening reading** (design §12's open point, before the cut; survey §8.5, `docs/exe/map.toml`):
`hero_move_step` flags a firing event at 0x4811A3, lets the tile's guard visit at 0x48169D
(`after_step_guard_check` → `guard_attack`: the join and flight offers or the battle with its
levels) and fires the event only after it, at 0x481741, on the frame's flag alone. So **the guard
comes first**, and the fire does not look at how the guard's visit ended: a hero that lost or
retreated is already in the heroes' pool with owner −1 (0x4DA2B5) and still gets the event, its
resources added to "player −1" outside the player table (`give_resource` 0x4E3612). **A-26**,
state-corrupting: always fixed, no flag (`ADV-QUIRK-29` in T62b2). A flying hero stopped by a
firing event mid-move is at its move's end for the guard test, so its tile's guard attacks there.
**The cut** (decision 151, asked after the reading): T62b in two, the rest staying in T63 (the
user's choice; the agent recommended four parts with the rest moved first): T62b1 the record, the
box and the Pyramid; T62b2 events and the move. **Baseline:** `master` 240cf26 (#516 merged, CI
green on it); the sandbox's light gates passed; the original data staged from the user's computer.

- **The outcome** (`command.rs`): `AdventureOutcome::losses` (a record's negative resources);
  `NoRoom { hero, group }` (a join's one stack in slot 0, a record's stacks without room at their
  slots: the original's copy of the record's army, each joined slot cleared, 0x4A09BB);
  `OfferKind::OpenBox`, `OfferKind::SearchPyramid`; `Defender::Monsters { split, fixed }`.
- **The split** (`split.rs`, ADV-MON-012): `split_with_second` (the strength summed, the stacks
  less the second's, at least 1, at most the count, no upgrade and no second draw, the order of
  `monster_stack_order` 0x63DF94's rows 7n for n = 1 to 6, read whole: rows 7 to 42 hold the
  permutations, the row ignores the second creature's stacks); `split`'s count and even stacks
  factored out (`stack_count`, `even`), its draws unchanged.
- **The visits** (`state/records.rs`, dispatched by `Effect::PandoraBox` and `Effect::Pyramid`):
  `ask` (the question at every visit); `open_box` (the guards' battle at once through `piles.rs`'
  new `guards_battle`, which `fight_or_pick` now calls, else `give_box`: the record, then the
  removal); `give_record` (`h3_heroes::grant` against the store's details, the resources split into
  gains and losses, the creatures through `monsters.rs`' `add_stack` with `CreaturesJoined` to the
  owner, the rest one group); `search_pyramid` (the owner learns it, a looted one's bonus, else the
  battle against `Defender::Monsters` at its tile); `loot_pyramid` (emptied, the spell by
  `can_learn_spell(…, false)` and not yet known, else lost). `end_unowned_battle` dispatches by the
  kind at the tile: a monster's end, a won Pyramid's loot, a guarded object's survivors, then a
  box's record or a pickup.
- **The game:** `settle_outcome` takes the losses (`lose`: `Treasuries::pay`, never below 0,
  `GameEvent::ResourceLost` to the owner); `open_army_dialog` opens over the group, one
  `ArmyDialogOpened` per stack in slot order (the event stays `Copy`); `start_battle` splits
  `Defender::Monsters` with the tile's draw under `ADV-QUIRK-6`, else red's stream; `give_levels`
  draws nothing while the hero's level-up is open (a won box's record gives experience before the
  battle's own levels: without the guard a level would have been drawn twice). Game state hash
  v17 (the two offer kinds, 8 and 9).
- **Interim orders** (`ADV-STUB-10`, decision 151): a won box's record and a won Pyramid's spell
  before the winner's levels; a record's experience and its levels before its army dialog closes.
- **Rules:** ADV-RWD-001, ADV-PIK-013, ADV-MON-012 (new, `verified`: E7-12 to E7-16, E7-19, E7-20,
  E7-22); ADV-TCH-008 (the visit; `verified`: E7-21, E7-22); GAM-RES-001 (losses), GAM-DEC-004
  (the record's group), GAM-DEC-002 (no level drawn twice) amended; A-19 as `ADV-QUIRK-28`
  (exploit-fix, no flag; its test: two boxes, two records).
- *Agent's calls (change freely):* the empty box's message is presentation, no event (T62a's
  refusals send none either); the record's creatures join in the adventure before the game applies
  the hero's changes (nothing of either reads the other; the events' order is not the original's
  picture order); a looted Pyramid's bonus is in the outcome even when its mark is set (the hero
  model gates it, as the Tomb's); `Defender::Monsters` named by its shape, not by the Pyramid.
- **Tests:** `h3-adventure`'s `split/tests.rs` (the six rows and the bounds), `state/tests/records.rs`
  (7: the box's question, record and removal, the group without room, the guarded box's battle,
  loss, win and draw, the Pyramid's question, battle, loss, win and loot, the spell lost, the
  record per object); `h3-game`'s `tests/records.rs` (5: the box's record through `MoveHero` and
  `Answer`, a loss floored at 0, the dialog at the record's slots, a won guarded box, the
  Pyramid's split under both profiles and its luck) and `level_ups.rs` (an open level-up drawn
  once); `h3-map`'s play and view properties take a dialog opened over several stacks and the
  losses.
- **Replays**: the twelve re-recorded with their options, their commands unchanged (neither map has
  a box or a Pyramid): every hash moved.
- **Gates**: the scoped verify with the original data passed every step on its second run (the
  first: coverage, a box's record for a hero off the map and a box without its record, both
  tested since); `bench-check` at its baselines (no workload's map holds the new kinds). Then
  `cargo check --workspace --all-targets` and `h3-map`'s and `h3-sim`'s tests. A smoke run, not
  committed: random play on the eight shipped maps that run whole (the seven `.h3m` and the
  Tutorial; three seeds each, 14 days, at most 3 000 steps), no failure.

### Increment 7, round 6: T62b2, events and the move (2026-10-27, `adventure/e08-t62b2`)

**The cut** (decision 152): whole, the user's choice (the agent recommended two parts, the static
layer, the views and the property first). **Baseline:** `master` 8ef8244 (#521 merged, CI green on
it); the sandbox's full verify passed (no CI fingerprint on `master` yet when it started); the
original data staged from the user's computer.

- **The census first** (`h3-map`'s `original_data`, new counts): 486 events on the 160 shipped
  maps, 460 removed after they fire, 60 firing for computers too, 87 guarded, each with one
  visitable tile; **every one's template blocks its tile** (the draft left those tiles blocked: on
  a real map no hero could have stepped onto an event), none on another object's visitable or
  blocked tile, no tile with two events, 9 on water, 59 within one step of a monster.
- **The static map** (`state.rs`): `place` keeps an event apart from both masks (the original's
  placement clears the blocked and the visitable bit of its tile, 0x505C78): its tiles go to
  `StaticMap::events` and a bit per tile (`events_at`), never to the tiles' `blocked`/`visitable`
  nor the visits; `MapObject::event`, and `covers` answers neither for an event, so the
  occupancy's removals and the battle background's scan (GAM-BAT-011) pass it as the original's
  `place_object_tiles` does (no tile record for class 26). The map's masks stay on the object (the
  static hash's encoding unchanged).
- **The seats** (`setup.rs`, `Seat`): each player's colour and human flag by player id, which the
  game gives from its `PlayerSetup`s (`SetupError::Seats` when not one per player); a fixed array in
  the state, so a fork copies it; adventure state hash v15.
- **The move** (`move_hero`): the path is checked and priced whole as before, so nothing in its
  acceptance or price tells of an event; the first step onto a stop that visits nothing (a
  flyer's free tile mid-flight too, not a tile it only passes over) that fires an event ends the
  move there with that step's points left; the outcome's new `dropped` counts the steps not taken.
  A hero's start is no step.
- **The fire** (`state/events.rs`): `event_at` (the last live event of the tile in map order),
  `firing_event` (the seat's colour bit in the event's colours, a non-human only with the computer
  flag; none on a guarded tile until T63: `ADV-STUB-10`), `fire_event` (`EventFired { hero, tile }`
  to the owner alone, then the guards' battle at once through `guards_battle`, else
  `give_event`), `give_event` (the record by `give_record`, then the removal with the flag, told to
  nobody), `end_event_battle` (reached from `end_unowned_battle` when no object at the tile takes
  the battle: lost, the survivors stay; won, the guards are gone for good and the record follows).
- **The views** (`observe.rs`): an event is never an object view, and its tile is no object's in
  the tile views (it never was: before T62b2 its template made the tile blocked and visitable).
- **The game:** `GameSetup::adventure_setup` passes the seats; a won event's record before the winner's
  levels as a box's (`ADV-STUB-10`).
- **Rules:** ADV-OBJ-007 and ADV-MOV-011 (new, `verified`: E7-17, E7-18, E7-23), ADV-FOG-007 (new);
  A-26 as `ADV-QUIRK-29` (exploit-fix, no flag); `ADV-STUB-10` amended (the event on a guarded tile
  neither fires nor stops a flyer until T63).
- *Agent's calls (change freely):* the seats as a static setup input (the adventure has no other
  source of a player's colour or human flag); `EventFired` carries the tile for a client's message;
  a draw against an event's guards leaves none, so the next entry gives the record (the original's
  "lost: return"; VCMI removes a non-repeatable event after a draw); on a tile with several events
  the last live one in map order is the tile's (the original keeps one word, the last placed; none
  on the shipped maps); an event on another object's blocked tile does not unblock it as the
  original's placement would (none on the shipped maps); an event on water fires only once boats
  exist (9 on the shipped maps; the queue's inbox).
- **Tests:** `h3-adventure`'s `state/tests/events.rs` (7: the fire and the stop, the colours and
  the computer flag, the repeats and the start, the guards' battle lost and won, a guarded tile,
  a flyer stopped or passing, nothing seen, the paths and reach alike), `state/tests.rs` (the
  seats' error and hash); `h3-game`'s `tests/events.rs` (the record through `MoveHero`, the firing
  told to red alone, a guarded event's battle and win) with **the hidden-event property** (48 cases:
  an event anywhere a hero may stand, any colours and flags, guarded or not, blue human or not;
  every player's observation, legal commands, legal moves and told events the same as without it
  until it fires; checked to fail at once with events shown in the views); `h3-map`'s view
  property takes `EventFired`.
- **Replays**: the twelve re-recorded with their options, their commands and days unchanged (neither
  map has an event): the version and every hash moved.
- **Gates**: the scoped verify with the original data passed every step on its first run;
  `bench-check` at its baselines (no workload's map holds an event: a stop's step tests one bit);
  then `cargo check --workspace --all-targets` and `h3-sim`'s tests. A smoke run, not committed:
  random play on the eight shipped maps that run whole (three seeds each, 14 days, at most 3 000
  steps), no failure.

### Increment 7, round 7: T63a, the typed answers and increment 7's decisions (2026-10-27, `adventure/e08-t63a`, `cross-lane`)

**The cut** (decision 153): T63 in two, the design's (the agent recommended three, the typed
answers alone first); the user's consent to the `cross-lane` change of `h3-sim`'s action mapping
with a University's purchase entries now. **Baseline:** `master` dc4d7e6 (#526 merged); the
sandbox's light gates passed on its code (CI's fingerprint); the original data staged from the
user's computer.

- **The answers** (`command.rs`): `Answer` typed by the decision's kind (ADR-0017 point 6,
  decision 138): `Offer(Consent)`, `LevelUp(SkillOffer)`, `Chest(ChestChoice)`,
  `Arena(ArenaChoice)`, `School(SchoolChoice)`, `University(UniversityAnswer)`, `Close`; an answer
  of another kind is `WrongAnswer`; `SlotClosed` and `CannotPay` for a purchase; the events
  `ChoiceOpened` and `UniversityOpened` (the owner's alone); `DecisionClosed` also closes a choice
  and a University; `OfferRefused` also tells a School the treasury cannot pay.
- **The decisions** (`state.rs`): `Decision::Choice` (the adventure's `Choice`, with
  `guard_after`) opened by `open_choice` when the treasury can pay its price, answered by
  `answer_choice` (the price, then `AdventureState::choose`, then a waiting guard's visit);
  `Decision::University` opened by `open_university` whatever the treasury holds, `buy` (the slot
  open for the hero by `university_slot`, the price, `RaiseSkill` at Basic; it stays open) and its
  leaving (then a waiting guard's visit); `pick_of` types a choice's answer (a chest's; the Arena's
  for a choice that cannot be left; a School's for one that can); both hashed (tags 5 and 6; game
  state hash v18); `GameState::choice`, `university`, `slot_state`, `can_pay`; the defaults'
  `choose_by_default` gone.
- **Legal commands and defaults** (`legal.rs`): `answers` lists each kind's in a fixed order (an
  offer's accept then decline, a level-up's first then second, `Close`, a chest's gold then
  experience, the Arena's two, a School's two then leave, a University's purchases of the open
  slots it can pay in slot order then leave); `default_answer` per kind (`Decline`, the original's
  choice, `Close`, `Gold`, `Attack`, `Leave`, `Leave`).
- **The views** (`observe.rs`): `DecisionView::Choice(Choice)` and `DecisionView::University {
  visit, slots }` (each skill's `SlotState` for the hero).
- **A bug found and fixed** (`h3-adventure`, `state.rs`'s move): a monster guarding an entered
  object's tile answered instead of the object's outcome but its resources, so since T62a a
  Learning Stone, a Shrine, a Witch Hut, a gift or a find entered on a guarded tile marked or
  emptied it and gave the hero nothing. Now the guard waits for an offer, a choice or a University
  (`AdventureOutcome::waits`) and otherwise follows the object's resources, losses and hero gains
  (ADV-MON-006 amended).
- **The env** (`h3-sim`, the AI lane's, `cross-lane`): answers by role (`answer_entry`; 9 accept,
  10 decline, close or leave, 11 and 12 the first and second of a level-up, a chest, the Arena, a
  School), a University's purchases of slots 0 to 3 at 13 to 16 (`BUY_BASE`,
  `UNIVERSITY_SLOTS`); `decode` takes the listed answer of the entry's role, else the offer's or
  level-up's or the purchase (rejected by the game); action encoding v5. The AI lane's inbox told.
- **Replays:** format v7 (`answer <kind> <answer>`, `answer close`); the twelve re-recorded with
  their options (the scripted two from their commands in v7's words), their commands unchanged
  but for their words: every hash moved.
- **Rules:** GAM-DEC-005 and GAM-DEC-006 new; GAM-DEC-001 to -004, ADV-PIK-005, ADV-TCH-001,
  -002, -007 and ADV-MON-006 amended; `ADV-STUB-10` narrowed to the rest (T63b).
- *Agent's calls (change freely):* decision 153's.
- **Tests:** `h3-game`'s `tests/rewards.rs` (the chest's, the Arena's, a School's and a
  University's decisions with every other kind's answer rejected, their views, legal answers,
  defaults, events and hashes; a guard waiting for each and following a Learning Stone's
  experience, which failed before the fix); the test rules' entered gifts and University; `h3-map`'s
  play property lists and applies the new answers (`choice_answers`), its free answers of every
  kind, its view property the new events; `h3-cli`'s words of every answer and the bad ones;
  `h3-sim`'s entries of a University's purchases and leaving and of the Arena through the env.
- **Gates:** the scoped verify with the original data: its first run failed coverage (the new
  decisions' hash arms, a School's first answer, two errors' text, an unreachable empty answer
  list), all fixed; the second passed every step but `check-lane` (the `cross-lane` change, by
  design); `bench-check` at its baselines (`random-play-duel-36` −0.47 %, `fork-xl-144` +0.76 %,
  `manage-army-12` +0.35 %); `cargo clippy --workspace --all-targets` clean; `h3-sim`'s,
  `h3-cli`'s and `h3-map`'s tests. A smoke run, not committed: random play on the eight shipped
  maps that run whole (three seeds each, 14 days, at most 3 000 steps), no failure.

### Increment 7, round 7: T63b, the rest (2026-10-28, `adventure/e08-t63b`)

**The cut** (decision 154): whole (the agent recommended three parts), with a bug found while
sizing it fixed in it. **Baseline:** `master` fdf002f (#530 merged, CI green on its merge); the
sandbox's light gates passed on its code (CI's fingerprint); the original data staged from the
user's computer.

- **The rests** (`h3-game`, `state/rests.rs`; GAM-DEC-007 new): what a visit still owes its hero,
  held in a list beside the decisions (`GameState::rests`, `Held { owner, hero, rest }`) and run by
  `run_rests` after every command (`apply`), the last held first, each once its owner has no open
  decision; after every command each held rest's owner has one. Four kinds: `Guard` (the visit of
  the monster guarding the hero's tile after the visit of the tile's object), `Experience(u32)` (a
  reward record's experience after its army dialog, with its levels), `Won(Pos)` (a won guards'
  battle's reward after the winner's levels), `Event(Pos)` (an event on a guarded tile after the
  guard's visit). Hashed after the decisions (game state hash v19: tags 0 to 3); the decisions'
  `guard_after` flags gone (`Decision::{Offer, Choice, University}`, their hash's booleans).
- **A bug found and fixed** (ADV-MON-006 amended): the guard's visit that waited for an object's
  decision started as soon as the decision closed, beside what its answer opened; and since T63a
  an entered gift's experience (a Learning Stone; a Tree of Knowledge's level) opened its level-up
  with the guard's battle in one outcome: two decisions open (ADR-0017 point 6's one), the
  battle's hero read before the level. Now the adventure's move sets `guard_after` (the hero) for
  every visited object on a guarded tile and the game holds the guard's visit as a rest, so it
  waits for the whole visit (its questions, choices, University, levels and battles). The test
  `a_guard_waits_for_a_choice_or_a_university_and_follows_a_gift` failed before the fix (the
  battle opened beside the level-up, its hero's primary skills before the level). A box and a
  chest are visited from the tile beside them, so no guard waits on them (the question to the user
  named them wrongly: corrected under decision 154).
- **The record's experience** (`settle_outcome`): with a group without room the record's
  experience is held and given with its levels after the army dialog closes (ADV-RWD-001, D-39).
- **The won guards' battle** (`h3-adventure`, `piles.rs`, `events.rs`): `end_unowned_battle` marks
  a won object's guards beaten at once (`army_left` empty) and names the reward
  (`AdventureOutcome::reward_after`); `AdventureState::give_won` gives it later, read again when
  given: a guarded pile's, artifact's or scroll's pickup, a box's record, an event's record, a
  Pyramid's loot and spell (its Wisdom test after the levels, E7-22); nothing when the object left
  the map meanwhile. `end_event_battle` no longer gives the record.
- **The event on a guarded tile** (`state.rs`'s move, `events.rs`; `ADV-QUIRK-29`):
  `firing_event` no longer skips a guarded tile; a step that fires an event on a guarded tile ends
  the move there (a flyer's too, at its move's end for the guard test), the guard visits, and the
  event is named (`AdventureOutcome::event_after`); `AdventureState::fire_held_event` fires it once
  the guard's visit is over, for a hero still on its tile that it still fires for (A-26 fixed).
- **The battle bonuses** (`state/battle.rs`'s `end_battle`; GAM-BAT-014 new): every hero of a
  battle loses its bonuses and once-marks (`HeroChange::ClearBonuses`) after the heroes' part and
  before the winner's levels, told to its owner when it had any.
- **Replays:** the twelve re-recorded with their options (the scripted two from their commands),
  their commands unchanged: every hash moved (game state hash v19).
- **Rules:** GAM-DEC-007 (verified: E7-22) and GAM-BAT-014 (verified: E7-11, E7-21) new;
  ADV-MON-006, ADV-OBJ-007, ADV-MOV-011, ADV-PIK-004, -007, -008, -013, ADV-TCH-008, ADV-RWD-001,
  GAM-DEC-004, -006, GAM-BAT-008 and HER-ADV-008 amended; `ADV-QUIRK-29`'s ledger entry (its
  regression tests); `ADV-STUB-10` closed.
- *Agent's calls (change freely):* decision 154's.
- **Tests:** `h3-game`'s `tests/rests.rs` (a record's experience after its dialog; a won guarded
  artifact after the winner's level-up; the hash sees each kind of rest), `tests/rewards.rs` (the
  gift's level-up before the guard's battle, the battle reading the levelled hero),
  `tests/events.rs` (an event on a guarded tile fires once a joining guard's visit is over),
  `tests/aftermath.rs` (the bonuses cleared for both heroes whatever the result, before the
  winner's levels, untold without bonuses); `h3-adventure`'s piles, records, rewards and events
  tests through `give_won` and `fire_held_event` (a flyer stopped on a guarded event); `h3-map`'s
  play property: a won pile is picked up in the battle's last command unless a level-up is left
  open (its seed committed).
- **Gates:** the scoped verify with the original data: its first run failed coverage (a held event's
  hash arm), fixed in the game's event test; the second passed every step; `bench-check` at its
  baselines (`fork-xl-144` +1.35 % and `fork-meeting-xl-144` +1.29 %, T63a's +0.76 % on the
  first: likely the empty list of rests each fork copies; `manage-army-12` +0.34 %,
  `random-play-duel-36` −0.46 %, the reach workloads −0.24 % and −0.99 %); `cargo clippy
  --workspace --all-targets` clean; `h3-sim`'s tests, `h3-cli`'s adventure replays and `h3-map`'s
  tests. A smoke run, not committed: random play on the eight shipped maps that run whole (the Tutorial and seven `.h3m`; three seeds each, 14 days,
  at most 3 000 steps), no failure.

### Increment 7, round 8: T64a, the rewards map, its replays and benchmarks (2026-10-28, `adventure/e08-t64a`)

**The cut** (decision 155): T64 in two, the user's "Two parts" (the agent recommended three): T64a
the test map, its replays and benchmarks; T64b the properties and the 144 × 144 maps.
**Baseline:** `master` 87613c9 (#533 merged, CI green on it); the sandbox's light gates passed on
its code (CI's fingerprint); the original data staged from the user's computer.

- **The rewards map** (`fixtures/maps/e08-rewards-36.h3m`; `h3-map`'s `tests/rewards_map.rs`,
  `the_rewards_map_is_committed_as_written`): SoD, 36 × 36 of grass, one level, written by
  `h3_map::writer::write_with` (the writer's heroes, monsters, plain and reward objects, and
  `Extras`' objects with the editor's body for the classes the writer has no type of: the Arena,
  Marletto Tower, the Garden of Revelation, the Lean-To, the Library, the two Schools, the
  Mercenary Camp, the Star Axis, a Scholar, the Tree of Knowledge), each object from the
  original's template line (`Objects.txt`, in the test: its shape and entrance) placed by its
  entrance. 42 objects: every one of `codes.rs`' 31 classes of increment 7 (the test imports the
  map and finds each type), gnolls with an artifact, the paths' guards. Red: Orrin (hero 0) at
  (6, 4) with a full army of seven kinds; Adela (hero 1) at (28, 4), basic Wisdom alone, a Spell
  Book, no spell, twenty archangels, 950 experience; Mephala (hero 2) at (5, 28). Blue: two heroes
  of their types' own at (32, 30) and (32, 18). Beside red's heroes: a box giving ten wood elves
  and 1 000 experience on (9, 4); an event for red on (6, 8) guarded by two savage gnolls on
  (7, 9); a Learning Stone on (3, 8) guarded by two on (2, 9), entered from (4, 8); a box on
  (31, 4) guarded by ten pikemen giving Bless and Prayer; an event for red on (8, 28); the
  University's entrance on (13, 26). The rest in rows (`fixtures/maps/README.md`).
- **The golden replays** (`adventure_replays.rs` lists what each meets; the test expects 21):
  random play `rewards-s162-d28`, `-s243-d28` and `-s88-d28` (`--seed N --agent-seed N --days
  28`), the best cover of three in a scratch search over the seeds 1 to 400 (a game's moves end
  on 3 to 9 of the 31 classes; every class in some game, the University in 18, its decision in
  15); scripted (`--seed 1 --checkpoint-every 5 --script`): `rewards-box-dialog-s1`,
  `-event-stop-s1`, `-university-s1`, `-guarded-box-s1`, `-event-guard-s1` and
  `-stone-guard-s1`, each design §16's order told by its events: the dialog closes before the
  box's experience; the step stops on the event; two purchases, the next day's visit offering
  only the other two; the level-up's advanced Wisdom before the box's Bless and Prayer, Prayer
  taught (with the other offer, Scouting, Bless alone: the probe's check); the guard's battle
  before the event; the stone's level-up before the guard's battle. The battles' commands came
  from a scratch driver (an attack or a shot when listed, else a defence).
- **The benchmarks:** `random-play-rewards-36` (5 games of 28 days, 1 358 steps, inside
  `random_rollout`): 1 171 206 348 Ir, about 862 000 a step (the duel's about 305 000);
  `fork-rewards-36` (100 000 forks after 7 days, 49 steps, inside `fork_workload`): 257 600 397
  Ir, 2 576 a fork (`fork-xl-144`: 3 564), the forks carrying the visits block;
  `BenchResult::visits` and the bench's line "forks with the visits block" (the CLI's bench test:
  none of the duel's forks, the rewards map's game's). The benches README's two rows:
  `shared/e08-t64a-benches` (#537), with `docs/06-security.md`'s increment 7 block (ADV-VIS-006,
  ADV-FOG-007), the line T62a left for the next `shared/` change.
- *Agent's calls (change freely):* decision 155's.
- **Versions:** none.
- **Gates:** the scoped verify with the original data passed every step (coverage, determinism
  with the 21 replays, `bench-check` with the eight workloads, the original data); `clippy -p
  h3-cli --all-targets` clean (the scope builds no `h3-cli`).
- **The epic file** stays above its aim (48 KB of 35): all of it is the open increment 7, which
  T65 moves to the archive.

### Increment 7, round 8: T64b, the properties and the 144 × 144 maps (2026-10-28, `adventure/e08-t64b`)

**The cut** (decision 156): whole, the user's "Whole in one" (the agent recommended two parts, the
properties first). **Baseline:** `master` c4ae2fa (#537, #538 merged, CI green on T64a's merge);
the sandbox's light gates passed on its code (CI's fingerprint); the original data staged from the
user's computer.

- **The plans** (`h3-map`'s `tests/play.rs`): 35 kinds appended to `KINDS` (`REWARDS`,
  `reward_body`): the Arena, Centaur's Axe, a box (500 experience, 500 gold, Magic Arrow, four
  pikemen), a Corpse, Marletto Tower, an event (300 experience and gold for every colour, removed),
  the Garden, a Lean To, the Library, the two Schools, the Mercenary Camp, the Star Axis, the
  Pyramid, the five random artifacts, a Scholar, the three Shrines (drawn), a scroll, a Learning
  Stone, a chest, a Tree of Knowledge, a University, a Wagon, a Warrior's Tomb, a Witch Hut of
  every skill; a box two gnolls guard (1 500 experience, Bless, Prayer), an event two gnolls
  guard that stays (Attack, luck), Blackshard whose gnolls ask, three savage gnolls with Centaur's
  Axe. `rewards()` is drawn last (the committed maps and the regression files' maps kept): up to
  six within two tiles of a hero in seven plans of eight. A placed artifact's type is one type
  whatever its artifact (`lay_out`'s twin rule; two on one anchor were `SharedAnchor`).
- **The checks** (every step of `every_step_keeps_the_commands_invariants`): the offers of
  increment 7 (`check_decisions`: a guarded artifact's or scroll's question, a gift's price and a
  School's refusal by `gift_price`, the Tomb's, a box's and the Pyramid's questions; a choice and
  a University open and close as told), management closed while a choice or a University is
  open; the gains and losses (`check_conservation`: `visit_gains` for a chest's gold and a find
  emptied, by its artifact received or its turn; `records` for a box's and a fired event's record
  in the events' order; `ResourceLost`; the School's and the University's payments); a hero's
  changes and levels applied in the order told (several levels in one command); a won guards'
  battle's gift (`check_unowned_battle_end`: a guarded artifact, box, the Pyramid, a guarded
  event; held only by a level-up the battle's own experience opens before anything of the gift);
  an event's stop, fired or held for its tile's guard (`landing`, `check_legal`: `Reach` crosses
  events, design §12); an artifact appears only as one an object that left held (`left_with`).
  New: `check_rewards` (removals, marks of heroes on the map, knowledge and the emptied objects
  only grow; an emptied find or a looted Pyramid gives no resource, artifact or spell at a later
  visit, `visited`).
- **The view property** (`view.rs`): a visit that shows what the build drew (gains, a choice or
  University opened, a price refused, the viewer's knowledge grown) ends the comparison as a
  pickup does, its contents' events left out (their number too).
- **The cover test** (`the_plans_visit_every_kind_and_answer_every_decision`, 512 fixed cases,
  its own step mix): every one of the 31 classes visited 9 to 79 times; both answers of a Tree's
  price (1 decline), the Tomb's, a box's, the Pyramid's and a guarded artifact's questions; a
  chest's gold and experience, the Arena's two skills, a School's two and leaving (1), a
  University's four purchases and leaving, a box's army dialog closed.
  `the_plans_meet_and_manage` goes from 96 to 320 cases: the plans' new draw moved its army
  dialogs' cover (taken stacks 0 at 96 and 192 cases, with or without the new objects).
- **The hidden-event property** (`h3-game`'s `tests/events.rs`): in half the cases a wandering
  monster of any disposition in both games, which guards the event's tile when beside it (the
  guard's visit first, the event held: GAM-DEC-007); 48 cases, 1 000 run once.
- **Decision 133's maps** (`tests/full_maps.rs`): `e08-boxes-144.h3m` (98 KB) and
  `e08-stones-144.h3m` (72 KB), SoD, 144 × 144 with an underground, red's heroes on (8, 8) of
  each level and blue's on (135, 135), a box (100 experience, 100 gold) or a Learning Stone on
  each of the other 41 468 tiles, written by `h3_map::writer`; `the_full_maps_are_committed_as_written`
  imports each (41 468 objects, 4 heroes); `a_map_full_of_boxes_or_stones_builds_plays_and_forks`
  plays 300 steps (54 boxes opened; 54 stones marked, the heroes walking from stone to stone, the
  stones entered), forks, and checks the fork against its parent.
- **The benchmarks:** `fork-boxes-xl-144` and `fork-stones-xl-144` (100 000 forks after 7 days of
  random play, 52 and 46 steps): 236 700 024 Ir each, 2 367 a fork (`fork-xl-144` with a battle
  open: 3 564; `fork-rewards-36`: 2 576); only the stones' forks carry the visits block (boxes
  leave the map), and it adds nothing to a fork (design §10's estimate: about 20 Ir). The CLI's
  bench test checks the block on the stones' forks, none on the boxes'. The benches README's rows:
  `shared/e08-t64b-benches`.
- *Agent's calls (change freely):* decision 156's.
- **Versions:** none.
- **Long runs** (release, not committed, fresh seeds each): the step, fork and both view properties
  over 1 500 to 4 000 cases, until clean; the hidden-event property over 3 000. Each failure was
  the tests' oracle, never the engine: two placed artifacts on one anchor (`SharedAnchor`), an
  event held mid-path on a guarded tile, a find's artifact against its resources, levels given at
  once, a meeting read as a visit, a chest's choice in the view property, a monster's treasure and
  a held event's record out of order, several levels in one command. No engine bug found.
- **Gates:** the scoped verify with the original data passed every step (coverage, determinism,
  `bench-check` with the ten workloads, the original data); `clippy -p h3-cli --all-targets`
  clean and the CLI's tests passed (the scope builds no `h3-cli`). The lane's lasting trap of the
  plans' draws: `TRAPS.md`.

### Increment 7, round 8: T65, the audit; increment 7 closed; increment 8 refined (2026-10-28, `adventure/e08-t65`)

Baseline: `master` at e8a7990 (#544 and #543 merged). No original files staged: the task changes
docs, the catalog's sources and statuses and eleven code comments only. At its start the user chose
T65 whole (decision 157, `AskUserQuestion`; not chosen: T65a the audit and the closure, T65b the
refinement) and answered increment 8's two questions with it (ADV-Q31).

**The audit of increment 7's exit criteria** (design §19, moved below), on `master`'s code:

1. The pickups: the treasure chest (ADV-PIK-005: `visit_chest`, its gold or experience a choice,
   GAM-DEC-005, the full backpack's 1 000 gold), placed and random artifacts, guarded too
   (ADV-PIK-006, -007, ADV-OBJ-006), Spell Scrolls (ADV-PIK-008) and the monsters' treasure
   artifacts (ADV-MON-011), a full backpack refusing under `ADV-QUIRK-22` in both profiles; the 8
   maps the import refused for a monster's artifact import further (T61b's census: 3 whole, 3 to a
   random hero, 2 to a hero in a town). Observed: E7-1 to E7-6.
2. The twenty classes that teach or hold a find (ADV-TCH-001 to -007, ADV-PIK-009 to -012:
   `visit_gift`, `visit_shrine`, `visit_witch_hut`, `visit_university`, `visit_find`, the
   Schools' and the Arena's choices, the University's purchases, GAM-DEC-006); the once-per-hero
   marks per object and hero and the emptied finds in `Visits`, without a bound (`ADV-QUIRK-27`;
   `h3-map`'s `check_rewards` at every step of the plans). Observed: E7-4, E7-7 to E7-10.
3. Pandora's Box and events (ADV-PIK-013, ADV-OBJ-007, ADV-RWD-001, HER-CHG-003): the record's
   order as batch E7 decided D-37 to D-39, the guards at once as a guarded pile's, an event firing
   for its colours and stopping the move (ADV-MOV-011), the leftover creatures through the army
   dialog and the experience after it (GAM-DEC-004, GAM-DEC-007; the replay
   `rewards-box-dialog-s1`). Observed: E7-12 to E7-15, E7-17 to E7-20.
4. The Pyramid (ADV-TCH-008, ADV-MON-012): its drawn spell, the question at every visit, the second
   creature split by 0x63DF94's order, the spell taught or lost, a looted visitor's −2 luck once.
   Observed: E7-16, E7-21, E7-22.
5. The bonuses (HER-ADV-008, GAM-BAT-014) reach the next battle and end with it; a won guards'
   battle's levels come before the visit's rest (GAM-DEC-007; `rewards-guarded-box-s1`,
   `rewards-stone-guard-s1`). Observed: E7-11, E7-21, E7-22.
6. Contents and draws hidden until a visit, the owner's knowledge kept (ADV-VIS-006: `known`,
   `OwnHeroView::marks`; `h3-map`'s view property), events never observed and blind to `Reach`
   (ADV-FOG-007, ADV-MOV-011; the hidden-event property with a guard, T64b; A-23 withdrawn);
   decision 133's maps (`tests/full_maps.rs`: 41 468 boxes or Learning Stones build, play 300 steps
   and fork; `fork-boxes-xl-144`, `fork-stones-xl-144`).
7. `check-rules` passes; the ledger holds `ADV-QUIRK-21` (A-21, flag 37) and `-22` (A-25, flag 38)
   with their tests in both profiles, A-17 to A-20, A-22, A-24 as `ADV-QUIRK-27`, `-23`, `-28`,
   `-24`, `-25`, `-26` (exploit-fix, no flag) and A-26 as `-29`, each with its regression test.
   The gates: CI's full verify on #544 and on `master` after it (coverage, determinism with the
   twelve re-recorded and nine new replays, `bench-check` with the four new workloads baselined:
   `random-play-rewards-36`, `fork-rewards-36`, `fork-boxes-xl-144`, `fork-stones-xl-144`). The
   env plays on through every new decision: `h3-sim`'s
   `increment_7s_decisions_are_answered_through_the_env`, and the three random-play replays on
   the rewards map, recorded by `random_rollout` (chests, the Tomb, the box, the Pyramid, the
   Schools, the University, an army dialog).
8. Batch E7 run (T57) and scored (T58, survey §15): every dispute's row the code's side, none
   reverted, no `ADV-Q`.

**The gaps fixed:** criterion 8 wants the rules behind the disputes and the consensus rows
`verified`, but nine stayed `implemented` though they cite their rows (T61b wrote them before the
visits, T62a built the visits and left the statuses): ADV-OBJ-006 (E7-3, D-31), ADV-PIK-005 (E7-4,
E7-5), ADV-PIK-009 and -010 (E7-10), ADV-TCH-004 (E7-8, D-34), ADV-TCH-005 (E7-7, D-33),
ADV-TCH-007 (E7-9, D-36), HER-CHG-003 (E7-5, E7-6, E7-10, E7-12 to E7-15, E7-19; D-37 to D-39)
and HER-ADV-008 (E7-11, E7-21): now `verified`. ADV-PIK-011 (the Warrior's Tomb) cited no row,
though E7-10 and E7-11 played both Tombs: the observation added, `verified`. Three observations
named `E08-batch-7b.md` for rows of `-7a.md` (ADV-TCH-007's E7-9, ADV-PIK-009's and -010's E7-10):
corrected. Rows added where a rule stays `implemented`: ADV-TCH-003 (E7-4's Learning Stone; the
Tree of Knowledge's price had no row) and GAM-DEC-005 (E7-4's and E7-5's chest choices; the
Arena's and the Schools' had none); GAM-DEC-004 gained E7-20 (the dialog over a box's stacks,
its T62b1 amendment). Still `implemented` with no row of any batch: ADV-TCH-001, -002, -006,
ADV-PIK-012, GAM-DEC-006 (no purchase was played), ADV-VIS-006 and ADV-FOG-007 (projections, a
design), GAM-SET-003, HER-CHG-004, and the amended ADV-OBJ-002, ADV-PIK-004, ADV-MON-001, -006.
Nothing else was missing.

**The design note** moved below ("Increment 7's design note at its closure"); `E08-design.md` keeps
a pointer until T66b writes increment 8's note. The eleven code comments that cited it by section
(`h3-adventure`'s `persistent.rs`, `ruleset/rewards.rs`, `setup.rs` twice, `state/draws.rs`,
`events.rs`, `holds.rs`, `records.rs`, `rewards.rs`, `visits.rs`; `h3-game`'s `state/rests.rs`),
the base pack's comment on increment 7's kinds, survey `E08-rewards.md`'s note on T55b's part and
the catalog's design sources (ADV-VIS-006, GAM-DEC-005, -006, -007, GAM-BAT-014) now name the note
in this log; so do four that T34b and T53 left at the file (GAM-DEC-003, -004 and GAM-MAN-002:
increment 6's §8, §9, §16; HER-EXP-002: increment 4's §7). `h3-heroes`' `reward.rs` and
`teach.rs` already pointed here since T60b. The shared paths (ADR-0017, `docs/06-security.md`),
the other lanes' notes and the batch specs keep theirs: historical.

**Increment 7 closed** (2026-10-28); its scope, decisions 121 to 157 and task list moved to
`E08-archive.md` (the cuts T59 moved early now a part of its section), the epic file from 50 KB
to 18 KB.

**Increment 8 refined** (ADV-Q31; decisions 158, 159): the user chose **boats and the sea with the
sea's pickups** (boats on the map and in the writer, CLT-Q5; boarding, sailing and landing with
Navigation, the sea's artifacts and Favorable Winds; battles with a hero in a boat; the events on
water; Sea Chest, Shipwreck Survivor, Flotsam and Ocean Bottle), not boats and sailing alone,
every sea object (Whirlpools, Buoys, Mermaids and Sirens too) or towns (E09) instead; Shipyards
and Lighthouses, owned objects, stay with the sketch's item 9. The first round a survey and the ADR
amendment drafts with the design note and the cut (T66a, T66b), not the survey alone. The inbox:
the map client lane's note of the writer's edge (2026-10-27, nothing to do) to the archive; the
events on water re-tagged for T66a; CLT-Q5 told of decision 158.

**Verify:** the scoped `cargo xtask verify --scope adventure --summary` passed every step it runs
(fmt, clippy, check-arch, check-rules, check-stubs, check-docs, check-exe-map, clients-lock,
check-lane, test, coverage, determinism, bench-check, doc-drift; deny and ffi-smoke skipped by the
scope; exit 2, its normal result); no original data staged (no step needs them for comments and
docs). CI's full verify on the pull request is the record. The sandbox restarted once during the
session (its background baseline died with it; CI's run on `master` e8a7990 was green).

#### Increment 7's design note at its closure (moved by T65)

The note as T64b left it, every section built (T56 to T64b; T65 marked rounds 7 and 8 done and
wrote §17's last versions); its § numbers are the ones the code's comments cite as "increment 7's
design note".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 7's
amendments of ADR-0013 (points 1, 2, 15), ADR-0016 (points 1, 3, 4, 6) and ADR-0017 (points 1,
4, 6, 8–11), applied by T59 from the drafts T55b wrote
from T55a's survey and the answers at ADV-Q28 (the epic's decisions 129–136), accepted with the
cut at ADV-Q29 (decisions 138–141; decision 138's typed answers written into §14). The task that
builds a section deletes it; the file goes when the epic closes. Facts and sources: survey `docs/rules/surveys/E08-rewards.md` (§ numbers are the
survey's unless they name this note); the proposed rule ids are the survey's §12, with the
additions of §6 and §17 here.

Increment 6's note is in `E08-log.md` ("Increment 6's design note at its closure", moved by T53).

Written whole (decision 137): **part A**, the hero side (`h3-heroes`, the core content); **part
B**, the map (`h3-map`, `h3-adventure`); **part C**, the game, the env, batch E7 and the cut.

##### Part A — the hero side (`h3-heroes`, the core content)

###### 1, 2. The data and the hero changes

Built by T60a (`E08-log.md`, "T60a"): `HeroDetails::bonuses` (`BattleBonuses`), the changes
`GainPrimary`, `RaiseSkill`, `Bonus`, `ClearBonuses`, the details' hash in the game state hash
v15. Already there for §3 and §4: the artifact classes (`ArtifactClass`, Special never placed at
random), the spells' level, schools and `granted_by_name_only` (Titan's Lightning Bolt, flag
0x2000), the classes' level-up weights (the University's slots), the Learning factor
(HER-EXP-001), the learnable level (HER-SPL-005), the backpack's most (64, HER-ART-002), the most
skills (8), `HeroParams::mana_ceiling` (999, T59). Not the details': the once-per-hero marks of
objects (the adventure's, by hero id, §10) and the players' knowledge (§10).

###### 3, 4. The reward record's hero half and what a teacher may teach

Built by T60b (`E08-log.md`, "T60b"): `grant` with `HeroReward` and `Granted` (HER-CHG-003; the
experience in `Granted::experience` for the game to give last, §14), `can_receive_artifact`
under flag 38 (A-25), and the teachers (HER-CHG-004): `can_learn_spell` (the known-counts form;
the Pyramid passes `false`), `hut_teaches`, `university_slot` (`SlotState`), `scholar_reward`
(`ScholarGift`: kind and id in one; the fallback for a spell or skill), `library_admits`,
`learning_experience`, `next_level_experience`. For §11's visits.

###### 5. The bonuses in battle

Built by T60a (HER-ADV-008: `derive_battle_hero` folds the bonuses into the own side's morale and
luck sums). Left for T63: the game applies `ClearBonuses` to every hero of a battle when it ends
(§15, GAM-BAT-014); its test: a hero with a −3 bonus and Leadership Basic gets −2 morale on its
stacks; after any battle, 0.

###### 6. Rule ids, quirks, shared needs (the hero side)

Built: HER-ADV-008 and HER-CHG-001's summary (T60a), HER-CHG-003 and HER-CHG-004 (the teachers,
an id beyond survey §12; T60b). No quirk of the hero side but A-25's (`FULL_BACKPACK_REFUSES`,
flag 38: its regression test in both profiles by T60b; the wraps are A-9's class).

##### Part B — the map (`h3-map`, `h3-adventure`)

###### 7, 8. The import and the kinds

Built by T61a (`E08-log.md`, "T61a"): the 31 classes typed (`codes.rs`), the kinds with their
type parameters in `adventure.toml` (`ObjectKind::Chest`, `Artifact`, `Scroll`, `Gift`,
`Shrine`, `WitchHut`, `Scholar`, `University`, `Find`, `PandoraBox`, `Event`, `Pyramid`;
ruleset part v12), what the map puts in them (`ObjectSetup::holds`, `Holds`; the reward record
`RewardRecord` with `h3_heroes::HeroReward` as its hero half, `EventSetup`), the guards of
artifacts, scrolls, boxes and events, the monster's treasure artifact (`MonsterSetup::artifact`),
A-24 refused (`Unsupported::RewardArmy`); map import v12, adventure state hash v13 (the static
map's holdings). A scroll holds the Spell Scroll with its spell as an `ArtifactInstance`, so its
kind has no parameter; an empty Witch Hut mask imports as every skill. **The bans**
(T61b, map import v13, GAM-SET-003): `GameSetup::banned_spells` and `banned_artifacts` beside
`banned_skills`, each checked against the ruleset; the game passes the three to the adventure's
build as `h3_adventure::Bans`; the import reads the artifact set within the edition's codes (the
editor's default bans of the unused records 141 to 143 and an AB set's padding ban nothing; the AI
lane's two literals `cross-lane`).
Left for T62: an event's tile apart from the movement's masks (§12's "Static"; until then a
typed event blocks a walk as the blocker it was); every visit (`ObjectKind::effect` gives none
for the new kinds). The census (T61b's run, `h3m.md` "Import"): 8 maps import whole, 3 more
than before, the 8 refused for a monster's artifact before all moving on.

###### 9. The draws when the game is built (ADR-0016 point 4; GAM-SET-001's stream)

Built by T61b (`state/draws.rs`; `MapObject::resolved`, `Resolved`, a chest's gold and a find's
resources in `MapObject::contents`; adventure state hash v14; ADV-OBJ-006, ADV-PIK-005, ADV-PIK-009
to -012, ADV-TCH-003 to -008). Two points the build settled: a draw whose classes are out restarts
the marks from the bans *before* it widens to every class, as 0x4C94D0 does, so the widened draws
may repeat; a Scholar's empty pool (every skill or spell banned) gives a primary skill, and the
Wagon's resources are drawn beside its artifact (the turn `drawn`). A-18, A-20, A-22 and A-24 are
`ADV-QUIRK-23` to `-26` (exploit-fix, no flag).

From the setup stream, after today's draws (piles, campfires, monsters), in two passes in object
order (the original resolves the random artifacts first, then walks the rest, §3.1):

1. **The random artifacts** (65–69): uniform among the ruleset's artifacts of the type's classes,
   never Special, banned, placed (a class-5 object) or drawn before; none left, the marks restart
   from the bans; still none, any class (D-31, D-32).
2. **Every other draw**, each object's in its own shape: a chest (r in 0–99 by the type's
   chances; an artifact from the same pool and marks, class treasure); the Tree's price (one of
   three, uniform); a Shrine without a spell (uniform among its level's hero spells with a school,
   not banned and not marked; the marks start as the bans and clear per level when the level is
   used up: D-34); a Witch Hut (the r-th of its mask less the bans, uniform; none: refuses every
   visit); a Scholar (kind 0–2 uniform when random; its id among the unbanned; a fallback primary
   0–3 for kinds 1 and 2); a University (four distinct unbanned skills, Necromancy included: D-36;
   fewer left, all of them: A-22 fixed); a Find (its table, the artifact from the pool); a Pyramid
   (uniform among the level-5 hero spells with a school, not banned, not `granted_by_name_only`;
   none: no spell, A-20 fixed: it fights and gives nothing).

The draws' order is the engine's (decision 17), so the original's dropped draws (a placed
artifact's, §3.1) have no counterpart. The tutorial is not modelled (decision 134).

###### 10. The state (ADR-0016 point 1; decision 133)

Built by T61b: the army left as `persistent::Table` (a top table of mid tables of 64 leaves of 16
shared entries; the hash's encoding unchanged, so increment 3's replays moved only by the
version); the visits' block as `h3_adventure::Visits`, a public value with its own methods (the
repo forbids crate-private code without a caller, so T62's visits store and write it through
`AdventureState`'s field; nothing outside the state can put one in a game), none in a state until
then, hashed after the army left. Measured: `fork-xl-144` (see the log, T61b).

- **The visits' block** (`Option<Arc<Visits>>`, none until the first visit that writes one):
  - `marks: BTreeMap<HeroId, BitGrid>` — a hero's once-per-hero marks, a bit per object index
    (the grid of `Occupancy::removed`'s shape: 32 objects a row);
  - `known: Vec<BitGrid>` — per player, a bit per object index: the player's knowledge (§13);
  - `emptied: BitGrid` — a find searched, a Pyramid looted.
- **The army left** becomes a two-level table of leaves (64 × 64 entries a mid table, leaves of
  16 shared armies), so a write copies at most a top table, a mid table and a leaf; its fork cost
  stays one reference count.
- **Removals** stay `Occupancy`'s (chests, artifacts, scrolls, Scholars, boxes, events with the
  flag).
- **Sizes** (the XL map's two levels, 41 472 tiles, one object a tile; T55b's measurement, a
  throwaway callgrind program on `BitGrid` and stand-in tables, release build, not committed):

| Candidate | A fork | Its first write |
|---|---|---|
| marks in one map by hero id (8 / 48 / 156 heroes, 30 marks each) | 19 Ir | 2 727 / 7 973 / 26 233 Ir |
| marks on each hero (8 / 48 heroes) | 453 / 1 781 Ir | 1 909 / 3 210 Ir |
| knowledge, 8 players, every object known | 20 Ir | 2 238 Ir |
| an object bit grid (`emptied`) | 40 Ir | 898 Ir |
| army left, today's one map (30 / 1 000 / 41 472 entries) | 19 Ir | 11 086 / 388 877 / 16 459 140 Ir |
| army left, two levels, leaves of 64 armies (30 / 41 472) | 16 Ir | 16 498 / 32 016 Ir |
| the same, armies shared, leaves of 64 / 16 (41 472) | 16 Ir | 4 652 / 4 197 Ir |

  (A write includes about 700 Ir of the value's own allocation.) Today's env fork on the XL map
  is about 3 560 Ir (`fork-xl-144`): the block adds one reference count (about 0.5 %, inside the
  gate's 2 %); the marks on each hero would add 13 %. The build measures again with the real
  types (`fork-xl-144`, and a fork with every tile a box, §16).
- **The hash** walks the block in canonical order (hero ids, players, object indices); the
  adventure state hash's version moves.

###### 11. The visits (`h3-adventure`)

Built by T62a for the pickups, teachers and finds (`state/rewards.rs`), by T62b1 for the record,
Pandora's Box and the Pyramid (`state/records.rs`; decision 151; events are T62b2's): the outcome's `HeroGains` (the changes, then the experience the game gives
with its level-ups), `Choice` (`ChoiceKind::Chest`, `ChoiceKind::Primary`) answered by
`AdventureState::choose` with a `Pick` (first, second, leave), `UniversityVisit`, the offers
`OfferKind::Gift { price }` (the Tree's, the design's `PayTree`) and `OfferKind::Search` (the
Tomb's); `h3-game` applies the hero's gains and, until T63's decisions, answers the choices with
their defaults and leaves the University (`ADV-STUB-10`, decision 150). The calls the build made
are in the log (T62a).

Each kind's visit, in the original's order (survey §2 to §7), returns its outcome (ADR-0017
point 10); a refusal is an outcome. Where the original records the visitor's team mark the
visitor's owner's knowledge bit is set at that point (§13); the hero's once-per-hero mark is set
only when it gives.

- **Chest** (ADV-PIK-005): an artifact with room (§4) → `AddArtifact`, removed; else (gold, or an
  artifact with a full backpack as 1 000 gold) a **choice** decision: gold to the owner or
  `learning_experience(gold − 500)` to the hero; removed either way (the removal with the visit,
  the choice's answer gives).
- **Artifact** (ADV-PIK-006, -007): a full backpack refuses (nothing changes); guards → the
  `Fight` offer, a win's rest picks it up; else `AddArtifact`, removed.
- **Scroll** (ADV-PIK-008): as the artifact, its guards asking only with a text (else a battle at
  once), the Spell Scroll with its spell.
- **Monster's treasure** (ADV-MON-011): after a won battle or a join, the artifact to the hero
  with room, else lost; then the resources (today's).
- **Gifts** (ADV-TCH-001 to -003): a hero's mark set → refused; the condition (the Library) or
  the price unpaid (the Schools, the Tree's, before asking) → refused; a question or choice when
  the type has one; then the gain, the mark. The Learning Stone `learning_experience(1 000)`; the
  Tree `next_level_experience`.
- **Shrine, Witch Hut** (ADV-TCH-004, -005): knowledge first; refused by §4's tests (or no skill
  drawn); else `LearnSpell` / `RaiseSkill { level: 1 }`; no mark, every visit.
- **Scholar** (ADV-TCH-006): `scholar_reward`, removed.
- **University** (ADV-TCH-007): opens the decision (§14) over its four skills; knowledge set.
- **Finds** (ADV-PIK-009 to -012): emptied → nothing (the Tomb's bonus still applies); else the
  contents with §8.2's full-backpack turns; the Tomb asks first and adds `Bonus { morale: −3,
  mark: Some(0) }` whether or not it held something; emptied.
- **Pandora's Box** (ADV-PIK-013): the offer; accepted, guards → a battle (announced, unsplit),
  its rest after the levels; then the record (ADV-RWD-001, §11.1); removed.
- **Pyramid** (ADV-TCH-008): the offer at every visit; knowledge after it; looted → `Bonus { luck:
  −2, mark: Some(1) }`; else the monster battle (§11.2), its rest after the levels: looted (emptied),
  the spell by `can_learn_spell(…, false)` or lost.

**11.1 The record** (ADV-RWD-001, the adventure's half): `grant` (§3) for the hero; the resources
(gains, and losses that floor at 0); the creatures slot by slot onto the army (a stack of its
creature, else the first empty slot, the cap of ADV-ARM-007), what finds no room into **one army
dialog** after the last stack; the experience (`Granted::experience`) is carried in the outcome
for the game to give last (§14). Built by T62b1 (`give_record`): the losses as
`AdventureOutcome::losses` (the game's `ResourceLost`), the group at the record's slots as
`NoRoom::group` (one `ArmyDialogOpened` per stack); *agent's call:* the empty box's message is
presentation, no event (as T62a's refusals).

**11.2 The monster battle with a second creature** (ADV-MON-012, D-40): the Pyramid's guards
through `split` with the first creature's count, n less the second creature's stacks (at least
1), and the order table 0x63DF94's row for n (slot by slot: n = 1 D D – G – – –; 2 D D G – G – –;
3 D G D G – G –; 4 G D G D G – G; 5 G G D G D G G); the split's draws as a monster's
(`ADV-QUIRK-6`). Losses are not kept (no army left). Built by T62b1: `Defender::Monsters`,
`split_with_second` (the rows for n = 1 to 6 with no third creature; E7-16 and E7-22 confirmed
two of them).

###### 12. Events and the move (ADV-OBJ-007, ADV-MOV-011; A-23)

- **Static:** an event is a `MapObject` without a visitable mask in the movement's sense: the
  static map keeps a separate bit per tile ("an event lies here") and the event's index, so the
  movement rules, the occupancy and the observation see nothing unless a rule reads that bit.
- **The step:** after a step onto a tile with a live event whose colours hold the mover's colour
  (a non-human player only with the computer flag, T54's `human`), the event **fires**: the move
  ends on the tile with the points left after the step (the rest of the path dropped; the
  outcome says so), the guards fight at once (a guarded pile's battle, its rest after the levels),
  else the record (§11.1); removed with its flag. A move is never rejected because of an event; a
  hero that starts its move on an event does not fire it; a non-firing event changes nothing.
- **`Reach`** never reads the bit, in every profile: batch E7's E7-23 showed a path crossing a
  hidden event and the re-read found an event's tile no stop for the original's pathfinder (its
  entrance bit cleared at the placement), so A-23 is withdrawn (decision 131; T58, survey
  `E08-rewards.md` §15). No flag, no `ADV-QUIRK-22`.
- **Security:** nothing leaks through the legal moves, so `docs/06-security.md` lists nothing new
  for events. A property test: two games that differ by a hidden event give every player the same
  observation and legal moves until it fires.
- **An event on a guarded tile** (T62b's opening reading, survey §8.5): the guard visits first
  (0x48169D), the event fires after the guard's visit (0x481741). The original fires it whatever
  that visit ended in, also for a hero the guard beat or that retreated (in the pool, owner −1:
  A-26, state-corrupting); fixed, no flag (`ADV-QUIRK-29`, T62b2): the event fires only for a
  hero still on its tile once the guard's visit is over (its offers answered, its battle and
  levels given), and stays otherwise. The wait across the guard's decisions is T63's rest
  (GAM-DEC-007; built by T63b, decision 154: `AdventureOutcome::event_after`; from T62b2 to T63b
  a guarded tile's event did not fire, `ADV-STUB-10`, decision 151).

###### 13. Information (ADV-VIS-006, ADV-FOG-007)

Built by T62a for its kinds: `AdventureObservation::known` (per viewer, the objects still on the
map by type and anchor with what it learned: `Known::Spell`, `Skill`, `Price`, `Skills`, `Emptied`
as the find stands now), `OwnHeroView::marks`. Events (ADV-FOG-007) are T62b's.

- **Object views:** unchanged in what is public (type, anchor, masks): chests, scrolls and boxes
  look alike; events are never in a view; a removal is `ObjectRemoved` where seen.
- **The viewer's knowledge** (a player's `known` bit): its view of the object adds what it learned
  (a Shrine's spell, a hut's skill, a Tree's price, a University's skills, a find emptied, a
  Pyramid looted). Recorded where the original records the team's mark: the Shrines, the Witch
  Hut, the Schools and the Tree before their tests, the Corpse, the Wagon and the Lean To at their
  visit, the Pyramid and the Tomb after their question (the Tomb's order not read: as the
  Pyramid's), the University when it opens. Allies' knowledge comes with teams (E10).
- **Own heroes:** each hero's once-per-hero marks, named by type and anchor as the views name
  objects (a marked object stood on a tile the hero explored, so nothing hidden shows), and its
  bonuses; the open decision's view (§14).
- **Events to viewers:** a visit's gains and changes to the owner alone; an event's firing (its
  record's parts given) to its visitor's owner alone; other players see the steps they see.

##### Part C — the game, the env, batch E7 and the cut

###### 14. The decisions and their answers (`h3-game`; GAM-DEC-005, -006, -004 amended)

Built by T63a (decision 153) and T63b (decision 154: the rest, held by the game in a list beside
the decisions rather than in the army dialog, the guard's visit after a visited object among the
rests; `state/rests.rs`): the typed
answers, the choices and the University (`Decision::Choice`, `Decision::University`; the views
`DecisionView::Choice`, `DecisionView::University { visit, slots }`), the env's answers by role
with four purchase entries (action encoding v5). The record's group dialog came with T62b1.

- **Typed answers** (decision 138; ADR-0017 point 6): `Answer` becomes an enum by
  decision kind, each variant carrying its kind's own type; every decision's answers move,
  increments 2 to 6's with increment 7's, in one replay format bump (v7):

  | Variant | Its type | For | Default |
  |---|---|---|---|
  | `Offer(Consent)` | `Accept`, `Decline` | every yes-or-no offer: the Cartographer, the border guard, a monster's join, join for gold, pursuit, a guarded object's `Fight`; new: `OpenBox`, `SearchPyramid`, `SearchTomb`, `PayTree { price }` | `Decline` |
  | `LevelUp(SkillOffer)` | `First`, `Second` | a level-up with two offers | the original's choice |
  | `Chest(ChestChoice)` | `Gold`, `Experience` | a treasure chest's gold | `Gold` |
  | `Arena(ArenaChoice)` | `Attack`, `Defense` | the Arena (no leaving) | `Attack` |
  | `School(SchoolChoice)` | `First`, `Second`, `Leave` (the School's two skills by its type's order) | a School of Magic or War | `Leave` |
  | `University(UniversityAnswer)` | `Buy(slot)`, `Leave` | the University (open until `Leave`) | `Leave` |
  | `Close` | — | a meeting, an army dialog (increment 6's `Decline`) | `Close` |

  `WrongAnswer` stays for a variant that is not the open decision's kind; a University's `Buy` of
  a closed slot or one the treasury cannot pay is rejected (`SlotClosed`, `CannotPay`) and changes
  nothing. The system actor's `answer_default` (its closed command set, ADR-0017 point 2) gives
  the default per kind.
- **`Decision`** gains: `Choice { owner, hero, object, kind, amounts }` (the chest: gold and
  experience; the Arena; a School: its two skills and price); `University { owner, hero, object,
  skills, price }`; the new `OfferKind`s beside increment 3's `Fight`; `ArmyDialog` gains the
  group of up to seven stacks and a **rest** (`Option<Rest>`).
- **The rest** (GAM-DEC-007, new; decision 141): what a visit owes after a decision closes or a
  battle's levels are given: the experience (and its level-ups), the pickup or the record after a
  won guards' battle, the Pyramid's spell. The game holds it beside the decisions (the hero, the
  object, what remains) and runs it when the hero has no level-up pending and no army dialog open
  (at once when none). Its owner sees nothing of it but the decisions it leads to.
- **Views** (`DecisionView`): the choice's amounts; the offer's price (the Tree); the University's
  skills and slot states; the dialog's group.
- **`legal_commands`:** each decision's answers: both of a choice (and `Leave` for a School), one
  `Buy` per open slot the treasury can pay and `Leave` for a University, `Close` for a meeting or
  an army dialog.

###### 15. The game's application

Built by T62a, T62b1 and T63b (decision 154: the experience after the army dialog, `ClearBonuses`
in `end_battle`).

- The adventure's outcome's hero changes applied in order through `h3-heroes` (as today); the
  resources lost floor at 0 (GAM-RES); `Granted::experience` given after the army dialog or at
  once without one, then its level-ups (increment 4's decisions).
- **After every battle** (GAM-BAT-014, new): `ClearBonuses` for each hero of the battle,
  winner and loser, before the winner's level-ups (the original's `battle_end` clears first).
- The scenario's bans to the adventure's build (§7).

###### 16. The env, replays, fixtures, properties, benchmarks

- **The env** (the AI lane's): its action mapping (`h3-sim`'s `adventure/action.rs`, which builds
  `Answer::Accept` and the others from its entries) moves onto the typed answers in T63
  (`cross-lane`, decision 138's consent): its accept, decline, first and second entries onto
  `Offer`, `LevelUp`, `Close` and the new two-way choices; its masks allow at least one answer per
  new decision (the defaults of §14); a University's purchase head is the AI lane's call. Its
  inbox is told (T55b).
- **Replays:** format v7 (the typed answers); the twelve goldens re-recorded; new: random play on a
  rewards fixture `fixtures/maps/e08-rewards-*.h3m` (every class of increment 7, written by
  `h3_map::writer`), and scripted replays of the order-dependent paths: a box with creatures and
  no room (the dialog, then the experience), an event that stops a move, a University's purchases,
  a won guarded box whose level-up gives Wisdom before its spells.
- **Properties** (`h3-map`'s `play.rs`): the plans visit every kind, answer every decision, check
  the invariants after every command (marks only grow, an emptied find gives nothing, a removed
  object never returns); a rejected command leaves the hash; the hidden-event property (§12); **decision 133's test**: a 144 × 144 two-level map with a box on every free tile
  (and one with a Learning Stone on every free tile) builds, plays a few hundred steps with
  visits, and forks within the gate's cost.
- **Benchmarks:** `random-play` on the rewards map (a new workload); `fork-xl-144` with the block
  live (+ about 20 Ir); a fork of the all-boxes map (a new workload); the reach workloads
  unchanged (events are blind to `Reach`, §12).

###### 17. Versions and the remaining rule ids

| Version | Change | Task |
|---|---|---|
| core ruleset hash v49, pack format v48 | the mana ceiling; A-21's and A-25's flags (A-23 withdrawn, T58) | T59 (done) |
| details' hash, game state hash v15 | the battle bonuses | T60a (done) |
| adventure ruleset part v12 | the kinds | T61a (done) |
| map import v12 | the classes, bodies, records, the monster's artifact (the bans: T61b, import v13) | T61a (done) |
| adventure state hash v13 | the objects' holdings and a monster's treasure artifact | T61a (done) |
| map import v13 | the scenario's banned spells and artifacts | T61b (done) |
| adventure state hash v14 | the draws on objects and the visits' block (the army left's table keeps its encoding) | T61b (done) |
| adventure state hash v15 | the players' seats (events) | T62b2 (done) |
| game state hash v16 to v19 | the offers (T62a), the record (T62b1), the decisions and their typed answers (T63a), the rests (T63b) | T62a to T63b (done) |
| adventure replay format v7 | the typed answers (every decision's) | T63a (done) |

Ids beyond survey §12: **GAM-SET-003** (the scenario's spell and artifact bans as setup inputs),
**GAM-DEC-007** (the rest), **GAM-BAT-014** (the bonuses cleared at a battle's end). Quirks (T59,
in `docs/rules/quirks-adventure.md`): A-21 → `ADV-QUIRK-21`, `quirk.adventure.roe_witch_hut_mask`
(flag 37, `ROE_WITCH_HUT_MASK`); A-25 (decision 145) → `ADV-QUIRK-22`,
`quirk.adventure.full_backpack_refuses` (flag 38, `FULL_BACKPACK_REFUSES`): `faithful` the count
test, `strict-fixed` the placement first (survey §15); A-23 withdrawn (T58: no quirk, no flag);
A-17 to A-20, A-22 and A-24 recorded in the ledger as not reproduced or fixed
(no flags).

###### 18. Batch E7

Survey §11's 23 rows in two runs (decision 132), on the harness (`clients/capture/`, common) in SoD
on Easy, hot seat, from maps `h3-experiment` writes; the runs E7a (rows 1–11; row 7 on a map of its
own in the RoE format) and E7b (rows 12–23).

- **The writer** (`h3_map::writer`, this lane's): the object bodies of classes 5, 6, 22, 26, 63,
  65, 88, 93, 100, 101, 104, 105, 108 and 113 (a box's and an event's record, guards, text,
  colours and flags; an artifact's guards and text), a monster's treasure artifact, and an RoE
  map (format 0x0E: its header, no masks).
- **The spec** (`h3-experiment`, battle-owned: `cross-lane`): the rows' objects, heroes' artifacts
  and backpacks (64), skills, spell books, gold; the two maps of E7a and E7b's; the results
  templates `experiments/results/E08-batch-7a.md`, `-7b.md`.
- **The harness** (common; CLT-Q18: this lane builds it with its check and tells the client and
  map-client lanes' inboxes): reads of an object's word on its tile, the game's universities
  (+0x4E3CC), a hero's experience, primary and secondary skills, spell points, bonuses (+0x11A,
  +0x11B) and status flags, the backpack's count, the treasury; the two-picture choice's items
  (0x7809, 0x780A); the University dialog's slots and its purchase button; the path marks (batch
  E1a's) for E7-23.
- **The scoring:** survey §15 (new), each row against §11's prediction; decision 131's outcome
  for A-23 (E7-23); a dispute a row contradicts reverts to the other side (decision 129) or
  becomes an `ADV-Q`; the consensus rows' sources `verified`.

###### 19. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T59's `shared/` task (the ADRs applied with batch E7's outcome, the
mana ceiling, the quirk flags, `docs/03-domain.md`'s terms: reward record, once-per-hero mark,
knowledge, battle bonuses, event); batch E7's spec (`h3-experiment`, `cross-lane`); the harness
(common). Everything else is the lane's own.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E7 runs before the build** (decision 132) **and before the shared
task**, so the ADRs are applied once with E7-23's outcome and the disputes' verdicts (in
increment 6 the shared task came first; here the batch decides an ADR clause, A-23):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T56** | Batch E7's maps: the writer's bodies and the RoE format (§18), the spec's fields (`cross-lane`), the two specs and templates. Likely cut in two at its start (the writer; the spec) |
| 2 | **T57** + **T58** | T57: the harness's reads and clicks, the recipes, the runs E7a and E7b (as T47: cut at its start). T58: scored (survey §15); A-23 settled (decision 131); an `ADV-Q` if a row contradicts the reading |
| 3 | **T59** | `shared/`: the three ADRs amended (with T58's outcome: A-23 withdrawn, events blind to `Reach`), the mana ceiling, A-21's and A-25's flags, the glossary |
| 4 | **T60** | `h3-heroes`: part A (§1–§6): the bonuses, the changes, `grant`, the teachers' functions; the details' hash |
| 5 | **T61** | `h3-map` and `h3-adventure`: the import (§7), the kinds (§8), the draws (§9), the state (§10). In two (decision 148): T61a the import and kinds (done: map import v12, ruleset part v12, adventure state hash v13); T61b the bans, the draws and the state (done: map import v13, adventure state hash v14) |
| 6 | **T62** | `h3-adventure`: the visits (§11), events and the move (§12), information (§13). In two (decision 150): T62a the pickups, teachers and finds, the game applying the hero's gains (done: game state hash v16); T62b boxes, events, the Pyramid |
| 7 | **T63** | `h3-game`: the decisions, their typed answers (every decision's, migrated; `h3-sim`'s action mapping `cross-lane`) and the rest (§14), the application (§15), replay format v7. Likely cut in two (the typed answers' migration; increment 7's decisions). Done in two (decision 153): T63a, T63b |
| 8 | **T64** + **T65** | T64: replays, fixtures, properties, benchmarks, `--with-original-data` (§16). T65: the audit against the exit criteria, increment 7 closed, increment 8 refined. Done: T64 in two (decision 155), T64a and T64b; T65 whole (decision 157) |

**Increment 7's exit criteria:**
1. Treasure chests, placed and random artifacts (guarded too), Spell Scrolls and the monsters'
   treasure artifacts are taken as the original: the chest's choice, a full backpack refusing,
   turning or losing as survey §8.2; the 8 maps refused for a monster's artifact import further
   (ADV-PIK-005 to -008, ADV-OBJ-006, ADV-MON-011).
2. The twenty classes that teach or hold a find give, ask, charge and refuse as the original;
   once-per-hero marks and the finds' emptying have no bound (ADV-TCH-001 to -007, ADV-PIK-009 to
   -012).
3. Pandora's Box and events give the reward record in the original's order (D-37 to D-39 as batch
   E7 decided); their guards fight as a guarded pile's; an event fires for its colours and stops
   the move; leftover creatures go through the army dialog and the experience comes after it
   (ADV-PIK-013, ADV-OBJ-007, ADV-RWD-001, HER-CHG-003, GAM-DEC-004).
4. The Pyramid draws its spell, asks at every visit, fights with a second creature split as the
   original, teaches or loses its spell and costs a looted visitor −2 luck once (ADV-TCH-008,
   ADV-MON-012).
5. The morale and luck bonuses reach the hero's next battle and end with it (HER-ADV-008,
   GAM-BAT-014); a won guards' battle's levels come before the visit's rest (GAM-DEC-007).
6. Contents and draws stay hidden until a visit; the visitor's owner keeps what it learned; events
   are never observed and blind to `Reach` (A-23 withdrawn by batch E7; ADV-VIS-006, ADV-FOG-007, ADV-MOV-011); a map with
   a box on every tile builds, plays and forks (decision 133).
7. Every new rule has its id, source and tests; A-21 and A-25 in the ledger with their flags
   (A-23 withdrawn, T58), A-17 to A-20, A-22, A-24 recorded; the gates pass (coverage, determinism with the
   re-recorded and new replays, the benchmarks at their baselines and the new ones baselined); the
   env plays on through every new decision.
8. Batch E7 run and scored; D-29 to D-31, D-33, D-34, D-36 to D-40 and the consensus rows
   `verified` or reverted.

###### 20. Risks and open points

- **The typed answers' migration** (decision 138) touches every caller that builds an answer:
  the AI lane's action mapping (`cross-lane`), the CLI's replay files, the properties' plans and
  the tests; done first in T63, with the twelve goldens re-recorded, so increment 7's decisions
  land on the new types.
- **The rest across decisions** (§14) is the increment's new mechanism: a visit that spans a
  battle, its level-ups and an army dialog. The scripted replays (§16) pin its order; a forgotten
  path would give a box's experience before its dialog (VCMI's order, D-39).
- ~~**The import's reach**~~ (T61b's census): 8 maps import whole; the rest stop at towns (128)
  and random heroes (21) or a hero in a town (3).
- **Batch E7's size:** 23 rows, two maps for E7a, a harness grown by about ten reads; the cut of
  T56 and T57 at their starts (as T47's three parts).
- ~~**A-23**~~ withdrawn by T58 (survey `E08-rewards.md` §15): the move's stop at a firing event
  is the rule (§12) and `Reach`'s callers (the env's masks) do not change.
- ~~**The army left's table**~~ (§10, T61b): built with the hash's encoding unchanged; the
  replays' commands did not move.
- **Not in increment 7** (decisions 127, 136): the creature banks, the Refugee Camp, Seer's Huts
  and quest guards, the map's timed events, the computer's answers and the end-of-game tests after
  an event (E10), casting the adventure spells scrolls hold, teams (allies' knowledge, E10).

### Increment 8, round 1: T66a1, the boats and sea survey's first part (2026-10-28, `adventure/e08-t66a1`)

Baseline: `master` at e5cfa46 (#546 and #534 merged; CI's run on it failed in the ai lane's
code, below). Original files staged from the user's computer: `Heroes3.exe`, `maps.tar`,
`extracted.tar`. At the task's start the user cut T66a in two (decision 160, `AskUserQuestion`;
measured on the human player's path, about 2 300 to 2 800 instructions not read: 1 600 to 2 000
for the boats, about 700 for the pickups; not chosen: whole, three parts).

**Done:** survey `docs/rules/surveys/E08-sea.md` §0 to §6 (the census of the 160 maps with a
throwaway example; the boats' records and import, boarding, landing, the guards, sailing, the
day's points at sea, what a boat visits, events on water, scouting, what another player sees, the
writer's boat; disputes D-41 to D-44); the address map (`create_boat` 0x4BB250,
`boat_free_record` 0x4BB170, `summon_boat_choice` 0x4E54A0 read; `hero_boards_boat` and
`hero_lands_from_boat` read whole; `hero_step_allowed` read for its boat and water branches;
`pathfinder_run`'s boat leaf, `pathfinder_expand`'s boat node, the move step's boat sites, the
loader's case 8, the coast's class 3; nine inferred entries for the paths not followed) and
`structs.toml` (the boat's fields, the tile's class 3). A first reading of the boat's search had a
quirk candidate (a boat's path crossing a coastal strip); `pathfinder_run` expands a boat node only
on water (0x56BBC1), so it was dropped before it got a number (A-27 stays free). No code, no
version, no rule id (T66a2 proposes them).

**The census in short:** 118 boats on 41 maps (at most 15), no hero placed on water, 9 events
on water (the queue inbox's note of 2026-10-27, confirmed and moved to `QUEUE-ARCHIVE.md`), 48
on coastal land, Favorable Winds on 3 maps; Race for Ardintinny imports whole with 5 boats and
128 of the sea's pickups.

**CI on `master`:** the run on e5cfa46 (#534, the ai lane's E25 T1a to T2a, merged after #546)
failed in `cargo xtask verify` (the log is not reachable from the sandbox; its annotations say
only "exit code 1"). This task changes docs only. The sandbox's full verify of the same code
(fingerprint 7c14229…, without the original LODs) passed all fifteen steps (`verify-once:
verified (sandbox) code 7c1422972258263eff244e4168c2e132cfdec7b7 on 317cd61`), so the failure looks
like the runner's, not the code's; a re-run of CI's job on `master` would tell.

### Increment 8, round 1: T66a2, the boats and sea survey's second part (2026-10-28, `adventure/e08-t66a2`)

Baseline: `master` at 86c2d35 (#550 merged; CI's run on e5cfa46, #534, shows success now and every
later run on `master` passed, so T66a1's reported failure is settled). Original files staged from
the user's computer: `Heroes3.exe`, `maps.tar`, `extracted.tar`. At the task's start the user took
T66a2 whole (decision 161, `AskUserQuestion`; not chosen: two parts, the reading then the plan;
three parts).

**Done:** survey `docs/rules/surveys/E08-sea.md` §7 to §14 and §0's T66a2 part: the battle at
sea (the deck by either hero's boat, the two boats by both, nothing else in a battle tests a boat:
every use of the in-boat bit listed), who fights a hero at sea, the boat gone with its hero, VCMI
and the wiki in consensus; the four pickups' draws and visits read whole (Flotsam, Sea Chest,
Shipwreck Survivor, Ocean Bottle with the signs' loader and default texts), who visits them (a
boat; a flyer from the shore by a click: D-45), VCMI and the wiki; the interfaces (no new
command or decision, `Reach`'s boat layer, the state, the observation, the env); batch E8's plan
(16 rows in two runs, the harness's and the spec's gaps); the proposed rule ids (`ADV-SEA-001` to
`-007`, ADV-PIK-014 to -018, HER-ADV-009, amendments); the second reading by two fresh agents;
the checkpoint's questions (ADV-Q32 in `OPEN-QUESTIONS.md`). The second reading corrected three
points: the day's maximum is kept without the Admiral's Hat (A-28), the drop at sea is 66 on a
Favorable Winds tile, and a water walker never takes a sea pickup from the shore. The census grew (§1: objects on water
by class, 55 monsters on water, 3 guarded events on water, 395 pickups beside land, the bottles'
texts). The address map: `visit_flotsam` 0x4A2230, `visit_sea_chest` 0x4A5030,
`visit_shipwreck_survivor` 0x4A52A0, `load_message_body` 0x501270, `hero_step_or_meet_allowed`
0x481AD0, `sign_default_texts` 0x696C74 (read), six entries amended. No code, no version.
