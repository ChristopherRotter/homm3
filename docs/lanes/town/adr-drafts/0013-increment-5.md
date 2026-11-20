# Draft: ADR-0013 amended for E09 increment 5 (a town's spells and spell points)

- Status: **Draft**, written by E09 T43 (2026-11-20), for the user's acceptance at its checkpoint
  (`TWN-Q33`); then moved into `docs/adr/0013-heroes-in-battle.md` by the increment's `shared/`
  task.
- Decisions: the epic's 162, 177.
- Working detail: survey `docs/rules/surveys/E09-guild.md` §11, §14, §15; design note
  `docs/epics/E09-design.md` Part G §88, §92, §93.

## Point 15, a sentence after 15.3

**E09 increment 5:** a town's guild teaches by the same tests as the objects that teach
(`can_learn_spell`: a worn spell book, a level at most the learnable level of HER-SPL-005, not
known), the map's bans not tested; the spells it may teach are the town's part's (ADR-0028 point
2), the hero's side only the test and the `LearnSpell` changes.

## Point 16, a sentence after 16.3

**E09 increment 5:** a town with a Mage Guild raises its visitor's and garrison hero's points to
their maximum (16.2's maximum, HER-MAN-001) after the day's regain, never lowering them; the Mana
Vortex sets them to twice the maximum (the town part's factor), no ceiling (the reward records'
999, D-38, is theirs alone; as read, `E09-guild.md` §15). Both are the game's
`HeroChange::SetSpellPoints` (ADR-0017 points 3 and 10); `h3-heroes` gives the maximum.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 5's `shared/` task): points 15 and 16 (a guild's teaching by
  `can_learn_spell`; the town's refill and the Vortex's points).
