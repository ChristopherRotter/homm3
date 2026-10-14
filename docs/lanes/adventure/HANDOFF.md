# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1 and 2 **closed** (2026-10-12, -13). **Increment 3** (monsters and battles, decisions 19–28): T17a (survey `E08-battles.md`), T17b (the ADR amendments), **round 1 done** 2026-10-14 (trimmed by the user, decision 28): T18a (#185, `shared/` + `cross-lane`: the amendments applied, `h3-heroes` in the shared layer, four effect kinds) and T18b (`adventure/e08-t18b`, `cross-lane`: the store of heroes). Design and cut: `E08-design.md`. **Open:** #185, then #188 (T18b, stacked on it). Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 4, `docs/formats/h3m.md` "Import": object types by class and subclass, a pile's fixed amount, guarded piles `Unsupported`, the default difficulty, since T18b the heroes' formation and customised details, an experience of 1 000 or more `Unsupported`). Every shipped map's static layer imports and validates; 5 maps import whole, the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps.
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId` (`hero`, `heroes`), built from `GameHeroSetup.details` (`HeroDetailsSetup`) or the defaults (40–90 experience from the setup stream after the armies, the class's primary skills, the type's skills, spells and spell book, the rolled war machines' artifacts), spell points full (`h3_heroes::max_mana`, HER-MAN-001), validated by `h3-heroes`; `GameObservation.heroes` (`HeroInfo`: the type of each hero seen, the owner's details).
- **Objects** (T12–T15): types in the ruleset part (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removed objects (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at(part, player, pos)`, `tile_object`, `PastVisit`, `FromAbove`); gains and the offer in `AdventureOutcome`, `ObjectRemoved` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001, -003); the observatory (-002); offers `Reveal { price }`, `Pass` (-004), `offer_stands`.
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc`, heroes by id with their movement points and formation, state hash v7; ADV-MAP-001, ADV-MOV-001/-002); `h3_game::{GameRuleset, GameSetup, GameState}`: the starting armies (GAM-SET-001), a stream per player, sequential turn groups, day 1, `apply` with `EndTurn` and `Adventure(MoveHero)` (GAM-TRN-001/002, GAM-CAL-001), treasuries in one shared block (T13: GAM-RES-001/-002, `GameSetup.difficulty`, gains with `ResourceGained` to the owner), pending decisions (T14b: GAM-DEC-001, `Answer`, only its answers legal while open; `OfferGone` when the object left, then `Decline` only). game state hash v5. `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006: the source tile prices a step, the diagonal exception under `ADV-QUIRK-2`/`-3`, points below the cheapest next step lost), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `AdventureState::apply` with `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `AdventureState::reach` (`Reach`, one path per tile as the original's; the user keeps it).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands and properties** (T8b2): `h3_game::legal_commands` (`EndTurn`) and `legal_moves` (a hero's `Reach`, not a list of moves, the user), empty outside the turn; the properties on generated playable maps (`h3-map`'s `play.rs`, `view.rs`).
- **The env** (T9a, `h3-sim/src/adventure/`): `AdventureEnv` (reset, `to_move`, `legal_mask`/`tile_mask`, `step`/`step_with`, `step_decoded`/`step_command` since T9b, fork, hash, the view as observation, a day limit), the action encoding v2 (`EndTurn`, eight hero slots, `Accept`, `Decline`; a destination tile, tile 0 for the rest; the query's path), `random_rollout`, `reset_with_difficulty`.
- **Replays and benchmarks** (T9b): `adventure::replay` (format v2), `h3 adventure run/verify/bench` (`crates/h3-cli/src/adventure/`), six golden replays in `fixtures/replays/adventure/` (determinism matrix; re-record with the options in `crates/h3-cli/tests/adventure_replays.rs`), the maps `fixtures/maps/e08-duel-36.h3m` and `e08-xl-144.h3m` (`h3-map`'s `tests/fixtures.rs`), three gated workloads in `benches/adventure/` (play, reach, fork; `02-architecture.md` §8).
- **Experiments**: `h3-experiment` (battle-owned) has the adventure shape (`docs/experiments/README.md`; the spec stamp, `[[map.pile]]`); batches 1 and 2 scored (T10c, T16c). **From increment 3 agents run batches** on the client lane's harness (decision 21; increment 3's needs in its inbox since T17b: `E08-battles.md` §13).
- **Adventure ruleset part** (v5 since T13a): terrains, roads, rivers, native cost, land table, base scouting radius, the setup bounds (`max_players` ≤ `MAX_PLAYERS` = 8), the starting stack chances, the object types, the starting resources by difficulty.
- **Readings** (surveys `E08-adventure-core.md`, `-objects.md`, `-battles.md`): conflicts with the wiki or VCMI go to the user.
- **Next:** #185 merged, then #188; then round 2: T19a + T19b (the queue).
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), their increment-2 amendments (#115), increment 3's (ADR-0013 too) applied by T18a (#185).
- **Fuzzing is stable-only** (user, 2026-10-10): property tests on arbitrary bytes.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ownership:** the four crates, `src/adventure/` in `h3-sim` and `h3-cli` (their `Cargo.toml`, `lib.rs` and `h3-cli`'s `main.rs` are battle's: `cross-lane`), `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, `benches/adventure/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`. `h3-heroes` moves to this lane as soon as the battle lane is done with it (decision 25; its inbox).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fog fringe; 2, 3 the diagonal exception; 4 leading neutral stacks; 5, 6 the tile's obstacles and split; flags 19–22 in `h3_content`'s registry, 22 the fringe's), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; token efficiency first.

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- Staging the shipped maps costs ~13 k tokens per 50 files: pack them into one archive in the user's `original-data/` (gitignored; `tar -cf original-data/maps.tar -C original-data/install Maps` through the device shell, it is there since 2026-10-11) and stage that one file.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-14 — round 1: T18a and T18b

The user trimmed the round (decision 28). T18a #185: `cross-lane` for the battle lane's syntax test (the kinds in `synthetic-ext`). T18b: the store, HER-MAN-001 (code reading), the import of customisations. `master` was red before (BAT-Q41 cites an unmapped address; `battle/e04c-t5` maps it).

### 2026-10-13 — T17a checkpoint and T17b: the ADR amendment drafts

ADV-Q12, ADV-Q13 answered (decisions 22–27: the hero model, ADV-QUIRK-5/-6, the original's battle AI). T17b: the drafts, the design note, the cut, inbox lines for the client and battle lanes (`cross-lane`).

### 2026-10-13 — T16c checkpoint and T17a: the battles survey

The user took T17a alone: the survey (a code reading re-read by a fresh agent, VCMI, the wiki, a census); ADV-Q12.
