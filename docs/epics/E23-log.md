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
