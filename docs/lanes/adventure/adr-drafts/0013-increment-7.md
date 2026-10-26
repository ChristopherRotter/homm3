# ADR-0013 amendment (draft): rewards on the hero, the battle bonuses

- Status: Proposed (E08 T55b, 2026-10-26), from T55a's survey and the answers at ADV-Q28 (the
  epic's decisions 129–136). The user accepts it at ADV-Q29; the round-3 `shared/` task (T59)
  applies it to ADR-0013.
- Amends: ADR-0013 points 1 and 2; adds point 15.
- Detail: `docs/epics/E08-design.md` part A (§1–§5); facts: survey
  `docs/rules/surveys/E08-rewards.md` §6.2–§6.9, §7.4, §7.5, §8.2, §8.3.

## Context

Increment 7 (rewards on the map, E08 decision 121) gives a visiting hero experience, primary and
secondary skills, spells, spell points, artifacts, and morale and luck bonuses that last until the
end of its next battle. Each object decides what it offers and asks; what the hero may take, how
much and in which order is a property of the hero's details, which `h3-heroes` derives once
(point 2, the global trap of `docs/HANDOFF.md`). The original keeps the bonuses in the hero
(+0x11A morale, +0x11B luck, status bits 12 and 21 the Pyramid's and the Warrior's Tomb's
once-flags) and clears them when a battle ends (`hero_battle_bonuses_clear` 0x4DA510).

## Decision

1. *(amended, the details)* The details also hold the hero's **battle bonuses**: a morale bonus
   and a luck bonus (each an `i8`, added to and held saturating: the original's byte wraps past
   127, the class of A-9, fixed) and a small set of **once-marks** (the original's status bits:
   the Warrior's Tomb's, the Pyramid's). They last until the end of the hero's next battle. The
   once-per-hero marks of map objects are not the details' but the adventure's (ADR-0016 point 1).
2. *(amended, the hero changes)* `HeroChange` grows by what rewards need: a **primary skill
   gain** (each value above 0 added, saturating at `primary_max`: A-9's class), a **skill raised
   to a level** (never lowered; a new skill only below the most), a **bonus** (morale and luck
   added, with an optional once-mark: a marked bonus is taken only while its mark is clear, then
   sets it) and **clearing the bonuses** (both to 0, every mark clear). Spell points, spells,
   artifacts and experience use today's changes. Each is checked against the details first, as
   every change is.
15. **Rewards are `h3-heroes`' rules** (E08 increment 7; pure functions of the details, the
    ruleset and explicit inputs; the adventure reads them through these functions and returns the
    changes, the game applies them, point 12's pattern):
    1. **The battle bonuses reach the battle as the hero's own effects**: `derive_battle_hero`
       adds the morale bonus to the hero's own-side morale effects and the luck bonus to its luck
       effects (point 6's side modifiers; the battle's own bounds apply). No seam changes. The
       game clears them for every hero of a battle when it ends (ADR-0017 point 11).
    2. **The reward record's hero half** (Pandora's Box, events; the survey's §7.4), in the
       original's order against the details as each part leaves them: the experience amount
       computed first, with the hero's Learning before the record's skills (HER-EXP-001's factor,
       truncated); the primary skills (positive values only); the skills raised to their levels; the
       spell points added and the total held between 0 and the ruleset's **mana ceiling** (999,
       a hero parameter: a hero above it falls to it); the bonuses added; the artifacts, each given
       while the backpack holds fewer than its most (the first position that takes it, else the
       backpack: HER-CHG-002) and **lost** otherwise; the spells, each learned only with a worn
       spell book, of a level at most the learnable level (HER-SPL-005) and not known, the map's
       bans not tested; the experience **last**. Resources and creatures are the adventure's and
       the game's (ADR-0016 point 4, ADR-0017 point 6).
    3. **What a teacher may teach** is derived here: a Witch Hut's skill (not known, below the
       most skills; no class test); a University's slot (not known, below the most, and the
       class's level-up weight above 0); a Scholar's spell (as point 15.2's), skill (a known one
       a level up below Expert, a new one at Basic below the most; no class test) and its
       fallback; a Shrine's and the Pyramid's spell (a worn spell book, the learnable level, a
       Shrine also not known); the Library's condition (the level plus 2 × the Diplomacy level at
       least 10); the Learning Stone's and a chest's experience (an amount × the Learning factor,
       truncated); the Tree of Knowledge's (the experience to the next level, no Learning).
    4. **Whether a hero can receive an artifact** is the backpack's count below its most (64),
       whatever the free positions: the original tests the count alone (§3.2, §8.2).

## Consequences

- `HeroDetails` grows by the bonuses: the details' hash and the game state hash change; the
  import keeps them at 0.
- `derive_battle_hero` reads two more fields; every battle from the map gets them, a battle
  without the map (E01 to E04c's setups) has none.
- The mana ceiling is a hero parameter of the core content (pack format and core ruleset hash,
  shared).
- No rule outside `h3-heroes` computes a learnable spell, a skill's room or a reward's order.

## Alternatives considered

- **The bonuses on the hero on the map** (`h3-adventure`): the battle's derivation would need a
  second source beside the details; the original keeps them in the hero.
- **The bonuses as a temporary artifact-like effect**: the original adds them up and clears them
  at a battle's end, which an effect list does not express.
- **VCMI's reward order and sums** (D-37 to D-39; decision 129: the code's side, batch E7 confirms
  or reverts each).
