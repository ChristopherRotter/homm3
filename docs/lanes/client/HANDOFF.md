# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M6 done** (M6 closed at T7c on 2026-11-10; M6's record: `E17-archive.md`,
  `E17-log.md`). **M7, the battle client's last steps** (refined at T7c with the user; the tasks
  in QUEUE): **T1 done** (the answered slips, cut in four); **T2 done** (the DEF button's own
  loop, CLT-Q72 (a); design §13.1); **T3** the battle as an element a host's window runs (the
  map client's E24 M4 T5 waits on it), cut in four with the user: **T3a1 done** (`h3-view`'s
  binary became the library crate `h3-battle-client`), **T3a2 done** (the element
  `window::Hosted`, `h3-view window` its host; design §13.2; checked with the data staged in the
  next session), T3b cut in two with the user: **T3b1** a whole game's record's battle and its
  outcome handed back, **T3b2** the same played live through the host's game session; T3c the
  sound on the host's device; T4 the closure, **which closes E17**. **Next:** T3b1. The pull
  request of T3a1 and T3a2: the branch `claude/ecstatic-thompson-cjowg2`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** none of the lane's
  questions. Answered, waiting on other lanes: CLT-Q63 (a) (the automatic war machines' turns,
  the battle lane), CLT-Q65 (b) (a whole game's learned spells), CLT-Q69 (a) (the start split,
  the battle and server lanes; their inboxes, 2026-11-10).
  `CLT-STUB-1` (T5a, G1: a server's seat lacks the other hero's Orb and Cloak; the user requires
  parity with local play, asked of the battle and server lanes) is commented in
  `docs/stubs.toml` until `check-stubs` scans `clients/` (a `shared/` change). An engine change the lane needs goes as a small cross-lane PR with the user's consent
  (#739 and #745 were). Next ids:
  CLT-Q78, CLT-STUB-4.
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

- **2026-11-11, E17 M7 T3a2's checks (done; `claude/ecstatic-thompson-cjowg2`).** The
  original data staged at once (57 files); the baseline's `check.sh` on the branch was lost to a
  VM restart in coverage and passed when run again, the staged-data tests run (the windowless
  host test among them). The gate; `window --check` on a scenario fought here (a human, the
  result closed) and on a whole battle's record, every frame exact; `drawn` all the same. T3b
  cut in two with the user (T3b1, T3b2). Record: `E17-log.md`.
- **2026-11-11, E17 M7 T3a1 and T3a2 (done, T3a2 unverified; `claude/ecstatic-thompson-cjowg2`,
  no pull request).** T3 cut in four with the user, who agreed the entry's shape for the map
  client lane (the host owns the window and the frame, the battle an element drawn natively)
  and the adapter crate. The baseline's `check.sh` passed on `master`; no original data (no
  computer linked): the user chose T3a2 unverified. T3a1 the move (build, clippy, 1 326 tests);
  T3a2 the loop as `Hosted` (token-diffed against the old loop; 159 tests, the new staged-data
  test skipped). The map client lane's inbox told the shape. M7 review item 5. Record:
  `E17-log.md`.
- **2026-11-11, E17 M7 T2 (done; `client/e17-m7-t2`).** Kept whole with the user (the loop and
  the harness's stills). The original data staged at once (57 files); the baseline's `check.sh`
  passed on `master`. The loop in both profiles for the panel, the stats dialog, the options
  dialog and the boxes; the harness's `down:` and three stills, exact once the battle's and the
  stats creature's standing (which they found) were built. M7 review item 4. Record: `E17-log.md`.
