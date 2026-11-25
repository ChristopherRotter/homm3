# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** to
  **M4** closed and archived with their review lists (`E24-archive.md`; M4's accepted at M5 T0);
  the crates `clients/crates/h3-map-view` (pure) and `h3-map-client` (its adapters and the
  `h3-map-view` binary, since M5 T2a); 56 situations and every sequence and scripted game as
  recorded (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change
  needs"; `--bugs original` the comparisons' profile); the 66 golden player records play to
  their end. **M5 refined 2026-11-24 at T0** (hot seat on the adventure map; M6 the hero's
  screens, M7 the town screen, M8 live play sketched): the user's rules there bind every task,
  above all **the engine and the server the only authority** (no rules table or map text read in
  play; the pictures from the map file until an ADR) and **the shell on top** (`h3-map-client`
  this lane's adapters library, the shell's own crate the client lane's). **M5 T1 cut in three**
  by the user at its start: **T1a done** 2026-11-24 (the hero on its tile through a battle, the
  panel after a retreat: design §5.22), **T1b done** 2026-11-25 (MCL-Q51: `battle-mine-retreat`
  recorded, `visit_mine`'s path read for a draw, the built form kept: design §5.23), **T1c done**
  2026-11-25 (MCL-Q52: the loops restarted at a won battle's lift, as read and sound-recorded:
  design §5.24; a plain cut's battle in T6's scope). **M5 T2 cut in three** by the user at its
  start: **T2a done** 2026-11-25 (`h3-map-client`: the binary's adapters a library, the binary a
  thin host, nothing changed), **T2b done** 2026-11-25 (`h3_map_client::play`, play's entry for
  the shell, its API agreed by the user for the client lane and in that lane's inbox; the
  window's loop on a lent window and sound; until T3 the first human's start shown standing;
  `h3-map-view play` the command line start), **T2c done** 2026-11-25 (MCL-Q57: one record
  reader, `h3_protocol::record` beside the server's writer, the user's place; `h3-view`'s and
  `h3-map-view`'s readers on it, their types and readings kept). **Next: M5 T3** (the session
  in-process). The M4 hand-test notes (`clients/out/win/map-m4-notes.txt`) still blank at T2c.
  Open questions: MCL-Q29 to MCL-Q49 (none blocks). Stubs: `MCL-STUB-1` (a
  random artifact's picture), `MCL-STUB-3` (a defended battle, M5 T6), `MCL-STUB-4` (no days
  without a town on a record, E10); planned `MCL-STUB-5` (T3), `MCL-STUB-6` (T4). Next ids
  **MCL-Q58**, **MCL-STUB-5**.
- **Open pull requests** (2026-11-25): T2c's #1182 (`map-client/e24-m5-t2c`, `cross-lane`: the
  server lane's `h3-protocol` and the client lane's `h3-view`, agreed by the user; T2b's #1175
  merged): check it first.
  The review branches `map-client/e24-m3-t7-review`, `-m4-t6-review` are not for merging. Merged
  branches are the user's to delete (the proxy refuses it).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`, `h3-map-client`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
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

- **2026-11-25, E24 M5 T2c** (MCL-Q57: the place and both records' readers, the user at the
  start): `h3_protocol::record` (`read_battle`, `read_game`, `is_game`, `RecordError`,
  `MAX_LINE` 64 MB) with its tests; `h3_view::record::parse`, `game_record::parse` and
  `h3_map_view::engine::parse` thin wrappers on it; notes in the client and server lanes'
  inboxes. The PC up at the start only (58 files staged, SHA-256 checked against T2b's sums).

- **2026-11-25, E24 M5 T2b** (one PR, the API agreed for the client lane, the engine's start
  until T3: the user at the start): `h3_map_client::play` (`Game`, `Folders`, `End`, `play`);
  `window`'s loop on a lent window and sound (`Source`, `Host`; the map's streams added on the
  host's `HostSound` and removed again); `paint::for_text` (a record kept in memory);
  `h3-map-view play`. The PC up at the start only (58 files staged, SHA-256 checked; a stray
  `.keep` of a `git fetch` left in the user's `.git`, harmless).

- **2026-11-25, E24 M5 T2a** (T2 cut in three by the user at its start): `h3-map-view`'s binary
  moved by `git mv` into the new library `h3-map-client`, the binary a thin host in its package;
  nothing changed: the 236 golden player records' timelines those of `master`'s binary, `drawn`
  all the same, the window under Xvfb exact. The PC up at the start only (67 files staged,
  SHA-256 checked); the VM restarted once (the setup's baseline lost, CI's `master` standing in).
