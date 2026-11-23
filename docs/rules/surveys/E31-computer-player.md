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
4. **The plan** (`ai_hero_plan` 0x42DE50; T1a1b), except on Easy for a player whose team has no
   human.
5. **The boat marks**, with at least 10 wood and 1000 gold: each of the player's towns marks its
   boat tile (map item +0xD bit 0x08) when its Shipyard is built, or can be bought now with the
   Shipyard's cost plus 10 wood and 1000 gold left over; each map Shipyard it owns marks its boat
   tile without a test. The marks are cleared once the walk is over. (How the search and the step
   use them: T1a1c; the step's boat is `ai_boat_create` 0x430F80.)
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
- **The search** (`ai_hero_replan` 0x42E0B0; T1a1b) runs up to five times. While it finds no
  destination, or a negative value without the struct's +0xD flag, and the pathfinder reports that
  it stopped short (+0x20), the budget doubles, up to 32000 or the patrol's bound.
- **No destination:** the hero is put asleep and its destination cleared.
- **The week's last day:** a hero on a town's entrance whose destination is worth less than 75
  stays (asleep) on day 7, to meet the new week's growth there.
- Otherwise **the walk** (`ai_hero_walk` 0x42FEE0; T1a1c).
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
mark), +0x35 to +0x40 (the destination), +0x41 and +0x43 (written by the search, T1a1b), +0x44 to
+0x46 (the patrol), +0x109 and +0x47A to +0x491 (H3API's AI values; T1a2), +0x11D (cleared by the
threat fold); the player's +0x39, +0x128, +0x160, +0x164; the town's +0x3; the map item's +0xD bit
0x08 (the boat marks, only during a walk); the pathfinder's +0x20 and +0x6C.

**What persists between turns** (decision 5's input): nothing in the record is carried into the
computer's own next turn, since the start recomputes every field from the game's state. But the
record is read **during other players' turns**: the gold weight by a computer hero's level-up
(`ai_secondary_skill_value`, ADV-STUB-8) and the Eye value by the fight estimate, so the values of
the computer's last turn decide there. In the game's records, the hero's destination (+0x35) and
+0x41 survive into the next turn and widen its search (§1.4); +0x43 is cleared at the start. T1a4
completes the list.

### 1.7 Hidden reads, draws and floats met so far

- **Hidden reads:** the threat folds read every enemy hero's place and army, explored or not; the
  Eye value reads the whole map's objects; the walk's reveal tests the current player's own
  vision (not hidden). The pathfinder's fog test (`pathfinder_fog_test` 0x4B25D4, ADV-MOV-008)
  re-read: a computer player's step into an unexplored tile is refused only when it comes from an
  unexplored tile **and the player owns a town** (+0x3E, the town count): one step into the fog
  while it has a town, the fog ignored while it has none (the map's earlier summary had the test
  the other way round).
- **Draws:** none in §1's functions.
- **Floats:** x87 in `ai_resource_weights` (f64), `ai_turn_start_values` (the artifacts' mean, the
  difficulty weights, f32), and the values T1a2 reads.
- **Asymmetries met** (for T1a3's table): the patrol of radius 0 that never moves; the boat marks;
  the garrison swap at a turn's end; the resources shared with allies; Easy without a human ally
  skips the plan; the computer's sleep lasting one turn.

### 1.8 Not read yet

The planning (`ai_hero_plan` 0x42DE50, `ai_hero_replan` 0x42E0B0 and its helpers): T1a1b. The
walk (`ai_hero_walk` 0x42FEE0, the step's points, the path, the spells cast on the way): T1a1c.
The values the frame calls (`ai_vision_area_value`, the defence functor, `ai_hero_effectiveness`,
the puzzle's target, the list inside `ai_resource_weights`): T1a2. The build loop, the artifact
step and the allies' shares: T1a3.
