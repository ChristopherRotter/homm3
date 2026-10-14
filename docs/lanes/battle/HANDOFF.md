# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E04c increment 5, T7 (arrow towers):** built on `battle/e04c-t7` (2026-10-14; BAT-Q44; BAT-SIE-016 to 019; batch 3e: as the code, S-06/S-07 refuted); BAT-Q45 open (the automatic turret's target); the `cross-lane` pull request awaits the user's checkpoint.
- **Merged:** up to #199.
- **Open for later tasks:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q45 (none blocks), BAT-STUB-1, BAT-STUB-2 (T8), `QUEUE.md`'s inbox, Q43; S-23, S-24 and batch 2g later.
- **Versions** (T7): battle state hash v48, battle part hash v5, replay format v27, action encoding v13, C ABI v22, observation encoding v18; shared: core ruleset hash v43, pack format v40, protocol v31. Next ids: BAT-Q46, BAT-QUIRK-23, BAT-STUB-3, BAT-TER-007, BAT-SIE-020; survey D-20, S-30.
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
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent, Catapult), not in `BattleSetup.war_machines`' order; then a town's turrets (keep, upper, lower), off the field (254, 255, 251): a rule that walks stacks for targets or distances leaves them out (`Stack::is_turret`).
- Defend spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start`). A test ruleset may not make bad morale certain (254/255).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off; no Berserk in its books, BAT-Q31); the automatic Ballista reads `movement::reachable_in_public`. `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- A dead stack's timed effects stay until the command's end (`effects::remove_dead`), its permanent ones for good: code that brings a stack back within a command removes the timed ones first (the rebirth), and a reused record is cleared with `effects::forget`. The battle's first activation runs without the freezing checks (`turn::activate_first`); good morale goes through `turn::morale_turn`.
- Hero spells and masteries come from `h3-heroes` (ADR-0013 point 2); the battle reads `BattleMagic` only. Tests: `support::masteries(r, [schools])`. A stack's spell test needs the casting side (`affects_stack`, `share`, `resists`: the heroes' `Guard`, BAT-MAG-066); battle-wide hero magic is `BattleState.limits`, derived at the start.
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. `Stack.spells` holds the spells' values only: a creature's `innate_spell` is read where the original reads the creature (`magic::mirror_chance`, `magic::reflection`), not by Hypnotize, Dispel or usefulness tests. Two `skill_specialty` effects in one specialty are rejected.
- A two-hex stack's position is its rear hex (BAT-GEO-002): where VCMI measures at its head, use `Footprint::front` (placement, a shot's aim, the berserk shooter); a shot's distance is `footprint_distance` (the original turns its shooter, BAT-Q36); `hexes()` keeps the front hex first. Tests place two-hex stacks by the rear hex.
- Relations go by the acting stack's controller and the other stack's own side (`Stack::may_attack`; `is_hostile_to` only for blocking and multi-headed strikes).
- A town's gate opens in `obstacles::moved`: a quiet sink moves a walker in one step, so only a recording sink sees it open mid-walk; a rule reading the gate mid-walk belongs in `travel_through`.
- A retired quirk flag keeps its `REGISTRY` position without a name (`flags()` lists the live ones). A flag that comes or goes changes the faithful ruleset hashes: the golden replays, and the adventure lane's faithful replays (a `cross-lane` header line).

- The observation is complete since T9a: a new state field a rule reads goes into `observe_as` (public, own part or neither: ADR-0014 point 8), the flat encoding (version and ABI bump) and the hygiene property.
- `h3-protocol`'s hygiene tests compare a viewer's frames across seeds on the `full` scenario up to the first hidden cast: no draw the viewer sees may come before it, public or its own (T9b: resistance on Brea's stacks, Eagle Eye listing Mire or Snare).

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (T5c1, T6b): compare per-function profiles (`callgrind_annotate --inclusive=no`) on the old workload.
- The random agent and `legal_mask` ask for the active stack's cast at every step: keep `casters::ready` a cheap early exit (casts left first; the targets' buffer only for a caster cost +2 % Ir in T7b1).
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks faster; the old commit in a `git worktree`, own target dir) and per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, the out file's `summary:`; `h3 bench` where `packs/` resolves).

**Process**
- `xtask` takes the workspace root from its build: never share a target directory between worktrees (two workers' checks read each other's tree, T11b).
- The user merges with merge commits or squashes (#164: merge master into a stacked branch, keep its side of its own files); after a rebase-merge, `git rebase --onto origin/master <old base head>`. Fetch master again before opening a pull request that bumps a shared version (#167 took protocol v27).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- Agent runs (`experiments/agent/`): one map per encounter (`--only`); the random starting bonus may arm the hero: `opening.py` replays such a battle. Sieges: `siege.py` (hot seat, both armies) counts rounds by the turn flags +0x4F0, `catapult.py` by the acted bits (the turn flags' clear moment can be shorter than a step). A map's draws repeat exactly: a rerun is no new sample. A hero's spell book fails the battle screen's probes. A worktree runs the harness through a symlinked `original-data`.
- The `.h3m` writer: terrains, water (coast flag), roads, piles, grounds, towns (`docs/experiments/README.md`). A hero type once per map, eight heroes per player. Wandering monsters act on their own: use a Blue hero to hold still. Check moves against speed (14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn spoils a count. Every hero has a Catapult.

## Recent sessions

### 2026-10-14 (sixty-third) — E04c T7: the arrow towers (increment 5)

Reading §7.30 (0x465AD0 against the wiki's S-06/S-07); BAT-Q44; the battle (BAT-SIE-016 to 019); batch 3e by an agent (`turrets.py`): as the code, the keep's automatic target as the wiki's (BAT-Q45). Detail: `docs/epics/E04c-log.md`.

### 2026-10-14 (sixty-second) — E04c T6: master merged, the full verify, the pull request

Master merged (`ballistics` code 43, pack format v39); the verify's gaps (coverage, rustls 0.23.46); PR #199. Detail: `docs/epics/E04c-log.md`.

### 2026-10-14 (sixty-first) — E04c T6: the Catapult, Ballistics and Earthquake (increment 4)

Seam #191 (BAT-Q43); reading §7.29; the battle (BAT-SIE-009 to 015, HER-BAT-018, BAT-QUIRK-22); batch 3d: as the code. Detail: `docs/epics/E04c-log.md`.
