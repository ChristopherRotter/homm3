# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

- **T1b2 The towns survey, part 2b: income, views and the checkpoint** (next; T1b1, the visit,
  capture and the battle at a town, done: survey §7 to §10). The hall's income
  (`town_gold_income` 0x5BF600, `player_gold_income` 0x4C7930) and what the Capitol pays (one
  per player after a capture: `capitol_limit`, §8.4; the wiki's word on it not found at T1b1);
  what a viewer sees of a town (the map client's readings, E24 M3 T5b1 and T6a1: the status
  window, the hint); batch TWN-1's rows gathered (T1a2's TD-3 and TD-5, survey §6; T1b1's four,
  §10); the checkpoint's questions to the user (`TWN-Q2`, interactive, with options and a
  recommendation each: the random hero in increment 1 (§4.4), T-2 to T-5's classifications,
  TD-1 to TD-7 where a choice is the user's, the scope's other open points of §6 and §10).
- **T2 The ADR drafts, the design note and the cut** (after T1b's checkpoint): a new ADR draft
  for `h3-towns` in `docs/lanes/town/adr-drafts/` and the amendments of ADR-0015 to -0017 and
  ADR-0023 the survey calls for; `docs/epics/E09-design.md`; the build's rounds as queue entries;
  the checkpoint (`TWN-Q3`).

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
- (adventure archive, 2026-10-30, T77a) `town_gold_income` 0x5BF600 is a town's income. Increment 1.
- (ADR-0023, Consequences) The town context's derivation of `Siege` gets a rule list from survey
  `E04c-sieges` §1; the game's version (the moat) is probably the map's format (an experiment
  candidate); experiment maps need towns (the writer has towns of a fixed faction). Increment 1.
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
