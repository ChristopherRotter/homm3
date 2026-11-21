# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 5: the mage guild, refined at T41, `TWN-Q30`, decisions
161 to 164; its survey T42a and T42b with the checkpoint `TWN-Q31`, decisions 165 to 171; its
drafts, cut and exit criteria by T43, `TWN-Q32`, `TWN-Q33`, decisions 172 to 181; batch TWN-5 by
T44, decision 182; the drafts in the ADRs by T45, decision 183; the part v9 by T46, decisions 184,
185; T47a cut in two, T47a1's module, decisions 186, 187; T47a2 whole, decision 188; T47b whole with the inbox's guild fix, decision 189; T48a whole, decision 190; T48b whole, decision 191; the services are increment 6, travel and
the Grail 7, decision 162); done tasks
in `E09-log.md`; increments 1 to 4 (closed by T10, T20, T30 and T41) in `E09-archive.md`. Each
task's cut confirmed at its start; every entry's acceptance includes the Definition of Done
(`docs/05-testing.md`).

**E09 T47 to T51** (queued, design §98's rounds 5 to 8; each cut confirmed at its start; **T49a**
next): T47 the guild drawn and shown (§84 to §87, §94; T47a cut in two, decision 186: T47a1
`h3-towns`' guild module and T47a2 the map's town spell sets with import v28, the store v4, the
game's draw at the build, game state hash v38, every golden re-recorded, both done; **T47b** the
owner's view (TWN-VEW-004, the protocol v78), done (decision 189)); T48 learning (§88, §89: T48a the set
with the Aurora and T-18, the town's own triggers, `TWN-STUB-6` and `-15` lifted, done (decision
190); T48b the visit's end, a capture, a siege's end, `cross-lane`, done (decision 191)); T49 the spell points and the commands (§90
to §93, §95: **T49a** the night's refill, the Mana Vortex, `Open`, its env entry and replay word; T49b
`BuySpellBook`, its entry and word); T50 the guild map, its replays, the census, the properties,
the benchmarks (§97); T51 the audit against §99's criteria, increment 5 closed, increment 6
refined.

## Inbox

- 2026-11-21 (the adventure lane, E08 T137b, `adventure/e08-t137b`; **for your planning, nothing
  to build**): from E08 T141a every prisoner is a hero in the store with details (a placed hero's:
  levels at load, army, skills, artifacts, spells), in no list, still `HeroStatus::Prisoner` in the
  pool; a prison's freeing takes the type for the visitor's colour (its mask's bit added) and
  shares the hire's limit (the town part's most heroes, TWN-TAV-002). Code that reads the store
  by status sees them (design note `docs/epics/E08-design.md` §15).

- 2026-11-21 (the town lane itself, E09 T48a's verify; **a pre-existing hygiene failure, to
  triage**): `h3-map`'s `a_viewer_learns_nothing_it_cannot_see` (`src/tests/view.rs`, the
  adventure lane's file) failed on a random case that fails on `master` (c0ce729's code) too, so
  not T48a's, the seed
  `cc cc8693ab49d6fe44669fbab37bc9a94db548674cbd75e9d3d0027203fa6895e5` (an RoE plan of side 14,
  two levels). The two games tell the viewer (blue) different `TownStackSeen`s after its town 1 is
  captured by red: `HeroRemoved` of blue's hero 4 (a garrison hero with an empty army,
  `TWN-QUIRK-11`'s capture?), then the town's garrison stacks, five in one game, two in the other
  (red's hidden armies differ). Not added to `proptest-regressions/tests/view.txt` (it would make
  every pull request red): add that line with the fix. Likely the town lane's (a captured town's upper
  bar, TWN-VEW-002/-003, TWN-CAP-001) or the property's (`outcome`, the end of the comparison):
  the user's call when to take it.

- 2026-11-20 (the adventure lane, E08 T133a2 and T133b, `adventure/e08-t133a2`,
  `adventure/e08-t133b`; **for your increment 6's planning, nothing to build**): the map's
  markets are built whole: the checks (`h3_adventure::AdventureState`'s `trade_lot`,
  `check_screen`, `check_sale`, `sell`, `check_buy`, `buy`; `MarketError`; A-46 fixed as
  ADV-QUIRK-48: a purchase refused before paying without room) and what a window offers
  (`market_offers`: `MarketOffers` with the rates), and the game's decision (`h3_game`'s
  `Decision::Market`, `MarketAnswer`, `MarketView`; GAM-DEC-011). **ADV-Q60 answered (a)** (E08
  decision 384): `[params.market]` names the cost resource (`cost = "gold"`, `MarketParams::cost`),
  the checks read it themselves (no `value` argument); your Marketplace and Artifact Merchants
  price on the same rates and resource.

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-19 (the adventure lane, E08 T132b, `adventure/e08-t132b`; **a version taken, nothing
  to do now**): map import v27 is taken (the seven trade classes typed at subclass 0: 2, 7, 35,
  99, 106, 213, 221 as 99); your next import change takes v28. Your six taverns replays (#1053)
  were re-recorded by T132b's follow-up (`adventure/e08-t132b-taverns`): only their
  `import_version` moved. `h3-map`'s
  original-data example gained `increment_14` after your `map_taverns` (`Peaks`' field 15).

- 2026-11-19 (the adventure lane, E08 T132a1, `adventure/e08-t132a1`; **the market's rates built,
  for your Marketplace; nothing to do now**): `h3_adventure::MarketParams` (exported with `Lot` and
  `MarketParamsError`) computes the original's rates in `X87`: `resource_lot(count, give, take)`
  (screen 0), `creature_lot(count, gold cost, take)` (screen 4), `artifact_price(count, cost, pay)`
  (screen 2), each `None` where the original has no rate, `count` from 1 to the parameters' rows
  (your D-70 mapping of a town's Marketplaces to a row is yours); D-70's rows hold as the original's
  (wood for gold 37, 62, 112 at 2, 4, 8). The part's `[params.market]` compiles into it since E08
  T132a2: `AdventureRuleset::params().market()` (the base pack's values and factors). **A correction** to survey `E08-trades.md` §4: a precious resource
  for wood or ore is **2 : 1** at five Marketplaces (the first table had 1 : 2; the code read
  again). The artifact sale (screen 3) stays yours to add on the same module.

- 2026-11-19 (the adventure lane, E08 T131, `shared/e08-t131`, `cross-lane`, docs only; **the ADRs'
  new parts, for your increment 5's amendments; nothing to do now**): ADR-0016 points 1, 4 and 6
  and ADR-0017 points 1, 6, 8, 9 and 10 now hold E08's increment 14 (the market with its rates in
  the adventure part, the Black Market's stock, the Altar, the Hill Fort, the factory; the
  decisions GAM-DEC-011 to -013), each placed after your increment 4's text; ADR-0017 point 9
  names the next versions after yours (part v23, map import v27, adventure state hash v27, game
  state hash v35, replay format v20, encoding v14), whichever lane merges first taking them.
  Batch E14 (survey `E08-trades.md` §21) confirmed D-71 and the rates at five Marketplaces
  (E14-1, E14-5); the Hill Fort's elementals follow the enabled packs (E08 decision 378, as your
  E09 decision 14's moat).

- 2026-11-18 (the adventure lane, E08 T127b, `adventure/e08-t127b`, docs only; **the market's
  rates' home and the versions both lanes plan; nothing to do now**): at T127b's checkpoint
  (ADV-Q58, E08 decision 370) the user put the market's rates in **the adventure part**
  (`[market]`: two value tables by resource, three factor tables by the count 1 to 10 in
  hundredths) with the computation in `h3-adventure`'s `market` module (lots; the original's
  binary32 steps in `h3_kernel::arith::X87`), built by E08 T132a. Your Marketplace (in your
  `h3-game` modules) calls it with the player's count, as you reuse the hire dialog; screen 3
  (the artifact sale, `market_artifact_sale_rate` 0x5ECDC0) is yours to add on the same module.
  `Decision::Market` (GAM-DEC-011, E08 T133b) holds the map's markets; a town source for it or a
  decision of your own is yours to design (E08 design note `docs/epics/E08-design.md` §2, §13).
  E08's increment 14 plans map import v26 (T132b), replay format v19 and env v13 (T135a) and a
  protocol bump per wire task, as your T39a does: whichever merges first takes the number.

- 2026-11-18 (the adventure lane, E08 T127a2, `adventure/e08-t127a2`, docs only; **the user's
  answers on the shared market rules, for your increment 5; nothing to do now**): at T127a2's
  checkpoint (ADV-Q57, E08 decisions 365, 366) the user chose **D-71 the original's** (a
  creature's sale rounds half up after the binary32 steps, survey `E08-trades.md` §4: your
  Freelancer's Guild too) and **A-46 fixed** (a purchase is refused before paying when the hero
  cannot receive the artifact: your Artifact Merchants too). D-70 stays yours. Also read there:
  `upgrade_cost` 0x54E750, the town creature dialog's upgrade price (per resource the difference
  times the count), which the Hill Fort scales (§13).

- 2026-11-18 (the adventure lane, E08 T127a1, `adventure/e08-t127a1`, docs only; **the market's
  rates as read, for your increment 5; nothing to do now**): survey
  `docs/rules/surveys/E08-trades.md` §3, §4 and §11 (the address map's `market_*` entries). The
  town's Marketplace, Artifact Merchants and Freelancer's Guild and the map's Trading Posts,
  Freelancer's Guild and Black Market share one window (`market_window_run` 0x5EA130) of five
  screens (resources for resources, resources to a player, resources for artifacts, artifacts for
  resources, creatures for resources). **The count:** a town trades at the current player's towns
  with the Marketplace built (building 14, at most 10); every map object at five. **The rates:**
  two value tables (250 wood and ore, 500 the rest, 1 gold; an artifact's price in half of them),
  three binary32 factor tables by the count (resources 0.1 to 0.5, artifacts 0.2 to 0.6, creatures
  0.3 to 1), each rate a binary32 quotient rounded half up in 53 bits (the reciprocal's binary32
  rounding decides the halves: the engine reproduces the steps through `kernel::num` or holds the
  finite tables checked against them; the decision is the adventure lane's T127b design, built
  once by whichever lane needs it first, decision 361). **Disputes:** D-70 is yours alone (wood or
  ore for gold at 2, 4 and 8 Marketplaces: the original's 37, 62, 112 against VCMI's 38, 63, 113;
  none at five); D-71 (the creature sale's rounding against VCMI's ceiling) and A-46 (a purchase
  from a hero who can hold no more artifacts takes the price and loses the artifact, the Artifact
  Merchants too) go to the user at the adventure lane's T127a2 checkpoint; the rule built once
  serves your guild and merchants as well. Not read
  here: the artifact sale's screen (`market_sell_artifact_proc` 0x5EDF60) and the merchants'
  monthly draw (`new_month` 0x4C8F40, game +0x1F664).

- 2026-11-18 (the adventure lane, E08 T126, `adventure/e08-t126`; **for your increment 5's planning, nothing to do now**): E08's increment 14 is **the trades and services** on the map (E08 decision 361, ADV-Q56): the Black Market (7), the War Machine Factory (106), the Altar of Sacrifice (2), the Trading Posts (99, 221), the Hill Fort (35) and the Freelancer's Guild (213); the map Tavern (95) and the Den of Thieves (97) stay yours (E09 decision 106). The Trading Posts and the Freelancer's Guild trade at the market's rates, which your increment 5 builds with the town's Marketplace: **the rates are built once, by whichever lane needs them first, and the other reuses them** (as E09 decision 4's hire dialog). The adventure lane's survey (E08 T127a, `docs/rules/surveys/E08-trades.md`, next) reads the original's rates and how a Trading Post's relate to the Marketplaces a player owns, and tells you here what it finds before either lane builds them.

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
  the game gives it.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 4, the Tavern (weights: only the tavern's draw reads them, survey `E09-towns.md`
  §4.4; read by T32: `E09-tavern.md` §2.2, the Complete's file has the nine columns, harvested by T36),
  increment 5 (chances: read by T42a, survey `E09-guild.md` §2: the nine columns at the spell
  record's +0x44; their home in the packs is T43's call, §2.3).
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 6, the services (decision 162).
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 6, the services.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 6, the services.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 6, the services.
