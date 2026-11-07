# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** on 2026-11-06 by T10's audit (`E09-log.md`); its scope, decisions
  8–35 and tasks in `E09-archive.md`, its design note kept in `E09-design.md` §1–§22 (decision
  40). **Increment 2: construction with the town events** (`TWN-Q7`; decisions 36–40): T11 done
  (`ADV-STUB-13` lifted), the survey T12 done (T12a, T12b; `TWN-Q8`, decisions 41–44), **T13 done**
  (`TWN-Q9` before the drafts, decisions 45–47: the build a town command, the Shipyard's water now,
  the views as the original's; the drafts and the design note's Part D §23–§36 accepted at
  `TWN-Q10`, decisions 48–51: the cut, the exit criteria in the epic, the agent's calls, T-9
  fixed). **T14** (batch TWN-2) **done** (survey `E09-construction.md` §13: decisions 41 and 42
  stand). **T15 done** (`shared/`, docs only: the drafts in ADR-0028 points 2, 5, 6, 9, ADR-0016
  points 1, 4, 6, ADR-0017 points 1 to 4, 6 to 10; `docs/03`'s terms, `docs/06`'s section "A
  town's construction"; the drafts deleted). **Next: T16a** (the part's requirements, costs,
  role `shipyard`, part v4; design §24; queued), then T16b, T17, T18a–c, T19, T20 (design §35). Waiting elsewhere: a walled
  Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's inbox), the guild's spells at
  the start (`TWN-STUB-6`, increment 4).
- **Open pull requests:** T15's, branch `claude/zealous-euler-msupe1` (`cross-lane`: the shared
  docs with the town lane's; it merges before T16a).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a to T15 none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 11), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7, planned for T-9, design §34), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10; 10 to 12 planned by design §34, registered by their tasks), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 52, ADRs: 0028 is the lane's (accepted, T4a; increment 2's amendments applied by T15; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 3; TWN-2 scored by T14c, survey `E09-construction.md` §13; TWN-1 survey `E09-towns.md` §15), the survey's disputes `TD-<n>` (next 13; TD-11 by T12a, TD-12 by T12b) and quirk candidates `T-<n>` (next 10; T-8 by T12b, T-9 by T13; T14c raised none).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-07 — T15: increment 2's drafts into `docs/adr/`

Baseline c22c25c (#786 merged). Kept whole (the user, asked). The user's computer was not
connected (no folder, no file tool): not needed, docs only. The session's given branch
`claude/zealous-euler-msupe1` (not `shared/e09-t15`), `cross-lane` as T4a. The three drafts
folded in point by point (a script checked every sentence), the ADR index, `docs/03`'s terms,
`docs/06`'s construction section, the design note's and the adventure inbox's pointers; T16a
queued.

### 2026-11-07 — T14c: batch TWN-2 scored

Baseline 74c42c4 (#783, #784, #785 merged). Kept whole (the user, asked; a cut in two
recommended). The user's computer's shell did not start: a sandbox cloned from GitHub, the
original data staged with the file tool at the start. Docs only: survey §13 (TD-9 as read; TD-11
refused by the build dialog's owner test, 0x5D5AE2, which T12a had not read; the stray bit is the
town screen's display piece, `town_screen_pieces` 0x5C33F0, nothing for the engine), design §33,
the drafts' lines, the experiments README's row, the exe map, a trap; T15 queued.

### 2026-11-07 — T14a, T14b: batch TWN-2's spec, the hall path, two runs

Baseline 2efcb26 (#782 merged; CI green on efaae07, the same code; the setup's baseline lost to a
VM restart). T14 cut in three (the user, asked; recommended). The user's computer's shell did not
start: a sandbox cloned from GitHub, the original data staged with the file tool at the start.
The spec (two changes found by the probe: a hero for Blue, Tan as an enemy), the harness's hall
path learned live under Wine with a throwaway probe, the recipe, two runs recorded; the harness
check passed; the client, map-client and adventure lanes' inboxes told. Not scored (T14c).
