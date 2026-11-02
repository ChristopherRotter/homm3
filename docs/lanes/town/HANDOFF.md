# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). T1a, T1b and T1b2 were each cut in two (the user, 2026-11-02); **T1a1, T1a2, T1b1 and T1b2a done** (survey `E09-towns.md` §0 to §13). Next: **T1b2b**, the checkpoint's questions (`TWN-Q2`; the queue's entry, survey §13's list).
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; the crate is a `shared/` change after the ADR draft is accepted), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 2), quirks `TWN-QUIRK-<n>` (next 1), stubs `TWN-STUB-<n>` (next 1), rules `TWN-*` (none yet), decisions next 8, experiment batches `TWN-<n>` (next 1; TWN-1's rows planned, survey §13), the survey's disputes `TD-<n>` (next 8) and quirk candidates `T-<n>` (next 6).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-02 — T1b2a: income, views, batch TWN-1's rows

Baseline 43b1c3ad (#648 merged; CI green on it). The user cut T1b2 in two (T1b2a, T1b2b) for the views' new reading. Survey `E09-towns.md` §11 to §13 and findings 24 to 27: a town's gold by its highest hall and the Grail, paid to its owner from day 2; one Capitol per player only after a capture; what another player sees (the picture, the hint, the cursor, the right-click's fort level and the army by its taverns, the counts and hall for the owner's team, expert Visions or a Rogue); seven batch rows; 10 exe map entries. Pull request: from `town/e09-t1b2a` (opened at the session's end).

### 2026-11-02 — T1b1: the town on the map

Baseline ad0c9805 (#644 merged; CI green on 34fc55ef). The user cut T1b in two (T1b1, T1b2). Survey `E09-towns.md` §7 to §10 and findings 17 to 23: the step onto a gate (a defended town from the neighbouring tile), `town_visit` and `town_change_owner` read whole, the attacker outside after a siege, one Capitol per player, the garrison merged into a lone visitor's army, ADR-0023's facts from the town, the game's version at a new game; TD-7, T-5; 15 exe map entries. Pull request: #648 (`town/e09-t1b1`, merged).

### 2026-11-02 — T1a2: the heroes and the towns at the start

Baseline d5c70795 (#641 merged). The user kept T1a2 whole (asked at its start). Survey `E09-towns.md` §4 to §6 and findings 9 to 16: the pool, the starting-hero choice, one uniform draw for the random hero and the main town's hero (no tavern weights: increment 3's), the visitor, a hero in a town, the towns' garrisons and buildings at the start; TD-3 to TD-6, T-2 to T-4; 29 exe map entries, `structs.toml`'s `town`. Pull request: #644 (`town/e09-t1a2`, merged).
