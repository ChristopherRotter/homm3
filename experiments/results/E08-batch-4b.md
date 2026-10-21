# E08 batch E4b — results

> Played by the capture harness on 2026-10-21 (`clients/capture/h3adv.py batch clients/capture/batches/E08-batch-4b.toml`): the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, difficulty Easy, the starting bonus random, Show Route on; read from the game's memory.
>
> One run (E08 T32b2); the scorer's report beside it, `E08-batch-4b-score.md` (`crates/h3-map/examples/e4_score.rs`).

Map file: `E08-batch-4b.h3m`. Fill in the blanks below and keep this file; an agent records the results (`docs/experiments/README.md`).

**Stamp `8d4a4c2d`.** The map's description in the scenario list starts with "Stamp 8d4a4c2d.": compare the two before you play. If they differ, the map is not this template's build: do not play it, ask for the map again.

**Difficulty: Easy.** Choose it in the scenario list before you start.

1. Copy `E08-batch-4b.h3m` into `D:\Heroes3\HoMM 3 Complete\Maps`.
2. Use the HD launcher with plain HD (HD+ off).
3. Start the map in hot seat: *New Game → Multiplayer → Hot Seat*, two players, Red and Blue both human, difficulty **Easy**. Before you start it, compare the stamp at the start of the map's description with the stamp above. The scenario's information shows its level limit, 5. Save right away (save 1).
4. The heroes' seed bytes (hero +0x8F) and the levels Wisdom (+0x90) and a magic school (+0x3F) were last offered show only in the game's memory: the capture harness reads them (clients/capture/batches/E08-batch-4b.toml). Played by hand, leave them empty.
5. Order: day 1, Red's turn: E4-6a; end the turn. Blue's turn: E4-6b; end the turn. Then the map is done.
6. Fill in this file (anything unexpected goes into the encounter's last line) and keep it in the repo.

## E4-6a — A level limit of 5: a hero with 10 000 experience at the start, then a gain (D-21, D-19, D-20, HER-EXP-003, GAM-SET-002)

- Red hero **E4-6a** (Tyris, specialty creature Cavalier, 10000 experience): attack 2, defense 2, power 1, knowledge 1; Basic Leadership; no artifacts; no spells; army 2 Archangel
- Opponent: 1 Pikeman (wandering monster, never flees)

1. Day 1, Red's turn, before anything moves: read E4-6a's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the Pikeman east of E4-6a twice; the battle by auto combat. Note any message after it and any level-up dialog (answer left).
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 71
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 5
- experience at the start: 4600
- level at the start: 5
- primary skills at the start (attack/defense/power/knowledge): 3/4/1/2
- secondary skills at the start (in the hero's order, each with its level): Leadership expert, Ballistics advanced
- the messages after the battle: GENRLTXT:2, GENRLTXT:7
- dialog 1, level: no dialog
- dialog 1, primary skill raised: no dialog
- dialog 1, left offer (or the only one): no dialog
- dialog 1, right offer: no dialog
- dialog 1, taken: no dialog
- experience after: 4600
- level after: 5
- primary skills after (attack/defense/power/knowledge): 3/4/1/2
- secondary skills after (in the hero's order, each with its level): Leadership expert, Ballistics advanced
- anything odd: 

## E4-6b — A level limit of 5: a hero with 4 000 experience gains 700 (HER-EXP-003, D-19)

- Blue hero **E4-6b** (Sorsha, specialty creature Swordsman, 4000 experience): attack 2, defense 2, power 1, knowledge 1; Basic Leadership; no artifacts; no spells; army 5 Archangel
- Opponent: 20 Swordsman (wandering monster, never flees)

1. Day 1, Blue's turn: read E4-6b's seed byte and the two levels last offered (the harness), its experience, level, primary and secondary skills.
2. Click the 20 Swordsmen east of E4-6b twice; the battle by auto combat. Answer the level-up dialog left and note it; note any message.
3. Note the hero's experience, level, primary and secondary skills after.

**Record**

- seed byte (hero +0x8F): 107
- Wisdom last offered (hero +0x90): 0
- a school last offered (hero +0x3F): 4
- experience at the start: 4000
- level at the start: 4
- primary skills at the start (attack/defense/power/knowledge): 4/3/1/1
- secondary skills at the start (in the hero's order, each with its level): Leadership expert, Artillery basic
- the messages after the battle: GENRLTXT:2
- dialog 1, level: 5
- dialog 1, primary skill raised: attack
- dialog 1, left offer (or the only one): Artillery
- dialog 1, right offer: Offense
- dialog 1, taken: left (Artillery)
- experience after: 4600
- level after: 5
- primary skills after (attack/defense/power/knowledge): 5/3/1/1
- secondary skills after (in the hero's order, each with its level): Leadership expert, Artillery advanced
- anything odd: worn artifacts at the start but the war machines (the starting bonus): Pendant of Dispassion (neck)
