# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md` (refined 2026-10-08, reviewed with the user the same day; decisions 1–12 there; done tasks in `E08-log.md`). Increment 1 = T1a, T1b, T2–T10, a checkpoint with the user at T1a (done: ADR-0015–0017), after T3b and at T10. T1a, T1b, T2 and T3a are done; the import moved from T3 to T5 and T6 (user, 2026-10-10).

### E08 T3b — Map reader: templates, objects, events

- **Scope:** E08 T3b in `h3-map`: the object templates (`docs/formats/h3m.md` has the writer's layout), every object body of RoE/AB/SoD (about 30 body kinds over the classes; VCMI 1.7.5's `CMapLoaderH3M::readObject` and the readers it calls, skipping HotA's branches), the timed events, then reject trailing bytes and remove ADV-STUB-1. Extend the raw model, the test encoder and strategies (`src/tests/`), the arbitrary-bytes properties and the bounds (object and template counts, message lengths: set from the shipped maps with `examples/original_data.rs`'s peaks). Round trip with the experiment writer's whole maps (`h3_formats::h3m`, dev-dependency). Fixtures in `fixtures/maps/`: the user's editor maps when they arrive (one per version, with a customised hero, many object kinds, a special condition, a rumor, a timed event, one with an underground) and small maps of our writer.
- **Acceptance:** the Definition of Done; `cargo xtask verify --with-original-data` reads every shipped map to its end; the new bounds have rejection tests; `docs/formats/h3m.md` covers the object bodies.
- **Depends on:** T3a (`adventure/e08-t3a`).
- **Status:** ready once T3a merges; then the user's checkpoint.

### E08 T4–T10

- **Scope:** as in the epic file: T4 adventure content, T5 state and the persistent grid (with the two copy-on-write properties; the persistent module is `src/persistent.rs`, the only place `arch.toml` allows `ptr_eq`/`as_ptr`; `AdventureSetup` and the import of terrain and footprints), T6 `h3-game` thin (`GameSetup`, the import of players and placed heroes, `Unsupported`, `MAP_IMPORT_VERSION` documented; parse, import and validation property-tested together on arbitrary bytes), T7 movement (starts with CR-1, `ADV-Q3`, then the user classifies A-2 and A-3), T8 fog and projection (starts with CR-3, `ADV-Q5`; A-1 needs the user's look at a map), T9 env/CLI/gates (a replay names its map by content hash, path and `MAP_IMPORT_VERSION`, ADV-Q2), T10 closure. To be refined into queue entries (ADR-0015–0017 accepted).
- **Status:** waiting (refinement into entries, before T4).

## Inbox

- 2026-10-10 (T2): the game context's rule ids use the existing prefix `GAM-` (`check-rules`, catalog file `docs/rules/catalog/game.toml`), not `GAME-` as the T1b survey proposed.
- 2026-10-10 (T2): already allowed by `arch.toml`, so T5–T9 need no shared change: `h3-sim` → `h3-adventure`, `h3-game` (dev: `h3-map`, `h3-pack-adventure`); `h3-cli` → the four crates; `h3-map` dev → `h3-pack`, `h3-pack-adventure`. Golden replays go to `fixtures/replays/adventure/` with the test `crates/h3-cli/tests/adventure_replays.rs` (both adventure-owned): `cargo xtask determinism` runs it once it exists, and `verify --scope adventure` runs the determinism step once the folder has files (T9).
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
