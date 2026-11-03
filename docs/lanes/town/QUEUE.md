# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entries below are round 5's (rounds 2 to 4, T5a1 and T5b1 are done, `E09-log.md`); each later round's
entries are written when it comes (the epic's list), each task's cut confirmed at its start. Every
entry's acceptance includes the Definition of Done (`docs/05-testing.md`). Round 5's order is
T5a1, T5b (cut in two: T5b1, T5b2), T5a2 (the user, 2026-11-03: the content before
`GameRuleset`, so the golden replays' header moves once).

**E09 T5b2 — the content harvested**

- **Scope:** round 5's second part, after T5b1's reading (T5b cut in two at its start, the user;
  design §3, decisions 23, 27 to 30; survey §16): `h3-harvest`'s gap-fill (`cross-lane`, the
  battle lane's crate, as E08 T81b) reads the executable's tables of survey §16 (an
  `h3-formats` reader of the faction lists 0x63EC80, the supersede lists 0x63FBC4,
  `map_building_ids` 0x6888C0, `town_creatures` 0x6747B4, the hordes 0x6887A0, the silo
  0x688EB4, the garrison 0x688E84; every address is in `docs/exe/map.toml`) and the names
  (`BldgNeut.txt`, `BldgSpec.txt`, `Dwelling.txt`; names only, Q13), and writes
  `packs/base/towns.toml` (the eight factions; the moat as RoE and AB play it),
  `packs/ab/towns.toml` (Conflux), `packs/sod/towns.toml` (the overrides; decision 20). Each
  record: the faction's listed buildings plus the Village Hall (`hall = 1`), not 23 or 27 to 29
  (decision 27) nor the ship 20; `slot` from `map_building_ids`; `supersedes` the common and the
  faction's lists (direct); the roles of survey §16.6 (TD-8 by the code: the Fortress' 21
  `defender.attack = 2`, 22 `defense = 2`, decision 30); `creatures` seven levels with upgrades
  from `town_creatures`; `silo`; `[town_params]` (the gold, the halls, the Grail, the reveal, the
  30, the garrison rows, `capitol_demoted_to`). **The hordes' upgrade as data** (decision 29: the upgraded
  horde 19 or 25 ever built once its horde and the level's upgraded dwelling are; the field is
  this task's call, e.g. a building's `auto = true`, VCMI's `mode: auto`; `h3-towns`' compiler
  checks it, T6's setup applies it); the keep's derivation (decision 28) is T6's. The synthetic
  packs' small hand-written records (`synthetic`: `vale`, levels 1 and 2 without upgrades;
  `synthetic-ext`: an override, as T5a1's test files); `packs/README.md`'s row ("no pack has
  one yet"; the battle lane's file); `h3-pack-towns`' constants for the files (`SYNTHETIC_TOWNS`,
  `BASE_TOWNS`, …, `ORIGINAL_PACKS`, as `h3-pack-adventure`'s).
- **Acceptance:** the original packs compile with T5a1's compiler, each of the nine factions with
  seven levels and their upgrades and its listed buildings (a test); `cargo xtask verify
  --with-original-data` (the harvest reproduces the files); CI green.
- **Notes:** the hordes and their upgrades share a name in every faction (keys need a suffix);
  Conflux's Grail is spelled "Aurora Borealias" in the texts; the format and each role's set are
  in `h3-pack-towns`' `lib.rs`. TD-9 (a requirement) and every requirement wait for increment 2.
- **Depends on:** T5b1 (its pull request merged). **Status:** ready once it merges; needs the
  original data staged (`Heroes3.exe`, the LODs or `extracted.tar`).

**E09 T5a2 — `GameRuleset`, its callers, adventure replay format v9**

- **Scope:** round 5's last part (design §2's `GameRuleset` paragraph; `cross-lane`):
  `GameRuleset::new(core, adventure, battle, towns)` with the extended mismatch check (the town
  part on the same core; every faction with a town record has the battle part's `[[siege]]`
  record) and its callers (h3-sim's encounter and tests, h3-cli's adventure loader and census,
  h3-map's tests and examples: the compiled part, or `TownRuleset::empty(core)` for an
  encounter); the CLI's loader reads `towns.toml` beside `adventure.toml` and `battle.toml`; the
  adventure replay's header records the part's hash (adventure replay format v9; the golden
  replays re-recorded, skill `homm3-rerecord-replays`).
- **Acceptance:** the mismatch check's new cases each refused by a test; the adventure replays at
  v9 with only header and hash lines moved; `check-lane --cross-lane`; CI green.
- **Notes:** `arch.toml` already allows the callers' edges (`h3-sim` → `h3-towns`, dev
  `h3-pack-towns`; `h3-cli` → both; `h3-map`'s dev → both) and `clients.yml` watches
  `crates/h3-towns/`; once `h3-game` depends on `h3-towns`, `clients/Cargo.lock` (common) changes
  and `clients/scripts/check.sh` runs. The adventure lane's open pull requests first
  (`TRAPS.md`, "Merge in order"; it bumps the replay format too).
- **Depends on:** T5b2. **Status:** after T5b2.

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
