# ADR-0016 amendment (draft): increment 10, dwellings and hiring (the map side)

- Status: Draft (E08 T88b1, 2026-11-03), from the survey `docs/rules/surveys/E08-dwellings.md` and
  the user's answers at ADV-Q41 and ADV-Q42 with decision 238 (E08 decisions 231, 235–238); for
  the user at ADV-Q43 (T88b2's checkpoint), then into ADR-0016 through a `shared/` pull request.
- Amends: ADR-0016 points 1, 3, 4, 5 and 6 (each text below goes in after the point's increment 9
  and E09 parts); its revision history.
- Detail: `docs/epics/E08-design.md` parts A and B (§1 to §11). The game's side (the hire dialog
  as a pending decision, the guards' battle, the new week's order, the draws) is ADR-0017's draft
  (T88b2).

## Context

Increment 10 adds the creature dwellings (classes 17 and 20: 2 061 objects on the shipped maps),
the random dwellings that take a faction set (216 to 218: 206 in scope) and the Refugee Camp (78:
321). The original keeps a record per dwelling (its creatures, their counts, its guards, its owner),
renews counts and guards every week for every dwelling whoever owns it, flags a dwelling for its
visitor after its guards, lets the owner's visitor take the level-1 creatures free and hire the
others in a dialog, and resolves a random dwelling at the load into the drawn dwelling with that
dwelling's own template, which moves its entrance. A dwelling's guards are its own creatures, never
troops its owner left: the original fights them unowned (D-51), and the user confirmed that rule
(decision 238). ADR-0015 needs no amendment: its import still draws nothing.

## Decision

**Point 1** (storage), a part **Increment 10's dwellings and Refugee Camps** (E08 decision 231):

- **Owners:** the dwellings are ownable objects (point 4), their owners in the owners' block by
  ownable index with the mines', the owners at load the setup's.
- **A dwelling's week:** its counts as they stand and its guards as they stand (the kind's full
  guards, a battle's survivors, or none once beaten) in one block by **dwelling index** (the static
  map's list of dwellings in object order), **none while every dwelling is as its week began**
  (when the game is built and after each week's step, which writes none: every dwelling refilled
  at once); a visit that changes a dwelling makes or copies the block. Equality and the hash read
  the counts and guards as they stand, so no block and a block of full entries are one state
  (point 2). Not the armies on objects: a dwelling's guards are not an army a battle or a dialog
  left on the map but the week's.
- **A Refugee Camp's creature and count:** one block by camp index, drawn when the game is built and
  at every week's step, lowered by a purchase; none for a map without camps.
- **A random dwelling** is resolved when the game is built into the static map: the drawn dwelling
  type, that type's own footprint as its masks, its anchor moved as the original moves it (the
  moved anchor may lie one column past the map's right edge, holding only a free cell); from then
  on it is a dwelling like a placed one.

**Point 3** (heroes on the map), after the Leave Guards text: **an own dwelling's visit opens no
Leave Guards dialog**: a dwelling holds none of its owner's troops (decision 238). Creatures a
dwelling gives (free or hired) join the hero's army where the room rule puts a join (a stack of the
type within the stack's most, else the first free slot; ADV-ARM-007); with no room they do not
join (the free ones stay counted; no merge of the hero's stacks, D-52 as the original).

**Point 4** (kinds), a part **Increment 10's kinds** (the import maps classes 17, 20, 78 and 216 to
218; every one entered, none visitable from above):

- **Dwellings** (`dwelling`: one to four creatures in the original's order, its own footprint;
  ADV-DWL-001, -002): when the game is built and at every new week each count is its creature's
  growth (set, never added; no week's or month's kind applies) and the guards are, for each creature
  of the part's guard level (5) or more, a stack of the guard factor (3) times its growth, in
  creature order (ADV-DWL-002, -003). **The visit** (ADV-DWL-004): one of the visitor's team (an
  ally's from E10's teams) skips the guards and the flag; another team's or a neutral one with
  guards standing asks first (the question shows the first stack's creature and the size word of
  the guards' total), then fights them **unsplit, unowned, without a hero** (the neutral seat
  plays them, whoever owns the dwelling: decision 238; ADR-0017 point 11); a loss, a retreat, a
  surrender or a draw leaves the survivors as its guards until the week's step and its owner
  unchanged; a win clears them and, after the winner's levels, the rest of the visit flags it
  (ADV-OWN-001, the reveal of point 5); an unguarded one is flagged at once. Then the **hire
  question**: on the visitor's own dwelling the creatures of the part's free level (1) or less join
  free, all of each, the count set to 0 (none without room, the count kept); with a creature above
  that level the **hire dialog** opens (ADR-0017 point 6: a pending decision); on an ally's the
  dialog opens read-only. A purchase (ADV-HIR-002) takes a creature of the source, an amount from
  1 to its count and room in the army, lowers the count and adds the creatures; the price (the
  creature's cost times the amount) is the game's to check and pay.
- **Random dwellings** (`random_dwelling`; ADV-DWL-005): the map's faction set and level range
  (217's level and 218's faction normalised into them); when the game is built a faction is drawn
  uniformly from the set, a level uniformly from the range, and the part's table names the dwelling
  type for the faction and level (one per faction and level: the original's unupgraded creature's
  dwelling, the Golem Factory for the Tower's level 3). The import refuses an empty set (A-34), a
  town identifier no town of the map has (A-35), a level past 6 or a lowest level above the
  highest, classes 18 and 19 and a dwelling subclass past its table (A-36): input bounds, no flag.
  A random dwelling tied to a town is a blocker until E09's random towns resolve it (ADV-STUB-13).
  A hero on a random dwelling's visitable tile is refused by the setup (the draw would block it).
- **The Refugee Camp** (`refugee_camp`: its levels; ADV-RFG-001): not ownable, no guards; its
  creature drawn among the random monsters of levels 1 to 7 (ADV-MON-003's set) and its count that
  creature's growth, when the game is built and every week. Any player's visit: a count of 0 is told
  as deserted; else the hire question naming its creature, then the hire dialog; no flag, no free
  creatures.
- **The week's step:** every dwelling refilled before the weekly objects' contents; the camps
  drawn with the weekly objects in one pass in object order; then the monsters' growth (ADR-0017
  point 3). A defeated player's dwellings become neutral, keeping their guards (ADV-OWN-004, with
  E10's defeat).

**Point 5** (fog), after the increment 9 text: from increment 10 a **dwelling's flag** reveals as a
mine's (within 3 of its entrance, to its new owner's team), and the start of the game reveals the
same around every dwelling owned at load.

**Point 6** (information), after the increment 9 text: from increment 10 every viewer that sees a
**dwelling** sees its type (so its name and creatures; a random dwelling's drawn type, footprint and
moved anchor, as the original draws it) and its owner; **never its counts or guards** (the
original's hint and popup show neither). A **Refugee Camp** shows its type only, never its creature
or count, and no knowledge of it is kept after a visit. The guards' question (its creature and size
word), the hire question (a camp's creature), the free creatures joined or refused, the dialog's
counts and costs and the purchases are told to the visitor's owner alone. A flag is seen where its
tiles are, its reveal told to the new owner; the guards' battle is seen as any battle; the week's
refill and the camps' draws are told to no one. `docs/06-security.md` lists it.

**Revision history**, a line: the date of the `shared/` pull request (E08 T88b2's checkpoint, the
user at ADV-Q41 to ADV-Q43): points 1, 3, 4, 5 and 6 amended for E08 increment 10, dwellings and
hiring. Drafted in E08 T88b1 (survey `E08-dwellings.md`).

## Consequences

- The map's dwellings, random dwellings and camps become typed objects: the shipped maps' static
  maps change (map import v16), and so do the golden replays on maps that hold them (T88b2's cut).
- The adventure part gains three kinds, `[params.dwellings]` and the random dwellings' table
  (ruleset part v16); the state gains the dwellings' and camps' blocks (adventure state hash v20).
  E09 bumps the same three; the second to merge bumps again.
- A week's step refills every dwelling by writing one field; a fork costs nothing more for a map
  whose dwellings are untouched this week, one reference count otherwise.
- The hire dialog's purchase rule is the map's, reused by E09's town recruitment (its increment 3)
  for the hero's or the garrison's army.

## Alternatives considered

- **An owned dwelling's guards on its owner's side** (VCMI; the agent's first proposal of a flag,
  decision 234): rejected by the user (decision 238): the owner never puts them there; an owned
  mine's guards are its owner's because Leave Guards puts them there.
- **A dwelling's guards in the armies on objects** (as a mine's): rejected (agent's call): the
  week's refill would have to clear every dwelling's entry of a table shared with the monsters and
  mines, where one field resets them all.
- **A resolved random dwelling keeping the random dwelling's 2 × 2** (no footprints as content):
  rejected by the user (decision 236): the original's entrance moves with the draw.
- **The free creatures merging two of the hero's stacks first** (VCMI's `mergeOnRecruit`):
  rejected (decision 235, D-52).
- **The original's derivation of a random dwelling's dwelling** (the last class-17 subclass listing
  the creature) instead of the part's table: rejected (agent's call): the same outcomes as data,
  checked against the derivation by a test.
