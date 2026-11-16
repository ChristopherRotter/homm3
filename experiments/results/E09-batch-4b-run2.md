# E09 batch TWN-4b — results

> Played by the capture harness on 2026-11-16 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-4b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-4b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `4d0e143c`.** The map's description in the scenario list starts with "Stamp 4d0e143c.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-4b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a player's two tavern offers (player record +0x28, +0x2C: hero types), each hero's record (game +0x21620 + 0x492 x type: owner +0x22, army +0x91, points +0x4D of the day's +0x49, spell points +0x18, the returning mark bit 17 of +0x105, the artifacts), the hero pool (game +0x4DF18: -1 free, 0x40 reserved, a colour). The capture harness reads them, writes the date and some heroes' spell points, narrows the pool and logs the draws (`clients/capture/batches/E09-batch-4b.toml`, E09 T34). Played by hand the tavern dialog shows the offers.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile a hero visits. A step: click the target once (the path's marks show), then again (the hero goes).
6. Order: Red's day 6 (TWN-4-3's retreat, TWN-4-4's surrender and loss); Blue's day 6; Red's day 7; Blue's day 7 (TWN-4-3's attack on TWN-4-3f); Red's day 8 (the new week; TWN-4-5).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-4-3 — TD-18: a hero that flees on day 6 in Red's turn and one that flees on day 7 in Blue's; Red's offers on day 8 (TWN-TAV-002, TWN-TAV-005, TWN-TAV-006)

- Red hero **TWN-4-3** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Swordsman, 2 Monk
- Opponent: 5 Gnoll (wandering monster, never flees)
- Red hero **TWN-4-3f** (Ingham, specialty creature Monk): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Cavalier
- Blue hero **TWN-4-3b** (Yog, specialty creature Cyclops): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Ogre

1. Red's turn, day 1, before any hero moves: the harness writes the date to day 6 of week 1. Read Red's two offers and the pool states of TWN-4-3, -3f, -4, -4b and -5 (Red's).
2. Set TWN-4-3's spell points to 3. TWN-4-3 (4, 20) attacks the Gnolls east of it (5, 20) and retreats at its first stack's turn. Read Red's offers, TWN-4-3's record (owner, army, points, spell points, returning mark), its pool state and the slot's draw.
3. (TWN-4-4's surrender and TWN-4-4b's loss follow, then Red's turn ends.) On Blue's turn of day 6: read TWN-4-3's and TWN-4-4's returning marks. On Red's day 7: read Red's offers and TWN-4-3's and TWN-4-4's points; set TWN-4-3f's spell points to 4; end Red's turn.
4. Blue's day 7: Blue's TWN-4-3b (13, 20) attacks Red's TWN-4-3f (12, 20); TWN-4-3f retreats at its first stack's turn. Read Red's offers, TWN-4-3f's record, the pool states of TWN-4-3, -3f and -4, the slot's draw.
5. End Blue's turn: week 2 starts. On Red's day 8 read Red's offers, the pool states of TWN-4-3, -3f, -4 and -4b, and the week's draws.

**Record**

- before (Red's day 1, written to day 6): Red's offers; the pool states: slot 0: Adela (Cleric), slot 1: Luna (Elementalist); pool: TWN-4-3: red; -3f: red; -4: red; -4b: red; -5: red
- TWN-4-3 retreats (day 6, Red's turn): Red's offers; its record; its pool state; the slot's draw: slot 0: Adela (Cleric), slot 1: TWN-4-3 (Knight); TWN-4-3: TWN-4-3 (Knight, type 5): 1 Pikeman; points 1530 of 1630; spell points 3; artifacts worn 0, backpack 0; returning; owner: none; points before the battle: 1630; pool: reserved; the draw: leave_slot 1 of 0-1
- Blue's turn, day 6: the returning marks of TWN-4-3 and TWN-4-4: TWN-4-3: no; TWN-4-4: no; slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3 (Knight)
- Red's day 7: Red's offers; TWN-4-3's and TWN-4-4's points (of the day's): slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3 (Knight); TWN-4-3: 1560; of: 1560; TWN-4-4: 1630; of: 1630
- TWN-4-3f retreats (day 7, Blue's turn): Red's offers; its record; the pool states; the slot's draw: slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3f (Cleric); TWN-4-3f: TWN-4-3f (Cleric, type 12): 1 Pikeman; points 1760 of 1760; spell points 4; artifacts worn 0, backpack 0; returning; owner: none; pool: TWN-4-3: free; -3f: reserved; -4: reserved; ; the draw: leave_slot 1 of 0-1
- Red's day 8 (week 2): Red's offers; the pool states; the week's draws: slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3f (Cleric); slot 0: TWN-4-4 (Knight, type 7): 14 Pikemen, 7 Archers, 2 Griffins; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0; slot 1: TWN-4-3f (Cleric, type 12): 1 Pikeman; points 1560 of 1560; spell points 5; artifacts worn 0, backpack 0; pool: TWN-4-3: free; -3f: reserved; -4: reserved; -4b: free; ; the draws: tavern_class 3 of 1-6, tavern_type 7 of 1-8, tavern_class 7 of 1-12, tavern_type 2 of 1-8, tavern_class 91 of 1-94, tavern_type 7 of 1-8
- anything odd: the game's version; the date; the pool's states: SoD; month 1, week 1, day 6; blue 2, free 132, red 6, reserved 16; TWN-4-3's battle: the dialogs in order; messages: battle (the hero on 4,20); none; Red's day 7: the date: month 1, week 1, day 7; TWN-4-3f's battle: the dialogs in order; messages; the date: battle (the hero on 13,20); none; month 1, week 1, day 7; Red's day 8: the date: month 1, week 2, day 1

## TWN-4-4 — A hero that surrenders (TWN-4-4) and one that loses (TWN-4-4b) on day 6, beside TWN-4-3's retreat (TWN-TAV-006)

- Red hero **TWN-4-4** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 4 Swordsman, 3 Monk
- Opponent: Blue hero **TWN-4-4x** (Crag Hack, specialty skill Offense): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Orc
- Red hero **TWN-4-4b** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Pikeman

1. Red's day 6, after TWN-4-3's retreat. Set TWN-4-4's spell points to 2. TWN-4-4 (4, 26) attacks Blue's TWN-4-4x east of it (5, 26) and surrenders at its first stack's turn (the price paid). Read Red's offers, TWN-4-4's record, its pool state, Red's gold before and after, and the slot's draw.
2. Set TWN-4-4b's spell points to 1. TWN-4-4b (12, 26) attacks the Black Dragons east of it (13, 26) with auto combat and loses. Read TWN-4-4b's record (owner, army, points of the day's, spell points), its pool state and Red's offers.

**Record**

- TWN-4-4 surrenders (day 6): Red's offers; its record; its pool state; Red's gold before and after; the slot's draw: slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3 (Knight); TWN-4-4: TWN-4-4 (Knight, type 7): 4 Swordsmen, 3 Monks; points 1530 of 1630; spell points 2; artifacts worn 0, backpack 0; returning; owner: none; points before the battle: 1630; pool: reserved; Red before: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 31000 gold; after: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 29800 gold; the draw: leave_slot 1 of 0-1
- TWN-4-4b loses (day 6): its record; its pool state; Red's offers; the battle's winner: TWN-4-4b: TWN-4-4b (Ranger, type 16): 1 Centaur; points 2000 of 2000; spell points 10; artifacts worn 0, backpack 0; owner: none; points before the battle: 1560; pool: free; slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3 (Knight); winner: defender
- anything odd: TWN-4-4's battle: the dialogs in order; messages: battle (the hero on 4,26); none; TWN-4-4b's battle: the dialogs in order; messages: battle (the hero on 12,26); none

## TWN-4-4b — TWN-4-4b's opponent: 30 Black Dragons (the loss of row TWN-4-4) (TWN-TAV-006)

- Red hero **TWN-4-4z** (Clancy, specialty creature Unicorn): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Opponent: 30 Black Dragon (wandering monster, never flees)

1. TWN-4-4's second half: the Black Dragons east of TWN-4-4b, whom it attacks and loses to.

**Record**

- anything odd: 

## TWN-4-5 — T-14: TWN-4-5 dismissed with its artifacts, then drawn again as a replacement (GAM-MAN-003, TWN-TAV-004)

- Red hero **TWN-4-5** (Neela, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; Badge of Courage (misc 1), Clover of Fortune (backpack); no spells; army 1 Pikeman

1. Red's day 8, after TWN-4-3's reads. Read TWN-4-5's artifacts (worn and in the backpack). The harness narrows the pool: every free type reserved but TWN-4-4b's.
2. Open TWN-4-5's hero screen and click Dismiss (the question: yes). Read its owner, pool state, artifacts and army, and the pool's states.
3. Open R1's screen from the town list, click the Tavern, select slot 0, Recruit. Read the hired hero, Red's slot 0 (the replacement: its name, army, artifacts) and the draws.
4. Open R2's screen from the town list, the same hire of slot 0. Read the same.

**Record**

- before: TWN-4-5's artifacts (worn, backpack); the pool narrowed (its states): worn: Badge of Courage (misc 1), Catapult (war machine 4); backpack: Clover of Fortune; pool: blue 2, free 1, red 2, reserved 151
- TWN-4-5 dismissed: the question; its owner, pool state, artifacts, army; the pool's states: GENRLTXT:23; owner: none; pool: free; worn: Badge of Courage (misc 1), Catapult (war machine 4); backpack: Clover of Fortune; army: 1 Gremlin; points: 2000; spell points: 10; pool: blue 2, free 2, red 1, reserved 151
- the first hire (R1, slot 0): the hired hero; Red's slot 0 now (the replacement); the draws: hired: TWN-4-4 (Knight, type 7): 14 Pikemen, 7 Archers, 2 Griffins; points 1560 of 1560; spell points 10; artifacts worn 0, backpack 0; ; slot 0: TWN-4-5 (Alchemist, type 35): 1 Gremlin; points 2000 of 2000; spell points 10; artifacts worn 1, backpack 1; worn: Badge of Courage (misc 1), Catapult (war machine 4); backpack: Clover of Fortune; class: Alchemist (4); ; slot 1: TWN-4-3f (Cleric, type 12): 1 Pikeman; points 1560 of 1560; spell points 5; artifacts worn 0, backpack 0; the draws: tavern_class 11 of 1-12, tavern_type 1 of 1-1
- the second hire (R2, slot 0): the hired hero; Red's slot 0 now (the replacement); the draws: hired: TWN-4-5 (Alchemist, type 35): 1 Gremlin; points 2000 of 2000; spell points 10; artifacts worn 1, backpack 1; ; slot 0: TWN-4-4b (Ranger, type 16): 1 Centaur; points 2000 of 2000; spell points 10; artifacts worn 0, backpack 0; worn: Catapult (war machine 4); backpack: none; class: Ranger (2); ; slot 1: TWN-4-3f (Cleric, type 12): 1 Pikeman; points 1560 of 1560; spell points 5; artifacts worn 0, backpack 0; the draws: tavern_class 3 of 1-6, tavern_type 1 of 1-1; ; the pool: blue 2, red 3, reserved 151
- anything odd: R1, slot 0 selected: the dialog; Red's offers: offers shown 0, 1; selected 0 (framed 0); Recruit enabled (state 0x6); slot 0: TWN-4-4 (Knight), slot 1: TWN-4-3f (Cleric)
