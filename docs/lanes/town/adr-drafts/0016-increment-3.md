# Draft: ADR-0016 amended for E09 increment 3 (growth, recruitment and the army rows)

- Status: **Draft**, whole: growth's half by E09 T22a, the rows' and recruitment's by T22b
  (2026-11-11, the army reference as agreed with E08's increment 12, its decision 305), put to
  the user at T22b's checkpoint (`TWN-Q14`); moved into `docs/adr/0016-adventure-state.md` by
  the `shared/` task after batch TWN-3 (T24), after or before E08's T112, whichever merges
  second rebasing (both amend points 1, 3, 4 and 6).
- Decisions: the epic's 65, 67 and 73 (the army reference), 72 (TD-15 by the reading), 75, 78,
  79 and `TWN-Q14`'s; E08's 305 (the reference and the one owner rule, accepted for both lanes).
- Working detail: survey `docs/rules/surveys/E09-growth.md` §4, §5, §8 to §10, §15; design note
  `docs/epics/E09-design.md` Part E §44, §45, §49 to §53; E08's design note §12, §13
  (`docs/epics/E08-design.md`).

## Point 1, E09's towns: replaces "the setup's (its random stacks resolved when the game is built, ADR-0017 point 4) until a rule writes it"

the setup's (its random stacks resolved when the game is built, ADR-0017 point 4) until a rule
writes it: **from E09's increment 3** a neutral town's garrison week (the game writes the stacks
`h3-towns`' rule returns, through the town module's writer; TWN-GRO-006).

## Point 1, E09's towns, a new sentence after it

The game reads, for a town's growth and its Portal, **each dwelling's owner and its kind's
creatures** in object order (`StaticMap::dwellings`, `AdventureState::owner`; ADR-0028 point 6):
the external dwellings count by their first creature (the original's record +4,
`town_change_owner` 0x4C62B5), the Portal draws one of the owner's dwellings (classes 17 and
20; never a Refugee Camp), then one of its kind's creatures. No adventure state is added for
growth.

## Point 1, E09's towns, after growth's sentences

**From E09's increment 3** a town's record holds its **garrison hero**, if one stands: the whole
record of the hero on the map (point 3) moved into the town's (`Holds::Town`), its tile the
entrance's, unread while it stands there. The town's garrison is empty while a garrison hero
stands (the hero took it in, point 3); recruits and a neutral week never reach such a town's own
garrison.

## Point 3, after E08 increment 12's text (the heroes on the map, the army commands)

**E09's increment 3, the garrison hero** (TWN-ROW-002 to -005; survey `E09-growth.md` §9, §10):
a hero of the town's owner moved into a town's garrison **keeps its whole record** (its army,
its points and day's maximum, its formation and tactics flag, and from E08's increment 12 its
battle bonuses and timed marks: the original keeps the hero record whole, `town_visitor_to_garrison`
0x4B9FC0); the record leaves the heroes on the map for the town's (point 1), so it has no tile,
reach, meeting or visit, is not the town's visitor (TWN-VIS-002) and is not among a player's
heroes on the map (their most, the part's 8, counts only those). E08 decision 307's drop "when a
hero leaves the map" does not apply: the record moves, it is not dropped. Three moves, each the
town owner's in its turn (the adventure's town module checks the town, the game the turn):
**the visitor enters** the garrison when none stands (the town's garrison joins the hero's army
all or nothing, by `army_absorb` 0x44B620's steps: each garrison stack in slot order onto the
hero's stack of its creature, else into the first empty slot, else the hero's own two stacks of
one creature merged, the later into the earlier, and the slot tried again; refused, typed, when
none frees a slot, so when the two armies' creatures counted once exceed the slots);
**the garrison hero leaves** onto a free entrance, refused while the owner has its most heroes
on the map or its army holds no stack; **the two swap** (both the owner's), each keeping its
army. **The army references** (agreed with E08's increment 12; its decision 305): a town's
**upper bar** is `ArmyRef::Garrison(town)` without a garrison hero and `ArmyRef::Hero(the garrison
hero)` with one, its lower bar the visitor's `ArmyRef::Hero`; `ArmyRef::Hero` resolves to a
garrison hero's army too. **The one owner rule** (E08's) takes a town's garrison's bar owner as
its town's owner; a garrison hero's is its owner, so its last stack never leaves its army (the
original's town screen keeps it, 0x5C7436). `ArmyCommandError::Garrison` stays for a town's
garrison the game does not admit (ADR-0017 point 6). **The hire dialog's** source and target
gain a town's (E08 T95b built them to be extended): `HireSource::Town { town, index }`,
`HireSource::Portal { town }` and `HireTarget::Garrison(town)`, the town's upper bar's army; the
adventure gives the room of the target (the room rule ADV-ARM-007, ADV-HIR-002) and joins the
creatures into it, the counts being the town's (ADR-0028 point 6).

## Point 4, E09's towns, after "a town of **another team or none** that is **defended**"

**From E09's increment 3** a town with a **garrison hero** (survey `E09-towns.md` §9.1; TWN-SIE-001
amended): with a visitor too, the visit is **a battle against the visitor, not a siege** (the
gate tile's battlefield, no town bonuses; ADR-0023 point 1's amendment), the garrison not merged,
and the town is never taken by it (the attacker stays outside; a beaten visitor leaves the map);
without a visitor, a **siege against the garrison hero** with its own army; a win captures the
town and sends the beaten garrison hero to the pool. The garrison's merge into a lone visitor's
army (TD-7, TWN-SIE-002) happens only without a garrison hero. **T-5** (`TWN-QUIRK-11`, the
original's, decision 12): a garrison hero whose army is empty (a defence won by summoned
creatures alone) does not defend: the first visitor of another team enters and captures the town
without a battle, the garrison hero back to the pool with its artifacts, no experience given.

## Point 6, E09's towns, a new sentence

**From E09's increment 3** a town's view at the level that shows its garrison's army (TWN-VEW-001,
level 3) shows a garrison hero and its army as the garrison; its owner sees its garrison heroes
among its own heroes, each with its town in place of a tile, and nobody sees one on the map.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T24 `shared/`): points 1, 3, 4 and 6 amended for E09 increment 3 (growth's
  inputs, the garrison hero, the army references, the hire dialog's town sources, the visit of a
  town with a garrison hero). Drafted in E09 T22a and T22b.
