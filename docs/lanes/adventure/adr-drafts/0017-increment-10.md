# ADR-0017 amendment (draft): increment 10, dwellings and hiring (the game side)

- Status: Draft (E08 T88b2, 2026-11-03), from the survey `docs/rules/surveys/E08-dwellings.md` and
  the user's answers at ADV-Q41 and ADV-Q42 with decision 238 (E08 decisions 231, 235–238);
  **accepted as written** at ADV-Q43 (T88b2's checkpoint, decisions 241 and 243); into ADR-0017
  through T92's `shared/` pull request with batch E10's outcome.
- Amends: ADR-0017 points 1, 3, 4, 6, 8, 10 and 11 (each text below goes in after the point's
  increment 9 and E09 parts); its revision history.
- Detail: `docs/epics/E08-design.md` part C (§12 to §18), on parts A and B. The map's side (the
  kinds, the import, the state, the visits, the week, information) is ADR-0016's draft
  (`0016-increment-10.md`, T88b1).

## Context

Increment 10's dwellings ask two questions at a visit (fight the guards, recruit), fight their
guards as the original does (unsplit, unowned: D-51 as the original, decision 238), give an own
dwelling's level-1 creatures free and open the original's hire dialog (`H3RecruitMgr`), which
stays open for purchases until closed and is bounded by the count and the treasury (gold and at
most one other resource). The Refugee Camp opens the same dialog for any visitor. The random
dwellings are drawn when the game is built, and their draw fixes their footprints, which the
static map's later draws read; the camps draw a creature at the build and every week. E09 reuses
the dialog for a town's recruitment (E09 decision 4).

## Decision

**Point 1** (one entry point), after the E09 text: **increment 10** adds no command: a dwelling's
or a Refugee Camp's visit is a step of `MoveHero` (both kinds are entered), and the free creatures
and the purchases are answers (point 6). The setup gains nothing beyond ADR-0016's (the
dwellings' owners and the random dwellings' faction sets and level ranges come with the map's
objects).

**Point 3** (turn groups), after the increment 9 text: from increment 10 **the adventure's week
step** first refills **every dwelling**, owned or not (each count its creature's growth, set; the
guards the kind's full guards; no week's or month's kind), then draws the weekly objects' contents
and **the camps' creatures** in one pass in object order, then grows the monsters (GAM-CAL-002
amended; ADR-0016 point 4).

**Point 4** (streams), after the E09 text: **increment 10's setup placeholders**: in the static
map's placement walk, in object order, each **random dwelling's faction** (uniform among its set)
then **level** (uniform in its range), after E09's placeholders and **before every other existing
one** (the drawn type's footprint fixes the tiles the later draws read: a Shipyard's boat tile, the
free tiles); a map without random dwellings draws nothing more, so nothing shifts. **A Refugee
Camp's creature** (uniform among the random monsters of its levels, ADV-MON-003's set; its count
the creature's growth, no draw) in the first week's walk with the weekly objects, in object order.
**At a new week** each camp's creature from the game's stream in the weekly objects' pass, in
object order. No visit, question, battle's end or purchase of increment 10 draws.

**Point 6** (pending decisions), after the increment 9 text: increment 10's decisions:

- **A guarded dwelling's question** (another team's or a neutral dwelling whose guards stand): an
  offer naming the guards' first creature and the size word of their total (`Offer(Consent)`,
  [`Decline`]); accepted, the guards' battle (point 11); declined, nothing.
- **The hire question** (a dwelling of the visitor's team, so its own once flagged; a camp with
  creatures, for any visitor): an offer naming the creatures (a camp's hidden until then). Accepted, on one's
  own dwelling the creatures of the part's free level or less join free, all of each or none (no
  room: none, the count kept), then the hire dialog when a creature is above that level.
- **The hire dialog** (GAM-DEC-008; a new kind, its owner's alone, holding no one else, blocking
  its owner's `EndTurn` like an offer), answered **`Hire(HireAnswer)`**: `Buy { creature, amount
  }` and `Close` [`Close`]. A purchase is refused (typed, nothing changed) unless the creature is
  the source's, the amount from 1 to its count, the army has room for it (a stack of the type
  within the stack's most, else a free slot) and the treasury pays its cost times the amount; then
  the price is paid, the count lowered and the creatures join. The dialog stays open until
  `Close`; a dialog of **one** creature closes after its purchase, as the original's. Its legal
  answers are each creature's purchase of **the most** it may buy (the count, the treasury's
  bound and the room's), then `Close`; a query gives the range of amounts from 1 to the most, as
  a split's counts. Its view (its owner's) shows the source by type and anchor and each
  creature's count and cost. An ally's dwelling (E10's teams) opens it **read-only**, `Close`
  alone. E09's recruitment reuses the kind, with a town's dwelling as its source and the hero's or
  the garrison's army as its target.

**Point 8** (`AdventureEnv`), after the increment 9 text: increment 10's **action encoding v7**
appends four entries, each buying the most of the open hire dialog's creature in that place (the
original's four), masked by its legality; the dialog's `Close` is the closing entry, the two
questions the offer's entries. The observation the env returns gains the dialog's view
(`GameObservation` as it is, decision 187). The change is the AI lane's (point 8's ownership),
made `cross-lane` in the game's task with the user's consent, as increments 7's and 9's.

**Point 10** (outcomes), after the E09 text: **increment 10's outcomes**: a visit's free creatures
(the adventure's own change of the hero's army, told by its events); **the hire dialog to open**
(the hero and the source, read-only or not); a won guards' battle's rest (the flag, then the hire
question, after the winner's levels, GAM-DEC-007). **A purchase:** the adventure checks the source,
the amount and the room and names the price (the creature's cost times the amount, by resource);
the game checks the treasury before and pays after, then calls the adventure's infallible
purchase; a rejection leaves everything untouched. The flag is increment 9's.

**Point 11** (battles), after the E09 text: **increment 10's battles** (GAM-BAT-001 amended): a
dwelling's guards defend on its entrance as an army without a hero, unsplit and **unowned**, played
by the neutral seat whoever owns the dwelling (D-51 as the original, decision 238; an owned mine's
guards stay their owner's), on the entrance tile's battlefield (no terrain of the type). Their
survivors stay the dwelling's guards until the week's step (none after a draw); a loss or a
retreat changes nothing else; a win flags the dwelling for the attacker after its levels, then the
hire question.

**Revision history**, a line: the date of the `shared/` pull request (E08 T92, the user at ADV-Q41
to ADV-Q43): points 1, 3, 4, 6, 8, 10 and 11 amended for E08 increment 10, dwellings and hiring:
no command; the week's refill and the camps' draws; the random dwellings' and camps' draws; the
guards' and the hire questions, the hire dialog; the env's v7; the purchase's outcome; the guards'
battle unowned. Drafted in E08 T88b2 (survey `E08-dwellings.md` §4, §5, §6, §7, §12).

## Consequences

- One new decision kind and its answer: the game state hash (v23) and the adventure replay format
  (v9) bump; E09 bumps the same, the second to merge bumps again.
- The dialog's purchase rule is one, the map's (ADR-0016 point 4), checked by the game's treasury:
  E09's recruitment adds a source and a target, not a second dialog.
- The random dwellings' draws come first among the static map's: a map that holds one draws every
  later placeholder differently than before; no recorded game holds one.
- An agent of v7 buys the most of a creature, never less; a finer amount is the AI lane's.

## Alternatives considered

- **Every amount in the legal set** (each creature's 1 to its most): rejected (agent's call): up
  to the count per creature, where the most and a query of the range serve, as a split's counts.
- **An ally's dialog as an event** (VCMI shows none): rejected (agent's call): the original shows
  the disabled dialog, so a client draws one view; it is built with E10's teams.
- **The dialog open after a one-creature purchase:** rejected: the original closes it (survey
  §5.3; batch E10's E10-4 records it).
- **`Answer::Close` for the dialog** (the exchange dialogs'): rejected (agent's call): the kind's own
  answer, as the University's `Leave`, so an answer names its decision.
- **The random dwellings drawn after the existing placeholders:** rejected: their footprints fix
  the tiles the later draws read.
- **The guards on the owner's side** (VCMI): rejected by the user (decision 238; ADR-0016's draft).
