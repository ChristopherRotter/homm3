# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1, 2 and 3 **closed** (2026-10-12, -13, -20; increment 3: decisions 19–47). **Increment 4: hero progression only** (decisions 48–62; survey `E08-progression.md`, design `E08-design.md`, cut §17): rounds 1 and 2 done; round 3 (decision 63): T29 done, T30 next (its cut first). Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 8, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; of the 160 shipped maps 5 import whole (their 9 heroes get 41 levels at load), the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps (with the monsters', grounds' and guarded piles' census).
- **Monsters, grounds, guarded piles** (T20a2a): kinds `Monster`, `Ground`; `[params.monsters]`; `ObjectSetup.monster`/`.drawn` (ADV-MON-002 to -004); `army_left`; `battle_ground` (ADV-GRD-001); `ObjectView.monster`. T20a2b: `guard_of` (ADV-MON-005, -006). T20b1: `answer.rs` (ADV-MON-007), offers `Join`, `JoinForGold`, `Pursue` (ADV-MON-008). T20b2: `split` (ADV-MON-009, flag 24), `Defender::Monster`, `Actor::System`, `end_monster_battle` (ADV-MON-010). T20b3b1: `MonsterChanged`; fringe monsters in strict (decision 40). T21a (ADV-PIK-004, `state/piles.rs`): `Guards`, `guards_left`, `OfferKind::Fight`, `Defender::Guards`, `end_unowned_battle`.
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId`, from `GameHeroSetup.details` or the defaults, validated by `h3-heroes`; `attack_defense`, `join_bonus` (T20a1); `GameObservation.heroes`. Progression (T26b data, T27 rules): `h3_heroes`' `level.rs`, `level_up.rs` (`draw_level_up` → `LevelUp`, `OriginalDraws`), `GainLevel`, `apply_change_limited`, `HeroDetails.offered`; a level above the experience's is invalid (decision 62). T28: `original_choice` (HER-LVL-006). T29: `state/levels.rs`' `Progression` (the limit, the bans, the level seeds; GAM-HER-002, flag 27), the levels at load (GAM-SET-002), `StaticMap::terrain_shares` (ADV-MAP-002).
- **Objects** (T12–T15): types (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removals (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at`, `tile_object`); gains and offers in `AdventureOutcome` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001 to -004); `offer_stands`.
- **Battles** (T19a1, GAM-BAT-001 to -005; `h3_game::state::battle`): a step toward another player's hero visits it (`BattleStart`, `battle_ground`), `Decision::Battle(Arc<OpenBattle>)` both owners play (`GameCommand::Battle`), seeds under `ADV-QUIRK-5`, events and `GameObservation.battle` to the owners; the outcome's armies, `HeroRemoved`, a surrender's gold; the heroes' part (`heroes_after_battle`).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc` with the coast bits, heroes by id with their movement points and formation, the difficulty, state hash v10; ADV-MAP-001, ADV-MOV-001/-002); `h3_game::{GameRuleset, GameSetup, GameState}`: the starting armies (GAM-SET-001), a stream per player, sequential turn groups, `apply` (GAM-TRN-001/002, GAM-CAL-001), treasuries (T13: GAM-RES-001/-002, `GameSetup.difficulty`), pending decisions (T14b: GAM-DEC-001, `Answer`, `OfferGone`). Game state hash v10; `GameRuleset` with the battle part. `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006, `ADV-QUIRK-2`/`-3`), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `reach` (`Reach`, one path per tile as the original's).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands** (T8b2): `h3_game::legal_commands` and `legal_moves` (a hero's `Reach`, the user), empty outside the turn.
- **The env** (T9a, `h3-sim/src/adventure/`): `AdventureEnv` (reset, `to_move`, masks, steps, fork, hash, the view, a day limit), action encoding v3 since T19b1 (`Phase`; `MapAction`: `EndTurn`, eight hero slots, `Accept`, `Decline`, a tile; in a battle the battle's encoding; `to_move` yields `Seat`s, since T20b3a `Seat::Neutral` for an unowned side, played as `Actor::System` with `neutral_observation`), `random_rollout` (every seat).
- **Replays and benchmarks** (T9b): `adventure::replay` (format v4 since T20b2), `h3 adventure run/verify/bench`, nine golden replays (since T20b3b2: monsters, guarded piles, hero battles; re-record with the options in `crates/h3-cli/tests/adventure_replays.rs`), the maps `fixtures/maps/e08-{duel-36,xl-144}.h3m` (`h3-map`'s `tests/fixtures.rs`; monsters, guarded piles, grounds since T20b3b2), three gated workloads in `benches/adventure/` (play, reach, fork with a battle open).
- **Experiments**: `h3-experiment` (battle's) has the adventure shape (batch E3's since T22a1); batches 1, 2 and E3 scored. **Agents run batches** on the client lane's harness (decision 21; E17 H1, H2).
- **Adventure ruleset part** (v9 since T29: the costs by Pathfinding level): terrains, roads, rivers, the movement and setup parameters, the object types, the starting resources by difficulty, the battle terrains, the monsters' parameters (with the price resource).
- **Properties** on generated playable maps (`h3-map`'s `play.rs`, `view.rs`): `Step::Fight` runs (hero casts through `h3-battle`, a dev-dependency), monsters, grounds, the system actor, guarded piles; `the_plans_play_battles_to_a_victory` counts 192 fixed cases.
- **Next:** T29's pull request (`adventure/e08-t29`, `cross-lane`) to merge; then T30's cut (design §13).
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), their increment-2 amendments (#115), increment 3's (ADR-0013 too) applied by T18a (#185).
- **Fuzzing is stable-only** (user, 2026-10-10).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ownership:** the four crates and `h3-heroes` (since E04c), `src/adventure/` in `h3-sim` and `h3-cli` (their `Cargo.toml`, `lib.rs` and `h3-cli`'s `main.rs` are battle's: `cross-lane`), `crates/h3-cli/tests/adventure_replays.rs`, `fixtures/replays/adventure/`, `fixtures/maps/`, `benches/adventure/`, each pack's `adventure.toml`, `docs/formats/h3m.md`, `docs/rules/catalog/{adventure,game,town}.toml`.
- **Ids:** questions `ADV-Q<n>`, quirks `ADV-QUIRK-<n>` (1 the fog fringe; 2, 3 the diagonal exception; 4 leading neutral stacks; 5, 6 the tile's obstacles and split; 7 A-6; 8 the map's formation; flags 19–24), stubs `ADV-STUB-<n>`, rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first.

## Standing traps

- Cheap forks are a hard requirement (decision 6): no per-fork deep copy of map-sized data; pointer identity never in logic or the hash.
- RNG seeds and stream positions are server-only state (decision 9): never in an observation, never seeded from client data.
- Template masks: bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above the anchor (`h3m.md` had it mirrored until T5a); check a geometric reading of the format against the shipped maps (towns' gates, heroes' tiles) before building on it.
- Stage archives, not files (~13 k tokens per 50): the user's `original-data/maps.tar` (the shipped maps; untar into `original-data/install`) and `extracted.tar` (the text tables, since 2026-10-15); the sprite archives only for `.msk` sizes.
- A shallow clone: `git fetch --depth=60 origin master` before `sandbox-setup.sh`, or its baseline runs in full.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-21 — increment 4, twelfth session: T29

Baseline 528b602. Decision 63 (`cross-lane`: the AI lane's census). The seeds drawn last (the replays' commands unchanged); 41 levels at load.

### 2026-10-21 — increment 4, eleventh session: T28, round 2 closed

Baseline 79551a5. The four functions re-read; `original_choice` (the shares through `X87`); E3-6 reproduced; HER-LVL-006, `ADV-QUIRK-14`.

### 2026-10-20 — increment 4, tenth session: T27

Baseline b989682. Decision 62: part A §2–§7, the other lanes' heroes at their level's experience (`cross-lane`), game state hash v9.
