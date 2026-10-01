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

## Answered

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
