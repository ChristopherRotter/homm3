# E09 design note

Increment 1's note (closed 2026-11-06 by T10; Parts A to C, §1 to §22, kept here as the code cites
them: the epic's decision 40). Increment 2's note (construction, decision 37) is Part D, §23 to
§36, written by T13 (2026-11-07), kept at increment 2's closure (T20, 2026-11-11) as Parts A to C
were. Increment 3's note (growth, recruitment and the army rows, decision 65) is Part E, §37 to
§58, written by T22a and T22b (2026-11-11), kept at increment 3's closure (T30, 2026-11-15). Increment
4's note (the Tavern, decision 102) is Part F from §59: the hire §59 to §71 by T33a (2026-11-16),
the information, the gates and the cut from §72 by T33b. Increment 5's note (the mage guild,
decision 162) is Part G, §82 to §100, written by T43 (2026-11-20).

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

## Part E — increment 3: growth, recruitment and the army rows

### 37. Scope, sources and choices

Increment 3's note (the epic's decisions 65 and 66). A handover, not binding, as Parts A to D:
the working detail behind increment 3's amendments of ADR-0028, ADR-0017, ADR-0016 and ADR-0013
(drafted here, moved into `docs/adr/` by T24 on 2026-11-12), written from the survey
`docs/rules/surveys/E09-growth.md` (§ numbers are the survey's unless they name this note), the
survey's checkpoint (`TWN-Q12`, decisions 69 to 73) and `TWN-Q13` (decisions 74 to 77). Written
in two halves (decision 74): **T22a, growth** (§37 to §47, this text) and **T22b, recruitment and
the two rows** (from §48: the hire through E08's dialog, the Portal's hire, the army reference
agreed with E08's increment 12, a garrison hero and the heroes' moves, T-5), which also writes the
building tasks' cut, the exit criteria and the checkpoint (`TWN-Q14`). Batch TWN-3 is its own task
after the checkpoint (T23). *Agent's call (change freely)* marks a choice of this note the user may
still change at `TWN-Q14` (decision 78 lists them).

- **Growth builds:** the town's available creatures and its Portal in the store (§40); a
  dwelling's growth and its inputs (§41); the week's kind and the week's growth (§42); the month
  (§43); a neutral town's garrison week (§44); the Portal's draws (§45); a dwelling's first
  creatures and the town events' creatures (§46, `TWN-STUB-10` lifted); the part's roles and
  parameters (§38); the core's hero effects, quirk flags and stream (§39); the day's order, the
  views, the versions and the gates (§47).
- **Not growth's:** buying the creatures (T22b's half: the hire takes the counts this half keeps);
  the growth panel's figures (§7, a display: the client computes them from the views); the
  computer's recruitment and its use of the Portal (E13); the Necropolis' and Conflux's Grail
  extras (increment 5); the RoE and AB rules of growth (E11).
- **Sources and disputes:** the reading for every rule; TD-13 and TD-14 the reading with a
  `fix-optional` flag each (T-13, T-12; decisions 69, 70); TD-15 and TD-16 the reading (72, 73);
  T-10 fixed (a count saturates, 73); T-11 drawn at the build and at a dwelling flagged (71).

### 38. The town part: the roles `horde`, `portal`, `week_of`, a record's `roe`, the parameters (`h3-towns`, `h3-pack-towns`; part v5)

- **The format** (`h3-pack-towns`' format doc): a building's `horde = [level, upgraded]` (as
  `dwelling`: the level within the record's, the upgraded flag needing the level's upgrade);
  `portal = true`; `week_of = level` (within the record's levels). A record's `roe = false` (absent:
  true). `[town_params]` gains `week_creature_chance = [1, 4]` (a creature's week: 1 in 4),
  `week_creature_bonus = 5` and `garrison_week = { odds = [40, 80], out_of = 101, weights = [2, 3,
  4, 5, 4, 3], weeks = 8, upgrade = 5 }` (the odds without and with a standing fort; the weights
  `neutral_garrison_level_weights` 0x6779B0; the upgrade in 100).
- **Compiled:** `BuildingRoles::horde: Option<Horde { level, upgraded }>`, `portal: bool`,
  `week_of: Option<u8>`; `TownRecord::roe: bool`; `TownParams::week` and `::garrison_week`.
  Refused (`TownCompileError`'s variants, new ones named by the task): two hordes of one index in
  a record (no faction has two on a level, §7), a second `portal` or `week_of` in a record, a
  level outside the record's, an upgraded horde of a level without an upgrade, a chance whose
  numerator exceeds its denominator or is 0, an odds above `out_of`, empty weights. A horde's
  growth is **the core's `horde_growth`** of its index's creature (the figures
  `horde_table_init` 0x5BDF60 copies into `town_hordes`; checked equal for the twenty base
  creatures with a horde, the AB pack's Pixies and Sprites too): one fact, held once (*agent's
  call*).
- **The hash** (v5): after `shipyard`, the horde (0 none, else 1, its level, its flag), `portal`,
  `week_of` (0 none, else 1 and its level); after the record's buildings `roe`; the parameters'
  week and garrison week after Visions'. Pinned in `ruleset/tests.rs`.
- **Overrides:** none (`TownPatch` gains nothing; RoE's and AB's growth are E11's).
- **The content** (T16b's route, `h3-harvest`'s `src/towns.rs`, `cross-lane`, original data):
  `horde` from the horde table it reads already (`town_hordes` 0x6887A0, `horde_buildings`
  0x642E20: the level by the record's index, upgraded for 19 and 25); `portal` on the Dungeon's
  building 22 and `week_of = 1` on the Inferno's 26 as gap-fill roles (the harvest's role spec;
  the executable tests the ids, not data); `roe = false` on Conflux's record (the AB pack's); the
  parameters in the base pack's `[town_params]`. The report's line "the hordes' growth not
  harvested" goes.
- **The synthetic pack:** a horde and its upgrade, a Portal and a `week_of` building in its
  records, so `h3-towns`' tests and the scripted replays reach each role without original data.
- *As built (T25a, `town/e09-t25a`):* as above; `Horde { level, upgraded }`, `WeekParams {
  creature_chance, creature_bonus }`, `GarrisonWeekParams { odds, out_of, weights, weeks, upgrade }`
  (`u16` chances, `u8` weights, one to six of them); the refusals `InvalidBuilding` "a horde's
  level, or its upgrade, is not the faction's" and "a week's level is not the faction's",
  `InvalidTown` "has a horde of one index twice" and "has more than one Portal of Summoning or
  week's building", four `InvalidParams`; part hash v5 pinned (`0x84b4_0449_24d2_2388`). The
  harvest also refuses a horde record whose growth is not its creature's `horde_growth` (all
  eleven hordes match) and a gap-fill `horde`; `vale`'s upgraded horde has no role (no upgrade
  in its levels), its Lookout is the Portal and its Grail the week's building.

### 39. The core: two hero effects, two quirk flags, the towns' stream (`shared/` and `cross-lane`)

- **Hero effects** (decision 76; ADR-0013 point 16.4's draft; `h3-content`'s `effect.rs`,
  `shared/`, pack format v52): `HeroEffect::TownGrowth { level: u8 (1..=7), amount: u16 (≥ 1) }`
  (file `{ kind = "town_growth", level = 2, amount = 5 }`) and `HeroEffect::OwnerTownGrowth { bp:
  u16 (1..=10 000) }` (`{ kind = "owner_town_growth", bp = 5000 }`), codes after
  `WhirlpoolProtection`'s 60. The base pack's five pieces (`legs_of_legion` +5 at level 2 to
  `head_of_legion` +1 at level 6) and the SoD pack's `statue_of_legion` (5 000) get them, their
  "pending" comments dropped (the packs are the battle lane's: `cross-lane`).
- **The derivations** (`h3-heroes`, the adventure lane's: `cross-lane`): `town_growth(details,
  core) -> [u16; 7]` by level and `owner_town_growth(details, core) -> u16`, each worn type once,
  a worn combination's parts counting (`hero_wears_artifact` 0x4D91F0; as `WhirlpoolProtection`
  reads the Sea Captain's Hat through the Admiral's): a worn Statue gives its share and the five
  pieces' amounts.
- **Quirk flags** (`h3-content`'s `quirks.rs`, `shared/`; the profiles in the base pack's
  `core.toml`, `cross-lane`): `quirk.town.creature_month_exact` (T-13, `QuirkFlag(39)`) and
  `quirk.town.neutral_plague_full_growth` (T-12, `QuirkFlag(40)`), each on in `faithful` and off
  in `strict-fixed` (the class `fix-optional`, decisions 69, 70).
- **The stream** (decision 75; `h3-kernel`, `shared/`): `StreamId::TOWNS = Self(7)`, "the towns'
  weekly randomness" in the closed list. `GameState` holds its `Rng` beside the game's (§42).
- *As built (T24, `shared/e09-t24`):* as above; `h3_content::TOWN_GROWTH_LEVELS` (7) bounds
  `level`, two `town_growth` of one list collide only at one level (as `daily_resource`'s by
  resource), core ruleset hash v53 (golden base `0x9649_783d_4639_ded0`); the synthetic packs'
  profiles name neither flag (off, as the adventure's flags), the `synthetic-ext` Storm Regalia
  carries both kinds (the every-kind check; a combination's own, as the Statue's).

### 40. The record and the store: the available creatures, the Portal (`h3-towns`; town state hash v3)

- **The record:** `Town` gains `available: [u16; INDEXES]` (`INDEXES` = 2 ×
  `MAX_CREATURE_LEVELS` = 14: index `level − 1` the level's creature, `level − 1 + 7` its
  upgrade; the original's +0x16) and `portal: Option<Summoned { creature, count: u16 }>` (+0x3C,
  +0x40). `MAX_AVAILABLE = 32 767` bounds every count (**T-10 fixed**, planned `TWN-QUIRK-8`, no
  flag: the original's i16 adds and doubles wrap past it; below it nothing differs). Accessors
  `available(index)`, `portal()`.
- **The store's writes** (as T17a's): `set_available`, `set_portal`, and the rules' whole-town
  results (§42 to §46) through one `write(town, Town)`; a fork's first write copies that town.
- **The hash** (v3): after the day's mark, the fourteen counts (`u16` each) and the Portal (0
  none, else 1, the creature's handle, the count). Pinned in the store's tests.
- **At the build** (`h3-towns`' setup): each level whose dwelling is ever built gets its
  creature's growth at the index of its standing dwelling (the upgrade's when the upgraded
  dwelling is ever built, else the base's): what the original's `town_build` in id order leaves
  (§6; VCMI's first day agrees). A Portal is drawn for an owned town only (§45); a neutral town's
  (one shipped map's) stays empty until owned.
- **A capture** keeps the counts and the Portal (`town_change_owner` writes neither).
- *As built (T25a):* `CREATURE_INDEXES` (14) and `MAX_AVAILABLE` at the crate's root,
  `Summoned`, `Town::available`, `available_all`, `portal`, `with_available`, `with_portal`
  (each saturating); the store's `set_available`, `set_portal` and `write(town, Town)` (its
  closure write now `update`); the setup's first creatures through §46's rule over the
  ever-built set (`town_setup` takes the core); no Portal drawn yet (§45 is T25b's). Store hash
  v3 pinned (`0xFA14_A109_A1EC_2745`), game state hash v28 (§47's table: the week's kind is then
  v29).

### 41. A dwelling's growth (TWN-GRO-001; `h3-towns`' `growth.rs`, the inputs from `h3-game`'s `src/town*`)

- **The rule** `growth(town, part, core, index, &GrowthInputs) -> u16`, in §1's order, in `u32`
  then capped at `MAX_AVAILABLE`: none without the index's dwelling ever built (a building of role
  `dwelling` with the index's level and flag), none at a base index whose upgraded dwelling is
  ever built; g the index's creature's core `growth`; + g with a standing fort level 3, + g ÷ 2
  with level 2; for an owned town + (g + that) × `share_bp` ÷ 10 000 (truncated: the Statue's
  5 000 is the original's ÷ 2) + `legion[level]`; + the first building in index order ever built
  whose `horde` is the index (its creature's `horde_growth`); + `dwellings[index]`; then with a
  `grail` building ever built + the sum ÷ 2.
- **The inputs** (`GrowthInputs { owned, share_bp, legion: [u16; 7], dwellings: [u16; 14] }`),
  computed by the game once per town and step: `owned` the adventure's owner; `share_bp` the
  highest `owner_town_growth` of the owner's heroes on the map and in its towns' garrisons
  (`player_hero_wears_artifact` 0x4BACB0; garrison heroes come with T22b's half); `legion` the
  visitor's `town_growth` plus the garrison hero's (each hero once, §1's "each hero apart");
  `dwellings` per index the owner's dwellings whose kind's **first** creature is that index's (the
  original's record +4), a base creature counting at its upgrade's index too. A neutral town's
  inputs are all zero. *Agent's call:* the dwellings' table built once per player per step (a
  `Vec` by creature handle), not per town.
- **Not modelled:** the original's stored counts +0x118 (derived here from the owners: one fact
  once); the growth panel (§7).
- *As built (T25a):* `h3_towns::growth` with `GrowthInputs` (`NEUTRAL` its default) and
  `index_creature`, summed in `u64` (a share past 10 000 overflows nothing); batch TWN-3's row 2
  (66 and 48) is a unit test on the fixture.
- *As built (T26a1):* `h3_heroes::town_growth` (by level) and `owner_town_growth` (the highest
  share) in `h3-heroes`' `day.rs`, over the hero's sources with each worn type once; `h3-game`'s
  `town/growth.rs` `inputs`, by `TownId`, once a week before the towns' week and kept for the
  month of the same day: the owner the adventure's, a per-player table built at its first town
  (the share over its heroes on the map; its dwellings' first creatures, a `Vec` by creature
  handle), the visitor's Legion, a base dwelling counting at the upgrade's index only where the
  level has an upgrade. The garrison hero's share and Legion: T28b.

### 42. The week (TWN-GRO-002, -003; GAM-CAL-002 amended; `h3-game`'s `src/town/week.rs`, `state.rs` and `state/day.rs` `cross-lane`)

- **The kind** `h3_towns::WeekKind` (*agent's call*: the town crate's type, read by its rules,
  re-exported by `h3-game`; the value in `GameState` beside `MonthKind`, decision 77): `Normal`,
  `Creature(CreatureTypeId)`, `Of { creatures: [Option<CreatureTypeId>; 2], bonus: u16 }` (a
  `week_of` level's base and upgrade, the base's core growth as the bonus: the Imps' 15). Hash 0,
  1 and the handle, 2 and both and the bonus. The game's first week is `Normal`.
- **The draw** (TWN-GRO-002), on a week's first day before `WeekStarted`, from the towns' stream:
  unless the day is a month's first, `below(4)` (the part's chance) and on 0 a creature `below(n)`
  of the eligible list (every town record's levels' creatures, base then upgrade, records in the
  part's order; on an RoE map only the records with `roe`; the original's §2 set, checked: 126
  creatures with AB, 112 without Conflux); then the first town in `TownId` order with a standing
  `week_of` building, of any owner, makes it `Of` its level's, the draw kept or not (the
  original's order: drawn, then overwritten, every week the month's first too). The setup's
  **`roe`** flag (`GameSetup`, the import's `Version::RoE`; map import v22, `cross-lane`) is read
  only here.
- **`GameEvent::WeekStarted { week, kind }`** and `GameObservation::week` (public, as the
  month's), `cross-lane` with the protocol's mapping (the global trap: the same pull request).
- **The towns' week** (TWN-GRO-003; `h3_towns::week_growth`): in `TownId` order, each index whose
  growth is above 0 gains growth + the kind's bonus when its creature is one of the kind's
  (`Creature`: that creature alone, 5; `Of`: either form, the bonus), halved (truncated, after the
  bonus) in a town without an owner, added saturating.
- **Then** the adventure's week step as before (the game's stream; the map's dwellings read no
  week's kind, ADV-DWL-003: the inbox's E08 T97a note asks the building task to extend
  `a_month_s_kind_never_reaches_a_dwelling_or_a_camp` with a week of the hut's creature), then
  §44, then §45.
- *As built (T25b1):* `h3_towns::WeekKind` with `names` and `bonus` (a creature's week the
  part's `creature_bonus`) and its hash; `week_growth(town, part, core, &kind, &inputs) -> Town`.
  The draw (TWN-GRO-002) is T26a2's (decision 89).
- *As built (T26a1):* the towns' week (`town::growth::grow_week`) after `WeekStarted` and before
  the adventure's week step, each town written only where it changes; every week `Normal` until
  T26a2 draws the kind.
- *As built (T26a2):* `h3_towns::draw_week(towns, part, core, roe, month_first, draws)` with
  `week_creatures(part, roe)` and `building_week(town, part, core)`; the chance's draw
  `below(denominator) < numerator`, then the creature's `below(n)`, no name index; a part without
  parameters or without an eligible creature draws no creature. The game state holds the week
  (`GameState::week`), `roe` and the towns' stream (`StreamId::TOWNS`; game state hash v30 after E08 T114a2's v29, both
  after the month's kind, the stream after the game's); `GameSetup::roe` from the import's format
  (map import v23); `WeekStarted { week, kind }`, `GameObservation::week`, the wire's `Week` and
  the fold (protocol v53; the server lane's E16 M5 T11 and T12 and E08 T114a2 took v50 to v52). `h3-game` re-exports `WeekKind`.

### 43. The month (TWN-GRO-004; GAM-CAL-003 amended)

- **The kind** as GAM-CAL-003 draws it (the game's stream), but when the week's kind is `Of`
  (the month's first week, so a `week_of` building's) it becomes `MonthKind::Creature(that base
  creature)` without the creature's draw (`new_month` 0x4C8F5E draws, 0x4C8F63 to 0x4C8F82
  overwrites, no second draw): `MonthStarted` names it and the adventure's month step spawns that
  creature on the map (the original's monsters of the month).
- **The towns' month** (`h3_towns::month`), in `TownId` order after `MonthStarted` and before the
  adventure's month step, each index whose growth (§41, the same inputs) is above 0:
  - a `Creature(c)` month, unless the week is `Of`: the count doubles (saturating) when the
    index's creature is c; with `quirk.town.creature_month_exact` off, when c is either of the
    level's creatures (T-13, planned `TWN-QUIRK-10`, VCMI 1.7.5's rule);
  - the Plague: the count loses the growth (at least 0), then halves (truncated); with
    `quirk.town.neutral_plague_full_growth` off, a neutral town loses half the growth (what its
    week added), so every town keeps half of its count before the week (T-12, planned
    `TWN-QUIRK-9`).
  The fifteenth index of the original (building 44's bit, never set) is nothing. The Portal and
  the garrisons are untouched.
- *As built (T25b1):* `h3_towns::month(town, part, core, MonthGrowth, &week, &inputs) -> Town`
  with the town crate's `MonthGrowth { Normal, Creature, Plague }` (the game's `MonthKind` stays
  the game's and is mapped to it, T26a); the flags read from the core's quirks; batch TWN-3's
  rows 1 and 3 are unit tests on the fixture (its growths 14 and 9 are the rows' towns').
- *As built (T26a1):* `town::growth::grow_month` after `MonthStarted` and before the adventure's
  month step, `MonthKind` mapped one to one (a normal month writes nothing), the week's inputs
  reused; `advance_day`'s month block sits inside its week block. The month after a `week_of`
  week (GAM-CAL-003 amended) is T26a2's.
- *As built (T26a2):* `draw_month` reads the week: after a building's week with a base creature
  the month is `MonthKind::Creature(base)`, the kind's draw made (when the part has the months'
  parameters) and the creature's not; `grow_week` and `grow_month` pass `GameState::week`.

### 44. A neutral town's garrison week (TWN-GRO-006; `h3-towns`' `garrison.rs`, `h3-adventure`'s `src/town.rs`)

- **The rule** `garrison_week(town, part, core, weeks, garrison: &[ArmyStack], draws) ->
  Option<GarrisonWeek { stacks, town }>`, for each town without an owner in `TownId` order after
  the adventure's week step, from the towns' stream:
  1. `below(out_of)` under the odds (80 with a standing fort level, else 40, in 101); else none.
  2. n = min(`weeks`, the parameters' 8), `weeks` the weeks since the start, (day − 1) ÷ week
     length (the original's days ÷ 7 on the week's first day); S the sum of three
     `range_inclusive(0, n + 1)`; the level the first weight S does not exceed, S less each
     passed, else level 7.
  3. The creature: the level's base, its upgrade when a garrison stack holds the upgrade; the
     count its core growth.
  4. Room: a stack of the creature or an empty slot (seven); else the stack of least AI value
     (core `ai_value`) × count below the new stack's, the first of equals in slot order, cleared;
     else none (nothing lowered, no further draw). Then merged into the creature's stack or put in
     the first empty slot.
  5. The level's two counts each lose the count (at least 0).
  6. For a base creature, `range_inclusive(1, 100)` at most 5: every stack of it becomes the
     upgrade (a level without one: unchanged).
- **The game** writes the stacks with `AdventureState::set_town_garrison` (a writer in the town
  lane's `src/town.rs`; the armies on objects, ADR-0016 point 1's draft) and the town into the
  store, in one application. An owned town's garrison never grows.
- **Views:** the garrison's change is told to every viewer whose view of the town moves
  (TWN-VEW-002's levels), with the towns' day's `TownChanged`s (§47).
- TD-15 by the reading (decision 72); batch TWN-3's row TWN-3-4 observes the distribution.
- *As built (T25b2):* `h3_towns::garrison_week(town, part, core, weeks, garrison, rules, draws)
  -> Option<GarrisonWeek { stacks, town }>` (`garrison.rs`), `rules` the adventure part's army
  slots and a stack's most (the room is ADV-ARM-007's), the stacks returned in slot order; a level
  the record lacks, a creature growing none and a growth above a stack's most give none after the
  sums; row TWN-3-4's room cases are unit tests (decision 88).
- *As built (T26b):* `h3-game`'s `town::garrison::garrison_weeks(state, rules, weeks)` after the
  adventure's week step, `weeks` the day less one over the week's days; the state's
  `town_draws(Draws::Towns)` gives the adventure, the store and the towns' stream at once;
  `AdventureState::set_town_garrison(town, stacks)` (the town lane's `src/town.rs`). Told with the
  towns' day: the day takes every viewer's views of every town before the week's growth (a
  recording sink only) and `town::start_day` tells the changed ones after the marks.

### 45. The Portal of Summoning (TWN-GRO-007)

- **The draw** `h3_towns::summon(town, part, core, offer: &[&[CreatureTypeId]], draws) -> Town`:
  `offer` the owner's dwellings' creature lists in object order (`StaticMap::dwellings`, never a
  camp); none: the Portal empty; else a dwelling `below(n)`, then one of its creatures
  `below(k)`: `Summoned { creature, count: its core growth }`, set, not added; a dwelling without
  creatures leaves the Portal as it was. No castle, horde, Grail, week, month or Plague touches
  it (TD-16).
- **When** (decision 71, ADR-0017 point 4's draft), each only for a town with an owner and a
  standing `portal` building:
  - each week, after §44, every such town in `TownId` order (the towns' stream);
  - at its build: a command's (`town::build`, after the checks and the payment, the builder's
    stream), a town event's (the towns' stream), the setup's (the setup stream, after every
    other placeholder);
  - when its owner flags a dwelling while it has none: after the visit's outcome names the flag,
    each of the visitor's player's towns with an empty Portal in `TownId` order (the visitor's
    stream, after the visit's draws).
- **At a capture** that finds a standing Portal empty (decision 82, `TWN-Q14`; §51): drawn from
  the capturer's stream after the capture's own draws (the original keeps +0x3C and a human's
  capture opens the screen, which draws at once). **Not drawn** in a neutral town (the original's
  week skips it).
- **Its hire** is T22b's half (into the garrison army, survey §5).
- *As built (T25b2):* `h3_towns::summon(town, part, core, offer, draws) -> Town` (`portal.rs`):
  a town without a standing `portal` building unchanged with no draw, an empty `offer` empties
  the Portal with no draw; the owner and the streams the game's (T26b).
- *As built (T26b):* `h3-game`'s `town/portal.rs` (`Draws::{Towns, Player}`; `weekly`,
  `at_setup`, `at_build`, `at_capture`, `on_flag`), the offer `town::growth::owned_dwellings`
  (the growth's inputs read their first creatures); `h3_towns::has_portal`. A build draws when it
  makes the Portal stand (the store's build in `build_told`, before `TownBuilt`, so the build's
  `Told` carries it); the setup's after the levels at load; the flag by
  `AdventureOutcome::flagged_dwelling` (the adventure lane's `enter_dwelling`, set when the visit
  flags; `settle_outcome` draws right after a capture, before the rest of the outcome), each
  empty Portal of the player's towns with one `Told`; a capture's inside the capture's `Told`.
  "Holds nothing" is `Town::portal() == None` (a hire that takes every creature keeps it, T27).

### 46. A dwelling's first creatures and the town events' creatures (TWN-GRO-005, TWN-EVT-004 amended)

- **First creatures** (TWN-GRO-005): the build's result (`h3_towns::built_with`, T17a) applies,
  for each building it makes ever built with role `dwelling`, base first: a base dwelling sets
  its index's count to its creature's growth (the earlier count overwritten); an upgraded one
  sets its index's count to the base index's and the base's to 0. A horde adds nothing at once.
  The same at a command's, an event's and (§40) the setup's build.
- **The events' creatures** (TWN-EVT-004 amended; `h3_towns::event_creatures`): after the
  event's buildings, each level's count added (saturating) at the upgraded dwelling's index when
  it is ever built, else at the dwelling's, else lost. `TWN-STUB-10` lifted (`h3-game`'s
  `town/events.rs`; its test's "creatures alone" case then adds them).
- *As built (T25a):* the first creatures in `h3-towns`' `growth.rs` (crate-private), applied by
  `Towns::build` (a command's and an event's build; it takes the core) and `town_setup`; the
  events' creatures are T26a's.
- *As built (T26a1):* `h3_towns::event_creatures(town, part, &creatures) -> Town` (`growth.rs`),
  each level's count added and held at `MAX_AVAILABLE`, applied by `h3-game`'s `town/events.rs`
  after the event's buildings, written only where it changes, told by no event (the counts are
  in no view before T26b); `TWN-STUB-10` lifted.

### 47. Growth's day, views, versions, ids and gates

- **The day's order** (GAM-CAL-002 amended; ADR-0017 point 3's draft): the day + 1; on a week's
  first day the week's kind (§42), `WeekStarted`, the towns' week, the adventure's week step, the
  neutral garrisons (§44), the Portals (§45); on a month's first day the month's kind (§43),
  `MonthStarted`, the towns' month, the adventure's month step; the towns' day (the marks; the
  views told); the income; the spell points; the adventure's day.
- **Views** (TWN-VEW-004 amended; `docs/06-security.md`'s town section by the `shared/` task):
  the owner part of the town view (`TownBuildingsView`, the owner's alone) gains the fourteen
  counts and the Portal; the towns' day tells `TownChanged` to the owner of every town the week,
  the month, a build or an event wrote, and to each viewer of a neutral town whose garrison grew
  (the `Told` comparison, T18b2a's). The protocol maps them (`cross-lane`, the server lane's fold
  gate).
- **Versions** (each bumped by the first building task that needs it; the order with E08's
  increment 12 at T22b's cut):

  | Version | Change | Owner |
  |---|---|---|
  | town state hash v3 | the counts, the Portal | town |
  | town ruleset part v5 | `horde`, `portal`, `week_of`, `roe`, the week's and garrison week's parameters | town |
  | pack format v52 | `town_growth`, `owner_town_growth`; two quirk flags | `shared/` (`h3-content`) |
  | kernel streams | `StreamId::TOWNS` | `shared/` (`h3-kernel`) |
  | game state hash v28 (after E08's v27) | the store's v3 (T25a, as built); the week's kind and the towns' stream then v29 (T26a) | adventure, `cross-lane` |
  | map import v22 | the setup's `roe` | adventure, `cross-lane` |
  | protocol v42 | `WeekStarted`'s kind, the observation's week, the owner's counts and Portal | server, `cross-lane` |
  | adventure replay format, env encodings | none (no growth command; no flat adventure observation, E08 decision 187) | — |

- **Rule ids** (`town.toml`): TWN-GRO-001 a dwelling's growth, -002 the week's kind, -003 the
  week's growth, -004 the month, -005 a dwelling's first creatures, -006 a neutral town's
  garrison week, -007 the Portal of Summoning. Amended: TWN-EVT-004 (the creatures), TWN-VEW-004
  (the owner's counts), GAM-CAL-002 (the order), GAM-CAL-003 (the month after a `week_of` week).
- **Quirks** (`quirks-town.md`, registered by the tasks that build them, in this order):
  `TWN-QUIRK-8` T-10 (the counts saturate; fixed), `-9` T-12 (the neutral Plague,
  `fix-optional`), `-10` T-13 (the creature month's exact creature, `fix-optional`). **Stubs:**
  `TWN-STUB-10` lifted; none new for growth.
- **Gates:** each rule's unit tests in `h3-towns` (the synthetic pack's roles; the boundaries:
  a count at `MAX_AVAILABLE`, the Plague at 0, a full garrison with and without a weaker stack,
  the Portal without dwellings and with an empty dwelling); the game's tests (the order of the
  day, the stream's isolation: a town week leaves the game's stream where it was, a refused
  build draws nothing); properties (no count above the bound, the Plague never raises a count, a
  fork's week equals its parent's); scripted replays on the builds map (`e09-builds-36.h3m`,
  T19a's) through a month with a horde, a Grail, a neutral town and a Portal (the writer builds a
  Dungeon with one); the census of `original_data` (T19c2's walk: every shipped map to day 29,
  each town's counts checked against an independent walk of §41 to §46); the benchmark
  `day-start-builds-36` re-baselined or a week workload added (skill `homm3-bench-regression`).
  The adventure replays are re-recorded (skill `homm3-rerecord-replays`).
- *As built (T26b):* `TownBuildingsView::available` (`[u16; CREATURE_INDEXES]`) and `portal`
  (`Option<Summoned>`; both re-exported by `h3-game`), `GameEvent` still `Copy`; the wire's
  `TownBuildings` and `Summoned`, protocol **v55** (the table's "v42" was its planning number;
  E08 T114b took v54 first);
  the day's week `Told` (every town, every viewer) told at the towns' day; an event's creatures
  with their own `Told`. No version of the game state hash, the map import or the replay format
  moved: the 72 adventure replays' hash lines and the session's golden finals moved (behaviour).
- **Batch TWN-3** (survey §14; T23 after `TWN-Q14`): rows TWN-3-1 (T-13), -2 (§1's sum, a worn
  piece), -3 (T-12), -4 (TD-15), -5 (§5, T-11) are growth's; TWN-3-6 is T22b's half.
- **Open for T22b:** the garrison hero as the Legion's second hero and in the owner's share
  (§41's inputs); the Portal's hire; a capture's empty Portal (§45); the `cross-lane` bumps'
  order with E08's increment 12 (T108b's drafts touch ADR-0013 and ADR-0016 too, and pack format
  or game state hash numbers may move: whichever merges second bumps again).

### 48. Recruitment and the rows: scope, sources and choices (T22b)

The note's second half (decision 74; T22b whole, decision 79), written from the survey §5, §8 to
§10, §15, E08's increment 12 note (`docs/epics/E08-design.md` §12, §13: the army reference agreed,
E08 decision 305, the exchange, the one owner rule, the visit marks on `HeroOnMap`) and the
address map. The drafts were the same files' second blocks, with ADR-0023's new (all five in
`docs/adr/` since T24).

- **Recruitment builds:** the town command `Recruit` and the hire dialog with a town's source
  (§49), the purchase across the store, the adventure and the treasury (§50), the Portal's hire
  (§51).
- **The rows build:** the town's two bars under E08's one owner rule, admitted without a decision
  (§52), the garrison hero's record and its three moves (§53), its day, income and growth inputs
  (§53), what it changes at a visit and in the views (§54).
- **Not this increment's:** the computer's recruitment and hero moves (E13); an ally's visitor at a
  town (E10: the one owner rule already covers its stacks, TD-17 its hero moves); a garrison hero
  learning the guild's spells and its spell points at a guild (increment 5); the Tavern's hire and
  its limit (increment 4, which reads this half's count of heroes on the map); the town screen's
  displays (the client's).
- **Read by T22b** (from the address map's entries and two slices of `next_player_turn`, no
  survey section): the turn's start sets the points of the player's heroes, then of its two tavern
  heroes, then of each of its towns' **garrison heroes** (+0xC; 0x4C7486 to 0x4C74AE);
  `new_day`'s and the week's loops over **every** hero record (the day flags 0x4C80CF, the Stables
  0x4C8D2A, the spell points) reach a garrison hero too; `town_visitor_to_garrison` takes the hero
  out of its owner's list, so the economy survey's open point (§14 there: a garrison hero counted
  twice) is answered: once.
- **The open points of §47 settled:** the garrison hero in growth's inputs (§53); the Portal's
  hire (§51); a capture's empty Portal (`TWN-Q14`, §51); the `cross-lane` bumps' order with E08's
  increment 12 (§55, §56).

### 49. Recruitment: the command and the dialog (TWN-REC-001; GAM-DEC-008 amended)

- **The command** `TownCommand::Recruit { town: TownId, building: BuildingId }` (`h3-towns`;
  *agent's call:* by building, the original's click paths, survey §8, and the env's building head):
  `h3_towns::recruit_source(town, part, building) -> Result<RecruitSource, TownCommandError>`, a
  standing building (ever built, no built building superseding it) of role `dwelling` (its level
  and flag: index `level − 1`, `+ 7` upgraded), `horde` (its horde's index, the same way) or
  `portal`; refused, typed: another role (`NotRecruiting`), not standing (`NotStanding`), a base
  index whose upgraded dwelling is ever built (`NotStanding`: its count was moved, its growth is
  none). `RecruitSource::{Index(u8), Portal}`.
- **The game** (`h3-game`'s `src/town/recruit.rs`): the actor the town's owner (`NotYourTown`), its
  turn, no decision open (GAM-DEC-001); a Portal with nothing summoned is refused
  (`TownCommandError::NothingSummoned`: the original's message, no draw at the command, decision
  71); else `Decision::Hire { owner, dialog: HireDialog { target: HireTarget::Garrison(town),
  source } }` with `source` `HireSource::Town { town, index }` or `HireSource::Portal { town }`,
  and `HireOpened` to the owner. *Agent's call:* opened at a count of 0 too (the original's dialog
  shows it; its only legal answer is `Close`).
- **The adventure's types** (E08 T95b built them to be extended; `command.rs`, `cross-lane`):
  `HireSource::Town { town: TownId, index: u8 }`, `HireSource::Portal { town: TownId }` (a plain
  index: `h3-adventure` imports no town type), `HireTarget::Garrison(TownId)`: the town's upper
  bar (§52), the garrison hero's army when one stands, else the town's garrison, **never the
  visitor's** (survey §8: the manual, the reading and VCMI's client agree; VCMI's server also takes
  the visitor, a deviation of no rule). `hire_stock` gives nothing for a town's source (the counts
  are the store's), `hire_room` the room of the garrison target (ADV-ARM-007's room rule, the
  stack's most).
- **The view and the event** (the owner's; the protocol maps them, `cross-lane`): `HireView` and
  `HireOpened` name their source as `HireSeen::Object { object_type, anchor }` (as now),
  `::Town { town, index }` or `::Portal { town }`; the creatures, counts and costs read at each
  view, the counts from the store.
- **The legal commands** (GAM-DEC-001's set when no decision is open): per own town in `TownId`
  order, after its builds, a `Recruit` for each standing building of the three roles in the part's
  building order (a Portal only with a creature summoned). The env's entry (§55).
- **Replay** (the adventure replay format, `cross-lane`): `town recruit <town> <building>`, the
  purchases the hire dialog's lines as now.
- *As built (T27):* `TownCommand::Recruit` and the errors `NotRecruiting`, `NotStanding`,
  `NothingSummoned` (`h3-towns`' `command.rs`); `recruit.rs` with `RecruitSource`,
  `recruit_source` (the Portal holding nothing tested there, with the record, not in the game:
  *agent's call*) and `recruiting` (the legal set in one pass over the standing buildings, the
  same test). `h3-game`'s `town/recruit.rs` (`open`, `town_source`, `stock`, `buy`) behind
  `town::command`; `GameState::open_hire` now `pub(crate)`, `GameState::hire_stock` public (any
  source's stock; the env reads a dialog's creatures from it). `HireSeen` in `observe.rs`, carried
  by `HireView::source` and `HireOpened::source` (the object's type and anchor moved into
  `HireSeen::Object`). The legal recruitments after each town's builds in `town/offer.rs`'s
  `legal_builds`. Game state hash v32 (the target's tag 1, the sources' tags 1 and 2; E08
  T115a's #906 took v31), protocol v59 (#904 took v56, #906 v57, #907 v58), replay format v15 (`town recruit`, #904 took v14), env encoding v10
  (`RECRUIT`, entry 31, the two heads of `BUILD`).

### 50. The purchase (TWN-REC-001; ADV-HIR-002's room and price as built)

- **The stock** `h3_towns::recruit_stock(town, part, core, source) -> Vec<(CreatureTypeId, u16)>`:
  an index below 7 its creature with its count; an upgraded index its upgrade, then its base
  creature, **both with the index's one count** (survey §8, `recruit_dialog_init_town` 0x551560:
  the upgrade selected first); the Portal its creature and count; empty when nothing is summoned.
- **The check** (the game's `hire` for a town's source, in this order): `h3_towns::check_recruit`
  (the creature in the stock, else `NotOffered`; the amount from 1 to the count, else `Amount`),
  the adventure's room (`hire_room`; `NoRoom`), the treasury against `hire_price` (the core's cost
  times the amount; `CannotPay`). *Agent's call:* the town's two refusals are given as
  `HireError::NotOffered` and `::Amount`, one vocabulary for the client and the env whatever the
  source.
- **The application** (infallible): the price paid; the source's count lowered by the amount
  (`h3_towns::recruit` returns the town, the store writes it; the Portal's count too, its creature
  kept until the week sets it again); the creatures joined into the target army by the adventure
  (`join_at` at the room's slot; `StackChanged` naming `ArmyRef::Garrison(town)` or
  `ArmyRef::Hero(garrison hero)` to the owner; the town's viewers told the upper bar's change at
  their level, increment 1's `TownStackSeen` and the `Told` comparison); the dialog closes after
  the purchase of a one-creature source (`DecisionClosed`), else stays open, as the map's.
- **The most** each legal `Buy` names: the count, the treasury's bound and the room's, as
  `hire_mosts` (the counts from the store).
- **Tests** (`h3-towns`, `h3-game`): every refusal changes nothing (the store, the army, the
  treasury); an upgraded index sells either creature from one count and the other's count falls
  with it; a full garrison refuses with `NoRoom` while a stack of the creature takes it; a purchase
  into a garrison hero's army (§53).
- *As built (T27):* `h3_towns::recruit_stock(town, part, source)` (no core: the record names the
  creatures), `check_recruit(town, part, source, (creature, amount)) -> Result<(), RecruitError>`
  (`NotOffered`, `Amount`, given by the game as `HireError`'s), `recruit(town, source, amount)`;
  the adventure's `check_join((core, part), target, (creature, amount))` (the target's room and
  the price; `check_hire` now builds on it) and `hire` returning the garrison's changed
  `ArmySlot` (none for a hero's army), `HirePurchase` naming its army (`Joining`) and the map's
  count to lower, none for a town's. The game's `buy`: the store's town, its tests, the room,
  the treasury, then `Told::before`, the price, the store's write, the join, `StackChanged` to
  the owner, `Told::tell`; the one-creature close stays `GameState::hire`'s.

### 51. The Portal's hire (TWN-REC-002) and a capture's empty Portal

- `Recruit` at the `portal` building (survey §5, `town_portal_hire` 0x5D2950): the Portal's
  creature and count, into the town's upper bar's army (`town_garrison_army`, as a dwelling's), a
  one-creature dialog; refused while nothing is summoned.
- **A capture's empty Portal** (§45's open point; decision 82, `TWN-Q14`, recommended): the
  original keeps +0x3C at a capture, and a human's capture opens its town screen at once, whose
  creature panel draws an empty Portal's creature (T-11's path). So it is **drawn at the capture**
  when the Portal stands empty and the capturer owns a dwelling (from the capturer's stream, after
  the capture's own draws; decision 71's "what a human sees at once" carried to the capture). Not
  chosen: no draw at a capture (the week or a flag draws).
- *As built (T27):* `RecruitSource::Portal` / `HireSource::Portal`; a purchase lowers the count
  and keeps the creature, at 0 too, so a bought-out Portal is still listed and opens a dialog
  whose only answer is `Close` (TWN-REC-002).

### 52. The town's two bars (TWN-ROW-001; GAM-MAN-001, -002 amended)

- **The references** (agreed: E08 decision 305, the town lane's proposal): the **upper bar** is
  `ArmyRef::Garrison(town)` without a garrison hero, `ArmyRef::Hero(the garrison hero)` with one;
  the **lower bar** the visitor's `ArmyRef::Hero`. Their **bar owners** by E08's one owner rule
  (built by E08 T114a in `h3-adventure`'s army commands): a town's garrison its town's owner (the
  adventure's owner of the town object; the town lane wires it into the rule's lookup), a hero its
  owner.
- **Admitted without a decision** (decision 80, `TWN-Q14`, recommended): in its turn with no decision open, the
  owner's `Manage::Army` commands may name an own town's upper bar **alone** (moves within it, its
  dismissals) or **with that town's visitor** (between the bars, either way); the game's
  management check admits the pair as a meeting's two heroes (`Together`'s rule gains a town's
  two bars); `ArmyCommandError::Garrison` stays for a garrison the game does not admit (another's
  town, a neutral one). The original's town screen opens at will and needs no answer, so nothing
  is opened or closed; E08's `Exchange` (GAM-DEC-009) stays the map garrison's, opened by a visit.
  *Not chosen:* the town's rows as E08's `Exchange`, opened by a town command or at entering and
  closed by `Close`: a decision to open and close at every use, and the hero moves (§53) would
  change the armies the open decision holds.
- **The rules on the bars** are the army commands' (ADV-ARM-001 to -004, -007) under the one owner
  rule: a stack only from the acting player's bar; into another owner's only a move or split into
  an empty slot (an ally's visitor, E10); a hero's last stack never leaves its army (a garrison
  hero's too: the original's 0x5C7436 keeps it); the town's own garrison may be emptied.
- **The listings** (GAM-MAN-002 amended): the visitor's per-hero listing gains the upper bar as
  its other army while it stands on an own town's entrance with no decision open (after its own
  slots, as a meeting's other hero's); `legal_town_management(state, rules, actor, town)` lists the
  upper bar's own moves and dismissals (a garrison hero's whole management, its artifacts and
  formation too, through `legal_management` with that hero). The env plays no army management
  (its encoding has no army entry, before or after).
- **Told:** the commands' `StackChanged` to the owner; the upper bar's change to the town's
  viewers at their level (increment 1's `Told`); the visitor's army as any hero's.
- *As built (T28a, decision 93):* the admission is `GameState::admits_army` (`town_bars`, out of
  line) before `Together::admits`; the adventure's `held` answers `ArmyRef::Garrison` for an own
  town without a garrison hero (`Held::Town`, may be emptied) and a garrison hero after the heroes
  on the map (`Held::TownHero`, keeps its last stack), both `#[cold]`; a town's garrison named
  with a hero not its visitor, or while a garrison hero stands there, is `Garrison`; the listings
  `town_partner` in `legal_management` and `legal_town_management`. The town's views are as before
  (the garrison hero's army in them is T28b's).

### 53. The garrison hero (TWN-ROW-002 to -005; ADV-MOV-002, GAM-RES-003, -005 amended)

- **Its record** (decision 81, `TWN-Q14`, recommended): **the whole record of the hero on the map moves into
  the town's record in the adventure** (`Holds::Town { formation, garrison_hero:
  Option<HeroOnMap> }`; adventure state hash next): its army, points and day's maximum, formation
  and tactics flag, and from E08's T115a its battle bonuses and timed marks, kept as they were (the
  original keeps the hero record whole); its tile stays the entrance's, unread. It is **not on the
  map**: no tile, reach, meeting or visit, not the town's visitor (TWN-VIS-002), not in the env's
  hero slots, not among the player's heroes on the map (the part's most, 8: the original's player
  record +1). Its details stay in the game's store. E08 decision 307's drop at leaving the map does
  not apply (the inbox's call: the record moves, it is not dropped). *Not chosen:* the hero kept
  among the heroes on the map with a placement flag (every map query, the reach, the tiles, the
  views, the env's slots, the visitor's derivation, would have to skip it); the hero leaving the map
  and losing its on-map state (the original keeps its points, marks and bonuses).
- **The lookups:** `ArmyRef::Hero(id)` resolves to a garrison hero's army after the heroes on the
  map (the adventure's army lookup); the game's ownership check admits a garrison hero for its own
  management and its town's bars, never for a move, a visit or a battle of its own.
- **The moves** `AdventureCommand::TownHeroes { town, change: TownHeroChange }` (*agent's call:*
  the adventure's command, since the garrison and the visitor are the adventure's facts, ADR-0028
  point 2; the variant in `command.rs`, its rules in the town module `src/town.rs`), each the town's
  owner's in its turn with no decision open (the original's moves are a human's town screen's; a
  computer's are E13's), each refusal typed (`TownHeroError`) and changing nothing:
  - **`Enter`** (TWN-ROW-002; `town_visitor_to_garrison` 0x4B9FC0, `army_absorb` 0x44B620): a
    visitor of the town's owner, no garrison hero (`GarrisonHeld`, `NoVisitor`); the town's
    garrison joins the visitor's army by the reading's steps (each garrison stack in slot order:
    onto the hero's stack of its creature, else the first empty slot, else the hero's own two
    stacks of one creature merged, the later into the earlier, and the slot tried again), all or
    nothing: refused when no slot frees (`ArmiesDoNotFit`, GENRLTXT 276; VCMI's test, the
    creatures of both counted once fit the slots) or a merge would pass a stack's most
    (ADV-ARM-007; `TooManyCreatures`, unreachable with the original's counts); else the hero's
    record moves into the town (its garrison then empty), its tile freed.
  - **`Leave`** (TWN-ROW-003; `town_hero_out_click` 0x5D5220, `town_garrison_to_visitor`
    0x5BE390): a garrison hero (`NoGarrisonHero`), no visitor (`EntranceHeld`), fewer than the most
    heroes on the map (`TooManyHeroes`, GENRLTXT 19), a stack in its army (`EmptyArmy`, GENRLTXT 20;
    T-5's case only): its record back onto the entrance as the visitor, its points as kept.
  - **`Swap`** (TWN-ROW-004; `town_swap_heroes` 0x5BE450): both, the visitor the town's owner's
    (TD-17 by the reading: an ally's visitor moves no hero, E10): the visitor's record into the
    town, the garrison hero's onto the entrance, each with its army.
  - **The invariant:** while a garrison hero stands, the town's own garrison is empty (`Enter`
    takes it in; recruits go to the hero's army, §49; a neutral week has no garrison hero, §44).
    Tested as a property.
  - **Told:** the visitor's leaving and arriving as a hero's leaving and entering the map to the
    viewers who see its tile; a new `GarrisonHeroChanged { town, hero: Option<HeroId> }` to the
    owner and to each viewer whose town view shows the garrison's army (*agent's call*), with the
    upper bar's `TownStackSeen`.
- **Its day** (TWN-ROW-005; §48's reading): its day's maximum and points set at its owner's turn
  start after the heroes on the map, in `TownId` order (ADV-MOV-002 amended; on land, no boat);
  its spell points each day as every hero's (the store's loop, as now); its day marks dropped at
  each day's start and its week marks at each week's (E08 T115a's drops, over the garrison heroes
  too); its battle bonuses kept until its next battle. **Its income:** its `daily_income`
  (Estates, the worn resource artifacts; `town_gold_income`'s garrison hero, `count_worn_artifact`'s
  towns' heroes) with its owner's (GAM-RES-003 amended); **its army's weekly resources** and the
  town's garrison's: the Crystal Dragons in an owned town's upper bar (the garrison hero's army,
  else the town's garrison) count once with the heroes' (GAM-RES-005's "a town's garrison is
  E09's" lifted; economy survey §7.3 point 3). Not reached: its reveal (off the map; the town's own
  radius covers the entrance; the original's `reveal_owned` over the hero records is not read for a
  hidden hero, an open point that matters only after a Cover of Darkness).
- **Growth's inputs** (§41, its open point): `legion` the visitor's `town_growth` plus the garrison
  hero's; `share_bp` the highest of the owner's heroes on the map and its garrison heroes
  (`player_hero_wears_artifact` 0x4BACB0: the list, then the towns' +0xC heroes).
- **Its views:** the owner's own hero views list its garrison heroes with the town in place of a
  tile (`OwnHeroView`'s place `Map(pos)` or `Garrison(town)`, *agent's call*; the protocol maps
  it); every other viewer sees it only through the town's view (§54).
- *As built (T28a, decision 93):* the moves are `TownCommand::Heroes { town, change }`
  (`TownHeroChange`), run by `h3-game`'s `town/rows.rs` (the store, the owner, then
  `AdventureState::move_town_hero`, `StackChanged` of `Enter`'s slots, the town's `Told`), listed
  after each own town's recruitments (`check_town_hero`); the record in
  `AdventureState::town_heroes` (a block by town, none while no town holds one; adventure state
  hash v25 after the heroes on the map, each as a hero's), its marks in the marks block by hero id;
  `TooManyHeroes` reads the town part's `most_heroes` (8, part v6); told as `GarrisonEntered` and
  `GarrisonLeft` (the viewers of the entrance; `Swap` the two in that order); the owner's view is
  `AdventureObservation::garrison_heroes` (`GarrisonHeroView`) and its details in
  `GameObservation::heroes`; `OwnHeroes` covers them; replay `town heroes <town> enter|leave|swap`
  (v16); the env's `TOWN_HEROES` + 3 (v11); protocol v64. Its day, income and growth inputs are
  T28b's; until T28b's rows 1 and 3 a garrison hero does not defend (`TWN-STUB-14`: its town falls
  as T-5's, `release_town_hero` at the capture).
- *As built (T28b1, decision 94):* its day is `AdventureState::start_town_heroes` (the town
  lane's `src/town/rows.rs`), called last by `start_turn`: `day_points` on land with its week
  marks' points, written only where they change (a garrison hero spends none); the marks' drops
  (`drop_timed_marks`) change the block by hero id directly, so they reach the garrison heroes
  (`change_marks` admits only heroes on the map); its spell points were the store's loop
  already. Its income: `GameState::pay_income`'s heroes chain `town_heroes`; the week's
  resources read `AdventureState::upper_bars` (`src/town.rs`: each owned town's garrison hero's
  army, else its garrison; VCMI counts the towns' garrisons too). Growth's `Owner::share_bp`
  chains `town_heroes`, `legion` sums the visitor's and the garrison hero's by level,
  saturating. No version moved; the 86 replays hold no garrison hero. The plans: `check_income`
  over the garrison heroes and the owned towns' garrisons, `check_isolation` over another
  player's garrison hero (its record kept, its points full at its owner's turn start); the view
  property's checks of a hero's points and marks events read the garrison heroes' views too.

### 54. What a garrison hero changes (TWN-SIE-001, -002, TWN-CAP-003, TWN-VEW-001 amended; GAM-BAT-001)

- **A visit of another team** (survey `E09-towns.md` §9.1, rows 1 and 3; increment 1 built rows 2
  and 4):
  - **a visitor and a garrison hero** (row 1): a battle against the visitor with its own army,
    **not a siege** (`siege: None`, the gate tile's battlefield as any hero's battle, no town
    bonuses; ADR-0023 point 1's draft), its owner's side; no merge; the town stays its owner's
    whatever the outcome (the attacker outside; a beaten visitor to the pool); the next visit
    meets the garrison hero (the guide's "another click", VCMI's `isBattleOutsideTown`);
  - **a garrison hero alone** (row 3): **a siege** against it with its own army
    (`Defender::Hero`), its town bonuses (ADR-0023 point 3); a win captures the town (TWN-CAP-003)
    and the beaten garrison hero goes to the pool, its record gone from the town; a loss or a
    retreat leaves it.
  - **T-5** (`TWN-QUIRK-11`, faithful, decision 12; `town_visit` 0x4AB0EE): a garrison hero whose
    army is empty does not defend: the first visitor of another team enters and captures the town
    without a battle, the hero back to the pool with its artifacts, no experience, nothing taken.
    Reachable only by a defence won with no stack of its own left (summoned creatures alone; the
    bars keep a hero's last stack); the tests build it as a state.
  - **The merge** (TD-7, TWN-SIE-002) only without a garrison hero (row 2).
- **The views** (TWN-VEW-001 amended; survey §10): at level 3 a garrison hero and its army as the
  town's garrison (the quick view), at lower levels the garrison's levels from its army as now;
  the owner's view marks the town's garrison hero (the status window's `townqkgh.pcx`).
- **Increment 2's reads:** the build and the events read no hero; the income reads the garrison
  hero's (§53).
- *As built (T28b2, decision 95, whole):* the visit is `AdventureState::town_visit` (a town of
  another team defended by a garrison hero with a stack; one without not, `TWN-QUIRK-11`, the
  original's `town_defended` 0x5BDFE0 read a third time) and `siege_start` (the defender the
  visitor, else the garrison hero as `Defender::Hero` off the map; `BattleStart::town` none when
  both stand: row 1 is the gate tile's battle). No merge for a garrison hero (`town::merge` skips
  a defender off the map). Its battle's end: `AdventureState::end_battle` hands a hero not on the
  map to `end_town_hero_battle` (the town lane's `src/town/rows.rs`: the survivors by slot, an
  empty army kept for a winner; else `release_town_hero`, `HeroRemoved` to its owner at the
  entrance); the game's `end_battle` keeps a garrison winner's survivors even when none is left;
  Necromancy reaches it (`raise` through `hero_or_held_mut`), and its bonuses are cleared
  (`change_marks` admits a garrison hero). The view: `AdventureState::upper_bar` is every level's
  garrison (`GarrisonView::stacks` and the owner's `army`), the mark `TownOwnerView::garrison_hero`
  and `TownChanged::garrison_hero` (protocol v67), not a hero id (the original's mark is a picture;
  the owner sees the hero in its own heroes) and no `GarrisonHeroChanged` (the town's whole view is
  told: `Told` around a garrison hero's battle's end and the army commands naming its army, in
  `state/manage.rs`' `town_told`). No hash, format or encoding moved; `TWN-STUB-14` lifted.

### 55. Recruitment's and the rows' views, versions, ids and gates

- **Views** (`docs/06-security.md`'s town section, the `shared/` task): the hire dialog's town
  sources the owner's alone; a garrison hero at the town's level 3 and its owner's; the hero moves
  told as §53 says; the protocol maps the new commands, events and view fields (`cross-lane`, the
  server lane's fold gate).
- **Versions** (each bumped by the first task that needs it, the number the next at merge; T22a's
  table in §47 for growth's):

  | Version | Change | Task | Owner |
  |---|---|---|---|
  | game state hash | `Decision::Hire`'s town sources and target | T27 | adventure, `cross-lane` |
  | adventure state hash | a town's garrison hero | T28a | adventure, `cross-lane` |
  | adventure replay format | `town recruit`; `town heroes <town> enter\|leave\|swap`, management lines naming a town's bar | T27, T28a | adventure, `cross-lane` |
  | env action encoding v10, v11 | `RECRUIT` (tile and building heads); `TOWN_HEROES` + 3 (tile head) | T27, T28a | AI lane, `cross-lane` |
  | protocol | the commands, `HireSeen`, `HireTarget::Garrison`, `GarrisonHeroChanged`, the own hero view's place | T27, T28a | server lane, `cross-lane` |
  | town state hash, town part | none (the counts and roles are growth's) | — | — |

- **Rule ids** (`town.toml`): TWN-REC-001 recruitment at a town, -002 the Portal's hire; TWN-ROW-001
  the town's two bars, -002 the visitor into the garrison, -003 the garrison hero out, -004 the
  swap, -005 a garrison hero's day and income. Amended: TWN-SIE-001, -002, TWN-CAP-003, TWN-VEW-001,
  TWN-VIS-002; GAM-DEC-008, GAM-MAN-001, -002, GAM-BAT-001, GAM-RES-003, -005, ADV-MOV-002.
- **Quirks:** `TWN-QUIRK-11` T-5 (faithful), after growth's `-8` to `-10`. **Stubs:** none new (an
  ally's visitor and town are E10's: the one owner rule and TD-17 decide them without a stub; the
  computer's town play is E13's, no player meets it).
- **Gates:** each rule's unit tests (the refusals change nothing; an upgraded index's one count;
  the absorb's steps, its all-or-nothing refusal and the hero's own merge; each move's refusals;
  the record kept whole across `Enter` and `Leave`, its points and marks; the bars under the owner
  rule; rows 1 and 3, T-5); properties (no hero both on the map and in a garrison; a garrison hero's
  town has an empty garrison; a purchase never drives a count below 0; a refused command leaves
  the state's hash); scripted replays on the builds map (a purchase at an upgraded dwelling and at
  the Portal, `Enter`, a week in the garrison, `Leave` with full points, `Swap`, a siege of a
  garrison hero and the battle beside one); the env's tests (the entries, their masks); the fold
  gate; batch TWN-3's row 6.
- **Batch TWN-3's row 6** (survey §14) gains one record: the garrison hero's points at its owner's
  next turn (the reading predicts its maximum).

### 56. The building tasks (the whole increment; the order with E08's increment 12)

The tasks continue the epic's ids; each task's cut confirmed at its start. **E08's increment 12
runs at the same time** (its rounds 2 to 8, T109 to T117, design note §19 there): every
`cross-lane` number is the next at merge, the second to merge rebases and bumps again (the town
lane's `TRAPS.md`); the rows wait on E08's army reference and owner rule **built** (T114a) and the
visit marks on `HeroOnMap` (T115a).

| Round | Tasks | Content |
|---|---|---|
| 2 | **T23** | Batch TWN-3 (survey §14, six rows; row 6 with §55's record): the spec and template, the harness's reads and writes (the week's and month's kinds, a town's counts, Portal and garrison hero; the writer's garrison hero and Portal), the recipe, the runs, the scoring. Needs the original data. Cut at its start (increment 2's T14 was cut in three) |
| 3 | **T24** | `shared/` (`cross-lane`): the five drafts applied with T23's verdicts; `StreamId::TOWNS`; `h3-content`'s `TownGrowth`, `OwnerTownGrowth` and two quirk flags (pack format v52) with the packs' six artifacts (`cross-lane`); the glossary (available creatures, the week's kind, the Portal, the upper bar, the garrison hero); `docs/06-security.md`'s town lines. In merge order with E08's T112 (both amend ADR-0013, -0016, -0017) |
| 4 | **T25** | Growth's town side (the town lane's paths only): T25a the part's roles, `roe` and parameters (part v5), the record's counts and Portal (store v3), TWN-GRO-001 and -005 with `TWN-QUIRK-8`; T25b TWN-GRO-003, -004 (`TWN-QUIRK-9`, `-10` and their flags), -006, -007 as pure functions |
| 5 | **T26** | Growth in the game (`cross-lane`): T26a the week's kind and the towns' stream (TWN-GRO-002), the day's order (GAM-CAL-002, -003), the inputs (owners, dwellings, the Legion), the week and the month, the first creatures at every build, the events' creatures (`TWN-STUB-10` lifted), map import (`roe`), the game state hash; T26b the neutral garrison week, the Portal's draws (at a capture too, decision 82), the views (the owner's counts and Portal, the week's kind) and the protocol, the replays re-recorded, the bench |
| 6 | **T27** | Recruitment (`cross-lane`): `TownCommand::Recruit`, the hire dialog's town sources and target (the town's garrison until T28a), the purchase across the store and the adventure, the legal commands, `HireSeen` and the protocol, `town recruit`, the env's `RECRUIT` (v10) |
| 7 | **T28** | The rows (`cross-lane`; after E08's T114a and T115a merged): T28a the town's bars admitted (TWN-ROW-001), the garrison hero's record (adventure state hash), `Enter`, `Leave`, `Swap` (TWN-ROW-002 to -004), the hire's target into its army, its events and views, the replay format, the env's entries (v11), the protocol; T28b what it changes: its day and income (TWN-ROW-005; ADV-MOV-002, GAM-RES-003, -005 with the town's garrison's Crystal Dragons), growth's inputs with it, rows 1 and 3 of a visit (TWN-SIE-001, -002, TWN-CAP-003, GAM-BAT-001), T-5 (`TWN-QUIRK-11`), TWN-VEW-001 |
| 8 | **T29** + **T30** | T29: the scripted replays on the builds map (growth through a month, recruitment, the rows), the census of `original_data` (§47's walk), the properties, the benchmarks (`day-start-builds-36` re-baselined, a week workload), `--with-original-data`. T30: the audit against the exit criteria; increment 3 closed; increment 4 (the Tavern) refined |

*Agent's calls:* growth before recruitment (the dialog sells the counts growth keeps), recruitment
before the rows (it needs no garrison hero; T28a widens its target); T25 in the town lane's paths
alone, so it never waits on E08's merges; each env entry with its command's task (two bumps, each
the AI lane's `cross-lane` consent).
- *As built (T29a, decision 96: T29 in two):* the builds map gains red's Dungeon "Pit" (a horde,
  the Portal, the Grail), the neutral Rampart "Grove", red's Valeska and a Guardhouse red owns, so
  §47's and §55's scripted replays play on it (`builds-month-s1`, `-rows-s1`, `-siege-s1`); the
  census in `h3-map`'s `original_data` walks every count of the 160 maps to day 29 (`played_growth`;
  the draws read from the game: the kinds from their events, a garrison week's level from the
  garrison). T29b: the properties and the week workload.
- *As built (T29b, decision 97, whole):* the properties in `h3-map`'s plans (the counts' bound in
  `check_invariants` with town events giving 65 535 of a level; the Plague and a purchase's
  amount in `check_towns`; `a_forks_week_is_its_parents`), `Step::Town` playing the hero moves
  so that garrison heroes defend sieges alone and beside a visitor (the cover test counts each,
  won and lost); the week workload is `month-start-builds-36` (the step that starts day 29:
  week 5 and month 2 over the six towns), `day-start-builds-36` already timing a week's first day.

### 57. Increment 3's exit criteria (accepted at `TWN-Q14`, decision 83; the epic's list)

1. The town part's roles `horde`, `portal`, `week_of`, a record's `roe` and the growth parameters
   compile from the packs; each town holds its fourteen counts, bounded at 32 767 (`TWN-QUIRK-8`),
   and its Portal; a built dwelling gives its first creatures at the setup, a command's and an
   event's build (TWN-GRO-005).
2. A dwelling's growth is the reading's sum (the castle by the fort, the owner's share, the
   Legion of the visitor and the garrison hero, the first horde, the owner's dwellings by their
   first creature, the Grail); each week adds it with the week's bonus, halved in a neutral town
   (TWN-GRO-001, -003).
3. The week's kind is drawn from the towns' stream (1 in 4 a creature's; on an RoE map among the
   `roe` records; a standing `week_of` building's week), public in `WeekStarted` and the
   observation; a month after a `week_of` week is its level's base creature's without the
   creature's draw; a creature's month doubles the exact creature (`TWN-QUIRK-10` off: the level),
   the Plague takes the growth then halves (`TWN-QUIRK-9` off: every town halves) (TWN-GRO-002,
   -004; GAM-CAL-002, -003).
4. A neutral town's garrison grows each week as read; the Portal is drawn each week, at its build,
   at a flagged dwelling and at a capture that finds it empty, its count set; a town event's
   creatures reach the counts (TWN-GRO-006, -007, TWN-EVT-004; `TWN-STUB-10` lifted).
5. Its owner recruits at a standing dwelling, horde or Portal through the hire dialog into the
   town's upper bar's army, an upgraded index selling both creatures from one count; every refused
   purchase changes nothing (TWN-REC-001, -002; GAM-DEC-008).
6. Its owner's army commands move stacks within an own town's upper bar and between it and the
   visitor by the one owner rule agreed with E08's increment 12 (TWN-ROW-001; GAM-MAN-001, -002).
7. The visitor enters the garrison all or nothing by the absorb's steps, the garrison hero leaves
   onto a free entrance within the most heroes on the map, the two swap; a garrison hero keeps its
   whole record, gets its points at its owner's turn start, its spell points and its marks' drops
   each day and week, its income and its army's weekly resources (TWN-ROW-002 to -005;
   ADV-MOV-002, GAM-RES-003, -005).
8. A garrison hero alone defends the siege; a visitor beside one fights outside it and the town
   stays; an empty garrison hero yields the town (`TWN-QUIRK-11`); a viewer at level 3 sees a
   garrison hero and its army (TWN-SIE-001, -002, TWN-CAP-003, TWN-VEW-001).
9. A town's counts, its Portal and its hire dialog are its owner's alone, the week's kind
   everyone's; every viewer's changed view is told (the fold gate) (TWN-VEW-004).
10. The env plays recruitment and the hero moves (encodings v10, v11); the replays record them.
11. Every new rule has its id, source and tests; `TWN-QUIRK-8` to `-11` recorded; the gates pass
    (coverage; determinism with the re-recorded replays; the census of `original_data` to day 29;
    the benchmarks at their baselines and the week workload baselined); batch TWN-3 run and scored,
    its consensus rows `verified`.

### 58. Risks and open points

- **Two lanes in the same crates at once:** E08's T113 to T115 and this increment's T26 to T28
  change `h3-adventure`'s state and hashes, the game's day, the army commands and the hire dialog:
  merges in order, a rebase and a second re-recording when both bump.
- **The rows wait on E08's T114a and T115a:** a delay there delays T28 (growth and recruitment do
  not wait).
- **The garrison hero's loops:** every per-hero step that must reach it (the turn's points, the
  marks' drops, the income, the weekly resources, the Legion's share) is explicit; a missed one is
  a silent divergence, so the census walk and a property (a week in a garrison against the same
  week on the map, the same points, marks and gains) guard them.
- **The legal sets grow:** a `Recruit` per standing dwelling of every own town each step (the
  env's masks; the AI lane's step cap already bounds the revisit loops).
- **Open:** a garrison hero's reveal (§53; only after a Cover of Darkness).

## Part F — increment 4: the Tavern

Increment 4's note (the Tavern; the epic's decisions 99, 100, 102), written by T33a (the hire,
§59 to §71) and T33b (the information, the views, the env, the gates, the cut and the exit
criteria, from §72) from the survey `docs/rules/surveys/E09-tavern.md` (§ numbers are the
survey's unless they name this note) and the answers at `TWN-Q16` (decisions 103 to 107) and
`TWN-Q17` (decisions 109 to 112). A handover, not binding: the drafts
`docs/lanes/town/adr-drafts/*-increment-4.md` held the decisions, accepted at T33b's checkpoint
(`TWN-Q18`) before anything is built, and moved into ADR-0013, -0016, -0017 and -0028 by T35a
(2026-11-16, with batch TWN-4's verdicts and decisions 126, 127). *Agent's call (change freely)* marks a choice of this
note's (decision 113) that the task building it may still change.

### 59. The hire: scope, sources and choices (T33a)

The note's first half (decision 108: T33a the hire, T33b the information and the cut), written
from the survey §2 to §5 and §12 (the dismissal, read by T33a), the address map and the code as
it stands (`h3-game`'s `town/pool.rs`, `setup.rs`, `state/battle.rs`; `h3-towns`' `command.rs`,
`view.rs`).

- **This half builds:** the part's tavern weights and price (§60), the pool's records, away
  records and the players' two tavern heroes (§61), the draw (§62), the start's and the week's
  fills (§63), the town's hire (§64), the replacement (§65), the adventure map's Tavern (§66), a
  hero that leaves the game and the heroes that come back (§67), the tavern heroes over a turn
  (§68), the dismissal (§69), the map's hero settings (§70); ids, versions, open points (§71).
- **Not this half's:** the Thieves' Guild, the Den of Thieves, the views and events, the env, the
  gates, the cut and the exit criteria (T33b). **Not this increment's:** a hired hero learning
  the town's guild spells (increment 5, `TWN-STUB-15`); the computer's hires and its choice to
  retreat (E13; `ADV-STUB-9`, E10); a defeated player's heroes and tavern (`player_defeated`
  0x4F13C6, E10); the loss condition's hero at a dismissal (E10, `TWN-STUB-16`); the tutorial's
  fixed classes and a campaign's carried heroes (not followed); the rumours (decision 99).
- **`TWN-Q17`** (one round of four, each as recommended): a hero type's record made at its first
  draw and kept by type (decision 109; not chosen: every record at the build, a fresh record at
  each draw); the tavern weights in the town part's faction records (decision 110; not chosen:
  the core's hero classes); a town command for a town's tavern and a decision for the map's
  (decision 111; not chosen: a decision for both); the dismissal in this increment (decision 112;
  not chosen: later).
- **Read by T33a** (survey §12 and the address map): the dismissal's button and its five refusals
  (`hero_screen_refresh` 0x4E1A50, `hero_mission_critical` 0x5F2A40, `hero_screen_no_dismiss`
  0x698A90); neither hire reveals (`hero_place` 0x4D7900 and `hero_show` 0x4D75D0 read whole, no
  reveal call); a leave without bit 17 refills the points after clearing the army, so
  `hero_max_land_movement` 0x4E4990 starts from speed 20 (T-16, §67).
- **The agent's calls** (decision 113, each confirmed at `TWN-Q18`, decisions 115 to 122): the
  taverns' stream (§63); the away record in the pool (§61); a Random faction no start read drawn
  at the first fill (§61); a player without a faction weighted evenly (§62); the start's fill
  after the build's last draw (§63); a new record's points (§67); `ADV-STUB-9` left to E10 whole;
  T-15 closed as unreachable (§65).

### 60. The town part: the tavern weights and the hire's price (TWN-TAV-001's data; `h3-towns`, `h3-pack-towns`; part v7)

- **`TownRecord::tavern`**: per hero class of the core its weight in that faction's taverns;
  `towns.toml` gives `[town.tavern]` as class key = weight (a `u8`), compiled to a dense list by
  the core's class index (a class not given 0). The compiler refuses a class the core lacks
  (`UnknownHeroClass`), a class given twice and a record whose weights sum to 0 (the draw would
  have nothing to weigh). *Agent's call:* in the record, not in the parameters: the column is the
  faction's (decision 110).
- **`TownParams::hire_price`**: resources of the core, as a building's cost (2 500 gold,
  `hero_price` 0x67814C; tier-2 data of the reading, every edition and difficulty, the
  computer's hires too).
- **The harvest** (`h3-harvest`, `cross-lane`, original data): `HCTRAITS.TXT`'s tavern columns,
  Castle to Conflux, by the class rows (0 Knight to 17 Elementalist, the core's order): the base
  pack's eight records from `H3bitmap.lod`'s file, the AB pack's Conflux from `H3ab_bmp.lod`'s
  (both files the same nine columns in this build). RoE's file lacks the Conflux column; an RoE
  edition (E11) has no Conflux record, so nothing overrides. The weights read 5 or 6 a class and
  sum to 100 a column (§2.2); the harvest's test pins both.
- **The roles and the bound** stay: `tavern` (a standing Tavern or Castle's Brotherhood,
  `has_tavern`, increment 1) opens a town's tavern; `most_heroes` (8, increment 3) bounds the
  hire. Part v7 (the record's field, the parameter).
- *As built* (T36): `TownDef::tavern` and `TownRecord::tavern` (a `u8` per core class, by
  `HeroClassId`), `TownParamsDef::hire_price` and `TownParams::hire_price`,
  `TownCompileError::UnknownHeroClass`; the sum checked after the overrides. **The base records
  cannot name the AB pack's two classes** (outside their closure), which weigh 5 in every SoD
  faction's column: the AB pack overrides each base record with them, `TownPatch::tavern` changing
  or adding the classes it lists (the silo's patch replaces its list). So a base faction's tavern
  sums to 100 with AB or SoD and to 90 on the base pack alone (an RoE edition's own weights are
  E11's). The harvest reads `H3bitmap.lod`'s `HCTRAITS.TXT` as the class harvest does
  (`H3ab_bmp.lod`'s file is byte-identical in this build, checked); the price is the gap-fill's
  `[towns.tavern]` (source `exe-hero-price`), as `most_heroes`. Decision 132.

### 61. The pool's records and the players' tavern heroes (TWN-SET-005, TWN-SET-001 amended; `h3-game`'s `town/pool.rs`, `state.rs` `cross-lane`)

- **The statuses** gain **`Offered`**: a tavern hero's type, out of every draw, its player the
  slot's (the original's 0x40, which also marks a type the map does not allow: the engine keeps
  `Out` apart, *agent's call*). A type is `Free`, `Taken(colour)` (on the map or in a garrison),
  `Offered`, `Prisoner` or `Out`.
- **A type's record** (decision 109): `HeroPool`'s entry gains the type's `HeroId` once it has a
  record. The build gives the placed and generated heroes theirs as now (the setup's order, then
  the main towns' heroes); a draw that offers a type without one **makes it**: the next id (one
  past the store's last), its details GAM-HER-001's defaults under the type's map-wide entry
  (§70), its level seed (GAM-HER-002), its spell points full, from the stream of that draw (§63,
  §65). From then on the type keeps its id: its details stay in the store whatever happens to the
  hero, and the pool names that id at every later draw of the type, by any player.
- **The away record** (*agent's call*, ADR-0016's draft point 3): beside a recorded type off the
  map the pool keeps what the adventure keeps of a hero on the map: its points left and the day's
  maximum, its formation and tactics flag, and its army (meaningful only while the type is a
  tavern hero: every draw replaces it). A placement hands it to the adventure; a leave hands it
  back (§67). A garrison hero has none: the adventure keeps its record in its town (ADR-0016
  point 3).
- **Each player** gains **its faction** (`Option<FactionId>`: TWN-SET-001's resolved choice, kept;
  none without a choice) and **its two tavern heroes**: two slots, each a `HeroId` or empty, and
  each a **returning mark** (the original's bit 17 of +0x105). *Agent's call:* a Random choice
  that no start reads (`needs_faction` false: no main town, no random hero, no random town of
  its own) stays undrawn at the build and is drawn at the player's first fill from the taverns'
  stream, before its slot 0, so the setup stream does not move for any map.
- **Where:** the pool, its records' away parts, the factions and the slots are one block,
  `Arc<HeroPool>` as today: shared by forks, copied on a write (a fill, a hire, a leave, a
  turn's end or a day's start that clears a mark, decision 126). The store of heroes grows at a new record (its outer list
  copied once on a fork's write, as a hero's change copies it now); the level seeds' block
  (`Progression`, static until now) grows with it. *Bench:* a fork still counts the same
  references; the week's fill is one pass over the classes and types per slot (156 types, 18
  classes): `day-start-builds-36` and `month-start-builds-36` move and are re-baselined.
- **The hash** (the game state hash, `cross-lane`): the pool's encoding gains per type its record
  (its id and away record) and per player its faction and its two slots with their marks; the
  taverns' stream's position (§63).
- *As built* (T37a, decisions 133, 136): `HeroStatus::Offered`; each pool entry its record
  (`Option<HeroId>`: the build's placed and generated heroes' by type, the first of a repeated type,
  a drawn type's from its first draw) and its away record (`Away`: points left, the day's maximum,
  formation, tactics, army; a new one with no points, loose, tactics on; two flags for the speed
  gate, `stale`, its points to derive at the next turn start, and `full`, its record's spell points
  at their maximum since a fill, which the day's regain skips: `day_mana` never changes points at
  the maximum, so the result is the same); per player a `Tavern`
  (faction, two slots and their marks): **a slot names its hero's type**, its entry the record (the
  agent's call; the observation names the id). `Progression::record` grows the level seeds and the
  sexes by one. The factions come from the build's resolved choices (`HeroPool::set_factions`); a
  `Random` choice no start read is drawn at the start's fill. Game state hash **v33**: the pool's
  encoding as above, the taverns' stream after the towns'.

### 62. The draw (TWN-TAV-001; `h3-game`'s `town/tavern.rs`, new; `tavern_hero_draw` 0x4BB5E0)

A pure function of the pool, the core, the part and its arguments: the player's colour and
faction, the class to avoid (the other slot's hero's, or none), whether only its faction's
classes count (slot 0), and the stream; it returns a type or none and takes nothing (the caller
marks the type `Offered`).

1. Per hero class in the core's order: its **count**, the types of the class that are `Free` and
   whose mask holds the colour (TWN-SET-005's masks, SoD's disposed heroes); its **weight**, the
   faction's record's tavern weight of the class (§60). *Agent's call:* a player without a
   faction (a setup without towns) weighs every class 1 and has no own faction.
2. No class with a count: no type, no draw.
3. A class with no count weighs 0.
4. The class to avoid weighs 0 while another class has a count.
5. Own faction only (slot 0): when a class of the player's faction (the class record's faction)
   still weighs anything, every other class weighs 0.
6. A total weight of 0 (not reachable with the shipped data: every weight is 5 or 6): no type,
   no draw (*agent's call*, the original's `random_between(1, 0)`).
7. Else a class by one draw of 1 to the total over the classes in order (the first whose running
   sum reaches it), then the n-th counted type of the class in type order, n one draw of 1 to its
   count: two draws, the original's order.

The SoD edition's Conflux rule (step 3 of the reading) is not modelled: dead in this build
(edition 3), E11's lead. The forced class (the tutorial's) is not followed.
- *As built* (T37a): `town::tavern::draw`, as the steps; a faction without a town record
  weighs every class 1 too.

### 63. The fills: the game's start and the week (TWN-TAV-002; GAM-CAL-002 amended)

- **The release** (the week only; `new_day_week_events` 0x4C8780): per player in player order,
  slot 0 then slot 1, a slot holding a hero whose returning mark is clear lets it go: its type
  `Free` (its record and away record kept), the slot empty. A returning hero stays (TD-18,
  decision 103).
- **The fill** (`tavern_new_heroes` 0x4C8450), per player in player order (every player: none is
  defeated before E10), slot 0 then slot 1, each **empty** slot: the draw (§62) with the other
  slot's hero's class to avoid (none when that slot is empty) and own faction only for slot 0;
  a type found is `Offered` and the slot holds its hero id, its record made when it has none
  (§61). Then the type's **full starting army** (GAM-SET-001: the first stack's count, each later
  slot's roll and count; a rolled war machine's artifact worn at the first free position that
  takes it, as GAM-HER-001), replacing its away army; its **backpack equipped** (ADR-0013's draft
  point 14.5: each artifact from the last place to the first into the first free position that
  takes it); its spell points full (HER-MAN-001). Its points are set at its owner's turn start
  (§68).
- **The stream** (decision 115, `TWN-Q18`): **the taverns' stream**, `StreamId::TAVERNS`, a
  new named kernel stream (8, beside `TOWNS`; a `shared/` change as T24's `TOWNS`), for both
  fills: no existing draw of any stream moves, so the replays keep their outcomes but for the new
  state. Per filled slot: the class, the type, a new record's draws (its default experience or a
  custom experience's levels at load, GAM-SET-002; then its level seed), then the army. The
  alternative, the towns' stream (decision 75: "every weekly town draw"), would shift every later
  week's kind, garrison and Portal draws.
- **When:** the week's, in a new week's day start after the towns' week (its kind, the neutral
  garrisons, the Portals) and before the adventure's week step (the original's: after the towns'
  growth, before the dwellings); the fill reads only the pool, so its place moves no other state.
  The game's start's (`new_game_heroes`' taverns, 0x4C00C0, after the main towns' heroes): in
  `GameState::new` after the setup stream's last draw (the Portals') and the adventure's start of
  the game, before day 1's start-of-day step (*agent's call*).
- **Told** (T33b): each player's changed slots to that player.
- *As built* (T37a, decision 136): the release and the fill in `town::tavern` (`at_start`,
  `weekly`); per filled slot the original's order: the type offered, a new record (the next id,
  `setup::hero_details`' defaults with the experience drawn, then `Progression::record`'s seed),
  the backpack equipped (`h3_heroes::backpack_equipped`, ADR-0013 point 14.5), the spell points full,
  then `setup::starting_army` (empty for a type with more starting slots than an army) with
  `setup::wear_machines`: the same draws as the text's order. The week's after `grow_week`, before
  the adventure's week step; the neutral garrisons' week and the Portals stay after it, as built
  before (the text's parenthesis named them before it). The start's after the build's state is
  made (after the adventure's start of the game and of day 1, which read no tavern hero), before
  the first group's turn; it releases nothing (no slot is held).

### 64. The town's hire (TWN-TAV-003; `TownCommand::Hire { town, slot }`; `town_tavern_hire` 0x5C12E0)

- **`h3-towns`:** `TownCommand::Hire { town: TownId, slot: u8 }` (the fourth variant); a check
  `h3_towns::tavern(town, part)`: a standing building of role `tavern` (`has_tavern`), else
  `TownCommandError::NoTavern`; the owner's check as the build's (`NotOwner`); the price, the
  part's `hire_price`.
- **The game** (`town/tavern.rs`): the actor the town's owner, in its turn, no decision open
  (GAM-DEC-001); the slot 0 or 1 (`UnknownSlot`) holding a hero (`EmptySlot`); the player's heroes
  on the map below `most_heroes` (the garrison heroes not counted; TWN-ROW-003's `TooManyHeroes`);
  no visitor on the town's entrance (TWN-ROW-003's `EntranceHeld`, the adventure's visitor
  query); the treasury holding the price (`GameCommandError::CannotPay`, after the town's checks,
  as the build's). Each refusal typed: nothing changes, nothing is drawn.
- **The application** (infallible, ADR-0017 point 10; the original's order): the price paid; the
  adventure places the hero on the town's entrance as its visitor with its away record (ADR-0016's
  draft point 3); its type `Taken` by the player's colour, the colour's bit added to its mask
  (`hero_place`); the slot's returning mark cleared (`hero_place`'s flag clears bits 17 and 18);
  the guild's spells not taught until increment 5 (`TWN-STUB-15`, `town_heroes_learn_spells`
  0x5BE030); the replacement drawn into the slot (§65). No reveal (§59).
- **The legal commands** (GAM-DEC-001's set with no decision open): per own town with a standing
  Tavern in `TownId` order, after its recruitments, a `Hire` for each slot that passes every
  check. **Replay** (the adventure replay format, `cross-lane`): `town hire <town> <slot>`.

### 65. The replacement (TWN-TAV-004; `tavern_slot_replace` 0x4C86A0)

At once after a hire (the town's or the map's), the hired slot: the draw (§62) with the other
slot's hero's class to avoid and **any faction**, from **the hirer's stream** after the hire's
checks (ADR-0017 point 4); the type `Offered`, a new record's draws when it has none; its army
**one creature of its type's first starting stack** (GAM-SET-001's first stack at a count of 1,
no draw; `draw_starting_army` 0x4C9730 with its one-creature flag); its spell points full; **no
backpack equipping and no points set**: its points are its away record's until its owner's next
turn start (§68; T-16 for a record a leave refilled, §67). No type free: the slot stays empty.
**T-15** (a war machine as the first starting stack) is not reachable by construction: the core's
hero type keeps its war machines apart from its starting stacks, whose first is a creature
(GAM-SET-001's `HeroType::war_machines`); no ledger row (*agent's call*, decision 107's note
closed).

*As built* (T38a1, decisions 140 to 142, `TWN-Q22`): §64 and §65 as drafted in `h3-game`'s
`town/hire.rs` (`hire`, `replace`, `HiredPart`), `h3-towns`' `check_hire` (`NoTavern`; the
price) and the adventure's `src/town/hire.rs` (`check_tavern_hire`: the room, then the entrance;
`place_hired`: a hero's arrival, the tile's viewers). `UnknownSlot` and `EmptySlot` are the town's
refusals. The pool's `hire` empties the slot, clears its returning mark, takes the type by the
colour with its bit in the mask and hands the away record over (dropped from the pool: the turn's
derivation and the day's regain read only heroes off the map). A new record's draws are the fill's
(`tavern::record_of`), from the hirer's stream (`GameState::taverns_for`). The guild's spells are
`TWN-STUB-15`; the listing and the env's entry `TWN-STUB-19` (T38a2). The replay word `town hire
<town> <slot>` and the format v17 came with T38a1, not T38a2: the server journals every command
the wire takes in the replay text (`h3-server`'s `journal/whole.rs`), so the word and the bump go
with the wire's command.

### 66. The adventure map's Tavern (class 95; the kind `Tavern`; GAM-DEC-010; `map_tavern_visit` 0x5D7E90, `map_tavern_hire` 0x4D7890)

- **The kind** (ADR-0016's draft point 4; the adventure lane's paths, `cross-lane`): **visited
  from a neighbour** (VCMI's `blockVisit`), no owner, no mark, no draw; a step toward its entrance
  visits it and opens **the tavern decision** for the visitor's owner. A hero standing on its
  entrance hides it (a step meets or fights the hero) and a `Revisit` there visits nothing (the
  original skips case 95 when the visitor stands on the visited tile). Every player's hero
  visits (the original's computer uses its own code, E13). Its rule id is the adventure lane's,
  given by the building task.
- **The decision** (`Decision::Tavern { owner, hero, object }`): answered `Hire { slot }` or
  `Close`. A hire is checked as the town's (§64) **without** the visitor test and the town's
  checks, and is refused while any hero stands on the Tavern's entrance (`Occupied`; VCMI's, the
  original never reaches it); a refused answer is typed and leaves the decision open. The
  application as the town's, the hero placed on the Tavern's entrance, no guild spells; then the
  replacement (§65) and the decision **closes** (one hire a visit, the original's dialog).
- **The import** (map import, `cross-lane`): class 95 typed (the 93 Taverns on 47 maps, §1); the
  writer writes it. Which of the 47 maps import whole once the Den of Thieves is typed too is
  T33b's census.
- **Replay:** `answer tavern hire <slot>`, `answer tavern close`. Its view and the env (T33b).
- *As built* (T39a, decisions 149, 150): `ObjectKind::Tavern` (ADV-VIS-007; the base pack's
  `base:tavern`, `visit = "neighbour"`, not from above; part hash tag 45, the adventure part
  v22), class 95 imported (map import v26). Its visit's outcome `AdventureOutcome::tavern`, a
  `TavernVisit` (the hero, the type, the anchor, the visitable tile), opens `Decision::Tavern`
  (`GameState::open_tavern`: `TavernOpened` to the owner, whatever its gold and room; hash kind 9,
  game state hash v34). `Answer::Tavern(TavernAnswer::Hire { slot } | Close)`: the hire through
  `h3-game`'s `town::hire::map_hire` (the slot, `AdventureState::check_map_tavern_hire`'s room
  and tile, the price; `TavernError::{UnknownSlot, EmptySlot, TooManyHeroes, Occupied}`, then
  `CannotPay`), applied by the town hire's own `place` with the Tavern's tile, then
  `DecisionClosed`; `Close` only closes (the default answer). The listing `map_hires` (the
  `hire_slots` part, the tile free) then `Close`. `DecisionView::Tavern { hero, object_type,
  anchor }`. The env's v13 (the answers by role), the replay format v19, the protocol v73 (the
  answer, `TavernReject`, `Decision::Tavern`, `TavernOpened` folded whole). A hired hero stands on
  the Tavern's tile: a step toward it meets that hero, its `Revisit` is never listed (a
  neighbour-visited object's), and a setup still places no hero there (ADV-MAP-001).

### 67. A hero that leaves the game (TWN-TAV-006; GAM-BAT-003 and the capture's leave amended; `hero_leave_game` 0x4D9EC0; T-16)

Every path off the map: a battle's (a loser, a retreat, a surrender, both after a draw, a winner
left with created stacks only), the capture's empty garrison hero (`TWN-QUIRK-11`) and the
dismissal (§69). In the battle's end the game composes it after the adventure's end (which
removes the heroes and hands back their away records) and after the heroes' part (each side's
hero keeps the spell points it has left, its war machines' artifacts lost, GAM-BAT-002), so the
leave's own writes come last, as the original's after the battle.

1. **Its away record** back in the pool (ADR-0016's draft point 3): its battle bonuses and timed
   marks dropped (E08 decision 307), its formation and tactics flag kept.
2. **After a retreat or a surrender** (the battle's `Retreat(side)` or `Surrender(side)`, its
   side's hero): into its owner's tavern at once: a slot drawn, one draw of two from **its
   owner's stream** (its own command ended the battle), the other slot when that one's returning
   mark is set (so a third returning hero replaces one of two); that slot's hero, if any, goes
   `Free`; the type `Offered`, the slot holding it, its mark set. **A surrendered hero keeps its
   army** (its side's survivors, as the adventure's end left them), **a retreated hero keeps one
   creature** of its type's first starting stack; both keep their points left, the day's maximum
   and their spell points as the battle left them (TD-19, decision 104).
3. **Every other leave** (a loss, a draw, the capture's, the dismissal): the type `Free`; the
   army cleared to one creature of its first starting stack (unseen: a later draw replaces it);
   the spell points full (HER-MAN-001; through `h3-heroes`' change); the points left and the
   day's maximum **the land maximum the original computes over the cleared army**: from the empty
   slots' starting speed, 20, the land table's most (with the hero's Logistics and artifacts).
   **T-16** (`TWN-QUIRK-12`, decision 116): those points are seen only when the type is drawn as
   a replacement and hired in the same turn (a fill's points are set at the turn start first);
   built as the reading under a `fix-optional` flag (on in `faithful`; off, the land maximum of
   its one creature).
4. **What it keeps:** its record (level, experience, skills, spells) and the artifacts it holds
   then: a beaten hero those the winner had no room for (GAM-BAT-004), a retreated or surrendered
   hero all of them (the battle's capture returns at once), a dismissed hero all of them (decision
   105; row TWN-4-5 confirms the dialog strips nothing).
5. **A new record's points** (*agent's call*): the land maximum of the army its first draw gives
   it (the original sets every record's at the new game with its full starting army; only a
   replacement hired in its first turn could tell).

A defeated player's heroes and slots (`player_defeated`) are E10's. Told (T33b): `HeroRemoved`
as now, the owner's slot changes. `ADV-STUB-9` stays E10's whole: the Tavern fact of the retreat
facts is `has_tavern` over the owner's towns other than the battle's, ready, but only a computer's
hero reads the facts.

*As built* (T38b1, decisions 145, 146; the dismissal's leave is T38b2's): `h3-game`'s
`town/leave.rs` (new): `leave(state, rules, hero, how)` with `Leave::{Retreated, Surrendered(
survivors), Gone}` over the `HeroOnMap` the adventure took off, and `after_battle` (the side that
retreated or surrendered returns, every other is `Gone`; the owner's changed slots told by
`tavern::tell` against the views before, only for a recording sink and only when a hero returns).
The adventure hands the heroes back (`cross-lane`): `AdventureState::end_battle` returns the heroes
that left in the order given (`Vec<HeroOnMap>`, as each stood before the end: the army it fought
with, its formation, tactics flag and points; its marks gone, its boat freed), and
`release_town_hero` and `end_town_hero_battle` the garrison hero (`Option<HeroOnMap>`). The game's
`end_battle` composes the leave **last, after the winner's levels** (the original's leave call
0x4AE6D5 after its levels 0x4AE324), `free_hero_types` removed; the capture's empty garrison hero
leaves `Gone`. The pool's writes `HeroPool::leave` (the type free, the away record) and
`return_to` (the slot's hero free, the type offered and held, the mark set, the away record). A
`Gone` leave: one creature (`leave::one_creature`, the replacement's too), the record's spell
points written only when not full, the points `h3_adventure::day_points` over `&[]` with
`LEAVE_POINTS_OVER_CLEARED_ARMY` (an empty army counts as the fastest: the table's most, which the
original's speed 20 reaches in every pack's table) else over the one creature, `stale` off and
`full` on (a free type is no slot's: every draw sets `stale` again, and a replacement keeps these
points until its owner's turn start, so T-16 shows only for a replacement hired at once). A
return: the slot `range_inclusive(0, 1)` from the owner's stream, the marked slot's other; the
army the survivors (one creature when none: decision 146), the points kept and `stale` on, `full`
by the record's spell points. A hero that is not its type's record becomes it as it leaves unless
the type is offered (decision 146: the town tests' setups place two heroes of type 0). Point 5
built in the replacement (`hire::replace`: a new record's away points the land maximum of its one
creature). Tests: `town/tests/hire/leave.rs` (five: a loss under both settings of the flag, a draw
and a created-stacks winner, a retreat with its slot, view, turn start and rehire, a surrender
with the marks and without survivors, a repeated type's other hero beside an offered record), the capture's in `rows.rs`; the plans' battle oracle
(`h3-map`'s `check_heroes_after_battle`) taught the untold refill of a hero that leaves without
returning.

### 68. The tavern heroes over a turn (TWN-TAV-005; ADV-MOV-002's derivation; `next_player_turn` 0x4C6FE0)

- **At its owner's turn start**, after the player's heroes on the map and before its garrison
  heroes (the original's order; none draws, so the order is the reading's for the record), each
  tavern hero's day's maximum and points left set to its land points: `h3-heroes`' derivation
  (ADR-0013 point 13) over its details and its away army, as a hero on the map's. **Its spell
  points regain at the day's start** with every record of the store (`GameState::regain_mana`,
  HER-MAN-002, unchanged: decision 127 at T35's start amended TD-20's spell-point half, decision
  104, after batch TWN-4's code reading, survey §14); besides, they are set full at a draw and at a
  non-returning leave.
- **At its owner's turn end** (`EndTurn`, 0x4C705D after the victory tests) its two slots'
  returning marks are cleared. So a hero that came back in its owner's own turn goes at the next
  week's release, and one that came back after that turn ended (another player's attack) stays
  through the next day's start, a week's release included (TD-18).
- **At every day's start** (decision 126, T35's start; survey §14: the original's `new_day`
  0x4C80CF to 0x4C80FD) every player's two marks are cleared, after the taverns' week when one
  runs: a mark lasts until its owner's turn ends or the next day starts, whichever comes first;
  the week's release, before the clear, still spares a hero that came back after its owner's
  turn ended. The clear writes the pool's block only when a mark is set (a fork's day costs
  nothing more otherwise).
- *As built* (T37a): `town::tavern::start_turn` after the adventure's turn start (its heroes
  on the map and its garrison heroes; none draws), deriving only a stale hero's points (a fill's;
  nothing else changes what the derivation reads off the map, so the others' are its value
  already: `bench-check`, a derivation every turn start cost `day-start-builds-36` +11 %);
  `clear_marks` at `EndTurn` for the player and
  in the day's advance after the week's block for all, each writing only where a mark is set. No
  mark is set before T38b (a test helper sets one).

### 69. The dismissal (GAM-MAN-003; `Manage::Dismiss { hero }`; decision 112; survey §12)

- **The checks:** the actor's hero (`NotYourHero`), in its turn, no decision open (GAM-DEC-001);
  on the map (a garrison hero is not: the town screen offers no dismissal, `NotOnMap`); not
  standing on a town's entrance (`InTown`: the original's visitor test; **TD-22**, VCMI's button
  does not test it: the reading, decision 118); not the player's last hero on the map
  while it owns no town (`LastHero`). The loss condition's "lose this hero" waits for E10's
  scenarios (`TWN-STUB-16`). The adventure checks the map half (the entrance), the game the rest.
- **The application:** the leave (§67, a non-returning one): the adventure removes the hero (its
  boat freed, ADV-SEA-006; `HeroRemoved` to its owner and the players that see its tile), the type
  `Free` with its record and every artifact (decision 105). No draw.
- **The legal commands:** per own hero on the map in hero order, after its management commands,
  a `Dismiss` when it passes. **Replay:** `manage dismiss <hero>`. The env (T33b).
- *As built* (T38b2, decision 147): `Manage::Dismiss { hero }` (`h3-game`'s `manage.rs`,
  `cross-lane`) and `ManageError::{NotOnMap, InTown, LastHero}`; `GameState::manage` refuses it
  with `DecisionOpen` unless no decision is open (`Together::Alone`: a meeting, an army dialog or
  an exchange holds the turn) and hands it to the town lane's `town::leave::dismiss`: a garrison
  hero of the player's `NotOnMap`, then for the player's hero on the map `check_dismissal`
  (`InTown` by `StaticMap::town_at` on its tile, any town's; `LastHero` when the player has no
  other hero on the map and owns no town), then the adventure's `remove_hero((player, hero))`
  (`cross-lane`: `end_battle`'s removal made one private `remove_at`, shared), whose `None` is
  `NotYourHero` (another player's, a tavern hero, one out of the game), then `leave(..,
  Leave::Gone)`. `TWN-STUB-16` marks `check_dismissal` (the loss condition's hero). The listing:
  `legal_management` appends the dismissal last, outside a decision, for a hero on the map that
  `check_dismissal` passes (a garrison hero's listing has none). The replay word is **`dismiss
  <hero>`** (format v18; the agent's call: the format's other management words carry no
  `manage`), the protocol v72's `Manage::Dismiss` and `ManageReject::{NotOnMap, InTown,
  LastHero}`; `HeroRemoved` folds as a battle's (`a_dismissal_folds_as_observe_shows_it`). Tests:
  `town/tests/hire/dismiss.rs` (the leave with every artifact, its type free, one creature, full
  spell points, no draw, told as the removal; every refusal and the listing beside it; a meeting
  holding the turn), the adventure's `a_removed_hero_leaves_the_map_with_its_boat_as_a_beaten_one`,
  `h3-files`' word. The plans dismiss in one `Step::Manage` of sixteen (`dismissal`; `managing`
  leaves the dismissals out, so the other steps play as before); `check_management`,
  `check_conservation` (the record's spell points full untold, the army off the map) and
  `check_isolation` know it.

### 70. The map's hero settings (GAM-HER-001, TWN-SET-005 amended; `GameSetup`, the import `cross-lane`)

- **The customised types** (SoD's map-wide entries, §5): `GameSetup` gains `custom_types:
  Vec<(HeroTypeId, HeroDetailsSetup)>`, every flagged type's entry (the import from `h3-map`'s
  `custom_heroes`, which it merges into a placed hero's body today; the placed heroes keep theirs).
  A record made at a draw takes its type's entry over GAM-HER-001's defaults; so `Marshland
  Menace`'s 15 Fortress types are offered at 2 000 experience (their levels at load at their first
  draw, from that draw's stream). The setup checks an entry's type is the ruleset's and once.
- **The masks** (SoD's disposed heroes): the draw's count reads them (§62); a placement adds its
  colour's bit (§64).
- **Banned skills** do not touch a hired hero's starting skills (the reading; VCMI agrees):
  nothing to build; a test pins a type with a banned starting skill offered with it.
- **Not in the pool**, as built: `Out` (not allowed), `Prisoner`, the placed heroes' `Taken`.
- *As built* (T37b, decisions 137, 138; map import v25): `GameSetup::custom_types` (the import's
  entries that set a field the game keeps, a biography alone left out; the placed heroes' bodies
  still merged over them by the import); the setup checks each entry's type (the ruleset's, named
  once: `GameSetupError::CustomType`) and the entry on its own at experience 0
  (`CustomDetails`), before any draw, so the heroes' draws decide no validity. **Every hero the
  build makes** takes its type's entry under its own customisations
  (`HeroDetailsSetup::over`, field by field; idempotent for an imported placed hero): a random
  hero by its drawn type, a main town's hero (the code reading: `load_hero_body` 0x502374 to
  0x5023E7, `placed_heroes_setup` before `main_town_heroes`; VCMI agrees but for the random hero,
  TD-24). **A record at the build** (`town::custom_records`) for each customised type still
  free after the build's heroes, in type order, its id after them, its default experience in the
  build's pass, no army; its level seed in `Progression::new`'s pass, then its levels at load,
  so the order T37a left open is the build's (the seed first, which `faithful` reads). A draw
  finds the record made; no new state, the game state hash's encoding unchanged (v33).
  `Marshland Menace`: 13 or 14 records at level 3, the Fortress player's main town's and random
  heroes at 2 000 whenever they draw one of the 15 types. The fixture maps' engine-written entry
  (type 3) is placed on the duel; on `e08-army-12` and `e08-xl-144` it becomes a record, so the
  army map's two replays' hashes moved and the XL map's two reach workloads' games changed
  (re-baselined).

### 71. The hire's ids, versions and open points (for T33b)

- **Rules** planned: TWN-TAV-001 to -006 (a new area, the Tavern), GAM-DEC-010 (the tavern
  decision), GAM-MAN-003 (the dismissal), the adventure lane's kind `Tavern` (its id at the
  building task); amended TWN-SET-001 (the faction kept; a Random choice drawn at the first fill),
  TWN-SET-005 (`Offered`, the records, the away records), GAM-HER-001 and GAM-HER-002 (a record
  made at a draw), GAM-SET-001 (the one-creature form), GAM-BAT-003 and the capture's leave (the
  leave of §67), GAM-CAL-002 (the taverns' week), ADV-MOV-002 (the tavern heroes' points beside
  the heroes' on the map).
- **Stubs:** `TWN-STUB-15` (a hire's guild spells, increment 5), `TWN-STUB-16` (a dismissal of the
  loss condition's hero, E10); `ADV-STUB-9`'s reason updated when built. **Quirk candidate:**
  T-16. **Dispute:** TD-22.
- **Versions** (this half's): the town part v7; the game state hash; the adventure ruleset part
  and state hash (the kind `Tavern`); the map import (class 95, the setup's customised types);
  the adventure replay format (`town hire`, `answer tavern`, `manage dismiss`); shared:
  `StreamId::TAVERNS`. T33b adds the protocol's and the env's.
- **Open for T33b:** the views and events (the owner's two tavern heroes with their details,
  army and points; the hire, the replacement, a hero's return; a placed hero to the players that
  see its tile; the tavern decision's view); the env's entries (`Hire`, the tavern answers,
  `Dismiss`); the Thieves' Guild from a town's tavern and the map's Tavern and the Den of Thieves
  (class 97, §6); the gates (a taverns map and its replays, the pool's invariants as properties:
  a type in at most one slot, `Offered` exactly when in a slot, `Taken` exactly when on the map or
  in a garrison; the census of the 160 maps' first offers); batch TWN-4's rows (§9) placed; the
  building tasks and their order with E08's increment 13; the exit criteria; the checkpoint
  `TWN-Q18` (both halves' drafts, the taverns' stream, T-16, TD-22, the cut).

### 72. The information: scope, sources and choices (T33b)

The note's second half (decision 108; T33b kept whole, decision 114), written from the survey §6
and §13 (the Thieves' Guild read to its end by T33b: the rows' values, the places, the four
per-player parts, the window's clicks), §1 (the census), §9 (batch TWN-4), VCMI 1.7.5's
`GameStatistics.cpp` and `CGameState::obtainPlayersStats`, the manuals, and the code as it stands
(`h3-game`'s `observe.rs`, `town/view.rs`, `state/day.rs`; `h3-adventure`'s `answer.rs`,
`observe.rs`; `h3-sim`'s `adventure/action.rs`; `h3-protocol`'s `wire/game/fold.rs`).

- **This half builds:** the Thieves' Guild (§73), the Den of Thieves (§74), the views and events
  of both halves (§75), the env and the protocol (§76); then the whole increment's ids, versions,
  quirks and stubs (§77), the gates and batch TWN-4 placed (§78), the building tasks (§79), the
  exit criteria (§80), the risks (§81).
- **Not this increment's** (beyond §59's): the guild's personality part (the computer's tactic,
  E13; `TWN-STUB-18`); the obelisks row's values (increment 6 places the obelisks; `TWN-STUB-17`);
  a defeated player's column (E10); the guild's hints and texts (presentation).
- **Read by T33b** (survey §13; six address-map entries): `thieves_guild_fill` 0x5DDA10 to its
  end, `thieves_guild_row_value` 0x5DEE70 whole, `thieves_guild_rank_sort` 0x5DF160,
  `hero_worn_count` 0x4D9070, `army_power` 0x427690, `thieves_guild_proc` 0x5C9930 at its clicks.
  New: **TD-23** (the best hero by army power and the towns' armies in the army row and the best
  creature, against VCMI's experience and heroes only) and **T-17** (the artifacts row ranks the
  backpacks alone: `hero_worn_count`'s argument 1 counts nothing), both settled at `TWN-Q18`
  (decisions 117, 118).
- **The agent's calls** (each confirmed at `TWN-Q18`, decisions 119 to 122): the owner's tavern
  heroes always in its observation (§75); the guild a query, a Den's view an event (§73, §74); the
  places' order within a tie by player id (§73); the dismissal without an env entry (§76).

### 73. The Thieves' Guild (TWN-TAV-007 to -009; `h3-game`'s `town/thieves.rs`, new; `thieves_guild_fill` 0x5DDA10)

- **Where:** a pure function of the game, `h3_game::thieves_guild(state, rules, viewer) ->
  Option<ThievesGuildView>` (the town lane's `src/town*`; exported from `lib.rs`, `cross-lane`),
  and `thieves_guild_at(state, rules, viewer, level)` for a Den's visit (§74). Nothing is stored:
  the original's window is a snapshot when it opens.
- **When it answers** (TWN-TAV-007; decision 119): while the viewer's turn runs
  (`GameState::is_acting`) and it owns a town with a standing Tavern (`has_tavern`), or its open
  decision is a tavern decision (§66); otherwise none. The level: its towns with a Tavern, counted
  to 5 (`player_tavern_count`; the popup's `tavern_level` stops at 2: a sibling counting further,
  `town/view.rs`); at a map Tavern without a town tavern 0. A Den's level is everything (5).
- **The rows** (TWN-TAV-008): shown by the level, 2 at 0 or 1, 4 at 2, 6 at 3, 8 at 4, all 9 from
  5, in the original's order: 0 towns (owned, the adventure's owners), 1 heroes (on the map; a
  garrison hero is not), 2 gold, 3 wood and ore, 4 mercury, sulfur, crystal and gems (the
  treasury), 5 obelisks visited (0 for every player until increment 6, `TWN-STUB-17`), 6
  artifacts (over its heroes on the map, the backpacks' counts alone under `TWN-QUIRK-13`, T-17's
  `fix-optional` flag, decision 117: on in `faithful`; off, the worn and the backpack's
  artifacts, VCMI's), 7 army strength
  (over its heroes on the map the power the wandering monsters read, ADV-MON-007's
  `h3_adventure::power`, made public `cross-lane`; plus each of its towns with a visitor or a
  creature in the army it shows, the garrison hero's or its own, that army's plain value, the
  32-bit sum: **TD-23**), 8 gold income (the day's gold of GAM-RES-003, -004 and the towns'
  TWN-INC-001, -002, as the day's start would pay it now; factored out of
  `GameState::pay_income`, `cross-lane`; no difficulty or handicap). A row is its **places**: the
  players in the game sorted by value from the highest, equal values sharing one place; within a
  place in player id order (*agent's call*: the original's exchange sort keeps no order, and the
  order is not a rule). Every player of the game is in it (defeat is E10's).
- **The parts** (TWN-TAV-009), per player in id order: from level 1 its **best hero**: over its
  heroes on the map in the adventure's order of its heroes the greatest power (as row 7's), a
  strictly greater one replacing, none with no hero or every power 0 (**TD-23**: VCMI takes the
  most experience); its type; from 2 that hero's four primary skills as derived
  (`h3_heroes`' primary skills with every bonus, ADR-0013 point 13), each clamped to 0 to 99; from
  4 its **best creature**: over the armies its towns show (in town order) and then its heroes' on
  the map (in the same hero order), the creature with the greatest AI value, a strictly greater
  one replacing (**TD-23**: VCMI's heroes only). From 3 the original shows the player's tactic:
  not shown (`TWN-STUB-18`; the computer's play is E13's).
- **Tests:** each threshold (0 to 5 and a Den); a tie's shared place; the artifacts row with worn
  artifacts only (`TWN-QUIRK-13` both ways); a garrison hero and a town's own garrison in the army row and as the best
  creature; the first of two equal heroes; none for a player without heroes; a primary skill past
  99 clamped; the query's none outside the turn and without a Tavern; the map Tavern's level 0.
- *As built* (T39b1, decisions 151 to 153): `town/thieves.rs` (new): `h3_game::thieves_guild`
  and `thieves_guild_at(state, rules, level)` (the viewer drops out: the view is the same for every
  viewer of a level), `GUILD_LEVELS` and `DEN_LEVEL` (5), `ThievesGuildView { level, rows, players
  }`, each row a `GuildRanking { row: GuildRow, places }` (**each player's place by id**, 0 the
  first, dense: the next lower value the next place), each player a `GuildPlayer { hero:
  Option<GuildHero { hero_type, skills }>, creature }`. **The resource rows are data** (decision
  152, over the agent's core order): the town part's `guild_resources`, three lists of core
  resources (part **v8**; the base pack's from the gap-fill's `RESTYPES.TXT` indexes 6; 0, 2; 1, 3
  to 5, read at 0x5DEF3F to 0x5DEF93: player +0x9C is the treasury; the synthetic pack's coin,
  shard, none), each resource in at most one row. Made public `cross-lane`:
  `h3_adventure::army_value` and `power` (ADV-MON-007), `AdventureState::upper_bars` (the town
  lane's file), `h3_heroes::primary_skills` (HER-ADV-001), `GameState::day_income` (`pub(crate)`,
  factored out of `pay_income`, without the week's resources). Read at T39b1: **the best creature
  starts from 0 too** (0x5DE992, `jle`) and skips a stack of count 0; the towns in the player's
  town list's order (TownId order here). The tavern count to 5 is `view.rs`' `tavern_count`
  beside the popup's `tavern_level`. Sums saturate in u64. Flag off of `TWN-QUIRK-13`: every worn
  position's artifact and the backpack's (VCMI counts a combination's locked slots too). Tests
  `town/tests/hire/thieves.rs` (nine, on the hire tests' game; the guild's rows checked by moving
  the one test resource between them: the adventure test pack wants a mine per core resource).

### 74. The Den of Thieves (class 97; the kind `DenOfThieves`; `visit_object`'s case 97)

- **The kind** (the adventure lane's id at the building task; ADR-0016's draft point 4): an
  entered object (VCMI 1.7.5's template: visited from the sides and below, the entrance
  passable), no owner, no mark, no parameter, no draw; every visit the same (a step onto the
  entrance, a `Revisit` too).
- **The visit** (the game's outcome, ADR-0017's draft point 10): `thieves_guild_at` at everything
  for the visitor's owner, told as `ThievesGuildSeen { player, view }` to that player alone (§75);
  the hero's move ends on the entrance as at any entered object. The original's message (ADVEVENT)
  is presentation.
- **Import and writer:** class 97 mapped to the kind (`codes.rs`, the adventure lane's,
  `cross-lane`), its 227 objects on 107 maps; the writer's plain object for the fixtures (as
  T119's banks). A map with a Den and no Tavern loses no blocker beyond it.
- **Tests:** the visit's view at everything whatever the visitor's taverns; a second visit the
  same day; a `Revisit`; the event told to the owner alone.
- *As built* (T39a, the kind only; decision 149): `ObjectKind::DenOfThieves` (ADV-VIS-008; the
  base pack's `base:den_of_thieves`, `visit = "enter"`, not from above; part hash tag 46), class
  97 imported (map import v26); a step onto it ends the move there and its `Revisit` is listed;
  its visit gives nothing yet (`TWN-STUB-20`, `h3-adventure`'s `visit`): T39b adds the outcome and
  `ThievesGuildSeen`.
- *As built* (T39b2, decisions 154, 155): the visit's outcome a flag,
  `AdventureOutcome::thieves_guild` (`h3-adventure`'s `visit`, `cross-lane`), which the game's
  `settle_outcome` tells after the visit's other effects through `town::thieves::tell_den`:
  `thieves_guild_at(state, rules, DEN_LEVEL)` whatever the visitor's Taverns, as §75's events.
  Every visit alike (a second step onto it the same day, a `Revisit`); a sink that keeps nothing
  is told nothing and nothing is computed (`Sink::RECORDS`; the state the same). `TWN-STUB-20`
  lifted, ADV-VIS-008's summary amended. **The census:** the 98 golden replays play 12 maps with
  no Den (and no map Tavern): none steps onto one, nothing re-recorded. Tests
  `town/tests/hire/map.rs` (three: the guild at everything to red alone after the step, its parts
  in order, blue told nothing; a `Revisit` and a second step alike; a `NoopSink` visit's state).

### 75. The views and events (both halves; `docs/06-security.md`'s town section by the `shared/` task)

- **The tavern heroes** (decision 119): `GameObservation::tavern`, the viewer's two slots, each
  `Option<TavernHeroView>`: the hero's id, its type, its details (`Arc<HeroDetails>`), its army by
  slot, its formation and tactics flag (what the tavern dialog and the view-only hero screen
  show); no points and no returning mark (the original shows neither; the hired hero's points
  come by `MovementChanged`). Shown whether or not the viewer owns a Tavern (they tell nothing of
  another player; gating them would need an event at each Tavern built or lost). Never another
  player's.
- **`TavernOffered { player, slot, offer }`** (seen by `player`): a slot's new view, or none when
  emptied, after each change: the week's release and fill (each changed slot in player then slot
  order, after the towns' week events, before the adventure's week step), a hire's replacement
  (after `HeroHired`), a hero's return (after the battle's `HeroRemoved`, its slot; the slot's
  former hero freed by the same event). The game's start's fill is the first observation's.
- **A hire:** `ResourcePaid` (the price), then **`HeroHired { player, hero, hero_type, slot,
  pos }`**, seen by the owner and every player that sees `pos` (a step's `seen_to`, ADV-FOG-004,
  -005); the slot projected to the owner only. A viewer's fold adds the hero on the map as its
  view shows it (owner, tile, type; no boat: a hire is on land); the owner moves the slot's view
  into its heroes (details, army, formation, tactics) and takes the points from the command's
  `MovementChanged` (`OwnHeroes::tell`). Then the replacement's `TavernOffered`; for a town's hire
  the town's views do not move (the visitor is the adventure's hero view).
- **The tavern decision:** **`TavernOpened { player, hero, object_type, anchor }`** (seen by the
  owner) and `DecisionView::Tavern` with the same three: whole, so no `SRV-STUB-5` gap (the offers
  are the observation's tavern heroes). Its answers: a hire as above, then `DecisionClosed`; a
  `Close`, `DecisionClosed`.
- **A leave:** the battle's `HeroRemoved` as now (and the capture's); a dismissal's
  `HeroRemoved` (its owner and the viewers of its tile, ADR-0016's draft point 6). A return: its
  owner's `TavernOffered` after the battle's end events. A leave without return tells nothing
  more (the pool is hidden).
- **The Thieves' Guild:** the query's view (§73) and the Den's `ThievesGuildSeen { player, view }`
  (seen by `player`). `ThievesGuildView`: the level, the players, the rows (each its kind and its
  places), per player the best hero (its type and, from 2, the four skills) and the best creature.
- **`docs/06-security.md`'s town lines** (the `shared/` task): a player's tavern heroes are its
  own; a hired hero is seen as any hero on the map; the Thieves' Guild shows the ranks (never
  the values), the best hero's type and skills and the best creature of every player in the game,
  by the viewer's taverns in its turn or at a Den's visit, as the original's window does;
  the pool's statuses, the away records and the returning marks are never shown.
- **The fold gate** (`h3-protocol`'s `game_fold`): the tavern heroes fold from `TavernOffered`,
  `HeroHired`, `HeroRemoved`; the guild is not in the observation. A returning hero's points
  (kept from the battle) are not in the tavern view, so nothing waits on them. **From decision
  127** (T35's start) a tavern hero's spell points, in its details, regain at the day's start with
  no event, as an own hero's on the map (`regain_mana` tells nothing): the gate's `spell points`
  part (`SRV-STUB-6`) covers the tavern views too, or T37a tells both (with the server lane).
- *As built* (T37a, decisions 134, 135): **the view is the tavern's held hero screen's**
  (`tavern_dialog_proc` 0x5D7D7D opens `hero_screen_show` as a held popup, 0x4E1969; the manual and
  VCMI's `CTavernWindow` agree): `TavernHeroView { hero: TavernHero, army, skills, artifacts }`,
  the head (`Copy`) the id, type, level, experience, primary skills, spell points, the backpack's
  count (`carried`), formation and tactics; the artifacts worn by position, then the backpack's
  first `TAVERN_BACKPACK_SHOWN` (5) places; **no learned spells, no offered levels** (not
  `Arc<HeroDetails>` as drafted). `GameObservation::tavern: [Option<TavernHeroView>; 2]`.
  **`TavernOffered { player, slot, hero: Option<TavernHero> }`** then **`TavernPartSeen { player,
  slot, part: TavernPart }`** per stack, skill and artifact in that order (`GameEvent` stays
  `Copy`), each slot whose view the week changed, for a recording sink only. Protocol **v70** (the
  wire's `TavernHero`, `TavernHeroHead`, `TavernPart`, both events, their fold); the fold gate's
  part **tavern heroes** exact, its **spell points** part (`SRV-STUB-6`) covering the tavern views'
  too; `h3-map`'s told-views property folds them likewise. For T38a: a hire's owner learns the
  hired hero's spells and offered levels from no tavern view, so its `HeroHired` (or parts after
  it) must tell them for the fold to stay exact.
- *As built* (T38a1, decision 141): **`HeroHired { player, hero, hero_type, slot: Option<u8>, pos,
  seen_by }`**, the slot blanked by the projection for another viewer, then to the owner alone
  **`HiredPartSeen { player, hero, part: HiredPart }`** per part its slot's view lacked:
  `Details { offered, positions }` once (the levels last offered and the worn positions' count, the
  details' `worn` holding one entry per position), `Carried` per backpack artifact past
  `TAVERN_BACKPACK_SHOWN`, `Spell` per learned spell; then the slot's `TavernOffered` (its
  replacement or its emptying, `tavern::tell` against the views before). Protocol **v71** (the
  command, the three refusals, both events; the fold adds the hero to every viewer's heroes on the
  map and, for the owner, moves the slot's view into its heroes, the worn positions grown to the
  told count). `h3-protocol`'s `a_hire_folds_as_observe_shows_it` plays a hire on three golden
  games (none of them hires: `TWN-STUB-19`) and compares every viewer's fold with `observe`.
- *As built* (T39b2, decisions 154, 155): **`ThievesGuildSeen { player, level }`** then
  **`GuildPartSeen { player, part: GuildPart }`** per part (`GameEvent` stays `Copy`), in
  `ThievesGuildView::parts`' order: `GuildPart::Place { row, of, place }` for each shown row's
  players by id, the rows in order, then `GuildPart::Player { of, shown: GuildPlayer }` per
  player; seen by `player` alone (`project`). The engine folds no parts. Protocol **v74**: the
  wire's `GuildRow`, `GuildRanking`, `GuildHero`, `GuildPlayer`, `ThievesGuild` (with
  `ThievesGuild::at(level)` and `see(part)`, a client's fold of the parts: a place of a row other
  than the last begins that row), `GuildPart`; both events mapped and dropped by the fold (the
  guild is not in the observation); `GameQuery::ThievesGuild` answered by
  `GameAnswer::ThievesGuild { guild }` (`map::game::query::answer`: `h3_game::thieves_guild` for
  the asking seat's player). Tests: `game_event`'s every event (46 kinds) and
  `a_dens_guild_on_the_wire_is_the_guild_its_parts_make` (a level-5 view's parts map back, round
  trip, and `see` gives the query's mapping), the fold's no-change list, `game_query`'s
  `the_thieves_guild_is_the_engines_for_the_asking_player` (every golden's first 30 commands, each
  player and one beyond), the wire's message round trips. `h3-map`'s told views take both events
  as the viewer's own.

### 76. The env and the protocol

- **The action encoding** (the AI lane's, `cross-lane`; ADR-0017's draft point 8): v12 appends
  `HIRE + s`, s < 2 (the town by its entrance in the tile head, the building head 0; one action per
  listed `Hire`); v13 maps the tavern decision onto the answers by role (`Hire { slot: 0 }` the
  first, `Hire { slot: 1 }` the second, `Close` the closing role; no new entry), or one bump with
  v12 when they land together. **No entry for `Dismiss`** (decision 120: a management command,
  E08 decision 99, as the army commands); a client's command. The guild is a query the AI lane
  may read; no observation encoding moves (the adventure's observation is the game's struct).
- **The legal commands** (`h3-game`'s `legal.rs`, `cross-lane`): after a town's builds and
  recruitments, its `Hire`s by slot; while a tavern decision is open its two hires that pass and
  its `Close` alone; a `Dismiss` per own hero on the map that passes, after its management
  commands (§69).
- **The protocol** (the server lane's, `cross-lane`, in the tasks that add each; the server
  lane's inbox told at the checkpoint): the commands `TownCommand::Hire`, `Manage::Dismiss`, the
  tavern answers; the events `TavernOffered`, `HeroHired`, `TavernOpened`, `ThievesGuildSeen`;
  the observation's `tavern` and `DecisionView::Tavern`; a request for the guild and its answer
  (the server answers it from the current state for the asking seat; a client's window is the
  answer). Each bumps `PROTOCOL_VERSION` at its task.
- **The replays** (`h3-sim`'s `replay.rs`, the adventure lane's, `cross-lane`): `town hire <town>
  <slot>`, `answer tavern hire <slot>|close`, `manage dismiss <hero>` (format bumps at T38a, T38b
  and T39a, or fewer when they land together).

*As built* (T38a2, decisions 143, 144): the listing in `h3-game`'s `town/offer.rs`
(`legal_builds`: each own town's builds, recruitments, **hires**, then hero moves) through
`town/hire.rs`' `hire_slots` (once a listing: the part's price in the treasury, room below the most
heroes, `AdventureState::has_room`; the slots holding a hero) and `legal_hires` (a town: its
entrance free, `AdventureState::entrance_free`, since E09 T49a2 the listing's `town_entrance_hero`, then its standing Tavern), the same conditions as
`hire`'s checks, the cheapest first; `legal_commands` folds its chain into a list sized 16 (the
bench, the town lane's `TRAPS.md`). The env's **v12**: `TAVERN_HIRE + s` (35, 36; `TAVERN_SLOTS`;
`Action::tavern_hire`), masked and decoded as the hero moves between the bars (the town's entrance
in the tile head, the building head 0). The plans: `town_builds` probes the hires of slots 0 to 2
after the recruitments, `check_town_hire` reads a listed hire apart (the actor's tavern view's
hero on the entrance with the view's army, the price, `HeroHired`), `Step::Town` hires in one step
of eight; the oracles taught: the isolation (a hired hero joins), the conservation (the price, the
army from the slot's view, the refill's record), the told views (`HeroHired` shows the hero on its
tile; the owner's slot view folded with its `HiredPartSeen`s is its details whole, `check_hired`),
the hygiene (a hire told ends the comparison). No protocol, replay or hash version moved
(`TWN-STUB-19` lifted).

### 77. The increment's ids, versions, quirks and stubs

- **Rule ids** (`town.toml`): TWN-TAV-001 the draw, -002 the fills and the week, -003 the town's
  hire, -004 the replacement, -005 the tavern heroes over a turn, -006 a hero that leaves the game,
  -007 the Thieves' Guild's level and when it answers, -008 its rows, -009 its per-player parts;
  `game.toml`: GAM-DEC-010 the tavern decision, GAM-MAN-003 the dismissal; the adventure lane's
  kinds `Tavern` and `DenOfThieves` (their ids at T39a). Amended: TWN-SET-001, TWN-SET-005,
  GAM-HER-001, GAM-HER-002, GAM-SET-001, GAM-BAT-003 and the capture's leave, GAM-CAL-002,
  ADV-MOV-002 (§71), ADV-MON-007 (its power shared with the guild).
- **Versions** (each bumped by the first task that needs it, the number the next at merge):

  | Version | Change | Task | Owner |
  |---|---|---|---|
  | town ruleset part v7 | the tavern weights, the hire's price | T36 | town |
  | game state hash | the pool's `Offered`, ids and away records, the players' factions and slots, the taverns' stream | T37a | adventure, `cross-lane` |
  | map import | the customised types (T37b); classes 95 and 97 (T39a) | T37b, T39a | adventure, `cross-lane` |
  | adventure ruleset part | the kinds `Tavern`, `DenOfThieves` | T39a | adventure, `cross-lane` |
  | adventure replay format | `town hire`; `manage dismiss`; `answer tavern` | T38a, T38b, T39a | adventure, `cross-lane` |
  | env action encoding v12, v13 | `HIRE` + 2 (tile head); the tavern answers by role | T38a, T39a | AI lane, `cross-lane` |
  | protocol | the commands, events, views and the guild's request | T37a, T38a, T38b, T39a, T39b | server lane, `cross-lane` |
  | shared | `StreamId::TAVERNS`; the flags of `TWN-QUIRK-12`, `-13` (no version moves: the profiles' ruleset hashes do) | T35b | shared |
  | town state hash, adventure state hash | none (nothing of the tavern is the town's; the kinds hold no state) | — | — |

- **Quirks** (`quirks-town.md`, recorded by the tasks that build them): `TWN-QUIRK-12` T-16
  (`fix-optional`, on in `faithful`; T38b) and `-13` T-17 (`fix-optional`, on in `faithful`;
  T39b), decisions 116 and 117; their flags shared (T35). **Stubs:**
  `TWN-STUB-15` (a hire's guild spells, increment 5), `-16` (the loss condition's hero at a
  dismissal, E10), `-17` (the obelisks row, increment 6), `-18` (the guild's personality, E13);
  `ADV-STUB-9`'s reason updated at T38b. **Disputes:** TD-22, TD-23. **Inbox notes** after the
  checkpoint: the adventure lane (the `cross-lane` changes in its paths, §71 and above), the AI
  lane (v12, v13), the server lane (the protocol's additions).

### 78. The gates and batch TWN-4

- **Unit tests** per rule: the draw (the weights, slot 0's own faction, the other's class
  avoided, the masks, an empty slot), the fills (the full army, the backpack equipped, the spell
  points), the week's release sparing a returning offer (TD-18), the hire's refusals (each typed,
  the state's hash unchanged), the replacement's one creature from the hirer's stream, each leave
  (its army, points and spell points; T-16's flag both ways), the return's slot (TD-19), the turn's
  points and the mark's clearing (TD-20), the dismissal's five refusals (TD-22), the map's
  customised types, the guild (§73's list), the Den (§74's).
- **Properties** (in `h3-map`'s plans, as increment 3's): a hero type in at most one slot over all
  players; a type `Offered` exactly when a slot holds it; `Taken` exactly when its hero is on the
  map or in a garrison; every slot's hero has a record; a player's heroes on the map never pass
  `most_heroes` by a hire; a refused hire or dismissal leaves the state's hash; a fork's week is
  its parent's (the taverns' stream forked with the state).
- **A taverns map** (the builds map grown, or a new fixture: the task's call): two players with
  Taverns (one a Castle with the Brotherhood), a map Tavern, a Den of Thieves, wandering monsters
  to flee from and to surrender to, few free types (so a dismissed type comes back). **Scripted
  replays:** four weeks of fills; the town's hire and the map Tavern's with their replacements; a
  retreat and a surrender in the owner's own turn and in another's (TD-18); a dismissal and the
  type drawn again (T-14's reading); a Den's visit. Golden replays re-recorded where a hash moves.
- **The census** (`h3-map`'s `original_data`): the 160 maps' first offers (every player's two:
  slot 0 of its faction where one weighs, no type twice, every army full), the customised types'
  records (`Marshland Menace`), and the maps that import whole once classes 95 and 97 do.
- *As built (T40a, decisions 156 to 158):* the census (`original_data`'s `map_taverns`,
  `played_taverns`, `fills`, `pool_holds`, `customised`) types every class 95 and 97 object as its
  kind (93 on 47 maps, 227 on 107, the survey's) and plays every game by `EndTurn` alone to day
  29: the start's fill and every week's by the census's own walk of the draw (what was free at
  each draw: the free types now with every slot of the player's from that one on and of later
  players; the masks; the class weights, slot 1's avoidance, slot 0's own faction), each hero its
  type's record with full spell points and a starting army; the pool after every `EndTurn`; the
  customised types' records (17 on the maps, 13 off the map at level 3 with their 2 000). The
  properties: `check_invariants` ends with `check_pool` (the same statuses against the slots, the
  map, the garrisons and the store), `a_forks_week_is_its_parents` asserts the week's fill and the
  slots of both. Both read the pool through `GameState::hero_status`, `player_tavern` and
  `hero_record` (decision 158). A refused hire or dismissal leaving the state was T38a2's and
  T38b2's (`check_town_builds`, the dismissal's probes), the room by a town's hire
  `check_town_hire`'s; the map Tavern's hire, the taverns map, its replays and the benchmarks are
  T40b's.
- *As built (T40b, decisions 159, 160):* a new fixture, `e09-taverns-36.h3m` (the builds map
  kept: its replays name it by content hash), written by the writer and its allowed heroes set in
  the raw model (six free types: two of each player's faction's classes, two alchemists); six
  scripted replays `taverns-{weeks,hires,retreats,surrenders,dismiss,den}-s1`, each with
  `taverns_map.rs`' test of its commands (`crates/h3-cli/tests/adventure_replays.rs` lists what
  each shows). A surrender needs an enemy hero (BAT-FLE-002): the map's golems are retreated
  from, the surrenders made to blue's heroes; a retreated hero comes back with one creature of
  its first starting stack, a surrendered one with its army (TWN-TAV-006); blue's hire with no
  type free leaves its slot empty. The properties' plans (`tavern_plan`: `plan()`'s, then
  `Plan::taverns`, one or two map Taverns or Dens near the heroes in one plan of three; the cover
  tests keep `plan()`'s cases, which a draw inside it shifted, failing six of them); `check_legal` lists and probes the tavern decision's answers
  (`tavern_answers`, `check_map_hire`: the hire's tile, room, army, price, `HeroHired`, the
  decision closed; an unlisted slot refused with the hash), `check_decisions` opens it at
  `TavernOpened` and closes it as told, `managed` holds no army while it is open,
  `check_conservation` pays the map hire's price. `check_isolation` needed nothing (a hired hero
  is any `HeroHired`'s), nor did a stand oracle (`check_stand` is the setup's; a hired hero walks
  off the Tavern's tile as off any); `check_pool` held over every hire. The cover test
  `the_plans_hire_at_map_taverns`; the shifted cover cases met a Whirlpool taking creatures a
  guard's battle raised (`check_heroes_after_battle` subtracts the loss). Benchmarks: the new
  `week-start-taverns-36` (1 833 609 Ir, master's code); the guild's query timed once in release:
  1.9 µs on the taverns map, 7.3 µs on `Myth and Legend` (8 players, 21 towns).
- **Benchmarks:** the week workload (`month-start-builds-36` or the taverns map's) re-baselined
  for the fill's draws; the step's legal commands with the hires; the guild's query timed once
  (not on the step path).
- **Batch TWN-4** (survey §9's eight rows, row 7 with §13's records for TD-23 and T-17): on the
  capture harness as TWN-1 to TWN-3 (T34), its verdicts into the drafts at the `shared/` task. What
  the harness lacks (survey §9): the reads of the player record's offers (+0x28, +0x2C), a hero's
  +0x105, the pool's states (game +0x4DF18), the guild window's places and parts; a retreat and a
  surrender in a recipe (the battle lane's harness has both).

### 79. The building tasks (the whole increment; the order with E08's increment 13)

The tasks continue the epic's ids; each task's cut confirmed at its start. **E08's increment 13
runs at the same time** (T120 to T126, its design note part D §20): every `cross-lane` number is
the next at merge, the second to merge rebases and bumps again (the town lane's `TRAPS.md`);
T35 and E08's T122 both amend ADR-0016 and -0017; T37b and T39a import after E08's T123 (map
import v24, the bank kinds); T38b's leave and E08's T124b both end battles in `state/battle.rs`.

| Round | Tasks | Content |
|---|---|---|
| 2 | **T34** | Batch TWN-4 (§78): the spec and template (the writer's Tavern and Den for the fixture, `cross-lane` as T119's banks), the harness's reads and the retreat and surrender steps, the recipe, the runs, the scoring. Needs the original data and the client lane's harness set-up. Cut at its start |
| 3 | **T35** | Cut in two at its start (decision 125): **T35a** the docs, **T35b** the code. `shared/` (`cross-lane`): the four drafts applied with T34's verdicts; `StreamId::TAVERNS`; the quirk flags of `TWN-QUIRK-12` and `-13` (`h3-content`, pack format) with the packs' artifacts; the glossary (the tavern heroes, the away record, the returning mark, the Thieves' Guild, the Den of Thieves); `docs/06-security.md`'s town lines (§75). In merge order with E08's T122 |
| 4 | **T36** | The town part (the town lane's paths; the harvest `cross-lane`): the tavern weights and the price (part v7, TWN-TAV-001's data), `HCTRAITS.TXT`'s tavern columns harvested, the packs |
| 5 | **T37** | The pool in the game (`cross-lane`): T37a the pool's `Offered`, the records at a first draw, the away records, the players' factions and slots (game state hash), the draw and the fills from the taverns' stream (TWN-TAV-001, -002; GAM-CAL-002), the turn's points and marks (TWN-TAV-005), `GameObservation::tavern` and `TavernOffered`, the protocol, the replays re-recorded, the week's bench; T37b the map's customised types (GAM-HER-001, TWN-SET-005; `GameSetup`, the import) |
| 6 | **T38** | The hire and the leaves (`cross-lane`): T38a the town's hire (`TownCommand::Hire`, TWN-TAV-003), the adventure's placement, the replacement (TWN-TAV-004), `HeroHired`, the legal commands, `town hire`, the env's `HIRE` (v12), the protocol; T38b a hero that leaves the game (TWN-TAV-006; GAM-BAT-003, the capture's leave; T-16), the returns (TD-18, TD-19), the dismissal (GAM-MAN-003, TD-22, `manage dismiss`), the protocol |
| 7 | **T39** | The map's objects and the information (`cross-lane`; after E08's T123): T39a the kinds `Tavern` and `DenOfThieves` (the adventure part, classes 95 and 97 imported and written), the tavern decision (GAM-DEC-010, `TavernOpened`, `answer tavern`, the env's answers v13); T39b the Thieves' Guild (TWN-TAV-007 to -009: the query, the Den's `ThievesGuildSeen`, T-17, TD-23, `TWN-STUB-17`, `-18`; the power and the day's gold made public), the protocol's request |
| 8 | **T40** + **T41** | T40: the taverns map and its scripted replays, the census, the properties, the benchmarks, `--with-original-data` (§78). T41: the audit against the exit criteria; increment 4 closed; increment 5 (the mage guild and the services) refined |

*Agent's calls:* batch TWN-4 before the `shared/` task (its verdicts go into the ADRs, as TWN-3's
did); the town part first among the builds (no wait on E08); the pool before the hire (a hire
takes an offer); the leaves with the hire (both write the pool's statuses; T38b after T38a so a
hired hero can leave in the replays); the map's objects after E08's import bump; the guild last
(it reads every other part, nothing reads it).

### 80. Increment 4's exit criteria (accepted at `TWN-Q18`, decision 121; the epic's list)

1. The town part's tavern weights and the hire's price compile from the packs (part v7); each
   player holds its faction and its two slots; the pool's `Offered`, a type's record made at its
   first draw and kept by type, the away records (TWN-SET-001, -005).
2. The draw by the weights (slot 0 its faction's classes where one weighs, the other offer's class
   avoided, a free type the colour may take, uniform; an empty slot when none) from the taverns'
   stream; the start's and every week's fill with the full starting army, the backpack equipped
   and the spell points full; the week's release sparing a returning offer (TWN-TAV-001, -002;
   GAM-CAL-002; TD-18).
3. A town's Tavern hires onto a free entrance within `most_heroes` for the price, the map's
   Tavern (class 95) through the tavern decision onto its visited tile; each refusal typed and
   changing nothing; the slot's replacement drawn from the hirer's stream with one creature
   (TWN-TAV-003, -004; GAM-DEC-010).
4. A hero that leaves the game keeps its record by type; a retreated or surrendered hero comes
   back into its owner's tavern at once (its slot drawn, TD-19; its army as read; its points and
   spell points kept); every other leave frees the type with full spell points and the cleared
   army's land maximum under `TWN-QUIRK-12` (TWN-TAV-006; GAM-BAT-003).
5. The tavern heroes' points at their owner's turn start, their spell points regained each day
   as every record's (decision 127), the returning marks cleared at its turn's end and at every
   day's start (decision 126) (TWN-TAV-005; TD-20).
6. The dismissal with its refusals (TD-22) frees its hero with every artifact (GAM-MAN-003;
   decision 105).
7. The map's customised hero types reach a record made at a draw (`Marshland Menace`'s 2 000
   experience) (GAM-HER-001, TWN-SET-005).
8. The Thieves' Guild answers in the viewer's turn by its taverns (and at a map Tavern), a Den's
   visit at everything: the rows its level shows as places, the best hero, its primary skills and
   the best creature by TD-23's reading, the artifacts row under `TWN-QUIRK-13`; the obelisks row 0
   (`TWN-STUB-17`), no personality (`TWN-STUB-18`) (TWN-TAV-007 to -009).
9. Classes 95 and 97 import and play (93 Taverns on 47 maps, 227 Dens on 107).
10. The views: the owner's two tavern heroes, every change told (`TavernOffered`), a hired hero's
    appearance to every player that sees its tile, the tavern decision's view whole; the fold gate
    passes without a new gap; the protocol maps the commands, events, views and the guild's
    request.
11. The env plays the town's hire and the tavern decision (encodings v12, v13); the replays record
    the hire, the tavern answers and the dismissal.
12. Every new rule has its id, source and tests; `TWN-QUIRK-12` (T-16) and `-13` (T-17) recorded
    with their flags; the gates pass (coverage; determinism with the re-recorded replays; the
    census of the 160 maps' first offers; the properties of the pool; the benchmarks at their
    baselines, the week's re-baselined); batch TWN-4 run and scored, its consensus rows
    `verified`.

### 81. Risks and open points

- **Two lanes in the same crates at once:** E08's T122 to T125 and this increment's T35, T37 to
  T39 change ADR-0016 and -0017, the import, the adventure part, the game's battle end and the
  replays: merges in order, a rebase and a second re-recording when both bump.
- **Every path off the map:** a battle's (each result), a capture's empty garrison hero and a
  dismissal must hand back the away record and set the type's status; a missed path leaves a type
  neither free nor taken (lost to every draw). The pool's properties and the census guard them.
- **The store of heroes grows in play:** hero ids follow the draws' order, so any change of the
  draws' order moves every later id (the replays' outcomes; the env's hero slots by id).
- **The batch's harness work** is new (the guild window's capture, a retreat and a surrender in a
  recipe): TWN-4 may be cut in two at T34's start.
- **The observation grows** by two tavern heroes' details per viewer (shared `Arc`s: a clone is
  cheap); the guild's query reads every player's heroes and towns (only on request).
- **Open:** the computer's hires and its use of the map's Taverns (E13); a defeated player's
  heroes, slots and column (E10); the rumours (out, decision 99).

## Part G — increment 5: the mage guild

Increment 5's note (the mage guild, decision 162), written by T43 (2026-11-20, kept whole:
decision 172) from the survey `docs/rules/surveys/E09-guild.md` (§ numbers are the survey's unless
they name this note) and the answers at `TWN-Q31` (decisions 167 to 171) and `TWN-Q32` (decisions
172 to 176), with T43's agent's calls (decision 177) as the checkpoint `TWN-Q33` settled them
(decisions 178 to 181: the towns' stream for the draw over the agent's new stream). Its ADR
amendments are the drafts in `docs/lanes/town/adr-drafts/` (`0028-`, `0017-`, `0016-` and
`0013-increment-5.md`), accepted at `TWN-Q33` before anything is built, with the cut of §98 and the
exit criteria of §99.

### 82. Scope, sources and choices (T43)

**Scope** (decision 162): the guild's spells drawn into its levels in every town at the new game
(the spells' town chances, the map's required and banned spells of a town, the scenario's bans, the
faction's highest level, the Tower's Library), what the guild shows as it is built, learning them
(the spell book and Wisdom) at the original's triggers with an opening command of the owner's
(TD-27), the spell book's purchase, the spell points restored at night in a town with a guild, the
Dungeon's Mana Vortex, the Conflux's Aurora Borealis (decision 174), the views, the env, the
protocol, the replays and batch TWN-5. **Not here:** Town Portal's and Castle Gate's arrivals
(increment 7: they teach there, §89), the computer player's book purchase and its heroes' re-visits
at each turn (E10, E13; decision 171), an ally's visit (E10's teams, `TWN-STUB-11`), the other
buildings a hero visits (`town_visit_buildings` from 0x5BD99A: increment 6), RoE's own
`SPTRAITS.TXT` (E11).

**The user's choices** (`TWN-Q31`, `TWN-Q32`): TD-10 the original (a main town's hero learns at
the start, decision 167); TD-25 the original (a spell the town requires and bans is never in its
guild, 168); TD-26 the original's set with T-18 `fix-optional` (169); TD-27 the original's
triggers with a free opening command (170); T43 whole (172); the weights in the town records
(173); the Aurora in this increment (174); `TownCommand::Open`, listed only where it would change
something, a visit's end, a capture and a siege's end applying the same (175); the spell book as
`TownCommand::BuySpellBook` (176).

**The agent's calls** (decision 177; change freely but where `TWN-Q33` decided, decisions 178 to
181):
1. **The draws' stream: the towns'** (`StreamId::TOWNS`; decision 179, the user's over the agent's
   recommended new named stream, as `TAVERNS` was by decision 115): no `shared/` change; the
   guilds are its first draws, at the build, so every later draw of it (the week's kind, the
   neutral garrisons' weeks, the Portals, an event's Portal) moves and the replays that play a week
   re-record their outcomes (§97). The setup stream was not chosen (every later setup draw would
   move: each replay's map).
2. **The spare drawn in every town** (the original's): the draw is slot by slot without
   replacement, so the spare (each level's last slot) never changes what a town without a Library
   shows; drawing it everywhere keeps one rule and lets a Library built later show at once (§87).
3. **The opening teaches both heroes whenever a guild stands** (confirmed: decision 178), not
   only with a visitor: the
   original's guild window, free from the same screen, teaches the visitor then the garrison hero
   whoever stands there (§12's third caller, `town_screen_proc` 0x5D3589), so a garrison hero alone
   learns by it at will; one command stands for the screen and its window (§90).
4. **A visit's end applies the opening, then the learning** (the human's visit opens the town
   screen, `town_screen_init`'s Vortex and the bars' learning, then `town_visit`'s learning at its
   end: the learning twice is once); **a siege's end the learning alone** (no screen opens: the
   attacker stays outside, the defender's heroes learn where they stand).
5. **No new event kind:** learning, the Vortex's points and the book are `HeroChanged` (E08's,
   the meeting's way), the Vortex's flag the town view's `TownChanged`; the night's refill is as
   silent as the day's regain (HER-MAN-002). The original's Vortex message (GENRLTXT 580) is the
   client's, from the two events.
6. **The owner's view** gains the shown spells per level, the Aurora mark and the Vortex flag
   (§94); nothing for anybody else (calls 5 and 6 confirmed: decision 180).
7. **The Aurora set is a town's static part**, fixed at the build (the town's bans, the spells
   granted by name only and, with `TWN-QUIRK-14`'s flag off, the scenario's bans taken out): the
   game keeps no ban after the build (GAM-SET-003 as it stands).
8. **The Vortex's flag in every town** (the original's +0x33), read only where a `mana_vortex`
   building stands: no faction test in the rule.
9. **The book's hero is named by the town** (the visitor, else the garrison hero, the original's);
   `BuySpellBook` carries the town alone. An ally's visitor (the original lets the owner buy for
   it) waits for E10's teams.
10. **No ceiling on the Vortex's points** (as read, §15; the reward records' 999, D-38, is theirs).
11. **The roles' names** `guild` (a level), `library`, `aurora`, `mana_vortex` (§83); the faction
    tests of the original (Tower 2, Dungeon 5, Conflux 8) become roles of those records' buildings.
12. **The start's learning after every draw of the build** (the guilds', the taverns' fill): the
    original teaches in `main_town_heroes` before its towns' spells are drawn for the other towns,
    but a main town's own guild is drawn by then (its `town_setup` ran); the engine draws all
    first, which teaches the same.

### 83. The town part v9: the guild weights, the roles, the slots, the price, the factor (TWN-GLD-001; `h3-towns`, `h3-pack-towns`)

- **`[town.guild]`** in each town record: a spell key of the core and its weight (an integer 1 to
  10 000; 0s omitted, as `[town.tavern]`'s classes). The compiler (`h3-pack-towns`) refuses a key
  the core lacks or whose category is a creature ability (`SpellCategory`), a key listed twice; a
  spell granted by name only may be listed (its weight is 0 in every column of the original's data,
  and the draw excludes it anyway, §85). `TownRecord::guild_weight(spell) -> u16`, 0 when absent.
  No sum test (the original's columns sum to 100 a reachable level, but nothing reads the sum).
- **Roles** (`BuildingRoles`): `guild: Option<u8>` (1 to 5; at most one building of a record per
  level), `library: bool`, `aurora: bool`, `mana_vortex: bool` (each at most one building of a
  record). The compiler refuses a level outside 1 to 5, a level twice, a second `library`, `aurora`
  or `mana_vortex`. Read as ADR-0028 point 5 has it: `guild` standing for the level and ever built
  for a guild's presence (equivalent: a guild level supersedes the lower ones, and every build adds
  what it supersedes, TWN-BLD-005), `library` and `aurora` ever built (the original's +0x158), the
  Vortex standing (+0x150).
- **Parameters** (`[town_params]`): `guild_slots = [5, 4, 3, 2, 1]` (each 1 to 6: the spare makes
  at most 6, the slots' arrays' width, §86), `spell_book_price = { gold = 500 }`,
  `vortex_factor = 2` (1 to 10). The spell book is the core's first artifact with the `spell_book`
  effect (no parameter: HER-ART-007's).
- **The harvest** (`h3-harvest`, the battle lane's crate, `cross-lane`, as T36's tavern weights):
  `SPTRAITS.TXT`'s fields 16 to 24 of the 70 hero spells' rows (§2.1: rows 5 to 14 and 18 to 77),
  the base pack's eight records from `H3bitmap.lod`'s copy, the AB pack's Conflux record from its
  ninth column (the AB copy differs only in row 57's name, all its chances 0: §2.2); the roles in
  the gap-fill file (`guild` by the Mage Guilds' keys `mage_guild_level_1` to `_5`, `library` on the
  Tower's `library`, `aurora` on the Conflux's `aurora_borealis`, `mana_vortex` on the Dungeon's
  `mana_vortex`; the keys as the packs have them, checked at T46); the parameters tier-2 data.
  The packs' headers cite survey `E09-guild.md` §2 and this section.
- **Version:** the town ruleset part v9 (`TOWN_RULESET_HASH_VERSION`), the hash folding the
  weights in each record's spell order, the roles and the parameters.
- **Tests:** the compiler's refusals each; the weights' round trip; the harvest's columns against
  §2.2's sums (each faction's reachable levels sum to 100, the others 0; Titan's Lightning Bolt 0
  everywhere) in `h3-harvest`'s original-data test.
- *As built* (T46, decisions 184, 185): as above, with these differences. The slots are bounded
  **1 to 5** (the spare makes at most six, the original's slots a level: "1 to 6" above
  contradicted its own reason). A weight of 0 is refused (0s are omitted); no override changes the
  weights (no pack needs one). The roles come from the gap-fill by building id: the Mage Guilds 0
  to 4 for **every faction whose list holds the building** (a role of every faction now skips a
  faction without it, an error when none has it; before, every such role was on a building every
  faction has), the Tower's 22, the Dungeon's 21 and the Conflux's 26. The Conflux's key is
  **`aurora_borealias`** (the packs' spelling, from `BldgSpec.txt`). A `sod` spell weighing in a
  guild is a harvest error (none does). The §2.2 checks are `h3-pack-towns`' `the_original_guilds`
  on the harvested packs (as T36's taverns: `h3-harvest` has no original-data test, `verify
  --with-original-data` re-harvests and compares), with Armageddon, Death Ripple and Animate Dead
  by faction as spot checks. Pins: the test part `0x67ee_ef1c_7b56_fda6`, the empty part
  `0xedec_d7c9_34cc_e444`; the goldens' town hash `0xcf8db1aa2bb768d2`.

### 84. The map's town spell sets (TWN-SET-012; `h3-map`'s `import/town.rs`, `h3-towns`' `TownSetup`)

- `h3_map`'s reader holds both sets already (`TownBody::required_spells` read on maps above RoE,
  `banned_spells` on every map; bits 70 and 71 dropped, §3.4). The import maps each set bit 0 to 69
  to the core's spell of that code (as GAM-SET-003's map-wide set), in spell order, once.
- **`TownSetup`** gains `required_spells: Vec<SpellId>` and `banned_spells: Vec<SpellId>` (sorted,
  no repeats; empty for a map that gives none: RoE's required set, the writer's default). Both
  kept to the build only (the draw, §85; the Aurora set, §86).
- **`GameSetup::validate`** refuses a town spell the ruleset lacks or holds as a creature ability
  (`GameSetupError::TownSpell(TownId, SpellId)`), after the scenario's bans (GAM-SET-003's order).
- **The writer** (`h3-map`'s `writer/town*`, the town lane's) writes both sets (for batch TWN-5's
  maps, §97, and the plans).
- **Version:** the map import v28 (the adventure lane's number, taken by the next import change:
  its inbox of 2026-11-19), `cross-lane`.
- **Census** (§1): 32 towns on 10 maps require spells, 396 on 65 ban some; `Faeries`' Rampart
  requires and bans Dimension Door (TD-25).
- **Tests:** an AB map's required and banned bits to spells; an RoE map's required set empty; a
  bit past 69 dropped; the refusal.
- *As built* (T47a2, decision 188): `h3_map`'s `import/town.rs` `town_spells` (the bits 0 to 69
  by `codes::SPELLS`, sorted by handle); a spell the ruleset lacks is **left out**, as the pool's
  hero types (no guild can draw it; every hero spell is a base pack's, so the shipped maps lose
  none), not refused by the import; the setup's refusal `GameSetupError::TownSpell(TownId,
  SpellId)` (a spell the ruleset lacks or a creature ability; one granted by name only allowed)
  right after the scenario's bans, through `h3_game`'s `town::town_spell_refused`. Tests: an AB
  and a SoD map's sets (Titan's Lightning Bolt required, Dimension Door required and banned, bits
  70 and 71 dropped), an RoE map's, a town without its body; the refusal's order and text.

### 85. The draw (TWN-GLD-002; `h3-towns`' `guild` module, new; `town_setup_spells` 0x5BE600)

- **`h3_towns::draw_guild(record, part, core, sets, draws) -> GuildSlots`**: `sets` the town's
  required and banned spells and the scenario's bans; `draws` the towns' stream (`&mut Rng`). The
  excluded set starts with the town's banned spells, the scenario's and every spell granted by name
  only (Titan's Lightning Bolt: the original's flag 0x2000 sets its game ban, §3.1); for each level
  L from 1 to 5 and each of its `guild_slots[L] + 1` slots in order: scan the core's hero spells
  of level L in spell order, skipping the excluded; the first in the town's required set is taken
  (whatever its weight); else the total of the scanned spells' weights (the town record's, by the
  town's faction); 0 leaves the slot empty without a draw; else a draw of 1 to the total
  (`Rng::range_inclusive`) and the first scanned spell whose running sum reaches it (a 0 weight is
  never chosen). The taken spell joins the excluded set. TD-25 falls out: a banned spell is
  excluded before the required test.
- **The game** (`h3-game`'s `town/guild.rs`, new) draws every town's guild at the build, in
  `TownId` order, after the towns' records are built (each faction resolved, its buildings set:
  `town_setup`) and before the start's learning (§89), from the towns' stream (`StreamId::TOWNS`,
  decision 179), its first draws: the build holds the state's `towns_rng` from its start, so the
  guilds come before any week's draw of it. Nothing in play draws a guild.
- **The original's order** interleaves these draws with the rest of the setup's on one generator
  (§3.2); the engine's draw order is its own (E08 decision 17): the same law per slot.
- **Tests:** a required spell first in spell order, its weight 0 or not; a required and banned
  spell absent (TD-25); the scenario's ban; 57 never; an empty level when every spell is excluded
  (`Heroes of Might not Magic`'s towns, §1); the draw's law on a small record (the running sums);
  two games of one seed draw alike; a fork shares the slots.
- *As built* (T47a1, decisions 186, 187): `h3_towns::draw_guild(record, params, core, sets,
  draws)` takes the part's parameters (`&TownParams`, which the game holds where it builds a
  town: no `None` to handle) in place of the part, and `sets` as a `GuildSets` of three
  `SpellSet`s (`required`, `banned`, `scenario`; the game makes them of `TownSetup`'s lists and
  the scenario's bans, T47a2). A level's slots are `guild_slots[L] + 1`, at most
  `MAX_GUILD_SLOTS` (6). The unit tests on the fixture's guild (`guild_core`'s spells,
  `guild_keep`'s record): each level's weighted spells once, the rest empty; the law by the first
  slot's roll replayed over 64 seeds; the required spells first in spell order, a weight of 0
  among them, and a guild whose every slot is required or empty drawing nothing; TD-25; the
  scenario's ban, Titan's Lightning Bolt and a creature ability (each weighted, required too)
  never; an empty level; one seed alike. The game's draw at the build, two games and a fork:
  T47a2.
- *As built* (T47a2, decision 188): `h3_game`'s `town/guild.rs` `draw_guilds`, called by
  `GameState::new` right after the Shipyards without water (`without_water`) and before the store
  is made, on the built `(Town, TownStatic)` pairs: each town's `GuildSets` of its `TownSetup`'s
  lists and the scenario's `banned_spells`, its slots by `draw_guild` written with
  `Town::with_guild`, its Aurora set by `aurora_set` into `TownStatic::aurora`; the towns' stream
  made before them and moved into the state. A pack without guild weights (the game's own test
  part) draws nothing, so no existing unit test's stream moved. Tests (`town/tests/guild.rs`): the
  slots of two towns (a required spell first, a town's ban, a spell granted by name only never),
  the towns' stream after the build equal to the guilds' draws replayed, the Aurora sets, nothing
  shown without a Mage Guild, two games alike and a fork; the scenario's ban with the quirk.

### 86. The store v4: the slots, the Vortex's flag, the Aurora set (`h3-towns`' `record.rs`, `store.rs`)

- **`Town`** gains `guild: GuildSlots`: the twenty slots (6, 5, 4, 3, 2 by level), each a spell or
  empty (a compact array; the spare is each level's last), set by the build and never written
  again; and `vortex: bool`, the Mana Vortex's week flag, true from the build.
- **`TownStatic`** gains `aurora: SpellSet` (a set over the core's hero spells): every hero spell
  the town's banned set does not hold, less those granted by name only, less the scenario's bans
  when `quirk.town.aurora_ignores_map_bans` (`TWN-QUIRK-14`, T-18) is off; computed at the build for
  every town (cheap; only a town with an `aurora` building ever built reads it).
- **Writes** (`Towns`): `start_week()` sets every town's flag (the towns' week, GAM-CAL-002); a
  town's flag cleared by the Vortex (`take_vortex(town)`), each copying only its town (ADR-0028
  point 4).
- **Hash** v4 (`TOWN_STATE_HASH_VERSION`): after each town's Portal its twenty slots (0 empty, else
  1 and the spell's handle) and its flag; after each static part's events its Aurora set (its bits).
- **Tests:** the hash pinned; a fork's flag write copies one town; the week sets every flag.
- *As built in part* (T47a1, decision 187): `h3_towns::GuildSlots` holds **six slots at every
  level** (thirty bytes, a spell's handle a byte, `u8::MAX` empty): T46 bounds the part's slots 1
  to 5 at every level, so any level's spare may make six; "the twenty slots (6, 5, 4, 3, 2)"
  above are the base part's, and the hash v4 folds the thirty the same way. `h3_towns::SpellSet`
  (one `u128`, a bit per handle below `MAX_SPELLS`) holds the Aurora set and the draw's sets;
  `h3_towns::aurora_set(core, sets)` computes the set, reading `TWN-QUIRK-14`'s flag from the
  core (the quirk's rule test there, TWN-GLD-004's set). The store's fields, writes and hash v4
  are T47a2's.
- *As built* (T47a2, decision 188): `Town`'s `guild` (`GuildSlots`, empty from `Town::new`;
  `with_guild`, public for the game's build; `guild()`) and `vortex` (true from `Town::new`;
  `vortex()`, the crate's `with_vortex`); `TownStatic::aurora` (a public field, empty until the
  game's build: `town_setup` lacks the scenario's bans); `Towns::start_week` (writes only the
  towns whose flag is clear) and `Towns::take_vortex(town) -> bool`, both called by T49a with the
  Vortex (nothing clears a flag before it); `GuildSlots::all` (the thirty slots in the hash's
  order). Hash v4: after each town's Portal its thirty slots (0, or 1 and the handle) and its flag;
  after each static part's events its Aurora set's sixteen bytes. The store's pinned hash
  0x79C0_7378_D0E0_4452.

### 87. What the guild shows (TWN-GLD-003; §3.3, §4)

- **`h3_towns::guild_level(town, part) -> u8`**: the highest level of a standing `guild` building,
  0 none. **`shown(town, part, level) -> &[SpellId]`**: for a level up to the guild level its first
  `guild_slots[level]` slots, one more with a `library` building ever built, cut at the first empty
  slot (an empty slot is only followed by empty ones of its level, §3.2); none above it.
- A level or the Library built later shows at once (the counts are derived, not stored: the
  original recomputes them at every build, §4). The faction's highest level is its record's
  buildings (Castle 4, Stronghold and Fortress 3, the others 5).
- **Tests:** each level's count with and without the Library; a level built later; the Library
  built later shows each built level's spare; an empty slot ends the level.
- *As built* (T47a1, decision 187): `shown(slots, town, part, level)` returns an iterator of the
  spells (the slots are bytes, no slice of `SpellId`s), the slots passed beside the town until
  they join its record (T47a2: `shown(town.guild(), town, part, level)`); nothing at a level 0 or
  above 5, for a faction without a record or a part without parameters. `guild_level` gives 0
  for a faction without a record.

### 88. What a hero learns (TWN-GLD-004; §5, §11; `h3-towns`' `guild` module)

- **`h3_towns::teaches(town, statics, part, core, details) -> Vec<SpellId>`**: none without a
  `guild` building ever built or without a worn spell book (`h3_heroes`' test, HER-BAT-011's);
  with an `aurora` building ever built every spell of the town's Aurora set (spell order); else the
  shown spells (§87) of each level, level then slot order; in both, only the spells
  `h3_heroes::can_learn_spell` allows (a level at most HER-SPL-005's learnable level, not known:
  the guild level does not bound the Aurora's). The game applies each as
  `HeroChange::LearnSpell` (the details valid with it, HER-CHG-001).
- **T-18** (`TWN-QUIRK-14`, `fix-optional`, on in `faithful`, decision 169): in `faithful` the
  Aurora teaches a spell the scenario bans (the original's 0x5BE10C to 0x5BE172 do not test it);
  `strict-fixed` takes the scenario's bans out of the Aurora set (§86). The flag waits for its
  confirmation (row TWN-5-2 or a second look at the code, `docs/04-rules-and-sources.md`,
  "Confirm before flagging"); the `shared/` task adds it.
- **Tests:** a hero without a book learns nothing; Wisdom's bound (none 2, Basic 3, Expert 5);
  a known spell skipped; the Aurora's set at a level-1 guild to level 5; the town's ban kept out of
  the Aurora's, the scenario's in by the flag on and out by it off; no owner or class test.
- *As built* (T48a, decision 190): `h3_towns::teaches(town, statics, part, core, details)` in the
  guild module (`h3-towns` now depends on `h3-heroes`, as `arch.toml` allowed): the guild and the
  Aurora read by role from the town's every-built set through its faction's record (none without
  it), each spell through `can_learn_spell` with the known spells counting (the book, the level,
  the known spell in one test). The fixture's `guild_core` gained Wisdom, a class, the type `SAGE`
  and the spell book `BOOK` (`fixture::sage`); three tests.

### 89. When the town's heroes learn (TWN-GLD-005; §12; ADR-0017 point 10's list)

`h3-game`'s `town::guild::learn(state, rules, town, heroes, sink)` teaches the named heroes in
order (each that exists: a missing visitor or garrison hero is skipped), each spell told
`HeroChanged` to the hero's owner. The triggers and their places in the engine:

| Original caller (§12) | Engine | Heroes | Module |
|---|---|---|---|
| `town_visit` 0x4AB3C3 (an own or allied town, an undefended capture) | a step entering a town of the team, a `Revisit` there, an undefended capture: the outcome's `town_entered` after `capture` (§90's opening first) | visitor, garrison hero | `state.rs`' `settle_outcome` (`cross-lane`), `h3-adventure`'s `town.rs` (`visit_town`, the town lane's) |
| `town_visit` after a siege's battle | the siege's end, after the battle's outcome and a capture | the town's visitor, garrison hero as they stand | the game's siege end (`town/siege.rs` or `state/battle.rs`, `cross-lane`: T48b's call) |
| `town_screen_bars` 0x5C7007 | `Open` (§90), a bars' move (`TownCommand::Heroes`) while a visitor stands | visitor, garrison hero | `town/guild.rs`, `town/rows.rs` |
| `town_screen_proc` 0x5D3589 (the guild window) | `Open` (§90), the book's purchase (§91) | visitor, garrison hero | `town/guild.rs` |
| `town_build_effects` 0x5BF034 | every build, the command's and an event's (`town::command::build_told`), then the Vortex (§93) | visitor, garrison hero | `town/command.rs` |
| `town_tavern_hire` 0x5C138E | a town's hire (lifts `TWN-STUB-15`), then the Vortex to the hired hero | the hired visitor, garrison hero | `town/hire.rs` |
| `main_town_heroes` 0x4CA1A2 | the build's end, each player with a main town's hero in colour order (TD-10; lifts `TWN-STUB-6`) | visitor, garrison hero | `town/heroes.rs` |
| `hero_relocate_spell`, `town_castle_gate` | Town Portal, Castle Gate | — | increment 7 |
| `ai_town_visit` | the computer player's visits and turn starts | — | E10, E13 |

- **Nothing else teaches** (TD-27): no turn start, no new day, no level-up, skill or book a hero
  gets while it stands in a town; such a hero learns at the next trigger.
- **The visit's level-up:** the engine's level-ups are decisions opened after the battle or the
  gain that made them (GAM-DEC-002); a step onto an own town gains no experience, so the original's
  level-up after the visit's learning (`hero_level_up` 0x4AB3CB) has no counterpart but at a
  siege's end, where the engine's capture already follows the winner's levels (TWN-CAP-003): a
  Wisdom gained there counts at once for whoever learns (the attacker does not: it stays outside).
- **Tests** per trigger (a hero with a book learns each guild spell once; the order of the two
  heroes); nothing at a turn start or a new day; a capture teaches the capturer; a siege's
  defender learns; a bars' move without a visitor teaches nothing.
- *As built* (T48a, decision 190; the town's own triggers): `town::guild::learn(state, rules,
  town, sink)` teaches the town's visitor, then its garrison hero (no hero list: every trigger of
  T48a names both; the computer's one-hero call is E10's), each spell by `GameState::change_hero`
  (made `pub(crate)`, `cross-lane`). Called at the end of `town::command::build_told` (after
  `TownBuilt` and the views told; an event's buildings each in turn), after `town::hire::hire`'s
  placement and its events (the map Tavern's hire none), at the end of `town::rows::heroes` while
  a visitor stands after the move, and by `town::guild::at_start` in `GameState::new` once the
  state is assembled, before the start's tavern fill and the first turn (each generating main
  town in player order, `NoopSink`: the build tells nothing). Six tests (`town/tests/learn.rs`).
  No golden replay moved (none of the 111 meets one of these triggers with a guild built and a
  hero with a book there), and no version moved.
- *As built* (T48b, decision 191; the visit's end and the siege's end): `h3_adventure`'s
  `AdventureOutcome::town_entered` (new, `cross-lane`) names the town `visit_town` entered
  (`Visit::Enter`: a step or a `Revisit` into a town of the team, or the undefended capture);
  `GameState::settle_outcome` calls `learn` for it right after `town::capture` (the capture's
  reveal, Portal and views first). A siege's end: `GameState::end_battle` calls `learn` for the
  battle's town (`town`, a siege only: the battle beside a garrison hero has none) after the
  outcome, the unowned battle's end and the attacker's held capture, **before** `give_levels`
  (the original's 0x4AB3C3 before 0x4AB3CB; the engine's level-ups are decisions, so a Wisdom
  the battle brings counts at the next trigger, as the original's, not at once as the
  level-up paragraph above guessed). A defending visitor or garrison hero that won learns; an
  attacker's win teaches nobody (the winner outside, the defender gone); the siege's capture
  after the levels (`town_won`) sets no `town_entered`. Three tests (`town/tests/learn.rs`).
  No golden replay, bot report or version moved; `h3-map`'s plan properties pass unchanged.

### 90. The opening (TWN-GLD-006; `TownCommand::Open { town }`; decisions 170, 175)

- **Checked** (`town::command`): the turn (`check_ready`), the town (`UnknownTown`), the owner
  (`NotOwner`; TD-11's rule: only the owner opens its town screen), no decision open. No other
  refusal: an opening that changes nothing is legal and changes nothing (its hash unchanged).
- **Applied:** the Vortex (§93) to the garrison hero, then the visitor; then the learning (§89) of
  the visitor, then the garrison hero (decision 177's call 3: the guild window's learning, so with
  or without a visitor); the events in that order; then the town's changed view (`Told`: the flag).
- **Listed** (`town::legal_builds`, after the town's hires and before its hero moves): only where
  it would change something: the Vortex would be taken (a standing `mana_vortex`, the flag set, the
  garrison hero or the visitor below the factor times its maximum) or a hero there would learn a
  spell (`teaches` not empty for one). The env therefore sees no idle opening (the town lane's
  `TRAPS.md`: a command listed for every town floods the plans and moves every random game).
- **The visit's end** (§89's first row) runs the same application (Vortex, then learning) for the
  entering hero's town, after a capture's effects (the capture's reveal, Portal and views first).
- **Tests:** each refusal changes nothing; the Vortex to the garrison hero first; a garrison hero
  alone learns; the listing on and off; the replay word.
- *As built* (T49a2, decision 193): `h3_towns::TownCommand::Open { town }`, no new refusal and no
  event. The game's `town::guild::open` checks the store's town, then the owner
  (`command::owner`), then runs `visit` (§93's *As built*): the Vortex with its told flag, then the
  learning, as at the visit's end, so the flag's view (told whole, its parts after its
  `TownChanged`) comes between the Vortex's `HeroChanged` and the learning's. `opens` serves the
  listing: the garrison hero (`town_hero`, a binary search) and the visitor (the hero on the
  entrance, which `legal_builds` now finds once a town for the hires, the opening and the moves:
  `AdventureState::town_entrance_hero`), `false` for a town no hero stands in; then
  `h3_towns::opening_gives` (the same answer as `vortex` or `teaches` for either, read through the
  part's derived `GuildRoles`: the bench gate, the lane's `TRAPS.md`); chained after the hires
  and before the moves. Tests: the
  game's four in `town/tests/open.rs` (the Vortex to the garrison hero with the flag told before
  the visitor's then the garrison hero's learning, a repeat changing nothing; a garrison hero
  alone learning, no guild nothing; the refusals; the listing's place, on and off with the week's
  flag and the threshold, none in a town without a hero); `h3-files`' round trip and refusals of
  `town open`. The plans (`h3-map`'s `play.rs`): `town_builds` probes each town's opening after
  its hires; `accepted_builds` keeps an accepted opening only where the probe changed the state;
  an unlisted opening may be accepted when it changes nothing; `check_town_open` reads a listed
  one apart (the town the actor's, the state changed, the Vortex to the first of the garrison hero
  and the visitor it raises, each hero's spells grown by `teaches`, the record but the flag, the
  heroes and the treasury unchanged); `Step::Town` plays a listed opening in one step of eight.

### 91. The spell book (TWN-GLD-007; `TownCommand::BuySpellBook { town }`; decision 176; §6, §13)

- **Checked** in order, each refusal typed and changing nothing: the turn, the town
  (`UnknownTown`), the owner (`NotOwner`), no decision open, a standing guild (`NoGuild`:
  `h3_towns::check_spell_book`, which names the part's price), the hero (the visitor, else the
  garrison hero; `NoHero`), that it wears no spell book (`HasSpellBook`), the treasury
  (`CannotPay`).
- **Applied:** the price paid (`ResourcePaid`), the book given (`HeroChange::AddArtifact` of the
  core's spell book, worn at its free position: HER-CHG-002; told `HeroChanged`), then the
  learning of the visitor, then the garrison hero (§89; the original's window after the purchase).
  The original's question (GENRLTXT 215) is the command itself; its refusal with less than 500
  (GENRLTXT 214) is `CannotPay`; Yog's campaign refusal (GENRLTXT 737) waits for the campaigns.
- **Listed** after the opening: where the guild stands, the hero lacks a book and the treasury
  holds the price.
- **Tests:** each refusal; the book worn; the learning right after; the price.
- *As built* (T49b, decision 194: kept whole): `h3_towns::TownCommand::BuySpellBook { town }` and
  the refusals `NoGuild`, `NoHero`, `HasSpellBook` (appended to `TownCommandError`).
  `h3_towns::check_spell_book` reads a standing guild through the part's derived `GuildRoles`
  (built ∩ `guild` less what the built `superseding` buildings supersede: the private
  `GuildRead`, shared with `opening_gives`) and names `spell_book_price`. `h3_heroes` gains
  `wears_spell_book` (the private `wears_book`: a worn artifact's `spell_book` effect) and
  `spell_book` (the ruleset's first artifact with the effect, the original's artifact 0, as the
  setup and Titan's Thunder find it); the original's `hero_has_artifact` counts the backpack too,
  which a spell book never reaches (its slot `backpack = false`, HER-ART-002), so "wears" is the
  original's test. The game's `town::guild::buy_book` checks the store's town, the owner, then
  `check_spell_book`, then `h3_heroes::spell_book` (none: `NoGuild`), then the hero (`heroes`'
  visitor, any player's, else the garrison hero: `NoHero`), its book (`HasSpellBook`: a visitor
  with one is offered none even beside a bookless garrison hero, as the original's window),
  then the treasury (`CannotPay`); pays, gives `HeroChange::AddArtifact` (told `HeroChanged` to
  the hero's owner), then `learn` (no Vortex). The listing: `town::guild::offers` (replacing
  T49a2's `opens`) asks both guild commands of a town at once: no hero there, neither; else
  `h3_towns::guild_offers` (`GuildOffers`: the opening's answer as `opening_gives` and the book's
  price where a guild stands, the derived roles read once), then the treasury, the ruleset's book
  and the hero's own book last (an effects walk); `legal_builds` finds the garrison hero once for
  both and chains them as town commands after the hires and before the moves (the bench gate:
  the lane's `TRAPS.md`). Tests: the game's
  four in `town/tests/book.rs` (the events in order, the price, a repeat refused; the garrison
  hero alone, a hired visitor with its type's book refusing, the swap; every refusal and a
  ruleset without a book; the listing's place after the opening, on and off); `h3-towns'`
  `a_spell_book_is_sold_where_a_mage_guild_stands_for_the_part_s_price` and the opening's test
  checking `guild_offers` against `opening_gives` and `check_spell_book` (any level, the Library
  or the Aurora alone none, a guild superseded by a non-guild none, another price, no record or
  parameters none); `h3-heroes'` Titan's Thunder test with the two queries; `h3-files`' round
  trip and four refusals of `town book`. The plans (`h3-map`'s `play.rs`): `town_builds` probes
  each town's purchase after its opening; `check_town_book` reads a listed one apart (the town
  the actor's with a guild, the visitor else the garrison hero bookless before and wearing the
  ruleset's book after, its other artifacts and every hero's points unchanged, each hero's spells
  grown by `teaches` read after the book, the price paid, the town's record unchanged);
  `check_conservation` takes the book's price; `Step::Town` plays a listed purchase in one step
  of eight (`n % 8 == 7`).

### 92. The night's refill (TWN-GLD-008; GAM-CAL-002 amended; §14)

- In `advance_day` (`state/day.rs`, the adventure lane's, `cross-lane`: one call), right after
  `regain_mana`: `town::guild::refill(state, rules)`: each town in `TownId` order with a `guild`
  building ever built raises its visitor's, then its garrison hero's spell points to the maximum
  (`h3_heroes::max_mana`, HER-MAN-001), never lowering; no spell book, no owner test; written only
  where it changes (as `regain_mana`), no event.
- **Tests:** a visitor's and a garrison hero's points raised; above the maximum kept; no guild no
  refill; day 1 none.
- *As built* (T49a1, decision 192): `h3_towns::refills(town, part)` (a guild ever built); the
  game's `town::guild::refilled` lists the heroes it refills (`TownId` order, the visitor then the
  garrison hero; towns without a hero skipped before the guild test), read by `regain_mana`, which
  gives each `points.max(max_mana)` in place of `day_mana`: the regain never passes the maximum
  nor lowers the points, so regain-then-refill is that value, the maximum computed once. A
  separate pass after the regain (the first draft) cost `day-start-builds-36` +5.4 %,
  `day-start-towns-36` +3.3 %, `month-start-builds-36` +2.8 % (`max_mana` twice for every
  refilled hero). Tests: `h3-towns`' one (a superseded guild counts), the game's two in
  `town/tests/mana.rs` (a turn's end no night; a hero beside the town nothing). No golden replay
  moved.

### 93. The Mana Vortex (TWN-GLD-009; §15)

- **`h3_towns::vortex(town, part, core, details) -> Option<u16>`**: with a standing `mana_vortex`
  building and the flag set, the points `vortex_factor × max_mana` when the hero's are below them.
  The game applies `HeroChange::SetSpellPoints`, tells `HeroChanged`, clears the flag
  (`take_vortex`) and tells the town's view; the first hero that takes it ends it for the week.
- **Given** at the opening and a visit's end (the garrison hero, then the visitor), after every
  build (the same order, after the build's learning), after a town's hire (the hired hero). Not at
  a turn start, not at the night, not at a siege's end.
- **The week:** `Towns::start_week` with the towns' week (GAM-CAL-002: a week's first day, before
  the taverns' week); the build sets it (`town_init`).
- **Tests:** once a week a town; twice the maximum; a hero at or above it leaves the flag; the
  garrison hero first; the build's and the hire's.
- *As built* (T49a1, decision 192): `h3_towns::vortex` checks the parameters, the flag and the
  standing building before the maximum (`max_mana × factor`, saturating). The game's
  `town::guild::vortex_to(state, rules, town, heroes, sink)` gives the first hero it raises,
  then `take_vortex` and the town's `Told` (taken only when a hero takes it); `vortex` orders the
  garrison hero, then the visitor (a build after its learning, in `build_told`), `visit` runs it
  before the learning (the visit's end in `settle_outcome`, and T49a2's `Open`), the hire passes
  the hired visitor alone. `Towns::start_week` after the week's growth, before the taverns' week,
  so `Told::before` (taken before the growth) tells the set flags with the week's views. No
  golden replay moved (none takes a Vortex); no version moved. Tests: `h3-towns`' one, the game's
  three in `town/tests/mana.rs` (the visit's end over two weeks with the told flags, the build's
  garrison hero first and at the threshold the visitor, the hire's hired hero alone).

### 94. The views and events (TWN-VEW-004 amended; `docs/06-security.md`'s town section by the `shared/` task)

- **`TownBuildingsView`** (the owner's alone) gains `guild: [Vec<SpellId>; 5]` (§87's shown
  spells by level; empty above the guild level), `aurora: bool` (an `aurora` building ever built:
  the window's "all spells" scroll) and `vortex: bool` (the week flag). Never to another viewer, not
  even Visions' top level (the original's quick view shows no spells, `E09-towns.md` §12.3).
- **Told:** a build's or an event's (a guild level, the Library) and a capture's views as now
  (`Told`); the Vortex's cleared flag after its `HeroChanged`; the week's set flags in the week's
  views (`Told::before` already spans the week's growth); learning, the Vortex's points and the
  book as `HeroChanged` to the hero's owner (§82's call 5).
- **The fold gate** (`h3-protocol`'s `tests/game_fold.rs`): the new fields fold from `TownChanged`
  (no new gap); `HeroChanged`'s `LearnSpell` and `SetSpellPoints` fold already (E08's).
- **Tests** (`town/tests/view.rs`): the owner sees the guild, another player nothing; each change
  told once.
- *As built* (T47b, decision 189): `TownBuildingsView::guild` is an `h3_towns::GuildSlots` holding
  only the shown slots (`h3_towns::shown_slots`: each level's shown spells in its first slots, in
  slot order, the rest empty), not `[Vec<SpellId>; 5]`: `GameEvent` is `Copy` and `TownChanged`
  carries the view (`GameEvent` grows from 72 to 104 bytes, `TownChanged` its largest variant).
  `aurora` is `h3_towns::has_aurora` (a building of role `aurora` ever built); `vortex` the raw
  flag of every town (set from the build; nothing clears it before T49a). On the wire (protocol
  v78) `TownBuildings`' `guild: [Vec<u16>; 5]` (the handles by level), `aurora`, `vortex`;
  `TownBuildings` is no longer `Copy` (the fold clones it). `GuildSlots::with` is public (the
  protocol's tests build a view back). No `Told` change: the guild changes only by a build, an
  event or a capture, which tell the owner's view already; the Vortex's cleared flag and the
  week's set flags are T49a's. Tests: `h3-towns`' two (`shown_slots`, `has_aurora`), the game's
  `the_owner_sees_what_its_guild_shows_told_at_each_build_and_another_player_nothing` (a level,
  the Library's spare, the Aurora, each told once to the owner, blue the mark alone), the
  protocol's round trips and fold with a guild; `h3-map`'s hygiene oracle reads the same.

### 95. The env, the protocol and the replays

- **The env** (`h3-sim`, the AI lane's paths, `cross-lane`): after `TAVERN_HIRE`'s two entries,
  `TOWN_OPEN` (the town by its entrance in the tile head) and `SPELL_BOOK` (the same head), each
  one action per listed command; the action encoding the next at merge for each (E08 T133b took
  v14, its T134a plans v15: so v15 and v16 or later), or one bump when both land together. `h3-sim`'s town-map tests and the `*-towns-36` workloads move (the town lane's
  `TRAPS.md`).
- **The protocol** (the server lane's, `cross-lane`): `TownCommand::Open` and `BuySpellBook`, the
  errors `NoGuild`, `NoHero`, `HasSpellBook`, `TownSetup`'s two sets where the setup crosses, the
  town view's three fields; a protocol bump per wire task.
- **The replays** (`h3-sim`'s adventure replay, `h3-files`' format, the adventure lane's,
  `cross-lane`): `town open <town>`, `town book <town>` (the replay word in the same pull request as
  the command, the town lane's `TRAPS.md`); the format the next at merge (E08 T133b took v21 after
  the client lane's v20; its T134a plans v22).
- *As built* (T49a2, decision 193): the env's encoding **v18** appends `TOWN_OPEN` (103) after
  E08 T135a's trades and services (encoding v17, written after this section), not beside the
  taverns' entries, so that no entry moves; `tileless` stops before it; `tile_mask`, `decode`
  and `encode` take the town by its entrance (building 0); the mask property's
  `check_town_heroes` covers it. The protocol **v79**: `wire::game::TownCommand::Open { town }`.
  The adventure replay format **v24** (`town open <town>`): the 111 goldens' `format` line only,
  `h3-cli`'s and `h3-sim`'s literals. The adventure lane's planned v79, v24 and v18 (E08 T143,
  T144; `E08-design.md` §16) move to v80, v25 and v19 (its inbox, the AI lane's and the server
  lane's). An opening is listed only where it changes something, which random play rarely meets:
  `h3-sim`'s town-map tests kept their seeds.
- *As built* (T49b, decision 194): encoding **v19** appends `SPELL_BOOK` (104) after
  `TOWN_OPEN`; `tileless` stops before both; `tile_mask` (the two through `guild_command`),
  `decode` and `encode` take the town by its entrance (building 0); the mask property's
  `check_town_heroes` covers it. The protocol **v80**: `wire::game::TownCommand::BuySpellBook {
  town }`, `TownReject::NoGuild`, `NoHero`, `HasSpellBook`. The adventure replay format **v25**
  (`town book <town>`): the 111 goldens' `format` line only, `h3-cli`'s and `h3-sim`'s literals.
  The adventure lane's planned v80, v25 and v19 (E08 T143, T144) move to v81, v26 and v20 (its
  inbox, the AI lane's and the server lane's). A purchase is listed wherever a bookless hero
  stands in an own town with a guild and the treasury holds 500 gold, which random play meets
  often: the town-map games change course (`h3-sim`'s seeds, the `*-towns-36` workloads).

### 96. The increment's ids, versions, quirks and stubs

- **Rule ids** (`town.toml`): TWN-GLD-001 the guild weights, roles and parameters, -002 the draw,
  -003 what the guild shows, -004 what a hero learns (the Aurora, T-18), -005 when the town's heroes
  learn, -006 the opening, -007 the spell book, -008 the night's refill, -009 the Mana Vortex;
  TWN-SET-012 the map's town spell sets. **Amended:** TWN-SET-009 (TD-10), TWN-BLD-005 and
  TWN-EVT-003 (a build's learning and Vortex), TWN-TAV-003 (the hire's), TWN-ROW-002 to -004 (the
  bars' learning), TWN-VIS-001 and TWN-CAP-001, -003 (the visit's end, a capture, a siege's end),
  TWN-VEW-004 (the owner's guild), GAM-CAL-002 (the week's flags, the night's refill), GAM-SET-003
  (the scenario's bans read by the guild's draw and the Aurora set), HER-SPL-005 (its mage guilds).
- **Versions** (each bumped by the first task that needs it, the number the next at merge):

  | Version | Change | Task | Owner |
  |---|---|---|---|
  | town ruleset part v9 | the guild weights, the four roles, the slots, the book's price, the Vortex's factor | T46 | town |
  | town state hash v4 | the guild's slots, the Vortex's flag, the Aurora set | T47a | town |
  | map import v28 | the towns' required and banned spells | T47a | adventure, `cross-lane` |
  | game state hash | the store's v4 (the towns' stream's positions move: the week replays re-recorded) | T47a | adventure, `cross-lane` |
  | adventure replay format | `town open`; `town book` | T49a, T49b | adventure, `cross-lane` |
  | env action encoding | `TOWN_OPEN`; `SPELL_BOOK` | T49a, T49b | AI lane, `cross-lane` |
  | protocol | the town view's guild (T47b), `Open` (T49a), `BuySpellBook` (T49b) | T47b, T49a, T49b | server lane, `cross-lane` |
  | shared | the flag of `TWN-QUIRK-14` (the profiles' ruleset hashes move); no new stream (decision 179) | T45b | shared |
  | adventure state hash, adventure part | none (`AdventureOutcome::town_entered` is not state) | — | — |

- **Quirks** (`quirks-town.md`): `TWN-QUIRK-14` T-18 (`fix-optional`, on in `faithful`; T48a),
  its flag `quirk.town.aurora_ignores_map_bans` (T45b, once confirmed). **Stubs:** `TWN-STUB-6`
  and `-15` lifted (T48a); none planned (Town Portal's and Castle Gate's arrivals are increment 7's
  scope, the computer's book and re-visits E10's and E13's). **Disputes:** TD-10, TD-25, TD-26,
  TD-27 decided; none new (next TD-28). **Quirk candidates:** none new (next T-19).
- **Inbox notes** after the checkpoint: the adventure lane (the `cross-lane` changes in its paths:
  the outcome's field, `settle_outcome`'s call, `advance_day`'s refill, the import, the replay
  words, the game state hash; the merge order with its increment 14), the AI lane (the two entries),
  the server lane (the protocol's additions), the battle lane (the harvest's columns).

### 97. The gates and batch TWN-5

- **Unit tests** per rule (§83 to §93's lists).
- **Properties** (in `h3-map`'s plans, the adventure lane's files): a guild's slots never hold an
  excluded spell nor one twice; what it shows never more than the part's count plus the Library's;
  every hero's learned spells a superset across a town's trigger (learning never forgets); the
  Vortex at most once per town per week; a refused command leaves the hash; a fork's guild its
  parent's. The plans play `Open` and `BuySpellBook` in a fraction of their steps (the town lane's
  `TRAPS.md`: a new kind in `pick`).
- **A guild map** (the taverns map grown, or a new fixture: the task's call): a Tower with Mage
  Guild 4 and its Library buildable, a Dungeon with the Mana Vortex and a garrison hero, a Conflux
  with the Grail building (a custom set) and a guild, required and banned spells, a hero without a
  book. **Scripted replays:** the start's learning (TD-10), a visit, a build of a guild level and
  the Library with a visitor, a hire, the book bought, the opening's Vortex over two weeks, a
  siege's end. Golden replays re-recorded where a hash moves (`homm3-rerecord-replays`): at T47a
  every golden adventure replay (the store's hash) and the outcomes of those that play a week (the
  towns' stream moved by the draw, decision 179: the week's kind, the neutral garrisons, the
  Portals), each changed line classified; `h3-sim`'s town-map tests and the `*-towns-36` and
  month workloads too.
- **The census** (`h3-map`'s `original_data`): the 160 maps' guilds drawn (no excluded spell in a
  slot, every required spell shown where its level is built and its slot reached; the 14 random
  towns of `Heroes of Might not Magic` with short levels), the start's learning (the 33 main towns'
  heroes and 8 placed ones of §10 with a book: the main towns' learn, the placed ones not).
- **Benchmarks:** the build (`setup` workloads) re-baselined for the draws; the step's legal
  commands with the opening's predicate (cheap, but on every listing: measured,
  `homm3-bench-regression`); the night's refill in the week workload.
- **Batch TWN-5** (survey §18's five rows): on the capture harness as TWN-1 to TWN-4 (T44), its
  verdicts into the drafts at the `shared/` task (TWN-5-2 confirms T-18's flag before it is added).
  What the harness may lack (§18): the town screen opened from the town list in a recipe; the reads
  of a hero's spell points (+0x18) and learned spells (+0x3EA) at a turn's start; the writer's town
  spell sets (§84) and the scenario's spell bans (the header's, the adventure lane's writer,
  `cross-lane`), a Conflux with building 26 in a custom set. The batch's games are the original's,
  so the engine's stream choice does not touch it. *As run* (T44): specs `E09-batch-5a` (rows 1 to
  4) and `-5b` (row 5), twice each; every row as the reading, T-18 confirmed (survey §21).
- *As built* (T50, decision 195; `E09-log.md`): the census in `h3-map`'s `original_data`
  (`guild_differences`, 25 of the survey's counts), the properties `check_guilds`,
  `check_guild_step` and `check_guild_refusals` in the plans, the map `e09-guild-36` (a Necropolis
  for blue; its Tower requires and bans) with the replays `guild-*-s1`, the workload
  `day-start-guild-36`; the setup workloads and the listing measured at T47a2 to T49b.

### 98. The building tasks (the whole increment; the order with E08's increment 14)

The tasks continue the epic's ids; each task's cut confirmed at its start. **E08's increment 14
runs at the same time** (T133b to T135; its design note `E08-design.md` §21) and the client lane's
E27 M4 T3a (#1066, replay format v20): every `cross-lane` number is the next at merge, the second
to merge rebases and bumps again (the town lane's `TRAPS.md`); T45 and E08's increment 14 both
amend ADR-0016 and -0017; T47a's import after E08's T132b (import v27, merged).

| Round | Tasks | Content |
|---|---|---|
| 2 | **T44** | Batch TWN-5 (§97): the specs and templates (the writer's town spell sets, the scenario's bans `cross-lane`, a Conflux with its Grail building, a Dungeon with the Vortex), the harness's reads and the town screen from the town list, the recipes, the runs, the scoring. Needs the original data and the client lane's harness set-up. Cut at its start |
| 3 | **T45** | `shared/` (`cross-lane`): T45a the four drafts applied with T44's verdicts, the glossary (the guild's slots and spare, the guild level, the Aurora set, the Vortex's week flag, the opening), `docs/06-security.md`'s town lines (§94); T45b the flag of `TWN-QUIRK-14` (`h3-content`) once TWN-5-2 confirms it (no new stream: decision 179). In merge order with E08's increment 14's `shared/` task |
| 4 | **T46** | The town part v9 (the town lane's paths; the harvest `cross-lane`): the guild weights harvested from `SPTRAITS.TXT`, the four roles, the slots, the book's price, the Vortex's factor (TWN-GLD-001), the packs |
| 5 | **T47** | The guild drawn and shown (`cross-lane`): T47a the map's town spell sets (TWN-SET-012, `TownSetup`, the import v28, the writer), the draw from the towns' stream at the build (TWN-GLD-002, decision 179), what the guild shows (TWN-GLD-003), the store v4 (the slots, the flag, the Aurora set), the game state hash, the replays re-recorded (every hash; the week replays' outcomes); T47b the owner's view (TWN-VEW-004), the protocol |
| 6 | **T48** | Learning (`cross-lane`): T48a `teaches` with the Aurora and T-18 (TWN-GLD-004, `TWN-QUIRK-14`), the triggers in the town's own paths: a build and an event, a hire (lifts `TWN-STUB-15`), the start (TD-10, lifts `TWN-STUB-6`), the bars' moves (TWN-GLD-005); T48b the visit's end, a capture and a siege's end (`AdventureOutcome::town_entered`, `settle_outcome`, the siege's end) |
| 7 | **T49** | The spell points and the commands (`cross-lane`): T49a the night's refill (TWN-GLD-008, GAM-CAL-002), the Mana Vortex with its week (TWN-GLD-009), `TownCommand::Open` (TWN-GLD-006) with its listing, `town open`, the env's `TOWN_OPEN`, the protocol; T49b `TownCommand::BuySpellBook` (TWN-GLD-007), `town book`, the env's `SPELL_BOOK`, the protocol |
| 8 | **T50** + **T51** | T50: the guild map and its scripted replays, the census, the properties, the benchmarks, `--with-original-data` (§97). T51: the audit against the exit criteria; increment 5 closed; increment 6 (the services) refined |

*Agent's calls:* batch TWN-5 before the `shared/` task (its verdicts go into the ADRs, and
TWN-5-2 gates T-18's flag); the part first among the builds (no wait on E08); the draw before the
learning (learning reads the slots); the town's own triggers before the visit's (the visit's end is
the `cross-lane` hook in the adventure lane's files); the commands after the learning they call;
the Vortex with the opening (the opening's other half); the census last.

### 99. Increment 5's exit criteria (accepted at `TWN-Q33`, decision 181; the epic's list)

1. The town part's guild weights, the roles `guild`, `library`, `aurora` and `mana_vortex`, the
   slots, the book's price and the Vortex's factor compile from the packs (part v9; TWN-GLD-001).
2. Every town's guild drawn once when the game is built from the towns' stream: a required spell
   first in spell order, else by the faction's weights; the town's banned spells, the scenario's
   bans and Titan's Lightning Bolt never (TD-25); the spare in every town (TWN-GLD-002;
   TWN-SET-012).
3. What the guild shows by its standing level, the Library's one more, an empty slot ending a
   level, a level or the Library built later shown at once (TWN-GLD-003).
4. A hero with a spell book in a town with a guild learns the shown spells up to its learnable
   level; a Conflux with the Aurora teaches every spell of its Aurora set to that level, the
   scenario's bans by `TWN-QUIRK-14` (TWN-GLD-004).
5. The heroes learn at the visit's end, a capture, a siege's end, a build (an event's too), a
   town's hire, the start (TD-10), a bars' move with a visitor, the opening and the book's
   purchase; never at a turn start (TD-27; TWN-GLD-005).
6. `Open` gives the Vortex (the garrison hero, then the visitor), then teaches both, listed only
   where it changes something; `BuySpellBook` buys the core's book for the visitor else the
   garrison hero for the part's price; each refusal typed and changing nothing (TWN-GLD-006, -007).
7. The night's refill after the day's regain in every town with a guild (TWN-GLD-008;
   GAM-CAL-002).
8. The Mana Vortex once a week a town, twice the maximum, at the opening, a visit's end, a build and
   a hire (TWN-GLD-009).
9. The views: the owner's guild, Aurora mark and week flag, every change told; the fold gate passes
   without a new gap; the protocol maps the commands, errors and views.
10. The env plays the opening and the book (its two entries); the replays record `town open` and
    `town book`.
11. The census: the 160 maps' guilds drawn without an excluded spell, the main towns' heroes
    learning at the start.
12. Every new rule has its id, source and tests; `TWN-QUIRK-14` (T-18) recorded with its flag;
    `TWN-STUB-6` and `-15` lifted; the gates pass (coverage; determinism with the re-recorded
    replays; the census; the properties; the benchmarks at their baselines); batch TWN-5 run and
    scored, its consensus rows `verified`.

### 100. Risks and open points

- **Two lanes in the same crates at once:** E08's increment 14 (T133b to T135) and this
  increment's T45, T47 to T49 change ADR-0016 and -0017, the import, the game state hash, the
  replay format, the env's encoding and the protocol: merges in order, a rebase and a second
  re-recording when both bump.
- **The towns' stream moves** (decision 179): T47a's draw shifts every later draw of it, so every
  replay that plays a week changes its outcome and the growth's oracles that compare exact counts
  (the census of growth to day 29, `h3-sim`'s town-map seeds) move with it; T47a re-records and
  re-seeds them in one pull request, in merge order with the adventure lane's re-recordings.
- **Every trigger is a separate path:** a missed one (an event's build, a capture) leaves a hero
  without spells the original teaches; the census and the guild map's replays check them, the
  property "learning never forgets" does not catch a missing teach.
- **The opening's listing predicate** runs per owned town at every listing: measured at T49a; if
  dear, it caches per town by the state's version as the builds' offer does.
- **The harvest's 630 numbers** in nine records: the original-data test checks the sums and the
  wiki's chances (§3.5) as a second source.
- **Open:** the computer player's purchases and re-visits (E10, E13); an ally's visit and book
  (E10); Town Portal's and Castle Gate's arrivals (increment 7); RoE's `SPTRAITS.TXT` (E11); the
  town's guild window's view for a client (the shown spells are the view; the scroll pictures the
  client's).
