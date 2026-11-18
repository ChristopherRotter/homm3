# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–150, `TWN-Q15` to `-Q25`): T31 to T38 done;
  **T39a done whole** (the map's Tavern and Den of Thieves, the tavern decision GAM-DEC-010;
  `TWN-Q25`, decisions 149, 150). **Next:** T39b (the Thieves' Guild with the Den's visit,
  lifting `TWN-STUB-20`; `QUEUE.md`), its cut confirmed at its start. The guild and the
  services are increment 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the battle
  lane's inbox), `TWN-STUB-6`, `-13` and `-15` (increment 5), `TWN-STUB-2` (the victory
  conditions), `TWN-STUB-16` (E10's loss condition).
- **Open pull requests:** T39a's `town/e09-t39a` (`cross-lane`), opened at the session's end (its
  number in the newest session entry); watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v7: T36), both in `h3-towns`; the adventure lane's
  (game state hash v34, the adventure part v22, map import v26 and replay format v19, all by
  T39a; adventure state hash v26 by E08's T124b1), the env's (v13 by T39a) and the protocol's
  (v73 by T39a: T39b takes v74) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q26`; decisions 151; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b; `-12`'s test by T38b1, `-13`'s with T39b); stubs `TWN-STUB-21` (`-11`, `-12` planned by
  §34; `-16` by T38b2, `-20` by T39a; `-17`, `-18` by §71, §77); rules: TWN-TAV-001 to -006,
  GAM-MAN-003, GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built, TWN-TAV-007 to -009
  planned; experiment batch `TWN-5`; disputes `TD-25`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-18 — T39a: the map's Tavern and Den of Thieves, the tavern decision

Baseline 34dddd8 (#1018 merged; CI's fingerprint on `master`, the setup ran the light gates).
The user's computer was connected at the start only: 28 files staged (the four LODs, both
executables, the DLLs, both `.snd`, two `.vid`, both tars, the saves, the manuals, two help
files; SHA-256 checked). The agent recommended the Tavern alone (the Den to T39b); the user kept
T39a whole as queued and consented to the `cross-lane` PR (decision 149). Built: design §66's
and §74's As built (the two kinds, part v22, import v26; `Decision::Tavern`, game state hash v34;
`map_hire` through the town hire's `place`; protocol v73, env v13, replay format v19; the 98
goldens and the bots tables re-recorded); every workload inside its gate. In progress: the
scoped verify, then the PR.

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
