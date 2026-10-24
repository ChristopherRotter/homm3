# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24 refined 2026-10-23** (T0, with the user): `docs/epics/E24-map-client.md` (decisions, milestones; log `E24-log.md`). **M1, stills of the map view, closed 2026-10-24** (T6b: the reviewers, the review list accepted; `E24-design.md` §1, §2; the crate `clients/crates/h3-map-view`; the capture set `clients/map/situations/`, 40, all exact; a change to the viewer runs `h3map.py drawn` first: the runbook, "Which captures a change needs"). **Next: M2 T0** (`QUEUE.md`: M2, the map moving, refined with the user).
- **T6b's pull request** (`map-client/e24-t6b`, 2026-10-24): check it first (merged, or its mergeability with `master`).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view` from T2), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included: what M3 needs of it moves into a common crate, its inbox has the note).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests (the user, T0), noted in that lane's inbox. Since T3a `write_with(&map, &Extras { tiles, objects })`: `GivenTile`, `AnyObject` (any template but a hero's, the editor's body, an owner); `docs/formats/h3m.md`. It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py` reads its memory; `h3map.py`: `clients/capture/README.md`, "The map's stills").
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no index (M4's need, the adventure lane's inbox); no adventure messages in `h3-protocol` yet.
- **Checkpoints** per milestone (the autonomy contract below), a fresh reviewer agent first (T6b: three in parallel, one per area).

## Autonomy contract (agreed by the user, 2026-10-12, for the client lane; this lane's since the split, 2026-10-23)

- **Agent decides, marked provisional:** the inner structure of the client crates, names, test layout, CLI flags, caching, debug output; asset fixes whose source is a code reading or a reference frame (each an entry with its id). Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency (`png`, an MP3 decoder), the asset tables' and layouts' formats, a visual deviation from the original without a reference. List them in the epic's review list and go on.
- **Ask as an open question** (the user, 2026-10-22): every oddity of the original a reading or a recording finds becomes an `MCL-Q<n>` in the same task (copy, fix with the original's for the comparisons only, or an option; a recommendation); build it as the original's meanwhile.
- **Stop the line:** a change to an engine crate or another lane's paths (CLT-Q1 was one; a change to the client lane's paths, `h3-view` included), shared paths (CI, `lanes.toml`: a `shared/` PR), heavy dependencies (the window/audio library, a GPU stack, a UI framework, a video codec), licence questions (reading MMArchive's, homm3-unpacker's or DDrawCompat's code included), committing anything taken from the original beyond identifiers and numbers (ADR-0009), anything that would write into the installation. Write the question (`MCL-Q<n>`), stop that task and take the next independent one.

## Standing traps

Moved to `TRAPS.md`: CI out, the gate's and `h3map.py`'s staged files, a template's class in memory, `STALE` situations, one game at a time; the client lane's traps apply to the foundation and the harness (`docs/lanes/client/TRAPS.md`).

## Asset notes

The user's dictation of 2026-10-12 is in `docs/lanes/client/ASSET-NOTES.md`; this lane's research items (the object draw order, the objects' shadows) in `ASSET-NOTES.md` here.

## Recent sessions

- **2026-10-24, E24 M1 T6b** (`map-client/e24-t6b`): M1's closure; three reviewers in parallel, 35 of 37 findings fixed (the fallen objects' lists, the harness's checks, `drawn`'s `STALE`), 2 queued; the set (40) again, exact; the review list accepted; M2 T0 queued.
- **2026-10-24, E24 M1 T6a** (#456, merged): the half tick (the rivers a step behind for a whole game, §1.6; MCL-Q5 answered (a)); `h3map.py drawn`, the kept live parts; the set (40) captured.
- **2026-10-24, E24 M1 T5b2** (#453, merged): the path's arrows (§2.10), the explored tiles, boarded boats' colours; five exact.
