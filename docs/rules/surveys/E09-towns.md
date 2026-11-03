# E09 towns survey (E09 T1): the census, the start, the town on the map

Date: 2026-11-02. Scope: E09 increment 1 (towns on the map; the epic's decision 5), input for
T2's ADR drafts, the design note and the cut. In three parts, T1a cut in two at its start (the
user, 2026-11-02): **T1a1** the census of the shipped maps and the players' faction choice with
the random towns (§0 to §3); **T1a2** the main town's generated hero and the pool it draws from,
the random hero, a hero placed on a town's blocked tile, the town's buildings at the start (§4 on,
appended); **T1b** the town on the map, cut in two at its start (the user, 2026-11-02): **T1b1**
the visit, the visiting hero, capture and the battle at a town with ADR-0023's facts (§7 to §10);
**T1b2**, cut in two at its start (the user, 2026-11-02): **T1b2a** the hall's income and the
Capitol, the views, batch TWN-1's rows (§11 to §13); **T1b2b** the checkpoint's `TWN-Q2` (§14);
later **T3c** batch TWN-1 scored (§15, 2026-11-03); **T5b1** the buildings' tables read, building
23 found, VCMI compared (§16, 2026-11-03; T5b cut in two at its start, the user).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-11-02):
- `original-data`: the 160 shipped maps (`maps.tar`) read with `h3_map::read` and imported with
  `h3_map::import` (a throwaway example, not committed); a code reading of the user's
  `Heroes3.exe` (SoD 4.0 Complete, SHA-256 `aac39e8c…`); every address cited has an entry in
  `docs/exe/map.toml` or `structs.toml`. **Read whole:** `random_allowed_faction` 0x5132D0,
  `count_allowed_factions` 0x5132B0, `faction_choice_offered` 0x58CE70. **Read in part:**
  `load_town_body` 0x5019F0 (the record's identifier, owner and type, the SoD alignment byte, the
  random town's faction; the name, garrison, buildings, spells and events not followed),
  `load_object_body` 0x502E00 at its switch (classes 77 and 98 reach `load_town_body`),
  `scenario_dialog_begin` 0x588330 at its loop over the colours (0x5885CD to 0x588729),
  `setup_towns` 0x4CAA70 at the random town's conversion and the names (0x4CAC33 to 0x4CAD40).
  **Not followed:** the scenario dialog's drawing and its arrows (it shows a colour's choice; what
  the server must check is in §2.2), network play (the host's copy of the choices), the campaign's
  and the random map generator's paths, the computer player.
- VCMI 1.7.5 (`lib/mapping/MapFormatH3M.cpp`, `lib/mapping/CMapHeader.cpp`,
  `lib/mapObjects/CGTownInstance.cpp`, `lib/gameState/CGameState.cpp`).
- The wiki: [Map Editor - Objects](https://heroes.thelazy.net/index.php/Map_Editor_-_Objects)
  ("Random Towns"), [Map Editor](https://heroes.thelazy.net/index.php/Map_Editor) ("Player
  Specs"), [Horn of the Abyss (Changelog)](https://heroes.thelazy.net/index.php/Horn_of_the_Abyss_(Changelog))
  (1.6.0, 1.3.0), through the search's summaries.
- **T1a2** (retrieved 2026-11-02): a census of the same maps (a throwaway crate over
  `h3_map::read`, not committed); the code reading, every address in `docs/exe/map.toml`.
  **Read whole:** `hero_pool_init` 0x4C4E30, `hero_draw_of_faction` 0x4BB400, `main_town_heroes`
  0x4CA040, `town_hero_place` 0x5C13B0, `hero_place` 0x4D7900, `hero_show` 0x4D75D0,
  `placed_heroes_setup` 0x4CAE10, `scenario_hero_lists` 0x583890, `game_reset` 0x4BF1A0,
  `town_setup` 0x5C0670, `town_buildings_setup` 0x5C08C0, `tavern_hero_draw` 0x4BB5E0 (for the
  weights' role only; increment 3's). **Read in part:** `new_game_heroes` 0x4BFE70 (its steps'
  order and the chosen heroes, 0x4BFF4A to 0x4C012F), `new_game_bans` 0x4C2450 (its start),
  `map_header_read` 0x4C4390 (the allowed heroes, 0x4C47E4 to 0x4C498C), `load_hero_body`
  0x5021C0 (the type, the random hero's draw and the owner byte, 0x5021FA to 0x502374 and
  0x50294D to 0x50298F), `new_game_start` 0x58C570 (the choices, 0x58C680 to 0x58C6CE),
  `town_build` 0x5BE930 (its first lines). **Not followed:** the scenario dialog's drawing (what
  it shows in the hero column), network play's copy of the choices (0x4EF611), the campaign's
  paths (carried heroes, 0x4CA1A7's experience), the tavern's weekly heroes
  (`tavern_new_heroes` 0x4C8450), the horde part of `town_build`, the building tables' start-up
  (0x4EB730), a town's change of owner (0x4C61E0, T1b's), the guild's spells (0x5BE030, 0x5BE600)
  and the Shipyard's spot (0x5C0C90), increment 4's.
- VCMI 1.7.5 for T1a2: `lib/gameState/CGameState.cpp` (`pickNextHeroType`,
  `pickUnusedHeroTypeRandomly`, `getUnusedAllowedHeroes`, `placeStartingHeroes`,
  `placeHeroesInTowns`, `initVisitingAndGarrisonedHeroes`, `initTowns`),
  `lib/mapObjects/CGHeroInstance.cpp` (`pickRandomObject`), `lib/mapObjects/CGTownInstance.cpp`
  (`initializeNeutralTownGarrison`), `lib/mapping/MapFormatH3M.cpp` (the town's buildings),
  `config/gameConfig.json` (`startingDwellingChances`).
- The wiki and others for T1a2: [Town](https://heroes.thelazy.net/index.php/Town) ("Neutral Town
  Garrison"); the HotA team's [documentation](https://h3hota.com/en/documentation) (its own towns'
  default buildings); a GOG forum thread on the starting hero
  ([link](https://www.gog.com/forum/heroes_of_might_and_magic_series/homm3_is_there_a_way_to_pick_your_initial_hero_in_the_very_beginning_of_sp_play))
  and a Celestial Heavens thread on "Generate hero at main town"
  ([link](https://www.celestialheavens.com/forum/7/14260)), through the search's summaries.
- **T1b1** (retrieved 2026-11-02): the code reading, every address in `docs/exe/map.toml`.
  **Read whole:** `town_visit` 0x4AAFD0 (named `hero_battle_on_map` before), `town_change_owner`
  0x4C61E0, `garrison_merge` 0x44B820, `capitol_limit` 0x5BEC60, `town_at_entrance` 0x4BB870,
  `town_remove_from_owner` 0x5BE2D0, `army_reset` 0x44AB00, `hero_loss_effect` 0x4ACBB0 (named
  `hero_off_map` before), `dwelling_creature_faction` 0x4C6690, `town_dwelling_count_add`
  0x5BFE50. **Read in part:** `hero_move_step` 0x4805E0 (its case 98, 0x480E3B to 0x480EDF),
  `visit_object` 0x4A84F0 (its case 98), `visit_hero` 0x4A2800 (a hero in a town),
  `adventure_battle` 0x4AD470 (the side setup's call, 0x4AE238 to 0x4AE27C, and the outcome,
  0x4AE431 to 0x4AE898), `hero_hide` 0x4D76E0 (the town's visitor, 0x4D776F to 0x4D77A5),
  `new_game_heroes` 0x4BFE70 (the game's version, 0x4BFEE9 to 0x4BFF4A), `new_game_start`
  0x58C570 (its version, 0x58C593 to 0x58C5B7), `scenario_version_from_slots` 0x58E700 and
  `scenario_version_highest` 0x58EA00 (their last loops). **Not followed:** the town screen
  (0x5BE210) and the computer's town visit (0x5253D0); the scenario's tests (0x5F2860, 0x5F2E40,
  0x5F2260, 0x5F22A0: E10's); network play (`town_owner_message` 0x49C190); the battle's own
  reading of the town (E04c's, `installation.md` §7.27 to §7.33); what reads the external
  dwellings' counts (increment 3); the masks the scenario dialog's version is taken from.
- **T1b2a** (retrieved 2026-11-02): the code reading, every address in `docs/exe/map.toml`.
  **Read whole:** `town_gold_income` 0x5BF600, `town_silo_income` 0x5C1280,
  `town_quick_view_create` 0x530120, `quick_view_army` 0x530990, `player_tavern_count`
  0x4CCE30, `hero_visions_level` 0x4E5DE0, `hero_visions_reach` 0x4E5E10. **Read in part:**
  `town_popup` 0x4167A0 (all but its debug branch, 0x4168D7 to 0x416E62), `tile_click_path`
  0x40A5D0 (its right-button branch), `player_gold_income` 0x4C7930 (its towns, again) and
  `player_daily_income` 0x4B8AF0 (its gold call). **Not followed:** `object_popup_text` 0x4137C0,
  the hero's, monster's and garrison's popups, the town list's right click, the Visions cast
  0x41C490 beyond its level, the silo's, Treasury's and Mystic Pond's payment (increment 4). The
  census of §11.4 and §12.4 (a throwaway crate over `h3_map::read`, not committed).
- VCMI 1.7.5 for T1b2a: `lib/mapObjects/CGTownInstance.cpp` (`dailyIncome`, `setOwner`),
  `config/buildingsLibrary.json`, `config/factions/*.json`, `lib/callback/CGameInfoCallback.cpp`
  (`getTownInfo`, `hasAccess`), `lib/gameState/InfoAboutArmy.cpp`,
  `lib/mapObjects/CGHeroInstance.cpp` (`hasVisions`), `config/creatures/neutral.json` (the
  Rogue), `config/spells/adventure.json` (Visions).
- The wiki for T1b2a: [Halls](https://heroes.thelazy.net/index.php/Halls),
  [Grail](https://heroes.thelazy.net/index.php/Grail),
  [Tavern](https://heroes.thelazy.net/index.php/Tavern),
  [Thieves' Guild](https://heroes.thelazy.net/index.php/Thieves'_Guild),
  [Visions](https://heroes.thelazy.net/index.php/Visions); the RoE manual on heroes3wog.net
  (the sword over a defended town), through the search's summaries.
- VCMI 1.7.5 for T1b1: `lib/mapObjects/CGTownInstance.cpp` (`onHeroVisit`,
  `mergeGarrisonOnSiege`, `battleFinished`, `onTownCaptured`, `setOwner`, `removeCapitols`,
  `armedGarrison`, `getSightRadius`), `lib/mapObjects/CGTownInstance.h` (`isBattleOutsideTown`),
  `server/CGameHandler.cpp` (`visitObjectOnTile`'s `startVisit`).
- Others for T1b1 (through the search's summaries): the wiki's
  [Tutorial](https://heroes.thelazy.net/index.php/Tutorial) (a garrison hero defends from within
  the fortifications); coyot's [HOMM3 guide](http://home.pinknet.cz/~coyot/homm3/gameplay.php)
  (a visitor beside a garrison hero is fought on free terrain; the garrison merged into a lone
  visitor's army, "what cannot be merged is unused"); a reader's comment on heroes3wog.net's
  [Did you know?](https://heroes3wog.net/did-you-know-second-edition/) (a garrisoned hero left
  without units by dismissing them in the town screen). The wiki's Town page was not reachable
  (not on the session's fetch list).

## 0. Findings in short (T1a1, T1a2, T1b1, T1b2a, T1b2b)

1. **Every shipped map has towns:** 1 468 on the 160 maps, 876 random towns (class 77, on 127
   maps) and 592 of a fixed faction (98); **no Conflux town is placed on any map** (98's subclass 8
   never occurs). 889 are owned at load, every one by a colour that plays; 579 are neutral. All
   twenty town templates share one footprint (§1.1).
2. **All 160 maps import once three things exist:** the hero generated at a main town (128 maps
   refused for it today), a hero standing in a town (3 maps) and **the random hero** (class 70: 21
   maps refused for it alone, 15 more behind a town refusal; 176 random heroes on 36 maps). The
   random hero is drawn from the same pool as the main town's hero, so it is a scope candidate for
   increment 1 (the checkpoint's `TWN-Q2`; T1a2 reads its draw beside the main town's).
3. **The header's main town is its entrance tile** (the town's anchor less two columns), as VCMI's
   loader converts it; all 553 main towns match a town of their own player; 538 generate a hero.
4. **The player's faction** (§2.2): the scenario dialog keeps one choice per colour, a faction or
   Random; when the game begins, Random becomes a faction drawn uniformly among the colour's
   allowed ones (the map's mask: eight bits in RoE, nine from AB on). The map's "random town" flag
   is read only by the dialog (whether it offers the choice) and changes no rule.
5. **A random town's faction** (§2.3; `load_town_body`, as the map's objects are read at the new
   game, in object order): a SoD alignment to a colour that plays gives that colour's faction;
   else an owned town takes its owner's; else (neutral, or aligned to a colour nobody plays) a
   uniform draw over **eight factions on an RoE map, nine (Conflux included) on an AB or SoD map**.
   The wiki agrees on each point; VCMI differs on two (TD-1, TD-2).
6. **Random stacks in town garrisons:** 12 stacks in 7 maps, all in random towns; the import
   keeps no garrison yet, so increment 1 resolves them by the town's faction or refuses them
   (T1a2 reads the draw; a random hero's army has the same kind, and HotA 1.3.0 fixed a SoD bug
   in it).
7. **Random dwellings tied to a town:** 1 397 on 87 maps (classes 216 and 217 by the town's
   identifier), **every one tied to a random town**: the random dwellings of E08's increment 10
   resolve after this lane's random towns (a note in the adventure lane's inbox).
8. **Disputes:** TD-1 (the alignment before the owner; no shipped map affected), TD-2 (Conflux on
   RoE maps). **Quirk candidate:** T-1 (an alignment byte from 8 to 254 reads past the eight
   colours' records; none on the shipped maps). Content for the harvest: none from T1a1 (factions
   are core content; the towns' name lists are text).

**T1a2 (§4 to §6):**

9. **The pool** (`hero_pool_init` 0x4C4E30, right after the map's header): a hero type is free
   when the map allows it, else out of the game; SoD's disposed heroes narrow who may take one (a
   colour mask per hero). An RoE map allows its 128 bits and **the 16 Conflux heroes besides**,
   never the 12 campaign heroes. On the shipped maps **no placed hero or prisoner is in its map's
   allowed set** (108 heroes, 89 prisoners).
10. **One draw for both starting heroes** (`hero_draw_of_faction` 0x4BB400): uniform over the free
    heroes of the colour's faction's two classes that the colour may take; with none, uniform over
    every free hero it may take. **No tavern weights:** they are the tavern's draw's (0x4BB5E0,
    increment 3), so increment 1 harvests none (the epic named them).
11. **The random hero** (class 70: 176 on 36 maps) is drawn while the map's objects are read, in
    the file's order, from its owner's faction; **a colour's starting-hero choice goes to its first
    random hero**. Every owner of one has the map's random-hero flag (140 players) and no other
    player has it.
12. **The main town's hero** (`main_town_heroes` 0x4CA040, after the placed heroes): per colour in
    colour order, with the generate flag: a human's chosen hero, else the same draw; it stands on
    the town's entrance as the **visiting hero** (`hero_show` 0x4D75D0 makes every hero on a
    town's entrance its visitor) and learns the guild's spells at once (increment 4).
13. **A hero in a town** (`placed_heroes_setup` 0x4CAE10): a hero (not a prisoner) whose tile is
    right of a town's entrance moves onto the entrance. All 23 on the shipped maps are such, in
    towns of their own owners, none in a main town that generates a hero.
14. **The buildings at the start** (§5.3): every town a Village Hall. The standard set: a Fort
    with the map's flag; **an owned town a Tavern, and only with a Fort a level-1 dwelling and 30
    times in 100 a level-2 one; a neutral town both dwellings**, fort or not. A custom set: the
    map's built buildings with the ones they supersede, less the forbidden ones.
15. **The garrison** (§5.2): a custom one as the map gives it, its random stacks by the town's
    faction (level and upgrade); without one, an owned town's is empty and **a neutral town's is
    drawn**: four rolls for levels 1 to 4 (33, 33, 20, 13 in 100); the wiki agrees.
16. **Disputes** TD-3 (the standard buildings: VCMI gives every one a level-1 dwelling and half
    the time a level-2), TD-4 (level 4's chance, VCMI 14), TD-5 (a neutral town's explicitly empty
    garrison: VCMI draws one), TD-6 (a hero on a town's other blocked tiles). **Quirk candidates**
    T-2 (the starting-hero choice is not reserved: one hero can stand twice on the map), T-3 (a
    random hero can take the type of a hero placed later in the file), T-4 (the draws' empty
    cases). **Content:** none new (the creatures by faction and level are in the packs; the
    neutral garrison's table and the 30 are constants of the executable).

**T1b1 (§7 to §10):**

17. **A town is entered, never from above** (§7.1): a step onto the gate of an own or allied town,
    or of an undefended one, enters it; a town of another team or none that holds a visiting hero
    or any creature in its garrison is visited from the tile before (the step paid, the hero stays,
    as a monster). A hero standing on the gate is met first: its own team's hero meets it, another
    team's hero attacks the town.
18. **The visiting hero is the hero on the gate** (§7.3): `hero_show` sets it, `hero_hide` clears
    it. No map sets a garrison hero (the town screen's swap, increment 3), so increment 1's towns
    hold at most a visitor and the town's own army.
19. **Capture** (§8): an undefended town is entered, changes owner, and opens its screen; after a
    siege won the owner changes but **the attacker stays outside** and enters by a later step. A
    defender's win or a battle with no winner changes no owner.
20. **The change of owner** (§8.3): the garrison emptied, the town moved between the players'
    lists, the external dwellings' counts moved (increment 3's), radius 5 revealed around the
    gate (20 for a Tower with its Lookout Tower, every level for a Tower with the Grail), the
    scenario's tests; then **one Capitol per player** (§8.4): a captured Capitol becomes a City
    Hall when the captor holds another.
21. **Who defends** (§9.1): a visitor beside a garrison hero fights alone in the open (no siege;
    the town is not taken); a lone visitor defends a siege with **the garrison merged into its
    army** by AI value (§9.2; the rest stays out of the battle and is lost with the town); a
    garrison hero defends a siege; else the town's army. The defending side is the town's owner.
22. **ADR-0023's facts** come from the town's faction, its two building sets and the game's
    version (§9.4); the version's new-game source is read: the scenario dialog's version (1 AB,
    2 or 3 SoD, any other RoE), a campaign's by its index. Whether an RoE map's game plays under RoE rules
    in the Complete edition is batch TWN-1's row.
23. **Disputes** TD-7 (the merge's order and ties, VCMI's other); **quirk candidate** T-5 (a
    garrison hero without creatures: the town falls without a battle, the hero back in the
    pool); **content:** none new (the AI values are in the packs).

**T1b2a (§11 to §13):**

24. **A town's gold a day** (§11.1): 500, the Town Hall 1 000, the City Hall 2 000, the Capitol
    4 000 (the highest hall), **+5 000 with the Grail building**, the same in every faction; the
    Resource Silo adds no gold (its resources are paid beside, increment 4). Paid to the town's
    owner at each day's start from day 2 with E08's income; a neutral town pays nobody. VCMI and
    the wiki agree.
25. **One Capitol per player only after a capture** (§11.3): the start keeps every Capitol the map
    gives (none of the shipped maps gives a player two). The census (§11.4): 23 Capitols (10
    owned), the Grail building in 10 towns (2 owned), the silo in 26 (7 owned).
26. **What another player sees** (§12): the picture (faction, owner, a Capitol, any fort), the
    name and faction (the hint), whether a reachable town is defended (the cursor, as the step
    tells it); the right-click adds the fort level at every level and the army by the viewer's
    level: **none without a Tavern, the stacks' creatures with one, their size words with two or
    more**, the counts with the hall, the income, the silo and the garrison hero's mark only for
    the owner's team or a hero with expert Visions or a Rogue within reach. The wiki agrees; VCMI
    has two levels (no taverns' levels). At the start 63 players have level 0, 600 level 1, 78
    level 2 (§12.4).
27. **Batch TWN-1** (§13): seven rows, TD-3, TD-5, TD-7, the capture's aftermath, the game's
    version, the income and the Capitol after a capture. No new dispute or quirk candidate;
    content: the silo's table (a constant of the executable) for increment 4.

**T1b2b (§14):**

28. **The checkpoint** (`TWN-Q2`; the epic's decisions 8 to 16): the random hero and the
    starting-hero pick are increment 1's, the silo pays from increment 1 (its table moves there),
    the views follow the original's levels, T-1 to T-5 as proposed, TD-1, TD-4 and TD-6 by the
    reading, **TD-2 by the enabled expansions** (E08 decision 104), batch TWN-1 keeps rows 1 to 4,
    an allied town is entered and never captured.

**T3c (§15):**

29. **Batch TWN-1 scored:** TD-3, TD-5 and TD-7 and the siege's aftermath as the reading (the
    level-2 dwelling 21 of 96, VCMI's 50 in 100 refuted; the empty garrisons 12 of 12 empty;
    VCMI's merge refuted). **Not predicted:** building 23 in every Rampart, Necropolis and
    Conflux town of the standard set, from a part of the setup the reading did not follow; VCMI
    has no such building; the keep may count it. Re-read in T5b (`TWN-Q5`, decision 25).

**T5b1 (§16):**

30. **Building 23 is a display piece:** in Rampart, Necropolis and Conflux each hall builds a
    companion with it (Village Hall 23, Town Hall 27, City Hall 28, Capitol 29; `town_build`'s
    end), in every town of the three, a custom set too. None is in the faction's building list,
    so none is buildable or counted by the keep, none is superseded, and no rule reads them.
31. **The keep counts the faction's listed buildings only:** the town's mask +0x160 is the
    faction's list less the forbidden, and **no list holds the Village Hall** (nor the
    companions, nor the ship 20): a town whose highest hall is the Village Hall counts 0 for it.
32. **The standing set depends on the build order:** `town_build` turns a horde into its upgrade
    when the level's upgraded dwelling stands (either order), and a custom set built in id order
    leaves a horde's dwelling standing beside it (T-6); the closure of `supersedes` alone gives
    neither.
33. **VCMI 1.7.5 matches the lists** (the same buildings, requirements and supersedes) but for
    the Village Hall and the ship as buildings, 27 to 29 as automatic buildings without 23, the
    hordes' upgrades and two requirements (increment 2). **TD-8:** the code gives the Fortress's
    Glyphs of Fear (21) +2 attack and the Blood Obelisk (22) +2 defense, as VCMI; the game's own
    texts and the fandom wiki say the reverse.

## 1. The census

The 160 shipped maps (49 RoE, 55 AB, 56 SoD), read with `h3_map::read`; counts are towns or
players / maps. 744 colours play (human or computer).

### 1.1 Towns

| Class | Towns / maps | Owned (a colour that plays) | Neutral | Underground | RoE / AB / SoD |
|---|---|---|---|---|---|
| 77 random town | 876 / 127 | 562 / 118 | 314 / 78 | 114 / 33 | 139 / 378 / 359 |
| 98 Castle | 119 / 48 | | | | |
| 98 Rampart | 83 / 35 | | | | |
| 98 Tower | 69 / 37 | | | | |
| 98 Inferno | 80 / 42 | | | | |
| 98 Necropolis | 32 / 23 | | | | |
| 98 Dungeon | 89 / 39 | | | | |
| 98 Stronghold | 62 / 33 | | | | |
| 98 Fortress | 58 / 32 | | | | |
| 98 Conflux | 0 | | | | |
| 98 (all) | 592 / 93 | 327 / 78 | 265 / 59 | 94 / 43 | 210 / 177 / 205 |

- **No owner byte names a colour nobody plays.** No town of class 77 has a subclass other than 0.
  Neutral random towns by the map's format: RoE 28 (12 maps), AB 157 (34), SoD 129 (32).
- **Templates:** 20 DEFs (`AVCcast0`/`AVCcasx0` … `AVCrand0`/`AVCranx0`, the second of each pair
  drawn with a fort), all with one footprint: passable bytes `ff ff ff 8f 07 07`, visitable
  `00 00 00 00 00 20`: the bottom two rows blocked over five columns (the anchor's and the four to
  its left), the row above over three (columns x−3 to x−1), the entrance in the bottom row two
  columns left of the anchor, blocked and visitable (entered from below or the sides).
- **The SoD alignment byte** (random towns only use it): 255 ("the owner's or random") on 229 of
  230 owned random towns and on 83 neutral ones; 0 to 6 on 46 neutral random towns (9 maps) and
  on 1 owned one (`Battle of the Sexes`, aligned to its owner, colour 3); every aligned colour
  plays; no value from 7 to 254. Every fixed SoD town has 255.

### 1.2 Players

| A playing colour | Players / maps |
|---|---|
| with a main town | 553 (77: 369, 98: 184); 538 generate a hero (472 human-playable, 66 computer-only) |
| … not generating its hero | 15 (`Caught in the Middle`, `Faeries`, `For Sale`, `Just A Visit`, `Meeting in Muzgob` and its allied copy, `The Gauntlet`, `The Newcomers`) |
| without a main town | 191; 3 own no town (`Adventures of Jared Haret`, `Knee Deep in the Dead`, `Pirates`) |
| the "random town" flag set | 447 |

- **The main town** (`MainTown.pos`) is the entrance tile of a town owned by its player in all 553
  (the anchor less two columns). Its AB/SoD type byte equals the faction of a fixed main town (96
  players) and is 255 for a random one (276).
- **Allowed factions against the main town:** a fixed main town allows only its faction on 179 of
  184 players (more on `Unexpected Inheritance`, 2, and `Warmongers`, 3); a random main town
  allows all on 347 of 369 (all with the flag), 2 to 8 factions without the flag on 22 (`Chasing
  a Dream`, `Last Chance` and its allied copy, `Shadow Valleys`, `Caught in the Middle`,
  `Marshland Menace`). Without a main town: one faction on 53 players, more on 138.
- **Owned towns per player:** one random town 449 players, two to four 47; one fixed town 181,
  two to five 60; both kinds 4 (`Warmongers`, `The Gauntlet`); none 3.
- **SoD's byte before the allowed set** (VCMI: "faction is selectable", skipped): 1 on 56 players
  of 12 maps, 0 elsewhere; not read here.
- **The starting hero's flags** with a generating main town: the random-hero flag on 16 players
  (8 maps), a customised main hero on 5 (`Dragon Orb`, `Good Witch, Bad Witch`, `South of Hell`
  and its allied copy); T1a2's.

### 1.3 The towns' records

- **Buildings:** the standard set on 894 towns (700 with the has-fort flag, 194 without), custom
  sets on 574 (308 random, 266 fixed; 25 build nothing, 549 build something, 173 forbid
  something). Built at the start in the custom sets: a Town Hall on 370, a City Hall on 112, a
  **Capitol on 23**, a Fort on 460, a Citadel on 71, a Castle on 45; the most forbidden are the
  level-7 dwelling and its upgrade (bits 39, 40). Bits per `docs/formats/h3m.md`, "Town
  buildings".
- **Garrisons:** customised on 113 towns (53 of them empty), 12 random stacks in 7 maps (all in
  random towns: `One Bad Day - Allied`, `Peaceful Ending` and its allied copy, `The Great Race`,
  `Freedom`, `Pestilence Lake` and its allied copy). Formation tight on 3.
- **Names** customised on 252 towns; **required spells** on 32, **banned spells** on 396.
- **Town events:** 369 on 110 towns: 70 build, 56 add creatures, 105 give resources,
  189 carry only a message (3 maps); the increments that apply them read them (decision 5).

### 1.4 Heroes in and at towns

- **On a town's blocked tile:** 21 placed heroes (8 maps) and 2 random heroes (`Good to Go`),
  each in a town of its own owner and **each on the tile right of the entrance** (the entrance's
  column plus one, the same row). The import refuses 3 maps for it today (the 5 others stop
  earlier at a main town's hero). No hero stands on a town's entrance.
- **Random heroes (70):** 176 on 36 maps (1 to 8 a map, 14 on two; T1a1 wrote 166 and 1 to 9,
  recounted at T1a2).

### 1.5 The import (`MAP_IMPORT_VERSION` 15)

| Outcome today | Maps | With the main town's hero and a hero in a town allowed | With random heroes too |
|---|---|---|---|
| imports whole | 8 | 8 | 8 |
| a hero generated at a main town | 128 | 114 import, 14 stop at a random hero | all import |
| a hero standing in a town | 3 | 2 import, 1 stops at a random hero | all import |
| a random hero | 21 | 21 stop at it | all import |

Measured by clearing each map's generate-hero flags, removing its heroes on a town's blocked tile,
then its random heroes, and importing again; towns themselves stay blockers in all three runs.

### 1.6 Towns in the scenario

- **Victory:** capture a town on 14 maps, upgrade a town on 1 (`Hold the middle`, hall 2 and
  castle 2), build the Grail on 7; **loss:** lose a town on 2 (`For Sale`, `Fort Noxis`): E10's.
- **Random dwellings** (216, 217): 1 397 tied to a town by its identifier (216: 197 on 19 maps,
  217: 1 200 on 68), every one to a random town; 205 take a faction set instead.

## 2. The players' factions and the random towns (T1a1)

### 2.1 The order at the start

1. The scenario dialog holds a record per colour (0x7C bytes; H3API's `H3ScenarioPlayer`, its
   town at +0x24, −1 for Random). `scenario_dialog_begin` 0x588330 copies each colour's into the
   game's chosen factions (`game.chosen_factions`, +0x1F6B0, `structs.toml`), drawing a Random one
   (§2.2).
2. The new game (`new_game_heroes` 0x4BFE70) reads the map; `load_object_body` 0x502E00 sends
   classes 77 and 98 to `load_town_body` 0x5019F0, which appends a town setup record
   (`town_setup`, 0x88 bytes, `game.town_setups` +0x94) and resolves a random town's faction
   there (§2.3), in the objects' file order. The random dwellings resolve after all bodies are
   read (`random_dwellings_resolve` 0x502B60), a linked one from its town's record.
3. `setup_towns` 0x4CAA70 turns each random town's entrance tiles into class 98 of the record's
   faction (its picture redrawn) and sets every town up (§2.4).

### 2.2 The player's faction

- **The draw** (0x5885CD to 0x5886DB, per colour in colour order): the colour's dialog record's
  town when it is not −1; for −1, `random_allowed_faction` 0x5132D0 over the colour's allowed
  factions (`player_attributes` +0x8, game +0x1F914 + 0x44 × colour) without its "first" flag: the
  k-th set bit of the nine for k = `random_between(1, count)`, so **uniform over the allowed set**
  (a set without a bit gives faction 0). Not followed: a record that names another source of
  the choice (its +0x0 set: another copy of the choices, presumably network play's, 0x58869D)
  and a colour marked as not playing
  (+0x1F6D0 negative), which keeps its entry.
- **What the dialog offers** (not followed in full): the town shown is the record's when the
  map's "random town" flag (+0xA) is set or `faction_choice_offered` 0x58CE70 holds (the flag
  clear and more than one allowed faction, `count_allowed_factions` 0x5132B0); else the first
  allowed faction. So a colour with one allowed faction gets it; with more, the player picks one of
  them or Random. The flag's other readers are all in the dialog's functions; no rule reads it.
- **For the engine** (the setup's faction per player is the seam with E10's scenario setup,
  decision 6): a choice per playing colour, a faction of its allowed set or Random, validated
  against the map; Random drawn uniformly over the allowed set from the game's seed. The wiki
  agrees: the allowed alignments restrict the town available, and the random pool rolls only from
  them ("Player Specs").

### 2.3 A random town's faction (`load_town_body` 0x5019F0)

- The record's type (+0x68) starts as the template's subclass (0x501A93): a fixed town (98) keeps
  its faction. The identifier is 0 on an RoE map (0x501AA0); the owner is the file's byte.
- The SoD alignment byte is read after the events (0x502049: 0xFF before SoD). For class 77
  (0x502077):
  1. **an alignment other than 0xFF to a colour that plays** (its human or computer flag,
     `player_attributes` +0x0 or +0x1): that colour's chosen faction (0x5020AB);
  2. else **an owner other than 0xFF**: the owner's chosen faction (0x50215B);
  3. else `random_allowed_faction` of **0xFF** (the eight factions), or **0x1FF** (nine) when the
     edition number is 1 or 3 and `map_kind` (game +0x1F698) is at least 1 (0x5020C5); this build's
     edition is always 3 (`edition_pointer` 0x69923C), so **8 on an RoE map, 9 on an AB or SoD
     map**, Conflux included, whatever the players' allowed sets.
- The alignment is a signed index: a byte from 8 to 254 reads another record than a colour's
  (T-1). A colour that does not play falls through to the owner, then to the draw.

### 2.4 The random town becomes a town (`setup_towns` 0x4CAA70)

Per entrance tile of a class 77 or 98 object (tile +0xD bit 0x10), the town's position; a class 77
tile becomes class 98 with the record's type as its subclass and is redrawn (0x4CAC86 to
0x4CAC9B). A town without a custom name draws one from its faction's name list without repeats,
from a faction drawn by `random_between(0, 8)` once its own is used up (0x4CACC7 to 0x4CACFE);
presentation only (the names are text, not harvested). Then the town's setup and its buildings
(T1a2).

### 2.5 VCMI 1.7.5 and the wiki

- **VCMI's reader** (`MapFormatH3M.cpp`): the allowed set kept only when it is not "random and
  every faction allowed" (then the game's default, all nine); the main town's position converted
  to the object's (the H3M's is its active tile, as §1.2); the alignment: 255 neutral, below 8 a
  colour that can play, 8 to 15 "not as player" (HotA, not implemented, warned), the rest
  "corrupted" (warned), each of these kept as "the owner's or random".
- **VCMI's draws:** a player's Random uniform over its allowed set
  (`CGameState::initRandomFactionsForPlayers`), as the reading; a random town
  (`CGTownInstance::randomizeFaction`) takes **the owner's faction first, then the alignment's**,
  else a uniform draw over every faction with a town (nine, on every map's format) (TD-1, TD-2).
- **The wiki** ("Random Towns"): "Same as Owner or Random" gives the owner's faction, a random
  one when unowned; "Same as player" applies even when that player is not the owner, so an owned
  town can be "a differently aligned town" (the reading's order, against VCMI's); it falls back
  to random when the player has no faction. HotA's changelog 1.6.0 adds Conflux to RoE maps'
  random towns, which implies SoD had none there (the reading).

## 3. Disputes, quirk candidates and open points (T1a1)

| Id | What | Reading | VCMI | Wiki | Shipped maps |
|---|---|---|---|---|---|
| TD-1 | An owned random town aligned to another colour | the alignment's colour | the owner's | the alignment's | none (the one owned aligned town is aligned to its owner) |
| TD-2 | Conflux for a random town on an RoE map | never: a neutral one draws from eight, a player's set has eight bits | a neutral one draws from nine; a player with the flag and all eight allowed draws from nine | none on RoE maps (HotA 1.6.0 added it) | 28 neutral random towns (12 maps); 105 players with the flag and all eight allowed |

- **T-1 (quirk candidate):** an alignment byte from 8 to 254 indexes the playable flags past the
  eight colours (a signed byte: up to 127 forward, 128 to 254 backward into other fields) and may
  take a faction from another table entry. No shipped map has one; the editor writes 0 to 7 or
  255. Proposal: the import refuses such a byte (a typed error), as VCMI warns.
- **Open for T1a2:** the hero at the main town and its pool (the hero classes' tavern weights, the
  map's allowed heroes, the faction, the player's starting-hero choice and the random-hero flag),
  the random hero's draw (class 70; finding 2), a hero on a town's blocked tile (§1.4: always the
  tile right of the entrance), the standard buildings and the has-fort flag at the start
  (`setup_towns`' setup 0x5C0670, its buildings 0x5C08C0), the garrison's random stacks
  (finding 6).

## 4. The heroes at the start (T1a2)

### 4.1 The order at the start

1. `game_reset` 0x4BF1A0: every hero type free (`game.hero_owner` +0x4DF18, −1) and takeable by
   every colour (`game.hero_masks` +0x4DFB4, 0xFF); the choices (`chosen_heroes` 0x69FB24) −1.
2. The scenario dialog's start (`new_game_start` 0x58C570, 0x58C680 to 0x58C6CE): each colour's
   chosen hero (its record's list at +0x2C, the index at +0x20; −1 Random) into
   `chosen_heroes` by colour; network play fills it at 0x4EF611 (not followed).
3. `new_game_heroes` 0x4BFE70 (with the choices, 0x4C05E5): every hero type's starting army,
   experience and movement first (E08's readings); then for each human colour whose main town
   generates a hero, its chosen hero marked as the colour's (0x4BFF83) and stored as the colour's
   first hero (`game.first_heroes` +0x1F844, when still −1). **This mark does not last:** the map
   read that follows starts the pool again (step 4; T-2).
4. The map read (`new_game_bans` 0x4C2450): the header (`map_header_read` 0x4C4390, the allowed
   heroes), **the pool** (`hero_pool_init` 0x4C4E30, §4.2), the bans, then the objects
   (0x4FD690) in the file's order: a placed hero's or prisoner's body marks its type
   (`load_hero_body` 0x5021C0, 0x502973: a prisoner 0x41, a hero its owner), **a random hero is
   drawn there** (§4.4).
5. Then: the random objects (0x4C9DD0), `setup_towns` 0x4CAA70 (§5), the placed heroes
   (`placed_heroes_setup` 0x4CAE10: a hero in a town moved, §4.6; each hero put on its tile),
   a campaign's carried heroes, **the main towns' heroes** (`main_town_heroes` 0x4CA040, §4.5),
   the players with neither a hero nor a town out of the game (0x4C007A), the taverns' heroes
   (`tavern_new_heroes` 0x4C8450, increment 3).

So the wiki's order holds: the random heroes are drawn before the main towns' heroes, and both
after every hero placed earlier in the file.

### 4.2 The pool (`hero_pool_init` 0x4C4E30)

- Per hero type 0 to 155: free (−1) when the map's allowed set (`game.allowed_heroes` +0x1FB5C)
  has its bit, else 0x40, out of the game for every draw. Then each SoD disposed hero whose mask is
  not 0xFF sets the type's colour mask to it (0x4C4EC4): only those colours may take it. Last, a
  victory "acquire an artifact" (0, or 10) marks its artifact (artifacts, not heroes).
- **The allowed set by version** (`map_header_read`, 0x4C47E4 to 0x4C498C): an RoE map's 16
  bytes give types 0 to 127 and, **outside a campaign, types 128 to 143 (the Conflux heroes) are
  set besides** (0x4C48EF), 144 to 155 never; AB and SoD read 20 bytes, types 0 to 155.
- **The census:** no placed hero (108) or prisoner (89) has its type allowed on its map; RoE
  maps 49 (their Conflux heroes allowed so); AB/SoD maps: all 16
  Conflux heroes allowed on 94, none on 13, 8 on 3, 15 on 1; some of the 12 campaign heroes
  allowed on 99 (151 in all); SoD's disposed heroes: 832 on 9 maps, each with a mask other than
  0xFF.
- **What takes a hero out of the pool:** its owner's colour when it is placed, drawn or
  generated (`hero_place` 0x4D7900, `load_hero_body`), 0x41 for a prisoner; −1 again when a
  tavern slot's hero is released at the start (0x4C00F7) or a player is defeated (0x4F13F3).

### 4.3 The player's starting-hero choice

- **The offer** (`scenario_hero_lists` 0x583890, read whole; the dialog's other tests of the
  flags not followed in full): for a colour that plays with a faction (not Random), up to
  **16 heroes in type order**: free in the dialog's pool (the header and `hero_pool_init`
  only, so placed heroes are not marked, but none is allowed on the shipped maps), of the
  faction's two classes (the hero table's class, `hero_table` +0x8), takeable by the colour (a
  disposed hero's mask). The column offers the choice when the colour's main town generates a
  hero or the colour has the random-hero flag; with neither it shows the map's customised main
  hero (`player_attributes` +0x1C), which nothing outside the dialog reads.
- **Where it goes:** the colour's **first random hero in the file's order** takes it
  (`load_hero_body` 0x5022ED: then the choice is cleared, so a second random hero draws); else
  **a human colour's** main-town hero (`main_town_heroes` 0x4CA146 tests the human flag +0xE2; a
  computer colour's choice is honoured only by a random hero). Neither checks that the hero is
  still free (T-2).
- **The census:** the generate flag with no customised main hero on 533 players (16 with the
  random-hero flag too), the random-hero flag without the generate flag on 124 (3 with a
  customised main hero), a customised main hero on 62 (5 with a generating main town).

### 4.4 The draw and the random hero

- **`hero_draw_of_faction` 0x4BB400** (faction, colour; read whole): the faction's two classes
  (0 and 1 for Castle, then 3 and 2, 5 and 4, 6 and 7, …, 16 and 17 for Conflux; a faction
  outside 0 to 8 takes Castle's, T-4); the list of every type that is free, takeable by the colour
  and of one of the two classes (the game's hero record +0x30), in type order; when empty, every
  free type takeable by the colour; one of them by `random_between(1, n)`. **No weights:** the
  hero classes' tavern weights (HCTRAITS, `hero_class` +0x34, one byte per faction) are read only
  by the tavern's draw (`tavern_hero_draw` 0x4BB5E0, read for this: a class drawn by those
  weights, then a hero of it uniformly; on an SoD map a non-Conflux colour's tavern skips the
  Conflux classes while other heroes are free). That draw is increment 3's.
- **The random hero** (class 70, `load_hero_body`): with the owner's choice (§4.3), else
  `hero_draw_of_faction(chosen_factions[owner], owner)` at once (0x502352); the type is the
  owner's from then (0x502988), so the next draw excludes it. The rest of its body applies as a
  placed hero's (`placed_hero_setup`, E08's). A type placed later
  in the file is still free at this draw (T-3).
- **The census:** 176 random heroes on 36 maps (T1a1's 166 was wrong); their owners allow one
  faction on 10, three to six on 8, eight on 28, nine on 130; 2 stand in a town (§4.6); no hero's
  army on the shipped maps holds a random stack (the import refuses one; not read).

### 4.5 The main town's hero (`main_town_heroes` 0x4CA040, read whole)

- Per colour 0 to 7, when `player_attributes` +0xB (generate a hero at the main town; RoE: always
  with a main town) is set: the town whose record's position (its entrance, §2.4) equals the
  main town's (+0x14); the hero: a human colour's choice, else
  `hero_draw_of_faction(chosen_factions[colour], colour)`; the colour's first hero if none yet;
  the type the colour's; `town_hero_place` 0x5C13B0 puts it on the town's entrance
  (`hero_place` 0x4D7900: added to the player's heroes, the type's mask gains the colour, shown
  on its tile); then the town's guild spells are learned (0x5BE030, increment 4); one campaign
  scenario adds experience (not followed).
- **The faction is the colour's, not the town's:** on the 5 players whose fixed main town allows
  other factions too (`Unexpected Inheritance`, `Warmongers`, §1.2), a hero of another faction can
  stand in it; VCMI does the same.
- **The visiting hero:** `hero_show` 0x4D75D0 (read whole) remembers the tile's object and makes
  the hero the tile's; on a town's entrance it sets the town's visiting hero (+0x10, 0x4D76A9).
  So the generated hero, and every hero shown on an entrance, is its town's visitor.

### 4.6 A hero on a town's blocked tile (`placed_heroes_setup` 0x4CAE10)

- For each placed record with a position: when the tile one column left of the hero's is an
  entrance (+0xD bit 0x10) of a town (class 98; random towns are 98 by then) and the hero is not a
  prisoner, the hero's column is decreased by one (0x4CAE95 to 0x4CAEC2); the hero is then shown
  on its tile (the visitor, §4.5). Any owner's town; no other blocked tile.
- **The census:** 23 such heroes (21 placed, 2 random), all on the tile right of the entrance, in
  towns of their own owners, none in a main town that generates a hero (so no town starts with
  two visitors).

### 4.7 VCMI 1.7.5 and the wiki

- **VCMI:** `pickNextHeroType` takes the player's chosen hero while no hero of that type exists
  on the map, else `pickUnusedHeroTypeRandomly`: uniform over the allowed heroes of the player's
  faction that the player may take, **without the other players' choices and every hero on the
  map**, else over the other factions' (then any not allowed, then an error). A random hero
  draws so at the objects' randomisation (before the starting heroes), every object first read;
  `placeStartingHeroes` for each player with the generate flag and a main town, at the town's
  entrance, for computer players' choices too. `placeHeroesInTowns`: a hero on **any blocked tile
  of a town of its own owner** moves to its entrance (TD-6); `initVisitingAndGarrisonedHeroes`
  makes a hero on its own town's entrance the visitor. So VCMI reserves the choices (T-2) and the
  map's heroes (T-3).
- **The wiki and others:** "generate hero at main town" makes no difference against a random
  hero placed in the town, the player choosing among the town's heroes (Celestial Heavens); the
  starting hero is one of the chosen town's, unless the map sets the hero (GOG forum). Both
  agree with the reading.

## 5. The towns at the start (T1a2)

### 5.1 The town's setup (`town_setup` 0x5C0670, read whole)

From `setup_towns` 0x4CAA70, per town record (`town_setup`, §2.4): the faction (+0x68), owner −1
then the record's (`town_change_owner` 0x4C61E0, T1b's), the second army cleared, the garrison
(§5.2), the buildings (`town_buildings_setup`, §5.3), the "every built" set (+0x158: the built
set with each built building's superseded ones, 0x6A8BB8), the spells' bytes (+0x70 to the
town's +0xD4), then 0x5BE600 with the record (not read; the guild's spells, increment 4,
presumably).

### 5.2 The garrison

- **A custom garrison** (+0x19): the seven slots as the map gives them; a stack with a count above
  0 and a code from −2 down (**the random stacks**) becomes the town's faction's creature of level
  (−2 − code) / 2, upgraded when the code is odd (`town_creatures` 0x6747B4); a count of 0 leaves
  the slot empty, so a custom empty garrison stays empty.
- **No custom garrison:** empty; then **a neutral town** draws: for levels 1 to 4,
  `random_between(1, 100)` at most 33, 33, 20, 13 (0x688E84) adds the faction's unupgraded
  creature of that level, `random_between` of 8–15, 5–7, 3–5, 1–3 (0x688E94, 0x688EA4) of them
  (`army_add_stack` with slot −1). An owned town's stays empty.
- **The census:** neutral towns: 497 without a garrison (drawn), 41 custom, 41 custom and empty;
  owned towns: 858 without (empty), 19 custom, 12 custom and empty. The 12 random stacks (7
  maps, T1a1) are in custom garrisons of random towns.
- **VCMI** (`initializeNeutralTownGarrison`): the same four rows with level 4 at 14 in 100
  (TD-4), and only when the town has no stack, so it also fills an explicitly empty one (its own
  FIXME; TD-5). **The wiki** ("Neutral Town Garrison"): the four rows as read, 13 in 100, and
  only when the map defines no garrison.

### 5.3 The buildings (`town_buildings_setup` 0x5C08C0, read whole)

1. The recruits, the built and the every-built sets cleared; **the Village Hall** (10) built.
2. **Forbidden** from the start: the Grail building (26) on a map whose victory is to build the
   Grail, in a town that condition does not name (0x5F2160, not read); the Shipyard (6) when the
   faction has none or neither (column − 1, row + 2) nor (column + 1, row + 2) from the entrance
   is a boat's spot (0x5C0C90, not read).
3. **A custom set** (+0x5): the map's forbidden bits (+0x10) through `map_building_ids` 0x6888C0
   added to the forbidden; a forbidden horde (18, 24) forbids its upgrade (19, 25). The buildable
   set (+0x160) is the faction's (0x6976F0) less the forbidden. The built: each of the map's
   built bits (+0x8) with **the buildings it supersedes** (0x6A8BB8, computed at start-up by
   0x4EB730; named from its use, as `town_build` removes them from the built set: so presumably a
   Capitol brings the lower halls and a Castle the Citadel and Fort), less what is not buildable;
   each built in id order (`town_build` 0x5BE930).
4. **The standard set** (+0x5 clear): the buildable set as above; **a Fort** when the map's flag
   (+0x18) is set; **an owned town** a Tavern (5), and **only with a Fort** the level-1 dwelling
   (30) and, at `random_between(1, 100)` at most 30, the level-2 one (31); **a neutral town** both
   dwellings (30, 31), with a Fort or without.
- **The census:** standard sets: owned with a Fort 495, owned without 51, neutral with 205,
  neutral without 143; custom sets: none builds a forbidden building; the Shipyard built in 9
  towns, the Grail in 10; 7 maps' victory is to build the Grail.
- **VCMI** (`initTowns`, `startingDwellingChances` [100, 50]): the Village Hall; a Tavern when
  owned; **the level-1 dwelling always and the level-2 one half the time, for every standard
  town**, owned or neutral, Fort or not (TD-3); a built Shipyard without water removed.
- **Others:** the HotA team's documentation of its own towns: the first two dwellings are built in
  a neutral town or one with a Fort, a Tavern in a town owned at the start, and "towns with Fort
  always have level 2 dwelling" (HotA's own rule; the reading's 30 in 100 is SoD's); a fandom page
  on random maps: player towns with a Tavern, the level-1 dwelling and sometimes the level-2,
  neutral ones without a Tavern. The Lazy wiki's Town page says nothing on the buildings.

## 6. Disputes, quirk candidates and open points (T1a2)

| Id | What | Reading | VCMI | Wiki, others | Shipped maps |
|---|---|---|---|---|---|
| TD-3 | A standard town's dwellings | owned: none without a Fort; with one level 1, level 2 at 30 in 100; neutral: levels 1 and 2 | levels 1 always, 2 at 50 in 100, every town | HotA: levels 1 and 2 in a neutral town or one with a Fort (level 2 always, HotA's); fandom: level 1, sometimes 2 | 894 standard towns, 51 owned without a Fort |
| TD-4 | A neutral town's level-4 roll | 13 in 100 | 14 | 13 (Lazy wiki) | 497 towns draw |
| TD-5 | A neutral town's explicitly empty custom garrison | stays empty | drawn (VCMI's FIXME) | only an undefined garrison is drawn | 41 towns |
| TD-6 | A hero on a town's blocked tile | only right of any town's entrance, onto it | any blocked tile of its own owner's town, onto the entrance | — | 23, all right of their own town's entrance |

- **T-2 (quirk candidate): the starting-hero choice is not reserved.** `new_game_heroes` marks a
  human's chosen hero (0x4BFF83), the map read's `hero_pool_init` frees it again (0x4C4E6B), and
  neither `load_hero_body` nor `main_town_heroes` checks a choice before placing it. So another
  colour's random hero, or an earlier colour's main-town draw, can take the chosen hero, which is
  then placed a second time for its chooser: one hero type on two tiles and in two players' lists
  (state-corrupting). Exposure: any start where a colour chooses and another colour can draw its
  faction's heroes. Proposal: fixed (a state-corrupting bug, `AGENTS.md`): the choices are
  reserved before any draw, as VCMI's; a batch TWN-1 row only if a cheap way to force the draw
  exists.
- **T-3 (quirk candidate): a random hero can take a later hero's type.** A placed hero or
  prisoner marks its type only when its body is read, so a random hero earlier in the file can
  draw it (when allowed); the later body then overwrites the type's record (`hero_setup` by type).
  None on the shipped maps (no placed hero is allowed). Proposal: fixed: every placed hero's and
  prisoner's type leaves the pool before the random heroes are drawn (VCMI's order).
- **T-4 (quirk candidate): the draws' empty cases.** A generating main town whose position names
  no town (the town pointer stays null and is used); a draw with no free hero at all
  (`random_between(1, 0)`, then a read below the list); a faction outside 0 to 8 drawing
  Castle's classes. None on the shipped maps (all 553 main towns match; 156 heroes). Proposal:
  the import refuses an unmatched main town (typed), a draw with nothing free fails typed (VCMI
  throws), the faction is always one of the nine.
- **Open, for T1b's checkpoint and T2:** the random hero in increment 1 (`TWN-Q2`; it shares
  §4.4's draw, so it costs little more than the main town's hero); who may choose a starting
  hero and from which list (E10's setup seam; the original's offer is §4.3, a computer colour's
  choice honoured only by a random hero); the generated hero's spells from the guild at once and
  the Shipyard's spot (increment 4); the superseded buildings (0x6A8BB8) and the factions' sets
  are increment 2's content, increment 1 needing the halls' and forts' chains and the first two
  dwellings; the neutral garrison's table and the 30 are constants of the executable (ADR-0006
  tier 2, with this reading as their source). TD-3 and TD-5 are batch TWN-1 rows (an owned
  standard town without a Fort; a neutral town with an empty custom garrison).

## 7. The visit and the visiting hero (T1b1)

### 7.1 The step onto a town's gate (`hero_move_step` 0x4805E0, case 98 at 0x480E3B)

- **Entered, not from above.** Class 98 (a random town is one from the new game on, §2.4) is on
  neither of E08's lists, `object_classes_not_entered` (0x63AA58) or
  `object_classes_from_above` (0x63AB00): the hero stands on the gate (ADV-OBJ-003's entered
  mode) and no step crosses the gate's top (ADV-OBJ-004). With the one footprint (§1.1) a
  walker cannot reach the gate from the row above or the sides anyway (the town's own blocked
  cells): it enters from below or the lower diagonals and leaves the same way. The setting
  matters for a flyer (ADV-MOV-009): it never comes down onto the gate from the three tiles
  above, and passes up off it over the town, as off every entered object not visited from above
  (the queue's T14a note asks for this typing: an untyped blocker closes nothing).
- **The step** (0x480E3B): the town by the tile's word (+0x0). Its owner on the current
  player's team (game +0x1F879): the ordinary step (0x480CB5), the hero enters. Else
  `town_defended` 0x5BDFE0 (a visiting hero, or any creature in the army
  `town_garrison_army` shows: the garrison hero's, else the town's +0xE0): the visit from the
  neighbouring tile (`visit_from_neighbour` 0x480240, ADV-OBJ-003's neighbour mode: the step
  paid, ADV-MOV-006's drop, the hero stays). Else (an undefended town of another team or none):
  the hero enters.
- **A hero on the gate** is what the step meets (class 34, the tile's object while it stands
  there): its own team's hero meets it (GAM-MET-001), an own or allied town's visitor included;
  another team's hero visits it, and `visit_hero` 0x4A2800 sends a hero standing on a town's gate
  (`town_at_entrance` 0x4BB870: the town whose +0x5 to +0x7 is the tile) to the town's visit
  (0x4A2904) instead of the field battle. Such a town is defended, so this too is a visit from
  the neighbouring tile.

### 7.2 The town's visit (`town_visit` 0x4AAFD0, read whole)

From `visit_object`'s case 98 (0x4AA2B6, after an entering step or a neighbour visit) and from
`visit_hero` (0x4A2904); the visiting hero is the current player's selected one (player +0x4).

1. The town at the tile (`town_at_entrance`); the hero stopped (`adventure_hero_stop`).
2. **The town's owner on the hero's team:** for an own town a victory condition's test
   (0x5F2860 with the hero and the tile, E10's) ends the visit when met; then a human's town
   screen (0x5BE210), a computer's town logic (0x5253D0, E10's). An ally's town the same without
   the test. No owner changes.
3. **Another team or none:** with a defender (the army `town_garrison_army` shows has a creature,
   or a visiting hero other than the visitor itself, which `hero_show` made the visitor as it
   entered) the battle (§9); else the capture (§8.1).
4. Where the path does not end early: the town's heroes learn the guild's spells
   (`town_heroes_learn_spells` with 0: the garrison hero and the visitor; increment 4), the
   visitor's level-up (`hero_level_up`), the scenario's tests (0x5F1B10, 0x5F1C80; E10's).

### 7.3 The visiting hero

- `hero_show` (T1a2) makes a hero put on a town's gate its visitor (+0x10); `hero_hide`
  0x4D76E0 clears it (0x4D77A5) when that hero leaves the gate or is taken off the map. So the
  visitor is exactly the hero standing on the gate: an entering hero (an own or allied town, an
  undefended town it captures), the main town's hero (§4.5), a hero moved into a town (§4.6).
- A garrison hero (+0xC) comes only from the town screen's swap (increment 3); the map format has
  none. In increment 1 a town holds at most a visitor and its own army (the garrison, §5.2).

## 8. Capture (T1b1)

### 8.1 An undefended town (`town_visit` 0x4AB0EE to 0x4AB255)

The hero has entered and is the visitor. In order:

1. A garrison hero whose army is empty (the town screen lets a garrisoned hero dismiss its last
   stack, a reader's comment) goes back into the heroes' pool (0x4D9EC0 with 1, 0, as a hero that
   lost a battle), the loss condition's hero test runs (0x5F2E10; E10's), and the town's garrison
   hero is −1: no battle, no experience, nothing taken (quirk candidate T-5).
2. `town_change_owner` 0x4C61E0 (the town, the current player, quiet 0, test 1): §8.3.
3. The Capitol's limit (`capitol_limit` 0x5BEC60): §8.4.
4. A loss condition's test (0x5F2E40 with the old owner and the town; E10's "lose a town").
5. The own town's victory test (as §7.2 step 2, 0x4AB22A); a human's town screen, a computer's
   town logic; then §7.2's step 4.

### 8.2 After a siege won (`town_visit` 0x4AB33D to 0x4AB3BA)

When the battle returns 0 (the attacker won; battle manager +0x13D48): the town's garrison hero
−1 (the battle took it off, §9.3), `town_change_owner` with the same arguments, the Capitol's
limit, the loss condition's test, then §7.2's step 4. **No town screen: the attacker stays on the
tile it attacked from** (§7.1's neighbour visit) and enters the town by a later step, which finds
it own. A defender's win (1) or no winner (−1, both sides destroyed) changes no owner.

### 8.3 A town's change of owner (`town_change_owner` 0x4C61E0, read whole)

Arguments: the town, the colour, quiet, test. Nothing when the colour already owns it. Else:

1. Unless the new game's setup runs (0x698400) or quiet: a map message and a record of the change
   (`town_owner_message` 0x49C190: network play's, and the replay of others' moves; not followed).
2. Town +0x32 cleared (not read).
3. Unless quiet, **the external dwellings' counts** (town +0x118, i32[14] by the faction's
   creature index: unupgraded 0 to 6, upgraded 7 to 13): for every creature dwelling (game
   +0x4E39C, 0x5C bytes, owner +0x57) of the old or the new owner whose creature has a faction
   (`dwelling_creature_faction` 0x4C6690: under RoE rules none for the four elementals 112 to
   115), every town of that owner of that faction loses 1 (`town_dwelling_count_add` 0x5BFE50: an
   unupgraded creature counts at its index and at the index + 7); after the owner's write the same
   dwellings add 1 again (0x4C6568's loop). So a town's counts follow its owner's dwellings.
   What reads them is increment 3's (growth).
4. An owned town leaves its old owner's list (`town_remove_from_owner` 0x5BE2D0: player +0x40,
   +0x3E; the town list redrawn); town +0x2 is cleared when the old owner's team has no human
   and the new owner's team has one (not read; probably the computer's).
5. **The owner** +0x1 is the colour (0x4C63DF). With quiet set the function ends here.
6. **The garrison emptied** (`army_reset` 0x44AB00 on `town_garrison_army`'s army; after §8.1 and
   §8.2 the garrison hero is −1, so the town's own +0xE0). So the stacks a visitor's merge left in
   the garrison (§9.2) are lost with the town.
7. A new owner (not −1) appends the town to its list; with test, the victory condition's tests
   (0x5F2260, 0x5F22A0; E10's "capture a town").
8. **The reveal** for the new owner (`reveal_disk` 0x49D160): radius 5 around the gate; a Tower
   (faction 2) with the Lookout Tower (building 21) radius 20; a Tower with the Grail (26) every
   tile of the surface (the map's middle, radius the map's width) and of the underground when the
   map has one (game +0x1FC48). The same radii as `reveal_owned` at the game's start (E08 T80).
9. Unless quiet, the counts' second half (step 3).

`town_setup` calls it for every town with the record's owner, quiet 0 and test 0 (§5.1; the town
was set to −1 just before), so the start's owned towns are listed, revealed and counted the same
way; the garrison it empties is set right after.

### 8.4 The Capitol's limit (`capitol_limit` 0x5BEC60, read whole)

After the change: a town with the Capitol (building 13) in its built set, whose owner has another
town with the Capitol, loses it: 13 leaves the built (+0x150) and every-built (+0x158) sets, the
City Hall (12) enters the built one, and the map picture is redrawn (`object_template_redraw`
0x4C9990: no Capitol's picture). A neutral owner skips it. So a player holds at most one Capitol
after a capture (VCMI's `removeCapitols` razes it the same way); T1b2 reads what a hall pays.

## 9. The battle at a town (T1b1)

### 9.1 Who defends (`town_visit` 0x4AB25A to 0x4AB338)

| The town holds | Battle | Defender | Its army |
|---|---|---|---|
| a visitor and a garrison hero | **not at the town** (town argument 0: no siege, the battlefield the gate tile's, ADR-0022) | the visitor, its owner's side | its own |
| a visitor, no garrison hero | a siege | the visitor | its own, the garrison merged in first (§9.2) |
| a garrison hero | a siege | the garrison hero | its own |
| neither (creatures in the garrison) | a siege | no hero | the town's (+0xE0) |

In a siege the defending side's player is the town's owner (+0x1; −1 neutral), the hero's owner
in the first row. After the first row's battle the town is not taken: the attacker's next visit
fights the garrison hero (coyot's guide: "another click"; VCMI's `isBattleOutsideTown`). In
increment 1 (no garrison hero, §7.3) only the second and fourth rows occur.

### 9.2 The garrison's merge into the visitor's army (`garrison_merge` 0x44B820, read whole)

Before a siege a lone visitor defends, the town's army joins it, by each stack's value (the
creature's AI value, its record's +0x40, times the count), round after round:

1. the visitor's weakest stack (the least value; of equal ones the last slot);
2. each garrison stack's gain: its value when the visitor has a stack of its creature or an empty
   slot, else its value less the weakest stack's; the stack of the greatest positive gain (of
   equal ones the first slot);
3. it joins the visitor's stack of its creature, else its first empty slot (`army_add_stack`
   0x44ACE0), and leaves the garrison; with no room it changes places with the weakest stack
   (creature and count swapped), which stays in the garrison.

It ends when no stack gains (every round empties a garrison slot or raises the army's value, so
it ends). What stays in the garrison does not fight and is lost if the town falls (§8.3 step 6);
after the defender's win the visitor keeps the merged army.

### 9.3 The battle and its outcome (`adventure_battle` 0x4AD470, its town paths)

- `battle_side_setup` 0x4639F0 (0x4AE27C) receives the town and stores it (+0x53C8): every
  battle at a town reads it from there (E04c; §9.4).
- After it (0x4AE431 to 0x4AE898): a defender's win (1) sends the attacking hero back into the
  pool (0x4D9EC0) with the loss's effect (`hero_loss_effect` 0x4ACBB0: only a sound and the view's
  dissolve); an attacker's win (0) the defending hero, the garrison hero without the effect
  (argument −1: it stands on no tile); with no winner (−1) both heroes go to `battle_aftermath`
  0x4AC930 (not read; the garrison hero again with −1). The
  survivors stay in the armies the battle was given: the visitor's, the garrison hero's or the
  town's own (as E08 T84a's `Defender::Garrison` for a mine's guards). The +500 experience of a
  battle at a town is the battle's (`installation.md` §7.27).

### 9.4 ADR-0023's facts from the town

Every fact was read by E04c (`installation.md` §7.27 to §7.33); in increment 1 a custom building
set (§5.3) can hold any building from the start, so the derivation needs all of them:

| `Siege` field | From the town |
|---|---|
| `faction` | +0x4 |
| `fort` | the built set +0x150: Castle (9), else Citadel (8), else Fort (7), else none (tested 7, 8, 9 in order, the last found wins) |
| `moat` | with a Citadel or Castle: absent for a Tower (its land mines instead) and, under RoE or AB rules, a Stronghold; else damaging under SoD rules, harmless under RoE or AB rules |
| `structures` | the building ids 0 to 43 in the built set among the buildable mask +0x160 (`town_turret_count` 0x5BF4F0 adds its 4) |
| `escape_tunnel` | a Stronghold (6) with building 17 in the every-built set +0x158 |
| `morale` | +2 for a Castle (0) with the Brotherhood of the Sword (22, which replaces the Tavern in the built set), else +1 with the Tavern (5) in the built set |
| `luck` | +2 for a Rampart (1) with the Fountain of Fortune (21, every-built set) |
| the defending hero's bonuses (point 3) | Tower Grail (26) +150 mana; Inferno 21 +2 power; Dungeon Grail +12 power; Stronghold Grail +20 attack; Fortress 21 +2 attack, 22 +2 defense, Grail +10 attack and defense |

**The game's version** (game +0x1F698, `map_kind`; ADR-0023's "probably the map's format"):
`new_game_heroes` writes its fourth argument (0x4BFF09); `new_game_start` passes the scenario
dialog's version (+0x1898: 1 AB, 2 or 3 SoD, any other RoE; 0x58C593); a campaign takes its
index instead (below 7 RoE, below 13 AB, else SoD; 0x4BFF24), a missing version SoD. The dialog
starts at the edition number (`edition_pointer`) and, as a scenario is chosen, takes the highest
of 3 down to 0 set in a mask (`scenario_version_from_slots` 0x58E700,
`scenario_version_highest` 0x58EA00: a mask per colour slot in use and the chosen scenario's;
which bits a map's format sets not followed). So an RoE map in the Complete edition
may play under SoD rules (a damaging moat, a Stronghold's moat): batch TWN-1's row (§10).

### 9.5 VCMI 1.7.5 and others

- `CGTownInstance::onHeroVisit`: an enemy facing a garrison with stacks or a garrison hero, or a
  visitor: the defender is the visitor, else the garrison hero, else the town;
  `isBattleOutsideTown` (a visitor defending while a garrison hero exists) passes no town;
  `mergeGarrisonOnSiege` before a siege the visitor defends; an undefended town `onTownCaptured`
  and the town's visit. `battleFinished`: an attacker's win clears the garrison (`clearArmy`) and
  captures. `onTownCaptured`: `setOwner` (`removeCapitols`) and the fog revealed at
  `getSightRadius` (a base setting and sight bonuses). `CGameHandler`'s `startVisit` sends a visit
  of an enemy hero in a town to the town unless the battle is outside (by owner, not team).
  **Agrees** on the defender, the battle outside, the merge's existence, the garrison cleared,
  the Capitol; **differs** on the merge's order (TD-7).
- VCMI's merge: as many rounds as the garrison had stacks, each taking the stack of most power
  (count times AI value, plus the visitor's stack of its creature), moved into the slot for its
  creature, else swapped with the visitor's weakest stack weaker than it (the first of equal
  ones).
- coyot's guide: with both heroes the visitor is fought on free terrain, "another click" fights
  the one inside; a lone visitor gets the garrison merged in, "what cannot be merged is unused",
  the author not knowing the order. The wiki's Tutorial: a garrison hero defends from within the
  fortifications.

## 10. Disputes, quirk candidates, open points and batch TWN-1's rows (T1b1)

| Id | What | Reading | VCMI | Others | Increment 1 |
|---|---|---|---|---|---|
| TD-7 | The garrison's merge into a lone visitor's army | greatest gain first (a stack's value, less the weakest stack's when it does not fit), until nothing gains; ties: the first garrison stack, the last weakest slot | most power first (plus the visitor's same-creature stack), one round per garrison stack, a swap only with a weaker stack, ties the first | the guide: merged, order unknown | every siege a visitor defends (the main town's hero, a placed hero in a town) |

- **T-5 (quirk candidate): a garrison hero without creatures.** The town falls to the first
  visitor without a battle (§8.1 step 1): the hero goes back into the pool (its artifacts with
  it, not followed), the attacker gains no experience and takes nothing. No state corruption; it
  needs the swap and the dismissal (increment 3). Proposal: the original's (faithful), recorded
  now, built with increment 3; no batch row until then.
- **Open, for T1b2's checkpoint and T2:** the town screen's own effects at a visit (the guild's
  spells: increment 4); allied towns (teams, E10: an ally's town entered and its screen opened, no
  capture; a visitor of an ally defending under the town's owner); the computer's town visit
  (E10, E13); the scenario's tests (E10); town +0x2 and +0x32 (not read); what reads the external
  dwellings' counts (increment 3, with E08's dwellings: E09 decision 4).
- **Batch TWN-1's rows from T1b1** (beside T1a2's TD-3 and TD-5): (1) a siege defended by a
  visitor with seven stacks and a garrison of mixed values (TD-7: the defending stacks and the
  garrison after); (2) after a siege won, the attacker's tile and points, no town screen, the
  flag; (3) the stacks left in the garrison gone after the capture; (4) the moat's damage in a
  game of an RoE-format map and of an SoD-format map in the Complete edition (the game's version,
  §9.4). Increment 3's, not now: the battle in the open beside a garrison hero, T-5.

## 11. The town's income (T1b2a)

### 11.1 A town's gold (`town_gold_income` 0x5BF600, read whole)

Arguments: the town, whether to count the Resource Silo. From the built set +0x150 (by
`building_bits` 0x66CD98): **500**; the Town Hall (11) **1 000**; the City Hall (12) **2 000**; the
Capitol (13) **4 000**, tested in that order, the last one found kept, so the highest hall pays (the
Village Hall, 10, is not tested: every town has it, §5.3). With the silo flag and the Resource Silo
(15) built, the gold of the silo's table (`town_silo_resources` 0x688EB4, seven i32 per faction,
read as an i16 at +24) is added: **0 for every faction** (the table gives wood and ore to Castle,
Necropolis, Stronghold and Fortress, crystal to Rampart, gems to Tower, mercury to Inferno and
Conflux, sulfur to Dungeon; `town_silo_income` 0x5C1280 returns its row). With the Grail building
(26) in the every-built set +0x158, **+5 000**. The faction plays no other part: halls and Grail
pay the same in every town. At most 9 000.

### 11.2 Who is paid, and when

`player_gold_income` 0x4C7930 (E08 T77a2, `E08-economy.md` §8.2) adds, for every town of the
game's list (+0x21614, 0x168 bytes each) whose owner (+0x1) is the player, the town's gold as its
low 16 bits sign-extended (0x4C79ED; never wrapped below 32 768) and its garrison hero's own gold
(`hero_gold_income` 0x4E4390; garrison heroes are increment 3's). The day's income
(`player_daily_income` 0x4B8AF0) calls it with the silo flag 0 (0x4B8D92), so the silo's gold
would not count there anyway. Paid at each day's start from day 2 on to every player not defeated
(E08 §8.3): **a neutral town pays nobody, and a town captured during a day pays its new owner at
the next day's start.** The day's income pays the town's other incomes beside it (`E08-economy.md`
§8.1 point 2, not read): the silo's resources, Rampart's Treasury and Mystic Pond (increment 4).
Not followed: `town_gold_income`'s four other callers (the status window 0x4521F0 passes 1) and
`player_gold_income`'s two others (views and the computer's, probably).

### 11.3 The Capitol

It pays 4 000 as the highest hall. **One per player only after a capture**: `capitol_limit`
0x5BEC60 (§8.4) has two callers, `town_visit`'s capture paths (0x4AB1BA after an undefended
town's change, 0x4AB363 after a siege won). The start never applies it (`town_setup` calls
`town_change_owner` alone, §8.3), so a map may give a player several Capitols, each paying 4 000
(the wiki says so too); building a second one is increment 2's rule.

### 11.4 The census (the 160 maps; a throwaway crate over `h3_map::read`, not committed)

The 574 custom building sets (the map's bits through `map_building_ids` 0x6888C0: bits 0 to 2 the
halls, 9 the silo, 17 the Grail for every faction): the highest hall a Village Hall in 204 towns
(120 owned), a Town Hall in 258 (166), a City Hall in 89 (47), **a Capitol in 23 (10 owned)**; the
894 standard sets have the Village Hall only (§5.3). **No player holds two Capitols at the
start.** The Grail building in 10 towns, **2 owned** (`Good to Go`, colours 0 and 1). The
Resource Silo in 26 towns, **7 owned**: they would give their resources from day 2. Rampart's
special buildings (98, faction 1: bits 18, 19, 20 are its 17, 21, 22) only in neutral towns.

### 11.5 VCMI 1.7.5 and the wiki

- VCMI (`CGTownInstance::dailyIncome`, `config/buildingsLibrary.json`, `config/factions/*.json`):
  each built building that no built upgrade supersedes produces its resources: the halls 500,
  1 000, 2 000, 4 000, the Grail 5 000, the silo its faction's resources; then the owner's
  handicap (E10's). The same values; `removeCapitols` only from `setOwner`, a change in play.
- The wiki ([Halls](https://heroes.thelazy.net/index.php/Halls),
  [Grail](https://heroes.thelazy.net/index.php/Grail)): the same values; a captured Capitol
  becomes a City Hall when the captor already holds one, the only downgrade of a building; a map
  may give a player two or more Capitols; the Grail building's 5 000 in every town. Also (§12): a
  Capitol's town looks like a Fort with an extra flag on the map, even without a fort, and the hall
  level in the right-click town window is shown to allies only.

## 12. What a viewer sees of a town (T1b2a)

The town screen opens to its owner only (§7.2), and the status window shows the viewer's own
selected town only (`status_town_create` 0x4521F0, E24 `E24-design.md` §4.8). Another player
sees a town in four places.

### 12.1 The picture and the flag on the map

`object_template_redraw` 0x4C9990 (E24 §2.9): one of the faction's three pictures, the Capitol's,
else (a Fort, Citadel or Castle built) the fort's, else the bare one; the flag in the owner's colour,
grey for none. So whoever sees the town's tiles sees **its faction, its owner, whether it has a
Capitol and whether it has any fort** (the wiki: a Capitol's town looks fortified, fort or not).
The picture changes with the buildings (increment 2) and a captured Capitol's razing (§8.4).

### 12.2 The mouse-over and the cursor

`map_hint` 0x40B150's case 98 (E24 §4.10): the town's **name and faction**, for every viewer. The
cursor of a selected hero over a town of another team (`town_defended` 0x5BDFE0, E24 §4.10), once
the pathfinder reaches the tile: the attack cursor when it holds a visiting hero or any creature in
the army shown (the garrison hero's, else the town's), else the enter cursor; an unreached town the
arrow. So **whether a reachable town is defended is public**, as the step itself tells it (§7.1: a
defended town is visited from the tile before; the RoE manual: the sword shows a town its garrison
will defend).

### 12.3 The right-click (`town_popup` 0x4167A0, read but its debug branch; `town_quick_view_create` 0x530120 and `quick_view_army` 0x530990, read whole)

The right button on a map tile (`tile_click_path` 0x40A5D0, its branch 0x40A703 to 0x40A85C): on
a tile the viewer has not explored `object_popup_text` 0x4137C0 (not read); on an explored one by
the tile's class (the switch 0x40A796): 33 a garrison, 34 a hero, 54 a monster, 98 a town
(0x40A802), every other class `object_popup_text`. The town's popup shows while the button is held,
beside the pointer:

1. **The level** (0x416E63): **3** when the town's owner and the viewer are of one team (game
   +0x1F879), or when one of the viewer's heroes has **Visions at level 3 within reach** of the
   town: `hero_visions_level` 0x4E5DE0, the level of the Visions it cast that day (hero +0x129,
   set by the cast 0x41C490), raised to 3 by a **Rogue** (creature 143) in its army;
   `hero_visions_reach` 0x4E5E10, on the town's level with the squared distance below the
   radius's square, the radius Visions' factor for the level (spell 2's record +0x34) times the
   hero's power (1 to 99), at least 3. Else **the number of the viewer's towns with a Tavern**
   (`player_tavern_count` 0x4CCE30: the Tavern, 5, built, or for Castle the Brotherhood of the
   Sword, 22, that replaces it), **at most 2**. A neutral town is nobody's team.
2. **At every level** (`town_quick_view_create` 0x530120): the background `townqvbk.pcx`
   (194 × 186) in the owner's colour, a neutral town's in the viewer's; the portrait (`itpt.def`,
   `town_list_frame` 0x5BD700: the faction, a Fort ever built or not, one frame up when the town's
   +0x2 is set, E24's "built today"); the name; **the fort level** (`itmcls.def`: Fort, Citadel,
   Castle or none).
3. **At level 3 only:** **the hall** (`itmtls.def`, Village Hall to Capitol), the garrison hero's
   mark (`townqkgh.pcx`), the silo's resources (as the status window shows them) and **the gold
   income** (`town_gold_income` with the silo flag 1).
4. **The army shown** (`quick_view_army` 0x530990, of `town_garrison_army` 0x5C1460: the
   garrison hero's army with one, else the town's own): **level 0 nothing**; **level 1 each
   stack's creature** (at the places of 0x6823B8, packed: so how many stacks and of which
   creatures); **level 2 also each stack's size word** (`creature_quantity_word` 0x44ADE0, form
   0); **level 3 each stack's count** (below 10 000 itself, else the thousands and `k`). The
   visiting hero is not in it: it stands on the gate and is right-clicked as a hero.

The debug branch (the adventure manager's +0x3D, E24's debug flag) writes the garrison's names and
counts as a text for a town not the current player's; not a rule. Not followed: the town list's
right click (the popup's other caller) and the garrison's popup (class 33, E08's garrisons).

### 12.4 VCMI 1.7.5 and the wiki

- VCMI (`CGameInfoCallback::getTownInfo`, `InfoAboutTown::initFromTown`, `ArmyDescriptor`,
  `CGHeroInstance::hasVisions`, `config/creatures/neutral.json`, `config/spells/adventure.json`):
  **two levels**. Detailed for an own or allied town (`hasAccess`) or when **the selected hero**
  has Visions on towns within reach (expert Visions, or a Rogue: the factor 3 times the power, at
  least 3): the counts, the hall, the gold income, the silo, the garrison hero. Otherwise the
  stacks' creatures with their size words, the built set (the picture), the fort level, the name
  and faction, **whatever the viewer's taverns** (the original: nothing without a Tavern, the
  creatures with one, the words with two). The original asks every hero of the viewer, VCMI the
  selected one.
- The wiki ([Tavern](https://heroes.thelazy.net/index.php/Tavern),
  [Thieves' Guild](https://heroes.thelazy.net/index.php/Thieves'_Guild),
  [Visions](https://heroes.thelazy.net/index.php/Visions),
  [Halls](https://heroes.thelazy.net/index.php/Halls)): no Tavern, an opponent's town looks empty
  (the attack cursor still shows); one, the stacks without numbers; two or more, the stacks and
  their approximate numbers; expert Visions shows an enemy town's statistics and garrison within
  power × 3 or 3 tiles; the hall level in the right-click window for allies only, the fort level
  for all. The reading agrees on every point.
- **The census** (the 160 maps; the players owning a town at the start; a standard set gives an
  owned town a Tavern, §5.3; a custom set its bit 6, Castle's bit 19 the Brotherhood): **63
  players start at level 0, 600 at level 1, 78 at level 2.** So increment 1's views differ by
  player from the first day.

### 12.5 For the views (input for T2's design and `docs/06-security.md`)

- **Public** where the viewer sees a town's tile: its faction, owner, name, fort level (the
  right-click's, a finer fact than the picture's fort mark), whether it has a Capitol; whether it
  is defended, for a town the viewer's hero reaches (the cursor; a step tells it in any case).
- **The army** by the viewer's level: none, the creatures, the creatures and size words (the size
  words of E08's monsters, `docs/06-security.md`'s increment 3); the counts at level 3. The level
  comes from the viewer's teams, its taverns (increment 1 builds no Tavern, but the start gives
  them) and Visions or a Rogue in reach (Visions is not placed yet, E08's "Not placed yet"; a Rogue
  can join an army). A choice for the checkpoint: the original's levels, VCMI's two, or the
  owner's team only until increment 3 (the Tavern's information is increment 3's, decision 6).
- **The owner's team only** (and level 3): the hall, the gold income, the garrison hero, the silo.
- **Increment 2's:** the portrait's "built today" frame (town +0x2) shows another player's
  construction of the day.

## 13. Batch TWN-1's rows (T1b2a)

Gathered for T2's cut and T3's specs (the rows of §6 and §10 and the income's); run as E08's
batches, on the client lane's harness in SoD from maps `h3-experiment` writes
(`docs/experiments/README.md`). T2 settles the runs and the order; the checkpoint may drop a row
with the point it settles.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| TWN-1-1 | TD-3 (§5.3) | standard sets: owned towns without a Fort, owned ones with a Fort, neutral ones with and without (several of each, or several new games) | each town's built set at day 1 | owned without a Fort: a Tavern, no dwelling; with a Fort: the level-1 dwelling, the level-2 one about 30 in 100; neutral: both dwellings (VCMI: level 1 always, level 2 at 50 in 100, every town) |
| TWN-1-2 | TD-5 (§5.2) | a neutral town with an explicitly empty custom garrison; a neutral town without one (a control) | each garrison at day 1 | the first stays empty (VCMI draws one); the control drawn by the four rolls |
| TWN-1-3 | TD-7 (§9.2) | a lone visitor of seven stacks of mixed values in Blue's town with a garrison of mixed values; Red besieges | the defending stacks in the battle's opening; the garrison right after the battle | the merge by greatest gain, ties the first garrison stack and the last weakest slot (VCMI: most power first, one round per garrison stack) |
| TWN-1-4 | §8.2, §8.3 | Red besieges and wins against a defended neutral town; then a later step onto the gate | after the battle: the attacker's tile and points, the owner, a town screen or none; the garrison | the owner changes, the attacker stays on the tile before with its points, no screen; the garrison emptied (the merge's leftovers lost, T1b1's rows 2 and 3); the screen at the later step |
| ~~TWN-1-5~~ (dropped, §14) | §9.4 | the same siege on an RoE-format map and on an SoD-format map, played in the Complete edition | the moat's damage to a stack ending in it | the scenario dialog's version: the highest set (an RoE map may play under SoD rules, a damaging moat) |
| ~~TWN-1-6~~ (dropped, §14) | §11.1, §11.2 | Red owns at load a town with a City Hall, one with the Grail building and a Village Hall, a Castle with a Resource Silo, and two with a Capitol each; a neutral Capitol | Red's treasury at days 1, 2 and 3 | no change on day 1; +2 000, +5 500, +500 gold and +1 wood and +1 ore (the silo), +8 000 from day 2; the neutral one pays nobody |
| ~~TWN-1-7~~ (dropped, §14) | §8.4, §11.3 | Red (holding the two Capitols of row 6) enters the undefended neutral Capitol town | its hall after the capture; Red's treasury the next day | Red keeps both of its Capitols; the captured one becomes a City Hall (+2 000 from the next day) |

**Not rows:** the views (§12: the reading and the wiki agree on the tavern levels, the hall for
allies and expert Visions; VCMI is the one that differs); T-2 to T-4 (state-corrupting or empty
cases, fixed by proposal); T-5 and the battle in the open beside a garrison hero (increment 3);
TD-1, TD-2, TD-4, TD-6 (no shipped map affected or a one-point chance; the checkpoint decides).

**What the harness and the spec lack** (to confirm at T2 against `docs/experiments/README.md`):
reads of a town's record (owner, built set, garrison, the garrison hero) and of a player's
treasury between turns; the battle's opening stacks at a siege (E04c's siege runs read them); the
writer's towns (`h3_map::writer::Town`) have an owner, a garrison and either building set, but no
name, events or spells, and the map's format for row 5 (the writer writes SoD's; an RoE-format
map is new).

**Open for T1b2b's checkpoint (`TWN-Q2`; answered in §14)**, beside the queue's list (the random hero in increment
1, §4.4; T-2 to T-5; TD-1 to TD-7 where the choice is the user's; §6's and §10's open points):
(a) **the views' army levels** (§12.5: the original's tavern levels, VCMI's two, or the owner's
team only until increment 3); (b) **the Resource Silo's resources in increment 1** (7 owned towns
start with one: paid from increment 1, with Rampart's Treasury and Mystic Pond left to increment
4, or all of it in increment 4 with the 7 towns' resources missing until then); (c) the batch's
rows (§13), any the user would rather record as disputes.

## 14. The checkpoint's answers (T1b2b, `TWN-Q2`)

Asked interactively on 2026-11-02 (three rounds); the decisions are the epic's 8 to 16, the
record `docs/lanes/town/QUESTIONS-ANSWERED.md`. What each settles here:

| Point | Answer | Decision |
|---|---|---|
| The random hero (§4.4, finding 2) | increment 1's, by the main town's draw | 8 |
| The starting-hero pick (§4.3, §6's open point) | in the setup beside the faction, as the original's offer and use (the human test kept), reserved before any draw | 9 |
| The Resource Silo (§11.1, §13 (b)) | paid from increment 1; its table is increment 1's content | 10 |
| The views' army levels (§12.5, §13 (a)) | the original's: by the viewer's taverns, team, expert Visions or a Rogue | 11 |
| T-1 to T-5 (§3, §6, §10) | as proposed: T-1 and T-4 typed, T-2 and T-3 fixed, T-5 faithful in increment 3 | 12 |
| TD-1, TD-4, TD-6 (§3, §6) | the reading | 13 |
| TD-3, TD-5, TD-7 (§6, §10) | the reading, confirmed by rows 1 to 3 | 13, 15 |
| TD-2 (§3) | **the enabled expansions, not the map's version** (E08 decision 104): with AB or SoD enabled a neutral random town draws from nine on any map; an RoE-format map's player with all eight allowed may be Conflux; with RoE alone, eight. The original's map-format test is a deviation | 14 |
| Batch TWN-1's rows (§13 (c)) | rows 1 to 4; 6 and 7 dropped (the sources agree), 5 dropped (decision 104 decides the moat; §9.4's reading is the original's source) | 15 |
| Allied towns (§7.1, §10's open point) | the step's team test: an own or allied town entered, never captured; a siege of a town held by an ally's visitor a typed refusal (a stub) until E10 | 16 |

Still open for later increments, unchanged: §6's guild spells and Shipyard spot (increment 4),
§10's town screen effects, the computer's visit and the scenario's tests (E10, E13), town +0x2
and +0x32 (not read), the external dwellings' counts (increment 3).

## 15. Batch TWN-1 scored (T3c, 2026-11-03)

Both whole runs of T3b (the capture harness under Wine, the Complete 4.0 executable in SoD on
Easy, hot seat; `experiments/results/E09-batch-1.md`, the main record, and `-run2.md`): six new
games for rows 1 and 2, rows 3 and 4 played twice. Counted over both records by a throwaway script
(not committed). **Rows 2 to 4 as the reading in every point; row 1's dwellings as the reading,
and building 23, which the reading does not build, in three factions' towns.**

- **TWN-1-1 (TD-3, §5.3).** 34 towns, six games each. Owned without a Fort (8): {5, 10} each time,
  no dwelling in 48 readings (VCMI's level 1 always: refuted). Owned with a Fort (16): {5, 7, 10,
  30} each time, the level-2 dwelling (31) in 3, 4, 4 and 4, 4, 2 a game, **21 of 96** (95 %
  interval 0.14 to 0.32; against the reading's 30 in 100 two-sided p = 0.09, low but inside;
  VCMI's 50: p ≈ 10⁻⁸; HotA's "always": refuted). Neutral (10): {10, 30, 31} without a Fort,
  {7, 10, 30, 31} with one, in all 60 readings (VCMI's level 2 at 50 in 100: refuted).
  Every-built equal to built in every town. **TD-3: the reading, confirmed.**
- **Building 23 (not predicted).** Built in every Rampart, Necropolis and Conflux town of the
  standard set (11 towns, owned and neutral, with and without a Fort: 66 readings of 66), in none
  of the other six factions' 23 towns. §5.3's reading of `town_buildings_setup` (read whole)
  builds no 23, so it comes from a part of the setup's path not read: `town_build`'s hordes
  (by faction; `town_build` 0x5BE930 read in part), `town_setup_spells` 0x5BE600 (§5.1),
  or the faction tables of `building_tables_init` 0x4EB730. **VCMI 1.7.5** has no building 23 in
  these factions (`config/factions/rampart.json`, `necropolis.json`, `conflux.json`:
  `"special4" : null`, as Castle's and Fortress's; 23 is the Tower's Wall of Knowledge, the
  Inferno's Order of Fire, the Dungeon's Academy of Battle Scholars, the Stronghold's Hall of
  Valhalla) and drops every built id its faction does not define (`CGameState::initTowns`, the
  comment "#1444"); its town screens of the three draw an extra animation without a building
  (`TBRMEXT2`, `TBNCEXT2`, `TBELEXT1`), but so do Castle's and Fortress's, so that alone does not
  explain it. The wiki and §5.3's other sources say nothing. **Its weight:** the keep's count
  (`town_turret_count` 0x5BF4F0: 4 + the built buildings among the town's mask +0x160,
  `docs/formats/installation.md` §7.27; design §6's `structures`) counts 23 if the town's
  buildable set holds it: the keep's count one higher (a Castle's two towers take half of it,
  rounded up) in every siege of such a town from day 1. Whether a custom set gets it too is open (the batch's only
  custom set is a Castle). **The observation stands** (`docs/04-rules-and-sources.md` §1: a
  reading of behaviour ranks with an observation, and one that contradicts it is read again);
  the re-read needs the executable: in T5b (`TWN-Q5`, the epic's decision 25).
- **TWN-1-2 (TD-5, §5.2).** The two explicitly empty custom garrisons (NEF, NE) empty in all six
  games, 12 of 12 (VCMI's draw leaves a town empty 0.312 of the time, all twelve 9 × 10⁻⁷).
  **TD-5: the reading, confirmed.** The eight controls: 48 draws, 15 empty (31 %; the reading's
  0.67² × 0.80 × 0.87 = 31.2 %), 46 stacks, each the faction's base creature of its level with a
  count inside its row's bounds, in level order from slot 0; levels 1 to 4 drawn 12, 18, 8 and 8
  times (15.8, 15.8, 9.6, 6.2 expected; TD-4's 13 or 14 in 100 not separable, as §13 planned).
  Every owned standard town's garrison empty (24 of 24; the 25th owned town is row 3's custom
  garrison).
- **TWN-1-3 (TD-7, §9.2).** Both runs alike: the opening [Swordsman 4, Halberdier 16, Marksman 10,
  Griffin 5, Cavalier 1, Monk 3, Pikeman 15], the garrison left [—, Centaur 4, Archer 5]: T3a's
  prediction of the reading exactly, each of its three points moving a stack: the last of the two
  equal weakest (the Cavalier into slot 4 and the 5 Pikemen out, not the Centaurs of slot 1), the
  first of the two equal gains (the Halberdiers before the Marksmen in round 2), the rounds until
  nothing gains (the 5 Pikemen back in round 4: Pikeman 15). VCMI's merge (Pikeman 10, the 5
  Pikemen left in the garrison) refuted. 3a, the defender's win: the visitor keeps the merged army,
  the leftovers stay in the garrison. **TD-7: the reading, confirmed.**
- **The siege's aftermath (§7.1, §8.2, §8.3; 3b and TWN-1-4).** After each siege won: the attacker
  on the tile it attacked from with 1 900 of its 2 000 points (the grass step paid, the neighbour
  visit's), the owner Red, no town screen, the garrison empty (3b's leftovers [Centaur 4, Archer
  5] lost), Blue's visitor off the map, the built set unchanged. Row 4's later step: onto the
  gate with 1 800 left, the town screen, the hero the town's visitor (the main record, read with
  another hero selected; run 2 read it while selected: −1, the harness's note, T3b). **As the
  reading.**

**For the design and the ADRs:** design §5 (TWN-SET-003, -004) and §10 (TWN-SIE-001, -002) stand
for the dwellings, the garrisons and the merge, and ADR-0017's amendment step 4 (the town draws)
is unchanged: building 23 involves no draw (built in every game alike). It reopens only §5's
standard set and §2's content (whether the three factions list 23, with what role, and whether
the keep counts it), no ADR step: T5b reads it again (`TWN-Q5`, decision 25).

## 16. The buildings' tables read (T5b1, 2026-11-03)

Input for T5b2's harvest (design §3, decision 23) and the answer to decision 25. **Sources**
(retrieved 2026-11-03): a code reading of the user's `Heroes3.exe` (SHA-256 `aac39e8c…`), every
address in `docs/exe/map.toml`. **Read whole:** `building_tables_init` 0x4EB730 and its helpers
`building_list_read` 0x4EBC50, `supersede_list_read` 0x4EBB10, `supersede_table_fill` 0x4EBB70,
`horde_table_init` 0x5BDF60; `town_build` 0x5BE930; `town_turret_count` 0x5BF4F0;
`town_buildings_setup` 0x5C08C0 again at its masks. **Tables dumped** (a throwaway script, not
committed): the faction lists 0x63EC80, the supersede lists 0x63FBC4, `map_building_ids` 0x6888C0,
`town_creatures` 0x6747B4, `town_hordes` 0x6887A0, `horde_buildings` 0x642E20, the silo
0x688EB4, the neutral garrison 0x688E84 to 0x688EA4. The text files `BldgNeut.txt`,
`BldgSpec.txt`, `Dwelling.txt` of both archives (first column only; the SoD and AB names are
equal). VCMI 1.7.5's `config/buildingsLibrary.json`, `config/factions/*.json` and
`lib/mapObjects/CGTownInstance.cpp` (`getTownLevel`, the towers' damage), compared by a
throwaway script.

### 16.1 The tables at start-up (`building_tables_init` 0x4EB730)

- **The faction lists** (0x63EC80 to 0x63FBC4, nine lists of i32): per building its id, then the
  ids it requires, then -1; a list ends with -100 after a -1. Read into **the faction's mask**
  (`faction_buildings` 0x6976F0, 64 bits a faction: every id listed) and **the requirements**
  (`building_requirements` 0x697798, 8 bytes a faction and building, 44 a faction: the listed ids
  and, transitively, their own, so a list names a building after what it requires). Factions 0 to 2
  inline, 3 to 8 through 0x4EBC50. Counts: Castle 35, Rampart 37, Tower 36, Inferno 37,
  Necropolis 36, Dungeon 36, Stronghold 34, Fortress 34, Conflux 35. **No list holds the Village
  Hall (10), the ship (20) or 27 to 29**; 23 only in the Tower's, the Inferno's, the Dungeon's and
  the Stronghold's (their fourth special building).
- **The supersede lists** (0x63FBC4 on, 0x24C bytes, the same layout): `superseded_buildings`
  0x6A8BB8 per faction is the common list (0x63FBC4: each mage guild level the one below, Citadel
  the Fort, Castle the Citadel, each hall the one below, 19 the horde 18, 25 the horde 24, each
  upgraded dwelling its dwelling) and then the faction's, each entry adding the superseded one's
  own set (transitive): Castle (0x63FCA0) the Brotherhood 22 the Tavern 5, the horde 18 its level-3
  dwelling 32, 19 both 39 and 18; Rampart (0x63FCCC) 18 the dwelling 31, 19 38 and 18, 24 the
  dwelling 34, 25 41 and 24, the Fountain of Fortune 21 the Mystic Pond 17; Tower (0x63FD14) 18
  31, 19 38 and 18; Inferno (0x63FD34) 18 30, 19 37 and 18, 24 32, 25 39 and 24; the five others
  (0x63FD70 to 0x63FDF0) 18 30, 19 37 and 18. **A horde supersedes its level's dwelling.**
- **The hordes** (`horde_table_init` 0x5BDF60, the tail call): `town_hordes` 0x6887A0 holds per
  faction four records of 8 bytes, {creature i32, growth i16, level i16}, for the horde 18, its
  upgrade 19, the horde 24, its upgrade 25 (`horde_buildings` 0x642E20); the data give the two
  base creatures and growths (Castle's griffins +3; Rampart's dwarves +4 and Dendroid Guards +2;
  the Tower's gargoyles +4; the Inferno's imps +8 and hell hounds +3; Necropolis' skeletons +6;
  the Dungeon's troglodytes +7; the Stronghold's goblins +8; the Fortress' gnolls +6; Conflux's
  pixies +10), the start-up fills each level (the creature's index among the faction's 14 in
  `town_creatures`), the upgrade's creature and its level (index + 7). A faction without a second
  horde keeps level 0 there (never read: 24 is not in its list). The growth is increment 3's.

### 16.2 Building a building (`town_build` 0x5BE930, read whole)

On the town's standing set +0x150 (the every-built set +0x158 is not touched): (1) the building
in, every building it supersedes out. (2) For each of the four horde records with level L: (a)
the building is the record's: the level's dwelling (30 + L) out, and for a base horde (L below 7)
with the level's upgraded dwelling (37 + L) standing, the building and 37 + L out and **the
upgraded horde** in; (b) the record's building stands and the building is 37 + L: both out, the
upgraded horde in. So a horde turns into its upgrade when the level's dwelling is upgraded,
whichever comes first. (3) A dwelling 30 to 36: the town's available creatures of its level
(+0x16, i16 a level, 14 of them) set to the creature's growth (the creature record +0x44); an
upgraded one 37 to 43: the base level's count moved to the upgrade's, the base's 0 (increment 3).
(4) **Rampart, Necropolis and Conflux only** (faction 1, 4, 8; the switch 0x5BEC01): building 10
also builds 23, 11 builds 27, 12 builds 28, 13 builds 29 (`hall_companions`, through
`town_build` again).

### 16.3 Building 23 (decision 25's three questions)

- **Where it comes from:** the Village Hall's companion (§16.2 (4)). `town_buildings_setup` builds
  the Village Hall through `town_build` for every town before it branches (0x5C0938), so every
  Rampart, Necropolis and Conflux town has 23 from the start, a standard set's and **a custom
  set's** alike (batch TWN-1's 66 of 66, §15, and none in the other six factions: as read). A
  custom set's Town Hall, City Hall or Capitol (§5.3's id order) brings 27, 28, 29 the same way.
- **The faction's list:** none of 23, 27, 28, 29 is in the three factions' lists (§16.1), so none
  is in the buildable mask +0x160, **the keep never counts them** (§16.4) and they cannot be built
  by a player. No supersede list names them, so all four stay standing as the halls rise (a
  Capitol's town stands 13, 23, 27, 28, 29); `capitol_limit` takes only 13 out.
- **Readers:** the only loads of 23's bit outside the setup are faction-tested (the Tower's Wall
  of Knowledge at a visit, 0x5BD8E0, and the computer's value of a town's visit buildings,
  0x52B1E0, for factions 2, 3, 5, 6); none of 27 to 29's bits is loaded anywhere. Loops over every
  id use the faction's mask (the keep). **They are the town screen's pieces** (BldgSpec's row for
  23 is blank in the three factions; VCMI draws the Village Hall's piece without a building and
  27 to 29 as automatic buildings, `extraTownHall` and its two).

### 16.4 The keep's count (`town_turret_count` 0x5BF4F0 with the setup's mask)

4 + the ids 0 to 43 both in the standing set and in the mask +0x160; the mask is written only by
`town_buildings_setup` (0x5C0AB0 for a custom set, 0x5C0BFC for the standard set; elsewhere only
whole-town copies) as the faction's list less the forbidden. So **the Village Hall never counts**
(a town of the standard set counts its Fort, Tavern and dwellings, not its hall), nor the hall
companions, nor the ship; a chain counts once while its top stands (a horde for its dwelling, an
upgraded dwelling for its base). Design §6's `structures` ("the standing buildings not
forbidden") counts the record's Village Hall: one too many in every town without a Town Hall.
VCMI 1.7.5 has another model (a fixed shooter count; the damage grows with `getTownLevel`, every
built building that upgrades nothing, the Village Hall and 27 to 29 included), not comparable; the
battle lane follows the original's count (ADR-0023 `structures`).

### 16.5 The standing set and the build order (T-6)

The design's record keeps the ever-built set and derives the standing one by the closure of
`supersedes` (§4). Against `town_build`'s path: (a) a horde and the level's upgraded dwelling
(either order, in play or in a custom set) stand as **the upgraded horde alone** in the original;
the closure leaves the horde and the upgraded dwelling standing (two for the keep, the original
one) unless the setup adds the upgraded horde as `town_build` does. (b) **T-6 (quirk candidate):**
a custom set with a horde builds in id order the horde (18 or 24) before its dwelling (30 + L,
which the horde's supersede set brings in), so the dwelling's later build removes nothing: **both
stand** and the keep counts one more than the same buildings built in play (dwelling first, the
horde then superseding it), and the town screen draws both. No state corruption; a +1 in the
siege's turrets of such a town. The census of custom sets with a horde is T7a's.

### 16.6 The other tables and the roles

- **`map_building_ids` 0x6888C0** (41 slots a faction, 44 none): slots 0 to 9 the Town Hall,
  City Hall, Capitol, Fort, Citadel, Castle, Tavern, Blacksmith, Marketplace, Silo; 11 to 17 the
  mage guilds 1 to 5, the Shipyard 6, the Grail 26; 22 to 40 the dwellings by level (base,
  upgrade) and the hordes; 10 and 18 to 21 and 24, 27, 30, 33, 36 the faction's specials and
  hordes, by faction (Castle 18 → 17, 19 → 22, 20 → 21; the Tower 10 → 17, 18 → 22, 19 → 23,
  20 → 21; Conflux 10 → 17, 18 → 21). No slot gives the Village Hall, the ship, an upgraded horde
  (19, 25: a forbidden horde forbids its upgrade, §5.3) or 27 to 29.
- **`town_creatures` 0x6747B4:** every faction seven levels, each with its upgrade (the base
  creatures at 0 to 6, the upgrades at 7 to 13; the Fortress' levels 3 to 5 the serpent fly, the
  basilisk, the gorgon; Conflux's the pixie, then the air, water, fire, earth and psychic
  elementals, the firebird).
- **As read before:** the silo (§11.1, 0x688EB4), the neutral garrison (§5.2), the level-2
  dwelling's 30 in 100 (§5.3, 0x5C0C5B).
- **The roles** (`h3-pack-towns`' format; §5.3, §8.3, §9.4, §11.1): `hall` 10 to 13; `fort` 7 to
  9; `tavern` 5 and the Castle's Brotherhood 22 (it supersedes the Tavern); `silo` 15 and `grail`
  26 in every faction; `dwelling` 30 to 36 (`[level, false]`), 37 to 43 (`[level, true]`);
  `morale` 5 +1, the Castle's 22 +2; `luck` the Rampart's Fountain of Fortune 21 +2;
  `escape_tunnel` the Stronghold's 17; `defender` the Tower's Grail +150 mana, the Inferno's
  Brimstone Stormclouds 21 +2 power, the Dungeon's Grail +12 power, the Stronghold's Grail +20
  attack, the Fortress' 21 +2 attack, 22 +2 defense (TD-8) and Grail +10 attack and defense;
  `reveal` the Tower's Lookout Tower 21 radius 20 and its Grail `all`.

### 16.7 Names

`BldgNeut.txt` rows 1 to 15 name ids 0 to 14 and row 17 the Blacksmith (16); `BldgSpec.txt` has
11 rows a faction in the order 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 15 (the silo's name),
then 11 generic rows; `Dwelling.txt` 14 rows a faction, 30 to 36 then 37 to 43 (the upgrades
named "Upg. …"), then 14 generic. **Every listed building has a name**; 23 is blank in the three
factions (and 27 to 29 have no row). The horde and its upgrade share a name in every faction
(Griffin Bastion twice, …), so their keys need a suffix (T5b2's call); Conflux's Grail is spelled
"Aurora Borealias". The Village Hall's name is row 11 (the record needs it: `hall` 1).

### 16.8 VCMI 1.7.5 compared

- **The lists:** the same building ids in every faction, plus the Village Hall (10, automatic)
  everywhere, the ship (20) with a Shipyard (Castle, Necropolis, Fortress, Conflux), and 27 to 29
  (automatic, from the Town Hall, City Hall, Capitol) in Rampart, Necropolis and Conflux; no 23
  there (§15).
- **The requirements:** the same in every faction (VCMI adds the superseded building as one,
  e.g. the Town Hall on the Village Hall), but the upgraded hordes (19, 25: the original's need
  the level's upgraded dwelling only, VCMI's the horde too; VCMI builds them automatically, as
  `town_build`), the Necropolis' Unearthed Graves (18: the Skeleton Transformer, VCMI also the
  level-1 dwelling, which the Transformer requires anyway), and **the Conflux's upgraded level-4
  dwelling** (40: the original's the Altar of Fire and the upgraded level-2 dwelling 38, VCMI's
  the Altar of Fire and the base level-2 dwelling 31). Increment 2's (requirements are not
  harvested now; its survey takes TD-9 up there).
- **The supersedes:** the same, but the upgraded horde supersedes the horde in the original (not
  in VCMI's `upgrades`, which name one building) and VCMI's ship upgrades the Shipyard.
- **TD-8:** the Fortress' 21 and 22 as the code adds them (+2 attack, +2 defense: VCMI's
  `special2` and `special3` the same); the game's descriptions (`BldgSpec.txt`, the Glyphs of Fear
  +2 defense, the Blood Obelisk +2 attack) and the fandom wiki's Fortress page the reverse;
  `docs/formats/installation.md` §7.27 names 21 the Blood Obelisk (the names swapped there, the
  values right). Only a town with the Glyphs and without the Obelisk differs (22 requires 21).

### 16.9 Disputes, quirk candidates and open points (T5b1)

| Id | What | Reading | VCMI | Others | Increment |
|---|---|---|---|---|---|
| TD-8 | The Fortress' Glyphs of Fear (21) and Blood Obelisk (22) for a siege's defender | 21 +2 attack, 22 +2 defense | the same | the game's texts and the fandom wiki: 21 defense, 22 attack | 1 (the `defender` role) |
| TD-9 | The Conflux's upgraded Altar of Fire (40) | needs the Altar of Fire and the upgraded Altar of Air (38) | the base Altar of Air (31) | not checked | 2 (requirements) |

- **T-6** (§16.5): a custom set's horde leaves its dwelling standing (the keep +1).
- **For the checkpoint (`TWN-Q6`):** the hall companions in the part or not (§16.3); the keep's
  count without the Village Hall (§16.4: a field of the building, or the derivation); the
  standing set stored as the original's or derived (§16.5, T-6's class); TD-8.
- **Open for T5b2:** the keys (the hordes' shared names, the harvested names' spelling); the
  synthetic records; how `h3-harvest` reads the executable's lists (an `h3-formats` reader of the
  addresses above, as its hero and siege tables).
