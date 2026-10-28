# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T65 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 7 are closed (2026-10-12, -13, -20, -21, -23, -26, -28; increment 7, rewards on the map, by T65's audit). **Increment 8, boats and the sea**, refined with the user at T65 (ADV-Q31; decisions 158, 159): the survey and the ADR amendment drafts first (T66a, T66b), each task's cut confirmed with the user at its start. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T66a — The boats and sea survey (in two, decision 160: T66a1 points 1, 2, 3, 6 — **in progress**, `adventure/e08-t66a1`; T66a2 points 4, 5, 7, 8)

- **Scope** (decisions 158, 159): a survey `docs/rules/surveys/E08-sea.md` (as T17a, T24a, T34a,
  T44a and T55a): the original's rules for boats and the sea, by code reading
  (`docs/exe/README.md`; what is read already: `hero_boards_boat` 0x4A1010, `hero_lands_from_boat`
  0x49E670 and the boat's points in survey `E08-hero-movement.md` §1.1, §1.4, §2.1, §3.1 and §6,
  the boats' drawing in `docs/epics/E24-design.md` §2.3) with VCMI 1.7.5 and the wiki, and a census
  of the 160 shipped maps. (1) **Boats** (class 8): how many, their subclasses, heroes placed in
  one, what a map leaves reachable only by sea; the body and the import. (2) **Boarding and
  landing:** which steps board and land, the points they cost and leave (the Admiral's Hat), what
  boarding clears (Fly, Water Walk), where the boat stays, a guarded or blocked shore. (3)
  **Sailing:** the step's price at sea and the day's points (Navigation, the Necklace of Ocean
  Guidance, the Sea Captain's Hat; a Lighthouse's bonus stays with item 9), Favorable Winds (225),
  what a hero in a boat visits and how (the sea's objects, another hero, the events on water of
  the inbox's note below), scouting at sea, what another player sees of a boat. (4) **A battle
  with a hero in a boat** (against a hero; the ship's deck and the two boats battlefields,
  GAM-BAT-001; whether anything else fights a hero at sea). (5) **The sea's pickups:** Sea Chest
  (82), Shipwreck Survivor (86), Flotsam (29), Ocean Bottle (59): their contents, draws at a new
  game and visits. (6) **The writer:** an empty boat and a hero standing in one (CLT-Q5). (7) The
  commands, decisions, events, the observation, replays and benchmarks it changes; **batch E8's
  plan** on the client lane's harness (decision 159's agent's call). (8) The proposed rule ids,
  quirk candidates and disputes; a fresh agent's second reading of the code; the checkpoint's
  questions (an `ADV-Q`).
- **Not in it** (decision 158): Shipyards (87) and Lighthouses (42), item 9; Whirlpools, Buoys,
  Mermaids, Sirens; the Derelict Ship and the Shipwreck (item 10); Summon Boat and Scuttle Boat
  (the adventure spells); a town's shipyard (E09).
- **Depends on:** T65 (done). **Status:** T66a1 in progress (the census, boats, boarding, landing,
  sailing, the events on water, the writer); T66a2 after it (the battle at sea, the four pickups,
  the commands and the observation, batch E8's plan, the rule ids, the second reading, the
  checkpoint's `ADV-Q`); a user checkpoint at T66a2's end (decision 159).

### E08 T66b — The ADR amendment drafts, the design note and the cut

- **Scope:** from T66a's survey and the answers at its checkpoint: the amendment drafts of the
  ADRs it touches (ADR-0016 for boats, the sea and a visit from a boat,
  `docs/lanes/adventure/adr-drafts/`; ADR-0017 for the commands, decisions and battles; ADR-0013
  if `h3-heroes` gains rules, such as the day's points at sea), `E08-design.md` rewritten for
  increment 8 (the hero side, the map, the game, the batch and the cut, with increment 8's exit
  criteria), the building tasks cut in rounds; the checkpoint's questions (an `ADV-Q`). The user
  accepts the drafts and the cut before anything is built (decision 159).
- **Depends on:** T66a and its checkpoint. **Status:** waiting for T66a.

## Inbox

- 2026-10-27 (E08 T62b2's census; **for T66a**, increment 8 builds boats, decision 158): 9 events of
  the shipped maps lie on water. The engine fires an event only for a hero that stands on its tile,
  so these fire once a hero in a boat can step onto them (the original's `hero_move_step` tests
  every tile a step enters, at sea too). Check the boat's step against
  `AdventureState::firing_event` then.

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
