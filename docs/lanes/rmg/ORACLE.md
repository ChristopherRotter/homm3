# RMG lane — the oracle

The original's random map generator run as the reference the port is compared with (E14 T1b1,
2026-11-23; decision 1: the same map per seed, byte for byte). The original runs under Wine in the
capture harness (`clients/capture/`, its runbook `README.md`), generates a map for given choices
and a given seed, and the oracle keeps the map's digests. **The maps are the original's output and
are never committed** (ADR-0009): they stay in `clients/out/rmg/`; the corpus's choices
(`clients/capture/rmg/corpus.toml`) and digests (`rmg/kept.txt`) are.

## How it runs

- **The seed.** The generator seeds the C library's `rand` with `time()` in seconds (0x536148);
  `crt_time` (0x6198E0, whose one caller is the generator's) reads KERNEL32's `GetLocalTime` and
  `GetSystemTime`. `xdd.dll` (the harness's DLL, `xdd.c` section 9) points `Heroes3.exe`'s two
  imports at a fixed date when `H3TIME` holds a number of seconds since 1970 UTC: with the runs'
  `TZ=UTC`, `time()` returns `H3TIME`, the seed. The executable's bytes stay as they are; the
  ack's seventeenth field says both imports are hooked. Every generated map prints its seed in its
  description (survey §1.6), and the driver checks it there.
- **The choices.** `h3rmg.py` plays the scenario screen as `h3shell.py`'s recipes do (a fresh
  start, the clock frozen from the game's first read): New Game, Single Scenario, the Random Map
  tab, a press per choice (size, levels, the human-or-computer players, their teams, the
  computer-only players, their teams, water, monsters; never Random, whose draws come from the
  game's earlier `rand` state, survey §1.1), Begin. One human, red (the screen's default seat);
  the towns stay the screen's (Random), drawn by the generator from the seed.
- **The file.** The run directory has a `random_maps/` folder (the shell's runs have none: E27 M3
  T5b recorded "Cannot create Random_Map.H3M in data directory." without it); the generator writes
  `random_map_1.h3m` there (gzip). The driver waits for a whole gzip stream, keeps it, checks the
  description's seed and choices, and stops the game. An OK box and no file means the original
  refused the choices (codes 1 to 3, survey §1.2).
- **Times:** a generation takes 2 to 3 s of real time whatever the size (XL with two levels
  included); a case about 9 s with the game's start; the corpus about 3 minutes.

```
cd clients/capture
python3 h3rmg.py one --seed 1000000000 --size M --levels 2 [--players 2 ...] [--again] [--before "<step>;..."]
python3 h3rmg.py corpus [--only <id>,...] [--twice] [--again] [--record]
python3 h3rmg.py read        # h3_map's reader over the corpus's maps (h3-map's original-data check)
python3 h3rmg.py templates   # each case's template predicted from either rmg.txt, against kept.txt
```

Setup: the client lane's staging set (`clients/capture/README.md`, "Before the first run"), then
`env -u TMPDIR clients/capture/setup.sh`. The tab's points are `adv-random`'s (E27 design §3.16).

## The corpus (21 cases)

`rmg/corpus.toml`: every size (S, M, L, XL), one and two levels (size indices 1 to 32), 2 to 8
players with and without teams, computer-only players with and without their teams, eight players
in all, water none, normal and islands, monsters weak, normal and strong, seeds from 0 to
2 147 483 647 (the last second `time()` gives) with three consecutive seconds for one set of
choices. Each `kept.txt` line: the seed and template the description names, the format, size,
levels, the decompressed map's length and the first 16 hex digits of the SHA-256 of the map and
of its gzip file.

## What it showed (2026-11-23)

- **Reproducible.** Each case from a fresh start in two runs of the corpus (and `m2-2p` in a third
  before them, the first map made): the same bytes, the gzip file's too (21 of 21). A second
  generation in the same game (the adventure map's System Options, Main Menu, the same choices):
  the same bytes (21 of 21) in T1b1's sandbox; not in T1b2's (below, "The phase oracle":
  `l1-4p-b`'s second map differs with that sandbox's environment). After another history before
  the generation (the Load Game menu opened first): the same bytes (3 water cases of 3). Another
  seed gives other bytes: the 21 digests differ, two cases of one seed and several of one
  template among them.
- **The memory-dependent draws** (survey §2.6: step 2's uninitialised town-type bytes, step 15's
  read past the cells) changed nothing in T1b1's runs. Wine's fresh memory is not Windows': the
  oracle cannot say what the bytes hold on Windows. T1b2 showed the draw 0x532A5B in step 2 and a
  second map that it changes (below); T1c classifies the quirk.
- **SoD's `rmg.txt`** (`H3bitmap.lod`'s, 53 templates; AB's `H3ab_bmp.lod` copy has 50) is the
  one the game reads: `h3rmg.py templates` predicts all 21 kept templates from SoD's table and
  misses 8 from AB's. The prediction confirms the reading of the template's draw: the templates
  kept by size index (islands halved) and by the zones' positions for the players and the players
  plus computers (survey §2.1), then the first `rand()` after `srand(seed)` modulo their count
  (0x549975; water not Random draws nothing before it).
- **A template's name holds a line break** as `rmg.txt` quotes it ("Meeting⏎in Muzgob"), and the
  description prints it so: the harvester of increment 2 reads quoted cells.
- **The plain writer as read** (survey §2.5): no 124 trailing zero bytes, every main town's type
  byte 0xFF, red named human in the description. `Untitled.h3m`'s three differences are the HD
  mod's, not the executable's.
- **The reader:** `h3_map` reads all 21 maps (the missing padding is accepted: no inbox item); 20
  import and validate in a game of their players; `l2-2p-c4-islands` is refused by TWN-SET-001
  (a player's main town not a town of its own): for T1c, beside survey §2.5's candidate that the
  generator writes the main town's entrance as its position.
- **The tab:** its first opening shows two levels; after a game it shows two levels again (the
  driver presses every choice anew either way).

## The phase oracle (T1b2, 2026-11-23)

**How it runs.** `h3rmg.py one|corpus --phases` sets `H3RMG`: `xdd.dll` (`xdd.c` section 10)
replaces `rand`'s two instructions after its seed's update (0x617E70) by a jump to its own code,
which logs the draw with its call site (the return address - 5) when it is on the generator's
thread between its `srand` (0x536148) and `rmg_write_map`'s return, and puts 18 calls through two
thunks: that `srand` (the log `rmg<g>.draws` opened in the run's clock folder, `seed <s>`),
`rmg_generate_map`'s (0x54C01C: `generate`; at its return the last dump and `generated <0|1>`),
the 15 steps' calls in `rmg_generate_map` (at a step's first call a `step <k>` line and the cells
as the steps before left them, `rmg<g>-<k>.cells` named by the last step that ran, 0 before step
1) and `rmg_write_map`'s (0x54C05B: `write`; at its return `end <draws> <other threads' draws>`).
Formats: `xdd.c`'s header, section 10; `h3rmg.read_cells` reads a dump. **The hooks run on their
own stack** and enter `rand` after its own call of its thread data: nothing they do writes the
game's stack below a call, whose bytes the generator reads uninitialised (step 2's temporary
zone), so a hooked run's memory is a plain run's. The check: the 21 first maps hooked are
`kept.txt`'s, and `l1-4p-b`'s second map hooked is the plain run's second map. A step that never
runs (7 without a junction zone, 11 with one level) has no dump; steps 9 and 12 draw nothing in
the corpus. About 20 minutes for the corpus with `--again --twice`; the dumps 0.06 to 2 MB each
(about 0.7 GB for the corpus's three runs, in `clients/out/rmg/`, never in git).

**Kept:** `clients/capture/rmg/phases.txt`, per case the draws, the log's digest and each step's
dump's (`corpus --phases --record`). For increments 3 to 5: the port's draws compared with the
log and its cells with the dump of the same step find a divergence at its step and draw.

**What it showed:**

- **Reproducible:** 21 of 21 logs and dumps the same from a fresh start (`--twice`); the hooks
  change no map (above).
- **The stream:** 21 454 (`s1-2p-dry`) to 500 780 (`xl2-8p-t4-islands`) draws a map, 3.13
  million in the corpus, step 10 the most (0x546154, `rmg_prototype_draw`, 1.43 million).
- **Against survey §2.6** (`h3rmg.py phases`): every draw site of the generator's range that drew
  is in §2.6's row of its step, in its order (step 1 per template zone its town type, then its
  placements, two more passes, then per zone 0x532B00 or none and 0x53BFC0; step 8 per mine
  0x545AA8, 0x54591A, its guard's three, 0x546154, 0x545D0C; ...). **Not in §2.6: a third of the
  stream** (1.03 million draws) comes from callees outside the range: the terrain painter's
  picture draws (`terrain_picture_draw` 0x5B3890 and its three kin of two picture sets: one or
  two per painted cell) in step 3 (21 of 21), step 6 (the islets and sealed shores, 12 of 21) and
  step 11 (8 of 8 two-level cases), and the road and river painter's (`road_river_picture_draw`
  0x4F9F00) in step 14 (21 of 21) and step 15 (the 16 cases with rivers). T1c reads them.
  Listed and never drawn: 0x537C89 (water Random; none in the corpus), step 4's far town sites
  (0x544DEC, 0x544E22, 0x545174: only in step 5), 0x546154 in steps 6 and 9 (no monolith Border
  Guard, no guard crossed).
- **0x532A5B draws in step 2** in 18 of 21 cases: once per level in most (the temporary zone, on
  the stack), 11 or 17 times with many water zones (operator new), never in `s1-2p-dry`,
  `m1-4p-t2` and `l1-8p-dry`.
- **A second map in one game can differ.** In T1b2's sandbox `l1-4p-b`'s second map is another
  map (132 431 bytes, `h3m=59ff5725…`), with H3RMG and without: its step 2 draws 0x532A5B five
  times, the first map's six, and the stream shifts from there. The environment's size alone
  moves it: a variable of 40 characters added to the game's environment makes the second map the
  first's, one of 2 000 does not (the heap's layout). The first maps stayed `kept.txt`'s in
  every run (with and without H3RMG, both paddings). The other 20 second maps are the first's.
- **Cell bits from memory.** `rmg_map_cell_reset` (0x530F10) keeps +0x14, +0x18, bits 30-31 of
  +0x24, bit 24 of +0x28 and bits 1-31 of +0x2C as the heap held them: 0 in a fresh start's first
  map; in a second map of the same game, for the S and M cases (their cells' memory reused), most
  cells differ there: +0x14, +0x18 and +0x28's bit 24 until step 6 writes them, +0x24's bits
  30-31 and +0x2C's to the end. No map changed by them. The dumps keep them as they are; `corpus
  --again` names them apart (`h3rmg.MEMORY_BITS`) and fails only on other differences.

## Not yet (T1c)

- The painters' draws read (their rules and pictures), the memory-dependent quirks classified
  (step 2's draw, the cell bits, step 15's read past the cells): T1c, with the user.
- The oracle covers the choices without Random, one human, the default towns; a human's town
  choice and more humans (the advanced options' seats) are not in the corpus yet.
