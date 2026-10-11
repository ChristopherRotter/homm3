# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md` (refined 2026-10-08, reviewed with the user the same day; decisions 1–12 there; done tasks in `E08-log.md`). Increment 1 = T1a, T1b, T2–T10, with a checkpoint with the user at T1a (done: ADR-0015–0017), after T3 (done, 2026-10-10) and at T10. T1a, T1b, T2, T3, T4, T5a, T5b, T6a, T6b, T7a, T7b, T8a and T8b1 are done; the import moved from T3 to T5 and T6 (user, 2026-10-10). Refined into the entries below at the T3 checkpoint (2026-10-10): the user takes one or two tasks per round, each round ends at a checkpoint (the round after T3: T4 alone; after T4: T5a and T6a; after T5a/T6a: T6b and T7a; after T6b/T7a: T7b and T5b; after T7b/T5b: T8a alone, T8 split into T8a and T8b; after T8a: T8b1 alone, T8b split into T8b1 and T8b2; after T8b1: T8b2a alone, T8b2 split into T8b2a, what a command may do, and T8b2b, what a viewer may learn). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T8b2a — Legal commands and the commands' property tests

- **Scope:** `legal_commands` in `h3-game` (the game's: `EndTurn` while the player acts, and `MoveHero` for each of its heroes to each tile of the reachable-tiles query, the path from `Reach`; nothing for a player whose turn is not running), sharing the turn check with `apply` (ADR-0017's consequence); a strategy for playable maps (generated, imported, heroes of several players) and a random-play driver in `h3-map`'s tests (`h3-map` depends on `h3-game`), with `MoveHero` and `EndTurn` among the commands; the properties of what a command may do: atomicity (a rejected command changes nothing and emits nothing), legality consistency (every legal command applies, a sample of illegal ones is rejected), ownership isolation (no command of one player moves another's hero or changes another's explored tiles), clone equivalence (a fork and its parent evolve identically), determinism (the same commands give the same states, events and hashes).
- **Acceptance:** the property tests pass on imported maps; `legal_commands` is property-tested against `apply`.
- **Depends on:** T8b1 (done). **Status:** in progress (`adventure/e08-t8b2a`, 2026-10-11T17:00Z).

### E08 T8b2b — The observation's property tests

- **Scope:** on T8b2a's playable maps and driver, the properties of what a viewer may learn: observation hygiene (a viewer's observation and projected events do not depend on what it cannot see: the seed, other players' armies and points, hidden tiles), the observation's consistency with the events (a viewer's hero list moves as the projected steps say).
- **Acceptance:** the property tests pass on imported maps.
- **Depends on:** T8b2a. **Status:** waiting (T8b2a).

### E08 T9 — Env, CLI, gates

- **Scope:** `AdventureEnv` (reset, step, mask, fork, hash; the action a destination among the reachable tiles), a random agent, the CLI's run, replay verify and bench, golden replays naming their map by content hash, path and `MAP_IMPORT_VERSION` (ADV-Q2), the benchmark workloads and their baselines (decision 11: random play, pathfinding, and the fork cost moved here from T5b), the CLI wiring in battle-owned `crates/h3-cli` files (`Cargo.toml`, the dispatch: a `cross-lane` change with the user's consent).
- **Acceptance:** the replays pass the determinism matrix; the +2 % gate has a baseline.
- **Depends on:** T8b2b; the editor maps as committed fixtures (Inbox; the user asked at the T8b2a start why an agent cannot make them: a generated map for the replays is offered for the checkpoint). **Status:** waiting (T8b2b).

### E08 T10 — Closure

- **Scope:** the movement experiments' spec shape (cross-lane, decision 12) and a batch (ADV-Q3's D-2/D-3 candidates, ADV-Q4's turn-start experiment, ADV-Q6's starting-army map), the docs, the refinement of increment 2, the checkpoint.
- **Depends on:** T9. **Status:** waiting (T9).

## Inbox

- 2026-10-11 (T8b1): the fringe is the eight tiles around an explored one (ADR-0016's "touching"); how the original draws its half-dark border (which hidden tiles get an edge sprite) was not read: a code reading or a look in SoD with a single explored diagonal, with T10's batch.
- 2026-10-11 (T8b1): VCMI shows an object whole once one of its covered tiles is visible (`CGameState::isVisibleFor`); the observation shows what objects cover tile by tile (increment 1's objects have no type). Decide with increment 2's object types what of a partly seen object shows.
- 2026-10-11 (T8b1): `observe` builds one `TileView` per tile on every call (41 472 on 144 × 144 with an underground): T9's benchmark measures it; a cached static part or planes per layer if it matters.
- 2026-10-11 (T8b1): of another player's hero the observation shows tile, owner and identity (ADV-FOG-006, the user): survey what the original's right-click view shows (army, approximate counts) with the battle seam (increment 3). The spectator viewer for the adventure when the server lane hosts adventure games.
- 2026-10-11 (T8a): a step's reveal writes one `set_run` per row (about 2 400 instructions at radius 5, T5b's measurement), and every step of a move reveals: T9's benchmark decides whether a per-leaf batch on `BitGrid` (one copy-on-write check per leaf) is worth its API.
- 2026-10-11 (T8a): the Scouting specialty multiplies the radius by 1 + 0.05 × level (0x4E4070, truncated), not "+1 every six levels" (survey §2): compare with the wiki and VCMI with the adventure profile (increment 3).
- 2026-10-11 (T8a): the original's reveal sets the bits of every player of the revealer's team (allies share the map; VCMI too): with teams (E10). The new day reveals again around everything a player owns (0x4C7C50): with the first rule that hides tiles (Cover of Darkness, towns).
- 2026-10-11 (T7b): the step's price leaves out Pathfinding, Nomads, flying, water walking, Favorable Winds and Cursed Ground (0x4B1740 has them): increment 3 with the adventure profile, Cursed Ground with objects.
- 2026-10-11 (T5b): `examples/grid_bench.rs` is the fork-cost measurement; T9's gated workload in `benches/adventure/` can run it (or the env's fork) under `bench-check`.
- 2026-10-10 (T6b review): building a game emits no events (the first `TurnStarted` goes to a `NoopSink`): T8/T9 decide whether a replay or a client's event stream needs a start event, or reads the start from `observe`. `GameState` keeps no hero types yet: the game's store of heroes (hero details) comes with `h3-heroes` in increment 3.
- 2026-10-10 (T6b): the original plays every human player before every computer player each day (0x4C6FE0; VCMI `TurnOrderProcessor` too). The engine has no human/computer distinction; decide with the server lane's seats or the turn-group work whether a schedule should order players so (GAM-TRN-001's note).
- 2026-10-10 (T6b): weeks of seven days and months of four (wiki Time; VCMI parameters) come with the first rule that reads them (week events, increment 4); GAM-CAL-001 counts days only.
- 2026-10-10 (T6b): the starting army's other callers (the tavern, 0x4C86A0 and 0x4D9EC0) give one creature of the first stack; for the towns epic. The war machine of a starting army (artifact 4 or 6, ADV-Q6) for increment 3 with the hero details.
- 2026-10-10 (T6b): `docs/formats/installation.md` §7.1 says the nine war machines are the heroes' *third* starting creature; the hero table has them second (battle-owned doc; tell the battle lane or fix it with the formats lane).

- 2026-10-10 (T5a/T6a): for E09 (towns): a hero placed on a town's blocked tile is the town's visiting hero (VCMI `CGameState::placeHeroesInTowns` moves it to its own town's entrance, an "H3M format quirk"; 3 shipped maps), and a hero generated at a main town (129 maps); both are `Unsupported` in the import until towns exist.
- 2026-10-10 (T5a/T6a): not checked yet, each for the increment that needs it: a hero type placed twice on one map, the most heroes a player may have on the map (8 in the original; with recruiting), a war machine in a hero's custom army (battle content; with the battle seam, increment 3).
- 2026-10-10 (T5a): `h3_formats::h3m`'s comment on `mask_from_text` says `Objects.txt` masks are "top left first"; they are anchor first (VCMI `readTxt`; `docs/formats/h3m.md` corrected). The writer's code is right; fix the comment when the writer moves into `h3-map` (battle-owned until then).

- 2026-10-10 (T4): check `packs/base/adventure.toml`'s land table against the installation's `movement.txt` under `--with-original-data` (an `examples/original_data.rs` in `h3-pack-adventure` needs `h3-formats` as a dev-dependency to read the LOD: a shared `arch.toml` change), or harvest the table when adventure data is harvested.
- 2026-10-10 (T3b): fixtures in `fixtures/maps/` once the user's editor maps arrive (one per version: a customised hero, many object kinds, a special condition, a rumor, a timed event, one underground): commit them with a test that reads each and checks a few fields, so CI reads real editor output (the shipped maps stay local).

- 2026-10-10 (T2): the game context's rule ids use the existing prefix `GAM-` (`check-rules`, catalog file `docs/rules/catalog/game.toml`), not `GAME-` as the T1b survey proposed.
- 2026-10-10 (T2): already allowed by `arch.toml`, so T5–T9 need no shared change: `h3-sim` → `h3-adventure`, `h3-game` (dev: `h3-map`, `h3-pack-adventure`); `h3-cli` → the four crates; `h3-map` dev → `h3-pack`, `h3-pack-adventure`. Golden replays go to `fixtures/replays/adventure/` with the test `crates/h3-cli/tests/adventure_replays.rs` (both adventure-owned): `cargo xtask determinism` runs it once it exists, and `verify --scope adventure` runs the determinism step once the folder has files (T9).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
