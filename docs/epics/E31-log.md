# E31 — log

The record of E31's tasks (`docs/epics/E31-original-computer-player.md`), one section per task,
newest last. Grep the task id (`## T1a1a`); never read whole.

## T1a1a — the turn's loop and the records (2026-11-23, increment 1)

- **The cut** (AI-Q24, the user at T1a1's start): T1a1 as queued was about 16 KB with its own
  helpers, about E23's whole battle AI; three parts of about 5 KB each (the loop and the
  records; the planning; the walk), not two or one.
- **Read** (the survey `docs/rules/surveys/E31-computer-player.md` §1): `ai_player_turn` 0x525E80
  whole (the tile buffer, the start, the turn's flag, the two passes, the middle, the end);
  `ai_next_hero` 0x526A90 (free heroes first, then the least sum of primary skills; the garrison
  hero sent out with fewer than eight on the map); `ai_hero_turn` 0x5261F0 whole (the stationary
  patrol, Easy without a human ally skips the plan, the boat marks, the garrison swap at the
  end); 0x5267B0 (the search budget and its doublings, the day-7 wait in a town, the reveal that
  wakes every hero); the start `ai_turn_start` 0x4297C0 and `ai_turn_start_values` 0x527960 (the
  difficulty weights), the middle `ai_turn_middle` 0x428DD0; inside them `ai_eyes_value` 0x429910,
  `ai_recruit_reserve` 0x429AD0, `ai_resource_weights` 0x428740 (but its dwellings' list) and the
  reach folds' frame `ai_enemy_reach_fold` 0x4280E0 with the threat counter.
- **The records:** `ai_player_record` (0x98 bytes) in `structs.toml`: the colour, the Eye value,
  the recruit reserve, the means, the demand, the f64 weights (+0x90 the gold weight,
  `ai_player_values`); recomputed at each turn's start and middle, read during other players'
  turns (the level-up's Estates, the fight estimate). Fields added to the hero (+0x41, +0x43, the
  patrol +0x44 to +0x46, +0x109, +0x11C asleep, +0x11D), the player (+0x39, +0x128, +0x160,
  +0x164), the town (+0x3), the pathfinder (+0x20, +0x6C) and the map item's +0xD bit 0x08.
- **Mapped:** 42 entries: 35 new, seven rewritten (`ai_player_turn`, `ai_hero_turn`, `ai_next_hero`,
  `ai_player_records`, `ai_player_values`, 0x429110 and the fog test 0x4B25D4);
  `message_with_pictures` 0x429110 renamed `ai_ally_resource_share` (its only caller is the
  turn's middle); `ai_player_values` 0x6929E0 sized 0x430, which uncovered the artifact effect
  table 0x692E18 (`ai_artifact_effects`, now its own entry).
- **Found on the way:** the computer's code starts at 0x423EE0 (a third range of about 24 KB:
  the epic's first-look section corrected; T1a4 maps it); the fog test 0x4B25D4 reads the other
  way round from its entry (refused only from an unexplored tile while the player owns a town;
  the fog ignored without towns): the map entry corrected; ADV-MOV-008's text ("anywhere at some
  setting") stands, and the asymmetry goes to the adventure lane with T1a3's table.
  `ai_turn_counter` 0x691684 is a vestige (stepped, never read).
- **Not read:** the planning (T1a1b), the walk (T1a1c), the values the frame calls (T1a2), the
  build loop, the artifact step and the allies' shares (T1a3).

## T1a1b — the planning (2026-11-23, increment 1)

- **Read** (the survey §1.9): `ai_danger_map` 0x42DE50 (was `ai_hero_plan`), `ai_hero_destination`
  0x42E0B0 (was `ai_hero_replan`; called by the search and mid-walk), `ai_hero_candidates`
  0x42EDD0 and `ai_hero_reach_claims` 0x42F570 (were `ai_turn_part_b` and `_c`), `ai_candidate_score`
  0x42F980 with the draw (the site renamed `ai_candidate_draw`), the five helpers after
  0x42E0B0 (two confirm H3API's `H3Pathfinder::GetPathNode` and `H3Main::GetMapItem`),
  `hero_patrol_allows` 0x4E56E0, `ai_ui_yield` 0x5242D0 (the message pump, no state),
  `ai_class_tables_init` 0x428070 and its tables; `pathfinder_reset` 0x4B1530 inferred; of
  `pathfinder_relax` the danger's fold, the box and the writer of the pathfinder's +0x20.
- **Records:** `pathfinder_node` (H3API's `H3PathNode`: the position, the flags, the from-tile,
  the key +0x10 and the danger +0x14, the two costs) and `ai_candidate` (the 16-byte candidate and
  destination: the position, the value, the cost, near, must-go) new; the pathfinder's +0x8,
  +0x28 (the box: H3API's map width and height are its right and bottom) and +0x58 (the reached
  list) new, +0x20's writer read, +0x6C's readers; the hero's +0x41 `ai_points_beyond` and +0x43
  `ai_must_go` (renamed, read), +0x35's computer use; the player's +0x39, a packed position.
- **Found:** a turn draws once per candidate scored, many times (the epic's first look counted
  the one call site; its "Draws" line corrected); the danger map reads every enemy hero, and
  without a town the computer values unexplored entrances by their objects (with a town a flat
  100, or 100 000 before the turn's first reveal), so without towns it ignores the fog in its
  values as the fog test does in its search; the kept destination's × 1.5 + 20 in f32 and the
  negative score's division in f64. Two quirk candidates for T1b's second look (the survey
  §1.9's end): the claimed classes marked by count (0 to 36 instead of the 37 listed), and the
  claims' 16-bit offset that wraps near a colleague's destination.
- **Not read:** the values the planning calls (`ai_army_fight_sim`, `ai_fight_estimate`, the
  node's key +0x10 and the expansion's sixth argument behind it, the per-class value table):
  T1a2; the walk: T1a1c; `ai_hero_candidates`' other caller `ai_town_hero_prospects` 0x431BD0
  (inferred, mapped): T1a3, T1a4.

## T1a1c — the walk (2026-11-23, increment 1)

- **Read** (the survey §1.10): `ai_hero_walk` 0x42FEE0 whole, `ai_walk_cast` 0x4309A0,
  `ai_walk_dimension_door` 0x430AB0 (the target's choice in outline), `ai_boat_create` 0x430F80
  whole; `ai_walk_path` 0x430610 and `ai_hero_step` 0x42FC50 in the parts the walk depends on.
  Done in the session of T1a1b, on its branch (the user: a couple of small tasks at a time).
- **Mapped:** 10 entries: four new functions (`ai_walk_path`, `ai_walk_cast`,
  `ai_walk_dimension_door`, `hero_summon_boat_possible` 0x4E5550 inferred), the table
  `ai_door_thresholds` 0x660500 (it follows `ai_difficulty_weight_b` 0x6604FC, read through that
  address: that entry corrected), `ai_hero_walk` and `ai_boat_create` rewritten, `ai_hero_step`
  extended, 0x40EC90 renamed `hero_dig` (was `path_rebuild`; the map-client lane's inbox told).
- **Found:** the walk's own rules for T1a3's table: the boat bought without a visit, a town's
  Shipyard and the Castle Gate (building 22) bought mid-walk, the gate's jump made even when its
  purchase fails, Town Portal and Dimension Door applied directly (Town Portal 300 points, 200 at
  expert; the door no points), Fly, Water Walk and Summon Boat cast as a human casts them; the
  dig at the puzzle target; a visit ends the walk; the Eye value zeroed after a Hut of the Magi
  (the record's +0x04 write the queue asked for).
- **Not read:** `ai_walk_path`'s branch for a Fly node not next to its from-tile, the move's flag
  that zeroes the points, how the pathfinder makes the walk's marks and reads the boat marks
  (`pathfinder_gate_exit` 0x56A730 among them): T1a3, T1a4.


## T1a2 — the values (2026-11-23, increment 1)

- **Cut** at its start (AI-Q25): as queued about 45 KB with its helpers (39 KB unread), so four
  parts by the code's seams; the user: the main session reads one and three sub-agents the
  others, each in its own worktree, the main session merging. T1a2a the object value (main
  session, about 12.6 KB), T1a2b the simulated combat (about 14 KB), T1a2c the army values (about
  7 KB), T1a2d the towns', heroes' and artifacts' values and the frame's leftovers (about 6 KB).
- **Read** (the survey §2): `ai_object_value` 0x528040 whole (renamed from `ai_fight_estimate`:
  the value of any map object, a switch on its class; `ai_fight_answer` returns it) with its 40
  per-class helpers; the simulated combat 0x423EE0 to 0x427330 but its spell values,
  `ai_army_fight_sim`, `ai_quick_battle`, `sim_battle_finish`, `ai_town_capture_sim`; the army plan
  (the seven slots, morale, speed, shooters), the dwelling's purchase, the army exchange, the
  resource weights' demand list; the town's three values (capture, visit, own), the hero's values
  (`ai_hero_effectiveness`: H3API's AI fields agree), `ai_hero_artifact_value`, the vision area
  and its per-class table, the defence functor, the puzzle target, the node's key +0x10.
- **Mapped:** 150 new entries and 23 rewritten (overlaps between parts merged into one line each,
  the fuller reading kept); `structs.toml`: `sim_combat`, `sim_combat_stack`, `sim_spell`,
  `ai_army_plan`, `ai_army_offer`, `pathfinder_node` +0x10, the hero's AI fields +0x109, +0x11D
  (now `ai_threat_value`), +0x47A to +0x48E. Renamed: 0x528040 `ai_object_value`, 0x428580
  `ai_town_defence_recruit` (was `ai_town_defence_value`: an action, not a value).
- **Found:** `ai_quick_battle` decides every battle without a human side by the simulation and
  three draws, with real casualties, experience, artifacts and captures (a rule for the battle and
  adventure lanes; the epic's battle seat to revisit at T1b); a Quick Combat battle past round 30
  is re-fought by the simulation (the battle lane's; this reading alone); the defence functor buys
  recruits into threatened towns (a rule, T1a3); the values read the true state nearly everywhere
  (§2.5); no draw in any value; 19 quirk candidates for T1b, one a possible crash (the Angelic
  Alliance's faction set at −1, §2.3).
- **Process:** a scratch merge script over the four worktrees' `map.toml` lines by address; the
  main session's compact listings hid `push esi`/`edi`/`ebx` once and so misread two call sites'
  arguments (caught against T1a2d's signature, corrected before the merge): filter only the
  prologue and epilogue. Scratch scripts not committed.
- **Not read** (each section's last list): the spell values inside the simulation, the 24
  artifact effect methods, the puzzle's pick, the quests' checks and the Seer's reward value, the
  army plan's commit (0x42C3B0, 0x42D8E0), the town builders: T1a3, T1a4.
