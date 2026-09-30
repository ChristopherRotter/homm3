# E02 — Content pipeline

Status: todo (blocked on the user for real data: OPEN-QUESTIONS Q8, Q9) · Refined: 2026-09-30 (end of E01)

## Goal

Turn the user's own game installation into content packs (`base`, `ab`, `sod`) that the existing `Ruleset` compiler accepts, with a documented pack format rich enough for E03, and conformance tests that check imported data against the original tables. CI keeps running on synthetic packs only.

## Why now

E03 (faithful battle rules) needs real creature data (levels, abilities, shots, growth, costs) and a pack format that can express it. E01 built the compiler and a minimal format; E02 grows both from real data instead of from memory.

## Inputs needed from the user

- **Q8:** the reference installation/version (and its files) — the agent must catalogue what is actually there, not assume file names or layouts from memory.
- **Q9:** whether numeric fact tables may be committed (default: import-only, nothing original in the repo).
- **Q11:** attack-bonus cap value for the base pack (or accept "wiki value until an observation fixture exists").

## Scope

- `h3-formats` (adapter, new crate — record in ADR-0005 amendment/`arch.toml`): readers for the archive container(s) and text tables found in the installation; strict bounds, no `unsafe`, fuzzed (`cargo-fuzz`, Linux).
- Catalogue: a doc (`docs/formats/…`) listing every archive and table found, with its structure as observed, and which pack fields it feeds.
- `h3-import` (adapter, new crate): extracts base/AB/SoD data into content packs on the user's machine (`h3 import --install <path> --out <dir>`); import is deterministic and reports unknown/unused fields.
- Pack format v2: fields E03 needs (level, faction/town, flags such as flying/shooting/two-hex/undead, shots, growth, cost, AI value if present), abilities as references to Tier-3 primitives (names only until E03 defines them), load order and **override rules** for extension packs (deferred in E01), profile-level Tier-2 overrides (deferred in E01).
- Synthetic packs extended to cover every new field (tests never need original data).
- Original-data conformance tests behind `cargo xtask verify --with-original-data` (reads a user-provided path; never part of CI or coverage).

## Non-goals

Maps (E07), adventure objects, graphics/sound/text assets, battle rule changes (E03).

## Exit criteria

1. Import of the user's installation produces packs that compile; a conformance test compares a sample of imported values against the source tables.
2. Pack format v2 documented (`packs/README.md`) and validated by the compiler with tests for every new rule.
3. Parsers fuzzed (smoke run in verify with `--fuzz-smoke`), bounded on hostile input.
4. `cargo xtask verify` green; synthetic packs only in CI; no original data committed unless Q9 says otherwise.

## Tasks (one session each; reorder freely)

- **T1 Catalogue.** With the user's files: list archives/tables, document observed layouts. No code beyond throwaway inspection.
- **T2 `h3-formats` archive reader** + fuzz target.
- **T3 Text-table reader(s)** + fuzz target(s).
- **T4 Pack format v2 + compiler validation** (fields, overrides, profile overrides) with synthetic packs.
- **T5 `h3-import`** base pack; then AB/SoD packs as extensions.
- **T6 Conformance tests** (`--with-original-data`), HANDOFF, refine E03 if the data changes assumptions.

## Risks

- Original tables may encode things differently from what wikis suggest: the catalogue (T1) is authoritative, record surprises as rules/quirks.
- License/IP: keep imported data on the user's machine by default (Q9).
