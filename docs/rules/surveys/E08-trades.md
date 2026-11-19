# E08 trades and services survey (E08 T127a)

Date: 2026-11-18. Scope: E08 increment 14 (decisions 361, 362), input for T127b's ADR amendment
drafts, the design note and the cut. **In two parts** (the user at T127a's start, decision 363,
after the reading was measured: about 30 KB of code, the market about 15 KB, the Altar of
Sacrifice about 8 KB of a 22 KB dialog, the Hill Fort about 5 KB, the War Machine Factory about
1.5 KB; T118a's whole survey had read about 5 KB): **T127a1** (§0 to §11) the census of
the seven classes and the market: the Trading Posts (99, 221), the Freelancer's Guild (213), the
Black Market (7) and the trade window they share with the town's Marketplace, its rates;
**T127a2** (§12 to §20, whole: the user at its start, decision 364, after its reading was measured
again at about 14.5 KB) the services (the Altar of Sacrifice 2, the Hill Fort 35, the War Machine
Factory 106) with their VCMI and wiki comparison, the interfaces' sketch for all seven, batch E14's
plan, the proposed rule ids and the checkpoint's questions; later **T130** (§21, 2026-11-19) batch
E14 scored. Input, not rules (`README.md`): a fact
becomes binding when a task writes it into the catalog with its sources and tests.

**Sources** (retrieved 2026-11-18):

- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_object` 0x4A84F0 at its cases 7 (0x4A86AD), 99
  (0x4AA2CE) and 213 (0x4A8E34), and the dispatch of cases 2 (0x4A8541), 35 (0x4A9172) and 106
  (0x4AA622) for T127a2; `setup_map_objects` 0x4C0CC0 at case 7 (`black_market_draw` 0x4C0E31);
  the window's entries `trading_post_trade` 0x5EA0C0, `freelancers_guild_trade` 0x5E9E60,
  `black_market_trade` 0x5EA100 and the town's `town_marketplace_trade` 0x5E9FE0,
  `town_freelancers_guild_trade` 0x5E9EA0, `town_artifact_merchants_trade` 0x5E9D20;
  `market_window_run` 0x5EA130 (the screens' loop); the procedures of four screens
  (`market_resources_proc` 0x5ECFE0, `market_transfer_proc` 0x5ED550,
  `market_buy_artifact_proc` 0x5ED9E0, `market_sell_creatures_proc` 0x5EE890) and the rates
  (`resource_value` 0x5ECD10, `market_creature_rate` 0x5ECE80, `market_artifact_sale_rate`
  0x5ECDC0). **Read in part:** the screens' redraws (`market_resources_redraw` 0x5EA6E0,
  `market_transfer_redraw` 0x5EAF50, `market_buy_artifact_redraw` 0x5EB6A0,
  `market_sell_creatures_redraw` 0x5EC550: the controls each enables and the window's title),
  `market_resources_create` 0x5DF5F0 (its buttons), `new_game_heroes` 0x4BFE70 (the defeat flags
  at a new game), `hero_give_artifact` 0x4E3070 (its full backpack). **Read before** and used here:
  `random_artifact_of_level` 0x4C94D0, `object_class_values` 0x63A6E4 (221 as 99), `new_month`
  0x4C8F40 (the Artifact Merchants' seven), `player_defeated` 0x4F11A0, the x87 control word
  (`docs/formats/installation.md` §7.10: 53 bits, to nearest). **Not followed:** the computer
  player (`ai_black_market_visit` 0x5243F0 and the computer's own market trades: E10's, E13's), the
  artifact sale's screen (`market_sell_artifact_proc` 0x5EDF60, the town's Artifact Merchants:
  the town lane's), network play (the transfer's message), the pictures, texts and sounds.
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`): `lib/mapObjects/IMarket.cpp` (`getOffer`: every trade's
  rate), `lib/mapObjects/CGMarket.cpp` (`CGBlackMarket::newTurn`: the restock),
  `lib/mapObjectConstructors/MarketInstanceConstructor.cpp` (the efficiency's default),
  `config/objects/markets.json` (the objects' modes), `config/gameConfig.json`
  (`markets.blackMarketRestockPeriod`), `lib/callback/GameRandomizer.cpp`
  (`rollMarketArtifactSet`), `server/CGameHandler.cpp` (`buyArtifact`, `tradeResources`,
  `sellCreatures`, `sendResources`).
- `wiki`: <https://mightandmagic.fandom.com/wiki/Freelancer%27s_Guild> (the creature rates, the
  map's guild at five Marketplaces); search results' pages of the mirror
  <https://heroes.shoutwiki.com/wiki/Artifact_merchants> (the purchase and sale percentages) and
  <https://heroes.shoutwiki.com/wiki/List_of_adventure_map_objects> (the Black Market's line); VCMI's
  modders' page `docs/modders/Map_Objects/Market.md` as packaged by Debian
  (<https://sources.debian.org/src/vcmi/1.6.5%2Bdfsg-2/docs/modders/Map_Objects/Market.md>: the
  Trading Post's two modes). All read through a summarising tool or a search snippet (the mirror's
  pages refuse a direct fetch): re-check a quote against the page before a rule rests on it.
- The census (§1): a throwaway example over `h3_map::read` on the 160 shipped maps (not
  committed). The rates' comparison (§4, §9): a throwaway script emulating the binary32 steps over
  ARTRAITS.TXT's costs and CRTRAITS.TXT's gold costs (not committed).

**Sources of part 2** (T127a2, retrieved 2026-11-18):

- `original-data`, the human player's path. **Read whole:** the Altar's run
  (`altar_dialog_run` 0x5653B0), screens (`altar_show_artifacts` 0x562A20, `altar_show_creatures`
  0x563150), update (`altar_update` 0x562500), the clicks on the hero's positions
  (`altar_worn_click` 0x5632A0) and the buttons (`altar_backpack_to_altar` 0x564060,
  `altar_all_to_altar` 0x564340, `altar_sacrifice` 0x5646A0, `altar_to_creatures` 0x564910,
  `altar_done` 0x564A80, `altar_all_creatures` 0x564CF0, `altar_creature_max` 0x564DE0,
  `altar_to_artifacts` 0x564E70, `altar_slider_moved` 0x565380), the experience
  (`altar_artifact_entry` 0x55FC30, `altar_set_creature_amount` 0x564C00) and the turn's end
  (`altar_time_expired` 0x565430); the Hill Fort's `hill_fort_dialog_run` 0x4E7E90,
  `hill_fort_rows` 0x4E7EB0, `hill_fort_upgrade_slot` 0x4E8500 and `upgrade_cost` 0x54E750; the
  factory's dispatch (0x4AA622 to its end) and `war_machine_creature` 0x550360. **Read in part:**
  `altar_dialog_create` 0x55FDD0 (the modes), `altar_artifacts_create` 0x560380 (the places),
  `altar_backpack_click` 0x5636C0, `altar_select_creature_slot` 0x564FE0 (the slider's most),
  `altar_dialog_destroy` 0x5623A0 (its calls), `hill_fort_dialog_proc` 0x4E8850 (the clicks), the
  war machines' branches of `recruit_dialog_select` 0x5503A0 and `recruit_dialog_proc` 0x550940.
  **Inferred, not read:** `altar_creatures_create` 0x5610F0, `altar_place_click` 0x563A80,
  `hill_fort_dialog_create` 0x4E75F0. **Not followed:** the computer's visits
  (`ai_hill_fort_visit` 0x527BB0, `ai_war_machine_factory_visit` 0x524FC0: E10's), the redraws,
  pictures, texts and sounds, the Hill Fort's hints on a mouse move.
- `vcmi` (tag `1.7.5`): `lib/mapObjects/IMarket.cpp` (`getOffer`'s `CREATURE_EXP` and
  `ARTIFACT_EXP`), `server/CGameHandler.cpp` (`sacrificeCreatures`, `sacrificeArtifact`,
  `upgradeCreature`, `recruitCreatures`), `client/windows/CMarketWindow.cpp` (the Altar's screens by
  alignment), `lib/entities/artifact/CArtifact.cpp` (`isTradable`, `isBig`),
  `lib/mapObjects/CGHeroInstance.cpp` (`calculateXp`, `getAlignment`), `config/factions/*.json`
  (the alignments), `config/objects/generic.json` (the Hill Fort's `upgradeCostFactor`, the
  factory's creatures), `lib/mapObjects/MiscObjects.cpp` (`HillFort::fillUpgradeInfo`),
  `lib/gameState/UpgradeInfo.cpp` (`addUpgrade`), `lib/mapObjects/CGDwelling.cpp` (the factory's
  machines).
- `wiki`, through search snippets only (the fandom pages refuse a fetch from this session):
  <https://heroes.shoutwiki.com/wiki/List_of_adventure_map_objects> (the Altar's line),
  <https://heroes.shoutwiki.com/wiki/Restoration_of_Erathia_Manual_Page_117> (the war machines'
  prices, one of each per hero), <https://heroes.shoutwiki.com/wiki/Catapult> (the factory sells
  every machine but the Catapult), VCMI's bug <https://bugs.vcmi.eu/view.php?id=558> (the original's
  22 artifacts on the altar per sacrifice). No page read gives the Altar's experience values or the
  Hill Fort's percentages; the user's manuals (`D:\Heroes3\HoMM 3 Complete`, the three PDFs and
  `HEROES3.HLP`) describe none of the three objects.
- The Hill Fort's comparison (§15): a throwaway script over the packs' creatures, their levels,
  costs and upgrades (not committed).

## 0. Findings in short (part 1)

- **The census** (§1): 1 058 objects of the seven classes on 148 of the 160 maps; every one has
  subclass 0, no body in the map file and one entrance, on land, **entered** (corrected by T127b:
  the class lists, not the template's blocked tile). The Freelancer's Guild (213) and the snow
  Trading Post (221) are on AB and SoD maps only.
- **One window, five screens** (§3): the Trading Posts, the Freelancer's Guild and the Black
  Market open the town's market window in a mode of their own that shows one or two of its five
  screens (resources for resources, resources to a player, resources for artifacts, artifacts for
  resources, creatures for resources), and every map object trades **as five Marketplaces**; a
  town trades at the number of the player's towns with a Marketplace, at most 10.
- **The rates** (§4): resource values (250 for wood and ore, 500 for the rest, 1 for gold; for an
  artifact's price half of them, gold 1) and three binary32 factor tables by the Marketplaces'
  count; each rate is a binary32 quotient rounded half up (53-bit), so the halves fall on the
  binary32 side, not the decimal one. At five: a wood for 75 gold, 833 gold for a wood, a precious
  resource for 150 gold; a creature for 70 % of its gold cost; an artifact for 2.5 times its cost.
- **The Trading Posts** (§5): screens 0 and 1: the resource trade and the transfer to any other
  player still in the game, 1:1. **The Freelancer's Guild** (§6): screen 4 only, the visitor's
  army, its last creature kept. **The Black Market** (§7): seven artifacts drawn at a new game
  (three treasures, three minors, one major; drawn as given, never restocked), one bought at a
  time, the entry emptied.
- **Only a human trades** at the visit (the computer's Black Market branch is E10's, E13's); none
  of the three marks a visit or has an owner (§2, §8).
- **VCMI** (§9) agrees on the factors, the modes, the five Marketplaces, the Black Market's set and
  its never restocking, the last creature and the transfer's recipients, but takes the ceiling in
  doubles where one unit buys several: **D-70** (the resources at 2, 4 and 8 Marketplaces, the
  town lane's; none at five) and **D-71** (the creature sale: 99 of 504 cases differ at five).
  **A-46**: the purchase from a hero who cannot take the artifact takes the price and loses it.

**Part 2:**

- **The Altar of Sacrifice** (§12): its screens by the hero's class's faction: Castle, Rampart and
  Tower heroes sacrifice artifacts only, Inferno, Necropolis and Dungeon heroes creatures only,
  Stronghold, Fortress and Conflux heroes both (the artifacts first). An artifact gives **1 000,
  1 500, 3 000 or 6 000** experience by its class (treasure, minor, major, relic), a special one
  **0**; a creature **5 × ⌊AI value / 40⌋**; the hero's Learning factor (HER-EXP-001's product) is
  applied per artifact and per stack and truncated each time. Every artifact but the Spell Book
  and the Catapult may go onto the altar (the Ballista, the Ammo Cart, the First Aid Tent, a Spell
  Scroll and the Grail for 0); the army keeps one creature; Sacrifice needs a total above 0 and
  gives the levels at once, the dialog staying open; 22 places on the altar (and the one artifact
  held) per sacrifice, as many sacrifices as the player likes.
- **The Hill Fort** (§13): each of the visitor's stacks of a town's unupgraded creature (at the
  game's version 0 not the four elementals: an RoE campaign's, never an RoE map's in this edition, §21) upgrades to its town's upgrade for the cost difference times the
  count, **the gold alone scaled by the creature's level**: free at level 1, **25 %, 50 %, 75 %**
  at levels 2 to 4, whole from level 5, truncated once per stack; one stack at a time, or all
  when the whole is affordable.
- **The War Machine Factory** (§14): a question, then the hire dialog with the Ballista, the First
  Aid Tent and the Ammo Cart at their gold costs (2 500, 750, 1 000), one of each the hero lacks,
  bought as the artifact into its position.
- **VCMI** (§15) agrees on the screens by alignment, the experience values and the per-creature
  formula, the last creature, the Hill Fort's factors and the factory; it differs on the war
  machines at the Altar (**D-72**: refused), the Learning's rounding (**D-73**: once over the sum)
  and the Hill Fort's rounding (**D-74**: per creature, every resource scaled). **A-47**: a
  special artifact sacrificed for no experience; **A-48**: what lies on the altar lost when the
  turn's time runs out.
- **For T127b** (§17 to §20): the interfaces' sketch of all seven (decisions as the University's
  and the hire dialog's, recommended), batch E14's plan (15 rows), the proposed ids (ADV-MKT,
  ADV-ALT, ADV-HFT, ADV-WMF, GAM-DEC-011 to -013), the checkpoint's questions.

## 1. The census (the seven classes)

Over the 160 shipped maps (49 RoE, 55 AB, 56 SoD; T97c's counts confirmed):

| Class | Object | Objects / maps | Maps RoE / AB / SoD | Underground | Most on one map | Templates |
|---|---|---|---|---|---|---|
| 2 | Altar of Sacrifice | 177 / 88 | 32 / 28 / 28 | 44 | 7 (`Myth and Legend`) | `AvXAltar` |
| 7 | Black Market | 154 / 105 | 27 / 44 / 34 | 31 | 6 (`Race for Ardintinny`) | `AVXmktb0` |
| 35 | Hill Fort | 146 / 69 | 21 / 25 / 23 | 25 | 11 (`Myth and Legend`) | `AVXhild0`, `AVXhilg0` (two masks each) |
| 99 | Trading Post | 204 / 87 | 32 / 32 / 23 | 24 | 9 (`Myth and Legend`) | `AVXpost0`, `AVXpstr0` |
| 221 | Trading Post (snow) | 33 / 27 | 0 / 10 / 17 | 1 | 2 (`Reclamation Allied`) | `AVXpsSn` |
| 106 | War Machine Factory | 258 / 104 | 20 / 41 / 43 | 32 | 7 (`Back For Revenge - Allied`) | `AVGsieg0` (two masks) |
| 213 | Freelancer's Guild | 86 / 40 | 0 / 20 / 20 | 14 | 7 (`Rediscovery`) | `avxfgld` |

- 148 maps hold one or more of the seven (T97c's 149 counted the map Tavern beside them).
- Every object has **subclass 0** and **no body** (`docs/formats/h3m.md`'s "every other class:
  nothing"); its template has **one visitable tile**, marked blocked in the mask. *Corrected by
  T127b* (increment 14's design note, preamble): the mask does not decide the visit mode, the
  original's class lists do (E08 decision 317's rule): `object_classes_not_entered` 0x63AA58 and
  `object_classes_from_above` 0x63AB00 list none of the seven, so **each is entered** (the hero
  stands on the entrance) and none from above; VCMI 1.7.5's configs agree (no `blockedVisitable`).
  T127a read "visited from a neighbouring tile, as a resource pile" from the mask. Every entrance
  stands on land (no water entrance); the 33 class-221 posts all on snow. The War Machine Factory's two masks differ only in where the entrance is
  (255 and 3 objects).
- **The Black Market:** one on 70 maps, two on 28, three on 2, four on 4, six on 1; 78 of its 105
  maps ban artifacts (a ban takes an artifact out of the market's draw, §7.1).
- The original loads class 221 as class 99 (`object_class_values` 0x63A6E4, E08 T103b's table):
  the visit never meets 221, and VCMI's config gives both one handler. The engine's import types
  221 as the Trading Post too (the map's own class kept for its picture, as 220 for the abandoned
  mine).

## 2. The visit (`visit_object`'s seven cases)

`visit_object` 0x4A84F0 dispatches on the tile's class; for these seven its last argument (the
hero's owner a human) decides:

| Class | A human | The computer | Marks |
|---|---|---|---|
| 7 Black Market (0x4A86AD) | `black_market_trade` 0x5EA100 with the hero and the market's record (§7) | `ai_black_market_visit` 0x5243F0 (not read; E10's, E13's) | none |
| 99 (and 221) Trading Post (0x4AA2CE) | `trading_post_trade` 0x5EA0C0 (§5) | nothing | none |
| 213 Freelancer's Guild (0x4A8E34) | `freelancers_guild_trade` 0x5E9E60 with the hero (§6) | nothing | none |
| 2 Altar of Sacrifice (0x4A8541) | its own dialog (§12) | nothing | the team's kind mark k 28 (game +0x4E360), a human's visit only |
| 35 Hill Fort (0x4A9172) | its own dialog (§13) | `ai_hill_fort_visit` 0x527BB0 (not read) | the team's kind mark k 14 (game +0x4E352), every visit |
| 106 War Machine Factory (0x4AA622) | ADVEVENT's question, then the hire dialog with the three war machines (§14) | `ai_war_machine_factory_visit` 0x524FC0 (not read) | none |

- No visit tests a day, a week, an owner or an earlier visit: a hero may trade at the same object
  every time it visits it (an entered object, corrected by T127b: stepping off and on again, or
  the engine's `Revisit`, ADV-TEL-008).
- No visit spends anything but the step onto the entrance: the window's trades take no movement
  points (none is read in the procedures of §5 to §7).

## 3. The trade window (shared with the town)

`market_window_run` 0x5EA130 runs one screen at a time; a screen's button for another screen ends
it with that screen's number, the close button with the window's end, and the loop opens the next.
Its inputs are five globals the entry sets:

| Global | What |
|---|---|
| `market_screen` 0x6AAAA4 | the first screen: 0 resources for resources, 1 resources to a player, 2 resources for artifacts, 3 artifacts for resources, 4 creatures for resources |
| `market_kind` 0x6AAAC4 | 0 a town's, 1 a Trading Post, 2 a Black Market, 3 a map's Freelancer's Guild: the screens' buttons and the title |
| `market_count` 0x6AAA98 | the Marketplaces counted (the rates' row, §4) |
| `market_hero` 0x6AAA78 | the hero whose army or artifacts trade (none at a Trading Post, none in a town without a visiting hero) |
| `market_artifacts` 0x6AAA74 | the seven artifacts for sale: the Black Market's record, else the Artifact Merchants' (game +0x1F664) |

The entries:

| Entry | Kind | Screen | Count | Hero |
|---|---|---|---|---|
| `trading_post_trade` 0x5EA0C0 | 1 | 0 | 5 | not set |
| `freelancers_guild_trade` 0x5E9E60 | 3 | 4 | 5 | the visitor |
| `black_market_trade` 0x5EA100 | 2 | 2 | 5 | the visitor |
| `town_marketplace_trade` 0x5E9FE0 | 0 | 0 | the player's towns with the Marketplace (building 14), at most 10 | the town's visiting hero, or none |
| `town_artifact_merchants_trade` 0x5E9D20 | 0 | 2 | the same | the town's visiting hero; without one a message and no window |
| `town_freelancers_guild_trade` 0x5E9EA0 | 0 | 4 | the same | the same |

**The screens a kind reaches** (the redraws' tests of the buttons): a Trading Post shows screens 0
and 1 (no artifacts, no creatures); a Black Market only screen 2; a map Freelancer's Guild only
screen 4 (the four buttons hidden); a town every screen, screens 2 and 3 only with a hero in a town
with the Artifact Merchants and screen 4 only with a hero in a Stronghold with its guild (the
town lane's to read: the buttons' tests name the factions 2 and 5 for the merchants and 6 for the
guild). The title follows the kind (the Marketplace's, the Trading Post's, the Freelancer's
Guild's name).

**Every trade is the current player's:** the treasury changed is the current player's record
(+0x9C, `player_record`); the army or the artifacts are `market_hero`'s. No procedure re-tests a
price against the treasury when the trade is clicked: the amount slider's most (§5 to §7) is the
only bound, so the engine's command validates the amount itself.

## 4. The rates

**Values.** `resource_values` 0x68C482 (u16 by resource, read by `resource_value` 0x5ECD10):
wood 250, mercury 500, ore 250, sulfur 500, crystal 500, gems 500, gold 1.
`resource_values_artifacts` 0x68C492, the purchase's: 125, 250, 125, 250, 250, 250, 1 (half,
gold kept). A creature's value is its gold cost (`creature_record` +0x38), an artifact's its cost
(`artifact_setup` +0x4, ARTRAITS.TXT's Cost).

**Factors** by the Marketplaces' count *m*, binary32 (index 0 unused: every entry counts 5 or a
town with its own Marketplace):

| *m* | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| resources R (`market_resource_rates` 0x678344) | 0.1 | 0.15 | 0.2 | 0.25 | 0.3 | 0.35 | 0.4 | 0.45 | 0.5 | 0.5 |
| artifacts A (`market_artifact_rates` 0x678370) | 0.2 | 0.25 | 0.3 | 0.35 | 0.4 | 0.45 | 0.5 | 0.55 | 0.6 | 0.6 |
| creatures C (`market_creature_rates` 0x67839C) | 0.3 | 0.45 | 0.5 | 0.65 | 0.7 | 0.85 | 0.9 | 1 | 1 | 1 |

**The four rates** (each step stored as binary32, `f32(…)`; `round(x)` = trunc(x + 0.5) in the
53-bit precision the game runs at; `1 / q` in 53 bits):

- **Resources** (`market_resources_proc` 0x5ECFE0; given g, wanted t): a = f32(v(g) × R),
  q = f32(v(t) / a). q ≥ 1: **round(q) of g for one t**; q < 1: **one g for round(1 / q) of t**.
- **Creatures** (`market_creature_rate` 0x5ECE80): a = f32(cost × C), q = f32(v(t) / a). q ≥ 1:
  **round(q) creatures for one t**; q < 1: **round(1 / q) of t for one creature** (q = 0 never:
  every value is above 0).
- **An artifact's price** (`market_buy_artifact_proc` 0x5ED9E0; paid in r): a = f32(w(r) × A),
  price = round(f32(cost / a)) of r, 0 when a or the cost is 0 (no buyable artifact: §7.2).
- **An artifact's sale** (`market_artifact_sale_rate` 0x5ECDC0; screen 3, the town's):
  max(1, f32(f32(cost × A) / v(r))) rounded.

**The halves fall on the binary32 side.** The rounding of a reciprocal decides where the exact
quotient is a half: a creature of gold cost 25 at five Marketplaces gives 17 gold (25 × 0.7 is
17.5, but the binary32 of 1 / 17.5 is above it), a wood at two Marketplaces 37 gold. The control
word's precision matters only for the last step, and 53 bits it is (§7.10's finding; under 24 bits
the 37 would be 38). So the engine either computes the steps as binary32 and 53-bit values through
`kernel::num` (ADR-0003) or holds the finite tables (resources: 10 counts × 42 pairs; creatures:
each gold cost × 7 resources × 10; artifacts: each cost × 7 × 10) checked against such a
computation.

**At five Marketplaces** (every map object):

| Give ↓ for → | wood | mercury | ore | sulfur | crystal | gems | gold |
|---|---|---|---|---|---|---|---|
| wood | | 7 : 1 | 3 : 1 | 7 : 1 | 7 : 1 | 7 : 1 | 1 : 75 |
| mercury, sulfur, crystal, gems | 2 : 1 | 3 : 1 | 2 : 1 | 3 : 1 | 3 : 1 | 3 : 1 | 1 : 150 |
| ore | 3 : 1 | 7 : 1 | | 7 : 1 | 7 : 1 | 7 : 1 | 1 : 75 |
| gold | 833 : 1 | 1667 : 1 | 833 : 1 | 1667 : 1 | 1667 : 1 | 1667 : 1 | |

(a : b = give a, get b; a precious resource for another precious one 3 : 1, for wood or ore 2 : 1:
q = 250 / 150 is at least 1, so round(q) given for one wanted; the first version of this table had
1 : 2 there, corrected at T132a1 when `market_resources_proc` was read again and the module's
tests ran every case against a binary32 model.) A creature sells for
round(1 / q) gold, 70 % of its cost as rounded above (a Pikeman's 60: 42; 25: 17; 75: 52); for
wood or ore one per creature from a cost of about 357 to 714 (and 8 wood for a 3 000-gold creature),
for a precious resource one per creature from about 714. An artifact costs 2.5 times its cost in
gold, cost / 50 wood or ore, cost / 100 of a precious resource (500: 1 250 gold, 10 wood, 5 gems;
10 000: 25 000, 200, 100).

## 5. The Trading Posts (99, 221)

`trading_post_trade` 0x5EA0C0: kind 1, screen 0, five Marketplaces, the hero not set (no screen it
reaches reads it).

- **Screen 0, the trade** (`market_resources_proc` 0x5ECFE0, `market_resources_redraw` 0x5EA6E0):
  the player picks a given and a wanted resource among the seven each side; while they are the
  same resource, or one is missing, the trade's controls are disabled. The rate is §4's. The
  slider counts **the wanted units** when q ≥ 1 (most: what the player has of g ÷ round(q)) and
  **the given units** when q < 1 (most: what the player has of g); Max sets the most; Trade
  subtracts and adds at once (an amount of 0 trades nothing). The window stays open for more trades.
- **Screen 1, the transfer** (`market_transfer_proc` 0x5ED550): a resource, a recipient among the
  colours other than the current player's whose defeat flag (game +0x1F636) is clear, and an
  amount from 1 to what the player has; Trade moves it **1 : 1** into the recipient's treasury.
  Allies and enemies alike. At a new game `new_game_heroes` 0x4BFE70 sets the flag of every colour
  nobody plays, so only players in the game are offered.
- From either screen the buttons lead only to the other (the artifacts' and the guild's are
  hidden for kind 1).

## 6. The Freelancer's Guild (213)

`freelancers_guild_trade` 0x5E9E60: kind 3, screen 4, the visitor, five Marketplaces; the other
screens' buttons hidden (`market_sell_creatures_redraw` 0x5EC550).

- **Screen 4, the sale** (`market_sell_creatures_proc` 0x5EE890): one of the hero's seven slots and
  a wanted resource; the rate is §4's with the slot's creature's gold cost. When one creature buys
  one or more units (q < 1) the slider counts **creatures** and Trade gives round(1 / q) of t for
  each; else it counts **units of t** and takes round(q) creatures for each. Most: the stack's count,
  **less one when the stack is the army's only one** (the hero keeps a creature), divided by
  round(q) in the second case. A stack sold to 0 leaves its slot empty. A right click on a slot shows
  the creature.
- Any creature sells, of any faction or level, upgraded or not (the procedure tests only the slot).
  The hero's war machines are not in its army and cannot be sold.

## 7. The Black Market (7)

### 7.1 The draw at a new game

`setup_map_objects` 0x4C0CC0 walks the entrances (level, row, column) and at case 7
(`black_market_draw` 0x4C0E31) draws **three treasures, three minors and one major**, in that order,
with `random_artifact_of_level` 0x4C94D0 (levels 2, 4, 8), appends the seven to the game's list
(`black_markets`, game +0x1F680; a `black_market` record of seven artifacts) and stores the record's
index in the object's word. The draw takes only artifacts the map does not ban and not yet given in
this game, and marks each as given: markets do not repeat each other nor the map's other random
artifacts until a level's pool is empty (then the pool restarts from the bans).

**Never restocked:** in play only the visit's purchase writes the list (an entry emptied); besides
the setup, only a saved game's save and load and the game's reset touch it. `new_month` 0x4C8F40
redraws the town Artifact Merchants' seven (game +0x1F664), not the markets'.

### 7.2 The purchase

`black_market_trade` 0x5EA100: kind 2, screen 2, the visitor, the market's record, five Marketplaces.

- **Screen 2** (`market_buy_artifact_proc` 0x5ED9E0, `market_buy_artifact_redraw` 0x5EB6A0): the
  seven entries (a sold one shown empty, not selectable) and the seven resources to pay with; the
  price is §4's (at five: 2.5 × cost in gold, cost / 50 wood or ore, cost / 100 a precious resource).
  Trade is enabled while the player has the price; a click buys **one**: the price paid, the artifact
  given to the hero (`hero_give_artifact` 0x4E3070: the first body position that takes it, else the
  backpack; in SoD a completed combination's question), the entry emptied. A right click shows the
  artifact.
- The cheapest artifact the draw can place costs 500 (ARTRAITS.TXT: treasures 500 to 5 000, minors
  3 000 to 6 000, majors 2 000 to 10 000), so no price is 0 or rounds to 0 (a price of 0 would make
  the slider's division by it fail).
- **A-46 (quirk candidate):** the payment and the entry's emptying do not test
  `hero_give_artifact`'s result: a hero with no body position for the artifact and 64 artifacts in
  the backpack pays and the artifact is lost. The same window serves the town's Artifact Merchants.

## 8. What a viewer sees

- None of the three classes has an owner or sets a mark (§2): the map shows each as its type, the
  same to every viewer; E24's hints by the team's kind marks have nothing to show for them.
- The Black Market's seven are shown only in the window, to the visiting human; no other view tells
  them. In the engine they are hidden information outside the visitor's open decision (the design is
  T127b's: what a viewer keeps of a market it visited).
- The Trading Post's transfer is seen by the recipient only as a change of its treasury (the
  original's network message aside).

## 9. VCMI and the wiki

**VCMI 1.7.5** (`IMarket::getOffer`, `markets.json`, `CGameHandler`):

- The factors agree: resources min((m + 1) / 20, 0.5), artifacts min((m + 3) / 20, 0.6) with the
  non-gold values halved for a purchase, creatures {0, 0.3, 0.45, 0.5, 0.65, 0.7, 0.85, 0.9, 1} by
  min(m, 8).
- The modes agree (`markets.json`): the Trading Posts (99 and 221) resource-resource and
  resource-player, the Freelancer's Guild creature-resource, the Black Market resource-artifact; every
  map market at **five** (the efficiency's default, `MarketInstanceConstructor`).
- **The rounding differs.** VCMI computes in doubles and, where one given unit buys several,
  takes the **ceiling** of the quotient; the original rounds half up after binary32 steps (§4).
  Over every case (the throwaway script): **resources** differ only at 2, 4 and 8 Marketplaces, wood
  or ore for gold (37 against VCMI's 38, 62 against 63, 112 against 113) and **not at five** (D-70,
  the town's); **creatures** differ at every count, at five in 99 of the 504 (gold cost, resource)
  cases (Pixies' 25: 17 gold against 18; a cost of 360 to 499 for wood or ore: 1 against 2 per
  creature; 3 000 for wood: 8 against 9) (D-71); **artifacts** agree at every count, bought (the 97
  treasures, minors and majors the draws place) and sold (the 122 artifacts not special). VCMI keeps a purchase at 1 or more; the original's never falls
  that low (§7.2).
- **The Black Market** (`CGBlackMarket::newTurn`, `rollMarketArtifactSet`): three treasures, three
  minors, one major, rolled on the first day; `blackMarketRestockPeriod` 0 (the default) never
  restocks: agrees (the roll's day differs, not what the game sees).
- `sellCreatures` refuses selling the army's last stack whole: agrees. `sendResources` requires the
  recipient in the game: agrees. `buyArtifact` gives the artifact at the first free position, the
  backpack unbounded by default: A-46's loss does not arise.

**The wiki:** the fandom page's creature percentages (30, 45, 50, 65, 70, 85, 90, 100 from eight)
and the map guild "as with five Marketplaces" agree; the mirror's Artifact Merchants percentages
(buy 500 % to 167 %, sell 20 % to 60 %, nine or more the best) agree with A; its Black Market line
(2.5 times the basic price, three treasures, three minors and one major once a game) agrees. VCMI's
modders' page names the Trading Post's transfer to another player. No source states a rounding.

## 10. Disputes, quirk candidates and open points

- **D-70** (the town lane's; the map's Trading Posts unaffected): wood or ore for gold at 2, 4 and
  8 Marketplaces, the original's 37, 62, 112 against VCMI's ceiling 38, 63, 113. The code reading
  and VCMI disagree, the wiki is silent: the town lane's to settle (its inbox told, §11).
- **D-71** (the map's Freelancer's Guild and the town's): the creature sale's rounding, the
  original's half up after binary32 against VCMI's ceiling (99 of 504 cases at five, many at a
  creature's resource rate: one wood instead of two for a 360-gold creature). Recommended: the
  original's (the code reading of this build; a batch E14 row to confirm: E14-1, §18).
- **A-46** (quirk candidate, the Black Market and the town's Artifact Merchants): the price taken
  and the artifact lost when the hero can hold nothing more. Options: as the original (the loss), or
  fixed (the purchase refused before paying, as a pickup with a full backpack, GAM's rule).
  Recommended: fixed; the user classifies it at the checkpoint (an ambiguous quirk).
- **Open for T127b** (the design): the window as a decision open at the object (as increment 10's
  hire dialog) or as commands valid while the hero stands beside it; the slider's amount and its
  most as the command's validation (§3: the original re-tests nothing); the rates' representation
  (§4); what a viewer keeps of a Black Market's seven; the transfer's event for the recipient.

## 11. For the town lane (its inbox told)

The market's rates are this window's: the engine builds them once (decision 361). What the town's
Marketplace (E09 increment 5) reuses: §3's screens and their kinds; §4's values, factor tables and
rounding (the count is the player's towns with the Marketplace, at most 10); §5's trade and
transfer; §6's sale (the town's guild at the player's count); §7.2's purchase (the Artifact
Merchants' seven at game +0x1F664, `new_month` 0x4C8F40's draw) with A-46. Not read here: screen 3
(the artifact sale's procedure `market_sell_artifact_proc` 0x5EDF60; its rate,
`market_artifact_sale_rate`, is §4's) and the merchants' monthly draw. D-70 is the town's alone;
D-71 and A-46 are shared.

## 12. The Altar of Sacrifice (2)

### 12.1 The dialog and its two screens

`visit_object`'s case 2 (0x4A8541), a human only: `altar_dialog_create` 0x55FDD0 with the hero,
`altar_dialog_run` 0x5653B0, the destructor (`altar_dialog_destroy` 0x5623A0), then the team's kind
mark k 28 (§2). The computer does nothing there.

- **The screens by alignment.** The dialog reads the faction of the hero's class (hero +0x30, the
  class record's first word): **artifacts allowed** (+0x75) for factions 0 to 2 and 6 to 8,
  **creatures allowed** (+0x76) for 3 to 8. The run opens the artifacts' screen
  (`altar_show_artifacts` 0x562A20) when artifacts are allowed, else the creatures'
  (`altar_show_creatures` 0x563150); each screen's button to the other is enabled by the other's
  flag. So **Castle, Rampart and Tower** heroes sacrifice artifacts only, **Inferno, Necropolis and
  Dungeon** heroes creatures only, **Stronghold, Fortress and Conflux** heroes both, starting on the
  artifacts. Leaving a screen undoes it: every artifact taken goes back to the hero and the total is
  cleared (`altar_to_creatures` 0x564910, `altar_to_artifacts` 0x564E70); the creatures' amounts are
  cleared when their screen opens.
- **The total** (+0x78) is what Sacrifice gives: `altar_update` 0x562500 shows it with the
  experience the hero lacks for its next level, and **enables Sacrifice only while the total is
  above 0**.
- **Sacrifice** (`altar_sacrifice` 0x5646A0) destroys what the screen offers (§12.2, §12.3), then
  calls `hero_gain_experience` 0x4E33B0 with the total, the levels given and the owner told (its
  level-up dialogs over the altar's), clears the total and **leaves the dialog open**: the player
  may sacrifice again, and with the levels' new Learning. No visit limit (§2): every visit opens
  the dialog. **Done** (`altar_done` 0x564A80) returns every artifact taken and closes.

### 12.2 The artifacts

- **Where they come from:** the hero's worn positions (18; 19 in SoD, game +0x1F698 ≥ 2, the fifth
  misc) and its backpack (64). The altar has **22 places** (`altar_slot_rows` 0x6412C0, five rows;
  `altar_slot_row_counts` 0x6412E8: 5, 5, 5, 5, 2) and the cursor holds one more.
- **Taking one** (`altar_worn_click` 0x5632A0, `altar_backpack_click` 0x5636C0): a worn artifact is
  taken off the hero (`hero_remove_artifact` 0x4E2BD0, its primary skills with it) onto the cursor,
  **except the Spell Book** (the click opens the book) **and the Catapult** (a message); so the
  Ballista, the Ammo Cart and the First Aid Tent may be taken. Any backpack artifact may be taken:
  the Grail, a Spell Scroll. The cursor's artifact goes onto an altar place (`altar_place_click`
  0x563A80) or back onto the hero, a position its kind allows (`hero_can_replace_artifact`
  0x4E2840), its experience leaving the total. `altar_backpack_to_altar` 0x564060 moves the
  backpack's artifacts in order onto the free places; `altar_all_to_altar` 0x564340 the worn
  positions 0 to 15 (the three machines among them; not the Catapult 16, the book 17 nor the fifth
  misc 18), then the backpack, both until the places run out. No class is tested.
- **The experience** (`altar_artifact_entry` 0x55FC30 and the clicks' copies of it), when an
  artifact is taken: by its class (`artifact_setup` +0xC): **treasure 1 000, minor 1 500, major
  3 000, relic 6 000, special 0** (the seven special artifacts: the Spell Book, a Spell Scroll, the
  Grail and the four war machines, ARTRAITS.TXT's class S), times the hero's Learning factor
  (`hero_learning` 0x4E4840) as HER-EXP-001's product, **trunc(L × f32(base)) for each artifact**,
  added to the total. The SoD combinations are relics (the `sod` pack): 6 000.
- **Sacrifice:** every artifact on the altar and the one on the cursor is destroyed. **Done** and
  the switch to the creatures return each taken artifact to its former worn position, else any
  position that takes it, else the backpack (always room: it came from the hero).
- A sacrifice of a special artifact alone cannot be made (a total of 0), but with others it is
  destroyed for nothing (**A-47**).

### 12.3 The creatures

- **The amounts** (+0x138 by slot): `altar_select_creature_slot` 0x564FE0 sets the slider's most to
  the stack's count, **less one when every other slot's amount is its whole count or the slot is
  empty**: the army keeps one creature. `altar_set_creature_amount` 0x564C00 sets a slot's amount
  and moves the total by the difference of **trunc(L × f32(amount × e))**, e = **5 × ⌊AI value /
  40⌋** (`creature_record` +0x40; the division signed, every value positive): the Learning
  truncated per stack. `altar_all_creatures` 0x564CF0 takes every stack whole from slot 6 down to
  0, the last processed less one (the lowest occupied slot keeps a creature); `altar_creature_max`
  0x564DE0 the selected slot's most.
- **Sacrifice:** each slot's count lowered by its amount, a slot at 0 cleared (`army_clear_slot`
  0x44AB60), the experience as above. Any creature, of any faction, level or upgrade; the war
  machines are not in the army.
- At five Pikemen (AI value 80): 10 each, 50; an Angel (5 019): 625; a Skeleton (60): 5.

### 12.4 The turn's time (A-48)

When the turn's time runs out, `dialog_proc_base` 0x41B1C0 calls the dialog's vtable +0x38
(`altar_dialog_vtable` 0x641620): `altar_time_expired` 0x565430 returns **only the artifact on the
cursor** and closes; the destructor returns nothing. The artifacts lying on the altar are lost, for
no experience (**A-48**; a timed game only).

## 13. The Hill Fort (35)

`visit_object`'s case 35 (0x4A9172): a human `hill_fort_dialog_create` 0x4E75F0 (its controls, not
read) and `hill_fort_dialog_run` 0x4E7E90 (`hill_fort_rows` 0x4E7EB0, then the modal loop with
`hill_fort_dialog_proc` 0x4E8850); the computer `ai_hill_fort_visit` 0x527BB0 (not read); then
every visit the team's kind mark k 14 (§2). The army is the current player's selected hero's, the
visitor's.

- **Which stacks** (`hill_fort_rows`, per slot): a stack upgrades when its creature is a town's
  unupgraded one (`creature_is_unupgraded` 0x47B120: the town creature table holds it as a base)
  and, in a game of version 0 (RoE, game +0x1F698: an RoE campaign or a peer with RoE alone, never a
  map's format in this edition, §21), it is not one of the four elementals (Air,
  Earth, Fire, Water: 112 to 115; their upgrades are AB's); its upgrade is `creature_upgrade`
  0x47B1A0's. Neutral and upgraded creatures do not.
- **The price:** `upgrade_cost` 0x54E750 (also the town's creature dialog's and the computer's):
  for each of the seven resources max(0, the upgrade's cost − the creature's) **× the count**; then
  **the gold alone** becomes **trunc(f32(gold) × factor)**, the factor by the creature's level
  (`creature_record` +4, `hill_fort_gold_factors` 0x63EB4C): **0, 0.25, 0.5, 0.75, 1, 1, 1** for
  levels 1 to 7. A level-1 stack's price is not computed at all: **free**, a text in its place. The
  other resources are not scaled; no town creature of levels 1 to 4 costs more of another resource
  than its base (the packs), so the gold decides. 10 Archers (50 more gold each): 125 gold;
  3 Pegasi (25 more): trunc(75 × 0.75) = 56.
- **The states:** each row 0 (no upgrade), 1 (every resource of its price within the treasury) or 2;
  the whole (+0x298) 1 when the sum of the rows' prices is affordable and a row upgrades, 0 when
  none upgrades, else 2.
- **The upgrade** (`hill_fort_dialog_proc`): a row's button (items 261 to 267) calls
  `hill_fort_upgrade_slot` 0x4E8500: state 1 replaces the stack's creature by its upgrade (the count
  kept) and pays the row's price from the treasury (+0x9C); state 0 or 2 shows a message, nothing
  else. **All** (item 204): the whole's state 1 upgrades every row of state 1 in slot order; 0 or 2
  a message, nothing. After either the rows are computed again. Nothing is tested again at the
  click, so the states are the only bound; no limit per visit, no movement spent.

## 14. The War Machine Factory (106)

`visit_object`'s case 106 (0x4AA622): the computer `ai_war_machine_factory_visit` 0x524FC0 (not
read); a human ADVEVENT's question, a yes and the hire dialog (`H3RecruitMgr`, increment 10's,
`E08-dwellings.md` §5) for the hero with three creatures in this order: the **Ballista** (146), the
**First Aid Tent** (147), the **Ammo Cart** (148), the three sharing one count of 4 000; no mark.

- **The most** (`recruit_dialog_select` 0x5503A0): for a war machine (creature flag 0x40) the count
  is first set to 1 less whether the hero holds its artifact (`hero_has_artifact` 0x4D91B0;
  `war_machine_creature` 0x550360 maps the artifacts 3 to 6 to the creatures 145, 146, 148, 147),
  at least 0; the most is then the least of the count and the gold over the cost: **1 or 0**.
- **The prices:** the creatures' gold costs: Ballista 2 500, First Aid Tent 750, Ammo Cart 1 000.
- **The purchase** (`recruit_dialog_proc` 0x550940's Recruit): the gold paid, `hero_give_artifact`
  0x4E3070 with the machine's artifact (4, 6, 5) into its position; three creatures, so the dialog
  stays open for the others. The Catapult is never offered.

## 15. VCMI and the wiki (part 2)

**VCMI 1.7.5:**

- **The Altar's screens:** `markets.json` gives the Altar both experience modes; `CMarketWindow`
  blocks the creatures for a good hero and the artifacts for an evil one, the alignment being the
  hero's class's faction's (`config/factions`: Castle, Rampart, Tower good; Inferno, Necropolis,
  Dungeon evil; Stronghold, Fortress, Conflux neutral): **agrees**. Its server refuses an evil
  hero's artifacts and tests no alignment for creatures (the client's only).
- **The values** (`IMarket::getOffer`): `CREATURE_EXP` (AI value / 40) × 5 per creature;
  `ARTIFACT_EXP` 1 000, 1 500, 3 000, 6 000 by class, any other class 0: **agree**.
- **Which artifacts:** `sacrificeArtifact` takes only a tradable one (`CArtifact::isTradable`: not
  the Spell Book, not a war machine, `isBig`); a Spell Scroll and the Grail are tradable and sum 0.
  So VCMI refuses the Ballista, the Ammo Cart and the First Aid Tent the original takes for 0
  (**D-72**); the Grail and a scroll for 0 agree (A-47's case).
- **The Learning:** VCMI sums a sacrifice's experience and applies `calculateXp` once (× the
  experience percentage / 100.0, truncated); the original truncates per artifact and per stack. With
  Learning the original's is lower by up to one per artifact or stack after the first (**D-73**):
  Advanced Learning, 3 and 1 Skeletons in two stacks: 16 + 5 = 21 against 22.
- **The last creature:** `sacrificeCreatures` refuses to empty the last stack: **agrees** (one kept).
  The 22 places: VCMI's bug 558 reports the original's 22 artifacts per sacrifice: **agrees**.
- **The Hill Fort** (`generic.json`'s `upgradeCostFactor` 0, 25, 50, 75, 100 from level 5;
  `HillFort::fillUpgradeInfo`): **the factors agree**. `UpgradeInfo::addUpgrade` scales every
  resource of the difference per creature, truncating (× percentage / 100), and
  `CGameHandler::upgradeCreature` multiplies by the count; the original truncates the stack's gold
  once and scales no other resource (**D-74**). With the packs' costs only the gold differs: 11
  pairs at levels 2 to 4 (Archer, Dwarf, Wood Elf, Pegasus, Stone Gargoyle, Gog, Walking Dead,
  Medusa, Orc, Lizardman, Basilisk) whenever the count is not a multiple of 2 or 4: 10 Archers 125
  against 120, 3 Wood Elves 37 against 36, 3 Pegasi 56 against 54. Which stacks: VCMI's creatures'
  `upgrades` (none for a neutral): agrees; the RoE elementals' test not checked.
- **The factory** (`generic.json`: ballista, firstAidTent, ammoCart; `CGDwelling`: one of each the
  hero lacks, the question ADVOB_TXT 157; `recruitCreatures`: the Catapult refused, the machine
  given): **agrees**.

**The wiki** (search snippets): the mirror's list of objects: good heroes sacrifice artifacts only,
evil ones creatures only, neutral ones both (agrees); the RoE manual's page 117: the Ammo Cart
1 000, the Ballista 2 500, the First Aid Tent 750, one of each per hero (agrees); the mirror's war
machine page: the factory sells every machine but the Catapult (agrees). No page read gives the
Altar's values or the Hill Fort's percentages: VCMI and the code reading are their two sources.

## 16. Disputes, quirk candidates and open points (part 2)

- **D-72** (the Altar): the Ballista, the Ammo Cart and the First Aid Tent sacrificed for 0 (the
  original) or refused (VCMI). Recommended: the original's, with A-47's answer (a batch E14 row).
- **D-73** (the Altar): the Learning truncated per artifact and per stack (the original) or once
  over the sum (VCMI). Recommended: the original's (HER-EXP-001's product per item; a row).
- **D-74** (the Hill Fort): the stack's gold scaled and truncated once, the other resources whole
  (the original), or every resource per creature (VCMI). Recommended: the original's (a row).
- **A-47** (quirk candidate): a special artifact (the Grail, a Spell Scroll, the Ballista, the Ammo
  Cart, the First Aid Tent) destroyed for no experience beside others on the altar. Options: as the
  original (recommended: nothing is corrupted, the total shown before the press is the player's
  choice), or refused (the engine's Altar takes only the four classes; VCMI refuses the machines
  only).
- **A-48** (quirk candidate): the artifacts on the altar lost when the turn's time runs out.
  Options: fixed (recommended: in the engine a sacrifice names its artifacts in one answer, so no
  artifact lies on an altar between commands; the decision's default answer at the time's end
  leaves with everything kept), or as the original (a held list lost at the default answer).
- **Open for T127b** (the design; part 1's in §10):
  - **The levels inside the Altar.** The original gives a sacrifice's levels at once, the dialog
    open beneath them, and a later sacrifice reads the Learning they brought. The engine's player
    has one open decision (ADR-0017 point 6). Options: each sacrifice closes the Altar's decision,
    its levels come (GAM-DEC-002) and the Altar opens again as their rest (GAM-DEC-007; the
    original's order), or the levels wait until the Altar is left (a Learning specialist's later
    sacrifices read the old level).
  - **The answers' shape:** a sacrifice of artifacts by position (worn or backpack), bounded by the
    22 places and the cursor (23) or not (the same experience once the levels come between
    sacrifices); a sacrifice of creatures by an amount per slot.
  - **The Hill Fort's two answers** (a slot, all) and its view (each slot's price and state).
  - **The factory** as the hire dialog with a source of war machines (GAM-DEC-008, ADV-HIR-002): a
    machine's purchase gives its artifact (no army room), its most 1 or 0.

## 17. The interfaces' sketch (the seven classes)

For T127b's drafts and design note (ADR-0015 to -0017, ADR-0013); a sketch, not decided.

- **Commands:** none new for the visits: a move's last step onto the object's entrance (every
  one is entered, §1 as corrected by T127b) or `Revisit`. Each object's window as **a decision of the
  visitor's owner** (GAM-DEC-001), opened by the visit as the University (GAM-DEC-006) and the hire
  dialog (GAM-DEC-008) are, with its kind's answers (recommended: the original's windows are modal,
  the decision holds the trade's context, the default answer closes it). Not chosen: commands valid
  while the hero stands beside the object (no open state, but every trade re-proves the object and
  nothing ends it). Every player answers: the original's human-only branches (§2) are the computer
  player's behaviour (E10), not a rule of the engine.
  - **The market** (`Decision::Market`; 99, 221, 213, 7): the screens by the object's kind (a Trading
    Post 0 and 1, the guild 4, the Black Market 2; the town's Marketplace E09's); answers
    `Trade { give, take, amount }` (screen 0, the amount in the slider's unit, §5), `Transfer {
    resource, amount, to }` (screen 1), `Sell { slot, take, amount }` (screen 4, §6), `Buy {
    entry, pay }` (screen 2, §7.2), `Leave`. The validation re-tests what the original's slider
    bounds (§3): the amount 1 to the most, the treasury, the recipient in the game, the army's
    last creature, the entry not sold. The rates in the ruleset, built once with the town lane
    (decision 361; §4's binary32 steps through `kernel::num` or checked tables).
  - **The Altar** (`Decision::Altar`): the screens the hero's alignment allows (§12.1); answers
    `Artifacts(positions)`, `Creatures(amounts by slot)`, `Leave`; the view the total each would
    give and the experience to the next level.
  - **The Hill Fort** (`Decision::HillFort`): answers `Upgrade(slot)`, `All`, `Leave`; the view each
    slot's upgrade, price and state (§13).
  - **The factory:** the hire question (the offer, as a dwelling's) and `Decision::Hire` with a
    source of three machines; `HireAnswer::Buy { creature, amount: 1 }` gives the artifact.
- **Events:** a window's opening (`MarketOpened`, `AltarOpened`, `HillFortOpened`: the hero, the
  object by type and anchor) and closing (`DecisionClosed`); the resources paid and gained
  (`ResourcePaid` and its gain); the transfer told to the recipient (its treasury's change, from
  whom); a stack sold, sacrificed or upgraded (`StackChanged`); an artifact bought, taken or
  sacrificed (the hero's changes); the experience and its levels; a Black Market's entry emptied (to
  the visitor). A new variant of `h3-game`'s or `h3-adventure`'s events breaks `h3-protocol`'s build
  until mapped (the global trap: `cross-lane`, or the server lane first).
- **The observation:** every class's type to every viewer of its tile; no owner; the Altar's and the
  Hill Fort's kind marks for the hints (k 28, k 14; ADV-VIS-006, increment 12's marks); a Black
  Market's seven only in the visitor's open decision, server-only otherwise; what a viewer keeps of
  a market it visited: nothing (recommended, as the original) or the last seen seven (a knowledge
  by object, ADV-VIS-006). The rates are the ruleset's, public. No flat encoding (decision 187).
- **State and versions:** the Black Markets' records drawn at the build (the adventure state hash;
  their place in the build's order of draws, `setup_map_objects`' walk, is T127b's), the kinds and
  rates (the adventure ruleset part, perhaps the core's for the shared rates), the seven classes'
  import typed (map import version; 221 as the Trading Post, its class kept), the decisions (game
  state hash). Replays: new answers, new words (adventure replay format). The env (the AI lane's):
  new decision kinds to encode; `legal_commands` lists one answer per pair or slot at its most, as
  the hire dialog does (every amount would be thousands).
- **Benchmarks:** the 1 058 objects stop being inert blockers on 148 shipped maps (their
  entrances end paths, ADV-OBJ-003; T127b's correction), and their answers enter the random agent's draws (the
  lane's trap: every random game's course moves): the `random-play-*` workloads re-baselined with
  the reason; a market-heavy workload if T127b's cut wants one (`homm3-bench-regression`).

## 18. Batch E14's plan

Increment 14's batch, **batch E14**, runs on the client lane's harness in SoD on Easy, hot seat,
from maps `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E13; its spec and
recipe are tasks of T127b's cut. Two players (Blue for the transfer), days 1 and 2, and day 29 for
E14-3 (28 days' turns ended); every price, rate and total read from the window before the click.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E14-1 | §6, D-71 | Red's hero sells at a Freelancer's Guild 10 Pixies for gold, then 2 Vampires for wood | the treasury after each | 170 gold (VCMI 180); 2 wood (VCMI 4) |
| E14-2 | §6 | the guild with the hero's only stack of 10 | the slider's most | 9 |
| E14-3 | §7.1 | two Black Markets on a map that bans some artifacts | each market's seven at day 1 and day 29 | three treasures, three minors, one major, no banned one, the two markets distinct; unchanged at day 29 |
| E14-4 | §7.2, A-46 | Red's hero with every body position full and 64 in the backpack buys at a Black Market | the treasury, the hero's artifacts, the entry | the price paid, the artifact nowhere, the entry empty |
| E14-5 | §4, §5 | Red at a Trading Post: 2 wood for gold, 833 gold for a wood, 3 mercury for a gem | the treasury | +150 gold; +1 wood; +1 gem |
| E14-6 | §5 | Red transfers 5 ore to Blue at a Trading Post | both treasuries | Red −5, Blue +5 |
| E14-7 | §12.1 | a Castle, a Necropolis and a Fortress hero visit an Altar | the first screen; the other screen's button | artifacts, disabled; creatures, disabled; artifacts, enabled |
| E14-8 | §12.2 | a Castle hero without Learning sacrifices a treasure, a minor, a major and a relic | the experience before and after | +11 500 |
| E14-9 | §12.2, D-72, A-47 | a Fortress hero puts its Ballista, a Spell Scroll and the Grail with a treasure on the altar, then clicks its Catapult | the total; the artifacts after Sacrifice; the Catapult's click | 1 000; the four gone; a message, the Catapult kept (VCMI: the Ballista refused) |
| E14-10 | §12.3, D-73 | a Necropolis hero with Advanced Learning sacrifices 3 Skeletons and 1 Skeleton in two slots, a third stack kept | the total and the experience | 21 (VCMI 22) |
| E14-11 | §12.3 | the same hero's only stack of 10 Skeletons | the slider's most; all's amounts | 9; 9 |
| E14-12 | §13, D-74 | Red's hero with Pikemen, 10 Archers, 3 Wood Elves, 3 Pegasi and 1 Angel at a Hill Fort | each row's price; the treasury after all | 0; 125; 37; 56; 2 000 gold and 2 gems (VCMI 120, 36, 54); the sum paid |
| E14-13 | §13 | a hero with Rogues, Marksmen and a Pegasus stack at a Hill Fort, too poor for the Pegasi | the rows' states; all's message | Rogues and Marksmen not upgradable; the Pegasi state 2; all refused, nothing paid |
| E14-14 | §13 | a RoE map: a hero with Air Elementals at a Hill Fort | the row | not upgradable |
| E14-15 | §14 | a hero with a Ballista at a War Machine Factory buys what it can, then revisits | the dialog's counts; the treasury; the positions | Ballista 0, the tent and the cart 1; −1 750; both equipped; then all 0 |

**Not in the batch:** the computer's visits (E10), network play, the town's Marketplace (the town
lane's), A-48 (a timed turn: only if the harness can set the turn's duration in the original's hot
seat; else left to the reading). **What the harness lacks** (`clients/capture/`): reads of the
market window (the screen, the selected pair, the slider's most), of the Altar's dialog (+0x78 the
total, the places, the slider's most), of the Hill Fort's rows (prices, states, the whole) and of
the Black Markets' records (game +0x1F680); the hero's experience, artifacts and army and the
treasury (earlier batches' reads). **What the spec lacks** (`h3-experiment`, battle-owned:
`cross-lane`; the writer is this lane's): the seven classes by template (no body), a hero's
secondary skills, war machines and backpack at the start (64 artifacts for E14-4, the Grail and a
scroll for E14-9), a map's banned artifacts (E14-3), a RoE map (E14-14).

## 19. The proposed rule ids

Four families and three decisions (GAM-DEC-010 is the town lane's tavern):

- **ADV-MKT** the map's markets: **-001** the objects (99, 221 as 99, 213, 7: one entrance each,
  entered (T127b's correction), no body; visited every time, no mark, no owner); **-002** the rates (the
  values, the three factor tables, the binary32 steps and half-up rounding; built once with the
  town lane, decision 361); **-003** the Trading Post's trade (screen 0 at five Marketplaces);
  **-004** the transfer (screen 1: 1 : 1 to a player in the game); **-005** the Freelancer's
  Guild's sale (screen 4, the army keeps a creature, D-71); **-006** the Black Market's draw at the
  build (three, three, one; the bans; once per game, never restocked); **-007** its purchase (the
  price, one at a time, the entry emptied, A-46); **-008** what a viewer sees (the stock hidden,
  §8).
- **ADV-ALT** the Altar of Sacrifice: **-001** the visit (the window of the visitor's owner, the
  screens by the class's faction, the kind mark k 28); **-002** the artifacts (which, the experience
  by class, the Learning per artifact, D-72, A-47, the levels); **-003** the creatures (5 × ⌊AI value
  / 40⌋, the Learning per stack, D-73, the army keeps a creature).
- **ADV-HFT** the Hill Fort: **-001** the visit and which stacks (a town's unupgraded creature; the
  RoE elementals; the kind mark k 14); **-002** the price (the difference times the count, the gold
  by level, truncated per stack, D-74) and the upgrade (a slot, all when the whole is affordable).
- **ADV-WMF** the War Machine Factory: **-001** the question and the machines (one of each the hero
  lacks, the gold cost, the artifact into its position).
- **GAM-DEC-011** the market's decision, **GAM-DEC-012** the Altar's, **GAM-DEC-013** the Hill
  Fort's.

Also amended: GAM-DEC-008 and ADV-HIR-002 (a source of war machines), GAM-DEC-007 (the Altar's
reopening after its levels, if T127b so designs), ADV-OBJ-003 (the seven's entrances end a path),
ADV-VIS-006 (the two kind marks' hints), HER-EXP-001 (the Altar's products); the town lane's rules
when E09 increment 5's Marketplace reuses ADV-MKT-002 to -007.

## 20. For T127b: the checkpoint's questions

Asked at T127a2's end (`docs/OPEN-QUESTIONS.md`, ADV-Q57; the answers in the epic as decisions from
365):

1. **The roundings, D-71, D-73, D-74:** the original's in each (recommended: the code reading of
   this build, each confirmed by a batch E14 row), or VCMI's ceilings and per-creature steps.
2. **The losses, A-46 and A-48:** both fixed (recommended: a Black Market purchase refused before
   paying when the hero cannot take the artifact, as a pickup's room rule; the Altar's default
   answer keeps everything), or as the original.
3. **The special artifacts at the Altar, A-47 and D-72:** as the original (recommended: the Grail,
   a Spell Scroll and the three machines taken for 0 beside others), refused (the four classes
   only), or VCMI's (the machines refused).
4. **Batch E14:** before the build (recommended, as batches E9 to E13), after it, or not at all.

D-70 is the town lane's (§10, §11). The open points of §10 and §16 are T127b's calls.

**Answered** (2026-11-18, `AskUserQuestion`, one round; the epic's decisions 365 to 368; ADV-Q57):
(1) D-71, D-73 and D-74 **the original's** in each; (2) A-46 and A-48 **both fixed**; (3) the special
artifacts **as the original** (A-47 harmless and accepted, D-72 the original's); (4) **batch E14 before
the build**. All as recommended.

## 21. Batch E14 scored (T130)

Scored on 2026-11-19 (E08 T130; docs and one code reading) from the recorded runs
`experiments/results/E08-batch-14.md` (T129b's fourth run of the whole recipe, every section; its
third alike in every service row) and `-14-roe.md` (E14-14, one run), with T129a's two runs of the
markets (`E08-log.md`, T129a: alike in every rule-level record, the draws differing by start),
played by the capture harness (`clients/capture/batches/E08-batch-14.toml`, 33 steps, and
`E08-batch-14-roe.toml`) in SoD 4.0 on Easy, hot seat, and read from the game's memory. Each row
against §18's prediction with design §20's changes and the specs' headers
(`experiments/specs/E08-batch-14.toml`, `-14-roe.toml`). E14-2 is read in E14-1's visit, E14-7 in
E14-8's, E14-9's and E14-10's, E14-11 in E14-10's, E14-16 in every visit.

| Row | Predicted | Observed | Verdict |
|---|---|---|---|
| E14-1 (§6; D-71) | 10 Pixies 170 gold; 2 Vampires 2 wood (VCMI 180; 4) | the window screen 4, kind 3; 1 Pixie for 17 gold, the most 10: +170 gold, slot 0 emptied; 1 Vampire for 1 wood, the most 2: +2 wood; no points spent | **D-71 confirmed**: the original's half-up rounding after the binary32 steps (decision 365) |
| E14-2 (§6) | the only stack of 10: the most 9 | the Skeletons (the only stack left) the most 9, 42 gold each | confirmed: the army keeps a creature |
| E14-3 (§7.1) | each market three treasures, three minors, one major, none banned, the two distinct; unchanged at day 29 | M1 and M2 each 3, 3, 1 in that order, all from the seventeen allowed, distinct (and different between runs); day 29 (month 2, week 1, day 1): both records as on day 1 but M1's sold entry, still empty | confirmed: drawn once at the new game, never restocked, a sold entry stays empty |
| E14-4 (§7.2; A-46) | the price paid, the artifact nowhere, the entry empty | 5 000 gold for the Stoic Watchman (a treasure of cost 2 000; the most 6): no question, no message, −5 000, the 15 worn and the 64 in the backpack unchanged, the entry emptied in the window and the record | **A-46 seen** as read: the engine refuses that purchase before paying (decision 366); the row confirms what is fixed |
| E14-5 (§4, §5) | +150 gold; +1 wood; +1 gem | screen 0, kind 1, the transfer's button shown; wood → gold 1 for 75 (most 32): −2 wood, +150 gold; gold → wood 833 for 1 (most ⌊gold ÷ 833⌋ = 31): −833, +1 wood; mercury → gems 3 for 1 (most 5): −3, +1 gem | confirmed: §4's rates at five Marketplaces, trades by whole lots |
| E14-6 (§5) | Red −5 ore, Blue +5 | the transfer's screen offers Blue alone; 5 ore: Red 30 → 25, Blue 30 → 35, Blue's the same on its turn | confirmed: 1 : 1 to a player still in the game |
| E14-7 (§12.1) | Castle artifacts only; Necropolis creatures only; Fortress both, the artifacts first | E14-8 (Castle) the artifacts' screen, the creatures' button disabled; E14-10 (Necropolis) the creatures', the artifacts' disabled; E14-9 (Fortress) the artifacts' first, the creatures' enabled | confirmed |
| E14-8 (§12.2) | +11 500 (1 000, 1 500, 3 000, 6 000) | 68 700, level 19; the backpack's button put the four on: 1 000, 1 500, 3 000, 6 000 = 11 500; Sacrifice: no message, 80 200, level 19, the backpack empty, the Altar open | confirmed |
| E14-9 (§12.2; D-72, A-47) | 1 000; the four gone; the Catapult kept with a message (VCMI: the Ballista refused) | the Ballista, the Spell Scroll (Haste, from the misc position) and the Grail 0 each, the Talisman of Mana 1 000: total 1 000; the Catapult's click GENRLTXT 483, nothing held, the Catapult stays; Sacrifice: 2 100 → 3 100, all four gone | **D-72 confirmed** (the original's), **A-47 seen** (harmless and accepted, decision 367) |
| E14-10 (§12.3; D-73) | 21 (VCMI 22) | Advanced Learning; slot 1's most 1; 3 and 1 Skeletons 16 + 5 = 21; Sacrifice: 21, both slots cleared, the third stack kept | **D-73 confirmed**: the Learning per stack, each truncated (decision 365) |
| E14-11 (§12.3) | the most 9; All 9 | the only stack's most 9; All: 9, total 49; Done: nothing given | confirmed: the army keeps a creature; leaving gives and loses nothing |
| E14-12 (§13; D-74) | 0; 125; 37; 56; 2 000 gold and 2 gems; four Air Elementals 25 (VCMI 120, 36, 54; 24) | the Pikemen free (GENRLTXT 345), 10 Archers 125, 3 Wood Elves 37, 3 Pegasi 56, the Angel 2 000 gold and 2 gems, 4 Air Elementals 25; every row state 1, the whole 2 243 gold and 2 gems, state 1; All: no message, every stack upgraded, −2 243 gold, −2 gems, every row then GENRLTXT 346, state 0 | **D-74 confirmed**: the stack's gold scaled once and truncated, the other resources whole (decision 365); the SoD side of E14-14 |
| E14-13 (§13) | Rogues and Marksmen not upgradable; the Pegasi state 2; All refused, nothing paid | the Rogues and the Marksmen state 0; 3 000 Pegasi 56 250 gold, state 2; the whole state 2; the Pegasi's button GENRLTXT 315, All GENRLTXT 317, nothing changed | confirmed: neutral and upgraded creatures do not upgrade; the states are the only bound |
| E14-14 (§13) | an RoE map: the Air Elementals not upgradable (game version 0); the Pikemen free | the game's version **SoD** (game +0x1F698 2) on the RoE-format map (header 0x0E); the Air Elementals 31 gold, state 1, upgraded to Storm Elementals (−31 gold, no message); the Pikemen free, to Halberdiers | **the finding** (below): the reading's version-0 branch holds, but an RoE map does not take version 0; **decision 378**: the enabled packs decide |
| E14-15 (§14) | Ballista 0, the tent and the cart 1; −1 750; both equipped; then all 0 | ADVEVENT 157 on (29, 18) with 1 360; the Ballista 0 available (2 500), the First Aid Tent 1 (750), the Ammo Cart 1 (1 000); both bought: −1 750, positions 15 and 14, the dialog open; Space: the question, every count 0 | confirmed |
| E14-16 (design §0) | on the entrance at the window, after it and after Space; none entered from above | the guild (5, 8), M1 (12, 8), P1 (24, 8), P2 (26, 10), the Altars (6, 18), (12, 18), (18, 18), (12, 26), the Hill Forts (24, 18), (24, 26), the factory (29, 18): the hero on the entrance at the window, after closing it and after Space, which opens the same window again (or the factory's question); E14-8 from (6, 17) above A1's entrance walked (7, 18) then (6, 18), 241 points; P2 reached from below (482, 878 left) | confirmed: all seven entered (§2's correction, T127b), none from above |
| E14-17 (design §14) | the level-up over the Altar, then the Altar open; then 10 Gnolls 55 | Basic Learning: the Mystic Orb 1 050 (lacking 1 000); Sacrifice: the level-up (level 2, Attack, Advanced Learning its only offer) drawn **over the Altar's dialog**, the Altar visible around it, its places already empty, its texts the old total until the level-up's OK; then the Altar open, 1 050, level 2, Advanced Learning; 10 Gnolls (the Lizardmen kept): the most 10, total 55; Sacrifice: 1 105 | confirmed: the order and the new Learning read at once (HER-EXP-001); the drawing as decision 371's note asked (below) |

**The finding (E14-14): the game's version is the scenario dialog's, from the players' editions.**
Read at T130 (`docs/exe/structs.toml` `map_kind`, `docs/exe/map.toml`): `new_game_heroes`
0x4BFE70 writes game +0x1F698 from its fourth argument (0x4BFF09; a campaign by its index at
0x4BFF24: below 7 RoE, below 13 AB, else SoD), which `new_game_start` 0x58C570 takes from the
scenario dialog's version (+0x1898 at 0x58C593: 2 or 3 SoD, 1 AB, any other RoE). The dialog
starts at the edition (`scenario_dialog_create` 0x579BD9: `edition_pointer` 0x69923C's value, 3 in
this build) and `scenario_version_from_slots` 0x58E700 sets it to the highest format, 3 down to 0,
that **every colour slot's edition** allows (`edition_formats` 0x699240, a mask per edition, filled
at run time); the selected map's or save's format (game +0x1F86C: above 0x15 SoD, above 0x0E AB,
else RoE) is only tested against that version, and a map it does not allow hides the advanced
panel (0x58E817 to 0x58E871): **the map's format never lowers the version.** So in the Complete
edition every single-player and hot-seat game, of any map format, plays at version 2 (SoD); version
0 comes only from an RoE campaign (index below 7) or a network game with a peer whose edition
allows RoE alone (or a save of such a game, `saved_game_load` 0x4BCDA0). This explains batch E7a's
RoE map (`E08-rewards.md`, E7-7's version note, T57) and E14-14. The other sources: **VCMI
1.7.5** (`HillFort::fillUpgradeInfo`: every one of the creature's `upgrades` at its level's
percentage, no version test; the elementals' upgrades always in its creatures' config) and **the
wiki** (<https://heroes.thelazy.net/index.php/Hill_Fort>: "any non-upgraded creatures", the same
percentages; nothing on RoE or the elementals) have no RoE exception. The reading's branch on
version 0 (`hill_fort_rows` 0x4E7EB0) stands; it is reached by no map of the shipped 160 in this
edition's single and hot-seat play.

**Decision 378** (the user at T130, ADV-Q59; recommended): **the enabled packs decide** (decision
104's rule; E09 decision 14's for the moat): the Hill Fort upgrades a creature with an upgrade in
the ruleset. Under the base pack alone the four elementals have none (the ab pack's `upgrades`
override adds Storm, Magma, Ice and Energy Elementals), so none upgrades, as the original's version
0; under ab or sod they upgrade on every map, as the Complete edition plays an RoE-format map, VCMI
and the wiki. The original's version-0 branch (an RoE campaign, a peer with RoE alone) is the
recorded deviation. **Effect:** design §4's `roe_except` and §7's import mark `Holds::RoE` are
withdrawn (the part's `hill_fort { gold }`, no import change), exit criterion 6 reads the packs'
upgrades; ADR-0016's draft point drops the mark and its alternative is reversed (T131 applies it).
Not chosen: the mark kept (the original's RoE rule on every RoE-format map, against decision 104
and against what the Complete edition plays).

**The disputes:** D-71 the original (E14-1), D-72 the original (E14-9), D-73 the original (E14-10),
D-74 the original (E14-12): every VCMI difference this survey named is the original's in the
batch, as decisions 365 and 367 chose. A-46 (E14-4) and A-47 (E14-9) seen as read. No new dispute,
no quirk candidate.

**Beyond the rows:**
- **E14-17's drawing** (decision 371's note for the clients): the original draws the level-up over
  the Altar's dialog without covering it; the Altar's places are already empty and its total's
  texts stale until the level-up's OK, then redrawn. So a client keeps both drawn, the Altar
  beneath (the map client lane's inbox told).
- **E14-9's scroll** reached the misc position 9 at its pickup (HER-CHG-002's placement: a body
  position first), as the spec planned; the altar takes it from there.
- **E14-15's positions:** the First Aid Tent into 15, the Ammo Cart into 14 (the machines' own
  positions); the dialog stays open after each, as §14 read.
- **E14-8's path** from above an Altar's entrance goes round it (a diagonal step and a straight
  one): a step from above is refused, so the path finder takes another neighbour (ADV-OBJ-004).
- **Not played:** A-48 (a timed turn; the hot seat's turn time not set by the harness), the
  computer's visits (E10), the town's Marketplace (the town lane's).

**The rules toward `verified`:** every row but E14-14 agrees with the reading, and E14-14 with the
rule of decision 378 (the SoD side by E14-12's four elementals, the RoE-format map by E14-14 under
the sod pack), so each rule below, built and tested, takes the batch as its `observation` source
and is `verified` at once.
- T132a: ADV-MKT-002 (E14-1, E14-5), ADV-HFT-001's which stacks (E14-12, E14-13, E14-14).
- T132b: ADV-MKT-001 and ADV-OBJ-003 (E14-16).
- T133a: ADV-MKT-006 (E14-3), ADV-MKT-003, -004, -005 (E14-5, E14-6, E14-1, E14-2), ADV-MKT-007
  (E14-4: the price and the emptied entry; the refusal before paying is decision 366's, no row).
- T134a: ADV-ALT-001 (E14-7, E14-16), ADV-ALT-002 (E14-8, E14-9), ADV-ALT-003 (E14-10, E14-11,
  E14-17), HER-EXP-001's products (E14-10, E14-17), GAM-DEC-007's reopening order (E14-17).
- T134b: ADV-HFT-002 (E14-12, E14-13), ADV-WMF-001 with GAM-DEC-008 and ADV-HIR-002 (E14-15).
- ADV-MKT-008 (what a viewer sees) and GAM-DEC-011 to -013's validation have no row: the reading's
  and the design's (as planned, §18).

**For T131** (the drafts with this outcome): every point the rows test holds as drafted (ADR-0016
points 1, 4, 6: the seven entered, the draw, the purchase, the trades, the sacrifices, the
upgrades, the factory; ADR-0017 points 1, 6, 8 to 10: the decisions, the Altar's return after its
levels), but **ADR-0016's Hill Fort point**: the RoE mark withdrawn, the packs' upgrades deciding
(decision 378), its alternative reversed; ADR-0017's import line loses "the RoE Hill Fort's mark".

**The checkpoint** (ADV-Q59, `AskUserQuestion`, one round of two; the epic's decisions 378, 379):
(1) E14-14: **the enabled packs decide** (decision 378, as recommended); (2) **T130, then T131 in
this session** (decision 379, as recommended).
