# Draft: ADR-0028 amended for E09 increment 6 (the town's trades)

- Status: **Draft**, written by E09 T53 (2026-11-25), to be accepted at its checkpoint (`TWN-Q37`)
  before anything is built; then moved into `docs/adr/0028-town-context.md` by the increment's
  `shared/` task (T55a) with batch TWN-6's verdicts.
- Decisions: the epic's 197 (scope), 198 (the round), 200 to 207 (T52 and its checkpoint
  `TWN-Q35`: D-70 the original, TD-28 `fix-optional` as T-28, TD-29 the trader's count and at
  least 1 with T-19 fixed, TD-30 with T-23 `fix-optional`, T-22 fixed, TD-31 the original, T52's
  agent's calls each accepted), 208 to 212 (`TWN-Q36`: T53 whole; the market as E08's market
  decision opened by `TownCommand::Market`; the University as E08's University decision opened by
  `TownCommand::University`; the Blacksmith and the Ballista Yard as the one-shot
  `TownCommand::BuyWarMachine`; the stock and the pool's marks in the adventure state), 213 (T53's
  agent's calls, as `TWN-Q37` settles them).
- Working detail: the survey `docs/rules/surveys/E09-trades.md` (§2 the Marketplace's window, §3
  the merchants' stock, §4 the Blacksmith, §5 the Shipyard, §6 the University, §7 the
  Transformer, §11 batch TWN-6's rows), the design note `docs/epics/E09-design.md` Part H (§101 to
  §120). Each block below says where it goes in the ADR.

## Point 2, after increment 5's paragraph ("**Increment 5** (the mage guild; …", ending "… the garrison hero first (TWN-5-5).")

**Increment 6** (the town's trades; E09 decisions 197, 201 to 213) adds **nothing to the town's
record** (Part H §101 call 14): the Artifact Merchants' stock and the artifact pool's marks are the
adventure's (ADR-0016 point 1; E09 decision 212), the docked ship (building 20) is the boat spot's
occupancy, derived where a client asks (T-27, decision 207); the town state hash stays v4. Its
rules, each a pure function of `h3-towns` over the store, the part, the core and plain inputs the
game gives:
- **The count** (TWN-MKT-001; D-70, TD-29; survey `E09-trades.md` §2.1):
  `h3_towns::marketplaces(store, part, player)`, the number of the **trader's** towns (the window's
  owner's) with a building of role `marketplace` ever built (a capture moves one to its new owner's
  count at once), whoever owns the town traded at, the town itself needing none; the game clamps it
  to the market parameters' rows, at most 10 and **at least 1** (decision 203: a trader without one
  trades as with one; T-19 fixed, `TWN-QUIRK-17`). D-70's rows are E08's `MarketParams` (wood or
  ore for 37, 62 and 112 gold at 2, 4 and 8; decision 201).
- **What a town's window offers** (TWN-MKT-002; §2.2): **trades and the transfer** whenever it
  opens; **the artifact purchase and sale** with a hero where a building of role
  `artifact_merchants` stands; **the creature sale** with a hero where one of role `freelancers`
  stands (a town holds at most one of the two). **It opens** where a building of role
  `marketplace` stands (with or without a hero, §2.1), or one of role `artifact_merchants` or
  `freelancers` stands and the window has a hero (point 6); else refused, `NoMarket` (none of the
  three stands) or `NoVisitor` (no hero the rule names: the original's GENRLTXT 274).
- **The Blacksmith's machine** (TWN-SMT-001; §4.3, §4.6): the named building of the town's
  faction's record, standing, with a `war_machine` (`NoWarMachine` else; `UnknownBuilding`,
  `NotStanding` as the recruitment's); the machine that artifact, its price the machine's
  creature's cost in the core, resource by resource (T-25, decision 207, `TWN-QUIRK-22`; 2 500,
  1 000 and 750 gold with the shipped tables).
- **The boat, the University and the Transformer** (TWN-SHY-001, TWN-UNI-001, TWN-TRF-001; §5.3,
  §6.3, §7.5): a standing building of role `shipyard` (`NotStanding`), `university`
  (`NoUniversity`) or `transformer` (`NoTransformer`); the part's boat price and type, its four
  schools and their price (point 5); a creature's target by the part's table,
  `transformer_dragons_into` for the listed dragons and hydras, else `transformer_into`. The spot
  and its state are the adventure's, the scenario's banned skills the game's (point 6).

## Point 5, after increment 5's paragraph ("**Increment 5** (the mage guild; E09 decisions 162, 173) …", ending "… its weights 0 there.")

**Increment 6** (the town's trades; TWN-TRD-001; E09 decisions 197, 213) gives the town records'
buildings **six fields** (as `tavern`, `guild` and `library`): **`marketplace`** (a flag: building
14 of every faction, the count's building and the window's entry), **`artifact_merchants`** (a
flag: the Tower's, the Dungeon's and the Conflux's 17), **`freelancers`** (a flag: the
Stronghold's 21), **`war_machine`** (an artifact of the core with a `war_machine` creature: each
faction's Blacksmith 16 by `blacksmith_war_machines` 0x6AA9F8, Castle, Dungeon and Conflux
`ballista`, Tower, Inferno and Stronghold `ammo_cart`, Rampart, Necropolis and Fortress
`first_aid_tent`; the Stronghold's Ballista Yard 22 `ballista`), **`university`** (a flag: the
Conflux's 21) and **`transformer`** (a flag: the Necropolis's 22). The compiler refuses a
`war_machine` artifact the core lacks or that names no creature, and each flag twice in a record
(`marketplace` too: one count a town). The parameters (`[town_params]`) gain **`merchants_stock`**
(the classes in entry order: treasure three times, minor three times, major; `new_game_heroes`
0x4C0484, survey `E09-trades.md` §3.1; at most the adventure's `MAX_STOCK`, 8, none `special`),
**`boat_price`** (1 000 gold, 10 wood) and **`boat_type`** (1, every town's; code, §5.3),
**`university_skills`** (Fire, Air, Water and Earth Magic by key; code, §6.3) and
**`university_price`** (2 000 gold, §6.4), and **the Transformer's table**
(`skeleton_transformer_targets` 0x6412FC, §7.5): `transformer_into` (`skeleton`),
`transformer_dragons_into` (`bone_dragon`) and the creatures that become it, each pack naming its
own as the Altar's factions (`[[transformer_dragon]]`: the base pack's Green, Gold, Red and Black
Dragons, the Hydra and the Chaos Hydra; the AB pack's Azure, Crystal, Faerie and Rust Dragons). The
most Marketplaces counted is E08's `MarketParams::counts` (10). Each is tier-2 data of the
reading, harvested or the harvest's gap-fill (`h3-harvest`, `cross-lane`). **Not data:** the
screens a town offers (point 2's rule), the visitor-only rule (`TWN-QUIRK-15`'s flag), the sale's
refusals (the core's slots). **The core**
(`shared/`, T55b; HER-ART-008, Part H §109): a combination artifact whose pack names no cost costs
the resource-wise sum of its parts' costs (the original's `artifact_table_load` 0x44D031; SoD's
twelve sums, from the Angelic Alliance's 84 000 down to 3 000, survey §3.4), so the merchants, the
Black Markets and the sale read one cost; the core ruleset hash moves.

## Point 6, after increment 5's paragraph ("**Increment 5** (the mage guild; E09 decisions 170, 175, 176) …", ending "… and a turn's start never.")

**Increment 6** (the town's trades; E09 decisions 207, 209 to 213) adds five variants, each
`h3-towns` checking the town's part (point 2) and the game the turn, the owner and that no decision
is open, each refusal typed (`TownCommandError`) and changing nothing; every one is the town
owner's (an ally's town waits for E10's teams, `TWN-STUB-21`). **`TownCommand::Market { town }`**
(TWN-MKT-002; decision 209, as `Recruit` opens the hire dialog) names no building: the game names
**the window's hero** (the visitor; with `TWN-QUIRK-15`'s flag off, `strict-fixed`, T-28, decision
202, the garrison hero when the town has no visitor; never a field) and opens **E08's market
decision** with **the town as its place** (ADR-0017 point 6); the adventure's checks
(`markets.rs`, `cross-lane`) take the window as the game resolves it, the count read at each view
and answer (no build can happen while the window is open). **`TownCommand::University { town, hero
}`** (TWN-UNI-001; decision 210): the game checks the student, the town's visitor or its garrison
hero and the actor's (`NotTownHero`; decision 207), leaves out the scenario's banned skills with
`TWN-QUIRK-16`'s flag off (`strict-fixed`, T-23) and opens **E08's University decision** with the
town as its place. **`TownCommand::BuyWarMachine { town, building }`** (TWN-SMT-001; decision 211,
the original's own Buy-or-Cancel dialog): the game checks the window's hero (none `NoVisitor`),
that it wears no such machine (`HasWarMachine`; the backpack never holds one, HER-ART-002), its room (defensive:
`h3_heroes::check_change` with `AddArtifact`), then the treasury; then pays and gives the machine.
**`TownCommand::BuildBoat { town }`** (TWN-SHY-001; not E08's `BuildBoat`, which names a map
Shipyard): the adventure gives the town's spot and refuses it while an empty boat or a hero stands
there (`BoatSpotTaken`; T-21 fixed by ADV-SHY-003's rule, decision 207, `TWN-QUIRK-19`); the game
checks the treasury, then launches the boat and pays. **`TownCommand::Transform { town, army,
slot }`** (TWN-TRF-001; decision 207): the army an `h3_towns::TownArmy`, `Visitor` (the visitor's) or
`Garrison` (the upper bar's, `HireTarget::Garrison`'s meaning; no hero needed); the game checks
the army (`NoVisitor`, `UnknownSlot`, `EmptySlot`) and that the stack's creature is not its own
target (`NothingToTransform`: the Skeleton), then sets the slot's creature to the target, its
count kept, free (T-26 fixed by construction, `TWN-QUIRK-23`). Each applies infallibly once
checked (ADR-0017 point 10). Part H's new refusals: `NoMarket`, `NoWarMachine`,
`HasWarMachine`, `BoatSpotTaken`, `NoUniversity`, `NotTownHero`, `NoTransformer`,
`NothingToTransform` (`NoVisitor`, `UnknownBuilding`, `NotStanding`, `UnknownSlot` and `EmptySlot`
reused; the treasury's refusal the game's, as every price's). No new contract type: the commands
and their errors are `h3-towns`' types, which `h3-game` re-exports; the two places are E08's
visits (`MarketVisit`, `UniversityVisit`, the adventure lane's) naming a town by `TownId`, and the
new answer `MarketAnswer::SellArtifact` is E08's decision's (ADR-0017 point 6).

## Point 9, after increment 5's sentence ("**Increment 5** bumps the town ruleset part (v9: …", ending "… is not hashed).")

**Increment 6** bumps the town ruleset part (v10: the six building fields, the parameters; T56);
the town state hash (v4), the map import and the adventure ruleset part do not move (nothing new in
a town's record, the map or the adventure's tables). The adventure lane's versions it moves
(`cross-lane`): the adventure state hash (the pool's marks, the merchants' stock; T57), the game
state hash (the adventure's folded in, T57; the decisions' places, T58a and T59b) and the adventure
replay format (`town market`, `town machine`, `town boat`, `town university`, `town transform`,
`answer market sell-artifact`; every golden adventure replay re-recorded at T57); the protocol (the
server lane's: the places, `SellArtifact`, the five commands and their refusals) and the env's
action encoding (the AI lane's: 42 entries, ADR-0017 point 8), each `cross-lane`; `h3-harvest`'s
six fields and parameters (the battle lane's, `cross-lane`). `shared/` (T55b): the quirk flags of
`TWN-QUIRK-15` and `-16` once TWN-6-5 and TWN-6-7 confirm them (no version moves: the profiles'
ruleset hashes do) and the combinations' costs (the core ruleset hash); no new stream (decision
213). Each number is the next at merge, in merge order with the adventure lane's open work (E08
T142b, #1162): the second to merge rebases and bumps again.

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 6's `shared/` task T55a; decisions 197 to 213):
  points 2, 5, 6, 9 (nothing new in a town's record; the count, the window's offer, the machine,
  the boat, the University and the Transformer's target; the six building fields, the parameters
  and the core's combination costs; `Market`, `University`, `BuyWarMachine`, `BuildBoat` and
  `Transform`; the versions).
