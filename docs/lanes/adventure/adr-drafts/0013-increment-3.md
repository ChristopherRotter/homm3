# ADR-0013 amendment draft: `h3-heroes` as the hero model beneath the contexts

- Status: Accepted by the user at ADV-Q13 (2026-10-13; drafted in E08 T17b, revised after the
  review), to be applied by T18a; amends ADR-0013 (Accepted 2026-10-03) points 1 and 2, ADR-0015
  point 1, and withdraws E08 decision 5
- Deciders: user + adventure lane (E08 increment 3, the epic's decision 25; ADV-Q13)
- Input: `docs/epics/E08-design.md` §2. On acceptance the `shared/` pull request that applies the
  increment's amendments writes the points below into ADR-0013 and ADR-0015, changes
  `h3-heroes`' layer in `arch.toml` and adds the revision lines.

<!-- Decisions only. ADR-0013's core, the battle seam (points 3–11: `BattleHero`, derived values,
     the battle never depending on hero management), is unchanged. -->

## Context

ADR-0013 (E04a) made `h3-heroes` a domain crate beside the battle and, later, the adventure: the
hero's details, their validation and the translation into `BattleHero`. Nothing uses it at
runtime yet: staged battles and the CLI write `BattleHero` directly, and its only users are its own
tests and the pack tests; increment 3's `h3-game` is its first consumer. A domain crate may not be
depended on by another (ADR-0005, ADR-0015 point 1), so the adventure could see a hero only
through a seam of copied fields, the adventure profile (E08 decision 5), and change one only by
outcomes handled in `h3-game`. From increment 4 the adventure is the hero's main reader and
writer (movement, scouting, objects that give experience, skills, spells or artifacts, level-ups,
quests that test a hero), towns (E09) read and write heroes too, and the original keeps one record
per hero (all 156 from the game's start). The profile would grow into a copy of the details and
`h3-game` into the place where hero rules live.

## Decision

**ADR-0013 point 2, amended (the crate).** `h3-heroes` is the **hero model**: a logic crate in the
shared layer beneath the contexts (it depends on the kernel, the content and the contracts only).
It holds `HeroDetails`, its validation, what a hero has from its details in every context, derived
once (the battle hero, its spells and masteries; from increment 3 the adventure's Attack, Defense
and Diplomacy's join bonus, the maximum spell points, Necromancy, Learning; from increment 4
movement and scouting), and the typed **hero changes** (`HeroChange`: experience, spell points, a
spell, an artifact; increment 4 adds skills, primary skills and levels) with their checks. It has
no commands and no map, game or battle state. The adventure, the game and later the towns depend
on it; `h3-battle` never does, and sees only `BattleHero` (points 4 and 5).

**ADR-0013 point 1, extended (the store).** A cross-context `HeroId` names a record of the game's
store of heroes (ADR-0017 point 1): the contexts read a hero's details there by reference and
change them by returning hero changes that the game applies through `h3-heroes`.

**ADR-0015 point 1, amended.** "No domain crate depends on another" stays; `h3-heroes` is no
longer a domain crate (the `shared` layer, as `h3-content-battle`). `h3-game` depends on the
contexts it wires (`h3-adventure`, `h3-battle`) and on the hero model; `h3-adventure` depends on
the hero model.

**E08 decision 5 withdrawn.** No adventure profile seam: the adventure reads a hero's movement,
scouting and visit inputs through `h3-heroes`' derivations.

## Consequences

- `arch.toml` (shared): `h3-heroes` moves to the `shared` layer; `h3-adventure` gains it,
  `h3-game` gains it with `h3-battle` and `h3-content-battle`; `check-arch` still forbids
  `h3-battle → h3-heroes`. No contract type for a profile.
- One store, as the original's records: heroes off the map (the tavern pool, prisons; E09) need no
  second home.
- Rejections stay atomic: a context checks a hero's details before it returns changes, and the
  game applies them infallibly.
- The `HER-` rules gain the adventure's derivations; `h3-heroes` stays a logic crate under the
  100 % coverage gate. It moves to the adventure lane as soon as the battle lane is done with it
  (the user, ADV-Q13; a `shared/` change of `lanes.toml` then); until then increment 3's
  additions to it are `cross-lane`.

## Alternatives considered

- A peer domain crate with an adventure profile (decision 5, the first T17b draft): every read a
  copied field, every write an outcome variant and code in `h3-game`.
- The details with the hero on the map in the adventure: direct writes, but heroes off the map
  need a second home and towns would go through the game for each change.
- The hero rules merged into `h3-adventure`: the battle lane's pack tests and the towns would
  depend on the adventure context.

## Revision history lines

- For ADR-0013, 2026-10-1x: points 1 and 2 amended: `h3-heroes` is the hero model in the shared
  layer, read by the contexts and changed through typed hero changes; the store is the game's
  (E08 T17b, the user at ADV-Q13).
- For ADR-0015, 2026-10-1x: point 1 amended: `h3-heroes` in the shared layer (E08 T17b, ADV-Q13).
