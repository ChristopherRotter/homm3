# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7); the lane opened
  the same day beside the adventure lane's E08. **Increment 1: towns on the map**, built in rounds
  (the epic's task list; each task's record in `E09-log.md`): rounds 2 to 5 done (batch TWN-1, the
  ADRs and skeletons, the part, the content harvested, `GameRuleset` with the town part; decisions
  8–30, `TWN-Q2` to `TWN-Q6`). **Round 6:** T6 cut in three at its start (the user): **T6a** and
  **T6b done** (the record and the store; the derivations, `town.toml` with eight rules); next
  **T6c** (the setup's functions, in the queue), once T6b's pull request merges; rounds 7 to 10
  in the epic's list.
- **Open pull requests:** T6b's (`town/e09-t6b`); see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 2 (T5b2a's `auto_with`, pinned in its test). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (next 1; 1 to 5 reserved by the design §19 for T-1 to T-4 and T-6), stubs `TWN-STUB-<n>` (next 1; 1 and 2 reserved by the design §19), rules `TWN-*` (the design §19 names them; in `town.toml` since T6b: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002), decisions next 31, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 10) and quirk candidates `T-<n>` (next 7).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-03 — T6b: the derivations and `town.toml`

Baseline 4a46251 (#698 merged; the sandbox's full engine run as the baseline; the original data
not staged, not read). At the start (the user): T6b whole, split between the agent and one
sub-agent in its own worktree, merged into one branch. The common reading of a town, the income,
the siege's facts with the named bonuses (and the compiler's bound of 8), the reveal, the Tavern,
the Capitol's limit with the store's first write; `town.toml` created. Pull request: the one of
branch `town/e09-t6b`.

### 2026-11-03 — T6a: the town's record and the store

Baseline 9b07bea (#694 merged; CI on `master` green there; the sandbox's baseline the light
gates; the original data not staged, not read). At the start (the user): T6 cut in three (T6a
the record and the store, T6b the derivations, T6c the setup). `Town` with the standing set
derived, `TownStatic` and `TownEvent`, `Towns` (`Option<Arc<…>>`) and town state hash v1's first
encoding; `set_built` and `capitol` moved to T6b. Pull request: the one of branch `town/e09-t6a`.

### 2026-11-03 — T5a2: `GameRuleset` takes the town part

Baseline 2030ef5 (#689 merged; the sandbox's full engine verify green there; CI's run failed one
`h3-heroes` property case, fixed on `master` by 1981203 and merged in; the original data staged).
At the start (the user): the task whole. `GameRuleset::new` with the town part and its two new
refusals, the callers, the CLI's loader reading `towns.toml`, adventure replay format v9 and the
39 replays' headers. Pull request: #694 (`town/e09-t5a2`, `cross-lane`).
