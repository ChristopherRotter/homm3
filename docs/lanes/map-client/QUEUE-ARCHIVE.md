# Map client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/map-client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

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
