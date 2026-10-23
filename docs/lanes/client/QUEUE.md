# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23, the adventure map client being the map-client lane's E24). M3 (fidelity) done 2026-10-23 with T9d; M4 (sieges) refined at T0 (2026-10-23, with the user); T1a and T1b done (2026-10-23); T2a to T3 below, in order. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M4 T2a — the siege over time read

- **Scope:** a code reading of what moves in a siege: the Catapult's shot (`wall_shot`
  0x445D30, `wall_shot_animate` 0x445FD0: the rock, the explosion, the part's picture at frame 5,
  the sounds), a turret's shot (`turret_shot` 0x465AD0: from which point, which missile), the
  drawbridge (`gate_open` 0x4671C0, `gate_close` 0x4672E0: the original opens the gate as a
  defender begins its step onto hex 96, the engine reports `GateOpened` after the `Moved`, so
  the timeline reorders them; NOTES, "Sieges"), Earthquake (0x5A7C80), the moat's damage and the
  Tower's mines, the siege's cursors (the kinds 16, the Catapult's wall targets, and 22, the
  arrow towers; the mouse's siege rectangles 0xFE and 0xFF: design §9.9), its status lines and
  log lines, the end tune at a town (design §8.2). The timeline's
  plan for the siege events; T2b's and T2c's sequences listed, each with its forced rolls.
- **Acceptance:** the design section's second part, each cited address in the map; T2b's and
  T2c's sequence lists; each oddity a `CLT-Q<n>`.
- **Depends on:** T1a (done). **Status:** ready.

### E17 M4 T2b — the Catapult and the turrets recorded

- **Scope:** the timeline plays `WallShot` and `WallDestroyed` (the Catapult's and a Cyclops'
  shot) and the turrets' shots by T2a's reading, with their sounds; hot-seat siege sequences
  with the original's rolls forced (the runbook's "Forced rolls"): a hit, a miss, a wall and a
  tower destroyed, the keep's and the towers' shots on a stack.
- **Acceptance:** the sequences as recorded (the timeline check), their frames exact or each
  difference listed; `drawn` the same for every earlier item; `clients/scripts/check.sh` green.
- **Note (T1b):** a war machine acting on its own before the first command (the turrets every
  round first, then the Catapult) acts before the engine's record opens (`BattleState::new`
  drops the start's events, `h3-protocol`'s session uses it): a sequence's shots at the opening
  come from commanded machines (Ballistics, Artillery: the T1b situations' heroes), automatic
  ones need the start's events in the record (another lane's change: ask first). A hit part's
  picture state (1) comes from the timeline then, `Moment::walls` (T1b's override is
  *provisional*).
- **Depends on:** T2a. **Status:** waiting for it.

### E17 M4 T2c — the gate, the moat, Earthquake and the mouse recorded

- **Scope:** `GateOpened` and `GateClosed` (the drawbridge before the step), `MoatDamage` and a
  Tower's mine, `WallQuaked`; sequences: a defender out through the gate and back, a stack in the
  moat, a mine, Earthquake, a hero with Ballistics aiming the Catapult, a shot through the walls;
  the siege's cursors and status lines in the mouse's model; with them the moat's strip over a
  stack and its box drawn again, and the moving drawbridge (state 2), which no T1b still shows.
- **Acceptance:** as T2b's.
- **Depends on:** T2b. **Status:** waiting for it.

### E17 M4 T3 — M4's closure

- **Scope:** a fresh reviewer over M4, every capture afresh, the review list with the user, M5
  (live play) refined with the user.
- **Acceptance:** M4's exit (the epic file); the review list accepted; M5's tasks in this queue.
- **Depends on:** T2c. **Status:** waiting for it.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-23 (the map-client lane, E24 T0; for later, nothing to do now): **what the map client
  will need of `h3-view`.** E24 M1 (stills of the map view) builds only on the common foundation;
  M2's window will want `clock` and `sound`, M3's panels `ui`, `text` and `layout`, in a common
  crate. When M2 or M3 is refined, the map-client lane proposes the move here first (a new common
  crate is a `lanes.toml` change, shared); your M4 waits for none of it.
- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R2-7: a timeline that steps forward (`Moment::at`, `Heroes::posed` and `Timeline::at`
  rescan the battle for every frame: 2.3 ms a frame over 82 seconds, `play`; for live play's long
  games). R2-8: a record that opens mid-battle (the snapshot's dead stacks as corpses, a raise of
  one drawn; the box by the timeline's living set, not the first snapshot's `alive`). R4-2:
  `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s boxes, the log's
  text: digests recorded again from `master`'s viewer first, the runbook's rule). R4-9: a
  stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an artifact over almost
  anything; the counts stay exact). R3-1 goes with the created stacks below.

- 2026-10-21 to 22 (the adventure lane, E08 T32b1 to T37b; for information, nothing to do): the
  harness's reads and steps for batches E4 and E5 (moved to NOTES, "The adventure lane's
  harness", at T9c; the harness is common since the split, 2026-10-23).

- 2026-10-21 (T7b-1): **a hero's aim.** Between the book's place and the target's click the
  cursor is the spell's (`crspell.DEF`, animated: not read; the viewer leaves it as it was) and the
  status line says "Cast %s on %s" (GENRLTXT 28, built *provisional*, not forced; 27 elsewhere);
  `enter_player_cast` 0x59EC50's loop is not read (which targets it offers, the cursor's frames):
  with the mouse's model when a recording aims a hero's area spell at a hex (T7c-1's are the
  Faerie Dragon's clicks), and for live play. And `cast`'s 71
  pixels at 0: the original redraws the battlefield over a rock's top (obstacle 22) for the one
  frame of the click (0x479038's path, not read).

- 2026-10-21 (T8): the battle's result window (`battle_result_dialog` 0x4702D0, not read beyond
  its tune: the heroes, the texts, the casualties, OK) and the whole screen darkened under it
  (0x44E5F0 from 0x47782C): `end`'s last frame differs by them (its counts). The volume settings
  (0x5996C0) and a track resuming where it stopped (0x59AA82) with the client's options and a
  second battle (later); the network's `remote_turn` in the end's choice with live play.

- 2026-10-20 (the adventure lane, E08 T22a2b): its three recipes, its auto combat's change, for
  information (NOTES, "The adventure lane's harness"; the winner of −1 fixed in E17 M3 T6c-2).
- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".
- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".
- 2026-10-22 (T7c-3): **created stacks are not drawn.** `StackCreated` (Summon Elemental, Clone)
  and a Pit Lord's `RoseFromCorpse` add a stack the record's first snapshot lacks, and the scene
  draws only the snapshot's stacks (a wire `Stack` each): the viewer needs the new stack's
  numbers (from the ruleset, or a later snapshot in the record), its record index and the
  dissolve that shows it (`create_battle_stack`, 75 ms steps, the blend not read; T9d built a
  dissolve's time for Sacrifice's victim, `anim::GONE`, `DISSOLVE_STEP`, and the frame check of
  `sacrifice` lists its blend: `screen_dissolve` 0x602DC0, read for both); with them the
  message when no hex is free, a clone's `Vanished`, Teleport's dissolve (moved at once now) and
  Sacrifice's victim leaving no corpse; and the panel's side by each turn's
  controller (T9b, R3-1: `scene::controller` takes the opening's, so Hypnotize and a created stack
  are not followed). The battle waits for the heal's sound (T3b) and the other
  effects' (T7b-2b, T7c-3). NOTES, "The battle screen".
- 2026-10-20 (T6a): the tactics phase's panel (0x46C050, not read) and Auto Combat switched on
  (its button selected, the panel dimmed while stacks play themselves) wait for a record with a
  tactics phase and for live play. Design §9.7.
- When CLT-Q5 is answered: the deck and the two boats as situations; CLT-Q4: the background
  variants 0 and 1. NOTES, "Battlefields and boats".
- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks (drawn as the town's
  shooters since T1b), the Catapult, the gate opening before the step. NOTES, "Sieges"; M4 T2a
  to T2c take them.
- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)
- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".
