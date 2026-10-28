# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M4 done** (M4, sieges, closed at T3c on 2026-10-27 with the user: the review list
  accepted, CLT-Q22 to Q24 answered (a); the lane split at M3 T9c, CLT-Q18: the battle client
  here, the adventure map client the map-client lane's E24). **M5 (live play)** (QUEUE, the
  epic's M5): hot seat in-process first, then a game on a server and accounts; the new
  dependencies the user's at T6 and T7. **T1 to T3, T4a and T4b1 done** (the playhead; the
  timeline that grows, `live.rs`; the click's command; hot seat in the window; the spell book,
  design §7.17; the hero's aim, §7.18; the message box, Retreat and Surrender, §7.19; the result
  window without its clip, §7.20). T4 cut in three, T4b in two by the user (2026-10-28). **Next:
  M5 T4b2** (the window's Bink clip: a port of FFmpeg's decoder, CLT-Q29), then T4c (Auto Combat,
  the tactics phase). Open PR: #547, T4b1's (`client/e17-m5-t4b1`; check it first).
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§10; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5,
  CLT-Q26, CLT-Q27, CLT-Q28. Next id: CLT-Q30.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, for T4b2 `original-data/battle-videos.vid` (the result window's clips); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-10-28, E17 M5 T4b1 (done; #547 open: check it first).** T4b cut in two with the
  user (CLT-Q29: the clip T4b2, a port of FFmpeg's Bink decoder; the twelve clips copied to
  `original-data/battle-videos.vid`). Read the window (design §7.20: 0x4702D0 whole, its handler
  0x471B90, `battle_end`'s calls, `battle_experience` 0x46A350, the DEF item's mirror). Built
  `h3_view::result`, `results::window` (the engine's outcome), the window's, stills' and
  sequences' result; the heroes' portraits, a DEF item's mirror. `result-retreat`,
  `result-surrender` exact; `end`'s last frame exact, `defended`'s but 122 artifacts; review
  items 12–14.

- **2026-10-28, E17 M5 T4a (done; #542 merged).** T4 cut in three with the
  user. Read the box (design §7.19: 0x4F6570, 0x4F6990, the layout 0x4F5D80, the frame 0x48FF00,
  the font's measures, the picture slot, the handlers), Retreat and Surrender (0x474FA1 to
  0x475151), the book's and the aim's boxes, the text tables' quotes (0x5BBBA0). Built
  `h3_view::message`, the binary's `boxes`, the window's and the stills' boxes, `--leave`.
  `msg-*` (six) exact; `drawn` the other 124 the same; window checks by `--leave` and real keys.
  CLT-Q28; review items 8–11; the battle lane's inbox (`h3_formats`' quotes).

- **2026-10-28, E17 M5 T3b (done; #536 merged).** Whole (the user). Read the
  aim (design §7.18: 0x59EC50's handlers and ends, the loop 0x602520 with 0x5A3070, the cursor
  0x59F700, the cast's line 0x5A8690, the refusals 0x5A2880, the area's marks 0x59F8A0 and
  0x43EFE0, the spell cursor's frames in `mouse_poll`, the key F). Built `mouse::aim`, the aim in
  `track`, the binary's `aims`, the window's aim (F, Escape, the right button, F11 full screen),
  unattended casts through the book and the aim; the harness's `aim:` and the 100 ms grid.
  `spell-aim` recorded; `cast`'s 71 pixels a listed redraw artifact; CLT-Q27; review items 5–7.
