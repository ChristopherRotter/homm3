# Client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-10-27 (M4 T3c: taken by R2-7 into M5 T1, R2-8 with M4 T3a's R1-3 into T6, R3-1 into T5; R4-2 and R4-9 stay in the inbox; the entry verbatim):

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R2-7: a timeline that steps forward (`Moment::at`, `Heroes::posed` and `Timeline::at`
  rescan the battle for every frame: 2.3 ms a frame over 82 seconds, `play`; for live play's long
  games). R2-8: a record that opens mid-battle (the snapshot's dead stacks as corpses, a raise of
  one drawn; the box by the timeline's living set, not the first snapshot's `alive`; with it
  M4 T3a's R1-3: the moat's strip over a stack by the fixed moat rows, 0x63BCE8 and 0x63BCF4,
  not the record's moat, which leaves out the bridges while the opening's gate is open). R4-2:
  `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s boxes, the log's
  text: digests recorded again from `master`'s viewer first, the runbook's rule). R4-9: a
  stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an artifact over almost
  anything; the counts stay exact). R3-1 goes with the created stacks below.

- 2026-10-27 (M4 T3c: taken by M5 T3; the entry verbatim):

- 2026-10-21 (T7b-1): **a hero's aim.** Between the book's place and the target's click the
  cursor is the spell's (`crspell.DEF`, animated: not read; the viewer leaves it as it was) and the
  status line says "Cast %s on %s" (GENRLTXT 28, built *provisional*, not forced; 27 elsewhere);
  `enter_player_cast` 0x59EC50's loop is not read (which targets it offers, the cursor's frames):
  with the mouse's model when a recording aims a hero's area spell at a hex (T7c-1's are the
  Faerie Dragon's clicks), and for live play. And `cast`'s 71
  pixels at 0: the original redraws the battlefield over a rock's top (obstacle 22) for the one
  frame of the click (0x479038's path, not read).

- 2026-10-27 (M4 T3c: taken by M5 T4 (the volumes to `NOTES.md`, "Options and layouts"; `remote_turn` to T6); the entry verbatim):

- 2026-10-21 (T8): the battle's result window (`battle_result_dialog` 0x4702D0, not read beyond
  its tune: the heroes, the texts, the casualties, OK) and the whole screen darkened under it
  (0x44E5F0 from 0x47782C): `end`'s last frame differs by them (its counts). The volume settings
  (0x5996C0) and a track resuming where it stopped (0x59AA82) with the client's options and a
  second battle (later); the network's `remote_turn` in the end's choice with live play.

- 2026-10-27 (M4 T3c: taken by M5 T5; the entry verbatim):

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

- 2026-10-27 (M4 T3c: taken by M5 T4; the entry verbatim):

- 2026-10-20 (T6a): the tactics phase's panel (0x46C050, not read) and Auto Combat switched on
  (its button selected, the panel dimmed while stacks play themselves) wait for a record with a
  tactics phase and for live play. Design §9.7.

- 2026-10-27 (M4 T3c: taken by M4, done 2026-10-27; the entry verbatim):

- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks (drawn as the town's
  shooters since T1b), the Catapult, the gate opening before the step. NOTES, "Sieges"; M4 T2b
  to T2c-2 took the shots, the gate, the moat, the mines, Earthquake and the mouse (design
  §10.2).

- 2026-10-27 (M4 T3c: taken by M5 T6 and T7; the ladders, the pause and the server browser after M5; the entry verbatim):

- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".

- 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
  reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
  changed for every recipe: `before:<observation>` is now read right before the step (any
  observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
  pictures is answered by `choice` (it stopped the batch before), and each message closed with its
  OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
  `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
  stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
  `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

- 2026-10-26 (T2c-2): **the status line at a human turn with the mouse parked** is kept in hot
  seat and emptied with one human side, built as found (*provisional*; design §10.2, "Recorded
  (M4 T2c-2)"). The dispatch is read (an item writes its hint only on entering, `hovered_item`
  0x6AAC68); open: the item under the park point and what makes it entered again (`aim`'s line
  emptied at 8 175, `hit`'s kept). A capture reading 0x6AAC68 and that item's id per frame
  would settle it (T3's captures or live play). Done at M4 T3b-1 (2026-10-27,
  design §10.2 "Recorded afresh").

- 2026-10-23 (the map-client lane, E24 T0; for later, nothing to do now): **what the map client
  will need of `h3-view`.** E24 M1 (stills of the map view) builds only on the common foundation;
  M2's window will want `clock` and `sound`, M3's panels `ui`, `text` and `layout`, in a common
  crate. When M2 or M3 is refined, the map-client lane proposes the move here first (a new common
  crate is a `lanes.toml` change, shared); your M4 waits for none of it.

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

- 2026-10-21 to 22 (the adventure lane, E08 T32b1 to T37b; for information, nothing to do): the
  harness's reads and steps for batches E4 and E5 (moved to NOTES, "The adventure lane's
  harness", at T9c; the harness is common since the split, 2026-10-23).

- 2026-10-20 (the adventure lane, E08 T22a2b): its three recipes, its auto combat's change, for
  information (NOTES, "The adventure lane's harness"; the winner of −1 fixed in E17 M3 T6c-2).

