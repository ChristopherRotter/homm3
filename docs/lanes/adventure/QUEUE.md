# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T76 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30; increment 8, boats and the sea, by T76's audit). **Increment 9, mines, the week and owned objects**, refined with the user at T76 (ADV-Q36; decisions 192, 193): the survey and the ADR amendment drafts first (T77a, T77b), each task's cut confirmed with the user at its start. T77a in two (decision 194): T77a1 done (`adventure/e08-t77a1`: survey `E08-economy.md` §0 to §6). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T77a — The mines, week and owned objects survey** (in two, decision 194: T77a1 points 1 to 4 and the census done, survey §0 to §6; **T77a2** point 5, the mines' income of point 2, points 6 and 7 next, §7 on)

- **Scope** (decisions 192, 193): a survey `docs/rules/surveys/E08-economy.md` (as T17a, T24a,
  T34a, T44a, T55a and T66a): the original's rules by code reading (`docs/exe/README.md`; read so
  far: `visit_mine` 0x4A39A0 and the owned objects' code by T77a1 (the survey's Sources), `tile_object_owner` 0x40FD20 with the owned objects'
  record tables, `new_day` 0x4C7FE0 and `new_day_week_events` 0x4C8780 in part,
  `covered_view_new_week` 0x4CA840; `town_gold_income` 0x5BF600 is a town's, E09) with VCMI 1.7.5
  and the wiki, and a census of the 160 shipped maps (T76's counts in `E08-log.md`, T76). (1)
  **Flagging:** which classes a visit flags, what it costs or asks, a guarded owned object, a hero
  standing on an owned object's entrance, an owned object of a player out of the game, the owner
  at load (the map's owner fields), what another player sees (ADR-0016 point 6). (2) **Mines** (53
  subclasses 0 to 6; the abandoned mine 53.7 and 220): their guards, the Leave Guards dialog, the
  abandoned mine's guards and its resource; **the day's income** and its order at the day's
  start. (3) **Lighthouses** (42): the bonus to the owner's heroes at sea, its stacking, when it is
  read (the turn start's sea points, HER-ADV-009). (4) **Shipyards** (87): where the boat appears,
  its price, when it is refused (a boat or a hero on the tile), what the boat's record gets
  (ADV-SEA-001). (5) **The week:** its order (the day, the week, the month), the Windmill (112), the
  Water Wheel (109) and the Mystical Garden (55) (what each gives, who, how often, the reset), the
  monsters' weekly growth (the "does not grow" flag), the map's timed events (decision 127's
  agent's call). (6) The commands, decisions, events, the observation, replays and benchmarks it
  changes; **batch E9's plan** on the client lane's harness (decision 193's agent's call). (7) The
  proposed rule ids, quirk candidates and disputes; a fresh agent's second reading of the deciding
  code; the checkpoint's questions (an `ADV-Q`).
- **Not in it** (decision 192): dwellings and random dwellings, the Refugee Camp and the hire
  dialog, garrisons (item 10); the teleporters; towns' income and the week's creature (E09); a
  computer player's flags (E10); simultaneous play.
- **T77a2** (decision 194: what the day's start does): the day's, week's and month's order
  (`new_day` 0x4C7FE0, `new_day_week_events` 0x4C8780, read in part), the mines' income
  (`mine_daily_income` 0x678288, survey §3.5) and its order, point 5 whole, then points 6 and
  7: the survey's §7 on, with T77a1's inputs (§6: D-47, D-48, A-29 to A-31, the open points,
  among them the Shipyard's use without a visit, §5.3, for the second reading) and the
  checkpoint's `ADV-Q37`.
- **Depends on:** T76 (done). **Status:** T77a2 ready, its cut confirmed at its start; a user
  checkpoint at its end (decision 193).

**E08 T77b — The ADR amendment drafts, the design note and the cut**

- **Scope:** from T77a's survey and the answers at its checkpoint: the amendment drafts of the
  ADRs it touches (ADR-0016 for owned objects, the day's and the week's changes on the map;
  ADR-0017 for the day's and the week's order in the game and the income into the treasury;
  ADR-0013 if `h3-heroes` gains rules, such as the Lighthouse's sea points), `E08-design.md`
  written for increment 9 (with its exit criteria), the building tasks cut in rounds; the
  checkpoint's questions (an `ADV-Q`). The user accepts the drafts and the cut before anything is
  built (decision 193).
- **Depends on:** T77a and its checkpoint. **Status:** waiting; its cut confirmed at its start.

## Inbox

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

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
