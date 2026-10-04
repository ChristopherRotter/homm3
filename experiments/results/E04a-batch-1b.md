# E04a batch 1b — results

Map file: `E04a-batch-1b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

1. Copy `E04a-batch-1b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty 100 % (Normal) or easier, so that both players start with gold (R4 needs it). In the battles against Blue's heroes (R3–R7) you control both sides.
4. Save right away (for example `batch1b`). Do R1 first: it loads this save once.
5. On day 1, play Red's encounters R1–R8 (R1 first, the others in any order). Then end Red's turn and do B6 and B7 on Blue's turn.
6. In battle, the combat log at the bottom shows each strike's damage and each heal (scroll it with its arrow); right-click a stack to see its values. When a stack gets an extra turn from good morale, let it Defend unless the encounter says otherwise.
7. Where an encounter says *retreat*, end the battle with Retreat (Red loses that hero, which is intended). Use Auto combat only where an encounter allows it: some battles need a particular order of kills.
8. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## R1 — Angelic Alliance: which creatures may mix without a morale penalty (artifacts D1)

- Red hero **R1** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; Angelic Alliance (right hand); army 5 Pikeman, 5 Centaur, 5 Pixie, 5 Peasant

1. Right after saving: open R1's hero screen and right-click the morale icon. Note the morale value and the lines of its explanation (the army: Pikemen, Centaurs, Pixies, Peasants).
2. Dismiss the Peasants (select the stack, then the dismiss button). Note morale and explanation again (Pikemen, Centaurs, Pixies).
3. Load the save. Open R1's hero screen again and dismiss the Pixies. Note morale and explanation (Pikemen, Centaurs, Peasants).
4. Also dismiss the Peasants. Note morale and explanation (Pikemen, Centaurs).

**Record**

| reading | morale | explanation (its lines) |
|---|---|---|
|   |   |   |
|   |   |   |
|   |   |   |
|   |   |   |

- Anything unexpected: There are no morale modifiers shown in the hero screen (and it says neutral morale), even when unequipping the artifact. Deviating from the scenario, upon entering a battle: It shows "Troops of three alignments -1". When dismissing either pixies or peasants beforehand: neutral morale.

## R2 — Tactics phase: zone, war machines, placement (OQ-2, OQ-3)

- Red hero **R2** (Kyrre, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; Expert Tactics; Ballista (war machine 1), First Aid Tent (war machine 3); army 1 Pikeman, 1 Pikeman, 1 Pikeman, 1 Pikeman, 5 Peasant, 5 Pixie
- Opponent: 3 Peasant (wandering monster, never flees)

1. Attack the Peasants. A tactics phase should start before round 1 (Expert Tactics against a monster without a hero).
2. Before moving anything, look at the highlighted zone: in the top row and in the second row, which is the zone's rightmost hex, counted from that row's leftmost hex (1 = leftmost)? Hexes with a stack or an obstacle inside the zone count too.
3. Click the Ballista and the First Aid Tent: can either be selected or moved?
4. Which buttons are available during the phase? Is Retreat available (do not retreat)?
5. Select the Peasants (speed 3): can they move to a free hex of the zone more than 3 hexes away (for example from their place to the zone's far corner in another row)?
6. Move one stack twice: allowed? Can you pick any stack yourself (click it), or does the game hand you the stacks in a fixed order?
7. Pick an empty hex in the top row inside the zone, away from its left and right edge. Move four Pikemen onto the hexes left and right of it and the two hexes touching it from below. Then select the Peasants: can they move into the enclosed hex (is it highlighted)? Then select the Pixies (flyers): can they?
8. End the tactics phase and finish the battle any way (Auto combat is fine).

**Record**

- Zone: rightmost hex in the top row / in the second row: 7 for either
- Ballista or Tent selectable or movable (yes/no): no
- Buttons during the phase; Retreat available (yes/no): yes
- Peasants moved more than 3 hexes (yes/no): yes
- A stack moved twice (yes/no); stacks picked freely or in a fixed order: yes; can move any creature anywhere as long as pathing permits (cannot walk over obstacles), and any creature can move any amount of times; game dictates the order and moving the creature yields control to the next one in sequence
- Enclosed hex available to the Peasants / to the Pixies (yes/no each): Peasants: no, Pixies: yes
- Anything unexpected:

## R3 — Tactics: does a disabled Tactics formation still cancel the enemy's Tactics? (OQ-1)

- Red hero **R3** (Gunnar, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; Advanced Tactics; no artifacts; army 10 Pikeman
- Opponent: Blue hero **B1** (Dessa, specialty skill Logistics): attack 0, defense 0, power 1, knowledge 1; Basic Tactics; no artifacts; army 10 Pikeman

1. Before attacking, open R3's hero screen and switch the Tactics formation off (the tactics button next to the formation buttons; its tooltip reads *Disable Tactics formation* or similar).
2. Attack Blue's hero B1 (Basic Tactics, formation on). Does a tactics phase start, and for which side? If one starts, note the zone's outermost hex in the top row, counted from that side's edge (1 = the hex at the edge).
3. Retreat with Red (or end the tactics phase and retreat in round 1).

**Record**

- Tactics phase: none / Red / Blue: none
- If one: outermost zone hex in the top row: n/a
- Anything unexpected:

## R4 — Surrender: price, discounts, and whether the enemy may refuse (D-1, D-2, D-5)

- Red hero **R4** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1); army 7 Pikeman, 1 Pixie
- Opponent: Blue hero **B2** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; Basic Diplomacy; Statesman's Medal (neck), Diplomat's Ring (right ring); army 10 Pikeman

1. Attack Blue's hero B2.
2. At Red's first stack turn in round 1, press Surrender. Write down exactly what appears (text, price, buttons), then decline. Press Surrender again right away: what happens? Decline again.
3. At Blue's first stack turn, press Surrender for Blue. Note the price, then decline.
4. Play on until both sides have lost some creatures (Red's Ballista shoots on its own). At a Red turn, right-click to note the counts of Red's Pikemen, Red's Pixies and Blue's Pikemen, then press Surrender: note the price and decline.
5. Note Red's gold (resource bar). Press Surrender and accept. Is Blue asked whether to accept? If so, refuse once with Blue, play on, offer again in the next round and accept with Blue this time. Note the messages and Red's gold afterwards.
6. Only if Blue was never asked in step 5: start a new single-player game of this map with Blue as a computer player. With R4, attack B2 and press Surrender in round 1 and accept: does the computer accept? If the battle goes on, try again in rounds 2 and 3.

**Record**

- Step 2: what appeared (text, price, buttons): 'B2 states: "I will accept your surrender and grant you and your troops safe passage for the price of 1472 gold."'
- Step 2: Surrender pressed again right away: always the same, no matter how many times it is pressed + declined
- Step 3: Blue's price: 179 gold
- Step 4: Red Pikemen, Red Pixies, Blue Pikemen; Red's price: Red: 6 Pike, 0 Pixies, Blue: 9 Pike, Red's price: 1430
- Step 5: was Blue asked? What happened (messages): was not asked
- Step 5: Red's gold before / after: 30000 -> 28570
- Step 6 (only if played): computer accepted or refused, in which rounds: surrender just ends battle, computer player was not asked for permission
- Anything unexpected:

## R5 — Experience: killed war machines, Learning on the +500, captured war machines (D-3, OQ-6, OQ-5)

- Red hero **R5** (Jenova, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; Expert Learning; no artifacts; army 3 Archangel, 3 Archangel, 3 Archangel
- Opponent: Blue hero **B3** (Aine, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; Ballista (war machine 1), Ammo Cart (war machine 2), First Aid Tent (war machine 3); army 1 Gold Golem

1. Note R5's experience on the hero screen (0).
2. Attack Blue's hero B3. In round 1, destroy the Ballista with one Archangel stack and the First Aid Tent with another; B3's Gold Golem Defends. Do not attack the Ammo Cart. Then kill the Gold Golem with the third stack (or in round 2). No Auto combat.
3. Note the experience in the victory message (*For valor in combat, R5 receives … experience*).
4. Accept any level-up offers. Open R5's hero screen: does R5 now have an Ammo Cart (or any war machine)?

**Record**

- Experience before the battle: 0
- Experience in the victory message: 632
- Order of kills as played; did the Ammo Cart survive?: Ballista, Tent, Golem; Ammo cart survived
- War machines of R5 after the battle: Catapult only
- Anything unexpected:

## R6 — Necromancy: rounding per killed stack or over the sum (D-7)

- Red hero **R6** (Clavius, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; Basic Necromancy; no artifacts; army 3 Archangel, 3 Archangel, 3 Archangel
- Opponent: Blue hero **B4** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; army 6 Pikeman, 6 Pikeman, 6 Pikeman, 14 Pikeman, 17 Pikeman

1. Attack Blue's hero B4 and kill all five Pikeman stacks (any order; no Auto combat needed, but it is allowed if no Archangel stack dies).
2. After the battle, note the number of Skeletons in the Necromancy message (*… able to raise … Skeletons*) and R6's Skeletons on the hero screen.

**Record**

- Skeletons in the Necromancy message: 2
- Skeletons in R6's army afterwards: 2
- Anything unexpected:

## R7 — First Aid Tent under the hero's control: targets, Wait/Defend, heal amounts; Ballista at long range (T7b (BAT-WAR-007), artifacts open 2 (baseline))

- Red hero **R7** (Damacon, specialty resource gold): attack 4, defense 0, power 1, knowledge 1; Basic First Aid; Ballista (war machine 1), First Aid Tent (war machine 3); army 3 Behemoth, 3 Behemoth, 30 Peasant
- Opponent: Blue hero **B5** (Calid, specialty resource sulfur): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; army 25 Archer, 25 Archer

1. Attack Blue's hero B5. Every round, Red's Behemoths and Peasants Defend. Blue's Archers always shoot (never Defend): aim them so that before the tent's turn one Behemoth stack's top Behemoth has at most 110 of its 160 *Health Left*; spare volleys go to the Peasants. Never kill a Behemoth stack. In round 1, one volley goes to Red's Ballista.
2. At the tent's first turn: can you choose whom to heal? Are Wait and Defend available? Can you pick the hurt Ballista as the target? If Wait is there, use it once: does the tent get its turn again later in the round?
3. At each tent turn, right-click the Behemoth stack you heal (*Health Left*), heal it and note the amount from the log (*The First Aid Tent heals the Behemoth removing … points of damage*).
4. Red's Ballista shoots on its own at the Archers (no Artillery). Note the damage of each of its shots from the log.
5. Go on until 8 heals are noted, then retreat with Red.

**Record**

- Tent: target chosen by you (yes/no); Wait / Defend available; Ballista as target (yes/no): yes (but Ballista cannot be targeted for healing); yes; yes
- After Wait: did the tent act again later in the round?: yes

| round | Health Left before | healed |
|---|---|---|
|3|99|20|
|4|119|41|
|5|100|47|
|6|105|50|
|8|102|41|
|9|98|6|
|10|104|11|
|11|110|24|

- Ballista damage per shot: 7, 8, 9, 7, 10, 11, 11, 9, 9, 8, 10
- Anything unexpected: Archers did not enough damage, had to defend with the tent multiple rounds to get Behemoth <= 110 HP

## R8 — War machines and creature artifacts: health, speed, Golden Bow on the Ballista (artifacts open 1, artifacts open 2)

- Red hero **R8** (Saurug, specialty resource gems): attack 4, defense 0, power 1, knowledge 1; no skills; Ring of Life (left ring), Vial of Lifeblood (misc 1), Golden Bow (misc 2), Necklace of Swiftness (neck), Ring of Vitality (right ring), Cape of Velocity (shoulders), Ballista (war machine 1), Ammo Cart (war machine 2), First Aid Tent (war machine 3); army 60 Pikeman
- Opponent: 30 Archer (wandering monster, never flees)

1. Before attacking, right-click the Pikemen on R8's hero screen: note Health and Speed.
2. Attack the Archers. At the start of round 1, right-click the Ballista, the Ammo Cart and the First Aid Tent: note Health and Speed of each.
3. The Pikemen Defend every round. Note the damage of each of the Ballista's first three shots (it shoots on its own), then finish the battle any way (Auto combat is fine).

**Record**

- Pikemen on the hero screen: Health, Speed: 14, 7

| war machine | Health | Speed |
|---|---|---|
|Ballista|254|0|
|Ammo cart|104|0
|First aid tent|79|0|

- Ballista damage per shot: 23, 17, 20
- Anything unexpected:

## B6 — Shackles of War against a monster: can the wearer retreat? (D-6)

- Blue hero **B6** (Sephinroth, specialty resource crystal): attack 0, defense 0, power 1, knowledge 1; no skills; Shackles of War (misc 1); army 1 Archangel
- Opponent: 5 Peasant (wandering monster, never flees)

1. On Blue's turn, attack the Peasants with B6.
2. At B6's first stack turn, press Retreat. Note exactly what happens (message text). If the game asks *Are you sure you want to retreat?*, answer No.
3. Finish the battle any way (Auto combat is fine).

**Record**

- What happened when Retreat was pressed (message): 'The Shackles of War are present. B6 can not retreat!'
- Anything unexpected:

## B7 — Xeron's specialty: does it raise the Devils' speed? (H-2)

- Blue hero **B7** (Xeron, specialty creature stats Devil): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; army 5 Devil

1. On Blue's turn, open Xeron's hero screen (named B7) and right-click the Devils: note Attack, Defense and Speed as shown (Devil base values 19 / 21 / 11).

**Record**

| creature | attack | defense | speed |
|---|---|---|---|
|Devils|23|23|12|

- Anything unexpected:
