# ADR-0009: License and reference-material policy

- Status: **Proposed** (needs user decision — OPEN-QUESTIONS Q1)
- Date: 2026-09-30
- Deciders: pending

## Context

The user wants the license to be as open and permissive as possible, and said VCMI may be consulted because it is open source. VCMI's code is GPL-2+ ([Debian copyright file](https://metadata.ftp-master.debian.org/changelogs/contrib/v/vcmi/oldstable_copyright)). Code copied or translated from GPL code makes the result a derivative that must be GPL-licensed, so "permissive" and "port VCMI code" cannot both hold. Separately, the original game's assets and text are proprietary; the user supplies their own legitimate copy.

## Proposed decision (recommended option, Q1 (b))

1. **Project license: permissive** (MIT or Apache-2.0, or dual — exact choice pending). No `LICENSE` file is added until decided.
2. **VCMI is a behavioral reference and a black-box test oracle only.** Agents may read VCMI to understand behavior. They must not copy, port or mechanically translate its code. Behavior learned is written down as a rule in the catalog with `kind = "vcmi"` provenance (repo URL + commit/version), then implemented independently. VCMI may be run as a separate process in differential tests.
3. **Dependencies in logic crates** must be permissively licensed and compatible (`cargo deny` enforces); no GPL/LGPL dependencies in the core.
4. **No original assets or text in the repo.** Game data is imported at setup from the user's install by an adapter. CI uses synthetic content packs. Whether base fact tables (numbers/IDs) may be committed is OPEN-QUESTIONS Q9 (default: import-only).
5. Wiki/community knowledge is used as facts with labelled, linked sources; no text is copied from them into the repo beyond short, attributed identifiers/titles.
6. Project naming avoids implying endorsement by the trademark owner (Q2).

## Interim rule (in force until decided)

Do not copy or translate VCMI code. Everything else above applies.

## If the user chooses GPL (Q1 (a) or (c))

Update this ADR; relax rule 2 for the GPL-licensed parts; keep the rest. Note the license of each crate in its `Cargo.toml` and in a top-level `LICENSES.md`.

## Consequences (of the recommended option)

Maximum downstream freedom (closed or commercial clients, other engines, research use). Cost: slower bootstrap of fidelity knowledge than porting code directly; mitigated by VCMI as oracle and by the provenance workflow.

## Alternatives considered

GPL-2+ whole project (fastest reuse, restricts downstream users); permissive with copied VCMI code (license violation); ignoring VCMI entirely (throws away a valuable reference and oracle).
