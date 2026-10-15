# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1 and 2 **closed** (2026-10-12, -13). **Increment 3** (monsters and battles, decisions 19–38): rounds 1 to 3 (T17a to T19b2) **done**; round 4: T20a1, T20a2a, T20a2b, T20b1, T20b2 **done** (decisions 35–37), T20b3 split (decision 38): **T20b3a done** (#234: the neutral seat), T20b3b next (the maps, replays, plans and benchmarks with monsters). Design and cut: `E08-design.md`. Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 6, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; 4 maps import whole, the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps (with the monsters' and grounds' census).
- **Monsters and grounds** (T20a2a): kinds `Monster`, `Ground`; `[params.monsters]`; `ObjectSetup.monster`/`.drawn` (ADV-MON-002 to -004); `army_left`; `battle_ground` (ADV-GRD-001); `ObjectView.monster`. T20a2b: `guard_of` (ADV-MON-005, -006). T20b1: `answer.rs` (ADV-MON-007), offers `Join`, `JoinForGold`, `Pursue` (ADV-MON-008), `guard_after`, `visit_guard`, `settle_outcome`. T20b2: `split`, `TileRand` (ADV-MON-009, flag 24), `Defender::Monster`, `Actor::System`, `end_monster_battle` (ADV-MON-010). T20b3a: no guard after a neighbour visit (0x4800B8).
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId`, from `GameHeroSetup.details` or the defaults, validated by `h3-heroes`; `attack_defense`, `join_bonus` (T20a1); `GameObservation.heroes`.
- **Objects** (T12–T15): types (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removals (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at`, `tile_object`, `PastVisit`, `FromAbove`); gains and offers in `AdventureOutcome` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001 to -004); offers `Reveal { price }`, `Pass`, `offer_stands`.
- **Battles** (T19a1, GAM-BAT-001 to -005; `h3_game::state::battle`): a step toward another player's hero visits it (`BattleStart`, `battle_ground`), `Decision::Battle(Arc<OpenBattle>)` both owners play (`GameCommand::Battle`), seeds under `ADV-QUIRK-5`, events and `GameObservation.battle` to the owners; the outcome's armies, `HeroRemoved`, a surrender's gold; the heroes' part (`heroes_after_battle`: `HeroChange`s, Necromancy, Eagle Eye).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc` with the coast bits, heroes by id with their movement points and formation, the difficulty, state hash v9; ADV-MAP-001, ADV-MOV-001/-002); `h3_game::{GameRuleset, GameSetup, GameState}`: the starting armies (GAM-SET-001), a stream per player, sequential turn groups, `apply` (GAM-TRN-001/002, GAM-CAL-001), treasuries (T13: GAM-RES-001/-002, `GameSetup.difficulty`), pending decisions (T14b: GAM-DEC-001, `Answer`, `OfferGone`). Game state hash v6; `GameRuleset` with the battle part. `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006, `ADV-QUIRK-2`/`-3`), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `reach` (`Reach`, one path per tile as the original's).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands** (T8b2): `h3_game::legal_commands` and `legal_moves` (a hero's `Reach`, the user), empty outside the turn.
- **The env** (T9a, `h3-sim/src/adventure/`): `AdventureEnv` (reset, `to_move`, masks, steps, fork, hash, the view, a day limit), action encoding v3 since T19b1 (`Phase`; `MapAction`: `EndTurn`, eight hero slots, `Accept`, `Decline`, a tile; in a battle the battle's encoding; `to_move` yields `Seat`s: players, each battle's owner of the side to act, and since T20b3a `Seat::Neutral` for a battle's unowned side, played as `Actor::System` with its battle masks and `neutral_observation`), `random_rollout` (every seat).
- **Replays and benchmarks** (T9b): `adventure::replay` (format v4 since T20b2), `h3 adventure run/verify/bench`, eight golden replays, two with hero battles (re-record with the options in `crates/h3-cli/tests/adventure_replays.rs`), the maps `fixtures/maps/e08-{duel-36,xl-144}.h3m` (`h3-map`'s `tests/fixtures.rs`; no monsters until T20b3b), three gated workloads in `benches/adventure/` (play, reach, fork with a battle open).
- **Experiments**: `h3-experiment` (battle-owned) has the adventure shape; batches 1 and 2 scored. **From increment 3 agents run batches** on the client lane's harness (decision 21; `E08-battles.md` §13).
- **Adventure ruleset part** (v8 since T20b1): terrains, roads, rivers, the movement and setup parameters, the object types, the starting resources by difficulty, the battle terrains, the monsters' parameters (with the price resource).
- **Properties** on generated playable maps (`h3-map`'s `play.rs`, `view.rs`; T8b2); since T19b2 `Step::Fight` runs (seek, fight with hero casts through `h3-battle`, a dev-dependency), heroes with Eagle Eye of their own; `the_plans_play_battles_to_a_victory` counts 192 fixed cases.
- **Next:** #234 (T20b3a) to merge, then T20b3b (QUEUE).
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), their increment-2 amendments (#115), increment 3's (ADR-0013 too) applied by T18a (#185).
- **Fuzzing is stable-only** (user, 2026-10-10): property tests on arbitrary bytes.
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ownership:** the four crates, `src/adventure/` in `h3-sim` and `h3-cli` (their `Cargo.toml`, `lib.rs` and `h3-cli`'s `main.rs` are battle's: `cross-lane`), `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, `benches/adventure/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`. `h3-heroes` moves to this lane as soon as the battle lane is done with it (decision 25; its inbox).
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fog fringe; 2, 3 the diagonal exception; 4 leading neutral stacks; 5, 6 the tile's obstacles and split; flags 19–22 in `h3_content`'s registry, 22 the fringe's), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first.

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- Stage archives, not files (~13 k tokens per 50): the user's `original-data/maps.tar` (the shipped maps; untar into `original-data/install`) and `extracted.tar` (the text tables, since 2026-10-15); the sprite archives only for `.msk` sizes.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-15 — round 4, fourth session: T20b3a

#228, #230, #231 merged; baseline light (ae6d517 verified). Decision 38: T20b3 in two, T20b3a alone. The neutral seat (`Seat`, `ADV-STUB-7` closed); the reading found no guard after a neighbour visit, as built (a test, docs corrected). The eight replays byte-identical.

### 2026-10-15 — round 4, third session: T20b1 and T20b2

#220, #223 merged; baseline light (615286e verified). Decision 37: T20b in three; grounds' exemption confirmed. T20b1: the answer and its offers, part v8, game state hash v7. T20b2: flag 24 (`shared/`), the split, `Actor::System`, the outcome; replay format v4. Sandbox: the client check needs SDL's packages (`clients.yml`'s apt line).

### 2026-10-15 — round 4, second session: T20a2a (and T20a2b)

#215 merged; baseline green (0132a15). Decision 36. T20a2a: monsters and grounds as data, part v7, import v6, state hash v9; the grounds' draw order read (0x505230); grounds left out of `SharedAnchor`. T20a2b: guarded tiles, the store passed on.
