# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T51b (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 5 are closed (2026-10-12, -13, -20, -21, -23). **Increment 6, army and hero management** (decisions 90 to 117; survey `docs/rules/surveys/E08-army.md`, design `E08-design.md`): rounds 1 to 3 done (T44a to T49b); round 4 done (T50, T51a, T51b: the meeting with Scholar, the army dialog, the commands between two armies; game state hash v13; `ADV-STUB-2` closed); round 5: T52 done whole (decision 119: the plans and properties manage, the army map and two scripted golden replays, two new gated benchmarks; `E08-design.md` §12), **T53 next** (the audit against the exit criteria, increment 6 closed, increment 7 refined; §15), its cut confirmed with the user at its start. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

## Inbox

- 2026-10-24 (the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; **for you**, when battles start from the adventure
  map): the seam carries three facts for viewers, read by no rule, each defaulting to "not
  known": `BattleSetup.background` (the battlefield picture's variant on dirt, grass and snow:
  0x41ADB0 counts the dead vegetation, mountains and trees within 3 tiles of the battle's tile, 0
  where dead vegetation is the most, 1 where mountains are, 2 otherwise, an empty surrounding
  included; `docs/epics/E17-design.md` §5), `BattleSetup.colours` (each side's player colour by
  `BattleSide::index`, 0 red to 7 pink; `None` for a side without a player, a monster's or a
  neutral town's; one colour for both sides is `SetupError::InvalidColour`) and `BattleHero.sex`
  (`HeroSex`, where the map gave a placed hero one; `None` the hero type's own). `h3-game` should
  set them when it starts a battle. The battle keeps them as hashed state (battle state hash v54)
  and observes them; your ten golden replays were re-recorded with their options, their commands
  unchanged (only checkpoint hashes taken inside a battle moved).

- 2026-10-23 (the battle lane, E23 T7, `shared/e23-t7-human` #436 and `battle/e23-t7-rest`, both
  `cross-lane` with the user's consent; **for you**): **a battle rule reads `BattleSetup.human`
  now**: with the new fix-optional BAT-QUIRK-27 (`quirk.battle.computer_morale_let_off`, registry
  position 33, on in `faithful`) a frozen stack of a side whose player is not human escapes one
  bad-morale freeze in four (BAT-MOR-004, 0x464BDB; ADR-0023 point 7 widened). `h3-game`'s battle
  setup (`state/battle.rs`) leaves `human` and `difficulty` at their defaults, so in a game's battle
  every side, a human player's too, gets the let-off: set `human` from the side's player (true for
  a human player's hero, false for a computer player's and a neutral side) and `difficulty` from
  the game. In your paths: the base faithful core ruleset hash moved (`0x1ac6c50d158057de` →
  `0x0c7e01b4dffeba0c`), your eight faithful replays' headers patched (`ruleset_hash` only; all ten
  verify). Also BAT-WAR-003: the automatic tent heals the most hurt stack (no draw).

- 2026-10-23 (the battle lane, E23 T6's last part, `battle/e23-t6`, `cross-lane` with the user's
  consent; **for you**, when battles start from the adventure map): the battle now keeps
  `BattleSetup.retreat` and `attacker_ground` (ADR-0027 point 14) as hashed state, and
  `h3_battle_ai::choose` reads them: a hero's side retreats as the original's computer does
  (BAT-AI-024; `POLICY_VERSION` 18) when the facts allow it, and the odds value an attacking hero's
  spells at its own ground's mastery in `faithful`. The game orchestrator (`h3-game`) should derive
  them: per side the hero's experience, its artifacts' worth by the original's count (per worn or
  carried artifact the larger of the adventure AI's value, 0x433AA0, and half its gold cost), a
  Tavern town of its player to hire it again from (another than the battle's town for its
  defender), whether the scenario's victory condition wants that hero defeated; and the special
  battlefield of the attacking hero's own tile. The defaults never retreat. Your ten golden
  replays were re-recorded again with their options, their commands unchanged (only hash lines;
  `duel-s2-d7` none): the hash covers the new fields.

- 2026-10-23 (the map-client lane, E24 M1 T1a, `map-client/e24-t1a`; answers your T8b1 note,
  2026-10-11): **which hidden tiles the original draws with an edge piece**
  (`docs/epics/E24-design.md` §1.5, a code reading of 0x412220 and 0x40F8C0). Every tile the
  viewer has not explored and that has an explored tile among its eight neighbours gets a piece of
  `tshre.def` chosen by which of the eight are unexplored (neighbours off the map count as
  unexplored); every other unexplored tile gets the full `tshrc.def`; explored tiles nothing. So
  the half-dark border is exactly ADR-0016's fringe (the eight tiles around an explored one), as
  your T8b1 took it; VCMI agrees on which tiles. The view tests only the viewing player's own bit
  of the tile's vision word (0x69CCBC = 1 << the viewing colour), no team bits. Nothing to change.

- 2026-10-23 (T42, measured; for a performance task): **a flyer's or water walker's reach costs
  about 1.9 times a walker's query** on the XL map (`reach-lifted-xl-144`: 279 k Ir a query
  against `reach-xl-144`'s 148 k on the same games), and random play on the duel with one of each
  costs 56 % more a step (`random-play-duel-36`): `Reach::search_layered`'s second layer is the
  lever (`crates/h3-adventure/src/reach.rs`).

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for the neutral seat**, E23 exit criterion 5): the original's battle AI for a side without a hero is complete: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (shots, melee, moves, waits, defends, the creature casts), pure, deterministic, its draws from the stream given, its choice validated (Defend if refused). The neutral seat (decision 27, your random stand-in) can call it for a monster battle's unowned side; `POLICY_VERSION` 9. A side with a hero (a computer player's) casts nothing and never retreats until E23 T5 and T6 (BAT-STUB-5).

- 2026-10-22 (ADV-Q20, decision 75; restated at ADV-Q21, decision 78, in ADR-0016's draft point 4; for the increment that revisits flying): **A-13's trade-off.** Flying stays on explored tiles (A-13 fixed for now, provisional), so wide obstacle ranges with unexplored tiles beyond them stop a flyer; the user: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway". Decide the compromise, what it tells the player of the hidden tiles, and `ADV-QUIRK-15`'s class (built by T41: the unexplored tile's row of `AdventureState::passage`, `crates/h3-adventure/src/state/passage.rs`). ~~**A-14's gap**~~ (closed by T38: the tile bit +0xC 0x40 means "no object blocks the tile", not the coast, so `hero_flight_tile_test` passes on every free land tile and a flyer visits as a walker). ~~**D-23**: Cursed Ground's movement effect is built only if batch E5's row E5-6 confirms it.~~ (E5-6 confirmed it for both grounds, T38: built by T40.)

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T23, decision 48): what moved from increment 4 to the sketch's increment 5 is still called increment 4 where this docs-only change did not reach: ~~the code comment in `crates/h3-adventure/src/reach.rs` (movement skills)~~ (T33: increment 5); ADR-0016 (army management, increment 6 since decision 90: T44b's draft), with the amendment that builds army management (T24b2 needed none for ADR-0016; ADR-0013's movement and scouting line is in T24b1's draft, point 2). The surveys keep their *(inc. 4)* marks (input, not rules).

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

- 2026-10-19 (the client lane, E17 H1 and H2; your request of 2026-10-13): **the harness answers questions and plays battles from the map** (`clients/capture/README.md`, "An experiment batch"; design `docs/epics/E17-design.md` §6.3). A recipe now has `difficulty` (checked in memory), a step's `answer` (`yes`/`no`, or per kind: `join`, `join for gold`, `flee`, `other`: the question named by the record its text came from, ADVEVENT 86, 88 or 89 + 90, 91) and `battle` (`auto`, `retreat`, `surrender`, `defender retreat`, `defender surrender`), and records the questions asked, the battle's stacks at the start and the end (and hexes: the formation), its outcome, any hero's experience, level, primary and secondary skills, army and whether it is on the map, every player's resources before and after, and a tile's monster (count, aggression, never flees). Checked on Easy with what h3-experiment places today and, for the offers, on a local map with patched dispositions (join, flee, join then flee): a monster's offers in a spec need T22a's dispositions and "never flees". Two things for the spec: hot seat needs at least two human players, and a player left without heroes and towns is out (with one human left the game ends): keep each player a hero after its battles. H2 the same day: `walk = [x, y, level]` (the underground, E3-4; shown on a shipped map, since `h3-map`'s writer has no underground yet), more than five heroes per player (the list scrolled), `level_up` (`left` or `right`: a level-up's offered skill), the observations `heroes:experience` (the heroes off the map: E3-7) and `date`, and any number of days. Seen on the way (scored with batch E3 at T22b): a map hero with 5 000 experience and 2/3/1/1, basic Diplomacy and advanced Logistics started at level 5 in two runs, its level-ups drawn (3/5/1/2 with both expert, then 3/4/2/2 with Diplomacy advanced) (E3-6); a surrender in hot seat moved its 470 gold to the other player (E3-8); the 147 heroes off the map held 40 to 90 in two runs, 1 and 4 of them at 90 (E3-7).

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
