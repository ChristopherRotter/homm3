# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 32). **M1 closed** (2026-11-13; M1's tasks in `E27-archive.md`; the catalogue `E27-design.md`
  §1 to §6). **M2 T1 to T4 done** (2026-11-13 to 14: `h3-smacker`, the clips' sound, the Bink
  copy, the shell's frame, the LODs' tables, `h3shell.py drawn`; design §7 to §9). **M2 T5a and
  T5a2 done** (2026-11-14 to 15, decisions 31, 32): `h3_view::shell::start` (the clips timed on
  the game's clock, drawn into a back buffer and a screen), `h3_bink::rects`, `h3_smacker::Rects`
  (`SMACKW32`'s `SmackToBufferRect`, as the DLL's on every call), the replay's start; the `start`
  group's 12 lines exact; design §10. **Next: M2 T5b** (the start's sound), then T6, T7, T8.
  Decisions to carry: the dialogs' input one implementation (20; `h3_ui::input`); the shell's
  code as the battle's, the score the engine's (21); decision 22's fixes; every pixel of a
  Smacker clip through its palette (24); every clip's sound at the effects setting (27); every
  Bink frame whole, the comparisons through `Target` (28); the start's fade from the screen as
  shown, a frame's whole rectangle shown (32). Open pull request: **#936** (`client/e27-m2-t5a2`) (watch it until it merges). E17 done
  (`E17-archive.md`, `E17-log.md`; its open threads: the epic's "Placed at the closure", the
  queue's inbox). E28 and E29 are backlog sketches. Workspace `clients/` (`clients/README.md`),
  gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** none. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle) and `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q125,
  CLT-STUB-6.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs, `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens) and the 40 non-campaign MP3s; `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-15, E27 M2 T5a2 (done; `client/e27-m2-t5a2`).** One task, as queued. The original
  data staged at once (64 files, checked by SHA-256); the baseline passed. Read `SMACKW32.DLL`'s
  `SmackToBufferRect` whole (the filler, the list's join, the iterator over the decoder's map of
  changed spans) and the game's reads in `video_show`; built `h3_smacker::Rects`,
  `Frame::spans`, `check::rects`, `smackprobe rects` and the `.rects` lines of
  `smacker-check.sh`. Every call of every frame as the DLL's at the first run; `drawn --group
  start` 12 of 12 exact (recorded). No new question (the DLL's oddities fall under CLT-Q124's
  fix). Record: `E27-log.md`, "M2 T5a2".
- **2026-11-14, E27 M2 T5a (done; `client/e27-m2-t5a`).** T5 cut in two with the user
  (decision 31). The original data staged at once (64 files, checked by SHA-256); the baseline
  died in its coverage step (CI's green `master` stood in). Recorded the `start` group once for
  its frames (as kept); read 0x44DD20, the fade, `SmackWait`, `BinkGetRects` and the game's union
  of the rectangles; built `h3_view::shell::start`, `h3_bink::rects` and `copies`, the replay's
  start, `--video-quality`, `Videos`' shell stand-in. 11 of 12 exact; `start-smacker`'s clip
  waits for SMACKW32's rectangles (T5a2, the cut offered). CLT-Q123, CLT-Q124 fixed (decision
  32). Record: `E27-log.md`, "M2 T5a".
- **2026-11-14, E27 M2 T4 (done; `client/e27-m2-t4`).** The cut offered, the user kept it whole
  (decision 30). The original data staged at once (64 files, checked by SHA-256); the VM restarted
  during the baseline (CI's green `master` stood in). Built `shell::replay`, `shell::look`, `Built`,
  `h3-view shell --steps --frames` and `--window`; `h3shell.py drawn`, `sound`, `mask`; recorded
  `mp-tcp` again (masked) and `menu-sound` (a press does not wait for `button.wav`). `drawn`: 3
  shots exact (the main menu at rest), the rest stop at T5's or T6's screens. `check.sh` with the
  data staged. Record: `E27-log.md`, "M2 T4".
