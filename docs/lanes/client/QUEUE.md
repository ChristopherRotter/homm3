# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M3 T7c — Area spells, rays and spell obstacles

- **Scope:** built from T7a: the area spells (Fireball, Inferno, Meteor Shower, Frost Ring,
  Armageddon, Death Ripple and the like), the lightning spells' rays (`draw_ray` read and built at
  T1b-1, design §7.12; not yet its branches, randomness and colours 300, 301, 303, 305, nor
  Chain Lightning's call), the spell obstacles in the
  scene (fire walls, force fields, quicksand, land mines: `ObstaclePlaced`, `ObstacleRevealed`,
  `ObstacleTriggered`; `scene::battle` leaves them out so far), Teleport, created and raised
  stacks (`StackCreated`, `SpellRaised`, `Reborn`: drawn once the snapshot lacks them), their
  sounds. T7a-2b's recordings (`casts.toml`, design §7.16 "Recorded") are the references: `bolt`
  (its Chain Lightning; its Lightning Bolt is played since T7b-2b-3, `lightning`), `mass`, `raise`, `obstacles` (its Fire Wall's second hex placed at the first's: confirm by the
  frame check), `creatures` (the Liches' cloud), `dragon` (at a hex). May split when refined.
- **Acceptance:** their recorded sequences as recorded and their frames match (new recordings,
  T7b-2b-3's way: `sequences.toml` with `fail` and an engine `seed` searched; `bolt`'s round 1
  is `lightning` since T7b-2b-3).
- **Depends on:** T7a-2b (done). **Status:** ready.

### E17 M3 T9 — M3's closure

- **Scope:** every sequence and situation captured afresh (the runbook, "Which captures a
  change needs": `frames` and `set` in full, then `drawn --record`), M3's review list with the
  user, a fresh reviewer agent over M3, the next milestone
  refined (sieges, live play, HD or the adventure map: the user's choice), M3 marked done.
- **Acceptance:** M3's exit holds; the review list answered or carried on.
- **Depends on:** T7c (T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1, T4b-2b-2, T5a,
  T5b-1, T5b-2a, T5b-2b, T6a, T6b, T6c-1, T6c-2, T7a-1, T7a-2a, T7a-2b, T8, T7b-1, T7b-2a,
  T7b-2b-1, T7b-2b-2a, T7b-2b-2b, T7b-2b-3 done).
  **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

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
  with the mouse's model when T7c's area spells aim at hexes, and for live play. And `cast`'s 71
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
- For T7c: the message when no hex is free for a created stack; for T8: the battle waits for the
  heal's sound (T3b). NOTES, "The battle screen".
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
