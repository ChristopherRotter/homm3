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
| `xdd.c`, `xdd.def` | Our `xdd.dll`: forwards the GOG executable's `DirectDrawCreate` to Wine's ddraw, the **virtual clock** and the **forced rolls** (below). |
| `h3drive.py` | The driver: starts the game, the recipes (fixed coordinates, screens by probes), the clock, dumps; the commands `battle`, `set`, `learn`, `shot`, `sequence`, `sequences`, `frames`. |
| `screens.toml` | The screens the driver recognises: 12 probes `[x, y, 5-6-5 value]` each (numbers only). |
| `h3cap.py` | `diff` (our render against a capture, with masks) and `find` (the original's obstacles in a capture); `compare` and `boxes_at` for the frame check. |
| `h3mem.py` | Reads the running game's state from its memory: the battle's clocks, idle steps, active stack and the mouse's highlight, each stack's group, frame, hex, offsets and draw layer, and each side's hero's group and frame and its flag's frame (M3 T3a), the magic animation drawn and the stacks under it (M3 T3b), the battle's outcome (H1); the adventure map's players, heroes (with their experience, level, army and skills: H1), view and path, the difficulty, a tile's monster, the topmost dialog's text (H1; below). |
| `h3adv.py` | The adventure map (M2 T6, H1, H2): hot seat at a chosen difficulty, heroes (the list scrolled), tiles of either level clicked twice, questions answered, battles played, level-ups chosen, turns; `batch` plays an experiment batch and fills its results template, `compare` sets two filled templates side by side, `learn-button`. |
| `h3lod.py` | The original's text tables, read from the run directory's LODs at run time (H1): which record a dialog's text was written from (`ADVEVENT:86`); never their text. |
| `batches/` | The batches' recipes: a spec's procedure as steps (`E08-batch-2.toml`), and the harness's own check (`harness-check.toml`, its spec in `checks/`). |
| `../situations/` | The capture set: `set.toml` and its situations' scenario files; the recorded sequences, `sequences.toml`, their scenarios `seq-*.toml` and `timelines/`. |
| `h3-view situation` | Writes the original's map for a scenario's battle (`cargo run --release -p h3-view -- help`). |
| `../situations/kept/` | What the last capture of each sequence (`sequences/<id>.toml`) and situation (`set/<id>.toml`) kept: its frames' times and the fidgets and heroes the original's memory showed (numbers only), written by `frames` and `set`; and `drawn.txt`, the digests of the viewer's frames at those times (`drawn --record`). |

Before the first run: stage the copy's files (`Heroes3.exe`, the five DLLs `BINKW32.DLL`,
`IFC20.dll`, `MP3DEC.ASI`, `MSS32.DLL`, `SMACKW32.DLL`, `Data/` with the four LODs, `Heroes3.snd`,
`H3ab_ahd.snd`, `H3ab_ahd.vid`; one large file per staging call), run `clients/capture/setup.sh`,
and build `cargo build --release -p h3-view` in `clients/`. A run of the whole set (31 situations since M3 T4a) takes about 4 minutes.

## Which captures a change needs

Capturing the original is slow (all ten sequences and the set: about 80 minutes in a fresh
sandbox, which keeps no frames of an earlier one), so a task captures only what its change can
alter (the user, 2026-10-20):

1. **Always, without the original:** from `clients/`, after `cargo build --release -p h3-view`,
   `python3 capture/h3drive.py drawn` (about 3 minutes; no Wine, only the user's original files
   for the assets). The viewer draws every sequence and situation at the times `kept/` holds and
   compares each frame's digest with `kept/drawn.txt`; its timeline check (`h3-view sequences`)
   runs with it. The digest is of the frame's pixels outside the fixed masks its comparison
   applies (a situation's `mask`; a sequence's none since M3 T6c-2, its parked cursor from T6b,
   its panel before; E17 M3 T6a): a frame whose digest is the same shows the comparison what the
   last capture compared, so its recorded count still holds.
2. **Capture what changed:** `frames --only <id>` or `set --only <id>` for each one `drawn` lists
   as `CHANGED` or `new`, and for each sequence or situation whose entry the task adds or
   changes (scenario, actions, `ms`, `boxes`, masks: the kept times or the comparison change
   with it). Record their counts as before.
3. **Then** `drawn --record`, and commit `kept/` and `drawn.txt` with the task (the captures
   refresh `kept/` themselves).
4. **Everything** (`frames` and `set` without `--only`): at a milestone's closure, and after a
   change to the harness (`capture/*.py`, `xdd.c`, `setup.sh`, `screens.toml`) that can change
   the original's side: how it is driven, captured or read. A change to `drawn` alone (how the
   viewer's frames are digested) cannot: there the digests are recorded again from the last
   captured state, `drawn --record` with the viewer that state was compared with (E17 M3 T6a: a
   worktree of `master`), before the task's own change.

`drawn` reporting every one the same (a change that reaches no reference frame) needs no Wine
setup at all. Its digests are of the PNG files' pixels (`h3_render::png`'s unfiltered rows): a
change of the viewer's PNG encoder's format shows every frame as changed; then `drawn --record`
once, after `frames` and `set` passed.

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
frame (`grass t=0: 0 differing (recorded 0) ok`); exit 1 if any count changed. The mouse rests
where the situation's `mouse` says (M3 T6c-1; the harness moves it there at the opening, the clock
frozen; default the parked point): `frames.toml` names it, and the viewer draws its cursor, the
status line's hint and the stack it marks (design §9.9). Take such frames from 100 ms: at the
opening itself the original shows the new cursor where the mouse was.
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
artifacts (`attacker_war_machines`, `defender_war_machines`; M3 T3b), without other artifacts;
its skills only the school skills of its masteries (below) and those behind its `effects` (M3
T7b-2b-2a: a morale or luck change of 1 to 3 to its own stacks becomes Leadership or Luck at that
level, the pack's skill level whose effects are that change alone; any other effect is refused);
an entry's `hot_seat = true` (M3 T3b) starts it in hot seat, so that a faster stack of
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
(`hex:<n>` clicks a hex at `hex_point`, `key:d` defends, `click:x,y`, `move:x,y`, `step:<ms>`,
`cast:<k>[:<hex>]` casts a spell from the hero's book, below) and
then steps 1 ms at a time, reading the game's state after each step: `<out>/states.jsonl` (every
state, with `--screen` a digest of the battlefield's pixels) and `<out>/timeline.txt` (one line per
change of a stack's group, frame or hex, by the original's records: each side's army, then its
war machines; after them a change of the magic animation drawn, `<ms> magic <number> f<frame>
<side>.<record>…` or `<ms> magic -`, M3 T3b; then `<ms> end` once the battle is decided, M3 T8:
the manager's +0x132F8, when the result's window opens and its tune starts, `end`). `sequences`
replays the recorded ones and fails when a timeline changes; add one like a situation (its
entry, `new`, then copy its timeline into `timelines/`).

**Casts** (M3 T7a-2a, design §7.16): a scenario's hero gets a spell book with `spell_book`
(the engine's keys) and `mana` 10 per `knowledge` (the original's hero starts so; `h3-view
situation` refuses other mana, writes the book into slot 17 and numbers its spells by
`sptraits.txt`'s names). `cast:<k>[:<hex>]` opens the book by its key C, clicks its place `k`
(0 to 11: row by row, the left page first; the first opening shows the combat spells of all
schools, sorted by level, then school, then name) and then the target hex if the spell takes one.
A Faerie Dragon casts its drawn spell by a click on its target (`hex:<n>`: the cursor's cast).
`--heroes` (an entry's `heroes = true`) adds each change of a hero's group or frame to the
timeline (`<ms> hero <side> g<group> f<frame>`): a hero's cast. `--screens` (an entry's
`screens = true`, M3 T7a-2b) ends each time whose screen differs from the step before with
`<ms> screen <x0>,<y0>,<x1>,<y1>`, the box of the pixels that changed (x1, y1 exclusive; numbers
only): the magic animations memory does not show (the at-hex and Armageddon players, the
projectiles, the rays, the Liches' cloud). The screen shows a frame a deadline after memory
holds it (design §7.1: a 100 ms animation's frame drawn at 3 400 shows at 3 500). Such a run
dumps every step, about 25 ms a step (a 9 s sequence takes 6 minutes). A key pressed at the
frozen clock right after a hero's cast acts once the cast has played (its stack defends as the
hero stands). A
scenario's `masteries` become the hero's school skills on the original's map (`situation`: Fire,
Air, Water, Earth Magic at the level the masteries ask; a mass spell at Expert). A Fire Wall or
Force Field beside an obstacle is refused (the book closes, no cast): aim at open ground. The
casts' sequences, recorded at T7a-2a and T7a-2b (`dragon`, `bolt`, `mass`, `raise`, `obstacles`,
`creatures`; design §7.16, "Recorded"), waited in `situations/casts.toml` until the viewer's
timeline played them; all are in `sequences.toml` since T7c-3, which removed that file (`cast`,
`resist` and `arrow` moved at T7b-1, `haste` and `dispel` were recorded at T7b-2a). A recorded
timeline's `screen` lines are left out of the timing check (they are the frame check's).
Chain Lightning's rays turn by the original's draws: `xdd.dll` logs them (the `ray_aim` site,
"Logged draws" in `xdd.c`) and `frames` keeps them in the frame file's `rays`, so the viewer
draws the same rays in the frame check (T7c-3). A stack whose hero just
cast acts by `key:` only: a `turn:` naming the stack already active passes at once, so pick the
speeds so the turns alternate (`haste`'s Vampire Lords, speed 9).

**A battle in progress** (M3 T1a): `--hot-seat` (an entry's `hot_seat = true`) starts the map in
hot seat with Blue a human too (`h3-view situation --hot-seat`), so the actions play both sides and
nothing acts on its own. Among the actions, `turn:<side>.<record>` steps 1 ms at a time, each step
read, until that stack's turn starts (the game names it the active stack), and the actions after
it act then; its time goes to `<out>/turns.txt` (`at:<ms>` steps so up to a time; since M3 T7a-2a
a `turn:` that waits more than 30 s of game time fails, naming the active stack; since M3 T7a-2b
it first writes the states and the timeline so far). Act as the turn
starts: a player who waits lets fidgets start, which the viewer's timeline does not have. In the
entry's `[[sequence.command]]`s, `at` gives each player's command its time from `turns.txt` (`ms`
counts from the last one). `behind` (`seq-behind.toml`): two rounds, 8 commands, about 16.5 s
(to the Marksmen's second shot's end since M3 T5b-2a); `shots` (`seq-shots.toml`, M3 T1b-1): two
rounds, 6 commands, three rays, about 10 s; `fly` (`seq-fly.toml`, M3 T1b-2): two flights
across rows and a breath, about 9 s; `machines` (`seq-machines.toml`, M3 T3b): the war machines
act on their own after the Archers' shot (two Ballistas' shots and the tent's heal), about 8.5 s; `strike` (`seq-strike.toml`, M3 T4b-2b-2):
the Archangel's fixed damage kills the Pikemen, about 4.3 s, its log line compared (below). A war machine without Artillery or First Aid
needs no action (the engine plays its turn inside the command before it). A two-hex striker
stands where the cursor's slot inside the target's hex says (design §7.13): `click:x,y` with an
offset from the hex's point picks it. `hover` (M3 T1b-2) moves the mouse over a stack
(`move:x,y`) and away; it has no engine command: since M3 T6c-2 the viewer plays the opening's
battle with the harness's mouse over it (design §9.10), and the timeline check leaves out the
original's own fidgets (those of the stacks the mouse did not enter). Every sequence's actions
are the viewer's mouse too (`h3-view sequences` reads `actions`: the mouse parked first, `hex:`
and `click:` move, click and park it again, `move:` moves it, `key:` acts).

**The frame check** (M2 T4b-1, design §7.9): from `clients/`,

```
python3 capture/h3drive.py frames [--only <id>] [--out <dir>] [--reuse]
```

For each recorded sequence: its battle in the original, stepped 1 ms at a time, keeping every
distinct screen (its battlefield and, since M3 T6b, its panel, since T6c-2 its cursor: the
mouse poll's next time 0x69CA18 set to the opening's, so the cursor moves on the same 33 ms grid
in every run, design §9.10;
`<dir>/<id>/frames/t<ms>.npz` and their states in `frames.jsonl`, the log's item and entries
among them: `h3mem`'s `battle_log`); the viewer's frame at each of those times (`h3-view
sequences <id> --frames <dir>`: `ours/`, what it drew in `ours.txt` and its log item's text in
`ours-log.json`, with the heroes the original showed: their defeat and victory come from its
generator; the flags are the viewer's own; its cursor where the original's memory says the
cursor is drawn, `frames.toml`'s `cursor`, M3 T6c-2); the masked diff of each pair (the count
boxes where the drawn poses show them; the heroes are compared since M3 T3a, the panel since
T6b, the cursor since T6c-2), written to `frames.txt` and, for a frame that differs, `diff/t<ms>.png`; each line
compared with the sequence's recorded `timelines/<id>-frames.txt` (`ok`, `new`, `CHANGED`; exit 1
on a change). The log's item (M3 T6b, design §9.8) is compared where the original's shows the
text the viewer drew; where it shows another (a line whose numbers the engine rolled otherwise;
the status line's hints are the viewer's since M3 T6c-2) and its pixels differ, the item is
masked and listed at the end of the frame's line, ` log`. An entry with `boxes = true` (M3 T5a) compares the count boxes; any other
masks them where the drawn poses show them and where the viewer drew them (`ours.txt` lists the
boxes it drew, ` box<x>,<y>n<count>@<side>.<record>`). With `boxes`, a box whose count is not the
one the original's memory shows (`h3mem`'s `shown`, +0x50, while it is not −1, else its `count`;
M3 T5b, design §4.3: the engine rolls its own damage) is masked and listed at the end of its
frame's line, ` n<side>.<record>`. Each line is `<ms> <differing> <artifacts>` (M3 T1c): the differing
pixels outside the masks and how many of them are the original's **redraw artifacts**
(`h3cap.redraws`, design §7.11): its pixel is ours at that moment with one to five more
shadows, or one we drew there at an earlier moment of the sequence with at least one more. The
diff PNG marks those yellow, any other difference red; a change in either count fails. The
original's frames live only in the sandbox that captured them (`clients/out/`, never in git):
a new sandbox captures them again (all nine with frames, about 60 minutes: `shots`, `fly` and `machines` about 9 each) before `--reuse` works. `--reuse` keeps the original's frames from an earlier run and only redraws
and compares (after a change to the viewer). About 14 minutes for the first four sequences and 20
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
three minutes, the check about four (E08 T32b1).

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
  until one of its two offered skills is clicked: `level_up = "left"` (the default) or `"right"`,
  or a list of them, one per such dialog in order (the last one repeated; E08 T32b1). Every
  level-up dialog, with two offers, one or none, is recorded from the game's memory (E08 T32b1):
  the hero (its name in the dialog's text), its new level, the primary skill raised and the
  offers, read from the dialog's picture items (`level_up_dialog` 0x4F8880: item 2009 the primary
  skill's frame, 2010 the left offer or the only one, 2011 the right one, a skill's frame 3 ×
  skill + 3 + the hero's level of it before), and the one taken.
- clicking any other dialog's OK, its text's record noted (`messages`; E08 T32b1).

Since E08 T32b1 `battle` may also be `attack`: every stack attacks the other side's nearest one
(creatures before war machines) until the battle ends, as a player would: the mouse rests over the
target's hexes, off their centre toward each free neighbour in turn, until the combat cursor shows
a melee attack (its sword frames 7 to 14), and clicks there; a target out of reach this turn (the
hex records' attack mark, +0x4A) makes the stack move to the free hex nearest it within its speed
(a flyer's reach). The war machines act by themselves. Or a plan (batch E4's E4-9): `{ attack =
"Ballista", by = "defender", then = "attack" }`: while the other side has a living stack of that
creature, each stack of the side named attacks it and every other stack defends; then `then`
(`attack` or `auto`) plays the rest. The battle's record holds its field (`battlefield`).

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
| `level_up:<n>:<part>` | the step's n-th level-up dialog (from 1): `level`, `primary` (attack, defense, power, knowledge), `left`, `right` (a skill's name, `none`), `taken` (`left (Wisdom)`, or the only offer), `hero` (E08 T32b1) |
| `messages` | the records of the other dialogs closed with their OK (`GENRLTXT:79, …`, `?` for a text not identified, `none`) |
| `battlefield`, `battlefield:terrain` | the last battle's special field (`Clover Field`, `none`) and its terrain (`grass`) (battle manager +0x53C0, +0x5394) |
| `destroyed:<creature>` | yes or no: no stack of that creature that was in the last battle at its start stands at its end (`destroyed:Ballista`) |
| `question`, `questions` | the questions' kinds (`join, flee`; `none`), their records |
| `battle`, `winner`, `retreated`, `surrendered` | yes or no; the side (`attacker`, `defender`, `none`) |
| `stacks:start`, `stacks:end[:<side>]` | the battle's stacks (`attacker 20 Archangels \| defender 3 Pikemen, 2 Halberdiers`) |
| `hexes:start[:<side>]` | their hexes at the start (the formation) |
| `hero:<detail>`, `hero[<name>]:<detail>` | the step's hero after the step, or another by name: `experience`, `level`, `primary` (`3/5/1/2`), `army`, `skill:<name>` (none, basic, advanced, expert), `on_map`, `tile`; since E08 T32b1 `skills` (`Leadership expert, Wisdom basic`, in the hero's order), `seed` (the level seed, hero +0x8F), `wisdom_offer` and `school_offer` (the levels Wisdom and a school were last offered at, +0x90, +0x3F), `artifacts` (worn: `Ballista (war machine 1), …` in slot order) and `backpack`; since E08 T32b2 `equipment` (the worn artifacts but the war machines and the spell book: a starting bonus's) |
| `monster:x,y`, `aggression:x,y`, `never_flees:x,y` | the monster on a tile (`30 Pikemen`, `none`), its aggression (−4 to 10) and flag, from the tile's word |

`expect = { marks = <n>, path = "green", question = "join, flee", battle = true, dialogs = true }`
notes a difference under the encounter's "anything odd" (`dialogs = true`: dialogs passed after
the walk are expected). `note = { <label> = <observation> }` (E08 T32b2) reads as `record` does
and adds `<label>: <value>` under "anything odd" unless it reads `none` (batch E4: a hero's
`equipment`, the starting bonus's trap). `end_turn = true` ends the turn. Everything the record holds comes from
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
| Battle | the hero's spell book (M3 T7a-2a) | key `c` |
| Spell book | its place *k* (0 to 11) | (246 + 86 (k mod 2) + 216 ⌊k / 6⌋, 123 + 96 ⌊(k mod 6) / 2⌋): `cast:<k>[:<hex>]` |
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
`<seq> <time> <reads>`, the reads of the clock since the command (then the forced rolls'
three numbers, below). A game that keeps reading a stopped clock is idle at that time:
`Game.clock` waits for 20 reads, `Game.settled` for two equal dumps (Wine may draw a moment
later).

## Forced rolls

*Provisional* (E17 M3 T7b-2b-1, the user's wish: the effects that need a roll of the original's,
reproduced at will). At its load `xdd.dll` redirects the call of each roll in its table
(`sites`; `docs/exe/map.toml`'s `*_draw` entries) to its own function, which draws as the call
would (`random_between` 0x50B230 or the C library's `rand` 0x617E5C: the generator's sequence
stays the original's) and returns that draw unless the harness forced it. Only a call whose bytes
are the one named is redirected; that and the clock's import slot are the only bytes of the
running copy that change, never a file. Commands in `ctl`: `<seq> roll <name> <value> <count>`
(the next `<count>` draws return `<value>`), `<seq> rolls <name> <value>` (every other draw does)
and `<seq> rolls <name> natural` (they are drawn again, as at the start). `ack` goes on with
`<forced> <pending> <sites>`: the draws forced so far, the forced draws still waiting and the
calls redirected.

| Roll (`ROLLS`) | Draw (falls, fails) | Where, when |
|---|---|---|
| `good_morale` | `random(1, 24) <= morale` (1, 24) | 0x4649D3, after a walk, attack, shot or creature cast |
| `bad_morale` | `random(1, 12) <= -morale` (1, 12) | 0x464BC7, at a pick (none at the battle's first); a computer side's let-off draw is not forced: hot seat has none |
| `fear` | `rand() mod 10 = 0` (0, 1) | 0x464DF1, at a pick, beside Azure Dragons |
| `luck`, `shot_luck` | `random(1, 24) <= luck` (1, 24) | 0x441837 as a melee strike starts, 0x43F955 a shot; only with luck above 0: the hero's `effects = [{ kind = "luck", amount = 3 }]` (Expert Luck on the map, T7b-2b-2a) |
| `death_stare` | `random(1, 100) <= 10` (1, 100) | 0x440EE0, once per Mighty Gorgon: a creature killed each |
| `thunderbird` | `random(1, 100) <= 20` (1, 100) | 0x44119D, the Thunderbird's bolt |
| `acid` | `random(1, 100) <= 20` (1, 100) | 0x4414B7, the Rust Dragon's acid damage (its amount drawn after, not forced) |
| `poison` | `random(1, 100) <= 30` (1, 100) | 0x440839, the Wyvern Monarch's Poison |

A sequence's action `roll:<name>` forces the roll's next draw to fall, `roll:<name>:fail` to fail;
its key `fail = ["<name>", …]` (`--fail <name>` for `sequence`) fails every other draw of those
rolls, so nothing falls the sequence did not ask for (the runs' differences in "Traps" among
them). Each draw is made for every stack at its moment whatever its morale or luck (a morale of 0
draws and never falls): a forced draw goes to the next stack that rolls, so `roll:` comes right
before the action it is for (a pick's: before the action that ends the turn before). A sequence
fails when xdd.dll redirected fewer calls than the table has, or when a forced draw was left
untaken at its end. The viewer's mouse (`pointer::inputs`) takes `roll:` as no input; the engine's
own rolls follow its `seed`. Checked at T7b-2b-1: the Champions' walk (`seq-walk.toml`) with
`roll:good_morale` got another turn and animation 20 at 2 592 ms, failed they gave the Marksmen
theirs; an Azure Dragon's fear (`rand`) forced over the Champions (animation 15) and failed;
`walk`, `strike`, `shot` and `resist` replayed with the redirected calls as recorded. Checked at
T7b-2b-2a: `seq-shot.toml` with Orrin's `effects` luck 3 and morale 2 (Expert Luck, Advanced
Leadership on the map): the original's Marksmen and Pikemen at luck 3 and morale 3 (the one town's
1 too; `h3mem`'s `luck`, `morale`), as the engine gives them; `roll:shot_luck` played good luck's
animation 18 over the Marksmen before their shot and its line, failed (with `good_morale`) the
shot's timeline was the recorded `shot`'s. Recorded at T7b-2b-2b (`sequences.toml`, each with
`fail` on the rolls it forces): `luck` (`strike` with Orrin's luck 3: a lucky melee strike after
the Archangel's flight), `morale` (`shot` with luck 3 and morale 2: the first shot lucky, good
morale after the second), `bad` (bad morale in an army of three towns, at the pick after the
Champions' defend) and `fear` (an Azure Dragon's Fear at the Champions' pick); about 3 to 5
minutes each. Their `seed`s were searched so the engine's own rolls fall as the forced ones did:
for each seed, `h3-view sequences <id>` and its record's events (`out/sequences/<id>.jsonl`:
`lucky` strikes, `GoodMorale`, `BadMorale`, `Frightened`), the first seed that matches kept. The
original's idle fidgets go on after a freeze (`bad`'s Goblins, `fear`'s Marksmen): the timeline
check leaves them out as any. Recorded at T7b-2b-3 (about 3 to 4 minutes each): `thunder` (the
Thunderbird's bolt, Life Drain on the strike back), `acid` (Acid Breath, the acid, Poison on
the strike back), `stare` (Death Stare on the strike back) and `lightning` (`bolt`'s round 1 in
`sequences.toml`). From column 1 the defender's column 15 is twelve hexes away for a two-hex
attacker (thirteen for a one-hex one): a slower striker flies within reach in round 1 while the
other side defends (`turn:`, `key:d`), then strikes in round 2. `hex:<n>` moves a two-hex
attacker's front to n, so its engine `Move` goes to n − 1 (its rear, the position). Each search
for an engine seed (a loop over `h3-view sequences <id>` with the release binary, about a
quarter second a seed) also matched the original's damage and kills where it could (its log in
the recording's last state).

## The original's state

`h3mem.py` opens the game's `/proc/<pid>/mem` (Wine maps `Heroes3.exe` at its image base, so the
map's addresses hold): the frame deadline and the idle step's time, the combat speed, the mouse
(its hex, the cursor's kind, type, frame and the point it is drawn at: M3 T6c-2), and per
stack its creature, hex, group, frame, facing, offsets, times, count, the count its box shows
(`shown`, M3 T5b), its morale and luck (M3 T7b-2b-2a) (`docs/exe/structs.toml`). On
the adventure map (M2 T6): the current player, each player's selected hero, hero list and
resources, every hero's name, tile and points, the view's top-left tile, the tile under the
mouse and the selected hero's path (design §6.3). It only reads, but for the mouse poll's timer
a sequence's opening sets (M3 T6c-2). `python3 capture/h3mem.py`
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
  target) until a log line replaces it or a human's turn starts (the mouse posted again over
  nothing), and a line held there less than 3 000 ms keeps hints out but an attack's (design
  §9.8, §9.10): the viewer follows the harness's mouse since M3 T6c-2.
- The game reads the mouse on a 33 ms grid of its clock and draws a new cursor frame where it
  last read it (design §9.10): the harness sets the grid's next time at a sequence's opening
  (`h3mem.Mem.align_mouse_poll`, its only write into the game's memory, a timer of the mouse
  manager); without it the frames kept where the cursor moves change their times between runs.
- Entering a stack redraws only its new frame's rectangle: the old frame's pixels outside it
  stay until the next idle step (`hover`'s four frames of 1 and 13 pixels, design §9.10).
- The game draws its cursor at an even x (0x50CE04): at an odd point the canvas is a pixel
  left of the point less the hot spot (a drawing bug: the comparisons draw it so, the client
  does not; design §9.9).
- The game's memory can lag the screen by a step as a turn starts (below): the log's text read
  then may be the one before; the frame check lists the item only where its pixels differ too.
- A hero's reaction may be read a step late at its first frame: one of two runs of `strike`
  (M3 T5a) read the left hero standing at 3 730 ms while its screen showed its reaction (group 3)
  (2 400 pixels), one of two in T5b-1 its frame 0 while the screen showed frame 1 (2 152); run
  again, as recorded. A `CHANGED` with differences only on a hero at the frame its reaction
  starts is that, not a change of the viewer.
- The original's rolls can differ between runs (M3 T5b-2a, not explained: the scenarios hold
  morale and luck at 0): of three `strike` runs one left a Pikeman alive (9 killed, then their
  strike back) and one gave the Archangel good morale at 3 730 ms (magic 20); one `machines`
  run played magic 18 over the Archers as they shot; in one `climb` run the computer's stack
  walked at 2 396 ms. Run again: a `CHANGED` whose timeline shows such a roll or move is that,
  not a change of the viewer. A sequence's `fail` (M3 T7b-2b-1, "Forced rolls") keeps morale,
  luck and the abilities' rolls from falling; the damage rolls stay free.
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
- Auto combat does not play a battle between two human players in hot seat (E08 T32b1: neither
  its key nor its button did anything on a human stack's turn): a hero against another player's
  hero is played by `attack` or a plan.
- A game's first battle takes orders later than its later ones: a key before a stack's turn is
  lost (E08 T22a3: batch E3b stalled at its first battle in every run). `fight`'s auto combat
  waits for an active stack and presses again after a 10 s stall.
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
