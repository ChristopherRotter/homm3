# E08-batch-4b — the scorer's report

`cargo run -p h3-map --example e4_score -- <the batch's map> experiments/results/E08-batch-4b.md` (E08 T32b2): per hero, from its seed byte, each source's prediction beside the record and which fit (the map: the spec compiled by `h3-experiment`, the same stamp). Scored in T33.

## E4-6a (Tyris, Knight)

Seed byte 71.

At load (10000 experience):

- the reading: 4600 experience, level 5, 3/4/1/2, Leadership expert, Ballistics advanced; Wisdom last offered 0, a school 5
  - level 2: defense, Leadership | Artillery
  - level 3: attack, Leadership | Ballistics
  - level 4: knowledge, Leadership | Air Magic
  - level 5: defense, Ballistics | Earth Magic
- always-left: 4600 experience, level 5, 3/4/1/2, Leadership expert, Water Magic advanced; Wisdom last offered 4, a school 5
  - level 2: defense, Leadership | Artillery
  - level 3: attack, Leadership | Ballistics
  - level 4: knowledge, Water Magic | Wisdom
  - level 5: defense, Water Magic | Estates
- VCMI's limit not applied at load (D-21; its own draws: experience and level only): 10000 experience, level 8, 4/5/1/3, Leadership expert, Water Magic expert, Offense advanced; Wisdom last offered 4, a school 6
  - level 2: defense, Leadership | Artillery
  - level 3: attack, Leadership | Ballistics
  - level 4: knowledge, Water Magic | Wisdom
  - level 5: defense, Water Magic | Estates
  - level 6: knowledge, Water Magic | Tactics
  - level 7: defense, Offense | Archery
  - level 8: attack, Offense | Ballistics
- recorded: 4600 experience, level 5, 3/4/1/2, Leadership expert, Ballistics advanced; Wisdom last offered 0, a school 5
- fits: the reading

## E4-6b (Sorsha, Knight)

Seed byte 107.

At load (4000 experience):

- the reading: 4000 experience, level 4, 4/3/1/1, Leadership expert, Artillery basic; Wisdom last offered 0, a school 4
  - level 2: attack, Leadership | Navigation
  - level 3: defense, Leadership | Resistance
  - level 4: attack, Water Magic | Artillery
- always-left: 4000 experience, level 4, 4/3/1/1, Leadership expert, Water Magic basic; Wisdom last offered 0, a school 4
  - level 2: attack, Leadership | Navigation
  - level 3: defense, Leadership | Resistance
  - level 4: attack, Water Magic | Artillery
- VCMI's limit not applied at load (D-21; its own draws: experience and level only): 4000 experience, level 4, 4/3/1/1, Leadership expert, Water Magic basic; Wisdom last offered 0, a school 4
  - level 2: attack, Leadership | Navigation
  - level 3: defense, Leadership | Resistance
  - level 4: attack, Water Magic | Artillery
- recorded: 4000 experience, level 4, 4/3/1/1, Leadership expert, Artillery basic; Wisdom last offered 0, a school 4
- fits: the reading, VCMI's limit not applied at load (D-21; its own draws: experience and level only)

In play (1 dialog(s), 4600 experience after):

- the reading: level 5: attack, Artillery | Offense
- recorded: level 5: attack, Artillery | Offense (taken left (Artillery))
- fits: the reading 

