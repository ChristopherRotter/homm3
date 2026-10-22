# E08-batch-5d-run2 — the scorer's report

`cargo run -p h3-map --example e4_score -- <the batch's map> experiments/results/E08-batch-5d-run2.md` (E08 T37b): per hero, from its seed byte, each source's prediction beside the record and which fit (the map: the spec compiled by `h3-experiment`, the same stamp). Scored in T38.

## E5-2d (Kyrre, Ranger)

Seed byte 172.

At load (1000 experience):

- the reading: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 2, a school 0
  - level 2: attack, Logistics | Wisdom
- always-left: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 2, a school 0
  - level 2: attack, Logistics | Wisdom
- recorded: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 2, a school 0
- fits: the reading, always-left

Day's points (the land table's 1500):

- the reading's hero (level 2, Logistics advanced): single precision 1829, exact 1830
- the recorded hero (level 2, Logistics advanced): single precision 1829, exact 1830
- recorded: day 1 1829, day 2 1829
- fits: single precision (the reading, Q-0005)

## E5-2e (Gunnar, Overlord)

Seed byte 201.

At load (2000 experience):

- the reading: 2000 experience, level 3, 2/2/1/1, Logistics advanced, Archery basic; Wisdom last offered 0, a school 0
  - level 2: attack, Logistics | Artillery
  - level 3: defense, Logistics | Archery
- always-left: 2000 experience, level 3, 2/2/1/1, Logistics expert; Wisdom last offered 0, a school 0
  - level 2: attack, Logistics | Artillery
  - level 3: defense, Logistics | Archery
- recorded: 2000 experience, level 3, 2/2/1/1, Logistics advanced, Archery basic; Wisdom last offered 0, a school 0
- fits: the reading

Day's points (the land table's 1500):

- the reading's hero (level 3, Logistics advanced): single precision 1845, exact 1845
- the recorded hero (level 3, Logistics advanced): single precision 1845, exact 1845
- recorded: day 1 1845, day 2 1845
- fits: single precision (the reading, Q-0005), exact (strict-fixed) the arithmetics agree here: inconclusive

## E5-2f (Dessa, Battle Mage)

Seed byte 137.

At load (6200 experience):

- the reading: 6200 experience, level 6, 2/1/4/2, Logistics advanced, Archery basic, Armorer expert, Offense basic, Leadership basic, Luck basic, Resistance basic, First Aid expert; Wisdom last offered 0, a school 0
  - level 2: power, Resistance | Armorer
  - level 3: attack, First Aid | Logistics
  - level 4: power, Armorer | Offense
  - level 5: power, Archery | First Aid
  - level 6: knowledge, Logistics | Leadership
- always-left: 6200 experience, level 6, 2/1/4/2, Logistics advanced, Archery basic, Armorer advanced, Offense basic, Leadership basic, Luck basic, Resistance expert, First Aid advanced; Wisdom last offered 0, a school 0
  - level 2: power, Resistance | Armorer
  - level 3: attack, First Aid | Logistics
  - level 4: power, Armorer | Offense
  - level 5: power, Resistance | First Aid
  - level 6: knowledge, Logistics | Offense
- recorded: 6200 experience, level 6, 2/1/4/2, Logistics advanced, Archery basic, Armorer expert, Offense basic, Leadership basic, Luck basic, Resistance basic, First Aid expert; Wisdom last offered 0, a school 0
- fits: the reading

Day's points (the land table's 1500):

- the reading's hero (level 6, Logistics advanced): single precision 1889, exact 1890
- the recorded hero (level 6, Logistics advanced): single precision 1889, exact 1890
- recorded: day 1 1889, day 2 1889
- fits: single precision (the reading, Q-0005)

