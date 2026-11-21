# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): refined 2026-11-21 with the user (RMG-Q1; `docs/epics/E14-original-map-generator.md`, decisions 1 to 7). Increment 1 (the reading's map, the oracle, the survey) refined; increments 2 to 6 sketched. Nothing built yet.
- **Next:** **T1a1** (`QUEUE.md`): the generator's frame read from `Heroes3.exe`. Needs only `original-data/install/Heroes3.exe` staged (SHA-256 `aac39e8c…`) and `pip install capstone pefile`.
- **Open pull requests:** none of the lane's.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 1; ledger `docs/rules/quirks-rmg.md`, made with the first), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup").

## Recent sessions

### 2026-11-21 — E14 refined, the lane opened

The user asked to refine the port of the original's generator as the prerequisite of a new custom one. A first look at the executable (the generator's range, entry, seed and size: the epic's "What the first look found"), two rounds of questions (RMG-Q1): same map per seed, a new lane, the engine's game as the end, a quirk profile now, E30 split off, a checkpoint per increment. Written: the epic, this lane, the `shared/` changes (lanes, BACKLOG, vision, AGENTS, README). Next: T1a1.
