# ADR-0017 amendment (draft): increment 11, the teleporters (the game side)

- Status: **Accepted** at T98b2's checkpoint (ADV-Q46, 2026-11-06; E08 decision 273), to be
  applied by T102. Drafted by E08 T98b2 (2026-11-06), from the survey `docs/rules/surveys/E08-teleporters.md`
  and the user's answers at ADV-Q44 and ADV-Q45 (E08 decisions 260, 264–269); for the user's
  acceptance at T98b2's checkpoint (ADV-Q46) before anything is built (decision 261), then into
  ADR-0017 through the round's `shared/` pull request with batch E11's outcome.
- Amends: ADR-0017 points 1, 4, 6, 8, 10 and 11 (each text below goes in after the point's
  increment 10 part); its consequences, alternatives and revision history.
- Detail: `docs/epics/E08-design.md` part C (§12 to §18); the map side is
  `0016-increment-11.md` (parts A and B, and `Revisit`'s lines in its point 4).

## Context

Increment 11's jump is a visit (ADR-0016's draft, point 4): a step onto an entrance ends the move
and moves the hero to an exit, drawn for a monolith or a Whirlpool, the pair for a gate. Three
things reach the game: the draw (the first visit that draws, so the adventure needs a stream), an
enemy hero on the exit (fought first, the jump given only after a win and the winner's levels, as
the original's order), and the original's space key, which visits the entered object under the
hero again without points (E08 decision 268).

## Decision

**Point 1** (one entry point), after increment 10's text: **Increment 11** (the teleporters; E08
decision 260) adds one command, **`Revisit { hero }`** (an adventure command; E08 decision 268,
the original's space key): the entered object under the player's hero visited again, as the step
onto it visits it, without points spent or required, without a step and without a guard test. The
player's turn must be running with no open decision; the hero must be the player's and stand on
the visitable tile of an entered object whose kind has a built visit (one typed refusal for every
other case). A jump is a step of `MoveHero` (its last step's visit) and needs no command; no
command names an exit (D-53). The setup gains nothing: the channels are the static map's, built
with the game from its objects (ADR-0016 point 1).

**Point 4** (streams), after increment 10's text: **increment 11's jump** is the first draw a
command causes inside the adventure: a monolith's or a Whirlpool's exit is one uniform draw among
its candidates from **the acting player's stream** (the visitor's, ADR-0016 point 4), none for a
single candidate (the original's), none for a gate. `AdventureState::apply` takes that stream from
the game for every adventure command and draws only after the whole command is checked, so a
rejected command leaves the stream untouched. In a command the jump's draw comes before the draws
of the battle it opens (the battle's seed, then under `strict-fixed` its obstacles). The answers
and the rests draw nothing (a held jump's exit was drawn at its visit). Both profiles draw alike:
the original's exit comes from the C runtime's one `rand()` stream, which no profile reproduces.
No setup placeholder is added and none moves (the channels and the gates' pairs are built without
a draw).

**Point 6** (pending decisions), after increment 10's text: **Increment 11 adds no decision
kind.** A battle at a teleporter's exit is a hero battle (point 11), a hero of the team on a
gate's pair is increment 6's **meeting** (its owner's alone; the visitor stays on the entrance and
the two heroes are not neighbours), and `Revisit`'s visit opens what the object's visit opens.
**The jump after a won battle at an exit** is a fifth **visit's rest** (GAM-DEC-007): held by the
game when the battle opens and given once the winner's levels are answered, as the original moves
the hero after the levels; nothing when the mover left the map. A visit that jumps, or holds a
jump, holds no guard's visit of the entrance (subject to batch E11's E11-19).

**Point 8** (`AdventureEnv`), after increment 10's text: increment 11's **action encoding v8**
appends eight entries, the `Revisit` of the hero in each of the eight hero slots (the move entries'
slots), masked by its legality; a jump needs no entry (a destination among the reach's tiles). The
observation the env returns gains the teleporters' types in the objects' views (`GameObservation`
as it is, E08 decision 187). The change is the AI lane's (point 8's ownership), made `cross-lane`
in the game's task with the user's consent, as increments 7's, 9's and 10's. A revisit at no
points can repeat within a turn: the rollout's step cap is the AI lane's.

**Point 10** (outcomes), after increment 10's text: **Increment 11's outcomes:** a Whirlpool's
loss is the adventure's own change of the hero's army (told by its event to the owner alone), no
loss of resources; **the jump to hold** (the hero and the exit) beside the battle at an occupied
exit, given by the adventure's infallible arrival when the rest runs; a gate's meeting is the
outcome's meeting. No price, no gain.

**Point 11** (battles), after increment 10's text: **Increment 11's battles** (GAM-BAT-004
amended): an enemy hero on a teleporter's exit (a monolith's, a gate's, a Whirlpool's drawn tile:
A-37 fixed so, E08 decision 265) defends as any hero against the mover, who attacks from the
entrance: the battle at the defender's tile, the exit, with its terrain, special ground and
background, the attacker's ground the entrance's; at a Whirlpool both heroes are in boats, so on the
two boats. The tiles within 1 of the exit are revealed to the mover's owner first (ADR-0016 point
5). Won: the defender leaves the map, the winner's levels, then the held jump (point 6). Lost,
retreated or surrendered: the mover leaves the map (in a boat with it), the jump dropped; a draw,
both leave.

**Consequences**, a line: increment 11 gives `AdventureState::apply` the acting player's stream
(every caller passes it; a rejected command leaves it untouched), a fifth rest (game state hash
v26), one command (adventure replay format v11) and eight env entries (action encoding v8, the AI
lane's); the space key's revisit repeats at no points, so the env's rollout needs a step cap.

**Alternatives considered**, a line: increment 11 (ADV-Q45, ADV-Q46): the exit named by the command
(VCMI's, D-53), so no draw; the exit's draw from the game's stream or a stream inside the adventure
state; the jump kept in the open battle and given at its end, before the winner's levels; the
revisit for the teleporters only, or later as a stub (decision 268); the revisit with the guard's
test; one env entry for the selected hero instead of one per slot.

**Revision history**, a line: points 1, 4, 6, 8, 10 and 11 amended for E08 increment 11, the
teleporters: the command `Revisit`; the jump's draw from the acting player's stream, passed to the
adventure's `apply`; no decision kind, the jump after a won battle at an exit a visit's rest, a
gate's meeting; action encoding v8; the jump held as an outcome; the battle at an exit. Drafted in
E08 T98b2 (design part C); batch E11's outcome folded in.
