# Archive: superseded status docs (archived from the Project doc `claude/archive-status-docs.md`, 2026-10-24; copied into the repo 2026-10-31)

These were per-epic or per-session status snapshots. All were superseded by `docs/HANDOFF.md`, the lane HANDOFF/QUEUE files and the epic files and logs, which are authoritative. Kept so nothing is lost; not meant to be read at session start.

Contents: e01, e02, e03, e04a, e04-refinement, e04b, e05-refinement, exe-map, parallel-lanes-handover, lane-server, lane-ai.

---

## e01-walking-skeleton-status.md (closed 2026-09-30)

Branch `e01-walking-skeleton` (latest commit `8e913f6`).

- Rust workspace: logic crates `h3-kernel`, `h3-content`, `h3-contracts`, `h3-battle` (`no_std`, 100 % line+region coverage); adapters `h3-pack`, `h3-sim`, `h3-ffi` (C ABI v1), `h3-cli` (`h3 run/verify/bench`); tooling in `xtask`.
- `cargo xtask verify` green on Linux and Windows: 12 gates incl. coverage, determinism matrix, C ABI smoke test, instruction-count benchmark gate.
- Battle: 13 sourced rules; about 245 k steps/s per core.
- Decisions: GPL-2.0-or-later (ADR-0009); no CI yet; one feature branch per epic; Rust 1.95.0 pinned; attack-bonus cap +300 % (Q11); large-stack damage roll follows the wiki (Q12); original data is import-only (Q9), copies in gitignored `original-data/`, `check-arch` fails if anything there is tracked.
- Rule: the game installation (`D:\Heroes3\HoMM 3 Complete`) is read-only for agents: never edit, create, move or delete there; copy into `original-data/`.
- Rule: interruptions and resuming (`AGENTS.md`, "Interruptions and resuming"): sync every commit right away, keep `docs/HANDOFF.md` resumable, checkpoint when the user announces a stop, rebuild from the user's repo on resume.

## e02-content-pipeline-status.md (closed 2026-10-01)

Branch `e02-content-pipeline`, latest commit `7c02e2a`.

- `cargo xtask verify --with-original-data` green on Linux and Windows.
- Installation catalogue `docs/formats/installation.md`: GOG Complete 4.0.
- `h3-formats`: LOD archives, text tables, the exe's static creature table. Pack format v2 (dependency graph and closures, overrides, resources, factions, creature info and flags, ruleset hash v2).
- `h3-harvest`: sourced gap-fill file plus committed packs: `base` (RoE at its latest patch, 118 creatures), `ab` (Conflux, 23 creatures), `sod` (3 SoD-only overrides).
- Decisions: Q8 RoE/AB/SoD are packs `base` → `ab` → `sod`; Q13 harvest once, commit clean packs (gameplay data and names only); Q14 TOML; Q15 historical balance at the latest patches.
- Traps: packs are generated, never edit by hand (change the harvester or `gapfill.toml` and re-run); cloud sandbox: link `stable` (1.95.0) as the 1.95.0 toolchain, Ubuntu's LLVM 20 for coverage; new bundle file name per sync.

## e03-battle-core-status.md (closed, 2026-10-01)

Branch `e03-battle-core`, last commit `0a13da9`; ADR-0012 (modifiers and abilities) accepted.

- T1–T9: modifier ADR and `h3_kernel::modifier`; creature abilities as data (22 kinds); two-hex stacks, obstacles, flying/teleporting, 24 attack positions; ranged combat; melee abilities; morale and luck, quirk Q-0003; terrain types; war machines without heroes; draws (Q24 `BattleResult`), tight formation (Q25), observation encoding v3, C ABI v2, faster Ballista target choice, review fixes.
- 45 battle rules (49 with kernel rules); about 197k random-legal steps/s per core (target 100k). Versions then: action encoding v4, observation v3, C ABI v2, state hash v8, ruleset hash v8.
- Decided: Q16–Q20, Q24, Q25, Q26 (First Aid Tent heals 1–25 per the wiki). Open then, each with an experiment: Q21 (death stare top HP), Q22 (jousting distance), Q23 (exact vs float damage rounding).
- Moved to E04: special battlefields, spell-like abilities and immunities, tactics, sieges/Catapult/towers, hero control of war machines.
- Review backlog: bound the work of a single attack; random obstacles in sim and C ABI; keep the seed secret in server mode; compiler check for overlapping war-machine positions.

## e04a-status.md (2026-10-05, epic closed and verified)

Branch `e04a-heroes-in-battle`, last commit `148ed64`.

- Last session: `cd71ba0` two robustness tests; `c2da94c` closure (design note deleted into `docs/epics/E04a-log.md`); `148ed64` user's Windows verify recorded; new open question **Q40** (tactics hand-over order and what "next creature" does, with an experiment); stale `Claude outputs/AGENTS.md` removed.
- Experiment candidates: Q40 (hand-over follows a regular round's order but "next creature" seems to act like Wait; speed tie by army slot or current hex); Spirit of Oppression vs Minotaurs' minimum morale; luck plus Artillery double damage (added or multiplied); surrender price with Ammo Cart and First Aid Tent; may a shooter with the Bow of the Sharpshooter still melee when adjacent. Also unobserved: the Elixir's regeneration cap of 50, Tactics zones for level differences 1 and 2; Q22 stays open.
- Decisions: Q27–Q39: three-epic split, experiment pipeline, ADR-0013 accepted, soundness-only checks of a battle hero (Q31), surrender without consent (Q32), task splits (Q33–Q35, Q37, Q38), `FactionGroup` (Q36), `observe` with the ruleset (Q37), death-stare cutoff (Q39).

## e04-refinement-status.md (planning done 2026-10-01)

Decisions (user, 2026-10-01):

- Three epics: E04a heroes in battle without magic, E04b battle magic, E04c sieges and special battlefields (Q27).
- Hero boundary: hero in battle and hero on the map are distinct entities; no "god" hero type, no type named plain `Hero`. `h3-heroes` holds only the hero's lasting details (`HeroDetails`) and the translation to the battle's seam type `BattleHero` (in `h3-contracts`). Map presence is an Adventure type later (E08).
- Hero scope battle-oriented; no equip/learn/level-up commands yet.
- Spells: all battle spells in E04b; Earthquake with sieges. Seam: ADR-0013 in E04a T1, then stop for review.
- Experiments (Q28): batched and automated; the user can verify **SoD only**; the engine compiles a spec into an `.h3m` map the user loads and plays. First batch Q21, Q22, Q23, Q26 plus T1 disputes.
- Quirks: up-front survey in T1, the user classifies in one sitting; Q6 confirmed.
- Unconfirmed agent refinements: spell data rows harvested in E04a T3; the quirk survey covers all three epics; E04b designs a factorized action encoding and flags it to E05; the VCMI oracle harness stays deferred.
- Traps: the installation is read-only (generated maps go to a gitignored folder, the user copies them to the game's `Maps`); experiment results hold for SoD only; git on the user's machine from the device shell needs delete permission for the repo folder (lock files) and `-c user.name/user.email`; fresh sandbox: re-clone VCMI 1.7.5, re-confirm wiki fetch approval.

## e04b-status.md (2026-10-09, increment 11 = T7c1 done)

Decisions (user): Q41 one epic in increments with a user checkpoint each; Q42 ADR-0014 accepted; Q44–Q48 and Q50–Q54 (spell specialty rates, counter pairs, x87 at 53 bits, T3/T4/T5 splits, quirk flags Q-0007 to Q-0012); Q55/BAT-Q4/BAT-Q5 natural obstacles cross the seam as pieces with optional anchor (BAT-QUIRK-3), hidden-object draws follow the original (BAT-QUIRK-4); BAT-Q6 T6 before T7, BAT-QUIRK-5 fix-optional, BAT-QUIRK-6 (S-20); BAT-Q7/QUIRK-7/8 created stacks' kills in the outcome; BAT-Q8/Q9 Bind's binder on a killed stack not reproduced, S-14 fixed with no flag; BAT-Q10 increment 7 = T7b1 alone, `CreatureCast { target }`; BAT-Q11 increment 8 = T7b2 in two parts, drawn spell private to its controller, BAT-QUIRK-9 fix-optional; BAT-Q12 a spell book's presence is public, BAT-QUIRK-10 the rule; BAT-Q13 Pit Lord's Demons as `StackOrigin::Raised(slot)`, created stacks as `StackOrigin::Created` (BAT-QUIRK-11); BAT-Q14 increment 10 = T7b4 alone, S-12 fix-optional (BAT-QUIRK-13 `quirk.battle.periodic_first_stack`), Enchanter counters ready each battle (BAT-QUIRK-14); BAT-Q15 increment 11 = T7c1 alone, spell cost reads the armies (BAT-QUIRK-15), Familiars' loop bound always fixed (BAT-QUIRK-16).

Done: increments 1–6 (T1–T7a, batches 2a–2d); 7 T7b1 (BAT-MAG-048, #19); 8 T7b2a Faerie Dragon (BAT-MAG-049, #23), T7b2b Master Genie (BAT-MAG-050, #25), shared #22, #24; 9 shared #26, T6c #27, T7b3 #31; 10 T7b4 Enchanter (BAT-MAG-053, #36), shared #35; 11 T7c1 mana creatures (BAT-MAG-054 to 056, #38), shared #37. Next then: T7c2 (Faerie Dragons' 20 % mirror, Efreet Sultans' shield, Phoenix rebirth, elementals' damage factors).

## claude/e05-refinement-status.md (2026-10-18)

PR #245 `shared/ai-lane` (`cross-lane`). Decisions (user, 2026-10-18, AI-Q1):

1. Battle only (no adventure ABI; E13).
2. The AI lane opens: `h3-sim` (its `src/adventure/` stays adventure's), `h3-ffi`, `h3-cli/src/ai/`, `python/`, `benches/ai/`, E05, E06 (`lanes.toml`). Battle rules that change an encoding become `cross-lane`.
3. Q5: Python + PyTorch 2.14 (cu126), the last prebuilt release for the GTX 1070 (Pascal); JAX lacks Windows GPU; burn/candle noted.
4. Opponents: self-play and the system actor (neutrals; E23's policy) in an internal seat; random placeholder for tests only.
5. Casts: one call per head for the whole batch; sub-steps as fallback.
6. Observation: flat v21 first; entity sets and incremental encoding noted.
7. Encounter distribution v1 by approximate frequency, derived from the 160 shipped maps plus stated assumptions.
8. Heuristic bots: baselines only.
9. Reward: zero-sum value balance in `ai_value`, terminal, configurable weights.
10. Agents see what a human sees (projected observation); fork/hash are tools only.
11. Performance benchmarked, no fixed target.
12. E05 exit: a smoke PPO run wins ≥ 80 % vs random on held-out seeds (provisional); proficiency targets in E06.

Order: E05 → E23 → E06. Plan: T1 design + ADR draft (`docs/lanes/ai/adr-drafts/battle-env.md`) then stop for review; T2 scenarios through C ABI v26; T3 batch API; T4 reward; T5 encounter distribution; T6 Python binding; T7 baselines; T8 smoke training; T9 closure. The user's four own maps were removed from `original-data/maps.tar` and `install/Maps`; 160 shipped maps remain. Traps: global HANDOFF at 5 996 of 6 000 bytes; `verify --scope ai` skips golden replays and battle benchmarks (run `cargo xtask determinism` and battle bench-check after `h3-sim` changes); a `lanes.toml` change needs the full verify (fingerprint).

## claude/exe-map-status.md (2026-10-09, `shared/exe-map`, PR #30; trial note 2026-10-12)

Authoritative: `docs/exe/README.md`, `docs/04-rules-and-sources.md` §2.

- **Decompile-backend trial (2026-10-12, PR #133, `battle/exe-decompile-trial`): drop.** Ghidra 12.1.4 headless via REA (`rea-agents@6.4.0`) on 9 functions. Pseudocode only ~2.5× smaller than the listing (rule asks 3); silent errors: x87 values fed to `_ftol` lost, a reused argument slot drops Curse's `max(1, …)`, unknown callee convention hides `this`; REA adds provenance only; each CLI call ~200 s. Ghidra must be staged by the user (trimmed x86-only zip in `original-data/ghidra/`). Report: `docs/lanes/battle/exe-decompile-trial.md`. Open follow-up: in `structs.toml`, `battle_manager` +0x54A4 should be `u8[2]`, not `i32[2]`.
- **User's decisions (2026-10-08, Q57; addendum 2026-10-09):** build a shared exe map; H3API (MIT, 3.2 build) is a source of guesses; code reading may be preferable to experiments (Q29's "last resort" lifted; ADR-0009 amendment 3). Review of PR #30: a reading ranks with `observation`, which wins a conflict; readings at risk get a confirmation experiment queued; a quirk flag confirmed by a reading alone needs a second look; a reading names the caller path it followed; a `code-reading` catalog kind left to a later lane task (63 battle citations would move).
- **PR #30:** `scripts/exe.py` (commands `fn`, `switch`, `callers`, `callees`, `refs`, `grep`, `lookup`, `data`, `listing`, `cited`, `check [--fix-sizes]`, `stats`, `port-h3api`; ~3 s per command; needs staged `Heroes3.exe`, `pip install capstone pefile`, Python 3.11+); `docs/exe/map.toml` (253 entries; query with `lookup`, never read whole); `docs/exe/structs.toml`; `docs/exe/h3api.toml` (generated, 190 KB, never read whole; known miss `Rand(low, high)` now 0x50B230); gate `cargo xtask check-exe-map` plus `scripts/exe.py check` under `--with-original-data`.
- Review changes: 0x468860 → `stack_stats_dialog`; 0x46A4A0 → `battle_is_hidden`; `game_pointer` +0x1F698 is `mapKind` (0 RoE, 1 AB, 2 SoD).
- Tool ideas (in `docs/exe/README.md`): struct field names in `fn`; `callees --depth`; a sweep cache keyed by exe SHA and map mtime.

## claude/parallel-lanes-handover.md (2026-10-07; Phase 0 = PR #2)

Agreed (user, 2026-10-06/07):

- **Lanes with path ownership**, one agent at a time per domain lane: battle (`h3-battle`, `h3-heroes`, `h3-sim`/`h3-ffi` until the AI lane, fixtures, battle bench, BAT/HER rules, quirks ledger, experiments), adventure, server (`h3-protocol`, `h3-server`), client (`clients/`), later formats/tools and AI. Shared paths change only via small separate PRs that merge first: `h3-kernel`, `h3-contracts`, core of `h3-content`/`h3-pack` (for now stay with battle), `arch.toml`, `Cargo.toml`, `xtask`, ADRs, the BACKLOG table. `h3-heroes` moves to adventure at a handover checkpoint when hero progression starts.
- **Repo layout:** the root stays the engine workspace; clients in `clients/`, a separate Cargo workspace with its own lockfile (`exclude = ["clients"]`), own CI job; asset decoding in `clients/crates/h3-assets`. The core stays free of graphics.
- **Client:** Rust by default; walking skeleton with real assets from the start; assets read from the user's installation at run time, never in git or CI; decoders tested on small synthetic sprites plus a local real-asset gate; headless render-to-PNG mode for agent self-checks.
- **Timing:** client/server implementation waits until battle actions are mostly stable (E04b); refinements start now.
- **Checkpoints:** domain lanes per increment; client/server per milestone under an autonomy contract with three tiers (agent decides, provisional / queue for milestone review / stop the line: seams, heavy dependencies, licence, anything touching another lane); a fresh reviewer agent checks each milestone.
- **Long unattended runs:** each lane has a `QUEUE.md` of pre-refined tasks (about half a context each); a thin runner hands each to a fresh worker sub-agent and keeps only its short report; blocked tasks record their question; a scheduled task restarts the runner.
- **Phase 0 as implemented:** `lanes.toml` + `cargo xtask check-lane` (lane from `--lane`, branch name or diff; `cross-lane` label; own workflow `lanes.yml`); lane codes BAT, ADV, SRV, CLT; ids `<CODE>-QUIRK-<n>`, `<CODE>-STUB-<n>`, `<CODE>-Q<n>`; common paths `docs/OPEN-QUESTIONS.md`, `docs/stubs.toml`; global HANDOFF budget 6 000 bytes, lane HANDOFF 10 000 bytes and three sessions; baselines in `benches/<lane>/<workload>.toml`; `verify --scope <lane>` and `verify --summary` (log in `target/verify/verify.log`); ADR drafts in `docs/lanes/<lane>/adr-drafts/`. CI: GitHub Actions runs `cargo xtask verify` on Linux; Windows and `--with-original-data` stay local.
- **Traps:** git on the user's machine from the device shell needs delete permission for the repo folder and `-c user.name/user.email`, and cannot push; cloud sessions with GitHub attached push and open PRs via the REST API (GraphQL blocked); the original installation is read-only; original data never goes into git or CI (ADR-0009).

## claude/lane-server-status.md (2026-10-10, E16 refined)

Owns `h3-protocol`, `h3-server`, `benches/server/`, E16's files, `docs/lanes/server/`. Branches `server/<task>`; ids `SRV-Q<n>`, `SRV-QUIRK-<n>`, `SRV-STUB-<n>`. Autonomy contract confirmed 2026-10-09. Decisions (user, 2026-10-09): SRV-Q1 loopback and LAN until accounts (M3); SRV-Q2 seat tokens until M3; SRV-Q3 headless test client first, then the client lane's viewer; SRV-Q4 spectators via a battle spectator viewer before M2; SRV-Q5 `tokio` for connections (a task per connection; an OS thread per game), JSON in serde's default externally tagged enum form with packed arrays for bulk data, a negotiable `postcard` codec later (measured: +11 crates, JSON 8–10× `postcard`: 4.5 KB vs 0.46 KB for a 20-stack snapshot); SRV-Q6 covered by the adventure lane's `game-orchestration.md` (#32); SRV-Q7 M0 starts now; SRV-Q9 plain files; SRV-Q10 M4 (ranked battles) before M5 (whole game); direction only: SRV-Q8 password-derived keypair, SRV-Q11 web of trust. PR #34 `server/refinement-decisions` (docs only) rebased on master 2026-10-10. Next then: merge #34; M0 (ADR acceptance, `shared/server-skeleton`); M1 T1–T4.

## claude/lane-ai-status.md (2026-10-20, E05 closed pending #292 and #294)

- #292 `shared/e05-closure` (merge first) and #294 `ai/e05-closure`; #286 (T9a audit, T9b review) merged 2026-10-20.
- E05 exit criteria 1–9 met. Smoke run 0.800 vs Random after 40 minutes on the GTX 1070; Greedy 0.828 and Aggressor 0.800 against Random.
- T9c on the user's Ryzen 9 3900X (Windows 10, release, master b2d2e56, `h3 ai bench --encounters default`, 1 048 576 steps per row; thousands of steps/s inside the step / with the bench's random choice): N=64: 53/49, 177/127, 203/121, 199/95 for T = 1, 6, 12, 24; N=256: 50/46, 228/162, 335/194, 375/177; N=1024: 50/47, 233/172, 387/240, 497/264. Python within 2–7 % of Rust inside the step, 16–19 % with its numpy choice.
- Verifies with original data: Windows green (exe-map check skipped: system Python older than 3.11); Linux all 16 steps, 160 of 160 maps.
- Findings: one PowerShell script in the user's repo (`target\t9c\t9c.ps1`, gitignored) replaced pasting results (write logs with `Add-Content -Encoding UTF8`; `Tee-Object` writes UTF-16 in PS 5.1); the setup's baseline counts code verified without original data as verified (use `verify-once.sh engine --force -- --with-original-data`); never `pgrep -f`/`pkill -f`, wait on a PID; edit in a `git worktree` while a verify runs; deleting a design note that code comments cite changes the code fingerprint; CI blocked by billing.
- Next then: E06's refinement when the user calls it; inputs: the AI queue's inbox, AI-Q7.
