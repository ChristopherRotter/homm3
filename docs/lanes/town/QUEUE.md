# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entry below is round 4's (rounds 2 and 3, T3a to T3c, are done, `E09-log.md`); each later round's
entries are written when it comes (the epic's list), each task's cut confirmed at its start. Every
entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T4 — `shared/`: the ADRs, the glossary, the crate skeletons, `TownId`, the gates**

- **Scope:** round 4 (design §1, §21; decision 21), one `shared/<topic>` pull request that merges
  first: the five drafts of `adr-drafts/` into `docs/adr/` with batch TWN-1's verdicts (survey
  §15: TD-3, TD-5, TD-7 and the siege's aftermath confirmed; building 23 changes no ADR step,
  decision 25), ADR-0028's number checked; the skeletons `h3-towns` and `h3-pack-towns` (an empty
  part and store, both version constants 1); `arch.toml`'s edges; `lanes.toml`
  (`crates/h3-pack-towns/`, `packs/*/towns.toml` to the town lane); `TownId` in `h3-contracts`;
  the versions table's two rows; `docs/03-domain.md`'s terms, `docs/02-architecture.md` §3,
  `Moat`'s doc comment; a note in the battle lane's inbox on `siege::setup`'s comment (design §1
  lists each).
- **Acceptance:** the drafts accepted in `docs/adr/` (the index's "Read when" rows), the skeletons
  build under every gate (`check-arch`, coverage, `check-lane`), CI green on the pull request.
- **Notes:** a cut at its start may split it (the ADRs and docs, then the skeletons). The
  adventure lane's open pull requests first (`TRAPS.md`, "Merge in order").
- **Depends on:** T3c (done). **Status:** ready.

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
