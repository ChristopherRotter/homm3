# E08 design note

Increment 14's note: the trades and services (E08 decision 361). Increment 13's note (creature
banks) is in `E08-log.md` ("Increment 13's design note at its closure", moved by T126), increments
9's to 12's beside it (moved by T87c, T97c, T107 and T117); the code's comments cite these notes as
"increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 14's
amendments, drafted in the lane's `adr-drafts/` (`0016-` and `0017-increment-14.md`) and
applied to the ADRs by the round's `shared/` task with batch E14's outcome (the drafts then
deleted; *applied* by T131: ADR-0016 points 1, 4, 6 and ADR-0017 points 1, 6, 8 to 10 hold the
decisions, the drafts are gone). Written by T127b whole (decision 369, the user at its start; not the recommendation, a
cut by side) from T127a's survey and the answers at ADV-Q56 and ADV-Q57 (the epic's decisions 361
to 368). A building task adds an *As built* line to the section it builds; the note moves to
`E08-log.md` when the increment closes. Facts and sources: survey `docs/rules/surveys/E08-trades.md`
(§ numbers are the survey's unless they name this note); the proposed rule ids are the survey's §19.

Four parts by side: **A** the content (`h3-adventure`'s kinds and the market's rates,
`h3-pack-adventure`, the packs; §1 to §6), **B** the map (`h3-map`, `h3-adventure`; §7 to §12),
**C** the game (`h3-game`, the wire, the env, replays, benchmarks; §13 to §19), **D** batch E14, the
cut and the exit criteria (§20 to §22). ADR-0013 needs no amendment: the Altar's experience is
HER-EXP-001's `h3_heroes::gained_experience` per item and its artifacts leave by
`HeroChange::RemoveArtifact`, both as built; ADR-0015 none (no crate, no new dependency edge);
ADR-0003 none (§2: the rates in `h3_kernel::arith::X87`, the original's x87 arithmetic in
integers, as Q-0005's rates and the monsters' answer). *Agent's call (change
freely)* marks a choice the user could change; the checkpoint at this task's end accepts the note
and the drafts.

**The checkpoints' answers** this note builds on: the content (decision 361: the seven classes; the
market's rates built once, by whichever lane needs them first, the other reusing them), the first
round (362), the survey in two parts (363, 364), the original's roundings D-71, D-73, D-74 (365),
A-46 and A-48 fixed (366), the Altar's special artifacts as the original, A-47 harmless and
accepted, D-72 the original's (367), batch E14 before the build (368), this task whole (369);
**T127b's checkpoint** (ADV-Q58, decisions 370 to 373): the rates in the adventure part (370,
§2), the Altar's close and return around a level-up with a note for the clients (371, §14), the
env's 67 entries `cross-lane` (372, §18), the cut and every other agent's call as written (373).
The note and the two drafts accepted. **Batch E14's outcome** (T130, ADV-Q59): the Hill Fort's RoE side by the enabled
packs (378, §4, §7).

**A correction to the survey** (this task; survey §0, §1, §2, §17 and §19 corrected in the same
change): the seven classes are **entered**, not visited from a neighbour. The survey read the visit
mode from the templates' blocked entrance; the original's class lists decide (E08 decision 317's
rule, `TRAPS.md`): `object_classes_not_entered` 0x63AA58 lists none of 2, 7, 35, 99, 106, 213 and
221, and `object_classes_from_above` 0x63AB00 none of them either (`docs/exe/map.toml`). VCMI 1.7.5
agrees (`config/objects/markets.json` and `generic.json`: no `blockedVisitable` on any of them). So
the hero steps onto the entrance, stands there for the window and after it, the move ends there,
and `Revisit` (ADV-TEL-008) opens the window again; none is entered from above (ADV-OBJ-004).
Batch E14 records where the hero stands (§20, E14-16).

## Part A — the content (the adventure part, its packs)

### 1. The market kind (`h3-adventure`'s `ObjectKind::Market`; ADV-MKT-001)

- **`market { screens, count, stock }`**, one kind for the three markets (survey §3, §5 to §7):
  - `screens`: the window's screens the object shows, a set of `MarketScreen` (`Trade` the
    resources for resources, `Transfer` the resources to a player, `Buy` the resources for an
    artifact, `Sell` the creatures for resources; the town's artifact sale, the original's screen 3,
    is the town lane's to add with its Marketplace, §11's note). At least one.
  - `count`: the Marketplaces it trades as (the rates' row, §2): 1 to the market parameters' rows;
    every map object 5 (the entries' 5, survey §3).
  - `stock`: the artifact classes of a Black Market's draw in order (§9), none a Special; present
    exactly when `Buy` is among the screens.
- **The base pack's types:** `trading_post` (classes 99 and 221: `screens = ["trade",
  "transfer"]`), `black_market` (7: `["buy"]`, `stock` three treasures, three minors, one major);
  the ab pack's `freelancers_guild` (213: `["sell"]`). Each `visit = "enter"`, `from_above =
  false`, `kind_mark = false` (none of the three marks, survey §2). Not ownable, not weekly, no
  guards.
- The part refuses a market without a screen, a `count` outside the rows, a `stock` with `Buy`
  missing or a Special class, and `Buy` without a `stock` (the compiler's errors, tested).
- *As built* (T132a2): `ObjectKind::Market(Market)` (`ruleset/trades.rs`: `screens:
  Vec<MarketScreen>` of `Trade`, `Transfer`, `Sell`, `Buy`, `count`, `stock`; `Market::shows`),
  the same struct as its definition (no key); hash tag 47. The count is checked against
  `[params.market]`'s rows once every pack is in (`check_trades`; a market without the
  parameters refused). Every one of the four kinds entered (`a trade or service is not
  entered`); no visit until T133 (`effect` none).

### 2. The market's rates (ADV-MKT-002; survey §4, §9; D-70 the town lane's, D-71 the original's)

- **The data:** the adventure part's `[market]` parameters (`MarketParams`, Tier 2): `values` and
  `artifact_values` by core resource (the original's 250, 500, 250, 500, 500, 500, 1 and the
  purchase's 125, 250, 125, 250, 250, 250, 1), and the three factor tables by the Marketplaces'
  count 1 to 10 in hundredths (`resources` 10 to 50, `artifacts` 20 to 60, `creatures` 30 to 100).
  Each factor is used as the binary32 nearest its hundredths (`f32(p / 100)`), checked in the part's
  tests against the executable's tables' bit patterns (`market_resource_rates` 0x678344,
  `market_artifact_rates` 0x678370, `market_creature_rates` 0x67839C) and by the eighth
  original-data check (§6). A value of 0 is refused (a rate would divide by it).
- **The computation** (`h3_adventure::market` module, pure, `numeric = "float"` in the catalog as
  HER-EXP-001's): each step as the original stores it, in **`h3_kernel::arith::X87`** (the
  original's x87 arithmetic computed exactly in integers, ADR-0003's no-float rule kept: quirk
  Q-0005's model, already the monsters' answer's in `h3-adventure`): `b32(x)` a store to binary32
  (`X87::round(SINGLE_BITS)`), every operation at `X87_PRECISION`'s 53 bits
  (`docs/formats/installation.md` §7.10), `round(x) = trunc(x + 0.5)`, a factor `X87::ratio(p,
  100, SINGLE_BITS)` (the binary32 nearest p / 100, one rounding). Not a quirk: decision 365 made
  the original's rounding the rule. A rate is a **lot**: `Lot { give, take }`, one of them 1.
  - resources (given g, wanted t, count m): a = b32(v(g) × R_m), q = b32(v(t) / a); q ≥ 1 gives
    `Lot { give: round(q), take: 1 }`, else `Lot { give: 1, take: round(1 / q) }`;
  - a creature of gold cost c for t: a = b32(c × C_m), q = b32(v(t) / a), the lot as above in
    creatures given; a creature of no gold cost has no lot (none in the packs; refused by the
    sale, §13);
  - an artifact of gold cost c paid in r: price = round(b32(c / b32(w(r) × A_m))) of r; a price of
    0 is no price (refused, §13; the draw places none, survey §7.2);
  - the artifact sale (the town's screen 3, `market_artifact_sale_rate` 0x5ECDC0) is left to the
    town lane, which builds it on the same module.
- *Agent's call:* computed at each trade from the parameters (a few `X87` operations, no shared
  change); not chosen: finite tables compiled into the part (10 × 42 resource pairs, every
  creature cost and artifact cost × 7 × 10), which grow with the content and need the same
  computation to check them; `kernel::num`'s `Real` with a new conversion to binary32 (a `shared/`
  change for what `X87` already models).
- **The home** (decision 361: built once, by the first lane, reused by the other). *Decided*
  (decision 370, the user at ADV-Q58; recommended): the adventure part's `[market]` with the computation in `h3-adventure`, as increment 10's
  `hire_price` and `HireSource` serve E09's recruitment: the town lane's Marketplace (in its
  `h3-game` modules) calls `h3_adventure::market` with its own count; `h3-towns` needs no rate (the
  treasury and the trade are `h3-game`'s). Not chosen: the core ruleset (`h3-content`, a `shared/`
  change and ADR-0015 point 3 amended, so `h3-towns` could read it too). The town lane's inbox is
  told (§21).
- **The check** (tests): every case of the survey's throwaway script as tables in the tests: at
  five Marketplaces survey §4's table, the creature sale's 25 → 17 gold and 3 000 → 8 wood, the
  artifact's 2.5 × its cost in gold; D-70's 37, 62 and 112 at 2, 4 and 8 (the town's rows, the
  original's); a property: every lot's `give` and `take` at least 1. Batch E14's E14-1 and E14-5
  confirm against the original.
- *As built* (T132a2): `AdventureParamsDef::market: Option<MarketParamsDef>` (each value table
  by resource key, the three factor lists in hundredths), compiled into T132a1's `MarketParams`
  when the parameters resolve (`compile_market_params`: every core resource once in each table,
  in any order, the lists of one length; the constructor's errors as `InvalidParams`), hashed
  last in the part (v23); `AdventureParams::market()`. The parameters must reach every core
  resource, so the pack that gives them depends on every resource's pack (the base pack: all
  seven are base's). The pack syntax `[params.market]`; the base pack's values and factors.
- *As built* (T132a1, decision 380): `h3_adventure::MarketParams` (`new` from the two value
  tables by resource as `u16` and the rows in hundredths, refusing an empty or uneven table, a 0
  value, no row or more than 255, a 0 factor: `MarketParamsError`), its `resource_lot`,
  `creature_lot`, `artifact_price`, `factor_bits` (for the eighth check) and `counts`; `Lot`
  (`give`, `take`). A stored quotient is held scaled by 2^32 (neither rounding depends on a
  power of two), so a dear creature sold for gold keeps every bit `X87` holds. Tests: survey §4's
  table at five (its precious-for-wood-or-ore cells corrected to 2 : 1), the wood-for-gold and
  gold-for-wood rows at every count (D-70's 37, 62, 112; 88 at 6), E14-1's, E14-2's, E14-4's and
  E14-5's lots, the factors' bit patterns against the executable's tables, the refusals and a
  property (every lot at least 1, one side 1). A throwaway comparison with a binary32 model
  (Python, not kept) agreed on 1 120 490 cases: every pair at every count, creature and artifact
  costs 1 to 6 000 and on to 200 000 by 97. The part's `[market]` (its definition, compile and
  hash) is T132a2's.

### 3. The Altar of Sacrifice kind (ADV-ALT-001 to -003; survey §12)

- **`altar { artifacts, keeps, places, creature, artifact_factions, creature_factions }`**:
  - `artifacts`: the experience by artifact class (treasure 1 000, minor 1 500, major 3 000, relic
    6 000, special 0; the original's immediates in `altar_artifact_entry` 0x55FC30);
  - `keeps`: the artifacts never put on the altar (the base pack's Spell Book and Catapult,
    `altar_worn_click` 0x5632A0); any other may be, the special ones for 0 (A-47 accepted, D-72
    the original's, decision 367);
  - `places`: the most a sacrifice takes, **23** (the 22 places, `altar_slot_row_counts` 0x6412E8,
    and the one artifact held on the cursor, both destroyed by Sacrifice);
  - `creature`: a creature's experience as `factor × ⌊ai_value / divisor⌋` (5 and 40;
    `altar_set_creature_amount` 0x564C00), the core's AI value;
  - `artifact_factions`, `creature_factions`: the hero class's factions whose heroes may sacrifice
    artifacts and creatures (`altar_dialog_create` 0x55FDD0): the base pack's Castle, Rampart,
    Tower, Stronghold and Fortress for artifacts, Inferno, Necropolis, Dungeon, Stronghold and
    Fortress for creatures; the ab pack adds Conflux to both (a change of the type, as the ab pack
    changes the core). *Agent's call:* the lists on the kind, not an alignment on the core's
    factions (VCMI's), which would be a `shared/` change for one object.
- **The base pack's type** `altar_of_sacrifice` (class 2): `visit = "enter"`, `from_above =
  false`, **`kind_mark = true`** (the original's k 28, survey §2; the engine marks every player's
  visit, since every player answers, §10).
- The part refuses an empty `places`, a `divisor` of 0, a `keeps` artifact twice and a faction
  listed twice.
- *As built* (T132a2): `ObjectKind::Altar(Altar)` (`experience: [u32; 5]` by `ArtifactClass`,
  `keeps`, `places`, `creature_factor`, `creature_divisor`; `Altar::artifact_experience`), from
  `AltarDef` (the kept artifacts by key; `AdventureCompileError::UnknownArtifact`); hash tag 48.
  **The factions are part-level entries, not the kind's lists** (*agent's call*, changing this
  section's: the adventure part has no way for the ab pack to change a base type, so Conflux
  could not join the base type's lists): `[[altar_faction]]` (`AltarFactionDef`: a faction,
  `artifacts`, `creatures`, one at least), each pack its own factions as the random dwellings'
  tables (base's eight, ab's Conflux), compiled into `AdventureRuleset::altar_screens(faction)`;
  with an Altar type every core faction has one (`AdventureCompileError::AltarFaction`); hashed
  last in the part. Not chosen: an override of a base type by a later pack (a new mechanism for
  one object), an alignment on the core's factions (a `shared/` change).

### 4. The Hill Fort kind (ADV-HFT-001, -002; survey §13; D-74 the original's)

- **`hill_fort { gold }`**:
  - `gold`: the percent of the upgrade's gold difference paid, by creature level 1 to 7 (0, 25,
    50, 75, 100, 100, 100: `hill_fort_gold_factors` 0x63EB4C as percents); **a level of 0 % is free
    whole** (the original computes no price at level 1, `hill_fort_rows` 0x4E7EB0). The original's
    binary32 factors are quarters, so `trunc(f32(gold) × factor)` is exactly `gold × percent / 100`
    rounded down for a gold below 2²⁴: integer arithmetic (*agent's call*), the part refusing a
    percent that is not a multiple of 25 or above 100 so the equivalence holds.
  *Batch E14 (T130, decision 378):* `roe_except` withdrawn (it held the base pack's four
  elementals for an RoE map's Hill Fort): **the enabled packs decide** (decision 104's rule).
- **Which stacks:** a creature with an upgrade in the ruleset (`upgrades`' first): the original's
  `creature_is_unupgraded` 0x47B120 and
  `creature_upgrade` 0x47B1A0 (a town's base creature and its upgrade), equivalent on the shipped
  packs, where every creature with an upgrade is a town's base creature (the eighth check asserts
  it, §6). Neutral and upgraded creatures have no upgrade. The four elementals have an upgrade only
  under the ab pack's override (Storm, Magma, Ice, Energy): with the base pack alone none
  upgrades, as the original's game version 0 (`hill_fort_rows` skips 112 to 115 there); with ab
  or sod they upgrade on every map, as the Complete edition plays an RoE-format map (its version
  comes from the players' editions, never the map's format: survey §21, E14-14), VCMI and the wiki.
  The original's version 0 (an RoE campaign, a network peer with RoE alone) is the recorded
  deviation, as E09 decision 14's moat.
- **The price** of a stack of n (`upgrade_cost` 0x54E750): for each resource max(0, the upgrade's
  cost − the creature's) × n; then the gold alone scaled by its level's percent, rounded down once
  for the stack (D-74). A level-1 stack is free.
- **The base pack's type** `hill_fort` (class 35): `visit = "enter"`, `from_above = false`,
  **`kind_mark = true`** (the original's k 14, every visit).
- *As built* (T132a2): `ObjectKind::HillFort(HillFort { gold: [u8; 7] })`, the percents checked
  (to 100, multiples of 25); hash tag 49. Which stacks is the runtime's (T134b1, §15's *As
  built*): a creature with an upgrade in the enabled packs, the first of its upgrades.

### 5. The War Machine Factory kind (ADV-WMF-001; survey §14)

- **`war_machine_factory { machines }`**: the war machine creatures it sells, in the dialog's
  order (the base pack's Ballista, First Aid Tent, Ammo Cart; never the Catapult); each a war
  machine creature with exactly one artifact in the core (`ArtifactType::war_machine`), which a
  purchase gives. Its price is the creature's cost (2 500, 750, 1 000 gold; the core's).
- **The base pack's type** `war_machine_factory` (class 106): `visit = "enter"`, `from_above =
  false`, `kind_mark = false`.
- The part refuses an empty list, a creature twice, and a creature that is not a war machine with
  one artifact.
- *As built* (T132a2): `ObjectKind::WarMachineFactory(WarMachineFactory { machines })`, each
  machine with its one artifact found in the core (`WarMachineFactoryDef` by creature key); hash
  tag 50.

### 6. The packs and the eighth original-data check

- **The base pack** (`packs/base/adventure.toml`): the five types above with the original's
  values, each with its comment of sources (survey sections and addresses); `[market]`'s tables.
  **The ab pack:** `freelancers_guild` and the Altar's change (Conflux in both lists). The sod pack
  changes nothing.
- **The eighth original-data check** (`--with-original-data`, as increment 13's seventh): the
  executable's value tables (`resource_values` 0x68C482, `resource_values_artifacts` 0x68C492),
  the three factor tables' bit patterns and the Hill Fort's factors (0x63EB4C) against the base
  pack's `[market]` and `hill_fort`; every creature the packs give an upgrade is a town's base
  creature (the executable's town creature table, read for `creature_is_unupgraded`); the census
  (survey §1) over the shipped maps: 1 058 objects of the seven classes on 148 maps, every one at
  subclass 0.
- **Versions:** the adventure ruleset part's next (v22 today; the kinds' tags, `[market]`).
- *As built* (T132a2): base's five types after the Den of Thieves (`trading_post`,
  `black_market`, `altar_of_sacrifice`, `hill_fort`, `war_machine_factory`), `[params.market]`
  and the eight `[[altar_faction]]` entries; ab's `freelancers_guild` and Conflux's entry; each
  with its sources in the pack's comments. **Check 8** (`h3-pack-adventure`'s `original_data`):
  the seven classes' types, kinds and visit modes against the class lists, no other type of the
  four kinds; `[params.market]`'s values and the factors' bit patterns at counts 1 to 10 against
  the executable's tables; the Hill Fort's percents against its gold factors; the 63 creatures
  with an upgrade each a town's unupgraded one whose upgrade the town table names; the Altar and
  the Hill Fort among the kind marks (check 6). It passes on the staged installation and fails on
  three perturbations (a value, a percent, a factor). **The census** of the seven classes over
  the shipped maps moves to T132b, with the import that types them (the map reader's check).
  **Part v23** (the town lane's T39a took v22): the 98 goldens' part hash lines, and the 28
  played with ab or sod their checkpoints and final hashes (base's five new types move ab's and
  sod's ids); the AI lane's two bots tables' `engine.adventure` (`cross-lane`, the user's
  consent).

## Part B — the map (`h3-map`, `h3-adventure`)

### 7. The import and the writer (`h3-map`; map import's next version)

- **The import** maps class 2 to `base:altar_of_sacrifice`, 7 to `base:black_market`, 35 to
  `base:hill_fort`, 99 and 221 to `base:trading_post` (the map's class kept for its picture, as
  220's abandoned mine), 106 to `base:war_machine_factory`, 213 to `ab:freelancers_guild`; another
  subclass than 0 is refused; no body is read (`docs/formats/h3m.md`'s "every other class:
  nothing"). Their footprints are the templates' with the entrance enterable (§0's correction: the
  class lists' rule, as the Magic Wells').
- **An RoE map's Hill Fort:** nothing of its own (decision 378, T130, replacing T127b's mark
  `Holds::RoE` by the map's format): the enabled packs decide which stacks upgrade (§4). The
  original's game version is not the map's format (survey §21: the scenario dialog's, from the
  players' editions; SoD for every single and hot-seat game of this edition), so a mark would have
  refused what the Complete edition upgrades. Not chosen: the mark kept.
- **The writer** (`h3_map::writer`): the seven classes by template, no body, for the fixtures and
  batch E14's maps (§20); class 221 only in the AB and SoD formats, 213 likewise. *As built*
  (T128a): `TRADE_OBJECTS` among `PLAIN_OBJECTS`, `AB_TRADE_OBJECTS` refused in RoE ("an AB trade
  in an RoE map"); `Map::banned_artifacts` beside the editor's default bans (decision 375); the
  import reads the seven as untyped blockers until T132b (`tests/trades.rs`).
- **Versions:** map import's next (v26 today; the town lane's T39a plans v26 too: whichever merges
  first takes it, the other the next).
- *As built* (T132b, decision 382; **map import v27**, E09 T39a took v26): `codes.rs`'
  `object_type` maps the seven at subclass 0 (a subclass past 0 `ImportError::Subclass`), 221 to
  `base:trading_post` (the map keeps its class); no body, nothing held, no owner; the footprint
  the template's (the entrance blocked and visitable: the kind's `entered` decides the visit, as
  the Magic Well's). A step onto one is `AdventureCommandError::Visit` (`effect` none) until T133
  and T134 (`tests/trades.rs`). **The census** (`h3-map`'s `original_data`: `increment_14`,
  `SURVEY_TRADES`): each of the seven's 1 058 objects typed as its class's kind at subclass 0, no
  body, one entrance on land (221's on snow); by class the objects, the maps by version,
  underground, the most on one map and the pictures as survey §1 on its first run; 148 maps; the
  War Machine Factories 255 with the entrance west of the anchor (the game's `Objects.txt` line)
  and 3 at it; the maps by their Black Markets as §1. The example's other output as `master`'s
  (compared line by line). No fixture map holds one of the seven: the 98 goldens moved only
  `import_version`.

### 8. The state (`h3-adventure`; adventure state hash's next version)

- **A Black Market's seven** as drawn are the static map's `Resolved::Stock(Vec<ArtifactTypeId>)`
  (the draw's order, at most the kind's `stock` length: an entry the pool could not fill is left
  out; §9), never changed after the build.
- **Its sold entries** in the visits' block (`Visits`, increment 7's design note §10):
  `sold: BTreeMap<u32, u8>`, by object index a bit per entry, absent for a market nothing was
  bought at; the forks share the block, a purchase copies it once. *Agent's call:* in the visits'
  block, not a block of its own (a few markets per map, written rarely).
- **Nothing else:** the Trading Posts, the guild, the Altar, the Hill Fort and the factory hold no
  state (no visit limit, no owner, no week: survey §2); the kind marks are increment 12's flags.
- **The hash:** `Resolved`'s tag 10 (the stock as length and entries) and the visits' block's sold
  map after its emptied bits; the adventure state hash's next version (v27 today).
- *As built* (T133a1, decision 383; **adventure state hash v27**): `Resolved::Stock(Vec<ArtifactTypeId>)`
  (`stock()` reads it, empty for every other draw; hash tag 10); `Visits::sold:
  BTreeMap<u32, u8>` with `is_sold(index, entry)` and `sell(index, entry)` (an object past the
  map's or an entry past the eighth: nothing), hashed after the emptied bits as its number and,
  in object order, the index and its bits; `AdventureState::is_sold` reads it (none without a
  block). The kind's stock holds at most `MAX_STOCK` (8) entries, a bit each (`compile_market`:
  "a market's stock holds more than 8 entries").

### 9. The Black Market's draw (ADV-MKT-006; survey §7.1)

- **When the game is built**, in `resolve_rewards`' second walk in object order (beside increment
  7's objects and increment 13's banks), from the setup stream: for each class of the kind's
  `stock` in order (three treasures, three minors, one major) one artifact of that class from **the
  one pool** of the random artifacts, chests, finds and banks (`ArtifactPool::draw`, ADV-OBJ-006:
  the bans and the map's placed artifacts marked; an artifact drawn is marked, so markets repeat
  neither each other nor the map's other random artifacts until a class's pool empties, then it
  restarts from the bans, survey §7.1). An entry with nothing left to draw (every artifact of its
  class and of the fallback banned) is left out.
- **Never again:** no restock (survey §7.1, VCMI's `blackMarketRestockPeriod` 0 agrees); only a
  purchase writes the sold entries.
- The original draws in its entrances' walk (`setup_map_objects` 0x4C0CC0: level, row, column);
  the engine's walk is object order (E08 decision 17: the draws' order is the engine's).
- *As built* (T133a1): `stock_draw` in the new `state/markets.rs` (its own module, as the banks'
  draw: the release build's codegen units), from `resolve_rewards`' second walk for a market whose
  kind has a stock (a Trading Post or the guild draws nothing, its `resolved` `Nothing`):
  `ArtifactPool::draw` per class in order, an entry with none left out. Tests (`state/tests/draws.rs`):
  the classes in order with the placed and random treasures marked first and the pool's restart;
  two markets and a random treasure fifteen distinct artifacts; a banned class widened, never the
  ban; every artifact banned an empty stock. `h3-map`'s original-data check counts the shipped
  maps' stocks on the static layer and under the map's bans (each its kind's classes in order,
  none banned).

### 10. The visits (ADV-MKT-001, ADV-ALT-001, ADV-HFT-001, ADV-WMF-001: the adventure's half)

- **Entered** (§0's correction): a step onto the entrance visits the object and ends the move
  (ADV-OBJ-003); `Revisit` visits it again from the entrance (ADV-TEL-008); a guard on the tile is
  visited first (decision 279). Every visit, every time: no day, week, owner or earlier visit is
  tested (survey §2); no movement is spent but the step.
- **Every player's visit** opens the window: the original's human-only branches (the computer's
  `ai_black_market_visit`, `ai_hill_fort_visit`, `ai_war_machine_factory_visit`, nothing for the
  computer at the four others) are the computer player's behaviour (E10), not a rule of the engine
  (survey §17).
- **The outcome** (`AdventureOutcome`): one new field, `service: Option<ServiceVisit>`, the window
  the game opens as the visitor's owner's decision (*agent's call:* one field for the three, not
  one each, as the outcome grows):
  - `ServiceVisit::Market(MarketVisit { hero, index, object_type, anchor })`: the screens, count
    and stock are read from the kind and the state at each step, never copied;
  - `ServiceVisit::Altar(AltarVisit { hero, index, object_type, anchor })`, after the team's kind
    mark (ADV-BON-009's `mark_kind`, every visit);
  - `ServiceVisit::HillFort(HillFortVisit { … })`, after the kind mark.
  An Altar whose hero's class's faction is in neither list opens nothing (no screen; none in the
  packs: every faction is in one).
- **The factory** asks first: the outcome's `offer` is the hire question (`OfferKind::Hire {
  creatures }`, ADV-HIR-001's question without a price; the original's ADVEVENT question), whose yes
  opens the hire dialog (`AdventureOutcome::hire`, ADV-HIR-002) with the factory as an object
  source (`HireSource::Object`); no free creatures (`free_level` reads a dwelling only).
- *As built* (T133a2, decision 383; the markets only, the Altar's, the Hill Fort's and the
  factory's with T134): `Effect::Market` from `effect`, dispatched in `AdventureState::visit` to
  `AdventureOutcome::service: Option<ServiceVisit>` with `ServiceVisit::Market(MarketVisit { hero,
  index, object_type, anchor })`; entered and revisitable as every built entered kind (no mark,
  no event, no block written). The game opens nothing until T133b (`ADV-STUB-18`, marked in
  `GameState::settle_outcome`; lifted by T133b). `h3-map`'s `tests/trades.rs`: a market's step
  entered, the other three kinds still refused.

### 11. The adventure's checks and changes (the answers' adventure half)

Each answer is checked by the adventure against its state and the kind (the game then checks the
treasury, pays, and calls the adventure's infallible change: ADR-0017 point 10, as the hire's
`check_hire` and `hire`):

- **A sale** (`check_sale`, `sell`; ADV-MKT-005): the slot holds a stack of the visitor's army,
  its creature has a lot for the resource (§2), the lots from 1 to the most: when one creature
  gives one or more units the lots count creatures (the stack's count), else units (the count
  divided by the lot's creatures); **less one creature when the army's other slots are empty**
  (the army keeps a creature, survey §6). The change lowers the stack (cleared at 0) and names the
  gain.
- **A purchase** (`check_buy`, `buy`; ADV-MKT-007): the entry is drawn and not sold, its price in
  the resource is not 0, and **the hero can receive the artifact** (A-46 fixed, decision 366):
  `HeroChange::AddArtifact`'s check (HER-CHG-002: a position that takes it, else the backpack below
  its most), *not* `can_receive_artifact`'s full-backpack count (A-25 is the pickups' test; the
  original's market places into a free position whatever the backpack holds, survey §7.2). The
  change marks the entry sold and gives the artifact (`HeroChanged`; in SoD a completed
  combination's question as any received artifact's, HER-ART).
- **A sacrifice of artifacts** (`check_sacrifice`, `sacrifice`; ADV-ALT-002): the hero's class's
  faction allows artifacts; the places distinct, 1 to `places`, each holding an artifact of the
  hero's not in `keeps`; the total above 0 (the original's disabled Sacrifice; a lone special
  artifact is refused, A-47's case only beside others). Each artifact's experience is
  `gained_experience(hero, base)` (HER-EXP-001's single computation with quirk Q-0005, survey §12.2's
  `trunc(L × f32(base))` **per artifact**: D-73 the original's), summed (saturating). The change
  removes them (worn positions in position order, then the backpack's from the last index down,
  so no place moves before it is removed) and names the total.
- **A sacrifice of creatures** (ADV-ALT-003): the faction allows creatures; an amount per slot (0
  none), each at most its stack's count, **the army keeping at least one creature** (survey
  §12.3), the total above 0; each stack's experience `gained_experience(hero, amount ×
  e)` with e = `factor × ⌊ai_value / divisor⌋` (per stack: D-73), summed. The change lowers each
  stack.
- **An upgrade** (`hill_fort_rows`, `upgrade`; ADV-HFT-002): each slot's row (its upgrade, its
  price, §4) as the visit reads it; the change replaces the stack's creature by its upgrade, the
  count kept. Its affordability is the game's (§15).
- **The factory's purchase** (`check_hire` and `hire` extended; ADV-WMF-001, ADV-HIR-002
  amended): the creature one of its machines, the amount 1, **the hero lacking the machine's
  artifact** (`recruit_dialog_select` 0x5503A0: the count 1 less whether the hero holds it, at
  least 0); `hire_stock` gives each machine with 1 or 0, `hire_room` 1 when its position is free.
  The change gives the artifact into its position (`HeroChange::AddArtifact`), no army change.
- *As built* (T133a2): `state/markets.rs`' `check_sale` and `sell`, `check_buy` and `buy`, with
  `Sale` and `MarketPurchase` (private fields, their readers; not `Clone`: the hire's form) and
  `MarketError` (`command.rs`: the design's nine and, *agent's call*, `Visit` for a window whose
  hero is off the map or whose object is no market, as the hire's `NoSource`). The checks take
  the resource a creature's and an artifact's cost is read in from their caller (`value`: the
  original's gold; the part names none, ADV-Q60, for T133b). A sale's order: the visit, the
  screen, the slot, the lot (`NoPrice`), the lots (`Amount`: 0 or more creatures than the stack's),
  the lone stack whole (`LastCreature`); its gain in 64 bits saturated; `sell` returns the
  `ArmySlot` the game tells. A purchase's order: the visit, the screen, the entry (`Entry`: past
  the stock or sold), the price (`NoPrice`), the room (`NoRoom`: `check_change`'s
  `AddArtifact`; a hero the store lacks has none); `buy` writes the sold bit. Tests
  (`state/tests/markets.rs`, six): the visit and Revisit of each market, the sale's lots and
  refusals (survey §4's 25 → 17), the purchase's prices (E14-4's 5 000; 40 wood) and refusals,
  A-46's room cases, the windows that are not one, the errors' texts.

### 12. Information (ADV-MKT-008; ADR-0016 point 6; survey §8)

- **Every viewer** of an object's tile sees its type, the same to all (no owner, no flag); the
  rates are the ruleset's, public.
- **A Black Market's stock** is server-only outside the visitor's open decision: its view (§13)
  shows the seven with the sold ones empty, to the owner alone. **What a viewer keeps of a market
  it visited: nothing** (*agent's call*, the survey's recommendation: the original shows the stock
  only in the window; a player's or an agent's memory is its own). Not chosen: the last seen seven
  as a knowledge by object (ADV-VIS-006, as a visited University's skills), a snapshot that grows
  stale as others buy.
- **The kind marks** (the Altar's and the Hill Fort's): increment 12's `KindKnown` to the team
  that gains it (ADV-BON-009), the hint's "visited" text the map client's (E24).
- **No new adventure event:** a trade's resources, the army's changes and the hero's changes are
  the game's events (§17); the sold entry is shown by the view alone (as a University's bought
  slot).
- *As built* (T133b, decision 384): `AdventureState::market_offers` (`state/markets.rs`) gives
  `MarketOffers { screens, count, trades, sales, stock }` for the screens the window shows:
  `TradeOffer { give, take, lot }` per pair of distinct valued resources (given, then wanted, in
  the core's order), `SaleOffer { slot, lots }` per stack (none for a creature of no cost in the
  cost resource), `StockEntry { artifact, prices }` per entry (none and no price once sold); the
  visit refused as `Visit`. `GameState::market_view` adds the recipients (every other player,
  with the transfer's screen) into `h3_game::MarketView`, `DecisionView::Market`.

## Part C — the game (`h3-game`, the wire, the env; T127b)

Three new decision kinds, each its owner's alone, holding no one else and blocking its owner's
`EndTurn` like an offer, open until its `Leave`; every answer but `Leave` keeps it open (the
original's windows stay open for more trades). The system actor's default is `Leave` for each.

### 13. The market's decision (GAM-DEC-011; ADV-MKT-003 to -007)

- **`Decision::Market { owner, visit: MarketVisit }`**, opened by `ServiceVisit::Market`
  (`MarketOpened { player, hero, object_type, anchor }`, to the owner).
- **`Answer::Market(MarketAnswer)`**, each refused for a screen the kind lacks
  (`MarketError::Screen`):
  - `Trade { give, take, lots }` (screen `Trade`, ADV-MKT-003): `give ≠ take`, `lots` from 1 to the
    most (what the treasury holds of `give` divided by the lot's `give`); the lots' `give` paid
    (`ResourcePaid`), their `take` gained (`ResourceGained`, saturating, GAM-RES-001; products in
    64 bits). *Agent's call:* the amount as lots, the original's slider unit (survey §5), so every
    accepted amount is one the original could trade.
  - `Transfer { resource, amount, to }` (screen `Transfer`, ADV-MKT-004): `to` a player of the game
    other than the actor, still in it (E10's elimination; until then every player), `amount` from 1
    to the treasury's; paid by the actor (`ResourcePaid`, to it) and gained 1 : 1 by `to`
    (`ResourceGained`, to `to`), allies and enemies alike. *Agent's call:* no event naming the
    giver to the recipient (the original's hot seat shows only the treasury; its network message
    is not modelled); not chosen: a `ResourcesReceived { from }` event.
  - `Sell { slot, take, lots }` (screen `Sell`, ADV-MKT-005): the adventure's check (§11), then the
    stack lowered (`StackChanged`) and the gain (`ResourceGained`).
  - `Buy { entry, pay }` (screen `Buy`, ADV-MKT-007): the adventure's check (§11; A-46 fixed:
    `MarketError::NoRoom`), then the price against the treasury (`CannotPay`), then paid
    (`ResourcePaid`), the artifact given (`HeroChanged`), the entry sold.
  - `Leave`: closes it (`DecisionClosed`).
  Errors: `GameCommandError::Market(MarketError)` (`Screen`, `SameResource`, `Amount`,
  `Recipient`, `Slot`, `LastCreature`, `Entry`, `NoPrice`, `NoRoom`) and `CannotPay`; a rejected
  answer changes nothing (ADR-0004). Nothing in the payload is trusted: the rate, the price and the
  most are the state's answer (ADR-0004, `docs/06-security.md`).
- **The view** `DecisionView::Market(MarketView)`: the hero, the object by type and anchor, the
  screens, the count; for `Trade` the 42 lots (by given and wanted resource); for `Sell` each slot's
  lot per resource; for `Buy` the seven entries (the artifact or empty) with the price per
  resource; for `Transfer` the recipients. *Agent's call:* the view carries the rates, so a client
  needs no copy of the computation (the env reads it too, §18).
- *As built* (T133b, decision 384; **game state hash v35**, E09 T39a took v34; **adventure
  ruleset part v24**, ADV-Q60 (a): `[params.market]`'s `cost`, `MarketParams::cost`, the checks
  reading it): `Decision::Market { owner, visit: MarketVisit }` (hash kind 10: the owner, the
  hero, the object's index, type and anchor), opened in `settle_outcome` from
  `AdventureOutcome::service` (`state/market.rs`' `open_market`, `MarketOpened`);
  `answer_market`: `Trade` through the adventure's `trade_lot` (the visit, the screen,
  `SameResource`, `NoPrice`), then the game's `Amount` against the treasury; `Transfer` through
  `check_screen`, then `Recipient` (the actor, or no player of the game), then `Amount`; `Sell`
  through `check_sale`, then `StackChanged` and `ResourceGained`; `Buy` through `check_buy`, then
  `CannotPay`, `ResourcePaid`, `HeroChanged` (`AddArtifact`), the entry sold; `Leave` closes.
  `GameCommandError::Market(MarketError)`. `legal_commands` lists `Leave` alone and it is the
  default answer (T135a lists the rest, §18). Tests: `h3-game`'s `tests/markets.rs` (its own
  ruleset: wood beside the gold, the three markets at five).

### 14. The Altar's decision (GAM-DEC-012; ADV-ALT-001 to -003; A-48 fixed)

- **`Decision::Altar { owner, visit: AltarVisit }`**, opened by `ServiceVisit::Altar`
  (`AltarOpened`, to the owner).
- **`Answer::Altar(AltarAnswer)`**: `Artifacts(Vec<ArtifactPlace>)` (ADV-ALT-002), `Creatures([u32;
  7])` (amounts by slot, ADV-ALT-003), `Leave`; refused as §11 says (`AltarError`: `Screen`,
  `Places`, `Kept`, `Amount`, `LastCreature`, `Nothing`). **A sacrifice names everything it takes
  in one answer** (decision 366: A-48 fixed): nothing lies on an altar between commands, so the
  default `Leave` at the turn's end loses nothing.
- **A sacrifice:** the artifacts removed (`HeroChanged` each, §11's order) or the stacks lowered
  (`StackChanged` each), then the total as `HeroChange::GainExperience` (`HeroChanged`; the level
  limit's cap, HER-EXP-003), then **the levels** as any gain's (GAM-DEC-002).
- **The levels inside the Altar** (survey §16's open point). *Agent's call:* the original's order:
  levels without a decision (one offer or none) are given at once and the Altar stays open; when a
  level-up opens, **the Altar closes before it** (`DecisionClosed`) and its visit is held as the
  visit's rest (GAM-DEC-007's `Rest::Visit`, amended to name it): once the level-ups are answered
  the rest visits the Altar again (`AltarOpened`), so a later sacrifice reads the new level's
  Learning (a Learning specialty grows by level), as the original's dialog open beneath its level-up
  dialogs. The player keeps one open decision (ADR-0017 point 6). Not chosen: the levels held
  until the Altar is left (a Learning specialist's later sacrifices would read the old level);
  the level-ups queued in front of the open Altar (two open decisions of one player). *Decided*
  (decision 371, the user at ADV-Q58).
- **For the clients** (decision 371; the map client lane's inbox told): the close and the return
  are the engine's bookkeeping, not a screen change. A client anticipates them and keeps the
  Altar's dialog drawn beneath the level-up dialogs (its `DecisionClosed` followed, after the
  level-ups, by `AltarOpened` for the same hero and object), unless the level-up covers the
  Altar entirely, as the original draws them (`altar_sacrifice` 0x5646A0 calls
  `hero_gain_experience` 0x4E33B0 with the dialog open, its level-up dialogs over it; the client
  confirms the drawing against the original). *Recorded* (batch E14's E14-17, T130): the level-up
  over the Altar without covering it, the Altar's places already empty and its total's texts the
  old ones until the level-up's OK: both drawn (the map client lane's inbox told).
- **The view** `DecisionView::Altar(AltarView)`: the hero, the screens its faction allows, each
  place's artifact with the experience it gives (with Learning) or none for a kept one, each slot's
  experience per creature before Learning (e) and for its most with Learning, the experience to
  the next level, `places`.
- *As built* (T134a, decisions 385, 386; game state hash v36): **the places are a set**
  (`h3_adventure::AltarPlaces`: the worn positions' bits in a `u32`, the backpack indices' in a
  `u64`; decision 386, the user's, so `Answer` stays `Copy`): distinct by construction, the
  change's order read from the bits; the part refuses an Altar under a core backpack above 64
  (`ALTAR_BACKPACK_PLACES`) and, *agent's call*, under more than seven army slots (`ALTAR_SLOTS`,
  the creatures' `[u32; 7]`). The adventure (`state/altar.rs`): `Effect::Altar` (the team's kind
  mark first, then `ServiceVisit::Altar(AltarVisit { hero, index, object_type, anchor })` for a
  hero whose class's faction has its screens; nothing opens for a hero the store lacks),
  `check_artifact_sacrifice`, `check_creature_sacrifice` (`Sacrifice`, not `Clone`; `AltarError`:
  the design's six and, *agent's call*, `Visit` as the market's; §11's orders), `sacrifice`
  (`Sacrificed`: the places to remove in order, or the stacks lowered, and the total),
  `altar_offers` (`AltarOffers` with `AltarArtifact`, `AltarStack`, `to_next_level`). The game
  (`state/altar.rs`): `Decision::Altar` (kind 11), `open_altar` (`AltarOpened`), `answer_altar`
  (the removals as `HeroChange::RemoveArtifact`, `HeroChanged` each, or `StackChanged` each, then
  `GainExperience`; the levels by `give_plain_levels`, split out of `next_level_up`: at a level
  with two offers the Altar closed, `Rest::Visit` held, then `LevelUpOpened`), `altar_view`;
  `legal_commands` lists `Leave` alone, the default answer `Leave`. Tests: the adventure's
  `state/tests/altar.rs` (the visit and its kind mark by faction, both sacrifices with Learning per
  item, the offers, the places' set, the texts), the part's bounds (`ruleset/tests/trades.rs`), the
  game's `tests/altar.rs` (the decision, both sacrifices' events, the level-up's close and the
  Altar's return, levels with no offers given at once).

### 15. The Hill Fort's decision (GAM-DEC-013; ADV-HFT-001, -002; D-74 the original's)

- **`Decision::HillFort { owner, visit: HillFortVisit }`**, opened by `ServiceVisit::HillFort`
  (`HillFortOpened`, to the owner).
- **The rows** as the original's (survey §13): each slot 0 no upgrade, 1 its price within the
  treasury (every resource), 2 not; **the whole** 1 when some row upgrades and the sum of the rows'
  prices is within the treasury, 0 when none upgrades, else 2; computed at each step, never stored.
- **`Answer::HillFort(HillFortAnswer)`**: `Upgrade(slot)` a row of state 1 (else
  `HillFortError::NoUpgrade` or `CannotPay`): its price paid (`ResourcePaid` per resource, the core's
  order), the stack upgraded (`StackChanged`); `All` when the whole is 1 (else the same errors):
  every row of state 1 in slot order, each paid and upgraded; `Leave`. Free rows (level 1) pay
  nothing. Nothing is tested at the press but the states (the original's), so the states are the
  validation.
- **The view** `DecisionView::HillFort(HillFortView)`: each slot's row (the upgrade, its price,
  its state) and the whole's price and state.
- *As built* (T134b1, decision 387; game state hash v37): the adventure (`state/hill_fort.rs`):
  `Effect::HillFort` (the team's kind mark first, then `ServiceVisit::HillFort(HillFortVisit {
  hero, index, object_type, anchor })` at every visit), `hill_fort_rows` (`HillFortRow`: each
  stack of the army in its order with its creature, count, upgrade or none, and price; the gold the
  market parameters' cost resource, *agent's call*: the part refuses a Hill Fort without
  `[params.market]`), `check_upgrade`, `check_upgrade_all` (`HillFortUpgrade`, not `Clone`;
  `HillFortError`: `Visit`, `NoUpgrade(slot)` and, *agent's call*, `Nothing` for an `All` without
  a row that upgrades), `upgrade`. The game (`state/hill_fort.rs`): `Decision::HillFort` (kind 12),
  `open_hill_fort` (`HillFortOpened`), `answer_hill_fort` (the summed price against the treasury,
  the existing `CannotPay`, then each row paid and `StackChanged` in slot order), `hill_fort_view`
  (`HillFortView` with `HillFortRowView` and `HillFortState`: `NoUpgrade`, `Affordable`,
  `TooDear`); `legal_commands` lists `Leave` alone, the default answer `Leave`. Tests: the
  adventure's `state/tests/hill_fort.rs`, the part's refusal, the game's `tests/hill_fort.rs`.

### 16. The factory through the hire question and dialog (GAM-DEC-008, ADV-HIR-002 amended)

- The question is an offer (`Offer(Consent)`, [`Decline`]); accepted, the hire dialog opens
  (`HireOpened`, its source the factory by type and anchor, `HireSeen` as a dwelling's) with the
  three machines and their counts (1 or 0); `HireAnswer::Buy { creature, amount: 1 }` checked by
  the adventure (§11), then the treasury (`CannotPay`), then paid and the artifact given
  (`HeroChanged`, not `CreaturesJoined`); the dialog of three stays open; `Close` closes it.
  `hire_mosts` reads the stock's 1 or 0 with the gold bound, so `legal_commands` lists a machine
  only while the hero lacks it and can pay (as the original's slider's most 1 or 0).
- *As built* (T134b2, decision 388; no version moved): the adventure (`state/factory.rs`,
  `state/dwellings.rs`): `Effect::Factory` asks at every visit (`ask_factory`: `OfferKind::Hire`
  naming the machines, no mark), `accept_hire` opens the dialog for it as for a camp;
  `hire_stock((part, heroes), dialog)` reads the dialog, not its source alone (the user's choice:
  each machine 1 while the dialog's hero holds none of its artifact, worn or carried, else 0; the
  game's `GameState::hire_stock(rules, dialog)` and the env's one call follow);
  `dialog_room((core, part, heroes), dialog, creature)`, *agent's call*, the room of any dialog (a
  machine's 1 while `HeroChange::AddArtifact`'s check finds its artifact a position, else 0; any
  other source's the army's `hire_room`, whose signature the town lane's tests use);
  `check_hire((core, part, heroes), …)` with a machine's `Amount` (a held one's count 0) then
  `NoRoom`, its purchase `Goods::Machine` (`HirePurchase::machine`: the hero and the artifact),
  which `hire` leaves alone (no count, no army, no event). The game (`state/hire.rs`): after
  paying, `HeroChange::AddArtifact` through `change_hero` (`HeroChanged`); `hire_mosts` bounds by
  `dialog_room`. *Agent's call:* the factory keeps no count (the original's three share one count
  of 4 000, lowered by each purchase, which no game reaches). Tests: the adventure's
  `state/tests/factory.rs`, the game's `tests/factory.rs`, `h3-map`'s `tests/trades.rs` (every
  trade's step entered; the factory's dialog of the base pack's three at their costs).

### 17. The observation, the events and the wire (ADR-0016 point 6; ADR-0018 point 5)

- **New events** (`GameEvent`, each to the owner): `MarketOpened`, `AltarOpened`,
  `HillFortOpened` (the player, the hero, the object's type and anchor); every other change is told
  by the existing `ResourcePaid`, `ResourceGained` (the transfer's to the recipient alone),
  `StackChanged`, `HeroChanged`, `LevelUpOpened`, `LevelGained`, `DecisionClosed`, `OfferOpened`,
  `HireOpened`, `KindKnown`.
- **The views** (§13 to §15) to the owner alone; another player learns nothing of them (a transfer
  shows its recipient its gain only).
- **The wire** (the global trap: `h3-game`'s new answers, events, errors and the views' structs
  break `h3-protocol`'s build until mapped): each building task that adds them maps them in the same
  pull request (`cross-lane`, with the user's consent at its start, as T115b; or the server lane
  first), each with the protocol's next version and a fold test on golden games; the server lane's
  inbox told at each.
- *As built* (T133b, decision 384; **protocol v75**, `cross-lane`): `Answer::Market`
  (`MarketAnswer`, resources by handle), `GameReject::Market` (`MarketReject`), `Decision::Market`
  (`MarketWindow` with `MarketScreen`, `Lot`, `TradeLot`, `SaleLots`, `ResourceLot`,
  `StockEntry`), `GameEvent::MarketOpened`; the fold leaves a market's view untold as a
  University's (`SRV-STUB-5` extended). With the answer the replay words (**format v21**, the client lane's E27 M4 T3a having taken v20: `answer
  market trade|transfer|sell|buy|leave`, §18's) and the env's `Leave` on the closing entry
  (**encoding v14**, no new entry) came in the same pull request: a new answer breaks both
  crates' exhaustive matches. T134a (**protocol v76**, `cross-lane`): `Answer::Altar`
  (`AltarAnswer`: `Artifacts { worn, backpack }` the places' bits, `Creatures { amounts }`,
  `Leave`), `GameReject::Altar` (`AltarReject`), `Decision::Altar` (`AltarWindow` with `AltarItem`,
  `AltarStack`), `GameEvent::AltarOpened` (folded as a market's, `SRV-STUB-5` extended); the replay
  words (**format v22**: `answer altar artifacts <place> …`, `creatures <amount> …` ×7, `leave`)
  and the env's `Leave` (**encoding v15**). T134b1 (**protocol v77**, `cross-lane`):
  `Answer::HillFort` (`HillFortAnswer`: `Upgrade { slot }`, `All`, `Leave`), `GameReject::HillFort`
  (`HillFortReject`), `Decision::HillFort` (`HillFortWindow` with `HillFortRow`, `HillFortState`),
  `GameEvent::HillFortOpened` (folded as an Altar's, `SRV-STUB-5` extended); the replay words
  (**format v23**: `answer hill-fort upgrade <slot>`, `all`, `leave`) and the env's `Leave`
  (**encoding v16**).

### 18. The env, replays, maps, properties, benchmarks

- **`legal_commands`** lists a representative answer per action, as the hire dialog lists each
  creature's most (*agent's call*): a market's trades of **one lot** per pair it can pay, its sales
  of one lot **for gold** per slot (the army's last creature kept), its purchases **in gold** per
  unsold entry it can pay, its `Leave`; an Altar's two **alls** (the original's `altar_all_to_altar`
  0x564340: the worn positions 0 to 15, then the backpack, until `places`, the kept ones skipped;
  `altar_all_creatures` 0x564CF0: every stack whole but one creature of the lowest occupied slot)
  where the screen allows and the total is above 0, and its `Leave`; a Hill Fort's `Upgrade` per row
  of state 1, `All` when the whole is 1, `Leave`. **No transfer** is listed (as the management
  commands, decision 99: a client sends one from the view). Any other answer the view allows is
  still legal for `apply`.
- **The env** (`h3-sim`, the AI lane's; `cross-lane` with the user's consent, as versions 7 and 8):
  the action encoding's next version (v13 today; the town lane's T39a plans v13 too: whichever
  merges first) appends the entries of the listed answers: `MARKET_TRADE + 42` (the pairs, given
  then wanted in the core's order), `MARKET_SELL + 7` (slots), `MARKET_BUY + 7` (entries),
  `ALTAR_ARTIFACTS`, `ALTAR_CREATURES`, `HILL_FORT_UPGRADE + 7`, `HILL_FORT_ALL` and one `LEAVE`
  for the three kinds: 67 entries, tile 0 alone, legal where `legal_commands` lists the answer. The
  factory needs none (the offer's and the hire dialog's entries). An amount head stays the AI
  lane's. The AI lane's inbox told. *Decided* (decision 372, the user at ADV-Q58: the `cross-lane`
  change consented for T135a).
- *As built* (T135a, decisions 389, 390): the listing as above, the sales and purchases in the
  market parameters' cost resource, the Altar's artifacts' all over every worn position (the
  button's 0 to 15 left, decision 390), each kind's answers before its `Leave`
  (`GameState::market_answers`, `altar_answers`, `hill_fort_answers`); the env's version 17
  (version 13 went to the town lane's taverns, 14 to 16 to the closing roles) with **66** entries,
  the `LEAVE` being each kind's closing role: `MARKET_TRADE` 37 to `HILL_FORT_ALL` 102. A sale's,
  a purchase's and a sacrifice's entry encodes the listed answer alone; an entry of no listed
  answer decodes to the answer it names (an Altar's empty sacrifice), which `apply` rejects.
- **Replays** (the adventure replay format's next version, v19 today; the town lane's T39a plans
  v19 too): `answer market trade <give> <take> <lots>`, `answer market transfer <resource>
  <amount> <player>`, `answer market sell <slot> <take> <lots>`, `answer market buy <entry> <pay>`,
  `answer market leave`; `answer altar artifacts <place> …` (`w<position>`, `b<index>`, the
  management lines' places), `answer altar creatures <amount> …` (seven, by slot), `answer altar
  leave`; `answer hill-fort upgrade <slot>`, `answer hill-fort all`, `answer hill-fort leave`;
  resources by their index in the core's order. The factory's are the existing `answer offer` and
  `answer hire`.
- **Maps:** `e08-trades-36.h3m` (two players, every class twice, heroes of the three alignments,
  a Black Market under bans) with a random replay and a scripted one (each answer kind, a
  sacrifice with a level-up and the Altar's return, a transfer, a refused purchase with a full
  hero); `e08-trades-144.h3m` (XL, two levels) for the workloads.
- **The plans** (`h3-map`'s `tests/play.rs`): `TRADES`, `Step::Market`, `Step::Altar`,
  `Step::HillFort` and the factory's steps; oracles `check_trades` (each trade's paid and gained
  amounts as its lot says; a transfer's sum kept; the army never emptied; the stock's sold entries
  only grow; an artifact bought appears on the hero, a sacrificed one is gone; the experience the
  sum of the items'), and the conservation, isolation and management checks taught the new answers.
- **Benchmarks** (`homm3-bench-regression`): the 1 058 objects stop being inert blockers on 148
  shipped maps and their entrances end paths; their answers enter the random agent's draws (the
  lane's trap: every random game's course moves). The `random-play-*` workloads re-baselined with
  the reason, a new `random-play-trades-36`, `build-trades-xl-144` (the Black Markets' draws); the
  reach is unchanged (an entered object as any).

### 19. Versions, rule ids, quirks, stubs

- **Versions** (each the next free at its task): adventure ruleset part (v22, §6), map import
  (v26, §7), adventure state hash (v27, §8), game state hash (v34: the three decisions and the
  rest's name), adventure replay format (v19), the env's action encoding (v13, the AI lane's),
  `h3-protocol`'s protocol (one per wire task, the server lane's). The core ruleset hash and the
  pack format stay (the core is untouched; `[market]` is the adventure part's).
- **Rule ids** (survey §19): ADV-MKT-001 to -008, ADV-ALT-001 to -003, ADV-HFT-001, -002,
  ADV-WMF-001, GAM-DEC-011 to -013; amended: GAM-DEC-007 (`Rest::Visit` after an Altar's level-ups),
  GAM-DEC-008 and ADV-HIR-002 (the factory as a source of war machines), ADV-OBJ-003 (the seven's
  entrances end a path), ADV-BON-009 (the Altar's and the Hill Fort's kind marks), HER-EXP-001 (the
  Altar's products, a citation). The town lane's rules cite ADV-MKT-002 to -007 when its Marketplace
  reuses them.
- **Quirks** (`docs/rules/quirks-adventure.md`, with the build): **ADV-QUIRK-48** (A-46, fixed:
  the purchase refused before paying; the town lane's Artifact Merchants too), **ADV-QUIRK-49**
  (A-47, harmless and accepted: a special artifact destroyed for 0 beside others), **ADV-QUIRK-50**
  (A-48, fixed: nothing on the altar between commands). No profile flag (decisions 366, 367: fixed
  and accepted outright).
- **Stubs:** none. The computer's visits are E10's behaviour, not a stub; the town's artifact sale
  (screen 3) is the town lane's increment 5.

## Part D — batch E14 and the cut

### 20. Batch E14: the spec and the recipe (survey §18; decision 368)

Survey §18's fifteen rows in one run, as planned (SoD on Easy, hot seat, Red and Blue, days 1 and 2
and day 29 for E14-3, maps `h3-experiment` writes, the client lane's harness), with this note's
changes:

- **E14-16** (added; §0's correction): Red's hero steps onto each of the seven's entrances: where
  it stands at the window, after closing it and after a revisit with Space (the entrance each time;
  the move ended there); a step from above toward an Altar's entrance (refused, none from above).
- **E14-17** (added; §14): a hero of the Learning specialty one level short sacrifices a relic at
  an Altar: the level-up dialog over the Altar, then the Altar still open; a second sacrifice of a
  relic reads the new level (its total one more or equal, by the specialty's 5 % per level). The
  reading predicts the order; the total is HER-EXP-001's (Q-0005).
- **E14-4** stands as the original's loss (A-46): the engine refuses that purchase (decision 366);
  the row confirms what is fixed.
- **The spec** (`h3-experiment`, battle-owned: `cross-lane`; the writer this lane's): the seven
  classes by template (no body), a hero's secondary skills (Advanced Learning; a Learning
  specialist by its type), war machines and backpack at the start (64 artifacts for E14-4, the
  Grail and a scroll for E14-9), the map's banned artifacts (E14-3), Blue's starting resources for
  E14-6, an RoE map (E14-14; the writer's RoE format, as T56's batch E7 map). Two specs:
  `experiments/specs/E08-batch-14.toml` (SoD) and `E08-batch-14-roe.toml`.
- **The harness** (`clients/capture/`, common): reads of the market window (the screen, the pair,
  the slider's most; `market_screen` 0x6AAAA4, `market_kind` 0x6AAAC4, `market_count` 0x6AAA98), the
  Altar's dialog (+0x78 the total, the places, the slider's most), the Hill Fort's rows (prices,
  states, the whole +0x298), the Black Markets' records (game +0x1F680); the hero's experience,
  artifacts, army and tile and the treasury (earlier batches' reads); the dialogs' order (E14-17).
- **The recipe** (`clients/capture/batches/E08-batch-14.toml`): each row's clicks and keys with its
  reads before and after; E14-3's day 29 by 28 turns' ends of both players; E14-6's Blue read on
  Blue's turn. Not in the batch: the computer's visits (E10), network play, the town's
  Marketplace, A-48 (a timed turn) unless the harness sets the hot seat's turn time.
- **Scored** in the survey's new §21 (each row: as predicted, or the finding and its effect on this
  note and the drafts).
- *As built* (T128): the specs `experiments/specs/E08-batch-14.toml` (36 × 36 grass; Red's eight
  heroes and Blue's two, two towns for day 29; the bans leave seven treasures, seven minors and
  three majors) and `E08-batch-14-roe.toml`, their headers listing the changes: E14-17 a Fortress
  hero at level 1 with seven Expert skills and Basic Learning (no hero type has the Learning
  specialty): a treasure's 1 050 gives a level offering Advanced Learning alone, then 10 Gnolls
  read 55; heroes placed with experience take their levels at load (GAM-SET-002), so E14-8's and
  E14-9's know eight Expert skills; E14-12 adds four Air Elementals (25, VCMI 24); E14-13's 3 000
  Pegasi cost more than Blue holds; Blue's treasury read from memory (no field); every row on day
  1; E14-16 inside the other rows' visits. The spec gained an RoE hero's own army (no `primary`,
  no `spells`).
- *Scored* (T130, survey §21): every row as predicted but E14-14: the RoE-format map played at the
  game's version SoD (the version is the scenario dialog's, from the players' editions), its Air
  Elementals upgraded; decision 378: the enabled packs decide (§4, §7, exit criterion 6). D-71 to
  D-74 the original's, A-46 and A-47 seen; E14-17's level-up drawn over the Altar (§14's note).

### 21. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared:** the `shared/` task (`cross-lane`): the two drafts applied with batch E14's outcome
(ADR-0016 points 1, 4, 6, ADR-0017 points 1, 6, 8 to 10); `docs/03-domain.md`'s terms (market, a market's lot, the Black Market's stock, a transfer, a
sacrifice, the Hill Fort's upgrade, the War Machine Factory); `docs/06-security.md`'s lines (the
amounts, the recipient and the price validated; the stock server-only). **`cross-lane`
elsewhere:** batch E14's spec (`h3-experiment`), the harness (common), `h3-protocol` at each wire
task (the server lane's), the env (`h3-sim`, the AI lane's). **The inboxes:** the town lane's (the
rates' home and API, §2; A-46's fix and D-71 for its merchants and guild, already told; screen 3
theirs; the versions both lanes plan: map import v26, replay format v19, env v13), the server
lane's (the wire per task), the AI lane's (the 67 entries, v13), the map client lane's (the seven
types, the two kind marks' hints), the client lane's (the three windows' views for the shell's
later screens; informational).

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut confirmed
at its start). **Batch E14 runs before the build and before the shared task** (decision 368), so the
ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T128** | Batch E14's maps: the writer's seven classes (by template, no body; 213 and 221 in AB and SoD), the spec's fields (`cross-lane`), the spec and its two maps (§20) |
| 3 | **T129** + **T130** | T129: the harness's reads, the recipe, the run. T130: scored (survey §21) |
| 4 | **T131** | `shared/` (`cross-lane`): the two drafts with T130's outcome, the glossary, `docs/06-security.md` |
| 5 | **T132** | T132a: the five kinds, `[market]` and the rates' module with its tables (§1 to §5), the packs, the eighth original-data check (§6; part v22). T132b: the import and the writer's classes for fixtures (§7; map import v26); a step onto one of the seven refused as an unbuilt visit's until its task |
| 6 | **T133** | The market: T133a the state and the draw (§8, §9; adventure state hash v27), the visits' outcome (§10) and the adventure's checks (§11's sale and purchase); T133b the decision (§13; game state hash v34) with its wire (`cross-lane`) |
| 7 | **T134** | T134a: the Altar (§11's sacrifices, §14 with `Rest::Visit`, the kind mark) with its wire. T134b: the Hill Fort (§15) and the factory (§16, GAM-DEC-008 amended) with their wire |
| 8 | **T135** | T135a: `legal_commands`, the env's entries (v13, `cross-lane`) and the replay words (v19). T135b: the two maps, the replays, the plans and oracles, the benchmarks, `--with-original-data` (§18) |
| 9 | **T136** | The audit against the exit criteria; increment 14 closed; increment 15 refined |

*Agent's call:* the market first (four classes, the shared rates the town lane waits on), the three
services after it on the same decision shape; the env and replays once every answer exists, so
the encoding and the format move once.

**Increment 14's exit criteria:**
1. The seven classes import typed by the base and ab packs' five kinds (221 as the Trading Post),
   one entrance each, **entered**, none from above; the census holds; the eighth pack check passes
   against the executable's tables (ADV-MKT-001, ADV-OBJ-003).
2. The rates are the original's at every count 1 to 10 (binary32 steps, half up in 53 bits): the
   survey's tables, D-70's and D-71's cases, as lots of at least one (ADV-MKT-002).
3. A Trading Post trades resources by whole lots and transfers 1 : 1 to another player still in
   the game; the Freelancer's Guild buys any creature of the army by lots, the army keeping one
   (ADV-MKT-003 to -005; D-71 the original's).
4. Each Black Market draws three treasures, three minors and one major from the one pool when the
   game is built, never again; a purchase pays the price, gives the artifact and empties the entry,
   and is refused before paying when the hero cannot receive it (ADV-MKT-006, -007; A-46 fixed).
5. The Altar opens the screens the hero's class's faction allows; a sacrifice names its artifacts
   (at most 23, never the Spell Book or the Catapult, the special ones for 0) or its creatures (one
   kept) in one answer, its total above 0, each item's experience with Learning truncated; its
   levels come as the original's, the Altar returning after a level-up; leaving loses nothing
   (ADV-ALT-001 to -003, GAM-DEC-012, GAM-DEC-007; D-72, D-73 the original's; A-47 accepted, A-48
   fixed).
6. The Hill Fort upgrades a stack of a creature with an upgrade in the enabled packs (none for the
   four elementals under the base pack alone; decision 378) for
   the difference times the count, the gold scaled by its level and truncated once, a level-1
   stack free; one row at a time or all when the whole is affordable (ADV-HFT-001, -002,
   GAM-DEC-013; D-74 the original's).
7. The War Machine Factory asks, then sells each machine the hero lacks at its cost into its
   position, never the Catapult (ADV-WMF-001, GAM-DEC-008, ADV-HIR-002).
8. A viewer sees the seven by type; a Black Market's stock only in its visitor's open decision,
   nothing kept after; the two kind marks as increment 12's (ADV-MKT-008, ADV-BON-009,
   ADV-VIS-006); the events and the wire as §17.
9. Every answer is validated against the state and the trusted actor; a rejected one changes
   nothing; the system actor's default leaves each window with nothing lost (GAM-DEC-011 to -013).
10. Every new rule has its id, source and tests; the gates pass (coverage, determinism with the new
    and rehashed replays, the benchmarks at their baselines and the new ones baselined); batch E14
    run and scored (T130), the consensus rows `verified`.

### 22. Risks and open points

- **The versions the town lane plans** (map import v26, replay format v19, env v13, protocol v73 for
  its T39a): whichever merges first takes the number, the other rebases and bumps (§19).
- **The random agent in a market:** `legal_commands` gives a market about 60 answers and one
  `Leave`, so a random game trades many times per visit; the workloads' step counts and the env's
  rollouts grow (Revisit's precedent: a step cap is the AI lane's). T135b measures it; a lower
  weight for the answers is the bots' call, not the engine's.
- **The halves:** the rates rest on the x87's 53-bit precision (installation.md §7.10's
  inference, Q-0005's) and the binary32 stores read in the code; the tests pin the halves (survey
  §4's 17 and 37) and a wrong reading shows in E14-1 and E14-5.
- **RoE unread:** RoE's own executable is not read (E11's T1 may revisit); the Hill Fort's RoE
  side rests on the packs (decision 378) and SoD's test of version 0, which only an RoE campaign
  reaches in this edition (survey §21).
- **1 058 objects stop being inert** on 148 shipped maps: any test that used one of the seven
  classes as a plain blocker breaks (`TRAPS.md`), and the shipped-map counts move once (T132b,
  T135b).
- **The computer's visits** (`ai_black_market_visit` 0x5243F0, `ai_hill_fort_visit` 0x527BB0,
  `ai_war_machine_factory_visit` 0x524FC0) are E10's and E13's; the engine plays every player as a
  human.
- **Not read:** the windows' redraws and texts (the clients'), network play's transfer message, the
  town's screen 3 and the merchants' monthly draw (the town lane's).
