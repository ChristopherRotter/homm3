# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client, **M1: stills of the map view** (refined 2026-10-23, T0;
`docs/epics/E24-map-client.md`: the decisions, how the viewer sees a map, the milestones). Code
readings in the sandbox (`docs/exe/README.md`; every address cited needs a map entry), their
detail in `docs/epics/E24-design.md`; captures by the harness (`clients/capture/README.md`), never
in git. Every oddity of the original a reading or a capture finds is an `MCL-Q<n>` in the same
task (the HANDOFF's autonomy contract).

### E24 M1 T4b — the masks' gate check and shipped maps' views

T4 was split with the user (2026-10-23): T4a (done: the objects on the writer's maps,
`E24-design.md` §2.6), T4b, and the heroes' place among the objects and the path's arrows in T5.

- **Scope:** a gate check of every object DEF against its `.msk`'s masks (§2.5 point 7, D8: pixels
  in cells the original does not draw; T4a's survey found none but the radar's DEF, which is no
  map object); shipped maps at chosen views without heroes or objects of the eight owned
  classes in them (T5 colours their flags), their random objects' drawn types read from the
  original's memory (`h3mem.py`, additive) into the live part (and whether the original draws
  them before it places the objects on their tiles); the views as whole shipped maps load, not
  copies (`h3map.py` loads a situation's map; a shipped one needs its players' setup).
- **Acceptance:** the shipped maps' views match pixel for pixel outside the panels' masks;
  research item 9 closed in the design notes (the reading and its frames).
- **Depends on:** T4a (done). **Status:** ready.

### E24 M1 T5 — heroes, boats, flags, owners and the path's arrows

- **Scope:** a reading where T1a and T1b did not cover it (a hero's map DEF by class, its group by
  facing, its frame at rest, its flag by colour, boats, an owned object's colours: mines,
  dwellings, towns, the eight classes' flag pixels, D10), the tables' entries; heroes and boats
  as two-row objects among the objects' layers (§2.2, §2.3: D4, D5; from T4 with the user's
  split), the path's arrows and their shadows (passes 6 and 8, D9; from T4); the unexplored tiles
  from the original's explored tiles (`h3mem.py`) in the live part; situations of heroes of every
  class facing each way, of a hero beside an object's blocked cell and two heroes side by side, of
  boats (D6, D7), of a path under an object, and of owned objects of every colour.
- **Acceptance:** those situations match pixel for pixel outside the panels' masks.
- **Depends on:** T4a (done; T4b for shipped maps with heroes). **Status:** ready (after T4b
  in the queue's order).

### E24 M1 T6 — the capture set and M1's closure

- **Scope:** the map's capture set (`clients/map/situations/`, started by T3b2 with `set.toml`
  and `h3map.py set`: every situation of T3b2 to T5 with
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
