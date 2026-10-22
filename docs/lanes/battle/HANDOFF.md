# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a crate of its own, the code reading first, batch rows only for readings at risk. **T1 done** (#288 to #331): §7.34 to §7.46, `pathfinder.rs` (BAT-QUIRK-24), the survey `E23-battle-ai.md`, **ADR-0027** (BAT-Q55). **T2 done** (#334 to #347): `h3_battle_ai::choose`, the shooters, BAT-QUIRK-25, the benchmark `computer-decisions`. **The budget** (the user, 2026-10-21): the gated count +2 %, re-set on purpose per part; ≤ 1 ms for the slowest decision on a quiet sandbox, recorded per task. **T3 done** (#350 to #359): the walker, BAT-AI-005 to 011, slips 2 to 4, 18 to 21; slip 6 a gap (BAT-STUB-7, BAT-Q56). **T4a done** (#364, #365), **T4b1 done** (#369), **T4b2 done** (#374: the predictions, BAT-AI-015, §7.52; BAT-STUB-10, BAT-Q58). **T4c done** (the user: whole; #379 open: the Master Genie, BAT-AI-016, §7.53, slips 9 to 17, 22, 23; BAT-STUB-4 retired, BAT-STUB-11, 12; `POLICY_VERSION` 9): a side without a hero is complete (the AI and adventure inboxes). Next **T4d** (slip 24, BAT-Q60), then T1a6 and T5.
- E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q51, BAT-Q56 to BAT-Q58 (none blocks; BAT-Q33 and BAT-Q51 settle with E23's T6), BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43; S-23, S-24, batch 2g.
- **Versions:** battle state hash v51, battle part hash v7 (E23 T3-0; the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v46, pack format v45 (E23 T3-0), protocol v34. Next ids: BAT-Q61, BAT-QUIRK-26, BAT-STUB-13, BAT-AI-017, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
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
- The walker's hex scores (`scores.rs`) search each enemy's reach (`computer::reach_map`: speed + 1 as range and turn speed, `Query::speed`); `hexes::stack_neighbor_at` takes a hex index ± 1 as the original's arithmetic.
- The walker's odds (`odds.rs`) and threat records (`gain.rs`) are computed once per decision when first asked (a cell, a once-cell): they must read nothing the choice changes. The flag-1 records need the order of action's ranks (`order.rs`, the library sort above 16 stacks).
- The creature cast entry (`casts.rs`) comes after the walker's loop, before its branches, even without a candidate: `choose` writes the value first; `turn` asks again after nothing. Casts take only the engine's targets (`creature_cast_targets`; a mass version's `None`).
- The predictions (`predict.rs`, §7.52) run whole walker and shooter turns in a predicting context (`Context::predicting`): where the original passes the context's flag to 0x443080 read `Context::left`, where it passes a stack without hit points left over test `Context::spent`. A predicted walk, wait or fallback plans nothing and is not built.
- The spell context's threat records (`computer::threat`, 0x422B20 flag 0: range 127, no rank test, a shooter every enemy at one turn) are not the walker's (`gain.rs`, flag 1). `spells.rs` caches them per decision.
- `spells.rs`: a root (records, halves, tables: the battle's, built once) and `View`s on a state; a removal values on a clone (`View::on`). Slip 22: a cast entry that built a context and cast nothing sets `Walker::held`; the fallbacks read `Walker::cells`, their walks the walker's own map.
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

### 2026-10-22 (ninety-eighth) — E23 T4c, the Master Genie

The user: T4c whole, BAT-Q59 as slip 22; BAT-Q60 (a removal's copy) as slip 24 in T4d. Built: every evaluator, the tables (slip 23), the second context, the removals, slip 22 (BAT-STUB-11); a blind check (two fixes; the log); `computer-decisions` re-set (+8.55 %), the slowest decision 0.98 ms (a Faerie Dragon). The full verify's result is in the PR.

### 2026-10-22 (ninety-seventh) — E23 T4b2 finished

`master` merged; the full verify with the original data passed; #374 opened.

### 2026-10-22 (ninety-sixth) — E23 T4b2, the predictions; the mass-cast listing

T4b2 built, stopped by the user before the PR (the log). The user: T4b2 whole; BAT-STUB-10, BAT-Q58.
