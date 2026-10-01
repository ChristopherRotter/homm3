# Open questions

Questions waiting for the user. Agents add new ones here (context, options, recommendation, what is blocked). When answered, record the decision (ADR or doc) and move the entry to "Answered".

## Open

### Q2 — Project name and crate prefix

The repo is `homm3`; crates use the provisional prefix `h3-`. "Heroes of Might and Magic" is a trademark of its owner; a public project name should probably avoid implying endorsement. Decide before first public release; renaming later is cheap but not free.

### Q4 — Confirm local compute

Is the CPU a Ryzen 9 3900X (12 cores/24 threads) or the non-X 3900? How much RAM? Used to set initial benchmark and self-play expectations; not blocking.

### Q5 — RL implementation language/framework

Deferred. Options later: Python + PyTorch (ecosystem), Rust-native (burn/candle), or other via C ABI. Decide when E05/E06 are refined. Note the GTX 1070's Pascal architecture may constrain framework/CUDA versions.

### Q6 — Confirm quirk policy defaults

Proposed: `faithful` is the default profile; `exploit-fix` quirks are always fixed and never toggleable; decision guide in `04-rules-and-sources.md`. Confirm or adjust.

### Q7 — Supported platforms for the core

Proposed: native Windows, Linux, macOS and WASM compile targets for the logic crates; CI initially Linux + Windows. Confirm.

### Q10 — Multiplayer model confirmation

Design assumes **authoritative server with per-viewer projections** (`06-security.md`); lockstep with full state is excluded because it leaks hidden info. Confirm this matches intent for any future multiplayer.

### Q21 — Death stare: health of the new top creature (E03 T5)

Wiki (Mighty Gorgon page): death stare kills the topmost creatures and "the new topmost unit will always have full Health". VCMI 1.7.5 applies the kills as `kills × max_hp` damage, so the top creature keeps its damage. **Implemented: VCMI** (source hierarchy: the wiki never overrides VCMI; BAT-DMG-009). Impact: small (at most one creature's health per stare). **Experiment:** in the original, let Mighty Gorgons attack a stack whose top creature is damaged (e.g. 10 Pikemen after 5 damage: top at 5 HP), repeat until a death stare happens, and read the top creature's health in the stack window: 10 HP → wiki, 5 HP → VCMI.

### Q22 — Jousting: path length or start-to-end distance (E03 T5)

Wiki (Cavalier page): the bonus is "calculated based on the distance between starting and ending positions". VCMI passes the length of the path actually walked. They differ when obstacles or stacks force a detour. **Implemented: path length (VCMI)** (BAT-DMG-005). **Experiment:** place a Cavalier so that its shortest path to the target detours around an obstacle (e.g. 5 steps for a straight distance of 4); compare its damage on a fixed target with a 4-step straight charge (+25 % vs +20 % of base).

### Q23 — Damage rounding: one exact rounding or floating point (E03 T5)

BAT-DMG-002 computes damage exactly in integers and rounds down once. VCMI (and probably the original) multiplies in floating point, so results that are exact integers mathematically (e.g. 100 × 1.15 = 114.99999…) may round down one lower there. With several bonuses (jousting, hate, death blow, luck in T6) such cases become more frequent. **Implemented: exact integers** (determinism, ADR-0003). **Experiment:** find a combination whose exact result is an integer but whose double-precision product is just below it, and check whether the original reports the integer or one less. Base damage is random, so use a creature whose damage range is a single value (e.g. 2 Archangels, 50 each = 100 base, with an attack advantage of 3 points: 100 × (1 + 3 × 0.05) = 114.99999999999999 in double precision) or log many attacks and compare with the two predicted value sets.

## Answered

- *(2026-10-01, Q17, E03)* Luck without a hero: in the original, luck never triggers for creatures without a hero (wiki, Luck page, HotA section; Halfling page). **Supported as a quirk**: `BattleSetup` gets a per-side "led by a hero" flag now (E04 fills in the hero), and the quirk flag (on in `faithful`, off in `strict-fixed`) disables luck for hero-less sides. Seam change approved by the user.
- *(2026-10-01, Q18, E03)* Lucky strikes on attacks that hit several stacks (Cerberus, hydras, breath, area shots): **only the primary target** gets the luck bonus, as the wiki describes the original (Cerberus and Luck pages; HotA changed it). VCMI 1.7.5 applies it to every target; not followed.
- *(2026-10-01, Q19, E03)* Behemoth defense reduction: the target's defense against a Behemoth (Ancient Behemoth) is **floor(D × (100 − p) / 100)** with p = 40 (80), following the wiki wording ("ignores 40% of enemy Defense"); VCMI's floor(p × D) + 1 is not followed. Open for an experiment later: in the original, attack a stack with defense 10 with Behemoths and compare damage with defense 6 vs 5.
- *(2026-10-01, Q20, E03)* Regeneration (Wight, Wraith, Troll): the top creature regains its lost hit points **at the start of the stack's own turn** (first activation of the round, not when activated from waiting), following VCMI; the wiki's creature text says "at the beginning of each combat round".
- *(2026-10-01, Q16, E03 start)* E03 scope and sources: **in E03** war machines without heroes (own stats, turn phases, automatic control; hero skills in E04) and **native-terrain bonuses + random obstacle generation**; **deferred to E04** spell-like creature abilities, spell immunities/resistances, spellcasting creatures and the tactics phase. Sources: VCMI 1.7.5 and the Heroes 3 wiki (fetches approved). **No experiments in the original game** for now: uncertain rules become OPEN-QUESTIONS entries with a prepared experiment. The modifier ADR (ADR-0012) is reviewed with the E03 report, not before implementation. Recorded in the E03 epic file.
- *(2026-09-30, Q8, E02 T1)* Installation and supported versions: the installation is the **GOG Complete edition, version 4.0** (English; `Heroes3.exe` 4.0 "The Shadow of Death", built 2000-09-08; evidence in `docs/formats/installation.md`). The engine must support **RoE (base), AB and SoD**: AB and SoD are expansions that can be switched on or off by configuration, with a **dependency graph like a package manager** (SoD requires AB; AB requires base). Content is split into `base`, `ab` and `sod` packs. Recorded in ADR-0006 (amendment E02).
- *(2026-09-30, Q13, E02 T1 — revises Q9)* Configuration source of truth: a **new, clean, extensible configuration** is harvested **once** from the original text tables, with gaps filled from labelled wiki/VCMI sources. (Agent addition, not an explicit user decision: the executable's static creature table is original data too and ranks above wiki/VCMI, so the harvester uses it for level, town and flags; see `docs/formats/installation.md` §5.) The harvesting tool lives in this repo and its **clean output is committed** (the packs under `packs/`). What is committed: gameplay data and names (numbers, flags, levels, factions, costs, English names used as display names). What is not: flavor text (ability descriptions, biographies), graphics, sound, and the original files themselves (still only in the gitignored `original-data/`). Recorded in ADR-0009 (second amendment).
- *(2026-10-01, Q15, E02 T4)* Version balance of the packs: **historical, latest patches**. `base` = The Restoration of Erathia at its latest patch, `ab` = Armageddon's Blade at its latest patch, `sod` = The Shadow of Death (Complete 4.0). Consequence: changes that also shipped in RoE patch 1.3 / AB patch 2.1 (Lizardmen, Serpent/Dragon Flies) are already in `base`; SoD-only changes (Angel/Archangel gem cost, Cerberus damage) are `sod` overrides. Facts and sources in `crates/h3-harvest/data/gapfill.toml`.
- *(2026-09-30, Q14, E02 T1)* Pack format: **keep TOML** and extend it (comments carry sources and rule ids next to values). Recorded in ADR-0006 (amendment E02).
- *(2026-09-30, Q9 — superseded by Q13 the same day)* Base content tables: **import-only**. Files may be copied from the installation into the repo's `original-data/` directory, which is gitignored (and guarded by `check-arch`); nothing from the original game is committed. CI and tests keep using synthetic packs. Recorded in ADR-0009 (amendment), `.gitignore`, AGENTS.md.
- *(2026-09-30, Q11)* Attack-bonus cap: **+300 %** (damage at most 4× base, reached at 60 points of attack over defense), as the wiki states; VCMI's 400 % is not followed. The base pack (E02) must set `attack_bonus_cap_percent = 300`; the synthetic pack already does. Recorded in BAT-DMG-002.
- *(2026-09-30, Q12)* Damage roll of large stacks: **follow the wiki** — with 10 or more creatures, roll 10 integers in the damage range, sum them, multiply by N/10 and round down (for exactly 10 creatures this equals summing ten individual rolls, which is what E01 implements). Recorded in BAT-DMG-001; VCMI's averaging is not followed.
- *(2026-09-30, Q1)* License: **GPL-2.0-or-later** (user's choice, option (a)). VCMI code may be consulted and ported with attribution; dependencies must be GPLv2-compatible. Recorded in ADR-0009 (Accepted), `LICENSE`, `deny.toml`.
- *(2026-09-30, Q3)* CI host: **no CI configuration yet**; `cargo xtask verify` is run locally. Work happens on short-lived feature branches (E01: `e01-walking-skeleton`) that the user fast-forwards into `master`. The Windows half of the determinism check therefore has to be run by hand until CI exists (see HANDOFF).
- *(2026-09-30)* Configurability scope: tiers 1–3; architecture wins when in conflict.
- *(2026-09-30)* Fidelity definition: logic equivalence, not replay equivalence; profiles welcome.
- *(2026-09-30)* Save files: low priority, future scope.
- *(2026-09-30)* Core exposes a C ABI; RL language deferred.
- *(2026-09-30)* Walking-skeleton approach approved.
