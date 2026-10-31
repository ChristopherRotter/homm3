# Map client lane — standing traps

Moved out of the lane's HANDOFF on 2026-10-24 so that session start stays short; verbatim, grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc it belongs to, a `shared/` change).

- **The gate and `h3map.py`** need the four LODs and the shipped maps (the harness also the runbook's DLLs and sound archives, then `clients/capture/setup.sh`): `sandbox-setup.sh` copies only the bitmap LODs, so `cp -rn` the staged `original-data/.` into the clone (untar `maps.tar` into `install/`, `extracted.tar` beside it). Since M2 T5 the gate and the window's sound need `Data/Heroes3.snd`, `H3ab_ahd.snd` and the nine terrain tracks of `install/MP3/` (`DIRT.MP3` … `Underground.mp3`: stage them too; the gate reports a missing one).
- **A template's class in memory is the game's** (T5b1): the load turns the map's 219 into 33, 220 into 53, decorations 165–205 into others (§2.9), so `live.toml`'s `classes` differ from the map's with no random draw.
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
- **One game at a time:** `h3mem.Mem()` takes the first game process it finds, so a `still --keep` game left running makes the next still read it (T5a); stop it first (`wineserver -k` with `h3drive.env()`). At a human player's turn start the selected hero is off its tile until the view moves: find heroes by their records (`Mem.hero_on`).
- **Driving the window under Xvfb** (M2 T4b1): `xdotool click --window <id>` sends a synthetic
  event that SDL ignores; move the pointer with `mousemove --window` and click with a plain
  `xdotool click 1` (keys with `key --window` arrive). The window takes several seconds to open
  while the archives load: wait for `xdotool search --onlyvisible --name h3-map-view` before
  sending anything, and keep the run's output out of the repository.
- The client lane's standing traps apply to the foundation and the harness (`docs/lanes/client/HANDOFF.md`): the licences (`winit`, `cpal`, Bevy are excluded: SDL 3), the purity allowlist of a pure crate, `clients/Cargo.lock` as a common path, no float arithmetic in drawing and timing, never running the original from the installation, captures never in git, a capture run alone.
