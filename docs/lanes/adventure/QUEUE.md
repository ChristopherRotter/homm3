# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T107 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 11 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11; increment 11, the teleporters, by T107's audit). **Increment 12, garrisons and the bonus visitables**, refined with the user at T107 (ADV-Q48; decisions 295, 296): round 1 done: T108a in two (decision 297): T108a1 (`adventure/e08-t108a`: survey §0 to §9, the census of all seventeen classes and the garrisons), T108a2 (`adventure/e08-t108a2`, whole: decision 298; survey §10 to §14, the checkpoint ADV-Q49, decisions 299–303); T108b (`adventure/e08-t108b`, whole: decision 304; the design note and the three drafts, the checkpoint ADV-Q50, decisions 305–309, the cut T109 to T117). Round 2: T109 in two at the code's seam (decision 310): T109a done (`adventure/e08-t109a`, `cross-lane`: the writer's seventeen classes, the spec's garrisons; no spell points, decision 311); T109b done (`adventure/e08-t109b`, `cross-lane`, whole: decision 312; the specs E12a and E12b, the School of War in the writer, decision 313; E12-5 on the start's reveal, decision 314); round 2 done. Round 3: T110 done (`adventure/e08-t110`, whole: decision 316; the harness's reads and steps, the recipes `E08-batch-12a` and `-12b`, three runs each, the templates filled); T111 next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T111 — batch E12 scored** (decisions 299 to 303, 309, 312 to 314, 316; then **T112**)

- **Scope:** as T101 for increment 11: score T110's two filled templates against survey
  `E08-garrisons-bonuses.md`'s readings and the specs' predictions, row by row (E12-1 to E12-17),
  in a new survey §15 (each row: as read, or not, with the reading or source it settles; D-59 to
  D-64, A-44, A-45 where a row decides one); fold the verdicts for T112 (the three drafts'
  points they confirm or change) and checkpoint with the user where a row goes against the
  reading (ADV-Q51). Beyond the rows (T110's log): the Magic Wells and the Idol were **stood
  on** (entrances, not visited from a neighbour as the experiments README's "Increment 12's
  objects" and the design's §1 say; the water objects were visited from the neighbouring water):
  decide what the build takes and where the README is corrected (battle-owned: T112's
  `cross-lane`); a visit's or meeting's or attack's step paid (E12-4's 1 659, E12-10's 1 437);
  the Fountains' weekly values the same in three runs (the new game's not); the hint over an
  unexplored entrance (none).
- **Inputs:** `experiments/results/E08-batch-12a.md`, `-12b.md` (the third runs) and T110's
  record in `E08-log.md` (the raw observations, the runs' differences); the specs' predictions
  (`experiments/specs/E08-batch-12a.toml`, `-12b`); T101's record as the form.
- **Depends on:** T110 (done). **Status:** ready (docs only; no original data needed).

## Inbox

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

- 2026-11-11 (the town lane, E09 T22b, `town/e09-t22b`; E09 decisions 80 to 83, `TWN-Q14`; **the
  town's rows and the garrison hero in your paths, for T114a, T115a and the order**; the town
  epic's design note `docs/epics/E09-design.md` §52, §53, §56): (1) **the town's two rows open no
  decision**: the game's management check admits, with no decision open, an own town's upper bar
  (`ArmyRef::Garrison(town)`, or `ArmyRef::Hero(the garrison hero)`) alone or with its visitor, by
  your one owner rule; the town lane wires a town's garrison's bar owner (its town's owner) into the
  rule's lookup in its T28a (`cross-lane`), so T114a needs only the hook. (2) **The garrison hero's
  record moves whole** from `AdventureState`'s heroes on the map into the town's `Holds::Town`
  (army, points, formation, tactics, and your T115a's bonuses and timed marks kept): `ArmyRef::Hero`
  resolves to it after the heroes on the map; your day's and week's drops of the marks and the
  turn's points (ADV-MOV-002) must reach it too (the original's loops cover every hero record and
  the towns' +0xC heroes): T28b adds those loops (`cross-lane`); if T115a writes the drops as a
  per-record function, T28b only calls it for the garrison heroes. (3) **The plan in your paths**
  (`cross-lane`, each number the next at merge): T27 (`HireSource::Town`, `::Portal`,
  `HireTarget::Garrison` in `command.rs`; the game state hash; the replay format, `town recruit`),
  T28a (`AdventureCommand::TownHeroes`; the adventure state hash; the replay format; management
  lines naming a town's bar), T26a (the game state hash, map import `roe`); T28 waits for your
  T114a and T115a merged.

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
