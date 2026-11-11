# E08 garrisons and bonus visitables survey (E08 T108a)

Date: 2026-11-11. Scope: E08 increment 12 (decisions 295, 296), input for T108b's ADR amendment
drafts, the design note and the cut. In two parts (the user at T108a's start, decision 297):
**T108a1** (§0 to §9: the census of all seventeen classes, the garrisons in the file and at load,
the visit, the battle, the capture, the exchange, the passage, what a viewer sees, VCMI and the
wiki, the garrisons' disputes and proposed rules) and **T108a2** (§10 on: the fifteen bonus
visitables, the interfaces, batch E12's plan, the proposed ids, the checkpoint).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-11-11):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_object` 0x4A84F0 at its case 33 (0x4A9061);
  `load_garrison_body` 0x502A00; `garrison_set_owner` 0x4C6960; `garrison_owner_message` 0x4C23E0;
  `garrison_exchange_dialog` 0x5D1130; `garrison_popup` 0x416F80; `army_has_creatures` 0x4496A0;
  `town_garrison_hover_other` 0x5C8080; `pathfinder_pass_through` 0x56AB40 at its entry and case
  33; `hero_move_step` 0x4805E0 at its switch 0x481544 (case 33); `setup_map_objects` 0x4C0CC0 at
  its case 33; `garrison_dialog_hover` 0x5D05F0 (again, for the owner tests). **Read in part:**
  `garrison_exchange_create` 0x5D0E00 (its calls), `garrison_quick_view_create` 0x5307D0 (its
  items), `town_garrison_hover` 0x5C7400 (its owner tests). **Read before** and used here:
  `tile_special_ground` 0x4FCE20, `battle_setup_battlefield` 0x4643B0, `side_may_use_magic`
  0x41F890, `reveal_owned` 0x4C7C50, `player_defeated` 0x4F11A0, `tile_object_owner` 0x40FD20,
  `object_class_values` 0x63A6E4, `object_classes_from_above` 0x63AB00, `garrison_dialog_create`
  0x5CE830, `adventure_battle` 0x4AD470 (in part). **Not followed:** the computer player
  (`ai_garrison_visit` 0x524370, the pathfinder's modes from 4: E10's), network play
  (`map_message_send` 0x482390), the pictures, the cursor and the sounds (E24's).
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`), `lib/mapObjects/MiscObjects.cpp` (`CGGarrison`),
  `lib/mapping/MapFormatH3M.cpp` (`readGarrison`), `lib/pathfinder/PathfindingRules.cpp`,
  `server/CGameHandler.cpp` (`moveHero`, `setOwner`), `client/HeroMovementController.cpp`,
  `client/widgets/CGarrisonInt.cpp` (`mustForceReselection`), `lib/battle/CBattleInfoCallback.cpp`.
- `wiki`: <https://heroes.thelazy.net/index.php/Garrison>,
  <https://heroes.thelazy.net/index.php/Anti-Magic_Garrison>; the mirror's Trivia page
  (<https://heroes.shoutwiki.com/wiki/Trivia>: the removable troops' flag).
- The census (§1): a throwaway example over `h3_map::read` on the 160 shipped maps (not
  committed).

## 0. Findings in short (T108a1)

- **A correction of decision 295.** Class **219 is the vertical garrison**, not the anti-magic one:
  the loader makes it class 33 (`object_class_values` 0x63A6E4), keeping its subclass. The
  **anti-magic garrison is subclass 1 of both** (33.1 `AVCgar20.def`, 219.1 `avcvgarm.def`).
  Census (§1): 238 garrisons on 61 maps, 168 horizontal (33) on 53, 70 vertical (219) on 25 (AB
  and SoD maps only); 45 anti-magic on 21 maps (RoE 6, AB 9, SoD 6), 9 of them on 6 RoE-format
  maps.
- **The file** (§2): an owner, seven stacks and, from AB on, the "troops removable" byte (RoE: always
  removable); the load keeps a record of 0x40 bytes per garrison (H3API's `H3Garrison`). The shipped
  garrisons are clean: every owner a played colour, no random stack, no count of 0 or above 200,
  both sides of every passage open, no hero placed in one, no shared entrance.
- **The visit** (§3): a garrison of the visitor's team opens the exchange; any other is fought when
  it holds a creature, with no question, and taken when won or empty.
- **The battle** (§4): the garrison's army as it stands (never split), its owner the defending side
  without a hero (a neutral one unowned); on the tile's terrain; an anti-magic garrison's tile, or a
  hero standing on one, forbids every cast (the battle's `BattleSetup.anti_magic`, BAT-TER-002,
  already built).
- **The capture** (§5): the owner set, a radius of 3 revealed to the new owner, then the exchange; a
  defeated player's garrisons become neutral with their armies.
- **The exchange** (§6): with a removable garrison of the visitor's own, the army dialog's full
  rules; with an ally's or a non-removable one, the visitor may only put stacks into its **empty**
  slots: never take one, merge into one or swap with one.
- **The passage** (§7): a human's path goes **through** a garrison of its team without visiting it;
  every other garrison ends the path. A hero standing in a garrison stands in the passage.
- **What a viewer sees** (§8): the owner's flag; the army by the town's levels (team, Visions 3,
  the viewer's taverns), as the engine's TWN-VEW-002/-003 already show a town's garrison.
- **VCMI and the wiki** (§9): disputes **D-59** (the passage), **D-60** (a hero attacked in an
  anti-magic garrison), **D-61** (merging into another's garrison); no quirk candidate; open points
  on the packs and the import. A gap found: GAM-BAT-011's ownable classes lack 219.

## 1. The census (all seventeen classes)

The throwaway example read the 160 shipped maps (RoE 41, AB 60, SoD 59, all read) and counted each
object of classes 33, 219 and the fifteen bonus classes by its template's class and subclass. A
template's tiles are given as offsets (columns left, rows up) from its bottom right corner.

### 1.1 Garrisons

| Class.subclass | Template | Objects / maps | RoE / AB / SoD |
|---|---|---|---|
| 33.0 horizontal | `AVCgar10.def` | 140 / 42 | 84 / 29 / 27 |
| 33.1 horizontal, anti-magic | `AVCgar20.def` | 28 / 21 | 9 / 11 / 8 |
| 219.0 vertical | `AVCvgr.def` | 53 / 23 | 0 / 18 / 35 |
| 219.1 vertical, anti-magic | `avcvgarm.def` | 17 / 4 | 0 / 5 / 12 |
| all | | 238 / 61 | 93 / 63 / 82 |

Underground: 40 of class 33, 5 of class 219. The anti-magic ones stand on 21 maps: 6 RoE-format
(*Middletown*, *Noahs Ark*, *Serpents Treasure*, *The Five Rings*, *Titans Winter*, *Warmongers*),
9 AB and 6 SoD. The maps by version: 19 RoE, 23 AB, 19 SoD. The four
maps with a vertical anti-magic garrison also hold a horizontal one. Most on one map: 16 of class 33
(*Manifest Destiny*), 11 of class 219 (*Back For Revenge - Allied*).

- **Footprints:** the horizontal garrison blocks the three tiles (2,0), (1,0), (0,0) of its bottom
  row, its entrance the middle one (1,0); the vertical one blocks (0,2), (0,1), (0,0), its entrance
  the middle one (0,1). One template per class and subclass, each with one entrance.
- **Owners:** 199 neutral (33.0 109, 33.1 23, 219.0 50, 219.1 17), 39 owned (33.0 31, 33.1 5, 219.0
  3), every owner a colour the map plays (23 a colour a human may take, 16 a computer-only one).
- **Armies:** 82 neutral garrisons are empty, 117 hold one to seven stacks; 12 owned ones are empty,
  27 hold two to seven. 101 creatures (codes 0 to 144), no random stack, no count of 0, the largest
  stack 200. **Not removable:** 11 (7 neutral, 4 owned), all on AB or SoD maps.
- **The passage:** for each entrance the tiles beside it across the strip (the three above and the
  three below a horizontal one, the three left and right of a vertical one) were tested for a tile
  on the map, not rock and not blocked by another object: all 238 are open on both sides. No hero
  is placed on an entrance, no entrance is shared with another object, every entrance is on land.

### 1.2 The bonus visitables (for T108a2)

| Class | Object | Objects / maps | RoE / AB / SoD | Under­ground | Templates (entrances) | Most on one map |
|---|---|---|---|---|---|---|
| 11 | Buoy | 209 / 56 | 60 / 101 / 48 | 10 | `AVSbuoy0` (one tile, blocked) | 17 |
| 14 | Swan Pond | 183 / 73 | 86 / 65 / 32 | 9 | `AVSclvd0` 56, `AVSclvg0` 103, `AVSclvs0` 24 (1,0) | 8 |
| 28 | Faerie Ring | 255 / 102 | 82 / 93 / 80 | 18 | `AVSring0` (1,0) | 12 |
| 30 | Fountain of Fortune | 385 / 115 | 112 / 166 / 107 | 31 | `AVSfntn0` (one tile, blocked) | 11 |
| 31 | Fountain of Youth | 272 / 101 | 82 / 97 / 93 | 19 | `AVXfyth0` (0,0); 266 block four tiles, 6 two | 13 |
| 38 | Idol of Fortune | 563 / 133 | 120 / 276 / 167 | 117 | `AVSidol0` (one tile, blocked) | 15 |
| 48 | Magic Spring | 70 / 39 | 10 / 31 / 29 | 4 | `AVXmags0`, two entrances (1,0), (0,0) | 8 |
| 49 | Magic Well | 915 / 145 | 189 / 408 / 318 | 147 | `AVXwelg0` 312, `AVXwelr0` 523, `AVXwlsn0` 80 (one tile, blocked) | 32 |
| 52 | Mermaids | 133 / 59 | 41 / 62 / 30 | 4 | `AVXmerm0` (1,0) of three | 9 |
| 56 | Oasis | 55 / 37 | 10 / 24 / 21 | 4 | `AVXosis0`, two entrances (1,0), (0,0); 53 block four tiles, 2 two | 4 |
| 64 | Rally Flag | 331 / 91 | 57 / 139 / 135 | 60 | `AVXrlly0` (0,0) of two | 19 |
| 92 | Sirens | 107 / 57 | 34 / 52 / 21 | 4 | `AVXsirn0` (1,0) of three | 9 |
| 94 | Stables | 143 / 65 | 43 / 65 / 35 | 15 | `AVXstbl0` (0,0) of two | 10 |
| 96 | Temple | 409 / 117 | 99 / 206 / 104 | 55 | `AVStmpl0` (1,0) of two | 22 |
| 110 | Watering Hole | 55 / 43 | 13 / 31 / 11 | 7 | `AVXwtrh0`, four entrances (3,0) to (0,0) | 4 |

Together 4 085 objects on 159 maps; none has a body in the file (VCMI's `readGeneric`), one
subclass each (0). The Buoys, Mermaids and Sirens stand on water only; the Magic Springs on rough
ground (68) and dirt (2), the Oases on sand (53) and swamp (2), the Watering Holes on rough ground;
the rest on every land terrain. Three classes have several entrances (48 two, 56 two, 110 four), and
two have one picture under two masks (31: 266 with four blocked tiles, 6 with two; 56: 53 and 2), as
T98a1 found for 42 SoD monoliths. No entrance is shared with another object.

## 2. The garrison in the map file and at load

- **The body** (`docs/formats/h3m.md`'s row "33, 219 garrison"): a u32 owner (0xFF neutral), the
  army (seven stacks: the creature in one byte in RoE, two from AB on; the count in two), from AB
  on a "removable" byte, then 8 unused bytes. VCMI's `readGarrison` reads the same (an owner past
  the eight colours is made neutral with a warning; RoE's garrisons `removableUnits = true`).
- **The loader** (`load_garrison_body` 0x502A00, `load_object_body`'s case for class 33): the
  owner's low byte (the other three skipped); each stack's creature sign-extended (0xFF or 0xFFFF
  is -1, no creature) and its count **sign-extended from 16 bits** (as `army_swap_slots`'
  ADV-QUIRK-19: a count of 32 768 or more becomes negative; no shipped garrison comes near); on an
  RoE map (game +0x1F86C 0xE) removable 1, else the byte (non-zero 1); the entrance's column and row
  (`object_entrance_tile` 0x4FEC30) and the object's level. The record (`garrison_record`, 0x40
  bytes, H3API's `H3Garrison`: owner +0x0, army +0x4, removable +0x3C, entrance +0x3D to +0x3F) is
  appended to the game's vector (+0x4E3A8; begin +0x4E3AC), its index written into the object's
  word, and the 8 unused bytes read and dropped. Random stacks are not resolved here.
- **Class 219** is loaded as class 33 (`object_class_values` 0x63A6E4, as 220 is loaded as 53), so
  every later test (the visit, the pathfinder, the battlefield, `class_is_ownable` 0x40FCA0, the
  owner's colour `tile_object_owner` 0x40FD20) reads 33. Both classes are entered from above
  (`object_classes_from_above` 0x63AB00 lists 33 and 219): a horizontal garrison is entered from
  the three tiles above and the three below; a vertical one from the left and the right column,
  the diagonals included.
- **At a new game** (`setup_map_objects` 0x4C0CC0, case 33 at 0x4C14CA): the record's owner is
  written again through the network's owner message (`garrison_owner_message` 0x4C23E0, kind
  0x41F) and an owned garrison reveals a radius of 3 around its entrance for its owner
  (`reveal_disk` 0x49D160); `reveal_owned` 0x4C7C50 reveals the same at the game's start (read by
  T80, its garrisons' records at +0x4E3AC). The army is not touched.

## 3. The visit (`visit_object` 0x4A84F0, case 33 at 0x4A9061; read whole)

With the hero, the entrance's tile (its word the record's index), the position and the human flag
(the visit's fourth argument):

1. **The visitor's team** (game +0x1F879: the record's owner and the current player both set and of
   one team): straight to the exchange (step 4). So an own garrison and an ally's open the dialog;
   a neutral one (owner -1) never counts as the team's.
2. **Any other garrison holding a creature** (`army_has_creatures` 0x4496A0: a type other than -1 in
   any of the seven slots; the counts are not read): `adventure_battle` 0x4AD470 at once, **with
   no question** (a mine asks first, ADV-MIN-002; a garrison does not). The battle's arguments: the
   position, the hero and its army (+0x91), the record's owner as the defending owner (-1 for a
   neutral one), no town, no defending hero, the record's army, seed -1 (drawn by the battle), and
   the flags 1 and 0 as a dwelling's, a town's or a hero's battle passes them (their meaning not
   read). A result other than 0 (the attacker did not win) ends the visit (§4).
3. **Won, or nothing to fight:** `garrison_set_owner` 0x4C6960 with the current player (§5), then
   `hero_level_up` 0x4DA720 (the battle already gave its levels, `adventure_battle` calls it four
   times; this call does nothing new).
4. **The exchange:** a human's visit `garrison_exchange_dialog` 0x5D1130 (§6); a computer's
   `ai_garrison_visit` 0x524370 (not read, E10's).

So an empty garrison of another team, a neutral one too, is taken by walking in, and a won battle
leaves the hero standing in its new garrison with the exchange open on an empty army.

## 4. The battle

- **The sides:** the attacker the visiting hero; the defender the garrison's army **as it stands,
  unsplit**: case 33 calls `adventure_battle` directly, so a neutral garrison's single stack is not
  fought as a wandering monster (a neutral mine's is, through `monster_battle`, D-47). An owned
  garrison's side is its owner's, with no hero (the engine's `Defender::Garrison`, which an owned
  mine's guards already use: played by its owner outside its turn, batch E9's E9-2); a neutral
  one's is unowned (the engine's `Defender::Guards`).
- **The field:** `battle_setup_battlefield` 0x4643B0 at the position: the tile's terrain (no case
  for garrisons among its terrain overrides), then the special ground `tile_special_ground`
  0x4FCE20, which answers 33 for an **anti-magic garrison's entrance** (class 33, the entrance bit
  +0xD 0x10, subclass 1) **or a hero standing on one** (class 34 on an entrance, the hero's
  stood-on class +0xC 33, the tile's subclass 1): the battle's +0x53C4 is set, and `side_may_use_magic`
  0x41F890 then refuses every cast of either side, the hero's and the creatures'. The battle lane
  built this: BAT-TER-002 (`BattleSetup.anti_magic`), BAT-MAG rules citing 0x41F890; the adventure
  only has to set the flag. The garrison gives no walls, moat or towers (the wiki agrees).
- **The outcome:** the defender's survivors stay in the record (its army is the battle's); a loss
  or a retreat of the attacker (no surrender: no hero defends) ends the visit with the garrison unchanged but its
  losses (the attacker's hero leaves as after any battle, `adventure_battle`'s pool and loss
  effect); a win empties the record and goes on to the capture.

## 5. The capture and a defeat (`garrison_set_owner` 0x4C6960, read whole)

With the record's index and the new owner (-1 for none): the change is sent to the other players
(`map_message_send` 0x482390, kind 0x41F, the record's entrance packed); the owner byte +0x0 is
written; a new owner other than -1 reveals a radius of 3 around the entrance for it (`reveal_disk`,
as a mine's flag: ADV-OWN-001's `flag_radius`). Its callers: case 33 (§3), `player_defeated`
0x4F11A0 with -1 (a defeated player's garrisons become **neutral with their armies**, so a guarded
one then bars every hero), and the network's handler. Nothing else changes: the removable byte
stays the record's, so a non-removable garrison stays non-removable for its new owner.

## 6. The exchange (`garrison_exchange_dialog` 0x5D1130, read whole)

- **The dialog:** the owner passed is the record's, or **-1 when the record is not removable**
  (+0x3C 0); `garrison_exchange_create` 0x5D0E00 builds `garrison_dialog_create` 0x5CE830 (read by
  T44a: the upper bar the garrison's army with that owner and no hero, the lower the hero's army,
  no town) with a title; it runs modally and is destroyed. Moves change the record's army in place.
- **What the player may do** (`garrison_dialog_hover` 0x5D05F0, `town_garrison_hover` 0x5C7400,
  `town_garrison_hover_other` 0x5C8080; T44a read the moves for one owner): a move is offered only
  while the **selected stack's bar belongs to the current player**; else the hover only selects or
  shows (0x5C8080: its status texts, mode 0, or -2 for an empty slot). With a selected stack of the
  current player's bar and a target in a bar of **another owner** (-1 included), only a **move or
  split into an empty slot** is offered (0x5C7609: that path tests no owner); a target holding a
  stack, of the same creature or not, is refused (0x5C7532 and 0x5C7700 test the target bar's
  owner). So:
  - **an own, removable garrison:** both bars the current player's: the army dialog's rules (survey
    `E08-army.md` §1.1, GAM-DEC-004: move, merge, swap, split, dismiss; the hero's last stack never
    leaves its army, ADV-ARM-002); the garrison may be emptied;
  - **an ally's garrison** (owner another colour of the team) **or a non-removable one** (owner -1):
    the hero may put stacks (or a split part) into its empty slots, its last stack excepted (the
    same test, 0x5C7673), never take one out, merge into one or swap with one. Troops given to an ally's garrison are the ally's.
- The wiki's trivia: a removable garrison's first slot shows a flag (the dialog's drawing, not
  read); the wiki's Garrison page says troops may be added to but not removed from a non-removable
  garrison, which the reading confirms and sharpens (empty slots only).

## 7. The passage

- **The path** (`pathfinder_pass_through` 0x56AB40, its entry and case 33 at 0x56ACAE, read): for
  a stop node on a garrison's entrance, the path goes on when the record's owner and the hero's
  (+0x22) are set and of one team, in every mode; else only in the modes from 4 (the computer's,
  E10's). In the human's mouse mode (0) the entry answers no for every class but 33 and 212 before
  its switch (T55a4), so a human's path crosses **its team's garrisons** and ends at any other.
- **The step** (`hero_move_step` 0x4805E0, its switch 0x481544 on the stepped tile's class, case 33
  at 0x48154B, read): when the step's tile is not the hero's planned destination (hero +0x35,
  +0x39, +0x3D, H3API's `dest_x/y/z`), the player is human and the garrison is of its team, the
  step's visit is cleared before the guard test (`after_step_guard_check` 0x480090): **no
  exchange on the way**. The planned destination's garrison is visited (§3); a computer's hero
  visits every garrison it passes (its exchange, E10's). Case 212 (the border gate) clears it the
  same way without an owner test; a list of other classes always (not ours).
- **Standing in it:** the hero stands on the entrance, the tile's class reads 34 while it stands
  there (`tile_special_ground`'s first test); it is met or fought there as any hero (ADV-OWN-001's
  "a hero standing on an owned entrance"), the garrison's army not joining it, and an anti-magic
  garrison's tile forbids the battle's casts (§4). The space key's revisit (ADV-TEL-008) opens the
  exchange again. The wiki: "Heroes cannot occupy a garrison" (a hero does not enter its army, as a
  town's garrison hero does).
- **Not read:** whether anything ends the move after a won battle beyond the visit's end (the
  battle's own move rules, increment 3's).

## 8. What a viewer sees

- **The owner:** the flag's colour (`tile_object_owner` 0x40FD20; the minimap's colour,
  `minimap_draw` 0x412C40), as for every ownable object (ADV-OWN-003).
- **The hint:** the class's name only (`map_hint` 0x40B150 has no case for 33; E24's reading).
- **The right-click** (`garrison_popup` 0x416F80, read whole; from `tile_click_path`'s case 33):
  the level is **3** when the record's owner and the viewer are of one team, with the debug flag
  (the adventure manager's +0x3D), or when one of the viewer's heroes has Visions at level 3 within
  reach of the entrance (`hero_visions_level` 0x4E5DE0, `hero_visions_reach` 0x4E5E10, measured
  from the record's entrance; a lower Visions level counts for nothing); else 2 with two or more
  of the viewer's towns with a Tavern (`player_tavern_count` 0x4CCE30), 1 with one, 0 with none.
  `garrison_quick_view_create` 0x5307D0 shows `townqvbk.pcx`, the class's name (`object_names`
  0x6A79EC, entry 33) and the army at that level (`quick_view_army` 0x530990: 0 nothing, 1 each
  stack's creature packed, 2 also its size word, 3 its count). The town's popup has the same levels
  (survey `E09-towns.md` §12.3), and the engine's `h3_game::GarrisonView` already projects a town's
  garrison by them (TWN-VEW-002, TWN-VEW-003): the map's garrison can use it as it is.
- **The removable flag** shows in the exchange only (the visitor); a neutral garrison is nobody's
  team: every viewer sees it by its taverns or Visions.

## 9. VCMI, the wiki, disputes and the proposed rules (garrisons)

### 9.1 At a glance

| Point | Original (this reading) | VCMI 1.7.5 | Wiki |
|---|---|---|---|
| Classes | 33 horizontal, 219 vertical (made 33); subclass 1 anti-magic | `GARRISON`, `GARRISON2`; subtype `antiMagic` | horizontal RoE, vertical AB on |
| Removable | AB on a byte, RoE always | the same (`removableUnits`) | an editor check box (AB on) |
| Visit of the team | the exchange | `showGarrisonDialog` for an ally or the owner | owner and allies pass |
| Visit of another | a battle when armed, no question; then taken | `startBattle`; `battleFinished` visits again: flagged, then the dialog | enemies must win first |
| Empty, not the team's | taken at once, the exchange | taken (`setOwner`), the dialog | — |
| The battle's army | as it stands, unsplit, the owner's side | the garrison's army as it is | — |
| Anti-magic | every cast of both sides; on the entrance or for a hero standing on it | `BLOCK_ALL_MAGIC` on the garrison, battle-wide when it fights | every cast; also for a hero attacked on the tile; Faerie Dragons and three SoD artifacts excepted |
| The exchange, not removable or an ally's | into empty slots only | into empty slots; merge into an ally's stack of one creature allowed | added, not removed |
| The passage | a human's path crosses its team's garrison, no visit | crossed when empty or not an enemy's; visited on the way | owner and allies pass |
| Capture's reveal | radius 3 to the new owner | an owned object's sight radius 3 | — |

### 9.2 Disputes, quirk candidates and open points

- **D-59 (the passage).** The original: a human's path crosses only a garrison of its team and
  takes no visit there (§7); an empty garrison of another team, or a neutral one, ends the path and
  is taken by the visit. VCMI: `CGGarrison::passableFor` lets any empty garrison and any garrison
  of an ally or the owner be crossed (`PathfindingRules.cpp`: "transit via unguarded garrisons");
  its client ends each movement batch at a garrison and the server visits it on the way
  (`moveHero` with `VISIT_DEST`), so the dialog opens mid-move and an empty enemy garrison is
  flagged in passing. The wiki: owner and allies pass. Gameplay: whether a move through an own
  garrison stops for the dialog, and whether a path may cross an empty neutral garrison.
  Recommended: **the original** (the path crosses its team's garrisons only, no visit on the way).
- **D-60 (a hero attacked in an anti-magic garrison).** The original (`tile_special_ground`) and the
  wiki: the battle forbids casting when the defending hero stands on an anti-magic garrison's tile.
  VCMI puts `BLOCK_ALL_MAGIC` on the garrison object, so only a battle against the garrison's own
  army is without magic (its code carries a TODO on the garrison's effects). Recommended: **the
  original** (the entrance's tile decides).
- **D-61 (merging into another's garrison).** The original refuses a stack moved onto an ally's or a
  non-removable garrison's stack, of its creature too (only empty slots, §6). VCMI's
  `mustForceReselection` allows the merge into an ally's stack of one creature (its non-removable
  case refuses it, as the original). Recommended: **the original**. (Teams are E10's; until then
  only the non-removable case is reachable, where the sources agree.)
- **No quirk candidate.** The signed 16-bit counts follow ADV-QUIRK-19's handling (the import keeps
  the 16 bits whole); the computer's visits on the way and its paths through other garrisons are
  E10's; Faerie Dragons and the SoD artifacts' casts at an anti-magic garrison (the wiki's
  exceptions) are the battle lane's (BAT-TER-002 and its rules).
- **Open point (the packs).** The anti-magic garrison stands on 9 RoE-format objects (6 maps)
  although the wiki dates it from AB, and the executable tests no version: its kind belongs in the
  base (RoE) pack with the horizontal garrison, the vertical kinds (219) in the AB pack (no
  RoE-format map holds one). Recommended as such; T108b's call.
- **Open point (the import).** A random stack (the codes below the format's "none") in a
  garrison is resolved neither by the load nor by the new game's setup (§2; three other users
  of the records' vector not read): recommended unsupported, as for a placed hero
  (`h3m.md`, "Placed heroes"). A creature with a count of 0 counts as a creature for the visit
  (`army_has_creatures` reads the types only): recommended the import's existing army rule (an
  empty slot). No shipped map has either.
- **A gap found (GAM-BAT-011).** The battle background's ownable classes (`h3_map`'s
  `codes::scenery`) hold 33 but not 219, which the original makes 33: the 70 vertical garrisons
  count as other scenery for the picture's variant (a viewer's fact, no rule reads it). To be added
  with 219's kind (T108b), as 220 was for the abandoned mine.

### 9.3 The garrisons' proposed rules (the ids with T108a2's §13)

- **The objects** (ADV-GAR-001, proposed): classes 33 and 219 one ownable, entered kind each (by
  their templates), entered from above; subclass 1 anti-magic; the body's owner (ADV-OWN-002 amended:
  a colour nobody plays refused, as now), army (the import's army rules) and removable flag (RoE
  true); the census's numbers as the import's test.
- **The visit** (ADV-GAR-002): the team's garrison opens the exchange; another armed one is fought
  without a question (the owner's side without a hero, or unowned guards; unsplit); a win or an
  empty one flags it (ADV-OWN-001 amended), then the exchange; a loss leaves the survivors.
- **The exchange** (ADV-GAR-003; GAM-DEC-004 amended): an own removable garrison as the army
  dialog between the hero and the garrison; an ally's or a non-removable one, moves and splits into
  its empty slots only; told and projected as the Leave Guards dialog is.
- **The passage** (ADV-GAR-004; ADV-OBJ-003 and the reach amended): a garrison of the mover's team
  is crossed by a path and a move without a visit; it is visited when it is the move's end; any
  other garrison is a stop.
- **The anti-magic battle** (ADV-GAR-005; GAM-BAT amended): a battle on an anti-magic garrison's
  entrance, the garrison's own or a hero's standing on it, sets `BattleSetup.anti_magic`.
- **What a viewer sees** (ADV-GAR-006; ADV-OWN-003 amended): the owner to every viewer that sees it,
  the army by TWN-VEW-002/-003's levels, the removable flag to the owner's team.
- **A defeat** (ADV-OWN amended): a defeated player's garrisons become neutral with their armies.

### 9.4 For batch E12 (planned at T108a2)

Rows worth an experiment on the client lane's harness: a human's move through its own garrison
with the destination beyond (no dialog, D-59); an empty neutral garrison on a path (the path ends,
the visit takes it); a non-removable own garrison's exchange (a move into an empty slot, a merge
refused, nothing taken); a hero attacked while standing in an anti-magic garrison (no cast, D-60;
needs a second player's hero); the capture's reveal of radius 3; a garrison won with survivors of
the attacker standing in it afterwards.

## 10. The bonus visitables (T108a2)

T108a2's: their gains, durations and marks, the battle's morale and luck, the movement and spell
points, the Stables' Cavaliers, the Sirens' losses and experience, what a viewer sees, VCMI and the
wiki. The census is §1.2.

## 11. Commands, decisions, events, the observation, replays and benchmarks (T108a2)

## 12. Batch E12's plan (T108a2)

## 13. The proposed rule ids (T108a2)

## 14. For T108b: the checkpoint's questions (T108a2)

The garrisons' questions (D-59 to D-61, the packs, the import) are asked here, with the bonus
visitables'.
