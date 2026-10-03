# HANDOFF

Living state. The last agent updates this at the end of every session; the next agent reads it first. Newest entry on top; keep old entries short (or prune) once superseded.

---

## 2026-10-03 — E04a T1 done (waiting for review); T2 smoke map in progress

**State:** branch `e04a-heroes-in-battle` (created from `master` by the agent with the user's OK). Commits: toolchain pin 1.97.0 (user decision; bench-check re-baselined), AGENTS.md directive (token efficiency beats speed; narrow sub-agents only), T1 docs. No code change in T1.

**T1 results:** `docs/formats/installation.md` §6–§7 (hero/skill/artifact/spell tables; exe tables: heroes with RoE/expansion flags, specialties, per-level skill values, artifact primary skills, spells; §7.5 is a code reading pending Q29); `docs/rules/surveys/` (README with the batch-1 dispute list, skills, artifacts, heroes, battle, quirks); ADR-0013 Proposed; glossary entries in 03-domain; OPEN-QUESTIONS Q29 (may agents read the game's code?) and Q30 (review ADR-0013, classify the 22 quirks).

**Stop rule:** nothing builds on ADR-0013 until the user accepts it (Q30). T2 does not depend on it.

**In progress:** T2 smoke map (minimal SoD `.h3m` writer in `h3-formats`, one encounter, written to a gitignored folder for the user to copy into the game's `Maps` folder).

**Traps:** the sandbox has only rustc 1.97.0 (now the pin; symlink `stable` as `~/.rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu`). Sub-agents that run broad surveys burn tokens fast and hit session limits (this session lost three of five survey agents to a rate limit); do research yourself in narrow steps. The wiki is only reachable through WebFetch (summarised): ask for verbatim quotes, use `index.php?title=<Page>&action=raw`, and the search page `index.php?search=<term>&fulltext=1` to discover pages. Our packs inherited three bugged elemental levels from the exe (quirk S-05).

---

## 2026-10-01 — E04 refined (planning session with the user; nothing implemented)

**State:** docs only, committed on `master` (E03 is closed and merged; this commit sits on top of `0a13da9`). No code, pack or test changed.

**What the session produced:**
- `docs/epics/E04a-heroes-in-battle.md` — the refined epic (goal, the entities called "hero", what the installation holds, 12 proposed tasks T1–T12, open design points for ADR-0013, exit criteria, risks).
- OPEN-QUESTIONS: **Q27** (scope and slicing, the hero boundary), **Q28** (experiments batched and automated, SoD only), **Q6** answered (quirk policy confirmed); Q21–Q23 and Q26 now point to the first experiment batch.
- BACKLOG: E04 replaced by **E04a** (refined), **E04b** battle magic and **E04c** sieges and special battlefields (sketches); inbox items re-targeted; four new inbox lines.
- `03-domain.md`: open question on the Heroes crate updated; `README.md` index date.

**Decisions (user):** three epics as allocated in the epic file; battle-oriented hero scope with no god hero type — a domain crate `h3-heroes` holding only the hero's lasting details (`HeroDetails`, the user disliked "sheet"; the name is provisional) and the translation to the battle's own seam type `BattleHero`; all battle spells in E04b (Earthquake with the sieges, E04c); **ADR-0013 in E04a T1, and work stops for the user's review**; experiments in batches, automated by compiling a spec into an `.h3m` (the user can verify SoD only, and staging many battles by hand is too tedious); quirks surveyed up front, classified by the user in one sitting.

**Agent refinements the user has not explicitly confirmed (change freely):** the epic IDs E04a/b/c; spell *data rows* (not effects) are harvested in E04a T3 because heroes and artifacts reference spells; the quirk survey covers all three epics so one sitting suffices; E04b designs a factorized action encoding and flags it to E05; the VCMI oracle harness stays deferred.

**Next:** the user creates branch `e04a-heroes-in-battle` from `master`, and an agent starts **E04a T1** (catalogue the hero/skill/artifact/spell tables and look for static tables in `Heroes3.exe`; source survey; quirk survey; ADR-0013 Proposed) — then stops for the user's review of the ADR and the quirk classification. T2 (experiment pipeline) can start after T1; its first deliverable is a smoke map that the user loads in the game.

**Traps:** the installation stays read-only: the generated `.h3m` goes into a gitignored folder in the repo and the user copies it into the game's `Maps` folder. Experiment results hold for SoD only (the rule catalog must say which version a rule was observed in). Do not run `git` on the user's machine from the device shell without delete permission for the repo folder (stale `.git/index.lock`; this session ran only `git log`, `git branch` and `git status` and left no lock). Wiki fetches were approved for E03; re-confirm at T1. The sandbox is fresh: VCMI 1.7.5 has to be cloned again (`git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git`).

---

## 2026-10-01 — E03 closed (T9)

**State:** E03 (battle core) is **done**. The user ran `cargo xtask verify` (incl.
`--with-original-data`) on Windows: green; Linux was green in the sandbox. **ADR-0012 accepted**
by the user. Branch `e03-battle-core` (this commit on top of `c208a02`) is ready for the user to
fast-forward into `master`.

**T9 (this session):** T8 docs; draw on simultaneous wipe-out (BAT-END-002, Q24: `BattleResult`
enum in the seam, `BattleOutcome.result`; it also fixes an endless loop when only war machines
were left on both sides); tight formation (Q25: `BattleSetup.formations`, battle params
`loose_formation`/`tight_formation`, scenario/replay field `tight_formation`); observation
encoding v3 and C ABI v2 (`H3StepResult.result` with `H3_RESULT_*`; the ABI version now tracks
the encodings — T3–T8 had changed them without a bump); synthetic war machines in the `full`
scenario; lazy Ballista target choice (property-tested against the eager VCMI ranking);
golden replays re-recorded, bench re-baselined (≈ 197 k steps/s on the full scenario).
Review pass (two agents) and fixes: compiler rejects certain bad-morale freezes (they could
hang `BattleState::new`), `ammo_supply`/`first_aid` without flag `war_machine` and life drain
above 1000 %; overflow-safe life drain; `BattleState::start(…, sink)` reports the first
activation's events; `SetupError::UnknownTerrain`; fixed-capacity buffers in war-machine turns;
CLI validates pack ids; catalog wording fixed. Q26 (First Aid Tent heal: wiki 1–25 vs VCMI 10)
decided by the user: wiki. Remaining review findings are in the BACKLOG inbox (death-stare
work per attack, random obstacles in the sim/FFI, seed secrecy for server mode, war-machine
position overlap check, checked arithmetic for the Ballista ranking in E04).

**Versions now:** action encoding v4, observation encoding v3, C ABI v2, replay format v2,
state hash v8, ruleset hash v8.

**Next:** E04 (heroes, artifacts, skills, spells; also the E03 deferrals: special
battlefields, spell-like abilities and immunities, tactics, sieges/Catapult/towers, hero
control of war machines). Refine E04 with the user first. *(Superseded: E04 was refined and split into E04a/b/c, see the entry above.)* Open experiments: Q21–Q23 (and Q26 to
confirm), whenever observation runs in the original are allowed again.

**Traps (still valid):** see the E03 entry below (coverage of closures/`if let`, stale coverage
binaries, abilities via `ruleset.abilities`). New: a test ruleset may not make bad morale
certain (`tests/morale.rs::always` uses 254/255); `BattleState::new` drops the first
activation's events, use `start` where events matter.

**Sandbox notes (this session):** fresh sandbox; tooling as in the E02 notes (toolchain symlink
`1.95.0`, `cargo install cargo-deny cargo-llvm-cov --locked`, `llvm-20`, VCMI 1.7.5 clone at
`/home/claude/ref/vcmi`); original data staged from the user's `original-data/install` and
symlinked; syncing via bundles in the user's `target/` and a fast-forward with delete
permission for the repo folder (git lock files).

---

## 2026-10-01 — E03 T1–T8 (superseded by "E03 closed" above; kept for the per-task notes)

**State:** branch `e03-battle-core` from `master` (`7c02e2a`). Scope decisions at the start (user, OPEN-QUESTIONS Q16): war machines without heroes and terrain bonus + obstacle generation are **in**; spell-like abilities, spell immunities and tactics are **E04**; sources VCMI 1.7.5 + wiki (fetches approved); no original-game experiments (open questions instead); ADR-0012 reviewed with the E03 report. Task list and status: `docs/epics/E03-battle-core.md`.

**Done:** T1 — ADR-0012 (modifiers and abilities, Proposed) and `h3_kernel::modifier` (KER-MOD-001; `FixedVec::remove_if`). T2 — `h3_content::Ability` (20 kinds), pack syntax (`packs/README.md`), compiler checks, ruleset hash v3, flags/shots moved to `CreatureType`, harvester `creature_abilities` gap-fill entries (sourced) → packs regenerated; golden replays re-recorded (only the ruleset hash changed).

**User decisions during E03 (OPEN-QUESTIONS):** Q17 per-side "led by a hero" flag in `BattleSetup` + quirk "no luck without hero" (T6); Q18 luck only on the primary target; Q19 Behemoth: defense counts floor(D·(100−p)/100); Q20 regeneration at the stack's own turn.

T3 — footprints/two-hex, obstacles (`BattleSetup.obstacles`, scenario `obstacles = [...]`), flying and teleporting, attack positions (24 canonical slots), action encoding v2, replay format v2, golden replays re-recorded (skirmish now has obstacles), bench baseline updated.

T4 — ranged combat (BAT-RNG-001..004), damage factors generalised (`damage::Factors`), `combat.rs` for strike sequences, `StrikeFlags` in `Struck`, battle params `ranged_full_damage_distance`/`ranged_penalty_percent`/`melee_penalty_percent` (packs regenerated, ruleset hash v4), state hash v3, action encoding v3, CLI command `shoot`.

T5 — melee abilities: double attack, attack patterns (all adjacent / three-headed / breath, VCMI port), strike and return, no enemy retaliation, extra/unlimited retaliations, hate, jousting (path length) + charge immunity, defense reduction (Q19), death blow, life drain, death stare, regeneration (Q20); events `LifeDrained`, `DeathStared`, `Regenerated`; `Stack.base_count`/`retaliations`/`retaliations_left`; state hash v4; property test "ownership isolation" now checks that every changed enemy stack is explained by an event. New open questions Q21 (death stare top HP), Q22 (jousting distance), Q23 (float vs exact damage rounding), each with an experiment; the agent followed VCMI / kept exact integers meanwhile.

T6 — morale and luck: `h3_battle::morale` (modifier pool on sides, materialised `Stack.morale`/`luck`, rolls), good morale (extra turn, also after acting in a wait phase — wiki and VCMI agree), morale-and-wait phase, bad morale freeze (also at the first activation in `BattleState::new`), lucky strikes (primary target only, Q18), `BattleSetup.led_by_hero` + quirk Q-0003 `quirk.battle.no_luck_without_hero` (faithful on), roll tables in `[battle]` (`morale_dice`, `good_morale_chances`, `bad_morale_chances`, `luck_dice`, `good_luck_chances`), compiler limit of 8 distinct aura values, ruleset hash v5, state hash v5. Gargoyles keep their morale (original flag table and wiki; VCMI exempts them). The default test ruleset has all morale/luck chances 0; morale tests and the property tests use `ruleset_with`.

T7 — terrain: content types `terrain`, faction `native_terrain`, `obstacle` (pack format, compiler checks, ruleset hash v6), harvester reads `TERRNAME.txt` and gap-fills native terrains and VCMI's obstacle table (special-battlefield-only obstacles left out), `BattleSetup.terrain` + native bonus (BAT-TER-001), `h3_battle::generate_obstacles` (BAT-GEO-004, kernel stream `OBSTACLES` = 3), CLI `load_scenario(path, packs, seed)` generates for `random_obstacles` scenarios (the synthetic duel now has terrain + random obstacles; the replay records the generated hexes and the terrain). Special battlefields (Holy Ground, Cursed Ground, …) are E04 (BACKLOG).

T8 — war machines without heroes (BAT-WAR-001..004): `h3_battle::war` (automatic turns: Ballista target choice ported from VCMI `tryMakeAutomaticActionOfBallistaOrTowers`, First Aid Tent heals a random hurt ally 1..=25), Ammo Cart (`combat::ammo_supplied`: no shots used while it lives), war machines placed at `war_machine_positions` after the army (slots 16 + kind), never retaliate, no morale, do not count for alignment or the end of the battle; `BattleSetup.war_machines` / `BattleOutcome.war_machines` (contract change, covered by the Q16 scope decision), CLI scenario/replay fields `attacker_war_machines`/`defender_war_machines`, `STACK_CAPACITY` 38 → action encoding v4, observation encoding v2 (layout unchanged apart from the slot count), ruleset hash v7, state hash v7, packs regenerated, golden replays re-recorded (ruleset hash only). `cargo xtask verify` is green after the follow-up commit (coverage needed one more test ability; bench-check passed without a re-baseline).

**Traps (E03):** if `cargo xtask coverage` reports misses on lines that are not code (struct definitions), stale test binaries from older builds are being merged: run `cargo llvm-cov clean --workspace` and retry. Rule code reads abilities via `ruleset.abilities(creature)` (slices, no lookups by key); cheap per-attack booleans are cached as `StackTraits` bits at setup. Coverage counts closures and `if let` else paths: keep defaults eager (`unwrap_or`, `map_or`) where a branch cannot run.

**Sandbox notes (this session):** VCMI 1.7.5 is cloned at `/home/claude/ref/vcmi` (`git clone --depth 1 --branch 1.7.5 https://github.com/vcmi/vcmi.git`; github.com web pages are blocked, git works). The wiki is reachable only through WebFetch after the user approved it.

---

## 2026-10-01 — E02 closed

**State:** E02 (content pipeline) is **done**. The user ran `cargo xtask verify --with-original-data` on Windows: green. Branch `e02-content-pipeline` (last commit before this one: `df5a0d8`) is ready for the user to fast-forward into `master`.

**Decisions (user):** Q8 (installation = GOG Complete 4.0; RoE/AB/SoD as packs `base` → `ab` → `sod` in a dependency graph), Q13 (harvest once, commit the clean packs: gameplay data and names only; revises Q9), Q14 (keep TOML), Q15 (historical balance at the latest patches). Recorded in ADR-0005/0006/0009 amendments and OPEN-QUESTIONS. Agent decision noted in Q13: level/town/flags come from the executable's static creature table.

**What exists now:**
- `crates/h3-formats` (adapter): LOD archives, text tables, the exe's creature table; property tests on arbitrary input.
- Pack format v2 (`packs/README.md`): enabled packs in any order, dependency graph and closures, overrides, resources/factions, creature info and flags (`CreatureFlag`), ruleset hash v2.
- `crates/h3-harvest` (tool) + `data/gapfill.toml` (sourced facts) → committed `packs/base` (118 creatures), `packs/ab` (23 + Conflux), `packs/sod` (3 overrides). `cargo xtask verify --with-original-data [path]` re-harvests and requires identical packs.
- Catalogue of the installation: `docs/formats/installation.md`.

**Next:** E03 battle core (epic refined with the E02 findings, section "Inputs from E02"), starting with T1: modifier-system ADR + kernel implementation. New branch `e03-battle-core` from `master` once the user has fast-forwarded it.

**Traps / notes:**
- The packs under `packs/base|ab|sod` are generated: change `crates/h3-harvest` or its gap-fill file and re-run `cargo run -p h3-harvest -- --install original-data/install --out packs`, never edit them by hand.
- War machines are not harvested (BACKLOG); abilities beyond flags are not in data yet (E03 T2).
- Cloud sandbox: `static.rust-lang.org` is blocked, so link the sandbox's `stable` (1.95.0) as `~/.rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu`; coverage works with Ubuntu's LLVM 20 (`apt-get update && apt-get install llvm-20`, then `LLVM_COV=/usr/lib/llvm-20/bin/llvm-cov LLVM_PROFDATA=/usr/lib/llvm-20/bin/llvm-profdata`). For `--with-original-data` in the sandbox, stage `Heroes3.exe`, `Data/H3bitmap.lod`, `Data/H3ab_bmp.lod` from the user's `original-data/install` and symlink them there.
- Syncing to the user's repo: use a new bundle file name for each sync (re-committing to the same device path once left a stale file); the user's repo has no remote, so "unpushed commits" warnings about the sandbox clone are expected.

---

## 2026-09-30/10-01 — E02 sessions (superseded by "E02 closed" above)

T1 catalogue and decisions, T2 `h3-formats`, a user-requested checkpoint with T3 parked on a WIP branch (later folded into the T3 commit), then T3–T5 and the E03 refinement. Details are in the commit messages and the epic file.

---

## 2026-09-30 — E01 closed (final commit)

**State:** E01 is **done**. The user ran `cargo xtask verify` on Windows: green (ffi-smoke and bench-check N/A by design). Branch `e01-walking-skeleton` is ready for the user to fast-forward into `master`.

**Decisions recorded in this commit (user):**
- Q11: attack-bonus cap **+300 %** (BAT-DMG-002). Q12: large-stack damage roll per the **wiki** (BAT-DMG-001; for exactly 10 creatures both formulations coincide, test added).
- Q9: **import-only**. Copies of original files go to the repo's `original-data/` (gitignored; `check-arch` fails if anything there is tracked). ADR-0009 amended.
- Q8 (partly): the installation is available (on the user's machine at `D:\Heroes3\HoMM 3 Complete`, connected to sessions as an extra folder; it was not yet connected to the session that made this commit). Its version is identified in E02 T1.

**Rule for every agent from now on:** the installation folder is **read-only** (the user cannot enforce it). Never edit, create, move or delete anything in it, never run tools that write there, never request delete permission for it. Copy into `original-data/` and work on the copies (AGENTS.md, ADR-0009).

**Next:** E02 (content pipeline), starting with T1 (catalogue the installation, identify the version).

**Process addition (after closure):** `AGENTS.md` now has "Interruptions and resuming": sync every commit to the user's repo immediately, keep HANDOFF resumable, checkpoint when the user announces a stop, and rebuild from the user's repo when resuming (the user amended the closing commit in their repo; agents must build on the user's history, not on a stale sandbox copy).

**Trap:** on the user's machine, running `git status` from the agent's device shell without delete permission leaves a stale `.git/index.lock` (the shell may create but not delete files in connected folders). Prefer doing git work in the cloud workspace and syncing with a bundle; if you must run git on the device, get delete permission for the repo folder first.

---

## 2026-09-30 — E01 walking skeleton (one session, with the user present at the start)

**State of the repo:** branch `e01-walking-skeleton` (one commit per task plus a review-fix commit, on top of `master`), all E01 tasks T1–T10 done. An independent review pass at the end found no rule or validation bugs but several gate loopholes (test-code classification, header check by substring, skipped steps exiting 0, fail-open coverage/bench checks); all fixed in the last commit. `cargo xtask verify` is **green on x86_64 Linux** (all 12 steps, none skipped); the Windows run was done by the user afterwards (see the entry above).

**What was done:**
- Workspace: `h3-kernel`, `h3-content`, `h3-contracts`, `h3-battle` (logic, `no_std` + `alloc`, 100 % line+region coverage); `h3-pack`, `h3-sim`, `h3-ffi`, `h3-cli` (adapters); `xtask`. Graph and purity rules in `arch.toml`.
- Gates: `cargo xtask verify` = fmt, clippy, deny, check-arch, check-rules, check-stubs, test, coverage, determinism, ffi-smoke, bench-check, doc-drift (table in `docs/05-testing.md`). Single steps: `cargo xtask <step>`; `cargo xtask help`.
- Kernel: xoshiro256\*\* + SplitMix64 streams, FNV-1a canonical hashing, `Real` float surface, `FixedVec`, `Handle`, total-order sort helper.
- Battle: 13 sourced rules (`docs/rules/catalog/battle.toml`): geometry, placement, turn order with wait phase and speed ties, Defend, walking, melee, damage (roll, attack/defense modifier, top-creature-first), retaliation, end of battle. Commands validated before mutation; `legal_commands`; `observe`/`project`; events with visibility; property tests for every mandatory invariant.
- Sim/FFI/CLI: `BattleEnv` (449-action encoding, 612-float observation, truncation, rewards, fork), random legal agent, replay model; C ABI v1 with header and C smoke program; `h3 run --record`, `h3 verify`, `h3 bench`; golden replays in `fixtures/replays/`.
- Performance: ≈ 245 k random-legal steps/s/core, 944-byte state, ≈ 90 ns clone; instruction-count baseline in `benches/baseline.toml` (+2 %).

**Decisions taken (user):** Q1 → GPL-2.0-or-later (ADR-0009 accepted, `LICENSE`); Q3 → no CI config yet, feature branch per epic. **Decisions taken (agent, recorded as ADR amendments):** toolchain 1.95.0; `no_std` logic crates; `h3-pack` adapter crate; PRNG/hash algorithms; release overflow checks; geometry seam as a concrete type; step limits in the sim, not the domain; `design` source kind; valgrind-based benchmark gate; dev-dependencies excluded from the license check.

**Next:**
1. **User:** fast-forward `master` to `e01-walking-skeleton` if happy, and run `cargo xtask verify` once on Windows (needs `cargo install cargo-deny cargo-llvm-cov --locked` and `rustup component add llvm-tools-preview`; ffi-smoke and bench-check report N/A there). That closes E01 exit criteria 1–2.
2. **E02** (content pipeline) needs the user's installation and answers to Q8/Q9 (and Q11 for a parameter value). E03 is refined too. *(Superseded: see "E01 closed" above.)*

**Open questions added:** Q11 (attack-bonus cap 300 % vs 400 %), Q12 (large-stack damage roll: wiki vs VCMI) — both answered at closure.

**Traps / notes for the next agent:**
- **Coverage is strict about dead branches:** LLVM counts the implicit `else` of `if let` and the error arm of `?` as regions. After validation, use infallible accessors (`BattleState::at`/`at_mut`) or combinators; do not add defensive branches that cannot run.
- `cargo fmt` reflows code: when patching files with scripts, match against the formatted text (and check that the patch applied).
- Coverage tooling: `cargo llvm-cov` needs `llvm-tools-preview` or `LLVM_COV`/`LLVM_PROFDATA` pointing to LLVM tools that read rustc's profile format (in the E01 sandbox: Ubuntu's LLVM 20 worked with rustc 1.95's LLVM 22).
- Golden replays and `bench-check` baselines change whenever behavior, the synthetic pack, an algorithm/encoding version or (for instruction counts) the toolchain changes. Regenerate deliberately (`fixtures/README.md`, `cargo xtask bench-check --update`) and say why in the commit.
- Wiki/VCMI conflicts: the source hierarchy ranks VCMI above wikis; for real gameplay conflicts ask the user or record an observation fixture (Q11 and Q12 were settled that way: the user chose the wiki).
- The battle has no round limit (faithful). Random agents mostly walk around; `h3 bench`/rollouts use step limits from the sim layer.
- `Claude outputs/AGENTS.md` in the repo is an older copy of `AGENTS.md` (not updated by this session).

## 2026-09-30 — E00 foundations (planning session with the user) — superseded by the E01 entry above

**State of the repo (then):** documentation only.

**What was done:**
- Defined vision, architecture principles, bounded contexts, source-of-truth and provenance approach, quirks/profiles, testing gates, security model, simulation/AI strategy, work process.
- Recorded ADR-0001 … ADR-0011.
- Wrote the backlog (coarse) and refined E01 (walking skeleton).

**What is next:** Start E01 (`docs/epics/E01-walking-skeleton.md`), task T1 (workspace scaffold + `xtask verify` + CI skeleton).

**Waiting on the user (see OPEN-QUESTIONS):**
- Q1 license choice and VCMI policy — interim: do **not** copy/translate VCMI code. No `LICENSE` file has been added yet; add one once decided (E01 T1 may proceed without it, but note it).
- Q2 project/crate naming (working prefix `h3-`).
- Q8 which game version is the "original" baseline; Q9 whether base content tables may be committed.
- Others are lower priority and do not block E01.

**Traps / notes for the next agent:**
- Do not assume original data file names or layouts from memory; catalogue them from the files the user supplies (E02).
- Keep the skeleton *minimal*: its purpose is to exercise the architecture, not to be a nice battle system. Resist adding spells, heroes, or abilities.
- All tests in CI must run on synthetic content packs (neutral invented names).
- Determinism/numeric policy in ADR-0003 is strict: integers by default, floats only via `kernel::num` (no platform transcendentals, no fast-math), no `HashMap`/`HashSet` in logic crates, every sort has a unique tie-breaker.
- The Rust toolchain/versions are not pinned yet; pin in `rust-toolchain.toml` during T1 and record the MSRV policy in ADR-0002 if it needs refining.
