# F6 — what F1 to F4 left: findings

Covered: the binary's `window.rs` 720 to its end (the renew block, the due frame,
the redraw, the report, `Texture::describe`, `compare`, `summary`, the tests' claims);
`h3_ui::paint::reach` and `text_reach` against `paint::draw`, `h3_render::text::{draw_text,
draw_line, glyph}`, `h3_render::blit::{draw_frame_rows, darken}`, `h3_assets::fnt` (rows =
height); `h3_ui::cursor::Cursor::{draw, rect}` and the painter's `frame_into` cut; `h3_ui::ui`'s
`Dialog::draws` and `shadow_draws` (with M3's diff against e8a7990a's `h3-view/src/ui.rs`) and
their tests; `hover.rs`'s `NOT_ENTERED`, `OWN_CASES`, `Hover::{hint, contents, visits, kind}`.
Checked in the executable: `object_classes_not_entered` 0x63AA58 (42 u32, exactly
`NOT_ENTERED`, in its order); `tile_cursor_kind` 0x40E280 whole (exactly `Hover::kind`, frames
5, 0x1C, 6, 9, 4 and the shipwreck's 85); `map_hint`'s switch: the bound (class > 0xD7 → the
name, 0x40B1FB) and the index table 0x40D314 (216 bytes: the 62 classes with a case of their own
are exactly `OWN_CASES`, 212 sharing 9's case 2), the cases Corpse 22 (0x40B622: the selected
hero required, the screen player's +0xC0 bit word & 0x1F), Pyramid 63 (0x40C617: the hero's
owner +0x22, 0 to 7) and Water Wheel 109 (0x40CDF4: `current_player`'s bit 5 + p, then
(word & 0x1F) × 500 tested in 16 bits): all as design §4.11 and `h3mem.VISIT_RULES` give them.

Main question of fraction 2 (can the cursor or a text paint outside the rectangle the redraw
keeps?): no. `draw_frame_rows` draws only canvas pixels (mirrored or not), which is `reach`'s
button rectangle and `Cursor::rect`; the cursor is drawn into a cut of the composer's screen, so
it cannot touch pixels outside its rectangle at all; a text's rows are bounded by `draw_line`'s
height test (every glyph has `font.height` rows), its columns by the skip of glyphs starting left
of the box and the stop at the first passing its right edge, so only negative spaces pass the
box, which `text_reach` adds (exact while a glyph's `width + right >= 0`, which I did not check
on the installation's fonts; `smalfont.fnt`'s overhangs are 1 at most). Only the tests do not
show it (F6-3). Elsewhere: `window.rs` past 720 holds nothing beyond F2-1, F2-5 and F6-5 (the
look's renewals, a hardware cursor SDL refuses, the due frame's reuse all traced right);
`Dialog::draws`' M3 additions (a picture or button DEF given, a button's player) are right and
tested; only `shadow_draws` (F6-1).

Not covered: `tables.rs`'s sections F1 left (not reached); the hint's other 50-odd case bodies
in the executable (Corpse, Pyramid, Water Wheel and the cursor's kind only; the remaining
ones rest on design §4.11–§4.12 and F3's check of the code against it); `Replay::after`'s
600 s look-ahead (`LOOK_AHEAD_MS`) on an idle map, its cost and its interplay with input given
behind the replay's clock (pre-M3, not traced); `gate.rs`'s pre-M3 parts.

## F6-1 A dialog's shadow overflows into a panic on a crafted layout

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-ui/src/ui.rs:568` (`Dialog::shadow_draws`)
- **What:** `shadow_draws` adds and subtracts with plain `+`/`-` (`x + w`, `y + 9`, `h - 9`,
  `w - 2`, `y + h`), while `draws` saturates the items' places beside it. `Dialog` deserializes
  `shadow` (`#[serde(default)]`) for every dialog of the map's `adventure.toml` (`[dialog]`, the
  panels', the resource bar's), and nothing in the map viewer's layout check bounds it (no
  `shadow` anywhere in `h3-map-view/src`). F1-1's class (the workspace's release build checks
  overflow), at a site F1-1 does not list.
- **Failure scenario:** `<map>/layouts/adventure.toml`'s `[dialog]` with `place = [8, 0]` and
  `shadow = [2147483647, 600]` → `x + w` overflows in the first scene → the viewer panics
  instead of refusing the layout.
- **Fix:** saturating arithmetic there (as `draws`), or refuse a `shadow` the adventure screen's
  dialogs never have in the layout's check. No drawn frame changes.

## F6-2 The date's frames and pictures are zipped, so a short `frames` list goes unchecked

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/gate.rs:253` (`once(&date.day)
  .chain(&date.weeks).zip(&date.frames)`); `clients/crates/h3-map-view/src/status.rs:199`
  (`DateArt::shows`: a missing entry means 1 frame)
- **What:** nothing checks that `[map.status.date]`'s `frames` has one entry per picture (no
  `weeks` or `date` check in `tables.rs`'s `load`); the gate's `zip` stops at the shorter list,
  so a picture without its count is neither compared nor even looked for in the archives, and
  the viewer then steps it through one frame.
- **Failure scenario:** `base.toml` with `frames = [10, 4]` (a line lost in an edit) → the gate
  checks `NewDay.def` and `NewWeek1.def` only and passes; the viewer's second to fourth week
  windows show their first frame only, never stepping (the original steps 8, 12, 16).
- **Fix:** refuse in the tables' load a `frames` whose length is not `1 + weeks.len()`; the gate
  then zips safely. No drawn frame changes with the committed tables.

## F6-3 No test shows that a reach bounds what its drawer paints

- **Severity:** test gap
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-ui/src/paint/tests.rs:353`
  (`reach_is_the_rectangle_each_drawer_can_change`), `:411`
  (`a_texts_reach_takes_the_fonts_widest_overhangs`)
- **What:** both tests assert the rectangles `reach` and `text_reach` compute, never that `draw`
  of the same item changes no pixel outside them. T4c's partial background redraw is exact only
  if that holds (design §4.6, "Exact by construction"), and the text half rests on a reading of
  `draw_line`'s skip and stop (`text_reach`'s doc) that no test exercises: a first glyph with a
  negative left space, a later one with a negative left space, a last one with a negative right
  space, a centred or right-aligned line wider than its box, a mirrored button whose frame does
  not fill its canvas. (`Cursor::rect`'s test does count the changed pixels; the reach tests do
  not.)
- **Failure scenario:** a later change to `h3_render::text` (say, a glyph drawn at its pen before
  the skip test, or an alignment offset applied after it) lets a text paint a column past its
  reach; `cargo test` stays green and the incremental frame differs from the fresh one only on
  maps and moments where that text changes (caught, if at all, by `--check`'s `afresh` line).
- **Fix:** one test drawing each kind (the cases above) on a screen filled with a sentinel and
  asserting every changed pixel lies inside `reach`. No drawn frame changes.

## F6-4 The gate checks the creature portraits' DEF for creature 0's frame only

- **Severity:** test gap
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/gate.rs:205` (the `"creature"`
  need: `status.stack + 1` frames)
- **What:** the module's doc says the gate checks that the status window's DEFs "hold the frames
  its items draw", but a slot draws frame `creature + stack` (`status.rs:500`, `stack` 2) for any
  creature the live part gives (a `u16`, unbounded, `live.rs:339`), and the gate asks
  `cprsmall.def` for `stack + 1` = 3 frames: creature 0's. A DEF short of the later creatures'
  frames passes, and the window then draws frame 0 for them silently (`paint::button_frame`'s
  fallback, as drawn for a frame past the group's).
- **Failure scenario:** an installation (or a mod's archive found first) whose `cprsmall.def`
  holds 100 frames → `h3-map-view gate` reports nothing; a hero with an Azure Dragon (132) in the
  status window shows frame 0 where the original shows the dragon.
- **Fix:** ask for `stack + LAST_CREATURE + 1` frames (or the tables' creature count), as the
  morale, hall and fort needs ask for their highest frame. No drawn frame changes.

## F6-5 "Passed over" counts the late turns, not the frames passed over

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/window.rs:767` (and `Report::skipped`'s
  doc, line 371; the summary line 837)
- **What:** when the due frame's time is past and `replay.at(ms)` (the latest change at or
  before `ms`, `replay.rs:202`) is later, `skipped` grows by one, however many changes lay
  between the two. The doc says "Frames passed over because a later one was due already" and
  the run's summary "{n} passed over" next to "frames shown".
- **Failure scenario:** a 100 ms stall (a window drag, a slow upload) during a walk whose frames
  come every 25 ms → the changes at +25, +50 and +75 ms never shown (the one at +100 is), the
  report says "1 passed over".
- **Fix:** count the changes between `d.change.t` and `latest.t` (the replay knows them), or say
  "times a later change was shown instead". No drawn frame changes.
