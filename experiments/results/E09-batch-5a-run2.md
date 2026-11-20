# E09 batch TWN-5a — results

> Played by the capture harness on 2026-11-20 (`clients/capture/h3adv.py batch clients/capture/batches/E09-batch-5a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus gold, Show Route on; read from the game's memory.

Map file: `E09-batch-5a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `fe8d6a6b`.** The map's description in the scenario list starts with "Stamp fe8d6a6b.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E09-batch-5a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, Red and Blue both human, difficulty **Easy**, Red's starting hero Random, both players' starting bonus Gold. Before you start it, compare the stamp at the start of the map's description with the stamp above.
4. The scenario bans Armageddon (the scenario's information does not show it).
5. The records are the game's memory (the capture harness, `clients/capture/batches/E09-batch-5a.toml`, E09 T44): a town's record (game +0x21614, 0x168 bytes each, `docs/exe/structs.toml` "town"): its guild's six slots a level (+0x44), the count each level shows (+0xBC), the guild level (+0x14); a hero's learned spells (+0x3EA). Played by hand they show in the Mage Guild's window and the hero's spell book.
6. Tiles are (column, row) from 0 at the top left; a town's entrance is the tile its gate stands on. A step: click the target once (the path's marks show), then again (the hero goes).
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## TWN-5-1 — A level-1 spell both required and banned by its town (TD-25): the guilds' slots (TD-25, TWN-GLD-002, TWN-SET-012)

- Red hero **TWN-5-1** (Orrin, specialty skill Archery): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves. For R1a (15, 7), R1b (23, 7) and R1c (31, 7): note the guild's level-1 spells (the window shows five; the record's sixth slot is the spare) and every level's slots.
2. For every town of the map: note whether any guild slot holds Armageddon or Titan's Lightning Bolt.

**Record**

- R1a (Magic Arrow required and banned): the guild by level (shown | spare): level 1 shows 5: Protection from Fire, Shield, Bless, Cure, Slow | spare View Earth; level 2 shows 0: none | spare Scuttle Boat, Fire Wall, Ice Bolt, Blind, Weakness; level 3 shows 0: none | spare Mirth, Forgetfulness, Air Shield, Earthquake; level 4 shows 0: none | spare Water Walk, Berserk, Sorrow; level 5 shows 0: none | spare empty, empty
- R1b (Haste required and banned): the guild by level: level 1 shows 5: Shield, Stone Skin, Magic Arrow, View Earth, View Air | spare Dispel; level 2 shows 0: none | spare Blind, Ice Bolt, Quicksand, Disguise, Fortune; level 3 shows 0: none | spare Land Mine, Frost Ring, Misfortune, Destroy Undead; level 4 shows 0: none | spare Prayer, Water Walk, Frenzy; level 5 shows 0: none | spare empty, empty
- R1c (both required, none banned): the guild by level: level 1 shows 5: Magic Arrow, Haste, Shield, Summon Boat, Dispel | spare Cure; level 2 shows 0: none | spare Ice Bolt, Quicksand, Fortune, Blind, Scuttle Boat; level 3 shows 0: none | spare Prot. from Earth, Land Mine, Teleport, Earthquake; level 4 shows 0: none | spare Slayer, Frenzy, Clone; level 5 shows 0: none | spare empty, empty
- every town: a slot with Armageddon or Titan's Lightning Bolt: R4: level 1 shows 5: Shield, Magic Arrow, Dispel, Slow, Prot. from Water | spare Stone Skin; level 2 shows 4: Remove Obstacle, Precision, Lightning Bolt, Quicksand | spare Fire Wall; level 3 shows 0: none | spare Teleport, Destroy Undead, Land Mine, Earthquake; level 4 shows 0: none | spare Prayer, Sorrow, Clone; level 5 shows 0: none | spare Air Elemental, Earth Elemental;  || R3: level 1 shows 5: Protection from Fire, Prot. from Water, Slow, View Earth, Curse | spare View Air; level 2 shows 4: Quicksand, Lightning Bolt, Ice Bolt, Scuttle Boat | spare Precision; level 3 shows 3: Air Shield, Force Field, Mirth | spare Frost Ring; level 4 shows 2: Meteor Shower, Water Walk | spare Town Portal; level 5 shows 1: Fly | spare Magic Mirror;  || R2: level 1 shows 5: Cure, Slow, Magic Arrow, Shield, Bloodlust | spare Dispel; level 2 shows 0: none | spare Ice Bolt, Lightning Bolt, Remove Obstacle, Disrupting Ray, Protection from Air; level 3 shows 0: none | spare Forgetfulness, Earthquake, Force Field, Teleport; level 4 shows 0: none | spare Water Walk, Clone, Meteor Shower; level 5 shows 0: none | spare Magic Mirror, Water Elemental
- anything odd: 

## TWN-5-2 — The Aurora Borealis with Mage Guild 1 (TD-26, T-18, §5): the spells an Expert in Wisdom learns against the town's and the map's bans (TD-26, T-18, TWN-GLD-004)

- Red hero **TWN-5-2** (Grindan, specialty resource gold): attack 1, defense 1, power 1, knowledge 2; Expert Wisdom; Spell Book (spell book); no spells; army 1 Pikeman

1. Red's turn, day 1. Note TWN-5-2's spells (its own).
2. TWN-5-2 stands south of R2 at (15, 16): step north onto the entrance (15, 15). The town screen opens: close it.
3. Note TWN-5-2's spells by level, and whether Implosion (the town's ban) and Armageddon (the map's) are among them.

**Record**

- before: TWN-5-2's spells: none
- after the visit: TWN-5-2's spells: Summon Boat, Scuttle Boat, Visions, View Earth, Disguise, View Air, Fly, Water Walk, Dimension Door, Town Portal, Quicksand, Land Mine, Force Field, Fire Wall, Earthquake, Magic Arrow, Ice Bolt, Lightning Bolt, Chain Lightning, Frost Ring, Fireball, Inferno, Meteor Shower, Death Ripple, Destroy Undead, Armageddon, Shield, Air Shield, Fire Shield, Protection from Air, Protection from Fire, Prot. from Water, Prot. from Earth, Anti-Magic, Dispel, Magic Mirror, Cure, Resurrection, Animate Dead, Sacrifice, Bless, Curse, Bloodlust, Precision, Weakness, Stone Skin, Disrupting Ray, Prayer, Mirth, Sorrow, Fortune, Misfortune, Haste, Slow, Slayer, Frenzy, Counterstrike, Berserk, Hypnotize, Forgetfulness, Blind, Teleport, Remove Obstacle, Clone, Fire Elemental, Earth Elemental, Water Elemental, Air Elemental
- after the visit: Implosion; Armageddon; Titan's Lightning Bolt (learned or not): Implosion not learned; Armageddon learned; Titan's Lightning Bolt not learned (the agent, from the line above: the recipe has no read of its own; 68 spells)
- R2: the guild by level; the guild level: level 1 shows 5: Cure, Slow, Magic Arrow, Shield, Bloodlust | spare Dispel; level 2 shows 0: none | spare Ice Bolt, Lightning Bolt, Remove Obstacle, Disrupting Ray, Protection from Air; level 3 shows 0: none | spare Forgetfulness, Earthquake, Force Field, Teleport; level 4 shows 0: none | spare Water Walk, Clone, Meteor Shower; level 5 shows 0: none | spare Magic Mirror, Water Elemental; ; guild level: 1; ; every built: 0, 10, 23, 26
- anything odd: TWN-5-2's visit: a town screen; how it closed: yes; click

## TWN-5-3 — The Library built in the game with a visitor (§3.3, §4): each level's count before and after, the spares learned at once (TWN-GLD-003, TWN-GLD-005)

- Red hero **TWN-5-3** (Damacon, specialty resource gold): attack 1, defense 1, power 1, knowledge 2; Expert Wisdom; Spell Book (spell book); no spells; army 1 Pikeman

1. Red's turn, day 1. Note R3's guild by level (shown | spare) and TWN-5-3's spells.
2. TWN-5-3 stands south of R3 at (7, 16): step north onto the entrance (7, 15). On the town screen, note TWN-5-3's spells and R3's guild.
3. Set Red's resources to 30 of each and 50 000 gold. Open the hall, click the Library and Build. Note at once TWN-5-3's spells and R3's guild; close the town screen.

**Record**

- before: R3's guild by level; TWN-5-3's spells: level 1 shows 5: Protection from Fire, Prot. from Water, Slow, View Earth, Curse | spare View Air; level 2 shows 4: Quicksand, Lightning Bolt, Ice Bolt, Scuttle Boat | spare Precision; level 3 shows 3: Air Shield, Force Field, Mirth | spare Frost Ring; level 4 shows 2: Meteor Shower, Water Walk | spare Town Portal; level 5 shows 1: Fly | spare Magic Mirror;  || TWN-5-3: none
- on the town screen (the visit): TWN-5-3's spells; R3's guild: Scuttle Boat, View Earth, Fly, Water Walk, Quicksand, Force Field, Ice Bolt, Lightning Bolt, Meteor Shower, Air Shield, Protection from Fire, Prot. from Water, Curse, Mirth, Slow;  || R3: level 1 shows 5: Protection from Fire, Prot. from Water, Slow, View Earth, Curse | spare View Air; level 2 shows 4: Quicksand, Lightning Bolt, Ice Bolt, Scuttle Boat | spare Precision; level 3 shows 3: Air Shield, Force Field, Mirth | spare Frost Ring; level 4 shows 2: Meteor Shower, Water Walk | spare Town Portal; level 5 shows 1: Fly | spare Magic Mirror
- the Library built: TWN-5-3's spells; R3's guild; R3's buildings: Scuttle Boat, View Earth, View Air, Fly, Water Walk, Town Portal, Quicksand, Force Field, Ice Bolt, Lightning Bolt, Frost Ring, Meteor Shower, Air Shield, Protection from Fire, Prot. from Water, Magic Mirror, Curse, Precision, Mirth, Slow;  || R3: level 1 shows 6: Protection from Fire, Prot. from Water, Slow, View Earth, Curse, View Air | spare none; level 2 shows 5: Quicksand, Lightning Bolt, Ice Bolt, Scuttle Boat, Precision | spare none; level 3 shows 4: Air Shield, Force Field, Mirth, Frost Ring | spare none; level 4 shows 3: Meteor Shower, Water Walk, Town Portal | spare none; level 5 shows 2: Fly, Magic Mirror | spare none;  || built: 4, 10, 22; ; every: 0, 1, 2, 3, 4, 10, 22
- anything odd: TWN-5-3's visit: a town screen; how it closed: yes; click

## TWN-5-4 — A main town's hero with a spell book on day 1 (TD-10, §12): the guild's spells learned at the start (TD-10, TWN-GLD-005, TWN-SET-009)

- Red hero **TWN-5-4** (Valeska, specialty creature Archer): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 1 Pikeman

1. Red's turn, day 1, before any hero moves: the hero the game generated stands on R4's entrance (7, 7). Select another hero of Red's (TWN-5-4), then note the generated hero's type, secondary skills and spells, and R4's guild by level.

**Record**

- R4's visitor (the generated hero): its name; its secondary skills: Torosar ; ; skills: Mysticism basic, Tactics basic; ; the visitor field: Torosar 
- R4's visitor: its spells on day 1: Quicksand, Magic Arrow, Lightning Bolt, Shield, Prot. from Water, Dispel, Precision, Slow, Remove Obstacle
- R4: the guild by level; the guild level: level 1 shows 5: Shield, Magic Arrow, Dispel, Slow, Prot. from Water | spare Stone Skin; level 2 shows 4: Remove Obstacle, Precision, Lightning Bolt, Quicksand | spare Fire Wall; level 3 shows 0: none | spare Teleport, Destroy Undead, Land Mine, Earthquake; level 4 shows 0: none | spare Prayer, Sorrow, Clone; level 5 shows 0: none | spare Air Elemental, Earth Elemental; ; guild level: 2
- anything odd: the game's version: SoD
