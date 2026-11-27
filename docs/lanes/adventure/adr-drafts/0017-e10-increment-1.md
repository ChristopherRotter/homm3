# Draft: ADR-0017 amended for E10 increment 1 (the teams and the game's end)

Status: **Proposed**, accepted at the checkpoint (E10 T1b, 2026-11-27, decision 19; E10 decisions 9
to 19, ADV-Q65). Applied by the `shared/` task (T2) as accepted; amended as built with batch GAM-1's
outcome by the audit's `shared/` task (T7). Design note: `docs/epics/E10-design.md` §1, §2, §5, §8
to §17.

## Point 1 (one entry point), amended

> **The teams and the game's end** (E10 increment 1): `PlayerSetup` carries the player's team
> (the map's, every colour its own without; checked dense at the build); `GameState` holds per
> player its team, its defeat (the day and why: no hero and no town, or the seven days out) and its
> days without a town (none, or 7 to 0), and the game's **result** once over (the winning team or
> none, and why: a conquest, no human left). The game **checks the end** after every accepted
> command and system command but `EndTurn`, at each turn's end before the outgoing player's
> count-down and at a human's turn start (never the game's first; ADR point 3): the eliminations
> in colour order, then the verdict. A defeat is the game's step over the contexts' infallible
> functions: the adventure frees the player's ownables and takes its heroes off the map, the game
> moves them into the pool (E09's leave) and frees its tavern offers. Once over, **every command
> is refused** (`GameOver`), changing nothing, and no decision is open.

## Point 2 (actors and viewers), amended

> E10 increment 1: the system's closed set gains **`HandOver { player }`** (SRV-Q29's removal, E10
> decision 15): the player's `human` flag cleared, told to every player, then the check; refused for
> a defeated player or a computer; its turn and pieces stay, the host plays the colour with its
> system commands (and, with E31, the original's computer player). No player command removes a
> player. **Views:** each player's team, defeat and whether it is human are public, its days
> without a town its own, the result public; a viewer sees a hero, town or garrison of its team at
> the popups' top level (ADR-0016 point 6).

## Point 3 (turn groups), amended

> E10 increment 1: a **defeated** player leaves the schedule: its turn ends at its defeat (its
> group's ends when the others' have), it never acts again, and it leaves every step that walks
> the players (the day's income, the week's tavern heroes, the Thieves' Guild, the market's
> recipients; the original's readers of its defeated byte). At **each turn's end** the game checks,
> then lowers the outgoing player's days without a town by one when above 0 (a computer's too);
> at a **human's** turn start it checks after the turn's steps (a computer's runs none) and tells a
> player with days its days left. Building the state checks nothing (A-57, `ADV-QUIRK-61`).

## Point 6 (pending decisions), amended

> E10 increment 1: a step toward a hero of the visitor's **team** opens the meeting as the
> visitor's owner's decision, as for an own hero; onto the ally's hero it allows **gifts only**: a
> stack into an empty slot, an artifact into an empty place or the backpack, nothing taken, none of
> the ally's settings changed (typed refusals; D-82 the original's, decision 16); the ally is told
> its hero's changes. An ally's dwelling's hire dialog opens read-only, `Close` alone (built). The
> game's end drops every open decision.

## Point 8 (the env), amended

> E10 increment 1: the env's step reports the game's result and each player's defeat; a defeated
> player or an ended game has no seat to move. The action encoding stays v21 (the hand-over is the
> host's). Who is human is the seat's mark (`PlayerSetup::human`): the verdict's "no human left"
> reads it, and a game built with no human plays to one team.

## Point 9 (versions), amended

> E10 increment 1: the map import v30 (the teams), the adventure state hash v31 (the teams, the
> explored layer per team), the game state hash v41 (the teams, the defeats, the days, the result),
> the adventure replay format v28 (`hand-over`, the end); protocol v86 (the server lane's).

## Consequences (added)

- E10 increment 1: every loop over players must skip a defeated one; the check runs after every
  command, so a rule that changes a hero's or a town's count or an owner needs no call of its own.
  A finished game is a final state: replays, the env and the server stop at its `GameOver`.

## Alternatives considered (added)

- E10 increment 1 (ADV-Q65): VCMI's moment for the seven days (the defeat at the seventh turn's
  end; D-80); playing on after the last human (VCMI's, D-84); a removed player defeated with its
  towns made neutral, or kept by nobody's turn (D-83); merging gifts into an ally's stack (VCMI's,
  D-82); a check when the game is built (A-57); the check only at the original's 37 sites (each
  rule calling it: more code, the same verdicts but a days' defeat's moment).

## Revision history (line to add)

- 2026-11-?? (E10 T2 `shared/`; E10 decisions 9 to 19, ADV-Q65): points 1, 2, 3, 6, 8 and 9
  amended for E10 increment 1, the teams and the game's end. Drafted in E10 T1b (design note
  `E10-design.md`); as built with batch GAM-1's outcome at T7.
