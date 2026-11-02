# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). T1a, T1b and T1b2 were each cut in two (the user, 2026-11-02); **T1 done**: the survey `E09-towns.md` §0 to §14 and its checkpoint (`TWN-Q2`, the epic's decisions 8–16). Next: **T2**, the ADR drafts, the design note and the cut (the queue's entry; offer its seam at the start).
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; the crate is a `shared/` change after the ADR draft is accepted), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 3), quirks `TWN-QUIRK-<n>` (next 1), stubs `TWN-STUB-<n>` (next 1), rules `TWN-*` (none yet), decisions next 17, experiment batches `TWN-<n>` (next 1; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 8) and quirk candidates `T-<n>` (next 6).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-02 — T1b2b: the checkpoint

Baseline b6c6020c (#649 merged). `TWN-Q2` asked in three rounds: the epic's decisions 8–16 (the random hero and the starting-hero pick in increment 1, the silo paid, the views by the original's levels, T-1 to T-5 as proposed, the disputes by the reading but TD-2 by the enabled expansions, E08 decision 104; batch TWN-1's rows 1 to 4; an allied town entered, never captured). Survey §14; the queue's T2 rewritten; a note in the adventure lane's inbox. Docs only. Pull request: from `town/e09-t1b2b` (opened at the session's end).

### 2026-11-02 — T1b2a: income, views, batch TWN-1's rows

Baseline 43b1c3ad (#648 merged; CI green on it). The user cut T1b2 in two (T1b2a, T1b2b) for the views' new reading. Survey `E09-towns.md` §11 to §13 and findings 24 to 27: a town's gold by its highest hall and the Grail, paid to its owner from day 2; one Capitol per player only after a capture; what another player sees (the picture, the hint, the cursor, the right-click's fort level and the army by its taverns, the counts and hall for the owner's team, expert Visions or a Rogue); seven batch rows; 10 exe map entries. Pull request: #649 (`town/e09-t1b2a`, merged).

### 2026-11-02 — T1b1: the town on the map

Baseline ad0c9805 (#644 merged; CI green on 34fc55ef). The user cut T1b in two (T1b1, T1b2). Survey `E09-towns.md` §7 to §10 and findings 17 to 23: the step onto a gate (a defended town from the neighbouring tile), `town_visit` and `town_change_owner` read whole, the attacker outside after a siege, one Capitol per player, the garrison merged into a lone visitor's army, ADR-0023's facts from the town, the game's version at a new game; TD-7, T-5; 15 exe map entries. Pull request: #648 (`town/e09-t1b1`, merged).
