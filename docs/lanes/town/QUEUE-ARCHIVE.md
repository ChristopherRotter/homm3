# Town lane — QUEUE archive

Inbox notes that left the queue, verbatim, with where their work went (`docs/08-work-process.md`,
"Keeping the start-of-session reading small"). Read only when a doc points here.

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
