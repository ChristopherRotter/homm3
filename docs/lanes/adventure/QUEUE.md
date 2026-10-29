# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T65 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 7 are closed (2026-10-12, -13, -20, -21, -23, -26, -28; increment 7, rewards on the map, by T65's audit). **Increment 8, boats and the sea**, refined with the user at T65 (ADV-Q31; decisions 158, 159): the survey and the ADR amendment drafts first (T66a, T66b), each task's cut confirmed with the user at its start. T66a in two (decision 160): T66a1 done (`adventure/e08-t66a1`, #550: survey `E08-sea.md` §0 to §6), T66a2 done (`adventure/e08-t66a2`, decision 161: whole; §7 to §14, the checkpoint's ADV-Q32); T66b done (`adventure/e08-t66b`, decision 169: whole; the drafts, `E08-design.md` and the cut in eight rounds, ADV-Q33 accepted: decisions 170–173). Round 2: T67 done (`adventure/e08-t67`, decision 174: whole; the writer's sea objects, the spec's bottles and a monster's own tile, `cross-lane`; the specs E8a, its water map and E8b with their templates). Round 3: T68 in two by batch (decision 175): T68a done (`adventure/e08-t68a`: the harness's boat reads and steps, E8a and its water map run and recorded), T68b done (`adventure/e08-t68b`, decision 176: whole; the sea pickups' reads, E8b run four times, two recorded); T69 done (`adventure/e08-t69`, decision 177: whole with a re-read; batch E8 scored, D-45 reverted, D-46 to batch E8c: decision 179); T69b done (`adventure/e08-t69b`, decision 180: whole; batch E8c run and scored, D-46 confirmed, survey §16). Round 4: T70 done (`shared/e08-t70`, `cross-lane`, decision 181: whole; the three ADRs applied, the effect kinds, the packs and the glossary). T71 next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T71 — `h3-heroes`: the day's sea points and free boarding (HER-ADV-009)

- **Scope** (decision 171; design §2, §3, round 5): `h3_heroes::sea_points(details, core, table:
  &[u32; 4]) -> u32` (the table's entry at the highest `navigation` level, a Navigation
  specialist's ⌊level × table[0] × `bp_per_level` ÷ 10 000⌋ with the skill, each worn type's
  `sea_movement_points` once, saturating) and `free_boarding(details, core) -> bool` (a worn type
  with `free_boarding`, not its parts worn apart); HER-ADV-009 in the rule catalog (sources: the
  gap-fill's `exe-sea-movement`, `exe-free-boarding`, `vcmi-sea-movement`, `wiki-sea-movement`;
  survey `E08-sea.md` §3.1, §4.2); design §2's tests; 100 % coverage. No version (no
  `HeroDetails` field). T72 (the import, the kinds and the state) is the round's second task.
- **Depends on:** T70 (`shared/e08-t70`, merged first). **Status:** ready; its cut confirmed at
  its start. T71 is small: the inbox's failing property case (a fix wanted before T72) may join it.

## Inbox

- 2026-10-28 (T68a's scoped verify; **a failing property case**, for a fix task before T72):
  `h3-map`'s `every_step_keeps_the_commands_invariants` failed on a random case, reproducible
  with the regression line `cc a85563f06cafc2ccfda4e0e43ef5db0c90238e7cf03330cf9f12b82390be04fc`
  (not committed: it would fail CI until fixed; add it to
  `crates/h3-map/proptest-regressions/tests/play.txt` with the fix). At `play.rs` 2301 the actor
  has an army dialog open (a join without room) and its `Close` is accepted, but a **battle**
  is then open for the actor (`probe.battle(actor)`), where the plan asserts no decision. Likely a
  second monster guarding the joined one's tile (GAM-BAT-001's guards in turn): decide whether
  the battle after the join is the original's sequence (then the plan's assertion is too
  strict) or an engine bug. `master` 7dad572's CI passed: the case is rare.

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
