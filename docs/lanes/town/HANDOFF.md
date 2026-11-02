# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). T1a was cut in two (the user, 2026-11-02); **T1a1 and T1a2 done** (survey `E09-towns.md` §0 to §6). Next: **T1b**, the town on the map and the checkpoint's questions (the queue's entry).
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; the crate is a `shared/` change after the ADR draft is accepted), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 2), quirks `TWN-QUIRK-<n>` (next 1), stubs `TWN-STUB-<n>` (next 1), rules `TWN-*` (none yet), decisions next 8, experiment batches `TWN-<n>` (next 1), the survey's disputes `TD-<n>` (next 7) and quirk candidates `T-<n>` (next 5).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-02 — T1a2: the heroes and the towns at the start

Baseline d5c70795 (#641 merged). The user kept T1a2 whole (asked at its start). Survey `E09-towns.md` §4 to §6 and findings 9 to 16: the pool, the starting-hero choice, one uniform draw for the random hero and the main town's hero (no tavern weights: increment 3's), the visitor, a hero in a town, the towns' garrisons and buildings at the start; TD-3 to TD-6, T-2 to T-4; 29 exe map entries, `structs.toml`'s `town`. Pull request: `town/e09-t1a2` (open).

### 2026-11-02 — T1a1: the census and the factions

Baseline f949c899 (CI green, no open pull requests). The user cut T1a in two (T1a1, T1a2). Survey `E09-towns.md` §0 to §3: the census of the 160 maps (all import once the main town's hero, a hero in a town and the random hero exist), the players' factions and a random town's faction read (`load_town_body` 0x5019F0), VCMI and the wiki (TD-1, TD-2, T-1); the exe map's entries; the adventure lane's inbox note (random dwellings tied to random towns). Pull request: #641 (`town/e09-t1a1`, open).

### 2026-10-31 — T0: E09 refined, the town lane opened

Baseline 11f4a137 (#632, #633 merged). The user, asked at E09's refinement (TWN-Q1): a town lane now, in parallel with E08; the crate `h3-towns` with the town as the adventure's map object; town sub-paths in the adventure lane's crates; E08's increments 10–12 stay E08's; increment 1 towns on the map; E09 also owns the faction choice and random towns, Town Portal and Castle Gate, the Grail, the Thieves' Guild. One `shared/town-lane` pull request (`cross-lane`): `lanes.toml` (the lane, `verify_with`), `xtask` (the scope covers the `verify_with` lanes' crates, replays and workloads), the process docs, BACKLOG, the epic file and log, this folder, the adventure lane's inbox note. Pull request: #634 (`shared/town-lane`, merged).
