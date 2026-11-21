# E08 Prison, Market of Time and adventure spells survey (E08 T137a)

Date: 2026-11-21. Scope: E08 increment 15 (decisions 394, 395), input for T137b's ADR amendment
drafts, the design note and the cut. **Whole** (the user at T137a's start, after the reading was
measured: the Prison's visit about 0.9 KB with its callees, the Market of Time no code at all, the
five spells about 3.5 KB mostly unread, with A-13's trade-off and the other four named). Input,
not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with its
sources and tests.

**Sources** (retrieved 2026-11-21):

- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml` or `structs.toml`. The
  reading follows the human player's path. **Read whole:** `visit_prison` 0x4A3EB0 (`visit_object`
  0x4A84F0's case 62, 0x4A9CAD) and `hero_move_record` 0x49CB20; `placed_heroes_setup` 0x4CAE10
  again (its prisoner branch); `visit_object`'s switch (class 50's slot); `adventure_spell`
  0x41C490 (now all ten cases); `hero_cast_fly` 0x4E59A0, `hero_cast_water_walk` 0x4E5DD0;
  `scuttle_boat_cast` 0x41CDF0 and its target dialog (`scuttle_target_create` 0x491AF0,
  `scuttle_target_proc` 0x491D00); `adventure_spell_cast` 0x41D090 (Dimension Door) and its
  target dialog (`door_target_create` 0x4916F0, `door_target_proc` 0x491900); `view_area_contains`
  0x407AC0, `view_cell_tile` 0x407B10; `hero_special_ground` 0x4E4EC0; `summon_boat_choice`
  0x4E54A0 again (its ties); `new_day` 0x4C7FE0's hero loop (0x4C80CF). **Read in part:**
  `adventure_cast_open` 0x41C2F0 (its refusals, read before by E24), `spell_book_proc` 0x59D040 (a
  spell's click: the cost against the points), `hero_spell_cost` 0x4E5240 (read before: the armies'
  ±2 only with an enemy army), `hero_relocate` 0x41D930 (its calls: no boat, no guard, no visit).
  **Read before** and used here: `summon_boat_cast` 0x41C8A0 (E24 M4 T2d3), `create_boat`
  0x4BB250, `hero_boards_boat` 0x4A1010, `hero_lands_from_boat` 0x49E670, `tile_special_ground`
  0x4FCE20, `tile_spell_field` 0x4FCF40, `hero_school_mastery` 0x4E5100, `step_cost` 0x4B1740
  and the pathfinder (survey `E08-hero-movement.md` §2, §3), `hero_visions_level` 0x4E5DE0
  (E09). **Not followed:** the computer player (`ai_hero_walk` 0x42FEE0's casts, the pathfinder's
  mode 4: E10's, E31's), network play (the map messages), Town Portal (E09 increment 7),
  the pictures, texts and sounds, the drawing of the spells' dissolves.
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`): `config/spells/adventure.json` (every adventure spell's
  levels), `config/gameConfig.json` (`dimensionDoorTriggersGuards`, `perPlayerOnMapCap`),
  `lib/spells/adventure/DimensionDoorEffect.cpp`, `SummonBoatEffect.cpp`, `RemoveObjectEffect.cpp`
  (Scuttle Boat), `AdventureSpellMechanics.cpp` (the points, the casts per day),
  `lib/mapObjects/CGHeroInstance.cpp` (`onHeroVisit`'s prison branch), `config/objects/generic.json`
  (`marketOfTime`).
- `wiki`, through search snippets only (the page fetch is refused in this session):
  <https://heroes.shoutwiki.com/wiki/Dimension_Door>, <https://heroes.thelazy.net/index.php/Spells>,
  <https://heroes.shoutwiki.com/wiki/Fly>, <https://heroes.thelazy.net/index.php/Fly>,
  <https://heroes.shoutwiki.com/wiki/Sea>, <https://heroes.shoutwiki.com/wiki/Boat>,
  <https://heroes.thelazy.net/index.php/Prison>; re-check a quote against the page before a rule
  rests on it.
- `manual`: the user's RoE manual (`D:\Heroes3\HoMM 3 Complete\Heroes3_Manual.pdf`, the spell pages
  77 to 79 and the cursors' page); the AB and SoD manuals say nothing of these spells or objects.
- The census (§1): a throwaway program over `h3_map::read` on the 160 shipped maps (not
  committed).

## 0. Findings in short

- **The census** (§1): 89 prisons on 41 maps, none on water, no two prisoners of one type on a
  map, no prisoner of a type its map allows, restricts or customises map-wide; 61 with a custom
  experience (28 maps), 45 with an army, 12 with skills, 10 with artifacts (6 a spell book), none
  with custom spells or primary skills. **One** Market of Time (`Titans Winter`, RoE). The five
  spells come from scrolls (11), Summon Boat's 11 fixed shrines, two Pandora's Boxes, two placed
  heroes, one hero type (Gem's Summon Boat), artifacts and the mage guilds; 26 maps ban some of
  them.
- **The Prison** (§2): its prisoner is set up at load as a placed hero (levels, army, skills,
  artifacts) but joins nobody. A visit frees it for the visitor's player unless that player has
  **8 heroes on the map** (the player's list: garrison heroes not counted, as the tavern's): the
  prison removed, the hero shown on the prison's entrance, facing right, **its full points for the
  day**, its status flags cleared, last in the player's list. A prisoner whose type is no longer
  a prisoner's (only a campaign's carried hero can do it) empties the prison with a message.
- **The Market of Time** (§3) has **no visit at all**: class 50 falls in `visit_object`'s
  out-of-range case. A visit is a step onto its entrance and nothing more (VCMI: "not implemented").
- **Casting on the map** (§4): the cast button, the book (a spell costing more than the hero's
  points refused), then: **Cursed Ground** refuses spells above level 1, **an anti-magic
  garrison** refuses all, neither charged. The mastery is the hero's on **its tile's special
  ground** (Magic Plains every school Expert, Lucid Pools, Fiery Fields, Rocklands, Magic Clouds
  one school each): a change of `h3-heroes`' masteries (HER-SPL-003) for ADR-0013. The cost is the
  table's at that mastery (no army's ±2 on the map), at least 1. Every effect lasts until **the
  new day**, which clears the fly and water-walk levels, Disguise, Visions and the day's
  Dimension Door count of all 156 heroes.
- **Summon Boat** (§5, read before): on land with a free water neighbour; 50/50/75/100 %; the
  hero's last boat, else **the nearest by |dx| + |dy| on any level** among the empty boats of its
  player or of none; at Advanced and Expert a new boat when none (64 at most). **Scuttle Boat**
  (§6): any empty boat whose tile shows in the view, **fog or not**; 50/50/75/100 %; a failure
  charged; a cancel not.
- **Fly and Water Walk** (§7): the level is the mastery (the price tables' 140, 140, 120, 100
  from increment 5); **Fly is refused in a boat** (a message, free), and silently and free while
  it is on or with Angel Wings; **Water Walk is cast in a boat** (charged, the "already on" and
  Boots tests skipped) and holds after landing. Boarding clears both (increment 8).
- **Dimension Door** (§8): needs **points above 0**; **1, 2, 3 or 4 casts a day** by mastery
  (the spell table's effect column); the target any tile of the view centred on the hero
  (|dx| ≤ 9, dy −8 to **+9**: the pick's last 8 pixel rows), **neither blocked nor an object's
  entrance, explored or not**; land to land or water to water in a boat, **else "failed" and
  charged** (the mana, the points, a cast); then **300 points, 200 at Expert**, no visit, **no
  guard's battle**, the reveal around the arrival.
- **The other four** (§9): Visions (+0x129, read by four popups), Disguise (+0x10E, read by the
  hero's displayed army 0x52EAD0), View Earth and View Air (the world view, 0x5FBF90): about 1.5 KB
  read, 4 to 6 KB unread. Recommended: **E12** with the information objects.
- **VCMI** (§11) agrees on almost every amount; it differs on Summon Boat's choice (**D-75**),
  Scuttle Boat's fog (A-50), the door's range (**D-76**: 9 and 8) and the Prison's mana (equal in
  effect). **A-13's trade-off** (§13): three options for the checkpoint.

## 1. The census

The 160 shipped maps (49 RoE, 55 AB, 56 SoD; `original-data/maps.tar`), read with `h3-map`.

### 1.1 The prisons (62)

- **89 prisons on 41 maps** (25 objects on RoE maps, 42 AB, 22 SoD), 17 underground; most on
  `Pandora's Box ` (30), `Myth and Legend` (6). **None stands on water** (the tiles under the
  object's anchor and its left neighbour). Every body is a hero's (`docs/formats/h3m.md`): owner
  `0xFF`, a fixed type (no random prisoner), **no two prisoners of one type on a map** and no
  prisoner of a type a placed hero holds; no prisoner's type is allowed, restricted by SoD's
  disposed heroes, or customised map-wide (E09's census for the pool: survey `E09-towns.md` §4.2).
- **Customisations:** experience 61 (28 maps; levels 3 to 15 and two at 42, survey
  `E08-progression.md` §6.1), name 30, portrait 6, secondary skills 12, army 45, artifacts 10
  (6 with a spell book), patrol 1 (`Xathras Prize`, radius 10), formation all loose; custom
  spells and primary skills none; AB's spell byte "the type's own" on 42.

### 1.2 The Market of Time (50)

One, on `Titans Winter` (RoE) at (54, 102, 0), template `AVXmktt0.def`, one visitable tile.

### 1.3 The adventure spells' sources

| Source | Summon Boat | Scuttle Boat | Fly | Water Walk | Dimension Door | the other four, Town Portal |
|---|---|---|---|---|---|---|
| Spell Scrolls (93; 100 on the maps) | 4 | 0 | 4 | 1 | 2 | View Earth 5, View Air 10, Town Portal 4 |
| Shrines (88–90, fixed spell) | 11 (9 maps, level 1) | — | — | — | — | — |
| Pandora's Boxes and events | 0 | 2 | 0 | 0 | 0 | Visions 7, Disguise 2, View Earth 2, Town Portal 3 |
| Placed heroes' custom spells | 0 | 0 | 0 | 1 | 1 | Disguise 1 |
| Hero types' own (packs) | Gem | — | — | — | — | Ayden View Earth |
| Towns that require it | 29 | 2 | 0 | 0 | 1 | — |
| Maps that ban it (26 in all) | 14 | 14 | 15 | 19 | 17 | Visions, View Earth, Disguise, View Air 2 each; Town Portal 6 |

The artifacts: Boots of Levitation as 5 objects on 2 maps and worn by 1 hero; Angel Wings never
placed; the Sea Captain's Hat (both boat spells), the four tomes and the Spellbinder's Hat (level
5: Fly, Dimension Door) only by draws. The mage guilds hold them by their levels (E09's draw);
towns ban Fly 281 times and Dimension Door 290. 118 boats and 330 Shipyards stand on the maps.

### 1.4 The grounds and garrisons that change a cast

| Ground (class) | Objects | Maps | Effect on a cast (§4.2) |
|---|---|---|---|
| Cursed Ground (21; SoD 223) | 382; 48 | 53; 14 | spells above level 1 refused |
| Magic Plains (46; SoD 230) | 413; 23 | 41; 10 | every school Expert |
| Lucid Pools (228) | 78 | 15 | water Expert (Summon Boat, Scuttle Boat, Water Walk, Visions) |
| Fiery Fields (226) | 108 | 18 | fire Expert (Visions only) |
| Rocklands (231) | 184 | 28 | earth Expert (View Earth, Visions, Town Portal) |
| Magic Clouds (229) | 134 | 22 | air Expert (Fly, Dimension Door, View Air, Disguise, Visions) |
| Anti-magic garrisons (33 and 219, subclass 1) | 28 + 17 | — | every cast refused while standing in one |

## 2. The Prison

### 2.1 At load

`load_hero_body` (0x5021C0) marks a prisoner's type 0x41 in `hero_owner` (game +0x4DF18; survey
`E09-towns.md` §4.1), so no draw takes it. `placed_heroes_setup` (0x4CAE10, read again) gives
**every** type's record to `placed_hero_setup` (0x4D8B30), so a prisoner gets its custom
experience with its levels at load, its army, skills, artifacts and spell points exactly as a
placed hero (survey `E08-progression.md` §3.2); then, for 0x41, it skips the rest: no owner's list,
no reveal, not shown (0x4CAEF9), and it is never moved onto a town's entrance (0x4CAEA7). Its
record then rests in the game's array until a visit. The new day regenerates its spell points as
every record's (survey `E08-economy.md` §12), already at its maximum.

### 2.2 The visit (`visit_prison` 0x4A3EB0)

Args: the visitor, the tile, the packed entrance, the human flag (messages only). The prisoner's
type is the tile's word.

1. **Not a prisoner any more** (`hero_owner` not 0x41): ADVEVENT 104's box (to a human), the
   prison removed (`remove_map_object`, `hero_loss_effect` 1: a pickup sound and the dissolve),
   nothing else. Only a campaign's carried hero (`carried_hero_setup` 0x486DC1) rewrites a
   prisoner's mark on the human player's path; on the shipped maps the branch is unreachable.
2. **Eight heroes:** the current player's hero count (record +0x1, the list of heroes on the map:
   a garrison hero is not in it, survey `E09-tavern.md` §3) at 8 or more: ADVEVENT 103's box,
   nothing changes; the prison stays.
3. **Freed:** ADVEVENT 102's box; the prison removed; `hero_move_record` (only a message record for
   the other players' views, §2.3); the hero's owner (+0x22) and `hero_owner` the visitor's colour,
   the colour's bit added to `hero_masks` (+0x4DFB4); the hero appended to the player's list (+0x8,
   count +1); its tile **the prison's entrance**, its status flags (+0x105) 0, **facing right**
   (+0x47 2), its points and the day's maximum (+0x4D, +0x49) `hero_max_movement`'s: **a full
   day's points, the day it is freed**; the tile's object class cleared and the hero shown there
   (class 34); a pickup sound (`random_between(1, 7)`, presentation only), the dissolve, the network
   message, the hero list redrawn. The visitor's own state is untouched (its step into the
   neighbour's object is the visit's, increment 1).

Nothing tests the colour masks: a disposed hero's mask (SoD) does not stop its freeing (none on
the shipped maps, §1.1). Nothing else changes the prisoner: its experience, army, skills and spell
points are the load's.

### 2.3 Details

- `hero_move_record` (0x49CB20, read whole): a 28-byte record of the current
  player, the hero, its old and new owner, its in-boat bit, a flag, its old place and the new one,
  appended to the game's vector at +0x4E7AC (the turn's records, `turn_records_drop` 0x49D6C0);
  no game state.
- The prisoner is invisible to everyone: a right-click shows the object's name
  (`tile_click_path` 0x40A5D0 sends class 62 to `object_popup_text`), not the hero.
- A computer player's visit runs the same function with the human flag clear (E10's).

## 3. The Market of Time (50)

`visit_object`'s switch (0x4A853A, index = class − 2) sends class 50 to its out-of-range case
(0x4AA7CE) with 18, 19, 21, 26, 36, 40, 46, 65–77, 114–211 and 214: **no visit**. The template has a
visitable tile, so a hero steps onto the entrance (a normal step's price) and nothing happens; the
object is otherwise a decoration with its passability. VCMI 1.7.5's `generic.json` calls it an
"unused/not implemented H3 object present on some RoE maps" with a `generic` handler (no effect).
The wiki snippets say nothing. A rule of one line (ADV-OBJ-008, §16).

## 4. Casting on the adventure map

### 4.1 The path

`adventure_command`'s case 8 (the cast button, 0x409F64) → `adventure_cast_open` 0x41C2F0: the
selected hero (`mobilize_hero` with 0, 0, 1: **the view centred on it**), the spell book (kind 1,
the hero's tile's field) run modal. A click on a spell in the book (`spell_book_proc` 0x59D440) of
the book's kind: `hero_spell_cost` with the book's enemy army and ground against the hero's spell
points (+0x18); above them GENRLTXT 207's box and nothing; else the book closes with the spell.
Then `hero_special_ground` (0x4E4EC0, read whole: `tile_special_ground` of the hero's tile):

- **21 (Cursed Ground)** and the spell's level above 1: GENRLTXT 748's box, nothing spent;
- **33 (an anti-magic garrison's tile, or a hero standing in one)**: GENRLTXT 685's box, nothing
  spent;
- else `adventure_spell` (0x41C490) with the spell.

The button tests only a selected, owned hero and a human player (`cast_button_state` 0x403BF0).
**Not read:** what the book lists for a hero without a spell book (no test of the book on this
path: `hero_wears_artifact(0)` is called only by the battle and the computer; open point O-1,
batch row E15-15).

### 4.2 The mastery and the cost

`adventure_spell` takes the current player's selected hero, its mastery `hero_spell_mastery(spell,
hero_tile_spell_field)` and switches on the spell (0 to 9; above 9 nothing). The field is
`tile_spell_field` of the hero's tile: Magic Plains 1, Lucid Pools 6, Fiery Fields 7, Rocklands 8,
Magic Clouds 9, else −1; `hero_school_mastery` (0x4E5100) gives **Expert** on 1 for every school
and on 6–9 for water, fire, earth, air, else the best of the spell's schools' levels. So a hero
without Air Magic casts Dimension Door at Expert on Magic Plains or Magic Clouds (§1.4).
`h3-heroes`' `spell_masteries` today answers "on ordinary ground" (HER-SPL-003): the adventure needs
the ground's raise from the same derivation (the global HANDOFF's trap; ADR-0013, T137b).

The cost: `hero_spell_cost(spell, no army, field)`: the table's cost at that mastery (8/7/7/7 and
so on, the packs' `cost`), at least 1; the armies' ±2 (Pegasi, Mages) only with an enemy army
(0x4E52DA), never on the map; spent by `hero_spend_mana` (+0x18 less the cost, at least 0). Fly
computes the same inline (§7.1). **No day's limit** on the map but Dimension Door's (§8); several
spells and several casts a day as the points allow.

### 4.3 The day's end

`new_day` (0x4C7FE0, at 0x4C80CF to 0x4C80FD) runs over all 156 hero records: +0x105 bits 0 and
17 cleared, **+0x10E (Disguise), +0x112 (Fly), +0x116 (Water Walk), +0x129 (Visions) set to −1 and
+0x10D (the day's Dimension Doors) to 0**. Nothing else writes them but the casts, boarding
(`hero_boards_boat`: +0x112 and +0x116 −1), a new game (`hero_reset`) and the copies of a hero
record (the save, the network). So a spell cast on a player's turn lasts through the other
players' turns until the day changes.

## 5. Summon Boat (spell 0; read by E24 M4 T2d3)

`summon_boat_cast` (0x41C8A0): the hero on water (terrain 8): GENRLTXT 334, free. The first of
its eight neighbours clockwise from up (`step_directions` 0x678150) that is on the map, water and
free of objects (+0x1E 0); none: GENRLTXT 335, free. `random_between(1, 100)` above the chance
(the spell table's effect column, 50/50/75/100; the draw happens at 100 too): GENRLTXT 337,
**charged**. Else `summon_boat_choice` (0x4E54A0, read again):

1. `hero_boat(hero, 0)`: the first boat in use whose last captain (+0x20) is this hero and that
   no hero is aboard (+0x24 0), **whatever its owner and level**;
2. else among the boats in use, empty, of the current player or of none (+0x1C −1): **the least
   |dx| + |dy|, levels ignored**; a tie goes to **the later** in the game's vector (`jg`, 0x4E5530);
3. none: null.

A boat found: taken off its tile and put on the neighbour on the hero's level, its facing kept.
None: at Advanced or Expert `create_boat` (0x4BB250) there (the hero's owner, type 0; −1 when 64
boats are in use: GENRLTXT 336, charged); at None or Basic GENRLTXT 336, **charged**. Then the
sound, the dissolve and the cost. The summon writes no owner (E24's reading; boarding does, increment 8).

## 6. Scuttle Boat (spell 1)

`scuttle_boat_cast` (0x41CDF0, read whole): the target dialog (`scuttle_target_create` 0x491AF0,
`scuttle_target_proc` 0x491D00): over the view's map area (`view_area_contains` 0x407AC0) the
tile under the pointer (`view_cell_tile` 0x407B10, §8.2) is a target when its class is **8 (an
empty boat; a boat with a hero is the hero's tile, class 34) with the entrance bit**: the
scuttle cursor (0x2A); else none. **No fog test, no owner test**: any empty boat in the view,
the hidden ones included (A-50). A left click on a target closes the dialog with it; a cancel
(right click, Escape): GENRLTXT 732, **free**. Then `random_between(1, 100)` above the chance
(50/50/75/100): GENRLTXT 338 with the name, **charged**. Else the boat taken off its tile
(`hero_hide`), its record's "in use" (+0x18) cleared, the sound and the dissolve; then the cost.

## 7. Fly (spell 6) and Water Walk (spell 7)

### 7.1 The casts

- **Fly** (0x41C6B8): in a boat (+0x105 bit 18): GENRLTXT 193's box, **free**. Already flying
  (+0x112 not −1) or **wearing Angel Wings** (72): **nothing at all, free** (the book closes). Else
  the sound and `hero_cast_fly` (0x4E59A0, read whole): +0x112 the mastery; the cost computed again
  inline with `hero_spell_mastery(6, the hero's field)` and the table at +0x330 + 0x20, at least 1,
  taken from +0x18 (at least 0) without `hero_spend_mana` (the hero list's slot redrawn by hand):
  the same amount as `hero_spell_cost` on the map.
- **Water Walk** (0x41C796): **in a boat the two tests below are skipped and the spell is cast**;
  on land, already water-walking (+0x116 not −1) or **wearing Boots of Levitation** (90): nothing,
  free. Else `hero_cast_water_walk` (0x4E5DD0: +0x116 the mastery) and `hero_spend_mana`. In a boat
  the level is ignored by the price and the pathfinder (they take −1 there, survey
  `E08-hero-movement.md` §2.1), so a Water Walk cast at sea works after landing, the same day (A-49).
- **A later rise of the mastery** (Air Magic learned the same day) does not change a cast's level:
  the re-cast is refused as "already on" (the wiki calls it a bug, fixed in HotA; A-51).

### 7.2 The movement

The level 0 to 3 picks the price from `fly_walk_step_cost` (0x6778AC: 140, 140, 120, 100), already
data (`[params] fly_cost`, `water_walk_cost`, ADV-MOV-009, ADV-MOV-010): increment 5 built level 3
(the artifacts) and every path the levels take (the pathfinder, the end of a move, the guards
passed, the drop to 0 with the hero's own fields, `ADV-QUIRK-17`). The spells add the levels 0 to
2 and nothing new in the code: every reader tests −1 (§4.3's list, `hero_step_allowed`,
`after_step_guard_check`, `hero_move_loop`, `pathfinder_run`). With the spells the drop to 0
prices with the spell's level (the field is the spell's), so `ADV-QUIRK-17` stays the artifacts'
alone. The manual's "60 % / 80 % / 100 % of normal movement" is the same price read the other way.

### 7.3 A flight and the unexplored (A-13)

Unchanged by the spells: an unexplored tile is passable only flying and never a stop
(`pathfinder_expand`, survey `E08-hero-movement.md` §3.1 point 3), so the original's flyer crosses
hidden strips whatever its level; a water-walker does not (the fog test comes before water's
case). The wiki's Sea page says shroud on water between two seen land tiles does not stop a
walking or flying hero (**D-78**, row E15-14). The trade-off: §13.

## 8. Dimension Door (spell 8)

### 8.1 The cast (`adventure_spell_cast` 0x41D090, read whole; the mastery as argument)

1. **Points left not above 0** (+0x4D, signed): GENRLTXT 126's box, free.
2. **The day's casts** (+0x10D) at or above the spell table's effect column at the mastery
   (+0x440 + 0x34: **1, 2, 3, 4** for None to Expert): GENRLTXT 339 with the name, free.
3. The target dialog (§8.2); a cancel: the minimap redrawn, GENRLTXT 732, free.
4. The target off the map (`position_on_map`) or the dialog not accepted: as a cancel.
5. **The kind of the target:** a hero in a boat needs water (terrain 8) and a hero on land needs
   land; otherwise GENRLTXT 71 ("failed") and the minimap, **then everything below is spent anyway**
   (A-52; VCMI's `waterLandFailureTakesPoints`).
6. Else `hero_relocate(hero, target, the spell's sound, 0, 1, 0)`: hidden, put on the target,
   revealed around it by its scouting radius, shown, selected (E08-teleporters.md §5: no points,
   **no guard**, no visit). A hero at sea takes its boat along (the boat is put back where the
   hero leaves the water, `hero_move_step` 0x48095A).
7. **The points:** less **300, or 200 at Expert** (mastery 3; 0x41D28D), at least 0; the cost
   (`hero_spell_cost(8, …)`), the hero list's slot, and +0x10D + 1.

### 8.2 The target (`door_target_create` 0x4916F0, `door_target_proc` 0x491900)

A full-screen dialog over the adventure screen. On every mouse move inside the view's map area
(item 0 of the adventure dialog: x 8 to 599, y 8 to 551; `view_area_contains`, whose x bound adds
the item's y, harmless as both are 8) the screen cell (x / 32, y / 32) gives the tile
`view_cell_tile`: the view's corner (manager +0xE4, +0xE6) plus the cell. The tile is a target when
its map item's word +0xC has **neither bit 0x100 (blocked: an object's blocked tile, rock) nor
0x1000 (an object's entrance, a hero included)**: the door's cursor (0x29) and the accept flag;
else no cursor. A left click with the flag closes the dialog with the tile; a right click or
Escape cancels.

- **No fog test:** a hidden tile is a target when it is free, and the cursor tells which hidden
  tiles are (A-53).
- **The range:** the view is centred on the hero at the cast (its corner the hero's tile less
  (9, 8), E24 design §3.5), so cells 0–18 and 0–17 give **dx −9 to +9 and dy −8 to +9**. Row 17 is
  only the area's last 8 pixel rows (y 544 to 551): tiles are drawn at y 8 + 32 j but picked at
  y / 32, so the lower 8 pixels of each drawn row pick the row below, and row 17 is drawn nowhere
  (A-54). Off-map tiles are refused at step 4.
- Water and land are both targets here; step 5 sorts them out.

### 8.3 What it does not do

No battle at the arrival, also beside a monster (VCMI: `dimensionDoorTriggersGuards` false by
default); the next step out of a guarded tile is the movement's rule (increment 6). No visit of an
object (the target is never an entrance). No minimum points beyond 1: a hero with 1 point casts it
and ends at 0.

## 9. The other adventure spells (named for the checkpoint, decision 394)

| Spell | The cast (`adventure_spell`) | What reads it | Read | Unread (about) |
|---|---|---|---|---|
| Visions (2) | +0x129 the mastery, a sound, a message, the cost | `hero_visions_level` 0x4E5DE0 (read; a Rogue makes 3), `hero_visions_reach` 0x4E5E10; the hero, town, garrison and monster popups (`hero_popup` 0x416590, `town_popup` 0x4167A0 read by E09, `garrison_popup` 0x416F80, `monster_popup` 0x417150) | the cast, the level, the town popup | the reach and three popups' branches: 1.5 KB |
| View Earth (3) | `view_world_run(3, mastery)` 0x5FBF90, the cost | the world view's four handlers (`view_world_run`'s entry) and their filters (VCMI: resources; Advanced mines; Expert the terrain) | the cast | 2–3 KB |
| Disguise (4) | +0x10E the mastery, the cost | `hero_displayed_army` 0x52EAD0 (0x8C8 bytes; read +0x10E at 0x52EF7B, 0x52F12A), called by `hero_popup` and the computer | the cast | 2.2 KB |
| View Air (5) | `view_world_run(5, mastery)` | as View Earth (VCMI: artifacts; Advanced heroes; Expert towns) | the cast | shared with View Earth |

All four change only what a player sees (the observation's projections, ADR-0017 point 8), never
a rule of play: the family of the information objects decision 394 left out (Eye and Hut of the
Magi, the Sanctuary's sign, Cover of Darkness) and of E09's Visions groundwork (the town view's
Rogue, the inbox note of 2026-11-06). **Recommended: E12**, with the information objects, so the
viewer's model is designed once; the casts' four cases are a few lines then. Option: Visions alone
in increment 15 (its town half is built).

## 10. What a viewer sees

- **The Prison:** an object of its class to every player; the prisoner unseen by everyone,
  its holder's player included (no right-click details, §2.3). Freed: a hero appears on the
  entrance (seen by those who see the tile), the prison disappears.
- **The Market of Time:** a decoration with an entrance.
- **The casts:** the caster's owner sees the hero's points, spell points and the day's spell
  state (fly and water-walk levels, the door's casts left: the hero list's slot and the hero
  screen); another player sees only the map's changes on tiles it sees: a boat moved, created or
  gone, a hero vanishing and appearing (the door), never the spell. The door's and Scuttle Boat's
  cursors are the owner's (A-53, A-50: what they tell of hidden tiles).

## 11. VCMI and the wiki

| Point | Reading | VCMI 1.7.5 | Wiki, manual | Result |
|---|---|---|---|---|
| Prison: the limit | 8 on the map (the list) | `HEROES_PER_PLAYER_ON_MAP_CAP` 8, garrisons not counted | "eight heroes outside of towns" (thelazy) | agree |
| Prison: the freed hero | full points, flags 0, facing right, the mana as loaded | full points (a boat on water), the mana set to its maximum | — | agree (the loaded mana is the maximum) |
| Market of Time | no visit | "not implemented" | — | agree |
| Cursed Ground, anti-magic garrison | refused, free | a spell's problems (not compared further) | — | — |
| Summon Boat's choice | the hero's last boat, then nearest Manhattan, own or unowned, ties to the later | nearest by `dist2d`, any owner, not boarded | manual: the hero's boats, then other heroes'; a forum: own or neutral | **D-75** |
| Summon Boat's chance, creation | 50/50/75/100; new at Advanced and Expert; 64 at most | the level's power; `createdBoat` at Advanced and Expert | manual: the same, 64 | agree |
| Scuttle Boat's target | any empty boat in the view, fog or not | range 9 × 8, `ignoreFow` false | manual: "a boat, unless occupied" | A-50 |
| Fly, Water Walk prices | 140, 140, 120, 100 | 40, 40, 20, 0 over 100 | 40 %, 20 %, none; 60/80/100 % | agree (increment 5) |
| Fly in a boat | refused | — (not compared) | "cannot be used by a hero on a boat" (shoutwiki) | agree |
| Fly re-cast after Air Magic | refused, old level kept | — | a bug, fixed in HotA | A-51 |
| Door: casts a day | 1, 2, 3, 4 | 1, 2, 3, 4 | 2, 3, 4 (basic to expert) | agree |
| Door: the points | needs > 0; 300, 200 at Expert | `movementPointsRequired` 0, taken 300, 200 | manual: zero stops it; three tiles, two | agree |
| Door: the range | dx ±9, dy −8 to +9 | `rangeX` 9, `rangeY` 8 | manual: "visible location" | **D-76** (A-54) |
| Door: the fog | hidden free tiles accepted | `ignoreFow`, `exposeFow` true | manual: "visible" | A-53 |
| Door: water or land wrong | "failed", all spent | `waterLandFailureTakesPoints` true (SoD) | — | agree (A-52) |
| Door: guards at arrival | none | `dimensionDoorTriggersGuards` false | — | agree |
| Water walking over hidden water | not passable (fog first) | never (explored tiles) | Sea page: walking or flying across shroud works | **D-78** |

## 12. Disputes, quirk candidates and open points

| Id | What | Sources | Proposal |
|---|---|---|---|
| D-75 | Summon Boat's boat: the hero's last boat first, whatever owner and level, then the nearest (Manhattan, levels ignored, ties to the later) of the player's or nobody's | the reading, the manual (the hero's own boats first), a forum (own or neutral) against VCMI (nearest Euclidean, any owner) | the reading (two sources); row E15-9 |
| D-76 | The door's range: dy +9 included (the reading) or \|dy\| ≤ 8 (VCMI) | the reading against VCMI | VCMI's 9 × 8 as the rule, the extra row A-54 |
| D-77 | — (the door's points: > 0 agreed by the reading, VCMI and the manual; a guide's "0, 1 or 2 tiles" alone) | — | settled, no row |
| D-78 | A water-walker over unexplored water | the reading against the wiki's Sea page | matters only for §13's option C; row E15-14 |
| D-79 | A hero without a spell book casting a worn scroll's adventure spell | no book test on the path (the reading) against HER-BAT-011 (the battle's) | row E15-15 before a rule |
| A-49 | Water Walk is cast in a boat (charged, its "already on" and Boots tests skipped) and holds after landing | the reading | harmless and accepted (the player pays; the effect is the spell's) |
| A-50 | Scuttle Boat's target has no fog test: a hidden boat in the view is sunk, and the cursor reveals it | the reading; VCMI `ignoreFow` false | **fixed** (explored tiles only; the legal moves never depend on hidden tiles, ADV-MOV-008) |
| A-51 | A Fly or Water Walk re-cast is refused while on, so a mastery raised the same day keeps the first level | the reading, the wiki (a bug, HotA fixed) | harmless and accepted (faithful), or fix-optional |
| A-52 | The door's water/land failure spends the points, the mana and a cast | the reading, VCMI (SoD's) | harmless and accepted: the command is legal and fails, as the original's |
| A-53 | The door accepts a hidden free tile and refuses a hidden blocked one, and its cursor tells which | the reading, VCMI (`exposeFow`) | a choice (§13's sibling): (a) faithful, (b) explored tiles only, (c) any hidden tile a target, a blocked one failing as A-52 (recommended: nothing hidden in the legal moves, the original's reach kept) |
| A-54 | The door's pick reaches one row below the view (dy +9) through its last 8 pixel rows | the reading | fixed (the range a parameter, 9 × 8) |
| O-1 | What the spell book lists for a hero without a book (D-79) | not read | row E15-15 |
| O-2 | `hero_visions_reach`, the world view's filters, `hero_displayed_army` | not read | E12's (§9) |

The Prison's "released" branch (§2.2 point 1) is a campaign's and not a quirk: the engine has no
campaigns (E08 has no row for it).

## 13. A-13's trade-off (decisions 75, 78; `ADV-QUIRK-15`)

With the spells flying becomes common (scrolls, Magic Clouds, level-5 guilds), so the user's
"TBD later" is due. The original passes over unexplored tiles as over blocked ones (passable only
flying, never a stop; rock refused before the fog test, so a hidden rock stops the flight: the
leak). Three options:

- **A. Keep it fixed** (today): a flight never enters an unexplored tile. Wide obstacle ranges
  with hidden tiles behind their edge stop a flyer until a walker or a reveal explores them.
- **B. The original's** (faithful): the reach and the price use the hidden tiles' truth. The
  legal moves then tell rock from the rest under the fog (the information leak ADR-0017 point 8
  forbids for the masks).
- **C. The compromise** (recommended, the user's "route to the closest unexplored, but free hex"
  made exact): **an unexplored tile is assumed flyable, at the fly price from a blocked tile, never
  a stop** (as the original treats it, but without looking at what is hidden), so the reach and the
  path depend only on what the player knows. The move then walks the path step by step as it does
  today; each step reveals by the scouting radius, and **a step onto a tile that turns out to be
  rock (or, for D-78, water to a water-walker) stops the move on the last tile it may stop on**,
  with the points of the steps made; nothing is spent for the steps not made. The original's
  passes over hidden blocked land and water stay the same; only a hidden rock differs (the original
  never offers that path, the engine offers it and stops). With A-53's option (c) the door gets the
  same rule: knowledge decides what is offered, the truth what happens.

## 14. The interfaces' sketch (for T137b)

- **The prisoner** (h3-game, the town lane's pool): the import (map import v29) builds each
  prisoner's hero as a placed hero's (levels at load, GAM-SET-001's army, skills, artifacts, spell
  points) and the prison object holds its hero id (the adventure state: a `Prison { hero }` kind);
  E09's `HeroStatus::Prisoner` already keeps the type out of the pool. The visit: freed through the
  same path as the tavern's hire (the list's eight shared with TWN-TAV), the hero on the entrance,
  facing right, the day's full points, the prison gone; events `PrisonerFreed { hero, player,
  tile }`, `PrisonFull { player }`. The view: the prison's kind only.
- **The Market of Time:** its object type with `visit = "none"` (a step onto its entrance, no
  event), or the decoration it is today plus its entrance.
- **The cast:** one command `Cast { hero, spell, target: Option<Tile> }` in `h3-game` (the
  adventure's `AdventureCommand`), validated in this order: the actor's hero, a spell of the
  adventure category among `h3_heroes::available_spells` (D-79 decides the book), the ground (Cursed
  Ground's level, an anti-magic garrison), the cost against the spell points (the mastery from
  `h3-heroes` with the tile's ground), then the spell's own refusals (§5 to §8). Refused commands
  change nothing; the charged failures (Summon's and Scuttle's draws, A-52) are accepted commands
  with a `CastFailed` event. Draws from the adventure's stream (ADR-0016).
- **The hero's day state** (the adventure state hash): fly and water-walk levels (−1 or 0–3, beside
  the artifacts' 3), the door's casts today; cleared at the new day and at boarding; Disguise and
  Visions later (§9).
- **Events:** `SpellCast { hero, spell, mastery, cost }`, `BoatSummoned { boat, from, to }`,
  `BoatCreated`, `BoatScuttled { boat }`, `CastFailed`, `HeroTeleported { hero, from, to }`
  (each viewer told by what it sees, ADR-0017 point 8).
- **The ruleset** (adventure part): the door's points (300, 200 at Expert), its range (9, 8), the
  view window for Scuttle Boat; the chances and the door's casts are the core's spell effect columns
  (already harvested); the grounds' fields are core data the battle reads already (E04c).
- **Wire and env:** `h3-protocol` v79 (the command, events and the hero's day fields), the env's
  encoding v18 (the AI lane's: a cast without a target per spell, the door's and Scuttle Boat's
  targets over the 19 × 17 window around the hero), the replay format v24 (`cast` lines).

## 15. Batch E15's plan

On the client lane's harness, SoD on Easy, hot seat, maps from `h3-experiment` (as batches E3 to
E14); the spec and recipe are tasks of T137b's cut. Red casts from a spell book with the spells
learned (or worn scrolls), its points and spell points read before and after each.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E15-1 | §2.2 | a prisoner with 5 000 experience and an army; Red's hero frees it on day 1 | the freed hero's level, points, facing, tile; Red's list | its level at load; a full day's points; right; the prison's entrance; last in the list |
| E15-2 | §2.2 | Red with 7 heroes on the map and one garrison hero visits a prison; then with 8 on the map | the prison; the list | freed; then the message, nothing changed |
| E15-3 | §3 | a hero steps onto the Market of Time's entrance | the points, any message | a step's price, nothing |
| E15-4 | §4.1 | casts on Cursed Ground (Fly, Summon Boat) and in an anti-magic garrison | the spell points, the effects | Fly refused free; Summon Boat cast; all refused free |
| E15-5 | §4.2 | a hero without Air Magic on Magic Plains casts Dimension Door five times | the casts, the points after each | 4 casts, 200 each; the fifth refused |
| E15-6 | §8.1 | Dimension Door with 1 point left; with 0 | the result | cast, points 0; refused |
| E15-7 | §8.1, A-52 | Dimension Door from land onto a water tile | the points, the spell points, the casts left | "failed"; 300 and the cost spent, one cast used |
| E15-8 | §8.2, A-53 | the door's cursor over hidden free and hidden blocked tiles; a cast onto a hidden free tile | the cursor; the arrival | the door's cursor only on the free one; the hero there, the reveal |
| E15-9 | §5, D-75 | two empty boats: the hero's last (far, underground) and a nearer own one, an enemy's nearest | the boat summoned (twice, the boats reset) | the last boat, from the other level; never the enemy's |
| E15-10 | §6, A-50 | Scuttle Boat on an empty boat under the fog in the view; on an occupied boat | the cursor; the boat | the cursor; sunk at Expert; no cursor on the occupied |
| E15-11 | §7.1 | Fly cast twice; Fly with Angel Wings; Fly in a boat | the spell points | the second free and nothing; nothing; the boat message, free |
| E15-12 | §7.1, A-49 | Water Walk cast in a boat, then landing and walking onto water | the spell points; the path | charged; the hero water-walks |
| E15-13 | §13, A-13 | a Fly spell (level 0) facing a hidden strip, as E5-9 | the path's marks and price | through the strip at 140 a step |
| E15-14 | D-78 | Water Walk facing hidden water between two seen land tiles | the path | none (the reading) or across (the wiki) |
| E15-15 | D-79, O-1 | a hero without a spell book wearing a Dimension Door scroll | the cast button, the book | not read: the record decides |
| E15-16 | §8.2, A-54 | the door's target at dy +9 (the pointer at the view's bottom 8 pixels) | accepted? | accepted |

**Not in the batch:** the computer's casts (E10, E31), network play, Town Portal (E09), the four
of §9. **What the harness lacks** (`clients/capture/`): the cast's flow (the button, the book's
click by spell, the target's click by tile), reads of the hero's +0x10D, +0x112, +0x116 and the
mouse cursor's frame; the boats' records (game +0x4E3BC). **What the spec lacks** (`h3-experiment`,
battle-owned: `cross-lane`): prisons with a prisoner's body, the Market of Time, special grounds
under a hero (E04c's `[[map.ground]]` exists for battles), boats with a last captain, learned
spells and worn scrolls at the start, hidden tiles placed by design.

## 16. The proposed rule ids

- **ADV-PRI** the Prison: **-001** the prisoner at load (a placed hero's setup, in no list, out
  of the pool; TWN-SET-005 kept); **-002** the visit (the eight on the map, the freeing: the
  entrance, facing right, full points, flags cleared, the prison gone; no mask test); **-003** what
  a viewer sees (the prisoner hidden).
- **ADV-OBJ-008** the Market of Time (no visit).
- **ADV-SPL** the adventure spells: **-001** the cast (the command, the book's cost test, Cursed
  Ground above level 1, an anti-magic garrison, the mastery by the hero's ground, the cost at least
  1 without the armies'); **-002** the day's end (the new day clears the spells and the door's
  count; boarding clears Fly and Water Walk); **-003** Summon Boat (D-75); **-004** Scuttle Boat
  (A-50); **-005** Fly (its refusals, A-51); **-006** Water Walk (A-49, A-51); **-007** Dimension
  Door's limits (points above 0, the casts by mastery); **-008** its target (the range, free tiles,
  A-53, A-54); **-009** its effect (the water/land failure A-52, 300 or 200, no guard, no visit,
  the reveal); **-010** what a viewer sees.
- **HER-SPL-003** amended (or **HER-SPL-008**): a spell's mastery on a map tile's special ground
  (ADR-0013's derived property).
- **ADV-MOV-009, -010** amended: the spells' levels 0 to 2; **ADV-MOV-008** and
  **ADV-QUIRK-15** by §13's answer.

## 17. For T137b: the checkpoint's questions

1. **A-13** (§13): A, B or C (recommended C).
2. **A-53** (the door and the fog): (a), (b) or (c) (recommended (c), with §13's C).
3. **D-75** (Summon Boat's choice): the reading (recommended).
4. **D-76 / A-54** (the door's range): VCMI's 9 × 8 (recommended).
5. **A-49, A-51, A-52**: harmless and accepted (recommended); **A-50**: fixed (recommended).
6. **The other four spells** (§9): E12 with the information objects (recommended), or Visions now.
7. **D-79** (a bookless hero's scroll): settle by row E15-15 before a rule; meanwhile the battle's
   HER-BAT-011 (a book needed).
8. **The mastery on the ground** (§4.2): `h3-heroes` takes the ground (ADR-0013 point 2's
   derivation gains it; the battle's E04c raise moved there or shared).
9. **The cut:** likely the Prison and the Market of Time (one task), the cast's frame with Summon
   and Scuttle Boat, Fly and Water Walk, Dimension Door, the env and wire, batch E15's spec and
   recipe, its scoring, the audit.
