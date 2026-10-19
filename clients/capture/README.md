# Reference captures: the runbook

*Provisional* (E17 M1 T8, M2 T1, T3, T6, T4b-1, H1, H2; ADR-0020 point 9; findings in `docs/epics/E17-design.md` §6).
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
| `setup.sh [copy]` | Installs 32-bit Wine, Xvfb, xdotool, ImageMagick and mingw-w64; makes the run directory `original-data/wine/game` from the copy (default `original-data/install`); builds `xdd.dll`; writes the game's settings (no intro, no grid, no sound, full screen, a hero's path shown before it walks, auto combat playing the creatures) into the Wine prefix (`$WINEPREFIX`, default `~/.wine-h3`, in a directory the shell's user owns). |
| `xdd.c`, `xdd.def` | Our `xdd.dll`: forwards the GOG executable's `DirectDrawCreate` to Wine's ddraw, and the **virtual clock** (below). |
| `h3drive.py` | The driver: starts the game, the recipes (fixed coordinates, screens by probes), the clock, dumps; the commands `battle`, `set`, `learn`, `shot`, `sequence`, `sequences`, `frames`. |
| `screens.toml` | The screens the driver recognises: 12 probes `[x, y, 5-6-5 value]` each (numbers only). |
| `h3cap.py` | `diff` (our render against a capture, with masks) and `find` (the original's obstacles in a capture); `compare` and `boxes_at` for the frame check. |
| `h3mem.py` | Reads the running game's state from its memory: the battle's clocks, idle steps, active stack and the mouse's highlight, each stack's group, frame, hex, offsets and draw layer, and each side's hero's group and frame and its flag's frame (M3 T3a), the magic animation drawn and the stacks under it (M3 T3b), the battle's outcome (H1); the adventure map's players, heroes (with their experience, level, army and skills: H1), view and path, the difficulty, a tile's monster, the topmost dialog's text (H1; below). |
| `h3adv.py` | The adventure map (M2 T6, H1, H2): hot seat at a chosen difficulty, heroes (the list scrolled), tiles of either level clicked twice, questions answered, battles played, level-ups chosen, turns; `batch` plays an experiment batch and fills its results template, `compare` sets two filled templates side by side, `learn-button`. |
| `h3lod.py` | The original's text tables, read from the run directory's LODs at run time (H1): which record a dialog's text was written from (`ADVEVENT:86`); never their text. |
| `batches/` | The batches' recipes: a spec's procedure as steps (`E08-batch-2.toml`), and the harness's own check (`harness-check.toml`, its spec in `checks/`). |
| `../situations/` | The capture set: `set.toml` and its situations' scenario files; the recorded sequences, `sequences.toml`, their scenarios `seq-*.toml` and `timelines/`. |
| `h3-view situation` | Writes the original's map for a scenario's battle (`cargo run --release -p h3-view -- help`). |

Before the first run: stage the copy's files (`Heroes3.exe`, the five DLLs `BINKW32.DLL`,
`IFC20.dll`, `MP3DEC.ASI`, `MSS32.DLL`, `SMACKW32.DLL`, `Data/` with the four LODs, `Heroes3.snd`,
`H3ab_ahd.snd`, `H3ab_ahd.vid`; one large file per staging call), run `clients/capture/setup.sh`,
and build `cargo build --release -p h3-view` in `clients/`. A run of the whole set (31 situations since M3 T4a) takes about 4 minutes.

## Common uses

**Run the capture set** (the regression check): from `clients/`,

```
python3 capture/h3drive.py set [--only <id>] [--out <dir>]
```

For each situation: its map (`h3-view situation`), its frames in the original with the stacks
its memory holds at each (`h3mem`), our frame at each time (M3 T1b-2: `h3-view render --frames`,
which, as `sequences --frames`, draws the original's drawing bugs the client otherwise fixes, so
that the comparison stays exact: M3 T4a, design §3.6;
the outline's pulse on the original's clock, a fidget in the original's poses, as the frame check
does), and the masked diff of each frame against the count `set.toml` records. One line per
frame (`grass t=0: 0 differing (recorded 0) ok`); exit 1 if any count changed.
`<dir>/frames.sha1` holds the pixels' digests: two runs (say, one after a fresh `setup.sh`) agree
when these files do. Each situation's folder keeps the frames (`t00000.xwd`, …), their
`frames.toml`, ours (`ours/t00000.png`, …, and what it drew, `ours.txt`), its record
(`ours.jsonl`) and a `diff-t*.png` (differing pixels red, masked ones blue).

**One battle at chosen times:**

```
./target/release/h3-view situation situations/grass.toml --out out/grass.h3m [--tile 20,20]
python3 capture/h3drive.py battle out/grass.h3m out/grass --frames 0,100,200 [--keep]
python3 capture/h3cap.py diff out/grass/t00000.xwd ours.png --record ours.jsonl \
    --mask panel,count_boxes --out diff.png
```

The times are milliseconds after the battle's opening on the game's own clock. `--keep` leaves the
game running (frozen) for more commands; `wineserver -k` ends it.

**Add a situation:** write its scenario beside `set.toml` without obstacles, add its entry, run
`set --only <id>` (it prints `new`), then `python3 capture/h3cap.py find
<out>/<id>/t00000.xwd situations/<id>.toml` prints the original's obstacles as scenario lines (a
score of 1.0 is an exact match; an obstacle partly covered by another scores above 0.9). Paste
them, run again, and record each frame's count with why. Situations need the human's stack to
open (speed above every enemy's; the original's AI acts at once) and armies of one faction or
without Leadership's morale. Each side's hero is the scenario's (`attacker_hero`,
`defender_hero`, with their primary skills; M3 T3a) with the scenario's war machines as
artifacts (`attacker_war_machines`, `defender_war_machines`; M3 T3b), without skills or other
artifacts; an entry's `hot_seat = true` (M3 T3b) starts it in hot seat, so that a faster stack of
Blue's opens idle instead of acting (`smoke`);
the packs in their order (base, ab, sod) give the original's hero types. A scenario's
`battlefield` (M3 T2) is set up on the defender's (Blue's) tile: water east of it for the shore
(`base:shore`, on sand), a special ground under both heroes for the nine ground fields; `find`
then tries the field's own obstacle types. The deck and the two boats cannot be set up yet
(CLT-Q5).

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
change of a stack's group, frame or hex, by the original's records: each side's army, then its
war machines; after them a change of the magic animation drawn, `<ms> magic <number> f<frame>
<side>.<record>…` or `<ms> magic -`, M3 T3b). `sequences` replays the recorded ones and fails when a
timeline changes; add one like a situation (its entry, `new`, then copy its timeline into
`timelines/`).

**A battle in progress** (M3 T1a): `--hot-seat` (an entry's `hot_seat = true`) starts the map in
hot seat with Blue a human too (`h3-view situation --hot-seat`), so the actions play both sides and
nothing acts on its own. Among the actions, `turn:<side>.<record>` steps 1 ms at a time, each step
read, until that stack's turn starts (the game names it the active stack), and the actions after
it act then; its time goes to `<out>/turns.txt` (`at:<ms>` steps so up to a time). Act as the turn
starts: a player who waits lets fidgets start, which the viewer's timeline does not have. In the
entry's `[[sequence.command]]`s, `at` gives each player's command its time from `turns.txt` (`ms`
counts from the last one). `behind` (`seq-behind.toml`): two rounds, 8 commands, about 13 s;
`shots` (`seq-shots.toml`, M3 T1b-1): two rounds, 6 commands, three rays, about 10 s; `fly`
(`seq-fly.toml`, M3 T1b-2): two flights across rows and a breath, about 9 s; `machines`
(`seq-machines.toml`, M3 T3b): the war machines act on their own after the Archers' shot (two
Ballistas' shots and the tent's heal), about 8.5 s; `strike` (`seq-strike.toml`, M3 T4b-2b-2):
the Archangel's fixed damage kills the Pikemen, about 4.3 s, its log line compared (below). A war machine without Artillery or First Aid
needs no action (the engine plays its turn inside the command before it). A two-hex striker
stands where the cursor's slot inside the target's hex says (design §7.13): `click:x,y` with an
offset from the hex's point picks it. `hover` (M3 T1b-2) moves the mouse over a stack
(`move:x,y`) and away; it has no engine command (the viewer draws no mouse in a record), so
`h3-view sequences` and the frame check skip it.

**The frame check** (M2 T4b-1, design §7.9): from `clients/`,

```
python3 capture/h3drive.py frames [--only <id>] [--out <dir>] [--reuse]
```

For each recorded sequence: its battle in the original, stepped 1 ms at a time, keeping every
distinct battlefield (`<dir>/<id>/frames/t<ms>.npz` and their states in `frames.jsonl`); the
viewer's frame at each of those times (`h3-view sequences <id> --frames <dir>`: `ours/`, and what it
drew in `ours.txt`, with the heroes the original showed: their defeat and victory come from its
generator; the flags are the viewer's own); the masked diff of each pair (the panel, the count
boxes where the drawn poses show them; the heroes are compared since M3 T3a), written to `frames.txt` and, for a frame that differs, `diff/t<ms>.png`; each
line compared with the sequence's recorded `timelines/<id>-frames.txt` (`ok`, `new`, `CHANGED`;
exit 1 on a change). An entry's `log_at` (M3 T4b-2b-2) compares the log's item from that time on
(`panel_beside_log` instead of `panel`), the viewer drawing its record's first strike's line
there (design §9.6). Each line is `<ms> <differing> <artifacts>` (M3 T1c): the differing
pixels outside the masks and how many of them are the original's **redraw artifacts**
(`h3cap.redraws`, design §7.11): its pixel is ours at that moment with one to five more
shadows, or one we drew there at an earlier moment of the sequence with at least one more. The
diff PNG marks those yellow, any other difference red; a change in either count fails. The
original's frames live only in the sandbox that captured them (`clients/out/`, never in git):
a new sandbox captures them again (all nine with frames, about 60 minutes: `shots`, `fly` and `machines` about 9 each) before `--reuse` works. `--reuse` keeps the original's frames from an earlier run and only redraws
and compares (after a change to the viewer). About 14 minutes for the first four sequences and 15
more for `behind` (default `<dir>`: `clients/out/frames`). Only a fidget comes from the
original's memory; everything else is the viewer's own timeline.

**An experiment batch on the adventure map** (M2 T6, H1; `docs/experiments/README.md`): from
`clients/`,

```
python3 capture/h3adv.py batch capture/batches/E08-batch-2.toml [--map <h3m>] [--out <dir>]
python3 capture/h3adv.py compare out/batches/E08-batch-2/E08-batch-2.md ../experiments/results/E08-batch-2.md
python3 capture/h3adv.py batch capture/batches/harness-check.toml     # the harness's own check
```

`batch` compiles the recipe's spec with `h3-experiment` (the engine's workspace; `--map` plays a
map compiled before, after checking its stamp), starts it in hot seat with the recipe's
`players` as the humans (at least two: the hot-seat dialog takes no fewer) at the recipe's
`difficulty` (`easy` to `impossible`, default `normal`, checked in memory), plays the steps and
writes `<out>/<batch>.md`, the batch's results template filled in (a note at its top says how it
was played), and `<out>/steps.jsonl`, each step's observation. E08 batch 2 (41 steps) takes about
three minutes, the check about two.

A recipe is the spec's procedure as `[[step]]`s. A hero's step selects the hero (`hero`, its name
on the map; from the hero list, scrolled when the player has more than five) and, with `walk =
[x, y]` (or `[x, y, level]`, 1 the underground: the view switches by the panel's level button),
clicks the tile once, reads the path's marks from memory (on both levels), clicks it again and
waits until the hero, its points and the players' resources rest, on the way:

- **answering questions** (a monster's offer to join, to join for gold, to flee; a guarded pile's
  question): a dialog with both an OK and a Cancel button; its text, read from the game's memory,
  is named by the record it was written from (`h3lod`: `ADVEVENT:86` join, `88` or `89+90` join
  for gold, `91` flee; anything else `other`), and `answer` says `yes` or `no`, for every question
  or per kind (`answer = { join = "no", flee = "yes", other = "no" }`). A question the step does
  not answer stops the batch.
- **playing battles**: the battle screen; its stacks are read at the start and the end, and
  `battle` says how: `auto` (auto combat: the computer plays every human side to the end),
  `retreat` or `surrender` (the attacker; `defender retreat`, `defender surrender`: every other
  stack defends until one of that side's acts, which then retreats or surrenders, the
  confirmation's OK clicked). The result window's OK (`[button.battle_ok]`) ends it. A battle the
  step does not plan stops the batch.
- **choosing a level-up's skill**: the dialog whose OK stays disabled (`[button.ok_disabled]`)
  until one of its two offered skills is clicked: `level_up = "left"` (the default) or `"right"`.
- clicking any other dialog's OK.

A step without `walk` only reads. `record` fills the template's fields and table rows from the
observation; `before:<observation>` reads the same before the step:

| Observation | What |
|---|---|
| `arrived`, `moved` | yes or no: the hero stands on the tile; it left its tile |
| `mark`, `marks`, `points`, `points_if_moved` | the target's path mark at the first click (green, red, none), the marks, the points left (if it moved) |
| `resource:<name>[:<colour>]` | a resource of the hero's player (or of `red`, `blue`, …) |
| `difficulty`, `date` | the game's difficulty; the date (`month 1, week 1, day 2`) |
| `heroes:experience` | the heroes off the map (E3-7): `147 heroes: 40 to 90; 2 at 40, 1 at 90` |
| `level_ups` | the level-up dialogs passed |
| `question`, `questions` | the questions' kinds (`join, flee`; `none`), their records |
| `battle`, `winner`, `retreated`, `surrendered` | yes or no; the side (`attacker`, `defender`, `none`) |
| `stacks:start`, `stacks:end[:<side>]` | the battle's stacks (`attacker 20 Archangels \| defender 3 Pikemen, 2 Halberdiers`) |
| `hexes:start[:<side>]` | their hexes at the start (the formation) |
| `hero:<detail>`, `hero[<name>]:<detail>` | the step's hero after the step, or another by name: `experience`, `level`, `primary` (`3/5/1/2`), `army`, `skill:<name>` (none, basic, advanced, expert), `on_map`, `tile` |
| `monster:x,y`, `aggression:x,y`, `never_flees:x,y` | the monster on a tile (`30 Pikemen`, `none`), its aggression (−4 to 10) and flag, from the tile's word |

`expect = { marks = <n>, path = "green", question = "join, flee", battle = true, dialogs = true }`
notes a difference under the encounter's "anything odd" (`dialogs = true`: dialogs passed after
the walk are expected). `end_turn = true` ends the turn. Everything the record holds comes from
the game's memory, not from the screen. `compare` prints each record line of two filled templates
as the same or different (random amounts differ by nature: judge them by their ranges).

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
| Battle | park the cursor over the panel, beside the log's item (M3 T4b-2b-1) | (618, 593) |
| Battle | a hex *h* (row r, column c): move, shoot, attack from the side the cursor points | (36 + 44c + 22 if r is even, 112 + 42r): `hex:<h>` |
| Battle | attack hex *h* from its right neighbour (M3 T1a) | its point + (18, 0): `click:x,y` |
| Battle | wait, defend | keys `w`, `d` |
| New game menu | Multiplayer | (640, 185) |
| Multiplayer dialog | Hot Seat | (580, 152) |
| Hot-seat names | name *i* (0 to 7; the first has the focus) | (400, 186 + 30 *i*) |
| Hot-seat names | OK | (340, 444) |
| Any dialog | OK: found by `[button.ok]` | its corner + (33, 16) |
| Adventure map | the hero list's row *i* (0 to 4) | (638, 227 + 32 *i*) |
| Adventure map | the hero list's arrows: up, down (one row each) | (638, 203), (638, 379) |
| Adventure map | the surface or the underground (the level button) | (725, 211) |
| Level-up dialog | its two skills: left, right | the disabled OK's corner + (−150, −64), (−52, −64) |
| Adventure map | tile (x, y) in the view at (vx, vy) | (24 + 32 (x − vx), 24 + 32 (y − vy)) |
| Adventure map | the minimap (72 × 72 map): centre the view on (x, y) | (630 + 2x, 26 + 2y) |
| Adventure map | end the turn | key `e` |
| Adventure map | park the cursor over the right panel | (700, 300) |
| Scenario list | the players' difficulty *i* (0 Easy to 4 Impossible) | (521 + 32 *i*, 478) |
| Any question | Yes, No: found by `[button.ok]`, `[button.cancel]` | their corner + (33, 16) |
| Battle | auto combat, retreat, surrender, defend | keys `a`, `r`, `s`, `d` |
| Battle's result window | OK: found by `[button.battle_ok]` | its corner + (32, 16) |

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

- The game draws its own cursor: the recipes park it (the battle's at (618, 593) since M3
  T4b-2b-1, beside the log's item, which the set compares), and probes stay clear of its picture
  where it rests (the battle's nearest, 34 px right on its row and 15 px above it).
- `machines`' timeline (M3 T3b) varies between runs: the automatic Ballistas' shots, and the
  heal after one, start 1 ms later than recorded in most runs (M3 T4b-2b-1, 8 runs, at either
  park point); a `CHANGED` there with only such shifts is that, not a change of the viewer.
- The battle panel's buttons change with whose turn it is: the `battle` probes are the ones equal
  in both states.
- A click's hover leaves the status line's hint in the log's item (`Attack Pikemen` over a
  target) until a log line replaces it, and a line held there less than 3 000 ms keeps hints out
  (design §9.4, §9.6): compare the log's item only from a sequence's `log_at`.
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
  (a meeting): heroes are selected from the hero list only. The list holds the heroes by their
  type's number, not in the map's order; with more than five the harness scrolls it to its top
  first, then down to the hero's row (H2).
- The panel's level button changes with the map (enabled on a two-level map) and the level shown:
  the `adventure` probes leave it out (H2).
- The adventure panel takes the current player's colour: the `adventure` probes are the ones
  equal on Red's and Blue's turn (`adventure_map`, Red's, stays for the battle's recipes).
- A player without towns gets a message at the end of its turn and at the start of its next
  ("you have lost your last town"); `dismiss` and `end_turn` click every OK.
- The adventure map walks in real time (the clock runs); a walk is over when the hero, its points
  and its player's resources rest for 1.5 s. Keep heavy builds off the machine while a batch runs.
- Auto combat plays a human side's creatures only with the setting `Combat Auto Creatures` on
  (`setup.sh` sets it; a new prefix holds 0, and then the button only sets a flag and the turn
  still waits: 0x474CDF). The capture set never turns auto combat on.
- A player left without heroes and towns is out, and with one human left the game is won: the
  Hall of Fame shows and the batch stops (`neither the adventure map, a battle nor a dialog`). An
  experiment map keeps each player a hero after its battles (the check gives Blue a second one).
- A player without towns gets "you have lost your last town" after each battle too.
- The dialog's text is read from the topmost dialog's text items (design §6.3), not from the game's
  message buffer (0x6973D8), which a dialog with a map's own text may not touch.
- The game's memory can lag its screen by a step: after an animation it starts the next turn in
  the same millisecond after the 20 clock reads the driver waits for (design §7.9). A state read
  then is one step behind; the frame check draws the viewer's own timeline for that reason.
- Inside an animation the memory holds the frame drawn, the screen the one shown (design §7.1):
  never render the memory's poses as the screen at that time.
- The battle runs one idle step for a clock step of 100 ms or more (its next time moves to now,
  0x496820), so a jump to a frame far on would show too few steps (the outline's pulse, the
  flags, a hero's idle motion): `capture` steps at most 100 ms at a time (M3 T3a).
- In a battle in progress (M3 T1a) the game names a striker the active stack again while it
  turns back after its attack, so `turn:` waits for the stack named, not for any active one; a
  click before the turn starts is lost, and `click:` parks the game's cursor afterwards (the game
  draws it into the frames).
- Under load (a build beside it) a sequence once read the computer's first frame a millisecond
  late (`melee`: 1919 for 1918, `CHANGED`): the turn's start ran after the 20 clock reads the
  driver then waited for. A sequence's steps now wait for 500 reads; still, run `sequences` and
  `frames` on an otherwise idle machine and rerun a one-line change before believing it.
