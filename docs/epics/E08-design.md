# E08 design note

Increment 12's note: garrisons and the bonus visitables (E08 decision 295). Increment 11's note
(the teleporters) is in `E08-log.md` ("Increment 11's design note at its closure", moved by T107),
increments 9's and 10's beside it (moved by T87c and T97c); the code's comments cite these notes as
"increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 12's
amendments, drafted in the lane's `adr-drafts/` by T108b and applied to ADR-0013, ADR-0016 and
ADR-0017 by T112 with batch E12's outcome (the drafts then deleted); written from T108a's survey and the answers at ADV-Q48 and
ADV-Q49 (the epic's decisions 295, 299–303). A building task adds an *As built* line to the section
it builds; the note moves to `E08-log.md` when the increment closes. Facts and sources: survey
`docs/rules/surveys/E08-garrisons-bonuses.md` (§ numbers are the survey's unless they name this
note); the proposed rule ids are the survey's §9.3 and §13.

Written whole by T108b (decision 304): **part A** the content (`h3-adventure`'s kinds,
`h3-pack-adventure`, the packs), **part B** the map (`h3-map`, `h3-adventure`), **part C** the
visit marks on the hero on the map and the hero model's inputs (`h3-adventure`, `h3-heroes`),
**part D** the game, the env, batch E12 and the cut (`h3-game`, `h3-sim`'s replay, the gates).
ADR-0015 needs no amendment (no crate, no import draw). *Agent's call (change freely)* marks a
choice the user could change; **the checkpoint** (ADV-Q50, T108b; decisions 305–309) accepted the
note and the drafts with two changes, written in: the battle bonuses' once-marks a short dynamic
list, not a fixed set (306), and every visit's effect on the hero **on the map**, the battle
bonuses moved there from the details (307; part C).

**The checkpoints' answers** this note builds on: the content as the sketch's item 12 (decision
295); the garrisons as the original (299: D-59 the path crosses only its team's garrisons, with no
visit on the way; D-60 the entrance's tile decides the battle without spells; D-61 no merge into
an ally's or a non-removable garrison's stack; the anti-magic kinds in the base pack, 219's in the
AB pack; a random stack refused at the import, a count of 0 an empty slot); the bonus visitables as
the original (300: D-62 the Spring per entrance, D-63 the Stables' upgrade at every visit, D-64 the
Sirens' double product); A-44 a `fix-optional` flag and A-45 always fixed (301); the kind marks in
this increment (302); batch E12 before the build (303). And ADV-Q50's (decisions 305–309): the
exchange a decision with a direct army reference and one owner rule (305, part D §13); the
once-marks a short dynamic list with the seam's `u8` mark (306); the battle bonuses and the day's
and the week's marks on the hero on the map, the hero model taking them as inputs (307); A-44's
fix one Sirens mark kept through battles (308); the cut and every other agent's call as written
(309).

##### Part A — the content (the adventure part, its packs)

###### 1. The garrison kind (`h3-adventure`'s `ObjectKind`; ADV-GAR-001)

- **`garrison { anti_magic }`**: ownable, entered, **visitable from above** (the original's
  `object_classes_from_above` 0x63AB00 lists 33 and 219, survey §2: so a horizontal one is entered
  from the three tiles above and the three below, a vertical one from its left and right columns;
  the template's own blocked tiles close the rest). Four types: 33.0 and 33.1 (horizontal), 219.0
  and 219.1 (vertical), `anti_magic` for subclass 1. Their masks are the map's templates', as
  every kind's (one template per class and subclass, one entrance, §1.1).
- **Not weekly, no draw, no parameter beyond the flag.** What a garrison holds (its army, its
  removable flag) is the map's (§2 of this note's part B), its owner the ownable objects' block.
- The battle background's ownable classes (`h3_map`'s `codes::scenery`, GAM-BAT-011) gain **219**,
  which the original makes 33 (survey §9.2's gap), as 220 joined for the abandoned mine.

###### 2. The bonus kinds (ADV-BON-001; survey §10.2)

Six kinds for the fifteen classes, none ownable or guarded by the kind, each one type (subclass 0,
no body in the file; the templates' masks as the map's). Entered or visited from a neighbour by
the class (the original's `object_classes_not_entered` 0x63AA58, `E08-objects.md` §2.1): **visited
from a neighbour** the Buoy (11), the Mermaids (52) and the Sirens (92); **entered** the other
twelve, the Fountain of Fortune (30), the Idol of Fortune (38) and the Magic Well (49) among them.
Visitable from above (`object_classes_from_above` 0x63AB00) the Buoy alone. *Corrected by T111*
(batch E12, survey §15; ADV-Q51, decision 317): first written from the templates' blocked tile
(survey §1.2), which says nothing of the mode; the hero stood on the Wells and the Idol and visited
the Sirens, the Buoy and the Mermaids from the neighbouring water.

- **`bonus { mark, days, points, drain, kind_mark }`** (ten classes: the Buoy, the Swan Pond, the
  Faerie Ring, the Fountain of Youth, the Idol of Fortune, the Mermaids, the Oasis, the Rally Flag,
  the Temple, the Watering Hole): `mark` the battle bonus's once-mark (part C §12); `days` the
  morale and the luck by the day of the week (1 to 7), seven pairs; `points` the movement added to
  the points left **and** the day's maximum (0 none: the Oasis 800, the Fountain of Youth, the
  Rally Flag and the Watering Hole 400); `drain` the points left set to 0 (the Swan Pond);
  `kind_mark` whether its visit sets the team's kind mark (§3's list). The pack writes a constant
  pair once (`morale = 1`), the Idol and the Temple their week (`days = [[0, 1], [1, 0], …]`: the
  Idol odd days luck, even days morale, day 7 both; the Temple +1 on days 1 to 6, +2 on day 7).
  *Agent's call:* one kind with a week of amounts, not three kinds (a constant bonus, the Idol, the
  Temple): the original's handlers differ only in these numbers, and a by-day pair is data.
- **`fountain_of_fortune { mark, values }`** (30): the luck its visit adds is the fountain's
  value, drawn among `values` (−1, 1, 2, 3; equal chances) when the game is built and at each
  week's start (§5, §8.2; ADV-BON-003).
- **`magic_well { day_mark }`** (49): the spell points to the hero's maximum, once a day per hero
  (any Well): the day mark (part C §12.4).
- **`magic_spring`** (48): the spell points to twice the maximum, **per entrance**, once a week
  (D-62): no mark, the entrances' state (§5). Two entrances, (1,0) and (0,0) from the anchor.
- **`stables { week_mark, points, from, to }`** (94): at the first visit of a week `points`
  (400; `movement_bonus_stables` 0x698A94, movement.txt row 6) to the points left and the day's
  maximum and to every day's land points until the week ends (the week mark with its land points,
  part C §12.4); at **every** visit every slot of `from` (Cavaliers) becomes `to` (Champions) when
  the army holds `from` with a count above 0 (D-63). `from` and `to` are base-pack creatures.
- **`sirens { mark, keep }`** (92): each stack above one creature keeps `keep` of its count (the
  original's 0.7 as the ratio 7 / 10, §9 of this note), the experience the lost hit points × the
  Learning factor; its mark lasts until the next battle under `faithful` (A-44, `ADV-QUIRK-46`).
- **Not entered from water and never on it** but the Buoy, the Mermaids and the Sirens, which
  stand on water only (§1.2) and are reached only from a boat (ADV-PIK-018's passage, D-46), each
  visited from a neighbouring water tile (ADV-SEA-005: the step's price paid, the hero staying in
  its boat; the Buoy from any of the eight, the Mermaids and the Sirens not from the three above).
- *Agent's call:* the values (+800, +400, 0.7, the Fountain's four) are the kinds' parameters, not
  constants in code; the original holds them as immediates (§10.2's last paragraph) but a pack may
  change them, as the Whirlpool's divisor (increment 11's note §1).

###### 3. The packs and the kind marks' list

- **The base (RoE) pack:** garrisons 33.0 and 33.1 (the anti-magic one stands on 9 RoE-format
  objects, decision 299) and the fifteen bonus classes (all stand on RoE-format maps, §1.2, §10.9);
  **the AB pack:** 219.0 and 219.1. SoD adds none.
- **Names and pictures:** the types' names from `ObjNames.txt` (the harvest's check, as T103a's
  against both `Objects.txt` and the executable's class lists); the templates stay the map's.
- **The kind marks** (decision 302; ADV-BON-009): a type flag `kind_mark` in the part for every
  type whose visit sets the original's `kinds_visited` (game +0x4E344 + k, `structs.toml`): the
  twelve of the fifteen (all but the Fountain of Fortune, the Stables and the Sirens; the Fountain
  marks itself per object, §8.2) and increment 7's built kinds that set one (the Garden of
  Revelation 32, the Learning Stone 100, the Library 41, the Marletto Tower 23, the Mercenary Camp
  51, the Schools of Magic 47 and War 107, the Star Axis 61, the Witch Hut 113, the Tree of
  Knowledge 102, the three Shrines 88 to 90, the University 104). The Hill Fort (35, k 14) is not
  built (not in increment 12). *Agent's call:* the flag on the type (each original kind is one
  class here, one type each); a second type of one class would share its class's mark by its
  own entry, which no pack has.

##### Part B — the map (`h3-map`, `h3-adventure`)

###### 4. The import (`h3-map`; map import v22)

- **Garrisons (33, 219):** the body (`docs/formats/h3m.md`'s row "33, 219 garrison"): the owner
  (ADV-OWN-002: a colour the map does not play refused, as now; 0xFF neutral), the seven stacks by
  the import's army rules (a count of 0 an empty slot; the count's 16 bits read whole, as
  ADV-QUIRK-19's handling: the original sign-extends, no shipped garrison comes near), the
  removable byte from AB on (non-zero true), true on an RoE map; the 8 unused bytes dropped. **A
  random stack** (a creature code below the format's "none" in the random range) is refused,
  `ImportError::RandomStack` (decision 299: the original resolves none, survey §9.2), as a placed
  hero's. Class 219 maps to its own types (not to 33's: the AB pack holds them).
- **The fifteen:** typed by class (subclass 0), no body (VCMI's `readGeneric`); a Buoy, Mermaids or
  Sirens off water, or another of the fifteen on water, is imported as the map has it (the editor
  allows neither; the passage decides who reaches it).
- `MAP_IMPORT_VERSION` 22 (E09 T11 took 20, T103b 21). The census's numbers (§1) are the import's
  test on the shipped maps (`--with-original-data`).

###### 5. The state (`h3-adventure`; adventure state hash v22)

- **A garrison's army** is written on its object when the game is built (`army_left`, the
  persistent table, as a town's garrison since E09 T8a1: `Holds::Garrison { army, removable }` in
  the object's setup, checked by the army rules; an empty army written empty), its owner in the
  ownable objects' block (`owners`, ADV-OWN-002), its **removable flag** static: the object's
  (`garrison_set_owner` keeps it, survey §5), so the static map's, which its hash covers. No new
  per-game block for garrisons.
- **The Fountains' values**, one signed value per fountain by its place among the static map's
  fountains (a new list, in object order), a block `fortunes` of the adventure state: none for a
  map without one, else drawn when the game is built and at each week's step (§8.2), one block the
  forks share, the week's step making a new one.
- **The Springs' entrances**: one "full" bit per entrance of every Spring (the static map's
  springs list, each entrance by its tile, in object order then the template's walk), a block
  `springs` that stands **only while some entrance is emptied**: none when the game is built and
  after each week's step (every entrance full), as increment 10's dwellings' block (note §6
  there): a visit that empties one makes it, the forks share it. *Agent's call:* by entrance tile,
  not the original's counter mod 32 (the counter only feeds the hint's per-player mark, survey
  §10.6, and a 33rd Spring's shared bit is hint data: not reproduced).
- **The kind marks** (decision 302): in the per-player block `Flags` (ADR-0016 point 1, the
  Cartographer types bought and the keys), a list of the types marked, each with its player bits
  (the original's byte per kind), set for every player of the visitor's team (the visitor alone
  until E10's teams, `economy::allied`). None until a first mark, as now.
- **The visit marks are the hero on the map's** (part C; decision 307): the battle bonuses (the
  unmarked morale and luck and the once-marks with their amounts, moved from the details) and the
  day's and the week's marks, on `HeroOnMap`, dropped when the hero leaves the map (the user: no
  visit's effect carries over to a hero that leaves the map and is hired again).
- **The day of the week** is the game's (its calendar): `AdventureState::apply` (and the revisit,
  which runs through it) takes it as an input beside the visitor's stream (ADR-0017 point 1; the
  Idol's and the Temple's amounts, §8.1). *Agent's call:* an input, not a copy in the adventure's
  state (one fact, one place).
- The hash (v22): each hero on the map's battle bonuses and timed marks (part C), the fortunes,
  the springs' block (each entrance as it stands, so no block and a block of full entrances hash
  alike), the kind marks.

###### 6. The garrison's visit, battle and capture (ADV-GAR-002; survey §3 to §5)

The step onto the entrance (entered, so the move ends there, ADV-OBJ-003; a guard beside it is met
first, decision 279) visits it with the hero's owner:

1. **Its team's garrison** (owned by the visitor's team; a neutral one is nobody's): the exchange
   (§7 of part D; `AdventureOutcome::exchange`).
2. **Another's or a neutral one holding a creature** (any slot with a creature; the count not
   read, survey §3: a count of 0 is an empty slot at the import, so unreachable): a battle at once,
   **no question** (`AdventureOutcome::battle`): the defender the garrison's army as it stands,
   **unsplit**, its owner's side without a hero for an owned one (`Defender::Garrison`, as an owned
   mine's guards: its owner plays it outside its turn, batch E9's E9-2) or unowned guards for a
   neutral one (`Defender::Guards`); fought on the entrance (its terrain, no special field), with
   `BattleStart::anti_magic` for an anti-magic garrison (part D §14).
3. **Won, or nothing to fight:** the garrison's army emptied (a win leaves none), **flagged** for
   the visitor's owner (ADV-OWN-001 amended: `ObjectFlagged`, the flag's reveal of `flag_radius`, 3,
   around the entrance), then the exchange (§7 of part D). After a battle the flag is the battle's
   end's (as a mine's, `end_mine_battle`), and the exchange is a **visit's rest** given once the
   winner's levels are answered (the original's `hero_level_up` call before the dialog, survey §3
   step 3; part D §13, GAM-DEC-007 amended); nothing when the winner left the map.
4. **Lost or fled:** the garrison keeps its survivors (written back as a mine's guards), its owner
   unchanged; the attacker's end as any battle's.

A garrison's army changes only by its battles and its exchange. **A defeat** (a player defeated
makes its garrisons neutral with their armies, `player_defeated` 0x4F11A0, survey §5) is E10's: no
defeat is built; ADV-GAR-002's text names it for E10.

###### 7. The passage and the reach (ADV-GAR-004, ADV-OBJ-003 amended; D-59 as the original)

- **A garrison of the mover's team is crossed** by a path and a move without a visit, as a border
  gate by its key's holders (ADV-VIS-005): `visit_at` with the mover's owner answers `pass` for a
  garrison whose owner is of the owner's team, so the reach treats its entrance as a tile like any
  other, on the way or at the end, and the search goes on from it in every direction (the kind is
  visitable from above, §1). `MoveHero`'s steps over it take no visit; **at the move's end** it is
  visited (the exchange), as the original's planned destination (survey §7, `hero_move_step`'s case
  33). Any other garrison (another team's, a neutral one, empty or not) is a destination: the
  search reaches it and goes no further, the move ends there and visits it.
- **Every player's path as a human's** (the original's mouse mode; the engine plays every seat so:
  the computer's paths through other garrisons and its visits on the way are E10's).
- **Standing in it:** a hero standing on an entrance is met or fought there as any hero (a tile
  holding a hero is a destination, GAM-BAT-004, GAM-MET-001; the garrison's army does not join it);
  it leaves the entrance in any direction the kind allows; `Revisit` (ADV-TEL-008) visits the
  garrison again (the exchange for its team's).
- **The hot path:** the test is in the reach's visitable branch only (`visit_at`, reached for a
  tile an object covers), one owner read for a garrison's entrance; the walking loop is unchanged.
  Measured against `master` right after the first build (`homm3-bench-regression`; part D §16).

###### 8. The bonus visitables' visits (ADV-BON-002 to -007; survey §10.1 to §10.6)

Every visit reads the hero's marks first (part C) and, when its mark is set, gives nothing and
tells so (`NothingGained`, `why: Visited`; the original's "visited" message). Each kind's visit,
after the kind mark (§3) where the survey's table §10.2 sets it on every visit:

1. **`bonus`**: the day's pair (§5's input) on the hero's once-mark (part C: the mark recorded
   with its amounts); then `points` to the hero's points left and its day's
   maximum (saturating; the original's 32 bits unbounded), or with `drain` the points left set to
   0 and the maximum kept (`MovementChanged`); then the kind mark.
2. **`fountain_of_fortune`**: first the fountain's knowledge for the visitor's team on every visit
   (`Visits::known`, ADV-VIS-006: the original's tile team bits, which the hint reads); then,
   unmarked, the luck its value gives, on its once-mark. **The value** is drawn
   uniformly among `values` from the setup stream in the build's first-week walk (with the weekly
   objects and the camps, in object order) and from the game's stream in each week's step (the
   weekly pass, in object order; survey §10.6: among classes 54, 55, 78, 109, 112); a value of 0 is
   impossible (the part requires `values` non-zero, distinct, at least one).
3. **`magic_well`**: unmarked (the day mark), with spell points below the hero's maximum
   (`h3_heroes::max_mana`, HER-MAN-001): `HeroChange::SetSpellPoints(maximum)` and the day mark
   (its type, part C); at or above: `NothingGained { why: Full }`, **no mark** (so the hero may drink again that day after
   casting); the kind mark when not marked before (§10.2).
4. **`magic_spring`**: the kind mark on every visit; the entrance's state (§5): emptied this week
   `NothingGained { why: Emptied }`; full, with spell points below twice the maximum:
   `SetSpellPoints(2 × maximum)` (saturating at the 16 bits) and the entrance emptied; at or above:
   `NothingGained { why: Full }`, the entrance stays full. The other entrance keeps its own state
   (D-62).
5. **`stables`**: the week mark clear: `points` to the points left and the maximum
   (`MovementChanged`) and the week mark (its type, part C); the week mark set:
   no movement. Then **every visit**: when a slot holds `from` with a count above 0, every slot of
   `from` becomes `to`, its count kept (`CreaturesUpgraded`, a slot each). No price.
6. **`sirens`**: §9.

Points and the day's maximum are `HeroOnMap`'s (ADR-0016 point 3), so a later boarding or landing
with the Admiral's Hat divides by the raised maximum (A-28's proportion; batch E12's E12-15).
**`Revisit`** visits an entered one again (ADV-TEL-008; its rules unchanged: no step, no points):
the twelve entered, the Wells and the Idol too (batch E12's E12-14, E12-11); the Buoy, the Mermaids
and the Sirens, visited from a neighbour, are never stood on (§2, decision 317).

###### 9. The Sirens (ADV-BON-008; survey §10.5; D-64, A-44, A-45)

- **The losses**, slot by slot in slot order: a stack of more than one creature keeps
  **trunc(count × keep)** computed as the original's double product: `kernel::num::Real` of the
  count times `Real` 7 ÷ `Real` 10 (the correctly rounded quotient is the double nearest 0.7, the
  original's constant 0x63E688), truncated (`Real::trunc_to_i64`); no 16-bit cut (A-45 fixed: the
  survivors in full width, `ADV-QUIRK-47` recorded as fixed); a lone creature is kept. *Agent's
  call:* `Real` (ADR-0003's audited surface) rather than an exact integer form: none was found that
  equals the double product (74 892 counts below 3 000 000 differ from the exact tenths, §10.5),
  and an integer emulation of the rounding would be a second float model. The count fits a `Real`
  exactly (below 2^53; the stack's most is 1 000 000).
- **The experience:** the lost creatures' hit points summed (64-bit, saturating to `u32`) and
  given through `h3_heroes::gained_experience` (HER-EXP-001's form with Learning: the battle's,
  survey §10.5), with its levels (`give_levels`, HER-EXP-002's cap); told
  (`SirensLoss` per slot that lost, then the hero's change), or nothing lost (no `SirensLoss`, the
  experience 0: the original's message 134).
- **The mark** is set either way. **A-44, `ADV-QUIRK-46`** (`fix-optional`, decision 301): on
  (`faithful`) the mark is a battle bonus's and cleared at the next battle's end, so the hero visits
  any Sirens again after each battle (the original, VCMI); off (`strict-fixed`) the battle's end
  keeps it, so a hero visits Sirens once (the wiki's bug fixed). **Decided** (308): the fix keeps the
  original's one mark for every Sirens (a hero that visited one never visits another), the
  smallest change from the original's bit; not one mark per Sirens object (as HotA's per-object
  rule may be, unread). The game's clear keeps the marks the part names as outliving a battle
  under the profile (part C §12.3; decision 308).

###### 10. Information (ADV-GAR-006, ADV-BON-009; ADR-0016 point 6)

- **A garrison:** its type (the anti-magic flag in it: the picture) and owner (the flag) to every
  viewer that sees it (ADV-OWN-003); **its army at the viewer's level** (survey §8:
  `garrison_popup` 0x416F80): 3 for the owner's team or a viewer one of whose heroes has Visions at
  level 3 within reach of the entrance, else the viewer's towns with a standing Tavern, at most 2,
  shown as the town's garrison by TWN-VEW-002/-003 (`h3_game::GarrisonView`; the game's view, part
  D §15); its removable flag to the owner's team only (the original shows it in the exchange, to
  the visitor); a neutral garrison by the viewer's taverns or Visions alone.
- **The bonus visitables:** their type to every viewer that sees them; **the kind marks** the
  viewer's team holds (an observation's list of types: the hint's bonus line); the Fountain of
  Fortune's knowledge by object (`known`); **never** a Fountain's value (the visitor learns it as
  its hero's bonus) nor a Spring's entrances' state (increment 9's precedent: a weekly object's
  contents never show, `ObjectView::visited` says "visited once"; a Spring's `known` says the
  same). The hero's marks and their amounts, its day's and week's marks: in the owner's view of
  its hero on the map (`OwnHeroView`).
- **Events** (each new variant breaks `h3-protocol`'s build until mapped: the global trap):
  `GarrisonChanged` and `GarrisonStackSeen` (part D §15) to every viewer that sees the garrison,
  at every change of its army or its viewer's level, as the town's `TownChanged` and
  `TownStackSeen`; `MovementChanged { hero, movement, max_movement }` (owner alone; a part of
  `SRV-STUB-2` lifted for these visits); `SirensLoss { hero, slot, creature, count }` and
  `CreaturesUpgraded { hero, slot, from, to }` (owner alone, as `WhirlpoolLoss`); `NothingGained
  { hero, object_type, anchor, why }` (`Visited`, `Full`, `Emptied`; owner alone); `KindKnown {
  object_type }` (to each player of the team that did not hold the mark, as `ObjectMarked`).
  *Agent's call:* one `NothingGained` with a reason rather than `ObjectEmpty` widened (the client
  draws three different messages).
- `docs/06-security.md`'s lines (the round's `shared/` task): a garrison's army by the levels; a
  Fountain's value and a Spring's state hidden; the kind marks the team's.

###### 11. Ids, quirks and seams (parts A and B)

Rules: **ADV-GAR-001** (the objects, §1, §4), **-002** (the visit, the battle, the capture, §6),
**-003** (the exchange, part D §13), **-004** (the passage, §7), **-005** (the anti-magic battle,
part D §14), **-006** (what a viewer sees, §10); **ADV-BON-001** (the objects, §2, §4), **-002**
(the battle bonuses, §8.1), **-003** (the Fountain's value, §8.2), **-004** (the movement objects,
§8.1), **-005** (the Stables, §8.5), **-006** (the Well, §8.3), **-007** (the Spring, §8.4), **-008**
(the Sirens, §9), **-009** (what a viewer sees, the kind marks, §3, §10). Amended: ADV-OWN-001 to
-003, ADV-OBJ-003, ADV-VIS-006, ADV-MOV-002 (the maximum raised), ADV-WEK (the Fountain's draws in
the week's pass), GAM-BAT-011 (219). Quirks: **`ADV-QUIRK-46`** (A-44, `fix-optional`, on in
`faithful`), **`ADV-QUIRK-47`** (A-45, fixed in every profile, no flag: the ledger records it as
A-9's are). No stub: nothing in the increment's code is left incomplete (the defeat's neutral
garrisons and the computer's garrison visits are E10's, named in their rules). No seam changes:
`h3-contracts`' `EffectSource::Visit { mark }` and `BattleSetup.anti_magic` exist.

##### Part C — the visit marks on the hero on the map, and the hero model's inputs (ADR-0013, ADR-0016 point 3)

###### 12. The battle bonuses and the timed marks on `HeroOnMap` (HER-ADV-008 amended; decisions 306, 307)

1. **Where** (decision 307, the user at ADV-Q50 (3) and (6)): **every visit's effect lives on the
   hero on the map** (`HeroOnMap`, ADR-0016 point 3), not in the details: no visit's effect carries
   over to a hero that leaves the map and is hired again. So increment 7's battle bonuses
   (`h3_heroes::BattleBonuses` in `HeroDetails::bonuses`, ADR-0013 point 1) **move** to the hero on
   the map with increment 12's, and `h3-heroes` takes them as **inputs** where it derives from
   them. Every way off the map built today is a battle's end, which clears them anyway; dismissal,
   a tavern's rehire (E09) and the garrison hero (E09's increment 3) then start a hero without
   them. *Not read:* whether the original's rehire clears the status bits (the user's call stands).
2. **The battle bonuses** (`HeroOnMap::bonuses`, an adventure type; decision 306): the morale and
   the luck no once-mark names (a Pandora's Box's, an event's: the battle's "temporary" sources),
   each held saturating as now (A-9's class); and **a short dynamic list of once-marks**, each
   entry the mark and the morale and luck its visit gave, sorted by mark and unique, **no fixed
   count** (the user: content beyond SoD may add visitables; a hero visits few before a battle).
   The mark is a `u8` named by the content (decision 306: the seam's
   `EffectSource::Visit { mark: u8 }` unchanged, at most 256 kinds of once-mark across all content);
   the Tomb's 0 and the Pyramid's 1 kept, increment 12's twelve 2 to 13 in the original's bit order
   (the Buoy 2, the Swan Pond 3, the Idol 4, the Fountain of Fortune 5, the Watering Hole 6, the
   Oasis 7, the Temple 8, the Faerie Ring 9, the Fountain of Youth 10, the Mermaids 11, the Rally
   Flag 12, the Sirens 13: the original's 2 to 8, 13 to 16, 20). The amounts are kept per mark
   (survey §10.3: the Fountain's value is the week's and the Idol's and the Temple's the day's, so
   content cannot give them back); the original's several bits per kind (the Fountain's four, the
   Idol's and the Temple's two) encode an amount the entry keeps.
3. **The changes** are the adventure's own, as a hero's points are: a visit adds its pair on its
   mark only while the mark is not in the list (else nothing, `NothingGained`), the reward record's
   bonus half (Pandora's Box, events) adds to the unmarked sums (ADR-0013 point 15.2's order kept:
   the bonuses after the spell points, before the artifacts); a battle's end clears both heroes'
   bonuses but the marks the part names as outliving a battle under the profile (A-44's flag off:
   the Sirens' mark; decision 308). `h3-heroes`' `HeroChange::Bonus` and `ClearBonuses` leave
   (their callers, increment 7's Warrior's Tomb, Pyramid, Pandora's Box and events, move to the
   adventure's change), and `HeroDetails::bonuses` with them.
4. **The timed marks** (`HeroOnMap::timed`): the object types whose day's or week's mark the hero
   holds, sorted (the Magic Well's type, a day; the Stables', a week: each kind names its lifetime,
   decision 307: by type, no numbering). The adventure's **day start** drops every hero's day
   marks, its **week start** every hero's week marks (before the turn's points). **The week's land
   points** are derived, not stored: the `points` of every week-marked type's kind (the Stables'
   400).
5. **`derive_battle_hero`** (ADR-0013 point 15.1 amended) takes the hero's battle bonuses as an
   explicit input from the game (beside the details): the unmarked morale and luck as the
   temporary source, as now; each once-mark's non-zero morale and luck as
   `EffectSource::Visit { mark }` (the client lane's CLT-Q40 carry, its inbox of 2026-11-03: the
   original's +0x105 lines); the battle sums and bounds them (BAT-MOR-001, BAT-LCK-001). A mark of
   no amount (the Sirens') gives no effect. The input is `h3-heroes`' plain type (the unmarked pair
   and a slice of (mark, morale, luck)), so `h3-heroes` still knows no object and no adventure type.
6. **`land_points`** (HER-ADV-003 amended) takes **the week's land points** as an input, added
   after the growth and the worn artifacts, saturating (`hero_max_land_movement` 0x4E4990 adds the
   Stables' 400 last while bit 1 holds, survey §10.4), as `sea_points` takes the Lighthouses'
   (HER-ADV-009's input); on land only.
7. **The hashes:** the game state hash (v28) loses the details' bonuses; the adventure state hash
   (v22) gains each hero on the map's bonuses and timed marks. The owner sees them in its
   `OwnHeroView` (ADR-0016 point 6), no longer in the details' view.

##### Part D — the game, the env, batch E12 and the cut (`h3-game`)

###### 13. The exchange: a decision, the army reference and the one owner rule (ADV-GAR-003, GAM-DEC-009, GAM-MAN-001 amended)

- **A new decision kind, `Exchange`** (GAM-DEC-009): the visiting hero and the garrison, its owner's
  alone, holding no one else, blocking its owner's `EndTurn` like a meeting, closed by `Close`
  (the army dialog's answer). While it stands the owner's `Manage::Army` commands may name
  **the hero's army and the garrison's** (a new `Together::Exchange`, as a meeting's two heroes and
  an army dialog's hero and group, GAM-MAN-001). The commands change the state at once (no copy, so
  nothing is written back at the close); the close only closes. It opens at the visit (§6 of part
  B, a team's garrison, or one just taken) and as the visit's rest after a won battle
  (`Rest::Exchange`, GAM-DEC-007 amended: a sixth rest, given after the winner's levels). The
  original runs it modally (`garrison_exchange_dialog` 0x5D1130).
- **The army reference** (agreed with the town lane, its inbox; E09 decision 67, the town lane's
  survey `E09-growth.md` §15): **`ArmyRef::MapGarrison(index)`**, the garrison by its object index,
  a sibling of `ArmyRef::Garrison(TownId)` (a town's, the town lane's to admit in E09 increment 3:
  its upper bar `Garrison(town)` without a garrison hero, `Hero(garrison hero)` with one). The
  exchange of both is **one `Together::Exchange { hero, army }`** with the stationary army's
  reference, so the town screen's rows reuse the decision kind or its `Together` (the town lane's
  call in T22), and `ArmyCommandError::Garrison` is lifted for an army the open exchange holds.
- **The one owner rule** (`h3-adventure`'s army commands, ADR-0016 point 3 amended): each army
  named has a **bar owner**: a hero's its owner; a map garrison's its owner when **removable**,
  none when not (the original passes −1, survey §6); a town's garrison its town's owner (the town
  lane's); the group its dialog's owner. A command takes a stack only from a bar of the acting
  player; into a bar of another owner, none included, only a **move or a split into an empty
  slot**; never a merge into, a swap with, or a dismissal of such a bar's stack (D-61); a hero's
  last stack never leaves it (ADV-ARM-002); a stationary army may be emptied. Refused, typed
  (`ArmyCommandError::NotOwn`, `::OnlyEmpty`), nothing changed. So an own removable garrison takes
  the army dialog's full rules; a non-removable own one (the only restricted case until E10's
  teams) only takes stacks into its empty slots: troops left there are its owner's, never taken
  back. **Decided** (305): the direct reference with a decision (not the Leave Guards dialog's copied
  group, `Holder::Mine`): the town lane's proposal, one rule for both stationary armies, and every
  view current after each command (the fold's gate).
- **Its view** (the owner's): the garrison by type and anchor, its army by slot, its removable
  flag; told by `ExchangeOpened` (as `LeaveGuardsOpened`), the slots by the commands'
  `StackChanged` (owner alone) and by `GarrisonChanged` to the other viewers (§15).

###### 14. The battles (GAM-BAT-001 amended, ADV-GAR-005; D-60 as the original)

- **A garrison's battle** is built as an owned mine's (`Defender::Garrison`: an army without a hero,
  its owner's side, played outside its turn) or a guarded object's (`Defender::Guards`, the neutral
  seat's); its terrain the entrance tile's, its background by the scenery (GAM-BAT-011 with 219).
  Its end writes the defender's survivors back on the garrison (`GarrisonChanged`), and a win flags
  it and holds the exchange (§13).
- **Anti-magic:** `BattleStart::anti_magic` (new) is set when **the battle's tile is an anti-magic
  garrison's entrance**: the garrison's own battle and a hero standing in one attacked by another
  (`tile_special_ground` 0x4FCE20's two cases, survey §4; the battle's tile is the defender's); the
  game sets `BattleSetup.anti_magic` from it (BAT-TER-002, built by the battle lane: every cast of
  both sides refused). A hero attacking out of a garrison (the defender beside it) fights with
  magic (the attacker's tile is not the battle's).
- **The battle's end clears both heroes' battle bonuses** (GAM-BAT-014 amended: now the
  adventure's change of its heroes on the map, the game calling it), keeping the marks the part
  names as outliving a battle under the profile (§12.3; A-44, decision 308). The battle's setup
  takes each hero's bonuses from the hero on the map for `derive_battle_hero` (§12.5).

###### 15. The day, the week and the views (`h3-game`; GAM-CAL amended, ADV-GAR-006)

- **The day's start:** the adventure's day step drops every hero on the map's day marks;
  **the week's start:** its week marks (before the turn's day points are computed, so a new
  week's first day has no Stables' 400), then the adventure's week step refills every Spring's entrance (drops `springs`) and draws each
  Fountain's value in the weekly pass (§8.2), in object order with the Windmills, Gardens and
  camps (ADR-0017 points 3 and 4).
- **The weekday** passed to `AdventureState::apply` is the calendar's day of the week (1 to 7,
  the original's game +0x1F63E).
- **A map garrison's view** is the game's, as a town's (the levels need the viewer's taverns and
  heroes' Visions, which the game holds): `GameObservation`'s map garrisons by object (type and
  anchor), each `GarrisonView` (TWN-VEW-002, -003: `garrison_view` and the tavern and Visions
  levels, `hero_visions_level` with Visions' parameters from the town part, the town lane's inbox
  note of 2026-11-06), its removable flag to the owner's team. Told by `GarrisonChanged {
  player, object_type, anchor, owner, level, slots }` with a `GarrisonStackSeen` per stack, at
  every change: a battle's end, an exchange's command, a capture, and whenever a viewer's level
  changes (its towns' Taverns, a hero's Visions in reach: the town lane's `TownChanged` triggers,
  one function for both). *Agent's call:* the town lane's events' shape for the map's garrisons
  rather than `GuardsChanged`'s (a mine's hint), since the levels are the same.
- **The kind marks** in the observation: the types the viewer's team holds (`AdventureObservation`,
  `KindKnown` when one is added).

###### 16. The env, replays, fixtures, properties, benchmarks

- **The env** (the AI lane's; `AdventureEnv`): no new action: `Exchange` is answered by the close
  role the army dialog uses, so **action encoding v8 stays** (*agent's call*; else v9, `cross-lane`).
  Random play closes it; a scripted replay exchanges. The observation's additions are the AI
  lane's to encode (no flat encoding, decision 187). The env's `apply` passes the weekday.
- **Replays:** no new command; a management line naming a map garrison's bar needs the replay
  text's `MapGarrison` reference: **adventure replay format v13**. Two new maps written by
  `h3_map::writer` (its seventeen classes, T109's writer work): **`e08-garrisons-36.h3m`** (owned,
  neutral, empty, armed, non-removable and anti-magic garrisons in passages between two players'
  starts; a level-1 hero of each beside one) and **`e08-bonuses-36.h3m`** (each of the fifteen,
  the sea's three on a strait, two Springs, two Fountains, Stables with Cavaliers in reach). Golden
  replays: random play on each (three each, seeds per `adventure_replays.rs`) and scripted ones
  (the exchange's moves and refusals, a garrison won and its exchange after the levels, the
  passage through an own garrison, the Sirens twice around a battle in both profiles, a Spring's
  two entrances, the Stables twice in a week and a new week). Every golden replay's hash lines
  move with the hashes (`homm3-rerecord-replays`).
- **Properties** (`h3-map`'s `play.rs`): the seventeen kinds in the plans (`GARRISONS`, `BONUSES`),
  `Step::Exchange` with arbitrary army commands; `check_garrisons` (an army only changes by its
  battle or the exchange; the owner rule never broken; a non-removable garrison's stacks never
  leave), `check_bonuses` (a mark set once per battle; the amounts the visit's; the day's and the
  week's marks cleared at their steps; the points and the maximum raised together; the Sirens'
  survivors at least one, never more); the views follow (`check_told` for `GarrisonChanged`);
  fixed cases (256) that visit every class.
- **Benchmarks** (`homm3-bench-regression`): the passage in the reach first (T114a, right after
  its first build): `reach-xl-144` and `reach-lifted-xl-144` against `master` (no garrison: flat),
  a new **`reach-garrisons-xl-144`** (an XL map with garrisons across its passages, owned by the
  mover, another and none); `random-play-garrisons-36` and `random-play-bonuses-36`; the forks
  carrying the new blocks (`fork-bonuses-xl-144`, a Spring emptied and the fortunes). The hero
  on the map's growth (two small lists, empty for most heroes) is measured in the fork and play
  workloads.
- `--with-original-data`: the census's numbers (§1) and the import of all 160 shipped maps with
  the seventeen classes typed (4 323 objects become typed on 159 maps: their entrances end paths,
  so the shipped-map counts in `original_data.rs` move once).

###### 17. Versions, rule ids, quirks

| Version | From | To | By |
|---|---|---|---|
| adventure ruleset part | v19 | **v20** | T113a: the six bonus kinds, `garrison`, `kind_mark` |
| map import | v21 | **v22** | T113b: the seventeen classes, the garrison's body |
| adventure state hash | v21 | **v22** | T114a, T115a, T115b: the heroes' bonuses and timed marks, the fortunes, the springs, the kind marks |
| game state hash | v27 | **v28** | T114a (`Exchange`, `Rest::Exchange`), T115a (the details lose the bonuses) |
| adventure replay format | v12 | **v13** | T114b: `MapGarrison` in management lines |
| env action encoding | v8 | v8 | none (§16) |
| `h3-protocol` | v40 | the server lane's | the mapping, `cross-lane` in T114b and T115b |

Unmoved: the core ruleset hash and the pack format (no core change). The ids: part B §11 and part
C §12 (HER-ADV-003, HER-ADV-008 amended; HER-MAN-001 read), GAM-DEC-009 new, GAM-DEC-007,
GAM-MAN-001, GAM-BAT-001, GAM-BAT-014, GAM-CAL amended; `ADV-QUIRK-46`, `-47`; decisions 304 to
309 (next 310).
**The numbers are the next after today's, not reserved:** the town lane's E09 increment 3 plans
the same next ones (its T22a note: the game state hash for the week's kind, map import v22 for
`GameSetup`'s `roe` flag, pack format v52 for its two `HeroEffect` kinds, which increment 12 does
not touch), so whichever task merges first takes a number and the other rebases and bumps again
(the town lane's `TRAPS.md`; E09 T22b's cut orders its bumps with these).

###### 18. Batch E12

Survey §12's sixteen rows in two runs, with one added row; on the client lane's harness in SoD on
Easy, hot seat, from maps `h3-experiment` writes:

- **E12a** (the garrisons, day 1, two players): E12-1 to E12-6.
- **E12b** (the bonus visitables, days 1 to 7 and the next week's first, one player): E12-7 to
  E12-16, and **E12-17** (added): Red's hero without gold visits a School of War (refused: no
  gold), then the hint over a second School of War (the kind mark set before the gold test,
  `E08-rewards.md` §6.1: the bonus line shown). It checks one of increment 7's set points, which
  decision 302 builds from that reading.
- **Predictions added** to E12-4 (Blue's hero attacks from a tile beside the garrison: no cast for
  either; a second battle where Red's hero in the garrison attacks Blue beside it: casts allowed,
  §14), to E12-10 (the second hero's Champions after its own visit the same week: upgraded, no
  points) and to E12-13 (the day's maximum after the Swan Pond kept at the Oasis's raised value).
- **The writer and the spec** (T109; the spec `h3-experiment`'s, battle-owned, `cross-lane`; the
  writer this lane's): the garrisons by class and subclass with an owner, an army and the removable
  byte; the fifteen classes by template; a hero's spell points and army at the start; a second
  player's hero (E12-4).
- **The harness** (T110; `clients/capture/`, common): reads of a hero's +0x105, +0x11A, +0x11B,
  +0x49, +0x4D, +0x18 and of a tile's word (the Fountain's value, the Spring's bit 6), ending turns
  to a given day; the hint's text (E24's `visits` reads it). The rows' verdicts (T111) fold into
  the ADRs at the `shared/` task (T112).
- *Scored* (T111, survey §15): **every row as read** (D-59 to D-64 and A-44 confirmed, the raised
  maximum and the Hat's proportion by it as the original; the attack's and the meeting's steps
  paid, as built). Beyond the rows the visit modes of §2 were the templates', not the original's
  class list: corrected there and in §8 (ADV-Q51, decision 317).

###### 19. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** the `shared/` task (T112, `cross-lane`, as increment 11's T102): the
three drafts applied with batch E12's outcome; `docs/03-domain.md`'s terms (map garrison, exchange,
bar owner, anti-magic garrison, bonus visitable, once-mark and its amounts, day mark, week mark,
kind mark, the Fountain's value, a Spring's entrance); `docs/06-security.md`'s lines (§10). No core
or contract change. `cross-lane` elsewhere: batch E12's spec (`h3-experiment`, T109), the harness
(common, T110), `h3-protocol`'s mapping of the new events, decision, reference and details (T114b,
T115b: the server lane's inbox first, or `cross-lane` in the same pull request). **The inboxes:**
the town lane's (the army reference agreed, now: §13; the versions' order; the views' triggers,
§15), the server lane's (the wire items, `SRV-STUB-2`'s part, T114b), the AI lane's (v8 kept, the
weekday in `apply`, the new decision answered by its close role), the client lane's (CLT-Q40's
amounts, T115a), the map client lane's (the seventeen types, the garrison's view, the kind marks
for the hint, T113a and T115b).
*As built* (T112, `shared/e08-t112`, `cross-lane`; decision 318): the three drafts in ADR-0013
(points 1, 2, 13.1, 15.1, 15.2), ADR-0016 (points 1, 3, 4, 5, 6) and ADR-0017 (points 1, 2, 3, 4,
6, 9, 10, 11), each with batch E12's rows cited, and deleted; the index's Read-when cells; the
glossary's terms (a Fountain's value and a Spring's entrance among them, the battle bonuses' line
amended); `docs/06-security.md`'s increment 12 section (§10's lines and the exchange's
validation); `EffectSource::Visit`'s doc comment; the experiments README's visit modes (decision
317). No version moved.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E12 runs before the build and before the shared task** (decision
303), so the ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T109** | Batch E12's maps: the writer's seventeen classes (the garrison's body, the fifteen by template), the spec's fields (`cross-lane`), the two specs and templates (§18) |
| 3 | **T110** + **T111** | T110: the harness's reads, the recipes, the runs E12a and E12b. T111: scored (survey §15) |
| 4 | **T112** | `shared/` (`cross-lane`): the three ADRs with T111's outcome, the glossary, `docs/06-security.md` |
| 5 | **T113** | T113a: the kinds, the packs, the kind marks' flag, the harvest's check (§1 to §3; part v20). T113b: the import, the random stack's refusal, 219 in the background's classes (§4; map import v22); a step onto them refused as any unbuilt visit's until T114 and T115 |
| 6 | **T114** | T114a: the garrisons' state, visit, battle, capture, anti-magic, the passage with the reach's measure against `master`, the exchange's decision, `ArmyRef::MapGarrison` and the owner rule (§5 to §7, §13, §14; game state hash v28, adventure state hash v22). T114b: the views and their events (`GarrisonChanged`), the replay format v13, the env's close, `h3-protocol`'s mapping (`cross-lane`), the town lane's inbox |
| 7 | **T115** | T115a: the battle bonuses moved to the hero on the map as the dynamic list, increment 7's Tomb, Pyramid, Pandora's Box and events moved with them, the timed marks, `derive_battle_hero`'s and `land_points`' inputs, the battle's clear and the day's and the week's drops (§12, §14, §15; the client lane's inbox). T115b: the fifteen's visits, the Fountain's block and draws, the Springs' block, the Sirens (`ADV-QUIRK-46`, `-47` entries), the kind marks, the weekday in `apply`, the events and views (§8 to §10) |
| 8 | **T116** + **T117** | T116: the two maps, their replays and properties, the benchmarks, `--with-original-data` (§16). T117: the audit against the exit criteria; increment 12 closed; increment 13 refined |

*Agent's call:* the garrisons before the bonus visitables (they share nothing but the import and
the versions; the town lane's T22 waits on the army reference built, not only agreed), each a round
of two.

**Increment 12's exit criteria:**
1. The seventeen classes import typed by the packs of their expansions (33's and the fifteen in
   the base pack, 219's in the AB pack); a random stack in a garrison is refused, a count of 0 is an
   empty slot; the census holds (ADV-GAR-001, ADV-BON-001).
2. A garrison of the visitor's team opens the exchange; another armed one is fought at once, unsplit,
   its owner's side or unowned guards; a won or empty one is flagged with its reveal, then the
   exchange after the winner's levels; a lost battle leaves the survivors (ADV-GAR-002, ADV-OWN-001).
3. The exchange's commands follow the one owner rule: an own removable garrison the army dialog's
   rules, a non-removable one only moves and splits into its empty slots (D-61); the town lane's
   reuse agreed (ADV-GAR-003, GAM-DEC-009, GAM-MAN-001).
4. A path and a move cross the mover's team's garrisons without a visit and visit the one at their
   end; every other garrison is a destination (D-59); the reach's workloads hold their baselines
   (ADV-GAR-004, ADV-OBJ-003).
5. A battle on an anti-magic garrison's entrance, its own or a hero's standing on it, has no casts
   (D-60; ADV-GAR-005).
6. The ten `bonus` kinds and the Fountain give their morale and luck once until the next battle,
   by the day (the Idol, the Temple) and the week's value (the Fountain); each mark reaches the
   battle as its own source (ADV-BON-002, -003, HER-ADV-008).
7. The movement objects raise the points left and the day's maximum, the Swan Pond empties the
   points; the Stables give the week's movement once and upgrade at every visit (D-63); the Well
   fills to the maximum once a day, a full hero unmarked; each Spring entrance to twice it once a
   week (D-62) (ADV-BON-004 to -007, HER-ADV-003).
8. The Sirens keep the double product's survivors (D-64) in full width (A-45), give the experience
   with its levels, and are visited again after a battle only with `ADV-QUIRK-46` on (A-44)
   (ADV-BON-008).
9. A viewer sees a garrison's owner and its army at its level, the removable flag only in its
   owner's team; the kind marks its team holds; never a Fountain's value or a Spring's state; the
   events as §10 lists (ADV-GAR-006, ADV-BON-009).
10. Every new rule has its id, source and tests; `ADV-QUIRK-46` and `-47` recorded; the gates pass
    (coverage, determinism with the new and rehashed replays, the benchmarks at their baselines and
    the new ones baselined); batch E12 run and scored (T111), the consensus rows `verified`.

###### 20. Risks and open points

- **Two lanes in the same crates:** the town lane's E09 increment 3 (T22 on) builds the town
  screen's exchange on §13's reference and touches `h3-game`'s decisions and `h3-adventure`'s army
  commands; whichever builds first, the other reuses it (E09 decision 67). The versions of §17 are
  taken in merge order. The weekday parameter of `apply` (T115b) touches every caller: a wide,
  mechanical change in its own commit, the town lane told first.
- **The reach's visitable branch** gains an owner read for garrisons: flat on the maps without
  them, measured on `reach-garrisons-xl-144`.
- **4 323 objects stop being inert** on 159 shipped maps: the entrances become destinations, so
  any test that used one of these classes as a plain blocker breaks (`TRAPS.md`), and the
  shipped-map counts move once (T113b, T116).
- **The bonuses move** (decision 307): increment 7's Tomb, Pyramid, Pandora's Box and events, the
  battle's clear (GAM-BAT-014), `derive_battle_hero`'s callers (the game's battle setup, the
  siege's defender, the tests) and `h3-protocol`'s details mapping change in one task (T115a), its
  own commit, the server and town lanes told first. **A garrison hero** (E09's increment 3) needs
  its bonuses and timed marks kept while it stands in a town's garrison if it leaves the heroes on
  the map (the town lane's call; its inbox).
- **A-44's fix** (decision 308: one mark for every Sirens under `strict-fixed`) cannot be checked
  by a batch row (SoD reproduces the bug).
- **Not read** (survey §3, §7, §10): the computer's garrison visits and paths (`ai_garrison_visit`
  0x524370; E10), network play, the pictures and sounds (E24), the garrison dialog's drawing of
  the removable flag; whether anything ends the move after a won garrison battle beyond the visit's
  end (increment 3's battle rules hold).
- **Not in increment 12** (decision 295): the trades and services, the information objects, the
  Prison, the Market of Time, creature banks, quests, the adventure spells, simultaneous play,
  towns (E09), a computer player's routes (E10), a defeat's neutral garrisons (E10).
