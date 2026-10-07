# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-t5b` (PR #6, labelled `cross-lane`: the seam change, T5b and BAT-Q4, BAT-Q5; PRs #4 and #5 merged). Work on `battle/<task>` branches from `master`, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`.
- **E04b increment 3 (Q54, Q56, BAT-Q1, BAT-Q3, BAT-Q5): done.** T5a, T5b, T5c1, T5c2; batches 2b and 2c folded in. Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), Anti-Magic and Magic Mirror (BAT-MAG-021/022), Forgetfulness and Blind (BAT-MAG-023/024), Berserk and Hypnotize (BAT-MAG-025/026; a stack's `controller()` commands it), Force Field, Fire Wall, Teleport and Expert Dispel's clearing (BAT-MAG-027 to 030; `h3_battle::obstacles`), hidden obstacles, Quicksand and Land Mine (BAT-MAG-031 to 033; BAT-QUIRK-1/2/4), Remove Obstacle and the natural obstacles as pieces with anchors (BAT-MAG-034, BAT-GEO-003; `h3_battle::natural`, BAT-QUIRK-3), all after the executable's code (`installation.md` §7.8–§7.17). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the T5b checkpoint: the user reviews and merges PR #6 (cross-lane) and runs the Windows verify; then cuts the next increment: T6 (life and summons: Resurrection, Animate Dead, Sacrifice, Clone, the Elementals; `StackOrigin`, the design note's T6 section) or T7 (creature magic).
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v28, ruleset hash v25, replay format v18, action encoding v10, C ABI v10, observation encoding v6; pack format v16. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
- **Ownership:** `h3-content` and `h3-pack` stay with battle until further notice (user, 2026-10-07); `h3-kernel` and `h3-contracts` are shared, so a seam change is its own `shared/` pull request, merged before the battle change that uses it.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

**Code reading**
- The executable's x87 runs at 53 bits (`installation.md` §7.10): emulate it with `h3_kernel::arith::X87` and test against vectors from exact rational arithmetic; check each float constant's width (`qword` double or `dword` f32).
- Name an address only after reading its code: §7.15 had called 0x469E50 (the moat's damage) a Quicksand test.

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off). `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. Two `skill_specialty` effects in one specialty are rejected.

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- Code added to `movement::reachable_within` can push `fits` out of line (+1 % Ir in T5c1; it is `#[inline]` now): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-07/08 (thirtieth) — E04b T5b: Remove Obstacle, obstacles as pieces, BAT-QUIRK-3/4

Cloud session, linked to the computer at first, branch `battle/e04b-t5b`. BAT-Q5: finish T5 in one `cross-lane` PR; Q55 fix-optional with option (a), BAT-Q4 fix-optional. The user left after the decisions (the original files had been staged): unattended from there. `BattleSetup.obstacles` as `Obstacle { hexes, anchor }` (ADR-0014 point 10 amended), `h3_battle::natural`, BAT-MAG-034, BAT-QUIRK-3/4; 0x466B30 re-read (the clearing). Pack format v16, ruleset hash v25, state hash v28, replay format v18; `BattleState` 4 304 bytes; old workload +0.28 %, new −0.61 % (re-baselined); replays re-recorded, a sixth (`full-s3-a138`). An independent review found eight points, all fixed. Detail: `docs/epics/E04b-log.md` (T5b).

### 2026-10-07 (twenty-ninth) — E04b T5c2: Land Mine; the Q55 survey

Linked cloud session, branch `battle/e04b-t5c2`. BAT-Q3: T5c2 and the Q55 survey, a checkpoint after. Reading `installation.md` §7.17 (the book's refusal by the natives byte, the handler, the trigger after Fire Wall's; the walker walks on). Pack format v15 (kind `mine`), BAT-MAG-033; state and ruleset hash schemes unchanged; `BattleState` 4 064 bytes; old workload −0.07 %, new −4.26 % (mines shorten games, +0.54 % per step); replays re-recorded (`full-s3-a246` replaces `full-s3-a5`), `bench-check` re-baselined. A mine that does not go off reports nothing, not even a split move. Q55: no community source on the anchors (wiki, HotA changelog, VCMI, df2.ru). Detail: `docs/epics/E04b-log.md` (T5c2).

### 2026-10-07 (twenty-eighth) — E04b T5c1: Quicksand and hidden obstacles

Linked cloud session, branch `battle/e04b-t5c1`. BAT-Q1: T5c1 alone, a checkpoint before T5c2; the natives' sight (BAT-QUIRK-1) and an attack from a seen pit (BAT-QUIRK-2) fix-optional; ADR-0014 point 7 amended in `shared/adr-0014-natives` (PR #3). Reading `installation.md` §7.16 (0x469E50 is the moat's). Pack format v14, BAT-MAG-031/032, state hash v27, ruleset hash v24; old workload −0.20 %, new +7.13 % (+6 % steps); replays (a fifth, `full-s3-a5`) and `bench-check` re-baselined. Detail: `docs/epics/E04b-log.md` (T5c1). The user's batch 2c results folded in on the same branch: all as predicted (H1's round 3 after the user's correction, BAT-Q2); Q55's anchors confirmed.
