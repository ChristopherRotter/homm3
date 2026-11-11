# E08 teleporters survey (E08 T98a): Subterranean Gates, Monoliths, Whirlpools

Date: 2026-11-05. Scope: E08 increment 11 (decisions 260, 261), input for T98b's ADR amendment
drafts, the design note and the cut. In two parts (the user at T98a's start, decision 262):
**T98a1** (§0 to §8: the census, the objects, the channels, the jump, the arrival, the move
around a jump, VCMI and the wiki, the disputes) and **T98a2** (§9 to §14: the reach and the paths,
what a player sees, the interfaces, batch E11's plan, the proposed ids, the checkpoint; it also
corrected §4.1's draw for one candidate and added D-58 and A-41 to §8).
Input, not rules (`README.md`): a fact becomes binding when a task writes it into the catalog with
its sources and tests.

**Sources** (retrieved 2026-11-05):
- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete); every address
  cited has an entry in `docs/exe/map.toml` or `structs.toml`. The reading follows the human
  player's path. **Read whole:** `visit_object` 0x4A84F0 at its cases 43, 44, 45, 103 and 111;
  `visit_monolith_one_way` 0x4A8230, `visit_monolith_two_way` 0x4A8390, `teleport_exit_draw`
  0x4CDB80 and its three callers 0x4CDDC0, 0x4CDDF0, 0x4CDE20, `gate_exit` 0x4CDE40,
  `gates_pair` 0x4C0B60, `whirlpool_losses` 0x4ACAA0; T98a2: `random_between` 0x50B230,
  `pathfinder_pass_through` 0x56AB40 at its cases 43, 45, 103 and 111, `adventure_key_down`'s
  space key (0x408CC4 to 0x408E75), `hero_relocate_inner` 0x49CF50, `turn_records_drop` 0x49D6C0.
  **Read in part:** `pathfinder_teleport_exits` 0x56A400 (T98a2); `setup_map_objects`
  0x4C0CC0 at its cases 44, 45, 103 and 111; `hero_relocate` 0x41D930 (its state: the tile, the
  reveal, what it does not touch; not its drawing); `hero_move_loop` 0x407B80 around the visit;
  `adventure_key_down` 0x408C40 at the space key's visit (0x408E65; T98a2 read the case whole).
  **Not followed:** the computer player (its routes through teleporters are E10's: T98a2 named
  their entries, §9.1), network play (`map_message_send` 0x482390, the `SendData` calls), the
  replay's drawing (`turn_records_replay` 0x49D7A0, §10), the sounds and pictures.
- `vcmi`: VCMI 1.7.5 (tag `1.7.5`), `lib/mapObjects/MiscObjects.cpp` (`CGTeleport`, `CGMonolith`,
  `CGSubterraneanGate`, `CGWhirlpool`), `server/CGameHandler.cpp` (`moveHero`), T98a2:
  `config/gameConfig.json` (`pathfinder`), `lib/pathfinder/PathfinderOptions.cpp`,
  `lib/mapObjects/army/CStackInstance.cpp` (`getPower`), `lib/mapping/MapFormatH3M.cpp`
  (`readObject`'s default case, `readGeneric`).
- `wiki`: <https://heroes.thelazy.net/index.php/Whirlpool>,
  <https://heroes.thelazy.net/index.php/Subterranean_Gate>; the wiki's Monolith page through its
  mirror <https://heroes.shoutwiki.com/wiki/Lith> (a search engine's copy; the page refused a
  direct fetch); T98a2 (search engine copies, retrieved 2026-11-06): the mirror's
  <https://heroes.shoutwiki.com/wiki/Subterranean_Gate>, `Adventure_Map` and `Trivia` (the space
  bar), the Monolith page above (an enemy's exit shown in the shroud).
- `original-data`, T98a2: `Help.txt` (H3bitmap.lod), the Adventure Options' Replay Opponent Turn.
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
  (`random_between(1, n)`, no draw for one candidate: corrected by T98a2); a candidate is any other tile of the
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
- **T98a2** (§9 to §14): **a human's path ends at an entrance**: the pathfinder adds a channel's
  exits only in the computer's modes (§9.1; VCMI plans through them: D-58); the engine's reach
  needs no new read, an entrance being an entered object's destination (§9.2). A jump leaves a
  record for the others' Replay Opponent Turn and changes nothing else they see; the wiki's exit
  shown in the shroud is A-41 (§10). **The space key** revisits any entered object under the hero,
  at no points, no version test in SoD (§11). The interfaces' sketch (§11), batch E11's 18 rows
  (§12), the family ADV-TEL (§13) and the checkpoint's six questions (§14).
- **Batch E11 scored** (T101, §15): every row as read (D-53 to D-58 the original's, A-37, A-38 and
  A-41 as read, the draws uniform) but **E11-19**: a guard beside an entrance fights before the
  jump, the original's order for every entered object and VCMI's, against ADV-MON-006 as built
  (ADV-Q47: the original's taken, decision 279); A-37's consequences (a hero not drawn, a ghost)
  and A-43, a placed boat's Whirlpool tile out of the list (not reproduced, decision 280).

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
- **The most on one map** (E08 T106c, `h3-map`'s `original_data`, checked with the rows above;
  the first map by file name that holds it): one-way entrances 8 (Back For Revenge - Allied), exits
  8 (Jihad), two-way monoliths 17 (Serpents Treasure), Subterranean Gates 26 (Islands and Caves:
  13 pairs), Whirlpools 19 (Thousand Islands (allies)); one-way channels 4 (Battle of the Sexes
  Allied), two-way channels 6 (Hatchet Axe and Saw). The teleporters XL map
  (`fixtures/maps/e08-teleporters-144.h3m`) holds these numbers.

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
- *(T99b2b, read for batch E11b.)* The walk dispatches on the tile's class (map_item +0x1E,
  0x4C0D96), the last object written there; class 8, a boat, has no case (0x4C216F), so a
  Whirlpool tile whose class is a map-placed boat's is never appended to the list; a boat is its
  tile's class whatever the order (`create_boat` 0x4BB250 shows it as class 8, and a tile's refresh
  puts a boat back on top, `tile_refresh_around_heroes` 0x505A10; its word is the boat's index). The cell
  (2, 1) the six copy their word from is the Whirlpool's top-left tile (0x4C1FAD: the tile at
  column + x − 2, row + y − 1, cells counted from the anchor leftward and upward); none of the
  shipped maps holds a boat on a Whirlpool (§1).

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
Else `r = random_between(1, n)` (0x50B230: `1 + rand() mod n`, the C runtime's one stream) and the
`r`-th candidate in the list's order. **No draw when `n` is 1:** `random_between` returns an equal
bound at once, without `crt_rand` (0x50B236; T98a2's correction of T98a1, which read a draw there).

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
  the one-way and two-way lists) for the computer's paths only (§9.1, T98a2); the computer's
  routes are E10's.
- **The space key** visits the object under the hero again (`adventure_key_down` 0x408C40 →
  `visit_and_update` at the hero's own tile, 0x408E65, when that tile has the entrance bit and its
  class is not 3). The wiki: the space bar takes a hero through a gate again without spending
  movement points, likewise a two-way monolith (not in RoE) and a Whirlpool's square (AB and SoD).
  Its full gate (any entered class, T98a2) and the command's question: §11.
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
| The weakest stack | fight value × count, the first on a tie | AI value × count from the first slot, a reverse scan taking a strictly weaker one: the first slot when it ties, else the last of a tie (D-54) | total fight value |
| The loss | keeps `⌊n / 2⌋`; a lone creature kept | takes `max(1, ⌊n / 2⌋)` (D-55) | 50 % rounded up lost |
| The space key | visits any entered object under the hero, no points (§11) | — | AB and SoD, any repeatable object; not in RoE |
| A path through a teleporter | none for a human (§9.1) | two-way and gates; Whirlpools if protected (D-58) | — |
| An enemy's exit in the shroud | not read (the replay's drawing) | — | "shown" (A-41) |

## 8. Disputes, quirk candidates and open points

- **D-53 (an exit chosen).** VCMI's server takes an exit named by the client's answer
  (`CGMonolith::teleportDialogAnswered`: `exits[answer]`; every exit of a two-way channel, and every
  Whirlpool tile to a protected hero, is offered), which its client fills from a
  path through the channel; the original always draws. Gameplay: whether a player picks the exit of
  the 34 two-way channels of three or more and of the Whirlpools. Recommended: **the original**
  (drawn; a command never names the exit, ADR-0004).
- **D-54 (the weakest stack).** The original: least fight value × count, the first slot on a tie;
  VCMI: AI value × count (`CStackInstance::getPower`), the last slot on a tie (a reverse scan with
  a strict test). The wiki: total fight value. Recommended: **the original**. *(T99b2a's
  correction, `CGWhirlpool::onHeroVisit` read again: the target starts at the first slot and the
  reverse scan takes a slot only when strictly weaker, so a tie that holds the first slot keeps it,
  as the original; the last of a tie only when the first slot is not in it. E11-11's setup moved
  its strong stack to slot 0.)*
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
- **D-58 (T98a2; a path through a teleporter).** The original's human path ends at an entrance
  (mode 0; only the computer's modes add a channel's exits, §9.1); VCMI's pathfinder plans through
  two-way monoliths and gates, and Whirlpools for a protected hero, by default, its client naming
  the exit (D-53). The wiki is silent. Gameplay: a destination past a channel in one command.
  Recommended: **the original** (the entrance a destination; the env's action a reach's tile).
- **A-41 (T98a2; quirk candidate). An enemy's exit shown in the shroud** (the wiki; §10): a viewer
  watching an enemy hero enter a monolith in the open has its exit shown in its shroud. The reading
  stops at the record (the drawing not read). A display's leak of a channel's exit; recommended:
  **not reproduced**, no flag (a viewer learns nothing of a hidden tile, ADV-FOG).
- **Open points:** the gate's ADVEVENT 153 has no human test (a message, nothing for the engine);
  the draw's stream (the original's one `rand`; ADR-0017's streams) is T98b's; batch E11 (§12)
  observes the points that rest on the reading alone: the draw's uniformity, A-37, the gate's
  meeting without a move, the guards silent at an exit, the points kept, the lone creature's
  message. **The wiki's 1 776 Azure Dragons** (A-38's "bug") is the bound's exact count by fight
  value: 56 315 × 1 775 = 99 959 125 is below 99 999 999, × 1 776 = 100 015 440 is not (by AI
  value, 78 845, the count would be 1 269): it supports the reading's measure (D-54) and bound.

Next ids: disputes D-59, reading quirk candidates A-44 (A-42 the design note's, A-43 §15).

## 9. The reach and the paths through a teleporter (T98a2)

### 9.1 The original's paths (`pathfinder_pass_through` 0x56AB40 at its cases 43, 45, 103, 111, read whole)

- `pathfinder_run` asks `pathfinder_pass_through` of a stop node it pops (0x56BB7C; E08 T58), and
  a teleporter's entrance is one (§2: entered); and once at its start when the hero stands on an
  entrance of class 43, 45 or 103 (not 111; 0x56B9AA), on a copy of the start node with its +0x1A
  raised by 50 (the field the exits' price raises; not read). The test's fourth argument is the
  run's mode (its sixth, passed at 0x56B9C9 and 0x56BB80).
- **Mode 0** (the human's mouse path, from `movement_calculations_mouse` 0x41955B): no before the
  switch for every class but 33 and 212 (0x56AB69 to 0x56AB80; E08 T55a4). **A human's path ends
  at an entrance**: a tile beyond a teleporter gets a path around it or none, never one through.
- **The cases** (0x56ADCB to 0x56AED8) answer no in every mode. Before that, in the modes from 3,
  they hand the channel's exits to the search as successors of the entrance: 43 the one-way exits'
  list of the tile's subclass (class 44, nothing excluded; 0x56AE39), 45 the two-way list (class
  45, the tile's word excluded; 0x56AE81), 111 the Whirlpools' list (class 111, the word excluded)
  to `pathfinder_teleport_exits` 0x56A400; 103 its pair (`gate_exit`) to `pathfinder_gate_exit`
  0x56A730, unless `pathfinder_entrance_guarded` 0x56A360 says no. Read in part: every candidate
  of §4.1's test is relaxed (all exits, no draw), its price raised with the list's length; in mode
  4 a guarded exit of a one-way channel or a Whirlpool is weighed by `ai_fight_answer` and one too
  strong drops the channel.
- **Whose modes:** the four other functions that call `pathfinder_run` (modes 3, 2 or a value of
  their own; listed in the address map's `ai_player_turn`) lie under `ai_player_turn` 0x525E80,
  called from the adventure manager's procedure, whose per-player records share
  `ai_player_values`' stride: the computer's (inferred). Its routes through teleporters are E10's.
- **VCMI 1.7.5** plans a human's path through two-way monoliths and gates, and through Whirlpools
  for a protected hero, by default (`config/gameConfig.json` `pathfinder`: `useMonolithTwoWay` and
  `useWhirlpool` true, `useMonolithOneWayUnique` and `useMonolithOneWayRandom` false;
  `lib/pathfinder/PathfinderOptions.cpp`); its client then names the path's exit (D-53). **D-58.**
  The wiki is silent.

### 9.2 The engine's reach and passage

- **Today** the five classes are untyped blockers (T97c): the reach and `MoveHero` go around them.
- **Typed as entered objects** (§2), an entrance is a visit's tile, a row both searches have: the
  walking search's and `AdventureState::passage`'s entered object (ADV-OBJ-003: a visit ends the
  move; reached from any node; on land a flight also passes over it, as over every entered
  object's tile, ADV-MOV-009). That is the original's mode 0: the reach reports the entrance as a
  destination with the points left after the step (the jump costs none, §5), never as a tile on
  the way, and no exit. `MoveHero`'s last step may be onto the entrance; its visit is the jump.
- **From above** (§2): a monolith's or a gate's entrance is not entered from, or left toward, the
  three tiles above it (ADV-OBJ-004's rows; nothing new). The Whirlpool is entered from above.
- **On water:** a Whirlpool's six tiles are a water object's visit tiles, closed to a flight and a
  water walk (D-46's row) and entered by a hero in a boat (`sail_entry`'s entered object,
  ADV-SEA-005). The jump moves the boat with the hero (it stays in it, `Mode::Sail`).
- **A hero standing on an entrance** (a failed jump, a two-way exit, a gate after a jump) starts
  its reach there as on any entered object; a step off and back on visits again (§6).
- **The hot path:** no new read in the reach's or the passage's loops: the entrance's row comes
  from the static tile's visit, which the kinds set. The channels (§3) are read by the visit alone,
  once a jump (one list: at most 114 Whirlpool tiles on a shipped map, 19 Whirlpools), and are
  built with the game from the static map, shared by forks (decision 6). T98b plans the measure:
  `reach-xl-144` and the `random-play-*` workloads against `master` (TRAPS: a fixture's or test's
  blocker of one of the five classes becomes typed), and a teleporters' workload.
- **For the engine:** the reach and `MoveHero` keep the original's human paths (D-58 recommended):
  an entrance is a destination, the env's action stays a destination among the reach's tiles
  (ADR-0017 point 8). The computer's planning through channels (all exits, their pricing, the
  guards) is E10's.

## 10. What a player sees (T98a2)

- **The mover's team:** the exit revealed with the hero's scouting radius (§5); an occupied exit
  radius 1 before the battle (§4.2).
- **The record** (`hero_relocate_inner` 0x49CF50, read whole): before the hero's tile is written,
  a record of the move (the acting player, the hero, its facing, the old tile and the new) is
  appended to the game's `turn_records` (+0x4E7AC), the list of a turn's map changes (the steps,
  the owners' changes and the reveals append to it too). A player's records live until its next
  turn starts (`turn_records_drop` 0x49D6C0, read whole, from `next_player_turn`); the others play
  them back with the Adventure Options' **Replay Opponent Turn** (`adventure_options_run` 0x41AB00
  → `turn_records_replay` 0x49D7A0; `Help.txt`: the events since your last turn). No rule's state
  changes. What a record shows a viewer whose fog hides one end: not read (the drawing).
- **Network play:** `map_message_send` before the record (not followed).
- **Hot seat:** another player sees the hero where its explored tiles show heroes (ADV-FOG-004):
  gone from the entrance, at the exit if it explored it.
- **The wiki** (the Monolith page's mirror): when an enemy hero enters a monolith in the open part
  of the map, its exit is "shown" even in the shroud. The reading stops before the view that would
  show it (the replay's or the live move's); a display, but it tells a viewer where a channel
  leads. **A-41.**
- **For the engine:** one event a jump, projected as a step's (`HeroStepped`; ADV-FOG-004, -005):
  seen by the players that see the hero at either end, an end it does not see `None` in its
  projection; A-41 not reproduced (recommended): no viewer learns a hidden exit.

## 11. Commands, decisions, events, the observation, replays and benchmarks (T98a2)

For T98b's drafts and design note (ADR-0015 to -0017); a sketch, not decided.

- **The jump needs no command:** `MoveHero`'s last step onto an entrance visits it (ADV-OBJ-003);
  the jump is that visit, and no path goes on past it (§9).
- **The space key** (`adventure_key_down`'s case 0x408CC4 to 0x408E75, read whole; `key_space_visit`
  0x408E65): a selected hero, the current player human (both flags, 0x408C77: never on a
  computer's turn), the hero chosen again when it stands (manager +0x20C clear: `hero_select` lifts
  it, so its tile shows the object under it), then the tile visited (`visit_and_update`) when it is
  an entrance (+0xD 0x10) of any class but 3 (`ObjNames.txt`'s Anchor Point). **No teleporter
  test, no version test** in this executable (SoD; RoE's own is not readable here), **no points**
  spent or required (the visit tests none). So Space repeats the visit of any entered object the
  hero stands on: a two-way monolith or a gate back and forth, a Whirlpool with its losses again, a
  one-way exit's message, a dwelling's hire question, a weekly object's "already visited", a town.
  The wiki agrees: AB and SoD, without movement points, for every object with repeatable actions,
  "unlimited cycling through two-way monoliths", not in RoE (the Subterranean Gate page, Trivia).
  A two-way channel of three or more (34, §1) is re-drawn at will at 0 points. **For the engine:**
  a command `Revisit { hero }` (a name for T98b): the player's hero, its tile an entered object's
  built visit; no points; the step's own visit with its decisions. The checkpoint's question: in
  increment 11 with the original's gate (every entered kind; recommended), teleporters only, or
  later (a stub).
- **Decisions:** none new. The exit is drawn (D-53), the Whirlpool and the gate ask nothing. An
  occupied exit's battle is GAM-BAT-004's (the mover attacks the exit's hero), the move to the exit
  the visit's rest after a won battle (GAM-DEC-007); a gate's hero of the team is GAM-MET-001's
  meeting, the mover staying.
- **Events:** the jump (§10), the reveal (`TilesRevealed`), the Whirlpool's loss (new: the hero,
  the slot, the count lost, 0 for a lone creature kept; the owner alone, the original's ADVEVENT
  168 also when nothing is lost), "leads nowhere" for a one-way exit's visit (ADVEVENT 70; a
  human's only in the original, every owner's here, decision 16) and a gate without a pair
  (ADVEVENT 153), the owner alone; a monolith without a candidate tells nothing (the original's
  silence).
- **The observation:** each kind public, a monolith's channel (its subclass: the picture's
  colour) with it; no channel's exits or gate's pair listed (only the map's pictures show them).
- **State:** the hero's tile (and a boat's), the army, the fog; the channels static (built with
  the game, ADR-0015's static part). The adventure state hash likely unmoved; the ruleset part
  (the kinds, v18) and the map import (the five classes, A-39's refusal; v20) bump; a new command
  bumps the adventure replay format (v11) and the env's action encoding (the AI lane's,
  `cross-lane`).
- **The draw:** one from the adventure stream for two or more candidates, none for one (§4.1);
  which stream: T98b (ADR-0017's streams).
- **Replays and benchmarks:** golden replays of each kind's jump (a one-way channel with several
  exits, a two-way of three, a gate, a Whirlpool with its loss, an occupied exit's battle, the
  revisit if taken); §9.2's measure and a `random-play-teleporters-36` workload on a fixture map
  with every kind (as increment 10's dwellings' map).

## 12. Batch E11's plan (T98a2)

Increment 11's batch (named after the increment as batches E3 to E10; not the epic E11), **batch
E11**, runs on the client lane's harness in SoD on Easy, hot seat, from maps `h3-experiment` writes
(`docs/experiments/README.md`), as before (decision 261's agent's call); its spec and recipe are
tasks of T98b's cut. **Two runs:** **E11a** the jumps (day 1 and 2), **E11b** the repetitions with
the space key (one day). Rows for the points resting on the reading alone (§8): the draw's spread,
A-37, the gate's meeting without a move, the guards silent at an exit, the points kept, the lone
creature's message; and the disputes the batch can tell apart.

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E11-1 | §9.1, D-58 | Red's hero beside a two-way pair A, B; B's side reached by land only past the day's points | the path to a tile beside B; to A | none through A (around or none); A's ends on A |
| E11-2 | §5, §6 | Red's hero steps onto A | tile, points before and after, explored tiles round B, the walk | on B; the step's price paid, no more; B's scouting radius explored; the walk ended |
| E11-3 | §4.5, §5 | a monster on a tile next to B | battles on arrival | none (a guard fights only after a step) |
| E11-4 | §4.2 | a one-way entrance whose only exit holds Red's second hero | tile, messages | stays on the entrance, nothing shown |
| E11-5 | §4.2 | Red's hero steps onto a one-way exit | the message | ADVEVENT 70 |
| E11-6 | §4.3 | a gate pair, Red's second hero on the exit | the dialog, both tiles | the meeting; the mover stays on the entrance |
| E11-7 | §4.3 | a gate pair, Blue's hero on the exit | the battle; tiles after Red wins | the battle, then Red on the exit |
| E11-8 | §4.3 | a gate with no pair (one level's gates outnumber the other's) | the message, the tile | ADVEVENT 153; stays |
| E11-9 | §3.2, D-56 | a surface gate G, two underground gates at equal truncated distances from G and different squared ones, the farther (squared) first in the walk | which G leads to | the first in the walk (truncated); VCMI the nearer by squares |
| E11-10 | §4.4, D-54, D-55 | Red's hero in a boat with 3 Pegasi (slot 0), 3 Dendroid Guards (slot 1) and a strong stack enters a Whirlpool | army after; the message | Pegasi 3 → 1 (fight value × count 1 221 < 2 070); VCMI halves the Dendroids by AI value (1 551 < 1 554) and keeps 2; ADVEVENT 168 |
| E11-11 | §4.4, D-54's tie | a strong stack in slot 0, 5 Pikemen in slots 1 and 2 (T99b2a; with the tie in slots 0 and 2 VCMI takes slot 0 too) | army after | slot 1 keeps 2 (the first); VCMI slot 2 keeps 3 |
| E11-12 | §4.4, A-38 | a lone Pikeman; a lone stack of 1 775, then 1 776 Azure Dragons | army; the message | 1 kept with the message; 1 775 → 887; 1 776 all kept (56 315 × 1 776 past 99 999 999) |
| E11-13 | §4.4 | the Sea Captain's Hat worn; the Admiral's Hat worn | army; messages | nothing lost, no message |
| E11-14 | §11's space key | Red's hero on B with 0 points (spent on the way to A) presses Space; then on a dwelling it owns | tile, points; the question | back on A at 0 points; the hire question again |
| E11-15 | §4.1, D-53 | a two-way channel of four; Red's hero presses Space 60 times | the tile after each | each of the three others about 20 times, never the entrance |
| E11-16 | A-37 | Whirlpools W1, W2; Blue's hero in a boat on a W2 tile; Red with the Sea Captain's Hat in a boat in W1 presses Space until it lands on Blue's tile | both heroes' tiles, the tile's object, the next turn | both on one tile, no battle (A-37); what follows recorded |
| E11-17 | D-57 | three Whirlpools; Red's second hero in a boat on the W2 tile VCMI takes for its visitable one; Red presses Space 60 times from W1 | the Whirlpool and tile of each exit | W2's five other tiles and W3's six alike (5/11 on W2); VCMI never W2 |
| E11-18 | §10, A-41 | Blue's hero enters a monolith Red sees, its exit in Red's shroud; on Red's turn the Replay Opponent Turn | the view's corner during the replay; Red's explored tiles after | the view goes to the exit (the wiki), nothing explored |

**Not in the batch:** A-39 (refused at the import), A-40 and the computer's paths (E10), RoE's
space key (E11 the epic), network play. **What the harness lacks** (`clients/capture/`, a common
path: built with the harness check, the client and map-client lanes' inboxes told): the space key
repeated with a read after each press (the step-by-step table's rows, E08 T68b), the adventure
options' Replay item and the view's corner during it, the meeting dialog closed unchanged. **What
the spec lacks** (`h3-experiment`, battle-owned: `cross-lane`; the writer `h3_map::writer` is this
lane's): the five classes by picture and subclass on either level (`[[map.object]]`), a hero in a
boat at the start beside or on a Whirlpool, worn Hats; E11-9's and E11-16's geometry is the spec's
task to check (the gates' walk order, a hero standing on a Whirlpool tile). *(E08 T99a built them:
`docs/experiments/README.md`, "Increment 11's objects"; a boat on a Whirlpool's tile boarded from
the shore. VCMI 1.7.5's `CGSubterraneanGate::postInit` sorts every gate by its entrance, level,
row and column, the original's walk, and pairs only toward a deeper level, the same for two
levels; its visitable tile of a Whirlpool is the anchor, `ObjectTemplate::calculateVisitableOffset`
taking the visitable tile nearest it.)*

## 13. The proposed rule ids (T98a2)

A new family **ADV-TEL** (teleporters); the T98b drafts fix titles and cuts.

| Id | Proposed rule | Rests on |
|---|---|---|
| ADV-TEL-001 | The kinds: classes 43, 44, 45 by subclass (the channel), 103, 111; entered, only the Whirlpool from above; the import refuses a monolith subclass past 7 (A-39) | §1, §2, §3.1 |
| ADV-TEL-002 | The channels when the game is built: a list per one-way and two-way subclass, one of every Whirlpool tile; the gates paired greedily by truncated distance, the walk's order, the first on a tie (D-56) | §3 |
| ADV-TEL-003 | The exit's draw: the candidates (another tile of the channel without a hero of the mover's team), uniform, one draw for two or more, none for one; none: no jump (D-53, D-57) | §4.1 |
| ADV-TEL-004 | The monoliths: the draw; an enemy on the exit fought first (radius 1), the mover moves if it wins; a one-way exit leads nowhere | §4.2, §4.5 |
| ADV-TEL-005 | The gate: its pair; none: nowhere; a hero of the team met, the mover stays; an enemy fought, the mover moves if it wins | §4.3 |
| ADV-TEL-006 | The Whirlpool: the loss first (the weakest stack by fight value × count, the first slot, keeps half rounded down, a lone creature kept; no loss with the Sea Captain's Hat; D-54, D-55, A-38, A-40), then the draw; an enemy on the drawn tile per A-37's answer | §4.4 |
| ADV-TEL-007 | The arrival: points kept, no guard, no visit, the scouting radius revealed; the move ends | §5, §6 |
| ADV-TEL-008 | The revisit (if taken): the visit of the entered object under the hero again, without points | §11 |
| ADV-FOG-004 (amended) | A jump seen as a step's two ends (A-41 not reproduced) | §10 |
| ADV-OBJ-002 (amended) | The import's classes 43, 44, 45, 103, 111 and the refusal | §1, A-39 |
| ADV-OBJ-003 (amended) | The five entered; no path through an entrance (D-58) | §2, §9 |
| GAM-BAT-004, GAM-MET-001 (amended) | A battle or a meeting at a teleporter's exit, the mover on the entrance | §4.2, §4.3 |

## 14. For T98b: the checkpoint's questions (T98a2)

Asked at T98a2's end (`docs/OPEN-QUESTIONS.md`, ADV-Q45; the answers in the epic as decisions
from 264):

1. **D-53 to D-58**, the disputes: the original in each (recommended): the exit drawn, never named
   by a command (D-53); the weakest stack by fight value × count, the first slot (D-54); half kept
   rounded down, a lone creature kept (D-55); the gates' truncated distance (D-56); one draw over
   the Whirlpools' tiles (D-57); no path through a teleporter (D-58).
2. **A-37's fix** (state-corrupting): the battle as at a monolith (recommended), the tile not a
   candidate, or VCMI's (the mover stays, no battle).
3. **A-38**: no bound (the least stack, a 64-bit product; recommended), or the lone stack past the
   bound keeps its troops (the wiki's "bug", a `fix-optional` flag); the corrupting case is fixed
   either way.
4. **A-39** refused at the import, **A-40** not reproduced (every hero loses as a human's), **A-41**
   not reproduced (recommended for each).
5. **The space key:** `Revisit` in increment 11 with the original's gate (recommended), for
   teleporters only, or later.
6. **Batch E11** before the build (recommended, as batches E9 and E10), after it, or not at all.

**Answered** (2026-11-06, `AskUserQuestion`, two rounds; the epic's decisions 264–269; ADV-Q45):
each as recommended. (1) D-53 to D-58 **the original**; (2) A-37 **the battle**, as at a
monolith; (3) A-38 **no bound**; (4) A-39 **refused at the import**, A-40 and A-41 **not
reproduced**, no flags; (5) **`Revisit` in increment 11** with the original's gate (every entered
kind, no points); (6) **batch E11 before the build**.

## 15. Batch E11 scored (T101)

Scored on 2026-11-07 (E08 T101; docs only) from the recorded runs: E11a
`experiments/results/E08-batch-11a.md` (T100a: three runs, the second and third alike in all 97
records, the first differing only in E11-8's branches, since ordered) and E11b `E08-batch-11b.md`
(T100b: three runs, every record alike but the draws; runs 1 and 2's draws in `E08-log.md`,
T100b), played by the capture harness (`clients/capture/batches/E08-batch-11a.toml`, `-11b.toml`)
in SoD 4.0 on Easy, hot seat, and read from the game's memory. Each row against §12's prediction
with design §16's additions (the specs' tiles; the texts by table and line).

| Row | Predicted | Observed | Verdict |
|---|---|---|---|
| E11-1 (§9.1, D-58) | no path through A (around or none); A's ends on A | no path to B (beyond the wall, past A); the path to A the one tile A | **D-58 confirmed**: VCMI's path through a two-way pair is not the original's |
| E11-2 (§5, §6) | on B; the step's price paid, no more; B's scouting radius explored; the walk ended | on B (12, 10) with 1 900 of 2 000 (the step's 100, the maximum kept), not on again; around B (16, 13) and (17, 10) explored, (16, 14) and (18, 10) not; no message | confirmed |
| E11-3 (§4.5, §5) | no battle on arrival | none beside P (13, 11); the control step to (13, 10) fought P | confirmed (VCMI's `IGNORE_GUARDS`; the wiki: "A monolith or similar object does not trigger combat") |
| E11-4 (§4.2) | stays on the entrance, nothing shown | on E4 (32, 10) with 1 900; no message, screen or battle; E11-5 still on X4 | confirmed (VCMI's `isExitPassable`) |
| E11-5 (§4.2) | ADVEVENT 70 | ADVEVENT 70; on X4 with 1 900 | confirmed (VCMI the same) |
| E11-6 (§4.3; design §16) | the meeting; the mover stays; the tiles within 1 of the pair explored before it | E11-6b to U6 with 1 859 (a diagonal step); E11-6's step opened the exchange (E11-6 left, the visitor; E11-6b right), closed unchanged; E11-6 on G6's entrance (surface) with 1 859, E11-6b on U6; the ten underground tiles explored before and after (by E11-6b's arrival: the radius-1 reveal not tellable here; E11-7 tells it) | confirmed |
| E11-7 (§4.3; design §16) | the battle, then Red on the exit; radius 1 explored before it; the exit's terrain | Red's underground bits unexplored on day 1 and on day 2 before the step; at the battle's opening the 3 × 3 about U7 (44, 30) explored, (46, 30) not, Red's hero on G7's entrance (surface); the battle subterranean, Red attacking Blue's Pikeman; won (510 experience, no level-up); then on U7 with 1 859, (46, 30) and (49, 30) explored, (50, 30) not | confirmed |
| E11-8 (§4.3) | ADVEVENT 153; stays | the walk to Un's entrance (623: by (17, 28), (17, 29), (18, 30), (17, 31)), ADVEVENT 153, on Un with 1 277 | confirmed (VCMI and the wiki the same) |
| E11-9 (§3.2, D-56) | the first in the walk (truncated); VCMI the nearer by squares | G9 (20, 30) led to Uf (17, 27): truncated distances 4 and 4 (squares 18 and 16), Uf first in the walk; on Uf with 1 900 | **D-56 confirmed**: VCMI's squared distance (Un) is not the original's; the wiki's tie "north, then west" is the walk's order |
| E11-10 (§4.4, D-54, D-55) | Pegasi 3 → 1; VCMI the Dendroids 3 → 2; ADVEVENT 168 | Pegasi 3 → 1, the Dendroids' 3 and the Angels' 2 kept; ADVEVENT 168; on W3's (66, 69) in its boat, 1 400 of 1 500 | **D-54 and D-55 confirmed**: VCMI's AI value and `max(1, ⌊n / 2⌋)` are not the original's; the wiki's total fight value and "50 % rounded up" lost agree |
| E11-11 (§4.4, D-54's tie) | slot 1 keeps 2 (the first); VCMI slot 2 keeps 3 | slot 1's Pikemen 5 → 2, slot 2's 5 kept; ADVEVENT 168; on W3's (66, 68) | confirmed: the first slot of a tie |
| E11-12 (§4.4, A-38) | 1 kept with the message; 1 775 → 887; 1 776 all kept | one Pikeman kept, ADVEVENT 168; 1 775 → 887; 1 776 kept, ADVEVENT 168; each on W2 or W3 with 1 400 | confirmed: **A-38 as read**, at the wiki's 1 776 exactly (the engine has no bound, decision 266: 1 776 → 888) |
| E11-13 (§4.4) | nothing lost, no message | the Sea Captain's Hat: ten Pikemen kept, no message, on W3's (67, 69); the Admiral's Hat (E11-18's hero, in a boat with 2 615 of 3 000, a path at once to W1): ten kept, no message, on W2's (60, 68) | confirmed (VCMI's `WHIRLPOOL_PROTECTION`, the wiki: either Hat) |
| E11-14 (§11) | back on A at 0 points; the hire question again | onto S: ADVEVENT 35 (no), 1 400; Space on S: ADVEVENT 35 again, unmoved, 1 400; the walk to A with its last 400 points: on B with 0; Space: on A with 0; again: on B with 0; no message | confirmed (the wiki: AB and SoD, no movement points) |
| E11-15 (§4.1, D-53) | each of the three others about 20 times, never the entrance | 61 jumps a run (the step's and 60 presses), 1 900 kept by every press; never the monolith stood on (0 of 183); the spread below | confirmed: uniform among the three others |
| E11-16 (A-37) | both on one tile, no battle; what follows recorded | the landing on E11-17's (Red's) tile after 33, 26 and 1 presses (runs 1 to 3): no battle, message or dialog; both heroes on (17, 23), E11-16 with 2 707 and its Pikeman; the next turns below | **A-37 confirmed**, state-corrupting (below); fixed by the battle first (decision 265) |
| E11-17 (§4.1, D-57; design §16) | W2's five other tiles and W3's six alike (5/11 on W2); VCMI never W2; W3's empty boat never drawn | the list at the start 16 tiles in the walk's order, W1's six, W3's five, W2's five (both boats' tiles out, §3.1), the words W1 0, W3 1, W2 2; E11-17b boarded the boat on (18, 24) with no visit; 61 jumps a run, 2 750 and the five Archangels kept, no message; W2 drawn (58 of 183), never (18, 24) or (29, 20), never the Whirlpool stood on, every listed tile 7 to 19 times | **D-57 confirmed**: VCMI's "never W2" is not the original's. With both boats' tiles out of the list the row tells neither the team's hero's exclusion on a Whirlpool's tile nor the empty boat's at the draw (both §4.1's test, as E11-4) |
| E11-18 (§10, A-41) | the view goes to the exit (the wiki); nothing explored | Replay Opponent Turn (about a second of the game's time): the view's corner 35,22,0 → 35,22,1 (centred on U7, E11-7b's gate) → 38,25,0 → 39,25,0 (M18) → 57,52,0 (centred on X18, in Red's shroud) → back to 36,23,0; at each jump the old picture fades to black, the exit's surroundings never drawn; no view on the boardings' tiles; none of Red's tiles explored | confirmed: **A-41 as read**, the view's corner names a hidden exit, nothing drawn or explored (not reproduced, decision 267) |
| E11-19 (design §7, §13) | *the agent's call* (decision 272): no battle (not M: the hero jumped; not N: the arrival); at Space no battle, on A | **M fought first**: the battle opened with the hero on A19 (10, 50), won; then the jump, on B19 with 1 900, N (beside B19) untouched; Space on B19: no battle, on A19 with 1 900; the control: no battle (M gone) | **against the agent's call**: the step's guard before the entrance's visit, the jump after the win (VCMI the same: `CGameHandler::moveHero`'s `objectVisited(guardian)`, then the destination after a victory, `visitDestAfterVictory`); the arrival's silence (E11-3) and the revisit's (decision 273) confirmed. **ADV-Q47** |

**The spread** (all three runs of E11b; the draws differ between runs, the game running in real
time between the presses; χ² of each draw's candidates against equal odds):
- **E11-15:** the monoliths drawn per run (M1/M2/M3/M4) 13/18/14/16, 20/12/14/15, 14/18/14/15, over
  the three 47/48/42/46 (45.75 each expected: §12's "about 20 times" was a fixed entrance's); the
  draws by the monolith stood on, to the other three in order: M1 15/13/21, M2 15/18/15, M3 20/10/10,
  M4 12/23/11; χ² 13.3, 8 degrees of freedom, p ≈ 0.10.
- **E11-17:** the Whirlpools drawn per run (W1/W2/W3) 21/17/23, 22/21/18, 22/20/19, over the three
  65/58/60 (the chain's shares 12 : 11 : 11 give 64.6/59.2/59.2); from W1 W2 35, W3 33 (5 : 5),
  from W2 W1 28, W3 27 (6 : 5), from W3 W1 37, W2 23 (6 : 5); χ² by Whirlpool 1.6 (3 df, p ≈ 0.66),
  by tile over each draw's ten or eleven candidates 35.0 (29 df, p ≈ 0.21).
- **E11-16:** 34, 27 and 2 jumps from W1's (24, 15) to the landing, never the Whirlpool stood on;
  from W1 W2 6 and W3 17 (alone p ≈ 0.03; with E11-17's, W2 41 of 91); each landing from W3 (an
  enemy hero's tile is a candidate from its own Whirlpool too, §4.1: not seen).

So the draw is uniform over the candidates as read (`random_between(1, n)`, the r-th in the list's
order); its sequence is the game's (the engine draws from its own stream, ADR-0017).

**Beyond the predictions:**
- **A-37's consequences** (E11-16, three runs alike; the tile's records and the dumps): the landing
  puts the mover's record on the other hero's: (17, 23) names E11-16 (class 34) over E11-17 (34)
  over the Whirlpool (111, word 2), and the map draws the hero on top. Selecting a hero takes its
  record off the tile and writes back what lay under it (below): on Red's day 2 E11-17's selection
  writes (111, 2) over E11-16's record, so after E11-17's step off the tile shows the Whirlpool alone
  (**E11-16 not drawn for Red**, though it stands there); on Blue's day 2 E11-16's selection writes
  back E11-17's stale record, and after E11-16's step off **a ghost of E11-17** stays on (17, 23) (a
  boat with Red's flag besides E11-17's own on (17, 22)) and the Whirlpool's record is gone from the
  tile (class 0 under the ghost). No battle, meeting or message at any step. Recorded with A-37
  (`ADV-QUIRK-39`, written by T105b), *the agent's call*: one cause (two heroes on one tile) and one
  fix (decision 265: no two heroes share a tile), not a new candidate.
- **A-43 (quirk candidate). A Whirlpool's tile under a map-placed boat** (E11-17b's boarding, §3.1):
  the setup's walk skips the boat's tile, so it is never in the list, also once the boat has left
  (never an exit); after the boarding its record reads class 111 with the word −1 (the Whirlpool
  under the boat; read with E11-17b lifted, below). By §4.1's test (not seen) a visit there
  excludes the word −1, so its draw takes every listed tile, its own Whirlpool's five included.
  Harmless (nothing corrupted) and on crafted maps only: none of the 160 shipped maps holds a boat on
  a Whirlpool (§1). As designed (note §5, §7) the engine lists every Whirlpool tile and skips one
  while an empty boat stands on it, so it differs only after such a boat has left, or at a visit from
  its tile. **Not reproduced**, no flag (ADV-Q47, decision 280; `ADV-QUIRK-45`, written by T104).
- **The selected hero lifted** (the harness's read; `clients/capture/README.md`, the lane's
  `TRAPS.md`): at its selection and after its move the original takes the selected hero's record
  off its tile (the tile reads what lies under it) and a scroll of the view puts it back. A display's
  bookkeeping of the tile records: the engine keeps a hero's tile in the hero alone (ADR-0016 point
  1) and reproduces nothing of it; it explains the reads and A-37's vanishing hero.
- **The messages of a townless player** (GENRLTXT 7 after E11-3's control battle; at each turn's
  start GENRLTXT 14 and the countdown ARRAYTXT 128): the batch maps give no player a town, the
  harness's known messages (E10's defeat), not this increment's. **A boarding** without the Admiral's
  Hat leaves 0 points with the land's maximum, with it the points and the sea's maximum (increment
  8's, ADV-SEA).

**The rules** (§13, design §15): the new and amended rules cite these rows as `observation` sources
and are written `verified` when T103 to T105 build them; until then nothing in the catalog
changes. ADV-TEL-001 and ADV-OBJ-002 every row (the five classes as the original reads them);
ADV-TEL-002 E11-9 (D-56), E11-8, E11-17's list; ADV-TEL-003 E11-15, E11-16, E11-17 (the spread,
never the entrance), E11-4; ADV-TEL-004 E11-2, E11-4, E11-5, E11-15, E11-19; ADV-TEL-005 E11-6 to
E11-9; ADV-TEL-006 E11-10 to E11-13, E11-16, E11-17 (HER-ADV-009 E11-13); ADV-TEL-007 E11-2, E11-3,
E11-7, E11-10, E11-19; ADV-TEL-008 E11-14, E11-15, E11-17, E11-19's Space; ADV-OBJ-003 E11-1
(D-58); ADV-FOG-004 E11-18 (A-41); ADV-MON-006 E11-19, E11-3; GAM-BAT-004 E11-7, E11-16 (A-37);
GAM-MET-001 E11-6; GAM-DEC-007 E11-7 (the move after the won battle), E11-19.

**Against the design: E11-19 alone** (ADV-Q47, before T102). Every other row is as read: D-53 to
D-58 confirmed as the original (decision 264), A-37, A-38 and A-41 as read (their decisions 265 to
267 stand). E11-19 answers design §7's open point: the original's order is the guard first, then
the visit. It agrees with §6's reading (the step's guard test inside `hero_move_step`, 0x48169D;
the visit after the walk, `hero_move_loop` 0x4080E9) and with the event's order on a guarded tile
(`E08-rewards.md` §8.5, already built), and it is VCMI's for every visited object; but ADV-MON-006
as built (E08 T20b1, T63b) visits the tile's object first and its guard after ("testing the guard
once it returns", never read): **every entered object on a guarded tile**, not only a teleporter.
**Answered** (2026-11-07, ADV-Q47; decisions 279, 280): **the original's order for every entered
object** (the guard first, the tile's visit after it for a hero still there; ADV-MON-006 and
GAM-DEC-007 amended by their building task), and A-43 not reproduced. The user, from play: a
two-way monolith's exit starts no fight and a hero on it cycles between the pair without one; a
fight starts only on a step off it onto a guarded tile (E11-3, E11-19's Space agree); a one-way
exit probably alike (the hero then forced off it onto the guarded tile), **to be investigated**:
§5's reading (`visit_monolith_one_way` read whole, `hero_relocate` as the two-way's) predicts no
fight on arrival, unobserved; batch E11c (design §16) runs the row before T105a builds the arrival:
**as read**, §15.1.

### 15.1 Batch E11c scored (T103c)

Scored on 2026-11-07 (E08 T103c) from `experiments/results/E08-batch-11c.md`: decision 279's
investigation (design §16, the agent's call), the spec `experiments/specs/E08-batch-11c.toml` and
the recipe `clients/capture/batches/E08-batch-11c.toml`, played by the capture harness as batch
E11 (SoD 4.0, Easy, hot seat, Red's turn of day 1; no new read). Three runs, alike in every record
but Red's starting gold and E11-22's level-up offers (the game's draws). Which came first at a
guarded entered object is the game's memory as the step's first battle opens (`at_opening`).

| Row | Predicted | Observed | Verdict |
|---|---|---|---|
| E11-20 (§4.2, §5, §11) | the arrival on a one-way exit beside a guard: no battle; Space on it: ADVEVENT 70, no battle; the step off into the zone fought | the step onto E20 (4, 4): no battle, no message, on X20 (14, 4) with 1 900, P20 (beside X20) in place; Space: ADVEVENT 70 and nothing else, unmoved, 1 900; the step to (14, 5): the battle with P20 at once (the hero on (14, 5) as it opened), won, on (14, 5) with 1 800 | confirmed: **the one-way exit as the two-way's** (E11-3), the user's "probably alike" |
| E11-21 (decision 279) | a guarded mine: the guard first (the mine neutral at the opening), then the flag | the diagonal step onto the crystal cavern's entrance (5, 12): the battle with Q21 opened with the hero on the entrance and **the mine's owner none**; won; then MINEEVNT:4 (1/day), the mine Red's, Red's treasury unchanged; on (5, 12) with 1 859 | confirmed: **the guard before the visit** (against the engine as built) |
| E11-22 (decision 279) | a guarded Learning Stone: the guard first (experience 0 at the opening), then 1 000 | the step onto the stone (6, 20): the battle with R22 opened with the hero on the stone at **experience 0, level 1**; won (the Peasant's 1); then ADVEVENT:143 (1 000 experience), 1 001, the level-up to 2 after it (its offers the game's draws); on (6, 20) with 1 900 | confirmed: **the guard before the visit and its levels** (against the engine as built) |
| E11-23 (§11, decision 273) | Space cycling on a pair whose exit is guarded: no battle at any press; the control fought | the step onto A23: no battle, on B23 (14, 28) with 1 900; presses 1 to 4 on A23, B23, A23, B23 (two arrivals by the key on the guarded tile), no battle or message, 1 900 throughout; the control to (14, 29): the battle with N23, won, 1 800 | confirmed: the user's cycle, the wiki's "unlimited cycling" |

**Every row as read; no question.** Decision 279's order holds beyond the teleporters: a step onto a
guarded tile meets the guard first and the tile's entered object is visited after a win (a mine's
flag and message, a Learning Stone's experience with its level-up, as E11-19's jump), so T103d builds
it for every entered kind (built by E08 T103d). A teleporter's arrival tests no guard whether by a one-way or a two-way
exit or by the key (E11-3, E11-19, E11-20, E11-23), and the key's visit tests none either (E11-19,
E11-20, E11-23): a hero leaves a guarded exit freely except by a step into the guard's zone, which
is fought. **Beyond the predictions:** GENRLTXT 7 (the townless player's warning, as batch E11a's
E11-3 control) after the turn's first battle, E11-20's step off; the key's visit of a one-way exit
moves nothing (the harness's "nothing").

**The rules:** ADV-MON-006 E11-21, E11-22 (the order, T103d), E11-20's step off, E11-23's control;
GAM-DEC-007 E11-21, E11-22 (the visit as the battle's held rest, T103d); ADV-TEL-004 and -007
E11-20, E11-23 (the arrival); ADV-TEL-008 E11-20's and E11-23's presses; HER-EXP-002 E11-22 (the
level after the stone).
