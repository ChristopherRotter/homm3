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

### Q8 — Baseline game version for "original"

Which installation/version is the reference (and will the user supply)? Original game versions, patches and re-releases differ in data and in fixed bugs. Needed before E02. Please provide the install (or its archives) and name the version.

### Q9 — May base content tables be committed to the repo?

Options: (a) import-only — stats/text are extracted from the user's install at setup, never committed; CI uses synthetic packs; (b) commit fact tables (numbers/IDs, no original text or art) as data; other open-source engines ship such data. Recommendation: (a) until a legal read says otherwise; it fits "user owns a legitimate copy". Decision needed before E02 completes.

### Q10 — Multiplayer model confirmation

Design assumes **authoritative server with per-viewer projections** (`06-security.md`); lockstep with full state is excluded because it leaks hidden info. Confirm this matches intent for any future multiplayer.

## Answered

- *(2026-09-30, Q1)* License: **GPL-2.0-or-later** (user's choice, option (a)). VCMI code may be consulted and ported with attribution; dependencies must be GPLv2-compatible. Recorded in ADR-0009 (Accepted), `LICENSE`, `deny.toml`.
- *(2026-09-30, Q3)* CI host: **no CI configuration yet**; `cargo xtask verify` is run locally. Work happens on short-lived feature branches (E01: `e01-walking-skeleton`) that the user fast-forwards into `master`. The Windows half of the determinism check therefore has to be run by hand until CI exists (see HANDOFF).
- *(2026-09-30)* Configurability scope: tiers 1–3; architecture wins when in conflict.
- *(2026-09-30)* Fidelity definition: logic equivalence, not replay equivalence; profiles welcome.
- *(2026-09-30)* Save files: low priority, future scope.
- *(2026-09-30)* Core exposes a C ABI; RL language deferred.
- *(2026-09-30)* Walking-skeleton approach approved.
