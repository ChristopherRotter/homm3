# F2 — the frame's redraw: findings

Covered: `h3-map-view/src/compose.rs` (whole: `Composer`, `Changed`, `Grid`, `Buckets`,
`changed_areas`, `Background::redraw`, `same_moved`, `uses`, `move_pixels`, `rect_of`, the `Pen`),
`damage.rs`, `draw.rs` (whole); `scene.rs`'s `ViewPlace`, `TileIndex::build`, `scene_in`,
`object_piece`/`look_piece` (against the pre-M3 code), `lifted_whole`; `h3-render/src/screen.rs`'s
`cut`, `copy_rect`, `Widening` (`widen`, `widen_rect`, `widen_onto`); `h3-sdl/src/lib.rs`'s
`draw`, `present_part`, `present_read_part`, `read_back`; the binary's `paint.rs` `view`,
`fresh`, `frame_into`, `changed` (the widened frame and the history), `window.rs`'s `Frame`,
`Maker`, `Texture::show`, the loop's present branches, the redraw event; `render.rs`'s M3 diff
(live-part reading only, nothing incremental). Design §4.2, §4.3, §4.6. `compose/tests.rs`'s
`as_fresh` (checks the frame and the damage's completeness).

Not covered: `h3_ui::paint::reach` (F1's: T4c's areas are exact only if each reach bounds its
drawer); `h3_ui::cursor::Cursor::rect` against `draw` (F3's: the cursor's cut is exact only if
the rectangle bounds the drawing); `View::tiles` and the passes' places (pre-M3, only their
linearity in `origin` matters here, and a mismatch costs redraws, not pixels).

Verdict on the main question: within the scenes the scene builder makes from the committed layout
and tables, I found no way for the incremental frame to differ from the fresh one: each cell's
comparison is complete (every draw whose rectangle meets it, in order, its palette steps and the
flag palette), the move keeps only clip pixels whose source lay in the clip, T4c's areas dirty
every cell they meet and are refused where pixels move, the damage covers the move and every
restored rectangle, and the widened frames and the texture follow the compositions' chain exactly
(`between` is right in either order, a spare buffer older than the history is widened whole, both
cursors are always added). The divergences below need a crafted layout or table (F2-2, F2-3), or
are in the window's choice of what to present (F2-1).

## F2-1 A change of the buttons alone is never presented

- **Severity:** bug
- **Verdict:** CONFIRMED in the code; PLAUSIBLE that a buttons-only change occurs in a recording
  (not traced through the timeline)
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/window.rs:322` (`Frame::shows`), used
  at `:778`; `clients/crates/h3-map-view/src/replay.rs:250-253`
- **What:** M3 T2a added `buttons` to `Change` and the replay keeps a change when only the buttons
  differ (`replay.rs:252`), but `Frame::shows` (unchanged since M2) compares `drawn` and the
  cursor only. When such a change falls due, `Some(s) if s.shows(&change, look)` takes the shown
  frame as showing it, sets `s.change = change` and presents nothing; the frame made ahead for it
  (with the right buttons) is recycled.
- **Failure scenario:** a millisecond where `Loop::buttons()` changes (move hero or next hero's
  state, part of the background, §4.3/§4.6) while the draw and the cursor stay → the screen keeps
  the old button picture until the next change of the draw or cursor (an animation step or a
  cycle, up to `frame_ms` later; indefinitely on a still map). None of `--check`'s lines sees it:
  `headless` compares the change log (pushed regardless, `:775`), `afresh` each frame made with its
  own change, the read-back the frame shown.
- **Fix:** `shows` also compares `self.change.buttons == change.buttons` (or compare the changes
  minus `t`). Changes a drawn frame only where the bug bites (the window's frames, no capture).

## F2-2 The minimap's drawer is not cut to the rectangle the composer gives it

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/compose.rs:1139-1185` (`Pen::minimap`) against
  `rect_of` `:896-904`; `clients/crates/h3-map-view/src/minimap.rs:115-131` (`check`), `:290-303`
  (`spans`)
- **What:** every incremental path assumes a draw changes nothing outside `rect_of` (the cells'
  lists, the not-usable path's restored rectangle, the damage). The minimap's rectangle is the
  layout item's area (`look.size`), but its tiles' pixels are the pattern's widths summed over the
  map's side (`spans`), which nothing ties to the area: `MinimapArt::check` refuses only an empty
  pattern or a 0. The fresh path draws it with no cut (the whole screen).
- **Failure scenario:** a `[map.minimap]` size entry whose widths over the side exceed the item
  (e.g. `{ size = 36, columns = [5], rows = [5] }` with the 144 × 144 area: 180 pixels) → the first
  full frame draws 36 pixels past the area; a later scene that restores a cell there from the
  background never draws them again (the minimap is not listed in that cell), and a minimap change
  past the area is outside the damage → the window's frame differs from the fresh one and from
  what was uploaded. The committed patterns (`map/assets/base.toml:244-247`) all sum to 144.
- **Fix:** in `Pen::minimap`, meet the clip with the minimap's own rectangle (as `Radar` already
  does with `radar`'s); optionally `check` the pattern's sum against the area where both are
  known. No committed frame changes.

## F2-3 A move compares every cell with the last scene's draws moved, minimap aside, whether or not its pixels moved

- **Severity:** robustness
- **Verdict:** CONFIRMED (for a scene the composer accepts; the committed layout does not build one)
- **Where:** `clients/crates/h3-map-view/src/compose.rs:478-483` (`was_at`), `:521-531` (`stays`),
  `:812-815` (`stays_put`)
- **What:** after `move_pixels`, the pixels inside `kept` came from `cell − shift`, those clear of
  the clip did not move; but the comparison is chosen per draw (`stays_put`: only the minimap and
  its box unmoved), not per cell. A cell is right only if it is compared with the draws that put
  its current pixels there: inside `kept` all of them moved, clear of the clip none.
- **Failure scenario:** a layout whose minimap item overlaps the view's clip (the background still
  one colour there) and a scroll → a cell in `kept` meeting the minimap compares the minimap with
  itself unmoved (`now == was`), the pieces moved, finds them the same and is left with the
  minimap's pixels moved by the shift; the fresh frame has them in place. Conversely the gems'
  cells clear of the clip are compared moved and always redrawn on a scroll (cost only; today
  those cells are also partly in the clip). A future draw that moves with the view but lies
  outside the clip (M4) would be compared moved while its pixels stayed.
- **Fix:** bucket the last scene's rectangles twice (moved and unmoved) and compare a cell in
  `kept` with the moved lists (every draw moved, the minimap too), a cell clear of the clip with
  the unmoved ones (`same_moved` with shift 0); `stays_put` goes. Or refuse the move when a
  `stays_put` draw meets the clip. No committed frame changes.

## F2-4 "A jump moves nothing" is not what the composer does

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/compose.rs:82` ("a jump draws the whole view"),
  `docs/epics/E24-design.md:1671` ("A jump moves nothing: every cell differs"); the code at
  `compose.rs:450-464`
- **What:** the move is taken for any change of the origin smaller than the clip on both axes,
  with the same clip and tile and a one-colour background. A jump (a minimap click, next hero, a
  town chosen) of up to 18 columns and 16 rows (the clip `[8, 8, 600, 552]` is 592 × 544,
  `map/layouts/adventure.toml:22`) is a move: the pixels it keeps are moved and only
  the strip it brings in is drawn. Only a larger jump, or one to another level's background,
  draws the whole view.
- **Failure scenario:** none in the pixels (the move is exact for any shift); a reader of §4.3 or
  of the struct's comment expects a jump's frame to cost a full view and the damage to be the
  whole clip, which a measurement of a short jump will contradict.
- **Fix:** say "a move as large as the clip or larger (a far jump) draws the whole view; a nearer
  jump is a move". No frame changes.

## F2-5 A redraw request is lost or half-honoured when a frame falls due in the same turn

- **Severity:** robustness
- **Verdict:** PLAUSIBLE (depends on what the renderer keeps across the events)
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/window.rs:762-812`;
  `clients/crates/h3-sdl/src/lib.rs:647-653`
- **What:** the redraw (`EXPOSED`, pixel size, display scale, fullscreen) is handled only in the
  `else if redraw` branch, which sets `texture.held = None` ("the window's contents may be lost")
  and uploads the whole frame. When a frame falls due in the same turn, that branch is skipped:
  a differing frame is uploaded in part (against the comment's premise), and a due change that
  `shows` the frame shown presents nothing at all. M3's partial uploads make the first case new.
- **Failure scenario:** a resize or fullscreen toggle at the instant a frame is due → only the
  changed rectangles uploaded; if the texture's contents were in fact lost, the rest of the
  screen stays stale until a whole upload (the next redraw request). SDL3 keeps a streaming
  texture's contents across these window events (the back buffer is rebuilt from the whole
  texture at each present), so in practice only the "nothing presented" case shows, for one
  frame interval; the device-reset events that do lose textures
  (`SDL_EVENT_RENDER_DEVICE_RESET`) are not mapped at all.
- **Fix:** honour `redraw` after the due branch too (present again, and set `texture.held = None`
  before a due frame's upload if the premise is kept), or drop the premise and the whole upload if
  it is wrong and map `SDL_EVENT_RENDER_DEVICE_RESET` to a whole upload. No captured frame changes.

## F2-6 `cycled_kept` keys a DEF's palette by its steps alone, not its ranges

- **Severity:** robustness
- **Verdict:** CONFIRMED (unreachable with tables that do not change during a run)
- **Where:** `clients/crates/h3-map-view/src/compose.rs:1273-1293`; against `changed_defs`
  `:817-833`, which compares ranges and steps
- **What:** the kept palette is reused when the DEF's cycles' steps are the same, whatever their
  ranges; the cells' comparison (`changed_defs`) takes a change of ranges as a change. A composer
  given two scenes whose cycles for one DEF differ in ranges only draws the second with the first's
  palette, while a fresh composer rotates the new ranges.
- **Failure scenario:** today the ranges come from the tables (`scene_in` `:497-509`), fixed for
  the window's run, so no frame differs; a composer reused across tables (a test, a future reload
  of the tables) would draw stale water.
- **Fix:** keep `(ranges, steps)` per cycle in `kept` (the comparison `changed_defs` already makes).
  No frame changes.

## F2-7 The upload's and the widening's rectangle lists are not merged

- **Severity:** performance
- **Verdict:** PLAUSIBLE (not measured)
- **Where:** `clients/crates/h3-map-view/src/damage.rs:65-88` (`tidy`), `paint.rs:634-637`
  (`frame_into`)
- **What:** `frame_into` widens every rectangle the history gives, overlaps and repeats included
  (no `tidy`); `tidy` drops only exact repeats, in O(n²), then falls back to one bounding rectangle
  past 64. A composition's damage is the move plus a few merged cell runs, so two frames a few
  compositions apart repeat the same strips; a buffer up to 64 compositions old re-widens up to
  64 copies of them.
- **Failure scenario:** a spare buffer last used many compositions ago (the renew path makes
  frames again; the history keeps 64) → the same rectangles widened many times over; on the
  texture side, a scroll's damages over a few frames exceed 64 rectangles and upload their bounding
  box, the whole view.
- **Fix:** run `tidy` (or a union of the cells' rows) before widening, and join overlapping
  rectangles as `compose::joined` does rather than only exact repeats. No frame changes.

## F2-8 The widened frame's and the texture's partial paths have no test that `cargo test` runs

- **Severity:** test gap
- **Verdict:** CONFIRMED (grep: no test calls `frame_into`, `Painter::changed` or `Texture::show`)
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/paint.rs:616-681`,
  `window.rs:896-940`; `h3-sdl/src/lib.rs:693-714` (`draw`'s rectangles)
- **What:** `compose/tests.rs`'s `as_fresh` checks each composed frame and its damage, and
  `damage/tests.rs` the history and `tidy`, but the step from them to the window's pixels (a spare
  buffer widened only where the history says, the old cursor's rectangle widened again, the
  texture's rectangles from two `Held`s) is checked only by `--check`'s `afresh` and read-back
  lines under Xvfb or on the user's machine. A regression there (say a cursor rectangle not
  added) passes CI.
- **Failure scenario:** `frame_into` drops `self.rects.extend(held.cursor)` → every frame after a
  cursor move keeps the old cursor's pixels; `cargo test` and the verify stay green.
- **Fix:** a unit test in the binary driving a `Painter` over a few draws with one `Widened`
  buffer reused out of order and the cursor moved, comparing with `Widening::widen` of
  `Painter::fresh`, and one for `Texture`'s rectangle choice against a fake `changed`. No frame
  changes.
