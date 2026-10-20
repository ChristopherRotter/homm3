# ADR-0013 amendment (draft): hero progression in `h3-heroes`

- Status: Accepted by the user (2026-10-20, ADV-Q16; the epic's decision 58), drafted in E08
  T24b1 from the answers at ADV-Q15 (decision 55); goes into ADR-0013 through T26's `shared/`
  pull request.
- Amends: ADR-0013 points 1, 2, 3 and 11; adds point 12.
- Detail: `docs/epics/E08-design.md` part A (§1–§10); facts: survey
  `docs/rules/surveys/E08-progression.md`.

## Context

E08 increment 4 is hero progression (decision 48): experience becomes levels, a level-up gives a
primary skill and offers secondary skills, and a placed hero with a custom experience gets its
levels at load. The original draws a level from the hero's seed byte and the level (§2.2), caps
the experience at the scenario's level limit (§1.3), and lets the computer's evaluation choose at
load (§3.2, §9). ADR-0013 already makes `h3-heroes` the hero model beneath the contexts, changed
through typed hero changes, and foresees that increment 4 adds skills, primary skills and levels
(point 2). Other contexts reach the same rules later: map objects and town buildings give
experience (the levels then follow as after a battle), teach skills (Witch Huts, Scholars,
Universities: the same learning bound) and primary skills (the same saturation); prisons and
tavern heroes get levels at load; E10's computer players choose among the same offers. None of
them writes a hero: an object or a building only *describes* its effect, a context reads the
details through `h3-heroes`' derivations and returns typed hero changes in its outcome, and the
game (the orchestrator) applies them and gives the levels that follow (point 1, ADR-0017
point 10); the contexts never call each other.

## Decision

1. *(amended)* `HeroDetails` gains the level-up's two counters (the levels at which Wisdom and a
   magic school were last offered). Its level is the levels given, at most its experience's
   level; the difference is the hero's pending level-ups. The hero's level seed is not part of the
   details: it is server-only state of the game (ADR-0017).
2. *(amended)* The hero changes gain one level-up's change (the level, its primary skill, the
   learned skill if any, the counters), checked like the others. The movement and scouting
   derivations come with a later increment (E08 decision 48), no longer increment 4.
3. *(amended)* Progression is data: the hero parameters hold the experience of levels 1 to 12, the
   highest level and the skills a level-up forces (Wisdom, the magic schools); a hero class holds
   its primary chances, its skill weights and its two forcing intervals (Tier 2, ADR-0006).
11. *(amended)* Progression's rules are `HER-EXP-` and `HER-LVL-` rules in
    `docs/rules/catalog/heroes.toml`.
12. **Progression is `h3-heroes`' rules** (pure functions of the details, the ruleset and explicit
    inputs):
    1. **Experience to levels.** Levels 1 to 12 from the table, each further step the previous
       × 1.2 truncated (the original's); a hero's experience is bounded at the highest level's
       range, so **level 74 is the highest** (A-7, always fixed, as HotA's fix).
    2. **The scenario's level limit** is an input: a gain caps the experience at the limit's
       threshold and lowers experience above it (D-19); a limit of 0 or above the highest level
       is none (A-8, always fixed).
    3. **A level-up** is drawn one level at a time through a **draw source its caller gives**:
       the primary skill from the class's chances (the second row from the level-up to level 10,
       D-15), +1 saturating at `primary_max` (A-9, always fixed); two offers by the original's
       procedure (an advance, then a new skill; Wisdom and a school forced after the class's
       intervals; the class's weights, a known school weighing 1 when one is due, a banned skill
       0, or 1 if known: D-17, D-18), the map's banned skills an input. The Elementalist's
       intervals are the original's 6 and 4, fixed to the magic classes' 3 and 3 by a data quirk
       off in `strict-fixed` (A-10).
    4. **The original's source** is provided here: its C runtime generator reseeded per level
       from the hero's seed byte and the new level. Which source a game uses is ADR-0017's
       (D-14: the original's in `faithful`, the owner's stream in `strict-fixed`).
    5. **The answer** is the level-up's hero change; who answers is the game's (ADR-0017).
    6. **The original's choice between two offers** (the computer's evaluation, 0x52BBD0 with
       0x524690 and 0x524DD0) is ported as a rule here, read first: it chooses at load (D-20)
       and will choose for E10's computer players; what it needs beyond the details (the army's
       worth, the setup's defaults) is an argument.

## Consequences

- Faithful level-ups are a function of the seed byte, the level and the hero's skills, as the
  original's (a reload draws the same); `strict-fixed` draws them from the owner's stream.
- Later increments' experience, skill and primary-skill gains (objects, town buildings), the
  levels at load of prisons and tavern heroes, and E10's computer players reuse the same
  functions; no context keeps its own copy.
- The data adds hero parameters, a class field and a class override kind (shared core content,
  the pack format and the core ruleset hash); the details' counters move the game state hash.
- The evaluation's port needs a code reading first (about 1.9 KB); if it reaches the adventure
  AI's spell values or other game state, those become inputs or move beneath the contexts too.
- Quirks ADV-QUIRK-9 (D-14) and -10 (A-10) are fix-optional, -11 to -13 (A-7 to A-9) fixed.

## Alternatives considered

- **The first offer at load** (VCMI, the wiki): contradicts the code reading twice (D-20; batch
  E3c's E3-6 fits the reading); kept only as batch E4's row E4-1 to settle in SoD.
- **VCMI's table to level 197** (A-7): diverges from SoD's levels; the bound at 74 keeps every
  level the original gets right.
- **A hero's own stream per hero** (VCMI, D-14): not the original's draws; `strict-fixed`'s owner
  stream already hides them.
- **The intervals by a might/magic affinity of the class** (VCMI): no other rule of the
  increment reads an affinity; per-class intervals carry A-10 as a plain data quirk.
- **Progression in `h3-game`**: towns, objects and computer players would each need the game.
