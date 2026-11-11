# E09 batch TWN-3b — results

> Played by the capture harness on 2026-11-11 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-3b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-3b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `a20d41e4`.** The map's description in the scenario list starts with "Stamp a20d41e4.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-3b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml`): its garrison hero +0xC, its visitor +0x10, its available creatures +0x16 (i16 by the faction's index: 0 Pikemen, 7 Halberdiers), its garrison +0xE0; a hero's army and movement points (E08's). The capture harness reads them (`clients/capture/batches/E09-batch-3b.toml`, E09 T23). Played by hand they show in the town screen, the hire dialog and the hero's screen.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes). On the town screen the lower portrait is the visitor, the upper the garrison's (the town's flag without a garrison hero): click the lower, then the upper, to move the visitor into the garrison. A dwelling's building opens its hire dialog.
6. Order: day 1, Red's turn: G's visit to T, then V's to U; then end Red's and Blue's turns and read the heroes' points on Red's day 2.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-3-6 — Recruitment with a garrison hero (T's upgraded Guardhouse); a visitor whose army does not fit the garrison (U); the garrison hero's points at Red's next turn (TWN-REC-001, TWN-ROW-002, TWN-ROW-003, TWN-ROW-004)

- Red hero **TWN-3-6** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Halberdier
- Red hero **TWN-3-6v** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman, 1 Halberdier, 1 Swordsman, 1 Crusader
- Red hero **TWN-3-6z** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Halberdier
- Blue hero **TWN-3-6b** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Read T's (entrance (7, 7)) and U's (15, 7) garrison hero, visitor, available creatures and garrison; G's (TWN-3-6) and V's (TWN-3-6v) armies and points; Red's resources.
2. G stands south of T at (7, 8). Step north onto the entrance (the town screen opens). Click G's portrait (the lower bar), then the upper portrait (the town's flag): G moves into the garrison. Read T's garrison hero, visitor, garrison and G's army. Click the upgraded Guardhouse's building: the hire dialog opens; note its creatures and counts. Recruit 2 Halberdiers (the creature selected first); note the dialog's counts after. Close the dialog and the town screen. Read T's available creatures, its garrison, G's army and points, Red's resources.
3. V stands south of U at (15, 8). Step north onto the entrance (the town screen opens). Click V's portrait, then the upper portrait: note the message that shows and its OK. Read U's garrison hero, visitor and garrison, and V's army. Click the Guardhouse's building: recruit 1 Pikeman; read U's garrison and V's army. Close the dialog and the town screen.
4. End Red's turn, then Blue's. On Red's day 2 read G's points (the garrison hero) and Z's (TWN-3-6z, who never moved, with the same army's speed: the maximum), and T's garrison hero.

**Record**

- before: T's garrison hero, visitor, available creatures, garrison; U's the same: T hero: none; visitor: none; base 0, 0, 0, 0, 0, 0, 0; upgraded 14, 0, 0, 0, 0, 0, 0; empty; U hero: none; visitor: none; base 14, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; slot 0: 1 Archer, slot 1: 1 Griffin, slot 2: 1 Monk, slot 3: 1 Cavalier
- before: G's army and points; V's army and points; Red's resources: G: 1 Halberdier; 1630; V: 1 Pikeman, 1 Halberdier, 1 Swordsman, 1 Crusader; 1560; Red: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30700 gold
- G into T's garrison: the messages; T's garrison hero, visitor, garrison; G's army: none; hero: TWN-3-6; visitor: none; garrison: empty; G: 1 Halberdier
- T's upgraded Guardhouse: the hire dialog as it opened (creatures, counts): 14 Halberdiers, 14 Pikemen; most 14; amount 0; ally 0; selected 0 (counts 14, 14); max enabled, recruit disabled, close enabled, slider enabled
- 2 Halberdiers recruited: the dialog after (counts): closed; closed
- after G's visit: T's available creatures and garrison; G's army and points; Red's resources: base 0, 0, 0, 0, 0, 0, 0; upgraded 12, 0, 0, 0, 0, 0, 0; garrison: empty; G: 3 Halberdiers; 1530; Red: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30550 gold
- V into U's garrison: the messages; U's garrison hero, visitor, garrison; V's army: GENRLTXT:276; hero: none; visitor: TWN-3-6v; garrison: slot 0: 1 Archer, slot 1: 1 Griffin, slot 2: 1 Monk, slot 3: 1 Cavalier; V: 1 Pikeman, 1 Halberdier, 1 Swordsman, 1 Crusader
- 1 Pikeman recruited at U: U's garrison; V's army; U's available creatures: garrison: slot 0: 1 Archer, slot 1: 1 Griffin, slot 2: 1 Monk, slot 3: 1 Cavalier, slot 4: 1 Pikeman; V: 1 Pikeman, 1 Halberdier, 1 Swordsman, 1 Crusader; base 13, 0, 0, 0, 0, 0, 0; upgraded 0, 0, 0, 0, 0, 0, 0; closed
- Red's day 2: G's points; Z's points; T's garrison hero; G's army: G: 1630; of the day's: 1630; Z: 1630; of the day's: 1630; T's garrison hero: TWN-3-6; G's army: 3 Halberdiers
- anything odd: the game's version: SoD; G's visit: a town screen; how it closed: yes; click; U's Guardhouse: the hire dialog as it opened: 14 Pikemen; most 14; amount 0; ally 0; selected 0 (counts 14); max enabled, recruit disabled, close enabled, slider enabled; V's visit: a town screen; how it closed: yes; click; Blue's turn: G's and Z's points: G: 1530; Z: 1630; the date: month 1, week 1, day 2
