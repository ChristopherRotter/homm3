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

## T1a3a — the build choice (2026-11-23, increment 1)

- **The cut** (AI-Q26, the user at T1a3's start): T1a3 as queued about 6 000 instructions in
  about 60 functions, so four parts (T1a3a the build choice, T1a3b the hires, T1a3c the town visit
  with the army plan's open pieces, T1a3d the answers, the market visits and the table); this
  session T1a3a and T1a3b, one pull request.
- **Read** (the survey §3.1): `ai_build_excluded_creatures` 0x429D50 (renamed from
  `ai_build_candidates`: a creature list, not candidates), `ai_town_build_choice` 0x42AE00 whole
  with its two switches, `ai_build_score` 0x42A150, the six value helpers 0x42B520 to 0x42B8B0,
  `town_horde_record` 0x5C1230, `ai_town_buy_hook` 0x526D20, `creature_cost_array` 0x54E7C0.
- **Found:** one building a call, the best score over all towns; the score is value × 1 000 over
  the weighted cost of the building and its unbuilt requirements, added to each of them; dwellings
  and hordes priced with the army they would buy; under threat only the Citadel, the Castle and
  three defence specials; the Mage Guilds, the Tavern, the Marketplace, the Blacksmith and most
  specials have no value of their own; on Easy a handicap (no level 7, no growth past the
  strongest human's) that reads the humans' towns. All choices (the build is `town_buy`, a
  human's tests); three quirk candidates (the ×1 000 wraps for the victory bonus, the Citadel
  valued as the Castle, the Amplifier counting Necromancers only).

## T1a3b — the hires (2026-11-23, increment 1)

- **Read** (the survey §3.2): `ai_hero_hire_choice` 0x431360 (renamed from
  `ai_turn_artifacts`: the step values the offers' artifacts but is the hire), `ai_hero_hire`
  0x431800, `ai_town_hero_prospects` 0x431BD0 whole (its army plan's calls read at their calls:
  T1a3c), `game_hero` 0x4317D0 (H3API's `H3Main::GetHero`), `pathfinder_construct` 0x4B1370 and
  `pathfinder_destroy` 0x4B13E0; the hero limits by difficulty (0x660518, 0x66052C); the player's
  tavern offers (+0x28) in `structs.toml`; `ai_army_plan_run`'s arguments corrected (the morale
  and the spare were "a mode, a word").
- **Found:** one hire a day at most, in an own town's Tavern only (bought first when missing);
  per difficulty at most 2 to 6 heroes, and 8 to 20 for all computers together; the offer by its
  artifacts' value to the player's heroes, then the primary skills; the town by the hero's worth
  plus its prospects there (the hire played out on the real state and undone: the army plan with
  the garrison and the recruits, then the candidates from the entrance, shared with the own heroes
  already reaching it), above a bar of heroes owned × gold weight × 2 500. Choices (the human's
  hire and its tests). One quirk candidate: with no second offer the first is never hired (a stale
  stack slot, 0x431745).
- **Process:** a cloud clone, `Heroes3.exe` staged from the user's repo folder; the setup's
  baseline ran in the main clone while the edits went to a `git worktree`. Scratch scripts over
  `scripts/exe.py` (a function list by range, a call tree, the game's calls into the computer's
  ranges: 43 callees, the input of T1a3d), not committed.
- **Not read:** the town visit and the army plan's open pieces (T1a3c); the answers, the market
  visits, the allies' shares and the table (T1a3d).

## T1a3c — the town visit (2026-11-23, increment 1)

- **The cut** (AI-Q27, the user at T1a3c's start): about 3 000 instructions with the nested
  helpers, so two parts along the code's seam; the user: the main session reads one, a sub-agent
  the other. T1a3c1 the visit and its buys (main session, about 1 650), T1a3c2 the army (a
  general-purpose sub-agent in its own worktree, about 1 350); its `map.toml` lines merged by
  address (no overlap), its `structs.toml` changes applied, its survey text folded into §3.3.
- **Read** (the survey §3.3): `ai_town_visit` 0x5253D0 whole (the switch of visiting-bonus
  buildings 0x5256E7), `ai_town_guild_buy` 0x42BEB0, `ai_artifact_merchants_visit` 0x5259E0,
  `ai_buy_stock_artifacts` 0x524400 (also the Black Market's), `ai_buy_war_machine` 0x525CA0 with
  `blacksmith_war_machines` 0x6AA9F8 and its initializer 0x5C2E20, the artifact arrangement
  (`ai_heroes_artifact_exchange` 0x433FE0, `ai_hero_shed_artifacts` 0x433BB0,
  `ai_hero_equipped_value` 0x433C60, `ai_artifact_offer` 0x433E20, `ai_artifact_received` 0x433B40,
  its name kept: other lanes' docs cite it), `ai_university` 0x524ED0, `victory_grail_test`
  0x5F1EF0, `hero_backpack_last` 0x4DBD90; T1a3c2: `ai_victory_town_hold` 0x525200,
  `ai_town_visit_recruit` 0x42BA60, `ai_army_exchange_apply` 0x42C3B0, `ai_army_plan_init_town`
  0x42D1B0, `ai_army_merge_types` 0x42D870 (was `ai_army_plan_setup`), `ai_army_arrange` 0x42D8E0
  (was `ai_army_plan_commit`), the library's vector and sort helpers; `ai_battle_army_split`
  0x42DB20 and `army_stack_morale`, `army_stack_luck` inferred.
- **Found:** no commit step: `ai_army_plan_run` buys for real when handed a real army and the
  player's resources, so the visit's and the defence functor's purchases are made there; the visit
  recruits with costs not weighed, builds a base dwelling directly, the next guild level, the
  faction's visiting-bonus building, the Artifact Merchants (Tower and Dungeon only) and the
  Blacksmith's machine, outside the build choice and ignoring the recruit reserve; above Easy the
  visitor and the garrison hero exchange artifacts by their equipped value; a lesser hero holds
  the town a capture victory names, emptying its garrison. All choices; seven quirk candidates
  (the garrison emptied, the formation's 16-bit count, a full garrison's stack overwritten, the
  guild's Dispel-or-Cure test, the fifth misc slot unused, the University built without its gold
  test, a skill of value 0 bought).
- **Process:** a cloud clone, `Heroes3.exe` staged from the user's repo folder; the setup's
  baseline (only the light gates: the code verified before) in the main clone, the readings in two
  `git worktree`s; a scratch merge script over `map.toml` by address, not committed.
- **Not read:** `ai_battle_army_split` 0x42DB20 past its frame and 0x42DD70 (T1a3d); whether
  `ai_artifact_market_price`'s prices are the human's (T1a3d's market visits).

## T1a3d — the answers, the Seer, the markets, the allies and the table (2026-11-24, increment 1)

- **The cut** (AI-Q28, the user at T1a3d's start): about 2 800 instructions plus the table, so two
  parts; the user: the main session reads one, a sub-agent the other, the main session writes the
  table. T1a3d1 the answers to the visits (main session, about 1 550), T1a3d2 the Seer's reward,
  the markets, the allies' shares and the battle's split (a general-purpose sub-agent in its own
  worktree, about 1 250); its `map.toml` lines merged by address, its survey text folded into §3.5.
- **Read, T1a3d1** (the survey §3.4): `ai_garrison_visit` 0x524370, `ai_war_machine_factory_visit`
  0x524FC0, `ai_meeting` 0x525DC0, `ai_dwelling_visit` 0x527B00, `ai_hill_fort_visit` 0x527BB0,
  `ai_magic_school_choice` 0x527CD0, `ai_refugee_camp_hire` 0x527E10, `ai_gold_offer_answer`
  0x527F60, `ai_join_without_room` 0x52BC60, `ai_army_plan_init_creature` 0x42CF50, the 17 call
  sites of `ai_fight_answer` 0x52BD10 with `pathfinder_entrance_guarded` 0x56A360 and the new
  `pathfinder_target_weigh` 0x56AAD0, `creature_upgrade` 0x47B1A0, `hero_owner_record` 0x4E56B0,
  and new: `army_fill_split` 0x44AAA0, `ai_boat_purchase_cost` 0x431160 (the pathfinder's boat
  price), `ai_artifact_effects_clear` 0x434590, `ai_army_plan_destroy` 0x4286E0. The game's calls
  into the computer's ranges listed again (43 callees; a scratch script, not committed).
- **Found, T1a3d1:** every answer is a choice at the human's prices: the fight answer goes ahead
  above 0 (fourteen visits), the pathfinder drops a fight at or below −500 000 000 and adds a
  smaller loss to the node's value; the garrison and the meeting run §3.3's exchange (the stronger
  hero by primary sum takes; equal sums none; an ally's hero, a human's too, moved by the
  visitor); the dwelling, the camp and a join without room run the army plan for real (a join's
  replaced stack lost, no spare); the Factory buys the three machines by their value (none on Easy),
  the Hill Fort every affordable upgrade in slot order, the School of Magic Knowledge or Spell Power
  by the hero's two gains, the gold offer when the group's value less its weighted price beats the
  fight. Fewer moves than a human's: no team-mate's garrison, no RoE campaign garrison, no ally's
  dwelling. Three quirk candidates.
- **Read, T1a3d2** (the survey §3.5; the sub-agent): `seer_reward_value` 0x573A70 with the new
  `ai_seer_skill_value` 0x524630 and `ai_seer_spell_value` 0x527AA0, `seer_hut_visit` 0x573670,
  `seer_hut_record_init` 0x573580, `seer_reward_give` 0x573C80 (was `reward_creatures_2`),
  `seer_hut_load` 0x574610 in part; the map Tavern's case (0x4AA0E7); `ai_artifact_market_price`
  0x524550 against the human's `market_buy_artifact_proc` 0x5ED9E0; `ai_ally_resource_share`
  0x429110; `ai_battle_army_split` 0x42DB20 with `ai_army_split_into_free_slots` 0x42DD70 and
  `army_stack_count` 0x44ACC0; nine sites. The main session added the quest prices the Seer's test
  calls through each quest's vtable +4: `ai_quest_artifacts_price` 0x56F5A0, `ai_quest_resources_price`
  0x571560. In all 42 map entries: 21 new, 21 rewritten (one renamed).
- **Found, T1a3d2:** the Seer's quest completed when the reward's value less the quest's price is
  above 0 (a choice; a skill valued at its next level whatever the reward gives); the computer
  hires nothing at a map Tavern and never sells; its artifact price at the Black Market and the
  Merchants is truncated where the human's is rounded half up, so it pays the same or one unit less
  (a rule); the allies' shares give a team-mate (a human too, without a question) the giver's
  surplus within four caps (a choice: the Marketplace's transfer); before a battle with a human side
  a computer attacker, and a defending hero with the better Tactics, merge and split their armies
  against the strongest enemy stack (the attacker's a choice, the defender's a rule). Three quirk
  candidates.
- **The table** (the survey §3.6): eight rules (the quick battle, the hidden battle's round-30 end,
  the map Shipyard's boat without a visit, Dimension Door without points, the Castle Gate's jump
  after a failed purchase, the patrol, the artifact price, the defender's split), every other
  `ai_*` call a choice with its human command and lane, the policy's fewer moves, the reads and the
  memory; the rows go to the lanes' inboxes at T1b.
- **Process:** a cloud clone, `Heroes3.exe` staged from the user's repo folder; the setup's
  baseline stopped (CI green on `master`), the scoped verify without original data; the sub-agent in
  a `git worktree` (only `map.toml`), its diff applied with `git apply --3way` (one conflict of
  neighbouring lines, both kept); scratch scripts over `scripts/exe.py`, not committed. The first
  scoped verify caught two survey addresses without map entries (0x56F5A0, 0x571560): read and
  added.
- **Not read:** the other quest types' price (+4) and the slot's other callers, the rest of
  `seer_hut_load`, the luck reward's unwritten fields (T1a4); the trades' rounding against the
  human's market screen (the town lane, §3.6).

## T1a4 — the rest of the code (2026-11-24, increment 1)

- **The cut** (AI-Q29, the user at T1a4's start): about 3 500 instructions plus the lists, so three
  parts; the user: the main session reads one, two sub-agents the others. T1a4a the artifact
  effects (a general-purpose sub-agent, about 1 800), T1a4b the start-up initialisers and the
  leftovers of T1a1c and T1a3d (a second one after it, about 1 300), each in its own worktree
  (only `map.toml` and `structs.toml`, its survey text to a scratch file); T1a4c the lists (the
  main session), merging both and writing §4. The 18 functions still `inferred` at the start met
  the bar and only needed a phase.
- **Read, T1a4a** (the survey §4.1): `ai_artifact_effects_build` 0x434100 and its list
  `ai_artifact_effect_list` 0x63AC7C (135 artifacts), the 24 effect codes' value methods and the
  base (`ai_artifact_effect_vtables` moved to its true start 0x63B6B0), the table's start-up and
  exit (0x432460, 0x432490); `ai_spell_offer_value` renamed `ai_artifact_effect_spell` 0x432F90
  (code 18; +4 is the spell). Found: no draw, no hidden read; the values in the hero's army value
  and its effectiveness fields; fourteen quirk candidates (the single-stat weapons one point low,
  the Thunder Helmet's and Titan's Cuirass's numbers, the Spellbinder's Hat as mana, the Spell Book
  and the Catapult worth 0, a worn and a candidate artifact valued differently).
- **Read, T1a4b** (§4.2): the twelve start-up initialisers (ten `std::bitset<10>` constants and
  two guard bits: no state), `ai_army_offer_init` 0x4286B0, the quests' price slot of every type
  (type 6 `ai_quest_creatures_price` 0x570480, the others 0) and its three callers (the Seer's plan
  and visit, the pathfinder's Quest Guard case), `seer_hut_load` 0x574610 whole (the reward's
  written bytes by kind, the load's three `crt_rand` draws), the walk's marks (where the pathfinder
  sets and reads each bit of `pathfinder_node` +4, `pathfinder_town_portal` 0x56B050 new, the
  Castle Gate's and the teleporters' nodes, the boat marks read at 0x4B2B01), `ai_walk_path`'s Fly
  branch and `hero_move_step`'s points flag (only the network's replay passes 1). Found: 0x400 is
  a place to stop, not a waypoint; the plan prices a Castle Gate at 0 points and a Dimension Door at
  300 (the walk takes 100 and none); an unvisited Quest Guard is passed with no check; seven quirk
  candidates.
- **Lists, T1a4c** (§4.3 to §4.6): every one of the 255 functions of the three ranges by phase (240
  `read`, 15 `inferred`, none unnamed: exit criterion 1); the draws (the six sites in the ranges,
  the game's draws on the computer's paths, three calls deep); the floats (86 functions, 1 097 x87
  instructions, 164 truncations, no SSE); what persists between turns. Read for it:
  `puzzle_pieces_revealed` 0x4BAF00 and `puzzle_origin` 0x4CEA70 whole, `puzzle_map_compute`'s
  gate, `hero_visit_here` 0x4ACCA0 (new). Found: **the computer's turn start re-seeds the game's
  generator** through its Grail target when the puzzle's share reaches the difficulty's threshold
  (every turn on Expert and Impossible on a map with a Grail): by the colour, then by the Grail's
  place, so the turn's later draws start from the same point each such turn (a quirk candidate);
  `ai_player_records` are set once at the program's start, never at a new game or a load (a
  quirk candidate: a level-up before the computer's first turn reads stale weights); the heroes'
  destinations carry into the computer's next turn, the record and the heroes' values into the
  other players' turns.
- **In all:** 90 new map entries (55 functions, 18 sites, 15 globals, 2 tables), 17 rewritten (one
  renamed, one table moved), the structs `ai_artifact_effect` and `seer_hut_record`; the survey §4
  and pointers in §0, §1.3, §1.6, §1.8, §2.2, §2.4, §3.4 and §3.6 brought up to date. 23 quirk
  candidates for T1b.
- **Process:** a cloud clone, `Heroes3.exe` staged from the user's repo folder; the setup's
  baseline lost to a VM restart and replaced by CI's run on `master`; three `git worktree`s (one per
  part), the sub-agents one after the other; their diffs applied with `git apply --3way` (one
  conflict of neighbouring lines, both kept); scratch scripts over `scripts/exe.py` (the phases'
  reach, the draws' scan, the float census, the globals), not committed. `scripts/exe.py cited`
  caught two survey addresses without entries (0x4ACCA0, 0x691680): read and added. The town lane's
  inbox note (E09 T51: its services now increments 6 to 8) applied to decision 6 and archived.
- **Not read:** the 15 `inferred` functions (the simulated combat's spell values, the container
  code, `ai_army_offers_add`); the callees of the artifact effects beyond their entries; the quest
  texts' builder and the Quest Guard's completion; the pathfinder's mode-4 Fly, Water Walk and leg
  starts, `pathfinder_build_path`, the writers of its +0x14 and +0x15; 0x52CF10, the puzzle's pick.

## T1b — the survey's close, the ADR draft, the checkpoint (2026-11-24, increment 1)

- **The cut** (AI-Q30, the user at T1b's start): no cut, sub-agents used efficiently. Three
  general-purpose sub-agents, each with a narrow brief and a scratch file: the 61 quirk candidates
  classified from the survey's text (by the ledger's classes and BAT-QUIRK-25's precedent), the
  hidden reads' and the gaps' tables, the wiki's and VCMI's positions on the rule rows. The main
  session checked their tables (three classes changed, four gap rows corrected, a ninth rule row
  found: ADV-MOV-008 holds every player to the human's fog rule), then wrote the rest. The user's
  computer was unreachable for `Heroes3.exe` this session; nothing needed it (the blind re-reads
  stay with each slip's port task, Q30).
- **The survey §5**: §5.1 each phase's rules (the planned `CPU-` areas and the original's rules in
  each phase), §5.2 the 28 hidden reads (the fair variant's checklist), §5.3 where the nine rule
  rows went, the sources beside the reading (the wiki agrees on the build days, Easy's level 7,
  the Grail thresholds and the prices; VCMI on the patrol; no outside source for rows 1 to 3, 5, 8
  and 9) and two rules outside the code (the daily bonus, E10's; the explorer's 50 points, none
  yet), §5.4 the 40 gaps by epic (E08 increment 15: 4, E09: 15, E10: 8, E12: 4, E20: 1, the lanes'
  undecided rows: 6), §5.5 the second look (36 slips, 6 state, 5 exploit-fix, 13 none, 2 fixed).
- **The inboxes**: the battle lane (rows 1, 2, 8), the adventure lane (rows 3 to 9 with the town
  lane, the meeting of one team, the loader's two crashes, the explorer's points, what the port
  asks of its API), the town lane (rows 5 and 7, the trades' rounding, the allies' shares, where a
  town's creature upgrade is planned).
- **The ADR draft** (`docs/lanes/ai/adr-drafts/original-computer-player.md`, 14 points): a module
  of `h3-game` for now, `h3_game::computer::choose` one command a call, its battle side by the
  host's seat, always legal with stubs, the whole state read and nothing written, the memory the
  policy's (only the destinations carried between turns), the numbers once, every battle fought
  until the battle lane decides, the caller's stream, a signed `X87`, AI-QUIRK-1 for the slips,
  `CPU-` rule ids, a turn's budget, `POLICY_VERSION`, the fair variant on the player's world.
- **The design note** (`E31-design.md`): the traces T-A to T-C on compiled maps without setup draws
  (draw-free decisions equal; a destination equal unless noise-bound), experiments X1 (the reseed)
  and X2 (the records across games), the seam by lane, the memory's fields.
- **The checkpoint** (AI-Q31): the layer: a module of `h3-game` for now, refactored later (the
  draft's `policy` layer not taken now); the memory, the battles and the slips' flag as proposed;
  #26 a slip, #36 valued as luck in every profile; increments 2 to 7 confirmed. **Increment 1
  closed** (`E31-archive.md`).
- **Checks:** the scoped verify without original data (docs only).

## T2 — increment 2's refinement: the shared pull request, the measurement, the cut (2026-11-25, increment 2)

- **The shared pull request** #1163 (`shared/e31-module`, `cross-lane`; ADR-0030 points 1 and 9):
  `h3-game`'s `src/computer/`, `computer.toml` and `quirks-ai.md` the ai lane's (`lanes.toml`), its
  scoped verify with the adventure lane's crates (`verify_with`, as the town lane's: the module's
  tests are `h3-game`'s); `pub mod computer;` in `h3-game`'s `lib.rs` (a module of docs only, so that
  later ai pull requests stay in the lane's paths); `check-rules`' context `computer` (`CPU-`).
  **The ledger** `docs/rules/quirks-ai.md`: AI-QUIRK-1 reserved for the slips (written with the first
  slip built, increment 4), the three exploit-fix entries of the computer's own code (§5.5 #12 the
  Angelic Alliance's alignment of a creature without a faction, #27 the formation's 16-bit counts,
  #38 the split's division by zero: AI-QUIRK-2 to -4, each with its planned regression test); the
  loader's two (#54, #60) the adventure lane's reader's. **`SignedX87`** in `h3_kernel::arith`: sign
  and magnitude, each operation `X87`'s with IEEE 754's signs (a zero has none), `trunc` the
  `fistp qword` of `_ftol` (the integer indefinite `i64::MIN` from 2^63 on, as `h3_battle`'s
  `aim::ftol` reads it), `ftol` the callers' low 32 bits, the f32 patterns; audited by a property
  test against IEEE doubles in `num.rs` (products, sums, differences, quotients, the order, the
  truncations, the f32 stores and patterns, bit for bit; 20 000 cases passed locally) and unit tests
  for the zeros, the order, saturation and the indefinite; `arith.rs` at 100 % (llvm-cov).
  **ADR-0030** numbered (AI-Q32): the stale `h3_game_ai::` names now `h3_game::computer::`'s.
- **The measurement**: the sizes in `docs/exe/map.toml` of the functions each part ports (§4.3's
  phases), at about 2.7 bytes an instruction (T1a4's: 9.5 KB, about 3 500 instructions): the frame
  3.7 KB, the start's values 6.2, the simulated combat 14.3 (sides and melee 7.3, spells 7.0), the
  values 33.3 (the object value 11.6, the armies and towns 9.1, the artifacts 7.9, the spells and
  skills 4.7), the planning 7.7, the walk 5.8: about 71 KB, 26 000 instructions, 63 % of the 110 KB
  read; the x87 instructions mostly in the object value (331) and the simulated combat (271).
- **The cut** (AI-Q32): T2a to T2k in the epic file, by dependency (the frame, the simulated
  combat, the start's values, the values from the leaves up, the planning, the walk, trace T-A),
  about one session each.
- **Checks:** `h3-kernel`'s and `xtask`'s tests, clippy, `cargo check --workspace --all-targets`,
  `check-rules`, `check-docs`, `check-exe-map`, `check-arch`, `check-lane --cross-lane`; the scoped
  verify not run (the shared change is the kernel's arithmetic and docs; CI's full verify is the
  record).

## T2a — the frame (2026-11-25, increment 2)

- **The size, measured at the start** (`docs/exe/map.toml`): the nine functions of §1.1 to §1.4 and
  §1.6, 3 793 bytes, about 1 400 instructions, as the cut said; done alone (not paired with T2e).
- **Blind re-reads** (`Heroes3.exe` staged from the user's repo folder; nothing printed
  committed): `ai_player_turn` 0x525E80, `ai_next_hero` 0x526A90 (and the second pass's copy at
  0x526000), `ai_hero_turn` 0x5261F0; each agrees with the survey. Details the survey did not
  spell out, now in the catalog: the alone flag is set to 1 first and cleared by a second
  qualifying hero, so a garrison hero sent out plays without it; the skills' sum reads each
  byte signed (above 99 counted 99, not above 0 counted 0 for Attack and Defense and 1 for Spell
  Power and Knowledge); a strictly smaller sum replaces; the turn's flag is set again after the
  middle (0x525FD5); `town_visitor_to_garrison` 0x4B9FC0 does nothing when `army_absorb` refuses
  (the engine's refused `Enter`); the garrison step tests the town's owner against the hero's
  (+0x1 against +0x22).
- **Built** (`h3_game::computer`, the ai lane's module): `choose` (CPU-TRN-001; `None` while
  another player's level-up holds the game and for a battle, the battle seat's; an open decision
  by `default_answer`, AI-STUB-6), `Memory` (CPU-TRN-005: the turn's day, pass, flag, the hero a
  garrison sent out, the heroes done; encoding v1 `MEMORY_ENCODING_VERSION`, a strict decoder, the
  FNV-1a hash over the encoding), `POLICY_VERSION` 1; the turn's frame (CPU-TRN-002: the start, the
  first pass, the middle, the second pass, `EndTurn`; one command a call, the frame going on from
  its memory), the order of heroes (CPU-TRN-003), a hero's turn with the garrison step
  (CPU-TRN-004: `Enter`, `Swap`, and the garrison hero sent out by `Leave`, each only where the
  engine allows it). Stubs: AI-STUB-2 the search and the walk (every hero done at once), -3 the
  start's records and the middle, -4 the town visit, -5 the patrol (not in the game state), -6
  the answers.
- **Agent decisions (change freely):** the player's town list is the engine's `TownId` order (as
  `upper_bars` and the Thieves' Guild read it); the eight heroes' bound of `ai_next_hero` is its
  constant 8 beside the engine's `most_heroes`; the hero picked while already selected (its alone
  flag dropped) and the selection itself wait for T2i and T2j, since no hero is picked twice
  while each is done at once; the walk's "no points or done" test before the garrison step
  likewise (every hero reaching it is done).
- **Tests:** `h3-game`'s `computer/tests.rs` on the town tests' game and the plain game (the
  passes and a new day, the flag, an entry and a refused one, a swap, a garrison hero sent out
  and back, eight heroes on the map, the order with its tie, the signed bytes, a battle and a
  level-up's hold, the encoding and a property test of it): the module at 100 % lines and regions
  by `h3-game`'s own tests (the adventure and town lanes' scoped verifies cover `h3-game` without
  `h3-sim`'s). `h3_sim::computer::play` hosts every player as the computer (the battles by
  `h3_battle_ai::choose`, faithful): `h3-sim`'s tests play the five generated 36-tile maps
  (`e08-duel`, `e09-towns`, `-builds`, `-taverns`, `-guild`) at every difficulty (48 cases, up to
  a week; each command asked twice on copies, each turn within its bound, every command applied),
  and a swap back by the original packs' army values (the test core's creatures are worth 0).
  `examples/original_data.rs` (`--with-original-data`): every one of the 160 shipped maps imports
  and plays at every difficulty to day 7, five of them (Arrogance, Dead and Buried, Crimson and
  Clover, The Gauntlet, War of the Mighty) to day 28: 800 games, 28 350 turns, 68 805 commands
  (Enter 21 630, Leave 18 825), the longest turn 7 commands, about 30 s in debug.
- **The benchmark:** `h3 ai computer` and the lane-only workload `computer-turns-towns-36` (4
  games of 28 days on `e09-towns-36.h3m`, inside `h3_game::computer::choose`): 4 933 409 Ir,
  about 7 400 a command; 66 % the order's primary skills (`h3_heroes::primary_skills` of every
  awake hero at each pick), 10 % the frame, 9 % the towns' owners. Left as it is: a turn costs
  about 22 000 Ir, far inside the 50 ms budget; the later tasks' values will dominate.
- **Told:** the map-client lane the entry point (its inbox, `MCL-STUB-5`).

## T2b — the simulated combat's sides and melee (2026-11-25, increment 2)

- **The size, measured at the start** (`docs/exe/map.toml`): 7.4 KB, about 2 700 instructions, of
  which 1.4 KB the stack vector's container code (a `Vec` here), so about 2 200 of logic; the user
  asked whether to cut (sides, then melee; the sides with the loss steps, then the flow; no cut):
  **no cut** (AI-Q33), done in one session.
- **Blind re-reads** (`Heroes3.exe` staged from the user's repo folder; nothing printed
  committed): `sim_combat_start` 0x423EE0, `sim_side_build` 0x424120, `sim_side_walls` 0x424790,
  `sim_apply_losses` 0x424880, `sim_side_max_speed` 0x424960, `sim_spread_losses` 0x426170,
  `sim_side_wipe` 0x4262B0, `sim_side_take_damage` 0x426300, `sim_side_damage` 0x426390,
  `sim_side_hold_value` 0x426450, `sim_combat_finish` 0x4264D0, `sim_engage_now` 0x4267C0,
  `sim_combat_run` 0x426BC0, `sim_stack_take_value` 0x423EA0, and the three hero functions the
  build calls, read whole for the first time: `hero_archery` 0x4E4160, `hero_creature_speed_bonus`
  0x4E5AA0, `hero_creature_health_bonus` 0x4E5B80 (the map's entries now `read`). Each agrees with
  the survey but four details: (a) when only the attacker engages, its losses go first to its
  stacks in contact by the round (0x426CF8; the map's summary said all over round 0); (b) a hero's
  shooters' shot share is `hero_archery` / 5, and `hero_archery` is the Archery *bonus* (0, 0.10,
  0.25, 0.50 by level, a specialist's grown, the bows only with the skill), so 0 without the skill
  against 0.2 without a hero (the survey's "0.2 × Archery"); (c) the wall halving spares creature 35
  alone, the Arch Magi, not the Magi; (d) the finish's and the lookahead's stand-off losses are a
  loss in round 0 exactly as `sim_side_take_damage` takes it. (b) and (c) are §5.5's new state
  rows #63 and #64 (the quick battle's, as #7 and #8), told to the battle lane's inbox.
- **Built** (`h3_game::computer`'s private `sim.rs`; CPU-SIM-001 to -005): `HeroReads` (what a
  side reads of a hero), `side` (the start's army and walls, the build), `Side`'s `wipe`,
  `spread`, `take_damage`, `damage`, `hold`, `survivors` (`sim_apply_losses` without the quick
  battle's hero leaving the game), `finish`, `engages`, `run`; the round's casts call `cast`,
  which does nothing (**AI-STUB-7**, T2c). Values through `X87` / `SignedX87`, 32-bit integers
  wrapping as the code's `imul`, `add` and `sub` (the build's value and strength, the losses).
  No caller outside the tests until T2c's fight estimate: the module is
  `#[cfg_attr(not(test), expect(dead_code))]` until then. No `POLICY_VERSION` bump (no choice
  changes), the workload untouched.
- **Agent decisions (change freely):** the hero's reads come from the engine's battle hero
  (`derive_battle_hero` with no visit bonuses: Tactics, the Catapult's Ballistics level, the
  ranged rate as its single-precision value or its basis points' f32, the stack statistic
  effects), its available spells (`available_spells`, a spell with the battle part's `Earthquake`)
  and the town's `fort_level`, not the original's artifact and spell ids; the speed bonus counts
  the speed changes that select every creature (the original's function takes no creature, so no
  creature specialty), the hit-point bonus the changes that select the creature (a share rounded
  down); the creature's hit points, speed, flags and abilities the battle part's, its fight value
  the core's; the Arch Magi's spared shots read the battle part's `no_wall_penalty`, which the
  Magi have too (#64: no data names creature 35 alone); a creature of speed 0 is 4 rounds away
  (the original divides by 0), a unit value of 0 leaves a count of 0, a group of 0 takes a share of
  0 (the FPU's infinity truncated); the casting inputs of `sim_combat_start` (spell points, may
  cast, the tile's ground), the value per hit point (+0x30), the creature (+0x04) and
  `sim_stack_take_value` 0x423EA0 (mass damage only) wait for T2c, their only reader.
- **Tests** (`computer/sim/tests.rs`, 20): a content of the crate's test core with fight values,
  hit points, flags, the skills, artifacts, specialties and an Earthquake spell; the builds (no
  hero, a hero against an opponent, an opponent alone, the walls' table, walls and flyers and the
  shot shares, a specialist's single-precision rate, speed 0 and a 32-bit wrap, a hero that does
  not validate); the losses, the damage, the hold, the stand-off's four cases, the lookahead, the
  run's four engagement cases and its stand-off, the cast order; a property test (a run is
  repeatable and only takes). The expected numbers come from an IEEE double model of the reading,
  which also agreed with the port on 400 random runs (not committed). `sim.rs` at 100 % lines and
  regions by `h3-game`'s own tests.

## T2c — the casts, Necromancy and the fight estimate (2026-11-25, increment 2)

- **The size, measured at the start** (`docs/exe/map.toml`): 7.6 KB, about 2 800 instructions,
  4.9 KB of it never read before (the spell values 0x4249D0 to 0x425B10, 0x423C80, 0x423D00,
  0x423DE0); the user chose **no cut** (AI-Q34).
- **The reading** (`Heroes3.exe` staged from the user's repo folder; nothing printed committed):
  `sim_cast_spell` 0x425BD0 whole, its thirteen leaves read for the first time, `sim_combat_start`'s
  casting inputs, `sim_necromancy` 0x426DF0, `ai_army_fight_sim` 0x427330, `ai_experience_value`
  0x527710, `ai_difficulty_blend` 0x428710, `hero_reset`'s aggressiveness draw, the first tests and
  the end of `creature_spell_factor` 0x44A4D0 (the Dragon Father's levels first, Dispel's
  shortcut, 1 for a positive spell). Two inferred names were swapped: 0x4249D0 is the chain's next
  stack, 0x424BF0 the area's value (renamed in the map, with 0x424FB0 and 0x425100). Two new state
  candidates (§5.5): #65, an other spell sets a stack's value per hit point to the new value over
  old value × old value per hit point; #66, an other spell and a raise leave the side's strength.
  Both the quick battle's, ported as read.
- **The shape** (AI-Q34): the casts call four of the battle's helpers on creature types and heroes,
  which `h3-battle` had only crate-private on a battle's stacks; the user: the battle should provide
  the battle estimate entirely. **Built** (`cross-lane`): `h3_battle::estimate` (public: `side`,
  `run`, `Side::strength`, `Side::survivors`, `EstimateHero`, `Ground`), T2b's `sim.rs` moved there
  on contract inputs (each side's `BattleHero` and available spells, its `ArmyStack`s, the factor,
  the town's `FortLevel`, the tile's battlefield and anti-magic garrison) and its casts on the
  battle's own rules: `magic::creature_factor` (0x44A4D0's creature part: the Dragon Father first,
  then `affects`, 1 for a positive spell, Dispel or a negation, else the creature share less the
  controller's resistance; no aura), `magic::creature_damage` (the target's part of
  `spell_damage`, extracted), `untargeted_bonus`, `specialist_bonus`, `mastery_at` (now on a hero's
  `HeroMagic`), `mana::armies_cost_change` (extracted from `cost_change`), `Guard::of_heroes`.
  `h3_game::computer::fight` (private, `expect(dead_code)` until T2d, T2h, T2i call it): the
  estimate, Necromancy into the survivors, the experience's value, the blend.
- **Agent decisions (change freely):** spells are told by their data as the original tells them by
  their numbers (the AI kinds; a chain, an area's radius for Inferno's 2; `Raise` for 38 and 39; the
  counter-casting set by `Dispel` and the Protections', Anti-Magic's and Magic Mirror's timed
  changes; Dispel as `Dispel` without `Heal`); Familiars by `mana_channel`'s percent; the
  aggressiveness 1.0 (AI-STUB-8); the conditions' bonuses none (AI-STUB-9, E10 increment 2); the
  map's version SoD's (the Cursed Ground test never refuses); a division by 0 of the original (an
  aimed stack without value, a raise's unit value 0) gives 0, and the experience's gap of 0 the
  integer indefinite; the unreachable "no target" cases are `for_each` over an `Option` (no
  uncovered region).
- **Tests**: `h3-battle`'s `estimate/tests.rs` (37, on the crate's test ruleset with Fight Values
  and AI kinds patched in): T2b's hand-built cases unchanged in their numbers, the builds redone on
  `BattleHero`s, the casts (the order, the frame's refusals, the cost changes and Familiars, the
  loss chain with resistance, immunity, vulnerability, reduction, the Orb, the Dragon Father,
  Sorcery; the chain, the area, the mass damage's carried totals, the blessing's slip, the curse,
  the ward, Dispel at each mastery, Cure, the raise, the kinds), the property test; `h3-game`'s
  `fight/tests.rs` (7: the pieces and three whole estimates on the crate's games with enemies the
  combat beats before round 1). The numbers from an IEEE double model of the reading (Python, not
  committed); four of my expectations were wrong and the port right (a loss capped at the value,
  Cure's counter-casting test, the flyer's 8 hit points, 0.7 × 60 = 42 in doubles). `estimate.rs`
  and `fight.rs` at 100 % lines and regions by their crates' own tests; `h3-battle`, `h3-game`,
  `h3-sim` all pass.

## T2d — the start's values (2026-11-26, increment 2)

- **The size, measured at the start** (`docs/exe/map.toml`): the 14 functions, 6 157 bytes, about
  2 300 instructions; the terrain shares already the engine's (`h3_heroes::TerrainShares`, E08
  T25) and the difficulty's weights `fight::blend` (T2c), so about 1 950 left. The user chose **no
  cut**, three values moved to the tasks that build what they need, four small `cross-lane`
  changes, and the vision circle to T2h (AI-Q35).
- **Blind re-reads** (`Heroes3.exe` staged from the user's repo folder; nothing printed
  committed): `ai_turn_start` 0x4297C0, `ai_turn_start_values` 0x527960, `ai_resource_weights`
  0x428740 (whole, the creature list read for the first time), `ai_recruit_reserve` 0x429AD0
  (whole), `ai_eyes_value` 0x429910, `ai_enemy_reach_fold` 0x4280E0, `ai_reach_fold_towns`
  0x4282B0, `ai_town_capture_sim` 0x428410, `ai_town_threat_clear` 0x428260,
  `ai_town_threat_count` 0x428570, the two vtables, `ai_hero_effectiveness`'s +0x109, and the
  library sort's pieces (0x434CE0, 0x435240, 0x435290, 0x435160, 0x435310). Each agrees with the
  survey but five details, now in the survey and the catalog: (a) the reserve values a stack at
  the AI value's low 16 bits times the count in 16 bits, and a town with one stack counts none
  (the bound n − 2 compared unsigned): §5.5's new slips #67 and #68; (b) a demand above means of
  1 or less is the demand itself before the cap (0x428CF1); (c) the Marketplaces are the towns
  whose buildable mask holds building 14, built or not; (d) the demand's building part is per
  resource the largest cost, each resource on its own; (e) the defence fold runs first and sets
  the same threats, each enemy hero's cleared before its search, so with its purchases stubbed
  the threat fold's result is the start's.
- **Built** (`h3_game::computer`'s `start.rs`, CPU-STA-001 to -006): the start's `Record` in the
  memory (encoding **v2**: the Eye value, the reserve, the means, the demand, the rate's row, the
  mean weight, the heroes' experience weights, the threats, the towns' counts), computed at each
  turn start (`turn::start`); the threat fold (`threats`) with the town's capture estimate
  (`fight::captures` on `h3_battle::estimate`, its first caller outside `fight`); the reserve
  (`reserve`, `two_dearest`); the weights (`weights`, `three_dearest`, `factor`,
  `Record::weights`); the experience weight (`experience_weight`, the fight's
  `experience_value`). The library's `std::sort` (`order.rs`). **Cross-lane** (AI-Q35):
  `AdventureState::reach_with` (a search with the caller's points; `reach` calls it with the
  hero's) and `day_maximum_of` (`h3-adventure`); `GameState::day_income` public (the adventure
  lane's `h3-game`); `GameState::growth_inputs` (the town lane's `h3-game`), and three of the town
  tests' builders `pub(crate)` for the start's tests. Stubs: AI-STUB-3 now the middle alone;
  AI-STUB-10 the Eye value (no Eye of the Magi; the vision circle with T2h), -11 the fold's search
  the engine's reach, -12 the defence recruits (increment 3), -13 the spell gains (T2e), -14 the
  artifacts' mean (T2f), -15 the Grail target (T2i). No `POLICY_VERSION` bump: no choice reads
  the records yet.
- **Agent decisions (change freely):** the weights are kept as their inputs (the means, the
  demand, the rate's row) and computed on read exactly as the start computes them (no f64 bit
  pattern in `X87`'s surface); the threats and the towns' counts kept only where not 0, by id; the
  creature list in the core's order and length, not the original's 145 by number (the packs'
  order differs after the base creatures), so only equal values at the third place could end
  otherwise; the Marketplace found by the town record's key `marketplace` (no role yet: the town
  lane's inbox); the buildable-now set is the hall's test of the town without its mark,
  intersected with the hall's possible set (the held Capitol); an enemy's day's maximum is
  recomputed now (`day_maximum_of`), as `hero_max_movement` reads it, not the one stored at its
  turn start; the capture estimate only for towns reached, as the original, the enemy's battle
  reads derived once per hero; the library sort copied from `h3_battle_ai`'s (`h3-game` does not
  depend on it).
- **Tests:** `computer/start/tests.rs` (8, on the town tests' game with values, wood, a
  Marketplace and the market's rates: both players' whole records, the fold's cases (two towns,
  a garrison that holds, a hero walled in by rock, a player without towns), the reserve's 16-bit
  values and its one-stack slip, the three dearest of four creatures, the buildings read now
  with the day's mark and with a held Capitol, the growth from the week's fifth day, the factor's
  nine cases and the four comparisons); `computer/order/tests.rs` (4: the median's six paths,
  the insertion sort's ties, the quicksort's ties on 35 entries both ways, a property test);
  `computer/tests.rs`' memory tests for v2 (every field, each value out of range, the lists'
  order, the property test); `h3-adventure`'s reach test for `reach_with` and `day_maximum_of`.
  The doubles from an IEEE double model of the reading, the sort's orders from an independent
  model of the library's (Python, not committed), compared exactly (doubles scaled by 2^40); all
  agreed with the port at once. `start.rs`, `order.rs`, `memory.rs`, `fight.rs` and `turn.rs` at
  100 % lines and regions by `h3-game`'s own tests.
- **The benchmark** (`computer-turns-towns-36`): 4 933 409 Ir → 207 M Ir, re-baselined: 87 % the
  threat fold's searches (one reach per enemy hero with its maximum plus 800, the original's
  pathfinder run), about 925 000 Ir a turn (about 0.3 ms), inside the 50 ms budget. Tried and
  dropped: the capture estimate before the search (253 M: on that map most enemy heroes would
  take some town, so the searches stay and the estimates add up). The battle reads derived once
  per hero change nothing measurable there (207.0 M before, 207.2 M after: an enemy reaches one
  town there), kept for the maps where it reaches several.
- **Found:** `h3-adventure`'s property test `reach::tests::the_query_and_move_hero_agree` fails on
  `master` with a new case (the query and `MoveHero` disagree on the points left, 4 against 6):
  its `cc` line in the adventure lane's inbox, not committed (it would fail every lane's CI).

## T2e — the spells' and skills' values (2026-11-26, increment 2)

- **The size, measured at the start** (`docs/exe/map.toml`): the 11 functions, 4 958 bytes, about
  1 840 instructions (the caster record, the five kinds, the best value and the spell's gain 2.5
  KB, the hero's values 0.5 KB, the skill value, the school and the University 2.0 KB), of which
  E23 had ported the caster record and the damage kind for the battle's odds (`aim.rs`,
  BAT-AI-008) and E08 T25 the skill value's setup path (`h3_heroes::choice`, HER-LVL-006). The
  user chose **no cut** (offered: the spells first, the skills next session), the spell values in
  `h3_battle::estimate` with the odds calling them, and the skill value's flag-set path in
  `h3_heroes::choice` (both `cross-lane`; AI-Q36). The PC left before the original data were
  staged; the user attached `Heroes3.exe` to the chat for the readings.
- **Readings** (the main session, the survey's lines known: not blind): `ai_caster_record`
  0x526D40, `ai_damage_spell_value` 0x5270E0, `ai_mass_damage_spell_value` 0x5271C0,
  `ai_effect_spell_value` 0x527290, `ai_spell_value` 0x5273D0, `ai_best_spell_value` 0x5275B0,
  `ai_hero_spell_gain` 0x527640, `ai_hero_effectiveness` 0x527760, `ai_secondary_skill_value`
  0x524690 (every flag-set case), `ai_takes_new_secondary_skill` 0x524DD0, `ai_school_value`
  0x524D20, `ai_university_value` 0x525BF0, `hero_spell_cost` 0x4E5240 and the entry of
  `creature_spell_factor` 0x44A4D0. Each agrees with the survey and §7.39 but four details, now
  in the catalog, the address map and the survey: (a) the record's army list values a stack at
  its count's low 16 bits as a signed word (§5.5's new slip #69); (b) with no enemy army the
  cost counts neither the Pegasi nor the own Mages (§7.39 said the Mages counted; `aim.rs` had
  it right); (c) the Knowledge point's gain is 30 more mana's over 3 (the survey's "a tenth");
  (d) the flag-set path's schools take their values whatever Wisdom. #17 (a new school at
  Expert) and #20 (the University's gold once, four skills) confirmed as read.
- **Built.** `h3_battle::estimate`'s `caster.rs` (**cross-lane**; CPU-SPL-001 to -006):
  `CasterRecord` (public: its strength, power, rounds and mana fields, `new`, `value`, `best`),
  the kinds on the battle's own helpers (`mastery_at`, `mana::cost`, `untargeted_bonus`,
  `creature_factor`, `takes_stack`), the curve moved from `aim.rs`; `aim.rs`'s `hero_value` now
  builds a borrowed `SpellCaster` and asks `best_spell_value` with the damage kind (its private
  `spell_value` and `Caster` gone; the battle's tests unchanged). `h3_heroes::choice`
  (**cross-lane**; CPU-SKL-001): one switch `original` for both paths in the original's 32-bit
  integers (the setup path's values unchanged), `ComputerReads`, `counted_worth`,
  `computer_skill_value`, `computer_takes_new_skill` (signed ranking). `h3_game::computer`'s
  `spells.rs` (the hero's record on its tile, `gain` CPU-SPL-007, `gains` CPU-STA-007) and
  `skills.rs` (`school_value` CPU-SKL-002 on a changed copy of the details, `reads`,
  `university_value` CPU-SKL-003); the start's `Record` keeps per hero `HeroValues` (the
  experience weight and the five spell gains; the memory's encoding **v3**); AI-STUB-13 lifted.
  `fight::battle_hero` now sets the hero's maximum spell points (`derive_battle_hero` leaves 0;
  the simulated combat never read it). No `POLICY_VERSION` bump: no choice reads them yet; the
  gain and the skills' values `expect(dead_code)` until T2f to T2h.
- **Agent decisions (change freely):** the record holds its numbers as public fields, the
  callers change a clone (the original raises and lowers them in place); the record's mana is the
  battle hero's maximum (`h3_heroes::max_mana`, the same getter as the original's
  `trunc(f32(10 K) × i)`); the schools' values are computed only where the switch reaches them
  (below Expert, known or a free slot), 0 elsewhere (nothing reads them); the Ballista is a worn
  war machine whose creature shoots, the Tent one with `first_aid` (the engine has no artifact
  numbers); the gold weight and the University's gold at the market's cost resource (the base
  pack's gold); the unreachable Titan's Lightning Bolt case of 0x5270E0 not written.
- **Tests:** `h3-battle`'s `estimate/caster/tests.rs` (9: the record's numbers, its list's order
  and word counts, no power without a book or with `NoCasting`, the cost and the casts, the kinds
  none and several, the free spell, the field's mastery, the damage curve both ways and without
  strength, every stack's spared and hurt parts, the effect's three branches and the positive
  spell's immunes, a value below 0, the raise's two caps, the best value's kinds and level limit;
  the estimate tests' ruleset now with AI values and a free `quake`); `h3-heroes`'
  `tests/choice.rs` (3: every flag-set value, the counted army with its wrap, the signed
  ranking); `h3-game`'s `computer/spells/tests.rs` (4, on the crate's core with the original's 28
  skill places, Air's masteries, the gust of one AI kind and the market: the record and the five
  gains, the gain's four cases, the school's next level and a new one at Expert, the reads, the
  Estates and school values through them, the University's sum, gold test and eight skills, the
  worn machines); the start's and the memory's tests for v3. Numbers from an IEEE double model
  (Python, not committed); all agreed with the port once the record had its maximum mana. `h3-battle`
  and `h3-battle-ai` pass whole (724 and 149 tests).
- **The benchmark** (`computer-turns-towns-36`): 207 152 091 → 228 988 508 Ir (+10.5 %),
  re-baselined: a caster record per hero at each turn start (`derive_battle_hero`, 7.5 % of the
  count) and seven best values on it; about 1 020 000 Ir a turn.
- **Found:** `master` did not build its tests since #1190 merged after #1189 (T2d's start test
  without T56's `war_machine` field): fixed in #1195, which this branch sits on.
