# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14; ADR-0020, ADR-0024). M3 (fidelity) below, refined at M2's closure (T5c) and agreed by the user there, each task about half a context; the exit: the reference set passes, or each difference is a listed fix or a classified quirk. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the `faithful`/`fixed` profile) with the first one a task finds.

### E17 M3 T6c-1 — The mouse's still: the field and the panel

- **Scope:** T6c split with the user on 2026-10-20 (the still T6c-1; over time T6c-2). The
  cursor rules of the executable (`battle_cursor` 0x476490's presentation: which frame of
  which cursor DEF over which hex; the attack's direction by the slot, 0x474A00; how the game
  draws its cursor, its hot spots; T5a's `CRDEFLT.DEF` frame 0 is provisional), the status line's
  hints over the field (`battle_status_line` 0x492840: the cases the engine's battles reach) and
  over the panel's controls (H3API's `SetHints`), the stack under the mouse in a still (its
  outline 0x70 and group 1's frame 0, read at T1b-2: `NOTES.md`); a still and the window draw
  them from a mouse point; the harness parks the mouse over the field and the panel as
  situations.
- **Acceptance:** the mouse situations' frames match the original's outside the listed
  differences.
- **Depends on:** nothing (T6a, T6b done). **Status:** ready.

### E17 M3 T6c-2 — The mouse over time

- **Scope:** the harness's mouse moves in the viewer's timeline (the cursor and the hint after
  each move and at a turn's start, 0x4ECC00; the hover's hint forces the status line over new
  lines, a turn's posted mouse move does not: T6b, design §9.8, `NOTES.md`; group 1 as a fidget
  after a stack is marked), so the sequences' ` log` listings of hints go and `hover` is
  compared; in the window a button pressed (its click frame) and the panel's hotkeys (design
  §9.7).
- **Acceptance:** the sequences' hint listings gone, `hover`'s frames matched outside the listed
  differences.
- **Depends on:** T6c-1. **Status:** waiting.

### E17 M3 T7a — Code reading: the magic animations

- **Scope:** the user (T5c): the plethora of spell animations. How the executable draws a spell
  or a magic animation (H3API's PlayMagicAnimation and its table: read over one stack at T3b,
  design §7.15, the table harvested into `[battle] magic`; left: the hit it plays with
  `show_hit`, the places 3 and above, mass animations over several stacks (0x5A66D0), which spell
  plays which; each animation's DEF, where it
  is drawn: over a stack, a hex, the field, from caster to target; its timing and its place in
  `strike_animation`'s L = max(the magic animation's frames, A + R − 1), §7.4; the spells'
  sounds by the spell table's names); the hero's and a creature caster's cast animation; morale
  and luck through the same table. Recorded sequences of a few spells of each kind in the
  original (the harness casts from a hero's spell book: to build).
- **Acceptance:** design §7 gains the reading with every address in the map; each kind has at
  least one recorded sequence.
- **Depends on:** nothing (T1a's battle in progress). **Status:** ready.

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
- **Depends on:** T7a. **Status:** ready.

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
- **Depends on:** T7a. **Status:** ready.

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
- **Depends on:** T6c-1, T6c-2, T7a–T8 (T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1,
  T4b-2b-2, T5a, T5b-1, T5b-2a, T5b-2b, T6a, T6b done).
  **Status:** waiting.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-10-20 (the adventure lane, E08 T24a2b2, `adventure/e08-t24a2b2`; a request for increment 4's batch E4, survey `docs/rules/surveys/E08-progression.md` §7): **level-ups on the harness.** Batch E4 predicts each level's primary skill and offers from a hero's seed byte, so it needs: (1) the hero details `seed` (hero +0x8F), `wisdom_offer` (+0x90) and `school_offer` (+0x3F), one byte each (`docs/exe/structs.toml`); (2) a record of every level-up dialog passed (the hero, the new level, the primary skill gained, the offered skills left and right, the one taken); where the dialog keeps its offers was not read on our side (the dialog 0x4F8880 is called from `hero_level_up` 0x4DA720 after both offers are drawn); (3) `level_up` as a list, one answer per dialog, for a step whose battle gives several levels (row E4-4); (4) the hero's artifacts, worn and in the backpack (row E4-9: a destroyed Ballista's artifact); (5) the battle's field, read from memory as the stacks are (row E4-7: two special grounds on one tile). Rows and the predictions: the survey's §7; the spec and recipe follow with E08 T24b's batch task, after the user's checkpoint (ADV-Q15), so nothing is urgent.
- 2026-10-20 (the adventure lane, E08 T22a2b): `h3adv.value` prints a winner of −1 (none) as
  `defender` (`SIDES[-1]`): fix with the harness's next change. NOTES, "The adventure lane's
  harness" (also, for information: its three recipes, its auto combat's change).
- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".
- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".
- For T6c: the mouse over a stack (T1b-2), the shot cursor's half damage without the shooter's
  turn (0x4767E6); for T7c: the message when no hex is free for a created stack; for T8: the
  battle waits for the heal's sound (T3b). NOTES, "The battle screen".
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
