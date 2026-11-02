# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). T1a, T1b and T1b2 were each cut in two (the user, 2026-11-02); **T1 done**: the survey `E09-towns.md` §0 to §14 and its checkpoint (`TWN-Q2`, the epic's decisions 8–16). T2 cut in two (the user, 2026-11-02); **T2a done**: the ADR drafts in `adr-drafts/` (the draft ADR-0028 for `h3-towns`, the amendments to ADR-0015, -0016, -0017, -0023; `TWN-Q3`, decisions 17–20). Next: **T2b**, the design note `E09-design.md`, the cut into rounds and the checkpoint (`TWN-Q4`, the drafts accepted with it; the queue's entry).
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; it, the adapter `h3-pack-towns` and `packs/*/towns.toml` come in a `shared/` change after the drafts are accepted), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 4), quirks `TWN-QUIRK-<n>` (next 1), stubs `TWN-STUB-<n>` (next 1), rules `TWN-*` (none yet), decisions next 21, ADR numbers 0028 (the draft's, checked when it moves), experiment batches `TWN-<n>` (next 1; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 8) and quirk candidates `T-<n>` (next 6).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-02 — T2a: the ADR drafts

Baseline 4f457aaf (#652 merged). The user cut T2 in two (T2a the ADR drafts, T2b the design note, the cut and the checkpoint). `TWN-Q3` asked before writing, one round of four, each as recommended: the store of towns in the game state with `TownId`, the pool `h3-game`'s, a town part with `towns.toml` and `h3-pack-towns`, the moat per faction in it with the SoD pack's overrides (decisions 17–20). Five drafts in `adr-drafts/`. Docs only. Pull request: from `town/e09-t2a` (opened at the session's end).

### 2026-11-02 — T1b2b: the checkpoint

Baseline b6c6020c (#649 merged). `TWN-Q2` asked in three rounds: the epic's decisions 8–16 (the random hero and the starting-hero pick in increment 1, the silo paid, the views by the original's levels, T-1 to T-5 as proposed, the disputes by the reading but TD-2 by the enabled expansions, E08 decision 104; batch TWN-1's rows 1 to 4; an allied town entered, never captured). Survey §14; the queue's T2 rewritten; a note in the adventure lane's inbox. Docs only. Pull request: #652 (`town/e09-t1b2b`, merged).

### 2026-11-02 — T1b2a: income, views, batch TWN-1's rows

Baseline 43b1c3ad (#648 merged; CI green on it). The user cut T1b2 in two (T1b2a, T1b2b) for the views' new reading. Survey `E09-towns.md` §11 to §13 and findings 24 to 27: a town's gold by its highest hall and the Grail, paid to its owner from day 2; one Capitol per player only after a capture; what another player sees (the picture, the hint, the cursor, the right-click's fort level and the army by its taverns, the counts and hall for the owner's team, expert Visions or a Rogue); seven batch rows; 10 exe map entries. Pull request: #649 (`town/e09-t1b2a`, merged).
