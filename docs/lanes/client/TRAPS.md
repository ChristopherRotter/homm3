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
- A battle a human plays in the window (`--human`, E17 M5 T2b) grows from every side's events
  (`Live`, `Growing::push_seen`), not the spectator's: an event one side alone sees (a hidden
  obstacle's placement or removal, a Faerie Dragon's `SpellDrawn`) reaches `anim`'s player there;
  an object's sight is `Object::owner` with `Timeline::screen`. While it waits for a human the
  timeline shown is `Growing::waiting`'s copy, so a command starts a fresh playhead
  (`Battle::command`); presses and the battle's keys count only while it waits, at or past the
  timeline's end (`Battle::takes`).
- The spell book (E17 M5 T3a) is a modal dialog: the battle behind it stands still, so the
  window's `next_frame` returns at once while it is open (scanning the idle steps for a change
  never ends: the window hung). Its hero comes from the side to act's own observation
  (`Live::own_view`): the turn's observation (`Battle::turned`) is the spectator's, without
  `own_hero`. Casts are not in the engine's legal list: ask `h3_battle::cast_targets`
  (`Live::casts_untargeted`). A situation's `actions` run at the frozen clock: after a `move:` the
  original's cursor stays where it was until the clock steps (compare from 100 ms).
- A spell's aim (E17 M5 T3b) is a modal loop of its own: while `Battle::aiming` the window
  takes only moves, a press (the cast at the hex the last move's cursor took, `Shown::aimed`),
  Escape and the right button (`Key::RightButton`), the viewer's own keys still; the battle
  idles behind it. The book takes the mouse's moves as `Input::Book` to `Aim`, `Act` or `Cancel`
  (the battle takes none meanwhile), so a sequence's `cast:`/`aim:` start with `Book`. The aims'
  targets are the engine's, computed where the battle is (`scenario::Turned::of`, `Live`): a
  turn's `aims` hold only the spells asked (the window adds the book's choice as it closes).
- Auto Combat (E17 M5 T4c1) is the inputs' (`Input::Auto`, `Input::Stop`; `h3_view::auto`): the
  battle reads them before it decides a turn, so they go through `Battle::switch_auto` and
  `stop_auto` (which update its own inputs at once), never straight into the window's list. The
  turn after a command is decided as it starts (`Battle::play_next`), not when the command before
  it is taken: a stop pressed during that command's action makes a human's turn wait, its outline
  put back (`Growing::outline_waiting`). While it is on, every key but the window's own and every
  press is a stop first (the window's event loop).
- A message box (E17 M5 T4a) is modal as the book: while `Battle::message_open` the window's
  `next_frame` returns at once (else it hangs scanning idle steps), every key but the viewer's
  own goes to the box (Escape is Cancel's, never the window's quit) and the mouse's moves reach
  neither the book behind it nor the battle; the battle stands as it was when the box (or the
  book under it) opened. In a still the harness's park move after a `click:` that opened a box
  goes to the box: the book keeps its hint (`msg-mana`).
- The result window (E17 M5 T4b1) is a message box (`Asked::Result`): modal, its OK (Enter,
  Escape) ends the window's run, unattended runs leave it open (the presser skips it). Behind it
  the battle stands at its end with the heroes' flags of the moment before (`result::behind`):
  frozen 1 ms earlier, the end's last action is lost. Since T4b2b the harness's `VIDEO.VID` is
  the user's `battle-videos.vid` (`setup.sh`; an empty one without it: no clip), so the original
  plays the window's clip.
- The result window's clip (E17 M5 T4b2b, design §7.21): `BINKW32.DLL` reads `timeGetTime`
  through its own import, which `xdd.dll` redirects too (the ack's eighth field 1): an `xdd.dll`
  without it runs the clip in real time under the frozen clock. The DLL counts every decode, a
  redraw's too: two before the first tick put the original's clip two frames late (CLT-Q31,
  `Bugs::Original` only). `Draw::Video` carries the time since the window opened, so the draw
  list changes every millisecond: the window draws the clip at its decodes (`Battle::clip_times`),
  never by comparing lists. Its pixels differ from the original's only where `h3-bink`'s integer
  DCT rounds otherwise than the DLL's float one (CLT-Q30: five clips, listed); the colours are
  exact. The clips come from the video archives entry by entry (`videos.rs`; an installation
  copy without `VIDEO.VID` takes `original-data/battle-videos.vid`), never from the LODs. A
  new kind of asset in the batch needs its line in `battle.rs`'s `held`, or the window decodes
  it again at every frame (the clip did: 119 ms late a frame).
- The tactics phase (E17 M5 T4c2a, design §7.23): its shade is the viewers' (`Moment::shade`,
  which the timeline leaves empty): a viewer that draws a human's turn in the phase sets it from
  that turn's observation (`h3_view::tactics::shade`; the stills the opening's, the window the
  waiting turn's). Its message opens before the first turn: behind it the battle is
  `Moment::before_first_turn` (no outline, the flags one idle step back) and the panel
  `Panels::before` (as built, the defender's colours); the panel in the phase is the layout's
  `[tactics_panel]` (`BattleLayout::panel_in`), which has no log's item: the status line and the
  log's text go nowhere. Played (E17 M5 T4c2b, §7.24): the panel shown is the phase's until
  `EndTactics`, so the window finds a button through `Battle::panel_at`, never
  `layout().panel`; its Space and Enter are `keys::Context::Tactics`, asked before the battle's
  (the table refuses one chord twice in a context); a record's shade over time is
  `tactics::shade_at` (the log's `Action` and `Turn` entries, the turns' battles from
  `Panels::turns`); a placement's walk comes hex by hex in the engine's events (a recording
  sink), and `EndTactics` draws no frame.
- `h3_formats`' `c_string` takes printable ASCII alone: an executable string with line breaks
  (the book's place formats) is read by the client's `text::exe_string`.
- The viewer reads `clients/layouts/*.toml` at run time: a layout change made while a batch
  runs breaks (or changes) the release binary's comparisons of the batch (M4 T3b-1: an unknown
  field); keep the layout with the binary it was built for until the batch ends.
- A capture times out at the main menu when the CPU is busy: run it alone. A fresh sandbox's
  first capture once differed (T7c-2): recapture. A background job stops at 2 h: batches.
- Sieges: machines acting on their own at the opening act before the record opens (CLT-Q20: the
  defender commands its turrets); the turrets' counts need `structures = 1`; in hot seat the
  original shows the side to act's view (runbook, "A siege").
- The Bink decoder (`h3-bink`, E17 M5 T4b2a) is checked against FFmpeg only through
  `clients/scripts/bink-reference.py` (PyAV, each frame released before the next): the `ffmpeg`
  command line holds the frame before, so its decoder copies only the picture's rows into a new
  buffer and the chroma's hidden rows (60 to 63) differ (`LBSTART`, `RTLOOP`, `RTSTART`); and a
  test clip is 256 pixels wide, as the game's (FFmpeg pads narrower rows, so a vector crossing
  a row's end reads its padding).
- Coverage counts regions: an `if let Some(x) = s.get_mut(i) { … }` without `else` leaves one
  uncovered when the index is always inside (route such writes through one helper whose outside
  case a test takes, `h3_bink`'s `dsp::set`), and a `const fn` evaluated only at compile time is
  uncovered until a test calls it at run time.
