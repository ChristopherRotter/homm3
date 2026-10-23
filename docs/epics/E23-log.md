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
  that T4b1 found was fixed first in a PR of its own (#373, `cross-lane`: `creature_cast_targets`
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
- **Cost** (measured 2026-10-22, before the stop): `computer-decisions` 477 885 393 instructions,
  +0.14 % (no re-set). Release, each decision's best of five (a local patch of `h3 bench-computer`,
  not kept), 400 battles each of seeds 4 to 8: 34.0 to 38.8 µs per decision on average, the slowest
  0.41 to 0.75 ms; `master` on the same sandbox 32.1 to 38.7 µs, 0.19 to 0.26 ms. An Ogre Mage
  whose context predicts six more of its stacks (seven a side, the enemy slower): 0.24 ms on Easy,
  0.49 ms above (`master` 0.03, 0.06); a Faerie Dragon alike: 0.16 to 0.72 ms (`master` 0.03 to
  0.08). Within the budget, the dragon's the nearest to it.
- **A blind check** (a fresh agent, the disassembly against the port): every function and every
  reader of the prediction flag agrees. Its notes: a walk that writes no action leaves the run
  reading stale action fields (no run reaches it; not modelled, §7.52); the pathfinder after a
  spell context (BAT-Q59, outside T4b2).
- **Stopped, then finished** (the user, 2026-10-22: T4b2 in a new session): the next session
  merged `master` (#373 merged by then) without a conflict and ran the full verify with the
  original data: every step passed (code 22188e09 on 42aeffd, 2026-10-22); `computer-decisions`
  477 815 842 instructions, +0.13 %, the other workloads +0.06 to +0.68 % (no re-set). #374.

## T4c — the Master Genie (2026-10-22, increment 5)

- **The user** (2026-10-22): T4c whole, no cut (the agent proposed seven parts, about 13 KB of the
  original's code); BAT-Q59 as slip 22, built with it.
- **The reading** (§7.53). 0x43C4A0: each spell 0x447EB0 accepts counted once, 0x43C330's mastery
  (slip 8 for every drawn spell), power and rounds 6, on R or over the side; the mean by integer
  division. The tables' seed 0x43BF20 never moves its record pointer (slip 23, found here). The
  second context (+0x48) for the other side, read only by the removals. The removal 0x439A80:
  spell order, each once, ended in turn on a copy (Disrupting Ray stays), the stack's mastery and
  rounds left, added in the second context for a harmful spell on the side, else subtracted. The
  pathfinder after the context: the threat records' last search (side 0, then 1), then only
  Hypnotize's (0x43A5DB) and Berserk's (0x445582) values search.
- **The port.** `h3-battle-ai`'s `spells.rs` became a root (the planned damage, the threat records
  once, the side's half and the other's when first asked) and views on a state (the battle or a
  removal's clone); the evaluators by the spell's data (`Kind`, 31 kinds) in `spells/boosts.rs`,
  `defenses.rs`, `harms.rs`, `removals.rs`, the tables in `spells/tables.rs`; `casts.rs`'s
  `genie`, the scan for both casters, `hold` (slip 22). `h3-battle`'s `computer` gained the
  estimates the evaluators need (the damage range and per creature, the exchange, the spell
  protection, the effect factor, `end_spell`, the berserk candidates with their search,
  `map_within`, `unslowed_speed`, `last_searched`). Slips 9 to 17, 22, 23 under BAT-QUIRK-25;
  BAT-AI-016; BAT-STUB-4 retired; `POLICY_VERSION` 9.
- **Found and decided:** slip 23 (the seed). Slip 22's map can name a hex the walker's own walk
  does not reach: the besieger's walk and the guard then return true with the action fields as
  they were (0x421E15, 0x4208DE); the port defends (BAT-STUB-11). With no record searching (a
  hypnotized walker among shooters) the port keeps the walker's own map.
- **Kept apart** (documented): the threat records and the tables computed when first read, the
  second context when a removal first asks it (ADR-0027 point 11); a removal's copy is a clone of
  the battle; the bolts', Dispel's and Clone's evaluators wait for the hero (T5).
- **A blind check** (a fresh agent, 0x43C4A0, 0x43BF20 with 0x43C040, 0x439A80 with Cure and
  Anti-Magic against the port): the Genie's mean and the seed's slip agree. Fixed: the fill read
  the seeded enemy from the entry a stronger strike had taken (0x43C0EE reads it once); the
  removal kept a permanent Acid Breath on its copy (0x444510 keeps only Disrupting Ray). Kept
  apart as before (T4b1): the threat records' dead enemies, which the seed then reads. Found: the
  removal's copy is no record of the battle (BAT-Q60; the user: slip 24, T4d; BAT-STUB-12).
- **Cost** (2026-10-22): `computer-decisions` 518 013 597 instructions, +8.55 %, re-set on
  purpose (the Genie's values; slip 22's map when a context-building caster casts nothing).
  Release, each decision's best of five (a local patch of `h3 bench-computer`, not kept), 400
  battles each of seeds 4 to 8: 30.5 to 35.0 µs per decision on average, the slowest 0.38 to
  0.98 ms, a Faerie Dragon's cast each time it was the slowest (its predictions and hexes, T4a and
  T4b2). Ten Master Genies with six other stacks against a hero's seven, three spells lasting on
  the side: 0.25 to 0.45 ms, on Magic Plains (the mass values) up to 0.61 ms. Within the budget,
  the dragon's the nearest to it.

## T4d — a removal's copy (2026-10-22, increment 5)

- **The user** (2026-10-22): T4d whole (the agent proposed the reading first, the build in a
  second session: the call closure showed 35 functions reading the battle manager).
- **The reading** (§7.54). The copy (0x437A00) keeps every field of its own, its side and record
  index included, so what an evaluator takes from the context at its index or from its threat
  record is the stack's. Pointer tests over the battle's records never meet it (Disrupting Ray
  0x438E11, Slow 0x439379) or leave out only it (Berserk's candidates 0x4455C9, Hypnotize's walk
  0x43A5FE). A hex names the battle's record by side and index (0x4E7170): the fit test 0x446C40
  compares those, so the copy fits its own hexes and its search is the stack's; the blocked-shooter
  test 0x469600 compares by pointer (0x442880), so a two-hex shooter's copy is blocked by its own
  record where either is berserk or the copy hypnotized. The map's start, the record's position, is
  reached and marked (+0x4A): the record is Berserk's nearest target (distance or cost 0) and one
  of Hypnotize's, struck by the copy (0x43A340). Nothing else walks the records with the stack.
  Found besides, port errors in both profiles: 0x448BD0 counts the turns on the record it is
  given (the copy's speed and shot), the port kept the battle's; 0x4B2DA0 marks a shooter's whole
  other side (0x4B2F18), the port's map lacked the marks (Hypnotize's value of a shooter).
- **The port.** `h3-battle`'s computer: `berserk_search_of_copy` (the stack itself a candidate at
  0; `control::searched` with `itself`), `exchange_with_itself` (the copy's exchange with the
  battle's record: aim's estimate and exchange read each party from its own state, `Party`,
  `estimate_between`, `damage_between`, `exchange_between`; `combat::melee_damage` gone,
  `shot_damage` test-only), `block_by_itself` (a blocked copy marked as one that cannot shoot,
  asked again after each ended spell, as `end_spell` recomputes the marks). `h3-battle-ai`: the
  removal with BAT-QUIRK-25 values on `View::on_copy` (the clone with the spells ended, the
  battle beside it in `lone`); Disrupting Ray, Slow, Berserk and Hypnotize read it
  (`berserk_attack_on_itself`); `within_a_turn` counts on a copy at its own speed; Hypnotize's
  shooter marks. BAT-STUB-12 retired; `POLICY_VERSION` 10; the AI lane's inbox has the version.
- **Kept apart** (documented): none new. The port's copy is the battle's clone with the spells
  ended (its own record) beside the battle (the records the evaluators walk); a removal is rare in
  the gated workload (no heroes: only the creatures' own spells last).
- **A blind check** (a fresh agent, the disassembly against the port, without the reading): the
  pointer tests, Berserk's and Hypnotize's targets, the self-block, the copy's exchange, the
  fit test and the turns agree. Found and built: the copy keeps the stack's morale and luck (only
  0x469750 recomputes them, after an action): an ended Mirth, Sorrow, Fortune or Misfortune still
  counts there (slip 24; `View::kept`). Found and recorded: Hypnotize's end on the copy adds the
  copy to the battle's aura lists, which keep it after it is freed (0x444689 → 0x43EA70; a
  state-corrupting bug, never modelled, BAT-QUIRK-26, for the user's confirmation). Noted as
  known: the bow's test asks the controller's hero (the inbox's F-r2b-2).
- **Cost** (2026-10-22): `computer-decisions` 518 107 736 instructions, +0.02 % (no re-set).
  Release, each decision's best of five (a local patch of `h3 bench-computer`, not kept), 400
  battles each of seeds 4 to 8: 34.7 to 38.0 µs per decision on average, the slowest 0.42 to
  0.79 ms (a Faerie Dragon, a Master Genie or an Ogre Mage: their contexts and predictions).
  Within the budget.

## T1a6 — the hero's cast choice and the retreat test (2026-10-22, before increment 6)

- **The user** (2026-10-22): T1a6 whole (the agent proposed two parts, the cast choice of about
  4.9 KB first, the retreat test with part A after it). At the same time: BAT-QUIRK-26's class
  confirmed (always fixed, no flag); BAT-Q57 answered as slip 25 (a task of its own, queued); the
  BAT-MAG-010 note left in the inbox.
- **Read** (`docs/formats/installation.md` §7.55): part H in full (about 5.9 KB: 0x43C800, 0x43BB20,
  0x436FB0, 0x437310, 0x43ACA0, 0x43B1E0, 0x43AF50, 0x43B8F0, 0x43AA60, 0x5A9360, 0x41E570,
  0x41E440, 0x477A00), of part A 0x433AA0 and 0x4339E0 in full and 0x4336C0 by its cases.
- **Found, the cast:** over the spells the hero has, can pay and may cast there, the value is the
  evaluator's (§7.46) or a case of its own, × 2.5 with mana for 7 casts, else × √(casts left),
  then drawn at 75 to 100 %; the best is cast only when its "now" (§7.44) says it is due at this
  activation, or, when the retreat test said yes, always, a damage spell only, valued in the
  desperate mode. With the enemy beaten and no own creature about to die, only the raises. Never
  cast: Quicksand, Land Mine, Force Field, Fire Wall, Berserk (aimed at a hex: its evaluator
  serves only the removals), Remove Obstacle. Earthquake only for a besieger; Teleport as the
  gain of the walker's real turn with the destination map over its own, twice per candidate; the
  summons by AI value.
- **Found, the retreat:** the wiki's terms all hold (never on Easy, half the Normal draws, never
  for the victory condition's hero or without a tavern town, a walled-out besieger always), then
  the hero's worth (artifacts 1000 or experience 2000), the predictions leaving no creature
  (retreat), the gold for the price and 2500, and the share of Fight Value under a threshold of
  0.16 to 0.22 less the difficulty's term and the attacker's 0.06, capped at 0.16 (an
  experience term that the integer division makes a step at 200 000). The adventure's inputs have
  no seam: BAT-Q61.
- **Found, a slip** (survey §4, slip 26, for T5's checkpoint): 0x422DA0 asks the retreat test
  between the context and the cast choice; when the test reaches its predictions, the cast's
  values read their planned damage (both sides' runs) and the threat records the predicted turns
  left, until Teleport's evaluation fills the records again.
- **Map:** 20 entries (16 new: 0x41E440, 0x4336C0, 0x4339E0, 0x433AA0, 0x436FB0, 0x437310,
  0x43ACA0, 0x43AF50, 0x43B1E0, 0x43B8F0, 0x477A00, 0x4E2840, 0x527640, 0x5A9360 and the tables
  0x63ABC0, 0x63ABF8; 4 updated: 0x41E570, 0x43C800, and 0x43BB20 and 0x43AA60 renamed
  `ai_hero_spell_value` and `ai_hero_teleport_value`); nine struct fields (the game's victory
  condition and towns, the player's towns, the hero's id and spell lists).
- **Checks:** a fresh agent re-read three findings blind, on the tool and the map as they were
  before the edits (one `grep` met §7.55's two lines on game +0x1F89C): the cast choice's skips,
  its value (the 2.5, the root of the casts left, the 75–100 % draw, the strict best, the cast
  only when "now" or retreating, the desperate mode); 0x43BB20's zero spells, its area hexes and
  its summons; the retreat test's every term in order with its constants and float steps. It
  agreed on all three; two details it added went into §7.55 (Summon Elemental's refusal is for
  an elemental of another kind; an unordered 0 / 0 counts as below the threshold).
- **The wiki** (Combat, "AI Retreat", read later in the session at the user's request): every
  term agrees word for word, the walled-out siege's four conditions too (the attacker, a fort;
  the drawbridge up, no wall section destroyed; no flyers, shooters or catapult; every defender
  inside); row 4b R2 dropped, R1 (the threshold) stays at risk.

## Slip 25 — the elementals' protection in `strict-fixed` (2026-10-22, before T5)

- **The user** (2026-10-22, BAT-Q57 at T1a6's start): the elementals that never cast are slip 25
  under BAT-QUIRK-25. At this task's start: the Storm and Ice Elementals, which shoot, weigh the
  cast before their shot too (the agent had proposed the walker's entry alone, where a shooter
  that can shoot never arrives: the original's turn sends it to its shot, 0x41F060, and none of
  the casters of its table 0x42136C shoots). No new reading.
- **Built:** `casts.rs`: with the flag off every `spell_caster` goes to the scan (BAT-AI-012), its
  value by the spell's own evaluator (`fixed`: 0 while the spell lasts on the stack, Bloodlust's
  0x438100 or the Protections' 0x4399A0 to 0x439A40, the mass version over the side), so the
  Ogre Mage's path is unchanged; the scan takes a `Scanner` (the battle, the side, its context)
  instead of the walker, and `before_shot` asks it for a shooter that casts a fixed spell, with
  the shot's value as the value so far. `h3-battle`'s `computer::valued_shot`: the shot with its
  target's value (the chosen pick's; the first pick's for a wall shot; 0 without a target).
  `shooter.rs` asks `before_shot` after the shot's choice (its own draws first). `POLICY_VERSION`
  11 (the AI lane's inbox).
- **Tests** (harvested, both profiles): ten shooting Storm Elementals beside nine Dendroid Soldiers
  against Silver Pegasi led by a hero with Lightning Bolt protect the dendroids when the scan's
  draw lets them (16 of 20 streams; the shot is worth something), never without the enemy hero
  (the Protection is worth nothing then) and never with the flag; protected, the dendroids are not
  protected again (two casts left: they shoot). Blocked by the pegasi, the elementals take the
  walker's entry: an attack worth nothing to them, so they cast for every stream (no draw).
- **The cost:** `computer-decisions` plays `base:faithful`, where no choice changed. The new
  decisions in release, best of five: a shooting elemental's 60 µs, an Ogre Mage's 146 µs, within
  the budget.

## T5a — the hero's cast: the entry, the choice, the evaluators, the damage spells, the summons (2026-10-22)

- **The user** (T5's checkpoint, 2026-10-22): T5 in two parts, the previous agent's cut (T5a; T5b
  the raises, Sacrifice, Earthquake and Teleport, BAT-STUB-14); BAT-Q61 option (c): the retreat
  test's answer is no until T6 (BAT-STUB-13), so slip 26 waits for T6; T5a in this session. At the
  budget's check: **a decision with a hero's cast may take up to 5 ms** (the side without a hero
  keeps 1 ms).
- **Re-read** (`docs/formats/installation.md` §7.56): 0x43C800's arithmetic and raises-only test,
  0x43A670's "now" (the active record, the side's other able records, the protective flag; Haste),
  0x436C60 (the desperate mode, the other records, the active stack's strike on E, the threat mask
  and the ranks, which the context's 0x41F140 keeps fresh), 0x43BB20's Dispel and stacks' cases
  (the polarity's sign), the bolts' 0x436F60, Dispel's 0x439BC0 and Clone's 0x43B2E0.
- **Built:** `hero.rs`: `cast` (0x422DA0; `unchecked` asks it before the stack's action),
  `choose` (0x43C800), `value` (0x43BB20) with `evaluated`, `one_target` (Magic Mirror's scaling),
  `now_on_own`, `now_on_enemy`, `chain`, `area`, `mass_damage`, `summon`, `dispel`; a `Targets`
  bitmap of the engine's targets. `spells.rs`: the kinds `Bolt`, `Dispel`, `Clone` with their
  evaluators (`harms.rs`, `removals.rs`, `boosts.rs`); the context's `beaten`, `at_risk`,
  `desperately`; `casts.rs`'s chain, area and payoff helpers take a `Scanner`. `h3-battle`'s
  `computer`: `HeroCast` and `hero_cast`, `casting_forbidden`, `cast_mirror_chance`,
  `dispel_ends` (the rule's own test, `effects::ends_spell`), `Context::desperately`; the area
  estimate tests the area before the spell's reach (the cheaper test first, no change of result).
  `POLICY_VERSION` 12 (the AI lane's inbox). Rules BAT-AI-017 (the choice) and BAT-AI-018 (the
  damage spells and summons).
- **Tests:** `h3-battle-ai`'s `tests/hero.rs` (the test ruleset gained Dispel, Clone, an area, a
  mass damage, a chain, a summon and a raise): the scaling by hand (2.5 times, √casts); a lone stack
  bolts the enemy worth most, accepted, and once cast the stack acts; no cast without a book, under
  the enemy's Orb, at the Ballista's activation or without a hero; a bolt not due now waits, a
  retreating hero casts it (Haste left out); the raises-only mode casts nothing; the area, mass,
  chain and summon cases (10 runners × 126); a summon worth nothing once the enemy is beaten, 1000
  a creature desperate; Dispel at Basic, Advanced (the enemy's blessed runners; nothing on either
  side) and Expert; Magic Mirror's scaling moves the bolt; the buffs and Clone. The harvested
  property (random heroes with half the spells) stays legal.
- **The cost:** release, heroes with every combat spell and 500 mana on both sides, 12 battles of
  7 random harvested stacks a side, each decision's best of five: 645 decisions, mean 0.43 ms,
  **the slowest 2.9 ms** (within the hero's 5 ms; before the target bitmap and the area test's
  order 3.7 ms). Half of the hero's cost is the context the original builds too (predictions, every
  stack's threat record, the tables). `computer-decisions` (no heroes) unchanged in its choices.
- **The blind check** (a fresh agent with the tool, the port and the addresses, no reading in the
  brief): 0x43C800, 0x43A670, 0x436C60 and the three evaluators agree; **0x43BB20's Dispel case
  has no return and falls into the stacks' case**, where Dispel (neutral) is weighed on both sides
  again: no change for one target, but at Expert the mass sums overwrite the own + other total
  with the other side's sum alone. Built as **slip 27** under BAT-QUIRK-25 (faithful: the other
  side's sum; strict-fixed: both; `dispel_by_its_mastery` in both profiles, the side's own Bless
  counting against it); §7.56 and the survey record it.

## T5b1 — the hero's raises, Sacrifice and Earthquake (2026-10-22)

- **The user** (T5b's checkpoint, 2026-10-22): T5b in two parts, the agent's proposed cut: T5b1 the
  raises, Sacrifice and Earthquake (evaluators on the context's and the battle's existing pieces),
  T5b2 Teleport (the destination map, the walker's turn with its value, its cost against the 5 ms).
- **Re-read** (`docs/formats/installation.md` §7.57): the candidate's setup in 0x43C800 (value 0,
  "now" 0, hexes −1; each case writes "now" with a strictly larger value only), 0x43ACA0 (the
  target test at either hex, `n`, `l`, the three-quarter rule, Resurrection below Advanced, the
  worth with the record's shot flag and no desperate worth, the doubling, "now"), 0x43B1E0 and
  0x43AF50 (the victims' test, the creature table's hit points, the worth less the victim's
  strength), 0x43B8F0 (the eight targets from 0x7FFF, the defenders inside, the Tent and the Cart
  left out, the walkers while no target is down, the shooters' share through the walls).
- **Two slips found** (the survey's "a slip found later joins the flag", BAT-Q55): **slip 28**,
  Sacrifice values the target's worth with the victim's shot flag (0x43B06F: `ecx` still holds
  the victim's record); **slip 29**, Earthquake asks 0x467510 from the shooter's position, the
  rule's halving (0x443C79) from the hex it turns to: a two-hex shooter's (the Medusas, the
  Ballista) may differ. Both built under BAT-QUIRK-25 (faithful as read; strict-fixed the target's
  own flag, the rule's hex).
- **Built:** `hero/specials.rs`: `raise`, `sacrifice`, `earthquake` (BAT-AI-019); `hero.rs`'s
  `Valued` carries the second target (Sacrifice's victim) into the cast, Teleport stays 0
  (BAT-STUB-14 narrowed to it). `spells.rs`: the context's own able strength, `raises_double`,
  `threat_enemy`. `h3-battle`: `siege::walls_between` (the line test past the shooter's own tests,
  `wall_penalty` now calls it), `computer::walls_halve_from_position` and `computer::walls_halve`.
  The policy takes the engine's targets for the raises (living or the corpse on top) and
  Sacrifice's victims (`second_targets`). `POLICY_VERSION` 13 (the AI lane's inbox).
- **Tests:** `h3-battle-ai`'s `tests/specials.rs` (the test ruleset gained a raise that lasts the
  battle below Advanced, Sacrifice tested as it, Earthquake): a raise of 2 bolted gogs by hand, not
  due while the gogs can act, cast at their turn; more than the lost only from three quarters or
  with the enemy beaten (then the 6 lost archers, due now); the battle-long raise worth nothing
  then, at Advanced as the other; Sacrifice's imps by hand in both profiles (the slip's value the
  smaller), the hydra's never above its strength, the cast accepted; Earthquake's walled-out gogs
  and the archers' share in both profiles, nothing for the defender, with the enemy beaten, at a
  fortless town or in the field, and the walkers' nothing once an Expert quake brought a target
  down. `h3-battle`'s `the_computer_s_wall_test_asks_from_a_shooter_s_position` (a Medusa's rear
  outside and front on a destroyed wall: halved from its position only).
- **The cost:** release, T5a's shape (not committed): heroes with every combat spell and 500 mana
  on both sides, 12 battles of 7 random harvested stacks a side, half at a walled town, each
  decision's best of five: 609 decisions, mean 0.36 ms, **the slowest 2.5 ms** (an attack after
  the hero weighed its spells; within the hero's 5 ms); the hero cast a raise 9 times, Sacrifice 4
  and Earthquake 3. `master` on the same harness: 662 decisions, mean 0.37 ms, the slowest 2.2 ms
  (the battles diverge once the new casts happen). `computer-decisions` (no heroes) unchanged in
  its choices.
- **The blind check** (a fresh agent with the tool, the port and the addresses, no reading in the
  brief): the candidate's setup, 0x43ACA0, 0x43B1E0 with 0x43AF50 and 0x43B8F0 agree with the
  port; it confirmed both slips at their sites (0x43B06F, 0x43BAB4). Its notes, none a
  difference: the port leaves out the battle-long raise up front where the original tests it per
  stack after `k`, tests the defenders inside before `W`, and names the turrets, the Tent and the
  Cart by trait where the original names creatures 149, 147 and 148 (the same in the packs); its
  `checked_div` and saturating conversions guard cases the original never reaches.

## T5b2a — the hero's Teleport (2026-10-22)

- **The user** (T5b2's checkpoint, 2026-10-22): T5b2 in two parts, the agent's proposed cut: T5b2a
  Teleport's value for every stack, a caster's cast entry in the turns of a stack that is not the
  active one standing in (BAT-STUB-14), T5b2b that entry (the engine lists cast targets only for
  the active stack; the original reads the stack's own record, the Faerie Dragon's spell drawn at
  its last turn). Asked whether a Teleport into the moat had been observed: batch 3g TV (Advanced
  Teleport onto the moat hex 61, no damage, the Swordsmen defended there that turn) and BAT-SIE-004
  say a teleported stack loses no turn, so slip 30's strict-fixed side takes no stops.
- **Re-read** (`docs/formats/installation.md` §7.58): 0x43AA60 in full ("now" only for the active
  or a disabled stack: the search for another able record finds R itself), 0x421F80's value,
  0x421680's map call (range 0 while disabled), its can-act test for the melee value and the gain,
  its flag-1 branch (the best from its plan's hex when the written value is at least 0, else
  Defend, before the cast entry), 0x4B2FF0 in full (it writes nothing else: the stops stay the last
  search's) and 0x436300's stop test.
- **Slips:** **slip 30** (new), the destination map's plan reads the stops of the pathfinder's last
  search (the reach's last enemy on Normal and above, else the threat records' last own stack,
  else the stack's own): a destination in the moat or on a seen pit counts two turns; built under
  BAT-QUIRK-25 (strict-fixed: none). **Slip 31** (new, found by the blind check), on the
  destination map the melee value's jousting run-up is the map's cost, 1, where a teleported stack
  strikes from where it stands; built under BAT-QUIRK-25 (strict-fixed: none). **Slip 5** (a
  two-hex enemy's position marked twice) changes no choice: the flag-1 choice reads a mark only at
  a position; recorded, nothing built.
- **Built:** `hero/teleport.rs` (BAT-AI-020): per own stack alive, not acted, that the cast can
  affect and that cannot shoot now, the two turns (`Walker::teleport_turns`, their draws from the
  hero's stream: `hero::value` takes the stream), the gain at a stack the engine takes, the
  destination from the engine's Teleport rule (`second_targets`). `walker.rs`: `Walker::turn` with
  its value, the flag-1 branch, the can-act test, the range-0 map of a disabled stack,
  `teleported` (the second turn reuses the first's scores, odds and threat records: ADR-0027 point
  11) and `last_search`; `scores::last_reached`, `gain::last_recorded` share their loops'
  predicates. `h3-battle`: `computer::destination_map`, `computer::search_stops`
  (`pathfinder::stops` taken out of the search), `Search::placed`; `siege::teleport_allowed` tests
  the mastery before the line (no result changes: Expert no longer draws the line per hex).
  `POLICY_VERSION` 14 (the AI lane's inbox). BAT-STUB-14 narrowed to the non-active caster's cast
  entry and the stops a cast entry's spell context leaves.
- **Tests:** `h3-battle-ai`'s `tests/teleport.rs` (the test ruleset gained Teleport): the gain of
  the gogs' teleported turn over their walk, cast and followed by their attack; a stack not active
  waits for its turn, shooters and stacks that acted are not asked; a blind stack moved now to block
  the enemy's archers by its plan alone (range 0, no melee value); the flag-1 turn's Defend; the
  destination map's positions, marks and slip 5; slip 30 at a Citadel's moat in both profiles
  (faithful: the moat hex two turns, the archers attacked; strict-fixed: the runners from the moat)
  and its tiers on Easy (the threat records' flyer: none; the stack's own: the moat but the
  bridge); slip 31 with a jousting test creature (the champion) in both profiles.
- **The cost:** release, heroes with every combat spell, 500 mana, power and knowledge 10 and each
  school at Basic to Expert on both sides, 40 battles of 7 random harvested stacks a side, half at
  a Citadel with a damaging moat, on Hard; each decision's best of five: 2 334 decisions, mean 0.38
  to 0.42 ms, **the slowest 3.5 to 4.3 ms** over four runs on Hard and Impossible (within the
  hero's 5 ms; before the mastery-first line test 0.45 and 4.4 ms); the hero cast Teleport 7 times.
  `master` on the same harness: 2 469 decisions, mean 0.26 ms, the slowest 2.1 ms; the branch with
  Teleport out of the books plays `master`'s battles command for command. In a profile Teleport is about 40 % of the
  hero's cost: the engine's destinations (the siege's line test per hex, `blocked_as` per hex) and
  each stack's hex scores (every enemy's reach searched again per stack). If the budget tightens:
  the enemies' reach maps cached across the stacks, the blocked hexes computed once per stack.
  `computer-decisions` (no heroes) unchanged in its choices.
- **The blind check** (a fresh agent with the tool, the port and the addresses, no reading in the
  brief): 0x43AA60, 0x421F80, the parts of 0x421680, 0x4B2FF0 and 0x436300's stop test agree with
  the port. It found slip 31 (built) and noted, none a difference that needs code: an unmarked
  enemy's stale cell cost read by a bound or speedless two-hex walker (the port reads 0; §7.58);
  0x436300's 100 turns for a stack of speed 0 (only a war machine, never asked); the first turn's
  spell context as the last search for the active stack too (BAT-STUB-14 widened); and the stops of
  a hypnotized stack, which the original takes from its real side and the engine's pathfinder from
  its controller (Q-0002's fix, §7.16: the slip-30 estimate follows the engine).
- **Checkpoint, then the PR** (the user, 2026-10-22, 23:15: stop here, push, no PR; 23:40: finish
  the PR after all). The scoped verify passed but for the coverage gate (one dead branch, fixed)
  and `bench-check`: `computer-decisions` (no heroes) +2.62 %, over its 2 %. Per-function callgrind
  profiles against `master` (in a worktree with its own target dir; the same 1 052 decisions)
  put all of it in `Search::run`: taking its stops' code out into `pathfinder::stops` cost every
  search about 5 % (a call or `#[inline(always)]` alike). The search keeps its own copy again
  (518.27 M Ir against `master`'s 518.17 M, +0.02 %); `computer::search_stops` calls
  `pathfinder::stops`, and `the_search_stops_are_the_searchs_own` holds the two together. No
  re-baseline.

## T5b2b — a caster's cast entry in Teleport's turns (2026-10-23)

- **The user** (T5b2b's start, 2026-10-23): whole, `cross-lane`, rather than the agent's proposed
  cut (T5b2b1 the raisers, the scan and the stops; T5b2b2 the dragon's spell with the state hash)
  or the dragon as a gap.
- **Read** (`docs/formats/installation.md` §7.59): 0x421280, 0x421000, 0x420D20 with 0x4476C0's
  cases and 0x420F00 read only their caster's record (casts left, creature, count, controller,
  +0x4E0); nothing in the computer's creature casts reads the active record (only "now" and the
  hero's cases do); 0x446C40 compares the active record only with its second argument, which the
  computer's searches leave 0. +0x4E0 is written only by the draw and the removal's copy and never
  initialised (the records live for the session): a stack holds its last activation's draw, a
  reused record its occupant's, a record that never drew an earlier battle's (not modelled: the
  engine's battles start afresh). After the entry only the walk searches (0x41F580: the guard and
  the step to safety through it).
- **Built:** `h3-battle`: `Casts::last` (each stack's last draw; battle state hash v52, its pinned
  test), `computer::drawn_spell`, `computer::creature_cast_targets_of` (the targets of any caster
  as if it were the active stack, a random caster's at its last draw; `casters::cast_with` shares
  `stack_cast`'s match), `Map::stops`. `h3-battle-ai`: the entry takes its walker's own targets and
  drawn spell; `Walker::teleport_turns` asks it for every stack (BAT-STUB-14 retired); with
  BAT-QUIRK-25 a cast entry's spell context in the first turn leaves its last search's stops
  (`Walker::after_map`, `Walker::tracking`) unless a walk searched after it, the destination map's
  stops when no reach and no threat record searched (slip 30 widened); `POLICY_VERSION` 15.
- **Replays** (`cross-lane`, the user's consent): the ten golden battle replays keep their
  commands (`duel-s1`, `field-s5` rehashed from their stored setups, the others re-recorded); the
  adventure lane's re-recorded with their options, the commands unchanged (its T42 replaced them
  meanwhile: the ten of `master` 49b19b8, nine moving checkpoint hashes taken inside a battle;
  `fixtures/README.md` says how).
- **Tests:** `h3-battle`'s `a_random_caster_keeps_its_last_draw_for_the_computer`; `h3-battle-ai`
  (the test ruleset gained the faerie, a random caster of the bolt): a waited Master Genie asked
  its entry at the gogs' turn (it casts on them, leaving Teleport to the gogs); a Faerie Dragon
  that drew nothing walks (Teleport's best, not now: no cast) and one round later casts its kept
  bolt (Teleport goes to the gogs, now); after the faerie's context the destination map takes the
  defender's gogs' stops (the moat, not the bridge), its own (the bridge too) only untracked, none
  in strict-fixed, and the genie's walk after its context leaves its own (a flyer: none).
- **The cost** (the harness rebuilt, not committed: 40 battles of 7 random harvested stacks a
  side, heroes with every combat spell, 500 mana, power and knowledge 10, each school Basic to
  Expert, half at a Citadel with a damaging moat; best of five): seeds 77, 5, 9 on Hard and
  Impossible, mean 0.41 to 0.44 ms, the slowest 3.8 to 5.2 ms (one run 7.6 ms, a sandbox burst);
  `master` on the same harness 0.40 to 0.45 ms and 4.1 to 5.3 ms. This harness is heavier than
  T5b2a's (`master`'s slowest was 3.5 to 4.3 ms there): the hero's 5 ms budget is at its edge on
  `master` already, T5b2b adds nothing measurable. Without Teleport the branch plays `master`'s
  battles command for command (2 191 decisions).
- **The blind check** (a fresh agent with the tool, the port and the addresses, no reading in the
  brief): agrees on what the entry reads of its caster, on +0x4E0 (its writers, no
  initialisation, the earlier battle's spell not modelled) and on the searches after the entry.
  It found **slip 32** (new, built under BAT-QUIRK-25): the Archangel's test (0x421000 → 0x5A3CC0
  with caster kind 1 → 0x5A8090's raise case, 0x5A846D to 0x5A84B5) takes the hit points from the
  active record, its count × 100 for an Archangel, else × 50; in Teleport's turns of another
  Archangel the active stack's (`computer::raise_test_takes`; strict-fixed: the raiser's own), and
  that the walk 0x41F580 searches only for a stack not bound with speed (0x41F58D to 0x41F5A7),
  where the port had cleared a cast entry's stops for a bound one too (fixed; Teleport may move a
  bound stack). Its other note, 0x447A80 taking the acting side (+0x132C0), is Teleport's side:
  nothing to build. The test ruleset gained the angel, a stack raiser; `h3-battle`'s
  `the_computer_tests_a_raise_with_another_stacks_hit_points`, `h3-battle-ai`'s
  `a_stack_raiser_s_test_takes_the_active_stack_s_hit_points_with_the_slip`.
- **Verify** (CI cannot run): the full local verify with the original data, all 16 steps green on
  97ca6b8 (merged with `master` 49b19b8); the client workspace's check green. Earlier runs failed
  in coverage (the walk's fall-through for a bound walker, which no test reaches: written without
  the branch) and in `deny`: RUSTSEC-2026-0385 (`ctutils` 0.4.3 under `h3-server`, published on
  2026-10-23, `master` alike); `Cargo.lock` takes `ctutils` 0.4.5 (the server lane's inbox).
  `computer-decisions` +0.04 %. PR #408 (`cross-lane`).

## T6 — the hero's other choices (2026-10-23)

**Decisions (the user, at the start).** T6 whole, no cut (the agent proposed three parts: the war
machines, the tactics phase, the retreat). The policy's First Aid Tent heals the reading's patient
(the most lost hit points of the top creature, the later stack at a tie); BAT-WAR-003 keeps the
uniform draw until BAT-Q51's batch row. The seam (BAT-Q61 option a, BAT-Q56 option b): the
retreat facts and the attacking hero's own ground now (`shared/e23-t6-seam`, ADR-0027 point 14),
CLT-Q4's and CLT-Q6's viewer fields when a full game's viewer needs them. BAT-Q33: one bad-morale
roll for every side until S-25's batch row (T7). Slip 26 joins BAT-QUIRK-25 by the standing rule
(BAT-Q55). Checkpoint after the Catapult (the user): a branch, no pull request.

**The seam** (`shared/e23-t6-seam`, the full verify green on its code
3eeac6949ee2ad7aae0ae9b56ae43f3bddd6a306): `RetreatFacts { experience, artifacts, tavern, wanted }`
per side and `attacker_ground: Option<ContentKey>` (`None`: an ordinary ground); nothing reads them
yet, the two battle-lane literals take the defaults (`cross-lane`).

**Done on `battle/e23-t6`** (stacked on the seam): the commanded Catapult (BAT-AI-021, §7.60):
`siege::aim_with` holds the original's aim at any Ballistics level, the battle's automatic aim its
level 0 with the battle's draw, `computer::catapult_aim` the policy's with its own stream;
`POLICY_VERSION` 16; the harvested test plays eight battles at a Fort (a missed gate shot lands
on a wall, so not every battle reaches the four standing walls).

**Done in the second session** (the user: the tent and the whole tactics phase, `cross-lane` for
the state hash): the tent (BAT-AI-022, `h3_battle_ai::tent`) heals the patient of its side with the
most lost top hit points, the later record at a tie; the battle's automatic tent keeps its draw.
The tactics phase (BAT-AI-023, `tactics.rs`): the battle counts the phase's rounds for the
computer (`TacticsRounds`: the completed rounds, a placement in the running and the last one;
`BattleState.tactics_rounds`, outside `TacticsPhase`, so neither the observation nor the wire
moves; battle state hash v53, the ten golden and the adventure lane's replays rehashed with their
commands); the policy gives `EndTactics` at the first turn after a round without a placement or
after three. `computer::wait_phase` answers in the phase whether the active stack has waited (the
original's pick sets +0x13DE4 there too). The pathfinder's `Query.tactics` (0x4B34DE: range and
turn's speed 1000, before the speed-0 and Bind test) for `computer::map`, `map_within`, `path` and
`turn_map` while the phase runs: a stop's step costs the rest of 1000 and is reached without being
queued (the queue's buckets end at 255); a `Reach` search to it ends on it after the loop. The
shooters: 0x422060 over `Map::position` (a flyer's hexes flown over left out: no shooter of the
game flies). The walkers: no cast entry, the attack a walk, the walk the whole path without the
score test, its end in the zone (`tactics::in_zone`, with the engine's footprint rule, BAT-STUB-15);
the guard's turns 1; the flag-1 records' turns at the record's speed (`gain::Record.turns`; the run-up
only within the speed). Defend becomes Wait in the phase; a walker below Hard (not a human side)
that has waited gives Wait at once; on Hard a walker that defended is asked again (BAT-STUB-16);
`checked`'s fallback in the phase is Wait. BAT-STUB-5 now holds only the retreat test.
`POLICY_VERSION` 17. The battle lane's scoped verify on 46e5b5e: every step but `check-lane`
(stacked on the seam, `cross-lane`); `computer-decisions` (no heroes) within its gate.

**The design for the retreat** (from §7.60; for the next session):
- *The retreat* (BAT-AI-024): the test (§7.55, §7.60) at the hero's cast (when the hero may cast)
  and again at the stack's action outside the phase (not for the Catapult's or the tent's turns;
  the Ballista's yes); a yes is `Retreat` (the computer never surrenders, 0x47815A). The battle
  keeps the facts and the attacker's ground (hashed: still v53 while T6's branch is unmerged; scenarios and the replay file name them:
  the replay format's bump is the AI lane's, `cross-lane`); the price as 0x477A00 (the temporary
  creatures out, halved, × the hero's factor in `f32`); the predictions with flag 1 (the side's run
  and then the other side's from where it ended); slip 26 in `faithful`: the cast choice reads both
  runs' planned damage and, until Teleport's evaluation, each side's last predicted walker's flag-1
  records. Slip 6 (BAT-STUB-7) in `faithful`: an attacking hero's spells at the mastery of
  `attacker_ground`.

**Done in the third session** (the user: the retreat test with slips 26 and 6 in one session, no
cut; `cross-lane` again for the replay format and the rehash). *The facts*: `BattleState` keeps
`retreat` and `attacker_ground` (resolved; an unknown key is `UnknownBattlefield`) as hashed state
(still v53), not observed; `computer::retreat_facts`, `attacker_ground`, `funds`. Scenarios
(`attacker_retreat`, `defender_retreat` tables, `attacker_ground`) and the replay file name them
(`RetreatEntry`, each field omitted at 0 or false; replay format v30 in `h3-sim`); the ten golden
replays rehashed with their commands, the adventure lane's ten re-recorded with their options
(only hash lines moved; `duel-s2-d7` none). *The test* (BAT-AI-024, `retreat.rs`; re-read: §7.60's
last paragraph): in its order, the Normal draw after the hero test; `state.may_retreat` stands
for the Shackles and a town's defender without the Escape Tunnel; the walled-out siege
(`computer::walled_out`: an attacker's living Catapult or Cyclops keeps it out, as the original's
flag `0x20`); the price (`computer::retreat_price`, the reading's: temporary creatures and
created stacks out, the surrender share before the discount, 0 for a ruleset that prices none;
`flee::surrender_price` is unchanged, BAT-Q28 row 7); the predictions with flag 1
(`predict::plan_both`: the other side's run from the stack that ended the first); the strengths
over the first 20 stacks of each side with the turrets, `_ftol` of the products; the threshold as
`f32` steps; the quotient of two `f32`s. Asked in `hero::cast` between the context and the choice,
and in `unchecked` after the Catapult's and the tent's turns, before the shot or walk. *Slip 26*:
`SpellContext::after_retreat` takes the test's planned damage and each side's last overwrite of
the threat records (`predict::Overwrite`: a walker's records of its side via `gain::records`, now
with each record's cost; a shot's of the other side via `computer::threat`, which now passes over
spent stacks in a predicting context); Teleport's walkers refill them (`refill`); the threat
tables read the context's (`View::built_on`), "the enemy beaten" of both halves is fixed at the
build. *Slip 6*: `aim::ground`. *Found*: slip 33 (§7.60's last paragraph; BAT-STUB-17, not built):
Teleport's caster turns' spell contexts set the planned damage the hero's later spells read.
`POLICY_VERSION` 18. The default facts never retreat, so no golden replay or existing test changed
its commands; the Normal draw now comes before every hero cast's choice (the streams moved, the
tests' choices held). The property over random battles draws retreat facts and funds.
*Verify* (the third session): the battle lane's scoped verify on 54dcb37 failed `check-lane`
(stacked on the seam, `cross-lane`) and coverage (lines and regions the random battles had reached
only by chance: fixed tests in bb7a295, a bound walker's choice among them); the full verify with
the original data green on bb7a295 (`verify-once: verified (sandbox) code
b78ab748a2a372885542109d125928082f30ea7d on bb7a295c7f948e8c7b42c4597076579d4e22bbec`), the
clients' check green (`verify-once: clients verified (sandbox) code
61a3bebb5aa8eaa98b1077bc7e22aef64db1668b on bb7a295c7f948e8c7b42c4597076579d4e22bbec`; SDL's build
packages installed first). A trial merge with `master` (23e7924) conflicts in the AI and adventure
queues and the lane HANDOFF; #418 moved the twenty replays' `ruleset_hash` (core ruleset hash
v48): at the pull request, after the seam's, take `master`'s replays and rehash them on the merged
code.
*At the pull requests* (the user, 2026-10-23): `master` (7bd83d5, #420) merged into the seam
(#422, no conflict; `Cargo.lock`: tokio 1.54.1 for RUSTSEC-2026-0386, published that day) and the
seam into this branch (the queues and the HANDOFF resolved by hand); the twenty replays verify as
merged (#418's `ruleset_hash` lines and this branch's hash lines touch different lines and the
merged ruleset plays them alike).
The user squash-merged #422; `master` (328b808) merged into this branch again (the two battle-lane
literals kept this branch's), with the AI lane's System seat (#421, `h3_battle_ai::choose` behind
`Policy::System`): the full verify with the original data green on 8c37c73 (`verify-once: verified
(sandbox) code ef5e62438e5d4a95d310840954860cb95b984300`), recorded in #424.


## T7 — the batches 4a, 4b and 4c (2026-10-23)

**Decisions (the user, at the start).** T7 is large (slip 33, the survey's rows, the rules they
decide, the closure); the agent proposed five parts. The user: "design and run batches, no other
seams yet": this session designs and plays the survey's rows (§3) and S-25; the rest of T7 is
cut with the user later.

**Tools.** `experiments/agent/ai.py` plays a battle on the stepped clock against the original's
computer (`experiments/README.md`, "The original's computer played against by an agent"): one
map per encounter, Red by the recipe's plan (else Defend), the computer's side on its own;
`timeline.jsonl` holds every change (the battle manager's action +0x3C to +0x48, the turn
flags, each stack's hex, count, flags, spells, a Faerie Dragon's drawn spell, the walls), the
opening the setup's human bytes, the players' gold and the blocked hexes; forced rolls (S-25),
seeds (the clock idled before Red's first command: the fidgets draw from the rules' generator,
seeded per battle from its tile at 0x463A08), the aimed Catapult, auto combat. `ai_turns.py`
lists the actions. h3-experiment: `owner = "tan"` (a computer-only player: hot seat with Red and
Blue leaves it to the computer) and `[[map.town]]` (Tan's Tavern town for the retreat test).
Batch 4a holds the neutral rows that fit one map (eight heroes a player), 4c the rest, 4b a
computer player's hero. The port replayed in the engine (a throwaway test of the same armies,
no obstacles, or a siege, which has none) checked set-ups before they were played: it showed
W1's first design sallying after a breach, C4's first design without a walker's value (no skip
draw), and found C4's Easy set-up and the floor's contrast.

**Rows** (the results files hold the numbers):
- *S-25 (BAT-Q33)*: with every bad-morale draw forced to fall, 28 of 105 neutral picks (LO) and
  14 of 60 picks of Tan's stacks (LC) were let off, a human side's none (27 of 27): the second
  draw for a side whose +0x54A4 is 0, as read (the setup's bytes [1, 0]); the rule waits.
- *W1*: a Citadel's garrison walker holding its walls defends on Normal, waits on Hard (the
  waited bit at Red's slower Dwarves' turn, four rounds each). The first design's 10 Peasants
  died to the keep's first arrow; at the frozen clock the computer's Defend passes between two
  reads, so a slower Red stack's turn is where its flags show.
- *S4 (slip 2)*: behind a destroyed mid-upper wall the garrison holds (three runs, to round 9),
  behind a destroyed mid-lower wall it sallies at once (and in W1's first runs); Expert
  Ballistics' second shot broke a second wall in the first runs (Basic since).
- *C1*: no elemental's cast in five rounds. *C2*: a cast at every dragon turn (8 of 8), the
  area spells at the free hex between two stacks (4 of 4), the bolts at the first stack of the
  best kill value. *C3*: ten dead Peasants left for an attack (rounds 2 to 4), two of three dead
  Champions raised (round 2).
- *C4*: on Normal with Red's Archangels alone, a cast at all 6 turns at the 10-Champion stacks in
  the order records 5, 4, 3, as the port (the walker keeps no candidate: value 0, no draw); on
  Easy (a computer's walker keeps its negative candidate) 3 turns of 21 without a cast at one
  field across seeds, the casts at the 10-Champion stacks, the later record first. The floor
  (C4f3, C4f2 in batch 4c): the lone Champion blessed when the casters are weaker (1344 against
  1800), never when stronger (2016).
- *W3*: a Gold Dragon between two equal stacks took each across ten seeds (6 and 4).
- *H1*: the computer's Archers (Expert Tactics, the only stack) went in the tactics phase from
  hex 100 to hex 15, the port to hex 9. The original's neighbour table (battle manager +0x13468,
  read from memory in a running battle) has no neighbour in the side columns 0 and 16 for hexes
  of columns 1 to 15: hex 15 has two (32, 14), hex 9 four, so the first hex with the fewest is
  15. The reading's rule holds; the port's `hexes::neighbor` (the engine's field geometry) is
  wrong at eight call sites: the battle queue's inbox.
- *H3*: Bless at the Archers' own activation, none at the Wolf Riders' before it. *H4*: none of
  six never-cast spells in four rounds.
- *R1*: Hard, Tan's hero with 2000 experience and a Tavern town, 10 000 gold: a retreat at its
  first activation at 3312 / 23220 = 0.143 (R1a); at 0.172 a fight (R1b), and at its second
  stack's activation, the first having acted (× 1.0 instead of 1.2), 3726 / 23634 = 0.158 below
  0.16: a retreat in the same round, the reading's steps 7 to 9 exactly.
- *SC (S1, S2)*, Red in auto combat at a Castle's Fort with 100 Archers inside: Red's Catapult
  breached the upper wall before the Cyclops' first turn, so the locked-out wall shot and the
  walk to the row's wall were not reached; afterwards the Cyclops shot the Archers and the
  Pikemen went through the breach, as read.
- *TP, TP2, TP3 (BAT-Q51)*: in TP only the Swordsmen were ever hurt (the shooters' target): the
  tent healed them at every turn, both rules alike. TP2 hurt both with Red's own spell (Armageddon
  destroyed the tent with its first cast; Death Ripple spares it): one turn, the most hurt healed,
  then the golems destroyed the tent. TP3 outside W1's held Citadel: the most hurt healed at 6 of 6
  turns (7 of 7 with TP2; 0.8 % under a uniform draw). The idle tent's Defend left no trace.
- Not staged: W2, W4, W5, P1, C5, S3, H2 (survey §3), BAT-Q58's Harpy.

## T7 — the rest (2026-10-23)

**Decisions (the user).** All four parts the batches' session proposed in this one session (the
neighbours, BAT-Q33 with BAT-WAR-003, slip 33, the benchmarks and docs). The rules' breath into a
side column (BAT-Q62): the engine unchanged, an experiment first, a fix-optional quirk if it
holds. The let-off: "an unfair advantage for no good reason": a fix-optional quirk; the agent
judged it outside BAT-QUIRK-25 (a rule every side's stacks meet, not one of the computer's
estimates, ADR-0027 point 9), so it is BAT-QUIRK-27 of its own, its registration `cross-lane`
(#436).

- **The neighbours (H1).** 0x4640A0 fills the battle manager's table +0x13468 (read for E17 M3
  T6c-1, re-read): the field's geometry with every neighbour in column 0 or 16 or off the field
  -1, a side column's hex only its inner three. `hexes::table_neighbor` is that filter;
  `hexes::neighbor`, `stack_neighbor` and the hex scores' beside hex (0x420523, which reads the
  table directly) go through it. The guard hex's touch test (0x4466C0 → 0x4674C0, `b` in `a`'s
  row) needs nothing: a walker's hexes never lie in a side column. Tests: the H1 row (the
  defender's Archers on hex 15 stay, from hex 100 they go back to 15; the field's neighbours sent
  them to 9), the table's own (hexes 15, 9, 34, 17, a machine's ring), and the shooter placement,
  the area threat and the breath line at the table (no side-column hex). **The rules' uses**
  (the table's readers: 0x4674C0, 0x469612, 0x523C21, 0x523D90, 0x523DF0 and two unnamed routines of the pathfinder's and the neighbours'; 0x523D90's callers):
  a stack's adjacency, the pathfinder's marks and the shooter-blocked test come out the same,
  since every column-1 or column-15 neighbour of a war machine's rear hex also touches its front
  hex; the breath's second hex (melee_strike 0x441739) does not: BAT-Q62.
- **BAT-Q33 (BAT-QUIRK-27).** `morale::bad_morale` draws `range_inclusive(1, 4)` after a freeze
  for a stack whose own side (its record array, 0x464B40's argument: a hypnotized stack's real
  side) has no human player, with the flag; a 1 lets it off. The flag at position 33 in
  `h3-content`, on in the faithful profiles (`packs/base`, `packs/synthetic`, `h3-harvest`'s
  text, the pack tests' anchors): faithful core ruleset hashes synthetic `0x3156c56f2211ba02` →
  `0xa72ea0e9930772bc`, base `0x1ac6c50d158057de` → `0x0c7e01b4dffeba0c`; the twenty replays'
  headers patched in #436. With the rule `full-s3-a242` (both sides without a human player) let
  off a stack at command 376: re-recorded from its spec, the defender still wins, in 434
  commands. The defaults (`human` false) give every side of a game's battle the let-off until the
  adventure lane sets `human` (their inbox); the AI lane's batch plays both sides as human
  players'.
- **BAT-WAR-003.** `war::patient` (the most hit points lost on the top creature, the later stack
  at a tie) is the automatic tent's patient and, through `computer::tent_patient`, the policy's
  commanded tent's (BAT-AI-022): one implementation, no draw. An idle tent's Defend changes
  nothing for a war machine (0x479722) and is not modelled.
- **Slip 33.** A cast entry that builds a spell context (the scan's `SpellContext::new`, the
  Faerie Dragon's `predict::plan`) hands its planned damage to the walker in Teleport's turns
  (`Walker::planned_by_entry`, only while tracking); `Walker::teleport_turns` returns the last
  (`TeleportTurns`); `teleport` keeps the last over its stacks and, with the slips, gives it to the
  hero's context (`SpellContext::after_teleport`, a `OnceCell`: Teleport is valued once per
  choice); `planned_now` reads it before the retreat test's (slip 26) and the context's own. The
  threat records need nothing: Teleport's refill restores them. Tests: the turns give planned
  damage only for a caster and only with the slips, and the hero's valuation leaves the last
  caster's; `after_teleport` replaces the planned damage, the at-risk test reads it, the first
  holds. BAT-STUB-17 retired. `POLICY_VERSION` 19.
- **The cost.** `h3 bench-computer --heroes 1` (new: each side a hero with every combat spell at
  a mastery drawn per school, 500 mana, power and knowledge 10, Citadels with a damaging moat),
  40 battles, five runs over seeds 1, 5, 9: the slowest decision 3.0 to 4.1 ms (the first run
  7.3 ms, cold), the mean 0.24 to 0.33 ms: within the hero's 5 ms. Without heroes the mean 30 µs,
  the slowest 164 to 235 µs (`master`: 54 µs and 669 µs over other battles). Re-set on purpose:
  `computer-decisions` −9.89 % (1 482 decisions against `master`'s 1 052, about a third fewer
  instructions each), `random-rollouts-full` +6.38 % (7 768 steps against 7 279: the let-off
  makes the battles longer; about as many instructions per step).
- **The AI lane's tables** (their inbox's instructions; `cross-lane`): #436 only `engine.core`;
  the battle PR version 19: the system actor beats Greedy 0.655 (0.653 before) and Aggressor
  0.681 (0.681), and attacks best on the neutral set (value +0.285, Greedy +0.224).
- **Left for E23's closure:** exit criterion 4's gated batch workload with `System` seats (the AI
  lane's `benches/ai/`, their inbox), criterion 6 on the user's Windows machine, `BACKLOG.md`
  (`shared/`). BAT-Q62's row with the next batch.

## T7 — the closure (2026-10-24)

**Decision (the user, 2026-10-24).** Close E23 now that the AI lane's part is done. The lane's
next session works off the inbox's leftovers; only then does it open E11 (refined the same night,
Q58).

The exit criteria, each with its record:

1. **A legal command, the same for the same state and stream:** the property
   `the_choice_is_legal_and_the_same_for_the_same_stream` (`h3-battle-ai`'s tests: random
   battles of both sides, every difficulty, with and without heroes, on the field and at the town
   with every fort level, both profiles).
2. **Every branch of T1a's map modelled or recorded as a gap:** the survey
   (`docs/rules/surveys/E23-battle-ai.md`) and the stubs left with their reasons: BAT-STUB-8 (the
   dragon's free-hex targets, BAT-Q32 (1)), BAT-STUB-10 (a hypnotized walker on itself, BAT-Q58),
   BAT-STUB-11 (the action fields a failed walk leaves, slip 22), BAT-STUB-15 and BAT-STUB-16
   (the tactics phase's zone test and Defend). None blocks play.
3. **Readings at risk against played rows:** batches 4a to 4c, every played row as read; H1
   resolved in the port; S1, S2 not reached and W2, W4, W5, P1, C5, S3, H2 not staged (above),
   left with the readings they rest on. BAT-Q62 waits for a row of the next batch.
4. **The cost:** within T1b's budget (T7's rest: the slowest hero decision 3.0 to 4.1 ms of 5;
   the mean without heroes 30 µs); the batch benchmark with `System` seats is the AI lane's gated
   workload `batch-system-neutral-v1` (#442, re-baselined on `POLICY_VERSION` 19).
5. **The function in the other lanes' inboxes:** since T2a; the AI lane's `Policy::System` plays
   it, the adventure's neutral seat calls it.
6. **`cargo xtask verify` on Linux and Windows with the original data:** the full local Linux
   verify, 16 of 16 steps, on #441 (CI cannot start: billing); the user's Windows verify passed on
   2026-10-23.

Left for the lane (its inbox and OPEN-QUESTIONS): the AI lane's turret-shot preview (E06 T6a),
BAT-Q62's row, BAT-Q28, BAT-Q32, BAT-Q35, BAT-Q58 and the stubs above. `BACKLOG.md`:
`shared/e23-closure`.

