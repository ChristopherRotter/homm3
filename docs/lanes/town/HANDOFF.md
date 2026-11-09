# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; decisions 1–7). **Increment 1
  (towns on the map) closed** 2026-11-06 (T10; `E09-archive.md`; its design note kept in
  `E09-design.md` §1–§22, decision 40). **Increment 2: construction with the town events**
  (`TWN-Q7`; decisions 36–51; the design note's Part D §23–§36, the exit criteria in the epic): T11
  to T16b done (the survey, the drafts and `shared/` T15, batch TWN-2, the part's requirements,
  costs and Shipyard and their harvest). **T17 cut in two** (the user, 2026-11-09): **T17a done**
  (the day's mark, `built_with`, the store's writes; design §26 as built). **Next: T17b** (the
  rules that decide a build, `TownCommand`; design §27; queued), then T18a–c, T19, T20 (design
  §35). Waiting elsewhere: a walled Stronghold's natural obstacles (`TWN-STUB-8`, the battle
  lane's inbox), the guild's spells at the start (`TWN-STUB-6`, increment 4).
- **Open pull requests:** T17a's, branch `town/e09-t17a` (`cross-lane`: the game state hash's
  version, the adventure replays, the adventure lane's HANDOFF versions line and inbox).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, the adapter `h3-pack-towns` and each pack folder's `towns.toml` (T4b), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns the town state hash and the town ruleset-part hash (`TOWN_STATE_HASH_VERSION`, `TOWN_RULESET_HASH_VERSION`, `h3-towns`), the store's 2 (T17a's day's mark; v1 its first encoding, T6a), the part's 4 (T16a's requirements, costs and `shipyard`; v3 T8c2's Visions, v2 T5b2a's `auto_with`; pinned in its test, recorded in the adventure replays' headers). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests (T5a2: the replay format v9; T7a1: map import v17, after E08 T93b's v16; T7b1: game state hash v23; T8a1: part v17, map import v18, adventure state hash v21; T7b2a: game state hash v24; T7b2b, T8a2, T8b1, T8b2a, T8b2b, T8c1, T8c2, T9a, T9b1, T9b2a, T9b2b, T9c, T10 none; T11: map import v20; T12a to T16b none; T17a: game state hash v26); their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 11), quirks `TWN-QUIRK-<n>` (`docs/rules/quirks-town.md` holds 5, T-6, since T6c, 1 and 4 since T7a1, 2 and 3 since T7b1, 6, T-7, since T7b2b; next 7, planned for T-9, design §34), stubs `TWN-STUB-<n>` (2 registered by T6c; 3 registered by T7a1, lifted by T7b2b; 4 and 5 by T8a1, 5 narrowed by T7b2a, both lifted by T8a2; 6 by T7b2b, increment 4's; 7 by T8a2, narrowed by T8b2a, T8b2b's; 8 by T8b1, the battle lane's; 7 lifted by T8b2b; 9 by T8c1, lifted by T8c2; 1 reserved, E10's; next 10; 10 to 12 planned by design §34, registered by their tasks), rules `TWN-*` (the design §19 names them; in `town.toml`: TWN-INC-001, -002, TWN-SIE-003 to -005, TWN-CAP-001, -002, TWN-VEW-002 since T6b, TWN-SET-003, -004 since T6c, TWN-SET-001, -010 since T7a1, TWN-SET-002, -006, -008 since T7a2, TWN-SET-005, -007 since T7b1, TWN-SET-009 since T7b2b, TWN-VIS-001, -002 since T8a2, TWN-SIE-006, TWN-CAP-003 since T8b1, TWN-SIE-001 since T8b2a, TWN-SIE-002 since T8b2b, TWN-VEW-001, -003 since T8c1, TWN-BLD-004, -005 since T17a), decisions next 52, ADRs: 0028 is the lane's (accepted, T4a; increment 2's amendments applied by T15; a new one drafts in `adr-drafts/`), experiment batches `TWN-<n>` (next 3; TWN-2 scored by T14c, survey `E09-construction.md` §13; TWN-1 survey `E09-towns.md` §15), the survey's disputes `TD-<n>` (next 13; TD-11 by T12a, TD-12 by T12b) and quirk candidates `T-<n>` (next 10; T-8 by T12b, T-9 by T13; T14c raised none).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-09 — T17a: the day's mark, what a build does and the store's writes

Baseline 2e5e08f (#794, #795 merged; CI green; the setup found CI's fingerprint). T17 cut in two
at its start (the user): T17a this session, T17b queued. The user's computer was connected: the
original data staged at the start. `h3-towns`: `Town::built_today`, `built_with` (new
`build.rs`), `Towns::build`, `start_day`, `captured`, `facts`, the town state hash v2;
TWN-BLD-004, -005; the game state hash v26 and the 55 adventure replays re-hashed (only hash
lines), `cross-lane`; the adventure lane told (T105b's v27).

### 2026-11-07 — T16b: the harvest of the requirements, the costs and the role `shipyard`

Baseline a65f303 (#791, #792, #793 merged; the setup found CI's fingerprint). Kept whole (the
agent's call). The user's computer was connected: the original data staged at the start (the
harvest read `Heroes3.exe` and `H3bitmap.lod`). `h3-harvest`'s `Building.txt` rows as
0x5C14C0 reads them, the lists' requirements, the Shipyard; `packs/base` and `packs/ab`
re-harvested; `h3-pack-towns`' test of the original packs against survey §5 and §6; the 55
adventure replays re-hashed (only `town_ruleset_hash`), `cross-lane`; T17 queued.

### 2026-11-07 — T16a: the town part's requirements, costs and the role `shipyard`

Baseline 02b01bd (#788 merged; CI green; the setup found CI's fingerprint). Kept whole (the
agent's call: the user asks for a cut only for a big task). The user's computer was connected:
the original data staged at the start, not needed. `h3-towns`' and `h3-pack-towns`' format,
compiler, refusals and hash v4; the synthetic pack's requirements, costs and Shipyard; the 55
adventure replays re-hashed (only `town_ruleset_hash`), `cross-lane`; T16b queued.
