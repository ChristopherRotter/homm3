# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entries below are round 7's (rounds 2 to 6 are done, `E09-log.md`); each later round's entries are
written when it comes (the epic's list), each task's cut confirmed at its start. Every entry's
acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T7a — the import and the setup's checks** (`cross-lane`)

- **Scope:** round 7 (design §7, §13; ADR-0015's amendment point 3): `h3-map`'s import of the
  players' starts, the towns (classes 77, 98: the adventure's object and `GameSetup::towns[id]`, a
  `h3_towns::TownSetup` each, T6c), the random hero (class 70), a hero right of a town's entrance
  moved (TD-6, TWN-SET-010) and the pool's inputs; `PlayerSetup::start`, `GameSetup`'s new fields
  and `GameSetup::validate`'s new errors (§13); map import v16; `TWN-QUIRK-1` to `-4` in
  `docs/rules/quirks-town.md` (T6c created it with `-5`); the census with the original data
  (every shipped map imports, the custom sets with a horde, the Shipyards without water:
  `TWN-STUB-2`); the rules TWN-SET-001, -002's checks, -006, -010 as §19 names them.
- **Acceptance:** every rule with its id, source and test; 100 % coverage of the crates touched;
  the scoped verify with `--with-original-data`; the census in `docs/formats/h3m.md` ("Import").
- **Notes:** large: cut it at its start (the import, then the setup's checks, is a seam). The
  adventure lane's paths (`h3-map` outside `src/import/town*`, `setup.rs` in `h3-game`) make it
  `cross-lane`; look at the adventure lane's HANDOFF first: its T93 (increment 10's import) may
  bump map import before, then this task takes the next version (the lane's `TRAPS.md`).
- **Depends on:** T6c (its pull request merged). **Status:** ready once it merges.

**E09 T7b — the pool and the build's draws** (`cross-lane`)

- **Scope:** round 7 (design §14; ADR-0017's amendment points 1 and 4): `h3-game`'s `HeroPool`
  (TWN-SET-005; T-2, T-3 fixed), the build's draws from the setup stream in §14's order (the
  players' random choices TWN-SET-001, a random town's faction TWN-SET-002, a random hero's type
  TWN-SET-008, each main town's hero TWN-SET-009, the draw TWN-SET-007 with
  `GameSetupError::NoFreeHero`, then each town's `h3_towns::town_setup`, T6c); the store of
  towns beside the adventure's state, the garrisons and reveal radii into the adventure's setup;
  game state hash v23; the replays re-recorded where the hash moves (skill
  `homm3-rerecord-replays`).
- **Acceptance:** as T7a's; the determinism replays green.
- **Depends on:** T7a. **Status:** after T7a; its cut confirmed at its start.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-03 (the battle lane, CLT-Q40; its siege half done by T6b, `QUEUE-ARCHIVE.md`): the
  owner's Grail building that adds a hero's morale or luck (the Colossus, the Spirit Guardian)
  names itself as `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when
  the game gives it.
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
