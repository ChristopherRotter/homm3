# Draft: ADR-0028 amended for E09 increment 4 (the Tavern)

- Status: **Draft**, the hire's half written by E09 T33a (2026-11-16), the information's half by
  T33b (2026-11-16, the Thieves' Guild), asked at T33b's checkpoint (`TWN-Q18`) before anything
  is built; then moved into `docs/adr/0028-town-context.md` by the increment's `shared/` task
  with batch TWN-4's verdicts (as increments 2's T15 and 3's T24).
- Decisions: the epic's 99 (content), 103 to 107 (the survey's checkpoint, `TWN-Q16`), 108 (T33
  in two), 109 to 112 (`TWN-Q17`: a hero type's record at its first draw, the tavern weights in the
  town part, the hire's two shapes, the dismissal), 113 (T33a's agent's calls), 114 (T33b whole)
  and `TWN-Q18`'s.
- Working detail: the survey `docs/rules/surveys/E09-tavern.md` (§2 to §6, §12, §13), the design
  note `docs/epics/E09-design.md` Part F (§59 to §71 the hire, §72 to §80 the information, the
  views, the env, the cut). Each block below says where it goes in the ADR.

## Point 5, after increment 3's paragraph

**Increment 4** (the Tavern; E09 decisions 99, 110) gives each town record its **tavern
weights**: per hero class of the core (by key) the weight of that class in a tavern's draw for a
player of the record's faction (`HCTRAITS.TXT`'s tavern column of the faction: 5 or 6 a class in
this build's data, each column summing to 100; survey `E09-tavern.md` §2.2). A class the record
does not list weighs 0; the compiler refuses a class the core lacks, a class listed twice and a
record whose weights sum to 0. An edition without Conflux has no Conflux record, so RoE's
missing column needs no override (the inbox's E04a T3a note). The parameters gain the **hire's
price** (resources of the core: 2 500 gold, `hero_price` 0x67814C, the same for every hire and
every edition; tier-2 data of the reading). The role `tavern` (increment 1) names the building
that opens a town's tavern; `most_heroes` (increment 3) bounds the hire. The content: the
weights harvested from both LODs' `HCTRAITS.TXT` (the base pack's eight records, the AB pack's
Conflux; `h3-harvest`, `cross-lane`), the price tier-2 data.

## Point 6, after increment 3's recruitment

**Increment 4's hire** (TWN-TAV-003; E09 decision 111) adds the town command's fourth variant,
**`TownCommand::Hire { town, slot }`** (the town screen's tavern: `town_tavern_click` 0x5D82B0,
`town_tavern_hire` 0x5C12E0): `h3-towns` checks the town's part (a standing building of role
`tavern`, `has_tavern`) and names the price (the part's), each refusal typed
(`TownCommandError`); the adventure checks the entrance (no visitor) and places the hired hero
there; the game checks the turn, the owner, that no decision is open, the slot's offer, the
player's heroes on the map against `most_heroes` and the treasury, then pays, places, marks the
pool and draws the slot's replacement, in one infallible application (ADR-0017 point 10). The
adventure map's Tavern (class 95) hires through the game's tavern decision (ADR-0017 point 6),
not a town command: `h3-towns` takes no part in it but the part's price. No new contract type:
the slot is a plain number (0 or 1).

## Point 8, replaces its text

**The hero pool is the game's** (decision 18; ADR-0017 point 1): `h3-towns` holds no hero, no
pool and no tavern heroes. A town's **Tavern** is a building of role `tavern` whose hire the game
composes (point 6) from the pool, the players' two tavern heroes each (the survey's offers), the
store of heroes and the adventure's placement; the draw's weights are the part's (point 5).
**The Thieves' Guild** (increment 4's information; TWN-TAV-007 to -009; E09 decisions 6, 99) is a
projection the game composes (ADR-0017 point 2), as a town's view: its level is the number of the
viewer's towns where `has_tavern` holds (a standing building of role `tavern`, the Castle's
Brotherhood of the Sword among them; `player_tavern_count` 0x4CCE30), counted to 5; `h3-towns`
gives that predicate and each town's garrison, nothing else. The rows' values and the per-player
parts read what the game holds (the treasuries, the day's gold, the store of heroes) and the
adventure's owners, heroes and armies; no town state moves.

## Point 9, after increment 3's sentence

**Increment 4** bumps the town ruleset part (v7: the tavern weights, the hire's price); the town
state hash does not move (nothing of the tavern is the town's). The adventure lane's versions it
moves (`cross-lane`): the game state hash (the pool's statuses and records, the players'
factions and tavern heroes), the adventure ruleset part and state hash (the kind `Tavern`), the
map import (class 95, the map-wide hero entries in the setup), the adventure replay format (the
hire, the tavern decision's answers, the dismissal); the protocol (the server lane's: the hire,
the dismissal, the tavern decision, the tavern heroes' view and its events, a hired hero's
appearance, the Thieves' Guild's request and a Den's view) and the env's action encoding (the AI
lane's: the town's hire, then the tavern decision's answers); each `cross-lane`. The Den of
Thieves' kind moves the adventure ruleset part and the map import again (class 97) with the
Tavern's, in one task.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 4's `shared/` task; decisions 99, 103 to 113 and T33b's): points 5,
  6, 8, 9 (the tavern weights and the price, the hire, the pool's tavern heroes, the Thieves'
  Guild's level by the taverns).
