# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md` (refined 2026-10-08; decisions 1–12 there). Increment 1 = T1–T10, a checkpoint with the user at T1 (ADR review) and at T10.

### E08 T1 — Survey and ADR draft

- **Scope:** E08 T1. Sources and quirks for movement (movement points by army speed, terrain and road costs, diagonal cost, the last step of a day, leftover points), fog of war (scouting radius, the fringe, decision 7) and the day advance; HD's and HotA's contact rules for simultaneous turns (decision 8). Output: `docs/rules/surveys/E08-adventure-core.md`, `docs/rules/quirks-adventure.md` (`ADV-QUIRK-1`: the fringe), rule ids `ADV-*`/`GAME-*` proposed in the survey, and an ADR draft in `docs/lanes/adventure/adr-drafts/` covering decisions 2–9: crate shape and the ADR-0005 amendment, the persistent state, map objects (closed behaviour kinds), turn groups and RNG streams, the fog projection, where `AdventureEnv` lives.
- **Acceptance:** the user reviewed the ADR draft and classified the quirks; open points are `ADV-Q<n>` entries.
- **Depends on:** nothing.
- **Status:** ready (ends at a checkpoint with the user).

### E08 T2 — Shared skeleton

- **Scope:** a `shared/adventure-skeleton` pull request: crate skeletons `h3-map` (adapter), `h3-content-adventure` (shared), `h3-pack-adventure` (adapter), `h3-adventure` (domain), `h3-game` (application) with the logic headers; `arch.toml` edges; `lanes.toml` (the new crates, `packs/*/adventure.toml`, `docs/rules/catalog/game.toml`); `BACKLOG.md` (E07/E08 in progress, a row for the crate split after E04b); the global HANDOFF's lane row.
- **Acceptance:** full `cargo xtask verify` green in CI; merged before T3.
- **Depends on:** T1 accepted.
- **Status:** waiting (T1).

### E08 T3 — Map reader

- **Scope:** E08 T3 in `h3-map`, probably T3a (header, players, conditions, heroes, terrain) and T3b (templates, every object body of RoE/AB/SoD). Complete raw model, the import of terrain, roads, rivers, footprints and placed heroes; fuzzing; fixtures (our writer's maps, the user's editor maps committed, shipped maps in the local `--with-original-data` gate). Reference: VCMI 1.7.5's reader, `docs/formats/h3m.md` (battle-owned: format notes for the reader go into an adventure-owned file or a `cross-lane` change).
- **Acceptance:** the Definition of Done; every shipped map of the user's installation parses (local gate).
- **Depends on:** T2.
- **Status:** waiting (T2).

### E08 T4–T10

- **Scope:** as in the epic file: T4 adventure content, T5 state and the persistent grid, T6 `h3-game` thin, T7 movement, T8 fog and projection, T9 env/CLI/gates, T10 closure. Refined into queue entries after T1's ADR is accepted.
- **Status:** waiting (T1).

## Inbox

- 2026-10-08 (refinement): the crate split after E04b (E08 decisions 2–3) is a `shared/` + `cross-lane` task run with the battle lane paused; T2 adds it to `BACKLOG.md`.
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator): keep it in the formats adapter so both can extend it.
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
