# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1 to 5 **closed** (5, movement and scouting: decisions 71–89, survey `E08-hero-movement.md`). **Increment 6: army and hero management** (decisions 90–111; survey `E08-army.md`, design `E08-design.md`): rounds 1 to 3 done (T44a to T49b: survey, ADRs, the env to the AI lane, batch E6 run and scored as read; the positions; moves, combinations, Titan's book, Scholar); round 4: T50 done (decisions 113–115). Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 10: T49a's worn positions, T49b's Titan's book equipped, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; of the 160 shipped maps 5 import whole (9 heroes, 41 levels at load), the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps.
- **Monsters, grounds, guarded piles** (T20a2a to T21a; ADV-MON-002 to -010, ADV-GRD-001, ADV-PIK-004): kinds `Monster`, `Ground`; `[params.monsters]`; `ObjectSetup.monster`/`.drawn`; `army_left`; `battle_ground`; `guard_of`; `answer.rs`; `split` (flag 24); `Defender::{Monster, Guards}`, `Actor::System`, `end_monster_battle`; `MonsterChanged`; `state/piles.rs`.
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId` (`GameHeroSetup.details` or defaults, validated by `h3-heroes`); `attack_defense`, `join_bonus`; `GameObservation.heroes`. T39: `movement.rs` (HER-ADV-003 to -007). T49a: `worn` by position (`positions.rs`, `PlaceArtifact`; re-exported by `h3_game`). T49b: `moves.rs`, `combine.rs`, `equip` (Titan's book), `meeting_spells`. Progression (T26b–T30): `level.rs`, `level_up.rs`, `GainLevel`, `offered`; `original_choice` (HER-LVL-006); `state/levels.rs` (GAM-HER-002, flag 27), levels at load (GAM-SET-002), in play (`give_levels`, `Decision::LevelUp`; GAM-DEC-002, GAM-BAT-008).
- **Objects** (T12–T15): types (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removals (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at`, `tile_object`); gains and offers in `AdventureOutcome` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001 to -004); `offer_stands`.
- **Battles** (T19a1, GAM-BAT-001 to -005; `h3_game::state::battle`): a step toward another player's hero visits it (`BattleStart`, `battle_ground`), `Decision::Battle(Arc<OpenBattle>)` both owners play (`GameCommand::Battle`), seeds under `ADV-QUIRK-5`, events and `GameObservation.battle` to the owners; the outcome's armies, `HeroRemoved`, a surrender's gold; the heroes' part (`heroes_after_battle`).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc`, heroes by id with points, formation and tactics flag, the difficulty, state hash v11; ADV-MAP-001); T40: `start_game` after the levels at load, `start_turn`, `start_day` (`ADV-QUIRK-18`) read the store; `h3_game::{GameRuleset, GameSetup, GameState}`: a stream per player, sequential turn groups, treasuries, pending decisions (GAM-SET-001, GAM-TRN-*, GAM-CAL-001, GAM-RES-*, GAM-DEC-001). Game state hash v12 (T49a); `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006, `ADV-QUIRK-2`/`-3`; T40 Pathfinding, walkers, `no_native`; T41 fly and water-walk prices, `ADV-QUIRK-17`), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `reach` (one path per tile); T41: a flyer's or water walker's passage (`state/passage.rs`, `NoStop`, ADV-MOV-009, -010, `ADV-QUIRK-15`) and pass nodes; no step down onto a hero on a tile closed above (T33).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands** (T8b2): `h3_game::legal_commands` and `legal_moves` (a hero's `Reach`, the user), empty outside the turn.
- **The env** (T9a; the AI lane's since T45 but `replay.rs`): `AdventureEnv`, action encoding v4 (`EndTurn`, eight hero slots, the answers, a tile; in a battle the battle's; `to_move` yields `Seat`s, `Seat::Neutral` played as `Actor::System`), `random_rollout`.
- **Replays and benchmarks** (T9b): `adventure::replay` (format v5), `h3 adventure run/verify/bench`, ten golden replays (options in `adventure_replays.rs`), the maps `fixtures/maps/e08-{duel-36,xl-144}.h3m` (`h3-map`'s `tests/fixtures.rs`; T31: heroes near a level; T42: flyers, water walkers, the skills), four gated workloads in `benches/adventure/` (play, reach by walkers and by lifted heroes, fork with a battle open).
- **Experiments**: `h3-experiment` (battle's) has the adventure shape; batches 1 to E6 scored (`e4_score`); agents run them on the harness (decision 21).
- **Army management** (T50): `army.rs` (`ArmyCommand`, `split_range`), `state/manage.rs` (the group passed in); `max_stack_count` (ADV-ARM-007); `AdventureOutcome::{meeting, no_room}` for T51.
- **Adventure ruleset part** (v11 since T50: a stack's most): terrains, roads, rivers, movement and setup parameters, object types, starting resources by difficulty, battle terrains, monsters' parameters (with the price resource).
- **Properties** on generated playable maps (`h3-map`'s `play.rs`, `view.rs`): `Step::Fight` (hero casts via `h3-battle`, a dev-dependency), monsters, grounds, the system actor, piles, level-ups, `check_progression` per command; 192 fixed battle cases; T42: kits, flyers, water walkers, Nomads.
- **Next:** merge #455 (`shared/e08-t50-cap`), then #457 (`adventure/e08-t50`); T51 (`E08-design.md` §8–§11; it drops the join's no-room skip).
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ownership:** the four crates, `h3-heroes`, `h3-sim`'s `src/adventure/replay.rs`, `h3-cli`'s `src/adventure/` (`h3-cli`'s `Cargo.toml`, `lib.rs`, `main.rs` battle's, `h3-sim`'s the AI lane's: `cross-lane`), `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, `benches/adventure/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,heroes,town}.toml`.
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- Stage archives, not files: the user's `original-data/maps.tar` (the shipped maps; untar into `original-data/install`) and `extracted.tar` (the text tables); the sprite archives for `.msk` sizes. `sandbox-setup.sh --lane adventure` copies only the engine's files (copy the harness's DLLs, sprite LODs and sound archives to `original-data/install`) and skips the client gate's SDL packages.
- A shallow clone: `git fetch --depth=60 origin master` before `sandbox-setup.sh`, or its baseline runs in full.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-24 — increment 6, round 4: T50

Baseline f89aa89. T50 whole (decision 113), the cap first (#455); the no-room offer left to T51; four duplicate decision-112 blocks removed from the epic.

### 2026-10-24 — increment 6, round 3: T49b; round 3 closed

Baseline d0d7bb8. T49b whole plus the import's Titan's book (decision 112), import v10; full verify.

### 2026-10-24 — increment 6, round 3: T49a

Baseline a5fbe80. T49a whole: positions, hash v12, import v9, two lanes' call sites; full verify.
