# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04a-heroes-in-battle` (the user merges to `master` when an epic closes); `cargo xtask verify --with-original-data` green. Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04a** (`docs/epics/E04a-heroes-in-battle.md`): T1–T8 done, experiment batches 1a and 1b evaluated (`docs/epics/E04a-design.md`, "Batch 1b results" says what T9 and T10 must do). **Next: T9 (retreat, surrender, richer outcome)**; at the start, propose a split and let the user confirm it (T8 was one checkpoint, Q33). Then T10, T11, T12.
- Versions: state hash v13, ruleset hash v12, replay format v8, action encoding v6, C ABI v4, observation encoding v3. Each new command bumps the action encoding and the C ABI (ADR-0011).
- No decision is waiting for the user. Long-term open questions: `docs/OPEN-QUESTIONS.md`, section "Open".
- Working agreement: small increments, the user confirms the seam; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Sandbox and sync**
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise). A fast-forward once failed with "unable to unlink … Permission denied" (a file held open on Windows): write the file from `git show <commit>:<path>`, then `git reset --mixed <commit>`.
- Each bundle fetch adds a pack; past 50 packs a fetch starts a detached `git gc --auto`, which dies with the device shell and leaves `.git/gc.log.lock`. Fetch with `git -c gc.auto=0 fetch …`; if packs pile up, run `git gc` in the foreground. Old packs Windows keeps open stay behind as garbage without `.idx` (harmless; 47 such files since 2026-10-04, deletable once nothing holds them).
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

### 2026-10-04 (eleventh) — E04a T8 done (tactics)

Scope confirmed by the user (Q33): T8 as one checkpoint; a walking stack's path may leave the zone. BAT-TAC-001/002: `tactics.rs`, battle parameter `tactics_zone_columns`, `Command::EndTactics`, `Observation.tactics`, events `TacticsStarted`/`TacticsEnded`; state hash v13, ruleset hash v12, replay format v8, action encoding v6, C ABI v4; `full-s3` 484 commands; bench re-baselined (+28.6 % Ir: the `full` workload now has a tactics phase, +13.7 % steps, a phase step costs about twice a battle step). Agent decisions and measurements: `docs/epics/E04a-log.md`, T8. A detached auto-gc on the user's machine left a lock (new standing trap).

### 2026-10-04 (tenth) — E04a T2 done: batch 1b designed, played and evaluated; start cost reduced

Batch 1b (`49a7cd1`; the user's results `4610112`) and its evaluation (`492bc51`): `E04a-design.md` "Batch 1b results", Q-0005 extended to the surrender discount, ADR-0013 point 7 amended (no consent step), OPEN-QUESTIONS Q32, catalog sources on BAT-WAR-007 and BAT-RNG-002. User decisions (Q32): war machines count in the surrender price (a rule); Q-0005 covers the discount; consent dropped. Agent decision: the Tactics-formation toggle is not modelled. Then, on the user's request: HANDOFF cut to this format with `cargo xtask check-docs`, the epic's done tasks moved to `docs/epics/E04a-log.md`, `scripts/sandbox-setup.sh` and `scripts/sandbox-verify.sh`, AGENTS.md and `08-work-process.md` updated.

### 2026-10-04 (ninth) — E04a T7 done (T7a Ballista, T7b First Aid Tent)

`427de8e`, `534cbf2`: BAT-WAR-005/006/007, HER-BAT-005, quirk Q-0004, `Command::Heal`, state hash v12, replay format v7, action encoding v5, C ABI v3; `full-s3` 437 commands; bench re-baselined (−2.1 %, −6.0 %), about 200 k steps/s. Agent decisions: `docs/epics/E04a-log.md`, T7.
