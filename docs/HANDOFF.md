# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04a-heroes-in-battle`; `cargo xtask verify --with-original-data` green on Linux. Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04a is closed** (2026-10-05, `docs/epics/E04a-heroes-in-battle.md`; summary in `docs/BACKLOG.md`). The user runs verify on Windows (exit criterion 7) and merges the branch to `master`.
- **Next: E04b (battle magic)**, not refined yet: start with a refinement session with the user (scope in BACKLOG and Q27; what E04b needs from E04a: the epic file's last section and `docs/epics/E04a-log.md`, "The design note at closure"). Candidates for its first experiment batch: `docs/experiments/README.md`.
- Versions: state hash v17, ruleset hash v13, replay format v13, action encoding v8, C ABI v7, observation encoding v4. Each new command bumps the action encoding and the C ABI (ADR-0011).
- No decision is waiting for the user besides the merge. Long-term open questions: `docs/OPEN-QUESTIONS.md`, section "Open".
- Working agreement: small increments, the user confirms the seam; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Sandbox and sync**
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise). A fast-forward once failed with "unable to unlink … Permission denied" (a file held open on Windows): write the file from `git show <commit>:<path>`, then `git reset --mixed <commit>`.
- Each bundle fetch adds a pack; past 50 packs a fetch starts a detached `git gc --auto`, which dies with the device shell and leaves `.git/gc.log.lock`. Fetch with `git -c gc.auto=0 fetch …`; if packs pile up, run `git gc` in the foreground. Old packs Windows keeps open stay behind as garbage without `.idx` (harmless; 47 such files since 2026-10-04, deletable once nothing holds them).
- The sandbox VM can restart mid-session (files stay, background jobs and installs die): check `ps` and the logs before waiting on a background job.
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background and poll its log.
- The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with its own `CARGO_TARGET_DIR`, and commit there on a branch that the clone fast-forwards to. `pkill -f <pattern>` also kills the tool's own shell when the pattern is in the command line: kill by pid.
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

### 2026-10-05 (fourteenth) — E04a T12c: independent review, the epic closed

A fresh session reviewed E04a (baseline verify green on `7990c69`). Rules against sources: every E04a rule read against its sources and code, no conflict (the original's text confirms the Spirit of Oppression and Hourglass caps); five rules resting on VCMI or a generalisation are listed for the next batch (`docs/experiments/README.md`). Robustness: two new tests (`cd71ba0`; extreme sound heroes with counts up to `u32::MAX`, and random legal SoD heroes derived and played); 20 000 and 3 000 ad-hoc battles and 60 000 random-obstacle episodes ran clean. Security: commands, prices and private hero values checked; no change. Closure: the design note deleted, its last content in the log, epic, BACKLOG, ADR-0013's header, references and the project status doc updated. Detail: `docs/epics/E04a-log.md` (T12c).

### 2026-10-04 (thirteenth) — E04a T11, T12a and T12b done

T11 split with the user (Q37): T11a and T11b this session, T11c next; `observe` takes the ruleset (ADR-0004 amended); the experiment-spec runner stays in the BACKLOG. T11a (`e527a53`): `Observation.surrender_possible`, `HeroView.commands_ballista`/`commands_tent` public, `Observation.own_hero` (`OwnHeroView`) private; hygiene property test varies every private hero value. T11b (`adffa12`): observation encoding v4 (`OBSERVATION_LEN` 1229), C ABI v7. Then, on the user's request, T11c: the defender of `full` led by Corwin with T10 effects, both owners able to pay a surrender; `full-s3` re-recorded; `bench-check` re-baselined (+65 % steps, −14 % Ir/step); about 229–237 k steps/s. Then T12 split with the user (Q38) and T12a done: 21 observed rules `verified`, tests for the profiles' quirk flags and hero isolation, ADR-0013 revised. T12b: death-stare cutoff (Q39, KER-RNG-003), `BattleEnv::with_random_obstacles`. Detail: `docs/epics/E04a-log.md`.

### 2026-10-04 (twelfth) — E04a T10 done (stat effects, traits, alignment groups)

T10 split with the user (Q35: T10a, T10b now, T10c next; Q-0006's profile override deferred to E04b). T10a: HER-BAT-008 (derivation of `creature_stat`, `creature_stat_percent`, `creature_specialty`), BAT-HER-005 (applied in `Stack::new`; war machines no speed), `StatOp::PercentBpDown`, effect kind `stack_stat` in scenario and replay files (replay format v11), batch 1a R8/B4–B6 and 1b R8/B7 reproduced with the harvested packs (`h3-pack` test; `h3-battle` dev-dependency). `Field::neighbors` written out (codegen swing, −12.5 % Ir, re-baselined). T10b: HER-BAT-009, BAT-HER-006 (no range penalty, shooting while adjacent, the hero's regeneration capped at its hit points; `Stack.regeneration`, state hash v17). T10c (the user asked to fit it in): `FactionGroup` (Q36), HER-BAT-010, BAT-HER-007 (batch 1b R1 reproduced), replay format v13; T10 measurements in the log. Detail: `docs/epics/E04a-log.md`.
