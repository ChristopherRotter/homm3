# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T117 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 12 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11, -11-15; increment 12, garrisons and the bonus visitables, by T117's audit). **Increment 13, creature banks**, refined with the user at T117 (ADV-Q52; decisions 333, 334): round 1, the survey and the ADR amendment drafts first (T118a, T118b below). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T118a — the creature banks survey** (decision 334; as T108a for increment 12: survey file
`docs/rules/surveys/E08-banks.md`)

- **Scope:** the census of classes 16 (by subclass), 24, 25, 84 and 85 on the 160 shipped maps
  (T97c's counts in `E08-log.md`, T97c: 2 137 objects on 148 maps); their `.h3m` bodies and
  templates (`docs/formats/h3m.md`, VCMI 1.7.5's reader); the banks' configurations in the
  original data (the text tables the harvest reads: each bank's levels, their chances, guards and
  rewards); the original's code (`docs/exe/README.md`, `scripts/exe.py`): the draw of a bank's
  level, guards and reward at a new game (and at a week's or a month's start, if any), the visit
  (the question, the battle against the guards, the reward: resources, artifacts, creatures
  through the army dialog 0x4ABDC0), the bank emptied, a revisit and a lost battle; the bank
  battle (the battle's bank flag +0x53C5: no war machines, no tactics phase, the guards' own
  layout; the battle lane's: its queue's "War machines" note); what a viewer sees (a bank's
  visited state, its guards' size word); VCMI's and the wiki's rules; disputes `D-<n>` from 65,
  reading quirk candidates `A-<n>` from 46; the proposed rule ids; the batch's plan (E13) on the
  client lane's harness (decision 334's agent's call); what the bank battle needs, told to the
  battle lane's inbox. Likely in parts (the content and the draw; the visit, the battle and the
  reward), confirmed with the user at its start, the reading measured first.
- **Inputs:** decisions 333, 334; ADR-0015 to -0017 (ADR-0016 points 3, 4 and 6, ADR-0017's
  decisions and rests), ADR-0013 (the hero's derived properties); increment 3's monsters and
  battles on the map (the neutral seat), increment 7's rewards (the shared reward record, the
  artifacts), increment 10's hire dialog (creatures joining an army); `E08-archive.md` decision
  127 (T55a3's measure: the five banks about 3 000 instructions and the battle's bank flag, which
  `BattleSetup` lacks); the battle lane's queue ("War machines"); the lane's `TRAPS.md`.
- **Depends on:** T117 (done). **Status:** ready (the original data and the shipped maps:
  staging per `docs/08-work-process.md`, "Cloud sandbox").

**E08 T118b — the ADR amendment drafts, the design note and the cut** (decision 334)

- **Scope:** as T108b for increment 12: the drafts `adr-drafts/0016-increment-13.md` and
  `0017-increment-13.md` (and ADR-0013's or ADR-0014's if the survey finds a change of the hero's
  derived properties or of the battle's setup), increment 13's design note `E08-design.md` (its
  parts by side), the building tasks' cut and the exit criteria; the checkpoint with the user (the
  note, the drafts, the agent's calls, the cut).
- **Inputs:** the survey `docs/rules/surveys/E08-banks.md` (the proposed ids, the interfaces'
  sketch, the open points); decisions 333, 334 and T118a's checkpoint's; ADR-0015 to -0017; the
  battle lane's answer on the bank battle (its inbox); T108b's drafts as the form.
- **Depends on:** T118a. **Status:** after T118a (docs only).

## Inbox

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
