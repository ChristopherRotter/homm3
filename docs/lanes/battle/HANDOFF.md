# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **E23, the original's battle AI, in progress** (`docs/epics/E23-original-battle-ai.md`; refined 2026-10-19, BAT-Q52): the neutral side first, a battle-lane crate of its own, the code reading first with batch rows only for readings at risk. **T1a1 done** (2026-10-20, #288 open: `battle/e23-t1a1`, docs only): the entries, the dispatch and the map of the rest (`installation.md` §7.34). **Next:** the user's checkpoint on the cut T1a2 to T1a6 (the epic file; T1b after T1a5), then T1a2 (QUEUE; stage `Heroes3.exe`).
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

### 2026-10-20 (seventy-fifth) — E23 T1a1, the entries and the map

T1a was too large for one small increment; the user chose T1a1, the inventory and the dispatch. Read (§7.34): the hero's cast at every activation (0x422DA0, from the controls 0x4782D0), the stack's action 0x477EE0 (the computer always retreats, never surrenders; the human's auto-combat questions), the stack's turn, a shooter's tactics placement, a walker's turn (no Wait below Hard), the order of action 0x41F140. A sweep of the calls mapped the rest into seven parts (about 45 KB unread); 29 map entries. Found: the surrender's price leaves out temporary creatures (inbox); `exe.py switch` is one off at the dispatcher (inbox, a shared fix). Proposed cut T1a2 to T1a6, T1b after T1a5: the user's checkpoint.

### 2026-10-20 (seventy-fourth) — E23 refined

The user chose E23's epic file as the next small increment (after #277 merged). The agent proposed four refinement questions; the user decided (BAT-Q52): the neutral side first; a crate of its own (the agent had recommended a module of `h3-battle`); the reading first with spot checks (the agent had recommended capturing every decision); T1 split into T1a and T1b. `shared/e23-refinement` (lanes.toml's `docs/epics/E23*`, BACKLOG, 08's ownership line) merges first; `battle/e23-refinement` stacks on it (the epic file, BAT-Q52, this HANDOFF, the QUEUE).

### 2026-10-19 (seventy-third) — CLT-Q2's LOD reader; three test gaps

Small increments at the user's choice. #273 (merged), CLT-Q2: `h3_formats::lod`: `MAX_TABLE_BYTES`, `LodTable::parse(head, file_len)`, `LodEntry::stored_range`, `read_entry`; `LodArchive` rests on them, unchanged for the harvester and `h3-view`. A property: a table from any prefix of a mutated archive agrees with the whole parse. On the four real LODs both readers agree on every entry (an ad-hoc test, not committed; `installation.md` §3). CLT-Q2 answered; the switch is the client lane's (its inbox). Then `battle/test-gaps` (tests only, no behaviour changed, none failed): BAT-QUIRK-21's second half (a mutation to the original's listing fails only it), Bind released by a flying and a teleporting binder, and no attack from a seen Quicksand pit on the harvested packs (seeds 1 to 40 pick the draws; 8 cases per profile).
