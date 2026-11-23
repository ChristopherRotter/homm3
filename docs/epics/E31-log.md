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
