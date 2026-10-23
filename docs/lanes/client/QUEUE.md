# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

T9b (M3's review) split with the user (2026-10-23): T9b-1 T9a's two points; the fresh
reviewer in three parts by area, each a narrow read-only sub-agent whose findings the session
checks against the code (and the executable where one names the original: `scripts/exe.py`)
and fixes or lists; the review list with the user last. A reviewer's brief: its files, M3's
tasks that built them (the epic's log), the design sections they cite; findings as
`file:line`, the failure, a fix; no style notes. A fix that changes a drawn frame needs
`drawn` and the captures it lists (runbook).

### E17 M3 T9b-1 — T9a's two points

- **Scope:** `raise`'s kept `before` at 1 312 ms; the dump at a game's start.
- **Depends on:** T9a (done 2026-10-23). **Status:** in progress (2026-10-23, `client/e17-m3-t9b1`).

### E17 M3 T9b-2 — The reviewer: the timeline

- **Scope:** a fresh reviewer over `h3-view`'s `anim` (and `anim/`), `ray`, `hero`, `scene`
  and `compose` (about 8 000 lines; M3 T1a–T7d); its findings fixed or listed.
- **Acceptance:** each finding fixed (with a test), listed for the review, or shown not a bug;
  `drawn` the same or the changed captures recorded; the client check passes.
- **Depends on:** T9b-1. **Status:** ready once T9b-1 merges.

### E17 M3 T9b-3 — The reviewer: the interface and the adapters

- **Scope:** a fresh reviewer over `h3-view`'s `mouse` (and `mouse/track`), `panel`, `ui`,
  `battle_log`, `text` (and `text/`), `cursor`, `layout`, `tables`; `h3-render`'s `text` and
  `tint`; `h3-assets`' `fnt`; the binary's `sequences`, `situation`, `scenario`, `pointer`,
  `log`, `motion`, `render`, `harvest`, `window`, `audio` (M3 T1a–T8). The library first: when
  one brief is too wide, the binary's adapters go to a second agent.
- **Acceptance:** as T9b-2.
- **Depends on:** T9b-2. **Status:** waiting.

### E17 M3 T9b-4 — The reviewer: the harness

- **Scope:** a fresh reviewer over `clients/capture/` (`h3drive.py`, `h3mem.py`, `h3cap.py`,
  `h3lod.py`, `xdd.c`, `setup.sh`; the client lane's parts of `h3adv.py`), against the runbook:
  a read that can race the game, a capture that can pass while wrong, a write into the running
  copy beyond the runbook's. A fix to how the original is driven, captured or read means every
  capture again (runbook, "Which captures a change needs" 4).
- **Acceptance:** as T9b-2.
- **Depends on:** T9b-3. **Status:** waiting.

### E17 M3 T9b-5 — The review list with the user

- **Scope:** the epic's M3 review list (T1a–T7d and the reviewers' additions) and the open
  CLT-Q9 to Q17 the recordings and readings raised, with the user; M3's exit stated (the reference set
  passes, each difference a listed fix or a classified quirk).
- **Acceptance:** the review list accepted or carried on; the answers built or queued.
- **Depends on:** T9b-4. **Status:** waiting.

### E17 M3 T9c — M3's closure: the next milestone

- **Scope:** the next milestone refined with the user (sieges, live play, HD or the adventure
  map: the user's choice), its tasks queued, M3 marked done.
- **Acceptance:** the next milestone's tasks in this queue, agreed by the user.
- **Depends on:** T9b-5 (T1a–T7d done). **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-22 (the adventure lane, E08 T37b, `adventure/e08-t37b`, `cross-lane` with the user's
  consent; for information, nothing to do): batch E5's recipes `clients/capture/batches/E08-batch-5a.toml`
  to `-5e.toml`, each run clean under Wine at its first run; `h3adv`'s `level_up` may name a skill
  (`side_of`: taken on whichever side it stands) and `path_tiles[:<row>]` reads the path's marked
  tiles (the walk's observation keeps them as `tiles`). The runbook says how.

- 2026-10-22 (the adventure lane, E08 T37a, `adventure/e08-t37a`, `cross-lane` with the user's
  consent; for information, nothing to do): what the two entries below announce is built in
  `clients/capture/`.
  `h3mem.Mem.explored` (a tile's explored bit, the vision words at 0x6989F8); `h3adv`'s
  `explored:x,y[,level][:colour]`, the hero details `points` and `day_points`, the step `equip`
  (the hero screen opened by a click on the hero, backpack to slot, closed with `Return`; probes
  `hero_screen` in `screens.toml`) and a wider table's cells by `<row> | <column>` (`fill`,
  `records`); the harness check's Fog and Equip (Blue's heroes). The runbook says how.

- 2026-10-22 (the adventure lane, E08 T36b, `adventure/e08-t36b`, `cross-lane` with the user's
  consent; for information): `h3_map::writer::Map` gained `banned_skills`; `h3-view`'s situation
  map says none (one line, as for T32a's level limit). Batch E5e also needs an artifact moved from
  the backpack to a slot in the hero screen (E5-3b, E5-10b); the adventure lane builds it with the
  reads below in T37, or skips those two rows.

- 2026-10-22 (the adventure lane, E08 T34b; for your information, nothing to do): batch E5 (the
  hero's movement and scouting; `docs/epics/E08-design.md` §11) needs the harness to read a tile's
  explored bit for a player (`explored:x,y`) and a hero's points at a turn start without a walk;
  the adventure lane builds them in `clients/capture/` as `cross-lane` (T37, as T32b1 did) once the
  user accepts the cut (ADV-Q21), with the recipes.

- 2026-10-21 (the adventure lane, E08 T32b2, `adventure/e08-t32b2`, `cross-lane` as T32b1; for your
  information, nothing to do): **four more harness changes** while batch E4 ran (runbook updated):
  a step's `note` (read as `record`, under "anything odd" unless `none`) and the hero detail
  `equipment` (the worn artifacts but the war machines and the spell book; every hero wears a
  Catapult); `h3lod.identify` takes the most specific record (the most literal text) of any run
  of the dialog's text items: it took the first match of the whole text, and `%s the %s`
  (GENRLTXT 16) formats any text with " the " (a picture's caption is a text item of its own:
  the Necromancy dialog, GENRLTXT 146, never matched); `strike` tries its hover pass four times,
  a second apart (a good morale's extra turn showed only the plain pointer, frame 6). The
  capture set does not use them: no capture was taken. The harness check passed after them.

- 2026-10-21 (the adventure lane, E08 T32b1, `adventure/e08-t32b1`, `cross-lane` with the
  user's consent, decision 68; **your harness changed**, nothing to do but read): the adventure
  lane built batch E4's reads itself, so its two requests (2026-10-20, -21) are done and gone from
  this inbox, and so is your T7a-2a note (`h3adv` now knows the battle screen in its
  `battle_book` state too; the check's Gear hero carries a book). In `capture/`: `h3mem` reads a
  hero's seed byte and offer counters, its skills in order, its artifacts worn and in the backpack
  (names from the artifact table), the dialog's items and the level-up dialog (`level_up_dialog`
  0x4F8880's DEF pictures 2009 to 2011, read for them), the battlefield, the hex records (stack,
  flags, attack mark) and a stack's speed; `h3adv` records every level-up dialog (one or no offer
  too), takes `level_up` as a list, notes other dialogs' records (`messages`), has the
  observations of the runbook's table and plays battles by `attack` or a plan (`{ attack =
  "Ballista", by = "defender", then = "attack" }`): **auto combat does not play a battle between
  two human players in hot seat** (the runbook's traps), so the plans click like a player, the
  mouse resting until the combat cursor shows a melee attack. Fixed on the way: a local `value`
  in `value()` (your T6c-2 winner fix) made every `before:` read fail. `checks/harness-check.toml`
  gained Exp's and Lvl's new reads and the encounter Gear; the check passes (about four minutes).
  The recipes for batch E4 come with E08 T32b2 (`cross-lane` again).
- 2026-10-21 (the adventure lane, E08 T30, decision 65, ADR-0017 point 6 amended in #353; for
  the hot-seat client, when it plays whole games): **a level-up holds the game.** While a hero's
  level-up with two offers is open, the engine accepts no command but its owner's answer
  (`h3_game::GameState::waiting_for` names the owner; `level_up(owner)`, or the owner's
  `GameObservation.decision` as `DecisionView::LevelUp`, gives the hero, level, primary skill and
  the two offers; `Answer::First`/`Second` answer it). A battle's defender that won answers it
  outside its turn. The user's requirement for hot seat: when the player who must act changes,
  show a handover screen that hides everything (map, hero list, the dialog) until that player
  confirms it is in control, then the level-up dialog alone, and hand back to the player whose
  turn runs the same way, so no one sees another player's view.

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
  dissolve that shows it (`create_battle_stack`, 75 ms steps, the blend not read); with them the
  message when no hex is free, a clone's `Vanished`, Teleport's dissolve (moved at once now) and
  Sacrifice's victim leaving no corpse. The battle waits for the heal's sound (T3b) and the other
  effects' (T7b-2b, T7c-3). NOTES, "The battle screen".
- 2026-10-20 (T6a): the tactics phase's panel (0x46C050, not read) and Auto Combat switched on
  (its button selected, the panel dimmed while stacks play themselves) wait for a record with a
  tactics phase and for live play. Design §9.7.
- When CLT-Q5 is answered: the deck and the two boats as situations; CLT-Q4: the background
  variants 0 and 1. NOTES, "Battlefields and boats".
- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks, the Catapult, the gate
  opening before the step. NOTES, "Sieges".
- Later milestones: graphics options (the user, at M1), an HD layout's origin and viewport, the
  adventure map's object draw order (research item 9). NOTES, "Options, layouts, the adventure
  map".
- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".
