# Map client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/map-client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-10-28 (this lane, E24 M3 T0): M3 refined with the user; the queue's T0, and the inbox's
  two entries M3's tasks took (the frame's cost into T2, the sound without the focus into T6) and
  the adventure lane's notice of 2026-10-26 (for information), verbatim:

  The client lane's inbox entry it answered (2026-10-27), as it stood:

  - 2026-10-27 (the map client lane, E24 M2 T6b2a; **for you to answer before E24 M3 T0**):
    **the common crate, proposed.** E24's M2 decision moves what both viewers need into one crate
    of the client workspace common to both lanes (`lanes.toml`'s foundation list: a `shared/`
    pull request), at E24 M3 T0 (queued behind M2's closure): `h3-view`'s `clock` and `cursor`
    (copied into `h3-map-view` at M2: the copies go), `ui`, `text`, `layout` and the parts of
    `mouse` that are not the battle's (`screen_to_hex` and the battle's cursor stay); from
    `h3-map-view` the key table `keys` (both viewers' keys through it: the 2026-10-26 entry below)
    and perhaps `music` (the 2026-10-27 entry). Asked of you: what you would rather keep in
    `h3-view`, what E17 M5's tasks will change in these meanwhile (the move then waits for a quiet
    point), and a name if you have one; answer in the map client lane's inbox.

  ### E24 M3 T0 — M3 refined with the user

  - **Scope:** first the common crate, proposed to the client lane (its inbox, since T6b2a): one
    crate of the client workspace both lanes share, holding `h3-view`'s `clock`, `cursor`, `ui`,
    `text`, `layout` and the parts of `mouse` that are not the battle's, and `h3-map-view`'s
    `keys` (both viewers' keys through it) and perhaps `music`; one `shared/` pull request for
    `lanes.toml`, then the move, `h3-map-view`'s copies gone (the epic's M2 decision). Then M3 with
    the user: its scope, decisions and exit (the epic's "Later": the minimap, the hero and town
    lists, the status window, the resource bar and the date, the buttons, the mouse and its
    cursor; the string catalogue's keys), MCL-Q10's keys during a walk (Space's cancel, the
    input ring's bound), MCL-Q9 if still open, the inbox's frame cost and the sound without the
    focus placed; M3's tasks queued.
  - **Acceptance:** M3's section refined in the epic; the common crate agreed with the client lane
    (or its question asked); M3's tasks queued.
  - **Depends on:** M2 closed (2026-10-28). **Status:** the user present.

  - 2026-10-27 (this lane, E24 M2 T6a2): **the sound without the focus** is not built. The original
    on losing it (`window_procedure` 0x4F7E88, `sound_focus_lost` 0x599C40, read in part) stops
    every effect sample and fades the music out (the stub 0x59A830: `music_stop_fade`, its place
    kept); on regaining it `music_resume` 0x59AC00 asks for the track again (it fades in where it
    faded out) and the effects come back by 0x4F7E2E (not read). The window plays on without the
    focus. With M3's settings or the next sound task: read the rest, then `h3_map_view::music`'s
    model takes a fade without a pending track and the window follows `Window::focused()`.

  - 2026-10-27 (this lane, E24 M2 T4b3; the user: "it will do for now, but we should check back in
    at some point"): **a frame's fixed cost.** A frame takes 4 to 6 ms to make on the user's machine
    (5.7 in the sandbox), about 4 even for a near-empty view: the frame picture's and the terrain's
    pixels drawn every frame, the widening to 32 bits, the painter's two copies of the screen, the
    scene built afresh, the objects' transparent pixels still visited (design §3.12, "Left"). Take
    it up with the next work that touches the drawing (M3's panels at the latest): spans of a
    frame's drawn pixels made once, the frame picture converted once a viewer, no screen copies, a
    scroll or a walk redrawing only what moved; measure with `map-measure.cmd`'s "a frame made in".

  - 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
    reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
    changed for every recipe: `before:<observation>` is now read right before the step (any
    observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
    pictures is answered by `choice` (it stopped the batch before), and each message closed with its
    OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
    `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
    stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
    `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

- 2026-10-28 (this lane, E24 M2 closed at T6b2c): the queue's M2 history and its last task,
  verbatim:

  Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
  (the map moving) refined 2026-10-26** (M2 T0, with the user: `docs/epics/E24-map-client.md`, M2's
  scope, decisions and exit; the recording's format there). Tasks in order, each about half a
  context; a task that grows is cut at a seam agreed with the user. Code readings in the sandbox
  (`docs/exe/README.md`; every address cited needs a map entry), their detail in
  `docs/epics/E24-design.md` (§3 from T1a: the map moving); captures by the harness
  (`clients/capture/README.md`), never in git. Every oddity of the original a reading or a capture
  finds is an `MCL-Q<n>` in the same task (the HANDOFF's autonomy contract).

  T1a was cut at its seam before it started (the user, 2026-10-26): T1a1 the recording and the map
  standing still, T1a2 the scroll; T1b, the walk, done whole (all three done 2026-10-26:
  `E24-log.md`); T2, the map over time headless, done whole (2026-10-27: the user chose it over a
  cut before the frame check); T3 cut at its seam before it started (the user, 2026-10-27): T3a,
  the walk's timing headless, and T3b, the walk drawn (taken whole), both done 2026-10-27. T4 cut
  at its seam before it started (the user, 2026-10-27): T4a, the window replaying and the key
  table (done 2026-10-27), then T4b, the map's live input and the Windows build, cut at its seam
  before it started (the user, 2026-10-27): T4b1, the live input in the sandbox (done 2026-10-27),
  then T4b2, the Windows build on the user's machine (done 2026-10-27), which found the view's
  drawing too slow: T4b3 (the user, 2026-10-27; done 2026-10-27). T5, the sound, done whole 2026-10-27 (the user
  chose it over a cut).

  T6, M2's closure, cut at its seam before it started (the user, 2026-10-27): T6a, the reviewers
  and their fixes (done 2026-10-27: 24 of 29 fixed; `E24-log.md`), its five music findings cut out
  as T6a2 (the user; done 2026-10-27, whole), then T6b, the closure, cut at its seam before it
  started (the user, 2026-10-27): T6b1, the recordings again and R2-8 (done 2026-10-27: every
  sequence and still exact; R2-8 as #523), then T6b2, the user's, cut at its seam (the user,
  2026-10-27, away from the machine): T6b2a, the review list and the questions (done 2026-10-27:
  the review list accepted, MCL-Q7, Q8 and Q12 answered, a build and a checklist on the user's
  machine), then T6b2b, the answers built (done 2026-10-28: MCL-Q7, Q8 and Q12 the client's, the
  original's behind `--bugs original`), and T6b2c, the hand test and M2 archived.

  ### E24 M2 T6b2c — the hand test and M2 archived

  - **Scope:** the window by hand on the user's machine: `clients/out/win/map-try.cmd` (the T6a
    fixes: Alt+Tab with the pointer at an edge, fullscreen's bars, F7 in a situation; T6a2's
    music by ear in `seq-walk`, T6b2b's MCL-Q12 in item 2c: Grass comes back), the user's notes
    in `map-try-notes.txt` beside it; T6b2b put there the build of its branch (`--target
    x86_64-pc-windows-gnu`; T6b2a's kept as `h3-map-view-t6b2a.exe`): build again only if the
    viewer changed on `master` since and the notes are not written yet. Each finding checked
    (fixed, queued or an `MCL-Q<n>`);
    then M2's section moved verbatim to `E24-archive.md` (scope, decisions, task list, the
    accepted review list; a row and a pointer left) and the HANDOFF's M2 lines trimmed.
  - **Acceptance:** no finding of the hand test left unrecorded; M2 archived; the Definition of
    Done.
  - **Depends on:** T6b2b. **Status:** the user at the machine (or the notes file written).

- 2026-10-24 (this lane, E24 M1 T6b's reviewer, R2-8; with the next writer pull request): the
  writer's edge bound for an object (`crates/h3-map/src/writer/extras.rs` `PAST_RIGHT`, `PAST_BOTTOM`, its test
  at 799) lets an anchor lie 7 past the map's side, where an 8-wide template covers no cell on
  the map: one too generous, its comment off by one (6 and 4, or the comment). The adventure
  lane's crate: a `cross-lane` change, so it waits for the next addition the situations need.
  *Done 2026-10-27 (E24 M2 T6b1): #523.*

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

