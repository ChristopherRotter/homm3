# ADR-0017 amendment (draft): the hero's movement and scouting in the game's build and day

- Status: Draft (E08 T34b, 2026-10-22), from the user's answers at ADV-Q20 (the epic's decision 75)
  and at T34b (decision 77); put to the user with the ADR-0013 and ADR-0016 drafts and the cut
  (ADV-Q21), then into ADR-0017 through a `shared/` pull request.
- Amends: ADR-0017 points 1, 3 and 9.
- Detail: `docs/epics/E08-design.md` part B (§5, §9); facts: survey
  `docs/rules/surveys/E08-hero-movement.md` §1.4, §4.2.

## Context

From increment 5 a placed hero's day's points and scouting radius depend on its skills and
artifacts (ADR-0013 point 13), which its levels at load can change. The game's build places the
heroes on the map (with their points and the reveal around them) before it gives the levels at
load, which need the static map's terrain shares (E08 T29). The original gives the levels first:
`placed_hero_setup` (0x4D8B30) sets the custom skills, army and artifacts, the levels, then the
spell points and the movement points, and `placed_heroes_setup` (0x4CAE10) reveals around the hero
after it (0x4CAF74). The start of a day has had no rule so far; increment 5's `strict-fixed`
profile reveals there (ADV-QUIRK-18, decision 77).

## Decision

1. *(amended)* The build gives each placed hero its levels at load before its spell points, its
   **day's points** and the **reveal around it**, which read its details as the levels left them
   (the original's order): the adventure places the heroes, the game gives the levels, then the
   adventure's start of the game sets the points and reveals, reading the store.
3. *(amended)* The adventure's start-of-turn step reads the store of heroes (the day's points with
   each hero's bonuses, ADV-MOV-002). Its start-of-day step has its first rule: in `strict-fixed`
   (`ADV-QUIRK-18` off) every hero on the map reveals around it to its owner with its radius, after
   the calendar and before the first group's turn; `faithful` reveals nothing there until a rule
   hides tiles (the Cover of Darkness, E09).
9. *(amended)* Increment 5: the core ruleset hash and the pack format (shared, the six effect kinds
   and three flags), the adventure ruleset part v10 (the fly and water-walk prices, Nomads' terrain;
   Cursed Ground's parameter if batch E5 confirms it). The game and adventure state hashes, the map
   import, the action encoding (v4) and the adventure replay format (v5) do not change: a flying
   hero's destinations are tiles of the same mask.

## Consequences

- The golden replays' commands stay valid as data; where a fixture's hero has a movement or
  scouting skill or artifact, its play and its state hashes change and the replay is re-recorded
  (its header's core ruleset hash moves in any case).
- The first rule at the start of a day exists in one profile only: both profiles' properties run
  the day advance.

## Alternatives considered

- **The points and the reveal at the placement, before the levels at load** (today's build): a
  Scouting or Logistics gained at load would show only from the next step or turn, unlike the
  original's.
- **The levels at load before the adventure's state is built**: the evaluation needs the static
  map's terrain shares, which the adventure's build computes; a second pass over the heroes is the
  smaller change.
