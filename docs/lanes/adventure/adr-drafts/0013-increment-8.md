# ADR-0013 amendment draft: E08 increment 8, boats and the sea (the hero side)

- Status: Proposed (E08 T66b, 2026-10-28); accepted as written by the user at ADV-Q33 (E08
  decision 170), applied by T70 with batch E8's outcome
- Amends: ADR-0013 points 3 and 13 (adds 13.6 and 13.7)
- Sources: survey `docs/rules/surveys/E08-sea.md` §3.1, §3.2, §4.2 (T66a1, T66a2), survey
  `E08-hero-movement.md` §1.1 (T34a); the answers at ADV-Q32 (the epic's decisions 162–168);
  the working detail in `docs/epics/E08-design.md` part A.

Applied by the increment's `shared/` task with batch E8's outcome (design §18), as T59 applied
increment 7's. The text below is what the task writes into the ADR; *Agent's call* marks a choice
the user may change at the checkpoint.

## Point 3 (effects are data): three kinds, appended after increment 6's

> E08 increment 8 adds three kinds only the adventure's derivations read (none reaches
> `BattleEffect`): the **Navigation level** (`navigation { level }`, 0 to 3, the index into the
> adventure part's sea table, as the Pathfinding level indexes its prices; several sources: the
> highest), **sea movement points** (`sea_movement_points { points }`, flat: the Necklace of Ocean
> Guidance 1 000, the Sea Captain's Hat 500; the Admiral's Hat counts both through its parts) and
> **free boarding** (`free_boarding`, no parameter: the Admiral's Hat). A skill specialty on the
> skill that gives the Navigation level adds, with that skill, its `bp_per_level` of the sea
> table's first entry per hero level (the original's Navigation specialists, 75 a level), never a
> growth of the level. The packs carry them (Navigation's three levels, the three artifacts).

*Agent's call:* a level into a table (the original's `sea_movement_by_navigation` 0x698AEC, four
entries from `movement.txt`), not a rate as VCMI's `PERCENT_TO_BASE`: the original's data shape,
and the precedent of the Pathfinding level. Both give 1 500, 2 250, 3 000, 3 750.

## Point 13 (movement and scouting): two items, after 13.5

> 6. **The day's sea points** from the sea table the adventure gives (the points by Navigation
>    level, 0 to 3): the entry at the hero's Navigation level, then a Navigation specialist's
>    ⌊hero level × the table's first entry × `bp_per_level` ÷ 10 000⌋ when the hero has the skill
>    (⌊level × 1 500 ÷ 20⌋ in the original packs), then the flat sea movement points of the worn
>    artifact types (each type once, worn or as a part of a worn combination), saturating. The
>    army's speeds do not count (the original's `hero_max_land_movement` 0x4E4990 with its sea
>    argument). Exact integers in every profile: no x87 product, no Q-0005 term. Lighthouses (+500
>    each owned, the sketch's item 9) and the computer player's bonus (E10) come with them, as
>    inputs.
> 7. **Free boarding:** whether the hero wears an artifact type with `free_boarding` (or a
>    combination holding one; the original tests the Admiral's Hat, `hero_wears_artifact`
>    0x4D91F0 with 136, so its two parts worn apart do not count). The conversion it allows at a
>    boarding or a landing is the adventure's rule (ADR-0016 point 3); here only the test.

Point 13.5 (when a property takes effect) holds for both: a gain in the day changes neither the
points left nor the day's maximum until the owner's next turn start, or a boarding or a landing
(ADR-0016 point 3).

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q32 and ADV-Q33): points 3 and 13 amended for
>   E08 increment 8, boats and the sea: three hero-effect kinds (`navigation`,
>   `sea_movement_points`, `free_boarding`), the day's sea points (the sea table by Navigation's
>   level, a Navigation specialist's flat points, the sea artifacts') and the free boarding test as
>   `h3-heroes`' derivations (`HER-ADV-009`). Drafted in E08 T66b (survey `E08-sea.md`).

## Not changed

Points 1, 2 (the details hold nothing new: a hero's boat, its day's maximum and its points are
the adventure's, ADR-0016 point 3), 12, 14 and 15. No hero change, no seam change, no new
`HeroDetails` field, so the details' hash keeps its version; the core ruleset hash and the pack
format move with the three kinds (the `shared/` task).
