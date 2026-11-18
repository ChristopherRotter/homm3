# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2** and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 50 situations and 58 sequences exact on the whole screen (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"; `--bugs original` the comparisons' profile). **M4** refined (2026-10-31); T1a, T1b1, T1b2 done (2026-11-17); T2 cut in four by the user (2026-11-17: T2a the finds told, T2b the questions and choices, T2c a level-up's choice, T2d the map changed by a visit): **T2a done** (T2a1 2026-11-17, T2a2 2026-11-18: the box with pictures, the finds, kind 6, an object taken, the event's boxes, the turn's two boxes; design §5.3); T2b cut in two by the user (2026-11-18, as T2a): **T2b1 done** (2026-11-18: every question's and choice's box read, styles 2, 7 and 10 and the choice frames, 22 sequences `ask-*` and `choose-*` recorded into `clients/map/to-record.toml`; design §5.4), **next: T2b2** (the viewer plays them: the queue's entry, a cut offered at its start), then T2c, T2d; T3 (waits for the adventure lane's object identity; the server lane's whole-game record is there), T4 (has T1b2's gaps), T5 for the client lane's battle entry. Open questions: **MCL-Q29**, **MCL-Q30** (the sound run done: the new day sounds doubled; (b) recommended), **MCL-Q31**, **MCL-Q32** (two spells in a reward's first box replace the map's text; (b) recommended), **MCL-Q33** (the Arena's Cancel gives +2 Attack; (b) recommended); none blocks. Next id **MCL-Q34**.
- **Open pull requests** (2026-11-18): T2b1's **#1007** (`map-client/e24-m4-t2b1`): check it first. Merged branches are the user's to delete (the proxy refuses it).
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

- **2026-11-18, E24 M4 T2b2 — in progress** (the user at the start: no cut, the 22 whole): on
  `map-client/e24-m4-t2b2`. Done: original data staged (63 files, SHA-256 digest `9c890f81…` on
  both sides); the harness keeps `TMP`, `TEMP`, `TMPDIR` from Wine (Wine 9.0's loader aborted at
  every start); the 22 recorded again (running). Building in three parts by workers in turn: A1 (style 2,
  the eight declined, `ask-tomb`, `ask-hire`), A2 (the six accepted that change the map), B (the
  six choices); then `drawn`, `check.sh`, the docs and the PR. Nothing waits on the user's computer.
- **2026-11-18, E24 M4 T2b1** (the user at the start: T2b in two, as T2a): every question's and
  choice's box read (design §5.4: styles 2, 7 and 10, the choice frames; MCL-Q33), the harness's
  box marks and the words the load draws, 22 sequences recorded twice into `to-record.toml` (the
  log's "M4 T2b1"). Original data staged (63 files) and a ballast file for the game's disk check
  (TRAPS). Nothing is waiting on the user's computer.
- **2026-11-17 to 18, E24 M4 T2a2** (the user at the start: no cut): the viewer plays the
  eleven finds' sequences, every frame exact (the log's "M4 T2a2"), built by two workers in
  turn; `grant_rewards`' boxes read between them (MCL-Q32), two T2a1 misreadings corrected. The
  sequences recorded again here first. Original data staged (62 files) and a ballast file for
  the game's disk check (TRAPS). Nothing is waiting on the user's computer.

