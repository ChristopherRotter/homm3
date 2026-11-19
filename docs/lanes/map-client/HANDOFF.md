# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2**
  and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 53 situations and 96
  sequences exact on the whole screen (a viewer change runs `h3map.py drawn` first: the runbook,
  "Which captures a change needs"; `--bugs original` the comparisons' profile). **M4** refined
  (2026-10-31); **Part A done** (2026-11-19: T1, T2 in their cuts). **T3** cut in two by the user
  (2026-11-19): **T3a done** (design §5.9: `h3_map_view::engine`, the map's view from a record),
  **T3b** cut in two by the user (the same day, the panels first; a fact the record lacks an
  `MCL-Q<n>`, never a rule copied): **T3b1 done** (2026-11-19, design §5.10: the kingdom, the
  status window, the kingdom's window and `[chosen]` from the record, a town's own name, MCL-Q15
  (b) as `tiny.fnt`; `h3map.py engine` and `set.toml`'s `engine` entries: `engine-start-panels`
  exact on the whole screen, `engine-start-town` but MCL-Q36's morale, `engine-start-owners` but
  the bonus's gold and the morale). **T3b2 done** (2026-11-19, kept whole by the user; design
  §5.11: the `[hover]` facts from the record, every explored tile's hint and cursor compared, a
  still's `pointer` over the map, a town's picture by its view, a map's own town names; two
  shipped starts with no draw in sight, `start-jared` and `start-good-witch`; six engine starts as
  expected). **Next: the inbox's window-sound item, then T4** (has T1b2's gaps and T3b2's notes),
  T5 for the client lane's battle entry, T6 the closure. Open questions: **MCL-Q29**, **MCL-Q30**
  ((b) recommended), **MCL-Q31**, **MCL-Q32** ((b)), **MCL-Q33** ((b)), **MCL-Q34** ((a)),
  **MCL-Q35** ((b)), **MCL-Q36** and **MCL-Q37** (facts not in the record: a hero's morale, luck and
  whether it can move; the cursor's costs, guarded tiles, a defended town, a visited bank's guards;
  (a) recommended, the adventure and server lanes' inboxes told), **MCL-Q38** (a shipped map's town
  list order; (a)); none blocks. Next ids **MCL-Q39**, **MCL-STUB-2**.
- **Open pull requests** (2026-11-19): T3b2's (`map-client/e24-m4-t3b2`): check it first. Merged branches
  are the user's to delete (the proxy refuses it).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests, noted in that lane's inbox (`docs/formats/h3m.md`). It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none; a view's objects come in tile order without an index (found in the file by type and anchor, `h3_map_view::engine`); `h3-server record` writes a whole game's record per player (protocol v74, record format 5); every shipped map imports (E09).
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

- **2026-11-19, E24 M4 T3b2** (the user at the start: kept whole; MCL-STUB-1 still the adventure
  lane's): the hover's facts from a record, the towns' pictures, two shipped starts found by a
  scan of the 160 and compared, every explored tile's hint and cursor checked (the log's "M4
  T3b2"; MCL-Q37, MCL-Q38). Original data staged (63 files and the manuals), SHA-256 checked; the
  PC was not needed after the first minutes. Nothing is waiting on the user's computer.
- **2026-11-19, E24 M4 T3b1** (the user at the start: T3b in two, the panels first; missing facts
  asked, never copied): the panels from a record's start, MCL-Q15 (b), `h3map.py engine`, three
  engine situations captured and compared (the log's "M4 T3b1"; MCL-Q36). Original data staged
  (63 files and the manuals), SHA-256 checked; the PC was not needed after the first minutes.
  Nothing is waiting on the user's computer.
- **2026-11-19, E24 M4 T3a** (the user at the start: T3 in two, the map's view first; the object
  identity withdrawn, a random artifact's still asked): `h3_map_view::engine` and `render
  --record`/`--check`, two start situations captured (both exact) and the engine's starts equal to
  them, `tests/engine.rs` (the log's "M4 T3a"). Original data staged (63 files and the manuals),
  SHA-256 checked; the PC's shell went away about 15 minutes in, nothing needed by then. Nothing
  is waiting on the user's computer.
