# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. Increment 1 (the reading's map, the oracle, the survey): **T1a1, T1a2 and T1b1 done** (survey `docs/rules/surveys/E14-generator.md` §1, §2; the oracle `ORACLE.md`: `clients/capture/h3rmg.py`, a corpus of 21 cases; log `docs/epics/E14-log.md`). Increments 2 to 6 sketched. No engine code yet.
- **Next:** **T1b2** (the phase oracle: the draw log per site, the step dumps; T1b cut by the user), meant for the same chat as T1b1, whose sandbox holds the client lane's staged set (a new chat stages it from the PC again); then **T1c** (survey rules, quirk candidates classified with the user, the ADR draft, the cut of increments 2 to 6, the checkpoint RMG-Q2).
- **Open pull requests:** `rmg/e14-t1b1` (T1b1). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 1; ledger `docs/rules/quirks-rmg.md`, made with the first; candidates in the survey's §1 and §2, unclassified until T1c), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23).

## Recent sessions

### 2026-11-23 — T1b1 the oracle (T1b cut by the user)

Staged the client lane's set (the PC's shell worked). `xdd.dll` fixes `time()`'s date (`H3TIME`), `h3rmg.py` sets the Random Map tab and keeps the map: 21 cases, each the same from fresh starts, in a second generation in one game and after another history (no memory-dependent variation under Wine); SoD's `rmg.txt` is read (the templates predicted 21 of 21); the plain writer as read; `h3_map` reads all, one refused at import (TWN-SET-001, for T1c). The VM restarted once (the baseline lost, CI's run on `master` stands for it).

### 2026-11-23 — T1a1 the frame; T1a2 in five parts by workers

Staged `Heroes3.exe` and the small files (the PC's shell worked), and two maps the original generated, found in Complete's `random_maps` (the description names the template and the seed). T1a1 read the frame (survey §1). The user cut T1a2 into five parts of about 22 kB, run by five worker sub-agents one after the other (a common brief, about 370 000 tokens each); the range closed (254 of 255 starts with an entry), the steps named, §2.6 the stream. Found: two memory-dependent draws, an HD-mod-patched `Untitled.h3m`, quirk candidates for T1c.

### 2026-11-21 — E14 refined, the lane opened

The user asked to refine the port of the original's generator as the prerequisite of a new custom one. A first look at the executable (the generator's range, entry, seed and size: the epic's "What the first look found"), two rounds of questions (RMG-Q1): same map per seed, a new lane, the engine's game as the end, a quirk profile now, E30 split off, a checkpoint per increment. Written: the epic, this lane, the `shared/` changes (lanes, BACKLOG, vision, AGENTS, README). Next: T1a1.
