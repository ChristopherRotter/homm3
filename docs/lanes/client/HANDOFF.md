# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 49). **M1 and M2 closed** (2026-11-13, 2026-11-16; `E27-archive.md`; the build record
  `E27-log.md`; the readings `E27-design.md` §1 to §17, M3's from §18): the start's clips and
  their sound, start-up loading, the main menu, New Game's and Load Game's menus, Quit, the
  stubs and the client's own pack, the Multiplayer dialog and the hot seat's names, the edits;
  `drawn` exact on every recipe the shell builds. **M3** (refined at M2 T8, decision 44: every
  shipped map listed and read whole, Begin a notice after the loading bar, Options to M4):
  **T1a to T3b2 done** (2026-11-17 and -18, decisions 45 to 57: the frame and panel, the list, its
  right press, the difficulty, the empty list, the description's bar, the advanced options' panel,
  rows, town, bonus and hero; each recipe exact as the epic's M3 table says: design §18 to §25).
  **T3c1 done** (2026-11-18: the flags' hover and press, the name's edit, the
  handicap drawn up, `CLT-STUB-10`; CLT-Q143 (b), decision 58; `adv-single-rest`'s first twelve shots exact:
  design §26). **Next: M3 T3c2** (the duration's bar and the hints: the queue), then T4.
  Decisions to carry: the dialogs' input one
  implementation (20; `h3_ui::input`); the shell's code as the battle's (21); decision 22's
  fixes; every pixel of a Smacker clip through its palette (24); every clip's sound at the
  effects setting (27); every Bink frame whole (28); the start's fade from the screen as shown
  (32); a clip's track played out (34); the main menu's buttons drawn in their order (36); the
  edits type what the keyboard prints (39); a menu opening anew takes the hover (40); a silent
  Smacker clip's frames from their due times (43); the empty list's leftovers fixed (51); the
  description's bar takes no list key while the list shows (52); a main hero of its type's
  portrait shown (55); a pick named by its own name (56); the name's edit drawn clean (58). Open pull request: T3c1's
  (`client/e27-m3-t3c1`): watch it until it merges. E17 done (`E17-archive.md`, `E17-log.md`; its open threads: the
  epic's "Placed at the closure", the queue's inbox). E28 and E29 are backlog sketches.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+
  drafts in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** none. Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle), `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) and `CLT-STUB-6` (the Multiplayer dialog's network
  modes), `CLT-STUB-7` and `-8` (Campaign, Tutorial), `CLT-STUB-9` (the starting bonus) and `CLT-STUB-10` (the handicap) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q144, CLT-STUB-11.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs (`MP3DEC.ASI` there too since 2026-11-17), `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens) and the 40 non-campaign MP3s; `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-18, E27 M3 T3c1 (done; `client/e27-m3-t3c1`).** The PC's 64 files and the manuals
  staged (SHA-256 checked). Read the flag's press, the name's click, the hover, the name's edit's
  own key and loss of focus (its text back to the record); CLT-Q143 asked and answered (b)
  (the edit's doubled frame, the name under it), the original's for the comparisons; built the hover,
  the press, the edit, the screen's flags from the moved colour; `adv-single-rest`'s first twelve
  shots exact, every other group as recorded. Record: `E27-log.md`, "M3 T3c1".
- **2026-11-18, E27 M3 T3b2 (done; `client/e27-m3-t3b2`).** The PC's 64 files and the manuals
  staged (SHA-256 checked). The sandbox's `TMPDIR` crashed Wine; the harness drops it now
  (`TRAPS.md`). Read `scenario_hero_popup` whole; built the popup (`own_hero_popup`, the keys
  `shell.advanced.hero-popup.*`, `hero-specialty`, the classes from the game's rules); the four
  hero popups exact, every other group as recorded. Record: `E27-log.md`, "M3 T3b2".
- **2026-11-18, E27 M3 T3b1 (done; `client/e27-m3-t3b1`, #1008).** T3b cut in two with the user. The
  PC's 64 files and the manuals staged at once (SHA-256 checked). Read the hero's list, step, shown
  hero and name, the row's hero draw and the right press; CLT-Q141 and CLT-Q142 asked and
  answered (b); built the offers from each map's import (`maps::heroes`, the engine's
  `GameSetup::offer`), the hero's arrows, portraits and names, Random's popup, the asset tables'
  `small_portrait`; `adv-single-hero` 26 of 28 and `adv-single-maps` 11 of 13 exact (a hero's own
  popup: T3b2). Record: `E27-log.md`, "M3 T3b1".
