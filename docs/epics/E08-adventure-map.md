# E08 — Adventure map core (with E07's map reader)

Status: **in progress.** Increments 1 to 10 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05); increment 11 (the teleporters) runs in rounds, the position and the next task are in `docs/lanes/adventure/HANDOFF.md` and `QUEUE.md`. The record of every done task: `E08-log.md`. The closed increments' scopes, decisions 13–259 and task lists, and the original status paragraph: `E08-archive.md`. E07's `.h3m` reader was built here (T3); E10's players, turns, battle integration and projections are built here too (E10 keeps teams, victory and defeat, diplomacy and scenario setup beyond the import); towns (E09) are their own epic.

## Goal

The adventure map as a deterministic, validated, forkable domain context on real `.h3m` maps: tiles and terrain, heroes moving with movement points, the day advance, fog of war, and later objects, guards, the battle seam and towns. Built like E01: increment 1 is small in content and complete in structure (commands, validation, events, observation, a sim env, replays, a benchmark gate).

## Decisions at the refinement (2026-10-07/08, user; corrections from the review of 2026-10-08 marked)

1. **Real maps from the start, no synthetic maps.** The reader parses complete `.h3m` files (RoE, AB, SoD) into a raw model; the import into the game state is thin. In increment 1 every object is an inert blocker: its template's blocked mask *and* its visitable tiles block entering. A hero's own tile is always leavable (a placed hero may start on a town's entrance tile, which is visitable); the movement rules are specified for the real semantics (a visitable tile is enterable and ends the move) so that their ids survive increment 2. *T1a:* some objects are visited from a neighbouring tile without being entered (VCMI's `blockVisit`: resources, artifacts, monsters); the state draft has both modes, T1b lists them; in increment 1 a tile holding a hero blocks too. A map's templates carry their own masks, so no object harvest is needed yet. Increment 1 plays the user's editor maps and shipped maps whose heroes are placed (a hero generated at a main town needs towns); every other map fails the import with a typed error. *2026-10-11:* the user dropped the editor maps; the golden replays (T9) play a generated `.h3m` file, the shipped maps stay the reader's test locally.
2. **Crate shape: battle and adventure depend only on what is theirs.** Target shape (ADR-0015):

   | Layer | Crate | Holds | Lane |
   |---|---|---|---|
   | shared | `h3-content` | core: keys, packs and overrides, resources, factions, terrain ids, creature identity, cost and speed, hero types, artifact and spell identity, quirks and profiles | shared (after the split) |
   | shared | `h3-content-battle` | abilities, battle stats, battle obstacles, battle magic, battle effects | battle |
   | shared | `h3-contracts` | seams | shared |
   | domain | `h3-battle` | battle context | battle |
   | domain (`shared` since ADV-Q13, decision 25) | `h3-heroes` | the hero's own state and rules: details, validation, the pure translations to the seams (`derive_battle_hero`, `war_machines`, ADR-0013 point 2; later the adventure profile), later progression (experience, levels, skill offers); imported by neither battle nor adventure | adventure (handover when progression starts) |
   | domain | `h3-adventure` | map, heroes on the map with their armies, movement, pathing, objects, fog of war; its own ruleset part (movement costs, object kinds, adventure effects) compiled on top of the core ruleset | adventure |
   | application | `h3-game` | players, turn groups, calendar, per-player RNG streams, command routing, the whole-game setup, building `BattleSetup` and applying `BattleOutcome` with the hero crate's translations | adventure |
   | adapter | `h3-map` | `.h3m` read and write (the writer moves here from `h3-formats`) | adventure |
   | adapter | `h3-pack`, `h3-pack-adventure` | pack and scenario text formats per context | battle, adventure |
   | adapter | `h3-formats` | LOD, text tables, executable tables | battle (until a formats/tools lane) |
   | tool | `h3-experiment` | uses `h3-map`'s writer | battle |

   Context data hangs off the core entity by key; each context compiles its own ruleset part on top of the core ruleset. Pack folders get one file per context (`core.toml`, `battle.toml`, `adventure.toml`). *Review:* the translations stay in `h3-heroes` (ADR-0013 point 2 holds; an application crate may import a domain crate, so nothing moves). No `h3-content-adventure` crate: the adventure definitions and their compiler live in `h3-adventure` until a consumer outside it needs them, which the split can extract (the user, 2026-10-08: only the crates the increment needs). Increment 1 creates four crates: `h3-map`, `h3-pack-adventure`, `h3-adventure`, `h3-game`.
3. **Order of the refactor.** The split of `h3-content`/`h3-pack` and the move of the `.h3m` writer happen **after E04b closes** (a `shared/` + `cross-lane` refactor with the battle lane paused). Adventure does not wait: its crates are all new and depend only on the core part of today's `h3-content`, so the later carve-out does not touch them. *Review:* increment 3 (the battle seam) needs none of it: `h3-game` calls the hero crate's translations as they are.
4. **`h3-game` exists from increment 1** (the user, reversing the first proposal) to keep the domain contexts independent: no domain crate depends on another. Players, turn groups, the calendar and the RNG streams live there, not in `h3-adventure`. The import's target is the whole-game setup there (players, heroes, map); each domain crate validates its part from a setup type, as the battle validates `BattleSetup`, so a map cannot build state a command could not (`docs/06-security.md`, "Untrusted files"). The setup types and the import boundary are an ADR topic (T1a).
5. **The hero's adventure profile** (movement bonuses from Logistics, Pathfinding and artifacts; scouting radius) is a seam type in `h3-contracts`, derived by `h3-heroes` from the hero details as `derive_battle_hero` is and passed by `h3-game`, so `h3-adventure` does not import `h3-heroes`. Not needed before increment 3. *Withdrawn at ADV-Q13 (decision 25): the adventure reads a hero's details through `h3-heroes`.*
6. **Cheap forks are a hard requirement.** The adventure state is persistent: the static map (terrain, roads, rivers, fixed footprints, static object data) is one block behind an `Arc`; every changing per-tile layer (fog per player, tile overlays) is a **wide, shallow persistent grid** (an `Arc` to a table of `Arc` leaves; a leaf is a bitset or value block, for example 8×8 tiles; all-uniform leaves share one leaf through an `Option` or a sentinel, not a static); small hot parts (heroes, players, resources, calendar) are plain arrays. A fork costs a few reference increments; a write copies one table and one leaf (`Arc::make_mut`, `alloc::sync::Arc` in `no_std`). Hand-rolled, no persistent-collections dependency. Pointer identity never enters logic or the state hash. The leaf size is chosen by benchmark (T5), against a flat per-player copy as the baseline. Rejected: a quadtree (pointer chasing on hot reads, disk-shaped reveals fragment it, the observation needs flat planes), an undo log instead of forks (may still serve tree search later; batched RL needs independent copies). Fog can shrink again (Cover of Darkness), so the grid supports clearing. *Review:* no logic crate holds an `Arc` yet: the ADR draft says what sees through it (the canonical hash, `Clone`, the debug serialization of ADR-0003 point 12), and T5 proves copy-on-write with two properties: a write through a fork never reaches the parent, and equal content hashes equal whatever the sharing.
7. **Fog of war projection.** Each player sees explored tiles; hidden tiles bordering explored ones form a **fringe**. The original appears to leak what is drawn into the fringe (terrain and the sprites of objects reaching into it). Quirk candidate for `ADV-QUIRK-1`: confirmed before it gets a flag (Q30; the user checks a map in the editor or the game at the T1b checkpoint) and classified by the user; then in the `faithful` profile the fringe shows terrain, roads, rivers and object appearance, in `strict-fixed` terrain, roads and rivers only. The AI trains under the projection that is played. T8 writes the list of what each viewer may know (`docs/06-security.md`, "Information hiding": the adventure epic owns it), including the fringe case where an object's appearance is public while its contents stay hidden.
8. **Simultaneous turns are designed for, not built** (an HD/HotA feature, a non-goal of the vision; the guardrail is ADR-0006's: nothing built now may make them a rewrite; the user, 2026-10-08). Logic stays single-threaded; players' commands interleave in one authoritative order (the server's log, which is the replay). Built in increment 1 because they cost little and serve RL on their own: **turn groups** (the day is a schedule of groups; sequential play = groups of one player), **per-player RNG streams**, **no blocking prompts** (a level-up, an object dialog or a battle is a pending decision of one player and blocks only that player). *T1a:* a battle with another player's hero is owned by both players, who may answer outside their turn (orchestration draft, point 6). *Captured, decision deferred to the increment that builds simultaneous play:* the target that **commands of players in simultaneous play commute** (any interleaving, same state hash; a property test then), which needs a contact rule that ends simultaneous play for players who could interact (HD's and HotA's rules surveyed in T1b), per-player streams, and globally shared state (tavern pool, global counters) partitioned or explicitly accepted as order-dependent; under that target, a command that conflicts with the authoritative order is rejected and leaves the state untouched, the safety net. Later, mostly the server lane: concurrent battles in the state, turn timers and trusted system commands for disconnects, sequence numbers against duplicate commands, clients that run ahead of the server's confirmation.
9. **RNG streams are server-only.** Seeds and stream positions are authoritative state: never in an observation (observation hygiene and ownership isolation tests cover them), never seeded from client data; a replay (it contains the seed) is released only after the game. Optional for competitive play (server lane): publish a hash of the seed at the start, the seed at the end.
10. **Content scope:** RoE, AB and SoD as toggleable packs, as in battle. Values with sources from the start; harvesting adventure data (object classes, `Objects.txt`) when objects need it. `packs/*/adventure.toml` is hand-written source inside the generated pack folders: the harvester never touches it and `--with-original-data` skips it (`packs/README.md` says so; battle-owned, a one-line `cross-lane` note in T4).
11. **Benchmark:** random legal play on a 36×36 map (2 players, 2 heroes each, 28 days) with an instruction-count gate (+2 %), pathfinding timing on a 144×144 two-level map, and fork cost on both sizes (it must barely grow with map size). Targets set from the measurements, as in E01.
12. **Experiments** for movement facts reuse the SoD flow (`experiments/specs/E08*`, `docs/experiments/README.md`); the compiler writes staged battles only, so movement experiments need their own spec shape: a `cross-lane` change, sized at T10.

## Closed increments

Their scope, decisions and task lists are in `E08-archive.md` (cited as "E08 decision N"; `git grep -n 'decision 75' docs/epics/E08-archive.md`). Decisions 1–12 above and those from 260 below stay here. A decision that still binds later work was written into its ADR (ADR-0015 to -0017, amended per increment) or the code's rule entries.

| Increment | Closed | Tasks | Decisions | Content |
|---|---|---|---|---|
| 1 | 2026-10-12 | T1a–T10c | 1–12 | the map reader and import, tiles, movement, the day, fog of war, `AdventureEnv`, replays, benchmarks |
| 2 | 2026-10-13 | T10c–T16c | 13–18 | object types, the treasury, pickups, the Cartographer as the first pending decision, a few visitable objects |
| 3 | 2026-10-20 | T17a–T22b | 19–47 | monsters, guarded piles, battles on the map, the neutral seat, the store of heroes, batch E3 |
| 4 | 2026-10-21 | T23–T33 | 48–70 | hero progression: experience, levels, offers, Necromancy and Eagle Eye, batch E4 |
| 5 | 2026-10-23 | T34a–T43 | 71–89 | the hero's movement and scouting: Pathfinding, flying and water walking, Cursed Ground, batch E5 |
| 6 | 2026-10-26 | T43–T53 | 90–120 | army and hero management: the army commands, meetings with Scholar, the army dialog, the artifact positions, moves and combinations, the tactics flag, batch E6 |
| 7 | 2026-10-28 | T53–T65 | 121–157 | rewards on the map: treasure chests, artifacts and Spell Scrolls as pickups, the monsters' treasure artifacts, the twenty classes that teach or hold a find, Pandora's Box, events, the Pyramid, the battle bonuses, the visit's rests, batch E7 |
| 8 | 2026-10-30 | T65–T76 | 158–191 | boats and the sea: boats on the map and in the writer, boarding, sailing and landing with Navigation, the sea's artifacts and Favorable Winds, the battle at sea, events on water, the sea's four pickups, batch E8 and E8c |
| 9 | 2026-11-03 | T76–T87c | 192–230 | mines, the week and owned objects: flagging and owners, the seven mines and abandoned mines with their guards and Leave Guards, Lighthouses, Shipyards and `BuildBoat`, the weekly objects, the day's start (income, spell points, the week's growth, the month and its monsters), timed events, batch E9 |
| 10 | 2026-11-05 | T87c–T97c | 231–259 | dwellings and hiring: the dwellings (17, 20) and the random dwellings not tied to a town (216 to 218) flagged, with their guards and weekly creatures, the Refugee Camp, the hire question and the hire dialog, the dwellings entered from water, batch E10 |

## Increment 11 — the teleporters

Refined with the user on 2026-11-05 (T97c; ADV-Q44, asked after T97b2 merged). Built like increments 1 to 10: commands, validation, events, observation, the env, replays and the gates grow with the content. Increment 10's scope, decisions 231–259 and task list are in `E08-archive.md`, its design note and the audit of its exit criteria in `E08-log.md` (T97a, T97c).

### Decisions at the refinement (2026-11-05, user; ADV-Q44)

260. **Content: the teleporters** (the user at ADV-Q44, T97c; recommended): the **Subterranean Gates** (103: 602 objects on 98 of the 160 shipped maps; the only link between the levels on 98 of the 104 two-level maps), the **Monoliths** (43, one-way entrances: 193 / 59; 44, one-way exits: 157 / 59; 45, two-way: 495 / 98; by subclass) and the **Whirlpools** (111: 224 / 62, reached by boat); together on 146 maps, inert blockers until now. What a step onto an entrance does (the exit and how it is chosen, a drawn exit's stream, an exit that is occupied or guarded, the points left after it, a Whirlpool's losses), how the reach and the paths treat a teleporter (the hot path: the reach's and the passage's loops, the lane's `TRAPS.md`), what a viewer sees of a hero passing, the census and the code reading are the survey's. **Not in it:** garrisons (33, 219) and the other visitables (the sketch's item 12), the adventure spells (Dimension Door, Fly, Town Portal and the rest), creature banks, quests, simultaneous play, towns (E09), a computer player's routes (E10). Not chosen: garrisons with the fifteen bonus visitables (the sketch's item 11 as written), garrisons alone, the bonus visitables alone. (Counts: T97c's census, `E08-log.md` T97c.)
261. **The first round** is a survey and the ADR amendment drafts with the design note and the cut, as decisions 20, 49, 72, 91, 122, 159, 193 and 232 (T98a, T98b in the queue); the user accepts the drafts before anything is built, and the building tasks are cut at that checkpoint. Not chosen: the survey alone first. *Agent's call (change freely):* the survey plans the increment's batch (E11) on the client lane's harness, as batches E3 to E10.
262. **T98a in two** (the user at T98a's start, 2026-11-05, `AskUserQuestion`; recommended, the queue's sketch): **T98a1** the census, the objects and their templates, the channels and the jump (the code reading, VCMI and the wiki, the disputes and quirk candidates, the address map); **T98a2** the reach and the paths, the space key's visit, what a player sees, the interfaces' sketch, batch E11's plan, the proposed ids and the checkpoint. The reading was measured first: about 3 KB of code for the jump; the survey's size decided the cut. Not chosen: two parts with the original's paths in the first, the whole in one session.

### Decisions at the survey's checkpoint (2026-11-06, user; ADV-Q45)

263. **T98a2 whole** (the user at T98a2's start, `AskUserQuestion`): the readings (the paths, the views, the space key, about 1.3 KB of code measured first) and the design part (the interfaces' sketch, batch E11's plan, the ids, the checkpoint) in one session and one pull request. Not chosen: two parts, the readings first (recommended); two parts, both in the session.
264. **D-53 to D-58 as the original** (survey `E08-teleporters.md` §8; recommended): a monolith's or Whirlpool's exit is drawn, never named by a command; the Whirlpool's weakest stack by fight value × count, the first slot on a tie; it keeps `⌊n / 2⌋`, a lone creature kept; the gates paired by truncated distance; one draw over the other Whirlpools' tiles; no path goes through a teleporter (an entrance is the reach's destination). Not chosen: VCMI's paths through two-way monoliths and gates with the exit named.
265. **A-37 fixed by the battle** (recommended): an enemy hero on a Whirlpool's drawn tile is fought first, as at a monolith's exit; the mover arrives only if it wins. Not chosen: the tile not a candidate; VCMI's stay.
266. **A-38: no bound** (recommended): the weakest stack is the least of all stacks by a 64-bit product, so a lone stack past 99 999 999 is halved and several cannot corrupt the army. Not chosen: the lone stack's exemption as a `fix-optional` flag.
267. **A-39 refused at the import; A-40 and A-41 not reproduced**, no flags (recommended): every hero loses as a human's (decision 16, A-27's precedent; revisited with E10), and a viewer is told a jump's ends as a step's, never a hidden exit.
268. **The space key's revisit in increment 11** with the original's gate (survey §11; recommended): a command `Revisit { hero }` (its name T98b's) visits the entered object under the hero again, any kind with a built visit, the step's own visit and decisions, no points spent or required; the env's action encoding grows with it (the AI lane's, `cross-lane`). Not chosen: teleporters only; later, as a stub.
269. **Batch E11 before the build** (recommended, as batches E9 and E10): its spec and recipe first in T98b's cut, the rules built on its results. Not chosen: after the build; not at all.

### Decision at T98b's start (2026-11-06, user)

270. **T98b in two by side** (the user at T98b's start, `AskUserQuestion`; recommended, as T88b,
     decision 239): **T98b1** the design note's parts A and B (the kinds, the protection, the
     packs, the import, the build's surfaces and channels, the state, the jump, the Whirlpool's
     loss, the reach, information) and ADR-0016's draft (ADR-0015 needs none); **T98b2** part C
     (the battle and the meeting at an exit, the jump as a rest, the stream, `Revisit`, the env,
     replays, benchmarks, versions, ids, quirks, batch E11), ADR-0017's draft, the cut and the
     checkpoint. Not chosen: whole in one session; the note first, then the drafts.

### Decision at T98b2's start (2026-11-06, user)

271. **T98b2 whole** (the user at T98b2's start, `AskUserQuestion`; as T88b2, decision 240):
     design part C (§12 to §18), ADR-0017's draft with `Revisit`'s lines in ADR-0016's, the cut
     and the checkpoint ADV-Q46 in one session and one pull request. Not chosen: two parts, the
     design (§12 to §14 and the drafts) then the plan (§15 to §18 and the checkpoint), which was
     recommended; the note first, then the drafts.

### Decisions at the drafts' checkpoint (2026-11-06, user; ADV-Q46)

272. **The design note's parts A and B and ADR-0016's draft accepted as written** (the user asked
     for the map side's calls one by one, `AskUserQuestion`, two rounds; each as recommended): one
     `monolith` kind with a channel and a way (not three kinds); the Whirlpool's divisor as data,
     2 by default (not a constant); the protection as a core artifact effect kind
     `whirlpool_protection`, a `shared/` change in T102 (not the kind's parameter); **A-42** refused
     when the game is built (not an exit on the other surface left out); an empty boat's tile never
     an exit (not a candidate with the boats merged); a jump drops the entrance's held guard until
     batch E11's E11-19 says otherwise (not the guard fought first); the new events
     `HeroTeleported`, the Whirlpool's loss and "leads nowhere", a jump's reveals without an event
     (not `HeroStepped`).
273. **Part C's game side and ADR-0017's draft accepted as written** (recommended): the acting
     player's stream a parameter of `AdventureState::apply`; the jump after a won battle at an
     exit held as a rest after the winner's levels (`Rest::Jump`, game state hash v26); `Revisit`
     with one refusal (`NothingToRevisit`) and no guard test; the env's action encoding v8 with a
     `Revisit` entry per hero slot, `cross-lane` in T105b with the user's consent; a `build` bench
     workload for the gates' pairing. Not chosen: the revisit with the guard's test; the jump given
     at the battle's end, before the levels.
274. **The cut as proposed** (design §17; recommended): rounds 2 to 8, T99 (batch E11's maps) to
     T107 (the audit), batch E11 first with the added row E11-19 and design §16's added
     predictions; T102 one `shared/` pull request (`cross-lane`) with the ADRs, the glossary, the
     security doc and the effect kind. Not chosen: T102's code apart from its docs.

### Decision at T99's start (2026-11-06, user)

275. **T99 in two at the code's seam** (the user at T99's start, `AskUserQuestion`; recommended, as
     T89, decision 244): **T99a** the writer's five classes with their round trip and the import as
     it reads them, and the spec's fields and checks (`cross-lane`); **T99b** the specs
     `E08-batch-11a` and `-11b` with their templates and predictions, each walk priced. Not chosen:
     whole in one session; three parts (the writer and the fields, then E11a, then E11b).

### Decision at T99b's start (2026-11-06, user)

276. **T99b in two by land and water** (the user at T99b's start, `AskUserQuestion`; recommended):
     **T99b1** the spec `E08-batch-11a` without its Whirlpool rows (E11-1 to E11-9, E11-18,
     E11-19: the monoliths and the gates on both levels), compiled, priced and reviewed; **T99b2**
     11a's Whirlpool rows E11-10 to E11-13, the spec `E08-batch-11b` whole (E11-14 to E11-17), the
     batch's row in `docs/experiments/README.md` (`cross-lane`) and their review. Not chosen: by
     spec (11a whole, then 11b: about four times as large as the second); whole.

### Decision at T99b2's start (2026-11-06, user)

277. **T99b2 in two by spec** (the user at T99b2's start, `AskUserQuestion`; recommended):
     **T99b2a** 11a's Whirlpool rows E11-10 to E11-13 with its header, instructions and order,
     compiled, priced and reviewed; **T99b2b** the spec `E08-batch-11b` whole (E11-14 to E11-17)
     and the batch's row in `docs/experiments/README.md` (`cross-lane`). Not chosen: whole. The
     agent's call told at the start (the queue's): every Whirlpool army Blue's (six new heroes and
     E11-18's hero with the Admiral's Hat), boarded on Blue's day 1 and jumping on its day 2 (a
     boarding without the Admiral's Hat leaves 0 points), one entrance W1 for all.

### Decision at T100's start (2026-11-06, user)

278. **T100 in two by spec** (the user at T100's start, `AskUserQuestion`; recommended, as
     decision 277): **T100a** batch E11a (E11-1 to E11-13, E11-18, E11-19; two days): the
     harness's reads and keys it needs, the recipe `batches/E08-batch-11a.toml`, its runs and the
     filled template; **T100b** batch E11b (E11-14 to E11-17): Space repeated with a read after
     each press, the Whirlpools' list and the tiles' words, its recipe, runs and template. Each
     with the harness check and the client and map-client lanes' inboxes (CLT-Q18). Not chosen:
     11a finer by land and water, whole (as T90), by layer (the reads, then both runs).

### Decisions at batch E11's scoring (2026-11-07, user; ADV-Q47)

279. **A guard beside an entered object fights first, for every entered kind** (the user at T101's
     end, `AskUserQuestion`; survey §15's E11-19, the reading and VCMI): a step that enters a
     guarded tile meets the guard first (its answer, its battle and levels); the tile's own object
     is visited after it as a held rest (GAM-DEC-007, as an event on a guarded tile already is),
     only for a hero still on the tile; a teleporter's jump so follows a beaten guard. ADV-MON-006
     and GAM-DEC-007 amended for every kind by their building task (recommended: its own task
     before T105a; the placement is round 7's cut). Replaces decision 272's call (a jump drops the
     entrance's guard). The user added from play: a two-way monolith's exit starts no fight, a hero
     on the pair cycles between them without one, a fight starts only on a step off it onto a
     guarded tile; a one-way exit probably alike, **to be investigated**: *agent's call*, a small
     batch E11c (the arrival on a one-way exit beside a guard, the step off it; a guarded entered
     object of another kind for this decision's reach) run and scored before T105a. Not chosen:
     teleporters only; the design's call.
280. **A-43 not reproduced**, no flag (recommended): the engine lists every Whirlpool tile and
     skips one while an empty boat stands on it (design §5, §7); `ADV-QUIRK-45`, written by T104.
     Not chosen: refused when the game is built (as A-42); reproduced.

### Decision at T102's start (2026-11-07, user)

281. **T102 whole, one `shared/` pull request** (the user at T102's start, `AskUserQuestion`,
     with the consent to the `cross-lane` change; recommended, as decision 274 and T70): the two
     ADRs, the glossary, the security lines, the experiments README's row and the effect kind with
     the harvester's gap-fill, the replays' headers and the bot reports in one session. Not
     chosen: two pull requests (the docs, then the code), in one session or two.

### Decisions at T103's start (2026-11-07, user)

282. **Round 5 in four tasks** (the user at T103's start, `AskUserQuestion`; recommended): **T103a**
     the kinds, the packs, the harvest's check and `whirlpool_protection` (part v18) in this
     session; **T103b** the import, A-39's refusal and its entry (map import v21); **T103c** batch
     E11c (spec, recipe, three runs, scored); **T103d** decision 279's order for every entered
     kind, after T103c's scoring (its mine or Learning Stone row tests that order); all before
     T105a. Not chosen: T103a and T103b in one session (two pull requests); batch E11c first; T103a
     and T103b as one task with one re-record.
283. **The bots tables' adventure hash in T103a's pull request** (the user's consent, `cross-lane`,
     as decisions 214, 217, 249): the AI lane's `python/records/bots-report-v1.json` and
     `bots-neutral-v1.json` move only their `engine.adventure` line with part v18. Not chosen: the
     AI lane's inbox.

### Decision at T103c's start (2026-11-07, user)

284. **T103c whole** (the user at T103c's start, `AskUserQuestion`; recommended): the spec and
     template, the recipe, three runs and the scoring in survey §15 in one session and one pull
     request (a harness change or a row against the reading would have left the scoring as T103c2);
     the experiments README's row `cross-lane` with the user's consent. Not chosen: the runs, then
     the scoring; the spec, then the runs.

### Decisions at T104's start (2026-11-09, user)

285. **T104 whole, one pull request** (the user at T104's start, `AskUserQuestion`; recommended):
     the build (A-42, the channels, the gates' pairs), its tests and docs and the benchmark
     `build-gates-xl-144` in one session; if the benchmark shows the quadratic pairing dominating
     the build, its lever (design §18's row-band search) goes to a follow-up task instead of
     growing this one. Not chosen: two tasks and two pull requests (the build, then the
     benchmark), in one session or two.
286. **The shipped maps' check of the channels and the pairs brought forward** from T106 (the user
     at T104's start; recommended): `h3-map`'s `original_data` counts the build's channels and
     pairs on the 160 shipped maps and checks them against the survey's census (§1: 301 pairs, 18
     gates on 8 maps not with their nearest), no map refused by A-42; `maps.tar` staged. Not
     chosen: left to T106 as design §14 planned.

### Decision at T105's start (2026-11-09, user)

287. **T105a in two by land and water** (the user at T105's start, `AskUserQuestion`; recommended,
     as T99b's cut, decision 276; T105b as design §17 plans): **T105a1** the stream in
     `AdventureState::apply` (the town lane told first), the monoliths' and the gates' jumps (the
     candidates, the draw, the arrival, "leads nowhere", `HeroTeleported`), the reach's measure
     against `master`, A-41's entry; **T105a2** the Whirlpool (its loss, the Hats, the jump with the
     boat; A-38's and A-40's entries). Either way an exit held by a hero leaves the mover on the
     entrance as a registered stub until T105b. Not chosen: T105a whole; the stream alone first.

### Decision at T105b's start (2026-11-09, user)

288. **T105b in two by the seam of design §17's round 7** (the user at T105b's start,
     `AskUserQuestion`; recommended): **T105b1** the battle and the meeting at an occupied exit
     (`ADV-STUB-16` lifted), the held jump `Rest::Jump` after the winner's levels with
     `AdventureState::arrive` (game state hash v27), A-37's entry `ADV-QUIRK-39`; no `cross-lane`
     path. **T105b2** `Revisit`, `NothingToRevisit` and the legal set (replay format v11), the
     env's action encoding v8 (`cross-lane`, the AI lane's inbox with the step cap), the town
     lane's inbox (a town's revisit); its env masking `Revisit` in moves every `random-play-*`
     workload's course (re-baselined with the reason written). Not chosen: three parts (the env
     alone last); T105b whole.

### Tasks (queue entries in `docs/lanes/adventure/QUEUE.md`; one or two per round, the cut confirmed at each checkpoint)

- **T97c Increment 11 refined** (with increment 10's closure). Done (2026-11-05, `adventure/e08-t97c`; decisions 260, 261): the teleporters; the first round queued (T98a, T98b).
- **T98a The teleporters survey** (decision 261), in two (decision 262): **T98a1** done (2026-11-05, `adventure/e08-t98a1`: survey `E08-teleporters.md` §0 to §8, the census, the channels, the jump, the arrival; D-53 to D-57, A-37 to A-40; 18 address map entries); **T98a2** done (2026-11-06, `adventure/e08-t98a2`, whole: decision 263; §9 to §14, the paths, the views, the space key, the interfaces' sketch, batch E11's plan, ADV-TEL; D-58, A-41; §4.1's one-candidate draw corrected; the checkpoint ADV-Q45, decisions 264–269).
- **T98b The ADR amendment drafts, the design note and the cut** (decision 261; with the checkpoint's answers, decisions 264–269; batch E11 first, decision 269), in two by side (decision 270): **T98b1** done (2026-11-06, `adventure/e08-t98b1`: `E08-design.md` parts A and B, the draft `adr-drafts/0016-increment-11.md`; the agent's calls for the checkpoint, A-42 proposed); **T98b2** done (2026-11-06, `claude/sweet-dirac-9oz8pd`, whole: decision 271; part C §12 to §18, the draft `adr-drafts/0017-increment-11.md`, `Revisit`'s lines in ADR-0016's; the checkpoint ADV-Q46, decisions 272–274: the note, both drafts and the cut accepted).
- **T99 Batch E11's maps** (decisions 269, 274), in two at the code's seam (decision 275): **T99a** done (2026-11-06, `adventure/e08-t99a`, `cross-lane`: the writer's classes 43 to 45, 103 and 111 on either level, the spec's `level`, the gates' `pair` and `vcmi_pair`, a boat on a Whirlpool's tile and its `vcmi_tile`); **T99b** in two by land and water (decision 276): **T99b1** done (2026-11-06, `adventure/e08-t99b1`: the spec `E08-batch-11a` with E11-1 to E11-9, E11-18 and E11-19, its template, each walk priced and the fog checked on the written map, a fresh review); **T99b2** in two by spec (decision 277): **T99b2a** done (2026-11-06, `adventure/e08-t99b2a`: 11a's Whirlpool rows E11-10 to E11-13, every Whirlpool army Blue's, boarded on day 1 and jumping on day 2 through one entrance; E11-18's hero with the Admiral's Hat; the walks priced, a fresh review); **T99b2b** done (2026-11-06, `adventure/e08-t99b2b`, `cross-lane`: the spec `E08-batch-11b` with E11-14 to E11-17, E11-16's sides swapped (the agent's call), its template, the walks priced, the setup's Whirlpools' list read for a boat's tile, a fresh review; the batch's row in the experiments README). Round 2 done.
- **T100 Batch E11's harness reads, recipes and runs** (design §16, §17's round 3), in two by spec (decision 278): **T100a** done (2026-11-06, `adventure/e08-t100a`: the harness's replay on the frozen clock, branches by tile, box and opening reads; the recipe `E08-batch-11a`, three runs, the template filled); **T100b** done (2026-11-07, `adventure/e08-t100b`, whole: the key pressed again and again (`repeat`), the recipe's places, `look`, the Whirlpools' list and the tiles' reads; the recipe `E08-batch-11b`, three runs, the template filled); then **T101** the scoring.
- **T101 Batch E11 scored** (decisions 269, 274). Done (2026-11-07, whole, docs only: survey §15, every row as read but E11-19, the guard fought before the jump; the spread uniform; A-37's consequences, A-43; the checkpoint ADV-Q47, decisions 279, 280). Round 3 done.
- **T102 Increment 11's shared change** (decisions 272–274, 279, 280; `cross-lane`). Done (2026-11-07, `shared/e08-t102`, whole: decision 281): ADR-0016 points 1, 3 to 6 and ADR-0017 points 1, 4, 6, 8, 10, 11 amended with batch E11's outcome, the drafts deleted; the glossary's teleporter terms; `docs/06-security.md`'s increment 11 lines; the effect kind `whirlpool_protection` (pack format v51, core ruleset hash v52) on the Sea Captain's Hat with three sources; the 65 golden replays' `ruleset_hash` and the bot reports' `engine.core` moved alone; the experiments README's batch E11 row evaluated. Round 4 done.
- **T103 Increment 11's kinds and import, batch E11c, decision 279's order** (round 5), in four (decision 282): **T103a** done (2026-11-07, `adventure/e08-t103a`, `cross-lane`: the three kinds, the 26 types and their check, `whirlpool_protection`; ADV-TEL-001; part v18; decision 283); **T103b** done (2026-11-07, `adventure/e08-t103b`: the five classes typed, A-39 refused, `ADV-QUIRK-41`, 220 by the mines' table; map import v21; the shipped maps' census); **T103c** done (2026-11-07, `adventure/e08-t103c`, whole: decision 284; batch E11c, E11-20 to E11-23, every row as read, survey §15.1; the README row `cross-lane`); **T103d** done (2026-11-07, `adventure/e08-t103d`: decision 279's order for every entered kind, the guard first and the tile's visit held as `Rest::Visit` after it; ADV-MON-006 verified, GAM-DEC-007 and -008 amended; no version moved; two replays). Round 5 done. Records: `E08-log.md`.
- **T104 The build** (round 6; design §5). Done (2026-11-09, `adventure/e08-t104`, whole: decisions 285, 286): A-42 refused when the game is built (`SetupError::TeleporterSurface`, rock too; `ADV-QUIRK-44`), the channels and the gates' pairs (`StaticMap::channels`; ADV-TEL-002; `ADV-QUIRK-45` recorded), checked against the shipped maps' census; `build-gates-xl-144` (the gates XL map, the workload `build`), the pairing 91.6 % of that build, its lever queued. No version moved. Round 6 done. Records: `E08-log.md`.
- **T105 The jump and the game** (round 7; design §7 to §10, §12 to §14): T105a in two by land and water (decision 287): **T105a1** done (2026-11-09, `adventure/e08-t105a1`: the visitor's stream in `apply`, the monoliths' and the gates' jumps, `HeroTeleported` and `LeadsNowhere`, ADV-TEL-003 to -005 and -007, `ADV-QUIRK-43`, `ADV-STUB-16` for an occupied exit; the reach within 0.01 % of `master`; no version moved); **T105a2** done (2026-11-09, `adventure/e08-t105a2`: the Whirlpool's loss, unless the Hats, and its jump with the boat, `WhirlpoolLoss`, ADV-TEL-006, `ADV-QUIRK-40` and `-42`; the sea workloads at `master`'s count; no version moved); then **T105b** in two (decision 288): **T105b1** done (2026-11-09, `adventure/e08-t105b1`: the battle and the meeting at an occupied exit, `ADV-STUB-16` lifted, the held jump `Rest::Jump` and `AdventureState::arrive`, `ADV-QUIRK-39`; game state hash v27); **T105b2** (`Revisit`, the env) ready.
- **T106 and T107** (design §17's round 8): the fixtures and benchmarks (the shipped maps' channels already checked by T104, decision 286), the audit; queued when round 7 is done.

## Later increments (sketch)

The order and the cut of each are decided at its refinement (decision 48).

12. Garrisons (33, 219) and more visitable objects (among them the fifteen bonus visitables of T97c's census, `E08-log.md` T97c; the sketch's item 10 in part until decision 231, its 11 until decision 260); simultaneous play once the contact rule is surveyed and the commutation target is decided (the sketch's increment 5 until decision 48, its 6 until decision 71, its 7 until decision 90, its 8 until decision 121, its 9 until decision 192, its 10 until decision 231, its 11 until decision 260).
13. Creature banks (16, 24, 25, 84, 85; decision 127; the sketch's item 10 until decision 192, its 11 until decision 231, its 12 until decision 260): their guards and rewards drawn at a new game, the visit, the battle with the battle lane's bank flag (no war machines, no tactics phase, the guards' layout; the battle lane's queue, "War machines"), the reward's creatures through the army dialog (0x4ABDC0).
14. Quests, after E09 and E10 (decision 127; the sketch's item 11 until decision 192, its 12 until decision 231, its 13 until decision 260): Seer's Huts and quest guards (83, 215), their nine missions and the Seer's Hut's reward (its creatures through the army dialog, 0x573C80).

Not placed yet: the adventure spells (Fly, Water Walk, Dimension Door, Summon Boat, Scuttle Boat; survey `E08-hero-movement.md` §6), with which flying is revisited and the A-13 compromise decided (the queue's inbox, decisions 75, 78). The sea's other objects (Buoys, Mermaids, Sirens; decision 158; the Whirlpools are increment 11's, decision 260). Towns (E09) are refined after increment 1; maps whose heroes are generated at a main town need them.

## Risks

- The reader's object bodies: about 200 classes and version differences; VCMI 1.7.5's reader is the reference, the shipped maps are the test (locally; the editor maps were dropped, 2026-10-11).
- The persistent grid can cost more on hot reads than it saves on forks: benchmark both sides in T5.
- Building on the core of a not-yet-split `h3-content`: adventure crates must use only core items, or the split will touch them.
- After the writer's move, the battle lane's `h3-experiment` depends on `h3-map`: an adventure change can break the battle lane's build; CI's full verify catches it.
