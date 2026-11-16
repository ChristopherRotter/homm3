# E09 batch TWN-4a — results

> Played by the capture harness on 2026-11-16 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-4a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-4a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `da3a4a8f`.** The map's description in the scenario list starts with "Stamp da3a4a8f.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-4a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a player's two tavern offers (player record +0x28, +0x2C: hero types), each offer's hero record (game +0x21620 + 0x492 x type: its army +0x91, points +0x4D of the day's +0x49, spell points +0x18, class +0x30), the hero pool (game +0x4DF18). The capture harness reads them, writes the date for the weeks and logs the taverns' draws (`clients/capture/batches/E09-batch-4a.toml`, E09 T34). Played by hand the tavern dialog shows the two offers (portrait, name, level, class).
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile a hero visits. A step: click the target once (the path's marks show), then again (the hero goes).
6. Order: the offers on Red's day 1; then each week: end Red's turn and Blue's (by hand: play the days through), the offers on Red's new turn, eight weeks in all; then, in the eighth week, the hires and the Recruit button (TWN-4-6, TWN-4-2).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-4-1 — The two players' tavern offers over eight weeks (rows TWN-4-1 and TWN-4-8) (TWN-TAV-001, TWN-TAV-002)

- Red hero **TWN-4-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **TWN-4-1b** (Gurnisson, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Goblin

1. Red's turn, day 1, before any hero moves: read Red's and Blue's two offers (each: the hero, its class, army, points, spell points, artifacts) and the draws the game's start made.
2. Then each week (eight in all): write the date to the week's last day, end Red's turn and Blue's; on Red's new turn read the four offers again and the draws of the week's start.

**Record**

| week | Red's slot 0 | Red's slot 1 | Blue's slot 0 | Blue's slot 1 | the taverns' draws (class, type; with their ranges) |
|---|---|---|---|---|---|
| M1 W1 (the start) | Sanya (Cleric, type 13): 16 Pikemen, 5 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Krellion (Barbarian, type 101): 19 Goblins, 7 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Crag Hack (Barbarian, type 102): 24 Goblins, 6 Orcs; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Adelaide (Cleric, type 11): 15 Pikemen, 6 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | tavern_class 9 of 1-12, tavern_type 5 of 1-7, tavern_class 66 of 1-94, tavern_type 5 of 1-7, tavern_class 2 of 1-12, tavern_type 5 of 1-6, tavern_class 8 of 1-94, tavern_type 3 of 1-6 |
| M1 W2 | Adela (Cleric, type 9): 19 Pikemen, 7 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Coronius (Druid, type 24): 24 Centaurs, 5 Dwarves; points 1500 of 1500; spell points 20; artifacts worn 0, backpack 0 | Gretchin (Barbarian, type 100): 16 Goblins, 24 Goblins; points 1700 of 1700; spell points 10; artifacts worn 0, backpack 0 | Piquedram (Alchemist, type 32): 4 Stone Gargoyles, 3 Stone Gargoyles, 3 Stone Gargoyles; points 1760 of 1760; spell points 20; artifacts worn 0, backpack 0 | tavern_class 9 of 1-12, tavern_type 1 of 1-7, tavern_class 18 of 1-94, tavern_type 1 of 1-8, tavern_class 1 of 1-12, tavern_type 4 of 1-7, tavern_class 26 of 1-94, tavern_type 1 of 1-8 |
| M1 W3 | Tyris (Knight, type 7): 17 Pikemen, 5 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | Thant (Necromancer, type 76): 22 Skeletons, 5 Walking Dead; points 1500 of 1500; spell points 20; artifacts worn 0, backpack 0 | Gundula (Battle Mage, type 109): 23 Goblins, 7 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Straker (Death Knight, type 64): 6 Walking Dead, 6 Walking Dead; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | tavern_class 2 of 1-12, tavern_type 2 of 1-3, tavern_class 48 of 1-94, tavern_type 5 of 1-8, tavern_class 12 of 1-12, tavern_type 6 of 1-8, tavern_class 49 of 1-94, tavern_type 1 of 1-8 |
| M1 W4 | Cuthbert (Cleric, type 10): 19 Pikemen, 6 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Alagar (Druid, type 30): 19 Centaurs, 4 Dwarves; points 1500 of 1500; spell points 20; artifacts worn 0, backpack 0 | Jabarkas (Barbarian, type 98): 19 Goblins, 4 Orcs; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Kalt (Planeswalker, type 135): 23 Pixies, 4 Water Elementals; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | tavern_class 7 of 1-12, tavern_type 2 of 1-7, tavern_class 18 of 1-94, tavern_type 7 of 1-8, tavern_class 3 of 1-12, tavern_type 2 of 1-7, tavern_class 88 of 1-94, tavern_type 8 of 1-8 |
| M2 W1 | Sir Mullich (Knight, type 144): 14 Pikemen, 7 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | Rosic (Witch, type 121): 11 Gnolls, 4 Lizardmen; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Shiva (Barbarian, type 99): 25 Goblins, 6 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Galthran (Death Knight, type 71): 24 Skeletons, 23 Skeletons; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | tavern_class 6 of 1-12, tavern_type 3 of 1-3, tavern_class 84 of 1-94, tavern_type 2 of 1-8, tavern_class 4 of 1-12, tavern_type 3 of 1-7, tavern_class 47 of 1-94, tavern_type 8 of 1-8 |
| M2 W2 | Tyris (Knight, type 7): 10 Pikemen, 7 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | Sanya (Cleric, type 13): 11 Pikemen, 6 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Gird (Battle Mage, type 104): 15 Goblins, 7 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Damacon (Overlord, type 84): 40 Troglodytes, 6 Harpies; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | tavern_class 4 of 1-12, tavern_type 2 of 1-3, tavern_class 4 of 1-94, tavern_type 5 of 1-7, tavern_class 10 of 1-12, tavern_type 1 of 1-8, tavern_class 62 of 1-94, tavern_type 5 of 1-8 |
| M2 W3 | Adela (Cleric, type 9): 14 Pikemen, 5 Archers; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0 | Inteus (Elementalist, type 140): 25 Pixies, 5 Air Elementals; points 1760 of 1760; spell points 30; artifacts worn 0, backpack 0 | Krellion (Barbarian, type 101): 25 Goblins, 6 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Sorsha (Knight, type 5): 13 Pikemen, 3 Griffins; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | tavern_class 11 of 1-12, tavern_type 1 of 1-7, tavern_class 93 of 1-94, tavern_type 5 of 1-8, tavern_class 5 of 1-12, tavern_type 5 of 1-7, tavern_class 3 of 1-94, tavern_type 1 of 1-3 |
| M2 W4 | Tyris (Knight, type 7): 16 Pikemen, 5 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | Xarfax (Heretic, type 63): 18 Imps, 4 Gogs; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | Yog (Barbarian, type 96): 21 Goblins, 5 Wolf Riders; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0 | Sir Mullich (Knight, type 144): 13 Pikemen, 5 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0 | tavern_class 5 of 1-12, tavern_type 2 of 1-3, tavern_class 39 of 1-94, tavern_type 8 of 1-8, tavern_class 4 of 1-12, tavern_type 1 of 1-7, tavern_class 1 of 1-94, tavern_type 2 of 1-2 |

- anything odd: the game's version; the pool's states: SoD; blue 1, free 133, red 6, reserved 16; M1 W2: the date: month 1, week 2, day 1; M1 W3: the date: month 1, week 3, day 1; M1 W4: the date: month 1, week 4, day 1; M2 W1: the date: month 2, week 1, day 1; M2 W2: the date: month 2, week 2, day 1; M2 W3: the date: month 2, week 3, day 1; M2 W4: the date: month 2, week 4, day 1

## TWN-4-6 — The Recruit button: a visitor in the town; Red's heroes on the map with a garrison hero (5, 7, 8) (TWN-TAV-003)

- Red hero **TWN-4-6** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-6a** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-6b** (Sylvia, specialty skill Navigation): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-6c** (Christian, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. In the eighth week, Red's turn. TWN-4-6 stands south of R1 at (7, 11): step north onto the entrance (R1's screen opens; Red has 6 heroes on the map). Click the Tavern: note Recruit (enabled or greyed) and the status line with the mouse over it. Close the tavern.
2. Click TWN-4-6's portrait (the lower bar), then the upper one (the town's flag): it moves into the garrison (Red: 5 on the map). Click the Tavern again: note Recruit. Hire slot 0 (TWN-4-2's first hire). Click the Tavern again: note Recruit and the status line over it. Close the tavern and the town screen.
3. After TWN-4-2's hire in R2 (Red: 7 on the map, TWN-4-6 in R1's garrison): open R3's screen from the town list (no visitor), click the Tavern: note Recruit; hire slot 0 (Red: 8 on the map).
4. Open R4's screen from the town list (no visitor), click the Tavern: note Recruit and the status line over it. Close.

**Record**

- R1 with TWN-4-6 its visitor (Red 6 on the map): Recruit; the status line over it: disabled (state 0xe); Red on the map: 6: TWN-4-1, TWN-4-6a, TWN-4-6, TWN-4-6b, TWN-4-6c, TWN-4-2; the status line: TVRNINFO:2
- TWN-4-6 into R1's garrison: R1's garrison hero and visitor; Red's heroes on the map: none; garrison hero: TWN-4-6; visitor: none; Red on the map: 5: TWN-4-1, TWN-4-6a, TWN-4-6b, TWN-4-6c, TWN-4-2
- R1 without a visitor (Red 5 on the map, a garrison hero): Recruit: enabled (state 0x6)
- R1 after the hire (the hired hero its visitor; Red 6 on the map): Recruit; the status line over it: disabled (state 0xe); Red on the map: 6: TWN-4-1, TWN-4-6a, TWN-4-6b, TWN-4-6c, TWN-4-2, Tyris; the status line: TVRNINFO:2
- R3 from the list, no visitor (Red 7 on the map, a garrison hero in R1): Recruit; Red's heroes on the map after a hire there: enabled (state 0x6); Red on the map before: 7: TWN-4-1, TWN-4-6a, TWN-4-6b, TWN-4-6c, TWN-4-2, Tyris, Xarfax; after the hire: 8: TWN-4-1, TWN-4-6a, TWN-4-6b, TWN-4-6c, TWN-4-2, Tyris, Xarfax, Rosic
- R4 from the list, no visitor (Red 8 on the map, a garrison hero in R1): Recruit; the status line over it; Red's heroes on the map: disabled (state 0xe); Red on the map: 8: TWN-4-1, TWN-4-6a, TWN-4-6b, TWN-4-6c, TWN-4-2, Tyris, Xarfax, Rosic; R1's garrison hero: TWN-4-6; the status line: TVRNINFO:1
- anything odd: R1's visit: the screens; how the town screen closed: town; click

## TWN-4-2 — A hire from each slot: the hired hero's tile and spells, the slot's replacement (TWN-TAV-003, TWN-TAV-004)

- Red hero **TWN-4-2** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. In the eighth week, Red's turn, in R1 (TWN-4-6's visit, after it moved into the garrison): click the Tavern, select slot 0 and note the offer; click Recruit. Note the hired hero (on R1's entrance? its spells against R1's guild's), Red's two slots now (the replacement in slot 0), Red's gold, and the draws.
2. Open R2's screen from the town list (no visitor): click the Tavern, select slot 1 and note the offer; click Recruit. Note the same.
3. In R3 (TWN-4-6's third step): the hire of slot 0 there; note the same.

**Record**

- R1, slot 0 selected: the dialog; the offer: offers shown 0, 1; selected 0 (framed 0); Recruit enabled (state 0x6); Tyris (Knight, type 7): 16 Pikemen, 5 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0
- R1, slot 0 hired: the hired hero (R1's visitor: offer, tile, spells); R1's guild spells; Red's two slots; Red's gold: Tyris (Knight, type 7): 16 Pikemen, 5 Archers; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0; on ; 7,10,0; its spells: none; R1's guild: Dispel, Bless, Cure, Magic Arrow, Haste; Red's slot 0: Rosic (Witch, type 121): 1 Gnoll; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0; slot 1: Xarfax (Heretic, type 63): 18 Imps, 4 Gogs; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0; Red's gold: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 42400 gold
- R1's step: the taverns' draws: tavern_class 83 of 1-95, tavern_type 2 of 1-8
- R2, slot 1 selected: the dialog; the offer: offers shown 0, 1; selected 1 (framed 1); Recruit enabled (state 0x6); Xarfax (Heretic, type 63): 18 Imps, 4 Gogs; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0
- R2, slot 1 hired: the hired hero (R2's visitor: offer, tile, spells); R2's guild spells; Red's two slots; Red's gold: Xarfax (Heretic, type 63): 18 Imps, 4 Gogs; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0; on ; 15,10,0; its spells: Summon Boat, Magic Arrow, Fireball, Dispel, Bless, Stone Skin; R2's guild: Bless, Magic Arrow, Dispel, Summon Boat, Stone Skin; Red's slot 0: Rosic (Witch, type 121): 1 Gnoll; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0; slot 1: Galthran (Death Knight, type 71): 1 Skeleton; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0; Red's gold: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 39900 gold
- R2's step: the taverns' draws: tavern_class 49 of 1-94, tavern_type 8 of 1-8
- R3, slot 0 hired: the hired hero (R3's visitor: offer, tile); Red's two slots; Red's gold; the draws: Rosic (Witch, type 121): 1 Gnoll; points 1560 of 1560; spell points 20; artifacts worn 0, backpack 0; on ; 23,10,0; Red's slot 0: Brissa (Elementalist, type 137): 1 Pixie; points 1760 of 1760; spell points 30; artifacts worn 0, backpack 0; slot 1: Galthran (Death Knight, type 71): 1 Skeleton; points 1630 of 1630; spell points 10; artifacts worn 0, backpack 0; Red's gold: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 37400 gold; the draws: tavern_class 92 of 1-95, tavern_type 2 of 1-8
- anything odd: R2, no visitor (Red 6 on the map): Recruit: enabled (state 0x6)
