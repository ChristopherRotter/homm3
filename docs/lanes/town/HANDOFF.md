# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7); the lane opened
  the same day beside the adventure lane's E08. **Increment 1: towns on the map**, built in rounds
  (the epic's task list; each task's record in `E09-log.md`): rounds 2 to 6 done (batch TWN-1, the
  ADRs and skeletons, the part, the content harvested, `GameRuleset` with the town part, `h3-towns`'
  record, store, derivations and setup; decisions 8–30, `TWN-Q2` to `TWN-Q6`). **Round 7**: T7a
  cut in two (the user); **T7a1** done (#710: the setup's fields and the import, map import
  v17); **T7a2** done (#713: `validate`'s checks of the towns and the starts, the offer). T7b cut in two at the towns (the user, 2026-11-04:
  their owners and objects come with T8a): **T7b1** done (the hero pool, the players' random
  choices a start reads, the random heroes' types; game state hash v23; 31 shipped maps build).
  Until T7b2 the build refuses a main town that generates a hero (`TWN-STUB-3`). **Next: T7b2**
  (the towns' draws and the store, the queue) after T8a's town list and owners: round 8's order
  (T8a, possibly cut so its kind, list and owners come first, then T7b2) settled at its start;
  rounds 8 to 10 in the epic's list.
- **Open pull requests:** the one of branch `town/e09-t7b1` (T7b1, `cross-lane`: `h3-game`'s
  `setup.rs`, `state.rs`, `state/battle.rs`, `lib.rs` and tests, `h3-map`'s census example and a
  test, the 39 adventure replays, the adventure lane's HANDOFF versions line).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 2 (T5b2a's `auto_with`, pinned in its test). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1; next 6), stubs `TWN-STUB-<n>` (2 registered by T6c, 3 by T7a1, narrowed by T7b1, lifted by T7b2; 1 reserved, E10's; next 4), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1), decisions next 31, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 10) and quirk candidates `T-<n>` (next 7).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-04 — T7b1: the pool and the heroes' draws

Baseline e654837 (#713 merged; CI green there; the sandbox's baseline the light gates; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`). At the start (the user): T7b cut in two at the towns (T7b1 now, T7b2 after
T8a's town list and owners). The code, bench, replays and census by one sub-agent, the review and
docs by the agent. `HeroPool` (TWN-SET-005, -007; `TWN-QUIRK-2`, `-3`), `NoFreeHero`, game state
hash v23 (39 replays rehashed), forks +0.3 to +0.4 %; the census: 31 maps build (9 before), 125
refused for a main town's hero. Pull request: the one of branch `town/e09-t7b1`.

### 2026-11-04 — T7a2: the setup's checks

Baseline 0387933 (#710 merged; CI green there; the sandbox's baseline the light gates; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`).
At the start (the user): `TownCount` and `MainTown`'s owner to T8a; `TownAlignment` refuses a
colour no player has (no import change). The code by one sub-agent in a worktree, the review and
docs by the agent. `check_starts` (seven errors), `GameSetup::offer`; TWN-SET-002, -006, -008;
the census unchanged. A random case of `h3-map`'s view property test fails on `master`'s import
(`OwnedOnWater`; the adventure lane's inbox). Pull request: the one of branch `town/e09-t7a2`.

### 2026-11-04 — T7a1: the setup's new fields and the import

Baseline 03b6601 (#707 merged; CI green there; the sandbox's baseline lost to a VM restart; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`). At the start (the user): T7a cut in two (T7a1 now, T7a2 `validate`'s checks
next); until T7b the build refuses a setup that needs a draw (`TWN-STUB-3`). The code by one
sub-agent (resumed after a second VM restart), the review and docs by the agent; #708 merged in.
`PlayerStart`, `GameSetup`'s towns and pool inputs, the import (TWN-SET-001's half, -010;
`TWN-QUIRK-1`, `-4`), map import v17 (T93b, #711, took v16 first; `master` merged in), the
census (156 maps import, 4 refused by T93b's dwellings on water, 9 build). Pull request: the one
of branch `town/e09-t7a1`.
