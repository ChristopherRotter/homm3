# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27, M5 2026-10-31, M6 2026-11-10; ADR-0020, ADR-0024; the battle
client since the split of 2026-10-23, the adventure map client being the map-client lane's
E24). M7 (the battle client's last steps) refined at M6's closure (T7c, 2026-11-10, with the
user): M6's answers built, the panel buttons' own loop, the battle as an entry a host's window
runs, then the closure, **which closes E17** (the user: ranked play, the pause and the server
browser wait for E27 and a refinement after it). Its tasks below, in order, each cut with the
user at its start when it outgrows half a context. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M7 T1c — the reads and captures the answers name

T1 was cut in four with the user at its start (2026-11-10): T1a and T1b done, T1c, T1d.

- **Scope:** Q71's capture (hold D, click a stack, release D, close the dialog: the original's
  profile copies what it shows); Q73's 0x5994B0 (read; the client's kept either way, the record
  says fix or copy); Q74's sound sequence pressing a speed (T1a built one click); T7b's R2-1
  still `stats-ability`.
- **Acceptance:** the new captures as recorded or their differences listed; the records in the
  answers; `drawn` all the same.
- **Depends on:** nothing (T1b built Q71's client's profile, `window::release_taken`; the
  original's left held, as built). **Status:** next.

### E17 M7 T1d — the tools

- **Scope:** T7b's R10-2 leftovers: `Changes::after` and `cast_stop` under test, a presser option
  closing the result window for an unattended `--run`.
- **Acceptance:** the tests; the option documented in the runbook; `check.sh` passes.
- **Depends on:** nothing. **Status:** waiting.

### E17 M7 T2 — the DEF button's own loop

- **Scope:** CLT-Q72 (a), T7b's R3-4, M6 review item 52: `def_button_message` 0x45641C modelled
  for the panel's buttons, the stats dialog's OK and the combat options' Default, OK and speeds,
  in both profiles: while the left button holds an enabled button, keys and the right press are
  dropped and no right-button popup opens; the right button's release ends the hold, clicking
  when the button is drawn pressed; a disabled button runs no loop (its help opens, as now).
  R3-4's stale `over` goes with it; CLT-Q39's mouse case and R4-2 (a)'s dropped right press
  folded in.
- **Acceptance:** unit tests of each case; a sequence or still recorded where the harness can
  hold a button (else listed); `drawn` all the same or recorded.
- **Depends on:** nothing (T1b's Q71 let-go, `window::release_taken`, touches the same held
  state). **Status:** waiting.

### E17 M7 T3 — the battle as a hosted entry

- **Scope:** the map client lane's request for E24 M4 T5 (the inbox of 2026-10-31, now in
  `QUEUE-ARCHIVE.md`): an entry of `h3-view`'s library that a host's window runs: a battle
  started from a whole game's battle messages (the protocol's battle types as E16 M5 carries
  them; record format 4), drawn in the host's window with a clock and an audio output the host
  owns, played to its result window and handed back with its outcome; `h3-view window` becomes
  one host of it. With it: the wire's battlefield picture, colours and sex read (protocol v35:
  `Observation.background`, `SideView.colour`, `Hero.sex`; CLT-Q4, CLT-Q6; the battle lane's
  note of 2026-10-24); the music through the common `h3_ui::music` (the end's tune after the
  combat music's 900 ms fade-out, the map client lane's note of 2026-10-27; the focus lost and
  regained, its note of 2026-10-30); the frame widened by `h3_render::screen::Widening` if it
  measures faster (its note of 2026-10-28). The entry's shape (its types, who owns the event
  loop) agreed with the map client lane at the task's start (its inbox).
- **Acceptance:** `h3-view window` runs its battles through the entry with `drawn` all the
  same; a test host plays a whole game's battle to its result and gets the outcome back; the map
  client lane's inbox told the entry's API.
- **Depends on:** E16 M5 part 1's whole-game record (merged). **Status:** waiting.

### E17 M7 T4 — M7's and E17's closure

- **Scope:** a fresh reviewer; the captures where M7 changed the battle screen; the review list
  with the user; E17's open threads placed (the epic's "Later": HD layouts, graphics options,
  the client's options, video, M6 T5e; this inbox's waiting entries; ranked play's notes for the
  refinement after E27); E17 closed (`docs/BACKLOG.md`'s row, and E27's draft, which still names
  E17 M7 for the lobby and the browser, by a `shared/` pull request).
- **Acceptance:** the review list accepted; every open thread placed; E17 done.
- **Depends on:** T1b to T1d, T2, T3. **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-27 (the map client lane, E24 M2 T3b; for you when a translucent DEF of encoding 3
  appears): `draw_frame_blend` 0x47D4F0 is read (`docs/exe/map.toml`; E24 design §3.8): it blends
  encoding 3's literal pixels half and half with the screen as 0x47EF60 blends encoding 1's, and
  skips its runs (both callers pass no colour). `h3_render::blit`'s `Drawer::Translucent` still
  draws encoding 3 plain (*provisional*: no magic animation has it); the map view blends with its
  own drawer (`h3_map_view::draw::blend`), so nothing in the foundation changed.

- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".

- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".

- CLT-Q4: the background variants 0 and 1. NOTES, "Battlefields and boats".

- Later (placed at E17 M7 T4): graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): M5 T6 and T7 took the record file, the
  content check, a server restart and accounts; the ladders, the pause by agreement and the
  server browser are not E17's (the user, 2026-11-10: after E27, by a refinement of their own;
  the first ranked play through the website, E21). NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
