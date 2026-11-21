# E08 batch E15a — results

Map file: `E08-batch-15a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `a7b4e9e8`.** The map's description in the scenario list starts with "Stamp a7b4e9e8.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-15a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: a hero's tile, movement points (hero +0x4D) and day's maximum (+0x49), facing (+0x47), status flags (+0x105), experience, level, primary skills, spell points (+0x18) and army; the player's hero list (record +0x1 the count, +0x8 the list); `hero_owner` (game +0x4DF18) of the prisoners' types; the messages in their order. The capture harness reads them (`clients/capture/batches/E08-batch-15a.toml`, E08 T139). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Tiles are (column, row) from 0 at the top left. A walk: click the target once (the path's marks show), then again (the hero goes). Close every message with its OK button.
6. Order: day 1, Red's turn: the start reads (E15-1's first step), E15-2's garrison step, E15-1, E15-2's second visit, E15-3. Then the map is done.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E15-1 — A prisoner freed with seven heroes on the map and one in a garrison (E15-1, E15-2's first half); then a prison visited with eight on the map (E15-2) (ADV-PRI-001, ADV-PRI-002, TWN-SET-005, GAM-SET-002)

- Red hero **E15-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **E15-2g** (Crag Hack, specialty skill Offense): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **Filler 1** (Thorgrim, specialty skill Resistance): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **Filler 2** (Mephala, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **Filler 3** (Neela, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **Filler 4** (Tazar, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. **The start.** Day 1, Red's turn, before any hero moves: read Red's hero list (the count and the heroes in order) and both prisoners' `hero_owner` marks (Rion's, Prisoner 1's type; Adela's, Prisoner 2's): 0x41 means a prisoner. Right-click each prison (P1 at (10, 10), P2 at (16, 10)): note what the popup shows.
2. **E15-2's garrison step.** E15-2g stands at (18, 21), right below Red's town's entrance (18, 20). Click the entrance twice: E15-2g walks in and the town's screen opens. Move E15-2g from the visiting row into the garrison row (drag its portrait up, or select it and swap). Close the town's screen. Read Red's hero list again (the count and the heroes).
3. **E15-1.** E15-1 stands at (10, 12), two tiles below P1's entrance. Note its points. Click P1's entrance (10, 10) twice: E15-1 walks to (10, 11) and visits the prison. Note every message in order. Then read E15-1's tile and points, and the freed hero's: name, tile, facing, movement points and day's maximum, status flags, experience, level, primary skills, secondary skills, spell points and army. Read Red's hero list (the count, the heroes in order, the selected one), Rion's `hero_owner` mark and whether P1 still stands (right-click (10, 10) and (9, 10)).
4. **E15-2.** E15-2 stands at (16, 12), two tiles below P2's entrance. Click P2's entrance (16, 10) twice: E15-2 walks to (16, 11) and visits it. Note every message in order. Then read E15-2's tile and points, Red's hero list, Adela's `hero_owner` mark and whether P2 still stands (right-click it).

**Record**

- the start: Red's hero list (count; heroes in order); Rion's and Adela's `hero_owner`; P1's and P2's popups: 
- after the garrison step: Red's hero list (count; heroes); the town's garrison hero: 
- E15-1 before: points; at P1: the messages (in order): 
- E15-1 after: tile; points: 
- the freed hero: name; tile; facing; points; day's maximum; status flags: 
- the freed hero: experience; level; primary skills; secondary skills; spell points; army: 
- after the freeing: Red's hero list (count; heroes in order; the selected one); Rion's `hero_owner`; P1 still there: 
- E15-2 at P2: the messages (in order); E15-2's tile; points: 
- after P2: Red's hero list (count); Adela's `hero_owner`; P2 still there: 
- anything odd: 

## E15-2 — A prison visited with eight heroes on the map (E15-2's second half; the procedure is E15-1's last step) (ADV-PRI-002)

- Red hero **E15-2** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. E15-2 stands at (16, 12): its walk and visit are E15-1's last step (after the freeing). Record them there.

**Record**

- anything odd: 

## E15-3 — The Market of Time's entrance: a step and nothing else (ADV-OBJ-008)

- Red hero **E15-3** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, after E15-2: E15-3 stands at (22, 12), two tiles below the Market of Time's entrance (22, 10). Note its points. Click (22, 10) once: note the path's marks (the tiles, in order). Click it again: E15-3 walks. Note every message and window, E15-3's tile and points.
2. Press Space (the visit of the tile the hero stands on): note what opens and E15-3's tile and points. Right-click (22, 10) and (21, 10): note the popups.

**Record**

- before: points: 
- the path's marks: 
- after the walk: messages and windows; tile; points: 
- Space: what opens; tile; points: 
- the popups of (22, 10) and (21, 10): 
- anything odd: 

- A prison at [10, 10], its prisoner **Prisoner 1** (Rion, specialty skill First Aid, 5000 experience): attack 1, defense 1, power 2, knowledge 2; Expert Archery, Expert Armorer, Expert Offense, Expert Resistance, Expert Ballistics, Expert Artillery, Expert Tactics, Expert First Aid; no artifacts; no spells; army 4 Swordsman, 10 Pikeman.

- A prison at [16, 10], its prisoner **Prisoner 2** (Adela, specialty spell Bless): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Pikeman.
