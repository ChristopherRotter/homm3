# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-t6-checkpoint` (the checkpoint's notes). PRs #7, #8 and #9 (T6) are merged; their branches are gone. Work on `battle/<task>` branches from `master`, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`.
- **E04b increment 4 (the user, 2026-10-08): T6 done (T6a, T6b) and merged.** Increment 3 (T5, batches 2b and 2c) done. Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), Anti-Magic and Magic Mirror (BAT-MAG-021/022), Forgetfulness and Blind (BAT-MAG-023/024), Berserk and Hypnotize (BAT-MAG-025/026; a stack's `controller()` commands it), Force Field, Fire Wall, Teleport and Expert Dispel's clearing (BAT-MAG-027 to 030; `h3_battle::obstacles`), hidden obstacles, Quicksand and Land Mine (BAT-MAG-031 to 033; BAT-QUIRK-1/2/4), Remove Obstacle and the natural obstacles as pieces with anchors (BAT-MAG-034, BAT-GEO-003; `h3_battle::natural`, BAT-QUIRK-3), Resurrection, Animate Dead and Sacrifice (BAT-MAG-035/036; `h3_battle::raise`, BAT-QUIRK-5/6), Clone and Summon Elemental (BAT-MAG-037 to 039; `h3_battle::created`, BAT-QUIRK-7), all after the executable's code (`installation.md` §7.8–§7.19). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the user decided at the T6 checkpoint to settle BAT-QUIRK-7, BAT-Q7 and the clone's shots by experiment (QUEUE inbox); the next increment (that batch, T7 creature magic) is cut with the user.
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v30, ruleset hash v27, replay format v18, action encoding v10, C ABI v11, observation encoding v7; pack format v18. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
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
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice).
- Code added to `movement::reachable_within` can push `fits` out of line (+1 % Ir in T5c1; it is `#[inline]` and tests head and tail without the footprint iterator since T6b, −11 %): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-08 (thirty-second) — E04b T6b: Clone and Summon Elemental

Same cloud session as T6a, branch `battle/e04b-t6b` (from T6a's head). Reading `installation.md` §7.19: the stack creation (a dead created stack's record reused), Clone's test and place search, its rounds and deaths, Summon Elemental's column and row, the winner test (a side left with only created stacks does not win: `Draw`, the doc in PR #7). `h3_battle::created`, BAT-MAG-037 to 039, BAT-QUIRK-7 (a clone on its own original: exploit-fix, agent). Pack format v18, ruleset hash v27, state hash v30, observation v7, C ABI v11; `BattleState` 5 288 bytes. `movement::fits` without the footprint iterator: old workload −11.3 %, re-baselined. Replays re-recorded with new seeds. An independent review found two T6 items left from T3b2/T4a (Anti-Magic on a clone, a clone's Fire Shield reflection) and targets struck after a clone vanished: fixed. Open: BAT-Q7. Detail: `docs/epics/E04b-log.md` (T6b).

### 2026-10-08 (thirty-first) — E04b T6a: Resurrection, Animate Dead, Sacrifice

Linked cloud session; PR #6 had been merged. The user cut T6 (T6a + T6b) before T7 and questioned the `StackOrigin` variants: created stacks cross no seam (ADR-0014 amended, PR #7). Reading `installation.md` §7.18. BAT-QUIRK-5 (Sacrifice tested as Resurrection, fix-optional; S-15), BAT-QUIRK-6 (S-20: one creature kept, counting only army stacks). Pack format v17, ruleset hash v26, state hash v29; a cast that kills the active stack ends its turn; old workload +1.28 %, new +3.45 % (re-baselined); replays re-recorded (a199, a325, a52, a149). Detail: `docs/epics/E04b-log.md` (T6a).

### 2026-10-07/08 (thirtieth) — E04b T5b: Remove Obstacle, obstacles as pieces, BAT-QUIRK-3/4

Cloud session, branch `battle/e04b-t5b`. BAT-Q5: T5 finished in one `cross-lane` PR; Q55 fix-optional (a), BAT-Q4 fix-optional; unattended after the decisions. `BattleSetup.obstacles` as `Obstacle { hexes, anchor }` (ADR-0014 point 10 amended), `h3_battle::natural`, BAT-MAG-034, BAT-QUIRK-3/4. Pack format v16, state hash v28, replay format v18; replays re-recorded, `bench-check` re-baselined. Detail: `docs/epics/E04b-log.md` (T5b).
