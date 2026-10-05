# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04b-battle-magic` (created from `master` = the E04b refinement `3841f22`, on top of the merged E04a, verify green on Linux and Windows incl. `--with-original-data`); docs only since then. Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04b (battle magic) is refined** (Q41, `docs/epics/E04b-battle-magic.md`): increments with a user checkpoint each. **Increment 1** = T1 (survey and ADR-0014, then **stop for the user's review**) and T2 (casting core, checkpoints T2a–T2d, the factorized casting action in T2c, batch 2a at the end); after T2d the user confirms increment 2 (proposed: T3, T4).
- **T1 done:** survey `docs/rules/surveys/E04b-magic.md`, **ADR-0014 accepted** (Q42), design note `docs/epics/E04b-design.md`, record in `docs/epics/E04b-log.md`.
- **T2 split confirmed** (user, 2026-10-05): T2a + T2b, checkpoint; then T2c + T2d, checkpoint and batch 2a. **T2a done** (pack format v4, spell flags harvested, the hero's magic, HER-BAT-011/012). **In progress: T2b** (casting in battle; design note section T2b).
- Versions: state hash v17, ruleset hash v14, replay format v13, action encoding v8, C ABI v7, observation encoding v4. Each new command bumps the action encoding and the C ABI (ADR-0011).
- No decision is waiting for the user. Open questions: `docs/OPEN-QUESTIONS.md`, section "Open" (Q40 waits for batch 2a).
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Sandbox and sync**
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise). A fast-forward once failed with "unable to unlink … Permission denied" (a file held open on Windows): write the file from `git show <commit>:<path>`, then `git reset --mixed <commit>`.
- Each bundle fetch adds a pack; past 50 packs a fetch starts a detached `git gc --auto`, which dies with the device shell and leaves `.git/gc.log.lock`. Fetch with `git -c gc.auto=0 fetch …`; if packs pile up, run `git gc` in the foreground. Old packs Windows keeps open stay behind as garbage without `.idx` (harmless; 47 such files since 2026-10-04, deletable once nothing holds them).
- The sandbox VM can restart mid-session (files stay, background jobs and installs die): check `ps` and the logs before waiting on a background job.
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background and poll its log.
- The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with its own `CARGO_TARGET_DIR`, and commit there on a branch that the clone fast-forwards to. `pkill -f <pattern>` also kills the tool's own shell when the pattern is in the command line: kill by pid.
- Broad sub-agent surveys burn tokens and hit rate limits; research in narrow steps yourself.
- Docs-only sessions can work in the user's repo directly (the device shell has git and python, no cargo): commit with `-c user.name=... -c user.email=...` (no identity configured there) and check what `check-docs` checks by script (HANDOFF at most 10 000 bytes, three session entries).

**Sources**
- The wiki is reachable only through WebFetch (summarised) and needs the user's approval per session: ask for verbatim quotes, use `index.php?title=<Page>&action=raw` and `index.php?search=<term>&fulltext=1`; ask for the `swh` template's SoD argument, the summariser otherwise reports HotA.
- VCMI 1.7.5: `git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git` works through the proxy.
- Code reading (Q29): stage `Heroes3.exe`, `pip install capstone pefile`, disassemble `.text` with `skipdata`; known layouts in `docs/formats/installation.md` §7.5–§7.7.
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

### 2026-10-05 (fifteenth) — E04b refined (increment 1 = T1 + T2), T1 done, ADR-0014 accepted

Refinement, in the user's repo (no sandbox, no code): coarse outline T1–T11; the user chose (Q41) one epic in increments with a checkpoint each; increment 1 = T1 (survey, ADR-0014, stop) and T2 (casting core, 13 direct-damage spells, T2a–T2d, then batch 2a); casting in the sim with the core (T2c); a just-in-time survey. Commit `3841f22` (epic file, Q41, BACKLOG, HANDOFF; project doc `claude/e04b-status.md`). Then, on the user's request, branch `e04b-battle-magic` and T1's survey: VCMI 1.7.5 and wiki fetches, the executable's spell flags decoded by correlation; disputes M-1 to M-6 and candidates X-1 to X-6 for batch 2a (`f98fcc6`). The user clarified that a hero's cast is optional and that a counter spell replaces its pair (the later wins); then ADR-0014 (Proposed, `688739c`) and the design note; the user's first review led to revised points 5–7, then accepted it (Q42).

### 2026-10-05 (fourteenth) — E04a T12c: independent review, the epic closed

A fresh session reviewed E04a (baseline verify green on `7990c69`): every rule against its sources and code, no conflict; five rules resting on VCMI or a generalisation listed for the next batch (`docs/experiments/README.md`); two robustness tests (`cd71ba0`); security checked. Closure: design note deleted (its last content in the log), epic, BACKLOG, ADR-0013's header. The user: Windows verify green; the tactics hand-over needs an experiment (Q40). Detail: `docs/epics/E04a-log.md` (T12c).

### 2026-10-04 (thirteenth) — E04a T11, T12a and T12b done

T11 split with the user (Q37): `observe` takes the ruleset (ADR-0004 amended); T11a (`e527a53`) the observation's surrender option, machine control and private `own_hero`; T11b (`adffa12`) observation encoding v4, C ABI v7; T11c Corwin leads `full`'s defender, `full-s3` re-recorded, `bench-check` re-baselined (about 229–237 k steps/s). T12 split (Q38): T12a 21 observed rules `verified`, profile and hero-isolation tests, ADR-0013 revised; T12b the death-stare cutoff (Q39, KER-RNG-003) and random obstacles per episode. Detail: `docs/epics/E04a-log.md`.
