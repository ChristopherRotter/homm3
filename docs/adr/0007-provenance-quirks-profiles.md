# ADR-0007: Rule provenance, quirks ledger, profiles

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

No single source fully specifies the original game. Wikis are good but incomplete; the original data files are exact for data but not for behavior. Fidelity must be defensible, auditable and improvable. The original also contains bugs: some game-breaking or exploitable (must never be implemented), some harmless and accepted by the community.

## Decision

1. Every game rule has a stable ID in a machine-checked **rule catalog** with labelled **sources** (`original-data`, `observation`, `reverse-engineering`, `vcmi`, `wiki`, `community`), confidence, URL and retrieval date where applicable. Wiki-derived knowledge is always labelled and linked.
2. A **source hierarchy** resolves conflicts; unresolved gameplay-relevant conflicts go to `OPEN-QUESTIONS.md`.
3. Known deviations are entries in the **quirks ledger**, classified `exploit-fix` (always fixed, never toggleable), `preserve`, or `fix-optional`; each gets a quirk flag where applicable and, for exploits, a regression test.
4. **Profiles** (`faithful` default, `strict-fixed`) select quirk flags and Tier-2 parameters; they are part of the `Ruleset` and of replay headers.
5. Full details: `docs/04-rules-and-sources.md`.

## Consequences

Fidelity work is traceable; agents cannot smuggle in undocumented behavior; the community-accepted quirks stay available, exploits never do. Cost: catalog maintenance (enforced by `check-rules`).

## Alternatives considered

Trusting a single reference implementation (inherits its bugs and license constraints); no provenance (unreviewable); fixing all bugs blindly (breaks accepted gameplay); preserving all bugs (keeps exploits).

## Amendment — E01 walking skeleton (2026-09-30)

- Added the source kind **`design`** for engine rules that are not facts about the original game (PRNG, hashing, encodings in the `kernel` context) and for explicitly marked skeleton simplifications (e.g. BAT-SET-001's invented formation data). It never sources original gameplay behavior. Documented in `docs/04-rules-and-sources.md`.
- `check-rules` also requires a `rule:` marker (not only a `covers:` test) for `implemented`/`verified` rules, rejects `rule:` markers on `specified` or `retired` rules, and requires `covers:` markers to be in test code and `rule:` markers outside it.

## Amendment — E03 battle core (2026-10-01, T6)

- First registered quirk flag: `quirk.battle.no_luck_without_hero` (Q-0003, `fix-optional`,
  faithful on, strict-fixed off), read by BAT-LCK-002. Flags are a closed registry in code
  (`h3_content::REGISTRY`, index = position, never reordered); profiles in packs set them with
  `quirks = { "<flag>" = true | false }`, and the harvester writes the packs' profiles.
- A seam flag carries the condition the quirk needs (`BattleSetup.led_by_hero`, Q17).

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (`design` source kind, stricter traceability).
- 2026-10-01: Amended by E03 T6 (first quirk flag).
