# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M3 T7b-2a — The count box's tint, Haste and Slow's walk, Dispel, Cure, Disrupting Ray

- **Scope:** T7b-2 split with the user (2026-10-21: the parts without the original's rolls; T7b-1
  built a hero's cast on one stack: `anim::Player::cast`, design §7.16 "Built"): the count box's
  tint by the stack's spells (moved from T5, the user, 2026-10-20: design §4.3, 0x43E6AA to
  0x43E73E, the spell table's first field to harvest; `cast`'s and `resist`'s frames list it now:
  238 and 197 to 228 pixels, which it brings to 0); Haste and Slow's walk time (from T1b at its
  re-cut: W trunc(W0 x 0.65), trunc(W0 x 1.5), design §7.3); Dispel (41 before its effect), Cure
  (39 after) and Disrupting Ray (its projectile `c07spA0`, then 14) on one stack, their sounds
  and log lines. Two new recordings: `haste` (Haste, a hasted walk, Slow ending it, a slowed walk)
  and `dispel` (Slow, Disrupting Ray, Cure, Slow again, Dispel), both hot seat.
- **Acceptance:** `cast`'s and `resist`'s boxes exact; `haste` and `dispel` recorded, then played
  as recorded and their frames matched (in `sequences.toml` with their commands, frames and
  boxes).
- **Depends on:** T7b-1 (done). **Status:** in progress (2026-10-21, `client/e17-m3-t7b-2a`).

### E17 M3 T7b-2b — Morale, luck and the creatures' rolled animations

- **Scope:** the rest of T7b-2 (split 2026-10-21; each needs a roll of the original's, which the
  harness cannot force: a scenario that gives many chances, or runs repeated until the roll
  falls): good and bad morale (`GoodMorale`, `BadMorale`: the user at T5b, the Marksmen's second
  shot showed nothing) and luck (the strike's `lucky`), their sounds and their log lines' times
  (T6b writes morale's at the event, *provisional*; design §9.8); Poison's hit, and a magic
  animation lengthening a creature's strike (Death Stare, Acid Breath, Life Drain, the
  Thunderbird's bolt; player 1 and 2 of §7.16); Lightning Bolt's bolt from the sky with T7c's
  `bolt`.
- **Acceptance:** their recorded sequences as recorded and their frames match (new recordings
  where none exists, `casts.toml`'s way).
- **Depends on:** T7b-2a. **Status:** ready once T7b-2a is done.

### E17 M3 T7c — Area spells, rays and spell obstacles

- **Scope:** built from T7a: the area spells (Fireball, Inferno, Meteor Shower, Frost Ring,
  Armageddon, Death Ripple and the like), the lightning spells' rays (`draw_ray` read and built at
  T1b-1, design §7.12; not yet its branches, randomness and colours 300, 301, 303, 305, nor
  Chain Lightning's call), the spell obstacles in the
  scene (fire walls, force fields, quicksand, land mines: `ObstaclePlaced`, `ObstacleRevealed`,
  `ObstacleTriggered`; `scene::battle` leaves them out so far), Teleport, created and raised
  stacks (`StackCreated`, `SpellRaised`, `Reborn`: drawn once the snapshot lacks them), their
  sounds. T7a-2b's recordings (`casts.toml`, design §7.16 "Recorded") are the references: `bolt`,
  `mass`, `raise`, `obstacles` (its Fire Wall's second hex placed at the first's: confirm by the
  frame check), `creatures` (the Liches' cloud), `dragon` (at a hex). May split when refined.
- **Acceptance:** as T7b-2b.
- **Depends on:** T7a-2b (done). **Status:** ready.

### E17 M3 T9 — M3's closure

- **Scope:** every sequence and situation captured afresh (the runbook, "Which captures a
  change needs": `frames` and `set` in full, then `drawn --record`), M3's review list with the
  user, a fresh reviewer agent over M3, the next milestone
  refined (sieges, live play, HD or the adventure map: the user's choice), M3 marked done.
- **Acceptance:** M3's exit holds; the review list answered or carried on.
- **Depends on:** T7b-2b, T7c (T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1, T4b-2b-2,
  T5a, T5b-1, T5b-2a, T5b-2b, T6a, T6b, T6c-1, T6c-2, T7a-1, T7a-2a, T7a-2b, T8, T7b-1 done).
  **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-21 (the adventure lane, E08 T32a, `adventure/e08-t32a`, decision 67; your inbox's
  request of 2026-10-20 below, now with the specs): **batch E4's three recipes wait for these.**
  The specs and templates are committed: `experiments/specs/E08-batch-4a.toml` (sixteen heroes:
  levels at load and in play), `-4b.toml` (the map's level limit) and `-4c.toml` (grounds,
  Necromancy, a Ballista, a Cartographer); the recipes (`clients/capture/batches/E08-batch-4{a,b,c}.toml`)
  are yours or E08 T32b's, as you prefer. Each template's fields say what is read; beyond the
  2026-10-20 list (the seed byte +0x8F and the levels last offered +0x90, +0x3F; every level-up
  dialog's level, primary skill, left and right offers and the one taken, the template's
  `dialog N, …` fields; `level_up` as a list, E4-4's `right, left, right`; the hero's artifacts
  worn by slot and in the backpack; the battlefield): (6) a hero's **secondary skills as a
  list** (`Leadership expert, Wisdom basic`, in the hero's order: the skills learned at load are
  not known in advance), for the hero stepping and for another hero (`hero[E4-1a]:…`, E4-1's
  heroes never move); (7) **E4-9's battle played by plan** in hot seat (Red's Archangels defend
  until Blue's Thunderbirds have destroyed Red's Ballista, then attack; as the battle lane's
  `experiments/agent/siege.py` plans), and whether the Ballista was destroyed; (8) the messages
  after a battle (E4-6: "can gain no more"), as text or yes/no. The scorer that reads the filled
  templates is `crates/h3-map/examples/e4_score.rs` (it parses `- label: value` lines: numbers,
  `a/d/p/k`, skill names with `basic`/`advanced`/`expert`, a taken offer by its name or as
  `left`/`right`).

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

- 2026-10-21 (T7a-2a): `h3adv.py` knows a battle by the `battle` probes alone, which lie on the
  panel's spell-book button: a battle whose acting human hero has a book shows `battle_book`
  (`screens.toml`) and would stall a batch. Accept both when the adventure harness next changes
  (then its check, `capture/checks/harness-check.toml`).

- 2026-10-20 (the adventure lane, E08 T24a2b2, `adventure/e08-t24a2b2`; a request for increment 4's batch E4, survey `docs/rules/surveys/E08-progression.md` §7): **level-ups on the harness.** Batch E4 predicts each level's primary skill and offers from a hero's seed byte, so it needs: (1) the hero details `seed` (hero +0x8F), `wisdom_offer` (+0x90) and `school_offer` (+0x3F), one byte each (`docs/exe/structs.toml`); (2) a record of every level-up dialog passed (the hero, the new level, the primary skill gained, the offered skills left and right, the one taken); where the dialog keeps its offers was not read on our side (the dialog 0x4F8880 is called from `hero_level_up` 0x4DA720 after both offers are drawn); (3) `level_up` as a list, one answer per dialog, for a step whose battle gives several levels (row E4-4); (4) the hero's artifacts, worn and in the backpack (row E4-9: a destroyed Ballista's artifact); (5) the battle's field, read from memory as the stacks are (row E4-7: two special grounds on one tile). Rows and the predictions: the survey's §7; the spec and recipe follow with E08 T24b's batch task, after the user's checkpoint (ADV-Q15), so nothing is urgent.
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
