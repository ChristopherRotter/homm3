# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–144, `TWN-Q15` to `-Q22`): T31 to T37b done;
  T38a cut in two (decision 140): T38a1 done (the town's hire, protocol v71, replay format v17),
  **T38a2 done** (whole, decision 143: the hire listed, the env's `TAVERN_HIRE` v12, the plans
  and their oracles taught a hire, the town-map seeds, the benches; decision 144).
  **Next:** T38b (a hero that leaves the game, the returns, the dismissal; `QUEUE.md`), its cut
  confirmed at its start. The guild and the services are increment 5, travel and the Grail 6.
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6`, `-13` and `-15`
  (increment 5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T38a2's `town/e09-t38a2` (`cross-lane`), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v7: T36), both in `h3-towns`; the adventure lane's
  (game state hash v33 by T37a, adventure state hash v26 by E08's T124b1, map import v25 by
  T37b: T39a takes v26, replay format v17 by T38a1), the env's (**v12 by T38a2**; T39a takes
  v13) and the protocol's (v71 by T38a1) are bumped `cross-lane` (each task's record in
  `E09-log.md`; the plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q23`; decisions 145; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b, their tests with T38b, T39b); stubs `TWN-STUB-20` (`-11`, `-12` planned by §34; `-16`
  to `-18` by §71, §77); rules: TWN-TAV-001 to -005 built, TWN-TAV-006 to -009, GAM-DEC-010,
  GAM-MAN-003 planned; experiment batch `TWN-5`; disputes `TD-25`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-17 — T38a1: the town's hire, protocol v71, replay format v17

Baseline 7a0da76 (#995 to #997 merged; CI's fingerprint on `master`, the setup ran the light
gates). The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). The user cut T38a
in two (decision 140) and chose the owner's parts for what the slot's view lacks (141); the agent
moved the replay word into T38a1 (the server journals what the wire takes) and told the user.
Built: design §64, §65, §75's As built (`h3-game`'s `town/hire.rs`, the adventure's
`src/town/hire.rs`, `h3_towns::check_hire`); the wire's fold of a hire, tested on three golden
games; the 89 goldens' format line. #999 the `shared/` half (merged), #1001 the code.

### 2026-11-17 — T37b: the map's customised types, map import v25

Baseline 8944673 (#991 merged; CI's fingerprint on `master`, the setup ran the light gates). The
user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). A census first
(`Marshland Menace`'s Fortress-only player 5 owns its random hero, its main town generates one)
and a short code reading (`load_hero_body`'s field-by-field writes): the user chose every hero of
a customised type and the records at the build (decisions 137, 138; VCMI differs for a random
hero, TD-24). Built: `GameSetup::custom_types` and its checks, the merges, `town::custom_records`
(design §70 As built); the 89 goldens re-recorded (the import line; `e08-army-12`'s two replays'
hashes: the fixture maps' type-3 entry now makes a record); `reach-xl-144` and
`reach-lifted-xl-144` re-baselined (the XL map's games changed the same way).
