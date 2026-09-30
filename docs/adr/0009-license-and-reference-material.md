# ADR-0009: License and reference-material policy

- Status: Accepted
- Date: 2026-09-30
- Deciders: user (OPEN-QUESTIONS Q1, answered 2026-09-30), recorded by the E01 session

## Context

The user originally wanted the license to be as open and permissive as possible, and said VCMI may be consulted because it is open source. VCMI's code is GPL-2+ ([Debian copyright file](https://metadata.ftp-master.debian.org/changelogs/contrib/v/vcmi/oldstable_copyright)). Code copied or translated from GPL code makes the result a derivative that must be GPL-licensed, so "permissive" and "port VCMI code" cannot both hold (options in OPEN-QUESTIONS Q1). Separately, the original game's assets and text are proprietary; the user supplies their own legitimate copy.

On 2026-09-30 the user chose option (a): **GPL-2.0-or-later for the whole project**.

## Decision

1. **Project license: GPL-2.0-or-later.** The license text is in `LICENSE` (GPL version 2); every crate declares `license = "GPL-2.0-or-later"` via the workspace manifest.
2. **VCMI may be read, and its code may be ported or adapted**, because VCMI is GPL-2+ and therefore compatible. Conditions:
   - Ported or closely adapted code carries a comment naming the origin: `// Derived from VCMI <path> @ <commit or tag>, GPL-2.0-or-later, (c) VCMI authors`.
   - The behavior still gets a rule-catalog entry with `kind = "vcmi"` provenance (repo URL + commit/tag). Porting code does not replace the rule/source/test discipline (ADR-0007, ADR-0010).
   - Ported code must still satisfy our architecture: purity, determinism and the numeric model (ADR-0003; VCMI uses platform floats, `std` RNGs and unordered containers in places), the command/validation model (ADR-0004) and the bounded contexts (ADR-0005). In practice most VCMI logic is re-expressed rather than transliterated.
   - VCMI is a strong but fallible source: it deviates from the original in places (see the source hierarchy in `docs/04-rules-and-sources.md`). Where it disagrees with other sources, the conflict rule applies.
   - VCMI may also be run as a separate process for differential (oracle) tests.
3. **Dependencies must be compatible with GPL version 2.** `cargo deny` allows only GPLv2-compatible permissive licenses (MIT, BSD, ISC, Zlib, …; see `deny.toml`). Apache-2.0-*only* crates are rejected because Apache-2.0 is compatible with GPLv3 but not GPLv2; dual-licensed `MIT OR Apache-2.0` crates are accepted under MIT. Adding a copyleft or Apache-only dependency needs a new decision. Dev-dependencies (test/benchmark tooling that is never distributed, e.g. `criterion`, which pulls in the Apache-2.0-only `ciborium`) are excluded from the license check (`exclude-dev = true` in `deny.toml`) and reviewed by hand via `arch.toml`.
4. **No original assets or text in the repo.** Game data is imported at setup from the user's install by an adapter. CI and tests use synthetic content packs. Base fact tables are import-only too (Q9, decided 2026-09-30; see the amendment below).
5. Wiki/community knowledge is used as facts with labelled, linked sources; no text is copied from them into the repo beyond short, attributed identifiers/titles.
6. Project naming avoids implying endorsement by the trademark owner (Q2).

## Consequences

- Faster bootstrap of fidelity knowledge: VCMI code can be consulted and adapted directly, with attribution.
- Downstream users (clients, tools, anyone embedding the engine, including through the C ABI) are bound by the GPL when they distribute a combined work. Closed or commercial clients linking the engine are not possible without a license change, and relicensing to a permissive license later would require removing any VCMI-derived code and the agreement of all contributors.
- The dependency policy is slightly stricter than the Rust ecosystem default (no Apache-2.0-only crates).

## Alternatives considered

- Permissive (MIT/Apache-2.0) with VCMI as a behavioral reference and black-box oracle only (the previously recommended option (b)): maximum downstream freedom, but no VCMI code reuse.
- Split: permissive core, GPL adapters/clients (option (c)): more complex boundary management.
- Permissive with copied VCMI code: a license violation.

## Amendment — original data handling (2026-09-30, user decision on Q9)

- Point 4 is decided: **import-only**. Agents may copy files from the user's installation into the repo's `original-data/` directory for inspection and import; that directory is gitignored, and `cargo xtask check-arch` fails if any file under it is tracked. Nothing derived verbatim from the original game (assets, text, data tables) is committed; CI uses synthetic packs.
- The installation itself (on the user's machine, connected to sessions as a folder the user cannot make read-only) is **read-only by rule**: no edits, creations, moves or deletions, and no delete permission requests for it (AGENTS.md).

## Amendment 2 — harvested configuration is committed (2026-09-30, user decision Q13, revises Q9)

- Point 4 and the first amendment are revised: the original text tables (plus, where they lack data, the executable's static tables) are **harvested once** by a tool that lives in this repo; the tool's clean output — TOML packs `base`, `ab`, `sod` under `packs/` — **is committed** and is the configuration the engine uses. Gaps are filled from labelled wiki/VCMI sources.
- **Committed:** gameplay data (numbers, flags, levels, factions, costs, growth, …) and English names used as display names. **Not committed:** flavor text (ability descriptions, hero biographies, event texts), graphics, sound, video, and the original files themselves. The original files stay in the gitignored `original-data/` directory (`check-arch` still fails if anything there is tracked).
- Tests and CI may now use the committed packs; synthetic packs remain for tests that need controlled numbers.
- The installation stays read-only by rule.
- Consequence to watch: the committed packs contain facts and names from a trademarked game. Q2 (project naming) and any public release should keep this in mind; the user owns that decision.

## Revision history

- 2026-09-30: Proposed (recommended permissive option (b), interim rule "do not copy or translate VCMI code").
- 2026-09-30: Accepted with the user's decision for GPL-2.0-or-later (option (a)); interim rule lifted; dependency policy narrowed to GPLv2-compatible licenses.
- 2026-09-30: Amended: Q9 answered (import-only, gitignored `original-data/`, installation read-only by rule).
- 2026-09-30: Amended (E02 T1): Q13 revises Q9 — harvested gameplay data and names are committed as packs; original files and flavor text are not.
