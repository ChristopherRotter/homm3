# Draft: ADR-0023 amended for E09 increment 3 (a garrison hero at a town's battle)

- Status: **Accepted** at T22b's checkpoint (`TWN-Q14`, 2026-11-11; E09 decisions 81, 83);
  drafted by E09 T22b; to be moved into `docs/adr/0023-sieges.md` by the `shared/` task after batch TWN-3 (T24).
- Decisions: the epic's 12 (T-5 faithful, in increment 3), 65, 79, 81, 83.
- Working detail: survey `docs/rules/surveys/E09-towns.md` §8.1, §9.1 (`town_visit` 0x4AB0EE to
  0x4AB338), `E09-growth.md` §10; design note `docs/epics/E09-design.md` Part E §53. The game's
  side is `0017-increment-3.md` point 11, the visit's `0016-increment-3.md` point 4.

## Point 1, after "`Some` for every battle at a town"

**But one** (E09 increment 3; survey `E09-towns.md` §9.1, the original's town argument 0): the
battle against a town's **visitor beside a garrison hero** is not a siege (`siege: None`): it is
fought on the gate tile's battlefield as any hero's battle, and the town is not taken by it.

## Point 3, after "are applied to its `BattleHero` before the seam"

**From E09 increment 3** the defending hero of a siege is the visitor (the garrison merged into
its army first) or, without one, **the garrison hero** with its own army; either gets the town
bonuses. The visitor beside a garrison hero (point 1) gets none.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T24 `shared/`; `TWN-Q14`): points 1 and 3 amended for E09 increment 3 (the
  garrison hero). Drafted in E09 T22b.
