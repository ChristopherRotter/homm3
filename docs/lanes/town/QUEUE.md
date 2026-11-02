# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entries below are rounds 2 and 3; each later round's entries are written when it comes (the
epic's list), each task's cut confirmed at its start. Every entry's acceptance includes the
Definition of Done (`docs/05-testing.md`).

**E09 T3a — batch TWN-1's maps and spec**

- **Scope:** round 2 (design §20; decision 15): the spec `experiments/specs/E09-batch-1.toml`
  for survey §13's rows 1 to 4 and its maps from `h3-experiment`: row 1 owned standard towns
  without a Fort and with one, neutral ones with and without (several of each, or several new
  games for the 30 in 100); row 2 a neutral town with an explicitly empty custom garrison and a
  control without one; row 3 Blue's town with a lone visitor of seven stacks of mixed values and a
  garrison of mixed values, Red's hero besieging (an encounter's opponent town with its hero and
  garrison, E04c T5b's `TownSpec`, as it stands); row 4 a defended neutral town Red
  besieges and wins, then a later step onto the gate. `[[map.town]]` and `TownSpec` gain what the
  rows lack (today an absent `buildings` is the standard set without a Fort and an empty
  `garrison` the map's default): the standard set with a Fort, an explicitly empty custom
  garrison (`h3-experiment` is the battle lane's: `cross-lane`, the user's
  consent at the start, the label at once). Each row's prediction from survey §13.
- **Acceptance:** the spec writes its maps; `h3-experiment`'s tests read each new field back
  through `h3_map::read`; `cargo xtask check-lane --cross-lane`.
- **Depends on:** nothing. **Status:** ready.

**E09 T3b — batch TWN-1's harness and run**

- **Scope:** round 3 (design §20): the harness (`clients/capture/`, common; this lane builds it
  with its check and notes the client and map-client lanes' inboxes): reads of a town's record
  (owner +0x1, built +0x150, every-built +0x158, garrison +0xE0, visitor +0x10), the siege's
  opening stacks (E04c's reads), a hero's tile and points after a battle (E08's); the recipe
  `E09-batch-1`; the run in SoD on Easy, hot seat, as E08's T79 (original data staged); the
  results in `experiments/results/E09-batch-1.md`.
- **Acceptance:** every row's record filled; the harness's check passes.
- **Depends on:** T3a. **Status:** waiting (T3a).

**E09 T3c — batch TWN-1 scored**

- **Scope:** round 3: survey `E09-towns.md` §15, each row against its prediction (TD-3, TD-5,
  TD-7, the siege's aftermath); a row that disagrees reopens design §5 or §10 and the ADR step it
  names (a `TWN-Q` before T4 applies the ADRs); `docs/experiments/README.md`'s row.
- **Acceptance:** the verdicts in the survey and the design note.
- **Depends on:** T3b. **Status:** waiting (T3b).

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 3 (weights: only the tavern's draw reads them, survey `E09-towns.md` §4.4), increment
  4 (chances).
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
