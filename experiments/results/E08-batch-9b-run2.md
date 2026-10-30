# E08 batch E9b — results

> Played by the capture harness on 2026-10-30 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-9b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

> The second recorded run (E08 T79b, the third played), a new game right after the first recorded (`E08-batch-9b.md`, whose agent's note applies here too): its draws (the weekly objects', the starting bonus) add to the first's; every row's outcome reads as the first's.

Map file: `E08-batch-9b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `c267b8f6`.** The map's description in the scenario list starts with "Stamp c267b8f6.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-9b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of Red's turn on days 2, 3, 8, 9 and 15 (saves 2 to 6).
4. A hero standing on an object's entrance hides the object's word: read it in the hero's record (+0x14, `docs/exe/structs.toml` "hero"), the object's class in +0xC.
5. Most records are the game's memory: the date (game +0x1F63E day of the week, +0x1F640 week, +0x1F642 month), the players' treasuries and their income (player +0x108), the mine records (game +0x4E38C, survey §2.1), the explored tiles, a hero's spell points, the Mystical Garden's mask of a player (+0xB8), the weekly objects' and the monsters' tile words (a Windmill's resource bits 0 to 3 and amount bits 13 to 16, a Water Wheel's gold bits 0 to 4 in 500s, a Garden's resource bits 6 to 9 and full bit 10; a monster's count bits 0 to 11 and sixteenths bits 27 to 30). The capture harness reads them (`clients/capture/batches/E08-batch-9b.toml`, E08 T79). Played by hand, read the screens instead and leave the memory-only lines empty.
6. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A visit: walk to the tile named, then click the object's entrance from there (twice).
7. Order: day 1, Red's turn: E9-6's reads, E9-9's first reads, Red's treasury, E9-12, E9-8; Red's treasury; end the turn. Blue's turn: Blue's gold (E9-11), E9-11's visits (E9-8); end the turn. Day 2: Red: the treasury, E9-12, E9-8 again, the treasury; Blue: the gold. Days 3 to 7: Red's treasury (and E9-12 on day 3), Blue's gold until day 6; end each turn. Day 8: Red: the treasury, E9-9's reads, E9-8, the treasury. Day 9: Red: the treasury, Ivor's visits (E9-8), the treasury. Days 10 to 14: end the turns. Day 15: Red: E9-9's reads. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E9-7 — The day's income: Red's mines (E9-6), heroes and a Crystal Dragon (E9-7) (GAM-CAL-002, GAM-RES-003, GAM-RES-004, GAM-RES-005, A-33)

- Red hero **E9-7** (Ufretin, specialty creature Dwarf): attack 1, defense 1, power 1, knowledge 1; Basic Estates; no artifacts; no spells; army 1 Crystal Dragon
- Red hero **Ryland** (Ryland, specialty creature Dendroid Guard): attack 1, defense 1, power 1, knowledge 1; no skills; Endless Purse of Gold (misc 1); no spells; army 1 Pikeman
- Red hero **Ivor** (Ivor, specialty creature Wood Elf): attack 1, defense 1, power 1, knowledge 1; no skills; Cornucopia (misc 1); no spells; army 2 Angel
- Red hero **Sephinroth** (Sephinroth, specialty resource crystal): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before any hero moves (E9-6): Red owns a gold mine G (entrance (63, 8)) and an ore pit O (entrance (63, 20)), far from every hero. Read whether these tiles are explored for Red: (63, 11), (63, 12), (65, 10), (66, 10), (60, 8), (59, 8), (63, 23), (63, 24). Read both mines' records.
2. Red's heroes of this row never move but Ivor on day 9 (E9-8): E9-7 at (4, 4), Basic Estates, one Crystal Dragon; Ryland at (6, 4), the Endless Purse of Gold; Ivor at (6, 6), the Cornucopia; Sephinroth at (10, 4), a crystal specialist.
3. Read Red's treasury (the seven resources) and the date at the start of each Red turn of days 1 to 9, and at the end of each (after E9-8's visits on days 1, 2, 8 and 9). Read Red's income (player +0x108, seven amounts) on days 2 and 8.

**Record**

| tile | explored for Red at day 1 |
|---|---|
| (63, 11) | yes |
| (63, 12) | no |
| (65, 10) | yes |
| (66, 10) | no |
| (60, 8) | yes |
| (59, 8) | no |
| (63, 23) | yes |
| (63, 24) | no |

- G's and O's records (owner, resource): record 0; owner red; resource gold; abandoned 0; guards none; record 1; owner red; resource ore; abandoned 0; guards none

| Red, day | when | wood | mercury | ore | sulfur | crystal | gems | gold |
|---|---|---|---|---|---|---|---|---|
| 1 | start | 30 | 15 | 30 | 15 | 15 | 15 | 30500 |
| 1 | end | 30 | 21 | 30 | 15 | 15 | 15 | 31500 |
| 2 | start | 30 | 26 | 32 | 20 | 21 | 20 | 33625 |
| 2 | end | 30 | 26 | 32 | 20 | 21 | 20 | 33625 |
| 3 | start | 30 | 31 | 34 | 25 | 27 | 25 | 35750 |
| 3 | end | 30 | 31 | 34 | 25 | 27 | 25 | 35750 |
| 4 | start | 30 | 36 | 36 | 30 | 33 | 30 | 37875 |
| 4 | end | 30 | 36 | 36 | 30 | 33 | 30 | 37875 |
| 5 | start | 30 | 41 | 38 | 35 | 39 | 35 | 40000 |
| 5 | end | 30 | 41 | 38 | 35 | 39 | 35 | 40000 |
| 6 | start | 30 | 46 | 40 | 40 | 45 | 40 | 42125 |
| 6 | end | 30 | 46 | 40 | 40 | 45 | 40 | 42125 |
| 7 | start | 30 | 51 | 42 | 45 | 51 | 45 | 44250 |
| 7 | end | 30 | 51 | 42 | 45 | 51 | 45 | 44250 |
| 8 | start | 30 | 56 | 44 | 50 | 60 | 50 | 46375 |
| 8 | end | 30 | 56 | 44 | 50 | 64 | 50 | 47875 |
| 9 | start | 30 | 61 | 46 | 55 | 70 | 55 | 50000 |
| 9 | end | 30 | 67 | 46 | 55 | 70 | 55 | 51500 |

- the date's words at days 1, 8 and 9: month 1, week 1, day 1; month 1, week 2, day 1; month 1, week 2, day 2
- Red's income (+0x108) on day 2: 0 wood, 5 mercury, 2 ore, 5 sulfur, 6 crystal, 5 gems, 2125 gold
- Red's income (+0x108) on day 8: 0 wood, 5 mercury, 2 ore, 5 sulfur, 9 crystal, 5 gems, 2125 gold
- anything odd: the game's version: SoD; Red's income at day 1 (+0x108): 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 0 gold; Red's Garden mask at day 1: 0x00000000 (none); Red, day 2's start: the messages: GENRLTXT:14; Red, day 2: the date; G's and O's records: month 1, week 1, day 2; record 0; owner red; resource gold; abandoned 0; guards none; record 1; owner red; resource ore; abandoned 0; guards none; day 2's end, E9-7 selected: E9-8's tile; W1's tile and W1 (through E9-8's record): 10,40,0; 0x00000016 class 34.0; hero E9-8, under it 0xfffe0071 class 112; Windmill: empty (mercury); marks 0x03 (under E9-8); Red, day 3's start: the messages: GENRLTXT:14; Red, day 3: the date: month 1, week 1, day 3; Red, day 4's start: the messages: GENRLTXT:14; GENRLTXT:14: Red; Red, day 4: the date: month 1, week 1, day 4; Red, day 5's start: the messages: GENRLTXT:14; GENRLTXT:14: Red; Red, day 5: the date: month 1, week 1, day 5; Red, day 6's start: the messages: GENRLTXT:14; GENRLTXT:14: Red; Red, day 6: the date: month 1, week 1, day 6; Red, day 7's start: the messages: GENRLTXT:14; GENRLTXT:14: Red; Red, day 7: the date: month 1, week 1, day 7; Red, day 8's start: the messages: GENRLTXT:14, GENRLTXT:16; GENRLTXT:14: Red; day 8's start, E9-7 selected: W1's tile and W1 (through E9-8's record): 0x00000016 class 34.0; hero E9-8, under it 0xfffe8074 class 112; Windmill: 4 crystal; marks 0x03 (under E9-8); Red, day 9's start: the messages: GENRLTXT:14; GENRLTXT:14: Red; Red, day 15's start: the messages: GENRLTXT:14, GENRLTXT:16

## E9-8 — The Windmill, the Water Wheel and the Mystical Garden through two weeks (ADV-WEK-001, ADV-WEK-002, ADV-WEK-003, ADV-VIS-006)

- Red hero **E9-8** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel

1. The first set: the Windmill W1 (entrance (10, 40)), the Water Wheel H1 (entrance (14, 40); it covers (12, 39) to (14, 40)), the Mystical Garden M1 (entrance (17, 40); it covers (16, 40) to (18, 40)). The second set: W2 (entrance (5, 8)), H2 (entrance (10, 8); it covers (8, 7) to (10, 8)), M2 (entrance (13, 8)). For each visit note the message, the gain (the treasury before and after) and, before and after, the object's word (the Garden's: and the visitor's mask).
2. Day 1, Red's turn: before the visits read all six words. E9-8 stands at (12, 42) with two Angels. Walk to (10, 41), click W1 (10, 40); walk to (13, 41), click H1 (14, 40); walk to (16, 41), click M1 (17, 40); walk to (19, 42).
3. Day 1, Blue's turn: E9-11 stands at (12, 44) with two Angels. As Red: walk to (10, 41), click W1; walk to (13, 41), click H1; walk to (16, 41), click M1; walk to (18, 43).
4. Day 2, Red's turn: E9-8 visits the first set again, in the other order: walk to (16, 41), click M1 (17, 40); walk to (15, 41), click H1 (14, 40); walk to (11, 41), click W1 (10, 40). It stays on W1's entrance until day 8.
5. Day 8, Red's turn: before the visits read all six words (E9-8 still stands on W1's entrance). Click (10, 41) (off the entrance), then W1 (10, 40) again; walk to (13, 41), click H1; walk to (16, 41), click M1.
6. Day 9, Red's turn: before the visits read W2's, H2's and M2's words. Ivor (E9-7's, at (6, 6)) walks to (5, 9), clicks W2 (5, 8); walks to (10, 9), clicks H2 (10, 8); walks to (13, 9), clicks M2 (13, 8).

**Record**

| object | word at day 1's start | word at day 8's start |
|---|---|---|
| W1 | 0xfffec011; Windmill: 6 mercury; marks 0x00 | 0xfffe8074; Windmill: 4 crystal; marks 0x03 |
| H1 | 0xffffe001; Water Wheel: 500 gold; marks 0x00 | 0xffffe062; Water Wheel: 1000 gold; marks 0x03 |
| M1 | 0xfffffda1; Garden 1: full, 500 gold | 0xfffffda1; Garden 1: full, 500 gold |
| W2 | 0xfffea014; Windmill: 5 crystal; marks 0x00 | 0xfffec011; Windmill: 6 mercury; marks 0x00 |
| H2 | 0xffffe001; Water Wheel: 500 gold; marks 0x00 | 0xffffe002; Water Wheel: 1000 gold; marks 0x00 |
| M2 | 0xfffffda0; Garden 0: full, 500 gold | 0xfffffda0; Garden 0: full, 500 gold |

| day | visitor | object | message | gain | word after |
|---|---|---|---|---|---|
| 1 | E9-8 (Red) | W1 (10, 40) | ADVEVENT:170 | +6 mercury | 0xfffe0031; Windmill: empty (mercury); marks 0x01 |
| 1 | E9-8 (Red) | H1 (14, 40) | ADVEVENT:164 | +500 gold | 0xffffe020; Water Wheel: empty; marks 0x01 |
| 1 | E9-8 (Red) | M1 (17, 40) | ADVEVENT:92 | +500 gold | 0xfffff9a1; Garden 1: empty (500 gold) |
| 1 | E9-11 (Blue) | W1 (10, 40) | ADVEVENT:169 | none | 0xfffe0071; Windmill: empty (mercury); marks 0x03 |
| 1 | E9-11 (Blue) | H1 (14, 40) | ADVEVENT:165 | none | 0xffffe060; Water Wheel: empty; marks 0x03 |
| 1 | E9-11 (Blue) | M1 (17, 40) | ADVEVENT:93 | none | 0xfffff9a1; Garden 1: empty (500 gold) |
| 2 | E9-8 (Red) | M1 (17, 40) | ADVEVENT:93 | none | 0xfffff9a1; Garden 1: empty (500 gold) |
| 2 | E9-8 (Red) | H1 (14, 40) | ADVEVENT:165 | none | 0xffffe060; Water Wheel: empty; marks 0x03 |
| 2 | E9-8 (Red) | W1 (10, 40) | ADVEVENT:169 | none | 0xfffe0071; Windmill: empty (mercury); marks 0x03 |
| 8 | E9-8 (Red) | W1 (10, 40) | ADVEVENT:170 | +4 crystal | 0xfffe0074; Windmill: empty (crystal); marks 0x03 |
| 8 | E9-8 (Red) | H1 (14, 40) | ADVEVENT:164 | +1000 gold | 0xffffe060; Water Wheel: empty; marks 0x03 |
| 8 | E9-8 (Red) | M1 (17, 40) | ADVEVENT:92 | +500 gold | 0xfffff9a1; Garden 1: empty (500 gold) |
| 9 | Ivor (Red) | W2 (5, 8) | ADVEVENT:170 | +6 mercury | 0xfffe0031; Windmill: empty (mercury); marks 0x01 |
| 9 | Ivor (Red) | H2 (10, 8) | ADVEVENT:164 | +1000 gold | 0xffffe020; Water Wheel: empty; marks 0x01 |
| 9 | Ivor (Red) | M2 (13, 8) | ADVEVENT:92 | +500 gold | 0xfffff9a0; Garden 0: empty (500 gold) |

- anything odd: M1's visit by E9-8 (1): Red's Garden mask before; after: 0x00000000 (none); 0x00000002 (Gardens 1); M1's visit by E9-11 (1): Blue's Garden mask before; after: 0x00000000 (none); 0x00000002 (Gardens 1); M1's visit by E9-8 (2): Red's Garden mask before; after: 0x00000002 (Gardens 1); 0x00000002 (Gardens 1); day 8's start: E9-8's tile (on W1's entrance); W1's tile: 10,40,0; 0xfffe8074 class 112.0; M1's visit by E9-8 (8): Red's Garden mask before; after: 0x00000002 (Gardens 1); 0x00000002 (Gardens 1); day 9, before Ivor's visits: W2's, H2's and M2's words: 0xfffec011; Windmill: 6 mercury; marks 0x00; 0xffffe002; Water Wheel: 1000 gold; marks 0x00; 0xfffffda0; Garden 0: full, 500 gold; M2's visit by Ivor (9): Red's Garden mask before; after: 0x00000002 (Gardens 1); 0x00000003 (Gardens 0, 1)

## E9-9 — The monsters' growth in sixteenths (E9-9) and its cap above 4 000 (E9-10) (D-49, A-32, ADV-MON-013)

- Blue hero **E9-9** (Aislinn, specialty spell Meteor Shower): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Five wandering monsters stand on row 58, far from every hero: 43 Pikemen at (44, 58), 81 Archers at (48, 58), 15 Griffins at (52, 58) and 4 095 Troglodytes at (60, 58) grow; 43 Pikemen at (56, 58) do not grow (the map's flag). E9-9 (Blue's) stands at (50, 66) and never moves.
2. Read each monster's count and sixteenths (its tile word) at day 1, at Red's turn of day 8 and at Red's turn of day 15.

**Record**

| monster | day 1 | day 8 | day 15 |
|---|---|---|---|
| 43 Pikemen (44, 58) | 43 Pikemen and 0/16 (688 sixteenths), grows | 47 Pikemen and 4/16 (756 sixteenths), grows | 51 Pikemen and 15/16 (831 sixteenths), grows |
| 81 Archers (48, 58) | 81 Archers and 0/16 (1296 sixteenths), grows | 89 Archers and 1/16 (1425 sixteenths), grows | 97 Archers and 15/16 (1567 sixteenths), grows |
| 15 Griffins (52, 58) | 15 Griffins and 0/16 (240 sixteenths), grows | 16 Griffins and 8/16 (264 sixteenths), grows | 18 Griffins and 2/16 (290 sixteenths), grows |
| 43 Pikemen, no growth (56, 58) | 43 Pikemen and 0/16 (688 sixteenths), does not grow | 43 Pikemen and 0/16 (688 sixteenths), does not grow | 43 Pikemen and 0/16 (688 sixteenths), does not grow |
| 4 095 Troglodytes (60, 58) | 4095 Troglodytes and 0/16 (65520 sixteenths), grows | 4000 Troglodytes and 0/16 (64000 sixteenths), grows | 4000 Troglodytes and 0/16 (64000 sixteenths), grows |

- anything odd: the monsters' words at day 1 (count, aggression): 43 Pikemen; 81 Archers; 15 Griffins; 43 Pikemen; 4095 Troglodytes; Red, day 15: the date: month 1, week 3, day 1

## E9-11 — Timed events: their order, the floor at 0, the repeat (GAM-CAL-004)

- Blue hero **E9-11** (Alagar, specialty spell Ice Bolt): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel

1. Blue owns only its town (+500 gold a day from day 2); E9-11 (at (12, 44)) has no income. The map's events, in this order: day 1 takes 99 999 gold; day 1 gives 300 gold; from day 2 every two days takes 500 gold; all for Blue.
2. Read Blue's gold at the start of each Blue turn of days 1 to 6 (after the messages), and note the messages.

**Record**

| Blue, day | messages | gold at the turn's start |
|---|---|---|
| 1 | GENRLTXT:14, the map's text "A tax collector takes your gold.", the map's text "A patron gives you 300 gold."; GENRLTXT:14: Blue; the map's text "A tax collector takes your gold.": -30700 gold; the map's text "A patron gives you 300 gold.": 300 gold | 300 |
| 2 | GENRLTXT:14, the map's text "A toll of 500 gold."; GENRLTXT:14: Blue; the map's text "A toll of 500 gold.": -500 gold | 300 |
| 3 | GENRLTXT:14; GENRLTXT:14: Blue | 800 |
| 4 | GENRLTXT:14, the map's text "A toll of 500 gold."; GENRLTXT:14: Blue; the map's text "A toll of 500 gold.": -500 gold | 800 |
| 5 | GENRLTXT:14; GENRLTXT:14: Blue | 1300 |
| 6 | GENRLTXT:14, the map's text "A toll of 500 gold."; GENRLTXT:14: Blue; the map's text "A toll of 500 gold.": -500 gold | 1300 |

- anything odd: Blue, day 1: the date; its treasury: month 1, week 1, day 1; 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 300 gold; Blue, day 7's start: the messages; its gold: GENRLTXT:14; 1800; Blue, day 8's start: the messages; its gold: GENRLTXT:14, the map's text "A toll of 500 gold.", GENRLTXT:16; 1800

## E9-12 — The day's spell points: Mysticism, the Charm of Mana, the Wizard's Well (HER-MAN-002)

- Red hero **E9-12** (Coronius, specialty spell Slayer): attack 1, defense 1, power 1, knowledge 2; Basic Mysticism; no artifacts; no spells; army 1 Pikeman
- Red hero **Uland** (Uland, specialty spell Cure): attack 1, defense 1, power 1, knowledge 2; Basic Mysticism; Charm of Mana (misc 1); no spells; army 1 Pikeman
- Red hero **Melodia** (Melodia, specialty spell Fortune): attack 1, defense 1, power 1, knowledge 2; no skills; Wizard's Well (misc 1); no spells; army 1 Pikeman

1. Day 1, Red's turn: three heroes with Knowledge 2 (20 spell points at most) stand on row 16, an event east of each that takes 20 spell points: E9-12 at (4, 16) (Basic Mysticism), Uland at (8, 16) (Basic Mysticism and the Charm of Mana), Melodia at (12, 16) (the Wizard's Well). Click (5, 16) with E9-12, (9, 16) with Uland, (13, 16) with Melodia: each steps onto its event. Read each hero's spell points.
2. Days 2 and 3, Red's turn's start: read each hero's spell points.

**Record**

| hero | day 1 after the event | day 2 | day 3 |
|---|---|---|---|
| E9-12 | 0 | 2 | 4 |
| Uland | 0 | 3 | 6 |
| Melodia | 0 | 20 | 20 |

- anything odd: E9-12, day 1: the points before; the messages: 20; ADVEVENT:177; Uland, day 1: the points before; the messages: 20; ADVEVENT:177; Melodia, day 1: the points before; the messages: 20; ADVEVENT:177
