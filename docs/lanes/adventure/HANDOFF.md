# Adventure lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E08**: increments 1 to 6 **closed** (6 on 2026-10-26 by T53's audit). **Increment 7: rewards on the map** (decisions 121–125, ADV-Q26, ADV-Q27): T54 done (the battle seam's facts; the retreat facts behind `ADV-STUB-9`, E10); round 1: T55a cut in four (decision 125), T55a1 done (survey `E08-rewards.md` §1 to §5: the census and the pickups; disputes D-29 to D-32). Records: `E08-log.md`; increment 6's scope and decisions in `E08-archive.md`.
- **Open pull requests:** none (T55a1's #474 merged).
- **Next:** merge T55a1's pull request; then T55a2 (the objects that teach), T55a3 (creatures, Pandora's Box, events), T55a4 (the common rules, batch E7's plan, the rule ids, the second reading; a user checkpoint at its end), T55b (the ADR drafts, ADR-0017 point 11's line for T54 among them, the design note, the cut); each cut confirmed with the user at its start.
- **Versions** (`docs/08-work-process.md`, "Versions"): game state hash v14 (T54), adventure state hash v12 (T54), adventure ruleset part v11 (T50), map import v11 (`MAP_IMPORT_VERSION`, T54), adventure replay format v6 (T51a); the env's action encoding v4 is the AI lane's. Built so far, by area: `OVERVIEW.md`; ownership: `lanes.toml` (the details are in `OVERVIEW.md`, "Ownership").
- **ADRs accepted**: ADR-0015 to -0017 (2026-10-10), amended for increments 2 to 6 (last: T45).
- **R01** (the content split): the core `h3-content` and `h3-pack` (`core.toml`) are shared; the writer is `h3_map::writer`.
- **Ids:** questions `ADV-Q<n>` (next 28), quirks `ADV-QUIRK-<n>` (1 the fringe; 2, 3 the diagonal; 4 neutral stacks; 5, 6 obstacles, split; 7 A-6; 8 formation; 19, 20 A-15, A-16 (no flags); flags 19–24; 15, 17, 18 increment 5's, flags 31, 32; 14, 16 withdrawn, flag 30 retired), stubs `ADV-STUB-<n>` (next 10; 8 and 9 open, E10's), rules `ADV-*` and `GAM-*` (last `GAM-BAT-013`); the E08 surveys' disputes `D-<n>` (next 33) and reading quirk candidates `A-<n>` (next 17). Versions owned: game state hash, adventure state and ruleset-part hashes, map import, adventure replay format.
- **Working agreement:** a couple of tasks per checkpoint; the user confirms seams and ADRs; tokens first; fuzzing stable-only.

## Standing traps

Moved to `TRAPS.md`: forks without deep copies, RNG seeds server-only, template masks, staging the user's archives (`maps.tar`, `extracted.tar`), a shallow clone, extra target directories under `target/`, an experiment's map and template from one compile, shared changes and `cross-lane`.

## Recent sessions

### 2026-10-26 — increment 7, round 1: T55a2, the objects that teach (in progress)

Baseline bef62ab (#474 merged; CI green; the sandbox's baseline verified the code). Cut confirmed (decision 126: whole, the University's dialog for its rules only). In progress: the readings for survey `E08-rewards.md` §6 on `adventure/e08-t55a2`.

### 2026-10-26 — increment 7, round 1: T55a1, the rewards survey's census and pickups

Baseline e1217a2 (#473 merged; CI green; the scoped verify passed). T55a cut in four with the user (decision 125: about 8 000 to 10 000 instructions to read, measured). T55a1: survey `E08-rewards.md` §0 to §5, §9, §10: the census of the reward classes (a throwaway example in a worktree), chests, placed and random artifacts (the unreachable pickup conditions), Spell Scrolls, the monsters' artifacts; 18 `map.toml` readings; D-29 to D-32 for the checkpoint. Docs only; the scoped verify passed (no LODs staged).

### 2026-10-26 — increment 7: T54, the battle seam's facts

Baseline f2fc414 (#472 merged; CI green). T54 whole (ADV-Q27, decision 124: the human flag from the map's player kinds, a session overrides; three `cross-lane` literals with consent). 0x41ADB0's tile class read (0x505610's four walks, 0x40FCA0's ownable classes); `h3-game` fills difficulty, human, colours, sexes, background and the attacker's ground (GAM-BAT-010 to -013); game state hash v14, adventure state hash v12, map import v11; the twelve replays re-recorded, commands unchanged. Scoped verify with the original data on 0507f5c: every step passed but `check-lane` (the three `cross-lane` literals, labelled); CI's full verify on the pull request.
