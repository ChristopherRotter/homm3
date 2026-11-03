# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 1: towns on the map); done tasks in `E09-log.md`.

Increment 1's build (decision 22; `E09-design.md` §21): rounds 2 to 10, batch TWN-1 first. The
entries below are round 5's (rounds 2 to 4 are done, `E09-log.md`); each later round's entries
are written when it comes (the epic's list), each task's cut confirmed at its start. Every
entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T5a — the town part, its compiler and adapter, `GameRuleset`, replay format v9**

- **Scope:** round 5's first half (design §2): `TownRuleset` in `h3-towns` (`ruleset`: per
  faction a town record with its buildings and roles, the silo, the moat and `siege_obstacles`,
  the creatures by level; the part's parameters; `TownRuleset::empty(core)`), its compiler with
  §2's checks and its canonical hash (town ruleset part v1's first encoding; the constant stays
  1); `h3-pack-towns`' `compile(texts, profile)` (strict parsing as `h3-pack-adventure`'s,
  overrides as ADR-0006's, a folder without the file an empty one; the format documented in its
  `lib.rs`); `GameRuleset::new(core, adventure, battle, towns)` with the extended mismatch check
  and its callers (h3-sim's encounter and tests, h3-cli's adventure loader and census, h3-map's
  tests and examples; `cross-lane`); the adventure replay's header records the part's
  hash (adventure replay format v9; the golden replays re-recorded, skill
  `homm3-rerecord-replays`).
- **Acceptance:** the compiler's checks each refused by a test; the part's hash stable across
  runs and forks; 100 % coverage on `h3-towns`; the adventure replays at v9 with only header and
  hash lines moved; `check-lane --cross-lane`; CI green.
- **Notes from T4b:** `TownRuleset` and `Towns` are empty `Default` structs, replaced here and in
  T6; the two crates have no dependencies yet (their `Cargo.toml`s are the lane's). `arch.toml`
  already allows the callers' edges (`h3-sim` → `h3-towns`, dev `h3-pack-towns`; `h3-cli` →
  both; `h3-map`'s dev → both) and `clients.yml` watches
  `crates/h3-towns/`, so no `shared/` change is needed; once `h3-game` depends on `h3-towns`,
  `clients/Cargo.lock` (common) changes and `clients/scripts/check.sh` runs. A likely cut at its
  start: T5a1 the part, its compiler and the adapter (the lane's own crates); T5a2 `GameRuleset`,
  its callers and replay format v9 (`cross-lane`). The adventure lane's open pull requests first
  (`TRAPS.md`, "Merge in order"; it bumps the replay format too).
- **Depends on:** T4b (merged). **Status:** ready once T4b's pull request merges.

**E09 T5b — the content read and harvested; building 23**

- **Scope:** round 5's second half (design §3, decisions 23 and 25): the code reading of
  `building_tables_init` 0x4EB730 and the tables §3 names (every address into
  `docs/exe/map.toml`), the names from the text files (no descriptions, Q13), VCMI 1.7.5's
  factions and the wiki compared (a difference with gameplay impact a `TD-` row before it is
  written); building 23's source, whether a custom set gets it and whether the faction's list
  holds it; `h3-harvest`'s gap-fill (`cross-lane`, the battle lane's crate, as E08 T81b) writes
  `packs/base/towns.toml`, `packs/ab/towns.toml`, `packs/sod/towns.toml` (decision 20's moats);
  the synthetic packs' small hand-written records.
- **Acceptance:** the original packs compile with T5a's compiler; `cargo xtask verify
  --with-original-data` (the harvest reproduces the files); the survey's new `TD-` rows if any;
  CI green.
- **Depends on:** T5a. **Status:** after T5a; needs the original data staged (`Heroes3.exe`, the
  LODs' text files).

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
