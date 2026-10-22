# ADR-0016 amendment (draft): movement and scouting on the map; flying and water walking

- Status: Accepted by the user (ADV-Q21, 2026-10-22; the epic's decision 78) with A-13's caveat
  restated in point 4; drafted in E08 T34b from the answers at ADV-Q20 (decision 75) and at T34b
  (decision 77). T35's `shared/` pull request applies it to ADR-0016.
- Amends: ADR-0016 points 3, 4 and 5.
- Detail: `docs/epics/E08-design.md` part B (§5–§8); facts: survey
  `docs/rules/surveys/E08-hero-movement.md`.

## Context

Increment 5 makes a hero's skills, artifacts and specialty change its day's points, the price of
its steps and its scouting radius (ADR-0013 point 13), and brings two ways of moving that the
state's visit rules do not know: **flying** (Angel Wings) passes over blocked tiles, water and
guarded tiles; **water walking** (Boots of Levitation) passes over water. Point 4 says today that a
path continues past no visitable tile and that a step onto a guarded tile ends the move, and the
reachable-tiles query keeps one node per tile. The original keeps two nodes per tile, one a hero may
stop on and one it only passes (`pathfinder_relax` 0x4B1C4B, the node's bit 10), and ends a move
only on the first kind (D-25, settled by the second reading). It also lets a flight cross
unexplored tiles (A-13), which leaks hidden terrain into the legal moves. Its reveals take a grown
radius only at the hero's next step: the new day reveals around the heroes only after a Cover of
Darkness's hide (T34b's reading, survey §4.2).

## Decision

3. *(amended)* A hero on the map's **day's points** (the land table at its army's slowest speed,
   with the hero's bonuses) and **scouting radius** come from its details through `h3-heroes`
   (ADR-0013 point 13). They are set when the game builds the state, after the levels at load
   (ADR-0017 point 1), and the points again at each turn start of its owner (ADV-MOV-002); a gain
   in the day changes neither the points left nor the explored tiles.
4. *(amended)* **A hero's passage** depends on how it moves: walking, flying (a flying level from
   its details), or water walking (a water-walking level); a hero that does both flies.
   - **A walking hero** moves as before.
   - **A flying hero** also passes over blocked tiles, water, guarded tiles and the visitable tiles
     of objects and heroes, never over rock, and never over an **unexplored** tile: moves stay on
     explored tiles for every hero (A-13 fixed for now, `ADV-QUIRK-15`). **Provisional** (the user
     at ADV-Q20 and ADV-Q21): "probably fix, but gameplay impacts have to be considered (wide
     obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to
     the closest unexplored, but free hex; TBD later anyway". The compromise, and with it
     `ADV-QUIRK-15`'s class, is decided later (the lane's queue inbox); until then nothing hidden
     enters the legal moves. It is attacked
     by a guard and visits an object or a hero only at its move's end, and a step down onto the
     visitable tile of an object not visited from above stays impossible.
   - **A water-walking hero** also passes over water that holds no object; on land, and for the
     guards, it is a walking hero.
   - **A move ends only on a tile a hero may stop on** (D-25): a free land tile, or a visit made
     from such a tile (an entered object, or a neighbour's object or hero visited from it). A tile
     only passed over (blocked, water, a visitable tile flown over, a tile a flight enters by a
     step up off the visitable tile of an object not visited from above) is never a move's end nor
     the tile a visit is made from.
   - The **reachable-tiles query** keeps two kinds of node per tile for a flying or
     water-walking hero, one it may stop on and one it passes, and one kind for a walking hero
     (its search unchanged); a path follows each node's predecessor of its kind. Its destinations,
     and the env's mask (ADR-0017 point 8), are the tiles a hero may stop on.
   - **The step's price** follows the hero: its Pathfinding level, Nomads in its army, flying and
     water walking (the original's `step_cost` 0x4B1740), with two fix-optional quirks: flying
     costs the fly price from every inland tile, roads included (D-24, `ADV-QUIRK-16`;
     `strict-fixed` takes the lesser of the fly and the walking price everywhere), and the drop to
     0 after a step prices the cheapest step without the artifacts' flying and water walking (A-12,
     `ADV-QUIRK-17`). Cursed Ground's loss of the native price is built only if batch E5 confirms it
     in SoD (D-23).
5. *(amended)* **Reveals** use the hero's radius (ADR-0013 point 13.4): around each hero when the
   game is built and around each step's tile. A radius grown in the day shows at the hero's next
   step (the original's; `ADV-QUIRK-18`, fix-optional): `strict-fixed` also reveals around every
   hero for its owner at each new day's start (D-26, decision 77). The original's own new-day
   reveal around everything a player owns follows a Cover of Darkness's hide and comes with it
   (E09).

## Consequences

- Increment 5 reaches every caller of the query and of `MoveHero`: the env's mask, `legal_moves`,
  the properties' plans; a flying hero's path is longer and may cross tiles no walker could.
- A walking hero's search keeps its cost (`bench-check`); a flying or water-walking hero's search
  holds a second node layer in its window.
- The legal moves stay a function of the explored tiles alone (ADV-MOV-008; `docs/06-security.md`),
  at the price A-13's fix has: a flight over a wide range of obstacles stops at its unexplored
  edge, so such a range is impassable to a flyer until it is explored. The compromise point 4
  names (a route to the closest unexplored but free tile) would let a flight end there, and must
  then settle what that tells the player of the hidden tiles it crosses (`docs/06-security.md`).
- The state keeps no new field: the passage and the prices are rules over the hero's details; a
  hero's flying and water walking end with the artifact (the spells' day-long levels with the
  adventure spells).

## Alternatives considered

- **VCMI's layered pathfinder** (land, sail, water, air: a node per tile and layer): more than the
  original's two nodes per tile, which suffice until boats bring the sea's layer.
- **The original's flight over unexplored tiles** (A-13): the legal moves would depend on hidden
  terrain (whether the strip is rock, coast or water), an information leak the projection rules
  out; kept open as the basis of a later compromise (point 4).
- **A visit from a flyer's tile refused inland** (A-14, the reading of 0x4E5CE0): withdrawn by the
  user; a flying hero visits from its tile as a walking hero.
- **A reveal at once when the radius grows** (VCMI, for a skill): it would reward swapping radius
  artifacts between heroes (the user at ADV-Q20).
