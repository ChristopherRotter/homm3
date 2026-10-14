# Adventure lane — HANDOFF

The adventure lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E08**: increments 1 and 2 **closed** (2026-10-12, -13). **Increment 3** (monsters and battles, decisions 19–35): rounds 1 to 3 (T17a to T19b2) **done**, T19b2 2026-10-14 (`shared/e08-t19b2` first); round 4: **T20a1 done** (decision 35, `cross-lane`), T20a2 next. Design and cut: `E08-design.md`. Records: `E08-log.md`.
- **Map reader and import** (`h3-map`): `read`/`parse` into `model::RawMap`; `import` into `h3_game::GameSetup`, `import_map` its static layer (`MAP_IMPORT_VERSION` 5, `docs/formats/h3m.md` "Import"). Every shipped map's static layer imports and validates; 5 maps import whole, the rest are typed `Unsupported`. `verify --with-original-data` runs `crates/h3-map/examples/original_data.rs` on the shipped maps.
- **The store of heroes** (T18b, GAM-HER-001): `GameState` keeps `Arc<HeroDetails>` by `HeroId` (`hero`, `heroes`), from `GameHeroSetup.details` (`HeroDetailsSetup`) or the defaults (40–90 experience from the setup stream, the class's and type's values, the rolled war machines' artifacts), spell points full (`h3_heroes::max_mana`, HER-MAN-001), validated by `h3-heroes`; for the monsters' answer (T20a1) `attack_defense`, `join_bonus` (Diplomacy's in the packs); `GameObservation.heroes` (`HeroInfo`: the type of each hero seen, the owner's details).
- **Objects** (T12–T15): types in the ruleset part (`ruleset/objects.rs`, ADV-OBJ-002); contents drawn at the build (ADV-PIK-001 to -003); removed objects (`state/occupancy.rs`); `ObjectView`s (no index). Visits (ADV-OBJ-003, -004; `visit_at(part, player, pos)`, `tile_object`, `PastVisit`, `FromAbove`); gains and the offer in `AdventureOutcome`, `ObjectRemoved` (ADV-OBJ-005). Purchases and keys (`state/flags.rs`; ADV-VIS-001, -003); the observatory (-002); offers `Reveal { price }`, `Pass` (-004), `offer_stands`.
- **Battles** (T19a1, GAM-BAT-001 to -005; `h3_game::state::battle`): a step toward another player's hero visits it (`BattleStart`, `AdventureState::battle_ground`), `Decision::Battle(Arc<OpenBattle>)` both owners play (`GameCommand::Battle`), seeds under `ADV-QUIRK-5` (flag 23), events and `GameObservation.battle` to the owners; the outcome's armies, `HeroRemoved`, a surrender's gold; the heroes' part (`heroes_after_battle`: `HeroChange`s, `HeroChanged`; Necromancy, `CreaturesRaised`; Eagle Eye's `LearnSpell`); `Sink::RECORDS` passed on (T19b1).
- **Setups and state:** `h3_adventure::{AdventureSetup, AdventureState}` (static map behind an `Arc` with the coast bits, heroes by id with their movement points and formation, state hash v8; ADV-MAP-001, ADV-MOV-001/-002); `h3_game::{GameRuleset, GameSetup, GameState}`: the starting armies (GAM-SET-001), a stream per player, sequential turn groups, `apply` (GAM-TRN-001/002, GAM-CAL-001), treasuries (T13: GAM-RES-001/-002, `GameSetup.difficulty`), pending decisions (T14b: GAM-DEC-001, `Answer`, `OfferGone`). Game state hash v6; `GameRuleset` with the battle part. `GameSetup::validate` builds with seed 0.
- **Movement** (T7b): `Pricing` in `movement.rs` (ADV-MOV-003 to -006: the source tile prices a step, the diagonal exception under `ADV-QUIRK-2`/`-3`, points below the cheapest next step lost), `native_terrain` (ADV-MOV-004, `ADV-QUIRK-4`), `AdventureState::apply` with `MoveHero` (ADV-MOV-007, rejected whole, `HeroStepped` per step), `AdventureState::reach` (`Reach`, one path per tile as the original's; the user keeps it).
- **Fog of war** (T8a, `BitGrid`, 32 × 32 leaves): an explored layer per player, `fog::reveal` (ADV-FOG-002, -003), moves and `Reach` on explored tiles only (ADV-MOV-008), events `{ kind, seen_by: Viewers }` (ADV-FOG-004); the fringe (ADV-FOG-005 under `ADV-QUIRK-1`), `observe` and `project` in both crates (ADV-FOG-006).
- **Commands** (T8b2): `h3_game::legal_commands` and `legal_moves` (a hero's `Reach`, the user), empty outside the turn.
- **The env** (T9a, `h3-sim/src/adventure/`): `AdventureEnv` (reset, `to_move`, masks, steps, fork, hash, the view as observation, a day limit), the action encoding v3 since T19b1 (`Phase`; on the map v2's `MapAction`: `EndTurn`, eight hero slots, `Accept`, `Decline`, a destination tile, another player's hero's included; in a battle the battle's encoding for the player's side, `battle_mask` and the head masks; `to_move` adds each battle's owner of the side to act), `random_rollout` (the battle's agent in battles), `reset_with_difficulty`.
- **Replays and benchmarks** (T9b): `adventure::replay` (format v3 since T19b1: each command's `Actor`, battle commands), `h3 adventure run/verify/bench` (`crates/h3-cli/src/adventure/`), eight golden replays in `fixtures/replays/adventure/`, two with hero battles (re-record with the options in `crates/h3-cli/tests/adventure_replays.rs`), the maps `fixtures/maps/e08-{duel-36,xl-144}.h3m` (`h3-map`'s `tests/fixtures.rs`), three gated workloads in `benches/adventure/` (play, reach, fork with a battle open: `--battle 1`; `02-architecture.md` §8).
- **Experiments**: `h3-experiment` (battle-owned) has the adventure shape (`docs/experiments/README.md`); batches 1 and 2 scored. **From increment 3 agents run batches** on the client lane's harness (decision 21; needs in its inbox: `E08-battles.md` §13).
- **Adventure ruleset part** (v6 since T19a1): terrains, roads, rivers, native cost, land table, base scouting radius, the setup bounds (`max_players` ≤ `MAX_PLAYERS` = 8), the starting stack chances, the object types, the starting resources by difficulty, the battle terrains.
- **Properties** on generated playable maps (`h3-map`'s `play.rs`, `view.rs`; T8b2); since T19b2 `Step::Fight` runs (seek, fight with hero casts through `h3-battle`, a dev-dependency), heroes with Eagle Eye of their own; `the_plans_play_battles_to_a_victory` counts 192 fixed cases.
- **Next:** #213 (`shared/e08-t19b2`, merges first), #214 (T19b2), then T20a1's pull request (on #214); T20a2 at the next checkpoint; a quirk candidate for the user (the inbox).
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
- Staging the shipped maps costs ~13 k tokens per 50 files: pack them into one archive in the user's `original-data/` (gitignored; `tar -cf original-data/maps.tar -C original-data/install Maps` through the device shell, it is there since 2026-10-11) and stage that one file.
- A `CARGO_TARGET_DIR` outside `target/` is not gitignored: keep extra target directories under `target/` (`target/cov`), or `git add -A` commits gigabytes.
- An experiment's map and template must come from the same compile (batch 1a's map predated the review's Q4c, T10c): after any change to a spec, compile again and copy the map to the user's folder in the same step. Since T16b a mismatch shows: the map's description starts with `Stamp <hex>.`, the template's top with its own (`docs/experiments/README.md`).
- A shared change the lane needs (Cargo members, `arch.toml`, `lanes.toml`, contracts, kernel, xtask) goes in a `shared/` pull request; one that must land with lane paths (a new crate) needs the user's consent and the label `cross-lane`. A `shared/` or cross-lane branch runs the full verify (its scoped `check-lane` fails by design).

## Recent sessions

### 2026-10-14 — round 3, second session: T19b2 (and T20a1)

#211 merged; baseline green (f67aac8). Decision 35 (T19b2 + T20a1; `shared/e08-t19b2`). T19b2: the plans fight to a victory, nothing to fix; T19's exit criteria audited. T20a1: `attack_defense`, `join_bonus`, Diplomacy's values.

### 2026-10-14 — round 3, first session: T19b1

Baseline green (837cef2). Decision 34 (T19b split). T19b1: encoding v3, replay format v3, eight golden replays, the fork with a battle open; random play seldom fights.

### 2026-10-14 — round 2, fourth session: T19a2b

Decision 33. `shared/e08-t19a2b`, then T19a2b: Necromancy, Eagle Eye's learning.
