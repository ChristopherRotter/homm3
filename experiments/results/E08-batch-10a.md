# E08 batch E10a — results

> Played by the capture harness on 2026-11-03 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-10a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-10a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `8455031b`.** The map's description in the scenario list starts with "Stamp 8455031b.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

**Teams: red and blue.** The map makes them allies; any other player plays alone.

1. Copy `E08-batch-10a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**; Tan is the computer's (the map lets no human play it). Red and Blue are allied: note what the scenario's information shows of the teams. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), and at the start of each later turn (save 2 Blue's day 1, save 3 Red's day 2, save 4 Blue's day 2).
4. Most records are the game's memory: the dwelling records (game +0x4E39C, 0x5C bytes each: class +0x0, subclass +0x1, four creature types +0x4 and counts +0x14, the guards' seven types +0x1C and counts +0x38, the entrance +0x54, the owner +0x57; survey §2.1), the map's objects (class, subclass, anchor) and the tiles' blocked and visitable bits, the hire dialog (the most +0xAC, the amount +0xB8, the ally's flag +0x4C, whether its Max, Recruit and slider buttons are enabled, whether it is open), the players' teams (game +0x1F879) and treasuries, a hero's army and tile, a battle's terrain, special field, stacks at its opening and its sides' owners and controllers (`battle_manager`). The capture harness reads them (`clients/capture/batches/E08-batch-10a.toml`, E08 T90). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A visit: walk to the tile named, then click the object's entrance from there (twice).
6. Each battle by auto combat; a tactics phase is ended at once (its Start button). A level-up's dialog after a battle: take the left skill.
7. The hire dialog: **Max** sets the most, **Recruit** buys the amount set, **Close** (the dialog's exit) ends it. The most (+0xAC) is memory-only; by hand note the dialog's Available and the amount Max sets.
8. A dwelling's record is found in the vector by its entrance (+0x54), not by the tile's word, so a hero standing on the entrance after a visit hides nothing of it (a hero on a tile hides only the tile's word: its +0x14 holds it).
9. Order: day 1, Red's turn: Red's treasury (after the events' messages, E10-6's first line); E10-7's reads; E10-1; E10-3; E10-6; E10-4; Red's treasury (E10-4's last line); end the turn. Blue's turn: E10-6's Blue reads; end the turn. Tan plays its turn. Day 2, Red's turn: Red's treasury (after the messages); E10-5; Red's treasury; end the turn. Blue's turn: end the turn. Then the map is done.
10. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E10-1 — A hero visits three neutral dwellings of guarded creatures (ADV-DWL-002, ADV-DWL-004, ADV-OWN-001, GAM-BAT-001)

- Red hero **E10-1** (Edric, specialty creature Griffin): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn: E10-1 stands at (9, 12) with ten Archangels. Three neutral dwellings stand north of it: the Dragon Cliffs D (entrance (8, 10); it covers (7, 10) and (8, 10)), the Golem Factory F (entrance (13, 10); (12, 10) and (13, 10)) and the Elemental Conflux X (entrance (18, 10); it covers (17, 9) to (19, 10)). Before any move read the three records (owner, creatures and counts, guards).
2. Walk to (8, 11), then click D's entrance (8, 10). Note the question; answer yes. At the battle's opening note the terrain, the special field, the defending stacks (creature and count, by slot) and the defending side's owner. Auto combat. Note the messages after the battle and the next question; answer **no**. Read D's record.
3. Walk to (13, 11), then click F's entrance (13, 10). As for D: the question (yes), the opening, the messages, the next question (no), F's record.
4. Walk to (18, 11), then click X's entrance (18, 10). As for D: the question (yes), the opening, the messages, the next question (no), X's record. E10-1 ends the day on X's entrance.

**Record**

| dwelling | owner | creatures and counts | guards (creature and count by slot) |
|---|---|---|---|
| D before | none | 0: 1 Green Dragon | 0: 3 Green Dragons |
| F before | none | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems |
| X before | none | 0: 6 Air Elementals, 1: 5 Fire Elementals, 2: 4 Earth Elementals, 3: 6 Water Elementals | 0: 12 Earth Elementals |

- D: the question's text: GENRLTXT:422, ADVEVENT:35; size word: ARRAYTXT:176
- D: terrain, special field; the defending side's owner: terrain grass, special field none, ship flag 0, blocked hexes 81, 91, 92, 93, 94, 95, 96, 97, 98, 106, 107, 123; none (human 0, here 0)

| slot | creature | count |
|---|---|---|
| 0 | Green Dragons | 3 |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- D: the result; the messages; the next question's text: attacker; GENRLTXT:7; GENRLTXT:7: Red; GENRLTXT:422, ADVEVENT:35
- D after: owner, counts, guards: record 0; class 17.24; owner red; creatures 0: 1 Green Dragon; guards none
- F: the question's text: GENRLTXT:422, ADVEVENT:36; size word: ARRAYTXT:182
- F: terrain, special field; the defending side's owner: terrain grass, special field none, ship flag 0, blocked hexes 74, 75, 76, 77, 92, 93, 94, 95, 110, 111, 164, 165; none (human 0, here 0)

| slot | creature | count |
|---|---|---|
| 0 | Gold Golems | 9 |
| 1 | Diamond Golems | 6 |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- F: the result; the messages; the next question's text: attacker; none; none; GENRLTXT:422, ADVEVENT:36
- F after: owner, counts, guards: record 1; class 20.1; owner red; creatures 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems; guards none
- X: the question's text: GENRLTXT:422, ADVEVENT:36; size word: ARRAYTXT:182
- X: terrain, special field; the defending side's owner: terrain grass, special field none, ship flag 0, blocked hexes 61, 62, 72, 73, 77, 78, 79, 80, 95, 96, 97, 98, 132; none (human 0, here 0)

| slot | creature | count |
|---|---|---|
| 0 | Earth Elementals | 12 |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- X: the result; the messages; the next question's text: attacker; none; none; GENRLTXT:422, ADVEVENT:36
- X after: owner, counts, guards: record 2; class 20.0; owner red; creatures 0: 6 Air Elementals, 1: 5 Fire Elementals, 2: 4 Earth Elementals, 3: 6 Water Elementals; guards none
- anything odd: worn at the start (the starting bonus): Cape of Conjuring (shoulders); E10-1's army at the start: 0: 10 Archangels; D: the sides: attacker red (human 1, here 1); defender none (human 0, here 0); D: E10-1's tile after: 8,10,0; D: E10-1's army after: 0: 10 Archangels; F: the sides: attacker red (human 1, here 1); defender none (human 0, here 0); F: the level-ups: level 2: Offense or Learning, took left (Offense); F: E10-1's tile after: 13,10,0; F: E10-1's army after: 0: 10 Archangels; X: the sides: attacker red (human 1, here 1); defender none (human 0, here 0); X: E10-1's tile after: 18,10,0; X: E10-1's army after: 0: 10 Archangels

## E10-3 — A Guardhouse's free Pikemen: twice to one hero, then to a hero with no free slot (ADV-HIR-001, ADV-DWL-004, D-52)

- Red hero **E10-3** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel
- Red hero **E10-3b** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Archer, 1 Archer, 1 Griffin, 1 Swordsman, 1 Monk, 1 Cavalier, 1 Angel

1. Day 1, Red's turn: two neutral Guardhouses stand north of the heroes: P (entrance (30, 10); it covers (29, 10), (30, 10) and (29, 9)) and Q (entrance (35, 10); (34, 10), (35, 10) and (34, 9)). Read both records.
2. E10-3 stands at (30, 13) with two Angels. Walk to (30, 11), then click P's entrance (30, 10). Note the messages and the question; answer yes. Note the message; read E10-3's army and P's record.
3. Click (30, 11) (off the entrance), then (30, 10) again. Note the question; answer yes. Note the message; read E10-3's army and P's record.
4. E10-3b stands at (35, 13) with seven stacks: Archers, Archers, Griffins, Swordsmen, Monks, Cavaliers and Angels, one each. Walk to (35, 11), then click Q's entrance (35, 10). Note the question; answer yes. Note the message; read E10-3b's army and Q's record.

**Record**

- before: P's and Q's records (owner, Pikemen's count, guards): P: record 3; class 17.56; owner none; creatures 0: 14 Pikemen; guards none; Q: record 4; class 17.56; owner none; creatures 0: 14 Pikemen; guards none
- A's first visit: the question's text; the message after yes: ADVEVENT:35; GENRLTXT:424; none
- after it: E10-3's army (creature and count by slot); P's owner and count: 0: 2 Angels, 1: 14 Pikemen; record 3; class 17.56; owner red; creatures 0: 0 Pikemen; guards none
- A's second visit: the question's text; the message after yes: ADVEVENT:35; GENRLTXT:423; none
- after it: E10-3's army; P's owner and count: 0: 2 Angels, 1: 14 Pikemen; record 3; class 17.56; owner red; creatures 0: 0 Pikemen; guards none
- B's visit: the question's text; the message after yes: ADVEVENT:35; GENRLTXT:426; none
- after it: E10-3b's army (creature and count by slot); Q's owner and count: 0: 1 Archer, 1: 1 Archer, 2: 1 Griffin, 3: 1 Swordsman, 4: 1 Monk, 5: 1 Cavalier, 6: 1 Angel; record 4; class 17.56; owner red; creatures 0: 14 Pikemen; guards none
- anything odd: E10-3b's army at the start: 0: 1 Archer, 1: 1 Archer, 2: 1 Griffin, 3: 1 Swordsman, 4: 1 Monk, 5: 1 Cavalier, 6: 1 Angel; A's first visit: the screens passed, battles, E10-3's tile: none; no; 30,10,0; A's second visit: the screens passed, battles, E10-3's tile: none; no; 30,10,0; B's visit: the screens passed, battles, E10-3b's tile: none; no; 35,10,0

## E10-6 — Red's hero visits Blue's Barracks and Blue's guarded Dragon Cliffs; Red and Blue allied (ADV-DWL-004, ADV-HIR-002, ADV-OWN-001)

- Red hero **E10-6** (Cuthbert, specialty spell Weakness): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel
- Blue hero **E10-6c** (Loynis, specialty spell Prayer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Tan (the computer) hero **E10-6t** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before E10-4: Red's events leave 1 000 gold and no crystal. Blue owns the Barracks U (entrance (55, 10); it covers (54, 10) and (55, 10)) and the Dragon Cliffs V (entrance (59, 10); (58, 10) and (59, 10)), with its guards. Read Red's treasury, the players' team bytes (game +0x1F879) and U's and V's records.
2. E10-6 stands at (55, 13) with one Angel. Walk to (55, 11), then click U's entrance (55, 10). Note any message and the question; answer yes. Note the dialog: the most (+0xAC), the amount, which buttons are enabled (Max, Recruit, the slider), the ally's flag (+0x4C). Click Max; **if Recruit is enabled, do not click it**: note that. Close the dialog. Read U's record.
3. Walk to (59, 11), then click V's entrance (59, 10). Note any message: **if a guards' question shows, answer no** and note it. Else the question to recruit: yes; note the dialog as at U (Recruit never clicked while enabled) and close it. Read V's record. Walk to (59, 12).
4. Blue's turn of day 1: read U's and V's records again. Blue's hero E10-6c (at (60, 24)) never moves.

**Record**

- the scenario's information on the teams; the team bytes of Red, Blue and Tan: scenario information: not read (the harness starts the game from the scenario list and reads no screen text); team bytes: red 0, blue 0, tan 1, green 0, orange 0, purple 0, teal 0, pink 0
- before: Red's treasury (gold, crystal): 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 1000 gold
- before: U's and V's records (owner, counts, guards): U: record 7; class 17.58; owner blue; creatures 0: 4 Swordsmen; guards none; V: record 8; class 17.24; owner blue; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons
- U: messages before the question; the question's text: none; ADVEVENT:35
- U's dialog: the most; the amount; the buttons enabled; +0x4C; what Max did: as it opened: most 3; amount 0; ally 1; selected 0 (counts 4); max disabled, recruit disabled, close enabled, slider disabled; after Max: most 3; amount 0; ally 1; selected 0 (counts 4); max disabled, recruit disabled, close enabled, slider disabled
- after U: U's record (owner, count); E10-6's army; Red's treasury: record 7; class 17.58; owner blue; creatures 0: 4 Swordsmen; guards none; 0: 1 Angel; 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 1000 gold
- V: messages; the guards' question, if any; the question to recruit: none; ADVEVENT:35; size word: none
- V's dialog: the most; the buttons enabled; +0x4C: most 0; amount 0; ally 1; selected 0 (counts 1); max disabled, recruit disabled, close enabled, slider disabled
- after V: V's record (owner, count, guards); Red's treasury: record 8; class 17.24; owner blue; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons; 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 1000 gold
- Blue's turn: U's and V's records: U: record 7; class 17.58; owner blue; creatures 0: 4 Swordsmen; guards none; V: record 8; class 17.24; owner blue; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; Tan before (day 1, Red's turn): treasury; E10-6t's tile and points: 11 wood, 2 mercury, 11 ore, 2 sulfur, 2 crystal, 2 gems, 5000 gold; 66,66,0; 1560; U: the screens passed, battles, E10-6's tile: hire; no; 55,10,0; U: the dialog's items as it opened (id:state): 538:0x6, 534:0x6, 542:0x6, 30721:0x6, 65535:0x6, 30722:0x26, 65535:0x6, 532:0x26, 65535:0x6, 561:0x6, 560:0x6, 531:0x0, 530:0x6, 529:0x0, 528:0x6, 527:0x6, 559:0x2e, 526:0x6, 525:0x6, 521:0x6, 520:0x6, 516:0x0, 512:0x6, 508:0x0, 504:0x6, 500:0x6, 550:0x6, 554:0x6, 553:0x6, 552:0x6, 551:0x6, 0:0x6; V: the screens passed, battles, E10-6's tile, army: hire; no; 59,10,0; 0: 1 Angel; V: the dialog's items as it opened (id:state): 538:0x6, 534:0x6, 542:0x6, 30721:0x6, 65535:0x6, 30722:0x26, 65535:0x6, 532:0x26, 65535:0x6, 561:0x6, 560:0x6, 531:0x6, 530:0x6, 529:0x6, 528:0x6, 527:0x6, 559:0x2e, 526:0x6, 525:0x6, 521:0x6, 520:0x6, 516:0x6, 512:0x6, 508:0x6, 504:0x6, 500:0x6, 550:0x6, 554:0x6, 553:0x6, 552:0x6, 551:0x6, 0:0x6; Blue, day 1, the turn's start: treasury; the date: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30000 gold; month 1, week 1, day 1; Tan, day 1, Blue's turn: treasury; E10-6t's tile and points: 11 wood, 2 mercury, 11 ore, 2 sulfur, 2 crystal, 2 gems, 5000 gold; 66,66,0; 1560; the team bytes now: red 0, blue 0, tan 1, green 0, orange 0, purple 0, teal 0, pink 0; Blue, day 2, the turn's start: treasury; the date; U's and V's records: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30000 gold; month 1, week 1, day 2; record 7; class 17.58; owner blue; creatures 0: 4 Swordsmen; guards none; record 8; class 17.24; owner blue; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons

## E10-4 — Red's own Barracks: the most it may buy with 1 000 gold, the purchase (ADV-HIR-002, GAM-DEC-008)

- Red hero **E10-4** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 1, Red's turn, after E10-6: Red owns the Barracks S (entrance (41, 10); it covers (40, 10) and (41, 10)). Read Red's treasury and S's record.
2. E10-4 stands at (41, 13) with one Angel. Walk to (41, 11), then click S's entrance (41, 10). Note the question; answer yes. Note the dialog's most (+0xAC; by hand its Available) and amount before any click. Click Max, then Recruit. Note whether the dialog is still open; if it is, note the most it shows now and close it.
3. Read E10-4's army, Red's treasury and S's record.

**Record**

- before: Red's treasury (gold, crystal); S's record (owner, count, guards): 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 1000 gold; record 5; class 17.58; owner red; creatures 0: 4 Swordsmen; guards none
- the question's text: none; ADVEVENT:35
- the dialog before any click: the most; the amount: most 3; amount 0; ally 0; selected 0 (counts 4); max enabled, recruit disabled, close enabled, slider enabled
- after Recruit: the dialog open or closed (if open, its most): closed
- after: E10-4's army (creature and count by slot); Red's treasury; S's count: 0: 1 Angel, 1: 3 Swordsmen; 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 100 gold; record 5; class 17.58; owner red; creatures 0: 1 Swordsman; guards none
- anything odd: E10-4: the screens passed, E10-4's tile: hire; 41,10,0; Red, day 1, the turn's end: treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 100 gold

## E10-5 — Red's own Dragon Cliffs with 10 000 gold: no crystal, then one (ADV-HIR-002, ADV-DWL-004)

- Red hero **E10-5** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Angel

1. Day 2, Red's turn: Red's events leave 10 000 gold and no crystal. Red owns the Dragon Cliffs C (entrance (47, 10); it covers (46, 10) and (47, 10)); one crystal lies at (48, 12). Read Red's treasury and C's record.
2. E10-5 stands at (47, 13) with one Angel (it does not move on day 1). Walk to (47, 11), then click C's entrance (47, 10). Note any message and the question; answer yes. Note the dialog's most and which buttons are enabled. Close it.
3. Click (47, 11) (off the entrance), then click the crystal (48, 12): E10-5 picks it up from (47, 11). Read Red's treasury.
4. Click C's entrance (47, 10) again. The question: yes. Note the dialog's most and the buttons. Click Max, then Recruit; note whether the dialog is still open (close it if so). Read E10-5's army, Red's treasury and C's record.

**Record**

- before: Red's treasury (gold, crystal); C's record (owner, count, guards): 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 10000 gold; record 6; class 17.24; owner red; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons
- the first visit: messages before the question; the question's text: none; ADVEVENT:35
- the first dialog: the most; the buttons enabled: most 0; amount 0; ally 0; selected 0 (counts 1); max disabled, recruit disabled, close enabled, slider disabled
- after the crystal: Red's treasury (gold, crystal): 30 wood, 15 mercury, 30 ore, 15 sulfur, 1 crystal, 15 gems, 10000 gold
- the second dialog: the most; the buttons enabled: most 1; amount 0; ally 0; selected 0 (counts 1); max enabled, recruit disabled, close enabled, slider enabled
- after Recruit: the dialog open or closed; E10-5's army; Red's treasury; C's record: closed; 0: 1 Angel, 1: 1 Green Dragon; 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 7600 gold; record 6; class 17.24; owner red; creatures 0: 0 Green Dragons; guards 0: 3 Green Dragons
- anything odd: Red, day 2, the turn's start: the date: month 1, week 1, day 2; Tan, day 2 (after its turn): treasury; E10-6t's tile and points: 11 wood, 2 mercury, 11 ore, 2 sulfur, 2 crystal, 2 gems, 5000 gold; 66,66,0; 1560; day 2: S's, U's and V's records: record 5; class 17.58; owner red; creatures 0: 1 Swordsman; guards none; record 7; class 17.58; owner blue; creatures 0: 4 Swordsmen; guards none; record 8; class 17.24; owner blue; creatures 0: 1 Green Dragon; guards 0: 3 Green Dragons; first visit: the screens passed, battles, E10-5's tile: hire; no; 47,10,0; first visit: the dialog's items as it opened (id:state): 538:0x6, 534:0x6, 542:0x6, 30721:0x6, 65535:0x6, 30722:0x26, 65535:0x6, 532:0x26, 65535:0x6, 561:0x6, 560:0x6, 531:0x6, 530:0x6, 529:0x6, 528:0x6, 527:0x6, 559:0x2e, 526:0x6, 525:0x6, 521:0x6, 520:0x6, 516:0x6, 512:0x6, 508:0x6, 504:0x6, 500:0x6, 550:0x6, 554:0x6, 553:0x6, 552:0x6, 551:0x6, 0:0x6; the pile (clicked from (47, 11), the hero's tile): messages, E10-5's tile after, arrived, points: none; 47,11,0; no; 1459; second visit: the questions, the screens passed, E10-5's tile: ADVEVENT:35; hire; 47,10,0; second visit: the dialog's items as it opened (id:state): 538:0x6, 534:0x6, 542:0x6, 30721:0x6, 65535:0x6, 30722:0x26, 65535:0x6, 532:0x6, 65535:0x6, 561:0x6, 560:0x6, 531:0x6, 530:0x6, 529:0x6, 528:0x6, 527:0x6, 559:0x6, 526:0x6, 525:0x6, 521:0x6, 520:0x6, 516:0x6, 512:0x6, 508:0x6, 504:0x6, 500:0x6, 550:0x6, 554:0x6, 553:0x6, 552:0x6, 551:0x6, 0:0x6; Red, day 2, the turn's end: treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 0 crystal, 15 gems, 7600 gold

## E10-7 — Three random dwellings: what each became when the game started (ADV-DWL-005, ADV-DWL-002)

- Red hero **E10-7** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before any hero moves: three random dwellings were placed on row 30, each a 2 × 2 picture ending on its anchor: R1 at (12, 30) (any level 7 to 7 of Castle), R2 at (16, 30) (a level-3 one of Tower), R3 at (21, 30) (any level 2 to 2 of Rampart). E10-7 stands at (16, 31), where it sees them all, and never moves.
2. For each, read the map object at or around its tile (class, subclass, anchor), which of the tiles (10, 29) to (22, 30) are blocked and which visitable, and the dwelling record at its entrance (creatures and counts, guards, owner).

**Record**

| object | class | subclass | anchor | entrance | blocked tiles |
|---|---|---|---|---|---|
| R1 | 13,30 class 17 subclass 8 | 13,30 class 17 subclass 8 | 13,30 class 17 subclass 8 | record at 12,30: record 9; class 17.8; owner none; creatures 0: 1 Angel; guards 0: 3 Angels; record at 13,30: none | blocked 11,30; visitable 12,30 |
| R2 | 16,30 class 20 subclass 1; 17,31 class 34 subclass 0 | 16,30 class 20 subclass 1; 17,31 class 34 subclass 0 | 16,30 class 20 subclass 1; 17,31 class 34 subclass 0 | record at 16,30: record 10; class 20.1; owner none; creatures 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems; guards 0: 9 Gold Golems, 1: 6 Diamond Golems; record at 15,30: none | blocked 15,30; visitable 16,30 |
| R3 | 21,30 class 17 subclass 12 | 21,30 class 17 subclass 12 | 21,30 class 17 subclass 12 | record at 20,30: record 11; class 17.12; owner none; creatures 0: 8 Dwarves; guards none; record at 21,30: none | blocked 21,30; visitable 20,30 |

| object | creatures and counts | guards (creature and count by slot) | owner |
|---|---|---|---|
| R1 | 0: 1 Angel | 0: 3 Angels | none |
| R2 | 0: 6 Stone Golems, 1: 6 Iron Golems, 2: 3 Gold Golems, 3: 2 Diamond Golems | 0: 9 Gold Golems, 1: 6 Diamond Golems | none |
| R3 | 0: 8 Dwarves | none | none |

- anything odd: every object anchored in (9, 28) to (23, 31): 13,30 class 17 subclass 8; 16,30 class 20 subclass 1; 21,30 class 17 subclass 12; 17,31 class 34 subclass 0; the tiles (10, 29) to (22, 30): blocked and visitable: blocked 11,30 15,30 21,30; visitable 12,30 16,30 20,30; the records at the neighbouring tiles (13, 30), (15, 30), (21, 30): none; none; none
