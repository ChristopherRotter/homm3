# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23,
the adventure map client being the map-client lane's E24). M5 (live play) refined at M4's
closure (T3c, 2026-10-27, with the user): hot seat in-process first, then a game on a server;
its tasks below, in order (T1 and T2 done 2026-10-27, each cut in two by the user; T2b whole,
the user; T3 cut in two, T3a done 2026-10-27, T3b 2026-10-28; T4 cut in three, T4a done
2026-10-28, T4b cut in two, T4b1 done 2026-10-28). The original
files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M5 T4 — the battle's dialogs

Cut in three by the user on 2026-10-28 (T4a, the message box, Retreat and Surrender, done
2026-10-28; T4b cut in two, T4b1 done 2026-10-28; T4b2 cut in two, T4b2a and T4b2b done
2026-10-28; T4c below). The detail: `QUEUE-ARCHIVE.md`, 2026-10-21 (T8) and
2026-10-20 (T6a).

#### T4c — Auto Combat and the tactics phase

- **Scope:** Auto Combat (its button selected, the panel dimmed while the stacks play
  themselves), the tactics phase's panel (0x46C050) and its end; read, drawn, recorded.
- **Acceptance:** each still and a sequence through each as recorded, or each difference
  listed; the Definition of Done.
- **Depends on:** T2b (done 2026-10-27). **Status:** ready.

### E17 M5 T5 — what a played battle shows

- **Scope:** the created stacks (`StackCreated`: Summon Elemental, Clone; a Pit Lord's
  `RoseFromCorpse`): their numbers, their record index and the dissolve that shows them
  (`create_battle_stack`, 75 ms steps; `screen_dissolve` 0x602DC0's blend, Sacrifice's victim's
  too), the message when no hex is free, a clone's `Vanished`, Teleport's dissolve; the panel's
  side by each turn's controller (R3-1: Hypnotize, a created stack); the battle waiting for the
  heal's sound and the other effects'. The detail: `QUEUE-ARCHIVE.md`, 2026-10-22 (T7c-3).
  Recorded against the original.
- **Acceptance:** the new sequences as recorded, or each difference listed; `sacrifice`'s
  dissolve exact; the Definition of Done.
- **Depends on:** T1b (done 2026-10-27). **Status:** ready.

### E17 M5 T6 — a game on a server

- **Scope:** the window connects to `h3-server` over its WebSocket: the hello (a fresh nonce, the
  address dialed), the welcome's checks (its signature, its `id` the address dialed), the key
  pinned per address, a game joined by its seat token, the content check (the manifest's packs
  and profile compiled, the ruleset hash compared: the previews only when equal), the commands
  sent and the record growing from the server's messages; a record that opens mid-battle (R2-8:
  the snapshot's dead stacks as corpses, the box by the timeline's living set; M4 T3a's R1-3:
  the moat's strip by the fixed moat rows, 0x63BCE8 and 0x63BCF4) for a join and a re-join after
  a drop or a server restart (the numbering afresh); the end's `remote_turn`. The server lane's
  notes: `NOTES.md`, "The live client"; the detail: `QUEUE-ARCHIVE.md`, 2026-10-23 (T9b). Its
  dependency (a WebSocket client with TLS): asked of the user before it is added.
- **Acceptance:** a battle played to its end in two windows on a local `h3-server`, a seat each;
  a drop and a re-join; both outcomes of the content check; tests over loopback where the
  adapter allows; the Definition of Done.
- **Depends on:** T2b (done 2026-10-27); the user's choice of the dependency. **Status:** ready
  (the dependency asked at the task).

### E17 M5 T7 — accounts

- **Scope:** the account's key from the login and the password (argon2id with the server lane's
  parameters, the seed as Ed25519's secret key; the test vector in `NOTES.md`, "The live
  client"), derived once per device and cached (never the password); the login signed only for
  the address dialed, a registration's display name; seats by account (joined without a token,
  re-taken after a drop); `Rekey` after a password change; the goodbyes `Rekeyed`,
  `CommandLimit`, `Abandoned`. Its dependencies (argon2, Ed25519): asked of the user at the task.
- **Acceptance:** the test vector reproduced; a battle played logged in on a local `h3-server`,
  the seat re-taken after a drop; the Definition of Done.
- **Depends on:** T6; the user's choice of the dependencies. **Status:** ready after T6.

### E17 M5 T8 — M5's closure

- **Scope:** a fresh reviewer, the battle recipes' starting bonus set to Gold (M4's review item
  15: `to_scenario` and hot seat leave the scenario list's bonus random, which gave a hero an
  artifact for one run at M4 T3b-2), every capture afresh, the review list, the next milestone
  refined with the user (the ladders, the pause, the server browser; HD; graphics options).
- **Acceptance:** the review list accepted; every capture as recorded or its difference listed;
  the next milestone's tasks in this queue; M5 done.
- **Depends on:** T1 to T7. **Status:** waiting (the user, at its checkpoint).

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-28 (the map client lane, E24 M3 T2b1; additive, nothing of yours changed): **the
  widening by a table.** `h3_render::screen::Widening` holds every pixel of a format widened
  once (`Screen::xrgb`'s values, 65 536 entries) and widens a screen into a kept vector
  (`widen`) or onto part of a frame (`widen_onto`); also new: `Screen::cut`, `Screen::copy_rect`
  and `h3_ui::cursor::Cursor::rect`. In the map's window the per-pixel `xrgb` was 62 % of a
  frame's cost (`docs/epics/E24-design.md` §4.2); `h3-view`'s window widens every frame the same
  way and could take it.
- 2026-10-28 (E17 M5 T4b2b, CLT-Q30, the user: listed now, exact later): **`h3-bink` to decode
  as the game's `BINKW32.DLL` (Bink 0.5b).** Its inverse DCT is x87 single-precision floating
  point (`ExpandPlane`'s 8 × 8 transform: √2, 1.848, 2.613 and 1.082 as floats, float
  quantizers), FFmpeg's integer, so five clips differ by one level on a few pixels (`rtstart`,
  `rtloop`, `Surrender`, `defendloop`; `Defendall` up to 388 a frame; design §7.21). The task:
  the DLL's transform and quantization in integer arithmetic (no floats in the client: exact
  binary32 emulation or a proof that a fixed-point form rounds alike on every input the
  bundles allow), checked against the DLL's own planes (`binkprobe frames <vid> <clip> 7`:
  YUY2) on the ten clips and the synthetic samples; FFmpeg's check (`bink-check.sh`) then
  becomes the DLL's. Then the captures' listed DCT pixels (`result-retreat` 2 700 and 3 000,
  `defended`) go.

- 2026-10-28 (the map client lane, E24 M3 T2a; additive, nothing of yours drawn changed): **the
  dialog items' drawers in the common crate.** `h3_ui::paint::draw` draws each `h3_ui::ui::Draw`
  (picture, button, text, label, darken) as `h3_view::compose` does (ported from it, the same
  pixels) from any batch implementing `h3_ui::paint::Assets` (`sprite`, `picture`, `palette`,
  `font`), and `h3_ui::paint::names` gives the assets a draw needs; the map viewer draws its
  adventure dialog with it. M3's exit wants `h3-ui` the only copy of what both viewers share:
  your compositor's `Draw::Dialog` arms and `batch.rs`'s names switched to it (your `Batch`
  implementing `Assets`) remove your copy. Also additive: `Dialog::draws` gives a button the
  player's colours (`Shown::players`) and a DEF (`Shown::defs`) by its id, which no battle
  dialog sets (`h3drive.py drawn` all the same).
- 2026-10-28 (the map client lane, E24 M3 T1b; **your window's keys changed**; the user: every
  key through the table, no cut): **the battle window's keys go through `h3_ui::keys`** (#548,
  `cross-lane`, `map-client/e24-m3-t1b`, named in the map client lane's HANDOFF until it merges). The new `h3_view::keys` holds the battle's commands with their names and the
  original's keys by **context** (`Context::of`: your M5 T4b1's result window, else a message box,
  else the book, else an aim, else the battle): the window's own (`h3_ui::keys::Window`, the map's: F2 music, F3 the creatures'
  sounds, F5 pause, F6 a frame on, F7 restart, F8 vsync, F9 the cursor, F11 fullscreen, Esc
  quit; P, →, Home, M, E, V and H gone), `log-up`/`log-down` (↑ ↓), `dragon-spell` (F),
  the panel's seven buttons (`BUTTONS`: S, R, O, A, C, W, D and Space), the book's
  (`book::Key`: Esc and Enter close, A, C, ← → ↑ ↓) and the box's (`message::Key`: Enter OK,
  Esc Cancel), `aim-cancel` (Esc, the right button), `result-ok` (Enter and Escape, 0x4715C3;
  merged with #547: `result::window`'s OK and the stills' `one_button` lost their `keys`). The user's `keys.toml` (`--keys`, else the
  working directory) has a `[battle]` table (the map's `[map]`). API changes: `Book::key` and
  `book::Open::key` take a `book::Key`; `message::Open::key` a `message::Key` and the key's name
  (its release still by name); `h3_ui::ui::Button` lost `keys` and `Dialog::button_for_key`
  (the layout's panel `keys` gone: the table's defaults are their only copy); the stills' `key:`
  actions go through `Keys::original()` by context; `h3_sdl::Key::ALL` gained the missing
  `Return`. Behaviour changed: the book takes every key while open (the panel's letters and F
  reached the battle under it); unattended, Esc quits whatever is open and the right button no
  longer gives an aim up; a chord with Ctrl held gives nothing unless bound. `viewer_key` and
  `book_key` are gone. The 2026-10-26 entry below (custom key bindings) is done by it; your
  M5 T3b's F is the table's `dragon-spell`. A branch of yours merges `master` in after it merges.

- 2026-10-28 (the map client lane, E24 M3 T0; **decided by the user for both lanes**; the
  proposal of 2026-10-27 as it stood: `docs/lanes/map-client/QUEUE-ARCHIVE.md`): **the common crate
  `h3-ui`** (`clients/crates/h3-ui`, common to both client lanes as the foundation is: a
  `shared/` pull request for `lanes.toml` first) takes `h3-view`'s `clock`, `cursor` with
  `batch::Sprite`, `ui` (with draw items of its own instead of `scene::Draw`'s) and `text`, and
  `h3-map-view`'s `keys` (generic over a viewer's commands) and `music`; `layout` and `mouse`
  stay in `h3-view` (battle-only). The map client lane moves it as **E24 M3 T1a** in a
  `cross-lane` pull request (the user's consent): `h3-view`'s imports change, nothing it draws
  or plays. Then **E24 M3 T1b** takes `h3-view window`'s keys through the table (the 2026-10-26
  entry below), **both windows' own keys the map's** (the user: F2 music, F3 effects, F5 pause,
  F6 the next frame, F7 restart, F8 vsync, F9 the cursor's kind, F11 fullscreen, Esc quit; P,
  →, Home, M, E, F, V and H gone), so M5 T3b's F key is free. Starting M5 T3b before T1a and
  T1b merge means merging `master` in after them (the imports); the map client lane tells you
  here when each merges.

- 2026-10-27 (the map client lane, E24 M2 T6a2; for the battle's end tune when its sound is next
  touched): **the music thread read whole** (`docs/exe/map.toml`: `play_music` 0x59ACB0,
  `music_thread` 0x59A840, `music_stop_fade` 0x59B080; E24 design §3.11, "The music's thread").
  For `h3-view`: the end's tune does not cut the combat music; the thread fades it out first
  (900 ms by tenths from its volume, its place kept), then opens the tune, so the tune starts
  900 ms after `play_music`, and a battle ended within 900 ms of the music's start waits for the
  fade-in's end too (the thread holds the manager's +0xA8 through it). `h3_map_view::music`
  models the thread (looping tracks only: a track played once starts at its volume from its
  start and is never resumed), a candidate for M3 T0's common crate.

- 2026-10-27 (E17 M5 T2a, the click read; for M5 T4 or later): **the stack's and the turret's
  info by the mouse** are not built: a left press on a stack to look at (kind 5) opens its stats
  dialog (`stack_stats_dialog` 0x468860, modal), on a turret (22) its info (0x47A380); the right
  button pressed over the battlefield shows a popup while held (0x4769C0, not read), over a
  control its help (0x472A50). `mouse::click` gives no command there (design §9.11).

- 2026-10-27 (the map client lane, E24 M2 T3b; for you when a translucent DEF of encoding 3
  appears): `draw_frame_blend` 0x47D4F0 is read (`docs/exe/map.toml`; E24 design §3.8): it blends
  encoding 3's literal pixels half and half with the screen as 0x47EF60 blends encoding 1's, and
  skips its runs (both callers pass no colour). `h3_render::blit`'s `Drawer::Translucent` still
  draws encoding 3 plain (*provisional*: no magic animation has it); the map view blends with its
  own drawer (`h3_map_view::draw::blend`), so nothing in the foundation changed.

- 2026-10-26 (the map client lane, E24 M2 T1b; **for you, at M3 T0**): **custom key bindings**
  (the user, 2026-10-26: keys hard-coded so far are to be phased into bindings the player can
  change). The map client's window (E24 M2 T4) takes every key through a table of actions with
  the original's keys as defaults and a user file overriding them; at E24 M3 T0 that table moves
  into the common crate both lanes share (with `clock`, `cursor`, `ui`, …), and `h3-view
  window`'s keys (now matched in its event loop, `window.rs`) go through it too. Nothing to do
  before M3 T0's proposal; `h3-sdl`'s `Key` grows additively meanwhile (a common path).

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

- 2026-10-24 (the map-client lane, E24 M1 T6b's reviewer, R2-15; for you to take or leave):
  **`h3-view`'s `tests/purity.rs` has three holes** its copy in `h3-map-view` had: `sources` skips
  every folder named `bin` at any depth (only `src/bin/` is the adapter), a denied lint relaxed
  through its group (`#[allow(clippy::restriction)]`) or by an attribute over several lines is not
  seen, and the required header is matched as a substring (a commented-out `// #![no_std]`
  passes). `clients/crates/h3-map-view/tests/purity.rs` (E24 M1 T6b) has the fixes:
  `walk` from `src` leaving out `src/bin` only, `attributes` joining an attribute's lines, the
  `GROUPS` list, the header as a line of its own.

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R4-2: `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s
  boxes, the log's text: digests recorded again from `master`'s viewer first, the runbook's
  rule). R4-9: a stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an
  artifact over almost anything; the counts stay exact). (R2-7, R2-8 with R1-3 and R3-1 went into
  M5 T1, T6 and T5 at M4 T3c; the entry's full text in `QUEUE-ARCHIVE.md`.)

- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".

- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".

- When CLT-Q5 is answered: the deck and the two boats as situations; CLT-Q4: the background
  variants 0 and 1. NOTES, "Battlefields and boats".

- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): M5 T6 and T7 take the record file, the
  content check, a server restart and accounts; the ladders, the pause by agreement and the
  server browser come after M5. NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
