# Map client lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E24** (`docs/epics/E24-map-client.md`; log `E24-log.md`, design `E24-design.md`): **M1** to
  **M4** closed and archived with their review lists (`E24-archive.md`); the crate
  `clients/crates/h3-map-view`; 56 situations and every sequence and scripted game as recorded (a
  viewer change runs `h3map.py drawn` first: the runbook, "Which captures a change needs";
  `--bugs original` the comparisons' profile). **M4 closed 2026-11-23 at T6** (the user: no cut;
  six reviewers, 64 findings, seven fixers: the log's "M4 T6"): every golden replay's player
  record (66) plays headless to its end, in the window with its battles. **The user's checkpoint
  is open:** M4's review list (its T6 additions by fixer at the archive's end): the user reads it
  first, asked again at M5 T0; the questions **MCL-Q50** to **MCL-Q56**: answered one by one at
  M5 T0 (`MCL-STUB-4` kept by the user); the hand test's kit in the user's
  `clients/out/win/` (`map-m4.cmd`, notes in `map-m4-notes.txt`). **Next: M5 T0** (M5 refined with
  the user: QUEUE), after those answers. Open questions: MCL-Q29 to MCL-Q56 (none blocks). Stubs:
  `MCL-STUB-1` (a random artifact's picture), `MCL-STUB-3` (a defended battle, M5),
  `MCL-STUB-4` (no days without a town on a record, E10). Next ids **MCL-Q57**, **MCL-STUB-5**.
- **Open pull requests** (2026-11-23): T6's (`map-client/e24-m4-t6`) and the README's row
  (`shared/e24-m4-closure`, merge first): check both first. The review branch
  `map-client/e24-m4-t6-review` (the six reviewers' findings) is not for merging. Merged branches
  are the user's to delete (the proxy refuses it).
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

- **2026-11-23, E24 M4 T6, M4's closure** (the user: no cut, the work spread over sub-agents):
  six fresh reviewers (64 findings), seven fixers one at a time (each finding a test or a
  question), the exit checked (`drawn` all the same, the two shipped starts captured again with
  the Gold bonus, `battle-retreat`'s three frames now exact, 66 golden records to their end, the
  window under Xvfb, `check.sh`, the coverage, the scoped verify); M4 archived; the hand-test kit
  on the user's machine. The PC's shell worked this time; the session's server restarted once
  mid-fix (the fixer resumed). Disk: the coverage build is 6 GB; delete it after reading.

- **2026-11-21, E24 M4 T5b2** (kept whole by the user at its start): the switch's four fades (the
  host's, the battle's frames from its element), the result (a monster taken, a retreated hero
  gone and the next chosen, a mine's boxes and flag); T5a's three as scripted games, recorded
  again twice the same, every timeline as recorded (the log's "M4 T5b2"). Original data staged
  (the 64 files and the manuals; the PC's shell down, so sizes checked and the LODs and
  executables against `installation.md`'s hashes); the PC not needed after the first minutes.

- **2026-11-21, E24 M4 T5b1** (T5b cut in two by the user at its start, the hosting first; the
  crate structure the user's: a dependency on `h3-battle-client`): a walk's and an answer's
  battles handed over (`Loop::battles`, `Directed::battles`), hosted in the map's window by the
  binary's `battles`, the map's sound on `HostSound`; 12 battles of 10 golden replay records
  handed over, two viewed in the window under Xvfb, every map frame exact (the log's "M4 T5b1").
  Original data staged (the 64 files and the manuals; the PC's shell was down again, so sizes
  checked and the LODs and executables against `installation.md`'s hashes).
