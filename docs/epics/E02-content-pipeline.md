# E02 — Content pipeline

Status: in-progress (T1–T5 implemented; closing) · Refined: 2026-09-30 (end of E01) · Re-planned: 2026-09-30 (E02 T1, after user decisions Q8, Q13, Q14)

## Goal

Harvest the user's installation **once** into a clean, extensible configuration — committed TOML
packs `base` (RoE), `ab` and `sod` — that the `Ruleset` compiler accepts, with AB and SoD as
expansions that can be switched off (SoD requires AB, AB requires base). The harvesting tool lives
in the repo; its output is committed and from then on maintained as source.

## Why now

E03 (faithful battle rules) needs real creature data (levels, factions, abilities, shots, growth,
costs) and a pack format that can express it, including the RoE/AB/SoD differences.

## Inputs from the user (all answered)

- **Installation:** GOG Complete 4.0, English (`docs/formats/installation.md`). **Read-only by
  rule** (AGENTS.md); copies live in the gitignored `original-data/`.
- **Q8:** support RoE, AB and SoD; AB and SoD are toggleable expansions in a dependency graph.
- **Q13 (revises Q9):** harvest once, commit the clean output (gameplay data + names; no flavor
  text, no assets, no original files). Gaps filled from labelled wiki/VCMI sources.
- **Q14:** keep TOML.
- **Q11:** `attack_bonus_cap_percent = 300`.

## What the data can and cannot tell (from T1)

- `H3bitmap.lod` = SoD tables, `H3ab_bmp.lod` = AB-era tables (same 71 names, 23 differ). Their
  diff is the **AB → SoD change set** → the `sod` pack's overrides.
- Level, town and ability flags are only in `Heroes3.exe` (static 150-record table). The harvester
  reads it for this exact build (SHA-256 pinned) — an original-data source ranked above wiki/VCMI
  (agent decision, recorded in Q13's note).
- **Not in the installation:** RoE-era values and which entities AB added. These come from a
  small, committed **gap-fill file** with a labelled source (wiki/VCMI URL + date) on every entry.

## Scope

- `h3-formats` (adapter crate, used by the harvester only): LOD archive reader (stored + zlib
  entries), text-table reader (TAB fields, quoted multi-line fields, CRLF, Windows-1252), the exe's
  static creature table. Strict bounds, no `unsafe`, property tests on arbitrary bytes (no panics,
  bounded allocation).
- `h3-harvest` (tool crate, binary): `h3-harvest --install <dir> --out packs/` reads the LODs and
  the exe, applies the gap-fill file, writes `packs/{base,ab,sod}/pack.toml` deterministically
  (stable order, stable formatting) and reports unknown/unused fields.
- Pack format v2 (`packs/README.md`): factions and resources as content; creature fields level,
  faction, cost (all resources), fight/AI value, growth, horde growth, hit points, speed,
  attack, defense, damage, shots, casts, adventure-map count, flags, abilities (names of future
  Tier-3 primitives), display names; `depends`; **overrides** into the dependency closure;
  profile-level Tier-2 overrides.
- **Enabled-pack configuration:** which packs are active is configuration; the compiler validates
  the dependency graph (missing dependency, unknown pack, cycle → error) and loads in topological
  order with the pack id as tie-break.
- Synthetic packs extended to cover every new field and override rule.
- Conformance behind `cargo xtask verify --with-original-data`: re-harvest from `original-data/`
  and require the committed packs to be identical; sample checks against the source tables.

## Non-goals

Maps (E07), adventure objects, graphics/sound/text assets, battle rule changes (E03), other tables
than creatures/factions/resources unless E03 needs them (heroes, spells, artifacts come with E04).

## Exit criteria

1. `packs/base`, `packs/ab`, `packs/sod` are committed, compile alone (`base`), as `base+ab` and as
   `base+ab+sod`; `sod` without `ab` is rejected.
2. Re-harvesting the user's installation reproduces the committed packs byte for byte
   (`--with-original-data`).
3. Pack format v2 documented (`packs/README.md`) and validated by the compiler, with tests for every
   new rule.
4. Parsers bounded on hostile input (property tests); `cargo xtask verify` green; the installation
   untouched; no original files or flavor text committed.

## Tasks (one session each; reorder freely)

- **T1 Catalogue.** ✅ 2026-09-30: `docs/formats/installation.md`, version identified (Q8),
  decisions Q8/Q13/Q14 recorded (ADR-0006, ADR-0009 amendments).
- **T2 `h3-formats`** ✅ 2026-10-01: LOD reader, text-table reader, exe creature table; property
  tests on arbitrary/mutated input. Checked against the real files: every entry of both bitmap
  LODs (4 908 + 2 491) inflates to its declared size, all 71 tables of each parse, `CRTRAITS` is
  185 records × 25 fields, the exe table yields 150 records.
- **T3 Pack format v2 + compiler** ✅ 2026-10-01: enabled packs in any order, dependency graph
  (missing dependency, cycles), topological load order with pack-id tie-break, dependency
  closures for references and overrides, resources/factions, all creature fields, closed creature
  flag set, partial overrides; ruleset hash encoding v2; `packs/synthetic` extended and
  `packs/synthetic-ext` added; format in `packs/README.md`. Golden replays re-recorded (only the
  ruleset hash in their headers changed; all state hashes identical). Profile-level Tier-2
  overrides are still deferred (backlog).
- **T4 `h3-harvest` + gap-fill file** ✅ 2026-10-01: `crates/h3-harvest` (tool) +
  `data/gapfill.toml` (AB membership, RoE levels of the four elementals, classification of the
  AB-era→SoD table differences per Q15, battle parameters with the loose formation from VCMI; every
  entry sourced). Committed `packs/base` (118 creatures, 8 factions, 7 resources, battle, profiles),
  `packs/ab` (Conflux + 23 creatures, 4 elemental overrides), `packs/sod` (3 overrides). The harvester
  compiles its output in every allowed combination before writing. Not harvested (reported): war
  machines (speed 0, own rules → E03/E04), exe bit `0x800`, ability attributes and ability text.
  Cross-check finding: the table's attribute column is wrong for Hell Hound, Cerberus, Devil,
  Arch Devil, Crystal Dragon (not flying) and Psychic/Magic Elemental (not two-hex); the exe wins.
- **T5 Conformance + closing**: ✅ `cargo xtask verify --with-original-data [path]` re-harvests and
  requires byte-identical packs; `h3-pack` tests compile the committed packs in every allowed
  combination (CI-safe). Open for closing: refine E03 with the real data (abilities list from the
  harvest report, flags), user confirmation on Windows.

## Risks

- RoE-era values and AB membership are not observable in this installation: every gap-fill entry
  needs a labelled source; conflicts go to OPEN-QUESTIONS (conflict rule, `04-rules-and-sources`).
- The exe flag meanings are inferred (cross-checked with the attribute column); the harvester
  checks every flag against the attribute column where the column has it and reports mismatches.
- The installation folder cannot be made read-only by the user: only ever read from it.
