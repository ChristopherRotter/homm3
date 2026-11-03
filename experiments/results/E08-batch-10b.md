# E08 batch E10b — results

> Played by the capture harness on 2026-11-03 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-10b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E08-batch-10b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `6693babb`.** The map's description in the scenario list starts with "Stamp 6693babb.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-10b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of Red's turn on days 2, 7 and 8 (saves 2 to 4).
4. A hero standing on an object's entrance hides the object's word: every visit below ends with a step off the entrance before the reads.
5. Most records are the game's memory: the date (game +0x1F63E day of the week, +0x1F640 week, +0x1F642 month), the dwelling records (game +0x4E39C, 0x5C bytes each: four creature types +0x4 and counts +0x14, the guards' seven types +0x1C and counts +0x38, the entrance +0x54, the owner +0x57; survey §2.1), a Refugee Camp's tile words (the creature +0x22, the count +0x0 of its entrance tile), the hire dialog (the most +0xAC, the amount +0xB8, whether it is open), the players' treasuries, a hero's army, the status bar's hint and the right-click popup's text. The capture harness reads them (`clients/capture/batches/E08-batch-10b.toml`, E08 T90). Played by hand, read the screens instead and leave the memory-only lines empty.
6. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A visit: walk to the tile named, then click the object's entrance from there (twice).
7. Each battle by auto combat; a tactics phase is ended at once (its Start button). A level-up's dialog after a battle: take the left skill.
8. The hire dialog: **Max** sets the most, the slider sets an amount, **Recruit** buys the amount set, **Close** (the dialog's exit) ends it. The most (+0xAC) is memory-only; by hand note the dialog's Available and the amount Max sets.
9. Order: day 1, Red's turn: the first reads (E10-8, E10-9); E10-8; E10-9; E10-10's Red reads; end the turn. Blue's turn: E10-10; end the turn. Day 2, Red's turn: E10-8's reads, E10-9; end the turn; Blue: end the turn. Days 3 to 6: end the turns. Day 7, Red's turn: E10-8's and E10-9's reads; end the turns. Day 8, Red's turn: the week's message, E10-8's reads, E10-9. Then the map is done.
10. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E10-8 — The week's refill: a dwelling Red flagged, one nobody visited, Red's Barracks bought from (ADV-DWL-003, GAM-CAL-002, ADV-DWL-002)

- Red hero **E10-8** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Red hero **E10-8b** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Three dwellings stand on row 10: the neutral Dragon Cliffs D (entrance (10, 10); it covers (9, 10) and (10, 10)), the neutral Golem Factory F (entrance (13, 10); (12, 10) and (13, 10)), which no hero visits, and Red's Barracks S (entrance (16, 10); (15, 10) and (16, 10)). Day 1, Red's turn, before any move: read the three records (owner, creatures and counts, guards) and the date.
2. E10-8 stands at (11, 13) with ten Archangels. Walk to (10, 11), then click D's entrance (10, 10). Note the question; answer yes. Auto combat. The next question (to recruit): answer **no**. Walk to (10, 13). Read D's record.
3. E10-8b stands at (16, 13) with one Angel. Walk to (16, 11), then click S's entrance (16, 10). The question: yes. In the dialog set the amount to **2** (not Max) and click Recruit. Walk to (16, 13). Read E10-8b's army, Red's gold and S's record.
4. Day 2, day 7 and day 8, Red's turn, before any move: read the date and the three records. On day 8 note the week's message (its name) first.

**Record**

| dwelling | owner | creatures and counts | guards (creature and count by slot) |
|---|---|---|---|
| D day 1 before | none | 0: 1 Green Dragon | 0: 3 Green Dragons |
| F day 1 before | none | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems |
| S day 1 before | red | 0: 4 Swordsmen | none |
| D day 1 after its battle | red | 0: 1 Green Dragon | none |
| S day 1 after the purchase | red | 0: 2 Swordsmen | none |
| D day 2 | red | 0: 1 Green Dragon | none |
| F day 2 | none | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems |
| S day 2 | red | 0: 2 Swordsmen | none |
| D day 7 | red | 0: 1 Green Dragon | none |
| F day 7 | none | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems |
| S day 7 | red | 0: 2 Swordsmen | none |
| D day 8 | red | 0: 1 Green Dragon | 0: 3 Green Dragons |
| F day 8 | none | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems |
| S day 8 | red | 0: 4 Swordsmen | none |

- D's battle: the question's text; the defending stacks; the result: GENRLTXT:422, ADVEVENT:35; attacker 10 Archangels | defender 3 Green Dragons; attacker; none (human 0, here 0); attacker 10 Archangels | defender none
- S's purchase: the most; E10-8b's army; Red's gold before and after: ADVEVENT:35; 4; 30900; closed; 30300; 1 Angel, 2 Swordsmen; 30300
- the date's words on days 1, 2, 7 and 8: month 1, week 1, day 1; month 1, week 1, day 2; month 1, week 1, day 7; month 1, week 2, day 1
- day 8: the week's message (its name): GENRLTXT:14, GENRLTXT:16 [names Beholder]; GENRLTXT:14: Red
- anything odd: the objects anchored in (8, 9) to (20, 11), class and subclass: 19,10 class 78 subclass 0; 10,10 class 17 subclass 24; 13,10 class 20 subclass 1; 16,10 class 17 subclass 58; the players' teams: red 0, blue 1, tan 2, green 3, orange 4, purple 5, teal 6, pink 7; Red's treasury at the start: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30900 gold; Blue's treasury at the start: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; D's record right after the visit: record 0; class 17.24; owner red; creatures 0: 1 Green Dragon; guards none; S's hire dialog as it opened: most 4; amount 0; ally 0; selected 0 (counts 4); max enabled, recruit disabled, close enabled, slider enabled; day 8: the date: month 1, week 2, day 1

## E10-9 — A Refugee Camp: its creature and count through a week (ADV-RFG-001, ADV-HIR-002, ADV-MON-003)

- Red hero **E10-9** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. The Refugee Camp K stands on row 10 (entrance (19, 10); it covers (18, 10) and (19, 10)). Day 1, Red's turn, before any move: read K's tile words (the creature +0x22 and the count +0x0 of (19, 10)) and the map object's owner, if any.
2. E10-9 stands at (19, 13) with one Angel. Walk to (19, 11), then click K's entrance (19, 10). Note the question; answer yes. Note the dialog's most. If the count is 2 or more, set the amount to **1** and click Recruit; if it is 1, buy nothing. Close the dialog if it is still open. Walk to (19, 12). Read K's words, E10-9's army and Red's treasury; then right-click K (E10-10's Red line).
3. Day 2, Red's turn: read K's words. Click K's entrance (19, 10) from (19, 12) (a walk to (19, 11), then the click). The question: yes. Note the most; click Max, then Recruit; close the dialog if it is still open. Click (19, 11) (off the entrance), then K's entrance (19, 10) again: note the message. Walk to (19, 12); read K's words and E10-9's army.
4. Day 7, Red's turn: read K's words.
5. Day 8, Red's turn, after the week's message and before any move: read K's words. Walk to (19, 11), then click K's entrance (19, 10). Note the question; answer yes. Note the dialog's most; close it without buying. Walk to (19, 12).

**Record**

| when | creature (+0x22) | count (+0x0) |
|---|---|---|
| day 1 before | tile 19,10,0; creature 131 (Phoenixes); count word 2 | tile 19,10,0; creature 131 (Phoenixes); count word 2 |
| day 1 after the purchase | tile 19,10,0; creature 131 (Phoenix); count word 1 | tile 19,10,0; creature 131 (Phoenix); count word 1 |
| day 2 before | tile 19,10,0; creature 131 (Phoenix); count word 1 | tile 19,10,0; creature 131 (Phoenix); count word 1 |
| day 2 after the purchases | tile 19,10,0; creature 131 (Phoenixes); count word 0 | tile 19,10,0; creature 131 (Phoenixes); count word 0 |
| day 7 | tile 19,10,0; creature 131 (Phoenixes); count word 0 | tile 19,10,0; creature 131 (Phoenixes); count word 0 |
| day 8 before the visit | tile 19,10,0; creature 31 (Obsidian Gargoyles); count word 9 | tile 19,10,0; creature 31 (Obsidian Gargoyles); count word 9 |

- day 1: K's owner (none?); the question's text; the dialog's most; what was bought; E10-9's army; Red's treasury: class 78; ADVEVENT:35; 2; closed; 1 Angel, 1 Phoenix; 30 wood, 14 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 28300 gold
- day 2: the question's text; the dialog's most; what Max and Recruit bought; the message of the second visit; E10-9's army: ADVEVENT:35; 1; closed; ADVEVENT:44; none; none; 1 Angel, 2 Phoenixes
- day 8: the question's text; the dialog's most: ADVEVENT:35; 9; most 9; amount 0; ally 0; selected 0 (counts 9); max enabled, recruit disabled, close enabled, slider enabled
- anything odd: K's entrance tile (19, 10): 0x00000002 class 78.131; K's object (19, 10): class 78; K's tile (18, 10): 0xffffffff class 78.0; K's hire dialog as it opened: most 2; amount 0; ally 0; selected 0 (counts 2); max enabled, recruit disabled, close enabled, slider enabled; K's word right after the visit: tile 19,10,0; creature 131 (Phoenix); count word 1; K's object after the visit: class 78; day 2: K's hire dialog as it opened: most 1; amount 0; ally 0; selected 0 (counts 1); max enabled, recruit disabled, close enabled, slider enabled; day 8: K's hire dialog as it opened: most 9; amount 0; ally 0; selected 0 (counts 9); max enabled, recruit disabled, close enabled, slider enabled

## E10-10 — What a player sees of dwellings and a Refugee Camp: the hint and the popup (ADV-OWN-003)

- Blue hero **E10-10** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, after E10-9's purchase: right-click K's entrance (19, 10) and note the popup's text (Red has visited K this week).
2. Day 1, Blue's turn: E10-10 stands at (14, 12) with one Pikeman and never moves; it sees row 10 from (9, 10) to (19, 10). D is Red's now (E10-8), F neutral, S Red's, K a Refugee Camp. Hover the mouse over D's (10, 10), F's (13, 10), S's (16, 10) and K's (19, 10) tiles in turn and note the status bar's hint at each; then right-click each and note the popup's text.

**Record**

- Red, day 1: K's popup: ?: [class 78 name]

| object | the hint (hover) | the popup (right-click) |
|---|---|---|
| D (Red's, flagged on day 1) | [CRGEN1 row 24 name] - [owned-by text of red] | GENRLTXT:198: [CRGEN1 row 24 name] /  / [owned-by text of red] |
| F (neutral) | [CRGEN4 row 1 name] | ?: [CRGEN4 row 1 name] |
| S (Red's at load) | [CRGEN1 row 58 name] - [owned-by text of red] | GENRLTXT:198: [CRGEN1 row 58 name] /  / [owned-by text of red] |
| K (the camp) | [class 78 name] | ?: [class 78 name] |

- anything odd: Red's hint over K: [class 78 name]; Blue, day 1: the date: month 1, week 1, day 1; Blue's treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30800 gold; D (Red's, flagged on day 1): hint bar and buffer: same; F (neutral): hint bar and buffer: same; S (Red's at load): hint bar and buffer: same; K (the camp): hint bar and buffer: same
