# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** and **M2** closed (`E24-archive.md`): the crate `clients/crates/h3-map-view`, 50 situations and 22 sequences exact (a viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs"; `--bugs original` the comparisons' profile). **M3** (the adventure screen; the epic's M3 section, its review list; the queue's tasks): **T1a** (#545) to **T6a1** (#595) merged; T6a2 cut with the user (2026-10-30) into **T6a2a built** (the 35 classes of the visit marks; design §4.11; four sequences exact with the status bar compared; MCL-Q18's fix) and **T6a2b** (the 20 of the owners and contents). **Next: T6a2b or T6b** (independent; the user picks), then T7 (M3's closure, now with the status bar unmasked and everything recorded again). Open: **MCL-Q13** (keep, recommended), **MCL-Q14** (M4 records a cast first), **MCL-Q15** (the original's, recommended).
- **Open pull requests** (2026-10-30): T6a2a's (`map-client/e24-m3-t6a2a`): check it first (merged, or its mergeability with `master`). A merged branch is the user's to delete: the session's proxy refuses it.
- **Ownership** (`lanes.toml`): this lane's crates in the client workspace (`clients/crates/h3-map…`: `h3-map-view`), its data folder `clients/map/`, `docs/epics/E24*` and this folder; branches `map-client/<task>`; ids `MCL-Q<n>`, `MCL-QUIRK-<n>` (asset fixes), `MCL-STUB-<n>`; ADR drafts in `adr-drafts/`. **Common** with the client lane: the foundation (`h3-assets`, `h3-render`, `h3-sdl`, `h3-ui`, the workspace's `Cargo.toml`, `deny.toml`, `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, kept common by the user at T0) and the capture harness `clients/capture/` (the adventure lane changes it too): an additive change freely, a change in behaviour noted in the client lane's inbox. The rest of `clients/` is the client lane's (`h3-view` included).
- **The writer:** this lane adds what its situations need to `h3_map::writer` (the adventure lane's) in additive `cross-lane` pull requests, noted in that lane's inbox (`docs/formats/h3m.md`). It refuses an owner that is no player: a situation gives each owning colour a hero.
- **From E17** (`clients/README.md`; ADR-0020, ADR-0024): the decoders, the compositor, the SDL 3 adapter, the string catalogue, the gate, the harness (the original under Wine, `h3mem.py`, `h3map.py`: the runbook `clients/capture/README.md`).
- **The engine side:** `h3_map::parse` gives the raw map with every picture (`h3_map_view::art` reads it); the import keeps none, its `ObjectView`s have no index (M4's need, the adventure lane's inbox); no adventure messages in `h3-protocol` yet.
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

- **2026-10-30, E24 M3 T6a2a** (the user present: T6a2 cut into T6a2a, the visit marks, and
  T6a2b; MCL-Q17 to MCL-Q19 answered): T6a2a built (the log's "M3 T6a2a"); the original data
  staged from the user's repo folder first; the sandbox restarted during the setup, the baseline
  run again and green. Nothing is waiting on the user's computer.
- **2026-10-30, E24 M3 T6a1** (the user present: T6 cut into T6a and T6b, T6a into T6a1 and
  T6a2): T6a1 built (the log's "M3 T6a1"; #595); the original data staged from the user's repo folder
  first; the sandbox restarted during the setup, its baseline stopped: CI's green `master` the
  starting point. Nothing is waiting on the user's computer.
- **2026-10-29, E24 M3 T5b2** (the user present: T5b2 kept whole; MCL-Q16 answered): T5b2
  built (the log's "M3 T5b2"; #588); the original data staged from the user's repo folder first; the
  sandbox restarted during the setup, its baseline stopped: CI's green `master` the starting
  point. Nothing is waiting on the user's computer.
