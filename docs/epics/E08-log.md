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
untouched) and offered a generated map for T9's replays, the editor maps then only reader
fixtures for CI; the user decides at the checkpoint.

- **`legal_commands`** (`h3-game`, `legal.rs`; `h3_game::legal_commands(state, rules, actor)`):
  `EndTurn`, then for each of the actor's heroes by id a `MoveHero` to each tile of the
  reachable-tiles query, row by row, along the query's path; nothing for a player whose turn is
  not running or who is not in the game, through `apply`'s own turn check (`check_acting`).
  `MoveHero` is listed once per tile: `apply` also accepts every other payable path (one back to
  the hero's tile, or one the query does not keep, `Reach`'s doc), so for moves the legal set is
  one path per destination, not every accepted command. Unit tests on the 4 × 4 map: the order,
  the query's paths and points, nothing outside the turn.
- **Playable maps** (`h3-map`'s `tests/play.rs`): a `Plan` (version, side 5–16, one or two
  levels, mostly land with a twentieth each water and rock, roads and rivers, two to four colours,
  two to six heroes with distinct types and custom or starting armies of base creatures, up to
  eight blockers of three shapes, one visited from a tile outside its blocked ones) laid out as a
  raw map (heroes on distinct land tiles, owners round the players, blockers covering a hero's
  tile left out), written by the test encoder, read back and imported, so every case plays an
  imported file. At the end of a case players have explored a third to all of their map.
- **The driver:** steps resolved against the state: the acting player's `n`-th legal command, an
  `EndTurn`, a legal move edited (last step dropped, one more step, one more step onto a hidden
  tile, a step turned, another hero, another player), or a free command (`EndTurn` or a move by
  the acting player, a player of the game or one past them; walks in given directions with a
  jump anywhere, or a straight line to a tile across explored and hidden tiles; the hero the
  actor's own or any id up to one past the heroes). Over 48 cases of up to 40 steps about three
  in four commands apply (a quarter of them `EndTurn`; a day passes about once per case) and
  every rejection kind occurs.
- **The properties** (48 cases each, both profiles in the first): at every step only the acting
  player has legal commands, `EndTurn` first, then a move to each tile the query gives its heroes,
  by hero and row by row; every listed move applies and leaves the hero on its last tile with the
  query's points, one event per step (legality consistency); a rejected
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
  half its cases (the split is now an index into at least one step), the listed moves are
  compared with the query's tiles and an unlisted one-step move's tile looked up among them, the
  isolation check compares the hero counts before zipping, the edit onto a hidden tile tries all
  eight directions, the determinism test plays the second run on rules compiled again. Deferred:
  decision 9 says ownership isolation covers the players' streams; no command draws yet and the
  streams have no accessor (inbox).
- **Not done here:** observation hygiene and the observation against the events (T8b2b).
