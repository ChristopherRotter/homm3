# E08 mines, week and owned objects survey (E08 T77a): flagging, mines, Lighthouses, Shipyards, the day and the week

Date: 2026-10-30. Scope: E08 increment 9 (decisions 192, 193), input for T77b's ADR amendment
drafts, the design note and the cut. Cut in two (decision 194, the user's: by what runs the
rule): **T77a1** what a visit does: the census, flagging, a mine's visit and its guards, the
abandoned mine, Lighthouses and Shipyards (§0 to §6); **T77a2** what the day's start does: the
day's, week's and month's order, the mines' income, the Windmill, the Water Wheel, the Mystical
Garden, the monsters' weekly growth, the map's timed events; then the commands, decisions, events
and the observation, batch E9's plan, the rule ids, a fresh agent's second reading and the
checkpoint's `ADV-Q37` (§7 on).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-10-30):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_mine` 0x4A39A0, `set_mine_owner` 0x4C66E0,
  `set_shipyard_owner` 0x4C6A30, `visit_shipyard` 0x49E2E0, `shipyard_boat_tiles` 0x500DE0, the
  loader's two mine bodies (`load_mine_body` 0x501020, `load_abandoned_mine_body` 0x501130),
  `count_owned_records` 0x4BAE70, `boats_in_use` 0x4CCDD0, `random_between` 0x50B230; T77a2:
  `new_day` 0x4C7FE0, `new_day_week_events` 0x4C8780, `new_month` 0x4C8F40,
  `player_daily_income` 0x4B8AF0, `player_gold_income` 0x4C7930, `hero_gold_income` 0x4E4390,
  `count_worn_artifact` 0x4BABD0, `hero_mana_regained` 0x4E3F40, `visit_windmill` 0x4A7FC0,
  `visit_water_wheel` 0x4A7DE0, `visit_garden` 0x4A3BC0, `map_timed_events` 0x4CD910,
  `apply_timed_event` 0x4CD710.
  **Read in part:** `visit_object`'s cases 42, 53 and 87 (0x4A84F0), `load_object_body`'s cases
  53 and 87 (0x502E00), `setup_map_objects`'s cases 42, 53 and 87 (0x4C0CC0),
  `map_template_read` 0x503780 at the class's mapping (`object_class_values` 0x63A6E4),
  `player_defeated` 0x4F11A0 (its shipyards, mines and dwellings), `battle_setup_battlefield`
  0x4643B0 at its mine test, `hero_max_land_movement` 0x4E4990 at its Lighthouse terms,
  `mine_hint_text` 0x40D670, `adventure_mouse_move` at its two Shipyard sites (0x40E636,
  0x40E7D0), `hero_move_loop` 0x407B80 at its case 7, `end_turn_button` 0x409A70 (the adventure
  screen's control messages, its switch 0x409AB4; the name is its End Turn case's) at its control
  7; T77a2: `setup_map_objects`' cases 55, 109 and 112, `next_player_turn` 0x4C6FE0 at its
  call of the new day, `load_timed_event` 0x4FC1A0 at its days, `place_object` 0x4C9890 at its
  class. **Not followed:** the computer player (`ai_fight_answer` 0x52BD10, its own mines, its boats),
  network play (the message records 0x49BF90 and 0x482390, `network_map_message` 0x482010), saved
  games, the scenario's conditions after a flag (`scenario_condition_met` 0x4F2CE0: E10), the boat
  dialog itself (0x5D2720), the guards' description (0x40ABE0); T77a2: the towns', the taverns'
  and the dwellings' parts of the day's start (E09, increment 10).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`); T77a2:
  `server/processors/NewTurnProcessor.cpp` (`onNewTurn`, `generatePlayerIncome`, `pickWeekType`,
  `onPlayerTurnStarted`, `handleTimeEvents`), `server/CGameHandler.cpp`
  (`spawnWanderingMonsters`), `lib/callback/CGameInfoCallback.cpp` (`getFreeTiles`),
  `lib/mapObjects/CGCreature.cpp` (`newTurn`), `lib/mapObjects/CGHeroInstance.cpp`
  (`manaRegain`, `getManaNewTurn`), `lib/mapping/CMap.cpp` (`CMapEvent::occursToday`),
  `lib/rewardable/Limiter.cpp`, `config/gameConfig.json` ("creatures"),
  `config/objects/rewardableOncePerWeek.json`, `config/artifacts.json`, `config/creatures/*.json`
  (`hasDoubleWeek`); T77a1: `lib/mapObjects/MiscObjects.cpp`
  (`CGMine`, `CGShipyard`), `lib/mapObjects/MiscObjects.h` (`InitialGuards`),
  `lib/mapObjects/FlaggableMapObject.cpp`, `lib/mapObjects/IObjectInterface.cpp`
  (`IBoatGenerator::bestLocation`, `shipyardStatus`, `IShipyard::getBoatCost`),
  `lib/mapping/MapFormatH3M.cpp` (`readMine`, `readAbandonedMine`, `setOwnerAndValidate`),
  `server/CGameHandler.cpp` (`setOwner`, `changeFogOfWar`), `lib/gameState/CGameState.cpp`
  (`initFogOfWar`), `lib/gameState/GameStatePackVisitor.cpp`,
  `client/adventureMap/AdventureMapInterface.cpp` (the Shipyard's click and cursor),
  `config/objects/moddables.json` (mines), `config/objects/lighthouse.json`,
  `config/objects/generic.json` (the Shipyard); links
  `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the search tool's extracts, `https://heroes.thelazy.net/index.php/<Page>`:
  Mine, Lighthouse, Shipyard, Map Editor - Objects, Object Events; T77a2: Windmill, Water Wheel,
  Mystical Garden, Resource, Wandering creature, Growth, Plague, Troop stack, Map Editor,
  Cornucopia, Combination artifact. Re-check a quote before a rule
  cites it with `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §1; the text tables
  `advevent.txt`, `GENRLTXT.TXT` and `MINEEVNT.TXT` (`extracted.tar`) for which message is which.

## 0. Findings in short (T77a1, T77a2)

- **Census** (§1): 5 979 mines on 158 maps (60 owned at load, by colours that play), 158
  abandoned mines (64 of class 53, 94 of SoD's 220; every one subclass 7, never wood, 7 gold only),
  83 Lighthouses (all neutral, none on water), 331 Shipyards (one owned at load; two without a
  boat tile, both on Hoard); a monster beside the entrance of 3 553 mines; flag all mines the
  victory on 4 maps.
- **Records** (§2.1): mines and Lighthouses share one table of 0x40-byte records (owner, resource
  or 100 for a Lighthouse, the abandoned flag, a guards' army, the entrance); a Shipyard's owner
  and boat tile live in its tile's word. SoD's class 220 loads as class 53.
- **Flagging** (§2.3, §2.4): a visit flags a neutral or another team's object for the visiting
  player; an own or allied one is never re-flagged. A flag reveals a radius of 3 around the entrance
  to the new owner's team. A defeated player's objects become neutral, a mine keeping its guards
  (§2.5).
- **Mines** (§3): an own mine opens the Leave Guards dialog; an ally's does nothing; another's
  with guards asks first (two texts: neutral, owned), then fights. A neutral mine's single guard
  stack is fought as a wandering monster (split into stacks, `monster_battle`; D-47: VCMI fights
  the mine's army as it is); any other guards as an army without a hero. Crystal, gold and
  abandoned mines fight on the subterranean field.
- **The abandoned mine** (§3.4): its resource drawn at load from the map's set (`random_between`
  of its count), 100 to 200 Troglodytes in slot 0 at the new game (D-48: VCMI 100 to 199), its
  income by the drawn resource (2 wood or ore, 1 000 gold, 1 else). An empty set gives the
  resource 8, past the seven (A-31).
- **Lighthouses** (§4): flagged by a visit with a message, nothing for an own or allied one; each
  Lighthouse a player owns adds 500 (movement.txt) to the sea points of that player's heroes,
  stacking without bound; read when the sea's maximum is computed (HER-ADV-009). Every Castle
  town with its Lighthouse adds the same to every hero, whoever owns the town (A-30, E09).
- **Shipyards** (§5): the boat tile is fixed once at load: the first of twelve offsets around the
  entrance that is open water without another object's entrance (a boat allowed); a human's visit
  flags, then offers a boat for 1 000 gold and 10 wood, refused without a tile ("the Ways are
  blocked") or with a boat or hero on it; the boat is type 1, owned by the builder. An own or
  allied Shipyard can be used from a click on it without a visit (VCMI, the wiki; the original's
  cursor read, its click in part).
- **The day's start** (§7): no new day on day 1; from day 2 one new day for the whole game after
  the round's last turn: the new week on days 8, 15, … (weeks of 7 days), the new month on days
  29, 57, … (months of 4 weeks); then the income, then every hero's spell points; each player's
  timed events at its own turn start.
- **The income** (§8): mines (2 wood or ore, 1 of the rare ones, 1 000 gold), paid to every
  player not defeated at the new day; beyond mines the resource and gold artifacts (the
  Cornucopia 5 of four), Estates, resource and gold specialists, the Crystal Dragons' 3 crystal a
  week; towns E09, the computer's bonus and the handicap E10.
- **The weekly objects** (§9): the Windmill 3 to 6 of mercury to gems, the Water Wheel 500 gold
  the first week then 1 000, the Mystical Garden 5 gems or 500 gold; full at the new game, reset
  every week visited or not; the visited marks the team's (the Garden's the player's), kept.
- **Monsters** (§10): a tenth a week in sixteenths, capped at 4 000 (D-49 against VCMI's
  thousandths, A-32 a larger stack shrinks); a creature month (40 %; 10 % the Plague) puts its
  creature on one free land tile in 200, twice its range (D-50 against VCMI's exact count).
- **Timed events** (§11): resources (floored at 0) and a message, at the player's turn start, on
  the file's day + 1 and every interval after; VCMI the same.
- **Spell points** (§12): Mysticism's 1 to 4 a day plus the mana artifacts, to the cap; the
  Wizard's Well fills it; every hero record.

## 1. The census

The 160 shipped maps (`maps.tar`), read with `h3_map::read`; 8 import whole today. Objects / maps
(on the 8 whole maps):

| Class | Objects | Maps | Owned at load | Underground | Whole maps | Most on a map |
|---|---|---|---|---|---|---|
| 53.0 sawmill | 1 136 | 157 | 11 (8 maps) | 151 | 63 (8) | 26 |
| 53.1 alchemist's lab | 753 | 156 | 9 (6) | 99 | 39 (8) | 14 |
| 53.2 ore pit | 1 069 | 158 | 9 (7) | 146 | 57 (8) | 20 |
| 53.3 sulfur dune | 749 | 156 | 8 (6) | 102 | 34 (8) | 13 |
| 53.4 crystal cavern | 741 | 155 | 9 (6) | 99 | 40 (8) | 13 |
| 53.5 gem pond | 714 | 156 | 8 (6) | 103 | 38 (8) | 12 |
| 53.6 gold mine | 753 | 140 | 6 (3) | 112 | 40 (7) | 18 |
| 53.7 abandoned mine | 64 | 41 | — | 0 | 2 (1) | 4 |
| 220 abandoned mine (SoD) | 94 | 32 | — | 17 | 0 | 10 |
| 42 Lighthouse | 83 | 54 | 0 | 1 | 22 (3) | 16 (Pandora's Box) |
| 87 Shipyard | 331 | 82 | 1 (1) | 25 | 11 (4) | 11 |

- **Owners at load.** 60 mines and 1 Shipyard are owned at load, every one by a colour that plays
  (human or computer); no Lighthouse is. No object of these classes is owned by a colour nobody
  plays, and no owner field holds a value other than a colour or `0xFF`.
- **Abandoned mines.** Every class-220 object has subclass 7, as every 53.7. Their resource sets
  (the body's first byte, bit 0 wood to bit 6 gold; the other three bytes are always 0): all but
  wood on 143, gold only on 7, all but wood and gold on 3, all but wood, ore and gold on 3, all but
  wood and ore on 2; **wood is never allowed, no set is empty**, no bit beyond the seven resources
  is set.
- **Entrances.** No mine, abandoned mine or Lighthouse has its entrance on water or rock; every
  object of these classes has one entrance tile, shared with no other object's; no placed hero
  stands on one. The Lighthouse's template (`AVXlths0`) is one tile, blocked and its entrance (the
  class is entered, not visited from beside: `object_classes_not_entered` 0x63AA58 lacks 42); the
  Shipyard's (`AVXshyd0`) is three tiles in a row, blocked in its mask, the middle one its entrance.
- **Guards beside.** A monster (54, 71 to 75, 162 to 164) stands on one of the eight tiles around
  the entrance of 3 553 mines (331 ore pits to 587 sulfur dunes), 13 abandoned mines, 17
  Lighthouses and 157 Shipyards: the guard zone (ADV-GRD-001), not the object's own
  guards.
- **The Shipyards' boat tiles** by the original's rule (§5.1): offset 1 (two tiles left of the
  entrance) for 149, offset 2 (two right) for 143, offsets 3 to 9 for the other 37; **none for
  two** (Hoard and Hoard (Allies), the Shipyard at (62, 38, 0)); no earlier offset was passed over
  for another object's entrance; three hold a boat at load; no two Shipyards share a tile. 111
  Shipyards have no water on the eight tiles around their entrance, so the twelve offsets' reach
  of two columns matters.
- **Victory.** Flag all mines (goal 9) on 4 maps (two with the standard victory too, three for
  computers too); flag all dwellings on 5 (increment 10).
- **For T77a2** (counted here once): 666 timed events on 139 maps (114 with resources, 71 taking
  some; 61 repeating; 288 not for every player; 62 for computers too; 199 with a first day of 0);
  13 863 wandering monsters on 159 maps, 506 of them not growing (73 maps). T77a2 added: the
  intervals (5, 6, 7 ×24, 14 ×14, 21 ×3, 28 ×18; none of 1), the resources from −18 000 to
  +50 000 gold, 604 events for humans only and 1 for computers only; no monster placed above
  4 000 (13 123 with a drawn count); the weekly objects (decision 192's 780 Windmills, 430 Water
  Wheels, 286 Gardens) at most 18, 24 and 11 on a map.

## 2. Owned objects and flagging

### 2.1 The records

- **Mines and Lighthouses** share the game's table of 0x40-byte records (game +0x4E388, its
  vector's start +0x4E38C; H3API's mines). A record: +0x0 the owner (i8, -1 none); +0x1 the
  resource (a mine's subclass 0 to 6, the abandoned mine's drawn one, 100 for a Lighthouse);
  +0x2 the abandoned flag; +0x4 the guards, an army of seven slots (creature types +0x4 to +0x1F,
  counts +0x20 to +0x3B); +0x3C to +0x3E the entrance's column, row and level. The loader appends
  one record per object (`load_mine_body` 0x501020 for 53 below subclass 7 and 42,
  `load_abandoned_mine_body` 0x501130 for 53.7) and the object's word is its index. The table is
  what "mines" means to the game: the flag-all-mines victory and the wiki count Lighthouses and
  abandoned mines among them.
- **The owner from the file** (`load_mine_body`): its first byte (`0xFF` neutral; three more bytes
  read and dropped); the guards empty. The abandoned mine's body has no owner: neutral.
- **SoD's abandoned mine** (class 220) loads as class 53 (`map_template_read` maps every
  template's class through `object_class_values` 0x63A6E4: 220 as 53), so with its subclass 7 it
  is a 53.7 in every way; only its pictures differ (VCMI: its own class 220 on the same handler,
  `isAbandoned` by subclass 7).
- **A Shipyard** has no record: its tile's word holds the owner in its low byte (signed, `0xFF`
  none) and the boat tile's column and row in its next two bytes (`0xFF`, `0xFF` none). The loader
  (`load_object_body` case 87, 0x503047) stores the file's owner byte and marks the boat tile
  none; `shipyard_boat_tiles` 0x500DE0 sets it at load (§5.1). Each player record keeps a list
  of its owned Shipyards' positions (player +0x8C, a vector; `set_shipyard_owner`).

### 2.2 The owner at load

At a new game (`setup_map_objects` 0x4C0CC0, its walk over the entrance tiles by level, row and
column):

- **Case 53** (0x4C1878): the record whose entrance is the tile (a search of the table); an
  abandoned one gets its guards (§3.4); then `set_mine_owner(index, the record's owner, quiet)`:
  an owned mine reveals its radius at once (§2.3).
- **Case 42** (0x4C15D8): the same search; the tile's word becomes the index; no owner call (no
  shipped Lighthouse is owned at load; one owned in a crafted map keeps its owner without the
  reveal).
- **Case 87** (0x4C1C4D): an owner other than `0xFF` is cleared and set again through
  `set_shipyard_owner`, which reveals and lists it.

An owner colour the scenario does not play is kept as it is (none on the shipped maps; whether
the original's game start defeats an absent colour, which would neutralise its objects through
§2.5, was not read). VCMI's loader makes an object owned by a colour nobody can play neutral
(`setOwnerAndValidate`), and reveals each owned object's radius 3 at the start (`initFogOfWar`).

### 2.3 The flag

- **`set_mine_owner`** 0x4C66E0 (index, owner, how; mines and Lighthouses): with `how` 1 (a
  visit, a defeat) a message record for the other players (0x49BF90: the record's old owner;
  network); the record's owner set; for an owner other than -1 `reveal_disk` (the entrance, the
  owner, radius 3), which reveals for the owner's whole team (ADV-FOG-002's disk); with `how`
  not 0 the scenario's test (flag all mines, E10). `how` 0 at the new game, 3 from the network.
- **`set_shipyard_owner`** 0x4C6A30 (packed position, owner): a hero standing on the tile is
  lifted off it to reach the word and put back after (`hero_hide`, `hero_show`); nothing when the
  owner does not change; the position leaves the old owner's list and joins the new one's; the new
  owner's team gets the radius-3 reveal; the word's low byte the owner; a message record
  (0x482390).
- **VCMI:** `setOwner` changes the owner and the players' owned-object lists and runs the
  victory and loss checks; it reveals nothing at a flag. The visitor stands on the entrance and
  its own scouting radius (at least 5) already covers the radius 3, for its team in both (VCMI's
  fog is the team's), so the two agree in effect; they differ only for an owner set without a
  visitor (a crafted map's owner at load, which VCMI reveals at the start too).

### 2.4 Who flags what

Every test compares teams (the team bytes game +0x1F879) of the object's owner and the
**current player** (0x69CCA8; the visiting hero's owner whenever a human plays):

| Class | Neutral or another team's | An ally's | Own |
|---|---|---|---|
| 53 mine | guards first (§3.1), then flagged | nothing at all | the Leave Guards dialog (a human) |
| 42 Lighthouse | flagged, a message to a human (ADVEVENT 69) | nothing | nothing |
| 87 Shipyard | flagged silently, then the boat offer (a human) | the boat offer | the boat offer |

Nothing is asked before a flag except a guarded mine's battle; nothing is paid. The visit ends
the hero's walk on the entrance (an entered class), and the hero stays on it.

### 2.5 A player's defeat

`player_defeated` 0x4F11A0 (from `scenario_condition_met`; read for these classes): the
player's flag of defeat (game +0x1F636), then every Shipyard it owns made neutral
(`set_shipyard_owner` with -1), every mine and Lighthouse record (`set_mine_owner` with -1,
`how` 1) and every dwelling (`set_dwelling_owner` 0x4C67B0, increment 10). A mine keeps its record's guards: a
defeated player's Leave Guards garrison stays as a neutral mine's guards, in the slots the player
chose (A-29). VCMI does the same through its player's loss (not compared in detail; E10's).

### 2.6 A hero standing on an entrance

A hero standing on an owned object's entrance hides the object's tile (the tile's class 34; its
own class kept under the hero, `hero_hide`). A step onto it meets or fights the hero (GAM-MET-001,
GAM-BAT-004); the object is not visited and its flag does not change; the object's flag keeps its
owner's colour under the hero (`tile_object_owner` 0x40FD20 reads the object under a hero). The Shipyard's owner change lifts the hero to reach its word (§2.3). No shipped
map places a hero on one of these entrances (§1). Revisiting an own mine from the entrance (the
Leave Guards dialog again) needs a visit command, which the engine does not have: a T77a2 point.

### 2.7 What a viewer sees

- **The flag:** the object's owner colour, for every viewer who sees the object (the view's
  passes draw the flagged classes in their owner's colour, `def_draw_flagged` 0x47BE10; the
  minimap colours the blocked and entrance tiles of an owned object by its owner on explored
  tiles, `minimap_draw` 0x412C40). Increment 9 adds an owner to ADR-0016 point 6's appearance.
- **A mine's hint** (`mine_hint_text` 0x40D670; its callers are the status bar's hint and a
  popup from the click path, `object_popup_text` 0x4137C0): the mine's name (an abandoned one's "Abandoned Mine", whatever it
  produces), its owner's colour name when owned, **its daily production only for the viewer's
  team**, and its guards for every viewer when it has any (0x40ABE0, not read; VCMI: the army's
  words, "Guarded by a few …"). So an owned abandoned mine's resource is its owner's team's
  knowledge; VCMI's hover shows the resource to every viewer once the mine is owned.

### 2.8 VCMI and the wiki

- **VCMI** (`CGMine::onHeroVisit`, `FlaggableMapObject::onHeroVisit`, `CGShipyard::onHeroVisit`):
  the same table of §2.4 by player relations (same player, ally, enemy; neutral counts as an
  enemy).
- **Wiki:** mines "can be flagged after which they will start to produce" for their owner; a
  flagged Lighthouse's message names "all of your ships"; a Shipyard "can be used by the owner or
  an ally". Lighthouses and abandoned mines count as mines for flag all mines.

## 3. Mines

### 3.1 The visit

`visit_object` case 53 → `visit_mine` 0x4A39A0 (the tile, the hero, the position, human):

1. **An own or allied mine** (the owner's team the current player's): own and a human: the Leave
   Guards dialog (§3.3); anything else: nothing, no message.
2. **Guards** (the record's army has a creature): a human is asked (a two-button message box):
   ADVEVENT 84 for a neutral mine ("You come upon an abandoned mine. The mine appears to be
   infested with Troglodytes. Do you wish to enter?"), ADVEVENT 187 for an owned one
   ("Unfortunately, it appears that the owner has left some guards. Do you wish to fight
   them?"); no means nothing happens; the computer answers through `ai_fight_answer`. The text
   follows the owner, not the kind: a neutral mine with a defeated player's guards gets the
   Troglodytes' question.
3. **The battle** (§3.2). Lost (or retreated, surrendered): the visit ends, the mine unflagged.
4. **The flag:** a human sees the mine's message (MINEEVNT by the record's resource: "You gain
   control of a Sawmill. It will provide you with two units of wood per day.") with the resource's
   picture and its daily amount (`mine_daily_income` 0x678288); then `set_mine_owner(index,
   current player, 1)` (§2.3). An abandoned mine shows its drawn resource's mine's message (a
   Gem Pond's for gems), as VCMI.

### 3.2 The guards' battle

- **A neutral mine with one stack of guards:** `monster_battle` with slot 0's creature and count
  (record +0x4, +0x20): the wandering monster's battle (ADV-MON-009: its split by the army ratio
  table and a roll of 1 to 100, its seed from the tile), so 150 Troglodytes fight in several
  stacks. When creatures remain (a loss, a retreat, a surrender) slot 0's count
  becomes the count left and the visit ends; a win clears the guards, tells a human ADVEVENT 85
  ("You beat the Troglodytes and are able to restore the mine to production."), and the mine is
  flagged. Slot 0 is read whatever slot holds the stack (A-29).
- **Any other guards** (an owned mine's, or a neutral mine's of two stacks or more):
  `adventure_battle` with the guards' army as it is, their owner as the defending side's owner, no
  hero; a loss ends the visit, a win flags the mine (no message before the flag's). The guards'
  losses stay in the record (its army is the battle's).
- **The battlefield** (`battle_setup_battlefield` 0x46447B): a battle on a mine's entrance whose
  resource is crystal (4) or gold (6), or an abandoned mine, uses the subterranean field (terrain
  6, its native bonuses), whatever lies under it; the other mines the tile's terrain. Wiki: "any
  combat taking place in it will use subterranean terrain for graphics and when determining
  native terrain bonuses"; VCMI: `"battleground": "subterranean"` on the crystal, gold and
  abandoned mines. Three sources agree.
- **VCMI:** a yes starts `startBattle(hero, mine)`: the mine's army fights as its stacks are,
  never split (D-47); a win shows ADVEVENT 85 for an abandoned mine only, then flags.

### 3.3 Leave Guards

The own mine's visit runs `army_dialog_with` 0x5D12B0 with the record's army and its flag 1
(GENRLTXT 710 "Leave Guards" as the title): the army dialog of increment 6 between the hero's
army and the mine's (survey `E08-army.md` §1.1: merge, swap, split; the hero's last stack may not
move into the other army). The record's army is kept: what the player leaves are the mine's
guards until a battle or the player takes them back. Only a human visiting its own mine sees it;
VCMI shows the same exchange (`showGarrisonDialog` with removable units) to the owner's hero,
computer or human. Neither the original nor VCMI limits the guards beyond the army's seven slots.

### 3.4 The abandoned mine

- **Its resource** (`load_abandoned_mine_body`, at the map's load, in the file's object order):
  the body's first byte as a set of eight bits (the other three bytes dropped); n the count of set
  bits; r = `random_between(1, n)` (inclusive: a + rand() mod (b − a + 1); one draw only when n is
  2 or more); the resource is the r-th set bit (0 wood to 6 gold). With no bit set r is 1, no bit
  is found and the resource is 8, past the seven (A-31); bit 7 would give resource 7 (mithril,
  not in the game). The record: owner -1, the abandoned flag set, no guards yet. VCMI:
  `RandomGeneratorUtil::nextItem` over the map's set; an empty set logs an error and gives gold.
  The wiki: the editor requires "at least one resource".
- **Its guards** (`setup_map_objects` case 53, 0x4C18E6, at the new game, in the entrance walk's
  order): slot 0 creature 70 (Troglodytes), count `random_between(100, 200)`, so 100 to 200 (D-48:
  VCMI's `InitialGuards` 100 to 199; the wiki's "throng (100–249)" is the army word's band, not
  the range).
- **Its income** follows its drawn resource from the mines' table (2 wood or ore, 1 000 gold, 1
  for the others; VCMI's `defaultResProduction` for an abandoned mine gives the same).
- **Its battlefield** is subterranean (§3.2) and its message after a win is ADVEVENT 85.

### 3.5 The income table

`mine_daily_income` 0x678288 (eight i32; the eighth is not a resource's): wood 2, mercury 1,
ore 2, sulfur 1, crystal 1, gems 1, gold 1 000 (VCMI's `defaultQuantity` per mine and its
abandoned mine's table agree; the wiki's table the same). When and in what order the day's start
pays it is §8's: at the new day, from day 2, resources 0 to 5 by `mine_income` 0x677978, gold
by a constant.

### 3.6 VCMI and the wiki

Beyond D-47 and D-48: VCMI asks by the mine's kind (84 for an abandoned mine, 187 for any other
with guards; "TODO: alternative text for custom guards"), the original by its owner; VCMI shows
the own mine's exchange to a computer's hero too. The wiki's Mine page: eight kinds; production
at the beginning of each day; mines "can have creatures stationed in them in a similar way to
garrisons"; the owner's guards' question as the original's text.

## 4. Lighthouses

- **The visit** (`visit_object` case 42, 0x4A9479): the record's owner's team the current
  player's: nothing; otherwise a human sees ADVEVENT 69 ("This lighthouse is now under your
  control, and all of your ships will now move further each turn.") and `set_mine_owner(word,
  current player, 1)` flags it (§2.3). No guards, no question. VCMI (`FlaggableMapObject`): the
  same, its comment "H3 behavior - revisiting owned Lighthouse is a no-op".
- **The bonus** (`hero_max_land_movement` 0x4E4A18 with the sea's argument): the count of records
  owned by the hero's owner (+0x22) with the resource 100 (`count_owned_records` 0x4BAE70), times
  `movement_bonus_lighthouse` 0x698B0C (500 from movement.txt). So each Lighthouse its owner holds
  adds 500 to every one of that player's heroes' sea points, without bound and without
  Navigation's scaling (the wiki: "cumulative … Navigation secondary skill does not affect this
  bonus"); an ally's do not count (VCMI's bonus has the `PLAYER` propagator: the owner's heroes).
  It is read only where the day's sea maximum is computed (HER-ADV-009: the turn start in a boat,
  a boarding, a landing): a Lighthouse flagged today changes a hero already at sea from the next
  of those. Losing it (a capture, a defeat) lowers the next maximum.
- **The Castle's Lighthouse building** adds the same 500 in the same function for **every Castle
  town with it built, whoever owns the town** (0x4E4AB9 to 0x4E4AF9: no owner test), to every
  hero (A-30; E09's towns; the wiki's building text says "all ones heroes'", VCMI's building the
  owner's).
- **On water:** the wiki says a map Lighthouse may stand on water; no shipped one does (§1), and
  how a hero reaches an entered object on a water tile was not read (open point, §6).

## 5. Shipyards

### 5.1 The boat tile, fixed at load

`shipyard_boat_tiles` 0x500DE0 (from `map_objects_place_all`, after the objects are placed) walks
every entrance tile of class 87 and tries twelve offsets from it in order (the table
`shipyard_boat_offsets` 0x67FAA8, (column, row)): (−2, 0), (2, 0), (−2, 1), (2, 1), (−1, 1),
(1, 1), (0, 1), (−2, −1), (2, −1), (−1, −1), (1, −1), (0, −1), on the entrance's level. The
first tile on the map that is water, not blocked (map_item +0xD 0x01 clear) and either no
object's entrance or a boat's (class 8) is the boat tile: its column and row are written into the
word of each class-87 tile of the entrance's row from one left of it to one right of it (the
Shipyard's three tiles). None found: the word keeps `0xFFFF` and the Shipyard
can never build. Computed once: an object that later leaves an earlier offset does not move it,
and a later object on it does not either. VCMI's `CGShipyard::getOutOffsets` holds the same twelve
in the same order (its comment draws them), and `IBoatGenerator::bestLocation` the same test, but
evaluated at every visit (§6). The wiki: "Adventure map shipyard has 12 active tiles … if none of
them are on clear water, a boat cannot be built".

### 5.2 The visit

`visit_object` case 87 → `visit_shipyard` 0x49E2E0 (the tile, the position, human):

1. **The flag:** the owner's team not the current player's (or no owner): `set_shipyard_owner`
   with the current player (§2.3), silently.
2. **A human only** (the computer builds through its own code, E10):
   - no boat tile: ADVEVENT 189 when the player has no selected hero, else GENRLTXT 135 with the
     selected hero's name ("%s encounters an abandoned shipyard. Unfortunately the Ways are
     blocked, and it is not possible to build ships here."); nothing more;
   - the boat tile is an entrance with a boat (8) or a hero (34) on it: GENRLTXT 52 ("Cannot build
     another boat"); with another class's entrance on it: the same message when 64 boats are in
     use (`boats_in_use` 0x4CCDD0), else on to the price (a boat would be built over that object;
     unreachable on the shipped maps, §6);
   - fewer than 1 000 gold or 10 wood (player +0xB4, +0x9C): GENRLTXT 330 ("You can't afford to
     purchase a boat.");
   - else the boat dialog (0x5D2720, not read); its build (result 0x7802) calls `create_boat` 0x4BB250
     (the tile, the current player as owner, not quiet, type 1); when it returns -1 (64 boats in
     use) nothing is paid and nothing said; else 1 000 gold and 10 wood are taken (no check of the
     payment beyond the test above).
3. The new boat is an empty boat of ADV-SEA-001 on the boat tile: type 1, facing 2, its owner the
   builder, no captain. The engine has no 64-boat bound (decision 164, ADV-QUIRK-32), so only the
   tile's refusals remain.

VCMI: `CGShipyard::onHeroVisit` flags an enemy's or neutral Shipyard, then shows the problem
(GENERAL_TXT 51 / 134 / ADVOB 189, the same three texts) or the Shipyard window; the cost
`IShipyard::getBoatCost` 10 wood and 1 000 gold; the boat `boatCastle` (type 1). The wiki: "build
boats for 1000 and 10".

### 5.3 Using a Shipyard without a visit

- **The cursor** (`adventure_mouse_move`, two sites 0x40E636 and 0x40E7D0): over any tile of a
  Shipyard whose owner's team is the current player's (the owner read at the object's entrance,
  `tile_object_entrance`, `tile_object_setup`), cursor 6 and the click mode 8.
- **The use** (`hero_move_loop`'s table entry 7, 0x4085A8): the tile under the mouse given to
  `visit_shipyard` as if visited (its team test, then the boat offer). *Corrected by T77a2's
  second reading (§17):* the loop's jump index is the click mode minus 1 (0x407BC6), so entry 7
  runs for **mode 8**, the cursor's own; `end_turn_button`'s control 7 (0x409C80) sets mode 7,
  entry 6, a hero's walk, not the Shipyard. A left click (0x4097E0, its controls 0 and 37, then
  0x40A5D0: the clicked tile stored, visible to the viewer) reaches it with or without a selected
  hero (with one, over the Shipyard's blocked side tiles only, which the hero cannot reach), and
  the **clicked tile's** word is the one `visit_shipyard` reads (its owner byte and the boat
  tile, §5.1). Not read: what a side tile's owner byte holds after a flag in the game
  (`set_shipyard_owner` writes the entrance's word only; §14).
- **VCMI** (`AdventureMapInterface`): with a hero selected, a click on a Shipyard's tile the hero
  cannot reach, owned by the player or an ally, opens the Shipyard window or its problem, with the
  "sail" cursor over it. **Wiki:** an own or allied map Shipyard is used "by clicking on the left
  tile, without visiting". So the use without a visit is the three sources' (the original's in
  part); the boat goes to the fixed tile and the builder pays.

### 5.4 VCMI and the wiki

Beyond §5.1 to §5.3: the three sources agree on the price, the twelve tiles, the refusals and the
boat's type; they differ only in when the tile is found (D-49 is not raised: §6).

## 6. Disputes, quirk candidates and open points (T77a1)

The checkpoint's questions are T77a2's (§7 on); these are its inputs.

- **D-47. A neutral mine's single guard stack fought as a wandering monster.** The original fights
  it through `monster_battle` (§3.2): split into stacks by the army ratio table and a roll, seeded
  by the entrance tile, its count left kept; VCMI fights the mine's army as one stack. The wiki
  is silent on the split. Every abandoned mine (158 on the shipped maps) is such a mine. A conflict of a
  reading with VCMI with real gameplay impact (one stack of 150 Troglodytes against several).
  **Proposed:** the reading, with a batch E9 row (an abandoned mine's battle: its stacks) to
  confirm it.
- **D-48. The abandoned mine's Troglodytes: 100 to 200 or 100 to 199.** `random_between(100, 200)`
  is inclusive (read whole), VCMI draws 100 to 199; the wiki gives only the army word's band.
  Impact: one count in 101. **Proposed:** the reading; no batch row (an observation could not
  tell the two apart in a batch's size).
- **A-29. A neutral mine's single guard stack outside slot 0.** Only a defeated player's Leave
  Guards garrison (§2.5) can leave one stack in another slot; the visit then fights slot 0's
  creature and count (none, 0). *T77a2's second reading (§17):* `monster_battle` does not return
  early: an empty army (all slots −1/0) is fought through `adventure_battle` 0x4AD470 (how the
  battle settles an empty side not traced), then the count is rewritten as 0. **Proposed:** fixed (the battle takes the
  stack where it is), classified at the checkpoint; it matters only with player defeat (E10).
- **A-30. The Castle's Lighthouse building helps every hero** (§4): no owner test. For E09 (the
  town's buildings); recorded here because it shares the Lighthouse's bonus. **Proposed:** E09
  classifies it (the wiki and VCMI give the owner's heroes only).
- **A-31. An abandoned mine without a resource.** An empty set gives resource 8 (§3.4), whose
  income the day's start would add beyond the player's seven resources: state-corrupting, so
  always fixed. The editor refuses an empty set (the wiki); no shipped map has one or a bit past
  gold. **Proposed:** the import refuses such a mine (as an invalid map), no flag. *Corrected by
  T77a2 (§14): resource 8 adds no income; the message alone reads past its tables.*
- **The Shipyard's tile fixed at load** (§5.1) against VCMI's search at every visit: on the
  shipped maps the two always agree (no earlier offset holds another object's entrance at load,
  and after load nothing but boats and heroes appears on or leaves water, which both treat alike).
  **Proposed:** the original's (computed once at the setup), no dispute. Likewise the build over
  another object's entrance on the boat tile cannot happen on them.
- **The text of a guarded mine's question and its win message** follow the owner (§3.1, §3.2):
  the engine tells events, not texts; noted for the clients.
- **Information** (ADR-0016 point 6, for T77b): every viewer sees an owned object's owner; the
  owner's team alone sees an owned mine's production, so an abandoned mine's drawn resource stays
  hidden from the other teams (VCMI shows it); every viewer sees a mine's guards in the
  description words (0x40ABE0 not read; VCMI: "Guarded by" with the army's words).
  **Proposed:** as the original.
- **An owner colour nobody plays** (§2.2): none on the shipped maps; the original keeps the owner,
  VCMI makes the object neutral. **Proposed:** the import refuses it as it refuses a placed hero of
  such a colour (an error), the simplest bound.
- **Not read** (for T77a2's second reading where it decides a rule): the click from mode 8 to the
  Shipyard's use and which tile's word it reads (§5.3); `monster_battle` with an empty slot 0
  (A-29); how a hero reaches an entered object on water (a Lighthouse on water, §4); the boat
  dialog 0x5D2720; the guards' words 0x40ABE0; the computer player's guards, flags and boats
  (E10); the scenario's tests after a flag (E10).

## 7. The day's start (T77a2)

### 7.1 The date

The date is three words of the game: the day of the week +0x1F63E (1 to 7), the week of the month
+0x1F640 (1 to 4) and the month +0x1F642 (from 1). The day's number, where a rule needs it, is
((month − 1) × 4 + week − 1) × 7 + day (`map_timed_events` 0x4CD91B): **weeks of seven days,
months of four weeks** (28 days; GAM-CAL-001's "first rule that reads them").

`new_day` 0x4C7FE0 is called only by `next_player_turn` 0x4C6FE0 (0x4C71CC), when its pass over
the colours ends after the computers' pass (humans play first, then computers; the order is E10's,
GAM-TRN-001 plays the colour order). **The game's start does not run it:** day 1 gives no income,
grows nothing and resets nothing; the weekly objects' setup values (§9) are the first week's, as
the wiki's Water Wheel says (500 gold in the first week only). VCMI's `generateNewTurnPack` skips
the income on its first turn the same way.

### 7.2 The new day (`new_day` 0x4C7FE0, read whole)

In this order, once a day for the whole game (not per player):

1. **The date** (0x4C7FEB to 0x4C8031): the day of the week + 1; past 7 it becomes 1 and **the
   new week** runs (`new_day_week_events` 0x4C8780, §7.3); then, when the week of the month is
   past 4, it becomes 1 and **the new month** runs (`new_month` 0x4C8F40, §7.4). The week's
   counter is raised inside the new week (0x4C8A1D), so a month starts on the first day of every
   fifth week: days 29, 57, 85 and so on. A network client (global 0x6972B8 set) skips both.
2. Each town's byte +2 is cleared (difficulty 2 or more, or a human in its owner's team, 0x42B9E0)
   or counts down (E09; its meaning not read).
3. Every hero record (156) loses the day's spells and the flags of the day (T34b's reading, ADV-MOV).
4. Necropolis's Cover of Darkness hides the land around each town with it (0x4C7BA0), and when it
   hid something the owned objects' sight is revealed again (0x4C7C50; E09, T34b's reading).
5. On the week's first day, each Rampart with the Mystic Pond draws its resource (one of four by
   0x63E668) and 1 to 4 of it (E09).
6. **The day's income is computed** (`player_daily_income` 0x4B8AF0, §8.1) and **added** to every
   player not defeated (game +0x1F636): its seven resources += its income +0x108, unsaturated
   (0x4C81E2 to 0x4C821F).
7. The scenario's tests (E10; 0x4C8227).
8. **Every hero record's spell points** are restored (§12; 0x4C8237 to 0x4C82DE).
9. Each town with a Mage Guild fills its visiting and garrison heroes' spell points (E09).

### 7.3 The new week (`new_day_week_events` 0x4C8780, read whole)

1. **The week's kind** (0x4C87A6 to 0x4C896A): a name drawn (0 to 14; the wiki's fifteen animals,
   for the message), then, unless the week just ending is the month's fourth, a draw of 1 to 4:
   1 (25 %) makes a week of a creature, drawn among the creatures with a town and a level (in an
   RoE game, game +0x1F698 0, ids below 118 without the four elementals 112 to 115; else below
   145); an Inferno with the Deity of Fire (building bits +0xD0) makes it the week of the Imp
   (kind 2, creatures 42 and 43), whatever the draw. The kind and creature are globals (0x697750,
   0x6983FC) read by the towns' growth (E09).
2. Each town's growth (0x5BFDD0 by the kind) and its byte +0x33 set (E09).
3. **The week of the month + 1** (0x4C8A1D).
4. The taverns: each player's two tavern heroes without the flag +0x105 bit 17 leave the pool,
   then each player not defeated gets new ones (0x4C8450, E09).
5. **The dwellings' records** (game +0x4E39C, 0x5C bytes each) refill their creatures (0x4B8A60;
   increment 10).
6. **The map's scan** (0x4C8AE0 to 0x4C8D19): every tile of both levels whose +0xD bit 4 is set
   (an object's entrance tile), in level, row and column order. A hero standing there (class 34)
   is lifted off (`hero_hide` 0x4D76E0), so the object under it is read, and put back after
   (`hero_show` 0x4D75D0). By the entrance's class (the table 0x4C8EE4; classes 30 to 112):

   | Class | What the word gets | Whose |
   |---|---|---|
   | 30 Fountain of Fortune | bits 13 to 16 a new luck: 0 to 3 drawn, 0 written as 15 (−1), so −1, +1, +2 or +3 | a later increment (not built) |
   | 48 Magic Spring | bit 6 set (full again) | a later increment |
   | **54 monster** | **the weekly growth** (§10.1) | this increment |
   | **55 Mystical Garden** | bits 6 to 9 the resource 5 (gems) or 6 (gold), one in two; bit 10 set (full) | this increment (§9.3) |
   | 78 Refugee Camp | a creature of any level (0x4C92C0, levels 0 to 6) into the tile's subclass word +0x22, its weekly growth (creature +0x44) as the count | increment 10 |
   | **109 Water Wheel** | bits 0 to 4 become 2 (bit 1 set, bits 0, 2 to 4 cleared) | this increment (§9.2) |
   | **112 Windmill** | bits 0 to 3 a resource 1 to 5 (mercury to gems), bits 13 to 16 an amount 3 to 6 | this increment (§9.1) |

   No owner, visitor or visited mark is read: every such object is reset, visited or not, whoever
   stands on it.
7. Every hero record's flag +0x105 bit 1, the Stables' +400 (0x4E4990), is cleared (0x4C8D1F;
   the Stables are a later increment's).
8. The rumour (0x4C8D3E; the tavern's, E09).
9. The neutral towns' garrisons grow (0x4BF570, 80 % or 40 % a week by their buildings; E09), and
   each Dungeon with the Portal of Summoning draws its dwelling (0x5BD750; E09).

### 7.4 The new month (`new_month` 0x4C8F40, read whole)

1. **The month + 1** (0x4C8F55), then **the month's kind** from a draw of 1 to 10 (0x4C8F5E):
   1 to 5 (50 %) a normal month (its name drawn, 0 to 9); 6 to 9 (40 %) a **month of a creature**
   drawn from the twelve of `month_creatures` 0x63E678 (Serpent Fly, Griffin, Centaur, Gremlin,
   Hobgoblin, Gog, Troglodyte, Harpy, Lizardman, Pegasus, Wolf Rider, Wight: ids 104, 4, 14, 28,
   85, 44, 70, 72, 100, 20, 86, 60); 10 (10 %) **the Plague**. In the tutorial (game +0x1F69D)
   every month is normal; after the week of the Imp (§7.3) the month is the Imp's (kind 1,
   creature 42), whatever the draw. Kind and creature are globals (0x698834, 0x697748).
2. Each town's dwellings (E09): the month's creature's available count doubled; under the Plague
   each loses the week's growth (to 0 at least) and is halved.
3. **A creature month's monsters** (0x4C90D0 to 0x4C9236; §10.2): new wandering monsters of the
   month's creature on free land tiles, then the guarded tiles refreshed (`refresh_guard_flags`
   0x4CA410).
4. Seven artifacts drawn by level (treasure 2, 2, 2, minor 4, 4, 4, major 8) into game +0x1F664
   to +0x1F67C: the Artifact Merchants' stock (E09).

### 7.5 The player's turn start

After the new day, `next_player_turn` sets the new current player's heroes' points (ADV-MOV-002),
and its turn start (0x4CC7D0, called at 0x4C78F7) fires **the map's timed events** for that player
(`map_timed_events` 0x4CD910, §11), then the towns' events (0x4CDA10, E09) and the scenario's
tests. So on a day the income is paid to every player first, then each player's events at its
own turn.

### 7.6 VCMI and the wiki

- VCMI's `NewTurnProcessor::onNewTurn` (`server/processors/NewTurnProcessor.cpp`): no income on
  the first turn; the income, then the week's kind (`pickWeekType`: Deity of Fire first; a month
  40 % double growth, then 10 % plague; a week 25 % a creature's), the heroes' mana and points,
  the towns' growth, the neutral garrisons, the spawn at a double-growth or Deity of Fire month;
  its twelve `hasDoubleWeek` creatures (`config/creatures/*.json`) are the original's twelve.
  VCMI draws a week's creature among all creatures with a town, without the RoE limits.
- The wiki: income "at the beginning of each day"; growth "on the first day of each week (beyond
  the first)"; special weeks 25 %, a special month 40 %, the Plague 10 % "at the first day of each
  month after the first" (Growth, Plague).

## 8. The day's income (T77a2)

### 8.1 What is counted (`player_daily_income` 0x4B8AF0, read whole)

The income +0x108 (seven i32) of every player not defeated is cleared (a defeated player's keeps
its last values, never paid), then:

1. **Mines** (0x4B8B25): each mine record with an owner (≥ 0) and a resource **below 6** adds
   `mine_income` 0x677978 (2, 1, 2, 1, 1, 1 by resource; the same values as `mine_daily_income`
   0x678288) to its owner's resource. Gold mines (6), Lighthouses (100) and A-31's resource 8 add
   nothing here.
2. Towns (E09): the Resource Silo (0x5C1280), Rampart's Treasury (10 % of the gold the player
   holds, on the week's first day) and Mystic Pond.
3. **Crystal Dragons** (creature 133): a player with one in an owned town's garrison (its
   garrison hero's army if it has one, else the town's) or in any of its heroes' armies gets **3
   crystal on the week's first day**, however many (0x4B8C32, 0x4B8DCF, 0x4B8E26).
4. **The resource artifacts** worn by the player's heroes (`count_worn_artifact` 0x4BABD0: the
   nineteen worn slots of the heroes in the player's list and of its towns' garrison heroes):
   +1 a day for each Everflowing Crystal Cloak (109) of crystal, Ring of Infinite Gems (110) of
   gems, Everpouring Vial of Mercury (111), Inexhaustible Cart of Ore (112), Eversmoking Ring of
   Sulfur (113) and Inexhaustible Cart of Lumber (114) of their resources; **+5** of mercury,
   sulfur, crystal and gems for each Cornucopia (140) (0x4B8CED to 0x4B8DAC).
5. **Gold** (`player_gold_income` 0x4C7930, §8.2).
6. **The resource specialists** (0x4B8DC2): each hero with an owner (all 156 records) whose
   specialty is a resource (kind 2) from wood to gems adds 1 of it a day (gold is §8.2's).
7. The computer's bonus (E10): with difficulty +0x1F6D8 above 2, a computer player's wood and ore
   gain a quarter and every resource but gold (difficulty − 2) sevenths, spread over the week by
   the day.
8. **The handicap** (game +0x1F6A8 per colour, the new game's setting; E10's scenario setup):
   1 or 2 take 15 % or 30 % off the six resources but gold (`handicap_factors` 0x677998, x87,
   truncated); gold's own factors are §8.2's.

### 8.2 Gold (`player_gold_income` 0x4C7930 and `hero_gold_income` 0x4E4390, read whole)

- **1 000 for each gold mine** the player owns (resource 6; a constant, 0x4C7984).
- Each town's gold (`town_gold_income` 0x5BF600) and its garrison hero's (E09).
- The Endless Sack (115), Bag (116) and Purse (117) of Gold: 1 000, 750 and 500 a day each.
- Each hero in the player's list: **Estates** by level (`skill_estates` 0x63EA18: 0, 125, 250,
  500), for an Estates specialist (kind 0, skill 13) times (1 + 0.05 × the hero's level) in f32,
  truncated; plus **350** for a gold specialist (kind 2, resource 6).
- A computer player's gold by difficulty (×0.75, ×1.25, ×1.5; E10); the handicap's ×0.85 or ×0.7.

### 8.3 What is paid, and when

The income is computed and paid in `new_day` (§7.2 step 6), **for every player not defeated at
once, before the next day's first turn**, from day 2 on; a mine flagged during a day pays from the
next day's start. The other three callers of 0x4B8AF0 (not read) are likely
the kingdom's views. An i32 addition, unsaturated (GAM-RES-001 saturates).

### 8.4 VCMI and the wiki

- VCMI's `generatePlayerIncome`: the towns, the bonuses `RESOURCES_CONSTANT_BOOST` (artifacts and
  specialists), the Crystal Dragons' 3 a week (`SPECIAL_CRYSTAL_GENERATION`), the handicap, then
  each owned object's `dailyIncome()` (mines, unhandicapped), then the computer's weekly bonuses
  spread over the days. Its Cornucopia gives +4 of each (`config/artifacts.json`; whether its
  parts' +1 add on top was not checked), the original 5; the gold specialists' 350 agree.
- **The handicap's order differs:** VCMI handicaps the towns' and artifacts' income but not the
  mines'; the original takes the percentage off everything counted, mines included (§8.1 point 8).
  E10's (no handicap in the engine).
- The wiki's Mine page: production at the beginning of each day (T77a1's §3.6).

## 9. The weekly objects (T77a2)

Visited from the neighbouring tile by any hero, owned by nobody, never asking (no decision):
each gives its content to the visitor's owner (`give_resource` 0x4E3600) once, then stays empty
until the next week's scan (§7.3). Their visits are `visit_object`'s cases (0x4A84F0); the human
player's message is the only difference a computer's visit makes (the flag argument).

### 9.1 The Windmill (112; `visit_windmill` 0x4A7FC0, read whole)

- **Its word:** bits 0 to 3 the resource (signed), bits 13 to 16 the amount, bits 5 to 12 the
  visited marks (§9.4).
- **At the new game** (`setup_map_objects` 0x4C2040) and **every week**: a resource 1 to 5 (mercury,
  ore, sulfur, crystal, gems: never wood or gold) and an amount **3 to 6**, both drawn.
- **A visit:** marks the team, then with an amount 0 the "no resources" message; else the message,
  the amount of the resource to the visitor's owner, and the amount cleared (the resource kept).

### 9.2 The Water Wheel (109; `visit_water_wheel` 0x4A7DE0, read whole)

- **Its word:** bits 0 to 4 the gold in units of 500; bits 5 to 12 the visited marks.
- **At the new game** (0x4C1F78): 1, so **500 gold in the first week**; **every week after:
  2, 1 000 gold**, whether it was visited or not (the wiki: 1 000 a week, 500 the first).
- **A visit:** marks the team; 0 the "no gold" message; else that gold to the visitor's owner and
  bits 0 to 4 cleared.

### 9.3 The Mystical Garden (55; `visit_garden` 0x4A3BC0, read whole)

- **Its word:** bits 0 to 4 the garden's index (a counter over the map's Gardens at the setup,
  wrapping past 31; at most 11 on a shipped map), bits 6 to 9 the resource (5 gems or 6 gold),
  bit 10 full.
- **At the new game** (0x4C1965) and **every week**: full, the resource gems or gold one in two.
- **A visit:** sets the index's bit in the **current player's** mask +0xB8 (its visited mark: the
  player's own, not its team's); when full, gold gives **500**, gems **5**, and bit 10 is
  cleared; else the "already" message.

### 9.4 The visited marks

The Windmill and the Water Wheel mark every player of the visitor's team in the word's bits 5 to
12 (`tile_mark_visited_by_team` 0x4FBF90), the Garden the current player alone; **the weekly
scan clears neither** (the Water Wheel's reset keeps bits 5 to 7, the Windmill's mask
0xFFFE1FF0 bits 4 to 12). So the marks say "visited once", not "emptied this week"; what the
hint shows from them is the map client's (`map_hint` 0x40B150's cases, E24 M3 T6a2) and
ADR-0016 point 6's for T77b.

### 9.5 VCMI and the wiki

- VCMI's rewardable configs agree on the three: the Windmill 3 to 6 of mercury to gems, the Water
  Wheel 500 in the first week then 1 000, the Garden 500 gold or 5 gems; each visited once a week,
  reset at the week's start. The wiki's Windmill, Water Wheel and Mystical Garden pages say the
  same (the Windmill "will never produce wood or gold").
- Both reset every object, visited or not, as the original.

## 10. The monsters' growth and the month's monsters (T77a2)

### 10.1 The weekly growth (§7.3's scan, class 54)

- **Only class 54**: random monsters (71 to 75, 162 to 164) are class 54 after the new game's
  setup, so every wandering monster grows; a monster with **"does not grow" (bit 18)** does not.
- **The count is kept in sixteenths:** its 16-bit value is the count (bits 0 to 11) × 16 plus a
  fraction in bits 27 to 30 (which the loader leaves 0: `load_monster_body` writes bits 0 to 26
  and 31). A week adds a tenth of the value, truncated, **capped at 0xFA00 (4 000.0)**, and writes
  both parts back (0x4C8BA5 to 0x4C8C00). The fight takes the count; a stack that survives one
  gets its new count in bits 0 to 11 only (0x4A6C9C, the second reading), **the sixteenths kept**.
- **The cap applies to the result**, so a growing stack above 4 000 (a count of 4 001 to 4 095
  from the map) becomes 4 000 at its first week (**A-32**; none on the shipped maps, which place
  none above 4 000).
- **D-49, the precision:** VCMI keeps the count in thousandths (`temppower`, +10 % a week, no
  growth at 4 000 or more); the wiki's formula is exact (a stack of 1 reaches 2 "after 8
  weeks"). The sixteenths agree with VCMI after one growth for every count; after two they differ
  for 66 counts of 4 095 (43 becomes 51 in the original, 52 in VCMI); a stack of 1 needs 10
  growths to reach 2 (16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 33 sixteenths). The wiki's 15
  creatures (16, 18, 19) agree with both.

### 10.2 The month's monsters (`new_month`, 0x4C90D0 to 0x4C9236)

In a month of a creature (and the Imp's), every tile of both levels is tested in level, row and
column order:

- **free** (tile +0xC bit 6: no object blocks it), **no entrance** (+0xD bit 4 clear), **not water
  or rock** (terrain 8, 9), **no event** on it (class 26);
- then **one chance in 200** (`random_between(1, 200) == 1`) places a monster (`place_object`
  0x4C9890: class 71 placed as 54, subtype the creature) whose count is **twice** a draw from
  the creature's map range (creature table +0x6C to +0x70) and whose aggression is drawn **1 to
  10**; the word's other bits as `place_object` leaves them (not read: nothing here sets "never
  flees", "does not grow" or a treasure).
- Nothing tests a guard zone, a hero's tile beside it, or the player's view; the guarded tiles
  are refreshed once after the scan.
- **D-50:** VCMI (`spawnWanderingMonsters`) places exactly ⌊free tiles ÷ 200⌋ monsters on shuffled
  tiles, skipping tiles a monster already guards, with **one** draw of the creature's range (not
  twice). The wiki: "any free tile except water has a 0,5 % chance"; special months' monsters have
  a mood of 1 to 10 (both agree on the mood).

## 11. The map's timed events (T77a2; `map_timed_events` 0x4CD910, `apply_timed_event` 0x4CD710, read whole)

- **The record** (52 bytes in the game's list +0x1FBF4, the map's order): the message, seven i32
  resources (+0x10), the players' bits (+0x2C), the human flag (+0x2D; before SoD always 1), the
  computer flag (+0x2E), the first day (+0x30) and the interval (+0x32). The loader adds **1 to the
  file's first day** (0x4FC2E0): the file's 0 is day 1.
- **When:** at each player's turn start (§7.5), each event in list order fires for the current
  player when its bit is set, the player is human with the human flag or a computer with the
  computer flag, and the day number (§7.1) **is the first day**, or the interval is not 0, the day
  is past the first and (day − first) is a multiple of the interval.
- **What:** each of the seven resources += the event's amount, **floored at 0** (a loss larger
  than the treasury takes all of it, the message showing what was taken); then, for a human, the
  message (the map's text). Campaign maps have a day-36 special (0x4CD842; not ours).
- **VCMI** (`CMapEvent::occursToday`): the same days, except that an interval of 1 fires one day
  early (`(day − first − 1) % 1` is 0 at −1); none of the 61 repeating events on the shipped maps
  has an interval of 1, so no dispute. VCMI applies them at the player's turn as well.
- **The wiki** (Map Editor): "the event will play on the chosen date", repeating at the interval;
  a take larger than the supply sets the resource to zero.
- **The census** (§1's for T77a2): 666 events on 139 maps (43 RoE, 41 AB, 55 SoD), 114 with
  resources (71 taking some; from −18 000 to +50 000 gold), 61 repeating (every 5, 6, 7 ×24,
  14 ×14, 21 ×3, 28 ×18 days), 604 for humans only, 1 for computers only, 61 for both.
  Town events (buildings, creatures) are E09's.

## 12. The heroes' spell points (T77a2; `new_day` 0x4C8237, `hero_mana_regained` 0x4E3F40, read whole)

At every new day (not day 1), **every hero record** (156: on the map, in prisons, the taverns'
pool):

- the cap: 10 × Knowledge (1 to 99) × Intelligence's factor (`hero_intelligence` 0x4E48B0),
  truncated (HER-MAN-001's maximum);
- **the Wizard's Well** (138) worn: the points become the cap, if below it;
- else the points + the day's regain, at most the cap, and never lowered (points above the cap
  stay);
- **the regain:** Mysticism by level (`skill_mysticism` 0x63E9C8: 1, 2, 3, 4; level 0 is every
  hero's 1); for a Mysticism specialist (kind 0, skill 8) trunc(that × (1 + 0.05 × level)) **+ 1**;
  then +1 for the Charm of Mana (73), +2 the Talisman (74), +3 the Mystic Orb (75), each worn or
  inside a worn combination.
- VCMI (`getManaNewTurn`, `manaRegain`): the same cap and never lowered; the Well as a 100 %
  regeneration; heroes of players only (its pool's heroes at their hire). Its specialist's
  formula was not compared.

Not in decision 192; a question for the checkpoint (§18): today a hero that casts in a battle
never regains a point.

## 13. Commands, decisions, events, the observation, replays and benchmarks (T77a2)

What increment 9 changes in the game's interfaces (ADR-0016, ADR-0017; the design is T77b's),
for the whole survey (§2 to §12):

- **Commands.** Flagging, a mine's visit and the weekly objects are steps of `MoveHero` (a visit
  from the neighbouring tile, ADV-OBJ-003). New: **building a boat from an own or allied Shipyard
  without a visit** (§5.3: an adventure command naming the Shipyard by type and anchor, validated
  against the owner's team, the price and the boat tile). The day's start is `EndTurn`'s (the last
  group's), no command of its own.
- **Decisions.** A guarded mine's question before its battle (GAM-DEC's consent, as a guarded
  pile's); the own mine's Leave Guards, the army dialog with the mine's guards as the other
  holder (T44's army dialog); the Shipyard's offer at a visit (a price, as the offers'). Nothing
  at the day's start asks.
- **Events.** A flag (the object by type and anchor, its new owner; public where seen) and the
  reveal (`TilesRevealed`); the income, one `ResourceGained` per resource and player in player
  order after `DayStarted`, the owner's; a new week and month with their kinds (public: the
  original tells every player); a monster's growth as `MonsterChanged` where its word changes; a
  month's monster as a new object where seen (no such event exists yet); a weekly object's gain
  (`ResourceGained`) and its emptying (ADV-VIS-006's known facts); a timed event's gains and
  losses (`ResourceGained`, `ResourceLost`) and its message by index (the client the text), the
  owner's; a built boat (public where seen).
- **The observation.** Every owned object's owner; a mine's production and an abandoned mine's
  drawn resource to its owner's team (§6); the date's week and month and the month's kind; a weekly
  object's visited mark to the team (the Garden's to the player, §9.4); the monsters' words as
  today (the fraction never).
- **The state.** Owners; mines' guards; the weekly objects' contents; the Garden marks by player
  and the others by team; the monsters' sixteenths; the month's kind; the heroes' mana (exists);
  the timed events (from the import; whether one fires is the day's, so no fired flags).
- **The draws.** At the setup: the weekly objects' first contents and the Gardens' order. Every
  week: the Windmills' and Gardens' contents (map order). Every month: the month's kind, the
  spawn's tiles, counts and moods. Which stream (the game's, or the setup's for the first week)
  and the order are T77b's; the week's creature (E09's) drawn now or with E09 changes later
  streams.
- **Replays.** The adventure and game state hashes, the map import's version (mines, Lighthouses,
  Shipyards and the weekly objects typed, the timed events imported) and the replay format (the
  boat command, the new answers) move; the golden replays are re-recorded. New golden replays:
  random play over several weeks on a fixture with mines (guarded and not), an abandoned mine,
  weekly objects, a Lighthouse and a Shipyard, timed events; scripted flags, a guarded mine
  fought, a boat built both ways.
- **Benchmarks.** The day's start becomes the costliest step of `EndTurn`: the weekly scan is
  over objects (thousands on an XL map, §1), the income over the owned records; a
  `day-start-xl-144` workload (a new week on the XL map) and random play over weeks.
- **The env** (the AI lane's): the observation encoding's owners and weekly objects, the action
  encoding's boat command: `cross-lane` bumps T77b tells the AI lane's inbox.

## 14. Disputes, quirk candidates and open points (T77a2)

T77a1's D-47, D-48 and A-29 to A-31 are in §6; the checkpoint's questions are §18.

- **D-49. A monster's growth: sixteenths or exact** (§10.1). The reading keeps a sixteenth; VCMI
  thousandths; the wiki's formula is exact. They agree after one growth for every count, after two
  they differ for 66 counts in 4 095 (43: 51 against 52), and a stack of 1 reaches 2 after 10
  growths (the wiki: 8). Impact: one creature now and then on a slowly growing stack.
  **Proposed:** the reading, with a batch E9 row (E9-9).
- **D-50. The month's monsters** (§10.2): the reading draws one chance in 200 per free land tile
  (no entrance, no event), twice the creature's map range, any guard zone; VCMI places exactly
  ⌊free ÷ 200⌋ on shuffled tiles, skips guarded tiles, once the range. The wiki: 0.5 % per tile.
  Impact: on average the same number, twice the creatures each (the reading). **Proposed:** the
  reading; no batch row (a creature month cannot be forced: 40 % at a month's start).
- **A-32. The 4 000 cap shrinks a larger stack** (§10.1): a growing stack of 4 001 to 4 095 from
  the map becomes 4 000 at its first week (VCMI leaves it). None on the shipped maps.
  **Proposed:** the rule as read (the documented limit; no flag), its row cheap (E9-10).
- **A-31 corrected** (§6): an abandoned mine whose resource is 8 adds **no income** (§8.1: the
  mines' loop counts resources below 6, the gold mines' resource 6 only), so it corrupts no
  treasury; its visit's message reads past its two tables (MINEEVNT, `mine_daily_income`). The
  proposal stands (the import refuses an empty set), as a bound, not a fix.
- **The income's addition is unsaturated** in the original (i32): GAM-RES-001's saturation holds
  (always fixed, no candidate).
- **Consensus, no dispute:** no income or reset on day 1; weeks of 7 days and months of 4 weeks;
  the weekly objects reset visited or not, the Water Wheel's 500 then 1 000, the Windmill's 3 to 6
  of mercury to gems, the Garden's 5 gems or 500 gold; the month's twelve creatures and the kinds'
  odds; the timed events' days (the file's day + 1) and their floor at 0; the Cornucopia's 5 (the
  wiki); the gold specialist's 350; the mana's cap and the Wizard's Well.
- **E10's, recorded:** the computer's weekly bonus by difficulty, the handicap (VCMI exempts the
  mines from it, the original does not), humans before computers in a day, a computer's timed
  events.
- **Not read** (for the task that builds the rule): the three other callers of 0x4B8AF0 (the
  kingdom's views); whether a town's garrison hero is also in its owner's hero list (then its
  artifacts and gold count twice, §8.1, §8.2); what a Shipyard's side tile's owner byte holds
  after a flag in the game (§5.3: a click there passes that tile's word, the flag writes the
  entrance's; the engine's command names the Shipyard, so no rule depends on it); `place_object` 0x4C9890 beyond its class; the weekly
  objects' hints (E24); the Mysticism specialist's `+ 1` against VCMI's specialty formula.

## 15. Batch E9's plan (T77a2)

Increment 9's batch, **batch E9**, runs on the client lane's harness in SoD on Easy, hot seat,
from maps `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E8 (decision
193's agent's call). Its spec and recipe are tasks of T77b's cut; this is the plan. **Two runs**:
**E9a** the visits (E9-1 to E9-5), **E9b** the day's start (E9-6 to E9-12, two to three weeks of
end turns).

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E9-1 | D-47 | a hero visits two neutral abandoned mines (their Troglodytes drawn at the new game) | each battle's opening: the defender's stacks and counts | several stacks each, summing to the mine's 100 to 200 (the army ratio table; VCMI: one stack) |
| E9-2 | §3.2, §3.3 | Blue's hero leaves one stack each in Blue's crystal cavern and sawmill, both on grass (Leave Guards); Red's hero attacks both | the question's text; each battle's terrain, field and defending stacks | the owned mine's question; the subterranean field for the cavern, grass for the sawmill; the guards as left (an army, not split) |
| E9-3 | §2.3, §2.4, §3.1 | Red's hero flags a neutral unguarded sawmill, visits it again, then Blue's hero visits it | the owner after each; the tiles revealed to Red; Red's wood at its next turn | Red's, then Leave Guards, then Blue's after the question; radius 3; +2 wood the next day |
| E9-4 | §4 | Red owns two Lighthouses at load; a hero ends its turn in a boat | the sea points at its next turn | the sea's maximum + 1 000 |
| E9-5 | §5.2, §5.3 | Red's Shipyard: a visit buying a boat; then a click on it from the map with no hero near; then with a hero on the boat tile | the treasury, the boat's tile and type; what the click opens | 1 000 gold and 10 wood, a type-1 boat on the first offset tile; the click opens the offer; the third refused |
| E9-6 | §7.1, §8.3 | Red owns a gold mine and an ore pit at load | the treasury at days 1, 2 and 3 | no change on day 1; +1 000 gold and +2 ore on days 2 and 3 |
| E9-7 | §8.1, §8.2 | Red's heroes: one with Basic Estates, one with the Endless Purse of Gold, one with the Cornucopia, a crystal specialist; a Crystal Dragon in an army | the treasury's daily change, days 2 to 8 | +125, +500, +5 of mercury, sulfur, crystal and gems, +1 crystal a day; +3 crystal on day 8 |
| E9-8 | §9 | a Windmill, a Water Wheel and a Mystical Garden: visited on day 1, again on day 2, again on day 8; a second set never visited until day 9 | each gain; each word (resource, amount, full) | 3 to 6 of mercury to gems, 500 gold, 5 gems or 500 gold; nothing on day 2; again on day 8 with the Wheel's 1 000; the second set full on day 9 |
| E9-9 | D-49 | growing monsters of 43, 81 and 15 (a control); one of 43 that does not grow | their counts (and the word's bits 27 to 30) on days 8 and 15 | 47, 89, 16 then 51, 97, 18; the fourth 43 (VCMI 52 and 98 on day 15) |
| E9-10 | A-32 | a growing monster of 4 095 | its count on day 8 | 4 000 (VCMI 4 095) |
| E9-11 | §11 | Red's timed events in this order: two of the file's day 0 (day 1), the first taking 99 999 gold, the second giving 300; one of the file's day 1 (day 2) every 2 days taking 500 gold; Red owns nothing | Red's gold at each turn start, days 1 to 6 | 300 on day 1 (the list's order); 0 on day 2 (floored, not −200); the take again on days 4 and 6 (0), none on days 3 and 5 |
| E9-12 | §12 (if in the scope) | a hero with 0 spell points, Knowledge 2, Basic Mysticism; one with the Charm of Mana; one with the Wizard's Well | spell points on days 2 and 3 | +2 a day; +3 a day; the maximum |

**Not in the batch:** D-48 (a one-in-101 difference), D-50 and the month's kinds (no creature
month can be forced), A-29 and the defeat (E10), A-30 (E09), the handicap and the computer's bonus
(E10).

**What the harness lacks** (`clients/capture/` is a common path: this lane builds it with the
harness check and tells the client and map-client lanes' inboxes): reads of the mine records
(game +0x4E38C: owner, resource, guards), a player's income +0x108 and the Garden mask +0xB8, the
date's three words, a tile's word (the weekly objects', a monster's bits 27 to 30); a click at a
Shipyard and the boat dialog's button. **What the spec lacks** (`h3-experiment`, battle-owned:
`cross-lane`; the writer `h3_map::writer` is this lane's): mines by subclass with an owner and
guards, the abandoned mine's resource set, Lighthouses, Shipyards, the three weekly objects, a
monster's "does not grow" flag, timed events (the writer writes none today), a hero's Estates,
Mysticism and specialty.

## 16. The proposed rule ids (T77a2)

For T77b's cut, for the whole survey; ids follow the catalogs' prefixes (the last ones:
ADV-MAP-002, ADV-MON-012, ADV-OBJ-007, ADV-SEA-007, ADV-VIS-006, GAM-BAT-014, GAM-CAL-001,
GAM-RES-002, HER-ADV-009, HER-MAN-001), each written when its rule is built. `ADV-OWN` (owned
objects), `ADV-MIN` (mines), `ADV-SHY` (Shipyards) and `ADV-WEK` (the weekly objects) are new
families.

| Id | Rule | Survey | Open points |
|---|---|---|---|
| ADV-OWN-001 | flagging: the classes (53, 220, 42, 87 here; dwellings increment 10), a visit flags a neutral or another team's object for its visitor's owner, never an own or allied one; radius 3 revealed to the new owner's team | §2.3, §2.4 | — |
| ADV-OWN-002 | the owner at load from the map; a colour nobody plays refused by the import | §2.2, §6 | — |
| ADV-OWN-003 | what a viewer sees: every owner public, a mine's production to its owner's team | §2.7, §6 | — |
| ADV-OWN-004 | a defeated player's objects become neutral, a mine keeping its guards | §2.5 | A-29 (E10) |
| ADV-MIN-001 | mines on the map: subclasses 0 to 6, 220 as 53.7, a guards' army | §2.1, §3 | — |
| ADV-MIN-002 | a mine's visit: an own one the Leave Guards dialog, an ally's nothing, another's guards asked then fought (a neutral single stack as a wandering monster) | §3.1 to §3.3 | D-47 |
| ADV-MIN-003 | the abandoned mine: its resource drawn at load from the map's set, 100 to 200 Troglodytes at the new game, an empty set refused | §3.4 | D-48, A-31 |
| GAM-BAT-001 (amended) | a crystal, gold or abandoned mine's battle on the subterranean field | §3.2 | — |
| ADV-OWN-005, HER-ADV-009 (amended) | Lighthouses: flagged; +500 sea points a day for each owned one | §4 | A-30 (E09) |
| ADV-SHY-001 | the Shipyard's boat tile fixed at load | §5.1 | — |
| ADV-SHY-002 | a boat built at a visit: the price, the refusals, the new boat (ADV-SEA-001) | §5.2 | — |
| ADV-SHY-003 | a boat built from an own or allied Shipyard without a visit | §5.3 | §17 |
| GAM-CAL-002 | the new day (day 2 on): its order; the week on days 8, 15, …; the month on days 29, 57, … | §7.1, §7.2 | — |
| GAM-CAL-003 | the week's and the month's kinds | §7.3, §7.4 | (scope, §18) |
| GAM-CAL-004 | the map's timed events | §11 | — |
| GAM-RES-003 | the day's income from mines, paid to every player not defeated at the new day | §8.1, §8.3 | — |
| GAM-RES-004 | the heroes' income: resource and gold artifacts, the Cornucopia, Estates, resource and gold specialists (derived in `h3-heroes`, ADR-0013 point 2) | §8.1, §8.2 | (scope) |
| GAM-RES-005 | the Crystal Dragons' 3 crystal a week | §8.1 | (scope) |
| ADV-WEK-001 | the Windmill | §9.1 | — |
| ADV-WEK-002 | the Water Wheel | §9.2 | — |
| ADV-WEK-003 | the Mystical Garden | §9.3 | — |
| ADV-VIS-006 (amended) | the weekly objects' visited marks: the team's, the Garden's the player's | §9.4 | — |
| ADV-MON-013 | the weekly growth: sixteenths, a tenth a week, the 4 000 cap, "does not grow" | §10.1 | D-49, A-32 |
| ADV-MON-014 | the month's monsters | §10.2 | D-50 (scope) |
| HER-MAN-002 | the day's spell points | §12 | (scope) |
| ADV-MAP-002 (amended) | the import types mines, Lighthouses, Shipyards and the weekly objects; the timed events imported | §1, §11 | — |

## 17. The second reading (T77a2)

Two fresh agents, one after the other, re-read the deciding code without this survey or the
address map's summaries (2026-10-30), from questions per function: **A** the new day, the income
and its gold, the new week, the new month and the spell points (six questions); **B** the three
weekly objects' visits and setup, the timed events and their reader, the Shipyard's use without a
visit (§5.3), A-29's empty slot 0, a surviving monster's count written back, a combination's
locked slots (five questions).

**A confirmed** point by point: the date's three words and their rollovers, the new week inside
the day's branch and the month's test every day, the week's counter raised inside the new week
and the month's inside the new month (§7.1, §7.2); every contribution to the income with its
condition and amount, the mines' resource test below 6 and the table 0x677978, the gold mines'
constant 1 000, the Cornucopia's 5, the specialists' wood to gems, the Crystal Dragons' 3 once a
week, the computer's bonus and the handicap (§8.1); the gold's sources and factors, Estates and
the 350 (§8.2); the new week's order, the scan's tile test (the same bit, read as +0xC bit 12), the
hero lifted off and put back, each class's write (30, 48, 54, 55, 78, 109, 112), the Stables' bit
cleared, the neutral towns' 80 % or 40 % (§7.3, §10.1); the month's draw and kinds, the Imp's
override and the tutorial's normal months, the twelve creatures, the towns' doubling and the
Plague's halving, the spawn's tile tests, its one in 200, class 71 placed as 54, twice the
creature's range and the mood of 1 to 10, the seven artifacts (§7.4, §10.2); the spell points'
cap, the Wizard's Well, the regain with the specialist's + 1 and the three mana artifacts, every
hero record (§12).

**A added** (written into the sections): the towns' byte +2 is cleared at difficulty 2 or more or
with a human in the owner's team (§7.2); the Cover of Darkness hides and the owned sight is then
revealed again (§7.2); the Crystal Dragons count in an owned town's garrison hero's army or else
the town's (§8.1); a garrison hero in its owner's list too would count twice (an open point,
§14); the month's monster's other bits are not read (§10.2); the scenario's test at the new day is
the resource victory's (E10).

**B confirmed** point by point: the three weekly objects' words, setups, gains, clearings and
empty messages, the team's marks (bits 5 + colour for each colour of the owner's team) and the
Garden's mark in the current player's +0xB8, each written before the empty test (§9); the timed
events' day number, list, flags, days and repeat test, the seven additions floored at 0 with no
upper cap, the message for a player with both human flags, and the reader's first day + 1 and
human flag 1 before SoD (§11).

**B corrected one point** (checked in the code and written into §5.3 and the address map): **the
Shipyard's use runs for the cursor's click mode 8**, not a mode 7: `hero_move_loop`'s jump index
is the mode minus 1 (0x407BC6), so its entry 7 (0x4085A8) is mode 8's and mode 7 (the End Turn
dialog's control 7, 0x409C80) is entry 6, a hero's walk. `exe.py switch` labels the entries
without the minus 1. The click's path closes: the left click (0x4097E0, controls 0 and 37, then
0x40A5D0) stores the clicked tile, which must be visible to the viewer, and reaches the loop with
or without a selected hero; the **clicked tile's** word goes to `visit_shipyard`.

**B added:** A-29's call (§6): `monster_battle` with an empty slot 0 fights an empty army
(`adventure_battle` 0x4AD470), not traced further, then writes a count of 0; a surviving
monster's new count goes into bits 0 to 11 only, so the sixteenths survive a fight (0x4A6C9C,
§10.1); an assembled combination writes only its own slot and counts its parts' slots as locked
(per-kind counters at hero +0x1C5, `hero_equip_artifact` 0x4E2A00), so `count_worn_artifact`
never counts a part inside it and the Cornucopia's income is its 5 alone, the wiki's 4 + 1
(§8.1); the event's editor name is read and dropped.

**Not read by either:** a Shipyard side tile's owner byte after a flag (§14), how a battle settles
an empty side (A-29, E10), the computer player's branches (E10), network play, saved games, the
towns' and the dwellings' parts of the day's start (E09, increment 10).

## 18. For T77b: the checkpoint's questions (ADV-Q37)

**Answered** (2026-10-30, the user; the epic's decisions 196–199): every point as recommended;
in point 4, (a) to (c) in and (d) with E09.

1. **The disputes D-47 and D-49** (§6, §14): (a) the code's side, built and confirmed or reverted
   by batch E9's row (as decision 129); (b) only once batch E9 confirms it; (c) VCMI's. The wiki
   is silent on D-47 and gives the exact formula against D-49. Recommended (a).
2. **D-48 and D-50** (no batch row: a one-in-101 difference; a creature month cannot be forced):
   (a) the readings (100 to 200 inclusive; one chance in 200 a tile, twice the creature's range);
   (b) VCMI's. Recommended (a): the wiki's 0.5 % a tile agrees with D-50's reading.
3. **The quirk candidates:** A-29 (fixed: the battle takes the stack where it is; E10), A-30 (E09
   classifies), A-31 (corrected, §14: the import refuses an empty set as a bound), A-32 (the
   cap as read, no flag). Recommended: agreed.
4. **What increment 9 takes beyond decision 192** (each read here, all from the day's start):
   (a) **the heroes' income** (resource and gold artifacts, the Cornucopia, Estates, resource and
   gold specialists) and **the Crystal Dragons' crystal** with the mines' (one income, GAM-RES-004
   and -005); (b) **the day's spell points** (HER-MAN-002: today a hero never regains a point);
   (c) **the month's kind and its monsters** (GAM-CAL-003 in part, ADV-MON-014); (d) the week's
   creature drawn now (its growth E09's) or with E09. Recommended: (a), (b) and (c) in; (d) with
   E09 (a stream of its own then, so nothing shifts).
5. **What stays out, confirmed:** the towns' income and buildings (the Silo, the Treasury, the
   Mystic Pond, the Mage Guild), the towns' growth and the Plague's effect, the Artifact Merchants
   (E09); the tavern (E09); the dwellings' refill and the Refugee Camp (increment 10); the
   Fountain of Fortune's, Magic Spring's and Stables' weekly resets (their objects' increments);
   the computer's bonus, the handicap and the humans-first order of a day (E10). Recommended:
   confirmed.
6. **Batch E9** (§15): (a) all 12 rows in two runs before the build; (b) beside the build; (c) the
   disputes' rows only (E9-1, E9-9, E9-10); (d) none. Recommended (a), as batches E7 and E8.
7. **What a player sees** (§6, §9.4, §13): every owner public; a mine's production and an
   abandoned mine's drawn resource to the owner's team; a weekly object's visited mark to the team
   (the Garden's to the player alone), never its content before a visit; the week's and month's
   kinds public. Recommended: as the original.
8. **The common rules as written** (§7 to §12): no new day on day 1; the income for every player
   at once before the next day's first turn; the weekly objects reset visited or not, the monsters
   grown in map order with the sixteenths kept; the timed events at each player's turn start, in
   the map's order, floored at 0; one new command (a boat from an own or allied Shipyard); the
   Shipyard's tile at load and an owner colour nobody plays refused (§6). Recommended: as written.
