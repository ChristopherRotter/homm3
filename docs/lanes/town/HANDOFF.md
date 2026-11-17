# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–136, `TWN-Q15` to `-Q20`): T31 to T36 done;
  T37a done (decisions 133–136); **T37b done** (whole, decisions 137–139, `TWN-Q21`: every hero
  of a customised type takes its entry, a record per such free type at the build; map import
  v25). **Next:** T38a (the hire; `QUEUE.md`), its cut confirmed at its start. The guild and the services are increment
  5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6`
  and `-13` (increment 5), `TWN-STUB-2` (the victory conditions).
- **Open pull requests:** T37b's `town/e09-t37b` (`cross-lane`), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v7: T36), both in `h3-towns`; the adventure lane's
  (game state hash v33 by T37a, adventure state hash v26 by E08's T124b1, **map import v25 by
  T37b**: T39a takes v26, replay format v16 by T28a), the env's (v11 by T28a) and the protocol's (**v70 by
  T37a**) are bumped `cross-lane` (each task's record in `E09-log.md`; the plan in design §77).
  Shared: core ruleset hash v53, pack format v52 (T24), quirk flags 43 and `StreamId::TAVERNS` (8)
  by T35b.
- **Ids** (next): questions `TWN-Q22`; decisions 140; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b, their tests with T38b, T39b); stubs `TWN-STUB-19` (`-11`, `-12` planned by §34; `-15`
  to `-18` by §71, §77); rules: TWN-TAV-001, -002, -005 built (T37a), TWN-TAV-003, -004, -006 to
  -009, GAM-DEC-010, GAM-MAN-003 planned; experiment batch `TWN-5`; disputes `TD-25` (TD-24 by
  T37b); quirk
  candidates `T-18`. ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-17 — T37a: the pool in the game, the views, protocol v70

Baseline 2f6ed77 (#980 merged; the setup's baseline lost to a VM restart, CI's green `master`
stood in). The user kept T37a whole (decision 133) and, after the agent's check that the
original's tavern shows the offer's whole hero screen as a held popup, chose its parts exactly
(decision 134) and a head with one part event (135). The user's computer was connected at the
start only: 15 files staged (the four LODs, both executables, both tars, the saves, the manuals;
SHA-256 checked). Built: design §61 to §63, §68, §75 (game state hash v33, protocol v70); the
week-start benches measured, one design pass, re-baselined (+13 to +99 %: the week's fill); the 89
replays re-recorded; #984 the `shared/` half. CI's first run on #987 met `master`'s latent
`check-exe-map` gap (E24 M4 T1a's 14 cited addresses): entries added, the map-client lane told;
the Clients job is red on `master` too (`h3-ui`'s `message.rs`, not this branch's).

### 2026-11-17 — T35b and T36: the taverns' stream, the two flags, the town part v7

Baseline 4f4fbbc (#976 merged; the setup verified the code, e3cedff5, after a VM restart). The user
kept T35b whole and T36 after it, both `cross-lane` (decisions 129, 130). The user's computer was
connected at the start only: 11 files staged for T35b and T36 (the four LODs, both executables,
both tars, the saves; SHA-256 checked). Built: `StreamId::TAVERNS`, `QuirkFlag(42)`, `(43)`, the
profiles, the ledger entries; the 86 replays' `ruleset_hash`, the bot reports, the server literal
(#979). T36 stacked on it: the records' tavern weights and the hire's price (part v7), the
harvest, the AB pack's overrides (the base records cannot name its classes), the 89 replays'
`town_ruleset_hash`.
