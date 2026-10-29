# Open questions

Questions waiting for the user. Agents add new ones here (context, options, recommendation, what is blocked). When answered, record the decision (ADR or doc) and move the entry to the top of its lane's `QUESTIONS-ANSWERED.md` (table at the end of this file).

Since Phase 0 (2026-10-07) every lane adds its questions here with a lane id that counts per lane: `BAT-Q1`, `ADV-Q1`, `SRV-Q1`, `CLT-Q1`, `MCL-Q1`, `AI-Q1` (`lanes.toml`); the numbered ids (`Q55`) stay. A new entry goes under its lane's block at the end of the "Open" section, as `#### <CODE>-Q<n> — title` (blocks keep concurrent lanes from editing the same lines).

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

#### CLT-Q26 — The spell book's last hovered item is never reset (E17 M5 T3a)

**Context.** The book's handler changes the status line and the cursor only when the mouse moves over another item than the last one it was over (`spell_book_proc` 0x59D6DA against the global 0x684B4C, `docs/epics/E17-design.md` §7.17). The global starts at -1 and nothing resets it: when the book opens again with the mouse resting on the item it was last over (in any earlier opening), moving the mouse within that item shows no hint and leaves the cursor as the battle's until the mouse reaches another item. **Options:** (a) copy it; (b) fix it (a new opening starts with no item: the first move shows the hint), the original's for the comparisons only (`Bugs::Original`, as the cursor's x); (c) an option. **Recommendation:** (b): a drawing bug with no rule behind it, like the cursor's rounding. **Built:** as the original's (`book::Memory`). **Blocked:** nothing.

#### CLT-Q27 — The spell aim's slips (E17 M5 T3b)

**Context.** The aim's loop (`enter_player_cast` 0x59EC50, its handler 0x5A3070; `docs/epics/E17-design.md` §7.18) has four slips with no rule behind them. (1) A Faerie Dragon's aim given up (Escape, the right button) posts no mouse move, unlike the hero's book: the pointer (the combat cursor's frame 6) and the aim's status line stay until the mouse moves onto another hex than the one the battle last took (0x475CE8). (2) The cursor's answer at the aim's start (at the book's place or where F was pressed) is not kept (0x59EE56): over a hex it takes, the spell cursor shows, yet a click before the mouse moves casts nothing. (3) The spell cursor starts its frames again at each hex it takes, unless the frame shown then is the spell's number + 1 (0x50CD37: type 3 always draws its frame 0). (4) An area spell over a side column's hex is refused (frame 0, "Select Spell Target") yet marks its area's stacks (0x59F87E). **Options:** (a) copy them all; (b) fix (1) and (2) in the client (the key's cancel posts the mouse as the book's does; the start's hex kept, so a click casts there), the original's for the comparisons only (`Bugs::Original`, as the cursor's x), and copy (3) and (4) (cosmetic, harmless); (c) fix all four, the original's for the comparisons. **Recommendation:** (b). **Built:** as the original's (all four). **Blocked:** nothing.

#### CLT-Q28 — The message box's slips (E17 M5 T4a)

**Context.** The message box (`message_box_build` 0x4F6990, its layout 0x4F5D80; `docs/epics/E17-design.md` §7.19) has two slips with no rule behind them. (1) Its text sits at x 20 with the box's width less 50, so every text is centred 5 pixels left of the box's centre (the buttons and the picture are centred on the box). (2) Its frame takes the colours of the player whose view the screen shows (`viewing_player` 0x4CEA20: in hot seat the adventure map's current player), not the side's whose turn it is: in a hot-seat battle the defender's questions (Retreat, Surrender) and the book's boxes come framed in the attacker's colours, while the panel and the book take the defender's. **Options:** (a) copy both; (b) copy (1) (the original's look, cosmetic) and fix (2) in the client (the acting side's player's colours), the original's for the comparisons only (`Bugs::Original`, as the cursor's x); (c) fix both, the original's for the comparisons. **Recommendation:** (b). **Built:** as the original's (both; (2) as the client takes the viewing player, the attacker's when a human plays it here: design §7.19). **Blocked:** nothing.

#### CLT-Q32 — The battle's cursor after the tactics phase's message (E17 M5 T4c2a)

**Context.** The tactics phase's message (GENRLTXT 373, `battle_start` 0x4631E5; `docs/epics/E17-design.md` §7.23) opens before the battle's first turn. When its OK closes it, the first turn's start sets the combat cursor's pointer (0x478321) and posts a mouse move (0x478857), as every turn's start does, but that move does not reach the battle here: recorded (`tactics-rest`), the mouse moved over the message and left over a hex of the zone keeps the pointer and marks no stack at 100 and 1 000 ms, until it moves again (then the move cursor at once). Where the posted move is lost is not read. **Options:** (a) copy it (the first move after the message gives the cursor; harmless); (b) fix it in the client (the turn's start shows the cursor and the mark for where the mouse rests), the original's for the comparisons only (`Bugs::Original`, as the cursor's x). **Recommendation:** (a): the window shows what its last input left anyway, and a move is all it takes. **Built:** as the original's (the stills keep the pointer; the window's battle takes no move made over a box, E17 M5 T4a). **Blocked:** nothing.

#### CLT-Q33 — Summon Elemental's line without a stack (E17 M5 T5a)

**Context.** `summon_elemental` (0x5A7080; `docs/epics/E17-design.md` §7.25) writes its line
("%s summons %d %s", GENRLTXT 676) before it creates the stack, whether or not a column had a
place: without one the hex stays −1, the line is written all the same and `create_battle_stack`
0x47A100 is called with hex −1, whose test reads a byte before the hex records (the battle
manager's +0x16C, not read): nothing is created unless that byte is negative. The engine creates
nothing and reports `NotCreated` (BAT-MAG-037). Rare: every one of fifteen columns must be full.
**Options:** (a) copy the line (the original's log, a line about elementals that never appear);
(b) fix it in the client (no line when nothing is created), the original's for the comparisons
only (`Bugs::Original`, as the cursor's x); the read past the hex records is the engine's to
leave out either way (nothing created). **Recommendation:** (b): the log should not report a
summon that did not happen. **Built:** as the original's (the line written). **Blocked:**
nothing.

#### CLT-Q34 — The palette effects' slips (E17 M5 T5b1)

**Context.** `draw_battle_stack`'s palette effects (`docs/epics/E17-design.md` §7.26) take each
colour through HSV with every channel scaled to 0x7FFFFFFF / its mask. (1) The scales differ (at
5-6-5 a full blue scales to 2^31 − 2, a full red to 2^31 − 2 048), so a grey or white has a hue
(white's is a cyan, 0.51) and a saturation just above 0; where the effect moves the hue (Bloodlust's
glow), white turns purple on its way to pink instead of going straight to pink, and the round trip
through f32 values changes some colours with no effect intended (Bloodlust's step 0, Stone Gaze's
1.0: 0x1234 becomes 0x1213; a blue of 1 becomes black). (2) Each glow's last step (strength 0.1) is
drawn and shown, then replaced in the same millisecond by the redraw that clears the flag
(0x5A11DB): it never stays on the screen. **Options:** (a) copy both; (b) fix (1) in the client
(the channels scaled alike, a grey without hue), the original's for the comparisons only
(`Bugs::Original`, as the cursor's x), and copy (2); (c) fix both, the original's for the
comparisons. **Recommendation:** (a): both are how the original looks, briefly and on a few
colours, and no reference shows what was meant instead. **Built:** as the original's (both).
**Blocked:** nothing.

### Map client lane (`MCL-Q<n>`)

#### MCL-Q13 — A monster's head under the Dragon Utopia (E24 M3 T2b1; the user, 2026-10-28)

The user saw, in the window on `path-objects` (map-measure's runs 5 and 6), the Gold Dragon's
head drawn under the Dragon Utopia. The dragon (`AVWdrax0.def`, 2 × 2 at the map's (10, 10) and
(11, 11)) stands on cells of the Utopia's picture (`AVSutop0.def`); on those four tiles the draw
lists hold the dragon's body before the Utopia's (`tile_draw_insert` 0x505230's order by layer and
height, design §2.2), so the Utopia's base covers the dragon's upper half. **The original draws
the same:** the spot is in `path-objects`' start view, a still compared exact with the original's
capture (`set.toml`: 0 pixels differing). An oddity of the original (a guard placed on another
object's picture), not of the client. **Options:** (a) keep the original's drawing; (b) a client
rule drawing a monster over the object whose picture it stands on (the draw lists' order changed
for such cells: the stills' comparisons then need `--bugs original` for it). **Recommendation:**
(a): the order is the rule every still is checked against, and the map's author placed it so.
**Blocked:** nothing.

#### MCL-Q14 — The mana bar a spell leaves on the screen (E24 M3 T4b, a reading)

The hero list's slot (`hero_list_slot` 0x403560, every call with −1, 1, 1) draws the selected
hero's portrait, movement bar, mana bar and frame, then shows only its rectangle (609, 212 + 32 i,
48 × 32): the movement bar and the portrait's first 40 columns, not the mana bar (666–671) nor the
portrait's last 8 columns (E24 design §4.5, "The slot's rectangle"). After a spell cast on the
map (`adventure_spell_cast` 0x41D2D5, `hero_relocate_spell` 0x41D8B4: `hero_spend_mana`, then the slot;
`hero_spend_mana` 0x4D931E itself too) the new mana bar is drawn but not shown, so the screen
keeps the old one until something shows the whole screen (a list's update, a lift that centres
the view, a dialog closed). Inferred from the reading, not yet seen: M3 casts no spell, and what
follows a cast (its own dialogs, `path_rebuild`) may show the screen. **Options:** (a) the
original's: the mana bar shown only with the whole screen (the timeline keeps what each part
last showed); (b) the client shows the slot whole. **Recommendation:** M4 records a cast first;
if the stale bar is seen, (a) for the comparisons behind `--bugs original`, (b) by default (a
display artifact, not a rule); built meanwhile as the original's: T4b's slot shows the points
alone. **Blocked:** nothing.

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
| `Q<n>` (numbered) and the decisions from before the lanes | [`adr/answered-questions.md`](adr/answered-questions.md) (shared; changed in a `shared/` pull request) |
