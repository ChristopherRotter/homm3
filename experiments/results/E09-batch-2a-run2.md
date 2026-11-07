# E09 batch TWN-2 — results

> **The second recorded run** (E09 T14b; the main record is `E09-batch-2a.md`): the same recipe and map, played after the first; every record the same but the treasuries' gold, the random starting bonus (Red 500 here, 600 there; Blue the other way round).

> Played by the capture harness on 2026-11-07 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-2a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-2a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `1128298c`.** The map's description in the scenario list starts with "Stamp 1128298c.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

**Teams: red and blue.** The map makes them allies; any other player plays alone.

1. Copy `E09-batch-2a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human (one team: the map's alliance), Tan the computer, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The records are the game's memory: a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml`): its day's mark +0x2 (survey E09-construction.md §3), its standing set +0x150 and every-built set +0x158 (a bit per building id); a player's resources (the player record +0x9C: wood, mercury, ore, sulfur, crystal, gems, gold). The capture harness reads them (`clients/capture/batches/E09-batch-2a.toml`, E09 T14b). Played by hand, the sets show in the town screen and the resources in the kingdom's bar.
5. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes). The hall: on the town screen, the Village Hall's building (or its successor's); in the hall, a building's slot; in its dialog, the Build button.
6. Order: day 1, Red's turn, before any hero moves: the reads of TWN-2-1, then its two visits; then TWN-2-2. Close each town screen before the next step.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-2-1 — The Conflux's upgraded Altar of Fire (40): C1 without the upgraded Altar of Air (38), C2 with it (TD-9, TWN-BLD-002, TWN-BLD-007)

- Red hero **TWN-2-1** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Red hero **TWN-2-1c** (Jenova, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. Read C1's (entrance (7, 7)) and C2's (17, 7) standing and every-built sets and day's marks, and Red's resources.
2. C1: Red's hero TWN-2-1 stands south of it at (7, 8). Step north onto the entrance (the town screen opens). Open the hall (a click on the hall's building); the level-4 slot shows the upgraded Altar of Fire. Note the slot's look (built, buildable, not now, never). Click the slot: note whether the dialog's Build button is enabled or greyed and which buildings the text under it names. Press Build (a greyed one does nothing) and note what shows then; close what is left open (the dialog by its cancel, the hall, the town screen). Read C1's sets and mark, and Red's resources.
3. C2: Red's hero TWN-2-1c stands south of it at (17, 8). The same: step in, the hall, the level-4 slot, its dialog (Build's state, the text), Build pressed, what shows then, the screens closed. Read C2's sets and mark, and Red's resources.

**Record**

- before: C1's standing and every-built sets, its mark; C2's the same; Red's resources: C1 standing: 0, 7, 10, 23, 30, 31, 33; every: 0, 7, 10, 23, 30, 31, 33; mark: 0; C2 standing: 0, 7, 10, 23, 30, 33, 38; every: 0, 7, 10, 23, 30, 31, 33, 38; mark: 0; Red: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold
- C1: the screen after the hall's click; the level-4 slot's look: hall; not now
- C1: the slot's dialog: Build (enabled or greyed); the text (its record, the buildings it names): greyed (state 0xe); GENRLTXT:53, names 38
- C1: the screen after Build pressed: build
- C1 after the visit: its standing and every-built sets, its mark; Red's resources: standing: 0, 7, 10, 22, 23, 30, 31, 33; every: 0, 7, 10, 23, 30, 31, 33; mark: 0; Red: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30500 gold
- C2: the screen after the hall's click; the level-4 slot's look: hall; buildable
- C2: the slot's dialog: Build (enabled or greyed); the text (its record, the buildings it names): enabled (state 0x6); GENRLTXT:220
- C2: the screen after Build pressed: town
- C2 after the visit: its standing and every-built sets, its mark; Red's resources: standing: 0, 7, 10, 22, 23, 30, 38, 40; every: 0, 7, 10, 22, 23, 30, 31, 33, 38, 40; mark: 1; Red: 30 wood, 10 mercury, 25 ore, 15 sulfur, 15 crystal, 15 gems, 28500 gold
- anything odd: the game's version: SoD; the teams: red 0, blue 0, tan 1, green 0, orange 0, purple 0, teal 0, pink 0; C1's standing set while its town screen shows: 0, 7, 10, 22, 23, 30, 31, 33; C1's visit: a town screen; how it closed: yes; click; C2's standing set while its town screen shows: 0, 7, 10, 22, 23, 30, 33, 38; C2's visit: a town screen; how it closed: yes; click

## TWN-2-2 — An allied visitor's build: Red's hero in Blue's town (Red and Blue one team), the hall's Tavern (TD-11, TWN-BLD-001)

- Red hero **TWN-2-2** (Octavia, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman
- Blue hero **TWN-2-2b** (Aine, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. After TWN-2-1, in the same turn (Red's, day 1). Blue's Castle B1 (a Fort only) has its entrance at (27, 7); Red's hero TWN-2-2 stands south of it at (27, 8). Read B1's standing and every-built sets and mark, and both players' resources.
2. Step north onto the entrance: Red's hero enters its ally's town (a town screen: yes/no). Open the hall (what shows then); the Tavern's slot shows the Tavern. Note the slot's look; click it: note whether Build is enabled or greyed and what the text says. Press Build and note what shows then; close what is left open (the dialog by its cancel, the hall, the town screen).
3. Read B1's standing and every-built sets and mark, and both players' resources.

**Record**

- before: B1's standing and every-built sets, its mark; Red's resources; Blue's resources: standing: 7, 10; every: 7, 10; mark: 0; Red: 30 wood, 10 mercury, 25 ore, 15 sulfur, 15 crystal, 15 gems, 28500 gold; Blue: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30600 gold
- the step: a town screen (yes/no): yes
- the screen after the hall's click; the Tavern's slot's look: hall; buildable
- the Tavern's dialog: Build (enabled or greyed); the text (its record, the buildings it names): greyed (state 0x26); GENRLTXT:220
- the screen after Build pressed: build
- after the visit: B1's standing and every-built sets, its mark; Red's resources; Blue's resources: standing: 7, 10, 23; every: 7, 10; mark: 0; Red: 30 wood, 10 mercury, 25 ore, 15 sulfur, 15 crystal, 15 gems, 28500 gold; Blue: 30 wood, 15 mercury, 30 ore, 15 sulfur, 15 crystal, 15 gems, 30600 gold
- anything odd: B1's standing set while its town screen shows: 7, 10, 23; B1's visit: how the town screen closed: click
