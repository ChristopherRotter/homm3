# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). **T1 done** (the survey `E09-towns.md` §0 to §14; `TWN-Q2`, decisions 8–16). **T2 done**: T2a the ADR drafts in `adr-drafts/` (`TWN-Q3`, decisions 17–20); T2b the design note `E09-design.md` §1 to §22 and the cut (`TWN-Q4`, decisions 21–24: the drafts accepted as written, the cut as proposed, the content read and harvested, the agent's calls kept). **T3a done** (round 2): `fort` and an explicitly empty `garrison` for a spec's towns (`h3-experiment`, `cross-lane`), the spec `experiments/specs/E09-batch-1.toml` and its template. **T3b done** (round 3): the capture harness's town reads, a new-game step and the town screen; batch TWN-1 played twice under Wine (`experiments/results/E09-batch-1.md`, `-run2.md`). **T3c done** (survey §15): TD-3, TD-5, TD-7 and the siege's aftermath as the reading; building 23 in every Rampart, Necropolis and Conflux town of the standard set, not predicted, re-read in T5b (`TWN-Q5`, decisions 25, 26). **T4a done** (round 4's first half; T4 cut in two at its start, the user): ADR-0028 and E09's amendments of ADR-0015, -0016, -0017, -0023 in `docs/adr/`, the glossary. Next: **T4b** (`shared/`: the skeletons `h3-towns` and `h3-pack-towns`, `TownId`, the gates; the queue); rounds 5 to 10 in the epic's list.
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; it, the adapter `h3-pack-towns` and `packs/*/towns.toml` come with the `shared/` task T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 6), quirks `TWN-QUIRK-<n>` (next 1; 1 to 4 reserved by the design §19 for T-1 to T-4), stubs `TWN-STUB-<n>` (next 1; 1 and 2 reserved by the design §19), rules `TWN-*` (none yet; the design §19 names them), decisions next 27, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 1; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 8) and quirk candidates `T-<n>` (next 6).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-03 — T4a: the ADRs and the glossary

Baseline 971da9e (#666 merged; the sandbox's full engine run started at setup). The user cut T4 in
two (the ADRs first). ADR-0028 accepted into `docs/adr/`; the amendments folded into ADR-0015,
-0016, -0017 and -0023 with batch TWN-1's verdicts (no point changed); the index, the glossary,
`AGENTS.md`'s town list; the battle lane's inbox (`siege::setup`'s comment); the drafts deleted.
Docs only. Pull request: the one of branch `shared/e09-t4a` (`cross-lane`).

### 2026-11-03 — T3c: batch TWN-1 scored

Baseline dd99747 (#662 merged). Docs only, no game files (VCMI 1.7.5 cloned to scratch). Survey
§15: rows 2 to 4 and row 1's dwellings as the reading, VCMI refuted in each (21 of 96 level-2
dwellings). Building 23 in Rampart's, Necropolis' and Conflux's standard sets: not the reading's,
not VCMI's; the keep may count it. `TWN-Q5`: re-read in T5b; the experiments README's row
`cross-lane` (decisions 25, 26). The queue: T4 written. Pull request: the one of branch
`claude/peaceful-curie-q7wjex` (`cross-lane`).

### 2026-11-03 — T3b: batch TWN-1's harness and runs

Baseline e02c8e5 (#659 merged; CI on `master` green; the sandbox's full engine run green). The
user kept T3b whole with narrow sub-agents; 3b in a new game instead of a save. The harness's
town reads, `new_game` steps, the town screen (vtable 0x64372C) and two fixes (`clients/capture/`,
common; three lanes' inboxes told); a town's visitor reads −1 while its hero is selected (a probe).
Two whole runs recorded; the merge exactly as the reading; building 23 in every Rampart,
Necropolis and Conflux town (for T3c). The VM restarted once. Pull request: #662
(`town/e09-t3b`, merged).
