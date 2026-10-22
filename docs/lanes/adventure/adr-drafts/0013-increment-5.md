# ADR-0013 amendment (draft): the hero's movement and scouting in `h3-heroes`

- Status: Accepted by the user as drafted (ADV-Q21, 2026-10-22; the epic's decision 78); drafted
  in E08 T34b from the answers at ADV-Q20 (decision 75) and at T34b (decisions 76, 77). T35's
  `shared/` pull request applies it to ADR-0013.
- Amends: ADR-0013 points 2, 3 and 11; adds point 13.
- Detail: `docs/epics/E08-design.md` part A (§1–§4); facts: survey
  `docs/rules/surveys/E08-hero-movement.md`.

## Context

E08 increment 5 is the hero's movement and scouting (decisions 71, 74): the secondary skills a hero
gains at a level-up since increment 4 (Logistics, Pathfinding, Scouting) take effect on the map, and
so do the artifacts the heroes wear (Boots of Speed, Equestrian's Gloves, Angel Wings, Boots of
Levitation, the Speculum, the Spyglass) and the Logistics specialists' specialty. ADR-0013 point 2
already says that what a hero has from its details in every context is derived once in
`h3-heroes`, and names "later movement and scouting"; the adventure reads a hero's details through
the store of heroes (ADR-0017 point 1). The original computes these properties in the hero record's
getters (`hero_max_land_movement` 0x4E4990, `hero_scouting_radius` 0x4E4070, the fly and water-walk
levels `hero_step_cost` 0x4B18C0 forces), from the hero's skills, artifacts and specialty, with the
army's slowest creature and the adventure's own tables. The towns' buildings and the map's objects
(Stables, Lighthouses, later increments) and the spells (Fly, Water Walk) will add to the same
properties.

## Decision

2. *(amended)* What a hero has from its details in every context is derived here once: from E08
   increment 5 also its **movement and scouting** (point 13). The adventure reads them through
   these functions, never a copy; no profile seam (E08 decision 25).
3. *(amended)* Effects are data. Increment 5 adds six hero-effect kinds, read only by the
   adventure's derivations (none reaches the battle seam): the **Pathfinding level**, a **land
   movement rate** (basis points of the land table's entry, grown by a skill specialty as the
   battle's skill rates are), **land movement points** (a flat bonus), a **flying level** and a
   **water-walking level** (0 to 3, the original's index of its price table; the artifacts give 3),
   and a **scouting radius bonus** (tiles). Each artifact type counts once, worn or as a part of a
   worn combination. The tables the bonuses apply to (the land points by speed, the base radius,
   the step prices, the fly and water-walk prices) stay the adventure part's data and come in as
   arguments.
11. *(amended)* The adventure's derivations are `HER-ADV-` rules in `heroes.toml` (increment 3's
    Attack and Defense and join bonus; increment 5's movement and scouting).
13. **Movement and scouting are `h3-heroes`' derivations** (E08 increment 5; pure functions of the
    details, the ruleset and explicit inputs):
    1. **The day's land points** from the land table's entry the adventure gives: grown by the
       hero's land movement rates (Logistics 10, 20, 30 %; a Logistics specialist's rate grown by
       5 % of itself per level) and rounded down, then the flat points added (Boots of Speed 600,
       Equestrian's Gloves 300). The rate follows Q-0005's flag: the original's single-precision
       rate and x87 product in `faithful`, exact in `strict-fixed` (D-22).
    2. **A creature's speed for the day's points**: the creature table's speed, one faster for the
       hero's creature specialty's creature and its upgrades (the specialty's speed); no speed
       artifact counts (the code and the wiki). The army stays the adventure's, an argument.
    3. **The Pathfinding level**, the **flying level** and the **water-walking level**: the highest
       of the hero's sources, none without one; the adventure prices steps and passes tiles by
       them (ADR-0016 point 4).
    4. **The scouting radius** from the base radius the adventure gives, plus the hero's radius
       bonuses (Scouting 1, 2, 3; the Speculum and the Spyglass 1 each). The original's Scouting
       specialty (the radius × 1 + 0.05 × level) has no SoD hero: not built, and a skill specialty
       on Scouting grows nothing.
    5. **When** a property takes effect is the adventure's and the game's (ADR-0016 points 3 and 5,
       ADR-0017 point 3): a gain in the day changes neither the points left nor the explored tiles
       until the owner's next turn start or the hero's next step.

## Consequences

- The level-ups of increment 4 now matter on the map: a hero that learns Logistics, Pathfinding or
  Scouting moves and sees as the original's from its next turn or step.
- The data adds six effect kinds to the core content (the pack format and the core ruleset hash);
  the skills and artifacts get their effects in the packs (the harvester's gap-fill: the amounts
  of `movement.txt`, the levels and radii of the code reading).
- The stores, objects and spells of later increments (Stables, Lighthouses, Navigation, Fly, Water
  Walk) add sources or sea counterparts to the same functions; Navigation's sea points are a
  sibling of point 13.1 with boats.
- No new hero change: nothing in increment 5 writes a hero's details.

## Alternatives considered

- **One movement profile struct in `h3-contracts`** (E08 decision 5): withdrawn at ADV-Q13; the
  functions here are read through the store.
- **The whole day's points formula in `h3-heroes`, the land table as hero data**: the table is
  the adventure's (ADV-MOV-001, the movement part of `adventure.toml`); the hero model gives the
  bonuses on a base, as `gained_experience` does on a battle's experience.
- **VCMI's bonus system** (Logistics as a percentage of the base, the speed artifacts counting
  towards the slowest creature): the speed artifacts contradict the code and the wiki; integer
  percents of percents round the specialists differently from both profiles (D-22).
- **The Scouting specialty built from the code** (1 + 0.05 × level of the whole radius): no SoD
  content reaches it, and the wiki (HotA: +1 every six levels) and VCMI (the skill's +1 to +3 grown)
  each have another formula; left until content needs it.
