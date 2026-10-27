# Client lane — standing traps

Moved out of the lane's HANDOFF on 2026-10-24 so that session start stays short; verbatim, grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc it belongs to, a `shared/` change).

- **CI may be out** (billing, since 2026-10-12): `clients/scripts/check.sh` stands in, its result in the pull request.
- A new dependency of a pure crate goes into `h3-view/tests/purity.rs`'s allowlist deliberately; a window or audio crate belongs in an adapter crate, not in `h3-view`'s pure library.
- Sound in a sandbox: `SDL_AUDIO_DRIVER=disk` (the mix in `./sdlaudio.raw`).
- `winit`, `cpal`, `hound` (and Bevy, eframe, `rodio`) are Apache-2.0-only (`deny.toml`, ADR-0009): the window is SDL 3 (`h3-sdl`; building it: CMake, a C compiler, `clients.yml`'s apt list). Windows: `x86_64-pc-windows-gnu`, `mingw-w64`, the user's `clients/out/win/`.
- `clients/Cargo.lock` is a common path: the verify's `clients-lock` gate (#202) fails when an engine crate's dependencies change it; fix: `cargo update --workspace --manifest-path clients/Cargo.toml`.
- `ci.yml` ignores `clients/**`; `clients.yml` watches `clients/` and the engine crates the client builds (`check.sh`'s "ci paths" keeps that list complete).
- Never run the original from the installation (it writes into its folder): captures run the copy under `original-data/wine/`, the Wine prefix in a directory the shell's user owns; captures never enter git (ADR-0009).
- The original's memory holds the frame drawn, the screen the one shown (design §7.1, §7.9): never render memory's poses as the screen (a cast's first frame: `before`). More in the runbook's "Traps".
- No float arithmetic or platform transcendental in the client's drawing and timing, the binary included (`clippy.toml`): f32 behaviour in fixed point (`anim::f32_round`), tangents by series; SDL's floats only at the adapter's edge (coordinates, `audio::gain`).
- The harness's `actions` are the viewer's mouse (`pointer::inputs`, design §9.10): a new action
  kind needs both; `hex_point` and `PARK` mirror `h3drive.py`'s; a `key:` right after a `cast:`
  acts at the cast's end in both (T7b-1).
- A press's command is `h3_view::mouse::click`'s, by the cursor's kind (E17 M5 T2a): a new kind
  or a changed one needs its press there too; `h3-view`'s `clicks` test checks the presses
  against the engine's legal lists over every scenario's turns (CLT-Q25's walk the one gap).
  Input made while the battle does not take it is dropped at the next turn's start, not taken
  then (design §9.11; the tracker had it queued until T2a).
- Tell spells by `SpellLook::original`, not engine handles (Poison 70). After a hero's cast a
  stack acts by `key:` only (runbook).
- Sequences are spectators' records; the engine rolls its own (a box ` n…` or the log ` log`
  listed; a seed searched to match: runbook, "Forced rolls"). Records: army, then war machines.
- A new list of the timeline that `scene::Moment::at` reads needs its reader in
  `anim::Playhead` too (E17 M5 T1a; the frames are drawn through it): the tests check the two at
  every change of every replay (`anim/tests/playhead.rs`), `h3-view sequences` at every recorded
  sequence's.
- The timeline grows a command at a time (E17 M5 T1b, `anim::Growing`): `Player` may read back
  only what the command under way drew (its lists are handed over after each), and a later
  command adds only from the last one's end (the drawbridge's open span the one exception), or
  the playhead and `play`'s frames stop matching; the tests grow every replay and check it.
- A battle fought as it is played (`play`, the window: `live.rs`) has no events when its stage
  is read: a stage lookup that depends on the record's events (`motion::magic`, `motion::casts`)
  needs its `every` branch. A frame at or past a playing timeline's end is drawn only after the
  next command (`Battle::grow`).
- The viewer reads `clients/layouts/*.toml` at run time: a layout change made while a batch
  runs breaks (or changes) the release binary's comparisons of the batch (M4 T3b-1: an unknown
  field); keep the layout with the binary it was built for until the batch ends.
- A capture times out at the main menu when the CPU is busy: run it alone. A fresh sandbox's
  first capture once differed (T7c-2): recapture. A background job stops at 2 h: batches.
- Sieges: machines acting on their own at the opening act before the record opens (CLT-Q20: the
  defender commands its turrets); the turrets' counts need `structures = 1`; in hot seat the
  original shows the side to act's view (runbook, "A siege").
