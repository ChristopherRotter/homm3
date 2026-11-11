# E09 batch TWN-3a — results

> Played by the capture harness on 2026-11-11 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-3a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-3a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `d5f72a08`.** The map's description in the scenario list starts with "Stamp d5f72a08.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-3a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml`): its available creatures +0x16 (i16 by the faction's index, 0-6 base, 7-13 upgraded), its Portal's creature +0x3C and count +0x40, its garrison +0xE0, its external dwellings' counts +0x118; the date +0x1F63E. The capture harness reads them and forces the weeks' and months' draws (`clients/capture/batches/E09-batch-3a.toml`, E09 T23). Played by hand the counts show in the town screen (the Fort's list) and the garrisons in the town's popup; the weeks' and months' kinds come as they fall.
5. Tiles are (column, row) from 0 at the top left; a town's or dwelling's entrance is the tile a hero visits. A step: click the target once (the path's marks show), then again (the hero goes).
6. Order: Red's day 1 (the reads, the flags, the visits); then each week: end Red's turn and Blue's (by hand: play the days through), the reads on Red's new turn.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-3-1 — A month of the Griffin: K1's upgraded Griffin Tower and K2's Griffin Tower (TD-13, T-13, TWN-GRO-004)

- Red hero **TWN-3-1** (Grindan, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1: read K1's (entrance (7, 7)) and K2's (15, 7) available creatures.
2. Play the weeks as TWN-3-4 says. Before the second month starts (Red's turn of month 1, week 4, its last day; by the harness the second month's draw forced to the Griffin's) read K1's and K2's available creatures; on Red's first day of month 2 read them again, with the month's kind and creature.

**Record**

- day 1: K1's and K2's available creatures: K1: base 0, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 7, 0, 0, 0, 0; K2: base 0, 0, 7, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0
- month 1, week 4 (before the month): K1's and K2's available creatures: K1: base 0, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 37, 0, 0, 0, 0; K2: base 0, 0, 37, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0
- month 2, day 1: the week's and month's kinds; K1's and K2's available creatures: week 0 (name 8), month 1 (Griffin); K1: base 0, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 47, 0, 0, 0, 0; K2: base 0, 0, 94, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0
- anything odd: the game's version: SoD; the kinds at the start: week 0 (name 0), month 0 (name 0); M2 W1: Blue's turn: the date: month 1, week 4, day 7

## TWN-3-2 — A dwelling's growth with the Castle, a horde, the Grail, two flagged Guardhouses, the Statue of Legion elsewhere and Legs of Legion on the visitor (G1) (TWN-GRO-001, TWN-GRO-003)

- Red hero **TWN-3-2** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; Legs of Legion (misc 1); no spells; army 1 Pikeman
- Red hero **TWN-3-2s** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; Statue of Legion (misc 1); no spells; army 1 Pikeman
- Red hero **TWN-3-2t** (Nagash, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves: read G1's (entrance (23, 7)) available creatures and external dwellings' counts.
2. Read G1's standing and every-built sets. TWN-3-2s (the Statue of Legion) steps onto the Guardhouse GH1's entrance (10, 30), TWN-3-2t onto GH2's (16, 30): each flags it for Red (answer No to the hire question). Read G1's external dwellings' counts.
3. TWN-3-2 (Legs of Legion) steps north onto G1's entrance and stays there (close the town screen): G1's visitor.
4. At the first week's start (Red's turn of week 2, day 1) read G1's available creatures, its visitor and which heroes wear the Statue and Legs.

**Record**

- day 1: G1's available creatures; its external dwellings' counts; its standing and every-built sets: base 14, 9, 7, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; dwellings: base 0, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; standing: 9, 10, 18, 26, 30, 31, 32; every: 7, 8, 9, 10, 18, 26, 30, 31, 32
- after the flags: G1's external dwellings' counts; GH1's and GH2's owners: base 2, 0, 0, 0, 0, 0, 0; upgraded 2, 0, 0, 0, 0, 0, 0; GH1: red; GH2: red
- after the visit: G1's visitor: TWN-3-2
- week 2, day 1: G1's available creatures; its visitor; the Statue's and Legs' wearers: base 80, 57, 43, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; visitor: TWN-3-2; the Statue: Statue of Legion (misc 1); Legs: Legs of Legion (misc 1)
- anything odd: GH1's visit: the questions: ADVEVENT:35; GH2's visit: the questions: ADVEVENT:35; G1's visit: a town screen; how it closed: yes; click; week 2: G1's external dwellings' counts: base 2, 0, 0, 0, 0, 0, 0; upgraded 2, 0, 0, 0, 0, 0, 0; week 2: the kinds: week 0 (name 13), month 0 (name 0)

## TWN-3-3 — The Plague: an owned town (O1) and a neutral one (N1) with known counts (TD-14, T-12, TWN-GRO-004)

- Red hero **TWN-3-3** (Damacon, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1: read O1's (entrance (39, 7)) and N1's (47, 7) available creatures and N1's garrison.
2. Play the weeks as TWN-3-4 says. Before the third month starts (Red's turn of month 2, week 4, its last day; by the harness the third month's draw forced to the Plague) read O1's and N1's available creatures; on Red's first day of month 3 read them again, with the month's kind, and N1's garrison.

**Record**

- day 1: O1's and N1's available creatures; N1's garrison: O1: base 14, 9, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; N1: base 14, 9, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; N1's garrison: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils
- month 2, week 4 (before the Plague): O1's and N1's available creatures: O1: base 106, 61, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; N1: base 42, 25, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0
- month 3, day 1: the week's and month's kinds; O1's and N1's available creatures; N1's garrison: week 0 (name 3), month 2; O1: base 53, 30, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; N1: base 17, 10, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; N1's garrison: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils
- anything odd: M3 W1: Blue's turn: the date: month 2, week 4, day 7

## TWN-3-4 — The neutral towns' garrisons over eight weeks: with a Fort (F1, F2, FF), the Citadel (F3), without (W1, W2) (TD-15, TWN-GRO-006)

- Red hero **TWN-3-4** (Clavius, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1: read the six towns' garrisons (F1 (7, 15), F2 (15, 15), F3 (23, 15), W1 (31, 15), W2 (39, 15), FF (47, 15)) and F1's available creatures.
2. Each week: end Red's turn and Blue's (by the harness: the date first written to the week's last day); on Red's new turn read the date, the six garrisons, F1's available creatures and the draws the week made (by the harness: xdd.dll's log). The weeks: month 1 weeks 2 to 4, month 2 week 1, month 3 weeks 1 to 4 (month 2 weeks 2 to 4 skipped by writing the date).

**Record**

| week (the date) | F1 garrison | F2 garrison | F3 garrison | W1 garrison | W2 garrison | FF garrison | F1's available creatures | the garrisons' draws (in the towns' order) |
|---|---|---|---|---|---|---|---|---|
| month 1, week 1, day 1 | empty | empty | empty | empty | empty | slot 0: 1 Gremlin, slot 1: 1 Imp, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 14, 9, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 |
| month 1, week 2, day 1 | slot 0: 14 Pikemen | empty | slot 0: 7 Griffins | slot 0: 9 Archers | empty | slot 0: 9 Archers, slot 1: 1 Imp, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 7, 13, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 45, garrison_chance_fort 74, garrison_sum_1 1, garrison_sum_2 0, garrison_sum_3 1, garrison_upgrade 82, garrison_chance_fort 94, garrison_chance_fort 28, garrison_sum_1 2, garrison_sum_2 2, garrison_sum_3 2, garrison_upgrade 39, garrison_chance 11, garrison_sum_1 2, garrison_sum_2 2, garrison_sum_3 1, garrison_upgrade 15, garrison_chance 65, garrison_chance_fort 20, garrison_sum_1 1, garrison_sum_2 2, garrison_sum_3 2, garrison_upgrade 31 |
| month 1, week 3, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers | slot 0: 8 Dwarves | slot 0: 7 Griffins, slot 1: 9 Archers | slot 0: 18 Archers | slot 0: 9 Stone Gargoyles | slot 0: 18 Archers, slot 1: 1 Imp, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 14, 8, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 25, garrison_sum_1 1, garrison_sum_2 2, garrison_sum_3 2, garrison_chance_fort 7, garrison_sum_1 1, garrison_sum_2 1, garrison_sum_3 2, garrison_upgrade 65, garrison_chance_fort 39, garrison_sum_1 0, garrison_sum_2 0, garrison_sum_3 3, garrison_upgrade 79, garrison_chance_fort 3, garrison_sum_1 0, garrison_sum_2 1, garrison_sum_3 3, garrison_upgrade 24, garrison_chance 9, garrison_sum_1 3, garrison_sum_2 0, garrison_sum_3 2, garrison_upgrade 9, garrison_chance 3, garrison_sum_1 3, garrison_sum_2 0, garrison_sum_3 1, garrison_upgrade 93, garrison_chance_fort 7, garrison_sum_1 2, garrison_sum_2 1, garrison_sum_3 2, garrison_upgrade 40 |
| month 1, week 4, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 7 Griffins | slot 0: 8 Dwarves, slot 1: 7 Wood Elves | slot 0: 7 Griffins, slot 1: 18 Archers | slot 0: 27 Archers | slot 0: 9 Stone Gargoyles | slot 0: 18 Archers, slot 1: 1 Imp, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 21, 12, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 18, garrison_sum_1 0, garrison_sum_2 0, garrison_sum_3 2, garrison_chance_fort 61, garrison_sum_1 2, garrison_sum_2 2, garrison_sum_3 4, garrison_upgrade 37, garrison_chance_fort 4, garrison_sum_1 1, garrison_sum_2 2, garrison_sum_3 3, garrison_upgrade 62, garrison_chance_fort 30, garrison_sum_1 2, garrison_sum_2 2, garrison_sum_3 0, garrison_upgrade 82, garrison_chance 14, garrison_sum_1 2, garrison_sum_2 2, garrison_sum_3 1, garrison_upgrade 80, garrison_chance 62, garrison_chance_fort 97 |
| month 2, week 1, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 14 Griffins | slot 0: 8 Dwarves, slot 1: 14 Wood Elves | slot 0: 7 Griffins, slot 1: 18 Archers | slot 0: 27 Archers, slot 1: 7 Griffins | slot 0: 9 Stone Gargoyles | slot 0: 18 Archers, slot 1: 7 Griffins, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 28, 16, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 25, garrison_sum_1 2, garrison_sum_2 4, garrison_sum_3 5, garrison_chance_fort 68, garrison_sum_1 2, garrison_sum_2 4, garrison_sum_3 1, garrison_upgrade 66, garrison_chance_fort 21, garrison_sum_1 5, garrison_sum_2 0, garrison_sum_3 4, garrison_upgrade 28, garrison_chance_fort 90, garrison_chance 0, garrison_sum_1 1, garrison_sum_2 4, garrison_sum_3 1, garrison_upgrade 83, garrison_chance 91, garrison_chance_fort 22, garrison_sum_1 0, garrison_sum_2 4, garrison_sum_3 2, garrison_upgrade 32 |
| month 3, week 1, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 14 Griffins | slot 0: 8 Dwarves, slot 1: 14 Wood Elves, slot 2: 2 Unicorns | slot 0: 7 Griffins, slot 1: 18 Archers, slot 2: 2 Cavaliers | slot 0: 27 Archers, slot 1: 7 Griffins | slot 0: 9 Stone Gargoyles | slot 0: 18 Archers, slot 1: 7 Griffins, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 10, 5, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 50, garrison_chance_fort 84, garrison_chance_fort 58, garrison_sum_1 9, garrison_sum_2 1, garrison_sum_3 9, garrison_upgrade 55, garrison_chance_fort 70, garrison_sum_1 5, garrison_sum_2 8, garrison_sum_3 8, garrison_upgrade 73, garrison_chance 98, garrison_chance 46, garrison_chance_fort 84 |
| month 3, week 2, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 14 Griffins, slot 3: 3 Monks | slot 0: 8 Dwarves, slot 1: 14 Wood Elves, slot 2: 2 Unicorns | slot 0: 7 Griffins, slot 1: 18 Archers, slot 2: 2 Cavaliers, slot 3: 4 Swordsmen | slot 0: 27 Archers, slot 1: 7 Griffins | slot 0: 9 Stone Gargoyles | slot 0: 27 Archers, slot 1: 7 Griffins, slot 2: 1 Gnoll, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 17, 9, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 44, garrison_chance_fort 43, garrison_sum_1 9, garrison_sum_2 4, garrison_sum_3 2, garrison_upgrade 54, garrison_chance_fort 97, garrison_chance_fort 69, garrison_sum_1 2, garrison_sum_2 5, garrison_sum_3 7, garrison_upgrade 18, garrison_chance 43, garrison_chance 67, garrison_chance_fort 78, garrison_sum_1 1, garrison_sum_2 1, garrison_sum_3 1, garrison_upgrade 28 |
| month 3, week 3, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 14 Griffins, slot 3: 6 Zealots | slot 0: 8 Dwarves, slot 1: 21 Wood Elves, slot 2: 2 Unicorns | slot 0: 7 Griffins, slot 1: 18 Archers, slot 2: 2 Cavaliers, slot 3: 8 Swordsmen | slot 0: 27 Archers, slot 1: 7 Griffins | slot 0: 9 Stone Gargoyles | slot 0: 27 Archers, slot 1: 7 Griffins, slot 2: 1 Angel, slot 3: 1 Troglodyte, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 24, 13, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 4, garrison_sum_1 6, garrison_sum_2 7, garrison_sum_3 6, garrison_chance_fort 39, garrison_sum_1 6, garrison_sum_2 4, garrison_sum_3 8, garrison_upgrade 2, garrison_chance_fort 63, garrison_sum_1 3, garrison_sum_2 2, garrison_sum_3 4, garrison_upgrade 60, garrison_chance_fort 48, garrison_sum_1 9, garrison_sum_2 2, garrison_sum_3 1, garrison_upgrade 90, garrison_chance 94, garrison_chance 58, garrison_chance_fort 47, garrison_sum_1 9, garrison_sum_2 6, garrison_sum_3 8, garrison_upgrade 67 |
| month 3, week 4, day 1 | slot 0: 14 Pikemen, slot 1: 9 Archers, slot 2: 14 Griffins, slot 3: 6 Zealots | slot 0: 8 Dwarves, slot 1: 21 Wood Elves, slot 2: 2 Unicorns, slot 3: 5 Pegasi | slot 0: 7 Griffins, slot 1: 18 Archers, slot 2: 2 Cavaliers, slot 3: 12 Swordsmen | slot 0: 27 Archers, slot 1: 7 Griffins, slot 2: 4 Swordsmen | slot 0: 9 Stone Gargoyles, slot 1: 4 Arch Magi | slot 0: 27 Archers, slot 1: 7 Griffins, slot 2: 1 Angel, slot 3: 3 Monks, slot 4: 1 Skeleton, slot 5: 1 Goblin, slot 6: 1 Centaur | base 31, 17, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0 | garrison_chance 2, garrison_sum_1 4, garrison_sum_2 6, garrison_sum_3 4, garrison_chance_fort 99, garrison_chance_fort 36, garrison_sum_1 4, garrison_sum_2 6, garrison_sum_3 4, garrison_upgrade 72, garrison_chance_fort 9, garrison_sum_1 4, garrison_sum_2 4, garrison_sum_3 5, garrison_upgrade 75, garrison_chance 19, garrison_sum_1 9, garrison_sum_2 0, garrison_sum_3 1, garrison_upgrade 97, garrison_chance 3, garrison_sum_1 4, garrison_sum_2 3, garrison_sum_3 6, garrison_upgrade 4, garrison_chance_fort 37, garrison_sum_1 6, garrison_sum_2 4, garrison_sum_3 5, garrison_upgrade 53 |

- anything odd: M1 W2: Blue's turn: the date: month 1, week 1, day 7; M1 W2: the kinds: week 0 (name 13), month 0 (name 0); M1 W2: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 21, 13, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M1 W2: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 78 characters]; M1 W3: Blue's turn: the date: month 1, week 2, day 7; M1 W3: the kinds: week 0 (name 10), month 0 (name 0); M1 W3: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 28, 17, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M1 W3: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 78 characters]; M1 W4: Blue's turn: the date: month 1, week 3, day 7; M1 W4: the kinds: week 0 (name 1), month 0 (name 0); M1 W4: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 35, 21, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M1 W4: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 76 characters]; M2 W1: the kinds: week 0 (name 8), month 1 (Griffin); M2 W1: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 42, 25, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M2 W1: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [names Griffin]; M3 W1: the kinds: week 0 (name 3), month 2; M3 W1: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 17, 10, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M3 W1: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 70 characters]; M3 W2: Blue's turn: the date: month 3, week 1, day 7; M3 W2: the kinds: week 0 (name 9), month 2; M3 W2: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 24, 14, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M3 W2: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 73 characters]; M3 W3: Blue's turn: the date: month 3, week 2, day 7; M3 W3: the kinds: week 0 (name 1), month 2; M3 W3: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 31, 18, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M3 W3: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 76 characters]; M3 W4: Blue's turn: the date: month 3, week 3, day 7; M3 W4: the kinds: week 0 (name 7), month 2; M3 W4: N1's garrison and counts: slot 0: 10 Black Dragons, slot 1: 10 Gold Dragons, slot 2: 10 Titans, slot 3: 10 Ghost Dragons, slot 4: 10 Ancient Behemoths, slot 5: 10 Chaos Hydras, slot 6: 10 Arch Devils; base 38, 22, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; M3 W4: the new turn's messages: GENRLTXT:14, GENRLTXT:16 [other: 75 characters]

## TWN-3-5 — The Portal of Summoning (D1): its first creature at the town screen, the weekly count set, the Plague (TWN-GRO-007, T-11)

- Red hero **TWN-3-5** (Gelare, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-3-5d** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **TWN-3b** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves: read D1's (entrance (31, 7)) Portal (creature and count).
2. After TWN-3-2's flags, TWN-3-5 steps onto the Harpy Loft's entrance (22, 30) (Red's third flagged dwelling; close its hire dialog). Read D1's Portal.
3. TWN-3-5d steps north onto D1's entrance (the town screen opens; close it): read D1's Portal right before the step and after it.
4. Each week (TWN-3-4's), on Red's new turn: read D1's Portal and the Portal's draws the week made (by the harness: xdd.dll's log), with the month's kind (the third month the Plague).

**Record**

- day 1: D1's Portal: none (count 21872)
- after the three flags: D1's Portal; the Harpy Loft's owner: none (count 21872); the Harpy Loft: red
- D1's visit: the Portal before; after; the Portal's draws: before: none (count 21872); after: 14 Pikemen; draws: portal_dwelling 0, portal_creature 25322

| week (the date) | D1's Portal | the Portal's draws |
|---|---|---|
| month 1, week 2, day 1 | 8 Harpies | portal_dwelling 2, portal_creature 6883 |
| month 1, week 3, day 1 | 14 Pikemen | portal_dwelling 1, portal_creature 17806 |
| month 1, week 4, day 1 | 8 Harpies | portal_dwelling 2, portal_creature 10258 |
| month 2, week 1, day 1 | 14 Pikemen | portal_dwelling 0, portal_creature 18416 |
| month 3, week 1, day 1 | 14 Pikemen | portal_dwelling 1, portal_creature 16980 |
| month 3, week 2, day 1 | 8 Harpies | portal_dwelling 2, portal_creature 13899 |
| month 3, week 3, day 1 | 14 Pikemen | portal_dwelling 1, portal_creature 1092 |
| month 3, week 4, day 1 | 14 Pikemen | portal_dwelling 0, portal_creature 7324 |

- anything odd: the Harpy Loft's visit: the questions: ADVEVENT:35; the Harpy Loft's record: record 2; class 17.26; owner red; creatures 0: 8 Harpies; guards none; D1's visit: a town screen; how it closed: yes; click; M1 W2: the kinds: week 0 (name 13), month 0 (name 0); M1 W3: the kinds: week 0 (name 10), month 0 (name 0); M1 W4: the kinds: week 0 (name 1), month 0 (name 0); M2 W1: the kinds: week 0 (name 8), month 1 (Griffin); M3 W1: the kinds: week 0 (name 3), month 2; M3 W2: the kinds: week 0 (name 9), month 2; M3 W3: the kinds: week 0 (name 1), month 2; M3 W4: the kinds: week 0 (name 7), month 2
