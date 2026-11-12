# E09 design note

Increment 1's note (closed 2026-11-06 by T10; Parts A to C, §1 to §22, kept here as the code cites
them: the epic's decision 40). Increment 2's note (construction, decision 37) is Part D, §23 to
§36, written by T13 (2026-11-07), kept at increment 2's closure (T20, 2026-11-11) as Parts A to C
were. Increment 3's note joins as Part E, its sections numbered from §37 (decision 67).

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
