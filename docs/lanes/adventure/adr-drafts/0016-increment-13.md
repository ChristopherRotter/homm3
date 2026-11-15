# ADR-0016 amendment (draft): increment 13, creature banks (the map side)

- Status: **Draft** (E08 T118b1, 2026-11-15), for the checkpoint after T118b2 (decision 334: the
  user accepts the drafts before anything is built). Drafted from the survey
  `docs/rules/surveys/E08-banks.md` and the user's answers at ADV-Q52 and ADV-Q53 (E08 decisions
  333, 335 to 339), then into ADR-0016 through the round's `shared/` pull request with batch E13's
  outcome.
- Amends: ADR-0016 points 1, 4 and 6 (each text below goes in after the point's increment 12
  part); its consequences, alternatives and revision history. Points 2, 3 and 5 are unchanged: a
  bank adds no grid and no reveal, and its once-marks are the hero on the map's list as built
  (ADR-0013 point 1 as amended for increment 12).
- Detail: `docs/epics/E08-design.md` parts A to C (§1 to §12). The game's side (the bank battle's
  setup, the reward before the winner's levels, the env, batch E13) is part D and
  `0017-increment-13.md` (T118b2), with the battle seam's draft.
- ADR-0015 needs no amendment: no crate; the import draws nothing, the build draws a bank as it
  draws increment 7's objects. ADR-0013 needs none either: the three once-marks are content.

## Context

Increment 13 types the creature banks, inert blockers until now: class 16 by its subclass (the
Cyclops Stockpile, the Dwarven Treasury, the Griffin Conservatory, the Imp Cache, the Medusa Stores,
the Naga Bank and the Dragon Fly Hive), the Shipwreck (85), the Derelict Ship (24), the Crypt (84)
and Dragon Utopia (25): 2 137 objects on 148 of the 160 shipped maps, eleven kinds. Each holds a
level drawn when the game is built (its guards, split into stacks, one of them perhaps upgraded,
and its reward: resources, a creature, artifacts); a visit asks to attack (or, for the Crypt and
the two sea banks, to search first), the guards fight at once in a battle of their own setup, and
a won bank gives its reward and stays on the map looted. The user chose the original in every
dispute (D-65 to D-68; E08 decisions 336, 337), D-67's reward before the winner's levels among
them.

## Decision

**Point 1** (storage), a part **Increment 13's objects** (E08 decision 333): **no new block.**

- **What the build drew** for a bank is the static map's: the level's index and the artifacts in
  the draw's order (relics first), beside every other object's drawn contents; the level's
  resources and creature are read from the kind by the index, never copied.
- **Its guards as drawn** (split and upgraded) are the object's guards, as a guarded pile's, in
  the static map; a battle's survivors are its **army left** in increment 7's block (none after a
  draw).
- **Looted** is its **emptied** mark in increment 7's block, as a looted Pyramid's: a bank never
  leaves the map.
- **A team's visit** is the per-player **knowledge** of objects in increment 7's block, set for
  every player of the visitor's team.

**Point 4** (kinds), a part **Increment 13's kinds** (the import maps class 16 by subclass 0 to 6
and 85, 24, 84 and 25 at subclass 0; another subclass is refused, no body is read):

- **The bank** (`bank`; ADV-BNK-001): one kind for the eleven, all in the base pack, with its
  **question** (attack: class 16 and Dragon Utopia; search: the Crypt and the two sea banks), one
  to four **levels** as data (a chance, one to four guard creatures with counts, an upgrade chance,
  resources, a reward creature and count, artifact counts by class: the original's `CrBanks.txt`
  with the executable's creature tables, D-65 the original), a search kind's **looted**
  once-bonus, the sea banks' **deck** and the Shipwreck's **coast**. Its visit mode is the
  original's class lists' (E08 decision 317's rule): **the Shipwreck visited from a neighbour**
  and from above, **the ten others entered** (the hero stands on the entrance), none from above.
  Not ownable, not weekly.
- **The draw** (ADV-BNK-002), when the game is built, from the setup stream in object order with
  increment 7's rewards: the level (one draw over the chances); the artifacts, **relics first**,
  then majors, minors and treasures (D-68 the original), from the one pool of the random
  artifacts, chests and finds; the guards split by the original's rule for the level's number of
  creatures (five stacks for one, the Crypt's two fixed patterns, none from four on); the upgrade
  of the rule's slot by its chance, for a creature with an upgrade. Never again.
- **The visit** (ADV-BNK-003; D-66 the original): an attack kind marks the visitor's team at once,
  tells a looted one so and does nothing more, else asks (a pending offer of the visitor's owner;
  declined, nothing more); a search kind asks first, looted or not, and only a yes marks the team;
  a looted one's search gives its once-bonus (−1 morale until the end of the hero's next battle,
  once per kind while the mark is held). `Revisit` visits an entered bank again under these
  rules.
- **The battle** (ADV-BNK-004): a yes fights at once the guards as they stand, unowned and without
  a hero, at the entrance's tile; the battle is marked as a bank's by its cause (the visit's
  yes), never by the tile, so a battle between heroes on an entered bank's entrance is ordinary;
  the sea banks' on the sea terrain's deck, from the coast too. Lost or retreated, the survivors
  stay as its guards and it is not looted; with no guards standing, a yes gives the reward at
  once. The battle's setup and its rules are ADR-0017's and the battle seam's (part D).
- **The reward** (ADV-BNK-005; D-67 the original): won, the reward comes with the battle's end
  itself, **before the winner's levels** (ADR-0017's order, part D): the creature onto the army or
  into the army dialog, the artifacts in the record's order (relics first), the resources; then
  the bank is looted.
- **The Shipwreck's passage** (ADV-MOV-009, ADV-SEA-002 amended; ADV-STUB-12 lifted): its water
  entrance is, for a hero not in a boat, a step from a stop node on land (the visit from the
  coast) and a flight's pass from land or water, as an empty boat's tile (the original's class 85
  tests); a hero in a boat visits it from the water. The Derelict Ship stays a sea object visited
  only from a boat (D-46).

**Point 6** (information), after increment 12's text: from increment 13 every viewer that sees a
**creature bank** sees its type, and by its team's visit mark what the original's hint shows:
not visited; visited and looted or with no guards standing; or the word of the guards' total
count with their creature when every stack is of one (as a mine's guards, without their stacks).
The mark and that view are live for every team that visited (a battle's survivors, the loot). The
level, the counts, the upgrade, the artifacts and the reward stay hidden until the battle and the
win. A looted visit, the offer, the reward and the once-bonus are told to the visitor's owner
alone; a team's new marks to its players; the hint's change to the players whose team visited
it. `docs/06-security.md` lists it (the round's `shared/` task).

**Consequences**, a line: increment 13 makes the five classes typed objects (adventure ruleset part
v21, map import v24, adventure state hash v26: the banks' draws and guards in the static map), so
on 148 of the 160 shipped maps their entrances become destinations (ten kinds entered, the
Shipwreck visited from a neighbour or the coast) and a flight crosses a Shipwreck's water tile.

**Alternatives considered**, a line: increment 13 (ADV-Q53): VCMI's Naga Bank gems and an Imp
Cache without its 2 Mercury (D-65), a mark at every visit and a looted search's morale without a
question (D-66), the artifacts treasure first (D-68); the banks' visit modes from the templates'
blocked entrance (the survey before T118b1); the levels' stacks pre-split in the data (VCMI's form)
instead of the original's split rule; two kinds (attack, search) instead of one; a bank offer kind
beside `Fight` and `Search`; the bank battle marked by its tile, as the anti-magic garrison's.

**Revision history**, a line: points 1, 4 and 6 amended for E08 increment 13, creature banks: the
banks' draws, guards, loot and team marks in the existing blocks; the kind bank with its question,
levels, looted once-bonus, deck and coast, its draw, visit, battle start and end, its reward before
the winner's levels, the Shipwreck's coast passage; what a viewer sees of a bank by its team's
visit. Drafted in E08 T118b1 (survey `E08-banks.md`).
