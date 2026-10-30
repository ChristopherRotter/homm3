# F4 — the clicks, the keys, the walk's input, the sound without the focus: findings

Covered: `timeline.rs` `Action` … `Unplayable` (146–347), `run` … `choose_town` (1047–1686),
`edge` … `spend` (1780–2218), `step_objects`, `is_button`, `list_slot`; `h3-map-view/src/keys.rs`;
`h3-ui/src/keys.rs`; `h3-ui/src/music.rs` (`Music`, `Queue`, the focus); `h3-sdl/src/lib.rs`
`Audio::hold` and the event translation it relies on (`Event::Click`, the focus events); the
binary's `window.rs` event loop and focus (lines 470–720) and `audio.rs`'s `Player` (focus, play,
stop, service); design §4.13–§4.15; `hero_move_loop`'s drain (0x407F80–0x407FFA) and
`next_hero_find` 0x4BAA40 checked in the disassembly (both match the code); the tests
`a_press_a_hotkey_or_space_stops_the_paths_walk_too`, `the_input_ring_keeps_the_last_63_messages`,
`the_steps_queued_during_a_step_are_the_rings` checked against their claims (they hold).

Not covered: `window.rs` past line 720 (the present path, F2's) and its tests; `timeline.rs`'s
tests beyond the three above; `h3-ui/src/music.rs`'s own tests; `look_again` … `selection` (F3's).

## F4-1 Ctrl pressed while a hotkey is held leaves its button stuck pressed, its command lost

- **Severity:** bug
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/bin/h3-map-view/window.rs:577`
- **What:** a key's release is looked up with the Ctrl state *at the release*
  (`keys.command((), ctrl.contains(&true), k.name())`), not with the chord its down gave. Ctrl+H is
  bound to nothing, so H's up gives no `KeyUp`.
- **Failure scenario:** H down (button 11 drawn pressed, `keyed` holds it), Ctrl down, H up:
  `command(true, "H")` is `None`, no `Action::KeyUp(NextHero)`. Button 11 stays drawn pressed and
  Next Hero never runs until H is pressed and released again. The original matches the key up by
  its scan code alone (0x4564D6), whatever Ctrl does. The mirror case: a user's file binds
  `next-hero = ["Ctrl+H"]`, and Ctrl let go before H gives the same stuck button. This is not
  MCL-Q24 (that is the minimap's loop eating the up).
- **Fix:** keep the command each key's down gave (`held` as a `BTreeMap<Key, Option<Command>>`)
  and give that command's `release()` at its up. A live-only change: no capture is affected.

## F4-2 Losing the focus during a minimap drag jumps the view to the map's top-left corner

- **Severity:** bug
- **Verdict:** CONFIRMED
- **Where:** `window.rs:666–667` (the `OFF` pointer given at the loss), `timeline.rs:1385–1409`
  (`drag`)
- **What:** at the loss of the focus the window gives `Action::Pointer(OFF)` = `[-1, -1]` so the
  edge scroll stops. While `dragging`, `drag` takes that as a move: x and y clamp to the area's
  left and top, the tile is (0, 0), and the view's corner becomes (−9, −8).
- **Failure scenario:** live, hold the left button on the minimap, Alt-Tab away (or a system dialog
  takes the focus): the view and the minimap redraw at the map's top-left corner. The original gets
  no mouse move from WM_ACTIVATEAPP: its loop keeps the last corner. With a panel button held the
  same `OFF` releases the button's picture, which is less visible.
- **Fix:** give no `Pointer(OFF)` while the timeline holds a press (have the loop ignore a point
  off the frame in `drag`/`holding`, or let the window send the focus's loss as its own action that
  only the edge scroll reads). Live only, no capture changes.

## F4-3 A press or click taken from the queue after a key's step leaves the pointer stale

- **Severity:** bug
- **Verdict:** CONFIRMED
- **Where:** `timeline.rs:1183` (`holding`: `self.pointer = at`), `1390` (`drag`), reached from
  `resume` → `act(t, action, false)` (2107) and `act`'s `Click` arm (1152–1154)
- **What:** `act` with `fresh == false` keeps `self.pointer` as the latest point given
  (GetCursorPos), but once a queued press starts a hold, `holding`/`drag` overwrite it with each
  queued message's point. The queued actions after the hold ends go through `act(.., false)`,
  which no longer sets the pointer, so it stays at the release's point.
- **Failure scenario:** during a key's step (its actions all wait), the player clicks Next Hero
  (`Press(b)`, `Release(b)`) and then moves the mouse to `p` (`Pointer(p)`). `run` sets the pointer to
  `p` as given; after the step `resume` replays them: `holding(Release(b))` sets the pointer back
  to `b`, and `act(Pointer(p), false)` leaves it there. The cursor (the tick's poll), the edge
  scroll and the next hint read `b` until the mouse moves again. The same happens with a queued
  `Click` on a button or on the minimap.
- **Fix:** pass `fresh` to `holding`/`drag` and set `self.pointer` only when fresh; test
  `inside` and the drag's tile with the message's `at` (the original's loop tests the message's
  own point, `[esi + 4]`, `[esi + 8]` at 0x456359, checked). No recorded sequence clicks during a key's
  step: no capture changes.

## F4-4 A hotkey's key up clicks its button again after the mouse released it

- **Severity:** divergence
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/timeline.rs:1277` (`key_up`), with `holding` 1192–1200
- **What:** `key_up` gives the command whenever `keyed` holds the key, never asking whether the
  button still shows pressed. The design's own reading (§4.15: "its key's up then finds it
  released and does nothing, 0x4564D6") says the original's key up acts only on a pressed button.
- **Failure scenario:** H down (`Key(NextHero)`: button 11 pressed, `keyed` = [(NextHero, 11)]),
  press and release the mouse on button 11 (`holding`: released, `next_hero` runs), then H up:
  `key_up` finds the entry, runs `next_hero` a second time. The original chooses the next hero
  once. The same happens with a list's arrow (the list scrolls twice). And if H is held while the
  mouse presses button 11, slides off and lets go (released, no command), H's up still runs it.
- **Fix:** in `key_up`, give the command only while `self.drawn.pressed` holds `id` (and drop the
  `keyed` entries of `id` when a mouse hold releases it). No recorded frame changes (no sequence
  mixes a hotkey and the mouse on one button).

## F4-5 A right click neither ends a path's walk nor counts in the ring

- **Severity:** divergence
- **Verdict:** CONFIRMED
- **Where:** `timeline.rs:189–203` (`ends_walk`'s doc names "the right one's down, 0x20"),
  `h3-sdl/src/lib.rs:627–636` (the right button is `Key::RightButton`), `keys.rs:109–131` (not
  bound on the map)
- **What:** the original's drain (0x407FB1, checked) ends the walk at a right button's down (kind
  0x20), and its up is a message in the ring. The map's key table binds no `RightButton`, so the
  window gives no action for it, and no `Action` variant stands for it.
- **Failure scenario:** live, right-click during a path's walk: the hero walks on to its
  destination; the original stops after the step under way. During a key's step the right
  button's two messages are missing from the ring's 63, so a burst keeps two older messages more
  than the original's ring would. And a held button's loop ends at the right button's up as at the
  left's (`def_button_message` 0x456338: `cmp eax, 0x40; je 0x456451`, checked): left-press a
  button, right-click while holding it, and the original gives the command at the right button's
  up; the viewer waits for the left's release. §4.13 names only the left's up (0x10).
- **Fix:** an `Action::RightPress`/`RightRelease` (doing nothing on the map in M3, `ends_walk` for
  the down, one message each) that the window gives for `Key::RightButton`, or record it in §4.15's
  "Not built" with the minimap's right click. Live only.

## F4-6 Next Hero to a hero without a choice does nothing silently in a recording

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `timeline.rs:1533`
- **What:** `next_hero` finds the next movable hero and, when `chosen_hero(i)` is `None` (the live
  part gives no `[chosen]` for it, or its index names no map hero), does nothing. `hero_slot`
  refuses the same case (`unbuilt(Unplayable::Panel)`).
- **Failure scenario:** a recording whose live part lacks one hero's choice, then H: the original
  selects that hero and the viewer keeps the old one. The frames then differ with no
  `Unplayable` naming the reason, instead of refusing at the action.
- **Fix:** `next.map(|i| …).ok_or(…)`: refuse with `unbuilt(Unplayable::Command(NEXT_HERO))` when
  the hero found has no choice. No capture changes (every recorded hero has one).

## F4-7 A click's point is not clamped to the frame as the pointer's is

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `window.rs:560–568` (against `550–557`)
- **What:** `Event::Pointer` clamps a point on the window but off the frame to the frame's nearest
  (M2's R2-8: the original's mouse cannot leave its screen). `Event::Click`'s point, now given as a
  `Press` and a `Release` and stored as `mouse`, is not clamped.
- **Failure scenario:** fullscreen with bars: a press in the left bar beside the view. Clamped, it
  would be the view's column 0 (a tile's click, as the original's mouse at x = 0). Unclamped,
  `panels.at` finds nothing and the timeline's pointer goes off the frame: the edge scroll stops
  until the mouse moves, though the clamped point lies in the west zone. `mouse` (given again when
  the focus returns) is the off-frame point too.
- **Fix:** clamp `x`, `y` with `clamp(_, look.frame[_])` as the pointer's. Live only.

## F4-8 A loss and a regain of the focus within one batch of events are missed

- **Severity:** robustness
- **Verdict:** PLAUSIBLE
- **Where:** `window.rs:665–682`, `h3-sdl/src/lib.rs:643–645`
- **What:** the window reads `win.focused()` once after the whole batch, so a FOCUS_LOST and a
  FOCUS_GAINED pending together leave no change. The original handles each WM_ACTIVATEAPP: the
  music faded out by the stub and back in by `music_resume`, the effects stopped and resumed.
- **Failure scenario:** a quick switch away and back while the viewer's loop waits (the batch is
  everything pending at one `wait`): the sound plays on uninterrupted, and the edge scroll's `OFF`
  is never given. Rare and mostly harmless; the sound differs from the original's.
- **Fix:** have `Window::wait` give the focus changes as events (`Event::Focus(bool)`) and handle
  each in order. Live only.

## F4-9 `key_step` repeats `step_objects` inline

- **Severity:** simplification
- **Verdict:** CONFIRMED
- **Where:** `timeline.rs:1844–1850` (against `2394–2402`)
- **What:** the block after the path's clearing is `step_objects(t)` word for word.
- **Failure scenario:** none today; two copies of the deadline's rule to keep in step.
- **Fix:** call `self.step_objects(t)`. No frame changes.
