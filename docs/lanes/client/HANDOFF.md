# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 29). **M1 closed** (2026-11-13, T3; M1's tasks in `E27-archive.md`; the catalogue
  `E27-design.md` §1 to §6, §6.4 what M2 to M4 still read or record). **M2 T1, T2, T2b done**
  (2026-11-13 to 14: `h3-smacker`, the clips' sound, the DLL's masked Bink copy; design §7 to §9).
  **M2 T3a done** (2026-11-14, decision 29: T3 cut in two): `h3_ui::input` (the dialogs' shared
  input, the combat options on it), `h3_view::shell` (the menu loop's state machine) and
  `shell::text`, `h3-view shell`. **M2 T3b done** (2026-11-14, CLT-Q2's switch):
  `h3_battle_client::lods`, the LODs' tables at start and a batch by its ranges, `h3-view gate`
  measuring the start (1.3 ms; 145 ms before). **Next: M2 T4** (the comparison, `h3shell.py
  drawn`), then T5, T6. Decisions to carry: the dialogs' input one
  implementation (20; `h3_ui::input`); the shell's code as the battle's, the score the engine's
  (21); decision 22's fixes; a Smacker clip's every pixel through its palette (24); every clip's
  sound at the effects setting (27); every Bink frame whole, the comparisons through `Target`
  (28). Open pull request: **#923** (`client/e27-m2-t3b`) (watch it until it merges). E17
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
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q123,
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

- **2026-11-14, E27 M2 T3b (done; `client/e27-m2-t3b`).** One task, no cut. The original data
  staged at once (64 files, checked by SHA-256); the baseline passed. Built `h3_battle_client::lods`
  (`Lod`, `Reader`, `Lods`) and moved every LOD reader of the crate onto it (`render`, `battle`,
  `sequences`, `motion`, `log`, `text`, `sprite`, the binary's `situation` and `harvest`); `h3-view
  gate` measures the start; `h3_assets::archive::inventory` on a `LodTable`. Measured against
  `master`'s binary: the start 145 → 1.3 ms, a battle still's peak memory 243 → 28 MiB, the same
  PNG. `check.sh` with the data staged. Record: `E27-log.md`, "M2 T3b".
- **2026-11-14, E27 M2 T3a (done; `client/e27-m2-t3a`).** T3 cut in two with the user (decision
  29). The user's computer left the bridge at the start and came back; then the original data
  staged at once (64 files, checked by SHA-256); the baseline passed. Built `h3_ui::input` (moved
  from `h3_view::options`, whose tests pass unchanged; the map-client lane's inbox told),
  `h3_view::shell` (the menu loop's switches, design §2.2 to §2.4, §4.3, §5.3; each screen names
  its task) with `shell::text`, and `h3-view shell [--script] [--no-intro]`. `check.sh` with the
  data staged. Record: `E27-log.md`, "M2 T3a".
- **2026-11-14, E27 M2 T2b (done; `client/e27-m2-t2b`).** One task, no cut. The original data
  staged at once (64 files, checked by SHA-256); the baseline passed. Found by tracing blocks in a
  scratch copy against the DLL's planes and reading `BINKW32.DLL`: the decoding was right, the
  game's `BinkCopyToBuffer` copies only the cells `ExpandPlane` marked, the chroma's misfiled.
  Built `h3_bink::Target`, the decoder's mask, the samples' still cells; `Reading::Original`
  copies. Every frame of the shell's four Bink clips, the battle's ten and the samples the DLL's.
  CLT-Q122: shown whole (decision 28). The VM restarted once (files kept). Record: `E27-log.md`,
  "M2 T2b".
