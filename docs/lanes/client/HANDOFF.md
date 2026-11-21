# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E27, the game shell** (refined 2026-11-12, M0: `docs/epics/E27-game-shell.md`, decisions 4
  to 78). **M1 to M4 closed** (2026-11-13, -16, -19, -21; `E27-archive.md`; the build record
  `E27-log.md`; the readings `E27-design.md` §1 to §44): the start's clips and sound, the menus,
  the hot seat's names, the scenario screen and a game's begin (the setup kept for E24 M5, play's
  stub `CLT-STUB-13`), High Scores, Credits, the save file (the engine's adventure replay with
  the setup's choices and the client's table v2), Load Game single and hot seat (the loaded game
  and who is human after it kept for E24 M5: `save::Loaded`, `save::Humans`). **Next: none
  queued.** M5 (in play: decision 69) is refined with the user once E24 M5 hosts play; its
  prerequisites are the epic's "M5" section (the map-client and adventure lanes' inboxes told,
  2026-11-21); until then the lane's next work is the user's choice (E28, E29: `BACKLOG.md`).
  Decisions to carry (all: the epic's list): the dialogs' input one implementation (20;
  `h3_ui::input`); the shell's code as the battle's (21); every pixel of a Smacker clip through
  its palette (24); every clip's sound at the effects setting (27); the client draws whole where
  the original's partial redraws leave stale pixels (48, CLT-Q138's class); a save loads only
  under the rules and over the map it was made with (CLT-Q173: refused for now, the saves to be
  reworked later to be independent of the engine's version); the shell's recipes pinned to a
  time zone (`tz`); a disabled button takes no hotkey; the names decide who is human after a
  hot-seat load (CLT-Q160 (a)), a single load's current player alone (CLT-Q170 (a)). Open pull
  requests: T6's, branch `client/e27-m4-t6`, and the README's row, `shared/e27-m4-closure`:
  watch both until they merge. E17 done (`E17-archive.md`, `E17-log.md`; its open threads: the
  epic's "Placed at the closure", the queue's inbox). Workspace `clients/` (`clients/README.md`),
  gates `clients/scripts/check.sh`; ADR-0020 (+ drafts in `adr-drafts/`), ADR-0024; the harness
  `clients/capture/README.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*`, `E27*` (since E17 closed: #866) and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first (at M1 T3: one per area, the
  shots as contact sheets). **Open:** CLT-Q158 (after Reset the creatures' frame 0; built as the
  original's, blocks nothing), CLT-Q159 (Delete's unchecked failure; built as the original's,
  blocks nothing). Answered, waiting on other lanes: CLT-Q63 (a), Q65 (b), Q69 (a) (the queue's inbox,
  "E17's leftovers"). `CLT-STUB-1` (a server's seat lacks the other hero's Orb and Cloak),
  `CLT-STUB-4` (no Auto Combat switch in a live game's battle), `CLT-STUB-5` (a hot-seat
  battle between two of the host's humans) and `CLT-STUB-6` (the Multiplayer dialog's network
  modes), `CLT-STUB-7` and `-8` (Campaign, Tutorial), `CLT-STUB-9` (the starting bonus), `CLT-STUB-10` (the handicap), `CLT-STUB-11` (the turn's timer), `CLT-STUB-12` (the random map's generation) and `CLT-STUB-13` (play: the game begun or loaded, kept for E24 M5) are commented in `docs/stubs.toml` until
  `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a
  small cross-lane PR with the user's consent (#739 and #745 were). Next ids: CLT-Q174, CLT-STUB-16.
- Stage from `original-data/install/`: the four LODs (one per call), `h3maped.exe`, both `.snd`, `MP3/COMBAT01`–`04` and the end tunes, `Heroes3.exe`, the five DLLs (`MP3DEC.ASI` there too since 2026-11-17), `H3ab_ahd.vid`, and `original-data/battle-videos.vid` (the result window's clips: the harness's `VIDEO.VID`, the viewer's stand-in) and `original-data/shell-videos.vid` (E27's, 184 MB since T1b3a, a call of its own), `original-data/games/` (the installation's three saved games, copied at T1b1 for the load and save screens) and the 39 non-campaign MP3s (63 files); `cp -rn` the staged `original-data/.` into the clone. Frames stay in the sandbox.

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

- **2026-11-21, E27 M4 T6 (done; `client/e27-m4-t6`, `shared/e27-m4-closure`).** The PC's 71
  files staged (sizes and `installation.md`'s SHA-256). T6 in one session (decision 78): three
  fresh reviews (35 items kept, 7 changed, 28 findings, CLT-Q162 to CLT-Q173 answered), one fixer
  (24 fixes, design §44; the reading of Delete over the advanced options), every recipe played
  again (95, as kept; three with a first run a step off, as kept alone), `drawn` as recorded. M4
  to the archive, M5's prerequisites recorded, the inboxes told. Record: `E27-log.md`, "M4 T6".
- **2026-11-21, E27 M4 T5b2 (done; `client/e27-m4-t5b2`).** The PC's 71 files staged and checked
  as T5b1's. Kept whole. Read Random Map's case 130 (every item drawn
  and the redraw after its toggle, in a load too). Built a flag's press in a load (the saved
  choices taken, the client's profile's press only on a saved human's colour) and Random Map's
  whole redraw there. `adv-load` 19 of 19 exact. T6 cut in two (decision 77), T6a next session.
  Record: `E27-log.md`, "M4 T5b2".
- **2026-11-21, E27 M4 T5b1 (done; `client/e27-m4-t5b1`).** The PC's 64 files, the manuals and
  the help files staged (the PC's shell down: sizes against its listing, the LODs and executables
  against `installation.md`'s SHA-256). T5b cut in two, CLT-Q160 (a), CLT-Q161 (b) (decision 76).
  Read the load's dimmed duration bar (`dialog_item_darken`). Built the hot seat's load mode, the
  save's rows and duration, the names placed (§41.1), `Scenario::seated`, `h3-view save
  --player/--duration` and the makers' `viewer` players. `mp-load` 6 of 6, `adv-load` 14 of 19,
  `adv-load-more` 2 of 2. Record: `E27-log.md`, "M4 T5b1".
