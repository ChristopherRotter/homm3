# ADR-0013 amendment draft: E08 increment 9, mines, the week and owned objects (the hero side)

- Status: Proposed (E08 T77b, 2026-10-30); accepted by the user at ADV-Q38 (E08 decisions
  201–204) with decision 201's two changes written in, applied by T81 with batch E9's outcome
- Amends: ADR-0013 points 3 and 13 (13.6), adds point 16
- Sources: survey `docs/rules/surveys/E08-economy.md` §4, §8, §12, §17 (T77a1, T77a2); the
  answers at ADV-Q37 (the epic's decisions 196–199: decision 197 adds the heroes' income and the
  day's spell points); the working detail in `docs/epics/E08-design.md` part A.

Applied by the increment's `shared/` task with batch E9's outcome (design §18), as T70 applied
increment 8's. The text below is what the task writes into the ADR; *Agent's call* marks a choice
the user accepted at the checkpoint (ADV-Q38, E08 decision 204) and may still change.

## Point 3 (effects are data): four kinds, appended after increment 8's

> E08 increment 9 adds four kinds only the day's derivations read (none reaches
> `BattleEffect`): a **daily resource** (`daily_resource { resource, amount }`: an artifact's,
> a skill level's or a specialty's income a day: the six resource artifacts 1, the Cornucopia 4
> of mercury, sulfur, crystal and gems, the Endless Sack, Bag and Purse of Gold 1 000, 750 and
> 500, Estates 125, 250 and 500 gold, the resource specialists 1 of wood to gems, the gold
> specialists 350), the **day's spell point regain** (`mana_regain { points }`: Mysticism's 2, 3,
> 4; several sources: the highest), an **extra regain** (`extra_mana_regain { points }`: the
> Charm, the Talisman and the Mystic Orb of Mana 1, 2, 3) and **full mana** (`full_mana`, no
> parameter: the Wizard's Well). The hero parameters gain the base regain (1) and the regain
> specialists' bonus (1). The packs carry them (the artifacts, the two skills' levels, the
> thirteen resource and gold specialists, whose specialty the packs leave empty today: eleven in
> `base`, Gelare and Grindan in `ab`).

*Agent's call:* the Cornucopia's 4, not the original's 5: its four parts' own 1 each count
through the combination (point 16.1), which makes the original's 5 (`count_worn_artifact` counts
an assembled combination alone, survey §17), the wiki's "4 + 1". The specialists' income as a
specialty's `daily_resource`, not a specialty kind of its own: a specialty is a list of effects
already.

## Point 13 (movement and scouting): 13.6, appended

> From increment 9 the sea points also add an input the adventure gives: its owner's
> **Lighthouses**, each owned one's points (500, the kind's; ADV-OWN-005), after the artifacts',
> never grown by Navigation (the original's `hero_max_land_movement` 0x4E4A18 adds the count ×
> 500 last), saturating. The Castle's Lighthouse building (A-30) is E09's.

## Point 16 (new): the day's income and spell points

> 16. **The day's income and spell points are `h3-heroes`' derivations** (E08 increment 9; pure
>     functions of the details, the ruleset and explicit inputs; the game calls them at the day's
>     start, ADR-0017 point 3, and applies the points as a hero change):
>     1. **A hero's income** (GAM-RES-004), by resource: the `daily_resource` effects of every
>        artifact in a worn position and of the parts of each worn combination, **each copy
>        counting** (the original counts worn slots: two Rings of Infinite Gems give 2 gems),
>        unlike the movement's and the battle's each type once; those of its skills' levels,
>        each grown, when the hero has the skill, by a skill specialty on that skill by its
>        `bp_per_level` of itself per hero level, rounded down; and those of its specialty.
>        Exact integers in every profile, no Q-0005 term: the original's product keeps the
>        factor 1 + 0.05 × level in the x87 register (only the 0.05 is a single, 0x63EAE4), and
>        its excess never crosses an integer the exact product does not reach (the products are
>        multiples of 0.05). Saturating. The mines', the towns' and the creatures'
>        (`weekly_resource`) parts are the contexts' (ADR-0017 point 10).
>     2. **A hero's day's spell points** (HER-MAN-002), from its points and its maximum
>        (HER-MAN-001): with a `full_mana` effect worn the maximum when below it; else its points
>        plus the day's regain, at most the maximum, and never lowered (points above it stay).
>        The regain: the highest `mana_regain` of its sources, else the hero parameters' base;
>        with the skill that gives it and a skill specialty on that skill, grown as 16.1's,
>        rounded down, plus the parameters' specialists' bonus (`hero_mana_regained` 0x4E3F40:
>        nothing for a specialist without the skill); then each worn type's
>        `extra_mana_regain` once (worn or as a part of a worn combination, the original's
>        `hero_wears_artifact`).
>     3. **Every hero of the store** regains (the original's 156 records: on the map and off
>        it); a hero's income counts only for its owner's heroes on the map (garrison heroes:
>        E09).

*Agent's call:* the specialists' `+ 1` as read (survey §12; VCMI's specialty formula was not
compared, §14's open point), as a parameter so a re-read changes data, not code.

## Consequences: appended

> - Increment 9: four effect kinds and two hero parameters (core ruleset hash, pack format; the
>   harvester's gap-fill carries the thirteen specialties, `cross-lane` with the shared task); two
>   derivations; 13.6 gains an input. No seam changes.

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q37 and ADV-Q38, E08 decisions 196–204): points 3 and 13.6 amended
>   and point 16 added for E08 increment 9: the kinds `daily_resource`, `mana_regain`,
>   `extra_mana_regain` and `full_mana`, the base regain and the specialists' bonus; the
>   Lighthouses' input to the sea points; the heroes' income (each copy counting) and the day's
>   spell points. Drafted in E08 T77b (survey `E08-economy.md`).
