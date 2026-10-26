# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed** (6 on 2026-10-26 by T53's audit: every exit criterion met). **Increment 7: rewards on the map** (decisions 121–123, ADV-Q26): refined; nothing built yet. Records: `E08-log.md`; increment 6's scope and decisions in `E08-archive.md`, its design note in the log.
- **Open pull requests:** `adventure/e08-t53` (T53; docs and six code comments).
- **Next:** merge T53's pull request; then T54 (`QUEUE.md`: `h3-game` fills the battle seam's facts, decision 123; the source of a player's human flag is the user's question at its start), then round 1: T55a (the rewards survey), T55b (the ADR drafts, the design note, the cut); each cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v13 (T51b), adventure state hash v11, adventure ruleset part v11 (T50), map import v10 (`MAP_IMPORT_VERSION`, T49b), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 27), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>` (next 9, T54's), rules `ADV-*` and `GAM-*`. Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-26 — increment 6 closed (T53); increment 7 refined

Baseline e8b9164 (#470, #471 merged; CI green). T53 whole (decision 120): the audit (seven criteria met; GAM-MET-001 and GAM-MAN-001 `verified`, ADV-MON-008 cites E6-1, E6-2), increment 6 moved to the archive and its design note to the log (the code's six citations repointed), increment 7 refined with the user (ADV-Q26: rewards on the map; T54 the seam's facts first; T55a, T55b); three informational inbox notes archived. Scoped verify passed (exit 2, its normal result); CI's full verify on the pull request.

### 2026-10-26 — increment 6, round 5: T52

Baseline 7619a4a (#468 merged; verified). T52 whole (decision 119): the plans and properties manage (meetings, joins, arbitrary commands; invariants, effects, inverses), the army map and the scripted golden replays (`run --script`), two gated benchmarks (forks with a meeting, the listing in a meeting); no version moved. Full verify with the original data green (code 5a00f64), CI green; #469 merged, #470 open; #471 (shared) records the user's rule that CI's full verify is the record.

### 2026-10-26 — increment 6, round 4: T51b; round 4 closed

Baseline f72f8c2 (full verify green). T51b whole (decision 118): the meeting with Scholar, the army dialog with the join offered without room, `Manage` between two armies; game state hash v13, `ADV-STUB-2` closed; the ten replays' commands unchanged. Full verify green; #468 open.
