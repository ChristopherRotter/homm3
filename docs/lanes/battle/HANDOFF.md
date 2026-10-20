# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a battle-lane crate of its own, the code reading first with batch rows only for readings at risk. **T1a1 to T1a4b done** (#288 to #311 merged; `installation.md` §7.34 to §7.41). **T1c done** (BAT-Q53, BAT-Q54; #316, #317 merged): `pathfinder.rs` ports 0x4B3400 for the walks and the berserk turn, S-11 as the fix-optional BAT-QUIRK-24. **T1a5 cut in four** (the user, 2026-10-20: the context first); **T1a5 done as cut** (§7.42 to §7.45; #320, #323, #326 merged). **T1a5e done** (§7.46; #329 merged): the Genie needs 34 evaluators through the removals. **T1b done** (the survey `E23-battle-ai.md`, the ADR draft `adr-drafts/original-battle-ai.md`; #331, docs only, open): **the user's review, BAT-Q55** (decided: one fix-optional flag for all the AI's slips, BAT-QUIRK-25); then T2 (the crate's `shared/` registration first); T1a6 before T5.
- Since E04c's closure (2026-10-16): #265, #267, #268, #273, #277, #282, #283 merged. E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q55 (the review; blocks T2), BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q51 (none of these blocks; BAT-Q33 and BAT-Q51 settle with E23's T6), BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43; S-23, S-24, batch 2g.
- **Versions** (unchanged since T10b): battle state hash v51, battle part hash v6 (the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v45, pack format v43, protocol v33. Next ids: BAT-Q56, BAT-QUIRK-26, BAT-STUB-4, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
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
- At a town the computer's choices (`aim.rs`: turrets, the automatic Ballista, a weak walker's moat avoidance) read `BattleSetup.human` and Fight Values, which test creatures lack (`fighting_turret_ruleset`); `BattleState::new` has fired the keep's first shot.

**Tests**
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

### 2026-10-20 (eighty-eighth) — E23 T1b, the survey and the ADR draft

The user: no split. The survey (the reading in one place, the wiki and VCMI compared: the wiki agrees the computer never surrenders; 18 batch rows, 4a and 4b; the original's slips) and the ADR draft (`h3-battle-ai` in the application layer, `choose`, `h3-battle`'s public `computer` module, `BAT-AI-nnn`, the budget at T2). BAT-Q55 for the review. Also #330 (shared): code reading in the sandbox, not on the user's PC (the user).

### 2026-10-20 (eighty-seventh) — E23 T1a5e, the spell evaluators

#326 had merged. Proposed T1a5e in four parts by shared helper; the user: one part. Read in full (§7.46): the 36 evaluators and nine helpers (the protections, the removal 0x439A80, speed, the exchange, morale and luck). Found: the removals call each removed spell's evaluator, so the Genie needs 34 of the 36, not 21. Oddities kept as read for T1b. 48 map entries. A fresh agent re-read three findings blind.

### 2026-10-20 (eighty-sixth) — E23 T1a5c and T1a5d, T1a5 done as cut

#323 had merged; the user: "complete T1a5". Read in full (§7.44, §7.45): the Ogre Mage's and the Genie's scan (a draw skips three casts in ten when the walker has a value), Bloodlust's and the Genie's values (the mean over its spells), the evaluation with "now" for the hero, the attack bonus's value (the worth's rise for the rounds' share of the battle). 0x437A00 is the stack record's copy. Found: 0x43B690's 36 evaluators, reached only by its table, missing from T1a1's map (T1a5e, proposed). 52 map entries. A fresh agent re-read three findings blind.
