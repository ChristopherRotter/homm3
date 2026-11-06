# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1:
  towns on the map**, built in rounds (the epic's task list; each task's record in `E09-log.md`;
  decisions 8–30, `TWN-Q2` to `TWN-Q6`). Rounds 2 to 8 done: towns imported and built, entered,
  captured undefended or besieged (against the garrison or a visitor with the garrison merged).
  Waiting elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's
  inbox), the guild's spells at the start (`TWN-STUB-6`, increment 4). **Round 9:** T8c1, T8c2
  done (the income, the observation's towns, a Rogue's Visions). **T9** cut in three (the user,
  2026-11-06; decision 32): **T9a** done (the writer's town objects, the town map, its replays).
  **T9b** cut in two (the user, 2026-11-06; decision 33): **T9b1** done (the plans in `play.rs`
  lay out and play towns: `TownPlan`, `Step::Town`, `check_towns`, the entrances' invariant,
  `check_town_battle_end`, the cover test `the_plans_capture_and_besiege_towns`; `view.rs` draws
  plans without towns). **Next: T9b2** (the view properties with towns; its queue entry, its cut
  confirmed at its start), then T9c (the benchmarks, the env, the census), then round 10.
- **Open pull requests:** #758, T9b1's, branch `town/e09-t9b1` (`cross-lane`: `h3-map`'s
  `tests/` and `Cargo.toml`, `proptest-regressions/tests/play.txt`).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 3 (T8c2's Visions; v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1 none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1), decisions next 34, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 11) and quirk candidates `T-<n>` (next 8).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-06 — T9b1: the plans play towns

Baseline bdbd6c6 (#756, #757 merged; CI green on 3b18baa, the same engine code; the setup's
baseline lost to a VM restart). At the start (the user, asked): T9b cut in two, the plans first.
No original data, no version moved, no engine code changed; the properties stressed once with
400 cases and the invariants with town-heavy plans, then set back. The lane's `TRAPS.md` has the
plans' two traps.

### 2026-11-06 — T9a: the writer's town objects, the town map, its replays

Baseline 2f7b9c8 (#752, #753 merged; CI green there; the setup's `verify-once` found CI's
fingerprint; `maps.tar`, `extracted.tar` and `Objects.txt` staged with the file tool, the user's
computer's shell unavailable). At the start (the user, asked): T9 cut in three. The random
replays' seed search and the scripted replays by two narrow sub-agents; no version moved; no
engine code changed.

### 2026-11-06 — T8c2: the Rogue's Visions, the security doc, the inbox notes

Baseline a425d00 (#750 merged; CI green there; the setup's baseline lost to a VM restart; the
original data staged with the file tool, the user's computer's shell unavailable). At the start
(the user, asked): T8c2 kept whole, `h3_heroes::power` and the 47 replays' header in its
`cross-lane` PR. Town part v3; the replays' town hash line only. `day-start-xl-144` re-baselined: +1.6 % against `master` built here, in `_int_malloc` alone,
every counted function the same (the lane's `TRAPS.md`). An independent
review found no bug or leak; its test gaps (the entrance against the anchor, a neutral town) and
docs fixed.
