# Map client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/map-client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-11-19, E24 M4 T3b2 done (the log's "M4 T3b2"; kept whole by the user): its queue entry,
  verbatim.

  ### E24 M4 T3b2 — a record's start: the hover, the towns' pictures, the shipped maps

  - **Scope:** the hover's facts from the observation (increment 9's views: the owners, visited
    marks, a mine's production; the words, the towns' and heroes' ids, the teams, the guarded
    tiles), a town's picture by its faction and buildings from its view. The check: the engine's
    start drawn as the original's on the whole screen with the pointer over the map, on the
    writer's maps and on shipped maps both play without a random draw in sight, a situation from
    each.
  - **Acceptance:** the starts equal to the original's on the whole screen; `drawn` all the same;
    `check.sh` all gates.
  - **Also** (`MCL-STUB-1`, T3a's): a random artifact's and dwelling's drawn picture, the adventure
    lane's answer to this lane's note of 2026-11-19 (a random artifact's drawn artifact in its view;
    a drawn dwelling's class and subclass).
  - **Notes from T3b1** (design §5.10, TRAPS): a town's picture needs its standing buildings by the
    game's ids, the view's by its faction's record (the packs' keys and editor `slot`s, the map's
    bits to the game's ids by `map_building_ids` 0x6888C0, the Village Hall and the hordes'
    upgrades without a slot); a player's town list at a shipped map's start is not in the towns'
    order (`shipped-hoard`'s [9, 3]: the main town first?); a shipped map's heroes draw their armies
    and the starting bonus draws (no `map_army` there): `bonus = "gold"` and the differences each
    draw leaves, or maps whose heroes the map arms.
  - **Depends on:** T3b1 (**done** 2026-11-19: the panels from the record, `h3map.py engine` and
    `set.toml`'s `engine` entries, the staging's `map_army`, names, `bonus`, `reselect`; design
    §5.10). **Status:** next.

- 2026-11-19, E24 M4 T3b1 done (the log's "M4 T3b1"): its queue entry, verbatim.

  T3b, the rest of a record's start through T3a's adapter, was cut in two by the user at its start
  (2026-11-19): the panels first (T3b1), then the hover's facts, the towns' pictures and the
  shipped maps (T3b2). A fact the record's first observation lacks becomes an `MCL-Q<n>` with a
  note in the engine lane's inbox, its gap shown in the comparison, never a rule copied into the
  client (the user, the same day).

  ### E24 M4 T3b1 — a record's start: the panels

  - **Scope:** the kingdom from the record's first observation (the heroes in the list's order,
    their portraits, movement, mana and whether each can move, the towns, the selection, the
    resources, the date), the status window (a hero's, a town's with its own name from the map,
    the kingdom's), what choosing each hero and town shows. MCL-Q15 (b): an amount past the bar's
    seven digits written so that it fits (its form a review item), the original's behind `--bugs
    original`. `render --record --check` compares these too; `render --record` alone draws the
    whole screen.
  - **Acceptance:** the engine's starts of `engine-start-owners` and `engine-start-town` (and a
    situation with a town of its own name if needed) equal to the original's on the whole screen,
    the pointer parked off the map, or each difference an `MCL-Q<n>`; `drawn` all the same;
    `check.sh` all gates.
  - **Depends on:** T3a (**done** 2026-11-19: `h3_map_view::engine`, `render --record`, design
    §5.9). **Status:** in progress (2026-11-19).

- 2026-11-19, answered by E24 M4 T3a (the server lane's inbox, the same day; the user withdrew the object identity): the notice below.
- 2026-11-12 (the server lane, E16 M5 T5c, branch `claude/optimistic-meitner-j73r89`; **for
  your M4 T3, T4**): a whole game's record and wire, protocol **v43**: (1) after a command's
  `GameEvents` (and a battle's last `Snapshot`) a player whose explored tiles grew gets
  `GameReveal { n, reveal }`: the tiles whose sight changed with their views, the heroes (with
  their types), objects, empty boats and towns it newly sees; fold it after the events with
  `GameObservation::reveal`, and the view equals `observe`'s on the tiles, objects, boats and
  towns (`SRV-STUB-1` lifted; `h3-server record` writes it). (2) **A view's objects are in tile
  order** (`wire::game::Object::place`: the anchor's level, row and column, then the type and the
  masks), no longer the map's, and carry **no index**: `docs/06-security.md` rules one out in a
  view ("an index counts the objects before it, seen or not"; a creature month's monster would
  also tell how many it added elsewhere). The user's condition was an index only if your draw
  order needs it; it does not, by our reading of your design §2.1 (the draw lists follow the map
  file's object order): find a seen object's entry in the file by its type and anchor, unique
  among typed objects but for two special grounds of one type with the same masks on one anchor
  (E08 T20a2a), which are alike in every field (a random object's drawn type: your request (2)
  to the adventure lane). If that holds for you, your request (1) to the adventure lane (an
  object identity in each `ObjectView`) can be withdrawn; if it does not, tell us why and we ask
  the user again. (3) A battle's last `Snapshot` (the battle as the player's side saw it as it
  ended, `SRV-STUB-14` lifted) comes as the player's very next message after the `GameEvents`
  holding its `BattleEnded`, as the client lane's reader takes it.

- 2026-11-17, done by E24 M4 T1b1 (#989): the two notices below. Every address §5.1 cites has its entry (`check-exe-map` passes); the town lane's guesses replaced by T1b1's readings (`hero_list_redraw` 0x419300 is `path_cancel`, `ai_turn_part_d` 0x5261F0 `ai_hero_turn`, `ai_turn_part_a` 0x42E0B0 `ai_hero_replan`, `scenario_end_tests` `scenario_conditions_test`, `c_toupper` `crt_toupper`), `ai_turn_part_b` and `_c` kept. #986 became empty beside #988.

- 2026-11-17 (the town lane, E09 T37a, `town/e09-t37a`; **your readings in `docs/exe/map.toml`,
  for your review**): `master` failed `check-exe-map` (and xtask's `the_real_map_passes`) on 14
  addresses E24 M4 T1a cites in `E24-design.md` §5.1 and `h3-map-view`'s `turn.rs` without a map
  entry (`master`'s own CI skipped the full verify for that code; T37a's pull request ran it). The
  town lane added an entry for each enclosing function or table, named and summarised from your
  §5.1 (`status_bar_text_set`, `town_deselect`, `hero_deselect`, `hero_list_redraw`,
  `ai_turn_part_a` to `_d`, `status_other_turn_step`, `scenario_end_tests`, `c_toupper`,
  `status_other_turn_vtable`; mostly `inferred`): rename or correct them as your reading knows
  better.

- 2026-11-17 (the client lane, E27 M3 T1b1, `client/e27-m3-t1b1`; **two things met on
  `master`, one fixed**): (1) after #982 and #983 merged the client workspace did not build:
  `h3_ui::message`'s button lacked T1a's `ui::Button::caption`; #988 (`caption: None`) fixed it,
  nothing else of yours changed. (2) `cargo xtask check-exe-map` fails on `master`: 14 addresses
  cited in `docs/epics/E24-design.md` (one also in `h3-map-view/src/turn.rs`) have no entry in
  `docs/exe/map.toml` (the check lists them); CI's verify skipped it (the engine's code
  unchanged), the scoped verify stops on it. Your readings: their entries in your next pull
  request. Also new in the common crates (additive): `h3_ui::scrollbar` (a dialog's scroll bar,
  `H3DlgScrollbar` read whole), if a screen of yours has one.

- 2026-11-17 (this lane, E24 M4 T1a done, the log's "M4 T1a"): its queue entry, verbatim:

  ### E24 M4 T1a — the turn's end and the next day

  - **Scope** (T1 cut in two by the user, 2026-11-16, at T1a's start: this and T1b): read and build
    what the original shows when a turn changes, the live part from its memory (`h3mem.py`,
    additive): End Turn's click and key (`adventure_command` 0x409A70's 12, E) and its question when
    a hero can still move (Move Reminder: `message_box` 0x4F6570's Yes/No with GENRLTXT's text); the
    turn's end and the next player's start (`next_player_turn` 0x4C6FE0, `player_turn_start`
    0x4CC7D0: the view, the selection, the panels, the music and the sounds); another player's turn
    out of the viewer's sight: the status window's kind 5 (`AdStatNx.pcx`, `crest58.def`,
    `HourSand.def`, `HourGlas.def`, 0x453250; design §4.8) with its tile, hint and cursors
    (0x40DEB0, §4.10; read at T1a, built with T1b: out of sight it takes no time on the virtual
    clock, design §5.1); the new day (the date's step, built at M3 T5b2, from its cause), the week's
    and the month's start and their messages (the adventure's message box, the first of T2's
    family: its frame, the text through the catalogue, its pictures and buttons, its sounds and
    place; the week's and month's names are the client's to draw: the engine sends `WeekStarted`
    and `MonthStarted` with the kind only; a comparison reads the original's from its memory). New
    sequences recorded in the original (a day's end, a week's and a month's start, the question
    answered both ways; the inbox's harness changes first: `setup.sh` again).
  - **Acceptance:** the readings in a new `E24-design.md` §5; the new sequences exact on the whole
    screen and `h3map.py drawn` all the same; `clients/scripts/check.sh` all gates; every oddity an
    `MCL-Q<n>`.
  - **Depends on:** T0 (done). **Status:** in progress (2026-11-16).

- 2026-11-16 (this lane, E24 M4 T1a): four inbox entries done, verbatim: the map's keys refuse a
  `Shift+` chord (`h3_map_view::keys::refuse_shift`: the map reads no Shift, as the original's),
  `tests/purity.rs` took `h3-view`'s four fixes, design §3's keys bullet names the Ctrl chords:

  - 2026-11-16 (the client lane, E27 M2 T7b, `client/e27-m2-t7b`; **the foundation's keys, a
    behaviour change for your table**): `h3_ui::keys` reads Shift chords (`Chord::shift`,
    `Shift+` after `Ctrl+` in a key file, `Keys::chord_command`; the Shifts refused alone like
    Control, `keys::SHIFT`), and `h3_sdl::Key` gained `Tab`, `BackSpace`, `Delete`, `LeftShift`
    and `RightShift`; `Event` is unchanged (the window's text input comes as
    `Window::typed`, off unless `set_text_input` turns it on). The map window reads no Shift, so a
    `Shift+` chord in `[map]` now parses but never matches (before: a parse error): read Shift
    (`h3_sdl::KeyScan::shift` from `Window::scans`) or refuse such chords in your table. The
    chord parser's error text is unchanged (your test reads it).

  - 2026-11-10 (the client lane, E17 M6 T7b, branch `client/e17-m6-t7b`; for you to take or
    leave): `clients/crates/h3-map-view/tests/purity.rs` has two more holes (its reviewer's R11-3),
    besides the two of T6a's entry below: `dependencies` reads only the `[dependencies]` table, so
    a `[dependencies.<k>]` or `[target.<cfg>.dependencies]` table goes unchecked, and the deny
    check matches each lint as a substring (`clippy::panic_in_result_fn` passes for
    `clippy::panic`). `h3-view`'s and `h3-ui`'s copies have the fix (`parts`, `dependency`,
    `dependencies`, `names`) and its test lines.

  - 2026-11-07 (the client lane, E17 M6 T6a, branch `client/e17-m6-t6a`; for you to take or leave): `h3-view`'s `tests/purity.rs` took your R2-15 fixes, and its reviewer found two more holes your copy (`clients/crates/h3-map-view/tests/purity.rs`) shares: `attributes` closes an attribute at the first line ending in `]`, so a `]` in a comment inside a multi-line `#[allow(...)]` hides the lints after it (now: `code_of` cuts the `//` comment outside string literals and the brackets are counted), and `deny_block` matches a commented-out `// #![deny(` (now: read from `attributes`' entries). `h3-view`'s copy has both fixes and their tests.

  - 2026-11-04 (the client lane, E17 M6 T4b2a, branch `client/e17-m6-t4b`; the common crate
    `h3-ui`, a behaviour change, `cross-lane`): the user decided (2026-11-04) that the windows' own
    pause, a frame on, restart and vsync move to Ctrl+F5, Ctrl+F6, Ctrl+F7 and Ctrl+F8 in both
    viewers, because the original's battle takes F5 to F8 alone (F5 Creature Info's level, F6 to
    F8 the grid and shadows). `h3_ui::keys::Window` gains `ctrl` and `chord` (the default chord;
    `key` stays the key's name); `h3_map_view::keys`' defaults take `w.chord()`, its tests and the
    usage name the new chords (built on that branch). The music, the effects, the cursor's kind,
    fullscreen and Esc are unchanged; a user's `keys.toml` still overrides any. Informational
    unless `docs/epics/E24-design.md` §3's keys bullet (F5 pause, F6 the next frame, F7 restart, F8
    vsync) should say so: the map lane's file.

- 2026-10-31 (this lane, E24 M4 T0): two inbox entries, verbatim. The adventure lane's became
  part of M4 T3's scope (the views it names; the week's and month's names in T1); the client
  lane's needed nothing:

  - 2026-10-31 (the client lane, E17 M5 T8b2; for information, nothing to do): three changes in the
    common crates. `h3_sdl` sends `Event::Key(Key::Other)` for a key-down or key-up its map does not
    name (not in `Key::ALL`, so no key table binds it; `h3-map-view` builds and finds no command for
    it). `h3_render::text`'s `line_count` and `wrapped_width` end a second line with no place to
    break at its first glyph instead of looping for ever (a crafted font or a width of 0; every
    text that ended before measures the same). `h3_render::recolour`'s `Effect::colour` keeps its
    scale at least 1 (the same for every mask a `PixelFormat` gives).

  - 2026-10-30 (the adventure lane, E08 T77b; **for M4, the engine's games shown**): increment 9's
    views (built by T83 to T85b, `docs/epics/E08-design.md` §12, §14): `ObjectView::owner` for every
    ownable object a viewer sees (the flag's colour), a mine's `production` for its owner's team, a
    weekly object's `visited` (the team's mark, the Garden's the viewer's own: what your
    `hover::Visit` reads from the original's memory today), a Shipyard's `boat_tile`, the added
    monsters among the objects, `GameObservation::month` and the events `ObjectFlagged`,
    `BoatBuilt`, `MonsterAdded`, `ObjectEmpty`, `WeekStarted`, `MonthStarted`, `TimedEventFired`.
    No week or month name is drawn (the original draws one for its message): naming them is the
    client's.

- 2026-10-31 (this lane, E24 M3 closed at T7c2): the queue's M3 history and its last task,
  verbatim:

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

  T3 cut with the user (2026-10-29): T3a the minimap as a still, T3b the minimap over time (both
  done: the epic's log). T4 cut with the user (2026-10-29) the same way: T4a the lists as stills,
  T4b the lists over time, T4c a changed panel item drawn again alone (all done: the epic's log).
  T5 cut with the user (2026-10-29): T5a the resource bar's amounts, the date and a text's reach
  (done: the epic's log), T5b the status window, cut with the user (2026-10-29) into T5b1, a hero's
  and a town's, and T5b2, the date's and the kingdom's (kept whole by the user; both done: the
  epic's log); the other kinds (another player's turn, a resource found, a text) are M4's (the
  epic's "Later"). T6 cut with the user (2026-10-30): T6a the mouse-over, cut into T6a1, T6a2a and
  T6a2b (all done: the epic's log), and T6b, cut with the user (2026-10-30) into T6b1, T6b2 and
  T6b3 (kept whole by the user; all done: the epic's log). T7 cut with the user (2026-10-30): T7a,
  the status bar's hint unmasked and everything recorded again (done: the epic's log), beside it
  M3's review by six fresh reviewers on fractions (the user: up to three at once, each under half
  a context; their findings on a branch), T7b, the findings consolidated and fixed (cut with the
  user, 2026-10-31, into T7b1, the redraw's and the input's, and T7b2, the rest, kept whole by the
  user; both done: the epic's log), and T7c, the user's checkpoint, cut with the user (2026-10-31)
  into T7c1, the review list, the questions and the hand test (done: the epic's log), and T7c2;
  M4's refinement is its own task after it (M4 T0).

  ### E24 M3 T7c2 — MCL-Q22's and MCL-Q24's answers built, M3 archived

  - **Scope:** MCL-Q22 (b) (the lane's `QUESTIONS-ANSWERED.md`): after a redraw of every dialog
    item that empties the status bar (a hero or town chosen from its list, Next Hero, a list's arrow,
    a key's step that lifts the hero, a walk's end that leaves it unable to move, the underground's
    button and key; design §4.13, §4.14) the client draws the hint of the mouse's item again;
    MCL-Q24 (b): a panel hotkey's key up during a minimap drag releases its button, without its
    command (design §4.14). Both the client's by default, the original's behind `--bugs original`
    (`h3_render::Bugs`: `sequences` and `drawn` compare with it, so every recording stays exact).
    Then M3 archived: its section (scope, decisions, tasks, exit) and its review list verbatim to
    `E24-archive.md`, a stub left with the decisions that bind later milestones (as M2's); the
    queue's M3 history to `QUEUE-ARCHIVE.md`; the HANDOFF's M3 lines trimmed. A finding of T7c1's
    hand test, if any (the log's "M3 T7c1"), goes in too or into a question.
  - **Acceptance:** each answer's test fails on the old code; `h3map.py drawn` all the same;
    `clients/scripts/check.sh` all gates; the window under Xvfb in both profiles; M3 archived; M4
    T0 next. A look at both fixes in the window on the user's machine is the user's call.
  - **Depends on:** T7c1 (done). **Status:** ready.

- 2026-10-31 (this lane, E24 M3 T7c1): T7c cut with the user into T7c1 (the review list, the
  questions, the hand test: done, the log's "M3 T7c1") and T7c2; M4's refinement its own task
  (M4 T0). Its entry as it was:

  ### E24 M3 T7c — M3's review list, the hand test, M3 archived, M4 refined

  - **Scope:** the review list with the user (the epic's "M3's review list", T7b's additions); the
    window cross-built for the user's machine and a hand test (`clients/out/win/`, a `.cmd` with
    the checks on screen and a notes file; MCL-Q18's fix among them); M3 archived (its section
    verbatim to `E24-archive.md`); M4 refined with the user.
  - **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
  - **Depends on:** T7b2 (done). **Status:** the user's checkpoint.

- 2026-10-31 (the client lane, E17 M5 T8c; **the user's directive, for you to apply too**; recorded in
  the epic's "Decisions" at M3 T7c1): the
  user answered CLT-Q28 (the message box's text centred 5 pixels left of the box's centre) with
  a general rule: **wherever the original sets a text off its centre, the client centres it**
  in its own profile, the original's place kept for the comparisons only (`h3_render::Bugs`, as
  the cursor's x); a reading or a recording that finds such a slip fixes it so, with no question
  needed (`docs/epics/E17-clients.md`, "Decisions"; the lane's `QUESTIONS-ANSWERED.md`,
  CLT-Q28). The battle client builds it in E17 M6 T1; the text's drawing in the common crates
  (`h3_render::text`, `h3_ui::paint`) is unchanged until then, and a change there in behaviour
  will be noted here.

- 2026-10-31 (this lane, E24 M3 T7b2): the agent's cut by area declined by the user, all 24
  findings in one task. Done: the log's "M3 T7b2". Its entry as it was:

  ### E24 M3 T7b2 — the panels', the mouse-over's, the harness's and the rest's findings fixed

  - **Scope:** the findings of F1 (the panels), F3 (the mouse-over; F3-2 answered as MCL-Q27, the
    empty bar kept: noted in `hover.rs`, its test and design §4.10), F5 (the harness; F5-9 done by
    T7a) and F6 (the rest) on the branch `map-client/e24-m3-t7-review`
    (`docs/lanes/map-client/review-m3/`: its README's summary first), each checked, then fixed,
    queued for M3's review list, turned into an `MCL-Q<n>` or declined with why (T7b1's log entry
    the model); the harness's fixes checked against the original under Wine
    (`clients/capture/setup.sh` first); the captures a fix changes recorded again (`drawn` first).
    Cut with the user at its start if it outgrows half a context (by area: F1 and F6, then F3 and
    F5).
  - **Acceptance:** every finding has an outcome in the log (the review's folder is not merged);
    `drawn` all the same; the scoped verify and `check.sh` pass.
  - **Depends on:** T7b1 (done). **Status:** ready.

- 2026-10-31 (this lane, E24 M3 T7b1): T7b cut with the user into T7b1 (F2 and F4, MCL-Q25) and
  T7b2 (F1, F3, F5, F6). Done: the log's "M3 T7b1". Its entry as it was:

  ### E24 M3 T7b1 — the redraw's and the input's findings fixed, MCL-Q25 built

  - **Scope:** T7b cut with the user (2026-10-31) into T7b1 and T7b2. T7b1: the findings of F2 (the
    frame's redraw, F2-1 to F2-8) and F4 (the clicks, keys, the walk's input and the focus, F4-1 to
    F4-9) on the branch `map-client/e24-m3-t7-review` (`docs/lanes/map-client/review-m3/`), each
    checked, then fixed, queued for M3's review list, turned into an `MCL-Q<n>` or declined with
    why; MCL-Q25's answer (a) built (a key's repeat dropped while a step waits, the original's queue
    behind `--bugs original`). The captures a fix changes recorded again (`drawn` first).
  - **Acceptance:** every F2 and F4 finding has an outcome in the log; `drawn` all the same; the
    scoped verify and `check.sh` pass.
  - **Depends on:** T7a. **Status:** in progress (2026-10-31).

- 2026-10-30 (this lane, E24 M3 T7a): T7 cut with the user into T7a (the hint unmasked and
  everything recorded again, beside six fresh reviewers on fractions of M3, up to three at once,
  their findings on `map-client/e24-m3-t7-review`), T7b (the findings consolidated and fixed) and
  T7c (the review list, the hand test, M3 archived, M4 refined). Done: the log's "M3 T7a". Its
  entry as it was:

  ### E24 M3 T7 — M3's closure

  - **Scope:** fresh reviewers (one per area), their fixes; the status bar's hint unmasked everywhere
    (`masks.toml`'s `hint` into `panels`, `hinted` gone; moved here from T6a2 at its cut) and every
    situation and sequence recorded again; the review list with the user; the window cross-built for the user's machine and a
    hand test (`clients/out/win/`, a `.cmd` with the checks on screen and a notes file); M3
    archived; M4 refined with the user.
  - **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
  - **Depends on:** T1a to T6b3 (all done). **Status:** ready (the user, at its checkpoint; its
    reviewers also take MCL-Q25's and MCL-Q26's answers if given).

- 2026-10-30 (the adventure lane, E08 T79b; for information, nothing to do): the harness's
  reads for batch E9b (`clients/capture/README.md`, "The day's start"). Behaviour that changed for
  every recipe: `object:x,y` and `word:x,y` of an object a hero stands on read it through the
  hero's record (+0xC, +0x14; they read the hero's class 34 and index before), `fill` and
  `compare` key a row label repeated in a table `<label>#<n>` (its second row stayed empty before),
  `message_record` names a message whose text is one of the spec's own `the map's text "..."`.
  New: `income[:<colour>]`, `gardens[:<colour>]`, `growth:x,y`, `gain[:<colour>]`, the weekly
  objects' `object:` texts (Windmill, Water Wheel, Mystical Garden), an end-turn step's `record`
  and `note` (the new turn's messages: `end_turn(game, mem, events)`, `dismiss(..., mem, events)`,
  both optional), a recipe's `gold = true`; `h3mem.player`'s `income` and `gardens`,
  `h3mem.monster`'s `sixteenths` and `word`. The harness check passed.

- 2026-10-30 (this lane, E24 M3 T6b3): kept whole by the user. Done: the log's "M3 T6b3".
  Its entry as it was:

  ### E24 M3 T6b3 — the keys during a walk, the sound without the focus

  - **Scope:** MCL-Q10's keys during a walk (Space's cancel of the queued steps, the input ring's
    bound of 63 messages); the sound without the focus (`QUEUE-ARCHIVE.md` 2026-10-27, E24 M2 T6a2:
    the rest read, `h3_map_view::music`'s fade without a pending track, the window by
    `Window::focused()`). Recorded: a walk with Space. Found at T6b2: a path's walk ends on a
    left button's press and on any key down (0x407FA7: the kinds 8 and 1), but `Loop::carry_on`
    takes only `Step`, `Scroll` and `Click` as the end: `Press` and `Key` (the hotkeys) belong
    there too, the release (0x10) dropped.
  - **Acceptance:** the new sequence as recorded, every frame, the status bar compared; the focus's
    sound by a test and in the window; the Definition of Done.
  - **Depends on:** T6b1 (done: the key table's additions). **Status:** ready (T6b1's #605 merged;
    T6b2's pull request touches the same `timeline.rs`: start from it once merged).

- 2026-10-30 (this lane, E24 M3 T6b2): taken whole as the user's T6b cut left it. Done: the
  log's "M3 T6b2". Its entry as it was:

  ### E24 M3 T6b2 — the underground, the minimap's click, the status window's click

  - **Scope:** the underground's button and key (the view's level, its DEF and the minimap's);
    a click on the minimap and the drag while held (`minimap_click` 0x40A0C0, 0x40A280, 0x40A509);
    the status window's click (0x409FD8: the kingdom's and the date's windows in turn). Recorded:
    sequences with them.
  - **Acceptance:** the new sequences as recorded, every frame, the status bar compared; the clicks
    and keys in the window; the Definition of Done.
  - **Depends on:** T6b1 (done). **Status:** ready once T6b1's pull request is merged (its
    timeline's hold, `Action::Press`, `Hotkey`, `Chosen`: the underground's key is U, its command
    `Unplayable::Command(4)` until then; a click on the minimap `Unplayable::Panel`).

- 2026-10-30 (the adventure lane, E08 T79a; for information, nothing to do): the harness's
  reads and steps for batch E9a (`clients/capture/README.md`, "Mines, Lighthouses and Shipyards").
  Behaviour that changed for every recipe: `h3lod.identify` also knows `MINEEVNT.TXT` (a mine's
  flag message, `MINEEVNT:<n>`; it was named GENRLTXT:4, its picture's caption) and lets a
  record's title braces be missing; a walk records the cursor before its first click (`cursor`);
  a battle's record keeps each side's owner and controllers (`sides`); the boat dialog (vtable
  0x643908) is a screen a step's `actions` play, cancelled when left open. New: `mine:x,y`,
  `tile:x,y`, `screens`, `opening:sides`, `opening:stacks:<side>`, `opening:slots:<side>`, `stops`,
  `battle = "auto again"`, `row:<column>#2`; `h3mem.mines`, `tile_word`, `battle_sides`,
  `auto_combat_on`. The harness check passed.

- 2026-10-30 (this lane, E24 M3 T6a2b): the agent proposed cutting T6a2b at the quests (T6a2b1
  without them, T6a2b2 the Seer's Huts and Quest Guards); the user kept it whole. Done: the
  log's "M3 T6a2b". Its entry as it was:

  ### E24 M3 T6a2b — the map hint's owners and contents

  T6 cut with the user (2026-10-30) in two: T6a the mouse-over, T6a cut again (2026-10-30): T6a1
  (done: the epic's log) and T6a2, cut with the user (2026-10-30) by what runs the hint: T6a2a, the
  35 classes of the visit marks (done: the epic's log, design §4.11), and T6a2b.

  - **Scope:** `map_hint` 0x40B150's other 20 cases (switch 0x40B20E): the Border Guard and Gate
    (9, 212) and the Keymaster's Tent (10: a colour's name 0x6A7520, a key's mark by the game's
    +0x4E364), the dwellings (17, 20: the owner's colour, CrGen1 0x6914FC and CrGen4 0x691354), the
    Lighthouse (42, its owner), the mine (53, 0x40D670), the resource (79, its name 0x6A5E64), the
    creature banks (16, 24, 25, 84, 85: 0x40D3F0), the Seer's Hut (83, 0x5741B0), the Shrines (88 to
    90, 0x40D8D0), the Tree of Knowledge (102, 0x40DB00), the Witch Hut (113, 0x40DCC0), the Quest
    Guard (215, 0x573040): their facts in `[hover]` (`h3mem.Mem.hover`), keys and families; read the
    sub-functions first (`scripts/exe.py fn`; each cited address a map entry). Situations with each
    class and sequences over them, as T6a2a's (`hover-visits-*`).
  - **Acceptance:** new sequences with the pointer over every class of the 20, each exact with the
    status bar compared; the Definition of Done.
  - **Depends on:** T6a2a (done). **Status:** ready.

- 2026-10-30 (this lane, E24 M3 T6a2a): the agent proposed cutting T6a2 by what runs the hint;
  the user agreed: T6a2a, the 35 classes of the visit marks (done: the log's "M3 T6a2a"), and
  T6a2b, the 20 of the owners and contents (queued); the recapture with the hint unmasked moved
  to T7. MCL-Q17 to MCL-Q19 answered at its start. T6a2's entry as it was:

  ### E24 M3 T6a2 — the map hint's own cases

  T6 cut with the user (2026-10-30) in two: T6a the mouse-over, T6b the clicks, the keys during a
  walk and the sound without the focus; T6a cut again (2026-10-30): T6a1 (done: the epic's log), and
  T6a2.

  - **Scope:** `map_hint` 0x40B150's 58 cases of their own (`hover::OWN_CASES` but 0, 3, 26, 36,
    34, 54, 98; switch 0x40B20E): the visited marks (the viewing player's, a selected hero's), the
    owners, the contents (a resource's amount, an artifact's or spell's name, a dwelling's
    creatures, a keymaster's colour, ...), each one's facts in `[hover]` (`h3mem.Mem.hover`) and its
    keys; the sub-functions it calls (`scripts/exe.py callees 0x40B150`). Then the status
    bar's hint unmasked everywhere (`masks.toml`'s `hint` into `panels`, `hinted` gone): every
    situation and sequence captured again (`set`, `frames`), as M3's closure would.
  - **Acceptance:** new sequences with the pointer over every class with a case, each exact; every
    situation and sequence exact with the status bar compared; the Definition of Done.
  - **Depends on:** T6a1 (done). **Status:** ready.
- 2026-10-29 (this lane, E24 M3 T5b2): the agent proposed cutting T5b2 into the windows as stills
  and over time; the user kept it whole. Done: the log's "M3 T5b2" (the date's and the kingdom's
  windows, the chooser at every view's draw, a timed kind's end, the rebuilds). MCL-Q16 answered
  at its start (the original's income kept).
- 2026-10-29 (this lane, E24 M3 T5b1): T5b cut with the user into T5b1, a hero's and a town's
  window (done: the log's "M3 T5b1"; MCL-Q16 found), and T5b2, the date's and the kingdom's.
- 2026-10-29 (this lane, E24 M3 T5a): T5 cut with the user at the seam T4c's session proposed:
  T5a the resource bar's seven amounts, the date and a text's reach (done: the log's "M3 T5a";
  MCL-Q15 found), T5b the status window (queued).
- 2026-10-29 (this lane, E24 M3 T4c): a changed panel item drawn again alone, found by T4b's
  measurement and taken as a task of its own (not folded into T5); done: the log's "M3 T4c".
- 2026-10-29 (this lane, E24 M3 T4b): the lists over time, kept whole (the redraws M3's
  sequences meet: a step's end, a lift that centres the view, a walk's end that leaves the hero
  unable to move); done: the log's "M3 T4b".
- 2026-10-29 (this lane, E24 M3 T3b): the minimap over time, kept whole by the user when its
  reading grew (13 direct call sites and 20 through `minimap_redraw` 0x413790); done: the log's
  "M3 T3b".
- 2026-10-29 (this lane, E24 M3 T3a): T3 cut with the user into T3a (the minimap as a still,
  done: the log's "M3 T3a") and T3b (the minimap over time, queued).
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
- 2026-10-28 (this lane, E24 M3 T2b2): kept whole by the user ("no cut"; the cut proposed: the
  drawers' spans, the view's cells, the changed rectangles widened and uploaded), done: the
  log's "M3 T2b2".
- 2026-10-28 (this lane, E24 M3 T2b1): T2b cut with the user into T2b1 (the frame's fixed cost,
  done: the log's "M3 T2b1") and T2b2 (a frame redrawing only what changed, queued), the user
  adding a spatial structure for the draw objects (a jump and HotA's large maps) and the
  widening computed once; the queue's entry, verbatim:

  ### E24 M3 T2b — the frame's fixed cost (cut into T2b1 and T2b2)

  - **Scope:** cut from T2 with the user (2026-10-28; T2a, the panels right, done: the epic's
    log, "M3 T2a"): the frame's fixed cost (the inbox's, `QUEUE-ARCHIVE.md` 2026-10-27, E24 M2
    T4b3): the frame picture and the panels converted and drawn once a viewer, no screen copies, a
    frame redrawing what changed (a scroll or a walk only what moved; the buttons a walk sets);
    measured with `map-measure.cmd`'s "a frame made in" before and after on the user's machine.
  - **Acceptance:** every situation and sequence as recorded (`h3map.py drawn` all the same, the
    frame checks exact); a frame's fixed cost measured before and after; the Definition of Done.
  - **Depends on:** T2a. **Status:** ready.

- 2026-10-28 (this lane, E24 M3 T2a): T2 cut with the user into T2a (the panels right, done:
  the log's "M3 T2a") and T2b (the frame's fixed cost, queued); the queue's entry, verbatim:

  ### E24 M3 T2 — the adventure screen's dialog at a map's start (cut into T2a and T2b)

  - **Scope:** read `adventure_full_update` 0x417420's panels (the manager's +0x44) and the
    adventure dialog +0x50: its items in the order added and drawn, each one's id, place, picture
    or DEF and states; the buttons (`iam000` to `iam015`), the status bar (`AdRollvr.pcx`), the
    resource bar (`AResBar.pcx`), the status window's and the lists' frames as at a map's start;
    `h3_map_view::layout` gains the dialog (`h3_ui::ui`'s format), the screen drawn with it. The
    frame's fixed cost (the inbox's, `QUEUE-ARCHIVE.md` 2026-10-27, E24 M2 T4b3): the frame
    picture and the panels converted and drawn once a viewer, no screen copies, a frame redrawing
    what changed; measured with `map-measure.cmd`'s "a frame made in". The parked cursor drawn
    (the mask's). Recorded: the situations' screens with the panels; `panels` reduced to what
    T3 to T5 draw.
  - **Acceptance:** the static panels exact on the 40 situations and the 13 sequences; a frame's
    fixed cost measured before and after; the Definition of Done.
  - **Depends on:** T1a (merged, #545). **Status:** ready (T1b's pull request merged first, or
    `master` merged in after it: both touch `h3-ui`).

- 2026-10-28 (this lane, E24 M3 T1b): every key of both windows through the table (the user:
  no cut; the log's "M3 T1b"); the queue's entry, verbatim:

  ### E24 M3 T1b — the battle window's keys through the table

  - **Scope:** `h3-view window`'s keys (matched in its event loop, `window.rs`) as the battle
    viewer's commands through `h3_ui::keys`, the user's file (`--keys`, `keys.toml`: each viewer's
    commands in its own table, *provisional*); both windows' own keys the map's (the user, T0: F2
    music, F3 effects, F5 pause, F6 the next frame, F7 restart, F8 vsync, F9 the cursor's kind,
    F11 fullscreen, Esc quit; the battle's P, →, Home, M, E, F, V and H gone); the battle's own
    keys (the log's Up and Down, the book's; since E17 M5 T4a, #542, Retreat's R and Surrender's S
    and the message box's, whose Escape is its Cancel and never the window's quit while a box, the
    book or an aim is open; while a box is open every key but the viewer's own goes to it:
    `viewer_key` in `window.rs`, the set this task replaces) and `h3_sdl::Key` as they are,
    additively. The client lane's inbox told (its M5 T3b's F key then free).
  - **Acceptance:** every key of both windows through the table; `h3-view window --check` and a
    sequence in the window as before; check.sh green; the Definition of Done.
  - **Depends on:** T1a (built 2026-10-28, its pull request open). **Status:** ready once T1a's
    pull request merges.

- 2026-10-28 (this lane, E24 M3 T1a, merging `master` after #542): the client lane's inbox entry
  below, taken: T1a's part merged into the move (`h3_ui::ui::Draw`'s `size` and `player`,
  `h3_view::message` on `h3_ui::{ui, text}`, `text::pack` carried by the rename), T1b's into its
  scope (`viewer_key`, R, S, the box's keys); verbatim:

  - 2026-10-28 (the client lane, E17 M5 T4a; **for you at E24 M3 T1a and T1b**): T4a's pull request
    (`client/e17-m5-t4a`) changes what T1a moves into `h3-ui` and what T1b takes through the key
    table. `ui`: a picture item draws at most its `size` of its picture (the executable's
    `dialog_picture_draw` 0x450450), a DEF item takes a player's colours from `Shown::players`
    (`Draw::Picture`'s new `size`, `Draw::Button`'s new `player`); the new `h3_view::message` (the
    message box laid out as `ui` items) is built on `ui` and `text`; `text::pack` makes each run of
    quotes in a table's field one (the executable's reader, 0x5BBD29). `h3_render::text` gained
    `widest_word`, `widest_line`, `wrapped_width` and `blit` `draw_image_keyed_in_clipped`
    (additive). The window: R and S open Retreat's and Surrender's questions through the panel's
    buttons, and while a box is open every key but the viewer's own goes to it (`viewer_key` in
    `window.rs`: P, M, E, F11, V, H, the set T1b replaces). Design §7.19.

- 2026-10-28 (this lane, E24 M3 T1a): the common crate `h3-ui` built (the `shared/` pull request
  #541 for `lanes.toml`, then the move, `cross-lane`); the queue's entry, verbatim:

  ### E24 M3 T1a — the common crate `h3-ui`

  - **Scope:** first a `shared/` pull request: `lanes.toml`'s client foundation gains
    `clients/crates/h3-ui/` (its comment: both client lanes', as `h3-assets`); the global HANDOFF's
    "Common" line if it names the crates. Once merged, on `map-client/e24-m3-t1a` (`cross-lane`,
    the user's consent at T0): the crate (pure, `no_std` + `alloc`; `h3-map-view`'s
    `tests/purity.rs` with R2-15's fixes, the client lane's inbox) with `h3-view`'s `clock`,
    `cursor` and the `Sprite` it draws, `ui` (its draw items its own: picture, button, text, label,
    darkened box; `h3-view`'s scene and compositor take them), `text` with its `keys`, `message`,
    `pack` and `plural`; `h3-map-view`'s `keys` (generic over a viewer's command type: the chords,
    the user's file; the map's commands and defaults stay in `h3-map-view`) and `music`. Both
    viewers import them; `h3-map-view`'s `clock`, `cursor` and `Sprite` gone; `clients/README.md`'s
    crate list; the client lane's inbox told (its imports changed). Nothing drawn or played
    changes.
  - **Acceptance:** both pull requests merged; `clients/scripts/check.sh` green, coverage 100 %;
    `h3map.py drawn` and `h3drive.py drawn` all the same; no copy left in either viewer.
  - **Depends on:** T0 (done 2026-10-28). **Status:** ready (the `shared/` pull request merged by
    the user before the move's).

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

### E24 M4 T2a — the finds told

- **T2a, the finds told** (an OK alone): the adventure's message box with pictures
  (`message_box_pictures` 0x4F7690, `message_with_pictures` 0x429110: up to eight, their
  captions, the picture types the engine's finds give; its sounds, its place), the text through
  the catalogue, for a find with nothing to answer (a pile with its message, a Campfire's or a
  Windmill's resources, an artifact, an event's or a sign's text, a Learning Stone's experience);
  the status window's kind 6 (a resource found, 0x451220, 5 seconds) and kind 7 (a text,
  0x451820). The turn's boxes T1a left: the scenario's tests' "you have lost your last town" at a
  turn's end with no town (the player's flag, picture type 10; its text starts with an empty
  player name in the harness's game: whether a menu-started game names the player, read first)
  and `player_turn_start`'s days left (ArrayTxt 128, 129; design §5.1). **Depends on:** T1a
  (done: `h3_ui::message`, styles 1 and 2 without pictures). Cut again at its start (2026-11-17,
  the fallback the user agreed with the cut): **T2a1**, the reading and the recordings
  (`map-client/e24-m4-t2a`; design §5.3, the sequences in `clients/map/to-record.toml`), then
  **T2a2**, the viewer: `h3_ui::message`'s pictures (the client lane's `h3_view::message`'s
  `pictures` and its picture slots copied, as T1a did), the finds' boxes composed from their
  facts (the map, the live part's entrance words and hero, the catalogue's records: ADVEVENT,
  ARTEVENT, RANDSIGN, GENRLTXT, ArrayTxt), kind 6 and the pile's dissolve (an object taken: the
  view drawn without it, `h3_render::Screen::dissolve_rect`), the turn's two boxes (CLT-Q100
  (b): a single scenario's player named, the original's empty name behind `--bugs original`),
  the sequences moved into `sequences.toml`. **Kind 7** (a text) is a network game's message
  alone (`status_text_set` 0x416210): not M4's.
- **Status:** done 2026-11-18 (T2a1 2026-11-17, the log's "M4 T2a1"; T2a2 whole, the user kept it uncut: the log's "M4 T2a2").

### E24 M4 T1b2 — a computer player's moves in sight, played by the viewer

- **Scope:** T1b cut in two by the user at its start (2026-11-17); T1b1 read and recorded it
  (design §5.2, the log's "M4 T1b1"). The viewer plays the five `ai-*` sequences of
  `clients/map/to-record.toml` exactly: a computer player's turn after End Turn from the
  `[[turn]]` facts (kind 5 built, the view centred on the hero, the cursor hidden from its first
  seen step) and the timeline (its walk from the `hero` and `walk` lines as M2's, at the
  sequence's `computer_speed`; kind 5's step by §5.2's rule, checked against the `other` lines;
  `blackout`), the next day's heroes from the `[[turn]]` tables' `heroes`; the minimap's shield
  and the blank buttons of a computer's turn read first (`covered_view_redraw` without Blackout,
  the minimap's draw in a computer's turn); speed 4's single draws on the client's clock; each
  sequence then moved into `sequences.toml` (`to-record.toml` removed once empty), `drawn
  --record`; MCL-Q30's sound run; MCL-Q31 built as the original's.
- **Acceptance:** as T1a's.
- **Depends on:** T1b1 (done 2026-11-17). **Status:** done 2026-11-17 (whole; the log's "M4 T1b2").

### E24 M4 T2b1 — the questions and choices, read and recorded

- **T2b, the questions and choices** (an answer): Yes/No with pictures for the engine's offers
  (`h3_adventure::OfferKind`: a guard's passage, a monster's join free or for a price, its
  flight, a Cartographer's price; a guarded pile's, artifact's, scroll's, mine's and bank's
  question; the Tree of Knowledge's price, a search (the Warrior's Tomb, a bank), Pandora's Box,
  the Pyramid, a dwelling's hire and guards questions) and the choice box for its choices
  (`Choice`: a chest's gold or experience, the Arena's and a School's two skills; the pictures
  chosen by a click, the hidden frame a choice's mark), the answer the harness gives. Not the
  Shipyard's boat dialog and the recruit dialog (screens of their own: M5). **Cut in two by the
  user at its start** (2026-11-18, as T2a was; recommended was three, the choice box apart):
  - **T2b1, read and recorded:** every question's and choice's box read (its style, text,
    pictures, sounds, the answer's effect on the screen up to the box's close), the harness
    giving the answers, the sequences recorded twice into `clients/map/to-record.toml`. **Status:**
    in progress (`map-client/e24-m4-t2b1`, 2026-11-18T08:00Z).
  - **T2b2, the viewer:** the viewer plays T2b1's sequences frame-exact (a cut offered at its
    start, as T2a2's). **Depends on:** T2b1.
  **Status:** done 2026-11-18 (the log's "M4 T2b1").

### E24 M4 T2b2 — the questions and choices, played by the viewer

  - **T2b2, the viewer:** the viewer plays T2b1's 22 sequences frame-exact, each then moved into
    `sequences.toml` (`to-record.toml` removed once empty), `drawn --record`: style 2's Yes/No
    with a picture, styles 7 and 10's choice (the frames, the OK disabled until a pick, a click
    on a picture, the "or"), the texts through the catalogue (ADVEVENT's and GENRLTXT's records
    the design's tables name; the guarded artifact's GENRLTXT 421 from the guards), the answers'
    ends (a join's and a flight's dissolve with `killfade`, a Border Guard's and Pandora's with a
    pickup sound, the Cartographer's reveal, the army and skills in the status window). MCL-Q33
    built as the original's. A cut offered at its start (as T2a2's; a seam: the Yes/No questions
    first, the choices after). **Depends on:** T2b1 (done).
  **Status:** done 2026-11-18 (whole: the user kept it uncut at its start; built in three parts by
  three workers in turn; the log's "M4 T2b2").

### Inbox notices moved at E24 M4 T2b2 (2026-11-18): informational, nothing to act on

- 2026-11-17 (the client lane, E27 M2 T8, `client/e27-m2-t8`; **the foundation's API, nothing of
  yours breaks**): `h3_sdl::KeyScan` loses `code` and `h3_sdl::set1` is gone (nothing read them
  since T7b); `KeyScan::shift` and `Window::scans` are unchanged. `h3_ui::keys::Command` gains a
  provided method `asked_together(a, b)` (a context alone by default, so your table needs no
  change): override it if one key down of yours asks several contexts in turn, and a chord bound
  in two of them is then refused by the key file's check.

- 2026-11-15 (the server lane, E16 M5 T24, `claude/adoring-fermi-l0mvur`, `cross-lane`; **a
  protocol version taken; for when a seat of yours joins a whole game**): protocol **v68**
  (after the town lane's v67; SRV-Q38, 2c's reviewer: a seat watched without agreeing).
  `GameJoined` gains `spectators` (whether the game may be watched, as the lobby lists it:
  taking the seat agrees to it), and the new numbered `GameSpectators { n, open }` tells
  every player but the host the host's `SetSpectators` before the start (the host still gets
  `SpectatorsSet`), so a seat that does not agree may leave before the start: show both if you
  show the lobby's flag. On a server that requires logins a `GameWatch` now needs one
  (`NotLoggedIn`), as a battle's spectator's `Join`. Nothing of yours breaks (your matches
  have wildcards); `GameSpectators` takes its number in a seat's record.

- 2026-11-14 (the server lane, E16 M5 T21a and T22a, `claude/festive-albattani-mdvj1a`,
  `cross-lane`; **a protocol version taken; for when you show a whole game to someone who does
  not play it**): protocol **v66**. The user (SRV-Q36): a whole game's spectator sees **every
  player's view**, live, only in a game its creator opened before the start (closed by
  default), never a ranked one. On the wire: `CreateMapGame`'s `spectators: Option<bool>` (left
  out: closed), `GameWatch { game }` answered `GameWatching { game, manifest }`, then each
  player's messages wrapped as `GameWatched { n, player, message }` (`n` the spectator's own
  number, `message` exactly as that player's seat got it, with the player's own `n`); a watch
  starts with each player's `GameSnapshot`, its `GameClock` and its `GameTurn` as a re-taken
  seat gets them, numbered as the player's last message. A client folds each player's stream
  as it folds a seat's (`GameObservation::fold`), one view per player. `SetSpectators` works
  in a whole game (the host's, until the start: `GameReject::NotHost`, `AlreadyStarted`), the
  lobby's `MapGameInfo::spectators` says whether a game takes them. `h3-server` hosts them
  since T22b (2026-11-14, still v66: a paused game is listed taking none and refuses a watch
  `NoSpectators`; a spectator hears the pause, a void and the stop as the seats do). Nothing to build until you want a spectator's screen.

- 2026-11-14 (the server lane, E16 M5 T20, `claude/sweet-gauss-ngra43`; **a protocol version
  taken**): protocol **v65** (after the town lane's E09 T28a's v64): a whole game is void when the server's own
  commands in one of its battles reach `[limits] system_commands_per_battle` (10 000) with the
  battle still open (a battle both of whose sides the server's computer plays, which it did not
  end; SRV-Q35). Its connections hear `Goodbye { CommandLimit }` (until now a battle's alone), its
  journal goes to the void, it leaves the lobby; an in-process `GameSession` that is void
  (`GameSession::void`) rejects every command `GameReject::NotStarted {}`. No new message or
  field. Show the goodbye's reason for a whole game too if you show it; nothing else to build.

- 2026-11-14 (the client lane, E27 M2 T3a, `client/e27-m2-t3a`; **for your information, an
  addition to `h3-ui`, nothing changed**): `h3_ui::input` (`Input`, `Held`, `Did`), a dialog's
  press, hold and release of its DEF buttons and DEF pictures (boxes, bars) and its button held by
  a key, the right release ending a button's own loop, the item a right press's help names; the
  original's profile (`Bugs::Original`) clicks a picture at a release anywhere, the client's over
  it alone (E27 decision 20, CLT-Q102). The battle's combat options use it; the shell's dialogs
  will. Use it for a dialog of the map's if it fits.

- 2026-11-14 (the server lane, E16 M5 T19, `claude/gracious-maxwell-gu6rjb`; **a protocol
  version taken**): protocol **v63** (after T18's v62), 2b's reviewer's findings:
  `GameReject::WaitingForLevelUp {}` no longer names whose level-up holds the game (SRV-Q31:
  show only that the game waits, as `GameClock`'s `waits`); the lobby's `wire::game::MapGameInfo`
  gains `clock` (`OfferClock`, the game's own: the offer's at its creation, which a restart keeps
  whatever the offer's is now; show it for a listed game rather than its offer's); a seat whose
  turn or battle side's chess clock ran out gets no battle commands in its `GameTurn` (the
  computer plays the side at once); the join that starts a game now tells every player its clock
  as it runs (before, the first turn's timer looked stopped until the next reading). Nothing else
  to build.

- 2026-11-13 (the server lane, E16 M5 T15, `server/e16-m5-t15`; **for showing `GameClock`,
  nothing to do before it**): the original's turn timer is read in `docs/epics/E16-timer-survey.md`:
  the warnings' schedule and texts in the adventure screen's chat line (§3: "<SYSTEM>:" with
  GENRLTXT 627 to 630, at a quarter elapsed, a quarter left, a minute, ten seconds, then each
  second), the end's box GENRLTXT 202 (§4), the hot seat's covered "your turn" box outside the
  turn's time and the network one inside it (§8).

- 2026-11-14 (the server lane, E16 M5 T18, `claude/awesome-sagan-lue93w`; **a protocol
  version taken**): protocol **v62** (after the adventure lane's T115b's v61): the server bounds a whole game's seat's commands
  (`session::game::CommandBounds`, `GameOptions::commands`; `[limits] commands_per_turn`, 1 000,
  and `commands_per_battle`, 4 000). Past its turn's bound a seat's map commands (an adventure's,
  a management one, a town's) are rejected `GameReject::TurnLimit {}`, while ending the turn,
  answering a decision and battle commands stay accepted; past its battle's bound the server's
  computer plays its side for the rest of the battle (the system's `SystemCommand::Battle`
  events, as for a side whose chess clock ran out) and its own battle commands are rejected
  `GameReject::BattleLimit {}`. `GameTurn` leaves out what a bound refuses (no `GameTurn` when
  nothing is left). Show the two rejections if you show rejections; nothing else to build.

- 2026-11-14 (the server lane, E16 M5 T17, `claude/admiring-lovelace-jlv0hu`; **a protocol
  version taken**): protocol **v60** (after the town lane's T27's v59): a map offer shows how its games are timed,
  `wire::game::MapOffer`'s `clock` (`wire::game::OfferClock`: `turn_ms`, none for unlimited, an
  offer's one of the original's 1, 2, 4, 6, 8, 10, 15, 20, 25 or 30 minutes; `level_up_ms`, none
  when the game waits for a level-up; `battle_ms` and `increment_ms`, each side's chess clock in a
  battle between two players' heroes under a limited turn). The server's whole games are timed
  by their offer now (`[[map]] turn_minutes`, 60 s a level-up by default), so `GameClock` is told
  with running timers; T16's note above stands. Nothing to build unless your lobby shows the
  offer's clock (it decodes the new field with the crate).

- 2026-11-13 (the server lane, E16 M5 T16, `claude/charming-edison-866oew`; **a protocol version
  taken**): protocol **v58** (after the adventure lane's T115a's v57), the whole game's clock: `ServerMessage::GameClock { n, clock }`
  (`wire::game::GameClock`: the player's own `turn` timer, its battle's chess clocks `battle`
  against another player's hero (`Duel`'s attacker and defender), its own `level_up` time, and
  `waits`, the game waiting for another player's level-up), each `Timer` the milliseconds left
  when sent and whether it runs (count it down locally; a battle side at 0 is the server's
  computer's for the rest of the battle). Numbered and recorded: after a command's other
  messages and before the `GameTurn`s, alone when a timer ran out, and after a `GameSnapshot`
  that leaves something to tell. In an untimed game (every game until the server lane's T17)
  only `waits` is ever told. At a timer's end the server acts for the player with the system
  actor's commands (T14's `GameCommand::System`), told as every command's. Your readers'
  wildcards skip it; showing it is yours (E28's screens).

- 2026-11-13 (the client lane, E27 M1 T2, `client/e27-m1-t2`; the capture harness, **rerun
  `setup.sh` in a fresh sandbox as before**): `xdd.dll`'s ack has 13 fields (a stale `xdd.dll` in a
  run directory fails every run: rerun `setup.sh`); the run's `VIDEO.VID` joins `battle-videos.vid`
  with `shell-videos.vid` when both are staged (the start's clips play only with `Show Intro` on,
  which the harness's settings keep off); `SMACKW32.DLL` reads the virtual clock too and
  `BinkGetSummary` is guarded (a full-screen clip's end on a frozen clock). `h3drive.Game` takes
  `frozen` (`H3FROZEN`). New: `h3shell.py`, the shell's screens (the runbook's "The shell's
  screens"). The harness check passed.

- 2026-11-12 (the server lane, E16 M5 T11, branch `claude/vibrant-albattani-ml282o`; **for online
  play, nothing to do before it**): protocol **v50**, the game's queries: a seat sends `GameQuery {
  seq, query }` and gets `GameAnswer { n, seq, answer }`, unnumbered (`n` the number of its last
  message) and not recorded, so your record readers never meet it; a hero's moves come as
  `GameAnswer::Moves { moves: Option<wire::game::Moves> }`, a path tree read with `Moves::tiles`,
  `points_left` and `path` (the engine's `Reach`'s), the path one `MoveHero` takes; a town's hall as
  `GameAnswer::TownOffer` (`possible`, `buildable`, `payable` bit sets). A whole game's start now
  tells a player its town events of the build (day 1's of the first group) in one `GameEvents` after
  its `GameSnapshot`, before its turn (record format **v5**).

- 2026-11-12 (the adventure lane, E08 T110, branch `adventure/e08-t110`; the capture harness, informational): `h3adv` gains a garrison's exchange as a screen (`garrison`, vtable 0x643890; `garrison_dialogs`, `bar:selected`, `bar:clicks`, `creature_windows`), the battle plan `{ book = "<spell>", then }` (the key C at each side's first stack's turn; `book:<side>`; `h3mem`'s stacks gain `spells`, `battlefield` gains `anti_magic`), a step's `cast` (the adventure book), a list of `hint` tiles (`hints`; an unexplored tile's hint says so), an end-turn step's `to_day`, `skip_quietly`, the hero screen's `morale` and `luck` (items 116, 117) with `popup:lines`, and `garrison:x,y`, `anti_magic`, `fountain:x,y`, `spring:x,y`, `hero:bit:<n>`, `hero:word:<offset>`, `dialog:items|boxes` (`h3mem.garrisons`, `battle_hero`). **Behaviour changed:** `h3drive.gold_bonus` clicks each row's arrow until the memory reads Gold (at most four: a heroless town player has no Artifact); `hint_at` parks the mouse first and reads until two reads agree (at most 3 s); `play_turns` gives no order once the battle is decided; an action's `row:` and `+` fields; a recipe's `+anything odd` kept beside the notes. The harness check passed. `clients/capture/README.md`, "Garrisons and the bonus visitables".

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

- 2026-11-07 (the town lane, E09 T14b, branch `town/e09-t14`; the capture harness, informational):
  `h3adv` plays a walk's `actions` on the town screen when it opens (then closes what they left
  open), with the new actions `hall = true` (a click on the shown hall building, items 10 to 13: the
  hall, vtable 0x6437A0) and `slot = <building id>` (the hall's slot showing it: the building's
  dialog, 0x643944; `click = ["build"]` its Build 30722, `cancel` 30721), and the reads
  `town:x,y:mark`, `hall:slot:<id>`, `build:button`, `build:text`, `screen`, `town_screen_closed`;
  `h3mem`'s town record gains `mark` (+0x2), `h3lod` gains `building_names`. A walk without
  `actions` behaves as before. Learned: the panel's hall picture opens nothing; a map whose players
  are all allies ends at a visit's close (an enemy is needed). The harness check passed after.
  `clients/capture/README.md`, "The hall and the build".

- 2026-11-07 (the adventure lane, E08 T100b, branch `adventure/e08-t100b`; the capture harness, informational): `h3adv` gains a `key` step's `repeat = { most, first, until = { with } }` (the key pressed again and again, a row each: `presses`, `presses:<columns>`, `presses:changed`), a recipe's `[places]` (`hero:place`, `hero:at`), a step `look = "<hero>"` (the view centred by the minimap, nobody selected; its `before:` reads before the scroll), `walk = "off"` (the first open neighbour), `only_if`'s `empty` and `occupied`, and the observations `whirlpools` (`h3mem.whirlpools`: game +0x4E77C's vector, begin and end at +0x4E780), `tile:x0,y0..x1,y1`, `tile:from`, `tile:here`, `tile:hero[<name>]`, `answers`, `selected`, `target`. No behaviour of an existing step changes. Observed: the selected hero is lifted off its tile's record after its move and at its turn's start, and a scroll of the view puts it back (a tile read depends on the last scroll). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".

- 2026-11-06 (the adventure lane, E08 T100a, branch `adventure/e08-t100a`; the capture harness, informational): `h3adv` gains a step's `replay` (Replay Opponent Turn: the panel's Adventure Options button, the map dialog's item 9; its dialog vtable 0x63A610, item 5; played on the frozen clock, stepped 20 ms, the flag 0x696A04; `dumps = "every"`) and `at_opening` (reads as a step's first battle opens), `only_if = { at = [x, y] }`, `again_if = { marks = "any" }`, and the observations `replay`, `replay:views`, `replay:watched`, `replay:options`, `view:before`, `view:after`, `view:size`, `explored:x0,y0..x1,y1`, `at_opening:`, `battles`, `exchange`, `army:rows`, `hero:in_boat`. Behaviour changes: a `key` step plays what the key brings by the step's `answer`, `battle` and `level_up` (it stopped at a battle before); a skipped step leaves a field another step filled (it wrote "not played" over it). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".

- 2026-11-03 (the adventure lane, E08 T90, branch `adventure/e08-t90`; the capture harness, informational): `h3adv` and `h3mem` gain the dwellings' records (`dwelling:x,y`, game +0x4E39C), `teams`, `objects:` and `access:` (a rectangle's map objects with class and subclass, its tiles' blocked and visitable bits), `camp:x,y`, the hire dialog as a screen (vtable 0x640C4C; `hire`, `hire:opening`, `hire:after`; actions `click = ["max", "recruit"]`, `amount = <n>`), a step's `hint = [x, y]` and `popup = [x, y]`, `opening:defender`, `ai_turns`. Behaviour change: `end_turn` now waits out a computer player's turn (it returned during it); nothing changes on maps with humans only. `clients/capture/README.md`, "Dwellings, the hire dialog and a computer player's turn".

- 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
  unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
  `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
  `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
  30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
  in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
  its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
  side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
  --map` fills the template beside the map when its stamp matches (no cargo build). A town's
  visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
  after. `clients/capture/README.md`, "Towns (E09 T3b)".

- 2026-11-02 (the client lane, E17 M6 T2a1; for information, nothing to do): additions to the
  common crates for the battle's stack stats dialog (`docs/epics/E17-design.md` §12.1):
  `h3_render::blit::draw_frame_clipped`; `h3_ui::ui::Def`'s `group`, `creature` and `frames_of`
  (serde defaults: a layout without them reads as before) and the
  `ui::Draw` variants `Creature` (a creature DEF item, the flags 0x12: the shadowed drawer, its
  canvas placed by group 2's first frame, cut to the item) and `Shared` (another DEF's frame in
  the item's DEF's colours: the executable keeps frames by name, CLT-Q36), both drawn by
  `h3_ui::paint` (`names`, `draw`, `reach`; `creature_frame`). Every existing draw is the same;
  a `match` on `ui::Draw` needs the two arms.

- 2026-11-02 (the battle lane; for your information): `h3_formats::text::TextTable` now makes
  each run of quotes in a quoted field one, as the executable's text-table readers do
  (`installation.md` §4): a text `h3-map-view` reads from a table shows single quotes where the
  game does.

- 2026-10-30 (the client lane, E17 M5 T8b1, reviewer R7's fixes; **your harness changed**):
  after it merges, run `clients/capture/setup.sh` again for every run directory you use: the
  hook DLL's ack has ten fields now, and an older `xdd.dll` stops a run with "stale xdd.dll".
  `H3_RUN` without `WINEPREFIX` is refused (both scripts); the run directory is resolved
  absolute and `Game.start` fails unless the prefix's `d:` points at it. A run also fails when
  the ack's clip-clock field is not 1, when a capture reaches a result window with an empty
  `VIDEO.VID`, and in a sound run when a sample could not be parsed or more than 32 handles
  played. Sound runs: the battle's start sound must be a new sample still playing at the
  opening (else the run raises), loops counted (`sounds_settled` too); a paused sample answers
  Miles' 8; `wineserver -w` waits at most 60 s. Nothing else drives the original otherwise
  (runbook, "Sound", "The clips", "Traps").
- 2026-11-18 (this lane, E24 M4 T2c): kept whole by the user (the cut after the reading and the
  recordings as the fallback, not needed); done, the log's "M4 T2c". Its entry:
  - **T2c, a level-up's choice:** the level-up dialog (0x63FE60, `level_up`; the engine's
    `DecisionView::LevelUp`): the primary skill gained, one or two secondary offers, its own layout.
    **Depends on:** T2a.
- 2026-11-18 (this lane, E24 M4 T2d1): T2d cut in three by the user at its start (by family:
  T2d1 a flag raised, T2d2 a boat boarded and left, T2d3 a spell cast on the map; "a monster
  beaten" moved to T5); T2d1 done, the log's "M4 T2d1". T2d's entry before the cut:
  - **T2d, the map changed by a visit:** an object taken or a monster beaten (the map, the minimap
    and the panels redrawn); a flag raised (the owner's colour on the object and the minimap); a
    boat boarded and left; a spell cast on the map recorded (MCL-Q14: if the stale mana bar shows,
    the client shows the slot whole, the original's behind `--bugs original`). **Depends on:** T1a.
- 2026-11-19 (this lane, E24 M4 T2d2): T2d2 done whole, the log's "M4 T2d2" (design §5.7;
  MCL-Q35 new). Its entry before:
  - **T2d2, a boat boarded and left:** the step onto a boat (`hero_boards_boat` 0x4A1010: the
    hero's picture becomes the boat's, the boat taken off the map, the day's points) and off it
    onto a coast (`hero_lands_from_boat` 0x49E670, the landing in `hero_move_step` 0x480949 to
    0x480B0C: the boat put on its water tile, facing the hero's), their sounds and the panels;
    M2 left both out (design §3.5). **Depends on:** T1a.
- 2026-11-19 (read at E24 M4 T2d2: merged with `master`, nothing to do; the adventure lane, E08 T128, `adventure/e08-t128`, `cross-lane` with the user's
  consent; **two lines in your files, nothing to do**): `h3-map-view`'s `situation.rs` and
  `art/tests.rs` writer `Map` literals gain `banned_artifacts: Vec::new()`
  (`h3_map::writer::Map::banned_artifacts`, E08 decision 375). The writer also writes the seven
  trade classes as `Plain` objects (`TRADE_OBJECTS`; 213 and 221 refused in RoE) for your later
  situations.
- 2026-11-19 (this lane, E24 M4 T2d3): T2d3 done whole (Summon Boat, the user's choice at its
  start), the log's "M4 T2d3" (design §5.8; MCL-Q14 seen); with it T2 is done. T2's entry as it
  stood:

  ### E24 M4 T2 — a visit's results (cut in four by the user at its start, 2026-11-17)

  Read and build what the original shows when a visit or an event changes the game, the live part
  from its memory, for the finds and questions the engine's visits make; new sequences recorded in
  the original, the harness giving the answers. **Acceptance** of each: as T1a's. Cut by dialog
  family (the user, 2026-11-17: the recommended cut; a part that grows is cut again into its
  reading and recordings, then the viewer's build, as T1b was).

  - **T2a, the finds told:** done 2026-11-18 (its entry in `QUEUE-ARCHIVE.md`; the log's "M4 T2a1",
    "M4 T2a2"). Its pictures (`h3_ui::message`), `finds` and the timeline's boxes are T2b's base.
  - **T2b, the questions and choices:** done 2026-11-18 (T2b1 and T2b2; their entries in
    `QUEUE-ARCHIVE.md`; the log's "M4 T2b1", "M4 T2b2"). `asks`, `Question`, `Answer`, `Reply` and
    `h3_ui::message`'s styles 7 and 10 are T2c's base.
  - **T2c, a level-up's choice:** done 2026-11-18 (whole; its entry in `QUEUE-ARCHIVE.md`; the log's
    "M4 T2c"). `levels`, `LevelUp` and the timeline's `Loop::level_up` are T5's base for a battle's
    levels.
  - **T2d, the map changed by a visit:** cut in three by the user at its start (2026-11-18, by
    family, each read, recorded and built in one task as T2c was; "an object taken" was T2a2's and
    T2b2's; "a monster beaten" moved to T5, its recording needing a battle):
    - **T2d1, a flag raised:** done 2026-11-18 (its entry in `QUEUE-ARCHIVE.md`; the log's "M4
      T2d1"; design §5.6). `Found::flag`, the owners in memory and `Minimap::owners` are T2d2's and
      T4's base.
    - **T2d2, a boat boarded and left:** done 2026-11-19 (whole; its entry in `QUEUE-ARCHIVE.md`;
      the log's "M4 T2d2"; design §5.7). The boarding's and landing's dissolves and the boat lifted
      or put back are T2d3's base for Summon Boat and Scuttle Boat, and T4's for a record's boats.
    - **T2d3, a spell cast on the map:** a cast recorded (MCL-Q14: if the stale mana bar shows, the
      client shows the slot whole, the original's behind `--bugs original`); the spell book is M5's,
      so the recording needs a way past it (the harness's cast, or the frames under the book left
      out); a spell whose result the map shows (Summon Boat, Dimension Door, Scuttle Boat or View
      Air/Earth's screen: chosen at its start). **Depends on:** T1a; T2d2 for a boat's spells.
