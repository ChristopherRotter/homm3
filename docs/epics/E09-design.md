# E09 design note

Increment 1's note (closed 2026-11-06 by T10; Parts A to C, §1 to §22, kept here as the code cites
them: the epic's decision 40). Increment 2's note (construction, decision 37) is Part D, §23 to
§36, written by T13 (2026-11-07): the open part of this file.

Increment 1's note: towns on the map (the epic's decision 5). A handover, not binding
(`docs/adr/README.md`): the working detail behind ADR-0028 and E09's amendments of ADR-0015,
-0016, -0017 and -0023 (drafted in T2a, accepted at TWN-Q4, moved into `docs/adr/` by T4a), written by T2b from the survey
`docs/rules/surveys/E09-towns.md` (§ numbers are the survey's unless they name this note) and the
answers at `TWN-Q2` and `TWN-Q3` (the epic's decisions 8 to 20). The task that builds a section
deletes it; the file goes when the epic closes. The code cites it as "E09 increment 1's design note
§M".

Three parts: **A**, the town context (`h3-towns`, `h3-pack-towns`, the content); **B**, the map
(`h3-map`, `h3-adventure`); **C**, the game, the env, batch TWN-1 and the cut. Accepted at the
checkpoint (`TWN-Q4`, the epic's decisions 21 to 24): the drafts as written, the cut of §21, the
content's route of §3. *Agent's call (change freely)* marks a choice of this note the user kept
(decision 24) and the task that builds it may still change.

## Part A — the town context (`h3-towns`, `h3-pack-towns`, the content)

### 1. The crates (built by T4b and T5a1, `E09-log.md`)

- **`h3-towns`**'s modules to come: `record` (a town, §4), `store` (`Towns`, §4; a skeleton since
  T4b), `setup` (the build's town setup, §5), `derive` (§6); `building` and `ruleset` are T5a1's.

### 2. The town part (built by T5a1 but `GameRuleset`, T5a2)

The part, its compiler, its hash's first encoding and the adapter are built (`E09-log.md`, T5a1):
the format and each role's set (standing or ever built) are documented in `h3-pack-towns`'
`lib.rs`. Changed there from this note's first shape (*agent's calls*): `gold` names the paying
resource; one to seven creature levels; the siege's side facts are the roles `morale` (standing),
`luck` and `escape_tunnel` (ever built), not one `siege` table, which §6 reads accordingly.

**`GameRuleset`** (`h3-game`, `cross-lane`): `new(core, adventure, battle, towns)`, the mismatch
check extended: the town part on the same core, and every faction with a town record has the
battle part's `[[siege]]` record. Callers outside the town lane's paths (h3-sim's encounter and
tests, h3-cli's adventure loader and census, h3-map's tests and examples) pass the compiled part
or `TownRuleset::empty(core)` (an encounter has no towns) in the same pull request
(*agent's call*). The adventure replay's header records the part's hash (replay format v9).

### 3. The content and its harvest (T5b; read by T5b1, harvested by T5b2)

- **Read from `Heroes3.exe`** (a code reading, every address into `docs/exe/map.toml`):
  `building_tables_init` 0x4EB730 (each faction's buildings, its lists from 0x63EC80, and the
  supersede table 0x6A8BB8), `map_building_ids` 0x6888C0 (read by E24: a slot's building id per
  faction), `town_creatures` 0x6747B4 (levels and upgrades), the silo's table 0x688EB4 (§11.1),
  the neutral garrison's table 0x688E84, 0x688E94, 0x688EA4 (§5.2), the 30 at 0x5C0C5B (§5.3); the
  roles from the readings of §9.4 (the siege's buildings) and §8.3 (the reveal).
- **Read (T5b1, survey §16):** the lists' layout, the supersedes, the hordes, the slots, the
  roles by faction (§16.6), the names (§16.7), VCMI compared (§16.8). **Building 23** (decision
  25) is the Village Hall's town-screen companion in Rampart, Necropolis and Conflux (27 to 29 the
  higher halls'), in no faction's list: **not in the part** (decision 27). The harvest writes the
  original's lists as they are (each record's buildings the faction's list and the Village Hall;
  `supersedes` the common and the faction's supersede lists; the upgraded hordes as below, §4).
- **Names** from `BLDGNEUT.TXT`, `BLDGSPEC.TXT` and `DWELLING.TXT` (the names only: no
  descriptions, Q13); keys from the names, as the other packs'.
- **Compared** with VCMI 1.7.5's `config/factions/*.json` (buildings, `upgrades`) and the wiki; a
  difference with gameplay impact is a `TD-` row (survey) before it is written.
- **Where:** the harvester's gap-fill (`h3-harvest`, the battle lane's crate, `cross-lane`, as E08
  T81b) writes `packs/base/towns.toml` (the eight factions, the base pack's moat as RoE and AB play
  it: harmless, the Stronghold `absent` with `siege_obstacles = true`, the Tower `harmless`),
  `packs/ab/towns.toml` (Conflux) and `packs/sod/towns.toml` (overrides: `damaging`, the Stronghold
  `damaging` without obstacles, the Tower `damaging`; decision 20). The synthetic packs get a
  small hand-written record for their test factions. The route is the user's (decision 23).
- **Not harvested now:** requirements and costs (increment 2), the hordes' growth (core), the
  guild's levels and spells, the services (increments 3, 4), the tavern weights (increment 3).

### 4. The town's record and the store (`h3-towns`; town state hash v1)

Built by T6a (`E09-log.md`) but `set_built` and `capitol` (T6b, their first callers). Changed
from this note's first shape (*agent's calls*): `TownStatic` keeps no alignment (the build's
draw reads it from the setup); a town event has no message field and its buildings are the
faction's set; the store is `Option<Arc<…>>` (one reference count a fork, none without towns).

- **`Town`** (ADR-0028 point 2), *agent's call* in its shape: `faction: FactionId`; `built:
  BuildingSet` (a `u64` over the faction's buildings in the part's order: **every building ever
  built**, the superseded ones too, the original's "every-built" set +0x158); `forbidden:
  BuildingSet`. Derived, never stored: the **standing** set (`built` less every building a built
  one supersedes: the original's built set +0x150). The Capitol's limit is the only write of
  increment 1. **The hordes' upgrade** (survey §16.2, decision 29): a horde ever built with its
  level's upgraded dwelling makes the upgraded horde ever built too (the original's `town_build`
  turns the one into the other), so it alone stands; the part says which building that is (its
  `auto_with`, T5b2a: built once each listed building is ever built). The original's custom-set path that leaves a horde's dwelling standing beside it (T-6)
  is not reproduced: fixed.
- **`TownStatic`** per town, shared by every fork (one `Arc` for the store): its name if the map
  gives one (the map's text; an unnamed town's name is the client's), its map events kept for
  increment 2 (the `TownEvent` buildings and creatures, the timed part as `TimedEventSetup`'s
  fields), its alignment (for a later read; none in increment 1).
- **`Towns`**: `Arc<Vec<Arc<Town>>>` by `TownId` plus the static `Arc<[TownStatic]>`, as the store
  of heroes (ADR-0017 point 1): a fork costs a reference count, a write copies the list of
  references and the one town. Its canonical hash (`h3-kernel`'s `StateHasher`): the version, the
  count, then each town's faction handle, `built`, `forbidden`, then the static part in `TownId`
  order; the game state hash folds it in after the adventure's.
- `Towns::get(TownId) -> &Town`, `Towns::capitol(TownId, &TownRuleset) -> bool` (a standing hall-4
  building), `Towns::set_built(TownId, BuildingSet)` (crate-internal writes only through the rules
  below).

### 5. The town's setup at the build (TWN-SET-003, TWN-SET-004)

Built by T6c (`E09-log.md`; `h3_towns::town_setup`, `town_buildings_setup`, `town_garrison`).
*Agent's calls* confirmed at its start: the garrison's draws before the buildings' in a town; a
random stack at a level the record lacks an empty slot, an upgrade the level lacks the base; the
start's Tavern the `tavern` building that supersedes no other; `TWN-STUB-2` registered there.

`h3_towns::TownSetup` (in `GameSetup`, §13): `faction: TownFaction` (`Fixed(FactionId)` or
`Random { alignment: Option<u8> }`, the colour the SoD byte names), `buildings: BuildingsSetup`
(`Standard { fort }` or `Custom { built: SlotSet, forbidden: SlotSet }`, the map's 41 slots),
`garrison: Option<Vec<GarrisonStack>>` (the map's custom garrison: each stack a creature or
`Random { level, upgraded }`, with its count and slot; none for the map's default), `name`,
`events`. The game resolves the faction first (§14); then, by functions of `h3-towns` the game
calls with the setup stream:

- **The buildings** (`town_buildings_setup`, §5.3; TWN-SET-003): the faction's Village Hall. A
  custom set: the forbidden slots' buildings (a slot without one of the faction's: nothing), each
  `forbidden_with` building of a forbidden one too; the built slots' buildings with what they
  supersede, less the forbidden. A standard set: a Fort with the map's flag; an owned town a Tavern
  and, only with a Fort, the level-1 dwelling and, with one draw `1..=100 <= chance`, the level-2
  one; a neutral town both dwellings, Fort or not (TD-3 by the reading, confirmed by batch
  TWN-1's row 1, survey §15). Building 23 in a Rampart's, Necropolis' and Conflux's towns (row 1)
  is the Village Hall's town-screen companion, left out of the part (decision 27); the hordes'
  upgrade as §4.
- **The garrison** (`town_setup`, §5.2; TWN-SET-004): a custom one as the map gives it, each random
  stack the faction's creature of its level, upgraded when the code says so (no draw); a custom
  empty one stays empty (TD-5, confirmed by row 2); none: an owned town's empty, a neutral town's
  drawn: for levels 1 to 4 a draw `1..=100 <= chance` and on success a count drawn in the row's
  bounds, the base creature of the level, each new stack in the first free slot. Returned to the
  game, which writes it into the adventure (§8).
- **Not now:** the Grail forbidden on a Grail-victory map in another town (0x5F2160, increment 2)
  and the Shipyard forbidden without a boat spot (0x5C0C90, increment 4): `TWN-STUB-2` (a built
  Shipyard without water is kept, counted by the turrets; 9 towns on the shipped maps have one
  built, the census of T7a says how many lack water).

### 6. The derivations (`h3-towns`, pure functions of a town and the part)

Built by T6b (`E09-log.md`). Each takes `(&Town, &TownRuleset)` (*agent's call*): a faction
without a record reads as a town without buildings (the setup never makes one). Changed from
this note's first shape: `siege`'s morale and luck are its `bonuses` (CLT-Q40), at most 8 buildings
a record (the compiler's refusal); `daily_income` a list of resources; `capitol_limit` gives the
ever-built set and the store writes it (`Towns::apply_capitol_limit`, `set_built` crate-internal).

| Function | Gives | Rule |
|---|---|---|
| `hall_level(town)` | the highest standing `hall`, 1 to 4 | TWN-INC-001 |
| `daily_income(town)` | gold `hall_gold[level − 1]` + `grail_gold` with a `grail` building ever built; the faction's `silo` resources with a standing `silo` | TWN-INC-001, -002 |
| `fort_level(town)` | the highest standing `fort` (§9.4: the last of 7, 8, 9 found) | TWN-SIE-003 |
| `siege(town) -> Siege` | `faction`, `fort`, `moat` (the record's from fort level 2, else `Absent`), `structures` (the standing buildings not forbidden, less the hall-1 building: no faction's list holds the Village Hall, survey §16.4, decision 28; saturating at 255), `escape_tunnel` (ever built), `bonuses` (a `TownBonus` per building ever built with a change: its `morale` while standing, its `luck` once ever built) | TWN-SIE-003 (ADR-0023 point 2's amendment) |
| `siege_obstacles(town)` | the record's flag, from fort level 2 (a Citadel; the E04c survey's S-29) | TWN-SIE-005 |
| `defender_bonus(town)` | the sums of the built buildings' `defender` values | TWN-SIE-004 |
| `reveal(town)` | `reveal_radius`, or the largest built `reveal` (`all` wins) | TWN-CAP-001 |
| `has_tavern(town)` | a standing `tavern` building | TWN-VEW-002 |
| `capitol_limit(town, holds_another)` | with a standing hall 4 and `holds_another`: hall 4 out of `built`, `capitol_demoted_to` in (§8.4; a neutral owner never) | TWN-CAP-002 |

The roles read "standing" or "ever built" as §9.4 reads the original's two sets: the Tavern, the
fort, the halls and the turrets the standing set; the Grail, the special buildings (the Escape
Tunnel, the Fountain of Fortune, the hero bonuses) the ever-built set. A building's role names
which (*agent's call:* each role has one fixed reading, listed in `h3-pack-towns`' format doc).

## Part B — the map (`h3-map`, `h3-adventure`)

### 7. The import (`h3-map` `src/import/town*`; map import v17, T7a)

ADR-0015's amendment point 3, in the import's order:

- **Players** (`players`): each playing colour's `PlayerSetup::start` (§13): `factions` the map's
  `allowed_towns` bits that the ruleset has as factions with a town record (an RoE-format map's
  player whose eight bits are all set also Conflux when the ruleset has it, decision 14), `choice`
  `Fixed` when exactly one is allowed, else `Random` (*agent's call*); `pick` none; `main_town`
  the town whose entrance is the header's position (finding 3), with `generates_hero` (RoE always),
  else `ImportError::MainTown(colour)` (T-4 typed, `TWN-QUIRK-4`); `random_hero` the flag.
- **Towns** (classes 77 and 98, in object order; their index is the `TownId`): the adventure's
  object (§8: the `town` type, the owner as an ownable's, `Holds::Town { formation }`) and
  `GameSetup::towns[id]` (§5): class 98's faction by subclass (`TOWNTYPE.TXT`'s order, the table `FACTIONS` in `import/town.rs`), class
  77 `Random` with the SoD alignment (`0xFF` none; 0 to 7 a colour; 8 to 254
  `ImportError::Alignment(index)`, T-1, `TWN-QUIRK-1`); the buildings' slots as stored; the
  garrison's stacks (a random stack's code, the 14 below "none", a `Random { level, upgraded }`
  as §5.2 reads it; a count of 0 an empty slot); the name, the events. The required and banned spells stay
  unread (increment 4).
- **Heroes:** class 70 a `GameHeroSetup` with `hero_type: None` (the rest of its body a placed
  hero's, its custom experience included: 18 on the shipped maps); a placed hero (not a prisoner)
  whose tile is right of a town's entrance moves onto it (TD-6, `placed_heroes_setup`; TWN-SET-010);
  on any other blocked tile of a town `Unsupported::HeroInTown` stays. `HeroAtMainTown` and
  `RandomHero` leave `Unsupported`.
- **The pool's inputs:** `GameSetup::allowed_heroes` (the map's set: an RoE-format map's 128 bits
  and the 16 Conflux heroes when the ruleset has them, never the campaign heroes, survey §4.2),
  `hero_masks` (SoD's disposed heroes' colour masks), `prisoners` (the prisoners' types, class 62,
  still blockers: their types leave the pool, T-3).
- **The census** with the original data (T7a, `cross-lane` doc `docs/formats/h3m.md`, "Import"):
  every shipped map imports (exit criterion 1), or the typed refusal that remains.
- **As built (T7a1):** the order is the static layer, the main towns, the towns, the heroes, the
  banned sets, the events, the pool's inputs (a main town needs the towns); a town's entrance is
  blocked and visitable, so the hero-in-a-town test leaves the entrance out (a moved hero stands
  there); a hero type the ruleset lacks leaves the pool's inputs (not refused); until T7b the
  build refuses a setup that needs a draw (`GameSetupError::HeroDraw`, `TWN-STUB-3`); map import
  v17 (E08 T93b took v16 first). The census: 156 of the 160 shipped maps import, 4 are refused
  for a dwelling entered from water (`Unsupported::OwnedOnWater`, since T93b); `h3m.md`.

### 8. The kind `Town` and the static map (`h3-adventure`; adventure part v16, state hash v20, T8a)

- **The kind** `ObjectKind::Town` (no parameter; ADR-0016's amendment point 4), ownable; the base
  pack's object type `town` (classes 77 and 98, *agent's call:* one type, the faction being
  `h3-towns`'), its visit `Enter`, `from_above = false`. The type's kind is in the part's hash
  (v16).
- **The static map:** a `towns: Vec<usize>` list of the town objects' indices in object order (as
  `ownables`; `TownId` is the place in it: `town_id(index)`, `town_object(id)`), so no object
  stores its id (*agent's call*); `Holds::Town { formation }` the garrison's formation (*agent's
  call*: the map's byte, read when the garrison defends alone).
- **The garrison** is `army_left` of the town's object (as a mine's guards): the build writes the
  resolved garrison (§5) there through `AdventureState::new`'s setup (`ObjectSetup` gains nothing:
  the game passes the armies, *agent's call*), an empty one written empty. `town_garrison(index)`
  reads it.
- **Owners:** towns join the ownables, their owners in the owners block (ADV-OWN-002 amended); the
  start's reveal (`reveal_owned`) takes each town's radius from the game (a slice by `TownId`),
  every other ownable keeps the flag radius.
- The state hash (v20): towns in the ownables and the owners block, the static map's town list and
  holds.
- **From T7a2** (the user's cut, 2026-11-04): `GameSetup::validate`'s `TownCount` (the setup's
  towns against the static map's town list) and `MainTown`'s owner half (a main town of another
  owner), once the town list and the towns' owners exist (§13's "As built").
- **The cut (the user, 2026-11-04):** T8a in two, the order T8a1, T7b2, T8a2. **T8a1** this
  section with the adventure state hash v21 (E08 T94b took v20) and map import v18 (the towns
  typed); until T7b2 the game passes empty garrisons and the flag radius (`TWN-STUB-5`), until
  T8a2 the entrance is not entered (`TWN-STUB-4`). **T8a2** §9 and §11.
- **As built (T8a1):** `ObjectKind::Town` (the part's hash tag 33), the base type `town` (`town =
  {}`); `StaticMap::towns`, `town_id` and `town_object` (both options), `Holds::Town { formation
  }` (the state hash's tag 9; the map's byte, 0 loose, else tight); `AdventureSetup::garrisons`
  (one per `TownId`, `SetupError::Garrisons`, `Garrison`; `Towns` past a `TownId`),
  `AdventureState::town_garrison`; `start_game`'s `town_radii` by `TownId` (the flag radius past
  the slice). The import types 77 and 98 as `base:town` with the owner (`object_owner`) and the
  formation. *Agent's call:* the entrance carries no typed visit until T8a2, so every step and
  search meets a blocker's entrance as before. `validate`: `TownCount` first in `check_starts`
  (the objects of kind `Town`), `MainTown` widened to a town the player owns at the start (no new
  variant). The census unchanged (31 build).

### 9. The step onto a town (TWN-VIS-001, TWN-VIS-002; `h3-adventure` `src/town*`, T8a)

- **The visitor** is the hero standing on the entrance (`town_visitor(id)`: the hero on the
  entrance tile; never stored, ADR-0016's amendment point 2).
- **The step's three cases** (§7.1, decision 16), decided by the town's owner's team, its garrison
  and visitor, for the moving hero's owner:
  1. own or allied team: **entered** (the walk ends on the entrance; no service, no decision, no
     event beyond the step's: *agent's call*); a hero of the same team on it is met (GAM-MET-001);
  2. another team or none, **defended** (a creature in the garrison, or a visitor): **visited from
     the tile before** (ADV-OBJ-003's neighbour mode: the step paid, ADV-MOV-006's drop, the hero
     stays), the outcome a siege (§10); a step onto another team's hero on a town's entrance is
     this siege, never a field battle (`visit_hero`, §7.1); the ally's visitor (an ally of the
     owner): `AdventureError::AllyInTown`, `TWN-STUB-1` (unreachable until E10's teams);
  3. another team or none, **undefended**: **entered and captured** (§11).
- **The reach and the legal moves** treat a defended entrance of another team as a monster's
  tile (a destination reached from a neighbour, not passed) and the rest as an entered object; the
  test is per entrance tile the search meets (a town is rare), measured against `master`
  (`homm3-bench-regression`).
- **As built (T8a2;** its defended town's refusal superseded by §10's T8b2a: `town_defended`
  became `town_visit`**):** `h3_adventure`'s `src/town.rs`. A town's entrance is a typed visit
  (`StaticMap::place`; `Effect::Town`; `StaticMap::town_at(pos)`), so ADV-OBJ-004 reads the town:
  a hero on the entrance does not leave it upwards and the guard test skips the row above.
  `town_visitor(id)` the hero on the entrance. `town_defended(index, player)`: the owner not
  allied (`allied`, the player alone until E10) and a creature in the garrison, or, whoever owns
  it, a visitor not allied; `visit_at` gives no visit for a defended town, so the step refuses it
  as `AdventureCommandError::Visit` and the reach leaves it out. `besieges(player, other)`: a step
  onto a hero not allied standing on a town's entrance is refused the same way in `check_step`
  and `passage` (a flight passes over it, as over any hero, never stops on it), never a field
  battle. Both refusals are `TWN-STUB-7` until T8b's siege; case 2's visit from the tile before
  and the `AllyInTown` refusal are not built (`TWN-STUB-1` stays E10's). `visit_town`: a town of
  the hero's team gives nothing; another's or a neutral one, entered undefended, gives
  `AdventureOutcome::capture: Some(TownId)`, the owner unchanged until the game composes the
  capture (§11). The walking search's loop is master's: a first cut that tested the visitor in its
  occupied branch cost +7.7 % on `reach-xl-144`, so `Reach::search` runs a `#[cold]`
  `unmark_visitors` pass before the loop on a map with towns (a visitor not allied leaves its tile
  unmarked, and `visit_at`, now `#[inline(never)]` as the compiler had it, meets it through
  `town_defended`). Bench against `master` (the same toolchain): the seven reach and random-play
  workloads within −0.8 % to +0.38 % (after the review's fixes). `besieges` (and so
  `unmark_visitors`) reads the tile's object as `visit_at` does (`AdventureState::tile_index`, its
  own copy of `tile_object`'s lookup: sharing it cost +2 to +2.8 %) and asks that tile to be the
  town's entrance (the review: with a second typed object visited from the entrance tile, the reach
  and the step disagreed).

### 10. The siege's start and the merge (TWN-SIE-001, TWN-SIE-002; T8b)

- **The defender** (§9.1): the visitor, with **the garrison merged into its army first** (§9.2,
  TD-7 by the reading, confirmed by batch TWN-1's row 3, survey §15): rounds of the visitor's
  weakest stack (least AI value × count, of equal ones the last slot), each garrison stack's gain
  (its value with a stack of its creature or an empty slot, else its value less the weakest's),
  the greatest positive gain (the first slot of equal ones) joining its creature's stack, else the
  first empty slot, else changing places with the weakest; until no gain. What stays in the
  garrison does not fight.
  `Defender::Hero` with `BattleStart::town: Some(TownId)`.
- **Without a visitor:** the garrison as an army without a hero: `Defender::Garrison { owner,
  army }` for an owned town (its owner plays it outside its turn, as a mine's), `Defender::Guards
  { army }` for a neutral one (the neutral seat; *agent's call*, no new variant).
- **Events:** the merge's changes to the town's owner as the army dialog's (`StackChanged` per
  slot, the hero's and the garrison's; *agent's call*), then the game's `BattleStarted`.
- **As built (T8b2a; T8b2 cut in two at its start, the user, 2026-11-05: the town its garrison
  alone defends, then T8b2b the visitor with the merge):** `AdventureState::town_visit(index,
  player) -> Option<Visit>` (was `town_defended`): `Enter` for a town of the player's team or an
  undefended one, `Neighbour` when its owner is of another team or none and a creature stands in
  its garrison, none while a hero of another team stands on its entrance (`TWN-STUB-7`, narrowed
  to that visitor). `visit_at` takes a town's visit mode from it, so the step (ADV-OBJ-003's
  neighbour mode: the step paid, ADV-MOV-006's drop, the hero stays), the passage (a flight passes
  over the entrance, as over a monster's tile) and the reach (a destination, as a monster's; its
  walking loop untouched) read one answer. `visit_town(part, (player, hero), (index, mode))`: a
  neighbour visit gives `siege_start`: the garrison by slot as it stands, `Defender::Garrison {
  owner, army }` for an owned town, `Defender::Guards { army }` for a neutral one (the system
  actor plays it), `battle_at` the entrance with `town: Some(TownId)`; nothing else changes until
  the battle ends (T8b1's end). `StaticMap::town_formation(TownId)`: the game's `siege_setup`
  puts the town's formation (§8's byte) on the defending side when no hero defends. The merge,
  `Defender::Hero` and `besieges`' siege are T8b2b's. No version moved.
- **As built (T8b2b; kept whole, the user, 2026-11-05):** `town_visit` gives `Neighbour` also
  when a hero of another team stands on the entrance, whoever owns the town (an own town's only
  by a map's placement, TWN-SET-010), so `TWN-STUB-7` is lifted: `check_step`'s and the passage's
  hero branch, meeting such a visitor (`besieges`), visit the town instead (`visit_at`: from the
  tile before, a flight's from a stop node only, passing otherwise; a boat's step stays
  `sail_entry`'s landing refusal; the passage leaves such a visitor out of its hero, so the town
  is visited on every neighbour object's path), and the reach's cold `unmark_visitors` pass is
  kept, the walking loop untouched (the bench against `master`: the passage +0.7 % of its own
  count, `reach-lifted-xl-144` +0.24 %, `random-play-duel-36` +0.16 %, `reach-xl-144` −0.03 %;
  inside the gate). `siege_start` gives the visitor
  `Defender::Hero` (its army as it stands). **The merge** is the game's composition, as an army
  command's: `settle_outcome` calls `h3_game`'s `town::merge` before `start_battle` for a siege
  whose defender is a hero; it calls `AdventureState::merge_garrison(core, part, town)`, which
  plans with `h3_adventure`'s pure `town::merge` (the rounds above; the room is the joins'
  `room` with the part's slots and a stack's most, ADV-ARM-007; a creature the ruleset lacks is
  worth 0), writes both armies (the garrison only when it changed) and returns the slots it
  changed, the visitor's then the garrison's, each in slot order; the game sets the battle's
  defending army to the merged one and tells each slot as `StackChanged` to its army's owner (the
  visitor's to the visitor's owner, the garrison's to the town's, a neutral town's to nobody)
  (the events' **agent's call**, the user's choice of three: a new `ArmyRef::Garrison(TownId)`,
  which no army command may name before increment 3's town screen, `ArmyCommandError::Garrison`
  in the adventure and the game's `admits`; `h3-cli`'s slot text `t<town>:<slot>`), before
  `BattleStarted`. T8b1's tests that start a siege directly (`start_battle`) merge nothing. No
  version moved: no state field, no hash input, no replay with a town.

### 11. Capture (TWN-CAP-001 to -003; T8a, the siege's case T8b)

- **`AdventureState::capture_town(id, player, radius, sink)`** (infallible): the owner written,
  the garrison emptied (written empty), the reveal for the new owner by `radius` around the
  entrance (all tiles of every level for `all`), `TownCaptured { town, anchor, owner }` to every
  player that sees the town and the new owner, `TilesRevealed` to the new owner alone.
- **The game composes it** (ADR-0017's amendment point 7): `h3_towns::reveal`, then the adventure's
  capture, then the Capitol's limit (`capitol_limit` with whether the new owner holds another
  standing Capitol; the start never applies it), `TownChanged { town, capitol, seen_by }` when it
  changes, to `TownCaptured`'s viewers (§12). One infallible application: the checks are the
  step's.
- **After a siege won** (§8.2, row 4): the attacker stays on the tile it attacked from with its
  points; the town is captured; a later step finds it own. A defender's win or no winner changes
  no owner; the survivors stay in the armies that fought (the visitor's, else the garrison's as a
  mine's guards).
- **As built (T8a2):** `AdventureState::capture_town(core, town, player, reveal: TownReveal,
  sink)` (`core` for the viewers' fringe, as a mine's flag): the owner written through the
  ownables' block, the garrison written empty, `TownCaptured { town, anchor, owner }` to the
  players that see the town and the new owner, then `TilesRevealed` to the new owner alone;
  `TownReveal { Radius(u8), All }`, `All` every tile of every level, also `start_game`'s
  (`&[TownReveal]`; `h3_game::town::adventure_reveal`, which was `start_radius`): `TWN-STUB-5`
  lifted. The game's `town::capture::capture`, called first in `settle_outcome`'s loop (before the
  held rests, the gains and any decision): `adventure_reveal`, the adventure's capture, then the
  Capitol's limit with `holds_another` = another town of the map owned by the new owner (the
  adventure's owners, the captured town left out: its owner is already written) with a standing
  Capitol (`Towns::capitol`), and `GameEvent::TownChanged { town, capitol, seen_by }` when the
  limit changed it, told to the capture's viewers (`capture_town` returns them; §12's rule, the
  review's call: no player that does not see the town learns its Capitol); the limit runs only
  after a capture. A capture's events: the step's `HeroStepped`, `TownCaptured`,
  `TilesRevealed`, then `TownChanged`.
- **As built (T8b1, the siege's case; TWN-CAP-003):** the capture after a siege won comes once
  the attacker's owner has no open decision, after the winner's levels (survey §9.3: the
  original's levels at 0x4AE324 inside `adventure_battle`, before `town_visit`'s capture), as a
  guarded object's reward: a rest of the visit, `Rest::Won(entrance)` (GAM-DEC-007), whose
  `AdventureState::give_won` meets the town first (`StaticMap::town_at`) and gives
  `AdventureState::town_won`: `AdventureOutcome::capture` for the hero's owner unless the town is
  of its team by then, nothing when the hero left the map (a win with only created stacks, as a
  mine's flag); the game applies it with T8a2's `town::capture` (the leftovers of the garrison
  lost with it). Against a garrison without a hero `AdventureState::end_unowned_battle`'s
  `ObjectKind::Town` arm (`end_town_battle`) writes the garrison (empty after the win, else the
  survivors by slot) and holds the capture by `reward_after`; against a visitor
  `GameState::end_battle` holds it itself once the attacker won with an army (the beaten visitor
  already off the map, its type free). The battle's town is derived, not stored: a battle whose
  state has a siege (`h3_battle::computer::siege`) is at the town whose entrance is its tile
  (`settle`), so `OpenBattle` and the game state hash are unchanged; `end_battle` takes it as
  `town: Option<TownId>`.

### 12. Information (TWN-VEW-001 to -003; `docs/06-security.md`, T8c)

The words for `docs/06-security.md` (a section "Towns (E09 increment 1)"):

- **Public where the viewer has explored a tile of the town** (`ObjectView` gains `town:
  Option<TownId>`; the game's `TownView`, §17): its owner, faction, name, fort level, whether it
  has a Capitol; whether it is defended only as the step and the legal moves tell it (a reachable
  town's cursor).
- **The garrison** to another team's viewer by its **level**, the number of the viewer's towns
  with a standing `tavern` building, at most 2: 0 nothing (not even whether it is empty); 1 each
  stack's creature, packed in slot order (never the slots); 2 also each stack's size word (E08's monster words). The counts, the
  hall, the day's income and the silo's resources for the **owner's team**, or a viewer one of
  whose heroes has a Rogue in its army with the town on its level within its Visions radius
  (Visions' expert factor × the hero's power, at least 3; the factor a town part parameter until
  Visions is placed, *agent's call*). A neutral town is nobody's team.
- **Never public:** the town's other buildings, the Escape Tunnel, the defender's bonuses, the
  ever-built set; the merge's stacks (its owner's); a draw's values.
- **Events:** `TownCaptured` and `TownChanged` where the town is seen; the reveal to the new owner
  alone; a siege as any battle (its sides' owners), others learn its effects where they see the
  town.
- **As built (T8c2; decision 31):** the top level (`OWNER_LEVEL`, the counts and `owner_facts`,
  never `GarrisonView::army`) also for a viewer one of whose own heroes on the map has a creature
  of `TownParams::visions_creatures` in its army (any pack's `[town_visions]`; the AB pack's
  Rogue) and stands on the level of the town's **entrance** with `dx² + dy² < r²`, `r` the
  part's `visions_factor` (3) × `h3_heroes::power`, at least `visions_min_radius` (3); the
  heroes' radii taken once an observation, only when a town of another team needs them. The
  words for `docs/06-security.md` went there in a `shared/` pull request.

## Part C — the game, the env, batch TWN-1 and the cut

### 13. The setup and its checks (`h3-game` `src/town*` and `setup.rs`, `cross-lane`; T7a)

- **`PlayerSetup`** gains `start: PlayerStart` (default: none of it, for setups without towns):
  `factions: FactionSet` (a bit set over the core's factions), `choice: Option<FactionChoice>`
  (`Random`, `Fixed(FactionId)`), `pick: Option<HeroTypeId>`, `main_town: Option<MainTown {
  town: TownId, generates_hero: bool }>`, `random_hero: bool`. Whoever seats the players may change
  `choice` and `pick` before the build (decision 6's seam).
- **`GameSetup`** gains `towns: Vec<TownSetup>` (by `TownId`, joined to the static map's town
  list), `allowed_heroes: Vec<HeroTypeId>`, `hero_masks: Vec<(HeroTypeId, u8)>`, `prisoners:
  Vec<HeroTypeId>`; `GameHeroSetup::hero_type` becomes `Option<HeroTypeId>` (none: a random hero).
- **`GameSetup::validate`**, one check per invariant (ADR-0015 point 4), new errors in order:
  `TownCount` (the setup's towns and the static map's), `TownFaction` (a fixed faction without a
  town record), `TownAlignment` (a colour of no player), `Choice` (a fixed choice outside the
  player's set or without a town record; `Random` with none), `MainTown` (a town of another owner),
  `Pick` (outside **the offer**: for a `Fixed` choice with `generates_hero` or `random_hero`, the
  first 16 in type order of the free types of the faction's two classes the colour may take, after
  the placed heroes and prisoners are taken), `PickTaken` (two players), `RandomHeroOwner` (no
  player of the game).
- **As built (T7a2):** `h3_game::town::check_starts`, after the heroes' count and before the
  `HeroDraw` refusal (`TWN-STUB-3`), one pass each: `TownFaction`, `TownAlignment`, `Choice`,
  `MainTown`, `Pick`, `PickTaken`, `RandomHeroOwner`. **`TownCount` and `MainTown`'s owner move to
  T8a** (the user, 2026-11-04: the towns have no owner and the static map no town list before it);
  `MainTown` refuses a town that is none of the setup's. `TownAlignment` refuses a colour no
  player has, though the original falls through to the owner (the user, the same day; none on the
  shipped maps). *Agent's calls:* `Choice` also refuses a set holding a faction without a town
  record (`TWN-QUIRK-4`) and no choice where the start needs a faction (a main town, the
  random-hero flag, a random hero of its own, a random town aligned to its colour); the offer is
  public (`GameSetup::offer`, `OFFERED_HEROES` = 16) for whoever seats the players. The census:
  no shipped map refused by a new check (the import picks nothing, so `Pick` is not exercised).

### 14. The pool and the build's draws (`GameState::new`; game state hash v23, T7b)

- **The pool** (ADR-0017's amendment point 1, TWN-SET-005): `Arc<HeroPool>` in `GameState`, per
  core hero type `Free`, `Taken` or `Out` (not allowed) and its colour mask; built from the setup's
  allowed heroes and masks; the placed heroes' and prisoners' types taken, then each pick reserved
  (T-2, T-3 fixed: `TWN-QUIRK-2`, `-3`); a hero that leaves the map goes back `Free` (the beaten
  hero of every battle, a siege's too). Hashed after the store of heroes.
- **The draws** (ADR-0017's amendment point 4), from the setup stream before every existing one:
  1. each player's `Random` choice, colour order: uniform over its set (TWN-SET-001);
  2. object order: a random town's faction (TWN-SET-002: an alignment to a colour that plays gives
     that colour's resolved faction, else the owner's, else uniform over the factions with a
     town record: eight with the base pack, nine with AB; TD-1 by the reading, TD-2 by the
     packs) and a random hero's type (TWN-SET-008: its owner's pick for the owner's first random
     hero, else the draw);
  3. each main town's hero, colour order, when it generates one (TWN-SET-009): a human player's
     pick, else the draw; it stands on the town's entrance (its visitor), its army and experience
     the type's placeholders (GAM-SET-001, GAM-HER-001, drawn with the existing ones); its hero id
     after the setup's heroes, in colour order;
  4. the towns in `TownId` order: §5's buildings and garrison.
  **The draw** (TWN-SET-007): uniform over the free types of the faction's two classes (the core's
  classes of the faction) the colour may take, else over every free type it may take, else
  `GameSetupError::NoFreeHero` (T-4, `TWN-QUIRK-4`). An unused pick (a computer colour's without
  a random hero) goes back `Free` after step 3 (*agent's call*). The resolved choices are not kept
  (nothing reads them in increment 1, *agent's call*).
- **Then** the existing build: the adventure's setup with the towns' garrisons and each town's
  reveal radius, the store of towns (`Towns`) beside the adventure's state, the levels at load
  (the random heroes' 18 custom experiences included).
- **The cut (the user, 2026-11-04):** T7b in two at the towns, whose owners and objects come with
  T8a. **T7b1** the pool and the heroes' draws (step 1, step 2's random heroes, the draw, game
  state hash v23); **T7b2** step 2's random towns, step 3, step 4 and the store of towns, with the
  garrisons and radii into the adventure, once T8a gives the town list and the towns' owners.
- **As built (T7b1):** `h3_game`'s `town::pool` (`HeroPool`, `HeroStatus` free, taken by a
  colour, a prisoner's or out, each type's mask the AND of the setup's masks; `draw_heroes`);
  `GameState`'s `Arc<HeroPool>`, hashed after the store of heroes; `GameSetupError::NoFreeHero`
  (the drawing player); `end_battle` frees the type of every hero it removes from the map;
  `TWN-STUB-3` narrowed to a main town that generates a hero (`town::main_town_heroes`). Order:
  the placed heroes' types checked, the pool built, then the draws, before the starting armies.
  *Agent's calls:* a `Random` choice is drawn only when the start reads it (`town::needs_faction`,
  the test `Choice` uses: a main town, the random-hero flag, a random hero of its own, a random
  town aligned to its colour), so a setup whose choices nothing reads keeps its stream (the
  fixture maps of 12 replays have two Random players and no town); T7b2 adds a random town the
  player owns to that test. Step 2's random towns come after the random heroes (T7b2 appends
  them, so T7b1's draws do not move). A placed hero's and a prisoner's type are taken whatever the
  allowed set, and a hero that leaves the map frees its type even if the map did not allow it (the
  original's 0x4D9EC0). `validate` with the seed 0 may differ from a build with another seed in
  two cases, stated in its doc: `NoFreeHero` when a colour mask leaves a random hero's owner out
  of types an earlier draw may take, and a drawn type without a valid starting army (none in the
  core). The census: 31 shipped maps build (9 before), 125 refused for a main town's hero.
- **T7b2 cut in two (the user, 2026-11-04) at the store:** **T7b2a** step 2's random towns, step
  4, the store of towns, the garrisons and radii into the adventure (game state hash v24);
  **T7b2b** step 3, the main towns' heroes (its draws then come between step 2's and step 4's,
  which moves no recorded game: those maps are refused until it), the unused picks freed after it,
  `TWN-STUB-3` lifted.
- **As built (T7b2a):** `h3_game`'s `town::build` (`draw_towns`, `build_towns`, `start_radius`):
  `adventure_setup` runs `check_starts` (which now returns the towns' owners at the start), the
  heroes' draws, then `draw_towns` (each random town in `TownId` order, appended after the random
  heroes), then `build_towns` (`h3_towns::town_setup` in `TownId` order, owned when it has an
  owner) before the starting armies; the garrisons go into `AdventureSetup::garrisons`, each
  town's `h3_towns::reveal` into `start_game`'s radii (TWN-CAP-001's start), the records into
  `GameState`'s store of towns (`Towns`, `GameState::towns`), hashed after the pool (game state
  hash v24; a fork of a map without towns counts nothing, with towns one reference).
  `needs_faction` also reads a random town the player owns (so `Choice` refuses such an owner
  without a choice and its `Random` choice is drawn). *Agent's calls:* `TownFaction` also refuses
  a random town when no faction has a town record, so step 4 cannot fail after the checks (a town
  it could not build would be left out and the adventure refuse the garrisons' count, never
  reached); an owner that is no player draws as a neutral town (the adventure refuses it after);
  `Reveal::All` (a Tower's Grail built at the start) gives the radius 255, which covers the
  town's level on any map up to 181 tiles a side but not the other level (`TWN-STUB-5`
  narrowed); the censuses' static-layer games give each town the first faction with a town record
  (fixed), else `Choice` refuses the owned ones. The census: 31 maps build as before (294 towns:
  139 random towns by their owner, 52 drawn, none aligned; 79 neutral towns with a drawn garrison,
  16 of them drawn empty; no start radius other than 5), none refused by a new check; of the 156
  imported maps 9 towns have a custom set with a Grail that reveals every tile, 4 one with a
  Lookout Tower.
- **As built (T7b2b):** `h3_game`'s `town::heroes::main_town_heroes` (TWN-SET-009) after
  `draw_towns` and before `build_towns`: per player in id order (the colours strictly increase,
  so colour order) whose main town generates a hero, a human's pick unless its first random hero
  took it, else the draw over the player's resolved faction for its colour (`NoFreeHero`), taken
  at once; the hero on the town's entrance (`check_starts` now also returns each such player's
  entrance, the first visitable tile on the map as the adventure's `MapObject::entrance`; a main
  town without one is refused as `MainTown`), the starting army, the default formation and
  details; its id after the setup's heroes; the armies, then the details drawn in hero id order
  over both (so the setup's heroes' experience draws follow the new heroes' armies, on maps the
  build refused before). `draw_heroes` frees no pick now: every unused pick goes back free after
  step 3. `HeroDraw` and `GameSetupError::HeroDraw` are gone; the heroes' count includes the
  main towns' heroes. *Agent's calls (the user told):* a second hero on an entrance (a moved
  placed hero, TWN-SET-010, and a main town's) is refused by the adventure's
  `SetupError::HeroTile` (`Occupied`; `TWN-QUIRK-6`, the original draws the new hero over the
  visitor); the guild's spells at the start (0x5BE030) wait for increment 4 (`TWN-STUB-6`; TD-10:
  VCMI teaches none). At the user's question the hero right of an entrance was read again: the
  visitor, never a garrison hero (survey §4.6). The census: all 156 imported maps build (523
  main towns' heroes, all drawn; 33 with a spell book in a town with its Mage Guild).

### 15. The siege's battle (GAM-BAT-001 amended; TWN-SIE-003 to -005; T8b)

The game builds a siege's `BattleSetup` as a mine's (`state/battle.rs`) when `BattleStart::town`
is set:

- `siege: Some(h3_towns::siege(town))`; the defending hero's `BattleHero` raised by
  `defender_bonus` (ADR-0023 point 3); after the battle its mana the smaller of the mana left and
  the mana before.
- **The terrain:** the faction's native terrain (the E04c survey's "the town's native terrain",
  `code`), no special battlefield (*agent's call:* the shipped maps' census in T7a says whether a
  town's entrance lies on a special ground).
- **The obstacles:** `generate_setup_obstacles` (none at a walled town); with `siege_obstacles`
  (the base pack's Stronghold from a Citadel) the usual ones on the terrain
  (`generate_battlefield_obstacles`), from the same seed (ADV-QUIRK-5's rule unchanged).
- **The defending side** the visitor's owner, the town's owner for its garrison, the neutral seat;
  the attacker's Catapult is the battle's (ADR-0023 point 6: the town queue's inbox note on VCMI's
  catapult is settled).
- **The outcome:** the attacker's win captures (§11) with the attacker outside; a beaten hero
  leaves the map and goes back to the pool; the survivors as §11.
- **As built (T8b1):** `BattleStart::town: Option<TownId>` (the adventure's; none from every
  visit until T8b2). `h3_game`'s `src/town/siege.rs`: `siege_setup`, called by
  `GameState::start_battle` after the field battle's setup and before the obstacles' draw:
  `siege` (TWN-SIE-003), the defending hero raised (`raise`: attack, defense and power at most the
  core's `primary_max`, the spell points saturating; TWN-SIE-004), the terrain the faction's
  native one (the start's for a faction without one) and the battlefield the special ground on
  the entrance if any, never the shore or a deck (TWN-SIE-005: **the code reading, not the agent's
  call above**: `installation.md` §7.26 has the special ground in every case after a siege's
  terrain). The obstacles: `generate_setup_obstacles` (none at a fort); the base pack's walled
  Stronghold's usual obstacles are `TWN-STUB-8` (the original keeps them off the walls by a branch
  for town type 6 not read, 0x466147 to 0x466194, and `generate_battlefield_obstacles` knows no
  walls: the battle would refuse an obstacle on a wall; the battle lane's inbox). After the
  battle: the defending hero's spell points `mana_after` (the smaller of left and before) in the
  heroes' part; **the attacker's +500 experience** as the winner of a battle at a town
  (`siege_experience`, TWN-SIE-006: ADR-0023 point 1 and the code, 0x46A416, re-read: added
  whatever the result, after the defeated hero's +500 and the −500 of a retreat or a surrender),
  which this note had not listed. No version moved.

### 16. The day's income (TWN-INC-001, -002; T8c)

In T85b's day advance (`E08-design.md` §14 step 4, the adventure lane's), per player in id order,
beside the mines' and the heroes': each town it owns (the adventure's owners by `TownId`) adds
`h3_towns::daily_income` (gold and the silo's resources), gained saturating, one `ResourceGained`
per resource as today. From day 2; a neutral town pays nobody; a capture during the day pays the
new owner the next day. Several Capitols at the start pay 4 000 each (§11.3).

- **As built (T8c1; T8c cut in two at its start, the user, 2026-11-05: the Rogue's Visions,
  `docs/06-security.md` and the inbox notes in T8c2):** `h3_game`'s `town::add_income` (the
  town lane's `src/town/income.rs`) adds each owned town's `daily_income` into `pay_income`'s
  vector after the heroes', so the towns' gold and resources join the mines' and the heroes' in
  one `ResourceGained` per resource; the adventure lane's `day.rs` gains one call. No version
  moved: the state is unchanged, only the treasury's day changes, and no fixture map has a town.

### 17. The observation (T8c)

`GameObservation::towns: Vec<TownView>` in `TownId` order, the towns of which the viewer has
explored a tile: `town`, `anchor`, `owner`, `faction`, `name`, `fort`, `capitol`, `garrison:
GarrisonView` (`level`, then by level the creatures packed in slot order, their words, their
counts),
`owner_facts: Option<TownOwnerView { hall, income, silo }>` (§12's rules). The adventure's
`ObjectView::town` links an object to its view. `project` passes `TownCaptured` and `TownChanged`
to the viewers of §12. The map client (E24) and the env read the same (notes to their inboxes
when T8c lands).

- **As built (T8c1):** `ObjectView::town: Option<TownId>` (every seen object; `StaticMap::town_id`)
  and `GameObservation::towns` from `h3_game`'s `town::view::town_views`, over the adventure
  observation's objects, so in `TownId` order: a town a tile of which (blocked, visitable or
  drawn) the viewer has explored. A town only on the fringe, which `faithful` shows as an object
  (ADV-QUIRK-1), has its object's `town` but no view (the popup reads an explored tile only,
  *agent's call*). `TownView { town, anchor, owner, faction, name, fort, capitol, garrison,
  owner_facts }`; `GarrisonView { level, stacks, army }`: the stacks packed in slot order (the
  popup's places, never the slots), `GarrisonStackView { creature, count_word, count }` (the word
  from level 2, the count at 3), `army` the garrison by slot to the owner's team only (as a mine's
  `GuardsView`, the town screen's, which T8b2b's `StackChanged` with `ArmyRef::Garrison` changes);
  `TownOwnerView { hall, income }` (the income `daily_income`'s, the silo's resources within it).
  The level: `OWNER_LEVEL` (3) by `h3_adventure::allied` (made public), else the viewer's own
  towns with `has_tavern`, at most `TAVERN_LEVELS` (2), counted once an observation. The
  Rogue's Visions was `TWN-STUB-9`, lifted by T8c2 (§12). `project` was already the §12 one (T8a2, T8b1).

### 18. The env, replays, fixtures, properties, benchmarks (T9)

- **Fixtures** (the adventure lane's paths, `cross-lane`): `e09-towns-36.h3m` written by
  `h3_map::writer` (`crates/h3-map/src/tests/towns_map.rs`): owned towns of several factions with
  standard and custom sets (a Capitol, the Grail building, a silo), a neutral town drawn and one
  explicitly empty, a random town of each kind (aligned, owned, neutral), a main town that
  generates a hero, a random hero, a hero right of a town's entrance, a garrison with random
  stacks; the XL map `e08-xl-144.h3m` stays as it is (its fork benchmark measures the store's
  reference count).
- **Golden replays** (`fixtures/replays/adventure/towns-*`): random play on the town map; scripted:
  an undefended capture, a siege won against a garrison, a siege lost against a visitor with the
  merge, a Capitol limited, the day's income over a week. Every existing replay re-recorded at
  each bump (skill `homm3-rerecord-replays`).
- **Properties:** the plans (`crates/h3-map/src/tests/play.rs`) play towns: a capture keeps one
  owner per town and the garrison empty, the store forks and hashes as the state, a hero never
  stands on a town's entrance of another team.
- **Benchmarks** (`benches/adventure/`): `fork-xl-144` (the store adds one reference count),
  `day-start-towns-36` (new), the reach on the town map (the entrance test), a siege's setup.
- **The env** (`h3-sim`, the AI lane's, `cross-lane` with consent as decision 7): no new command,
  so no action encoding change; a test playing the town map to its end of days; the bot reports
  regenerated.
- **`--with-original-data`:** the 160 maps import and build (exit criterion 1).

### 19. Versions, rule ids, quirks, stubs

| Version | Change | Task |
|---|---|---|
| town state hash v1, town ruleset part v1 | new | T4b (constants), T5a, T6 |
| adventure replay format v9 | the town part's hash in the header | T5a |
| map import v17 (E08 T93b took v16) | towns, players' starts, the random hero, the main town, the moved heroes, the pool's inputs | T7a |
| game state hash v23 (after T85b's v22), v24 | the pool (T7b1); the store of towns (T7b2a) | T7b1, T7b2a |
| adventure ruleset part v17 (T93a took v16), adventure state hash v21 (E08 T94b took v20), map import v18 | the kind `Town`; towns ownable, the town list, `Holds::Town`; the towns typed with their owners | T8a1 |
| town ruleset part v3 | Visions' factor, least radius and creatures in the parameters | T8c2 |
| env encodings | none (no new command; no flat adventure observation, E08 decision 187) | — |

**Rule ids** (`docs/rules/catalog/town.toml`, created by T6): TWN-SET-001 the players' factions,
-002 a random town's faction, -003 the buildings at the start, -004 the garrison at the start,
-005 the pool, -006 the starting-hero pick (the offer, reserved), -007 the draw, -008 the random
hero, -009 the main town's hero, -010 a hero right of an entrance moved; TWN-VIS-001 the step,
-002 the visitor; TWN-CAP-001 capture, -002 the Capitol's limit, -003 the attacker outside after a
siege; TWN-SIE-001 who defends, -002 the merge, -003 the siege's facts, -004 the defender's
bonuses and mana, -005 the terrain and obstacles, -006 the attacker's experience (T8b1); TWN-INC-001 the gold, -002 the silo; TWN-VEW-001
the public facts, -002 the garrison's levels, -003 the owner's team. Amended: ADV-OBJ-002 (the
classes 77, 98 typed), ADV-OWN-002 (towns' owners, the start's reveal), GAM-BAT-001 (a siege),
GAM-RES-004's "not here" line (the towns'), GAM-SET-001 (the starting armies of the drawn heroes).

**Quirks** (`docs/rules/quirks-town.md`, created by T6c with `-5`; T7a1 added `-1`, `-4`; T7b1 `-2`, `-3` with the pool), none with a flag: `TWN-QUIRK-1` T-1 (an
alignment byte from 8 to 254: refused), `-2` T-2 (the pick not reserved: fixed), `-3` T-3 (a
random hero taking a later hero's type: fixed), `-4` T-4 (the draws' empty cases: typed), `-5`
T-6 (a custom set's horde leaving its dwelling standing: fixed, decision 29; T6 and T7a), `-6`
T-7 (a main town's hero drawn over the town's visitor: refused, T7b2b). T-5
comes with increment 3 (decision 12). TD-2's map-version test is the rules' recorded deviation
(decision 14), not a quirk (as E08 decision 104).

**Stubs:** `TWN-STUB-1` the siege of an ally's visitor (E10), `TWN-STUB-2` the start's forbidden
Grail and Shipyard (increments 2, 4; registered by T6c); `TWN-STUB-3` the build's
refusal of a setup that needs a draw (registered by T7a1, narrowed by T7b1 to a main town's
hero, lifted by T7b2b); `TWN-STUB-4` a town's entrance not entered (registered by T8a1, lifted
by T8a2), `TWN-STUB-5` the build's empty garrisons and flag radius for the towns (registered by
T8a1; narrowed by T7b2a to a whole-map reveal at the start, which reaches only the town's level;
lifted with the capture's reveal, T8a2); `TWN-STUB-6` a main town's hero learning no guild spell
at the start (registered by T7b2b; increment 4, TD-10); `TWN-STUB-7` a defended town refused
until the siege's start (registered by T8a2; narrowed by T8b2a to a visitor of another team;
lifted by T8b2b); `TWN-STUB-8` a walled Stronghold's natural
obstacles (registered by T8b1; a siege-aware draw in `h3-battle`); `TWN-STUB-9` a Rogue's
Visions not giving another team's viewer the top level (registered by T8c1; lifted by T8c2).

### 20. Batch TWN-1 (T3)

Rows 1 to 4 (§13, decision 15) in one run on the harness (`clients/capture/`, common) in SoD on
Easy, hot seat, from maps `h3-experiment` writes; several new games for row 1's 30 in 100.

- **The spec** (`h3-experiment`, the battle lane's, `cross-lane`): `[[map.town]]` and `TownSpec`
  gain what the rows lack (today an absent `buildings` is the standard set without a Fort, an
  empty `garrison` the map's default): the standard set with a Fort, an explicitly empty custom
  garrison; row 3's town with a visitor and a garrison is an encounter's opponent town (E04c
  T5b's `TownSpec` with its hero and garrison). The writer's `Town` already writes either set and
  either garrison.
- **The harness** (common; this lane builds it with its check and notes the client and map-client
  lanes' inboxes): reads of a town's record (owner +0x1, built +0x150 and every-built +0x158,
  garrison +0xE0, the visitor +0x10), the battle's opening stacks at a siege (E04c's), a hero's tile
  and points after a battle (E08's).
- **The scoring** (T3c, done 2026-11-03): survey §15. TD-3, TD-5, TD-7 and the siege's aftermath
  as the reading (VCMI refuted in each); §10 and the ADR drafts stand. Not predicted: building 23
  (§3, §5; `TWN-Q5`, decision 25: re-read in T5b1, survey §16: the Village Hall's town-screen companion, left out, decision 27).

### 21. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T4 (`shared/`, §1); the harvest (T5b, `h3-harvest`); batch TWN-1's
spec (`h3-experiment`, T3a); `GameRuleset`'s callers and the replay header (T5a); every version
bump of the adventure lane's (T5a, T7a, T7b, T8a), the setup structs, the `lib.rs` lines, the
fixtures and replays (T9); the env (`h3-sim`, T9). Each `cross-lane` pull request merges in order
with the adventure lane's (its T85b, T86, T87 first where both bump; the town lane's `TRAPS.md`).

**The building tasks**, in rounds (ids continue the epic's; each task's cut confirmed at its
start). **Batch TWN-1 runs before the build and before the shared task** (as E08's increments), so
the ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T3a** | Batch TWN-1's maps and spec (§20; `cross-lane` in `h3-experiment`) |
| 3 | **T3b** + **T3c** | T3b: the harness's town reads, the recipe, the run. T3c: scored (survey §15); TD-3, TD-5, TD-7 and row 4 settled |
| 4 | **T4a** + **T4b** | `shared/`, cut in two at its start: T4a the ADRs with T3c's outcome and the glossary (done); T4b the crate skeletons, `TownId`, the gates (§1) |
| 5 | **T5a** + **T5b** | T5a: the part, its compiler and adapter, `GameRuleset`, replay format v9 (§2). T5b: the content read (T5b1, survey §16) and harvested (T5b2), building 23's re-read (§3; original data) |
| 6 | **T6** | `h3-towns`: the record, the store, the setup's functions, the derivations, `town.toml`'s rules (§4–§6) |
| 7 | **T7a** + **T7b** | T7a: the import and the setup's checks (§7, §13; map import v17, the census). T7b: the pool and the build's draws (§14; game state hash v23) |
| 8 | **T8a** + **T8b** | T8a: the kind, the step, entering and an undefended capture (§8, §9, §11; part v16, state hash v20). T8b: the siege, cut in two (the user): T8b1 the battle's setup and its end (§15, §11; done), T8b2 the step onto a defended town, the merge, the defender (§9, §10) |
| 9 | **T8c** + **T9** | T8c: the income, the views, the observation, `docs/06-security.md` (§12, §16, §17), cut in two (the user): T8c1 the income and the observation's towns (done), T8c2 the Rogue's Visions, the security doc, the notes. T9: the fixtures, replays, properties, benchmarks, the env (§18) |
| 10 | **T10** | The audit against the exit criteria; increment 1 closed; increment 2 refined (done, 2026-11-06) |

**Increment 1's exit criteria** are the epic's seven (moved to `E09-archive.md` at the closure); T10 checked each against its rules and tests (all hold: `E09-log.md`, T10).

### 22. Risks and open points

- **Two lanes in the same crates** (the epic's risk): T5a, T7a, T7b and T8a bump the adventure
  lane's versions; the adventure lane's T85b (game state hash v22, the day advance the income
  joins) and T86 (the replays) merge first. T3, T4's docs, T5b and T6 do not wait for it.
- **The reach's hot loop** (§9): the entrance test is measured against `master` right after its
  first build; the town map adds a workload.
- **The content** (§3): nine factions of 34 to 37 listed buildings and the Village Hall, the
  supersede chains read (survey §16.1); VCMI's lists agree but for TD-8 (decided, 30) and the
  requirements' TD-9 (increment 2).
- **The ever-built set as the record** (§4): construction (increment 2) adds a building and its
  supersedes (the hordes' upgrade too); the Capitol's limit the only removal. If increment 2 finds
  a rule that needs the original's two sets apart, the record grows a second set (the hash
  bumps); decision 29 kept the derived set against the original's path (T-6 fixed).
- **Not read** (survey §6, §10): the boat spot (0x5C0C90), the Grail victory's town (0x5F2160)
  (`TWN-STUB-2`); a special ground under a town's entrance (§15, T7a's census); the Visions
  factor's source (§12).
- **Teams:** every team test is the player alone until E10 (`TWN-STUB-1`).

## Part D — increment 2: construction with the town events

### 23. Scope, sources and choices

Increment 2's note (the epic's decision 37). A handover, not binding, as Parts A to C: the working
detail behind the drafts `docs/lanes/town/adr-drafts/0028-increment-2.md`, `0016-increment-2.md`
and `0017-increment-2.md` (moved into ADR-0028, -0016 and -0017 by T15), written by T13 from the survey `docs/rules/surveys/E09-construction.md`
(§ numbers are the survey's unless they name this note) and the decisions 41–44 (`TWN-Q8`) and
45–47 (`TWN-Q9`). Accepted at the checkpoint (`TWN-Q10`, the epic's decisions 48–51): the drafts as
written, the cut of §35, the exit criteria, the agent's calls. The task that builds a section
shortens it to what it built; the code cites it as "E09 design §N" ("design §N" in the town
modules). *Agent's call (change freely)* marks a choice of this note the user kept (decision 51)
and the building task may still change.

- **Built:** the build command (§27, §29): the requirements and costs (§24, §25), one a town a
  day (§26), the forbidden buildings, one Capitol a player, the Shipyard's water (§28); the map's
  town events' resources and buildings (§30); the views (§31); the env, the gates (§32); batch
  TWN-2 (§33).
- **Not built** (each with the increment that builds the building's effect, decision 37): a
  dwelling's first creatures and the town events' creatures (increment 3: `TWN-STUB-10`); the
  guild's level and spells, the heroes' visit after a build, the Tower's Lookout Tower and Skyship
  reveals (increment 4: `TWN-STUB-12`); the town's boat on its spot (increment 4); the Grail's
  building and its forbid on a Grail-victory map (increment 5 and E10: `TWN-STUB-2`'s Grail half);
  the scenario's tests after a build (E10); the computer's every other day on Easy and Normal
  (E10, E13; decision 43); an ally's visit (E10's teams: `TWN-STUB-11`); the tutorial's exception
  (out of scope).

### 24. The town part: requirements, costs, the role `shipyard` (`h3-towns`, `h3-pack-towns`; part v4)

Built by T16a (`E09-log.md`).

- **The format** (`h3-pack-towns`' `lib.rs` documents it): each `[[town.building]]` has
  `requires = ["key", …]` (its **direct** requirements, buildings of the same record, in the
  executable's list order; absent: none), `cost = { gold = 2000, wood = 5 }` (core resources,
  another pack's with its prefix as a silo's; each amount at least 1; absent: free) and the role
  **`shipyard`** (a flag, at most one building a record: the original's 6).
- **Compiled:** `Building::requires: BuildingSet`, **the closure** (the original tests
  `building_requirements` 0x697798, the transitive closure, §2.1: after a town event built a
  building without its requirements, a building that needs it still needs theirs, §8.3), never
  itself; `Building::cost: Vec<(ResourceId, u32)>` in the core's resource order. Refused, each a
  `TownCompileError` of the existing variants: a requirement outside the record, a building
  requiring itself, directly or through others (a cycle; the first of it in the record named), a
  cost's unknown resource or one outside the pack's dependencies, a cost naming a resource twice
  or 0, a second `shipyard`.
- **The hash** (v4): after each building's `auto_with`, its required set's bits and its cost
  (length, each resource handle and amount); `shipyard` after `grail` among the roles. Pinned in
  `ruleset/tests.rs`.
- **Overrides:** none (RoE's costs are E11's; SoD's are AB's file); `TownPatch` gains nothing.
- **The synthetic pack:** requirements and costs on every building but the Village Hall and the
  Grail (the halls' chain to the Capitol through the Tavern and the Castle, the forts', the
  dwellings', the hordes'), a Shipyard requiring the Tavern. The shipped packs' come from T16b's
  harvest (§25).

### 25. The content and its harvest (`h3-harvest`, the battle lane's crate, `cross-lane`; original data)

Built by T16b (`E09-log.md`), as increment 1's content (decision 23's route, design §3):
`h3-harvest`'s `src/towns.rs`; no gap-fill (the executable has every fact).

- **Costs** from `Building.txt` (§5; `towns::Tables::costs`), its three tables as
  `building_costs_load` 0x5C14C0 reads them (`cost_row`): the faction's buildings 17 to 25 from row
  4, eleven rows a faction; the neutral 0 to 16 from row 104; the dwellings 30 to 43 from row 125,
  sixteen rows a faction; the ids 26 to 29 none (the Grail building is free). The seven columns in
  `RESTYPES.TXT`'s order, the zero columns dropped, the Village Hall's 1 gold kept; the AB file's
  costs name base's resources with their prefix (`"base:gold"`). Refused: a cell that is not a
  number, a listed building's blank row.
- **Requirements** from the executable's faction lists 0x63EC80 (`h3_formats::exe::towns`): each
  listed building's direct requirements as keys, in the list's order; refused: one outside the
  record. TD-9 (decision 41, batch TWN-2 as the reading): the Conflux's upgraded Altar of Fire
  keeps the list's two.
- **The role `shipyard`:** the building 6 (Castle, Necropolis, Fortress, Conflux); the gap-fill may
  not give it.
- **Checked** by the harvester's tests on its fixture (the rows by the original's layout, each
  faction's own rows, the prefix, the Grail, the Shipyard, the refusals) and by `h3-pack-towns`'
  test of the original packs (every closure inside its record and never itself, the Capitol's
  holding the Town Hall; the Capitol's and the upgraded level-7 dwelling's closures of every
  faction by §6; §5's neutral costs, the hordes' rows, the Conflux's dwellings; the four
  Shipyards); every generated `requires` also matched §6's dump by a throwaway script.

### 26. The record and the store: the day's mark (`h3-towns`; town state hash v2)

Built by T17a (`E09-log.md`).

- **`Town::built_today()`** (the original's +0x2 with the values 0 and 1 only, decision 43;
  *agent's call*: a flag, not the day of the last build, which would hash two states that play
  alike apart); false from `Town::new`.
- **`Towns`' writes**, each copying only the town it changes (`write`; a fork's first write copies
  the list of references and that town, ADR-0028 point 4):
  - `build(town, building, part, mark) -> Option<TownFacts>`: the ever-built set becomes
    `built_with`'s (§27), the mark set with `mark` and kept otherwise (a town event's build neither
    sets nor clears it); it returns the town's public facts before (`TownFacts { fort, capitol,
    built_today }`; `Towns::facts` the same now), so the game tells their change (§31); `None` for
    a town the store lacks or a building outside its record (the game checked both: never in play);
  - `start_day()`: clears every set mark (only the marked towns are written);
  - `captured(town, clear_mark) -> bool`: the capture's rule of the mark (§30), whether it wrote;
    the Capitol's limit stays `apply_capitol_limit` (the mark kept).
- **The hash** (v2): after each town's forbidden set, its mark. The game state hash folds the
  store in, so it bumped with it (v26; the replays record the game's version, not the store's).

### 27. The rules (`h3-towns`; TWN-BLD-001 to -006, TWN-EVT-003)

Built by T17a (`built_with`) and T17b (the rest; `E09-log.md`): pure functions of a town, the
part and plain inputs in `h3-towns`' `build.rs`, beside increment 1's derivations (§6). The game
gives `holds_capitol` (another town of the owner's with a standing Capitol) and `water` (the
town's boat spot, §28).

| Function | Gives | Rule |
|---|---|---|
| `buildable(town, part, holds_capitol) -> BuildingSet` | the **hall's offer**: the record's buildings `check_build` allows, in index order; empty while `built_today` and for a faction without a record | TWN-BLD-002 to -004 |
| `possible(town, part, holds_capitol) -> BuildingSet` | the hall's "not now" against "never" (§2.2, for clients over the owner's view): not forbidden, not a hall 4 while `holds_capitol`, each requirement ever built or not forbidden; neither "not built", the Grail nor the mark tested (the original's); holds `buildable`'s set | TWN-BLD-002 |
| `check_build(town, part, building, holds_capitol) -> Result<&[(ResourceId, u32)], TownCommandError>` | the building's cost (the part's slice; empty when free), or the first refusal in this order: `UnknownBuilding`, `Built`, `Forbidden`, `Grail`, `Requirements`, `Capitol`, `OfferedAs(BuildingId)`, `BuiltToday` | TWN-BLD-001 (the town's part), -006 |
| `built_with(town, part, building)` | the ever-built set after a build: the building and what it supersedes, then every building whose `auto_with` is all ever built, with what it supersedes, to a fixed point, with no forbidden test (survey §4 steps 1 and 3, `town_build` 0x5BE930); `None` outside the record | TWN-BLD-005 |
| `event_buildings(town, part, event, water) -> BuildingSet` | an event's buildings of the record in index order: its set less the ever-built, less what another of its buildings supersedes, less a `shipyard` without `water` (§8.3); no other test | TWN-EVT-003 |

- **The hall's offer** (*agent's call*, refined by T17b): the original's buildable-now set (§2.1)
  also holds a horde whose level's upgraded dwelling stands, which its hall shows as the upgraded
  horde (§1.2), at the same cost in every faction (`Building.txt`); the engine refuses the horde
  `h` with `OfferedAs(u)` for the record's first `u` that **supersedes `h`**, holds `h` in its
  `auto_with` with the rest ever built, and is not ever built itself, so that buying `u` leaves the
  same ever-built set; `u` is offered on its own tests (it requires the upgraded dwelling alone).
  T17b's refinement: the supersedes test (a building automatic with `h` that does not supersede it,
  the fixture's shrine, is no upgrade); a forbidden `u` leaves neither offered (a build of `h` would
  make `u`, `built_with` testing no forbidden set).
- **The order of the refusals** (*agent's call*): the hall's texts' order (built, never, missing
  requirements, the Capitol's own text) and the day's mark last. The errors carry no building but
  `OfferedAs`'s upgrade (the command names it); `Requirements` lists none (the client derives the
  missing ones from the part and the owner's view).
- **`TownCommand`** (decision 45; `command.rs`): `Build { town: TownId, building: BuildingId }`;
  `TownCommand` and `TownCommandError` `#[non_exhaustive]` for increment 3's and 4's variants;
  `TownCommandError::UnknownTown` and `NotOwner`, which the game gives (§29), come first.
- **TWN-QUIRK-7** (T-9): `SlotSet::from_bits` keeps the 41 slots, so an event's bits 41 to 47
  build nothing (the regression test with `event_buildings`).
- **Every derivation of increment 1** reads the record a build leaves (T17a's tests per role: a
  Town Hall's income, a Citadel's moat, a Tavern's view level, a silo, the Capitol).

### 28. The Shipyard's water (`h3-adventure` `src/town*`, `h3-game`; TWN-SET-011, decision 46)

- **The spot** (the adventure, the static map): the kind `Town` gains `boat_offsets` in the
  adventure part (the base pack's `[[-1, 2], [1, 2]]`, column and row from the entrance; adventure
  ruleset part bumped, `cross-lane`); `fix_boat_tiles` (ADV-SHY-001's step, after every object is
  placed and resolved) fixes a town's spot as a Shipyard's tile: the first offset on the
  entrance's level that is a tile of the map on water no object blocks or is visited from. Kept
  as the town object's `Resolved::BoatTile` (*agent's call*: the Shipyard's variant, so the static
  map's hash encoding is unchanged); `StaticMap::town_boat_spot(TownId) -> Option<Pos>`. Every
  town gets one, whatever its faction (the original computes it only for a faction with a
  Shipyard; no other rule reads it).
- **The start** (the game, after `AdventureState::new` built the static map, before the store is
  made): a town whose record has a building of role `shipyard` and whose spot is `None` gets it
  forbidden and, if its setup built it, out of the ever-built set (`h3_towns::without_water`; the
  original's buildable set less the forbidden, survey `E09-towns.md` §5.3); a building that
  required it stays (the original keeps it: a custom set's Lighthouse beside a removed
  Shipyard). No draw, no stream position moved (ADR-0017 point 4).
- **The order, read by T18a:** after. `new_game_heroes` 0x4BFE70 reads the map first
  (`new_game_bans` at 0x4BFFB9: `map_objects_place_all` resolves the random dwellings, 0x4FD90B,
  and fixes the Shipyards' boat tiles, 0x4FD935) and calls `setup_towns` (0x4C0013, whose
  `town_buildings_setup` 0x5C08C0 calls `town_boat_spot` 0x5C0C90) later in the same straight
  line of calls; so the spot is fixed in ADV-SHY-001's step, after every object.
- **The census** (with the original data, T18a, `h3-map`'s `original_data`, checked): 1 468
  towns, 67 with a spot; 625 of a faction with a Shipyard, 591 of them without a spot; the 9
  custom sets that build a Shipyard (survey `E09-towns.md` §5.3) all have a spot, none loses it.
- **As built (T18a):** `ObjectKind::Town { boat_offsets }` (adventure ruleset part v19; in the
  pack optional, none: no spot), `StaticMap::town_boat_spot`, `h3_towns::without_water` (the
  record's `shipyard` buildings forbidden and out of the ever-built set, the day's mark kept) and
  `h3-game`'s `town::without_water` before the reveals and the store. *Agent's calls:* the town's
  spot stays out of `ObjectView::boat_tile` (the Shipyard's; the town's Shipyard's view is §31's);
  `without_water` transforms the built town (the spots come after the towns' draws).

### 29. The command in the game (`h3-game` `src/town*`, `command.rs` `cross-lane`; TWN-BLD-001)

- **`GameCommand::Town(TownCommand)`**, run as `GameCommand::Adventure`: `check_ready` (the
  actor's turn, no open decision), then `town::command` (the town lane's module):
  1. the town in the store, else `TownCommandError::UnknownTown`;
  2. **the owner** (the adventure's owner of the town's object) is the actor, else `NotOwner`
     (TD-11, decision 42; a visitor of an ally's town is refused, as the original's build dialog
     refuses it: batch TWN-2, §33);
  3. `holds_capitol`: another town the actor owns with a standing Capitol (as the capture's,
     `town/capture.rs`);
  4. `h3_towns::check_build` → the cost;
  5. the treasury pays it (`GameState::can_pay`), else `GameCommandError::CannotPay` (`BuildBoat`'s);
  6. then, infallibly: the price paid (`pay_price`: `ResourceLost` per resource, as `BuildBoat`'s),
     `Towns::build(…, mark: true)`, the events (§31).
  A refusal at any step changes nothing (the store is written last). `GameCommandError::Town(
  TownCommandError)` wraps the town's refusals.
- **`legal_commands`**: after the boats, for each town the player owns in `TownId` order whose
  mark is clear, each building of `buildable` whose cost the treasury pays, in the record's
  order. A query `GameState::town_offer(town)` gives the owner's hall (`buildable`, `possible`
  and the payable ones) for a client (*agent's call*).
- **No hero**, no visit, no decision: the original's town list opens the hall (§1.1).
- **Atomic and validated** (ADR-0004): no field of the command is trusted but the two ids; the
  price is the part's.
- **As built (T18b2b; kept whole, the user, decisions 55, 56):** `h3_game`'s `town::offer`:
  `legal_builds` (chained after the boats by `legal_commands`: the owned towns by the map's town
  list, a marked town skipped before its Capitol test, the payable set of `buildable`) and
  `GameState::town_offer(rules, player, town) -> Option<TownOffer { possible, buildable, payable
  }>`, `None` unless `player` owns the town (the trusted viewer: the owner's buildings are its own,
  TWN-VEW-004), in its turn or not. The env's rollout draws from the listed commands but the
  builds (`drawn_commands`) until T19's entry; its mask never had an entry for them. Measured:
  the five `random-play-*` workloads (no towns) within 0.5 % of their baselines; on the town map
  (three games of 21 days) `legal_commands` is 0.65 % of the rollout's instructions, `buildable`
  0.34 %, `Reach::search` 97 %: no cached offer (§36).

### 30. The day and the turn (`h3-game` `state/day.rs` `cross-lane`, `src/town*`; TWN-BLD-004, TWN-EVT-001 to -004)

- **The day's start** (GAM-CAL-002 amended): `Towns::start_day()` at the income's step (the
  original's `new_day` towns' loop, 0x4C8067; nothing between reads the mark), every day from day
  2; no event (the day's start is public, and every mark clears with it).
- **A capture** (`town::capture`, TWN-CAP-001 amended): the old owner read before the adventure's
  capture; `Towns::captured(town, clear)` with `clear` when the old owner is not human (a neutral
  town's none) and the new one is (survey §3, `town_change_owner` 0x4C63B8, decision 43); otherwise
  the mark kept, so a town built in today by a human cannot be built in again today by the human
  who took it.
- **The town events** (`town::fire_events`, called by `start_group` right after the map's timed
  events, `fire_events`; decision 44):
  - for each town in `TownId` order the player owns now (the adventure's owners), each event of
    its `TownStatic::events` in order that fires for the player today, the map's timed events'
    test (GAM-CAL-004: its players, its kind by the player's `human`, its first day or every
    interval after it; TD-12 the original's): *agent's call*: one test function shared by
    `TimedEventSetup` and `TownEvent` in `h3-game`, so the day's rule is written once;
  - its **resources**: the gains, then the losses floored at 0, as a timed event's (`gain`,
    `lose`, each told to the player), then `TownEventFired { player, town, event }` (the event's
    place in the town's list; its text the client's);
  - its **buildings**: `h3_towns::event_buildings` with the spot's water, each built by
    `Towns::build(…, mark: false)` in order, each told as a build (§31): no requirement,
    forbidden, Capitol, Grail, cost or mark test (T-8's second Capitol stands, decision 44);
  - its **creatures**: none placed (`TWN-STUB-10`, TWN-EVT-004: increment 3 adds them to the
    level's upgraded dwelling's recruits, else its dwelling's, else loses them, §8.4).
- **Day 1:** the first group's turn starts inside the build, without a sink: its town events fire
  there with the start's owners; `GameState::town_events_at_start(player)` names them (as
  `events_today` names the timed events) for a client (*agent's call*).
- **Order in a turn start:** the contexts' start-of-turn steps, `TurnStarted`, the map's timed
  events, the town events. Nothing of increment 2 reads the movement points or the heroes there.
- **As built (T18c, 2026-11-10; decision 57):** `h3_game`'s `town::fire_events` (`town/events.rs`),
  called by `start_group` after the map's; the day's test `setup::Timing`, which
  `TimedEventSetup::fires` takes too; per event its gains, its losses, `TownEventFired { player,
  town, event }`, then each of `event_buildings`' set (water: `StaticMap::town_boat_spot`) not
  standing by then (an earlier building of the event may have brought it) through
  `town::command::build_told` (the command's build and tell, shared: `Told::before`, `Towns::build`
  with the mark only for the command, `TownBuilt { event }`, `Told::tell`; the command now pays
  before `Told::before`, the treasury being in no view). Day 1: the build's first `start_group`
  takes a sink that keeps only `TownEventFired` (`RECORDS` false, so no view is computed), kept in
  `GameState`'s calendar block (`Calendar`: the map's timed events, whether a town has events,
  the start's events; one thin `Option<Arc>`, shared by forks, none for a map with neither; the
  start's events left out of the hash: no rule reads them, they follow from the setup) and read by
  `GameState::town_events_at_start(player)` (by town and place; not in the observation, the server
  lane's inbox). The wire: protocol v41 (after E08 T105b2's v40),
  the fold ignores the event. The plans' checks of the events' builds and gains move to T19.

### 31. Information (TWN-VEW-004; `docs/06-security.md`, decision 47)

The original's views (survey `E09-towns.md` §7.2, §12.3; construction §1.1):

- **Public where the town is seen** (`TownView`, every viewer with an explored tile of it): as now
  (faction, owner, name, fort level, Capitol), plus **`built_today`** (the popup's portrait frame
  at every level, `town_list_frame` 0x5BD700).
- **The owner** (`TownView::buildings: Option<TownBuildingsView { built, forbidden }>`, the
  standing set derivable with the part): the town screen opens from its town list at any time. Not
  the owner's team (an ally sees the town screen only at a visit) and not Visions' top level (the
  popup shows no buildings).
- **An ally's visit** (E10's teams; `TWN-STUB-11`): the entering step tells the visiting player
  the town's buildings once (the original's town screen at the visit); until E10 no town is an
  ally's.
- **Events:** `TownBuilt { player, town, building, event: Option<u16> }` (a command's build or a
  town event's, to the owner alone); `TownEventFired { player, town, event }` (the owner);
  **`TownChanged`** grows from the Capitol alone to the town's public facts (`fort`, `capitol`,
  `built_today`) for `seen_by`, and `owner_facts` (hall, income) for the viewers at the top level
  (`top: Viewers`, the projection drops it for the others): told after a build or an event's
  build when a fact a viewer sees changed, and after a capture's Capitol limit as now
  (*agent's call*: one event for the town's seen facts). A new Tavern raises its owner's view level
  of every other town (TWN-VEW-002), told by its `TownBuilt` (the told-views property counts it).
- **Never:** another player's town's buildings, the boat spot (increment 4 places its view as
  ADV-SHY's, the owner's team), the legal builds of another player.
- **The words for `docs/06-security.md`** (its "Towns" section, the round's `shared/` pull
  request): the day's mark public; a town's buildings its owner's, an ally's at its visit (E10);
  the build and the town events told to the owner; the public facts' changes to the town's viewers.
- **As built (T18b2a, 2026-11-10; decisions 52 to 54, SRV-Q26):** `TownView::built_today` and
  `TownView::buildings: Option<TownBuildingsView { built, forbidden }>` (the owner's alone,
  `BuildingSet`s; `h3-protocol`'s `Town::built_today`, `buildings: Option<TownBuildings>` with
  the sets as `WireU64`; no protocol step: no message carries the game's observation before E16 M5
  T3). **Every viewer's changed view told to it** (decision 53, in place of the one `TownChanged`
  for the seen facts above and decision 51's call): `town::view::Told` takes each viewer's views
  of the touched towns before a change (`Told::before`, only for a sink that keeps events,
  `Sink::RECORDS`; the env, the replays and the benches keep none) and tells each changed one
  after it: `TownChanged { player, town, owner, fort, capitol, built_today, level, slots, hall,
  buildings }`, then a `TownStackSeen { stack, slot }` per stack the level shows (the slot to the
  owner's team) and a `TownIncomeSeen { resource, amount }` per income line of the top level, all
  `player`'s alone (`GameEvent` stays `Copy`, as E08's `StackChanged`). The changes: a build
  (after `TownBuilt`; a town without a standing Tavern before reads its owner's views of every
  town, whose levels a new Tavern raises), a capture (after `TownCaptured`, `TilesRevealed`;
  owner, mark, Capitol's limit, emptied garrison; with a Tavern, the old and the new owner's views
  of every town), a siege's merge (after its `StackChanged`), a garrison's battle (after its
  `BattleEnded`, before the outcome's settling), the day's start (`town::start_day`, the marks
  cleared, before the income). Not told: Visions' reach (`TWN-STUB-13`, decision 54) and a town
  that comes into view by exploring (SRV-Q26's query). One per-town view (`Seer::view`) serves
  `observe` and the told changes. The plans' told-views property folds them strictly (every other
  view as it was, Visions' views but their public facts exempt).

### 32. The env, fixtures, replays, properties, benchmarks, the census

- **The env** (`h3-sim`, the AI lane's, `cross-lane`, decision 7): an entry **build** with the
  tile head the town's entrance (as `BuildBoat`'s anchor) and a **building head** over
  `MAX_BUILDINGS` (64) indexes of the town's record, masked by `legal_commands`; the encoding's
  version and layout are the AI lane's (E08's increment 11 plans v8 first). The rollouts' random
  choice takes builds; a test plays the town map to its end of days with builds.
- **The writer** (`h3-map` `src/writer/town*`, the town lane's): a town's events (today `0`
  written), so fixtures and batch maps carry them.
- **Fixtures** (the adventure lane's paths, `cross-lane`): `e09-towns-36.h3m` gains a Castle town
  by water and one not (the Shipyard), a Conflux town (the Altars, TD-9), town events (a day-1
  build, a weekly and an every-day gold, a creature event, a Capitol event in a player's second
  town: T-8), a custom set forbidding a requirement.
- **Golden replays** (`fixtures/replays/adventure/towns-*`): a week of builds in one town (the
  chain, the mark, every refusal kind once), the Capitol refused while another stands, a capture
  of a town built in today, the town events' day-1 build and gold; random play with builds.
  Every replay re-recorded at the bumps (skill `homm3-rerecord-replays`).
- **Properties** (`h3-map`'s plans, the adventure lane's files: the town lane's `TRAPS.md`): a
  step `Build` (a random legal build of the player); the legal builds are exactly those `apply`
  accepts; a refused build changes no state hash; the told views and the hygiene property with
  builds (the mark is public, the buildings the owner's); the day's start clears every mark.
- **Benchmarks** (`benches/adventure/`): `legal_commands` on the town map (the builds' sets, each
  owned town a `buildable` and a treasury test), `day-start-towns-36` (the marks), a turn start
  with the fixture's events; compared with `master` (skill `homm3-bench-regression`).
- **The census** (`--with-original-data`; exit criterion 6): the 160 maps import and build (§28's
  Shipyards counted), then each played some days by `EndTurn` alone: the town events fired, built,
  skipped as standing, against survey §10 (57 built, 30 skipped, 27 without a requirement in the
  fixed towns).
- **As cut and built** (T19a, decisions 58, 59): in three parts, T19a the writer and the fixtures'
  part, T19b the properties, T19c the env, random play, the benchmarks and the census. The town
  map stays as it is: the towns and events above are increment 2's own map,
  `fixtures/maps/e09-builds-36.h3m` (`h3-map`'s `tests/builds_map.rs`), its replays `builds-*`. A
  replay holds no refused command, so every refusal kind and the Capitol refused while another
  stands are the map's test (the state hash unchanged after each), the replays the accepted
  builds: a week of them, a capture of a town built in today, the town events to day 8.
  T19c cut in two (decision 62): T19c1 the env (action encoding v9: the entry `BUILD`, 30, the
  town's entrance in the tile head, a building head of 64 masked by the listed builds of that
  town, 0 alone after every other entry; `h3_sim::adventure::action::building_mask`), the
  rollout drawing each listed build as a command, the town map's env test on new seeds, random
  replays with builds on the builds map (`builds-s262-d14`, `-s271-d14`, `-s290-d14`), the town
  map's three moved workloads re-baselined; T19c2 the new workloads and the census, kept whole
  (decision 63): the bench's workload kind `legal` (`h3_game::legal_commands` before each step of
  a player's seat in the map's phase, inside `legal_queries`) as `legal-towns-36` on the town
  map, `day-start-builds-36` the builds map's day-8 start (the week, the Harbour's every-day and
  weekly gifts), `day-start-towns-36` unchanged (since T19c1 its day starts clear the marks);
  the census in `h3-map`'s `original_data` (every game played by `EndTurn` alone, the town
  events' firings and builds checked against the census's own walk, the survey's order checked
  against §10's 57, 30 and 27: survey §10, "Checked by play").

### 33. Batch TWN-2 (decisions 41, 42)

Two rows, run as batch TWN-1 (design §20) on the client lane's harness (`clients/capture/`) in SoD,
hot seat, from maps `h3-experiment` writes (`experiments/specs/E09-batch-2a.toml`):

| Row | Setup | Observed | The reading predicts |
|---|---|---|---|
| 1 (TD-9) | a Conflux town of Red's, a custom set with the Altar of Fire (33) and its requirements, not the upgraded Altar of Air (38); the treasury full; a control town with 38 built | the hall's upgraded Altar of Fire (40): its dialog's Build button and text, Build pressed, the town's ever-built set (+0x158) read | refused without 38 (the text names the upgraded Altar of Air); built in the control |
| 2 (TD-11) | Red and Blue humans on one team (the writer's `teams`), Blue's town, Red's hero beside it; both treasuries known | Red's hero enters Blue's town; the town screen's hall: a building's Build pressed; the town's +0x158, +0x2, both treasuries (+0x9C) read | built, paid from Red's treasury (no owner test on the hall's path, §1.1, §1.4); VCMI refuses |

- **Verdicts:** row 1 overrides decision 41's list if the original builds 40 without 38 (a
  gap-fill override, §25); row 2 revisits decision 42 if the original builds (with E10's teams the
  command would then let an ally's visitor build at its own cost: a question to the user).
- **The harness:** the town reads exist (TWN-1); new: the hall's click path (the hall building on
  the town screen, a slot, Build), a treasury read; the writer's teams exist (E08 T89a).
- **Tasks** (§35's round 2): its maps and spec, the harness steps and the run, the scoring; cut at
  its start.
- **Scored** (T14c, 2026-11-07; survey `E09-construction.md` §13): row 1 as the reading (40
  refused without 38, the text naming 38; built in the control): decision 41's list stands, no
  override. Row 2 **refused**: the hall opens for the ally's visitor and shows the slot buildable,
  but the build dialog's constructor greys Build when the town's owner is not the current player
  (0x5D5AE2, which T12a did not read): decision 42 is the original's rule, §29 step 2 as written,
  nothing widens with E10's teams. Not predicted: the town screen builds a display piece into the
  standing set (Castle 23, Stronghold 27, Fortress 23, Conflux 22; outside every faction's list),
  left out of the part as decision 27's companions are; nothing to build.

### 34. Versions, rule ids, quirks, stubs

| Version | Change | Task |
|---|---|---|
| town ruleset part v4 | requirements, costs, the role `shipyard` | T16a |
| town state hash v2 | the day's mark | T17a |
| game state hash v26 (E08 T105b's then v27) | the store's v2 folded in | T17a |
| adventure ruleset part (v19 after E08 T103a's v18) | the kind `Town`'s boat offsets | T18a |
| adventure replay format v11 (before E08 T105b, whose bump is then v12) | `GameCommand::Town` | T18b1 |
| env action encoding (the AI lane's; after E08's v8) | the build entry and head | T19 |
| map import | none (the import reads the town events and the forbidden sets since T7a1) | — |

**Rule ids** (`docs/rules/catalog/town.toml`): TWN-BLD-001 the build command (the owner, the turn,
the price paid; the refusals), -002 buildable now (requirements in the ever-built set, the
forbidden, the Grail's never by the hall, one Capitol), -003 the hall's offer (a horde as its
upgrade), -004 one a town a day (the mark: set by a build, cleared each day, a capture's rule,
never by an event), -005 what a build does (the ever-built set, the automatic, increment 1's reads
at once), -006 the costs (`Building.txt`), -007 the requirements (the executable's lists, closed;
TD-9); TWN-SET-011 the Shipyard's water at the start; TWN-EVT-001 when a town event fires (the
owner's turn start after the map's events, the map's order, the day test, TD-12), -002 its
resources, -003 its buildings (T-8), -004 its creatures (`TWN-STUB-10`); TWN-VEW-004 the
buildings' and the mark's views. Amended: GAM-CAL-002 (the day's start), GAM-CAL-004 (the town
events after the map's), TWN-CAP-001 (the capture's mark), TWN-VEW-001 (the mark public),
ADV-SHY-001 (the towns' spots in the same step).

**Quirks** (`docs/rules/quirks-town.md`): **T-9** (new, this note; survey §8.3): an event's bit
loop runs over 44 bits of a table of 41 a faction, so bits 41 to 43 read the next faction's row
(past the table for the Conflux): **fixed without a flag** (the engine maps the 41 slots only; no
shipped event sets them; a read past a table is never reproduced): `TWN-QUIRK-7` with the build.
T-8 is no quirk (decision 44): the rule's summary says so.

**Stubs:** `TWN-STUB-2` narrowed to the Grail building (increment 5, E10's victory conditions) by
TWN-SET-011; `TWN-STUB-10` the town events' creatures (increment 3); `TWN-STUB-11` an ally's
visitor learning the town's buildings (E10's teams); `TWN-STUB-12` the Lookout Tower's and the
Skyship's reveals at their build (increment 4, `town_build_effects` step 7).

### 35. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T15 (`shared/`: the drafts, `docs/03`, `docs/06`); the harvest (T16b,
`h3-harvest`); batch TWN-2's spec (`h3-experiment`, T14); the static map's spot and the part's
offsets (T18a, the adventure lane's); `GameCommand`, the turn start, the day, the versions (T17,
T18); the fixtures, replays, properties and the env (T19, the adventure and AI lanes' paths). Each
`cross-lane` pull request merges in order with the adventure lane's (E08 T102 to T105 bump the same
versions; the second to merge bumps again and re-records).

**The building tasks**, in rounds (ids continue the epic's; each task's cut confirmed at its start;
batch TWN-2 runs first, so the ADRs are applied once with its verdicts, as increment 1's TWN-1):

| Round | Tasks | Content |
|---|---|---|
| 2 | **T14** | Batch TWN-2 (§33): the maps and spec, the harness's steps, the run, the scoring (cut at its start) |
| 3 | **T15** | `shared/`: the three drafts into `docs/adr/` with TWN-2's verdicts, `docs/03-domain.md`'s terms, `docs/06-security.md`'s words |
| 4 | **T16a** + **T16b** | T16a: the part's format, compiler, adapter and hash v4 (§24, the synthetic packs). T16b: the harvest (§25; original data) |
| 5 | **T17a** + **T17b** | `h3-towns`, cut at its start (the user, 2026-11-09). T17a: the mark and the store's writes with `built_with` (§26, hash v2), the game state hash's bump (`cross-lane`). T17b: the rules (§27), `TownCommand`, `town.toml`'s other TWN-BLD and TWN-EVT-003, `TWN-QUIRK-7` |
| 6 | **T18a** + **T18b** + **T18c** | T18a: the Shipyard's water (§28; the adventure part's offsets, the census). T18b: the command, the legal builds, the day's start, the capture's mark, the views and events (§29 to §31 but the town events; replay format). T18c: the town events at the turn start (§30, TWN-EVT-001 to -004) |
| 7 | **T19** | The writer's events, fixtures, replays, properties, benchmarks, the env (§32; `cross-lane`), the census with the original data |
| 8 | **T20** | The audit against the exit criteria; increment 2 closed; increment 3 refined |

**Increment 2's exit criteria** (for the epic at the checkpoint):

1. A town's owner, in its turn, builds a building of its town's faction that the original's hall
   offers it (its requirements in the ever-built set by the executable's lists, not forbidden, the
   Shipyard only with water, one Capitol a player, the hordes as their upgrades), one a town a
   day, paid from its treasury at `Building.txt`'s costs; every other build is refused, typed,
   changing nothing (TWN-BLD-001 to -007, TWN-SET-011).
2. A build changes increment 1's reads at once: the hall's income, the fort and the siege's facts,
   the Tavern's view levels, the silo, the Capitol (tests per role).
3. The map's town events fire as the original's: their days, players and kinds, at their owner's
   turn start after the map's timed events, in the map's order; their resources, then their
   buildings without the hall's tests; their creatures registered for increment 3 (TWN-EVT-001
   to -004; T-8 and TD-12 as decided).
4. The views are the original's: the owner sees its buildings, everyone who sees the town its
   day's mark, fort and Capitol; nothing else leaks (the hygiene and told-views properties with
   builds; `docs/06-security.md`).
5. The env plays the build; the plans build; golden replays cover a week of builds, each refusal,
   a capture after a build and the town events; determinism and the forks' sharing hold (a build
   copies one town).
6. With the original data all 160 shipped maps import, build and play their town events over
   their first days; the census (the Shipyards removed for want of water, the events fired, built
   and skipped) is recorded against the survey.
7. Batch TWN-2 is scored and its verdicts applied (TD-9, TD-11).
8. The gates hold (100 % coverage, the scoped verify, CI's full verify); the versions are bumped and
   recorded; the other lanes' inboxes are told (the AI lane's encoding, the map client's town
   screen, the server's manifest of the part's hash).

### 36. Risks and open points

- **Two lanes in the same crates:** E08's increment 11 (T102 `shared/`, T103a part v18, T105b game
  state hash v27 since T17a took v26, replay format v11, action encoding v8) bumps the same
  versions; whichever merges second rebases, bumps again and re-records. Check the adventure
  lane's open pull requests at every task's start.
- **`legal_commands`' cost:** every owned town's offer is computed for each list (the env calls it
  every step); measured at T18b2b (§29's as built: 0.65 % of random play on the town map), so no
  cached offer.
- **The hall's offer** is derived from `auto_with` (§27); batch TWN-2 does not observe it (the
  same cost and end state in every faction).
- **Not read:** the Shipyard's spot against the random dwellings' resolution (§28); the town
  events' messages for a computer owner (none shown: no rule).
- **TD-11:** settled by batch TWN-2 (§33): the original refuses an ally's visitor too; the owner
  test does not widen.
- **The creatures of 56 shipped town events** are lost until increment 3 (`TWN-STUB-10`).
