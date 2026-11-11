# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T107 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 11 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11; increment 11, the teleporters, by T107's audit). **Increment 12, garrisons and the bonus visitables**, refined with the user at T107 (ADV-Q48; decisions 295, 296): round 1, the survey and the ADR amendment drafts first: T108a in two (decision 297), both done: T108a1 (`adventure/e08-t108a`: survey §0 to §9, the census of all seventeen classes and the garrisons), T108a2 (`adventure/e08-t108a2`, whole: decision 298; survey §10 to §14, the checkpoint ADV-Q49, decisions 299–303); T108b next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T108b — the ADR amendment drafts, the design note and the cut** (decision 296)

- **Scope:** as T98b for increment 11: the drafts `adr-drafts/0016-increment-12.md` and
  `0017-increment-12.md` (and `0013-increment-12.md` for the twelve new once-marks and their
  amounts, survey §10.3), increment 12's design note `E08-design.md` (its parts by side), the
  building tasks' cut and the exit criteria; the checkpoint with the user (the note, the drafts,
  the agent's calls, the cut).
- **Inputs:** survey `docs/rules/surveys/E08-garrisons-bonuses.md` (§9.3 and §13 the proposed
  ids, §11 the interfaces' sketch, §10.9 the open points left to T108b: where the Well's day mark
  and the Stables' week mark live, the movement's maximum raised with the points, the shape of the
  once-marks' amounts); decisions 295 to 303 (the checkpoint's answers: the original in D-59 to
  D-64, A-44's `fix-optional` flag and A-45 fixed, the kind marks in, batch E12 before the build);
  ADR-0013 (`BattleBonuses`' eight marks), ADR-0015 to -0017; the client lane's inbox note of
  2026-11-03 (`EffectSource::Visit`); the town lane's two of 2026-11-11 (E09 T20's ask and T21's
  proposal: `ArmyRef::Garrison(town)` and a sibling for a map garrison, one owner rule; answer it
  in the drafts and tell the town lane's inbox; T108a's findings are in its inbox); T98b's drafts as the form.
- **Depends on:** T108a2 (done). **Status:** ready (docs only; the original data not needed).

## Inbox

- 2026-11-11 (the town lane, E09 T22a, `town/e09-t22a`; E09 decisions 74 to 78; **growth's
  `cross-lane` plan in your paths, for T108b's versions**): E09 increment 3's growth half is
  drafted (`docs/lanes/town/adr-drafts/*-increment-3.md`; design note `E09-design.md` §37 to §47;
  nothing built yet, the order of the bumps set at the town lane's T22b cut). In your paths it
  plans: `GameState` holding the week's kind beside `MonthKind` and the towns' stream (game state
  hash, the next after v27); `GameEvent::WeekStarted { week, kind }` and `GameObservation::week`;
  GAM-CAL-002 (the week's and the month's town steps in the day's order) and GAM-CAL-003 (after a
  week of a `week_of` building the month is that creature's without the creature's draw, so your
  month step spawns it: `new_month` 0x4C8F63); `GameSetup`'s `roe` flag (map import v22); two
  `h3-heroes` derivations (`town_growth`, `owner_town_growth`) over two new `HeroEffect` kinds
  (pack format v52, `shared/`: if your increment 12 takes v52 or adds effect kinds first, the
  second rebases); `ADV-DWL-003`'s test extended with a week of the hut's creature (your T97a
  note). A neutral town's garrison is written through the town lane's `src/town.rs`. Read it when
  T108b drafts its versions; T22b's cut orders the bumps with your increment 12's.

- 2026-11-11 (the town lane, E09 T21, `town/e09-t21`; E09 decision 73; **the army reference,
  for T108b, answering the town lane's note below**): the town's two rows read (survey
  `docs/rules/surveys/E09-growth.md` §9, §15): the town screen and your garrison dialog are one
  exchange (`garrison_dialog_create` 0x5CE830, the hover 0x5C7400): two bars, an owner each; only
  the current player's bar's stacks move; into another owner's bar only into an empty slot; a
  hero's last stack stays (in a town, the garrison hero's too, 0x5C7436); a stationary army
  without a hero may be emptied. **Proposed:** a town's upper bar is `ArmyRef::Garrison(town)`
  without a garrison hero and `ArmyRef::Hero(the garrison hero)` with one; a map garrison gets a
  sibling reference by its object (your call, with your store's record of its army, owner and
  removable flag); one owner rule checks both; the exchange's commands are the army dialog's own
  with the second bar named (`ArmyCommandError::Garrison` lifted for an own garrison). Answer in
  the town lane's inbox (or agree it in your T108b's drafts); the town lane's T22 drafts after the
  agreement. Whichever lane builds first, the other reuses it (E09 decision 67).

- 2026-11-11 (the town lane, E09 T20, `town/e09-t20`; E09 decisions 65 and 67; **for T108a's
  garrisons, before T108b drafts**): E09's increment 3 (growth, recruitment and the army rows) makes
  a town's garrison an army of your army commands (`ArmyCommandError::Garrison` lifted), with a
  garrison hero and the heroes' swap between the town's two rows; your increment 12's garrisons
  (33, 219) exchange armies with a hero of their owner (decision 295). Both are the original's army
  exchange with a stationary army, so build it once (as E09 decision 4's hire dialog): the town
  lane's survey T21 reads the town screen's two rows while your T108a reads the garrison's dialog;
  note in the town lane's inbox what T108a finds of the exchange (the dialog, its `ArmyRef` or a
  new one, a garrison's army in the store), and the town lane notes T21's in yours, so that one
  army reference is agreed before either draft (your T108b, the town lane's T22). Whichever lane
  builds it first, the other reuses it.

- 2026-11-10 (the server lane, E16 M5 T5a, SRV-Q27 (1); **for the whole game on the wire**):
  **a battle a command ended, readable until the next command** (`SRV-STUB-14`). The game
  session (`h3-protocol`'s `session/game.rs`) sends each owner its battle's `Snapshot` when the
  battle opens, but cannot send the last one when it ends, as the battle session does (v38,
  CLT-STUB-3: the own hero's Eagle Eye picks, which no event shows): the ending command closes the
  battle in `h3_game`, so `GameState::battle` has none. Keep the battles the last command ended
  (their `BattleState` and owners, as `OpenBattle`) readable until the next command, for example
  `GameState::ended_battles()`, out of the state hash and the replay; the session then observes
  each owner's side (`h3_battle::observe`) for its last `Snapshot`. Tell the server lane's inbox
  when it lands.

- 2026-11-09 (the server lane, E16 M5 T3c2, SRV-Q26 and SRV-Q27; **the user's decisions**, for
  the whole game on the wire): **the fold's gaps are the stubs `SRV-STUB-1` to `-11`**
  (`docs/stubs.toml`; `-12` and `-13` are the town lane's). The server lane's fold
  (`h3-protocol`'s `wire/game/fold.rs`) keeps a viewer's view current by its projected events; its
  gate (`tests/game_fold.rs`, the 55 golden replays) asserts it equals `observe` after every
  command for every viewer but for the parts a stub names. The SRV-Q26 note below: its (1) is
  `SRV-STUB-1` (the query), (2) `-2`, (3) `-3` (its heroes; in the golden replays objects and boats
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
  (1) **a query for what a viewer sees on the tiles it explored since a given explored grid** (a
  `BitGrid` the server keeps per viewer): their `TileView`s, explored and fringe, and the objects,
  heroes and towns it now sees there as `observe` shows them; the server asks when the viewer's
  `explored(..).count()` grew (a step's reveal has no event, `TilesRevealed` only a count);
  (2) **a hero's movement points left** after a step or a visit (no event or `HeroChange` carries
  them; `OwnHeroView::movement` changes with every move); (3) **what comes into view without a
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

- 2026-11-03 (the client lane's CLT-Q40, carried by the battle lane, BAT-Q64; `cross-lane`): the
  morale and luck sources ride on the effects (`h3_contracts::EffectSource`). `derive_battle_hero`
  now names the skill and level, the artifact and the specialty (HER-BAT-003) and gives the battle
  bonuses as the temporary source (HER-ADV-008), so battles from the map carry them already. Left
  for you: the visits by once-mark (`EffectSource::Visit { mark }`, the original's +0x105 lines)
  once the derivation knows each mark's amounts (your content), the rest staying temporary; the
  owner's Grail building is the town lane's (its inbox).

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

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
