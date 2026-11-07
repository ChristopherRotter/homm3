# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** on 2026-11-06 by T10's audit (all seven exit criteria hold;
  `E09-log.md`); its scope, decisions 8–35 and tasks in `E09-archive.md`, its design note kept in
  `E09-design.md` §1–§22 (decision 40). **Increment 2: construction with the town events**
  (`TWN-Q7`; decisions 36–40): the build command, requirements and costs, one a town a day, the
  forbidden buildings, the town events' buildings and resources. **T11 done** (`ADV-STUB-13`
  lifted: the random dwellings tied to a town take its faction; map import v20). **T12** (the
  construction survey, `docs/rules/surveys/E09-construction.md`) cut in two at its start (the
  user): **T12a done** (the build command's checks, what a build does, the costs and the
  requirements; TD-9 kept, TD-11 new). **Next: T12b** (the town events, the census of the shipped
  maps, the disputes of both parts, the checkpoint `TWN-Q8`), then T13 (the drafts, with ADR-0016
  point 4's sentence on the tied dwellings; the design note's Part D from §23; the cut). Waiting
  elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's inbox), the
  guild's spells at the start (`TWN-STUB-6`, increment 4).
- **Open pull requests:** T12a's, branch `town/e09-t12a` (docs only).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 8), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 41, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 12; TD-11 by T12a) and quirk candidates `T-<n>` (next 8).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-07 — T12b: the construction survey's second part (in progress)

Baseline d383d40 (#775, #776 merged; CI green on 708dd41; the setup's baseline lost to a VM
restart). Kept whole (the user, asked). Branch `town/e09-t12b`, docs only. **Done:** the survey
§8 to §11 (the town events' reading, VCMI and the wiki, the census, the disputes TD-9, TD-11, the
new TD-12 and T-8), the code map, `TWN-Q8` in `OPEN-QUESTIONS.md`. **Next:** the checkpoint's
answers (the survey's §12, the epic's decisions from 41), the bookkeeping, the pull request.

### 2026-11-07 — T12a: the construction survey's first part

Baseline cf3e701 (#771 to #773 merged; CI green there; the setup's baseline lost to a VM restart).
T12 cut in two at its start (the user, asked; recommended). The user's computer's shell did not
start: a sandbox cloned from GitHub, `Heroes3.exe`, both LODs, `maps.tar` and `extracted.tar`
staged with the file tool. Docs only: the survey §0 to §7, 34 code-map entries. VCMI 1.7.5 read
from a sparse clone of its tag; the Lazy wiki's pages are not on the fetch allowlist (found by
search only).

### 2026-11-06 — T11: the random dwellings tied to a town

Baseline e5a5840 (#769 to #771 merged; CI's fingerprint on `master`). Kept whole (the user,
asked). The user's computer's shell did not start: a sandbox cloned from GitHub, `maps.tar` and
`extracted.tar` staged. The planned `GameRuleset` check became a setup check
(`GameSetupError::LinkedTown`): the `synthetic` pack has a town record without tables. The census
diffed with `master`'s: all 160 build, the 1 397 resolved by their town's faction.
