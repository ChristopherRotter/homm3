# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04b-battle-magic` (created from `master` = the E04b refinement `3841f22`, on top of the merged E04a); T2a–T2d committed. **In progress (2026-10-06):** experiment batch 2a (task list below), then the user's increment 1 checkpoint. Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04b (battle magic) is refined** (Q41, `docs/epics/E04b-battle-magic.md`): increments with a user checkpoint each. **Increment 1** = T1 (survey, ADR-0014) and T2 (casting core, T2a–T2d, batch 2a at the end); after batch 2a the user confirms increment 2 (proposed: T3, T4).
- **T1, T2a–T2d done:** survey `docs/rules/surveys/E04b-magic.md` (§12: the executable's code), ADR-0014 accepted (Q42), design note `docs/epics/E04b-design.md` (T3 onwards). The 13 damage spells are castable by heroes: BAT-MAG-001 to 008 (damage order, immunities, Chain Lightning by screen geometry, areas, resistance after `docs/formats/installation.md` §7.8), HER-BAT-011 to 013, the factorized casting action (T2c).
- **Batch 2a (next):** the experiment spec gains spells and mana; cases: Chain Lightning pixel vs hex distance (a stack two hexes to the right, 88 px, against one two rows down, 84 px; Q44), X-3 (a cast during a commanded Ballista's turn), X-4 (costs with Mages, Pegasi, a Familiar), resistance and the aimed stack's fizzle, single-precision resistance (Q-0005: 10 % leaves 89 %), Q40 (tactics hand-over), rounding of Sorcery × Orb × specialty. Maps to `experiments/out/`.
- Versions: state hash v19, ruleset hash v16, replay format v15, action encoding v9, C ABI v8, observation encoding v5.
- Waiting for the user: nothing until batch 2a is compiled. Open questions: `docs/OPEN-QUESTIONS.md`, section "Open" (Q40 and Q43 wait for batch 2a).
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

### 2026-10-06 (seventeenth) — E04b T2c and T2d: the factorized action, the damage spells after the code

Sandbox session; the user confirmed T2c + T2d, then batch 2a, and kept T2b's decisions. T2c (`b7b152a`): 21 stack slots per side, action encoding v9, C ABI v8, observation v5. T2d: the executable's spell code read (`installation.md` §7.8; M-1, M-2, M-6 settled); the user decided (Q44) the specialty contract change, Chain Lightning by screen pixels (checked in batch 2a; sprite offset Q43) and single-precision resistance (Q-0005); seven creature abilities, the hero's `magic_resistance`, BAT-MAG-003 to 008. Detail: `docs/epics/E04b-log.md`.

### 2026-10-05 (sixteenth) — E04b T2a and T2b: spell content, the hero's magic, casting

Sandbox session; the user confirmed T2a–T2d, T2a + T2b now. T2a (`92bbdd4`): pack format v4 `SpellMagic` for the 13 damage spells (flag word, gap-fill), hero effects `school_mastery`, `spell_damage`, `spell_book`, specialty growth (wiki SoD: 3 %/level), HER-BAT-011/012. T2b: `h3_battle::magic`, `Command::Cast`, BAT-MAG-001 to 004, formats, property tests with casts. Detail and decisions: `docs/epics/E04b-log.md`.

### 2026-10-05 (fifteenth) — E04b refined (increment 1 = T1 + T2), T1 done, ADR-0014 accepted

Refinement in the user's repo (no code): outline T1–T11, one epic in increments with a checkpoint each (Q41); increment 1 = T1 and T2 (T2a–T2d, then batch 2a); commit `3841f22`. Then branch `e04b-battle-magic`, T1's survey (VCMI 1.7.5, wiki, the spell flags; disputes M-1 to M-6, candidates X-1 to X-6, `f98fcc6`) and ADR-0014, revised after the user's first review and accepted (Q42).
