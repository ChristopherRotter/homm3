# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-batch-2d-results` (batch 2d folded in). The T7a checkpoint is merged. Work on `battle/<task>` branches from `master`, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`.
- **E04b increment 6 (the user, 2026-10-08): T7a, its checkpoint and batch 2d's fold-in done** (the surrender price fixed, BAT-Q7 answered, BAT-QUIRK-7 fix-optional, BAT-QUIRK-8). Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), Anti-Magic and Magic Mirror (BAT-MAG-021/022), Forgetfulness and Blind (BAT-MAG-023/024), Berserk and Hypnotize (BAT-MAG-025/026; a stack's `controller()` commands it), Force Field, Fire Wall, Teleport and Expert Dispel's clearing (BAT-MAG-027 to 030; `h3_battle::obstacles`), hidden obstacles, Quicksand and Land Mine (BAT-MAG-031 to 033; BAT-QUIRK-1/2/4), Remove Obstacle and the natural obstacles as pieces with anchors (BAT-MAG-034, BAT-GEO-003; `h3_battle::natural`, BAT-QUIRK-3), Resurrection, Animate Dead and Sacrifice (BAT-MAG-035/036; `h3_battle::raise`, BAT-QUIRK-5/6), Clone and Summon Elemental (BAT-MAG-037 to 039; `h3_battle::created`, BAT-QUIRK-7), the creature magic after a melee strike (BAT-MAG-040 to 047; `h3_battle::after_strike`, `vitality`, `binds`), all after the executable's code (`installation.md` §7.8–§7.20). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the user confirms the proposed T7b/T7c cut (epic, T7; `installation.md` §7.21; proposed increment 7 = T7b1 + T7b2, QUEUE), decides T7b1's command shape and where T6c (BAT-QUIRK-7's flag, created stacks' kills in the outcome) goes. T7b3 needs the Pit Lord's outcome decision, T7b4 S-12's class.
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v32, ruleset hash v30, replay format v18, action encoding v10, C ABI v11, observation encoding v7; pack format v21. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
- **Ownership:** `h3-content` and `h3-pack` stay with battle until further notice (user, 2026-10-07); `h3-kernel` and `h3-contracts` are shared, so a seam change is its own `shared/` pull request, merged before the battle change that uses it.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

**Code reading**
- Edit docs with targeted replacements, never by rewriting a file from memory: T7a2's rewrite of `QUEUE.md` dropped its whole inbox (restored at the T7a checkpoint).
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
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (+1 % Ir in T5c1; it is `#[inline]` and tests head and tail without the footprint iterator since T6b, −11 %): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Pull requests**
- The user rebase-merges: once a stacked PR's base is merged, rebase the next branch with `git rebase --onto origin/master <old base head>` and push with a lease (#13 conflicted, 2026-10-08).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-08 (thirty-sixth) — E04b batch 2d folded in

Linked cloud session beside the T7a checkpoint, rebased after it. Batch 2d as the code reading but the elementals' native terrain (natives, as the engine). Fixed: the surrender price leaves created stacks out. The user: BAT-Q7 (b) and (a) (T6c); BAT-QUIRK-7 fix-optional after a community search (flag in T6c), BAT-QUIRK-8 split off. Detail: `docs/epics/E04b-log.md`.

### 2026-10-08 (thirty-fifth) — E04b T7a checkpoint, review, S-14, the T7b reading

Linked cloud session; master's verify green with original data. The user chose: close T7a, then read the casting creatures and propose the cut, then stop; BAT-Q8 not reproduced, Q-0005's extension confirmed. The QUEUE's inbox (lost in 5751e65) restored. Review fixes: a death recomputes the stack (Age's hit points), S-14 read (0x4789F8, 0x43EE10) and fixed after the user's look at community sentiment (BAT-Q9: no flag, a bound stack does not return; blind, petrified, paralysed Harpies stay), test gaps; replays' hashes re-recorded. T7b reading `installation.md` §7.21; the cut proposed in the epic. Detail: `docs/epics/E04b-log.md`.

### 2026-10-08 (thirty-fourth) — E04b T7 cut, T7a1 to T7a3: the creature magic after a strike

Linked cloud session; the baseline verify green with original data. The user cut increment 6 as T7a in parts, batch 2d at its checkpoint. Code reading `installation.md` §7.20. T7a1 (PR #12, merged): `h3_battle::after_strike` (BAT-MAG-040 to 042). T7a2 (PR #13): Stone Gaze, Paralyze (BAT-MAG-043), Poison, Age (BAT-MAG-044, `vitality`; Q-0005 extended), Disease. T7a3: Bind (BAT-MAG-045, `binds`; BAT-Q8), Acid Breath (046), the Rust Dragon's extra damage (047). Replays and bench re-baselined each time. Detail: `docs/epics/E04b-log.md`.
