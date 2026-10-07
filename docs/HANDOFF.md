# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04b-battle-magic` on GitHub (pushed after every commit; Q49); increment 1 done (T1, T2a–T2e, batch 2a evaluated). **Increment 2 (Q46, Q48): T3 in parts, then T4; T3a, T3b1a, T3b1b, T3b2, T3c1, T3c2 done; this session T3c3 (batch 2b) and T4a (Anti-Magic, Magic Mirror), a checkpoint after** (Q52: T4 in three parts by mechanism). Sandbox: `docs/08-work-process.md`, "Cloud sandbox and syncing" (original files staged from the user's repo folder through the device link, `scripts/sandbox-setup.sh`).
- **GitHub and CI:** `ChristopherRotter/homm3` (private); `.github/workflows/ci.yml` runs `cargo xtask verify` on Linux for PRs, `master` and weekly; changes reach `master` by pull request; a `bench-check` re-baseline must hold on the CI runner.
- **E04b (battle magic) is refined** (Q41, `docs/epics/E04b-battle-magic.md`): increments with a user checkpoint each; only the current increment's tasks are refined.
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§15), ADR-0014 accepted (point 5 amended, Q47), design note `docs/epics/E04b-design.md`. Castable by heroes: the 13 damage spells, the 14 value spells, Shield, Air Shield, the Protections (Q-0007), Slayer, Counterstrike (Q-0008), Dispel, Cure, Fire Shield (Q-0010) and Frenzy (Q-0009) (BAT-MAG-001 to 019), the spell specialists (BAT-MAG-020), all after the executable's code (`installation.md` §7.8–§7.12). **Physical damage follows the executable's three truncated stages** under Q-0005 (Q50, BAT-DMG-002, `h3_kernel::arith::X87` at 53 bits).
- **Hero rates** cross the seam as `h3_contracts::Rate` (basis points plus, under Q-0005, the original's single-precision value as an f32 bit pattern where it differs; Q45 (b), T3c2).
- **Next:** T4a, then the checkpoint; batch 2b (`experiments/specs/E04b-batch-2b.toml`) is with the user, fold it in when the results are in (T4b Forgetfulness and Blind, T4c Berserk and Hypnotize).
- **Open for later tasks:** batch 2a R6's costs (T7); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7); the damage stages' creature factors; Dispel's and Fire Shield's loose ends (Sphere T8, obstacles T5, the Sultans' shield T7; BACKLOG); a hypnotized stack's controller for Fire Shield and the specialists (T4).
- Versions: state hash v23, ruleset hash v20, replay format v16, action encoding v9, C ABI v9, observation encoding v6; pack format v9.
- Working agreement: increments of a couple of tasks with a user checkpoint (Q41); the user confirms seams and ADRs; token efficiency beats speed (AGENTS.md); the user plays experiment maps in SoD with plain HD and commits the results.

## Standing traps

**Sandbox and sync**
- Original files in a cloud session: stage them from the user's repo folder (`D:\Users\Christopher\Source\Repos\homm3\original-data`, read access through the device link; one large LOD per staging call), then `scripts/sandbox-setup.sh`. Without the link the session only prepares (Q48). Sync is `git push` (fetch `master` first so the push is thin).
- Git on the user's machine: get delete permission for the repo folder before *any* git command (even `git status` leaves `.git/index.lock` otherwise).
- The sandbox VM can restart mid-session (files stay, background jobs and installs die): check `ps` and the logs before waiting on a background job.
- `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background and poll its log.
- bench-check at 0 Ir: valgrind 3.22 read no symbols from the lld-linked `h3` (the gate links it with `-z norelro` since T3b2).
- The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with its own `CARGO_TARGET_DIR`, and commit there on a branch that the clone fast-forwards to. `pkill -f <pattern>` also kills the tool's own shell when the pattern is in the command line: kill by pid.
- Broad sub-agent surveys burn tokens and hit rate limits: research in narrow steps.

**Sources**
- The wiki is reachable only through WebFetch (summarised) and needs the user's approval per session: ask for verbatim quotes, use `index.php?title=<Page>&action=raw` and `index.php?search=<term>&fulltext=1`; ask for the `swh` template's SoD argument, the summariser otherwise reports HotA.
- VCMI 1.7.5: `git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git` works through the proxy.
- Code reading (Q29): stage `Heroes3.exe`, `pip install capstone pefile`, disassemble `.text` with `skipdata` into one text file and grep it; known layouts in `docs/formats/installation.md` §7.5–§7.10. Check each float constant's width (`qword` double or `dword` f32); the x87 runs at 53 bits (§7.10), emulate it with `h3_kernel::arith::X87` and test vectors from exact rational arithmetic.
- `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run the harvester (`cargo run -p h3-harvest -- --install original-data/install --out packs`); never edit them by hand. Harvested keys may coincide across kinds (`base:ballista` is a creature and an artifact).

**Coverage, lints, formatting**
- LLVM counts the implicit `else` of `if let`, the error arm of `?` and rarely reached closures (`Option::map` on a rare `Some`): no defensive dead branches; use infallible accessors and combinators, saturate instead of returning `Option` (`WideUint`), make property tests deterministic in what they cover.
- Coverage misses on non-code lines mean stale binaries: `cargo llvm-cov clean --workspace`.
- clippy: `should_implement_trait` rejects methods named `div`/`shr`/`mul`/`add`/`sub` (hence `X87::times`, `plus`, `minus`); `type_complexity` fires on arrays of fn pointers in tests (use a type alias). `cargo fmt` reflows code: match scripted patches against the formatted text.
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

### 2026-10-07 (twenty-fourth) — E04b T3c3 and T4a (in progress)

Linked cloud session. The user's checkpoint (Q52): batch 2b now, T4 in three parts (T4a defences, T4b Forgetfulness and Blind, T4c control), T3c3 + T4a this session; `Rate` keeps the f32 bit pattern. T3c3 done: batch 2b compiled (six encounters, predictions checked against the engine; the user plays it while T4 proceeds). **In progress:** T4a (the code reading of Anti-Magic and Magic Mirror).

### 2026-10-07 (twenty-third) — E04b T3c1 and T3c2: spell specialists, Q45

Linked cloud session. The user classified Fire Shield's base as fix-optional (Q-0010), kept the breath's second stack as the rule, and chose T3c1 + T3c2 (Q51). T3c1: read 0x4E5FF0 (`installation.md` §7.12), pack format v9, BAT-MAG-020, ruleset hash v20 (old workload +0.05 %). T3c2: `Rate` across the seam, the derivation in x87 single precision, state hash v23, replay format v16 (+0.71 % Ir). Fixed the x87 property test's helper. Detail: `docs/epics/E04b-log.md` (T3c1, T3c2).

### 2026-10-07 (twenty-second) — E04b T3b1b and T3b2: Dispel, Cure, Fire Shield, Frenzy

Linked cloud session. The user chose T3b1b + T3b2 (checkpoint before T3c), Q-0008 fix-optional and the executable's Behemoth rounding under Q-0005 (BAT-DMG-006). Read in the executable: Dispel skips immunity and resistance; Fire Shield reflects from stage 1; Frenzy by own turns, S-16 confirmed (Q-0009). Pack formats v7 and v8, BAT-MAG-016 to 019; state hash v22 (`BattleState` 3 840 bytes); replays and `bench-check` re-baselined (old workloads +0.35 %, +0.53 %); the bench gate links `h3` without RELRO. Sphere of Permanence → T8. Detail: `docs/epics/E04b-log.md` (T3b1b, T3b2).
