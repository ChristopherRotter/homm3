# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md` (refined 2026-10-08, reviewed with the user the same day; decisions 1–12 there). Increment 1 = T1a, T1b, T2–T10, a checkpoint with the user at T1a (ADR review) and at T10.

### E08 T1a — ADR drafts

- **Scope:** E08 T1a. Three ADR drafts in `docs/lanes/adventure/adr-drafts/`, decisions only (about 80 lines each, `docs/adr/README.md`), written from the epic's decisions 2–9 as they stand: (1) crate shape (the four crates `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game`; the `application` layer; the ADR-0005 amendment; the later content split; the setup types and the import boundary, decision 4); (2) adventure state (the persistent grid with what sees through its `Arc`s, decision 6; map objects as closed behaviour kinds with data parameters, ADR-0012 style, increment 1 uses only "blocker"; the fog projection, decision 7); (3) game orchestration (turn groups, per-player RNG streams, pending decisions, decisions 8–9; where `AdventureEnv` lives: adventure-owned sub-paths of `h3-sim` and `h3-cli`).
- **Acceptance:** the user reviewed the three drafts; open points are `ADV-Q<n>` entries.
- **Depends on:** nothing.
- **Status:** waiting (the user's review of the drafts, PR #32; open points `ADV-Q1`, `ADV-Q2`).

### E08 T1b — Survey and quirks

- **Scope:** E08 T1b. Sources and quirks for movement (movement points by army speed, terrain and road costs, diagonal cost, the last step of a day, leftover points), fog of war (scouting radius, the fringe, decision 7: confirmed per Q30 before it becomes `ADV-QUIRK-1`, by the user looking at a map) and the day advance; HD's and HotA's contact rules for simultaneous turns (decision 8, captured for a later decision). Output: `docs/rules/surveys/E08-adventure-core.md`, `docs/rules/quirks-adventure.md`, rule ids `ADV-*`/`GAME-*` proposed in the survey. The wiki needs the user's approval per session (`docs/HANDOFF.md`, "Sources").
- **Acceptance:** the user classified the quirks; disputed rules are `ADV-Q<n>` entries.
- **Depends on:** nothing (runs beside T2; gates T4, T7 and T8).
- **Status:** ready.

### E08 T2 — Shared skeleton

- **Scope:** a `shared/adventure-skeleton` pull request: crate skeletons `h3-map` (adapter), `h3-pack-adventure` (adapter), `h3-adventure` (domain), `h3-game` (application) with the logic headers; `arch.toml` edges; `lanes.toml` (the new crates, `packs/*/adventure.toml`, `docs/rules/catalog/game.toml`, `docs/formats/h3m.md` to adventure with the user's consent, the adventure sub-paths `crates/h3-sim/src/adventure/`, `crates/h3-cli/src/adventure/`, `fixtures/replays/adventure/` by the longest-match rule); `xtask` (the determinism step for adventure replays under `--scope adventure`; the fuzz harness, `cargo-fuzz` with its nightly toolchain question, here or as a separate shared task before T3); the versions table in `docs/08-work-process.md` (adventure state hash, game state hash, adventure replay format and encodings); `BACKLOG.md` (E07 folded into E08, E08 in progress, E09 and E10 restated, a row for the crate split after E04b); the global HANDOFF's lane row.
- **Acceptance:** full `cargo xtask verify` green in CI; merged before T3.
- **Depends on:** T1a accepted; `ADV-Q1` (the ids in `h3-contracts`).
- **Status:** waiting (T1a).

### E08 T3 — Map reader

- **Scope:** E08 T3 in `h3-map`, probably T3a (header, players, conditions, heroes, terrain) and T3b (templates, every object body of RoE/AB/SoD). Complete raw model with input bounds (map size, counts, string lengths; `docs/06-security.md`), the import of terrain, roads, rivers, footprints and placed heroes into the setup types; fuzzing; fixtures (our writer's maps, the user's editor maps committed, shipped maps in the local `--with-original-data` gate). Reference: VCMI 1.7.5's reader and `docs/formats/h3m.md` (adventure-owned after T2: the reader's notes go into the same file).
- **Acceptance:** the Definition of Done; every shipped map of the user's installation parses (local gate); the bounds have rejection tests.
- **Depends on:** T2 (and the fuzz harness).
- **Status:** waiting (T2).

### E08 T4–T10

- **Scope:** as in the epic file: T4 adventure content, T5 state and the persistent grid (with the two copy-on-write properties), T6 `h3-game` thin, T7 movement, T8 fog and projection (with the viewer knowledge list), T9 env/CLI/gates (needs `ADV-Q2`: how a replay names its map), T10 closure. Refined into queue entries after T1a's drafts are accepted.
- **Status:** waiting (T1a).

## Inbox

- 2026-10-09 (T1a): when the user accepts the drafts, a `shared/` change numbers them (`docs/adr/README.md`), amends ADR-0005 (the four crates, `h3-game` the first application crate and a logic crate) and ADR-0006 (ruleset parts, one pack file per context), updates `docs/02-architecture.md` §3 and `docs/03-domain.md` (the Game row, the answered object question, the terms: turn group, pending decision, static map, explored, fringe, object kind, blocked and visitable tile). It can ride on T2's pull request.
- 2026-10-08 (refinement): the crate split after E04b (E08 decisions 2–3) is a `shared/` + `cross-lane` task run with the battle lane paused; T2 adds it to `BACKLOG.md`.
- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
