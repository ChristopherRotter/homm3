# E23 — log

The record of E23's tasks: what each read, built and decided, beyond the one line in the epic file.

## T1a1 — the entries and the map (2026-10-20, increment 1)

- **The cut** (the user, 2026-10-20): T1a as queued was too large for one small increment (the
  sweep: about 45 KB of unread code beyond the dispatch, the hero's spell choice entering from
  the battle window's controls rather than the stack's turn). The user chose "the inventory and
  the dispatch" over "and the shooters" and "the whole neutral side".
- **Read** (`docs/formats/installation.md` §7.34): the two entries per activation (the hero's
  cast 0x422DA0 from 0x4782D0, then the stack's action 0x477EE0 from the battle loop); the
  retreat question and the surrender's terms in 0x477EE0 (the computer always retreats; the price
  as the player's button computes it); the stack's turn 0x4221F0; a shooter's placement in the
  tactics phase 0x422060 and the tactics zone 0x4763F0; a walker's turn 0x421F80 (Wait only from
  Hard); the order of action 0x41F140 without its sort; the predictions the spell choice makes
  through the stack routines (0x422880, 0x43AA60); the dispatcher's action codes.
- **Mapped:** a sweep of the calls from each part's roots (a scratch script on `scripts/exe.py`'s
  sweep, not committed) gave §7.34's table: parts S, O, W, P, C, H and A with sizes, counts and
  their largest functions; 29 map entries (new functions and the globals 0x695030, 0x698798) and
  four updated (0x41E570, 0x4221F0, 0x477EE0, 0x4782D0).
- **Found on the way:** `scripts/exe.py switch` labels the dispatcher's cases one lower (it misses
  the `dec` before the range check; a shared fix); the surrender's price leaves out Basic
  Resurrection's temporary creatures, which BAT-FLE-002 counts (the battle queue's inbox).
- **Readings at risk** for T1b: no surrender by the computer; no Wait below Hard; the tactics
  placement; the hero's cast considered at every activation.
- **Proposed** (the epic file): T1a2 to T1a6 from the table, T1b after T1a5; the user decides.

## T1a2 — the shooters' rest and the walker's choice (2026-10-20, increment 1)

- **The checkpoint** (the user, 2026-10-20): T1a1's cut confirmed, T1a2 as queued (part S and
  0x421680) over "part S only" and "0x421680 only"; T1b after T1a5, T1a6 just before T5.
- **Read** (`docs/formats/installation.md` §7.35): part S in full: the area shooter's value
  0x41EDA0 (the own side's stacks subtracted, the undead spared by the Liches, no fire-immunity
  test; a negative best is no candidate), the area's stacks 0x5A46F0 and hexes 0x5A4170 (the
  side columns never part of an area), the Cyclops' wall shot 0x41EEA0 (the attacker only; the
  locked-out share of its strength against 3.0 × the best shot; the weakest wall drawn); the
  context's remaining fields (the balance class +0x1C). The walker's choice 0x421680 in full:
  the hex scores (Fire Walls; the moat under SoD rules only; danger on Normal and above; the
  hero's spells on Hard and above), the plan per enemy, the value and its draw of 75 to 100 %,
  the best's keys, the actions (the creature cast first, move and attack, a walk, a besieger's
  walk to its row's wall, the fallbacks). Seven helpers read in full, four at their heads or
  ends.
- **Found:** the defender's walls test 0x4213F0 uses a part number as a target index (0x42142F),
  so it tests the wrong hexes (a destroyed mid-upper or lower wall is never a breach); the best's
  comparison divides only the kept best's value by its turns. Both are the computer's own
  behaviour for E23 to model (T1b's survey decides how); no rule of the engine changes.
- **Map:** 28 entries (21 new, 7 updated; `ai_walker_target_scan` renamed `ai_danger_scores`).

## T1a3a — the hex scores (2026-10-20, increment 1)

- **The cut** (the user, 2026-10-20): T1a3 had about 10.7 KB left (a sweep of the calls from its
  roots); the user chose "the hex scores only" over the enemies' reach alone, the scores with the
  plan, and the plan with the order's sort first. T1a3b (the plan's hexes and the order's sort,
  about 3.1 KB) and T1a3c (the fallbacks and the ends, about 4 KB) follow.
- **Read** (`docs/formats/installation.md` §7.36), all in full: the enemies' reach 0x420260 (a
  shooter only marks the mask; a walker's map with its speed + 1 as range and turn speed; an
  enemy that can strike any of the side's stacks this turn adds nothing to the scores, else its
  strike value on the stack at every hex it reaches), the multi-headed branch 0x41FD60 with
  0x420060 and the pattern 0x448AB0 (Hard and above for a computer side only), the crowding and
  breath scores 0x41FB60, and the area threat 0x41F920 (the area attackers in the mask and the
  enemy hero's best of Frost Ring, Fireball, Inferno and Meteor Shower).
- **Corrected:** §7.35 called 0x41F920 "the enemy hero's spell threat"; it is the area threat
  (§7.35's step 1 and fallback reworded). Three map names follow the reading:
  `ai_spell_damage_threat` → `ai_area_threat`, `ai_spell_threat_scores` →
  `ai_crowding_scores`, `ai_attack_position_value` → `ai_multi_headed_danger`.
- **Found:** the multi-headed branch never tests its strike value's sign, so with a value below 0
  the hexes around gain; it alone leaves a human side out. The scores read the enemy hero's
  available spells and the stops of the enemy's side (hidden from a player).
- **Map:** 8 entries (4 new: 0x420060, 0x448AB0 and the tables 0x660878, 0x660898; 4 updated).

## T1a3b — the strike hexes and the order's sort (2026-10-20, increment 1)

- **The seam** (the user, 2026-10-20): T1a3b as queued (both parts, about 3.1 KB), over the
  strike hexes alone (recommended) or the sort alone.
- **Read** (`docs/formats/installation.md` §7.37), all in full: the hexes around a hex 0x436300
  (turns with a stop's extra turn; the value from the shooters blocked 0x436180, the multi-headed
  and breath strikes 0x436620 and 0x436760 with 0x445840, less the enemy's retaliation by the same
  two, and the hex scores; the best by turns, value, cost, order) and the order's sort 0x423340
  with 0x423630, 0x4235C0, 0x4237C0, 0x423820: the shape of Visual C++ 5/6's `std::sort`, the
  predicate the higher key, then the lower record index.
- **Found:** at an equal value the Cavaliers and Champions on Normal and above (or for a human
  side) take the hex with the longer path, every other stack the shorter; a shooter the walker
  already blocks counts 1 (the worth's own blocked test leaves no blocker out); the breath line
  uses the desperate mode and counts the walker's own hexes as a friend's, the multi-headed
  strike neither; the retaliation is tested by Stone Gaze alone. The sort orders two stacks of
  the two sides with an equal key and record index only up to 16 stacks (as listed); above, the
  partitions decide, and 0x41F140's alternation does not cancel it. No rule of the engine changes.
- **Map:** 12 entries (5 new: 0x4235C0, 0x4237C0, 0x423820, 0x4429F0, 0x445840; 7 updated;
  the three helpers named `ai_blocked_shooters_value`, `ai_multi_headed_strike_value`,
  `ai_breath_strike_value`).

## T1a3c — the fallbacks and the ends (2026-10-20, increment 1)

- **The seam** (the user, 2026-10-20): all of T1a3c in one session (about 3.4 KB), over the
  fallbacks and the walk first (recommended) or the odds and the retaliation first. The hero's
  value inside the odds (0x526D40, 0x5275B0 and ten helpers, about 4 KB of the adventure AI)
  became its own task, T1a3d, before T1a4 (recommended; over keeping it in T1a6 or folding it
  into T1a5): the neutral side has no hero but its odds weigh the player's.
- **Read** (`docs/formats/installation.md` §7.38), all in full but for the hero's value: the walk
  0x41F580, the guard 0x420760 with 0x4205D0, 0x4B3F20, 0x4466C0, 0x4674C0, the step to safety
  0x4208F0, the destination map 0x4B2FF0, the last retaliation's gain 0x41F3B0 with 0x435980, and
  the odds 0x420A80.
- **Corrected:** §7.35 read 0x420A80 as the sides' whole strength; it compares their strength at
  range (the stacks that can shoot now, the hero's value, the defender's turrets as Archers), the
  whole strength feeding only the hero's value. `ai_walker_odds` renamed `ai_stronger_at_range`;
  §7.35 and 0x4213F0's entry reworded.
- **Found:** with the scores and its flag (Hard and above or a human side, the target more than a
  turn away) the walk takes only hexes that score no lower than the last and waits when it falls
  short; without the flag it first reaches the hex from which the rest takes whole turns. The
  walk ends after a stop it enters, so a berserk stack's walk ends on a pit it sees when it gets that far (§7.14
  updated; the inbox for BAT-MAG-025). The destination map is Teleport's, a two-hex enemy's other
  hex left unmarked. The desperate kills in 0x41F3B0 use the top creature's remaining hit points
  where its lost ones belong. No rule of the engine changes here.
- **Map:** 14 entries (5 new: 0x4205D0, 0x4B3F20, 0x435980, 0x4466C0, 0x4674C0; 9 updated).

## T1a3d — the hero's value in the odds (2026-10-20, increment 1)

- **The seam** (the user, 2026-10-20): all of T1a3d in one session (about 4.4 KB, of it about
  1.6 KB the library's vector and sort), over the record and the best spell's loop first
  (recommended) or a spell's value first.
- **Read** (`docs/formats/installation.md` §7.39), all in full: the caster record 0x526D40 with
  the vector's insert 0x4347A0 and the sort's parts 0x434CE0, 0x435160, 0x435240, 0x435290,
  0x52BD50 (Visual C++ 5/6's, recognised by shape); the best spell 0x5275B0; a spell's value
  0x5273D0 with the damage value 0x5270E0 and its curve 0x640398, the other kinds 0x5271C0 and
  0x527290; the hero's tile's field 0x4E4FA0 (H3API's `GetSpecialSpellTerrain` confirmed) with
  0x4FCF40.
- **Found:** in the battle only R+0, the power R+8 and the mana R+0x10 of the record count
  (0x420A80 overwrites R+4 with the side's whole strength `S`; the list and R+0xC serve only the
  other battle spells' value). The odds count the damage spells with the AI bit `0x8000` (Magic
  Arrow, Ice Bolt, Lightning Bolt, Implosion, Frost Ring, Fireball, Meteor Shower; Titan's
  Lightning Bolt 1), the best one worth `min(10 d φ_n', S × Y(10 d / S))`: `d` the spell's AI
  value × (power + mastery) with the Orbs and Sorcery, `Y` a concave curve that reaches 4, so at
  most four times `S`. The mana is the hero's maximum (HER-MAN-001's formula), not the points
  left; the mastery takes the hero's own tile's field, the cost none; only the hero's own
  Recanter's Cloak counts, Cursed Ground not. With `S` = 0 the value is 0 (a NaN product's
  0x80000000 under the floor of 0). No rule of the engine changes.
- **Map:** 16 entries (13 new; 0x526D40 and 0x5275B0 renamed `ai_caster_record` and
  `ai_best_spell_value` and read; 0x420A80 updated). The adventure lane's inbox has what of its AI
  is read; the seven other callers of 0x526D40 and their masks are not.
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map
  made before the edits: the record's power and mana (the maximum, not the points left), the
  best spell's tests (the range, the own hero's Cloak, the floor of 0) and the damage value's
  formula (the base, `X`, the row at a threshold, the smaller product). It agreed on all three
  and read the same table.

## T1a4a — the pathfinder's search (2026-10-20, increment 1)

- **The seam** (the user, 2026-10-20): T1a4 measured about 4.9 KB with its helpers (about 2 KB of
  it the library's vectors); cut in two, the search first (recommended) over the head and the
  path's trace first or all at once. The trace (about 0.2 KB) moved into the search: the tie
  order needs it. T1a4b keeps the head (about 1 KB).
- **Read** (`docs/formats/installation.md` §7.40), all in full: 0x4B3400 from its setup to its
  end, the push 0x4B3BB0, the cell 0x4B3B90, the opposite direction 0x523EC0; the library's
  vectors (0x4B3F70, 0x4B4270, 0x434620, 0x434BF0, 0x434C30, 0x54D120, 0x5093C0, the empty
  0x404140, which H3API calls `H3String::Assign`) by shape. The object is H3API's
  `H3Pathfinder` (+0x24 cells, +0x38 the queue, +0x48 the path, +0x68 the stops; `structs.toml`).
- **Found:** a uniform-cost search: the queue sorted by cost, a new entry before the equal ones,
  taken from the lowest end, so first in, first out at a tie; each taken hex's neighbours from
  top right clockwise; a hex keeps the first hex that reached it at its lowest cost (only a
  strictly cheaper path replaces). A stop costs the rest of the turn; a blocked neighbour gets the
  attack marks; flyers and Devils pass over blocked hexes up to their speed. The search ends at
  the target reached or marked and traces the path back. **BAT-MOV-003's tie order differs**
  (from the end, right first): simulated on batch 3c's fields, the reading gives M2's walk along
  row 5 and G4's through 94, as observed, the engine's not; batch 2c's F2 agrees with both. The
  queue holds at most 500 entries. S-11 is not in the search.
- **Map:** 14 entries (13 new; 0x4B3400 read, `pathfinder_push` and the rest named) and two
  struct records (`pathfinder`, `pathfinder_cell`) with the hex record's +0x4A. The battle queue's
  inbox has BAT-MOV-003's alignment for the user's call.
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map
  made before the edits: the push's tests and the queue's order at a tie (first in, first out),
  the neighbours' order and when a cell is replaced (only by a strictly lower cost), the trace and
  the result for a walk. It agreed on all three; its one addition (a target at the start gives
  false) is in §7.40.

## T1a4b — the pathfinder's head, its callers and S-11 (2026-10-20, increment 1)

- **The decision** (BAT-Q53, the user, after #307): "pathfinding must work exactly as in the
  original". The cut (the user): T1a4b first, then the walks and the berserk turn aligned together
  (T1c, queued), over the walks first (recommended) or both at once.
- **Read** (`docs/formats/installation.md` §7.41), all in full: 0x4B3400's head; the automatic
  test 0x474BF0 (renamed `stack_automatic`: the war machines' cases with the tactics phase, auto
  combat and the four combat settings, now named, then the player's test); the nodes' allocation
  0x4B1460; the walk's call `walker_path` 0x5239D0 and the cursor's 0x523A70 (renamed
  `cursor_path`): range the speed, so a walk's search stays within one turn.
- **Found: S-11 placed.** In an attack the search ends only on a blocked step at the target hex
  itself, and the berserk turn gives the target's position: a two-hex target reached only by its
  other hex is a candidate (its marks) the walk never reaches, so the berserk stack goes round or,
  with no hex next to the position within 127, does nothing (action 12). §7.40's "S-11 is not in
  the search" (T1a4a, #307) is corrected here. The harness plays without HD, which fixes it: a
  batch row for T1b.
- **Map:** 10 entries (3 new: the combat settings 0x69879C, 0x6987A0, 0x6987A4; 7 updated).
  BAT-Q53 answered in OPEN-QUESTIONS; the epic's T1c and the queue's task carry the alignment, the
  inbox's two items (BAT-MOV-003, BAT-MAG-025) folded into it.
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map
  made before the edits: 0x474BF0's tests for the Ballista in order and the Ammo Cart's case (no
  tactics or skill test), the range and speed a walk passes (its speed, −1), and which hex ends
  an attack with what the berserk turn does on an empty path. It agreed on all three: a one-hex
  berserk walker that reaches only hexes next to a two-hex target's other hex never ends the
  search and does nothing.

## T1c — pathfinding as the original's (2026-10-20, increment 1)

- **The decisions** (BAT-Q54, the user at the start): T1c whole, over the walks first
  (recommended); S-11 a fix-optional quirk, BAT-QUIRK-24 (`quirk.battle.berserk_walks_to_position_hex`,
  registry position 26, on `shared/e23-t1c-quirk`, which merges first): faithful as the original,
  strict-fixed as HD. Its consequences agreed: the faithful ruleset hashes move, so the adventure
  lane's faithful replays' headers too (`cross-lane`).
- **Read for it** (to settle the berserk turn's end, `installation.md` §7.41 and its catalog
  source): 0x4222C0 compares the cost in the target's *position* cell, which the marks write
  (0x4B3834–0x4B3850; a two-hex target's both hexes on any hit), with the speed, and attacks from
  the path's second cell (the hex to attack from); 0x4B3A1D writes the blocked step's cell N into
  the path's head; the move handler 0x445950 refuses a hex the stack does not fit on and the walk
  0x441FA0 one out of its move, after which 0x478900 skips the strike. 0x445490 keeps the nearest
  candidates and draws one (0x445688: the best cost follows each append).
- **Built** (`crates/h3-battle/src/pathfinder.rs`): the search as §7.40 and §7.41 read it, for
  three questions: a walk's path (`Goal::Reach`, range the speed; `movement::path`, BAT-MOV-003),
  the berserk candidates' map (`Goal::Map`, range 127) and the berserk walk (`Goal::Attack`, the
  target's position hex, or with the quirk off any of its hexes; the edge column's war machine by
  its other hex, 0x4222E0). Cells share cost between reaching and marking as the original's do (a
  flyer passing over a marked hex overwrites its mark). `control.rs` rewritten on it: the attack
  from the search's end when the target's cell cost is within the speed (no move: the move fails,
  `TurnPassed`), else the walk along the path, at most the speed in steps, ending after a stop, a
  flyer landing where it did not pass over. Not modelled, as no engine caller meets it: the cap of
  500 queued entries (a position is queued at most twice), the range-within-a-move refusal to mark
  from a stop (0x4B378D: only the walks' searches, which read no marks), a side's own stacks as no
  targets, the tactics phase's range. `aim.rs`'s threat records keep their own cost search (the
  inbox).
- **Equal for the walks** (the argument the module states): within one move a stop costs the whole
  move and is a leaf, so the walk's search reaches the same positions as `movement::reachable`;
  the paths differ at ties only. A walk's search skips positions reached at no more than one step
  from the taken one, tests a step's stop only where the stack fits, and reads the neighbours once
  per taken position (exact: no marks in it).
- **Tests:** batch 3c's M2 and G4 (`sieges.rs`), the tie's directions (`obstacles.rs`), S-11 in both
  settings with the three outcomes (round, nothing, the failed move), the war machine by its
  front, a two-hex walker's front, a flyer passing rocks and failing to land on one, a walk beyond
  its speed stopping in its side's pit; two siege tests moved to the new order (the outer bridge's
  walk to (8, 4); the weak walker's moat test against a slow on (8, 1)). Coverage 100 % on the three
  files.
- **Replays and benchmarks:** `full-s3-a1367` and `full-s3-a2378` re-recorded (another game from
  command 151 and 142); the other battle replays and the seven faithful adventure replays only
  their headers' ruleset hash (`duel-s1` and `field-s5` are not re-run: since 17a0489b their spec
  draws other obstacles; `fixtures/README.md`). `random-rollouts-full` 79.3 k Ir per step against
  master's 78.7 k (+0.8 %), `random-rollouts-siege` +1.4 % per step (every walk at a town traces
  its path); both re-baselined, profiles compared with master.
- **Checks:** a fresh agent re-read three points blind (0x4222C0's cost cell, hex to attack from
  and empty path; the marks' cell writes and the flyer's pass after them; the failed move in
  0x478900 with 0x445950 and 0x441FA0) and agreed on all three. It found one gap, fixed: 0x41F580
  searches anew for the position to attack from, which a flyer never reaches when it is a hex it
  passes over, so such a walk beyond the speed does nothing (a test). Its other note, the
  computer's stop avoidance in a walk's own search, changes no path within a move (the module's
  note).
- **Readings at risk** (for T1b's batch rows): S-11's three outcomes; a berserk flyer whose search
  ends on a hex it passed over (rocks or stacks on the target's near side) does nothing; the war
  machine's aim by its front; the walk beyond the speed into a seen pit of its own side.

## T1a5a — the spell context and the predictions (2026-10-20, increment 1)

- **The cut** (the user, 2026-10-20): part C (12.9 KB in 34 functions by T1a1's sweep, re-run
  the same) in four parts of about 3.2 KB: T1a5a the context and the predictions, T1a5b the raise
  and the Faerie Dragon, T1a5c the Ogre Mage's and the Master Genie's scan with the evaluation,
  T1a5d the estimates. The context first (recommended; three of the four cast kinds, the hero's
  cast and the retreat test read it), over the raise and the dragon first, or the estimates
  first.
- **Read** (`docs/formats/installation.md` §7.42), all in full: the context 0x4369C0 with the fill
  0x43C040, the seed 0x43BF20 and the destructors 0x436C30, 0x436BF0; the predictions 0x422A40
  with 0x4430D0, the side's run 0x422880, the melee prediction 0x4227A0 and the predicted strike
  0x4224E0.
- **Found:** by the cast entry's table only the Archangel, the Pit Lord, the Master Genie, the Ogre
  Mage and the Faerie Dragon cast for the computer; its elementals never do (a reading at risk).
  The context holds per stack of the side the largest strike it can meet this turn (melee, seeded
  with the retaliation at its own target, or a shot), by the enemies' whole counts; the enemy's
  arrow towers are no threat. It marks the enemy beaten when the side's predicted strikes leave
  only war machines, and builds a second context for the other side. The predictions cover only
  the side's run up to the other side's next stack in the predicted order (both runs for the
  retreat test): shots, strikes, multi-headed strikes, breath, Fire Shield's reflection, the
  second strike, and the retaliation only above Easy for a computer side. A predicted strike's
  jousting distance comes from the pathfinder's last search, so a retaliating Cavalier or
  Champion is predicted with the attacker's run-up. The predicted walkers draw their candidates'
  values from the battle's generator. No rule of the engine changes.
- **Map:** 13 entries (5 new: 0x4227A0, 0x4430D0, 0x43BF20, 0x436BF0, the pathfinder pointer
  0x699284; 8 updated, 0x4224E0, 0x422880 and 0x422A40 renamed `ai_predict_strike`,
  `ai_predict_run`, `ai_predictions`); structs: the stack's planned damage and threat record
  (+0x534 to +0x544), the context's layout `ai_spell_context`.
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map made
  before the edits: the cast entry's cases with every write of action 10 (the computer's three, the
  player's three), the run's skips, its end by the controller and 0x4227A0's retaliation tests (on
  Easy only for a human attacker), and the jousting distance's cell with the breath's missing side
  test. It agreed on all three; its note that nothing asks whether the target survived the strike
  is in §7.42 (a dead target strikes back by no hit points). The premise it left open, that the
  cells hold the attacker's own map, rests on 0x421680's call at 0x4217BE.

## T1a5b — the raise and the Faerie Dragon (2026-10-20, increment 1)

- **Scope:** T1a5's second part as cut (the user, 2026-10-20; #320 merged first).
- **Read** (`docs/formats/installation.md` §7.43), all in full: the raise 0x421000 with the count
  0x447330 (renamed `raise_count`: the Archangel's too); the dragon 0x420F00 with its value per
  hex 0x43C620, one stack's damage value 0x436E30, Chain Lightning's 0x437190 and an area's
  0x437040 with 0x5A66B0 and 0x5A4920; the stack record's constructor 0x43D250 and destructor
  0x43D400 by their shape.
- **Found:** the raise takes the side's dead stack (a Pit Lord's corpse) or raisable one (an
  Archangel's) worth the most, `trunc(n × worth)`, doubled when the side's able strength exceeds
  the other's and the balance class is 1, and casts only above the walker's value. The dragon
  takes the first hex whose value is above 0 whatever the walker's value, so it casts whenever its
  spell is worth anything; an area spell may go to a free hex. A spell's value is the kill value of
  its damage on each stack, the friendly part subtracted, and nothing when the cast costs the side
  a larger share of its strength than the enemy's. The estimate takes Expert on the spell's school
  field and the side's hero's bonuses for the bolts, which the rules give a creature's cast
  neither of. The dragon's value reads none of the context's tables. No rule of the engine changes.
- **Map:** 12 entries (9 new: 0x436E30, 0x437040, 0x437190, 0x43C620, 0x43D250, 0x43D400,
  0x5A4920, 0x5A66B0 and the jumps table 0x63B7C8; 3 updated: 0x420F00, 0x421000, 0x447330).
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map made
  before the edits: the dragon's hexes and its first positive value taken whatever the value so
  far; 0x43C620's mastery by field, its routing and the caster's hero in 0x436E30 with the sums'
  tests in 0x437040; the raise's candidates (dead only for the Pit Lord), its value, doubling and
  strict comparison with the entry value. It agreed on all three.

## T1a5c and T1a5d — the Ogre Mage's and the Master Genie's casts, the evaluation, the estimates (2026-10-20, increment 1)

- **Scope:** T1a5's last two parts as cut (the user: "complete T1a5", after #323), one branch.
- **Read** (`docs/formats/installation.md` §7.44, §7.45), all in full: the scan 0x420D20,
  Bloodlust's value 0x43C330 (renamed `ai_bloodlust_value`), the Genie's value 0x43C4A0 with
  0x447EB0, the evaluation 0x43A910 and 0x43A670 with "now" 0x436C60, the evaluators' table
  0x43B690 with the bolts' 0x436F60 and the empty 0x43B680; the attack bonus's value 0x437800
  (renamed `ai_attack_bonus_value`) and the damage of a base 0x443E30 (`damage_of_base`). By their
  shape: 0x437A00, the stack record's copy (`battle_stack_copy`; §7.34's table had it as the
  largest estimate), and the deque's helpers 0x43CB20, 0x43CDF0, 0x5586D0.
- **Found:** the Ogre Mage and the Genie must beat the walker's value; with one to beat, a draw of
  `random(1, 100)` skips the cast at 30 or less and only targets at least as strong as the caster
  count. The Genie's value is the mean over the spells it may draw (the rules' own candidate set).
  Bloodlust's value is the attack bonus's value: the rise of the stack's worth (√ of the damage's
  gain) for the share of the battle's expected length (the balance class) its rounds cover. The
  evaluation scales a value under Magic Mirror by `(50 − c) / 50` and sets "now" for the hero's
  choice. No rule of the engine changes.
- **Found, a gap in T1a1's map:** 0x43B690 hands out a function per spell, 36 of them unread (about
  11.6 KB with their helpers), reached only through its table, so the sweep of calls never counted
  them. The Genie's 21 spells take about 7.3 KB; the rest serve the hero. Proposed as T1a5e for the
  user to cut and place (the Genie's before T1b or T4, the hero's with T1a6).
- **Map:** 52 entries (45 new: the 36 evaluators unread, 0x43C4A0, 0x447EB0, 0x43A910, 0x43B690,
  0x436C60, 0x43CDF0, 0x5586D0, 0x436F60, 0x43B680; 7 updated: 0x420D20, 0x43C330, 0x43A670,
  0x437800, 0x437A00, 0x43CB20, 0x443E30).
- **Checks:** a fresh agent re-read three findings blind, on a copy of the tool and the map made
  before the edits: the scan's draw, order and strength floor; the Genie's mean, the empty
  evaluator and the Magic Mirror scaling; the attack bonus's value with 0x437A00 as the record's
  copy. It agreed on all three; its note that the strength floor also applies after a candidate
  taken in the loop (the value read anew) is in §7.44.

## T1a5e — the spell evaluators (2026-10-20, increment 1)

- **Scope:** found in T1a5c (§7.44: 0x43B690's evaluators, reached only through its table). The
  session proposed four parts by shared helper (the defenses; attack and speed; luck, morale and
  retaliation; the disabling spells and the removals); the user chose one part, before T1b.
- **Read** (`docs/formats/installation.md` §7.46), all in full: the 36 evaluators (§7.44's list)
  and nine helpers, the protection's value 0x4387C0, the defense bonus's value 0x438910 (renamed
  `ai_defense_bonus_value`), the spell protection's value 0x4396E0 (renamed
  `ai_spell_protection_value`; "creature cast value" was a guess), the removal 0x439A80, the speed
  bonus's value 0x439550, the exchange's value 0x435DC0, the damage per creature 0x4426F0, the
  morale or luck change 0x435850 and the berserk attack's value 0x43A340.
- **Found, a correction to T1a5c's sizing:** Cure, Anti-Magic and Dispel value what they would
  remove through 0x439A80, which calls each lasting spell's own evaluator, so the Genie's value
  (through Cure and Anti-Magic) reaches the evaluators of every spell that can last on a stack:
  the neutral side needs 34 of the 36 (only Dispel's and Clone's serve the hero alone), not the
  Genie's 21 (7.3 KB).
- **Found:** most values are the attack bonus's form `(√g − 1) × strength × the rounds' share`
  with `g` a factor on a strike (Bless, Fire Shield, Frenzy, Counterstrike, Fortune) or on the
  damage taken (Shield, Air Shield, Stone Skin through 0x4387C0); the protections take the best
  kill value of the damage they save from the enemy hero's damage spells; speed is valued as
  overtaking the threat (an exchange) and rounds of fighting won; morale and luck by fixed factors
  per point (0.0173 above 0, −0.0833 or −0.0122 below), Fortune as one chance in 24 per point of a
  doubled strike; the disabling spells by the stack's strength. As read, not rule quirks (for T1b
  and T4): the protections take the caster bonuses of the context's own hero and the caster's own
  army for the cost; Magic Mirror keeps `2 × (50 − c)` %; Anti-Magic's estimate removes helpful
  spells and counts level `v`; a morale or luck rise from below 0 is always valued as crossing 0;
  Paralyze reads Blind's record and resistance; Disease asks the resistance to Weakness; Frenzy is
  valued as a second strike without its own value. No rule of the engine changes.
- **Map:** 48 entries (10 new: 0x4387C0, 0x435850, 0x435DC0, 0x4426F0, 0x439550, 0x439A80,
  0x43A340 and the tables 0x63B780, 0x63B790, 0x660858; 38 updated: the 36 evaluators, 0x4396E0,
  0x438910).
- **Checks:** a fresh agent re-read three findings blind, on the tool and the map as they were
  before the edits (the user's `master`): the removal 0x439A80 (the ids it visits, Poison skipped
  without the flag, the spell ended before its evaluator, the context and sign by the side and the
  polarity, its three callers and their flags); 0x4396E0's spell tests, the own hero in 0x5A78E0's
  caster slot and the caster's army in the cost, Magic Mirror's `2 × (50 − c)`; 0x435850's rise
  test with the constants, Blind's record and resistance in 0x439100 and Weakness's in 0x438A10. It
  agreed on all three.
- **Tooling:** the reading ran on the user's computer through the device shell (Python 3.10
  there): `uv run -q --no-project --python 3.11 --with capstone --with pefile python
  scripts/exe.py …` with the repo's own `original-data/install/Heroes3.exe`, no staging. Not to
  repeat (the user, 2026-10-20; #330): the computer may go offline mid-reading; read in the sandbox.

## T1b — the survey and the ADR draft (2026-10-20, increment 2)

- **Scope:** the session proposed three parts (the ADR draft, the survey for the turn and the
  walkers, the survey for the casts); the user: no split.
- **The survey** (`docs/rules/surveys/E23-battle-ai.md`): the reading in one place (two decisions
  per activation, the shooter, the walker, the casts, the difficulty's branches, the draws), the
  wiki and VCMI compared, 18 planned batch rows (4a for the neutral side, 4b for a computer hero's
  side), the original's slips, the ADR's questions. The wiki (read with the summarising fetch):
  "enemy AI heroes … never surrender" (Combat) agrees with the reading, so that reading is no
  longer at risk; Difficulty level: on Easy the AI does not wait and uses no creature tactics, on
  Hard it keeps out of the enemies' reach (the reading: Wait from Hard, the reach from Normal:
  rows W1, W2); the retreat conditions (never on Easy, half the time on Normal, the siege's four)
  go to T1a6. VCMI 1.7.5 plays every side with its own `BattleAI` (default for neutrals too), waits
  by evaluation, casts with every caster creature, and its adventure AI only retreats.
- **The ADR draft** (then `docs/lanes/battle/adr-drafts/original-battle-ai.md`, now ADR-0027): `h3-battle-ai` in the
  **application** layer (arch.toml: a domain crate may not depend on another; the callers are
  adapters); `choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command`, one command
  per call (the hero's cast and the stack's action two calls, the round's mark skipping the hero),
  validated with a Defend fallback; the whole state, the caller's stream; `h3-battle`'s public
  `computer` module (aim.rs, the pathfinder) as the one implementation; destinations only;
  `BAT-AI-nnn`; the budget measured at T2 (provisional 1 ms); `POLICY_VERSION`; the spells' AI
  values into the battle pack before T3 (the neutral side reads the enemy hero's value in the odds).
- **For the user (BAT-Q55):** accept or change the draft; the slips ported as read without quirk
  flags (recommended) or as quirks or fixed; the batch rows. T2 waits for the review.
- **The user's answer on the slips (2026-10-20, BAT-Q55):** one fix-optional flag for all the
  fixes related to the original AI: BAT-QUIRK-25 `quirk.battle.original_ai_slips` (faithful on,
  strict-fixed off), registered with the first slip built. The survey's §4 lists the 17 slips
  with their fixes (a slip: a wrong record, argument, spell, side, mastery or test; tie orders,
  the library sort and the draws are no slips); the draft's point 9 says so.
- **Accepted (the user, 2026-10-20, BAT-Q55):** the ADR draft as amended and the survey's batch
  rows, "all accepted now": ADR-0027 (`docs/adr/0027-original-battle-ai.md`, `shared/` #332, merged
  before this task's #331, which drops the draft). T2's cut (the user): whole, as the epic says; the
  session proposed three parts and the wall shot in T6. Not started in this session.

## T2a — the crate and the plain shooters (2026-10-20, increment 3)

- **The cut** (the user, 2026-10-20): T2 in two, T2a now and T2b's shape at its checkpoint. The
  session proposed it because T2 whole is two pull requests, a new crate, its property tests and
  three kinds of shot, and because BAT-QUIRK-25's flag cannot take the registry's next position:
  the adventure lane's then unmerged `shared/e08-t26b` (#335, merged since) takes 27 and 28
  (ADV-QUIRK-9, ADV-QUIRK-10), so the flag takes 29. The plain shooters need no flag.
- **The registration** (`shared/e23-t2a-crate`, #334, `cross-lane`, merged): the workspace
  member, `[crate.h3-battle-ai]` (application, logic; test rulesets in code), `h3-pack-battle`'s
  dev-dependency on it, `crates/h3-battle-ai/` the battle lane's; the skeleton crate with it (the
  workspace needs the member; #45's precedent). CI could not start (the account's billing): the
  full local verify of the battle branch stands in.
- **`h3-battle`'s `computer` module** (ADR-0027 point 6): `shoots` (BAT-RNG-001's test, the
  original's 0x4428F0) and `shot_target` (`aim::ballista_target`, the automatic Ballista's
  0x41F060 and 0x41EB80); `Stack::is_catapult` public. Nothing else yet: what the policy needs is
  added when a task builds it.
- **`h3-battle-ai`**: `choose` = `unchecked` validated, Defend when refused (ADR-0027 point 3);
  `unchecked` follows 0x4221F0: the side not to act or a battle over: Defend; the tactics phase:
  EndTactics (stand-in, BAT-STUB-5); a stack that can shoot now: the shooter (`shooter.rs`: the
  target's front hex; no target: Defend); the commanded Catapult: the first wall shot the engine
  lists (BAT-STUB-5); any other stack (a walker, the commanded First Aid Tent): Defend (BAT-STUB-4).
  The area shooters and the Cyclops shoot as the others (BAT-STUB-6, T2b). The stream is not drawn
  from yet (`_rng`; the walker's draw, T3). `POLICY_VERSION` = 1. Rules BAT-AI-001 (the dispatch)
  and BAT-AI-002 (the shooter).
- **The shooter's test, not "the Ballista always"**: the original sends the Ballista to its
  shooter whether or not it can shoot; there it always can (unlimited shots, Q-0004). The policy
  asks the rule's test, so a Ballista out of shots (strict-fixed) is no shooter and defends, never
  an illegal shot.
- **Tests**: on a ruleset in code (`src/tests.rs`: the shot at the computer's target and front
  hex, the better value, a walker and a blocked shooter defend, the commanded Ballista shoots and
  the commanded tent defends once it has a patient, the tactics phase, the other side and a
  finished battle, the fallback; a property of 48 random battles with heroes, every difficulty,
  `human` both ways: every choice accepted, the same for the same stream, the same stream left);
  on the harvested packs (`h3-pack-battle/tests/harvested_computer.rs`: 48 battles per profile,
  half at towns of every faction and fort level, a quarter on special battlefields, random heroes
  with Tactics, Artillery, First Aid, Ballistics and spells; 7 steps in 10 play the choice, the
  others a cast or a legal command; reached: 1 734 shots, 42 wall shots, 59 tactics ends, 64
  battles over).
- **Cost** (ADR-0027 point 11, the ceiling 1 ms until T2b's gated benchmark; a throwaway release
  test, not committed): 200 battles of the harvested SoD creatures without heroes, every
  difficulty, half the steps the policy's: 34 532 decisions, 5.9 µs on average (most are the
  walkers' Defend stand-in), 4 185 shots 49 µs on average (the threat records' searches), the
  slowest 0.36 ms. A second run showed two decisions above 1 ms (2.2 ms) that took under 0.4 ms in
  the first: the sandbox's scheduling. The walker (T3) will cost more; T2b's benchmark counts
  instructions.
- **Master's `deny` gate** failed on a new advisory (RUSTSEC-2026-0382, `rustls-webpki` 0.103.15,
  through `h3-server`'s `rustls`); the adventure lane's #335 bumped the lockfile to 0.103.16 (the
  advisory's fix) before this pull request, which was rebased onto it.

## T2b — the area value, the wall shot, the benchmark (2026-10-21, increment 3)

- **The cut** (the user, 2026-10-21): T2b whole, as T2a's session proposed, with the battle
  inbox's failing property in the same session (#340: it fixed `master`'s test build too, which
  #338 and #339 had broken together: `harvested_computer.rs` built `HeroDetails` without T27's
  `offered`; the stack-limit invariant now bounds a side by the larger of the limit and its
  stacks at the start, since seven stacks, three war machines and three turrets pass the
  property's 12).
- **The flag** (`shared/e23-t2b`, `cross-lane` with the user's consent): `quirk.battle.original_ai_slips`
  at registry position 29 (`ORIGINAL_AI_SLIPS`), and `arch.toml` lets `h3-cli` depend on
  `h3-battle-ai` (the benchmark). A flag cannot be registered alone: `h3-pack-battle`'s
  `the_profiles_differ_exactly_by_the_quirk_flags` wants every flag on in faithful and off in
  strict-fixed, so the same pull request turns it on in the faithful profiles (`packs/base`,
  `packs/synthetic`, `h3-harvest`'s profile text): the faithful core ruleset hashes moved
  (synthetic `0x62e9ef353e1342b0` → `0x203012933e60e190`, base `0x6384653ed1e4c038` →
  `0xe6839bfb42ebe118`; strict-fixed unchanged), so the ten golden replays' and the adventure
  lane's seven faithful replays' headers were patched and the AI lane's `bots-report-v1.json`
  regenerated (only `engine.core` moved). No code reads the flag before the battle PR.
- **The area value** (BAT-AI-003; 0x41EDA0, slip 1): in `h3-battle`'s `aim.rs`, because the
  target choice 0x41EB80 it scores is the automatic Ballista's too (one implementation, ADR-0027
  point 6): `pick` scores an area shooter's target at its position and, for a two-hex stack, at
  its other hex when strictly higher (the shot aimed there), and skips one below 0; `area_value`
  sums `shot_value` for the other side's stacks within one hex and subtracts the undivided
  `kill_shot_value` (0x435C70) for the acting side's. Slip 1 behind BAT-QUIRK-25: faithful, an
  undead shooter leaves out the undead not covering the hex, nothing else; strict-fixed, the
  stacks the rule spares but the target (`combat::spares`, factored out of BAT-RNG-004's area).
  The side columns, which 0x5A4170 leaves out, hold only a war machine's rear hex, whose other
  hex is beside the aimed hex whenever the rear is: the estimate's stacks are the rule's. Re-read for the port: the side test is the record's side against the acting side; the
  area is asked for creatures 45, 64, 65 by number (the engine: the `area_shot` ability).
- **The frame and the wall shot** (BAT-AI-004; 0x41F060, 0x41EEA0): `aim::shot` is 0x41F060
  with the wall test as its caller's closure (the Ballista passes none; the policy passes
  0x41EEA0, which draws from its stream): the first choice (desperate mode first), the wall test
  for a `wall_shots` stack with the first choice's value, then no target gives Defend at once, a
  value of 0 in desperate mode a second choice (one that finds none, the original's action 7 at
  hex −1, gives nothing). The wall test in `h3-battle-ai`'s `shooter.rs`: not for the defender
  (the controller), not without a standing wall (`computer::walls`, kind wall in index order);
  `B` sums `computer::strength` (0x442E60) over the side's living stacks with no threat record
  (`computer::records_no_enemy`: `aim::turns` now also leaves out a stack blind or stone for more
  than a round, 0x422BD0, which its other callers never reach); `B / S₀ > 3 × best / S₁` as the
  FPU compares two rounded quotients (a NaN on either side counts as above, `fcompp`'s C0); the
  weakest standing wall, `random(1, n)` among equals in index order. `computer` exposes
  `Context` (its strengths), `Shot`, `shot`, `records_no_enemy`, `strength`, `walls`.
  `POLICY_VERSION` 2; BAT-STUB-6 closed.
- **Tests:** the AI crate's ruleset gained a magog, an undead lich, fast fire-immune, lifeless and
  undead walkers and a two-hex cavalier, and the profiles `t:faithful` (the flag) and `t:fixed`:
  own runners beside the target make the magogs shoot past it or defend (both profiles); the
  slip's three cases (efreets beside a magog's target, golems beside a lich's: faithful defends,
  fixed shoots; zombies: both shoot; a zombie target is shot); a two-hex target aimed at its
  position, or at its front when the runners stand beside the position only; the FPU comparison
  table; the property over random battles with the new creatures in both profiles. On the
  harvested packs: cyclopes and pikemen outside a Fort's walls draw every wall over 40 streams,
  then batter the weakest one (alone once a wall lost hit points) until they shoot; a lone
  cyclops, the open field and the defender's cyclopes shoot. `h3-battle`'s aim test: blind or
  stone for two rounds, no target; blind for one or paralysed, a target.
- **Cost** (ADR-0027 point 11): the gated workload `benches/battle/computer-decisions.toml`
  (`h3 bench-computer --games 40 --seed 1`: random harvested SoD armies without heroes, every
  difficulty; on the open field the defender is the computer against random commands, at a Fort
  both sides) counts 274 292 359 instructions inside `choose` over 7 301 decisions: **about 37 600
  instructions per decision**, most of them the walkers' Defend stand-in (T3 will move it). The
  shot values first ran a search for a target's turns at every call (an area shooter asks one
  per stack of each area, twice for a two-hex target, again in desperate mode's second choice):
  `aim::Threats` now computes each target's turns once per choice and only when a value needs
  them (eagerly, the workload fell by 20 % but the synthetic siege rollouts rose by 47 %: on Easy
  no value needs them). Release, on a quiet sandbox (`h3 bench-computer`, three
  workloads of 40 and one of 400 battles): 2.7 to 5.3 µs per decision on average, the slowest
  0.15 to 0.68 ms, within the 1 ms ceiling (with a full verify running beside it a decision took
  up to 4.8 ms: the sandbox's scheduling, as in T2a). The budget is the user's at this checkpoint
  (ADR-0027 point 11).
- **A blind check** (a fresh agent, the disassembly against the port, ON-branch only): no
  discrepancy in 0x41EB80, 0x41EDA0, 0x41F060, 0x41EEA0 or 0x422B20's skips; it noted the plain
  shot's front hex (the original aims at the position; the engine's validation wants the front,
  the same stack is hit) and the side columns' war machines (above).

## T3-0 — the spells' AI values (2026-10-21, increment 4)

- **The checkpoint** (the user, 2026-10-21). **The budget** (ADR-0027 point 11): the gated count
  of `computer-decisions` with its +2 %, re-set on purpose at each part of T3 (the walkers cost
  more), and at most 1 ms for the slowest decision on a quiet sandbox (`h3 bench-computer`),
  measured and recorded per task. **T3's cut**: T3-0 the pack data (ADR-0027 point 13), T3a the
  plan and the attack, T3b the hex scores and the hold-back, T3c the odds and the ends; this
  session T3-0 and T3a.
- **The data** (pack format v45): a spell's `ai = { kinds = [...], values = [none, basic,
  advanced, expert] }` in `battle.toml` (`h3_content_battle::SpellAi`, `BattleSpell.ai`; default no
  kind, values 0). The kinds are the executable's AI category bits `0x8000` to `0x100000`
  (`SpellAiKind`: `damage`, `damage_once`, `damage_all`, `effect`, `stacks`, `special`), a set:
  Titan's Lightning Bolt has `damage` and `special`, which the original values at 1
  (`SpellAiKinds::only`, §7.39). The values are `SPTRAITS.TXT`'s columns 25 to 28 (the harvester
  wrote "AI values not harvested" before). The battle part's hash takes both (v7); no rule reads
  them, so no behaviour moved. `installation.md` §7.6: the categories' meaning is confirmed by
  §7.39's reading (0x5273D0 dispatches on exactly these bits).
- **Harvested** (SoD's executable, the AB-era table): 81 spells, the seven damage spells the odds
  count with §7.39's values (Magic Arrow 10, Ice Bolt 20, Lightning Bolt 25, Implosion 75, Frost
  Ring 12, Fireball 15, Meteor Shower 37, the same at every mastery); `special` Summon Boat, Fly,
  Water Walk, Dimension Door, Town Portal and Earthquake; no category the other adventure spells,
  the four battlefield spells (10 to 13) and the creature abilities (70 to 80). The synthetic
  pack's spark, cinders, arc, blight and ward carry values for the tests.
- **Hashes:** the ten golden replays' `battle_ruleset_hash` `0x827992c874254045` →
  `0x7504e0bf2b62c97c` (headers patched, `h3 verify`); the AI lane's `bots-report-v1.json`
  regenerated (only `engine.battle` moved, `dd49b3f029b159bb` → `c14c573e9d2ee26f`; `cross-lane`,
  as #345). The adventure lane's replays store no battle hash.
- **Not here:** `h3-harvest` keeps the kinds' names in its own table (`SPELL_AI_KINDS`, a test
  compares it with `SpellAiKind::ALL`): a normal dependency on `h3-content-battle` would be an
  `arch.toml` change.

## T3a — the walker's plan, choice and walk (2026-10-21, increment 4)

- **The reading** (`installation.md` §7.47): the melee half of the exchange 0x4359B0 (E04c left it
  unread): Fire Shield's estimate 0x422440 of each strike, the retaliation by the survivors (none on
  Easy for a computer side, none for a creature whose strikes are not retaliated, a petrified
  enemy or one without a retaliation left) and a second strike for `0x8000`, also when no
  retaliation came; 0x442780 doubles only shots; 0x435B90 and 0x435C70. And the call sites the
  port needs: the map's side (its own stacks unmarked, 0x4B3801 by the record's side), the melee
  value's flag 0 and run-up, the plan's value and the draw's arithmetic, the comparison's order,
  the action; 0x436840's three hexes for a two-hex walker; 0x436300's stop by the position hex
  alone; 0x436180's doubles; 0x436620 and 0x436760's arguments; 0x448AB0 corrected (one
  direction on each side, not two clockwise; §7.36 and the map fixed); 0x523DF0, 0x523D90;
  0x41F580's own-side player test and the Wait's score condition; 0x421F80's Wait.
- **The port.** `h3-battle`'s `computer` gains the context, worth, the damage estimate (0x442780,
  any count, shot or strike with a run-up: `combat::melee_damage`), the damage value (0x442FD0),
  the strike value (0x435C70 with the exchange), the blocker-aware shot test (0x4428F0 with a
  stack left out: `combat::can_shoot_without`, beside the hot `can_shoot`), the walker's map
  (`pathfinder::Search` with a spared side, its stops) and the walk's path, `bound`, `ftol`, the
  difficulty, `human`, the wait phase; `aim::avoids_stops` splits off `fears_moat` (the
  computer's own walker is always automatic, 0x474BF0). `h3-battle-ai`: `hexes.rs` (the original's
  directions, 0x523D90, 0x523DF0, 0x445840, 0x448AB0) and `walker.rs` (0x421F80, 0x421680 outside a
  prediction, 0x4360C0, 0x436840, 0x436300, 0x436180, 0x436620, 0x436760, 0x41F580). By the engine's
  data, not the original's numbers: the multi-headed are `attacks_all_adjacent` and
  `three_headed_attack` (the Cerberi), the Harpies `strike_and_return`, the jousters the Jousting
  ability. `POLICY_VERSION` 3; the First Aid Tent keeps its stand-in (BAT-STUB-5).
- **A difference the property found** (the harvested packs, a Castle's moat): a two-hex walker
  plans a one-turn attack from a position whose other hex is in the moat (0x436300 counts a stop
  by the position hex alone); the engine refuses that attack (no attack from where the walk stops,
  BAT-MAG-032's rule). The original's walker stops there and the move and attack skips the
  attack (0x441FA0, 0x478971; §7.16), so the policy gives the move: the same outcome.
- **Not built here** (BAT-STUB-4): the hex scores (T3b; 0 meanwhile: the walk's score tests then
  never refuse, but its other rules hold), the stronger side and with it `P` alone for a negative
  value, the last retaliation's gain, holding the walls, a besieger's walk, the fallbacks (T3c),
  the creature casts (T4). Dead in the engine and left out: the plan's turns for a stack without
  speed (only war machines have none, and none walks), the walk's guard for a bound walker (its
  plan's hex is its own), the map's range 0 for a disabled stack (no turn).
- **Cost** (the budget: the count re-set on purpose, at most 1 ms for the slowest decision):
  `computer-decisions` counts 133 069 578 instructions over 589 decisions, **about 226 000 per
  decision** (T2b's walkers defended: 37 600), the battles shorter now that walkers fight; the
  other battle workloads within 0.02 % (`estimate` goes through the general estimate). Release on
  a quiet sandbox (`h3 bench-computer`, 400 battles, seeds 4 to 7): 24 to 37 µs per decision on
  average, the slowest 0.17 to 0.27 ms in most runs, up to 6.5 ms in others: the same
  deterministic decision varies run to run (the sandbox's two cores). Timed as each decision's
  best of five (a local patch, not kept), 2 000 battles of five seeds: 18 to 20 µs on average, the
  slowest 0.13 to 0.22 ms. Within the budget.
- **A blind check** (a fresh agent, the disassembly against the port): no discrepancy in 0x421680
  (outside a prediction), 0x421F80, 0x4360C0, 0x436840, 0x436300, 0x436180, 0x436620, 0x436760,
  0x448AB0, 0x445840, 0x523DF0, 0x523D90, 0x41F580, 0x4359B0, 0x435B90, 0x435C70, 0x442780 and
  0x422440; the stopped attack's move found sound (0x478971 jumps past the strike when the walk
  returns false). Three differences it names cannot arise: no direction to the struck stack (the
  original's mask 0x80000011 or a read just before the table 0x660878; the port 0), the turns without speed, the
  walk of a bound walker (the original's stale action; the port nothing).
- **Tests:** the runner attacks the archers and the blocked archers strike back from their hex;
  a Harpy flies to fifty archers without striking (`P` alone), then strikes once it blocks them
  (worth 1), and with its hex scored high stays; the walk with scores by hand (holding back on
  Hard, the whole-turns rule without the flag, no path); the hex geometry; the property over
  random battles with Harpies; on the harvested packs the Cyclopes blocked by a defender walk.

## T3b — the walker's hex scores (2026-10-21, increment 4)

- **The checkpoint** (the user, 2026-10-21): T3b whole, one pull request (the cut into the enemies'
  reach and the hazards was offered and declined).
- **The reading** (`installation.md` §7.48): §7.36's builders re-read in full for the port, the
  call sites in 0x421680's head, 0x4B2DA0's marks and positions (+0x4A, +0x4B), the speed argument
  of 0x4B3400 (a stop's cost, a mark from a stop, the refusal 0x4B378D with the range within the
  speed, a flyer's pass-over by its own speed 0x4B39E7), 0x41FD60's positions by index (0x41FEF8),
  0x41F920's area spells through 0x5A7890 and 0x5A8090 (the flags 0, 1, 0; `f` on the FPU, the
  damage stored as an `f32`). **Two new slips** (survey §4, BAT-QUIRK-25): the Fire Wall's scores
  value the damage without the trigger's immunity test (slip 18), and both hazards pass the
  context's least attack bonus for both bonuses (slip 19). Built with them: slip 3 (the
  multi-headed value's sign).
- **The port.** `h3-battle`: the pathfinder's query takes the turn's speed (`Query::speed`; every
  earlier caller passes none, the stack's own) and refuses the marks from a stop with the range
  within it (unobservable for the earlier callers: their range is 127 or reads no marks);
  `computer` gains the enemy's reach map (`reach_map`: speed + 1 as range and speed, the player's
  stop test as 0x474BF0's), `Map::reached` and `Map::mark`, the Fire Wall's damage with the rule's
  hurt test (`fire_wall_damage`, through `magic::obstacle_damage`, which the burn now calls too), the
  damaging moat (`damaging_moat`), the hero's best area spell (`area_spell_damage`:
  `aim::best_area_spell` with `magic::effect_factor`, 0x5A8090's `f` beside `affect_chance`), and
  `Context::attack_for_both`. `h3-battle-ai`: `scores.rs` (0x4214F0, 0x421590, 0x420260, 0x41FD60,
  0x420060, 0x41FB60, 0x41F920); `hexes::stack_neighbor` takes the original's index arithmetic for
  a two-hex stack's other hex (`stack_neighbor_at`; no difference at a stack's own position, so
  T3a's choices stay); the walker builds its scores (BAT-STUB-4 narrowed). By the engine's data,
  not the original's numbers: the multi-headed by their abilities, the ball shooters by
  `area_shot`, the area spells by a hex target with an area and a damage effect (Frost Ring,
  Fireball, Inferno, Meteor Shower in the packs). `POLICY_VERSION` 4. The mask goes to the area
  threat only; the fallback's own call (0x421E94) is T3c's.
- **Cost** (the budget: the count re-set on purpose, at most 1 ms for the slowest decision):
  `computer-decisions` counts 274 968 963 instructions over 831 decisions, **about 331 000 per
  decision** (T3a: 226 000; each walker now searches every enemy's reach), the battles longer again;
  the other workloads within +0.8 % (`random-rollouts-full` +0.05 %, `-siege` +0.36 %; the
  adventure lane's +0.5 to +0.8 %, under their gate). Release, each decision's best of five (a local
  patch of `h3 bench-computer`, not kept), 2 000 battles of seeds 4 to 8: 30 to 37 µs per decision on
  average, the slowest 0.16 to 0.28 ms; single timings up to 3.5 ms (the sandbox's scheduling, as in
  T3a). Within the budget.
- **Tests:** the reach (Easy none, a clamp at the floor for desperate imps, two enemies' values
  adding up, an enemy within a turn of the side adding nothing, a two-hex enemy's beside hexes), the
  multi-headed strike (Hard for a computer side only, the Cerberi's fewer hexes, a second struck
  stack's 1, slip 3 both ways), the crowding (the area threat around two friends, the breath line,
  an enemy dragon in the mask), the Fire Wall with slips 18 and 19 both ways on the test ruleset
  (a Fire Wall spell and the efreet's fire immunity added to it) and on the harvested packs (the
  minotaurs strike from a hex off the wall, the efreet only with the slip); `h3-battle`'s
  `tests/computer.rs` for the new surface (the reach map's speed + 1, a stop's cost and refused
  marks, the Fire Wall's damage and hurt test, the moat, the area spell with costs, immunity,
  resistance, an aura and the Orb of Vulnerability in both precisions). The harvested property's
  shot count floor went from 500 to 300 (415 shots now: the walkers' choices changed the games).
- **A blind check** (a fresh agent, the disassembly against the port with the slips on): no
  discrepancy in 0x421680's head, 0x4214F0, 0x421590, 0x420260, 0x41FD60, 0x420060, 0x41FB60,
  0x41F920 or 0x4B3400's uses of its speed argument; 0x5A78E0's defending hero, which the Fire
  Wall's call passes as the acting side's, is never read. Not in its brief: the queue's push
  0x4B3BB0 and the stops 0x4B3290 (T1c's port), the estimates' insides (T2, T3a's), the record
  order (`state::RECORD_ORDER`).
- **Not built here** (BAT-STUB-4, T3c): the stronger at range, the last retaliation's gain,
  holding the walls, a besieger's walk, the fallbacks. Predictions only (T4, T5): a struck stack
  without hit points after the planned damage (0x41FE2E), the hit points left after it (0x443080
  with the context's flag).

## T3c — the odds and the ends (2026-10-21, increment 4)

- **The checkpoint** (the user, 2026-10-21): T3c whole, one pull request (a cut into the ends, the
  last retaliation's gain and the odds was offered and declined). Two calls during the session:
  **slip 6** (the hero's value at the mastery of the hero's own adventure tile) stays a gap, both
  profiles take the battlefield's (the battle's tile is the defender's, so only an attacking hero
  on another magic ground differs; BAT-STUB-7, BAT-Q56 for a seam field later); the turrets valued
  at creature 2's AI value (the Archers) become **slip 20** under BAT-QUIRK-25, the turret's own
  value without it (no pack change).
- **The reading** (`installation.md` §7.49): 0x421680's tail re-read (the plan value per turn, the
  losing attack, the besieger's walk with 0x469DC0, the fallback's order); 0x41F140 and its sort
  read in full (Visual C++ 5/6's `std::sort` step by step: the median, the unguarded partition,
  the recursion into the part not larger, the insertion sorts); **0x422B20 with flag 1** (the
  walker's turn): no record for a stack that can shoot now, the map's range the stack's speed, and
  only enemies of a lower rank (acting later); a dead enemy's place counts by its stale rank;
  0x448840's turns and the strictly higher value; 0x41F3B0 (unlimited retaliations are 5000);
  0x420760, 0x4205D0 (the hex record's x anchor), 0x4B3F20, 0x4208F0 (the own position at cost 0
  takes an equal score); **0x4213F0's part 6** reads the battle manager's +0x164, byte 93 of the
  display overlay +0x107 (0x4937D0, H3API's `accessibleSquares2`): clear in a hidden battle, a
  breach; hex 0 never blocked at a fort; 0x420A80's turrets (creature 2's +0x40 at 0x420C59, the
  keep part 14, the towers parts 5 and 13); 0x5273D0's cost with no enemy army and field −1;
  0x5270E0's comparisons and stores; the table's doubles are the nearest to the decimals.
- **The port.** `h3-battle`: `computer::turn_map` (range the speed), `kept_out`, `hero_value`
  (`aim::hero_value`, `spell_value`, the curve as thousandths through `X87::ratio` at 53 bits),
  `siege` (the town whole), `SiegeState::inside` public, `magic::untargeted_bonus` (0x4E5760
  without a stack: the Orbs and Sorcery; `caster_rates` shared with `caster_stage`); the odds'
  cost is the spell's at the hero's own mastery without any army's change. `h3-battle-ai`: `order.rs` (the keys, the library sort, the alternation, the ranks),
  `gain.rs` (the flag-1 threat records, built once per decision when first needed; the gain),
  `odds.rs` (the stronger at range, the turrets), `ends.rs` (holding the walls with `breaches`,
  the besieger's walk, the fallback, the guard and its hex, the step to safety); the walker reads
  the odds once per decision (a cell) and the mask its scores built (`scores::build` returns it;
  `scores::area_threat`); `walk_with` walks without scores for the guard. `POLICY_VERSION` 5.
- **A blind check** (a fresh agent, the disassembly against the port with the slips on): no
  discrepancy in 0x41F140 and its sort, 0x422B20 with flag 1 and 0x448840, 0x421680's tail,
  0x4208F0 and 0x4213F0 (its part 6 confirmed: the overlay's byte 93); the table checked value by
  value. Three differences, all fixed: 0x41F3B0 loops over the acting side's records (a
  hypnotized walker's controller's; the port had the walker's own side); 0x4E5240 given no enemy
  army skips the own Mages too (0x4E52DA; the port had counted them); 0x420760 defends when the
  walker stands on the last shooter's guard hex, not the best's (0x4208B4: **slip 21**, joined to
  BAT-QUIRK-25 as the user had the turrets' creature).
- **Not built here** (BAT-STUB-4): the creature casts (T4). Kept apart (documented): a dead enemy
  in the flag-1 records; the stale action the original keeps when the besieger's or the guard's
  walk finds no path (the port defends).
- **Cost** (the budget: the count re-set on purpose, at most 1 ms for the slowest decision):
  `computer-decisions` counts 684 134 426 instructions over 1 784 decisions, **about 383 000 per
  decision** (T3b: 331 000), the battles twice as long (2 272 steps against 1 097: the walkers hold
  back, guard and walk to the walls); `random-rollouts-full` +0.08 %, `-siege` +0.42 %. Release,
  each decision's best of five (a local patch of `h3 bench-computer`, not kept), 2 000 battles of
  seeds 4 to 8: 31.7 to 37.2 µs per decision on average, the slowest 0.19 to 0.34 ms. Within the
  budget.
- **Tests:** `h3-battle-ai`: the library sort (sixteen equivalent pairs kept in order, a pinned
  order above sixteen, a property that it sorts), the ranks with equal speeds alternating and
  after a Defend and a Wait; the odds by shooters and the turrets with slip 20 both ways at each
  fort level; the gain against the formula, the desperate kills with slip 4 both ways (970 and
  70), no gain without the last retaliation or for a stack acting after the enemy; the besieger's
  walk (one- and two-hex walkers, with and without the moat), the walls test's breaches with slip
  2 both ways, holding at a Citadel (the keep) and a Fort (archers), the guard instead of a losing
  attack, no guard under an area threat, the guard's Defend and the step to safety; the property
  over random battles now one in three at the town. `h3-battle`'s `tests/computer.rs`: the hero's
  value (§7.39's example 7179, one cast 2697, the cost above the maximum mana, the curve's row 0, S
  = 0, Sorcery 9016 in both precisions, the level limit, no book, casting forbidden, neither the Mages
  nor the enemy's Pegasi, the kinds), the turn's map; slip 21 both ways; the attacker's two-hex
  multi-headed enemy in the hex scores (a fixed test for a branch the property had reached).

## T4a — the creature cast entry, the raise and the Faerie Dragon (2026-10-21, increment 5)

- **The checkpoint** (the user, 2026-10-21): T4 is a port of readings done in T1a5 (§7.42 to
  §7.46), cut in three: T4a the parts without the spell context (the entry, the raise, the
  Faerie Dragon), T4b the context's predictions and the Ogre Mage, T4c the Master Genie (cut again
  at its checkpoint). Two calls during the session: **the dragon's targets** follow the engine's
  (the original's dragon values every hex outside the side columns, a free one, its own and an
  immune stack's included, and casts at the best; the rules' cursor, BAT-MAG-049, offers only
  stacks the spell affects): the policy picks the best of the engine's targets, the difference a
  stub (BAT-STUB-8) noted under BAT-Q32 (1), which already asks whether the dragon's area spells
  may go to any hex; **the dragon's predictions** (no preference): the original builds the spell
  context and reads only its battle context, so the port skips the predictions and their draws.
- **The reading** (`installation.md` §7.50): 0x421680 writes the caller's value (the larger of the
  best's two per turn values, 0 without a best) before the entry, which it asks **whether or not
  there is a best**; the plan's hex alone writes its plan value then; 0x421F80 asks again with the
  value 0x421680 left. 0x421280's table (0 at 13 and 51, 1 at 37 and 91, 2 at 134); 0x421000's
  order, test and doubling; 0x435EC0's balance class; 0x43C620's mastery and spell jumps;
  0x436E30's arguments to 0x5A78E0 and 0x5A8090 (the creature flag reaches only the raise spells);
  0x437190's estimate branch of 0x5A61F0 (the factor above 0, no roll). No new slip; slip 7 built.
- **The port.** `h3-battle`'s `computer`: `spell_damage_value` (0x436E30, in `aim.rs` beside the
  hero's area spell), `raise_count` (0x447330, `casters::raise_count`), `created_worth` (the
  side's template of the raised creature: `aim::worth` split into `worth_of` for a stack off the
  field), `chain` (`magic::ChainEstimate`: the rules' chain search split into the rolls and
  `nearest`, the estimate without rolls), `area_stacks`, `creature_casts_blocked`,
  `creature_mastery`, `field_raises`; `CastSpell::named`. `h3-battle-ai`: `casts.rs` (the entry by
  ability: raisers, random casters, the rest nothing; the balance; the raise; the dragon with
  `Kind`, the per-stack value cache of the area spells and `pays`); the walker threads the value
  (`choose(rng, &mut value)`, the entry before its branches and again in `turn`).
  `POLICY_VERSION` 6.
- **Kept apart** (documented): the original's raise ignores the engine's slot test for the raised
  Demons (BAT-STUB-8); Titan's Lightning Bolt would be a bolt by its data (the original gives 57
  nothing; no random caster draws it); the estimate's area is the rules' (the original's
  0x5A46F0 never takes a side column).
- **Cost** (the budget: ≤ 1 ms for the slowest decision; the gated count re-set): the workload's
  battles run shorter now that the casters cast (1 151 decisions, 1 718 steps; T3c 1 784 and
  2 272), so `computer-decisions` fell to 477 211 553 instructions, **about 414 600 per decision**
  (T3c: 383 500, +8 %); `random-rollouts-full` +0.08 %, `-siege` +0.38 % (as on `master`).
  Release, each decision's best of five (a local patch of `h3 bench-computer`, not kept), 2 000
  battles of seeds 4 to 8: 50.2 to 57.8 µs per decision on average, the slowest 0.34 to 0.43 ms;
  `master` on the same sandbox 50.6 to 59.3 µs, the slowest 0.34 to 0.46 ms (this sandbox is
  slower than T3c's). Within the budget.
- **Tests:** `h3-battle`'s `tests/computer.rs`: the spell damage value (the kills, the cap, the
  immune stack, the hero's bonuses only with the flag, a disabled stack's `2v − w` and its whole
  hit points, a dead stack, a spell the ruleset lacks), the raise count (both raisers, whole
  creatures, nothing for another stack) and the raised imp's worth as an imp's on the field, the
  estimated chain against a real cast (the same stacks and values; four from Advanced; none for a
  bolt) and the nova's area, the casts' battlefields (Magic Plains, an elemental field, Cursed
  Ground, a ban) and `named`. `h3-battle-ai`'s `tests/casts.rs`: the balance classes 7, 5, 2, 1, 1
  from either side. The harvested `harvested_computer_casts.rs`: the Archangels raise the bolted
  Pikemen rather than strike a Peasant (both profiles; no target at their first turn; no cast
  left after it), the raise loses to a strike on Marksmen, the Pit Lords raise 11 Demons from the
  Imps' corpse, the Faerie Dragons cast every drawn spell at an enemy (Frost Ring at a stack's
  hex hits nothing among stacks far apart: no cast), no cast on Cursed Ground, slip 7 both ways on
  Lucid Pools (a Naga Queen worth more at Advanced, a Behemoth at Expert), no chain or area spell
  that hurts the side's Archangels more than a Peasant beside them. The random-battle property
  (`harvested_computer.rs`) now plays the casts.
- **A blind check** (a fresh agent, the disassembly against the port with the slips on): every
  function agrees. Two notes, no change: 0x5A61F0 tests a chain candidate as cast by the side
  opposite its own record side (only the Orb of Vulnerability's negation depends on it; the
  estimate keeps the rules' search, BAT-Q28; §7.50); the agent read the slip-off bolts' missing
  hero bonuses as wanted for the chain and area spells too, which the rules give the hero's
  (BAT-MAG-049, §7.21's review re-read).

## T4b1 — the Ogre Mage (2026-10-21, increment 5)

- **The checkpoint** (the user, 2026-10-21): T4b is cut in two. T4b1 the Ogre Mage (the scan,
  Bloodlust's value, the attack bonus's value), "the enemy beaten" from the state; T4b2 the
  predictions (§7.42), which on the neutral side decide only that flag; the context's threat
  tables and second context (0x43C040, 0x43BF20), which only the Genie's evaluators read there,
  move to T4c. **The Ogre Mage against the elementals:** the engine gives both one ability (a
  creature's cast of a fixed spell, BAT-MAG-048), the original sends only creature 91 to the
  scan; the policy tells them apart by the spell (the only fixed cast that raises the melee
  attack, Bloodlust), no pack change (the alternative, a pack field from the table 0x42136C, was
  declined).
- **The reading** (`installation.md` §7.51): the scan draws when the value so far is not 0 (a
  negative one too); the strength floor is re-read per record and lets an equal strength pass;
  0x43C330's mastery by battlefield (Bloodlust is fire: Fiery Fields only, slip 8); power and
  rounds 6; the single path is the evaluator 0x438100's; the mass path's skips (blind, petrified,
  paralysed at any duration, dead, the Tent and the Cart, the spell lasting; hypnotized stacks and
  war machines left to 0x5A3C80 and the evaluator), the caster included in the sum; 0x437800's
  order of operations; 0x443E30 is the estimate's stages (0x443840 passes `estimate == 0` to
  0x443320, 0x443E30 passes 0); 0x422B20 with flag 0 (range 127, a shooter every enemy record at
  cost 1, the dead included; no range 0 for a disabled stack there).
- **The port.** `h3-battle`'s `computer`: `threat` (0x422B20 flag 0's record: the enemy and its
  turns; `aim::threat`), `base_damage` (0x443E30 on a copy of the record with a bonus to its
  attack; `combat::estimated_damage`, which `melee_damage` and `shot_damage` now share),
  `spell_affects` (0x5A3C80 as `magic::affects_stack`), `takes_stack` (0x59E060 as the rules'
  `target_at`). `h3-battle-ai`: `spells.rs` (the spell context with the balance class and the
  threat records cached per decision; `attack_bonus`, 0x437800; `bloodlust`, 0x438100; `on_side`,
  the mass sum), `casts::scan` and `casts::bloodlust` (0x420D20, 0x43C330); the entry's
  `SpellCaster` arm; the entry takes the stream. `POLICY_VERSION` 7.
- **Found:** where the rules cast a creature's mass version (the Ogre Mage on Magic Plains, Expert
  by BAT-TER-005), `apply` accepts only `CreatureCast { target: None }`, which
  `creature_cast_targets` does not list: the scan then values each stack the spell affects and
  casts without a target. The listing's gap is in this lane's inbox (and the AI lane's: its
  action mask misses the cast).
- **Kept apart** (documented): the threat records leave out dead enemies (as T3c's walker
  records); "the enemy beaten" is never set and the predictions draw nothing (BAT-STUB-9, T4b2).
- **Cost:** `computer-decisions` 477 220 414 instructions, +0.00 % against its baseline (the
  workload's random armies seldom hold Ogre Mages that cast): no re-set. Release, each decision's
  best of five (a local patch of `h3 bench-computer`, not kept), 400 battles each of seeds 4 to
  8: 32.9 to 46.6 µs per decision on average, the slowest 0.23 to 0.29 ms; `master` on the same
  sandbox 31.1 to 44.5 µs, 0.22 to 0.28 ms. An Ogre Mage's decision with seven stacks a side at
  the battle's start (the mass version on Magic Plains included): 0.16 to 0.20 ms. Within the
  budget.
- **Tests** (the harvested `harvested_computer_casts.rs`): nine Dendroid Soldiers (strength 6 885,
  Bloodlust worth 930) are blessed over the Ogre Mages' attack on 15 Silver Pegasi unless the
  draw skips, eight (6 120, below the mages' 6 720) never; over 200 streams 120 to 160 casts, the
  rest attacks; an attack worth nothing (20 pegasi) draws nothing; 30 dendroids whose strike
  already takes 351 of 360 hit points (the cap: 292) do not beat the attack; the mass version on
  Magic Plains without a target, both stacks blessed (both profiles); slip 8 on Fiery Fields (the
  mass estimate, the mages' own share 247: a cast; without the slip the acted behemoths' 0: none);
  a blessed stack is not blessed again; Storm Elementals never cast (BAT-Q57); the threat records
  (a shooter's at one turn, a walker's further, none for the dead) and the damage of a base (105,
  52; 135, 67). `h3-battle-ai`'s `tests/spells.rs`: the attack bonus's value by hand (ten imps
  against ten, class 7: g = 125 / 97, 115 over six rounds, 135 over seven, 98 once they acted with
  their strength 1 024), the Ballista's record, none for the Tent or for imps behind a closed gate.
  The speed-0 test of 0x422B20 is not ported: no stack that searches has speed 0 here (the war
  machines are left out, the Ballista shoots).
- **A blind check** (a fresh agent, the disassembly against the port with the slips on): every
  function agrees. One note, no change: the original hard-codes the Ogre Mage's mastery 2 and its
  rounds 6, the port reads the ability's mastery and power (2 and 6 in the packs).

## T4b2 — the predictions (2026-10-22, increment 5)

- **The checkpoint** (the user, 2026-10-21): T4b2 whole (a cut into the shooters' predictions
  first, or the prediction mode first, was offered and declined). The creature cast listing's gap
  that T4b1 found was fixed first in a PR of its own (#MASS, `cross-lane`: `creature_cast_targets`
  lists `None` for a creature's mass version; the AI lane's mask gains it).
- **The reading** (`installation.md` §7.52): the run's pass-over tests (any duration of Blind,
  Stone Gaze, Paralyze; the Catapult by creature), its controller test, only actions 7 and 6
  counting; the flag's readers in both turns (the value 0x435B90 is 0 when either stack has no
  hit points left; the shooter's target, the threat records, the walker's candidates, the blocked
  shooters, the multi-headed danger pass such a stack over; the plan's counts, the floor and the
  last retaliation read the hit points left); in a prediction the walker asks no cast, gives its
  walks no careful flag and of its fallbacks keeps only the attack after all; 0x4224E0's order
  (the multi-headed record bit set before the hit-point test; Fire Shield before the damage; the
  breath after it, unreflected); the last search is the predicted walker's own map (its run-up,
  and the mark's cost for a retaliation); 0x4369C0's context keeps +9 clear; the enemy beaten
  passes over the war machines (the arrow towers among them).
- **The port.** `h3-battle`'s `computer`: `Planned` (the table by stack id, 0x4430D0's cap),
  `Context::predicting`, `predicts`, `left`, `spent`; `predicted_shot` (0x41F060 with the flag:
  the context predicting, no wall shot); `fire_shield_estimate` (0x422440). `aim.rs`: the value
  (`strike_value`, `kill_shot_value` through `pair_left`), the shooter's target (`pick`), the
  turns (`turns` with the planned damage). `h3-battle-ai`: `predict.rs` (`plan`, the run, the melee
  prediction, the strike), `order::list`, `Walker::predicting` and the prediction's branches in
  the walker, its plan, scores, gain and fallbacks; `spells.rs`'s `beaten` and the attack bonus's
  0; the scan and the dragon run the predictions first. `POLICY_VERSION` 8.
- **Found and decided** (the user): the walker's candidates hold a hypnotized walker itself (no
  test in 0x421680); a Harpy without retaliation valued its own strike and planned a move and
  attack on itself, which the engine refuses. The predictions' draws moved the harvested
  property's stream onto it. The port leaves the walker out (BAT-STUB-10, BAT-Q58).
- **Kept apart** (documented): the planned damage is the decision's own (the original keeps it on
  the records until the next prediction, read only by a prediction, the context and its fill); a
  predicted walk, wait or fallback other than the attack after all is not built (it plans and
  draws nothing, ADR-0027 point 11); the run's own battle context (0x422893) is unused.
