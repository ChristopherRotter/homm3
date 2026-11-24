# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades: the Marketplace's screens with the
  Artifact Merchants and the Freelancer's Guild on E08's market, the Blacksmith, the town's
  Shipyard, the Magic University, the Skeleton Transformer) refined at T51 (`TWN-Q34`, decisions
  196–199: the services split, 7 the buildings for heroes and the map, travel and the Grail 8;
  T51's audit and the closure in `E09-log.md`). **Next:** **T52**, the survey with its
  checkpoint `TWN-Q35` (`docs/rules/surveys/E09-trades.md`, docs only, needs the original data;
  its cut confirmed at its start), then **T53** the drafts with Part H from §101 (`QUEUE.md`).
  In the inbox for review: E08 T141a's note (2): the Thieves' Guild's best hero row may follow
  the list of heroes (`AdventureState::hero_list`, today by id). Waiting elsewhere: `TWN-STUB-8`
  (the battle lane's inbox), `-13` (Visions' reach), `TWN-STUB-2` (the victory conditions),
  `TWN-STUB-16` (E10's loss condition), `-17` (increment 8), `-18` (E13). For the next `shared/`
  task: `docs/06-security.md`'s ADV-FOG-006 line (T50's record).
- **Open pull requests:** T51's (`town/e09-t51`, docs only) and the README's row
  (`shared/e09-t51-readme`); watch them until they merge (T50's #1144 merged 2026-11-24).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (game state hash v39, map import v29, adventure part v25, adventure state hash v28: E08
  T141a; replay format v25, T49b), the env's (v20) and the protocol's (v81, both T141a) are bumped
  `cross-lane` (each task's record in `E09-log.md`). Shared: core ruleset hash v53, pack format
  v53 (E08 T124a), quirk flags 51 (E08 T140; `TWN-QUIRK-14`'s at T45).
- **Ids** (next): questions `TWN-Q35`; decisions 200; quirks `TWN-QUIRK-15`; stubs `TWN-STUB-21`;
  experiment batch `TWN-6`; disputes `TD-28`; quirk candidates `T-19`. ADR-0028 is the lane's; no
  drafts open; every TWN-GLD rule `verified` but TWN-GLD-007 (no row bought a book: T51).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-24 — T51: the audit, increment 5 closed, increment 6 refined, docs only

Baseline 25397e7 (#1144 merged). The user's computer was connected: 5 files staged, SHA-256
checked. At the start (`TWN-Q34`, with a first look at the audit) the user kept T51 whole and
split the services: 6 the town's trades, 7 the buildings for heroes and the map, 8 travel and
the Grail (decisions 196–198). The twelve criteria hold; the gap: eight rules batch TWN-5
settled were still `implemented`, now `verified`. The VM restarted once during the setup's
baseline; the scoped verify ran again on the branch (the engine's code `master`'s).

### 2026-11-24 — T50: the guild census, properties, map, replays and workload, `cross-lane`

Baseline d202566 (#1136 merged; the setup's light gates, CI's fingerprint on `master`'s code).
The user's computer was connected: 5 files staged, SHA-256 checked. The agent recommended a cut
in two; the user kept T50 whole with the inbox's hygiene failure and asked for sub-agents
(decision 195): four ran one after another (the failure, the properties, the census, the map).
The failure was the property's (`TRAPS.md`). The census holds on all 160 maps with 25 survey
counts, three mutations caught; the properties cost +1 % (a first draft +29 %). The VM restarted
twice and the server once; each part was pushed as it came, nothing lost.

### 2026-11-23 — T49b: a spell book bought at a town's Mage Guild, `cross-lane`

Baseline f1c4a53 (#1128 merged; the setup's light gates, CI's fingerprint on `master`'s code).
The user's computer was connected and its shell worked: 71 files staged, SHA-256 checked. The
agent recommended a cut in two; the user kept T49b whole (decision 194). Built: the command, its
three refusals, `check_spell_book`, `guild_offers`, the game's `buy_book` and `offers`; the
wire, the word, `SPELL_BOOK` (104); the plans. The town map's seed 2541 lost its won siege: 277
after a search of 3 000. `legal-towns-36` on `master`'s games +5.5 % at the first build, +1.6 %
after the design pass (`TRAPS.md`); `day-start-builds-36` re-baselined (its games bought books).
CI's first run met a gap of T49a1's in `h3-map`'s `check_towns` (a hire's Vortex flag): taught.
