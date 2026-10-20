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
