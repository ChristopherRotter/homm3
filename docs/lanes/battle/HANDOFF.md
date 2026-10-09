# Battle lane — HANDOFF

The battle lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Branch:** `battle/e04b-t7c2a` (one PR; no shared change but the global HANDOFF's battle row, `shared/handoff-battle-t7c2a`: merge it first). Work on `battle/<task>` branches from `master`, one per task or increment, merged at its checkpoint; HANDOFF content goes here, not into `docs/HANDOFF.md`.
- **E04b increment 12 (the user, 2026-10-09, BAT-Q16): T7c2a; done.** Sandbox: `docs/08-work-process.md`, "Cloud sandbox".
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. T1–T7b4, T7c1 and T7c2a (the epic file lists each with its rules): the hero's spells but the T8 long tail (BAT-MAG-001 to 039; Q-0007 to Q-0012, BAT-QUIRK-1 to 8), the strike's creature magic (BAT-MAG-040 to 047), the casting creatures (BAT-MAG-048 to 053; BAT-QUIRK-9 to 14), the mana creatures (BAT-MAG-054 to 056: costs from the armies, the Familiars' channel, the Wraiths' drain; BAT-QUIRK-15, 16), the creature defences (BAT-MAG-057, 058, BAT-DMG-010: the dragons' mirror, the Sultans' shield, the elementals' halves), all after the executable's code (`installation.md` §7.8–§7.21). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** the user's checkpoint for increment 12; then T7c2b (the Phoenix's rebirth: first look in `installation.md` §7.21; read 0x445A60, the raise's last argument and the outcome first).
- **Open for later tasks:** Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the Phoenix's rebirth after a mine's or Fire Wall's damage (T7c2b); Dispel's loose ends (Sphere T8); T4a's and T4c's loose ends (all in `QUEUE.md`, "Inbox").
- **Versions (battle owns them):** state hash v38, ruleset hash v37, replay format v19, action encoding v11, C ABI v15, observation encoding v11; pack format v28. New ids: `BAT-Q<n>`, `BAT-QUIRK-<n>`, `BAT-STUB-<n>` (old ids stay).
- **Ownership:** `h3-content` and `h3-pack` stay with battle until further notice (user, 2026-10-07); `h3-kernel` and `h3-contracts` are shared, so a seam change is its own `shared/` pull request, merged before the battle change that uses it.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Content and packs**
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

- Tests patch the synthetic pack by text (`SYNTHETIC_PACK.replace`): a patch that no longer matches does nothing, silently (T7c1); assert that the text changed. To pick replay seeds, count events per agent seed with a throwaway test in `crates/h3-cli/tests` (`run_random`, then re-apply the commands with an event sink).

**Code reading**
- Edit docs with targeted replacements, never by rewriting a file from memory: T7a2's rewrite of `QUEUE.md` dropped its whole inbox (restored at the T7a checkpoint).
- The executable's x87 runs at 53 bits (`installation.md` §7.10): emulate it with `h3_kernel::arith::X87` and test against vectors from exact rational arithmetic; check each float constant's width (`qword` double or `dword` f32).
- Name an address only after reading its code: §7.15 had called 0x469E50 (the moat's damage) a Quicksand test.

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Hidden obstacles leak through public events too: a hidden obstacle that does nothing must not change them (a mine that does not go off must not split a `Moved`). `hidden_obstacles_do_not_show` checks the acting side's commands and casts (its profiles leave BAT-QUIRK-4 off). `legal_commands` lists no casts: a property over legal commands must add `cast_targets` × `second_targets` itself.
- Natural obstacles follow the original's hex records (`h3_battle::natural`): the piece listed last wins a shared hex, and a removal clears its hexes and anchor for every piece (0x466B30).
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. `Stack.spells` holds the spells' values only: a creature's `innate_spell` is read where the original reads the creature (`magic::mirror_chance`, `magic::reflection`), not by Hypnotize, Dispel or usefulness tests. Two `skill_specialty` effects in one specialty are rejected.

**Fixtures and benchmarks**
- Golden replays and the `benches/battle/` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update --workload random-rollouts-full`) and give the reason in the commit.
- A field added to `Stack` costs Ir wherever stacks are walked (+0.6 % for 8 bytes in T6a): keep per-stack data the hot paths do not read beside the stacks (`BattleState.raised`, `.created`).
- A stack can now die from another's death (a clone with its original, BAT-MAG-038): loops over targets fixed before damage skip dead stacks, and every path that kills calls `created::died` (`effects::damaged` after damage, the death stare, the sacrifice; it also releases binds).
- Code added to `movement::reachable_within` can push `fits` out of line (+1 % Ir in T5c1; it is `#[inline]` and tests head and tail without the footprint iterator since T6b, −11 %): compare per-function profiles (`callgrind_annotate --inclusive=no`) of the old and new binaries on the old workload.
- The random agent and `legal_mask` ask for the active stack's cast at every step: keep `casters::ready` a cheap early exit (casts left first; the targets' buffer only for a caster cost +2 % Ir in T7b1).
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Pull requests**
- The user rebase-merges: once a stacked PR's base is merged, rebase the next branch with `git rebase --onto origin/master <old base head>` and push with a lease (#13 conflicted, 2026-10-08).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Check every move against the stack's speed (the armies start 14 columns apart). A single-alignment army has +1 morale: mix two where an extra turn would spoil a count. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-09 (forty-second) — E04b increment 12: T7c2a, the creature defences

Master green with original data. The user (BAT-Q16): T7c2 in two parts, T7c2a first; T7c1's own calls confirmed. Pack v28 (`innate_spell`, `damage_penalty`, `damage_penalty_by_flag`), BAT-MAG-057, 058, BAT-DMG-010, ruleset hash v37 (state hash unchanged); a first look at the Phoenix's rebirth for T7c2b; replays and bench re-baselined. Detail: `docs/epics/E04b-log.md`.

### 2026-10-09 (forty-first) — E04b increment 11: T7c1, the mana creatures

Master green. The user (BAT-Q15): T7c1 alone; the cost reads the armies (BAT-QUIRK-15, the rule); the Familiars' loop bound always fixed (BAT-QUIRK-16). Pack v27 (`spell_cost`, `mana_channel`, `mana_drain`), BAT-MAG-054 to 056 (`h3_battle::mana`; `BattleState.wait_phase`), ruleset hash v36, state hash v38; batch 2a R6 replayed on the harvested packs; replays and bench re-baselined. Detail: `docs/epics/E04b-log.md`.

### 2026-10-09 (fortieth) — E04b increment 10: T7b4, the Enchanter

Master green with original data. The user (BAT-Q14): T7b4 alone; S-12 fix-optional (BAT-QUIRK-13); every battle starts ready (BAT-QUIRK-14: the original's counters carry over between battles, found in the re-reading). Pack v26 (`periodic_spell_caster`), quirk flag 16, BAT-MAG-053, state hash v37, observation v11, C ABI v15 (shared PR for ADR-0011's list); the synthetic Chanters; replays and bench re-baselined. Detail: `docs/epics/E04b-log.md`.
