# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04a-heroes-in-battle` (the user merges to `master` when an epic closes); `cargo xtask verify --with-original-data` green. Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04a** (`docs/epics/E04a-heroes-in-battle.md`): T1–T10 done, experiment batches 1a and 1b evaluated and folded in (`docs/epics/E04a-design.md`); T10 in three checkpoints (Q35/Q36). T11 done (Q37). T12 in three checkpoints (Q38): T12a done (21 rules `verified`, profile and hero-isolation tests, ADR-0013 revised, exit-criteria audit in the epic). **Next: T12b** (hardening: the death-stare work bound — present the options to the user first: a ruleset `max_stack_count` checked at setup, or an exact integer binomial sampler, BACKLOG; random obstacles per episode in the sim and the C ABI, BACKLOG E03 T7/T9 review), then **T12c** in a fresh session (independent review pass: rules against sources, robustness, security; then the closure: design note deleted, HANDOFF, backlog; the user runs verify on Windows and merges to `master`).
- Versions: state hash v17, ruleset hash v13, replay format v13, action encoding v8, C ABI v7, observation encoding v4. Each new command bumps the action encoding and the C ABI (ADR-0011).
- No decision is waiting for the user. Long-term open questions: `docs/OPEN-QUESTIONS.md`, section "Open".
- Working agreement: small increments, the user confirms the seam; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Sandbox and sync**
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise). A fast-forward once failed with "unable to unlink … Permission denied" (a file held open on Windows): write the file from `git show <commit>:<path>`, then `git reset --mixed <commit>`.
- Each bundle fetch adds a pack; past 50 packs a fetch starts a detached `git gc --auto`, which dies with the device shell and leaves `.git/gc.log.lock`. Fetch with `git -c gc.auto=0 fetch …`; if packs pile up, run `git gc` in the foreground. Old packs Windows keeps open stay behind as garbage without `.idx` (harmless; 47 such files since 2026-10-04, deletable once nothing holds them).
- The sandbox VM can restart mid-session (files stay, background jobs and installs die): check `ps` and the logs before waiting on a background job.
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background and poll its log.
- Broad sub-agent surveys burn tokens and hit rate limits; research in narrow steps yourself.

**Sources**
- The wiki is reachable only through WebFetch (summarised) and needs the user's approval per session: ask for verbatim quotes, use `index.php?title=<Page>&action=raw` and `index.php?search=<term>&fulltext=1`; ask for the `swh` template's SoD argument, the summariser otherwise reports HotA.
- VCMI 1.7.5: `git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git` works through the proxy.
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

**Coverage, lints, formatting**
- LLVM counts the implicit `else` of `if let`, the error arm of `?` and rarely reached closures (`Option::map` on a rare `Some`): no defensive dead branches; use infallible accessors and combinators, saturate instead of returning `Option` (`WideUint`), make property tests deterministic in what they cover.
- Coverage misses on non-code lines mean stale binaries: `cargo llvm-cov clean --workspace`.
- clippy: `should_implement_trait` rejects methods named `div`/`shr`; `type_complexity` fires on arrays of fn pointers in tests (use a type alias). `cargo fmt` reflows code: match scripted patches against the formatted text.
- Dev-dependencies obey the layers too (`check-arch`): domain crates build test rulesets in code.
- serde ignores unknown keys of a unit variant in an internally tagged enum (use struct variants); TOML serialization needs table fields after plain values.

**Battle code and tests**
- Stack ids: war machines follow the army in kind order (Ballista, Ammo Cart, Tent), not in the order of `BattleSetup.war_machines`.
- Defend raises defense and spoils exact damage expectations: skip turns with `activate`. `BattleState::new` drops the first activation's events (use `start` where events matter). A test ruleset may not make bad morale certain (254/255).
- Rule code reads abilities via `ruleset.abilities(creature)`; per-attack booleans are `StackTraits` bits set at setup. Two `skill_specialty` effects in one specialty are rejected.

**Fixtures and benchmarks**
- Golden replays and `bench-check` baselines change with behavior, the synthetic pack, an encoding or the toolchain: regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update`) and give the reason in the commit.
- Before re-baselining, compare `h3 bench --games 20` step counts (a shorter workload looks like a speed-up; build the old commit in a `git worktree` with its own target dir; `git stash` keeps untracked files). The geometry hot path moves with unrelated codegen: compare per-function callgrind profiles (`--toggle-collect='*random_rollout*'`, read the out file's `summary:` line; run `h3 bench` where `packs/` resolves).

**Experiments in the original game**
- The installation is read-only: maps go to the gitignored `experiments/out/`, the user copies them. Results hold for SoD; observation sources name the setup.
- The `.h3m` writer knows dirt, grass and water tiles only (dirt: native to Necropolis only). A hero type once per map, at most eight heroes per player. Wandering monsters act on their own: use a Blue hero when the opponent must hold still. Every hero has a Catapult in the game; the hero screen shows no army-alignment morale.

## Recent sessions

### 2026-10-04 (thirteenth) — E04a T11 and T12a done

T11 split with the user (Q37): T11a and T11b this session, T11c next; `observe` takes the ruleset (ADR-0004 amended); the experiment-spec runner stays in the BACKLOG. T11a (`e527a53`): `Observation.surrender_possible`, `HeroView.commands_ballista`/`commands_tent` public, `Observation.own_hero` (`OwnHeroView`) private; hygiene property test varies every private hero value. T11b (`adffa12`): observation encoding v4 (`OBSERVATION_LEN` 1229), C ABI v7. Then, on the user's request, T11c: the defender of `full` led by Corwin with T10 effects, both owners able to pay a surrender; `full-s3` re-recorded; `bench-check` re-baselined (+65 % steps, −14 % Ir/step); about 229–237 k steps/s. Then T12 split with the user (Q38) and T12a done: 21 observed rules `verified`, tests for the profiles' quirk flags and hero isolation, ADR-0013 revised. Detail: `docs/epics/E04a-log.md`.

### 2026-10-04 (twelfth) — E04a T10 done (stat effects, traits, alignment groups)

T10 split with the user (Q35: T10a, T10b now, T10c next; Q-0006's profile override deferred to E04b). T10a: HER-BAT-008 (derivation of `creature_stat`, `creature_stat_percent`, `creature_specialty`), BAT-HER-005 (applied in `Stack::new`; war machines no speed), `StatOp::PercentBpDown`, effect kind `stack_stat` in scenario and replay files (replay format v11), batch 1a R8/B4–B6 and 1b R8/B7 reproduced with the harvested packs (`h3-pack` test; `h3-battle` dev-dependency). `Field::neighbors` written out (codegen swing, −12.5 % Ir, re-baselined). T10b: HER-BAT-009, BAT-HER-006 (no range penalty, shooting while adjacent, the hero's regeneration capped at its hit points; `Stack.regeneration`, state hash v17). T10c (the user asked to fit it in): `FactionGroup` (Q36), HER-BAT-010, BAT-HER-007 (batch 1b R1 reproduced), replay format v13; T10 measurements in the log. Detail: `docs/epics/E04a-log.md`.

### 2026-10-04 (eleventh) — E04a T8 and T9 done (tactics; retreat, surrender, outcome)

T8 as one checkpoint (Q33; a walking path may leave the zone): BAT-TAC-001/002, `tactics.rs`, battle parameter `tactics_zone_columns`, `EndTactics`; bench re-baselined (+28.6 % Ir: `full` now opens with a tactics phase). T9 in three checkpoints (Q34; the baseline random agent never leaves a battle): T9a Retreat (BAT-FLE-001, HER-BAT-006, results `Retreat`/`Surrender`), T9b Surrender (BAT-FLE-002, HER-BAT-007, `BattleSetup.funds`, battle parameter `surrender`, Q-0005 discount), T9c the outcome (BAT-END-003). Versions now: state hash v16, ruleset hash v13, replay format v10, action encoding v8, C ABI v6. Agent decisions: `docs/epics/E04a-log.md`. On the user's machine a detached auto-gc left a lock and a fast-forward could not replace a file held open (both handled; standing traps).
