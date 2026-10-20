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
