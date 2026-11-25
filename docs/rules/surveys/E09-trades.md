# E09 trades survey (E09 T52): the Marketplace's window at a town, the Artifact Merchants, the Blacksmith, the Shipyard, the Magic University, the Skeleton Transformer

Date: 2026-11-24. Scope: E09 increment 6 (the town's trades; the epic's decisions 197 and 198),
input for T53's ADR amendment drafts, the design note's Part H and the cut. **T52 whole** (the
user at its start, decision 200): every window, the census, the comparison, batch TWN-6's rows
and the checkpoint `TWN-Q35` in one docs-only pull request, the readings, the comparison and the
census done by sub-agents one narrow job each. Input, not rules (`README.md`): a fact becomes
binding when a task writes it into the catalog with its sources and tests.

Terms: a town's **trades** are the windows its owner (or a visiting hero) trades at: the
**market window** (`market_window_run` 0x5EA130) with its five **screens** (0 resources for
resources, 1 resources to another player, 2 resources for artifacts, 3 artifacts for resources,
4 creatures for resources; survey `E08-trades.md` §3), opened at a town by the **Marketplace**
(building 14), the **Artifact Merchants** (the Tower's, the Dungeon's and the Conflux's building
17) and the **Freelancer's Guild** (the Stronghold's 21); the **Blacksmith** (16) and the
Stronghold's **Ballista Yard** (22); the **Shipyard** (6) with its **ship** (20, a boat at the
town's **spot**); the Conflux's **Magic University** (21) and the Necropolis's **Skeleton
Transformer** (22). The **count** *m* is the number of Marketplaces the rates read
(`market_count` 0x6AAA98). The **stock** is the Artifact Merchants' seven artifacts (game
+0x1F664). A town's **visitor** is its visiting hero (+0x10), its **garrison hero** the one in
the garrison (+0xC). Building ids are the executable's (0 to 43); the packs' keys and the h3m's
bits are named where they differ (§1).

**Sources** (retrieved 2026-11-24):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml`. The readings follow the
  human player's path (the town screen's building clicks, `town_screen_proc` 0x5D3240's switch
  at 0x5D34DA; the new game's setup; the new day) and name the computer's where it differs (§8).
  Each section names what it read whole and in part. **Data:** `ARTRAITS.TXT` (both archives),
  `GENRLTXT.TXT`, `BldgSpec.txt`, `TCOMMAND.TXT` (`extracted.tar`).
- VCMI 1.7.5 (GPL-2+, tag `1.7.5`, commit `6901839ae976`), read from the tag's sources (a sparse
  clone of `lib`, `server`, `config`, `client/windows`): `server/CGameHandler.cpp`
  (`sendResources`, `sellArtifact`, `buyArtifact`, `sellCreatures`, `buildBoat`, `buySecSkill`,
  `transformInUndead`, `onNewTurn`), `server/NetPacksServer.cpp` (`visitTradeOnMarketplace`),
  `lib/mapObjects/IMarket.cpp` (`getOffer`), `lib/mapObjects/CGTownInstance.cpp`
  (`getMarketEfficiency`, `availableItemsIds`, `getOutOffsets`, `isWarMachineAvailable`),
  `lib/callback/GameRandomizer.cpp` (`rollMarketArtifactSet`, `rollArtifact`),
  `lib/mapObjects/IObjectInterface.cpp` (`getBoatCost`, `bestLocation`),
  `config/buildingsLibrary.json`, `config/factions/*.json`, `config/artifacts.json`,
  `config/creatures/*.json`, `config/gameConfig.json`; `client/windows/CCastleInterface.cpp`
  (which hero the town screen passes).
- The wiki (heroes.thelazy.net): `Marketplace`, `Artifact_merchants`, `Blacksmith`, `Shipyard`,
  `Skeleton_Transformer` (https://heroes.thelazy.net/index.php/<page>). The pages
  `Freelancer's_Guild`, `Ballista_yard` and `Magic_university` did not load (the site's
  robots.txt unreadable over six tries in 15 minutes); the Blacksmith's page names the Ballista
  Yard's sale. The wiki's HotA lines (Cove, Factory, Bulwark) are outside the engine's scope.
- The census: the 160 maps of the installation (§1), a throwaway example over `h3_map::read`
  and `h3_map::import`, not committed.

## 0. Findings in short

1. **The census** (§1): 1 468 towns on the 160 maps (592 of a fixed faction, none a Conflux; 876
   random). Built at the start: the Marketplace in 93 fixed and 53 random towns, the Blacksmith in
   69 and 61, the Shipyard in 9, the Artifact Merchants in 25 (SoD maps; 29 in the built game), the
   Freelancer's Guild and the Ballista Yard in 1, the Skeleton Transformer in 3, the Magic
   University in none; every town with the merchants or the guild has its Marketplace. No player
   starts with more than 2 Marketplaces (9 players on 3 maps). A boat spot: 18 of 119 Castles, 5 of
   58 Fortresses, no Necropolis, 27 random towns; none occupied at the start. 51 maps have teams;
   2 ban the four magic schools; no map bans an artifact of ids 0 to 6.
2. **The entries** (§2.1): the Marketplace (14) opens for a human at an own town, with a visitor
   or without; the Artifact Merchants (17 of the Tower, the Dungeon and the Conflux) and the
   Freelancer's Guild (21 of the Stronghold) open only for the town's visitor (never the garrison
   hero) and test no owner, so at an ally's town they serve the current player's visitor.
3. **The count** (D-70 as read, §2.1): the current player's towns with the Marketplace, at most 10,
   whichever town is traded at; the town itself needs none. A count of 0 arises only at an ally's
   town (or a preset town with the merchants and no Marketplace: none on the shipped maps).
4. **The screens a town reaches** (§2.2): without a visitor resources for resources and the
   transfer; with a visitor the artifact screens at a Tower or Dungeon whose merchants stand (the
   Conflux's only from its merchants' building: the Marketplace's button names factions 2 and 5),
   the creature screen at a Stronghold whose guild stands. **At a count of 0 an artifact costs
   nothing** (T-19, an exploit); the resource and creature trades divide by zero there (inferred:
   nothing tradable).
5. **Resources to another player** (§2.3): any undefeated other colour, ally or enemy, 1:1, up to
   the whole stock; the count plays no part; the recipient is not told.
6. **Artifacts sold** (§2.4): a worn artifact of positions 0 to 12 or any backpack artifact, one
   a trade, at E08's rate (at least 1 unit); the war machines and the spell book refused; Misc 5
   never offered (T-22); the Grail sellable (1 unit); a combination whole, at the sum of its
   parts. After the backpack's arrows moved, the price is one place's and the artifact removed
   another's (T-20, an exploit).
7. **Creatures sold** (§2.5): the map's guild at the player's count instead of 5 (C[*m*]: the whole
   gold cost from 8 Marketplaces), the visitor's army, the last creature kept.
8. **The stock** (§3): seven artifacts for the whole game (three treasures, three minors, one
   major), drawn last at the new game by the random artifacts' picker (`random_artifact_of_level`
   0x4C94D0: E08's ADV-OBJ-006 with its marks, D-32 settled as the code's) and redrawn whole on
   the first day of every month from the second, sold or not, with merchants anywhere or not; a
   purchase empties the entry for every town and player until then; a loaded game keeps it. A
   picker that finds every artifact of the four classes banned loops for ever (T-24).
9. **The combination artifacts' prices** (§3.4): blank in `ARTRAITS.TXT`, set at start-up to the
   sum of their parts' costs (Angelic Alliance 84 000 down to Wizard's Well and Ring of the Magi
   3 000); they reach the stock only through the picker's fallback.
10. **The Blacksmith** (§4): one machine by faction (the Ballista at the Castle, the Dungeon and the
    Conflux; the Ammo Cart at the Tower, the Inferno and the Stronghold; the First Aid Tent at the
    Rampart, the Necropolis and the Fortress) at its creature's gold cost (2 500, 1 000, 750: data),
    to the visitor only, refused (Buy disabled) to a hero who carries it or a player short of gold,
    no owner test; every cost paid as gold (T-25). **The Ballista Yard** (22) is a Castle's
    Blacksmith in a Stronghold: the Stronghold sells both machines.
11. **The Shipyard** (§5): the spot fixed at the new game (two rows below the entrance, one column
    left, else right); a boat for 1 000 gold and 10 wood (code), built for the viewing player at
    any time of the turn, no hero, no owner test, no day mark. **The spot's occupancy is never
    tested**: the ship (20), built while a boat or a hero stands at the spot and refreshed only when
    the town's screen is built, is drawn over the dock and takes its clicks, except 1 pixel in a
    Fortress and 2 to 4 in a Conflux, where a boat is bought over the docked one (T-21, T-27).
12. **The Magic University** (§6): Fire, Air, Water and Earth Magic, fixed, for the visitor, else
    the garrison hero (a selected garrison stack picks the garrison hero); the map's University's
    slots (a skill known, a class weight of 0, eight skills), 2 000 gold each from the hero's owner,
    Basic, several a visit; **a school the map bans is offered and taught** (T-23); no hero: the
    building's description.
13. **The Skeleton Transformer** (§7): the visitor's army, else the town's (the garrison hero's, or
    the garrison's without a hero); whole stacks, the count kept, free, any number of times; the
    eight dragons and both hydras of the table become Bone Dragons, every other creature but the
    Skeleton becomes Skeletons (Skeleton Warriors, Bone and Ghost Dragons too); the last stack
    allowed; no merging. In a timed game the expiry loses the stacks inside the dialog (T-26).
14. **The computer** (§8): its own paths (E31's readings): the Tower's and Dungeon's merchants only,
    the Blacksmith and the Ballista Yard, the University, boats on its walk, the market's trades
    written straight into its treasury; never the Transformer.
15. **VCMI 1.7.5 and the wiki** (§9): VCMI agrees on the modes, the factors, the transfer, the
    sale's refusals, the stock's shape, timing and sharing, the combinations' prices, the guild,
    the Blacksmith's machines and the Ballista Yard, the boat's price, the University's offer and
    price, the Transformer's targets; it differs on **D-70's rounding**, the hero at the merchants,
    the guild and the Blacksmith (the garrison hero too: **TD-28**), the count at an ally's town
    (the owner's: **TD-29**), the University's banned schools (**TD-30**) and the boat's spot (two
    tiles tried at each build: **TD-31**). The wiki's rate table agrees with the code's D-70 rows
    (37, 62, 112), its Shipyard with the one spot, its merchants' exploit with T-19.
16. **The checkpoint** (§13, `TWN-Q35`; decisions 201 to 207): D-70 the original; TD-28
    `fix-optional` (T-28); TD-29 the trader's count at least 1 (T-19 fixed); TD-30 with T-23
    `fix-optional`; T-22 fixed (Misc 5 sellable); TD-31 the original; the agent's calls each
    accepted; batch TWN-6's eight rows (§11).

## 1. The census (the 160 maps of the installation; a throwaway example over `h3_map::read` and `h3_map::import`, not committed)

These counts cover the 160 files in `original-data/install/Maps`: 159 `.h3m` and `Tutorial.tut`, which is RoE 49, AB 55 and SoD 56. Every file reads with `h3_map::read`, and every map imports whole with `h3_map::import` under the three original packs. The counts were made by a throwaway example `crates/h3-map/examples/t52_census.rs`, since deleted. Cells read RoE/AB/SoD = all, or give the total alone when the split adds nothing.

**Building mapping.** Each engine pack key maps to the town record's `Building::slot` (`TownRecord::building_by_key(key)` then `.slot`). The slot is the h3m's built/forbidden bit:

- `marketplace` 8 and `blacksmith` 7 in all nine factions.
- `shipyard` 16 in Castle, Necropolis, Fortress and Conflux.
- `artifact_merchants` 10 in Tower, Dungeon and Conflux.
- `freelancers_guild` 19 and `ballista_yard` 20 in Stronghold.
- `magic_university` 18 in Conflux.
- `skeleton_transformer` 20 in Necropolis.

The original building ids (Marketplace 14, Blacksmith 16, Shipyard 6) do not follow the h3m order. "Built" means the custom set's built bit is set and its forbidden bit is clear (`town_buildings_setup`: what the set picks, less what it forbids). A random town's bits are read per slot: slots 7 and 8 are the same building in every faction, while 10, 16, 18, 19 and 20 count only if the drawn faction has that building.

### 1.1 Towns

How: objects of class 98 (fixed; the faction is the template subclass) and class 77 (random). A town is owned when `TownBody.owner` is not `0xFF`. `TownBody.buildings` is either `Standard` (the default set) or `Custom`.

| towns | RoE/AB/SoD = all |
|---|--|
| all towns | 349/555/564 = **1468** |
| fixed Castle | 38/30/51 = 119 |
| fixed Rampart | 29/24/30 = 83 |
| fixed Tower | 25/17/27 = 69 |
| fixed Inferno | 21/32/27 = 80 |
| fixed Necropolis | 15/7/10 = 32 |
| fixed Dungeon | 36/31/22 = 89 |
| fixed Stronghold | 23/22/17 = 62 |
| fixed Fortress | 23/14/21 = 58 |
| fixed Conflux | **0** |
| random (class 77) | 139/378/359 = **876** (owned 562, neutral 314) |
| owned at the start / neutral | 257/323/309 = **889** / 92/232/255 = **579** |
| default set / custom set | 278/266/350 = 894 / 71/289/214 = 574 |

### 1.2 The buildings at the start

How: each fixed town of a faction that has the building, by `Buildings::Custom { built, forbidden }` at the building's slot. Towns with `Buildings::Standard` count as default. No town has a bit both built and forbidden.

| building | fixed towns | default set | built (owned) | forbidden | custom, neither |
|---|--:|--:|--:|--:|--:|
| marketplace | 210/177/205 = 592 | 326 | 19/27/47 = **93** (41) | 1 (RoE) | 172 |
| blacksmith | 592 | 326 | 18/28/23 = **69** (32) | 1 (SoD) | 196 |
| shipyard | 76/51/82 = 209 | 115 | 3/0/6 = **9** (1) | 4 (RoE) | 81 |
| artifact_merchants | 61/48/49 = 158 | 83 | 0/0/25 = **25** (8) | 0 | 50 |
| freelancers_guild | 62 | 45 | 1 RoE (1) | 0 | 16 |
| ballista_yard | 62 | 45 | 1 RoE (1) | 0 | 16 |
| magic_university | 0 (no fixed Conflux) | – | – | – | – |
| skeleton_transformer | 32 | 24 | 1/0/2 = 3 (1) | 2 (AB) | 3 |

The same counts by faction, over all versions. Each cell is built/forbidden by custom sets; "–" means the faction lacks the building.

| faction | towns | owned | default | custom | market | smith | shipyard | art. merch. | freelancers | ballista yard | transformer |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| Castle | 119 | 53 | 62 | 57 | 20/1 | 18/0 | 7/2 | – | – | – | – |
| Rampart | 83 | 42 | 51 | 32 | 9/0 | 7/1 | – | – | – | – | – |
| Tower | 69 | 35 | 40 | 29 | 16/0 | 6/0 | – | 14/0 | – | – | – |
| Inferno | 80 | 56 | 32 | 48 | 10/0 | 8/0 | – | – | – | – | – |
| Necropolis | 32 | 18 | 24 | 8 | 5/0 | 5/0 | 0/2 | – | – | – | 3/2 |
| Dungeon | 89 | 50 | 43 | 46 | 18/0 | 9/0 | – | 11/0 | – | – | – |
| Stronghold | 62 | 41 | 45 | 17 | 5/0 | 6/0 | – | – | 1/0 | 1/0 | – |
| Fortress | 58 | 32 | 29 | 29 | 10/0 | 10/0 | 2/0 | – | – | – | – |
| random (by slot) | 876 | 562 | 568 | 308 | 53/1 | 61/1 | 0/8 | 8/4 | 19: 0/0 | 20: 0/0 | 20: 0/0 |

Random towns by version: default 123/201/244, custom 16/177/115. Slot 8 built 4/17/32. Slot 7 built 6/23/32. Slot 16 forbidden 2/1/5. Slot 10 built 3/2/3 and forbidden 0/4/0. Slot 18 (the University) is never built or forbidden.

Engine cross-check: in the built game (`GameState::new` on the import, seed 0, random towns resolved), the counts are:

- **Marketplace:** built in 148 towns (64 owned). That is the 93 fixed plus the 53 random from the map's sets, plus 2 more: Fortress town 0 in `Resource War` and in its allied copy. Both have a default set and a town event on file day 0 (game day 1) that builds slot 8, and the built game has that building at the start.
- **Blacksmith:** built in 130 towns (55 owned), which is 69 plus 61.
- **Shipyard:** built in 9 towns; forbidden in 591 of the 625 towns of a shipyard faction, because they have no spot.
- **Artifact Merchants:** built in 29 towns.
- **Magic University:** 73 towns of its faction (random towns drawn as Conflux), none built.

Side finding: 32 fixed towns (7/20/5) have a City Hall or Capitol (slot 1 or 2) built without the Marketplace that the City Hall requires. The engine builds only the set's slots (with what they supersede), so these towns start without a Marketplace.

### 1.3 Players starting with two or more Marketplaces

How: for each playable colour (human or computer), the towns owned at the start whose slot 8 is built, fixed or random. The built game agrees.

| players | RoE/AB/SoD = all |
|---|--|
| playable / owning a town at the start | 226/253/265 = 744 / 741 |
| with 0 / 1 Marketplaces | 691 / 15/4/25 = 44 |
| **with 2 or more** | 0/8/1 = **9**, on **3 maps**; **the most is 2** |

The nine players: `Hoard` and `Hoard(Allies)` (AB), colours 0, 1, 3 and 4, each with 2 Marketplaces in its 2 towns; `Hatchet Axe and Saw` (SoD), colour 0, 2 in 2 towns.

### 1.4 Shipyard factions' towns and their boat spots

How: the static layer (`import_map`) was built into a `GameState` the way `original_data`'s check does it, with one placeholder town record per town. The spot is `StaticMap::town_boat_spot(TownId)` (TWN-SET-011). The engine gives every town a spot test, whatever its faction; the importer itself has no spot. Boats are `MapSetup.boats[].tile`. A hero's tile is its object's visitable tile, moved left onto a town's entrance as TWN-SET-010 does.

| towns | total | with a spot | without |
|---|--:|--:|--:|
| Castle | 119 | 7/1/10 = **18** (7 with a Shipyard built) | 101 |
| Necropolis | 32 | **0** | 32 |
| Fortress | 58 | 2/0/3 = **5** (2 with a Shipyard built) | 53 |
| Conflux (fixed) | 0 | – | – |
| random | 876 | 10/13/4 = **27** | 849 |
| other factions (no Shipyard) | – | 12/3/2 = 17 | – |

- **Custom sets building a Shipyard without a spot: 0.** All 9 built Shipyards have one.
- **Spots holding a boat at the start: 0. Spots holding a hero: 0.** No hero starts on water; ADV-SEA-007 would refuse it, and every map imports.

### 1.5 Teams

How: `RawMap.teams` (present when the map defines alliances); the team codes in `of_player` are counted over the playable colours only.

| teams | RoE/AB/SoD = all |
|---|--|
| maps with teams | 12/19/20 = **51**; every one has a team of 2 or more players |
| teams of 2 or more players | 22/43/48 = **113** |
| largest team | 5 (RoE, AB), 3 (SoD) |

### 1.6 Bans

How: `RawMap.banned_artifacts` bits 0 to `Version::artifacts()` (none in RoE), and `banned_skills` bits 0 to 27 (SoD only; Fire, Air, Water, Earth are codes 14 to 17).

- **Artifacts:** all 111 AB and SoD maps ban at least one artifact, up to **21** (and 7 at most in AB). Most of that comes from the editor's defaults: 127 (Vial of Dragon Blood) on 100 maps, 128 (Armageddon's Blade) on 99, the SoD combination artifacts 129 to 140 on all 56 SoD maps, and the unused codes 141 to 143 on 41 maps.
- **The maps' own bans** (codes below 127) appear on **26 maps**, at most **6** (`A Viking We Shall Go` and its allied copy). The most banned are 72 Angel Wings (13 maps), 90 Boots of Levitation (9), 87 Tome of Air Magic (7), 71 Necklace of Ocean Guidance and 123 Sea Captain's Hat (5 each). **No map bans codes 0 to 6**: the spell book, scroll, Grail, Catapult, Ballista, Ammo Cart and First Aid Tent.
- **Secondary skills:** **19 SoD maps** ban at least one, up to 13. **2 maps ban Fire, Air, Water or Earth Magic**: `Heroes of Might not Magic` and its allied copy ban all four schools (13 skills each).

### 1.7 Heroes holding a war machine at a town with a Blacksmith

How, from the map's data: placed heroes are objects of class 34 or 70. A hero is on or in a town when its tile (as in §4) is the town's entrance or a blocked tile. A Blacksmith is slot 7 built. The war machine comes from the map's artifacts (`HeroBody.artifacts`, else SoD's `custom_heroes` entry), any of codes 4 to 6. A hero type's own starting war machine (`HeroType::war_machines`) is drawn with the second slot's 88% chance, and only when the army is not customised (GAM-SET-001). A main town's hero means `generate_hero` (always in RoE).

| heroes | RoE/AB/SoD = all |
|---|--|
| placed heroes on or in a town / with a Blacksmith | 9/14/0 = 23 / 3/10/0 = **13** |
| …holding a machine by the map's artifacts | **0** |
| …of a fixed type with a starting machine, army not customised (88%) | **1** (`The Newcomers`, AB: Pyre, Ballista) |
| …of a random type, army not customised (depends on the draw) | 2 (RoE) |
| main towns generating a hero / with a Blacksmith built (drawn type) | 181/181/176 = 538 / 2/1/16 = **19** |

The built game at seed 0 (all 160 maps):

- 822 heroes are on the map, 96 of them holding a Ballista, Ammo Cart or First Aid Tent.
- 32 stand on a town with a Blacksmith.
- **1** of those holds a machine: `Battle of the Sexes` (SoD), a main town's hero drawn as Christian (Ballista), so it depends on the seed.
- Pyre in `The Newcomers` did not get his Ballista at seed 0 (the 12% case).

### 1.8 Merchants or guild without a Marketplace

How: custom sets only, with built meaning the built bit is set and the forbidden bit clear. For fixed towns, slot 10 counts as Artifact Merchants in Tower, Dungeon and Conflux, and slot 19 as the Freelancer's Guild in Stronghold. Random towns are read by slot: 10 or 19. A town qualifies when one of these is built and slot 8 (the Marketplace) is not.

- **Towns with the Artifact Merchants or the Freelancer's Guild built at the start: 34.** That is 25 fixed Artifact Merchants (all SoD), 1 fixed Freelancer's Guild (RoE) and 8 random towns with slot 10. No random town has slot 19 built.
- **Of these, with the Marketplace not built: 0.** Every one of the 34 has its Marketplace built too, so no owner's Marketplace count is 0 at such a town. This holds whether the town is neutral or owned.

## 2. The Marketplace's window at a town

Path followed: the human player's clicks in the town screen (`town_screen_proc` 0x5D3240's building
switch), the three town entries, `market_window_run` 0x5EA130 and the screens' create, redraw and
procedure functions. Not followed: the computer's trades (`ai_market_trades` 0x42AC20,
`ai_pay_with_trades` 0x42A470, E31), network play (the transfer's network message), the tutorial,
the game version (no test of `[game]+0x1F698` met in these functions). The window, its globals, the
map objects' entries, the rates and screens 0, 1, 2 and 4's procedures are E08's
(`E08-trades.md` §3 to §6, §11); the rates are cited from §4 there, not re-derived.

### 2.1 The entries and the count (D-70)

**Read:** `town_screen_proc` 0x5D3240 in part (the switch head 0x5D3490 to 0x5D34DA, cases 14
0x5D3869, 15 and 16 0x5D38D7 to 0x5D3963, 17 0x5D3CDC to 0x5D3DF0, 21 0x5D3963 to 0x5D3B38),
`town_marketplace_trade` 0x5E9FE0 whole, `town_artifact_merchants_trade` 0x5E9D20 and
`town_freelancers_guild_trade` 0x5E9EA0 in part (their no-hero branch; the rest E08's reading),
`player_is_human_both` 0x4BAD80 (E08's). `callers`: each town entry has exactly one call site, all
in `town_screen_proc` (0x5D389C, 0x5D3D62, 0x5D3B0A); `market_window_run` is reached only through
the six entries (two calls, four tail jumps).

- **A right click** on any of these buildings (the message's right-button flag, bit 0x200 of its
  +0xC, kept in the frame at 0x5D3317) opens nothing: 14 and 17 go to the screen's common building
  popup (0x5D3F53, not read), 21 shows the building's description in a popup (0x5D396E). The rest
  is the left click.
- **The Marketplace (14, 0x5D3869):** refused, silently, unless the current player's record is a
  human (`player_is_human_both`) **and** the town's owner (+0x1) is the current player
  (0x5D388A); then `town_marketplace_trade`. **No hero is needed:** the window opens on screen 0
  with `market_hero` the town's visiting hero (+0x10), or none.
- **Special 1 (17, 0x5D3CDC), by the town's faction (+0x4):** Tower (2), Dungeon (5) and Conflux
  (8) call `town_artifact_merchants_trade` (their 17 is the Artifact Merchants: BldgSpec.txt's
  rows 22, 55, 88); every other faction gets its building's description in an OK box
  (`town_building_description` 0x5D2A40 with 17, the faction's picture; 0x5D3D06): the Castle's
  Lighthouse, the Rampart's Mystic Pond, the Necropolis's Cover of Darkness, the Stronghold's Escape
  Tunnel, the Fortress's Cage of Warlords; the Inferno has no building 17. **No owner test, no
  human test.**
- **Special 2 (21, 0x5D3A10):** the Stronghold (6) calls `town_freelancers_guild_trade` (its 21 is
  the Freelancer's Guild, row 70), the Conflux (8) its Magic University (0x5D2DA0, §6); the other
  factions the description box. **No owner test either.** No other building id and no other case
  of the switch calls an entry (the callers above); 15 (the Silo) and 16 (the Blacksmith, §4) open
  no trade window.
- **Without a visiting hero:** the Marketplace opens (screens 0 and 1 only, §2.2); the merchants
  and the guild show GENRLTXT 274 ("only a visiting hero may use" the building, with the faction's
  name of building 17 or 21) in an OK box and open nothing (0x5E9D28, 0x5E9EAA).
- **The garrison hero never trades:** each entry reads only the town's +0x10 (the town of the
  town screen, `town_manager_pointer` +0x38; the guild's entry gets the town as `this`). Unlike the
  Grail's build and the spell book's purchase, which take the garrison hero when there is no
  visitor (`E09-guild.md` §13).
- **The count, `market_count` 0x6AAA98 (D-70 as read):** the current player's record's towns
  (count +0x3E, town indexes +0x40; an index -1 would make it read address 0x158, never met)
  whose every-built set +0x158 holds building 14 (`building_bits` +0x70), at most 10
  (0x5EA009 to 0x5EA063; the merchants' and the guild's entries the same loop). So:
  - it is the **current player's** count, not the town owner's: an ally's town's Marketplace never
    counts, and at an ally's town (below) the visitor trades at its own player's count;
  - a town counts from the build on (`town_build_effects` recomputes +0x158 at every build,
    `E09-construction.md` §4); no building supersedes the Marketplace (the supersede lists the
    packs hold, from 0x63FBC4), so +0x150 and +0x158 always agree on 14: the question "in +0x158
    but not +0x150" has no case;
  - the current town needs no Marketplace of its own: the entries never test it. The Marketplace's
    click needs the building (it is the building clicked); the Artifact Merchants and the
    Freelancer's Guild require the Marketplace to be built (the packs' `requires`), so a town
    built up in play always counts itself. A map's preset town with the merchants or the guild and
    no Marketplace (not checked whether the setup adds requirements) would trade at the player's
    other towns' count, 0 when none.
  - **m = 0** is possible only there or at an ally's town: every factor table's index 0 is 0
    (`market_resource_rates` 0x678344, `market_artifact_rates` 0x678370, `market_creature_rates`
    0x67839C); §2.2's consequences. Every count from 1 to 10 is reachable at a town, so D-70's rows
    (2, 4, 8 Marketplaces) are the town's.
- **All three entries** set `market_artifacts` 0x6AAA74 to the game's seven (+0x1F664, §3) and
  kind 0; the first screen 0 (the Marketplace), 2 (the merchants), 4 (the guild).
- **An ally's town:** an own or allied town's step opens the town screen for a human
  (`E09-towns.md` §7.1); there the Marketplace is refused but the merchants (17) and the guild (21)
  open for the current player's visiting hero, with the current player's count and treasury.

### 2.2 The screens a town reaches

**Read:** `market_window_run` 0x5EA130 whole; the redraws' button tests:
`market_resources_redraw` 0x5EA6E0 (0x5EAAC0 to 0x5EABA6), `market_transfer_redraw` 0x5EAF50
(0x5EB2B1 to 0x5EB325), `market_buy_artifact_redraw` 0x5EB6A0 (0x5EB9F7 to 0x5EBAA6),
`market_sell_artifact_redraw` 0x5EBE80 whole, `market_sell_creatures_redraw` 0x5EC550
(0x5EC915 to 0x5EC969); the creates' screen buttons (`market_transfer_create` 0x5E16C0,
`market_sell_artifact_create` 0x5E5730, `market_sell_creatures_create` 0x5E7C80, in part;
`market_resources_create` 0x5DF5F0 E08's); the procedures' button tables (each screen's button
16 + n ends the screen with screen n).

- **`market_window_run` tests nothing of the town:** it opens the screen in `market_screen`,
  clears the selection (given, wanted, amount, the traded flag and screen 3's backpack offset
  `market_backpack_offset` 0x6AAA70), runs the dialog and takes its result as the next screen
  (0x7802, the close button, ends). Which screens a town reaches is the buttons' doing.
- **The buttons each screen has** (made always; at kind 0 shown or hidden as below):

  | Screen | Its buttons to | Tested |
  |---|---|---|
  | 0 resources | 1 (17), 2 (18), 4 (20) | 18 and 20 |
  | 1 transfer | 0 (16), 2 (18) | 18 |
  | 2 artifacts bought | 0 (16), 1 (17), 3 (19) | hidden only for kinds 2 and 3 |
  | 3 artifacts sold | 0 (16), 1 (17), 2 (18) | never (no kind test in the screen) |
  | 4 creatures sold | 0 (16), 1 (17) | hidden only for kind 3 |

- **Button 18 (to screen 2)** is shown at screens 0 and 1 only when the kind is neither 1 nor 3,
  `market_hero` is set, the town screen's town is of faction **2 or 5** and building 17 stands
  (its built set +0x150, `building_bits` +0x88) (0x5EAAC0, 0x5EB2B1). **Button 20 (to screen 4)**
  at screen 0 only with the same kind and hero tests, faction **6** and building 21 standing
  (+0x150, +0xA8; 0x5EAB38). Screen 1 has no button 20, screen 4 none to 2 or 3 (its redraw's
  hiding of 18 and 19 for kind 3 meets no such item).
- **So at a town (kind 0):**
  - **No visiting hero:** screens 0 and 1, from the Marketplace only (the other entries refuse).
  - **A Tower or a Dungeon with the Artifact Merchants and a visitor:** from the Marketplace or
    the merchants every screen but 4 (0 ↔ 1, 0 or 1 → 2, 2 ↔ 3, 2 or 3 → 0 or 1).
  - **A Conflux with the merchants and a visitor:** the Marketplace's window stays on 0 and 1 (the
    faction test lacks 8); the merchants' window opens on 2 and reaches 3, 0 and 1, but from 0 or
    1 there is no way back to 2 or 3 (only 3's button 18 leads to 2). Close and click the
    merchants again.
  - **A Stronghold with the Freelancer's Guild and a visitor:** 0, 1 and 4 (0 → 4 by button 20,
    4 → 0 or 1; from 1 to 4 only through 0); from the guild the same.
  - Every other town, or a town whose merchants or guild is not built: 0 and 1.
- **No town reaches both** the artifact screens and screen 4 (one faction each). Screen 3 is
  reached only through screen 2, so only at a town (kind 0; a Black Market hides 19). The screens'
  titles: screen 0 and 1 GENRLTXT 159 ("Marketplace") at kind 0; screen 3 and 4 the names of
  BldgSpec's rows 22 and 70 whatever the town.
- **For the engine** the screens are the client's: the rule is which trades a town's window
  offers: resources for resources and the transfer at the owner's Marketplace, with or without a
  hero; the artifact purchase and sale with a visitor in a Tower, Dungeon or Conflux whose Artifact
  Merchants stand; the creature sale with a visitor in a Stronghold whose Freelancer's Guild
  stands (the Conflux's missing button is a path of the UI, not a rule; §10).
- **At m = 0** (§2.1): the artifact purchase's factor is 0, so its price is 0
  (`market_buy_artifact_proc` 0x5ED9E0 takes the zero branch, 0x5EDC08 and 0x5EDCB1: price 0, most
  0, amount 1 since the player has at least 0): **the artifact is given for nothing**; a sale
  pays 1 unit an artifact (§2.4); the resource trade and the creature sale divide by a zero value
  (an infinite q, its rounding the x87's integer indefinite: a slider's most of 0, inferred: run
  with exceptions masked, not observed); the transfer is unaffected.

### 2.3 Resources to another player (screen 1)

**Read:** `market_transfer_proc` 0x5ED550 whole; `market_window_run`'s case 1 (0x5EA270 to
0x5EA358, the recipients' list); `market_transfer_redraw` 0x5EAF50 in part (the button test and
the recipients' items, 0x5EB42C to 0x5EB65B); `market_transfer_create` 0x5E16C0 in part (its
buttons and items).

- **The recipients:** the list (the dialog's +0x60 count, +0x64 the colours) holds, in colour
  order 0 to 7, every colour other than the current player's whose defeat flag (game +0x1F636 +
  colour) is clear (0x5EA305 to 0x5EA338): **allies and enemies alike** (no team test), **never
  the player itself**, never a defeated player (and never a colour nobody plays: the new game sets
  its flag, `E08-trades.md` §5). The redraw shows one row per entry (the colour's name and flag)
  and enables only those rows' press items 70 to 76; the rest are hidden and inert (0x5EB525).
- **The trade:** a resource (items 28 to 34) and a recipient (70 to 76); then the rate 1, the most
  **the player's whole stock of that resource** (`market_most`; the slider from 0 to it,
  0x5ED74A); Max (7) sets the most. Trade (5) with an amount of 0 does nothing; else the current
  player's resource less the amount and the recipient's (its player record, game +0x20AD0 +
  0x168 × colour, +0x9C) plus the amount (0x5ED5C4 to 0x5ED60E), then the selection cleared.
  **The count of Marketplaces plays no part** (the rate is fixed 1:1, the same at the Trading
  Post), nor the hero (screen 1 needs none).
- **The recipient is not told:** no message, no event on the human path; only in network play a
  network message to a human recipient (0x5ED610, not followed). The giver's treasury is not
  re-tested at Trade (the slider's most is the bound, as E08 §3 says of every screen).
- Its buttons: to screen 0 (16, always) and 2 (18, §2.2's test); Max, Trade, the close.

### 2.4 Artifacts sold (screen 3)

**Read whole:** `market_sell_artifact_proc` 0x5EDF60, `market_sell_artifact_redraw` 0x5EBE80,
`market_sell_artifact_slot` 0x5EA5D0 (one position's drawing), `market_artifact_sale_rate`
0x5ECDC0 (again), `hero_remove_artifact` 0x4E2BD0, `hero_remove_backpack_artifact` 0x4E2D50.
**In part:** `market_sell_artifact_create` 0x5E5730 (its items and buttons),
`dialog_picture_message` 0x450550 (a picture passes a press only with its state bit 2, then
`dialog_area_message` 0x44FF60, whose press rule is E24's reading).

- **What is shown:** 23 positions: 0 to 17 the hero's worn positions 0 to 17 (`market_hero`
  +0x12D + 8 × position), 18 to 22 five places of the backpack (+0x1D4), each a picture (84 +
  position) under a frame (107 + position) that takes the press. With fewer than six backpack
  artifacts the five are the backpack's first five; with six or more they start at the backpack
  offset (`market_backpack_offset` 0x6AAA70) and wrap around (modulo the count), and the arrows
  130 and 131 (greyed below six) move the offset by one either way (0x5EE151, 0x5EE293). **The
  fifth misc position (18, SoD's) is never shown**, so an artifact there cannot be sold (the
  window has 18 worn positions).
- **What can be picked** (`market_sell_artifact_slot` sets the frame's press bit only for a
  position holding an artifact; `dialog_area_message` passes a press only with that bit): an empty
  position never. **Positions 0 to 12** (head, shoulders, neck, both hands, torso, both rings,
  feet, misc 1 to 4) and **every backpack place** are picked. **Positions 13 to 17** (the four war
  machines and the spell book) answer GENRLTXT 22 ("this item can't be traded") in an OK box
  (0x5EE555) and are not picked. A right click shows the artifact, except on position 17 (the
  book: nothing).
- **Not tested** by any path: the artifact's id, class or cost, a quest's need, the Grail. So the
  **Grail** (2, carried only in the backpack, `E08-army.md` §5) is sold; any artifact of cost 0 or
  a blank cost sells for **1 unit** (the rate's minimum, §4 of `E08-trades.md`); a combination
  artifact sells whole from its own position (its parts' other positions hold no artifact, so they
  are not pickable) at its own +0x4 cost: blank in ARTRAITS.TXT but filled at start-up with the sum of its parts'
  costs (§3.4), so priced as any artifact; 1 unit only for 141 to 143 (never placed).
- **The trade:** an artifact position and a resource (items 63 to 69), in either order; each pick
  computes the rate (`market_artifact_sale_rate`, or the same arithmetic inline at 0x5EE47D) and
  sets the amount and its most to **1**: one artifact a trade. Trade (5): the current player's
  resource plus the rate (0x5EE09C to 0x5EE0B9), then the artifact removed: a worn one by
  `hero_remove_artifact` (its primary skills and its parts' subtracted, the parts' locked positions
  freed, the available spells recomputed by `hero_available_spells` 0x4D95D0 when it or a part
  carries spells, +0x1D), a backpack one by `hero_remove_backpack_artifact` (the later ones moved
  down, the count less one); the selection cleared, the screen redrawn. The window stays on
  screen 3 for more sales; Max (7) does not exist here.
- **The price and the removal can name different backpack artifacts** (T-20, §10):
  every rate (the pick 0x5EE4C0, `market_artifact_sale_rate` 0x5ECDE7, the recomputation after a
  scroll 0x5EE219 and 0x5EE356, the redraw's price labels 0x5EC3F5) reads backpack place
  *p* − 18 **without the offset**, while the removal (0x5EE0D8), the offer text (0x5EBEED) and the
  right click add the offset modulo the count, and the drawing adds it only with six or more. With
  the offset 0 they agree; once the arrows have moved it (six or more artifacts), the player is paid
  the price of backpack place *p* − 18 for the artifact shown at *p*; the offset stays after a sale,
  so after the backpack falls below six the drawing and the price name place *p* − 18 while the
  sale removes place (offset + *p* − 18) mod count. Either way an expensive artifact's price can
  be taken for a cheap one, repeatedly.
- **What a sale can leave the hero without:** never the spell book or a war machine (refused);
  anything else, including every artifact (no minimum). The spell points are not lowered by the
  sale; the skills and spells as above. The current treasury is only added to: no refusal by
  resources.
- **Refusals:** Trade disabled (greyed) until both an artifact and a resource are picked; empty
  positions inert; 13 to 17 refused with GENRLTXT 22; Misc 5 not offered. The scroll's
  recomputation does not test the selection (with none it reads position -1 and resource -1:
  harmless while Trade stays disabled).
- Its buttons: to screens 0, 1 and 2 (16, 17, 18; never hidden), Trade, the arrows, the close.

### 2.5 Creatures sold at the Freelancer's Guild (screen 4)

**Read:** `market_sell_creatures_proc` 0x5EE890 (again, for kind and town tests: none),
`market_sell_creatures_redraw` 0x5EC550 in part (the kind test 0x5EC915, the slots 0x5ECA91 to
0x5ECB96), `market_sell_creatures_create` 0x5E7C80 in part (its buttons), `market_creature_rate`
0x5ECE80 (again). `refs` of `market_kind` 0x6AAAC4: no create and no procedure of screen 4 reads
it; its redraw only to hide the buttons.

- **What differs from the map's guild (kind 3):** only (a) the buttons: at kind 0 the buttons to
  screens 0 and 1 (16, 17) stay shown (kind 3 hides them), and (b) the count: the town's entry
  counts the current player's Marketplaces *m* (§2.1), the map's 5. The procedure, the rate and
  the slots are the same (`E08-trades.md` §6). The hero is the visitor in both (the town's
  +0x10; never the garrison hero, none: no window, §2.1).
- **The army sold:** the visitor's seven slots (+0x91 creatures, +0xAD counts); a slot with a count
  of 0 is hidden and inert (0x5ECAB7). Any creature, any faction, level, upgrade, undead or not
  (no test). War machines are not in the army.
- **The last creature kept:** when the army holds one stack (`army_stack_count`), the most is that
  count less one (divided by round(q) when one creature is worth a unit or less), so the hero keeps
  at least one creature; a stack sold to 0 empties its slot (0x5EE9F1). Two stacks: the first can be
  sold whole, then the second keeps one.
- **The count's effect:** the creature factor C[*m*] (`E08-trades.md` §4: 0.3, 0.45, 0.5, 0.65,
  0.7, 0.85, 0.9, then 1 from 8 Marketplaces) against the map's C[5] = 0.7: at one Marketplace a
  creature sells for 30 % of its value, at 8 to 10 for **its whole gold cost**, rounded as §4
  (D-71's rounding holds at every *m*). At an ally's Stronghold with *m* = 0 nothing can be sold
  (§2.2).

## 3. The Artifact Merchants' stock

Path followed: the human player's (a new game, the new day, the town's window). Not followed: the
computer's purchase (`ai_*`, named for comparison only), network play, the tutorial (no test of it
touches the stock), the second caller of `new_game_heroes` (§12).

### 3.1 The new game's draw (`new_game_heroes` 0x4BFE70, 0x4C0477 to 0x4C04DB)

**Read whole:** `random_artifact_of_level` 0x4C94D0, `artifacts_given_init` 0x4C0AA0.
**In part:** `new_game_heroes` 0x4BFE70 (0x4C0330 to 0x4C04EF), `new_game_bans` 0x4C2450 (the
artifacts' bans, 0x4C24E0 to 0x4C2850), `artifact_table_load` 0x44CD50 (0x44CE40 to 0x44D0C0),
`hero_pool_init` 0x4C4EF4 (one site), `saved_game_load` 0x4BD83D and `saved_game_write` 0x4BEBDF
(one site each). A grep of game +0x1F664 to +0x1F67C, +0x4E224 and +0x4E2B4 for the writers.

- **Who writes the seven** (game +0x1F664, i32[7]): only `new_game_heroes` (0x4C0484 to 0x4C04DB)
  and `new_month` 0x4C8F40 (0x4C9248 to 0x4C929E) draw them; a purchase empties one through
  `market_artifacts` (§3.3); a saved game writes the 28 bytes as they are (`saved_game_write`
  0x4BEBDF) and its load reads them back (`saved_game_load` 0x4BD83D): **a loaded game keeps its
  stock**, nothing is redrawn at a load.
- **When:** at the end of `new_game_heroes`, which every new game runs (`new_game_load` 0x4C05E5
  from `game_main` and `new_game_start`; a campaign's scenario through the same path, only its
  version set differently, 0x4BFF24): so a scenario and a campaign's scenario draw alike.
- **Its place among the setup's draws** (the one random stream, `random_between` 0x50B230 over
  the C library's `rand()`): the map read and its bans (`new_game_bans`), the given marks
  (`artifacts_given_init` 0x4BFFF0), the random objects (`resolve_random_objects` 0x4C9DD0), the
  objects' setup (`setup_map_objects` 0x4C0CC0: the Black Markets' seven each, the chests', the
  corpses', the wagons', the tombs' and the other draws), then the per-colour loop with the start
  bonuses (its artifact case 0x4C035F gives the colour's first hero a treasure,
  `random_artifact_of_level(2)`), `refresh_guard_flags` (0x4C0442), the rumour's
  `random_between(1, 100)` (0x4C0451) and its pick, and **last the seven**: entries 0 to 2 a
  treasure (level bits 2), 3 to 5 a minor (4), 6 a major (8), in that order, one call each.
  No relic and no special is asked for.
- **The picker** (`random_artifact_of_level`, the Black Markets' too): it counts n, the records of
  the 144 (artifact table 0x660B68, 0x20 bytes) whose class (+0xC) has a bit of the level, whose
  disabled byte (+0x1C) is 0 and whose given mark (game +0x4E224, a byte per id) is 0; draws
  r = `random_between(0, n - 1)` (one `rand()`, none when n is 1); takes the r-th such record in id
  order and sets its given mark. **n = 0:** the given marks of the level's records (not disabled)
  are reset to their bans (game +0x4E2B4; 0x4C950E), and the count again; when that still leaves
  none (every artifact of the level banned), the level becomes 0x1E (treasure, minor, major and
  relic) for this call and the count again (0x4C9549).
- **What is excluded** (the class letters as `artifact_table_load` reads them at 0x44CF82: R 16,
  J 8, N 4, T 2, any other 1; SoD's table `H3bitmap/artraits.txt` has 37 T, 21 N, 39 J, 40 R and
  7 S):
  - **by the class:** the specials (S: 0 the spell book, 1 the spell scroll, 2 the Grail, 3 to 6
    the war machines) never; the relics (every combination artifact 129 to 140 among them) only
    through the fallback to 0x1E (below and §3.4). The parts of a combination have their own class
    (e.g. Armor of Wonder N, Ring of Vitality T, Dead Man's Boots J) and are drawn like any other.
  - **disabled** (+0x1C): 141, 142, 143 (`artifacts_disabled` 0x63B98C, set at 0x44CFFB).
  - **banned** (+0x4E2B4, `new_game_bans`): first a copy of +0x1C (0x4C24E0); the game's version
    (`map_kind`, game +0x1F698) RoE bans 127 to 143, AB 129 to 143 (0x4C2500): all relics, so the
    version never changes the stock's three pools; the map's own list by its format (game
    +0x1F86C): an RoE map (0xE) none, an AB map (0x15) 129 bits with 129 to 143 banned when a
    combination (+0x14 not -1; 0x4C261D), an SoD map 144 bits (0x4C2580), each OR'd in
    (0x4C2605); on an SoD map also every spell-granting artifact (+0x1D: 1, 86 to 89, 123, 124, 128,
    135) whose spells (`artifact_spell_set` 0x4D9350) hold a spell the map bans (0x4C27D9); the
    victory condition's artifact, kinds 0 and 10 (`hero_pool_init` 0x4C4EF4); an RoE Seer's Hut's
    artifact quest's artifact (`seer_hut_load` 0x5746DF) and, inferred, the artifacts of two more
    quest loaders (§12). A banned artifact is never drawn, even after a reset.
  - **already given:** the marks start as a copy of the bans, then every Artifact object (class 5)
    on the map marks its artifact (`artifacts_given_init`); then every artifact the picker drew:
    the random artifacts (classes 65 to 69), the setup's draws (the Black Markets' and the other
    objects'), the start bonuses and the earlier stocks. So at a new game the seven avoid the map's
    artifacts and the Black Markets' while the level's pool lasts.
  - **not excluded:** the artifacts a placed hero, a campaign's carried hero, a Pandora's Box, an
    event or a Seer's Hut's reward holds (no other writer of +0x4E224 than the three named, a
    saved game's load and a clearing of the game record, §12).
- **A repeat within the seven:** none while the level's pool lasts (each draw marked). A draw that
  finds its level's pool empty resets the level's marks from the bans: every non-banned artifact of
  the level is free again, the one just drawn, a Black Market's, a map's or a hero's included. So
  after a reset the seven can hold the same artifact twice, or one the map or a market holds.
- **The fallback:** a map that bans every treasure (or minor, or major) gets, for that entry, an
  artifact of any of the four classes, relics and combination artifacts included (on an SoD map
  that does not ban them). A map that bans every treasure, minor, major and relic makes the picker
  loop for ever (T-24, §10).

### 3.2 The month's draw (`new_month` 0x4C8F40, 0x4C90D0 to 0x4C929E)

**Read:** `new_day` 0x4C7FE0 (0x4C7FE0 to 0x4C8036), `new_month`'s end (0x4C90C0 to 0x4C92B7)
and its branches to it.

- **When:** `new_day` adds a day; unless the scenario has ended (`scenario_ended` 0x6972B8), a day
  above 7 becomes 1 with the week's events (`new_day_week_events` 0x4C8780 at 0x4C8017), and a
  week above 4 becomes 1 with `new_month` (0x4C8031). So **the first day of every month from the
  second on**, after the week's events; month 1's stock is the new game's (day 1 has no new day,
  GAM-CAL-002).
- **All seven redrawn:** `new_month` ends with the same seven calls in the same order (0x4C923B
  to 0x4C929E: three treasures, three minors, one major), each entry overwritten whether sold (-1)
  or not; nothing tests the old entries. The old seven's given marks are **not** cleared: an unsold
  artifact stays out of the pool until its level's next reset. The same picker and exclusions as
  §3.1; the new month's draws come last in `new_month` (then the view's redraw, 0x4C92AC).
- **The month's kind:** a creature month (`month_kind` 1, 0x4C90D0) first places its monsters and
  calls `refresh_guard_flags` (0x4C9236); every other kind (normal, the Plague; the tutorial's)
  jumps straight to the draw (0x4C90D7). No kind, week or day test skips or changes it. Only the
  stream position differs (a creature month's placement draws first).
- **Unconditional:** at a new game and every month, whether or not any town has the Artifact
  Merchants: the draws use the stream and drain the pools every other random artifact uses
  (seven a month: the 21 minors, less the map's and the markets', run out within a few months,
  then their reset frees them all again).

### 3.3 A purchase at a town

**Read:** `market_buy_artifact_proc` 0x5ED9E0 (0x5EDAD0 to 0x5EDB3B, the Trade), the stores of
`market_artifacts` 0x6AAA74 (its 17 references). Not re-read: the price (E08 §4, §7.2), the
drawing of an empty entry (E08 §7.2).

- **One stock for the game:** `town_artifact_merchants_trade` 0x5E9D20 (0x5E9D91),
  `town_marketplace_trade` 0x5E9FE0 (0x5E9FF0) and `town_freelancers_guild_trade` 0x5E9EA0
  (0x5E9F14) all pass game +0x1F664 as `market_artifacts` (the Marketplace's and the guild's
  windows reach screen 2 by its button, §2.2); `trading_post_trade` 0x5EA0C0 passes it too but its
  kind 1 never reaches screen 2 (§2.2). Every town of every player shows the same seven; a Black
  Market passes its own record instead (`black_market_trade` 0x5EA100).
- **The purchase:** Trade at screen 2 takes the price from the current player (0x5EDAEC), gives
  the entry's artifact to `market_hero` (`hero_give_artifact`, 0x5EDB11, its result not tested:
  A-46) and sets **the entry to -1** (0x5EDB22), in game +0x1F664 itself; the selection is
  cleared (0x5EDB2F, 0x5EDB35). An entry of -1 is drawn empty and cannot be picked (E08 §7.2).
  So a purchase in one town empties that entry for every town and player **until the next month's
  draw**; nothing refills it sooner. The marks do not change (the artifact was marked at its draw).
- **The computer** (for comparison): `ai_artifact_merchants_visit` 0x5259E0 passes the same stock
  (0x525BD0) to `ai_buy_stock_artifacts` 0x524400, which also sets the bought place to -1: the
  same shared effect (its own choices, the Tower and Dungeon test and its price: E31).
- **A sale at screen 3** (§2.4) never puts an artifact into the stock (no write of +0x1F664 there)
  nor back into the pool (no write of +0x4E224).

### 3.4 The combination artifacts' prices (`artifact_table_load` 0x44CD50, 0x44D031 to 0x44D0BB)

**Read:** `artifact_table_load` 0x44CD50 (the row's cost and class, 0x44CE40 to 0x44CFDF; the
combination pass, 0x44CFFB to 0x44D0BB), `combination_table_init` 0x44C960 (whole, its ids),
ARTRAITS.TXT of both archives.

- **ARTRAITS.TXT:** SoD's table (`H3bitmap/artraits.txt`) names 129 to 143 (the twelve
  combinations, then 141 to 143) with a **blank Cost** and class R; AB's (`H3ab_bmp/ArTraits.txt`)
  has the rows 129 to 143 empty. The loader reads the cost with the C library's `atoi` (0x44CEEE):
  a blank cost is **0**; +0x14 and +0x18 become -1 for every row (0x44CFC4).
- **The cost filled in at start-up:** after the rows, for each of the twelve combination records
  (`combination_setups` 0x6938D8: the combination's id and the set of its parts, built by
  `combination_table_init`), the combination's +0x14 becomes its index (0 to 11), its cost +0x4 is
  set to 0 and each part's cost is added; each part's +0x18 becomes the combination's index
  (0x44D031 to 0x44D0BB). So **a combination's cost is the sum of its parts' costs** (SoD's
  table):

  | Id | Combination | Parts | Cost |
  |---|---|---|---|
  | 129 | Angelic Alliance | 31 to 36 | 84 000 |
  | 130 | Cloak of the Undead King | 54, 55, 56 | 12 000 |
  | 131 | Elixir of Life | 94, 95, 96 | 20 000 |
  | 132 | Armor of the Damned | 8, 14, 20, 26 | 12 000 |
  | 133 | Statue of Legion | 118 to 122 | 25 000 |
  | 134 | Power of the Dragon Father | 37 to 45 | 42 000 |
  | 135 | Titan's Thunder | 12, 18, 24, 30 | 40 000 |
  | 136 | Admiral's Hat | 71, 123 | 25 000 |
  | 137 | Bow of the Sharpshooter | 60, 61, 62 | 12 000 |
  | 138 | Wizard's Well | 73, 74, 75 | 3 000 |
  | 139 | Ring of the Magi | 76, 77, 78 | 3 000 |
  | 140 | Cornucopia | 109, 110, 111, 113 | 20 000 |

  A blank cost stays 0 only for 141 to 143 (disabled, never drawn).
- **In the stock:** a combination is a relic, so only the fallback (§3.1: a map that bans every
  artifact of the entry's level, on an SoD map that does not ban the combination) can put one
  there; its price is then screen 2's for the sum above (E08 §4).
- **At screen 3** (§2.4): a combination sells whole at the same sum, through the sale's rate: not
  at the 1-unit minimum (this answers §2.4's question). The Black Market's draw is the same picker
  with the same levels, so the same holds there.

## 4. The Blacksmith with the Stronghold's Ballista Yard

Path followed: the human player's clicks on the town screen (`town_screen_proc` 0x5D3240's building
switch 0x5D34DA, the id the item's value - 1 at 0x5D34A5). Not followed: the computer's `ai_*`
path (pointers in §4.8 and §5.7), network play (the screen's branch at 0x5D328A, `create_boat`'s
message), the turn timer's branch at the procedure's start (0x5D3281). No tutorial or game-version
test lies on these paths.

Read: `town_screen_proc` 0x5D3240 at its click flag (0x5D3302 to 0x5D3317), its case 16 (0x5D3935
to 0x5D395E) and case 22 (0x5D3B38, the faction sub-switch 0x5D3B53, its eight cases to 0x5D3BF6);
`town_blacksmith_buy` 0x5D1D30 (read whole); `blacksmith_dialog_create` 0x5D1360 (read in part: the
price, the title, the buttons and their state, 0x5D13A0 to 0x5D1530, 0x5D1590 to 0x5D15C7 and
0x5D17FD to 0x5D19F7; the pictures' placement not read); `blacksmith_dialog_proc` 0x5D1C60 (read
whole); `blacksmith_dialog_right_click` 0x5D1AA0 (read in part); the tables
`blacksmith_war_machines` 0x6AA9F8 (E31's) and `blacksmith_machine_creatures` 0x642E90 (dumped);
the supersede lists of the Castle (0x63FCA0) and the Stronghold (0x63FDB0) dumped again
(`supersede_lists` 0x63FBC4, E09 T5b1).

### 4.1 The click

- **The flag** (0x5D330E): bit 9 of the message's +0xC, kept in the procedure's first-argument
  slot, is tested first by case 16 and by every case below. Set (the right button: the popup) the
  building's description (0x5D3F53, `town_building_description` 0x5D2A40, inferred; no rule);
  clear the building's action.
- **Case 16** (0x5D3935): `town_blacksmith_buy` 0x5D1D30 with the town's visiting hero (+0x10) and
  the town's faction (+0x4) (0x5D3940 to 0x5D394A), then the resource bar redrawn. No test of the
  town's owner, of a human player or of the building here (a building is clicked only where the
  screen draws it).

### 4.2 The hero

- **The visitor only.** With no visitor (+0x10 = -1) GENRLTXT 274 (only visiting heroes may use
  the building, its name the neutral building 16's as `town_building_name` 0x4610E0 reads it) in an
  OK box, and no dialog (0x5D1D4D to 0x5D1D8F). A garrison hero (+0xC) alone is never served:
  unlike the Mage Guild's spell book (`E09-guild.md` §13, the visitor else the garrison hero).
- **Its owner is not tested** (neither in the case nor in the dialog nor in the purchase): the
  machine goes to whatever hero is the town's visitor, and the player at the screen pays (§4.5).

### 4.3 The machine and its price

- **The machine** by the faction argument: `blacksmith_war_machines` 0x6AA9F8's record (the
  artifact, then -1; filled at start-up by `blacksmith_war_machines_init` 0x5C2E20): Castle 4
  (Ballista), Rampart 6 (First Aid Tent), Tower 5 (Ammo Cart), Inferno 5, Necropolis 6, Dungeon 4,
  Stronghold 5, Fortress 6, Conflux 4. Its creature from a second table by the same faction,
  `blacksmith_machine_creatures` 0x642E90 (146 Ballista, 147 First Aid Tent, 148 Ammo Cart; the two
  tables agree for all nine), not through `war_machine_creature` 0x550360.
- **The price** (0x5D13AF to 0x5D13CF): that creature's gold cost (the creature record's +0x38,
  the last of its seven costs at +0x20), shown in decimal (0x5D15B8). So it is data (CRTRAITS):
  Ballista 2 500, Ammo Cart 1 000, First Aid Tent 750 with the shipped tables (the figures of
  `E08-trades.md` §14). No artifact cost is read; no other resource is tested.
- **The dialog** (`blacksmith_dialog_create`, 0x70 bytes, vtable `blacksmith_dialog_vtable`
  0x6438CC, kept in `blacksmith_dialog_pointer` 0x6AA620 while it runs): the picture TPSmith.pcx,
  the machine's picture by its creature, the title GENRLTXT 275 with the creature's singular name
  (+0x14) (0x5D14AF), the price, Buy (iBUY30.def, item 0x7802) and Cancel (iCancel.def, 0x7801).
  Its procedure (`blacksmith_dialog_proc` 0x5D1C60) adds only a right click's description of the
  machine (`blacksmith_dialog_right_click` 0x5D1AA0), the hover text and the animation.

### 4.4 The refusals (all in the dialog's state, no message; 0x5D18D5 to 0x5D19D0)

- The current player not human (`player_is_human_both` 0x4BAD80): Buy disabled (0x5D18E4).
- The **viewing** player's gold (the player record's +0xB4, `viewing_player` 0x4CEA20) below the
  price, or the visitor already carrying the faction's machine (`hero_has_artifact` 0x4D91B0: a
  worn slot or the backpack): Buy disabled (its state changed twice, 0x5D199C to 0x5D19D0). The
  player can only cancel.
- No refusal for a full slot or a full backpack: the machine has its own position (§4.5).

### 4.5 The purchase (`town_blacksmith_buy` 0x5D1DFE to 0x5D1ED8)

Only when the current player is human and the dialog ended with Buy (the window manager's result
0x7802):

1. `hero_give_artifact` 0x4E3070 gives the visitor the faction's machine (the record of
   0x6AA9F8): at its first position that takes it, the machine's own; the result is not tested.
2. **The payment, by the current player** (`current_player_record`): each of the creature's
   seven costs (+0x20 to +0x38) is subtracted from the player's **gold** (+0xB4) (the loop
   0x5D1E6D to 0x5D1E87 never moves to the next resource). With the shipped tables a machine costs
   gold only, so the payment is the price; a ruleset that gives a machine another resource would
   pay that amount in gold (T-25, §10).
3. TCOMMAND 31 (`town_text_purchased` 0x6A5E00, inferred) with the creature's singular name in an
   OK box.

So one machine an opening: the next opening finds the hero carrying it (Buy disabled). No limit a
day or a week, no day mark of the town, no stock, no movement spent. The test reads the viewing
player's gold, the payment the current player's: the same player on a human's turn in a
single-player or hot-seat game (network play not followed).

### 4.6 Building 22: the Stronghold's Ballista Yard and the other factions

Case 22 (0x5D3B38): the click flag as above; a left click goes by the town's faction (0x5D3B46;
the sub-switch 0x5D3B53 has factions 0 to 7, the Conflux falls out at 0x5D3B4D). The building
names from BldgSpec (record faction x 11 + 5, `town_building_name`'s rows):

| Faction | Building 22 | Left click |
|---|---|---|
| Castle | Brotherhood of the Sword (it supersedes the Tavern 5, 0x63FCA0) | `town_tavern_click` 0x5D82B0 (0x5D3B5A): the Tavern; not this increment's |
| Rampart | Treasury | its description (0x5D3BED, `town_building_description` with 22) |
| Tower | Library | its description |
| Inferno | Castle Gate | no visitor: GENRLTXT 127 (only visiting heroes may use the gate); else `town_castle_gate` 0x5D8480 (0x5D3B6D); a later increment's |
| Necropolis | Skeleton Transformer | 0x5D3BCE: §7 |
| Dungeon | Portal of Summoning | `town_portal_hire` 0x5D2950 (0x5D3BDA; `E09-growth.md` §8) |
| Stronghold | Ballista Yard | `town_blacksmith_buy` with the visitor and faction **0** (0x5D3BB0 to 0x5D3BC4) |
| Fortress | Blood Obelisk | its description |
| Conflux | none (its 22 the screen's display piece, `town_screen_pieces` 0x5C33F0) | nothing |

- **The Ballista Yard is a Castle's Blacksmith**: the same dialog with faction 0, so the Ballista
  (artifact 4, creature 146), its price (2 500), the visitor only (none: GENRLTXT 274 naming the
  *Blacksmith*, not the Ballista Yard), a hero carrying a Ballista refused, the same purchase.
- **Both buildings sell.** The Stronghold's supersede list (0x63FDB0) holds only the hordes' two
  entries: 22 does not supersede 16, so the Blacksmith stays standing with the Ballista Yard, and
  a Stronghold sells the Ammo Cart at 16 and the Ballista at 22. Both stay clickable (§5.8's
  click map): 22 is drawn right after 16 (`town_screen_draw_order`, places 33 and 34) at (617,
  286) beside 16's (660, 286), and its hit zone (1 294 pixels) leaves 5 444 of the Blacksmith's
  5 551 to it; no later piece but the Grail's is drawn.

### 4.7 The map's War Machine Factory compared (`E08-trades.md` §14)

| | Blacksmith (16), Ballista Yard (22) | War Machine Factory (class 106) |
|---|---|---|
| Hero | the town's visitor only | the visiting hero |
| Offer | one machine by faction (a Stronghold two, one per building) | Ballista, First Aid Tent, Ammo Cart in one dialog |
| Dialog | its own (Buy, Cancel), no question | a question, then the hire dialog with a count of 0 or 1 |
| Machine to creature | `blacksmith_machine_creatures` 0x642E90 | `war_machine_creature` 0x550360 |
| Price | the creature's gold cost; the test gold only (the viewing player); every cost paid as gold | the creatures' costs through the hire dialog |
| Machine already carried | Buy disabled | the most 0 |

Neither offers the Catapult; neither marks the building or spends movement.

### 4.8 The computer (not followed; §8)

`ai_town_visit` 0x5253D0 buys through `ai_buy_war_machine` 0x525CA0 (`E31-computer-player.md`
§3.3): the Blacksmith 16 by `blacksmith_war_machines` and a Stronghold's 22 with the Ballista,
for the visiting hero itself, above Easy with a garrison hero, with its own value test, the cost
tested in every resource and paid at player +0x9C, the building bought first when missing.

## 5. The town's Shipyard and its boat

Read: `town_screen_proc`'s case 6 (0x5D36B4 to 0x5D3864) and its switch's entry for 20;
`shipyard_dialog_create` 0x5D1EF0 (read in part, after E08 T79a: its argument's picture 0x5D2266 to
0x5D2281, its strings, its Build button's state 0x5D242D to 0x5D24E7); `create_boat` 0x4BB250
(read whole again); `adventure_mouse_move` 0x40E360 at its class tests (0x40E579, 0x40E636);
`shipyard_dialog` 0x5D2720's argument (0x5D2738 to 0x5D2757; `visit_shipyard` 0x49E582). Cited
from E09's and E08's readings: `town_buildings_setup` 0x5C08C0, `town_boat_spot` 0x5C0C90,
`town_ship_update` 0x5BF210, `town_screen_show_build` 0x5D6A80, `town_build_effects` 0x5BEDE0,
`visit_shipyard` 0x49E2E0.

### 5.1 The spot (cited)

- **Once at the new game** (`town_buildings_setup` 0x5C08C0 calls `town_boat_spot` 0x5C0C90 at
  0x5C09A5, then 0x5C09BF): the tile two rows below the entrance and one column left, else one
  column right, on the town's level (+0x7); water, not blocked, and no object or a boat: its column
  and row into +0x8, +0x9. Neither: both stay 0xFF and the Shipyard (6) is forbidden (as in a
  faction without one). Never recomputed.
- **Buildability without a spot** (E09 T12a): `town_buildable_now` 0x5C0F20, `town_build_possible`
  0x5C0E60 and `town_can_buy` 0x5C0D20 leave 6 out, the hall's text says so
  (`build_dialog_requirements_text` 0x5D5BE0), a town event skips it (`town_events_apply`
  0x5BFEB0). So a standing Shipyard always has a spot, and the boat path does not test it again.
- **The ship (20)** is no construction: at each build of the screen (`town_screen_load` 0x5C6870)
  `town_ship_update` 0x5BF210 builds 20 when the Shipyard stands and a boat or a hero (class 8,
  34) is at the spot, and takes it out when none is.

### 5.2 The click (case 6, 0x5D36B4)

- The click flag as in §4.1 (a right click the description). A left click:
  1. **64 boats in use** (`boats_in_use` 0x4CCDD0, 0x5D36E1): GENRLTXT 52 (no more boats) in an
     OK box (0x5D3829), no dialog.
  2. Else **the boat dialog**: `shipyard_dialog_create` 0x5D1EF0 with the town's faction
     (0x5D36F4), kept in `shipyard_dialog_pointer` 0x6AA9B0 (the map's dialog's global too), run
     modally and destroyed; the resource bar redrawn.
  3. **Its Build** (result 0x7802, 0x5D3764): `create_boat` 0x4BB250 at the spot (+0x8, +0x9, the
     town's level +0x7) for the **viewing player** (`viewing_player` 0x4CEA20), not quiet, type 1
     (0x5D3771 to 0x5D379C). On -1 (64 in use) nothing more. Else `town_screen_show_build`
     0x5D6A80 with 20: the ship shown and built through `town_build_effects` (no day mark for 20,
     `E09-construction.md` §3; with the visit: the town's heroes' `town_visit_buildings`; nothing
     when 20 already stands); then **1 000 gold and 10 wood** taken from the viewing player
     (+0xB4, +0x9C; 0x5D37C9 to 0x5D380B), the resource bar redrawn.
- **The ship (20) has no case**: its index goes to the switch's end (0x5D4217), so a click on it,
  either button, does nothing.
- The screen's item 0x7800 is set and restored around the dialog (`window_manager_send` 0x5D36D6,
  0x5D3818): not a rule.

### 5.3 The dialog's refusals (`shipyard_dialog_create` 0x5D242D to 0x5D24E7)

- Build (0x7802) disabled, with no message: the current player not human
  (`player_is_human_both`, 0x5D2433); the viewing player's gold below 1 000 or wood below 10
  (0x5D2473, 0x5D2491).
- **The price is code**: the dialog's two price texts are the fixed strings "1000" and "10"
  (0x5D20E0, 0x5D2135), and the test's and the payment's constants are the same; nothing is read
  from data (as at the map's Shipyard, `E08-economy.md` §5.2).
- The boat's picture by the faction argument (`shipyard_boat_pictures` 0x68A340: Castle AB02,
  Necropolis AB01, Fortress AB03, Conflux AB01; the map's Shipyard passes 0, 0x49E582); the boat
  built is type 1 from every town (the picture only differs).

### 5.4 What is not tested

- **The spot's occupancy.** Neither the case, nor the dialog, nor `create_boat` (read whole: a
  free record, the tile, in use, owner, then `hero_show` 0x4D75D0) looks at the spot: a boat
  created where a boat or a hero stands is shown over it, the one below kept as its under-object.
  The map's Shipyard refuses that (GENRLTXT 52, `E08-economy.md` §5.2). The town relies on its
  screen: with a boat or a hero at the spot when the screen was built the ship (20) stands, and a
  boat bought on the screen builds 20 at once; the ship's click zone covers the Shipyard's all
  but a few pixels in the Fortress and the Conflux (§5.8).
- **The owner.** No owner test on the path: a town screen opened for an ally's town (a visit)
  builds a boat that the viewing player owns and pays for.
- **A hero.** None needed: the owner opens the screen from the town list, so a boat appears at
  the spot for 1 000 gold and 10 wood at any time of the turn.
- **A day's limit.** No mark: the boat is not the day's building and several boats a day are
  bounded only by the spot and the 64 boats.

### 5.5 From the adventure map

The Shipyard's cursor and its use without a visit (click mode 8, `shipyard_use_case` 0x4085A8 to
`visit_shipyard`) are for class 87 alone (`adventure_mouse_move` 0x40E636); a town (98, 0x40E579)
gets the town's cursor. A town's Shipyard is used only through the town screen.

### 5.6 The map's Shipyard compared (`E08-economy.md` §5.2, §5.3)

| | The town's Shipyard (6) | The map's Shipyard (87) |
|---|---|---|
| Reached by | the town screen (a visit or the owner at will) | a visit, or the cursor of the owner's team |
| Spot | fixed at the new game, beside the entrance | fixed at load, the first of twelve offsets |
| Occupied spot | not tested; the ship's click zone covers the Shipyard's (§5.8) | GENRLTXT 52 |
| Too poor | Build disabled | GENRLTXT 330, no dialog |
| Builder, payer | the viewing player | the current player |
| Owner test | none | flagged at the visit; the team for the cursor |
| Price | 1 000 gold, 10 wood (code) | the same |

### 5.7 The computer (not followed; §8)

`ai_boat_create` 0x430F80 (a computer hero's walk: the town's Shipyard bought first when missing,
10 wood and 1 000 gold, `create_boat` for the hero's owner, no dialog), `ai_boat_purchase_cost`
0x431160 (the pathfinder's price) and `ai_hero_turn` 0x5261F0's boat marks (`E31-computer-player.md`).

### 5.8 Building 20 and the docked boat

Read: every write of the town's two sets (+0x150 to +0x15F; `exe.py grep` of the stores and of
20's mask `building_bits` +0xA0): `town_ship_update` 0x5BF210 (read whole again),
`town_screen_load` 0x5C6870 at its piece loops (0x5C6AA1 to 0x5C6B8E, 0x5C6BD5 to 0x5C6C40),
`town_screen_piece_create` 0x5C2EA0 (read whole), `town_screen_piece_draw` 0x5C2FF0 (read in
part: its start and its hit map, 0x5C32CF to 0x5C330C), `pcx8_mask_fill` 0x44FBA0 at its inner
loop (0x44FC2D to 0x44FC4F), `town_screen_refresh` 0x5D5410 (0x5D5425 to 0x5D5469),
`town_screen_show_build` 0x5D6A80 at its piece calls; the tables `town_screen_draw_order`
0x642EB4, `town_building_positions` 0x68A9BC and `town_building_outlines` 0x68A38C (dumped); the
supersede lists and `map_building_ids` 0x6888C0 (dumped); the pack (`packs/base/towns.toml`). The
hit zones' masks measured in the staged `H3bitmap.lod` (pixel counts only).

- **Who writes 20.** Set: `town_ship_update` (through `town_build`, at 0x5BF2F8; the standing
  set +0x150 alone, so the every-built set takes 20 only at the town's next `town_build_effects`)
  and the boat's purchase (`town_screen_show_build` 20, §5.2). Cleared: `town_ship_update` alone
  (0x5BF32C to 0x5BF396: 20 out of +0x150, +0x158 recomputed from it with the supersedes). Every
  other store of the sets is a constructor, the new game's setup, a whole-record copy (four
  functions), `town_build`, `town_build_effects` or `capitol_limit`; no map bit gives 20
  (`map_building_ids` has 6 at bit 16, never 20), no faction list holds it (so neither the hall
  nor the build-all cheat builds it), no supersede list names 20 or 6, nor does the pack (no
  `supersedes` on the Shipyard, no ship building).
- **When 20 is cleared.** Only when a town screen is built (`town_screen_load` 0x5C68BB, the
  only caller of `town_ship_update`), and then only with the Shipyard in +0x158 and neither a boat
  nor a hero at the spot (the spot's entrance bit and class 8 or 34). Nothing clears it when the
  boat sails, a hero boards it, Scuttle Boat sinks it, the town is captured or a day begins: a
  stale 20 stays until that town's screen is next built, and every screen build clears it before
  its pieces are made (0x5C68BB comes before the piece loop 0x5C6AA1). So the next opening after
  the boat left shows the Shipyard free.
- **The drawing.** `town_screen_load` makes one piece per entry of `town_screen_draw_order`
  0x642EB4 (44 bytes a faction, -1 for none; `town_screen_piece_create`: its DEF, its position
  from `town_building_positions`, an area item with the building as id, its outline PCX from
  `town_building_outlines` and its hit-zone PCX, the same name with Z as the second letter); a
  piece is shown only while its building is in +0x150 (0x5C6B36 to 0x5C6B62). In all four
  factions with a Shipyard, 20 comes right after 6 in the order (Castle places 26, 27;
  Necropolis 33, 34; Fortress 35, 36; Conflux 14, 15) and at the same position (Castle (478,
  134), Necropolis (617, 265), Fortress (197, 294), Conflux (239, 215)): so both are drawn and the
  ship is drawn over the dock. 20 hides 6 only by drawing over it, not by a supersede.
- **The click zone.** The refresh clears the screen's 800 x 600 word map (the town dialog's +0x50)
  and has every shown piece write building + 1 at each non-zero pixel of its hit-zone PCX, in the
  pieces' order (`town_screen_refresh` 0x5D5425 to 0x5D5465, `town_screen_piece_draw` 0x5C32D6,
  `pcx8_mask_fill`); after a build the animation redraws every piece with its mask (0x5D6DF9;
whether it clears the map first, 0x5D6DD2, not read). A click on a
  building area reads the word under the mouse (0x5D34A5): the last piece drawn there wins. The
  masks measured: Castle and Necropolis, the ship's zone is the dock's exactly (689 and 3 090
  pixels); Fortress, the ship's (284 x 80) covers the dock's (175 x 80) but for 1 pixel, (233,
  341) on the screen, which no later piece covers; Conflux, the ship's (2 108 pixels) covers the
  dock's (1 262) but for 4: (263, 246) and (264, 246), covered by any Mage Guild level or 21
  drawn later, and (300, 245) and (328, 250), left the Shipyard's.
- **The case does not test 20** (0x5D36B4 to 0x5D3864, re-read): a click that reaches 6 opens
  the dialog whatever stands at the spot.
- **So:** on the human path a second boat cannot be bought onto a docked one (or over a sailing
  hero at the spot) in a Castle or a Necropolis town, nor anywhere but at those few pixels: a
  click on the ship does nothing (§5.2). In a Fortress town (1 pixel) and a Conflux town (2 to 4
  pixels) a click on the uncovered pixels reaches 6, and the purchase then puts a boat over the
  boat or hero at the spot (`create_boat` tests no tile; `hero_show` keeps the one below as the
  new boat's under-object). The dock is cleared when the town's screen is next built after the
  boat or hero left the spot, never at the moment it leaves.

## 6. The Conflux's Magic University

Path followed: the human player's click on the town screen (`town_screen_proc` 0x5D3240's building
switch 0x5D34DA). Not followed: the computer (`ai_*`), network play (the map's University tests
`remote_turn` 0x691209 before its dialog; the town path has no such test, the town screen being the
local player's), the tutorial, the game version `[game]+0x1F698` (none of the functions below reads
it). Text numbers are GENRLTXT's 0-based lines (the general text table's offset / 4, as in
`E09-guild.md` §13).

### 6.1 The click (`town_screen_proc` 0x5D3240, case 21 0x5D3963 to 0x5D3B05)

- **Read:** case 21 whole (with the faction test 0x5D3A10).
- The right button's flag (the message's flag, 0x5D3963) shows building 21's description
  (`town_building_description` 0x5D2A40 with 21) in a popup (message_box kind 4) for every faction.
- A left click goes by the town's faction (+0x4): the Stronghold (6) the Freelancer's Guild
  (`town_freelancers_guild_trade`, §2.5), the **Conflux (8) `town_university_open` 0x5D2DA0**,
  then the resource bar redrawn (0x5D3AF6); every other faction the description in an OK box
  (message_box kind 1, the faction's building picture). Building 21 of the other factions:
  Castle Stables, Rampart Fountain of Fortune, Tower Lookout Tower, Inferno Brimstone
  Stormclouds, Necropolis Necromancy Amplifier, Dungeon Mana Vortex (`E09-guild.md` §15), Fortress
  Glyphs of Fear (BldgSpec, the faction's row + 4): a click on them opens nothing.
- **No owner, human or day test** in the case; the building is clickable only while it stands (the
  screen's pieces, §5).

### 6.2 The student (`town_university_open` 0x5D2DA0, read whole)

- **The visitor** (town +0x10 at or above 0), **else the garrison hero** (+0xC), else none
  (0x5D2DC8 to 0x5D2E09).
- **The selection overrides it:** when a stack of the upper bar is selected (the town screen's
  selected bar +0x12C, set by `town_garrison_click`'s select 0x5D4C3F and cleared by
  `town_screen_bars` 0x5C7087, equals the upper bar +0x11C, the garrison's) and the town has a
  garrison hero, the **garrison hero** is the student even with a visitor (0x5D2E10 to 0x5D2E38);
  without a garrison hero the visitor stays.
- **No hero:** building 21's description in an OK box (0x5D2E50, message_box 0x5D2EB1), no
  dialog, nothing else.
- The tests for -1 after the tests for a negative index (0x5D2DD7, 0x5D2DF6, 0x5D2E29) are never
  reached: harmless.

### 6.3 The four skills and the dialog

- **Fixed:** Fire, Air, Water and Earth Magic (14 to 17) written as constants into a buffer
  (0x5D2F0A to 0x5D2F1F); `conflux_university_skills` 0x5D2D80 holds the same four for the
  computer and is not called here. Nothing is drawn, nothing stored in the town.
- `university_dialog` 0x5EF500 (read in part, 0x5EF500 to 0x5EF7EE) with the hero, the four and
  the flag 1; run modally and destroyed (0x5D2F26 to 0x5D2F72). The hero is kept at +0x60, the
  skills copied into the dialog; **the flag only picks the picture** (0x5EF6AF: 1 the Conflux's
  building from hallelem.def, frame 21; 0, the map's call, univbldg.pcx) and is not kept. The
  slots, the price and the purchase are therefore the map's University's (`E08-rewards.md` §6.8).
- **Bans:** neither `university_slot_state` 0x5F0790, `university_slot_click` 0x5F0AC0,
  `university_learn` 0x5F0F60 nor `hero_learn_secondary_skill` 0x4E22D0 reads the map's banned
  skills (no use of the game record in the first three; the learn reads the hero only): a magic
  school the map bans is **offered and learnable** at the Conflux. The map's University never
  offers one (its four are drawn among the unbanned, `university_setup` 0x4C06F0).

### 6.4 The slots, the price and the purchase (re-read at their tests)

- **Read:** `university_slot_state` 0x5F0790 (its tests), `university_slot_click` 0x5F0AC0 (its
  tests and the gold test 0x5F0CFD to 0x5F0D29), `university_learn` 0x5F0F60 whole.
- **A slot's state, in this order** (`university_slot_state`): the hero knows the skill (its
  level +0xC9 + skill above 0): disabled, GENRLTXT 605 (already known); the hero's class weighs it
  0 (the class table +0x18 + skill, the class the hero's +0x30): disabled, 606 (cannot learn);
  eight skills (+0x101): disabled, 607 (no more skills); else learnable, 608 with the price, the
  constant 2 000 (0x5F0920). **Gold is not tested here:** a learnable slot shows as such without
  the gold.
- **A click** (`university_slot_click`): a disabled slot gives its reason as a message (the same
  three texts, 0x5F0AFA to 0x5F0B8C); a learnable one opens the purchase box (609, the tuition,
  with 2 000; 610 the purchase) whose buy button is **enabled only when the hero's owner has 2 000
  gold or more** (`hero_owner_record` 0x4E56B0, its +0xB4; 0x5F0D18). Below it the box stays
  open with the button disabled; no message.
- **The purchase** (`university_learn`, the box's buy button): **2 000 gold** taken from the
  hero's owner (0x5F10A0 to 0x5F10B3; no test there, the button's enabling is the test), then
  `hero_learn_secondary_skill`(skill, 1): **Basic**. The box closes and the four slots are
  evaluated again (0x5F1122 to 0x5F1136), so **several schools may be bought in one visit**,
  each 2 000, while the gold lasts and the hero has fewer than eight skills.
- **Gold only**; the price is the constant 2 000 at all four sites (the hint 0x5F0920, the box
  0x5F0C05, the test 0x5F0D18, the payment 0x5F10AB); **the payer is the hero's owner**, not the
  town's owner and not the current player (the same player for a human's own hero).
- A right press on the buy button (`university_learn`'s first branch, message 0xE) shows the
  chosen skill (+0xD4) at Basic with its 2 000 in a popup; no rule. The eight-skills tests compare
  the count with 8 for equality (0x5F08B1, 0x5F0B7A): harmless, a hero never holds more.

### 6.5 The owner and the turn's time

- **No owner or human test** on the path (case 21, `town_university_open`, the dialog's three
  functions). At an ally's town opened by a visit (§2.1) the visitor (the human's) learns at its
  owner's gold. Whether the human can select a stack of an ally's garrison bar, which would make
  the ally's garrison hero learn at the ally's gold, was not read (`town_garrison_hover`): §10, §12.
- **The turn's time:** the dialog's vtable (`university_dialog_vtable` 0x643BD8) has at +0x38
  `dialog_close_on_time_expired` 0x5F1180 (read whole): the dialog closes; a purchase is atomic
  (0x5F10A0 to 0x5F10C3), so nothing is half done.

### 6.6 The town's University against the map's (§6.8 of `E08-rewards.md`)

| | The map's University (class 104) | The Conflux's Magic University (21) |
|---|---|---|
| The four skills | drawn at a new game among the unbanned (`university_setup`) | Fire, Air, Water, Earth Magic, fixed; a banned school offered |
| The student | the visiting hero | the visitor, else the garrison hero; a selected upper-bar stack picks the garrison hero |
| No hero | not possible | the building's description, no dialog |
| The slots, the price, the level | `university_slot_state`, `_click`, `_learn`: 2 000 gold each, Basic, several per visit | the same functions |
| The payer | the hero's owner | the hero's owner |
| Picture, position | univbldg.pcx, the dialog centred | hallelem.def frame 21, not centred (drawing only) |
| After | the team's kind mark (game +0x4E35C) | the resource bar redrawn; no mark |
| A limit | none (any visit) | none: at any time of the turn, any number of times |

### 6.7 The computer (pointer, §8)

`ai_town_visit` 0x5253D0 (E31, its map summary; not read here): at any Conflux town with 21 in
the every-built set `ai_university` 0x524ED0 with `conflux_university_skills`; its own tests are
the human's slot tests without the bans, and the purchase the same 2 000 and Basic.

## 7. The Necropolis's Skeleton Transformer

### 7.1 The click (`town_screen_proc` 0x5D3240, case 22 0x5D3B38 and case 23 0x5D3C52)

- **Read:** cases 22 and 23 whole, with their sub-switches (`switch`).
- **Case 22:** the right button's flag goes to the common popup (0x5D3F53, not read); a left
  click goes by the faction (0x5D3B46; the sub-switch 0x5D3B53 has factions 0 to 7, the Conflux
  falls out at 0x5D3B4D). **The Necropolis (4): `town_skeleton_transformer_open` 0x5D2F90**
  (0x5D3BCE); no owner, human, hero or day test. The other factions' 22 (BldgSpec, the faction's
  row + 5; consistent with §4.6):
  - Castle, Brotherhood of the Sword: `town_tavern_click` 0x5D82B0 (the Tavern; not this increment's).
  - Rampart Treasury, Tower Library, Fortress Blood Obelisk: the description in an OK box (0x5D3BED).
  - Inferno, Castle Gate: no visitor, GENRLTXT 127 (only a visiting hero may use the gate); else
    `town_castle_gate` 0x5D8480 (a later increment's).
  - Dungeon, Portal of Summoning: `town_portal_hire` 0x5D2950 (`E09-growth.md` §8).
  - Stronghold, Ballista Yard: `town_blacksmith_buy` 0x5D1D30 with faction 0 (§4).
  - Conflux: no building 22 (the screen's display piece): nothing.
- **Case 23** (0x5D3C52, the sub-switch 0x5D3C65 on faction - 2): Tower (Wall of Knowledge),
  Inferno (Order of Fire), Dungeon (Battle Scholar Academy) and Stronghold (Hall of Valhalla): a
  left click the description in an OK box (0x5D3C6C, `town_building_description` with 23), the
  right flag the common popup (0x5D3C71); the Necropolis (4, the table's out-of-range entry) and
  the factions outside 2 to 6 (Castle, Rampart, Fortress, Conflux, none of which has a 23):
  nothing. Their visit effects are `town_visit_buildings`' (increment 4's, `E09-guild.md` §15).

### 7.2 Whose army (`town_skeleton_transformer_open` 0x5D2F90, read whole)

- **The visitor's army** (hero record +0x91) when the town has a visitor (+0x10 at or above 0);
  else **the army the town shows** (`town_garrison_army` 0x5C1460: the garrison hero's, else the
  town's own garrison +0xE0). **No hero is needed**: the town's garrison alone is transformed.
- **The selection overrides it** as at the University: a stack selected in the upper bar (the
  screen's +0x12C equal to +0x11C) gives the upper bar's army (`town_garrison_army`, 0x5D3005),
  even with a visitor.
- Then the dialog (`transformer_dialog_create` 0x5654F0 with the army), run modally
  (`dialog_run_modal` 0x5FF460), destroyed (`transformer_dialog_destroy` 0x565F60); after it the
  screen's selections cleared (+0x12C, +0x134, +0x1C4), both bars redrawn, a message to the
  screen (`window_manager_send` 0x5D30A5) and `town_screen_refresh` 0x5D5410.

### 7.3 The dialog (`transformer_dialog_create` 0x5654F0, read in part)

- **Read:** its start (0x5654F0 to 0x565590), the slots and the three buttons (0x565640 to
  0x565A54, 0x565BA6 to 0x565C04), its end (0x565EC0 to 0x565F29); the drawing between not read.
- **Two armies:** the army given (+0xAC) and the dialog's own **holding army** (+0x74, emptied by
  `army_reset`; +0xB0 points at it); no selection (+0x6C side, +0x70 slot, both -1).
- **Fourteen slots** (`transformer_slot_row_create` 0x566760, four rows: each army's slots 0 to 5
  in three columns of two and its slot 6 alone), each an item of `transformer_slot_vtable`
  0x64165C whose +0x34, `transformer_slot_notify` 0x5654C0 (read whole), passes a press inside
  the slot (`dialog_def_message` 0x4EA8CE: the left button's message 8, the right's 0x20) to
  `transformer_slot_click` 0x566490 with the item's side (+0x48) and slot (+0x4C).
- **Three buttons:** AltArmy.def, `transformer_move_all` 0x5669F0 (+0x68); AltSacr.def,
  `transformer_convert` 0x566C20 (+0x64), **disabled at the start** (0x565A51); iCancel.def,
  `transformer_close` 0x566B30. No hotkey is added (the create never calls
  `vector_insert_u32` 0x404200). The art and the slot items are the Altar of Sacrifice's
  (`E08-trades.md` §12).
- `transformer_slot_refresh` 0x566030 (read in part: its hint branch 0x5661E2 to 0x56638D) for
  slots 0 to 6 of the given army at the end: the hints only (GENRLTXT 482 for the army's side; on
  the holding side 490 naming the target, singular for a count of 1, else plural, or 491/492 for
  a Skeleton stack that stays one). No rule.
- The procedure (+0x24) `transformer_dialog_proc` 0x5666F0 (read whole; the Altar's too):
  `dialog_proc_base` 0x41B1C0 and nothing of its own.

### 7.4 Moving stacks (`transformer_slot_click` 0x566490, read whole)

- **A right press** on a creature: its view held while pressed (`dialog_show_held` 0x530D30); a
  left press on the selected slot again: the creature's view (`creature_dialog_run` 0x5F47F0).
  Both are built from the creature's type alone (no army passed): no dismiss, no upgrade.
- **A left press with nothing selected** selects the slot (any slot, an empty one too; its frame
  shown).
- **A left press with a slot selected** (either army): the same creature type (or both empty)
  **merges** the selected stack into the pressed slot (`army_add_stack` 0x44ACE0 at that slot,
  the count added) and empties the selected one (`army_clear_slot` 0x44AB60); different types
  **swap** the two slots' types and counts (0x56657B to 0x5665C4). Either way the selection ends.
- **No test** of the creature (an undead, a Skeleton, a dragon all move), of the count, or of the
  army being emptied: the given army may be left with nothing inside the dialog.
- After each move (as after the buttons' work): **Convert enabled only while the holding army
  holds a stack whose target differs from its creature** (0x566617 to 0x56663B); Move all enabled
  while the given army holds any creature (`army_has_creatures` 0x4496A0).

### 7.5 The transformation (`transformer_convert` 0x566C20, read whole; `skeleton_transformer_targets` 0x6412FC)

- **Convert** (the button's command, message 0xD without the right flag): every holding slot in
  order whose creature c is not -1 and whose target t = table[c] differs from c: the creature's
  sound played (its record +0x8 prefix, `wav_load`, `play_sample`), **the slot's type set to t,
  its count unchanged** (0x566CF4 to 0x566D00), the slot redrawn. The given army is not touched.
- **The table** (145 i32, creatures 0 to 144, read whole with `data`): **Bone Dragon (68)** for
  Green and Gold Dragons (26, 27), Red and Black Dragons (82, 83), **Hydra and Chaos Hydra (110,
  111)**, Azure, Crystal, Faerie and Rust Dragons (132 to 135); **Skeleton (56) for every other
  creature, the undead included**: Skeleton Warriors (57), Walking Dead and Zombies, Wights and
  Wraiths, Vampires, Liches, Black Knights and **Bone and Ghost Dragons (68, 69)** all become
  Skeletons. The only creature equal to its target is the **Skeleton** (56): it stays and keeps
  Convert disabled (the hint 491/492). The war machines (145 to 149) lie beyond the table; an army
  never holds one, and no bound is tested (0x566C8C).
- **Free:** no callback reads a resource or the player record. **No limit:** no mark, no day or
  week test; the Transformer may be used any number of times a turn.
- A right press on a button shows its help in a popup (message 0xE with the right flag, 0x566C37,
  0x566A09, 0x566B4B); no rule.

### 7.6 Move all and the close (`transformer_move_all` 0x5669F0, `transformer_close` 0x566B30, read whole)

- **Move all:** each slot i of the given army with a creature goes to the holding army
  (`army_add_stack`: at slot i when that is empty, else onto the first holding stack of the same
  type, else the first empty slot) and its slot is emptied. **The last stack too:** the given army
  may be emptied.
- **The close** (iCancel.def): **every holding stack goes back** to the given army, converted or
  not: at its own slot index when the army's slot there is empty, else onto the army's first
  stack of the same type, else its first empty slot; the holding slot emptied; then the dialog
  ends (0x200/10). The destructor returns nothing (`transformer_dialog_destroy` 0x565F60 stops
  the sounds and frees).
- **No stack can be lost by the moves:** the two armies together never hold more than seven
  stacks (a move keeps or lowers their number), so `army_add_stack` always finds a slot (its
  result is not tested at 0x566A81 and 0x566BC8, harmless).
- **The turn's time** (a timed game): `dialog_proc_base` calls the dialog's vtable +0x38
  (`transformer_dialog_vtable` 0x641694), `dialog_close_on_time_expired` 0x5F1180, which ends the
  dialog **without the close's return: the holding army's stacks are lost** (T-26, §10; the Altar's A-48,
  `E08-trades.md` §12.4); a hero whose stacks all lay there is left with an empty army.

### 7.7 The rules (the original, the human's path)

- **Whose army:** the visitor's, else the garrison hero's, else the town's garrison; the upper
  bar's (garrison hero's or town's) when one of its stacks is selected.
- **Which creatures:** any whole stack; every creature but the Skeleton changes; the undead too.
- **Into what:** the ten dragons and hydras above Bone Dragons, everything else Skeletons
  (Skeleton Warriors, Bone and Ghost Dragons included: a downgrade the player may choose).
- **Whole or part:** whole stacks only in the dialog (no split); a part only by a split on the
  town screen before.
- **The count:** the same. **The cost:** none. **Limit:** none.
- **The last stack:** allowed (the army is whole again at the close).
- **Merging:** the conversion merges nothing; the close puts a stack back into its own empty slot
  (so two Skeleton stacks may stand in one army), and the player may merge by clicks. In effect:
  **the chosen stacks transformed in place**, plus the rearrangements the town screen allows anyway.
- **The owner:** no test (an ally's town: the visitor's own army; an ally's garrison only through
  the upper bar's selection, not read whether possible: §12).
- **What it writes:** the given army's seven types and counts only.

### 7.8 The computer (pointer, §8)

`ai_town_visit` 0x5253D0's summary (E31; not read here) names no Necropolis building 22: the
computer never uses the Transformer.

## 8. The computer at these buildings (pointers; E31's readings, `E31-computer-player.md`)

Not followed here: the computer player's paths are E10's and E13's. What its readings say, for
the design's comparison only:

- **The market:** `ai_pay_with_trades` 0x42A470, `ai_trade_affordable_count` 0x42A580,
  `ai_market_trades` 0x42AC20 trade at the rate of its towns with a Marketplace ever built (at most
  10), written straight into its treasury (`ai_market_trade_write` 0x42AD82), no screen and no
  hero; `ai_buy_marketplaces` 0x42AB40 buys Marketplaces for that; `ai_ally_resource_share`
  0x429110 gives allies resources 1:1, as the human's transfer (§2.3) does.
- **The Artifact Merchants:** `ai_artifact_merchants_visit` 0x5259E0 at a visit of its own Tower or
  Dungeon only (never the Conflux's merchants), buying from the same stock
  (`ai_buy_stock_artifacts` 0x524400, the place set to -1 as a human's purchase, §3.3), the
  building bought first when worth it.
- **The Blacksmith and the Ballista Yard:** `ai_buy_war_machine` 0x525CA0 (16 by
  `blacksmith_war_machines`, a Stronghold's 22 with the Ballista), the cost tested in every
  resource, the building bought first when missing.
- **The University:** `ai_university` 0x524ED0 at any visit of a Conflux with 21, by
  `conflux_university_skills` 0x5D2D80, 2 000 gold and Basic; its tests are the human's slot tests
  (none of the map's bans either).
- **Boats:** `ai_boat_create` 0x430F80 (bought on its walk, the town's Shipyard first when missing,
  no dialog), `ai_boat_purchase_cost` 0x431160 (its pathfinder's price).
- **The Transformer:** never used (`ai_town_visit` 0x5253D0 names no Necropolis 22).

## 9. VCMI 1.7.5 and the wiki compared

**VCMI** (its server is the rule; what only its client does is named so). One gap first: its
`visitTradeOnMarketplace` never checks that the town offers the trade's mode (only the
artifact-for-experience trade is tested), so its server accepts any town trade at any town the
actor may use, built or not, with the count 0 without a Marketplace: a validation gap, not a rule.

| Topic | The reading | VCMI 1.7.5 | The wiki | Verdict |
|---|---|---|---|---|
| The Marketplace's modes | resources and the transfer, no hero needed, an own town only (§2.1) | the same (`buildingsLibrary.json` 14; `throwIfWrongOwner`) | trades and gives resources | agree |
| The count | the current player's towns with 14 ever built, at most 10, the town itself none (§2.1) | the town owner's towns with a Marketplace, 0 unless this town has one; no cap (the factors saturate at 9, the creatures' at 8) | "the Marketplaces owned", best at 9 or more | **TD-29** at an ally's town; elsewhere the same |
| Wood or ore for gold at 2, 4, 8 | 37, 62, 112 (E08 §4) | 38, 63, 113 (the ceiling) | 37, 62, 112 (its rate table) | **D-70**: the code and the wiki |
| The hero at the merchants, the guild | the visitor only (§2.1) | a hero of the actor's on the entrance: the visitor or the garrison hero (its client passes the visitor, else the garrison hero) | "a visiting hero" (the merchants) | **TD-28** |
| The transfer | any undefeated other colour, 1:1, up to the stock, the recipient not told (§2.3) | any player in the game, clamped to the stock, the recipient told | gives resources to other players (whom: not said) | agree in law (the message is a view) |
| Artifacts sold | positions 0 to 12 and the backpack; the war machines and the book refused; Misc 5 never offered; at least 1 unit; a combination whole (§2.4) | worn and backpack; the book and the four war machines refused; at least 1 unit; a combination whole | sell 20 % to 60 % (`B(1 + 0.25(M − 1))/5`) | agree but **Misc 5** (T-22; VCMI sells it) |
| An artifact bought at a count of 0 | free (T-19, §2.2) | efficiency min((0 + 3)/20, 0.6) = 0.15: priced, at least 1 | free with 0 Marketplaces at an ally's merchants: an exploit, fixed by HD Mod and HotA | T-19 (an exploit) |
| The stock | one for the game, 3 treasures, 3 minors, 1 major; the new game and the first day of each month from the second, all seven; a purchase empties the entry till then (§3) | one list (`townMerchantArtifacts`), the same shape; the game's start and each new month; a bought entry empty till the roll | one stock for all, 3, 3, 1, renewed each month | agree |
| The stock's exclusions | the bans, the map's Artifact objects and every artifact drawn before, till the pool empties (§3.1) | the bans and the combinations; the least rolled first, placed artifacts not excluded | not said | E08's **D-32** (the code's, ADV-OBJ-006) |
| Every artifact of the classes banned | the picker loops for ever (T-24) | the Grail | not said | T-24 (a hang) |
| The combinations' prices | the sum of the parts' costs, computed at start-up (§3.4) | fixed numbers in `artifacts.json`, each equal to the sum | not said | agree |
| The Freelancer's Guild | the visitor's army, the player's count, the last creature kept, any creature (§2.5) | the same (a count off the lots loses the rest in release builds: its bug) | page not reachable | agree |
| The Blacksmith's machines | Ballista: Castle, Dungeon, Conflux; Ammo Cart: Tower, Inferno, Stronghold; First Aid Tent: Rampart, Necropolis, Fortress (§4.3) | the same (`config/factions/*.json`) | the same (2 500, 1 000, 750) | agree |
| The Blacksmith's hero, refusals | the visitor only; a hero carrying it, too little gold (§4.2, §4.4) | the visitor or the garrison hero (its client: the visitor); the same refusals; gold only | not said | **TD-28** |
| The Ballista Yard | a Castle's Blacksmith in a Stronghold: both sold (§4.6) | adds the Ballista: both sold | sells a visitor the Ballista for 2 500 | agree |
| The boat's spot | one tile fixed at the new game: (−1, +2), else (+1, +2) from the entrance (§5.1) | the first free of the same two at each build: two boats can wait | the boat only on the first of two tiles; any boat there blocks a new one | **TD-31**: the code and the wiki |
| An occupied spot | not tested; the ship covers the Shipyard's click but at 1 to 4 pixels (§5.8) | skipped (the second tile then) | blocks a new boat | T-21 (state-corrupting) |
| The boat's price, who builds | 1 000 gold and 10 wood (code); no hero, no owner test (§5.2) | the same price; no hero; an ally may build | the same price; the owner or an ally | agree |
| The University's offer | Fire, Air, Water, Earth Magic, 2 000 gold, Basic; known, class weight 0, eight skills refused (§6) | the same | page not reachable | agree |
| The University's banned schools | offered and taught (§6.3) | refused (`isAllowed`) | not said (the map's University draws among the unbanned) | **TD-30** with T-23 |
| The University's student | the visitor, else the garrison hero (a selected garrison stack: the garrison hero) (§6.2) | the visitor or the garrison hero (its client: the visitor, else the garrison hero) | not said | agree (either hero) |
| The Transformer's army | the visitor's, else the garrison hero's, else the garrison (§7.2) | the hero's (the visitor, else the garrison hero), else the garrison | not said | agree |
| The Transformer's targets | dragons and hydras (ten) to Bone Dragons, every other but the Skeleton to Skeletons; whole stacks, free, the last allowed (§7.5) | the same ten by a bonus in the base config; every other to Skeletons; the last stack and Skeletons allowed | all to Skeletons but the living dragons and hydras (and HotA's) to Bone Dragons; Bone and Ghost Dragons to Skeletons | agree |

**Not compared:** the screens' paths (the Conflux's missing artifact button, §2.2), the stale ship
(T-27) and the computer's paths: UI or E10's and E13's.

## 10. Disputes, quirk candidates and open points

**Disputes** (for the checkpoint, §13; next `TD-32`):

- **D-70 — the resource rates at 2, 4 and 8 Marketplaces** (E08's, the town's alone: E08 T127a1's
  inbox note). The code: wood or ore for gold 37, 62, 112 (the binary32 steps, `E08-trades.md`
  §4); VCMI: 38, 63, 113 (the ceiling); the wiki's table: 37, 62, 112. The town reaches every
  count from 1 to 10 (§2.1); the census: no player starts above 2. Recommended: the original
  (the code and the wiki), as D-71 was (E08 decision 365). Row TWN-6-1.
- **TD-28 — who trades at the Artifact Merchants, the Freelancer's Guild, the Blacksmith and the
  Ballista Yard.** The code: the town's visitor only (GENRLTXT 274 without one; the garrison hero
  never), unlike the spell book, the University and the Transformer, which take the garrison
  hero too. VCMI's server: a hero of the actor's on the entrance, the garrison hero included (its
  client passes the visitor, else the garrison hero). The wiki: "a visiting hero" (the merchants,
  the Ballista Yard). Impact: a garrison hero must step out to buy or sell. Recommended: the
  original. Row TWN-6-5.
- **TD-29 — the count at an ally's town, with T-19.** The code: the merchants and the guild test
  no owner, so at an ally's Tower, Dungeon, Conflux or Stronghold the current player's visitor
  trades at the **current player's** count and treasury; with no Marketplace of its own the count
  is 0, the artifact purchase's factor 0 and **the price 0** (T-19), the sale 1 unit an
  artifact, the resource and creature trades a division by zero (inferred: nothing tradable).
  VCMI: the **town owner's** count (at least 1 at an ally's town with a Marketplace), and at a
  count of 0 its factor 0.15, so never free. The wiki: the free artifacts are an exploit, fixed by
  HD Mod and HotA (how: not said). An exploit is always fixed (`AGENTS.md`); the question is how.
  Options: (a) the trader's count, **every trade refused at a count of 0** (what the original's
  resource and creature screens already amount to); (b) the trader's count, floored at 1; (c) the
  owner's count (VCMI's). Recommended: (a). Row TWN-6-2.
- **TD-30 — the Conflux's University and the map's banned skills, with T-23.** The code: Fire,
  Air, Water and Earth Magic offered and taught whatever the map bans (no ban test in the dialog,
  its slots or the learn); the map's University draws its four among the unbanned. VCMI: a banned
  skill refused. The wiki: not reachable. The census: 2 maps (`Heroes of Might not Magic` and its
  allied copy) ban all four schools, their random towns may be Conflux. Options: the original;
  T-23 `fix-optional` (faithful teaches, `strict-fixed` refuses), as T-18 for the Aurora's
  map-wide bans (decision 169); always refused (VCMI's). Recommended: `fix-optional`. Row TWN-6-7.
- **TD-31 — the town's boat spot.** The code: one tile, fixed at the new game (two rows below the
  entrance, one column left, else one right), never recomputed; a town whose two tiles both fail
  then has no Shipyard. VCMI: the first free of the same two tiles at each build, so two boats can
  wait and a town keeps its Shipyard while one tile is taken. The wiki: the boat appears only on
  the first; any boat there blocks a new one. Recommended: the original (the code and the wiki).
  Row TWN-6-6.

**Quirk candidates** (`docs/rules/quirks-ledger.md`'s classes; T-28 added at the checkpoint, §13;
next `T-29`):

- **T-19 (exploit): artifacts free at a count of 0** (§2.2, TD-29). Fixed; the fix is TD-29's.
- **T-20 (exploit): the artifact sale's backpack price** (§2.4). After the backpack's arrows moved
  (six or more artifacts) the price is backpack place *p* − 18's and the artifact removed the one
  at (offset + *p* − 18) mod count; the offset stays after a sale. A cheap artifact sells at an
  expensive one's price, repeatably. Fixed by construction: the engine's sale names the artifact
  and prices that one. Row TWN-6-3 (its confirmation).
- **T-21 (state-corrupting): a boat over a docked boat or hero** (§5.4, §5.8). The town's path
  never tests the spot; the ship's click zone covers the Shipyard's but for 1 pixel in a
  Fortress and 2 to 4 in a Conflux, where a purchase stacks a second boat on the spot (the one
  below kept as its under-object). Fixed: a boat is refused while a boat or a hero stands at the
  spot (the map's Shipyard's GENRLTXT 52 rule, ADV-SHY-003). Row TWN-6-6.
- **T-22 (harmless): Misc 5 never offered at the artifact sale** (§2.4: the window has 18 worn
  positions). Moving the artifact in the hero's screen first works. Options: accepted (the sale
  refuses position 18), fixed (sellable). Recommended: accepted (the original's).
- **T-23 (fix-optional candidate): the University ignores the map's banned skills** (TD-30).
- **T-24 (state-corrupting, a hang): the picker's endless loop** (§3.1, `random_artifact_of_level`
  0x4C9549): a map banning every treasure, minor, major and relic never starts (the merchants' draw
  runs at every new game). The engine's pool already gives none there (ADV-OBJ-006): the entry
  stays empty. Fixed by construction; no shipped map bans that much (§1.6).
- **T-25 (harmless): the Blacksmith pays every cost as gold** (§4.5). With the shipped tables a
  machine costs gold only. The engine charges the creature's cost per resource (equal here).
- **T-26 (state-corrupting): the Transformer's expiry** (§7.6). A timed game's expiry closes the
  dialog without returning the stacks moved into it; the Altar's A-48 class
  (`E08-trades.md` §12.4). Moot for an engine whose transformation is one command: fixed by
  construction.
- **T-28 (`fix-optional`, from TD-28 at the checkpoint): the visitor-only buildings.** The
  Artifact Merchants, the Freelancer's Guild, the Blacksmith and the Ballista Yard refuse a town
  whose only hero is its garrison hero (GENRLTXT 274), where the spell book, the University and the
  Transformer take it: `faithful` the visitor only, `strict-fixed` the garrison hero too.
- **T-27 (harmless, UI): the stale ship** (§5.8). 20 is set and cleared only when the town's screen
  is built (and at the boat's purchase), so it stands after the boat sailed until the next
  opening. The engine has no screen: the ship is the spot's occupancy, derived when asked.

**Open points for T53** (the drafts and Part H):

- **Where the trades live:** the town's commands in `h3-towns` and `h3-game`'s `src/town*`, or
  E08's market (`Decision::Market`, GAM-DEC-011, `market_offers`, the checks
  `trade_lot`/`check_sale`/`check_buy`) with a town source; screen 3's sale on the adventure
  lane's `market` module (its rate `artifact_price`'s sale side is ours to add, the inbox's note
  of E08 T132a1); the transfer (screen 1) for a town and a Trading Post alike. The actor is the
  hero's owner: an ally's hero at the owner's town (§2's open point) is never the owner's to trade.
- **The stock in the state:** seven entries and **the pool's marks kept in the game state** (the
  month's draws go on from the new game's marks; today `ArtifactPool` lives only during the
  build), so both enter a state hash; the month's draw at the first day of each month after the
  week's events (GAM-CAL-002's order), with merchants anywhere or not (the stream's parity: the
  original draws regardless); who sees the stock (any player at a merchants' window: public?
  `docs/06-security.md`).
- **The combinations' prices:** the pack's or computed at compile time (the sum of the parts).
- **The Blacksmith and the Ballista Yard:** a town command (the visitor, the machine of the
  building), the price from the machine's creature.
- **The Shipyard:** the town's `BuildBoat` (E08's command for a map Shipyard, ADV-SHY-001 to -003)
  or a town command; the spot is TWN-SET-011's; the boat's type (1 from every town: the picture
  only); the ship (20) derived, not a built building (T-27).
- **The University and the Transformer:** commands naming the hero (the visitor or the garrison
  hero) or the army (the visitor's, the garrison hero's, the garrison's) and the stacks; the
  payer the hero's owner; a selection of an ally's garrison (open, `E` of §6) never the actor's.
- **What each view tells** (the owner's, the recipient's of a transfer, a spectator's), the env's
  encodings (the AI lane), the wire (the server lane), the replay format: increment 5's pattern.

## 11. Batch TWN-6's rows (run on the capture harness; `docs/experiments/README.md`)

The rows confirm the readings that contradict VCMI, stand alone, or rest on values the game
computes at start-up (`docs/exe/README.md`, "Confidence and reliability").

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| TWN-6-1 | D-70 (§2.1; `E08-trades.md` §4) | red owns 2, then 4, then 8 towns with the Marketplace; a town's Marketplace, screen 0 | the gold offered for 1 wood and for 1 ore | 37, 62, 112 at 2, 4, 8 |
| TWN-6-2 | TD-29, T-19 (§2.1, §2.2) | hot-seat, red and blue allied; blue's Tower with the Marketplace and the Artifact Merchants; red owns no Marketplace; red's hero visits it, opens the merchants, buys an entry | the prices shown, red's treasury and the hero's artifacts after the purchase | every price 0; the artifact given, the treasury unchanged |
| TWN-6-3 | T-20 (§2.4) | a visitor in a Tower with the merchants, its backpack an expensive artifact at place 0 and five cheap ones after it; screen 3, the right arrow once, the place shown first picked with gold | the gold offered, the gold received, the artifact removed | the expensive one's price; a cheap one removed |
| TWN-6-4 | §3.4 (the combinations' prices) | a visitor carrying Ring of the Magi in a Tower with the merchants, 1 Marketplace; screen 3 with gold | the gold offered | 3 000 × 0.2: 600, not 1 |
| TWN-6-5 | TD-28 (§2.1, §4.2) | a Tower with the merchants and a Stronghold with the guild, the Blacksmith and the Ballista Yard, each with a garrison hero and no visitor | each building's click | GENRLTXT 274 and no window or dialog at each; the Marketplace's window with screens 0 and 1 only |
| TWN-6-6 | TD-31, T-21 (§5.1, §5.8) | a Fortress with the Shipyard whose spot is the left tile; a boat bought; the town reopened; a click on the ship, then at (233, 341) and Build | the boats at the spot, the boats in use, the treasury | nothing at the ship; at the pixel a second boat on the spot, 1 000 gold and 10 wood taken |
| TWN-6-7 | TD-30, T-23 (§6.3) | an SoD map banning Fire Magic; a Conflux with the Magic University; a visitor whose class can learn Fire Magic, without it, 2 000 gold | the four slots' states; the skill after the purchase | Fire Magic offered and learned at Basic |
| TWN-6-8 | §3.1, §3.2 (the stock; D-32) | a map placing 18 of the 21 minors as Artifact objects, nothing else drawn of them; the stock read on day 1 and on day 29 | the seven, by class | day 1: the three minors the map does not place; day 29: seven again, sold or not |

**Not rows:** the transfer, the creature sale, the Blacksmith's machines and prices, the Ballista
Yard, the boat's price, the University's offer and price, the Transformer's targets (VCMI and, where
reachable, the wiki agree), the computer's paths (E10, E13). **What the harness may lack** (checked
at the batch's cut): a read of the stock (game +0x1F664), a click at a pixel of the town screen,
a hot-seat ally's turn in a recipe.

## 12. Not read

- `town_screen_proc`'s paths outside its building switch and its cases outside this increment,
  the common popup 0x5D3F53, `town_building_description` beyond its first branches (17 and up);
  the town screen's drawing beyond its pieces' loops and hit map (§5.8).
- The screens' creates beyond their items and buttons, the hint handlers, the info popups; a
  greyed button's handling (assumed: no command); the x87 control word at a count of 0 (the
  resource and creature trades' division by zero: inferred).
- Whether a map's preset town gets its buildings' requirements (the census: the merchants and the
  guild always with their Marketplace, 32 halls without one); whether an ally's hero can be the
  visitor at the owner's town on the owner's turn, and whether a human can select an ally's
  garrison bar (the University's and the Transformer's selection).
- Two quest loaders' writes of the artifact bans (inferred), `new_game_heroes`' second caller
  (`game_main`'s other path) and the order of `setup_map_objects`' draws before the stock, the 38 run-time callers
  of `random_artifact_of_level`, the loaded game's older mark reads; which `ARTRAITS.TXT` copy the
  game loads (the SoD one, by the combinations' names in play).
- A sold Spell Scroll's spell (`hero_remove_artifact` recomputes the spells only for an artifact
  whose run-time +0x1D is set) and Titan's Thunder's book on its sale.
- The Blacksmith dialog's pictures and hover text, `town_text_purchased`'s loader (inferred), the
  Transformer's and the University's drawing, whether the turn's timer runs during the town
  screen's modal dialogs (T-26: inferred).
- Network play (the transfer's message, the town screen's network branch), the tutorial, the
  computer's paths (§8).

## 13. The checkpoint (`TWN-Q35`, the user, 2026-11-24)

Asked interactively (`AskUserQuestion`, a round of four, then one of three, then the agent's calls
one by one at the user's asking, a recommendation each); the epic's decisions 201 to 207:

- **D-70** (decision 201): the original. Wood or ore for gold at 2, 4 and 8 Marketplaces gives 37,
  62 and 112 (E08's `MarketParams` already computes them); row TWN-6-1.
- **TD-28** (decision 202): **`fix-optional`**, the new quirk candidate **T-28** (the user: the
  visitor-only rule seemed arbitrary; if so, a flag). It is: no other rule rests on it, the
  original's own spell book, University, Transformer and Grail take the garrison hero, and the only
  trace of a reason is the text GENRLTXT 274. So the Artifact Merchants, the Freelancer's Guild, the
  Blacksmith and the Ballista Yard serve the visitor only in `faithful`, and the visitor or the
  garrison hero in `strict-fixed` (the trade window's hero then the visitor, else the garrison
  hero). The flag waits for row TWN-6-5 ("Confirm before flagging"); over the original without a
  flag and VCMI's without one.
- **TD-29 with T-19** (decision 203): **the trader's count, at least 1**. The rates read the trading
  player's towns with the Marketplace, at most 10, whoever owns the town traded at; a trader with
  none trades as with one, so the merchants' artifacts are never free (T-19 fixed); over the
  recommended refusal of every trade at a count of 0 and over the town owner's count (VCMI's).
- **TD-30 with T-23** (decision 204): `fix-optional`, as T-18 (decision 169). The Conflux's
  University teaches a school the map bans in `faithful`, never in `strict-fixed`; the flag waits
  for row TWN-6-7; over the original without a flag and the ban always respected (VCMI's).
- **T-22** (decision 205): **fixed**. An artifact worn in SoD's fifth misc position sells like
  the other worn ones; over accepted (recommended) and `fix-optional`.
- **TD-31** (decision 206): the original. One boat spot a town, fixed at the new game
  (TWN-SET-011), never recomputed; over VCMI's first free of two tiles at each build.
- **The agent's calls** (decision 207, each asked and accepted): T-20 fixed by construction (the
  sale names one artifact, priced and removed); T-21 fixed (a boat refused while a boat or a hero
  stands at the spot, ADV-SHY-003's rule); T-24 an empty stock entry when the picker finds none
  (ADV-OBJ-006), over the Grail (VCMI's); T-26 fixed by construction (one atomic transformation);
  T-25 the machine's cost per resource; T-27 the ship derived from the spot, never stored; the
  stock's seven and the pool's marks in the game state, drawn at the new game and on each month's
  first day whether or not any merchants stand; the commands name the hero or army where the
  original lets the player choose, the actor owning it; batch TWN-6's eight rows (§11) for T53's
  cut.

**Next:** T53, the ADR amendment drafts (`docs/lanes/town/adr-drafts/*-increment-6.md`), the design
note's Part H from §101, the cut and the exit criteria, with §10's open points for T53.

## 14. Batch TWN-6's results (T54, 2026-11-25)

Run by an agent on the capture harness (the same Complete 4.0 `Heroes3.exe` under Wine, without
the HD mod; hot seat, Easy, the starting bonus Gold), each part twice in new games: specs
`experiments/specs/E09-batch-6a.toml` (rows 1, 3, 4, 5), `-6b.toml` (row 2: Red and Blue allied,
Tan the computer's), `-6c.toml` (row 8, day 1 to day 29), `-6d.toml` (rows 6, 7), recipes
`clients/capture/batches/E09-batch-6a.toml` to `-6d.toml`, the filled templates
`experiments/results/E09-batch-6a.md` to `-6d.md` and their `-run2.md`. Every row as the code
reading, in both runs:

| Row | Observed (both runs) | Verdict |
|---|---|---|
| TWN-6-1 | T1's Marketplace with no visitor opens on screen 0; count 2, 4, 8 (after the Marketplaces built in C1 and C2, then C3 to C6: each town's standing set gains 14): **1 wood for 37, 62, 112 gold**, 1 ore the same | **D-70 the reading** (and the wiki's table); VCMI's 38, 63, 113 refuted |
| TWN-6-2 | at Blue's Tower (an ally's), Red's visitor: the Marketplace opens nothing, no message; the merchants open on screen 2 with **count 0**; entries 0 and 6 priced **0 gold**, most 0, amount 1, Trade enabled; the Trade gives the major (worn), **Red's treasury unchanged**, the entry empty in the window and the stock | **TD-29 the reading, T-19 confirmed** (the current player's count, free at 0); VCMI's owner's count refuted |
| TWN-6-3 | screen 3 at Red's count 8: offset 0, the places Titan's Gladius to Speculum; the right arrow (item 131): offset 1, the places Badge of Courage to Spyglass; place 18 with gold offers **5 500 gold** (Titan's Gladius's price, 10 000 × 0.55); the Trade pays 5 500 and **removes Badge of Courage** (shown at 18), Titan's Gladius stays | **T-20 confirmed**: the price of backpack place 0, the removal of the place shown |
| TWN-6-4 | Ring of the Magi (backpack) at Blue's count 1: **600 gold** | §3.4 the reading (the combination's cost its parts' sum, 3 000) |
| TWN-6-5 | with a garrison hero and no visitor: the merchants (T1), the Freelancer's Guild, the Blacksmith and the Ballista Yard (S1) each **GENRLTXT 274** in an OK box, nothing opened; the Marketplace at both: screen 0 with only the transfer's button (17), count 8 | **TD-28 the reading, T-28 confirmed**; VCMI's garrison hero refuted |
| TWN-6-6 | F's spot (9, 11) (the left tile; the right one water too); the first Build: a boat on (9, 11), 1 000 gold and 10 wood (taken after the ship's build shows), standing 6, 10, 20, 23; reopened, the ship's click opens nothing; at **(233, 341) the boat dialog**; its Build: **a second boat on (9, 11)**, two records there, (11, 11) empty, 2 in use, 1 000 gold and 10 wood more | **TD-31 the reading, T-21 confirmed** (one fixed spot, never tested); VCMI's second tile refuted |
| TWN-6-7 | the map banning Fire Magic: the University's slots **Fire learnable (GENRLTXT 608)**, Air known (605), Water and Earth learnable (608); Fire's click opens the purchase box, buy enabled (5 000 gold); the purchase teaches **Fire Magic at Basic** for 2 000 (5 000 to 3 000), Fire's slot then known (605) | **TD-30 the reading, T-23 confirmed**; VCMI's ban refuted |
| TWN-6-8 | day 1: three treasures, **the three unplaced minors** (Rib Cage, Ring of Life, Boots of Speed, in another order each run), one major; none of the 18 placed; entry 0 bought at count 1 (**5 times its cost**: 5 000 and 10 000 gold for costs 1 000 and 2 000), the entry empty; day 29 (month 2, day 1): all seven redrawn, entry 0 refilled, the treasures none of day 1's, the minors from all 21 (run 1: two of day 1's and a placed one; run 2: three placed ones), the major another | **§3.1, §3.2 the reading, D-32 the code's** (placed artifacts excluded while the pool lasts, the reset frees them); VCMI's refuted |

**The spec's slips, not the game's:** TWN-6-8's prediction priced the purchase at 2.5 times the
cost, the five Marketplaces' factor (`E08-trades.md` §4); the reading's formula at count 1 (cost /
0.2) gives the 5 times observed. Its minors "in the draw's order" are a draw: the set is the
reading's, the order the stream's. **Not predicted, nothing for the engine:** the town's boat is
paid only after the ship's build animation (the harness waits for it); a placed hero carries a
Catapult. **For T55:** the drafts' verdicts as decided (decisions 201 to 206); TWN-6-5 confirms
T-28 and TWN-6-7 T-23, so `TWN-QUIRK-15`'s and `-16`'s flags may be added (T55b).
