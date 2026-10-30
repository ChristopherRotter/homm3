# F5 — the harness's M3 additions: findings

Covered: `capture/h3map.py`'s M3 diff (the masks, `lists`, `town_index`, `stage`, `choose_each`,
`chosen_toml`, `ask_status`, `look_over_view`, `live_part`, `status_toml`, `view_changed`,
`map_timeline`, the inputs and keys, `sequence`, `frame_check`, `reused`, `run_frames`'s keeping,
the digests); `capture/h3mem.py`'s map methods of M3 (`points` and the writes, `status_items`,
`status`, `realm`, `ask_status`, `town`, `town_list`, `kingdom`, `buttons`, `item_states`,
`node_cost`, `hover`, `visit_state`, `visit`, `contents`, `quest`, `write_content`,
`write_visit`, `signed`; `game_pid`); `map/masks.toml`; `sequences.toml`'s header and keys;
`situation.rs`'s M3 parts (the specs, `quests`, `selection`, the owner checks); the binary's
`sequences.rs` diff with `files::load_masks`, `timeline::changed` and `timeline::lines`; the
runbook's map sections. Code readings checked: `town_defended` 0x5BDFE0, `town_garrison_army`
0x5C1460, the cursor's class switch 0x40E9DE (class 219 falls to the default, as harness and
viewer have it).
Not covered: the offsets not re-read in the executable (the status items 2001-2009, +0x394 to
+0x39C, `OUT_OF_GAME`, `TEAMS`, `KINDS_VISITED`, the quest vectors and bank records, a boat
record's +0x14, `node_cost`'s index `(y + 2zh)w + x` for the underground); h3adv.py's M3
helpers beyond what `choose_each` relies on (`select_town`'s scrolling, `show_path`,
`set_view`); `drawn`'s paths beyond the mask; `sequences.toml`'s entries one by one; h3mem.py's
E08 additions in the same diff (not M3's); the tests of `situation.rs`.

## F5-1 A killed run's held hotkey or mouse button is not released at the next run's start

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/capture/h3map.py:1028` (`sequence`), against `README.md:808`
- **What:** the cleanup at a run's start releases Control, the eight direction keys and Space
  (`OTHER_KEYS`), not the eleven hotkeys (`HOTKEYS`, M3 T6b1) nor the left button (`press`
  without its `release`). It also runs after `stage`, so the staging's own clicks (the
  selection, the minimap, the path) meet whatever a killed run left down. The runbook says "a
  key a killed run left down is released at the next run's start".
- **Failure scenario:** a run killed (timeout, Ctrl-C) between `press` and `release` of a panel's
  button, or between `keydown = "k"` and its `keyup`: the display keeps the button or key down;
  the next run's `xdotool mousedown 1` / `keydown k` meets a held one (with key repeat off X
  drops the second press), and the staging's `game.click`es run with the button held, so the
  run records without the input, or fails in its staging for a reason unrelated to the
  sequence.
- **Fix:** release `*HOTKEYS` too and `xdotool mouseup 1`, before `stage` (the keys with no game
  focused are harmless). No drawn frame changes.

## F5-2 `write_visit` rewrites a hero's index when a hero stands on the object's entrance

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/capture/h3mem.py:1726` (`write_visit`: `word = self.u32(tile)`), and
  `write_content`'s final `write_u32(tile, word)` (`:1705`)
- **What:** `hover` reads an entrance whose tile holds a hero (class 34) or boat (8) by the word
  under it (the record's +0x14), but `write_visit` and `write_content` take the tile's own word
  and write the visit bits back into it. With a hero standing there that word is the hero's
  index (the tile's class 34), so the bits land in it.
- **Failure scenario:** a situation whose `visits` names an object Red's (or Blue's) hero stands
  on, e.g. a hero placed on a Windmill's or Shrine's entrance: `word |= 1 << 5 + shown` turns
  hero index 3 into 3 + 32·2^shown; the map's tile then names another hero record (the view
  draws or lifts the wrong one, the hint reads garbage), and `hover` still reads the object's
  unchanged +0x14 word, so the mark asked is not shown either. No situation does this today.
- **Fix:** as `hover`/`tile_word`, read and write the hero's (boat's) record +0x14 when the
  tile's class is 34 (8), or refuse such a visit. No drawn frame changes.

## F5-3 `write_content`'s hero write is unchecked for an object with no spell or skill

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/capture/h3mem.py:1696` (`write_content`, the `hero is not None` branch)
- **What:** the byte written is `HERO_SPELLS + signed(word, 13, 10)` or
  `SECONDARY + signed(word, 13, 7)` with no range check, while `contents` reads it only for
  `0 <= spell < 70` / `0 <= skill < 28` and treats a Witch Hut's 0xFE000 (skill -1) as none.
- **Failure scenario:** a situation's `visits` with `hero = true` on a Witch Hut whose word holds
  no skill (bits 13–19 all set: -1) writes 1 into the hero record's byte before its secondary
  skills (`SECONDARY - 1`), a field of the record the status window or the game reads; the
  live part is then read from a corrupted hero, silently.
- **Fix:** raise when the field is outside 0..70 (0..28), as `contents` bounds it. No drawn
  frame changes.

## F5-4 `choose_each` waits for the status window's kind, not for the window of the one chosen

- **Severity:** robustness
- **Verdict:** PLAUSIBLE (not traced into hero_select/town_select's build of the window)
- **Where:** `clients/capture/h3map.py:584` and `:590` (`choose_each`), `h3mem.py`'s `status`
- **What:** for the second town onwards `status_shown() == 4` holds already from the town before;
  for a hero the test `status(colour)["hero"] == hero` reads the player's selected hero (+0x4),
  which `select` has just set, not the hero the window was built for. `status` itself mixes
  sources: a town's name, buildings and garrison from the selected town's record, its silo and
  income from the window's items (a hero's skills and army from the record, morale and luck
  from the items). Only `game.click`'s 0.6 s real-time sleep orders the read after the rebuild.
- **Failure scenario:** a slow step (the machine loaded, as now) where the window is rebuilt by a
  view draw after the 0.6 s: town B's `[chosen]` row gets town B's name with town A's silo and
  income; the viewer draws that and the choice's frames differ from the original's (a flake
  that looks like a viewer bug), or, recorded once that way, the next good run reports CHANGED.
- **Fix:** wait for a fact of the window itself (e.g. the income item's text or the status
  items' list changing from the one read before the click), or read the status twice until
  two reads agree after a view draw. No drawn frame changes.

## F5-5 `look_over_view` holds the pointer in an edge zone with the clock running

- **Severity:** robustness
- **Verdict:** PLAUSIBLE (no recorded sequence has such a point today)
- **Where:** `clients/capture/h3map.py:656` (`look_over_view`), called at `:1047` before the
  clock is frozen
- **What:** every `pointer`/`click` point `over_view` accepts is visited, including those within
  16 pixels of the left or top side (x 8–15, y 8–15), where the map scrolls every 70 ms while
  the clock runs in real time; the tile is computed from `sit["view"]`, and the view is not set
  back afterwards.
- **Failure scenario:** a sequence with `pointer = [12, 300]` (an edge scroll that starts over
  the view): during the look the view scrolls left once or more, so `mouse_tile()` never equals
  the tile computed from the situation's view (a 3 s timeout), or equals it by chance after a
  scroll and reads another tile's cost; the run's 0 then starts from a view that depends on real
  time.
- **Fix:** skip the points inside the edge zones (they are looked at only after the scroll
  anyway), or `set_view` again after the look. No drawn frame changes.

## F5-6 The parser's checks and the harness's writes disagree on what a hero or town has written

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/situation.rs:712` (`heroes`: `written` is points or
  mana only), against `HeroSpec::written` and `clients/capture/h3map.py:372`, `:377` (`lists`)
- **What:** (a) the owner check refuses points and mana on a hero not Red's but lets `skills`,
  `morale`, `luck` and `army` through, which `HeroSpec`'s docs give as Red's only ("likewise")
  and which `lists` writes into whichever hero stands on the tile. (b) Rust counts an `army` or
  `garrison` as written only when it is not empty; Python writes on the key's presence, so
  `army = []` / `garrison = []` writes seven empty slots while the parser saw nothing written
  (no selection asked, any owner allowed).
- **Failure scenario:** a situation with `garrison = []` on Blue's town and no `select`: the
  parser accepts it, the harness empties Blue's town's garrison (the hover's `armed` and the
  cursor over it change), or `army = []` on Red's selected hero with no `select`: memory's army
  is empty, the window drawn at the turn's start still shows the old one, the live part's
  `status` gives the empty one, and the still differs for a reason no check names.
- **Fix:** test `h.written()` in the owner check, and in `lists` write `army`/`garrison` only
  when not empty (or refuse an empty one in the parser). No drawn frame changes.

## F5-7 `press` and `release` are not paired by either parser

- **Severity:** robustness
- **Verdict:** PLAUSIBLE (X's handling of an unpaired fake button event not checked)
- **Where:** `clients/capture/h3map.py:971` (`sequence_inputs`), `clients/crates/h3-map-view/src/sequence.rs:230`
- **What:** keys are tracked (a `keyup` never pressed is refused on both sides) but the button
  is not: a `release` with no `press` before it, a second `press`, or a `click` while the
  button is held all pass, though `xdotool mouseup 1` with no button down is dropped by the X
  server while the viewer plays an `Action::Release` message.
- **Failure scenario:** a typo'd sequence (`release` at 1200, its `press` at 1300) records with
  the original seeing one press and no release, the viewer one release then one press; the
  frames differ from the release on, which reads as a viewer bug.
- **Fix:** track the button as the keys (`down` holds "button"): refuse a release without a
  press, a press or click while it is down, in both parsers. No drawn frame changes.

## F5-8 `town_defended`'s reading and `hover`'s names swap the garrison and visiting heroes

- **Severity:** docs drift
- **Verdict:** CONFIRMED (disassembly of 0x5BDFE0 against `town_garrison_army` 0x5C1460)
- **Where:** `docs/exe/map.toml:1270` (`town_defended`'s summary), `clients/capture/h3mem.py:1446`
- **What:** the code tests +0x10 ≥ 0 first (defended), else the army of the hero at +0xC, else
  the town's +0xE0. `town_garrison_army` (and `h3mem.TOWN_GARRISON_HERO`, the runbook, the
  status window) read +0xC as the garrison hero whose army counts, so +0x10 is the visiting
  hero; `town_defended`'s summary calls +0x10 "a garrisoned hero" and +0xC "a visiting hero",
  and `hover` names them `garrison` and `visiting` after it. The behaviour is right; the names
  are swapped.
- **Failure scenario:** a later task "tidies" `hover` to use `TOWN_GARRISON_HERO` for its
  `garrison` variable: the test becomes +0xC ≥ 0 first and +0x10's army, a town with an empty
  garrison hero reads defended, and the cursor's attack over it diverges.
- **Fix:** swap the two names in `map.toml`'s summary and in `hover` (use the constants). No
  drawn frame changes.

## F5-9 `h3map.py`'s mask comment says `panels` holds what it no longer holds

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `clients/capture/h3map.py:93`; `clients/capture/README.md:738`
- **What:** the comment gives `panels` as "the status window, the amounts and the date, the
  status bar's hint"; `masks.toml` has `panels = []` since T6a1 (the status window since T5b1,
  the amounts and the date since T5a compared). The runbook's mask paragraph ends with
  "(`panels` the status bar's hint alone)", true only between T5b1 and T6a1. Both sit next to the
  hint mask the runner is removing, so the same edit can settle them.
- **Fix:** say `panels` is empty (what a later task draws goes there). No drawn frame changes.
