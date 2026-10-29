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
  `frames --only <id> --reuse` keeps its live part.
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
- **One game at a time:** `h3mem.Mem()` takes the first game process it finds, so a `still --keep` game left running makes the next still read it (T5a); stop it first (`wineserver -k` with `h3drive.env()`). At a human player's turn start the selected hero is off its tile until the view moves: find heroes by their records (`Mem.hero_on`).
- **Driving the window under Xvfb** (M2 T4b1): `xdotool click --window <id>` sends a synthetic
  event that SDL ignores; move the pointer with `mousemove --window` and click with a plain
  `xdotool click 1` (keys with `key --window` arrive). The window takes several seconds to open
  while the archives load: wait for `xdotool search --onlyvisible --name h3-map-view` before
  sending anything, and keep the run's output out of the repository.
- The client lane's standing traps apply to the foundation and the harness (`docs/lanes/client/HANDOFF.md`): the licences (`winit`, `cpal`, Bevy are excluded: SDL 3), the purity allowlist of a pure crate, `clients/Cargo.lock` as a common path, no float arithmetic in drawing and timing, never running the original from the installation, captures never in git, a capture run alone.
