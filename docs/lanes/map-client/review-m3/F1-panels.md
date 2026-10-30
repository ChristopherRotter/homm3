# F1 — the panels as drawn and over time: findings

Covered: `h3-map-view/src/{panels,lists,status,minimap,record,layout}.rs` whole; `tables.rs`'s
`Tables::defs` and `load` (the `[map.minimap]`, `[map.lists]`, `[map.status]` checks) and the
committed `[map.lists]`, `[map.status]`, `[map.minimap]` values; `h3-ui/src/paint.rs` whole; the
binary's `texts.rs` whole and `paint.rs`'s `Painter::view`, `scene`, `walk`, `screen`, `fresh`,
`frame_into`; `timeline.rs`'s `Minimap`, `Draw`, `spend` … `step_objects`, and the sites that
call `draw_lists`, `draw_minimap`, `draw_slot` (`underground`, `scroll_list`, `choose_town`,
`finish`, `lift`, the step's end); `scene.rs`'s panel draws (lines 420–495); `live.rs`'s kingdom
parse (505–582). Design §4.1, §4.4, §4.5, §4.7, §4.8, §4.9 read against the code; one code
reading checked (`town_list_frame` 0x5BD700: the built bit counts with either argument, so the
status window's town picture, the list's frame less 2, is right).

Not covered: the panels' unit tests (`panels/tests.rs`, `lists/tests.rs`, `status/tests.rs`)
beyond the minimap's scale test; `tables.rs`'s other sections and the gate's frame checks
(`bin/h3-map-view/gate.rs`, outside the fraction); `h3_ui::ui::Dialog::draws` (where the
items' places are added up).

## F1-1 A text or label item's box or colour overflows into a panic on a crafted layout

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-ui/src/paint.rs:142` (`Draw::Text`'s clip) and `:177`
  (`Draw::Label`'s colour)
- **What:** `[*x, *y, x + item.size[0], y + item.size[1]]` and `item.colour + 2` use plain `+`,
  and the workspace builds release with `overflow-checks = true` (`clients/Cargo.toml:128`), so
  an overflow panics in every build. Nothing bounds a text item's size or a label's colour when
  the layout loads (`AdventureLayout::check` checks names, the clip, the cursor and the edge
  only); `reach` for the same draws saturates, so the two disagree on the same input.
- **Failure scenario:** `<map>/layouts/adventure.toml` with the status bar's text item (id 200,
  at x 8) given `size = [2147483647, 18]`, or any label with `colour = 4294967295` drawn
  disabled → the first frame panics in `h3_ui::paint::draw` (the viewer aborts instead of
  refusing the layout).
- **Fix:** `x.saturating_add(item.size[0])` (as the picture arm does) and `colour.saturating_add`;
  better, refuse a negative or overflowing item box in the layout's check. No drawn frame changes.

## F1-2 The minimap's tiles are not cut to its area

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/minimap.rs:115-144` (`MinimapArt::check`),
  `:356-366` (`Look::size` kept); the fill in `compose.rs:1145` (F2's file) clips to the screen
  and the cell only
- **What:** `Look` carries the area's size, but only the box (`Base::radar`) is cut to it; the
  tiles' spans run `side` × the pattern's pixels from the area's corner, and neither the tables'
  check (which cannot see the layout) nor `Base::build` bounds that sum by the area. The
  original cannot overrun: its jump tables fix each size's pixels (§4.4).
- **Failure scenario:** a `[map.minimap]` size `{ size = 36, columns = [8], rows = [8], … }`
  (or a layout with a smaller area 1) → the tiles fill 288 × 288 pixels from (630, 26), over the
  hero and town lists, the buttons and the status window; no error.
- **Fix:** refuse in `Base::build` (or the gate) a size whose pattern summed over `side` passes
  the area, or cut the tiles' fill to `look.at`/`look.size`. No committed frame changes (every
  committed size sums to 144).

## F1-3 `finish` says the unpaid step set the points to 0, but leaves memory's points as the last `spend` gave them

- **Severity:** docs drift
- **Verdict:** PLAUSIBLE (the recordings' sampling not traced)
- **Where:** `clients/crates/h3-map-view/src/timeline.rs:2072-2086`
- **What:** at a walk's end without a green arrow the comment says the next step "set them to 0
  (0x480864)" and the code marks the hero unable to move and calls `draw_lists`, which shows
  memory's kingdom, but nothing sets the selected hero's points to 0 or calls `spend(t)`: the
  movement bar shows 0 only because the recording's `points` line at the last step's end
  (taken there by `spend`) already reads 0. That holds when the step's end and the next step's
  refused start fall in the same recorded millisecond (they share one call chain), which the
  T4b checks bear out for `walk-points`; a recording sampled between them would leave the bar at
  the leftover points while the original shows frame 0.
- **Failure scenario:** a sequence whose last paid step leaves points at or above the cheapest
  step from the new tile but below the next path step's cost, with the recording's 0 written a
  millisecond after the step's end → `draw_lists` at 0x4081B9 shows the leftover bar; the
  original shows 0.
- **Fix:** call `self.spend(t)` (or set the selected hero's points to 0) before `draw_lists` in
  that branch, or reword the comment to say the points are the recording's. No current capture
  changes.

## F1-4 The loop's `minimap` field duplicates `drawn.minimap`

- **Severity:** simplification
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/timeline.rs:780`, `:908`, `:2261-2266`, `:2281`
- **What:** `Loop::minimap` is written only by `draw_minimap`, which writes `drawn.minimap` with
  the same value, and read only by `draw`, which copies it into the new `drawn`; both start as the
  same value. The two can never differ.
- **Failure scenario:** none today; a later change that writes one and not the other would make
  the minimap's last draw and the next view's disagree.
- **Fix:** drop the field; `draw` keeps `self.drawn.minimap`. No drawn frame changes.

## F1-5 Per-frame clones in the chooser and the painter's key

- **Severity:** performance
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/timeline.rs:2305-2316` (`choose`);
  `clients/crates/h3-map-view/src/bin/h3-map-view/paint.rs:457` (`Painter::view`)
- **What:** every view's draw runs `choose(t, false)`, which clones `kingdom.selection` (or
  `kingdom.status`) before it knows it needs it, and with the kingdom's window asked clones the
  realm (two `Vec`s) at every draw, though an unforced draw of the same kind builds nothing
  (§4.9's "the chooser at each draw compares a status or returns" is not quite so).
  `Painter::view` clones the whole `Draw` (its path, hint and pressed `Vec`s) on every call
  before comparing it with the one shown, even when the frame is unchanged.
- **Failure scenario:** a town selected (its status holds the silo `Vec`) or the kingdom's window
  asked → one or two heap allocations a draw for nothing; `view` allocates the path's copy at
  every frame of a walk.
- **Fix:** compare kinds first and clone only the window built; in `view`, compare
  `(draw, buttons)` with the key by reference and clone only on a miss. No drawn frame changes.
