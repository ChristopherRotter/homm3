# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **No epic in progress.** E04c closed 2026-10-16; since then #265 (the AI lane's two proptest cases) and the docs' cleanup (#267 merged; the QUEUE's consolidation on `battle/queue-consolidation`, open), all 2026-10-19. E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Next, the user's choice:** E23, the original's battle AI (after E05; to refine: a reading increment, then about three), or E11's battle part.
- **Open:** BAT-Q28, BAT-Q32, BAT-Q33, BAT-Q35, BAT-Q51 (none blocks), BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43; S-23, S-24, batch 2g.
- **Versions** (unchanged since T10b): battle state hash v51, battle part hash v6 (the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v45, pack format v43, protocol v33. Next ids: BAT-Q52, BAT-QUIRK-24, BAT-STUB-4, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent.

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

**Packs, replays, benchmarks**
- `packs/base|ab|sod` are generated: change `h3-harvest` or its gap-fill file and re-run it; harvested keys may coincide across kinds (`base:ballista`).
- A quirk flag that comes or goes (a retired one keeps its `REGISTRY` position) changes the faithful ruleset hashes: the golden replays and the adventure lane's faithful replays (`cross-lane`).
- Golden replays and `benches/battle/` change with behaviour, the synthetic pack, an encoding or the toolchain: regenerate deliberately, the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (cold per-stack data goes beside them: `BattleState.raised`, `.created`); code in `movement::reachable_within` or `casters::ready` swings inlining. Before re-baselining, measure `master` in a worktree (own target dir) and compare per-function callgrind profiles. `random-rollouts-full` was at +1.34 % on `master` (2026-10-19).

**Process**
- Edit the long docs (`installation.md`, the catalog) by targeted replacements, never by rewriting from memory.
- The user merges with merge commits or squashes: rebase a stacked branch with `git rebase --onto origin/master <old base head>`; fetch `master` again before a pull request that bumps a shared version.

## Recent sessions

### 2026-10-19 (seventy-second) — the AI lane's two proptest cases

#265 (merged): a walk's step onto the outer bridge opened the gate only with a recording sink, and a Clone on the gate's hex kept it open (fixed in `obstacles::go`, `travel_through`); the invariant property missed BAT-SIE-011's pass. Then, at the user's request, the docs' cleanup: this HANDOFF trimmed (#267: duplicates and stale traps out, area traps moved to the experiments and fixtures READMEs); the QUEUE's inbox merged by kind (20.6 → 9.1 KB), its items for other lanes sent to their inboxes, the First Aid Tent made BAT-Q51, BAT-Q33 brought up to date (`BattleSetup.human`, AI-Q3).

### 2026-10-16 (seventy-first) — E04c T10c: the closure (increment 10)

Closed: the shared sentences (BAT-Q50), `h3-heroes` handed over, E23, the lanes' inboxes, the user's Windows run. Detail: the log.

### 2026-10-15 (seventieth) — E04c T10b: the last reviews and the fold-in (increment 10)

r6 and security; the user's decisions BAT-Q50; five fixers; observation v21, ABI v25; siege-s7 and the adventure lane's nine replays redone. Detail: the log.
