# ADR-NNNN: Adventure and game crates, ruleset parts and the import boundary

- Status: Proposed (E08 T1a draft, 2026-10-09; under the user's review)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 2–4 of the epic)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md (decisions 2-4 and the
     review's corrections). Open points: ADV-Q1, ADV-Q2 (docs/OPEN-QUESTIONS.md). Longer than
     80 lines because it covers decisions 2-4 at once (T1a: three drafts for decisions 2-9). -->

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
   an increment needs it (`h3-adventure` now, `h3-heroes` and `h3-battle` at increment 3). No
   domain crate depends on another or on `h3-game`; adapters may depend on `h3-game`.
2. **Ruleset parts.** A game's rules are the core ruleset (`h3_content::Ruleset`) plus one
   compiled part per context, each immutable, `Arc`-shared, with its own canonical hash and
   version. A part references core entities by handle, and its source references them by key
   (context data hangs off the core entity). Pack folders get one file per context
   (`core.toml`, `battle.toml`, `adventure.toml`); increment 1 adds `adventure.toml` beside
   today's `pack.toml`, hand-written source the harvester never touches (E08 decision 10).
   `h3-game`'s `GameRuleset` bundles the core ruleset and the parts; replay headers and the
   server's manifest record every part's hash.
3. **The content split waits for E04b.** Until it closes, adventure crates use only the core
   items of `h3-content` (keys, packs, resources, factions, terrain ids, creature identity, cost
   and speed, hero types, artifact and spell identity, quirks and profiles). The split into
   `h3-content` (core) and `h3-content-battle`, the matching split of `h3-pack`, and the
   writer's move from `h3-formats` into `h3-map` are one `shared/` + `cross-lane` refactor with
   the battle lane paused. No `h3-content-adventure`: the adventure definitions stay in
   `h3-adventure` until a consumer outside it needs them.
4. **The import boundary has three stages, each with one job.**
   - *Parse* (`h3-map`): bytes → raw model, faithful to the file's version (RoE, AB, SoD),
     without ruleset handles; typed errors, no panics; fuzzed. Input bounds are configured
     limits: by default the largest map the supported versions allow (144×144, two levels,
     eight players), with caps on counts and string lengths that T3 sets from the shipped maps.
   - *Import* (`h3-map`): raw model + rules → `h3_game::GameSetup`. Maps the format's codes to
     content keys and handles; draws nothing (random placeholders stay placeholders, resolved
     by `h3-game` from a setup stream, ADR draft "game orchestration"); rejects what the
     current increment cannot represent with a typed `Unsupported` error (in increment 1: a
     random hero, a hero generated at a main town). Checks no game invariant.
   - *Validate* (logic): `h3-game` builds the state from the setup, checks its own part
     (players, schedule) and hands each context its setup type (`h3_adventure::AdventureSetup`),
     which the context validates as `h3-battle` validates `BattleSetup`. A setup must satisfy
     every invariant that commands maintain (a hero never on a blocked tile, but possibly on a
     visitable one); setup and commands share one check per invariant.
   Setup types are plain owned data, owned by the crate that validates them; no context
   exchanges them, so they are not seams. Tests build setups in code.
5. **Cross-context identities are contract types:** `PlayerId` and `HeroId` in `h3-contracts`
   (ADR-0013 point 1 foresaw `HeroId`). Context-internal handles stay in their crates.
6. **Sizes come from data** (ADR-0006 guardrails): map dimensions, levels and player counts come
   from the map, bounded by ruleset parameters, never by constants in logic.

## Consequences

- T2's `shared/` pull request creates the four skeletons, the `arch.toml` edges, the lane
  paths, the two ids in `h3-contracts` (a seam addition no battle code reads), the kernel's
  per-player stream ids (ADR draft "game orchestration", point 4) and the versions rows. On
  acceptance this ADR amends ADR-0005 (the crates, the first application crate as a logic
  crate) and ADR-0006 (ruleset parts, one pack file per context); `docs/02` §3 and `docs/03`
  follow.
- One validator per invariant: a map cannot start a state that breaks one. The fuzz target runs
  parse, import and validation together.
- After the writer's move, `h3-experiment` (battle) depends on `h3-map` (adventure): an
  adventure change can break the battle lane's build; CI's full verify catches it.
- How a replay names its map (embedded setup, or the file by hash with an import version) is
  open: ADV-Q2, needed by T9.

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
