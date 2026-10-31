# E08 design note

Increment 9's note: mines, the week and owned objects (E08 decision 192). Increment 8's note
(boats and the sea) is in `E08-log.md` ("Increment 8's design note at its closure", moved by T76);
the code's comments cite these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 9's
amendments of ADR-0013 (points 3, 13.6, a new point 16), ADR-0016 (points 1, 3, 4, 5, 6) and
ADR-0017 (points 1, 3, 4, 6, 8, 10, 11), drafted in the lane's `adr-drafts/` and applied by T81a (`shared/e08-t81a`, decision 211; the drafts deleted);
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

Built by T81b (`shared/e08-t81b`; its record in `E08-log.md`): the four kinds `daily_resource {
resource, amount }`, `mana_regain { points }`, `extra_mana_regain { points }`, `full_mana` (none
reaches `BattleEffect`); `[hero_params]`' `base_mana_regain` (1) and `regain_specialist_bonus`
(1); the core creature's abilities outside battle, `CreatureInfo::abilities` (`weekly_resource {
resource, amount }`), written `abilities_outside_battle` in `core.toml` (decision 212: `abilities`
is the battle file's field); the packs: the six resource artifacts 1, the gold artifacts 1 000,
750, 500, the Cornucopia 4 of mercury, sulfur, crystal and gems (its parts' 1 each count through
it: the original's 5), the Charm, Talisman and Mystic Orb 1, 2, 3, the Wizard's Well, Estates 125,
250, 500 gold, Mysticism 2, 3, 4, the thirteen resource and gold specialists (1 or 350), the
Crystal Dragon's 3 crystal (`ab`). Core ruleset hash v51, pack format v50.

### 2. The derivations (`h3-heroes`; HER-MAN-002, GAM-RES-004, HER-ADV-009 amended)

Built by T82 (`adventure/e08-t82-t83a`, decision 213; its record in `E08-log.md`):
`h3_heroes::daily_income(details, core) -> Vec<u32>` (dense by resource in the core's order, each
worn copy and a worn combination's parts counting, a skill's amount grown by its specialty, exact
integers, saturating), `day_mana(details, core) -> u16` (the Well, the highest regain or the base,
a specialist's growth and bonus only with the skill, each type's extra once, never lowered) and
`sea_points(details, core, table, lighthouse_points)` (added last; `h3_adventure::sea_day_points`
passes 0 until §7's visits give it the owner's Lighthouses, T84).

### 3. Rule ids, quirks, shared needs (the hero side)

Built by T82: HER-MAN-002 (`verified`, E9-12), GAM-RES-004 (`implemented`: its payment is the
game's, §14, T85b), HER-ADV-009 amended (`verified`, E9-4). No quirk.

## Part B — the map (`h3-map`, `h3-adventure`)

### 4. The import (`h3-map`; map import v15)

Built by T83a (`adventure/e08-t82-t83a`, decision 213; its record in `E08-log.md`): the classes
typed (53 by subclass, 220 as 53.7, 42, 87, 112, 109, 55; 220 ownable scenery), an ownable
object's owner as `ObjectSetup::owner` (`ObjectOwner` for a colour nobody plays), an abandoned
mine's set as `Holds::Resources` (`MineResources`), `Unsupported::OwnedOnWater`, the timed events
as `GameSetup::events` (`h3_game::TimedEventSetup`; `Events` past a `u16`). The setup checks an
owner (an ownable object, a player of the game: `SetupError::ObjectOwner`), the set
(`check_holds`), the events (`GameSetupError::Event`); the static map hashes the set (adventure
state hash v17, *agent's call*: the bump this note gave §6's state moves to T83b's v18); the
owners and the events are not kept by the state yet (§6, §14).

### 5. The kinds and the part (`h3-adventure`, `h3-pack-adventure`; adventure ruleset part v14)

Built by T83a: the seven kinds as the table had them, with two *agent's calls*: the Water Wheel
names its `resource`, the Garden takes a list of gifts (`gifts = [{ resource, amount }]`), both
data where the table fixed gold and two; every kind entered and not from above (ADV-Q39, decision
214, the weekly ones too), the four ownable ones refused otherwise; hash tags 23 to 29. One mine a
resource, and with an abandoned mine one for every resource (`AdventureRuleset::mine_daily`); an
abandoned mine's count at most `max_stack_count`. `[params.calendar]` (required), the monsters'
growth (`growth_bp` at most 10 000, `growth_unit` 1 to 256, `growth_cap` at most
`MAX_MONSTER_COUNT`), `[params.months]` (optional; its `monster`, an object type of kind `monster`
without levels, *agent's call*: the type the month's monsters are, §11; its creatures no war
machine, with an adventure count, the count times `spawn_factor` at most `MAX_MONSTER_COUNT`; it
needs `[params.monsters]`). The base pack's thirteen types and parameters with their sources; the
synthetic pack's calendar.

### 6. The state (`h3-adventure`; adventure state hash v19, built)

T83b1 built the static map's ownable and weekly lists, the draws, the owners, a mine's guards,
the weekly contents, `check_owners`, `check_weekly` and the start's reveal (adventure state hash
v18; decisions 215, 217); T83b2a the monsters' fractions, the objects added in play in the live
presence and `check_added` (adventure state hash v19; decision 218). Their records and this
section's built text are in `E08-log.md` ("T83b1", "T83b2a"). Left: T85a's audit of the static
reads that never meet an added monster's index before its month (§19; the list in T83b2a's
record).

### 7. The owned objects' visits (ADV-OWN-001, ADV-MIN-002, ADV-OWN-005, ADV-SHY-002)

`visit_at`'s `Effect` gains `Mine`, `AbandonedMine`, `Lighthouse`, `Shipyard`. The team test is
`allied(a, b)` = `a == b` (E10's teams change only it).

- **Flagging** (`AdventureState::flag(index, player)`): the owner written, `ObjectFlagged {
  object_type, anchor, owner }` emitted to the players who see the object and the new owner, the
  disk of the part's flag radius (3, decision 217) around the entrance revealed to the new owner (ADV-FOG-002's `reveal`; its
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

Entered (ADV-Q39, decision 214: the original's class lists and batch E9's E9-8, against this
note's first wording), any hero, no question. **The contents are the object's, one a week,
for whoever comes first** (decision 201): the week's first visit by any player's hero takes them
and empties the object for every player until the next week's step; no per-player contents, no
per-player test. The mark first (the Windmill's and the Water Wheel's for the visitor's team, the
Garden's for the visitor's owner: hint data only, never read by the gain), then: contents left,
the outcome's gain (as a pickup's, `ResourceGained` by the game) and the amount set to 0; none,
`ObjectEmpty { object_type, anchor }` to the visitor's owner (the original's "nothing" message).
The visit ends the walk on the entrance, as any entered visit.

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

### 12. Information (ADV-OWN-003, ADV-FOG-006 amended; built)

T83b2b built the views (decision 219: a mine's guards as the original's hint, not "creatures and
their words"); its record and this section's text are in `E08-log.md` ("T83b2b"). Left: the
events of §7 to §11 carry their viewers as ADR-0016 point 6 says (T84, T85), and
`docs/06-security.md` gains their lines and the month's kind with them.

## Part C — the game, the env, batch E9 and the cut

### 13. The mines' battles (GAM-BAT-001 amended; ADV-MIN-002)

- **The defender:** a neutral mine whose guards are one stack, `Defender::Monster { creature,
  count }` (split by the game as a monster's, `ADV-QUIRK-6`'s draws; D-47, confirmed by E9-1); a neutral mine with
  more, `Defender::Guards { army }` (unsplit, the neutral seat); an owned mine's,
  **`Defender::Garrison { owner, army }`** (new): the side's owner the mine's, its human flag the
  setup's (GAM-BAT-010), played by that player through `GameCommand::Battle` outside its turn as a
  hero battle's defender (*agent's call*, confirmed by E9-2: the original hands its human owner
  the side, each stack waiting for its order; survey §19).
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
     `income(player, week_start)` (its mines' daily amounts by resource, each mine's
     `production` as T83b2b reads it, an abandoned mine's through its drawn resource; on a week's first day, for each creature type with a
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
| core ruleset hash v51, pack format v50 | the four kinds, the two hero parameters, the core creature's abilities; the packs | T81b (`shared/`), done |
| adventure ruleset part v14 | the seven kinds; the calendar, growth and months parameters | T83a, done |
| map import v15 | classes 42, 53, 55, 87, 109, 112, 220; owners; the timed events; `ObjectOwner`, `MineResources`, `OwnedOnWater` | T83a, done |
| adventure state hash v17 | the static map's abandoned mines' sets (`Holds::Resources`) | T83a, done |
| adventure ruleset part v15, adventure state hash v18 | the flag radius (decision 217); owners, guards, weekly, the draws of abandoned mines and Shipyards | T83b1, done |
| adventure state hash v19 | the objects added in play, the monsters' fractions (decision 218) | T83b2a, done |
| game state hash v21 | the month, the events, the dialog's holder | T85 |
| adventure replay format v8 | `BuildBoat` | T84 |
| env action encoding v6 (the AI lane's) | `BuildBoat` | T85b (`cross-lane`) |

Rule ids: survey §16 (ADV-OWN-001 to -005, ADV-MIN-001 to -003, ADV-SHY-001 to -003, ADV-WEK-001
to -003, ADV-MON-013, -014, GAM-CAL-002 to -004, GAM-RES-003 to -005, HER-MAN-002 and the
amendments of GAM-BAT-001, HER-ADV-009, ADV-VIS-006, ADV-MAP-002); new here: GAM-DEC-004 amended
(the kept group), ADV-FOG-006 amended (the owners' views). ADV-OWN-004 (a defeat's) is E10's: not
written now. Quirks, recorded in `docs/rules/quirks-adventure.md` without flags: `ADV-QUIRK-33`
A-31 (an abandoned mine without a resource: refused at the import), not reproduced. A-33 is
withdrawn (T80, survey §19: the original's game start reveals around every owned object, a
Lighthouse too), so `ADV-QUIRK-34` is not recorded. A-29 is recorded with E10's defeat; A-30 with
E09; A-32 is the rule (no quirk; confirmed by E9-10).

### 17. Batch E9

Survey §15's 12 rows in two runs (decision 198), on the harness (`clients/capture/`, common) in
SoD on Easy, hot seat, from maps `h3-experiment` writes: **E9a** the visits (E9-1 to E9-5),
**E9b** the day's start (E9-6 to E9-12, two to three weeks of end turns). Two predictions are
added to the survey's rows: **E9-2** records whether Blue (human) is asked to play its guards'
side when Red attacks them (§13's call); **E9-4** records whether the tiles around Red's two
Lighthouses owned at load are explored on day 1 (A-33: the reading says not; they were, T80).

- **The writer and the specs:** built by T78 (`E08-log.md`, T78).
- **The harness** (T79; common, CLT-Q18: this lane builds it with its check and tells the client
  and map-client lanes' inboxes): reads of the mine records (game +0x4E38C: owner, resource,
  guards), a player's income +0x108 and the Garden mask +0xB8, the date's three words
  (+0x1F63E to +0x1F642), a tile's word (the weekly objects', a monster's bits 27 to 30), the
  explored tiles; a click at a Shipyard with no hero near and the boat dialog's build button.
- **The scoring** (T80, done): survey §19, each row against its prediction. Every row as the
  reading predicted but E9-4's surroundings: D-47, D-49 and A-32 confirmed, §13's call confirmed,
  A-33 withdrawn (the original reveals around every owned object at the start); no dispute
  reverts, no `ADV-Q`. The rows each new rule cites are listed there (`verified` when built).
  D-48 and D-50 stay as read (decision 199), without rows.

### 18. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T81's `shared/` task in two (decision 211): T81a the three ADRs
applied with batch E9's outcome and `docs/03-domain.md`'s terms (owner, flag, ownable object, mine,
guards, Leave Guards, weekly object, the day's start with the week's and the month's steps, the
month's kind, timed event, income), done; T81b the four kinds, the two parameters and the core
creature's abilities in `h3-content` and `h3-pack` with the packs and the harvester's gap-fill,
`cross-lane`, done (decision 212: whole); batch E9's spec (`h3-experiment`, `cross-lane`, T78); the harness (common,
T79); the env's action encoding (`h3-sim`, `cross-lane`, T85b). Everything else is the lane's own.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E9 runs before the build and before the shared task** (decision
198, as increments 7 and 8), so the ADRs are applied once with the disputes' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T78** | Batch E9's maps: the writer's seven classes and timed events (§4); the spec's fields (`cross-lane`); the two specs and templates (§17) |
| 3 | **T79** + **T80** | T79: the harness's reads and clicks, the recipes, the runs E9a and E9b. T80: scored (survey §19); D-47, D-49, A-32 settled |
| 4 | **T81** | `shared/` in two (decision 211): T81a the three ADRs amended with T80's outcome (and ADR-0015's core row), the glossary (done); T81b the four kinds, the parameters, the core creature's abilities and the packs (done, decision 212) |
| 5 | **T82** + **T83** | T82: `h3-heroes` (§2; small; done). T83 in two (decision 213): T83a the kinds and parameters (§5), the import (§4); map import v15, part v14, adventure state hash v17 (done); T83b in two (decision 215): T83b1 the state the visits read (§6; adventure state hash v18, part v15; done), T83b2 in two (decision 218): T83b2a the fractions and the added objects (§6; adventure state hash v19; done), T83b2b the views (§12; done) |
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
   revealing radius 3; every owned object reveals at the start (as the original, A-33 withdrawn); every
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
9. Every new rule has its id, source and tests; `ADV-QUIRK-33` recorded as not reproduced; the gates pass (coverage, determinism with the re-recorded and new replays, the
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
- ~~**The owner's guards side** (§13)~~: closed by T80, E9-2 showed the original hands the side
  to its human owner: `Defender::Garrison { owner, army }` stands, no neutral seat.
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
