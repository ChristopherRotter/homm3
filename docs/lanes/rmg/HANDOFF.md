# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. Increment 1 (the reading's map, the oracle, the survey): **T1a1 done** (the frame; survey `docs/rules/surveys/E14-generator.md` §1, log `docs/epics/E14-log.md`). Increments 2 to 6 sketched. Nothing built yet.
- **Next:** **T1a2a to T1a2e** (`QUEUE.md`; the user's cut into five parts, run by worker sub-agents in the session of 2026-11-23), then T1b (the oracle; the client lane's staging set), T1c (the checkpoint RMG-Q2).
- **Open pull requests:** none yet (the branch `rmg/e14-t1a1` carries T1a1 and T1a2; the README's row and BACKLOG's status go in `shared/e14-start`).
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 1; ledger `docs/rules/quirks-rmg.md`, made with the first; candidates so far in the survey, unclassified), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user (2026-11-23).

## Recent sessions

### 2026-11-23 — T1a1, the frame; T1a2 cut in five

Staged `Heroes3.exe` and the small files (the PC's shell worked), and two maps the original generated, found in Complete's `random_maps` (the description names the template and the seed). Read the frame (survey §1): the dialog's Random draws, the run's codes, the seed and water's draw in the constructor, the files read, `rmg_generate_map`'s template draw, positions and 15 steps, `rmg_write_map`'s outline with the teams drawn at write time; 46 map entries, 12 records. The user cut T1a2 into five parts and asked for worker sub-agents in this session. **In progress:** T1a2a to T1a2e, one worker each, in order.

### 2026-11-21 — E14 refined, the lane opened

The user asked to refine the port of the original's generator as the prerequisite of a new custom one. A first look at the executable (the generator's range, entry, seed and size: the epic's "What the first look found"), two rounds of questions (RMG-Q1): same map per seed, a new lane, the engine's game as the end, a quirk profile now, E30 split off, a checkpoint per increment. Written: the epic, this lane, the `shared/` changes (lanes, BACKLOG, vision, AGENTS, README). Next: T1a1.
