# HANDOFF

Living state between sessions; every session reads it first. Rules (`docs/08-work-process.md`, "Keeping the start-of-session reading small"): rewrite **Current state** each session, keep **Standing traps** to traps that still apply, add one short entry (about ten lines) to **Recent sessions** and keep at most three. Detail belongs in the catalog, ledger, ADRs, epic log and commit messages; git history keeps old entries. `cargo xtask check-docs` enforces 10 000 bytes and three entries.

## Current state

- Branch `e04b-battle-magic` (created from `master` = the E04b refinement `3841f22`, on top of the merged E04a); increment 1 done (T1, T2a–T2e, batch 2a evaluated). **Increment 2 (Q46): T3 in three parts, then T4; T3a done 2026-10-06. Waiting for the user's checkpoint** (review the effect list and its measurements; confirm T3b, alone or with T3c). Sandbox and syncing: `docs/08-work-process.md`, "Cloud sandbox and syncing" (`scripts/sandbox-setup.sh`).
- **E04b (battle magic) is refined** (Q41, `docs/epics/E04b-battle-magic.md`): increments with a user checkpoint each; only the current increment's tasks are refined.
- **Done:** survey `docs/rules/surveys/E04b-magic.md` (§12–§14), ADR-0014 accepted (Q42; point 5 amended, Q47), design note `docs/epics/E04b-design.md` (T5 onwards). The 13 damage spells (BAT-MAG-001 to 008) and the 14 value spells (Haste, Slow, Bloodlust, Precision, Weakness, Stone Skin, Disrupting Ray, Prayer, Bless, Curse, Mirth, Sorrow, Fortune, Misfortune; BAT-MAG-009 to 011, `h3_battle::effects`) are castable by heroes, with mass versions, mind immunity and the Conjuring durations (HER-BAT-014), after the executable's code (`installation.md` §7.8, §7.9).
- **Next:** T3b (Shield, Air Shield, the four Protections with S-19, Fire Shield, Slayer, Frenzy with S-16, Counterstrike; Dispel, Cure), T3c (specialties in timed spells: STUB-0002; Q45 option (b); batch 2b), then T4.
- **Open for later tasks:** batch 2a R6's costs (T7 reads the cost code first); Q43 (sprite offset, keep the approximation); creature casters' re-casts (T7: the first value stays).
- Versions: state hash v20, ruleset hash v17, replay format v15, action encoding v9, C ABI v9, observation encoding v6; pack format v5.
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

### 2026-10-06 (nineteenth) — E04b T3a: the effect list and the value spells

Sandbox session; the user cut increment 2 (Q46: T3a alone, then a checkpoint; Q45 option (b) in T3c). Read the executable's timed-spell code (`installation.md` §7.9, survey §14): durations, what a cast changes and the end gives back, rounds fall at the round start, Disrupting Ray never ends; the user chose the executable's counter pairs and amended ADR-0014 point 5 (Q47). Built pack format v5, HER-BAT-014, BAT-MAG-009 to 011 (`h3_battle::effects`), observation v6, C ABI v9; golden replays and `bench-check` re-baselined (new workload; +0.17 % on the old one). Detail: `docs/epics/E04b-log.md` (T3a).

### 2026-10-06 (eighteenth) — E04b T2e: batch 2a folded in, increment 1 done

Sandbox session; the user committed batch 2a's results and chose T2e alone as the seam. Confirmed: Chain Lightning's screen geometry and ties, the damage order and rounding, immunities, the Ballista-turn cast. Refuted and fixed after re-reading the targeting code (`installation.md` §7.8): Chain Lightning only at enemies, area spells at any hex, no fizzle from an area spell's centre stack, the tactics hand-over (rounds with Wait, Q40); a stack that always resists is no target; `SpellDamage` carries value and caster damage. `full-s3` and `bench-check` re-baselined. Q45 opened. Detail: `docs/epics/E04b-log.md` (T2e).

### 2026-10-06 (seventeenth) — E04b T2c and T2d: the factorized action, the damage spells after the code

Sandbox session; the user confirmed T2c + T2d, then batch 2a, and kept T2b's decisions. T2c (`b7b152a`): 21 stack slots per side, action encoding v9, C ABI v8, observation v5. T2d: the executable's spell code read (`installation.md` §7.8; M-1, M-2, M-6 settled); the user decided (Q44) the specialty contract change, Chain Lightning by screen pixels (checked in batch 2a; sprite offset Q43) and single-precision resistance (Q-0005); seven creature abilities, the hero's `magic_resistance`, BAT-MAG-003 to 008. Then batch 2a compiled (spells in experiment specs). Detail: `docs/epics/E04b-log.md`.
