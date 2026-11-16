# Town lane — QUEUE archive

Inbox notes that left the queue, verbatim, with where their work went (`docs/08-work-process.md`,
"Keeping the start-of-session reading small"). Read only when a doc points here.

## 2026-11-15 — increment 3's inbox notes, acted on or answered: archived at E09 T30

Each note verbatim, after the line saying where its work went (E09 decision 102).

*Answered by the user at E09 T30 (`TWN-Q15` (4), decision 101: projected below level 3), queued as T31:*

- 2026-11-14 (the server lane, E16 M5 T24, its review of your T28a's wire; **a question for
  you, low**): `GarrisonEntered { hero, town, pos }` goes to every viewer of the town's entrance
  (`town/rows.rs`, `observe.rs`' pass-through), so a viewer below the level that shows
  `TownOwnerFacts::garrison_hero` is told by name that the hero went into that town's garrison,
  while `docs/06-security.md`'s town part says the moves are told "as heroes leaving and
  entering the map". It is inferable there (the hero vanishes from the entrance), so the
  choice is yours: project it to a non-owner as `HeroRemoved` (and `GarrisonLeft` as a hero
  coming into sight), or say in that line that the move is public because the map shows it.
  Nothing in `h3-protocol` changes either way (the fold already reads both).

*Acted on by T28a and T28b1 (the garrison hero's marks held in `Holds::Town`, their drops, its land points):*

- 2026-11-13 (the adventure lane, E08 T115a, `adventure/e08-t115a`, `cross-lane`; **for your T28a
  and T28b, the garrison hero's marks**, your inbox note of 2026-11-11 (2)): a hero's battle
  bonuses and timed marks are `h3_adventure::VisitMarks`, one record (`bonuses()`, `timed()`,
  `add`, `add_once`, `add_timed`, `clear_bonuses`, `drop_timed(part, MarkLifetime)`,
  `week_land_points(part)`), kept for the heroes on the map in **one state block by hero id**,
  not in `HeroOnMap` (`AdventureState::visit_marks`, `change_marks`; measured: fields on the hero
  cost every fork +11 to +15 %). A garrison hero leaving the heroes on the map takes its record
  (`visit_marks`, public, then `leave_marks`, crate-visible: your `src/town*` modules reach it;
  `state/bonuses.rs`) and holds it in `Holds::Town`; the day's and week's drops are `VisitMarks::drop_timed` per record,
  and its land points take `week_land_points` (`day_points`' new last argument). Coming back onto
  the map it gives the record back through `change_marks`. `derive_battle_hero` takes the
  bonuses beside the details (`h3_heroes::BattleBonuses`; your `town/tests/siege.rs` passes
  `NONE`); a siege's defending hero on the map gets its own from `GameState::battle_hero`.
  Versions: adventure state hash v23, game state hash v31 (the details without the bonuses).
  Also (informational): CI on #906 found a case of `h3-map`'s `every_step_keeps_the_commands_
  invariants` failing on `master` 202aa3f: your T26b's `TownChanged` after a town event's
  creatures, with no build, was not among the views the turn's town-event oracle drops; it drops
  them after a `TownEventFired` now too (the case in `proptest-regressions/tests/play.txt`).

*Acted on by T27 (`HireSource::Town`, `HireTarget::Garrison`; TWN-REC-001):*

- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`; **for a town's recruitment,
  E09 decision 4**): the hire dialog is built to be reused (increment 10's design note §13,
  "E09's reuse"). `h3_adventure::HireDialog { target: HireTarget, source: HireSource }`: extend
  `HireSource` with a town's dwelling and `HireTarget` with the garrison; the map's purchase rule
  stays one (`AdventureState::hire_stock`, `hire_room`, `check_hire` → `HirePurchase`, `hire`;
  ADV-HIR-002), and so does the game's decision (`h3-game`'s `state/hire.rs`: `Decision::Hire`,
  `Answer::Hire(HireAnswer::{Buy, Close})`, the legal set of each creature's most, `hire_range`,
  `DecisionView::Hire`, `GameState::hire_dialog`; GAM-DEC-008). The env's four purchases (action
  encoding v7) take the source's order, so a town's dwelling of up to four creatures needs no
  new entry. Game state hash v25 is T95b's.

*Acted on by T25a and T26a1 (the owner's dwellings by their first creature in a town's growth, TWN-GRO-001):*

- 2026-11-04 (the adventure lane, E08 T94b, `adventure/e08-t94b`; **for the towns' external
  dwellings' growth**; its versions half done by E09 T8a1, v21): A dwelling's counts and guards as
  they stand:
  `AdventureState::dwelling_counts` and `dwelling_guards` (by object index; none of a block while
  every dwelling is as its week began); the week's refill of every dwelling is the first step of
  `AdventureState::start_week`, before the weekly objects and the camps, reading no week's or
  month's kind (ADV-DWL-003; your towns' growth is separate, survey `E08-dwellings.md` §6.1).

*Acted on by T25a and T26a1 (as the note above):*

- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **for the towns' external
  dwellings' growth**, E09 survey §8.3; adventure design note §11): the static map lists the
  dwellings in object order (`StaticMap::dwellings`, `dwelling_index`; a random dwelling not tied
  to a town is resolved into its drawn faction's and level's dwelling when the game is built, so
  it is listed as a placed one; since E09 T11 one tied to a town too), their owners are `AdventureState::owner` (ownable since T93a, the
  setup's owner at the start) and their creatures the kind's (`ObjectKind::Dwelling::creatures`,
  each creature's faction the core's). Their counts and guards come with T94b (adventure state hash
  v20). A random dwelling tied to a town resolves since E09 T11, after the towns' draws (ADR-0017
  point 4).

*Acted on by T27 (the hire dialog reused):*

- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town resolve since E09 T11. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.

*Acted on by T24, T25b1 and T26a2 (`StreamId::TOWNS`, TWN-GRO-002 to -004):*

- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.

## 2026-11-14 — E08 T116b1's note on the plans' town purchases: read at E09 T28b2's start

Informational (nothing to do); T28b2 met no plan purchase that it changed.

- 2026-11-14 (the adventure lane, E08 T116b1, `adventure/e08-t116b1`; **in the plans, for
  your information**): `h3-map`'s `play.rs` `hire_army` and `check_hire` now read a town's
  purchase into its garrison hero's army when one holds the town (your T28a's upper bar, the
  town epic's design note §49), and so does `check_towns`' purchase (its counts' change): the
  plans' oracles still expected the town's garrison, and T116b1's new draws reached the case
  (`ResourcePaid`, then `StackChanged` on `Hero(the garrison hero)`). Nothing to do.

## 2026-11-13 — E08 T114b's view changes: reviewed at E09 T26b's merge

Reviewed when T26b merged `master` in after #902: nothing to change. T26b's week takes every
viewer's views before its growth with `every` all the players, so its `Told` holds every map
garrison's views too (told after the towns' at the towns' day; no map garrison changes at a week,
so none is told); the setup's, a build's, a capture's and a flag's Portal draws sit inside
`Told`s as built. T26b took protocol v55 after T114b's v54 and re-baselined `day-start-towns-36`
again (+8.10 % against the new `master`); `day-start-builds-36` stays at T114b's baseline (+1.60 %).

- 2026-11-13 (the adventure lane, E08 T114b, `adventure/e08-t114b`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): `h3-game`'s `town/view.rs`: `visions`,
  `in_visions`, `tavern_level` and `garrison_view` are `pub(crate)` (the map garrisons' views use
  the same levels, ADV-GAR-006, `crate::garrisons`), and `Told` holds the map garrisons' views of
  `every`'s players too (`crate::garrisons::Told::before` with `every`, told after the towns'),
  and its `tell` returns at once for a sink that keeps nothing (`bench-check`'s
  `day-start-builds-36`; `day-start-builds-36` and `-towns-36` re-baselined, `master` +1.68 % and
  +1.85 % on their files after your T26a2):
  a Tavern's build and a town's capture tell the raised or lowered views of the map garrisons as
  of the towns. `town/tests/view.rs` gains
  `a_map_garrison_shows_at_a_towns_levels_and_a_new_tavern_tells_its_raised_level` (its helper
  `map_garrison`). `TWN-STUB-13`'s text names the map garrisons too (Visions' reach untold for
  them as for towns). `h3_adventure::AdventureOutcome` gains `garrison_flagged` (default false).
  No version of yours moved; protocol v54 (the server lane's, after your v53).

## 2026-11-12 — E09 T26a2: the versions taken, the hygiene note classified, the hut's week tested

The first note's map import v22 was T113b's: T26a2 took v23 (and game state hash v30, after E08 T114a2's v29). The
hygiene failure was the property plan's (a neutral plan's town visited at load is its visitor's
owner's and drew its level-2 dwelling): `h3-map`'s `hygienic_town` counts it as owned, the case's
`cc` line committed (`E09-log.md`, T26a2). The T97a note's check is `h3-game`'s
`town/tests/growth.rs` `a_creature_s_week_is_drawn_from_the_towns_stream_told_and_grown` (the
huts of the week's creature at their growth; `hiring.rs`' rules have no town part).

- 2026-11-12 (the adventure lane, E08 T113b, `adventure/e08-t113b`; **two versions taken, for
  your T26a and T28a**): map import **v22** (the garrisons and the bonus visitables imported, a
  garrison's body in the new `h3_adventure::Holds::Garrison { army, removable }`) and adventure
  state hash **v22** (`Holds` tag 11) are taken: T26a's `roe` import bump is v23 or the next at
  merge, an adventure state hash bump of yours the next after v22 (the adventure lane's T114a
  plans v23 and the game state hash v29, as your T25a note said). The 72 adventure replays'
  version lines and hashes were re-recorded; whichever merges second rebases and re-records.
- 2026-11-11 (the adventure lane, found by `adventure/srv-q26-reveal`'s scoped verify; **a hygiene
  failure on `master`, for you to classify**): `h3-map`'s `tests::view::a_viewer_learns_nothing_it_cannot_see`
  fails on `master` e085069 with `cc b9900443ad98acb94e02343d7679acd5f4888dc38169e4f8ff92d9563d82c007`
  (append it to `crates/h3-map/proptest-regressions/tests/view.txt` to replay; RoE, side 14, two
  towns with timed events): two games that differ only in what viewer 2 never sees give it day 1's
  observations that differ in one of its own towns' `buildings.built` (`BuildingSet(2097448)`
  against `6291752`: bit 22). Likely a setup draw (TWN-SET-003/-004, the towns' events) whose
  stream position moves with the hidden content `vary` changes: a leak to fix in the draws' order,
  or a `vary` the property must not make (then the adventure lane's test: tell us). Not committed
  as a regression line, so CI's random cases may meet it again.
- 2026-11-05 (the adventure lane, E08 T97a, `adventure/e08-t97a`; **for the week's creature,
  increment 3**): the map's dwellings and Refugee Camps ignore the week's and the month's kind
  (ADV-DWL-003: the original's `dwelling_week_refill` 0x4B8A60 reads neither; survey
  `E08-dwellings.md` §6.1). The month's side is tested (`h3-game`'s
  `a_month_s_kind_never_reaches_a_dwelling_or_a_camp`: a Plague and a creature month); the week's
  creature is not drawn yet. When it lands, extend that test with a week of the hut's creature:
  the hut's and the camp's counts stay at their growth.

## 2026-11-11 — E09 T22b: carried by the drafts and the design note's §52, §53

The two notes on the army reference: T108b's answer (E08 decision 305) is design note
`docs/epics/E09-design.md` §52 and the drafts' ADR-0016 point 3; T108a's reading of the exchange is
§52's rules on the bars; the garrison hero's visit marks (E08 decision 307) are §53's record (E09
decision 81). The weekday in `AdventureState::apply` (E08 T115b) is the adventure lane's, told again
when it lands.

- 2026-11-11 (the adventure lane, E08 T108b, `adventure/e08-t108b`; E08 decisions 305 and 307;
  **the army reference agreed, for T22**, answering your T21 proposal): the user accepted it as
  proposed (design note `docs/epics/E08-design.md` §13; ADR drafts
  `docs/lanes/adventure/adr-drafts/0016-increment-12.md` point 3, `0017-increment-12.md` point 6):
  a map garrison is **`ArmyRef::MapGarrison(object index)`**, a sibling of your
  `ArmyRef::Garrison(town)`; **one owner rule** in `h3-adventure`'s army commands by each bar's
  owner (a hero's its owner; a map garrison its owner when removable, none when not; a town's
  garrison its town's owner, yours to wire; the group its dialog's owner): a stack only from the
  acting player's bar, into another owner's bar only a move or a split into an empty slot, never a
  merge, swap or dismissal there, a hero's last stack stays, a stationary army may be emptied. The
  map's exchange is a new decision kind **`Exchange`** (GAM-DEC-009: the hero and the stationary
  army, `Together::Exchange { hero, army }`, closed by `Close`, commands changing the state at once,
  nothing written back); the town screen may reuse the kind or its `Together` with
  `ArmyRef::Garrison(town)` or the garrison hero's army as its upper bar (your call in T22), and
  `ArmyCommandError::Garrison` is lifted for an army the open exchange holds. Built by E08 T114a
  (round 6, after batch E12 and the `shared/` task T112); whichever lane builds first, the other
  reuses it (your decision 67). **Also for you** (decision 307): every visit's effect (the battle
  bonuses, moved out of `HeroDetails`, and the day's and the week's marks) now lives on
  `HeroOnMap` and is dropped when a hero leaves the map, so a **garrison hero** (your increment 3)
  needs them kept while it stands in a town's garrison if it leaves the heroes on the map (your
  call); `AdventureState::apply` gains the day of the week (E08 T115b, a wide mechanical change, you
  are told first); the versions of the note's §17 are taken in merge order.

- 2026-11-11 (the adventure lane, E08 T108a1 and T108a2, `adventure/e08-t108a` and
  `adventure/e08-t108a2`; **for T22 and the army reference, E09 decisions 67 and 73**): what
  T108a read of a garrison's exchange (survey `docs/rules/surveys/E08-garrisons-bonuses.md` §6):
  `garrison_exchange_dialog` 0x5D1130 builds the army dialog `garrison_dialog_create` 0x5CE830
  with the garrison's army and owner in the upper bar (no hero) and the visitor's in the lower (no
  town); its moves go through the town screen's own hover code (`town_garrison_hover` 0x5C7400,
  `town_garrison_hover_other` 0x5C8080: a move only from the current player's bar; into another
  owner's bar only into an empty slot), so the town's two rows and the map's garrisons likely
  share one dialog. Decided (E08 decision 299, D-61 the original): an ally's or a non-removable
  garrison takes the visitor's stacks into its empty slots only, no take, merge or swap. Sketched
  (survey §11, not decided): the exchange as a pending decision of the visitor's owner like the
  Leave Guards dialog (GAM-DEC-004 amended), the garrison's army in the adventure state's records.
  This agrees with your T21's reading (one exchange, survey `E09-growth.md` §9); your proposal in
  the adventure lane's inbox (`ArmyRef::Garrison(town)`, a sibling reference for a map garrison,
  one owner rule) is answered in the adventure lane's T108b drafts, which tell your inbox.


## 2026-11-02 — E09 T2b: carried by the design note `docs/epics/E09-design.md`

Each note's work is a section of the design note and a task of the cut (decision 22): the main
town's hero and a hero in a town §7, §14 (T7a, T7b); the town classes typed §8 (T8a); the income
§6, §16 (T6, T8c); ADR-0023's derivation and the experiment maps' towns §6, §15, §20 (T3a, T6,
T8b; the moat by the packs, decisions 14 and 20); the attacker's Catapult settled by ADR-0023
point 6 (§15); Conflux and the moat by expansion by decisions 14 and 20, Conflux's record in the
AB pack (§3, T5b).

- (adventure archive, 2026-10-10, T5a/T6a) A hero placed on a town's blocked tile is the town's
  visiting hero (VCMI `CGameState::placeHeroesInTowns`), and heroes generated at a main town (129
  maps then, 128 now): both `Unsupported` until towns exist. Increment 1.
- (adventure archive, 2026-10-13, T14a) Untyped blockers have no from-above setting, so a hero on
  a town's gate leaves upward: type the town classes. Increment 1.
- (adventure archive, 2026-10-30, T77a) `town_gold_income` 0x5BF600 is a town's income (read at
  T1b2a: survey `E09-towns.md` §11). Increment 1.
- (ADR-0023, Consequences) The town context's derivation of `Siege` gets a rule list from survey
  `E04c-sieges` §1; the game's version (the moat) follows the enabled packs (decision 14, E08
  decision 104; no experiment); experiment maps need towns (the writer has towns of a fixed
  faction). Increment 1.
- (adventure archive, 2026-10-14, T18b) VCMI gives every hero a catapult; the original was not
  read; it matters in sieges. Increment 1 (the siege's war machines).
- (E11, Q58) Conflux as a town and the moat by expansion are E09's.

## 2026-11-03 — E09 T6b: carried by `h3_towns::siege`

The siege's `bonuses` (`siege.rs`, TWN-SIE-003): a `TownBonus` per building ever built with a
morale or luck change, its morale while it stands and its luck once ever built (the format doc's
readings, survey §9.4: the note's "standing" for both differs only for a superseded luck
building, which no pack has); at most 8 a record (the compiler). The Grail building's
`EffectSource::Building` stays in the queue's inbox, shortened (the game's, when it gives the
owner's heroes that bonus).

- 2026-11-03 (the battle lane, CLT-Q40, BAT-Q64; ADR-0023 point 2 amended; **for your siege
  derivation**): `Siege.morale` and `luck` became `Siege.bonuses` (`h3_contracts::TownBonuses`, at
  most 8): one `TownBonus { building, morale, luck }` per standing building with a morale or luck
  role (`BuildingRoles.morale`, `luck`), `building` its `BuildingId` index; the battle sums them and
  shows them to everyone for the stats dialog. And the owner's Grail building that adds a hero's
  morale or luck (the Colossus, the Spirit Guardian) names itself as
  `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when the game gives it.

Moved from the inbox by E09 T11 (2026-11-06; its `ADV-STUB-13` part done there, its T7a part by
T7a1):

- 2026-11-04 (the adventure lane, E08 T93b, `adventure/e08-t93b`; **for T7a, the import**):
  `h3-map`'s `import/codes.rs` has `FACTIONS`, the original's faction codes 0 to 8 to the core
  factions' keys (`base:castle` … `ab:conflux`), for the towns' import to reuse (E08 design §4,
  §11). A random dwelling of class 216 or 217 whose town identifier names a town of the map stays
  an untyped blocker (ADV-STUB-13, epic E09: it takes the town's drawn faction once random towns
  are resolved); one whose identifier no town has is refused (`ImportError::DwellingTown`). Both
  bump map import (T93b v16, E09 T7a1 v17).

## 2026-11-10 — E09 T18b2a: SRV-Q26's town gap, done

- 2026-11-09 (the server lane, E16 M5 T1, SRV-Q26; **the user's decision**): **a town's garrison
  change is told to every player who sees the town.** In `towns-s173-d21` command 205 player 1's
  battle empties neutral town 4's garrison; player 0 sees the town and its `TownView::garrison`
  changes, but every event of the command is player 1's alone. A server game keeps a viewer's view
  current by events only (`docs/epics/E16-design.md` §14), so the change needs an event projected
  to every player whose view of the town changes (a `TownChanged`'s `seen_by`, or a garrison
  event). The server's fold test over the golden replays (M5 T3) checks it, a stub there until it
  lands.
  **Done** by E09 T18b2a (2026-11-10, decisions 53, 54): every viewer's changed view of a town
  told to it (`TownChanged`, `TownStackSeen`, `TownIncomeSeen`), the garrison's battle among the
  changes; Visions' reach is `TWN-STUB-13`.

## 2026-11-10 — the hygiene property's counterexample, done

- 2026-11-10 (the server lane, on #816's CI; **a counterexample of your proptest**): `h3-map`'s
  `tests::view::a_viewer_learns_nothing_it_cannot_see` (`src/tests/view.rs`, E09 T18b2a's town
  views) failed under CI's coverage step (run 42779356175, job 129705398501, on #816's cfcc5c6,
  whose diff touches only `h3-protocol`): its shrunk `vary` adds `((1, 252, 252), 2)`,
  `((175, 156, 5), 2)` and `((128, 184, 82), 0)`, with no removals, moves, owners or towns. The
  panic message is in that job's log, ahead of the input; 12 local release runs (576 random cases)
  passed. Find the case (more cases, or the log's input), fix it and keep its seed in
  `proptest-regressions/tests/view.txt`.
  **Done** (2026-11-10, `town/e09-view-hygiene`, `cross-lane`): the seed replayed on `master` as
  CI ran it (780fbcc): another player's hero accepted a monster's offer to join beside an
  Alchemist's Lab the viewer saw; its army, which the property varies where the viewer never sees
  it, had room in one game (the creatures joined, the lab's visit flagged it at once) and not in
  the other (an army dialog opened and held the visit, as the original's does). A gap of the
  property, not a leak: `outcome` counted only the offers among the decisions; it counts every
  decision opened or closed now, and the seed is kept in `proptest-regressions/tests/view.txt`.

## 2026-11-11 — E09 T20: kept as built (decision 67)

- 2026-11-11 (the adventure lane, E08 T107, `adventure/e08-t107`, #844; **for your town checks,
  nothing to do unless you prefer one event**): CI's random plans found a counterexample of
  `master`'s `check_towns` (`h3-map`'s `tests/play.rs`): a siege won by a player who holds another
  Capitol tells the old owner `TownChanged` with `capitol: true` after the garrison's battle, then
  the capture's Capitol limit (TWN-CAP-002) demotes the town and both viewers are told `capitol:
  false`. The engine is consistent (each event tells the town as it stood); the check assumed
  every `TownChanged` of a command carries the final Capitol, so it now compares each viewer's
  last one. The seed is kept in `proptest-regressions/tests/play.txt`. If you would rather not
  tell the old owner the intermediate town (the capture follows in the same command), the check
  holds either way.

(Done by E09 T28a, 2026-11-14: the town's garrison arm of `held` and the bars' admission.)

- 2026-11-12 (the adventure lane, E08 T114a2, `adventure/e08-t114a2`, `cross-lane`; **a version
  taken and the hook for your T28a**): game state hash **v29** (a map garrison's exchange among the
  decisions, tag 8), so your T26a2's bump is v30 (done, #899); adventure replay format **v13** (`m<index>:<slot>`,
  a map garrison's army) and protocol **v52** (the server lane's, `cross-lane`); every adventure
  golden replay re-recorded (versions and hash lines only). The one owner rule is in
  `h3-adventure`'s `state/manage.rs`: `held` gives each army with its bar ownership (a hero's and
  the group the player's, a map garrison its owner's when removable, nobody's when not), so your
  `ArmyRef::Garrison(town)` gets its arm there (its town's owner) and `Held::keeps_last` stays false
  for it; `h3-game`'s `Together::Exchange { hero, army }` holds a stationary `ArmyRef` (a town's
  garrison may be its `army`), and `Together::admits` refuses a map garrison its decision does not
  hold (`DecisionOpen`; alone `NotTogether`). Design note §13's *As built* (`E08-design.md`).

## 2026-11-16 — archived at E09 T34

Used by batch TWN-4's recipes (E09 T34; the gold bonus, `hint_at`).

- 2026-11-12 (the adventure lane, E08 T110, `adventure/e08-t110`; the capture harness, informational, for your next batch): `h3drive.gold_bonus` now clicks each row's arrow until the memory reads Gold (a heroless town player has no Artifact in its row); `hint_at` parks the mouse and reads until two reads agree; `play_turns` gives no order once the battle is decided; a recipe's `+anything odd` is kept beside the notes. New steps and reads: `clients/capture/README.md`, "Garrisons and the bonus visitables".
