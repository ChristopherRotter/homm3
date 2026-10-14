# E04c batch 3e — results

Map file: `E04c-batch-3e.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `5478ac84`.** The map's description in the scenario list starts with "Stamp 5478ac84.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Run by an agent** (E04c T7c, 2026-10-14) on the client lane's capture harness: the same Complete
4.0 `Heroes3.exe`, under Wine in the cloud sandbox, without the HD mod; hot seat (Red and Blue
human, the agent commanding both armies and, in AT, the keep), one map per encounter
(`h3-experiment … --only <id>`), played by `experiments/agent/turrets.py` (built on `catapult.py`
and `siege.py`): the clock stepped 10 ms at a time and the game's memory read after each step
(each stack's creature +0x34, position +0x38, count +0x4C, lost hit points +0x58, hit points
+0xC0, flags +0x84, shots +0xD8, Air Shield's rounds left +0x208; the battle manager's last
activated stack +0x132B8/+0x132BC and active stack +0x132C8). A turret's shot is a drop of its
target's hit points (count x hit points − lost) while that turret is the last activated stack;
the order within a round is the order of activations (changes of that pair). The active-stack
pointer is set to an automatic turret only for an instant (never seen for the keep); while the
commanded keep waited for Blue it held the keep. A turret is taken as waiting for a command when
it is the acting stack of a human side and nothing read changes for 3 s of game time. Every stack
without a step defended. Rounds are counted by the creatures' acted bits (`catapult.py`); each
battle was played one round past the procedure (that round's shots are given as extra). K was
played twice and AS, AR, AC and AI twice (the second runs to check the draws): the values agree
in every shot. AT was played four times: rounds 1 and 2 drew the same every time, rounds 3 and 4
two ways (below), with each command at the same game time. Attempts where either hero wore a
starting-bonus artifact were played again (AC needed five and seven attempts, K and AT
one to three). A run took about
30 s (AI) to 2 min (K) of real time.

1. Copy `E04c-batch-3e.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps` and start it in hot seat (Red and Blue human). Save right away.
2. Each Red hero stands south of a Blue town's entrance. Play the encounters in any order: select the hero and step north into the town; command both armies as the encounter says (every stack without a step defends).
3. Note every shot of the keep and the towers: the round, which turret, the stack it hit and the hit points that stack lost. Then end the battle as you like and load the save for the next encounter.
4. Fill in this file (anything unexpected goes into the last line of an encounter) and keep it in the repo.

## K — A Castle's keep and two towers against an army without protections: order and damage (BAT-SIE-016, BAT-SIE-018, D-18)

- Red hero **K** (Caitlin, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 6 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel, castle); garrison 10 Peasant

1. Every stack defends; the turrets and the Catapult act on their own.
2. Play to round 8, noting every turret's shot, then end the battle as you like.

**Record**

- K1: the turrets' stacks: each one's position and count: Blue's records after the Peasants (record 0, 10 Peasants): the keep record 1, position 254, count 5; the upper tower record 2, position 255, count 3; the lower tower record 3, position 251, count 3; all creature 149 with 99 shots. Counts and shots unchanged to the end (5, 3, 3; 99 each).
- K2: every turret's shot: the round, the turret, the order within the round, the target and the hit points it lost: every round the same order of activations: the keep, the upper tower, the lower tower, the Catapult, the Titans, the Peasants; each turret one shot, every shot at the Titans (Red record 0), none at the Catapult. Hit points lost (keep, upper tower, lower tower): round 1: 17, 8, 6; round 2: 15, 10, 10; round 3: 14, 9, 10; round 4: 15, 11, 9; round 5: 15, 9, 8; round 6: 13, 9, 10; round 7: 13, 6, 11; round 8: 11, 7, 10 (extra, round 9: 14, 10, 10). Sums over rounds 1–8: keep 113, upper tower 69, lower tower 74. The Titans 1800 → 1544 hit points after round 8, still 6. The Catapult's shot came after the three turrets' every round (wall parts: round 1 part 8 −2, round 2 part 8 −1, round 3 part 10 −2, round 4 part 12 −1, round 5 part 10 −1, round 6 part 12 −1, round 7 part 6 −2, round 8 part 6 −1).
- anything odd: the keep's count is 5 here (Castle with fort, citadel and castle) as in every other encounter of the batch (fort and citadel only). From round 2 on the turrets' shots land while the Titans still carry the defending bit (0x8000000) of their Defend in the round before (it clears at their next turn's start); round 1's shots land before any Defend. Each automatic turret's hit came 1.1–1.2 s of game time after its activation.

## AS — Air Shield on the target: the keep's shots against shielded Titans (BAT-SIE-018, D-16, S-06)

- Red hero **AS** (Jenova, specialty resource gold): attack 0, defense 0, power 10, knowledge 5; Expert Air Magic; Spell Book (spell book); spells Air Shield; army 6 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Every stack defends unless a step below says otherwise.
2. Round 1: at the Titans' turn the Red hero casts Air Shield on them (Expert Air Magic: on every stack of the side), then the Titans defend.
3. Play to round 6, noting the keep's shots, then end the battle as you like.

**Record**

- AS1: the keep's count; every keep's shot: the round, whether Air Shield was on the Titans, the hit points they lost: the keep's count 5 (record 1, position 254). Red's hero cast Air Shield at the Titans' first turn of round 1 (`c`, the book's first spell; at Expert it took effect with no target click), rounds left 10 right after the cast; then the Titans defended. The keep's shots, all at the Titans (rounds left on the Titans when the shot landed): round 1: 15 (no Air Shield, 0); round 2: 9 (on, 9); round 3: 7 (on, 8); round 4: 7 (on, 7); round 5: 7 (on, 6); round 6: 8 (on, 5) (extra, round 7: 9, on, 4).
- anything odd: none. The order every round: the keep, the Catapult, the Titans, the Peasants.

## AR — Expert Armorer on the attacking hero: the keep's shots (BAT-SIE-018, D-16, S-06)

- Red hero **AR** (Rissa, specialty resource mercury): attack 0, defense 0, power 1, knowledge 1; Expert Armorer; no artifacts; no spells; army 6 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Every stack defends.
2. Play to round 10, noting the keep's shots, then end the battle as you like.

**Record**

- AR1: the keep's count; every keep's shot: the round and the hit points the Titans lost: the keep's count 5 (record 1, position 254). The keep's shots, all at the Titans: round 1: 13; round 2: 13; round 3: 11; round 4: 11; round 5: 11; round 6: 14; round 7: 14; round 8: 14; round 9: 12; round 10: 10 (extra, round 11: 13). Sum over rounds 1–10: 123.
- anything odd: none. The order every round: the keep, the Catapult, the Titans, the Peasants.

## AC — Expert Archery on the defending hero (no Artillery): the keep's shots (BAT-SIE-018, D-17, S-07)

- Red hero **AC** (Octavia, specialty resource gold): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 6 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant; visiting hero: Blue hero **AC** (Sylvia, specialty skill Navigation): attack 0, defense 0, power 1, knowledge 1; Expert Archery; no artifacts; no spells; army 10 Peasant

1. Every stack defends; the keep shoots on its own (its hero has no Artillery).
2. Play to round 10, noting the keep's shots, then end the battle as you like.

**Record**

- AC1: the keep's count; did the game wait for a command at the keep's turn; every keep's shot: the round and the hit points the Titans lost: the keep's count 5 (record 1, position 254). The game did not wait: at every keep's turn the keep shot with no command, its hit 1.1–1.2 s of game time after its activation (as K's automatic turrets). The keep's shots, all at the Titans: round 1: 15; round 2: 10; round 3: 14; round 4: 16; round 5: 16; round 6: 15; round 7: 16; round 8: 16; round 9: 17; round 10: 17. Sum over rounds 1–10: 152.
- anything odd: Blue's side holds one stack of 20 Peasants (the visiting hero's 10 and the garrison's 10 in one record). The Catapult's shot of round 10 (after the keep's shot) took part 15 to 0 and the keep's record got the dead bit (flags 0x220444): there was no keep's shot in round 11. The order every round: the keep, the Catapult, the Titans, the Peasants.

## AT — Expert Artillery on the defending hero: the keep commanded (BAT-SIE-016, BAT-SIE-018)

- Red hero **AT** (Sorsha, specialty creature Swordsman): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 6 Titan
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant; visiting hero: Blue hero **AT** (Edric, specialty creature Griffin): attack 0, defense 0, power 1, knowledge 1; Expert Artillery; no artifacts; no spells; army 10 Peasant

1. Every stack defends unless a step below says otherwise.
2. Round 1: at the keep's turn, Blue shoots the Titans with it (a click on the Titans).
3. Round 2: at the keep's turn, Blue makes it wait; when it gets its turn again, it shoots the Titans.
4. Rounds 3 and 4: the keep shoots the Titans at its turn. Then end the battle as you like.

**Record**

- AT1: did the game wait for Blue's command at the keep's turn; how many shots per turn; the Wait accepted, and when the keep acted after it: yes: at every keep's turn the keep became the active stack (+0x132C8) and the game stood still until Blue's click (the agent clicked after 3 s of game time each time); the keep was the round's first activation in rounds 1, 3 and 4. One shot per turn: after the click one hit of the Titans 1.2 s later, then the Catapult's activation; the keep's shots stayed 99. Round 2: the Wait (key `w`) was accepted at once: the keep's waited bit (0x2000000) set, then the Catapult, the Titans and the Peasants acted, and the keep was activated again as the round's fifth and last activation (after every other stack), waited for a command again, and shot. Round 3 opened with the keep again.
- AT2: every keep's shot: the round and the hit points the Titans lost: round 1: 16; round 2 (after the Wait): 15; round 3: 13; round 4: 14 (runs 2 and 3); runs 1 and 4 drew round 3: 11, round 4: 15. Rounds 1 and 2 the same in all four runs.
- anything odd: the draws of rounds 3 and 4 differ between runs (two variants in four runs), although every command came at the same game time in each run (the only encounter whose draws did not repeat). Blue's side holds one stack of 20 Peasants (the visiting hero's 10 and the garrison's 10).

## AI — The automatic keep's target: ten Marksmen or one (BAT-SIE-016, D-19)

- Red hero **AI** (Tyris, specialty creature Cavalier): attack 0, defense 0, power 1, knowledge 1; no skills; no artifacts; no spells; army 10 Marksman, 1 Marksman
- Opponent: Blue Castle town (Village Hall, fort, citadel); garrison 10 Peasant

1. Every stack defends; the keep shoots on its own.
2. Play to round 3, noting the keep's targets, then end the battle as you like.

**Record**

- AI1: the keep's count; every keep's shot: the round, the stack it hit (slot) and the hit points it lost: the keep's count 5 (record 1, position 254). Red's records: 10 Marksmen record 0 (slot 0, hex 35), 1 Marksman record 1 (slot 1, hex 137), the Catapult record 2. Round 1: the single Marksman (slot 1): 10 (its whole 10 hit points: killed, 1 → 0); round 2: the ten Marksmen (slot 0): 15 (10 → 9); round 3: slot 0: 14 (9 → 8) (extra, round 4: slot 0: 16, 8 → 6). The Catapult was never hit.
- anything odd: round 1's shot killed the single Marksman, so its damage shows only up to that stack's 10 hit points. The order every round: the keep, the Catapult, the ten Marksmen, the Peasants (the single Marksman's turn never came: it died to round 1's shot).
