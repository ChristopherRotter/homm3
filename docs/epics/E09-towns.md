# E09 — Towns

Status: **in progress** (refined 2026-10-31 with the user, TWN-Q1; the town lane opened the same day). Increment 1 (towns on the map) closed on 2026-11-06 (T10; its scope, decisions 8–35 and tasks in `E09-archive.md`); increment 2 (construction) runs in rounds; the position and the next task are in `docs/lanes/town/HANDOFF.md` and `QUEUE.md`. The record of every done task: `E09-log.md`.

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
2. **Construction:** each faction's buildings and their requirements and costs (harvested), one building a day, the hall and fort levels, the forbidden buildings, the town events' buildings (refined below).
3. **Growth, recruitment and the Tavern:** the weekly growth (the dwellings, the castle's bonus, the hordes, the week's creature and the Plague), recruitment into the garrison or the visiting hero, the town's two army rows and the heroes' swap, the Tavern's hire with its pool and the eight heroes' limit, the Tavern's information and the Thieves' Guild.
4. **The mage guild and the services:** the guild's spells drawn (the town chances, the map's required and banned spells), learning them (Wisdom), the spell points restored; the Marketplace and the Resource Silo, the Blacksmith, the town's Shipyard (E08's `BuildBoat`), the Artifact Merchants and the other faction services; the special buildings for heroes and sieges (the Castle's Lighthouse, A-30; the Necropolis's Necromancy amplifier, the queue archive's T19a2b note; Cover of Darkness).
5. **Travel and the Grail:** Town Portal and Castle Gate; the obelisks, the puzzle map, digging and the Grail buildings.

## Closed increments

Their scope, decisions and task lists are in `E09-archive.md` (cited as "E09 decision N"; `git grep -n 'decision 22' docs/epics/E09-archive.md`); increment 1's design note stays in `E09-design.md` (decision 40), the record of every task in `E09-log.md`. Decisions 1–7 above and those from 36 below stay here.

| Increment | Closed | Tasks | Decisions | Content |
|---|---|---|---|---|
| 1 | 2026-11-06 | T0–T10 | 8–35 | towns on the map: fixed and random towns imported and built, the players' factions and the starting-hero pick, the hero pool with the main towns' and the random heroes, the town's record and store, entering, capture and the siege with its facts from the town and the garrison's merge, the day's income with the silo, the views by the original's levels, batch TWN-1; all 160 shipped maps import and build |

## Increment 2 — construction

Refined with the user on 2026-11-06 (T10; `TWN-Q7`, asked at T10's start with a first look at the audit). Built like increment 1: commands, validation, events, observation, the env, replays and the gates grow with the content. The audit of increment 1's exit criteria is in `E09-log.md` (T10).

### Decisions at the refinement (2026-11-06, the user; TWN-Q7)

36. **T10 whole** (the user at T10's start, `AskUserQuestion`; recommended): the audit, the closure and this refinement in one session and one docs-only pull request, as E08's T53. Not chosen: T10a (the audit and the closure), then T10b (the refinement).
37. **Content: construction with the town events** (recommended). **The build command:** a town's owner, in its turn, builds one of the town's faction's buildings not built and not forbidden whose requirements stand, paid from the treasury, one a town a day (the original's rules as the survey reads them: the requirements and costs, one Capitol per player, the Shipyard's water and the Grail building, which the survey places); what a new building changes of increment 1's reads at once (the hall's income, the fort's siege, the Tavern's views, the silo); the map's forbidden buildings (173 custom sets forbid some, survey `E09-towns.md` §1.3). **The map's town events** (369 on 110 towns of the shipped maps: 70 build, 105 give resources, 56 add creatures, 189 carry a message only; §1.3): their buildings and resources on their days, their creatures with increment 3's growth. **The content:** each faction's requirements and costs harvested (TD-9 settled by the survey). A building's other effects come with the increments that build them (a dwelling's creatures 3, the guild and the services 4, the Grail's 5). The env plays the build command (decision 7). Not chosen: construction alone (the town events with increment 3); construction with growth and recruitment (increments 2 and 3 in one).
38. **The first round** (recommended): a survey (T12) and its checkpoint, then the ADR amendment drafts with the design note and the cut (T13), accepted before anything is built, as increment 1 (decision 7). Not chosen: the survey alone first.
39. **`ADV-STUB-13` first, a small task T11** (recommended): the random dwellings tied to a town (classes 216 and 217 whose identifier a town of the map has: 1 397 on the shipped maps, all tied to random towns, untyped blockers since E08 T93b) take the town's drawn faction at the build (`random_dwellings_resolve` 0x502B60, survey `E08-dwellings.md` §3.2; E08 decision 231), then play as E08's dwellings; `cross-lane` in the adventure lane's import and build, map import bumped, the census re-run with the original data; before increment 2's survey. Not chosen: inside increment 2; with increment 3; the adventure lane's inbox.
40. *Agent's call (change freely):* **increment 1's design note stays in `E09-design.md`** (§1 to §22; its header: the file goes when the epic closes): about 85 lines of code comments in 37 files of seven crates, in three lanes' paths (town, adventure, battle), cite it as "`E09-design.md` §N" or, in the town modules, "design §N". Increment 2's note joins the same file (T13) as its Part D, its sections numbered from §23, so no citation changes meaning. Not chosen: E08's way (the note moved into the log at the closure, its citations rewritten).

### Tasks (outline; queue entries in `docs/lanes/town/QUEUE.md`, each task's cut confirmed at its start)

- **T11 The random dwellings tied to a town** (`ADV-STUB-13`, decision 39; `cross-lane`). Queued.
- **T12 The construction survey** (decision 38): the code reading of the build command's checks, the requirements and costs (`building_tables_init` 0x4EB730 and `town_build` 0x5BE930, read in part by T5b1, survey §16), the day's limit and the town events' day; VCMI and the wiki compared; a census of the shipped maps (forbidden buildings, custom sets, the town events); the disputes (TD-9 first) and quirk candidates; the checkpoint with the user. Queued after T11.
- **T13 The ADR amendment drafts, the design note (Part D of `E09-design.md`, from §23) and the cut**, accepted by the user before the build.
- The build's rounds, queued as the round before is done; the audit last.

### Exit criteria

Written by T13's design note and accepted at its checkpoint (increment 1's: `E09-archive.md`).

## Risks

- **Two lanes in the same crates:** the town modules sit in the adventure lane's crates, and their common parts, versions and replays change in both lanes; merges in order, a rebase and a second re-recording when both bump. Check the adventure lane's open pull requests at the start of every task.
- **The size of the content:** nine factions of about forty buildings each, with requirements read from the executable; the harvest is in the battle lane's crate.
- **Editions:** the moat (ADR-0023), Conflux and the AB buildings, RoE's differences (E11).
- **The faction choice touches E10's scenario setup** (decision 6): the setup's faction per player is the seam; the rest of the scenario setup stays E10's.
- **Other lanes read the town:** the env (AI lane), the map client (E24 waits for towns), the server's wire for new commands and events; notes to their inboxes when a town change lands.
