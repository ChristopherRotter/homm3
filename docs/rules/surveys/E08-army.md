# E08 army and hero management survey (E08 T44a): the army dialog, meetings, a hero's army, the formation, the hero's artifacts

Date: 2026-10-23. Scope: E08 increment 6 (decisions 90, 91; T44a whole: decision 92), input for
T44b's ADR amendment drafts, the design note and the cut. Input, not rules (`README.md`): a fact
becomes binding when a task writes it into the catalog with its sources and tests. **The
checkpoint** (ADV-Q24, 2026-10-23: the epic's decisions 93–100) answered §12's questions; §12
carries the answers.

**Sources** (retrieved 2026-10-23):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path in hot seat: the join that finds no room (`monster_join_free` 0x4A7000,
  `monster_join_for_gold` 0x4A7250) into `army_full_dialog` 0x5D11D0 and the garrison exchange it
  builds (0x5D0B40, 0x5CE830, its procedure 0x5D0910, the town screen's garrison logic 0x5D4C10
  and 0x5C7400 that it reuses); the meeting (`visit_hero` 0x4A2800, read in T17a) with Scholar
  (0x4A2940) and the exchange screen (its manager 0x5AE500, procedure 0x5AFDF0, army 0x5AF2B0 and
  0x5B0DA0, worn artifacts 0x5AF590, backpack 0x5AF990, closing 0x5AED20); the hero screen's
  procedure `hero_screen_proc` 0x4DD2D0 (its army slots, buttons, worn positions and backpack
  places) and its closing 0x4E1550; the split dialog 0x449E90; the creature dialog 0x4C6C50 (its
  dismiss); `army_add_stack` 0x44ACE0 and `army_swap_slots` 0x44AD60; the artifacts' helpers
  (`hero_can_replace_artifact` 0x4E2840, the combinations 0x4DC100, 0x4D9CC0, 0x4DBE80, 0x4DBF30,
  0x4DC070, read with T19a2a's `hero_equip_artifact` 0x4E2A00, `hero_remove_artifact` 0x4E2BD0,
  `hero_give_backpack_artifact` 0x4E2F90, `hero_give_artifact` 0x4E3070); the tactics phase's test
  in `battle_start` (0x462C2C to 0x462D0D). **Not followed:** network play and allied players
  (the exchange's branches on the game type 0x6989F0, its +0x5C/+0x5D, the team checks at
  `game` +0x1F879: §2.3), the computer player's paths (0x52BC60 for a join without room, 0x525DC0
  for a meeting), the town screen's own garrisons (E09), the creature dialog's upgrade (E09), the
  other callers of 0x5D11D0 (0x4ABDC0, 0x573C80: rewards that give creatures, increment 7).
- `vcmi`: VCMI 1.7.5 (`git clone --depth 1 --branch 1.7.5`): `server/CGameHandler.cpp`
  (`tryJoiningArmy`, `showGarrisonDialog`, `heroExchange`, `useScholarSkill`, `arrangeStacks`,
  `moveStack`, `swapStacks`, `eraseStack`, `disbandCreature`, `setFormation`, `setTactics`,
  `moveArtifact`, `assembleArtifacts`), `server/NetPacksServer.cpp` (each command's owner and
  turn checks), `lib/networkPacks/PacksForServer.h`, `lib/mapObjects/CGCreature.cpp`
  (`joinDecision`), `lib/mapObjects/CGHeroInstance.cpp` (`onHeroVisit`),
  `lib/entities/artifact/ArtifactUtils.cpp`, `lib/entities/artifact/CArtifact.cpp`
  (`canBePutAt`), `lib/battle/BattleInfo.cpp` (the tactics side), `config/gameConfig.json`
  (`backpackSize` -1), `client/widgets/CGarrisonInt.cpp`, `client/ArtifactsUIController.cpp`;
  links `https://github.com/vcmi/vcmi/blob/1.7.5/<file>`.
- `wiki`: read through the summarising fetch tool, `https://heroes.thelazy.net/index.php/<Page>`:
  Artifact, Combination artifact, Scholar, Titan's Thunder, Tactics, Troop stack, Restoration of
  Erathia Manual pages 30, 31, 32, 41, 42 (the manual describes RoE: no combinations). Thin on the
  hero screen's rules; quotes are short (the tool's limit).
- `original-data` (the census): the installation's 160 shipped maps (`original-data/maps.tar`)
  read with `h3-map`'s reader and a throwaway example (not committed), §7.

## 0. Findings in short

- **The army dialog** (§1): when a joining group finds neither a stack of its creature nor a free
  slot (`army_add_stack` 0x44ACE0: the first stack of the type, else the first free slot), the
  human player's join is not refused: after the yes (and the gold of a join for gold) the game
  opens a modal "Make room for the %s in your army" (GENRLTXT 36): a garrison exchange between the
  group, held as one stack in an army of seven slots with the player as owner and no hero, and the
  hero's army. The player moves, swaps, merges, splits and dismisses stacks between the two (the
  hero's last stack may not leave it), may open the hero screen, and closes with OK; whatever is
  left in the group is lost, the treasure is given and the monster leaves the map as after any
  join. VCMI does the same (it first moves the stacks that fit).
- **Meetings** (§2): a step toward another hero of the same team (increment 6: the same player's)
  visits it from the neighbouring tile at the step's price, the move ends (as a battle's visit,
  GAM-BAT-004). First, for both heroes, **Scholar** (0x4A2940): with the better Scholar level s of
  the two above 0 and both wearing the Spell Book, each learns the other's learned spells it
  lacks up to its own limit min(Wisdom level + 2, s + 1) (Basic Scholar: level 2). Then the
  exchange screen opens for a human player; it stays open until closed.
- **A hero's army** (§3): within one hero, a click on a second slot merges a stack of the same
  creature into it, moves to an empty slot or swaps; with the split button or Shift, a split
  dialog. Between two heroes the same, but the source hero's **last stack** may not merge away
  ("Cannot combine hero's last army") nor move to an empty slot; it may swap. The split dialog
  keeps one creature in a hero's last stack, and (an oddity, A-16) also on the source side when
  the destination hero has a single stack. A stack is dismissed from the creature dialog, offered
  only while its army has more than one stack. `army_swap_slots` 0x44AD60 moves one of the two
  counts through 16 bits: a stack of 32 768 or more creatures loses its count on a move or a swap
  (quirk candidate **A-15**, state-corrupting).
- **The formation** (§4): the hero's byte +0x48, bit 0 tight (the battle reads it for a human
  player's side only, T22b) and **bit 1 the tactics formation**, toggled by a third button (the
  manual: shown to heroes with Tactics); every hero starts with +0x48 = 2 (loose, tactics on). A
  human player's hero whose Tactics would give its side the tactics phase gets none with bit 1
  clear. Changing either costs nothing.
- **The artifacts** (§5): 19 numbered positions (the order matters for the capture, HER-ART-004,
  and for "the first position that takes it") and a backpack of 64 in order: a dropped artifact is
  inserted at the place it is dropped on, a picked one closes the gap, and the backpack's arrows
  **rotate the stored array**. The Spell Book and the Catapult never move ("The Catapult must be
  equipped.", "This item can't be traded."); the war machines move only between machine
  positions (the backpack refuses them); the Grail only between backpacks. An artifact held on the
  cursor when a screen closes is given back (equipped, else the backpack). **Combinations** (SoD
  maps only, the game's version +0x1F698 ≥ 2): assembled at a worn part's position when every part
  is worn (a right-click asks every time; equipping a part asks once per player and combination;
  a part received and equipped asks a human once, builds at once for a computer); the parts leave (one copy each,
  positions 18 down to 0) and the combination goes to the first position that takes it.
  Disassembly: a right-click on a worn combination, the parts to the first free positions; never
  in the backpack (VCMI allows it: D-28). Equipping Titan's Thunder gives a Spell Book to a hero
  without one, kept when the Thunder is removed.
- **What another player sees** (§6): nothing: all of it is the owner's (another player's hero's
  army and details are never shown, ADV-FOG-006). Every change is told to the owner only.
- **Content** (§7): of the 373 heroes placed in the shipped maps, 40 have custom artifacts, 11 a
  backpack (one or two artifacts; the Grail in Sangraal's Thief), none a combination or a full set
  of parts, none Titan's Thunder; one is tight (Unholy Quest, known); 3 have seven stacks.

## 1. The army dialog (`army_full_dialog` 0x5D11D0)

### 1.1 The reading

1. **When.** `monster_join_free` (0x4A7000) after the human player's yes, and
   `monster_join_for_gold` (0x4A7250) after the gold is paid, add the whole group as one stack
   with `army_add_stack` (0x44ACE0, read: with slot -1 the first slot of the creature, else the
   first empty slot; none: 0, nothing changes). Only when that fails (0x4A7105) does a human
   player (the routine's argument) get `army_full_dialog`; a computer player gets 0x52BC60 (not
   read, E10). Then, either way, the monster's treasure (0x4A6B30) and the monster's removal. The
   group's creature can be no creature of the hero's army there, and the army has seven stacks.
2. **The dialog** (0x5D11D0, read): a local army of seven empty slots (0x44AA80, 0x44AB00) gets
   the whole group as one stack (`army_add_stack`, -1); 0x5D0B40 builds the dialog over the
   garrison exchange 0x5CE830 (the hero, its owner, the local army): the upper garrison bar is the
   local army with the hero's owner as owner and no hero (`garrison_interface_create` 0x5A9D20:
   +0x28 the owner, +0x74 the hero 0), the lower one the hero's army (+0x91) with the hero; the
   town manager's (0x6994FC) +0x38, the town, is 0. Its title is GENRLTXT 36 "Make room for the %s
   in your army" with the creature's singular for a count of 1, else the plural (0x5D0C40). It
   runs modally (0x5FF460); when it returns, the local army is destroyed: what is left in it is
   lost.
3. **What the player may do** (the procedure 0x5D0910: the upper slots are items 0x73-0x79, the
   lower 0x8C-0x92, the hero's portrait 0x7C, the split button 0x9A; a right-click on a slot shows
   the creature's dialog, 0x4C6C50): the clicks go to the town screen's garrison logic, 0x5D4C10,
   in the mode the last hover set (0x5D05F0 → 0x5C7400 for an army of the current player, read;
   0x5C8080 for another's, not read). With no town, 0x5C7400 decides for a selected stack and a
   target slot: the same slot again views the stack (its dialog, with the dismiss offered for a
   garrison without a hero, always, and for the hero's army while it has more than one stack,
   0x5D4CC0); a stack of the same creature merges into the target, an empty target takes it, any
   other is swapped; with the split button or Shift (the message's flags & 3) a split dialog
   (0x449E90, §3.3). The **hero's last stack** (the selected army the lower bar, one stack, the
   target in the other bar) may not merge into the group ("Cannot combine hero's last army",
   TCommand 1) nor move to an empty upper slot ("Cannot move last army to garrison", TCommand 5);
   it may be swapped. The portrait opens the hero screen (mode 4, `H3Hero::ShowDialog` 0x4E1800),
   where the hero's own army and artifacts are managed (§3.1, §5). OK closes the dialog.
4. So the player makes room by dismissing a stack of the hero's (a free slot), merging two of its
   stacks of one creature, or moving one of its stacks into the group (lost with it), and then
   takes the group or part of it (a split into a free slot); or takes nothing.
5. **The other callers** of 0x5D11D0, 0x4ABDC0 (0x4AC3FB) and 0x573C80 (0x573EBB), give a
   reward's creatures the same way (increment 7, rewards on the map). The dialog's other title
   (an argument of 0x5D0B40: GENRLTXT 710 "Leave Guards") belongs to 0x5D12B0 (not read; mines
   and garrisons, E09 and later).

### 1.2 VCMI and the wiki

`vcmi`: `CGCreature::joinDecision` gives the reward, sets the counts and calls
`tryJoiningArmy(this, hero, removeObjWhenFinished = true, allowMerging = true)`: when the group
cannot be merged, it first moves every stack that finds a slot (one creature type: none here) and
then shows `showGarrisonDialog(group, hero, removableUnits = true)`; the object leaves when the
dialog closes, the creatures left with it. `moveStack` refuses moving a hero's last whole stack
away (`needsLastStack`), as the original. VCMI's client adds conveniences (moving all but one
creature of a last stack, the radial menu's splits), not rules. **Consensus.** `wiki`: silent
on the dialog.

## 2. Meetings of a player's own heroes (`visit_hero` 0x4A2800)

### 2.1 Where and at what price

`original-data` (T17a's reading, survey `E08-battles.md` §7.1, and GAM-BAT-004): a hero is
visited from the neighbouring tile (class 34, a neighbour-visited object: the step toward it is
paid as if taken and the hero stays, ADV-OBJ-003); `visit_hero` stops the move (0x417680) and
compares the teams (`game` +0x1F879 + player, 0x4A287C): a hero of the visitor's team is met, an
enemy is attacked. The visited hero's object is what the step meets, whatever lies under it, but
not across the top of a tile not visited from above (GAM-BAT-004's case); an own hero in a town is
the town's visit (E09). The meeting: Scholar for both (0x4A2940, §2.2), then for a human player
the exchange screen (0x4AADF0 → 0x5AE500, §2.3), for a computer player 0x525DC0 (not read, E10).
The engine refuses the step today (`a_hero_visit_ends_the_move_and_an_own_hero_blocks`).

### 2.2 Scholar (0x4A2940)

`original-data` (read, 0x4A2940 to 0x4A2AD7): s is the greater of the two heroes' Scholar levels
(+0xDB, skill 18). With s = 0 nothing happens. Both must wear the Spell Book
(`hero_wears_artifact` with 0, the book position). Each hero's limit is the lesser of its Wisdom
level (+0xD0) + 2 and s + 1. For every spell 0 to 69: a spell one hero has learned (+0x3EA) and the
other has not is learned by the other (`hero_learn_spell` 0x4D9330) when its level is at most the
learner's limit. Only learned spells pass (not an artifact's or a scroll's). A human current
player gets a message (GENRLTXT 140 to 143, presentation). It runs at every meeting, before the
screen. `vcmi` `useScholarSkill`: the same (`LEARN_MEETING_SPELL_LIMIT` 2, 3, 4; each hero's
`maxSpellLevel`; both spell books; each learns the other's). `wiki` (Scholar): "teach each other
1st and 2nd level spells" at Basic, to the 3rd and 4th above; "both gathering heroes learn".
**Consensus.** The engine's learn level (HER-SPL-005: 2 + Wisdom, at most 5) is the limit's
Wisdom side.

### 2.3 The exchange screen (`swap_manager_create` 0x5AE500, `swap_manager_proc` 0x5AFDF0)

`original-data` (read for two heroes of one human player): the manager keeps the visitor
(+0x40) and the visited (+0x44); its items: the portraits (1, 2: the hero screen of a hero of the
local player), each hero's army (13-19 and 20-26; also 65-71, 72-78), worn positions (27-45,
46-64) and backpack places (89-93, 94-98), the split button, the skills' views. Army clicks:
§3.2; artifacts: §5.2. It stays open until closed; on closing (0x5AED20) an artifact still held
on the cursor goes to the **visitor** (`hero_give_artifact`, whichever hero it came from; lost
when the visitor has neither a position nor backpack room for it: the second reading). No
formation buttons, no dismissal of a hero. **Not followed:** two heroes of different players of
one team: a hero of the other player can only receive (a stack into an empty slot, 0x5B0DA0; an
artifact into an empty position or the backpack, 0x5AF590, 0x5AF990), and network play's
messages; teams are E10. `wiki` (the manual, page 32): "You may move, split, combine, swap, and
dismiss troops as described in Hero Army"; each hero keeps at least one troop; between players'
heroes trades are one-way. `vcmi` (`heroExchange`): an exchange query between any two non-enemy
heroes, Scholar first.

## 3. A hero's army

### 3.1 In the hero screen (`hero_screen_proc` 0x4DD2D0, items 68-74)

`original-data` (read, 0x4DD962 to 0x4DDBCD): the selected slot is 0x697738 (-1: none). A
first click selects a non-empty slot. A click on the selected slot again opens the creature
dialog (0x4C6C50) with the dismiss offered when the army has more than one stack (0x4DDA0E) and
the screen is not view-only (0x6AA9D8). A click on another slot: with the split button pressed
(item 127 sets 0x698A44) or Shift held, and the target empty or of the same creature, the split
dialog (0x449E90 with both flags 0, §3.3); else a target of the same creature takes the selected
stack whole (the counts added, the slot emptied, 0x4DDB07); else `army_swap_slots` (a move into
an empty slot, or a swap). Within one army nothing is refused: the army keeps a stack.

### 3.2 Between two heroes (`swap_army_click` 0x5AF2B0, `swap_army_move` 0x5B0DA0)

`original-data` (read): only a stack of a hero of the local player is selected (+0x48 the side,
+0x50 the slot); a right-click shows the stack (no dismiss). A click on the selected slot again
opens the creature dialog with the dismiss offered while that army has more than one stack
(0x5AF43A). On another slot, with the split button (0x6A3D08) or Shift and the target empty or of
the same creature: the split dialog (0x449E90 with both flags 1, §3.3). Otherwise 0x5B0DA0:
- the target of the same creature: the source stack merges into it, **unless the source army has
  a single stack** (refused; this also covers the hero's own army, where it cannot arise);
- else, within one army: `army_swap_slots`;
- else, the source army with a single stack and the target slot empty: **refused**; a swap is
  allowed (each army keeps one stack);
- else the move or the swap, the target hero being the local player's (another player's hero of
  the team: only into an empty slot).

### 3.3 The split dialog (0x449E90)

`original-data` (read): its arguments are the source army and slot, the target army and slot and
two flags (keep the source, keep the target). With a the source's count and b the target's (0
for an empty slot), t = a + b; m₁ = 1 when (the first flag and the source army has a single
stack) or (the second flag and the target army has a single stack), else 0; m₂ = 1 when the second
flag and the target army has a single stack. The slider starts at b − m₂; at OK the target's new
count is b′ held to [m₂, t − m₁] (0x44A0E2: below m₂ it becomes m₂, above t − m₁ that), the
source's is t − b′; a count of 0 empties its slot; the target takes the source's creature. The
hero screen passes 0, 0 (one army: no bounds but [0, t]); the exchange passes 1, 1. So between
two heroes a hero's last stack keeps a creature, a target hero's only stack keeps one, and, **the
oddity (A-16)**, when the target hero has a single stack the source keeps one creature even if
the source hero has other stacks (a plain click moves that stack whole: §3.2). Its title is
"Split %s" (GENRLTXT 257, 0x449790).

### 3.4 Dismissing a stack (the creature dialog 0x4C6C50)

`original-data` (read): the dialog's dismiss button (item 0x7803 in its procedure 0x5F4850)
asks "Are you sure you want to dismiss this army?" (GENRLTXT 13; presentation); on yes the slot
is cleared (`H3Army::Clear` 0x44AB60). Offered: §3.1, §3.2, §1.1 point 3. A dismissed stack is
gone (no gold back). `vcmi` (`disbandCreature` → `eraseStack`): refused for a hero's last stack;
`wiki` (the manual): "you can't trade away or dismiss a hero's last one". **Consensus.**

### 3.5 `army_swap_slots` (0x44AD60) and A-15

`original-data` (read): it swaps two slots' creatures and counts, `this` army's slot a with
another army's slot b; slot b receives a's count through a 16-bit register (`mov di, word ptr`,
then `movsx`): a count of 32 768 or more arrives as its low 16 bits sign-extended (40 000 becomes
−25 536). Its callers: the hero screen's move or swap (a: the clicked slot, b: the selected one,
0x4DDB53), the exchange's (a: the source, b: the target, 0x5B0ED5), the town's garrisons (0x5D4E97).
A merge (added counts) and a split (the dialog's 32-bit counts) are untouched. **Quirk candidate
A-15**: state-corrupting (a negative or wrong count), so fixed in every profile (AGENTS.md); no
shipped map places such a stack (§7), a long game can grow one (Necromancy). `vcmi`: 32-bit
counts. `wiki` (Troop stack): "unlimited quantity in game before overflow" (silent on this).

### 3.6 VCMI and the wiki

`vcmi` `arrangeStacks` (swap, merge, split of a new count), `moveStack`, `swapStacks`,
`eraseStack`, each behind the owner's and the active player's checks (`throwIfWrongOwner`,
`throwIfPlayerNotActive`): the same rules but A-16's (its split takes any count that leaves a
hero's last stack one creature, `CGarrisonSlot::split`'s `minLeft`, `minRight` for a target
that is the same creature's only stack: D-27) and A-15. `wiki`: "A hero's army must have at
least one troop stack and can include up to seven"; the manual's swap ("click on one of the
troops to highlight it, and then click on the second troop").

## 4. The formation and the tactics formation

`original-data` (read): `hero_screen_proc`'s buttons: item 122 clears bit 0 of +0x48 (loose,
0x4DD577), 124 sets it (tight, 0x4DD587), **126 toggles bit 1** (0x4DD5AD); 127 is the split
button, 128 the quest log, 129 the hero's dismissal (GENRLTXT 23, E09); 77 and 78 the backpack's
arrows (§5.1). A hero's flags start at 2 (0x4D8ADD: loose, the tactics formation on); the map's
formation flag never reaches them (T22b, `ADV-QUIRK-8`). The battle: bit 0 places a human
player's side tight (`battle_war_machine_stacks` 0x463673); `battle_start` (0x462C2C to 0x462D0D)
opens the tactics phase when the two heroes' Tactics levels (+0xDC; 0 without a hero) differ
(and +0x53C5 is clear: a setup argument that also skips the obstacles, probably a creature bank's
battle, not read), for the side with the higher one, and then clears it again when that side's player is human
(+0x54A4 + side) and its hero's bit 1 is clear (0x462D07), unless the game's +0x1F69D or the
global 0x69877C is set or network play takes the other branch (not read). `wiki` (the manual,
pages 30 and 42): the Tactics Formation button appears only for heroes with Tactics and "toggles
the Tactics formation option on and off"; HeroScrn.txt's help: "Enabling Tactics formation will
allow you to arrange your troops prior to combat (must have the Tactics skill)". `vcmi`
(`tacticFormationEnabled`, true at creation; `setTactics`; `BattleInfo`): a hero with it off
gets no reposition but still blocks the enemy's; in SoD that gives the original's outcomes.
**Consensus.** Both buttons cost nothing and change nothing else. The engine has the formation
(ADR-0016 point 3; T18b) but no command, and no tactics flag: BAT-TAC-001 gives the phase from
the heroes' Tactics levels alone.

## 5. The hero's artifacts

### 5.1 Positions and the backpack

`original-data` (read, with T19a2a's readings): 19 numbered positions (+0x12D, 8 bytes: the
artifact and its second word, a scroll's spell): head 0, shoulders 1, neck 2, right hand 3, left
hand 4, torso 5, rings 6 and 7, feet 8, misc 9 to 12 and 18, war machines 13 to 16, spell book 17
(ArtSlots.txt). An artifact goes to a position of its kind's set (`hero_can_replace_artifact`
0x4E2840, read: the position in the set, then `hero_can_place_artifact` 0x4E2550 with the
combination's locks counted, +0x1C5; for an occupied position the same test with it removed and
put back). "The first position that takes it" (`hero_equip_artifact` with -1) walks them in
order, so positions decide where an artifact lands and, through HER-ART-004's capture in
position order, which artifacts a winner takes first. The backpack (+0x1D4, 64 entries, +0x3D4 the
count) is ordered: `hero_give_backpack_artifact` (0x4E2F90) inserts at an index (later entries
move up) or at the end with -1, and refuses at 64 and for the war machines 3 to 6 ("Your backpack
is full.", "Alas, your backpack cannot hold something as large as a %s.": 0x4E2ED0, GENRLTXT 153,
154). The hero screen shows the array's first five entries; its arrows **rotate the array**
(item 77: the last becomes the first, 78: the first the last, 0x4DD46E, 0x4DD4E2), so scrolling
reorders what HER-ART-004's capture walks from the end. The engine keeps worn artifacts by kind
(`HeroDetails.worn`, HER-ART-001: "positions of one slot are interchangeable") and the backpack
in order.

### 5.2 What moves and what never does

`original-data` (read): in the hero screen (0x4DDBCD to 0x4DE2E0) a click with nothing held picks
a worn artifact up onto the cursor (0x698A88, removed from the hero: `hero_remove_artifact`),
except the Spell Book (position 17: the click opens the book) and the Catapult (16: "The
Catapult must be equipped.", GENRLTXT 313); a backpack entry is picked up the same way (the later
ones move down). With an artifact held: a worn position takes it when 0x4E2840 allows (equipped;
an artifact there goes onto the cursor: a swap); a backpack place takes it by insertion (refused:
the message, it stays held). War machines go only to their own positions (the backpack refuses
them), so within one hero they only go back. The exchange (0x5AF590, 0x5AF990) is the same between
the two heroes: positions 16 and 17 are never picked up ("This item can't be traded.", GENRLTXT
22; a right-click on the book says it too), a drop on an occupied position swaps, a drop into the
backpack inserts. So a war machine passes to the other hero's empty machine position (or swaps
with the same machine), the Grail (no position) from backpack to backpack. When the hero screen
closes (0x4E1550) a held artifact is given back with `hero_give_artifact` (the first position that
takes it, else the backpack). `vcmi` (`moveArtifact`, `ArtifactUtils::unmovableSlots`: the book
and the Catapult; `isBig`: no war machine in the backpack; the backpack's cap
`HEROES_BACKPACK_CAP`, `backpackSize` -1: unlimited, D7 of HER-ART-002): the same rules, the cap
aside. `wiki` (Artifact): 64 artifacts in the backpack; the four machine slots and the book slot
"reserved only" for theirs; "A hero may carry only one of each war machine type at a time" (the
manual). **Consensus** but D7.

### 5.3 Combinations (SoD)

`original-data` (read): every path tests the game's version (+0x1F698 ≥ 2, SoD); in an RoE or
AB map the parts never combine.
- **Assembling** at the position p of a worn part of combination c (setup +0x18) when every part
  is worn (each part's type at least once; 0x4DBE80 or the hero screen's inline test):
  `hero_build_combination` (0x4DBF30, H3API `BuildCombinationArtifact`, read) removes the part at
  p, then one copy of each other part from the worn positions 18 down to 0, and equips the
  combination at the first position that takes it (not necessarily p). Offered: a right-click on
  the part in the hero screen or the exchange asks every time (0x4D9CC0: "You possess all of the
  components needed to assemble the %s. Do you wish to assemble this artifact?", GENRLTXT 733);
  after an equip in either screen 0x4DC100 asks once per player and combination: the player
  record's bit set at +0xE8 (bit c) is set when the question is asked, or when a combination is
  equipped, and a set bit skips it; `hero_give_artifact` (an artifact received and equipped, not
  one put into the backpack: a capture, later a reward) asks the local human while the bit is
  clear, builds at once for a computer player, and sets the bit either way (T19a2a; the second
  reading: a human's set bit means neither a question nor a build). A declined question leaves
  the right-click.
- **Disassembling** a worn combination: a right-click asks (GENRLTXT 734, 0x4D9B30) and then
  `hero_disassemble_combination` (0x4DC070, the hero screen's own copy at 0x4DDCDF) removes it and
  equips each part at the first position that takes it (a part that found none would be lost;
  the combination's locks kept those places, so none can). A combination in the backpack is not
  disassembled (a right-click there shows it).
- The combination's bonuses are its own plus its parts' (`hero_equip_artifact` adds both, T20a1;
  the wiki: "Original artifacts still give their regular bonuses").

`vcmi` (`assembleArtifacts`, `assemblyPossibilities`, `ArtifactsUIController`): the same
assembly (all constituents worn when the client asks after an equip from another hero; any
constituents found otherwise), the combination at the clicked position; disassembly also in the
backpack when it has room for the parts (**D-28**). `wiki` (Combination artifact): "Combining is
performed by equipping all the required artifacts, and right-clicking one of them"; it "reserves
the same artifact slots as its original artifacts". **Consensus** but D-28.

### 5.4 Titan's Thunder and the Grail

`original-data`: `hero_equip_artifact` (0x4E2A64) gives a hero equipping Titan's Thunder (135),
on any path (a drop, a give, a combination's build), a Spell Book in an empty book position;
`hero_remove_artifact` takes nothing back, so the book stays, and every hero that equips the
Thunder gains one. `vcmi` (`checkSpellbookIsNeeded`): a book on a move of the Thunder into the
right hand of a hero without one; kept. `wiki` (Titan's Thunder): "Gain a Spell Book". **Consensus**
(the original's rule, not a quirk). The Grail (2): no position, the backpack only; the capture
keeps it out (0x46A070, T19a2a); a meeting may pass it on.

### 5.5 What a gain or a loss changes, and when

`original-data`: equipping and removing change the hero's primary skills at once (+0x476 to
+0x479, T20a1) and recompute its available spells (`hero_available_spells` 0x4D95D0 is called
only from 0x4E2A00 and 0x4E2BD0); the spell points are not touched (a lost Knowledge artifact
leaves them above the new maximum); the day's points and the scouting radius wait for the next
turn start and step (ADV-MOV-002, ADV-FOG-003, increment 5); the hero screen's 0x4E16D0 only
redraws. The engine derives all of it from the details on demand (`h3-heroes`), so a move of an
artifact needs nothing else.

### 5.6 The engine's model against the original's

`HeroDetails.worn` lists artifacts without positions (HER-ART-001); a command that names a
position, "the first position that takes it" (a give, a disassembly, a closing screen) and
HER-ART-004's capture order need the positions (§12, question 3). The backpack's order is kept;
its rotation is a reorder. Scrolls keep their spell (the second word, HER-ART-003).

## 6. Commands, validation, events, what another player sees, the env

The original's screens are interactive; the engine has no blocking prompts (ADR-0017 point 6),
so what a screen allows becomes commands and pending decisions. What the reading gives each:

- **The commands** (each the actor's, in its turn, for its own heroes; validated against the
  state, a rejection changes nothing): a stack's move, swap, merge and split (a target count)
  within a hero or between two met heroes (§3); a dismissal (§3.4); the formation and the tactics
  flag (§4); an artifact's move between a position and a position, a position and the backpack,
  or two backpack places, within a hero or between two met heroes (§5.2), with the Spell Book,
  the Catapult and the machines' rules; an assembly at a worn part and a disassembly of a worn
  combination (§5.3). The army dialog's commands are the same stack commands between the hero and
  the decision's group, plus closing it.
- **Decisions** (ADR-0017 point 6): the army dialog as a pending decision of the visitor's owner
  (the group; closed by an answer, the rest lost: §1), which closes `ADV-STUB-2`; the meeting
  either as a decision of the owner (the two heroes; the exchange's commands legal while it is
  open, closed by an answer) or as a lasting state (the heroes may exchange while they stand side
  by side); the original's is the first: the screen exists only during the meeting, the next one
  needs another step (§12, question 1). The once-per-player assembly question and the question
  at a received part become the assembly command, always legal when the parts are worn (a
  computer player's build at once: E10).
- **What another player sees**: nothing (§0). Each change is the owner's: an army changed, an
  artifact moved, a spell learned (Scholar), the formation or the flag set; the hero's details
  changed are `HeroChanged` as after battles (`docs/06-security.md`, increment 3). A meeting's
  step is a step (its viewers as today); the meeting itself is private.
- **The env** (ADR-0017 point 8, provisional until E13's macro-actions): new entries for the
  formation and the flag (per hero slot), the stack commands (a hero slot, a source slot, a
  target hero of the meeting or the same, a target slot), a split's count (a head of fractions,
  or a few amounts: one, half, all but one), the dismissal, the artifact commands (a source and a
  target among 19 positions and the backpack's places), assembly and disassembly, the decisions'
  closing; or a subset now (the formation, the flag, the stack moves without a count, the army
  dialog) with the rest left to `h3_game::legal_commands` and the replays (§12, question 7).
- **Replays and benchmarks**: the new commands in the adventure replay format (v6), a replay of a
  meeting with Scholar and an exchange and one of the army dialog; the store of heroes is behind
  an `Arc` per hero (a write copies one hero), the armies are cloned with the heroes (T14b: 15 %
  of `fork-xl-144`): a benchmark of a turn of management on the XL map, and `fork-xl-144` with an
  open meeting.

## 7. The census

`original-data` (the 160 shipped maps; `h3-map`'s reader, a throwaway example): 373 placed heroes
and 832 custom hero records (none with artifacts). **Artifacts:** 40 placed heroes have custom
artifacts; 11 a backpack of one or two (Knee Deep in the Dead, Pandora's Box twice, Race for
Ardintinny four times, Sangraal's Thief and its allied version: the Grail, Thousand Islands and its
allies version: two each); none holds a combination, a full set of a combination's parts, Titan's
Thunder, or a war machine or the book in the backpack. **Armies:** 3 heroes have seven stacks (a
join there opens the army dialog); 8 hold two stacks of one creature; no count reaches 32 768.
**Formation:** one tight hero (Unholy Quest's, known from T17a). So the shipped maps exercise
little of increment 6 at the start; play does (joins, meetings, captures, later rewards).

## 8. Disputes and quirk candidates

| Id | What | Sources | Proposal |
|---|---|---|---|
| D7 | the backpack's capacity (HER-ART-002) | code 64, wiki 64; VCMI unlimited | the original's 64 (as HER-ART-002 has it) |
| D-27 | the split's bounds: the original keeps one creature on the source side when the target hero has a single stack (A-16) | code; VCMI (`minLeft` only for the source's last stack); wiki silent | see A-16 |
| D-28 | disassembling a combination in the backpack | code: worn only; VCMI: in the backpack too, with room for the parts; wiki: "right-clicking" a worn one | the original's (worn only) |
| A-15 | `army_swap_slots` moves a count through 16 bits (§3.5) | code; VCMI 32 bits | state-corrupting: fixed in every profile (recorded in the ledger as fixed) |
| A-16 | the split dialog's source-keeps-one when the target hero has one stack (§3.3) | code | harmless (a plain click moves the stack whole); (a) the original's bounds in every profile, (b) a flag, faithful the original's, strict-fixed VCMI's, (c) VCMI's bounds; recommended (a) |

Not quirks: Titan's Thunder's book (the original's rule, §5.4); the held artifact given to the
visitor when the exchange closes and the once-per-player assembly question (screens, gone with
atomic commands, §2.3, §5.3); the tactics flag read for human players only (the engine has no
computer player, decision 16: every hero's flag counts; E10 decides).

## 9. Batch E6's plan

Increment 6's batch, **batch E6**, runs on the client lane's harness in SoD on Easy, hot seat,
from a map `h3-experiment` writes (`docs/experiments/README.md`), as batches E3 to E5. Its spec and
recipe are T44b's cut's tasks; this is the plan. Each row names what it settles and what it records.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E6-1 | §1 (a check of the dialog) | a hero of seven stacks next to a friendly group of an eighth creature (aggression 1, joins for free) | the question, then OK without a change; the army after, the monster's tile, the treasury | the join asked; the dialog's title record GENRLTXT 36; the army unchanged, the monster gone |
| E6-2 | §1.1 point 3 | the same, then: dismiss a stack in the dialog, take the group into the free slot | the army after | the group in the freed slot |
| E6-3 | §3.2 (last stack) | two heroes side by side; A with one stack, B with two: A's stack to B's empty slot, then onto B's stack of its creature, then onto B's other stack | the two armies after each click | refused, refused, swapped |
| E6-4 | A-16, D-27 | A with two stacks, B with one: the split dialog from A's stack into B's empty slot, the slider at its maximum | the counts after | A keeps one creature |
| E6-5 | A-15 | a hero with a stack of 40 000 (the writer's count is 16 bits: 40 000 fits) moved to an empty slot in the hero screen | the count after (memory and screen) | −25 536 |
| E6-6 | §4 | a hero with Advanced Tactics against a monster (no hero), the tactics formation off; again with it on | whether the battle starts in the tactics phase (battle manager +0x13D68) | no phase, then a phase |
| E6-7 | §2.2 | A with Expert Scholar and Expert Wisdom knowing a level-4 and a level-3 spell, B with no Wisdom and a level-1 spell, both with books, meet | both heroes' learned spells after | B learns the level 2 limit only (none of A's 3, 4); A learns B's level 1 |
| E6-8 | §5.3 (once per player) | a hero wearing all parts of a combination but one, the last in the backpack: equip it; decline; unequip and equip again; then right-click a part, accept | the questions (records), the worn positions | GENRLTXT 733 at the first equip only; the right-click asks; the combination at the first position that takes it |
| E6-9 | §5.4 | a hero without a book with Titan's Thunder in the backpack: equip it, then unequip it | the book position after each | the book after both |
| E6-10 | §5.1 (the rotation) | a hero with six backpack artifacts: one click on the right arrow, close the screen | the backpack's order in memory | rotated by one |
| E6-11 | §2.1 (the price) | A meets B from a straight grass step | A's points before and after, A's tile | 100 less, A on its tile |

**What the harness lacks** (`clients/capture/` is a common path since the client lane's split,
CLT-Q18: this lane builds it with the harness check and tells the client and map-client lanes'
inboxes): clicks on the hero screen's army slots and buttons (split,
formation, tactics), the creature dialog's dismiss and its confirmation, the split dialog's slider
and OK, right-clicks on artifacts, the backpack's arrows, the exchange screen (its armies, worn
positions, backpack places; closing), the army dialog's bars and OK; reads of a hero's learned
spells (+0x3EA), its backpack in order (+0x1D4), its flags (+0x48), the battle manager's tactics
flag; the question's record for the assembly. **What the spec lacks** (`h3-experiment`,
battle-owned: `cross-lane`): two heroes of one player side by side, a hero's learned spells and
Scholar, artifacts in the backpack in an order, a stack of 40 000, a friendly group next to a full
army (batch E4 had joins).

## 10. The proposed rule ids

For T44b's cut; ids follow the catalogs' prefixes, each written when its rule is built.

| Id | Rule | Survey | Open points |
|---|---|---|---|
| ADV-ARM-001 | a hero's army: up to seven stacks, at least one; within a hero a stack merges into one of its creature, moves to an empty slot or swaps | §3.1 | — |
| ADV-ARM-002 | between two met heroes: the same, the source's last stack never merging away nor moving to an empty slot | §3.2 | — |
| ADV-ARM-003 | the split: the target's new count within the dialog's bounds | §3.3 | A-16 |
| ADV-ARM-004 | dismissing a stack, never a hero's last | §3.4 | — |
| ADV-ARM-005 | a move or a swap keeps the counts whole | §3.5 | A-15 (fixed) |
| ADV-ARM-006 | the formation and the tactics flag: set by their owner at no cost; every hero starts loose with the flag on | §4 | — |
| ADV-MON-008 (amended) | a join that finds no room opens the army dialog (a decision); the group's rest is lost; the treasure and the monster's leaving as any join | §1 | the decision's shape |
| GAM-MET-001 | a step toward an own hero visits it from the neighbouring tile at the step's price and opens the meeting | §2.1 | the meeting's shape |
| HER-SPL-007 | Scholar at a meeting | §2.2 | — |
| HER-ART-005 | moving artifacts: a position of the kind (with the locks), the backpack (insertion, 64, never a machine or the book), the book and the Catapult never | §5.1, §5.2 | positions |
| HER-ART-006 | combinations (SoD): assembly with every part worn, disassembly of a worn one | §5.3 | D-28; the game's version |
| HER-ART-007 | Titan's Thunder gives a Spell Book, kept | §5.4 | — |
| GAM-BAT-0xx (amended BAT-TAC-001's input) | a side's tactics phase off when the hero's flag is off | §4 | the seam (§12, question 4) |
| GAM-DEC-00x | the army dialog's and the meeting's decisions | §6 | — |

## 11. The second reading

A fresh agent re-read the deciding code without this survey (2026-10-23), from ten questions,
and **confirmed §1 to §5 point by point**: the army dialog's two armies, title and lost rest, its
last-stack refusals with their status texts and the allowed swap (§1.1); the exchange's selection,
merge, swap and move rules and the single-stack refusals (§3.2); `army_swap_slots`' 16-bit count
(A-15, §3.5); the split dialog's bounds with both minima, A-16's included, and the flags the two
screens pass (§3.3); the dismiss offered only above one stack, cleared without a refund (§3.4);
the buttons 77, 78, 122, 124, 126, 127, the flags' default 2 and the tactics phase cleared for a
human side whose hero's bit 1 is clear (§4); the artifacts that never move, the drop tests, the
backpack's refusals and insertion, the rotation, the held artifact given back on closing (§5.1,
§5.2); the combinations' questions, the bit set +0xE8, the build and the disassembly from worn
only (§5.3); Scholar's levels, requirements and both directions (§2.2).

**What it adds** (written into the sections above): a held artifact the exchange gives the
visitor on closing is lost when the visitor has no room (§2.3); `hero_give_artifact`'s question
only for an equipped part, and the bit set either way (§5.3); a disassembled part that found no
position would be lost (none can, §5.3); +0x53C5 probably a creature bank's battle (§4). Also:
in the exchange the drops into an empty slot, position or backpack place and the right-click's
assembly and disassembly skip the target hero's owner (teams, E10); a campaign's check ("Only
Gelu can carry Armageddon's Blade", GENRLTXT 762) tests the target position's worn artifact
(campaigns, not built); the screen's view-only global 0x6AA9D8 is set by the town's sub-screens
(not read). Not traced by it either: 0x5C8080, the town garrisons' modes 7 and 8, the reward
callers of the army dialog.

## 12. For T44b: the checkpoint's questions (ADV-Q24)

*Answered (2026-10-23, the epic's decisions 93–100):* 1 (a); 2 (a); 3 (a); 4 (b) with the user's
condition that the two heroes' Tactics keep cancelling as the experiments confirmed, so both go
to the battle as 0 when the side that would get the phase has the flag off (decision 96); 5 in;
6 A-15 fixed, A-16 (a); 7 neither: **the adventure's RL encoding goes to the AI lane** (decision
99); 8 all eleven rows; 9 stands.

1. **The meeting's shape**: (a) a pending decision of the owner opened by the step, the exchange's
   commands legal while it is open, closed by an answer (the original's: one meeting per step,
   Scholar at its start); (b) the heroes may exchange whenever they stand side by side (no
   decision; a step only for Scholar). Recommended (a).
2. **The army dialog's shape**: (a) a pending decision with the group as a small army: the stack
   commands between the hero and the group and a closing answer (the original's); (b) one answer
   that names what to dismiss and where the group goes. Recommended (a).
3. **The artifacts' positions**: (a) the original's 19 positions in `h3-heroes` (`HeroDetails`
   and HER-ART-001 change: commands name positions, "the first position that takes it" and the
   capture follow the original); (b) keep the kinds (positions interchangeable), the capture's
   order the list's. Recommended (a) (ADR-0013 amendment).
4. **The tactics flag**: in increment 6 (a hero-screen button), with a seam for the battle: (a) a
   field of the battle's setup per side (cross-lane, the battle and server lanes: ADR-0018 point
   5's mapping), (b) the game passes the hero's Tactics as 0 to the battle when the flag is off
   and its side would have the phase (no seam change; equal outcome in SoD, since the original
   only clears the phase of the side that has it); or later. Recommended (b), the cheapest.
5. **Scholar** with meetings: in (recommended) or later.
6. **A-15** fixed in every profile (confirm); **A-16**: (a), (b) or (c) of §8 (recommended (a)).
7. **The env's coverage**: every command with its heads now (the split's count as fractions), or
   a subset (the formation and the flag, the stack moves without a count, the decisions' answers)
   with the rest left to `legal_commands`. Recommended: the subset, the rest when E13's
   macro-actions come.
8. **Batch E6**: the rows of §9 (all, or which), the harness's and the spec's extensions
   (`cross-lane`).
9. Out of increment 6 (confirm): allied heroes' meetings and one-way trades (teams, E10), the
   computer player's join without room and meetings (E10), hero dismissal (E09), the creature
   dialog's upgrade (E09), the game's version for combinations (an RoE or AB map: no assembly;
   with the game's version, as the moat's, E04c's hand-on).
