# Draft: ADR-0013 amended for E09 increment 3 (the Legion's town growth)

- Status: **Draft** (E09 T22a, 2026-11-11); put to the user with T22b's drafts at its
  checkpoint (`TWN-Q14`); moved into `docs/adr/0013-heroes-in-battle.md` by the `shared/` task
  after batch TWN-3.
- Decisions: the epic's 76 (`TWN-Q13`: the Legion as hero effects, recommended) and 78.
- Working detail: survey `docs/rules/surveys/E09-growth.md` §1 (`town_legion_growth` 0x5BF900,
  `player_hero_wears_artifact` 0x4BACB0); design note `docs/epics/E09-design.md` Part E §39.

## Point 16, a new item after 16.3

4. **A hero's town growth** (E09 increment 3; TWN-GRO-001's inputs), two pure derivations of the
   details and the ruleset, each worn artifact type counting once whether worn or a part of a
   worn combination (the original's `hero_wears_artifact` 0x4D91F0: a yes or no per type):
   - **by level**, the `town_growth` effects' amounts summed per creature level (the Legs, Loins,
     Torso, Arms and Head of Legion: +5, +4, +3, +2, +1 at levels 2 to 6); the game adds the
     garrison hero's and the visitor's to their town's growth, nobody else's;
   - **the owner's share**, the highest `owner_town_growth` in basis points of the hero's sources
     (the Statue of Legion: 5 000, half); the game takes the highest of the owner's heroes on the
     map and in its towns' garrisons (the original's `player_hero_wears_artifact`) for each of the
     owner's towns.
   A worn Statue of Legion, a combination of the five pieces, gives both: its share and, through
   its parts, the pieces' amounts (batch TWN-3's row TWN-3-2 observes a worn piece).
   Pack format v52 (`h3-content`'s `HeroEffect::TownGrowth { level, amount }` and
   `HeroEffect::OwnerTownGrowth { bp }`; the base pack's five pieces, the SoD pack's Statue).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T2x `shared/`; decision 76): point 16.4 added, a hero's town growth (E09
  increment 3). Drafted in E09 T22a.
