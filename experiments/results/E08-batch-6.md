# E08 batch E6 — results

> Played by the capture harness on 2026-10-23 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-6.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.

> Run 3 of 3 (E08 T47b2): the three runs recorded every field the same; run 1's recipe did not yet expect E6-7's message and noted it under "anything odd" (the recipe expects it since). Two lines were filled by the agent from this run, not by `fill`: E6-7's message, whose text `h3lod` does not name (it is composed of several records), from the dialog's text items as the run read them; and E6-8's screen line, from a dump of the screen taken at the read after the assembly. The dumps at E6-9's reads also show the Spell Book in its slot with the Thunder on and after it is off, and locks on the head, torso and left hand while the Thunder is worn (no picture committed).

Map file: `E08-batch-6.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `e17cb77c`.** The map's description in the scenario list starts with "Stamp e17cb77c.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-6.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. If the scenario's options let you, set both players' starting bonus to gold. Before you start it, compare the stamp at the start of the map's description with the stamp above. Save right away (save 1).
4. A hero's army (creature ids +0x91, counts +0xAD), its worn positions (+0x12D, 8 bytes each), its backpack in order (+0x1D4, the count +0x3D4), its learned spells (+0x3EA), its flags (+0x48: bit 0 tight, bit 1 the tactics formation) and movement points, and the battle manager's tactics phase (+0x13D68) show in the game's memory: the capture harness reads them (clients/capture/batches/E08-batch-6.toml). Played by hand, read the screens instead and leave the memory-only lines empty.
5. Positions are the original's numbers: head 0, shoulders 1, neck 2, right hand 3, left hand 4, torso 5, rings 6 and 7, feet 8, misc 9 to 12 (the hero screen's misc 1 to 4) and 18 (misc 5), war machines 13 to 16, spell book 17. Army slots count from 0 at the left.
6. Answer questions as the encounter says; write each question's or message's text (or its first words) down where the encounter asks for it.
7. Each battle by auto combat; a tactics phase is ended at once (its Start button) before the auto combat.
8. Order: day 1, Red's turn: E6-1, E6-2, E6-3, E6-4, E6-11; end the turn. Blue's turn (save 2 at its start): E6-6a, E6-6b, E6-7, E6-8, E6-10 (with E6-9 on the same hero). Then the map is done.
9. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E6-1 — The army dialog: a full army joined by a friendly group, OK without a change (ADV-MON-008, GAM-DEC-004, ADV-STUB-2)

- Red hero **E6-1** (Christian, specialty creature Ballista): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Archer, 5 Griffin, 5 Swordsman, 3 Monk, 2 Cavalier, 1 Angel
- Opponent: 10 Wolf Rider (wandering monster, compliant, never flees)

1. Day 1, Red's turn: read E6-1's army and Red's gold. The 10 Wolf Riders east of E6-1 are compliant: they always offer to join.
2. Click the Wolf Riders twice. Note the question (its first words) and answer **Yes**.
3. If a dialog with two armies opens (the group above, E6-1's army below): note its title, change nothing and press **OK**.
4. Read E6-1's army, Red's gold, and whether the Wolf Riders still stand at (4, 3).

**Record**

- army before (slot: creature count): 0: 10 Pikemen, 1: 10 Archers, 2: 5 Griffins, 3: 5 Swordsmen, 4: 3 Monks, 5: 2 Cavaliers, 6: 1 Angel
- gold before: 30000
- the question: ADVEVENT:86
- the dialog's title (or: no dialog): GENRLTXT:36
- army after: 0: 10 Pikemen, 1: 10 Archers, 2: 5 Griffins, 3: 5 Swordsmen, 4: 3 Monks, 5: 2 Cavaliers, 6: 1 Angel
- gold after: 30000
- Wolf Riders on (4, 3) after (count, or none): none
- anything odd: 

## E6-2 — The army dialog: room made by dismissing a stack, the group taken into the freed slot (ADV-MON-008, GAM-DEC-004, ADV-STUB-2)

- Red hero **E6-2** (Torosar, specialty creature Ballista): attack 2, defense 2, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Archer, 5 Griffin, 5 Swordsman, 3 Monk, 2 Cavalier, 1 Angel
- Opponent: 10 Wolf Rider (wandering monster, compliant, never flees)

1. Day 1, Red's turn, after E6-1: read E6-2's army. The 10 Wolf Riders east of it are compliant.
2. Click the Wolf Riders twice; answer the question **Yes**.
3. In the dialog: click E6-2's Swordsmen (slot 3, the lower bar) once, then once more: the creature's window opens. Press its dismiss button; note the question and answer **Yes**.
4. Click the Wolf Riders in the upper bar, then the emptied slot 3 in the lower bar. Press **OK**.
5. Read E6-2's army and whether the Wolf Riders still stand at (10, 3).

**Record**

- army before: 0: 10 Pikemen, 1: 10 Archers, 2: 5 Griffins, 3: 5 Swordsmen, 4: 3 Monks, 5: 2 Cavaliers, 6: 1 Angel
- the dismiss question: GENRLTXT:13
- army after: 0: 10 Pikemen, 1: 10 Archers, 2: 5 Griffins, 3: 10 Wolf Riders, 4: 3 Monks, 5: 2 Cavaliers, 6: 1 Angel
- Wolf Riders on (10, 3) after (count, or none): none
- anything odd: 

## E6-3 — A hero's last stack in the exchange: a move and a merge refused, a swap allowed (ADV-ARM-001, GAM-MET-001)

- Red hero **E6-3** (Fiona, specialty creature Hell Hound): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **E6-3 B** (Marius, specialty creature Demon): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 5 Pikeman, 5 Archer

1. Day 1, Red's turn, after E6-2: E6-3 (10 Pikemen) stands west of E6-3 B (5 Pikemen in slot 0, 5 Archers in slot 1). Click E6-3 B twice: the exchange screen opens (E6-3 left, E6-3 B right).
2. Click E6-3's Pikemen, then E6-3 B's empty slot 2. Note any message; read both armies.
3. Click E6-3's Pikemen, then E6-3 B's Pikemen (slot 0). Note any message; read both armies.
4. Click E6-3's Pikemen, then E6-3 B's Archers (slot 1). Note any message; read both armies. Close the exchange.

**Record**

| click | message | E6-3's army after | E6-3 B's army after |
|---|---|---|---|
| to the empty slot 2 | none | 0: 10 Pikemen | 0: 5 Pikemen, 1: 5 Archers |
| onto the Pikemen (slot 0) | none | 0: 10 Pikemen | 0: 5 Pikemen, 1: 5 Archers |
| onto the Archers (slot 1) | none | 0: 5 Archers | 0: 5 Pikemen, 1: 10 Pikemen |

- anything odd: 

## E6-4 — The split dialog into the other hero's empty slot when that hero has one stack (A-16) (A-16, D-27, ADV-ARM-002)

- Red hero **E6-4** (Ignatius, specialty creature Imp): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman, 10 Archer
- Red hero **E6-4 B** (Calh, specialty creature Gog): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Swordsman

1. Day 1, Red's turn, after E6-3: E6-4 (10 Pikemen in slot 0, 10 Archers in slot 1) stands west of E6-4 B (10 Swordsmen in slot 0). Click E6-4 B twice: the exchange screen opens.
2. Click E6-4's Pikemen, press the split button (or hold Shift), then click E6-4 B's empty slot 1: the split dialog opens. Note the two counts it shows at the start.
3. Move the slider to its maximum (all the way toward E6-4 B); note the two counts it shows there; press **OK**.
4. Read both armies. Close the exchange.

**Record**

- the dialog's counts at the start (E6-4 : E6-4 B): 10 : 0
- the dialog's counts at the maximum: 0 : 10
- E6-4's army after: 0: 1 Pikeman, 1: 10 Archers
- E6-4 B's army after: 0: 10 Swordsmen, 1: 9 Pikemen
- anything odd: 

## E6-11 — A meeting's price: a straight grass step toward an own hero (GAM-MET-001, ADV-OBJ-003)

- Red hero **E6-11** (Pyre, specialty creature Ballista): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman
- Red hero **E6-11 B** (Nymus, specialty creature Pit Fiend): attack 1, defense 1, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Pikeman

1. Day 1, Red's turn, after E6-4: read E6-11's movement points and those of E6-11 B (east of it).
2. Click E6-11 B twice: the exchange screen opens; close it without a change.
3. Read E6-11's movement points and tile, and E6-11 B's movement points and tile. End Red's turn.

**Record**

- E6-11 points before: 1560
- E6-11 B points before: 1560
- E6-11 points after: 1460
- E6-11 tile after: 27,3,0
- E6-11 B points after: 1560
- E6-11 B tile after: 28,3,0
- anything odd: 

## E6-6a — Advanced Tactics against a monster with the tactics formation off (GAM-BAT-009, BAT-TAC-001)

- Blue hero **E6-6a** (Yog, specialty creature Cyclops): attack 2, defense 2, power 1, knowledge 1; Advanced Tactics; no artifacts; no spells; army 20 Swordsman
- Opponent: 5 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn: read E6-6a's flags (hero +0x48). Open its hero screen: press the tactics formation button once (it shows only for a hero with Tactics) and close the screen. Read the flags again.
2. Click the 5 Pikemen east of E6-6a twice (savage: a battle). Read whether the battle starts in the tactics phase (battle manager +0x13D68; on screen: the tactics phase's Start button and the placement zone). End a tactics phase at once; the battle by auto combat.

**Record**

- flags before: 2 (loose, tactics formation on)
- flags after the button: 0 (loose, tactics formation off)
- tactics phase (yes/no; +0x13D68): no
- battle's outcome: attacker
- anything odd: 

## E6-6b — Advanced Tactics against a monster with the tactics formation on (the control) (GAM-BAT-009, BAT-TAC-001)

- Blue hero **E6-6b** (Krellion, specialty creature Ogre): attack 2, defense 2, power 1, knowledge 1; Advanced Tactics; no artifacts; no spells; army 20 Swordsman
- Opponent: 5 Pikeman (wandering monster, never flees)

1. Day 1, Blue's turn, after E6-6a: read E6-6b's flags (hero +0x48); do not press any button.
2. Click the 5 Pikemen east of E6-6b twice. Read whether the battle starts in the tactics phase, as in E6-6a. End the tactics phase at once (Start); the battle by auto combat.

**Record**

- flags: 2 (loose, tactics formation on)
- tactics phase (yes/no; +0x13D68): yes
- battle's outcome: attacker
- anything odd: 

## E6-7 — Scholar at a meeting: each hero learns the other's spells up to its own limit (HER-SPL-007, GAM-MET-001)

- Blue hero **E6-7** (Halon, specialty skill Mysticism): attack 1, defense 1, power 2, knowledge 2; Expert Scholar, Expert Wisdom; Spell Book (spell book); spells Precision, Fireball, Prayer; army 10 Pikeman
- Blue hero **E6-7 B** (Theodorus, specialty creature Mage): attack 1, defense 1, power 2, knowledge 2; no skills; Spell Book (spell book); spells Bless, Chain Lightning, Implosion; army 10 Pikeman

1. Day 1, Blue's turn, after E6-6b: read the learned spells of E6-7 (Expert Scholar, Expert Wisdom: Precision, Fireball, Prayer) and of E6-7 B east of it (no skills: Bless, Chain Lightning, Implosion), from memory or each spell book.
2. Click E6-7 B twice. Note the message before the exchange screen (its first words and the spells it names); close the exchange without a change.
3. Read both heroes' learned spells again.

**Record**

- E6-7 spells before: Fireball, Precision, Prayer
- E6-7 B spells before: Implosion, Chain Lightning, Bless
- the message: ? (not one record: composed of GENRLTXT 140, 141, 142, 143, 148 and 149 with the heroes' and spells' names; it says E6-7 learns Chain Lightning and Bless from E6-7 B and teaches Precision to E6-7 B; its pictures: Precision, Bless, Chain Lightning and Expert Scholar)
- E6-7 spells after: Chain Lightning, Fireball, Bless, Precision, Prayer
- E6-7 B spells after: Implosion, Chain Lightning, Bless, Precision
- anything odd: 

## E6-8 — Wizard's Well: the question after an equip once, the right-click's question, the position, the backpack (HER-ART-006, HER-ART-001, D-28)

- Blue hero **E6-8** (Calid, specialty resource sulfur): attack 1, defense 1, power 1, knowledge 1; no skills; Charm of Mana (misc 2), Talisman of Mana (misc 3), Mystic Orb of Mana (backpack); no spells; army 10 Pikeman

1. Day 1, Blue's turn, after E6-7: read E6-8's worn positions and backpack (Charm of Mana on misc 2 = 10, Talisman of Mana on misc 3 = 11, Mystic Orb of Mana in the backpack; misc 1 = 9 and misc 4 = 12 empty). Open its hero screen.
2. Pick the Mystic Orb of Mana up from the backpack and put it on **misc 4 (position 12)**. Note any question (its first words) and answer **No**. Read the worn positions.
3. Pick the Orb up from misc 4 and put it back on misc 4. Note any question (answer **No** if one comes).
4. Right-click the Orb on misc 4. Note the question and answer **Yes**. Read the worn positions (and note on screen where the Wizard's Well and the locks show).
5. Pick the Wizard's Well up and put it into the backpack. Right-click it there: note what opens (a question, and its first words, or only the artifact's window); answer a question **No**. Read the worn positions and the backpack. Close the screen.

**Record**

- worn at the start (positions): 10: Charm of Mana, 11: Talisman of Mana, 16: Catapult
- backpack at the start: Mystic Orb of Mana
- question after the first equip: GENRLTXT:733
- worn after the first equip: 10: Charm of Mana, 11: Talisman of Mana, 12: Mystic Orb of Mana, 16: Catapult
- question after the second equip: none
- question on the right-click: question GENRLTXT:733
- worn after the assembly (positions): 9: Wizard's Well, 16: Catapult
- on screen: the Well's slot and the locked slots: the Well on misc 1 (position 9); locks on misc 4 (12) and misc 5 (18); misc 2 and 3 (10, 11) shown empty
- right-click on the Well in the backpack: what opens: window ?
- worn at the end (positions): 16: Catapult
- backpack at the end: Wizard's Well
- anything odd: 

## E6-10 — The backpack's arrow rotates the array (E6-10); then Titan's Thunder's book (E6-9) (HER-ART-002, HER-ART-007)

- Blue hero **E6-10** (Tyris, specialty creature Cavalier): attack 1, defense 1, power 1, knowledge 1; no skills; Centaurs Axe (backpack), Greater Gnoll's Flail (backpack), Titan's Thunder (backpack), Ogre's Club of Havoc (backpack), Sword of Hellfire (backpack), Shield of the Dwarven Lords (backpack); no spells; army 10 Pikeman

1. Day 1, Blue's turn, after E6-8: read E6-10's backpack in order and its book position (17): it has no Spell Book. Open its hero screen.
2. **E6-10:** click the backpack's **right** arrow once and close the screen. Read the backpack in order again (and note which five the screen showed before and after the click).
3. **E6-9:** open the hero screen again. Pick Titan's Thunder up from the backpack and put it on the right hand (position 3). Read the book position and whether a Spell Book shows; note any message.
4. Pick Titan's Thunder up from the right hand and put it into the backpack. Read the book position again. Close the screen.

**Record**

- backpack before (in order): Centaurs Axe, Greater Gnoll's Flail, Titan's Thunder, Ogre's Club of Havoc, Sword of Hellfire, Shield of the Dwarven Lords
- book position at the start: 16: Catapult
- on screen before the click (five, left to right): Centaurs Axe, Greater Gnoll's Flail, Titan's Thunder, Ogre's Club of Havoc, Sword of Hellfire
- backpack after the click (in order): Greater Gnoll's Flail, Titan's Thunder, Ogre's Club of Havoc, Sword of Hellfire, Shield of the Dwarven Lords, Centaurs Axe
- on screen after the click: Greater Gnoll's Flail, Titan's Thunder, Ogre's Club of Havoc, Sword of Hellfire, Shield of the Dwarven Lords
- E6-9: book position with the Thunder on: 3: Titan's Thunder, 16: Catapult, 17: Spell Book
- E6-9: message (or none): none
- E6-9: book position with the Thunder off: 16: Catapult, 17: Spell Book
- E6-9: backpack after (in order): Titan's Thunder, Greater Gnoll's Flail, Ogre's Club of Havoc, Sword of Hellfire, Shield of the Dwarven Lords, Centaurs Axe
- anything odd: 
