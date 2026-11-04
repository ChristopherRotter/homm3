# Open questions

Questions waiting for the user. Agents add new ones here (context, options, recommendation, what is blocked). When answered, record the decision (ADR or doc) and move the entry to the top of its lane's `QUESTIONS-ANSWERED.md` (table at the end of this file).

Since Phase 0 (2026-10-07) every lane adds its questions here with a lane id that counts per lane: `BAT-Q1`, `ADV-Q1`, `SRV-Q1`, `CLT-Q1`, `MCL-Q1`, `TWN-Q1`, `AI-Q1` (`lanes.toml`); the numbered ids (`Q55`) stay. A new entry goes under its lane's block at the end of the "Open" section, as `#### <CODE>-Q<n> — title` (blocks keep concurrent lanes from editing the same lines).

## Open

### Q2 — Project name and crate prefix

The repo is `homm3`; crates use the provisional prefix `h3-`. "Heroes of Might and Magic" is a trademark of its owner; a public project name should probably avoid implying endorsement. Decide before first public release; renaming later is cheap but not free.

**2026-10-24 (Q59):** asked before the repository goes public, the user had no preference: it goes public as `homm3` with the prefix `h3-`, and the README says the project is not affiliated. A later rename stays possible (it then also moves links and forks).

### Q4 — Confirm local compute

Is the CPU a Ryzen 9 3900X (12 cores/24 threads) or the non-X 3900? How much RAM? Used to set initial benchmark and self-play expectations; not blocking.

### Q7 — Supported platforms for the core

Proposed: native Windows, Linux, macOS and WASM compile targets for the logic crates; CI initially Linux + Windows. Confirm.

### Q10 — Multiplayer model confirmation

Design assumes **authoritative server with per-viewer projections** (`06-security.md`); lockstep with full state is excluded because it leaks hidden info. Confirm this matches intent for any future multiplayer.

### Q22 — Jousting: path length or start-to-end distance (E03 T5)

Wiki (Cavalier page): the bonus is "calculated based on the distance between starting and ending positions". VCMI passes the length of the path actually walked. They differ when obstacles or stacks force a detour. **Implemented: path length (VCMI)** (BAT-DMG-005). **Experiment:** place a Cavalier so that its shortest path to the target detours around an obstacle (e.g. 5 steps for a straight distance of 4); compare its damage on a fixed target with a 4-step straight charge (+25 % vs +20 % of base).

**Not in batch 1a** (2026-10-03): the experiment needs a controlled detour and, because Champion damage is random, many repetitions. If a task needs the answer, reading the game's code is the cheaper last resort (Q29); otherwise VCMI's path length stays.

### Q43 — Chain Lightning: the sprite offset in the jump distance (E04b T2d)

The original picks Chain Lightning's next target by screen distance; the y position of each stack is lifted by half of `267 − the top offset of its creature's sprite frame` (`docs/formats/installation.md` §7.8). The offset comes from the creature's DEF file (`H3sprite.lod`), a graphics value the packs do not hold. **Implemented (user, 2026-10-06):** hex-centre geometry with the two-hex shift and the original's tie order, without the offset (BAT-MAG-006); batch 2a checks the geometry. **Options later:** harvest the offset per creature (stage `H3sprite.lod`, a minimal DEF reader in `h3-harvest`, a creature field), or keep the approximation. Matters only when two candidates are nearly equally far; decide after batch 2a. **Batch 2a** (R1, R2) confirmed the screen geometry and the tie order with stacks of one creature type, where the offset cancels; it says nothing about mixed types. Still the agent's recommendation: keep the approximation until a task needs the sprite data anyway. **Client lane (E17 M1 T4, 2026-10-13):** the offset is the top offset of frame 0 of group 2 (standing) of the creature's DEF: 0x43D9F0 stores 267 minus it at stack +0x16C (`docs/epics/E17-design.md` §5); the client's asset tables name each creature's DEF (`clients/assets/`), so a harvest of the value needs no new reading.

### Battle lane (`BAT-Q<n>`)

#### BAT-Q28 — The review's readings against VCMI or the wiki (E04b T11b)

The independent review re-read the executable for every E04b rule. On these rows its reading differs from the engine, and VCMI or the wiki contradicts it or was not compared (the review had no wiki access): conflicts of sources (`docs/04-rules-and-sources.md` §2). The engine is unchanged but for row 2. Readings: `docs/formats/installation.md` §7 ("Review re-read"); experiments and the wiki's quotes: `docs/experiments/README.md` ("From the E04b review", "The wiki checks"). Bracketed ids: the review's findings. **Consolidated 2026-10-19:** row 2 (built) and row 13's record (the points where the engine follows the reading and only VCMI differs, with the wiki's verdicts on them) moved to the experiments README ("BAT-Q28's record"); the wiki's verdicts on the rows below are in their third column.

| # | The reading (deciding address) | VCMI; the wiki | The engine today | The reading in play; the experiment |
|---|---|---|---|---|
| 1 (r1 F-1) | Chain Lightning searches the attacker's stacks first for its next target; of two equally near, the earlier wins (0x5A61F0) | a tie-break by position; "randomly chosen" | the defender's first (BAT-MAG-006) | stacks of both sides equally near swap places in the chain; such a tie, one creature type |
| 3 (r2a F-4, r4b F-4) | The Defend bonus is fixed when the stack defends and sits inside its defense: later spells do not move it, and Acid Breath tests and lowers the defense with it (0x47974A, 0x4407F8) | +20 % of the current defense, as the engine; fits both | recomputed at each strike, outside that test (BAT-TRN-004, BAT-MAG-046); a fix needs a state field | 1 or 2 defense points on a defending stack; defense 12, Defend, Expert Stone Skin: 20 (reading) or 21 |
| 5 (r3 F-5) | A two-hex walker with its front hex on a Quicksand pit that is ordered forward along its row is stopped again after one step: the hex it keeps covering is the pit's (0x442266); the other way it is free | the hexes a stack starts on never stop it; not checked | as VCMI (BAT-MAG-032) | a pit holds cavalry two turns, not one; Cavaliers with their front hex on a pit, ordered forward |
| 6 (r4a F-2) | A dead stack's round state is frozen: the round start skips it, a raise clears only "dead" (0x44712D) | every stack is reset, dead or alive; not checked | as VCMI (BAT-MAG-035) | ordinary play: a stack that acted, died and is raised a round later gets no turn in it; that sequence, with a control |
| 11 (r1 F-6, r6 F-2) | Orb of Vulnerability. (a) A chain tests each later candidate as cast by the side opposite to the candidate's, so either hero's Orb counts (0x5A6269). (b) The Unicorns' aura has no Orb test (0x5A8592; a second reading) | (a) not compared, the wiki silent on chains (its Orb page fits the engine on casting); (b) both lift the aura, as the Orb page; batch 2e: 12 of 12 slowed (6.9 % under the reading, no control); the user decided, BAT-Q27 | (a) at the caster's own stacks only the caster's Orb counts; (b) lifted (BAT-MAG-066) | (a) under the other hero's Orb a chain strikes the caster's own Black Dragons; (b) batch 2e's row again, with a control and 30 rolls |
| 12 (r2a F-6) | Dispel below Expert cannot be aimed at a stack whose only effect is Acid Breath: the aim wants an effect with a duration (0x5A81A2) | neither was compared | legal, a useless cast (BAT-MAG-016); a fix needs a spell flag in the pack format | rare (Rust Dragons): one legal cast fewer; the cursor |

**Batch 2g (2026-10-24, played by an agent; `experiments/results/E04b-batch-2g.md`, `-2h.md`):** rows 4, 7, 8, 9 and 10 as the reading, out of the table: row 7 (the surrender price leaves the temporary creatures out), 8 (the wait phase orders every stack alike and offers no Wait) and 9 (a freeze is no activation) are the engine's rules now (BAT-FLE-002, BAT-TRN-002, BAT-TRN-001); rows 4 and 10 are fix-optional quirks (the user: BAT-QUIRK-29, S-23; BAT-QUIRK-30, S-24; on in `faithful`). **User (2026-10-13, #156's checkpoint):** row 2 follows the reading (built: a stack's area spell or chain takes its controller's hero's bonuses, BAT-MAG-048); rows 4 and 10 (S-23, S-24) are noted and investigated later, no quirk yet; row 13's knights keep the code's 25 % and mastery 0 until batch 2g (the wiki says 20 % and Advanced, as VCMI). The other rows stay; the wiki is silent on rows 1, 3, 5 to 9 and 12.

**Options:** (a) today's behaviour on every row until one experiment batch (2g) settles it; (b) the reading now on chosen rows. **Recommendation:** (a): by §2 a reading's side is never implemented alone; rows 4 and 10 would be quirks and wait for a confirmation anyway (§4). **Blocked:** nothing.

#### BAT-Q32 — The Faerie Dragon's F-key path (E04b T11b; r5a F-2, F-4)

The original casts a Faerie Dragon's drawn spell in two ways: the mouse cursor on a stack, and the F key (the log says "press F to cast"; 0x475C17 → 0x59EC50; `installation.md` §7.21). The engine's rule (BAT-MAG-049) comes from the cursor alone. **Open, (1) the review's reading of the F path:** the spell is aimed as a hero's: an area spell (41 of 100 draws) at any hex outside the two side columns, an empty one included; VCMI aims at a location too; the engine allows only hexes of stacks the spell can affect (no Fireball between two stacks). Two corners are unsettled: the path returns at once when the side's hero has cast this round, and for a side without a hero it reads a hero that is not there; under the Orb of Inhibition the key path refuses a Chain Lightning at the caster's side's stacks, which the engine still offers. **The wiki (2026-10-13)** is silent on (1) for SoD: "Area-of-effect spells can be targeted at any hex of the battlefield" stands under the HotA heading with the HotA key G, and HotA 1.5.4 "Faerie dragons now cannot aim Chain Lightning at allied stacks" (SoD could, as the engine allows). **Done:** (2), the path passes no "magic allowed" test (0x41F890): the Orb of Inhibition does not stop the dragons, as the wiki says (the user, 2026-10-13, #156's checkpoint; built: the Faerie Dragon's `ignores_magic_block`, BAT-MAG-061; BAT-Q21 point 1, §7.23).

**The computer's dragon (E23 T4a, `installation.md` §7.43, §7.50):** its cast (0x420F00) values every hex outside the side columns, a free one included, and casts at the best, so the original's computer aims area spells as (1) reads. The policy takes only the engine's targets (the user, 2026-10-21; BAT-STUB-8, BAT-AI-013): with (b) it follows the wider targets at once. **Options:** (a) today's behaviour until one Faerie Dragon session settles (1); (b) widen the area spells' targets now (the reading and VCMI agree). **Recommendation:** (a): the session is short, and the corners decide when the wider targets hold. **The computer's dragon observed (E23 T7, batch 4a C2, 2026-10-23):** neutral dragons cast Fireball and Frost Ring at the free hex between two of Red's stacks four times of four, as 0x420F00 reads; the policy's BAT-STUB-8 (the engine's targets only) is a gap of the computer's side either way; (1) for a human's F key stays unplayed. **Blocked:** nothing.

#### BAT-Q35 — An expired created stack's record and the observation (E04b T11b; r4a F-4, the check of the fixes)

The review's fix C3 follows the reading of 0x47A100: a created stack whose rounds ran out keeps its record, so no later created stack takes its id and it counts towards the side's stack limit (BAT-MAG-037, BAT-MAG-052; the wiki and VCMI are silent). The rule now reads the engine's mark of such a stack (`Created::vanished`), which the observation does not show: a killed clone and an expired one look alike but differ in whether a Pit Lord may raise at the limit and in the next created stack's id. Nothing hidden leaks (both ends are public events) and the legal mask stays exact, but the observation is no longer complete in ADR-0014 point 8's sense. Registered as BAT-STUB-1. **Options:** (a) keep the fix and add a per-stack public mark to the observation at the next observation-encoding bump (the flat encoding and the C ABI change, and the server's wire maps the observation: a `cross-lane` change); (b) take the fix out until then (one commit; the engine again reuses an expired stack's record, unlike the reading). **Recommendation:** (a) with E05's batch API or E04c, whichever bumps the encoding first. **Blocked:** nothing.

#### BAT-Q58 — A hypnotized walker that values itself (E23 T4b2; low priority)

**Context.** The computer's walker (0x421680) values every record of the other side as a candidate (`installation.md` §7.52); a hypnotized walker, controlled by the side that hypnotized it, sits among those records and tests none against itself. One whose strikes meet no retaliation (a Harpy) finds its strike on itself worth taking and plans a move and attack on its own position, which the engine refuses (`NotAnEnemy`); E23 T4b2's property over random battles found it. **Built (the user, 2026-10-21):** the walker leaves itself out and takes its best other candidate (BAT-STUB-10, ADR-0027 point 3). **Open:** what the original does with that move and attack (after the walk, its old position may hold no stack: perhaps a plain move). **Options:** (a) keep the port's choice; (b) once a batch row shows the original's outcome, model it where the engine can (a move to the plan's hex). **Recommendation:** (a) until the AI batches (T7) play a row: a computer's hypnotized Harpy next to its owner's stacks. Not staged in batches 4a to 4c (E23 T7, 2026-10-23): it needs a computer hero that hypnotizes the Harpies first (Tan's hero, batch 4b's set-up, with Hypnotize as its best cast). **Blocked:** nothing.

### Adventure lane (`ADV-Q<n>`)

(no open questions)

### Server lane (`SRV-Q<n>`)

(no open questions)

### AI lane (`AI-Q<n>`)

(no open questions)

### Client lane (`CLT-Q<n>`)

#### CLT-Q5 — Boats in `h3-map`'s writer, for the deck and the two boats (E17 M3 T2; for the adventure lane)

**Context.** The capture set shows the original every battlefield the harness can set up through `h3-map`'s writer (`h3-view situation`, `clients/situations/set.toml`); since E17 M3 T2 the shore and the nine special grounds, all exact (`docs/epics/E17-design.md` §5, §6.2). Two special battlefields need boats on the map (survey `E04c-battlefields.md` §1): a ship's deck (`base:deck`, `CmBkDeck.pcx`: a hero in a boat against one on land, or the guards of a Derelict Ship, class 24, or a Shipwreck, class 85) and two boats (`base:boats`, `CmBkBoat.pcx`: both heroes in boats). The writer has neither a boat (class 8, `AVXboat0`–`2`), a hero placed in one, nor those two locations (its objects: heroes, monsters, piles, grounds, towns). The client draws both from a scenario (`h3-view render`: the deck's water obstacles, the boats' fixed hexes), but cannot compare them with the original. **Options:** (a) the writer gains boats (an empty boat and a hero standing in one on water) when the adventure lane builds boats (GAM-BAT-001: "Boats … come with the increments that build them"); (b) a Derelict Ship or Shipwreck only (the deck alone); (c) leave both unchecked. **Recommendation:** (a), whenever boats come; the client lane then adds the two situations (`set.toml` entries) without further reading. Not urgent: both pictures are whole PCX files, drawn as every other background is. **2026-10-28 (the adventure lane, E08 T65, decision 158):** increment 8 builds boats, so (a): the writer gains an empty boat and a hero standing in one; the adventure lane tells the client lane's inbox when they land. **2026-10-28 (E08 T67):** the empty boat landed (`writer::PLAIN_OBJECTS`, class 8); a hero standing in one cannot be written (the original gives a hero placed on water no boat, survey `E08-sea.md` §2): a hero beside an empty boat boards it in the game (the client lane's inbox has the setups).

#### CLT-Q42 — The hero's info window's morale ignores the Spirit of Oppression, its luck honours the Hourglass (E17 M6 T3b2)

**Context.** The battle's hero info window (the right button over a hero, `docs/epics/E17-design.md` §12.8) shows the hero's own morale and luck by `hero_morale` 0x4E39B0 and `hero_luck` 0x4E36C0, the functions the adventure map's status window uses. `hero_luck` returns 0 when this hero or the other side's wears the Hourglass of the Evil Hour (85), as every stack's luck then is; `hero_morale` is passed the other hero too but never reads it, and tests no Spirit of Oppression (84) on either: a hero with Expert Leadership shows morale +3 in the window while a Spirit of Oppression on either side holds every stack's morale at 0 (the stacks' dialogs show the suppression, §12.4). Read, not recorded (the battle's map writes no worn artifacts). **Options:** (a) copy: the window shows the hero's own morale, as the status window does; (b) fix in the client's profile: a morale cap reaching the side gives the window its value, as luck's (the original's for the comparisons only); (c) an option. **Recommendation:** (a): the window is the hero's, the asymmetry harmless and the status window's too (the map client lane's to match if it changes). **Built:** as the original's (`h3_view::hero_info::values`).

#### CLT-Q43 — The combat options dialog's input slips (E17 M6 T4a)

**Context.** The battle's combat options dialog (`combat_options_proc` 0x46F7B0, `docs/epics/E17-design.md` §12.12) has three slips a player can meet: (1) the speed buttons play the click twice, once as a DEF button's press and once from the procedure, which plays it for any press on items 225 to 238 and the speeds besides; (2) a checkbox or a volume bar acts on the left button's release anywhere on the screen after its press, where a DEF button needs the release inside; (3) after the close the cursor stays the arrow, its hover state the old one, until the mouse moves. Read; (1) is not heard in the captures (the harness has no sound). **Options:** (a) copy; (b) fix in the client's profile (one click; the release inside the item; the battle's mouse posted at the close, as after the stats dialog), the original's for the comparisons only; (c) an option. **Recommendation:** (b) for all three: slips no player expects, fixed as CLT-Q26's and Q27's were. **Built:** as the original's (`h3_view::options`).

#### CLT-Q44 — The combat options dialog's no-sound guard (E17 M6 T4a)

**Context.** Without a digital sound driver the dialog refuses a volume bar's click with GENRLTXT 151 in an OK box, but only while that volume is 0 (0x46FA1F): above 0 every bar works without a sound, at 0 none does; the harness runs the original so (its volumes 0, no sound in Wine: `options-no-sound`). The client tests its own audio device the same way (none with `--mute` or without a device). Besides, the effects bars also write "Last Sound Volume" and the music bars never write "Last Music Volume" (0x698768): no effect in a battle. **Options:** (a) copy; (b) fix in the client's profile: without a device every bar is refused, whatever the volume (the original's for the comparisons only); (c) the client never refuses (its settings kept for a device later). **Recommendation:** (b): the guard's evident intent; the last volumes copied (the map client's system options decide their use). **Built:** as the original's.

#### CLT-Q52 — The cursor shadow's saved hex outlives the combat options dialog (E17 M6 T4c)

**Context.** The cursor shadow ("Show Combat Mouse Hex", `docs/epics/E17-design.md` §12.16) shades the hex under the mouse into the battle's background and keeps that hex's unshaded block to put back when the mouse moves on (`battle_shade_hexes` 0x493EA0, a static list and a last key that outlive a battle). The keys F6 to F8 take the shade off before they change the options; the combat options dialog (its button, or its key O with the mouse over a hex) does not: its close composes the background again with the options as left, so (1) the resting mouse's hex shows no shade until the mouse goes to another hex (the last key is still its hex), (2) that move puts back the block saved before the dialog: with the grid or the movement shadow changed in the dialog a 45 × 52 block of the old state (a grid cell, a shaded or unshaded cell) stays until the background is composed again, and (3) with the cursor shadow turned off in the dialog the old block waits for the option's return, in a later battle restored from a save picture that battle never wrote. Read, not recorded. **Options:** (a) copy; (b) fix in the client's profile: the client's shade follows the state (the hex under the mouse, drawn on the composed background), so a composition forgets it and the next move or the posted mouse (CLT-Q43 (3)) shades the mouse's hex; the original's for the comparisons only; (c) an option. **Recommendation:** (b): a block of a state the player just changed is a slip no player expects. **Meanwhile:** T4c builds the original's.

#### CLT-Q53 — A given-up area or Fire Wall aim leaves its shade (E17 M6 T4c)

**Context.** With the cursor shadow on, an area spell's aim (and Fire Wall's) shades the area's hexes (§12.16). With the movement shadow on too, the aim's end calls `battle_set_shades` 0x479FC0, which takes the shade off; with the movement shadow off nothing does: after Escape or the right button the book's posted mouse (0x474F72) comes with the same hex as the aim's last, so up to 19 shaded hexes stay until the mouse moves to another hex (a cast's action takes them off at its start). Read, not recorded. **Options:** (a) copy; (b) fix in the client's profile: a given-up aim's end takes its shade off whatever the movement shadow, the posted mouse then shading the mouse's hex (the original's for the comparisons only); (c) an option. **Recommendation:** (b): the shade of an aim given up belongs to no command. **Meanwhile:** T4c builds the original's.

#### CLT-Q54 — The cursor shadow in the other aims (E17 M6 T4c)

**Context.** Only the area spells' and Fire Wall's aims draw the cursor shadow (§12.16). (1) A targeted spell's aim, Teleport's and Sacrifice's call no shade: with only the cursor shadow on, the hex shaded when the aim began (the book opened by its key C with the mouse over the field; through the book's button the mouse was over the panel, nothing shaded) stays frozen through the aim while the mouse moves; with both shadows on the targeted aim starts by taking it off (and turns the movement shadow off through the aim, as every aim of the common handler and the wall's does), Teleport's and Sacrifice's keep it. (2) Force Field's aim shades nothing (0x5A334D tests the spell), where Fire Wall's shades its 2 or 3 hexes; its hexes (0, −16, −34 for both sides, `installation.md` §7.15) are not the mirrored ones the wall's list builds, which may be why. Read, not recorded. **Options:** (a) copy both; (b) fix in the client's profile: (1) every aim starts with the cursor's shade off (the original's for the comparisons only); (2) Force Field shades its own hexes; (c) an option. **Recommendation:** (b) for (1): a frozen shade on a hex the aim has nothing to do with is a slip; (a) for (2): an aim without a shade is consistent with the targeted aims and shows nothing wrong. **Meanwhile:** T4c builds the original's.

#### CLT-Q55 — An Inferno's aim off the field marks and shades hexes 1 and 18 (E17 M6 T4c)

**Context.** The aim's cursor calls the area's marks for a refused hex too (§7.18, "Elsewhere"), the mouse off the field included (hex −1: the panel, the margins). `area_hexes` 0x5A4170 takes −1 as column −1 of row 0, so at radius 2 (Inferno; Berserk at Expert through 0x5A4430, which counts alike) the area holds hexes 1 and 18 (row 0's and row 1's column 1): their stacks are marked (the outline 0x70, group 1 started; Inferno's marks walk the same hexes, Berserk's `stacks_in_area` not read) and, with the cursor shadow on, both hexes are shaded (§12.16, oddity 5). At radius 1 nothing. The client's marks show none off the field (`h3_view::mouse::aim`). Read from the arithmetic, not recorded. **Options:** (a) copy (marks and shade on hexes 1 and 18); (b) fix in the client's profile: off the field an aim marks and shades nothing, as the client's marks do now (the original's for the comparisons only); (c) an option. **Recommendation:** (b): an area outside the field touching the corner's stacks is a slip; to record first (`shade-area` with a `move:` onto the panel, §12.16 "To record" 8) to confirm the reading. **Meanwhile:** T4c builds the original's shade; the marks stay as built.

#### CLT-Q56 — The wall aims' last hex survives the aim (E17 M6 T4c)

**Context.** Force Field's and Fire Wall's aim (`aim_wall_handler` 0x5A3250) keeps the hex its cursor last accepted in 0x688328, set at each accepted hex and −1 at a refusal, but never at the aim's start or end (the common aim resets its own, §7.18). So a new wall aim, in this or a later battle, (1) does nothing on that hex (no cursor, no line, no shade) until the mouse moves to another one, and (2) a left press before any move casts at that hex (0x5A34E2 sets the target +0x44 untested), wherever the mouse is. The engine validates the cast it is sent (the battle lane's rules); the original's own cast at a hex that is no longer valid not read. Read, not recorded. **Options:** (a) copy (the client sends the stale hex; the engine refuses it if invalid); (b) fix in the client's profile: a wall aim starts with no hex, as the common aim (a press before a move casts nothing; the first move takes the hex under the mouse), the original's for the comparisons only; (c) an option. **Recommendation:** (b): a cast at a hex the mouse is not on is a slip, and (2) a command the player did not aim. **Meanwhile:** T4c builds the original's shade (1); the press (2) belongs to the aims' commands (§7.18), not to T4c.

### Map client lane (`MCL-Q<n>`)

(no open questions)

### Town lane (`TWN-Q<n>`)

(no open questions)

## Answered

Answered questions are history, kept out of the start-of-session reading. Read one only when a doc, a code comment or a queue entry cites its id (`git grep '<id>' -- docs`). The id's prefix says where it is; when a question is answered, move its entry from "Open" above to the **top** of that file (each lane edits only its own file, so lanes working at once never touch the same lines):

| Prefix | Answered entries |
|---|---|
| `BAT-Q` | [`lanes/battle/QUESTIONS-ANSWERED.md`](lanes/battle/QUESTIONS-ANSWERED.md) |
| `ADV-Q` | [`lanes/adventure/QUESTIONS-ANSWERED.md`](lanes/adventure/QUESTIONS-ANSWERED.md) |
| `SRV-Q` | [`lanes/server/QUESTIONS-ANSWERED.md`](lanes/server/QUESTIONS-ANSWERED.md) |
| `AI-Q` | [`lanes/ai/QUESTIONS-ANSWERED.md`](lanes/ai/QUESTIONS-ANSWERED.md) |
| `CLT-Q` | [`lanes/client/QUESTIONS-ANSWERED.md`](lanes/client/QUESTIONS-ANSWERED.md) |
| `MCL-Q` | [`lanes/map-client/QUESTIONS-ANSWERED.md`](lanes/map-client/QUESTIONS-ANSWERED.md) |
| `TWN-Q` | [`lanes/town/QUESTIONS-ANSWERED.md`](lanes/town/QUESTIONS-ANSWERED.md) |
| `Q<n>` (numbered) and the decisions from before the lanes | [`adr/answered-questions.md`](adr/answered-questions.md) (shared; changed in a `shared/` pull request) |
