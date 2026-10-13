# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E04c increment 2** (the user, 2026-10-13): T3 in **#179** (`cross-lane`: `h3-map`'s writer): batches 3a and 3b run by an agent; D-1, D-2, D-4 as the code, the defender's tile decides; BAT-Q39: S-28 not a quirk. T4 in **#180** (stacked on #179): the siege survey, §7.27, the ADR draft, accepted at BAT-Q40. Next: T5 (`QUEUE.md`).
- **Merged:** #164, #169 (CLT-Q1, wire 29), #170, #171. E04b closed 2026-10-12.
- **Open for later tasks:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35 (none blocks), BAT-STUB-1, `QUEUE.md`'s inbox, Q43; S-23, S-24 and batch 2g later.
- **Versions** (unchanged by T3): battle state hash v45, battle part hash v2, replay format v25, action encoding v12, C ABI v19, observation encoding v15; shared: core ruleset hash v43, pack format v36, protocol v29. Next ids: BAT-Q41, BAT-QUIRK-22, BAT-STUB-2, BAT-TER-007; survey D-8, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches are played by the user or an agent.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated (`core.toml`, `battle.toml`): change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

- A data quirk's value stays in the pack; its fix is an override gated by a `quirk.content` flag, read only by the compiler (ADR-0007).
- Tests patch the synthetic pack by text (`h3-pack-battle`'s `Texts`): each patch asserts it matched; a text in both files names its file. To pick replay seeds, count events per agent seed with a throwaway test in `crates/h3-cli/tests` (`run_random`, then re-apply the commands with an event sink).

**Code reading**
- Edit docs with targeted replacements, never by rewriting a file from memory (T7a2 lost `QUEUE.md`'s inbox so).
- The executable's x87 runs at 53 bits (`installation.md` §7.10): emulate it with `h3_kernel::arith::X87` and test against vectors from exact rational arithmetic; check each float constant's width (`qword` double or `dword` f32).
- Name an address only after reading its code: §7.15 had called 0x469E50 (the moat's damage) a Quicksand test.

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off; no Berserk in its books, BAT-Q31); the automatic Ballista reads `movement::reachable_in_public`. `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- A dead stack's timed effects stay until the command's end (`effects::remove_dead`), its permanent ones for good: code that brings a stack back within a command removes the timed ones first (the rebirth), and a reused record is cleared with `effects::forget`. The battle's first activation runs without the freezing checks (`turn::activate_first`); good morale goes through `turn::morale_turn`.
- Hero spells and masteries come from `h3-heroes` (ADR-0013 point 2); the battle reads `BattleMagic` only. Tests: `support::masteries(r, [schools])`. A stack's spell test needs the casting side (`affects_stack`, `share`, `resists`: the heroes' `Guard`, BAT-MAG-066); battle-wide hero magic is `BattleState.limits`, derived at the start.
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. `Stack.spells` holds the spells' values only: a creature's `innate_spell` is read where the original reads the creature (`magic::mirror_chance`, `magic::reflection`), not by Hypnotize, Dispel or usefulness tests. Two `skill_specialty` effects in one specialty are rejected.
- A two-hex stack's position is its rear hex (BAT-GEO-002): where VCMI measures at its head, use `Footprint::front` (placement, a shot's aim, the berserk shooter); a shot's distance is `footprint_distance` (the original turns its shooter, BAT-Q36); `hexes()` keeps the front hex first. Tests place two-hex stacks by the rear hex.
- Relations go by the acting stack's controller and the other stack's own side (`Stack::may_attack`; `is_hostile_to` only for blocking and multi-headed strikes). A created stack that ran out of rounds keeps its record (`created::reusable`).
- A retired quirk flag keeps its `REGISTRY` position without a name (`flags()` lists the live ones). A flag that comes or goes changes the faithful ruleset hashes: the golden replays, and the adventure lane's faithful replays (a `cross-lane` header line).

- The observation is complete since T9a: a new state field a rule reads goes into `observe_as` (public, own part or neither: ADR-0014 point 8), the flat encoding (version and ABI bump) and the hygiene property.
- `h3-protocol`'s hygiene tests compare a viewer's frames across seeds on the `full` scenario up to the first hidden cast: no draw the viewer sees may come before it, public or its own (T9b: resistance on Brea's stacks, Eagle Eye listing Mire or Snare).

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (T5c1, T6b): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- The random agent and `legal_mask` ask for the active stack's cast at every step: keep `casters::ready` a cheap early exit (casts left first; the targets' buffer only for a caster cost +2 % Ir in T7b1).
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Process**
- `xtask` takes the workspace root from its build: never share a target directory between worktrees (two workers' checks read each other's tree, T11b).
- The user merges with merge commits or squashes (#164: merge master into a stacked branch, keep its side of its own files); after a rebase-merge, `git rebase --onto origin/master <old base head>`. Fetch master again before opening a pull request that bumps a shared version (#167 took protocol v27).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- Agent runs (`experiments/agent/`): one map per encounter (`--only`); the random starting bonus may arm the hero: `opening.py` replays such a battle (for the global HANDOFF at the next shared change). A worktree runs the harness through a symlinked `original-data`.
- The `.h3m` writer: land terrains, water patches (with the coast flag), roads, piles, special grounds (`[[map.ground]]`). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (14 columns apart at the start). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-13 (fifty-seventh) — E04c T3: the battlefield experiments run by an agent

The user's cut (T3, then T4). T3: the writer's grounds and coast flag, `--only`; batches 3a and 3b run on the harness after batch 1a's R1 matched (#179). T4: the siege reading, survey and ADR draft; BAT-Q40 asked. Detail: `docs/epics/E04c-log.md`.

### 2026-10-13 (fifty-sixth) — E04c refined; T1 and T2, the battlefields

The user's answers (BAT-Q37); T1 (the readings, the survey, the ADR draft; #164); T2 built on the draft while the user was away, then adjusted to BAT-Q38 (ADR-0022, S-28); protocol v28 after master's v27. Detail: `docs/epics/E04c-log.md`.

### 2026-10-13 (fifty-fifth) — the wiki checks; the shot's distance; the user's answers

The wiki checks, BAT-Q36 (#156); the Faerie Dragon's readings, BAT-QUIRK-1 retired (#162).
