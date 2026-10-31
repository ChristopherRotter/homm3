# E09 — Towns

Status: **in progress** (refined 2026-10-31 with the user, TWN-Q1; the town lane opened the same day). Increment 1 (towns on the map) runs in rounds; the position and the next task are in `docs/lanes/town/HANDOFF.md` and `QUEUE.md`. The record of every done task: `E09-log.md`.

## Goal

Towns as the original plays them, on the adventure map of E08: the town context in its own domain crate `h3-towns` (buildings and construction, growth and the available creatures, the mage guild, the market and the other services), the town as the adventure's map object (owner, garrison, visiting hero, entering, capture by siege), composed by `h3-game`. Built like E08: each increment is complete in structure (commands, validation, events, views, the env, replays, the gates) and grows the content; RoE, AB and SoD for what it builds (E11: the edition of the pack set). The first increment unlocks the shipped maps that wait only for towns: 128 of the 160 are refused today for a hero generated at a main town (`docs/formats/h3m.md`, "Import").

## Decisions at the refinement (2026-10-31, the user, TWN-Q1)

1. **A lane of its own, now.** E09 opens at once in a new lane `town` (code `TWN`), in parallel with the adventure lane's E08 (increment 9's rounds 6 to 8, then its sketched increments 10 to 12). Not chosen: E09 after E08's increment 9 with E08 closing (its increments 10 to 12 split between E09 and E12); after increment 9 with E08 kept open; after E08's increments 10 to 12.
2. **The crate `h3-towns`** (the user's name; ADR-level, the details in increment 1's ADR draft): a domain crate holding the town context's own state and rules (buildings, construction, growth, the available creatures, the mage guild, the market and the services). **The town as a map object stays the adventure's** (its tiles and entrance, owner and flag, garrison, visiting hero), as a mine and its guards; **`h3-game` composes** the visit, recruitment into an army, the town's part of the day and the week, and the siege's derived facts (ADR-0023). The ADR draft decides where the town's state lives in the game state (its fork and hash), its ruleset part and pack file, the seam types and the hero pool's home. Not chosen: a module inside `h3-adventure`, split out later; `h3-towns` owning the town's armies too.
3. **Ownership: town sub-paths** (`lanes.toml`). The town lane owns `h3-towns` and the town modules inside the adventure lane's crates by the longest match: `h3-adventure`'s and `h3-game`'s `src/town*`, `h3-map`'s `src/import/town*` and `src/writer/town*`; the town rules (`docs/rules/catalog/town.toml`), its quirks ledger `docs/rules/quirks-town.md`, E09's epic, surveys and experiments. Those crates' common parts (`lib.rs`, the state structs and hashes, the versions, the replays and the bot reports) stay the adventure lane's: the town lane changes them in a `cross-lane` pull request, merged in order with the adventure lane's (the second rebases and bumps again). The scoped verify `--scope town` runs the adventure lane's crates, replays and workloads too (`verify_with`). Not chosen: the town lane owning only `h3-towns` with every other change `cross-lane` by consent; the adventure lane building the map side from its inbox; increment 1 reordered to the town's own rules first.
4. **E08's sketched increments 10 to 12 stay E08's** (dwellings, random dwellings and the Refugee Camp with hiring, garrisons; creature banks; quests). The hire dialog and the weekly growth are built once, by whichever lane needs them first, and the other reuses them (a note in the other lane's inbox when they land). Not chosen: dwellings, hiring and garrisons moved to E09; E08 closing after its increment 9.
5. **Increment 1: towns on the map** (below). Not chosen: the economy loop first (construction, growth, recruitment), capture and the main town's hero in increment 2; both in one increment.
6. **E09 also owns** (all four of the border topics offered): **the player's faction choice at the start and random towns** (class 77) resolved from it (otherwise E10's scenario setup); **Town Portal and Castle Gate** (the town-travel spell and the Inferno building; the other adventure spells stay unplaced, E08's "Not placed yet"); **the Grail** (obelisks, the puzzle map, digging and the Grail buildings); **the Tavern's information and the Thieves' Guild** (what a player learns of the others by taverns owned; a projection).
7. *Agent's calls (change freely):* each increment is built as E08's are: a survey (the code reading of `Heroes3.exe`, the wiki and VCMI, a census of the shipped maps), the ADR drafts with a design note and the cut, the user's acceptance before the build, an experiment batch for the readings at risk, the build in rounds, the replays and gates, an audit of the exit criteria; a user checkpoint per increment and the cut confirmed at each round, as a domain lane. The env plays a new town command in the same increment, `cross-lane` in the AI lane's paths (as E08 decision 203). Experiments are files `experiments/specs/E09-batch-<n><x>.toml`, called "batch TWN-<n>" in prose to keep them apart from E08's "batch E9". The computer player's town play is E10's and E13's.

## Starting point (2026-10-31)

- **The map:** `h3-map` reads the towns' records (classes 77 and 98: owner, name, garrison and formation, the built and forbidden buildings or the has-fort flag, the required and banned spells, the town events, SoD's alignment byte; `docs/formats/h3m.md`'s object table). The import keeps a town as an untyped blocker (its footprint; a hero leaving a town's gate leaves upward, the queue archive's T14a note) and refuses a hero generated at a main town (128 maps) and a hero standing on a town's blocked tile (`Unsupported::HeroInTown`, 3 maps). The writer writes towns of a fixed faction (E04c T5b) for experiment maps.
- **The battle:** a siege plays from `BattleSetup.siege` (ADR-0023: the faction, the fort level, the moat by edition, the town's battle facts), given today by scenarios and the CLI; whoever starts a battle at a town derives them ("the town context, E09"). `Defender::Garrison` (E08 T84a) fights a mine's guards.
- **The game:** the store of heroes (ADR-0017; "the pool and the tavern are E09's"), the day's income (GAM-RES-004), the month's kind with the Plague, which changes nothing before towns (ADR-0017 point 4), the week's creature waiting for E09 and drawn from a stream of its own (E08 decision 197), `ADV-STUB-9` (a Tavern town to hire a retreated hero again), flags and owned objects (E08 increment 9).
- **Content:** factions are core content; no building, dwelling or town service is harvested yet, nor the spells' town chances or the hero classes' tavern weights (E04a T3a's note).
- **Notes from earlier work** for E09 are gathered in the town queue's inbox (`docs/lanes/town/QUEUE.md`).

## Increments (sketch; the order and the cut of each decided at its refinement, as E08 decision 48)

1. **Towns on the map** (refined below).
2. **Construction:** each faction's buildings and their requirements and costs (harvested), one building a day, the hall and fort levels, the forbidden buildings, the town events' buildings.
3. **Growth, recruitment and the Tavern:** the weekly growth (the dwellings, the castle's bonus, the hordes, the week's creature and the Plague), recruitment into the garrison or the visiting hero, the town's two army rows and the heroes' swap, the Tavern's hire with its pool and the eight heroes' limit, the Tavern's information and the Thieves' Guild.
4. **The mage guild and the services:** the guild's spells drawn (the town chances, the map's required and banned spells), learning them (Wisdom), the spell points restored; the Marketplace and the Resource Silo, the Blacksmith, the town's Shipyard (E08's `BuildBoat`), the Artifact Merchants and the other faction services; the special buildings for heroes and sieges (the Castle's Lighthouse, A-30; the Necropolis's Necromancy amplifier, the queue archive's T19a2b note; Cover of Darkness).
5. **Travel and the Grail:** Town Portal and Castle Gate; the obelisks, the puzzle map, digging and the Grail buildings.

## Increment 1 — towns on the map

### Scope

- **The import and the start:** towns of a fixed faction (98) and random towns (77); the players' faction choice at the start and a random town resolved from it (the original's rule for a neutral random town: the survey's); the town's record into the state (owner, garrison and formation, the buildings built at the start as the map sets them, its town events kept for the increments that apply them); **the main town's generated hero** with the hero pool it draws from (the faction's classes, the map's allowed heroes; the original's draw read); a hero placed on a town's blocked tile moved to the town's entrance, as VCMI's "H3M format quirk" (3 maps).
- **The town on the map:** its entrance entered (the town classes typed: a visit from the entrance tile, the gate's leaving); the visiting hero; a player's own town visited (no service yet); **capture**: an unguarded town of another player or of none flagged by its visitor, a garrisoned or visited one taken by a **siege** with ADR-0023's facts derived from the town (faction, fort level, the moat by edition) against the garrison or the visiting hero (`Defender::Garrison`), the outcome applied (the survivors, the flag); the flag's reveal (E08's flag radius).
- **The day's income** of a town's hall (and the Capitol's limit, if the survey finds one in play in increment 1), read by the day's start (E08 increment 9's path).
- **What a viewer sees** of a town (owner, faction, the fort level, the garrison as the original shows another's), written into `docs/06-security.md`.
- **Content** for what it reads: the buildings' identities per faction, the hall and fort levels, the hero classes' tavern weights; the harvest `cross-lane` (`h3-harvest` is the battle lane's) or a `shared/` task (E08 T81b's way).
- **Not in it:** construction, growth and recruitment, the Tavern's hire, the mage guild and every other service (increments 2 to 4); a player's defeat for losing all towns (E10); a computer player's town play (E10, E13).

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, the cut confirmed at each checkpoint)

- **T0 E09 refined, the town lane opened.** Done (2026-10-31, `shared/town-lane`; decisions 1–7; TWN-Q1).
- **T1a The towns survey, part 1:** the census of towns on the 160 shipped maps (classes 77 and 98, owners, random towns, main towns and their generated heroes, preset buildings, garrisons, visiting heroes, town events) and the reading of the start: the faction choice, random towns, the main town's hero and the pool.
- **T1b The towns survey, part 2:** the visit, capture and the siege's derivation, the hall's income, the views; the disputes and quirk candidates; the checkpoint's questions.
- **T2 The ADR drafts, the design note and the cut:** a new ADR for `h3-towns` (the crate, the town's state and its home, the ruleset part and pack file, the seams, the pool), the amendments to ADR-0015 to -0017 and ADR-0023; `E09-design.md`; the build's rounds; the checkpoint.
- **T3 Batch TWN-1:** the maps and specs, the harness's town reads, the runs, the scoring (as E08 T78 to T80).
- **Later rounds** (cut at T2's checkpoint): the shared task (the ADRs accepted, the crate created, the core content), the content and harvest, the import, the state, the visit and capture, the income, the views, the env (`cross-lane`), the replays and gates, the audit with increment 2's refinement.

### Exit criteria

1. Every shipped map the import refused only for towns (a main town's hero, a hero in a town) imports, or a typed refusal names what is still missing; the census is in `docs/formats/h3m.md`.
2. Towns are entered and captured, by a visit or a siege whose facts are derived from the town; the siege plays the battle lane's rules on the map.
3. A town's hall pays its owner at the day's start.
4. Every rule has an id (`TWN-*`, or `ADV-`/`GAM-` where it amends one), a source and a test; the readings at risk are confirmed by batch TWN-1 or recorded as disputes.
5. The gates hold: 100 % coverage on `h3-towns` and the town modules, golden replays with a town captured, the adventure benchmarks inside their gate, the map oracle's property plans playing towns.
6. What each viewer sees of a town is in `docs/06-security.md` and tested.
7. The env plays the maps with towns (`cross-lane`).

## Risks

- **Two lanes in the same crates:** the town modules sit in the adventure lane's crates, and their common parts, versions and replays change in both lanes; merges in order, a rebase and a second re-recording when both bump. Check the adventure lane's open pull requests at the start of every task.
- **The size of the content:** nine factions of about forty buildings each, with requirements read from the executable; the harvest is in the battle lane's crate.
- **Editions:** the moat (ADR-0023), Conflux and the AB buildings, RoE's differences (E11).
- **The faction choice touches E10's scenario setup** (decision 6): the setup's faction per player is the seam; the rest of the scenario setup stays E10's.
- **Other lanes read the town:** the env (AI lane), the map client (E24 waits for towns), the server's wire for new commands and events; notes to their inboxes when a town change lands.
