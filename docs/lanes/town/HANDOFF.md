# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** 2026-11-06 (T10; `E09-archive.md`; its design note kept in
  `E09-design.md` §1–§22, decision 40). **Increment 2: construction with the town events**
  (`TWN-Q7`; decisions 36–54; the design note's Part D §23–§36, the exit criteria in the epic): T11
  to T18b2a done (the survey, the drafts and `shared/` T15, batch TWN-2, the part's requirements,
  costs and Shipyard and their harvest, the day's mark and the store's writes, the rules that
  decide a build with `TownCommand`, the Shipyard's water, the build command in the game with the
  day's start and the capture's mark, the views and every viewer's changed view told; design §26
  to §31 as built, §30's day and capture). **Next: T18b2b** (the legal builds and `town_offer`,
  the bench gate, the plans building, the rollout's choice; `cross-lane`; queued), then T18c, T19,
  T20 (design §35). Until T18b2b `legal_commands` lists no build: a build is accepted by `apply`
  only. Waiting elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's
  inbox), the guild's spells at the start (`TWN-STUB-6`, increment 4), the Grail building's forbid
  (`TWN-STUB-2`, the victory conditions), Visions' reach told (`TWN-STUB-13`, increment 4; the
  server lane's fold test exempts it).
- **Open pull requests:** T18b2a's (branch `town/e09-t18b2a`, `cross-lane`: `h3-game`'s events,
  projection, capture, merge, battle and day hooks, `h3-map`'s properties, `h3-protocol`'s wire
  town); a `shared/` PR with `docs/06-security.md`'s towns events words (merge it first).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 2 (T17a's day's mark; v1 its first encoding, T6a), the part's 4 (T16a's requirements, costs and `shipyard`; v3 T8c2's Visions, v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a to T16b none; T17a: game state hash v26; T17b none; T18a: adventure ruleset part v19; T18b1: adventure replay format v11; T18b2a none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 11), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b, 7, T-9, since T17b; next 8), stubs `TWN-STUB-<n>` (2 registered by T6c, narrowed to the Grail by T18a; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 13 by T18b2a, Visions' reach; 1 reserved, E10's; 10 to 12 planned by design §34, registered by their tasks; next 14), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1, TWN-BLD-004, -005 since T17a, TWN-BLD-001 to -003, -006, -007, TWN-EVT-003 since T17b, TWN-SET-011 since T18a, TWN-VEW-004 since T18b2a), decisions next 55, ADRs: 0028 is the lane's (accepted, T4a; increment 2's amendments applied by T15; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 3; TWN-2 scored by T14c, survey `E09-construction.md` §13; TWN-1 survey `E09-towns.md` §15), the survey's disputes `TD-<n>` (next 13; TD-11 by T12a, TD-12 by T12b) and quirk candidates `T-<n>` (next 10; T-8 by T12b, T-9 by T13; T14c raised none).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-10 — T18b2a: the views, every viewer's changed view told

Baseline d1b90fb (#809, #810 merged; the setup's full engine baseline green, CI's run on
`master` green). T18b2 cut in two at its start (the user, decision 52); SRV-Q26's town gap told
per viewer, `GameEvent` kept `Copy` (decision 53), Visions' reach stubbed (decision 54). The
user's computer was connected only at the start: the original data staged then. `TownView`'s
mark and the owner's buildings; `Told` and `TownChanged`, `TownStackSeen`, `TownIncomeSeen` at the
build, the capture, the merge, the garrison's battle and the day's start; the wire's town
fields; the plans' told views folded strictly; TWN-VEW-001 amended, TWN-VEW-004, `TWN-STUB-13`;
the server lane's inbox told; no version moved.

### 2026-11-09 — T18b1: the build command, the day's start, the capture's mark

Baseline 31e0ea3 (#806, #807 merged; the setup's baseline lost to a sandbox restart, CI's run on
`master` stands for it). T18b cut in two at its start (the user): T18b1 this session, T18b2
queued (as soon as builds are listed the plans build, so the legal builds go with the views).
The user's computer was connected: the original data staged at the start. `GameCommand::Town`,
`GameCommandError::Town`, `GameEvent::TownBuilt`, `town::command` and `town::holds_capitol`;
`Towns::start_day` before the income; the capture's mark; `TownCommand` and its error made
exhaustive (SRV-Q25); replay format v11, the 55 replays' `format` line only; the adventure and
server lanes' inboxes told; #808 merged in (the replay words now in `h3-files`); PR #809.

### 2026-11-09 — T18a: the Shipyard's water

Baseline ae7d260 (#801, #802, #804 merged; the setup's baseline lost to a sandbox restart, CI's
green run on `master` stands for it). Kept whole (the user). The user's computer was connected:
the original data staged at the start (the census, the scoped verify, the setup's order read).
The order settled (the towns' setup after the map's read: "after"); `ObjectKind::Town {
boat_offsets }`, part v19, `StaticMap::town_boat_spot`, `h3_towns::without_water` and the game's
call; TWN-SET-011; the census (591 of 625 towns with a Shipyard in their faction have no spot,
the 9 built ones all water); 55 replays and the bot reports re-hashed; `cross-lane`, PR #807.
