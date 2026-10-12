# 04 — Rules, sources, provenance, quirks

## 1. The fidelity problem

"Identical to the original" needs a baseline. No single source is complete, so we combine several, rank them, label every fact with where it came from, and track disagreements explicitly.

**Fidelity target:** any difference between the original game and this engine must come down to RNG seed and rollouts, never to differences in game logic. Bit-exact replay of original games is not a goal.

## 2. Source hierarchy (highest authority first)

| # | Kind (`kind` in catalog) | What | Notes |
|---|---|---|---|
| 1 | `original-data` | Data files from the user's own installation (the `*.TXT` tables and similar inside the game archives: creature, spell, artifact, hero, class, skill, building, object tables; map and archive formats) | Facts. Supplied by the user (installation connected read-only; copies go to the gitignored `original-data/` directory; the files are never committed, the harvested packs are — Q13). Observed layouts: `docs/formats/installation.md`; do not assume others from memory. Includes static tables inside `Heroes3.exe` (e.g. creature level/town/flags). **Code reading** of the user's `Heroes3.exe` (user, Q29, Q57) is a regular way to settle a rule, and preferred to a new experiment when a reading can settle it (experiments in the original game are slow, Q57), provided it is reliable: the agent reads with `scripts/exe.py` and the address map (`docs/exe/README.md`), reads every path that decides the rule, names an address only after reading it, says which caller path it followed (the human player's action, not the computer player's own routines or a version, tutorial, hidden-battle or network branch), records the finding in our own words and numbers (with addresses) in `docs/formats/installation.md` (or its lane's notes) and the map, says what it did not read, and cites it with this kind. **Rank:** a table read from the executable is a fact of rank 1; a reading of *behaviour* is an agent's interpretation and ranks with `observation` (row 2): an experiment that contradicts it stands, and the reading goes back for a re-read (user, 2026-10-09). **Consensus before trust** (user, 2026-10-10, BAT-Q21): a code reading is non-trivial and can lose nuances (another caller, the computer player's own path, a version branch), so before a rule rests on one the agent compares it with the wiki page(s) and VCMI for the same rule, cites them as sources too and says where they agree. The reading carries the rule only when at least one of them agrees with it, or when both are silent on the point (the reading then stands alone and its point is an experiment candidate). Where the wiki or VCMI contradicts the reading, the agent re-reads the code for the path the other source describes; if the conflict stands it is a conflict of sources ("Conflict rule" below): it goes to `OPEN-QUESTIONS.md` with a confirmation experiment proposed, and the user decides which behaviour the engine has until the experiment settles it; the reading's side is never implemented alone. A reading that rests on a float boundary or truncation, or depends on values loaded at run time from the `.TXT` tables, gets a confirmation experiment queued too (`docs/exe/README.md`, "Confidence and reliability"). For insight only: no code, listings or decompiled output are copied or committed (ADR-0009, amendment 3). |
| 2 | `observation` | Controlled experiments in the original game, recorded as fixtures (inputs, outcome, game version) | Final arbiter for behavior: equal in rank to a code reading of behaviour and the winner when the two disagree (the reading is re-read). Expensive; used for disputed/high-impact rules, for what a code reading leaves open, and to confirm the readings most likely to be wrong (Q57). |
| 3 | `reverse-engineering` | Community technical documentation of formats and mechanics | Label with URL. H3API's names for the executable (`docs/exe/h3api.toml`) are guesses that point to where to read, not a source for a rule on their own (Q57). |
| 4 | `vcmi` | Behavior of the VCMI open-source engine | Reference and oracle; code may be ported with attribution since the project is GPL-2.0-or-later (ADR-0009). Note version/commit. Treat as a strong but fallible source (VCMI itself deviates from the original in places). |
| 5 | `wiki` | Fan wikis (e.g., the community Heroes III wikis) | Label with page title + URL + retrieval date. Good but not 100% complete or correct. |
| 6 | `community` | Forum posts, videos, hearsay | Needs corroboration before use; never the only source for a `high` confidence rule. |
| — | `design` | An engine design decision, not a fact about the original game (e.g. the PRNG algorithm, hashing, encodings) | Only for engine rules in the `kernel` context or for skeleton simplifications that are explicitly marked as such; `ref` names the ADR/doc. Never the source for original gameplay behavior. |

**Conflict rule:** higher-numbered authority never overrides a lower-numbered one; if two sources of equal rank disagree, a wiki disagrees with VCMI, or a code reading disagrees with the wiki or VCMI ("Consensus before trust" above: the reading's rank does not settle it alone), record an entry in `OPEN-QUESTIONS.md` when gameplay impact is real, and/or a quirks-ledger entry when the disagreement is a known original-game oddity.

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

Every known difference between "what the original does" and "what a clean design would do" becomes a quirk entry in a quirks ledger: `docs/rules/quirks-ledger.md` (the battle lane's, ids `Q-NNNN` and from Phase 0 on `BAT-QUIRK-<n>`) or a lane's own `docs/rules/quirks-<lane>.md` (ids `<CODE>-QUIRK-<n>`, e.g. `ADV-QUIRK-1`; `docs/08-work-process.md`, "Lanes"). `check-rules` reads every `docs/rules/quirks*.md`:

| Class | Meaning | Engine behavior |
|---|---|---|
| `exploit-fix` | Enables cheating against other players, corrupts/soft-locks the game, crashes, or breaks invariants | **Always fixed. Not toggleable.** Never implemented. |
| `preserve` | Harmless/accepted deviation that alters gameplay in a way the community accepts | Implemented; controlled by a quirk flag; on in the `faithful` profile |
| `fix-optional` | Unintended, mild, arguably better fixed | Flag exists; default differs by profile |

**Confirm before flagging** (user, 2026-10-03, Q30; 2026-10-09, Q57): a documented bug is implemented as a quirk (with a flag) only after it is confirmed for SoD — by an experiment, by original data or by a code reading that a second, independent look confirms (a fresh agent's re-read of the deciding paths, or VCMI's agreement; a quirk is exactly where a plausible reading of the "obvious" logic is likeliest to be wrong). Until then the engine implements the clean behaviour and the quirk waits in its survey with the experiment that would confirm it. `exploit-fix` entries need no confirmation (they are never implemented). A confirmed quirk takes the class suggested in its survey unless the user objects when the result is reported.

Decision guide:
1. Does it break security/consistency (see `06-security.md`), crash, soft-lock or corrupt state? → `exploit-fix`.
2. Otherwise, does the community treat it as part of the game's known behavior (strategy guides, tournaments)? → `preserve`.
3. Otherwise → `fix-optional`, default to fixed in `strict-fixed`, preserved in `faithful`.
4. Not sure? Ask (AGENTS.md). Do not guess on gameplay-relevant quirks.

**Without a flag, by decision:** the user may decide that a confirmed oddity is simply the rule in every profile; its ledger entry then says "no flag" and `strict-fixed` keeps it (BAT-QUIRK-10, 11, 15 and 18: BAT-Q12, BAT-Q13, BAT-Q15, BAT-Q18). The profiles differ exactly by the quirks that have a flag. **Retired:** when a later reading or observation shows that a flagged quirk does not exist in the original, its flag is retired and the entry stays as "none: considered, not a quirk" (ADR-0007, amendment E04b T11b: BAT-QUIRK-2, BAT-QUIRK-9).

### Profiles

A **profile** is a named set of quirk flags plus Tier-2 parameters. Initial profiles:

- `faithful` (default): all `preserve` and `fix-optional` quirks on; `exploit-fix` quirks always fixed.
- `strict-fixed`: all quirks fixed.

Profiles are data (ADR-0006). The active profile is part of the `Ruleset` and of the replay header. A **data quirk** (a wrong value in the content data, such as a creature's level) has a `quirk.content.<name>` flag like any other; the pack keeps the original's value and an override gated by the flag (`fixes = "<flag>"`) carries the fix, applied in the core compile when the profile turns the flag off (ADR-0007, amendment E04b T8e).

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
