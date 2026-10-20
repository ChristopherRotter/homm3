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
