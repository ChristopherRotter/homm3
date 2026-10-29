# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T65 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 7 are closed (2026-10-12, -13, -20, -21, -23, -26, -28; increment 7, rewards on the map, by T65's audit). **Increment 8, boats and the sea**, refined with the user at T65 (ADV-Q31; decisions 158, 159): the survey and the ADR amendment drafts first (T66a, T66b), each task's cut confirmed with the user at its start. T66a in two (decision 160): T66a1 done (`adventure/e08-t66a1`, #550: survey `E08-sea.md` §0 to §6), T66a2 done (`adventure/e08-t66a2`, decision 161: whole; §7 to §14, the checkpoint's ADV-Q32); T66b done (`adventure/e08-t66b`, decision 169: whole; the drafts, `E08-design.md` and the cut in eight rounds, ADV-Q33 accepted: decisions 170–173). Round 2: T67 done (`adventure/e08-t67`, decision 174: whole; the writer's sea objects, the spec's bottles and a monster's own tile, `cross-lane`; the specs E8a, its water map and E8b with their templates). Round 3: T68 in two by batch (decision 175): T68a done (`adventure/e08-t68a`: the harness's boat reads and steps, E8a and its water map run and recorded), T68b done (`adventure/e08-t68b`, decision 176: whole; the sea pickups' reads, E8b run four times, two recorded); T69 done (`adventure/e08-t69`, decision 177: whole with a re-read; batch E8 scored, D-45 reverted, D-46 to batch E8c: decision 179); T69b done (`adventure/e08-t69b`, decision 180: whole; batch E8c run and scored, D-46 confirmed, survey §16). Round 4: T70 done (`shared/e08-t70`, `cross-lane`, decision 181: whole; the three ADRs applied, the effect kinds, the packs and the glossary). Round 5: T71 done (`adventure/e08-t71`, decision 182: with the inbox's property case; HER-ADV-009), T72 done (`adventure/e08-t72`, `cross-lane`, decision 183: whole; the import, kinds, state and views; the AI lane's bots tables' fingerprint). T73 next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T73 — `h3-adventure`: moving at sea, the prices and the conversion, `Reach`, visits at sea

- **Scope** (decision 171; design §7 to §10, round 6): `Mode::Sail` and the passage's boat rows
  (boarding from a stop node, no guard test there; landing onto a free coastal tile only, D-42;
  the diagonal between two waters; never onto an empty boat; D-46: no flight over a water tile an
  object other than a boat, a boat's hero or class 85 is visited from, ADV-MOV-009 amended); the
  sea's prices (100, 141; Favorable Winds' `sea_ground` share from `StaticTile::winds`), the drop
  and the diagonal exception at sea (D-43), boarding's and landing's conversion with the
  Admiral's Hat (`h3_heroes::free_boarding`, D-41: after the step's drop; `ADV-QUIRK-31`, A-28 not
  reproduced) switching `HeroOnMap::max_movement`; a landing's facing (the original's order,
  tested against E8-2's and E8-15's records), `Boarded` and `Landed` events; `Reach` for a hero in
  a boat and the boat rows (ADV-MOV-007 amended); visits from a boat (ADV-SEA-005: the built kinds
  from the neighbouring water tile, a boat's hero met or attacked, a hero on land never reached),
  events on water (ADV-OBJ-007, ADV-MOV-011 amended). Rule ids: ADV-SEA-002 to -005 (survey
  `E08-sea.md` §12). T72 left `AdventureState::put_aboard` (tests only) as the boarding's state
  change and `StaticTile::hole` for the landing.
- **Depends on:** T72 (`adventure/e08-t72`). **Status:** ready once T72's pull request merged; big
  (design §18: likely in two, the boat's moves and prices, then `Reach`), its seam confirmed at its
  start.

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
