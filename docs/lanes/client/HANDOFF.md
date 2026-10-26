# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M3 done** (the lane split at M3 T9c, CLT-Q18: the battle client here, the adventure
  map client the map-client lane's E24); **M4 (sieges): T0 to T2c-2 done** (§10.1, §10.2,
  CLT-Q19 to Q21); **T3 cut in three** (the user, 2026-10-26: T3a the reviewers' findings
  fixed, T3b every capture afresh with the hovered item read, T3c the review list and M5 with
  the user); **T3a done; next T3b**; M5 live play after M4. Open PR: #485 (T3a; check it
  first). Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`; design §3–§10; records in `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q4–Q6, Q22, Q23.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`; `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

## Autonomy contract (agreed by the user, 2026-10-12)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Ask as an open question** (the user, 2026-10-22): every oddity of the original a reading or a recording finds becomes a `CLT-Q<n>` in the same task (copy, fix with the original's for the comparisons only, or an option; a recommendation); build it as the original's meanwhile.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 was one), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`CLT-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: CI out, purity allowlist, licences and SDL, the lockfile gate, never running the original from the installation, memory against screen, no floats in drawing, the harness's actions and rolls, captures, sieges.

## Asset notes

The user's dictation of 2026-10-12 and the research items it led to: `ASSET-NOTES.md` (client lane's file; the map-client lane's own research items in `docs/lanes/map-client/ASSET-NOTES.md`). Read it before decoding or drawing an asset.

## Recent sessions

- **2026-10-26, E17 M4 T3a (done; #485 open: check it first).** T3 cut in three (the user).
  Three fresh reviewers one at a time (R1 drawing and timeline, R2 mouse and adapters, R3
  harness): 16 of 17 findings fixed, R1-3 queued (QUEUE's inbox); review items 10 to 12. The
  harness now refuses rolls it cannot force (`xdd.dll`'s ack: a 7th field). `drawn` unchanged.
- **2026-10-26, E17 M4 T2c-2 (done; #483 merged; not cut: the user).** Earthquake,
  the end tune at a town, the siege's mouse (16, 22, the broken arrow); `quake`, `defended`,
  `aim`, `through` as recorded, frames exact but listed differences; T2b's and T2c-1's nine
  recaptured without their ` log` masks. Found: the status line kept at a hot-seat human turn
  with the mouse parked (built as found, *provisional*; `aim`'s exception and the dispatch
  reading in QUEUE's inbox). CLT-Q22 and Q23 still open.
- **2026-10-24, E17 M4 T2c-1 (done; #459 merged).** T2c cut in two (the user).
  The drawbridge, the moat, a mine (`anim::siege`); `gate`, `gate-fortress`, `moat`, `mine` as
  recorded, frames exact but listed differences; CLT-Q23 asked (built as recommended).
