# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades: the Marketplace's window with the
  Artifact Merchants and the Freelancer's Guild on E08's market, the Blacksmith, the town's
  Shipyard, the Magic University, the Skeleton Transformer; decisions 196–207): **T52** the survey
  `docs/rules/surveys/E09-trades.md` with its checkpoint `TWN-Q35` (decisions 200–207: D-70 the
  original; TD-28 `fix-optional` as T-28; the trader's count at least 1; TD-30 with T-23
  `fix-optional`; Misc 5 sellable; one fixed boat spot; the agent's calls each asked). **Next:**
  **T53**, the drafts with Part H from §101, the cut with batch TWN-6's eight rows (`QUEUE.md`;
  the survey's §10 open points). In the inbox for review: E08 T141a's note (2): the Thieves'
  Guild's best hero row may follow the list of heroes (`AdventureState::hero_list`, today by id).
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `TWN-STUB-2`
  (the victory conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 8), `-18`
  (E13). For the next `shared/` task: `docs/06-security.md`'s ADV-FOG-006 line (T50's record).
- **Open pull requests:** T52's #1161 (`town/e09-t52`, docs only); watch it until it merges (T51's
  #1149 and the README's #1148 merged 2026-11-24).
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
- **Ids** (next): questions `TWN-Q36`; decisions 208; quirks `TWN-QUIRK-15`; stubs `TWN-STUB-21`;
  experiment batch `TWN-6` (its eight rows planned, survey §11); disputes `TD-32`; quirk
  candidates `T-29`. ADR-0028 is the lane's; no
  drafts open; every TWN-GLD rule `verified` but TWN-GLD-007 (no row bought a book: T51).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-24 — T52: increment 6's survey and its checkpoint TWN-Q35, docs only

Baseline 7782735 (#1149, #1148 merged; the setup's full baseline on `master`'s code passed). The
user's computer was connected: 5 files staged, SHA-256 checked. The user kept T52 whole and asked
for sub-agents (decision 200): seven narrow jobs (four code readings, VCMI, the wiki, the census),
at most two at a time, each writing a notes file the session merged (46 new map entries, 35
changed). The wiki's Freelancer's Guild, Ballista Yard and Magic University pages did not load.
At the checkpoint the user chose three answers of their own (TD-28 `fix-optional`, the count
at least 1, Misc 5 sellable) and had the agent's calls asked one by one.

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
