# ADR-0015: Adventure and game crates, ruleset parts and the import boundary

- Status: Accepted (2026-10-10)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 2–4 of the epic; ADV-Q1,
  ADV-Q2)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md (decisions 2-4 and the
     review's corrections). Longer than 80 lines because it covers decisions 2-4 at once and E09's
     towns (ADR-0028). -->

## Context

E08 builds the adventure map on real `.h3m` files and needs players, turns and a calendar from
its first increment. ADR-0005 forbids edges between domain contexts; a map file is untrusted
input that must not build a state no sequence of commands could build (`docs/06-security.md`,
"Untrusted files"); `h3-content` and `h3-pack` stay with the battle lane until E04b closes.

## Decision

1. **Four new crates.**

   | Crate | Layer | Logic | Holds |
   |---|---|---|---|
   | `h3-adventure` | domain | yes | the adventure context: map, heroes on the map with their armies, movement and pathing, objects, fog of war; its ruleset part and that part's compiler |
   | `h3-game` | application | yes | the whole game: its setup and state, players, turn groups, calendar, RNG streams, command routing, projections across contexts; from increment 3 the battle seam with `h3-heroes`' translations |
   | `h3-map` | adapter | no | `.h3m` read (raw model) and the import into setup types; the writer after E04b |
   | `h3-pack-adventure` | adapter | no | `packs/*/adventure.toml` → `h3-adventure`'s definitions |

   `h3-game` is the first application crate and a logic crate (`no_std`, `check-arch` purity,
   the 100 % coverage gate). It depends on shared and domain crates, adding a domain edge when
   an increment needs it (`h3-adventure` now, `h3-battle` at increment 3), and on the hero model
   `h3-heroes`. No domain crate depends on another or on `h3-game`; `h3-heroes` is no longer a
   domain crate but the hero model in the `shared` layer, as `h3-content-battle` (ADR-0013 point
   2), so `h3-adventure` depends on it too; adapters may depend on `h3-game`.
   **E09's increment 1** (towns; E09 decisions 2, 3; ADR-0028 point 1): two more, `h3-towns`
   (domain, logic) and `h3-pack-towns` (adapter, `packs/*/towns.toml`); `h3-game` depends on
   `h3-towns`. The import's towns are `h3-map`'s `src/import/town*`, the town lane's.
2. **Ruleset parts.** A game's rules are the core ruleset (`h3_content::Ruleset`) plus one
   compiled part per context, each immutable, `Arc`-shared, with its own canonical hash and
   version. A part references core entities by handle, and its source references them by key
   (context data hangs off the core entity). Pack folders get one file per context
   (`core.toml`, `battle.toml`, `adventure.toml`); increment 1 adds `adventure.toml` beside
   the pack's file, hand-written source the harvester never touches (E08 decision 10).
   `h3-game`'s `GameRuleset` bundles the core ruleset and the parts; replay headers and the
   server's manifest record every part's hash. **E09** adds a third context part, the **town
   part** (`TownRuleset`, ADR-0028 point 5): a pack folder holds `core.toml`, `battle.toml`,
   `adventure.toml` and `towns.toml`, and `GameRuleset` bundles the core and the three parts.
3. **The content split waits for E04b.** Until it closes, adventure crates use only the core
   items of `h3-content` (keys, packs, resources, factions, terrain ids, creature identity, cost
   and speed, hero types, artifact and spell identity, quirks and profiles). The split into
   `h3-content` (core) and `h3-content-battle`, the matching split of `h3-pack`, and the
   writer's move from `h3-formats` into `h3-map` are one `shared/` + `cross-lane` refactor with
   the battle lane paused. No `h3-content-adventure`: the adventure definitions stay in
   `h3-adventure` until a consumer outside it needs them.
4. **The import boundary has three stages, each with one job.**
   - *Parse* (`h3-map`): bytes → raw model, faithful to the file's version (RoE, AB, SoD),
     without ruleset handles; typed errors, no panics; property-tested on arbitrary bytes
     (stable; `cargo-fuzz` later, the user 2026-10-10). Input bounds are configured
     limits: by default the largest map the supported versions allow (144×144, two levels,
     eight players), with caps on counts and string lengths that T3 sets from the shipped maps.
   - *Import* (`h3-map`): raw model + rules → `h3_game::GameSetup`. Maps the format's codes to
     content keys and handles; draws nothing (random placeholders stay placeholders, resolved
     by `h3-game` from a setup stream, ADR-0017); rejects what the
     current increment cannot represent with a typed `Unsupported` error (in increment 1: a
     random hero, a hero generated at a main town; both lifted by E09, below). Checks no game
     invariant.
   - *The import from E09's increment 1* (towns and starting heroes; E09 decisions 6, 8, 9, 12 to
     14). It still draws nothing and checks no game invariant:
     - **Towns** (classes 77 and 98) become two setups: `h3-towns`' town setup (a fixed faction,
       or a random town's rule with its alignment and owner; the built and forbidden buildings as
       the map gives them, or the standard set with the map's Fort flag; the events; the name)
       and the adventure's town object (its template, owner and garrison; ADR-0016 point 1),
       joined by the town's `TownId`. A random stack in a garrison stays a placeholder of its
       level and upgrade. An alignment byte from 8 to 254 is an invalid map, a typed error (T-1,
       E09 decision 12), as an owner colour nobody plays.
     - **Players:** each player's setup gains its **allowed factions** (the map's set; an
       RoE-format map's player whose eight are all allowed also Conflux when the ruleset has it,
       E09 decision 14), its **faction choice** (the map's default: the one allowed faction, else
       Random), its **starting-hero pick** (none from the map), its **main town** (the header's,
       as the town's entrance tile) with whether it **generates a hero**, and its **random-hero
       flag**. Whoever seats the players may change the choice and the pick before the game is
       built, as the human flag (E09 decision 6: the seam with E10's scenario setup); the game
       validates them (ADR-0017 point 1).
     - **The random hero** (class 70) is a hero placeholder with its owner and the rest of its
       body as a placed hero's (E09 decision 8); **the main town's hero** is the game's when the
       main town generates one. Neither is `Unsupported` any longer.
     - **A hero on a town's blocked tile** right of its entrance is moved onto the entrance (the
       original's `placed_heroes_setup`, VCMI's "H3M format quirk"; TD-6 by the reading, E09
       decision 13); elsewhere on a town's blocked tiles it stays `Unsupported::HeroInTown` (none
       on the shipped maps).
     - **The allowed heroes** are the map's set, an RoE-format map's with the Conflux heroes when
       the ruleset has them (by the packs, E09 decision 14) and never the campaign heroes; SoD's
       disposed heroes give their colour masks.
     - No version is read for a rule: the moat and Conflux follow the packs (E09 decision 14;
       ADR-0023 point 2).
   - *Validate* (logic): `h3-game` builds the state from the setup, checks its own part
     (players, schedule) and hands each context its setup type (`h3_adventure::AdventureSetup`),
     which the context validates as `h3-battle` validates `BattleSetup`. A setup must satisfy
     every invariant that commands maintain (a hero never on a blocked tile, but possibly on a
     visitable one); setup and commands share one check per invariant.
   Setup types are plain owned data, owned by the crate that validates them; no context
   exchanges them, so they are not seams. Tests build setups in code.
5. **Cross-context identities are contract types:** `PlayerId` and `HeroId` in `h3-contracts`
   (ADR-0013 point 1 foresaw `HeroId`), from E09 `TownId` beside them (ADR-0028 point 3).
   Context-internal handles stay in their crates.
6. **Sizes come from data** (ADR-0006 guardrails): map dimensions, levels and player counts come
   from the map, bounded by ruleset parameters, never by constants in logic.

## Consequences

- T2's `shared/` pull request creates the four skeletons, the `arch.toml` edges, the lane
  paths, the two ids in `h3-contracts` (a seam addition no battle code reads), the kernel's
  per-player stream ids (ADR-0017 point 4) and the versions rows. On
  acceptance this ADR amends ADR-0005 (the crates, the first application crate as a logic
  crate) and ADR-0006 (ruleset parts, one pack file per context); `docs/02` §3 and `docs/03`
  follow.
- One validator per invariant: a map cannot start a state that breaks one. The arbitrary-bytes
  property test (later the fuzz target) runs parse, import and validation together.
- After the writer's move, `h3-experiment` (battle) depends on `h3-map` (adventure): an
  adventure change can break the battle lane's build; CI's full verify catches it.
- A replay names its map by content hash and path (a committed fixture, or the server's copy next
  to its journal) plus `h3-map`'s import version; verifying re-imports the map, and an import
  change bumps that version (ADV-Q2). Shipped maps never enter git (ADR-0009).
- E09's increment 1: the map import's version bumps (the adventure lane's, `cross-lane`); exit
  criterion 1's census of the imported maps goes into `docs/formats/h3m.md`; the arbitrary-bytes
  property test covers the towns' and players' new fields with the rest; `h3-map`'s writer (the
  experiment maps' towns of a fixed faction, E04c T5b) gains what E09 needs in `src/writer/town*`.

## Amendment — R01, the content split (2026-10-13; proposed, for the user to confirm)

Point 3 carried out (BACKLOG R01; detail in `docs/epics/R01-content-split.md`):

1. **The cut.** The core keeps, beside point 3's list, the whole hero content (classes, types and
   specialties, secondary skills, artifacts, their effects: the hero context's vocabulary, which
   `h3-heroes` translates to both seams), the creatures' flags, their abilities outside battle
   (E08 increment 9: a closed core kind beside the growth and flags, first `weekly_resource`,
   read by the game's day, never by the battle; ADR-0017's consequences) and `Formation`.
   `h3-content-battle` holds the battle part: creature battle stats, casts and abilities, the
   spells' battle magic (polarity, effects, counters, dispels, spares, `tests_as`, `summons`),
   obstacle types and the battle parameters.
2. **`BattleRuleset`** is compiled on a core like `AdventureRuleset` (point 2: its own hash, the
   core's recorded beside it) and also keeps the `Arc` of its core and dereferences to it; its
   creature and spell rows carry copies of the core fields the battle reads.
3. **Packs:** each folder holds `core.toml` (with the manifest), `battle.toml` and
   `adventure.toml` (E09: and `towns.toml`, point 2); adapters `h3-pack` (core),
   `h3-pack-battle`, `h3-pack-adventure` (E09: and `h3-pack-towns`).
4. **Owners:** `h3-content` and `h3-pack` are shared paths; `h3-content-battle` and
   `h3-pack-battle` the battle lane's.
5. **Versions:** core ruleset hash v43, battle part hash v1, pack format v35, battle replay
   format v22; the server's manifest and offers carry both hashes (protocol v21).

## Alternatives considered

- Players, turns and calendar in `h3-adventure` (the first proposal): every later context
  (towns, heroes) would depend on it; the user reversed it.
- An `h3-content-adventure` crate now: no consumer outside `h3-adventure`; the split can
  extract it later.
- Setup types in `h3-contracts`: they cross no context boundary, and every change would stop the
  line in every lane.
- An import that builds state directly: a second set of invariant checks beside the commands',
  and a map could build what play cannot.
- Ids local to each context, mapped by `h3-game`: a mapping table in every projection and seam
  for no gain.
- A replay that embeds the setup or the map file's bytes: every replay carries the whole map (up
  to 41 472 tiles); an embedded setup returns only if saves need it.
- E09: the import resolving a random town or drawing a starting hero (the import draws nothing,
  so a placeholder stays one until the game's build); a hero on any of a town's blocked tiles
  moved (TD-6: only the reading's case, E09 decision 13).

## Revision history

- 2026-10-10: Accepted (E08 T1a draft; ADV-Q1 and ADV-Q2 answered as recommended; parsers
  property-tested on stable until `cargo-fuzz` is set up).
- 2026-10-13: Amendment R01 (the content split), proposed.
- 2026-10-14 (E08 T18a; the user, ADV-Q13): point 1 amended: `h3-heroes` is the hero model in the
  `shared` layer (ADR-0013 point 2); `h3-game` depends on `h3-adventure`, `h3-battle` and it,
  `h3-adventure` on it.
- 2026-10-30 (E08 T81a `shared/`; the user at ADV-Q38, E08 decision 201): amendment R01's
  point 1 amended: the core keeps a creature's abilities outside battle (`weekly_resource`, the
  Crystal Dragons' crystal), the battle's abilities staying the battle part's.
- 2026-11-03 (E09 T4a `shared/`; the user at TWN-Q1 to TWN-Q4, E09 decisions 2, 3, 6, 8, 9, 12
  to 14, 17 to 19, 21): points 1, 2, 4 and 5 amended for E09 increment 1, towns on the map: the
  crates `h3-towns` and `h3-pack-towns` (ADR-0028), the town part and `towns.toml`, the import of
  towns, the players' factions, choices and picks, the random hero, the main town's hero, a hero
  on a town's tile and the allowed heroes; `TownId`. Drafted in E09 T2a; batch TWN-1's outcome
  (T3c, survey `E09-towns.md` §15) changes none of it.
