# E08 teleporters survey (E08 T98a): Subterranean Gates, Monoliths, Whirlpools

Date: 2026-11-05. Scope: E08 increment 11 (decisions 260, 261), input for T98b's ADR amendment
drafts, the design note and the cut. In two parts (the user at T98a's start, decision 262):
**T98a1** (this part, §0 to §8: the census, the objects, the channels, the jump, the arrival, the
move around a jump, VCMI and the wiki, the disputes) and **T98a2** (§9 to §14: the reach and the
paths, what a player sees, the interfaces, batch E11's plan, the proposed ids, the checkpoint).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-11-05):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_object` 0x4A84F0 at its cases 43, 44, 45, 103 and 111;
  `visit_monolith_one_way` 0x4A8230, `visit_monolith_two_way` 0x4A8390, `teleport_exit_draw`
  0x4CDB80 and its three callers 0x4CDDC0, 0x4CDDF0, 0x4CDE20, `gate_exit` 0x4CDE40,
  `gates_pair` 0x4C0B60, `whirlpool_losses` 0x4ACAA0. **Read in part:** `setup_map_objects`
  0x4C0CC0 at its cases 44, 45, 103 and 111; `hero_relocate` 0x41D930 (its state: the tile, the
  reveal, what it does not touch; not its drawing); `hero_move_loop` 0x407B80 around the visit;
  `adventure_key_down` 0x408C40 at the space key's visit (0x408E65). **Not followed:** the computer
  player (its routes through teleporters are E10's), network play (`map_message_send` 0x482390,
  the `SendData` calls), `hero_relocate`'s call 0x49CF50 (T98a2, what a player sees), the
  pathfinder's use of the lists (T98a2), the sounds and pictures.
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`), `lib/mapObjects/MiscObjects.cpp` (`CGTeleport`, `CGMonolith`,
  `CGSubterraneanGate`, `CGWhirlpool`), `server/CGameHandler.cpp` (`moveHero`),
  `lib/mapObjects/army/CStackInstance.cpp` (`getPower`), `lib/mapping/MapFormatH3M.cpp`
  (`readObject`'s default case, `readGeneric`).
- `wiki`: <https://heroes.thelazy.net/index.php/Whirlpool>,
  <https://heroes.thelazy.net/index.php/Subterranean_Gate>; the wiki's Monolith page through its
  mirror <https://heroes.shoutwiki.com/wiki/Lith> (a search engine's copy; the page refused a
  direct fetch).
- The census (§1): a throwaway example over `h3_map::read` on the 160 shipped maps (not
  committed).

## 0. Findings in short (T98a1)

- **On 146 of the 160 maps** (RoE 41, AB 51, SoD 54): 602 gates on 98 maps (301 on each level,
  every pair found), 193 one-way entrances and 157 exits on 59 maps (111 channels, 22 of them with
  several exits, none without an exit), 495 two-way monoliths on 98 maps (219 channels, 34 of three
  or more, none alone), 224 Whirlpools on 62 maps (two or more on each). No body in the file; no
  entrance shared with another object.
- **Channels** are built at a new game (§3): a list per one-way subclass (the exits only), per
  two-way subclass, one for every Whirlpool tile (six per Whirlpool, subclass ignored), and the
  gates paired greedily in the walk's order with the nearest unpaired gate of the other level by
  truncated Euclidean distance.
- **The jump** (§4): a monolith's or Whirlpool's exit is **drawn** uniformly among the candidates
  (`random_between(1, n)`, one draw even for one candidate); a candidate is any other tile of the
  channel unless a hero of the mover's team stands on it. An enemy on a monolith's or gate's exit is
  fought first; a hero of the team on a gate's exit is met and the mover stays. A Whirlpool costs a
  human's hero half of its weakest stack (fight value × count, rounded down kept) unless it wears
  the Sea Captain's Hat (alone or in the Admiral's Hat); a computer's hero loses nothing.
- **The arrival** (§5) keeps the points, fights no guard, visits nothing and reveals the exit with
  the hero's scouting radius. **A walk** (§6) ends at the entrance; nothing goes on past the exit.
- **Disputes** D-53 to D-57 (a path's exit in VCMI, the weakest stack's measure and tie, the loss on
  odd counts, the gates' tie, the Whirlpool's two draws); **quirk candidates** A-37 to A-40 (a
  Whirlpool lands on an enemy hero, the 99 999 999 bound, a subclass past 7, the computer's
  exemption). §8.

## 1. The census

The 160 maps of the installation's `Maps` (`maps.tar`), read by `h3_map::read`; an object's tiles
from its template in the map (bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i`
rows above the anchor, `h3m.md`).

| Class | Object | Objects (maps) | By subclass: objects (surface + underground) |
|---|---|---|---|
| 103 | Subterranean Gate | 602 (98) | 0: 301 + 301 |
| 43 | Monolith One Way Entrance | 193 (59) | 0: 39 + 9, 1: 25 + 19, 2: 20 + 6, 3: 5, 4: 34, 5: 19, 6: 4 + 6, 7: 7 |
| 44 | Monolith One Way Exit | 157 (59) | 0: 30 + 6, 1: 30 + 19, 2: 17 + 7, 3: 3, 4: 18 + 2, 5: 13, 6: 4 + 2, 7: 5 + 1 |
| 45 | Two-Way Monolith | 495 (98) | 0: 128 + 44, 1: 71 + 37, 2: 69 + 31, 3: 18 + 2, 4: 31 + 3, 5: 17 + 3, 6: 20 + 6, 7: 10 + 5 |
| 111 | Whirlpool | 224 (62) | 0: 215 + 9 |

- **Versions:** RoE 41 maps, AB 51, SoD 54. Subclasses 3 to 7 only on SoD maps (SoD's
  `Objects.txt` lists 0 to 7; RoE's and AB's 0 to 2): 75 of SoD's 83 one-way entrances, 48 of its
  56 exits, 115 of its 216 two-way monoliths.
- **Levels:** all 98 maps with gates have two levels (104 maps have two, T97c); 46 of the 330
  monolith channels span both levels; 9 Whirlpools lie underground.
- **One-way channels** (a map's entrances and exits of one subclass), entrances → exits: 1 → 1 71,
  2 → 1 8, 3 → 1 7, 4 → 1, 5 → 1, 7 → 1 one each; 1 → 2, 1 → 5, 1 → 7 one each; 2 → 2 4, 3 → 2 2,
  3 → 3 8, 4 → 4 3, 8 → 3 2. **22 of 111 have several exits** (a real draw); none lacks an exit or
  an entrance.
- **Two-way channels** by size: 2 185, 3 19, 4 11, 5 2, 6 1, 8 1 (**34 of 219 with a real
  draw**); none alone.
- **Whirlpools** per map: 2 on 35 maps, 3 on 8, 4 and 5 on 5 each, 6 on 4, 7 on 1, 8 on 2, 19 on 2;
  all six visitable tiles of every Whirlpool on water (1 344 tiles), subclass 0 only.
- **Gates' pairs** by the original's rule (§3.2): every gate paired (301 pairs); no pair decided by
  a tie at the truncated minimum, so VCMI's rule gives the same pairs on every map (D-56); **18
  gates on 8 maps get a gate that is not their nearest** (the greedy order): Divided Loyalties and
  its allied version 2 each, Sangraal's Thief and its allied version 3 each, Terrible Rumor 3,
  The Newcomers 1, War of the Mighty and its allied version 2 each.
- **Entrance terrains:** gates on every land terrain (subterranean 229, dirt 88, grass 83, sand
  53, lava 46, rough 43, snow 36, swamp 24); monoliths on land only (six templates allow water,
  none stands on it); Whirlpools on water only.
- **Clean inputs:** no teleporter entrance is the visitable tile of another object or under another
  object's blocked tile, none lies off the map, none has a body.

## 2. The objects in the map file and at load

- **Classes and subclasses:** 43, 44, 45 by colour 0 to 7 (the channel), 103 and 111 subclass 0.
  One template per (class, subclass) in `Objects.txt`: a monolith (`AVXmn*`, `AVXmx*`) is one
  visitable tile, at the anchor with one blocked tile (subclasses 0 to 3) or one left of it with two
  (4 to 7); a gate (`AvTCave`) visits one left of its anchor with four blocked; a Whirlpool
  (`AVXwhrl0`) is a 3 × 2 block of six tiles, all blocked and visitable, on water only. **The maps'
  templates rule** (the import reads them, as for every object): 42 monoliths on SoD maps carry a
  template of the same picture that blocks one tile where `Objects.txt` blocks two.
- **No body:** `h3m.md`'s plain objects; VCMI's reader takes them through `readObject`'s default
  case (`readGeneric`). The import leaves them inert blockers today (T97c's census of the classes
  still left blockers).
- **Entered**, not visited from the next tile: none of the five is in
  `object_classes_not_entered` 0x63AA58, so the hero steps onto the visitable tile (E08-objects.md
  §2.1). **From above:** only the Whirlpool is in `object_classes_from_above` 0x63AB00; a monolith
  or gate is not entered from (or left toward) the three tiles above its visitable tile.

## 3. The channels at a new game

### 3.1 The lists (`setup_map_objects` 0x4C0CC0, its cases 44, 45, 103, 111)

The setup walks every tile that is an object's entrance, level by level, row by row, column by
column, and fills its class's tile word (`map_item` +0x0). The lists are vectors of packed
positions in the game record:

- **44** (0x4C1653): the word is the tile's index in the one-way exits' list of its subclass
  (game +0x4E6FC + 16 × subclass); the position appended.
- **45** (0x4C174C): the same in the two-way list of its subclass (+0x4E67C + 16 × subclass).
- **43**: no list (the default case); the draw takes the exits of its subclass (§4.1).
- **103** (0x4C16C1): the word is the gate's index in the gates' list (+0x4E78C); the position
  appended, and −1 to the pairs' list (+0x4E79C).
- **111** (0x4C1F88): each of the six visitable tiles is appended to the one Whirlpools' list
  (+0x4E77C), whatever the subclass; the six share one word, a counter of Whirlpools set on the tile
  whose cell is (2, 1) in the object and copied by the others from it.
- The subclass indexes eight lists of 16 bytes without a bound: a subclass from 8 writes past them
  (A-39).

### 3.2 The gates' pairs (`gates_pair` 0x4C0B60, read whole; from `new_game_heroes` 0x4C0005)

In the list's order (the walk's: level, row, column), each gate `i` still unpaired takes the
unpaired gate `j > i` on another level with the least `trunc(sqrt(dx² + dy²))` (`crt_sqrt` on
f64, `crt_ftol`; `dx`, `dy` the entrances' columns and rows), the first on a tie, and both are
marked. A gate left over keeps −1. A map has two levels at most, so a surface gate pairs with an
underground one while one is left; with more gates on one level the extra ones keep −1 (none
shipped, §1). The truncation equals an integer square root (exact for map sizes),
so the engine needs no float (ADR-0003).

## 4. The jump (`visit_object` 0x4A84F0, cases 43, 44, 45, 103, 111; read whole)

### 4.1 The draw (`teleport_exit_draw` 0x4CDB80, read whole)

Arguments: a list, the out position, a class, a word to exclude. Over the list in order, a
**candidate** is a tile with the entrance bit (`map_item` +0xD 0x10) and either the list's class
and a word other than the excluded one, or a hero (class 34) whose owner is not on the current
player's team (game +0x1F879; an owner or current player below 0 counts as not). None: false.
Else `r = random_between(1, n)` (0x50B230: `1 + rand() mod n`, the C runtime's one stream, **drawn
even when `n` is 1**) and the `r`-th candidate in the list's order.

- **One-way** (0x4CDDC0): the exits' list of the entrance's subclass, class 44, nothing excluded.
- **Two-way** (0x4CDDF0): the two-way list of the subclass, class 45, the entrance's word excluded.
- **Whirlpool** (0x4CDE20): the Whirlpools' list, class 111, the entrance's word excluded (its six
  tiles). Each other Whirlpool offers each of its six tiles: the object and the tile come from one
  draw.
- The hero stands on the entrance during the visit (it is entered), so the entrance's own tile is
  a hero of the mover's team and never a candidate in any case.

### 4.2 The monoliths

- **43** (`visit_monolith_one_way` 0x4A8230): the one-way draw by the tile's subclass (+0x22);
  none: nothing at all (no message; the hero stays on the entrance). Else, when a hero stands on
  the exit (an enemy, by §4.1): in a network game a message; `reveal_disk` 0x49D160 at the exit
  for the current player with radius 1; `visit_hero` 0x4A2800 (the battle at once, the mover
  attacking); the mover lost (owner below 0): the end. Then, the exit free or won:
  `hero_walk_end` 0x47F7D0 (the walk's animation) and `hero_relocate` 0x41D930 to the exit with
  the sound `telptout.wav` (§5).
- **45** (`visit_monolith_two_way` 0x4A8390): the same with the two-way draw.
- **44** (0x4A9538): a human's visit shows ADVEVENT 70 (the gate that no longer works); a
  computer's does nothing.

### 4.3 The Subterranean Gate (0x4AA35E; `gate_exit` 0x4CDE40)

`gate_exit` takes the pair of the entrance's word (+0x4E7A0); −1, or a pair whose tile is neither a
gate (103) nor a hero (34, with the entrance bit), gives none. **None:** ADVEVENT 153 (the blocked
tunnel's text), shown without the human test. **A hero on the exit, of any team:**
the radius-1 reveal and `visit_hero` (the meeting with a hero of the team, the battle with an
enemy); the mover lost: the end; then the move happens **only if the exit's tile is a gate again**
(0x4AA4A6: the enemy beaten, never after a meeting). No draw, no `hero_walk_end`, no sound.

### 4.4 The Whirlpool (0x4A95A5; `whirlpool_losses` 0x4ACAA0)

First the losses, then the Whirlpool draw; none: the hero stays, the losses taken (the wiki: a lone
Whirlpool only takes troops). Else `hero_walk_end` and `hero_relocate` to the
drawn tile, no sound, **without testing for a hero on it** (A-37).

**The losses**, only when the hero's owner is human (`player_id_is_human` 0x4CE940; A-40) and the
hero does not wear artifact 123, the Sea Captain's Hat, alone or in a combination (the Admiral's
Hat holds it; `hero_wears_artifact` 0x4D91F0 looks into combinations):
- **The weakest stack:** the least creature fight value (creature record +0x3C) × count (an `i32`
  product), starting from 99 999 999 and strictly less (the **first slot** on a tie); empty slots
  skipped.
- **The loss:** with more than one stack (`army_stack_count` 0x44ACC0) its count becomes `count >>
  1` (a stack of 1 emptied, its type −1); with one stack only when its count is above 1 (a lone
  creature is kept).
- **The message:** ADVEVENT 168 (a plain message, no picture), also when a lone creature lost
  nothing.
- **No stack below the bound** (A-38): the slot stays −1; with one stack the count read there is
  the seventh slot's type (−1: nothing happens, the wiki's "bug" of 1 776 Azure Dragons), with
  several the seventh type is halved and, at 0, the word before the types (hero +0x8D) set to −1.

### 4.5 An occupied or guarded exit

| The exit holds | A monolith (43, 45) | A Whirlpool tile | A gate |
|---|---|---|---|
| a hero of the mover's team | not a candidate | not a candidate (that tile) | the meeting; the mover stays |
| an enemy hero | a candidate; the battle; the mover moves if it wins | a candidate; moved onto it (A-37) | the battle; the mover moves if it wins |
| nothing | moved | moved | moved |

**Guards** (a monster next to the exit) never fight on arrival: `hero_relocate` runs no guard test
(§5). A one-way channel whose exits all hold heroes of the team leaves the hero on the entrance
with no message.

## 5. The arrival (`hero_relocate` 0x41D930, read in part)

Every teleport calls it with (hero, exit, sound, 0, 1, 0). With its fourth and sixth arguments 0 it
first sends the network message (`map_message_send`) and calls 0x49CF50 (not read: T98a2, what a
player sees). Then it hides the hero (`hero_hide` 0x4D76E0), writes its tile (hero +0x0, +0x2,
+0x4), reveals around the exit for the current player's team with **the hero's scouting radius**
(`hero_scouting_radius` 0x4E4070, `reveal_disk`), shows the hero (`hero_show` 0x4D75D0), plays the
sound and redraws (selection, minimap).

It touches **neither the points** (+0x4D) nor the day's maximum (+0x49), runs **no guard test**
(`after_step_guard_check` runs only at a step's end, 0x4800A5) and **visits nothing** at the exit
(a two-way exit is not re-entered). The fog: the step onto the entrance revealed around the
entrance (the step's own reveal); the exit gets the scouting radius; an occupied exit gets radius 1
before the battle (§4.2).

## 6. The move around a jump

- **The step onto the entrance** is priced and paid as any step (ADV-MOV), its drop to 0 below the
  cheapest next step tested at the entrance (`hero_move_step` 0x481182), not again at the exit.
- **A human's walk ends at the entrance:** `hero_move_loop` 0x407B80 clears the path once the hero
  stands on its destination (0x4080AE), ends the walk (0x4080D2) and only then visits
  (`visit_and_update` 0x4AAAA0 at 0x4080E9); a human's path never goes past a stop tile
  (`pathfinder_pass_through` with its fourth argument 0, E08 T55a4). Nothing goes on after the
  jump: the hero stands on the exit with its points.
- **The pathfinder reads the monolith lists** (`pathfinder_pass_through` at 0x56AE39 and 0x56AE81,
  the one-way and two-way lists): whose paths those are and how the reach treats a teleporter is
  T98a2's (§9); the computer's routes are E10's.
- **The space key** visits the object under the hero again (`adventure_key_down` 0x408C40 →
  `visit_and_update` at the hero's own tile, 0x408E65, when that tile has the entrance bit and its
  class is not 3; read in part). The wiki: the space bar takes a hero through a gate again without spending
  movement points, likewise a two-way monolith (not in RoE) and a Whirlpool's square (AB and SoD). Its full gate and whether increment 11 takes it as a
  command: T98a2 (§11).
- **After the jump** the hero stands on the exit object; stepping off and on again visits it again
  (a two-way monolith or a gate leads back, a one-way exit shows ADVEVENT 70).

## 7. VCMI and the wiki at a glance

| Point | Original | VCMI 1.7.5 | Wiki |
|---|---|---|---|
| A monolith's channel | its class pair and subclass | the same (`findMeChannel`) | the colour |
| The exit | drawn among the candidates | drawn; an exit named by the answer is taken (D-53) | random among several |
| An exit with a hero of the team | not a candidate; a gate: the meeting | the same (`isExitPassable`) | the same |
| An exit with an enemy | the battle, the winner moves | a blocking visit (the battle) | a battle |
| The gates' pairs | greedy, walk order, truncated distance, first on a tie | greedy, sorted by position, squared distance, first on a tie (D-56) | nearest; a tie north, then west |
| A gate without a pair | ADVEVENT 153 | the same | the same message |
| A one-way exit visited | ADVEVENT 70 (a human) | the same | — |
| The arrival | points kept, no guards, no visit, the scouting reveal | `TELEPORTATION`, `IGNORE_GUARDS`, `DONT_VISIT_DEST`, points kept | — |
| A Whirlpool's exit | one draw over the other Whirlpools' tiles | a Whirlpool, then one of its blocked tiles (D-57) | random; six squares to exit |
| Whose troops a Whirlpool takes | a human's hero (A-40) | every hero | — |
| Protection | the Sea Captain's Hat, alone or in the Admiral's Hat | `WHIRLPOOL_PROTECTION`, a lone creature | either Hat |
| The weakest stack | fight value × count, the first on a tie | AI value × count, the last on a tie (D-54) | total fight value |
| The loss | keeps `⌊n / 2⌋`; a lone creature kept | takes `max(1, ⌊n / 2⌋)` (D-55) | 50 % rounded up lost |
| The space key | visits the object under the hero (in part) | — | AB and SoD; two-way monoliths not in RoE |

## 8. Disputes, quirk candidates and open points

- **D-53 (an exit chosen).** VCMI's server takes an exit named by the client's answer
  (`CGMonolith::teleportDialogAnswered`: `exits[answer]`; every exit of a two-way channel, and every
  Whirlpool tile to a protected hero, is offered), which its client fills from a
  path through the channel; the original always draws. Gameplay: whether a player picks the exit of
  the 34 two-way channels of three or more and of the Whirlpools. Recommended: **the original**
  (drawn; a command never names the exit, ADR-0004).
- **D-54 (the weakest stack).** The original: least fight value × count, the first slot on a tie;
  VCMI: AI value × count (`CStackInstance::getPower`), the last slot on a tie (a reverse scan with
  a strict test). The wiki: total fight value. Recommended: **the original**.
- **D-55 (what a Whirlpool takes).** The original keeps `⌊n / 2⌋` (a stack of 3 keeps 1; a stack of
  1 among others is lost; a lone creature kept, the message shown); VCMI takes `max(1, ⌊n / 2⌋)` (a
  stack of 3 keeps 2) and gives a lone creature no message. The wiki: "loses 50% rounded up (i.e. is
  left with 50% rounded down)". Recommended: **the original**.
- **D-56 (the gates' tie).** The original compares truncated distances, VCMI squared ones; both
  greedy in the same order, first on a tie. No shipped map has a tie at the truncated minimum (§1),
  so the pairs agree on all 98 maps; a map built for it differs. The wiki's tie (north, then west)
  is the walk's order. Recommended: **the original** (truncated).
- **D-57 (a Whirlpool's exit tile).** The original: one draw over every other Whirlpool's six tiles,
  a tile with a hero of the team skipped; VCMI: a Whirlpool (skipped when its visitable tile holds a
  hero of the team), then a second draw among its blocked tiles. Equal odds when nothing stands on
  them. Recommended: **the original**.
- **A-37 (quirk candidate). A Whirlpool lands on an enemy hero.** The draw counts an enemy's tile
  (§4.1) and the Whirlpool's case has no hero test: `hero_relocate` puts the mover onto the other
  hero's tile, two heroes on one tile. Reachable in play (a hero ends its move on the exit
  Whirlpool it came out of). **State-corrupting: fixed** (non-negotiable); how is the checkpoint's
  question (T98a2): the battle as at a monolith (the original's own pattern for 43, 45, 103;
  recommended), the tile not a candidate (as a hero of the team), or VCMI's (the mover stays, no
  battle).
- **A-38 (quirk candidate). The weakest stack's bound.** A stack whose fight value × count reaches
  99 999 999 (or overflows `i32` to a negative product, which then is the weakest) is mis-ranked;
  when no stack is below the bound, one stack loses nothing and several corrupt the seventh slot's
  type and hero +0x8D (§4.4). The corrupting case is **fixed**; whether a lone stack past the bound
  keeps its troops (the wiki's "bug") is the checkpoint's question. Recommended: no bound (the least
  of all stacks, a 64-bit product).
- **A-39 (quirk candidate). A monolith subclass past 7** overruns the eight lists (§3.1). None
  shipped; the editor offers 0 to 7. Recommended: **refused at the import** (an input bound, as
  A-34 to A-36), no flag.
- **A-40 (quirk candidate). A computer player's hero loses nothing in a Whirlpool** (0x4ACAB3;
  players' reports agree, the wiki is silent). The engine has no human or computer player
  (decision 16). Recommended: **not reproduced**, no flag: every hero loses as a human's, as A-27's
  Ocean Bottle (decision 166); revisited with E10.
- **Open points:** the gate's ADVEVENT 153 has no human test (a message, nothing for the engine);
  the space key's full gate (§6) and 0x49CF50 (§5) are T98a2's; the draw's stream (the original's
  one `rand`; ADR-0017's streams) is T98b's; batch E11 (T98a2) should observe the points that rest
  on the reading alone: the draw's uniformity, A-37, the gate's meeting without a move, the guards
  silent at an exit, the points kept, the lone creature's message.

Next ids: disputes D-58, reading quirk candidates A-41.

## 9. The reach and the paths through a teleporter (T98a2)

## 10. What a player sees (T98a2)

## 11. Commands, decisions, events, the observation, replays and benchmarks (T98a2)

## 12. Batch E11's plan (T98a2)

## 13. The proposed rule ids (T98a2)

## 14. For T98b: the checkpoint's questions (T98a2)
