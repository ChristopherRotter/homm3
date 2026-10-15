# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E04c increment 10, T10b (2026-10-15):** `battle/e04c-t10b`: the last two reviews (r6, security), the fold-in of all ten reports (BAT-Q50), the replays re-recorded; reports on `battle/e04c-t10b-review-reports` (backup, not for merging). **Next:** the user's checkpoint, then T10c (the inbox's "E04c T10b" bullet).
- **Open:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35 (none blocks), BAT-STUB-1, the inbox, Q43; S-23, S-24, batch 2g.
- **Versions** (T10b): battle state hash v51, battle part hash v6, replay format v29, action encoding v13, C ABI v25, observation encoding v21; shared: core ruleset hash v45, pack format v43, protocol v33. Next ids: BAT-Q51, BAT-QUIRK-24, BAT-STUB-4, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated (`core.toml`, `battle.toml`): change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

- A data quirk's value stays in the pack; its fix is a `quirk.content`-gated override the compiler reads (ADR-0007).
- Tests patch the synthetic pack by text (`h3-pack-battle`'s `Texts`): each patch asserts it matched; a text in both files names its file. To pick replay seeds, count events per agent seed with a throwaway test in `crates/h3-cli/tests` (`run_random`, then re-apply the commands with an event sink).

**Code reading**
- Edit docs with targeted replacements, never by rewriting a file from memory.
- The executable's x87 runs at 53 bits (`installation.md` §7.10): emulate it with `h3_kernel::arith::X87` and test against vectors from exact rational arithmetic; check each float constant's width (`qword` double or `dword` f32).
- Name an address only after reading its code (§7.15 had misnamed 0x469E50).

**Battle code and tests**
- Stack ids: war machines follow the army in the original's record order (Catapult, Ballista, Tent, Ammo Cart: `state::RECORD_ORDER`, D-23), not in `BattleSetup.war_machines`' order; then a town's turrets (keep, upper, lower), off the field (254, 255, 251): a rule that walks stacks for targets or distances leaves them out (`Stack::is_turret`).
- The aim's paths read `BattleSetup.human`: a computer-controlled weak walker keeps out of the moat (BAT-SIE-022), so set `human` in a test of turns at a town.
- The computer's choices (`aim.rs`) read Fight Values, which test creatures lack (`fighting_turret_ruleset`); `BattleState::new` has run the keep's first shot already (`tests/aim.rs`'s `fresh`). It reads a war machine's damage as the original's record (`aim::damage`); its values are small, so ties (the later stack) decide often.
- Defend spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start`).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off; no Berserk in its books, BAT-Q31); the computer's aim (the turrets', the automatic Ballista's) reads only `obstacles::public_pits`. `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- A dead stack's timed effects stay until the command's end (`effects::remove_dead`): code that brings a stack back within a command removes them first, a reused record is cleared with `effects::forget`. The first activation skips the freezing checks (`turn::activate_first`); good morale goes through `turn::morale_turn`.
- Hero spells and masteries come from `h3-heroes` (ADR-0013 point 2); the battle reads `BattleMagic` only. Tests: `support::masteries(r, [schools])`. A stack's spell test needs the casting side (`affects_stack`, `share`, `resists`: the heroes' `Guard`, BAT-MAG-066); battle-wide hero magic is `BattleState.limits`, derived at the start.
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. `Stack.spells` holds the spells' values only: a creature's `innate_spell` is read where the original reads the creature (`magic::mirror_chance`, `magic::reflection`).
- A two-hex stack's position is its rear hex (BAT-GEO-002): where VCMI measures at its head, use `Footprint::front` (placement, a shot's aim, the berserk shooter); a shot's distance is `footprint_distance` (the original turns its shooter, BAT-Q36); `hexes()` keeps the front hex first. Tests place two-hex stacks by the rear hex.
- Relations go by the acting stack's controller and the other stack's own side (`Stack::may_attack`; `is_hostile_to` only for blocking and multi-headed strikes).
- A town's gate opens in `obstacles::moved`: a quiet sink moves a walker in one step, so only a recording sink sees it open mid-walk; a rule reading the gate mid-walk belongs in `travel_through`.
- A retired quirk flag keeps its `REGISTRY` position without a name (`flags()` lists the live ones). A flag that comes or goes changes the faithful ruleset hashes: the golden replays, and the adventure lane's faithful replays (a `cross-lane` header line).

- `properties.rs` plays its games at a `Place` (field, battlefield, garrison, town): what a property assumes must hold at each.
- The observation is complete since T9a: a new state field a rule reads goes into `observe_as` (public, own part or neither: ADR-0014 point 8), the flat encoding (version and ABI bump) and the hygiene property.
- `h3-protocol`'s hygiene tests compare a viewer's frames across seeds on the `full` scenario up to the first hidden cast: no draw the viewer sees may come before it, public or its own (T9b: resistance on Brea's stacks, Eagle Eye listing Mire or Snare).

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (T5c1, T6b); keep `casters::ready` a cheap early exit (+2 % Ir in T7b1). Before re-baselining, compare `h3 bench --games 20` step counts (the old commit in a worktree) and per-function callgrind profiles (`--inclusive=no`, `--toggle-collect='*random_rollout*'`).

**Process**
- `xtask` takes the workspace root from its build: never share a target directory between worktrees (two workers' checks read each other's tree, T11b).
- The user merges with merge commits or squashes (#164: merge master into a stacked branch, keep its side of its own files); after a rebase-merge, `git rebase --onto origin/master <old base head>`. Fetch master again before opening a pull request that bumps a shared version (#167 took protocol v27).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- Agent runs (`experiments/agent/`): one map per encounter (`--only`); a random starting bonus may arm a hero: play again. Rounds by the acted bits (`catapult.py`). A rerun is no new sample (the draws repeat). A hero's spell book fails the battle screen's probes. A worktree runs the harness through a symlinked `original-data`. Predictions: a throwaway test in `crates/h3-cli/tests` on the base pack (every stack defends).
- The `.h3m` writer: terrains, water (coast flag), roads, piles, grounds, towns (`docs/experiments/README.md`). A hero type once per map, eight heroes per player. Wandering monsters act on their own: use a Blue hero to hold still. Check moves against speed (14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn spoils a count. Every hero has a Catapult.

## Recent sessions

### 2026-10-15 (seventieth) — E04c T10b: the last reviews and the fold-in (increment 10)

r6 and security; the user's decisions BAT-Q50 (the original's obstacles and moat avoidance, the tunnel hidden); five fixers; observation v21, ABI v25; siege-s7 and the adventure lane's nine replays redone. Detail: the log.

### 2026-10-15 (sixty-ninth) — E04c T10b: the review's first eight reports (increment 10)

Fresh reviewers in small slices: eight reports (one high, seven medium), nothing folded in. Detail: the log (T10b).

### 2026-10-15 (sixty-eighth) — E04c T10a: the fold-in and the audit (increment 10)

T10 cut as E04b's T11. Wall line in integers (siege −17 % Ir); war machines in 0x463600's order (D-23); the reach test; summaries; the inbox. Detail: the log.
