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
- **T108a2** (§10 to §14; its readings listed in §10's head): `vcmi` VCMI 1.7.5
  `config/objects/rewardableBonusing.json` (eleven of the fifteen), `magicSpring.json`,
  `magicWell.json`, `generic.json` (`sirens`), `lib/mapObjects/MiscObjects.cpp` (`CGSirens`),
  `lib/mapObjects/CRewardableObject.cpp` (the visit modes), `lib/bonuses/BonusEnum.h`
  (`ONE_WEEK`); `wiki` <https://heroes.thelazy.net/index.php/Morale>,
  <https://heroes.thelazy.net/index.php/Luck>,
  <https://heroes.thelazy.net/index.php/Idol_of_Fortune>,
  <https://heroes.thelazy.net/index.php/Buoy>,
  <https://heroes.thelazy.net/index.php/List_of_adventure_map_objects> (the Magic Spring, the
  Magic Well, the Sirens, the Stables, the Oasis rows); the integer checks of §10.5 (a throwaway
  script over the counts, not committed).

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

### 0.1 Findings in short (T108a2)

- **The battle bonuses** (§10.2, §10.3): eleven objects add morale or luck until the end of the
  hero's next battle, one per kind (the hero's status bits, cleared with the bonuses); the Idol by
  the day (odd luck, even morale, day 7 both), the Temple +2 on day 7, the Fountain of Fortune its
  value (−1, +1, +2, +3), drawn per fountain at the start and each week. The engine's eight
  once-marks need twelve more kinds, and each mark's amounts for the battle's sources.
- **Movement** (§10.4): +800 or +400 to the points left **and** the day's maximum; the Swan Pond's
  points to 0; the Stables +400 now and to every day's land points until the week ends, and the
  Cavaliers upgraded at **every** visit (D-63).
- **Spell points** (§10.4, §10.6): the Well to the maximum once a day (a full hero unmarked); the
  Spring to twice it, **per entrance** once a week (D-62).
- **The Sirens** (§10.5): stacks above 1 keep trunc(count × 0.7) by the double product (90 keeps
  62; VCMI 63, D-64), cut to 16 bits (A-45); the experience is the lost hit points × learning;
  again after every battle (A-44, the wiki's bug).
- **What a viewer sees** (§10.7): the hint's bonus line by a per-player kind mark the engine lacks;
  the marks by the selected hero's bits.
- **The interfaces, batch E12, the ids, the checkpoint:** §11 to §14 (no new command or decision
  for the fifteen; sixteen batch rows; ADV-GAR and ADV-BON).

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

**Read whole** (the human player's path; `human` false skips only the messages, every gain is the
same for a computer): `visit_object`'s inline cases 11 (0x4A88FA), 14 (0x4A8BB1), 28 (0x4A8ED6)
and 52 (0x4A968E); `visit_fountain_of_fortune` 0x4A2480, `visit_fountain_of_youth` 0x4A25F0,
`visit_idol_of_fortune` 0x4A12F0, `visit_magic_spring` 0x4A3590, `visit_magic_well` 0x4A3730,
`visit_oasis` 0x4A3CA0, `visit_rally_flag` 0x4A44C0, `visit_sirens` 0x4A5980 with
`sirens_losses` 0x527EC0, `visit_stables` 0x4A60A0 with `army_count_type` 0x4E2340 and
`army_replace_type` 0x4E2370, `visit_temple` 0x4A6200, `visit_watering_hole` 0x4A7EA0;
`hero_battle_bonuses_clear` 0x4DA510 again (every bit it clears). **Read in part:**
`setup_map_objects` 0x4C0CC0 at its cases 30 (0x4C1479) and 48 (0x4C17C9);
`new_day_week_events` 0x4C8780 at its cases 30 (0x4C8C9F) and 48 (0x4C8B9C) and its heroes' loop
(0x4C8D2A). **Read before** and used here: `hero_morale_description` 0x4DC320 and
`hero_luck_description` 0x4DCAC0 (E17 M6 T3c1: the visits' lines by bit, `E17-design.md` §12.9),
`new_day` 0x4C7FE0 (bits 0 and 17 cleared), `hero_max_land_movement` 0x4E4990 (bit 1),
`hero_intelligence` 0x4E48B0, `hero_learning` 0x4E4840, `hero_gain_experience` 0x4E33B0,
`tile_mark_visited_by_team` 0x4FBF90, `hero_boards_boat` 0x4A1010 and `hero_lands_from_boat`
0x49E670 (the day's maximum), the map hint's marks (`E24-design.md` §4.11). **Not followed:** the
computer's choice to visit (E10), network play, the pictures and sounds (E24's).

### 10.1 What the fifteen share

- **The hero's status bits** (`status_flags` +0x105) say what a hero has visited; each object
  tests its own bits first and, when set, shows its "visited" message and gives nothing. By
  lifetime:
  - **until the end of the hero's next battle** (`hero_battle_bonuses_clear`, which also sets
    morale +0x11A and luck +0x11B to 0): bits 2 to 16, 20, 21 and 25 to 29. Among them the Buoy 2,
    the Swan Pond 3, the Idol 4 (luck) and 25 (morale), the Fountain of Fortune 5, 27, 28 and 29
    (its value −1, +1, +2, +3), the Watering Hole 6, the Oasis 7, the Temple 8 and 26 (day 7), the
    Faerie Ring 13, the Fountain of Youth 14, the Mermaids 15, the Rally Flag 16 and the **Sirens
    20**; the others are the creature banks' (9 to 11), the Pyramid's 12 and the Warrior's Tomb's 21
    (increment 7);
  - **a day**: bit 0, the Magic Well (any Well; `new_day` clears it with bit 17 for every hero);
  - **a week**: bit 1, the Stables (any Stables; the new week clears it for every hero, 0x4C8D2A).
- **Morale and luck** are byte adds to +0x11A and +0x11B (they wrap past 127: A-9's class, which
  the engine holds saturating, HER-ADV-008). One bonus per kind and battle; kinds add up, and the
  battle bounds the sum (BAT-MOR-001, BAT-LCK-001).
- **Movement:** +0x4D the points left, +0x49 the day's maximum (the points the day began with;
  the Admiral's Hat's proportion at a boarding or a landing divides by it, `hero_boards_boat`,
  `hero_lands_from_boat`). The four movement objects add to **both**, in 32 bits, unbounded.
- **The team's kind mark** (`kinds_visited` game +0x4E344 + k: a bit for every player of the
  visitor's team, `player_team`) tells the map hint the kind's bonus (§10.7). Most set it only on
  the gain path; the Magic Spring sets its on every visit, the Magic Well also when the hero is
  full, the Fountain of Fortune its tile's team bits on every visit (the hint reads those, not its
  k); the Stables and the Sirens have none.
- **Messages:** ADVEVENT lines by index (the adventure's text +0x20), none of them read by a rule.

### 10.2 The fifteen, one by one

| Class | Object | Visited when | Gain (else nothing) | Marks set | k | ADVEVENT gain / visited |
|---|---|---|---|---|---|---|
| 11 | Buoy | bit 2 | +1 morale | bit 2, k | 0 | 21 / 22 |
| 14 | Swan Pond | bit 3 | +2 luck; the points left set to 0, the day's maximum kept | bit 3, k | 1 | 29 / 30 |
| 28 | Faerie Ring | bit 13 | +1 luck | bit 13, k | 2 | 49 / 50 |
| 30 | Fountain of Fortune | bit 5, 27, 28 or 29 | luck + the fountain's value (§10.6): −1, +1, +2 or +3 | the value's bit; k; the tile's team bits 5–12 first, every visit | 3 | 55 (picture by sign) / 56 |
| 31 | Fountain of Youth | bit 14 | +1 morale, +400 points left and maximum | bit 14, k | 13 | 57 / 58 |
| 38 | Idol of Fortune | bit 4 or 25 | day 7: +1 morale and +1 luck (bits 4, 25); an odd day (1, 3, 5): +1 luck (bit 4); an even day: +1 morale (bit 25) | as given, k | 22 | 62 / 63 |
| 48 | Magic Spring | the entrance's word bit 6 clear (emptied this week) | spell points below twice the maximum: set to twice it, the entrance's bit 6 cleared; at or above: message 76, nothing | k and the player record's +0xBC bit (word bits 0–4) first, every visit | 15 | 74 / 75 |
| 49 | Magic Well | bit 0 | spell points below the maximum: set to it, bit 0; at or above: message 79, no bit | the tile's word set to 0 (no reader); k when not visited | 25 | 77 / 78 |
| 52 | Mermaids | bit 15 | +1 luck | bit 15, k | 16 | 83 / 82 |
| 56 | Oasis | bit 7 | +1 morale, +800 points left and maximum | bit 7, k | 26 | 95 / 94 |
| 64 | Rally Flag | bit 16 | +1 morale, +1 luck, +400 points left and maximum | bit 16, k | 17 | 111 / 110 |
| 92 | Sirens | bit 20 | the losses and experience (§10.5) | bit 20, also when nothing is lost | — | 132 (the experience) or 134 (nothing lost) / 133 |
| 94 | Stables | bit 1, for the movement only | not visited this week: +400 points left and maximum, then the day's land points +400 until the week ends (§10.4); **every visit**: each Cavalier slot (10) becomes Champions (11) | bit 1 | — | 137 movement, 138 upgrade, 139 both / 136 |
| 96 | Temple | bit 8 or 26 | day 7: +2 morale (bit 26); else +1 (bit 8) | as given, k | 23 | 140 / 141 |
| 110 | Watering Hole | bit 6 | +1 morale, +400 points left and maximum | bit 6, k | 27 | 166 / 167 |

The day is the date's day of the week (game +0x1F63E, 1 to 7; `new_day` steps it). The values come
from the code, not from data: +400 for the Stables is `movement_bonus_stables` 0x698A94
(movement.txt row 6); the Oasis's 800 and the others' 400 are immediates (0x4A3D84, 0x4A26D7,
0x4A45AA, 0x4A7F85). The Swan Pond's 0 is written even when the hero had points left.

### 10.3 The battle's morale and luck

- In battle the hero's own morale and luck (`hero_morale` 0x4E39B0, `hero_luck` 0x4E36C0) carry
  +0x11A and +0x11B; the descriptions list a line per set bit, in their order (`E17-design.md`
  §12.9; the Fountain of Fortune's one line takes its text "−1" to "+3" from
  `fountain_of_fortune_luck_texts` 0x679D08 by the bit). Both sides' heroes are cleared at the
  battle's end (GAM-BAT-014, already built).
- **The engine today** (HER-ADV-008, `h3_heroes::BattleBonuses`): morale and luck held saturating
  and eight once-marks, two taken (the Tomb 0, the Pyramid 1). The fifteen need **twelve more**
  kinds that last until the battle: the Buoy, the Swan Pond, the Faerie Ring, the Fountain of
  Fortune, the Idol, the Mermaids, the Temple, the Oasis, the Watering Hole, the Fountain of Youth,
  the Rally Flag and the Sirens (whose mark gives nothing in battle). So fourteen kinds: more than
  eight (ADR-0013 point 1's struct, the game state hash; T108b's).
- **The client lane's carry** (the inbox of 2026-11-03, CLT-Q40): each visit's own source
  (`EffectSource::Visit { mark }`) needs each mark's amounts. The Fountain's depends on the fountain
  (the week's draw), and the Idol's and the Temple's on the day, so the hero must keep what each
  mark gave (the original keeps the Fountain's in which of four bits it sets), not only the sums.

### 10.4 Movement and spell points

- **The movement objects** (Fountain of Youth, Oasis, Rally Flag, Watering Hole) raise the points
  left and the day's maximum together, so a later boarding or landing with the Admiral's Hat
  divides by the raised maximum (A-28's proportion; the engine's `HeroOnMap::max_movement`). All
  four stand on land; a revisit after a battle gains again (the wiki says so of the Oasis).
- **The Swan Pond** sets the points left to 0 and leaves the maximum.
- **The Stables:** the first visit of a week sets bit 1 and adds 400 to the points left and the
  maximum; while bit 1 holds, `hero_max_land_movement` adds the same 400 to the day's land points
  (after the Logistics rate and the two artifacts, as ADV-MOV-001's note "Stables come later"
  awaited; not at sea, its argument 1). The new week clears bit 1 for every hero. A visit with bit 1
  set adds nothing to the movement but still upgrades: `army_count_type(10)` counts the hero's
  slots of Cavaliers with a count above 0 (+0x91 types, +0xAD counts), and when there is one,
  `army_replace_type(10, 11)` makes every slot of type 10 type 11 (its count kept; no test of the
  count, no price).
- **The Magic Well and the Magic Spring** read the hero's maximum as the hero's own formula:
  trunc(f32(10 × Knowledge clamped to 1–99) × `hero_intelligence`) at the x87's 53 bits
  (HER-MAN-001, the engine's `h3_heroes::max_mana`). The Well sets the spell points (+0x18, 16
  bits) to it when below and marks bit 0; a full hero is told so (79) and keeps no mark, so it may
  drink again the same day after casting. The Spring sets twice it when below; at or above it
  tells 76 and the entrance stays full.
- **The space key** (`Revisit`, increment 11): the entered ones (the Swan Pond, the Faerie Ring,
  the Fountain of Youth, the Magic Spring, the Mermaids, the Oasis, the Rally Flag, the Sirens, the
  Stables, the Temple, the Watering Hole) are visited again by a hero standing on them; the four
  blocked ones (the Buoy, the Fountain of Fortune, the Idol, the Magic Well) are visited from a
  neighbouring tile and are never stood on.

### 10.5 The Sirens

- `sirens_losses` 0x527EC0 walks the seven slots; a slot with a creature (type not −1) and **a
  count above 1** keeps **trunc(f32(count) × 0.7)**, the constant an f64 (0x63E688) and the
  product rounded at the x87's 53 bits, that is the IEEE double product of the two
  (`installation.md` §7.10), then **cut to 16 bits signed** (`movsx eax, ax`, 0x527F03). A lone
  creature is kept, so no slot empties. The loss times the creature's hit points (record +0x4C)
  is summed in 32 bits; the experience is trunc(f32(sum) × `hero_learning`), the battle's form
  (HER-EXP-001). `visit_sirens` gives it with its levels (`hero_gain_experience` with both flags,
  the scenario's cap HER-EXP-002 included) and tells it (132, "%d"), or 134 when it is 0; then bit
  20, either way.
- **Not floor(7 × count / 10):** the double product falls just below an integer for some multiples
  of 10, so 90 keeps 62, 170 keeps 118 (74 892 of the counts up to 3 000 000 differ from the exact
  tenths; the first 90, 170, 180, 330, 340). VCMI drowns ceil(count × 0.3), exact for these: 90
  loses 27 there, 28 here (2 336 of the counts 2 to 100 000 differ; D-64).
- **Bit 20 is cleared at a battle's end** (with the battle bonuses), so a hero may visit the
  Sirens again after any battle; the wiki calls that a bug, which HotA fixed (A-44).
- **16 bits:** a stack of 46 812 or more keeps −32 768 or another wrong count (A-45). The f32 of the
  count is exact below 2^24.

### 10.6 Several entrances, the Spring's state and the Fountain's draw

- **The Magic Spring** (two entrances, (1,0) and (0,0) from the bottom right): at a new game
  `setup_map_objects` (case 48) gives an entrance whose left neighbour is also an entrance of class
  48 that neighbour's word (the right entrance copies the left, which the walk by level, row and
  column meets first); any other gets a counter (bits 0–4, the map's springs numbered mod 32) with
  bit 6 set (full). **Each entrance keeps its own word:** a visit clears bit 6 only on the entrance
  used, so the other still refills that week (the wiki: the two tiles are independent, each once a
  week; D-62). The new week sets bit 6 on every entrance (0x4C8B9C). The counter only feeds the
  player record's +0xBC (the hint's mark); a 33rd spring shares the first one's bit there (hint
  data only, the wiki's map limit 32).
- **The Oasis** (two entrances) and **the Watering Hole** (four) keep no state but the hero's bit:
  every entrance is the same object.
- **The Fountain of Fortune's value** is the tile word's bits 13–16 (signed), drawn at a new game
  (0x4C1479, which also clears the tile's team bits 5–12) and again on every new week (0x4C8C9F,
  the team bits kept): `random_between(0, 3)`, 0 giving −1 and 1 to 3 themselves, so −1, +1, +2
  and +3 each 1 in 4; the draws come in the map walk's order among the other weekly classes (54,
  55, 78, 109, 112). No class-30 object has a body in the file, so a value of 0 or outside −1..3
  (no bit set, the visit repeatable, the luck added anyway) is unreachable.

### 10.7 What a viewer sees

Read by E24 M3 T6a2a (`E24-design.md` §4.11), used here: the map hint shows a kind's bonus line
when the viewing player's team holds the kind's mark (k in §10.2), the Fountain of Fortune's by its
own tile's team bit; it shows "visited" or "not visited" by the selected hero's bits (§10.1), the
Magic Spring's by the player record's bit and the entrance's bit 6 clear. The hero screen's morale
and luck lists show the owner each visit's line (§10.3). Nothing tells another player of a visit.
**The engine** knows a visit's knowledge by object (ADV-VIS-006: `AdventureObservation::known`,
`OwnHeroView::marks`), not by kind: the original's kind marks are a per-player set of kinds the
engine lacks (§13's ADV-BON-009, a checkpoint question).

### 10.8 VCMI and the wiki

| Point | Original (this reading) | VCMI 1.7.5 | Wiki |
|---|---|---|---|
| A visit's limit | one per kind and hero until the next battle (the hero's bit) | `visitMode: bonus`: a bonus from the object's type, `ONE_BATTLE` | until the next battle |
| Fountain of Fortune | −1, +1, +2, +3, 1 in 4 each; drawn per fountain at the start and each week | four rewards of 25 %, reset every 7 days (`resetParameters`) | −1 to +3, changed on a week's first day |
| Idol of Fortune | odd days luck, even days morale, day 7 both | the same (`dayOfWeek` limiters) | the same |
| Temple | +1, day 7 +2 | the same | the same |
| Swan Pond | +2 luck, the points left 0 | `movePercentage: 0` | loses the points left |
| The four movement objects | +800 or +400 to the points left and the day's maximum | `movePoints` (the points only; its maximum computed anew) | until the day's end; the Oasis again after a battle |
| Stables | +400 now and to the land points until the week ends; Cavaliers upgraded at every visit | `ONE_WEEK` land movement +400 (to the week's end); nothing at all while it holds; the upgrade only with the bonus | +400 a day for the rest of the week; Cavaliers upgraded free |
| Magic Well | to the maximum once a day (any Well); a full hero not marked | `bonus` with `ONE_DAY`; below 100 % only | once a turn |
| Magic Spring | to twice the maximum, per entrance, once a week | one object (`visitMode: once`, its visitors reset weekly); below 200 % only | two independent tiles, each once a week; at most 32 |
| Sirens | stacks above 1 keep trunc(count × 0.7) (double product, 16 bits); experience hit points × loss × learning; again after a battle | ceil(count × 0.3) drowned; the same experience; a `ONE_BATTLE` dummy bonus (again after a battle) | 30 % of each stack into experience by the lost health; once per hero, the revisit after a battle a bug fixed in HotA |

### 10.9 Disputes, quirk candidates and open points (the bonus visitables)

- **D-62 (the Magic Spring's entrances).** The original and the wiki: each entrance refills once a
  week on its own, so one hero (or two) may refill twice in a week. VCMI: one object, once a week.
  Gameplay: a second refill a week. Recommended: **the original** (the state per entrance).
- **D-63 (the Stables visited again).** The original upgrades Cavaliers at every visit, the
  movement only at the week's first; VCMI gives nothing while its week bonus holds (no upgrade of
  Cavaliers bought after a first visit). The wiki names both gains, not the case. Recommended:
  **the original**.
- **D-64 (the Sirens' loss).** The original keeps trunc(count × 0.7) by the double product (90
  keeps 62); VCMI drowns ceil(count × 0.3) exactly (90 keeps 63); the wiki says 30 %. Recommended:
  **the original**, through `kernel::num`'s audited double product (ADR-0003), or an exact
  integer form equal to it if one is found; batch E12 shows the 90.
- **A-44 (the Sirens again after a battle;** quirk candidate). The original clears the Sirens' bit
  with the battle bonuses, so a hero visits them again after every battle; VCMI does the same; the
  wiki calls it a bug (HotA fixed it). Repeated, it turns weak stacks into experience. Options: a
  `fix-optional` flag (faithful on, strict-fixed once per hero; recommended), reproduce with no
  flag, or always fixed.
- **A-45 (the Sirens' 16-bit survivors;** quirk candidate). A stack of 46 812 or more keeps a
  negative or wrong count: state-corrupting, so **always fixed** (A-9's class): the survivors
  computed in full width. No shipped map comes near it.
- **No other candidate.** The byte morale and luck are A-9's (already fixed); the Magic Well's
  cleared word has no reader; the Fountain's 0 is unreachable; a 33rd Spring's shared bit is hint
  data.
- **Open point (the movement's maximum).** The original raises the day's maximum with the points
  (the Admiral's Hat's proportion reads it); VCMI's maximum is computed anew. Recommended: the
  original (the engine's `max_movement` raised too); a batch row.
- **Open point (where the day's and the week's marks live).** The Well's bit 0 and the Stables'
  bit 1 are hero flags with their own lifetimes (the day, the week), and the Stables' 400 enters
  the day's land points (`h3_heroes::land_points`, HER-ADV-003): in `h3-heroes`' details beside
  the battle bonuses, or in the adventure's `HeroOnMap` beside `max_movement`. T108b's (ADR-0013
  point 2: a hero-level property is derived once, in `h3-heroes`).
- **Open point (the kind marks).** §10.7: the per-player kinds the hint reads, for the fifteen and
  for the kinds increment 7 built (the Marletto Tower and the rest set theirs too, unmodelled).
- **Open point (the packs).** Every one of the fifteen stands on RoE-format maps (§1.2), so their
  kinds belong in the base pack.

## 11. Commands, decisions, events, the observation, replays and benchmarks (T108a2)

For T108b's drafts and design note (ADR-0015 to -0017, ADR-0013); a sketch, not decided.

- **Commands:** none new. A garrison is visited by `MoveHero`'s last step (ADV-OBJ-003) and
  crossed by a path of its team (D-59); its exchange is the army dialog's own commands (moves,
  splits, the close; GAM-DEC-004). A bonus visitable is visited by the step onto an entrance (the
  entered eleven) or toward a blocked one (the Buoy, the Fountain of Fortune, the Idol, the Magic
  Well: `Visit::Neighbour`), and again by `Revisit` for the entered ones.
- **Decisions:** the garrison's exchange, a pending decision of the visitor's owner like the
  Leave Guards dialog (GAM-DEC-004 amended: an ally's or a non-removable garrison takes moves into
  empty slots only). The fifteen ask nothing.
- **Events:** a garrison's battle as the existing ones (the owner's side without a hero, or
  unowned guards), `ObjectFlagged` and `TilesRevealed` (radius 3) at its capture, the exchange's
  moves as the Leave Guards dialog's; a defeated player's garrisons made neutral (with
  `ObjectFlagged`). The bonuses: `HeroChange::Bonus` (morale and luck with the mark; the mark's
  amounts if T108b keeps them), a movement gain (new: the points left and the maximum; the Swan
  Pond's 0), `HeroChange::SetSpellPoints` (the Well, the Spring), the Sirens' loss per slot (new,
  as `WhirlpoolLoss`) with `HeroChange::GainExperience` and the levels, the Stables' upgrade (new:
  the slots changed), "already visited" or "full" (new, or `ObjectEmpty`'s use widened), a kind
  learned by a team (new, if the kind marks are in: §14). Each new variant breaks `h3-protocol`'s
  build until mapped (the global trap: `cross-lane`, or the server lane first).
- **The observation:** a garrison's owner to every viewer that sees it, its army by the town's
  levels (TWN-VEW-002/-003), its removable flag to the owner's team; the bonuses' marks and
  amounts, the Well's day mark and the Stables' week mark in the owner's view of its heroes
  (`OwnHeroView`); a Spring's emptied entrance to the players who visited it, the Fountain's
  knowledge by object (ADV-VIS-006); the kinds known, if in. No flat encoding (decision 187).
- **State and versions:** the garrisons' records (owner, army, removable) in the adventure state;
  the Fountain's value per object and the Spring's two entrances (weekly state, ADV-WEK's store);
  the twelve marks and their amounts (`BattleBonuses`, ADR-0013: the game state hash); the day's
  and the week's marks (§10.9); the seventeen classes' import (map import version); their kinds
  (adventure ruleset part, perhaps the pack format). Replays: no new command, so the format stays;
  the golden replays' hash lines move with the hashes (`homm3-rerecord-replays`).
- **The env** (the AI lane's): no new action unless the garrison's exchange is a new decision for
  it; the observation's additions are theirs to encode.
- **Benchmarks:** the passage through a team's garrison is in the reach's loop (the hot path:
  `homm3-bench-regression`), measured against `master` right after its first build; a workload
  with garrisons in passages (an XL map, as T106c's). The 4 085 bonus objects stop being inert
  blockers: entrances end paths (ADV-OBJ-003) on 159 shipped maps, so the shipped-map workloads'
  counts move once (re-baselined); a test that uses one of these classes as a plain blocker breaks
  (`TRAPS.md`).

## 12. Batch E12's plan (T108a2)

Increment 12's batch, **batch E12**, runs on the client lane's harness in SoD on Easy, hot seat,
from maps `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E11; its spec
and recipe are tasks of T108b's cut. **Two runs:** **E12a** the garrisons (day 1, two players),
**E12b** the bonus visitables (days 1 to 7 and the next week's first, one player). Rows for the
points resting on the reading alone and for the disputes the batch can tell apart.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E12-1 | §7, D-59 | Red's hero west of its own garrison, the destination beyond it | the path; the dialogs; the tile after | the path through, no dialog; VCMI stops at it |
| E12-2 | §3, §7 | an empty neutral garrison on Red's path | the path's end; the owner after | the path ends on it; Red owns it, the exchange opens |
| E12-3 | §6 | Red's own non-removable garrison with a stack of Pikemen, Red's hero with Pikemen and Archers | the exchange: a move into an empty slot, a merge onto its Pikemen, a take | the move only; the merge and the take refused |
| E12-4 | §4, D-60 | Red's hero standing in an anti-magic garrison of Red's; Blue's hero attacks it | the battle's spell book for both sides | no cast for either (VCMI: casts allowed) |
| E12-5 | §5 | Red's hero wins a neutral garrison | the explored tiles round it | radius 3 revealed |
| E12-6 | §5 | Red's hero wins a garrison with survivors | the hero's tile and army; the garrison's army | the hero on the entrance; the garrison empty, Red's; the exchange |
| E12-7 | §10.5, D-64 | Red's hero (no Learning) with 90 Pikemen, 10 Archers, 1 Griffin enters the Sirens | army after; the experience | 62, 7, 1; (28 × 10 + 3 × 10) × 1 = 310 (VCMI: 63, 7, 1, 300) |
| E12-8 | §10.5, A-44 | after E12-7 the hero wins a battle, then enters the Sirens again | army; the experience; bit 20 | losses again (A-44) |
| E12-9 | §10.6, D-62 | Red's hero (spell points 0) enters a Spring's left entrance, casts a spell, steps onto its right entrance; the next week both | spell points; each entrance's bit 6 | 2 × maximum each time; both refilled at the week (VCMI: the second refused) |
| E12-10 | §10.4, D-63 | Red's hero with no Cavaliers visits the Stables, takes Cavaliers from a second hero, revisits on day 2 | the army; the points; the day's maximum | +400 once; Champions at the revisit, no points (VCMI: Cavaliers kept); day 2's land points +400 |
| E12-11 | §10.2 | Red's heroes visit an Idol on day 1, day 2 and day 7; a Temple on day 6 and day 7 | +0x11A, +0x11B, the bits | luck; morale; both; Temple +1, +2 |
| E12-12 | §10.6 | two Fountains of Fortune; their words on day 1 of weeks 1 to 4 | the values | −1, +1, +2 or +3; changed only at a week's start |
| E12-13 | §10.4 | Red's hero at an Oasis, then a Watering Hole, then a Swan Pond | +0x4D and +0x49 after each | +800 both; +400 both; points 0, maximum kept |
| E12-14 | §10.4 | Red's hero, full, at a Magic Well; casts; the Well again; another Well | the spell points; bit 0 | refused, no mark; refilled; refused (once a day) |
| E12-15 | §10.4, open point | Red's hero with the Admiral's Hat at an Oasis, then boards a boat | the points after boarding | points × sea maximum / (land maximum + 800) |
| E12-16 | §10.7 | Red visits a Buoy, a Faerie Ring, the Mermaids, a Rally Flag and a Fountain of Youth | the hint of a second object of each kind; the marks | the bonus line shown; "visited" for the visitor, "not visited" for another hero |

**Not in the batch:** the computer's visits (E10), network play, A-45 (no army reaches it).
**What the harness lacks** (`clients/capture/`): reads of a hero's +0x105, +0x11A, +0x11B, +0x49,
+0x4D and +0x18 and of a tile's word (the Fountain's value, the Spring's bit 6), the hint's text
(E24's `visits` reads it already), ending turns to a given day. **What the spec lacks**
(`h3-experiment`, battle-owned: `cross-lane`; the writer is this lane's): the garrisons by class
and subclass with an owner, an army and the removable byte, the fifteen classes by template, a
hero's spell points at the start; a second player's hero for E12-4.

## 13. The proposed rule ids (T108a2)

The garrisons' (§9.3): **ADV-GAR-001** to **-006** (the objects, the visit, the exchange, the
passage, the anti-magic battle, what a viewer sees) with ADV-OWN-001 to -003 amended (a defeat).
The bonus visitables' (a family **ADV-BON**):

- **ADV-BON-001** the objects: the fifteen classes, one kind each, entered or visited from a
  neighbour by their templates (the three with several entrances), in the base pack; no body.
- **ADV-BON-002** the battle bonuses: the morale and luck of §10.2, one kind each until the end of
  the hero's next battle, the Idol's and the Temple's by the day of the week; HER-ADV-008 amended
  (twelve more once-marks, each mark's amounts, `EffectSource::Visit`).
- **ADV-BON-003** the Fountain of Fortune's value, drawn per fountain when the game is built and
  each week (ADV-WEK amended).
- **ADV-BON-004** the movement objects and the Swan Pond: the points left and the day's maximum
  (ADV-MOV-002 amended).
- **ADV-BON-005** the Stables: the week's movement (ADV-MOV-001 and HER-ADV-003 amended) and the
  Cavaliers' upgrade at every visit.
- **ADV-BON-006** the Magic Well: the maximum (HER-MAN-001's), once a day per hero, a full hero not
  marked.
- **ADV-BON-007** the Magic Spring: twice the maximum, per entrance, refilled each week.
- **ADV-BON-008** the Sirens: the losses, the experience (HER-EXP-001's form, HER-EXP-002's cap),
  the mark until the next battle (A-44's answer), A-45 fixed.
- **ADV-BON-009** what a viewer sees: the marks to the owner, the knowledge by object
  (ADV-VIS-006 amended) and, if the checkpoint takes them, the kinds a team knows.

Also amended: GAM-BAT-011 (219 among the ownable classes, §9.2), GAM-BAT-014 (the clear holds the
Sirens' mark), ADV-OBJ-003 (the passage through a team's garrison).

## 14. For T108b: the checkpoint's questions (T108a2)

Asked at T108a2's end (`docs/OPEN-QUESTIONS.md`, ADV-Q49; the answers in the epic as decisions
from 299):

1. **D-59 to D-61** (the garrisons): the original in each (recommended): the path crosses only
   its team's garrisons, no visit on the way (D-59); the entrance's tile decides the anti-magic
   battle (D-60); no merge into another's garrison (D-61).
2. **The garrisons' open points:** the anti-magic kinds in the base pack with 33, 219's kinds in
   the AB pack; a random stack refused at the import, a count of 0 an empty slot (recommended).
3. **D-62 to D-64** (the bonus visitables): the original in each (recommended): the Spring per
   entrance; the Stables' upgrade at every visit; the Sirens' double product.
4. **A-44:** a `fix-optional` flag (faithful on: again after a battle; strict-fixed: once per
   hero; recommended), reproduce with no flag, or always fixed. **A-45** always fixed.
5. **The kind marks** (§10.7): in increment 12 for every kind that sets one (recommended: the
   map hint and the server's fold read them), or a stub for E24's map client.
6. **Batch E12** before the build (recommended, as batches E9 to E11), after it, or not at all.

The open points left to T108b's calls: where the day's and the week's marks live (§10.9), the
movement's maximum raised with the points (the original), the marks' amounts' shape.

**Answered** (2026-11-11, `AskUserQuestion`, two rounds; the epic's decisions 299–303; ADV-Q49):
each as recommended. (1) D-59 to D-61 **the original**; (2) the anti-magic kinds in the base pack,
219's in the AB pack, a random stack refused at the import, a count of 0 an empty slot; (3) D-62
to D-64 **the original**; (4) A-44 **a `fix-optional` flag** (faithful again after a battle,
strict-fixed once per hero), A-45 always fixed (state-corrupting, no question); (5) **the kind
marks in increment 12**, for every kind that sets one; (6) **batch E12 before the build**.
