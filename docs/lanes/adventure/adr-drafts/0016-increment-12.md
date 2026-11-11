# ADR-0016 amendment (draft): increment 12, garrisons and the bonus visitables (the map side)

- Status: **Accepted** at T108b's checkpoint (ADV-Q50, 2026-11-11; E08 decisions 305–309, the
  visit marks on the hero on the map by decisions 306 and 307), to be applied by T112. Drafted by
  E08 T108b (2026-11-11) from the survey `docs/rules/surveys/E08-garrisons-bonuses.md` and the
  user's answers at ADV-Q48 and ADV-Q49 (E08 decisions 295, 299–303), then into ADR-0016 through
  the round's `shared/` pull request (T112) with batch E12's outcome.
- Amends: ADR-0016 points 1, 3, 4, 5 and 6 (each text below goes in after the point's increment 11
  part, and after E09's where a point has one); its consequences, alternatives and revision
  history.
- Detail: `docs/epics/E08-design.md` parts A and B (§1 to §11), and part D §13 for the exchange's
  owner rule. The game's side (the exchange as a decision, the battles, the day and the week, the
  views) is `0017-increment-12.md`; the hero's marks are `0013-increment-12.md`.
- ADR-0015 needs no amendment: no crate, and the import still draws nothing.

## Context

Increment 12 types the garrisons (33 horizontal, 219 vertical, which the original loads as 33;
subclass 1 anti-magic: 238 objects on 61 of the 160 shipped maps) and the fifteen bonus visitables
(4 085 on 159), inert blockers until now. A garrison is an owned army in a passage: its team's
heroes cross it and exchange troops with it, any other hero fights it or takes it empty. The bonus
visitables give a hero morale and luck until its next battle, movement, spell points, or (the
Sirens) experience for a part of its army, once per hero for a lifetime each: a battle, a day, a
week, or per entrance a week. The user chose the original in every dispute (D-59 to D-64), a flag
for A-44 and the fix for A-45, the kind marks in this increment (E08 decisions 299 to 302).

## Decision

**Point 1** (storage), a part **Increment 12's objects** (E08 decision 295):

- **A garrison's army** is written on its object when the game is built, in the table of the
  armies on objects (as a town's garrison), its **owner** in the ownable objects' block, its
  **removable flag** static (the object's own: a capture keeps it), so the static map's. No new
  block for garrisons.
- **The Fountains of Fortune's values**, one per fountain in the static map's fountains' order, a
  block drawn when the game is built and at each week's step; none for a map without one.
- **The Magic Springs' entrances**, a "full" bit per entrance (each Spring's two), a block that
  stands only while some entrance is emptied: none when the game is built and after each week's
  step (as increment 10's dwellings' week). By entrance, not the original's counter mod 32.
- **The kind marks** (E08 decision 302): in the per-player block of object types (beside the
  Cartographer types bought and the keys), the types whose kind a team's visit marked, each with
  its player bits, set for every player of the visitor's team. None until a first mark.
- **Every visit's effect is the hero on the map's** (point 3; E08 decision 307): its battle
  bonuses and its day's and week's marks, not the hero's details (ADR-0013 point 1 as amended):
  none carries over to a hero that leaves the map and is hired again.
- **The day of the week** is the game's calendar's: an input of the adventure's command
  application (ADR-0017 point 1), never copied into the adventure's state.

**Point 3** (heroes on the map), after increment 11's text: from increment 12 a visit may raise a
hero's points left **and** its day's maximum together (the movement objects, the Stables' first
visit of a week), so a later boarding or landing converts by the raised maximum, or set the points
left to 0 and keep the maximum (the Swan Pond). A hero on the map also holds **its battle bonuses**
(moved from the details, E08 decisions 306, 307): the morale and the luck no once-mark names (a
Pandora's Box's, an event's), each held saturating, and **a short list of once-marks**, each the
mark (a `u8` the content names) and the morale and luck its visit gave, sorted by mark and unique,
with no fixed count; a visit adds its pair only while its mark is not listed. They last until the
end of the hero's next battle, which clears them but the marks the part names as outliving a
battle under the profile (the Sirens' with `ADV-QUIRK-46` off: A-44, decision 308). And **its
timed marks**: the object types whose day's or week's mark it holds (the Magic Well's, the
Stables'), each kind naming its lifetime, dropped at the next day's or week's start; the week
marks' kinds' points are added to its day's land points (`h3-heroes`' input, ADR-0013 point 13).
Each is dropped when the hero leaves the map. The army commands gain **one owner rule** for the
armies they name (E08 decision 299, D-61): each army has a **bar owner**: a hero's army its owner,
a map garrison's (the reference `MapGarrison` by its object) its owner when removable and none
when not, a town's garrison its town's owner (E09's), an army dialog's group its owner. A command
takes a stack only from a bar of the acting player; into a bar of another owner, none included, it
only moves or splits a stack into an **empty** slot, never merges into, swaps with or dismisses
that bar's stacks; a hero's last stack never leaves its army; a stationary army may be emptied.
Each refusal is typed and changes nothing. The Sirens' visit lowers stacks and never empties a
slot; the Stables' changes a slot's creature, its count kept.

**Point 4** (kinds), a part **Increment 12's kinds** (the import maps 33 and 219 by subclass and
the fifteen bonus classes; none of the fifteen visitable from above):

- **The garrison** (`garrison { anti_magic }`; ADV-GAR-001): ownable, entered, visitable from above
  (the original's class list: a horizontal one entered from above and below, a vertical one from
  the sides). The import reads its owner (a colour the map does not play refused), its army (a
  count of 0 an empty slot; a **random stack refused**, `ImportError::RandomStack`) and its
  removable flag (RoE maps: removable). Its kinds 33.0 and 33.1 in the base pack, 219.0 and 219.1 in
  the AB pack.
- **Its visit** (ADV-GAR-002): a garrison of the visitor's team opens the **exchange** (ADR-0017
  point 6); another team's or a neutral one holding a creature is fought at once, **without a
  question**, its army as it stands, unsplit (its owner's side without a hero, or unowned guards);
  won, or holding nothing, it is **flagged** for the visitor's owner (the flag's reveal), its army
  empty, then the exchange (after a battle, once the winner's levels are answered); lost, its
  survivors stay. A defeated player's garrisons become neutral with their armies (E10's defeat).
- **The passage** (ADV-GAR-004, ADV-OBJ-003 amended; D-59 the original): a garrison of the mover's
  team is crossed by a path and a move **without a visit**, as a border gate by its key's holders
  (a tile like any other, on the way or at the end, the search going on from it in every
  direction); the one at the move's end is visited. Every other garrison is a destination. Every
  player's path as a human's (the computer's, E10's).
- **The anti-magic battle** (ADV-GAR-005; D-60 the original): a battle whose tile is an anti-magic
  garrison's entrance, its own or a hero's standing on it, is fought without magic (the battle's
  setup flag, ADR-0017 point 11).
- **The bonus kinds** (ADV-BON-001; all in the base pack; the Buoy, the Fountain of Fortune, the
  Idol and the Magic Well visited from a neighbour, the eleven others entered; the Buoy, the
  Mermaids and the Sirens on water, reached from a boat): **`bonus`** (ten classes: a once-mark,
  the morale and luck by the day of the week, the movement added to the points left and the day's
  maximum or the points drained, the kind mark), **`fountain_of_fortune`** (a once-mark and the
  luck values its weekly draw picks among, equal chances), **`magic_well`** (a day mark),
  **`magic_spring`** (per entrance), **`stables`** (a week mark, its points, the upgrade's two
  creatures), **`sirens`** (a once-mark, the kept ratio 7 / 10). The amounts are parameters (the
  original's immediates), as the Whirlpool's divisor.
- **Their visits** (ADV-BON-002 to -008): the hero's mark set gives nothing; else a `bonus` adds the
  day's morale and luck with its mark (the amounts kept on it), then its movement; a Fountain of
  Fortune its value's luck (its knowledge marked for the team at every visit); a Magic Well the
  spell points to the hero's maximum and the day mark, a full hero refused **unmarked**; a Magic
  Spring's entrance, full and the hero below twice its maximum, twice the maximum, the entrance
  emptied (D-62: each entrance apart); the Stables at the week's first visit their points now and
  to each day's land points until the week ends (the week mark with its points), and at **every**
  visit every slot of Cavaliers made Champions when the army holds one (D-63); the Sirens each
  stack above one creature **trunc(count × 0.7)** by the double product (`kernel::num`, D-64),
  in full width (A-45 fixed), the lost hit points as experience with the Learning factor, the mark
  either way, cleared at the next battle's end only under `ADV-QUIRK-46` (A-44, `fix-optional`, on
  in `faithful`). The kind mark is set where the original's handler sets it (on the gain path for
  most, on every visit for the Spring, also when full for the Well; increment 7's kinds as read,
  some before a refusal).
- **The draws:** the Fountains' values are drawn when the game is built (the setup stream, the
  first week's walk with the weekly objects, in object order) and at each week's step (the game's
  stream, the weekly pass, in object order), uniformly among the kind's values.
- `Revisit` visits an entered one again under its rules (no step, no points).

**Point 5** (fog), after increment 11's text: from increment 12 a garrison's flag reveals its
radius (the flag radius, 3) around its entrance for its new owner, and an owned garrison at the
start for its owner, as every ownable object's.

**Point 6** (information), after increment 11's text: from increment 12 every viewer that sees a
**garrison** sees its type and owner, and **its army at the viewer's level** (the town's garrison
levels: the owner's team or Visions at level 3 in reach of the entrance all, else by the viewer's
towns with a Tavern), the game's view (ADR-0017 point 2); its removable flag only the owner's
team.
Every viewer that sees a **bonus visitable** sees its type; a viewer's team's **kind marks** are
its own; a Fountain of Fortune's knowledge is by object, its **value never**; a Spring's
entrances' state never (a weekly object's contents never show). A hero's marks and their amounts
are its owner's (its own hero's view on the map). The exchange's moves, the movement gains, the Sirens' losses,
the Stables' upgrade and a refused visit are told to the visitor's owner alone; a garrison's
changed army to every viewer that sees it, at its level. `docs/06-security.md` lists it (the
round's `shared/` task).

**Consequences**, a line: increment 12 makes the seventeen classes typed objects (adventure ruleset
part v20, map import v22, adventure state hash v22: the heroes' bonuses and timed marks, the
fortunes, the springs, the kind marks), so
on 159 of the 160 shipped maps their entrances become destinations and the passages' garrisons
crossable; the army commands gain the owner rule the town's rows reuse; the adventure's command
application takes the day of the week.

**Alternatives considered**, a line: increment 12 (ADV-Q49, ADV-Q50): VCMI's passage through any
empty garrison with a visit on the way (D-59), its magic forbidden only against the garrison's own
army (D-60), its merge into an ally's stack (D-61), its one refill a week for a Spring (D-62), its
Stables giving nothing while the week's bonus holds (D-63), its ceil(count × 0.3) drowned (D-64);
the Sirens' revisit reproduced with no flag or always fixed (A-44); the kind marks left for E24; a
garrison's exchange as the Leave Guards dialog's copied group instead of a direct reference; the
Springs by the original's counter; one kind per bonus class instead of six; the visit marks
in the hero's details as the original's records keep them (a fixed set of 32 once-marks with their
amounts, the day's and the week's marks beside them).

**Revision history**, a line: points 1, 3, 4, 5 and 6 amended for E08 increment 12, garrisons and
the bonus visitables: a garrison's army, owner and static removable flag, the Fountains' values,
the Springs' entrances and the kind marks; the raised day's maximum, the battle bonuses (moved
from the details) and the timed marks on the hero on the map, the army commands' one owner rule; the kinds garrison, bonus, Fountain of Fortune, Magic Well, Magic Spring, Stables and
Sirens with their visits, the passage, the anti-magic battle, the import's refusal of a random
stack; a garrison's flag reveal; a garrison's army by the viewer's level, a Fountain's value and a
Spring's state hidden. Drafted in E08 T108b (survey `E08-garrisons-bonuses.md`); batch E12's
outcome folded in.
