# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-t5c1` (T5c1, its pull request after `shared/adr-0014-natives`, PR #3). Work on `battle/<task>` branches from `master`, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`. The stale `e04b-battle-magic` and `claude/hopeful-mayer-ppui03` (both in `master`) wait for the user to delete them: the session proxy refuses ref deletion.
- **E04b increment 3 (Q54, Q56):** T5a done; batch 2b folded in (all six encounters as predicted); batch 2c compiled and with the user; T5b (Remove Obstacle) waits for its A1–A3 (Q55); T5c in two parts (BAT-Q1): T5c1 done, T5c2 (Land Mine) after the checkpoint. Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), Anti-Magic and Magic Mirror (BAT-MAG-021/022), Forgetfulness and Blind (BAT-MAG-023/024), Berserk and Hypnotize (BAT-MAG-025/026; a stack's `controller()` commands it), Force Field, Fire Wall, Teleport and Expert Dispel's clearing (BAT-MAG-027 to 030; `h3_battle::obstacles`), hidden obstacles and Quicksand (BAT-MAG-031/032; BAT-QUIRK-1/2), all after the executable's code (`installation.md` §7.8–§7.16). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the user merges PR #3 (ADR-0014 point 7), the T5c1 branch rebases onto it and merges at the checkpoint; then T5c2 (Land Mine, `E04b-design.md` T5). Batch 2c is with the user (`experiments/specs/E04b-batch-2c.toml`); when its results are in, fold them in, put Q55's options to the user, then T5b.
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v27, ruleset hash v24, replay format v17, action encoding v10, C ABI v10, observation encoding v6; pack format v14. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
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
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. Two `skill_specialty` effects in one specialty are rejected.

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- Code added to `movement::reachable_within` can push `fits` out of line (+1 % Ir in T5c1; it is `#[inline]` now): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-07 (twenty-eighth) — E04b T5c1: Quicksand and hidden obstacles

Linked cloud session, branch `battle/e04b-t5c1`. BAT-Q1: T5c1 alone, a checkpoint before T5c2; the natives' sight (BAT-QUIRK-1) and an attack from a seen pit (BAT-QUIRK-2) fix-optional; ADR-0014 point 7 amended in `shared/adr-0014-natives` (PR #3). Reading `installation.md` §7.16 (0x469E50 is the moat's). Pack format v14, BAT-MAG-031/032, state hash v27, ruleset hash v24; old workload −0.20 %, new +7.13 % (+6 % steps); replays (a fifth, `full-s3-a5`) and `bench-check` re-baselined. Detail: `docs/epics/E04b-log.md` (T5c1).

### 2026-10-07 (twenty-seventh) — batch 2b folded in, batch 2c compiled

Linked cloud session. Q56: the fold-in and batch 2c only, a checkpoint after (T5c next). Batch 2b: every encounter as the engine's faithful prediction (53 bits, Q-0008, Q-0010, Q47, Q45 (b)); observation sources on nine rules, the ledger, fixtures; no behaviour change. Batch 2c: Q55 (A1–A3), Fire Wall (F1, F2), Berserk (B1), Hypnotize and Cure (H1, C1). Detail: `docs/epics/E04b-log.md`.

### 2026-10-07 (twenty-sixth) — E04b T5a, T5b in progress

Linked cloud session. Q54: T5a + T5b, then a checkpoint; S-17 and the T4c decisions kept. T5a done (`installation.md` §7.15; pack format v13; BAT-MAG-027 to 030, BAT-MOV-003; old workload +0.58 %; replays and `bench-check` re-baselined). T5b waits: the user wants the anchor hexes (Q55) observed first. Detail: `docs/epics/E04b-log.md` (T5a).
