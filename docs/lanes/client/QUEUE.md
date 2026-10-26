# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23, the adventure map client being the map-client lane's E24). M3 (fidelity) done 2026-10-23 with T9d; M4 (sieges) refined at T0 (2026-10-23, with the user); T1a, T1b, T2a and T2b done (2026-10-23), T2c-1 (2026-10-24, T2c cut in two by the user), T2c-2 (2026-10-26); T3 below, cut in three (T3a to T3c). The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

M4's closure, T3, cut in three by the user (2026-10-26): the review first, so that every capture
afresh also captures its fixes.

### E17 M4 T3a — M4's review: the reviewers' findings fixed

- **Scope:** fresh read-only reviewers over M4's code by area (R1 the siege's drawing and
  timeline, R2 the mouse, the interface and the adapters, R3 the harness's siege additions), one
  at a time; each finding checked against the code and fixed with its test, or queued for the
  review list.
- **Acceptance:** the findings fixed or listed; `drawn` as before or its changes listed for T3b.
- **Depends on:** T2c-2 (done). **Status:** in progress (2026-10-26).

### E17 M4 T3b — Every capture afresh

- **Scope:** the harness reads the hovered item (`hovered_item` 0x6AAC68 and its id) per frame
  (the user, 2026-10-26: read-only, so that the recaptures settle the status line kept at a
  hot-seat human turn, the inbox's entry); then every situation (`set`) and every sequence
  (`frames`) under Wine, in batches (M3 T9a's: two sessions); `drawn --record`.
- **Acceptance:** M4's exit (the epic file): each capture as recorded or its difference a listed
  fix or a classified quirk; the status line's rule settled or a `CLT-Q<n>`.
- **Depends on:** T3a. **Status:** ready after T3a.

### E17 M4 T3c — M4's review list, M5 refined

- **Scope:** the review list with the user (the epic's "Review lists", T3a's queued findings),
  CLT-Q22 and Q23, M5 (live play) refined with the user.
- **Acceptance:** the review list accepted; M5's tasks in this queue; M4 done.
- **Depends on:** T3b. **Status:** ready after T3b; the user present.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-26 (T2c-2): **the status line at a human turn with the mouse parked** is kept in hot
  seat and emptied with one human side, built as found (*provisional*; design §10.2, "Recorded
  (M4 T2c-2)"). The dispatch is read (an item writes its hint only on entering, `hovered_item`
  0x6AAC68); open: the item under the park point and what makes it entered again (`aim`'s line
  emptied at 8 175, `hit`'s kept). A capture reading 0x6AAC68 and that item's id per frame
  would settle it (T3's captures or live play).

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
  to T2c-2 took the shots, the gate, the moat, the mines, Earthquake and the mouse (design
  §10.2).

- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
