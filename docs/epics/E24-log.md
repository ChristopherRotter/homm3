# E24 log: done tasks in full

The full record of E24's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file
keeps one line per done task; read the matching section here when a task builds on an earlier
one or a decision is questioned.

## Done

### T0 — E24 refined with the user (2026-10-23)

One session, the user present; branch `map-client/e24-t0` (docs only, `cross-lane` for the
BACKLOG row, the user's consent). The agent read the lane's docs, `clients/README.md`, E17's epic
file and design notes §6.3 (the harness on the adventure map), `h3-map`'s raw model and import,
the executable map's adventure entries, and the adventure lane's state; it proposed the
milestones and M1's tasks, and the user took every recommendation:

- **M1's scope:** the map view and the screen's frame; the panels in M3 (over the whole
  adventure screen in M1).
- **The order:** the map moving (M2) before the panels (M3).
- **Test maps:** this lane adds what it needs to `h3_map::writer` in additive `cross-lane` pull
  requests (over the adventure lane's inbox or a writer of the lane's own).
- **The string catalogue** stays common (T9c's call, confirmed).
- **The BACKLOG row** in this pull request, `cross-lane` (over a `shared/` one).

What shaped the proposal (the epic file's "How the viewer sees a map" keeps the result):

- `h3_map::import` keeps no pictures and imports 5 of the 160 shipped maps whole; `RawMap` keeps
  every tile's terrain, river and road with their pictures and flags (`model::Tile`) and every
  template's DEF, masks, class, group and priority (`model::objects::Template`): the static art
  comes from the raw map, the live part from a second input.
- The writer has terrain patches (whole rectangles, so the transitions are whatever pictures it
  writes), straight roads (pictures 15, 12, 13), heroes, monsters, piles, grounds, plain objects
  of `PLAIN_OBJECTS`' five classes (the Cartographers, three kinds of trees, the Border Gates),
  towns and an underground level; no rivers, no free tile pictures, no other templates.
- The executable map already names the per-tile draw lists (0x505B20, 0x505230, 0x505060) and the
  shadow pass (0x47D930); the harness knows the view's geometry and its top-left in memory.
- A shipped map's random objects are drawn at the game's start in real time (E17 design §6.3).

Inbox notes written: the adventure lane's (the writer's additions to come; the object identity
M4 needs), the client lane's (the common crate M3 needs).

### M1 T1a — code reading: the view's tiles and the moment (2026-10-23)

One session, branch `map-client/e24-t1a` (docs only: the design notes §1, `docs/exe/map.toml` and
`structs.toml`, MCL-Q1, an inbox note). The user kept the task whole when asked for a seam
(the agent had proposed the moment as a second part).

- **Path:** the adventure manager's DEF loads (0x406FD0) led to the view draw 0x40F3F0 and its
  eleven passes; read in full: the terrain and edge, river, road and unexplored-tile passes, the
  unexplored tiles' mask and table, the run-shadow drawer 0x47E820 (formerly
  `draw_frame_variant`, inferred), the corner gems, the tick 0x4ED490 with its palette ranges, the
  idle step's animation counter; the object and path passes only for their place in the order and
  their frame rule (T1b reads them).
- **Findings that change earlier text:** a tile's left edge is at 32 (x − vx), not 8 + 32 (x − vx)
  (E17 design §6.3 had it from clicks; the epic file and `structs.toml` corrected); the view has no
  pixel scrolling (+0xF4, +0xF8 stay 0); the four corner gems are drawn by the view, after the
  shroud, in the viewing player's colour; objects get a random animation phase per object at load.
- **VCMI 1.7.5** (§1.7): agrees on the tile layers, mirrors, the road's offset, the edge's
  corners and sides, which hidden tiles get a piece, the palette ranges and direction; differs on
  the place and clip (one pixel), the far edge pattern for odd negative coordinates, the full
  piece's frame, 16 of 256 edge-piece masks and the parity variants, the shadows' rounding, the
  palette step (180 ms continuous against 200 ms polled) and the objects' phase. The edge-piece
  comparison ran VCMI's table through the original's masks (a throwaway script).
- **Agent decision (provisional):** a map whose picture lies past its DEF's frames is refused by
  the viewer (the original reads past the table).
- **Questions:** MCL-Q1 (the bottom edge's x test against the height; harmless). The mouse's
  possible 8-pixel offset stays an open point for M3 (§1.8) until a capture shows it.
- **Verify:** `cargo xtask check-exe-map`, `check-docs` and `check-lane` passed; `scripts/exe.py
  check` agrees with the executable (44 entries added, one renamed and read).
