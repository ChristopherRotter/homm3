# E08-batch-4a — the scorer's report

`cargo run -p h3-map --example e4_score -- <the batch's map> experiments/results/E08-batch-4a.md` (E08 T32b2): per hero, from its seed byte, each source's prediction beside the record and which fit (the map: the spec compiled by `h3-experiment`, the same stamp). Scored in T33.

## E4-1a (Orrin, Knight)

Seed byte 163.

At load (3200 experience):

- the reading: 3200 experience, level 4, 3/4/1/1, Leadership basic, Armorer basic, Offense advanced; Wisdom last offered 0, a school 4
  - level 2: attack, Leadership | Armorer
  - level 3: defense, Leadership | Offense
  - level 4: defense, Offense | Water Magic
- always-left: 3200 experience, level 4, 3/4/1/1, Leadership expert, Water Magic basic; Wisdom last offered 0, a school 4
  - level 2: attack, Leadership | Armorer
  - level 3: defense, Leadership | Learning
  - level 4: defense, Water Magic | Tactics
- recorded: 3200 experience, level 4, 3/4/1/1, Leadership basic, Armorer basic, Offense advanced; Wisdom last offered 0, a school 4
- fits: the reading

## E4-1b (Valeska, Knight)

Seed byte 245.

At load (3200 experience):

- the reading: 3200 experience, level 4, 4/3/1/1, Archery expert, Ballistics basic; Wisdom last offered 0, a school 3
  - level 2: attack, Archery | Sorcery
  - level 3: defense, Archery | Air Magic
  - level 4: attack, Ballistics | Learning
- always-left: 3200 experience, level 4, 4/3/1/1, Archery expert, Ballistics basic; Wisdom last offered 0, a school 3
  - level 2: attack, Archery | Sorcery
  - level 3: defense, Archery | Air Magic
  - level 4: attack, Ballistics | Learning
- recorded: 3200 experience, level 4, 4/3/1/1, Archery expert, Ballistics basic; Wisdom last offered 0, a school 3
- fits: the reading, always-left

## E4-1c (Edric, Knight)

Seed byte 224.

At load (3200 experience):

- the reading: 3200 experience, level 4, 4/3/1/1, Navigation expert, Archery basic; Wisdom last offered 0, a school 4
  - level 2: attack, Navigation | Archery
  - level 3: attack, Navigation | Leadership
  - level 4: defense, Navigation | Water Magic
- always-left: 3200 experience, level 4, 4/3/1/1, Navigation expert, Water Magic basic; Wisdom last offered 0, a school 4
  - level 2: attack, Navigation | Archery
  - level 3: attack, Navigation | Ballistics
  - level 4: defense, Water Magic | Ballistics
- recorded: 3200 experience, level 4, 4/3/1/1, Navigation expert, Archery basic; Wisdom last offered 0, a school 4
- fits: the reading

## E4-1d (Sylvia, Knight)

Seed byte 19.

At load (3200 experience):

- the reading: 3200 experience, level 4, 3/3/1/2, Wisdom advanced, Offense advanced; Wisdom last offered 3, a school 4
  - level 2: attack, Wisdom | Tactics
  - level 3: knowledge, Wisdom | Offense
  - level 4: defense, Offense | Air Magic
- always-left: 3200 experience, level 4, 3/3/1/2, Wisdom expert, Water Magic basic; Wisdom last offered 3, a school 4
  - level 2: attack, Wisdom | Tactics
  - level 3: knowledge, Wisdom | Offense
  - level 4: defense, Water Magic | Logistics
- recorded: 3200 experience, level 4, 3/3/1/2, Wisdom advanced, Offense advanced; Wisdom last offered 3, a school 4
- fits: the reading

## E4-1e (Clavius, Death Knight)

Seed byte 105.

At load (5000 experience):

- the reading: 5000 experience, level 5, 2/5/1/3, Pathfinding basic, Archery basic, Logistics advanced, Scouting basic, Navigation basic, Mysticism advanced, Offense advanced, Armorer advanced; Wisdom last offered 0, a school 0
  - level 2: knowledge, Navigation | Mysticism
  - level 3: defense, Offense | Pathfinding
  - level 4: knowledge, Armorer | Offense
  - level 5: defense, Logistics | Scouting
- always-left: 5000 experience, level 5, 2/5/1/3, Pathfinding basic, Archery basic, Logistics advanced, Scouting basic, Navigation advanced, Mysticism basic, Offense advanced, Armorer advanced; Wisdom last offered 0, a school 0
  - level 2: knowledge, Navigation | Mysticism
  - level 3: defense, Offense | Pathfinding
  - level 4: knowledge, Armorer | Offense
  - level 5: defense, Logistics | Scouting
- recorded: 5000 experience, level 5, 2/5/1/3, Pathfinding basic, Archery basic, Logistics advanced, Scouting basic, Navigation basic, Mysticism advanced, Offense advanced, Armorer advanced; Wisdom last offered 0, a school 0
- fits: the reading

## E4-2a (Yog, Barbarian)

Seed byte 147.

At load (14699 experience):

- the reading: 14699 experience, level 9, 8/2/2/2, Offense basic, Tactics expert, Ballistics advanced, Armorer advanced, Logistics advanced; Wisdom last offered 6, a school 8
  - level 2: attack, Offense | Ballistics
  - level 3: power, Tactics | Scouting
  - level 4: defense, Tactics | Air Magic
  - level 5: attack, Offense | Armorer
  - level 6: defense, Ballistics | Wisdom
  - level 7: attack, Ballistics | Logistics
  - level 8: knowledge, Armorer | Fire Magic
  - level 9: attack, Logistics | Eagle Eye
- always-left: 14699 experience, level 9, 8/2/2/2, Offense expert, Tactics expert, Wisdom expert, Mysticism basic; Wisdom last offered 8, a school 8
  - level 2: attack, Offense | Ballistics
  - level 3: power, Offense | Resistance
  - level 4: defense, Tactics | Air Magic
  - level 5: attack, Tactics | Artillery
  - level 6: defense, Wisdom | Scouting
  - level 7: attack, Wisdom | Archery
  - level 8: knowledge, Wisdom | Fire Magic
  - level 9: attack, Mysticism | Leadership
- recorded: 14699 experience, level 9, 8/2/2/2, Offense basic, Tactics expert, Ballistics advanced, Armorer advanced, Logistics advanced; Wisdom last offered 6, a school 8
- fits: the reading

In play (1 dialog(s), 14709 experience after):

- the reading: level 10: attack, Ballistics | Archery
- VCMI's row by the old level (D-15): level 10: attack, Ballistics | Archery
- recorded: level 10: attack, Ballistics | Archery (taken left (Ballistics))
- fits: the reading, VCMI's row by the old level (D-15) the rules agree here: inconclusive

## E4-2b (Jabarkas, Barbarian)

Seed byte 189.

At load (14699 experience):

- the reading: 14699 experience, level 9, 8/3/1/2, Offense advanced, Tactics advanced, Logistics expert, Archery advanced, Artillery basic; Wisdom last offered 6, a school 8
  - level 2: defense, Offense | Logistics
  - level 3: defense, Logistics | Archery
  - level 4: attack, Logistics | Air Magic
  - level 5: defense, Tactics | Mysticism
  - level 6: attack, Offense | Wisdom
  - level 7: knowledge, Archery | Artillery
  - level 8: attack, Logistics | Fire Magic
  - level 9: attack, Offense | Artillery
- always-left: 14699 experience, level 9, 8/3/1/2, Offense expert, Tactics expert, Wisdom expert, Resistance basic; Wisdom last offered 8, a school 8
  - level 2: defense, Offense | Logistics
  - level 3: defense, Tactics | Ballistics
  - level 4: attack, Tactics | Air Magic
  - level 5: defense, Offense | Resistance
  - level 6: attack, Wisdom | Artillery
  - level 7: knowledge, Wisdom | Scouting
  - level 8: attack, Wisdom | Fire Magic
  - level 9: attack, Resistance | Mysticism
- recorded: 14699 experience, level 9, 8/3/1/2, Offense advanced, Tactics advanced, Logistics expert, Archery advanced, Artillery basic; Wisdom last offered 6, a school 8
- fits: the reading

In play (1 dialog(s), 14709 experience after):

- the reading: level 10: power, Offense | Scouting
- VCMI's row by the old level (D-15): level 10: defense, Offense | Scouting
- recorded: level 10: power, Offense | Scouting (taken left (Offense))
- fits: the reading 

## E4-2c (Shiva, Barbarian)

Seed byte 30.

At load (14699 experience):

- the reading: 14699 experience, level 9, 8/2/1/3, Offense expert, Tactics advanced, Logistics advanced, Estates advanced, Archery basic; Wisdom last offered 6, a school 8
  - level 2: attack, Offense | Artillery
  - level 3: knowledge, Tactics | Logistics
  - level 4: attack, Logistics | Earth Magic
  - level 5: knowledge, Offense | Pathfinding
  - level 6: defense, Tactics | Wisdom
  - level 7: attack, Tactics | Estates
  - level 8: defense, Estates | Air Magic
  - level 9: attack, Logistics | Archery
- always-left: 14699 experience, level 9, 8/2/1/3, Offense expert, Tactics expert, Wisdom expert, Learning basic; Wisdom last offered 8, a school 9
  - level 2: attack, Offense | Artillery
  - level 3: knowledge, Tactics | Logistics
  - level 4: attack, Tactics | Earth Magic
  - level 5: knowledge, Offense | Ballistics
  - level 6: defense, Wisdom | Luck
  - level 7: attack, Wisdom | Archery
  - level 8: defense, Wisdom | Air Magic
  - level 9: attack, Learning | Air Magic
- recorded: 14699 experience, level 9, 8/2/1/3, Offense expert, Tactics advanced, Logistics advanced, Estates advanced, Archery basic; Wisdom last offered 6, a school 8
- fits: the reading

In play (1 dialog(s), 14709 experience after):

- the reading: level 10: attack, Tactics | Pathfinding
- VCMI's row by the old level (D-15): level 10: attack, Tactics | Pathfinding
- recorded: level 10: attack, Tactics | Pathfinding (taken left (Tactics))
- fits: the reading, VCMI's row by the old level (D-15) the rules agree here: inconclusive

## E4-3a (Luna, Elementalist)

Seed byte 239.

At load (1999 experience):

- the reading: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
  - level 2: attack, Leadership | Intelligence
- always-left: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
  - level 2: attack, Leadership | Intelligence
- recorded: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
- fits: the reading, always-left

In play (1 dialog(s), 2009 experience after):

- the reading: level 3: power, Logistics | Fire Magic
- intervals 3 and 3 (VCMI, HotA: D-16, A-10): level 3: power, Logistics | Wisdom
- recorded: level 3: power, Logistics | Fire Magic (taken left (Logistics))
- fits: the reading 

## E4-3b (Brissa, Elementalist)

Seed byte 209.

At load (1999 experience):

- the reading: 1999 experience, level 2, 0/0/4/3, Leadership basic, Logistics basic, Ballistics basic; Wisdom last offered 0, a school 0
  - level 2: power, Leadership | Ballistics
- always-left: 1999 experience, level 2, 0/0/4/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
  - level 2: power, Leadership | Ballistics
- recorded: 1999 experience, level 2, 0/0/4/3, Leadership basic, Logistics basic, Ballistics basic; Wisdom last offered 0, a school 0
- fits: the reading

In play (1 dialog(s), 2009 experience after):

- the reading: level 3: attack, Logistics | Fire Magic
- intervals 3 and 3 (VCMI, HotA: D-16, A-10): level 3: attack, Logistics | Wisdom
- recorded: level 3: attack, Logistics | Fire Magic (taken left (Logistics))
- fits: the reading 

## E4-3c (Ciele, Elementalist)

Seed byte 124.

At load (1999 experience):

- the reading: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
  - level 2: attack, Leadership | Diplomacy
- always-left: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
  - level 2: attack, Leadership | Diplomacy
- recorded: 1999 experience, level 2, 1/0/3/3, Leadership advanced, Logistics basic; Wisdom last offered 0, a school 0
- fits: the reading, always-left

In play (1 dialog(s), 2009 experience after):

- the reading: level 3: knowledge, Leadership | Sorcery
- intervals 3 and 3 (VCMI, HotA: D-16, A-10): level 3: knowledge, Leadership | Wisdom
- recorded: level 3: knowledge, Leadership | Sorcery (taken left (Leadership))
- fits: the reading 

## E4-4a (Jenova, Ranger)

Seed byte 145.

In play (3 dialog(s), 3449 experience after):

- the reading: level 2: attack, Archery | Wisdom; level 3: defense, Wisdom | Offense; level 4: attack, Archery | Earth Magic
- drawn up front: level 2: attack, Archery | Wisdom; level 3: defense, Archery | Armorer; level 4: attack, Leadership | Earth Magic
- recorded: level 2: attack, Archery | Wisdom (taken right (Wisdom)); level 3: defense, Wisdom | Offense (taken left (Wisdom)); level 4: attack, Archery | Earth Magic (taken right (Earth Magic))
- fits: the reading 

## E4-4b (Ivor, Ranger)

Seed byte 141.

In play (3 dialog(s), 3449 experience after):

- the reading: level 2: attack, Leadership | Water Magic; level 3: defense, Leadership | Artillery; level 4: defense, Leadership | Pathfinding
- drawn up front: level 2: attack, Leadership | Water Magic; level 3: defense, Archery | Learning; level 4: defense, Archery | Intelligence
- recorded: level 2: attack, Leadership | Water Magic (taken right (Water Magic)); level 3: defense, Leadership | Artillery (taken left (Leadership)); level 4: defense, Leadership | Pathfinding (taken right (Pathfinding))
- fits: the reading 

## E4-5a (Fiona, Demoniac)

Seed byte 242.

At load (3199 experience):

- the reading: 3199 experience, level 3, 3/3/1/1, Fire Magic basic, Water Magic basic, Logistics advanced, Offense basic, Armorer basic, Ballistics basic; Wisdom last offered 0, a school 0
  - level 2: defense, Logistics | Ballistics
  - level 3: attack, Logistics | Pathfinding
- always-left: 3199 experience, level 3, 3/3/1/1, Fire Magic advanced, Water Magic basic, Logistics advanced, Offense basic, Armorer basic; Wisdom last offered 0, a school 3
  - level 2: defense, Logistics | Ballistics
  - level 3: attack, Fire Magic | Resistance
- recorded: 3199 experience, level 3, 3/3/1/1, Fire Magic basic, Water Magic basic, Logistics advanced, Offense basic, Armorer basic, Ballistics basic; Wisdom last offered 0, a school 0
- fits: the reading

In play (1 dialog(s), 3209 experience after):

- the reading: level 4: defense, Fire Magic | Resistance
- a known school by its class weight (VCMI, D-17): level 4: defense, Fire Magic | Resistance
- recorded: level 4: defense, Fire Magic | Resistance (taken left (Fire Magic))
- fits: the reading, a known school by its class weight (VCMI, D-17) the rules agree here: inconclusive

## E4-5b (Rashka, Demoniac)

Seed byte 36.

At load (3199 experience):

- the reading: 3199 experience, level 3, 3/2/2/1, Fire Magic advanced, Water Magic basic, Logistics basic, Offense basic, Armorer advanced; Wisdom last offered 0, a school 3
  - level 2: attack, Armorer | Luck
  - level 3: power, Fire Magic | Navigation
- always-left: 3199 experience, level 3, 3/2/2/1, Fire Magic advanced, Water Magic basic, Logistics basic, Offense basic, Armorer advanced; Wisdom last offered 0, a school 3
  - level 2: attack, Armorer | Luck
  - level 3: power, Fire Magic | Navigation
- recorded: 3199 experience, level 3, 3/2/2/1, Fire Magic advanced, Water Magic basic, Logistics basic, Offense basic, Armorer advanced; Wisdom last offered 0, a school 3
- fits: the reading, always-left

In play (1 dialog(s), 3209 experience after):

- the reading: level 4: defense, Offense | Archery
- a known school by its class weight (VCMI, D-17): level 4: defense, Offense | Archery
- recorded: level 4: defense, Offense | Archery (taken left (Offense))
- fits: the reading, a known school by its class weight (VCMI, D-17) the rules agree here: inconclusive

## E4-5c (Marius, Demoniac)

Seed byte 6.

At load (3199 experience):

- the reading: 3199 experience, level 3, 2/3/1/2, Fire Magic basic, Water Magic basic, Logistics expert, Offense basic, Armorer basic; Wisdom last offered 0, a school 0
  - level 2: knowledge, Logistics | Leadership
  - level 3: defense, Logistics | Scouting
- always-left: 3199 experience, level 3, 2/3/1/2, Fire Magic basic, Water Magic basic, Logistics expert, Offense basic, Armorer basic; Wisdom last offered 0, a school 0
  - level 2: knowledge, Logistics | Leadership
  - level 3: defense, Logistics | Scouting
- recorded: 3199 experience, level 3, 2/3/1/2, Fire Magic basic, Water Magic basic, Logistics expert, Offense basic, Armorer basic; Wisdom last offered 0, a school 0
- fits: the reading, always-left

In play (1 dialog(s), 3209 experience after):

- the reading: level 4: attack, Water Magic | First Aid
- a known school by its class weight (VCMI, D-17): level 4: attack, Fire Magic | First Aid
- recorded: level 4: attack, Water Magic | First Aid (taken left (Water Magic))
- fits: the reading 

