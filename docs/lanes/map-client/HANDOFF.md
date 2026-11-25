# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** to
  **M4** closed and archived with their review lists (`E24-archive.md`; M4's accepted at M5 T0);
  the crate `clients/crates/h3-map-view`; 56 situations and every sequence and scripted game as
  recorded (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change
  needs"; `--bugs original` the comparisons' profile); the 66 golden player records play to
  their end. **M5 refined 2026-11-24 at T0** (hot seat on the adventure map; M6 the hero's
  screens, M7 the town screen, M8 live play sketched): the user's rules there bind every task,
  above all **the engine and the server the only authority** (no rules table or map text read in
  play; the pictures from the map file until an ADR) and **the shell on top** (`h3-map-client`
  this lane's adapters library, the shell's own crate the client lane's). **M5 T1 cut in three**
  by the user at its start: **T1a done** 2026-11-24 (the hero on its tile through a battle, the
  panel after a retreat: design §5.22), **T1b done** 2026-11-25 (MCL-Q51: `battle-mine-retreat`
  recorded, `visit_mine`'s path read for a draw, the built form kept: design §5.23). **Next: M5
  T1c** (MCL-Q52), then T2 (the user's M4 hand-test notes first if written:
  `clients/out/win/map-m4-notes.txt`, still blank at T1b).
  Open questions: MCL-Q29 to MCL-Q49 (none blocks). Stubs: `MCL-STUB-1` (a
  random artifact's picture), `MCL-STUB-3` (a defended battle, M5 T6), `MCL-STUB-4` (no days
  without a town on a record, E10); planned `MCL-STUB-5` (T3), `MCL-STUB-6` (T4). Next ids
  **MCL-Q57**, **MCL-STUB-5**.
- **Open pull requests** (2026-11-25): T1b's (`map-client/e24-m5-t1b`): check it first.
  The review branches `map-client/e24-m3-t7-review`, `-m4-t6-review` are not for merging. Merged
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

- **2026-11-25, E24 M5 T1b** (MCL-Q51): `battle-mine-retreat` recorded twice the same (no box,
  the mine unflagged, the hero gone as `battle-retreat`'s); `visit_mine`, `monster_battle`'s end
  and `adventure_battle`'s outcomes read: a loss, a retreat and a draw show the same, so M4 T6's
  built form stays (the log's "M5 T1b", design §5.23); a draw's zero-count guards noted in the
  adventure lane's inbox. The PC up for the first minutes (everything staged, SHA-256 checked);
  the setup's baseline stopped at coverage by hand, CI's `master` standing in.

- **2026-11-24, E24 M5 T1a** (T1 cut in three by the user at its start): M4's "moving pose" was
  the hero drawn on its tile (`adventure_hero_stop` at every battle's start: no pass 9 blend),
  the map's return with it there and the mouse hidden, lifted again at the fade in's end; sleep
  and cast kept after a retreat; the three battle sequences recorded again, their frame lists
  M4's known differences only (the log's "M5 T1a"). The PC up at the start (58 files and the
  manuals staged, SHA-256 checked); the VM restarted once (the setup's baseline lost, CI's
  `master` standing in); a disk ballast for the game's disk check.

- **2026-11-24, E24 M5 T0** (the user: all of T0 at once, the hand test not run yet): M4's
  review list accepted (six items asked one by one, 39 in a summary), MCL-Q50 to Q56 answered
  (Q54: the engine and the server the only authority; facts first, the pictures later), M5
  refined and T1 to T8 queued, the other lanes' inboxes told. Docs only; the PC's bridge came and
  went (the executable, the text tables and the manuals staged at the start); the VM restarted
  once.
