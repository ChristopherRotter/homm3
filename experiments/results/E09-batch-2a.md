# E09 batch TWN-2 — results

Map file: `E09-batch-2a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `cdf47995`.** The map's description in the scenario list starts with "Stamp cdf47995.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

**Teams: red and blue.** The map makes them allies; any other player plays alone.

1. Copy `E09-batch-2a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human (one team: the map's alliance), difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml`): its day's mark +0x2 (survey E09-construction.md §3), its standing set +0x150 and every-built set +0x158 (a bit per building id); a player's resources (the player record +0x9C: wood, mercury, ore, sulfur, crystal, gems, gold). The capture harness reads them (`clients/capture/batches/E09-batch-2a.toml`, E09 T14b). Played by hand, the sets show in the town screen and the resources in the kingdom's bar.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes). The hall: on the town screen, the Village Hall's building (or its successor's); in the hall, a building's slot; in its dialog, the Build button.
6. Order: day 1, Red's turn, before any hero moves: the reads of TWN-2-1, then its two visits; then TWN-2-2. Close each town screen before the next step.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-2-1 — The Conflux's upgraded Altar of Fire (40): C1 without the upgraded Altar of Air (38), C2 with it (TD-9, TWN-BLD-002, TWN-BLD-007)

- Red hero **TWN-2-1** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-2-1c** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Read C1's (entrance (7, 7)) and C2's (17, 7) standing and every-built sets and day's marks, and Red's resources.
2. C1: Red's hero TWN-2-1 stands south of it at (7, 8). Step north onto the entrance (the town screen opens). Open the hall; the level-4 slot shows the upgraded Altar of Fire. Note the slot's look (built, buildable, short of resources, not now). Click the slot: note whether the dialog's Build button is enabled and which buildings the text under it names. Press Build (it does nothing when disabled); close the dialogs and the town screen. Read C1's sets and mark, and Red's resources.
3. C2: Red's hero TWN-2-1c stands south of it at (17, 8). The same: step in, the hall, the level-4 slot, its dialog (Build's state, the text), Build pressed, the screens closed. Read C2's sets and mark, and Red's resources.

**Record**

- before: C1's standing and every-built sets, its mark; C2's the same; Red's resources: 
- C1: the slot's look; Build enabled (yes/no); the buildings the text names: 
- C1 after Build pressed: its standing and every-built sets, its mark; Red's resources: 
- C2: the slot's look; Build enabled (yes/no); the buildings the text names: 
- C2 after Build pressed: its standing and every-built sets, its mark; Red's resources: 
- anything odd: 

## TWN-2-2 — An allied visitor's build: Red's hero in Blue's town (Red and Blue one team), the hall's Tavern (TD-11, TWN-BLD-001)

- Red hero **TWN-2-2** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. After TWN-2-1, in the same turn (Red's, day 1). Blue's Castle B1 (a Fort only) has its entrance at (27, 7); Red's hero TWN-2-2 stands south of it at (27, 8). Read B1's standing and every-built sets and mark, and both players' resources.
2. Step north onto the entrance: Red's hero enters its ally's town (a town screen: yes/no). Open the hall; the Tavern's slot shows the Tavern. Note the slot's look; click it: note whether Build is enabled and what the text says. Press Build; close the dialogs and the town screen.
3. Read B1's standing and every-built sets and mark, and both players' resources.

**Record**

- before: B1's standing and every-built sets, its mark; Red's resources; Blue's resources: 
- the step: a town screen (yes/no); the hall opened (yes/no): 
- the Tavern's slot: its look; Build enabled (yes/no); the text's refusal, if any: 
- after Build pressed: B1's standing and every-built sets, its mark; Red's resources; Blue's resources: 
- anything odd: 
