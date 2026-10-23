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
   and after it here, which no rule can tell apart. The import keeps the map's positions.
2. **Crates.** `h3-heroes` is the **hero model**: a logic crate in the shared layer beneath the
   contexts (kernel, content, contracts only) with `HeroDetails`, its validation, the translations
   (`derive_battle_hero`, `war_machines`) and the typed hero changes (`HeroChange`: experience,
   spell points, a spell, an artifact; from E08 increment 4 a level-up: the level, its primary
   skill, the learned skill, the counters) with their checks; no commands, no map, game or battle state. The adventure, the game and later the
   towns depend on it; `h3-battle` never does and sees only `BattleHero` (points 4 and 5). **What
   a hero has from its details in every context is derived here once** (BAT-Q19): its available
   spells (learned, worn scrolls, the spell-giving artifacts) and its mastery of a spell
   (`h3_heroes::available_spells`, `spell_masteries`), from E08 increment 3 the adventure's Attack
   and Defense, Diplomacy's join bonus, the maximum spell points, Necromancy and Learning, from
   increment 4 progression (point 12), from increment 5 its movement and scouting (point 13); no
   context keeps its own copy, and no adventure profile seam copies them (E08 decision 5
   withdrawn).
3. **Effects are data.** Artifacts, skill levels and specialties carry a closed `HeroEffect` enum in
   content (Tier 3, like `Ability`, ADR-0012); the translation resolves them into a closed
   `BattleEffect` enum in contracts. Percentages are **basis points** in content and seam.
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
   point 2); skill levels, bonuses, effects, spell book, funds and the surrender price are private
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
       rate and x87 product in `faithful`, exact in `strict-fixed` (D-22).
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
