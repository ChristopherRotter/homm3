# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a battle-lane crate of its own, the code reading first with batch rows only for readings at risk. **T1a1 to T1a3d done** (#288 to #305 merged; `installation.md` §7.34 to §7.39). **T1a4a** (#307, open for review) and **T1a4b** (stacked on it: `battle/e23-t1a4b`, open for review; rebase it onto `master` when #307 merges) done: the pathfinder read (§7.40, §7.41), S-11 placed. **BAT-Q53** (the user): pathfinding exactly as the original's. **Next:** T1c, the walks and the berserk turn aligned (QUEUE; code, a full verify; ask S-11's class at its start); then T1a5, T1b; T1a6 before T5.
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

### 2026-10-20 (eighty-third) — E23 T1c, pathfinding as the original's (in progress)

#307 and #311 had merged. The user: T1c whole (over the walks first), S-11 a fix-optional quirk (BAT-QUIRK-24, BAT-Q54; the flag on `shared/e23-t1c-quirk`, merged into `battle/e23-t1c`; the adventure lane's faithful replays re-recorded, `cross-lane`, agreed). In progress: `pathfinder.rs` (0x4B3400 ported: the walks' paths and the berserk turn's map and walk), `control.rs` rewritten on it.

### 2026-10-20 (eighty-second) — E23 T1a4b, the pathfinder's head; BAT-Q53

After #307 the user decided that pathfinding must work exactly as the original's (BAT-Q53) and chose T1a4b first, then the walks and the berserk turn aligned together (T1c, queued; the inbox's BAT-MOV-003 and BAT-MAG-025 items folded in). Read in full (§7.41): the head, `stack_automatic` 0x474BF0 (the war machines' cases, auto combat and the four combat settings), 0x4B1460, `walker_path` and `cursor_path` (range the speed). S-11 placed: an attack ends only at the target's position hex, so a two-hex target reachable only by its other hex is a candidate the berserk walk never reaches (it goes round or does nothing); §7.40's "not in the search" corrected. Branch `battle/e23-t1a4b` stacked on #307. A fresh agent re-read three findings blind (the Ballista's and the Ammo Cart's cases of 0x474BF0, a walk's range, the attack's end and the berserk turn's empty path) and agreed on all three.

### 2026-10-20 (eighty-first) — E23 T1a4a, the pathfinder's search

#305 had merged at the start. T1a4 measured about 4.9 KB (2 KB of it the library's vectors); the user chose the search first (T1a4b the head, about 1 KB); the path's trace came along, as the tie order needs it. Read in full (§7.40): 0x4B3400's search with the push 0x4B3BB0: a uniform-cost search, first in first out at a tie, neighbours from top right clockwise, a hex keeping its first cheapest discoverer; stops cost the rest of the turn; the attack marks; flyers passing over; the trace. Simulated on batch 3c's fields it gives M2's and G4's observed walks, which BAT-MOV-003 (from the end, right first) does not: the inbox, for the user's call. 14 map entries (13 new), the `pathfinder` and `pathfinder_cell` records. A fresh agent re-read three findings blind (the queue's order at a tie, the neighbours' order and a cell's replacement, the trace and the result) on a copy of the map before the edits and agreed on all three.

### 2026-10-20 (eightieth) — E23 T1a3d, the hero's value in the odds

#304 had merged at the start. The cloud session could clone the private repository only once it was attached to the session (`add_repo`; a plain clone asked for credentials): a trap of every lane for the global HANDOFF. The user chose all of T1a3d in one part (over the record and the loop first, recommended, or a spell's value first). Read in full (§7.39): the caster record 0x526D40 (its army list's vector and sort recognised as Visual C++ 5/6's), the best spell 0x5275B0, a spell's value 0x5273D0 with 0x5270E0, 0x5271C0, 0x527290, the tile's field 0x4E4FA0 (H3API's name confirmed) with 0x4FCF40, the curve 0x640398. Found: the odds count only the damage spells with the AI bit `0x8000` (not Chain Lightning, Inferno or the mass ones), worth `min(10 d φ_n, S × Y(10 d / S))`, at most four times the side's whole strength; the mana is the hero's maximum, not the points left; only its own Recanter's Cloak counts. 16 map entries (13 new; 0x526D40 and 0x5275B0 renamed and read, 0x420A80 updated). A fresh agent re-read three findings blind (the record's power and mana, the best spell's tests and floor, the damage value's formula with its row at a threshold) on a copy of the map before the edits and agreed on all three.
