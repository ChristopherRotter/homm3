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
  `count_owned_records` 0x4BAE70, `boats_in_use` 0x4CCDD0, `random_between` 0x50B230.
  **Read in part:** `visit_object`'s cases 42, 53 and 87 (0x4A84F0), `load_object_body`'s cases
  53 and 87 (0x502E00), `setup_map_objects`'s cases 42, 53 and 87 (0x4C0CC0),
  `map_template_read` 0x503780 at the class's mapping (`object_class_values` 0x63A6E4),
  `player_defeated` 0x4F11A0 (its shipyards, mines and dwellings), `battle_setup_battlefield`
  0x4643B0 at its mine test, `hero_max_land_movement` 0x4E4990 at its Lighthouse terms,
  `mine_hint_text` 0x40D670, `adventure_mouse_move` at its two Shipyard sites (0x40E636,
  0x40E7D0), `hero_move_loop` 0x407B80 at its case 7, `end_turn_button` 0x409A70 (the adventure
  screen's control messages, its switch 0x409AB4; the name is its End Turn case's) at its control
  7. **Not followed:** the computer player (`ai_fight_answer` 0x52BD10, its own mines, its boats),
  network play (the message records 0x49BF90 and 0x482390, `network_map_message` 0x482010), saved
  games, the scenario's conditions after a flag (`scenario_condition_met` 0x4F2CE0: E10), the boat
  dialog itself (0x5D2720), the guards' description (0x40ABE0), the day's start (T77a2).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `lib/mapObjects/MiscObjects.cpp`
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
  Mine, Lighthouse, Shipyard, Map Editor - Objects, Object Events. Re-check a quote before a rule
  cites it with `high` confidence.
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §1; the text tables
  `advevent.txt`, `GENRLTXT.TXT` and `MINEEVNT.TXT` (`extracted.tar`) for which message is which.

## 0. Findings in short (T77a1)

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
  13 863 wandering monsters on 159 maps, 506 of them not growing (73 maps).

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
pays it is T77a2's (`new_day` 0x4C7FE0).

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
- **The use** (`hero_move_loop` case 7, 0x4085A8): the tile under the mouse given to
  `visit_shipyard` as if visited (its team test, then the boat offer); mode 7 is set by
  `end_turn_button` 0x409A70's control 7 (0x409C80). How a click in mode 8 becomes control 7
  was not traced, nor which tile's word the use then reads (the clicked tile; a blocked tile's word
  holds the boat tile, §5.1, and the owner byte the loader gave it): the second reading's (§6).
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
  creature and count (none, 0). What `monster_battle` does with them was not read (a battle with
  no creature, or the guards cleared without a fight). **Proposed:** fixed (the battle takes the
  stack where it is), classified at the checkpoint; it matters only with player defeat (E10).
- **A-30. The Castle's Lighthouse building helps every hero** (§4): no owner test. For E09 (the
  town's buildings); recorded here because it shares the Lighthouse's bonus. **Proposed:** E09
  classifies it (the wiki and VCMI give the owner's heroes only).
- **A-31. An abandoned mine without a resource.** An empty set gives resource 8 (§3.4), whose
  income the day's start would add beyond the player's seven resources: state-corrupting, so
  always fixed. The editor refuses an empty set (the wiki); no shipped map has one or a bit past
  gold. **Proposed:** the import refuses such a mine (as an invalid map), no flag.
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
