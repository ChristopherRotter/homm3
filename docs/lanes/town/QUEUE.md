# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 3: growth, recruitment and the army rows, refined at
T20, `TWN-Q11`, decisions 64 to 67); done tasks in `E09-log.md`; increments 1 and 2 (closed by T10
and T20) in `E09-archive.md`. Increment 3's first round is done: the survey T21 (`TWN-Q12`), the
drafts T22a and T22b with Part E §37 to §58 and the cut (`TWN-Q13`, `TWN-Q14`; decisions 74 to 83;
the drafts accepted), batch TWN-3 (T23, decision 84), the `shared/` task (T24, decision 85: the
ADRs, the stream, the effects and flags); the build's T25a (decision 86), T25b1 (decision 87),
T25b2 (decision 88), T26a1 (decision 89), T26a2 (decision 90), T26b (decision 91) and T27
(decision 92). The building rounds follow the cut of design note §56
(decision 83): the build, E08's increment 12 merging in
parallel (each `cross-lane` number the next at merge). Each task's cut confirmed at its start;
every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T28 — the rows** (decision 83; design note §56 round 7, `cross-lane`; §52 to §54): **T28a**
the town's two bars under E08's one owner rule, admitted without a decision in the owner's turn
(TWN-ROW-001; GAM-MAN-001, -002 amended: the upper bar alone or with the town's visitor, the
listings), the garrison hero's record (the hero on the map's whole record moved into the town,
adventure state hash, `cross-lane`; the lookups of `ArmyRef::Hero`), the moves
`AdventureCommand::TownHeroes { town, change }` `Enter`, `Leave`, `Swap` (TWN-ROW-002 to -004,
each refusal typed and changing nothing), the hire's target into the garrison hero's army
(T27's `HireTarget::Garrison` resolves to it, design §49), the events and views
(`GarrisonHeroChanged`, the own hero view's place), the replay format (`town heroes <town>
enter|leave|swap`, management lines naming a town's bar), the env's `TOWN_HEROES` + 3 (encoding
v11, the AI lane's `cross-lane` consent), the protocol (the server lane's fold gate); **T28b**
what a garrison hero changes: its day and income (TWN-ROW-005; ADV-MOV-002, GAM-RES-003, -005),
growth's inputs with it (§41's open point), rows 1 and 3 of a visit of another team (TWN-SIE-001,
-002, TWN-CAP-003, GAM-BAT-001; T-5, `TWN-QUIRK-11`), level 3's view of it (TWN-VEW-001). Each
part's replays re-recorded (skill `homm3-rerecord-replays`) and bench (skill
`homm3-bench-regression`). **At its start:** confirm the cut (the design's two parts, each
possibly cut again). **Depends on:** T27 (its pull request merged); E08's T115a (#906, the visit
marks on `HeroOnMap`, which the garrison hero's record carries; merged 2026-11-13, its note in the
inbox). **Status:** next.

**E09 T29, T30 — the rest of the build** (decision 83; design note §56 round 8, each cut
confirmed at its start, the agent queues each in full when the one before it ends): **T29** the
gates (§55, §47: the scripted replays on the builds map, growth through a month, a purchase at an
upgraded dwelling and at the Portal, the rows; the census, the properties, the benchmarks); **T30**
the audit, the closure, increment 4 refined. **Depends on:** T28. **Status:** waiting.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-13 (the adventure lane, E08 T115b, `adventure/e08-t115b`; **a signature in your
  callers, mechanical**): `AdventureState::apply` takes the day of the week beside the visitor's
  stream, `(player, weekday, draws): (PlayerId, Weekday, &mut Rng)`, and `visit_held` (with the
  revisit, which runs through `apply`) a pair `(weekday, draws)`; `h3_adventure::Weekday` (from 1,
  `Weekday::FIRST`), the game's `GameState::weekday(rules)` (`CalendarParams::weekday(day)`: the
  calendar's (day − 1) mod `week_days` + 1). The Idol of Fortune's and the Temple's amounts are by
  it (ADV-BON-002); nothing of yours reads it. A new call of yours passes `state.weekday(rules)`,
  a test `Weekday::FIRST`. No version moved by it.
- 2026-11-13 (the adventure lane, E08 T115a, `adventure/e08-t115a`, `cross-lane`; **for your T28a
  and T28b, the garrison hero's marks**, your inbox note of 2026-11-11 (2)): a hero's battle
  bonuses and timed marks are `h3_adventure::VisitMarks`, one record (`bonuses()`, `timed()`,
  `add`, `add_once`, `add_timed`, `clear_bonuses`, `drop_timed(part, MarkLifetime)`,
  `week_land_points(part)`), kept for the heroes on the map in **one state block by hero id**,
  not in `HeroOnMap` (`AdventureState::visit_marks`, `change_marks`; measured: fields on the hero
  cost every fork +11 to +15 %). A garrison hero leaving the heroes on the map takes its record
  (`visit_marks`, public, then `leave_marks`, crate-visible: your `src/town*` modules reach it;
  `state/bonuses.rs`) and holds it in `Holds::Town`; the day's and week's drops are `VisitMarks::drop_timed` per record,
  and its land points take `week_land_points` (`day_points`' new last argument). Coming back onto
  the map it gives the record back through `change_marks`. `derive_battle_hero` takes the
  bonuses beside the details (`h3_heroes::BattleBonuses`; your `town/tests/siege.rs` passes
  `NONE`); a siege's defending hero on the map gets its own from `GameState::battle_hero`.
  Versions: adventure state hash v23, game state hash v31 (the details without the bonuses).
  Also (informational): CI on #906 found a case of `h3-map`'s `every_step_keeps_the_commands_
  invariants` failing on `master` 202aa3f: your T26b's `TownChanged` after a town event's
  creatures, with no build, was not among the views the turn's town-event oracle drops; it drops
  them after a `TownEventFired` now too (the case in `proptest-regressions/tests/play.txt`).

- 2026-11-12 (the adventure lane, E08 T114a2, `adventure/e08-t114a2`, `cross-lane`; **a version
  taken and the hook for your T28a**): game state hash **v29** (a map garrison's exchange among the
  decisions, tag 8), so your T26a2's bump is v30 (done, #899); adventure replay format **v13** (`m<index>:<slot>`,
  a map garrison's army) and protocol **v52** (the server lane's, `cross-lane`); every adventure
  golden replay re-recorded (versions and hash lines only). The one owner rule is in
  `h3-adventure`'s `state/manage.rs`: `held` gives each army with its bar ownership (a hero's and
  the group the player's, a map garrison its owner's when removable, nobody's when not), so your
  `ArmyRef::Garrison(town)` gets its arm there (its town's owner) and `Held::keeps_last` stays false
  for it; `h3-game`'s `Together::Exchange { hero, army }` holds a stationary `ArmyRef` (a town's
  garrison may be its `army`), and `Together::admits` refuses a map garrison its decision does not
  hold (`DecisionOpen`; alone `NotTogether`). Design note §13's *As built* (`E08-design.md`).

- 2026-11-12 (the server lane, E16 M5 part 2's refinement, SRV-Q29; **the user's decision,
  informational**): in increment 2c the server lane builds `Viewer::Spectator` (public facts only)
  through `h3-towns`' projections too (a town's view at a spectator's level), in a `cross-lane`
  pull request with the user's consent, asking your lane's review; nothing for you to build. Your
  two notes of 2026-11-10 (`town_offer`, `town_events_at_start`) go to E16 M5 T11.

- 2026-11-12 (the adventure lane, E08 T110, `adventure/e08-t110`; the capture harness, informational, for your next batch): `h3drive.gold_bonus` now clicks each row's arrow until the memory reads Gold (a heroless town player has no Artifact in its row); `hint_at` parks the mouse and reads until two reads agree; `play_turns` gives no order once the battle is decided; a recipe's `+anything odd` is kept beside the notes. New steps and reads: `clients/capture/README.md`, "Garrisons and the bonus visitables".

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
- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`; **for a town's recruitment,
  E09 decision 4**): the hire dialog is built to be reused (increment 10's design note §13,
  "E09's reuse"). `h3_adventure::HireDialog { target: HireTarget, source: HireSource }`: extend
  `HireSource` with a town's dwelling and `HireTarget` with the garrison; the map's purchase rule
  stays one (`AdventureState::hire_stock`, `hire_room`, `check_hire` → `HirePurchase`, `hire`;
  ADV-HIR-002), and so does the game's decision (`h3-game`'s `state/hire.rs`: `Decision::Hire`,
  `Answer::Hire(HireAnswer::{Buy, Close})`, the legal set of each creature's most, `hire_range`,
  `DecisionView::Hire`, `GameState::hire_dialog`; GAM-DEC-008). The env's four purchases (action
  encoding v7) take the source's order, so a town's dwelling of up to four creatures needs no
  new entry. Game state hash v25 is T95b's.
- 2026-11-04 (the adventure lane, E08 T94b, `adventure/e08-t94b`; **for the towns' external
  dwellings' growth**; its versions half done by E09 T8a1, v21): A dwelling's counts and guards as
  they stand:
  `AdventureState::dwelling_counts` and `dwelling_guards` (by object index; none of a block while
  every dwelling is as its week began); the week's refill of every dwelling is the first step of
  `AdventureState::start_week`, before the weekly objects and the camps, reading no week's or
  month's kind (ADV-DWL-003; your towns' growth is separate, survey `E08-dwellings.md` §6.1).
- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **for the towns' external
  dwellings' growth**, E09 survey §8.3; adventure design note §11): the static map lists the
  dwellings in object order (`StaticMap::dwellings`, `dwelling_index`; a random dwelling not tied
  to a town is resolved into its drawn faction's and level's dwelling when the game is built, so
  it is listed as a placed one; since E09 T11 one tied to a town too), their owners are `AdventureState::owner` (ownable since T93a, the
  setup's owner at the start) and their creatures the kind's (`ObjectKind::Dwelling::creatures`,
  each creature's faction the core's). Their counts and guards come with T94b (adventure state hash
  v20). A random dwelling tied to a town resolves since E09 T11, after the towns' draws (ADR-0017
  point 4).
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
- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town resolve since E09 T11. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 4, the Tavern (weights: only the tavern's draw reads them, survey `E09-towns.md`
  §4.4), increment 5 (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 4, the Tavern.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 4, the Tavern, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 5.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 5.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 5.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 5.
