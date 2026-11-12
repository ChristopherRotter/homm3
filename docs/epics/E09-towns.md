# E09 — Towns

Status: **in progress** (refined 2026-10-31 with the user, TWN-Q1; the town lane opened the same day). Increment 1 (towns on the map) closed on 2026-11-06 (T10; its scope, decisions 8–35 and tasks in `E09-archive.md`), increment 2 (construction with the town events) on 2026-11-11 (T20; decisions 36–63, `E09-archive.md`); increment 3 (growth, recruitment and the army rows) refined the same day (TWN-Q11), its survey done (T21, TWN-Q12), its drafts accepted (T22a, T22b; TWN-Q13, TWN-Q14), batch TWN-3 run and scored (T23), its `shared/` task done (T24: the ADRs, the core changes), T25a done (the part v5, the store v3, TWN-GRO-001 and -005), T25b1 done (the week and the month, TWN-GRO-003, -004), T25b2 done (the garrison week and the Portal, TWN-GRO-006, -007), T26 next; the position and the next task are in `docs/lanes/town/HANDOFF.md` and `QUEUE.md`. The record of every done task: `E09-log.md`.

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
3. **Growth, recruitment and the army rows** (refined below, decision 65): the weekly growth (the dwellings, the castle's bonus, the hordes, the week's creature and the Plague, the town events' creatures), recruitment into the garrison or the visiting hero, the town's two army rows and the heroes' swap.
4. **The Tavern** (the sketch's increment 3 in part until decision 65): the Tavern's hire with its pool and the eight heroes' limit, the Tavern's information and the Thieves' Guild.
5. **The mage guild and the services** (the sketch's increment 4 until decision 65): the guild's spells drawn (the town chances, the map's required and banned spells), learning them (Wisdom), the spell points restored; the Marketplace and the Resource Silo, the Blacksmith, the town's Shipyard (E08's `BuildBoat`), the Artifact Merchants and the other faction services; the special buildings for heroes and sieges (the Castle's Lighthouse, A-30; the Necropolis's Necromancy amplifier, the queue archive's T19a2b note; Cover of Darkness).
6. **Travel and the Grail** (the sketch's increment 5 until decision 65): Town Portal and Castle Gate; the obelisks, the puzzle map, digging and the Grail buildings.

Citations of "increment 4" and "increment 5" written before decision 65 in the log, the archive, the surveys and the design note's Parts A to D mean today's 5 and 6 (decision 67).

## Closed increments

Their scope, decisions and task lists are in `E09-archive.md` (cited as "E09 decision N"; `git grep -n 'decision 22' docs/epics/E09-archive.md`); increments 1's and 2's design notes stay in `E09-design.md` (decision 40; Parts A to D), the record of every task in `E09-log.md`. Decisions 1–7 above and those from 64 below stay here.

| Increment | Closed | Tasks | Decisions | Content |
|---|---|---|---|---|
| 1 | 2026-11-06 | T0–T10 | 8–35 | towns on the map: fixed and random towns imported and built, the players' factions and the starting-hero pick, the hero pool with the main towns' and the random heroes, the town's record and store, entering, capture and the siege with its facts from the town and the garrison's merge, the day's income with the silo, the views by the original's levels, batch TWN-1; all 160 shipped maps import and build |
| 2 | 2026-11-11 | T11–T20 | 36–63 | construction with the town events: the random dwellings tied to a town, the build command (`TownCommand::Build`: the owner in its turn, the hall's offer by the executable's closed requirement lists, the forbidden, one Capitol a player, the hordes as their upgrades, one a town a day, `Building.txt`'s costs), what a build changes of increment 1's reads, the Shipyard's water, the map's town events (their days, players and kinds, resources and buildings; their creatures `TWN-STUB-10`), the views and every viewer's changed view told, the env's build entry, the builds map and its replays, batch TWN-2 (TD-9, TD-11); all 160 shipped maps play their town events |

## Increment 3 — growth, recruitment and the army rows

Refined with the user on 2026-11-11 (T20; `TWN-Q11`, asked at T20's start with a first look at the audit). Built like increments 1 and 2: commands, validation, events, observation, the env, replays and the gates grow with the content. The audit of increment 2's exit criteria is in `E09-log.md` (T20).

### Decisions at the refinement (2026-11-11, the user; TWN-Q11)

64. **T20 whole** (the user at T20's start, `AskUserQuestion`; recommended): the audit, the closure and this refinement in one session and one docs-only pull request, as T10 (decision 36); a gap needing code queued as a small task. Not chosen: T20a (the audit and the closure), then T20b (the refinement).
65. **Content: growth, recruitment and the army rows** (recommended). **The weekly growth** of a town's dwellings: each dwelling's creatures a week, the castle's bonus by the fort's level, the hordes, the week's creature (drawn from a stream of its own, ADR-0017 point 4; E08 decision 197) and the month's kinds with the Plague, the owned dwellings on the map where the original's growth reads them (the survey; the inbox's E08 T94a, T94b, T97a notes), the town events' creatures (`TWN-STUB-10`: 59 firings on the shipped maps in their first days, T19c2's census); a dwelling's first creatures at its build (ADR-0017's increment 2 note). **Recruitment** at a town's dwellings into the garrison or the visiting hero through E08's hire dialog, reused (`HireSource` with a town's dwelling, `HireTarget` with the garrison; the inbox's E08 T95b note; the env's four purchases). **The town's two army rows:** the garrison as an army of E08's army commands (`ArmyCommandError::Garrison` lifted), a garrison hero and the heroes' swap between the visitor's row and the garrison's, and what a garrison hero changes of increments 1's and 2's reads (the siege's defender, TWN-SIE-001; the views' marks, TWN-VEW-001). The env plays the new commands (decision 7). **The Tavern becomes increment 4** of the sketch (its hire with the pool's offer and the eight heroes' limit, the hero classes' tavern weights, a defeated player's heroes, `ADV-STUB-9`'s rehire with E10, `Marshland Menace`'s tavern experience, the Tavern's information and the Thieves' Guild); the mage guild and the services become increment 5, travel and the Grail 6. Not chosen: as sketched (the Tavern's hire, its information and the Thieves' Guild in increment 3 too); growth, recruitment, the army rows and the Tavern's hire, with its information and the Thieves' Guild later.
66. **The first round** (recommended): a survey (T21: the code reading of `Heroes3.exe` for the week's growth, a dwelling's recruitment and the town screen's two rows and swap, VCMI and the wiki compared, a census of the shipped maps) and its checkpoint, then the ADR amendment drafts with the design note's Part E and the cut (T22), accepted before anything is built, as decisions 7 and 38. Not chosen: the survey alone first.
67. *Agent's calls (change freely):* **the design note's Part E** in `E09-design.md`, its sections numbered from §37, as decision 40. **The renumbering:** the live citations of the sketch's increments (the rules, the stubs, the town modules' comments, the queue; ADR-0016, -0017, -0028 and `docs/06-security.md` in `shared/e09-t20-increments`) renumbered by T20; the log, the archive, the surveys and the design note's Parts A to D keep their words, as E08's sketch keeps its trail. **The army rows with E08's increment 12:** its garrisons (classes 33, 219) exchange armies with a hero of their owner (E08 decision 295), as a town's garrison will: T21 reads the original's town screen and the adventure lane's T108a its garrison dialog, and the two lanes agree one army reference through their inboxes before either drafts (E08 T108b, T22); built once, by whichever lane builds first, as decision 4's hire dialog (the adventure lane's inbox told). **The inbox's 2026-11-11 note kept as built:** a siege won by a player who holds another Capitol tells the old owner the town with its Capitol before the limit demotes it (each event tells the town as it stood; the plans' check compares each viewer's last `TownChanged`).

68. **T21 whole** (the user at T21's start, `AskUserQuestion`; the agent recommended two parts, growth first, then recruitment and the rows): the survey in one session and one docs-only pull request. Not chosen: two parts; three (growth, recruitment, the rows).
69. **TD-13, a creature's month** (`TWN-Q12`; the user's own answer, "fix-optional flag"; survey `E09-growth.md` §3, §16): a quirk (candidate T-13), class `fix-optional`: faithful, only the index whose creature is the month's doubles (the original's code, the wiki, VCMI's forum); with the flag, the level whichever of its creatures is the month's (VCMI 1.7.5's code). Registered by its building task.
70. **TD-14 and T-12, the Plague in a neutral town** (`TWN-Q12`; the user's own answer, "fix-optional"): class `fix-optional`: faithful as read (the full growth taken off a count that got half of it); with the flag, every town keeps half of its count before the week.
71. **T-11, the Portal of Summoning's first creature** (`TWN-Q12`, recommended): drawn when the Portal is built and when its owner flags a dwelling while it has none, besides each week; the original's draw at the town screen is not modelled. Not chosen: only each week (the computer's path).
72. **TD-15, a neutral town's garrison week** (`TWN-Q12`, recommended): the reading (survey §4). Not chosen: VCMI's approximation.
73. *Agent's calls (change freely):* TD-16 (the Portal: a dwelling, then one of its creatures; the count set; no Plague) and TD-17 (an ally's visitor moves no hero, E10) as read; T-10 fixed by proposal (a town's counts saturate where the original's 16 bits wrap); the army reference (survey §15: the town's upper bar `ArmyRef::Garrison` or the garrison hero's, a map garrison a sibling reference, one owner rule) proposed in the adventure lane's inbox, settled in T22's and T108b's drafts; batch TWN-3's six rows (survey §14) for T22's cut.

74. **T22 in two halves, batch TWN-3 its own task** (`TWN-Q13`, the user at T22's start; recommended): **T22a** growth's drafts and the design note's Part E from §37 (docs only); **T22b** recruitment's and the two rows' (the army reference after the adventure lane's T108b answers T21's proposal), the building tasks' cut, the exit criteria and the checkpoint `TWN-Q14`; **T23** batch TWN-3 (survey §14), cut at its start, as increment 2's T13 and T14. Not chosen: T22 whole without the batch, the army reference settled with the user then; batch TWN-3 first.
75. **One towns' stream** (`TWN-Q13`; recommended): a new named kernel stream (`StreamId::TOWNS`, `shared/`) for every weekly town draw: the week's kind and creature, the neutral towns' garrison weeks, the Portals' weekly picks; the game's stream (the weekly objects, the month, the spawn) shifts nowhere. Not chosen: the week's creature alone on its own stream (E08 decision 197), the garrisons and the Portals on the game's in the original's order.
76. **The Legion's growth as hero effects** (`TWN-Q13`; recommended): `HeroEffect::TownGrowth` (the five pieces, by level) and `HeroEffect::OwnerTownGrowth` (the Statue, in basis points) in the core (pack format v52, `shared/`; the base and SoD packs' artifacts, `cross-lane`), derived by `h3-heroes` with each worn type once and a worn combination's parts counting (ADR-0013 point 16.4's draft). Not chosen: town part parameters naming the six artifacts.
77. **The week's kind in the game state** (`TWN-Q13`; recommended): beside the month's kind, `WeekStarted` carrying it and the observation showing it as the month's (`cross-lane`, with the protocol's mapping). Not chosen: the town store with a town event.
78. *Agent's calls of T22a (change freely; listed for `TWN-Q14`):* a horde's growth the core's `horde_growth` (the executable's horde table holds the same figures); `h3_towns::WeekKind` the town crate's type, its value the game's; the eligible week's creatures the town records' levels, an RoE map's only the records with `roe` (a setup flag from the import, map import v22); a `week_of` week drawn then overwritten, the month after it the level's base creature's without the creature's draw (`new_month` 0x4C8F63); the counts bounded at 32 767 (T-10); the external dwellings derived from the owners at each step, by a dwelling's first creature; the Portal drawn at a command's build from the builder's stream, at an event's from the towns' stream, at the setup from the setup stream, at a flagged dwelling from the visitor's, never at a capture (an open point); the week's, the garrisons' and the Portals' loops in `TownId` order; the owner's view showing the counts and the Portal; the quirks planned as `TWN-QUIRK-8` (T-10, fixed), `-9` (T-12), `-10` (T-13). Design note §37 to §47.

79. **T22b whole** (the user at T22b's start, `AskUserQuestion`; the agent recommended the design first,
    then the cut, the exit criteria and the checkpoint as T22b2): recruitment's and the rows' drafts,
    Part E from §48, the cut, the exit criteria and `TWN-Q14` in one session. The army reference was
    settled by the adventure lane's T108b before it (E08 decision 305, T21's proposal as written;
    the user at T22b's start: "decided by the other lane and merged"). Not chosen: the design, then
    the plan; recruitment, then the rest.
80. **The town's two rows open no decision** (`TWN-Q14` (1), recommended; design note §52): in its
    turn with no decision open, the owner's army commands name an own town's upper bar alone or
    with its visitor, by E08's one owner rule, as the original's town screen opens at will. Not
    chosen: E08's `Exchange` decision reused, opened by a town command and closed.
81. **The garrison hero's record moves whole into the town's** (`TWN-Q14` (2), recommended; §53):
    `Holds::Town`'s garrison hero, its army, points, marks and bonuses kept; the turn's, the day's
    and the week's steps, the income and the Legion's share reach it explicitly; not on the map.
    Not chosen: kept among the heroes on the map with a placement flag; leaving the map and
    dropping its on-map state.
82. **A capture's empty Portal is drawn** (`TWN-Q14` (3), recommended; §45, §51): at a capture that
    finds a standing Portal empty, from the capturer's stream after the capture's own draws (a
    human's capture opens the town screen, which draws at once). Not chosen: no draw at a capture.
83. **The cut, the exit criteria and the agent's calls as written** (`TWN-Q14` (4), recommended):
    the building tasks T23 to T30 in rounds 2 to 8, the rows after E08's T114a and T115a (§56); the
    eleven exit criteria below (§57); decision 78's calls and T22b's (§49 to §56): recruitment by
    building (`TownCommand::Recruit { town, building }`), the dialog opened at a count of 0 too, a
    Portal with nothing summoned refused, the town's refusals given as `HireError`'s, the hire's
    target the town's upper bar and never the visitor, `HireSeen` in the view and the event, the
    hero moves as `AdventureCommand::TownHeroes` (`Enter`, `Leave`, `Swap`), `GarrisonHeroChanged`,
    the own hero view's place, each env entry with its command's task (v10, v11), batch TWN-3's
    row 6 with the garrison hero's points, `TWN-QUIRK-11` for T-5.
84. **T23 kept whole** (the user at T23's start, asked; recommended was three parts: growth's rows,
    row 6, the scoring), its branch `cross-lane` (the battle lane's experiments README; the
    common capture harness). The week's and month's kinds forced as draws by xdd.dll, not written
    (survey §17: the globals are written by the draw and read straight after).
85. **T24 kept whole** (the user at T24's start, asked; recommended was two parts: the core code
    with ADR-0013, then the other ADRs, the glossary and the security lines). *Agent's calls*
    (change freely): `town_growth` collides in a list only at one level; the bound
    `TOWN_GROWTH_LEVELS` in `h3-content`; the two flags off in the synthetic packs' profiles (as
    the adventure's); the `synthetic-ext` Storm Regalia carries both kinds; ADR-0016's and
    ADR-0017's increment 3 texts placed after each point's increment 2 text (ADR-0016 point 3's
    garrison hero at the point's end, before E08's increment 12 text once T112 adds it).
86. **T25a kept whole** (the user at T25's start, asked; recommended was T25a in two: the part v5
    with its harvest first, then the store v3 with TWN-GRO-001 and -005). *Agent's calls* (change
    freely): `CREATURE_INDEXES` and `MAX_AVAILABLE` at the crate's root (design §40's `INDEXES`);
    the core passed to `Towns::build` and `town_setup` for the growths (one fact, held once, not
    copied into the part); the store's public `write(town, Town)`, its closure write renamed
    `update`; `GAME_STATE_HASH_VERSION` 28 taken now (the store's v3 moves every game hash), so
    T26a's week kind is v29; the harvest refuses a horde record whose growth is not its creature's
    `horde_growth`; the synthetic `vale`'s upgraded horde without a role (its levels have no
    upgrade), its Portal the Lookout's and its week's building the Grail's (no new building, so
    no synthetic id or legal command moves); the garrison week's weights one to six.
87. **T25b in two** (the user at T25b's start, asked; recommended): **T25b1** the week and the
    month (`WeekKind`, TWN-GRO-003, -004, `TWN-QUIRK-9`, `-10`), which draw nothing; **T25b2** a
    neutral town's garrison week and the Portal's draw (TWN-GRO-006, -007). Not chosen: T25b
    whole; T25b1 with the inbox's hygiene failure. *Agent's calls* (change freely): the month's
    kind as the towns read it the town crate's `MonthGrowth` (the game's `MonthKind` mapped to it
    by T26a); `WeekKind::names` and `bonus` public; the fixture's `core_with` (a profile's flags).
88. **T25b2 kept whole** (the user at T25b2's start, asked; recommended; not chosen: in two, the
    garrison week then the Portal; whole with the inbox's hygiene note). *Agent's calls* (change
    freely): `garrison_week` takes the army's slots and a stack's most from the game (`rules`,
    the adventure part's; the room ADV-ARM-007's, one army rule), returns the garrison in slot
    order and the town; a level the record lacks, a creature growing none and a growth above a
    stack's most add nothing (after the sums); `summon` checks the standing Portal itself (no
    draw without one) and empties the Portal without a dwelling (no draw); the fixture's AI values
    (ten times a handle) and `part_changed`.
89. **T26a in two** (the user at T26's start, asked; recommended): **T26a1** the towns' growth in
    the day with every week a normal one (each town's inputs with the Legion's two effects from
    `h3-heroes`, the week and the month in the day's order, the events' creatures with
    `TWN-STUB-10` lifted, the quirk flags' "planned" marks), no version moved; **T26a2** the week's
    kind (the towns' stream, the game state's week and hash v29, the draw with the setup's `roe`
    and map import v22 or the next free after E08's T113b, `WeekStarted`'s kind, the observation
    and the protocol, the month after a `week_of` week, the inbox's T97a note). Not chosen: T26a
    whole; the week's kind first (it races T113b for map import v22). *Agent's calls* (change
    freely): `h3_heroes::town_growth` and `owner_town_growth` beside the day's income in
    `day.rs`; the game's `town/growth.rs` with `inputs` (once a week, kept for the month of the
    same day), `grow_week`, `grow_month`; the month's block inside the week's in `advance_day`
    (a month's first day is a week's); a base dwelling counts at its upgrade's index only where
    the level has an upgrade; an event's creatures held at the counts' bound (`TWN-QUIRK-8`) and
    told by no event until T26b's views.

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, each task's cut confirmed at its start)

- **T20** the audit of increment 2's exit criteria, its closure, this refinement. Done 2026-11-11 (decisions 64–67; `E09-log.md`).
- **T21** the survey (decision 66), kept whole (decision 68). Done 2026-11-11: `docs/rules/surveys/E09-growth.md`, the checkpoint `TWN-Q12` (decisions 69–73; `E09-log.md`).
- **T22** the ADR amendment drafts, the design note's Part E and the cut (decision 66), in two halves (decision 74): **T22a** growth's: done 2026-11-11 (`TWN-Q13`, decisions 74–78). **T22b** recruitment's and the rows', the cut, the exit criteria: done 2026-11-11 (whole, decision 79; `TWN-Q14`, decisions 80–83; the five drafts `adr-drafts/*-increment-3.md` accepted; Part E §48 to §58; no version moved; `E09-log.md`).
- **T23** batch TWN-3 (survey §14, row 6 with design §55's record; decision 74). Done 2026-11-11, whole (decision 84): two runs of each part, every row as the reading (survey §17; `E09-log.md`).
- **T24** `shared/` (`cross-lane`): done 2026-11-12, whole (decision 85): the five drafts in `docs/adr/` with T23's verdicts, `StreamId::TOWNS`, the Legion's two effects and the two town flags (pack format v52, core ruleset hash v53), the packs' six artifacts, the 82 replays' and the bot reports' core hash, the glossary, `docs/06-security.md` (`E09-log.md`).
- **T25** growth's town side: **T25a** done 2026-11-12, whole (decision 86): the part's roles `horde`, `portal`, `week_of`, `roe` and the growth's parameters (part v5; the packs re-harvested, `cross-lane`), each town's fourteen counts and Portal (store v3, `TWN-QUIRK-8`), TWN-GRO-001, TWN-GRO-005; game state hash v28 and the 72 adventure replays (`cross-lane`; `E09-log.md`). **T25b** in two (decision 87): **T25b1** done 2026-11-12: `WeekKind`, TWN-GRO-003, -004, `TWN-QUIRK-9`, `-10` (no version moved; `E09-log.md`); **T25b2** done 2026-11-12, whole (decision 88): `garrison_week`, `summon`, TWN-GRO-006, -007 (no version moved; `E09-log.md`). **T26** growth in the game (`cross-lane`), T26a in two (decision 89): **T26a1** done 2026-11-12: the inputs (`h3_heroes::town_growth`, `owner_town_growth`), the towns' week and month in the day (TWN-GRO-001, -003, -004 in the game; GAM-CAL-002), the events' creatures (TWN-EVT-004, `TWN-STUB-10` lifted), no version moved (`E09-log.md`); **T26a2** the week's kind; **T26b**; **T27** recruitment (`cross-lane`); **T28** the rows (T28a, T28b; `cross-lane`; after E08's T114a and T115a); **T29** the gates; **T30** the audit and the closure (design note §56).

### Exit criteria

Accepted at `TWN-Q14` (decision 83; the detail in design note §57):

1. The part's roles `horde`, `portal`, `week_of`, `roe` and the growth parameters compile; each town holds its fourteen counts (at most 32 767, `TWN-QUIRK-8`) and its Portal; a built dwelling gives its first creatures (TWN-GRO-005).
2. A dwelling's growth is the reading's sum; each week adds it with the week's bonus, halved in a neutral town (TWN-GRO-001, -003).
3. The week's kind from the towns' stream, public; the month after a `week_of` week; a creature's month (`TWN-QUIRK-10`) and the Plague (`TWN-QUIRK-9`) (TWN-GRO-002, -004; GAM-CAL-002, -003).
4. A neutral garrison's week; the Portal each week, at its build, a flagged dwelling and a capture; a town event's creatures (TWN-GRO-006, -007, TWN-EVT-004; `TWN-STUB-10` lifted).
5. Recruitment at a standing dwelling, horde or Portal through the hire dialog into the upper bar's army, an upgraded index's two creatures from one count; refusals change nothing (TWN-REC-001, -002; GAM-DEC-008).
6. Stacks within an own town's upper bar and between it and the visitor, by the one owner rule (TWN-ROW-001; GAM-MAN-001, -002).
7. The visitor enters the garrison all or nothing, the garrison hero leaves within the most heroes on the map, the two swap; a garrison hero keeps its record, its points at the turn's start, its spell points and marks' drops, its income and weekly resources (TWN-ROW-002 to -005; ADV-MOV-002, GAM-RES-003, -005).
8. A garrison hero alone defends the siege, a visitor beside one fights outside and the town stays, an empty one yields the town (`TWN-QUIRK-11`); level 3 sees it (TWN-SIE-001, -002, TWN-CAP-003, TWN-VEW-001).
9. Counts, Portal and hire dialog the owner's, the week's kind everyone's; every changed view told (TWN-VEW-004).
10. The env plays recruitment and the hero moves (v10, v11); the replays record them.
11. Every rule with id, source and tests; `TWN-QUIRK-8` to `-11`; the gates (coverage, the re-recorded replays, the census to day 29, the benchmarks); batch TWN-3 run and scored.

## Risks

- **Two lanes in the same crates:** the town modules sit in the adventure lane's crates, and their common parts, versions and replays change in both lanes; merges in order, a rebase and a second re-recording when both bump. Check the adventure lane's open pull requests at the start of every task.
- **The size of the content:** nine factions of about forty buildings each, with requirements read from the executable; the harvest is in the battle lane's crate.
- **Editions:** the moat (ADR-0023), Conflux and the AB buildings, RoE's differences (E11).
- **The faction choice touches E10's scenario setup** (decision 6): the setup's faction per player is the seam; the rest of the scenario setup stays E10's.
- **Other lanes read the town:** the env (AI lane), the map client (E24 waits for towns), the server's wire for new commands and events; notes to their inboxes when a town change lands.
