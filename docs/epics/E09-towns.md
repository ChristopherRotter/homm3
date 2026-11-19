# E09 — Towns

Status: **in progress** (refined 2026-10-31 with the user, TWN-Q1; the town lane opened the same day). Increment 1 (towns on the map) closed on 2026-11-06 (T10; its scope, decisions 8–35 and tasks in `E09-archive.md`), increment 2 (construction with the town events) on 2026-11-11 (T20; decisions 36–63), increment 3 (growth, recruitment and the army rows) on 2026-11-15 (T30; decisions 64–97), all three in `E09-archive.md`; increment 4 (the Tavern) refined the same day (TWN-Q15, decisions 98–102), its drafts accepted at T33b (TWN-Q18, decisions 114–122), batch TWN-4 run and scored by T34 (decisions 123, 124), T35 cut in two at its start (TWN-Q19, decisions 125–128): T35a the ADRs done, T35b the code done (decisions 129–131), T36 the town part v7 done (decision 132), T37a done (decisions 133–136), T37b done (decisions 137–139), T38a cut in two at its start (TWN-Q22, decisions 140–142): T38a1 done, T38a2 done (decisions 143, 144), T38b cut in two at its start (TWN-Q23, decision 145): T38b1 done (decision 146), T38b2 done whole (TWN-Q24, decisions 147, 148); the position and the next task are in `docs/lanes/town/HANDOFF.md` and `QUEUE.md`. The record of every done task: `E09-log.md`.

## Goal

Towns as the original plays them, on the adventure map of E08: the town context in its own domain crate `h3-towns` (buildings and construction, growth and the available creatures, the mage guild, the market and the other services), the town as the adventure's map object (owner, garrison, visiting hero, entering, capture by siege), composed by `h3-game`. Built like E08: each increment is complete in structure (commands, validation, events, views, the env, replays, the gates) and grows the content; RoE, AB and SoD for what it builds (E11: the edition of the pack set). The first increment unlocks the shipped maps that wait only for towns: 128 of the 160 are refused today for a hero generated at a main town (`docs/formats/h3m.md`, "Import").

## Decisions at the refinement (2026-10-31, the user, TWN-Q1)

1. **A lane of its own, now.** E09 opens at once in a new lane `town` (code `TWN`), in parallel with the adventure lane's E08 (increment 9's rounds 6 to 8, then its sketched increments 10 to 12). Not chosen: E09 after E08's increment 9 with E08 closing (its increments 10 to 12 split between E09 and E12); after increment 9 with E08 kept open; after E08's increments 10 to 12.
2. **The crate `h3-towns`** (the user's name; ADR-level, the details in increment 1's ADR draft): a domain crate holding the town context's own state and rules (buildings, construction, growth, the available creatures, the mage guild, the market and the services). **The town as a map object stays the adventure's** (its tiles and entrance, owner and flag, garrison, visiting hero), as a mine and its guards; **`h3-game` composes** the visit, recruitment into an army, the town's part of the day and the week, and the siege's derived facts (ADR-0023). The ADR draft decides where the town's state lives in the game state (its fork and hash), its ruleset part and pack file, the seam types and the hero pool's home. Not chosen: a module inside `h3-adventure`, split out later; `h3-towns` owning the town's armies too.
3. **Ownership: town sub-paths** (`lanes.toml`). The town lane owns `h3-towns` and the town modules inside the adventure lane's crates by the longest match: `h3-adventure`'s and `h3-game`'s `src/town*`, `h3-map`'s `src/import/town*` and `src/writer/town*`; the town rules (`docs/rules/catalog/town.toml`), its quirks ledger `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Those crates' common parts (`lib.rs`, the state structs and hashes, the versions, the replays and the bot reports) stay the adventure lane's: the town lane changes them in a `cross-lane` pull request, merged in order with the adventure lane's (the second rebases and bumps again). The scoped verify `--scope town` runs the adventure lane's crates, replays and workloads too (`verify_with`). Not chosen: the town lane owning only `h3-towns` with every other change `cross-lane` by consent; the adventure lane building the map side from its inbox; increment 1 reordered to the town's own rules first.
4. **E08's sketched increments 10 to 12 stay E08's** (dwellings, random dwellings and the Refugee Camp with hiring, garrisons; creature banks; quests). The hire dialog and the weekly growth are built once, by whichever lane needs them first, and the other reuses them (a note in the other lane's inbox when they land). Not chosen: dwellings, hiring and garrisons moved to E09; E08 closing after its increment 9.
5. **Increment 1: towns on the map** (closed; `E09-archive.md`). Not chosen: the economy loop first (construction, growth, recruitment), capture and the main town's hero in increment 2; both in one increment.
6. **E09 also owns** (all four of the border topics offered): **the player's faction choice at the start and random towns** (class 77) resolved from it (otherwise E10's scenario setup); **Town Portal and Castle Gate** (the town-travel spell and the Inferno building; the other adventure spells stay unplaced, E08's "Not placed yet"); **the Grail** (obelisks, the puzzle map, digging and the Grail buildings); **the Tavern's information and the Thieves' Guild** (what a player learns of the others by taverns owned; a projection).
7. *Agent's calls (change freely):* each increment is built as E08's are: a survey (the code reading of `Heroes3.exe`, the wiki and VCMI, a census of the shipped maps), the ADR drafts with a design note and the cut, the user's acceptance before the build, an experiment batch for the readings at risk, the build in rounds, the replays and gates, an audit of the exit criteria; a user checkpoint per increment and the cut confirmed at each round, as a domain lane. The env plays a new town command in the same increment, `cross-lane` in the AI lane's paths (as E08 decision 203). Experiments are files `experiments/specs/E09-batch-<n><x>.toml`, called "batch TWN-<n>" in prose to keep them apart from E08's "batch E9". The computer player's town play is E10's and E13's.

## Increments (sketch; the order and the cut of each decided at its refinement, as E08 decision 48)

1. **Towns on the map** (closed 2026-11-06; `E09-archive.md`).
2. **Construction** (closed 2026-11-11; `E09-archive.md`).
3. **Growth, recruitment and the army rows** (closed 2026-11-15; `E09-archive.md`).
4. **The Tavern** (refined below, decision 99; the sketch's increment 3 in part until decision 65): the Tavern's hire with its pool and the eight heroes' limit, the Tavern's information and the Thieves' Guild.
5. **The mage guild and the services** (the sketch's increment 4 until decision 65): the guild's spells drawn (the town chances, the map's required and banned spells), learning them (Wisdom), the spell points restored; the Marketplace and the Resource Silo, the Blacksmith, the town's Shipyard (E08's `BuildBoat`), the Artifact Merchants and the other faction services; the special buildings for heroes and sieges (the Castle's Lighthouse, A-30; the Necropolis's Necromancy amplifier, the queue archive's T19a2b note; Cover of Darkness).
6. **Travel and the Grail** (the sketch's increment 5 until decision 65): Town Portal and Castle Gate; the obelisks, the puzzle map, digging and the Grail buildings.

Citations of "increment 4" and "increment 5" written before decision 65 in the log, the archive, the surveys and the design note's Parts A to D mean today's 5 and 6 (decision 67).

## Closed increments

Their scope, decisions and task lists are in `E09-archive.md` (cited as "E09 decision N"; `git grep -n 'decision 22' docs/epics/E09-archive.md`); increments 1's to 3's design notes stay in `E09-design.md` (decision 40; Parts A to E), the record of every task in `E09-log.md`. Decisions 1–7 above and those from 98 below stay here.

| Increment | Closed | Tasks | Decisions | Content |
|---|---|---|---|---|
| 1 | 2026-11-06 | T0–T10 | 8–35 | towns on the map: fixed and random towns imported and built, the players' factions and the starting-hero pick, the hero pool with the main towns' and the random heroes, the town's record and store, entering, capture and the siege with its facts from the town and the garrison's merge, the day's income with the silo, the views by the original's levels, batch TWN-1; all 160 shipped maps import and build |
| 2 | 2026-11-11 | T11–T20 | 36–63 | construction with the town events: the random dwellings tied to a town, the build command (`TownCommand::Build`: the owner in its turn, the hall's offer by the executable's closed requirement lists, the forbidden, one Capitol a player, the hordes as their upgrades, one a town a day, `Building.txt`'s costs), what a build changes of increment 1's reads, the Shipyard's water, the map's town events (their days, players and kinds, resources and buildings; their creatures `TWN-STUB-10`), the views and every viewer's changed view told, the env's build entry, the builds map and its replays, batch TWN-2 (TD-9, TD-11); all 160 shipped maps play their town events |
| 3 | 2026-11-15 | T21–T30 | 64–97 | growth, recruitment and the army rows: the part's roles `horde`, `portal`, `week_of` and `roe` with the growth's parameters, each town's fourteen counts and Portal (`TWN-QUIRK-8`), a dwelling's growth and first creatures, the week's kind from the towns' stream, the week and the month with a creature's and the Plague (`TWN-QUIRK-9`, `-10`), a neutral garrison's week, the Portal's draws, the town events' creatures, recruitment through the hire dialog, the town's two bars, the garrison hero (enter, leave, swap; its day, income and defence; `TWN-QUIRK-11`), the views, the env's `RECRUIT` and `TOWN_HEROES`, batch TWN-3; the census of the 160 maps to day 29 |

## Increment 4 — the Tavern

Refined with the user on 2026-11-15 (T30; `TWN-Q15`, asked at T30's start with a first look at the audit). Built like increments 1 to 3: commands, validation, events, observation, the env, replays and the gates grow with the content. The audit of increment 3's exit criteria is in `E09-log.md` (T30).

### Decisions at the refinement (2026-11-15, the user; TWN-Q15)

98. **T30 whole** (the user at T30's start, `AskUserQuestion`; recommended): the audit, the closure and this refinement in one session and one docs-only pull request, as T10 and T20 (decisions 36, 64); a gap needing code queued. Not chosen: T30a (the audit and the closure), then T30b (the refinement).
99. **Content: the Tavern as sketched** (recommended). **The Tavern's hire:** each player's two offers in its towns' taverns, drawn from the shared hero pool (the game's, decision 18) by the hero classes' tavern weights (harvested: the classes' table, whose RoE form lacks the Conflux column; the inbox's E04a T3a note), the price, the most heroes on the map (the part's `most_heroes`, 8), the week's refresh and a hired offer's replacement, the hired hero on the town's entrance; the heroes that lost, retreated or surrendered back in the pool and offered again (the battle's retreat facts' Tavern town, `ADV-STUB-9`, the computer's choice to retreat E10's); the map's customized heroes as the pool's (`Marshland Menace`'s map-wide 2 000 experience of 15 hero types; a hired type's skill its map bans, the adventure lane's inbox note of 2026-10-20). **The Tavern's information:** the Thieves' Guild's overview by the taverns a player owns (a projection, decision 6) and the Den of Thieves on the map. The rumours stay out (text, no rule). The survey settles each point's reading and the drafts place the Den of Thieves (the adventure's object kinds). Not chosen: the Tavern's hire alone, its information and the Thieves' Guild a later increment.
100. **The first round** (recommended): a survey (T32: the code reading of `Heroes3.exe` for the tavern's offers and their draw, the refresh, the returning heroes and the Thieves' Guild's levels; VCMI and the wiki compared; a census of the 160 shipped maps' hero settings and taverns) and its checkpoint, then the ADR amendment drafts with the design note's Part F and the cut (T33), accepted before anything is built, as increments 1 to 3 (decision 7). Not chosen: the survey alone first.
101. **The garrison moves below level 3** (`TWN-Q15`, the server lane's inbox note of 2026-11-14; recommended): a viewer that does not see a town's garrison hero (below TWN-VEW-001's level 3: neither the owner's team nor expert Visions or a Rogue within reach) is told `GarrisonEntered` as the hero leaving the map and `GarrisonLeft` as a hero coming into sight; the owner's team and a viewer at level 3 keep the named events (`docs/06-security.md`'s town part already says so: the code comes into line). A small task T31 before the survey (code; `cross-lane` where it touches the adventure lane's paths). Not chosen: the move public, `docs/06-security.md`'s town line worded so (a hero vanishing from an entrance may also be a dismissal, so the name tells more than the original's map shows).
102. *Agent's calls (change freely):* **the design note's Part F** in `E09-design.md`, its sections numbered from §59, as decisions 40 and 67; the survey `docs/rules/surveys/E09-tavern.md`; the experiment batch TWN-4; the inbox's notes that increment 3 acted on archived (`QUEUE-ARCHIVE.md`), the Tavern's three kept for T32.

### Decisions at the survey's checkpoint (2026-11-15, the user; TWN-Q16)

103. **TD-18, the week keeps a returning offer** (recommended): the reading (survey `E09-tavern.md`
     §2.3, §4): the week releases every offer but a retreated or surrendered hero that came back
     after its owner's turn ended. Not chosen: VCMI's release of both slots; the reading with a
     `fix-optional` flag.
104. **TD-19 and TD-20** (recommended): the reading for both: a returning hero's slot random,
     sparing another returning offer; the offers' movement at their owner's turn start, spell
     points only at a draw or a leave. Not chosen: VCMI's roles and daily regain. (The spell-point
     half amended by decision 127.)
105. **T-14, a dismissed hero's artifacts** (recommended): the original's, not a quirk: they stay
     with the hero in the pool (row TWN-4-5 confirms). Not chosen: a `fix-optional` flag.
106. **The map's Tavern in increment 4** (recommended): class 95 hires as a town's tavern onto its
     visited tile, no guild spells (`cross-lane`: the adventure's object kinds). Not chosen: E08's
     trades and services.
107. *Agent's calls (change freely):* TD-21 as read (one bound, `most_heroes`, on the map only);
     T-15 for the packs' validation; the SoD edition's Conflux rule to E11 as a lead; batch TWN-4's
     rows for T33's cut.

### Decisions at T33's start (2026-11-16, the user; TWN-Q17)

108. **T33 in two** (recommended): T33a the hire (the offers and their draw, the week and the
     start, the town's and the map Tavern's hire, the heroes that come back, the map's hero
     settings: their drafts and Part F §59 to §71); T33b the information (the Thieves' Guild, the
     Den of Thieves), the views, the env, the gates, batch TWN-4's rows, the building tasks and the
     exit criteria, then the checkpoint. Not chosen: T33 whole; every draft first, the design
     note's sections next.
109. **A hero type's record at its first draw** (recommended): a type gets its hero id and details
     the first time a tavern offers it and keeps them by type from then on (the original's 156
     records: a hero that left the game comes back as it left); the store of heroes grows in play.
     Not chosen: every allowed type's record at the build; a fresh record at every draw.
110. **The tavern weights in the town part** (recommended): each faction's town record holds its
     weight per hero class (`HCTRAITS.TXT`'s column); no shared change, no override for RoE's
     missing Conflux column. Not chosen: the core's hero classes with a weight per faction.
111. **The hire's two shapes** (recommended): a town's tavern by a town command (the town and the
     slot, in the owner's turn, no decision); the map's Tavern by a decision its visit opens,
     answered by a hire of one slot or a close. Not chosen: a decision for both.
112. **The dismissal in increment 4** (recommended): a management command; its hero leaves the
     game as a lost hero's does, its artifacts kept (decision 105). Not chosen: later.
113. *Agent's calls (change freely; for T33b's checkpoint):* a new named kernel stream for the
     taverns' fills (`StreamId::TAVERNS`); the away record in the pool; a Random faction no start
     read drawn at the first fill; a player without a faction weighted evenly; the start's fill
     after the build's last draw; a new record's points; `ADV-STUB-9` left to E10; T-15 closed as
     unreachable; T-16 (design §67: a `fix-optional` flag, on in `faithful`) and TD-22 (survey
     §12: the reading) as recommended for the checkpoint.

### Decisions at T33b's start and checkpoint (2026-11-16, the user; TWN-Q18)

114. **T33b whole** (the user at its start, `AskUserQuestion`): the information, the views, the env,
     the gates, batch TWN-4's rows, the cut and the exit criteria, then the checkpoint, in one
     session and one docs-only pull request, as T22b. Not chosen: T33b1 (the information, the
     views, the env and the protocol: the drafts made whole) then T33b2 (the gates, the cut, the
     checkpoint), the agent's recommendation; the drafts first, then the plan.
115. **The taverns' stream** (recommended): a new named kernel stream `StreamId::TAVERNS` for the
     start's and the week's fills (decision 113's call). Not chosen: the towns' stream.
116. **T-16, a non-returning leave's points over the cleared army** (recommended):
     `TWN-QUIRK-12`, `fix-optional`, on in `faithful` (off: the land maximum of its one
     creature). Not chosen: `preserve` without a flag; fixed without a flag.
117. **T-17, the Thieves' Guild's artifacts row by the backpacks alone** (survey `E09-tavern.md`
     §13): `TWN-QUIRK-13`, `fix-optional`, on in `faithful` (off: the worn and the backpack's
     artifacts, VCMI's). Not chosen: `preserve` without a flag (the agent's recommendation:
     information only); fixed without a flag.
118. **TD-22 and TD-23: the reading for both** (recommended): the dismissal refused for a town's
     visitor; the guild's best hero by army power, the towns' shown armies in the army row and the
     best creature. Not chosen: VCMI's for both; TD-22 the reading and TD-23 VCMI's.
119. **The views** (recommended; design §73, §75): a player's two tavern heroes always in its own
     observation, every change told (`TavernOffered`); the Thieves' Guild a query answered in the
     viewer's turn by its taverns (or at a map Tavern); a Den of Thieves' view an event. Not
     chosen: the tavern heroes only while the player owns a Tavern or stands at a map Tavern; the
     guild in every observation, told by events.
120. **The env** (recommended; design §76): `HIRE` + 2 (v12) and the tavern decision's answers by
     role (v13); no entry for `Dismiss` (a management command, E08 decision 99). Not chosen: a
     `Dismiss` entry per hero slot.
121. **The cut and the exit criteria as written** (recommended; design §79, §80): T34 to T41 in
     seven rounds, each task's own cut at its start; the twelve criteria below. Not chosen: the
     builds before batch TWN-4.
122. *Agent's calls (change freely):* the places' order within a tie by player id; the guild in
     the viewer's turn only and at level 0 at a map Tavern without a town tavern; `TavernHeroView`
     without points or returning mark; `HeroHired` carrying the hero's type, owner and tile to every
     viewer of the tile; the Den an entered object; the personality and the obelisks row as stubs
     (`TWN-STUB-18`, `-17`).

### Decisions at T34's start (2026-11-16, the user)

123. **T34 whole** (the user at its start, `AskUserQuestion`): the writer's Tavern and Den, the
     specs, the harness's reads and steps, the recipes, the runs and the scoring in one session
     and one pull request. Not chosen: three parts as E08's T119 to T121 (recommended: the map and
     specs; the harness and the runs; the scoring), two by subject, two by stage. The `cross-lane`
     changes of `h3-map`'s writer and `h3-experiment` consented (recommended).
124. *Agent's calls (change freely):* three maps (4a the offers over eight weeks, the hires and the
     Recruit button; 4b the leaves and the dismissal over days 6 to 8; 4c the Thieves' Guild); the
     taverns' draws logged by xdd.dll with their ranges (row TWN-4-8 settled by each class draw's
     total, not by chance); the narrowed pool written by the harness in place of a map's allowed
     heroes (the writer's `Map` holds none; 46 constructors); spell points written before the
     leaves (kept against refilled) and the resources before the guild's reads; a fresh agent's
     review of the specs skipped (the runs checked the setups).

### Decisions at T35's start (2026-11-16, the user; TWN-Q19)

125. **T35 in two** (recommended; `AskUserQuestion`): **T35a** the docs (the four drafts into
     ADR-0013, -0016, -0017 and -0028 with batch TWN-4's verdicts and decisions 126 and 127, the
     glossary's terms, `docs/06-security.md`'s section; no code, no version moved), **T35b** the
     code (`StreamId::TAVERNS`, the flags of `TWN-QUIRK-12` and `-13`, the profiles' ruleset hashes (no
     version: asked as "v54", corrected after, as E08 T115b's flag), the packs' profiles, the golden replays' `ruleset_hash` and the bot reports),
     before T37a. Not chosen: the code first; T35 whole (as T24).
126. **The returning marks cleared at every day's start** (recommended; survey `E09-tavern.md` §14,
     the original's `new_day` 0x4C80CF to 0x4C80FD): after the taverns' week when one runs, every
     player's two marks, beside its turn's end and a hire (design §68); TD-18 stands (the week's
     release comes first). Not chosen: the turn's end and the hire only (as drafted; not the
     original's).
127. **Every record's day's spell points** (recommended; survey §14, 0x4C8237): a tavern hero and a
     hero out of the game regain their day's spell points with every record of the store
     (HER-MAN-002 as built, ADR-0013 point 16.3), so TD-20's spell-point half (decision 104) is the
     original's and VCMI's; its movement half stands. Not chosen: the spell points set only at a
     draw and a leave (decision 104 as taken; not the original's).
128. *Agent's calls (change freely):* ADR-0017 point 9 and ADR-0028 point 9 without the adventure
     state hash (the drafts named it; design §77's table: the kinds hold no state); the marks'
     clear after the taverns' week (only the release reads the marks at a day's start); short
     Consequences and Alternatives lines in each of the four ADRs from the not-chosen options of
     decisions 103 to 127; each draft sentence moved verbatim but where a batch row is cited, a
     decision changes it or an anchor moved.

### Decisions at T35b's and T36's start (2026-11-16, the user)

129. **T35b whole** (recommended; `AskUserQuestion`): one `shared/` pull request, `cross-lane`
     consented (the battle lane's harvester and packs, the adventure lane's golden replays, the AI
     lane's bot reports, the server lane's test literal), as T24. Not chosen: the code and the flags
     first, the re-record second.
130. **T36 in the same session, whole** (the user; the agent recommended stopping after T35b, then
     T36 whole): one pull request after T35b's, `cross-lane` consented (the battle lane's harvester,
     the adventure lane's replays' `town_ruleset_hash`). Not chosen: the record and the parameter
     first with hand-written packs, the harvest second.
131. *Agent's calls (change freely):* the flags' names `quirk.town.leave_points_over_cleared_army`
     and `quirk.town.guild_artifacts_by_backpacks`; both ledger entries made with the flags (their
     regression tests with T38b and T39b).
132. *Agent's calls at T36 (change freely; design §60's As built):* the AB pack's overrides give
     its two classes' weights to the base records (the base pack cannot name them), a patch's
     weights changing or adding the classes it lists; the harvest from `H3bitmap.lod`'s
     `HCTRAITS.TXT` (`H3ab_bmp.lod`'s byte-identical); the price in the gap-fill, as
     `most_heroes`, not read from 0x67814C; an empty price compiles (a free hire, as a building's
     free cost); TWN-TAV-001's catalog entry left to T37a, which builds the draw.

### Decisions at T37a's start (2026-11-17, the user; TWN-Q20)

133. **T37a whole** (the user; `AskUserQuestion`; the agent recommended two parts, the hidden
     state and its rules first, the views, the events and the protocol second): one `cross-lane`
     pull request. Not chosen: two parts; three (the start's fill, the week and the turn, the
     views).
134. **The tavern view is the held hero screen's** (the user, after the agent's check; survey §2.1,
     `tavern_dialog_proc` 0x5D7D7D opening `hero_screen_show` as a held popup, 0x4E1969, the manual
     and VCMI's `CTavernWindow` agreeing): `GameObservation::tavern`'s `TavernHeroView` holds the id,
     the type, the level, the experience, the primary skills, the spell points, the skills, the worn
     artifacts, **the backpack's first five places and its count**, the army, the formation and the
     tactics flag; no learned spells (the spell book is a screen of its own), no offered levels.
     Design §75's `details: Arc<HeroDetails>` amended. Not chosen: the screen's parts with the whole
     backpack (recommended: simpler, a small leak); the whole details (§75 as drafted).
135. **`TavernOffered` a head and one part event** (recommended; `GameEvent` stays `Copy`, design
     §28's call): `TavernOffered { player, slot, hero: Option<TavernHero> }` with the view's parts
     without a list, then `TavernPartSeen { player, slot, part }` per stack, skill and artifact
     (worn by position, then the backpack's shown places) in that order; the fold exact, the
     spell points under `SRV-STUB-6`. Not chosen: a typed event per kind; the head alone with a new
     `SRV-STUB` gap (against exit criterion 10).
136. *Agent's calls at T37a (change freely; design §61 to §63, §68 and §75's As built):* a slot
     names its hero's type (its pool entry the record); a new record's away record has no points
     until its owner's turn start, the loose formation and the tactics flag on; a type with no
     valid starting army gets an empty one; the fill's steps in the original's order (the backpack,
     the spell points, then the army: the same draws as §63's text); the week's fill after the
     towns' growth and before the adventure's week step (the neutral garrisons and the Portals stay
     after it, as built); the start's fill after the build's state is made, before the first
     group's turn; the week tells a slot whose view changed; the five shown backpack places a
     constant (`TAVERN_BACKPACK_SHOWN`).

### Decisions at T37b's start (2026-11-17, the user; TWN-Q21)

137. **Every hero of a customised type takes its entry** (recommended; the code reading:
     `map_custom_heroes_read` 0x4C2CE0 fills the types' setup records before the objects are
     read, `load_hero_body` writes a body's set fields over its type's record, a random hero's
     by its drawn type, 0x502374 to 0x5023E7, and `placed_heroes_setup` 0x4CAE10 sets up all 156
     types' heroes from them before `main_town_heroes` and the taverns): a placed hero (its body
     over the entry, as built), **a random hero** (its body over its drawn type's entry), **a main
     town's hero** (the entry) and a tavern hero. VCMI 1.7.5 agrees for the main towns
     (`placeStartingHero` takes the map's predefined hero) and the taverns (`initHeroes`), not for
     a random hero (a new instance, `CGHeroInstance::pickRandomObject`): `TD-24`, the original
     followed. `Marshland Menace`'s player 5 (Fortress only) starts its main town's hero and its
     random hero at 2 000 experience whenever it draws one of the 15 customised types. Not chosen:
     the tavern records only (design §70 as drafted), the rest a stub.
138. **A customised type's record at the build** (recommended; amends decision 109 for these
     types): after the build's heroes, one record per customised type still free, in type order,
     its default experience, level seed and levels at load in the build's passes (hero id order,
     the setup stream); a draw finds the record made. The original's timing (every type's hero set
     up before the first fill); no new state, the game state hash's encoding unchanged; the order
     T37a left open is the build's (the seed before the levels, which `faithful` reads). Not
     chosen: the record at its first draw (§70 as drafted: the entries in the state, game state
     hash v34, the seed then the levels from the taverns' stream).
139. **T37b whole** (recommended): one `cross-lane` pull request (map import v25, `GameSetup`'s
     `custom_types` and its checks, the build, the goldens' import line). Not chosen: the import
     and the setup first, the build second.
140. **T38a cut in two** (the user, `TWN-Q22`; recommended): **T38a1** the hire's rule end to end
     (`TownCommand::Hire` and its refusals, the adventure's placement, the replacement, `HeroHired`,
     the protocol v71) and, as the agent found at the build, the replay word `town hire` (replay
     format v17: the server journals every command the wire takes, so a v16 journal could hold a
     line no v16 reader parses; the 89 goldens' format line only); **T38a2** the hire listed (the
     legal commands, the env's `HIRE` v12, the plans, the bench; `TWN-STUB-19` until then). Not
     chosen: T38a whole (T28a's size, more lost to an interruption).
141. **A hire tells its owner what the slot's view lacked** (the user, `TWN-Q22`; recommended; the
     T37a As built's note for T38a): after `HeroHired`, to the owner alone, `HiredPartSeen` per
     part (`HiredPart`: first the levels last offered with the worn positions' count, then the
     backpack past its first five, then the learned spells), so that the fold of the slot's view
     is the hero's details whole. ADR-0017 point 2's, ADR-0016 point 6's and `06-security.md`'s
     sentence on the owner's view gain it in the session's `shared/` pull request. Not chosen: the
     details told whole as parts (more events a hire, the slot's view unused), a fold gap as a
     stub.
142. *Agent's calls at T38a1 (change freely; design §64, §65, §75's As built):* the hire's refusals
     in §64's order with `NoTavern` after the owner and `UnknownSlot`, `EmptySlot` as the town's
     (not the game's) refusals; the room before the entrance (§64's order; `Leave`'s is the other,
     the original's button tests both at once); the adventure's half as `check_tavern_hire` and
     `place_hired` beside the rows (`src/town/hire.rs`), reusing a hero's arrival; the slot named
     in the event as `Option<u8>` (the projection blanks another viewer's); the parts only for a
     sink that keeps events; a new record's draws shared with the fill (`tavern::record_of`); the
     replacement's war machines none (the one-creature form gives only the first stack).
143. **T38a2 whole** (the user; the agent recommended a cut in two: the listing with the plans
     and every oracle taught a hire, the random agent skipping the listed hires as decision 56 did
     for the builds; then the env's entries with the seeds and the workloads): one `cross-lane`
     pull request, the listing, the plans, the env's v12, the seeds and the benches together.
144. *Agent's calls at T38a2 (change freely; design §64, §76's As built):* a town's hires listed
     after its recruitments and before its hero moves (§64's "after its recruitments", §76's
     "after a town's builds and recruitments"); the listing reuses the hire's checks
     (`town::hire::check`, one test of the Tavern a town before them); the env's entries named
     `TAVERN_HIRE + s` (`HIRE_BASE` already names the hire dialog's purchases), their mask and
     decoding as the hero moves' between the bars; the plans' `Step::Town` hires a listed hero in
     one step of eight; the hygiene's comparison ends at a hire the viewer is told (the hired hero
     is a draw of the taverns' stream the hidden differences move).
145. **T38b cut in two** (the user, `TWN-Q23`; recommended): **T38b1** every leave by a battle
     or a capture (the away record handed back by the adventure's end, the non-returning leave's
     one creature, spell points and T-16's points, the returns after a retreat or a surrender,
     `TavernOffered` after the battle's end), **T38b2** the dismissal (`Manage::Dismiss` with its
     refusals, TD-22, the legal commands, `manage dismiss`, the protocol, the plans); over T38b
     whole and over three (the leaves without return, the returns, the dismissal).
146. *Agent's calls at T38b1 (change freely; design §67's As built):* the leave runs last in the
     battle's end, after the winner's levels (the original's `hero_leave_game` 0x4AE6D5 after its
     levels 0x4AE324), the return told then; the return's slot `range_inclusive(0, 1)` from the
     owner's stream; a surrender whose survivors are none (its war machines or created stacks
     alone paid) returns with one creature, as a retreat (the original's empty army not kept: a
     hire never places a hero without a stack); a hero that is not its type's record (a setup
     that repeats a type; no shipped map does) becomes the record as it leaves, unless the type
     is offered (then the pool is left as it is); a new record's replacement takes the land points
     of its one creature (design §67 point 5, built here); the adventure's `end_battle` returns
     the heroes that left (`Vec<HeroOnMap>`), `release_town_hero` the hero (`HeroOnMap`).
147. **T38b2 kept whole** (the user, `TWN-Q24`; the agent recommended a cut in two as T38a's,
     decision 140: T38b2a the command, its refusals, the leave, the replay word and the protocol;
     T38b2b the listing, the plans and the bench).
148. *Agent's calls at T38b2 (change freely; design §69's As built):* the replay word `dismiss
     <hero>` (format v18), as the format's other management words, not `manage dismiss`; the
     refusals in the order `NotOnMap`, `InTown`, `LastHero`, then `NotYourHero` by the
     adventure's removal (`remove_hero` checks the owner); a dismissal inside a meeting, an army
     dialog or an exchange `DecisionOpen`; `InTown` for any town's entrance (the original tests
     the tile); the plans dismiss in one `Step::Manage` of sixteen, `managing` without the
     dismissals, so that the other steps play as before.

### Decisions at T39a's start (2026-11-18, the user; TWN-Q25)

149. **T39a kept whole as queued** (the user, `TWN-Q25`; the agent recommended the Tavern alone
     with the Den of Thieves' kind moved to T39b, whose visit is the Thieves' Guild): both kinds,
     the tavern decision, the env's v13, the replay word v19 and the protocol v73 in one
     `cross-lane` pull request (the user's consent), the Den's visit a stub until T39b
     (`TWN-STUB-20`); the game state hash v34 with it (the decision's hash kind 9, which design
     §77's table missed).
150. *Agent's calls at T39a (change freely; design §66's and §74's As built):* the kinds' rule ids
     ADV-VIS-007 and ADV-VIS-008 (the adventure's visit family), a pack's Tavern entered or Den
     visited from a neighbour refused; the visit's `h3_adventure::TavernVisit` (the hero, the
     Tavern's type and anchor, its visitable tile), held whole by the decision; the decision opens
     whatever the gold and the room (the original's dialog opens with its Recruit button
     disabled), its listing then `Close` alone; `Answer::Tavern(TavernAnswer::{Hire { slot },
     Close})`, the kind's own as the hire dialog's; the refusals a new
     `GameCommandError::Tavern(TavernError::{UnknownSlot, EmptySlot, TooManyHeroes, Occupied})`
     in that order, then `CannotPay` (the town's order without the town); a refused hire leaves
     the decision open, a hire closes it after its events; `DecisionView::Tavern` and
     `TavernOpened` the hero, the type and the anchor (the fold takes the view whole); the env's
     v13 the hire of slot 0 the first role, of slot 1 the second, `Close` the closing; the replay
     words `answer tavern hire <slot>` and `answer tavern close`; the plans name the kind in
     `settle` and `hiring` only (their maps lay out no Tavern; `check_legal` is T40's when a plan
     lays one out).

### Decisions at T39b's start (2026-11-19, the user; TWN-Q26)

151. **T39b cut in two** (recommended; `AskUserQuestion`): **T39b1** the Thieves' Guild as a query
     (TWN-TAV-007 to -009, TD-23, `TWN-QUIRK-13`'s test, `TWN-STUB-17`, `-18`), **T39b2** the Den
     of Thieves' visit (the adventure's outcome, `ThievesGuildSeen`, `TWN-STUB-20` lifted) with
     the whole protocol (v74: the guild's request and the events, the fold), each its own pull
     request, one a session; `cross-lane` consented for both (the adventure lane's power, the
     day's gold, the visit's outcome; the server lane's protocol). Not chosen: the request with
     T39b1 (v74, then v75 for the events); whole as queued.
152. **The guild's resource rows a town-part parameter** (the user, against the agent's
     recommendation): `[town_params]`'s `guild_resources`, three lists of core resources (part
     v8; the harvester's gap-fill gives `RESTYPES.TXT`'s indexes, `cross-lane`), the goldens'
     `town_ruleset_hash` moved. Not chosen: the rows by the core's resource order (the original's
     indexes in code, no version), the agent's recommendation.
153. *Agent's calls at T39b1 (change freely; design §73's As built):* a row's places by player
     id (each player's place, dense); `thieves_guild_at` without the viewer (the view is the same
     for every viewer of a level); the parts `GuildHero { hero_type, skills }` and `GuildPlayer
     { hero, creature }`; the best creature from 0 as the best hero (read at T39b1), a stack of
     count 0 skipped; the towns in `TownId` order (the original's town list's); sums saturating
     in u64; `TWN-QUIRK-13` off counting every worn position's artifact; the tavern count to 5 a
     sibling of the popup's; the income without the week's resources; `h3_heroes::primary_skills`
     beside `attack_defense`; the tests on the hire tests' game.

### Decisions at T39b2's start (2026-11-19, the user; TWN-Q27)

154. **T39b2 kept whole** (recommended; `AskUserQuestion`): the Den's visit, `ThievesGuildSeen`
     with its parts, the census and the whole protocol v74 (the events, their fold, the guild's
     request) in one `cross-lane` pull request (decision 151's consent). Not chosen: a cut at the
     request (the visit and the events with their wire and fold, v74; the request v75 in a next
     session). A new game event is mapped on the wire in its own pull request (the global trap),
     so the events could not leave the protocol.
155. *Agent's calls at T39b2 (change freely; design §74's and §75's As built):* the adventure's
     outcome a flag (`AdventureOutcome::thieves_guild`), told by the game after the visit's other
     effects (`settle_outcome`, `town::thieves::tell_den`); the guild told as `ThievesGuildSeen {
     player, level }` then `GuildPartSeen { player, part }` per `GuildPart` (`Place { row, of,
     place }` per shown row's player, the rows in order, then `Player { of, shown }` per player;
     `ThievesGuildView::parts`), nothing told or computed for a sink that keeps nothing
     (`Sink::RECORDS`); the engine has no fold of the parts (the wire's `ThievesGuild::see`, a
     client's); on the wire `GameQuery::ThievesGuild` (no field: the asking seat's player) and
     `GameAnswer::ThievesGuild { guild: Option<ThievesGuild> }`, the wire's `GuildRow`,
     `GuildRanking`, `GuildHero`, `GuildPlayer`, `GuildPart`; the fold drops both events; the
     ADR's wording by a `shared/` pull request (#1039).

### Exit criteria (accepted at `TWN-Q18`, decision 121; design note §80)

1. The town part's tavern weights and the hire's price compile (part v7); each player's faction
   and two slots; the pool's `Offered`, records made at a first draw and kept by type, the away
   records (TWN-SET-001, -005).
2. The draw by the weights from the taverns' stream; the start's and every week's fill (full army,
   backpack equipped, spell points full); the week's release sparing a returning offer
   (TWN-TAV-001, -002; GAM-CAL-002; TD-18).
3. The town's and the map Tavern's hire, every refusal typed and changing nothing; the
   replacement from the hirer's stream with one creature (TWN-TAV-003, -004; GAM-DEC-010).
4. A hero that leaves the game keeps its record; a retreat or a surrender returns it at once
   (TD-19); every other leave frees the type under `TWN-QUIRK-12` (TWN-TAV-006; GAM-BAT-003).
5. The tavern heroes' points at the turn start, their spell points regained each day as every
   record's (decision 127), the marks cleared at the turn's end and at every day's start (decision
   126) (TWN-TAV-005; TD-20).
6. The dismissal with its refusals (TD-22), its hero freed with every artifact (GAM-MAN-003).
7. The map's customised types reach a record made at a draw (GAM-HER-001).
8. The Thieves' Guild by the viewer's taverns and a Den's everything: the rows as places, the best
   hero, its skills, the best creature (TD-23), the artifacts row under `TWN-QUIRK-13`;
   `TWN-STUB-17`, `-18` (TWN-TAV-007 to -009).
9. Classes 95 and 97 import and play.
10. The views and their events; the fold gate without a new gap; the protocol.
11. The env plays the hire and the tavern decision (v12, v13); the replays record the hire, the
    tavern answers and the dismissal.
12. Every new rule has its id, source and tests; `TWN-QUIRK-12`, `-13` recorded; the gates pass;
    batch TWN-4 run and scored, its consensus rows `verified`.

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, each task's cut confirmed at its start)

- **T30** the audit of increment 3's exit criteria, its closure, this refinement. Done 2026-11-15 (decisions 98–102; `E09-log.md`).
- **T31** the garrison moves below level 3 (decision 101). Done 2026-11-15 (the projection; `E09-log.md`).
- **T32** the survey (decision 100), with its checkpoint (`TWN-Q16`). Done 2026-11-15 (decisions 103–107; `E09-log.md`).
- **T33** the ADR amendment drafts, the design note's Part F, the cut and the exit criteria (decision 100), in two (decision 108): **T33a** the hire's: done 2026-11-16 (`TWN-Q17`, decisions 109–113; the drafts `adr-drafts/*-increment-4.md`, Part F §59 to §71; `E09-log.md`). **T33b** the information's, the cut, the exit criteria and the checkpoint: done 2026-11-16 (whole, decision 114; `TWN-Q18`, decisions 115–122; the drafts accepted; Part F §72 to §81; survey §13; `E09-log.md`).
- **T34** batch TWN-4 (round 2): done 2026-11-16 (whole, decisions 123, 124; specs
  `experiments/specs/E09-batch-4a.toml` to `-4c.toml`, two runs each in `experiments/results/`;
  every row as the reading, survey §14; two points the reading missed, the new day's clear of the
  returning mark and the day's spell points of every hero record, for T35's checkpoint; the writer's
  classes 95 and 97, `cross-lane`; `E09-log.md`).
- **T35** `shared/` (round 3), in two (decision 125): **T35a** the drafts into the four ADRs, the
  glossary, `docs/06-security.md`: done 2026-11-16 (`TWN-Q19`, decisions 125–128: the marks cleared
  at every day's start, every record's day's spell points; the drafts deleted; `E09-log.md`).
  **T35b** `StreamId::TAVERNS`, the two quirk flags, the packs and the replays' hashes: done
  2026-11-17 (whole, decisions 129, 131; the base faithful hash 0x6ba8109889b34a25; `E09-log.md`). **T36** the town part v7 (round 4): done 2026-11-17 (whole, decisions 130, 132; `E09-log.md`). **T37** T37a the pool, the draw and the fills, the turn's points and marks, the views and the protocol: done 2026-11-17 (whole, decisions 133–136; game state hash v33, protocol v70; `E09-log.md`); T37b the map's customised types: done 2026-11-17 (whole, decisions 137–139; map import v25; `E09-log.md`). **T38** T38a the town's hire and the replacement, cut in two at its start (decision 140): T38a1 the hire's rule, the replacement, `HeroHired` and its parts, the protocol and the replay word: done 2026-11-17 (decisions 140–142; protocol v71, replay format v17; `E09-log.md`); T38a2 the hire listed (the legal commands, the env's `TAVERN_HIRE` v12, the plans, the town-map seeds, the benches): done 2026-11-18 (whole, decisions 143, 144; action encoding v12; `E09-log.md`); T38b cut in two at its start (decision 145): T38b1 the leaves and the returns (every leave by a battle or a capture, TWN-TAV-006, TD-18, TD-19, T-16): done 2026-11-18 (decision 146; `E09-log.md`); T38b2 the dismissal (`Manage::Dismiss`, its refusals, the listing, the plans): done 2026-11-18 (whole, decisions 147, 148; replay format v18, protocol v72; `E09-log.md`). **T39** T39a the map's Tavern and the Den of Thieves with the tavern decision: done 2026-11-18 (whole, decisions 149, 150; map import v26, the adventure part v22, game state hash v34, replay format v19, action encoding v13, protocol v73; `E09-log.md`); T39b cut in two at its start (decision 151): T39b1 the Thieves' Guild as a query (TWN-TAV-007 to -009, TD-23, `TWN-QUIRK-13`, `TWN-STUB-17`, `-18`; its resource rows the town part's, decision 152): done 2026-11-19 (decisions 151–153; town part v8; `E09-log.md`); T39b2 the Den's visit and the protocol: done 2026-11-19 (whole, decisions 154, 155; `TWN-STUB-20` lifted; protocol v74; `E09-log.md`); T39 done. **T40** the gates, **T41** the audit and increment 5's refinement (round 8). Design note §79.

## Risks

- **Two lanes in the same crates:** the town modules sit in the adventure lane's crates, and their common parts, versions and replays change in both lanes; merges in order, a rebase and a second re-recording when both bump. Check the adventure lane's open pull requests at the start of every task.
- **The size of the content:** nine factions of about forty buildings each, with requirements read from the executable; the harvest is in the battle lane's crate.
- **Editions:** the moat (ADR-0023), Conflux and the AB buildings, RoE's differences (E11).
- **The faction choice touches E10's scenario setup** (decision 6): the setup's faction per player is the seam; the rest of the scenario setup stays E10's.
- **Other lanes read the town:** the env (AI lane), the map client (E24 waits for towns), the server's wire for new commands and events; notes to their inboxes when a town change lands.
