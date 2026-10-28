# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** (stills) and **M2** (the map moving) closed (`E24-archive.md`): the crate `clients/crates/h3-map-view`, 40 situations and 13 sequences exact (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"; `--bugs original` the comparisons' profile). **M3** (the adventure screen) refined 2026-10-28 at T0 with the user (the epic's M3 section: scope, decisions, exit; the queue's T1b to T7). **T1a built** 2026-10-28: the common crate `clients/crates/h3-ui` (`clients/README.md`). **Next: M3 T1b** (the battle window's keys through `h3_ui::keys`) once T1a's pull request merges. No open question.
- **Open pull requests** (2026-10-28): T1a's move (`map-client/e24-m3-t1a`, `cross-lane`; #541, its `lanes.toml`, merged by the user): check it first (merged, or its mergeability with `master`).
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests (the user, T0), noted in that lane's inbox; its additions (T3a's `write_with`, `Extras`): `docs/formats/h3m.md`. It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no index (M4's need, the adventure lane's inbox); no adventure messages in `h3-protocol` yet.
- **Checkpoints** per milestone (the autonomy contract below), a fresh reviewer agent first (M1 and M2: three in parallel, one per area), then the user's review list and, where the window changed, a hand test on the user's machine (`clients/out/win/`, a `.cmd` with the checks on screen and a notes file).

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

- **2026-10-28, E24 M3 T1a** (the user present at the start): the original data staged (with the whole `MP3/` folder and the harness's DLLs); #541 (`lanes.toml`) merged by the user; the move in one pull request (the user's choice): `h3-ui` built (the log's "M3 T1a"); check.sh green, coverage 100 %; both `drawn` all the same. Next: T1a's pull request merged, then T1b.
- **2026-10-28, E24 M3 T0** (the user present): M3 refined with the user (the common crate `h3-ui`, T1a to T7 queued); MCL-Q12's provisional part confirmed; MCL-Q9 answered and built.
- **2026-10-28, E24 M2 T6b2c** (the user present): the hand test with `map-try.cmd` passed; M2 archived and closed.
