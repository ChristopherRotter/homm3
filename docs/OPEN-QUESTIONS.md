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

### Q8 — Baseline game version for "original" (partly answered)

**Answered 2026-09-30:** the user's installation is available to agent sessions as an additional folder (on the user's machine at `D:\Heroes3\HoMM 3 Complete`). **Still open:** which version/edition it is (original patch level vs. a re-release). E02 T1 determines it from the files themselves (executable/version data, archive contents), records it in the format catalogue, and confirms it with the user. The installation folder is strictly read-only for agents (see AGENTS.md).

### Q10 — Multiplayer model confirmation

Design assumes **authoritative server with per-viewer projections** (`06-security.md`); lockstep with full state is excluded because it leaks hidden info. Confirm this matches intent for any future multiplayer.

## Answered

- *(2026-09-30, Q9)* Base content tables: **import-only**. Files may be copied from the installation into the repo's `original-data/` directory, which is gitignored (and guarded by `check-arch`); nothing from the original game is committed. CI and tests keep using synthetic packs. Recorded in ADR-0009 (amendment), `.gitignore`, AGENTS.md.
- *(2026-09-30, Q11)* Attack-bonus cap: **+300 %** (damage at most 4× base, reached at 60 points of attack over defense), as the wiki states; VCMI's 400 % is not followed. The base pack (E02) must set `attack_bonus_cap_percent = 300`; the synthetic pack already does. Recorded in BAT-DMG-002.
- *(2026-09-30, Q12)* Damage roll of large stacks: **follow the wiki** — with 10 or more creatures, roll 10 integers in the damage range, sum them, multiply by N/10 and round down (for exactly 10 creatures this equals summing ten individual rolls, which is what E01 implements). Recorded in BAT-DMG-001; VCMI's averaging is not followed.
- *(2026-09-30, Q1)* License: **GPL-2.0-or-later** (user's choice, option (a)). VCMI code may be consulted and ported with attribution; dependencies must be GPLv2-compatible. Recorded in ADR-0009 (Accepted), `LICENSE`, `deny.toml`.
- *(2026-09-30, Q3)* CI host: **no CI configuration yet**; `cargo xtask verify` is run locally. Work happens on short-lived feature branches (E01: `e01-walking-skeleton`) that the user fast-forwards into `master`. The Windows half of the determinism check therefore has to be run by hand until CI exists (see HANDOFF).
- *(2026-09-30)* Configurability scope: tiers 1–3; architecture wins when in conflict.
- *(2026-09-30)* Fidelity definition: logic equivalence, not replay equivalence; profiles welcome.
- *(2026-09-30)* Save files: low priority, future scope.
- *(2026-09-30)* Core exposes a C ABI; RL language deferred.
- *(2026-09-30)* Walking-skeleton approach approved.
