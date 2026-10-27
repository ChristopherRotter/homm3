# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** (stills) closed 2026-10-24 (the crate `clients/crates/h3-map-view`; the capture set `clients/map/situations/`, 40, all exact; a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"). **M2** (the map moving; tasks in `QUEUE.md`, the built parts in the epic's M2 task list): T1a1 to T6a2 done 2026-10-26/27 (the recordings `h3map.py frames`, 13 sequences, every line and their 450 frames exact; `h3_map_view::timeline`, `replay`, `keys`, `sound`, `music`; `h3-map-view sequences` and `window`, live input, the map's sound, the Windows build; design §3.1 to §3.12). **Open: MCL-Q7, Q8, Q9, Q12** (all built as the original's). **Review list:** M2's in the epic (T4a to T6a2). **Next: M2 T6b** (M2's closure: the sequences recorded again, the window's T6a fixes and T6a2's music tried on the user's machine, the review list to the user); the inbox's frame cost (4 to 6 ms a frame) when the drawing is next touched, the sound without the focus with the next sound task.
- **M2 T6a2's pull request** (`map-client/e24-m2-t6a2`, 2026-10-27): check it first (merged, or its mergeability with `master`); #517 (T6a) merged.
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

- **2026-10-27, E24 M2 T6a2** (whole): the music thread read (`play_music`, `music_thread`, `music_stop_fade`; the reviewer's five confirmed, R2-4 an oddity: MCL-Q12); `h3_map_view::music` (the original's state on the frames heard, and a queue at the frames heard and queued) and the window's music rebuilt on it; checked in the window with the `disk` driver (Dirt kept after a quick back-step, a request during a fade-in waiting for it). Inboxes: the battle's end tune (client lane), the sound without the focus (here).
- **2026-10-27, E24 M2 T6a** (T6 cut by the user: T6a the reviewers and their fixes, T6a2 the music player, T6b the closure): three reviewers in parallel, 8, 11 and 10 findings, each checked against the code and the executable; 24 fixed with tests (a path's walk and the input given during it as `hero_move_loop` takes it; the window without the focus, fullscreen's bars, an empty track, a blocking decode; the harness's verdicts, cleanup and checks), the music player's five cut out as T6a2; MCL-Q11 answered. The harness's changes not run yet (no game staged): T6b's recordings run them.
- **2026-10-27, E24 M2 T4b3** (#512): the drawers by rows, 31 times fewer instructions a view, every frame the same; on the user's machine a frame 4 to 6 ms, none passed over; MCL-Q11 asked. The frame's fixed cost to check again (the inbox).
