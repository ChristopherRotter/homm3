# ADR-0013 amendment (draft): artifacts on positions, artifact moves, combinations, Scholar

- Status: Proposed (E08 T44b, 2026-10-23), from the answers at ADV-Q24 (the epic's decisions
  93–100); for the user's acceptance at ADV-Q25. The round-1 `shared/` task applies it to
  ADR-0013.
- Amends: ADR-0013 point 1 (the details' artifacts), point 3 (two effect kinds); adds point 14.
- Detail: `docs/epics/E08-design.md` part A (§1–§6); facts: survey
  `docs/rules/surveys/E08-army.md` §2.2, §5.

## Context

Increment 6 lets a player manage its heroes' artifacts and armies (ADR-0016's draft) and meet its
own heroes (ADR-0017's draft). The original keeps a hero's worn artifacts on 19 numbered
positions: an artifact goes to "the first position that takes it" in position order, and a
battle's winner captures the worn ones in position order (HER-ART-004). Today's details keep worn
artifacts as a list by slot kind ("positions of one slot are interchangeable", HER-ART-001), which
cannot name a position for a command nor follow the original's order after a move (decision 95).
Two rules need data the packs lack: Scholar's spell exchange at a meeting (survey §2.2; decision
97) and the Spell Book that equipping Titan's Thunder gives (§5.4). Combinations (SoD) are
assembled and disassembled while worn (§5.3).

## Decision

1. *(amended, the artifacts)* A hero's details hold its **worn artifacts by position**: one
   place per position of the ruleset's artifact slots, in the slots' order and each slot's count
   (head, shoulders, neck, right hand, left hand, torso, two rings, feet, the misc positions, the
   four war machines, the spell book), each empty or holding an artifact instance; the backpack
   stays an ordered list. A combination's locks stay counts per slot (the original's): the free
   positions of every slot, less the locks of the worn combinations, never go below zero. The
   positions' order is the original's walk for "the first position that takes it" (a received
   artifact, a disassembled part, a combination built) and for the capture (HER-ART-004); within
   one slot it is the original's numbering, across slots the war machines and the spell book,
   which no walk takes, sit before the fifth misc position in the original and after it here,
   which no rule can tell apart. The import keeps the map's positions.
3. *(amended, two effect kinds)* `meeting_spells { level }` (Scholar: the highest spell level a
   meeting may teach, 2, 3, 4 by the skill's level) and `gives_spell_book` (Titan's Thunder:
   equipping the artifact puts a Spell Book into the empty spell book position; removing it takes
   nothing back). The packs carry them (Scholar's three levels, Titan's Thunder).
14. **Artifact moves, combinations and Scholar are `h3-heroes`' rules** (E08 increment 6; pure
    functions of one or two heroes' details and the ruleset, checked before they change anything,
    applied as hero changes, ADR-0013 point 2):
    1. **A move** takes an artifact from a position or a backpack place of a hero to a position
       or a backpack place of the same hero or of the hero it meets (ADR-0017's draft): to a
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
       neither (D-28, the original's). Combinations follow the game's edition (ADR-0017's draft):
       none in a map before SoD. The original's once-per-player question after an equip and its
       question at a received part are the assembly offered as a command; a computer player's
       build at once is E10's.
    3. **Scholar at a meeting**: s is the greater `meeting_spells` level of the two heroes; with
       none, or either hero without a worn spell book, nothing; else each hero learns the spells
       the other has learned (the details' spells, not an artifact's or a scroll's) and it has not,
       of a level at most the lesser of its learnable level (HER-SPL-005) and s, in spell order.
    4. **Hero changes** grow by what a move, an assembly and a disassembly need (a placed
       artifact, a removal at a position, the swap's return); a move between two heroes is checked
       on both before either changes.

## Consequences

- Every place that builds `HeroDetails` changes (the import, the setup's defaults, the tests and
  the experiments of the battle, AI and client lanes): a constructor that places a list of
  artifacts at their first free positions keeps those call sites one line each (`cross-lane`
  where they are another lane's).
- The capture, a lost war machine's artifact and a received artifact follow positions; the
  details' hash and the game state hash change; the golden replays are re-recorded.
- A meeting's Scholar and a hero's moves are deterministic functions: no stream, no draw.

## Alternatives considered

- **Keep the slot kinds** (positions interchangeable): no command could name a position, and the
  capture's order would follow the list instead of the original's positions (not chosen at
  ADV-Q24).
- **The combination at the clicked part's position** (VCMI): the original equips it at the first
  position that takes it.
- **Disassembly in the backpack** (VCMI, with room for the parts): the original only disassembles
  a worn combination.
