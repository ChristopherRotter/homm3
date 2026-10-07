# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order (`docs/08-work-process.md`, "Lanes"). A runner hands the first ready task to a fresh worker and keeps only the worker's short report; a worker reads AGENTS.md, the global and lane HANDOFF, the lane's reading list and its one task. A task is about half a context of work and has:

- **Scope:** what to build or decide, with pointers (epic task, ADR, rule ids).
- **Acceptance:** what must hold when it is done (always: the Definition of Done in `docs/05-testing.md`).
- **Depends on:** tasks, questions or user input that must come first.
- **Status:** `ready`, `waiting` (for the user, with what), `blocked` (with the question id; the runner takes the next independent task) or `in progress` (branch).

Done tasks leave the queue (the epic file and its log keep the record). **Inbox:** discovered work, one line each, until a refinement turns it into tasks or a backlog item.

## Tasks

### Adventure refinement (E07/E08)

- **Scope:** the refinement session with the user; the brief is in `HANDOFF.md` ("Brief for the refinement session"). Output: refined epic files, ADR drafts (`docs/lanes/adventure/adr-drafts/`), questions (`ADV-Q<n>`), and the first tasks of this queue.
- **Acceptance:** the user agreed the first slice and its seams; this queue holds pre-refined tasks for it.
- **Depends on:** Phase 0 merged.
- **Status:** ready (a session with the user, not an unattended worker).

## Inbox

- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).
- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator): keep it in the formats adapter so both can extend it.
- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).
- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7).
- 2026-10-01 (E04 refinement): hero progression (experience table, level-up algorithm with the class probabilities, skill offers) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10).
- 2026-10-04 (E04a T9): applying a battle outcome (E08–E10) follows batch 1b (`experiments/results/E04a-batch-1b.md`): killed war machines give no experience; Learning multiplies the whole sum including the +500 for a defeated hero (R5: floor(550 × 1.15) = 632; D-3, OQ-6); the loser's surviving war machines are not captured (OQ-5); Necromancy rounds down per killed stack (R6: 6, 6, 6, 14, 17 at 10 % → 2 Skeletons; D-7), so it needs `BattleOutcome.stacks`. Also check whether a player without towns may retreat or surrender (not researched).
