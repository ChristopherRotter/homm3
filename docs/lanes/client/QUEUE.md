# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23, the adventure map client being the map-client lane's E24). M3 (fidelity) done 2026-10-23 with T9d; M4 (sieges) refined at T0 (2026-10-23, with the user); T1a, T1b, T2a and T2b done (2026-10-23), T2c-1 (2026-10-24, T2c cut in two by the user); T2c-2 and T3 below, in order. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M4 T2c-2 — Earthquake, the shots and the mouse recorded

- **Scope:** `WallQuaked` (the shake, copied in every profile, CLT-Q19 (6); the explosions);
  `ROLLS` gains `quake_target`; a shot through the walls (the broken arrow, the halved damage);
  the keep's last kill and the end tune at a town (`Siege` in the record); the siege's
  rectangles 0xFE and 0xFF (a siege only; the original's for the comparisons, CLT-Q21), the
  kinds 16 and 22 and their status lines in the mouse's model. The sequences (§10.2): `quake`,
  `through`, `defended`, `aim`. With the kind 16's hint drawn, T2b's and T2c-1's sequences lose
  their ` log` masks (`Attack Upper Wall` while the Catapult or a Cyclops aims): their frames' counts recorded
  again (`frames --only <id> --reuse` after a capture as recorded; design §10.2 "Recorded").
- **Acceptance:** the sequences as recorded (the timeline check), their frames exact or each
  difference listed; `drawn` the same for every earlier item; `clients/scripts/check.sh` green.
- **Depends on:** T2c-1 (done). **Status:** ready.

### E17 M4 T3 — M4's closure

- **Scope:** a fresh reviewer over M4, every capture afresh, the review list with the user, M5
  (live play) refined with the user.
- **Acceptance:** M4's exit (the epic file); the review list accepted; M5's tasks in this queue.
- **Depends on:** T2c-2. **Status:** waiting for it.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

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
  findings. R2-7: a timeline that steps forward (`Moment::at`, `Heroes::posed` and `Timeline::at`
  rescan the battle for every frame: 2.3 ms a frame over 82 seconds, `play`; for live play's long
  games). R2-8: a record that opens mid-battle (the snapshot's dead stacks as corpses, a raise of
  one drawn; the box by the timeline's living set, not the first snapshot's `alive`). R4-2:
  `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s boxes, the log's
  text: digests recorded again from `master`'s viewer first, the runbook's rule). R4-9: a
  stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an artifact over almost
  anything; the counts stay exact). R3-1 goes with the created stacks below.

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
  shooters since T1b), the Catapult, the gate opening before the step. NOTES, "Sieges"; M4 T2b
  and T2c-1 took the shots, the gate, the moat and the mines; T2c-2 takes Earthquake and the
  mouse (read at T2a, design §10.2).

- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
