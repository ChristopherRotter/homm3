# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M4 done** (M4, sieges, closed at T3c on 2026-10-27 with the user: the review list
  accepted, CLT-Q22 to Q24 answered (a); the lane split at M3 T9c, CLT-Q18: the battle client
  here, the adventure map client the map-client lane's E24). **M5 (live play)** (QUEUE, the
  epic's M5): hot seat in-process first, then a game on a server and accounts; the new
  dependencies the user's at T6 and T7. **T1a, T1b and T2a done** (the playhead; the timeline
  that grows, `live.rs`; the click's command, `mouse::click` and `button`). T2 cut in two by the
  user (T2a, T2b; the Faerie Dragon's cast into T3). **Next: M5 T2b** (hot seat in the window).
  Open PR: #515 (T2a; check it first). Workspace `clients/` (`clients/README.md`), gates
  `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`; design §3–§10; records in `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5. Next
  id: CLT-Q26.
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

- **2026-10-27, E17 M5 T2a (done; #515 open: check it first).** T2 cut in two by the user (T2a,
  T2b; the Faerie Dragon's cast into T3). Read (design §9.11): the press by the cursor's kind,
  the slots' from-hexes, the input queue emptied at every turn's start (T6c-2's queued moves
  corrected). `mouse::click`, `button`, `attack_slots`; CLT-Q24's switch; `clicks.rs`: presses
  over 1 455 turns give the legal commands but CLT-Q25's walk (asked and answered: the
  original's copied, an option later). `sequences` all 47 as recorded.
- **2026-10-27, E17 M5 T1b (done; #509 open: check it first).** Whole, as cut at T1a.
  `anim::Growing` (the timeline a command at a time, only appended to; a drawbridge span open
  until the next redraw), `Playhead::extend`; `battle.rs` as `Ground` and `Battle<'g>`,
  `live.rs` (the session in-process, the bot's command when a frame reaches the timeline's end);
  `motion`'s `every`; the panel's reads logarithmic. `play` byte for byte as fought at once;
  `window --check` 1 094 frames exact. No new question.
- **2026-10-27, E17 M5 T1a (done; #506 merged).** T1 cut in two (the user: T1a,
  T1b). The sandbox new, the original files staged again before the computer disconnected.
  Measured: R2-7's 2.3 ms was the composing; `Moment::at` 1.6 to 7 µs over `play`'s battle,
  306 µs half an hour into its idle end. `anim::Playhead` reads the timeline forward (0.3 to
  1 µs anywhere); `play`, `window`, `sequences` draw through it, `sequences` checks it against
  `Moment::at` at every recorded change, `play --moments` times both. `drawn` all 119 the same.
  The panel's per-frame scans of the turns left for T1b. No new question.
