# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** and **M2** closed (`E24-archive.md`): the crate `clients/crates/h3-map-view`; 50 situations and 35 sequences exact on the whole screen since M3 T7a (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"; `--bugs original` the comparisons' profile). **M3** (the adventure screen; the epic's M3 section): **T1a** (#545) to **T7b2** (#623) merged; T7c cut with the user (2026-10-31): **T7c1 built** (M3's review list accepted; MCL-Q13 to Q15 and Q20 to Q24 answered; the hand test on the user's machine passed, all as described; the Windows build's catalogue read beside `--map`; the log's "M3 T7c1"). **Next: T7c2** (MCL-Q22's and Q24's answers, the client's fixes with the original's behind `--bugs original`; M3 archived), then **M4 T0** (M4 refined with the user: the queue). Open questions: none. Next id **MCL-Q28**.
- **Open pull requests** (2026-10-31): T7c1's (`map-client/e24-m3-t7c1`; this lane's paths and the common `OPEN-QUESTIONS.md`): check it first (merged, or its mergeability with `master`). A merged branch is the user's to delete: the session's proxy refuses it.
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests, noted in that lane's inbox (`docs/formats/h3m.md`). It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no index (M4's need, the adventure lane's inbox); no adventure messages in `h3-protocol` yet.
- **Checkpoints** per milestone (the autonomy contract below): fresh reviewers (one per area), the user's review list, a hand test on the user's machine where the window changed (`clients/out/win/`: a `.cmd` with the checks on screen, a notes file).

## Autonomy contract (agreed by the user, 2026-10-12, for the client lane; this lane's since the split, 2026-10-23)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Ask as an open question** (the user, 2026-10-22): every oddity of the original a reading or a recording finds becomes an `MCL-Q<n>` in the same task (copy, fix with the original's for the comparisons only, or an option; a recommendation); build it as the original's meanwhile.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 was one; a change to the client lane's paths, `h3-view` included), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`MCL-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: the gate's and `h3map.py`'s staged files, a template's class in memory, `STALE` situations, one game at a time; the client lane's traps apply to the foundation and the harness (`docs/lanes/client/TRAPS.md`).

## Asset notes

The user's dictation (2026-10-12): `docs/lanes/client/ASSET-NOTES.md`; this lane's research items: `ASSET-NOTES.md` here.

## Recent sessions

- **2026-10-31, E24 M3 T7c1** (the user present and at the machine: T7c cut into T7c1, T7c2 and M4
  T0): the review list accepted, the eight questions answered, the hand test passed (the log's "M3
  T7c1"; its pull request); the original data staged first; the baseline killed by the agent's
  pause of it: CI's green `master` the starting point. Nothing is waiting on the user's computer
  (`clients/out/win/m3/` holds the build and its data).
- **2026-10-31, E24 M3 T7b2** (the user present: the agent's cut by area declined, all 24
  findings in one task): T7b2 built (the log's "M3 T7b2"; its pull request); the original data
  staged first; the baseline spoiled by edits made during it: CI's green `master` the starting
  point. Nothing is waiting on the user's computer.
- **2026-10-31, E24 M3 T7b1** (the user present: T7b cut into T7b1, F2 and F4, and T7b2, the
  rest; MCL-Q25, MCL-Q26 answered, MCL-Q27 asked and answered): T7b1 built (the log's "M3
  T7b1"; its pull request); the original data staged first; the VM restarted during the
  baseline: CI's green `master` the starting point. Nothing is waiting on the user's computer.
