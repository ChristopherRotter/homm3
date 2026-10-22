# E08-batch-5c-run2 — the scorer's report

`cargo run -p h3-map --example e4_score -- <the batch's map> experiments/results/E08-batch-5c-run2.md` (E08 T37b): per hero, from its seed byte, each source's prediction beside the record and which fit (the map: the spec compiled by `h3-experiment`, the same stamp). Scored in T38.

## E5-2a (Gunnar, Overlord)

Seed byte 195.

At load (1000 experience):

- the reading: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 0, a school 0
  - level 2: attack, Logistics | Luck
- always-left: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 0, a school 0
  - level 2: attack, Logistics | Luck
- recorded: 1000 experience, level 2, 2/1/1/1, Logistics advanced; Wisdom last offered 0, a school 0
- fits: the reading, always-left

Day's points (the land table's 1500):

- the reading's hero (level 2, Logistics advanced): single precision 1829, exact 1830
- the recorded hero (level 2, Logistics advanced): single precision 1829, exact 1830
- recorded: day 1 1829, day 2 1829
- fits: single precision (the reading, Q-0005)

## E5-2b (Kyrre, Ranger)

Seed byte 24.

At load (6200 experience):

- the reading: 6200 experience, level 6, 4/2/2/1, Logistics basic, Archery advanced, Armorer expert, Offense advanced, Leadership basic, Luck basic, Resistance advanced, First Aid basic; Wisdom last offered 0, a school 0
  - level 2: attack, Armorer | Leadership
  - level 3: power, Offense | Leadership
  - level 4: attack, Luck | Armorer
  - level 5: attack, Archery | Resistance
  - level 6: defense, Leadership | Resistance
- always-left: 6200 experience, level 6, 4/2/2/1, Logistics basic, Archery basic, Armorer expert, Offense advanced, Leadership advanced, Luck advanced, Resistance basic, First Aid basic; Wisdom last offered 0, a school 0
  - level 2: attack, Armorer | Leadership
  - level 3: power, Offense | Leadership
  - level 4: attack, Luck | Armorer
  - level 5: attack, Armorer | Offense
  - level 6: defense, Leadership | Resistance
- recorded: 6200 experience, level 6, 4/2/2/1, Logistics basic, Archery advanced, Armorer expert, Offense advanced, Leadership basic, Luck basic, Resistance advanced, First Aid basic; Wisdom last offered 0, a school 0
- fits: the reading

Day's points (the land table's 1500):

- the reading's hero (level 6, Logistics basic): single precision 1694, exact 1695
- the recorded hero (level 6, Logistics basic): single precision 1694, exact 1695
- recorded: day 1 1694, day 2 1694
- fits: single precision (the reading, Q-0005)

## E5-2c (Dessa, Battle Mage)

Seed byte 183.

At load (20600 experience):

- the reading: 20600 experience, level 12, 5/2/4/4, Logistics expert, Offense expert, Wisdom expert, First Aid expert; Wisdom last offered 9, a school 12
  - level 2: attack, Logistics | Offense
  - level 3: power, Offense | Wisdom
  - level 4: attack, Logistics | Fire Magic
  - level 5: knowledge, Offense | First Aid
  - level 6: defense, Wisdom | Mysticism
  - level 7: knowledge, First Aid | Water Magic
  - level 8: power, Offense | Tactics
  - level 9: attack, Wisdom | Artillery
  - level 10: power, Offense | Fire Magic
  - level 11: attack, Logistics | Sorcery
  - level 12: knowledge, First Aid | Water Magic
- always-left: 20600 experience, level 12, 5/2/4/4, Logistics expert, Earth Magic expert, Scholar expert, Air Magic expert; Wisdom last offered 12, a school 12
  - level 2: attack, Logistics | Offense
  - level 3: power, Logistics | Wisdom
  - level 4: attack, Earth Magic | Armorer
  - level 5: knowledge, Earth Magic | Offense
  - level 6: defense, Earth Magic | Wisdom
  - level 7: knowledge, Scholar | Leadership
  - level 8: power, Scholar | Water Magic
  - level 9: attack, Scholar | Wisdom
  - level 10: power, Air Magic | Offense
  - level 11: attack, Air Magic | Archery
  - level 12: knowledge, Air Magic | Wisdom
- recorded: 20600 experience, level 12, 7/2/4/4, Logistics expert, Offense expert, Wisdom expert, First Aid expert; Wisdom last offered 9, a school 12
- fits: none

Day's points (the land table's 1500):

- the reading's hero (level 12, Logistics expert): single precision 2220, exact 2220
- the recorded hero (level 12, Logistics expert): single precision 2220, exact 2220
- recorded: day 1 2220, day 2 2220
- fits: single precision (the reading, Q-0005), exact (strict-fixed) the arithmetics agree here: inconclusive

