# E31 — the original's computer player: survey

The reading behind E31 (`docs/epics/E31-original-computer-player.md`): what the original's
computer does in its adventure turns, as read from `Heroes3.exe` (GOG Complete, SoD's code,
SHA-256 `aac39e8c…`; `docs/exe/README.md`). Built up by increment 1's tasks: §1 the turn (T1a1a
the loop and the records, T1a1b the planning, T1a1c the walk), §2 the values (T1a2a to T1a2d), §3 the towns
and the answers (T1a3a to T1a3d), §4 the rest of the code (T1a4); T1b adds the wiki's and VCMI's
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
the turn's start and middle (`ai_enemy_reach_fold` 0x4280E0 to `ai_build_excluded_creatures` 0x429D50,
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

1. A hero standing on a town's entrance visits the town first (`ai_town_visit` 0x5253D0; §3.3):
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
  (`ai_town_threat_vtable` 0x63B670); the other (`ai_town_defence_vtable` 0x63B67C), for each of
  its towns the enemy could take, sets that hero's threat (+0x11D) and **buys** the town's available
  creatures into its garrison with the player's resources (`ai_town_defence_recruit` 0x428580: a
  rule, not a value; §2.4);
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
(`ai_build_excluded_creatures` 0x429D50) and `ai_town_build_choice` 0x42AE00 again and again while it
builds (§3.1); the hire (`ai_hero_hire_choice` 0x431360; §3.2); the weights again; and, when
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
| +0x04 | the Eye value (i32) | the start; a reveal (§1.4); `ai_hero_step` (T1a1c) | `ai_object_value` 0x528040 returns it (T1a2) |
| +0x08 | the recruit reserve (i32[7]) | the start; the middle subtracts the income | the build choice (§3.1); the visit ignores it (§3.3) |
| +0x24 | the means (i32[7]) | the weights | the weights; `ai_town_guild_buy` 0x42BEB0 (wood, ore; §3.3) |
| +0x40 | the demand (i32[7]) | the weights | the weights; `ai_town_guild_buy` 0x42BEB0 (wood, ore; §3.3) |
| +0x60 | the weights (f64[7]; +0x90 gold, `ai_player_values` 0x6929E0) | the weights, start and middle | the values; `ai_secondary_skill_value` 0x524690 (Estates) |

Fields of the game's own records that the computer uses: the hero's +0x11C (asleep: the done
mark), +0x35 to +0x40 (the destination), +0x41 (the points beyond) and +0x43 (the must-go),
written by the destination's choice (§1.9), +0x44 to +0x46 (the patrol), +0x109 and +0x47A to
+0x491 (H3API's AI values; §2.2, §2.4), +0x11D (the threat, §2.4); the player's +0x39 (the
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
among them) and reads the boat marks: T1a4. The values the frame and the planning call:
§2 (T1a2). The build loop and the hire (the step once called the artifact step): §3.1, §3.2
(T1a3a, T1a3b); `ai_hero_candidates`' other caller, `ai_town_hero_prospects` 0x431BD0: §3.2. The
allies' shares: §3.5 (T1a3d2).

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
   anything else `ai_object_value` (T1a2). A value above 0 is kept; 0 only while the hero stands
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

## 2. The values (T1a2)

What a hero's plan weighs (§1.9): the value of every candidate, the fight estimates of the danger
map and of a candidate's guard, and the hero's and player's values behind them. Read in four parts
(AI-Q25): §2.1 the object value and its per-class helpers (T1a2a), §2.2 the simulated combat
(T1a2b), §2.3 the army values (T1a2c), §2.4 the towns, heroes, artifacts and the frame's leftovers
(T1a2d); §2.5 gathers what T1b needs from all four. The details of each routine are in its map
entry.

**One currency.** Every value is an i32 in the creature AI value's units (creature record +0x40
times the count, `army_value`'s scale): resources through the player's weights (§1.5),
experience through the hero's +0x109, spell power, knowledge and mana through the hero's own
values (§2.4), artifacts through `ai_hero_artifact_value` or the player's mean, fights as army
value won or lost (§2.2). 5 000 000 marks a must (a victory condition's target, the Grail); at or
below −500 000 000 a never.

### 2.1 The object value (`ai_object_value` 0x528040; T1a2a)

What a hero's planning calls a candidate's value (§1.9) and a guard's fight estimate is one
function, read whole with its per-class helpers: the value to a hero of the object whose entrance
is a tile (formerly `ai_object_value`; `ai_fight_answer` 0x52BD10 returns it too, the computer's
answer to a monster's flee offer and a guarded pile). Its arguments are the hero, the tile and the
candidate's cost, an in-out i32 that several cases change (below). A tile that is not an entrance
(map item +0xD bit 0x10) is worth 0; otherwise a switch on the class (+0x1E): 67 handlers for 73
classes, the other 37 classes of 4 to 113 worth 0 (among them the Boat, the Border Guard, the
Cartographer, the Eye of the Magi, events, monoliths and gates, the Whirlpool, signs and bottles,
the Sanctuary, the Tavern, the Den of Thieves, the Trading Post and the random classes).

**Units.** Everything is an i32 in one currency, the computer's "value", with these exchange rates:

- **resources**: an amount times the player's resource weight (§1.5: its +0x128, f64), truncated;
  gold piles store hundreds; the player's +0x160 (the mean of the first six weights) stands for
  an unknown non-gold resource, the gold weight (+0x158) for gold;
- **experience**: points times the hero's experience weight (+0x109, f32, from
  `ai_hero_effectiveness`, T1a2d); **a primary or secondary skill point** as one level's
  experience (`experience_to_next_level` of the hero's level) times that weight; the Arena twice
  that; a Learning Stone 1 000 experience;
- **spell power, knowledge and mana**: the hero's own values +0x47E (power), +0x486 (knowledge),
  +0x48A (double mana) and +0x48E (extra mana) (T1a2d);
- **an artifact**: `ai_artifact_value_to_player` 0x433AA0 for a known one; an unknown one the
  player's mean artifact value (+0x164, f32), or a share of it for a chance (a Corpse a fifth, a Sea
  Chest a tenth, a Treasure Chest a twentieth, a Wagon two fifths);
- **creatures**: their AI value (creature record +0x40) times the count, only for those the army
  can take (`army_can_take`);
- **the army's strength**: `army_value` × (40 + `hero_primary_sum`) / 40, integer
  division: each primary point is 2.5 % of the army (power and knowledge count at least 1);
- **morale and luck**: E23's `ai_morale_change_value` 0x435850 (a fraction: 0.0173 a point above
  0, 0.0833 or 0.0122 below) of the change from the hero's current morale or luck, times the
  army's strength (`ai_morale_gain_value` 0x527CF0, 0 for an undead army; `ai_luck_gain_value`
  0x527D80);
- **a fight**: `ai_army_fight_sim` 0x427330 (T1a2b), negative when it is lost; a value at or below
  −500 000 000 is returned alone by the banks, mines and crypts (the candidate's "never").

**The table** (by class; "visited" is the hero's or the player's bit for that object, the hero's
+0x57 to +0x7B or the word's bits 5 + colour, `object_word_player_bit` 0x529690; such an object
is worth 0 once visited):

| Class | Value |
|---|---|
| 4 Arena | 2 levels' experience |
| 5 Artifact | its value, at least 10, by its pickup condition: free; guards (their fight + the value); 2 000 gold, 2 500 gold + 3 or 3 000 + 5 of a resource (`ai_paid_reward_value` 0x529810: 0 if unaffordable, else the value less the price); Wisdom or Leadership (0 without it). 0 with 64 artifacts in the backpack |
| 6 Pandora's Box | the guards' fight + experience + resources + primary and secondary skill points (a new skill only below eight) + artifacts × the mean + spells (with a Spell Book, `ai_learnable_spell_value` 0x5298D0: level ≤ Wisdom + 2, not known) + creatures; mana, morale and luck ignored |
| 7 Black Market | the sum over its seven artifacts of the value less its price in the cheapest weighted resource the player can pay (`ai_artifact_market_price` 0x524550), at least 0 |
| 10 Keymaster's Tent | 5 000 unvisited by the current player |
| 11 Buoy, 28 Faerie Ring, 14 Swan Pond | +1, +1, +2 morale or luck × the strength; reachable today only; the Swan Pond only when at most 200 points would be left, and it raises the cost to the day's points |
| 12 Campfire | its amount of its resource + 100 × that in gold |
| 16, 24, 25 Creature Bank, Derelict Ship, Dragon Utopia; 84, 85 Crypt, Shipwreck | the fight + resources + creatures + artifacts × the mean; 0 once emptied |
| 17, 20 dwellings | T1a2c (`0x529A30`) |
| 22 Corpse | a fifth of the mean artifact value (200 gold when the backpack is full) |
| 23 Marletto Tower, 51 Mercenary Camp, 81 Scholar | one level's experience |
| 29 Flotsam | 175 gold + 5 wood |
| 30 Fountain of Fortune, 38 Idol of Fortune | the bare morale or luck fraction, truncated: **always 0** (below) |
| 31 Fountain of Youth, 56 Oasis, 110 Watering Hole, 64 Rally Flag | the cost lowered by 200, 400, 200 or 200 points and +1 morale × the strength (the Rally Flag adds the bare luck fraction); 10 000 with the cost set to 0 when the bonus covers the path |
| 32 Garden of Revelation, 61 Star Axis | the knowledge or the spell power value |
| 33 Garrison | the team's own with removable units: an army exchange (0x42C4A0, T1a2c; 0 in a campaign whose id is below 7); an enemy's: its fight |
| 34 Hero | an enemy: the fight, plus when reached today `ai_difficulty_blend` × (its +0x11D + 10 000), plus, standing in a town, the town's value (0x529CB0, T1a2d; a hopeless fight becomes −1 250 000 when the player has no town); one's own: an army exchange, half when the visitor is the stronger (0 on Easy) |
| 35 Hill Fort | the AI value gained by the upgrades the player can afford slot by slot (the gold at `hill_fort_gold_factors` by level; no elemental upgrade in an RoE game); 0 when more than 500 points away and worth less than a third of the army |
| 37 Hut of the Magi | the player record's Eye value (§1.5) |
| 39 Lean-To | 3 × the mean resource weight |
| 41 Library of Enlightenment | 2 × (power + knowledge values) + the army's value / 10; needs level + 2 × Diplomacy ≥ 10 |
| 42 Lighthouse, 87 Shipyard | 1 000 when not the team's |
| 47 School of Magic, 107 School of War | the better of power and knowledge, or one level's experience, less 1 000 gold (0 without it) |
| 48 Magic Spring, 49 Magic Well | the double or extra mana value; farther than 300 points only while the hero is already headed to one |
| 52 Mermaid, 96 Temple | +1 luck, +2 morale × the strength |
| 53 Mine | its guards' fight + (`ai_difficulty_blend` of its owner + 1) × two days' income; under "flag all mines" + 5 000 000 / the mines' list count (which also holds the Lighthouses); 0 when the team's |
| 54 Monster | its fight (the exact count); + 5 000 000 if it is the "defeat a monster" target |
| 55 Mystical Garden | half of 500 gold + 5 gems while it holds its gift |
| 57 Obelisk | the Grail's value / the obelisks' count; 0 once the puzzle target is the Grail's true tile |
| 58, 60 Redwood Observatory, Pillar of Fire | `ai_vision_area_value` radius 20 (T1a2d) |
| 62 Prison | 2 500 gold + the prisoner's army value; 0 with eight heroes |
| 63 Pyramid | the fight against 40 Gold and 20 Diamond Golems + with Expert Wisdom the mean gain of the level-5 spells (not its own spell) |
| 78 Refugee Camp | buying its creatures (T1a2c's 0x42D780) |
| 79 Resource | the amount at its weight + its guards' fight |
| 82 Sea Chest | 1 200 gold + a tenth of the mean artifact |
| 83 Seer's Hut | unvisited: its reward's value, at least 20; visited: the reward less the quest's price if the hero can complete it before its deadline |
| 86 Shipwreck Survivor, 108 Warrior's Tomb | the mean artifact value (the Survivor's case reads the backpack only) |
| 88 to 90 Shrines | the spell to the hero (`ai_learnable_spell_value`) |
| 92 Sirens | the experience of the 30 % lost at (2 500 + the army left) / (40 × a level's experience) a point, less the strength lost |
| 93 Spell Scroll | 10 + guards + its spell's gain (as an artifact, 0x4336C0) unless the hero can cast it |
| 94 Stables | half the week's remaining movement bonus off the cost (base 50, or 10 000) + the Cavaliers' upgrade |
| 98 Town | T1a2d (`0x52AB80`) |
| 100 Learning Stone | 1 000 experience |
| 101 Treasure Chest | per tier the better of experience and gold at 32, 32, 31 % + a twentieth of the mean |
| 102 Tree of Knowledge | a level less its known price, or less the expected price capped at ⅔ |
| 104 University | T1a2d (`0x525BF0`) |
| 105 Wagon | 1¾ × the mean resource weight + ⅖ of the mean artifact |
| 106 War Machine Factory | the three machines' values less their prices (0 on Easy) |
| 109 Water Wheel | 500 gold a week stored once visited, else 1 000 |
| 112 Windmill | 4½ × the mean resource weight |
| 113 Witch Hut | its skill once known (`ai_secondary_skill_value`), else one level's experience |

**The cost.** The planning's rate divides a candidate's score by its cost (§1.9). Objects whose gift
is movement lower that cost by about half their bonus (the Fountain of Youth, the Oasis and
Watering Hole, the Rally Flag, the Stables), and set it to 0 with a value of 10 000 when the
bonus covers the path; the Swan Pond raises it to the day's points (it ends the day).

**Hidden reads:**

- the exact counts of every army it fights: monsters (word bits 0-11, 0x52A140), banks' and
  crypts' guards, Pandora's guards and creatures, mines', piles' and scrolls' guards, garrisons,
  enemy heroes' armies (0x528C5D), and an enemy hero's +0x11D (§1.6's scratch, T1a2d);
- a Pandora's Box's and an event's whole contents, a bank's rewards, a prisoner's army
  (0x52A3A0) and an unvisited Seer's Hut's reward (0x5735C6);
- the Grail's true tile, against the puzzle target, at an Obelisk (0x52A30B);
- not hidden: a Fountain of Fortune's luck, a Tree of Knowledge's price and a Witch Hut's skill
  are read only once the player's bit says it has seen them; the Pyramid's spell is never read.

**Draws:** none. **Floats:** x87 throughout, f32 and f64 mixed by case (the experience weight and
the mean artifact value f32, the resource weights f64; the Sirens' and the Hero's in f32), each
result truncated by `crt_ftol`; `ai_resources_value` 0x526C70 truncates after each resource.
**Asymmetries:** none: every case only reads.

**Quirk candidates** (for T1b's second look; each only shapes the computer's choices):

1. The Fountain of Fortune and the Idol of Fortune are worth 0: their morale and luck fraction
   (below 1) is truncated without the army's strength that every other source multiplies by.
2. The luck clamp moves the wrong argument (the Swan Pond, the Faerie Ring, `ai_luck_gain_value`
   for the Mermaid): above 3 the current luck is replaced by 3 − luck instead of capping the
   change, so a hero at luck 3 values +1 or +2 as a rise from 0 (`ai_morale_change_value` caps the
   change itself).
3. The War Machine Factory ignores a machine's price within 500 points.
4. Under "flag all mines" the bonus divides by the mines' list, Lighthouses included.

**Not read here:** the dwellings, the army exchanges and the Refugee Camp's purchase (T1a2c), the
fight (T1a2b), the town, the enemy hero's town, the University, the artifact, spell and vision
values (T1a2d), `seer_reward_value` 0x573A70 and the quests' own checks (T1a3, E12), how
`ai_fight_answer`'s 17 callers use the value (T1a3).

### 2.2 The simulated combat (T1a2b)

Read whole: 0x423EE0 to 0x427330 but the spell values and the container code, and the users
0x428410, 0x428710, 0x527710, 0x529890, 0x476200 (names in `map.toml`). Values are army-value units (fight value scaled), not hit points.

**A side** (`sim_combat`, 0x34 bytes; `sim_combat_stack`, 0x48, in `structs.toml`) holds its hero,
army, the opposing hero, spell points, whether it may cast (a spellbook, no Orb of Inhibition on
either hero, no anti-magic garrison), the special ground of the battle tile, Tactics (own less the
opponent's) and the walls it faces (Fort 4, Citadel 5, Castle 6, less Ballistics, at least 2, at
most 4; none if the hero has Earthquake). The side's **factor** is the caller's factor times
sqrt(1 + 0.05 a) × sqrt(1 + 0.05 d), a and d the hero's Attack over the enemy's Defense and its
Defense over the enemy's Attack (each at least 0). Per stack: **unit value** = fight value ×
sqrt(health with the hero's bonus / health) × factor, truncated; **stack value** = that × count;
the side's **strength** is their sum. Rounds to contact: 0 for a shooter, else
(speed + 2 × (7 − Tactics)) / speed, at most 4, raised to the wall level for a walker. Shares of
the value dealt per round: melee 0.2 (a shooter with the melee penalty 0.1), shot 0.2 × Archery
(× 2 striking twice, ÷ 2 against walls except creature 35). Stacks are sorted weakest first.
Morale, luck, war machines, retaliation and other abilities are not modelled.

**The fight** (`sim_combat_run` 0x426BC0): at most three rounds. Each round each side decides
whether to engage now (`sim_engage_now` 0x4267C0: a lookahead on copies over every engagement
round up to 4, scored by its strength left). The faster side casts first: one spell per side and
round, the best of spells 10 to 69 it can afford, by the spell's AI category (damage, mass damage
in which its own stacks count, other effects valued like the adventure AI's spell value with a
÷ 500, Resurrection and Animate Dead); Cursed Ground allows level 1 only, Recanter's Cloak level 2
at most; Familiars return a fifth of the cost. Then both deal damage at once: shots only while
neither engages, the engaging side's arrived stacks and shooters against the other's whole army
in melee. Losses spread over the stacks in proportion to their values; a count is the ceiling of
the value left over the unit value. After three rounds a stand-off: the larger hold value wins,
the other is wiped, the winner loses (loser/winner + 0.05) × loser. Equal values: the first side
(the attacker) loses.

**`ai_army_fight_sim`** 0x427330 (the estimate; no state changes): own factor the hero's +0x47A
(an f32, H3API's aiAggressiveness), halved for the victory condition's hero; the enemy's 1.25, or
with any human owner involved `ai_human_enemy_factor` 0x6604D0 (0.5, 0.5, 1.0, 1.25, 1.25 by
difficulty: the computer underestimates humans below Expert). A loss returns −1 000 000 000. A win:
the experience (enemy army's health total × Learning) converted by `ai_experience_value`
((2 500 + surviving army value) / (40 × experience to the next level)), plus Necromancy, less the
army value lost; a net loss under a quarter of the army's value counts 0. With an enemy hero or
town, plus the enemy army's value × `ai_difficulty_blend` (weight_a for a computer owner, weight_b
for a human, 0 for neutrals); plus 5 000 000 each for the victory condition's or the loss
condition's hero. **Negative**: the fight costs more army value than it brings; the danger map adds
it to the enemy's reach. 0x529890 adds it for an object's guards; `ai_town_capture_sim` asks
whether an enemy hero would take a town (1.25 against the garrison's 0.75, no garrison hero).

**`ai_quick_battle` is a rule.** When neither side is human, `adventure_battle` never runs a battle:
two draws `random_between(75, 125)` / 100 give the factors, the simulation decides the winner (the
attacker while its strength stays above 0), and `sim_battle_result` writes real outcomes: both
armies' survivors, a beaten side's hero leaving the game (`battle_aftermath`), spell points,
experience to the winner (the loser's whole army's health + 500 for a hero, × Learning), a third
draw `random_between(0, 99)` < 60 making a beaten hero a retreat (it keeps its artifacts, no 500),
else its artifacts to the winner, the loser's Grail removed, the town captured, Necromancy, and
Eagle Eye learning the loser's lowest-id spell (no chance draw).

**A second rule:** with Quick Combat (`battle_is_hidden`) a battle past round 30 is ended by
0x476200: `sim_battle_finish` 0x427210 re-fights the armies as they stood at the battle's start
(two more draws) and overwrites every army stack's count; the rounds fought do not enter.

1. **Hidden reads**: the enemy army's exact counts (0x427395; a hero's anywhere, neutrals, an
   object's guards 0x529890, a town's garrison 0x428410); the enemy hero's primary skills,
   Tactics, spell points, available spells, artifacts (0x423EE0, 0x424120, 0x425BD0); the town's
   fort level (0x424790).
2. **Draws**: 0x4270EC and 0x427110 (each `ai_quick_battle`), 0x426F3A (its result, both sides with
   heroes and the winner alive), 0x42723C and 0x427260 (each hidden battle past round 30). The
   estimate draws nothing.
3. **Floats**: f64 x87 throughout the stacks (factors, shares, sqrt, spread shares), each sum
   truncated by `crt_ftol` per stack; f32 in the stand-offs (0x4264D0, 0x4267C0), the factors'
   ÷ 100, Necromancy (+0.02 f32), experience × Learning, `ai_experience_value`, the blend.
4. **Asymmetries**: `ai_quick_battle` (computer against computer or neutrals) and the hidden
   battle's round-30 end decide casualties, winners, experience, artifacts and spells by the
   simulation and its draws, not by battle rules.
5. **Quirk candidates**: only the defender engaging fights both sides fully in melee (0x426D17);
   the mass-damage running total is carried from stack to stack (0x425FF6); Necromancy in the quick
   battle counts the loser's army after it was cleared, so it raises 1 (0x427003); a mutual wipe
   and an equal stand-off go to the defender; Eagle Eye takes a spell from the loser's available
   spells, not its cast ones.
6. **Not read**: the spell value leaves 0x4249D0 to 0x425B10, 0x423C80 and 0x423DE0, and the spell
   record's helpers (T1a4); 0x4E23D0, 0x4E5AA0, 0x4E5B80 (T1a4); the container code 0x4276C0, 0x427780,
   0x427A90, 0x427C30.

### 2.3 The army values (T1a2c)

**The army plan** (`ai_army_plan`, 0x3C bytes on the stack; structs.toml): an army and a spare, the
Angelic Alliance flag, the hero's own morale, the army's alignment counts, the AI value gained
(+0x18), a primary-skill advantage, the owner, a resources pointer and a list of 12-byte offers
(creature, count, free). Every value works on copies; none changes the game. Units throughout: the
creature's **AI value** (creature +0x40) x count, the scale of `army_value`.

**A stack's worth to the army** (`ai_stack_marginal_value` 0x42C830): v = AI value x count, then:
- *morale:* a creature of a new alignment, joining an army whose morale (the army's base morale
  plus the hero's) is below 1 (below 2 for a Necropolis creature), is **morale-bad** when the
  morale-feeling stacks (not flag 0x20000, not the Minotaurs) are worth at least 10 v;
- *speed:* slower than the army's slowest (capped at 20), v + (V x b / a - V) with V = the army's
  value + 500 and a, b the day's land movement at the old and new slowest speeds (f64, truncated);
- *the slot (the seven slots):* its type's stack if present; else the first empty slot, but with a
  spare army and six stacks the seventh stays free; morale-bad (or a forced swap) or no room: it
  **replaces** the least valuable stack (first on ties), less that stack's value; under morale-bad
  only a stack alone in its alignment; a new shooter with more than three shooting stacks only a
  shooter; a non-shooter never the army's only shooter (`ai_stack_to_replace` 0x42C690).
The Angelic Alliance counts Castle, Rampart, Tower, Stronghold and Fortress as one alignment
(Conflux and neutrals apart; `ai_army_plan_alignments` 0x42C060). With a spare army the plan first
**sheds** each stack whose own worth is not above 0 (`ai_army_plan_shed` 0x42C5B0).

**A dwelling's value** (`ai_dwelling_value` 0x529A30, the object value's classes 17 and 20): an ally's
dwelling is worth 0. Not its own and guarded: the simulated fight against the guards (T1a2b's
`ai_army_fight_sim`), returned at once when at most -500 000 000. Plus **the purchase**
(`ai_army_purchase_value` 0x42D780): same-type stacks merged, then greedy steps while one gains:
each step takes, over the dwelling's four creatures, the **affordable count** (the dwelling's count,
at most the current player's resources / cost per resource; level-1 creatures free and whole,
the visit flags the dwelling first), its marginal worth, less the cost weighed by the player's
**resource weights** (`ai_weighted_cost` 0x526CC0: cost x weight, truncated per resource) but
never by more than three quarters of the worth; the best step is bought into its slot and the
resources copy pays. So a payable purchase keeps at least a quarter of its worth. On its own
dwelling, with the candidate's cost at least 400 points, the purchase counts only when it adds at
least a third of the army's value. Under the victory "flag all dwellings" a dwelling not on its
team (its first creature of a faction) adds 5 000 000 / the map's number of dwellings.

**An army exchange's value** (`ai_army_exchange_value` 0x42C4A0, the object value's garrison and
hero cases): the other army is the spare; greedy transfers while one gains, each the best of the other
army's stacks (the whole stack; the other army's last stack only by a swap, or all but one
creature), its marginal worth x the **primary advantage** / 40: the hero's four primary skills
summed (each 0 to 99, Spell Power and Knowledge at least 1) less the other hero's (a garrison: 0),
at least 0. Equal or weaker heroes value an exchange at 0.

**Units.** The fight result (T1a2b's) and the purchase add as they are; the cost term is in the
weights' units (base value x demand factor, §1.5); the victory bonus a flat share of 5 000 000.

**The resource weights' demand, the part left** (0x4288A3 to 0x428C06): a list of the 145
creatures, each the count of its towns' available recruits (from the week's fifth day plus the
next week's growth) x its AI value, sorted highest first; the full costs of the first three join
the demand.

**Paying with trades** (`ai_pay_with_trades` 0x42A470; only with a flag the value path never
sets): a surplus above the larger of the recruit reserve and 20 (at most the means less the
demand) is traded at the market rate for the short resources, Marketplaces bought first.

1. **Hidden reads**: 0x529A9A the dwelling's whole record, the guards' exact army and the counts,
   of any dwelling reached, seen or not; 0x529C58 the map's number of dwellings; 0x42C4A0 the other
   army's exact counts and the other hero's primaries (as the object value passes them: a colleague's
   or a garrison's).
2. **Draws**: none in this part (the fight simulation's are T1a2b's).
3. **Floats**: 0x42CA78 to 0x42CA96 the speed loss, f64 from integers, `crt_ftol` (toward zero);
   0x526CEA to 0x526CFD the weighted cost, f64 weights, truncated after each resource; 0x42A66D
   and 0x42ACC0 the market rate f32 widened to f64; 0x42A900 to 0x42A915 the surplus' worth, f64,
   truncated per resource; 0x42AD13 to 0x42AD84 the trades: the factor (0x5ECD20, f64), the units
   given trunc(0.99999 + shortage x factor) (0.99999 f64 at 0x63B688), trunc(trunc(surplus /
   factor) x factor), the units got trunc(given / factor).
4. **Asymmetries**: 0x42AC20 (0x42AD82, 0x42AD96) the computer trades by writing its treasury
   directly at its market rate, then recomputes the weights; 0x42AB40 buys Marketplaces
   (`town_buy`) to trade. Both only on the hire path with trades (0x42D690's flag), not in a value.
5. **Quirk candidates**: (a) 0x42C8B6, 0x42C781: with the Angelic Alliance worn by any of the
   player's heroes, a creature of no faction (a neutral; RoE's elementals) indexes the nine-town
   set at -1 and the library's range check throws `std::out_of_range`; reached by a neutral
   dwelling's value or an exchange with neutrals; whether anything catches it: not read.
   (b) `ai_army_plan_put` 0x42C130: a displaced stack that finds the spare full overwrites the
   spare's least valuable stack, lost (harmless in a value; it matters: the visit's runs buy for
   real, §3.3 (c)). (c) `ai_dwelling_value` mixes the current player's treasury and team with
   the hero's owner's artifact and weights (the same on its own turn). (d) `ai_buy_marketplaces`
   tests the current player's resources, not the town owner's.
6. **Not read**: `ai_army_offers_add` 0x42CE30, `ai_army_plan_init_creature` 0x42CF50,
   `market_trade_factor` 0x5ECD20, `stack_base_morale`'s middle. Read at T1a3c (§3.3):
   `ai_army_exchange_apply` 0x42C3B0, `ai_army_arrange` 0x42D8E0, `ai_army_plan_run`'s town
   callers, `ai_army_plan_init_town` 0x42D1B0, `ai_army_merge_types` 0x42D870.

### 2.4 The towns, heroes, artifacts and the frame's leftovers (T1a2d)

Units: everything is in the fight estimates' units (creature AI value × count); resources enter
through the resource weights (player +0x128, f64, §1.5); 5 000 000 marks a must.

**A town** (class 98, `ai_town_value` 0x52AB80). Another team's or a neutral town: the capture
value (below); a visiting hero's army is simulated first and the result dropped. An ally's: the
visit value. Its own: `ai_own_town_army_value` 0x52B090 (A/2 + B of T1a2c's army plan with the
garrison and the owner's resources; 0 when the cost is 400 or more and the plan's +0x18 is below a
third of the hero's army) plus the visit value, plus 5 000 000 each for: the Grail carried to a
town that may build its structure (type 4 victory there; else the Grail's artifact value), the
transport victory's artifact carried there, a threatened town that is the capture victory's
target (else 5 000 000 / all towns on the map); plus the garrison hero's value of each Statue of
Legion part (118-122) the hero carries; and (above Easy, more than one hero, the hero 25 % above
a garrison hero's primary sum) 5 000 000 for the capture target when the hero is not the
player's strongest. Capped at 5 000 000.

**The capture value** (`ai_town_capture_value` 0x529CB0; also the hero case of an enemy hero
in a town): r = the simulated fight against the garrison (and its hero); r ≤ −500 000 000 is
returned when the player owns a town, else r becomes −2 500 000. V = 3 × the daily gold ×
the gold weight, + 3 × the Silo's weighted resources, + over the 14 recruit slots the count
(plus a week's growth when day of week + days to arrive ≥ 7) × (AI value − weighted cost, if
positive); V × (1 + the owner's difficulty weight: 0 neutral, weight b human, a computer, f32);
+ 5 000 000 when the player has no town, else 5 000 000 / the map's towns; + 5 000 000 for the
loss condition's town. Returns r + V.

**The visit value** (`ai_town_building_value` 0x52B1E0): the Conflux University's four schools
(`ai_university_value` 0x525BF0: 0 with 8 skills or under 2 000 gold, else the sum of the four
skills' values the class allows); a Mage Guild: no book 1 000 (with 500 gold), else the gain of
each unknown guild spell up to level 2 + Wisdom; the faction's +1 building once per hero and
town: Tower +0x486, Inferno +0x47E, Dungeon 1 000 × +0x109, Stronghold and Fortress the
experience to the next level × +0x109.

**The hero's values** (`ai_hero_effectiveness` 0x527760, H3API's fields): +0x109 = (2 500 +
army value) / (40 × experience to next level), per experience point (f32); +0x47E one Spell
Power, +0x486 a tenth of 30 mana (both at least 10), +0x482 one duration, +0x48E mana refilled,
+0x48A mana doubled, all as gains of the best spell value. A spell's gain
(`ai_hero_spell_gain` 0x527640) is its value less the best of the same AI kind (at least 1); a
school's (`ai_school_value` 0x524D20) the best-value gain with the level raised (Expert at once).

**Artifacts** (`ai_hero_artifact_value` 0x4336C0): the scroll its spell's gain; the Ballista
q·s/(q+s) with s = 500 √(Attack+1) × (1 + Artillery/2), q = army value × (primary sum + 40)/40;
the Ammo Cart a fortieth of the shooters' value; the Tent the largest stack value it can heal
(25 × First Aid hit points); any other 5 000 000 if the victory's artifact, plus its effect
objects' values (24 kinds, not read) and, for a combination, its parts'.

**The frame's leftovers.** `ai_vision_area_value` 0x432220: in a circle (dx² + dy² ≤ r² + r) of
one level, 1 per tile the colour has not explored plus `ai_class_scout_values` 0x6925AC (1 to
1 000, by class) for each object entrance on such a tile. The table is filled at start-up by
`ai_class_tables_init` 0x428070 from 75 pairs; only this function reads it. The defence functor
(0x428580, now `ai_town_defence_recruit`) is no value: for each of the computer's towns an enemy
hero reaches it merges the garrison's stacks and **buys** the town's recruits with the player's
resources (`ai_army_plan_run` 0x42D690, §2.3); the fold step also sets the enemy hero's +0x11D to
5 000 000 / own towns. `player_puzzle_target` 0x4BAE50: the puzzle grid is placed from the
Grail's true place; a target only when the uncovered share reaches 1.1 / 0.5 / 0.25 / 0 / 0
(Easy never, Expert and Impossible from the first piece); the pick (0x52CF10) not read. The node
key +0x10: 0 at the start, copied along each step; lowered by a teleport's lowest exit guard
answer, 500 for a Whirlpool, and a Castle Gate's unbuilt cost value.

1. **Hidden reads**: another's town's exact garrison and hero (0x529CF7), its recruits,
   Silo and buildings (0x529E0B), a visiting enemy hero's army (0x52ACDA); objects under the fog
   (0x4323CB); the Grail's true place for the puzzle grid (0x52CAF9; Expert/Impossible from one
   piece); every enemy hero's reach of the computer's towns (0x4282B0).
2. **Draws**: none in these functions (the black boxes 0x427330, 0x42C4A0, 0x42D780 aside).
3. **Floats**: 0x529CB0 f64 products truncated each step, the weight f32 with V rounded to f32;
   0x52B1E0 f32 (+0x109 × 1 000.0, the sum to f32, truncated); 0x4336C0 √ f64 × 500.0, Tent
   × 25.0 f32; 0x432220 √ f64 vs r + 0.5 (exact); 0x527760 f32 division stored; 0x52C9B0 share
   / 48.0 vs an f64 table; 0x428710 f32 constants.
4. **Asymmetries**: 0x428580 recruits into threatened towns at the turn's start and middle
   (its rules for T1a3); 0x524D20 writes and restores a skill level.
5. **Quirk candidates**: the visiting hero's fight dropped (0x52ACDA); a new school valued at
   Expert (0x524D20); the Ballista's q × s overflows 32 bits for large armies (0x4337C3); the
   week-roll test counts day 7 itself (0x529DEF); the University test needs 2 000 gold once but
   sums four skills (0x525BF0); the guild loop reads one level beyond the built one (harmless).
6. **Not read**: the 24 artifact effect methods (vtables 0x63B6B8..0x63B778, built by
   0x434100; T1a4); 0x52CF10, the puzzle's pick (T1a4).

### 2.5 What the values read, draw and change (for T1b)

- **Hidden reads: nearly everything.** The values and the fights read the true state wherever
  they look: the exact counts of every army they price (monsters, guards, garrisons, an enemy
  hero's army anywhere on the map, with its skills, spells and artifacts), the contents of
  Pandora's Boxes, banks, Prisons, dwellings and an unvisited Seer's Hut, an enemy town's
  garrison, recruits and buildings, the objects under the fog (`ai_vision_area_value`), the
  Grail's true tile (the Obelisk's test, the puzzle grid's placement), and the map's counts of
  dwellings, towns and mines. Each is listed in its section with its address. The fair variant
  (increment 6) must replace every one of them by what the projection shows; the few reads that
  wait for the player's own knowledge (a Fountain of Fortune's luck, a Tree of Knowledge's price, a
  Witch Hut's skill, the Pyramid's spell never) show the original's own line.
- **Draws:** the values draw nothing. The only draws met in T1a2 are rules: `ai_quick_battle`'s
  two factors and the beaten hero's retreat (§2.2), and the hidden battle's round-30 end (two
  more). With §1.9's draw per candidate scored, a computer turn's draws are the planning's and
  its battles'.
- **Floats:** x87 everywhere, f32 and f64 mixed by function, results truncated toward zero by
  `crt_ftol` (often after every step of a sum); square roots in the simulated combat, the
  Ballista's value and the vision circle. The port needs them under `kernel::num` (T1b's ADR
  draft): each site's width and rounding is in its section.
- **Asymmetries** (for T1a3's table; the owning lanes decide): (1) **`ai_quick_battle`**: a battle
  in which neither side is human is not fought; the simulated combat and its three draws decide
  the winner, the survivors, the experience, the artifacts, the Grail, the town, Necromancy and
  Eagle Eye (battle and adventure lanes; the epic's "battles through the system actor's seat"
  needs revisiting at T1b); (2) the hidden battle's end after round 30 re-fought by the
  simulation from the battle's start (the battle lane; it rests on this reading alone); (3)
  `ai_town_defence_recruit`: a computer buys recruits into every town an enemy could take, at its
  turn's start and middle (§3.3: straight into the town's army, a replaced stack dropped); (4) the trades written straight into the
  treasury on the hiring path (§2.3; T1a3).
- **Quirk candidates** (not flagged; T1b's second look): §2.1's four, §2.2's five, §2.3's four,
  §2.4's six. One may matter beyond the computer's choices: §2.3 (a), the Angelic Alliance's
  faction set indexed at −1 for a creature without a faction, which throws `std::out_of_range`
  (a crash in the original if nothing catches it). The rest only shape what the computer picks.
- **Units to keep apart in the port:** the experience weight +0x109 is the same expression as the
  fight's `ai_experience_value` (§2.2) and the Sirens' (§2.1); the army's strength
  (`army_value` × (40 + `hero_primary_sum`) / 40) recurs in the morale and luck values, the
  Ballista and the exchange's advantage.

## 3. The towns and the answers (T1a3)

What the computer does with its towns, and how it answers what the game's visits ask. Read in
four parts (AI-Q26): §3.1 the build choice (T1a3a) and §3.2 the hires (T1a3b), both in the
turn's middle (§1.5); §3.3 the town visit with the army plan's open pieces (T1a3c); T1a3d in two
(AI-Q28): §3.4 the answers the game's visits ask (T1a3d1), §3.5 the Seer's reward, the markets, the
allies' shares and the battle's split (T1a3d2), §3.6 the table of choices and rules for the owning
lanes. Units are §2's; the detail of each routine is in its map entry.

### 3.1 The build choice (T1a3a)

**When.** In the turn's middle, after the threat folds: `ai_build_excluded_creatures` 0x429D50
once, then `ai_town_build_choice` 0x42AE00 again while it builds. Each call scores every building
of every town not marked built today (+0x2) and buys the single best one, so a town gets at most
one building a day and the towns compete for the treasury in score order. The mark is the town
lane's rule (`E09-construction.md` §3: a computer-only team on Easy or Normal builds in a town
every second day).

**The excluded creatures** (a u8 per creature 0–144). A week's net per resource is seven days of
income less what a week's growth of all its towns' dwellings would cost. A creature is excluded
when it needs a resource other than gold whose net is 0 or less. **On Easy only**, and only for a
player with no human on its team, also every level-7 creature and every creature whose weekly
growth value (growth × AI value) would lift the player's own weekly growth value above the
strongest living human's: a handicap that reads the humans' towns.

**A building's value** (−1 drops it; 0 gives it no score of its own):

- −1: not allowed in the town, already built (the every-built set), or the Grail's building 26;
- **Citadel, Castle** (`ai_build_castle_value`): from the week's fifth day one creature's AI
  value per dwelling index with a growth (the Citadel and the Castle alike); 5 000 000 when the
  victory condition is to upgrade this town and its fort level is not built;
- **the halls 10–13** (`ai_build_hall_value`): the gold weight × 3 500, 3 500, 7 000, 14 000 (a
  week of each step's added income); 5 000 000 from the hall level the upgrade victory asks;
- **the Resource Silo**: seven days of its income at the resource weights;
- **a dwelling** (`ai_build_dwelling_value`): its growth × AI value, from the fifth day twice the
  growth plus the castle's share; **an upgrade** (`ai_build_upgrade_value`): the AI value
  difference × the base level's available creatures (plus a week's growth from the fifth day);
  **a horde building** and its upgrade: the horde's growth × AI value. Each of these adds the
  creatures' cost to the building's, so the score prices the army it would buy;
- **under threat** (an enemy hero reaches the town, +0x3; §1.5) the halls, the Silo, the
  dwellings and the hordes are −1; only the Citadel and the Castle stay, and three specials gain
  a value: the Escape Tunnel 5 000 with a garrison hero, the Glyphs of Fear and the Blood Obelisk
  the garrison's `army_value` / 20;
- **the faction specials**: the Treasury on day 7 only, the gold × the gold weight / 10; the
  Mystic Pond twice the mean weight; the Lookout Tower 100; the Necromancy Amplifier 1 000 per
  Necromancer (class 9) the player has; the Cover of Darkness 10;
- **everything else 0**: the Mage Guilds, the Tavern, the Shipyard, the Fort, the Marketplace, the
  Blacksmith and the other specials are built only as another building's requirement (and the
  Marketplace through the trades, §2.3; the Tavern through a hire, §3.2; the Grail's building
  through the visit, §3.3).

**The score.** Each building of value above 0 is taken with its unbuilt requirements, transitively
(`building_requirements` by faction; one not allowed drops it). Its cost is the creatures' cost
plus every one of those buildings'. `ai_build_score` 0x42A150 turns value and cost into value ×
1 000 / W, W the cost's worth at the record's weights (+0x60, f64, truncated at each resource).
When a resource it lacks has no income it also asks the market (`ai_trade_balance`,
`ai_trade_affordable_count`, T1a2c, neither trades) and gives −1 when trades cannot pay. The score
is added to every building of the chain, so a requirement scores the sum of what it opens.

**The purchase.** The best score above 0 among the buildings buildable now (`town_buildable_now`),
over all towns, the first town and the lowest id on ties. Then `ai_pay_with_trades` for its own
cost (it may buy Marketplaces and trade, §2.3); a town marked meanwhile gives up; the halls, the
Marketplace and the Silo need only the current player's resources to cover it, every other
building leaves the recruit reserve (record +0x8, §1.5) intact; `town_buy` (its computer branch,
`ai_town_buy_hook` 0x526D20, asks the market once more); the weights recomputed.

**Reads, draws, floats.** Its own towns and treasury; the threat count (§1.5's folds over the
enemy heroes' true positions); on Easy every human's towns' dwellings and growth (hidden); the
victory condition (public). No draws. Floats: f64 in `ai_build_score`, the halls, the Treasury and
the Silo (`ai_resources_value`), truncated toward zero.

**Choice or rule.** All choices: the purchase is `town_buy`, which tests what a human's build
does (`town_can_buy`, the cost), so the port issues the town lane's build command; its trades are
§2.3's (market commands before the build). The day's mark is the town lane's rule (above).

**Quirk candidates** (not flagged; T1b's second look): (a) value × 1 000 is computed in 32 bits,
so the victory's 5 000 000 wraps to about 7 × 10⁸ (still the largest score); (b) the Citadel and
the Castle are valued alike, one creature per dwelling index, not the growth they add; (c) the
Necromancy Amplifier counts Necromancers only, not Death Knights. Not quirks, but to keep in the
port: the halls, the Marketplace and the Silo ignore the recruit reserve; W is 0 only for a free
chain (none in the shipped data), which would divide by zero.

### 3.2 The hires (T1a3b)

**When.** Once a turn, in the middle after the builds (`ai_hero_hire_choice` 0x431360, its result
unused): at most one hero a day, always in one of its own towns' Taverns (`town_tavern_hire`, the
human's town hire without the dialog; the computer never hires at a map Tavern, whose visit is a
human's, `E09-tavern.md` §3).

**The limits.** No hire with 8 heroes, less than 2 500 gold, as many heroes as
`ai_hero_limits` 0x660518 allows (Easy 2, Normal 3, Hard 4, Expert 5, Impossible 6), or, once it
has a hero, while all living computer players' heroes together reach `ai_computer_hero_limits`
0x66052C (8, 11, 14, 17, 20): a cap shared by every computer.

**The offer.** Each of the two offers (player +0x28, +0x2C) is valued by its artifacts, worn and
carried: each artifact's best `ai_artifact_value_to_hero` among the player's heroes, at least 10.
The first offer is taken unless the second is worth more; equal, the larger primary-skill sum
(the first on ties). With no second offer the code compares against a stale stack slot holding
the player record's address (0x431745), so it never hires the first (a quirk candidate, rare:
only when no hero type is free).

**The town and the bar** (`ai_hero_hire` 0x431800). The hero's worth V: its carried artifacts as
above, its worn ones at `ai_artifact_value_to_player`, its army's cost at the weights. The bar T =
heroes owned × the gold weight × 2 500: with T above V and less gold than heroes owned × 2 500 no
hire. Each own town whose entrance is free (no visitor, as the human's tavern asks) scores V,
less the Tavern's weighted cost when it has none (then it must be buildable and payable now),
plus the hero's prospects there; the best town must score above T. A missing Tavern is bought
first (`town_buy`), the gold checked again, then the hire.

**The prospects** (`ai_town_hero_prospects` 0x431BD0): the hire played out on the real state and
undone. The hero is given the town's owner and 2 500 gold is taken; an army plan (§2.3) exchanges
its army with a copy of the town's garrison and buys from the town's recruits with the treasury;
the hero is put on the town's entrance and its candidates listed from there (§1.9's
`ai_hero_candidates`, a budget of 32 767). The sum of the candidates' values: those reached past
no guard count whole; those behind a guard (the node's +0xC) count through that guard, whose key
(its strength, below 0) gains their values and counts once it ends above 0. Then n = 1 plus the
own heroes on the town's level that the search reached, M the best of their nodes' keys (with the
guard on the way when negative); the prospects are (sum + M) / n, so a town that own heroes
already reach is worth less. The army, owner, resources and recruits are restored; the hero's tile
and its +0x109 are not (harmless for an offer that has no tile).

**Reads, draws, floats.** Every computer player's hero count; the offers' artifacts, skills and
armies; through the candidates, the true map (§2.5). No draws. Floats: f64 for V and T, truncated.

**Choice or rule.** Choices: the hire is the human's town hire with the human's tests (gold, 8
heroes, a free entrance), so the port issues the town lane's hire command (and a build command for
a missing Tavern). The hero limits are the computer's own choices, not rules of the game.

### 3.3 The town visit (T1a3c)

**When.** `ai_town_visit` 0x5253D0 runs at every visit of a town by a computer hero: its own, an
ally's or a captured one (`town_visit`, where a human gets the town screen), and at the start of
each turn for a hero standing on an own town's entrance (§1.3). Where a human works in the town
screen, the computer runs the steps below in a fixed order. Read in two parts (AI-Q27): T1a3c1 the
visit and its buys (main session), T1a3c2 the army (a sub-agent).

**The order** (the visiting hero H, the town T, the garrison hero G):

1. **The Grail.** H carries it and 26 is not built but allowed: it is built at once (no question,
   as the human's hall asks), then `victory_grail_test` 0x5F1EF0 (the game's: the condition's
   kind 4, any town of the player's team with 26 at its position) and the victory. No owner test:
   an ally's town gets the Grail, as the human's hall allows (`E09-construction.md` TD-11).
2. **In an own town:** `ai_victory_town_hold` 0x525200 (it may end the hero's day), then **with
   points left** (+0x4D) `ai_town_visit_recruit` 0x42BA60 (the garrison exchange, the recruits, a
   dwelling) and `ai_town_guild_buy` 0x42BEB0.
3. **In any town:** the spell book for 500 gold of H's owner, without a question, when T has a
   Mage Guild; then `town_heroes_learn_spells` for H alone (both E09 T42b, `E09-guild.md` §12).
4. **In an own town with points left:** the upgrades, the visiting-bonus building, the Artifact
   Merchants, and above Easy the artifact exchange with G and the war machines.
5. **In any Conflux town** with the Magic University: `ai_university` 0x524ED0.
6. `adventure_hero_stop`, then `town_visit_buildings` for G, then H (the town's rules).

**The victory town** (`ai_victory_town_hold` 0x525200). Only on Normal or above (game +0x1F6D8),
with at least two heroes, the town not threatened (+0x3 0), the victory condition "capture a town"
(6) naming this town (its tile, `town_at_entrance`), a garrison hero, if any, of a lower
`hero_primary_sum` than the visitor, and the visitor not the player's hero of the highest sum (the
last of equals). Then the visitor's points are set to 0 (0x525359: it stays, and so never recruits
on this visit), it exchanges with the garrison (below), and without a garrison hero it also takes
the garrison's better stacks (`garrison_merge` 0x44B820, the siege's merge by AI value x count) and
the garrison is emptied (`army_reset`, 0x5253C4): what the merge left there is gone. A lesser hero
parks in the town the victory names, with the garrison's best stacks.

**The exchange** (`ai_army_exchange_apply` 0x42C3B0): §2.3's exchange value played on the real
armies, no copies. The hero's same-type stacks merged; its worthless stacks shed into the other army
(`ai_army_plan_shed`, while the other has fewer than 7); then greedy transfers while one gains, each
weighed by the primary advantage / 40 (the hero's sum less the other hero's, the whole sum against a
garrison without hero, at least 0: an equal or weaker hero takes nothing but still sheds); the other
army keeps one stack; a displaced stack goes into it. Then the formation.

**The recruits** (`ai_town_visit_recruit` 0x42BA60). One army plan on the real armies: the hero's
army, the town's shown army as the spare (`town_garrison_army`: a garrison hero's, else the town's),
the hero's owner's resources (player +0x9C). (1) The exchange with the garrison as above (advantage
over the garrison hero, else the whole sum), the formation. (2) `ai_army_plan_run` with trades and
**costs not weighed** (+0x28 cleared, 0x42BBC5): offers the town's 14 counts above 0
(`ai_army_plan_init_town` 0x42D1B0; none free, unlike a dwelling's level 1), bought greedily while a
stack adds AI value, as many as the resources pay (the trades may first buy Marketplaces and sell
surplus, §2.3). (3) **A dwelling**, unless the town is marked built today (+0x2) and, on Easy, only
for a player with a human on its team: of the base dwellings 30–36 buildable now
(`town_buildable_now`) whose cost every resource covers, the one whose week's growth of its base
creature (creature +0x44), bought after paying the building, `ai_army_purchase_value` values highest
above 0 (the lowest id on ties) is built at once (0x42BDE0, `town_build_effects` with the mark and
the visit; the cost taken directly, not through `town_buy`, the recruit reserve not kept); (4) the
run once more on the town's counts, now with the new dwelling's growth. Never an upgraded dwelling
or a horde building here.

**Where the purchases go.** There is no commit: `ai_army_plan_run` works on whatever it is given, so
with a real army and the player's resources its purchases are real. `ai_army_purchase_step` 0x42D420
puts the stack (`ai_army_plan_put` 0x42C130: its type's slot, an empty one, or over the stack it
replaces, which goes to the spare, over the spare's weakest when the spare is full) and takes cost x
count from the resources; the run writes each offer's count back to the town (+0x16). No routine of
the human's recruit is called.

**The formation** (`ai_army_arrange` 0x42D8E0, was `ai_army_plan_commit`): after every exchange,
never after a purchase. The stacks sorted by the creature's base speed (+0x50), lowest first, stable
(`army_entries_insertion_sort_low` 0x435310); the shooters (flag 0x4) from the fastest into slots 0,
2, 4, 6, 1, 3, 5, the others from the slowest into the first empty slots.

**The other users.** `ai_town_defence_recruit` 0x428580 buys through the run alone: the town's shown
army, morale 3 (not a mode), no spare (a replaced stack is dropped), costs not weighed, trades, no
formation. `ai_town_hero_prospects` 0x431BD0 (§3.2 confirmed): the town's offers,
`ai_army_merge_types` 0x42D870 (was `ai_army_plan_setup`), shed, exchange steps, the formation, the
run without trades but with costs weighed (+0x28 1), all undone. `ai_garrison_visit` 0x524370: the
exchange with a map garrison (no hero). `ai_meeting` 0x525DC0: the exchange with a colleague, then
0x433FE0 both ways (both §3.4). `ai_battle_army_split` 0x42DB20 (`adventure_battle` 0x4AE290,
0x4AE2B8; §3.5): merges, values the enemy at the heroes' `hero_attack_defense_factor` ratio, splits
its stacks into free slots (0x42DD70), then the formation.

**The guild** (`ai_town_guild_buy` 0x42BEB0). The next level L (the town's guild level as the
building id) is bought when the hero could learn it (L below its Wisdom level + 2, at most the
fifth) and either its Spell Power and Knowledge are both 3 or more or, for the first level only, it
has neither Dispel nor Cure available. A higher guild in another own town that the hero could still
use and whose next level is buyable now takes precedence, except for a first level when the means
cover the demand in wood and ore (record +0x24, +0x40; §1.6). Paid through the market
(`ai_pay_with_trades`, §2.3), the day's mark respected.

**The upgrades.** Per level whose upgraded dwelling is built, each of H's stacks of that level's
base creature is upgraded whole when the current player's resources cover the cost difference ×
count in every resource; never a part of a stack, never G's or the garrison's.

**The visiting-bonus building** (not built today, `town_can_buy`; switch 0x5256E7): bought when its
worth to H is above the weighted cost (`ai_resources_value`): the Tower's Wall of Knowledge (23) for
H's Knowledge gain (+0x486), the Inferno's Order of Fire (23) for its Spell Power gain (+0x47E), the
Dungeon's Academy (23) for 1 000 × the value of an experience point (+0x109), the Stronghold's Hall
of Valhalla (23) and the Fortress's 17 (by its value the Cage of Warlords; the id not checked
against the data) for a level's experience × that value (f32), the Conflux's Magic University (21)
when `ai_university_value` (without its gold test) is above 0. Castle, Rampart and Necropolis have
none. H gains the bonus at the visit's end (`town_visit_buildings`).

**The Artifact Merchants** (`ai_artifact_merchants_visit` 0x5259E0; a Tower or Dungeon town only,
not the Conflux's). With them built, `ai_buy_stock_artifacts` 0x524400 buys from the game's one
stock (game +0x1F664) every artifact whose worth to the player (`ai_artifact_value_to_player`)
exceeds its weighted price, the best first, until none is left or the backpack is full; the price
and its resource from `ai_artifact_market_price` (§2.1) with the player's Marketplaces (at most 10)
as the rate; then `ai_artifact_received` arranges H's artifacts. Without them, they are built when
the stock's positive nets (for what the resources left after the building pay) sum to at least the
building's weighted cost. The Black Market's visit buys the same way (rate 5).

**The artifact exchange** (above Easy, with G; `ai_heroes_artifact_exchange` 0x433FE0, H to G then G
to H; also `ai_meeting` 0x525DC0's). Both heroes first take off into the backpack every worn
artifact whose removal raises their value (`ai_hero_shed_artifacts` 0x433BB0). Each of the giver's
worn artifacts (slots 0 to 16) is then offered: the receiver tries every slot that takes it
(`ai_artifact_offer` 0x433E20) and takes it into the slot of largest gain when that gain, plus the
giver's gain from wearing the displaced artifact in the freed slot, is above the giver's loss; the
displaced one goes to the giver or to the receiver's backpack. Then the giver's backpack, last place
first, each taken when it raises the receiver's value. The value (`ai_hero_equipped_value`
0x433C60): the army's AI value with each stack's morale and luck factors, times the attack/defense
factor, plus the two best spells' values, plus each worn artifact's own value.
`ai_artifact_received` 0x433B40 (after every artifact a computer hero gains) is the same arrangement
of one hero alone.

**The war machines** (above Easy; `ai_buy_war_machine` 0x525CA0): the faction's Blacksmith machine
(`blacksmith_war_machines` 0x6AA9F8: Ballista for Castle, Dungeon and Conflux, Ammo Cart for Tower,
Inferno and Stronghold, First Aid Tent for Rampart, Necropolis and Fortress) and in a Stronghold the
Ballista Yard's Ballista: bought when H lacks it, values it above 0 and the resources cover its
creature's cost; the building first when missing. The value is never weighed against the price.

**The University** (`ai_university` 0x524ED0, also the map's University): while H has fewer than 8
skills and the player 2 000 gold, the best of the four skills H may learn (its class allows it,
`ai_takes_new_secondary_skill` accepts it), at Basic for 2 000 gold.

**Reads, draws, floats.** Its own heroes, towns, garrisons and treasury; the difficulty, the victory
condition and, on Easy, whether its team has a human (public); the merchants' stock before any
merchant is built (a human sees it only in a merchant's window). No draws. Floats: none in the army
routines themselves (inside the runs the trades' and `ai_stack_marginal_value`'s, §2.3); f32 in the
experience buildings' tests; f64 in the stock's nets, the merchants' worth and
`ai_hero_equipped_value` (truncated at each step).

**Choice or rule.** All choices, each a human's command: the exchanges, merges and the formation
(stack moves between a hero and a garrison or inside an army; the emptied garrison and a dropped or
overwritten stack are dismissals; the victory hold's stop is the hero ending its moves); the
purchases (the town lane's recruit per creature and count: what a human's recruit tests, a full army
needing a move to the garrison first); the dwelling, the guild, the visiting-bonus building, the
merchants, the Blacksmith and the University (`town_buy` or the dwelling's direct build: the town
lane's build command, `town_can_buy`'s tests); the trades (market commands first); the book, the
upgrades (the town's upgrade of a hero's stack), the stock purchases (the market's buy), the war
machines (the Blacksmith's buy), the University's skills, the artifact moves between the two heroes
and into the backpack (the hero exchange). The Grail's and the book's dialogs are the human's
questions answered yes. Rules: the victory test, the spells learned, the buildings' visit effects.

**Quirk candidates** (not flagged; T1b's second look): (a) 0x5253C4: the victory hold destroys the
garrison stacks the visitor did not take; (b) 0x42D922: the formation keeps a count in 16 bits, so a
stack above 32 767 comes back wrapped (40 000 → −25 536, kept negative by `army_add_stack`; 65 536 →
0); (c) §2.3's (b) is live here: a purchase displacing a stack into a full garrison overwrites the
garrison's weakest, and the defence recruit drops a replaced garrison stack; (d) the first guild
level's spell test asks for neither Dispel (35) nor Cure (37), two Water spells of level 1, an odd
pair; (e) the fifth misc slot (18, SoD's) is never shed nor filled by the exchange (slots 0 to 16
only); (f) the University is built without the 2 000-gold test, so a skill may not be affordable
after it; (g) a skill valued 0 is still bought (the test is at least 0). Not quirks, but to keep in
the port: no recruits, upgrades or buys with 0 points left (0x52548D; the exchange with the victory
town still runs, and the next turn's start visits again); the visit's purchases weigh no costs and
ignore the recruit reserve, as does its dwelling (like the halls, §3.1); the Conflux's merchants
unused; the Blacksmith's machine bought at any worth; a refused artifact goes back to the first slot
that takes it.

### 3.4 The answers to the visits (T1a3d1)

**Where.** Wherever a visit asks a human a question or opens a dialog or a screen, the game's code
calls the computer's answer instead. Read in two parts (AI-Q28): T1a3d1 these answers (the main
session), T1a3d2 the Seer's reward, the markets, the allies' shares and the battle's split (§3.5, a
sub-agent). The game calls 43 functions of the computer's two ranges; besides the answers below
they are the turn (`ai_player_turn`, §1), the town visit and the artifacts' arrangement (§3.3), the
University (§3.3), the level-up (`ai_level_up_choice` 0x52BBD0, E08 T24a1), the gold or the
experience of a chest, a Tree of Knowledge or a School of War (`ai_gold_or_experience` 0x527FE0,
E08 T55a1: the gold when its weighted worth beats the experience's), the quick battle and the
simulated battle's end (§2.2), the battle AI's spell values (`ai_caster_record`,
`ai_best_spell_value`, E23), and code of the game's own that lives there: `sirens_losses`
0x527EC0 (the Sirens' rule for every player), `hero_army_power` and `army_power` (the monster's
popup, the Thieves' Guild), `team_has_human`, `ai_map_terrain_shares` (at a new game and a load),
`town_has_building`, `hero_position`, `game_map_item`, `game_hero`, the records' setup and
`ai_artifact_effects_clear` 0x434590 (at the shutdown), the vector helpers, and
`ai_army_plan_destroy` 0x4286E0 (the exception unwind's). Outside the ranges, two quest methods
call into them as the quest's price to a computer (each quest type's vtable +4, `quest_vtables`
0x64174C; §3.5's Seer): `ai_quest_artifacts_price` 0x56F5A0 (type 5, the artifacts' values) and
`ai_quest_resources_price` 0x571560 (type 7, the amounts' weighted cost); the other types' are T1a4's.

**The fight answer** (`ai_fight_answer` 0x52BD10, 17 callers) is §2.1's object value for the hero,
with no state change. Fourteen visits that ask a human yes or no go ahead when it is above 0 and
decline at 0 or below: the guarded artifact and its two pickup conditions (fight, pay), the guarded
resource and Spell Scroll, Pandora's Box, the creature banks, Dragon Utopia, the Derelict Ship, the
Crypt and the Shipwreck, the Pyramid, a dwelling's and a mine's guards, the School of Magic, and a
fleeing monster (above 0 it is pursued and fought). No other test: the value already weighs the
fight, the reward and the price. The computer's pathfinder asks it three times, in its modes from
4: `pathfinder_entrance_guarded` 0x56A360 for a tile in a monster's zone (the map's guard flag, not
the fog), `pathfinder_target_weigh` 0x56AAD0 for a step onto a hero, a garrison or a monster, and
`pathfinder_teleport_exits` 0x56A400 for a guarded exit: at or below −500 000 000 (a lost fight,
§2.2's −1 000 000 000 plus its gains) the path stops there (a teleporter's whole channel is
dropped); below 0 the value is added to the node's value (+0x10) and the tile kept as its target
(+0xC), so a path through a costly fight is worth less.

**The answers**, each where a human gets a dialog:

| Visit | The computer's answer | What it does |
|---|---|---|
| Garrison (`visit_garrison` 0x4A9061; a human `garrison_exchange_dialog`) | `ai_garrison_visit` 0x524370 | only a garrison of its own colour (a team-mate's: nothing) whose troops may be taken out (+0x3C), never in RoE's campaigns (index below 7); §3.3's exchange with the garrison as the other army, no other hero |
| Meeting (`visit_hero` 0x4A2800, same team; a human `meeting_exchange_screen`) | `ai_meeting` 0x525DC0 | after the Scholar's spells (the game's): the hero of the larger primary sum takes §3.3's exchange from the other (equal sums: no army exchange); then the artifact exchange both ways (§3.3). The other may be an ally's hero, a human's included: the visitor decides for both, as a human visitor does |
| Dwelling (`visit_dwelling` 0x4A18B0; a human `dwelling_offer_hire`) | `ai_dwelling_visit` 0x527B00 | its own or an unflagged one only, never an ally's; the army plan on the dwelling's creatures (level 1 free, as the human's) bought for real with trades (§3.3's run), the counts written back |
| Refugee Camp (`visit_refugee_camp`; a human the hire dialog) | `ai_refugee_camp_hire` 0x527E10 | the plan on the camp's creature (`ai_army_plan_init_creature` 0x42CF50), paid, with trades |
| War Machine Factory (case 106; a human the question and the hire dialog) | `ai_war_machine_factory_visit` 0x524FC0 | the Ballista, the Ammo Cart, then the First Aid Tent, each bought when §2.1's `ai_war_machine_value` 0x525120 is above 0 (never on Easy): the machine's creature's cost, one of each |
| Hill Fort (case 35; a human the upgrade dialog) | `ai_hill_fort_visit` 0x527BB0 | every stack with an upgrade (`creature_upgrade` 0x47B1A0; RoE's elementals excluded in an RoE game) upgraded whole in slot order when the resources cover it, at the human's price (`upgrade_cost` with the Fort's gold factors); the value never weighed |
| School of Magic (`visit_school_of_magic`; a human picks) | `ai_fight_answer`, then `ai_magic_school_choice` 0x527CD0 | Knowledge when the hero's Knowledge gain (+0x486, §2.4) is above its Spell Power gain (+0x47E), else Spell Power; 1 000 gold |
| Join for gold (`monster_join_for_gold`; a human asked) | `ai_gold_offer_answer` 0x527F60 | yes when the group's value (split over seven slots, `army_fill_split` 0x44AAA0) less the price × the gold weight is above the fight's estimate (`ai_army_fight_sim`, §2.2) |
| Join for free (`monster_join_free`) | accepted (E08) | — |
| A joining group, a reward's or a bank's creatures, with no free slot (a human `army_full_dialog`) | `ai_join_without_room` 0x52BC60 | the plan with the group free and no spare army: the group goes over the slot of best marginal value above 0, that stack lost; none above 0: the group lost |

**Prices.** Every purchase pays what a human pays at the same object: the creatures' costs, the
Hill Fort's `upgrade_cost` with its gold factors, the war machines' creature costs, 1 000 gold at
the School, and a boat's 1 000 gold and 10 wood. The last is only planned here:
`ai_boat_purchase_cost` 0x431160 prices for the pathfinder's boat node (pathfinder +0x14) a boat at
a tile where one of the player's towns places its boats (plus the Shipyard's cost when not built,
without `town_can_buy`) or one of its map Shipyards (player +0x8C), −200 000 elsewhere.

**Reads, draws, floats.** The guards', groups' and objects' true contents through §2.1's value and
§2.2's fight (§2.5), guards under the fog included (the pathfinder's guard flag is the map's); the
rest is the hero's own army, the player's treasury and weights, the difficulty and the edition. No
draws. Floats: f32 in the Hill Fort's gold (as the human's), f64 in the gold offer's test.

**Choice or rule.** All choices, each a human's command with the same tests and prices: the
exchanges (the garrison and hero exchange), the purchases (the dwelling's, the camp's, the
Factory's and the Fort's), the School's pick, the joins (accept, refuse, dismiss a stack for room),
the fights and refusals. Fewer moves than a human's: no exchange at a team-mate's garrison or in
RoE's campaigns, no hire at an ally's dwelling, no army exchange between heroes of equal primary
sums.

**Quirk candidates** (not flagged; T1b's second look): (a) `ai_meeting` with equal primary sums
exchanges no troops at all, while a sum one higher takes everything worth taking; (b)
`ai_boat_purchase_cost` counts a town's Shipyard without asking whether it can be built, so the
path may plan a boat the town cannot sell; (c) `ai_join_without_room` passes no spare army, so the
stack the group replaces is lost even when a garrison or another hero is at hand.

### 3.5 The Seer's reward, the markets, the allies' shares and the battle's split (T1a3d2)

Read by a sub-agent (AI-Q28) in its own worktree; merged by the main session. Units are §2's.

**The Seer's reward.** Runs twice: in the plan, `ai_seer_hut_value` 0x5735A0 values a hut through
`seer_reward_value` 0x573A70 (an unvisited hut's reward read, hidden, §2.1), and at the visit,
`seer_hut_visit` 0x573670 (visit_object's case 83) decides whether to complete the quest. The
visit sets the colour's visited bit first, runs the quest's check for the hero and, for the
computer, completes only when the reward's value less the quest's price (its vtable +4, §3.4) is
above 0 (0x5738FF);
a human gets the same check and a yes-or-no question. The value by the reward's kind (switch
0x573A89; the record's +5 kind, +9 a, +0xD b), in §2's units:
- experience: trunc(f32(a) x the hero's experience weight +0x109), without the Learning bonus
  that `seer_reward_give` 0x573C80 applies when it pays;
- spell points: a x the hero's knowledge value +0x486 / 20 (integer);
- morale: `ai_morale_gain_value` 0x527CF0 of a;
- luck: `ai_luck_gain_value` 0x527D80 of a, but its result is dropped: the case has no return and
  runs into the resource case (0x573B02), see the quirks;
- a resource: trunc(f64(b) x the owner's weight +0x128[a]);
- a primary skill: Attack and Defense b x one level's experience (`experience_to_next_level`) x
  +0x109 (x87, truncated), Spell Power b x +0x47E, Knowledge b x +0x486;
- a secondary skill (`ai_seer_skill_value` 0x524630): 0 when the hero has it at the reward's level
  or above, or when a new skill is not wanted (eight known, or `ai_takes_new_secondary_skill`
  0x524DD0 declines); else `ai_secondary_skill_value` of its next level, whatever level the
  reward gives;
- an artifact: 0 with 64 in the backpack, else `ai_artifact_value_to_player` 0x433AA0 (at least 10);
- a spell (`ai_seer_spell_value` 0x527AA0, a copy of `ai_learnable_spell_value` 0x5298D0):
  Spell Book, level at most Wisdom + 2, not known, then `ai_hero_spell_gain`;
- creatures: AI value x count, with no test of the army's room;
- no reward: 0, so the computer never completes a quest without one (a choice).
Hidden: nothing beyond the planning's read of an unvisited reward. Draws: none on the computer's
path (the human's no-quest message draws one of three texts, 0x57396B). Floats: x87, f32 for
experience and skills, f64 for resources, truncated. **A choice** (the human may accept or
refuse the same completion); the giving (`seer_reward_give`) is shared, except that creatures
without room go to `ai_join_without_room` 0x52BC60 (§3.4) instead of the human's `army_full_dialog`.

**The markets.** The map Tavern (visit_object's case 95, 0x4AA0E7): the computer returns at
once, no hire; it hires only in its own towns (§3.2). The Trading Post and the Freelancer's Guild
on the map: nothing (0x4AA2CE, 0x4A8E34). The computer's town visit (`ai_town_visit` 0x5253D0)
reaches no market but the Artifact Merchants; its resource trades are `ai_market_trades`
0x42AC20 (§2.3); it never sells artifacts or creatures. The artifact purchases
(`ai_buy_stock_artifacts` 0x524400, from `ai_black_market_visit` 0x5243F0 with the row 5 and from
`ai_artifact_merchants_visit` 0x5259E0 with the own towns with a Marketplace, at most 10) use the
same rate row as the human's screen (`black_market_trade` sets 5, `town_artifact_merchants_trade`
the same count) and the same seven resources, but not the same price.
`ai_artifact_market_price` 0x524550 truncates the gold price trunc(cost / rate) (0x524581) and
takes a resource's amount as that price twice over `resource_values` 0x68C482 (250, 500, 250,
500, 500, 500, 1), which is exactly twice `resource_values_artifacts` 0x68C492 (125, 250, 125,
250, 250, 250, 1): floor(trunc(cost / rate) / value). The human's `market_buy_artifact_proc`
0x5ED9E0 rounds half up: trunc(f32(cost / f32(value x rate)) + 0.5) (0x5EDCCF). So the computer
pays the same or one unit less, never more (all costs 50 to 20 000 by 50, every row and resource:
61 % one less). The f32 rates 0.2, 0.3, 0.4, 0.55 and 0.6 lie just above their decimals, so an
exact quotient falls just below its integer: an artifact of cost 1 000 at a Black Market costs
the computer 2 499 gold, 19 wood or ore or 9 of a rare resource, the human 2 500, 20 or 10. The
payment is written into the treasury (+0x9C) as a human's Trade does. Which resource pays is the
computer's choice (the least weighted cost among those it holds enough of). **A rule asymmetry**
(the price); the choice of item and resource is a choice. No draws; floats x87 with f32 rates.

**The allies' shares.** `ai_turn_middle` 0x428DD0 calls `ai_ally_resource_share` 0x429110 for
each living player of the computer's team (computers first, then humans) when one of its towns
has a Marketplace. Per resource the share is the giver's surplus (means +0x24 less demand +0x40),
held to half the difference between its amount and the ally's (truncated toward 0) and to its
amount less 20 (gold: less 10 000), less its recruit reserve (+0x8); dropped below 5 (gold
1 000) and unless it is at least five times what the ally holds (0x429228). A computer ally's
weights are recomputed (`ai_resource_weights` on its record, during the giver's turn) and each
share is held to its demand less its means. A human ally gets every share as it stands
(0x42933B), written from treasury to treasury, with no question: a message with the gifts'
pictures (GENRLTXT 359) and one naming the giver's shortages (360, 361), each with the turn
timer's 15-s timeout, no answer read. Hidden: the ally's treasury, and a computer ally's plans.
No draws, no floats. **A choice**: a human with a Marketplace can transfer resources 1:1 to any
player (market screen 1, `market_transfer_proc` 0x5ED550); the computer does it without the
screen and only to allies.

**The battle's split.** In `adventure_battle` 0x4AD470, after `battle_side_setup` stores the armies
and before the battle places the stacks, `ai_battle_army_split` 0x42DB20 reshapes a computer
side's army: a computer attacker's always (0x4AE290), a computer defending hero's only when its
Tactics is above the attacker's (0x4AE2A0); only on the path with a human side (a battle of
computers alone is resolved at 0x4AD5EE). It merges same-type stacks, counts the free slots,
values each enemy stack at trunc(AI value x count x f), f = f32 of the enemy hero's attack and
defence factor (`hero_attack_defense_factor` 0x4E5400, 1.0 without one) over its own hero's, and
keeps M, the strongest such stack, and the enemy shooters' sum S and count n. Each own shooter is
cut into parts of about 5M (`ai_army_split_into_free_slots` 0x42DD70: AI value x count / T parts,
at most the free slots + 1 and the count, filling empty slots in order); when the enemy has
shooters and its own shooters' raw value A is below S, its other stacks are cut into parts of
about M, into at most (n - A x n / S + 1) / 2 less the number of its non-shooting stacks new
slots. `ai_army_arrange` 0x42D8E0 then orders the army (§3.3). Hidden: the enemy's exact counts
and hero, read before the battle. No draws; f32 factor, x87 products, truncated. The attacker's
split is **a choice** (a human splits and merges in the hero screen before attacking); the
defender's is **an asymmetry**: a human defender's army enters the battle as it stands.

**Quirk candidates** (not flagged; T1b's second look): (a) 0x573B02: the Seer's luck reward is
valued as a resource of index a (its luck amount, upper bytes unwritten by the loader, 0x5747B8) and
amount b (never written, `seer_hut_record_init` 0x573580): leftover stack data (what the stack held
there not traced), possibly an out-of-range read of the weights; with zeros there the value is 0 and
the computer never completes such a quest; (b) 0x42DDA1: the split's threshold read as i16: above
32 767 it wraps (no split, or the most parts), and a zero low half or an empty enemy army divides by
zero; (c) 0x573CF9: a spell-points reward whose total falls below 0 adds the hero's mana to itself
(unreachable with the map's unsigned amounts).

### 3.6 Choice or rule: the table (T1a3d)

Every `ai_*` call read in §1 to §3.5, sorted into two kinds. A **choice** is a move a human player
could make with the same command, tests and prices: the port issues it as that command, and the
owning lane's rule decides it. A **rule** is something the original gives or does to a computer
player (or for a computer's side) that a human's command cannot: the owning lane decides whether
the engine copies it (decision 2 of the epic: the computer's rules are built by their lanes), and
the port cannot reach it through a command. A third kind, **reads and memory**, changes no rule but
matters to the port and the fair variant. The rows go into the owning lanes' inboxes at T1b.

**Rules** (asymmetries):

| # | The original (address, §) | What a human gets instead | Lane |
|---|---|---|---|
| 1 | `ai_quick_battle` 0x4270C0 (§2.2): a battle with no human side is not fought; the simulated combat and three draws decide the winner, the survivors, experience, artifacts, the Grail, the town, Necromancy and Eagle Eye | every battle with a human side is fought by the battle rules (Quick Combat too) | battle, adventure (and ai: "battles through the system actor's seat" revisited at T1b) |
| 2 | `sim_battle_finish` 0x427210 from 0x476200 (§2.2): a hidden (Quick Combat) battle past round 30 is re-fought by the simulation from its start, its two draws overwriting the counts | the same for a human's Quick Combat: a rule of hidden battles, not of the computer, met in its code | battle |
| 3 | `ai_boat_create` 0x430F80 (§1.10): a marked tile's boat bought from afar, at a map Shipyard without its visit | a map Shipyard's boat is bought at the Shipyard's dialog by a visiting hero (a town's Shipyard: the town screen, a choice) | adventure |
| 4 | `ai_walk_dimension_door` 0x430AB0 (§1.10): Dimension Door applied directly, the mana spent and no movement points | the cast (`adventure_spell_cast` 0x41D090) takes 300 points, 200 at Expert | adventure |
| 5 | The Castle Gate's jump (§1.10): made even when a purchase of building 22 fails (the result not tested); whether the pathfinder offers it with a gate unbuyable is T1a4's | the jump needs the Gate built in both towns | town, adventure |
| 6 | The patrol (§1.2, §1.3): a placed computer hero with a patrol radius stays inside it, radius 0 never moves | a human's heroes have no patrol | adventure (the map's field kept), ai (the policy honours it) |
| 7 | `ai_artifact_market_price` 0x524550 (0x524581, §3.5): an artifact at the Black Market or the Artifact Merchants costs trunc(cost / rate) gold, a resource's amount that floored over twice its value | the human's price is rounded half up (`market_buy_artifact_proc` 0x5ED9E0, 0x5EDCCF): the computer pays the same or one unit less (61 % of costs one less), never more | adventure (the Black Market), town (the Merchants) |
| 8 | `ai_battle_army_split` 0x42DB20 (0x4AE2A0, §3.5): a computer defending hero with the better Tactics merges and splits its army after the battle's sides are set, before the stacks are placed (only in a battle with a human side) | a human defender's army enters as it stands (the attacker's split is a choice: a human splits in the hero screen before attacking) | battle, adventure (`adventure_battle`'s setup) |

**Choices** (each a human's command; the port issues the command, the lane's tests decide):

| The original (§) | The human's command | Lane |
|---|---|---|
| The heroes' order, the danger map (none on Easy without a human ally), the candidates, the destination and its draws, the walk's steps and visits (§1.2 to §1.10) | moves and visits | adventure |
| Fly, Water Walk, Summon Boat on the way, Town Portal (300 points, 200 at Expert; the human's cast `hero_relocate_spell` 0x41D360 holds the same two, 0x41D39E, not read further), the dig (§1.10) | the casts, the dig | adventure |
| The garrison at a turn's end, the swap (§1.3) | the town's visitor and garrison hero exchanged | town |
| The build choice, the halls, the Marketplaces bought for trades (§3.1, §2.3) | `town_buy` (the town's build) | town |
| The hires (§3.2) | the town's Tavern hire | town |
| The town visit: the exchanges and the formation, the recruits and the dwelling built, the guild, the upgrades, the visiting-bonus building, the merchants and their stock, the Blacksmith, the University, the book, the Grail (§3.3) | the town screen's commands | town |
| The defence recruit into threatened towns at the turn's start and middle, a replaced stack dropped (§2.4, §3.3) | the town's recruit into its garrison and a dismissal | town |
| The trades at the market rate written into the treasury (§2.3) | the Marketplace's trade; **the amounts' rounding to compare** with the human's screen (`market_resources_proc` 0x5ECFE0) before the port relies on it | town |
| The answers to the visits (§3.4): the garrison and hero exchanges, the dwelling's, camp's, Factory's and Fort's purchases, the School's pick, the joins, the guarded visits' yes or no, the pursued monster | the dialogs' commands with the human's prices | adventure |
| The Seer's quest completed when the reward's value less the quest's price is above 0 (§3.5) | the hut's yes | adventure |
| The allies' shares: surplus resources given to the team's players with a Marketplace, a human ally's written into its treasury without a question (§3.5) | the Marketplace's transfer to a player (`market_transfer_proc` 0x5ED550), which the receiver is not asked either | town |
| The stock purchases' artifact and paying resource (§3.3, §3.5; the price is row 7) | the market's buy | adventure, town |
| A computer attacker's split before a battle with a human side (§3.5) | the hero screen's splits and merges | adventure |

Fewer moves than a human's, by the policy (no lane acts): no exchange at a team-mate's garrison or
in RoE's campaigns, no hire at an ally's dwelling, no troops exchanged between heroes of equal
primary sums, the Conflux's merchants unused, no recruits, upgrades or buys with 0 points left, no
hire at a map Tavern, nothing at a Trading Post or a Freelancer's Guild, no artifact or creature
ever sold. One rule the human shares and the engine must allow: at a meeting of two heroes of one
team the visitor moves both heroes' troops and artifacts, an ally's (a human's) included
(`visit_hero` 0x4A2800; the adventure lane to check its meeting command). The quirk candidates of
§1 to §3.5 (T1b's second look) are not repeated here; rows 4, 5 and 7 may also be classified as
quirks by their lanes.

**Reads and memory** (the ai lane's, for the port and the fair variant):

- Hidden reads: §1.7, §2.5, §3.3 and §3.4 (the threats, the true armies and contents everywhere,
  the fog ignored without a town, guards under the fog on the path), and §3.5's (an unvisited
  Seer's reward, the allies' treasuries and a computer ally's plans, the enemy's exact counts at the
  split). The fair variant (increment 6) replaces each by the projection.
- State written and undone inside a value: `ai_town_hero_prospects`' hire (§3.2), 0x524D20's skill
  level (§2.4); written by the walk into the map's tiles: the boat marks (map item +0xD bit 0x08,
  §1.3). The port computes them on its own copies (T1b's ADR draft).
- The computer's own flags in the game's records: the heroes' sleep as a done mark cleared at each
  turn's start (+0x11C, §1.2), the destination (+0x35), the values +0x109 and +0x47A to +0x48E, the
  records of §1.6.
