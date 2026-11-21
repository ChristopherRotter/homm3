# Draft: ADR-0017 amended for E08 increment 15 (the cast, the prisoners)

Status: **Proposed** (E08 T137b, 2026-11-21; decisions 394, 404, 405, ADV-Q63). Applied by the
`shared/` task (T140) with batch E15's outcome. Design note: `docs/epics/E08-design.md` §14 to §18.

## Point 1 (one entry point), amended

> **The cast** (increment 15): `GameCommand::Cast { hero, spell, target }`, in the actor's turn
> with no decision open. The game checks, in order, each refusal typed and changing nothing: the
> actor's hero on the map; the spell castable on the map (`h3_heroes::can_cast_on_map`: known, a
> book worn, ADR-0013 point 17); one of the adventure part's spells; the ground (the special
> ground's battlefield's highest spell level, an anti-magic garrison's tile); the cost
> (`map_spell_cost` at the mastery on the hero's tile's ground, the raise from the battle part)
> against the spell points; then the adventure's checks (ADR-0016 point 4). The adventure applies,
> the game sets the spell points less the cost (at least 0). **The prisoners** (increment 15):
> the build stores each prisoner's details as a placed hero's (its levels at load, army, skills,
> artifacts, spells, then its spell points), in no player's list, its type `Prisoner` in the pool
> (E09's). A prison's visit outcome names it; the game frees it unless the visitor's player has the
> town part's most heroes on the map (garrisons not counted, as the hire): its colour takes the type,
> the hero joins the end of the player's list with its day's land points.

## Point 4 (streams), amended

> Increment 15: Summon Boat's and Scuttle Boat's chance draws come from the caster's stream, one
> draw each, in the cast's order (the draw made at 100 % too, as the original's).

## Point 8 (the env), amended

> Increment 15's action encoding v18 (the AI lane's, `cross-lane`): three casts without a target
> and two 19 × 17 windows around the hero (Dimension Door, Scuttle Boat); the masks from
> `legal_commands`, which lists each castable spell after the hero's moves (every offered target
> of the two windows in row order). The own hero's observation gains its day spells.

## Point 9 (versions), amended

> Increment 15: the adventure ruleset part v25, the map import v29, the adventure state hash v28,
> the game state hash v39, the adventure replay format v24 (`cast`), the action encoding v18 (the
> AI lane's), protocol v79 (the server lane's); the core ruleset hash for the quirk flags (shared).

## Point 10 (outcomes), amended

> Increment 15's outcomes: a prison's visit names its prisoner (the game frees it or reports the
> eight; no decision); a cast's outcome names its effects (a boat moved, created or sunk, a hero
> relocated, the day spells set) or its charged failure with its reason; the spell points are the
> game's change (the hero's details), never the adventure's.

## Consequences (added)

- Increment 15: the cast is one command with an optional target, validated in the game and the
  adventure before anything changes; a charged failure is an accepted command, not a rejection, so
  a rejected cast still leaves the state untouched.

## Alternatives considered (added)

- Increment 15 (ADV-Q63): the cast as an adventure command validating the spell points itself
  (the adventure would hold or copy the hero's mana and the raise: rejected by ADR-0013 point 2);
  a target dialog as a pending decision (the original's modal dialog; nothing waits on another
  player, and a cancel is no command, so one command suffices); Visions as part of increment 15
  (E12 with the information objects, decision 406).

## Revision history (line to add)

- 2026-11-?? (E08 T140 `shared/`; E08 decisions 394, 404 to 406, ADV-Q63): points 1, 4, 8, 9 and 10
  amended for increment 15, the cast and the prisoners. Drafted in E08 T137b (design note
  `E08-design.md` §14 to §18); batch E15's outcome (T139) folded in.
