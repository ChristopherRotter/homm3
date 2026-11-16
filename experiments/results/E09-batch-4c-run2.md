# E09 batch TWN-4c — results

> Played by the capture harness on 2026-11-16 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-4c.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-4c.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `9cfcd174`.** The map's description in the scenario list starts with "Stamp 9cfcd174.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-4c.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. Set the players' resources as the encounter says (the capture harness writes them, player record +0x9C; by hand: the console or a trainer, or note the resources each player has and score the rows by them).
5. The Thieves' Guild opens from a town's Tavern (the town screen's Tavern building, then the Thieves' Guild button) and at the Den of Thieves; it shows each row's places by the players' flags and, per player, its best hero's portrait and primary skills, its tactic and its best creature, by the viewer's taverns. Note each row's places (the colours in each place, from the first) and each player's parts.
6. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile a hero visits. A step: click the target once (the path's marks show), then again (the hero goes).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-4-7 — The Thieves' Guild at 1 to 5 taverns and at the Den of Thieves: the rows' places and the players' parts (TWN-TAV-007, TWN-TAV-008, TWN-TAV-009)

- Red hero **TWN-4-7s** (Orrin, specialty skill Archery): attack 5, defense 5, power 1, knowledge 1; no skills; Badge of Courage (misc 1), Clover of Fortune (misc 2), Cards of Prophecy (misc 3); no spells; army 10 Champion
- Red hero **TWN-4-7e** (Valeska, specialty creature Archer, 30000 experience): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-7a** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-7b** (Sylvia, specialty skill Navigation): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-7c** (Sorsha, specialty creature Swordsman): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-7f** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-4-7d** (Rion, specialty skill First Aid): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **TWN-4-7x** (Tazar, specialty skill Armorer): attack 1, defense 1, power 1, knowledge 1; no skills; Ladybird of Luck (backpack); no spells; army 20 Champion

1. Red's turn, day 1. Set the resources (Red: gold 20 000, wood 5, ore 5, the rare four 10 each; Blue: gold 10 000, wood 20, ore 20, the rare four 1 each).
2. Open R1's town screen from the town list (no hero visits it), click the Tavern, then the Thieves' Guild button: note each row's places and each player's parts (one tavern). Close the guild, the tavern and the town screen.
3. TWN-4-7a stands south of N2 at (15, 11): step north onto the entrance (the town is taken, its screen opens). Click the Tavern, then the Thieves' Guild: note the rows and the parts (two taverns). Close them.
4. The same with TWN-4-7b at N3 (23, 11) (three taverns), TWN-4-7c at N4 (31, 11) (four) and TWN-4-7f at N5 (39, 11) (five).
5. TWN-4-7d stands south of the Den of Thieves at (52, 11): step north onto it; note its message, then the guild's rows and parts. Close it.

**Record**

- one tavern (R1): the rows' places; the parts: 2 rows: towns: 1st blue+red; heroes: 1st red, 2nd blue; parts: red: hero TWN-4-7s (type 0); blue: hero TWN-4-7x (type 115)
- two taverns (N2 taken): the rows' places; the parts: 4 rows: towns: 1st red, 2nd blue; heroes: 1st red, 2nd blue; gold: 1st red, 2nd blue; wood and ore: 1st blue, 2nd red; parts: red: hero TWN-4-7s (type 0), skills 5 5 1 1; blue: hero TWN-4-7x (type 115), skills 1 1 1 1
- three taverns (N3 taken): the rows' places; the parts: 6 rows: towns: 1st red, 2nd blue; heroes: 1st red, 2nd blue; gold: 1st red, 2nd blue; wood and ore: 1st blue, 2nd red; rare resources: 1st red, 2nd blue; obelisks: 1st blue+red; parts: red: hero TWN-4-7s (type 0), skills 5 5 1 1, tactic GENRLTXT:500; blue: hero TWN-4-7x (type 115), skills 1 1 1 1, tactic GENRLTXT:500
- four taverns (N4 taken): the rows' places; the parts: 8 rows: towns: 1st red, 2nd blue; heroes: 1st red, 2nd blue; gold: 1st red, 2nd blue; wood and ore: 1st blue, 2nd red; rare resources: 1st red, 2nd blue; obelisks: 1st blue+red; artifacts: 1st blue, 2nd red; army: 1st red, 2nd blue; parts: red: hero TWN-4-7s (type 0), skills 5 5 1 1, tactic GENRLTXT:500, creature Archangel; blue: hero TWN-4-7x (type 115), skills 1 1 1 1, tactic GENRLTXT:500, creature Champion
- five taverns (N5 taken): the rows' places; the parts: 9 rows: towns: 1st red, 2nd blue; heroes: 1st red, 2nd blue; gold: 1st red, 2nd blue; wood and ore: 1st blue, 2nd red; rare resources: 1st red, 2nd blue; obelisks: 1st blue+red; artifacts: 1st blue, 2nd red; army: 1st red, 2nd blue; income: 1st red, 2nd blue; parts: red: hero TWN-4-7s (type 0), skills 5 5 1 1, tactic GENRLTXT:500, creature Archangel; blue: hero TWN-4-7x (type 115), skills 1 1 1 1, tactic GENRLTXT:500, creature Champion
- the Den of Thieves: its message; the rows' places; the parts: ADVEVENT:142; 9 rows: towns: 1st red, 2nd blue; heroes: 1st red, 2nd blue; gold: 1st red, 2nd blue; wood and ore: 1st blue, 2nd red; rare resources: 1st red, 2nd blue; obelisks: 1st blue+red; artifacts: 1st blue, 2nd red; army: 1st red, 2nd blue; income: 1st red, 2nd blue; parts: red: hero TWN-4-7s (type 0), skills 5 5 1 1, tactic GENRLTXT:500, creature Archangel; blue: hero TWN-4-7x (type 115), skills 1 1 1 1, tactic GENRLTXT:500, creature Champion
- anything odd: R1's tavern dialog: offers shown 0, 1; selected 0 (framed 0); Recruit enabled (state 0x6); the game's version; Red's and Blue's resources: SoD; red: 5 wood, 10 mercury, 5 ore, 10 sulfur, 10 crystal, 10 gems, 20000 gold; blue: 20 wood, 1 mercury, 20 ore, 1 sulfur, 1 crystal, 1 gems, 10000 gold; N2's owner; how the screens closed: red; click; N3's owner: red; N4's owner: red; N5's owner; Red's resources: red; 5 wood, 10 mercury, 5 ore, 10 sulfur, 10 crystal, 10 gems, 20000 gold; 1 dialog(s) after the walk to [52, 10]; the Den's visit: the dialogs in order: message ADVEVENT:142; guild window
