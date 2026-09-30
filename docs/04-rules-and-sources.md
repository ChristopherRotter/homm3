# 04 — Rules, sources, provenance, quirks

## 1. The fidelity problem

"Identical to the original" needs a baseline. No single source is complete, so we combine several, rank them, label every fact with where it came from, and track disagreements explicitly.

**Fidelity target:** any difference between the original game and this engine must come down to RNG seed and rollouts, never to differences in game logic. Bit-exact replay of original games is not a goal.

## 2. Source hierarchy (highest authority first)

| # | Kind (`kind` in catalog) | What | Notes |
|---|---|---|---|
| 1 | `original-data` | Data files from the user's own installation (the `*.TXT` tables and similar inside the game archives: creature, spell, artifact, hero, class, skill, building, object tables; map and archive formats) | Facts. Supplied by the user (installation connected read-only; copies go to the gitignored `original-data/` directory; the files are never committed, the harvested packs are — Q13). Observed layouts: `docs/formats/installation.md`; do not assume others from memory. Includes static tables inside `Heroes3.exe` (e.g. creature level/town/flags). |
| 2 | `observation` | Controlled experiments in the original game, recorded as fixtures (inputs, outcome, game version) | Final arbiter for behavior. Expensive; used for disputed/high-impact rules. |
| 3 | `reverse-engineering` | Community technical documentation of formats and mechanics | Label with URL. |
| 4 | `vcmi` | Behavior of the VCMI open-source engine | Reference and oracle; code may be ported with attribution since the project is GPL-2.0-or-later (ADR-0009). Note version/commit. Treat as a strong but fallible source (VCMI itself deviates from the original in places). |
| 5 | `wiki` | Fan wikis (e.g., the community Heroes III wikis) | Label with page title + URL + retrieval date. Good but not 100% complete or correct. |
| 6 | `community` | Forum posts, videos, hearsay | Needs corroboration before use; never the only source for a `high` confidence rule. |
| — | `design` | An engine design decision, not a fact about the original game (e.g. the PRNG algorithm, hashing, encodings) | Only for engine rules in the `kernel` context or for skeleton simplifications that are explicitly marked as such; `ref` names the ADR/doc. Never the source for original gameplay behavior. |

**Conflict rule:** higher-numbered authority never overrides a lower-numbered one; if two sources of equal rank disagree, or a wiki disagrees with VCMI, record an entry in `OPEN-QUESTIONS.md` when gameplay impact is real, and/or a quirks-ledger entry when the disagreement is a known original-game oddity.

## 3. Rule catalog (machine-checked)

Every piece of game logic is a **rule** with a stable ID. The catalog lives in `docs/rules/catalog/*.toml`, one file per context. Format:

```toml
[[rule]]
id       = "BAT-DMG-001"            # <CONTEXT>-<AREA>-<NNN>
title    = "Base damage of a stack"
context  = "battle"
status   = "specified"              # specified | implemented | verified
summary  = "One or two sentences of what the rule says. Precise rounding/truncation behavior stated explicitly."
quirks   = []                       # ids from the quirks ledger that affect this rule, e.g. ["Q-0007"]
numeric  = "integer"                # integer | float (float: state exact rounding/truncation in summary; see ADR-0003)

  [[rule.source]]
  kind       = "wiki"               # see the table above
  ref        = "Page or file/section"
  url        = "https://…"          # required for wiki / reverse-engineering / community / vcmi
  retrieved  = "2026-09-30"
  confidence = "high"               # high | medium | low
  note       = "optional"
```

Status meaning: **specified** = written down with sources; **implemented** = code exists and a unit test references the rule; **verified** = matches an `observation`, `original-data`-derived conformance test, or an agreed acceptance scenario.

**Traceability (enforced by `cargo xtask check-rules`):**

- Code implementing a rule carries a `// rule: <ID>` marker; tests that cover it carry `// covers: <ID>` (comma-separated lists are allowed). `rule:` markers must be in non-test code; `covers:` markers must be in test code (`#[cfg(test)]` items, `tests/`, `tests.rs`).
- Every rule with status ≥ `implemented` must have ≥ 1 `rule:` marker and ≥ 1 `covers:` test; a rule with a `rule:` marker must not still be `specified`; `retired` rules must not be referenced.
- Every marker must reference an existing rule ID; IDs are unique and their prefix matches `context`.
- Every rule must have ≥ 1 source; sources of kind `wiki`, `reverse-engineering`, `community`, `vcmi` must have an http(s) URL and a `retrieved = "YYYY-MM-DD"` date; `quirks` entries must exist in the quirks ledger.
- Wiki-derived knowledge is never "anonymous": in code the rule ID is the pointer, in the catalog is the labelled, linked source.

## 4. Quirks and profiles

### Classification

Every known difference between "what the original does" and "what a clean design would do" becomes a quirk entry in `docs/rules/quirks-ledger.md`:

| Class | Meaning | Engine behavior |
|---|---|---|
| `exploit-fix` | Enables cheating against other players, corrupts/soft-locks the game, crashes, or breaks invariants | **Always fixed. Not toggleable.** Never implemented. |
| `preserve` | Harmless/accepted deviation that alters gameplay in a way the community accepts | Implemented; controlled by a quirk flag; on in the `faithful` profile |
| `fix-optional` | Unintended, mild, arguably better fixed | Flag exists; default differs by profile |

Decision guide:
1. Does it break security/consistency (see `06-security.md`), crash, soft-lock or corrupt state? → `exploit-fix`.
2. Otherwise, does the community treat it as part of the game's known behavior (strategy guides, tournaments)? → `preserve`.
3. Otherwise → `fix-optional`, default to fixed in `strict-fixed`, preserved in `faithful`.
4. Not sure? Ask (AGENTS.md). Do not guess on gameplay-relevant quirks.

### Profiles

A **profile** is a named set of quirk flags plus Tier-2 parameters. Initial profiles:

- `faithful` (default): all `preserve` and `fix-optional` quirks on; `exploit-fix` quirks always fixed.
- `strict-fixed`: all quirks fixed.

Profiles are data (ADR-0006). The active profile is part of the `Ruleset` and of the replay header.

### Ledger entry template

See `docs/rules/quirks-ledger.md`.

## 5. Conformance testing against references

- **Worked examples** from sources (e.g., damage examples on a wiki) become unit tests linked to rule IDs.
- **Oracle testing:** generate scenarios, run them through this engine and through a reference implementation (VCMI run as a separate process, black box), compare outcomes under a controlled RNG or by comparing distributions. Differences are triaged into: our bug, reference bug, known original quirk, RNG difference.
- **Original-data conformance:** tests that need the user's game files are a separate category (`--with-original-data`), never required for CI coverage, which uses synthetic content packs.
- **Statistical tests** for random mechanics (morale/luck proc rates, damage ranges): assert distributions over many seeds, not single values.

## 6. Open points

- The installation is the GOG Complete edition 4.0 (Q8, answered; `docs/formats/installation.md`). RoE, AB and SoD are all supported as packs `base`, `ab`, `sod`.
- Content tables are harvested once and the clean packs are committed (Q13, revising Q9).
