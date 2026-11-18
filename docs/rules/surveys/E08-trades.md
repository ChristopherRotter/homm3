# E08 trades and services survey (E08 T127a)

Date: 2026-11-18. Scope: E08 increment 14 (decisions 361, 362), input for T127b's ADR amendment
drafts, the design note and the cut. **In two parts** (the user at T127a's start, decision 363,
after the reading was measured: about 30 KB of code, the market about 15 KB, the Altar of
Sacrifice about 8 KB of a 22 KB dialog, the Hill Fort about 5 KB, the War Machine Factory about
1.5 KB; T118a's whole survey had read about 5 KB): **T127a1** (this part, §0 to §11) the census of
the seven classes and the market: the Trading Posts (99, 221), the Freelancer's Guild (213), the
Black Market (7) and the trade window they share with the town's Marketplace, its rates;
**T127a2** the services (the Altar of Sacrifice 2, the Hill Fort 35, the War Machine Factory 106)
with their VCMI and wiki comparison, the interfaces' sketch, batch E14's plan, the proposed rule
ids and the checkpoint's questions (§12 on). Input, not rules (`README.md`): a fact becomes binding
when a task writes it into the catalog with its sources and tests.

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

## 0. Findings in short (part 1)

- **The census** (§1): 1 058 objects of the seven classes on 148 of the 160 maps; every one has
  subclass 0, no body in the map file and one entrance, blocked (visited from a neighbour, on
  land). The Freelancer's Guild (213) and the snow Trading Post (221) are on AB and SoD maps only.
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
  nothing"); its template has **one visitable tile, blocked**: each is visited from a neighbouring
  tile, as a resource pile. Every entrance stands on land (no water entrance); the 33 class-221
  posts all on snow. The War Machine Factory's two masks differ only in where the entrance is
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
| 2 Altar of Sacrifice (0x4A8541) | its own dialog (T127a2) | nothing | the team's kind mark k 28 (game +0x4E360), a human's visit only |
| 35 Hill Fort (0x4A9172) | its own dialog (T127a2) | its own routine (not read) | the team's kind mark k 14 (game +0x4E352), every visit |
| 106 War Machine Factory (0x4AA622) | ADVEVENT's question, then the hire dialog with the three war machines (T127a2) | its own routine (not read) | none |

- No visit tests a day, a week, an owner or an earlier visit: a hero may trade at the same object
  every time it visits it (a visit from a neighbour, so leaving and stepping back, or the
  engine's `Revisit`, ADV-OBJ's).
- No visit spends anything but the step onto the neighbour: the window's trades take no movement
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
| mercury, sulfur, crystal, gems | 1 : 2 | 3 : 1 | 1 : 2 | 3 : 1 | 3 : 1 | 3 : 1 | 1 : 150 |
| ore | 3 : 1 | 7 : 1 | | 7 : 1 | 7 : 1 | 7 : 1 | 1 : 75 |
| gold | 833 : 1 | 1667 : 1 | 833 : 1 | 1667 : 1 | 1667 : 1 | 1667 : 1 | |

(a : b = give a, get b; a precious resource for another precious one 3 : 1.) A creature sells for
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
  original's (the code reading of this build; a batch E14 row to confirm, T127a2's plan).
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

## 12. Part 2 (T127a2)

To come: the Altar of Sacrifice (its dialog, what it takes for experience by the hero's alignment),
the Hill Fort (its upgrade prices), the War Machine Factory (the hire dialog's war machines); their
VCMI and wiki comparison and disputes from D-72, quirk candidates from A-47; the interfaces' sketch;
batch E14's plan (D-71's and A-46's rows among them); the proposed rule ids; the checkpoint's
questions.
