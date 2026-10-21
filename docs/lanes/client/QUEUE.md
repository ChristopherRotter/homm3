# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M3 T7a-2a — The harness casts (in progress, `client/e17-m3-t7a-2a`)

- **Scope:** T7a-2 split with the user (2026-10-21: the harness first). The scenarios' heroes
  get a spell book, spells and mana (`h3-view situation`: `h3-map`'s writer's `spells` and the
  book in slot 17); the book's dialog read (0x59BDF0: its spells' places, pages, sort, school
  tabs, how a click returns the spell); a `cast:` action in `h3drive.py` and the viewer's
  `pointer::inputs`; a creature caster's cast (a Faerie Dragon's: the cursor over its target).
  Two proofs recorded: a hero's timed spell on one stack (player 1 of §7.16, the hero's group 4),
  a Faerie Dragon's cast at targets above, level and below (§7.16's caster's angle). Their
  timelines kept beside `sequences.toml`, outside the timeline check (T7b, T7c add them).
- **Acceptance:** the harness casts from a hero's book and a Faerie Dragon's cursor; the two
  proofs recorded; the angle candidate confirmed or the reading corrected.
- **Depends on:** T7a-1 (done). **Status:** in progress.

### E17 M3 T7a-2b — The magic animations recorded

- **Scope:** with T7a-2a's casts, a sequence of each remaining player of §7.16 (one stack: a
  resisted spell; marked stacks: Magic Arrow or Lightning Bolt; several: a mass spell, Death
  Ripple; a raise; a hex: an area spell, a spell obstacle; the field: Armageddon; a projectile)
  and the hero's cast at targets above, level and below (its cast point). `h3mem` sees only the
  first four players' frames: the rest by the frame check's screens. Check §7.16's remaining
  candidates (Chain Lightning's and Fire Shield's undrawn animations, Animate Dead's, a mass
  animation's last frame, the Liches' plain cloud) and correct the design where a recording
  disagrees. The timelines kept as T7a-2a's.
- **Acceptance:** each player of §7.16 has a recorded sequence; every candidate confirmed or the
  reading corrected.
- **Depends on:** T7a-2a. **Status:** ready.

### E17 M3 T7b — Spells on stacks, morale and luck

- **Scope:** built from T7a: the spells cast on stacks (blessings, curses, damage on one stack,
  `SpellCast`, `StackCast`, `EffectApplied`, `SpellResisted`), Haste and Slow's walk time (from T1b
  at its re-cut: the harness casts from T7a's spell book), good and bad morale (`GoodMorale`,
  `BadMorale`: the user at T5b, the Marksmen's second shot showed nothing) and luck (the strike's
  `lucky`), their sounds and their log lines' times (T6b writes morale's at the event,
  *provisional*; design §9.8), a magic animation lengthening a strike's loop; the count box's tint by
  the stack's spells (moved from T5, the user, 2026-10-20: design §4.3, 0x43E6AA to 0x43E73E,
  the spell table's first field to harvest).
- **Acceptance:** their recorded sequences as recorded and their frames match.
- **Depends on:** T7a-2b. **Status:** ready.

### E17 M3 T7c — Area spells, rays and spell obstacles

- **Scope:** built from T7a: the area spells (Fireball, Inferno, Meteor Shower, Frost Ring,
  Armageddon, Death Ripple and the like), the lightning spells' rays (`draw_ray` read and built at
  T1b-1, design §7.12; not yet its branches, randomness and colours 300, 301, 303, 305, nor
  Chain Lightning's call), the spell obstacles in the
  scene (fire walls, force fields, quicksand, land mines: `ObstaclePlaced`, `ObstacleRevealed`,
  `ObstacleTriggered`; `scene::battle` leaves them out so far), Teleport, created and raised
  stacks (`StackCreated`, `SpellRaised`, `Reborn`: drawn once the snapshot lacks them), their
  sounds. May split when refined.
- **Acceptance:** as T7b.
- **Depends on:** T7a-2b. **Status:** ready.

### E17 M3 T8 — The sound's remainder

- **Scope:** the battle's end (`battle_end` 0x477470 calls `play_music` twice: the victory and
  defeat tunes, with the results), the start sound's volume (its loader's default, not read);
  the volume settings (0x5996C0) and a track resuming where it last stopped (0x59AA82) wait for
  the client's options and a second battle (later).
- **Acceptance:** the end's tunes play at the times the executable plays them.
- **Depends on:** nothing. **Status:** ready.

### E17 M3 T9 — M3's closure

- **Scope:** every sequence and situation captured afresh (the runbook, "Which captures a
  change needs": `frames` and `set` in full, then `drawn --record`), M3's review list with the
  user, a fresh reviewer agent over M3, the next milestone
  refined (sieges, live play, HD or the adventure map: the user's choice), M3 marked done.
- **Acceptance:** M3's exit holds; the review list answered or carried on.
- **Depends on:** T7a-2a–T8 (T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1, T4b-2b-2,
  T5a, T5b-1, T5b-2a, T5b-2b, T6a, T6b, T6c-1, T6c-2, T7a-1 done).
  **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

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
