# Draft: ADR-0017 amended for E09 increment 6 (the town's trades)

- Status: **Draft**, written by E09 T53 (2026-11-25), to be accepted at its checkpoint (`TWN-Q37`)
  before anything is built; then moved into `docs/adr/0017-game-orchestration.md` by the
  increment's `shared/` task (T55a) with batch TWN-6's verdicts.
- Decisions: the epic's 197, 198, 200 to 207 (`TWN-Q35`: D-70 the original, TD-28 `fix-optional`
  as T-28, TD-29 the trader's count and at least 1, TD-30 with T-23 `fix-optional`, T-22 fixed,
  TD-31 the original, T52's agent's calls), 208 to 212 (`TWN-Q36`: T53 whole, the market and the
  University as E08's decisions opened by town commands, `BuyWarMachine` one-shot, the stock and the
  pool's marks in the adventure state), 213 (T53's agent's calls, asked at `TWN-Q37`: **the towns'
  stream** for the month's draw over a named stream of its own, the combinations' costs in the
  core's compile, the sale's env entries).
- Working detail: survey `docs/rules/surveys/E09-trades.md` §2 to §7, §11; design note
  `docs/epics/E09-design.md` Part H §101 to §120.

## Point 1, after E09's increment 5 paragraph ("**E09's increment 5** (the mage guild; …", its build ending "… on day 1, TD-10).")

**E09's increment 6** (the town's trades; E09 decisions 197, 209 to 213) adds five town commands
(ADR-0028 point 6), each its player's, in its turn, with no decision open (GAM-DEC-001), the town
the actor's (an ally's waits for E10's teams, `TWN-STUB-21`), each refusal typed and changing
nothing: **`TownCommand::Market { town }`** (TWN-MKT-002; decision 209) opens E08's market
decision with the town as its place (point 6); **`TownCommand::University { town, hero }`**
(TWN-UNI-001; decision 210) opens E08's University decision for the named visitor or garrison
hero; **`TownCommand::BuyWarMachine { town, building }`** (TWN-SMT-001; decision 211) buys the
building's machine for the window's hero, the Stronghold's Blacksmith and Ballista Yard each by its
own building; **`TownCommand::BuildBoat { town }`** (TWN-SHY-001; TWN-SET-011 amended) launches the
part's boat at the town's spot, no hero needed, several a day while the spot frees;
**`TownCommand::Transform { town, army, slot }`** (TWN-TRF-001) turns one whole stack of the
visitor's army or the upper bar's into its target in place, any number a turn. **The answer**
`MarketAnswer::SellArtifact { place, take }` (TWN-MKT-005; ADV-MKT-002 amended) is new, offered
only by a town's window with the merchants (point 6).
**The state.** A town's record gains nothing (the town state hash stays v4); the adventure state
holds the merchants' stock and the artifact pool's marks (ADR-0016 point 1; decision 212).
**The build** draws the stock's seven (TWN-STK-001) last among its setup draws (point 4), so they
avoid the map's artifacts, the random ones and the Black Markets' while the pools last.

## Point 3, after E09's increment 5 paragraph on the Vortex's week and the night's refill ("**From E09 increment 5** (TWN-GLD-008, -009; …", ending "… the flag set again each week).")

**From E09 increment 6** (TWN-STK-002; GAM-CAL-002 amended) **the first day of every month from
the second** (days 29, 57, …) ends the month's start with **the merchants' draw**: after the
month's kind, the towns' month and, in a creature's month, the adventure's month step (ADV-MON-014),
before the towns' day, as the original's `new_month` ends with it (survey `E09-trades.md` §3.2).
Every entry is redrawn in order, sold or not, the old ones' marks kept (an unsold artifact stays
out of the pool until its class resets), whether or not any merchants stand (decision 207); an
entry stays empty where the pool gives none (T-24 fixed by construction, `TWN-QUIRK-21`: the
original loops for ever). Between two draws a purchase empties its entry for every town and
player; a sale never fills one; nothing else writes the stock.

## Point 4, after E09's increment 5 paragraph on the guilds' draw ("**E09's increment 5** draws **the guilds from the towns' stream** …", ending "… its weights 0 there).")

**E09's increment 6** draws **the merchants' stock** (TWN-STK-001, -002; Part H §101 call 4):
**the new game's seven** from the setup stream, last among the build's setup draws (after the
Portals' `town::portal::at_setup`, the setup stream's last today), so no earlier draw moves but
every state hash does; **every later month's** on its first day (point 3) from **the towns'
stream** (`StreamId::TOWNS`, as decision 179's guilds; a named stream of its own was the
alternative, asked at `TWN-Q37`), so every replay that passes day 28 changes its later towns' draws
(the census of growth to day 29, `h3-sim`'s town-map seeds), which T57 re-records and re-seeds in
one pull request. Entry by entry in the part's class order, equal chances among the class's
unmarked artifacts; with none left the marks reset to the bans, then any class but `special`
(relics and combinations through the fallback only); the drawn one marked.

## Point 6, after the point's last paragraph, increment 14's decisions ("**Increment 14's decisions:** three new kinds …", ending "… with the new level read (E14-17).")

**E09's increment 6** (GAM-DEC-011 and GAM-DEC-006 amended; E09 decisions 209, 210) adds **no
decision kind**: a town opens E08's market and University decisions as their **place**, a variant
of the visit's place (a map object as today, or the town by `TownId`), not a second decision.
- **The market** (GAM-DEC-011): its hero none possible at a town (the window's hero, ADR-0028 point
  6); open until its `Leave` and blocking as E08's; its hash writes the place's tag before its
  fields (the game state hash). E08's answers stand at the trader's count (TWN-MKT-003, -004):
  `Trade`, `Transfer` (to any other player still in the game), `Sell` (the window's hero's army),
  `Buy` on the merchants' stock (`Entry`, `NoPrice`, `NoRoom` with A-46 fixed as `ADV-QUIRK-48`,
  then `CannotPay`); a screen the town does not offer is refused (`Screen`). **`SellArtifact {
  place, take }`** (TWN-MKT-005): one artifact of the window's hero by its
  `h3_heroes::ArtifactPlace` (a worn position or a backpack index), refused in this order: a
  window without the sale (`Screen`), an empty place (`Place`), a worn artifact whose slot the
  backpack does not take (HER-ART-002's: the war machines, the Catapult, the spell book;
  `NotTradable`), a resource the parameters do not value (`NoPrice`).
  SoD's fifth misc position sells like the others (T-22 fixed, decision 205, `TWN-QUIRK-20`), every
  backpack artifact too, the Grail for 1 at least, a combination whole from its own position. Its
  view (`MarketView`, the window's owner's alone): the place, the hero, E08's `MarketOffers` (the
  screens, the count, the lots, the stock with its prices) with the sale's places and prices, the
  recipients.
- **The University** (GAM-DEC-006): its visit names the place, the hero, the skills (the part's
  four; with `TWN-QUIRK-16`'s flag off the scenario's banned ones, `GameState::banned_skills`, left
  out when it opens; on, a banned school offered and taught) and the price; E08's answers
  (`Buy(slot)` by `h3_heroes::university_slot` against the actor's treasury, several a visit;
  `Leave`); no mark at a town. Its view: the place, the hero, the skills with their slot states, the
  price.
`legal_commands` lists each new command where it is legal (point 8): `Market` where the window opens
and offers a legal answer; the sale for the cost resource per place that sells; `BuyWarMachine`
where the purchase is legal; `BuildBoat`; `University` per hero where a slot is open and the
treasury holds the price; `Transform` per stack that changes. Inside a town's window the answers
are listed as a map market's, no transfer.

## Point 8, after E09's increment 5 paragraph ("**E09's increment 5** appends … **the town's opening** …", ending "… no learning is an action.")

**E09's increment 6** appends (`cross-lane`, the AI lane's paths, E09 decision 7) **42 entries**
(the encoding from 105 to 147), each legal where `legal_commands` lists its command, the town by its
entrance in the tile head: **the market's opening** (1); **the artifact sale** for the cost resource
by place (22: worn positions 0 to 13, then backpack places 0 to 7); **the Blacksmith's machine** (2:
the town's first and second building with a `war_machine`, in building order); **the boat** (1);
**the University's opening** (2: the visitor, the garrison hero); **the transformation** (14: the
two armies' seven slots), each listed as point 6 says. The trades, sales and purchases inside a
town's window are increment 14's 56 entries, the University's purchases its four, each window's
`Leave` the closing role; no transfer (E08 decision 372). The next version at merge per task, or
one when tasks land together; the sale's 22 places, the largest, are a candidate to trim at its
task if the AI lane prefers (the sale without env entries was the alternative, asked at
`TWN-Q37`).

## Point 9, after E09's increment 5 sentence ("**E09's increment 5** bumps the game state hash (the store's v4 folded in) …", ending "… encoding v15 and protocol v76).")

**E09's increment 6** bumps the adventure state hash (the pool's marks, the merchants' stock;
T57), the game state hash (the adventure's folded in, T57; the decisions' places, T58a and T59b)
and the adventure replay format (`town market`, `town machine`, `town boat`, `town university`,
`town transform`, `answer market sell-artifact`; T58a to T59b), each `cross-lane`; every golden
adventure replay is re-recorded at T57 (the marks and the stock; the towns' stream's draws on day 29
in the replays that pass it), each changed line classified; the protocol (the server lane's,
`cross-lane`: `MarketWindow`'s object or town with its hero optional, the University's place,
`SellArtifact` with its refusals, the five commands with theirs, the events' places) and the env's
action encoding (point 8, the AI lane's, `cross-lane`), the next per task at merge. The map import
and the adventure ruleset part do not move; the town part v10 is the town lane's (ADR-0028 point
9). Shared (T55b): the quirk flags of `TWN-QUIRK-15` and `-16` once batch TWN-6's TWN-6-5 and -7
confirm them (no version moves: the profiles' ruleset hashes do) and the combinations' costs
(`h3-content`, HER-ART-008: the core ruleset hash moves); no new stream (decision 213). The
numbers are the next at merge, in merge order with the adventure lane's open work (E08 T142b,
#1162, and its queue): the second to merge rebases and bumps again.

## Point 10, after E09's increment 5 paragraph on the learning's triggers ("**E09's increment 5: the town's heroes learn** …", ending "… no turn start).")

**E09's increment 6: the trades' gains and costs** (Part H §105 to §113), each applied infallibly
once checked, a rejection leaving everything untouched, every rate and price the state's answer,
never the command's:
- **A trade, a transfer and a creature sale at a town** are increment 14's at the trader's count
  (TWN-MKT-003; at least 1, so never the division by zero the original's 0 makes); the transfer
  1 : 1 up to the whole stock, its gain told to the recipient alone; the sale at the creature
  factor of the count (a whole gold cost from 8 Marketplaces), the army keeping a creature.
- **The merchants' purchase** (TWN-MKT-004): `MarketParams::artifact_price(count, cost, pay)` (a
  combination's cost its parts' sum); paid, the artifact given (`HeroChanged`), the entry emptied
  for every town and player until the next month's draw.
- **The artifact sale** (TWN-MKT-005): `MarketParams::artifact_sale(count, cost, take)` (E08's
  module, `cross-lane`), max(1, round(f32(f32(cost × A) / v(take)))) with A the artifacts' factor
  at the count and v the trades' value table (`market_artifact_sale_rate` 0x5ECDC0,
  `E08-trades.md` §4), the X87 steps of the other three rates; the artifact removed
  (`HeroChange::RemoveArtifact`, its parts' locks freed, its effects gone), then the price gained
  (`ResourceGained`); the stock, the pool and the spell points untouched. **T-20 fixed by
  construction** (`TWN-QUIRK-18`): the place priced is the place removed.
- **The machine** (TWN-SMT-001): its creature's cost, each resource its own (`TWN-QUIRK-22`), paid
  (`ResourcePaid`), then the machine given (`HeroChanged`; a war machine has its own position).
- **The boat** (TWN-SHY-001): launched at the spot (E08's `launch`, an empty boat of the part's
  type, its builder the actor), `BoatBuilt`, then paid (`ResourcePaid`), E08's order for a Shipyard.
- **The University's purchase** is E08's (the part's price; `HeroChange::RaiseSkill` at Basic);
  **the transformation** (TWN-TRF-001) sets the slot's creature to the target, its count kept
  (`StackChanged`), free, the army's last stack allowed, nothing merged (T-26 fixed by
  construction, `TWN-QUIRK-23`: one command one whole stack, no holding army to lose).
**No new event kind** (Part H §101 call 13): `MarketOpened` and `UniversityOpened` name their place
(a map object or a town); every change is an existing event (`ResourcePaid`, `ResourceGained`,
`StackChanged`, `HeroChanged`, `BoatBuilt`, `DecisionClosed`), told to whom E08 tells it
(`ResourceGained` to a transfer's recipient alone; `BoatBuilt` by the fog's rules; `StackChanged`
and `HeroChanged` to the owner and to the viewers who see the army or the hero); the fold gate
covers each.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 6's `shared/` task T55a): points 1, 3, 4, 6, 8, 9, 10 amended for E09
  increment 6, the town's trades: the five town commands and `SellArtifact`, the stock drawn last
  at the build and at every later month's start from the towns' stream, the market's and the
  University's decisions with a town as their place, the env's 42 entries, the versions, the
  trades' gains and costs.
