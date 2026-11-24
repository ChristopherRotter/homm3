# RMG lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`.

## Current state

- **E14** (the original's random map generator, ported to give the same map per seed): in progress since 2026-11-23. Increment 1 (the reading's map, the oracle, the survey): **T1a1, T1a2, T1b1 and T1b2 done** (survey `docs/rules/surveys/E14-generator.md` §1, §2; the oracle and the phase oracle `ORACLE.md`: `clients/capture/h3rmg.py`, 21 cases, `rmg/kept.txt` and `rmg/phases.txt`; log `docs/epics/E14-log.md`). Increments 2 to 6 sketched. No engine code yet.
- **Next:** **T1c** (survey rules; the painters' draws outside the range; quirk candidates classified with the user, the memory-dependent ones among them; the ADR draft; the cut of increments 2 to 6; the checkpoint RMG-Q2). Big: cut it at a seam with the user first. A new chat stages the client lane's set from the PC again only for oracle runs; reading needs `Heroes3.exe` alone.
- **Open pull requests:** `rmg/e14-t1b2` (T1b2). Watch it until it merges.
- **Owns** (`lanes.toml`): `crates/h3-rmg/` (created at increment 2, a shared change), the packs' `rmg.toml`, the RMG rules and quirks, E14's epic, surveys and experiments, this folder.
- **Ids:** questions `RMG-Q<n>` (next 2; none open), quirks `RMG-QUIRK-<n>` (next 1; ledger `docs/rules/quirks-rmg.md`, made with the first; candidates in the survey's §1 and §2 and ORACLE.md, unclassified until T1c), stubs `RMG-STUB-<n>` (next 1), decisions next 8.
- **Working agreement:** a checkpoint with the user per increment (decision 6); tokens first; read the executable in the cloud sandbox (`docs/exe/README.md`, "Setup"); a big task cut at a seam with the user, worker sub-agents allowed for its parts (2026-11-23).

## Recent sessions

### 2026-11-23/24 — T1b2 the phase oracle

A new chat: the client lane's set staged from the PC again (60 files). `xdd.dll`'s section 10 (`H3RMG`) logs every draw of the generation from `rand`'s tail with its call site and dumps the cells after each step, its hooks on their own stack; `h3rmg.py --phases`, `phases`. The corpus reproducible, no map changed by the hooks; a third of the draws come from the terrain and road painters outside the range (survey §2.6 lacks them); 0x532A5B draws in step 2 in 18 of 21, and `l1-4p-b`'s second map in one game differs by it, moved by the environment's size (T1b1's "21 of 21" was its sandbox's).

### 2026-11-23 — T1b1 the oracle (T1b cut by the user)

Staged the client lane's set (the PC's shell worked). `xdd.dll` fixes `time()`'s date (`H3TIME`), `h3rmg.py` sets the Random Map tab and keeps the map: 21 cases, each the same from fresh starts, in a second generation in one game and after another history (no memory-dependent variation under Wine); SoD's `rmg.txt` is read (the templates predicted 21 of 21); the plain writer as read; `h3_map` reads all, one refused at import (TWN-SET-001, for T1c). The VM restarted once (the baseline lost, CI's run on `master` stands for it).

### 2026-11-23 — T1a1 the frame; T1a2 in five parts by workers

Staged `Heroes3.exe` and the small files (the PC's shell worked), and two maps the original generated, found in Complete's `random_maps` (the description names the template and the seed). T1a1 read the frame (survey §1). The user cut T1a2 into five parts of about 22 kB, run by five worker sub-agents one after the other (a common brief, about 370 000 tokens each); the range closed (254 of 255 starts with an entry), the steps named, §2.6 the stream. Found: two memory-dependent draws, an HD-mod-patched `Untitled.h3m`, quirk candidates for T1c.
