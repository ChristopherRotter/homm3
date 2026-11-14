# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 27). **M1 closed** (2026-11-13, T3; M1's tasks in `E27-archive.md`; the catalogue
  `E27-design.md` §1 to §6, §6.4 what M2 to M4 still read or record). **M2 T1 done** (2026-11-13:
  `h3-smacker`'s video, design §7). **M2 T2 done** (2026-11-14: the clips' sound, every track the
  DLLs' byte for byte; design §8; decisions 25 to 27). **Next: M2 T2b** (the shell's Bink clips'
  video as `BINKW32.DLL`'s, found at T2: design §8.4; before T5) or **T3** (the shell's frame;
  independent). Decisions to carry: the dialogs' input one implementation shared with the
  battle's (20); the shell's code as the battle's, the score the engine's (21); decision 22's
  fixes; a Smacker clip's every pixel through its palette (24); every clip's sound at the effects
  setting (27). Open pull request: **#915** (`client/e27-m2-t2`) (watch it until it merges). E17
  done (`E17-archive.md`, `E17-log.md`; its open threads: the epic's "Placed at the closure", the
  queue's inbox). E28 and E29 are backlog sketches. Workspace `clients/` (`clients/README.md`),
  gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`; design §3–§13; records in `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** none. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle) and `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q122,
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

- **2026-11-14, E27 M2 T2 (done; `client/e27-m2-t2`).** Kept whole with the user (decision 25).
  Built a stand-in Miles for the probes (`capture/mss32.c`), `smackprobe audio`, `binkprobe
  sound`; `h3_smacker::sound` (FFmpeg's), `h3_bink::audio` with Ooura's FFT as `BINKW32.DLL`
  compiled it (CLT-Q120: ported, credited; generated from a symbolic run of the DLL's x87 code),
  `h3_view::clip_sound`, `HostSound::play_clip`, `video --play`. Every track of the shell's clips
  and the samples the DLLs' byte for byte. CLT-Q121: every clip at the effects setting. Found:
  `h3-bink`'s video of the shell's four Bink clips is not the DLL's (T2b). The VM restarted once
  (files kept). Record: `E27-log.md`, "M2 T2".
- **2026-11-13, E27 M2 T1 (done; `client/e27-m2-t1`).** Kept whole with the user (decision 23).
  The original data staged at once (64 files, checked by SHA-256); the baseline lost to a VM
  restart, CI's green `master` stood for it. Built `h3-smacker` (FFmpeg 6.1.1's Smacker ported,
  100 % covered), `video --smacker`, `smackprobe.exe`, `smacker-dll.py`, `smacker-check.sh`. Checked
  against FFmpeg and `SMACKW32.DLL`: the shell's eight clips, AB's `Credits.smk` and 150 samples,
  every frame, palette and 16-bit copy byte for byte. Found: the DLL's 16-bit copy keeps a skipped
  block's colours after a palette change (CLT-Q119, fixed: decision 24); the DLL faults on a byte
  tree left out and on a key-frame mark (samples avoid both). Record: `E27-log.md`, "M2 T1".
- **2026-11-13, E27 M1 T3 (done; `client/e27-m1-t3`).** Kept whole with the user (decision 19),
  sub-agents for the long parts. The original data staged at once (64 files, checked by SHA-256).
  Every recipe but the two full starts recorded again: all as kept. Two fresh reviewers (the
  catalogue; §3 with 383 shots as contact sheets), a fixer for the doc fixes: 57 findings fixed in
  place, listed in design §6.4 or asked; CLT-Q100 to CLT-Q118 answered (decisions 20 to 22); the
  review list accepted, M1 archived, M2 refined (eight tasks). The VM restarted once (after the
  runs). Record: `E27-log.md`, "M1 T3".
