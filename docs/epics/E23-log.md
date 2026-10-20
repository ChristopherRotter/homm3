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
