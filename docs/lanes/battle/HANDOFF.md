# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a battle-lane crate of its own, the code reading first with batch rows only for readings at risk. **T1a1 and T1a2 done** (#288, #291 merged; `installation.md` §7.34, §7.35). The user cut T1a3 in three (2026-10-20): **T1a3a done** (2026-10-20, `battle/e23-t1a3a`, docs only; its pull request for the user's review): the hex scores 0x420260 and 0x41FB60 with 0x41FD60 and the area threat 0x41F920 (§7.36). **Next:** T1a3b, the plan's hexes and the order's sort (QUEUE; stage `Heroes3.exe`), then T1a3c; T1a4, T1a5, then T1b; T1a6 before T5.
- Since E04c's closure (2026-10-16): #265, #267, #268, #273, #277, #282, #283 merged. E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q51 (none blocks; BAT-Q33 and BAT-Q51 settle with E23's T6), BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43; S-23, S-24, batch 2g.
- **Versions** (unchanged since T10b): battle state hash v51, battle part hash v6 (the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v45, pack format v43, protocol v33. Next ids: BAT-Q53, BAT-QUIRK-24, BAT-STUB-4, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (the user, 2026-10-19).

## Standing traps

Area traps live where they are used: experiments in `docs/experiments/README.md`, replays in `fixtures/README.md`, packs in `packs/README.md`, code reading in the global HANDOFF and `docs/exe/README.md`.

**Battle code**
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
- A quirk flag that comes or goes (a retired one keeps its `REGISTRY` position) changes the faithful ruleset hashes: the golden replays and the adventure lane's faithful replays (`cross-lane`).
- Golden replays and `benches/battle/` change with behaviour, the synthetic pack, an encoding or the toolchain: regenerate deliberately, the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (cold per-stack data goes beside them: `BattleState.raised`, `.created`); code in `movement::reachable_within` or `casters::ready` swings inlining. Before re-baselining, measure `master` in a worktree (own target dir) and compare per-function callgrind profiles. `random-rollouts-full` was at +1.34 % on `master` (2026-10-19).

**Process**
- Edit the long docs (`installation.md`, the catalog) by targeted replacements, never by rewriting from memory.
- The user merges with merge commits or squashes: rebase a stacked branch with `git rebase --onto origin/master <old base head>`; fetch `master` again before a pull request that bumps a shared version.

## Recent sessions

### 2026-10-20 (seventy-seventh) — E23 T1a3a, the hex scores

A size sweep showed about 10.7 KB left in T1a3; the user chose "the hex scores only" (T1a3b the plan and the order's sort, T1a3c the fallbacks and ends). Read in full (§7.36): 0x420260 (an enemy that can strike any of the side's stacks this turn adds no reach; the map with speed + 1), the multi-headed branch 0x41FD60 with 0x420060 and 0x448AB0 (Hard and above, computer sides only; the value's sign untested), 0x41FB60 (crowding, breath lines) and 0x41F920, which is the area threat, not the hero's spell threat §7.35 named (reworded); 8 map entries, three renamed. In progress: a fresh agent re-reads three findings blind (on a copy of the map without the new summaries); the full verify runs; then the pull request. The PC's Python is 3.10: `exe.py` runs only in the cloud workspace (3.11+).

### 2026-10-20 (seventy-sixth) — E23 T1a2, the shooters' rest and the walker's choice

The user confirmed T1a1's cut (T1a2 as queued, over "part S only"; T1b after T1a5). Read (§7.35): part S in full (the area value 0x41EDA0, the area's hexes 0x5A4170 without the side columns, the Cyclops' wall shot 0x41EEA0) and 0x421680 in full (hex scores, the plan per enemy, the draw of 75 to 100 %, the best's keys, the actions) with its helpers by role; 28 map entries. Found: the defender's walls test misreads its records (0x42142F); the best's comparison divides only the kept best by its turns. A fresh agent re-read three readings blind and agreed on all three.

### 2026-10-20 (seventy-fifth) — E23 T1a1, the entries and the map

T1a was too large for one small increment; the user chose T1a1, the inventory and the dispatch. Read (§7.34): the hero's cast at every activation (0x422DA0, from the controls 0x4782D0), the stack's action 0x477EE0 (the computer always retreats, never surrenders; the human's auto-combat questions), the stack's turn, a shooter's tactics placement, a walker's turn (no Wait below Hard), the order of action 0x41F140. A sweep of the calls mapped the rest into seven parts (about 45 KB unread); 29 map entries. Found: the surrender's price leaves out temporary creatures (inbox); `exe.py switch` is one off at the dispatcher (inbox, a shared fix). Proposed cut T1a2 to T1a6, T1b after T1a5: the user's checkpoint.
