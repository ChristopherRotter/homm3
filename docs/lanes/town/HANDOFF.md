# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns) refined with the user on 2026-10-31 (TWN-Q1; the epic's decisions 1–7); the lane opened the same day beside the adventure lane's E08. **Increment 1: towns on the map** (the epic's scope). **T1 done** (the survey `E09-towns.md` §0 to §14; `TWN-Q2`, decisions 8–16). **T2 done**: T2a the ADR drafts in `adr-drafts/` (`TWN-Q3`, decisions 17–20); T2b the design note `E09-design.md` §1 to §22 and the cut (`TWN-Q4`, decisions 21–24: the drafts accepted as written, the cut as proposed, the content read and harvested, the agent's calls kept). **T3a done** (round 2): `fort` and an explicitly empty `garrison` for a spec's towns (`h3-experiment`, `cross-lane`), the spec `experiments/specs/E09-batch-1.toml` and its template. Next: **T3b**, batch TWN-1's harness and run (round 3; original data staged, the capture harness), then T3c; rounds 4 to 10 in the epic's list.
- **Open pull requests:** see the newest session entry.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns` (not created yet; it, the adapter `h3-pack-towns` and `packs/*/towns.toml` come with the `shared/` task T4, after batch TWN-1), the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and `src/writer/town*` of `h3-map`, `docs/rules/catalog/town.toml`, `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Everything else in the adventure lane's crates is the adventure lane's: a change there is `cross-lane`, merged in order with the adventure lane's pull requests.
- **Versions:** the lane owns none yet (`h3-towns`' come with the crate). It bumps the adventure lane's (game state hash, adventure state and ruleset-part hashes, map import, adventure replay format) in `cross-lane` pull requests; their current values are in the adventure lane's HANDOFF.
- **Ids:** questions `TWN-Q<n>` (next 5), quirks `TWN-QUIRK-<n>` (next 1; 1 to 4 reserved by the design §19 for T-1 to T-4), stubs `TWN-STUB-<n>` (next 1; 1 and 2 reserved by the design §19), rules `TWN-*` (none yet; the design §19 names them), decisions next 25, ADR numbers 0028 (the draft's, checked when it moves), experiment batches `TWN-<n>` (next 1; TWN-1's rows 1 to 4, survey §13, decision 15), the survey's disputes `TD-<n>` (next 8) and quirk candidates `T-<n>` (next 6).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-02 — T3a: batch TWN-1's maps and spec

Baseline 75cfdd7 (#656 merged; CI on `master` green at c1a6f68). The user kept T3a whole and
consented to the `cross-lane` change. `h3-experiment`'s towns gain `fort` and `garrison = []`
(the read-back test); the spec for rows 1 to 4 (row 3 designed by a scratch simulation of both
merges so each tie and the extra round moves a stack) and its template. CI's first run failed a
random `h3-map` property case on `master`'s code (the adventure lane's inbox). Pull request: #659
(`town/e09-t3a`, `cross-lane`).

### 2026-11-02 — T2b: the design note, the cut and the checkpoint

Baseline c1a6f68 (#653 merged; CI on `master` green). The user kept T2b whole. `E09-design.md` §1 to §22 from the code the build touches; the cut: fourteen tasks in rounds 2 to 10, batch TWN-1 first (§21). `TWN-Q4` in one round of four, each as recommended (decisions 21–24). The queue: T3a to T3c written, six inbox notes the note carries moved to `QUEUE-ARCHIVE.md`; a note in the adventure lane's inbox on the planned bumps. The VM restarted once (nothing lost). Docs only. Pull request: #656 (`town/e09-t2b`, merged).

### 2026-11-02 — T2a: the ADR drafts

Baseline 4f457aaf (#652 merged). The user cut T2 in two (T2a the ADR drafts, T2b the design note, the cut and the checkpoint). `TWN-Q3` asked before writing, one round of four, each as recommended: the store of towns in the game state with `TownId`, the pool `h3-game`'s, a town part with `towns.toml` and `h3-pack-towns`, the moat per faction in it with the SoD pack's overrides (decisions 17–20). Five drafts in `adr-drafts/`. Docs only. Pull request: #653 (`town/e09-t2a`, merged).
