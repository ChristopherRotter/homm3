# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a battle-lane crate of its own, the code reading first with batch rows only for readings at risk. **T1 done** (T1a1 to T1a5e, T1b, T1c; #288 to #331): the reading `installation.md` §7.34 to §7.46, `pathfinder.rs` (0x4B3400, BAT-QUIRK-24), the survey `E23-battle-ai.md`, **ADR-0027 accepted** (BAT-Q55). **T2 done** (cut in two, the user, 2026-10-20): **T2a** (`shared/` #334, #338 merged): `h3-battle`'s public `computer` module, `h3_battle_ai::choose` (always legal, property-tested), the plain shooters (BAT-AI-001, BAT-AI-002). **T2b** (whole; #345, #346, #347 merged): BAT-QUIRK-25's flag (position 29, on in faithful), the area value with slip 1 (BAT-AI-003), the Cyclops's wall shot (BAT-AI-004, the policy's first draw), the gated benchmark `computer-decisions`; `POLICY_VERSION` 2. **The budget** (the user, 2026-10-21): the gated count +2 %, re-set on purpose at each part of T3, and ≤ 1 ms for the slowest decision on a quiet sandbox, recorded per task. **T3 cut in four** (the user, 2026-10-21): **T3-0 done** (#350, `cross-lane`: the bots report): each spell's `ai` (the AI categories and values; pack format v45, battle part hash v7). **T3a done** (`battle/e23-t3a`, stacked on #350): the walker's choice (BAT-AI-005; the plan, the melee estimate 0x4359B0 read in full, the draw) and the walk (BAT-AI-006); `POLICY_VERSION` 3; the workload re-set. Watch #350 and T3a's PR until they merge. **Next:** T3b (the hex scores and the hold-back), T3c (the odds and the ends: the stronger side, the last retaliation, the fallbacks, the sieges' walks). T1a6 before T5.
- Since E04c's closure (2026-10-16): #265, #267, #268, #273, #277, #282, #283 merged. E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q51 (none blocks; BAT-Q33 and BAT-Q51 settle with E23's T6), BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43; S-23, S-24, batch 2g.
- **Versions:** battle state hash v51, battle part hash v7 (E23 T3-0; the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v46, pack format v45 (E23 T3-0), protocol v34. Next ids: BAT-Q56, BAT-QUIRK-26, BAT-STUB-7, BAT-AI-005, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (the user, 2026-10-19).

## Standing traps

Area traps live where they are used: experiments in `docs/experiments/README.md`, replays in `fixtures/README.md`, packs in `packs/README.md`, code reading in the global HANDOFF and `docs/exe/README.md`.

**Battle code**
- Paths: a walk's and the berserk turn's go through `pathfinder::Search` (0x4B3400, BAT-MOV-003); `movement::reachable` stays the legality search (the same positions within a move). Its cell cost is a mark or a reach, as the original's.
- Stack ids: war machines follow the army in the original's record order (`state::RECORD_ORDER`: Catapult, Ballista, Tent, Ammo Cart), then a town's turrets, off the field: a rule that walks stacks for targets or distances leaves them out (`Stack::is_turret`).
- A two-hex stack's position is its rear hex (BAT-GEO-002): where VCMI measures at its head, use `Footprint::front`; a shot's distance is `footprint_distance`; `hexes()` keeps the front hex first. Tests place two-hex stacks by the rear hex.
- Relations go by the acting stack's controller and the other stack's own side (`Stack::may_attack`; `is_hostile_to` only for blocking and multi-headed strikes).
- Abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup; `Stack.spells` holds values only (a creature's `innate_spell` is read where the original reads the creature).
- Hero spells and masteries come from `h3-heroes` (ADR-0013 point 2; `cross-lane`); the battle reads `BattleMagic`, battle-wide hero magic `BattleState.limits`. A stack's spell test needs the casting side (`affects_stack`, `share`, `resists`).
- A dead stack's timed effects stay until the command's end (`effects::remove_dead`): code that brings a stack back within a command removes them first (`effects::forget` for a reused record). A stack can die from another's death (a clone with its original): loops over targets fixed before damage skip dead stacks; every path that kills calls `created::died`.
- The first activation skips the freezing checks (`turn::activate_first`); good morale goes through `turn::morale_turn`.
- Hidden information leaks through public events too: a hidden obstacle that does nothing must not change them. A new state field a rule reads goes into `observe_as` (ADR-0014 point 8) and, `cross-lane`, the AI lane's encoding; `h3-protocol`'s hygiene tests fail on a draw a viewer sees before the first hidden cast.
- The computer's walker (`walker.rs`): an attack the engine refuses because its walk stops at the hex (a moat or seen pit under a two-hex walker's other hex) is the original's move without the attack (0x478971): give `Move`. Its map spares its own side and fears the moat without the player test (`computer::map`, `aim::fears_moat`).
- At a town the computer's choices (`aim.rs`: turrets, the automatic Ballista, a weak walker's moat avoidance) read `BattleSetup.human` and Fight Values, which test creatures lack (`fighting_turret_ruleset`); `BattleState::new` has fired the keep's first shot.

**Tests**
- `h3-battle-ai` reads the battle through its public API and `computer` only; its tests build a ruleset in code (`src/tests.rs`), and the harvested property `h3-pack-battle/tests/harvested_computer.rs` (sieges, the special fields, the commanded Catapult's stand-in) is part of its coverage.
- Defend spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start`).
- `legal_commands` lists no casts: a property over legal commands adds `cast_targets` × `second_targets`. `properties.rs` plays at every `Place`.
- Tests patch the synthetic pack by text (`Texts`): each patch asserts it matched.
- A creature the test table lacks: `support::ruleset_with_creatures` adds a test's own, `CREATURES` stays. On the harvested packs a pit lies where Quicksand draws it: loop over seeds (`harvested_control_and_field.rs`).

**Packs, replays, benchmarks**
- `packs/base|ab|sod` are generated: change `h3-harvest` or its gap-fill file and re-run it; harvested keys may coincide across kinds (`base:ballista`).
- A quirk flag that comes or goes (a retired one keeps its `REGISTRY` position) changes the faithful ruleset hashes: the golden replays and the adventure lane's faithful replays (`cross-lane`): patch the headers' `ruleset_hash` (`fixtures/README.md`; re-running `duel-s1`'s or `field-s5`'s spec draws other obstacles).
- Golden replays and `benches/battle/` change with behaviour, the synthetic pack, an encoding or the toolchain: regenerate deliberately, the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (cold per-stack data goes beside them: `BattleState.raised`, `.created`); code in `movement::reachable_within` or `casters::ready` swings inlining. Before re-baselining, measure `master` in a worktree (own target dir) and compare per-function callgrind profiles. `random-rollouts-full` was at +1.34 % on `master` (2026-10-19).

**Process**
- Edit the long docs (`installation.md`, the catalog) by targeted replacements, never by rewriting from memory.
- The user merges with merge commits or squashes: rebase a stacked branch with `git rebase --onto origin/master <old base head>`; fetch `master` again before a pull request that bumps a shared version.

## Recent sessions

### 2026-10-21 (ninety-first) — E23 T3-0 and T3a, the spells' AI values, the plan and the attack

The user: the budget as proposed (the count +2 %, re-set per T3 part; ≤ 1 ms for the slowest decision), T3 cut in four, T3-0 and T3a this session. T3-0 (#350): each spell's `ai` (`SpellAi`: the executable's AI categories, `SPTRAITS.TXT`'s four AI values; pack format v45, battle part hash v7), the golden replays' headers, the AI lane's bots report (`cross-lane`; regenerated again after the adventure lane's T29 merged); `h3-game`'s test constructor too. T3a: the walker (`walker.rs`, `hexes.rs`; `computer` gains the context, worth, damage estimate and value, the strike value, the walker's map and path), the melee half of 0x4359B0 read (§7.47); a planned attack whose walk stops there becomes a move (the original skips the attack, 0x478971). A fresh agent re-read the port against the disassembly. CI cannot start (billing): the full local verify stands in.

### 2026-10-21 (ninetieth) — E23 T2b, the area value, the wall shot, the benchmark

The user: T2b whole. #340 fixed `master`'s tests (#338 and #339 merged together) and the stack-limit invariant. `shared/e23-t2b` (the flag needs the faithful profiles in the same PR: the replays' hashes, the bots' report; `cross-lane`); the battle PR: `aim::shot` with the wall test as a closure, the area value, `h3 bench-computer` (the log).

### 2026-10-20 (eighty-ninth) — E23 T2a, the crate and the plain shooters

The user: T2a now, T2b's shape later. `shared/` #334 (the registration); the battle PR: `computer` (`shoots`, `shot_target`), `choose`/`unchecked`, the shooter, the stand-ins, two property tests (the log).
