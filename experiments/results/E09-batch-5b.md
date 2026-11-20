# E09 batch TWN-5b — results

> Played by the capture harness on 2026-11-20 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-5b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-5b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1ca79538`.** The map's description in the scenario list starts with "Stamp 1ca79538.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-5b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, both players' starting bonus Gold. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory (the capture harness, `clients/capture/batches/E09-batch-5b.toml`, E09 T44): a hero's spell points (+0x18) and a town's record (game +0x21614, `docs/exe/structs.toml` "town"): its garrison hero (+0xC), visitor (+0x10) and the Mana Vortex's week flag (+0x33). The harness writes the date (game +0x1F63E) and the spell points; by hand, play the days out instead and spend points by casting so that they are below the maximum before each week's end.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes). On the town screen the lower portrait is the visitor, the upper the garrison's: click the lower, then the upper, to move the visitor into the garrison. The town list (the right panel's lower list) opens a town's screen without a hero: click its row twice.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-5-5 — The Mana Vortex with a garrison hero and a visitor over three weeks (TD-27, §14, §15): the spell points at each step (TD-27, TWN-GLD-006, TWN-GLD-008, TWN-GLD-009)

- Red hero **TWN-5-5** (Sephinroth, specialty resource crystal): attack 1, defense 1, power 1, knowledge 2; Basic Leadership; Spell Book (spell book); no spells; army 1 Pikeman
- Red hero **TWN-5-5v** (Geon, specialty skill Eagle Eye): attack 1, defense 1, power 1, knowledge 3; Basic Leadership; Spell Book (spell book); no spells; army 1 Pikeman

1. Red's turn, day 1 (week 1). Note G's and V's spell points and D's week flag.
2. G steps north onto D's entrance (7, 7): the town screen opens (note any message). Move G into the garrison (the lower portrait, then the upper). Note the messages, G's points and the flag; close the screen.
3. Set G's spell points to 5. Week 2: end the turns to week 2's first day (Red's turn, day 8). At the turn's start, before anything else: note G's and V's points and the flag.
4. Open D's screen from the town list (no hero walks): note any message, then G's points and the flag; close it.
5. V walks to D's entrance (7, 7): the town screen opens (note any message); close it. Note V's and G's points and the flag.
6. Set G's and V's points to 5. Week 3: end the turns to week 3's first day (Red's turn, day 15). At the turn's start: note G's and V's points, D's garrison hero and visitor, the flag.
7. Open D's screen from the town list: note any message, then G's and V's points and the flag; close it.

**Record**

- week 1, day 1: G's points; V's points; the flag: G: 20; V: 30; flag: 1
- week 1, G's visit: G's points and the flag as the screen shows: G: 40; flag: 0
- week 1, G's visit and move into the garrison: the messages; G's points; the flag: GENRLTXT:580; G: 40; flag: 0; garrison hero: TWN-5-5
- week 2, day 8, the turn's start: G's points; V's points; the flag: G: 20; V: 30; flag: 1; date: month 1, week 2, day 1
- week 2, D's screen opened from the town list: the messages; G's points; the flag: GENRLTXT:580; G: 40; flag: 0
- week 2, V's visit: the messages; V's points; G's points; the flag: none; V: 30; G: 40; flag: 0
- week 3, day 15, the turn's start: G's points; V's points; D's garrison hero and visitor; the flag: G: 20; V: 30; garrison hero: TWN-5-5; standing: TWN-5-5v; flag: 1; date: month 1, week 3, day 1
- week 3, D's screen opened: the messages; G's points; V's points; the flag: GENRLTXT:580; G: 40; V: 30; flag: 0
- anything odd: the game's version: SoD; G's visit: a town screen; how it closed: yes; click; week 2's start: the messages: GENRLTXT:14, GENRLTXT:16 [other: 78 characters]; V's visit: a town screen; how it closed: yes; click; week 3's start: the messages: GENRLTXT:14, GENRLTXT:16 [other: 76 characters]
