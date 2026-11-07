# Client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E17 M0–M5 done** (M5 closed at T8c on 2026-10-31: the review list accepted; the lane split at M3
  T9c, CLT-Q18: the battle client here, the adventure map client the map-client lane's E24). **M6
  (the battle screen complete)**, its tasks in QUEUE and the epic's M6 (each task's cuts, dates and
  pull requests there): T1 to T5 done (T5e deferred by the user to the battle client inside an
  adventure game, the epic's "Later"). **T6** cut in three by the user (2026-11-07): **T6a done**
  (#781); T6b cut in two by the user (2026-11-07; R4-9 moved into T7): **T6b1 done** (#785, CLT-Q5's
  deck and boats, design §6.2); **T6b2 done** (#789, CLT-Q67, design §7.25). T6c cut in two by the
  user (2026-11-07): **T6c1 done** (`h3-bink`'s DCT as `BINKW32.DLL`'s x87 one in integers; the ten
  clips and the samples exact against the DLL's planes; CLT-Q68 answered fix; design §7.27; the PR
  on `client/e17-m6-t6c1`: check it first). **Next: T6c2** (the captures' listed DCT pixels captured
  again, `drawn` recorded: its result-window clip digests are stale since T6c1, no gate runs it;
  CLT-Q30 closed), T7 (with R4-9); then M7 ranked play and the server browser.
  M5's record: `E17-archive.md`, `E17-log.md`.
  Workspace `clients/` (`clients/README.md`), gates `clients/scripts/check.sh`; ADR-0020 (+ drafts
  in `adr-drafts/`), ADR-0024; the harness `clients/capture/README.md`; design §3–§12; records in
  `E17-log.md`.
- **Captures** (the user, 2026-10-20): `h3drive.py drawn` first (no Wine); capture what it lists or the task changes, all at a milestone's closure or a harness change (runbook); a capture not as recorded leaves `kept/` (T9b: record, then `--reuse`); later tasks' detail in `NOTES.md`.
- **The user's requirements** (epic, "Decisions"): a string catalogue (JSON packs, no committed translation, CLT-Q7); graphics options (`faithful` default); exhaustive diffs; drawing bugs fixed, the original's only for comparisons (`Bugs`).
- The lane owns `clients/` but the map-client lane's paths and the common ones (`lanes.toml`: the foundation, `text/`, the harness; an additive change freely, a change in behaviour into the map-client lane's inbox), plus `docs/epics/E17*` and this folder; branches `client/<task>`; ids `CLT-Q<n>`, `CLT-QUIRK-<n>` (asset fixes), `CLT-STUB-<n>`; ADR drafts in `adr-drafts/`, numbered when accepted.
- **Checkpoints** per milestone (below), a fresh reviewer agent first. **Open:** CLT-Q42 (T3b2: copy recommended, built so), CLT-Q43 and Q44 (T4a: fixes
  recommended, built as the original's), CLT-Q63 (T5b2a: copy recommended, not built), CLT-Q64 (T5b2b: fix recommended, CLT-Q45 (1)'s rule built), CLT-Q65 (T5c: the unlearned
  picks; fix once a whole game tells what was learned), CLT-Q66 (T5c: the window under the
  boxes; copy recommended, built so). `CLT-STUB-1` (T5a, G1: a server's seat lacks the other
  hero's Orb and Cloak; the user requires parity with local play, asked of the battle and server
  lanes) and `CLT-STUB-3` (T5c: a seat gets no Eagle Eye picks at the end; asked of the server
  lane) are commented in `docs/stubs.toml` until `check-stubs` scans `clients/` (a `shared/`
  change). An engine change the lane needs goes as a small cross-lane PR with the user's consent
  (#739 and #745 were). Next ids:
  CLT-Q69, CLT-STUB-4.
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

- **2026-11-07, E17 M6 T6c1 (done; `client/e17-m6-t6c1`).** T6c cut by the user. The original
  data staged by the file tool (the PC's shell did not start). Read the DLL's DCT (a symbolic
  trace of its x87 code), built it in integers, checked against the DLL's planes under Wine
  (`binkprobe` now at the game's control word); CLT-Q68 asked (fix). Record: `E17-log.md`.
- **2026-11-07, E17 M6 T6b2 (done; `client/e17-m6-t6b2`).** Kept whole. The original data
  staged by the file tool before the PC went (its shell not used). Read (the cast's backdrop,
  `compose_battle_background`'s whole copy), probed (the back buffer read from memory each
  millisecond of `demons`), asked (CLT-Q67, answered (a)), built (`scene::Saved`, `compose`'s
  `back_buffer`), `demons` exact; reviewed by a fresh reviewer. Record: `E17-log.md`.
- **2026-11-07, E17 M6 T6b1 (done; `client/e17-m6-t6b1`).** T6b cut by the user (R4-9 into
  T7). The original data staged by the file tool (the PC's shell did not start). Built (`h3-view situation`'s boats, `h3drive.py`'s `board` and
  `into_battle`'s `click`), recorded (`deck`, `boats` exact), the whole set captured again,
  reviewed by a fresh reviewer. Record: `E17-log.md`.
