# E08 — Adventure map core (with E07's map reader)

Status: **refined** (2026-10-08, refinement session with the user; increment 1 = T1–T10, not started). E07's `.h3m` reader is built inside this epic (T3); E07's remaining scope (scenario loading beyond the import, validation of map content) is refined with increment 2.

## Goal

The adventure map as a deterministic, validated, forkable domain context on real `.h3m` maps: tiles and terrain, heroes moving with movement points, the day advance, fog of war, and later objects, guards, the battle seam and towns. Built like E01: increment 1 is small in content and complete in structure (commands, validation, events, observation, a sim env, replays, a benchmark gate).

## Decisions at the refinement (2026-10-07/08, user)

1. **Real maps from the start, no synthetic maps.** The reader parses complete `.h3m` files (RoE, AB, SoD) into a raw model; the import into the adventure state is thin. In increment 1 every object is an inert blocker: its template's blocked mask *and* its visitable tiles block. A map's templates carry their own masks, so no object harvest is needed yet. Increment 1 plays maps whose heroes are placed (a hero generated at a main town needs towns).
2. **Crate shape: battle and adventure depend only on what is theirs.** Target shape (ADR draft, T1):

   | Layer | Crate | Holds | Lane |
   |---|---|---|---|
   | shared | `h3-content` | core: keys, packs and overrides, resources, factions, terrain ids, creature identity, cost and speed, hero types, artifact and spell identity, quirks and profiles | shared (after the split) |
   | shared | `h3-content-battle` | abilities, battle stats, battle obstacles, battle magic, battle effects | battle |
   | shared | `h3-content-adventure` | movement costs, object kinds, adventure effects | adventure |
   | shared | `h3-contracts` | seams | shared |
   | domain | `h3-battle` | battle context | battle |
   | domain | `h3-heroes` | the hero's own state and rules: details, validation, later progression (experience, levels, skill offers); imported by neither battle nor adventure | adventure (handover when progression starts) |
   | domain | `h3-adventure` | map, heroes on the map with their armies, movement, pathing, objects, fog of war | adventure |
   | application | `h3-game` | players, turn groups, calendar, per-player RNG streams, command routing, the translations `derive_battle_hero`, `war_machines` and the hero's adventure profile, building `BattleSetup` and applying `BattleOutcome` | adventure |
   | adapter | `h3-map` | `.h3m` read and write (the writer moves here from `h3-formats`) | adventure |
   | adapter | `h3-pack`, `h3-pack-adventure` | pack and scenario text formats per context | battle, adventure |
   | adapter | `h3-formats` | LOD, text tables, executable tables | battle (until a formats/tools lane) |
   | tool | `h3-experiment` | uses `h3-map`'s writer | battle |

   Context data hangs off the core entity by key; each context compiles its own ruleset part on top of the core ruleset. Pack folders get one file per context (`core.toml`, `battle.toml`, `adventure.toml`).
3. **Order of the refactor.** The split of `h3-content`/`h3-pack`, the move of the `.h3m` writer and the move of `derive_battle_hero`/`war_machines` into `h3-game` happen **after E04b closes** (a `shared/` + `cross-lane` refactor with the battle lane paused). Adventure does not wait: its crates are all new and depend only on the core part of today's `h3-content`, so the later carve-out does not touch them. Increment 3 (the battle seam) is the first that needs the moved translations.
4. **`h3-game` exists from increment 1** (the user, reversing the first proposal) to keep the domain contexts independent: no domain crate depends on another. Players, turn groups, the calendar and the RNG streams live there, not in `h3-adventure`.
5. **The hero's adventure profile** (movement bonuses from Logistics, Pathfinding and artifacts; scouting radius) is a seam type in `h3-contracts` derived by `h3-game` from the hero details, so `h3-adventure` does not import `h3-heroes`. Not needed before increment 3.
6. **Cheap forks are a hard requirement.** The adventure state is persistent: the static map (terrain, roads, rivers, fixed footprints, static object data) is one block behind an `Arc`; every changing per-tile layer (fog per player, tile overlays) is a **wide, shallow persistent grid** (an `Arc` to a table of `Arc` leaves; a leaf is a bitset or value block, for example 8×8 tiles; all-uniform leaves share one static leaf); small hot parts (heroes, players, resources, calendar) are plain arrays. A fork costs a few reference increments; a write copies one table and one leaf (`Arc::make_mut`, `alloc::sync::Arc` in `no_std`). Hand-rolled, no persistent-collections dependency. Pointer identity never enters logic or the state hash. The leaf size is chosen by benchmark (T5). Rejected: a quadtree (pointer chasing on hot reads, disk-shaped reveals fragment it, the observation needs flat planes), an undo log instead of forks (may still serve tree search later; batched RL needs independent copies). Fog can shrink again (Cover of Darkness), so the grid supports clearing.
7. **Fog of war projection.** Each player sees explored tiles; hidden tiles bordering explored ones form a **fringe**. The original leaks what is drawn into the fringe (terrain and the sprites of objects reaching into it). Quirk (`ADV-QUIRK-1`, T1): in the `faithful` profile the fringe shows terrain, roads, rivers and object appearance; in the fixed profile terrain, roads and rivers only. The AI trains under the projection that is played.
8. **Simultaneous turns are first-class** (an HD/HotA feature, not in the base rules). Logic stays single-threaded; players' commands interleave in one authoritative order (the server's log, which is the replay); a command that loses a race is rejected and leaves the state untouched. Design target: **commands of players in simultaneous play commute** (any interleaving, same state hash; a property test once simultaneous play exists), which needs a contact rule that ends simultaneous play for players who could interact (HD's and HotA's rules surveyed in T1) and per-player RNG streams, with globally shared state (tavern pool, global counters) identified and partitioned or explicitly accepted as order-dependent. Built in increment 1 so it is not bolted on: **turn groups** (the day is a schedule of groups; sequential play = groups of one player), **per-player RNG streams**, **no blocking prompts** (a level-up, an object dialog or a battle is a pending decision of one player and blocks only that player). Later, mostly the server lane: concurrent battles in the state, turn timers and trusted system commands for disconnects, sequence numbers against duplicate commands, clients that run ahead of the server's confirmation.
9. **RNG streams are server-only.** Seeds and stream positions are authoritative state: never in an observation (observation hygiene and ownership isolation tests cover them), never seeded from client data; a replay (it contains the seed) is released only after the game. Optional for competitive play (server lane): publish a hash of the seed at the start, the seed at the end.
10. **Content scope:** RoE, AB and SoD as toggleable packs, as in battle. Values with sources from the start; harvesting adventure data (object classes, `Objects.txt`) when objects need it.
11. **Benchmark:** random legal play on a 36×36 map (2 players, 2 heroes each, 28 days) with an instruction-count gate (+2 %), pathfinding timing on a 144×144 two-level map, and fork cost on both sizes (it must barely grow with map size). Targets set from the measurements, as in E01.
12. **Experiments** reuse the SoD pipeline for movement facts (`experiments/specs/E08*`); compiler changes are `cross-lane` pull requests.

## Increment 1 — scope

- `h3-map`: complete reader of RoE/AB/SoD maps (raw model), fuzzed; import of terrain, roads, rivers, object footprints and placed heroes.
- `h3-content-adventure` + `h3-pack-adventure`: terrain and road movement costs, the movement-point table by army speed; adventure pack files.
- `h3-adventure`: static map, persistent grids, heroes on the map, movement (`MoveHero` with a full path, validated step by step, rejected whole), the reachable-tiles query (costs and the chosen shortest path, a first-class core query, ADR-0006), fog of war with the fringe quirk, events with a visibility scope.
- `h3-game`: players, turn groups (sequential play only), calendar and the day advance (`EndTurn`), per-player RNG streams, `apply`/`legal_commands`/`observe` across contexts.
- `AdventureEnv` (reset, step, mask, fork, hash), a random agent, the CLI (run, replay verify, bench), golden replays, the benchmark gate.
- Property tests as in E01: atomicity, legality consistency, ownership isolation, observation hygiene, clone equivalence, determinism.

**Non-goals:** towns, monsters and guards, battles, visiting or picking up objects, hero progression, skills and artifacts in movement (the adventure profile), simultaneous play itself, the water and boats, the subterranean gate and other teleporters.

## Increment 1 — tasks

- **T1 Survey and ADR draft.** Sources and quirks for movement (movement points by speed, terrain and road costs, diagonal cost, the last step of a day, leftover points), fog (scouting radius, the fringe), the day advance; HD's and HotA's simultaneous-turn contact rules. Survey `docs/rules/surveys/E08-adventure-core.md`, quirks in `docs/rules/quirks-adventure.md`, rule ids `ADV-*` (and `GAME-*`). ADR draft (`docs/lanes/adventure/adr-drafts/`): crate shape and the ADR-0005 amendment, the persistent state, map-object representation (closed behaviour kinds with data parameters, ADR-0012 style; increment 1 uses only "blocker"), turn groups and RNG streams, the fog projection, where `AdventureEnv` lives (`h3-sim` belongs to battle until the AI lane). **Stops for the user's review.**
- **T2 Shared skeleton.** A `shared/` pull request: the five crate skeletons, `arch.toml` (layers, the `h3-game` application crate), `lanes.toml` (new crates, `packs/*/adventure.toml`, `docs/rules/catalog/game.toml`), `BACKLOG.md` (E07/E08 in progress; a row for the post-E04b crate split), the global HANDOFF's lane row.
- **T3 Map reader** (`h3-map`, probably T3a header/players/terrain, T3b templates and every object body). Complete raw model; round-trip with the writer's subset; fuzzing; fixtures: maps written by our writer and maps the user makes in the editor (committed), the shipped maps in the local `--with-original-data` gate only.
- **T4 Adventure content.** Movement costs and the movement-point table in `h3-content-adventure`, the pack format in `h3-pack-adventure`, values with sources.
- **T5 State and the persistent grid.** Static layer, grid with leaf-size benchmark, heroes on the map with armies, the import from `h3-map`, canonical state hash; fork benchmarks on 36×36 and 144×144.
- **T6 `h3-game`, thin.** Players, turn groups, calendar, RNG streams, `EndTurn`, routing.
- **T7 Movement.** Movement points per day, `MoveHero`, the reachable-tiles query, rules `ADV-MOV-*`.
- **T8 Fog and projection.** Reveal by scouting radius, the fringe quirk, `observe`, `legal_commands`, the property tests.
- **T9 Env, CLI, gates.** `AdventureEnv`, random agent, replays and golden hashes, the benchmark workload and baseline.
- **T10 Closure.** Experiment batch for the movement facts, docs, refinement of increment 2, checkpoint with the user.

## Later increments (sketch)

2. Scenario loading and validation (rest of E07), pickups (resource piles, treasure chests as a pending decision), the first visitable objects.
3. Guards and wandering monsters, the battle seam through `h3-game` (needs the moved translations), applying outcomes (batch 1b: experience, Necromancy, war machines), the adventure profile (Logistics, Pathfinding, Scouting, boots), hero progression with the `h3-heroes` handover.
4. Mines and the week, flaggable objects, more visitable objects; simultaneous play once the contact rule is surveyed.

## Risks

- The reader's object bodies: about 200 classes and version differences; VCMI 1.7.5's reader is the reference, the user's editor maps and the shipped maps are the test.
- The persistent grid can cost more on hot reads than it saves on forks: benchmark both sides in T5.
- Building on the core of a not-yet-split `h3-content`: adventure crates must use only core items, or the split will touch them.
