# E08 design note: increment 4

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 4's
amendments of ADR-0013 (`docs/lanes/adventure/adr-drafts/0013-increment-4.md`, T24b1) and
ADR-0017 (T24b2), from the user's answers at ADV-Q15 (the epic's decision 55). The task that
builds a section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-progression.md` (§ numbers are the survey's unless they name this note).
Increment 3's note was pruned by T24b1: every section was built, its records are in `E08-log.md`
(T18a to T22b) and its last versions in §15 below.

Written in two parts (decision 56): **part A**, the hero side (`h3-heroes`), by T24b1; **part B**,
the game side and the cut, by T24b2 (headings only until then).

## Part A — the hero side (`h3-heroes`; T24b1)

Everything in part A is a rule of `h3-heroes`, the hero model beneath the contexts (ADR-0013
point 2): pure functions of a hero's details, the ruleset and explicit inputs (the scenario's
level limit and banned skills, a draw source). Who calls them, when, with which source, and what a
player sees is the game's (part B).

### 1. The data

- **Hero parameters** (Tier 2, `[hero_params]` in the core pack, `h3_content::HeroParams`):
  - `experience`: the thresholds of levels 1 to 12 (0, 1 000, 2 000, 3 200, 4 600, 6 200,
    8 000, 10 000, 12 200, 14 700, 17 500, 20 600; `experience_by_level` 0x679C88, §1.1);
  - `max_level`: 74 (A-7's fix, §2);
  - the skills the level-up forces: **Wisdom** and the **magic schools** (Fire, Air, Water,
    Earth), as keys the compile resolves (no code names a skill, ADR-0016 point 4; a per-skill
    marker would do as well, the building task chooses).
  The compile extends the table to `max_level + 1` (each step the previous + previous / 5, §2)
  and refuses a table that does not rise strictly or overflows 32 bits (a `max_level` above 78
  would). The compiled thresholds are kept in the ruleset (a level is then a search, not a loop).
- **The hero class** (`[[hero_class]]`, `h3_content::HeroClass`) gains `offer_intervals =
  [wisdom, school]`: the levels after which Wisdom and a school are forced (§2.4: 3 and 3 for
  the classes the original counts as magic, 6 and 4 for the others). Beside `primary_gain` and
  `skill_weights`, which the packs already hold. **A-10 as a data quirk** (decision 55: magic in
  `strict-fixed` only): the `ab` pack's Elementalist gets the original's `[6, 4]` and a
  `[[hero_class_override]]` with `fixes = "quirk.content.elementalist_offer_intervals"` gives it
  `[3, 3]` where the flag is off (the pattern of `[[hero_override]]`, `packs/README.md`). A new
  override kind; the Planeswalker is `[6, 4]` either way.
- Sources: `original-data` (code reading: the table 0x679C88, the intervals 0x4DAD74, the forced
  skills 0x4DAD00 and 0x679CBC); the values are not in the text tables, so they are gap-filled
  (the harvester's gap-fill or the packs directly).
- **Not data**: the scenario's level limit and banned skills are the map's (`RawMap::level_limit`,
  `banned_skills`), inputs from the game (§11).

### 2. Experience to levels and its bound (HER-EXP-002; A-7)

- **Thresholds**: levels 1 to 12 from the table; from 13 each step is the previous step + step / 5
  (integer), which equals the original's truncated f64 product × 1.2 at every level up to the
  overflow (§1.1): 13 → 24 320, 14 → 28 784, 15 → 34 140, 16 → 40 567, 20 → 81 961,
  30 → 496 963, 40 → 3 066 455, 50 → 18 975 933, 60 → 117 483 099, 70 → 727 413 425,
  74 → 1 508 362 195, 75 → 1 810 034 207 (the test vectors).
- **The level of an experience**: the number of thresholds at or below it, at most `max_level`
  (`h3_heroes::level_of`).
- **The bound** (A-7, always fixed, decision 55): a hero's experience is at most level 75's
  threshold − 1 = **1 810 034 206**, so level 74 is the highest, as HotA's fix (the wiki). The
  original's wrapped levels past 74 (88, 100, …), its hang at 2³¹ − 1 and its negative experience
  cannot happen: every gain saturates at the bound (§3).

### 3. The scenario's level limit (HER-EXP-003; D-19, D-21, A-8)

- The **cap** of a limit L is level L's threshold when 1 ≤ L ≤ `max_level`; a limit of 0 (none) or
  above `max_level` is **no limit** (A-8 always fixed: the original's wrapped, often negative cap
  of a limit above 75 cannot happen); without a limit the cap is the bound (§2).
- **A gain** of n points makes the experience min(experience + n, cap), saturating: within the
  cap as without a limit; experience already above the cap is **lowered to it** (D-19, the
  original's 0x4E347E; a gain of 0 lowers it too). `HeroChange::GainExperience(u32)` keeps its
  shape; applying it needs the limit, which the game passes (an argument of `apply_change`, or a
  limits value beside the ruleset: the building task chooses).
- **At load** (D-21): a placed hero's custom experience is gained from 0 (§8), so it is capped at
  the limit and the hero gets the limit's levels, as the original's; a custom experience above
  the bound is capped the same way, not refused (the census found none near it, §6.4).
- The original's message that a hero can gain no more (a human owner here) is an event for the
  game (§13). No shipped map has a limit (§6.2): the rule is tested by unit tests and batch E4.

### 4. The draws (HER-LVL-002; D-14)

- A level-up draws through a **draw source** its caller gives (ADR-0003 point 7: randomness through
  an explicit parameter): one operation, `between(low, high)`, inclusive, as the original's
  `random_between` (0x50B230).
- **The original's source** (`faithful`, decision 55): the C runtime's generator (`rand`:
  state × 214 013 + 2 531 011, bits 16 to 30) seeded per level with
  214 013 L + 156 823 S + 154 079 (32-bit, wrapping), L the new level, S the hero's seed byte
  (1–255); `between(a, b)` = a + rand() mod (b − a + 1) (§2.2). A small type in `h3-heroes`
  beside the trait, the twin of `h3_adventure::split::TileRand` (one generator type in `h3-kernel`
  could serve both and the battle's obstacles later: a shared change, not needed now).
- **The owner's stream** (`strict-fixed`): the source implemented over `kernel::Rng`
  (`range_inclusive`). Which source, the seed byte and the stream are the game's (§11, ADR-0017).
- Order of a level's draws: the primary skill, the first offer, the second offer (§5, §6), then
  the choice at load if it draws (§8). A level draws 1 + 0 to 4 numbers.
- Test vectors (§2.3): a Knight with S = 1 reaching level 2: seed 738 928, the primary's draw 3
  (Attack); with S = 200: seed 31 946 705, the draw 91 (Knowledge).

### 5. The primary skill (HER-LVL-001; D-15, A-9)

- r = `between(1, 100)`; the class's `primary_gain` row 1 for a new level of 2 to 9, row 2 from
  the level-up **to** 10 (D-15, the new level); r walks Attack, Defense, Spell Power, Knowledge
  subtracting each chance, and the skill it stops on gains 1.
- **A-9, always fixed**: the gain saturates at the ruleset's `primary_max` (99) on the base the
  engine keeps without artifacts (`HeroDetails.primary`, §4.2); the original's byte, which holds
  the artifacts' bonuses too, wraps at 128. The effective skill (base + artifacts, clamped by its
  readers) is the original's at every value below the wrap.

### 6. The offers (HER-LVL-003, HER-LVL-004; D-16, D-17, D-18, A-10)

`h3_heroes`' offer (the original's `hero_secondary_offer` 0x4DAD00, §2.4): given the hero, a
window of skill levels [lowest, bound), an excluded skill, the new level L, the banned skills and
the source, it returns a skill or none.
- **Candidates**: the skills whose level (0 when unknown) is in the window, not the excluded one;
  with eight skills known (`MAX_HERO_SKILLS`) the lowest becomes 1 (only advances); an empty
  window gives none.
- **1. Wisdom**, returned without a draw, when L ≥ the Wisdom counter + the class's Wisdom
  interval, it is a candidate, not excluded and not banned.
- **2. A school**, when L ≥ the school counter + the class's school interval and the excluded skill
  is no school: a draw among the candidate schools not banned, **a known school weighing 1**
  (D-17), an unknown one its class weight; no weight at all: step 3.
- **3. Any candidate**, by weight: the class's (`skill_weights`, an absent skill 0), a known skill
  at least 1; **a banned skill 0, or 1 if the hero knows it** (D-18). No weight: none.
- **A draw**: `between(1, total)`, then the candidates in the ruleset's skill order (the original's
  ids' order, which the packs keep), each subtracting its weight until the rest is at or below 0.
- **The level's two offers** (0x4DA94D–0x4DA9AE): the first an advance (window [1, 3)), else any
  skill below Expert ([0, 3)); a school as the first sets the school counter to L before the
  second is drawn; the second a new skill ([0, 1)) other than the first, else any skill below
  Expert other than the first ([0, 3)). After both: an offered Wisdom sets the Wisdom counter to
  L, an offered school the school counter. The counters follow the offers, not the answer.
- **The intervals** are the class's `offer_intervals` (§1); the Elementalist's follow A-10's data
  quirk (`faithful` 6 and 4, `strict-fixed` 3 and 3).
- The result of a level, `h3_heroes::LevelUp` (names the building task's): the new level, the
  primary skill gained, the offers (none, one, or two in order: left and right) and the counters
  after it. Drawing it changes nothing; the answer does (§7).
- Test vectors (§2.4): the Knight above, knowing basic Leadership only: S = 1 offers Leadership
  and Tactics, S = 200 Leadership and Sorcery; batch E3c's E3-6 (§3.3): a Death Knight with
  S = 75 and eight basic skills, levels 2 to 5.

### 7. The answer as a hero change; the details (HER-LVL-005)

- **`HeroChange::GainLevel`** (name the building task's): the level + 1, the primary skill + 1
  (§5), the learned skill if any (a known skill one level up, at most Expert; a new one at Basic in
  the next slot, at most eight), the counters set (§6). One change per level, atomic. Checked
  (HER-CHG-001): a level is pending (§ below), the learned skill can be learned, the counters at
  most the new level. Whether the skill is one of the level's offers is the game's check (it holds
  the drawn level-up); `h3-heroes` checks only that the details stay valid.
- Later increments add the plain gains (a skill from a Witch Hut, a primary skill from an
  object: increment 5's sketch) as their own variants when they need them.
- **`HeroDetails`** gains the two counters (the levels at which Wisdom and a school were last
  offered: the original's +0x90 and +0x3F), 0 at the start; the hero's seed byte S is **not** in
  the details (server-only, decision 9: the game keeps it beside them, §11).
- **The level** stays the details' field, the levels given so far (the original's +0x55): at most
  the experience's level; the difference is the hero's **pending level-ups**, which the game turns
  into decisions or gives at load. The validation (HER-DET-001) gains: the experience at most the
  bound, the level at most the experience's level, the counters at most the level.
- Every rule that reads the level (the specialties' growth HER-BAT-004, -005; Learning HER-EXP-001;
  Intelligence HER-MAN-001; the spell specialties) reads the given level, unchanged (§4.2): while
  a level-up is pending its gains are not yet the hero's, as in the original's modal dialog.

### 8. The original's choice between two offers (HER-LVL-006; D-20)

- Decision 55: **the computer's evaluation is ported**: `ai_level_up_choice` (0x52BBD0) with
  `ai_secondary_skill_value` (0x524690, 0x684 bytes) and `ai_takes_new_secondary_skill` (0x524DD0,
  0xF3 bytes): both offers known or both new, the higher of the two values, the first on a tie;
  otherwise 0x524DD0 decides whether the new skill is taken (§2.5, §9).
- **Not read yet**: the first building task reads 0x524690 and 0x524DD0 (and what they call) with
  VCMI's `CGHeroInstance` counterpart and the wiki, as a survey section (§10 of the survey), before
  the rule is built. Known: the flag argument, clear at load and for a hero without an owner,
  replaces the army's summed creature values by the defaults 1 000 and 500 (0x5246D5–0x524722).
- **Its place**: an `h3-heroes` rule (the hero model beneath the contexts), so the game's levels at
  load and E10's computer players call the same function. **Its inputs**: the hero's details, the
  two offers, the army's worth (the setup's defaults, or a computer player's army: an argument,
  since the army is not part of the details) and the level's draw source, if the reading finds
  that 0x524DD0 draws (the survey's open point, §9); only the setup's path is built in
  increment 4.
- **A risk the reading settles**: if 0x524690 reaches the adventure AI's spell values (0x5275B0
  and its helpers, read by the battle lane, E23, `installation.md` §7.39) or other game state, those
  inputs come from the caller or move beneath the contexts too; the reading's task says which and
  the cut's later tasks adapt.
- **The levels at load** seen from the hero (the game's sequence is GAM-SET-002, §12): after the
  custom skills, army, spells and artifacts, the experience set to 0 and the custom experience
  gained (§3), then each pending level drawn with the original's source and S (§4), two offers
  chosen by this rule with the setup's defaults, one learned, none the primary only, each applied
  as `GainLevel` (§7); the spell points after (HER-MAN-001). In `strict-fixed` the owner's stream
  draws (D-14's flag).

### 9. Rule ids and quirks proposed

- Rules (survey §8, ids written when built): HER-EXP-002 (§2), HER-EXP-003 (§3), HER-LVL-001
  (§5), HER-LVL-002 (§4), HER-LVL-003 and -004 (§6), HER-LVL-005 (§7), and **HER-LVL-006** (new,
  §8: the original's choice between two offers). HER-DET-001 and HER-CHG-001 gain the checks of
  §7. The game's GAM-HER-002, GAM-DEC-002, GAM-SET-002, GAM-BAT-008: part B.
- Quirks for `docs/rules/quirks-adventure.md` (confirmed by the second reading, §9), each written
  with its rule: **ADV-QUIRK-9** a level's draws follow the hero's seed byte and the level (D-14;
  fix-optional, its flag the game's: §11); **ADV-QUIRK-10** the Elementalist's might intervals
  (A-10; fix-optional, the data quirk `quirk.content.elementalist_offer_intervals`, §1);
  **ADV-QUIRK-11** the levels past 74 wrap (A-7), **ADV-QUIRK-12** a limit above 75 caps at a
  wrapped threshold (A-8), **ADV-QUIRK-13** a primary skill wraps at 128 (A-9): fixed, no flag.
  The two flags take the registry's next free positions (after 25).

### 10. The hero side's shared and `cross-lane` needs (for the cut, §17)

- `h3-content` (shared core, a `shared/` pull request): the hero parameters' table, highest level
  and forced skills; the class's `offer_intervals`; `[[hero_class_override]]`; the quirk flag in
  the registry; the pack format and core ruleset hash bumps.
- The packs' `core.toml` (`base`, `ab`, the synthetic packs; battle-owned, `cross-lane` or in the
  shared change) and the harvester's gap-fill; the golden replays' headers move with the core
  ruleset hash (both lanes' re-recordings, as at E04c T7e).
- `h3-heroes` (this lane): §2 to §8's rules, `HeroDetails`' counters, `GainLevel`, the checks;
  the game state hash moves with the details (§15).

## Part B — the game side (T24b2)

Headings only until T24b2 writes them (ADR-0017's draft; QUEUE's T24b2 entry).

### 11. The seed byte and the stream (GAM-HER-002; D-14's flag)

### 12. The map's level limit and banned skills; the import; levels at load (GAM-SET-002; D-21)

### 13. A level-up as a pending decision (GAM-DEC-002); the battle's experience into levels (GAM-BAT-008); events and what a player sees

### 14. The env, replays and properties

### 15. Versions

Increment 3's last: core ruleset hash v45 (the battle lane's since), game state hash v8,
adventure state hash v10, adventure ruleset part v8, map import v7, action encoding v3, adventure
replay format v4. Each bumps once in the increment, in the task that first changes it.

### 16. Batch E4

### 17. Shared and `cross-lane` work; the building tasks
