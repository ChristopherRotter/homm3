# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2**
  and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 53 situations and 99
  sequences (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change
  needs"; `--bugs original` the comparisons' profile). **M4** refined (2026-10-31): Part A done
  (2026-11-19); T3 done in its cuts (T3a, T3b1, T3b2: a record's start, design §5.9 to §5.11);
  T4 cut in three by the user (T4a the viewer's own turns, T4b the visits, T4c the other players,
  the controls and a golden replay): **T4a1 done** (design §5.12), the labels task done
  (ADR-0009), **T4a2 done** (2026-11-20; design §5.13: the scripted games played from their
  engine records, `sequences <id> --record`, the gaps in `timelines/<id>-gaps.txt`); T4b cut in
  two by the user (2026-11-20, by family; Summon Boat out of T4b): **T4b1 done** (the message
  boxes from a record's visits, design §5.14: `game-finds`, `game-asks`, `game-level` exact but
  MCL-Q41's chest amount). **Next: T4b2** (the flags and the boats from a record), then T4c, T5,
  T6. The inbox: the viewer's tests' catalogues hold a few of the original's sentences (the
  user's to decide). Open questions: **MCL-Q29** to **MCL-Q42** (none blocks; MCL-Q36, Q37, Q39
  to Q42: facts not in the record, (a) recommended, the engine lanes' inboxes told). Next ids
  **MCL-Q43**, **MCL-STUB-2**.
- **Open pull requests** (2026-11-20): T4b1's (`map-client/e24-m4-t4b1`): check it first. Merged
  branches are the user's to delete (the proxy refuses it).
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

- **2026-11-20, E24 M4 T4b1** (the user at the start: T4b cut in two by family, T4b1 kept
  whole; Summon Boat out of T4b): three scripted games with visits recorded and played from
  their records (`engine::visits`, the record's level-ups, `engine_seed` by a search; the
  timeline's lingering `flush` fixed); MCL-Q41, MCL-Q42 asked (the log's "M4 T4b1"). Original
  data staged (the 64 files and the manuals), SHA-256 checked; the PC was not needed after the
  first minutes. Nothing is waiting on the user's computer.
- **2026-11-20, E24 M4 T4a2** (the user at the start: kept whole, the suggested cut declined):
  the three scripted games recorded again and played from their engine records (`engine::played`,
  the timeline's `moves`, the path's arrows, the walks' reveal on the view and the minimap,
  `sequences --record`, `h3map.py`'s `scripted`; the gaps listed); `master`'s #1066 merged in and
  the games checked again (the log's "M4 T4a2"). Original data staged (the 64 files and the
  manuals), SHA-256 checked; the PC was not needed after the first minutes. Nothing is waiting on
  the user's computer.
- **2026-11-20, E24 M4, the kept texts as labels** (the user at the start: the queued task,
  kept whole): `h3lod.label` and `h3map.py`'s writers; the 42 kept live parts' 353 texts as
  their records (135 lines), three sequences recorded again to check it; the guard
  `tests/texts.rs`; a scan of `clients/map/` for the original's texts (names only), the
  viewer's test catalogues found (the inbox; the log's "M4, the labels"). Original data staged
  (the 64 files and the manuals), SHA-256 checked; the PC was not needed after the first
  minutes. Nothing is waiting on the user's computer.
