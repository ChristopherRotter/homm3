# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client, **M1: stills of the map view** (refined 2026-10-23, T0;
`docs/epics/E24-map-client.md`: the decisions, how the viewer sees a map, the milestones). Code
readings in the sandbox (`docs/exe/README.md`; every address cited needs a map entry), their
detail in `docs/epics/E24-design.md`; captures by the harness (`clients/capture/README.md`), never
in git. Every oddity of the original a reading or a capture finds is an `MCL-Q<n>` in the same
task (the HANDOFF's autonomy contract).

### E24 M1 T3a — the writer's additions (`cross-lane`)

- **Scope:** what T3b's to T5's situations need of `h3_map::writer` (the adventure lane's crate),
  added without changing what it writes today (the user's consent, 2026-10-23): rivers, a tile's
  terrain, river and road pictures and flags as given (so transitions can be written exactly),
  objects of any template (any class `Objects.txt` lists, without a body or with an empty one).
  Its own small pull request, label `cross-lane`, with the adventure lane's tests and a note in
  its inbox; `cargo xtask verify --scope adventure` without a failing step.
- **Acceptance:** the writer's tests cover the additions; every map the writer wrote before is
  byte for byte unchanged (the fixtures and the replays pass as they are).
- **Depends on:** T1a, T1b (done; the situations' needs: `E24-design.md` §2.5). **Status:** ready.

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
- **Depends on:** T2 (done: `h3-map-view`, its tables, layout, `art` and `live`), T3a.
  **Status:** waiting (T3a).

### E24 M1 T4 — the objects and their shadows in the original's order

- **Scope:** the objects drawn as T1b read (`E24-design.md` §2: the draw lists and layers, the
  passes, the shadows by the compositor's shadow drawer, heroes and boats as two-row objects,
  objects past the map's edges); a gate check of every object DEF against its `.msk`'s masks
  (§2.5 point 7: pixels in cells the original does not draw); the situations T1b named (§2.5, the
  writer's maps of overlapping objects, each showing one rule or one difference from VCMI);
  shipped maps at chosen views, their random objects' drawn types read from the original's memory
  (`h3mem.py`, additive) into the live part.
- **Acceptance:** the situations and the shipped maps' views match pixel for pixel outside the
  masks of what T5 draws; research item 9 closed in the design notes (the reading and its frames).
- **Depends on:** T1b (done), T3b. **Status:** waiting (T3b).

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
