# F3 — the mouse-over: hints and cursor: findings

Covered: `h3-map-view/src/hover.rs` (whole: `Facts::read`, `Hover::{new, base, current,
hint_tile, entrance, hint, contents, visits, cursor, kind}`, `days`, `PanelItems`, `takes`,
`panel_hint`, `parked_hint`); `hover/contents.rs` (whole); `live.rs`'s `hover` field and the
binary's `[hover]` read (`render.rs:617`); `timeline.rs`'s `Cursor`, `Rules::{keeps_edge,
over_view, tile_at, column_at}`, `look_again` … `selection`, the `walked`/`selected_id`/`column`
resets; `h3-ui/src/cursor.rs` (whole, all of it new since e8a7990a); `clients/text/keys.toml`'s
map hint keys and families; the hover and timeline tests' claims where a branch looked unreached.
Checked against the original with `scripts/exe.py` and found as the design says: the cursor's
kinds and frame (0x40E960 to 0x40EB6A, including the boat-from-a-boat kind 0 that still takes
`6 d`: frame 6, 12, 18; tested), `days` (0x40E98F, signed `idiv`), `hint_tile` (0x40AFB0 whole),
`adventure_panel_hint` (0x403010 whole: an empty slot's text the empty string 0x691210), the
special ground's and the default paths (0x40B215, 0x40D0E7), the monster's creature bound
(0x40C350, the subclass read signed), the Buoy's case (0x40B3D0: name, entrance test, bonus by
`kinds_visited`); `OWN_CASES`, `VISITS`, `BONUSES`, the banks' kinds and the keys' records against
§4.10 to §4.12.

Not covered: `NOT_ENTERED`'s 42 classes against the table 0x63AA58 (data, not dumped); the 55
cases' readings beyond what the design gives (only the Buoy's spot-checked); `record.rs`'s classes
and owners the tiles take; the hint's drawing in the status bar (F1) and the harness's writers
(F5; read only where a finding needed it).

## F3-1 A defeat-hero quest without its hero refuses the whole live part

- **Severity:** robustness
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/hover/contents.rs:242` (the `hero` closure, `byte(id)?`),
  `:218` (`mission(...)?` inside `Written::contents`); the writer `clients/capture/h3mem.py:1638`
- **What:** design §4.12 (line 2457) and MCL-Q21 (a) say a defeat-hero quest without its hero
  "shows nothing there", but `Mission::DefeatHero(u8, bool)`/`BeHero` cannot hold one: the
  harness writes the quest's +0x44 as it is (`field(0x44)`, -1 for no hero), `byte(-1)` fails,
  `contents()` returns none, `Facts::read` none, and `render.rs:621` refuses the live part.
- **Failure scenario:** a crafted map whose Seer's Hut (visited) holds a defeat-hero quest with
  hero index -1 (MCL-Q21's case) → `h3map.py still` writes `quests = [[o, 1, s, 3, v, [-1, 0],
  []]]` → the viewer stops with "`[hover]` is not words and armed …" instead of drawing the
  still with the hut's hint lacking the name. The contents tests list `[300, 0]` among the rows
  refused (`hover/contents/tests.rs:147`) but no test pins the -1 case either way.
- **Fix:** take -1 as "no hero" (`DefeatHero(Option<(u8, bool)>)`, same for `BeHero`), give an
  empty name for it in `quest_text`, and test it; no recorded frame changes.

## F3-2 A monster's tile off its entrance shows nothing; the original shows the buffer's old text

- **Severity:** divergence
- **Verdict:** CONFIRMED (the jump); PLAUSIBLE (what the buffer then holds)
- **Where:** `clients/crates/h3-map-view/src/hover.rs:812` (`MONSTER => Hint::Empty`); the test
  `hover/tests.rs:423` (`a_monsters_body_without_its_entrance_has_no_hint`)
- **What:** case 54 tests the entrance bit and, without it, jumps straight to the tail
  (0x40C33E `test [ebx+0xd], 0x10`, 0x40C342 `je 0x40D13B`) without writing the message buffer
  0x6973D8, which the frame does not clear (0x40B150 to 0x40B20E). The status bar then gets
  whatever that buffer last held: the last map hint, or the last hero or town slot's text (a
  panel's pair is set by pointer, 0x403194, and leaves the buffer alone). The code shows nothing,
  and the test's name presents that as the original's; nothing in the design or a question
  records the choice. Every other "nothing" path writes the empty string (0x40D116).
- **Failure scenario:** only a crafted map: a monster whose template has no visitable cell on the
  map (so `tile_object_entrance` gives none and `hint_tile` returns the tile itself, the test's
  case); hovering a guarded chest ("Treasure Chest"), then that monster: the original shows
  "Treasure Chest" again, the viewer an empty bar.
- **Fix:** either keep the last buffer-written hint in the loop (map hints and slot hints set it)
  and show it there, or note the deviation as a chosen fix (MCL-Q21's family: "nothing") in the
  code, the test and §4.10. No recorded capture reaches it.

## F3-3 A table index past its family shows the catalogue key in the status bar

- **Severity:** robustness
- **Verdict:** CONFIRMED (the fallback); PLAUSIBLE (`creature.150` missing: the family's size
  not checked against the data)
- **Where:** `clients/crates/h3-ui/src/text.rs:128` (`Catalogue::text` returns the key itself
  when no pack has it); unbounded lookups at `hover.rs:478` (`hero.{id}`), `:480`
  (`hero-class.{class}`), `:486-487` (`town-name.{n}`, `town-type.{n}`), `:494` (`creature.{n}`,
  `n <= LAST_CREATURE = 150` while the creatures are 0 to 149), `contents.rs:405` (`hero.{id}`),
  `:482` (`artifact.{id}`), `:612` (`spell.{n}`), `:620` (`secondary-skill.{n}`)
- **What:** MCL-Q21 (a), as built, gives an empty name for data past a table, and `entry()`
  does so for the colours, resources, mines, banks, dwellings, tree prices and seers; the
  lookups above bypass it, so a crafted live part or map writes the key's own text.
- **Failure scenario:** a live part with `shrines = [[o, 1, 700, 0]]` (a visited Shrine) →
  "Shrine of Magic Incantation (Learn spell.700)"; a map monster of subclass 150 → "Pack
  creature.150"; `towns = [[999, 12]]` → "town-name.999, town-type.12".
- **Fix:** route these through a lookup that gives the empty text for a missing family entry
  (a `Catalogue` "entry or empty" call, or `entry()` with each family's count); no recorded
  frame changes.

## F3-4 §4.10 counts 58 remaining hint cases; there are 55

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `docs/epics/E24-design.md:2338`
- **What:** "the other 58 cases (visits, owners, contents): T6a2" — of the switch's 59 cases the
  four built in T6a1 (0, 34, 54, 98) leave 55 (35 in §4.11, 20 in §4.12; `OWN_CASES` holds 62
  classes, case 0 four of them), as "the hints' 55 cases" two lines below says.
- **Failure scenario:** a reader reconciling §4.10 with §4.11/§4.12 and `OWN_CASES` looks for
  three cases that do not exist.
- **Fix:** "the other 55 cases". No frame.

## F3-5 A hint test's comment still calls the Arena's case unbuilt

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-map-view/src/hover/tests.rs:344-346`
- **What:** "one with a case T6a2 builds (its name meanwhile)" over `hint(5, 6) ==
  Hint::Object(4)`: T6a2a built the Arena's case (`VISITS` holds 4); the assertion holds only
  because the fixture's facts carry no `visits`, so it no longer tests what it says.
- **Failure scenario:** none at run time; a reader takes the Arena for unbuilt.
- **Fix:** say "an Arena of which the viewer knows nothing: its name alone" (or give it a visit
  and assert `Hint::Visits`). No frame.

## F3-6 The cursor module's doc says the cursor code is unread

- **Severity:** docs drift
- **Verdict:** CONFIRMED
- **Where:** `clients/crates/h3-ui/src/cursor.rs:5-9`
- **What:** the module doc says the frame, hot spot and drawer are provisional because "the
  executable's cursor code is not read yet (E17 M3 …)", the hot spot at the canvas's corner;
  `Cursor::read` (lines 66-69), which both viewers' cursors now use, documents the mouse
  manager's drawer as read (0x50D8B0, E17 M3 T6c-1) and the map's frames and hot spots from the
  layout's `[cursor]`.
- **Failure scenario:** none at run time; a reader takes the map's cursor drawing for a guess.
- **Fix:** limit the "provisional" sentence to `Cursor::new`'s default drawer and hot spot. No
  frame.
