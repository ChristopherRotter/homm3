# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1:
  towns on the map**, built in rounds (the epic's task list; each task's record in `E09-log.md`;
  decisions 8–35, `TWN-Q2` to `TWN-Q6`). Rounds 2 to 9 done: towns imported and built, entered,
  captured undefended or besieged, paying their income, seen by each viewer; the town map, its
  replays, the plans' and the views' properties with towns; **T9c** (kept whole, the user,
  2026-11-06; decision 35): the benchmarks on the town map (`siege-towns-36`, `day-start-towns-36`,
  `reach-towns-36`, `fork-towns-36`; `fork-xl-144` re-set), the env's test, the bot reports
  unchanged, the census: all 160 shipped maps import and build (exit criterion 1). Waiting
  elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's inbox), the
  guild's spells at the start (`TWN-STUB-6`, increment 4). **Next: T10** (round 10, the audit
  against the exit criteria; increment 1 closed; increment 2 refined), once T9c's pull request
  merges.
- **Open pull requests:** T9c's, branch `town/e09-t9c` (`cross-lane`: `benches/adventure/`,
  `h3-cli`'s `src/adventure/`, `h3-map`'s census example, `h3-sim`'s tests).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 36, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 11) and quirk candidates `T-<n>` (next 8).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-06 — T9c: the benchmarks, the env and the census on the town map

Baseline cf8325e (#764 merged; CI green there; `verify-once` found CI's fingerprint). At the
start (the user, asked): kept whole. The user's computer's shell did not start: a sandbox cloned
from GitHub, `maps.tar` and `extracted.tar` staged with the file tool. No engine code changed, no
version moved; the bot reports regenerate byte-identical. A siege's step is 80 % the battle's
start (`closest_free` 46 %); the town map's reach query 2.7 times an XL map's (its heroes reach
the whole map; the entrance test under 0.3 %).

### 2026-11-06 — T9b2b: the hygiene property meets the towns

Baseline 0178a2c (#761 merged; CI green there; the setup's full local verify green). The user's
computer's shell did not start: a sandbox cloned from GitHub, no original data needed. Kept whole
(the agent's call, told). No version moved, no engine code changed. The first stress run found a
gap of `master`'s (a Shipyard's boat tile moved by an unseen terrain change): `vary` keeps the
offsets, the seed committed; the lane's `TRAPS.md` has the trap.

### 2026-11-06 — T9b2a: the told views meet the towns

Baseline 1b1792b (#758 merged; CI green there; `verify-once` found CI's fingerprint). At the
start (the user, asked): T9b2 cut in two, the told views first, the hygiene property's calls told
with the question. No original data, no version moved, no engine code changed; the plain
property met few captures, so a cover test over town-heavy plans; three engine mutations caught.
A random seed failed `master`'s hygiene property (another player's guards left at a seen mine):
fixed in `outcome`, the seed committed.
