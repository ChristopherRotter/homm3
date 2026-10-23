# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client, **M1: stills of the map view** (refined 2026-10-23, T0;
`docs/epics/E24-map-client.md`: the decisions, how the viewer sees a map, the milestones). Code
readings in the sandbox (`docs/exe/README.md`; every address cited needs a map entry), their
detail in `docs/epics/E24-design.md`; captures by the harness (`clients/capture/README.md`), never
in git. Every oddity of the original a reading or a capture finds is an `MCL-Q<n>` in the same
task (the HANDOFF's autonomy contract).

T5 split with the user (2026-10-23) into T5a (heroes and boats, done) and T5b (owners, the path,
the explored tiles).

### E24 M1 T5b — owners' colours, the path's arrows and the explored tiles

- **Scope:** an owned object's colours (mines, dwellings, towns, the eight classes' flag pixels,
  D10; the owner's tile 0x4FEC30 as the entrance), the tables' entries; a town's picture by its
  buildings (the same copy as a random object's, 0x4C9990, `E24-design.md` §2.7); a random
  dwelling's anchor at a new game (§2.7, not read: `render --live` draws it at the map's
  meanwhile); the path's arrows and their shadows (passes 6 and 8, D9); the unexplored tiles from
  the original's explored tiles (`h3mem.py`) in the live part; situations of a path under an
  object, of owned objects of every colour, and shipped maps' views with heroes and owned objects;
  if cheap, a boat's flag of a colour other than Red's (T5a captured Red's only: another colour's
  hero must board, hot seat).
- **Acceptance:** those situations match pixel for pixel outside the panels' masks.
- **Depends on:** T5a (done). **Status:** ready. Since T5a the situations give heroes of any
  colour and boats, Red's heroes walk by play and the harness turns heroes and boats
  (`E24-design.md` §2.8); a path is shown by a hero's first click on a tile (`h3adv.walk`'s
  first click; `setup.sh` sets Show Route).

### E24 M1 T6 — the capture set and M1's closure

- **Scope:** the map's capture set (`clients/map/situations/`, started by T3b2 with `set.toml`
  and `h3map.py set`: every situation of T3b2 to T5b with
  its masks) and its digests, so a later task captures only what it changes (as E17's `drawn`,
  the runbook's "Which captures a change needs"); M1's closure: a fresh reviewer agent, its
  findings fixed or queued, the review list (the epic's), the user's checkpoint; M2 T0 queued.
- **Acceptance:** the whole set matches as recorded; the review list accepted by the user.
- **Depends on:** T5b. **Status:** waiting (T5b).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-23 (this lane, E24 M1 T4b; for T6, the capture set): **the palettes at the frozen clock
  can be half a tick apart.** In about one still of seven `h3map.py`'s palette read found the
  terrains' cycling palettes (`lavatl`, `watrtl`) one step ahead of the rivers' for the whole 2 s
  it reads (its error names each range's fits); `set` shows the situation again and it is exact.
  Read whether the tick (0x4ED490) rotates the rivers apart from the terrains, what such a
  screen shows, and either step the clock past it or give `render` a step per DEF.

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

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
