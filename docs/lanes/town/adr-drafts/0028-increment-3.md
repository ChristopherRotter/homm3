# Draft: ADR-0028 amended for E09 increment 3 (growth, recruitment and the army rows)

- Status: **Draft**, growth's half (E09 T22a, 2026-11-11); T22b adds recruitment's and the
  rows' half and puts the whole to the user at its checkpoint (`TWN-Q14`); the `shared/` task
  after batch TWN-3 moves it into `docs/adr/0028-town-context.md` (as increment 2's T15).
- Decisions: the epic's 65 (content), 69 to 73 (the survey's checkpoint, `TWN-Q12`), 74 to 77
  (`TWN-Q13`: the cut, the towns' stream, the Legion as hero effects, the week's kind in the
  game), 78 (the agent's calls of T22a).
- Working detail: the survey `docs/rules/surveys/E09-growth.md` (§1 to §7, §11 to §13), the
  design note `docs/epics/E09-design.md` Part E (§37 to §47). Each block below says where it
  goes in the ADR.

## Point 2, after increment 2's paragraph

**Increment 3** (growth; E09 decisions 65, 69 to 78) adds to the town's record **the available
creatures**, a count by the faction's **index** (0 to 6 the levels' creatures, 7 to 13 their
upgrades; the original's i16 counts +0x16), each at most 32 767 (T-10 fixed: a count saturates
where the original's wraps), and **the Portal of Summoning's** creature and count (+0x3C, +0x40;
none until one is drawn). Its rules, each a pure function of a town, the part, the core and plain
inputs the game gives:

- **A dwelling's growth** at an index (TWN-GRO-001; survey §1): none without the level's dwelling
  ever built, none at a base index whose upgraded dwelling is ever built; else the creature's
  growth (the core's), plus the castle's (a standing fort level 3 adds the growth again, level 2
  half of it, truncated); for an owned town plus the owner's share of the two (the Statue of
  Legion's half, truncated) and the Legion's amount at the index's level (the inputs); plus the
  first ever-built horde of the index (its creature's horde growth, the core's); plus the owner's
  dwellings of the index's creature (an input); then, with a building of role `grail` ever
  built, plus half of the sum, truncated.
- **The week's growth** (TWN-GRO-003; §2): each index whose growth is above 0 gains it, plus the
  week's bonus when its creature is one of the week's, halved (truncated) in a town without an
  owner, saturating.
- **The month** (TWN-GRO-004; §3): in a creature's month each index whose growth is above 0 and
  whose creature is the month's doubles its count (the exact creature, T-13 `fix-optional`: with
  its flag off, both indexes of the level holding it), not in a month whose first week is a
  `week_of` week; in the Plague each such index loses its growth (at least 0), then halves (T-12
  `fix-optional`: with its flag off, a neutral town loses the half growth it got).
- **A dwelling's first creatures** (TWN-GRO-005; §6): a dwelling built sets its index's count to
  its creature's growth; an upgraded dwelling built moves the base index's count to its own. The
  setup's, a command's and an event's builds alike.
- **A town event's creatures** (TWN-EVT-004 amended; §6): each level's count to the upgraded
  dwelling's index when it is ever built, else to the dwelling's, else none; saturating.
- **A neutral town's garrison week** (TWN-GRO-006; §4, TD-15 by the reading): with draws from the
  stream the game passes (the towns'), whether its garrison grows (the part's odds by a standing
  fort), the level (three draws, widening with the weeks), the creature (the level's, its upgrade
  when the garrison holds the upgrade), its room (a stack of it, an empty slot, else the stack of
  least AI value times count below the new one's, replaced; else nothing), the town's two counts
  of the level lowered by it, then a 5-in-100 upgrade of every stack of a base creature. It reads
  and returns the garrison as `h3_contracts::ArmyStack`s; the adventure still holds the army.
- **The Portal of Summoning** (TWN-GRO-007; §5, TD-16 by the reading, T-11 by decision 71): given
  the owner's dwellings' creatures in object order, one dwelling drawn, then one of its
  creatures; the count set to that creature's growth (not added; no castle, Grail, week, month or
  Plague); no dwelling: none. The same draw when the Portal is built and when its owner flags a
  dwelling while it has none.

Every derivation reads the record as the last write left it, so a build, an event and a week
change the counts at once.

## Point 5, after increment 2's paragraph

**Increment 3** gives buildings three roles: **`horde`** (a level and whether upgraded, as
`dwelling`: its growth is the core's `horde_growth` of that index's creature, the figures the
executable's `town_hordes` 0x6887A0 copies), **`portal`** (the Portal of Summoning) and
**`week_of`** (a level: while the building stands in any town, every week is the week of that
level's two creatures of its faction, the Deity of Fire's week of the Imp). Each town record
gains **`roe`** (whether its creatures may be the week's on an RoE map; false for the AB pack's
Conflux). The parameters gain the week's creature chance (1 in 4) and bonus (5) and a neutral
town's garrison week (the odds 40 and 80 in 101 by a standing fort, the level weights 2, 3, 4, 5,
4, 3, the weeks' cap 8, the upgrade's 5 in 100). The creatures' growths stay the core's. The
content: `horde` harvested from the executable's horde table, `portal` and `week_of` the
harvest's gap-fill roles (the Dungeon's and the Inferno's specials), the parameters tier-2 data of
the reading.

## Point 6, after increment 2's paragraph

**Increment 3's growth** crosses as plain inputs and the store's writes: the game gives each rule
whether the town is owned, the owner's share (`h3-heroes`' town growth of the owner's heroes,
ADR-0013 point 16.4), the Legion's amounts (the garrison hero's and the visitor's), the owner's
dwellings' creatures (the adventure's owners and kinds), the week's kind and the stream; it writes
the counts and the Portal into the store and the returned garrison into the adventure (ADR-0016
point 1) in one infallible application. No new contract type.

## Point 9, after increment 2's paragraph

**Increment 3's growth** bumps the town state hash (v3: the counts, the Portal) and the town part
(v5: the roles, `roe`, the parameters); `cross-lane`: the game state hash (the week's kind, the
towns' stream, the store), the protocol (the week's kind, the owner's town view), the adventure
replays re-recorded; `shared/`: the kernel's stream id, `h3-content`'s two hero effects and two
quirk flags (pack format v52).

## Consequences, a new item

- Increment 3: each week writes every town with a dwelling (a fork's first week copies most
  towns); the day's start computes each town's growth twice at a month's first day (the week and
  the month), measured against `day-start-builds-36`.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 T2x `shared/`; `TWN-Q12` to `TWN-Q14`, decisions 65 to 78 and T22b's): points
  2, 5, 6 and 9 amended for increment 3. Drafted in E09 T22a and T22b (design note Part E); batch
  TWN-3's verdicts (survey §14).
