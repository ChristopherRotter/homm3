# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T76 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30; increment 8, boats and the sea, by T76's audit). **Increment 9, mines, the week and owned objects**, refined with the user at T76 (ADV-Q36; decisions 192, 193): the survey and the ADR amendment drafts first (T77a, T77b), each task's cut confirmed with the user at its start. T77a in two (decision 194): T77a1 done (`adventure/e08-t77a1`: survey `E08-economy.md` §0 to §6), T77a2 done whole (decision 195; `adventure/e08-t77a2`: §7 to §18, the checkpoint's `ADV-Q37`). T77b done whole (decision 200; `adventure/e08-t77b`: the drafts, `E08-design.md`, the cut in rounds 2 to 8, decision 202; the checkpoint's `ADV-Q38`, decisions 201–204). T78 done whole (decision 205; `adventure/e08-t78`, `cross-lane`, decision 206: the writer's ownable objects, weekly objects and timed events, the spec's fields, the specs `E08-batch-9a` and `-9b`). T79 in two by batch (decision 207): T79a
done (`adventure/e08-t79a`: the harness's mine records, tile words, boat dialog, Leave Guards,
battle sides and `auto again`; the recipe `E08-batch-9a` and its runs); T79b done whole (decision
208; `adventure/e08-t79b`: the harness's income, Garden mask, weekly objects' and monsters' words,
end-turn messages; the recipe `E08-batch-9b` and its runs). T80 done (decisions 209, 210: whole
with A-33's re-read; `adventure/e08-t80`, `cross-lane`: batch E9 scored, survey §19, A-33
withdrawn). T81 in two, the ADRs first (decision 211): T81a done (`shared/e08-t81a`: the three
drafts applied, ADR-0015's core row, the index, the glossary); T81b done (decision 212: whole;
`shared/e08-t81b`, `cross-lane`: the four kinds, the two parameters, the creature's abilities
outside battle, the packs; core ruleset hash v51, pack format v50). Round 5: T82 and T83a done
together (decision 213; `adventure/e08-t82-t83a`, `cross-lane`, decision 214: the day's income,
spell points and the Lighthouses' input; the seven kinds and parameters, the import), T83b in two by what reads it (decision 215): T83b1 done
(`adventure/e08-t83b1`, `cross-lane`, decision 217: the owners, guards, weekly contents, draws,
the start's reveal, the flag radius), T83b2 in two, the state first (decision 218): T83b2a done
(`adventure/e08-t83b2a`: the fractions, the added objects, `check_added`, adventure state hash
v19), T83b2b next.
Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T83b2b — `h3-adventure`: the views**

- **Scope:** design §12 (decision 218: T83b2's second half): `ObjectView`'s `owner` (every viewer
  who sees the object), `production` (a mine its viewer's team owns: an abandoned mine's drawn
  resource hidden from the others), `guards` (a guarded pile's view: creatures and their words,
  the counts for the owner's team), `visited` (a weekly object's mark: the viewer's team's, the
  Garden's its own), a Shipyard's `boat_tile` (static, public); the added monsters among the
  objects as T83b2a built them, a fraction never shown; ADV-OWN-003 and ADV-FOG-006 amended; the
  h3-map view tests (`crates/h3-map/src/tests/view.rs`); `docs/06-security.md`'s owners,
  production and marks (shared: its own `shared/` pull request, merged first, or the user's
  consent for `cross-lane`).
- **Inputs:** design §12; ADR-0016 points 5, 6; survey `E08-economy.md` §2.7, §9.4; T83b1's
  record in `E08-log.md` (the owners, `mine_guards`, `Weekly`, `Resolved`'s draws); T83b2a's
  (the added objects in the observation); T72's record (the boats' view, the pattern).
- **Depends on:** T83b2a (`adventure/e08-t83b2a` merged). **Status:** ready after that merge;
  its cut confirmed with the user at its start.

## Inbox

- 2026-10-31 (the client lane, E17 M5 T8b2, review R7-8; for information, nothing to do): the
  harness checks the starting bonus Gold it sets. `h3drive.gold_bonus` (a recipe's `gold = true`,
  `h3adv.to_hot_seat(..., gold=True)`) reads the scenario list's rows in memory
  (`h3mem.starting_bonuses`: H3API's `H3SelectScenarioDialog` pointer 0x69FBE8, `bonusType` +0x6C,
  3 Random, 0 Artifact, 1 Gold; observed) and stops the run unless exactly its players read Gold.
  Recipes without `gold` are unchanged.

- 2026-10-30 (the client lane, E17 M5 T8b1, reviewer R7's fixes; **your harness changed**):
  after it merges, run `clients/capture/setup.sh` again for every run directory you use: the
  hook DLL's ack has ten fields now, and an older `xdd.dll` stops a run with "stale xdd.dll".
  `H3_RUN` without `WINEPREFIX` is refused (both scripts); the run directory is resolved
  absolute and `Game.start` fails unless the prefix's `d:` points at it. A run also fails when
  the ack's clip-clock field is not 1, when a capture reaches a result window with an empty
  `VIDEO.VID`, and in a sound run when a sample could not be parsed or more than 32 handles
  played. Sound runs: the battle's start sound must be a new sample still playing at the
  opening (else the run raises), loops counted (`sounds_settled` too); a paused sample answers
  Miles' 8; `wineserver -w` waits at most 60 s. Nothing else drives the original otherwise
  (runbook, "Sound", "The clips", "Traps").

- 2026-10-30 (the map client lane, E24 M3 T6a2b; the user's standing consent for the writer, E24
  T0; for your information, nothing to do): **the writer writes quests**: `Extras::quests` maps
  an index of `Extras::objects` (a Seer's Hut, 83, or a Quest Guard, 215) to a
  `model::Mission`, written in the SoD layout with no deadline, no texts and a seer's reward
  nothing (`writer::extras::write_quest`; refused on another class or with a list past 255).
  A literal `Extras { tiles, objects }` now needs `..Extras::default()`.

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
