# ADR-0013 amendment (draft): increment 12, garrisons and the bonus visitables (the hero side)

- Status: **Accepted** at T108b's checkpoint (ADV-Q50, 2026-11-11; E08 decisions 306 and 307: the
  user moved every visit's effect to the hero on the map and asked for a short dynamic list of
  once-marks), to be applied by T112. Drafted by E08 T108b (2026-11-11) from the survey
  `docs/rules/surveys/E08-garrisons-bonuses.md` §10 (§10.3, §10.4, §10.9) and the user's answers at
  ADV-Q49 (E08 decisions 300, 301), then into ADR-0013 through the round's `shared/` pull request
  (T112) with batch E12's outcome.
- Amends: ADR-0013 points 1, 2, 13, 15 (15.1, 15.2); its consequences, alternatives and revision
  history.
- Detail: `docs/epics/E08-design.md` part C (§12); the map side is `0016-increment-12.md` (the
  hero on the map's bonuses and timed marks, its point 3), the game's `0017-increment-12.md`. **No
  seam changes:** `h3-contracts`' `EffectSource::Visit { mark: u8 }` stays (decision 306), its doc
  comment's pointer to `h3_heroes::BattleBonuses::marks` corrected in T112 (the adventure's hero on
  the map holds the marks).

## Context

The original keeps what a hero visited in its status bits (+0x105): those until the end of its
next battle (eleven of increment 12's objects, the Sirens, the Warrior's Tomb's and the Pyramid's),
one for the day (the Magic Well) and one for the week (the Stables). Increment 7 put the battle
bonuses in the hero's details (a summed morale and luck and eight once-marks). Increment 12 needs
twelve more marks and each mark's own amount (the Fountain of Fortune's luck is the week's draw, the
Idol's and the Temple's amounts the day's; the battle names each visit's line by its source). The
user decided (ADV-Q50) that no visit's effect carries over to a hero that leaves the map and is
hired again, so all of them live on the hero on the map, and that the once-marks are a short
dynamic list rather than a fixed set (content beyond SoD may add visitables).

## Decision

**Point 1** (entities), replacing "From E08 increment 7 the details hold the hero's battle bonuses
… (ADR-0016 point 1)": From E08 increment 7 a hero has **battle bonuses** (a morale and a luck
bonus, each held saturating: the original's byte wraps past 127, A-9's class, fixed; and its
**once-marks**, the original's status bits), which last until the end of its next battle (point
15.1). **From increment 12 they are the hero on the map's** (ADR-0016 point 3 as amended), with its
day's and week's marks, **not the details'**: no visit's effect carries over to a hero that leaves
the map. The once-marks are a short list, each the mark (a `u8` the adventure's content names) with
the morale and the luck its visit gave, sorted by mark and unique, no fixed count; the morale and
luck bonuses are the part no once-mark names (a Pandora's Box's, an event's). The once-per-hero
marks of map objects stay the adventure's (ADR-0016 point 1).

**Point 2** (crates), in the typed hero changes' list: from increment 12 the bonus (morale and luck
with an optional once-mark) and clearing the bonuses are **no longer hero changes** (the adventure
changes its hero on the map); the list keeps every other change.

**Point 13** (movement and scouting), a line in 13's land points: the day's land points take **the
week's land points** as an input (the adventure's: the points of the kinds whose week marks the hero
on the map holds, the Stables' 400), added last, after the growth and the worn artifacts' points,
saturating (the original's `hero_max_land_movement` 0x4E4990 adds the Stables' 400 while bit 1
holds), as the sea's points take the Lighthouses' (HER-ADV-003 amended); the sea's points never
read them.

**Point 15.1** (the battle bonuses reach the battle), replacing its first two sentences: **the
battle bonuses reach the battle as the hero's own effects**: `derive_battle_hero` takes them as an
explicit input beside the details (the game reads them from the hero on the map), and adds the
unmarked morale and luck as the temporary source and each once-mark's non-zero morale and luck as
its own source (`EffectSource::Visit { mark }`; point 6's side modifiers; the battle sums them and
its own bounds apply); a mark with no amount (the Sirens') adds nothing. The input is `h3-heroes`'
plain type (the unmarked pair and the (mark, morale, luck) entries), so the crate still knows no
object. No seam changes. The adventure clears both heroes' bonuses when a battle ends (ADR-0017
point 11), keeping the once-marks its part names as outliving a battle under the profile (A-44's
flag off: the Sirens' mark; E08 decision 308).

**Point 15.2** (the reward record's hero half), a sentence: from increment 12 its bonuses are added
by the adventure to the hero on the map, in the same place of the order (after the spell points,
before the artifacts).

**Consequences**, a line: increment 12 moves the battle bonuses out of the details (the game state
hash v28 without them; the adventure state hash v22 with them on the hero on the map); increment 7's
Warrior's Tomb, Pyramid, Pandora's Box and events change the hero on the map instead of returning a
hero change; `derive_battle_hero` and `land_points` take an input more; the battle names each
visit's morale and luck by its mark, which the client lane's lines read.

**Alternatives considered**, a line: increment 12 (ADV-Q50): the battle bonuses kept in the details
with a fixed set of 32 once-marks and their amounts, and the day's and the week's marks beside them
(the original's records keep them for a hero off the map); the original's several bits per kind
(the Fountain's four, the Idol's and the Temple's two) with the amounts from content (a table
passed in, the unmarked part only by subtraction); the mark widened to `u16` with the seam.

**Revision history**, a line: points 1, 2, 13, 15.1 and 15.2 amended for E08 increment 12, the bonus
visitables: the battle bonuses moved to the hero on the map as a short list of once-marks with
their amounts and the unmarked sums, no longer hero changes; the land points' week input; each
visit's battle source from the bonuses passed in; the clear's kept marks. Drafted in E08 T108b
(survey `E08-garrisons-bonuses.md` §10); batch E12's outcome folded in.
