# HANDOFF

Living state. The last agent updates this at the end of every session; the next agent reads it first. Newest entry on top; keep old entries short (or prune) once superseded.

---

## 2026-10-04 (sixth session) — E04a T4 done (T4b1 and T4b2: the battle hero and the seam switch)

**State:** branch `e04a-heroes-in-battle`, `cargo xtask verify` green (also `--with-original-data`). Scope chosen by the user at the start: T4b split into **T4b1 and T4b2**, both this session, as separate checkpoints. The user also decided (OPEN-QUESTIONS **Q31**): a scenario is a battle-setup file and gives the battle hero **directly** (no derivation); the battle takes a `BattleHero` as given and checks it **only for soundness**, not whether a real hero could have its values. Nothing half-done.

**T4b1:** `h3-contracts` `hero.rs`: `BattleHero` (`new` = neutral), `DamageBonuses`, `MachineControl` (`BallistaControl { shots, double_damage_chance_bp }`, `TentControl { heal_bonus_bp: u32 }`, reserved `#[non_exhaustive] CatapultControl {}`), `BattleMagic`, `BattleEffect` (content's `CreatureSelector`, `StatOp`), `MAX_BATTLE_EFFECTS` 64; `h3_heroes::derive_battle_hero` (validates first; HER-BAT-001 effective primary skills from skills, worn artifacts per copy with a combination's parts, `needs_skill`, specialty, clamped to the hero parameters; HER-BAT-002 Tactics; mana passed through; the rest neutral until T5–T10/E04b). Tests: `crates/h3-heroes/src/tests/derive.rs` (`tests.rs` became `tests/mod.rs`), real packs in `h3-pack` (Angelic Alliance's parts +21).

**T4b2:** `BattleSetup.heroes` replaces `led_by_hero`; `h3-battle` `hero.rs`: soundness check (BAT-HER-001; `SetupError::InvalidHero { side, error: HeroError }`), per-side `HeroRecord` (type, primary skills, Tactics, mana, max mana) in the state, `BattleState::hero(side) -> Option<HeroView>`, `Observation.heroes` (public part; Tactics private, hygiene property test extended); Q-0003 reads hero presence; **state hash v9**; `h3_pack::HeroEntry` (scenario `[scenario.attacker_hero]`, replay `[setup.attacker_hero]`; refuses to write fields the format does not hold yet: `PackError::HeroNotWritable`); **replay format v3**; `full` scenario's attacker led by Brea; golden replays re-recorded (hashes and headers only; commands identical). Unchanged: observation encoding v3, action encoding v4, C ABI v2, ruleset hash v11. Measured (release, `h3 bench` full, 300 games): `BattleState` 2008 → 2032 bytes, steps/s unchanged within noise (≈ 170–180 k), identical step count.

**Agent decisions (change freely; in the epic and design note):** `funds` waits for T9; the battle's hero record and the file format hold only what the battle reads so far — each later task adds its fields to record, format and hash together; `derive_battle_hero` returns `Result` (validates first) instead of the sketch's bare `BattleHero`; seam field shapes per design note §4.

**Next:** the user decides. Recommended: **T5** (primary skills and hero morale/luck: the hero's attack and defense on its stacks, Leadership/Luck/artifacts as side modifiers, the pool capacity, the death-stare change Q21). Alternative: design batch 1b.

**Traps:** clippy's `type_complexity` fires on `[(fn(&mut T), &str); N]` in tests (use a type alias). TOML serialization needs table fields (`attacker_hero`) declared after plain values in a serde struct. `git stash` leaves untracked files in place (handy for A/B benchmarks of the old code). Sandbox and sync as before (bundle in `Claude outputs/`, deleted afterwards; delete permission for the repo folder).

---

## 2026-10-04 (fifth session) — E04a T4a done (hero details)

**State:** branch `e04a-heroes-in-battle`, `cargo xtask verify` green (also `--with-original-data`). Scope chosen by the user at the start: T4 split at the seam, **T4a only**. Nothing half-done.

**T4a:** new domain logic crate `h3-heroes` (content only so far; `arch.toml`, doc-drift list): `HeroDetails { hero_type, level, experience, primary, skills, worn, backpack, spells, mana }`, `validate` → `DetailsError`, `war_machines`, `PrimarySkill`; rule catalog `docs/rules/catalog/heroes.toml` (HER-DET-001, HER-SEC-001, HER-ART-001/002, HER-SPL-001, HER-WAR-001). Content: `[hero_params]` (Tier 2; optional, at most one pack; `primary_min`, `primary_max`, `backpack`) and `backpack = false` on `[[artifact_slot]]`, harvested from gap-fill `[heroes.params]` and `[artifacts] no_backpack` (sources: wiki Primary skill, Secondary skill, Artifact; VCMI gameConfig/ArtifactUtils); `compile_battle` rejects war-machine positions whose two-hex footprints leave the field or overlap (E03 review item, BAT-WAR-001; the battle's `NoRoom` for machines is now reached only through a setup obstacle); ruleset hash v11; golden replays re-recorded (hash only); `h3-pack` tests hero details against the harvested packs (dev-dependency on `h3-heroes`). Agent decisions (in the epic; change freely): `worn` lists artifacts, not positions; the spell book is the worn artifact, `spells` the known spells (no `spell_book` option; "spells need a book" waits for E04b); experience and mana unchecked; base primary skills ≤ 99, below the minimum allowed; no class limits on skills; 8 skills stays `MAX_HERO_SKILLS`; backpack 64 (wiki, D7).

**Next:** the user decides. Recommended: **T4b** (`BattleHero` and friends in contracts, `derive_battle_hero`, `BattleSetup.heroes` replacing `led_by_hero`, battle validation, state hash v9, observation struct, scenario/replay/CLI/sim follow-ups). Open point to settle at its start: how scenario files name a hero before T11 (e.g. a minimal `[scenario.heroes.<side>]` table → `HeroDetails` → derive). Alternative: design batch 1b.

**Traps:** a dev-dependency must respect the layers too (`check-arch`), so domain crates build test rulesets in code; `quiet_eye_of_the_dragon` is a ring, not a misc artifact. Syncing worked with a bundle in `Claude outputs/` (deleted afterwards) and delete permission for the repo folder. Sandbox as before.

---

## 2026-10-03 (fourth session) — E04a T3b done (artifacts)

**State:** branch `e04a-heroes-in-battle`, `cargo xtask verify` green (also `--with-original-data`). Scope chosen by the user at the start: **T3b only**. The user confirmed quirk Q-0005 as **fix-optional** (ledger, OPEN-QUESTIONS Q23). Nothing half-done.

**T3b:** pack format v3 gains `[[artifact_slot]]` (a kind of equipment position with a count), `[[artifact_slot_override]]` and `[[artifact]]` (class, cost, slot or backpack-only, war machine, `needs_skill`, effects, combination parts; the compiler derives a combination's locks and checks parts, slots and war machines after all packs); nine new `HeroEffect` kinds (`primary_skills`, `morale_at_most`, `luck_at_most`, `alignment_group`, `creature_stat_percent`, `regeneration`, `no_range_penalty`, `shoot_when_adjacent`, `no_retreat`); `HeroEffect<C, K, S, F>` now has a faction parameter and is `Clone` only; ruleset hash v10; `h3_formats::exe::HeroData::artifact_primary` (§7.3); harvester module `h3_harvest::artifacts` and gap-fill `[artifacts]` (membership, 3 war machines, 12 combinations, effects with sources, `pending` lists that must cover every artifact without a modelled effect); packs regenerated: base 14 slot kinds and 127 artifacts, ab 2, sod the fifth misc position and 12 combinations (the 3 unused rows skipped); synthetic packs use every artifact feature and effect kind; golden replays re-recorded (ruleset hash only). Docs: `packs/README.md` (format, what is left out), installation.md §7.3, design note §3 rewritten for T4/T10, glossary, BACKLOG inbox. Agent decisions (in the epic; change freely): `primary_skills` counts per copy, every other kind once per artifact type; `needs_skill` on the artifact; Necromancy artifacts stay pending with the Necromancy skill (E08–E10); combination costs left out.

**Next:** the user decides. Recommended: **T4** (`h3-heroes`, `HeroDetails` with slots/locks/backpack validation, `BattleHero`, `BattleSetup.heroes`; seam change already accepted in ADR-0013) — it is unblocked now. Alternative: design **batch 1b** (T8–T10 disputes, the First Aid Tent with the skill, and the artifact candidates D1 Angelic Alliance mixing, open checks 1–2: stat artifacts on war machines, Golden Bow on the Ballista).

**Traps:** serde ignores unknown keys of a *unit* variant in an internally tagged enum, so `EffectDto::NoRetreat {}` is a struct variant; harvested artifact keys may equal creature or slot keys (`base:ballista` is a creature and an artifact; references are typed by where they appear); coverage counts the implicit `else` of `if let` (the artifact compiler uses iterator combinators where a branch cannot run). Syncing: a fast-forward in the user's repo once failed with "unable to unlink old 'crates/h3-harvest/data/gapfill.toml': Permission denied" (probably held open on Windows) and left HEAD behind with a half-updated working tree; recovered by writing the file in place from `git show <commit>:<path>` and `git reset --mixed <commit>` (no lock files were left). Sandbox as before: symlink `stable` as the 1.97.0 toolchain, `apt-get update` before `apt-get install llvm-20`, `cargo install cargo-deny cargo-llvm-cov --locked`, stage `Heroes3.exe`, both `Data/*.lod` and `extracted/H3bitmap/Objects.txt`.

---

## 2026-10-03 (third session) — E04a T3a done; batch 1a evaluated

**State:** branch `e04a-heroes-in-battle`, `cargo xtask verify` green (also `--with-original-data`). Session closed with the user; nothing half-done.

**T3a:** pack format v3 with `[[spell_school]]`, `[[spell]]` (data rows), `[[secondary_skill]]` (effects per level), `[[hero_class]]`, `[[hero]]` and creature `upgrades` (`packs/README.md`); `h3_content::HeroEffect<C, K, S>` with 13 kinds, compiled in `h3_content::heroes` (validation after all packs, so later overrides are checked too); ruleset hash v9; `h3_formats::exe::hero_data` (hero, specialty, skill-value and spell tables of the known build); harvester module `h3_harvest::heroes` plus the gap-fill section `[heroes]` (elemental upgrades, creature-specialty parameters, Tactics and Artillery effects, Xeron's speed); packs regenerated (base: 4 schools, 80 spells, 28 skills, 16 classes, 128 heroes, 56 upgrade pairs; ab: 2 classes, 28 heroes, Conflux and elemental upgrades; sod: Titan's Lightning Bolt); synthetic packs got an invented hero set that uses every effect kind; golden replays re-recorded (ruleset hash only); `h3-experiment` switched to the new reader (batch-1a map byte-identical). Agent decisions (change freely; in the epic): per-town columns, class aggression, spell AI values and spell flags not harvested (BACKLOG inbox); hero availability waits for E08–E10; a duplicate hero name gets the class appended (`lord_haart_death_knight`). The gap-fill facts were re-checked against the wiki's raw text.

**Batch 1a evaluated** (user's results `08899c3`; played through the HD Mod 5.6 R9 launcher with HD+ on, none of whose fixes touches these battles; from batch 1b on plain HD): Q21 → wiki (new top creature at full health), Q23 → the original uses single-precision rates (all 18 values; quirk Q-0005, class fix-optional *proposed* — the user may object), S-01 confirmed (Q-0004, unlimited Ballista shots), S-03 confirmed for Roland only (Q-0006, data quirk), S-04 not reproduced, D1 (Basic Artillery: one shot), OQ-4 (a commanded Ballista may Wait), the hero's attack counts in the Ballista's damage range and attack advantage (R5). Recorded in OPEN-QUESTIONS (answered), the quirks ledger, the catalog (observation sources on BAT-DMG-002/009, BAT-WAR-002, BAT-TRN-005) and `E04a-design.md` ("Batch 1a results": what T5, T6, T7, T3b/T10 must change). **No rule code changed yet:** BAT-DMG-009 still follows VCMI and BAT-DMG-002 uses decimal rates until T5/T6 implement the results.

**Next:** the user decides between T3b (artifacts: slots, the 144 artifacts, the remaining effect kinds in `E04a-design.md` §3, combination parts, synthetic artifacts) and designing batch 1b (T8–T10 disputes in `docs/rules/surveys/README.md`, plus the First Aid Tent with the skill, which R6 could not show). **Traps:** wiki fetches need the user's approval in the session; ask for verbatim `swh` template arguments, the summariser otherwise reports the HotA branch. The harvester needs `HCTRAITS/HOTRAITS/SSTRAITS/SPTRAITS` from both LODs, which `Inputs::read` takes from the staged `Data/*.lod` files. Sandbox: `apt-get update` first, then `apt-get install llvm-20` works; `rustup component add llvm-tools-preview` fails behind the proxy.

---

## 2026-10-03 (second session) — decisions recorded; T2 batch 1a delivered, waiting for the user

**State:** branch `e04a-heroes-in-battle`, `cargo xtask verify` green. User answers recorded (OPEN-QUESTIONS Q29, Q30): ADR-0013 **accepted** and cut to its decisions (detail in `docs/epics/E04a-design.md`, which each task prunes as it implements it); new ADR size rule (`docs/adr/README.md`, "Read when" column); ADR-0004 and ADR-0012 trimmed (old text in git history at `22b6bc0`); code reading of `Heroes3.exe` allowed as a last resort, for insight only (`docs/04-rules-and-sources.md` §2); quirks are confirmed for SoD before they get a flag (§4; the survey's classification table); results via lean templates; batch 1 split into 1a (T5–T7) and 1b (T8–T10). The smoke map works in the user's game.

**T2 (this session):** new tool crate `h3-experiment` (spec TOML → SoD `.h3m` + results template; names resolve against the installation's tables, hero specialties are checked against the executable; `--list` for spec authors). Batch 1a: spec `experiments/specs/E04a-batch-1a.toml` (11 encounters, played in hot seat; predictions per hypothesis inside the spec), template `experiments/results/E04a-batch-1a.md` (committed, for the user to fill in), map `experiments/out/E04a-batch-1a.h3m` (gitignored; also copied into the user's repo folder). Q23 is covered by three encounters whose combined results separate five arithmetic models (exact, f32, f64, x87-style, x87 with a stored factor). Q22 is not staged (needs controlled geometry and many repetitions; code reading if ever needed).

**Next:** the user plays batch 1a and fills in the template. Meanwhile **T3 (content)** can start; it does not depend on the batch. After the results: evaluate them against the spec's predictions, record observations in the catalog (BAT-DMG-002 for Q23, BAT-DMG-009 for Q21, BAT-WAR-002 for S-01) and confirmed quirks in the ledger, then design batch 1b.

**Traps:** the `.h3m` writer knows tile pictures for dirt, grass and water only (dirt is native to Necropolis only, so it is the neutral experiment terrain). A hero type can appear once per map and a player can have eight heroes. Batch 1a places campaign heroes (Catherine, Roland) and a combination artifact (Elixir of Life) for the first time; if the game rejects them, drop those encounters. Sandbox: rustc 1.97.0 is the image's `stable` (symlink it as `~/.rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu`); `cargo install cargo-deny cargo-llvm-cov --locked`, `apt-get install llvm-20` and `LLVM_COV`/`LLVM_PROFDATA` from `/usr/lib/llvm-20/bin` for coverage; stage `Heroes3.exe`, `Data/H3bitmap.lod`, `Data/H3ab_bmp.lod` and `extracted/H3bitmap/Objects.txt` from the user's `original-data/`.

---

## 2026-10-03 — E04a T1 done (waiting for review); T2 smoke map in progress

**State:** branch `e04a-heroes-in-battle` (created from `master` by the agent with the user's OK). Commits: toolchain pin 1.97.0 (user decision; bench-check re-baselined), AGENTS.md directive (token efficiency beats speed; narrow sub-agents only), T1 docs. No code change in T1.

**T1 results:** `docs/formats/installation.md` §6–§7 (hero/skill/artifact/spell tables; exe tables: heroes with RoE/expansion flags, specialties, per-level skill values, artifact primary skills, spells; §7.5 is a code reading pending Q29); `docs/rules/surveys/` (README with the batch-1 dispute list, skills, artifacts, heroes, battle, quirks); ADR-0013 Proposed; glossary entries in 03-domain; OPEN-QUESTIONS Q29 (may agents read the game's code?) and Q30 (review ADR-0013, classify the 22 quirks).

**Stop rule:** *(lifted: ADR-0013 accepted, see the entry above.)*

**T2 (started):** `h3_formats::h3m` writes the SoD map subset staged experiments need (one terrain, players without towns, heroes with custom army/skills/artifacts/spells/primary skills, wandering monsters; gzip with our own CRC-32, deflate from `miniz_oxide`); layout in `docs/formats/h3m.md`, flow and smoke protocol in `docs/experiments/README.md`. The smoke map `experiments/out/E04a-smoke.h3m` (gitignored; also copied into the user's repo folder) is generated by `cargo run -p h3-formats --example smoke_map -- original-data/extracted/H3bitmap/Objects.txt experiments/out/E04a-smoke.h3m`. It was parsed back with an independent Python parser written from VCMI's reader and compared with the game's SoD maps (`Deluge.h3m`, the user's `experiment.h3m`).

**Next:** the user (1) reviews ADR-0013 and classifies the quirks (Q30), (2) answers Q29, (3) loads the smoke map and reports. Then: T2 spec format + results template + batch 1 (the dispute list in `docs/rules/surveys/README.md`), T3 content (can start once the ADR is accepted, since the content types follow ADR-0013 §3).

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
