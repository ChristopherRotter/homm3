# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** 2026-11-06 (T10; `E09-archive.md`; its design note kept in
  `E09-design.md` §1–§22, decision 40). **Increment 2: construction with the town events**
  (`TWN-Q7`; decisions 36–57; the design note's Part D §23–§36, the exit criteria in the epic): T11
  to T18c done (the survey, the drafts and `shared/` T15, batch TWN-2, the part's requirements,
  costs and Shipyard and their harvest, the day's mark and the store's writes, the rules that
  decide a build with `TownCommand`, the Shipyard's water, the build command in the game with the
  day's start and the capture's mark, the views and every viewer's changed view told, the legal
  builds and the hall's offer, the town events at the turn start; design §26 to §31 as built).
  T19 (§32) cut in three (the user, decisions 58, 59): T19a done (the writer's events,
  increment 2's own map `e09-builds-36.h3m`, its replays `builds-*`), T19b done (the plans'
  towns' events, `Step::Build`, `check_timed` and `check_towns` taught the events; decisions 60,
  61). T19c cut in two (decision 62): **T19c1 done** (the env's build entry and building head,
  the AI lane's action encoding v9; the rollout draws the builds; random replays
  `builds-s262-d14`, `-s271-d14`, `-s290-d14`; three `towns-36` workloads re-baselined).
  **Next: T19c2** (the new workloads against `master`, the census of the 160 maps with the
  original data), then T20 (design §35). Waiting elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's
  inbox), the guild's spells at the start (`TWN-STUB-6`, increment 4), the Grail building's forbid
  (`TWN-STUB-2`, the victory conditions), Visions' reach told (`TWN-STUB-13`, increment 4; the
  server lane's fold test exempts it).
- **Open pull requests:** T19c1's #837 (branch `town/e09-t19c1`, `cross-lane`: `h3-sim`'s
  `src/adventure/`, `h3-cli`'s bench and replay test, the replays, the workloads, the replay
  counts of `h3-protocol` and `h3-server`). #832 merged.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 2 (T17a's day's mark; v1 its first encoding, T6a), the part's 4 (T16a's requirements, costs and `shipyard`; v3 T8c2's Visions, v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a to T16b none; T17a: game state hash v26; T17b none; T18a: adventure ruleset part v19; T18b1: adventure replay format v11; T18b2a, T18b2b none; T18c: the server lane's protocol v41; T19a, T19b none; T19c1: the AI lane's adventure action
  encoding v9); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 11), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b, 7, T-9, since T17b; next 8), stubs `TWN-STUB-<n>` (2 registered by T6c, narrowed to the Grail by T18a; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 13 by T18b2a, Visions' reach; 10 by T18c, the events' creatures; 1 reserved, E10's; 11, 12 planned by design §34, registered by their tasks; next 14), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1, TWN-BLD-004, -005 since T17a, TWN-BLD-001 to -003, -006, -007, TWN-EVT-003 since T17b, TWN-SET-011 since T18a, TWN-VEW-004 since T18b2a, TWN-EVT-001, -002, -004 since T18c), decisions next 63, ADRs: 0028 is the lane's (accepted, T4a; increment 2's amendments applied by T15; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 3; TWN-2 scored by T14c, survey `E09-construction.md` §13; TWN-1 survey `E09-towns.md` §15), the survey's disputes `TD-<n>` (next 13; TD-11 by T12a, TD-12 by T12b) and quirk candidates `T-<n>` (next 10; T-8 by T12b, T-9 by T13; T14c raised none).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-10 — T19c1: the env's build entry, random play with builds

Baseline 49cc2d5 (#832, #833 merged; the setup's baseline the light gates, CI's full verify green
on the same engine code). The user's computer was connected only at the start: the original data
staged then. T19c cut in two, the env first (the user, decision 62). Action encoding v9: `BUILD`
(30) with the town's entrance in the tile head and a building head of 64 (`building_mask`); the
rollout draws every listed build; the town map's env tests on new seeds (142, 244); three random
replays with builds on the builds map (72 replays); `day-start-`, `fork-` and `reach-towns-36`
re-baselined (the random games changed course, the same code per function, measured against
`master` in the sandbox). The AI lane's inbox told.

### 2026-11-10 — T19b: the plans build

Baseline 1769e23 (#827, #828 merged; the setup's full engine verify with the original data
green). The user's computer was connected only at the start: the original data staged then.
Kept whole, `master`'s counterexample first (the user, decisions 60, 61; the adventure lane's
#830 fixed it too, its fix kept at the merge); a hygiene gap the new draws found fixed
(`outcome` compares the removed objects). The plans' towns get events
(`town_events`), `Step::Build` and its runs; `check_timed` compares the town events after the
map's (the build's views left out), `check_towns` folds an event's builds without the mark. The
towns' cover test counts builds by command and event and the events fired, gave, took, skipped;
the town views' counts the mark told to others (512 cases now). Tests only; no version moved.

### 2026-11-10 — T19a: the writer's town events, the builds map, its replays

Baseline 3f60bf4 (#825, #826 merged during the setup; its first baseline lost to a VM restart,
run again). The user's computer was connected only at the start: the original data staged then.
T19 cut in three, increment 2's own map (the user, decisions 58, 59). `writer::TownEvent` and
`AnyTown::events`; `e09-builds-36.h3m` (the Harbour by the sea, the Keep inland, the Conflux
Nexus forbidding a requirement, the Tower Spire; the town events) and its tests, every refusal
kind once with the state hash unchanged (a replay holds none); `builds-week-s1`, `-capture-s1`,
`-events-s1` (69 replays; the server lane's counts in `h3-protocol` and `h3-server`); no version
moved. The setup's baseline on `master` found a property counterexample of the adventure lane's
`check_rewards` (a monster's join before the garden it guarded): their inbox, at the user's word.
