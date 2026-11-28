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
  this lane's adapters library, the shell's own crate the client lane's). **T1** to **T5** done
  2026-11-24 to 28, each cut at its start by the user (the epic's M5 task line, the log's entries):
  `h3-map-client` the adapters library, `play` the shell's entry, `hot_seat` the session
  in-process, each human's view in turn, the walks and the boxes through the engine (`reach`,
  `picks`), the screens' notice, the facts audit, T4's game, the covered hand-over and the turn's
  clock (`timeline::hand`, `h3_map_view::clock`, the 14 `seat-*` sequences). **T6** cut in four by
  the user (2026-11-28; a fifth, T6e, at T6a's end). **T6a done** 2026-11-28 (three parts, design
  §5.33): a human's attack played live in play's window (the host's seat and battle AI, the clock
  held: MCL-Q75 (b); `Loop::battle_asked`, the record laid out again after it), the terrain's music
  asked again at the walk's end after a battle (MCL-Q47 read: `Cue::Again`) and a plain cut's
  loops again. **Next:** T6b (the defended and two humans' battles under Wine) or T6e (the guards'
  fights, no Wine), the user's pick; T6c and T6d change `h3-battle-client` in `cross-lane` pull
  requests (the user's consent, 2026-11-28). Open questions: MCL-Q29 to MCL-Q46, MCL-Q48, MCL-Q49,
  MCL-Q58 to MCL-Q74, MCL-Q76 to MCL-Q78 (none blocks). Stubs: `MCL-STUB-1` (a random artifact's
  picture), `-3` (a defended battle, T6c), `-4` (no days without a town on a record, E10), `-5`
  (a computer's turn ended at once, E31's increment 2), `-7` (an M6/M7 screen or a guards' fight:
  its notice, T6e), `-8` (no timed event's box in play), `-9` (a battle no find tells, T6e),
  `-10` (an answer's battle's boxes, T6e), `-11` (the track's kept place at a battle's return,
  MCL-Q78). Next ids **MCL-Q79**, **MCL-STUB-12**.
- **Open pull requests** (2026-11-28): T6a's (`map-client/e24-m5-t6a`; T5's #1223 merged): check it
  first. The review branches `map-client/e24-m3-t7-review`, `-m4-t6-review` are not for merging.
  Merged branches are the user's to delete (the proxy refuses it).
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

- **2026-11-28, E24 M5 T6a** (T6 cut in four by the user at its start, MCL-Q75 (b), the client
  lane's parts this lane's; one runner, a reading sub-agent and three workers in turn): the host's
  battle seat, the window's hosting, the music and loops at the return (the log's three parts);
  T6e cut out at the end (the guards' questions were T4c's notices). The PC up at the start (58
  originals, sums checked; `shell-videos.vid`, the saves, the SoD tables, the manuals); the VM
  restarted once (the baseline lost: CI's green `master` stood in); `master` merged after E09
  T59b1 (#1224, clean).

- **2026-11-27 to 28, E24 M5 T5** (kept whole by the user at its start; one runner, five
  worker sub-agents in turn): the reading, the hot seat recorded, the hand-over built, the clock
  recorded and built (the log's five parts); MCL-Q70 to Q77 asked, MCL-Q47 answered and MCL-Q11
  (c) settled by the user's M4 hand test (parts 3 to 5; the kit's packs and ADPCM samples: TRAPS).
  The PC up at the start (58 originals, sums checked); the session's server restarted four times
  (the VM kept, each worker resumed from its transcript); the disk watched (prune `incremental`
  and `llvm-cov-target` between builds). `master` merged in after E10 T3a (two doc conflicts).

- **2026-11-27, E24 M5 T4c** (kept whole by the user at its start, "use subagents efficiently";
  one runner, five worker sub-agents in turn): a read-only audit of play's facts; the notice
  (`MCL-STUB-7`, 17 keys in `clients/text/map/`, MCL-Q68); the audit's class-A rows (a price paid
  never left the bar: fixed); a wire creature is the packs' handle (Peasants drawn as Boars:
  fixed); T4's game (a random artifact's pickup and a later start's taken piles fixed). Not done:
  play's frames against the original's (no digests on the window's clock: M5's review list).
  Staging: the 58 originals (sums checked), the saves, the SoD tables and the manuals from the PC
  at the start; the VM restarted twice (files kept; a stopped worker resumed from its transcript).
