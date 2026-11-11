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
check; CI's job runs the same), the real-asset gate `cargo run -p h3-battle-client -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M7 T3 — the battle as a hosted entry

- **Scope:** the map client lane's request for E24 M4 T5 (the inbox of 2026-10-31, now in
  `QUEUE-ARCHIVE.md`): the battle as an element a host's window runs (the shape agreed with the
  user for both lanes, 2026-11-11: the host owns the window, its loop and the frame, the battle
  drawn natively as any element of it; design §13.2), started from a whole game's battle
  messages and handed back with its outcome. **Cut in four** with the user (2026-11-11):
  - **T3a1 done**, **T3a2 done** (2026-11-11, branch `claude/ecstatic-thompson-cjowg2`; T3a2
    checked with the data staged by the next session): the crate `h3-battle-client` (h3-view's
    binary until then), the element `window::Hosted` with `h3-view window` its host.
  - **T3b**, cut in two with the user (2026-11-11):
    - **T3b1 done** (2026-11-11, branch `client/e17-m7-t3b1`; design §13.3): a whole game's
      record's battle played and handed back with its outcome; the wire's picture variant and
      colours read.
    - **T3b2 done** (2026-11-11, branch `client/e17-m7-t3b2` after #851; design §13.4,
      §13.5): the same battle played live through the host's game (`game_seat::GameSeat`,
      `Ground::load_game`; a test host's real game to its result and outcome); the hero's battle
      picture by the map's sex in the client's profile (CLT-Q78 (b)).
  - **T3c:** the sound on the host's device, the music through the common `h3_ui::music` (the
    end's tune after the combat music's 900 ms fade-out, the map client lane's note of
    2026-10-27; the focus lost and regained, its note of 2026-10-30); the frame widened by
    `h3_render::screen::Widening` if it measures faster (its note of 2026-10-28).
- **Acceptance:** `h3-view window` runs its battles through the entry with `drawn` all the
  same; a test host plays a whole game's battle to its result and gets the outcome back; the map
  client lane's inbox told the entry's API.
- **Depends on:** E16 M5 part 1's whole-game record (merged). **Status:** T3c next (T3a and
  T3b done 2026-11-11).

### E17 M7 T4 — M7's and E17's closure

- **Scope:** a fresh reviewer; the captures where M7 changed the battle screen; the review list
  with the user; E17's open threads placed (the epic's "Later": HD layouts, graphics options,
  the client's options, video, M6 T5e; this inbox's waiting entries; ranked play's notes for the
  refinement after E27); E17 closed (`docs/BACKLOG.md`'s row, and E27's draft, which still names
  E17 M7 for the lobby and the browser, by a `shared/` pull request).
- **Acceptance:** the review list accepted; every open thread placed; E17 done.
- **Depends on:** T3 (T1, T2 done). **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

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

- CLT-Q4: the background variants 0 and 1 (read from the wire since E17 M7 T3b1; not recorded:
  a situation needs trees, mountains or dead vegetation around the battle). NOTES,
  "Battlefields and boats".

- Later (placed at E17 M7 T4): graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): M5 T6 and T7 took the record file, the
  content check, a server restart and accounts; the ladders, the pause by agreement and the
  server browser are not E17's (the user, 2026-11-10: after E27, by a refinement of their own;
  the first ranked play through the website, E21). NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
