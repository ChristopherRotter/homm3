# E08 design note

Increment 9's note: mines, the week and owned objects (E08 decision 192). Increment 8's note
(boats and the sea) is in `E08-log.md` ("Increment 8's design note at its closure", moved by T76);
the code's comments cite these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 9's
amendments of ADR-0013 (points 3, 13.6, a new point 16), ADR-0016 (points 1, 3, 4, 5, 6) and
ADR-0017 (points 1, 3, 4, 6, 8, 10, 11), drafted in
`docs/lanes/adventure/adr-drafts/*-increment-9.md` and applied by the increment's `shared/` task;
written by T77b from T77a's survey and the answers at ADV-Q36 and ADV-Q37 (the epic's decisions
192, 193, 196–199). The task that builds a section deletes it; the file goes when the epic
closes. Facts and sources: survey `docs/rules/surveys/E08-economy.md` (§ numbers are the
survey's unless they name this note); the proposed rule ids are the survey's §16.

Written whole (decision 200): **part A**, the hero side (`h3-heroes`, the core content); **part
B**, the map (`h3-map`, `h3-adventure`); **part C**, the game, the env, batch E9 and the cut.
*Agent's call (change freely)* marks a choice the user accepted at the checkpoint (ADV-Q38, decision
204) and may still change; decision 201's two changes are written in.

## Part A — the hero side (`h3-heroes`, the core content)

### 1. The data (the core content and packs: shared)

- **Four effect kinds** (ADR-0013 point 3's draft), compiled by `h3-content` and read by
  `h3-pack` (the shared task): `daily_resource { resource, amount }` (amount 1 to `MAX_GAIN`, a
  known resource), `mana_regain { points }` and `extra_mana_regain { points }` (1 to 999),
  `full_mana`. None reaches `BattleEffect`; the battle's derivation skips them as it skips
  increment 8's.
- **Two hero parameters:** `base_mana_regain = 1` (the original's `skill_mysticism` 0x63E9C8 entry
  0, every hero's) and `regain_specialist_bonus = 1` (the `inc ebx` at 0x4E3FA8).
- **The packs** (`base`, `ab`, `sod` through the harvester's gap-fill, battle-owned: the shared
  task is `cross-lane`, as T70):
  - the resource artifacts `daily_resource` 1: Everflowing Crystal Cloak (109) crystal,
    Ring of Infinite Gems (110) gems, Everpouring Vial of Mercury (111), Inexhaustible Cart of
    Ore (112), Eversmoking Ring of Sulfur (113), Inexhaustible Cart of Lumber (114) wood; the
    gold artifacts' gold: Endless Sack (115) 1 000, Endless Bag (116) 750, Endless Purse (117)
    500 (`player_gold_income` 0x4C7930); the Cornucopia (140, `sod`) 4 each of mercury, sulfur,
    crystal and gems (*agent's call*: its four parts' 1 each count through the combination, so
    a worn Cornucopia gives the original's 5, `count_worn_artifact` 0x4BABD0 counting the
    assembled artifact alone, survey §17);
  - `extra_mana_regain`: the Charm of Mana (73) 1, the Talisman of Mana (74) 2, the Mystic Orb of
    Mana (75) 3; `full_mana`: the Wizard's Well (138, `sod`);
  - Estates' levels `daily_resource` gold 125, 250, 500 (`skill_estates` 0x63EA18); Mysticism's
    levels `mana_regain` 2, 3, 4 (0x63E9C8);
  - the **thirteen resource and gold specialists** (specialty kind 2, `hero_specialty_pointer`
    0x679C80), whose specialty the packs leave empty today: `base`'s Caitlin, Jenova, Rissa,
    Aine, Octavia, Calid, Clavius, Nagash, Damacon, Sephinroth, Saurug, `ab`'s Gelare and
    Grindan: a specialty of one `daily_resource`, 1 of the table's resource from wood to gems,
    350 for gold (0x4E4410). The shared task reads each one's resource from the table and
    checks it against the wiki's hero pages.
  The "pending: E08-E10" comments on those artifacts in the packs go with them.
- **A creature ability of the core** (the user at ADV-Q38, decision 201: the Crystal Dragons'
  crystal is an innate creature ability, modelled as one): the core creature (`h3-content`'s
  creature type, beside its growth and flags) gains `abilities`, a closed core kind for what a
  creature does outside battle, its first `weekly_resource { resource, amount }` (GAM-RES-005;
  VCMI's `SPECIAL_CRYSTAL_GENERATION`; the original tests creature 133 in `player_daily_income`
  0x4B8C32). The battle part's abilities (ADR-0012) stay the battle's; the core's are read by the
  day's income (§14), never by the battle. The `ab` pack's Crystal Dragon: `abilities = [{ kind =
  "weekly_resource", resource = "crystal", amount = 3 }]` in `core.toml` (the gap-fill).

### 2. The derivations (`h3-heroes`; HER-MAN-002, GAM-RES-004, HER-ADV-009 amended)

- **`daily_income(details, core) -> Resources`** (ADR-0013 point 16.1): the `daily_resource` of
  every artifact in a worn position and of every part of a worn combination, **each copy
  counting** (a new walk over the positions, not the "each type once" set the movement and the
  battle use); of each secondary skill's level, grown when the hero has a skill specialty on
  that skill by `⌊amount × (10 000 + bp_per_level × level) ÷ 10 000⌋`; of the specialty's
  effects. Exact integers in every profile: the original keeps the factor in the x87 register
  (`hero_gold_income` 0x4E43DA to 0x4E43F2; only the 0.05 is a single, 0x63EAE4), so its
  product's excess (at most about 10⁻⁴) never reaches the next integer (the products are
  multiples of 0.05). Saturating per resource.
- **`day_mana(details, core) -> u16`** (point 16.2; HER-MAN-002): `max` = HER-MAN-001's; with
  `full_mana` worn (or a combination holding it), `max(points, max)`; else
  `max(points, min(points + regain, max))`, so points above the maximum stay. `regain` = the
  highest `mana_regain` of the sources, else `base_mana_regain`; with that skill and a skill
  specialty on it, `⌊regain × (10 000 + bp × level) ÷ 10 000⌋ + regain_specialist_bonus`
  (`hero_mana_regained` 0x4E3F5F: no growth for a specialist without the skill); then each worn
  type's `extra_mana_regain` once (worn or a part of a worn combination: the original's two
  scans of the worn slots and `hero_wears_artifact` 0x4D91F0 for the combination). The Well's
  parts (the three mana artifacts) do not matter once the Well fills.
- **`sea_day_points`** gains the Lighthouses' input (point 13.6): `lighthouse_points: u32`,
  added after the artifacts' points, saturating, never grown by Navigation; the adventure
  passes its owner's Lighthouses' points (§7).

### 3. Rule ids, quirks, shared needs (the hero side)

HER-MAN-002 (the day's spell points), GAM-RES-004 (the heroes' income: its derivation here, its
payment the game's), HER-ADV-009 amended (the Lighthouses' input). No quirk. Shared: the four
kinds, the two parameters, the core creature's abilities, the packs and the gap-fill (T81).

## Part B — the map (`h3-map`, `h3-adventure`)

### 4. The import (`h3-map`; map import v15)

- **Classes typed** (`codes::object_type`): 53 by subclass (0 to 6 the seven mines, 7 the
  abandoned mine), 220 as 53.7 (the original's `object_class_values` 0x63A6E4; its pictures
  stay the template's), 42 the Lighthouse, 87 the Shipyard, 112 the Windmill, 109 the Water Wheel,
  55 the Mystical Garden. Classes 42, 53 and 87 are `Scenery::Ownable` already (GAM-BAT-011), and
  220 becomes one as 53.
- **The bodies** (the reader's `Body::Mine`, `Body::AbandonedMine`, `Body::Owned` exist): an
  owner `0xFF` is none; another colour is the player that colour is (`players`' map), and a
  colour no player of the scenario plays is refused (`ImportError::ObjectOwner(index)`, as
  `HeroOwner`; ADV-OWN-002). The abandoned mine's set (the body's first byte, bits 0 to 6) goes
  to `ObjectSetup` as the resources it may draw; an empty set or a bit past the core's
  resources is refused (`ImportError::MineResources(index)`, A-31's bound, `ADV-QUIRK-33`).
- **An ownable object whose entrance is on water** is refused as `Unsupported::OwnedOnWater(index)`
  (none on the shipped maps; how a hero reaches an entered object on water was not read, survey
  §4).
- **The timed events** (`RawMap::events`) become `GameSetup::events` in the map's order
  (GAM-CAL-004): each `TimedEventSetup` with its players as player ids (the colours that play; a
  colour nobody plays dropped), `human` (the reader gives `true` before SoD, as the original's
  0x4FC1A0), `computer`, `first_day` = the file's day + 1 (0x4FC2E0), `interval` (0 once), the
  seven resources as `(ResourceId, i32)` pairs without zeros, and `message: u16`, the event's
  index in the list (the text is the client's). None is refused; an event for no player that
  plays is kept and never fires.
- **The writer:** built by T78 (`E08-log.md`, T78).

### 5. The kinds and the part (`h3-adventure`, `h3-pack-adventure`; adventure ruleset part v14)

New `ObjectKind`s (ADR-0016 point 4's draft), each with a visit (`Effect`):

| Kind | Parameters | Base pack types |
|---|---|---|
| `Mine` | `resource`, `daily` (its amount a day), `battle_terrain: Option<TerrainId>` | `base:sawmill` (wood 2), `alchemists_lab` (mercury 1), `ore_pit` (ore 2), `sulfur_dune` (1), `crystal_cavern` (1, subterranean), `gem_pond` (1), `gold_mine` (gold 1 000, subterranean) |
| `AbandonedMine` | `guards: CreatureTypeId`, `count: [u32; 2]`, `battle_terrain` | `base:abandoned_mine` (Troglodytes, 100 to 200, subterranean); its daily amount the drawn resource's `Mine` type's |
| `Lighthouse` | `sea_points` | `base:lighthouse` (500, `movement_bonus_lighthouse` 0x698B0C) |
| `Shipyard` | `price: Resources`, `boat_type: u8`, `offsets: Vec<(i8, i8)>` | `base:shipyard` (1 000 gold and 10 wood; type 1; the twelve of 0x67FAA8 in order) |
| `Windmill` | `resources: Vec<ResourceId>`, `amount: [u32; 2]` | `base:windmill` (mercury to gems, 3 to 6) |
| `WaterWheel` | `first`, `later` (gold) | `base:water_wheel` (500, 1 000) |
| `Garden` | `gifts: [(ResourceId, u32); 2]` | `base:mystical_garden` (5 gems, 500 gold) |

The ownable kinds (`Mine`, `AbandonedMine`, `Lighthouse`, `Shipyard`) are entered (`Visit::Enter`);
the weekly kinds are visited from the neighbouring tile (`Visit::Neighbour`, survey §9). The
abandoned mine's daily amount is looked up by its drawn resource among the part's `Mine` types
(the compile checks that each resource has exactly one). **Parameters** (`[params]`):

- `[params.calendar]`: `week_days = 7`, `month_weeks = 4` (GAM-CAL-002; the game reads them
  through the part, as the starting resources).
- `[params.monsters]` gains the growth: `growth_bp = 1000` (a tenth), `growth_unit = 16` (the
  sixteenths), `growth_cap = 4000` (ADV-MON-013; D-49: VCMI's thousandths are `growth_unit =
  1000` by data, should batch E9 revert it).
- `[params.months]`: the odds of a normal month, a creature's and the Plague (5, 4, 1 of 10), the
  twelve creatures (`month_creatures` 0x63E678 by key), `spawn_one_in = 200`, `spawn_factor = 2`,
  `spawn_aggression = [1, 10]` (ADV-MON-014, GAM-CAL-003).

### 6. The state (`h3-adventure`; adventure state hash v17)

- **The static map** lists the ownable objects (`ownables: Vec<usize>`, object order) and the
  weekly ones (`weeklies: Vec<usize>`); `Resolved` gains `Mine { resource, guards: u32 }` (an
  abandoned mine's drawn resource and Troglodytes' count; for the seven mines `Nothing`) and
  `BoatTile(Pos)` (a Shipyard's tile found at the build, ADV-SHY-001; `Nothing` when none: Hoard's
  Shipyard at (62, 38, 0)).
- **`owners: Arc<[Option<PlayerId>]>`** by ownable index, built from the setup's owners
  (ADV-OWN-002); a flag copies it (a few hundred bytes on the largest shipped map).
- **A mine's guards** live in `army_left` (renamed in the code's comments "the armies on objects";
  the type stays): the Leave Guards dialog's group written at its close, a battle's survivors;
  until written, an abandoned mine's guards are `Resolved::Mine`'s count of the kind's creature in
  slot 0, the seven mines' none. An empty army written means no guards (the entry stays, so the
  drawn Troglodytes do not come back).
- **`weekly: Option<Arc<[Weekly]>>`** by weekly index: each `{ resource, amount, marks: u8 }`
  (the Windmill's resource and amount, the Water Wheel's gold as `(gold, n)`, the Garden's gift
  and `amount` 0 when emptied; `marks` the players that visited, bit by player id, never cleared:
  *agent's call*, the marks with the contents rather than in the visits' block, one write a
  visit). None for a map without weekly objects.
- **`fractions: Table<u8>`** by object index (the persistent two-level table of `army_left`): a
  monster's sixteenths, none for 0.
- **The live presence** (`Occupancy`, renamed in its comments): beside `removed`, `unblocked` and
  `unvisitable`, the **added objects** (`added: Vec<MapObject>`, index `objects().len() + n`), a
  grid of the tiles they block and are visited from, and a grid of the tiles near an added
  monster (its 3 × 3, for `guard_at`'s early test that the static map's `near_monster` gives the
  placed ones). `tile_object`, `guard_at`, `live`, the observation and the passage read the
  added objects only where their grid's bit is set, so a map without a spawn pays one test a
  tile and a fork nothing more. **Every read of `map.objects()[index]` goes through a state
  method that knows the added objects** (43 sites in `h3-adventure` today; T85a's audit, §19).
- **Invariants** (`check_owners`, `check_weekly`, `check_added`, run after every command in the
  properties): an owner only on an ownable, a player of the game; a mine's guards within seven
  slots and the stack bound; a weekly amount within its kind's range or 0; a fraction below the
  unit; an added object a monster on a land tile no other object holds.

### 7. The owned objects' visits (ADV-OWN-001, ADV-MIN-002, ADV-OWN-005, ADV-SHY-002)

`visit_at`'s `Effect` gains `Mine`, `AbandonedMine`, `Lighthouse`, `Shipyard`. The team test is
`allied(a, b)` = `a == b` (E10's teams change only it).

- **Flagging** (`AdventureState::flag(index, player)`): the owner written, `ObjectFlagged {
  object_type, anchor, owner }` emitted to the players who see the object and the new owner, the
  disk of radius 3 around the entrance revealed to the new owner (ADV-FOG-002's `reveal`; its
  `TilesRevealed`). No flag for an own or allied object.
- **A mine** (own: the outcome's `garrison: Some(index)`, which the game opens as the Leave Guards
  dialog, §14; ally: nothing; else): with guards (`guards_of(index)` not empty) the outcome's
  offer `OfferKind::Fight` (the guarded pile's question), answered by the game: accepted, the
  battle (§13), whose win the game reports back as `AdventureState::end_mine_battle(index, won,
  survivors)`: won, the guards cleared and the mine flagged; else the survivors written and no
  flag. Without guards: flagged at once.
- **A Lighthouse:** flagged unless own or allied; nothing else. Its effect is read by
  `day_maximum` in a boat: `lighthouse_points(owner)` = Σ over the owner's Lighthouses of their
  kind's `sea_points`, passed to `sea_day_points` (§2).
- **A Shipyard:** flagged unless own or allied, then the boat offer: no tile, `ShipyardBlocked {
  object_type, anchor, reason: NoTile }` to the visitor's owner; a boat or a hero on the tile,
  `reason: Taken`; else the outcome's offer `OfferKind::Boat { price }`, which the game makes only
  when the treasury can pay (`OfferRefused` else). Accepted: `AdventureState::build_boat(index,
  player)` (the offer re-checked by `offer_stands`: the tile still free) puts an empty boat of the
  kind's type on the tile (ADV-SEA-001's record: facing 2, the builder as owner, no captain; the
  empty boats' grid set), `BoatBuilt { tile, boat_type }` to the players who see the tile.
- **A hero on an owned entrance** stays a hero on its tile: the step meets or fights it before
  any object (today's rule); nothing new.

### 8. `BuildBoat` (ADV-SHY-003)

`AdventureCommand::BuildBoat { object_type, anchor }`. Validation in this order, rejected whole:
the object exists at the anchor with that type, a `Shipyard` (`UnknownObject`); the player has
explored one of its tiles (`UnknownObject`, so a hidden Shipyard is not revealed by the error);
its owner is the player or an ally (`NotOwner`); its boat tile exists (`NoBoatTile`); the tile is
free (`BoatTileTaken`). The game checks the treasury against the kind's price first
(`GameCommandError::CannotPay`, the state unchanged), then the adventure applies: the boat as
§7's, the outcome's price paid by the game. No hero, no movement points, no visit.

### 9. The weekly objects' visits (ADV-WEK-001 to -003, ADV-VIS-006 amended)

From the neighbouring tile, any hero, no question. **The contents are the object's, one a week,
for whoever comes first** (decision 201): the week's first visit by any player's hero takes them
and empties the object for every player until the next week's step; no per-player contents, no
per-player test. The mark first (the Windmill's and the Water Wheel's for the visitor's team, the
Garden's for the visitor's owner: hint data only, never read by the gain), then: contents left,
the outcome's gain (as a pickup's, `ResourceGained` by the game) and the amount set to 0; none,
`ObjectEmpty { object_type, anchor }` to the visitor's owner (the original's "nothing" message).
The visit never ends the walk early beyond a neighbour visit's usual stop.

### 10. The week's step (`AdventureState::start_week`; ADV-WEK-001 to -003, ADV-MON-013)

Called by the game on a week's first day (§14) with the game's stream:

1. **The weekly objects**, in weekly index order: the Windmill draws its resource then its amount;
   the Water Wheel takes `later`; the Garden draws its gift and is full. Marks kept.
2. **The growth**, in object index order over the monsters on the map (the static ones not
   removed, then the added): skipped when `no_growth`; `v = count × unit + fraction`, `v += ⌊v ×
   growth_bp ÷ 10 000⌋`, `v = min(v, cap × unit)`; the count `v ÷ unit` written to `army_left` as
   one stack of the creature (the monster's survivors' shape), the fraction to `fractions`;
   `MonsterChanged` emitted where its word or count changed, to the players who see it (as a
   battle's survivors). A count from the map above the cap falls to it (A-32, no flag).

No hero is lifted (the original lifts one standing on an entrance only to read the object under
it). The draws: §14's stream order.

### 11. The month's step (`AdventureState::start_month`; ADV-MON-014)

Called in a creature month with the creature and the game's stream: every tile in level, row and
column order that is land (not water or rock, by the part's passage), that no object blocks or is
visited from (live: removed objects free their tiles, added ones hold theirs), with no event and
no hero, draws `1..=spawn_one_in`; a 1 adds a monster: the part's monster type for a placed
monster (class 54), the creature, a count `spawn_factor × ` a draw from the creature's
adventure-map range (ADV-MON-004), an aggression drawn from `spawn_aggression`, `never_flees`
false, growing, no treasure, its template the creature's (the original's `place_object` 0x4C9890:
one tile, blocked and visited from neighbours and from above). `MonsterAdded { object_type, tile,
creature }` to the players who see the tile. The guard zones read the added objects at once (the
original refreshes them after the walk, `refresh_guard_flags` 0x4CA410: no test inside the walk
reads them, so the two agree). Not built: the original's Plague and normal months add nothing on
the map.

### 12. Information (ADV-OWN-003, ADV-FOG-006 amended)

- `ObjectView` gains `owner: Option<PlayerId>` (every viewer who sees the object);
  `production: Option<(ResourceId, u32)>` for a mine the viewer's team owns (an abandoned mine's
  drawn resource thus hidden from the others); `guards` as a guarded pile's view (creatures and
  their words; the counts for the owner's team); `visited: bool` for a weekly object (the
  viewer's team's mark, the Garden's the viewer's own); a Shipyard's `boat_tile: Option<Pos>`
  (static, public).
- The added monsters are objects among the others in the observation where their tile is
  explored; a fraction is never shown.
- The events of §7 to §11 carry their viewers as in ADR-0016 point 6's draft. `docs/06-security.md`
  ("Information hiding") gains the owners, the production, the marks and the month's kind (T85).

## Part C — the game, the env, batch E9 and the cut

### 13. The mines' battles (GAM-BAT-001 amended; ADV-MIN-002)

- **The defender:** a neutral mine whose guards are one stack, `Defender::Monster { creature,
  count }` (split by the game as a monster's, `ADV-QUIRK-6`'s draws; D-47); a neutral mine with
  more, `Defender::Guards { army }` (unsplit, the neutral seat); an owned mine's,
  **`Defender::Garrison { owner, army }`** (new): the side's owner the mine's, its human flag the
  setup's (GAM-BAT-010), played by that player through `GameCommand::Battle` outside its turn as a
  hero battle's defender (*agent's call*; E9-2 records it, §17).
- **The terrain:** the mine type's `battle_terrain` when set (`battle_at`: crystal, gold and
  abandoned mines on the subterranean), else the entrance tile's; special grounds as before.
- **After:** the survivors go back to the mine (`end_mine_battle`); a win flags it for the
  attacker's owner; a lost hero leaves the map as in any battle. The abandoned mine's win message
  (ADVEVENT 85) and the owned mine's question text are the client's (the events carry the owner).

### 14. The game (`h3-game`; game state hash v21)

- **The setup:** `GameSetup::events: Vec<TimedEventSetup>` (§4), validated (players in range,
  known resources); the adventure setup's owners (§4).
- **The state:** `month: MonthKind { Normal, Creature(CreatureTypeId), Plague }` (`Normal` at the
  build); `events: Arc<[TimedEventSetup]>` (static). The week and the month are derived from the
  day (`week_of(day)`, `month_of(day)`, the part's calendar).
- **The day advance** (`advance_day`, GAM-CAL-002, -003, ADR-0017 point 3's draft):
  1. the day + 1, `DayStarted`;
  2. on a week's first day (`(day − 1) % week_days == 0`): `WeekStarted { week }`, then the
     adventure's `start_week` with the game's stream;
  3. on a month's first day (`(day − 1) % (week_days × month_weeks) == 0`): the kind drawn
     (`1..=10` against the odds; a creature month then draws its creature among the twelve),
     `MonthStarted { month, kind }`, then in a creature month the adventure's `start_month`;
  4. **the income** (GAM-RES-003 to -005): for each player in id order, the adventure's
     `production(player, week_start)` (its mines' daily amounts by resource, through the
     abandoned mines' drawn resource; on a week's first day, for each creature type with a
     `weekly_resource` ability (§1) in one of the player's heroes' armies, its amount once,
     however many: the Crystal Dragons' 3 crystal) plus `h3_heroes::daily_income` of each of its heroes
     on the map, gained (saturating), one `ResourceGained` per resource with a non-zero amount;
  5. **the spell points** (HER-MAN-002): each hero of the store, `h3_heroes::day_mana`, written
     only when it changes (a write copies one hero's details);
  6. the adventure's `start_day` (increment 5's reveal under `strict-fixed`).
  Day 1 runs none of 1 to 5 (the build calls `start_day` only, as today).
- **The turn start** (`start_group`): after each player's `start_turn` and `TurnStarted`, its
  **timed events** (GAM-CAL-004): each in list order whose players include it, whose `human` or
  `computer` flag matches its `PlayerSetup::human`, and whose day it is (`day == first_day`, or
  `interval > 0`, `day > first_day` and `(day − first_day) % interval == 0`): positive amounts
  gained, negative ones lost down to 0 (`ResourceLost` the amount taken), then
  `TimedEventFired { player, event }` (the owner's). Day 1's fire inside the build (no sink):
  *agent's call*, `PlayerView::events_today` (the owner's) names the events that fired at its
  turn start today, derived from the day and the setup, so a client shows day 1's too.
- **Decisions** (ADR-0017 point 6's draft): `Decision::ArmyDialog` gains `holder: Holder { Join,
  Mine(usize) }` (GAM-DEC-004 amended): `Close` loses a join's group as today and writes a mine's
  back (`AdventureState::set_guards`); the hero's last stack rule holds. The mine's question and
  the boat offer are `Decision::Offer`s; an accepted boat offer pays the price, then
  `build_boat`. The battle with an owned mine's guards is a two-owner `Decision::Battle`.
- **`BuildBoat`** (§8): the game's `check_ready`, then `AdventureState::boat_price(player,
  object_type, anchor)` (the adventure's validation, read-only), the treasury test, then the
  command applied and the price paid.
- **The observation:** `GameObservation::month` (public), `PlayerView::events_today` (the
  owner's); the adventure's views (§12). `project` passes `WeekStarted`, `MonthStarted` to every
  viewer, `TimedEventFired` to its player.

### 15. The env, replays, fixtures, properties, benchmarks

- **The env** (`h3-sim`, the AI lane's; `cross-lane` in T85b with the user's consent, as decision
  172): the action encoding v6 gains `BuildBoat` (one entry per Shipyard the viewer may use, by its
  anchor's tile index), masked by `legal_commands`; the army dialog and the offers are typed
  answers already. No flat observation encoding (decision 187), so nothing to bump there.
- **Replays:** the adventure replay format v8 (`BuildBoat`); every golden replay of eight days or
  more re-recorded (the growth on day 8 changes the monsters' counts), the shorter ones rehashed
  (the hashes' versions). New, on **`e08-economy-36.h3m`** (written by `h3_map::writer` in
  `crates/h3-map/tests/economy_map.rs`: two players; the seven mines, guarded by monsters beside
  and not, one owned at load; an abandoned mine; two Lighthouses on a coast; a Shipyard with its
  boat tile; two of each weekly object; growing and not growing monsters; timed events once,
  repeating and taking; heroes with Estates, Mysticism, a resource specialist, the income and mana
  artifacts and a Crystal Dragon): random play over 60 days (two month starts; a creature month by
  a seed search), scripted ones for a flag and Leave Guards, a neutral and an owned mine fought,
  a boat built at a visit and by `BuildBoat`, a weekly object visited twice in a week.
- **Properties** (`h3-map`'s `play.rs`, `view.rs`): the plans place the new classes
  (`ECONOMY`), steps `Step::{Garrison, BuildBoat}`, the end of turn over weeks; checks:
  §6's invariants after every command; the income conserved (the treasuries' change at a new day
  equals an independent sum of §14's parts); a week's step leaves every weekly object full and
  every growing monster at `min(⌊v × 1.1⌋, cap)`; an added monster behaves as a placed one (its
  visit, its guard zone, its view); every legal `BuildBoat` accepted, every other rejected with the
  state unchanged; the views (§12) to the right viewers only.
- **Benchmarks** (`benches/adventure/`): `day-start-xl-144` (on **`e08-economy-144.h3m`**, the
  XL map with as many mines and weekly objects as the largest shipped map and a monster on every
  hundredth free tile: the `EndTurn` that starts day 8), `month-start-xl-144` (the same map, the
  `EndTurn` that starts day 29 in a creature month: the spawn's walk), `random-play-economy-36`,
  `fork-economy-xl-144` (forks after a week's writes). The existing workloads re-baselined where
  the growth or the typed mines change their games, each with its reason.
- **`--with-original-data`** (`crates/h3-map/examples/original_data.rs`): every shipped map's
  static layer with the new classes: the owners at load (60 mines, one Shipyard), the boat tiles
  (none for Hoard's two), the abandoned mines' sets, the 666 timed events.

### 16. Versions, rule ids, quirks

| Version | Change | Task |
|---|---|---|
| core ruleset hash v51, pack format v50 | the four kinds, the two hero parameters, the core creature's abilities; the packs | T81 (`shared/`) |
| adventure ruleset part v14 | the seven kinds; the calendar, growth, months and income parameters | T83 |
| map import v15 | classes 42, 53, 55, 87, 109, 112, 220; owners; the timed events; `ObjectOwner`, `MineResources`, `OwnedOnWater` | T83 |
| adventure state hash v17, game state hash v21 | owners, guards, weekly, fractions, added objects; the month, the events, the dialog's holder | T83, T85 |
| adventure replay format v8 | `BuildBoat` | T84 |
| env action encoding v6 (the AI lane's) | `BuildBoat` | T85b (`cross-lane`) |

Rule ids: survey §16 (ADV-OWN-001 to -005, ADV-MIN-001 to -003, ADV-SHY-001 to -003, ADV-WEK-001
to -003, ADV-MON-013, -014, GAM-CAL-002 to -004, GAM-RES-003 to -005, HER-MAN-002 and the
amendments of GAM-BAT-001, HER-ADV-009, ADV-VIS-006, ADV-MAP-002); new here: GAM-DEC-004 amended
(the kept group), ADV-FOG-006 amended (the owners' views). ADV-OWN-004 (a defeat's) is E10's: not
written now. Quirks, recorded in `docs/rules/quirks-adventure.md` without flags: `ADV-QUIRK-33`
A-31 (an abandoned mine without a resource: refused at the import), `ADV-QUIRK-34` A-33 (an owned
Lighthouse at load reveals nothing in the original: every owned object reveals here); both not
reproduced. A-29 is recorded with E10's defeat; A-30 with E09; A-32 is the rule (no quirk).

### 17. Batch E9

Survey §15's 12 rows in two runs (decision 198), on the harness (`clients/capture/`, common) in
SoD on Easy, hot seat, from maps `h3-experiment` writes: **E9a** the visits (E9-1 to E9-5),
**E9b** the day's start (E9-6 to E9-12, two to three weeks of end turns). Two predictions are
added to the survey's rows: **E9-2** records whether Blue (human) is asked to play its guards'
side when Red attacks them (§13's call); **E9-4** records whether the tiles around Red's two
Lighthouses owned at load are explored on day 1 (A-33: the reading says not).

- **The writer and the specs:** built by T78 (`E08-log.md`, T78).
- **The harness** (T79; common, CLT-Q18: this lane builds it with its check and tells the client
  and map-client lanes' inboxes): reads of the mine records (game +0x4E38C: owner, resource,
  guards), a player's income +0x108 and the Garden mask +0xB8, the date's three words
  (+0x1F63E to +0x1F642), a tile's word (the weekly objects', a monster's bits 27 to 30), the
  explored tiles; a click at a Shipyard with no hero near and the boat dialog's build button.
- **The scoring** (T80): survey §19 (new), each row against its prediction; a dispute a row
  contradicts reverts to the other side (decision 196) or becomes an `ADV-Q`; the consensus rows'
  sources `verified`. D-48 and D-50 stay as read (decision 199), without rows.

### 18. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T81's `shared/` task (the three ADRs applied with batch E9's outcome;
the four kinds, the two parameters and the core creature's abilities in `h3-content` and `h3-pack`
with the packs and the harvester's gap-fill, `cross-lane`; `docs/03-domain.md`'s terms: owner, flag, ownable object,
mine, guards, Leave Guards, weekly object, the week's and the month's steps, the month's kind,
timed event, income); batch E9's spec (`h3-experiment`, `cross-lane`, T78); the harness (common,
T79); the env's action encoding (`h3-sim`, `cross-lane`, T85b). Everything else is the lane's own.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E9 runs before the build and before the shared task** (decision
198, as increments 7 and 8), so the ADRs are applied once with the disputes' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T78** | Batch E9's maps: the writer's seven classes and timed events (§4); the spec's fields (`cross-lane`); the two specs and templates (§17) |
| 3 | **T79** + **T80** | T79: the harness's reads and clicks, the recipes, the runs E9a and E9b. T80: scored (survey §19); D-47, D-49, A-32 settled |
| 4 | **T81** | `shared/`: the three ADRs amended with T80's outcome (and ADR-0015 point 2's core row), the four kinds, the parameters, the core creature's abilities and the packs, the glossary |
| 5 | **T82** + **T83** | T82: `h3-heroes` (§2; small). T83: `h3-map` and `h3-adventure`: the import (§4), the kinds and parameters (§5), the state (§6), the views (§12); map import v15, part v14, adventure state hash v17 |
| 6 | **T84** | `h3-adventure` and `h3-game`: the owned objects' visits (§7), `BuildBoat` (§8), the weekly visits (§9), the mines' battles (§13), the kept dialog and the boat offer (§14); likely cut in two at its start (flagging and mines with their battles; Lighthouses, Shipyards, `BuildBoat` and the weekly visits) |
| 7 | **T85** | The day's start: T85a `h3-adventure`'s week and month steps (§10, §11; the added objects and the audit of the static reads); T85b `h3-game`'s day advance, income, spell points, timed events and observation (§14), the env (`cross-lane`, §15) |
| 8 | **T86** + **T87** | T86: the economy maps, their replays and properties, the benchmarks, `--with-original-data` (§15). T87: the audit against the exit criteria, increment 9 closed, increment 10 refined |

**Increment 9's exit criteria:**
1. Mines, abandoned mines, Lighthouses, Shipyards and the three weekly objects import typed with
   their owners; an owner nobody plays, an abandoned mine without a resource and an ownable
   entrance on water are refused; an abandoned mine's resource and Troglodytes and a Shipyard's
   boat tile are fixed when the game is built (ADV-OWN-002, ADV-MIN-001, -003, ADV-SHY-001,
   ADV-MAP-002).
2. A visit flags a neutral or another's object for its visitor's owner, never an own one,
   revealing radius 3; every owned object reveals at the start (A-33 not reproduced); every
   viewer sees the owners (ADV-OWN-001, -003).
3. An own mine's visit opens Leave Guards, whose group stays as the mine's guards; another's
   guards are asked, then fought (a neutral single stack split, D-47 as batch E9 decided; others
   as an army, an owned mine's played by its owner), on the subterranean for crystal, gold and
   abandoned mines; the survivors stay, a win flags (ADV-MIN-002, GAM-BAT-001, GAM-DEC-004).
4. Lighthouses add 500 each to their owner's heroes' sea points; a Shipyard builds a boat at its
   visit's offer and by `BuildBoat` without a visit, for the price, on its fixed tile, refused
   without a tile or with a boat or a hero on it (ADV-OWN-005, HER-ADV-009, ADV-SHY-002, -003).
5. No new day on day 1; from day 2 the week on days 8, 15, …, the month on days 29, 57, … with
   its kind; the income of mines, the heroes' artifacts, Estates and specialists and the Crystal
   Dragons paid to every player before the day's first turn; every hero's spell points regained
   (GAM-CAL-002, -003, GAM-RES-003 to -005, HER-MAN-002).
6. Each week resets the Windmills, Water Wheels and Gardens, visited or not, and grows every
   wandering monster in sixteenths to the cap (D-49 as decided, A-32); a creature month adds its
   monsters, which behave as placed ones (ADV-WEK-001 to -003, ADV-VIS-006, ADV-MON-013, -014).
7. Timed events fire at their players' turn start on their days, in the map's order, floored at
   0 (GAM-CAL-004).
8. A viewer sees every owner, its own team's mines' production, its team's weekly marks (the
   Garden's its own), the month's kind; never a weekly object's contents, another team's
   production nor a monster's fraction (ADV-OWN-003, ADV-FOG-006).
9. Every new rule has its id, source and tests; `ADV-QUIRK-33` and `-34` recorded as not
   reproduced; the gates pass (coverage, determinism with the re-recorded and new replays, the
   benchmarks at their baselines and the new ones baselined); the env plays `BuildBoat`.
10. Batch E9 run and scored (T80); the consensus rows `verified`.

### 19. Risks and open points

- **Objects added in play are a first.** Every path that reads the static objects by index (43
  sites of `map.objects()` in `h3-adventure` today) must read the live presence: T85a starts
  with that audit, behind one accessor, and the property that an added monster is visited,
  guards, fights and is seen as a placed one. `Reach` and the passage read added objects only
  where their grid's bit is set (`bench-check` on `reach-xl-144`).
- **The day's start's cost.** A month's walk draws once per free land tile (about 20 000 on an
  XL map); a week writes every growing monster and every weekly object (a fork after it copies
  those tables on its next write). `month-start-xl-144` and `day-start-xl-144` measure both;
  the walk's tile test should reuse the static map's flags rather than `tile_object`.
- **Re-recording.** Every golden replay of eight days or more changes (the growth); the maps with
  mines also change their visits (blockers until T84). T86 re-records them with the README loop
  (`docs/lanes/adventure/TRAPS.md`), checking each keeps the features its README row names.
- **The owner's guards side** (§13): if E9-2 shows the original plays an owned mine's guards for
  their owner, the side goes to the neutral seat (`Defender::Guards` with an owner for the
  battle's colours only); T84's cut confirms it after T80.
- **Teams:** every "team" test is `allied`, the player alone until E10; an allied map plays its
  allies as enemies, as today.
- **Not read** (survey §14, §17): the three other callers of the income (the kingdom's views); a
  garrison hero counted twice (E09); a Shipyard side tile's owner byte (no rule depends on it:
  the command names the Shipyard); the boat dialog itself (0x5D2720); the guards' words
  (0x40ABE0: the view uses the monsters' words); how an empty side settles a battle (A-29, E10).
- **Not in increment 9** (decision 199): towns' income and buildings, the Plague's and the
  week's creature's effects, the Artifact Merchants, the tavern (E09); dwellings and the Refugee
  Camp (increment 10); the Fountain of Fortune's, Magic Spring's and Stables' resets (their
  objects' increments); the computer's bonus, the handicap, humans first in a day, a defeat
  (E10); a revisit command (the original has none).
