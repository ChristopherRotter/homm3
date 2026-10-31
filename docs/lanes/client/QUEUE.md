# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27, M5 2026-10-31; ADR-0020, ADR-0024; the battle client since the split
of 2026-10-23, the adventure map client being the map-client lane's E24). M6 (the battle screen
complete) refined at M5's closure (T8c, 2026-10-31, with the user): what a played battle shows
of the original's battle screen that M5 did not build, M5's gaps and the inbox's waiting items;
its tasks below, in order, each cut with the user at its start when it outgrows half a context.
M7 (ranked play and the server browser) follows. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M6 T2 — a stack's info

- **Scope:** what a press on a stack gives besides a command (`QUEUE-ARCHIVE.md`, the 2026-10-27
  entry; design §9.11), read and built: the left press on a stack to look at (cursor kind 5)
  opens its stats dialog (`stack_stats_dialog` 0x468860, modal: the creature's picture, its
  numbers, its effects, morale and luck; a creature's cast button, 0x59EC50), the right button
  held over a stack shows it as a popup (0x4769C0, not read yet), a turret's info (kind 22,
  0x47A380); in the window in hot seat and on a server (what the seat's observation shows of an
  enemy stack); stills and a sequence recorded against the original.
- **Acceptance:** each dialog exact against its recordings or each difference listed; review
  items; `check.sh`.
- **Depends on:** T1 (done). **Status:** ready; likely cut at its start (the dialog, then the
  popup and the turret).

### E17 M6 T3 — the right button's other popups

- **Scope:** the right button's popups besides a stack's, read and built: over the battlefield
  (0x4769C0: a hero of either side, the other items the reading finds) and over a control
  (0x472A50: the panel's buttons, the log); recorded.
- **Acceptance:** as T2's. **Depends on:** T2 (the popup). **Status:** ready.

### E17 M6 T4 — the combat options dialog

- **Scope:** the panel's Options (O): the dialog read (its items, keys and defaults), drawn and
  in effect: each setting the original has (the hex grid; the movement and cursor shadows, the
  hex shade design §9.11 does not model yet; the combat speed, whose factors design §7.1 holds;
  the Auto Combat options restricting the window's bot; the others as read), kept in the
  client's settings file (beside `keys.toml`, *provisional*); recorded with settings on.
- **Acceptance:** the dialog and each setting's drawing exact against recordings or listed;
  `check.sh`. **Depends on:** T1 (done). **Status:** ready; likely cut at its start (the dialog,
  then the shadows and the grid).

### E17 M6 T5 — M5's gaps

- **Scope:** what M5's review list (`E17-log.md`, "M5 review list") names as not built, each as
  far as a reading allows: the book's refusal messages (items 5 and 9: `hero_book_button`'s
  refusals and the aim's entry's other messages), the Auto Combat bot's casts and its retreat
  question by T4's options (item 24), Force Field's and Fire Wall's aim (item 84), Eagle Eye's
  message and the experience given after the window (item 79), the Ogre Magi's Bloodlust glow
  (item 42), the effects not drawn yet and their sounds' waits (item 45: the Wraiths' mana
  drain, regeneration, Poison's damage, Death Blow, Ice Bolt's explosion, the Familiars'
  channel).
- **Acceptance:** each built and recorded, or its reason in M6's review list; `check.sh`.
- **Depends on:** T4 (the Auto Combat options). **Status:** ready; cut at its start.

### E17 M6 T6 — the inbox's waiting items

- **Scope:** the items that waited for a free slot (their entries in `QUEUE-ARCHIVE.md`,
  2026-10-31): the deck and the two boats as capture-set situations (CLT-Q5: a hero beside an
  empty boat boards it in the game; `NOTES.md`, "Battlefields and boats"); `h3-bink` decoding as
  `BINKW32.DLL` (CLT-Q30: its float transform in integer arithmetic, checked against the DLL's
  planes; the captures' listed DCT pixels then go); the Demons' dissolve re-investigated (the
  user, 2026-10-29; design §7.25); the compositor's dialog items through `h3_ui::paint`
  (`h3-view`'s copy removed); `tests/purity.rs`'s three holes (R2-15: the map viewer's fixes);
  R4-2 (`drawn`'s digest of what the masks depend on) and R4-9 (a stricter shadow chain in
  `h3cap.redraws`).
- **Acceptance:** each done or its reason recorded; `drawn` and the captures it touches as
  recorded; `check.sh`. **Depends on:** nothing (independent of T1 to T5). **Status:** ready;
  cut at its start (two or three parts).

### E17 M6 T7 — M6's closure

- **Scope:** a fresh reviewer, every capture afresh, the review list, M7 refined with the user
  (ranked play and the server browser: the ladders, the pause by agreement, the browser;
  `NOTES.md`, "The live client"), HD and graphics options placed.
- **Acceptance:** the review list accepted; every capture as recorded or its difference listed;
  M7's tasks in this queue; M6 done. **Depends on:** T1 to T6. **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-31 (the map client lane, E24 M3 T7b2, the review's fixes; for information, nothing to
  do): four changes in the common crates. `h3_ui::paint::draw` saturates a text item's clip and
  a label's colour, and `h3_ui::ui::Dialog::shadow_draws` its strips (a crafted layout at the
  integers' end panicked: the release build checks overflow); `h3_render::blit::darken` cuts its
  rectangle to the screen before its loop (the same pixels); `h3_ui::text::Catalogue::known` is
  new (additive: none for a key no pack has; `text` calls it). Every drawn frame is the same.
  A test now checks that every pixel a drawer changes lies inside `paint::reach`.

- 2026-10-30 (the map client lane, E24 M3 T6b3; for the battle window's sound when next touched):
  the common crates gained the original's sound without the focus (`docs/epics/E24-design.md`
  §4.15: `sound_focus_lost` 0x599C40 and `sound_focus_regained` 0x599B90, the same sound manager
  in a battle): `h3_ui::music::Music::focus` and `Queue::focus` (the music faded out by the stub
  thread once its lock is free, a track opened meanwhile held silent, asked for again at the
  return) and `h3_sdl::Audio::hold` (a stream held where it is: the effects playing at the loss
  resume at the return). `h3-map-view`'s window follows `Window::focused()` with them; `h3-view`
  plays on without the focus. Nothing changed for an existing caller. The harness: `space` is a key
  a sequence may press.

- 2026-10-28 (the map client lane, E24 M3 T2b1; additive, nothing of yours changed): **the
  widening by a table.** `h3_render::screen::Widening` holds every pixel of a format widened
  once (`Screen::xrgb`'s values, 65 536 entries) and widens a screen into a kept vector
  (`widen`) or onto part of a frame (`widen_onto`); also new: `Screen::cut`, `Screen::copy_rect`
  and `h3_ui::cursor::Cursor::rect`. In the map's window the per-pixel `xrgb` was 62 % of a
  frame's cost (`docs/epics/E24-design.md` §4.2); `h3-view`'s window widens every frame the same
  way and could take it.

- 2026-10-27 (the map client lane, E24 M2 T6a2; for the battle's end tune when its sound is next
  touched): **the music thread read whole** (`docs/exe/map.toml`: `play_music` 0x59ACB0,
  `music_thread` 0x59A840, `music_stop_fade` 0x59B080; E24 design §3.11, "The music's thread").
  For `h3-view`: the end's tune does not cut the combat music; the thread fades it out first
  (900 ms by tenths from its volume, its place kept), then opens the tune, so the tune starts
  900 ms after `play_music`, and a battle ended within 900 ms of the music's start waits for the
  fade-in's end too (the thread holds the manager's +0xA8 through it). `h3_map_view::music`
  models the thread (looping tracks only: a track played once starts at its volume from its
  start and is never resumed), a candidate for M3 T0's common crate.

- 2026-10-27 (the map client lane, E24 M2 T3b; for you when a translucent DEF of encoding 3
  appears): `draw_frame_blend` 0x47D4F0 is read (`docs/exe/map.toml`; E24 design §3.8): it blends
  encoding 3's literal pixels half and half with the screen as 0x47EF60 blends encoding 1's, and
  skips its runs (both callers pass no colour). `h3_render::blit`'s `Drawer::Translucent` still
  draws encoding 3 plain (*provisional*: no magic animation has it); the map view blends with its
  own drawer (`h3_map_view::draw::blend`), so nothing in the foundation changed.

- 2026-10-24 (the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; **for you**): **the wire carries the viewers' facts**
  (protocol v35): `Observation.background` (0 to 2, the battlefield picture's variant on dirt,
  grass and snow; `None` not known: take the third picture, as now), `SideView.colour` (the
  side's player colour, 0 red to 7 pink; `None` no player or not known: red for the attacker,
  blue for the defender, as now) and `Hero.sex` (`wire::Sex`; `None` the hero type's own, the
  asset tables' `[[hero]]`). Your wire literals got `background: None`, `colour: None` and
  `sex: None` (`h3-view`'s `record.rs`, the mouse, scene and panel tests); nothing reads them yet.
  The scenarios and the battle replay file (format v31: `background`, `attacker_colour`,
  `defender_colour`, a hero's `sex`, each omitted when not known) may name them; the adventure
  map's battles will set them (the adventure lane's inbox). CLT-Q4 and CLT-Q6 are answered.

- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".

- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".

- CLT-Q4: the background variants 0 and 1. NOTES, "Battlefields and boats" (the deck and
  the two boats: M6 T6).

- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): M5 T6 and T7 took the record file, the
  content check, a server restart and accounts; the ladders, the pause by agreement and the
  server browser are M7's, after M6. NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
