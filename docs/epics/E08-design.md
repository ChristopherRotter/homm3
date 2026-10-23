# E08 design note: between increments 5 and 6

A handover, not binding (`docs/adr/README.md`). The task that builds a section deletes it; the
file goes when the epic closes.

**Increment 5's note** (the hero's movement and scouting) was moved by T43 into `E08-log.md`
("Increment 5's design note at its closure"): every section was built (T35 to T42), and the
audit of its exit criteria (§12 there) found them met. Two code comments still cite it as
"design note" by section (`crates/h3-adventure/src/state/passage.rs`, §7's table;
`crates/h3-sim/src/adventure/tests.rs`, §8); the lane's next code change repoints them to the log.
What increment 5 left out is in survey `docs/rules/surveys/E08-hero-movement.md` §6 and the
epic's sketch.

**Increment 6's note** (army and hero management; the epic's decisions 90, 91) is written by T44b
from T44a's survey and the answers at its checkpoint (ADV-Q24): part A the hero side, part B the
map, the game, the batch and the cut, with increment 6's exit criteria.
