# Reference captures: the runbook

*Provisional* (E17 M1 T8, M2 T1; ADR-0020 point 9; findings in `docs/epics/E17-design.md` §6).
The original game runs unattended in a Linux sandbox under Wine, on a virtual X display at 16 bits,
so an XWD dump of the display is the original's own 5-6-5 screen and compares exactly with our
renders. A script steers it through its menus by fixed coordinates and recognises each screen by a
few probe pixels, so nobody has to look at a screenshot. Our `xdd.dll` takes over the game's
clock, so each frame is taken at an exact game time.

Two rules: the game runs from a copy under `original-data/wine/` (never the installation, ADR-0009:
it writes saves and settings into its folder), and captures and renders of the original never
enter git (they stay in the sandbox or `clients/out/`).

## The pieces

| File | What it does |
|---|---|
| `setup.sh [copy]` | Installs 32-bit Wine, Xvfb, xdotool, ImageMagick and mingw-w64; makes the run directory `original-data/wine/game` from the copy (default `original-data/install`); builds `xdd.dll`; writes the game's settings (no intro, no grid, no sound, full screen) into the Wine prefix (`$WINEPREFIX`, default `~/.wine-h3`, in a directory the shell's user owns). |
| `xdd.c`, `xdd.def` | Our `xdd.dll`: forwards the GOG executable's `DirectDrawCreate` to Wine's ddraw, and the **virtual clock** (below). |
| `h3drive.py` | The driver: starts the game, the recipes (fixed coordinates, screens by probes), the clock, dumps; the commands `battle`, `set`, `learn`, `shot`. |
| `screens.toml` | The screens the driver recognises: 12 probes `[x, y, 5-6-5 value]` each (numbers only). |
| `h3cap.py` | `diff` (our render against a capture, with masks) and `find` (the original's obstacles in a capture). |
| `../situations/` | The capture set: `set.toml` and its situations' scenario files. |
| `h3-view situation` | Writes the original's map for a scenario's battle (`cargo run --release -p h3-view -- help`). |

Before the first run: stage the copy's files (`Heroes3.exe`, the five DLLs `BINKW32.DLL`,
`IFC20.dll`, `MP3DEC.ASI`, `MSS32.DLL`, `SMACKW32.DLL`, `Data/` with the four LODs, `Heroes3.snd`,
`H3ab_ahd.snd`, `H3ab_ahd.vid`; one large file per staging call), run `clients/capture/setup.sh`,
and build `cargo build --release -p h3-view` in `clients/`. A run of the whole set takes about 70 s.

## Common uses

**Run the capture set** (the regression check): from `clients/`,

```
python3 capture/h3drive.py set [--only <id>] [--out <dir>]
```

For each situation: its map (`h3-view situation`), its frames in the original, our render, and
the masked diff of each frame against the count `set.toml` records. One line per frame
(`grass t=0: 412 differing (recorded 412) ok`); exit 1 if any count changed. `<dir>/frames.sha1`
holds the pixels' digests: two runs (say, one after a fresh `setup.sh`) agree when these files do.
Each situation's folder keeps the frames (`t00000.xwd`, …), `ours.png`, its record and a
`diff-t*.png` (differing pixels red, masked ones blue).

**One battle at chosen times:**

```
./target/release/h3-view situation situations/grass.toml --out out/grass.h3m [--tile 20,20]
python3 capture/h3drive.py battle out/grass.h3m out/grass --frames 0,100,200 [--keep]
python3 capture/h3cap.py diff out/grass/t00000.xwd ours.png --record ours.jsonl \
    --mask panel,heroes,count_boxes --out diff.png
```

The times are milliseconds after the battle's opening on the game's own clock. `--keep` leaves the
game running (frozen) for more commands; `wineserver -k` ends it.

**Add a situation:** write its scenario beside `set.toml` without obstacles, add its entry, run
`set --only <id>` (it prints `new`), then `python3 capture/h3cap.py find
<out>/<id>/t00000.xwd situations/<id>.toml` prints the original's obstacles as scenario lines (a
score of 1.0 is an exact match; an obstacle partly covered by another scores above 0.9). Paste
them, run again, and record each frame's count with why. Situations need the human's stack to
open (speed above every enemy's; the original's AI acts at once) and armies of one faction or
without Leadership's morale; the heroes have no skills, artifacts or war machines.

**Another battlefield:** the original's obstacles depend on the battle's tile, the background on
what stands around it (§4.4): `--tile x,y` (the set's `tile`) moves the battle. At (20, 20) on dirt
the original places an absolute obstacle.

**Frame by frame** (timings, T3), as a library from `clients/capture/`:

```python
import h3drive
game = h3drive.Game("out/steps")
game.start("out/grass.h3m")
h3drive.to_scenario(game)              # main menu -> adventure map
t0 = h3drive.into_battle(game)         # freezes the clock, steps 10 ms until the battle shows
for ms in range(1, 101):
    game.clock("step 1")               # one millisecond on; waits until the game is idle
    game.dump(f"out/steps/{ms:04d}.xwd")
game.stop()
```

**A new screen** (another recipe): get there by hand once with `Game.click`/`key`, then
`python3 capture/h3drive.py learn <name> --in x0,y0,x1,y1 [--avoid …] [--also other-state.xwd]`
records its probes from the part of the screen that tells it apart (`--also`: the same screen in
another state, whose probes must agree). Add the recipe to `h3drive.py` with its coordinates.

## The recipes' coordinates (800 × 600)

| Where | Action | Point |
|---|---|---|
| Main menu | New Game | (640, 75) |
| New game menu | Single Scenario | (640, 75) |
| New game menu | Back | (640, 515) |
| Scenario list | Begin (the only map in `Maps/` is selected) | (497, 553) |
| Adventure map | park the cursor (an edge scrolls the map) | (700, 300) |
| Adventure map | step the selected hero east into the enemy | key `Right` |
| Battle | park the cursor over the panel | (420, 580) |

## The clock

`xdd.dll` points `Heroes3.exe`'s one clock import, WINMM's `timeGetTime` (6 call sites; the
executable imports no other clock or timer), at a virtual clock; the executable's bytes stay as
they are. With `H3CLOCK` set to a directory (a Windows path; the driver uses `Z:` + its output
folder's `clock/`), it runs in real time until a command arrives in `ctl`, one line
`<seq> freeze | step <ms> | run`; a new `<seq>` applies once. Every 2 ms it writes `ack`:
`<seq> <time> <reads>`, the reads of the clock since the command. A game that keeps reading a
stopped clock is idle at that time: `Game.clock` waits for 20 reads, `Game.settled` for two equal
dumps (Wine may draw a moment later).

## Traps

- The game draws its own cursor: probes keep 50 px away from where it rests, and the recipes
  park it.
- The battle panel's buttons change with whose turn it is: the `battle` probes are the ones equal
  in both states.
- An XWD file's colour map holds bytes the X server leaves unset: compare pixels, never files.
- The menus run in real time; only the battle runs on the stepped clock. Two runs still agree:
  the battle's setup does not depend on the clock (observed: the obstacles follow the tile).
- Display `:99` belongs to the driver; `setup.sh` uses `:98` for `wineboot`.
