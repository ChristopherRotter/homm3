# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2**
  and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 50 situations and 96
  sequences exact on the whole screen (a viewer change runs `h3map.py drawn` first: the runbook,
  "Which captures a change needs"; `--bugs original` the comparisons' profile). **M4** refined
  (2026-10-31); **Part A done** (2026-11-19): T1a, T1b (T1b1, T1b2), T2 cut in four by the user (T2a
  the finds told, T2b the questions and choices, T2c a level-up's choice, T2d the map changed by a
  visit: T2d1 a flag raised, T2d2 a boat boarded and left, **T2d3 a spell cast on the map**: Summon
  Boat, the user's choice, done whole 2026-11-19, design §5.8; MCL-Q14 seen and fixed as answered).
  **Next: T3** (a record's start: check its dependencies first, the adventure lane's object identity
  and increment 9's views; the server lane's whole-game record is there since protocol v43/v44, its
  inbox note of 2026-11-12 asks whether the object identity is still needed); if T3 still waits, the
  inbox's window-sound item (2026-11-19). T4 (has T1b2's gaps), T5 for the client lane's battle
  entry, T6 the closure. Open questions: **MCL-Q29**, **MCL-Q30** (the new day sounds doubled; (b)
  recommended), **MCL-Q31**, **MCL-Q32** (two spells in a reward's first box replace the map's text;
  (b) recommended), **MCL-Q33** (the Arena's Cancel gives +2 Attack; (b) recommended), **MCL-Q34**
  (the level-up's sentence naming the offers never shows; (a) recommended), **MCL-Q35** (a boarded
  boat drawn turned; (b) recommended); none blocks. Next id **MCL-Q36**.
- **Open pull requests** (2026-11-19): T2d3's (`map-client/e24-m4-t2d3`): check it first. Merged branches
  are the user's to delete (the proxy refuses it).
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

- **2026-11-19, E24 M4 T2d3** (the user at the start: Summon Boat of the four, whole, the cut
  after the reading, the harness's cast and the recordings the stated fallback, not needed): the
  cast read (design §5.8; MCL-Q14's stale mana bar seen), the harness's spell book and `[[cast]]`
  table and a hero's `spells`, `cast-boat`, `-far`, `-new` recorded twice and a sound run, the
  viewer built by one worker, every frame exact (the log's "M4 T2d3"). Original data staged (63
  files and the manuals) and a ballast file; the PC's tools went away about 90 minutes in.
  Nothing is waiting on the user's computer.
- **2026-11-19, E24 M4 T2d2** (the user at the start: big tasks cut only with their consent;
  T2d2 sized as T2d1 and done whole, the cut after the reading and the recordings the stated
  fallback): the boarding and the landing read (design §5.7, MCL-Q35), four sequences recorded
  twice and two sound runs, the viewer built by one worker, every frame exact (the log's "M4
  T2d2"). The session's server stopped overnight mid-build (the VM restarted, files kept): the
  worker resumed from its transcript. Original data staged (63 files and the manuals); the PC
  left about 20 minutes in. Nothing is waiting on the user's computer.
- **2026-11-18, E24 M4 T2d1** (the user at the start: T2d in three by family, "a monster beaten"
  to T5; MINEEVNT's cross-lane test word allowed, not needed: a family): the flag raised read
  (design §5.6: no redraw by the visit, the minimap at `visit_and_update`, the view at the next
  redraw; a dwelling's flag before its question), four `flag-*` sequences recorded twice, the
  viewer built by one worker, every frame exact (the log's "M4 T2d1"). Original data staged
  (63 files and the manuals) and a ballast file for the game's disk check. Nothing is waiting on
  the user's computer.
