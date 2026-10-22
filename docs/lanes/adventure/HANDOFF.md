# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1 to 4 **closed** (2026-10-12, -13, -20, -21). **Increment 5: the hero's movement and scouting** (decisions 71–83; flying and water walking in): survey `E08-hero-movement.md` (T34a), design `E08-design.md`, batch E5 first (T34b; ADV-Q21). Round 1 **closed**: T35a (#377, ADRs), T35b (#380: six hero-effect kinds, flags 30–32; no reader until T39–T41), T36a, T36b (#383, #387): batch E5's specs, scorer. Round 2: T37a (#389), T37b (decision 83: every batch run; **E5-4a against D-24**), T38. Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 8, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; of the 160 shipped maps 5 import whole (9 heroes, 41 levels at load), the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps.
- **Monsters, grounds, guarded piles** (T20a2a to T21a; ADV-MON-002 to -010, ADV-GRD-001, ADV-PIK-004): kinds `Monster`, `Ground`; `[params.monsters]`; `ObjectSetup.monster`/`.drawn`; `army_left`; `battle_ground`; `guard_of`; `answer.rs` (`Join`, `JoinForGold`, `Pursue`); `split` (flag 24); `Defender::{Monster, Guards}`, `Actor::System`, `end_monster_battle`, `end_unowned_battle`; `MonsterChanged`; `state/piles.rs` (`Guards`, `OfferKind::Fight`).
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId` (`GameHeroSetup.details` or defaults, validated by `h3-heroes`); `attack_defense`, `join_bonus`; `GameObservation.heroes`. Progression (T26b–T30): `h3_heroes`' `level.rs`, `level_up.rs` (`draw_level_up`), `GainLevel`, `offered`; no level above the experience's; `original_choice` (HER-LVL-006); `state/levels.rs`' `Progression` (GAM-HER-002, flag 27), levels at load (GAM-SET-002), `StaticMap::terrain_shares`; levels in play (`give_levels`, `Decision::LevelUp`, `DecisionView`, `default_answer`; GAM-DEC-002, GAM-BAT-008).
- **Objects** (T12–T15): types (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removals (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at`, `tile_object`); gains and offers in `AdventureOutcome` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001 to -004); `offer_stands`.
- **Battles** (T19a1, GAM-BAT-001 to -005; `h3_game::state::battle`): a step toward another player's hero visits it (`BattleStart`, `battle_ground`), `Decision::Battle(Arc<OpenBattle>)` both owners play (`GameCommand::Battle`), seeds under `ADV-QUIRK-5`, events and `GameObservation.battle` to the owners; the outcome's armies, `HeroRemoved`, a surrender's gold; the heroes' part (`heroes_after_battle`).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc` with the coast bits, heroes by id with their movement points and formation, the difficulty, state hash v10; ADV-MAP-001, ADV-MOV-001/-002); `h3_game::{GameRuleset, GameSetup, GameState}`: a stream per player, sequential turn groups, treasuries, pending decisions (GAM-SET-001, GAM-TRN-001/002, GAM-CAL-001, GAM-RES-001/-002, GAM-DEC-001). Game state hash v11; `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006, `ADV-QUIRK-2`/`-3`), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `reach` (`Reach`, one path per tile as the original's); no step down onto a hero on a tile closed above (T33, ADV-OBJ-004).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands** (T8b2): `h3_game::legal_commands` and `legal_moves` (a hero's `Reach`, the user), empty outside the turn.
- **The env** (T9a, `h3-sim/src/adventure/`): `AdventureEnv` (reset, `to_move`, masks, steps, fork, hash, the view, a day limit), action encoding v4 since T30 (`Phase`; `MapAction`: `EndTurn`, eight hero slots, `Accept`, `Decline`, `First`, `Second`, a tile; in a battle the battle's encoding; `to_move` yields `Seat`s, since T20b3a `Seat::Neutral` for an unowned side, played as `Actor::System` with `neutral_observation`), `random_rollout` (every seat).
- **Replays and benchmarks** (T9b): `adventure::replay` (format v5), `h3 adventure run/verify/bench`, nine golden replays (re-record with the options in `crates/h3-cli/tests/adventure_replays.rs`), the maps `fixtures/maps/e08-{duel-36,xl-144}.h3m` (`h3-map`'s `tests/fixtures.rs`; since T31 heroes near a level, one levelled at load), three gated workloads in `benches/adventure/` (play, reach, fork with a battle open).
- **Experiments**: `h3-experiment` (battle's) has the adventure shape (T22a1, T32a, T36a, T36b); batches 1, 2, E3, E4 scored (T33); E5a–E5e run (T37b; E5c, E5d twice), to score; `crates/h3-map/examples/e4_score.rs` scores E4 and E5-2, -3, -10. **Agents run batches** on the client lane's harness (decision 21; changed `cross-lane` by T32b1 to T37b; the runbook).
- **Adventure ruleset part** (v9 since T29: costs by Pathfinding level): terrains, roads, rivers, movement and setup parameters, object types, starting resources by difficulty, battle terrains, monsters' parameters (with the price resource).
- **Properties** on generated playable maps (`h3-map`'s `play.rs`, `view.rs`): `Step::Fight` runs (hero casts via `h3-battle`, a dev-dependency), monsters, grounds, the system actor, guarded piles, level-ups (T30), `check_progression` after every command (T31); `the_plans_play_battles_to_a_victory`: 192 fixed cases.
- **Next:** the T37b pull request (`cross-lane`) to merge; then T38 (batch E5 scored; an ADV-Q for D-24).
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 (#115), 3 (T18a, #185, with ADR-0013), 4 (T26a, #325) and 5 (T35a, #377: ADR-0013, -0016, -0017).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ownership:** the four crates and `h3-heroes` (since E04c), `src/adventure/` in `h3-sim` and `h3-cli` (their `Cargo.toml`, `lib.rs` and `h3-cli`'s `main.rs` are battle's: `cross-lane`), `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, `benches/adventure/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`.
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fog fringe; 2, 3 the diagonal exception; 4 leading neutral stacks; 5, 6 the tile's obstacles and split; 7 A-6; 8 the map's formation; flags 19–24; 16–18 increment 5's, flags 30–32), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- Stage archives, not files (~13 k tokens per 50): the user's `original-data/maps.tar` (the shipped maps; untar into `original-data/install`) and `extracted.tar` (the text tables); the sprite archives for `.msk` sizes. `sandbox-setup.sh --lane adventure` copies only the engine's files (copy the harness's DLLs, sprite LODs and sound archives to `original-data/install`) and skips the client gate's SDL packages.
- A shallow clone: `git fetch --depth=60 origin master` before `sandbox-setup.sh`, or its baseline runs in full.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-22 — increment 5, seventh session: T37b

Baseline f494cb1 (full verify). Decision 83. Every batch clean at its first run; E5-4a against D-24, the rest as read.

### 2026-10-22 — increment 5, sixth session: T37a

Baseline 1b280e0 (full verify). Decision 82 (T37 in two). The harness's reads; #389.

### 2026-10-22 — increment 5, fifth session: T36b

Baseline c84370f (full verify). Decision 81. #387.
