# E09 — archive: closed increments

Moved out of `E09-towns.md` verbatim on 2026-11-06 (increment 1, T10) and on 2026-11-11 (increment 2, T20; decisions 36 to 63), so that the epic file holds what is open and the decisions that apply to the whole epic (1 to 7, and those from 64). Cited elsewhere as "E09 decision N", "increment N" or by task (T7b2a …). The first section is the epic's starting point at its refinement. Increment 1's design note stays in `E09-design.md` (its Parts A to C, §1 to §22; decision 40), and so does increment 2's (Part D, §23 to §36; T20); the record of every task is in `E09-log.md`, the audits of the exit criteria in its T10 and T20.

## Starting point (2026-10-31)

- **The map:** `h3-map` reads the towns' records (classes 77 and 98: owner, name, garrison and formation, the built and forbidden buildings or the has-fort flag, the required and banned spells, the town events, SoD's alignment byte; `docs/formats/h3m.md`'s object table). The import keeps a town as an untyped blocker (its footprint; a hero leaving a town's gate leaves upward, the queue archive's T14a note) and refuses a hero generated at a main town (128 maps) and a hero standing on a town's blocked tile (`Unsupported::HeroInTown`, 3 maps). The writer writes towns of a fixed faction (E04c T5b) for experiment maps.
- **The battle:** a siege plays from `BattleSetup.siege` (ADR-0023: the faction, the fort level, the moat by edition, the town's battle facts), given today by scenarios and the CLI; whoever starts a battle at a town derives them ("the town context, E09"). `Defender::Garrison` (E08 T84a) fights a mine's guards.
- **The game:** the store of heroes (ADR-0017; "the pool and the tavern are E09's"), the day's income (GAM-RES-004), the month's kind with the Plague, which changes nothing before towns (ADR-0017 point 4), the week's creature waiting for E09 and drawn from a stream of its own (E08 decision 197), `ADV-STUB-9` (a Tavern town to hire a retreated hero again), flags and owned objects (E08 increment 9).
- **Content:** factions are core content; no building, dwelling or town service is harvested yet, nor the spells' town chances or the hero classes' tavern weights (E04a T3a's note).
- **Notes from earlier work** for E09 are gathered in the town queue's inbox (`docs/lanes/town/QUEUE.md`).

## Increment 1 — towns on the map

### Scope

- **The import and the start:** towns of a fixed faction (98) and random towns (77); the players' faction choice at the start and a random town resolved from it (the original's rule for a neutral random town: the survey's); the town's record into the state (owner, garrison and formation, the buildings built at the start as the map sets them, its town events kept for the increments that apply them); **the main town's generated hero** with the hero pool it draws from (the faction's classes, the map's allowed heroes; the original's draw read); **the random hero** (class 70) by the same draw (decision 8) and **the starting-hero pick** beside the faction in the setup (decision 9); a hero placed on a town's blocked tile moved to the town's entrance, as VCMI's "H3M format quirk" (3 maps; only right of the entrance, decision 13).
- **The town on the map:** its entrance entered (the town classes typed: a visit from the entrance tile, the gate's leaving); the visiting hero; a player's own town visited (no service yet); **capture**: an unguarded town of another player or of none flagged by its visitor, a garrisoned or visited one taken by a **siege** with ADR-0023's facts derived from the town (faction, fort level, the moat by edition) against the garrison or the visiting hero (`Defender::Garrison`), the outcome applied (the survivors, the flag); the flag's reveal (E08's flag radius); an own or allied town entered, never captured (decision 16).
- **The day's income** of a town's hall and the Grail, with its Resource Silo's resources (decision 10), read by the day's start (E08 increment 9's path); one Capitol per player after a capture (survey §8.4, §11.3).
- **What a viewer sees** of a town (owner, faction, the fort level, the garrison by the original's levels: decision 11), written into `docs/06-security.md`.
- **Content** for what it reads: the buildings' identities per faction, the hall and fort levels, the silo's resources (T1a2: the starting heroes' draw uses no tavern weights, which move to increment 3); the harvest `cross-lane` (`h3-harvest` is the battle lane's) or a `shared/` task (E08 T81b's way).
- **Not in it:** construction, growth and recruitment, the Tavern's hire, the mage guild and every other service (increments 2 to 4); a player's defeat for losing all towns (E10); a computer player's town play (E10, E13).

### Decisions at the survey's checkpoint (2026-11-02, the user, TWN-Q2)

8. **The random hero in increment 1** (class 70; 176 on 36 shipped maps, 21 refused for it alone): drawn as the main town's hero (survey §4.4), its body then a placed hero's; every shipped map imports. Not chosen: later, with the Tavern (increment 3).
9. **The starting-hero pick, as the original** (survey §4.3): a hero per colour in the setup beside its faction (decision 6's seam), checked against the original's offer (up to 16 free heroes of the faction's two classes the colour may take, offered when its main town generates a hero or it has the random-hero flag); it goes to the colour's first random hero, else to a human colour's main-town hero; reserved before any draw (decision 12, T-2). Not chosen: the same for every colour (a computer colour's main-town hero too); every starting hero drawn until E10.
10. **The Resource Silo pays from increment 1** (survey §11.1): its resources a day by faction beside the gold (a constant of the executable, ADR-0006 tier 2, the reading its source); Rampart's Treasury and the Mystic Pond stay increment 4's. Not chosen: all of it in increment 4.
11. **The views by the original's levels** (survey §12.3, §12.5): the army another player sees by the viewer's towns with a Tavern (none: nothing; one: the stacks' creatures; two or more: their size words too), the counts with the hall, the income, the silo and the garrison hero's mark for the owner's team or a viewer's hero with expert Visions or a Rogue within reach (Visions when it is placed, E08's "Not placed yet"). Not chosen: VCMI's two levels; the owner's team only until increment 3.
12. **The quirk candidates as proposed:** T-1 (an alignment byte from 8 to 254) refused by the import, typed; T-2 (the pick not reserved) and T-3 (a random hero taking a later hero's type) fixed (state-corrupting): the picks reserved and the placed heroes' and prisoners' types out of the pool before any draw; T-4 (the draws' empty cases) typed failures; T-5 (a garrison hero without creatures) the original's, built with increment 3. Not chosen: T-5 fixed as a bug.
13. **TD-1, TD-4 and TD-6 by the reading** (no shipped map affected or a one-point chance; the wiki agrees where it speaks): an owned random town aligned to another colour takes the alignment's faction; a neutral town's level-4 roll 13 in 100; only a hero right of a town's entrance moves onto it. **TD-3, TD-5 and TD-7** are built by the reading and confirmed by batch TWN-1 (decision 15). Not chosen: batch rows for TD-1, TD-2 and TD-6; VCMI's where it differs.
14. **TD-2 by the enabled expansions, not the map's version** (the user: "the map version is never the decisive factor; it is always the enabled expansion that delivers the rules"; E08 decision 104, ADR-0017): with AB or SoD in the ruleset's packs, a neutral random town draws from nine factions on any map, and a player of an RoE-format map whose eight bits are all allowed may have Conflux (its Random draw or its pick); a player restricted to fewer stays restricted (VCMI's rule); with RoE alone, eight. The original's map-format test (survey §2.3) is the recorded deviation. Not chosen: Conflux always for an RoE-format map's players; never for players.
15. **Batch TWN-1 keeps rows 1 to 4** (TD-3, TD-5, TD-7, a siege's aftermath; survey §13). Rows 6 and 7 (the income, the Capitol after a capture) dropped: the reading, VCMI and the wiki agree. Row 5 (the game's version for an RoE-format map) dropped: by decision 14's rule the moat follows the enabled packs, and the reading (the scenario dialog's version, survey §9.4) is the original's source. Not chosen: all seven; rows 1 to 3 only; row 5 kept as a record.
16. **Teams at a town: the step's test** (survey §7.1): an own or allied town is entered, never captured; a defended town of another team or of none is attacked. A siege of a town whose visitor is an ally's hero (the ally defending under the town's owner) is a typed refusal, a stub until E10. Not chosen: the ally's defence built now; no teams until E10.

### Decisions at the ADR drafts (2026-11-02, the user, TWN-Q3)

17. **A store of towns in the game state:** `h3-towns`' state beside the adventure's, every town's record by a contract id `TownId` (its index among the map's towns), each shared by forks and copied on its first write as the store of heroes; its own hash part and version; the faction held once there (the draft ADR-0028). Not chosen: the town records inside the adventure state; the same store with an id local to `h3-game`.
18. **The hero pool is `h3-game`'s,** beside the store of heroes (ADR-0017's amendment): the starting heroes drawn from it at the build, the Tavern's offers (increment 3) composed by the game. Not chosen: the pool in `h3-towns`.
19. **A town ruleset part of its own:** `TownRuleset` compiled on the core, `packs/*/towns.toml`, an adapter crate `h3-pack-towns` (the town lane's). Not chosen: a section of `adventure.toml`; the core content.
20. **The moat in the town part, the SoD pack overriding** (decision 14's data): per faction the moat's kind and the siege's natural obstacles, the base pack as RoE and AB play them, the SoD pack's overrides as SoD plays them. Not chosen: the battle part's siege record; an edition input derived from the packs.

### Decisions at the design's checkpoint (2026-11-02, the user, TWN-Q4)

21. **The five ADR drafts accepted as written** (the draft ADR-0028 and the amendments of ADR-0015, -0016, -0017, -0023 in `docs/lanes/town/adr-drafts/`): T4 moves them into `docs/adr/` after batch TWN-1, with its verdicts (ADR-0028's number checked then). Not chosen: accepted with changes.
22. **The cut as proposed** (`E09-design.md` §21): fourteen tasks in rounds 2 to 10, batch TWN-1 first (T3a, T3b, T3c), then the `shared/` task (T4), the part and its content (T5a, T5b), `h3-towns` (T6), the import and the build (T7a, T7b), the map side (T8a, T8b, T8c), the gates (T9), the audit (T10); each task's cut confirmed at its start. Not chosen: the shared task before the batch; fewer, larger tasks.
23. **The buildings' content read and harvested** (design §3): a code reading of `building_tables_init` 0x4EB730 and the tables it fills, the names from the text files, VCMI's lists compared, written by `h3-harvest`'s gap-fill (`cross-lane`, as E08 T81b). Not chosen: a hand-written `towns.toml` with each value's source; only the roles increment 1 reads, the full lists in increment 2.
24. **The design note's agent's calls kept** (marked *agent's call* in `E09-design.md`; each stays changeable by the task that builds it). Not chosen: changes now.

### Decisions at batch TWN-1's scoring (2026-11-03, the user, TWN-Q5)

25. **Building 23 re-read in T5b** (survey §15: built in every Rampart, Necropolis and Conflux town of the standard set, not by the reading, not in VCMI): T5b, which stages the executable and reads `building_tables_init`'s lists anyway, reads where 23 comes from, whether a custom set gets it and whether the faction's list and so the keep count it; the content and TWN-SET-003 build it as found, a batch row only if a point stays open. Not chosen: a separate re-read task before T4; VCMI's way (no building 23).
26. **Batch TWN-1's row in `docs/experiments/README.md`** by T3c, `cross-lane` (the battle lane's file, as T3a and E08 T80). Not chosen: a note in the battle lane's inbox.

### Decisions at the buildings' reading (2026-11-03, the user, TWN-Q6)

27. **The hall companions left out of the town part** (survey §16.3): building 23 in Rampart,
    Necropolis and Conflux is the Village Hall's town-screen piece (27 to 29 the higher halls'),
    built with each hall, in no faction's list, read by no rule; a client draws them. Not chosen:
    companion buildings in the three records, built with each hall and excluded from the keep.
28. **The keep's count skips the hall-1 building in the derivation** (survey §16.4; design §6's
    `structures`): no faction's list holds the Village Hall, so the original never counts it. Not
    chosen: a building flag `buildable = false` (recommended: it would also refuse building it in
    increment 2); counting it as a recorded deviation.
29. **The standing set derived, T-6 fixed** (survey §16.5; design §4): the closure of
    `supersedes` plus the hordes' upgrade (a horde and its level's upgraded dwelling ever built
    bring the upgraded horde, as `town_build`); a custom set's horde does not leave its dwelling
    standing (`TWN-QUIRK-5`, fixed). Not chosen: both sets stored and built as `town_build` does,
    T-6 kept (recommended).
30. **TD-8 by the code** (survey §16.8): the Fortress' Glyphs of Fear (21) +2 attack, the Blood
    Obelisk (22) +2 defense for a siege's defender, as VCMI; the game's descriptions and the
    fandom wiki the reverse. Not chosen: the descriptions; a batch row first.
31. **A Rogue's Visions as town part data** (T8c2, *agent's call*; survey §12.3): any pack's
    `[town_visions]` names the creatures (the AB pack the Rogue), the base pack's `[town_params]`
    Visions' expert factor and least radius (3, 3), until the adventure map places Visions
    (then its spell's own record); measured from the town's entrance, strictly below, the
    viewer's own heroes only (the original's; VCMI the selected hero). Not chosen: a creature
    flag in the core (a `shared/` format change), the factor read from the core's spell now.
32. **T9 in three** (the user, 2026-11-06, at its start; as E08's T96): **T9a** the writer's town
    objects (a player's main town, random towns, a random hero, random garrison stacks), the town
    map and its golden replays; **T9b** the plans' towns and their properties; **T9c** the
    benchmarks, the env's test and the bot reports, the census with the original data (exit
    criterion 1). Not chosen: T9a in two (the writer and map, then the replays); two parts, the
    properties with the map.
33. **T9b in two** (the user, 2026-11-06, at its start; as E08's T96b1 was built, by halves):
    **T9b1** the plans in `play.rs` (towns laid out and played, the towns' checks, the cover
    test; `view.rs` keeps plans without towns); **T9b2** the view properties with towns
    (`view.rs`: the hygiene property's changes and the told views). Not chosen: T9b whole; three
    parts (the layout and invariants, the steps and the cover test, the views).
34. **T9b2 in two, the told views first** (the user, 2026-11-06, at its start): **T9b2a**
    `views_move_as_the_events_say` and the boat test with the plans' towns (`check_told`'s town
    events, a check of the town views against the state); **T9b2b** the hygiene property with
    towns (`vary` learns them; `townless` gone). The agent's call told with the question: the
    hygiene property varies the seed, and a seen random town's faction or a neutral garrison is a
    public draw, so its plans' towns draw nothing public (as E08 T96b1's random dwellings of one
    outcome), and `vary` removes a town only after every seen one (a seen town's `TownId` counts
    the towns before it, `docs/06-security.md`). Not chosen: T9b2 whole; the hygiene property
    first.
35. **T9c kept whole** (the user, 2026-11-06, at its start; offered in two, the benchmarks then
    the env and the census, or the reverse): one pull request with the benchmarks, the env's test
    and the bot reports, the census with the original data. *Agent's calls (change freely):* the
    siege's workload counts the game's step that opens the siege, the move decoded before it (its
    reachable-tiles query, two thirds of the env's step, is the reach workloads'); a fork workload
    on the town map (`fork-towns-36`: the design named only `fork-xl-144`, whose map has no
    towns); the census also counts the built towns' siege facts, halls and income. Not chosen:
    T9c in two.

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, the cut confirmed at each checkpoint)

- **T0 E09 refined, the town lane opened.** Done (2026-10-31, `shared/town-lane`; decisions 1–7; TWN-Q1).
- **T1a1 The towns survey, part 1a: the census and the factions.** Done (2026-11-02, `town/e09-t1a1`; T1a cut in two at its start, the user): survey `E09-towns.md` §0 to §3; every map imports once the main town's hero, a hero in a town and the random hero exist; TD-1, TD-2, T-1.
- **T1a2 The towns survey, part 1b.** Done (2026-11-02, `town/e09-t1a2`; the user kept it whole): survey `E09-towns.md` §4 to §6; one uniform draw for the main town's and the random hero (no tavern weights); TD-3 to TD-6, T-2 to T-4.
- **T1b1 The towns survey, part 2a: the town on the map.** Done (2026-11-02, `town/e09-t1b1`; T1b cut in two at its start, the user): survey `E09-towns.md` §7 to §10; a defended town visited from the neighbouring tile, capture (the attacker stays outside after a siege), the change of owner and one Capitol per player, the garrison merged into a lone visitor's army, ADR-0023's facts and the game's version; TD-7, T-5.
- **T1b2a The towns survey, part 2b1: income, views and batch TWN-1's rows.** Done (2026-11-02, `town/e09-t1b2a`; T1b2 cut in two at its start, the user): survey `E09-towns.md` §11 to §13; a town's gold by its highest hall and the Grail, one Capitol per player only after a capture; another player sees the fort level and the army by its taverns (none, creatures, size words), the counts and the hall only as the owner's team or with expert Visions or a Rogue; seven rows.
- **T1b2b The checkpoint.** Done (2026-11-02, `town/e09-t1b2b`; `TWN-Q2`, three rounds): decisions 8–16: the random hero and the starting-hero pick in increment 1, the silo paid, the views by the original's levels, the quirk candidates as proposed, the disputes by the reading but TD-2 (the enabled expansions), batch TWN-1's rows 1 to 4, the allied town entered.
- **T2a The ADR drafts.** Done (2026-11-02, `town/e09-t2a`; T2 cut in two at its start, the user; `TWN-Q3`, decisions 17–20): `docs/lanes/town/adr-drafts/`: the draft ADR-0028 (`h3-towns` and `h3-pack-towns`, the split of a town, `TownId`, the store of towns, the town part, the seams, the moat by the packs, the pool the game's) and the amendments to ADR-0015 (crates, part, the import's towns and starting heroes), -0016 (the town as a map object: entered, capture, reveal, views), -0017 (setup, pool, the build's draws, income, capture and sieges), -0023 (the derivation, the moat by the packs).
- **T2b The design note, the cut and the checkpoint.** Done (2026-11-02, `town/e09-t2b`; the user kept it whole; `TWN-Q4`, decisions 21–24): `E09-design.md` §1 to §22 (the town part, the record and store, the setup and derivations; the import, the kind `Town`, the step, the siege, capture, the views; the setup's checks, the pool and the draws, the siege's battle, the income, the observation, the gates, versions and ids, batch TWN-1, the cut); the drafts accepted, the cut as proposed, the content harvested, the agent's calls kept.
- **The build** (decision 22; design §21; the queue holds the next round's entries):
  - Round 2: **T3a** batch TWN-1's maps and spec (§20; `cross-lane` in `h3-experiment`). Done (2026-11-02, `town/e09-t3a`; the user kept it whole): `fort` and an explicitly empty `garrison` for a spec's towns; `experiments/specs/E09-batch-1.toml`, rows 1 to 4 (row 3 played twice from one save).
  - Round 3: **T3b** batch TWN-1's harness and runs. Done (2026-11-03, `town/e09-t3b`; the user kept it whole, 3b in a new game): the harness's town reads, a new-game step, the town screen known; two whole runs recorded (`experiments/results/E09-batch-1.md`, `-run2.md`). **T3c** scored. Done (2026-11-03, `claude/peaceful-curie-q7wjex`; `TWN-Q5`, decisions 25, 26): survey `E09-towns.md` §15; TD-3, TD-5, TD-7 and the siege's aftermath as the reading, VCMI refuted in each; building 23 in every Rampart, Necropolis and Conflux town of the standard set, not predicted: re-read in T5b; the experiments README's row (`cross-lane`).
  - Round 4: **T4** `shared/` (cut in two at its start, the user: the ADRs first). **T4a** the ADRs and the glossary. Done (2026-11-03, `shared/e09-t4a`, `cross-lane`): ADR-0028 and the amendments of ADR-0015, -0016, -0017, -0023 in `docs/adr/` with T3c's verdicts (no point changed), the index's rows, `docs/03-domain.md`'s terms, the battle lane's inbox note; the drafts deleted. **T4b** the crate skeletons, `TownId`, the gates (§1). Done (2026-11-03, `shared/e09-t4b`, `cross-lane`): `h3-towns` (domain, logic: the empty `TownRuleset` and `Towns`, both version constants 1) and `h3-pack-towns` (adapter, doc only); `TownId` in `h3-contracts`; `Moat`'s doc comment (the packs decide); `arch.toml`'s edges, T5a's callers' too; `lanes.toml`; `clients.yml`; the versions rows, `docs/02` §3, ADR-0005, ADR-0006, `packs/README.md`; round 5's queue.
  - Round 5 (T5a cut in two at its start and the order changed, the user: the content before `GameRuleset`, so the replays move once): **T5a1** the town part, its compiler and the adapter (§2). Done (2026-11-03, `town/e09-t5a1`): `TownRuleset` with the checks and the hash's first encoding, `BuildingId` and `BuildingSet`, `h3-pack-towns`' parser and format; the file's shape changed where built (`gold`, one to seven levels, the siege's roles split; the log). **T5b** the content (§3, decision 23), cut in two at its start (the user): **T5b1** the reading. Done (2026-11-03, `town/e09-t5b1`; `TWN-Q6`, decisions 27–30): survey §16: `building_tables_init` and `town_build` read whole, building 23 the Village Hall's town-screen companion in three factions (decision 25), the keep never counting the Village Hall, the standing set's order (T-6), VCMI compared, TD-8, TD-9. **T5b2** the harvest, cut in two at its start (the user): **T5b2a** the hordes' upgrade as data and the synthetic records. Done (2026-11-03, `town/e09-t5b2a`): a building's `auto_with` (the compiler's checks, the part's hash v2), the synthetic packs' `towns.toml`, the adapter's constants; **T5b2b** the reader, the harvest and the three packs, cut in two at its start (the user): **T5b2b1** the reader. Done (2026-11-03, `town/e09-t5b2b1`, `cross-lane`): `h3-formats`' `TownTables` and `town_data` (the faction lists, the supersede lists, the map's slots, the creatures, the hordes, the silo, the neutral garrison, the 30), `installation.md` §7.61, `h3-harvest`'s `Inputs::town_tables`; **T5b2b2** the harvest and the three packs, whole (the user), the harvester compiling the town files after a `shared/` edge (the user). Done (2026-11-03, `town/e09-t5b2b2`, `cross-lane`; `shared/e09-harvest-towns-edge` first): `h3-harvest`'s `towns` module and the gap-fill's `[towns]` (the parameters, the roles, the moats by edition), `packs/{base,ab,sod}/towns.toml`, `h3-pack-towns`' `ORIGINAL_PACKS` and their test; **T5a2** `GameRuleset`, its callers and replay format v9 (§2), whole (the user). Done (2026-11-03, `town/e09-t5a2`, `cross-lane`): `GameRuleset::new` takes the town part, its check the part's core and a siege record for every town record; the callers pass the original part or the empty one (an encounter); the CLI's loader reads `towns.toml`; the adventure replay's header the part's hash (format v9), the 39 replays' headers re-recorded.
  - Round 6: **T6** `h3-towns`: the record, the store, the setup's functions, the derivations, `town.toml` (§4–§6); cut in three at its start (the user). **T6a** the record and the store. Done (2026-11-03, `town/e09-t6a`): `Town` (the ever-built and forbidden sets, the standing set derived), `TownStatic` and `TownEvent`, the store `Towns` and town state hash v1's first encoding. **T6b** the derivations (§6), whole, split between the agent and a sub-agent (the user). Done (2026-11-03, `town/e09-t6b`): the hall and the day's income, the fort level, the siege's facts with the named bonuses, the natural obstacles, the defender's bonuses, the reveal, the Tavern, the Capitol's limit with the store's first write; `docs/rules/catalog/town.toml` (eight rules). **T6c** the setup's functions (§5), whole, its code by a sub-agent (the user). Done (2026-11-04, `town/e09-t6c`): `TownSetup` and its types, the buildings at the start (TWN-SET-003; T-6 fixed as `TWN-QUIRK-5`, `docs/rules/quirks-town.md`; `TWN-STUB-2`), the garrison (TWN-SET-004), `town_setup` (the garrison's draws, then the buildings').
  - Round 7: **T7a** the import and the setup's checks (§7, §13), cut in two at its start (the user). **T7a1** the setup's new fields and the import. Done (2026-11-04, `town/e09-t7a1`, `cross-lane`): `PlayerStart` and `GameSetup`'s towns and pool inputs, a random hero without a type, the towns, the starts, the moved heroes (TWN-SET-010) and the pool's inputs imported; map import v17 (E08 T93b took v16); until T7b the build refuses a setup that needs a draw (`TWN-STUB-3`); 156 of the 160 shipped maps import, 4 are refused for a dwelling entered from water since E08 T93b (the census in `h3m.md`). **T7a2** `validate`'s checks (§13). Done (2026-11-04, `town/e09-t7a2`, `cross-lane`): `TownFaction`, `TownAlignment`, `Choice`, `MainTown` (its town one of the setup's), `Pick` (the offer, `GameSetup::offer`), `PickTaken`, `RandomHeroOwner`; `TownCount` and `MainTown`'s owner moved to T8a (the user); no version moved; no shipped map refused by a new check. **T7b** the pool and the build's draws (§14), cut in two at the towns at its start (the user: their owners and objects come with T8a). **T7b1** the pool and the heroes' draws. Done (2026-11-04, `town/e09-t7b1`, `cross-lane`): `HeroPool` in `GameState` (TWN-SET-005; `TWN-QUIRK-2`, `-3`), the players' `Random` choices a start reads (TWN-SET-001), the random heroes' types by pick or draw (TWN-SET-007, -008; `NoFreeHero`), a hero that leaves the map freed; game state hash v23, the 39 replays rehashed; `TWN-STUB-3` narrowed to a main town's hero; 31 shipped maps build (9 before). **T7b2** the towns' draws and the store (§14 steps 2's towns, 3 and 4; the garrisons and radii into the adventure), after T8a1 gives the town list and owners (round 8's order, the user, 2026-11-04); cut in two at the store at its start (the user). **T7b2a** the towns resolved and stored. Done (2026-11-04, `town/e09-t7b2a`, `cross-lane`): the random towns' factions, each town's `town_setup`, the store of towns in the game state (game state hash v24, the 39 replays rehashed), the garrisons and the towns' own reveal radii into the adventure; `TWN-STUB-5` narrowed to a whole-map reveal at the start; the census unchanged (31 build, 294 towns). **T7b2b** the main towns' heroes (§14 step 3; `TWN-STUB-3` lifted). Done (2026-11-05, `town/e09-t7b2b`, `cross-lane`): each main town's hero (TWN-SET-009: a human's pick, else the draw) on its entrance, ids after the setup's heroes, the unused picks freed after it; a second hero on an entrance refused (`TWN-QUIRK-6`); the guild's spells at the start wait for increment 4 (`TWN-STUB-6`, TD-10); the hero right of an entrance confirmed the visitor (the user's question); no version moved; all 156 imported maps build (31 before).
  - Round 8: **T8a** the kind, the step, entering and an undefended capture (§8, §9, §11), with `validate`'s `TownCount` and `MainTown`'s owner (from T7a2), cut in two at its start and the order changed (the user, 2026-11-04: T8a1, T7b2, T8a2, then T8b). **T8a1** the town as a map object (§8): the kind `Town` and the type `town` (adventure part v17), the import's towns typed with their owners (map import v18), the static map's town list and `Holds::Town`, towns ownable, the garrisons and the towns' reveal radii from the game (empty and the flag's until T7b2, `TWN-STUB-5`), `TownCount` and `MainTown`'s owner; adventure state hash v21 (after E08 T94b's v20); the entrance not entered until T8a2 (`TWN-STUB-4`). Done (2026-11-04, `town/e09-t8a1`, `cross-lane`, after E08 T94b's #716): `ObjectKind::Town`, the static map's `towns`, `town_id`, `town_object`, `Holds::Town`, `AdventureSetup::garrisons`, `town_garrison`, `start_game`'s radii; `GameSetupError::TownCount`, `MainTown` widened to a town of its own; the census unchanged (31 build), 1 468 towns typed. Then **T7b2** (above). **T8a2** the step onto a town, entering and an undefended capture (§9, §11), kept whole (the user). Done (2026-11-05, `town/e09-t8a2`, `cross-lane`): the entrance a typed visit (`TWN-STUB-4` lifted), an own town entered, an allied visitor met, an undefended town of another player or none captured (`AdventureOutcome::capture`, `capture_town`, the game's `town::capture` with the Capitol's limit, `TownCaptured`, `TownChanged`), a whole-map reveal at the start and the capture (`TownReveal`, `TWN-STUB-5` lifted); a defended town and another team's visitor refused until T8b (`TWN-STUB-7`); the reach's loop unchanged (a cold pass), the bench within −0.8 % to +0.38 %; no version moved. **T8b** the siege (§10, §15), cut in two at its start (the user). **T8b1** the siege's battle. Done (2026-11-05, `town/e09-t8b1`, `cross-lane`): `BattleStart::town`; the game's siege setup (the town's facts, the defending hero raised, the native terrain and the entrance's special ground), the defender's spell points after it, the attacker's +500 (TWN-SIE-006), the garrison's survivors and the capture after an attacker's win with the attacker outside, after the winner's levels (TWN-CAP-003); a walled Stronghold's natural obstacles `TWN-STUB-8`; no version moved. **T8b2** the step onto a defended town, the merge and the defender (§9, §10), cut in two at its start (the user, 2026-11-05). **T8b2a** the town its garrison alone defends. Done (2026-11-05, `town/e09-t8b2a`, `cross-lane`): `town_visit` (the visit's mode for the player: entered, from the tile before, or refused for another team's visitor), `visit_at` reading it, `visit_town`'s `siege_start` (`Defender::Garrison` owned, `Defender::Guards` neutral, at the entrance, `BattleStart::town`), `StaticMap::town_formation` set by the game's `siege_setup`; TWN-SIE-001; `TWN-STUB-7` narrowed to a visitor of another team; the reach's loop untouched, the bench within its gate; no version moved. **T8b2b** the siege against a town's visitor and the garrison's merge (§10), kept whole (the user, 2026-11-05). Done (2026-11-05, `town/e09-t8b2b`, `cross-lane`): `town_visit` gives a town another team's visitor defends the neighbour's visit, the step onto that visitor (`besieges`) the town's siege in the walk and the passage (`TWN-STUB-7` lifted), `siege_start`'s `Defender::Hero`; the merge (TWN-SIE-002: `h3_adventure`'s `town::merge`, `AdventureState::merge_garrison`; the game's `town::merge` before `start_battle`), its slots told as `StackChanged` to each army's owner by a new `ArmyRef::Garrison` (the user's choice; no army command manages it before increment 3, `ArmyCommandError::Garrison`); batch TWN-1's row 3 reproduced; the reach's loop untouched; no version moved. Round 8 closed.
  - Round 9: **T8c** the income, the views, the observation, `docs/06-security.md` (§12, §16, §17), cut in two at its start (the user, 2026-11-05: at the Rogue). **T8c1** the income and the observation's towns. Done (2026-11-05, `town/e09-t8c1`, `cross-lane`): each owned town's `daily_income` in the day's advance (TWN-INC-001, -002), `ObjectView::town` and `GameObservation::towns` (`TownView`: the public facts, the garrison by the viewer's taverns, the owner's team's top level; TWN-VEW-001 to -003); `TWN-STUB-9` (a Rogue's Visions); no version moved. **T8c2** the Rogue's Visions, `docs/06-security.md`'s "Towns", the inbox notes, kept whole (the user, 2026-11-06). Done (2026-11-06, `town/e09-t8c2`, `cross-lane`; the security doc a `shared/` pull request): `TWN-STUB-9` lifted (TWN-VEW-003; decision 31), town part v3, `h3_heroes::power`. **T9** the fixtures, replays, properties, benchmarks, the env (§18), cut in three at its start (the user, 2026-11-06; decision 32): **T9a** the writer's town objects, the town map, its replays. Done (2026-11-06, `town/e09-t9a`, `cross-lane`): `writer::AnyTown` (random towns, main towns, names, random stacks), `writer::RandomHero`; `fixtures/maps/e09-towns-36.h3m`; golden replays `towns-s173-d21`, `-s1702-d21`, `-s3435-d21` (random) and `towns-capture-s1`, `-siege-s1`, `-visitor-s1`, `-capitol-s1`, `-income-s1`; 55 adventure replays; no version moved. **T9b** the plans' properties, cut in two at its start (the user, 2026-11-06; decision 33): **T9b1** the plans play towns. Done (2026-11-06, `town/e09-t9b1`, `cross-lane`): `TownPlan` (one to four towns in three plans of four: fixed or random, owned or neutral, garrisons, the standard and Capitol sets, visited at load), `Step::Town` and its runs, `check_towns`, the entrances' invariant, `check_town_battle_end`, the income's and the isolation's town parts, the forks' store; the cover test `the_plans_capture_and_besiege_towns`; no version moved. **T9b2** the view properties with towns, cut in two at its start (the user, 2026-11-06; decision 34): **T9b2a** the told views with towns. Done (2026-11-06, `town/e09-t9b2a`, `cross-lane`): `check_told` follows `TownCaptured`, `TownChanged`, the merge's garrison slots and a town's view; `check_town_views` (the towns' objects and views against the state); `views_move_as_the_events_say` and the boat test with the plans' towns; the cover test `town_views_move_as_the_events_say`; no version moved. **T9b2b** the hygiene property with towns, kept whole (the agent, told at its start). Done (2026-11-06, `town/e09-t9b2b`, `cross-lane`): `hygienic` (the plans' towns draw nothing public), `vary` learns towns (where unseen another owner, garrison and buildings; removed only after every seen one, added after every other) and keeps a Shipyard's boat offsets (a gap of `master`'s), `outcome` compares the Capitols' changes, `check_town_views` in both games; two unit tests; no version moved. **T9c** the benchmarks, the env, the census, kept whole (the user, 2026-11-06; decision 35). Done (2026-11-06, `town/e09-t9c`, `cross-lane`): the workloads `siege-towns-36` (`h3 adventure bench --workload siege`), `day-start-towns-36`, `reach-towns-36`, `fork-towns-36`, `fork-xl-144` re-set at `master`'s count; the env's test on the town map (`h3-sim`'s `adventure/tests/towns.rs`, the town map in the main property); the bot reports unchanged; the census: all 160 shipped maps import and build (exit criterion 1), with the towns' siege facts, halls and income; no version moved.
  - Round 10: **T10** the audit against the exit criteria; increment 1 closed; increment 2 refined. Done whole (2026-11-06, `town/e09-t10`; decision 36; `TWN-Q7`, decisions 37–40): the seven criteria hold (`E09-log.md`); TWN-SIE-001, -002, TWN-CAP-003, TWN-SET-003, -004 `verified` by batch TWN-1; increment 1 closed; increment 2 refined (construction with the town events; T11 the random dwellings tied to a town, then the survey T12).

### Exit criteria

1. Every shipped map the import refused only for towns (a main town's hero, a hero in a town) imports, or a typed refusal names what is still missing; the census is in `docs/formats/h3m.md`.
2. Towns are entered and captured, by a visit or a siege whose facts are derived from the town; the siege plays the battle lane's rules on the map.
3. A town's hall pays its owner at the day's start.
4. Every rule has an id (`TWN-*`, or `ADV-`/`GAM-` where it amends one), a source and a test; the readings at risk are confirmed by batch TWN-1 or recorded as disputes.
5. The gates hold: 100 % coverage on `h3-towns` and the town modules, golden replays with a town captured, the adventure benchmarks inside their gate, the map oracle's property plans playing towns.
6. What each viewer sees of a town is in `docs/06-security.md` and tested.
7. The env plays the maps with towns (`cross-lane`).

## Increment 2 — construction

Refined with the user on 2026-11-06 (T10; `TWN-Q7`, asked at T10's start with a first look at the audit). Built like increment 1: commands, validation, events, observation, the env, replays and the gates grow with the content. The audit of increment 1's exit criteria is in `E09-log.md` (T10).

### Decisions at the refinement (2026-11-06, the user; TWN-Q7)

36. **T10 whole** (the user at T10's start, `AskUserQuestion`; recommended): the audit, the closure and this refinement in one session and one docs-only pull request, as E08's T53. Not chosen: T10a (the audit and the closure), then T10b (the refinement).
37. **Content: construction with the town events** (recommended). **The build command:** a town's owner, in its turn, builds one of the town's faction's buildings not built and not forbidden whose requirements stand, paid from the treasury, one a town a day (the original's rules as the survey reads them: the requirements and costs, one Capitol per player, the Shipyard's water and the Grail building, which the survey places); what a new building changes of increment 1's reads at once (the hall's income, the fort's siege, the Tavern's views, the silo); the map's forbidden buildings (173 custom sets forbid some, survey `E09-towns.md` §1.3). **The map's town events** (369 on 110 towns of the shipped maps: 70 build, 105 give resources, 56 add creatures, 189 carry a message only; §1.3): their buildings and resources on their days, their creatures with increment 3's growth. **The content:** each faction's requirements and costs harvested (TD-9 settled by the survey). A building's other effects come with the increments that build them (a dwelling's creatures 3, the guild and the services 4, the Grail's 5). The env plays the build command (decision 7). Not chosen: construction alone (the town events with increment 3); construction with growth and recruitment (increments 2 and 3 in one).
38. **The first round** (recommended): a survey (T12) and its checkpoint, then the ADR amendment drafts with the design note and the cut (T13), accepted before anything is built, as increment 1 (decision 7). Not chosen: the survey alone first.
39. **`ADV-STUB-13` first, a small task T11** (recommended): the random dwellings tied to a town (classes 216 and 217 whose identifier a town of the map has: 1 397 on the shipped maps, all tied to random towns, untyped blockers since E08 T93b) take the town's drawn faction at the build (`random_dwellings_resolve` 0x502B60, survey `E08-dwellings.md` §3.2; E08 decision 231), then play as E08's dwellings; `cross-lane` in the adventure lane's import and build, map import bumped, the census re-run with the original data; before increment 2's survey. Not chosen: inside increment 2; with increment 3; the adventure lane's inbox.
40. *Agent's call (change freely):* **increment 1's design note stays in `E09-design.md`** (§1 to §22; its header: the file goes when the epic closes): about 85 lines of code comments in 37 files of seven crates, in three lanes' paths (town, adventure, battle), cite it as "`E09-design.md` §N" or, in the town modules, "design §N". Increment 2's note joins the same file (T13) as its Part D, its sections numbered from §23, so no citation changes meaning. Not chosen: E08's way (the note moved into the log at the closure, its citations rewritten).

### Decisions at the survey's checkpoint (2026-11-07, the user; TWN-Q8)

41. **TD-9: the executable's list** (recommended; survey `E09-construction.md` §6, §11): the Conflux's upgraded Altar of Fire (40) needs the Altar of Fire (33) and the upgraded Altar of Air (38); a row of an experiment batch TWN-2 observes the original's hall, and its verdict overrides. Not chosen: the list without an experiment; VCMI's and the fandom wiki's (the Altar of Fire).
42. **TD-11: the town's owner builds and pays** (recommended; VCMI's rule): a TWN-2 row (two allied humans in hot seat, the visitor at the ally's hall) observes the original, which shows no owner test on the hall's path (§1.1, §1.4); revisited if it lets the visitor build. Not chosen: the owner without an experiment; the reading's (the current player in any town it visits, from its own treasury).
43. **One a town a day for every player** (recommended; §3): the computer's every other day on Easy and Normal comes with the computer player (E10, E13); the day's mark is kept across a capture and cleared when the town passes from a team without a human to a team with one, as read. Not chosen: a difficulty in the setup with the handicap now.
44. **The town events as the original plays them** (recommended; §8 to §10): at the owner's turn start, after the map's timed events, in the map's order; their days, players and kinds as the map's events'; the resources, then the buildings (the event's bits through the town's faction, less what stands, what the faction lacks, what the event's own buildings supersede and the Shipyard without its spot; no requirement, forbidden, Capitol, Grail, cost or day's-mark test; with the heroes' visit), then the creatures (the level's upgraded dwelling, else its dwelling, else lost). **T-8 is not a quirk to fix** (an event's Capitol beside the owner's other one stands); **TD-12 the original's day** (an every-day event fires from its first day, not VCMI's day before). Not chosen: one Capitol per player for events too; TWN-2 rows for both first.

### Decisions at the drafts (2026-11-07, the user; TWN-Q9)

Asked at T13's start, before the drafts were written (T13 kept whole, the user).

45. **The build is a town command** (recommended): `GameCommand::Town(h3_towns::TownCommand::Build
    { town, building })`; `h3-towns` checks the building (buildable now, the day's mark) and names
    the price, the game the turn, the owner and the treasury, then pays and applies, as
    `BuildBoat`; later town actions join `TownCommand`. No hero in the town (the original's town
    list opens the hall). Not chosen: a game command `GameCommand::Build` with each later town
    action its own variant; a pending decision opened by entering an own town.
46. **The Shipyard's water now** (recommended; `TWN-STUB-2`'s Shipyard half moved from increment
    4): the adventure fixes each town's boat spot when the game is built, as a Shipyard's tile
    (ADV-SHY-001); without one the Shipyard is forbidden from the start and a custom set's built
    Shipyard removed, as the original; the build and the town events skip it; the boat stays
    increment 4's. Not chosen: the Shipyard refused until increment 4 (the Castle's Lighthouse with
    it); buildable everywhere until then.
47. **The views as the original shows them** (the user: "this depends on what the original does;
    if a visiting ally hero sees the buildings, they should be revealed to them upon the
    visitation event"): the owner sees its town's buildings (the town list opens its town screen at
    any time); a player whose hero enters an ally's town learns them at that visit (the town screen
    opens for an allied town, survey `E09-towns.md` §7.2; with E10's teams); whether a town was
    built in today is public where the town is seen (the popup's portrait); the fort and the
    Capitol stay public, the top level's facts as they are; nobody else learns the buildings. Not
    chosen: the owner's team at any time; also at Visions' top level; the owner alone.

### Decisions at the checkpoint (2026-11-07, the user; TWN-Q10)

48. **The drafts accepted as written** (recommended): `docs/lanes/town/adr-drafts/0028-increment-2.md`,
    `0016-increment-2.md`, `0017-increment-2.md`, applied into `docs/adr/` by the `shared/` task
    T15 after batch TWN-2, with its verdicts. Not chosen: accepted with changes.
49. **The cut as proposed** (recommended; design §35): batch TWN-2 first (T14), the `shared/` task
    (T15), the part and the harvest (T16a, T16b), `h3-towns` (T17), the water, the command and the
    town events (T18a to T18c), the gates and the env (T19), the audit (T20). Not chosen: TWN-2
    alongside the build; fewer, larger tasks.
50. **The exit criteria as proposed** (recommended; below, design §35).
51. **The agent's calls kept** (recommended; design §26 to §31): the hall's offer (a horde as its
    upgraded horde), the day's mark a flag, the town's boat spot as the town object's
    `Resolved::BoatTile` fixed after the random dwellings resolve (until the build task reads the
    original's order), one `TownChanged` for the town's seen facts; **T-9 fixed without a flag**
    (the event loop's bits 41 to 43 read past the table: `TWN-QUIRK-7` with the build). Not chosen:
    T-9 with a `faithful` flag; changes now.

### Decisions at T18b2's start (2026-11-10, the user; SRV-Q26)

52. **T18b2 cut in two:** T18b2a the views and events of §31 with SRV-Q26's town gap; T18b2b the
    legal builds, `town_offer`, the bench gate, the plans building, the rollout's choice. Not
    chosen: three parts (the gap a task of its own); T18b2 whole.
53. **Every viewer's changed view of a town told to it** (the agent's recommendation, after the
    user's first pick was a misreading): a small `TownChanged` with the view's facts, then a
    `TownStackSeen` per shown stack and a `TownIncomeSeen` per income line (`GameEvent` stays
    `Copy`), after a build, a capture, a siege's merge, a garrison's battle and the day's start; it
    replaces decision 51's one `TownChanged` for the seen facts and covers a neutral garrison
    emptied by another player's battle and a new Tavern's raised levels. Not chosen: the whole view
    in one `TownChanged` (fixed arrays, every event larger); a notice and a query.
54. **Visions' reach stubbed** (`TWN-STUB-13`, the server lane told): a hero's step, army or Power
    moving another team's town in or out of its viewer's top level is not told. Not chosen: each
    Visions holder's reach compared after every command now.

### Decisions at T18b2b's start (2026-11-10, the user)

55. **T18b2b whole** (recommended): the listing, `town_offer`, the bench gate, the plans building
    and the rollout's choice in one session and one `cross-lane` pull request. Not chosen: two
    parts (the listing, the offer, the rollout and the bench; then the plans' new checks).
56. **The env's rollout skips the builds until T19** (recommended): its random agent draws from
    the listed commands but a town's build, which has no action before T19's build entry (drawn,
    it would end the turn); nothing in the env moves; T19 drops the skip with the entry. Not
    chosen: kept, a drawn build ending the turn (the town map's random games moved).

### Decisions at T18c's start (2026-11-10, the user)

57. **T18c whole, the plans' checks to T19** (recommended): the game's town events, their wire
    mapping and day 1's in one session and one `cross-lane` pull request; the plans'
    `check_towns` and `check_rewards` taught the events' builds and gains in T19, with the writer
    that first gives the plans' towns events (the writer writes none before T19, so checks taught
    now would run on nothing). Not chosen: as queued (the checks now, unexercised); two parts (the
    resources, `TownEventFired` and day 1; then the buildings, the views and the checks). The
    user also had the hygiene property's CI counterexample fixed first (#822).

### Decisions at T19's start (2026-11-10, the user)

58. **T19 cut in three** (recommended, as increment 1's T9, decision 32): **T19a** the writer's
    town events, the map and its scripted golden replays; **T19b** the plans (the step `Build`,
    `check_towns` and `check_rewards` taught the events, decision 57, the legal-builds,
    refused-build, told-views, hygiene and mark-clearing properties); **T19c** the env's build
    entry (the AI lane's action encoding), the rollout's skip dropped (decision 56), random play
    with builds, the benchmarks and the census. Not chosen: the plans folded into T19a (two
    parts); T19c cut again (the env and random play; the benchmarks and the census).
59. **Increment 2's own map** (recommended): `fixtures/maps/e09-builds-36.h3m` carries the Castle
    by water and the one inland, the Conflux, the town events and the custom set forbidding a
    requirement, so that the town map `e09-towns-36.h3m` and its replays, the env's seeds and the
    four `towns-36` workloads stay as they are (design §32 had the town map gain them). The env's
    and the benchmarks' builds (T19c) may use either map. Not chosen: the town map extended (eight
    replays re-recorded, the env's seeds searched again, four workloads re-set).

### Decisions at T19b's start (2026-11-10, the user)

60. **T19b whole** (recommended): the plans' towns given events, the step `Build` with runs of
    builds, `check_timed` and `check_towns` taught the events' gains and builds, the towns' cover
    test counting them, the hygiene and told-views properties with both, in one session and one
    `cross-lane` pull request. Not chosen: two parts (the events and their checks; then the step
    and the views).
61. **`master`'s counterexample of the plans' gains fixed first** (recommended): the adventure
    lane's inbox entry of T19a (a guarded Mystical Garden's gain expected before the joining
    monster's treasure), reproduced on `master`'s strategy and fixed in the same pull request,
    since T19b's new plan field changes the draws and its seed then replays another case. Not
    chosen: left to the adventure lane.

### Decisions at T19c's start (2026-11-10, the user)

62. **T19c cut in two, the env first** (recommended): **T19c1** the env's build entry and
    building head (the AI lane's action encoding v9), the rollout's skip dropped (decision 56), a
    test playing the town map to its end of days with builds, random replays with builds, the
    town map's workloads re-baselined where their random games moved, the AI lane's inbox told;
    **T19c2** the new benchmark workloads against `master` and the census of the 160 shipped maps
    with the original data, the map client's inbox told. Not chosen: the census and the new
    workloads first (the env's random games would then move the workloads a second time); T19c
    whole (one long session, one pull request over the AI, adventure and server lanes' paths).

### Decisions at T19c2's start (2026-11-11, the user)

63. **T19c2 whole** (the agent recommended the census first, while the original data were
    staged, and the workloads in a later session, which needs no original data): the census of
    the 160 shipped maps, the new benchmark workloads against `master` and the map client's inbox
    in one session and one `cross-lane` pull request (the benches README's rows in a `shared/`
    one). Not chosen: the census now and the workloads next; the workloads first.

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, each task's cut confirmed at its start)

- **T11 The random dwellings tied to a town** (`ADV-STUB-13`, decision 39; `cross-lane`). Done 2026-11-06 (map import v20; `E09-log.md`).
- **T12 The construction survey** (decision 38; survey `E09-construction.md`), cut in two at its start (the user, 2026-11-06): **T12a** the build command's checks, what a build does, the cost and requirement tables, VCMI and the wiki compared: done 2026-11-07 (§0 to §7; TD-9 kept, TD-11 new; `E09-log.md`). **T12b** the town events, the census of the shipped maps, the disputes and quirk candidates of both parts, the checkpoint with the user: done 2026-11-07 (§8 to §12; TD-12 and T-8 new; `TWN-Q8`, decisions 41–44; `E09-log.md`).
- **T13 The ADR amendment drafts, the design note (Part D of `E09-design.md`, §23 to §36) and the cut**, kept whole at its start (the user): done 2026-11-07 (`TWN-Q9` before the drafts, decisions 45–47; the checkpoint `TWN-Q10`, decisions 48–51; `E09-log.md`).
- **T14 Batch TWN-2** (design §33; TD-9's and TD-11's rows), cut in three at its start (the user, 2026-11-07, as TWN-1's T3a to T3c): **T14a** the spec and template and **T14b** the harness's hall path, the recipe and two runs: done 2026-11-07 (`experiments/results/E09-batch-2a.md`, `-run2.md`; `E09-log.md`). **T14c** the scoring, kept whole at its start (the user): done 2026-11-07 (survey `E09-construction.md` §13: TD-9 as the reading, TD-11 refused by the build dialog's owner test; decisions 41 and 42 stand; the town screen's display pieces, nothing to build, no `T-10`; `E09-log.md`).
- **T15** `shared/`: the drafts into `docs/adr/` with TWN-2's verdicts, `docs/03`'s terms, `docs/06-security.md`'s words (design §35), kept whole at its start (the user): done 2026-11-07 (ADR-0028 points 2, 5, 6, 9; ADR-0016 points 1, 4, 6; ADR-0017 points 1 to 4, 6 to 10; the drafts deleted; `E09-log.md`).
- **T16a** the part's requirements, costs and the role `shipyard` (§24, part v4), kept whole at its start: done 2026-11-07 (part v4; the adventure replays' `town_ruleset_hash` line, `cross-lane`; `E09-log.md`). **T16b** their harvest (§25, `cross-lane`, original data), kept whole at its start: done 2026-11-07 (the base and AB packs re-harvested; the adventure replays' `town_ruleset_hash` line, `cross-lane`; `E09-log.md`).
- **T17** `h3-towns`: the day's mark, the store's writes, the rules, `TownCommand` (§26, §27), cut in two at its start (the user, 2026-11-09): **T17a** the mark, `built_with` and the store's writes: done 2026-11-09 (town state hash v2, game state hash v26, `cross-lane`; TWN-BLD-004, -005; `E09-log.md`). **T17b** the rules that decide a build, `TownCommand` and its errors, the other catalog entries, `TWN-QUIRK-7`: done 2026-11-09 (no version moved; TWN-BLD-001 to -003, -006, -007, TWN-EVT-003; `E09-log.md`).
- **T18a** the Shipyard's water (§28), kept whole at its start (the user, 2026-11-09): done 2026-11-09 (adventure ruleset part v19, `cross-lane`; TWN-SET-011; `E09-log.md`); **T18b** the command, the legal builds, the day's start, the capture's mark, the views (§29 to §31), cut in two at its start (the user, 2026-11-09): **T18b1** the command, the day's start, the capture's mark: done 2026-11-09 (adventure replay format v11, `cross-lane`; TWN-BLD-001, -004 the game's halves; `E09-log.md`); **T18b2** the legal builds and the views, cut in two at its start (the user, 2026-11-10; decisions 52 to 54): **T18b2a** the views and events with SRV-Q26's town gap: done 2026-11-10 (no version moved, `cross-lane`; TWN-VEW-001 amended, TWN-VEW-004, `TWN-STUB-13`; `E09-log.md`); **T18b2b** the legal builds and `town_offer`, kept whole at its start (the user, decisions 55, 56): done 2026-11-10 (no version moved, `cross-lane`; TWN-BLD-001 amended; `E09-log.md`); **T18c** the town events at the turn start (§30), kept whole at its start with the plans' checks moved to T19 (the user, decision 57): done 2026-11-10 (protocol v41, `cross-lane`; TWN-EVT-001, -002, -004, TWN-EVT-003's game half, GAM-CAL-004 amended, `TWN-STUB-10`; `E09-log.md`).
- **T19** the writer's town events, fixtures, replays, properties (with the plans' `check_towns` and `check_rewards` taught the events' builds and gains, decision 57), benchmarks, the env, the census (§32), cut in three at its start (the user, decisions 58, 59): **T19a** the writer's events, the builds map and its scripted replays: done 2026-11-10 (no version moved, `cross-lane`; `writer::TownEvent`, `e09-builds-36.h3m`, `builds-week-s1`, `-capture-s1`, `-events-s1`; `E09-log.md`); **T19b** the plans, kept whole at its start with `master`'s counterexample of the plans' gains fixed first (the user, decisions 60, 61): done 2026-11-10 (no version moved, `cross-lane`, tests only; `E09-log.md`); **T19c** the env, random play with builds, the benchmarks, the census, cut in two at its start (the user, decision 62): **T19c1** the env and random play: done 2026-11-10 (the AI lane's action encoding v9, `cross-lane`; `builds-s262-d14`, `-s271-d14`, `-s290-d14`; three workloads re-baselined; `E09-log.md`); **T19c2** the new benchmark workloads and the census, kept whole at its start (the user, decision 63): done 2026-11-11 (no version moved, `cross-lane`; `legal-towns-36`, `day-start-builds-36`; the census of exit criterion 6 in `h3-map`'s `original_data`; `E09-log.md`).
- **T20** the audit against the exit criteria; increment 2 closed; increment 3 refined.

### Exit criteria

Accepted at T13's checkpoint (decision 50; design §35; increment 1's: `E09-archive.md`):

1. A town's owner, in its turn, builds a building of its town's faction that the original's hall
   offers it (its requirements in the ever-built set by the executable's lists, not forbidden, the
   Shipyard only with water, one Capitol a player, the hordes as their upgrades), one a town a day,
   paid from its treasury at `Building.txt`'s costs; every other build is refused, typed, changing
   nothing (TWN-BLD-001 to -007, TWN-SET-011).
2. A build changes increment 1's reads at once: the hall's income, the fort and the siege's facts,
   the Tavern's view levels, the silo, the Capitol (tests per role).
3. The map's town events fire as the original's: their days, players and kinds, at their owner's
   turn start after the map's timed events, in the map's order; their resources, then their
   buildings without the hall's tests; their creatures registered for increment 3 (TWN-EVT-001 to
   -004; T-8 and TD-12 as decided).
4. The views are the original's: the owner sees its buildings, everyone who sees the town its
   day's mark, fort and Capitol; nothing else leaks (the hygiene and told-views properties with
   builds; `docs/06-security.md`).
5. The env plays the build; the plans build; golden replays cover a week of builds, a capture
   after a build and the town events, and the builds map's test each refusal (a replay holds no
   refused command, T19a); determinism and the forks' sharing hold (a build copies one town)
   (worded as built by T20).
6. With the original data all 160 shipped maps import, build and play their town events over
   their first days; the census (the Shipyards removed for want of water, the events fired, built
   and skipped) is recorded against the survey.
7. Batch TWN-2 is scored and its verdicts applied (TD-9, TD-11).
8. The gates hold (100 % coverage, the scoped verify, CI's full verify); the versions are bumped and
   recorded; the other lanes' inboxes are told (the AI lane's encoding, the map client's town
   screen, the server lane's wire for the command and the events); the server's manifest carries
   the part's hash since its own E16 T4 (worded as built by T20).
