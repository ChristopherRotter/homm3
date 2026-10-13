# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **The rear hex as a two-hex stack's position waits for the user's checkpoint** (2026-10-13, `battle/rear-hex-position`, `cross-lane`; BAT-GEO-002): a renaming, the same games; its calls are in the pull request.
- **R01 is merged** (#137, #141; `docs/epics/R01-content-split.md`): the lane owns `h3-content-battle` (`BattleRuleset`, derefs to the core) and `h3-pack-battle`; `h3-content`, `h3-pack` are shared.
- **E04b is closed** (2026-10-12; `docs/epics/E04b-log.md`): battle magic after the executable's code (`installation.md` §7.8–§7.24); physical damage in three truncated stages under Q-0005 (`h3_kernel::arith::X87`, 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points, plus the original's f32 bits under Q-0005; Q45 (b)).
- **Next, at the user's checkpoint:** E04c's refinement (sieges, special battlefields); batch 2g when the user plays it.
- **Open for later tasks:** BAT-Q28, BAT-Q32 to BAT-Q35 (none blocks), batch 2g and 49 wiki checks (`docs/experiments/README.md`), BAT-STUB-1 (an expired created stack in the observation), `QUEUE.md`'s inbox by destination, Q43.
- **Versions:** battle owns state hash v42, battle part hash v1, replay format v23, action encoding v12, C ABI v18, observation encoding v14; shared: core ruleset hash v43, pack format v35. Next ids: BAT-Q36, BAT-QUIRK-22, BAT-STUB-2.
- **Ownership:** `lanes.toml`; a seam change (`h3-kernel`, `h3-contracts`) is its own `shared/` pull request, merged first.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; the user plays experiment maps in SoD with plain HD and commits the results.

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
- A two-hex stack's position is its rear hex (BAT-GEO-002): where VCMI measures at its head, use `Footprint::front` (placement, a shot's aim and distance, the berserk shooter); `hexes()` keeps the front hex first. Tests place two-hex stacks by the rear hex.
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
- The user merges with merge commits; after a rebase-merge, rebase a stacked branch with `git rebase --onto origin/master <old base head>` and push with a lease.

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (14 columns apart at the start). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-13 (fifty-fourth) — the rear hex as a two-hex stack's position

The user left the choice to the agent: the queued rear-hex change. `Footprint { position, other }`; encodings, replays and protocol bumped; 800 random battles and the golden seeds play master's games. Detail: the pull request.

### 2026-10-12 to 13 (fifty-third) — R01, the content split

R01: the writer moved (#137); the content, the pack adapter and the pack files split (#141); behaviour unchanged, protocol v21. Detail: the design note.

### 2026-10-11 to 12 (fifty-second) — E04b's review and closure

Eleven reviews, 24 fixes, E04b closed (BAT-Q29 to BAT-Q31 answered). Detail: `docs/epics/E04b-log.md`.
