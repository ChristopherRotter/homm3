# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client, **M1: stills of the map view** (refined 2026-10-23, T0;
`docs/epics/E24-map-client.md`: the decisions, how the viewer sees a map, the milestones). Code
readings in the sandbox (`docs/exe/README.md`; every address cited needs a map entry), their
detail in `docs/epics/E24-design.md`; captures by the harness (`clients/capture/README.md`), never
in git. Every oddity of the original a reading or a capture finds is an `MCL-Q<n>` in the same
task (the HANDOFF's autonomy contract).

### E24 M1 T1b — code reading: the objects' draw order and shadows (research item 9)

- **Scope:** how the original orders and draws the objects over the view: how a tile's draw list
  is built (0x505B20, 0x505230: the order of entries by the template's priority, its layer per
  tile from 0x505060, the anchor, flat templates), how the drawing walks the tiles and their
  lists (an object drawn once or per tile, clipped; objects reaching past the map's top and
  left), the passes (the shadows by 0x47D930 before the bodies, or per object), where heroes,
  boats and the selected hero's path fall in that order. VCMI 1.7.5's order compared; the user's
  note (neither their engine nor VCMI got it exactly right) answered by naming where VCMI
  differs. Result: `E24-design.md` §2; the frames that confirm it come in T4. The order inside a
  tile's list answers the adventure lane's open point (its inbox, 2026-10-13, T14a): a note there.
- **Acceptance:** the order written as an algorithm a test can follow, with its addresses; the
  differences from VCMI listed with a situation each that would show them (T4's situations).
- **Depends on:** nothing (T1a's passes, `E24-design.md` §1.1, frame it). **Status:** ready.

### E24 M1 T2 — the crate, the map's asset tables and the layout

- **Scope:** `clients/crates/h3-map-view` (a pure library with the purity test as `h3-view`'s
  `tests/purity.rs`, the adapters in its binary; `clients/Cargo.toml`'s members, a common file:
  an additive change); the scene's input types (the static art from `h3_map`'s `RawMap`, the
  live part as its own type: the objects standing, a random object's drawn type, owners, heroes,
  the explored tiles, the moment; *provisional*); the map's asset tables in `clients/map/assets/`
  (terrain, river and road DEFs by code, the edge, the unexplored tiles' pieces, from T1a; the
  heroes' and flags' DEFs come with T5) and the adventure screen's layout
  `clients/map/layouts/adventure.toml` (the frame picture, the view's origin and size: an HD
  layout needs both in the format); a `gate` command checking the tables' names against the
  archives (`h3-assets`' checks) and T1a's open points (`E24-design.md` §1.8: the frame counts,
  runs in terrain frames). The checks of `clients/scripts/check.sh` cover the new crate
  (100 % on its library).
- **Acceptance:** the tables and the layout load (pure loaders, tested on synthetic data); the
  gate finds and decodes every name on the user's files; `check.sh` passes.
- **Depends on:** T1a (done: `E24-design.md` §1, the fog table among the tables). **Status:** ready.

### E24 M1 T3a — the writer's additions (`cross-lane`)

- **Scope:** what T3b's to T5's situations need of `h3_map::writer` (the adventure lane's crate),
  added without changing what it writes today (the user's consent, 2026-10-23): rivers, a tile's
  terrain, river and road pictures and flags as given (so transitions can be written exactly),
  objects of any template (any class `Objects.txt` lists, without a body or with an empty one).
  Its own small pull request, label `cross-lane`, with the adventure lane's tests and a note in
  its inbox; `cargo xtask verify --scope adventure` without a failing step.
- **Acceptance:** the writer's tests cover the additions; every map the writer wrote before is
  byte for byte unchanged (the fixtures and the replays pass as they are).
- **Depends on:** T1a (done), T1b (what the situations need). **Status:** waiting (T1b).

### E24 M1 T3b — the tiles drawn and the first exact diff

- **Scope:** `h3-map-view render <h3m> --view x,y,z [--at <ms>]`: the adventure screen's frame
  and the view's terrain, rivers, roads, edge and unexplored tiles (objects not yet); the harness
  side (an additive change: `h3adv.py` or a new script loads a map in hot seat, sets the view,
  parks the mouse, dumps at a known game time); whether the virtual clock drives the adventure
  map's animations, and if not the moment read from memory; the first situations (the writer's
  maps of terrain transitions, roads and rivers, their heroes outside the view); the comparison
  masks the panels and whatever the view shows that T4 and T5 draw.
- **Acceptance:** each situation's still matches the original pixel for pixel outside its masks,
  or the differences are listed with their cause.
- **Depends on:** T2, T3a. **Status:** waiting (T2, T3a).

### E24 M1 T4 — the objects and their shadows in the original's order

- **Scope:** the objects drawn as T1b read (the draw lists, the passes, the shadows by the
  compositor's shadow drawer, objects past the map's edges); the situations T1b named (the
  writer's maps of overlapping objects, each showing one rule or one difference from VCMI);
  shipped maps at chosen views, their random objects' drawn types read from the original's memory
  (`h3mem.py`, additive) into the live part.
- **Acceptance:** the situations and the shipped maps' views match pixel for pixel outside the
  masks of what T5 draws; research item 9 closed in the design notes (the reading and its frames).
- **Depends on:** T1b, T3b. **Status:** waiting (T1b, T3b).

### E24 M1 T5 — heroes, flags, owners and the explored tiles

- **Scope:** a reading where T1a and T1b did not cover it (a hero's map DEF by class, its group by
  facing, its frame at rest, its flag by colour, boats, an owned object's colours: mines,
  dwellings, towns), the tables' entries; the unexplored tiles from the original's explored
  tiles (`h3mem.py`) in the live part; situations of heroes of every class facing each way and
  of owned objects of every colour.
- **Acceptance:** those situations match pixel for pixel outside the panels' masks.
- **Depends on:** T4. **Status:** waiting (T4).

### E24 M1 T6 — the capture set and M1's closure

- **Scope:** the map's capture set (`clients/map/situations/`: every situation of T3b to T5 with
  its masks) and its digests, so a later task captures only what it changes (as E17's `drawn`,
  the runbook's "Which captures a change needs"); M1's closure: a fresh reviewer agent, its
  findings fixed or queued, the review list (the epic's), the user's checkpoint; M2 T0 queued.
- **Acceptance:** the whole set matches as recorded; the review list accepted by the user.
- **Depends on:** T5. **Status:** waiting (T5).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-23 (the client lane, E17 M3 T9c, at the split): moved here from the client lane's
  inbox, the adventure lane's entry of 2026-10-21 (E08 T30, decision 65, ADR-0017 point 6 amended
  in #353; for the hot-seat client, E24 M5): **a level-up holds the game.**
  While a hero's level-up with two offers is open, the engine accepts no command but its owner's
  answer (`h3_game::GameState::waiting_for` names the owner; `level_up(owner)`, or the owner's
  `GameObservation.decision` as `DecisionView::LevelUp`, gives the hero, level, primary skill and
  the two offers; `Answer::First`/`Second` answer it). A battle's defender that won answers it
  outside its turn. The user's requirement for hot seat: when the player who must act changes,
  show a handover screen that hides everything (map, hero list, the dialog) until that player
  confirms it is in control, then the level-up dialog alone, and hand back to the player whose
  turn runs the same way, so no one sees another player's view.
