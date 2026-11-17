# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2** and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 50 situations and 47 sequences exact on the whole screen (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"; `--bugs original` the comparisons' profile). **M4** refined (2026-10-31); T1a done (2026-11-17); T1b in two by the user: T1b1 and **T1b2 done** (2026-11-17: a computer player's turn in sight played by the viewer, its five `ai-*` sequences exact; speed 4's held draws on the window's clock; M4's review list started in the epic), **next: T2** (a visit's results; likely cut at its start with the user); then T3 (waits for the adventure lane's object identity; the server lane's whole-game record is there), T4 (has T1b2's gaps), T5 for the client lane's battle entry. Open questions: **MCL-Q29**, **MCL-Q30** (the sound run done: the new day sounds doubled; (b) recommended), **MCL-Q31** (nothing blocked). Next id **MCL-Q32**.
- **Open pull requests** (2026-11-17): T1b2's (`map-client/e24-m4-t1b2`): check it first. Merged branches are the user's to delete (the proxy refuses it).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests, noted in that lane's inbox (`docs/formats/h3m.md`). It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no identity yet, it plays 8 of the 160 shipped maps whole; `h3-protocol` carries no whole game yet (E16 M5's start, the server lane's, the user's order at M4 T0).
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

- **2026-11-17, E24 M4 T1b2** (the user at the start: no cut): the computer's screen read
  (the minimap's shield, kind 5's value), the five `ai-*` recorded again here, MCL-Q30's sound run
  (`h3map.py`'s `sound`), the viewer's build by a worker (the log's "M4 T1b2"). Original data
  staged (63 files) and a ballast file for the game's disk check (TRAPS). Nothing is waiting on the user's computer.
- **2026-11-17, E24 M4 T1b1** (the user present at the cut): T1b cut in two; `master`'s clients
  build fixed apart (#986, empty beside #988); the computer's turn read (design §5.2), the harness's `seed` (the game
  seeds its generator from the running clock) and kind 5's facts, five `ai-*` sequences recorded
  twice (the log's "M4 T1b1"); MCL-Q31. Original data staged (63 files) and a ballast file for the
  game's disk check (TRAPS). Nothing is waiting on the user's computer.
- **2026-11-16 to 17, E24 M4 T1a** (the user present at the cut, then away): T1 cut in two;
  the inbox's keys and purity fixes; the reading (design §5.1), seven `turn-*` sequences recorded
  and drawn exact, the viewer's turn's change by a worker (the log's "M4 T1a"); MCL-Q29, Q30.
  The server stopped twice and the VM restarted three times: the work resumed from the pushed
  commits. Original data staged (63 files) and a ballast file for the game's disk check (TRAPS).
  Nothing is waiting on the user's computer.
