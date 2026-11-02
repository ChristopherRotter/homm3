# ADR-0028 (draft): The town context: `h3-towns`, the store of towns, its ruleset part and seams

- Status: Draft (E09 T2a, 2026-11-02). Its choices were made by the user: the crate and the split
  of a town at E09's refinement (the epic's decisions 2 and 3, TWN-Q1), the store, the pool, the
  part and the moat's data at the drafts (decisions 17 to 20, TWN-Q3). Accepted at T2b's
  checkpoint (`TWN-Q4`), it moves into `docs/adr/` (the number checked then) with the four
  amendments beside it, in the `shared/` pull request that creates the crates.
- Date: 2026-11-02
- Deciders: user; town lane session (E09 T2a)
- Amends with it: ADR-0015, ADR-0016, ADR-0017, ADR-0023 (`0015-` to `0023-amendment-towns.md`
  in this folder).
- Working detail: the survey `docs/rules/surveys/E09-towns.md` (§0's findings), the design note
  `docs/epics/E09-design.md` (T2b).

<!-- Decisions only; the readings are the survey's, the build's detail the design note's. -->

## Context

E09 builds towns on E08's adventure map. `docs/03-domain.md` gives the town context buildings,
construction, growth, recruitment and the town's services; decision 2 keeps the town as a map
object (its tiles and entrance, owner, garrison, visiting hero) in the adventure, as a mine and
its guards, and has `h3-game` compose the visit, recruitment, the town's part of the day and the
siege's facts. ADR-0005 forbids edges between domain crates; ADR-0015 point 5 makes identities
that cross contexts contract types; ADR-0017's store of heroes is the shape of per-entity state
that forks share. Increment 1 reads from a town its faction, its buildings at the start and the
forbidden ones, its hall (the day's gold), its fort (the siege, the views), its Resource Silo, its
Tavern (another player's view) and the Capitol's limit; every later increment adds state to the
same record (the day's construction, the available creatures, the guild's spells).

## Decision

1. **Two new crates.**

   | Crate | Layer | Logic | Holds |
   |---|---|---|---|
   | `h3-towns` | domain | yes | the town context: every town's own record, its rules and derivations, its ruleset part and that part's compiler |
   | `h3-pack-towns` | adapter | no | `packs/*/towns.toml` → `h3-towns`' definitions |

   `h3-towns` is a logic crate (`no_std`, `check-arch` purity, the 100 % coverage gate). It
   depends on the kernel, the core content, `h3-contracts` and the hero model `h3-heroes` (the
   shared layer, for the services that change a hero from increment 4); no domain crate depends
   on it, `h3-game` does. Both crates and the pack files `packs/*/towns.toml` are the town
   lane's (`lanes.toml`).
2. **A town is split between two contexts, each fact held once** (decision 2):
   - **`h3-towns`** holds the town's own record: its **faction**, the buildings **built** and
     **forbidden**, its map **events** (kept from the import for the increments that apply them;
     increment 1 applies none), the map's name if it gives one (an unnamed town's name is the
     client's: the original's lists are text); later increments add the day's construction, the
     available creatures, the guild's spells. Its rules are pure functions of a town, the part
     and plain inputs: the **hall level** and the **day's income** (the gold by the highest hall,
     the Grail's, the silo's resources), the **fort level**, the **siege's facts** (point 7), the
     **reveal radius** of a capture, whether it has a **Tavern** (another player's view level),
     the **Capitol's limit** (given the owner's other towns).
   - **The adventure** holds the map object of kind `Town` (ADR-0016's amendment): its static
     part (footprint, entrance, the `TownId`), its **owner**, its **garrison** (the armies on
     objects, as a mine's guards) and its **visiting hero** (the hero on its entrance).
   - **`h3-game`** composes them (ADR-0017's amendment): the build (a random town's faction, the
     starting heroes and the pool), the visit and the capture, the siege's setup, the day's
     income, the views.
3. **`TownId`** (`h3-contracts`, ADR-0015 point 5): a town's index among the map's towns (classes
   77 and 98) in object order, dense from 0, fixed when the game is built (no rule creates or
   removes a town). The adventure's town object holds it, `h3-towns`' records are by it, and the
   commands, events and observations name a town by it.
4. **The store of towns** (decision 17): `GameState` holds `h3-towns`' `Towns` beside the
   adventure's state: every town's record by `TownId`, each shared between forks and copied on
   its first write, the list shared too (a fork costs one reference count; a write copies the
   list of references and the one town), as the store of heroes (ADR-0017 point 1). Nothing sees
   the sharing (ADR-0016 point 2's rule): `h3-towns` has its own canonical state hash with a
   version of the town lane's, which the game state hash folds in, in `TownId` order. A map
   without towns has an empty store.
5. **The town ruleset part** (decision 19): `TownRuleset`, compiled on the core as the adventure
   and battle parts (ADR-0015 point 2, its R01 amendment point 2): its own canonical hash and
   version, the core's recorded beside it; core entities (factions, creatures, resources) by
   handle, by key in its source. Per faction a **town record**: its buildings (identities, the
   hall and fort level each gives, which it supersedes), the silo's resources, the moat and the
   siege's natural obstacles (point 7); the part's parameters (the gold by hall level and the
   Grail's, the neutral garrison's odds by level, the standard set's level-2 dwelling chance, the
   hall a second Capitol becomes). Only what an increment reads; increment 2 adds requirements
   and costs. One file per pack folder, `towns.toml` (base: the eight factions; AB: Conflux;
   SoD: its overrides), overrides as ADR-0006's. `GameRuleset` bundles the part; a replay's
   header and the server's manifest record its hash. Content is harvested where the original
   has it as data; a constant of the executable is tier-2 data whose source is the reading
   (decision 10).
6. **Seams.** `h3-towns` and `h3-adventure` never import each other; `h3-game` reads both. The
   only new contract type of increment 1 is `TownId`. The siege's facts cross to the battle as
   ADR-0023's `Siege` (point 7). From increment 4 a town's service reads a hero's details by
   reference and returns `h3_heroes::HeroChange`s, which the game applies (ADR-0017 point 10), as
   a visit of the adventure does; recruitment returns the creatures for the game to place in an
   army the adventure holds.
7. **The siege's facts and the editions by the packs** (decisions 14 and 20): `h3-towns` derives
   `Siege` (ADR-0023's amendment) from the town and its faction's record; no game version exists.
   The faction's record holds the moat's kind (absent, harmless, damaging) and whether a siege
   there draws natural obstacles: the base pack gives them as RoE and AB play them (harmless; the
   Stronghold none, with natural obstacles), the SoD pack overrides them as SoD plays them
   (damaging; the Stronghold's moat, no natural obstacles); a Tower's kind decides its land
   mines' damage, as ADR-0023 point 2 has it.
   Conflux is the AB pack's faction, so the factions a random town or a Random choice may take
   are the ruleset's factions with a town record (eight with the base pack alone, nine with AB),
   within the map's allowed set for a player (ADR-0017's amendment).
8. **The hero pool is the game's** (decision 18; ADR-0017's amendment): `h3-towns` holds no hero
   and no pool; the Tavern (increment 3) is a town's building whose hire the game composes from
   the pool and the store.
9. **Versions owned by the town lane:** `h3-towns`' state hash and the town part's hash (v1 when
   the crate is created). Its changes bump the adventure lane's versions it touches (the game
   state hash, the import, the adventure replay format) in `cross-lane` pull requests.

## Consequences

- The `shared/` pull request after acceptance: the two crate skeletons, `arch.toml`'s edges
  (`h3-game` → `h3-towns`; `h3-towns` → kernel, content, contracts, heroes; `h3-pack-towns` →
  `h3-towns`, `h3-content`, `h3-pack`, as `h3-pack-adventure`), `lanes.toml` (the adapter and `packs/*/towns.toml` to the town lane),
  `TownId`, the versions rows, this ADR and the four amendments into `docs/adr/`; `docs/02` §3
  and `docs/03` follow.
- `cross-lane` after it: `GameState` and its hash, `GameRuleset`, the adventure replay's header
  (the part's hash), each adventure golden replay re-recorded (skill `homm3-rerecord-replays`);
  the server's manifest (the server lane's inbox) and the env (the AI lane's) as their notes.
- A fork costs one more reference count (the XL fork benchmark, `bench-check`).
- A town is never half in one context: a capture writes the owner in the adventure and the
  Capitol's limit in the store in one infallible application (ADR-0017 point 11's rule).

## Alternatives considered

- The town records inside the adventure state, `h3-towns` a library of rules over types passed
  in (TWN-Q3): the adventure lane's structs would change with every town feature.
- The same store with an id local to `h3-game` (TWN-Q3): a mapping in every projection and seam,
  as ADR-0015 found for local ids.
- The pool in `h3-towns` (TWN-Q3): the random hero is drawn with no town involved, and the store
  of heroes is the game's.
- The town data as a section of `adventure.toml`, or in the core content (TWN-Q3): the adventure
  lane's adapter, or a `shared/` pull request, for every town content change.
- The moat in the battle part's siege record, or an edition derived from the packs' names
  (TWN-Q3): each change `cross-lane`; a rule keyed by names, against ADR-0006.
- A module inside `h3-adventure` split out later; `h3-towns` owning the town's armies (TWN-Q1,
  decision 2).
