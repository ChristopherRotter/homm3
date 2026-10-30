# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T76 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30; increment 8, boats and the sea, by T76's audit). **Increment 9, mines, the week and owned objects**, refined with the user at T76 (ADV-Q36; decisions 192, 193): the survey and the ADR amendment drafts first (T77a, T77b), each task's cut confirmed with the user at its start. T77a in two (decision 194): T77a1 done (`adventure/e08-t77a1`: survey `E08-economy.md` §0 to §6), T77a2 done whole (decision 195; `adventure/e08-t77a2`: §7 to §18, the checkpoint's `ADV-Q37`). T77b done whole (decision 200; `adventure/e08-t77b`: the drafts, `E08-design.md`, the cut in rounds 2 to 8, decision 202; the checkpoint's `ADV-Q38`, decisions 201–204). Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T78 — Batch E9's maps and specs**

- **Scope:** increment 9's design note §4 (the writer) and §17: `h3_map::writer` writes the seven
  classes with their bodies (a mine's owner by subclass, an abandoned mine's resource set, a
  Lighthouse's and a Shipyard's owner, the three weekly objects) and the map's timed events (it
  writes none today), with tests that read them back; the spec's fields in `h3-experiment`
  (battle-owned, `cross-lane` by decisions 198 and 202): mines by subclass with an owner and
  guards, the abandoned mine's set, Lighthouses, Shipyards, the weekly objects, a monster's "does
  not grow", timed events, a hero's Estates, Mysticism and specialty; the specs `E08-batch-9a`
  (E9-1 to E9-5) and `-9b` (E9-6 to E9-12) with their templates and each row's prediction (survey
  §15, plus design §17's two: E9-2 who plays Blue's guards, E9-4 the Lighthouses' surroundings at
  day 1). `docs/experiments/README.md` names the new fields.
- **Inputs:** survey `E08-economy.md` §15 (the plan, the harness's and the spec's gaps); design
  §4, §17; T67's record in `E08-log.md` (increment 8's writer and specs, the same shape).
- **Depends on:** T77b (done; ADV-Q38 answered, decisions 201–204). **Status:** ready; its cut
  confirmed at its start (T67 was taken whole).

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
