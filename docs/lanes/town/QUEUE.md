# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

T2 was cut in two at its start (the user, 2026-11-02): T2a the ADR drafts, T2b the design note,
the cut and the checkpoint.

- **T2a The ADR drafts** (in progress (`town/e09-t2a`, 2026-11-02 21:20 UTC); T1's survey and
  checkpoint done: `E09-towns.md` §0 to §14, the epic's decisions 8 to 16, `TWN-Q2`). A new ADR
  draft for `h3-towns` in `docs/lanes/town/adr-drafts/` (the crate, the town's state and its home
  in the game state, its fork and hash, the ruleset part and pack file, the seams, the hero
  pool's home) and the amendments of ADR-0015 to -0017 and ADR-0023 the survey calls for: the
  setup's faction and starting-hero pick per colour (decisions 6 and 9), the pool's draws for the
  main town's and the random hero (decision 8; T-2 to T-4 by decision 12), the moat and Conflux
  by the enabled packs (decision 14; ADR-0023's version line). The ADR-level choices asked as
  they come up (`TWN-Q3`).
- **T2b The design note, the cut and the checkpoint** (after T2a). `docs/epics/E09-design.md`:
  the import (random towns, the main town's and the random hero, the random hero's body as a
  placed hero's with the 18 custom experiences, the adventure queue's 2026-10-20 note), the
  state, the visit and capture with the team test and the ally's stub (decision 16), the income
  with the silo (decision 10), the views by the original's levels (decision 11) for
  `docs/06-security.md`, the content and its harvest (the buildings' identities, halls, forts,
  the silo's table, the neutral garrison's table and the 30). The build's rounds as queue
  entries, batch TWN-1's rows 1 to 4 as T3 (decision 15); the checkpoint (`TWN-Q4`; the ADR
  drafts accepted with it).

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- (adventure archive, 2026-10-10, T5a/T6a) A hero placed on a town's blocked tile is the town's
  visiting hero (VCMI `CGameState::placeHeroesInTowns`), and heroes generated at a main town (129
  maps then, 128 now): both `Unsupported` until towns exist. Increment 1.
- (adventure archive, 2026-10-13, T14a) Untyped blockers have no from-above setting, so a hero on
  a town's gate leaves upward: type the town classes. Increment 1.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 3 (weights: only the tavern's draw reads them, survey `E09-towns.md` §4.4), increment
  4 (chances).
- (adventure archive, 2026-10-30, T77a) `town_gold_income` 0x5BF600 is a town's income (read at
  T1b2a: survey `E09-towns.md` §11). Increment 1.
- (ADR-0023, Consequences) The town context's derivation of `Siege` gets a rule list from survey
  `E04c-sieges` §1; the game's version (the moat) follows the enabled packs (decision 14, E08
  decision 104; no experiment); experiment maps need towns (the writer has towns of a fixed
  faction). Increment 1.
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 3.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 3, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 4.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 4.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 4.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 4.
- (adventure archive, 2026-10-14, T18b) VCMI gives every hero a catapult; the original was not
  read; it matters in sieges. Increment 1 (the siege's war machines).
- (E11, Q58) Conflux as a town and the moat by expansion are E09's.
