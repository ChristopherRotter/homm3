# Map client lane — standing traps

Moved out of the lane's HANDOFF on 2026-10-24 so that session start stays short; verbatim, grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc it belongs to, a `shared/` change).

- **The gate and `h3map.py`** need the four LODs and the shipped maps (the harness also the runbook's DLLs and sound archives, then `clients/capture/setup.sh`): `sandbox-setup.sh` copies only the bitmap LODs, so `cp -rn` the staged `original-data/.` into the clone (untar `maps.tar` into `install/`, `extracted.tar` beside it).
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
- **A sequence's objects' phases and frame steps are 0** (T1b): `frames` writes them (the load's random phases and real time moved the screen's changed boxes between runs); a still's (`set`, `still`) stay the load's, read into its live part.
- **One game at a time:** `h3mem.Mem()` takes the first game process it finds, so a `still --keep` game left running makes the next still read it (T5a); stop it first (`wineserver -k` with `h3drive.env()`). At a human player's turn start the selected hero is off its tile until the view moves: find heroes by their records (`Mem.hero_on`).
- The client lane's standing traps apply to the foundation and the harness (`docs/lanes/client/HANDOFF.md`): the licences (`winit`, `cpal`, Bevy are excluded: SDL 3), the purity allowlist of a pure crate, `clients/Cargo.lock` as a common path, no float arithmetic in drawing and timing, never running the original from the installation, captures never in git, a capture run alone.
