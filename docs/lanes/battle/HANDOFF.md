# Battle lane — HANDOFF

The battle lane's living state (`docs/08-work-process.md`, "Lanes"). Read after the global `docs/HANDOFF.md`. Rewrite **Current state** each session, keep **Standing traps** to battle traps that still apply (traps of every lane go into the global HANDOFF, a shared change), add one short entry (about ten lines) to **Recent sessions** and keep at most three. `cargo xtask check-docs` enforces 10 000 bytes and three entries. Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `e04b-battle-magic` (= `master` `b7a5db2` when Phase 0 started). After Phase 0 merges, rebase onto `master` and continue on `battle/<task>` branches, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`.
- **E04b increment 3 (Q54, Q56):** T5a done; batch 2b folded in (all six encounters as predicted); batch 2c compiled and with the user; T5b (Remove Obstacle) waits for its A1–A3 (Q55); T5c next. Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), Anti-Magic and Magic Mirror (BAT-MAG-021/022), Forgetfulness and Blind (BAT-MAG-023/024), Berserk and Hypnotize (BAT-MAG-025/026; a stack's `controller()` commands it), Force Field, Fire Wall, Teleport and Expert Dispel's clearing (BAT-MAG-027 to 030; `h3_battle::obstacles`), all after the executable's code (`installation.md` §7.8–§7.15). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the user's checkpoint (Q56 planned T5c: Quicksand, Land Mine, hidden, ADR-0014 point 7) while the user plays batch 2c (`experiments/specs/E04b-batch-2c.toml`; the map is in the user's `experiments/out/`); when its results are in, fold them in, put Q55's options to the user, then T5b.
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v26, ruleset hash v23, replay format v17, action encoding v10, C ABI v10, observation encoding v6; pack format v13. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
- **Ownership:** `h3-content` and `h3-pack` stay with battle until further notice (user, 2026-10-07); `h3-kernel` and `h3-contracts` are shared, so a seam change is its own `shared/` pull request, merged before the battle change that uses it.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. Two `skill_specialty` effects in one specialty are rejected.

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-07 (twenty-seventh) — batch 2b folded in, batch 2c compiled

Linked cloud session. Q56: the fold-in and batch 2c only, a checkpoint after (T5c next). Batch 2b: every encounter as the engine's faithful prediction (53 bits, Q-0008, Q-0010, Q47, Q45 (b)); observation sources on nine rules, the ledger, fixtures; no behaviour change. Batch 2c: Q55 (A1–A3), Fire Wall (F1, F2), Berserk (B1), Hypnotize and Cure (H1, C1). Detail: `docs/epics/E04b-log.md`.

### 2026-10-07 (twenty-sixth) — E04b T5a, T5b in progress

Linked cloud session. Q54: T5a + T5b, then a checkpoint; S-17 and the T4c decisions kept. T5a done (`installation.md` §7.15; pack format v13; BAT-MAG-027 to 030, BAT-MOV-003; old workload +0.58 %; replays and `bench-check` re-baselined). T5b waits: the user wants the anchor hexes (Q55) observed first. Detail: `docs/epics/E04b-log.md` (T5a).

### 2026-10-07 (twenty-fifth) — E04b T4b and T4c: Forgetfulness, Blind, Berserk, Hypnotize

Linked cloud session. Q53: T4b + T4c; after the reading (`installation.md` §7.14) S-09 the rule (no flag), S-10 fix-optional (Q-0011), S-11 unconfirmed, S-13 fix-optional (Q-0012). T4b: pack format v11, BAT-MAG-023/024, `StackFlags::STARTED`, state hash v25; old workload −0.08 %. T4c: pack format v12, BAT-MAG-025/026, `Stack::controller()`, the berserk turn (`control.rs`), Q-0011/Q-0012, ruleset hash v22; old workload +0.68 %. Replays and `bench-check` re-baselined. Detail: `docs/epics/E04b-log.md` (T4b, T4c).
