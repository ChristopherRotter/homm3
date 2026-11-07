# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** on 2026-11-06 by T10's audit (`E09-log.md`); its scope, decisions
  8–35 and tasks in `E09-archive.md`, its design note kept in `E09-design.md` §1–§22 (decision
  40). **Increment 2: construction with the town events** (`TWN-Q7`; decisions 36–40): T11 done
  (`ADV-STUB-13` lifted), the survey T12 done (T12a, T12b; `TWN-Q8`, decisions 41–44), **T13 done**
  (`TWN-Q9` before the drafts, decisions 45–47: the build a town command, the Shipyard's water now,
  the views as the original's; the drafts `adr-drafts/*-increment-2.md` and the design note's
  Part D §23–§36 accepted at `TWN-Q10`, decisions 48–51: the cut, the exit criteria in the epic,
  the agent's calls, T-9 fixed). **T14** (batch TWN-2, design §33) cut in three (the user): T14a
  (the spec) and T14b (the harness's hall path, two runs) **done**; **Next: T14c** (the scoring:
  TD-9, TD-11, the visit's stray standing bit; docs only), then T15 `shared/` (the drafts into
  `docs/adr/`), T16a/b, T17, T18a–c, T19, T20 (design §35). Waiting elsewhere: a walled
  Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's inbox), the guild's spells at
  the start (`TWN-STUB-6`, increment 4).
- **Open pull requests:** T14a's and T14b's, branch `town/e09-t14` (`cross-lane`: the experiments
  README's row; the capture harness is common).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a, T12b none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 11), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7, planned for T-9, design §34), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10; 10 to 12 planned by design §34, registered by their tasks), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 52, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 3; TWN-2's two rows design §33, decisions 41, 42, run by T14b, `experiments/results/E09-batch-2a*.md`; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 13; TD-11 by T12a, TD-12 by T12b) and quirk candidates `T-<n>` (next 10; T-8 by T12b, T-9 by T13).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-07 — T14a, T14b: batch TWN-2's spec, the hall path, two runs

Baseline 2efcb26 (#782 merged; CI green on efaae07, the same code; the setup's baseline lost to a
VM restart). T14 cut in three (the user, asked; recommended). The user's computer's shell did not
start: a sandbox cloned from GitHub, the original data staged with the file tool at the start.
The spec (two changes found by the probe: a hero for Blue, Tan as an enemy), the harness's hall
path learned live under Wine with a throwaway probe, the recipe, two runs recorded; the harness
check passed; the client, map-client and adventure lanes' inboxes told. Not scored (T14c).

### 2026-11-07 — T13: increment 2's drafts, the design note and the cut

Baseline 98b2c14 (#779 merged; CI green there; the setup's baseline lost to a VM restart). Kept
whole (the user, asked). The user's computer's shell did not start: a sandbox cloned from GitHub,
the original data staged with the file tool at the start. Docs only: `TWN-Q9` (three ADR-level
choices) before the drafts; the three drafts, Part D of the design note (§23–§36), the checkpoint
`TWN-Q10` (all as recommended); T14 queued; the adventure lane's inbox told of the planned bumps.

### 2026-11-07 — T12b: the construction survey's second part

Baseline d383d40 (#775, #776 merged; CI green on 708dd41; the setup's baseline lost to a VM
restart). Kept whole (the user, asked). The user's computer's shell did not start: a sandbox
cloned from GitHub, the original data staged with the file tool before the computer went offline.
Docs only: the survey §8 to §12 (the town events' reading, VCMI and the wiki, the census), the
code map, `TWN-Q8` asked and answered (all four as recommended). The census was a throwaway
example over `h3_map::read`, not committed.
