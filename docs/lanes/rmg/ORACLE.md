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
  the same bytes (21 of 21). After another history before the generation (the Load Game menu
  opened first): the same bytes (3 water cases of 3). Another seed gives other bytes: the 21
  digests differ, two cases of one seed and several of one template among them.
- **The memory-dependent draws** (survey §2.6: step 2's uninitialised town-type bytes, step 15's
  read past the cells) changed nothing in any of these runs. Wine's fresh memory is not Windows':
  the oracle cannot say what the bytes hold on Windows, only that under Wine they do not vary with
  the history. Whether the draw 0x532A5B happens in step 2 at all is T1b2's draw log's to show;
  T1c classifies the quirk.
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

## Not yet (T1b2, then T1c)

- **T1b2, the phase oracle:** every draw of a generation logged with its site and value
  (`xdd.dll`'s logged sites, section 4, over survey §2.6's sites), and the generator's cells
  dumped after each of the 15 steps.
- The oracle covers the choices without Random, one human, the default towns; a human's town
  choice and more humans (the advanced options' seats) are not in the corpus yet.
