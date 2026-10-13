# Reference captures: the runbook

*Provisional* (E17 M1 T8, M2 T1, T3, T6; ADR-0020 point 9; findings in `docs/epics/E17-design.md` §6).
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
| `setup.sh [copy]` | Installs 32-bit Wine, Xvfb, xdotool, ImageMagick and mingw-w64; makes the run directory `original-data/wine/game` from the copy (default `original-data/install`); builds `xdd.dll`; writes the game's settings (no intro, no grid, no sound, full screen, a hero's path shown before it walks) into the Wine prefix (`$WINEPREFIX`, default `~/.wine-h3`, in a directory the shell's user owns). |
| `xdd.c`, `xdd.def` | Our `xdd.dll`: forwards the GOG executable's `DirectDrawCreate` to Wine's ddraw, and the **virtual clock** (below). |
| `h3drive.py` | The driver: starts the game, the recipes (fixed coordinates, screens by probes), the clock, dumps; the commands `battle`, `set`, `learn`, `shot`. |
| `screens.toml` | The screens the driver recognises: 12 probes `[x, y, 5-6-5 value]` each (numbers only). |
| `h3cap.py` | `diff` (our render against a capture, with masks) and `find` (the original's obstacles in a capture). |
| `h3mem.py` | Reads the running game's state from its memory: the battle's clocks and each stack's group, frame, hex; the adventure map's players, heroes, view and path (below). |
| `h3adv.py` | The adventure map (M2 T6): hot seat, heroes, tiles clicked twice, turns; `batch` plays an experiment batch and fills its results template, `compare` sets two filled templates side by side, `learn-button`. |
| `batches/` | The batches' recipes: a spec's procedure as steps (`E08-batch-2.toml`). |
| `../situations/` | The capture set: `set.toml` and its situations' scenario files; the recorded sequences, `sequences.toml`, their scenarios `seq-*.toml` and `timelines/`. |
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

**A sequence frame by frame** (no screenshots): from `clients/`,

```
python3 capture/h3drive.py sequence out/walk.h3m out/walk --action hex:40 --ms 2700 [--pre <ms>] [--screen]
python3 capture/h3drive.py sequences [--only walk]
```

`sequence` opens the battle, steps the clock `--pre` ms, performs the actions at the frozen clock
(`hex:<n>` clicks a hex at `hex_point`, `key:d` defends, `click:x,y`, `move:x,y`, `step:<ms>`) and
then steps 1 ms at a time, reading the game's state after each step: `<out>/states.jsonl` (every
state, with `--screen` a digest of the battlefield's pixels) and `<out>/timeline.txt` (one line per
change of a stack's group, frame or hex). `sequences` replays the recorded ones and fails when a
timeline changes; add one like a situation (its entry, `new`, then copy its timeline into
`timelines/`).

**An experiment batch on the adventure map** (M2 T6; `docs/experiments/README.md`): from
`clients/`,

```
python3 capture/h3adv.py batch capture/batches/E08-batch-2.toml [--map <h3m>] [--out <dir>]
python3 capture/h3adv.py compare out/batches/E08-batch-2/E08-batch-2.md ../experiments/results/E08-batch-2.md
```

`batch` compiles the recipe's spec with `h3-experiment` (the engine's workspace; `--map` plays a
map compiled before, after checking its stamp), starts it in hot seat with the recipe's
`players` as the humans, plays the steps and writes `<out>/<batch>.md`, the batch's results
template filled in (a note at its top says how it was played), and `<out>/steps.jsonl`, each
step's observation. E08 batch 2 (41 steps) takes about three minutes. A recipe is the spec's
procedure as `[[step]]`s: a hero's step selects the hero (`hero`, its name on the map), clicks
`walk`'s tile once, reads the path's marks from memory, clicks it again and waits until the hero,
its points and its player's resources rest; `record` fills the template's fields and table rows
from the observation (`arrived`, `moved`, `mark`, `marks`, `points`, `points_if_moved`,
`resource:<name>`), `expect` notes a difference under "anything odd"; `end_turn = true` ends the
turn. A step without `walk` only reads. Everything the record holds comes from the game's memory,
not from the screen. `compare` prints each record line of two filled templates as the same or
different (random amounts differ by nature: judge them by their ranges).

**A new screen** (another recipe): get there by hand once with `Game.click`/`key`, then
`python3 capture/h3drive.py learn <name> --in x0,y0,x1,y1 [--avoid …] [--also other-state.xwd]`
records its probes from the part of the screen that tells it apart (`--also`: the same screen in
another state, whose probes must agree). Add the recipe to `h3drive.py` with its coordinates. A
button that shows in more than one place (a dialog's OK): `python3 capture/h3adv.py learn-button
<name> x0,y0,x1,y1 [<dump.xwd>]` records 12 probes relative to its corner (`[button.<name>]`);
`h3adv.find_button` finds it anywhere on the screen.

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
| New game menu | Multiplayer | (640, 185) |
| Multiplayer dialog | Hot Seat | (580, 152) |
| Hot-seat names | name *i* (0 to 7; the first has the focus) | (400, 186 + 30 *i*) |
| Hot-seat names | OK | (340, 444) |
| Any dialog | OK: found by `[button.ok]` | its corner + (33, 16) |
| Adventure map | the hero list's row *i* (0 to 4) | (638, 227 + 32 *i*) |
| Adventure map | tile (x, y) in the view at (vx, vy) | (24 + 32 (x − vx), 24 + 32 (y − vy)) |
| Adventure map | the minimap (72 × 72 map): centre the view on (x, y) | (630 + 2x, 26 + 2y) |
| Adventure map | end the turn | key `e` |
| Adventure map | park the cursor over the right panel | (700, 300) |

## The clock

`xdd.dll` points `Heroes3.exe`'s one clock import, WINMM's `timeGetTime` (6 call sites; the
executable imports no other clock or timer), at a virtual clock; the executable's bytes stay as
they are. With `H3CLOCK` set to a directory (a Windows path; the driver uses `Z:` + its output
folder's `clock/`), it runs in real time until a command arrives in `ctl`, one line
`<seq> freeze | step <ms> | run`; a new `<seq>` applies once. Every 2 ms it writes `ack`:
`<seq> <time> <reads>`, the reads of the clock since the command. A game that keeps reading a
stopped clock is idle at that time: `Game.clock` waits for 20 reads, `Game.settled` for two equal
dumps (Wine may draw a moment later).

## The original's state

`h3mem.py` opens the game's `/proc/<pid>/mem` (Wine maps `Heroes3.exe` at its image base, so the
map's addresses hold): the frame deadline and the idle step's time, the combat speed, and per
stack its creature, hex, group, frame, facing, offsets and times (`docs/exe/structs.toml`). On
the adventure map (M2 T6): the current player, each player's selected hero, hero list and
resources, every hero's name, tile and points, the view's top-left tile, the tile under the
mouse and the selected hero's path (design §6.3). It only reads. `python3 capture/h3mem.py`
prints the battle's state as JSON while a battle runs, `h3mem.py adventure` the adventure map's;
`Mem` is the library. How to read a battle's state (drawn now, shown at the next deadline):
design §7.1.

## Traps

- The game draws its own cursor: probes keep 50 px away from where it rests, and the recipes
  park it.
- The battle panel's buttons change with whose turn it is: the `battle` probes are the ones equal
  in both states.
- An XWD file's colour map holds bytes the X server leaves unset: compare pixels, never files.
- The menus run in real time; only the battle runs on the stepped clock. Two runs still agree:
  the battle's setup does not depend on the clock (observed: the obstacles follow the tile).
- Display `:99` belongs to the driver; `setup.sh` uses `:98` for `wineboot`.
- A click at the frozen clock acts at once and the computer's reply too (the AI acts in the same
  millisecond); a sequence's timeline therefore also holds the computer's moves. A click on an
  enemy the active stack cannot reach may still end its turn (seen once with Angels): click
  reachable hexes.
- The combat speed lives in the prefix's registry (`Combat Speed`, 0 to 2, not set by
  `setup.sh`: 0); a run at another speed sets it with `wine regedit` and puts it back.
- `setup.sh` sets `Show Route` (the game's default in a new prefix is 0: the first click on a
  tile walks at once). The game also walks on the first click when the hero's path already ends
  on that tile; `walk` notes it under "anything odd".
- On the adventure map a click on another of the player's heroes routes the selected hero to it
  (a meeting): heroes are selected from the hero list only. The list's scrolling is not built: a
  hero beyond its fifth row stops the batch.
- The adventure panel takes the current player's colour: the `adventure` probes are the ones
  equal on Red's and Blue's turn (`adventure_map`, Red's, stays for the battle's recipes).
- A player without towns gets a message at the end of its turn and at the start of its next
  ("you have lost your last town"); `dismiss` and `end_turn` click every OK.
- The adventure map walks in real time (the clock runs); a walk is over when the hero, its points
  and its player's resources rest for 1.5 s. Keep heavy builds off the machine while a batch runs.
