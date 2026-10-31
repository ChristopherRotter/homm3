# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). Next: **T1a**, the towns survey's census and the start (the queue's entry).
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; the crate is a `shared/` change after the ADR draft is accepted), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 2), quirks `TWN-QUIRK-<n>` (next 1), stubs `TWN-STUB-<n>` (next 1), rules `TWN-*` (none yet), decisions next 8, experiment batches `TWN-<n>` (next 1).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-10-31 — T0: E09 refined, the town lane opened

Baseline 11f4a137 (#632, #633 merged). The user, asked at E09's refinement (TWN-Q1): a town lane now, in parallel with E08; the crate `h3-towns` with the town as the adventure's map object; town sub-paths in the adventure lane's crates; E08's increments 10–12 stay E08's; increment 1 towns on the map; E09 also owns the faction choice and random towns, Town Portal and Castle Gate, the Grail, the Thieves' Guild. One `shared/town-lane` pull request (`cross-lane`): `lanes.toml` (the lane, `verify_with`), `xtask` (the scope covers the `verify_with` lanes' crates, replays and workloads), the process docs, BACKLOG, the epic file and log, this folder, the adventure lane's inbox note. Pull request: #634 (`shared/town-lane`, open).
