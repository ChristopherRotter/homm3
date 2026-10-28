# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23,
the adventure map client being the map-client lane's E24). M5 (live play) refined at M4's
closure (T3c, 2026-10-27, with the user): hot seat in-process first, then a game on a server;
its tasks below, in order (T1 and T2 done 2026-10-27, each cut in two by the user; T2b whole,
the user; T3 cut in two, T3a done 2026-10-27, T3b 2026-10-28; T4 cut in three, 2026-10-28). The original
files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M5 T4 — the battle's dialogs

Cut in three by the user on 2026-10-28 (T4a, T4b, T4c below, in order). The detail:
`QUEUE-ARCHIVE.md`, 2026-10-21 (T8) and 2026-10-20 (T6a).

#### T4a — the message box, Retreat and Surrender

- **Scope:** the message box (`message_box_build` 0x4F6990: its frame, text, buttons and
  picture), the spell book's refusals (a spell above the hero's mana GENRLTXT 207, "That spell
  will affect no one!" GENRLTXT 186) and an adventure spell's description (T3a's; design §7.17);
  Retreat and Surrender with their confirmations and the surrender's price; read, drawn,
  recorded. `hero_book_button` 0x59E900's refusals need facts the client lacks: listed.
- **Acceptance:** each box's still as recorded, or each difference listed; battles ended by a
  surrender and a retreat in the window; the Definition of Done.
- **Depends on:** T2b (done 2026-10-27). **Status:** in progress (2026-10-28).

#### T4b — the result window

- **Scope:** the result window (`battle_result_dialog` 0x4702D0: the heroes, the texts, the
  casualties, OK; the screen darkened under it, 0x44E5F0 from 0x47782C: `end`'s last frame
  differs by them now); read, drawn, recorded, with sequences through a retreat and a surrender.
- **Acceptance:** its still and sequences through it as recorded, or each difference listed;
  battles ended by a surrender, a retreat and a victory in the window, each to its window; the
  Definition of Done.
- **Depends on:** T4a. **Status:** ready after T4a.

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
