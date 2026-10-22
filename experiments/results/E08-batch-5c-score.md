# E08-batch-5c — the scorer's report

`cargo run -p h3-map --example e4_score -- <the batch's map> experiments/results/E08-batch-5c.md` (E08 T37b): per hero, from its seed byte, each source's prediction beside the record and which fit (the map: the spec compiled by `h3-experiment`, the same stamp). Scored in T38.

## E5-2a (Gunnar, Overlord)

Seed byte 126.

At load (1000 experience):

- the reading: 1000 experience, level 2, 1/1/2/1, Logistics advanced; Wisdom last offered 0, a school 0
  - level 2: power, Logistics | Leadership
- always-left: 1000 experience, level 2, 1/1/2/1, Logistics advanced; Wisdom last offered 0, a school 0
  - level 2: power, Logistics | Leadership
- recorded: 1000 experience, level 2, 1/1/2/1, Logistics advanced; Wisdom last offered 0, a school 0
- fits: the reading, always-left

Day's points (the land table's 1500):

- the reading's hero (level 2, Logistics advanced): single precision 1829, exact 1830
- the recorded hero (level 2, Logistics advanced): single precision 1829, exact 1830
- recorded: day 1 1829, day 2 1829
- fits: single precision (the reading, Q-0005)

## E5-2b (Kyrre, Ranger)

Seed byte 167.

At load (6200 experience):

- the reading: 6200 experience, level 6, 2/4/2/1, Logistics expert, Archery basic, Armorer expert, Offense basic, Leadership basic, Luck basic, Resistance basic, First Aid advanced; Wisdom last offered 0, a school 0
  - level 2: power, Logistics | Armorer
  - level 3: defense, Archery | First Aid
  - level 4: attack, Leadership | Logistics
  - level 5: defense, Leadership | Armorer
  - level 6: defense, Leadership | Armorer
- always-left: 6200 experience, level 6, 2/4/2/1, Logistics advanced, Archery expert, Armorer basic, Offense basic, Leadership expert, Luck basic, Resistance basic, First Aid basic; Wisdom last offered 0, a school 0
  - level 2: power, Logistics | Armorer
  - level 3: defense, Archery | First Aid
  - level 4: attack, Leadership | Logistics
  - level 5: defense, Leadership | Luck
  - level 6: defense, Archery | Luck
- recorded: 6200 experience, level 6, 2/4/2/1, Logistics expert, Archery basic, Armorer expert, Offense basic, Leadership basic, Luck basic, Resistance basic, First Aid advanced; Wisdom last offered 0, a school 0
- fits: the reading

Day's points (the land table's 1500):

- the reading's hero (level 6, Logistics expert): single precision 2085, exact 2085
- the recorded hero (level 6, Logistics expert): single precision 2085, exact 2085
- recorded: day 1 2085, day 2 2085
- fits: single precision (the reading, Q-0005), exact (strict-fixed) the arithmetics agree here: inconclusive

## E5-2c (Dessa, Battle Mage)

Seed byte 164.

At load (20600 experience):

- the reading: 20600 experience, level 12, 4/4/4/3, Logistics expert, Scouting expert, Wisdom expert, Intelligence expert; Wisdom last offered 8, a school 12
  - level 2: power, Logistics | Fire Magic
  - level 3: attack, Logistics | Wisdom
  - level 4: power, Scouting | Artillery
  - level 5: defense, Scouting | Fire Magic
  - level 6: knowledge, Scouting | Wisdom
  - level 7: defense, Wisdom | Luck
  - level 8: attack, Wisdom | Fire Magic
  - level 9: power, Scouting | Fire Magic
  - level 10: attack, Eagle Eye | Intelligence
  - level 11: knowledge, Intelligence | Sorcery
  - level 12: defense, Intelligence | Earth Magic
- always-left: 20600 experience, level 12, 4/4/4/3, Logistics expert, Scouting expert, First Aid expert, Artillery expert; Wisdom last offered 12, a school 11
  - level 2: power, Logistics | Fire Magic
  - level 3: attack, Logistics | Wisdom
  - level 4: power, Scouting | Artillery
  - level 5: defense, Scouting | Fire Magic
  - level 6: knowledge, Scouting | Wisdom
  - level 7: defense, First Aid | Resistance
  - level 8: attack, First Aid | Fire Magic
  - level 9: power, First Aid | Wisdom
  - level 10: attack, Artillery | Tactics
  - level 11: knowledge, Artillery | Water Magic
  - level 12: defense, Artillery | Wisdom
- recorded: 20600 experience, level 12, 4/4/4/3, Logistics expert, Scouting expert, Wisdom expert, Intelligence expert; Wisdom last offered 8, a school 12
- fits: the reading

Day's points (the land table's 1500):

- the reading's hero (level 12, Logistics expert): single precision 2220, exact 2220
- the recorded hero (level 12, Logistics expert): single precision 2220, exact 2220
- recorded: day 1 2220, day 2 2220
- fits: single precision (the reading, Q-0005), exact (strict-fixed) the arithmetics agree here: inconclusive

