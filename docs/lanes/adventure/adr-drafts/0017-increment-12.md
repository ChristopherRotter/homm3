# ADR-0017 amendment (draft): increment 12, garrisons and the bonus visitables (the game side)

- Status: **Accepted** at T108b's checkpoint (ADV-Q50, 2026-11-11; E08 decisions 305–309), to be
  applied by T112. Drafted by E08 T108b (2026-11-11) from the survey
  `docs/rules/surveys/E08-garrisons-bonuses.md` and the user's answers at ADV-Q48 and ADV-Q49 (E08
  decisions 295, 299–303), then into ADR-0017 through the round's `shared/` pull request (T112)
  with batch E12's outcome.
- Amends: ADR-0017 points 1, 2, 3, 4, 6, 9, 10 and 11 (each text below goes in after the point's
  increment 11 part, and after E09's where a point has one); its consequences, alternatives and
  revision history.
- Detail: `docs/epics/E08-design.md` part D (§13 to §20); the map side is `0016-increment-12.md`,
  the hero's marks `0013-increment-12.md`.

## Context

A garrison of the visitor's team, or one it just took, opens the original's exchange: a modal
dialog of two bars, the hero's army and the garrison's, an owner each (survey §6). The town's two
rows are the same exchange (the town lane's survey `E09-growth.md` §9, §15; E09 decision 67), so
the game builds it once. A garrison's battle is an owned object's or a guarded one's, without magic
on an anti-magic garrison's entrance. The bonus visitables need the day of the week, the week's
draws of the Fountains of Fortune, the Springs' refill and the hero marks' clears at the day's and
the week's start.

## Decision

**Point 1** (one entry point), after increment 11's text: **Increment 12** (garrisons and the
bonus visitables; E08 decision 295) adds **no command**: a garrison is visited by `MoveHero`'s last
step or `Revisit` and crossed by a path of its team; its exchange is the management commands'
(`Manage::Army`), naming the hero's army and **`ArmyRef::MapGarrison(object)`**, a sibling of a
town's `ArmyRef::Garrison(town)`. The adventure's command application takes **the day of the week**
(1 to 7, the calendar's) beside the acting player's stream: every caller passes it.

**Point 2** (actors and viewers), after E09's text: from increment 12 the observation adds to each
**map garrison** a viewer sees (by object: type and anchor) its army at the viewer's level, which
the game derives as a town's garrison's (the owner's team, a hero with Visions at level 3 within
reach of the entrance, else the viewer's towns with a Tavern, at most 2), and its removable flag to
the owner's team; and the types whose kind the viewer's team has marked.

**Point 3** (turn groups and the day), after E09's text: from increment 12 the adventure's **day
step** drops every hero on the map's day marks, its **week step** every hero's week marks (before
the turn's points), refills every Magic Spring's entrance and draws each Fountain of Fortune's
value.

**Point 4** (streams), after increment 11's text: **Increment 12's draws**: a Fountain of Fortune's
value is drawn when the game is built from the setup stream in the first week's walk, and at each
week's start from the game's stream in the weekly objects' pass, each in object order with the
Windmills, the Gardens and the camps (one uniform draw among the kind's values each). No visit of
increment 12 draws.

**Point 6** (pending decisions), after increment 11's text: **Increment 12 adds one decision
kind, the exchange** (GAM-DEC-009; the original's `garrison_exchange_dialog` 0x5D1130): the
visiting hero and the garrison, its owner's alone, holding no one else, closed by its owner's
`Close` and blocking its `EndTurn` until then. While it stands its owner's management commands may
name the hero's army and the garrison's (a third pair of armies held together, beside a meeting's
two heroes and an army dialog's hero and group); each command changes the armies at once, by the
adventure's one owner rule (ADR-0016 point 3 as amended: a non-removable garrison takes stacks
only into its empty slots), so the close writes nothing back. It opens at a visit of a team's
garrison and at the capture of one, and **after a won garrison battle as a sixth visit's rest**
(GAM-DEC-007), given once the winner's levels are answered, as the original's order; nothing when
the winner left the map. Its view (the owner's): the garrison by type and anchor, its army by slot,
its removable flag. The town's rows reuse the kind or its armies held together (E09 decision 67;
the town lane's increment 3), with `ArmyRef::Garrison(town)` or the garrison hero's army as the
stationary bar. The bonus visitables ask nothing.

**Point 9** (versions), a line: increment 12 bumps the game state hash (v28: the exchange, its
rest, the hero details' marks), the adventure state hash (v22), the adventure ruleset part (v20),
the map import (v22) and the adventure replay format (v13: the map garrison's reference in a
management line); the env's action encoding stays v8 (the exchange is answered by the close role).

**Point 10** (outcomes), after increment 11's text: **Increment 12's outcomes**: the exchange is an
outcome the game opens (or holds after a battle); a movement gain, the Sirens' losses and the
Stables' upgrade, the battle bonuses and the timed marks are the adventure's own changes of the
hero on the map and its army (E08 decision 307: the bonuses no longer a hero change); the spell
points and the Sirens' experience are hero changes the game applies through `h3-heroes`, the
experience's levels given as every gain's.

**Point 11** (battles through the game), after increment 11's text: **Increment 12's battles**
(GAM-BAT-001 amended): a garrison's army defends on its entrance unsplit, as an owned mine's
guards (its owner's side without a hero, played outside its turn) or a guarded object's (unowned,
the neutral seat's); its end writes the survivors back, a win flags it and holds the exchange.
**A battle whose tile is an anti-magic garrison's entrance** (the garrison's own, or a hero's
standing in it attacked) sets the setup's anti-magic flag (BAT-TER-002; D-60 the original). The
battle's setup derives each hero's side with **its bonuses on the map** as `derive_battle_hero`'s
input (ADR-0013 point 15.1 as amended); the battle's end has the adventure clear both heroes' battle
bonuses but the once-marks the part names as outliving a battle under the profile (GAM-BAT-014
amended; the Sirens' with `ADV-QUIRK-46` off: A-44, decision 308).

**Consequences**, a line: increment 12 adds one decision kind and its rest (game state hash v28),
the exchange's direct army reference (replay format v13), the day of the week in the adventure's
command application (every caller), the Fountains' draws in the week's pass, the battle bonuses
read from the hero on the map for a battle's setup and cleared by the adventure; the env's
encoding stays v8. Each new event, the decision's view, the
reference and the details' fields break `h3-protocol`'s build until mapped (`cross-lane`).

**Alternatives considered**, a line: increment 12 (ADV-Q50): the exchange as the Leave Guards
dialog's copied group, written back at the close (one rule fewer to share with the town's rows,
but views stale until the close); the exchange without a decision (any management command naming
a garrison the hero stands in, which the original's modal dialog does not allow); a new action for
the exchange in the env; the day of the week kept in the adventure's state; the day's and the
week's clears for every hero of the store (the marks in the details).

**Revision history**, a line: points 1, 2, 3, 4, 6, 9, 10 and 11 amended for E08 increment 12,
garrisons and the bonus visitables: no command, `ArmyRef::MapGarrison` and the day of the week in
the adventure's application; a map garrison's view by the viewer's level and the kind marks in the
observation; the day's and the week's drops, the Springs' refill and the Fountains' draws; the
exchange as a decision kind and a visit's rest; the versions; the outcomes; a garrison's battle and
the anti-magic flag, the bonuses from the hero on the map, the clear's kept marks. Drafted in E08 T108b (survey `E08-garrisons-bonuses.md`);
batch E12's outcome folded in.
