# ADR-0013: Heroes in battle — entities, the `BattleHero` seam and the hero's battle rules

- Status: Accepted (user, 2026-10-03; OPEN-QUESTIONS Q30)
- Date: 2026-10-03
- Deciders: user; agent session (E04a T1)
- Working detail (field lists, formulas, sizes): the code documentation (`h3_contracts` `hero.rs`, `h3_heroes`, `h3_battle`), the rule catalog and [`docs/epics/E04a-log.md`](../epics/E04a-log.md) ("The design note at closure"; the design note `E04a-design.md` was deleted when E04a closed, 2026-10-05)

## Context

E04a lets a hero lead a battle without magic; E04b adds magic, E04c sieges (Q27). Heroes in battle
and on the adventure map are distinct entities, with no "god" hero type (user, Q27); derived values
are computed before the seam (03-domain). The seam must change once for E04a and E04b.

## Decision

1. **Entities.** Hero type (content, `h3_content::HeroType`), hero details (`h3_heroes::HeroDetails`:
   level, experience, primary and secondary skills, artifacts, spell book, mana), hero on the map
   (E08, own type), battle hero (`h3_contracts::BattleHero`, all derived). No type is named `Hero`;
   contexts meet through ids and explicit pure translations. A cross-context `HeroId` names a
   record of the game's **store of heroes** (ADR-0017 point 1): the contexts read a hero's details
   there by reference and change them by returning **hero changes** that the game applies through
   `h3-heroes`. The army is not part of `HeroDetails`. The details hold the level-up's two
   counters (the levels at which Wisdom and a magic school were last offered; E08 increment 4);
   their level is the levels given, at most the experience's level, the difference the hero's
   pending level-ups. A hero's level seed is not part of them: it is the game's server-only state
   (ADR-0017 point 1). From E08 increment 6 the details hold the **worn artifacts by position**:
   one place per position of the ruleset's artifact slots, in the slots' order and each slot's
   count (head, shoulders, neck, right hand, left hand, torso, two rings, feet, the misc
   positions, the four war machines, the spell book), each empty or holding an artifact
   instance; the backpack stays an ordered list. A combination's locks stay counts per slot (the
   original's): the free positions of every slot, less the locks of the worn combinations, never
   go below zero. The positions' order is the original's walk for "the first position that takes
   it" (a received artifact, a disassembled part, a combination built) and for the capture
   (HER-ART-004); within one slot it is the original's numbering; across slots the war machines
   and the spell book, which no walk takes, sit before the fifth misc position in the original
   and after it here, which no rule can tell apart. The import keeps the map's positions. From E08
   increment 7 a hero has **battle bonuses** (a morale and a luck bonus, each held saturating: the
   original's byte wraps past 127, A-9's class, fixed; and its **once-marks**, the original's
   status bits), which last until the end of its next battle (point 15.1). **From increment 12
   they are the hero on the map's** (ADR-0016 point 3), with its day's and week's marks, **not the
   details'** (increment 7 held them in the details, a small set of marks; E08 decisions 306,
   307): no visit's effect carries over to a hero that leaves the map. The once-marks are a short
   list, each the mark (a `u8` the adventure's content names) with the morale and the luck its
   visit gave, sorted by mark and unique, no fixed count; the morale and luck bonuses are the part
   no once-mark names (a Pandora's Box's, an event's). Batch E12 read them so (E12-8, E12-10,
   E12-11: the status bits set by each visit, cleared by a battle, the week's bit the visitor's
   alone). The once-per-hero marks of map objects stay the adventure's (ADR-0016 point 1).
2. **Crates.** `h3-heroes` is the **hero model**: a logic crate in the shared layer beneath the
   contexts (kernel, content, contracts only) with `HeroDetails`, its validation, the translations
   (`derive_battle_hero`, `war_machines`) and the typed hero changes (`HeroChange`: experience,
   spell points, a spell, an artifact; from E08 increment 4 a level-up: the level, its primary
   skill, the learned skill, the counters; from increment 7 a primary skill gain, each value above 0
   added and saturating at `primary_max`, A-9's class; a skill raised to a level, never lowered, a
   new skill only below the most; a bonus, morale and luck added with an optional once-mark, a
   marked bonus taken only while its mark is clear, then setting it; clearing the bonuses, both to 0
   and every mark clear; from increment 12 the bonus and the clear are **no longer hero changes**,
   the adventure changing its hero on the map, point 1; the list keeps every other change) with
   their checks; no commands, no map, game or battle state. The adventure, the game and later the
   towns depend on it; `h3-battle` never does and sees only `BattleHero` (points 4 and 5). **What a
   hero has from its details in every context is derived here once** (BAT-Q19): its available spells
   (learned, worn scrolls, the spell-giving artifacts) and its mastery of a spell
   (`h3_heroes::available_spells`, `spell_masteries`), from E08 increment 3 the adventure's Attack
   and Defense, Diplomacy's join bonus, the maximum spell points, Necromancy and Learning, from
   increment 4 progression (point 12), from increment 5 its movement and scouting (point 13), from
   increment 6 its artifact moves (point 14), from increment 7 its rewards (point 15); no context
   keeps its own copy, and no adventure profile seam copies them (E08 decision 5 withdrawn).
3. **Effects are data.** Artifacts, skill levels and specialties carry a closed `HeroEffect` enum in
   content (Tier 3, like `Ability`, ADR-0012); the translation resolves them into a closed
   `BattleEffect` enum in contracts. Its morale and luck kinds and alignment groups carry their
   **source** (`EffectSource`, CLT-Q40, BAT-Q64: a skill and its level, an artifact type, the
   specialty, a visit's once-mark, a town building, temporary), one effect per source: the battle
   sums them and reads no source, which is for viewers. Percentages are **basis points** in
   content and seam.
   Progression is data too (Tier 2, ADR-0006): the hero parameters hold the experience of levels
   1 to 12, the highest level and the skills a level-up forces (Wisdom, the magic schools); a
   hero class holds its primary chances, skill weights and two forcing intervals. E08 increment
   5 adds six kinds that only the adventure's derivations read (none reaches `BattleEffect`): the
   **Pathfinding level**, a **land movement rate** (basis points of the land table's entry, grown
   by a skill specialty as the battle's skill rates are), **land movement points** (flat), a
   **flying level** and a **water-walking level** (0 to 3, the original's index into its price
   table; the artifacts give 3) and a **scouting radius bonus** (tiles). Each artifact type counts
   once, worn or as a part of a worn combination. The tables they apply to (the land points by
   speed, the base radius, the step, fly and water-walk prices) are the adventure part's data and
   come in as arguments. E08 increment 6 adds two: **`meeting_spells { level }`** (Scholar: the
   highest spell level a meeting may teach, 2, 3, 4 by the skill's level; several sources: the
   highest) and **`gives_spell_book`** (Titan's Thunder: equipping the artifact puts a Spell Book
   into the empty spell book position; removing it takes nothing back). The packs carry them.
   E08 increment 8 adds three kinds only the adventure's derivations read (none reaches
   `BattleEffect`): the **Navigation level** (`navigation { level }`, 1 to 3: the index into the
   adventure part's sea table, 0 without one, as the Pathfinding level indexes its prices;
   several sources: the highest), **sea movement points** (`sea_movement_points { points }`,
   flat: the Necklace of Ocean Guidance 1 000, the Sea Captain's Hat 500; the Admiral's Hat
   counts both through its parts) and **free boarding** (`free_boarding`, no parameter: the
   Admiral's Hat). A skill specialty on the skill that gives the Navigation level adds, with that
   skill, its `bp_per_level` of the sea table's first entry per hero level (the original's
   Navigation specialists, 75 a level), never a growth of the level. The packs carry them
   (Navigation's three levels, the three artifacts).
   E08 increment 9 adds four kinds only the day's derivations read (none reaches
   `BattleEffect`): a **daily resource** (`daily_resource { resource, amount }`: an artifact's,
   a skill level's or a specialty's income a day: the six resource artifacts 1, the Cornucopia 4
   of mercury, sulfur, crystal and gems, the Endless Sack, Bag and Purse of Gold 1 000, 750 and
   500, Estates 125, 250 and 500 gold, the resource specialists 1 of wood to gems, the gold
   specialists 350), the **day's spell point regain** (`mana_regain { points }`: Mysticism's 2, 3,
   4; several sources: the highest), an **extra regain** (`extra_mana_regain { points }`: the
   Charm, the Talisman and the Mystic Orb of Mana 1, 2, 3) and **full mana** (`full_mana`, no
   parameter: the Wizard's Well). The hero parameters gain the base regain (1) and the regain
   specialists' bonus (1). The packs carry them (the artifacts, the two skills' levels, the
   thirteen resource and gold specialists, whose specialty the packs left empty before: eleven in
   `base`, Gelare and Grindan in `ab`). The Cornucopia's 4, not the original's 5: its four parts'
   own 1 each count through the combination (point 16.1), which makes the original's 5
   (`count_worn_artifact` counts an assembled combination alone; batch E9 confirmed the 5), the
   wiki's "4 + 1". The specialists' income is a specialty's `daily_resource`, not a specialty kind
   of its own: a specialty is a list of effects already.
4. **`BattleHero` is a hybrid:** typed fields for what the battle reads by name (primary skills,
   Tactics level, damage bonuses, war-machine control as derived numbers (the Catapult's: the
   Ballistics level, ADR-0023 point 6; the Ballista's also the Artillery level), surrender
   discount), a bounded list of resolved
   `BattleEffect`s for everything else, and E04b's magic fields reserved
   and validated from E04a on. The battle checks only what its rules need to run soundly
   (references, bounds); keeping a hero within the game's limits is the derivation's job (Q31).
5. **Seam changes.** `BattleSetup.heroes: [Option<BattleHero>; 2]` replaces `led_by_hero` (Q-0003
   reads hero presence); `BattleSetup.funds` for surrenders. `BattleResult` gains `Retreat(side)`
   and `Surrender(side)`. `BattleOutcome` reports facts — per-stack start and end counts, the
   surrender cost, per-hero results, rounds — and the orchestrator applies the consequences
   (casualties, captured artifacts, experience, Necromancy). Later epics add only enum variants;
   E04c adds the reserved optional fields `siege` on setup and outcome and a battlefield kind.
6. **Battle-long hero effects are applied at setup** to the stacks' base stats; morale and luck
   effects become side modifiers (ADR-0012 pool, capacity sized by construction).
7. **Phases and commands.** A tactics phase (round 0) reuses `Move` with cycling activations plus
   `EndTactics`; `Retreat`; `Surrender`, which ends the battle at once (no consent step); a
   hero-controlled Ballista uses `Shoot`, a hero-controlled First Aid Tent a new `Heal`.
8. **The battle prices a surrender**, because every input is in the battle context (living stacks,
   the ruleset's costs, the hero's discount); funds come in through the setup. Should a rule ever
   need information outside the battle, the price becomes an injected input, never an import.
9. **Visibility.** What the original shows in battle (hero identity, primary skills, mana, war
   machines and which of them the hero commands, tactics zone, retreat/surrender options) is
   public (but at a town the defender's option to retreat, which only the defender sees: ADR-0023
   point 2), and so are the hero's morale and luck effects with their sources and each stack's
   unclamped morale and luck (CLT-Q40, BAT-Q64: as VCMI's descriptions of any stack list the enemy
   hero's); other skill levels, bonuses, effects, spell book, funds and the surrender price are private
   to the hero's side, which sees them in its own observation (`observe` reads the ruleset, Q37).
   Hidden battlefield objects (E04b) use the same owner-side projection (ADR-0004).
10. **Encodings.** E04a extends the dense action encoding (v8: heals, `EndTactics`, Retreat,
    Surrender) and the observation encoding (v4: a hero block per side, the opponent's without
    its private part); E04b designs a factorized one for casting together with E05.
11. **Rule ids.** Battle rules keep `BAT-` (new areas `BAT-TAC`, `BAT-FLE`, `BAT-HER`); derivation
    rules are `HER-` rules in `docs/rules/catalog/heroes.toml`, progression's `HER-EXP-` and
    `HER-LVL-` (E08 increment 4), the adventure's derivations `HER-ADV-` (increment 3's Attack and
    Defense and join bonus, increment 5's movement and scouting).
12. **Progression is `h3-heroes`' rules** (E08 increment 4; pure functions of the details, the
    ruleset and explicit inputs). No context writes a hero: an object or a building describes its
    effect, a context reads the details through `h3-heroes` and returns typed hero changes, and
    the game applies them and gives the levels that follow (point 1, ADR-0017 point 10); later
    increments' experience, skill and primary-skill gains, the levels at load of prisons and
    tavern heroes, and E10's computer players reuse the same functions.
    1. **Experience to levels:** levels 1 to 12 from the table, each further step the previous
       × 1.2 truncated (the original's); the experience is bounded at the highest level's range,
       so **level 74 is the highest** (A-7, always fixed, as HotA's fix).
    2. **The scenario's level limit** is an input: a gain caps the experience at the limit's
       threshold and lowers experience above it (D-19); a limit of 0 or above the highest level
       is none (A-8, always fixed).
    3. **A level-up** is drawn one level at a time through a **draw source its caller gives**:
       the primary skill from the class's chances (the second row from the level-up to level 10,
       D-15), +1 saturating at `primary_max` (A-9, always fixed); two offers by the original's
       procedure (an advance, then a new skill; Wisdom and a school forced after the class's
       intervals; the class's weights, a known school weighing 1 when one is due, a banned skill
       0, or 1 if known: D-17, D-18), the map's banned skills an input. The Elementalist's
       intervals are the original's 6 and 4, the magic classes' 3 and 3 by a data quirk off in
       `strict-fixed` (A-10, `ADV-QUIRK-10`).
    4. **The original's source** is provided here: its C runtime generator reseeded per level
       from the hero's seed and the new level. Which source a game uses is ADR-0017's (point 4).
    5. **The answer** is the level-up's hero change; who answers is the game's (ADR-0017 point 6).
    6. **The original's choice between two offers** (the computer's evaluation, 0x52BBD0 with
       0x524690 and 0x524DD0) is ported as a rule here, read first: it chooses at load (D-20) and
       for the system actor's default, and will for E10's computer players; what it needs beyond
       the details is an argument: the army's worth (the setup's defaults at load) and the map's
       terrain shares, which the game computes from the static map (E08 T25, ADV-Q17).
13. **Movement and scouting are `h3-heroes`' derivations** (E08 increment 5; pure functions of
    the details, the ruleset and explicit inputs):
    1. **The day's land points** from the land table's entry the adventure gives: grown by the
       hero's land movement rates (Logistics 10, 20, 30 %; a Logistics specialist's rate grown by
       5 % of itself per level) and rounded down, then the flat points added (Boots of Speed 600,
       Equestrian's Gloves 300). The rate follows Q-0005's flag: the original's single-precision
       rate and x87 product in `faithful`, exact in `strict-fixed` (D-22). From E08 increment 12
       they take **the week's land points** as an input (the adventure's: the points of the kinds
       whose week marks the hero on the map holds, the Stables' 400), added last, after the
       growth and the worn artifacts' points, saturating (the original's
       `hero_max_land_movement` 0x4E4990 adds the Stables' 400 while bit 1 holds; batch E12's
       E12-10: day 2's 1 960 for each visitor, 1 560 once the week ended), as the sea's points
       take the Lighthouses' (item 6; HER-ADV-003 amended); the sea's points never read them.
    2. **A creature's speed for the day's points**: the creature table's speed, one faster for the
       hero's creature specialty's creature and its upgrades; no speed artifact counts (the code
       and the wiki). The army stays the adventure's, an argument.
    3. **The Pathfinding, flying and water-walking levels**: the highest of the hero's sources,
       none without one; the adventure prices steps and passes tiles by them (ADR-0016 point 4).
    4. **The scouting radius**: the base radius the adventure gives plus the hero's bonuses
       (Scouting 1, 2, 3; the Speculum and the Spyglass 1 each). The original's Scouting specialty
       (the radius × (1 + 0.05 × level)) has no SoD hero: not built; a skill specialty on Scouting
       grows nothing.
    5. **When** a property takes effect is the adventure's and the game's (ADR-0016 points 3 and
       5, ADR-0017 point 3): a gain in the day changes neither the points left nor the explored
       tiles until the owner's next turn start or the hero's next step.
    6. **The day's sea points** (E08 increment 8) from the sea table the adventure gives (the
       points by Navigation level, 0 to 3): the entry at the hero's Navigation level, then a
       Navigation specialist's ⌊hero level × the table's first entry × `bp_per_level` ÷ 10 000⌋
       when the hero has the skill (⌊level × 1 500 ÷ 20⌋ in the original packs), then the flat
       sea movement points of the worn artifact types (each type once, worn or as a part of a
       worn combination), saturating. The army's speeds do not count (the original's
       `hero_max_land_movement` 0x4E4990 with its sea argument). Exact integers in every
       profile: no x87 product, no Q-0005 term. The computer player's bonus (E10) comes as an
       input. From increment 9 the sea points also add an input the adventure gives: its
       owner's **Lighthouses**, each owned one's points (500, the kind's; ADV-OWN-005), after the
       artifacts', never grown by Navigation (the original's `hero_max_land_movement` 0x4E4A18
       adds the count × 500 last), saturating. The Castle's Lighthouse building (A-30) is E09's.
    7. **Free boarding** (E08 increment 8): whether the hero wears an artifact type with
       `free_boarding` (or a combination holding one; the original tests the Admiral's Hat,
       `hero_wears_artifact` 0x4D91F0 with 136, so its two parts worn apart do not count). The
       conversion it allows at a boarding or a landing is the adventure's rule (ADR-0016 point
       3); here only the test.

    Item 5 holds for both: a gain in the day changes neither the points left nor the day's
    maximum until the owner's next turn start, or a boarding or a landing (ADR-0016 point 3).
14. **Artifact moves, combinations and Scholar are `h3-heroes`' rules** (E08 increment 6; pure
    functions of one or two heroes' details and the ruleset, checked before they change anything,
    applied as hero changes, point 2):
    1. **A move** takes an artifact from a position or a backpack place of a hero to a position
       or a backpack place of the same hero or of the hero it meets (ADR-0017 point 6): to a
       position of its slot when the locks fit (an artifact there swaps back to the source's
       place), into the backpack at an index (refused at the ruleset's 64 and for the slots not
       carried: the war machines, the Catapult, the spell book). The Spell Book and the Catapult
       never move; the war machines move only between machine positions; an artifact without a
       slot (the Grail) only between backpacks. A move that equips Titan's Thunder applies
       `gives_spell_book`, as every equip does (a received artifact, a built combination).
    2. **A combination** is assembled at the position of a worn part when every part is worn
       (each part's type at least once): the part there, then one copy of each other part from
       the positions in descending order, leave; the combination goes to the first position that
       takes it. A worn combination is disassembled: it leaves, and each part, in the ruleset's
       artifact order, goes to the first position that takes it. A combination in the backpack is
       neither (D-28, the original's). Combinations exist where the game's configuration enables
       them (the expansions the ruleset is compiled from, ADR-0017 point 1), whatever the map's
       version. The original's once-per-player question after an equip and its question at a
       received part are the assembly offered as a command; a computer player's build at once is
       E10's.
    3. **Scholar at a meeting**: s is the greater `meeting_spells` level of the two heroes; with
       none, or either hero without a worn spell book, nothing; else each hero learns the spells
       the other has learned (the details' spells, not an artifact's or a scroll's) and it has not,
       of a level at most the lesser of its learnable level (HER-SPL-005) and s, in spell order.
    4. **Hero changes** grow by what a move, an assembly and a disassembly need (a placed
       artifact, a removal at a position, the swap's return); a move between two heroes is checked
       on both before either changes.
15. **Rewards are `h3-heroes`' rules** (E08 increment 7; pure functions of the details, the
    ruleset and explicit inputs; the adventure reads them through these functions and returns the
    changes, the game applies them, point 12's pattern):
    1. **The battle bonuses reach the battle as the hero's own effects**: from E08 increment 12
       `derive_battle_hero` takes them as an explicit input beside the details (the game reads
       them from the hero on the map; increment 7 read them from the details), and adds the
       unmarked morale and luck as the temporary source and each once-mark's non-zero morale and
       luck as its own source (`EffectSource::Visit { mark }`; point 6's side modifiers; the
       battle sums them and its own bounds apply); a mark with no amount (the Sirens') adds
       nothing. The input is `h3-heroes`' plain type (the unmarked pair and the (mark, morale,
       luck) entries), so the crate still knows no object. No seam changes. The adventure clears
       both heroes' bonuses when a battle ends (ADR-0017 point 11; increment 7's game cleared them
       in the details), keeping the once-marks its part names as outliving a battle under the
       profile (A-44's flag off: the Sirens' mark; E08 decision 308). Batch E12 read the clear
       (E12-8: the Sirens' bit cleared by a won battle, the boat's kept; E12-11: each battle
       clears the Idol's and the Temple's bits and bytes).
    2. **The reward record's hero half** (Pandora's Box, events), in the original's order against
       the details as each part leaves them: the experience amount computed first, with the
       hero's Learning before the record's skills (HER-EXP-001's factor, truncated); the primary
       skills (positive values only); the skills raised to their levels; the spell points added
       and the total held between 0 and the ruleset's **mana ceiling** (999, a hero parameter: a
       hero above it falls to it); the bonuses added; the artifacts, each received as point 15.4
       says and **lost** when refused; the spells, each learned only with a worn spell book, of a
       level at most the learnable level (HER-SPL-005) and not known, the map's bans not tested;
       the experience **last**. Resources and creatures are the adventure's and the game's
       (ADR-0016 point 4, ADR-0017 point 6). From E08 increment 12 its bonuses are added by the
       adventure to the hero on the map (point 1), in the same place of the order (after the spell
       points, before the artifacts).
    3. **What a teacher may teach** is derived here: a Witch Hut's skill (not known, below the
       most skills; no class test); a University's slot (not known, below the most, and the
       class's level-up weight above 0); a Scholar's spell (as point 15.2's), skill (a known one
       a level up below Expert, a new one at Basic below the most; no class test) and its
       fallback; a Shrine's and the Pyramid's spell (a worn spell book, the learnable level, a
       Shrine also not known); the Library's condition (the level plus 2 × the Diplomacy level at
       least 10); the Learning Stone's and a chest's experience (an amount × the Learning factor,
       truncated); the Tree of Knowledge's (the experience to the next level, no Learning).
    4. **Whether a hero can receive an artifact**: the original tests the backpack's count below
       its most (64) before it places anything, so a full backpack refuses an artifact whatever
       the free positions (A-25, `ADV-QUIRK-22`, on in `faithful`); with the flag off
       (`strict-fixed`) the artifact is placed first (the first position that takes it, else the
       backpack below its most: HER-CHG-002), and only an artifact without room is refused (E08
       decision 145). The caller loses or converts a refused artifact.

16. **The day's income and spell points are `h3-heroes`' derivations** (E08 increment 9; pure
    functions of the details, the ruleset and explicit inputs; the game calls them at the day's
    start, ADR-0017 point 3, and applies the points as a hero change):
    1. **A hero's income** (GAM-RES-004), by resource: the `daily_resource` effects of every
       artifact in a worn position and of the parts of each worn combination, **each copy
       counting** (the original counts worn slots: two Rings of Infinite Gems give 2 gems),
       unlike the movement's and the battle's each type once; those of its skills' levels,
       each grown, when the hero has the skill, by a skill specialty on that skill by its
       `bp_per_level` of itself per hero level, rounded down; and those of its specialty.
       Exact integers in every profile, no Q-0005 term: the original's product keeps the
       factor 1 + 0.05 × level in the x87 register (only the 0.05 is a single, 0x63EAE4), and
       its excess never crosses an integer the exact product does not reach (the products are
       multiples of 0.05). Saturating. The mines', the towns' and the creatures'
       (`weekly_resource`) parts are the contexts' (ADR-0017 point 10).
    2. **A hero's day's spell points** (HER-MAN-002), from its points and its maximum
       (HER-MAN-001): with a `full_mana` effect worn the maximum when below it; else its points
       plus the day's regain, at most the maximum, and never lowered (points above it stay).
       The regain: the highest `mana_regain` of its sources, else the hero parameters' base;
       with the skill that gives it and a skill specialty on that skill, grown as 16.1's,
       rounded down, plus the parameters' specialists' bonus (`hero_mana_regained` 0x4E3F40:
       nothing for a specialist without the skill; the `+ 1` as read, a parameter so a re-read
       changes data, not code); then each worn type's `extra_mana_regain` once (worn or as a
       part of a worn combination, the original's `hero_wears_artifact`). Batch E9 confirmed
       the day's points (E08 T80).
    3. **Every hero of the store** regains (the original's 156 records: on the map and off
       it); a hero's income counts only for its owner's heroes on the map (garrison heroes:
       E09).
    4. **A hero's town growth** (E09 increment 3; TWN-GRO-001's inputs), two pure derivations
       of the details and the ruleset, each worn artifact type counting once whether worn or a
       part of a worn combination (the original's `hero_wears_artifact` 0x4D91F0: a yes or no
       per type):
       - **by level**, the `town_growth` effects' amounts summed per creature level (the Legs,
         Loins, Torso, Arms and Head of Legion: +5, +4, +3, +2, +1 at levels 2 to 6); the game
         adds the garrison hero's and the visitor's to their town's growth, nobody else's;
       - **the owner's share**, the highest `owner_town_growth` in basis points of the hero's
         sources (the Statue of Legion: 5 000, half); the game takes the highest of the owner's
         heroes on the map and in its towns' garrisons (the original's
         `player_hero_wears_artifact` 0x4BACB0) for each of the owner's towns.
       A worn Statue of Legion, a combination of the five pieces, gives both: its share and,
       through its parts, the pieces' amounts. Batch TWN-3 confirmed both (E09 T23: the Statue
       worn elsewhere in every town of its owner, TWN-3-1; the Legs of Legion on the visitor,
       TWN-3-2). Pack format v52 (`h3-content`'s `HeroEffect::TownGrowth { level, amount }` and
       `HeroEffect::OwnerTownGrowth { bp }`, core ruleset hash v53; the base pack's five pieces,
       the SoD pack's Statue).

## Consequences

- The battle stays ignorant of hero management; a skill's numbers are a data change, a new effect
  kind is one variant in content, one in contracts and one rule.
- Every caller of the seam changes in E04a (T4, T9, T11); state hash v9, ruleset hash v9.
- Enforced by `check-arch` (no `h3-battle` → `h3-heroes` edge: a forbidden edge in `arch.toml`,
  since the layers alone allow it from E08 T18a), property tests (hero isolation, observation
  hygiene, conservation of creatures) and `SetupError` tests for every validation.
- One store of heroes, as the original keeps one record per hero: heroes off the map (the tavern
  pool, prisons; E09) need no second home. Rejections stay atomic: a context checks a hero's
  details before it returns changes, and the game applies them infallibly (point 2).
- `h3-heroes` moves to the adventure lane once the battle lane is done with it (the user,
  ADV-Q13; a `shared/` change of `lanes.toml`); until then the adventure's additions to it are
  `cross-lane`.
- This revises the epic's non-goal "the orchestrator computes the surrender cost" (point 8).
- Progression (point 12): faithful level-ups are a function of the seed, the level and the hero's
  skills, as the original's; the data adds hero parameters, a class field and a class override
  kind (the pack format, the core ruleset hash); the details' counters move the game state hash;
  the evaluation's port needs a code reading first (E08 T25).
- Movement and scouting (point 13): increment 4's level-ups now matter on the map from the
  hero's next turn or step; six effect kinds (the pack format, the core ruleset hash), the skills
  and artifacts getting their effects through the harvester's gap-fill; later sources (Stables,
  Lighthouses, Fly, Water Walk) add to the same functions, Navigation's sea points become a
  sibling of point 13.1 with boats; nothing in increment 5 writes a hero.
- Artifacts by position (point 1, increment 6): every place that builds `HeroDetails` changes
  (the import, the setup's defaults, the tests and the experiments of the battle, AI and client
  lanes); a constructor that places a list of artifacts at their first free positions keeps
  those call sites one line each (`cross-lane` where they are another lane's). The capture, a
  lost war machine's artifact and a received artifact follow positions; the details' hash and the
  game state hash change; the golden replays are re-recorded. Two effect kinds (the pack format,
  the core ruleset hash). A meeting's Scholar and a hero's moves are deterministic functions: no
  stream, no draw.
- Rewards (point 15, increment 7): `HeroDetails` grows by the battle bonuses (the details' hash
  and the game state hash; the import keeps them at 0); `derive_battle_hero` reads two more
  fields, so every battle from the map gets them and a battle without the map (E01 to E04c's
  setups) has none. The mana ceiling is a hero parameter of the core content and A-25 a quirk
  flag (the pack format, the core ruleset hash). No rule outside `h3-heroes` computes a learnable
  spell, a skill's room or a reward's order.
- Boats and the sea (points 3, 13.6 and 13.7, increment 8): three effect kinds (the pack format,
  the core ruleset hash), Navigation and the three sea artifacts getting them through the
  harvester's gap-fill; no hero change and no new `HeroDetails` field, so the details' hash keeps
  its version: a hero's boat, its day's maximum and its points are the adventure's (ADR-0016
  point 3).
- Income and spell points (points 3, 13.6 and 16, increment 9): four effect kinds and two hero
  parameters (core ruleset hash, pack format; the harvester's gap-fill carries the thirteen
  specialties, `cross-lane` with the shared task); two derivations; 13.6 gains an input. No seam
  changes.
- The bonus visitables (points 1, 2, 13.1 and 15, increment 12): the battle bonuses move out of
  the details (the game state hash v28 without them; the adventure state hash v22 with them on
  the hero on the map); increment 7's Warrior's Tomb, Pyramid, Pandora's Box and events change the
  hero on the map instead of returning a hero change; `derive_battle_hero` and the land points
  take an input more; the battle names each visit's morale and luck by its mark, which the client
  lane's lines read. No seam changes (`EffectSource::Visit { mark: u8 }` stays).

## Alternatives considered

- One hero type for all contexts, or `HeroDetails` across the seam: couples battle to hero
  management (rejected by the user, Q27).
- `h3-heroes` as a peer domain crate the adventure sees through a profile seam (E08 decision 5):
  every read a copied field, every write an outcome variant and code in `h3-game`; the details
  with the hero on the map in the adventure: heroes off the map need a second home; the hero rules
  in `h3-adventure`: the battle's pack tests and the towns would depend on the adventure (E08
  T17b, ADV-Q13).
- Typed fields only (seam grows per artifact kind) or an effects list only (list searches on the hot
  path, legality depends on list contents).
- Hero attack and defense as pool modifiers: recomputation for values fixed for the whole battle.
- Whole percents: cannot express 17.5 % or 0.25 % steps; floats are banned (ADR-0003).
- A `Place { stack, hex }` tactics command: about 3 500 extra actions for little gain.
- The orchestrator pricing surrenders: the action mask could not tell whether surrender is
  affordable.
- Progression (E08 ADV-Q16): the first offer at load (VCMI, the wiki; the code reading twice says
  otherwise, D-20); VCMI's table to level 197 (diverges from SoD's levels, A-7); the intervals by a
  might/magic affinity of the class (no other rule reads one); progression in `h3-game` (towns,
  objects and computer players would each need the game).
- Movement and scouting (E08 ADV-Q21): the whole day's points formula here with the land table as
  hero data (the table is the adventure's, ADV-MOV-001); VCMI's bonus system (its speed artifacts
  count towards the slowest creature, against the code and the wiki; its integer percents round
  the specialists differently from both profiles, D-22); the Scouting specialty built from the
  code (no SoD content reaches it; the wiki and VCMI each have another formula).
- Artifacts and Scholar (E08 ADV-Q24): the slot kinds kept, positions interchangeable (no command
  could name a position, and the capture's order would follow the list, not the original's
  positions); the combination at the clicked part's position (VCMI; the original equips it at the
  first position that takes it); disassembly in the backpack (VCMI, with room for the parts; the
  original disassembles only a worn combination).
- Rewards (E08 ADV-Q28, ADV-Q29): the battle bonuses on the hero on the map (`h3-adventure`; the
  battle's derivation would need a second source, the original keeps them in the hero) or as a
  temporary artifact-like effect (the original adds them up and clears them at a battle's end,
  which an effect list does not express); VCMI's reward order and sums (D-37 to D-39: batch E7
  showed the code's side, E08 T58).
- Boats (E08 ADV-Q32): Navigation as a rate of the base sea points (VCMI's `PERCENT_TO_BASE`),
  not a level into the sea table (the original's `sea_movement_by_navigation` 0x698AEC, four
  entries from `movement.txt`, and the Pathfinding level's precedent; both give 1 500, 2 250,
  3 000, 3 750).
- The bonus visitables (E08 ADV-Q50): the battle bonuses kept in the details with a fixed set of
  32 once-marks and their amounts, and the day's and the week's marks beside them (the original's
  records keep them for a hero off the map); the original's several bits per kind (the
  Fountain's four, the Idol's and the Temple's two) with the amounts from content (a table passed
  in, the unmarked part only by subtraction); the mark widened to `u16` with the seam.

## Revision history

- 2026-10-03: Proposed (E04a T1); revised after an internal consistency review.
- 2026-10-03: Accepted by the user. Cut to its decisions (ADR size rule, `docs/adr/README.md`); the
  working detail moved to `docs/epics/E04a-design.md`. Point 8 records the user's condition.
- 2026-10-04: point 7 — the optional consent step for a surrender is dropped (user, OPEN-QUESTIONS
  Q32): in SoD neither a human nor the computer opponent is asked (experiment batch 1b R4), and in
  HotA the enemy cannot refuse either (wiki, Combat). Point 9 no longer lists offers.
- 2026-10-04 (E04a T12a): what E04a's tasks decided, folded in — point 4: the battle checks only
  soundness (Q31); point 9: which war machines a hero commands is public, the own side's private
  part is in its observation (T11a, Q37); point 10: the encodings E04a ended with (T11b). Detail
  that stays out of the ADR: `FactionGroup` (Q36) and `StatOp` in the crate docs.
- 2026-10-05 (E04a closure): no decision changed; the header points to the code documentation and
  the epic's log, because the design note was deleted.
- 2026-10-10 (E04b T8a; the user, BAT-Q19): point 2 — hero-level magic (available spells, the
  mastery of a spell) is derived once in `h3-heroes` for every context, as the original keeps it
  in the hero record (`installation.md` §7.22); the battle's own best-school code went, and the
  seam carries a mastery per spell (ADR-0014 point 2).
- 2026-10-13 (E04c T5a; the user, BAT-Q40): point 5's reserved `siege` fields are decided by
  ADR-0023: the setup's is `BattleSetup.siege`, the outcome's is dropped (the original writes
  nothing about the walls back to the town). The battlefield kind is ADR-0022's.
- 2026-10-14 (E08 T18a; the user, ADV-Q13): points 1 and 2 amended: `h3-heroes` is the hero
  model in the shared layer beneath the contexts, read by them by reference and changed through
  typed hero changes; the store of heroes is the game's (ADR-0017 point 1); E08 decision 5's
  adventure profile withdrawn (drafted in E08 T17b, #181).
- 2026-10-14 (E04c T6; the user, BAT-Q43): point 4 — the Catapult's control carries the hero's
  Ballistics level, not derived numbers (ADR-0023 point 6): the battle's own Ballistics table also
  serves the uncontrolled Catapult and the Cyclops.
- 2026-10-15 (E04c T7e, BAT-STUB-3): point 4 — the Ballista's control carries the hero's Artillery
  level beside its derived shots and chance: the computer's worth of a Ballista grows with it
  (`installation.md` §7.33), and the turrets and the automatic Ballista read that worth. Private,
  as the rest of the control (point 9).
- 2026-10-15 (E04c T10b; the user, BAT-Q50): point 9 — at a town the defender's option to retreat
  (the town's Escape Tunnel) is private to the defender: the original enables the retreat control
  for the acting side only (0x478573), and a town's buildings are no public fact (ADR-0023 point 2).
- 2026-10-20 (E08 T26a; the user at ADV-Q15 and ADV-Q16): points 1, 2, 3 and 11 amended and point
  12 added for E08 increment 4, hero progression: the details' counters and the level's meaning,
  the level-up's hero change, progression's data, `HER-EXP-` and `HER-LVL-`, and progression as
  `h3-heroes`' rules (experience to levels bounded at 74, the level limit as an input, a level-up
  through a draw source its caller gives, the original's offers, the original's choice ported);
  no context writes a hero (the user's note at ADV-Q16). Drafted in E08 T24b1 (#318; survey
  `E08-progression.md`).
- 2026-10-22 (E08 T35a; the user at ADV-Q20 and ADV-Q21): points 2, 3 and 11 amended and point 13
  added for E08 increment 5, the hero's movement and scouting: six hero-effect kinds the
  adventure's derivations read, `HER-ADV-` rules, and the day's land points (D-22 under Q-0005),
  a creature's speed for them, the Pathfinding, flying and water-walking levels and the scouting
  radius as `h3-heroes`' derivations, taking effect at the owner's next turn or the hero's next
  step. Drafted in E08 T34b (#375; survey `E08-hero-movement.md`).
- 2026-10-23 (E08 T45; the user at ADV-Q24 and ADV-Q25): points 1 and 3 amended and point 14
  added for E08 increment 6, army and hero management: the worn artifacts by position (the
  original's walk and the capture's order; the locks per slot), two effect kinds
  (`meeting_spells`, `gives_spell_book`), artifact moves, combinations (where the game's
  configuration enables them, E08 decision 104) and Scholar at a meeting as `h3-heroes`' rules.
  Drafted in E08 T44b (#412; survey `E08-army.md` §2.2, §5).
- 2026-10-27 (E08 T59; the user at ADV-Q28, ADV-Q29 and ADV-Q30): points 1 and 2 amended and
  point 15 added for E08 increment 7, rewards on the map: the battle bonuses and once-marks in the
  details, four hero changes, and rewards as `h3-heroes`' rules (the bonuses as the hero's own
  effects in its next battle, the reward record's hero half in the original's order with the mana
  ceiling, what a teacher may teach, an artifact received under A-25's flag). Drafted in E08 T55b
  (#484; survey `E08-rewards.md`), batch E7's outcome (T58, §15) and decision 145 folded in.
- 2026-10-29 (E08 T70 `shared/`; the user at ADV-Q32 and ADV-Q33): points 3 and 13 amended for
  E08 increment 8, boats and the sea: three hero-effect kinds (`navigation`,
  `sea_movement_points`, `free_boarding`), the day's sea points (the sea table by Navigation's
  level, a Navigation specialist's flat points, the sea artifacts') and the free boarding test as
  `h3-heroes`' derivations (`HER-ADV-009`). Drafted in E08 T66b (survey `E08-sea.md`).
- 2026-10-30 (E08 T81a `shared/`; the user at ADV-Q37 and ADV-Q38, E08 decisions 196–204):
  points 3 and 13.6 amended and point 16 added for E08 increment 9: the kinds `daily_resource`,
  `mana_regain`, `extra_mana_regain` and `full_mana`, the base regain and the specialists' bonus;
  the Lighthouses' input to the sea points; the heroes' income (each copy counting) and the day's
  spell points. Drafted in E08 T77b (survey `E08-economy.md`), batch E9's outcome (T80, §19)
  folded in.
- 2026-11-03 (CLT-Q40, BAT-Q64; the user): points 3 and 9 amended. The morale and luck kinds and
  the alignment groups of `BattleEffect` carry their source, one effect per source (`h3-heroes`
  derives them so; the battle sums them as before); the hero's morale and luck effects with their
  sources and the stacks' unclamped morale and luck are public, as VCMI shows them, for the stats
  dialog's descriptions (`docs/epics/E17-design.md` §12.4).
- 2026-11-12 (E09 T24 `shared/`; E09 decision 76, `TWN-Q13`): point 16.4 added, a hero's town
  growth (E09 increment 3; pack format v52). Drafted in E09 T22a, accepted at `TWN-Q14`; batch
  TWN-3's verdicts (E09 T23) cited.
- 2026-11-12 (E08 T112 `shared/`; the user at ADV-Q50, E08 decisions 306–308): points 1, 2,
  13.1, 15.1 and 15.2 amended for E08 increment 12, the bonus visitables: the battle bonuses
  moved to the hero on the map as a short list of once-marks with their amounts and the unmarked
  sums, no longer hero changes; the land points' week input; each visit's battle source from the
  bonuses passed in; the clear's kept marks. Drafted in E08 T108b (survey
  `E08-garrisons-bonuses.md` §10); batch E12's outcome (T111, survey §15) folded in: every tested
  point as drafted.
