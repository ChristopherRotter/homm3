# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7); the lane opened
  the same day beside the adventure lane's E08. **Increment 1: towns on the map**, built in rounds
  (the epic's task list; each task's record in `E09-log.md`): rounds 2 to 7's T7a and T7b done
  (decisions 8–30, `TWN-Q2` to `TWN-Q6`). **Round 8** (the user, 2026-11-04): the order T8a1,
  T7b2, T8a2, then T8b (cut in two at its start, the user, 2026-11-05: T8b1, T8b2). **T8a1**,
  **T7b2a**, **T7b2b**, **T8a2** and **T8b1** done: a town's entrance is entered by its owner's
  heroes, an undefended town of another player or none is captured, a town may reveal the whole
  map; a siege's battle is built from its town and its end applied (the capture after an
  attacker's win, the attacker outside), but no step starts one yet: a defended town and another
  team's visitor are refused (`TWN-STUB-7`, T8b2's). A walled Stronghold's natural obstacles wait
  for the battle lane (`TWN-STUB-8`, its inbox); the guild's spells at the start for increment 4
  (`TWN-STUB-6`, TD-10). **Next: T8b2** (the step onto a defended town; its queue entry), once
  #737 merges; then rounds 9, 10 in the epic's list.
- **Open pull requests:** #737, branch `town/e09-t8b1` (T8b1, `cross-lane`: `h3-adventure`'s
  `command.rs`, `state.rs`, `state/piles.rs` and tests; `h3-game`'s `state/battle.rs`,
  `state/rests.rs` and tests; `game.toml`; the battle lane's `QUEUE.md` inbox).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 1 (its first encoding, T6a), the part's 2 (T5b2a's `auto_with`, pinned in its test). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1 none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 7), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, T8b2's; 8 by T8b1, the battle lane's; 1 reserved, E10's; next 9), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1), decisions next 31, ADRs: 0028 is the lane's (accepted, T4a; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 2; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 11) and quirk candidates `T-<n>` (next 8).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-05 — T8b1: the siege's battle

Baseline 9d65268 (#733 merged; CI green there, the sandbox's light gates; the original data
staged as before). At the start (the user): T8b cut in two at the battle (T8b1 now: the setup
and the end, started directly in the tests; T8b2 the step, the reach with the bench, the merge).
No version moved; no hot path; no replay touched. Two calls against the design note, both by
the code: the special ground stays in a siege; the attacker's +500 (TWN-SIE-006) added. A gap
for the battle lane: a walled Stronghold's natural obstacles (`TWN-STUB-8`).

### 2026-11-05 — T8a2: the step onto a town and an undefended capture

Baseline a415b1d (#727 merged; the sandbox's light gates there; the original data staged as
before). At the start: the agent proposed a cut in two (entering, then the capture); the user
chose T8a2 whole. The adventure side with the bench and the game's composition with the gates by
two sub-agents in turn, an independent review, the docs by the agent; `master` (#728 to #730)
merged in. No version moved; no replay touched (no fixture map has a town); the reach's loop kept
`master`'s (a first cut cost +7.7 %; the adventure lane's trap); `TWN-STUB-7` until the siege.

### 2026-11-05 — T7b2b: the main towns' heroes

Baseline f720585 (#724 merged; the sandbox's light gates on 1c906d6; the original data staged as
before, and `h3maped.exe` with its help file; the user's computer's shell unavailable, staging
worked). At the start (the user): T7b2b whole; asked whether a hero right of a town's entrance is
the garrison hero: read again (the executable, VCMI 1.7.5), the visitor (survey §4.6). Two
readings and the code, census and gates by sub-agents, the review and docs by the agent. No
version moved; all 156 imported maps build (523 main towns' heroes); `TWN-QUIRK-6`,
`TWN-STUB-6`; no replay (the shipped maps are not committed); the bench inside its gate.
