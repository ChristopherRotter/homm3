# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–148, `TWN-Q15` to `-Q24`): T31 to T38 done;
  **T38b2 done whole** (the dismissal, `TWN-Q24`, decisions 147, 148). **Next:** T39a (the map's
  Tavern and Den of Thieves, the tavern decision, `answer tavern`; `QUEUE.md`), its cut
  confirmed at its start. The guild and the services are increment 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6`, `-13` and `-15`
  (increment 5), `TWN-STUB-2` (the victory conditions), `TWN-STUB-16` (E10's loss condition).
- **Open pull requests:** #1018 (T38b2's `town/e09-t38b2`, `cross-lane`), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v7: T36), both in `h3-towns`; the adventure lane's
  (game state hash v33 by T37a, adventure state hash v26 by E08's T124b1, map import v25 by
  T37b: T39a takes v26, replay format v18 by T38b2: T39a takes v19), the env's (v12 by T38a2;
  T39a takes v13) and the protocol's (v72 by T38b2: T39a takes v73) are bumped `cross-lane` (each task's record in
  `E09-log.md`; the plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q25`; decisions 149; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b; `-12`'s test by T38b1, `-13`'s with T39b); stubs `TWN-STUB-20` (`-11`, `-12` planned by §34; `-16`
  registered by T38b2, `-17`, `-18` by §71, §77); rules: TWN-TAV-001 to -006 and GAM-MAN-003
  built, TWN-TAV-007 to -009, GAM-DEC-010 planned; experiment batch `TWN-5`; disputes `TD-25`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-18 — T38b2: the dismissal

Baseline 8625fed (#1013, #1014 merged; CI's fingerprint on `master`, the setup ran the light
gates). The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). The agent
recommended a cut in two as T38a's; the user kept T38b2 whole (decision 147). Built: design §69's
As built (`Manage::Dismiss` with `NotOnMap`, `InTown`, `LastHero`; `town::leave::dismiss` and
`check_dismissal`; the adventure's `remove_hero`; the listing; replay format v18 `dismiss
<hero>`, protocol v72 with a fold test; the plans dismiss in one `Step::Manage` of sixteen, three
oracles taught); GAM-MAN-003 `verified`, `TWN-STUB-16`; every workload inside its gate.

### 2026-11-18 — T38b1: the leaves and the returns

Baseline d5e5742 (#1006 merged; CI's fingerprint on `master`, the setup ran the light gates).
The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). The user cut
T38b in two (decision 145). Built: design §67's As built (`h3-game`'s `town/leave.rs`, the
adventure's `end_battle` and `release_town_hero` returning the heroes that left, the pool's
`leave` and `return_to`, §67 point 5 in the replacement); TWN-TAV-006 `verified`; the plans'
battle oracle taught the untold refill; 33 goldens re-recorded (hash lines only); the town-map
seed 2541; four town-map workloads re-baselined (the games changed, the same code). No version
moved.

### 2026-11-18 — T38a2: the hire listed, the env's v12, the plans hire

Baseline 52a3cb5 (#1001 merged; CI's fingerprint on `master`, the setup ran the light gates).
The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). The agent
recommended a cut in two (the listing with the plans, then the env); the user kept T38a2 whole
(decision 143). Built: design §76's As built (the listing after the recruitments, the env's
`TAVERN_HIRE + s`, the plans' probe and hire step, four oracles taught a hire); the town-map
seeds searched again (2369, 6); the listing folded into a sized list after the first bench
(+17 % on the same games to +4.6 %), then the town workloads re-baselined. No protocol, replay
or hash version moved; `TWN-STUB-19` lifted.
