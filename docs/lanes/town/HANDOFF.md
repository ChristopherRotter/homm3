# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increments 1
  (towns on the map) and 2 (construction with the town events) closed** 2026-11-06 (T10) and
  2026-11-11 (T20; decisions 36–63; the audit in `E09-log.md`, T20); both in `E09-archive.md`,
  their design notes kept in `E09-design.md` Parts A to D (decision 40). **Increment 3: growth,
  recruitment and the army rows** refined at T20 (`TWN-Q11`; decisions 64–67): the weekly growth
  (dwellings, castle's bonus, hordes, the week's and month's creature with the Plague, the town
  events' creatures `TWN-STUB-10`), recruitment through E08's hire dialog into the garrison or the
  visitor, the garrison as an army and the heroes' swap. The Tavern is now increment 4, the guild
  and the services 5, travel and the Grail 6 (decision 67: older docs' "increment 4/5" mean 5/6).
  **T21 done** (kept whole, decision 68): the survey `docs/rules/surveys/E09-growth.md` and its
  checkpoint `TWN-Q12` (decisions 69–73: TD-13 and TD-14/T-12 `fix-optional`, the Portal drawn
  at its build and a dwelling flagged, TD-15 the reading). **Next: T22** (the drafts, Part E from
  §37, the cut, batch TWN-3's runs; its cut confirmed at its start), after the army reference is
  agreed with the adventure lane (T21's note in its inbox; its T108b drafts the map's garrisons).
  Waiting elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle lane's
  inbox), the guild's spells at the start (`TWN-STUB-6`, increment 5), the Grail building's forbid
  (`TWN-STUB-2`, the victory conditions), Visions' reach told (`TWN-STUB-13`, increment 5; the
  server lane's fold test exempts it).
- **Open pull requests:** T21's (branch `town/e09-t21`, docs only: the survey, the address map,
  the lane's docs and the adventure lane's `QUEUE.md`). #846, #848 merged.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 2 (T17a's day's mark; v1 its first encoding, T6a), the part's 4 (T16a's requirements, costs and `shipyard`; v3 T8c2's Visions, v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a to T16b none; T17a: game state hash v26; T17b none; T18a: adventure ruleset part v19; T18b1: adventure replay format v11; T18b2a, T18b2b none; T18c: the server lane's protocol v41; T19a, T19b none; T19c1: the AI lane's adventure action
  encoding v9; T19c2, T20 none); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 13), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b, 7, T-9, since T17b; next 8), stubs `TWN-STUB-<n>` (2 registered by T6c, narrowed to the Grail by T18a; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 5's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 13 by T18b2a, Visions' reach; 10 by T18c, the events' creatures; 1 reserved, E10's; 11, 12 planned by design §34, registered by their tasks; next 14), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1, TWN-BLD-004, -005 since T17a, TWN-BLD-001 to -003, -006, -007, TWN-EVT-003 since T17b, TWN-SET-011 since T18a, TWN-VEW-004 since T18b2a, TWN-EVT-001, -002, -004 since T18c), decisions next 74, ADRs: 0028 is the lane's (accepted, T4a; increment 2's amendments applied by T15; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 4; TWN-3's rows gathered by T21, survey `E09-growth.md` §14, to run per T22; TWN-2 scored by T14c, survey `E09-construction.md` §13; TWN-1 survey `E09-towns.md` §15), the survey's disputes `TD-<n>` (next 18; TD-13 to TD-17 by T21) and quirk candidates `T-<n>` (next 14; T-10 to T-13 by T21; T-12 and T-13 `fix-optional`, registered as `TWN-QUIRK-<n>` by their building tasks).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-11 — T21: increment 3's survey and its checkpoint

Baseline d3bf756 (#846 to #850 merged; the setup's baseline the light gates, CI's fingerprint).
The user's computer was connected only at the start: the original data staged then (57 files).
The cut measured (growth 3.5 to 5.5 KB unread, recruitment and the rows 2 to 3 KB); the agent
recommended two parts, the user kept T21 whole (decision 68). The survey `E09-growth.md` (code
reading, VCMI 1.7.5, the wiki, the census by a throwaway function in `original_data`, not
committed); 22 new address map entries; the checkpoint `TWN-Q12` (decisions 69–73); the
adventure lane's inbox told the army reference proposal. Docs only; no version moved.

### 2026-11-11 — T20: increment 2 audited and closed, increment 3 refined

Baseline 1e54a50 (#841 to #845 merged; the setup's baseline the light gates, CI's fingerprint).
The user's computer was connected only at the start: the original data staged then. T20 whole,
increment 3 as growth, recruitment and the army rows, the survey first (the user, decisions
64–66). The eight exit criteria hold; criteria 5 (refusals are the builds map's test) and 8 (the
server's manifest its own) worded as built; TWN-BLD-001 `verified`; the census run again, T19c2's
counts. The sketch renumbered (the Tavern 4) and the live citations with it, the ADRs' in
`shared/e09-t20-increments`; the adventure lane told of the army exchange. No version moved.

### 2026-11-11 — T19c2: the census of the 160 maps, the new workloads

Baseline 81024ba (#839, #840 merged); the setup's full baseline lost to a VM restart, CI's full
verify on `master`'s engine code stands for it. The user's computer was connected only at the
start: the original data staged then. Kept whole (the user, decision 63). The census plays every
game by `EndTurn` alone (a week, the 29 maps with events to their last first day) and checks the
town events' firings and builds against its own walk of the rules; survey §10's 57/30/27 come
back in T12b's terms (each building one by one, direct requirements). `h3 adventure bench
--workload legal`; `legal-towns-36`, `day-start-builds-36` baselined (master the same within
0.05 %). The README's rows in `shared/e09-t19c2-benches` (#841). No version moved.
