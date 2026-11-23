# E31 — the original's computer player: survey

The reading behind E31 (`docs/epics/E31-original-computer-player.md`): what the original's
computer does in its adventure turns, as read from `Heroes3.exe` (GOG Complete, SoD's code,
SHA-256 `aac39e8c…`; `docs/exe/README.md`). Built up by increment 1's tasks: §1 the turn (T1a1a
the loop and the records, T1a1b the planning, T1a1c the walk), §2 the values (T1a2), §3 the towns
and the answers (T1a3), §4 the rest of the code (T1a4); T1b adds the wiki's and VCMI's
positions, the hidden reads in one list, the asymmetries and the engine's gaps. The detail of each
routine is in its map entry (`scripts/exe.py lookup <address>`); this file says what the turn
does with them.

Sources: `code` only so far (insight, Q29: addresses, names and our own words; nothing printed by
the tool). Path read: `adventure_manager_procedure` 0x408BD8 calls `ai_player_turn` 0x525E80 for a
player who is not human. Branches met and not followed: the network game and Blackout Computer
(they only decide whether the computer's moves are shown), the debugging switch
`ai_only_player` 0x6994F0 (never set).

## 0. Where the code lies

The first look (the epic file) found two ranges, 0x42A000–0x434620 and 0x524370–0x52BD50. T1a1a's
call trees show more: **0x423EE0–0x42A000** is the computer's too, about 24 KB: a simulated combat
of about 13 KB (0x423EE0 to 0x427330; H3API's guesses call it `H3SimulatedCombat`), used by
`ai_quick_battle` 0x4270C0 and by the computer's fight estimates through 0x427330, and about 8 KB of
the turn's start and middle (`ai_enemy_reach_fold` 0x4280E0 to `ai_build_candidates` 0x429D50,
`ai_ally_resource_share` 0x429110 among them). The second range runs past 0x52BD50 too
(`puzzle_map_compute` 0x52C9B0 is the puzzle's, not the computer's). The total is about 95 KB
rather than 72 KB; T1a4 maps the third range.

## 1. The turn

### 1.1 The phases (`ai_player_turn` 0x525E80)

1. The adventure map's tick and the other player's status window (kind 5). Nothing more once the
   scenario has ended.
2. A **tile buffer** for the turn: one dword per tile of every level, handed to each hero's turn
   (cleared there) and set as the pathfinder's +0x6C while a hero searches; freed at the end.
3. **The start** (§1.5): `ai_turn_start` 0x4297C0 on the player's record, then
   `ai_turn_start_values` 0x527960.
4. **The turn's flag** F: the difficulty is above Easy and the player owns a town. Its address goes
   down to every hero's search (§1.4), which may clear it.
5. **The first pass:** `ai_next_hero` 0x526A90 picks a hero (§1.2) and `ai_hero_turn` 0x5261F0
   plays it (§1.3), until no hero is left or the scenario ends.
6. **The middle** (§1.5): `ai_turn_middle` 0x428DD0: the towns build, the resources are weighed
   again, allies may receive resources.
7. **The second pass:** the same choice as `ai_next_hero`, written out again inside the function,
   repeated until no hero qualifies: a hero with points left after the middle plays again here
   (whether the middle hires heroes is T1a3's).
8. **The end:** the selected hero is stopped, the buffer freed, the mouse shown.

`ai_turn_counter` 0x691684 is stepped in steps 3 and 5 by the number of heroes (capped at 9) but
nothing reads it: a vestige. No draw lies in any function of this section.

### 1.2 The order of heroes (`ai_next_hero` 0x526A90)

A hero qualifies with movement points left (+0x4D) and not asleep (+0x11C). Among them:

1. a hero without a patrol (+0x44 = 0xFF) comes before one with a patrol;
2. then the **least** sum of the primary skills, Attack and Defense counted 0 to 99 and Spell
   Power and Knowledge 1 to 99 (the hero's own values, worn artifacts included);
3. a tie goes to the earlier hero in the player's list.

The weakest free hero moves first. The call also reports whether exactly one hero qualified (the
"alone" flag, which widens that hero's search, §1.4). When none qualifies, the player's selected
hero is set to none and, if fewer than eight of its heroes are on the map, the first town in its
list with a garrison hero, no visiting hero, and that hero carrying creatures, with points and
awake, sends the hero out to the entrance (`town_garrison_to_visitor` 0x5BE390); that hero is the
next.

A computer hero's **sleep is a turn's done mark**: the start clears +0x11C for every hero and
garrison hero (and +0x43), so nothing a human would call sleeping lasts past the turn.

### 1.3 A hero's turn (`ai_hero_turn` 0x5261F0)

1. A hero standing on a town's entrance visits the town first (`ai_town_visit` 0x5253D0; T1a3):
   at every turn it starts there.
2. A patrolling hero of radius 0 (+0x44 set, +0x46 = 0) loses its points and ends: it never moves.
3. With points left, the screen follows: the hero is shown when the viewer sees its tile and
   neither Blackout Computer nor a network game hides the computer's moves. A hero that is not
   the player's selected one is selected, the turn's tile buffer cleared and step 4 taken; a hero
   already selected (it played last and is picked again) skips both, and its alone flag is
   dropped.
4. **The danger map** (`ai_danger_map` 0x42DE50, §1.9) into the turn's tile buffer, except on
   Easy for a player whose team has no human (that player's heroes then plan with an empty map).
5. **The boat marks**, with at least 10 wood and 1000 gold: each of the player's towns marks its
   boat tile (map item +0xD bit 0x08) when its Shipyard is built, or can be bought now with the
   Shipyard's cost plus 10 wood and 1000 gold left over; each map Shipyard it owns marks its boat
   tile without a test. The marks are cleared once the walk is over. A step onto a marked tile buys
   the boat (`ai_boat_create` 0x430F80, §1.10); where the search reads the mark is not found yet
   (T1a4).
6. **The search and the walk** (0x5267B0, §1.4).
7. **The garrison:** a hero still on the map, without points or asleep, standing on its own town's
   entrance, goes into an empty garrison; with a garrison hero there, the two swap when the
   garrison hero has points and is awake, or when the garrison's army (not empty) is worth less
   than the visitor's (`army_value` 0x44AC80). The stronger army ends the turn inside.

### 1.4 The search and the walk (`ai_hero_search_and_walk` 0x5267B0)

- **The budget** in movement points for the search: 1000; with a destination kept from an earlier
  turn (+0x35), the hero's +0x41 plus its points plus 200 if that is more; 32000 for a hero alone;
  for a patrolling hero at most 200 × (its distance to the patrol's centre, in columns plus rows,
  plus the radius).
- **The search** (`ai_hero_destination` 0x42E0B0, §1.9) runs up to five times, on one
  destination (`ai_candidate`, `structs.toml`) that starts as none and carries over between the
  runs. While it has no destination, or its best rate is negative and the destination is no
  must-go (+0xD), and a step was refused at the bound (the pathfinder's +0x20, set by
  `pathfinder_relax` 0x4B1D15), the budget doubles, up to 32000 or the patrol's bound: a hero
  standing in danger searches farther for a way out.
- **No destination:** the hero is put asleep and its destination cleared.
- **The week's last day:** a hero on a town's entrance whose destination is worth less than 75
  stays (asleep) on day 7, to meet the new week's growth there.
- Otherwise **the walk** (`ai_hero_walk` 0x42FEE0, §1.10).
- **A reveal:** if the destination was unexplored before the walk and is explored after it, and
  the turn's flag F is still set, the record's Eye value is recomputed (§1.6), F is cleared and
  every hero of the player is woken: they all plan again once in this turn.

### 1.5 The start and the middle

**The start** (`ai_turn_start` 0x4297C0, `ai_turn_start_values` 0x527960):

- every hero's and garrison hero's +0x43 and +0x11C cleared; the day's income computed;
- the record's **Eye value** (+0x04): `ai_eyes_value` 0x429910 sums `ai_vision_area_value`
  0x432220 (radius 10; T1a2) over every Eye of the Magi's entrance on the map, explored or not;
  with F set and a sum above 0 it is 1 000 000 instead;
- **the threats:** `ai_enemy_reach_fold` 0x4280E0 runs the pathfinder from every hero of every
  player not on its team and not defeated, with the hero's maximum points plus 800, and hands
  each search to a functor: one counts in each town's +0x3 the enemy heroes that reach it
  (`ai_town_threat_vtable` 0x63B670), the other values a reached town's defence
  (`ai_town_defence_vtable` 0x63B67C; T1a2);
- **the recruit reserve** (+0x08, `ai_recruit_reserve` 0x429AD0): per resource the most that one
  town's two most valuable available stacks (count × AI value) cost;
- **the resource weights** (`ai_resource_weights` 0x428740, below);
- the player's +0x39 from the puzzle map (`player_puzzle_target` 0x4BAE50; T1a2);
- each hero's effectiveness values (`ai_hero_effectiveness` 0x527760: H3API's AI fields of the
  hero; T1a2); the mean value of artifacts 7 to 143 that the setup allows, at the player's +0x164;
- **the difficulty weights**, two f32 globals for the values (T1a2): `ai_difficulty_weight_a`
  0x6604F8 and `ai_difficulty_weight_b` 0x6604FC are 1.0 and −0.4 on Easy, else 0.75 − d/4 and
  (d + 1)/4 for the difficulty d (Normal 0.5 and 0.5 up to Impossible −0.25 and 1.25).

**The middle** (`ai_turn_middle` 0x428DD0): the income again, and the recruit reserve less the
day's income (at least 0); both threat folds again (the heroes have moved); the candidate list
(`ai_build_candidates` 0x429D50) and `ai_town_build_choice` 0x42AE00 again and again while it
builds (T1a3); an artifact step (`ai_turn_artifacts` 0x431360; T1a3); the weights again; and, when
one of its towns has a Marketplace, `ai_ally_resource_share` 0x429110 with each living ally,
computers first and humans after (a human ally gets a message box; the amounts T1a3). Last, a
debugging check shows a warning box for every resource below 0.

**The resource weights** (`ai_resource_weights` 0x428740): per resource the **means** (the stock
plus twice the day's income, +0x24) and the **demand** (+0x40: the largest cost of any building
buildable now in any of its towns, plus the costs of three most valuable stacks of a list built
from its towns' dwellings, that part not read). A rate r comes from the market's table by the
number of its towns that may build a Marketplace (1 to 10). The factor: no demand, r; a demand at
most the means, (surplus × r + demand) / means; more than the means, demand / means, at most 1 / r.
The weight is the factor times the resource's base value (`resource_value` 0x5ECD10), stored as an
f64 in the record (+0x60) and in the player (+0x128); the player's +0x160 is the first six weights'
running integer sum divided by 5 (six summed, five divided: as read).

### 1.6 The records (the computer's memory)

`ai_player_records` 0x692950: one `ai_player_record` (0x98 bytes, `structs.toml`) per colour.
Only the colour is set at a new game (`ai_player_records_init` 0x4340E0).

| Offset | What | Written | Read |
|---|---|---|---|
| +0x00 | the colour (i16) | game_init | every routine that takes the record |
| +0x04 | the Eye value (i32) | the start; a reveal (§1.4); `ai_hero_step` (T1a1c) | `ai_fight_estimate` 0x528040 returns it (T1a2) |
| +0x08 | the recruit reserve (i32[7]) | the start; the middle subtracts the income | the towns (T1a3) |
| +0x24 | the means (i32[7]) | the weights | the weights; probably `ai_town_visit`'s callees (T1a3) |
| +0x40 | the demand (i32[7]) | the weights | the weights; probably `ai_town_visit`'s callees (T1a3) |
| +0x60 | the weights (f64[7]; +0x90 gold, `ai_player_values` 0x6929E0) | the weights, start and middle | the values; `ai_secondary_skill_value` 0x524690 (Estates) |

Fields of the game's own records that the computer uses: the hero's +0x11C (asleep: the done
mark), +0x35 to +0x40 (the destination), +0x41 (the points beyond) and +0x43 (the must-go),
written by the destination's choice (§1.9), +0x44 to +0x46 (the patrol), +0x109 and +0x47A to
+0x491 (H3API's AI values; T1a2), +0x11D (cleared by the threat fold); the player's +0x39 (the
puzzle target, a packed position: §1.9), +0x128, +0x160, +0x164; the town's +0x3; the map item's
+0xD bit 0x08 (the boat marks, only during a walk); the pathfinder's +0x8, +0x20, +0x28, +0x58
and +0x6C, and its nodes' +0x10 and +0x14 (`pathfinder_node`, §1.9).

**What persists between turns** (decision 5's input): nothing in the record is carried into the
computer's own next turn, since the start recomputes every field from the game's state. But the
record is read **during other players' turns**: the gold weight by a computer hero's level-up
(`ai_secondary_skill_value`, ADV-STUB-8) and the Eye value by the fight estimate, so the values of
the computer's last turn decide there. In the game's records, the hero's destination (+0x35) and
+0x41 survive into the next turn: they widen its search (§1.4), the destination is favoured again
(× 1.5 + 20, §1.9) and the other heroes' destinations and +0x41 shape a hero's claims (§1.9);
+0x43 is cleared at the start, so a must-go lasts the turn. T1a4 completes the list.

### 1.7 Hidden reads, draws and floats met so far

- **Hidden reads:** the threat folds read every enemy hero's place and army, explored or not; the
  Eye value reads the whole map's objects; the walk's reveal tests the current player's own
  vision (not hidden). The pathfinder's fog test (`pathfinder_fog_test` 0x4B25D4, ADV-MOV-008)
  re-read: a computer player's step into an unexplored tile is refused only when it comes from an
  unexplored tile **and the player owns a town** (+0x3E, the town count): one step into the fog
  while it has a town, the fog ignored while it has none (the map's earlier summary had the test
  the other way round). The planning (§1.9): the danger map reads every enemy hero's tile and
  army; every reachable entrance is a candidate, explored or not, and while the player has no town
  an unexplored one is valued by its object (with a town, a flat exploration value); a guard's
  fight estimate is read for a candidate on an unexplored tile too. Without a town the computer
  thus ignores the fog in both its search and its values.
- **Draws:** none in §1.1 to §1.8's functions. The destination's choice draws once for every
  candidate it scores (§1.9), up to five searches per hero and pass plus the walk's: a turn draws
  many times from the game's generator (`crt_rand` through `random_between`). The first look's
  "almost none" counted call sites.
- **Floats:** x87 in `ai_resource_weights` (f64), `ai_turn_start_values` (the artifacts' mean, the
  difficulty weights, f32), `ai_candidate_score` (× 1.5 or ÷ 1.5 in f32 for the kept destination,
  (100.0 / r) × the score in f64 for a score not above 0, both truncated by `crt_ftol`), and the
  values T1a2 reads. The rates and the spread are i32, not saturated.
- **Asymmetries met** (for T1a3's table): the patrol of radius 0 that never moves; the boat marks;
  the garrison swap at a turn's end; the resources shared with allies; Easy without a human ally
  skips the danger map; the computer's sleep lasting one turn. The planning adds none: the danger
  map, the candidates and the choice are the policy's, and the pathfinder's computer mode (the
  tile buffer folded into its nodes) shapes only the computer's own searches. The walk's (§1.10):
  the boat bought without visiting the Shipyard, and a town's Shipyard bought mid-walk; the
  Castle Gate bought in both towns mid-walk and the jump made even when a purchase fails; Town
  Portal and Dimension Door applied directly (moved, mana spent; Town Portal 300 points, 200 at
  expert; Dimension Door none) instead of the human's cast; Fly, Water Walk and Summon Boat cast
  as a human casts them; the dig.

### 1.8 Not read yet

The walk's leftovers (§1.10): `ai_walk_path`'s branch for a Fly node not next to its from-tile,
the flag of `hero_move_step` that zeroes the points, how the pathfinder makes the walk's marks
(0x200, 0x400, 0x800, the Castle Gate's and Town Portal's nodes: `pathfinder_gate_exit` 0x56A730
among them) and reads the boat marks: T1a3 and T1a4. The values the frame and the planning call (`ai_vision_area_value`, the defence functor,
`ai_hero_effectiveness`, the puzzle's target, the list inside `ai_resource_weights`,
`ai_army_fight_sim` 0x427330, `ai_fight_estimate` 0x528040, the node's key +0x10 and the
expansion's sixth argument behind it, the per-class value table `ai_class_tables_init` fills):
T1a2. The build loop, the artifact step and the allies' shares: T1a3. `ai_hero_candidates`'
other caller, `ai_town_hero_prospects` 0x431BD0 (inferred): T1a3, T1a4.

### 1.9 The planning: the danger map and the destination (T1a1b)

Read whole: `ai_danger_map` 0x42DE50, `ai_hero_destination` 0x42E0B0, `ai_hero_candidates`
0x42EDD0 (but the values it calls), `ai_hero_reach_claims` 0x42F570, `ai_candidate_score`
0x42F980 and the small helpers after 0x42E0B0 (`position_equal`, `hero_position`,
`pathfinder_node_at`, `pathfinder_ai_tile_value`, `game_map_item`); of the pathfinder, the parts
of `pathfinder_relax` 0x4B1A70 that read the computer's tile buffer and set +0x20. Positions are
packed (the column in bits 0-9, the row in 16-25, the level in 26-29); costs are movement points.

**The danger map** (once per hero, when it is selected; §1.3 step 4). For every hero of every
player not defeated and not on the hero's team: a simulated fight of the hero against that hero's
army (`ai_army_fight_sim`, T1a2). When it comes out negative, that enemy's search from its tile
(its maximum points plus 300) marks every tile it reaches: the result is added to the tile's
dword, or −1 000 000 000 is written when the result is below −500 000 000. The map lives in the
turn's tile buffer, which the pathfinder reads during the hero's own searches (pathfinder +0x6C):
a node's danger (+0x14) is the tile's value while the cost is within the day's points, and past
them the lowest value on the path; a node past the day's points whose danger is at or below
−500 000 000 is not entered. A hero may cross danger today, but not plan through it beyond.

**The candidates** (`ai_hero_candidates`, from the hero's search with the budget of §1.4):

1. `ai_hero_reach_claims` runs the hero's search (its points left as start, mode 4 from the
   search and 5 from the walk) and returns the danger at the hero's own tile. Then for each other
   hero of the player that this search reaches, a search from that hero's kept destination (or its
   tile) with its day's points marks the **claims** buffer with the lowest cost at which a
   colleague gets to each tile (inside its patrol).
2. On a town's entrance threatened by two or more enemy heroes (town +0x3) the hero has no
   candidate; threatened by one, its only candidates are enemy heroes it reaches with its points
   left, each a **must-go**.
3. Else every tile the search reached, from the reached list's end (the candidates keep that
   order, which decides ties): an object's entrance (explored or not)
   or, while the player owns a town, an unexplored tile; inside the hero's patrol; not its own
   tile; for a class marked in `ai_claimed_classes`, only if no colleague claims it at less cost.
   Its cost is the node's second cost (+0x1A). Its value: an unexplored tile, while the player owns
   a town, 100, or 100 000 while the turn's flag F is set (before the turn's first reveal, §1.4);
   anything else `ai_fight_estimate` (T1a2). A value above 0 is kept; 0 only while the hero stands
   in danger (places to flee to).
4. The Grail: the player's puzzle target (+0x39), when reached, inside the patrol and not claimed
   more cheaply, is worth 5 000 000 under the "build the Grail" victory condition and otherwise
   the Grail's value to the player; its cost is at least the points left plus a full day.

**The value field.** Each candidate spreads its value over a dword per tile: an unexplored one
only on its own tile; an explored one over the tiles a 500-point search from it reaches, within 5
tiles each way: the whole value up to m, then value × 300 / (300 + cost − m). For a class the
hero visits from a neighbouring tile, m is the cost of the dearest of the 3 × 3 tiles around it;
otherwise 0. A candidate's field thus adds the values of the objects near it.

**The score** (`ai_candidate_score`). For an explored object visited from a neighbouring tile, the
candidate's cost becomes the cost of the tile it is visited from. The score is the field at the
tile, plus the node's key (+0x10, T1a2), plus its danger (+0x14; a danger at or below
−500 000 000 is softened to −2 500 000 for a score of 5 000 000 or more, so the Grail is sought
through danger), plus, for an object entered, the fight estimate of a monster guarding it. The
hero's kept destination gets × 1.5 + 20 (÷ 1.5 + 20 when negative) and stays a must-go if it was
one; every other candidate is multiplied by a draw of 76 % to 100 % (a negative score divided by
it). The **rate** is the score per 100 points of the candidate's cost (cost floored at 100), at
least 1 when the score is positive.

**The choice** (`ai_hero_destination`). The best rate starts at the danger of the hero's tile.
A candidate is **near** when it is an entrance next to the hero, or its cost is at most 21 % of
the day's points, but not when its node's second cost exceeds its cost, its score is negative or
the player has no town. In order: a must-go is taken at once and ends the choice; a valued
candidate beats a best without value; a near one beats a far one; among near ones the cheaper
node wins (equal: the higher rate); among far ones the higher rate; a candidate without value
counts only above the best rate and while the best has none. A hero standing on an object's
entrance (not its own town nor a Sanctuary) that finds nothing, out of danger and with the search
complete, steps to its cheapest free, unguarded neighbour out of danger. At the end the hero's
+0x35 holds the destination (none: −1), +0x41 the cost beyond the points left, +0x43 the must-go
flag; the rate is returned (§1.4 doubles the budget on a negative one).

**Two oddities for T1b's second look** (quirk candidates, not flagged): `ai_class_tables_init`
0x428070 marks `ai_claimed_classes` by the list's count, not its entries, so the claims apply to
classes 0 to 36 instead of the 37 listed (the artifact, the resources, the chests); and a
colleague's claims are offset by its +0x41 less its points left only when +0x41 is not larger
(16-bit), so tiles nearer its destination than that difference claim nothing. Both shape only
which of a player's heroes takes which target.

### 1.10 The walk (T1a1c)

Read whole: `ai_hero_walk` 0x42FEE0, `ai_walk_cast` 0x4309A0, `ai_walk_dimension_door` 0x430AB0
(its target's choice in outline), `ai_boat_create` 0x430F80; `ai_walk_path` 0x430610 and the step
`ai_hero_step` 0x42FC50 (with E24 M4's reading) in the parts the walk depends on. The walk follows
the path the pathfinder builds to the hero's destination (+0x35), node by node, with no wait
between steps; the node's flags carry the walk's marks (`pathfinder_node` +0x4).

- **The Grail:** a hero whose destination is the player's puzzle target and who stands on it digs
  (`hero_dig` 0x40EC90, the human's dig) if it has not moved today, else its points go to 0 and it
  waits for the next day.
- **The path:** none, and the points go to 0 (the hero is done). A first node marked 0x200: the
  hero visits the object it stands on, and the walk ends.
- **Jumps:** a Castle Gate node (0x100): building 22 is bought in the town the hero stands on and
  in the node's town when missing (`town_buy`, paid; the result is not tested, so the jump is made
  even when a purchase fails: whether the pathfinder ever offers such a jump is T1a3's), 100
  points are taken and the hero is moved; the walk ends. A Town Portal first node (0x40): the hero
  is moved, the spell's mana spent, 300 points taken (200 at expert); the walk ends. A Dimension
  Door (at a node marked 0x800, `ai_walk_dimension_door`): with the spell, off Cursed Ground, at
  least its cost plus 20 spell points and a cast left today, to the farthest waypoint within 9
  columns and 8 rows whose saving over walking (its cost less the cost so far, plus 200) reaches a
  threshold set by how many casts it can pay for (1000 % of the day's points for one, then 150,
  100, 75, 50 and 25 % for six or more; 200 points toward a Magic Well or Spring), or any when the
  stretch needs a jump; the mana is spent, no points; the walk ends.
- **Spells on the way:** a Fly or Water Walk stretch is cast for (as a human casts it) when the
  hero lacks the effect and has the points for the stretch up to the next waypoint; otherwise the
  walk ends (at the first node with the points zeroed). A boat node entered from land: a hero
  that can summon a boat casts Summon Boat on the walk's first step and stops (it plans again in
  the second pass); else a marked boat tile has its boat bought (`ai_boat_create`: the town's
  Shipyard bought first when missing, then 10 wood and 1000 gold, no visit).
- **The step:** the last one when the next node costs more than the points left. A step that ends
  on an object visits it and ends the walk (the hero plans again in the second pass); after a Hut
  of the Magi the record's Eye value (+0x04) is set to 0; after any visit the hero's values are
  recomputed (`ai_hero_effectiveness`, T1a2). A hero no longer the player's (lost) ends it.
- **Replanning:** after a waypoint (0x400) with 100 points or more left and no must-go, the
  destination is chosen again (`ai_hero_destination`, mode 5, the budget the cost of the eighth
  node ahead or the last, at most the points left, at least 350); a new destination rebuilds the
  path. The destination's cost (+8) is lowered by each step's cost.

No draw, hidden read or float of its own in the walk (the replans draw, §1.9).
