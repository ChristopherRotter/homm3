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

#### CLT-Q63 — Auto Combat's retreat question at an automatic Ballista's turn (E17 M6 T5b2a)

**Context.** The original asks a human the retreat question (GENRLTXT 414, `docs/epics/E17-design.md` §12.19) from `ai_battle_action` 0x477EE0, which the battle loop runs for every stack that acts on its own (0x4744CF). Such a stack is not only Auto Combat's: `stack_automatic` 0x474BF0 counts the Ballista as automatic when its hero lacks Artillery (or it has no hero) and an arrow tower when the hero lacks it, with Auto Combat off (0x474C59; a fresh reviewer's reading of T5b2a, 2026-11-06, against the map's entries). So a human whose hopeless hero is worth keeping, with a Ballista and no Artillery, is asked at the Ballista's turn without having touched Auto Combat; its Cancel lets the Ballista shoot and asks no more this battle, its OK leaves the battle. The engine plays the automatic Ballista itself (BAT-WAR-002), so the client is never asked there, and a computer side's retreat at that turn is missed too (the battle lane's inbox, 2026-11-06). The arrow towers' path through 0x477EE0 is not read.

**Options.** (a) Copy: ask it there as the original does; it needs the engine to hand the client the automatic war machine's turn (a battle-lane change, a cross-lane PR like #745). (b) Fix in the client's profile: ask it only under Auto Combat (the question names the odds, not the Ballista; a player without Auto Combat chose to fight), the original's way only for the comparisons.

**Recommendation.** (a) copy, built when the battle lane hands over the automatic war machines' turns (it also fixes the computer side's missed retreat); until then *not built* (design §12.19).

#### CLT-Q64 — Auto Combat's button on a turn it plays after a switch on a human-played turn (E17 M6 T5b2b)

**Context.** Recorded (`auto-retreat-later`, `docs/epics/E17-design.md` §12.20): A on a turn Auto Combat does not play (a commanded Ballista whose Combat Ballista box is clear: the turn stays the human's, CLT-Q45's case) sets the flag; the next turn it plays from its start (the Dwarves' waited turn) shows the panel dimmed with Auto Combat's button dimmed and unselected too. A turn it plays after a switch in that turn (`auto`, `auto-retreat-cancel`) shows the button enabled and selected; the code reading of 0x4784F9 (§12.13) had it so on every turn Auto Combat plays, a reading to recheck. So a player who switched Auto Combat on at a war machine's turn sees no sign of it on the turns it then plays, and no button to switch it off there (a key or a press still stops it as the next turn starts).

**Options.** (a) Copy. (b) Fix in the client's profile: the button enabled and selected on every turn Auto Combat plays, the original's for the comparisons only. (c) An option.

**Recommendation.** (b): the same slip as CLT-Q45 (1), whose answer (the button selected while Auto Combat is on) already covers it. **Built:** the original's profile copies it (the comparisons, `panel::Autoplay::switched`); the client's profile shows the button selected while Auto Combat is on (`Autoplay::on`, CLT-Q45 (1)'s answer).

#### CLT-Q65 — Eagle Eye's message names spells the hero did not learn (E17 M6 T5c)

**Context.** Read (`docs/epics/E17-design.md` §12.21): after the result window of a winner played by a human here, `eagle_eye_message` 0x476FE0 names every spell the side's Eagle Eye picked in the battle ("is able to learn …", GENRLTXT 222), but `eagle_eye_learn` 0x469FE0 taught the hero only those it can learn: while it wears a Spell Book and the spell's level is at most its Wisdom + 2 (HER-SPL-006). A winner without its book, or with a level 3 or 4 pick and too little Wisdom, is told it learns spells it does not get.

**Options.** (a) Copy. (b) Fix in the client's profile: name only the spells the hero learns, the original's for the comparisons only. The battle's outcome reports the picks (`HeroOutcome::learned`); who learns them is the orchestrator's (BAT-Q24), so in a whole game the client takes the game's answer, and in a battle without a map (no hero kept after it) the picks of a hero that has a spell book. (c) An option.

**Recommendation.** (b), once a whole game's battle tells the viewer what its hero learned (the map client's battles, E24 M4 T5, or the server's whole game, E16 M5); until then the picks are all the client knows. **Built:** as the original's (every pick named) in both profiles. **Recorded** (M6 T5c, `eagle-eye`): Orrin without Wisdom is told he learns Force Field (level 3) and Chain Lightning (level 4).

#### CLT-Q66 — The result window stays under Eagle Eye's boxes (E17 M6 T5c)

**Context.** Recorded (`eagle-eye`, `eagle-eye-one`; `docs/epics/E17-design.md` §12.21, "Recorded"): the window's OK closes it, but nothing redraws the screen before Eagle Eye's first box opens: every box stands over the window as it last showed (its OK up, its clip's frame of the close's millisecond, still), over the battle darkened, until the last box's OK ends the battle's screen. The window looks open while it no longer takes input, and its clip stands still where a smaller box leaves it visible.

**Options.** (a) Copy. (b) Fix in the client's profile: the boxes over the darkened battle alone, the original's for the comparisons only. (c) An option.

**Recommendation.** (a): harmless, and the boxes read as the window's report continued (the spoils beside the casualties). **Built:** as the original's in both profiles (`results::released`, the sequences' and stills' `window`, `Battle::window_behind`).

### Map client lane (`MCL-Q<n>`)

(no open questions)

### Town lane (`TWN-Q<n>`)

#### TWN-Q8 — Increment 2's survey checkpoint: TD-9, TD-11, TD-12, T-8, the computer's day (E09 T12b)

**Context.** The construction survey `docs/rules/surveys/E09-construction.md` (§0 to §11) is done: the human's build through the hall, its tests, costs and requirements (T12a), the map's town events and the census (T12b). Its §11 lists what T13's drafts need decided. **TD-9** (the Conflux's upgraded Altar of Fire, 40): the executable's list needs the Altar of Fire and the upgraded Altar of Air; VCMI and the fandom wiki the Altar of Fire. **TD-11** (who builds and pays): the hall's path tests no owner and the current player pays, so a visitor might build in an ally's town at its own cost; VCMI the owner only. **TD-12** (an every-day town event, 18 on the shipped maps, daily gold): the reading from its first day, as the wiki; VCMI from the day before. **T-8** (quirk candidate): an event's Capitol beside the owner's other one stands (the reading and VCMI; six shipped events can give it). **The computer's every other day** on Easy and Normal and the day's mark across a capture (§3): the engine has no difficulty and no computer player yet. **Options and recommendations:** TD-9 and TD-11 the reading or VCMI, or observed first in a batch TWN-2 (recommended: TD-9 the executable's list until a TWN-2 row says otherwise; TD-11 the owner only until a TWN-2 row says otherwise); TD-12 and T-8 as the original (recommended); one a town a day for every player, the handicap left to E10 and E13, the capture's mark as read (recommended). **Blocked:** T13 (the drafts, the design note's Part D and the cut).

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
