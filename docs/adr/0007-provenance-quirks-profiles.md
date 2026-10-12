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
- The seam carries the condition the quirk needs (Q17; since E04a T4b the hero presence in `BattleSetup.heroes`).

## Amendment — E04b T8e: data quirks (2026-10-11, BAT-Q25)

Two confirmed quirks are wrong values in the content data, not behaviour: Roland's specialty
names the Crusader (Q-0006) and the Ice, Magma and Energy Elementals have the wrong level (S-05).
Levels are read in battle (spell specialties, Clone) and later by the adventure, so the fix
belongs in the data every context reads.

1. **Data quirks get flags.** A quirk whose wrong value is content data has a flag in the same
   closed registry, named `quirk.content.<name>`, with a ledger entry and defaults like any
   other. Profiles stay a selection of flags; `faithful` and `strict-fixed` differ exactly by
   their flags. Rule code never reads a `content` flag; only the compiler does.
2. **The pack keeps the original's value; the fix is a gated override.** The entity keeps the
   harvested (faithful) value. An override in a pack whose dependency closure holds the entity
   carries the fixed values and names its flag (`fixes = "quirk.content.<name>"`); the compiler
   applies it in load order, as any override, only when the active profile turns that flag off.
   It must name a registered `content` flag. Ungated overrides are unchanged.
3. **In the core compile.** Gated overrides are applied while `h3-content` compiles the core
   ruleset, before any ruleset part reads the entity: every context sees one profile's values.
   The ruleset hash covers the resulting data (and the profile, as before).
4. **Scope:** gating for `[[creature_override]]` and a new `[[hero_override]]` (for now only
   `specialty`, replacing the whole list); the harvester writes both from its gap-fill file.
   Per-profile Tier-2 *parameters* stay deferred (ADR-0006).

Rejected: overrides inside a profile (`base`'s profiles cannot reach `ab`'s entities, closure
rule; a profile would become a data patch); a second "level for calculations" field (every
reader must choose, and it does not fix Q-0006); rule code reading the flag (a missed reader goes
unnoticed).

## Amendment — E04b T11b: retiring a flag (2026-10-12, BAT-Q29, BAT-Q30)

The E04b review re-read the executable on two flagged quirks and found that neither exists in
the original (`quirk.battle.attack_from_seen_pit`, BAT-QUIRK-2; `quirk.battle.creature_mirror_untested`,
BAT-QUIRK-9). The user retired both. How a flag is retired:

1. The engine does what the original does, in every profile; the flag leaves the code and every
   pack's profiles.
2. Its position in the registry stays reserved as an entry without a name (index = position
   still holds for every other flag). No profile can set it: a profile that names a retired
   flag does not compile (`UnknownQuirk`). `QuirkRegistry::flags()` lists the flags that exist.
3. The ledger keeps the entry under its id with the class "none: considered, not a quirk", the
   reading that retired it and the retired flag's name; ids are not reused.
4. No version changes (adding a flag changes none either); the ruleset hashes of the profiles
   that had the flag on change, so replays recorded in them are recorded again.

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (`design` source kind, stricter traceability).
- 2026-10-01: Amended by E03 T6 (first quirk flag).
- 2026-10-11: Amended by E04b T8e (data quirks: `content` flags and gated overrides; BAT-Q25).
- 2026-10-12: Amended by E04b T11b (retiring a flag; BAT-Q29, BAT-Q30).
