# E08 batch E7a — results

> Played by the capture harness on 2026-10-27 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-7a.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

Map file: `E08-batch-7a.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `48a36c8c`.** The map's description in the scenario list starts with "Stamp 48a36c8c.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-7a.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. Most records are the game's memory: an object's word on its tile, a hero's experience, primary and secondary skills, spell points, morale and luck bonuses (+0x11A, +0x11B) and status flags, its worn positions (+0x12D, 8 bytes each), its backpack in order (+0x1D4, the count +0x3D4), its learned spells (+0x3EA), its army (creature ids +0x91, counts +0xAD), the players' treasuries, the game's Universities (+0x4E3CC). The capture harness reads them (`clients/capture/batches/E08-batch-7a.toml`, E08 T57). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Positions are the original's numbers: head 0, shoulders 1, neck 2, right hand 3, left hand 4, torso 5, rings 6 and 7, feet 8, misc 9 to 12 (the hero screen's misc 1 to 4) and 18 (misc 5), war machines 13 to 16, spell book 17. Army slots count from 0 at the left. Tiles are (column, row) from 0 at the top left.
6. Answer questions as the encounter says; write each question's or message's text (or its first words) down where the encounter asks for it. A level-up: take the offer the encounter names, else the left one.
7. Each battle by auto combat unless the encounter says otherwise; a tactics phase is ended at once (its Start button).
8. At the start (save 1) read every chest's, Corpse's and Wagon's word: if none of E7-5's forty chests (26, 4) to (44, 10), none of E7-10's Corpses or none of its Wagons holds an artifact, restart the scenario (a new game draws again) and save again. Rows E7-3, E7-8 and E7-9 read the map of the game that is played; nothing is played for them.
9. Order: day 1, Red's turn: E7-1, E7-2, E7-4, E7-6, E7-5 (a hero out of movement points goes on the next day); end the turn. Blue's turn (save 2 at its start): E7-10, E7-11. Further days only for E7-4, E7-5 and E7-10 if their heroes need them. Then the map is done.
10. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E7-1 — A guarded artifact without a text: a question or the battle (D-29, ADV-PIK-007, D-30)

- Red hero **E7-1** (Sylvia, specialty skill Navigation): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Angel

1. Day 1, Red's turn: E7-1 stands at (4, 4) wearing and carrying nothing. A Centaurs Axe lies at (7, 4), guarded by 10 Pikemen; its message record has no text.
2. Click the axe once (the path), then once more: the hero walks to (6, 4).
3. Note what follows the step: a question (its text or first words) or the battle at once. A question: answer **No**.
4. After No: note whether the axe still lies at (7, 4) and read E7-1's worn positions and backpack. Click the axe again and answer **Yes**: the battle, auto combat.
5. After the battle: read E7-1's army, worn positions and backpack, and whether the axe still lies at (7, 4).

**Record**

- after the step (the question's text, or: the battle at once): GENRLTXT:421; no
- after No: the axe on (7, 4) (yes or no); E7-1's artifacts: Centaurs Axe; 9: Spyglass, 16: Catapult; none
- the battle's result; E7-1's army after: GENRLTXT:421; yes; attacker; 5 Angels
- worn positions after (position: artifact): 3: Centaurs Axe, 9: Spyglass, 16: Catapult
- backpack after: none
- the axe on (7, 4) after (yes or no): none
- anything odd: messages: GENRLTXT:7, ADVEVENT:8

## E7-2 — Where a pickup's artifact goes: a Centaurs Axe, then a Spell Scroll (D-30, ADV-PIK-006, ADV-PIK-008, HER-CHG-002)

- Red hero **E7-2** (Sorsha, specialty creature Swordsman): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 2 Angel

1. Day 1, Red's turn: E7-2 stands at (4, 8) wearing and carrying nothing (no Spell Book). A Centaurs Axe lies at (7, 8), a Spell Scroll (Magic Arrow) at (10, 8), neither guarded.
2. Click the axe twice; note the message. Read E7-2's worn positions and backpack.
3. Click the scroll twice; note the message. Read E7-2's worn positions, backpack and learned spells.

**Record**

- the axe's message (first words): GENRLTXT:16; GENRLTXT:16: Centaurs Axe
- worn positions after the axe (position: artifact): 3: Centaurs Axe, 16: Catapult
- backpack after the axe: none
- the scroll's message (first words): ADVEVENT:135; ADVEVENT:135: Magic Arrow
- worn positions after the scroll: 3: Centaurs Axe, 9: Spell Scroll, 16: Catapult
- backpack after the scroll: none
- learned spells after the scroll: none
- anything odd: 

## E7-3 — Forty random artifacts (class 65): their artifacts at the new game (D-31, D-32, ADV-OBJ-006)

- Red hero **E7-3** (Valeska, specialty creature Archer): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1), before anything is played: forty random artifacts (class 65, any class) stand at (4, 40) to (13, 43), ten to a row. Read each one's word on its tile (bits 0 to 9: the artifact).
2. E7-3's hero stands at (2, 44) only to name the row; it plays nothing.

**Record**

| tile | the word (hex) | the artifact |
|---|---|---|
| (4, 40) | 0x00000000 | Red Dragon Flame Tongue |
| (5, 40) | 0x00000000 | Ladybird of Luck |
| (6, 40) | 0x00000000 | Dragon Scale Shield |
| (7, 40) | 0x00000000 | Vial of Lifeblood |
| (8, 40) | 0x00000000 | Arms of Legion |
| (9, 40) | 0x00000000 | Garniture of Interference |
| (10, 40) | 0x00000000 | Equestrian's Gloves |
| (11, 40) | 0x00000000 | Crest of Valor |
| (12, 40) | 0x00000000 | Mystic Orb of Mana |
| (13, 40) | 0x00000000 | Greater Gnoll's Flail |
| (4, 41) | 0x00000000 | Bow of Elven Cherrywood |
| (5, 41) | 0x00000000 | Ring of Life |
| (6, 41) | 0x00000000 | Ring of Conjuring |
| (7, 41) | 0x00000000 | Necklace of Swiftness |
| (8, 41) | 0x00000000 | Bird of Perception |
| (9, 41) | 0x00000000 | Pendant of Total Recall |
| (10, 41) | 0x00000000 | Collar of Conjuring |
| (11, 41) | 0x00000000 | Rib Cage |
| (12, 41) | 0x00000000 | Tunic of the Cyclops King |
| (13, 41) | 0x00000000 | Loins of Legion |
| (4, 42) | 0x00000000 | Endless Bag of Gold |
| (5, 42) | 0x00000000 | Helm of the Alabaster Unicorn |
| (6, 42) | 0x00000000 | Spirit of Oppression |
| (7, 42) | 0x00000000 | Eversmoking Ring of Sulfur |
| (8, 42) | 0x00000000 | Pendant of Dispassion |
| (9, 42) | 0x00000000 | Orb of Driving Rain |
| (10, 42) | 0x00000000 | Shield of the Yawning Dead |
| (11, 42) | 0x00000000 | Cape of Velocity |
| (12, 42) | 0x00000000 | Crown of the Supreme Magi |
| (13, 42) | 0x00000000 | Ring of Vitality |
| (4, 43) | 0x00000000 | Ring of Infinite Gems |
| (5, 43) | 0x00000000 | Golden Bow |
| (6, 43) | 0x00000000 | Orb of Silt |
| (7, 43) | 0x00000000 | Ogre's Club of Havoc |
| (8, 43) | 0x00000000 | Hellstorm Helmet |
| (9, 43) | 0x00000000 | Ring of the Wayfarer |
| (10, 43) | 0x00000000 | Sphere of Permanence |
| (11, 43) | 0x00000000 | Recanter's Cloak |
| (12, 43) | 0x00000000 | Breastplate of Brimstone |
| (13, 43) | 0x00000000 | Orb of the Firmament |

- anything odd: 

## E7-4 — Six chests and a Learning Stone with Basic Learning: the experience a choice gives (ADV-PIK-005, ADV-TCH-003, HER-EXP-001)

- Red hero **E7-4** (Caitlin, specialty resource gold): attack 1, defense 1, power 1, knowledge 1; Basic Learning, Expert Logistics; no artifacts; no spells; army 3 Angel

1. Day 1, Red's turn: E7-4 stands at (4, 12) with Basic Learning (and no Learning specialty) and Expert Logistics (for the points). Six chests stand at (7, 12) to (17, 12), every second tile; a Learning Stone at (18, 12). Read each chest's word first (bit 10: an artifact; bits 11 to 14: the gold in 500s). A hero out of movement points goes on the next day.
2. For each chest from west to east: read E7-4's experience, click the chest twice, note the two pictures' values (gold and experience; or the artifact) and **take the experience** (the right picture); read the experience again. A level-up between: take the offer that is **not Learning** (the left one if neither is).
3. Then the Learning Stone: read the experience, click it twice, note the message, read the experience again (a level-up as above).
4. Read E7-4's secondary skills at the end (Learning must still be Basic).

**Record**

| chest | its word (hex) | gold shown | experience shown | experience before | experience after |
|---|---|---|---|---|---|
| (7, 12) | 0xffff93ff; 1000 gold | 1000 gold | 525 experience | 0 | 525 |
| (9, 12) | 0xffff9bff; 1500 gold | 1500 gold | 1050 experience | 525 | 1575 |
| (11, 12) | 0xffff9bff; 1500 gold | 1500 gold | 1050 experience | 1575 | 2625 |
| (13, 12) | 0xffff93ff; 1000 gold | 1000 gold | 525 experience | 2625 | 3150 |
| (15, 12) | 0xffffa3ff; 2000 gold | 2000 gold | 1575 experience | 3150 | 4725 |
| (17, 12) | 0xffffa3ff; 2000 gold | 2000 gold | 1575 experience | 4725 | 6300 |

- the Learning Stone: experience before, after, its message: 6300; 7350; ADVEVENT:143
- level-ups (level: the two offers, the one taken): level 2: Learning or Mysticism, took right (Mysticism); level 3: Learning or Wisdom, took right (Wisdom); level 4: Mysticism or Earth Magic, took left (Mysticism); level 5: Learning or First Aid, took right (First Aid); level 6: Wisdom or Archery, took left (Wisdom)
- secondary skills at the end: Learning basic, Logistics expert, Mysticism advanced, Wisdom advanced, First Aid basic
- anything odd: 

## E7-5 — Chests with a full backpack: an artifact chest's offer (ADV-PIK-005)

- Red hero **E7-5** (Edric, specialty creature Griffin): attack 1, defense 1, power 1, knowledge 1; Expert Logistics; the backpack filled to 64 with Centaurs Axe; no spells; army 3 Angel

1. Day 1, Red's turn (after E7-6): E7-5 stands at (24, 6) with 64 Centaurs Axes in its backpack, nothing worn, and Expert Logistics (for the points: without it E08 T57's third run took E7-5 and E7-10 into a sixth day, where it stopped). Forty chests stand at (26, 4) to (44, 10), every second column and row. Read every chest's word at the start (save 1): bit 10 marks an artifact (bits 0 to 9 the artifact), else bits 11 to 14 the gold in 500s.
2. Visit **every chest whose word holds an artifact**, then the three nearest gold chests. At each: read E7-5's backpack count and Red's gold, click the chest twice, note what is shown (the pictures' values, or the artifact) and take the **gold** (the left picture, or OK); read the backpack count and Red's gold again. A hero out of movement points goes on the next day.

**Record**

| chest | its word (hex) | what is shown | backpack before / after | gold before / after |
|---|---|---|---|---|
| (26, 4) | 0xffff9bff; 1500 gold |   |   |   |
| (28, 4) | 0xffffa3ff; 2000 gold |   |   |   |
| (30, 4) | 0xffffa3ff; 2000 gold |   |   |   |
| (32, 4) | 0xffffa3ff; 2000 gold |   |   |   |
| (34, 4) | 0xffff93ff; 1000 gold |   |   |   |
| (36, 4) | 0xffff9bff; 1500 gold |   |   |   |
| (38, 4) | 0xffff93ff; 1000 gold | 1000 gold or 500 experience: left; none | 64; 64 | 39350; 40350 |
| (40, 4) | 0xffffa3ff; 2000 gold | 2000 gold or 1500 experience: left; none | 64; 64 | 37350; 39350 |
| (42, 4) | 0xffffa3ff; 2000 gold | 2000 gold or 1500 experience: left; none | 64; 64 | 35350; 37350 |
| (44, 4) | 0xfffffc40; artifact Stoic Watchman | 1000 gold or 500 experience: left; none | 64; 64 | 34350; 35350 |
| (26, 6) | 0xffffa3ff; 2000 gold |   |   |   |
| (28, 6) | 0xfffffc25; artifact Quiet Eye of the Dragon | 1000 gold or 500 experience: left; none | 64; 64 | 31000; 32000 |
| (30, 6) | 0xffff93ff; 1000 gold |   |   |   |
| (32, 6) | 0xffffa3ff; 2000 gold |   |   |   |
| (34, 6) | 0xffff93ff; 1000 gold |   |   |   |
| (36, 6) | 0xffff93ff; 1000 gold |   |   |   |
| (38, 6) | 0xffffa3ff; 2000 gold |   |   |   |
| (40, 6) | 0xffffa3ff; 2000 gold |   |   |   |
| (42, 6) | 0xfffffc2e; artifact Clover of Fortune | 1000 gold or 500 experience: left; none | 64; 64 | 33000; 34000 |
| (44, 6) | 0xffffa3ff; 2000 gold |   |   |   |
| (26, 8) | 0xffffa3ff; 2000 gold |   |   |   |
| (28, 8) | 0xffffa3ff; 2000 gold |   |   |   |
| (30, 8) | 0xffffa3ff; 2000 gold |   |   |   |
| (32, 8) | 0xffff93ff; 1000 gold |   |   |   |
| (34, 8) | 0xfffffc55; artifact Hourglass of the Evil Hour | 1000 gold or 500 experience: left; none | 64; 64 | 32000; 33000 |
| (36, 8) | 0xffff93ff; 1000 gold |   |   |   |
| (38, 8) | 0xffff9bff; 1500 gold |   |   |   |
| (40, 8) | 0xffff93ff; 1000 gold |   |   |   |
| (42, 8) | 0xffff9bff; 1500 gold |   |   |   |
| (44, 8) | 0xffff9bff; 1500 gold |   |   |   |
| (26, 10) | 0xffff93ff; 1000 gold |   |   |   |
| (28, 10) | 0xffffa3ff; 2000 gold |   |   |   |
| (30, 10) | 0xffffa3ff; 2000 gold |   |   |   |
| (32, 10) | 0xffffa3ff; 2000 gold |   |   |   |
| (34, 10) | 0xffff93ff; 1000 gold |   |   |   |
| (36, 10) | 0xffffa3ff; 2000 gold |   |   |   |
| (38, 10) | 0xffff93ff; 1000 gold |   |   |   |
| (40, 10) | 0xffffa3ff; 2000 gold |   |   |   |
| (42, 10) | 0xffff9bff; 1500 gold |   |   |   |
| (44, 10) | 0xffffa3ff; 2000 gold |   |   |   |

- anything odd: the walk to [44, 4] waited 1 day(s) before its step: beyond the day's points

## E7-6 — A monster's treasure artifact with a full backpack (ADV-MON-011)

- Red hero **E7-6** (Tyris, specialty creature Cavalier): attack 2, defense 2, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 5 Angel
- Opponent: 10 Pikeman (wandering monster, never flees, its treasure Centaurs Axe)

1. Day 1, Red's turn: E7-6 stands at (4, 16) with 64 Centaurs Axes in its backpack, nothing worn. Ten savage Pikemen east of it hold a treasure: 1 000 gold and a Centaurs Axe.
2. Read E7-6's backpack count and worn positions and Red's gold. Attack the Pikemen (auto combat).
3. Note every message after the battle (first words). Read E7-6's backpack count, worn positions and Red's gold.

**Record**

- before: backpack count, worn positions, gold: 64; 16: Catapult; 30000
- the messages after the battle (first words, in order): ADVEVENT:2, ?; ?: 1000 gold
- after: backpack count, worn positions, gold: 64; 16: Catapult; 31000
- anything odd: battle: attacker

## E7-8 — Fifteen random Shrines of the first circle: their spells (D-34, ADV-TCH-004)

- Red hero **E7-8** (Christian, specialty creature Ballista): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1): fifteen Shrines of the first circle stand at (4, 48) to (32, 48), every second column, their spells drawn at the new game. Read each one's word on its tile (bits 13 to 22: the spell).
2. E7-8's hero stands at (2, 50) only to name the row; it plays nothing.

**Record**

| shrine | the word (hex) | the spell |
|---|---|---|
| (4, 48) | 0xff81e01f | Magic Arrow |
| (6, 48) | 0xff80601f | View Earth |
| (8, 48) | 0xff83601f | Shield |
| (10, 48) | 0xff84a01f | Cure |
| (12, 48) | 0xff86c01f | Slow |
| (14, 48) | 0xff85401f | Curse |
| (16, 48) | 0xff86a01f | Haste |
| (18, 48) | 0xff85201f | Bless |
| (20, 48) | 0xff85c01f | Stone Skin |
| (22, 48) | 0xff83e01f | Protection from Fire |
| (24, 48) | 0xff85601f | Bloodlust |
| (26, 48) | 0xff84601f | Dispel |
| (28, 48) | 0xff84001f | Prot. from Water |
| (30, 48) | 0xff80001f | Summon Boat |
| (32, 48) | 0xff80a01f | View Air |

- anything odd: 

## E7-9 — Thirty Universities: Necromancy among their skills (D-36, ADV-TCH-007)

- Red hero **E7-9** (Ingham, specialty creature Monk): attack 0, defense 0, power 0, knowledge 0; no skills; no artifacts; no spells; army 1 Pikeman

1. At the start (save 1): thirty Universities stand with their bottom-right corners at (40, 40) to (65, 56), six to a row 5 columns apart, the rows 4 apart. Read the game's Universities (+0x4E3CC): each one's four skills.
2. E7-9's hero stands at (36, 58) only to name the row; it plays nothing.

**Record**

| University | its four skills |
|---|---|
| (40, 40) | Artillery, Scouting, Wisdom, Mysticism |
| (45, 40) | Fire Magic, Scholar, Pathfinding, Learning |
| (50, 40) | Ballistics, First Aid, Estates, Intelligence |
| (55, 40) | Ballistics, Tactics, Scholar, Learning |
| (60, 40) | Scholar, Mysticism, Resistance, Fire Magic |
| (65, 40) | Intelligence, Sorcery, Ballistics, Necromancy |
| (40, 44) | Navigation, Estates, Eagle Eye, Learning |
| (45, 44) | Armorer, Leadership, Resistance, Scouting |
| (50, 44) | Necromancy, Archery, Resistance, Fire Magic |
| (55, 44) | Air Magic, Resistance, Luck, Fire Magic |
| (60, 44) | Earth Magic, Fire Magic, Water Magic, Pathfinding |
| (65, 44) | Resistance, Earth Magic, Necromancy, Pathfinding |
| (40, 48) | Pathfinding, Ballistics, Resistance, Eagle Eye |
| (45, 48) | Diplomacy, Eagle Eye, Estates, Intelligence |
| (50, 48) | Artillery, Diplomacy, Armorer, Mysticism |
| (55, 48) | Logistics, Diplomacy, Resistance, Scouting |
| (60, 48) | Estates, Tactics, Diplomacy, Wisdom |
| (65, 48) | Armorer, Earth Magic, Sorcery, Ballistics |
| (40, 52) | Wisdom, Luck, Resistance, Air Magic |
| (45, 52) | Intelligence, Resistance, Scouting, Learning |
| (50, 52) | Navigation, Estates, Ballistics, Fire Magic |
| (55, 52) | First Aid, Luck, Navigation, Scouting |
| (60, 52) | Pathfinding, Air Magic, Mysticism, Navigation |
| (65, 52) | Estates, Scouting, Diplomacy, Ballistics |
| (40, 56) | Armorer, First Aid, Scholar, Intelligence |
| (45, 56) | Earth Magic, First Aid, Artillery, Necromancy |
| (50, 56) | Earth Magic, Offense, Diplomacy, Water Magic |
| (55, 56) | Scouting, Resistance, Diplomacy, Armorer |
| (60, 56) | Resistance, Tactics, Archery, Armorer |
| (65, 56) | Scholar, Resistance, Fire Magic, Water Magic |

- how many offer Necromancy: 4 of 30
- anything odd: 

## E7-10 — Corpses, Wagons and Warrior's Tombs with a full backpack (ADV-PIK-009, ADV-PIK-010, ADV-PIK-011)

- Blue hero **E7-10** (Rissa, specialty resource mercury): attack 1, defense 1, power 1, knowledge 1; Expert Logistics; the backpack filled to 64 with Centaurs Axe; no spells; army 3 Angel

1. Blue's turn (save 2 at its start): E7-10 stands at (24, 18) with 64 Centaurs Axes in its backpack, nothing worn, and Expert Logistics (for the points, as E7-5). Twenty Corpses stand at (26, 16) to (44, 18), every second column and row; ten Wagons at (26, 22) to (44, 22), every second column; two Warrior's Tombs with their bottom-right corners at (48, 16) and (51, 16). Read every Corpse's, Wagon's and Tomb's word at the start (save 1).
2. Visit **every Corpse and every Wagon whose word holds an artifact**, then one Corpse and one Wagon without one. At each: read Blue's treasury and E7-10's backpack count, click it twice, note the message (first words), read the treasury, the backpack count and its word again. A hero out of movement points goes on the next day.
3. Then both Tombs: at each, read E7-10's morale bonus (+0x11A) and backpack count, click it twice, answer the question **Yes**, note the messages, read the morale bonus, the backpack count and the Tomb's word again.

**Record**

| Corpse | its word before | message | treasury before / after | backpack before / after | its word after |
|---|---|---|---|---|---|
| (26, 16) | 0xfffeffe0; empty |   |   |   |   |
| (28, 16) | 0xffff07e1; artifact Armor of Wonder | ?; ?: 1000 gold | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 30500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 31500 gold | 64; 64 | 0xfffeffe1; empty |
| (30, 16) | 0xffff0de2; artifact Vampire's Cowl | ?; ?: 1000 gold | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 31500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 32500 gold | 64; 64 | 0xfffeffe2; empty |
| (32, 16) | 0xfffeffe3; empty |   |   |   |   |
| (34, 16) | 0xfffeffe4; empty |   |   |   |   |
| (36, 16) | 0xfffeffe5; empty |   |   |   |   |
| (38, 16) | 0xfffeffe6; empty |   |   |   |   |
| (40, 16) | 0xfffeffe7; empty | ADVEVENT:38; none | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 35500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 35500 gold | 64; 64 | 0xfffeffe7; empty |
| (42, 16) | 0xfffeffe8; empty |   |   |   |   |
| (44, 16) | 0xfffeffe9; empty |   |   |   |   |
| (26, 18) | 0xfffeffea; empty |   |   |   |   |
| (28, 18) | 0xfffeffeb; empty |   |   |   |   |
| (30, 18) | 0xfffeffec; empty |   |   |   |   |
| (32, 18) | 0xfffeffed; empty |   |   |   |   |
| (34, 18) | 0xfffeffee; empty |   |   |   |   |
| (36, 18) | 0xffff03ef; artifact Buckler of the Gnoll King | ?; ?: 1000 gold | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 32500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 33500 gold | 64; 64 | 0xfffeffef; empty |
| (38, 18) | 0xfffefff0; empty |   |   |   |   |
| (40, 18) | 0xffff0cf1; artifact Glyph of Gallantry | ?; ?: 1000 gold | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 33500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 34500 gold | 64; 64 | 0xfffefff1; empty |
| (42, 18) | 0xffff0372; artifact Shield of the Dwarven Lords | ?; ?: 1000 gold | 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 34500 gold; 30 wood, 16 mercury, 30 ore, 16 sulfur, 15 crystal, 15 gems, 35500 gold | 64; 64 | 0xfffefff2; empty |
| (44, 18) | 0xfffefff3; empty |   |   |   |   |

| Wagon | its word before | message | treasury before / after | backpack before / after | its word after |
|---|---|---|---|---|---|
| (26, 22) | 0xe7ffa005; 5 sulfur | ADVEVENT:154; ADVEVENT:154: 5 sulfur | 30 wood, 19 mercury, 33 ore, 19 sulfur, 19 crystal, 19 gems, 35500 gold; 30 wood, 19 mercury, 33 ore, 24 sulfur, 19 crystal, 19 gems, 35500 gold | 64; 64 | 0xe7ff8045; empty |
| (28, 22) | 0xe61a6002; artifact Speculum | ADVEVENT:154; ADVEVENT:154: 2 sulfur | 30 wood, 19 mercury, 33 ore, 17 sulfur, 19 crystal, 19 gems, 35500 gold; 30 wood, 19 mercury, 33 ore, 19 sulfur, 19 crystal, 19 gems, 35500 gold | 64; 64 | 0xe61a4042; empty |
| (30, 22) | 0xe80ce004; artifact Breastplate of Petrified Wood | ADVEVENT:154; ADVEVENT:154: 4 crystal | 30 wood, 19 mercury, 33 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold; 30 wood, 19 mercury, 33 ore, 17 sulfur, 19 crystal, 19 gems, 35500 gold | 64; 64 | 0xe80cc044; empty |
| (32, 22) | 0xe2396002; artifact Inexhaustible Cart of Lumber | ADVEVENT:154; ADVEVENT:154: 2 mercury | 30 wood, 17 mercury, 33 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold; 30 wood, 19 mercury, 33 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold | 64; 64 | 0xe2394042; empty |
| (34, 22) | 0xe4156003; artifact Dragon Wing Tabard | ADVEVENT:154; ADVEVENT:154: 3 ore | 30 wood, 17 mercury, 30 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold; 30 wood, 17 mercury, 33 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold | 64; 64 | 0xe4154043; empty |
| (36, 22) | 0xe5ffa005; 5 ore |   |   |   |   |
| (38, 22) | 0xe5ffa003; 3 ore |   |   |   |   |
| (40, 22) | 0xe5ffa005; 5 ore |   |   |   |   |
| (42, 22) | 0xe3ffa003; 3 mercury |   |   |   |   |
| (44, 22) | 0xea34e004; artifact Pendant of Free Will | ADVEVENT:154; ADVEVENT:154: 4 gems | 30 wood, 17 mercury, 30 ore, 17 sulfur, 15 crystal, 15 gems, 35500 gold; 30 wood, 17 mercury, 30 ore, 17 sulfur, 15 crystal, 19 gems, 35500 gold | 64; 64 | 0xea34c044; empty |

| Tomb | its word before | the question | messages | morale bonus before / after | backpack before / after | its word after |
|---|---|---|---|---|---|---|
| (48, 16) | 0xff8f001f; artifact Torso of Legion | ADVEVENT:161 | ADVEVENT:163; ADVEVENT:163: morale (frame 2), morale (frame 2), morale (frame 2) | 0; -3 | 64; 64 | 0xff8f005f; artifact Torso of Legion |
| (51, 16) | 0xff83601f; artifact Scales of the Greater Basilisk | ADVEVENT:161 | ADVEVENT:163; ADVEVENT:163: morale (frame 2), morale (frame 2), morale (frame 2) | -3; -3 | 64; 64 | 0xff83605f; artifact Scales of the Greater Basilisk |

- anything odd: the walk to [44, 22] waited 1 day(s) before its step: beyond the day's points; the walk to [48, 16] waited 2 day(s) before its step: beyond the day's points

## E7-11 — Two Warrior's Tombs, then a battle: the morale bonus (ADV-PIK-011, HER-ADV-008, GAM-BAT-014)

- Blue hero **E7-11** (Calid, specialty resource sulfur): attack 1, defense 1, power 1, knowledge 1; no skills; the backpack filled to 64 with Centaurs Axe; no spells; army 20 Pikeman, 10 Centaur
- Opponent: 5 Pikeman (wandering monster, never flees)

1. Blue's turn, after E7-10: E7-11 stands at (24, 28) with Pikemen and Centaurs (two factions: no morale from its army's mix beyond 0), no skills, nothing worn and 64 Centaurs Axes in its backpack (a Tomb's artifact stays in the Tomb, so no artifact changes the morale). Five savage Pikemen stand east of it at (25, 28). Two Warrior's Tombs stand west, their bottom-right corners at (21, 28) and (21, 31); a hero enters a Tomb's tile.
2. Read E7-11's morale bonus (+0x11A) and status flags. Walk west (away from the Pikemen) and visit the first Tomb (click it twice, answer **Yes**); note the messages; read the morale bonus and status flags.
3. Visit the second Tomb the same way; read the morale bonus and status flags.
4. Attack the Pikemen. At the battle's opening read the stacks' morale (the capture harness's `opening.py`, or the creature windows); then auto combat.
5. After the battle: read E7-11's morale bonus and status flags.

**Record**

- before: morale bonus, status flags (hex): 0; 0x0
- first Tomb: messages; morale bonus, status flags after: ADVEVENT:161; ADVEVENT:163; -3; 0x200000
- second Tomb: messages; morale bonus, status flags after: ADVEVENT:161; ADVEVENT:163; -3; 0x200000
- the opening: each of E7-11's stacks' morale: 20 Pikemen -3, 10 Centaurs -3
- after the battle: morale bonus, status flags: 0; 0x0
- anything odd: battle: attacker; the opponent's morale at the opening: 2 Pikemen 1, 2 Pikemen 1, 1 Pikeman 1
