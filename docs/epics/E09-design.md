# E09 design note

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
  stack's creature by slot; 2 also each stack's size word (E08's monster words). The counts, the
  hall, the day's income and the silo's resources for the **owner's team**, or a viewer one of
  whose heroes has a Rogue in its army with the town on its level within its Visions radius
  (Visions' expert factor × the hero's power, at least 3; the factor a town part parameter until
  Visions is placed, *agent's call*). A neutral town is nobody's team.
- **Never public:** the town's other buildings, the Escape Tunnel, the defender's bonuses, the
  ever-built set; the merge's stacks (its owner's); a draw's values.
- **Events:** `TownCaptured` and `TownChanged` where the town is seen; the reveal to the new owner
  alone; a siege as any battle (its sides' owners), others learn its effects where they see the
  town.

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

### 17. The observation (T8c)

`GameObservation::towns: Vec<TownView>` in `TownId` order, the towns of which the viewer has
explored a tile: `town`, `anchor`, `owner`, `faction`, `name`, `fort`, `capitol`, `garrison:
GarrisonView` (`level`, then by level the creatures by slot, their words, their counts),
`owner_facts: Option<TownOwnerView { hall, income, silo }>` (§12's rules). The adventure's
`ObjectView::town` links an object to its view. `project` passes `TownCaptured` and `TownChanged`
to the viewers of §12. The map client (E24) and the env read the same (notes to their inboxes
when T8c lands).

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
T8b2b's); `TWN-STUB-8` a walled Stronghold's natural
obstacles (registered by T8b1; a siege-aware draw in `h3-battle`).

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
| 9 | **T8c** + **T9** | T8c: the income, the views, the observation, `docs/06-security.md` (§12, §16, §17). T9: the fixtures, replays, properties, benchmarks, the env (§18) |
| 10 | **T10** | The audit against the exit criteria; increment 1 closed; increment 2 refined |

**Increment 1's exit criteria** are the epic's seven; T10 checks each against its rules and tests.

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
