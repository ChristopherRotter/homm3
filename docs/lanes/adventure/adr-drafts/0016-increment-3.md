# ADR-0016 amendment draft: monsters, guarded tiles and objects, the split, what a viewer sees

- Status: Accepted by the user at ADV-Q13 (2026-10-13; drafted in E08 T17b, revised after the
  review), to be applied by T18a; amends ADR-0016 (Accepted 2026-10-10, amended 2026-10-11 and
  2026-10-12)
- Deciders: user + adventure lane (E08 increment 3: decisions 19–27; ADV-Q11 to ADV-Q13)
- Input: survey `docs/rules/surveys/E08-battles.md`; working detail `docs/epics/E08-design.md`
  (§3–§7). On acceptance a `shared/` pull request writes the points below into ADR-0016 and
  adds the revision line at the end.

<!-- Decisions only. Each block replaces or extends the named point of ADR-0016; text not named
     here stays as it is. The game's side of the same seam (offers, battles, outcomes) is the
     ADR-0017 amendment draft beside this one. -->

## Context

Increment 3 makes wandering monsters, guarded piles and enemy heroes things a visit can meet
(decisions 19 and 22). The original decides a monster's answer from the hero's army, Attack,
Defense and Diplomacy, the game's difficulty and the monster's hidden aggression (survey §3), and
splits it into stacks by a generator seeded from its tile (§4.1). A monster guards the tiles
around it (§2.1). Increment 2's object model (types with a closed kind, two visit properties,
contents drawn at setup) carries all of it.

## Decision

**Point 1, extended (per-object state).** Per-object state gains a guarding object's **army
left**: a monster's count, a guarded object's guards, written when a battle it survives ends
(its static part keeps what setup drew). A monster keeps the survivors of all its stacks, an
upgraded stack's as its own creature (A-6 fixed, decision 23). Heroes on the map keep their
army's **formation** with the army (`HeroDetails` holds no army, ADR-0013 point 1).

**Point 3, amended (a hero's details).** Its last sentence becomes: a hero's details are
`h3-heroes`' model (the ADR-0013 amendment draft), kept in the game's store; the adventure reads
them by reference (the game passes the store to its commands and queries) through `h3-heroes`'
derivations, and returns what a visit changes in a hero as typed hero changes in its outcome,
which the game applies (ADR-0017 points 1 and 10; no adventure profile, decision 25). The
adventure owns the hero on the
map and its army; a battle's outcome reaches them through the game's infallible application
(ADR-0017 point 11).

**Point 4, extended (increment 3's kinds and rules).** Its bullet "A tile holding a hero still
blocks other heroes" becomes: a tile holding another player's hero is visited from a neighbouring
tile, and the visit is a battle at once (survey §7.1); an own hero's tile still blocks (meetings
and the army exchange come with increment 4's army management); a hero's own tile is always
leavable. New:
- **`Monster`** (classes 54, 71–75, 162–164): a level range for a random monster (the type's),
  else the creature as instance data (class 54's subclass, mapped by the import); a count
  (drawn at setup from the creature's adventure-map range when the map gives none); a
  disposition, whose aggression is drawn at setup; never flees; does not grow (increment 5); a
  treasure of resources. Visited from a neighbouring tile, also from above (survey §2.3). It
  **guards** the eight tiles around its visitable tile on its level, of its own water status (the
  original's 3 × 3 scan and its order, survey §2.1): a step onto a guarded tile ends the move and
  visits the guard; a path continues past no guarded tile but its start.
- **The monster's answer** (ADV-MON-007) is the adventure's rule: its inputs are the visiting
  hero's army and details (Attack, Defense, Diplomacy's join bonus, through `h3-heroes`), the
  monster, and the game's difficulty (a static input of the
  adventure's setup); its outcomes are a battle or an offer (join, join for gold, pursue; survey
  §3.2); the monster leaves the map after a declined pursuit, or when it joins. A group that joins
  becomes one stack of the hero's army (with a stack of its creature, else in a free slot), its
  treasure's resources go to the owner; with neither, the join offer is skipped as declined (a
  feature to-do, decision 22).
- **The split** (ADV-MON-009): a monster's stacks in its battle follow from its count and the
  visitor's army by the original's table and two draws; in `faithful` the draws are the
  original's generator (the C runtime's, seeded from the tile, survey §4.1) written as the rule,
  so the split is a function of the tile and the armies (`ADV-QUIRK-6`, fix-optional, decision
  26); in `strict-fixed` they come from the attacking player's stream.
- **Guarded objects** (ADV-PIK-004): a pile's guards record from the map (up to seven stacks and
  an optional question); without guards a plain pile. A question is an offer, no question a
  battle at once; the guards fight unsplit; after a won battle the pile is picked up.
- **`Ground`** (Cursed Ground, Magic Plains and SoD's special grounds): no visit; a battle on one
  of its tiles is fought on its battlefield (E04c's order, ADR-0017 point 11).
- Random contents of increment 3 are drawn at setup, in object order, from the setup stream
  (ADR-0017 point 4). The import refuses what increment 3 cannot represent: a treasure with an
  artifact, a count above 4 095 (the original keeps 12 bits).

**Point 6, extended (what a viewer sees).** Of a monster: its creature and its count as the
original's word (few, several, … legion), never the exact count, aggression, flags or treasure.
Of another player's hero: identity, owner, class and its army's creatures with their words. Of a
battle: nothing but its players' (its side's observation, ADR-0013 point 9); a battle's effects
reach other viewers as removals (a monster, a hero) where they see the tiles. The viewer
knowledge list in `docs/06-security.md` gains these rows when they are built.

## Consequences

- Rules keep reading kinds: the import's class mapping stays the only place that knows class
  numbers; the split's generator lives in its rule, not the kernel.
- In `faithful` a monster's split needs no player's dice: a visit of the same tile with the same
  armies splits the same way (the original's fixed split, survey §4.1); in `strict-fixed` it costs
  the attacker two draws. The battle's own seed is the attacker's draw in both (ADR-0017 point 4).
- The adventure ruleset part (v6) gains the monster and ground kinds; the import (v4) types the
  monster classes and guarded piles, lifting D-6's `Unsupported`.

## Alternatives considered

- The split from the tile in every profile: predictable, so an attacker tunes its army to the
  split (the user, ADV-Q13); from a stream in every profile: unlike the original's fixed split.
- Monsters as their own context: they are map objects with a visit; the battle is the game's.
- Showing a monster's exact count: the original shows the word only (survey §9).

## Revision history line (for ADR-0016)

- 2026-10-1x: points 1, 4 and 6 extended and point 3 amended for increment 3: a guarding object's
  army left, the heroes' formation, a hero's details read by reference and changed through hero
  changes, enemy heroes visited from a neighbour as a battle, the monster and ground kinds,
  guarded tiles, the monster's answer, the split by the profile (ADV-QUIRK-6), guarded piles, what
  a viewer sees of monsters, heroes and battles (E08 T17b, the user at ADV-Q12 and ADV-Q13; survey
  `E08-battles.md`).
