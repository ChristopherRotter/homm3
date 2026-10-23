# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23, the adventure map client being the map-client lane's E24). M3 (fidelity) done 2026-10-23 with T9d; M4 (sieges) next, refined by its T0 below. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M4 T0 — M4 (sieges) refined

- **Scope:** M4, the siege battle screen (the user, 2026-10-23, at T9c: sieges, then live play
  as M5; the adventure map went to the new map-client lane, E24), refined into tasks of about half
  a context in the epic file and this queue, as T5c refined M3: what the original draws in a
  siege (a code reading of the drawing's siege branches, named but not read since M1 T5, design
  §4: the town's background and its walls, gate, moat and towers per town, their damaged and
  destroyed states, the turrets as stacks, the Catapult and its shots, Earthquake, the gate's
  opening, the siege's cursors and sounds), what the engine records (E04c: `Siege.parts`,
  `ShootWall`, `WallShot`, `WallQuaked`, `WallDestroyed`, `GateOpened`; the turrets at hexes
  251, 254, 255), the capture set's siege situations and sequences (the harness sets sieges up:
  `h3-experiment`'s `[encounter.opponent.town]`, `experiments/agent/siege.py`) and the open
  questions; the inbox's siege entries (`NOTES.md`, "Sieges") and R2-8/R3-1 where M4 needs them.
- **Acceptance:** M4's tasks in this queue and the epic file, agreed by the user.
- **Depends on:** T9d (done 2026-10-23: M3 done). **Status:** ready.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

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
- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks, the Catapult, the gate
  opening before the step. NOTES, "Sieges"; M4 T0 takes them.
- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)
- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".
