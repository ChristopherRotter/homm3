# E09 growth survey (E09 T21): the week's growth, recruitment at a town, the town screen's two rows

Date: 2026-11-11. Scope: E09 increment 3 (growth, recruitment and the army rows; the epic's
decisions 65 and 66), input for T22's ADR amendment drafts, the design note's Part E and the cut.
Kept whole at its start (the user, decision 68; the agent's recommendation was two parts). Input,
not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with its
sources and tests. Building ids are the original's (survey `E09-construction.md`, its head): 7 to
9 the Fort, Citadel, Castle, 18, 19, 24, 25 the hordes and their upgrades, 22 a faction special
(Dungeon's Portal of Summoning), 26 the Grail's, 30 to 36 the dwellings by level, 37 to 43 their
upgrades. A town's **index** i is its creature's place among the faction's fourteen
(`town_creatures` 0x6747B4): 0 to 6 the levels' base creatures, 7 to 13 the upgrades; its
**available creatures** are the town's i16 counts +0x16 by index (`structs.toml` "town").

**Sources** (retrieved 2026-11-11):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path (the new day, the town screen) and names the computer's where it differs. **Read
  whole:** `town_dwelling_growth` 0x5BFB60, `town_legion_growth` 0x5BF900, `town_week_growth`
  0x5BFDD0, `neutral_town_garrison_week` 0x4BF570, `portal_of_summoning_week` 0x5BD750,
  `town_portal_hire` 0x5D2950, `recruit_dialog_init_town` 0x551560, `fort_dialog_hire` 0x5DCE50,
  `town_hero_hover` 0x5C7250, `town_hero_in_click` 0x5D5150, `town_hero_out_click` 0x5D5220,
  `army_absorb` 0x44B620, the growth panel's four helpers (`town_castle_growth` 0x5BF570,
  `town_horde_building` 0x5BF6D0, `town_horde_growth` 0x5BF770, `town_statue_growth` 0x5BF810),
  `town_garrison_army` 0x5C1460, `town_has_building` 0x4305A0 (H3API's `H3Town::IsBuildingBuilt`). **Read in part:**
  `new_day_week_events` 0x4C8780 (the week's kind, the towns' growth, the neutral towns, the
  Portal), `new_month` 0x4C8F40 (its towns), `new_day` 0x4C7FE0 (the order), `town_build`
  0x5BE930 (a dwelling's first creatures, again), `town_screen_proc` 0x5D3240 (its building clicks),
  `town_garrison_click` 0x5D4C10 (its modes 7 to 9), `town_garrison_hover` 0x5C7400 (the last
  stack), `town_creature_panel` 0x5C5B40 (its Portal), `town_init` 0x5BDE80 (the Portal's fields).
  **Tables dumped:** `neutral_garrison_level_weights` 0x6779B0, `horde1_hire_levels` 0x68A364,
  `horde2_hire_levels` 0x68A378, `month_creatures` 0x63E678 (again). **Not followed:** the
  computer's recruitment and hero moves (the callers the map's entries name), the screens'
  drawing, the messages' wording, network play, the tutorial's branches.
- Earlier readings this survey rests on: `E09-towns.md` §4.6 (no garrison hero at a new game),
  §5.2 (the neutral garrison at the start), §9 (who defends), §12.3 (the views), §16.2
  (`town_build`); `E09-construction.md` §8.4 (the events' creatures); `E08-dwellings.md` §5 (the
  hire dialog) and §6.1 (the map's dwellings ignore the week's kind); `E08-army.md` §1.1 (the army
  dialog's moves); `E08-garrisons-bonuses.md` §6 (the garrison exchange); `E08-economy.md` §7.3,
  §7.4 (the week and the month).
- The data's text tables, their numbers and names only (no text committed): `artraits.txt` (the
  Legion artifacts' ids and levels), `CRTRAITS.TXT` (names), `GENRLTXT.TXT` (which message a
  refusal shows).
- VCMI 1.7.5 (tag `1.7.5`, a shallow clone): `config/artifacts.json` (the Legion artifacts),
  `lib/mapObjects/CGTownInstance.cpp`
  (`getGrowthInfo`, `getDwellingBonus`, `getUpperArmy`), `server/processors/NewTurnProcessor.cpp`
  (`generateTownGrowth`, `updateNeutralTownGarrison`, `pickWeekType`), `server/CGameHandler.cpp`
  (`setPortalDwelling`, `recruitCreatures`, `garrisonSwap`), `lib/mapObjects/army/CCreatureSet.cpp`
  (`canBeMergedWith`), `client/windows/CCastleInterface.cpp` (the recruit's destination),
  `config/gameConfig.json` (the special weeks' chances), `config/creatures/*.json`
  (`hasDoubleWeek`).
- The wiki and others: the Heroes 3 wiki's growth page as the shoutwiki copy has it (search
  snippets only; a fetch is refused: https://heroes.shoutwiki.com/wiki/Growth), its dwelling page
  (https://heroes.thelazy.net/index.php/Dwelling), its Dungeon page
  (https://heroes.thelazy.net/index.php/Dungeon); the game's manual (the towns' chapter,
  https://issuu.com/ngutri.com/docs/heroes3_manual/s/13274881; page 88 of the RoE manual as the
  shoutwiki copy has it, https://heroes.shoutwiki.com/wiki/Restoration_of_Erathia_Manual_Page_88);
  VCMI's forum thread on the creature weeks (https://forum.vcmi.eu/t/creature-weeks-mechanics/329)
  and its bug 318 (https://bugs.vcmi.eu/print_bug_page.php?bug_id=318); the Might and Magic fandom
  wiki's growth page (https://mightandmagic.fandom.com/wiki/Growth).

## 0. Findings in short

1. **A dwelling's growth** (`town_dwelling_growth`, §1): 0 without its dwelling in the every-built
   set, and 0 at a base index whose upgraded dwelling stands (the growth goes to the upgrade's
   index). Else, in this order: the creature's growth (its record +0x44); **+ the castle's**: the
   growth again with the Castle standing, half (truncated) with the Citadel; for an owned town
   **+ the Statue of Legion's** (133 on any hero of the owner): (growth + castle's) ÷ 2, and **+ the
   Legion pieces'** (118 to 122 on the town's garrison hero or the hero on its entrance: +5, +4,
   +3, +2, +1 for levels 2 to 6, each hero apart); **+ one horde's** (the first of 18, 19, 24, 25
   ever built whose level is the index); **+ the external dwellings'** count (+1 a flagged
   dwelling of the creature, a base one counting for the upgrade too); **+ the Grail's**: half of
   the whole (truncated), with building 26 ever built.
2. **The week** (§2): on a week's first day (not the game's first), the week's kind first: 1 in 4
   a creature's (not when the ending week was a month's fourth), the creature drawn among every
   creature with a faction and a level (RoE's four elementals excluded on an RoE map); then any
   Inferno town with the Deity of Fire standing makes it the week of the Imp, whatever was drawn.
   Then every town's growth: each index's growth, **+5 for the week's creature** (that creature
   alone, not its other form), **+ the Imp's growth (15) for Imps and Familiars** in the Deity's
   week, **halved (truncated) in a neutral town**, added to the available creatures (16 bits, no
   cap).
3. **The month** (§3), after the week's growth: a draw of 1 to 10: 1 to 5 normal, 6 to 9 a
   creature's (one of twelve, `month_creatures`), 10 the Plague; normal in the tutorial; the Imp's
   after a Deity's week. **A creature's month** doubles the available creatures of each index whose
   creature is the month's (the exact creature: a base creature's month misses a town whose
   dwelling is upgraded) and whose growth is above 0 (not after a Deity's week). **The Plague**
   takes each index's growth back off its count (at least 0) and halves the rest: an owned town
   keeps half of what it had before the week; a neutral town less (TD-14).
4. **A neutral town's garrison grows** (§4): each week, 80 in 101 with a Fort, Citadel or Castle
   standing, else 40 in 101, one stack of a level drawn by three draws whose range widens with
   the weeks (up to 8), the faction's creature (the upgrade when the garrison holds it), as many
   as its growth, taken from the town's available creatures; without room it replaces the weakest
   stack weaker than it, else nothing; 5 in 100 then upgrades every stack of a base creature.
5. **The Portal of Summoning** (§5): each week, for each owned Dungeon with it, a dwelling drawn
   among the owner's flagged ones (17, 20), then one of its creatures: the Portal's creature, its
   count **set** to the creature's growth (not added; the Plague and the castle do not touch it).
   A Portal without a creature draws one **when the town screen shows it** or its button is
   pressed (T-11). Its hire goes into the town's garrison army.
6. **First creatures** (§6): a base dwelling built sets its level's count to the creature's growth
   alone (no modifier until the next week); its upgrade moves the base count to the upgrade's. The
   town events' creatures go to the upgraded dwelling, else the base one, else are lost (read by
   T12b; `TWN-STUB-10`).
7. **Recruitment** (§8): every path of the town screen (a dwelling, a horde, the Fort's creature
   list) opens one hire dialog over the town's index: the creature, and for an upgraded dwelling
   **both creatures from one count** (the upgrade first); **the army is always the garrison's**
   (the garrison hero's when one stands, else the town's own), never the visitor's; an ally's
   town can be viewed, not bought from. The purchase is E08's (`E08-dwellings.md` §5.3).
8. **The two rows** (§9): the upper bar the garrison army, the lower the visitor's; both the
   owner's, the army dialog's rules, **a hero's last stack kept in either row** (the garrison
   hero's too), the town's own garrison may be emptied. Three hero moves, a human's only: **the
   visitor into an empty garrison** (the town's army must fit into the hero's, merging by
   creature and the hero's own duplicates; else refused), **the garrison hero out** (refused with
   8 heroes on the map, or with no creatures), **the swap** (both of the town's owner). A visitor
   of another player moves nothing.
9. **What a garrison hero changes** (§10): the siege's defender (rows 1 and 3 of `E09-towns.md`
   §9.1), the views' marks (§12.3 there), the hire's army, the Legion pieces, the heroes' list and
   its limit of eight.
10. **Disputes** TD-13 to TD-17, **quirk candidates** T-10 to T-13, batch TWN-3's six rows, and
    the army reference proposed to the adventure lane (§13 to §15); the checkpoint's answers in
    §16 (two quirks `fix-optional`, the Portal drawn at its build and a dwelling flagged).

## 1. A dwelling's growth (`town_dwelling_growth` 0x5BFB60, `town_legion_growth` 0x5BF900, read whole)

`town_dwelling_growth(town, i)` (14 callers: the week, the month, the town screen's panel, the
computer's choices) returns an i16:

1. **No dwelling:** 0 when building 30 + i is not in the every-built set (+0x158); for i below 7
   also 0 when 37 + i is (the upgraded dwelling's count is index i + 7's).
2. **The base:** g = the creature's growth (its record +0x44; the creature `town_creatures`[faction
   × 14 + i]).
3. **The castle's** (over the standing set +0x150): the Castle (9) adds g, else the Citadel (8)
   adds g ÷ 2 (truncated). The Fort adds nothing.
4. **For an owned town only** (+0x1 not negative):
   - **the Statue of Legion** (133) worn by any hero of the owner (`player_hero_wears_artifact`
     0x4BACB0): + (g + the castle's) ÷ 2, truncated;
   - **the Legion pieces** (`town_legion_growth`): the level is i mod 7 + 1; for levels 2 to 6
     the piece 118 to 122 (Legs, Loins, Torso, Arms, Head of Legion; `artraits.txt`) gives +5, +4,
     +3, +2, +1 **for each of two heroes wearing it** (`hero_wears_artifact` 0x4D91F0, a yes or
     no: two of a piece on one hero count once): the garrison hero (+0xC) and the visitor (+0x10; when the
     field is −1, the hero standing on the town's entrance tile, class 34 there; the field is −1
     while the visitor is the selected hero, `structs.toml`). Levels 1 and 7 have none. A hero
     elsewhere gives nothing.
5. **One horde:** the first of the buildings 18, 19, 24, 25 (`horde_buildings` 0x642E20) ever
   built (+0x158) whose record's level (`town_hordes` 0x6887A0 +6, the index of its creature,
   E09 T5b1) is i adds its growth (+4); the loop stops at the first. A horde's upgrade's record has
   the upgrade's index, so 19 counts at i + 7 (and `town_build` turns 18 into 19 when the dwelling
   is upgraded, `E09-towns.md` §16.2).
6. **The external dwellings:** + the town's count at i (+0x118, `town_dwelling_count_add`
   0x5BFE50: a flagged dwelling of the creature counts 1 at its index, a base one at the upgrade's
   too; moved with the owner).
7. **The Grail** (26 ever built): + the sum so far ÷ 2, truncated (on the i16, so on everything
   above, the external dwellings included).

So, with g the base growth and c the castle's: owned growth = g + c + [Statue] (g + c) ÷ 2 +
Legion + horde + dwellings, then × 1.5 (truncated) with the Grail; a neutral town's has no Statue
and no Legion (its dwelling counts are 0, nobody flags for it).

## 2. The week (`new_day` 0x4C7FE0, `new_day_week_events` 0x4C8780, `town_week_growth` 0x5BFDD0)

**The order** (`new_day`): the day + 1; past day 7 the day is 1 and `new_day_week_events` runs
(which also counts the week up); past week 4 the week is 1 and `new_month` runs. So a month's
first day runs the week first, the month after it (§3). Day 1 of the game runs neither.

**The week's kind** (`new_day_week_events`' start; E08 T77a2 read it, here the towns' side):
1. Kind 0 (normal), the week's name index drawn (`random_between(0, 14)`, its name only), the
   week's creatures none, the bonus 5.
2. When the ending week is not the month's fourth (+0x1F640 not 4): `random_between(1, 4)` = 1
   makes it a creature's week (kind 1): the eligible creatures, counted from the last down, have a
   faction (record +0) and a level (+4) not negative, and on an RoE-format map (+0x1F698 0) are
   below 118 and not the elementals 112 to 115; with r = `crt_rand()` mod their count, the
   (r + 1)-th eligible creature from the top is the week's. Creatures with no faction (the
   neutrals) never come; upgrades can.
3. **The Deity of Fire:** any Inferno town (faction 3, any owner) with building 26 standing
   (+0x150) sets kind 2, creatures 42 and 43 (Imp, Familiar), the bonus the Imp's growth (+0x44 of
   record 42: 15), the week's creature 42; this runs every week, the month's fourth too, after
   (and over) the draw.

**Every town** (`town_week_growth(a, b, bonus)` with the two creatures, −1 none): for each index
0 to 13 with a growth above 0: + bonus when the index's creature is a or b; **halved when the
town has no owner** (truncated, after the bonus); added to the count (an i16 add, no cap, T-10).
Then the week + 1, the taverns, the map's dwellings (`dwelling_week_refill` 0x4B8A60, which reads
no week's kind, `E08-dwellings.md` §6.1), the map's weekly objects, **the neutral towns' garrisons**
(§4), the Portals (§5). Every town's byte +0x33 is set to 1 here (and by `town_init`; not read
further).

## 3. The month (`new_month` 0x4C8F40 at its towns)

After the month + 1 and its kind (`random_between(1, 10)`: the Deity's week gives a creature month
of the Imp; else 6 to 9 a creature's month (kind 1) with `month_creatures`[`random_between(0,
11)`]: Serpent Fly, Griffin, Centaur, Gremlin, Hobgoblin, Gog, Troglodyte, Harpy, Lizardman,
Pegasus, Wolf Rider, Wight; 10 the Plague (kind 2); 1 to 5, or any in the tutorial, normal with a
name index), every town, every index 0 to 14 (the fifteenth reads building 44's bit, never set:
growth 0, no effect) with a growth above 0:

- **a creature's month:** when the week was not the Deity's and the index's creature **is** the
  month's, the count doubles (a 16-bit shift). The week's growth is in the count already, so the
  doubled count is (before + the week's growth) × 2. A base creature's month misses every town
  whose dwelling of that level is upgraded (its base index has growth 0, its upgrade's creature is
  another); a month of the Hobgoblin (an upgrade) doubles only upgraded Goblin dwellings.
- **the Plague:** the count less the index's growth (`town_dwelling_growth` again: unhalved, no
  week's bonus), at least 0, then halved (truncated). For an owned town that is half the count
  before the week; for a neutral town (which got half the growth) max(0, before − ⌈g ÷ 2⌉) ÷ 2
  (TD-14, T-12).

The map's dwellings, the Portal (+0x3C, +0x40, outside the indexes) and the garrisons are not
touched by either. The creature month's monsters on the map and the Artifact Merchants follow
(E08's).

## 4. A neutral town (`new_day_week_events` at its neutral towns; `neutral_town_garrison_week` 0x4BF570, read whole)

- **Its growth** is halved each week (§2); its dwellings are those of its start (`E09-towns.md`
  §5.3: both of the first two levels for a standard set; a custom set's).
- **Its garrison** (after the map's dwellings): for each town without an owner,
  `random_between(0, 100)` below 80 with a Fort, Citadel or Castle standing, else below 40, calls
  the garrison's week:
  1. n = min(days ÷ 7, 8), the days counted from the game's start ((month × 4 + week − 5) × 7 +
     day, the week already counted up); S = three draws of `random_between(0, n + 1)` summed.
  2. **The level:** the first of the weights 2, 3, 4, 5, 4, 3 (`neutral_garrison_level_weights`
     0x6779B0) that S does not exceed, S lessened by each passed: S ≤ 2 level 1, ≤ 5 level 2, ≤ 9
     level 3, ≤ 14 level 4, ≤ 18 level 5, ≤ 21 level 6, else level 7. The first week (n = 1, S ≤
     6) gives levels 1 to 3; level 7 needs S ≥ 22, so n ≥ 7 (the seventh week).
  3. **The creature:** the faction's base creature of the level, its upgrade when the garrison
     already holds the upgrade (`army_count_of_type` 0x44ADC0); the count its growth (+0x44).
  4. **Room:** `army_can_take` (a stack of the creature or an empty slot); none: the stack whose
     AI value (record +0x40) × count is the least and below the new stack's (the first of equal
     ones) is cleared (`army_clear_slot` 0x44AB60), and none such: nothing more happens. Then
     `army_add_stack` (merged into a stack of the creature, else the first empty slot).
  5. **The town's counts** at the level's base and upgraded indexes each lose the count (at
     least 0): the garrison's growth comes out of the available creatures.
  6. For a base creature, `random_between(1, 100)` at most 5 turns **every** stack of it in the
     garrison into the upgrade.

The draws come from the one stream (`random_between`), after the towns' growth and the map's
weekly objects. An owned town's garrison never grows.

## 5. The Portal of Summoning (`portal_of_summoning_week` 0x5BD750, `town_portal_hire` 0x5D2950, read whole)

- **The town's fields:** +0x3C the Portal's creature (−1 none, `town_init` 0x5BDE80), +0x40 its
  count (i16); not touched by a change of owner (`town_change_owner` 0x4C61E0).
- **Each week** (`new_day_week_events`' last loop): for each player not out of the game, each of
  its towns, a Dungeon (faction 5) with building 22 standing: the map's dwellings (the game's
  +0x4E39C vector: classes 17 and 20; not the Refugee Camps) whose owner (+0x57) is the town's
  are listed; none: the creature −1. Else one is drawn (`random_between(0, n − 1)`) and one of its
  (up to four) creatures (`crt_rand()` mod their number): the creature, **the count set to its
  growth** (+0x44: what was not hired is gone, nothing accumulates). A drawn dwelling without a
  creature changes nothing (the last week's stays).
- **At the screen:** the town screen's creature panel (`town_creature_panel` 0x5C5B40, from
  `H3TownManager::RefreshScreen`, 0x5C6147) and the Portal's hire (`town_portal_hire`) call the
  same draw when the Portal stands and its creature is −1: a Portal built or left empty draws its
  first creature the moment its owner's town screen shows it (T-11). The hire: none still, a
  message; else the hire dialog (`recruit_dialog_init` 0x551350 with the Portal's flag, the
  creature and its count) over **the town's garrison army** (`town_garrison_army` 0x5C1460).
- **Not touched by** the castle, the Grail, the week's or the month's kind, the Plague (§3).

## 6. A dwelling's first creatures and the town events' creatures

- **Built** (`town_build` 0x5BE930 at 0x5BEB7E to 0x5BEBE5; `E09-towns.md` §16.2): a base dwelling
  (30 to 36) sets its index's count to the creature's growth (+0x44) alone: no castle, horde,
  Legion or Grail until the next week (the wiki's dwelling page says the same). Its upgrade (37 to
  43) moves the base count to the upgrade's index (the upgrade's growth first written, then
  overwritten) and zeroes the base. A horde built adds nothing at once. The setup's buildings are
  built the same way, in id order (`E09-towns.md` §5.3), so a town starts with each built
  dwelling's base growth; VCMI's first day agrees (`generateTownGrowth`'s `firstDay`).
- **The town events' creatures** (`E09-construction.md` §8.4, T12b): per level, to the upgraded
  dwelling's index when it is ever built, else the base's, else lost; no cap, after the event's
  buildings. 56 shipped events add creatures (§12); `TWN-STUB-10` until this increment.

## 7. The growth shown (`town_creature_panel` 0x5C5B40, in part; four helpers, read whole)

The town screen's creature panel (from `H3TownManager::RefreshScreen` 0x5D5410, through one
more call) calls the growth (0x5C5C4B) and lists its parts through helpers that repeat §1's rules:
`town_castle_growth` 0x5BF570 (a creature's castle's part), `town_horde_building` 0x5BF6D0 (the
level's horde building, 44 none), `town_horde_growth` 0x5BF770 (its growth; the last match, where
the growth takes the first: no faction has two hordes on one level), `town_statue_growth` 0x5BF810
((g + castle's) ÷ 2), and `town_legion_growth`. A display only; it adds nothing to the rules.

## 8. Recruitment (`town_screen_proc` 0x5D3240 at its building clicks; `recruit_dialog_init_town` 0x551560, read whole)

- **The paths** (the click's building id is the item's value − 1, `town_screen_proc` 0x5D34A5):
  a dwelling 30 to 43 (0x5D3EC4) opens the hire of index id − 30; a horde 18 or 19 (0x5D34E1) and
  24 or 25 (0x5D3DF0) the hire of its level's index by the faction (`horde1_hire_levels` 0x68A364,
  `horde2_hire_levels` 0x68A378: per faction a pair, the horde's base index and its upgrade's,
  e.g. the Castle's 18 index 2, 19 index 9, Rampart's 24 index 4); the Fort's creature list
  (`fort_dialog_hire` 0x5DCE50) the hire of the row's index when its dwelling is ever built; the
  Portal (22 for a Dungeon, 0x5D3BDC) §5's. The other mouse path on a dwelling or a horde shows a
  held popup of the level's creature and count (0x551750; no change). After the hire the screen
  is refreshed (`H3TownManager::RefreshScreen` 0x5D5410 at 0x5D3E87 and 0x5D3F42), so §5's draw runs then.
- **The dialog** (`recruit_dialog_init_town(town, i, 1)`): the creature `town_creatures`[faction ×
  14 + i] with **the index's count** (+0x16 + 2i, lowered in place by a purchase); for an
  upgraded index (i ≥ 7) a second creature, the base one, **with the same count**: one count
  buys either (the upgrade selected first); **the army: `town_garrison_army`** (the garrison
  hero's army when +0xC holds one, else the town's +0xE0); the flag +0x9C set as for the Portal,
  so a full army shows the garrison's no-room message (GENRLTXT 18), not the hero's; **+0x4C
  set when the town's owner is not the current player** (an ally's town: viewed, nothing
  bought, `E08-dwellings.md` §5.2).
- **The purchase** is the hire dialog's (`E08-dwellings.md` §5.2, §5.3; ADV-HIR-002): at most what
  the treasury pays and the count holds, into a stack of the creature or an empty slot of that
  army, else refused; the dialog stays open for more while it offers two creatures. **The visitor
  never receives** a town's recruits (the manual: recruits go to the town's garrison); moving them
  is the rows' work (§9). The env's four purchases (E08 T95b) fit: a town's index offers at most
  two creatures.

## 9. The town screen's two rows (`town_garrison_click` 0x5D4C10, `town_hero_hover` 0x5C7250, `town_hero_in_click` 0x5D5150, `town_hero_out_click` 0x5D5220, `army_absorb` 0x44B620)

- **The bars:** the upper is `town_garrison_army` (the garrison hero's army or the town's own),
  owned by the town's owner; the lower the visitor's army (+0x10's hero), the bar's +0x24 set. The
  stacks' moves are the army dialog's (`E08-army.md` §1.1; `town_garrison_hover` 0x5C7400; GAM-DEC-004:
  view, merge, swap, split, move, dismiss), between the bars or within one, the current player's
  bars only (`E08-garrisons-bonuses.md` §6). **The last stack** (0x5C7436 to 0x5C746D): a bar with
  one stack keeps it when the bar is the lower one, **or the upper one with a garrison hero**: no
  hero's army is emptied; the town's own garrison (no garrison hero) may be.
- **The hero moves** (the portraits; `town_hero_hover` sets the mode, `town_garrison_click`'s
  cases 7 to 9 run it, each for a human current player only, `player_is_human_both` 0x4BAD80):
  - **7, the visitor into the garrison** (the lower portrait selected, the upper empty; the
    visitor's owner the town's): `town_visitor_to_garrison` 0x4B9FC0 with `army_absorb`: the
    town's garrison army joins the hero's in a copy: per garrison slot in order, onto the hero's
    stack of its creature, else into the first empty slot, else the hero's own two stacks of one
    creature are merged (the later into the earlier) and the slot tried again; when none frees a
    slot the whole move is refused (GENRLTXT 276, nothing changed); else the copies are written
    (the town's army empty) and the hero becomes the garrison hero (+0xC; +0x10 −1; off the map
    and out of the player's list, the selection cleared). The test is VCMI's (`canBeMergedWith`):
    the two armies' creatures, counted once each, fit in seven slots.
  - **8, the garrison hero out** (the upper portrait selected, no visitor): refused when the
    owner has 8 heroes on the map (the player record +1; GENRLTXT 19) or the garrison army has no
    stack (GENRLTXT 20; unreachable through the screen, which keeps a hero's last stack); else
    `town_garrison_to_visitor` 0x5BE390: the visitor, put on the entrance. The town's own army
    stays empty (it was merged into the hero at move 7).
  - **9, the swap** (both heroes; from the upper portrait both of one owner, from the lower the
    visitor the town owner's): `town_swap_heroes` 0x5BE450; each hero keeps its army.
  - **4** (a portrait on itself) opens that hero's screen; any other pairing offers nothing. A
    visitor of another player (an ally's hero at the town, E10) moves nothing.
- **Not reached at a new game** (`E09-towns.md` §4.6): a game's garrison heroes come only from
  move 7 (and the computer's own calls).

## 10. What a garrison hero changes

- **The siege** (`E09-towns.md` §9.1, built for its rows 2 and 4 by increment 1): a garrison hero
  alone defends with its army (row 3); with a visitor too the battle is the visitor's, at the gate
  and not a siege (row 1), the town taken only at a later visit. `town_defended` 0x5BDFE0: the
  garrison hero's army stands for the town's. After a siege won the garrison hero is −1
  (`structs.toml` +0xC).
- **The views** (`E09-towns.md` §12.3, TWN-VEW-001): the quick view at level 3 shows the
  garrison hero and its army as the town's; the status window marks a garrison hero
  (`status_town_create` 0x4521F0's `townqkgh.pcx`).
- **The rules above:** the hire's army (§8), the Legion pieces (§1), the heroes' list and its
  eight (§9), the hero's spells at the town's guild (`town_heroes_learn_spells` 0x5BE030 reads
  both heroes; increment 5).

## 11. VCMI 1.7.5 and the wiki compared

| Fact | Reading | VCMI 1.7.5 | Wiki and others |
|---|---|---|---|
| The order of the parts (§1) | base, castle, Statue on base + castle, Legion, one horde, dwellings, Grail on all | the same (`getGrowthInfo`: the horde its creature's `horde` value; Statue-like bonuses on base + castle; the Grail on the total) | the growth page: Citadel, Castle and the Statue multiply the base only, the Grail last; bug 318 says the same |
| The Legion pieces | +5 to +1 for levels 2 to 6, a hero in the town (garrison or visitor) | the same (`config/artifacts.json`: `CREATURE_GROWTH` by level, propagated to the hero's town; the Statue's `CREATURE_GROWTH_PERCENT` 50 to the player) | `artraits.txt`: equipped by a hero in a town |
| External dwellings | +1 a flagged one of the creature, a base one for the upgrade too | +1 a dwelling providing a creature of the level (`getDwellingBonus`) | the dwelling page: +1, base and upgrade |
| The week's creature | +5, the exact creature; 25 in 100, not the month's first week | the same (`BONUS_GROWTH` on `cre->getId()`, 25, `CREATURES_ADDITIONAL_WEEKLY_GROWTH_SPECIAL_WEEK` 5) | the growth page: the base type only (Gorgons not Mighty Gorgons) |
| The Deity of Fire | Imps and Familiars + 15 every week | the level holding the Imp + the bonus | the growth page: every week and month the Imp's, +15 |
| Neutral towns | half the growth (after the week's bonus) | the same | — |
| **A creature's month** | 40 in 100; ×2 of (count + week's growth), **the exact creature only** | 40; ×2 for the **level** holding the creature (base or upgrade) | the growth page: the available creatures doubled, this week's growth included; VCMI's forum: an upgraded dwelling gets no bonus for its base creature's week or month (**TD-13**) |
| **The Plague** | 10 in 100; count − growth, ≥ 0, halved: owned half, neutral less | 10; the count halved, no growth | the fandom page: no growth and half the population (**TD-14** for neutral towns) |
| **A neutral garrison's week** | §4 | 80/40 in 100 by a fort; the level the mean of three draws 0 to the week + 1, clamped; a stack of the level grows or a free slot; never a replacement; 5 in 100 that stack upgraded (VCMI's comment: observed, may not match) | VCMI's bug 448 (observed): about 90 in 100 a week, levels 1 to 4 the first month, 1 to 6 the second, 1 to 7 the third (**TD-15**) |
| **The Portal** | a dwelling, then its creature; count set to the growth; no Plague; drawn at the screen when none | a creature among the set of the owner's dwellings' creatures; count the growth, the Plague halves it (at least 1); drawn at the week, the build, a capture, a dwelling flagged | the RoE manual: day 1 of each week a random creature type of a flagged dwelling; VCMI's forum: the Plague leaves it (**TD-16**, T-11) |
| Recruitment's army | the garrison army only | the client's the same (`getUpperArmy`); the server also takes the town's garrison hero or visitor (`recruitCreatures`) | the manual: into the town's garrison |
| Visitor into the garrison | refused unless the armies fit (types once each) | `canBeMergedWith`, the same | — |
| Garrison hero out | refused at 8 heroes (and without creatures) | the map's cap, no creature test | — |
| **An ally's visitor** | moves nothing (§9) | no owner test in `garrisonSwap` (**TD-17**) | — |

## 12. The census (the 160 maps; a throwaway function over the game `original_data` builds, seed as there, not committed)

- **Towns:** 1 468 (889 owned, 579 neutral at load).
- **Dwellings ever built at the start** (levels 1 to 7; the upgraded ones also in the base's
  count): owned 736, 269, 65, 14, 4, 4, 4 (upgraded 48, 27, 24, 10, 4, 4, 4); neutral 483, 452,
  91, 56, 42, 23, 33 (upgraded 36, 37, 30, 24, 25, 16, 18). Towns without a dwelling: 152 owned,
  66 neutral; with five or more: 70.
- **The castle:** owned 92 none, 779 a Fort, 8 a Citadel, 10 a Castle; neutral 216 none (the
  garrison's 40 in 101), 310 a Fort, 18 a Citadel, 35 a Castle (80 in 101).
- **Hordes built:** 43 owned, 57 neutral. **Grail buildings built:** 10, one of them a Deity of
  Fire (that map's every week the Imp's from week 2). **Portals built:** 1 (neutral).
- **Garrisons at the start:** 428 neutral and 19 owned towns hold one (`E09-towns.md` §5.2's
  draws and custom ones); 13 are full.
- **Heroes in towns:** 23 visitors at the start, no garrison hero (`E09-towns.md` §4.6).
- **The town events' creatures:** 56 events (`E09-construction.md` §10; 59 firings on the shipped
  maps in their first days, T19c2's census).
- **Legion artifacts:** 34 placed on the maps (118 to 122, 133), none on a placed hero.
- **External dwellings owned at the start:** 15.

## 13. Disputes, quirk candidates and open points

**Disputes** (two sources or more against one; the checkpoint decides):
- **TD-13 — a creature's month on an upgraded dwelling.** The reading, the wiki's growth page
  (the week: the base type only) and VCMI's forum: the exact creature (a Griffin month misses
  Royal Griffins). VCMI 1.7.5 doubles the level. Impact: every creature month with an upgraded
  dwelling of the month's base creature (most shipped maps' mid-game). Recommended: the reading.
- **TD-14 — the Plague in a neutral town.** The reading: max(0, count before the week − ⌈g ÷ 2⌉) ÷
  2; VCMI and the fandom page: half the count. Owned towns agree. Recommended: the reading (it is
  the original's; T-12 asks whether it is a bug).
- **TD-15 — a neutral garrison's week.** The reading's §4; VCMI's is its own approximation (its
  comment), VCMI's bug 448 an observation that the reading's ranges fit (the first month: n ≤ 4,
  levels 1 to 4 in practice). Recommended: the reading.
- **TD-16 — the Portal's draw.** The reading: a dwelling, then one of its creatures (a dwelling
  with four creatures splits its share), the count set, no Plague; VCMI: a creature from the set
  (each once), the Plague halves. Recommended: the reading.
- **TD-17 — an ally's visitor in the rows.** The reading: no hero move; VCMI: no test. Only with
  allies (E10). Recommended: the reading.

**Quirk candidates** (`T-<n>`; the ledger after the checkpoint):
- **T-10 — the 16-bit counts.** A town's available creatures are i16, added to (§2) and doubled
  (§3) without a cap: past 32 767 they wrap negative (state-corrupting, so fixed by proposal: a
  count saturates). Reachable only in very long games (a neutral town's level 1 with a Castle
  grows 14 a week).
- **T-11 — the Portal's first creature drawn by a view.** §5: the draw happens when the owner's
  town screen shows a Portal without a creature (or its button is pressed), so when the first
  creature comes, and the stream's state after it, depend on what a human looks at; a computer
  owner's waits for the week. The engine has no screen: draw it when the Portal is built and when
  its owner flags a dwelling while it has none (VCMI's triggers, what a human sees at once), or
  only at the week. The checkpoint decides.
- **T-12 — the Plague's neutral subtraction.** §3, TD-14: the Plague subtracts the full growth
  from a count that received half of it. Harmless (a smaller neutral population for a month);
  accepted as the original's unless the checkpoint says otherwise.

**Open points:** the computer's recruitment, its hero moves and its use of the Portal (E10/E13);
what reads +0x33; whether a capture or a loss of the Portal (none: buildings stay) changes +0x3C
(nothing in `town_change_owner`); the screen's display of hordes and the Portal.

## 14. Batch TWN-3's rows (for T22's cut; run as TWN-1 and TWN-2, on the capture harness)

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| TWN-3-1 | TD-13 | a Castle with the upgraded Griffin Tower and one with the base, a month of the Griffin forced (the month's kind written before the month: 0x698834, 0x697748) | both towns' counts on the month's first day | only the base Griffin Tower doubles |
| TWN-3-2 | §1 | an owned town with a Castle, a horde, the Grail, two flagged external dwellings of one creature, the Statue on a hero elsewhere and Legs of Legion on the visitor | the counts at day 8 | each index as §1 computes it (the Statue on base + castle, the Grail on the whole) |
| TWN-3-3 | TD-14 | a neutral and an owned town with known counts, a Plague forced | the counts on the month's first day | owned: half of before; neutral: max(0, before − ⌈g ÷ 2⌉) ÷ 2 |
| TWN-3-4 | TD-15 | neutral towns with and without a Fort, several weeks | each garrison each week | §4's chances and levels, the replaced stack, the counts lowered |
| TWN-3-5 | §5, T-11 | a Dungeon with the Portal, two flagged dwellings, a Plague month | +0x3C, +0x40 each week and before and after the town screen opens | the count set (not added), no Plague, the first creature at the screen |
| TWN-3-6 | §8, §9 | a town with a garrison hero, an upgraded dwelling; a visitor whose army does not fit the garrison | where a purchase goes; the visitor's move into the garrison | into the garrison hero's army, one count for both creatures; the move refused |

**Not rows:** the order of §1's parts, the week's exact creature, the Deity, the external
dwellings, the first creatures (the reading, VCMI and the wiki agree); T-10 (fixed by proposal).
**What the harness lacks** (`docs/experiments/README.md`): writes of the week's and month's kinds
before a new week; reads of a town's counts (+0x16), its Portal (+0x3C, +0x40) and its garrison
hero (+0xC; read by TWN-1 already); the writer's towns have no garrison hero and no Portal's state.

## 15. The army reference with E08 increment 12 (decision 67)

The adventure lane's T108a1 read the map's garrison exchange (`E08-garrisons-bonuses.md` §6): the
same dialog (`garrison_dialog_create` 0x5CE830) and the same hover code as the town's rows (§9),
with the garrison's army and owner in the upper bar and no garrison hero. So **one exchange
rule** covers both: two bars, an owner each, the current player's bar moves, another owner's bar
only takes stacks into empty slots, a hero's last stack stays, a stationary army without a hero
may be emptied. The engine already names a town's garrison `ArmyRef::Garrison(TownId)` (TWN-SIE-002's
events; refused by every army command until this increment, `ArmyCommandError::Garrison`).
**Proposed** to the adventure lane (its inbox, this task): a town's upper bar is
`ArmyRef::Garrison(town)` without a garrison hero and `ArmyRef::Hero(the garrison hero)` with
one; a map garrison gets a sibling reference by its object (the adventure lane's call, with its
store's record), both checked by one owner rule; the exchange's commands are the army dialog's
own with the second bar named. The agreement lands in T22's and T108b's drafts.

## 16. The checkpoint (T21; `TWN-Q12`, the user, 2026-11-11)

Asked interactively (`AskUserQuestion`, one round of four, a recommendation each); the epic's
decisions 69 to 73:

- **TD-13** (decision 69; the user's own answer, "fix-optional flag"): the original's rule is a
  quirk, candidate **T-13**, class `fix-optional` (the quirks ledger's: a flag, its default per
  profile): faithful, a creature's month doubles only the index whose creature is the month's (as
  read, §3); with the flag, the level's count whichever of its two creatures is the month's
  (VCMI 1.7.5's rule). Row TWN-3-1 still observes the original.
- **TD-14 / T-12** (decision 70; the user's own answer, "fix-optional"): class `fix-optional`:
  faithful, a neutral town's Plague as read (max(0, before − ⌈g ÷ 2⌉) ÷ 2); with the flag, every
  town keeps half of its count before the week.
- **T-11** (decision 71, recommended): a Portal of Summoning without a creature draws one when it
  is built and when its owner flags a dwelling while it has none, besides each week's draw (§5);
  the original's draw at the screen is not modelled (the engine has no screen; a human sees the
  creature at once there). Not a quirk to register.
- **TD-15** (decision 72, recommended): the reading (§4); row TWN-3-4 observes the distribution.
- **The agent's calls** (decision 73, change freely): TD-16 and TD-17 as read; T-10 fixed by
  proposal (a count saturates); the army reference of §15 proposed in the adventure lane's
  inbox; batch TWN-3's rows (§14) for T22's cut.
