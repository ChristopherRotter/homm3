# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1**, **M2**
  and **M3** closed (`E24-archive.md`); the crate `clients/crates/h3-map-view`; 56 situations and 106
  sequences (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change
  needs"; `--bugs original` the comparisons' profile). **M4** refined (2026-10-31): Part A done
  (2026-11-19); T3 done in its cuts (T3a, T3b1, T3b2: a record's start, design §5.9 to §5.11);
  T4 cut in three by the user (T4a the viewer's own turns, T4b the visits, T4c the other players,
  the controls and a golden replay): **T4a1 done** (design §5.12), the labels task done
  (ADR-0009), **T4a2 done** (2026-11-20; design §5.13: the scripted games played from their
  engine records, `sequences <id> --record`, the gaps in `timelines/<id>-gaps.txt`); T4b cut in
  two by the user (2026-11-20, by family; Summon Boat out of T4b): **T4b1 done**, **T4b2 done**
  (design §5.14, §5.15); **T4c done** (2026-11-20, kept whole by the user: the other players'
  turns from a record, the record player `director`, `record` and `window --record` with the
  playback keys, ten golden replays viewed to their end; design §5.16, §5.17); **T4c2 done**
  (2026-11-21, whole: a computer's boarding and landing in sight, design §5.18; `game-ai-boat` in
  `sequences.toml`, exact but MCL-Q43's and MCL-Q46's gaps); T5 cut in two by the user
  (2026-11-21, the reading first): **T5a done** (design §5.19: the switch into a battle and back
  read, the harness's battle step, `battle-won`, `battle-mine` and `battle-retreat` recorded in
  `map/to-record.toml`; MCL-Q47); T5b cut in two by the user (2026-11-21, the hosting first; the
  crate: `h3-map-view`'s binary depends on `h3-battle-client`): **T5b1 done** (2026-11-21, design
  §5.20: a record's battles hosted in the map's window, the map's sound on `HostSound`;
  `MCL-STUB-3`, a defending battle). **Next: T5b2** (the switch and the result as the original's,
  T5a's three compared: QUEUE), then T6. The inbox: the
  viewer's tests' catalogues hold a few of the original's sentences (the user's to decide; offered
  at T4c2's start, no answer yet). Open questions: **MCL-Q29** to **MCL-Q47** (none blocks;
  MCL-Q36, Q37, Q39 to Q44, Q46: facts not in the record, (a) recommended, the engine lanes'
  inboxes told; MCL-Q45 (b) built; MCL-Q46 (c) built; MCL-Q47 (a) built at T5b1, by ear at T6).
  Next ids **MCL-Q48**, **MCL-STUB-4**.
- **Open pull requests** (2026-11-21): T5b1's (`map-client/e24-m4-t5b1`): check it first. Merged
  branches are the user's to delete (the proxy refuses it).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests, noted in that lane's inbox (`docs/formats/h3m.md`). It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none; a view's objects come in tile order without an index (found in the file by type and anchor, `h3_map_view::engine`); `h3-server record` writes a whole game's record per player (protocol v76, record format 5); every shipped map imports (E09).
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

- **2026-11-21, E24 M4 T5b1** (T5b cut in two by the user at its start, the hosting first; the
  crate structure the user's: a dependency on `h3-battle-client`): a walk's and an answer's
  battles handed over (`Loop::battles`, `Directed::battles`), hosted in the map's window by the
  binary's `battles`, the map's sound on `HostSound`; 12 battles of 10 golden replay records
  handed over, two viewed in the window under Xvfb, every map frame exact (the log's "M4 T5b1").
  Original data staged (the 64 files and the manuals; the PC's shell was down again, so sizes
  checked and the LODs and executables against `installation.md`'s hashes).

- **2026-11-21, E24 M4 T5a** (T5 cut in two by the user at its start, the reading first): the
  client lane's entry checked complete; the switch into a battle and back read (design §5.19);
  the harness's sequences play a battle (`battle`, common, additive); three battle sequences
  recorded twice the same in `map/to-record.toml` (the log's "M4 T5a"); MCL-Q47. Original data
  staged (the 64 files and the manuals; the PC's shell was down, so sizes checked and the LODs and
  executables against `installation.md`'s hashes); the PC was not needed after the first minutes.

- **2026-11-21, E24 M4 T4c2** (offered whole at the start, no seam): `game-ai-boat` recorded
  again twice the same, a computer's boarding and landing in sight built (the same class switch
  as a human's; `hero_boards_boat` draws nothing for a computer), the move's last step's draw and
  a turn's end's single objects' step; day 3's pickup without a step not in the record (MCL-Q46,
  (c) built; the adventure lane's inbox). The log's "M4 T4c2". Original data staged (the 64 files
  and the manuals), SHA-256 checked; the PC was not needed after the first minutes.
