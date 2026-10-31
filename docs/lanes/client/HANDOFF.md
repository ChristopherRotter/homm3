# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M4 done** (M4, sieges, closed at T3c on 2026-10-27 with the user: the review list
  accepted, CLT-Q22 to Q24 answered (a); the lane split at M3 T9c, CLT-Q18: the battle client
  here, the adventure map client the map-client lane's E24). **M5 (live play)** (QUEUE, the
  epic's M5): hot seat in-process first, then a game on a server and accounts; the new
  dependencies the user's at T6 and T7. **T1 to T7 done** (the playhead; the timeline that grows,
  `live.rs`; the click's command; hot seat in the window; the spell book, design §7.17; the
  hero's aim, §7.18; the message box, Retreat and Surrender, §7.19; the result window, §7.20, its
  Bink decoder `h3-bink`, CLT-Q29, and its clip, §7.21; Auto Combat, §7.22, `h3_view::auto`; the
  tactics phase drawn, §7.23, `h3_view::tactics`, and played, §7.24: the engine's placement
  steps, `cross-lane`; the dissolves and the created stacks, §7.25, `anim::created`, the engine's
  `NotCreated`, `cross-lane`; T5b cut in two, the palette effects done, §7.26,
  `h3_render::recolour`; the panel by each turn's controller and the sounds' waits, §8.3,
  `Timeline::controllers`, `sound::Lengths`, the harness's `sound` runs: T5 done; T6 cut in two
  by the user, T6a done: the connection headless, design §11.1, `h3_view::remote`, the binary's
  `net` and `connect`, the server's TLS stack the user's choice; T6b cut in two by the user,
  T6b1 done: the window plays a seat on a server, design §11.2, `remote::seen`, the binary's
  `online`, `Battle`'s `Fight`; T6b2 done, kept whole by the user: the record that opens
  mid-battle, design §11.3, `anim::lies`, `scene::living`, the moat by fixed rows,
  `remote::rejoined`, `Battle::afresh`: T6 done; T7 done, kept whole by the user: accounts,
  design §11.4, `argon2` the engine's and the password from standard input (the user's
  choices), `h3-view account`, `remote::Accounts`, `Client::login` with the re-key,
  `remote::farewell`). **M5 T8** (M5's closure) cut by the user into concurrent fractions (two
  capture agents on the harness's two instances, one reviewer or fixer; QUEUE): **T8a done**
  (#606), **T8b1 done** (#614): every capture as recorded, R1 to R4, R7, R2a/R2b fixed; **T8b2
  done** (PR from `client/e17-m5-t8b2`; check it first): R4b, R5a to R5c, R6 fixed, the
  leftovers (the playhead carried for entered stacks, the Gold bonus read in memory). **Next:
  T8c** at the user's checkpoint (QUEUE): review items 1 to 86, CLT-Q35, the next milestone
  refined. The inbox's CLT-Q30 task (`h3-bink` as BINKW32's float DCT) and the user's Demons
  note (T5a's dissolve, to re-investigate) wait for a free slot.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§11; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q5,
  CLT-Q26, CLT-Q27, CLT-Q28, CLT-Q32, CLT-Q33, CLT-Q35. Next id: CLT-Q36.
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

- **2026-10-30/31, E17 M5 T8b2 (done; its PR open: check it first).** The original data staged
  (the user's computer offline half an hour in); the user's cut: three concurrent sub-agents, the
  runner the leftovers; the VM restarted ~10 minutes in (CI's green `master` the baseline).
  Reviewers R5a, R5b, R6, R4b, R5c and their fixers in worktrees, merged; the leftovers; the
  harness set up for a live probe of the Gold bonus (0x69FBE8, observed); `drawn` all 156 the
  same; `check.sh`. Items 78–86, CLT-Q35.

- **2026-10-30, E17 M5 T8b1 (done; #614 merged).** The original data staged
  (the user's computer offline an hour in); the user's cut of T8b in two; the VM restarted ~40
  minutes in (two captures rerun). R7 (the harness) and R2a/R2b (the decoder, the clip) and
  their fixers (F7 in a worktree, merged with both instances idle and set up again); sequences
  28 to 64 as recorded (`auto`'s 7 530 now exact, recorded); the harness check passed at its
  rerun; `drawn` and `check.sh` with the fixes. Items 73–77.

- **2026-10-30, E17 M5 T8a (done; #606 merged).** The original data staged;
  the VM restarted ~10 minutes in. The user's cut: three concurrent sub-agents, each a fraction
  of about half a context. Built the harness's two instances (one fault found by a capture
  agent: `h3mem.game_pid` read the other instance's game; fixed, the void ids redone) and the
  recipes' bonus Gold; reviewers R1, R3, R4 and their fixers (log); 91 stills and sequences 0
  to 27 as recorded; `drawn` all 156 the same with the fixes; `check.sh` green. Items 65–72.
