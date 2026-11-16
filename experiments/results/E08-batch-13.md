# E08 batch E13 — results

> Played by the capture harness on 2026-11-16 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-13.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory. Started again 1 time(s) for the recipe's `require` (an Imp Cache of level 1 (1000 gold) among I2, I3, I4 missing).

Map file: `E08-batch-13.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `23ef005b`.** The map's description in the scenario list starts with "Stamp 23ef005b.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-13.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1), at the start of Blue's turn (save 2) and at the start of Red's day 2 (save 3).
4. Most records are the game's memory: a hero's tile, its movement points (hero +0x4D), army, experience, level, status bits (+0x105) and morale byte (+0x11A), its worn artifacts and backpack; a bank's record (the banks' vector at game +0x4E3D8, 0x6C bytes each) and its object's word (the visited bits 5 to 12, the record's index 13 to 24, the looted bit 25); the questions, messages and dialogs in their order; the treasury; the path's marks after one click; the hint (the status bar's text over a tile); and as a battle opens its stacks with their hexes, the tactics phase, the obstacles, the war machines and the battlefield. The capture harness reads them (`clients/capture/batches/E08-batch-13.toml`, E08 T120). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). A bank's question: answer as the procedure says (yes: attack or search; no: leave).
6. Battles against a bank's guards by auto combat unless the procedure says otherwise. A level-up's dialog: take the left skill. An army dialog (a reward's creatures with no room): note both bars, then close it with OK without a move.
7. Order: day 1, Red's turn: E13-1's reads, E13-2, E13-9, E13-3, E13-6, E13-7, E13-5 (the coast and the boarding); end the turn. Blue's turn: E13-1's Blue reads, E13-12; end the turn. Day 2, Red's turn: E13-10 (in E13-3's encounter), E13-5's Derelict Ship, E13-8 (in E13-7's encounter); end the turn. Then the map is done.
8. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E13-1 — Every bank's record at the start (E13-1); Blue's hints over the declined Stockpile and Crypt (E13-2) and the Imp Cache left after a retreat (E13-9) (ADV-BNK-001, ADV-BNK-002, ADV-BNK-006, D-65, D-66, D-68)

- Blue hero **E13-1** (Melodia, specialty spell Fortune): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. **E13-1.** Day 1, Red's turn, first, before any hero moves: read every bank's record and its object's word (27 banks, by entrance): I1 (10, 4), S1 (6, 8), C1 (10, 8), S2 (14, 8), T1 (10, 20), I2 (14, 21), I3 (7, 21), N1 (8, 24), I4 (12, 24), G2 (12, 34), U1 (16, 44), C2 (20, 44), C3 (24, 44), W1 (27, 46), U2 (16, 50), D1 (30, 54), D2 (39, 40), W2 (50, 4), W3 (56, 5), W4 (62, 4), T2 (5, 64), G1 (12, 64), M1 (17, 64), M2 (23, 64), N2 (29, 64), F1 (35, 64), F2 (41, 64). For each: the guards by slot (creature, count), the resources (wood, mercury, ore, sulfur, crystal, gems, gold), the creature and its count, the artifacts in the vector's order (name; class), the word's record index.
2. **The draws' check** (before any move): if none of the Imp Caches I2, I3 and I4 is of level 1 (twenty Imps a stack, 1 000 gold and 2 Mercury), or both U1 and U2 hold four relics, restart the scenario (a new game: new draws), note it and read again; at most five starts, then go on with the last.
3. **Blue's turn** (day 1, after Red's): E13-1 (Blue's Melodia at (8, 5)) does not move. Hover each bank's tile beside its entrance and note the hint: the Stockpile S1 (7, 8), the Crypt C1 (11, 8), the Imp Cache I1 (9, 4) (the hint makes no entrance test, survey §7). Also read the three words' visited bits.

**Record**

| bank | entrance | guards by slot | resources | creature × count | artifacts (name; class) | record index |
|---|---|---|---|---|---|---|
| I1 | (10, 4) | 0: 60 Imps, 1: 60 Imps, 2: 60 Familiars, 3: 60 Imps, 4: 60 Imps | 0 wood, 6 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 3000 gold | none | none | 0; visited none, looted no (word 0xfc00001f) |
| S1 | (6, 8) | 0: 8 Cyclopes, 1: 8 Cyclopes, 2: 8 Cyclopes, 3: 8 Cyclopes, 4: 8 Cyclopes | 8 wood, 8 mercury, 8 ore, 8 sulfur, 8 crystal, 8 gems, 0 gold | none | none | 4; visited none, looted no (word 0xfc00801f) |
| C1 | (10, 8) | 0: 10 Skeletons, 1: 10 Walking Dead, 2: 10 Walking Dead, 3: 10 Skeletons, 4: 10 Skeletons | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 1500 gold | none | none | 5; visited none, looted no (word 0xfc00a01f) |
| S2 | (14, 8) | 0: 6 Cyclopes, 1: 6 Cyclopes, 2: 6 Cyclopes, 3: 6 Cyclopes, 4: 6 Cyclopes | 6 wood, 6 mercury, 6 ore, 6 sulfur, 6 crystal, 6 gems, 0 gold | none | none | 6; visited none, looted no (word 0xfc00c01f) |
| T1 | (10, 20) | 0: 15 Dwarves, 1: 15 Dwarves, 2: 15 Battle Dwarves, 3: 15 Dwarves, 4: 15 Dwarves | 0 wood, 0 mercury, 0 ore, 0 sulfur, 3 crystal, 0 gems, 4000 gold | none | none | 7; visited none, looted no (word 0xfc00e01f) |
| I2 | (14, 21) | 0: 40 Imps, 1: 40 Imps, 2: 40 Familiars, 3: 40 Imps, 4: 40 Imps | 0 wood, 4 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 2000 gold | none | none | 9; visited none, looted no (word 0xfc01201f) |
| I3 | (7, 21) | 0: 20 Imps, 1: 20 Imps, 2: 20 Imps, 3: 20 Imps, 4: 20 Imps | 0 wood, 2 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 1000 gold | none | none | 8; visited none, looted no (word 0xfc01001f) |
| N1 | (8, 24) | 0: 4 Nagas, 1: 4 Nagas, 2: 4 Nagas, 3: 4 Nagas, 4: 4 Nagas | 0 wood, 0 mercury, 0 ore, 16 sulfur, 0 crystal, 0 gems, 8000 gold | none | none | 10; visited none, looted no (word 0xfc01401f) |
| I4 | (12, 24) | 0: 20 Imps, 1: 20 Imps, 2: 20 Familiars, 3: 20 Imps, 4: 20 Imps | 0 wood, 2 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 1000 gold | none | none | 11; visited none, looted no (word 0xfc01601f) |
| G2 | (12, 34) | 0: 30 Griffins, 1: 30 Griffins, 2: 30 Griffins, 3: 30 Griffins, 4: 30 Griffins | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 0 gold | 3 Angels | none | 12; visited none, looted no (word 0xfc01801f) |
| U1 | (16, 44) | 0: 8 Green Dragons, 1: 6 Red Dragons, 2: 3 Gold Dragons, 3: 2 Black Dragons | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 30000 gold | none | Thunder Helmet (relic), Sentinel's Shield (relic), Sword of Hellfire (major), Crown of the Supreme Magi (minor) | 14; visited none, looted no (word 0xfc01c01f) |
| C2 | (20, 44) | 0: 20 Skeletons, 1: 20 Walking Dead, 2: 10 Wights, 3: 5 Vampires | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 2500 gold | none | Quiet Eye of the Dragon (treasure) | 15; visited none, looted no (word 0xfc01e01f) |
| C3 | (24, 44) | 0: 13 Skeletons, 1: 10 Walking Dead, 2: 5 Wights, 3: 10 Walking Dead, 4: 12 Skeletons | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 2000 gold | none | none | 16; visited none, looted no (word 0xfc02001f) |
| W1 | (27, 46) | 0: 2 Wights, 1: 2 Wights, 2: 2 Wights, 3: 2 Wights, 4: 2 Wights | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 2000 gold | none | none | 17; visited none, looted no (word 0xfc02201f) |
| U2 | (16, 50) | 0: 8 Green Dragons, 1: 5 Red Dragons, 2: 2 Gold Dragons, 3: 1 Black Dragon | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 20000 gold | none | Angel Wings (relic), Breastplate of Brimstone (major), Dragon Wing Tabard (minor), Ladybird of Luck (treasure) | 18; visited none, looted no (word 0xfc02401f) |
| D1 | (30, 54) | 0: 6 Water Elementals, 1: 6 Water Elementals, 2: 6 Water Elementals, 3: 6 Water Elementals, 4: 6 Water Elementals | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 3000 gold | none | Crest of Valor (treasure) | 19; visited none, looted no (word 0xfc02601f) |
| D2 | (39, 40) | 0: 6 Water Elementals, 1: 6 Water Elementals, 2: 6 Water Elementals, 3: 6 Water Elementals, 4: 6 Water Elementals | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 3000 gold | none | Collar of Conjuring (treasure) | 13; visited none, looted no (word 0xfc01a01f) |
| W2 | (50, 4) | 0: 5 Wights, 1: 5 Wights, 2: 5 Wights, 3: 5 Wights, 4: 5 Wights | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 4000 gold | none | Talisman of Mana (treasure) | 1; visited none, looted no (word 0xfc00201f) |
| W3 | (56, 5) | 0: 5 Wights, 1: 5 Wights, 2: 5 Wights, 3: 5 Wights, 4: 5 Wights | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 4000 gold | none | Skull Helmet (treasure) | 3; visited none, looted no (word 0xfc00601f) |
| W4 | (62, 4) | 0: 5 Wights, 1: 5 Wights, 2: 5 Wights, 3: 5 Wights, 4: 5 Wights | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 4000 gold | none | Pendant of Total Recall (treasure) | 2; visited none, looted no (word 0xfc00401f) |
| T2 | (5, 64) | 0: 15 Dwarves, 1: 15 Dwarves, 2: 15 Dwarves, 3: 15 Dwarves, 4: 15 Dwarves | 0 wood, 0 mercury, 0 ore, 0 sulfur, 3 crystal, 0 gems, 4000 gold | none | none | 20; visited none, looted no (word 0xfc02801f) |
| G1 | (12, 64) | 0: 10 Griffins, 1: 10 Griffins, 2: 10 Griffins, 3: 10 Griffins, 4: 10 Griffins | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 0 gold | 1 Angel | none | 21; visited none, looted no (word 0xfc02a01f) |
| M1 | (17, 64) | 0: 8 Medusas, 1: 8 Medusas, 2: 8 Medusas, 3: 8 Medusas, 4: 8 Medusas | 0 wood, 0 mercury, 0 ore, 8 sulfur, 0 crystal, 0 gems, 4000 gold | none | none | 22; visited none, looted no (word 0xfc02c01f) |
| M2 | (23, 64) | 0: 8 Medusas, 1: 8 Medusas, 2: 8 Medusas, 3: 8 Medusas, 4: 8 Medusas | 0 wood, 0 mercury, 0 ore, 8 sulfur, 0 crystal, 0 gems, 4000 gold | none | none | 23; visited none, looted no (word 0xfc02e01f) |
| N2 | (29, 64) | 0: 2 Nagas, 1: 2 Nagas, 2: 2 Nagas, 3: 2 Nagas, 4: 2 Nagas | 0 wood, 0 mercury, 0 ore, 8 sulfur, 0 crystal, 0 gems, 4000 gold | none | none | 24; visited none, looted no (word 0xfc03001f) |
| F1 | (35, 64) | 0: 6 Dragon Flies, 1: 6 Dragon Flies, 2: 6 Dragon Flies, 3: 6 Dragon Flies, 4: 6 Dragon Flies | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 0 gold | 4 Wyverns | none | 25; visited none, looted no (word 0xfc03201f) |
| F2 | (41, 64) | 0: 12 Dragon Flies, 1: 12 Dragon Flies, 2: 12 Dragon Flies, 3: 12 Dragon Flies, 4: 12 Dragon Flies | 0 wood, 0 mercury, 0 ore, 0 sulfur, 0 crystal, 0 gems, 0 gold | 8 Wyverns | none | 26; visited none, looted no (word 0xfc03401f) |

- the starts needed (1 to 5): 2 (start 1: an Imp Cache of level 1 (1000 gold) among I2, I3, I4 missing)
- Blue's hints: S1 (7, 8); C1 (11, 8); I1 (9, 4): 7,8: [CRBANKS row 2 name] [GENRLTXT 354]; 11,8: [CRBANKS row 38 name] [GENRLTXT 354]; 9,4: [CRBANKS row 14 name] [GENRLTXT 354]
- the words' visited bits on Blue's turn: S1; C1; I1: S1: visited red, looted no (word 0xfc00803f); C1: visited none, looted no (word 0xfc00a01f); I1: visited red, looted no (word 0xfc00003f)
- anything odd: the date at the start: month 1, week 1, day 1; the game's version: SoD; the difficulty: easy

## E13-2 — A declined Cyclops Stockpile and a declined Crypt, their hints for Red (E13-2); a Stockpile declined, visited again and won: where the hero stands (E13-11) (D-66, ADV-BNK-003, ADV-BNK-006, ADV-VIS-006)

- Red hero **E13-2** (Adela, specialty spell Bless): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Archangel

1. **E13-2.** Day 1, Red's turn, after E13-1's reads: E13-2 stands at (6, 10) with twenty Archangels. Read its points. Click the Stockpile S1's entrance (6, 8) twice: E13-2 walks onto it. Note every question and message in order and answer the question **no**. Note E13-2's tile and points, S1's word, and the hint over S1's other tile (7, 8) (the entrance shows the hero).
2. Click the Crypt C1's entrance (10, 8) twice: E13-2 walks onto it. Note every question and message and answer **no**. Note E13-2's tile and points, C1's word and the hint over C1's tile (11, 8).
3. **E13-11.** Click the Stockpile S2's entrance (14, 8) twice: E13-2 walks onto it. Note every question and message, E13-2's tile and points at the question, and answer **no**. Note E13-2's tile, points and S2's word.
4. Press Space (the visit of the tile the hero stands on). Note every question and message; if nothing comes, step to (14, 9) and back onto (14, 8) and note that. Answer the question **yes**. As the battle opens note each side's stacks and their hexes; then auto combat. After it note every dialog and message in order (the level-ups: the left skill; note the skills taken), then E13-2's tile, points, army and experience, S2's word, the hint over S2's other tile (15, 8) and Red's treasury before and after.

**Record**

- E13-2 before: points; Red's treasury: points: 2000; treasury: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold
- S1: questions and messages (in order): the path: 6,8 6,9; in order: question ADVEVENT:32 (no)
- after no at S1: E13-2's tile; points; S1's word (bits 5 to 12, 25); the hint over (7, 8): tile: 6,8,0; points: 1800; S1: visited red, looted no (word 0xfc00803f); the hint: 7,8: [CRBANKS row 2 name] ([GENRLTXT 203] [ARRAYTXT 185] [creature 94 plural])
- C1: questions and messages (in order): the path: 10,8 7,9 8,9 9,9; in order: question ADVEVENT:119 (no)
- after no at C1: E13-2's tile; points; C1's word; the hint over (11, 8): tile: 10,8,0; points: 1318; C1: visited none, looted no (word 0xfc00a01f); the hint: 11,8: [CRBANKS row 38 name] [GENRLTXT 354]
- S2: questions and messages; at the question E13-2's tile; points: the path: 14,8 11,9 12,9 13,9; in order: question ADVEVENT:32 (no); at the question: ADVEVENT:32 (no) on 14,8 with 836
- after no at S2: E13-2's tile; points; S2's word: tile: 14,8,0; points: 836; S2: visited red, looted no (word 0xfc00c03f)
- Space on S2: questions and messages (or the step off and back): the key: nothing; in order: question ADVEVENT:32 (yes); battle (the hero on 14,8); level-up (E13-2, level 2); level-up (E13-2, level 3); message GENRLTXT:7; message ADVEVENT:34; at the question: ADVEVENT:32 (yes) on 14,8 with 836
- the battle as it opens: E13-2's stacks (creature, count, hexes); the guards' (creature, count, hexes): attacker slot 0: 20 Archangels 57-58 | defender slot 0: 6 Cyclopes 15, slot 1: 6 Cyclopes 185, slot 2: 6 Cyclopes 172, slot 3: 6 Cyclopes 2, slot 4: 6 Cyclopes 100; tactics phase: no; obstacles: none; war machines: none; field: terrain grass, special field none, ship flag 0, blocked hexes none
- after the battle: dialogs and messages in order; E13-2's tile; points; army; experience: in order: question ADVEVENT:32 (yes); battle (the hero on 14,8); level-up (E13-2, level 2); level-up (E13-2, level 3); message GENRLTXT:7; message ADVEVENT:34; the level-ups: level 2: Scouting or Luck, took left (Scouting); level 3: Scouting or Wisdom, took left (Scouting); the messages' pictures: GENRLTXT:7: Red; ADVEVENT:34: 6 gems, 6 crystal, 6 sulfur, 6 ore, 6 mercury, 6 wood; tile: 14,8,0; points: 836; army: 0: 20 Archangels; experience: 2100; skills: Scouting advanced
- after it: S2's word; the hint over (15, 8); Red's treasury: S2: record 6; guards empty; resources 6 wood, 6 mercury, 6 ore, 6 sulfur, 6 crystal, 6 gems, 0 gold; creature none; artifacts none; visited red, looted yes (word 0xfe00c03f); treasury before: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold; after: 36 wood, 21 mercury, 36 ore, 21 sulfur, 21 crystal, 21 gems, 30500 gold; the hint: 15,8: [CRBANKS row 2 name] [GENRLTXT 353]
- anything odd: 

## E13-9 — A weak hero attacks an Imp Cache and retreats: the record after, the hint (ADV-BNK-004, ADV-BNK-005, ADV-BNK-006)

- Red hero **E13-9** (Uland, specialty spell Cure): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 20 Wood Elf

1. Day 1, Red's turn, after E13-2: E13-9 stands at (12, 4) with twenty Wood Elves. Read its points and I1's record. Click the Imp Cache I1's entrance (10, 4) twice: E13-9 walks onto it. Note every question and message and answer **yes**.
2. As the battle opens note each side's stacks and their hexes. At E13-9's first turn: shoot the guard stack nearest the Wood Elves (note which, and how many die); if a guard stack stands beside the Elves, attack it instead and note it; if the guards act first, note what they do. Then, at the next turn of E13-9's, **retreat** (the retreat button, then yes). Just before the retreat note each guard stack's count by slot. Note the battle's end.
3. After it: I1's record (the guards by slot) and its word, whether E13-9 is still on the map, and the hint over I1's tile (9, 4).

**Record**

- E13-9 before: points; I1's record (guards by slot): points: 1700; I1: 0: 60 Imps, 1: 60 Imps, 2: 60 Familiars, 3: 60 Imps, 4: 60 Imps
- I1: questions and messages: the path: 10,4 11,4; in order: question ADVEVENT:32 (yes); battle (the hero on 10,4); at the question: ADVEVENT:32 (yes) on 10,4 with 1500
- the battle as it opens: E13-9's stacks and hexes; the guards' (creature, count, hexes): attacker slot 0: 20 Wood Elves 57 | defender slot 0: 60 Imps 15, slot 1: 60 Imps 185, slot 2: 60 Familiars 172, slot 3: 60 Imps 2, slot 4: 60 Imps 100; tactics phase: no; obstacles: none; war machines: none; shots: attacker 20 Wood Elves 24
- the shot: the target; the guards' losses; anything the guards did before it: the Elves: shoots 60 Imps at hex 2 (cursor frame 3); shots: 24; then: 23; the guards as the Elves' turn came: slot 0: 60 Imps, slot 1: 60 Imps, slot 2: 60 Familiars, slot 3: 60 Imps, slot 4: 60 Imps; after the shot: slot 0: 60 Imps, slot 1: 60 Imps, slot 2: 51 Familiars, slot 3: 35 Imps, slot 4: 60 Imps
- just before the retreat: each guard stack's count by slot: slot 0: 60 Imps, slot 1: 60 Imps, slot 2: 51 Familiars, slot 3: 35 Imps, slot 4: 60 Imps
- the retreat: the battle's end: winner: defender; retreated: attacker; the stacks at the end: attacker 12 Wood Elves | defender 60 Imps, 60 Imps, 51 Familiars, 35 Imps, 60 Imps; then: question ADVEVENT:32 (yes); battle (the hero on 10,4)
- after it: I1's record (guards by slot); I1's word; E13-9 on the map?; the hint over I1: I1: 0: 60 Imps, 1: 60 Imps, 2: 51 Familiars, 3: 35 Imps, 4: 60 Imps; word: visited red, looted no (word 0xfc00003f); on the map: no; the hint (E13-3 selected): 9,4: [CRBANKS row 14 name] ([GENRLTXT 203] [ARRAYTXT 194] [GENRLTXT 44])
- anything odd: (agent, T120) the Wood Elves' shots 24 then 23 in runs 2 and 3 (no Ammo Cart); a killed stack reads `0 Imps` in the plan's by-slot lines, the record after holds the survivors only

## E13-3 — The bank battle's rules: Expert Tactics, a Ballista, an Ammo Cart and four stacks in slots 0, 2, 4, 6 against a Dwarven Treasury (E13-3); day 2: an Imp Cache of level 1 and a Naga Bank won, the treasury (E13-10) (BAT-TAC-001, BAT-GEO-004, BAT-WAR-001, BAT-WAR-004, BAT-SET-001, ADV-BNK-004, D-65, ADV-BNK-005)

- Red hero **E13-3** (Tyris, specialty creature Cavalier): attack 10, defense 10, power 1, knowledge 1; Expert Tactics; Ballista (war machine 1), Ammo Cart (war machine 2); no spells; army 10 Archangel, 20 Pikeman, 10 Swordsman, 10 Archer (slots 0, 2, 4, 6)

1. **E13-3.** Day 1, Red's turn, after E13-9: E13-3 stands at (10, 22) with Expert Tactics, a Ballista and an Ammo Cart, in tight formation, its army in slots 0, 2, 4 and 6: ten Archangels, twenty Pikemen, ten Swordsmen, ten Archers. Read its points. Click the Dwarven Treasury T1's entrance (10, 20) twice: E13-3 walks onto it; answer the question **yes**.
2. As the battle opens, before any stack acts: whether a tactics phase opens (if one opens, note it and end it without a move); each side's stacks (creature, count, hexes); the obstacles (none, or each); the war machines on the field (each). Then play: at each Red stack's turn before the Archers' first: Defend. At the Archers' first turn note their shots, shoot any guard stack, and note their shots again. Then auto combat to the end.
3. After it note the dialogs and messages in order (the level-ups: the left skill; note the skills taken), E13-3's tile, points and army, and Red's treasury.
4. **E13-10, day 2, Red's turn, first:** E13-3 stands on (10, 20). From E13-1's records pick the Imp Cache of level 1 (twenty Imps a stack, 1 000 gold and 2 Mercury) among I2 (14, 21), I3 (7, 21) and I4 (12, 24), the first in that order; if none is level 1, I2. Read Red's treasury, walk onto its entrance (twice), answer **yes**, auto combat; read the treasury after the reward.
5. Then the Naga Bank N1 (8, 24): read the treasury, walk onto its entrance, **yes**, auto combat; read the treasury after.

**Record**

- E13-3 before: points: 1560
- the battle as it opens: a tactics phase?; E13-3's stacks (creature, count, hexes); the guards' (creature, count, hexes): tactics phase: no; stacks: attacker slot 0: 10 Archangels 57-58, slot 2: 20 Pikemen 61, slot 4: 10 Swordsmen 90, slot 6: 10 Archers 93 | defender slot 0: 15 Dwarves 15, slot 1: 15 Dwarves 185, slot 2: 15 Battle Dwarves 172, slot 3: 15 Dwarves 2, slot 4: 15 Dwarves 100
- the obstacles; the war machines on the field: obstacles: none; war machines: none; field: terrain grass, special field none, ship flag 0, blocked hexes none
- the Archers' shots before their first shot; after it: at the opening: attacker 10 Archers 12; the turn: shots 12 then 12; shoots 10 Battle Dwarves at hex 106 (cursor frame 3); the defender before slot 0: 15 Dwarves, slot 1: 15 Dwarves, slot 2: 10 Battle Dwarves, slot 3: 15 Dwarves, slot 4: 15 Dwarves, after slot 0: 15 Dwarves, slot 1: 15 Dwarves, slot 2: 8 Battle Dwarves, slot 3: 15 Dwarves, slot 4: 15 Dwarves
- after the battle: dialogs and messages in order; E13-3's tile; points; army; Red's treasury: the path: 10,20 10,21; in order: question ADVEVENT:32 (yes); battle (the hero on 10,20); level-up (E13-3, level 2); message ADVEVENT:34; the level-ups: level 2: Leadership or Wisdom, took left (Leadership); the messages' pictures: ADVEVENT:34: 4000 gold, 3 crystal; tile: 10,20,0; points: 1360; army: 0: 10 Archangels, 2: 11 Pikemen, 4: 9 Swordsmen, 6: 10 Archers; skills: Tactics expert, Leadership basic; treasury before: 36 wood, 21 mercury, 36 ore, 21 sulfur, 21 crystal, 21 gems, 30500 gold; after: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 34500 gold
- E13-10: the Imp Cache taken (its record's level); E13-3's points at the start of day 2: taken: 7,21 (the first that meets {'gold': 1000}); points: 1560; day's points: 1560; tile: 10,20,0
- the Imp Cache: Red's treasury before; the reward's message; the treasury after: before: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 71000 gold; the path: 7,21 8,21 9,21; in order: question ADVEVENT:32 (yes); battle (the hero on 7,21); message ADVEVENT:34; pictures: ADVEVENT:34: 1000 gold, 2 mercury; after: 36 wood, 23 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 72000 gold
- the Naga Bank N1: Red's treasury before; the reward's message; the treasury after: before: 36 wood, 23 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 72000 gold; the path: 7,22 7,23 7,24 8,24; in order: question ADVEVENT:32 (yes); battle (the hero on 8,24); level-up (E13-3, level 3); level-up (E13-3, level 4); message ADVEVENT:34; pictures: ADVEVENT:34: 8000 gold, 16 sulfur; after: 36 wood, 23 mercury, 36 ore, 37 sulfur, 24 crystal, 21 gems, 80000 gold; E13-3 then: 8,24,0; 819
- anything odd: E13-3 at the start: skills; army slots; formation; worn: Tactics expert; 0: 10 Archangels, 2: 20 Pikemen, 4: 10 Swordsmen, 6: 10 Archers; 2 (loose, tactics formation on); Ballista (war machine 1), Ammo Cart (war machine 2), Catapult (war machine 4); (agent, T120) the Battle Dwarves (slot 2, speed 5) stood on hex 106 and had lost creatures before the Archers' turn (a Red stack's retaliation; runs 2 and 3 alike), the shot went to them as the nearest; the Archers' shots 12 before and after in runs 2 and 3 (the first run read them during the shot's animation, fixed: E08-log T120)

## E13-6 — A full army near a level wins a Griffin Conservatory: the two-hex guards' hexes (E13-4), the dialogs' order (E13-6); the looted Conservatory visited again (E13-8) (D-67, D-69, GAM-DEC-007, GAM-DEC-004, ADV-BNK-005, BAT-SET-001)

- Red hero **E13-6** (Loynis, specialty spell Prayer, 900 experience): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel, 10 Titan, 10 Archangel, 10 Titan, 10 Archangel, 10 Titan, 10 Archangel

1. Day 1, Red's turn, after E13-3: E13-6 stands at (12, 36) with seven stacks (Archangels and Titans, ten each, slots 0 to 6: the army full) and 900 experience. Read its points, experience and level. Click the Griffin Conservatory G2's entrance (12, 34) twice: E13-6 walks onto it; answer **yes**.
2. **E13-4.** As the battle opens: each side's stacks (creature, count) and **every hex each stack covers** (two for a Griffin). Then auto combat.
3. **E13-6.** After the battle note every dialog and message **in order** (each level-up: the left skill; the reward's message; the army dialog: both bars, then OK without a move). Then E13-6's tile, points, army, experience and level.
4. **E13-8 (the first part).** Press Space on (12, 34) (if nothing comes, step to (12, 35) and back). Note every question and message.

**Record**

- E13-6 before: points; experience; level: points: 2000; experience: 900; level: 1
- the battle as it opens: E13-6's stacks (creature, count, hexes): attacker slot 0: 10 Archangels 57-58, slot 1: 10 Titans 61, slot 2: 10 Archangels 90-91, slot 3: 10 Titans 93, slot 4: 10 Archangels 96-97, slot 5: 10 Titans 125, slot 6: 10 Archangels 129-130; tactics phase: no; obstacles: none

| guard slot | creature | count | hexes covered |
|---|---|---|---|
| 0 | Griffins | 30 | 14-15 |
| 1 | Griffins | 30 | 184-185 |
| 2 | Griffins | 30 | 171-172 |
| 3 | Griffins | 30 | 1-2 |
| 4 | Griffins | 30 | 99-100 |

| # | dialog or message (each level-up; the reward's message; the army dialog's two bars) |
|---|---|
| 1 | question ADVEVENT:32 (yes) |
| 2 | battle (the hero on 12,34) |
| 3 | level-up (E13-6, level 2) |
| 4 | level-up (E13-6, level 3) |
| 5 | level-up (E13-6, level 4) |
| 6 | level-up (E13-6, level 5) |
| 7 | message ADVEVENT:34 |
| 8 | army dialog (the title GENRLTXT:36 (Angels); upper 0: 3; lower 0: 10 Archangels; lower 1: 10 Titans; lower 2: 10 Archangels; lower 3: 10 Titans; lower 4: 10 Archangels; lower 5: 10 Titans; lower 6: 10 Archangels) |

- after OK: E13-6's tile; points; army; experience; level: tile: 12,34,0; points: 1800; army: 0: 10 Archangels, 1: 10 Titans, 2: 10 Archangels, 3: 10 Titans, 4: 10 Archangels, 5: 10 Titans, 6: 10 Archangels; experience: 4650; level: 5; the level-ups: level 2: Tactics or First Aid, took left (Tactics); level 3: Tactics or Wisdom, took left (Tactics); level 4: Tactics or Water Magic, took left (Tactics); level 5: Scholar or Water Magic, took left (Scholar); the messages' pictures: ADVEVENT:34: Angels; at the question: ADVEVENT:32 (yes) on 12,34 with 1800
- E13-8: Space on G2: questions and messages: the key: nothing; in order: message ADVEVENT:33; pictures: none; G2: visited red, looted yes (word 0xfe01803f)
- anything odd: (agent, T120) the army dialog's upper creature is read from its title (GENRLTXT 36 names the first stack's creature), its count from the bar; the first run's bar frames were no portraits

## E13-7 — Every body position full, Dragon Utopia won: the backpack's order (E13-7); two Crypts won; day 2: the two looted Crypts and the looted Shipwreck searched, then a battle (E13-8) (D-68, ADV-BNK-003, ADV-BNK-005, HER-ADV-008, GAM-BAT-014)

- Red hero **E13-7** (Coronius, specialty spell Slayer): attack 40, defense 40, power 1, knowledge 1; no skills; Boots of Polarity (feet), Helm of the Alabaster Unicorn (head), Shield of the Dwarven Lords (left hand), Still Eye of the Dragon (left ring), Clover of Fortune (misc 1), Cards of Prophecy (misc 2), Ladybird of Luck (misc 3), Badge of Courage (misc 4), Crest of Valor (misc 5), Necklace of Swiftness (neck), Centaurs Axe (right hand), Quiet Eye of the Dragon (right ring), Recanter's Cloak (shoulders), Breastplate of Petrified Wood (torso); no spells; army 20 Archangel

1. **E13-7.** Day 1, Red's turn, after E13-6: E13-7 stands at (16, 47) with twenty Archangels and an artifact at every body position (the backpack empty). Read its worn artifacts and its backpack (a starting bonus's artifact would be there). From E13-1's records take U1 (16, 44) unless its artifacts are four relics and U2's are not; in that case U2 (16, 50). Read Red's treasury. Walk onto its entrance (twice) and answer **yes**; auto combat.
2. After it note every dialog and message in order (the level-ups: the left skill), then E13-7's worn artifacts, **its backpack in order** (each artifact's name) and Red's treasury.
3. Walk onto the Crypt C2's entrance (20, 44), answer **yes**, auto combat; then onto C3's (24, 44), **yes**, auto combat. Note each reward's message and the backpack after. If the day's points run out, do the rest first on day 2.
4. **E13-8, day 2, Red's turn, after E13-10 and E13-5's Derelict Ship:** E13-7 stands on C3 (24, 44). Read its status bits 9 to 11 (+0x105) and morale byte (+0x11A). Press Space (or step to (24, 45) and back), answer **yes**; note the messages, the bits and the byte. Walk onto C2's entrance (20, 44), **yes**; the same notes.
5. Walk to (26, 45) on the coast, then click the Shipwreck W1's entrance (27, 46) (looted by E13-5 on day 1); answer **yes**; the same notes and E13-7's tile.
6. Click the Peasant at (24, 48): E13-7 attacks it; auto combat. Note the bits and the byte after the battle.

**Record**

- E13-7 before: points; worn artifacts; backpack; the Utopia taken (U1 or U2); Red's treasury: points: 2000; worn: Helm of the Alabaster Unicorn (head; treasure), Recanter's Cloak (shoulders; major), Necklace of Swiftness (neck; treasure), Centaurs Axe (right hand; treasure), Shield of the Dwarven Lords (left hand; treasure), Breastplate of Petrified Wood (torso; treasure), Quiet Eye of the Dragon (right ring; treasure), Still Eye of the Dragon (left ring; treasure), Boots of Polarity (feet; relic), Clover of Fortune (misc 1; treasure), Cards of Prophecy (misc 2; treasure), Ladybird of Luck (misc 3; treasure), Badge of Courage (misc 4; treasure), Crest of Valor (misc 5; treasure); backpack: none; treasury: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 34500 gold; taken: 16,44 (the first that meets {'relics_below': 4})
- the battle as it opens: the guards (creature, count, hexes): defender slot 0: 8 Green Dragons 14-15, slot 1: 6 Red Dragons 184-185, slot 2: 3 Gold Dragons 171-172, slot 3: 2 Black Dragons 1-2; E13-7: attacker slot 0: 20 Archangels 57-58; obstacles: none
- after it: dialogs and messages in order: the path: 16,44 16,45 16,46; in order: question ADVEVENT:47 (yes); battle (the hero on 16,44); level-up (E13-7, level 2); level-up (E13-7, level 3); level-up (E13-7, level 4); message ADVEVENT:34; the level-ups: level 2: Earth Magic or Fire Magic, took left (Earth Magic); level 3: Earth Magic or Wisdom, took left (Earth Magic); level 4: Earth Magic or Luck, took left (Earth Magic); the messages' pictures: ADVEVENT:34: 30000 (resource.def 6), Crown of the Supreme Magi, Sword of Hellfire, Sentinel's Shield, Thunder Helmet
- E13-7's worn artifacts after; its backpack in order (names; class each); Red's treasury: worn: Helm of the Alabaster Unicorn (head; treasure), Recanter's Cloak (shoulders; major), Necklace of Swiftness (neck; treasure), Centaurs Axe (right hand; treasure), Shield of the Dwarven Lords (left hand; treasure), Breastplate of Petrified Wood (torso; treasure), Quiet Eye of the Dragon (right ring; treasure), Still Eye of the Dragon (left ring; treasure), Boots of Polarity (feet; relic), Clover of Fortune (misc 1; treasure), Cards of Prophecy (misc 2; treasure), Ladybird of Luck (misc 3; treasure), Badge of Courage (misc 4; treasure), Crest of Valor (misc 5; treasure); backpack: Thunder Helmet (relic), Sentinel's Shield (relic), Sword of Hellfire (major), Crown of the Supreme Magi (minor); treasury before: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 34500 gold; after: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 64500 gold; points: 1700
- C2 and C3: the reward's messages; the backpack after; E13-7's points after C3: C2: question ADVEVENT:119 (yes); battle (the hero on 20,44); level-up (E13-7, level 5); message ADVEVENT:34; pictures: ADVEVENT:34: 2500 (resource.def 6), Quiet Eye of the Dragon; treasury: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 67000 gold; backpack: Thunder Helmet (relic), Sentinel's Shield (relic), Sword of Hellfire (major), Crown of the Supreme Magi (minor), Quiet Eye of the Dragon (treasure); points: 1218; C3: question ADVEVENT:119 (yes); battle (the hero on 24,44); message ADVEVENT:34; pictures: ADVEVENT:34: 2000 gold; treasury: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 69000 gold; backpack: Thunder Helmet (relic), Sentinel's Shield (relic), Sword of Hellfire (major), Crown of the Supreme Magi (minor), Quiet Eye of the Dragon (treasure); points: 736
- E13-8 day 2, before: +0x105 bits 9, 10, 11; +0x11A: bit 9: clear; bit 10: clear; bit 11: clear; +0x11A: 0; tile: 24,44,0; points: 2000
- C3 again: questions and messages; bits; +0x11A: the key: nothing; in order: question ADVEVENT:119 (yes); message ADVEVENT:120; bit 9: clear; bit 10: set; bit 11: clear; +0x11A: -1; C3: visited red, looted yes (word 0xfe02003f)
- C2 again: questions and messages; bits; +0x11A: the path: 20,44 21,45 22,45 23,45; in order: question ADVEVENT:119 (yes); message ADVEVENT:120; bit 9: clear; bit 10: set; bit 11: clear; +0x11A: -1; points: 1518
- W1 from (26, 45): questions and messages; E13-7's tile; bits; +0x11A: the walk to (26, 45): 21,45 22,45 23,45 24,45 25,45 26,45; then: 26,45,0; 877; W1: ADVEVENT:122 (yes) on 26,45 with 736; in order: question ADVEVENT:122 (yes); message ADVEVENT:123; tile: 26,45,0; points: 736; bit 9: set; bit 10: set; bit 11: clear; +0x11A: -2
- the Peasant: the battle's result; bits; +0x11A after: the path: 25,46 25,47 24,48; winner: attacker; in order: battle (the hero on 25,47); bit 9: clear; bit 10: clear; bit 11: clear; +0x11A: 0; tile: 25,47,0
- anything odd: 

## E13-5 — The Shipwreck attacked from the coast: the battlefield, where the hero stands (E13-5, E13-11); a hero boards a boat and, on day 2, attacks a Derelict Ship from it (E13-5) (ADV-BNK-004, GAM-BAT-001, ADV-MOV-009, ADV-SEA-002)

- Red hero **E13-5** (Cuthbert, specialty spell Weakness): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel
- Red hero **E13-5b** (Adelaide, specialty spell Frost Ring): attack 10, defense 10, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Archangel

1. Day 1, Red's turn, after E13-7: E13-5 stands at (26, 46) on the coast with ten Archangels; the Shipwreck W1's entrance (27, 46) is the water tile east of it. Read its points. Click W1's entrance (27, 46) twice. Note E13-5's tile and points at the question, every question and message; answer **yes**.
2. As the battle opens: the battlefield (the background: a deck, a shore, grass, …), the obstacles, each side's stacks. Auto combat. After it: the dialogs and messages in order, E13-5's tile and points, W1's word.
3. E13-5b stands at (26, 51) with ten Archangels beside an empty boat on (27, 51). Click the boat twice: E13-5b boards it. Note its tile, points and whether it is aboard. End Red's turn after E13-5b.
4. **Day 2, Red's turn, after E13-10:** E13-5b in its boat on (27, 51): read its points. Click the Derelict Ship D1's entrance (30, 54) twice: it sails onto it. Note every question and message; answer **yes**. As the battle opens: the battlefield and the obstacles. Auto combat; after it E13-5b's tile and the dialogs.

**Record**

- E13-5 before: points: 2000
- W1: at the question E13-5's tile; points; questions and messages: at the question: ADVEVENT:122 (yes) on 26,46 with 1900; the path: 27,46; in order: question ADVEVENT:122 (yes); battle (the hero on 26,46); message ADVEVENT:34
- the W1 battle as it opens: the battlefield; obstacles; the stacks: field: terrain water, special field none, ship flag 1, blocked hexes none; obstacles: none; war machines: none; stacks: attacker slot 0: 10 Archangels 57-58 | defender slot 0: 2 Wights 15, slot 1: 2 Wights 185, slot 2: 2 Wights 172, slot 3: 2 Wights 2, slot 4: 2 Wights 100
- after it: dialogs and messages; E13-5's tile; points; W1's word: in order: question ADVEVENT:122 (yes); battle (the hero on 26,46); message ADVEVENT:34; pictures: ADVEVENT:34: 2000 gold; tile: 26,46,0; points: 1900; W1: visited red, looted yes (word 0xfe02203f); treasury before: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 69000 gold; after: 36 wood, 21 mercury, 36 ore, 21 sulfur, 24 crystal, 21 gems, 71000 gold
- E13-5b: the boarding: its tile; points; aboard?: points before: 2000; tile: 27,51,0; points: 0; aboard: yes; messages: none
- day 2: E13-5b's points; D1: questions and messages: points: 1500; aboard: yes; the path: 27,52 28,53 29,54 30,54; at the question: ADVEVENT:41 (yes) on 30,54 with 1018; in order: question ADVEVENT:41 (yes); battle (the hero on 30,54); message ADVEVENT:34
- the D1 battle as it opens: the battlefield; obstacles: field: terrain water, special field none, ship flag 1, blocked hexes none; obstacles: none; stacks: attacker slot 0: 10 Archangels 57-58 | defender slot 0: 6 Water Elementals 14-15, slot 1: 6 Water Elementals 184-185, slot 2: 6 Water Elementals 171-172, slot 3: 6 Water Elementals 1-2, slot 4: 6 Water Elementals 99-100
- after it: E13-5b's tile; dialogs and messages: tile: 30,54,0; aboard: yes; pictures: ADVEVENT:34: 3000 (resource.def 6), Crest of Valor; treasury before: 36 wood, 23 mercury, 36 ore, 37 sulfur, 24 crystal, 21 gems, 80000 gold; after: 36 wood, 23 mercury, 36 ore, 37 sulfur, 24 crystal, 21 gems, 83000 gold; D1: visited red, looted yes (word 0xfe02603f)
- anything odd: 

## E13-12 — A flight over a Shipwreck's water entrance, from land and from water; a water walker beside one: no path across, the visit from the coast (ADV-MOV-009, ADV-SEA-002, ADV-STUB-12, D-46)

- Blue hero **E13-12** (Alagar, specialty spell Ice Bolt): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Blue hero **E13-12b** (Astral, specialty spell Hypnotize): attack 1, defense 1, power 1, knowledge 1; no skills; Angel Wings (shoulders); no spells; army 10 Pikeman
- Blue hero **E13-12c** (Daremyth, specialty spell Fortune): attack 1, defense 1, power 1, knowledge 1; no skills; Boots of Levitation (feet); no spells; army 10 Pikeman

1. Day 1, Blue's turn, after E13-1's Blue reads. **P1:** E13-12 (Angel Wings, ten Pikemen) stands at (50, 3) in a rock pocket; south of it the Shipwreck W2's entrance (50, 4) on the only water tile, then grass (50, 5) and (50, 6). Read its points. Click (50, 5) once: note the path's marks (or none). If a path shows, click again; note every question and message on the way, E13-12's tile and points and W2's word.
2. **P2:** E13-12b (Angel Wings, ten Pikemen) stands at (56, 3); south of it three water tiles (56, 4) to (56, 6), the Shipwreck W3's entrance the middle one (56, 5), then grass (56, 7) and (56, 8). Click (56, 7) once: note the marks; if a path shows, click again; the same notes (W3's word).
3. **P3:** E13-12c (Boots of Levitation, ten Pikemen) stands at (62, 3); south of it the Shipwreck W4's entrance (62, 4) on the only water tile, then grass (62, 5) and (62, 6). Click (62, 5) once: note the marks (or none). If a path shows, click again and note where it stops. Then click W4's entrance (62, 4) twice: note E13-12c's tile and points, every question and message; answer **no**. Note its tile, points and W4's word. End Blue's turn.

**Record**

- P1: E13-12's points; the marks to (50, 5) (tiles, or none): points: 1560; marks: 50,4 50,5; colours: green
- P1: the walk: questions and messages; E13-12's tile; points; W2's word: clicked again: yes; in order: none; tile: 50,5,0; points: 1360; W2: visited none, looted no (word 0xfc00201f)
- P2: E13-12b's points; the marks to (56, 7): points: 1560; marks: 56,4 56,5 56,6 56,7; colours: green
- P2: the walk: questions and messages; E13-12b's tile; points; W3's word: clicked again: yes; in order: none; tile: 56,7,0; points: 1160; W3: visited none, looted no (word 0xfc00601f)
- P3: E13-12c's points; the marks to (62, 5) (or none); if walked, where it stopped: points: 1560; marks: none; clicked again: no; tile: 62,3,0; points after: 1560; in order: none
- P3: W4's visit: E13-12c's tile and points at the question; questions and messages: at the question: ADVEVENT:122 (no) on 62,3 with 1460; the path: 62,4; in order: question ADVEVENT:122 (no)
- after no: E13-12c's tile; points; W4's word: tile: 62,3,0; points: 1460; W4: visited none, looted no (word 0xfc00401f)
- anything odd: 
