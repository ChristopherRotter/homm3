# E08 design note: increment 5

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 5's amendment
drafts of ADR-0013 (point 13), ADR-0016 (points 3–5) and ADR-0017 (points 1, 3, 9), all three
drafted in T34b (`docs/lanes/adventure/adr-drafts/*-increment-5.md`) from the answers at ADV-Q20
(the epic's decision 75) and at T34b (decision 77: D-26 revised), accepted at ADV-Q21 with the cut
(decision 78; ADR-0016's with A-13's caveat restated). The task that builds a section
deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-hero-movement.md` (§ numbers are the survey's unless they name this note).

Increment 4's note was moved by T34b into `E08-log.md` ("Increment 4's design note at its
closure"): every section was built (T25 to T33). Four code comments still cite its sections as
`E08-design.md` §2–§13 (`h3-heroes`' `level.rs`, `level_up.rs`, `choice.rs`; `h3-game`'s
`state/levels.rs`) and the gap-fill's source `exe-necromancy` (battle-owned) its file: they mean
that copy until T39 and T40 repoint them (the queue's inbox).

Written whole (decision 76): **part A**, the hero side (`h3-heroes` and the core content); **part
B**, the map, the game, the batch and the cut.

## Part A — the hero side (`h3-heroes`, the core content)

Everything in part A is either core content (shared: `h3-content`, `h3-pack`, the packs through
the harvester) or a rule of `h3-heroes`: pure functions of a hero's details, the ruleset and
explicit inputs (ADR-0013 point 13 of the draft). When the map reads them is part B's.

### 1. The data

**Six hero-effect kinds** (`h3_content::HeroEffect`, codes 45 to 50 after `learn_level`; pack
names as below; `NAMES`, `code`, the canonical hash and the pack's checks grow by one each):

| Kind (pack name) | Fields | Several sources | The base pack gives |
|---|---|---|---|
| `pathfinding` | `level` 1..=3 | the highest | Pathfinding 1, 2, 3 |
| `land_movement` | `bp` ≥ 1 | summed, each skill's grown by its skill specialties | Logistics 1 000, 2 000, 3 000 |
| `land_movement_points` | `points` ≥ 1 | summed | Boots of Speed 600, Equestrian's Gloves 300 |
| `flying` | `level` 0..=3 | the highest | Angel Wings 3 |
| `water_walking` | `level` 0..=3 | the highest | Boots of Levitation 3 |
| `scouting_radius` | `tiles` ≥ 1 | summed | Scouting 1, 2, 3; the Speculum 1; the Spyglass 1 |

Each artifact type counts once, worn or as a part of a worn combination (`worn_types`, the
original's `hero_wears_artifact` 0x4D91F0); an artifact that needs a skill counts only with it.
The flying and water-walking levels are the original's index into its price table (0 to 3, the
spells' masteries later; §1.1 point 5, §2.1). No kind reaches `BattleEffect`: the battle's
translation skips them.

- **The packs** (generated: `crates/h3-harvest/data/gapfill.toml`, battle-owned, `cross-lane`): the
  three skills' levels get their effects, the six artifacts theirs and lose `pending: E08-E10`.
  Sources: `movement.txt` (column 6: rows 2 and 3, 300 and 600; the code gives row 3's to Boots of
  Speed 98, though the file names the Ring of the Wayfarer: §1.1), `skill_logistics` 0x63EA68,
  `skill_scouting` 0x63E9D8 (5, 6, 7, 8 less the base 5), the fly and water-walk levels forced by
  `hero_step_cost` 0x4B18C0 (Angel Wings 72, Boots of Levitation 90), the Speculum's and the
  Spyglass's +1 in 0x4E4070; VCMI's `config/skills.json` and `config/artifacts.json` and the wiki
  agree on the amounts (§1.2, §4.1). The Logistics specialists (Kyrre, Gunnar, Dessa) already
  carry `skill_specialty` logistics 500 bp; the creature specialists `creature_specialty` with
  `speed = 1`. `packs/README.md`'s effect list (battle-owned) gains the six kinds.
- **The adventure part** (`packs/*/adventure.toml`, `h3-pack-adventure`; adventure-owned): the
  `[params]` gain `fly_cost` and `water_walk_cost`, each `[140, 140, 120, 100]` by level
  (`fly_walk_step_cost` 0x6778AC, one table in the original; two fields, so that data can tell
  them apart as VCMI's spells do: agent's call); the `ab` pack gains a **terrain walker**,
  `[[terrain_walker]] creature = "nomad", terrain = "desert", counts_as = "dirt"`
  (an army with a stack of the creature counts the terrain as the other for every price of a step
  from it, the diagonal's bound included: §2.1 point 1; the core's desert is the original's sand);
  **if batch E5's row E5-6 confirms D-23**, the `cursed_ground` object type's
  `ground = { battlefield = "cursed_ground", no_native = true }`. Adventure ruleset part v10.
- **Three quirk flags** (`h3_content`'s registry, the next free positions, 30 to 32 unless the
  battle lane takes one first), each on in `faithful`, off in `strict-fixed`:
  `quirk.adventure.flying_inland_price` (`ADV-QUIRK-16`, D-24),
  `quirk.adventure.drop_ignores_artifacts` (`ADV-QUIRK-17`, A-12),
  `quirk.adventure.radius_waits_for_a_step` (`ADV-QUIRK-18`, D-26 revised). The faithful core
  ruleset hash moves with them (both lanes' golden replays' headers).
- **D-22** needs no flag of its own: Q-0005's `quirk.battle.single_precision_rates` (§1.2).

### 2. The derivations (HER-ADV-003 to -007)

New in `h3_heroes` (a module `movement.rs`; names and signatures the agent's call):

- **`land_points(details, core, base) -> u32`** (HER-ADV-003): `base`, the land table's entry the
  adventure gives (§5), grown by the sum of the hero's `land_movement` rates (each skill's grown by
  its skill specialties, `grown_effects`), rounded down, then the sum of `land_movement_points`
  added (saturating). The growth is the existing `grown` helper's, as `gained_experience`'s: with
  Q-0005 `trunc((1 + the single-precision sum) × base)` in the x87 arithmetic at 53 bits, the rate
  `f32(r × (f32(level) × 0.05 + 1))` (`Growth::single`; the original's 0x4E4CBB); without it the
  exact grown basis points (`Growth::grow`; exact for Logistics, whose 5 % of 10 %, 20 %, 30 % per
  level is a whole number of basis points). **Vectors**: survey §1.3 (1 500 at Basic: 1 657, 1 664,
  1 672, 1 694, 1 739, 1 800 at levels 1, 2, 3, 6, 12, 20 with Q-0005; 1 665, 1 695, 1 740 exact at
  2, 6, 12; 2 000: 2 210, 2 219, 2 230, 2 259, 2 289, 2 319 at 1, 2, 3, 6, 9, 12, exact 2 220, 2 260,
  2 290, 2 320), E5-1's (1 560: 1 716, 1 872, 2 028; +600 2 160; +300 1 860; Expert with both
  2 928), and without a specialty `⌊points × (10 + level) / 10⌋` for every table entry in both
  profiles (§1.3's last sentence).
- **`movement_speed(details, core, creature) -> u8`** (HER-ADV-004): the creature's speed in the
  core (its table's, no effect), plus the `speed` of each of the hero's `creature_specialty`
  effects whose creature is it or one of its direct upgrades (`CreatureSelector::Creature`'s
  selection, `Ruleset::upgrades`), saturating; `u8::MAX` for a creature the core lacks (today's
  rule). Not modelled: the original's four elementals without an upgrade on a RoE map (mapKind 0,
  §1.1 point 2); the packs decide upgrades. No speed artifact and no `creature_stat` counts (the
  code and the wiki, §1.2).
- **`pathfinding(details, core) -> u8`** (HER-ADV-007): the highest `pathfinding` level, 0
  without.
- **`flying(details, core) -> Option<u8>`, `water_walking(details, core) -> Option<u8>`**
  (HER-ADV-005): the highest level, none without a source.
- **`scouting_radius(details, core, base) -> u8`** (HER-ADV-006): `base` (the adventure part's
  5) plus the sum of `scouting_radius` tiles, saturating. A skill specialty on Scouting grows
  nothing (ADR-0013 point 13.4 of the draft).
- **`movement(details, core) -> Movement`** (agent's call): `Pathfinding`, flying and
  water-walking levels in one pass, for the adventure's pricing (derived once per search or move,
  never per step; the reach's hot loop).

Test rulesets are built in code (`h3-heroes`' dev-dependencies obey the layers); the base pack's
data are checked by `h3-pack`'s harvested tests (the battle lane's pattern for Q-0005,
`harvested_magic_derives`).

### 3. Rule ids and quirks (the hero side)

- **HER-ADV-003** (the land points' bonuses; D-22 under Q-0005), **HER-ADV-004** (a creature's
  speed for the day's points), **HER-ADV-005** (the flying and water-walking levels),
  **HER-ADV-006** (the scouting radius), **HER-ADV-007** (the Pathfinding level; new beside the
  survey's §10, since the step's price may read no skill by name, ADR-0016 point 4), in
  `heroes.toml` with the code reading (0x4E4990, 0x4E4070, 0x4B18C0), VCMI and the wiki as
  sources (§1.2, §2.2, §2.3, §4.1).
- **Q-0005** (the battle lane's ledger, `cross-lane` in T35 or a line in the battle lane's
  inbox): its rules list gains HER-ADV-003.

### 4. The hero side's shared and `cross-lane` needs (for the cut, §12)

- `h3-content` (shared): the six kinds and the three flags; `h3-pack` (shared): their syntax;
  pack format v46, core ruleset hash v47 (the next numbers when T35 lands: the battle lane shares
  both).
- The harvester's gap-fill and the regenerated packs, `packs/README.md` (battle-owned,
  `cross-lane`); both lanes' golden replays' headers (the faithful core ruleset hash).
- Exhaustive matches over `HeroEffect` outside `h3-content`: `h3-heroes`' battle translation (a
  new arm that skips them), the harvester, and whatever `clients/` builds (`clients/scripts/
  check.sh`, or `cargo build --manifest-path clients/Cargo.toml`): T35 runs both.

## Part B — the map, the game, the batch and the cut

Rules of `h3-adventure` and `h3-game` around part A's: when a hero's properties are read, how
its steps are priced and where it may pass and stop. The drafts of ADR-0016 and ADR-0017 hold the
decisions.

### 5. The day's points and the radius; the build and the day (ADV-MOV-001, -002, ADV-FOG-001, -003)

- **ADV-MOV-001 (amended):** `day_points(core, part, details, army)`: the land table
  (`part.params().land_points`, its last entry for every faster speed) at the lowest
  `h3_heroes::movement_speed` of the army's creatures, then `h3_heroes::land_points` on it. A
  creature specialist's +1 can lift the slowest speed past the table's end: the last entry.
- **ADV-MOV-002 (amended):** `start_turn(core, part, heroes, player)` reads the store (the game
  passes it as to `apply` and `reach`). A gain in the day (a level-up's Logistics, an artifact)
  changes nothing until then (§1.4).
- **The build** (ADR-0017 point 1 of the draft): `AdventureState::new` places the heroes and no
  longer reveals or prices; a second step (`AdventureState::start_game(core, part, heroes)`,
  agent's call) gives each hero its day's points and reveals around it with its radius
  (ADV-FOG-003), called by `GameState::new` after the levels at load (the original's order:
  `placed_hero_setup` 0x4D8B30 sets the movement points after the levels, `placed_heroes_setup`
  reveals after it, 0x4CAF74). The adventure crate's own tests and `testing.rs` call both steps.
- **ADV-FOG-001 (amended):** a hero's radius is `h3_heroes::scouting_radius` on the part's base,
  derived once per move (a move changes no details).
- **ADV-FOG-003 (amended):** reveals at the start (as above) and after each step, also each step
  over a tile a flight passes (the original reveals at every step's destination, 0x48106B); a
  grown radius shows at the hero's next step (`ADV-QUIRK-18` on). **`strict-fixed`**
  (`ADV-QUIRK-18` off): the adventure's start-of-day step (`AdventureState::start_day(core, part,
  heroes)`, agent's call) reveals around every hero on the map to its owner, which the game calls
  in `advance_day` after the calendar and before the first group's turn, and in the build for day
  1 (where it adds nothing). No event, as the start's and the steps' reveals have none of their
  own: the owner's observation shows the explored tiles. The rule's summary no longer claims an
  original new-day reveal (§4.2's correction; the catalog is corrected by T34b, the code comment
  in `state.rs`' `reveal_at_start` by T40).
- **Rows**: E5-1, E5-2, E5-3, E5-10.

### 6. The step's price (ADV-MOV-003 to -006 amended, ADV-MOV-009, -010)

`Pricing::new(core, part, army, movement)` takes the hero's `Movement` (§2) and precomputes, per
terrain, the walking price at the hero's Pathfinding level (`part.step_cost`), so the hot path
keeps one load per step. A step's price from `from` to `to` (the original's `step_cost` 0x4B1740,
§2.1):

1. **Nomads** (ADV-MOV-003): with a stack of a terrain walker's creature in the army, the source's
   terrain counts as the walker's `counts_as` for all below, the diagonal's bound included.
2. **The walking price:** both tiles with a road: the source road's price (flat over
   Pathfinding); else the native cost when the source's terrain is the army's and the source is
   not under a ground with `no_native` (Cursed Ground, only if built); else the source terrain's
   price at the hero's Pathfinding level.
3. **Water walking** (ADV-MOV-010; level `w`): from a water tile, `water_walk_cost[w]` replaces
   the price.
4. **Flying** (ADV-MOV-009; level `f`): from a land tile with the coast flag (`StaticMap`'s coast
   bits), the lesser of `fly_cost[f]` and the price so far; from any other tile (inland land or
   water) `fly_cost[f]` replaces it (`ADV-QUIRK-16` on). `strict-fixed` takes the lesser from
   every tile (the wiki and VCMI).
5. **The diagonal** (ADV-MOV-005, -006): the exception's check cost C, under `ADV-QUIRK-2` or
   `-3`, is the walking check of today's code (the source terrain's price at the Pathfinding level
   after Nomads; water's 100), never the fly or water-walk price (the original); with both off,
   C is the step's own straight price, whatever priced it, the fly or water-walk price included
   (the existing fix's natural extension: agent's call, ADV-Q21 point 4). At level 3 only
   flying from rough terrain tells the two apart (a swamp's diagonal at 175 to 247 points costs
   100 in `faithful`, 141 in `strict-fixed`).
6. **The drop to 0** (ADV-MOV-006, A-12): after each step the points left fall to 0 when below
   the cheapest straight step from the new tile, priced against itself: its road's price, else
   the native cost (the terrain's price at the Pathfinding level, after Nomads, under a
   `no_native` ground), then water walking (on water) and flying (inland: the fly price; on the
   coast: the lesser) by the hero's **spell** levels under `ADV-QUIRK-17` (none in increment 5:
   a walking step), by its flying and water-walking levels without it (§2.5). Today's
   `left_after` is the walking case.
7. **Cursed Ground** (D-23): built only if batch E5's E5-6 shows 175 from swamp under it; then the
   static map keeps a bit per tile for the tiles a `no_native` ground lies on (as the coast's),
   read by points 2 and 6.

`Pricing::cheapest` (the reach's window bound) includes the fly and water-walk prices the hero
has.

### 7. The passage, the reach and `MoveHero` (ADV-MOV-007 to -010)

**The tiles by how a hero moves** (§3.1, the second reading; A-13 fixed, A-14 withdrawn).
"Stop" is a node the hero may end a move on and visit from; "pass" a node it only flies or walks
over; a flying hero that also water-walks is a flyer:

| The step enters | Walking | Flying | Water walking |
|---|---|---|---|
| off the map, rock, an unexplored tile | never | never | never |
| a free land tile, a gate whose key the owner holds | stop | stop | stop |
| a blocked tile (live occupancy) | never | pass | never |
| plain water | never | pass | pass |
| water holding an object | never (boats later) | pass | never |
| an entered object's visitable tile (a Cartographer) | stop: the visit ends the move | stop (the visit, from any node) and pass | stop |
| a neighbour-visited object's visitable tile (a pile, a monster, a border guard or gate) | a visit from the tile before, which ends the move | the same from a stop node; else and also pass | as walking |
| another player's hero's tile | a visit from the tile before: a battle | the same from a stop node; else and also pass | as walking |
| an own hero's tile | never (meetings later) | pass | never |
| a guarded tile | stop: the move ends, the guard visits | stop and the flight goes on; the guard only at the end | as walking |
| down onto the visitable tile of an object not visited from above | never | never | never |
| up off such a tile (the hero's own, or one flown over) | never | pass | never |

A town (98) is entered as a stop only from a stop node too (§3.1 point 6; towns are E09). The
start node is a stop node. Rock never; a hero in a boat neither flies nor water-walks (boats
later).

**The reach** (`Reach::search`): a walking hero's search is today's (one node per tile; no new
test in its hot loop: a const generic or a second function, `bench-check`). A flying or
water-walking hero's window holds a second layer of pass nodes (points left and the entering
direction), and each direction byte gains the predecessor's kind (the original's bit 11,
0x4B1B15): Dijkstra's over (tile, kind), each node replaced only by a strictly better one of its
kind, as the original's two nodes per tile (0x4B1C4B). The drop to 0 applies at every step of
either kind. `points_left`, `path` and `tiles` read the stop nodes; `path` follows each node's
predecessor of its recorded kind. The env's mask and `legal_moves` are the stop tiles (no
encoding change). Properties: every path the reach reports is one `MoveHero` accepts, every tile
one step away a stop tile is reported, and the exhaustive search's equality without the diagonal
exception holds over both kinds.

**`MoveHero`** (ADV-MOV-007 amended): validated step by step with the node kind the table gives;
a path may go on after a pass. New: `AdventureCommandError::NoStop { step }` (name the agent's
call), the path ends on a pass, or its last step visits from a pass; `Blocked` and `PastVisit`
keep their meaning for a walker (a flyer's step over an entrance mid-path is a pass, not a
visit). The `Unexplored` check stays first (nothing about a hidden tile leaks). A flyer's guard is
tested at its last tile only, and only that tile's object is visited (0x48147D to 0x4814E1). Each
step, passes included, emits `HeroStepped` with its viewers and reveals (§5).

### 8. The env, replays, fixtures, properties and benchmarks

- **The env** (`h3-sim`'s adventure sub-path): no encoding change (action v4); the mask is the
  reach's stop tiles; `random_rollout` plays flyers and water walkers as any hero.
- **The plans** (`h3-map`'s `play.rs`): heroes wearing each of the six artifacts and with the three
  skills (one flyer, one water walker per plan where the generated map has water), an army with
  Nomads under the `ab` pack, Cursed Ground objects if built; the seven properties and the
  copy-on-write ones cover them; observation hygiene compares the legal moves too (A-13's fix: a
  flyer's moves never depend on an unexplored tile).
- **Fixtures and replays:** the duel and XL maps (`fixtures/maps/`) gain a flyer and a water
  walker and heroes with the three skills; the nine golden replays are re-recorded (their
  commands change where a search does; the five searched ones may be replaced as in T31).
- **Benchmarks:** the reach workload stays within its +2 % (the walker's search unchanged); a
  flyer's reach on the XL map is baselined as a new workload or a variant of `reach`.
- **`--with-original-data`:** the 5 shipped maps import as before; the Tutorial's hero (Expert
  Logistics, Boots of Speed, Equestrian's Gloves) gets its day's points printed and asserted
  (its army's entry × 1.3 + 900).

### 9. Rule ids and quirks (the map side)

- **Amended:** ADV-MOV-001 (§5), -002 (§5), -003 (§6 points 1, 2, 7), -005 and -006 (§6 points
  5, 6), -007 (§7), ADV-FOG-001 and -003 (§5).
- **New:** **ADV-MOV-009** flying (the price §6.4, the passage §7, the end of a move §3.4),
  **ADV-MOV-010** water walking (the same).
- **Quirks** (`docs/rules/quirks-adventure.md`, each written with its rule): **ADV-QUIRK-15** a
  flight crosses unexplored tiles (A-13; fixed for now, no flag: Q-0002's class, **provisional**:
  the user's caveat at ADV-Q20 and ADV-Q21, §13; its class is revisited with the compromise), **ADV-QUIRK-16** flying costs the fly price from inland tiles, roads included (D-24;
  fix-optional), **ADV-QUIRK-17** the drop to 0 forgets the artifacts (A-12; fix-optional),
  **ADV-QUIRK-18** a grown radius waits for the hero's next step (D-26; fix-optional, decision 77).
  A-14 is withdrawn (no entry); D-23 is a rule (Cursed Ground) or nothing, after E5-6.

### 10. Versions

Increment 4's last: core ruleset hash v46, pack format v45 (the battle lane's E23 T3-0), game
state hash v11, adventure state hash v10, adventure ruleset part v9, map import v8, action
encoding v4, adventure replay format v5. Increment 5: the pack format v46 and the core ruleset hash
v47 (T35: the six kinds, the three flags), the adventure ruleset part v10 (the fly and water-walk
prices, the terrain walker; Cursed Ground's `no_native` in the same bump if E5-6 is scored first,
which the cut does). The game and adventure state hashes, the map import, the action encoding and
the replay format do not change (no new state; a flyer's destinations are tiles of the same
mask). Each bumps once, in the task that first changes it.

### 11. Batch E5

The plan is the survey's §9: rows E5-1 to E5-10, E5-8 dropped (A-14 withdrawn), E5-10's
prediction corrected by T34b (§4.2: nothing new after the battle nor at day 2; the radius 6 after
the first hero's step). *Agent's call (change freely):* **E5-11**, an Angel Wings hero facing a
border gate whose key it lacks, clicks a free tile beyond it; the reading predicts a path over the
gate (an entrance a flyer passes, §3.1 point 7), which no other source states (§13).

- **Spec** (`experiments/specs/E08-batch-5*.toml`, `h3-experiment` and `docs/experiments/`
  battle-owned: `cross-lane`): heroes wearing artifacts (`equipment`, since batch E4), a ground
  under a hero's tile (E5-6: the writer places grounds since E04c T3; check under a hero),
  unexplored strips (a hero far enough from the explored start), a border gate (E5-11:
  `[[map.object]]`), a Fortress army on swamp (E5-6), Gunnar and Kyrre at levels 2, 3, 6, 12 with
  Basic Logistics (E5-2: a custom experience gives levels at load, so the scorer predicts their
  skills after them; or levels the seed byte keeps clean of Logistics), a level-up the test
  forces (E5-3, E5-10: the seed byte's prediction, as batch E4's). Split by rows if a map cannot
  hold them all (batch E4's E4b precedent).
- **Scorer:** most rows predict fixed numbers (§9); E5-2, E5-3 and E5-10 need the seed byte's
  draws and the levels at load: `crates/h3-map/examples/e4_score.rs` extended or an `e5_score`
  beside it (on `h3-heroes`, `h3-map`'s dev-dependency since `shared/e08-t32a`), with the
  results template.
- **The harness's reads** (`clients/capture/`, client-owned; built by this lane, `cross-lane`, as
  T32b1): a tile's explored bit for a player (`explored:x,y`, the vision word against the player's
  bit; E5-9, E5-10, E5-11), a hero's points at a turn start without a walk (check whether a read
  step's `points` gives it), the next turn's start after `end_turn` (exists), the path's mark
  (exists).
- **The run and the scoring:** on the harness (decision 50), then scored: D-22 (E5-2), D-23
  (E5-6: Cursed Ground built or not), D-24 (E5-4), D-25 (E5-7), D-26 (E5-10), A-13's original
  path (E5-9), E5-11; the consensus rows (E5-1, -3, -5) as checks.

### 12. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T35's `shared/` pull request (the three accepted drafts applied;
`h3-content`'s kinds and flags, `h3-pack`'s syntax; part A §4) with the gap-fill and the packs
(battle-owned) and both lanes' replays' headers as `cross-lane` in it, as T26b; batch E5's spec
(`h3-experiment`, `docs/experiments/`; T36) and the harness's reads and recipes (`clients/capture/`;
T37), `cross-lane`; the client lane's inbox if `clients/` needs the new kinds mapped. Everything
else is the lane's own (`h3-heroes`, `h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, the
adventure packs, the env's and the CLI's adventure sub-paths, fixtures, replays, benchmarks).

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). *Confirmed at ADV-Q21 (decision 78):* batch E5 runs **before** the build (rounds 1 and 2), so
that its answers (D-23's Cursed Ground above all, and D-22, D-24) shape the rules once instead of
reopening them, and nothing in the batch needs the engine's increment 5 (it tests the original;
the scorer needs only increment 4's draws):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T35** + **T36** | T35: the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016 and -0017 amended from the accepted drafts; the six kinds, the three flags, their syntax; the packs' effects through the gap-fill; pack format v46, core ruleset hash v47; both lanes' replay headers; `clients/` checked (part A §1, §4). T36: batch E5's specs, results templates and scorer (§11; `cross-lane`) |
| 2 | **T37** + **T38** | T37: the harness's reads, the recipes and the run of batch E5 (`cross-lane`). T38: batch E5 scored, D-23 decided by E5-6; the survey's §8 and the rules' sources updated; an ADV-Q if a row contradicts the reading |
| 3 | **T39** + **T40** | T39: `h3-heroes`' derivations (part A §2; HER-ADV-003 to -007, §1.3's vectors; Q-0005's line). T40: the day's points, the radius and the walking price (§5, §6 points 1, 2, 7): ADV-MOV-001 to -004 amended, the build's order, `start_turn` and the reveals with the store, `ADV-QUIRK-18` and the strict start-of-day reveal, Nomads, Cursed Ground if confirmed; adventure ruleset part v10 |
| 4 | **T41** | Flying and water walking (§6 points 3 to 6, §7): the price (`ADV-QUIRK-16`), the drop (`ADV-QUIRK-17`), the passage, the reach's two kinds of node, `MoveHero`'s passage, end, visits and guards; ADV-MOV-006, -007 amended, -009, -010; `ADV-QUIRK-15` to `-17`. The largest task: cut at its start if needed (the price and the drop; the reach and `MoveHero`) |
| 5 | **T42** + **T43** | T42: the env, the plans and properties, the fixtures and replays, the benchmarks, `--with-original-data` (§8). T43: the audit against the exit criteria, increment 5 closed, increment 6 refined |

*Not chosen* at ADV-Q21: the batch at the end as in increment 4 (T35, T39 to T42 first, then
the batch and its scoring: the order of increments 3 and 4, at the risk of reopening a built rule
and building Cursed Ground late).

**Increment 5's exit criteria**:
1. A hero's day's points follow its Logistics (a specialist's growth by Q-0005), its creature
   specialty's speed and its worn Boots of Speed and Equestrian's Gloves (HER-ADV-003, -004,
   ADV-MOV-001), set at the build after the levels at load and at each turn start of its owner
   (ADV-MOV-002).
2. A step's price follows the hero's Pathfinding, Nomads in its army, flying and water walking
   (`ADV-QUIRK-16` by the profile), and Cursed Ground if batch E5 confirmed it; the drop to 0
   follows `ADV-QUIRK-17`.
3. A flying or water-walking hero passes as §7's table, never over an unexplored tile
   (`ADV-QUIRK-15`), and ends a move only where it may stop; the reach reports those tiles with
   paths `MoveHero` accepts; the env, the replays and the plans play flyers and water walkers.
4. A hero's radius follows Scouting and its worn Speculum and Spyglass (HER-ADV-006); a grown
   radius shows at the hero's next step, in `strict-fixed` also at the next day's start
   (`ADV-QUIRK-18`).
5. Every new rule has its id, source and tests; `ADV-QUIRK-15` to `-18` are in the ledger and
   Q-0005's entry names HER-ADV-003; the gates pass (coverage, determinism with the re-recorded
   replays, the benchmarks with the walker's search at its baseline and a flyer's baselined).
6. Batch E5 run and scored; D-22 to D-26, A-13's original path and E5-11 `verified` or reopened.

### 13. Risks and open points

- **The passage table rests on the code reading alone** where the wiki and VCMI are silent: a
  flyer passing over a border gate without its key, over another player's hero, and a step up
  off an object not visited from above landing only in flight. Fixed tests pin the reading;
  E5-11 checks the gate in SoD; the wiki's Border Gate and Border Guard pages are worth a look in
  T41. The reading also missed something about 0x4E5CE0 (A-14, withdrawn): its callers' start
  node (§3.3) is built as a walker's, and the gap stays in the queue's inbox.
- **A-13's cost** (the user at ADV-Q20 and ADV-Q21: "probably fix, but gameplay impacts have to
  be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like
  allowing to route to the closest unexplored, but free hex; TBD later anyway"): a flight over a
  wide obstacle range stops at its unexplored edge. Increment 5 builds the fix; the compromise is
  designed later (the queue's inbox), with what a flight's end beyond hidden tiles may tell the
  player (`docs/06-security.md`). Keep the reach's passage rules in one place, so that the
  compromise changes the unexplored tile's row of §7's table and nothing else.
- **The reach's speed:** the second layer only for flyers and water walkers; the walker's hot
  loop must not gain a branch (`bench-check`'s +2 %).
- **Not in increment 5** (survey §6): the sea's points (Navigation, its specialists, the
  Necklace of Ocean Guidance, the Sea Captain's Hat, the Admiral's Hat, Lighthouses, Favorable
  Winds), Stables, the spells Fly, Water Walk and Dimension Door with their levels 0 to 2 (the
  day's end of a spell, `new_day` 0x4C80CF), the computer players' bonus (E10), the cheat flag,
  the new-day reveal after a Cover of Darkness (E09).
