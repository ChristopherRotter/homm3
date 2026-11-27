# Draft: ADR-0016 amended for E10 increment 1 (the teams and the game's end)

Status: **Proposed** (E10 T1b, 2026-11-27; E10 decisions 2, 10, 15 to 17, ADV-Q65). Applied by the
`shared/` task (T2) as accepted; as built by T7. Design note: `docs/epics/E10-design.md` §2 to
§7, §11.

## Point 1 (three kinds of storage), amended

> E10 increment 1: the **teams** (a team per player, from the setup) are static: in the shared
> static part, hashed once; `AdventureState::allied(a, b)` reads them (a player with itself
> included). The explored layers are kept **per team** (point 5).

## Point 4 (map objects), amended

> E10 increment 1: "a team is one player" ends; every rule naming the team reads the teams (an
> ally's ownable not flagged, a mine's Leave Guards its owner's alone, an ally's dwelling's dialog
> read-only, an ally's Shipyard's boat, a team's garrison exchanged and passed, the visit marks the
> team's, a monolith's exit holding a hero of the team no candidate). A step toward a hero of the
> team **meets** it (ADR-0017 point 6), an enemy's is a battle as before. A town of the team is
> entered, never besieged; the siege of a town whose visitor is the owner's ally follows the
> original as read by E10 T3c (no longer a refusal). A **defeated player's** Shipyards, mines,
> Lighthouses, dwellings and garrisons become neutral (a mine's guards and a garrison's army kept;
> ADV-OWN-004), each told as freed, and its heroes leave the map (a boat with them), by an
> infallible step the game calls (ADR-0017 point 1).

## Point 5 (fog of war), amended

> E10 increment 1: **each team has an explored layer**: every reveal (a step's, a placement's, a
> flag's, a town's, a jump's, a door's, the Cartographer's, `strict-fixed`'s new day's, the failed
> casts' tiles) writes its team's, every hide clears every team's but the town owner's; a viewer
> reads its team's, the fringe its team's. "Shared vision of allies comes with teams" is built.

## Point 6 (information hiding), amended

> E10 increment 1: a viewer sees a hero of its team at the popup's top level (the exact army, as
> Visions' level 2), a town and a garrison of its team at level 3, a mine of its team's resource;
> a player whose hero enters an ally's town learns its buildings at that visit (built). A freed
> ownable is told to the players that see it and to its old owner.

## Alternatives considered (added)

- E10 increment 1: an explored layer per player with each reveal written to every member (the
  original's bits; the same views with fixed teams, n writes and n layers hashed).

## Revision history (line to add)

- 2026-11-?? (E10 T2 `shared/`; E10 decisions 2, 10, 15 to 17, ADV-Q65): points 1, 4, 5 and 6
  amended for E10 increment 1, the teams and a defeated player's objects. Drafted in E10 T1b
  (design note `E10-design.md`); as built at T7.
