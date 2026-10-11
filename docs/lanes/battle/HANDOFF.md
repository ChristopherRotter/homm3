# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-t9b` (BAT-Q27). Work on `battle/<task>` branches from `master`, merged at its checkpoint; HANDOFF content goes here.
- **E04b increment 20 (the user, BAT-Q27): T9b (the casting workload, the measurements) done; batch 2f staged next.** Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. T1–T9 (the epic file lists each with its rules): every battle spell, creature magic, casters, mana creatures, defences, the rebirth and Fear, the hero's spells and masteries in the Heroes context, the casting limits and spell defences, the per-spell conditions, the opening spells and Eagle Eye, the data quirks, a complete observation with a spectator viewer, the casting workload (BAT-MAG-001 to 069, HER-SPL-002 to 004, HER-BAT-015 to 017; Q-0006 to Q-0012, BAT-QUIRK-1 to 19), all after the executable's code (`installation.md` §7.8–§7.24). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** batch 2f staged (T10: increment 17's opening-spell candidates, T8b's leftovers, BAT-Q24's learning; 2e's weak results decisive, BAT-Q27), then its fold-in and T11. BAT-Q24 (who learns Eagle Eye's picks) waits for the adventure lane.
- **Open for later tasks:** Q43 (sprite offset, keep the approximation); BAT-QUIRK-17's later variant (rebirth right after a sacrifice, BAT-Q18); bad morale's `WAITED` gate (T7c3 review); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v41, ruleset hash v42, replay format v21, action encoding v11, C ABI v17, observation encoding v13; pack format v34. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
- **Ownership:** `h3-content` and `h3-pack` stay with battle until further notice (user, 2026-10-07); `h3-kernel` and `h3-contracts` are shared, so a seam change is its own `shared/` pull request, merged before the battle change that uses it.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

- A data quirk's value stays in the pack; its fix is an override gated by a `quirk.content` flag, read only by the compiler (ADR-0007).
- Tests patch the synthetic pack by text (`SYNTHETIC_PACK.replace`): a patch that no longer matches does nothing, silently (T7c1); assert that the text changed. To pick replay seeds, count events per agent seed with a throwaway test in `crates/h3-cli/tests` (`run_random`, then re-apply the commands with an event sink).

**Code reading**
- Edit docs with targeted replacements, never by rewriting a file from memory (T7a2 lost `QUEUE.md`'s inbox so).
- The executable's x87 runs at 53 bits (`installation.md` §7.10): emulate it with `h3_kernel::arith::X87` and test against vectors from exact rational arithmetic; check each float constant's width (`qword` double or `dword` f32).
- Name an address only after reading its code: §7.15 had called 0x469E50 (the moat's damage) a Quicksand test.

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off). `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- A dead stack's effects stay until the command's end (`effects::remove_dead`): code that brings a stack back within a command removes them first (the rebirth, T7c2b review). The battle's first activation runs without the freezing checks (`turn::activate_first`); good morale goes through `turn::morale_turn`.
- Hero spells and masteries come from `h3-heroes` (ADR-0013 point 2); the battle reads `BattleMagic` only. Tests: `support::masteries(r, [schools])`. A stack's spell test needs the casting side (`affects_stack`, `share`, `resists`: the heroes' `Guard`, BAT-MAG-066); battle-wide hero magic is `BattleState.limits`, derived at the start.
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. `Stack.spells` holds the spells' values only: a creature's `innate_spell` is read where the original reads the creature (`magic::mirror_chance`, `magic::reflection`), not by Hypnotize, Dispel or usefulness tests. Two `skill_specialty` effects in one specialty are rejected.

- The observation is complete since T9a: a new state field a rule reads goes into `observe_as` (public, own part or neither: ADR-0014 point 8), the flat encoding (version and ABI bump) and the hygiene property.
- `h3-protocol`'s hygiene tests compare a viewer's frames across seeds on the `full` scenario up to the first hidden cast: no public random draw may come before it (resistance on Brea's stacks did, T9b).

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (T5c1, T6b): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- The random agent and `legal_mask` ask for the active stack's cast at every step: keep `casters::ready` a cheap early exit (casts left first; the targets' buffer only for a caster cost +2 % Ir in T7b1).
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Pull requests**
- The user merges with merge commits; after a rebase-merge, rebase a stacked branch with `git rebase --onto origin/master <old base head>` and push with a lease.

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-11 (fiftieth) — E04b increment 20: T9b

The user (BAT-Q27): T9b, then batch 2f; batch 2e's weak results decisive. The `full` heroes gained opening spells, Eagle Eye, resistance, Pendants and Advanced spells; replays re-seeded, `bench-check` re-baselined; ≈ 105 k steps/s, ≈ 70 k with `--observe 1`. Detail: `docs/epics/E04b-log.md`.

### 2026-10-11 (forty-ninth) — E04b increment 19: T9a and the spectator viewer

The user (BAT-Q26): T9a with SRV-Q4's spectator viewer, per-hex planes, a hero's hidden magic private to its side. `Observation` complete, `observe_as`, `project_as`; encoding v13, C ABI v17. Detail: `docs/epics/E04b-log.md`.

### 2026-10-10/11 (forty-eighth) — E04b increment 18: T8e

The user (BAT-Q25): T8e alone, S-05 fix-optional (BAT-QUIRK-19); ADR-0007 amended (#64). Gated overrides, pack v34. Detail: `docs/epics/E04b-log.md`.
