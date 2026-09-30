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

### Q11 — Attack-bonus cap: +300 % (wiki) or +400 % (VCMI)? (blocks the base pack's parameter value, E02/E03)

**Context.** BAT-DMG-002: the Heroes 3 wiki ([Damage](https://heroes.thelazy.net/index.php/Damage)), the fandom wiki and guides say the attack bonus is capped at +300 % (reached at 60 points of attack over defense). VCMI 1.7.5 uses `attackPointDamageFactorCap = 4.0`, i.e. +400 % at 80 points. By the source hierarchy in `04-rules-and-sources.md`, VCMI (rank 4) outranks a wiki (rank 5), but VCMI is known to deviate from the original in places. Impact: only battles with ≥ 60 points of attack advantage (late game, strong heroes vs. weak neutrals).

**Options.** (a) +300 % (wiki, several independent descriptions); (b) +400 % (VCMI); (c) settle it with an observation fixture in the original game (rank 2, final arbiter).

**Recommendation.** (c), defaulting to (a) until then. The cap is already a ruleset parameter (`attack_bonus_cap_percent`), so the decision is a data value, not code. The synthetic pack uses 300.

### Q12 — Damage roll for stacks of more than 10: wiki algorithm or VCMI's? (does not block E01)

**Context.** BAT-DMG-001: the wiki says 10 rolls of the per-creature range are summed and scaled by N/10, rounded down. VCMI instead takes min(10, N) uniform draws from the already-modified whole-stack range and averages them. Same mean and range, different distribution (and different RNG consumption). Per the source hierarchy VCMI outranks the wiki, but the wiki describes the original's algorithm explicitly and VCMI's looks like an approximation.

**What was done.** E01 implements the wiki algorithm (catalog entry notes the conflict). **Recommendation:** keep the wiki algorithm and confirm with a statistical observation fixture in E03 (distribution of damage of a large stack against a fixed target over many hits).

## Answered

- *(2026-09-30, Q1)* License: **GPL-2.0-or-later** (user's choice, option (a)). VCMI code may be consulted and ported with attribution; dependencies must be GPLv2-compatible. Recorded in ADR-0009 (Accepted), `LICENSE`, `deny.toml`.
- *(2026-09-30, Q3)* CI host: **no CI configuration yet**; `cargo xtask verify` is run locally. Work happens on short-lived feature branches (E01: `e01-walking-skeleton`) that the user fast-forwards into `master`. The Windows half of the determinism check therefore has to be run by hand until CI exists (see HANDOFF).
- *(2026-09-30)* Configurability scope: tiers 1–3; architecture wins when in conflict.
- *(2026-09-30)* Fidelity definition: logic equivalence, not replay equivalence; profiles welcome.
- *(2026-09-30)* Save files: low priority, future scope.
- *(2026-09-30)* Core exposes a C ABI; RL language deferred.
- *(2026-09-30)* Walking-skeleton approach approved.
