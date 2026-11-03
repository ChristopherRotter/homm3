# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). **T1 done** (the survey `E09-towns.md` §0 to §14; `TWN-Q2`, decisions 8–16). **T2 done**: T2a the ADR drafts in `adr-drafts/` (`TWN-Q3`, decisions 17–20); T2b the design note `E09-design.md` §1 to §22 and the cut (`TWN-Q4`, decisions 21–24: the drafts accepted as written, the cut as proposed, the content read and harvested, the agent's calls kept). **T3a done** (round 2): `fort` and an explicitly empty `garrison` for a spec's towns (`h3-experiment`, `cross-lane`), the spec `experiments/specs/E09-batch-1.toml` and its template. **T3b done** (round 3): the capture harness's town reads, a new-game step and the town screen; batch TWN-1 played twice under Wine (`experiments/results/E09-batch-1.md`, `-run2.md`). **T3c done** (survey §15): TD-3, TD-5, TD-7 and the siege's aftermath as the reading; building 23 in every Rampart, Necropolis and Conflux town of the standard set, not predicted, re-read in T5b (`TWN-Q5`, decisions 25, 26). **T4 done** (round 4, cut in two at its start, the user): T4a ADR-0028 and E09's amendments of ADR-0015, -0016, -0017, -0023 in `docs/adr/`, the glossary; T4b the skeletons `h3-towns` and `h3-pack-towns`, `TownId`, the gates' files (`shared/e09-t4b`). **Round 5** (T5a cut in two, the order changed, the user: T5a1, T5b, T5a2): **T5a1 done** (the part `TownRuleset`, its compiler and hash, `h3-pack-towns`' parser and format). **T5b** cut in two (the user): **T5b1 done** (the reading, survey §16; building 23 the Village Hall's town-screen companion in three factions; `TWN-Q6`, decisions 27–30); **T5b2** cut in two again (the user): **T5b2a done** (a building's `auto_with`, the hordes' upgrade; the synthetic packs' `towns.toml`). Next: **T5b2b** (the reader, the harvest and the three packs, `cross-lane`; original data), then **T5a2** (`GameRuleset`, its callers, replay format v9; `cross-lane`), both in the queue; rounds 6 to 10 in the epic's list.
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding by T6), the part's 2 (T5b2a's `auto_with`, pinned in its test). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (next 1; 1 to 5 reserved by the design §19 for T-1 to T-4 and T-6), stubs `TWN-STUB-<n>` (next 1; 1 and 2 reserved by the design §19), rules `TWN-*` (none yet; the design §19 names them), decisions next 31, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 1; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 10) and quirk candidates `T-<n>` (next 7).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-03 — T5b2a: the hordes' upgrade as data, the synthetic town records

Baseline 9e94c8f (#678 merged; CI on `master` green at 1642a86; the sandbox's baseline the light
gates). T5b2 cut in two (the user: the town lane's format first, the `cross-lane` harvest next).
`auto_with` in `h3-towns` and `h3-pack-towns`, the part's hash v2; `packs/synthetic*/towns.toml`
and their constants. Pull request: the one of branch `town/e09-t5b2a`.

### 2026-11-03 — T5b1: the buildings' tables read, building 23 found

Baseline 0e37885 (#675 and #677 merged; CI on `master` green at 1642a86; docs only, no build).
T5b cut in two (the user: the reading, then the harvest). Survey §16, the address map; the
checkpoint `TWN-Q6` (decisions 27–30: the companions left out, the keep's count without the
Village Hall in the derivation, the standing set derived with T-6 fixed, TD-8 by the code).
Pull request: the one of branch `town/e09-t5b1`.

### 2026-11-03 — T5a1: the town part, its compiler and the adapter

Baseline 90e8638 (#671 merged; CI on `master` green; the sandbox's baseline the light gates). T5a
cut in two, the order changed (the user: the content before `GameRuleset`). `h3-towns`'
`TownRuleset` (the design §2's checks, the hash's first encoding, `empty`), `BuildingId` and
`BuildingSet`; `h3-pack-towns`' parser and format; the file's shape changed where built (the
log). Pull request: the one of branch `town/e09-t5a1`.
