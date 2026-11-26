# Map client lane — standing traps

Moved out of the lane's HANDOFF on 2026-10-24 so that session start stays short; verbatim, grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc it belongs to, a `shared/` change).

- **The gate and `h3map.py`** need the four LODs and the shipped maps (the harness also the runbook's DLLs and sound archives, then `clients/capture/setup.sh`): `sandbox-setup.sh` copies only the bitmap LODs, so `cp -rn` the staged `original-data/.` into the clone (untar `maps.tar` into `install/`, `extracted.tar` beside it). Since M2 T5 the gate and the window's sound need `Data/Heroes3.snd`, `H3ab_ahd.snd` and the nine terrain tracks of `install/MP3/` (`DIRT.MP3` … `Underground.mp3`: stage them too; the gate reports a missing one).
- **The game's own disk check** (E24 M4 T1a; the client lane's trap too): the original reads the free space in signed 32 bits (observed; `free_disk_space` 0x50C7A0 not read) and ends the game below 5 MiB (`main_menu_proc`'s floor of 5 242 880 bytes, read), so the disk's free bytes modulo 4 GiB must lie from 5 MiB up to 2 GiB: outside, every run stops at the main menu. Since E24 M4 T6 (R6-6) `Game.start` checks it first (`h3drive.disk_check_fault`: `os.statvfs` of the run directory, as `df -B1` gives it) and fails before the game starts, naming a ballast's size that brings the free bytes to 1 GiB (`fallocate -l <n> <file>`, outside the run directory); `H3_NO_DISK_CHECK=1` leaves the check out. Free space changes with every build (another session's too): size the ballast again before a long run.
- **A computer player's turn out of sight takes no virtual time** (E24 M4 T1a, design §5.1): on the harness's frozen clock everything from End Turn's command to the next day's start falls in that millisecond, so a recording never shows the status window's kind 5 there; a turn's facts come from the live part's `[[turn]]`, read in that millisecond.
- **A computer player's turn needs a `seed`** (E24 M4 T1b1, design §5.2): the game seeds its generator once, from the running clock, so without a sequence's `seed` no two runs walk alike. A computer with a Tavern in its town hires a hero and may walk it into sight (a hero the live part at 0 does not hold): the `ai-*` situations give Blue no town. An unseen step takes no time, so speed 4 and Blackout Computer show nothing of the turn either. A
  computer's `[[turn]]` table holds the computer's `kingdom` (its selected hero, points and kind
  5's `value`, T1b2), while the panels (the lists, the quest log, the resource bar) stay the
  viewer's through its turn (design §5.2, "The computer's screen").
- **A situation without a town of Red's warns** (E24 M4 T2a1, design §5.3): with heroes and no
  town the scenario's test (at End Turn, and after an event's visit, a battle and the other
  callers of `scenario_condition_met`) opens GENRLTXT 7's box, and every later turn's start the
  days left's (ArrayTxt 128): a sequence that is not about them gives Red a town out of the way
  (the `seq-find-*` situations). A pile's `amount` is the map's: gold in hundreds.
- **The original's new game draws what an engine's start must match** (E24 M4 T3b1, design
  §5.10): a hero's army (unless the map gives one), the starting bonus (500 to 1 000 gold, a
  resource, or an artifact that changes a hero's stored primary skills: `engine-start-owners`'
  Orrin showed [2, 2, 2, 2] with mana 10) and an unnamed town's name; the engine draws armies its
  own way and gives no bonus (E10's). A comparison of the panels gives `map_army`, the names,
  `bonus = "gold"` and the engine's `resources` (shown by a `select` and a `reselect`: a resources
  write needs a selection that changes, and one hero of Red's has none to change to).
- **A shipped map's hero may level up at the start** (E24 M4 T3b2, design §5.11): a map's
  experience (Jared Haret's, Glinda's 2 000) gives levels whose primary skills each game draws, a
  difference no staging writes yet; a scan for a shipped start without draws must look at the
  heroes' experience too, besides the random objects, monsters without a count, abandoned mines,
  unnamed towns and the heroes the map does not arm.
- **A still's `pointer` over the map** (E24 M4 T3b2): it must lie inside a drawn tile (x and y
  modulo 32 at least 8: MCL-Q18 keeps the original's tile one off over the last 8 pixels) and off
  the scroll zones; the capture reads its tile's cost first, and the engine's start has none
  (MCL-Q37), so a pointer over a reachable tile shows a cursor the record cannot: park it over a
  blocked tile, a hero's own tile or a town of Red's.
- **The starting bonus's rows sit in two arrays** (E24 M4 T3b1): `h3mem.starting_bonuses` reads
  16 values; with three players Blue's row was the second array's, so `h3drive.gold_bonus` counts
  the Gold read, not a row's index.
- **A template's class in memory is the game's** (T5b1): the load turns the map's 219 into 33, 220 into 53, decorations 165–205 into others (§2.9), so `live.toml`'s `classes` differ from the map's with no random draw. A class rule (the hint's, the cursor's, an engine start's `[hover]` facts: M4 T6, R4-1) reads the game's (`Records::game_class`); the engine's start's `classes` keep the map's (the sound reads 220).
- **A situation changed since its capture** shows as `STALE` in `drawn` (T6b: the kept live part records the situation's digest): capture it again (`set --only <id>`).
- **A recording reads the tree as it goes** (M2 T2): each `frames --only` run loads `h3map.py`
  afresh and its end draws ours with the release `h3-map-view` and the committed tables and
  layout, so a change to any of them while a recording runs breaks its later sequences (a new
  table field stopped a run's last draw): keep them as they are until it ends (a stopped run's
  screens stay; `frames --reuse` compares again).
- **A sequence's kept live part changes with each run** (M2 T2): its palette steps at 0 are the
  load's real time's (`cycles`, `river_cycles`), so a run as recorded keeps a new
  `timelines/<id>-live.toml` and `drawn` then finds its frames `CHANGED`: `drawn --record` after
  every `frames` run that kept one.
- **A sequence's points are the recording's** (M3 T4b): the viewer takes the selected hero's
  points at each step's end from the recorded timeline's `points` lines (as its paths from the
  `arrows` lines), and the kept live part's kingdom is the list as shown before the harness
  wrote the sequence's `points`: a new or changed walk draws its movement bar right only once
  its timeline is recorded (`frames`, then `--reuse` with the new timeline in place).
- **A sequence without `points` starts with the start's** (M3 T5a): the new game's random army
  sets the hero's points (its slowest creature), so `scroll-0`'s `0 points` line was 1700 in
  T4b's run and 1500 in T5a's: a `CHANGED` in that line alone is no regression (the frame check
  compares with the same run's screens): copy the run's `timeline.txt` into `map/timelines/` and
  `frames --only <id> --reuse` keeps its live part. The start's army and resources differ the
  same way (M3 T5b1: `scroll-0`'s gems 15 in T5a's run, 10 in T5b1's): the kept live part's
  `kingdom` and `status` are the run's. So do the status bar's hints (M3 T7a): a hero's name, a
  Witch Hut's skill, a Shrine's spell, a quest, a bank's guards, an army are the new game's draws,
  so a recorded hint's `screen` box moves between runs (21 of 35 timelines in T7a's): a `CHANGED`
  in `screen` or `0 points` lines alone, every frame exact and the viewer giving the run's lines,
  is no regression: copy the run's `timeline.txt` in and `--reuse` (its `frames.txt` too when it
  kept another screen).
- **A sequence's options left out are the original's settings** (M2 T3a): the harness writes
  only the options a sequence gives, so one that scrolls without its `scroll_speed` ran at what
  the game held (`walk-follow`'s Ctrl and Down moved one tile, speed 0, in T1b's run) while the
  viewer takes the default (1): give every option a sequence's inputs use.
- **A sequence's objects' phases and frame steps are 0** (T1b): `frames` writes them (the load's random phases and real time moved the screen's changed boxes between runs); a still's (`set`, `still`) stay the load's, read into its live part.
- **A level's index holds for the objects it was built for** (M3 T2b1, design §4.2): the
  painter's `TileIndex` (each tile's draw list, each object's DEF, shape, flag and phase; since
  M3 T3b the minimap's level colours with the explored tiles, §4.4) is built at the map's load;
  whatever changes the live part's objects or explored tiles (an owner, a picture, a fallen
  object, a hero's sight: the engine's, M4) must build that level's index again, or the view and
  the minimap draw the old ones. `scene` (the stills, `render`) builds its own each time.
- **A kept frame draws a cell again only when its draws change** (M3 T2b2, design §4.3): the
  composer compares each 32 × 32 cell's draws (with their DEFs' cycle steps and the flag
  palette's name) with the last scene's and leaves the cell's pixels when they are the same; what
  changes a piece's pixels otherwise (a DEF decoded again under its name, a palette changed but
  by the scene's cycles) needs a new `Composer`, and a scene's `view.origin` must be the point
  every piece of the view moves by (the clip's pixels move by its change), or a scroll leaves
  stale pixels. The window's `--check` line `afresh` finds either.
- **The status window over time** (M3 T5b2, design §4.9): the timeline's `Loop::draw` is a view's
  draw (0x40F3F0) and ends with the status window's chooser, so a new caller of it chooses too
  (a date's window steps, a kind asked ends there); a status the live part does not give (none)
  is left as it is, and a kind asked ends into the selection's window only when the live part
  gives it (`selection`, which the harness reads before it asks a kind: at the staging when the
  situation asks one, so a sequence asking a kind over it keeps that one). A date's window at a
  sequence's 0 must be the sequence's own `status` (built at 0): the harness refuses another.
- **The mouse-over's costs are per tile looked at** (M3 T6a1, design §4.10): the original's
  pathfinder stops at the tile under the pointer, so the live part's `[hover]` `reach` holds only
  the tiles a sequence's inputs visit (`h3map.look_over_view` reads each before 0): a new or moved
  pointer input over the view needs a new `frames` run, and a still's `reach` is empty (the window
  shows the arrow over the tiles it has no cost for, as after a walk). `[hover]` is the live part's
  last table: a key written after it belongs to it.
- **The visit marks' facts check themselves** (M3 T6a2a, design §4.11): a situation's `visits`
  are written by `h3mem.VISIT_RULES` inverted and the live part's `[hover]` `visits` read back by
  the same rules, so a wrong rule agrees with itself; only the original's status bar (in a
  recording, compared everywhere since M3 T7a) tells. They are the selected hero's as staged, as are the contents' (M3 T6a2b,
  §4.12: a Shrine's spell known, a Tree's mark, a Witch Hut's skill) and the reach: after a hero's
  choice (M3 T6b1) the viewer keeps them, so a sequence moves the pointer over the map only before
  its first choice. A kind known is a kind's bit: every object of the kind shows its bonus.
- **A staging's redraws empty the status bar** (M3 T7a): a list's redraw (MCL-Q22) or a path's
  click leaves the bar empty with the item looked at last kept, so a pointer parked on that same
  item shows no text (`show_path`'s stills differed by the parked button's hint): a staging step
  that redraws moves the pointer over another item (`MINIMAP_POINT`) before parking again.
- **The VM restarts mid-run** (M3 T7a: twice in one session; files kept, jobs gone): a whole
  `frames` run loses every sequence after the one in progress. Run `frames --only <id>` per
  sequence from a loop that marks each id done, so a restart costs one sequence; `set` is short
  (about 10 s a situation) and is simply run again.
- **A button held stops the map** (M3 T6b1, design §4.13): from the press to the release only the
  tick runs (the palettes step in memory, no redraw, no edge scroll, no hint); the release's
  millisecond redraws. A list's redraw (a choice, an arrow, Next Hero, a lift that centres, a walk
  that ends unable to move) leaves the status bar empty on screen with the hint in memory
  (MCL-Q22): only another item's hint shows a text again.
- **After a walk the mouse-over has no reach** (M3 T6b3): the facts' reach is the staged hero's
  tile's (the engine's from M4), so the viewer's cursor over the view is then the arrow, where the
  original's is the tile's; a click that starts a walk records that frame even hidden (its
  `cursor` line). A sequence walks a path again only from the staged tile: `walk-stopped`'s first
  run, which clicked the destination again after each walk it ended, became three sequences.
  Since M4 T2b2 a sequence takes that cursor's frame from its recorded timeline (`Start::reached`,
  as the points): a box closed with the mouse over the view checks only when the cursor changes.
- **The input ring counts messages, not actions** (M3 T6b3, design §4.15): a click two, every
  other action one; a scroll's Control key's own down and up and a button's move before it are not
  counted, so a sequence that fills the ring uses keys without Ctrl (`walk-queue`). The ring holds
  only what waits during a step, a walk's frames or a wait: nothing else bounds the actions.
- **The minimap and the status window hold the mouse their own ways** (M3 T6b2, design §4.14):
  while the mouse holds the minimap down only its moves (each drawn, the corner moved or not) and
  its release reach the timeline (`Loop::drag`), the idle step stopped; a press on the status
  window's picture clicks at the release, wherever it is, unless the window was built anew in
  between (MCL-Q23). A sequence's point off the panels that should not scroll keeps out of the
  edge zones: the resource bar's row below y 585 is the bottom one (`status-click`'s first run
  scrolled there at the original's held speed).
- **A situation's underground** (M3 T6b2): the harness explores it whole, waits for the screen
  `adventure` (the underground's button enabled breaks `adventure_map`'s probes) and reads only the
  view's level's heroes into the live part: a recording puts no hero or town below.
- **`choose = true` stages slowly** (M3 T6b1): `choose_each` clicks through every hero and town,
  then the staging's selection and the lists' first slots by their arrows; it needs the
  situation's `select` (the lists' writes) and every path's tile inside the situation's view.
- **The harness's writes run none of the game's updates** (M3 T6a2b): a quest's visitors written
  left the Quest Log as the map's start set it, so the live part reads the shown item's state
  (`Mem.item_states`), never one recomputed from what the harness wrote.
- **A Windows build finds nothing by the build's paths** (M3 T7c1): `env!("CARGO_MANIFEST_DIR")`
  is the sandbox's folder, absent on the user's machine, so every file a command reads comes from
  `--map` (the catalogue beside it, `<map>/../text/keys.toml`) and `--install`; a hand test's
  `.cmd` passes both, master's `clients/map` and `clients/text` copied into `out/win/<task>/` when
  the user's checkout is behind. A new file a command reads goes beside `--map` too.
- **A new table in the common catalogue fails the client lane's test** (M4 T2a2):
  `h3-battle-client`'s `text::tests::the_committed_catalogue_parses` lists every original table
  `clients/text/keys.toml`'s keys name (a family's do not count), so a key from a new file
  (ADVEVENT at T2a2) fails `check.sh`'s tests: one word in the client lane's crate, a
  `cross-lane` change with the user's consent, the client lane's inbox told.
- **The original's screens are local** (M4 T2a2): a sequence's frame check compares the viewer
  with the screens under `clients/out/map-frames/<id>/`, never in git, so a fresh sandbox records
  each sequence again first (`frames … --only <id>`, one id per run), then checks with `frames
  --only <id> --reuse` while the viewer changes.
- **A box's `[[message]]` tables grew** (M4 T2b1, design §5.4): each holds the box's `style`,
  `chosen` and `marks`, and a button pressed or a picture chosen appends a table of its own, so a
  T2a sequence recorded again (`find-*`, `turn-*`) keeps a live part with more tables than git's:
  expected, nothing of the viewer reads them. A choice's OK is disabled until a picture is
  chosen (styles 7 and 10): a sequence's Enter before the click does nothing.
- **A kept text is a label** (the labels task, ADR-0009): `frames` writes a box's, a level-up's
  and a find's texts as their records (`h3lod.label`), and `h3-map-view`'s `tests/texts.rs`
  fails on a kept live part with anything else (a live part from an older branch, a hand edit):
  label it with `h3lod.label` (a label stays as it is). A recording again may move the
  palette's phase at 0 (`cycles` 10 for 11, two of three runs at the labels task): keep git's
  lines where only the labels were meant to change.
- **A scripted sequence needs the engine's binaries** (M4 T4a2, design §5.13): `frames`' frame
  check and `drawn` make a sequence's record from its `script` with the root workspace's
  `target/debug/h3` and `h3-server` (`H3_ENGINE` another folder): build them first (`cargo build
  --locked -p h3-cli -p h3-server`, about 2 minutes), and again after merging `master` (an
  engine change may change the record: `frames --only <id> --reuse` the 13 scripted sequences,
  ten `game-*` and three `battle-*`, again). Since M4 T6 (R6-12) `engine`, `drawn` and `frames`'
  frame check say a missing binary once, up front (`h3map.check_engine`), and warn on stderr of
  binaries older than the newest `crates/**/*.rs` or `Cargo.toml`.
- **A record's gaps are listed** (M4 T4a2): a scripted sequence's timeline differs from the
  original's by `timelines/<id>-gaps.txt` exactly; a gap lifted or new (MCL-Q39 or MCL-Q40
  answered, a fact the record gains) reports "a gap CHANGED": the run's `gaps.txt` under
  `clients/out/map-frames/<id>/` is the new list, copied in under the file's comments once
  reviewed, with `<id>-frames.txt` and `drawn --only <id> --record`.
- **A record's find is its walk's** (M4 T6, the reviewer's R2-1, design §5.14): `Found::walk`
  ties a record's visit to the walk whose end visits it (`Visit::walk`: its place among
  `Played::moves`, not among the visits; an empty path is a walk without one); a find built for
  a record without it is told at every visit to its tile as a recording's (its `again`). Its
  level-ups ride on it (`Found::levels`), so a record's `Start::experience` and `level_ups` stay
  empty (the tables' thresholds are a recording's).
- **A record names players by id, the map draws colours** (M4 T6, the reviewer's R3-1, design
  §5.15): an event's `owner` or `player` (`ObjectFlagged`, `TurnStarted`, a hero's owner) is an
  index into the observation's `players`, each with its own `colour`; every M4 game has Red id 0
  and Blue id 1, so the two agree there. Turn it into the colour where the observation is at hand
  (`engine::played`: `Visit::flag`, `OtherOpen`'s colour); a test with ids and colours apart
  (`tests/engine.rs::a_record_s_flags_in_their_owners_colours`) catches a raw id drawn.
- **A record's walk need not take its whole path** (M4 T6, design §5.13): `MoveHero`'s path is
  what was clicked; the record's `HeroStepped` are what was walked (a visit from beside stops one
  short, an event or a guard's battle earlier). Read a walk's end from its steps (`Move::stops`,
  `Played::ends`), never from its path's last tile; the paths' search must start at the record's
  next walk (two walks can end on one tile). A golden record that stops on "no path given" or
  "a step ... the record's walks do not take" has usually lost a hero's place this way earlier.
- **The painter's live part follows the draws** (M5 T4a, design §5.26): `Painter::scene` moves
  its heroes to where each draw has them, so a record's heroes (`Played::shown`, by the record's
  start's tiles) matched to `painter.live.heroes` after a live walk lose the walked hero (T3b's
  End Turn in place then did nothing: its turn's hero list mapped to none). A live view matches
  by its start's ids (`for_live`'s, given to `live_turns`).
- **A scripted game draws at random in both games** (M4 T4b1, design §5.14): the original's
  draws under the sequence's `seed` and the engine's under its own differ (a level-up's primary
  skill and offers): record the original first, read its draws from the live part (`[[level_up]]`),
  search the engine's seeds (`h3 adventure run --seed`, each a fraction of a second) for the same
  and set the sequence's `engine_seed`; a draw the harness can write (a chest's gold, `visits`)
  is written as the engine's instead. Pick objects whose effects the record holds: an artifact's
  morale and luck (MCL-Q36) or a revisit to a once-per-hero object (MCL-Q42) are gaps, and a
  click's pointer left over the map gives the original's reach cursor after a box (MCL-Q37):
  move it off the map in the same millisecond.
- **A battle in a sequence runs on real time** (M4 T5a, design §5.19): `h3map.py`'s battle step
  plays it with the clock running, so its situation must have one outcome however long it runs (a
  sure win without losses, a retreat) and nothing after it may draw at random (a level-up's
  offers, Necromancy's sound); the inputs after it count from its `battle` line's millisecond (the
  battle takes none of the sequence's time), and from its result window until the map is the
  active manager again nothing is kept.
- **A battle sequence is a scripted game** (M4 T5b2, design §5.21): the viewer hands over only a
  record's battle, so the situation must agree with the engine's start (the new game's explored
  tiles, the gold bonus: a random one may give an artifact that widens the hero's sight) and the
  engine's draws (guards' counts, a mine's split) come from the map and its `engine_seed`: write
  them for the original (`visits`' guards) after reading the record, search seeds for a count whose
  experience reaches no level (a level-up's dialog holds the map), and keep the engine's script's
  battle commands in step (a changed map or seed changes them: `system battle defend` for the
  guards, the attacker's shots read from the record's legal commands).
- **A hero lifted and a hero on its tile differ by one unit** (M5 T1a, design §5.22): the lifted
  hero's pass 9 blends it 50/50 over itself, which drops each channel's low bit, so a frame
  difference that brightens a hero's whole picture by one unit (no shape changed) is where it
  stands (+0x1EC), not its pose. And a `Draw`'s `frames` counts 0's redraw's step: one more
  than the timeline's `objects` (`State::objects`) at the same moment.
- **Two tests pin the recorded sequences** (M4 T2d1): moving a task's sequences from
  `to-record.toml` into `sequences.toml` fails `sequence::tests::reads_the_recorded_sequences`
  (their count) and `timeline::tests::the_walks_frames_are_m2_t3bs` (the ids replayed, in order)
  until both list them; `drawn` then reports them `new` (`drawn --only <id> --record` each).
  Since M4 T6 (R6-7) a scratch file's run keeps no live part in `timelines/` (nor a sound run):
  once moved, `frames --only <id> --reuse` over the recording's `--out` keeps it, then `drawn`.
- **A hero's secondary skills written show at the walk** (M4 T2c, design §5.5): a situation's
  `secondary` is written without a selection (none of another hero to make), after the status
  window at 0 was built; the walk's first step builds it again from the hero. So the skills
  written, and the type's own they clear, must not change the morale or luck shown (Leadership,
  Luck): `level-up-one` and `-none` give a Cleric (Rion: Wisdom, First Aid) eight skills without
  either (their first staging, Mephala's Basic Leadership cleared and Expert Leadership written,
  differed from 100 in the morale's icon alone).
- **One game at a time:** `h3mem.Mem()` takes the first game process it finds, so a `still --keep` game left running makes the next still read it (T5a); stop it first (`wineserver -k` with `h3drive.env()`). At a human player's turn start the selected hero is off its tile until the view moves: find heroes by their records (`Mem.hero_on`).
- **Driving the window under Xvfb** (M2 T4b1): `xdotool click --window <id>` sends a synthetic
  event that SDL ignores; move the pointer with `mousemove --window` and click with a plain
  `xdotool click 1` (keys with `key --window` arrive). The window takes several seconds to open
  while the archives load: wait for `xdotool search --onlyvisible --name h3-map-view` before
  sending anything, and keep the run's output out of the repository. In play (M5 T4a) a new
  view's window is a new one (search again after End Turn passes the turn); `--check`'s seconds
  are the whole play's, so a script longer than them is cut silently; a root screenshot
  (`import -window root`) shows the window centred on the screen and no hardware cursor.
- The client lane's standing traps apply to the foundation and the harness (`docs/lanes/client/HANDOFF.md`): the licences (`winit`, `cpal`, Bevy are excluded: SDL 3), the purity allowlist of a pure crate, `clients/Cargo.lock` as a common path, no float arithmetic in drawing and timing, never running the original from the installation, captures never in git, a capture run alone.

- **A re-run of CI forces the full verify** (M4 T2b1, PR #1007): `ci.yml`'s plan sets `FORCE` for a
  run's second attempt, and the full verify can outlast the verify job's 40 minutes (cancelled in
  bench-check), while this lane's branches change no engine code and pass in the light path. When a
  light run fails on GitHub's API ("Record the verified code": an empty answer), push a commit to
  run again instead of re-running the job.
- **Sandbox routines as a skill:** GitHub (pull requests and CI by REST), waiting for long jobs, worktrees for concurrent fixers, VM restarts: the skill `homm3-sandbox-session`.
