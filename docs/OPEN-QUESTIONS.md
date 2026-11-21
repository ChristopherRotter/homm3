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

#### BAT-Q65 — The automatic war machines' turns handed to their side (CLT-Q63 (a); the battle lane's inbox, 2026-11-15)

**Context.** The original runs the computer's action `ai_battle_action` 0x477EE0 for every stack that acts on its own (the battle loop 0x4744CF; `stack_automatic` 0x474BF0: the Ballista without its hero's Artillery or without a hero, the First Aid Tent without First Aid, the Catapult and an arrow tower by their own cases, and every stack under Auto Combat). Its first step is the retreat test (BAT-AI-024, 0x41E570): a computer side retreats or surrenders there; a human side is asked GENRLTXT 414 while +0x1402F is set (its Cancel clears it: the machine acts and the question is not asked again this battle; its OK goes on to the surrender question). The engine plays these turns itself (`turn::begins` → `war::act`, BAT-WAR-002/003, BAT-SIE-011, BAT-SIE-020), so no controller is asked: a computer side misses a retreat the original takes at that turn, and a human is never asked. The user chose to copy it (CLT-Q63 (a), 2026-11-10). It needs a new decision point at those turns, which no controller has today. The arrow towers' path through 0x477EE0 is not read. **Options.** (a) **Every side** with a hero that may retreat gets the decision at each automatic war machine's turn: the machine is the active stack; legal are `Retreat`, `Surrender` where allowed, and a new `Command::Automatic` (the machine acts as now). The computer runs its retreat test there, else `Automatic`. A human client asks the question. The RL env's agents get one decision more per such turn (most battles with a Ballista and no Artillery). (b) **As (a), but only for the sides the setup marks** (`BattleSetup`, per side, default off): the server marks the seats the original's computer plays and the human seats whose client copies the question; the game marks its battles' computer and human sides; the RL env leaves them off, so its action space and speed stay as they are. The human's Cancel ("ask no more") is a per-side bit in the state. (c) **The engine asks the computer's test itself** for a side whose player is not human (`BattleSetup.human`): no new command, but the battle crate runs a controller's decision (ADR-0027 keeps the computer in `h3-battle-ai`), and the human question still needs (a) or (b). **Costs of (a) and (b):** a `Command` variant and the setup's flag on the wire (`cross-lane`, a protocol version); the battle state hash (v56) and the golden replays' hashes; the computer's choice at those turns (`POLICY_VERSION`, the AI lane's bots reports); the replay and scenario format (the flag); the client's question. (b) costs the setup flag in every builder of `BattleSetup` (the game's, the server's, the scenario files'). **Recommendation:** (b), with the arrow towers' path through 0x477EE0 read first (one task), then built in one `cross-lane` branch. The client keeps CLT-Q63 (a) until then. **Blocked:** the user's choice.

#### BAT-Q66 — The Angelic Alliance worn by another hero of the side's player (CLT-Q40's request; the battle lane's inbox, 2026-11-15)

**Context.** The original's side setup sets each side's flag +0x54B2 from `player_hero_wears_artifact` 0x4BACB0 with artifact 129: whether **any** hero of the side's player wears the Angelic Alliance, on the map or in one of its towns; `stack_morale` 0x43E000 reads the flag of the stack's controller's side (0x4697F2). So a player's army joins the good and neutral alignments in every battle once one of its heroes wears it, a heroless town garrison of that player included (`installation.md` §7.32, "The Angelic Alliance's join", read 2026-11-15). VCMI 1.7.5 gives `NONEVIL_ALIGNMENT_MIX` to the wearer alone (`config/artifacts.json`: no propagator, so the wearer's army), and the wiki's alignment page says it allows "a hero" who has it to mix good and neutral creatures. The engine follows VCMI and the wiki (BAT-HER-007: the battle's hero's alignment groups, derived in `h3-heroes`). Gameplay impact: real, wherever a player has the Alliance on one hero and fights with another, or defends a town without a hero. **Options.** (a) Keep the engine (VCMI, the wiki); record the reading as a quirk candidate. (b) The original's: a per-side input "the side's player has a hero who wears it" in the battle's setup (`BattleSetup` or the side's hero effects), derived by the game for every battle of a player (`cross-lane`: `h3-heroes`/`h3-game`, the adventure lane's; the wire). (c) An experiment first: a batch row in the original (a player's two heroes, the Alliance on the one who does not fight; the fighting hero's mixed Castle, Rampart and Tower army; the stats dialog's morale, and the same with a heroless garrison), then (a) or (b) by its result. **Recommendation:** (c), the engine unchanged meanwhile: by `docs/04-rules-and-sources.md` §2 a reading alone does not overturn the wiki and VCMI. **Blocked:** nothing.

### Adventure lane (`ADV-Q<n>`)

(no open questions)

### Server lane (`SRV-Q<n>`)

(no open questions)

### AI lane (`AI-Q<n>`)

(no open questions)

### Client lane (`CLT-Q<n>`)

#### CLT-Q158 — After Reset's OK the creatures show frame 0, then their counter's next frame (E27 M4 T1b, a reading)

High Scores' Reset (`high_scores_proc`'s OK, 0x4EA4B3 and 0x4EA535; `docs/epics/E27-design.md`
§35.1) gives every creature item its DEF again and frame 0, but leaves the dialog's frame
counters (+0xA4) as they are; the next step (at most 200 ms later, at once when one is due at
the close) shows each counter's next frame. So a creature that was at frame 5 shows frame 0 for
up to a step, then frame 6: its animation skips. In `scores-reset` it does not show (the step
follows the close before the next shot). Options: (a) the original's everywhere; (b) the
client's profile also sets the counters to 0 (the creatures start over from frame 0, then 1),
the original's for the comparisons; (c) the client's profile keeps each frame (no frame 0) where
the creature's DEF does not change. **Recommended (a)** (it lasts at most one step after a rare
action; either fix changes the look as much as it cures). Built as (a). Blocks nothing.

#### CLT-Q159 — Delete's OK takes the save off the list whether its file was deleted or not (E27 M4 T4b2, a reading)

The load mode's Delete (`saved_game_delete` 0x58E08B; `docs/epics/E27-design.md` §40.1) calls
`DeleteFileA` on the save and does not look at its result: the entry leaves the list either way,
and a file that could not be deleted (read-only, in use) shows again at the next opening, with no
word of the failure. Options: (a) the original's: the entry removed, the failure told only on the
client's standard error (built); (b) the client's profile keeps the entry and shows a message
(the client's own words) when the file does not delete, the original's for the comparisons;
(c) (b) without the message. **Recommended (a)** (a failure is rare, and the next opening shows
the truth; (b) adds a box of the client's own). Built as (a). Blocks nothing.

### Map client lane (`MCL-Q<n>`)

#### MCL-Q29 — The Deity's week names the Imp's growth for the Familiar (E24 M4 T1a, a reading)

`player_turn_start` 0x4CC7D0 writes the Deity's week (week kind 2, ArrayTxt 135) with the Imp's
(42) name twice, its growth (the creature table's +0x44), the Familiar's (43) name and the Imp's
growth again (0x4CCC8A to 0x4CCCB2: both growth arguments are +0x134C), so the Familiar's line
shows the Imp's number. With the shipped data both are 15, so nothing shows; a pack whose numbers
differ would show it. Built as the original's (design §5.1); no recording has a Deity's week.
Options: (a) the original's everywhere; (b) the client shows the Familiar's own growth, the
original's behind `--bugs original` for the comparisons. **Recommended (b)** (one argument, no
difference with the shipped data). Blocks nothing.

#### MCL-Q30 — The new day's sound started twice (E24 M4 T1a, a recording; sound run at T1b2)

At a turn's start the date's status window is built twice in the same millisecond (the chooser
after `status_window_set_kind` and again at the next view draw, design §5.1 and §4.9), and each
build starts `newday.wav` (`play_sound_async`, category 3). **The sound run (T1b2,
`turn-day` with `sound = true`):** two samples start at End Turn's millisecond on two channels
(Miles' sample handles 7 and 8), each 3 739 ms (`newday.wav`'s 82 432 frames at 22 050 Hz), so the
same waveform plays twice in phase: the new day sounds at double amplitude (about +6 dB louder
than one start), unless Miles' mix clips it. The viewer starts it twice too (built as the original's).
Options: (a) the original's; (b) the client starts it once, the original's behind `--bugs
original` for the comparisons. **Recommended (b)** (the doubling is an accident of the two
builds; one start sounds as the sample was made). Blocks nothing.

#### MCL-Q31 — The view follows a computer's hero still under the fog (E24 M4 T1b1, a recording)

A computer hero's step counts as seen when the viewer has explored its start or its destination
or any neighbour of either (`step_seen` 0x480000, `tile_or_neighbour_seen` 0x41A750; E24 design
§5.2), so the view centres on a hero whose step ends on an unexplored tile next to the explored
land: the screen shows the fog with the hero drawn under it, and the hero comes out only at its
next step (`ai-walk-fog` at 160 and 260). It tells the viewer where an enemy hero stands under the
fog one step early. Options: (a) the original's (the view moves one step before the hero shows);
(b) the client moves the view only at a step whose start or end tile the viewer explored, the
original's behind `--bugs original`. **Recommended (a)**: one tile of lead at the fog's edge,
and the computer's walk then reads as one movement. Built as the original's meanwhile (T1b2).
Blocks nothing.

#### MCL-Q32 — Two spells in a reward's first box replace the map's own text (E24 M4 T2a2, a reading)

`grant_rewards` 0x49FA90 (an event's or a Pandora's Box's rewards, E24 design §5.3 "The rewards'
boxes") shows the map's text in its first box and each part's own line in the later ones. For
the spells it sets the box's text to ADVEVENT 188 ("%s learns spells.") whenever the box already
holds a picture (0x4A05C4), without the test the other parts make (only when the text is empty).
So an event or a box whose rewards are two or more spells and nothing before them (no experience,
skills, spell points, morale, luck, resources or artifacts) shows "<hero> learns spells." instead
of the map maker's text, which is lost. No recording shows it; the shipped maps were not searched.
Options: (a) the original's; (b) the client keeps the map's text there (188 only over a part's
own line), the original's behind `--bugs original` for the comparisons. **Recommended (b)**: the
map's text is the map maker's message, and the other parts never overwrite it. Built as the
original's meanwhile (T2a2). Blocks nothing.

#### MCL-Q33 — The Arena's Cancel button gives +2 Attack (E24 M4 T2b1, a reading and a recording)

The Arena's choice (`visit_arena` 0x49E7D0, E24 design §5.4) is a box of style 10: its two
pictures, +2 Attack and +2 Defense, an OK enabled once one is chosen, and a Cancel. Every answer
but Defense's frame (0x780A) gives Attack, the Cancel's too (recorded: `choose-arena-cancel`, the
status window's Attack 1 to 3), so the Cancel cancels nothing; the Schools' box of the same style
lets the Cancel leave without paying. The engine has the Arena's choice without a way to leave
(`h3_adventure::ChoiceKind::Primary`, `leave` false), so a record never holds it; it matters when
the client plays (M5). Options: (a) the original's: a Cancel that gives Attack; (b) the client's
Arena box has no Cancel (the OK alone, enabled once a picture is chosen), the original's behind
`--bugs original` for the comparisons; (c) the Cancel shown but disabled. **Recommended (b)**: a
button that says Cancel and grants a skill misleads, and the engine offers no leaving. The viewer
plays the original's meanwhile (T2b2). Blocks nothing.

#### MCL-Q34 — The level-up dialog's sentence naming the offers never shows (E24 M4 T2c, a reading and a recording)

The level-up dialog (`level_up_dialog` 0x4F8880, E24 design §5.5) builds a text item under the
primary skill's bar (23, 270, 339 × 52, `medfont.fnt`) with GENRLTXT 205 ("Your hero may learn
either Advanced Armorer or Basic Eagle Eye.") for two offers, 204 ("Your hero has learned
Advanced Mysticism.") for one. Both records start with two line breaks, so at medfont's 20 pixels
the sentence's first line would start at 40 and end below the box: the text drawer draws none of
it (recorded: `level-up`, `level-up-one`; the dialog's texts in memory hold it). The captions under
the offers' pictures name the same skills and levels. Options: (a) the original's: nothing shown;
(b) the client shows the sentence (its leading line breaks dropped, in the box's 52 pixels: two
lines), the original's behind `--bugs original` for the comparisons. **Recommended (a)**: the
dialog's layout was made without it and the captions already tell it; (b) adds text no player of
the original saw. The viewer draws the original's meanwhile. Blocks nothing.

#### MCL-Q35 — A boarded boat drawn turned (E24 M4 T2d2, a reading and a recording)

A hero who boards a boat (`hero_boards_boat` 0x4A1010, E24 design §5.7) is drawn as the boat in
the group of the boat's own facing (`boat_map_group`), but the view mirrors it by the hero's
facing, the step's direction (`view_moving_hero_piece` 0x47F860). A boat whose facing lies on the
other side of the step's draws turned until the hero's next step: recorded, `board`'s boat facing
down-left, boarded by a step east, shows down-right (neither the boat's facing nor the step's); a
boat facing right boarded by a step west would show left. The empty boat a landing leaves faces
the step (the boat record's facing set from the hero's), so the turn shows only at a boarding.
Options: (a) the original's: the boat's group, the hero's mirror; (b) the client keeps the boat
as it lay, its own group and mirror, until the next step, the original's behind `--bugs original`
for the comparisons; (c) the client turns the boat to the step's facing at once (group and
mirror by the hero's, as the next step draws it). **Recommended (b)**: nothing should turn on its
own, and the boat seen before the step is the one the hero boards. The viewer draws the
original's meanwhile. Blocks nothing.

#### MCL-Q36 — A record's start lacks a hero's morale and luck and whether it can move (E24 M4 T3b1, a comparison)

The panels drawn from a whole game's record (E24 design §5.10) need three facts of the viewer's
own heroes its first observation does not hold: the status window's morale and luck
(`H3Hero::GetMoraleBonus` 0x4E39B0, `GetLuckBonus` 0x4E36C0: the hero's skills, artifacts and
specialty and its army's alignments; the engine derives them only into a battle,
`h3_heroes::derive_battle_hero` and `h3-battle`'s morale), and whether a hero can still move today
(`hero_can_move` 0x4E5F30, the hero list's movement mark: its points against the cheapest step
from its tile; the engine's reach answers it only to a `Moves` query, which a record does not
hold). Seen: `engine-start-town`'s Orrin shows morale 1 (Basic Leadership) in the original, 0
drawn from the record (157 pixels of the status window); `engine-start-panels`, no morale or luck
either way, is exact. Options: (a) the engine's view carries them to the owner: the adventure lane
derives a hero's adventure morale, luck and whether it can move (`h3-heroes`, `h3-game`), the
server lane puts them on `OwnMapHero`; the client draws them as given; (b) the client derives them
from the view and the ruleset (rules copied into the client: not wanted, the user at T3b1's
start); (c) the record's session asks a `Moves` query per hero at each turn's start and keeps the
answers (whether it can move only). **Recommended (a)**: one derivation, the engine's, which the
RL observation can use too; notes in the adventure and server lanes' inboxes (2026-11-19).
Meanwhile the viewer shows morale and luck 0 and a hero able to move while it has points
(`stub: MCL-Q36` in `h3_map_view::engine`'s panels). Blocks nothing.

#### MCL-Q37 — A record's start lacks the cursor's facts and a visited bank's guards (E24 M4 T3b2, a comparison)

The mouse-over drawn from a whole game's record (E24 design §5.11) needs facts its observation does
not hold: the selected hero's path costs over several days (the cursor's day groups, 0x40E98F: the
pathfinder's node cost of the tile under the mouse; a `Moves` query answers today's reach only and
a record holds no answer), the tiles a monster guards (the attack cursor, the vision word's bit 8;
the engine's reach knows them, ADV-MON-006, the view does not), whether another team's town or map
garrison holds an army when the viewer's level is 0 (TWN-VEW-002 shows nothing; the attack cursor,
`town_defended` 0x5BDFE0), and a visited creature bank's guards (the hint's "(guards)" after a
visit, 0x40D3F0; the view shows a mine's guards only). The hints are exact without them (every
explored tile of the six starts compared); the cursor differs only over a tile whose cost a
capture read, none of the starts' stills. Options: (a) the engine carries them: the adventure
lane a multi-day reach of the selected hero and the guarded tiles in the view (or a `Moves` answer
extended to days and attacks), the server lane its record of the answers a viewer's client would
ask (the cursor's), a bank's guards in the view to the visitor's team as a mine's; (b) the client
derives them from the view and the rules (copied: not wanted, the user at T3b1's start);
(c) a record viewed without them: the cursor over a reachable tile the arrow. **Recommended (a)**,
as MCL-Q36's (notes in the adventure and server lanes' inboxes, 2026-11-19). Meanwhile none is
written (no reach, no guarded tile, no army, no guards: `stub: MCL-Q37` in `h3_map_view::engine`'s
hover). Blocks nothing until M4 T4 plays a record's walks.

#### MCL-Q38 — A shipped map's town list at a new game is not in the map's order (E24 M4 T3b2, a capture)

At a new game the original lists a player's towns in the map's order on every writer's map
(`lists-towns`, `status-kingdom`: indices 0 to 6), but Hoard's Red lists its Fortress (the game's
town 9) before its Castle (3) (`shipped-hoard`, `start-hoard`: `town_list = [9, 3]`), and Hoard
names no main town for Red; the engine's view lists them in its towns' order (3, then 9), as the
viewer does. Not read: how `setup_towns` 0x4CAA70 or the turn's start builds the list (the player
record's +0x40). Options: (a) read it in the original (the list's build at a new game and the
starting hero's town), then the client orders a record's list as the original does if the rule
needs only the map, else the engine's view gives the order; (b) keep the view's order and list the
difference where a comparison meets it. **Recommended (a)** at the task that first compares a start
with several towns of the viewer's (M4 T4's scripted games or M5). Meanwhile the view's order
(`stub: MCL-Q38`). Blocks nothing: the starts compared hold one town of Red's.

#### MCL-Q39 — A record's walk lacks each step's points and reveal (E24 M4 T4a1, a reading and a recording)

The original takes each step's points off after the step's animation and reveals the tiles
around the step's destination before it (`hero_move_step` 0x4805E0: `reveal_disk` 0x49D160 at
0x48106B, the animation at 0x48117A, the points at 0x481182; E24 design §5.12): the hero list's
movement bar falls at each step's end (`game-walk`: 1560, 1460, 1360, 1260, 1160) and the shroud
lifts at each step's first frame. A whole game's record holds one `MovementChanged` after the walk
(the points it left) and one `GameReveal` after the command's events (every tile the walk
explored), so a viewer playing it knows neither a step's points nor the tiles a step revealed.
Options: (a) the engine tells them per step: the adventure lane's `HeroStepped` (or a
`MovementChanged` after each step) with the points left, and the server lane's reveal folded per
step (the tiles each step explored, in the step's order); (b) the client derives them (the step
costs and the scouting radius: rules copied, not wanted, the user at T3b1's start); (c) a record
viewed without them: the walk's points shown at its last step and its whole reveal at its first,
the difference listed where a comparison meets it. **Recommended (a)**, as MCL-Q36's (notes in the
adventure and server lanes' inboxes, 2026-11-19). Meanwhile (c), played so since T4a2 (`stub:
MCL-Q39`; its gaps listed in `clients/map/timelines/game-walk-gaps.txt` and `game-day-gaps.txt`,
E24 design §5.13). Blocks nothing.

#### MCL-Q40 — A record's normal week and month carry no name (E24 M4 T4a1, a recording)

A normal week's message names the week and a normal month's the month (ArrayTxt 133 and 130 with
a name the original draws, `week_name` and `month_name` among the harness's forced rolls; E24
design §5.1), but the engine's `WeekStarted` and `MonthStarted` say only `Normal` (the wire's
`Week::Normal`, `Month::Normal`): a record holds no name to show. Options: (a) the engine draws the
name with the week's kind and tells it (the adventure lane: an index into the name list, no text;
its own stream, so the golden replays' other draws stay); (b) the client draws it, deterministic
from the game and the week (the viewer's own choice, never the original's); (c) a fixed name (the
list's first). **Recommended (a)**: the name is a draw of the original's game (its `week_creature` 0x6983FC
holds a normal week's number) and every client of one game must show the same; a note in the adventure lane's inbox
(2026-11-19). Meanwhile (b), played so since T4a2 (`stub: MCL-Q40`: the week's number less one
modulo the 15 names, the month's modulo the 10); `game-week` forces the original's draw to a name
and its frames list the difference (E24 design §5.13). Blocks nothing.

#### MCL-Q41 — A record's choice and offer carry no parts (E24 M4 T4b1, a record and a recording)

A treasure chest's choice shows both its parts before the answer (`chest_gold_or_experience`
0x4A6440: ADVEVENT 146 with the gold's and the experience's pictures, each with its amount; E24
design §5.4), but a whole game's record opens the decision without its view (`ChoiceOpened` and
`OfferOpened` name only the object; the server lane's `SRV-STUB-5`: the views of an offer, a
choice, a University and a hire dialog are not folded): the viewer learns the part taken from the
answer's gain and nothing of the other. Options: (a) the engine's events carry the decision's
view (a chest's gold and experience, the Arena's and a School's skills and price, an offer's
kind and price: what the observation's `Decision` holds), so the fold keeps it (the server lane,
SRV-STUB-5, with the adventure lane's events); (b) the client derives the other part (the
chest's experience the gold less 500 by the hero's Learning: a rule copied, not wanted); (c) the
part not taken shown without its amount. **Recommended (a)**; notes in the server and adventure
lanes' inboxes (2026-11-20). Meanwhile (c), played so since T4b1 (`stub: MCL-Q41`: `game-asks`'s
chest box shows the experience's picture with no amount; its frames list the difference, E24
design §5.14). The Arena's gain and Pandora's Box's rewards come from the answer's gains, which
the record holds. Blocks nothing.

#### MCL-Q42 — A record's visit again to a once-per-hero object tells nothing (E24 M4 T4b1, a record)

A Learning Stone's or a Star Axis's second visit by the same hero shows its visited box
(ADVEVENT 144, 101; E24 design §5.3), and a Temple's, a bonus visitable's, comes as
`NothingGained { why: Visited }` in the record; but the once-per-hero objects (the hero's
`marks`, ADV-VIS-006: the Learning Stone, the Star Axis, the Tree of Knowledge, …) give no event
at a visit the mark refuses (the engine's `revisit` of both: an empty event list), and the fold
keeps no hero's marks after the snapshot: the viewer cannot tell such a visit from one that
gives nothing. Options: (a) the engine tells it as the bonus visitables' (`NothingGained {
Visited }`, the adventure lane), and the fold keeps the hero's marks; (b) the client tells a
visit with no event as visited (an inference from silence); (c) no box (the difference listed
where a comparison meets it). **Recommended (a)**; a note in the adventure lane's inbox
(2026-11-20). Meanwhile (c) (`engine::visits` tells the visited box once the event comes; no
scripted game visits one twice yet). Blocks nothing.

#### MCL-Q43 — A record does not tell another player's heroes' points: kind 5's hourglass (E24 M4 T4c, a record)

The original's status window of another player's turn (kind 5, `status_other_turn_step`
0x4536F0, E24 design §5.2) fills its glass by the computer's heroes' points: its value is 1 000
plus the sum of their maximum movement points, and each step raises the glass towards (value −
the points left) × 9 / value, so every player watches a computer's turn's progress. A viewer's
record holds no other player's heroes' points (`MovementChanged` goes to the owner alone), so the
viewer cannot fill the glass as the original does. Options: (a) the engine tells every viewer
the two numbers kind 5 shows (a computer's heroes' maximum points summed at its turn's start, its
points left at each of its commands' ends: what the original's hourglass already shows every
player), a protocol change (the adventure and server lanes); (b) the client fills the glass by
its own measure (the turn's seen steps); (c) the value 1 000 and no points left: the glass rises
one frame a step to its last (the comparisons list the difference). **Recommended (a)**; a note
in the adventure and server lanes' inboxes (2026-11-20). Meanwhile (c) (`stub: MCL-Q43` in
`h3_map_view::engine`'s played). Blocks nothing.

#### MCL-Q44 — A record does not tell a seen step's end the viewer does not see (E24 M4 T4c, a record)

The original draws another player's step when the viewer sees its start or its end tile or a
neighbour of either (`step_seen` 0x480000, design §5.2): a hero walking into sight is drawn
stepping out of the fog, one walking out of sight stepping into it. The engine's projection
(ADR-0016 point 6) leaves out a step's tile the viewer does not see (`HeroStepped`'s `from` or
`to` none), so the viewer has neither the step's direction nor its other tile. Options: (a) the
engine tells both tiles of a step the viewer sees by either end (the hidden tile is a neighbour
of one the viewer sees, and the original draws the hero there), amending ADR-0016 point 6 (the
adventure lane); (b) the client shows a step into sight as the hero appearing on its tile (no
animation, no time) and a step out of sight not at all, the hero left where it was last seen;
(c) a guessed direction. **Recommended (a)**; a note in the adventure lane's inbox (2026-11-20).
Meanwhile (b) (`stub: MCL-Q44` in `h3_map_view::turn::record_walks`). Blocks nothing.

#### MCL-Q45 — A record does not tell which of another player's heroes take a turn (E24 M4 T4c, a record)

A computer's turn selects each of its heroes in turn (`ai_hero_turn` 0x5261F0: the view centred
and drawn on one the viewer sees, even when it does not move, then its walk), and a second pass
over those with points left (design §5.2). A record holds the steps the viewer sees and nothing of
a hero that does not move or moves out of sight: the viewer cannot select those as the original
does (the view centred on a seen hero standing still). Options: (a) the engine tells each hero's
turn's start to the viewers that see the hero (an event of the AI's turn, the adventure lane; a
scripted game's heroes would need a command for it); (b) the client selects only the heroes the
record shows stepping, in the order they first step; (c) the client selects every seen hero of
that player in the kingdom's order. **Recommended (b)** for scripted games and records (a
script's turn has no AI order to tell); the difference listed where a comparison meets it. Built
as (b). Blocks nothing.

#### MCL-Q46 — A record does not tell which of another player's heroes takes an object without a step (E24 M4 T4c2, a record)

A computer's hero takes a pile or an artifact from beside without stepping when it already stands
next to it (`game-ai-boat`'s day 3: Blue's hero on (19, 12) takes the pile on (19, 11)): the
original selects the hero (the view centred on it, standing, turned to the object) and dissolves
the object off with the pickup's sound, as at a walk's end (design §5.16, §5.18). The viewer's
record holds only the command's `ObjectRemoved` (the type and the anchor): no step, no hero, so the
viewer cannot choose the hero, centre the view or turn it. Options: (a) the engine tells the visit
to the viewers who see it (the hero in `ObjectRemoved`, or an event of the visit with the hero and
the object; the adventure lane, a protocol change); (b) the client takes the player's one hero
standing beside the anchor (an inference; two beside it ambiguous); (c) the object taken off the
map as the next turn the record shows starts, the turn not shown (no time, nothing drawn; the
comparisons list the difference). **Recommended (a)**; a note in the adventure lane's inbox
(2026-11-21). Meanwhile (c) (`stub: MCL-Q46` in `h3_map_view::engine`'s played). Blocks nothing.

#### MCL-Q47 — The map's music after a battle (E24 M4 T5a, a code reading)

The original asks for no terrain track when the map comes back from a battle: the adventure
manager's exit lets the map's track play on into the battle (`music_track_play(-1)` asks for
nothing, 0x4077E0), the battle's combat music and its result's tune replace it, and back on the
map only `hero_select` (through `mobilize_hero`, 0x4AE8F3) asks for a track, and only when the
hero's terrain differs from the one asked for last (adventure manager +0x58, 0x418113). So after a
battle on the hero's own terrain the map would be silent once the tune ends, until the next walk's
or key's end asks for +0x58's track again (design §5.19). Read only: the harness logs samples, not
the music, and nobody has listened. Options: (a) as the original: no track asked for at the return;
(b) the client asks for the terrain's track as the map comes back (the client lane's
`HostSound` fades the tune out into it, E17 design §13.6); (c) (a) in the original's profile
(`--bugs original`), (b) in the client's. **Recommended (a)**, checked by ear at M4 T6's hand test
(the original on the user's machine: a battle won, then a wait without moving). Built as (a) at
T5b meanwhile. Blocks nothing.

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
