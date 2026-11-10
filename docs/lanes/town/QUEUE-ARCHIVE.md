# Town lane — QUEUE archive

Inbox notes that left the queue, verbatim, with where their work went (`docs/08-work-process.md`,
"Keeping the start-of-session reading small"). Read only when a doc points here.

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
