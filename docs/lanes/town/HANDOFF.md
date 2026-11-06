# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** on 2026-11-06 by T10's audit (all seven exit criteria hold;
  `E09-log.md`); its scope, decisions 8–35 and tasks in `E09-archive.md`, its design note kept in
  `E09-design.md` §1–§22 (decision 40). **Increment 2: construction with the town events**
  (`TWN-Q7`; decisions 36–40): the build command, requirements and costs, one a town a day, the
  forbidden buildings, the town events' buildings and resources. **T11 done** (`ADV-STUB-13`
  lifted: the random dwellings tied to a town take its faction; map import v20). **Next: T12**
  (the construction survey and its checkpoint), then T13 (the drafts, with ADR-0016 point 4's
  sentence on the tied dwellings; the design note's Part D from §23; the cut). Waiting elsewhere: a walled
  Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's inbox), the guild's spells at
  the start (`TWN-STUB-6`, increment 4).
- **Open pull requests:** T11's, branch `town/e09-t11` (`cross-lane`).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 8), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 41, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 11) and quirk candidates `T-<n>` (next 8).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-06 — T11: the random dwellings tied to a town

Baseline e5a5840 (#769 to #771 merged; CI's fingerprint on `master`). Kept whole (the user,
asked). The user's computer's shell did not start: a sandbox cloned from GitHub, `maps.tar` and
`extracted.tar` staged. The planned `GameRuleset` check became a setup check
(`GameSetupError::LinkedTown`): the `synthetic` pack has a town record without tables. The census
diffed with `master`'s: all 160 build, the 1 397 resolved by their town's faction.

### 2026-11-06 — T10: the audit; increment 1 closed; increment 2 refined

Baseline 798b2e2 (#767, #768 merged; CI green there; `verify-once` found CI's fingerprint). At
the start (the user, asked with a first look at the audit, `TWN-Q7`): T10 whole, increment 2 as
construction with the town events, a survey then the drafts, `ADV-STUB-13` first as T11. The
user's computer's shell did not start: a sandbox cloned from GitHub, docs only. Five rules
observed by batch TWN-1 now `verified`; increment 1's design note stays in place (85 comment
lines cite it).

### 2026-11-06 — T9c: the benchmarks, the env and the census on the town map

Baseline cf8325e (#764 merged; CI green there; `verify-once` found CI's fingerprint). At the
start (the user, asked): kept whole. The user's computer's shell did not start: a sandbox cloned
from GitHub, `maps.tar` and `extracted.tar` staged with the file tool. No engine code changed, no
version moved; the bot reports regenerate byte-identical. A siege's step is 80 % the battle's
start (`closest_free` 46 %); the town map's reach query 2.7 times an XL map's (its heroes reach
the whole map; the entrance test under 0.3 %).
