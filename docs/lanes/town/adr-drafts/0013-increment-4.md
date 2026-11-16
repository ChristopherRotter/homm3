# Draft: ADR-0013 amended for E09 increment 4 (a tavern hero's details)

- Status: **Draft**, written by E09 T33a (2026-11-16), **accepted** at T33b's checkpoint
  (`TWN-Q18`, decisions 114 to 122);
  then moved into `docs/adr/0013-heroes-in-battle.md` by the increment's `shared/` task. T33b
  adds nothing here (the Thieves' Guild reads a hero's primary skills and army power, both
  derived already).
- Decisions: the epic's 99, 109 (a hero type's record at its first draw), 113.
- Working detail: survey `docs/rules/surveys/E09-tavern.md` §2.3, §4; design note
  `docs/epics/E09-design.md` Part F §63, §67.

## Point 14, a new item after 14.4

5. **A tavern hero's backpack equipped** (E09 increment 4; TWN-TAV-002; the original's
   `tavern_new_heroes` 0x4C8450 with `hero_equip_artifact` 0x4E2A00): a pure function of the
   details and the ruleset, applied as hero changes (point 2) when a week's or the start's fill
   offers the hero: each backpack artifact, from the last place to the first, goes to the first
   free position that takes it (ADR-0013 point 14.1's locks, `first_position`), leaving the
   backpack; one no free position takes stays. A combination equipped so applies its effects as
   any equip (`gives_spell_book`); none is assembled. A replacement and a returning hero are not
   equipped (the original's `tavern_slot_replace` 0x4C86A0 and `hero_leave_game` 0x4D9EC0 do not).

## Point 16, a sentence after 16.4

**E09 increment 4:** a hero off the map regains no spell point by the day (TD-20, E09 decision
104): its spell points are set full (HER-MAN-001's maximum) only when a fill offers it, when it is
drawn as a replacement and when it leaves the game without returning (ADR-0017 point 11); a
returning hero keeps its own. Its day's points come from point 13's derivation over the army the
game keeps for it (ADR-0017 point 3).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 4's `shared/` task): points 14.5 and 16 (a tavern hero's backpack
  equipped at a fill, its spell points off the map).
