# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28** (its scope, decisions, tasks and review list:
`docs/epics/E24-archive.md`; its cuts' history: `QUEUE-ARCHIVE.md`); **M3 (the adventure
screen) refined 2026-10-28** (M3 T0, with the user: `docs/epics/E24-map-client.md`, M3's scope,
decisions and exit). Tasks in order, each about half a context; a task that grows is cut at a
seam agreed with the user. Code readings in the sandbox (`docs/exe/README.md`; every address
cited needs a map entry), their detail in `docs/epics/E24-design.md` (§4 from T2: the adventure
screen); captures by the harness (`clients/capture/README.md`), never in git. Every oddity of
the original a reading or a capture finds is an `MCL-Q<n>` in the same task (the HANDOFF's
autonomy contract). Each panel drawn leaves `h3map.py`'s `panels` mask, and `drawn` then
compares it on every situation and sequence.

### E24 M3 T3 — the minimap

- **Scope:** `minimap_draw` 0x412C40 read (13 callers) and built: each tile's colour (the
  terrain's DEF's palette entry 22, design §1.8), the owners' colours, the unexplored tiles, the
  view's box, the map's size scaled to its square, the underground's; redrawn at a scroll, a step
  and a selection as the original does (its callers). Recorded: the sequences show it scrolling.
- **Acceptance:** the minimap exact on every situation and sequence (`panels` without it);
  the Definition of Done.
- **Depends on:** T2a. **Status:** ready.

### E24 M3 T4 — the hero and town lists

- **Scope:** read and built: the hero list (a hero's small portrait `HPS*.pcx`, the movement and
  mana bars `imobil.def` and `imana.def`, the selection's frame, the scroll arrows, empty slots)
  and the town list (`ITPA.def`: a town's picture by its kind, fort and the build of the day; the
  selection, the scroll); `live` gains the viewing player's heroes and towns in their lists'
  order, a hero's movement and mana and the one selected (`h3mem.py`, additive). Situations with
  more heroes and towns than the lists' slots.
- **Acceptance:** both lists exact on every situation and the new ones; the Definition of Done.
- **Depends on:** T2a. **Status:** ready.

### E24 M3 T5 — the status window, the resource bar and the date

- **Scope:** read and built: the status window's kinds (`AdStatHr`, `AdStatCs`, `AdStatOt`,
  `ADSTATIN`, `ADSTATNX`: a hero's portrait, skills, morale, luck and army; a town's; the
  kingdom's and the date's) and when each shows; the resource bar's seven amounts and the date
  (the fonts, the string catalogue's keys from the original's tables); `live` gains the
  resources and the date. Situations for each kind.
- **Acceptance:** the status window, the resource bar and the date exact on every situation and
  the new ones (no `panels` mask left but the status bar's hints); the Definition of Done.
- **Depends on:** T2a (T4 for a hero's or a town's army and portraits). **Status:** ready after T4.

### E24 M3 T6 — the mouse, the keys during a walk, the sound without the focus

- **Scope:** `adventure_mouse_move` 0x40E360's unread part and `view_mouse_tile` 0x40DEB0 (the
  8-pixel question, design §1.8): the cursor over the map by its tile (`cradvntr.def`'s kinds;
  the reach and its days given: a recording's from the original's memory), the status bar's
  hints over the map's objects and the panels (the catalogue's keys); the view-only clicks (a
  hero or a town chosen, the lists scrolled, Next Hero, the underground, the minimap, the status
  window) and the other buttons' press; the panels' hotkeys through the key table; MCL-Q10's
  keys during a walk (Space's cancel, the input ring's bound of 63); the sound without the focus
  (`QUEUE-ARCHIVE.md` 2026-10-27, E24 M2 T6a2: the rest read, the window by
  `Window::focused()`). Recorded: sequences with the clicks and the cursor's kinds.
- **Acceptance:** the new sequences as recorded, every frame; the clicks in the window; the
  Definition of Done.
- **Depends on:** T3 to T5. **Status:** ready after T5.

### E24 M3 T7 — M3's closure

- **Scope:** fresh reviewers (one per area), their fixes; every situation and sequence recorded
  again; the review list with the user; the window cross-built for the user's machine and a
  hand test (`clients/out/win/`, a `.cmd` with the checks on screen and a notes file); M3
  archived; M4 refined with the user.
- **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
- **Depends on:** T1a to T6. **Status:** waiting (the user, at its checkpoint).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-29 (the adventure lane, E08 T68b; for information, nothing to do): the harness's
  reads and steps for batch E8b (`clients/capture/README.md`, "Boats and the sea", since T68b).
  Behaviour that changed for every recipe: `h3lod.identify` also knows `RANDSIGN.TXT` (a sign's
  or Ocean Bottle's default text, `RANDSIGN:<n>`) and lets a record's doubled quote match one or
  two (ADVEVENT 126 was not identified); a message one of the map's message records holds reads
  `the map's text "..."` before any table is tried (GENRLTXT 16, `%s the %s`, took it);
  `object:x,y` reads `none (class 3: the coast)` for a coastal tile without an object; `select`'s
  `controls` count only tiles that hold an object. New: `object:x,y` for the Sea Chest, the Survivor and the Ocean
  Bottle (an artifact chest and a Survivor hold an artifact for `require` and `select`), `select`
  by `is`, `count` and `then`, `h3mem.open_tiles(…, water=True)` (a hero in a boat approaches over
  water), `h3mem.messages`, table rows filled step by step (`row:<first column>`), a grid step's
  `note` at every tile, and the reads `battle:tile`, `boat_record:x,y`, `treasury_changed`,
  `text:<literal>`. The harness check passed.

- 2026-10-28 (the adventure lane, E08 T68a; for information, nothing to do): the harness's
  reads and steps for batch E8a (`clients/capture/README.md`, "Boats and the sea"). Behaviour that
  changed for every recipe: `h3mem.path` (the `marks`, `mark` and `path_tiles` reads) is empty
  while the map shows no path (adventure manager +0x4C, as `path_arrows`): a click that found no
  path left the last path's arrows, which it read before. New: the battle's ship flag in
  `battlefield`, `h3mem.obstacle_hexes`; `h3adv`'s `boat`, `boat[<hero>]`, `boat:x,y`,
  `boats:used`, `drawn`, `opening:field`, `battlefield:ship`, `blocked`, a Flotsam's `object:x,y`,
  and the steps `key`, `again_if`, `only_if = "aboard"` and `dump`. The harness check passed.

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

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
