# Draft: ADR-0016 amended for E08 increment 15 (the Prison, the Market of Time, the adventure spells)

Status: **Proposed** (E08 T137b, 2026-11-21; decisions 394, 399 to 403, ADV-Q63). Applied by the
`shared/` task (T140) with batch E15's outcome. Design note: `docs/epics/E08-design.md` §1 to §4,
§7 to §13.

## Point 1 (storage), amended: after increment 14's objects

> - **Increment 15's objects** (E08 decision 394): **no new block.**
>   - **A prison's prisoner** is its drawn content in the static map (its `HeroId`; the hero's
>     details are the game's store); the freeing removes the object (increment 2's occupancy).
>   - **The Market of Time** keeps no state.
>   - **A hero's day spells** are plain per-hero fields (point 3).

## Point 3 (heroes on the map), amended

> From increment 15 a hero on the map holds its **day spells** (`DaySpells`): the Fly and Water
> Walk levels (none, or 0 to 3 by the cast's mastery) and the day's Dimension Door casts. Its
> flying and water-walking levels for its steps are the best of its details' (`h3_heroes::movement`,
> the artifacts') and its day spells'. The adventure's start of the day clears them for every hero
> on the map; boarding clears Fly and Water Walk; nothing else but a cast writes them.

## Point 4 (kinds), amended: after increment 14's kinds

> - **Increment 15's kinds:** `prison` (class 62; visited from a neighbour, by every player; its
>   visit's outcome names the prisoner, the game decides: the eight heroes, the store; freed, the
>   hero is placed on the entrance with a full day's land points, nothing revealed or visited) and
>   `empty` (class 50, the Market of Time: entered, never visited, the move ending on its entrance).
> - **The adventure spells** (the adventure part's `[[spell]]` bindings of five core spells to a
>   closed `AdventureSpell`; `[params.spells]`: the door's points by mastery, the target window
>   9 × 8, the most boats): each cast is checked by the adventure after the game's checks
>   (ADR-0017 point 1) and applied infallibly; a charged failure (Summon Boat's and Scuttle Boat's
>   chance, a boat not found or not created, the door's wrong or hidden target) is an accepted
>   command. Summon Boat's choice by `ADV-QUIRK-53` (the reading's order, or the straight
>   distance on the hero's level), A-49 accepted, `ADV-QUIRK-54` (a re-cast) and `ADV-QUIRK-55` (the
>   door's wrong target charged) fix-optional, the window's extra row fixed (`ADV-QUIRK-56`).
> - **The flight over the unexplored** (`ADV-QUIRK-15`, reclassified fix-optional, E08 decision
>   399): on (`faithful`), the original's: an unexplored tile is passable only flying, never a
>   stop, refused when truly rock. Off (`strict-fixed`), the compromise: an unexplored tile is
>   assumed flyable at a pass's price, never a stop; the move walks step by step, each step
>   revealing; a step whose revealed tile the hero may not pass is not made, the move ending on
>   the last stop node reached, charged the steps made, the rest dropped. Walkers and water-walkers
>   never enter an unexplored tile in either setting.
>
> This amends this point's rule that the legal moves never depend on a hidden tile: it holds
> under `strict-fixed`; under `faithful` the flight's reach, the door's targets (`ADV-QUIRK-51`)
> and Scuttle Boat's (`ADV-QUIRK-52`) read the hidden truth, as the original's (the user's flags,
> E08 decisions 399, 402; E08 decision 7: the AI trains under the profile played).

## Point 5 (fog of war), amended

> From increment 15 three reveals: a door's arrival reveals around it by the scouting radius
> (increment 11's jump); under `strict-fixed` a door's failed cast onto a hidden tile, and every
> Scuttle Boat cast onto a hidden tile, reveal **that tile** for the caster's team (E08 decisions
> 399, 402).

## Point 6 (information hiding), amended

> From increment 15 a viewer that sees a prison sees its type, never its prisoner; a freed prisoner
> arrives as a hired hero does (its owner and the tile's viewers). A hero's day spells are its
> owner's; other viewers see their effects only (a door's jump by its two ends, a boat's leave,
> arrival, creation or sinking by its tiles). A failed cast and its reason are the caster's.
> `docs/06-security.md` lists it (E08 T140).

## Consequences (added)

- Increment 15 (E08 T137b, T140): 89 prisons on 41 shipped maps and the one Market of Time stop
  being inert; the passage's row for unexplored tiles gains its two settings on the reach loop
  (measured at T142); the hygiene property holds under `strict-fixed` only, and its plans run that
  profile.

## Alternatives considered (added)

- Increment 15 (ADV-Q63): the flight kept fixed (no unexplored tile ever entered; A-13's option A)
  and the original's alone (option B, no flag); the door's and Scuttle Boat's hidden targets refused
  (explored tiles only); a failed door revealing nothing; Summon Boat by VCMI's whole rule (any
  owner, no last boat first); the door's window with the extra row kept; the Market of Time as an
  inert blocker with an entrance (no kind).

## Revision history (line to add)

- 2026-11-?? (E08 T140 `shared/`; E08 decisions 394, 399 to 403, ADV-Q63): points 1, 3, 4, 5 and 6
  amended for increment 15, the Prison, the Market of Time and the adventure spells. Drafted in
  E08 T137b (design note `E08-design.md` §1 to §13); batch E15's outcome (T139) folded in.
