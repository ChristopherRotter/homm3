# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E10-game-orchestration.md` (refined 2026-11-21 with the user, ADV-Q62; its decisions 1 to 19, the next 20; increment 1's design note `E10-design.md`, its rounds T2 to T7 in §20). **E08 is done** (2026-11-27, T145: increments 1 to 15 closed; done tasks in `E08-log.md`, the chronicle of the rounds T1 to T145 in `QUEUE-ARCHIVE.md`). E10's increments are built as E08's (E10 decision 5): a survey, the ADR amendment drafts with the design note and the cut, the user's acceptance before the build, a batch for the readings at risk, the build in rounds, the audit. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E10 T3 — the teams** (round 3; design note `E10-design.md` §1 to §7; `adventure/e10-t3a`…)

- **T3a done** (2026-11-27, `adventure/e10-t3a`; `E10-log.md`): the setup's team, `allied` by
  state, one explored layer a team, measured (`passage::Side`), §4 re-tested with allies.
- **T3b:** the allies' meeting with give-only gifts (§5: GAM-TEM-003, `AllyHero`; until then a
  step toward an ally's hero is a battle), an ally's dwelling read-only (until then its dialog
  buys), the team's views (§6: GAM-TEM-004, `PlayerView::team`), and **the team's knowledge**
  (found by T3a: ADV-VIS-006's `Visits::known`, a Shrine's spell, a hut's skill, a find emptied,
  the Pyramid, a Fountain's and a Spring's visit, still the visitor's alone; the original's tile
  bits are the team's, the Corpse and Lean To the player's own, survey §6: each ally told the
  content by an event, so the wire's `cross-lane` mapping comes with it).
- **T3c** (`cross-lane`, the town lane's files; its inbox told at the start): an ally's town (§7):
  the entering's guild and buildings, the trades (`TWN-STUB-21` lifted), building and recruiting
  the owner's; the siege of an ally's visitor **read first** (`town_visit` 0x4AAFD0,
  `town_defended` 0x5BDFE0), its rule written and tested.
- **Inputs:** the design note §1 to §7, §18; the survey §6, §7; the adventure lane's `TRAPS.md`
  (the state's storage, the fog) and the town lane's for its files.
- **Depends on:** T2 (#1218) and T3a merged. **Status:** T3b next (each part's cut confirmed at its start).

## Inbox

- 2026-11-28 (the AI lane, E31 T2i, `ai/e31-t2i`, `cross-lane` with the user's consent, AI-Q42;
  **in your paths, for your review; nothing to do**): `h3_adventure::search`'s `SearchStart` has a
  new field `open` (the start tile read as no entrance for the whole search, as the original's
  value field clears its map item's entrance bit around a local search, 0x42E263; CPU-PTH-001);
  every other literal passes `false`, so no search of yours changes (`reach-xl-144` +0.03 %).

- 2026-11-27 (the AI lane, E31 T2i0, `ai/e31-t2i0`, `cross-lane` with the user's consent, AI-Q41;
  **in your paths, for your review; no rule of yours changed**): a new module
  `h3_adventure::search` (`src/search.rs` and `src/search/`, its own files) holds the original's
  computer pathfinder (`pathfinder_run` 0x56B440's modes 1 to 5; CPU-PTH-001 to -007, the ai
  lane's catalog `computer.toml`; survey `E31-computer-player.md` §1.11):
  `AdventureState::computer_search` over several days, the score order and its open list, the fog
  by the caller (`SearchFog`: the faithful policy the original's computer test, §5.3 row 9; the
  fair one the human's), the guards by the caller's fight answer (`SearchAsks`), the passage past an
  entrance, the teleporters and mode 4's jumps. It reuses your `Pricing`, `day_movement`,
  `visit_at`, `closed_above`, `may_guard`, the channels and the boats, crate-private as they are;
  the human reach, `passage` and the pricing are unchanged (your reach and random-play workloads
  measured flat). `E31-design.md` §3's first row (the computer query) is thereby done by the ai
  lane: no work for you; a change to the passage rules may want the same change there (its tests
  pin each case), and Castle Gate and Town Portal (E09 increment 8) and the Sanctuary and Quest
  Guard (E12) have stubs there (AI-STUB-22, -23).
- 2026-11-28 (the town lane, E09 T59b1, `town/e09-t59b1`, `cross-lane` with the user's
  consent, E09 decision 228; **in your paths, for your review; game state hash v42 taken**): a
  town's Magic University (TWN-UNI-001) opens your University decision (GAM-DEC-006 amended).
  (1) `h3-adventure`'s **`UniversityVisit` names its place**: `place: UniversityPlace` (`Object {
  object_type, anchor }` for the map's University, `Town(TownId)`) in place of `object_type` and
  `anchor` (`command.rs`, `lib.rs`, `state/rewards.rs`' visit and its test); (2) `h3-game`'s
  `GameEvent::UniversityOpened { player, hero, place }`, `GameState::open_university` now
  `pub(crate)`, the decision's hash writes the place's tag (**game state hash v42**); (3) the replay
  word `town university <town> <hero>` (`h3-files`): **replay format v30**; the 130 goldens
  re-recorded from their commands (only the hash lines, `format` and `game_state_hash_version`
  moved), `h3-server`'s record-test literals; (4) `h3-map`'s plans probe and play the University
  (`check_town_university`; `visited` gives no object for a town's), `view.rs` folds the place.
  The env's encoding is v25, the protocol v88.
- 2026-11-27 (the town lane, E09 T59a, `town/e09-t59a`, `cross-lane` with the user's consent,
  E09 decision 227; **in your paths, for your review; no hash of yours changed**): a town's war
  machines (TWN-SMT-001) and its boat (TWN-SHY-001). (1) `state/boats.rs`: **`launch` is
  `pub(crate)`** (the town lane's `town/boat.rs` launches a town's boat with it:
  `AdventureState::free_town_boat_spot` and `build_town_boat`, `BoatBuilt` by `viewers` as a
  Shipyard's); (2) the replay words `town machine <town> <building>` and `town boat <town>`
  (`h3-files`): **replay format v29**, the 130 goldens' `format` lines only (every hash
  reproduced), `tests/hiring.rs`' literal; (3) `h3-map`'s plans: `town_builds` probes each
  war-machine seller (with the first building and one past the record) and the boat,
  `check_town_machine` and `check_town_boat` read a listed one, `check_conservation` takes their
  prices, `check_guild_refusals` the two commands, `Step::Town` plays a listed machine in one step
  of sixteen and a boat in another; the plans met 11 machines in their cases and no boat (no plan
  town has a boat spot; the town lane's T60 trades map). The env's encoding is v24, the protocol
  v87.
- 2026-11-27 (the town lane, E09 T58b, `town/e09-t58b`, `cross-lane` with the user's consent,
  E09 decision 226; **in your paths, for your review; no hash of yours changed**): a town's
  artifact sale (TWN-MKT-005). (1) `MarketScreen::SellArtifact` (the part's hash tag 4; no pack
  names it), `MarketError::Place(ArtifactPlace)` and `NotTradable` (`command.rs`);
  `MarketParams::artifact_sale(count, cost, take)` (`market.rs`; ADV-MKT-002 amended in the
  catalog); `AdventureState::check_artifact_sale` and `ArtifactSale`, `MarketOffers::artifacts`
  (`ArtifactSaleOffer`), and **`market_offers` takes `(core, part, heroes)`** (`state/markets.rs`);
  (2) `h3-game`: `MarketAnswer::SellArtifact { place, take }`, its arm in `answer_market` (the
  artifact removed, `HeroChanged`, then the gain), `market_answers` lists a sale per offered place
  after the purchases; **found on the way:** a town's garrison hero's creature sale (T58a1) left
  its town's view untold (TWN-VEW-002), now told after the gain (`town::Told`), the plans' seed
  added to `proptest-regressions/tests/view.txt`; (3) the replay word `answer market sell-artifact
  <place> <take>` (`h3-files`): **replay format v28**, the 130 goldens re-recorded from their
  commands (only the `format` lines moved); (4) the plans (`play/trades.rs`): `drawn_answer` draws
  among five kinds (a sale at a drawn place), `trade_answers` probes every place of the window's
  hero, `trade_moves` and `check_trade_answer` know the sale, `check_trade_listing` refuses a
  backpack index past the most. Every adventure workload within its gate (`master` measured the
  same in this sandbox: `random-play-bonuses-36` +1.96 %, `fork-boxes-xl-144` +1.75 % against the
  committed baselines before any change). A pull request of yours that bumps the replay format
  after this one merges bumps again.
- 2026-11-27 (the AI lane, E31 T2h, `ai/e31-t2h`, `cross-lane` with the user's consent, AI-Q40;
  **in your paths, for your review, nothing to do**): `AdventureState::tile_index` is public
  (`state.rs`, its doc says why): the computer's object value (`h3_game::computer::objects::at`)
  finds the object whose entrance is a tile through it. Nothing else changed in your crates.

- 2026-11-27 (the map-client lane, E24 M5 T4c, `map-client/e24-m5-t4c`; **for your planning,
  nothing blocks**): the facts audit of play (E24 design §5.29) takes every fact an event carries
  from the record (the Tomb's morale, a Cartographer's tiles, the prices paid, the slots joined,
  the observatories' `TilesRevealed`) and asks the rest. Yours: **MCL-Q61** (a hero's own name and
  custom portrait in the state from the import, as a town's), **MCL-Q62** (a map text in its
  event or offer: Pandora's Box, signs, bottles, map, timed and town events, guarded messages,
  Seer's texts), **MCL-Q63** (a monster's offer naming the stack offered), **MCL-Q65** (a hero's
  derived primary skills, morale and luck as its window shows them), **MCL-Q69** (the slot a join
  fills: `CreaturesJoined` comes with no `StackChanged`). Each recommends (a).
- 2026-11-27 (the town lane, E09 T58a2, `town/e09-t58a2`, `cross-lane` with the user's consent,
  E09 decision 225; **in your paths, for your review; no rule or hash of yours changed**): (1)
  `MarketParams` (`market.rs`) tabulates every count's lot of every pair at its build
  (`resource_lot` reads the table: the same arithmetic, the same values) and derives
  `cheapest_lot(count, give)` and `cheapest_at_every_count(give)`, none hashed: a listing inside
  a window walked 42 lots of float arithmetic; (2) `h3-game`'s `market_answers` is
  `window_answers` on a resolved window, beside `offers_an_answer` and `pays_a_lot` (the town
  listing's tests); (3) the plans (`play.rs`, `play/trades.rs`): `trade_open` gives `(Option<HeroId>,
  TradeAt)` (`Object(type, anchor)` or `Town`), `trade_army` reads a garrison hero's army,
  `trade_answers` and the answer and listing oracles take a town's window, `check_decisions` a
  town's `MarketOpened`, `check_trades` the Artifact Merchants' stock, `town_builds` and
  `check_town_builds` probe `Market` (`check_town_market`), `visit_town` opens a listed window in
  one step of sixteen, `TOWN_BUILDINGS`' first custom set holds the Marketplace (bit 8), the
  towns' cover test counts the windows; (4) the town-map workloads re-baselined (their games
  changed course; the numbers in `E09-log.md`). **A seed of yours** (for your decision): at 240
  cases `a_viewer_learns_nothing_it_cannot_see` met `cc 5f490c79307edc688709adbe7638cfb284e69480d6eef3bc78eafec0f9169ceb` (a plan without towns, `Exchange`
  steps: another player's hero seen in one game and not in the other), which `master` a7b6752
  fails too; not added to `proptest-regressions/tests/view.txt`, as it would fail CI.
- 2026-11-27 (the rmg lane, E14 T4b, `shared/e14-t4b`, `cross-lane` with the user's consent, E14 decision 12; **in your paths, for your review; no rule of yours changed**): `faithful`'s core hash moved from `0x7d6081fbb0552a2f` to `0x810a2c4eb5a0373f` (two `quirk.rmg` flags registered, positions 60 and 61; `strict-fixed`'s unchanged). Your 123 golden replays' `ruleset_hash` lines were substituted (nothing else in them). A branch of yours that records new `faithful` replays before this merges needs the same substitution.
- 2026-11-26 (the town lane, E09 T58a1, `town/e09-t58a1`, `cross-lane` with the user's consent, E09
  decision 224; **in your paths, for your review; no rule of yours changed**): a town opens E08's
  market decision (`TownCommand::Market`, TWN-MKT-002), so (1) `MarketVisit` is `{ hero:
  Option<HeroId>, place: MarketPlace }` (`MarketPlace::Object { index, object_type, anchor }` or
  `Town(TownId)`; `command.rs`), the visit's outcome a map market's with `Some(hero)`; (2) every
  check in `state/markets.rs` takes a resolved `MarketWindow { visit, screens, count }`
  (`AdventureState::market_window(part, visit)` resolves a map market's, `Visit` for a town's
  place; `check_screen` is `MarketWindow::check_screen`; `market_offers` returns `MarketOffers`,
  the sales empty without a hero; `MarketPurchase` names the place; `buy` empties the Artifact
  Merchants' entry for a town's, `town/merchants.rs`), `sell` lowers a garrison hero's stack too
  (`town/rows.rs`' `garrisoned_mut`); (3) `h3-game`: `GameState::market_window` resolves a
  town's window (`town/market.rs`), every answer, `market_answers` and `market_view` read it once;
  `GameEvent::MarketOpened` and `MarketView` carry `hero: Option<HeroId>` and `place: MarketSeen`
  (no index); `Decision::Market`'s hash writes the hero as an option and the place's tag: **game
  state hash v40**; (4) the replay word `town market <town>`: **replay format v27**; the 118
  goldens re-recorded from their commands (only the format, `game_state_hash_version` and hash
  lines moved), `h3-server`'s `tests/record.rs` literals; (5) your tests: `state/tests/markets.rs`
  builds windows from the kind (`window`, `visit`) and gained a town window's test; `h3-map`'s
  `play.rs`, `play/trades.rs` and `view.rs` read a map market's place (the plans open no town's
  window until T58a2, which lists it and grows `trades::trade_open`). A pull request of yours that
  bumps the game state hash or the replay format after this one merges bumps again.
- 2026-11-26 (the map-client lane, E24 M5 T4b, `map-client/e24-m5-t4b`; **for your planning,
  nothing blocks**): play now answers a visit's questions and level-ups and Space's revisit
  through the engine, and some facts a box needs are still the client's (**MCL-Q59**, facts
  first). From the engine's side: a visit that changes only a once-per-hero mark tells nothing
  (a visited Learning Stone, by a walk or by `Revisit`: the original's box says it was visited;
  `SRV-STUB-4`), an object's own text (Pandora's Box's, an event's, a sign's) is in no event
  (the client reads the map file), and where an added artifact lands is not told (`SRV-STUB-7`).
  Each a candidate event or field when you next touch the visits; MCL-Q59 recommends it.

- 2026-11-26 (the town lane, E09 T57, `town/e09-t57`, `cross-lane` with the user's consent, E09
  decisions 221 and 223; **in your paths, for your review; one rule changed**): (1) **ADV-OBJ-006's
  restart** (`state/draws.rs`): a class without an unmarked artifact now restarts its own
  artifacts' marks from the bans alone, the other classes' kept, and the widening to the four
  classes restarts theirs when none of them is left (`ArtifactPool::restart`), as the original's
  `random_artifact_of_level` 0x4C94D0 (E09 T52 read it whole: the reset 0x4C950E covers the level's
  records only); before, every class's marks restarted. It moves a build's draws only where a class
  ran out (your `random_artifacts_draw_…` test asserts the first three widened draws distinct
  now); the catalog entry amended. (2) **The pool's marks kept:** `ArtifactPool` and its
  `ArtifactMarks` (the bans, the given marks, each class's artifacts on a list) moved to a module
  of their own, `state/pool.rs` (in `state::draws` they moved the reach loops' codegen units:
  `Reach::search` +10 %); a draw counts and finds the drawn one without a list (the same draws);
  `resolve_rewards` returns the marks and
  `StaticMap::new` gives it beside the map; `AdventureState::merchants` (an `Arc<Merchants>`, the
  town lane's `town/merchants.rs`: the marks and the Artifact Merchants' stock, TWN-STK-001, -002)
  is the state's last field and the hash's last block: **adventure state hash v30**. A fork shares
  it (one reference count). (3) `h3-game`: `state.rs` calls `town::merchants::at_setup` after the
  Portals (the setup stream's last draws), `state/day.rs` `town::merchants::monthly` after a
  creature month's step (the towns' stream; GAM-CAL-002 amended). (4) The 118 golden replays
  re-recorded from their commands: only `adventure_state_hash_version`, the checkpoints and the
  final hashes moved; the server's literals with them.
- 2026-11-26 (the AI lane, E31 T2e, `ai/e31-t2e`, `cross-lane` with the user's consent, AI-Q36;
  **in your paths, for your review, nothing to do**): `h3_heroes::choice` holds the flag-set path
  of 0x524690 beside the setup's in one switch (`Evaluation::original`, the original's 32-bit
  integers; the setup's values unchanged, HER-LVL-006's tests pass): `ComputerReads`,
  `counted_worth`, `computer_skill_value`, `computer_takes_new_skill` (CPU-SKL-001), which the
  computer player feeds with its own values; `tests/choice.rs` has three more tests.
  Another case of the reach property's bug below (its own inbox item) came up here, kept out of
  the regressions file: `cc d6c9b6e4d4b310888bd91ef7938e53df90c9b9fc9d2ac15176c57b56e7f751c2`
  (5 × 8, cells 48 to 51 at 15, 20, 15, 15 and 59 at 20, start (0, 0), one other hero on it,
  picks `[602]`, `(mode, level, both, free) = (1, 1, false, false)`; `reach/tests.rs:512`: 4
  against 7).

- 2026-11-26 (the map-client lane, E24 M5 T3b, `map-client/e24-m5-t3b`; **a fact the view lacks,
  MCL-Q58, nothing blocks**): the hot seat sets each human's view up again at its turn from a
  snapshot of its `GameObservation`, and `MapHero` names no facing (the original's hero +0x47),
  so the client draws a hero as its own window last drew it, else facing right. Asked: the
  engine keeps each hero's facing and the view shows it where it shows the hero
  (`MapHero::facing`; its moves set it as the original's steps do). Recommended in MCL-Q58 (a).
- 2026-11-26 (the AI lane, E31 T2d, `ai/e31-t2d`, `cross-lane` with the user's consent, AI-Q35;
  **in your paths, and one failure for you to look at**): (1) `h3_adventure::AdventureState`
  gained `reach_with(core, part, heroes, (hero, points))` (the reach with the caller's points in
  the place of the hero's points left; `reach` calls it with them, so every search is as before)
  and `day_maximum_of(core, part, heroes, hero)` (the day's maximum `start_turn` would set now):
  the original's computer searches from every enemy hero with its maximum plus 800 at its turn's
  start (`ai_enemy_reach_fold` 0x4280E0). `h3_game::GameState::day_income` is public (the
  computer's means). The fold uses your human's reach until a computer query exists (AI-STUB-11:
  `pathfinder_pass_through`'s modes from 3 go on past an object's tile and through the
  teleporters; `E31-design.md` §3's first row). (2) Your property test
  `reach::tests::the_query_and_move_hero_agree` failed on `master` (abf981f) with a new random
  case: the query and `MoveHero` disagree on the points left (4 against 6, `reach/tests.rs:493` on
  `master`). Not committed (it fails every lane's run); its line for
  `crates/h3-adventure/proptest-regressions/reach/tests.txt`:
  `cc e480b42be147825b9fb4ae4b88c9da8b6be90c3eda90aa07202bec75b213dd99 # shrinks to cells = [0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 20, 20, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], width = 4, height = 7, start = (0, 7), others = [(2, 0)], creatures = [0], faithful = false, picks = [], key = false, (mode, level, both, free) = (1, 1, false, false), towns = []`

- 2026-11-25 (the town lane, E09 T55, `shared/e09-t55`, `cross-lane`; **informational,
  mechanical at a merge**): the base profiles gained `quirk.town.visitor_only_trades` and
  `quirk.town.university_ignores_map_bans`, and `h3-content` gives a combination without a pack
  cost its parts' sum (SoD's twelve), so every golden adventure replay's `ruleset_hash` moved
  (on `master` after #1174: `base:faithful` 0x9078e1983efcc013 to 0x3f1f68842dab4e0c,
  `base:strict-fixed` 0xc67d550d834033d8 to 0x0bd901a144728cc7; nothing else in any file): a branch of yours that re-records after it
  merges takes the new header. `heroes.toml` gained HER-ART-008 (the combinations' costs,
  `verified`). The market's and the Black Market's purchase and the sale read a combination's cost
  as that sum from now on (a combination reaches the stock only through the picker's fallback).

- 2026-11-25 (the town lane, E09 T53, `town/e09-t53`, docs only; **for your planning, nothing to
  build now**): increment 6, the town's trades, is designed (design note `docs/epics/E09-design.md`
  Part H; drafts `docs/lanes/town/adr-drafts/*-increment-6.md`, accepted at `TWN-Q37`). Planned
  `cross-lane` in your paths, each with the user's consent at its task: **T57** the artifact
  pool's marks kept in `AdventureState` after the build (ADV-OBJ-006) and the Artifact Merchants'
  stock beside them (the adventure state hash; the town lane's code in `src/town*`), every golden
  replay re-recorded; **T58a** your market decision with a town as its place (`MarketVisit`'s
  place, its hero optional; `markets.rs`' checks on a resolved window, the map's markets byte for
  byte), the decision's hash, `MarketOpened`'s place, the word `town market`; **T58b**
  `MarketAnswer::SellArtifact` and `MarketParams::artifact_sale` (ADV-MKT-002), `MarketOffers`'
  sale places, `answer market sell-artifact`; **T59a** the town's boat through `launch` at the
  town's spot (ADV-SHY-003's rule); **T59b** `UniversityVisit`'s place (GAM-DEC-006) and
  `UniversityOpened`'s, the words `town machine`, `town boat`, `town university`, `town transform`.
  The month's draw is the towns' stream's at the month's start's end (GAM-CAL-002). Whichever
  merges first takes each number; the second rebases and bumps again.

- 2026-11-24 (the rmg lane, E14 T2a, `shared/e14-t2a`; **for your planning, nothing to do
  now**): the kernel holds the C runtime's generator, `h3_kernel::rng::CrtRand` (KER-RNG-004,
  ADR-0029 point 3: `new(seed)` is `srand`, `rand()`, `state()`), the same formula as
  `h3_adventure::split::TileRand` (ADV-MON-009, ADV-QUIRK-6) and `h3_heroes::OriginalDraws`
  (HER-LVL-002, ADV-QUIRK-9). Either may keep its own seeding and wrap a `CrtRand` (or become a
  constructor over it) whenever you next touch them; the draws are bit-identical (both tests'
  `srand(1)` vector is KER-RNG-004's), so no replay or version moves.
- 2026-11-24 (the AI lane, E31 T1b, `ai/e31-t1b`, docs only; **for your decision and your
  planning, nothing to build now**): the original's computer player's rules on the map (survey
  `docs/rules/surveys/E31-computer-player.md` §3.6; E31 decision 2: you decide whether the engine
  copies each, keyed on `human`; the policy plans as a human until then). Row 3: a marked tile's
  boat bought from afar at a map Shipyard without its visit (`ai_boat_create` 0x430F80); your
  `BuildBoat` already does it at an own or an ally's Shipyard (ADV-SHY-003), so only another
  player's or an unowned Shipyard is left to compare. Row 4: Dimension Door applied without
  movement points (`ai_walk_dimension_door` 0x430AB0; the human's cast takes 300, 200 at Expert).
  Row 5 (with the town lane): the Castle Gate's jump made even when the gate's purchase fails, and
  from an ally's gateless Inferno the walk buys the gate there (§5.5 #59). Row 6: a placed computer
  hero's patrol radius (the map's field: please keep it in the state; the policy honours it).
  Row 7 (with the town lane): the Black Market's artifact at trunc(cost / rate), one unit under
  the human's rounding in 61 % of costs. Rows 1 and 8 (the quick battle, the defender's split) are
  the battle lane's with your trigger. Outside the computer's code (§5.3, the wiki's AI Behavior
  page): an explorer-type computer hero gets 50 more movement points a day; no rule or reading yet.
  Row 9 (§5.3; not in §3.6's table): the pathfinder's fog
  test lets a computer step one tile into the unexplored while it owns a town and ignores the fog
  while it owns none (`pathfinder_fog_test` 0x4B25D4); ADV-MOV-008 holds every player to the
  human's rule, so until you decide the policy plans and walks on explored tiles only. Also: at a meeting of two heroes of one team the visitor
  moves both heroes' troops and artifacts, an ally's included (`visit_hero` 0x4A2800): please check
  your meeting command; the map loader's two crashes (§5.5 #54: a 49th Seer's Hut divides by zero;
  #60: a creature quest's counts read without a length test) for your reader's refusals. What the
  port will ask of your API (the pathfinder's query with the walk's marks as an overlay, the
  objects' contents read as the original does, the computer facts of ADV-STUB-8 and -9):
  `docs/epics/E31-design.md` §3, at E31's increments 2 and 4.

- 2026-11-24 (the map-client lane, E24 M5 T0, `map-client/e24-m5-t0`, docs only; **for your
  planning: facts the client needs from the engine**): the user's rule at E24 M5 T0 (MCL-Q54):
  **the engine and the server are the only authority for a client**; reading a game's facts
  from files at run time is phased out (facts first; the map's pictures later, by an ADR, so that
  a server's generated map reveals its contents only as discovered). E24 M5 (hot seat, in-process)
  reads no rules table and no map text in play, so it needs, each built plainly in the client
  meanwhile: (1) **MCL-Q53 (a)**: a hero coming into another player's sight told with its owner,
  type and boat (your inbox since 2026-11-09, with the server lane's `SRV-STUB-3`); (2) **MCL-Q54
  (a)**: a Pandora's Box's reward record as the original shows it (the parts the hero does not
  get, the creatures' order between those that joined and those without room, an empty Box,
  ADVEVENT 15); (3) **MCL-Q55 (a)**: a flagged mine's production (the new owner's view, then the
  server lane's wire lifting `SRV-STUB-10`); (4) the map's texts a visit shows (a sign's, an
  event's, a Box's, a Seer's) as the events' own; M5 T4 lists any further gap here. Also for M5:
  the step that changes a loaded game's humans (the client lane's note of 2026-11-21) lets M5
  start a game from Load.

- 2026-11-24 (the town lane, E09 T51, `town/e09-t51`, docs only; **the town's services
  renumbered again; nothing to build**): at T51 (`TWN-Q34`, E09 decision 197) the user split
  E09's services in two: **increment 6, the town's trades** (the Marketplace's screens with the
  Artifact Merchants and the Freelancer's Guild on your market of increment 14, the Blacksmith,
  the town's Shipyard, the Magic University, the Skeleton Transformer; its survey, E09 T52, reads
  the artifact sale (screen 3) and the merchants' monthly draw for your `market` module) and
  **increment 7, the buildings for heroes and the map** (among them the Castle's Lighthouse,
  A-30, and the Cover of Darkness); travel and the Grail (Town Portal, Castle Gate) are
  **increment 8**. Where your docs say "E09 increment 7" for Town Portal (`E08-design.md`), read
  8; "E09's increment 6" for the Artifact Merchants (ADV-QUIRK-48's ledger entry) stays right.
- 2026-11-24 (the rmg lane, E14 T1c3, `rmg/e14-t1c3`, RMG-Q2; **for your planning: needed before
  E14 increment 7**, which follows E14's increments 2 to 6; this replaces the rmg lane's T1c2 note):
  the user decided that E14 writes the original generator's maps through `h3_map::writer` with
  the gaps below closed (not a writer of `RawMap`), the uncompressed `.h3m` byte for byte against
  the oracle's 21 maps, and that a game on a generated map reads and imports those bytes as any
  map. What the writer lacks (survey `docs/rules/surveys/E14-generator.md` §3.3.3): (1) **one
  object list in the caller's order**, both levels and every kind interleaved (today the typed
  surface objects, then the underground's, then `Extras`); (2) **identifiers as given** for towns,
  monsters and prisons (the generator's serial, never the written index + 1); (3) **the template
  table as given** (its order, not first use, and two unused leading entries: the random
  monster's and the hole's first prototypes); (4) **a Seer's Hut's reward** (experience, one
  creature stack or gold) with its one-artifact quest, deadline −1, no texts (`Extras::quests`
  writes no reward); (5) **the banned artifacts as a whole set** (the generator bans 127-140; the
  writer cannot leave out the editor's 141-143); (6) the **AB layout** (0x15) and towns and
  `Extras` in **RoE** (E14 increment 7's census adds such cases; the corpus is SoD). The rmg lane
  turns its `GeneratedMap` into the writer's `Map` and `Extras` in `crates/h3-map/src/generated*`
  (its paths from increment 7, with the edge `h3-map → h3-rmg`; ADR-0029 point 2). A gap still open when E14 increment 7
  starts is closed by the rmg lane, `cross-lane` with the user's consent. For E12's order: the
  generated maps hold 115 Seer's Huts (18 of 21 maps), 278 Obelisks (20) and 5 Sanctuaries, all
  blockers in the import today.
- 2026-11-23 (the map-client lane, E24 M4 T6, `map-client/e24-m4-t6`; **for your planning,
  nothing to do now**): three facts a player's record lacks that the map viewer would show, each a
  question there with the gap built plainly meanwhile: (1) **MCL-Q53**, a hero coming into sight
  arrives as `HeroStepped { from: None }` without owner, type or boat (`SRV-STUB-3`, your inbox
  since 2026-11-09): the viewer leaves its walk out (`economy-guards-s1` Red, `sea-*-d28`); (2)
  **MCL-Q55**, a flagged mine's production (`SRV-STUB-10`): the viewer captions the mine's box with
  the original's income; (3) **MCL-STUB-4**, the seven days without a town (E10): when you build
  them, their warnings (GENRLTXT 7's box, the days left, the banishment) as events in the record,
  so the viewer shows them; until then a record shows none.

- 2026-11-23 (the map-client lane, E24 M4 T6, `map-client/e24-m4-t6`; **the capture harness,
  common: a change in behaviour**): `h3drive.Game.start` now refuses a start whose run
  directory's free bytes modulo 4 GiB lie outside [5 MiB, 2 GiB) (the game's own disk check reads
  the free space in signed 32 bits, and below the main menu's 5 MB floor it stops before the menu:
  runs used to fail as "main_menu did not appear"). The message names the ballast file's size to
  write; `H3_NO_DISK_CHECK=1` skips the check. `clients/capture/README.md`, "Traps", says it for
  every lane.

- 2026-11-21 (the town lane, E09 T49a1, `town/e09-t49a1`, `cross-lane` with the user's consent;
  **in your paths, nothing to do**): `GameState::advance_day` (`state/day.rs`) calls
  `towns_mut().start_week()` on a week's first day right after the towns' growth (every town's
  Mana Vortex flag set, TWN-GLD-009, told with the week's views), and `regain_mana` reads
  `town::guild::refilled` (TWN-GLD-008: a town with a guild ever built; its visitor and garrison
  hero get their points or their maximum, whichever is more, in place of `day_mana`: the same
  as the regain then the refill, the maximum once, for the day-start benchmarks); `settle_outcome` (`state.rs`) calls
  `town::guild::visit` for `town_entered` (the Vortex to the garrison hero, then the visitor,
  then the learning); GAM-CAL-002's summary says both. `h3-map`'s oracles taught them:
  `play.rs`' conservation applies `night_refill` after a `DayStarted`'s regain, a hire's check
  expects the town's flag cleared when the hired hero took the Vortex (`Town::with_vortex`, now
  public); `view.rs`' hire fold sets the hired view's points at a `SetSpellPoints`. No version,
  no golden replay moved.

- 2026-11-21 (the town lane, E09 T48a, `town/e09-t48a`, `cross-lane` with the user's consent;
  **in your paths, nothing to do**): `GameState::change_hero` (`state/battle.rs`) is
  `pub(crate)` (the town's guild teaches through it) and `GameState::new` (`state.rs`) calls
  `town::guild::at_start` once the state is assembled, before the start's tavern fill (TD-10: a
  generating main town's hero learns its guild's spells, no event). A town's build (a command's or
  an event's), a hire and a bars' move that leaves a visitor now tell `HeroChanged`
  (`LearnSpell`) for the town's visitor, then its garrison hero (TWN-GLD-005). `h3-map`'s plans
  follow: `check_timed` expects the learning after each event's build, the details' check and the
  views' fold take a hero hired in the command as the hirer's. No golden replay, version or bot
  report moved.
- 2026-11-21 (E08 T136; **for E10's elimination**): the market's transfer (ADV-MKT-004) refuses
  only the actor and an index past the game's players (`h3-game`'s `state/market.rs`:
  `answer_market`'s `Transfer`), and its view's `recipients` lists every other player, so "another
  player still in the game" holds only while nobody leaves it. When E10 builds an elimination
  (the original offers the colours whose defeat flag, game +0x1F636, is clear: survey
  `E08-trades.md` §5), both leave an eliminated player out (a test of each, ADV-MKT-004's summary).

- 2026-11-21 (the map-client lane, E24 M4 T5b2, MCL-Q48; **for a later task, nothing blocks**):
  a record does not tell a flagged mine's resource, so the map viewer cannot show an abandoned
  mine's MINEEVNT box after its guards' battle as the original does (its record by the resource
  the import drew). Recommended: `ObjectFlagged` (or the mine's view) carries the resource; the
  server lane's wire follows (`docs/OPEN-QUESTIONS.md`, MCL-Q48).

- 2026-11-21 (the client lane, E27 M4 T6, `client/e27-m4-t6`; **for your planning; nothing
  blocks**): M4's review of the client's saves (E27 decision 78) asks four things of yours.
  (1) A `Replay` function that applies a replay's commands and returns the state
  (`h3_sim::adventure::Replay`, with `verify`'s checks in its order): the client's load
  (`h3-battle-client`'s `save::load`) copies `verify`'s loop today and would call it instead.
  (2) `ReplayError::Incompatible` for the players' choices and the map prints "replay header
  mismatch", though neither is a header field (`replay.rs` ~278): a wording of their own.
  (3) Two comments in `h3-files`' `adventure/mod.rs` (~950, ~1001) still say "format v4".
  (4) CLT-Q160 (a) and CLT-Q170 (a): after a hot-seat load the names decide who is human (fewer
  names: the saved humans past them play on as computers; more: a name takes a computer's colour;
  a hot-seat save loaded from Load Game single: the loaded game's current player alone), so E24
  M5's play needs an engine step that changes a loaded game's human flags (a command or a setup
  step; the map-client lane told). Informational: the user wants the saves reworked later to be
  independent of the engine's version (CLT-Q173); today a save is your replay, refused under
  other rules or hash versions.

- 2026-11-21 (the town lane, E09 T47b, `town/e09-t47b`, `cross-lane` with the user's consent;
  **your report of E08 T135a fixed, in your paths; nothing to do**): `h3-map`'s
  `a_viewer_learns_nothing_it_cannot_see` (`src/tests/view.rs`) now ends its comparison when the
  viewer is shown a Thieves' Guild (`ThievesGuildSeen`, a Den of Thieves' visit), as at a hire;
  your seed `cc 9b8b2dc9…` is in `proptest-regressions/tests/view.txt` (it fails without the break,
  passes with it). The same oracle builds the owner's `TownBuildingsView` with E09 T47b's `guild`
  (`h3_towns::shown_slots`), `aurora` (`h3_towns::has_aurora`) and `vortex`: keep it in step with
  `h3_game`'s `town/view.rs` when the view changes. No version of yours moved.

- 2026-11-21 (the map client lane, E24 M4 T4c2, `map-client/e24-m4-t4c2`; **MCL-Q46, for your
  planning; the user's answer first**): another player's hero that takes a pile or an artifact
  from beside without a step (it stands next to it already) leaves in the viewer's record only the
  command's `ObjectRemoved` (type and anchor), so the viewer cannot select that hero, centre the
  view on it and turn it to the object as the original does (E24 design §5.18, `game-ai-boat`'s day
  3). Recommended: the visit told to the viewers who see it with its hero (the hero in
  `ObjectRemoved`, or an event of the visit). Meanwhile the viewer takes the object off as the
  next turn starts.

- 2026-11-20 (the map client lane, E24 M4 T4c, `map-client/e24-m4-t4c`; **MCL-Q43 and MCL-Q44,
  for your planning; the user's answers first**): another player's turn played from a viewer's
  record (E24 design §5.16). (1) MCL-Q43: the record holds no other player's heroes' points, which
  the original's kind 5 hourglass shows every player (1 000 plus the computer's heroes' maximum
  points, the points left at each step); recommended: the two numbers told to every viewer (with
  the server lane). (2) MCL-Q44: a step seen by one end only comes with the other end none
  (ADR-0016 point 6), where the original draws the hero stepping from or into the fog;
  recommended: both tiles of a step the viewer sees by either end.

- 2026-11-20 (the map client lane, E24 M4 T4b1, `map-client/e24-m4-t4b1`; **MCL-Q41 and MCL-Q42,
  for your planning; the user's answers first**): a record's visits played (E24 design §5.14).
  (1) MCL-Q41: `ChoiceOpened` and `OfferOpened` carry no view, so a chest's choice box cannot show
  both parts before the answer (the server lane's SRV-STUB-5 too); recommended: the events carry
  what the observation's `Decision` holds. (2) MCL-Q42: a once-per-hero object's visit the hero's
  mark refuses (the Learning Stone, the Star Axis: `revisit` gives an empty event list) tells
  nothing, where a Temple's gives `NothingGained { why: Visited }`; recommended: the same event
  for the marks' objects.

- 2026-11-20 (the town lane, E09 T43, `town/e09-t43`, docs only; **the planned `cross-lane`
  changes in your paths for E09 increment 5, the mage guild; nothing to do now**): accepted at
  `TWN-Q33` (E09 decisions 172 to 181; design note `docs/epics/E09-design.md` Part G §82 to §100,
  drafts `docs/lanes/town/adr-drafts/*-increment-5.md`). In your files: `AdventureOutcome` gains
  `town_entered: Option<TownId>` (the step entering a town of the team, a `Revisit` there, an
  undefended capture; set by the town lane's `visit_town`; not state, no hash) and
  `settle_outcome` one call after the capture (the town's opening and learning; T48b); the siege's
  end one call (T48b, the place the task's); `advance_day` one call after `regain_mana` (the night's
  refill, T49a); the import's town spell sets (map import **v28**, T47a); the game state hash (the
  store v4, T47a); the replay words `town open`, `town book` (T49a, T49b); ADR-0016's kind `Town`
  and ADR-0017 points 1 to 4 and 8 to 10 amended by T45a. **The guilds are drawn from the towns'
  stream at the build** (decision 179, the user's): every later draw of `StreamId::TOWNS` moves, so
  T47a re-records every golden adventure replay's hash and the outcomes of those that play a week;
  merged in order with your increment 14 (T133b took game state hash v35, replay format v21,
  encoding v14 and protocol v75; T134a plans the next), the second to merge rebasing and bumping
  again.

- 2026-11-19 (the map client lane, E24 M4 T4a1, `map-client/e24-m4-t4a`; **MCL-Q39 and MCL-Q40,
  for your planning; the user's answers first**): a record's walk (E24 design §5.12) tells the
  points left once, after the walk (`MovementChanged`), but the original's bar falls at each
  step's end (`hero_move_step` 0x4805E0 takes the step's price after its animation, 0x481182):
  recommended (MCL-Q39 (a)) the points left with each `HeroStepped` (or a `MovementChanged`
  after each step). A normal week's and month's message names it (ArrayTxt 133, 130), a draw of
  the original's game the engine does not make (`WeekStarted`, `MonthStarted`: `Normal` only):
  recommended (MCL-Q40 (a)) the name's index drawn with the kind (on a stream of its own, so the
  golden replays' other draws stay) and told. Also seen: the original banishes a player who has
  held no town for seven days (the scripted game `game-week`'s first take: Blue, with a hero and
  no town, banished on day 8, then Red's victory), a rule of E10's victory and loss; the
  scripted games give every player a town meanwhile. Nothing to build before the user answers.

- 2026-11-19 (the town lane, E09 T41, `town/e09-t41`, docs only; **the town's services renumbered,
  and two of your rules a town batch played; nothing to build**): (1) at T41 (`TWN-Q30`, E09
  decision 162) the user made E09's increment 5 **the mage guild alone**; the services are
  **increment 6** (the town's Marketplace, Artifact Merchants and Freelancer's Guild on your
  market of increment 14, the Blacksmith, the town's Shipyard and the rest) and travel and the
  Grail increment 7. Where your docs and ADR-0016's Consequences say "E09 increment 5" for the
  Marketplace or the artifact sale (screen 3), read increment 6; nothing of yours waits on it.
  (2) Batch TWN-4 (survey `docs/rules/surveys/E09-tavern.md` §14, both runs) played two rules of
  your catalogs as they read: **GAM-BAT-003** (rows TWN-4-3 and TWN-4-4: a retreating, a
  surrendering and a beaten hero each left the map, its owner none) and **ADV-VIS-008** (row
  TWN-4-7: the Den of Thieves' visit showed the guild at everything, its message ADVEVENT 142).
  Both stay `implemented`; marking them `verified` with the observation is yours if you agree.

- 2026-11-19 (the map client lane, E24 M4 T3b2, `map-client/e24-m4-t3b2`; **MCL-Q37, for your
  planning; the user's answer first**): a record's hover (E24 design §5.11) lacks the cursor's
  facts the view does not hold: the selected hero's costs over several days (the cursor's day
  groups; `legal_moves` answers today's), the tiles a monster guards (ADV-MON-006: the reach knows
  them), whether another team's town or map garrison holds an army at the viewer's level 0
  (TWN-VEW-002 shows nothing), and a visited creature bank's guards (the view shows a mine's
  only). Recommended (MCL-Q37 (a)): the view (or a `Moves` answer extended to days and attacks)
  carries them, the server lane's record keeps them. Also seen: the views show no Seer's Hut,
  Quest Guard or Obelisk yet (their hint is their name). Nothing to build before the user answers
  MCL-Q37.

- 2026-11-19 (the map client lane, E24 M4 T3b1, `map-client/e24-m4-t3b1`; **MCL-Q36, for your
  planning; the user's answer first**): a whole game's record shows the panels of the original's
  adventure screen (E24 design §5.10), and two facts of a viewer's own hero are not in the view:
  its morale and luck as the status window shows them (`H3Hero::GetMoraleBonus` 0x4E39B0,
  `GetLuckBonus` 0x4E36C0: its skills, artifacts, specialty and army's alignments, outside battle;
  today derived only into a battle, `h3_heroes::derive_battle_hero`) and whether it can still move
  today (`hero_can_move` 0x4E5F30: its points against the cheapest step from its tile). The map
  client lane recommends (MCL-Q36 (a)) that the engine derives them once for the adventure (an
  `h3-heroes` or `h3-game` reader the server lane's view then carries on `OwnMapHero`); the client
  copies no rule meanwhile (it draws 0, 0 and "has points"). Nothing to build before the user
  answers MCL-Q36.

- 2026-11-19 (the client lane, E27 M3 T7, `client/e27-m3-t7`; **for E10's starting bonus,
  nothing to do now**): the scenario screen's starting bonus is kept as the original keeps it
  across a map change and a flag's press (CLT-Q153 (a)): only the bonus's arrows skip Artifact,
  so a colour can begin with the Artifact bonus and no hero to take it (Arrogance's single player
  on Artifact, then Rebellion, whose human row has none). The shell hands each colour's bonus
  beside the setup (`h3-battle-client`'s `begin::Game::bonuses`, `CLT-STUB-9`); when E10 builds
  the bonus, decide what Artifact gives a colour with no hero (the original's effect is not read:
  `docs/epics/E27-design.md` §3.17).

- 2026-11-19 (the map client lane, E24 M4 T3a, `map-client/e24-m4-t3a`; **your inbox's entry of
  2026-10-31 from this lane, answered in part**): (1) **the object identity is withdrawn** (the
  user, 2026-11-19): the map viewer finds each object of a view in the map's file by its type and
  anchor (the server lane's note of 2026-11-12: a view's objects come in tile order without an
  index), a random one by its anchor among the file's random objects (a random dwelling's moved
  anchor allowed), a town by its `TownId`; an object added in play (a creature month's monster)
  is drawn after the file's, its order checked against the original's at E24 M4 T4. Nothing to
  build for it. (2) **Still asked: a random artifact's drawn artifact** (classes 65 to 69):
  `ObjectView` names no artifact, so the viewer cannot draw the picture the original draws
  (`ava%04d.def` of its id, E24 design §2.7; MCL-Q3: the client the artifact's own). Its picture
  shows it to every viewer that sees the object, so the view could carry it as it carries a
  random resource's drawn pile type (an artifact's id, or a type per artifact): your call. Until
  then the viewer draws a random artifact as its random template's own picture (`MCL-STUB` to
  come with T3a). The random resources (the pile's type), monsters (`MonsterView::creature`),
  dwellings (your T94a) and towns (the town's view) are covered. (3) **MCL-Q21 (b)** stays asked:
  tell this lane which of the cases you refuse at the import (the viewer's load refuses what the
  import refuses, from T3a on). (4) **A random dwelling's drawn picture** needs its class and
  subclass (17 or 20 and the original's subclass, design §2.9): the view gives the drawn
  dwelling's type, and the table from (class, subclass) to type is `h3_map::import::codes`'
  (crate-private). A public lookup the other way (a type's key to the original's class and
  subclass, `h3_map`) would serve; or say which you prefer. For E24 M4 T3b (`MCL-STUB-1`).

- 2026-11-17 (the client lane, E27 M3 T2a, `client/e27-m3-t2ab`; **for E10's victory conditions, nothing to do now**): the original rewrites a special victory's "standard victory also" flag (`h3_map`'s `Victory::Special::allow_standard`) after reading it: `victory_standard_set` 0x4BF780 (from the conditions' read 0x4C3555 with 0, from `new_game_heroes` 0x4C0139 with 1) sets it for goal 5 (a hero's defeat) always, and for goals 0, 4 and 10 (an artifact, the Grail, a transport) clears it when exactly one colour is left undefeated (game +0x1F636), else sets it outside a campaign (a campaign by its ids); the other goals keep the file's flag. The shipped maps: goals 0, 4, 10 all carry the flag, the five maps of goal 5 all lack it, and the scenario screen shows them "or Defeat All Enemies" (recorded, `scn-info`). VCMI 1.7.5 (`MapFormatH3M.cpp`) forces goal 5 too ("H3 behavior") and clears the flag for any goal on a map of one player. The client applies the screen's rule to its list only (`h3-battle-client`'s `maps::STANDARD_ALWAYS`); the game's rule is E10's (design note `docs/epics/E27-design.md` §21.1).

- 2026-11-13 (the client lane, E27 M1 T3, `client/e27-m1-t3`; **for E10's victory conditions,
  nothing to do before it**): the original's "defeat a hero" victory (code 5,
  `scenario_victory_test` 0x4F15E0; `docs/epics/E27-design.md` §5.2) counts only when the viewing
  player's team defeats the hero: another team's defeat of it shows nothing and ends nothing, and
  the condition can then never be met (the hero is gone). E10 decides the engine's rule (copy, or
  a fix such as a loss or a fallback to the standard victory); the shell shows whatever the engine
  reports. **And the won scenario's score is the engine's** (the user, CLT-Q105, E27 decision 21):
  E10 computes it (a rule id and its test; design §5.5: the base 200 − (days + 10) / (towns + 5)
  by integer division, + 25 when the player dug up the Grail, + 25 for a victory by conquest, times
  the difficulty's `f32` factor 0.8, 1, 1.3, 1.6 or 2, truncated; the rank's creature is the
  shell's) and reports it with the victory; the shell needs the day, the
  difficulty and the score, and stubs it until then.

- 2026-11-12 (the client lane, E27 M1 T1b3b, `client/e27-m1-t1b3b`; **a correction for your
  survey, and E10's needs**): 0x6972B8 is the scenario's end flag (`scenario_ended`, read: set
  with `scenario_result` 0x699510 by `scenario_condition_met` and the conditions' tests; the
  adventure manager's procedure then ends play), not a network client's: `E08-economy.md` §7.2's
  "a network client skips both" is `new_day` skipping the week and the month once the scenario
  has ended (`docs/exe/map.toml` renamed). For E10's refinement: the shell's messages and score
  (`E27-design.md` §5.1, §5.2, §5.5) need from the engine per colour its elimination and why (no
  hero and no town; the week without a town and its warning), the shell player's result with the
  condition's code (or the standard victory) and the values its text takes (the artifact, the
  count and creature, the amount and resource, the hero, the town), and for the score the day,
  the player's towns, the difficulty and whether that player dug up the Grail.
  *(E08 T117: the survey's correction made, `E08-economy.md` §7.2; E10's needs stay.)*

- 2026-11-12 (the client lane, E27 M0, branch `client/e27-m0`; **for E10's refinement**, the
  user's decision CLT-Q81): the game shell (E27) fills `h3_game::GameSetup` from the scenario
  screen and builds the original's setup and end screens before the engine has their rules; the
  user (CLT-Q79): **E10 adds what is missing to `GameSetup`**, the shell stubs it until then (no
  engine change ahead of E10). What the shell will need: a player's **starting bonus** (the
  original's gold, resource or artifact choice, or random) and its draw; the **handicap**; the
  **victory and defeat** conditions and a scenario's end (the end screens, the high score's
  numbers); later the computer players' turns (E13). Its readings of the original's setup (E27
  M1 T1b, `docs/epics/E27-design.md` §3) map each choice to a `GameSetup` field or to one of
  these gaps (T1b2, §3.9, §3.14, §3.15: the bonus's offer, no Artifact for a colour without a
  hero, no Resource under a Random town, the amounts the popups state, the handicap's three
  levels into game +0x1F6A8); nothing to do before E10, the order is yours. Also (CLT-Q83): a saved game is your
  adventure record (`h3-files`' replay format); what it lacks of a setup's choices is settled
  with you at E27 M4's T0.

- 2026-11-12 (the client lane, E17 M7 T4, at E17's closure; **the harness check's growth, yours
  to take when a batch needs it**, placed by the user): the harness's own check
  (`clients/capture/checks/harness-check.toml`, its recipe `capture/batches/harness-check.toml`)
  can gain what your tools placed since E08 T22a1: a monster's offers (join, join for gold, flee:
  `[encounter.opponent]`'s `disposition`, `flees`), a guarded pile's question (`[[map.pile]]`'s
  `guards`, `question`) and a walk on level 1 (`[map.underground]`); H1 checked those only on a
  local map, H2 the underground on the shipped `Arrogance`. The client lane's notes:
  `docs/lanes/client/NOTES.md`, "The adventure lane's harness".

- 2026-11-09 (the server lane, E16 M5 T3c2, SRV-Q26 and SRV-Q27; **the user's decisions**, for
  the whole game on the wire): **the fold's gaps are the stubs `SRV-STUB-1` to `-11`**
  (`docs/stubs.toml`; `-12` and `-13` are the town lane's). The server lane's fold
  (`h3-protocol`'s `wire/game/fold.rs`) keeps a viewer's view current by its projected events; its
  gate (`tests/game_fold.rs`, the 55 golden replays) asserts it equals `observe` after every
  command for every viewer but for the parts a stub names. The SRV-Q26 note below: its (1) is
  `SRV-STUB-1` (the query; landed 2026-11-11, `h3_game::reveal`, the server lane's T5c lifts the
  stub), ~~(2) `-2`~~ (lifted 2026-11-11: `MovementChanged`, protocol v42), (3) `-3` (its heroes; in the golden replays objects and boats
  appear with a reveal only), (4) `-4`. Beyond it (SRV-Q27 (2): what the rules derive):
  `-4` also what any visit tells (`AdventureObservation::known`, ADV-VIS-006, and
  `OwnHeroView::marks`: no event carries them); `-5` the views of an offer, a choice, a University
  and a hire dialog (their `…Opened` events open them without their view, nor tell a University's
  slots or a hire's counts after a purchase); `-6` the spell points a new day restores (no
  `SetSpellPoints` at the day's start); `-7` an added artifact's place (`HeroChange::AddArtifact`:
  a pickup, a reward, a battle's spoils); `-8` `GameEvent::LevelGained`'s levels at which Wisdom
  and a school were last offered (as `HeroChange::GainLevel` carries them); `-9` the slot where
  joined, hired or raised creatures land (`CreaturesJoined`, `CreaturesRaised`) and each hero's
  survivors at a battle's end (`BattleEnded`); `-10` what an object's new owner's team sees at
  `ObjectFlagged` (a mine's production, a Shipyard's boat tile); `-11` a Cartographer's type at
  `OfferAnswered`. Each lands as an event field or variant, which breaks `h3-protocol`'s build
  until mapped (the global trap: map it in the same `cross-lane` pull request, or ask the server
  lane first); once the fold folds it the gate fails until the stub is lifted from its part, and
  from the registry when no part and no marker names it.

- 2026-11-09 (the server lane, E16 M5 T1, SRV-Q26; **the user's decision**, for the whole game on
  the wire): **make the events foldable.** A server game keeps a viewer's view current by events
  only (the server lane's measurement, `docs/epics/E16-design.md` §14: diffing every viewer's
  observation costs 730 to 860 times a command's `apply` on an XL map with eight players), so a
  client must fold its snapshot with its projected events into what `h3_game::observe` gives after
  every command. What it cannot fold today (the golden replays and two eight-player XL games):
  ~~(1) **a query for what a viewer sees on the tiles it explored since a given explored grid** (a
  `BitGrid` the server keeps per viewer): their `TileView`s, explored and fringe, and the objects,
  heroes and towns it now sees there as `observe` shows them; the server asks when the viewer's
  `explored(..).count()` grew (a step's reveal has no event, `TilesRevealed` only a count)~~ (done
  2026-11-11, `adventure/srv-q26-reveal`: `h3_game::reveal`, the server lane's inbox told);
  ~~(2) **a hero's movement points left** after a step or a visit (no event or `HeroChange` carries
  them; `OwnHeroView::movement` changes with every move)~~ (done 2026-11-11: `MovementChanged`,
  told last to the owner for every hero a command changed, mapped at protocol v42); (3) **what comes into view without a
  reveal, with its public view**: another player's hero stepping into sight (`HeroStepped` gives
  its id and tiles, not its owner, type or boat), an object or a boat that appears (63 objects and
  9 boats in the golden replays); (4) **a visit that changes only the hero's points and the
  viewer's knowledge emits nothing**: `rewards-s162-d28` command 35, player 1's hero 3 visits the
  object of type 65 at (29, 23, 0), learns it is emptied and spends 141 points. Each lands as an
  event field or variant the server lane maps (the global trap from M5 T2 on); the server's fold
  test over the golden replays (M5 T3) checks them, each gap a stub there until it lands.

- 2026-11-09 (E08 T104, decision 285; **a performance task**): the gates' pairing
  (`state/channels.rs`' `pair_gates`) is 91.6 % of `build-gates-xl-144`'s build (898.7 M of
  981.3 M Ir for two builds of 8 116 gates, 63 ms each in release; about 17 M comparisons a build,
  about 26 Ir each: the scan visits the paired entries too). The levers: a compacted list of the
  unpaired entries per level (half the scan, no `is_some` test) or design §18's row-band search;
  both must keep the original's walk order and first-on-tie. Harmless on the shipped maps (a few
  dozen gates a map). Measure with `homm3-bench-regression`; re-baseline the workload down.

- 2026-11-06 (the town lane, E09 T8c2, `town/e09-t8c2`, `cross-lane` with the user's consent;
  **for when E08 places Visions**): new `h3_heroes::power` (HER-ADV-001 amended: the effective
  Power beside `attack_defense`); the 47 adventure replays' `town_ruleset_hash` line moved (town
  part v3), nothing else. A Rogue's Visions for the towns is town part data (`TownParams::
  visions_factor`, `visions_min_radius`, `visions_creatures`; the packs' `[town_visions]`, the AB
  pack's Rogue; E09 decision 31). The original's same test (`hero_visions_level` 0x4E5DE0: the
  day's cast, or expert with a Rogue; `hero_visions_reach` 0x4E5E10) also feeds the hero, garrison
  and monster popups (`hero_popup` 0x416590, `garrison_popup` 0x416F80, `monster_popup` 0x417150,
  inferred in `docs/exe/map.toml`): when you place Visions or the Rogue's effect on those views,
  read the same parameters, or move them to the spell's record and the core with us (a town part
  version to bump).

- 2026-10-31 (the map client lane, E24 M4 T0; **for the views, before E24 M4 T3**; your archive's
  note (3) of 2026-10-23, "refined with you at M4"): the map viewer takes the static art from the
  raw `.h3m` and the live part from the game's observation, through `h3-protocol` (the user, M4
  T0: ADR-0020 point 8 kept; the server lane maps it, its inbox the same day). It needs from
  you: (1) **an object identity:** each `ObjectView`, and each event that names an object
  (`ObjectRemoved` and the rest), naming its object in the map's list (`RawMap`'s objects, the
  order the viewer draws them in; the map is public, so its index reveals nothing the file does
  not), and an object added in play (T83b2a) an identity of its own (what it may reveal is your
  call, `docs/06-security.md`); (2) a random object's type as drawn, for every random kind the
  import supports (a random resource's is `object_type` today); (3) **MCL-Q21 (b)** (the user,
  2026-10-31): the import refuses a map whose data names nothing a hint can show, as A-31
  refuses an empty resource set: a Border Guard's, a Keymaster's Tent's or a Border Gate's
  colour past the eight, a resource's subclass past the seven, a dwelling's kind past its table,
  a seer's name past its table, a defeat-hero quest whose hero is −1 (the editor never writes
  them, a crafted map can; E24 design §4.12); tell this lane which you refuse already. Nothing
  is needed before the server lane's whole-game record exists; increment 9's views (your T83b2b,
  T85) are read by the same task.

- 2026-11-03 (E08 T87, decision 230; **a performance task**): the month's walk's tile test
  from the static map's flags, without a lookup of the events per tile (increment 9's design
  note §19): `month-start-xl-144` costs 12.7 M Ir a creature month's start, mostly the walk
  (`E08-log.md` T86b2). Measure against `master` (`homm3-bench-regression`).

- 2026-10-23 (T42, measured; for a performance task): **a flyer's or water walker's reach costs
  about 1.9 times a walker's query** on the XL map (`reach-lifted-xl-144`: 279 k Ir a query
  against `reach-xl-144`'s 148 k on the same games), and random play on the duel with one of each
  costs 56 % more a step (`random-play-duel-36`): `Reach::search_layered`'s second layer is the
  lever (`crates/h3-adventure/src/reach.rs`). E08 T75a: on the sea map (`random-play-sea-36`,
  one flyer and one water walker beside a 12-column sea) `Reach::expand_layered` is 52 % of the
  workload's Ir, the boats' search 4 %. E08 T75b: a boat's query on the open sea costs about
  2.29 M Ir (`reach-sea-xl-144`), 91 % in `Reach::expand_sail`; 8.7 % is `Search::landed`
  computing the land's day points (`movement::day_points`) at every landing tile, the same for
  all of a query's landings: computing it once a query is an easy cut.

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for the neutral seat**, E23 exit criterion 5): the original's battle AI for a side without a hero is complete: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (shots, melee, moves, waits, defends, the creature casts), pure, deterministic, its draws from the stream given, its choice validated (Defend if refused). The neutral seat (decision 27, your random stand-in) can call it for a monster battle's unowned side; `POLICY_VERSION` 9. A side with a hero (a computer player's) casts nothing and never retreats until E23 T5 and T6 (BAT-STUB-5).

- 2026-10-22 (ADV-Q20, decision 75; restated at ADV-Q21, decision 78, in ADR-0016's draft point 4; for the increment that revisits flying): **A-13's trade-off.** Flying stays on explored tiles (A-13 fixed for now, provisional), so wide obstacle ranges with unexplored tiles beyond them stop a flyer; the user: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway". Decide the compromise, what it tells the player of the hidden tiles, and `ADV-QUIRK-15`'s class (built by T41: the unexplored tile's row of `AdventureState::passage`, `crates/h3-adventure/src/state/passage.rs`). ~~**A-14's gap**~~ (closed by T38: the tile bit +0xC 0x40 means "no object blocks the tile", not the coast, so `hero_flight_tile_test` passes on every free land tile and a flyer visits as a walker). ~~**D-23**: Cursed Ground's movement effect is built only if batch E5's row E5-6 confirms it.~~ (E5-6 confirmed it for both grounds, T38: built by T40.)
  *(E08 T117: a flight passes over a garrison's entrance, its own team's or another's, as over any entered object's tile, ADV-MOV-009, ADV-GAR-004; batch E12 flew over none: the same increment checks it.)*

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
