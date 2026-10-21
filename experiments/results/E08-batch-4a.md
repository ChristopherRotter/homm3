# E08 batch E4a — results

> Played by the capture harness on 2026-10-21 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-4a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.
>
> Run 1 of 4 (E08 T32b2; the others: `E08-batch-4a.md` (run 1), `E08-batch-4a-run2.md` to `-run4.md`; each beside its scorer's report, `…-score.md`, from `crates/h3-map/examples/e4_score.rs`). This run's recipe noted every worn artifact under "anything odd", the Catapult every hero wears among them: those notes were removed by hand (the recipe has noted `equipment` since, the artifacts but the war machines); none else was worn.

Map file: `E08-batch-4a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `e9f2ca7f`.** The map's description in the scenario list starts with "Stamp e9f2ca7f.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-4a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Every hero's seed byte (hero +0x8F) and the levels Wisdom (+0x90) and a magic school (+0x3F) were last offered show only in the game's memory: the capture harness reads them (clients/capture/batches/E08-batch-4a.toml). Played by hand, leave them empty.
5. Each level-up dialog: note the new level, the primary skill it raises, the skill offered left and right (one skill alone: under left, right empty) and the one taken; answer as the encounter says.
6. Order: day 1, Red's turn: E4-2a to E4-2c, E4-3a to E4-3c, E4-4a, E4-4b, each as its encounter says; end the turn. Blue's turn: E4-5a to E4-5c; end the turn. E4-1's heroes never move. Then the map is done.
7. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E4-1a — Levels at load: a Knight with Basic Leadership and 3 200 experience (D-20, D-14, GAM-SET-002, HER-LVL-006)

- Blue hero **E4-1a** (Orrin, specialty skill Archery, 3200 experience): attack 2, defense 2, power 1, knowledge 1; Basic Leadership; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before anything moves: Blue's hero E4-1a (a Knight, 3 200 experience on the map, 2/2/1/1, Basic Leadership) never moves. Read its seed byte and the two levels last offered (the harness, from the game's memory), then its experience, level, primary and secondary skills (its portrait's window).

**Record**

- seed byte (hero +0x8F): 163
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 4
- experience at the start: 3200
- level at the start: 4
- primary skills at the start (attack/defense/power/knowledge): 3/4/1/1
- secondary skills at the start (in the hero's order, each with its level): Leadership basic, Armorer basic, Offense advanced
- anything odd: 

## E4-1b — Levels at load: a Knight with Basic Archery and 3 200 experience (D-20, D-14, GAM-SET-002, HER-LVL-006)

- Blue hero **E4-1b** (Valeska, specialty creature Archer, 3200 experience): attack 2, defense 2, power 1, knowledge 1; Basic Archery; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before anything moves: Blue's hero E4-1b (a Knight, 3 200 experience on the map, 2/2/1/1, Basic Archery) never moves. Read its seed byte and the two levels last offered (the harness, from the game's memory), then its experience, level, primary and secondary skills (its portrait's window).

**Record**

- seed byte (hero +0x8F): 245
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 3
- experience at the start: 3200
- level at the start: 4
- primary skills at the start (attack/defense/power/knowledge): 4/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Archery expert, Ballistics basic
- anything odd: 

## E4-1c — Levels at load: a Knight with Basic Navigation and 3 200 experience (D-20, D-14, GAM-SET-002, HER-LVL-006)

- Blue hero **E4-1c** (Edric, specialty creature Griffin, 3200 experience): attack 2, defense 2, power 1, knowledge 1; Basic Navigation; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before anything moves: Blue's hero E4-1c (a Knight, 3 200 experience on the map, 2/2/1/1, Basic Navigation) never moves. Read its seed byte and the two levels last offered (the harness, from the game's memory), then its experience, level, primary and secondary skills (its portrait's window).

**Record**

- seed byte (hero +0x8F): 224
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 4
- experience at the start: 3200
- level at the start: 4
- primary skills at the start (attack/defense/power/knowledge): 4/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Navigation expert, Archery basic
- anything odd: 

## E4-1d — Levels at load: a Knight with Basic Wisdom and 3 200 experience (D-20, D-14, GAM-SET-002, HER-LVL-006)

- Blue hero **E4-1d** (Sylvia, specialty skill Navigation, 3200 experience): attack 2, defense 2, power 1, knowledge 1; Basic Wisdom; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before anything moves: Blue's hero E4-1d (a Knight, 3 200 experience on the map, 2/2/1/1, Basic Wisdom) never moves. Read its seed byte and the two levels last offered (the harness, from the game's memory), then its experience, level, primary and secondary skills (its portrait's window).

**Record**

- seed byte (hero +0x8F): 19
- Wisdom last offered (hero +0x90): 3
- a school last offered (hero +0x3F): 4
- experience at the start: 3200
- level at the start: 4
- primary skills at the start (attack/defense/power/knowledge): 3/3/1/2
- secondary skills at the start (in the hero's order, each with its level): Wisdom advanced, Offense advanced
- anything odd: 

## E4-1e — Levels at load: E3-6's Death Knight again (5 000 experience, eight Basic skills) (D-20, D-14, §3.3, GAM-SET-002, HER-LVL-006)

- Blue hero **E4-1e** (Clavius, specialty resource gold, 5000 experience): attack 2, defense 3, power 1, knowledge 1; Basic Pathfinding, Basic Archery, Basic Logistics, Basic Scouting, Basic Navigation, Basic Mysticism, Basic Offense, Basic Armorer; no artifacts; no spells; army 1 Pikeman

1. Day 1, Red's turn, before anything moves: Blue's hero E4-1e (a Death Knight, 5 000 experience, 2/3/1/1, eight Basic skills) never moves. Read it as E4-1a.

**Record**

- seed byte (hero +0x8F): 105
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 5000
- level at the start: 5
- primary skills at the start (attack/defense/power/knowledge): 2/5/1/3
- secondary skills at the start (in the hero's order, each with its level): Pathfinding basic, Archery basic, Logistics advanced, Scouting basic, Navigation basic, Mysticism advanced, Offense advanced, Armorer advanced
- anything odd: 

## E4-2a — Level 10 in play: a Barbarian at 14 699 experience beats one Pikeman (D-15, D-14, HER-LVL-001)

- Red hero **E4-2a** (Yog, specialty creature Cyclops, 14699 experience): attack 4, defense 0, power 1, knowledge 1; Basic Offense, Basic Tactics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-2a's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-2a twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 147
- Wisdom last offered (hero +0x90): 6
- a school last offered (hero +0x3F): 8
- experience at the start: 14699
- level at the start: 9
- primary skills at the start (attack/defense/power/knowledge): 8/2/2/2
- secondary skills at the start (in the hero's order, each with its level): Offense basic, Tactics expert, Ballistics advanced, Armorer advanced, Logistics advanced
- dialog 1, level: 10
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Ballistics
- dialog 1, right offer: Archery
- dialog 1, taken: left (Ballistics)
- experience after: 14709
- level after: 10
- primary skills after (attack/defense/power/knowledge): 9/2/2/2
- secondary skills after (in the hero's order, each with its level): Offense basic, Tactics expert, Ballistics expert, Armorer advanced, Logistics advanced
- anything odd: 

## E4-2b — Level 10 in play: a Barbarian at 14 699 experience beats one Pikeman (D-15, D-14, HER-LVL-001)

- Red hero **E4-2b** (Jabarkas, specialty creature Orc, 14699 experience): attack 4, defense 0, power 1, knowledge 1; Basic Offense, Basic Tactics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-2b's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-2b twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 189
- Wisdom last offered (hero +0x90): 6
- a school last offered (hero +0x3F): 8
- experience at the start: 14699
- level at the start: 9
- primary skills at the start (attack/defense/power/knowledge): 8/3/1/2
- secondary skills at the start (in the hero's order, each with its level): Offense advanced, Tactics advanced, Logistics expert, Archery advanced, Artillery basic
- dialog 1, level: 10
- dialog 1, primary skill raised: power
- dialog 1, left offer (or the only one): Offense
- dialog 1, right offer: Scouting
- dialog 1, taken: left (Offense)
- experience after: 14709
- level after: 10
- primary skills after (attack/defense/power/knowledge): 8/3/2/2
- secondary skills after (in the hero's order, each with its level): Offense expert, Tactics advanced, Logistics expert, Archery advanced, Artillery basic
- anything odd: 

## E4-2c — Level 10 in play: a Barbarian at 14 699 experience beats one Pikeman (D-15, D-14, HER-LVL-001)

- Red hero **E4-2c** (Shiva, specialty creature Roc, 14699 experience): attack 4, defense 0, power 1, knowledge 1; Basic Offense, Basic Tactics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-2c's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-2c twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 30
- Wisdom last offered (hero +0x90): 6
- a school last offered (hero +0x3F): 8
- experience at the start: 14699
- level at the start: 9
- primary skills at the start (attack/defense/power/knowledge): 8/2/1/3
- secondary skills at the start (in the hero's order, each with its level): Offense expert, Tactics advanced, Logistics advanced, Estates advanced, Archery basic
- dialog 1, level: 10
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Tactics
- dialog 1, right offer: Pathfinding
- dialog 1, taken: left (Tactics)
- experience after: 14709
- level after: 10
- primary skills after (attack/defense/power/knowledge): 9/2/1/3
- secondary skills after (in the hero's order, each with its level): Offense expert, Tactics expert, Logistics advanced, Estates advanced, Archery basic
- anything odd: 

## E4-3a — Level 3 in play: an Elementalist without Wisdom at 1 999 experience (D-16, A-10, D-14, HER-LVL-004)

- Red hero **E4-3a** (Luna, specialty spell Fire Wall, 1999 experience): attack 0, defense 0, power 3, knowledge 3; Basic Leadership, Basic Logistics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-3a's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-3a twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 239
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 1999
- level at the start: 2
- primary skills at the start (attack/defense/power/knowledge): 1/0/3/3
- secondary skills at the start (in the hero's order, each with its level): Leadership advanced, Logistics basic
- dialog 1, level: 3
- dialog 1, primary skill raised: power
- dialog 1, left offer (or the only one): Logistics
- dialog 1, right offer: Fire Magic
- dialog 1, taken: left (Logistics)
- experience after: 2009
- level after: 3
- primary skills after (attack/defense/power/knowledge): 1/0/4/3
- secondary skills after (in the hero's order, each with its level): Leadership advanced, Logistics advanced
- anything odd: 

## E4-3b — Level 3 in play: an Elementalist without Wisdom at 1 999 experience (D-16, A-10, D-14, HER-LVL-004)

- Red hero **E4-3b** (Brissa, specialty spell Haste, 1999 experience): attack 0, defense 0, power 3, knowledge 3; Basic Leadership, Basic Logistics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-3b's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-3b twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 209
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 1999
- level at the start: 2
- primary skills at the start (attack/defense/power/knowledge): 0/0/4/3
- secondary skills at the start (in the hero's order, each with its level): Leadership basic, Logistics basic, Ballistics basic
- dialog 1, level: 3
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Logistics
- dialog 1, right offer: Fire Magic
- dialog 1, taken: left (Logistics)
- experience after: 2009
- level after: 3
- primary skills after (attack/defense/power/knowledge): 1/0/4/3
- secondary skills after (in the hero's order, each with its level): Leadership basic, Logistics advanced, Ballistics basic
- anything odd: 

## E4-3c — Level 3 in play: an Elementalist without Wisdom at 1 999 experience (D-16, A-10, D-14, HER-LVL-004)

- Red hero **E4-3c** (Ciele, specialty spell Magic Arrow, 1999 experience): attack 0, defense 0, power 3, knowledge 3; Basic Leadership, Basic Logistics; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-3c's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-3c twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 124
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 1999
- level at the start: 2
- primary skills at the start (attack/defense/power/knowledge): 1/0/3/3
- secondary skills at the start (in the hero's order, each with its level): Leadership advanced, Logistics basic
- dialog 1, level: 3
- dialog 1, primary skill raised: knowledge
- dialog 1, left offer (or the only one): Leadership
- dialog 1, right offer: Sorcery
- dialog 1, taken: left (Leadership)
- experience after: 2009
- level after: 3
- primary skills after (attack/defense/power/knowledge): 1/0/3/4
- secondary skills after (in the hero's order, each with its level): Leadership expert, Logistics basic
- anything odd: 

## E4-4a — Three levels from one battle: a Ranger at 999 experience beats 70 Swordsmen (§2.1, §4.1, D-14, GAM-DEC-002, GAM-BAT-008)

- Red hero **E4-4a** (Jenova, specialty resource gold, 999 experience): attack 1, defense 3, power 1, knowledge 1; Basic Archery, Basic Leadership; no artifacts; no spells; army 15 Archangel
- Opponent: 70 Swordsman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-4a's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the 70 Swordsmen east of E4-4a twice; the battle by auto combat. Answer the level-up dialogs right, left, right and note each one: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 145
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 999
- level at the start: 1
- primary skills at the start (attack/defense/power/knowledge): 1/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Archery basic, Leadership basic
- dialog 1, level: 2
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Archery
- dialog 1, right offer: Wisdom
- dialog 1, taken: right (Wisdom)
- dialog 2, level: 3
- dialog 2, primary skill raised: defense
- dialog 2, left offer (or the only one): Wisdom
- dialog 2, right offer: Offense
- dialog 2, taken: left (Wisdom)
- dialog 3, level: 4
- dialog 3, primary skill raised: attack
- dialog 3, left offer (or the only one): Archery
- dialog 3, right offer: Earth Magic
- dialog 3, taken: right (Earth Magic)
- experience after: 3449
- level after: 4
- primary skills after (attack/defense/power/knowledge): 3/4/1/1
- secondary skills after (in the hero's order, each with its level): Archery basic, Leadership basic, Wisdom advanced, Earth Magic basic
- anything odd: 

## E4-4b — Three levels from one battle: a Ranger at 999 experience beats 70 Swordsmen (§2.1, §4.1, D-14, GAM-DEC-002, GAM-BAT-008)

- Red hero **E4-4b** (Ivor, specialty creature Wood Elf, 999 experience): attack 1, defense 3, power 1, knowledge 1; Basic Archery, Basic Leadership; no artifacts; no spells; army 15 Archangel
- Opponent: 70 Swordsman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-4b's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the 70 Swordsmen east of E4-4b twice; the battle by auto combat. Answer the level-up dialogs right, left, right and note each one: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 141
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 999
- level at the start: 1
- primary skills at the start (attack/defense/power/knowledge): 1/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Archery basic, Leadership basic
- dialog 1, level: 2
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Leadership
- dialog 1, right offer: Water Magic
- dialog 1, taken: right (Water Magic)
- dialog 2, level: 3
- dialog 2, primary skill raised: defense
- dialog 2, left offer (or the only one): Leadership
- dialog 2, right offer: Artillery
- dialog 2, taken: left (Leadership)
- dialog 3, level: 4
- dialog 3, primary skill raised: defense
- dialog 3, left offer (or the only one): Leadership
- dialog 3, right offer: Pathfinding
- dialog 3, taken: right (Pathfinding)
- experience after: 3449
- level after: 4
- primary skills after (attack/defense/power/knowledge): 2/5/1/1
- secondary skills after (in the hero's order, each with its level): Archery basic, Leadership advanced, Water Magic basic, Pathfinding basic
- anything odd: 

## E4-5a — Level 4 in play: a Demoniac knowing Fire and Water Magic at 3 199 experience (D-17, D-14, HER-LVL-003)

- Blue hero **E4-5a** (Fiona, specialty creature Hell Hound, 3199 experience): attack 2, defense 2, power 1, knowledge 1; Basic Fire Magic, Basic Water Magic, Basic Logistics, Basic Offense, Basic Armorer; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn, before anything moves: read E4-5a's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-5a twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 242
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 3199
- level at the start: 3
- primary skills at the start (attack/defense/power/knowledge): 3/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Fire Magic basic, Water Magic basic, Logistics advanced, Offense basic, Armorer basic, Ballistics basic
- dialog 1, level: 4
- dialog 1, primary skill raised: defense
- dialog 1, left offer (or the only one): Fire Magic
- dialog 1, right offer: Resistance
- dialog 1, taken: left (Fire Magic)
- experience after: 3209
- level after: 4
- primary skills after (attack/defense/power/knowledge): 3/4/1/1
- secondary skills after (in the hero's order, each with its level): Fire Magic advanced, Water Magic basic, Logistics advanced, Offense basic, Armorer basic, Ballistics basic
- anything odd: 

## E4-5b — Level 4 in play: a Demoniac knowing Fire and Water Magic at 3 199 experience (D-17, D-14, HER-LVL-003)

- Blue hero **E4-5b** (Rashka, specialty creature Efreeti, 3199 experience): attack 2, defense 2, power 1, knowledge 1; Basic Fire Magic, Basic Water Magic, Basic Logistics, Basic Offense, Basic Armorer; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn, before anything moves: read E4-5b's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-5b twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 36
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 3
- experience at the start: 3199
- level at the start: 3
- primary skills at the start (attack/defense/power/knowledge): 3/2/2/1
- secondary skills at the start (in the hero's order, each with its level): Fire Magic advanced, Water Magic basic, Logistics basic, Offense basic, Armorer advanced
- dialog 1, level: 4
- dialog 1, primary skill raised: defense
- dialog 1, left offer (or the only one): Offense
- dialog 1, right offer: Archery
- dialog 1, taken: left (Offense)
- experience after: 3209
- level after: 4
- primary skills after (attack/defense/power/knowledge): 3/3/2/1
- secondary skills after (in the hero's order, each with its level): Fire Magic advanced, Water Magic basic, Logistics basic, Offense advanced, Armorer advanced
- anything odd: 

## E4-5c — Level 4 in play: a Demoniac knowing Fire and Water Magic at 3 199 experience (D-17, D-14, HER-LVL-003)

- Blue hero **E4-5c** (Marius, specialty creature Demon, 3199 experience): attack 2, defense 2, power 1, knowledge 1; Basic Fire Magic, Basic Water Magic, Basic Logistics, Basic Offense, Basic Armorer; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn, before anything moves: read E4-5c's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-5c twice; the battle by auto combat. Answer the level-up dialog left and note it: the level, the primary skill it raises, the skill offered left and right (or the one skill) and the one taken.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 6
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 0
- experience at the start: 3199
- level at the start: 3
- primary skills at the start (attack/defense/power/knowledge): 2/3/1/2
- secondary skills at the start (in the hero's order, each with its level): Fire Magic basic, Water Magic basic, Logistics expert, Offense basic, Armorer basic
- dialog 1, level: 4
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Water Magic
- dialog 1, right offer: First Aid
- dialog 1, taken: left (Water Magic)
- experience after: 3209
- level after: 4
- primary skills after (attack/defense/power/knowledge): 3/3/1/2
- secondary skills after (in the hero's order, each with its level): Fire Magic basic, Water Magic advanced, Logistics expert, Offense basic, Armorer basic
- anything odd: 
