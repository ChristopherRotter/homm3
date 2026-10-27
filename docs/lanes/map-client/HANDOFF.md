# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** (stills) closed 2026-10-24 (the crate `clients/crates/h3-map-view`; the capture set `clients/map/situations/`, 40, all exact; a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"). **M2** (the map moving; refined with the user 2026-10-26, tasks in `QUEUE.md`): T1a1 to T4a done 2026-10-26/27 (`h3map.py frames`, `clients/map/sequences.toml` and `timelines/`, `h3_map_view::timeline`, `h3-map-view sequences`, the frame check: all 13 sequences every line, their 450 frames exact; `h3-map-view window` replaying a sequence or a situation, `h3_map_view::keys` and `replay`; design §3.1 to §3.9). **Open: MCL-Q7** (the lifted hero cut at the view's tiles during a step at the map's side; built as the original's). **Review list:** M2's starts in the epic (T4a's key table, its window keys, the replay's cursor). **Next: M2 T4b** (the map's live input, the Windows build tried on the user's machine: it needs the link to the user's computer, or the user runs the `.cmd`).
- **M2 T4a's pull request** (`map-client/e24-m2-t4a`, #499, 2026-10-27): check it first (merged, or its mergeability with `master`).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included: M2 copies its `clock` and `cursor`, M3 T0 proposes the common crate).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests (the user, T0), noted in that lane's inbox; its additions (T3a's `write_with`, `Extras`): `docs/formats/h3m.md`. It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no index (M4's need, the adventure lane's inbox); no adventure messages in `h3-protocol` yet.
- **Checkpoints** per milestone (the autonomy contract below), a fresh reviewer agent first (T6b: three in parallel, one per area).

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

- **2026-10-27, E24 M2 T4a** (#499): T4 cut by the user (T4a the window replaying and the key table, T4b live input and Windows); `h3_map_view::keys` (F5 pause, F6 frame, F7 restart, F8 vsync, F11 fullscreen, Esc; the original's map keys bound; a user's `keys.toml`), `replay`, `clock` copied; `paint` shared by `sequences` and `window`; `h3-sdl`'s `Key` grown additively; Xvfb: every frame read back exact (walk, situations, scale 2); no `MCL-Q`. The link to the user's computer dropped after the staging (nothing lost).
- **2026-10-27, E24 M2 T3b** (#498): taken whole; T1b's seven walks recorded again (as recorded); `draw_frame_blend` read; the scene's lifted hero (passes 5, 7, 9 blended) and the view's offset; the late redraw drawn again (one objects' step a redraw); the gate's walking groups; `sequences`, the frame check and `drawn` over the walks: 13 sequences every line, 269 walk frames exact; MCL-Q7 asked.
- **2026-10-27, E24 M2 T3a** (#495, merged): T3 cut by the user (timing, then drawing); the walk in the timeline (a key's step, a path shown and walked, the lifted hero, the view centred and slid, the frames, the arrows, the cursor hidden, the actions during a walk), the tables' `[map.walk]`, `sequences` plays the walks; T1b's seven exact line by line but the screens, no recording; `walk-follow`'s scroll speed given; no `MCL-Q`.
