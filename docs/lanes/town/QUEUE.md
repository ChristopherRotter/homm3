# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entries below are round 6's (rounds 2 to 5 are done, `E09-log.md`); each later round's entries are
written when it comes (the epic's list), each task's cut confirmed at its start. Every entry's
acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T6 — `h3-towns`: the record, the store, the setup's functions, the derivations**

- **Scope:** round 6 (design §4 to §6; the lane's own crate): the record `Town` (`built`,
  `forbidden`; the standing set derived) and `TownStatic`, the store `Towns` and its canonical
  hash (town state hash v1's first encoding), the build's setup functions
  (`town_buildings_setup` TWN-SET-003, the garrison TWN-SET-004, drawn from the setup stream),
  the derivations of §6's table, and `docs/rules/catalog/town.toml` with the rules §19 names for
  them; the quirks T-4 and T-6 where §19 places them (`docs/rules/quirks-town.md`).
- **Acceptance:** every rule with its id, source and test; 100 % coverage of `h3-towns`; the
  scoped verify.
- **Notes:** probably cut in two at its start (the record, store and derivations; the setup's
  functions). §6's `siege(town)` gives `Siege.bonuses` (the inbox's CLT-Q40 note), not one morale
  and luck. No `cross-lane` path: `h3-game` calls these from T7.
- **Depends on:** T5a2 (its pull request merged). **Status:** ready once it merges.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-03 (the battle lane, CLT-Q40, BAT-Q64; ADR-0023 point 2 amended; **for your siege
  derivation**): `Siege.morale` and `luck` became `Siege.bonuses` (`h3_contracts::TownBonuses`, at
  most 8): one `TownBonus { building, morale, luck }` per standing building with a morale or luck
  role (`BuildingRoles.morale`, `luck`), `building` its `BuildingId` index; the battle sums them and
  shows them to everyone for the stats dialog. And the owner's Grail building that adds a hero's
  morale or luck (the Colossus, the Spirit Guardian) names itself as
  `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when the game gives it.
- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town wait for your random towns. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.
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
