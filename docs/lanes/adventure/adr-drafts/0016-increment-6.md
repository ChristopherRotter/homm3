# ADR-0016 amendment (draft): a hero's army, its formation and tactics flag; meeting an own hero

- Status: Proposed (E08 T44b, 2026-10-23), from the answers at ADV-Q24 (the epic's decisions
  93–100). **Accepted by the user** (ADV-Q25, 2026-10-23; decisions 102–105). The round-1 `shared/` task applies it to
  ADR-0016.
- Amends: ADR-0016 points 3, 4 and 6.
- Detail: `docs/epics/E08-design.md` part B (§7–§10); facts: survey
  `docs/rules/surveys/E08-army.md` §1, §2.1, §3, §4.

## Context

The adventure owns the hero on the map and its army (point 3). Increment 6 lets the owner
rearrange that army, within a hero and between two of its heroes that meet, set its formation
and the original's tactics formation flag, and make room for a joining group that finds none
(`ADV-STUB-2`). The original's rules for a hero's last stack, its split dialog and its dismissal
are survey §3's; its count moved through 16 bits is A-15 (fixed, decision 98). A step toward an
own hero is refused today; the original meets it (§2.1).

## Decision

3. *(amended)* **A hero on the map** holds its army (at most the ruleset's seven slots, at least
   one stack), its **formation** (loose or tight) and its **tactics flag** (on at the start, the
   original's +0x48 = 2), besides its tile and points. Its owner **manages the army**: a stack
   moved onto another slot merges into one of its creature, fills an empty slot or swaps; a
   split leaves both stacks non-empty (the original's dialog's bounds, A-16's included, refuse
   only a split's 0 or full count, which a move covers where the original's click allows it:
   decision 98);
   a stack is dismissed while its army keeps another. Between two armies a hero's last stack
   never merges away nor moves to an empty slot (it may swap). The armies are the hero's own,
   the two heroes of a meeting, or the hero and the group of an army dialog (a small army of
   the decision, seven slots, no hero, none of whose stacks is protected); counts keep their full
   width (A-15 fixed). The formation and the flag change at no cost. The adventure validates and
   applies these commands; the game routes them and checks the turn and the decision
   (ADR-0017's draft).
4. *(amended, the visits)* A hero visited from the neighbouring tile (a step toward it, at the
   step's price, the move ending) is attacked when another player's (GAM-BAT-004) and **met**
   when its owner's: the outcome reports the meeting, which the game opens as its owner's
   decision. The cases of an object under the visited hero and of tops not visited from above
   stay as for the battle's visit. A join that finds no room is offered as any join; accepted, it
   reports the group, which the game holds as the army dialog's decision (ADV-MON-008 amended).
6. *(amended, visibility)* A hero's army, formation, flag and details are its owner's only, so
   every management change, a meeting and an army dialog are told to the owner alone; another
   player learns of a meeting only the step it sees (a step is a step), and of the group only the
   monster's leaving where it sees the tiles.

## Consequences

- `HeroOnMap` gains the flag; the adventure state hash and its version change.
- The army commands touch only armies; a meeting's artifact moves are the store's (ADR-0013's
  draft), so the game checks both before applying either.
- The original's human-only reading of the flags (the formation and the tactics phase for a
  human player's side, survey §4) has no counterpart: the engine has no computer player
  (decision 16); E10 decides for its computer players.

## Alternatives considered

- **Exchanges whenever two heroes stand side by side** (no meeting): the original exchanges only
  in the meeting a step opens, which costs the step's points (not chosen at ADV-Q24).
- **One answer naming what to dismiss and where the group goes**: covers a fraction of the
  original's dialog (not chosen).
- **VCMI's split bounds or a flag for A-16** (not chosen).
