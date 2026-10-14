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
