# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7); the lane opened
  the same day beside the adventure lane's E08. **Increment 1: towns on the map**, built in rounds
  (the epic's task list; each task's record in `E09-log.md`): rounds 2 to 7's T7a and T7b1 done
  (decisions 8–30, `TWN-Q2` to `TWN-Q6`). **Round 8** (the user, 2026-11-04): the order T8a1,
  T7b2, T8a2, then T8b; T7b2 cut in two at the store (the user). **T8a1** and **T7b2a** done (the
  towns resolved and stored: the random towns' factions, each town's `town_setup`, the store of
  towns in `GameState`, the garrisons and the towns' own radii into the adventure; game state
  hash v24). Until T7b2b the build refuses a main town that generates a hero (`TWN-STUB-3`); until
  T8a2 a town's entrance is not entered (`TWN-STUB-4`) and a town that reveals every tile reveals
  only its level at the start (`TWN-STUB-5`, narrowed). **Next: T7b2b** (the main towns' heroes;
  its queue entry), once T7b2a's pull request merges; then T8a2, T8b; rounds 9, 10 in the epic's
  list.
- **Open pull requests:** the one of branch `town/e09-t7b2a` (T7b2a, `cross-lane`: `h3-game`,
  `h3-map`'s census example, `h3-cli`'s `src/ai/census.rs`, two adventure bench workloads, the 39
  adventure replays, the adventure lane's HANDOFF versions line and inbox, the battle lane's inbox).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 2 (T5b2a's `auto_with`, pinned in its test). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1; next 6), stubs `TWN-STUB-<n>` (2 registered by T6c, 3 by T7a1, narrowed by T7b1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 1 reserved, E10's; next 6), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1), decisions next 31, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 10) and quirk candidates `T-<n>` (next 7).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-04 — T7b2a: the towns resolved and stored

Baseline ae59f3b (#719 merged; CI green there; the sandbox's baseline the light gates; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`; the user's computer's shell unavailable, staging worked). At the start (the
user): T7b2 cut in two at the store (T7b2a now, T7b2b the main towns' heroes). The code, census,
bench and replays by one sub-agent, the review and docs by the agent; `master` (#720 to #722)
merged in before the pull request. Game state hash v24 (the adventure lane's T95b now takes v25,
its inbox); the census unchanged (31 build, 294 towns); forks +0.24 to +0.36 % against `master`,
two re-baselined; two random property cases independent of this task to the battle and
adventure inboxes.

### 2026-11-04 — T8a1: the town as a map object

Baseline 5dac0d2 (#715 merged; CI green there; the sandbox's baseline the light gates; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`; the user's computer's shell unavailable, staging worked). At the start (the
user): T8a in two, round 8's order T8a1, T7b2, T8a2; the branch stacked on #716, which was behind
`master` (41 files in conflict, the replays' hashes): `master` merged into the base in its own
commit; once #716 merged, the branch rebased onto `master` (the engine's code the same, the
replays kept). The code, census, bench and replays by one sub-agent, the review and docs by the agent.
Part v17, map import v18, adventure state hash v21; the census unchanged (31 build); the bench
inside its gate (forks +1.1 to +1.7 % on the stored baselines, the highest on maps without a town).

### 2026-11-04 — T7b1: the pool and the heroes' draws

Baseline e654837 (#713 merged; CI green there; the sandbox's baseline the light gates; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`). At the start (the user): T7b cut in two at the towns (T7b1 now, T7b2 after
T8a's town list and owners). The code, bench, replays and census by one sub-agent, the review and
docs by the agent. `HeroPool` (TWN-SET-005, -007; `TWN-QUIRK-2`, `-3`), `NoFreeHero`, game state
hash v23 (39 replays rehashed), forks +0.3 to +0.4 %; the census: 31 maps build (9 before), 125
refused for a main town's hero. Pull request: the one of branch `town/e09-t7b1`.
