# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 6: the town's trades, refined at T51, `TWN-Q34`,
decisions 196 to 199; its survey T52 with the checkpoint `TWN-Q35`, decisions 200 to 207; its
drafts, cut and exit criteria by T53, `TWN-Q36`, `TWN-Q37`, decisions 208 to 217; batch TWN-6
by T54, decision 218; T55 `shared/`, decision 219; T56, decision 220; T57, decisions 221 to 223; the buildings for heroes and the map are increment 7, travel and the Grail
8, decision 197); done
tasks in `E09-log.md`; increments 1 to 5 (closed by T10, T20, T30, T41 and T51) in
`E09-archive.md`. Each task's cut confirmed at its start; every entry's acceptance includes the
Definition of Done (`docs/05-testing.md`).

**E09 T58 to T61** (queued, design §118's rounds 6 to 8; each cut confirmed at its start): **Status:** T58a2 next (T58a1 done 2026-11-26, decision 224, `E09-log.md`). T58 the market
at a town (§103 to §107, `cross-lane`): **T58a2** the listing (`legal_commands` lists
`TownCommand::Market` where the window opens and offers a legal answer, one lookup a town as
`legal_builds`' others, the town lane's `TRAPS.md`; `TWN-STUB-22` lifted), the env's opening
entry (§115: 1 entry by the town's entrance, the encoding's next version), the plans
(`h3-map`'s `play.rs`: `trades::trade_open` and the oracles naming a town's window, today map
markets only), the town-map seeds and the workloads that move (`legal-towns-36`, the
`random-play-*` with towns; skill `homm3-bench-regression`); **T58b** the artifact sale
(`MarketAnswer::SellArtifact`, `MarketParams::artifact_sale`, `TWN-QUIRK-18`, `-20`, its env
entries and word, the protocol); T59 the services (§110 to §113: T59a `BuyWarMachine` and
`BuildBoat`; T59b `University` with T-23 and `Transform`; each with its env entries, word and
protocol, `cross-lane`); T60 the trades map, its replays, the census, the properties, the
benchmarks (§117); T61 the audit against §119's criteria, increment 6 closed, increment 7 refined.

## Inbox

- 2026-11-26 (the AI lane, E31 T2d, `ai/e31-t2d`, `cross-lane` with the user's consent, AI-Q35;
  **in your paths, nothing to do**): `h3_game::GameState::growth_inputs(rules)` is new and public
  in `src/town/growth.rs` (your `inputs`, by `TownId`): the computer's turn start adds the next
  week's growth (`h3_towns::growth`) to a town's available creatures from the week's fifth day
  (`ai_resource_weights` 0x428740 over `town_dwelling_growth` 0x5BFB60). Three of your tests'
  builders are `pub(crate)` now (`towns_core_pack`, `town_pack`, `rules_with_adventure` in
  `src/town/tests.rs`) for the computer's tests. The weights' rate counts the towns whose
  buildable mask holds the Marketplace by T56's role `marketplace` (E31 T2e; the request is
  dropped).

- 2026-11-24 (the AI lane, E31 T1b, `ai/e31-t1b`, docs only; **for your decision and your
  planning, nothing to build now**): the original's computer player's rules in towns (survey
  `docs/rules/surveys/E31-computer-player.md` §3.6; E31 decision 2: you decide whether the engine
  copies each, keyed on `human`). Row 5 (with the adventure lane): the Castle Gate's jump made even
  when the purchase of building 22 fails; from an ally's Inferno town without a gate the walk buys
  it there (§5.5 #59: who builds and who pays wants an experiment). Row 7 (with the adventure
  lane): an artifact at the Artifact Merchants costs trunc(cost / rate), one unit under the human's
  half-up rounding in 61 % of costs (`ai_artifact_market_price` 0x524550; E09 increment 6). Two
  checks for the port: the trades written into the treasury at the market's rate (§2.3) against
  the human's screen (`market_resources_proc` 0x5ECFE0) before the policy relies on their
  rounding, and the allies' shares, which the receiver is not asked (as your transfer). One question: the town visit upgrades
  creatures in the town's and the visitor's armies (§3.3) and the build choice values the upgrades
  (§3.1); no rule for a town's creature upgrade turned up in the catalog: where is it planned? Every other
  town decision of the computer is a choice the policy issues as your commands (§3.6's choices);
  what it reads of your API: `docs/epics/E31-design.md` §3, at E31's increment 3.

- 2026-11-24 (the rmg lane, E14 T1c2, `rmg/e14-t1c2`; **a question for you**: survey
  `docs/rules/surveys/E14-generator.md` §3.3.4): the generator writes each colour's main town as
  its zone's first town's entrance, as the editor does; in `l2-2p-c4-islands` (template Dragons,
  zone 32 asks for no colour town) purple's main town is a **neutral** town, which
  `GameSetup::validate` refuses (TWN-SET-001's owner half, the cut of 2026-11-04); the user kept it
  as the original in every profile (RMG-QUIRK-23, no flag). The original
  accepts it (observed: the town stays neutral, purple's hero stands on its entrance as its
  visitor, purple owns no town; `main_town_heroes` 0x4CA040 has no owner test). Whether to widen
  the check to that is yours (with the user); the writer also lacks (a) the main town's type byte
  0xFF (it writes the faction) and (b) a main town of a town not of its colour (`main_towns`).
  **Update (E14 T1c3, RMG-Q2):** the user chose the writer with its gaps closed, so (a) and (b)
  in `writer/town.rs` are needed before E14 increment 7 (after E14's increments 2 to 6); one
  still open then is closed by the rmg lane, `cross-lane` with the user's consent.
- 2026-11-23 (the adventure lane, E08 T141a, `adventure/e08-t141a`, `cross-lane` with the user's
  consent, E08 decisions 412, 413; **in your paths, for your review**): the Prison. (1) The pool:
  `GameSetup::prisoners` are `GamePrisonerSetup`s (the body read as a placed hero's): `HeroPool::new`
  takes each `hero_type` as a prisoner's (TWN-SET-005 kept), `GameSetup::validate` refuses a
  prisoner's type the ruleset lacks, a placed hero's or an earlier prisoner's
  (`GameSetupError::PrisonerType`), each prisoner is a hero of the store after the build's heroes
  with its type's record (`set_record`), and the freeing calls the new `HeroPool::free_prisoner`
  (taken, the colour's bit added); `import::town::pool` no longer reads the bodies (its third
  output gone). `town::rows::most_heroes` and `TavernHeroView::new` are `pub(crate)`. (2) **Each
  player's list of heroes** (GAM-HER-003): `HeroOnMap::listed`, kept by the adventure in your
  `town::rows` (`arrive` puts a hero last, `enter` and `swap` take the visitor out first, as the
  original's `town_garrison_to_visitor`, `town_visitor_to_garrison` and `town_swap_heroes`) and in
  `place_hired` through `arrive`; a hired hero's literal in `h3-game`'s `town/hire.rs` gives
  `listed: 0` (replaced). The Thieves' Guild's best hero row takes the first strictly greater in
  the original's list order (survey `E09-tavern.md` §3): `AdventureState::hero_list` now gives
  that order, if you want the row's tie-break to follow it (today by id). (3) Your tests:
  `town::tests` is `pub(crate)` with `towns_rules` and `towns_setup` (the game's prison tests use
  them); the tests with prisoners put a Prison on their maps (`crate::tests::imprison`, `held`);
  `custom.rs`' records moved one id up (the prisoner's record first). (4) `h3-map`'s
  `play.rs`: the town check now expects the week's start to fill every town's Vortex flag (T49a2's
  `Towns::start_week`, found by this branch's property run: the "rebuilt" check failed on it).

- 2026-11-21 (the adventure lane, E08 T137b, `adventure/e08-t137b`; **for your planning, nothing
  to build**): from E08 T141a every prisoner is a hero in the store with details (a placed hero's:
  levels at load, army, skills, artifacts, spells), in no list, still `HeroStatus::Prisoner` in the
  pool; a prison's freeing takes the type for the visitor's colour (its mask's bit added) and
  shares the hire's limit (the town part's most heroes, TWN-TAV-002). Code that reads the store
  by status sees them (design note `docs/epics/E08-design.md` §15).

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-17 (the map client lane, E24 M4 T1b1, `map-client/e24-m4-t1b1`; **the capture harness, rebuild `xdd.dll`**): `clients/capture/xdd.c` takes a new command, `<seq> seed <value>` (its 8.: the main thread's C library seed set at its next clock read, so that a computer player's turn draws alike in every run; `Game.seed`, and `h3drive.to_scenario(game, seed=...)` seeds just before Begin), and its acknowledgement has a sixteenth field (the seeds set): `h3drive.ACK_FIELDS` is 16, so a run with an `xdd.dll` built before fails as stale where it reads the acknowledgement whole (a forced roll, a batch's or a sequence's checks, a seed): rebuild it with `clients/capture/setup.sh`. Nothing else of `h3drive`, `h3adv` or `h3mem` changed in behaviour (`Mem.other_turn` is new; `Mem.status_items` also returns kind 5's items, ids 2190 to 2193).

- 2026-11-17 (the map client lane, E24 M4 T1a, `map-client/e24-m4-t1a`; **the capture harness, rebuild `xdd.dll`**): `clients/capture/xdd.c` forces two more draws (`week_name` 0x4C87A6, the week's name, and `month_name` 0x4C8FCC, a normal month's; `h3drive.ROLLS` grew by two), so a batch or sequence run with an `xdd.dll` built before fails the count of redirected calls: rebuild it with `clients/capture/setup.sh`. Nothing else of `h3adv` or `h3mem` changed in behaviour (`Mem.calendar` is new: the week's and the month's kinds and names).

- 2026-11-13 (the adventure lane, E08 T115b, `adventure/e08-t115b`; **a signature in your
  callers, mechanical**): `AdventureState::apply` takes the day of the week beside the visitor's
  stream, `(player, weekday, draws): (PlayerId, Weekday, &mut Rng)`, and `visit_held` (with the
  revisit, which runs through `apply`) a pair `(weekday, draws)`; `h3_adventure::Weekday` (from 1,
  `Weekday::FIRST`), the game's `GameState::weekday(rules)` (`CalendarParams::weekday(day)`: the
  calendar's (day − 1) mod `week_days` + 1). The Idol of Fortune's and the Temple's amounts are by
  it (ADV-BON-002); nothing of yours reads it. A new call of yours passes `state.weekday(rules)`,
  a test `Weekday::FIRST`. No version moved by it.

- 2026-11-12 (the server lane, E16 M5 part 2's refinement, SRV-Q29; **the user's decision,
  informational**): in increment 2c the server lane builds `Viewer::Spectator` (public facts only)
  through `h3-towns`' projections too (a town's view at a spectator's level), in a `cross-lane`
  pull request with the user's consent, asking your lane's review; nothing for you to build. Your
  two notes of 2026-11-10 (`town_offer`, `town_events_at_start`) go to E16 M5 T11.

- 2026-11-11 (the adventure lane, `adventure/srv-q26-reveal`, `cross-lane` with the user's consent;
  **in your paths, for information**): SRV-Q26 (a)'s query `h3_game::reveal` names the towns a
  viewer newly explored with their views through `town::view::views_of(state, rules, viewer,
  towns)`, a new `pub(crate)` function beside `town_views` (the `Seer`'s view of each, in order;
  the viewer's explored tiles read from the state). Your `Told`'s "a town the viewer comes to see
  by exploring" is that query's now. Tested in `town/tests/view.rs`
  (`a_town_explored_since_a_grid_is_revealed_with_its_view`); `town/tests/capture.rs`' `step`
  drops the `MovementChanged` events a command now tells last (SRV-Q26 (b)). Keep `views_of` in
  step with `town_views` when the views change.

- 2026-11-10 (the adventure lane, E08 T105b2, `adventure/e08-t105b2`, `cross-lane`; **a town's
  revisit, for your visit**; increment 11's design note §13): `AdventureCommand::Revisit { hero }`
  (ADV-TEL-008) visits the entered object under a hero again as a step's last step onto it would,
  no points, no step, no guard test: for a hero standing on its own town's entrance that is
  `visit_town` with `Visit::Enter` (`town_visit` as it stands), at 0 points too and as often as
  the player likes. `legal_commands` lists it for every such hero after the `BuildBoat`s (and
  before a town's builds once T18b2 lists them: `revisitable`'s place is after `buildable_shipyards`
  in `h3-game`'s `legal.rs`). Whatever a town's visit opens later (its screen, a garrison
  exchange) opens at a revisit too; nothing moved in your hashes. The replay format is v12
  (`revisit <hero>`): a town bump of the format is v13.

- 2026-11-10 (the server lane, E16 M5 T3d; **for when `TWN-STUB-13` is lifted**): your
  `TownChanged`, `TownStackSeen` and `TownIncomeSeen` (E09 T18b2a) are mapped and folded in
  `h3-protocol` (the fold replaces the viewer's town view); `SRV-STUB-12` and `-13` are lifted.
  The fold gate (`h3-protocol`'s `tests/game_fold.rs`, the 55 golden replays) now exempts the
  garrisons and the town owner facts only for a reveal and for `TWN-STUB-13` (3 viewer-commands:
  another team's town falling from level 3 to 2 as a Visions holder's reach is lost). Once
  Visions' reach is told, the gate fails with "lift [...] from it": drop `VISIONS` from those two
  parts in the same pull request (`cross-lane`), as a new event field or variant is mapped there
  (the global trap). This note replaces the server lane's of 2026-11-09 (T3c2).
- 2026-11-09 (the adventure lane, E08 T105a1, `adventure/e08-t105a1`; **a signature, for your
  callers**; increment 11's design note §13, §18): `AdventureState::apply` takes `(player, draws)`
  in place of `player` (`draws: &mut Rng`, the acting player's stream: a visit's draw, a
  monolith's exit, comes from it) and `visit_held` takes `draws` before the sink; the game passes
  them from `GameState::adventure_for(player)` (the adventure state, the store of heroes and that
  player's stream). Every caller in the adventure lane's paths is updated, none of yours called
  them; a new caller in `h3-game`'s `src/town*` (a town command that visits) uses
  `adventure_for`, and the adventure crate's tests `testing::stream()`. Nothing moved in a hash.
- 2026-11-04 (the client lane, E17 M6 T3c3, `client/e17-m6-t3c3`; **when you change
  `h3-towns`' or `h3-pack-towns`' public API**): the battle client's binary now compiles the packs'
  town part (`h3_pack_towns::compile`) and reads `TownRuleset::town`, `TownRecord::building` and
  `buildings`, `Building::key` and `slot` and `BuildingId::from_index` (a siege's buildings and a
  Grail building by key in the stats dialog, the original's map by slot; design E17 §12.11). CI's
  clients job watches both crates (#704): a change there runs `clients/scripts/check.sh`, locally
  `scripts/verify-once.sh clients --base origin/master`; a dependency change updates
  `clients/Cargo.lock`.
- 2026-11-03 (the battle lane, CLT-Q40; its siege half done by T6b, `QUEUE-ARCHIVE.md`): the
  owner's Grail building that adds a hero's morale or luck (the Colossus, the Spirit Guardian)
  names itself as `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when
  the game gives it. Increment 8, travel and the Grail (decision 197).
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 7, the buildings for heroes and the map (decision 197).
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 7, the buildings for heroes
  and the map (decision 197).
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 6, the town's trades (decision 197).
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 7, the buildings for heroes and the map
  (decision 197).
