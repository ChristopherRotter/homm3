# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M4 done** (M4, sieges, closed at T3c on 2026-10-27 with the user: the review list
  accepted, CLT-Q22 to Q24 answered (a); the lane split at M3 T9c, CLT-Q18: the battle client
  here, the adventure map client the map-client lane's E24). **M5 (live play)** (QUEUE, the
  epic's M5): hot seat in-process first, then a game on a server and accounts; the new
  dependencies the user's at T6 and T7. **T1 to T4 done** (the playhead; the timeline that grows,
  `live.rs`; the click's command; hot seat in the window; the spell book, design §7.17; the
  hero's aim, §7.18; the message box, Retreat and Surrender, §7.19; the result window, §7.20, its
  Bink decoder `h3-bink`, CLT-Q29, and its clip, §7.21; Auto Combat, §7.22, `h3_view::auto`; the
  tactics phase drawn, §7.23, `h3_view::tactics`, and played, §7.24: the engine's placement
  steps, `cross-lane`; the dissolves and the created stacks, §7.25, `anim::created`, the engine's
  `NotCreated`, `cross-lane`). **Next: M5 T5b** (the panel by the turn's controller, the sounds'
  waits, the palette effects: QUEUE), then T6. Open PR: T5a's (`client/e17-m5-t5a`,
  `cross-lane`; check it first). The inbox's CLT-Q30 task (`h3-bink` as BINKW32's float DCT)
  waits for a free slot.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§10; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5,
  CLT-Q26, CLT-Q27, CLT-Q28, CLT-Q32, CLT-Q33. Next id: CLT-Q34.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-10-29, E17 M5 T5a (done; its PR open: check it first).** T5 cut in two with the
  user; the engine's `NotCreated` (`cross-lane`, the user's call, wire v36). Read the dissolve's
  blend and rectangle, the stacks removed without a corpse, the created stacks, Clone, Teleport
  (design §7.25). Built `Screen::dissolve_rect`, `anim::created`, the window's notices.
  `teleport`, `clone`, `summon`, `demons` recorded, `sacrifice` exact; the clone's colours to
  T5b and the Demons' back buffer a slip not copied (the user's); CLT-Q33; items 36 to 39.

- **2026-10-29, E17 M5 T4c2b (done; #569 merged).** Kept whole by the user;
  the placement's path from the engine (`cross-lane`, the user's call: `tactics::place`, the gate
  opened on the way). Read the placement as the walk, Next Creature, Start Combat and the
  panels' swap, the shade over the turns, the computer's phase, Auto Combat in it, the keys
  (design §7.24). Built the buttons' commands, `Context::Tactics`, the window's panel shown,
  `tactics::shade_at`. `tactics-play` recorded: timeline as recorded, its frames exact but the
  walk's redraw artifacts; review items 32 to 35; no question.

- **2026-10-29, E17 M5 T4c2a (done; #564 merged).** T4c2 cut in two with the
  user (T4c2a the phase drawn, T4c2b played). Read the phase's message, its panel (built for the
  defender), the movement shade (overlay, search, drawer) and the cursor (design §7.23). Built
  `h3_view::tactics`, `Draw::Shade`, `[tactics_panel]`, the message before the first turn (the
  stills, the window); Tactics on the harness's maps. Seven stills exact; CLT-Q32; review items
  28 to 31.
