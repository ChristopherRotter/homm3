# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T51b (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 6 are closed (2026-10-12, -13, -20, -21, -23, -26; increment 6, army and hero management, by T53's audit). **Increment 7, rewards on the map**, refined with the user at T53 (ADV-Q26; decisions 121 to 123): T54 (the battle seam's facts) done; the survey and the ADR amendment drafts (T55a, T55b) done, each task's cut confirmed with the user at its start. T55a is cut in four (decision 125): all four done (survey `docs/rules/surveys/E08-rewards.md`; decisions 126 to 128 the cuts, its checkpoint ADV-Q28 decisions 129 to 136); T55b done (decision 137; its checkpoint ADV-Q29 decisions 138 to 141: the drafts accepted with typed answers, eight rounds with batch E7 first, design §19). T56 done (decision 142: whole, batch E7's maps and specs). Round 2: T57 done (decision 143: whole, the harness's reads, the recipes and the three runs), T58 done (decision 144: scored, A-23 withdrawn). T58b done (decision 144: `h3-map`'s view test's oracle; ADV-Q30, decision 145: A-25 a quirk with a flag). Round 3: T59 done (decision 146: whole, `shared/e08-t59`). Round 4: T60 in two (decision 147): T60a done (`adventure/e08-t60a`, the bonuses and the four changes), T60b done (`adventure/e08-t60b`, `grant` and the teachers). Round 5: T61 in two (decision 148): T61a done (`adventure/e08-t61a`, the import and the kinds), T61b done (`adventure/e08-t61b`, decision 149: whole, the bans, the draws and the state). Round 6: T62 in two (decision 150): T62a done (`adventure/e08-t62a`, the pickups, teachers and finds), T62b in two (decision 151: the rest stays in T63): T62b1 done (`adventure/e08-t62b1`, the record, the box and the Pyramid), T62b2 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T62b2 — `h3-adventure`: events and the move (design §12, §13's events)

- **Scope** (decision 151; design §12, §13; ADR-0016 points 1, 6; ADR-0017 point 10): an event's
  tile apart from the movement's masks and the step that fires it (ADV-OBJ-007, ADV-MOV-011: the
  move's stop with the points left, the rest of the path dropped, the guards at once as a guarded
  pile's, the record of T62b1, the removal flag; a hero that starts on it does not fire it); the
  guard of the tile first (survey §8.5): until T63's rest an event on a guarded tile does not fire
  (`ADV-STUB-10`); A-26 fixed as `ADV-QUIRK-29` (no flag: the event fires only for a hero still on
  its tile after the guard's visit); what a player sees and is told (ADV-FOG-007: never observed,
  its firing told to its visitor's owner alone; the hidden-event property: two games that differ
  by a hidden event give every player the same observation and legal moves until it fires).
  For the next `shared/` change (not a lane path): `docs/06-security.md`'s "Its own" gains the
  viewer's knowledge of visited objects and its heroes' marks (ADV-VIS-006, T62a).
- **Cut:** confirmed with the user at its start.
- **Depends on:** T62b1 (`adventure/e08-t62b1`). **Status:** ready once T62b1 merges.

## Inbox

- 2026-10-23 (T42, measured; for a performance task): **a flyer's or water walker's reach costs
  about 1.9 times a walker's query** on the XL map (`reach-lifted-xl-144`: 279 k Ir a query
  against `reach-xl-144`'s 148 k on the same games), and random play on the duel with one of each
  costs 56 % more a step (`random-play-duel-36`): `Reach::search_layered`'s second layer is the
  lever (`crates/h3-adventure/src/reach.rs`).

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for the neutral seat**, E23 exit criterion 5): the original's battle AI for a side without a hero is complete: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (shots, melee, moves, waits, defends, the creature casts), pure, deterministic, its draws from the stream given, its choice validated (Defend if refused). The neutral seat (decision 27, your random stand-in) can call it for a monster battle's unowned side; `POLICY_VERSION` 9. A side with a hero (a computer player's) casts nothing and never retreats until E23 T5 and T6 (BAT-STUB-5).

- 2026-10-22 (ADV-Q20, decision 75; restated at ADV-Q21, decision 78, in ADR-0016's draft point 4; for the increment that revisits flying): **A-13's trade-off.** Flying stays on explored tiles (A-13 fixed for now, provisional), so wide obstacle ranges with unexplored tiles beyond them stop a flyer; the user: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway". Decide the compromise, what it tells the player of the hidden tiles, and `ADV-QUIRK-15`'s class (built by T41: the unexplored tile's row of `AdventureState::passage`, `crates/h3-adventure/src/state/passage.rs`). ~~**A-14's gap**~~ (closed by T38: the tile bit +0xC 0x40 means "no object blocks the tile", not the coast, so `hero_flight_tile_test` passes on every free land tile and a flyer visits as a walker). ~~**D-23**: Cursed Ground's movement effect is built only if batch E5's row E5-6 confirms it.~~ (E5-6 confirmed it for both grounds, T38: built by T40.)

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
