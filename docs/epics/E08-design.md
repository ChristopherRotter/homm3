# E08 design note: increment 15 (the Prison, the Market of Time and the adventure spells)

Written by T137b (2026-11-21; E08 decisions 394, 395, 397 and the checkpoint's 398 to 406). The
code cites it as "increment 15's design note §M". Increment 14's note (the trades and services) is
in `E08-log.md` ("Increment 14's design note at its closure", moved by T136), increments 9's to
13's beside it. Inputs: the survey `docs/rules/surveys/E08-spells.md` (§N below is the survey's
when it says so), ADR-0013, ADR-0015 to -0017. Drafts: `docs/lanes/adventure/adr-drafts/`
`0013-increment-15.md` (point 2 and a new point 17: the mastery on a ground, the cost on the map),
`0016-increment-15.md` (points 1, 3, 4, 5, 6) and `0017-increment-15.md` (points 1, 4, 8, 9, 10).
ADR-0015 needs none (no crate, no new dependency: the game hands the adventure the ground's raise,
§5).

**The checkpoint's answers** (ADV-Q63, the user at T137b's start; decisions 398 to 406): the note
whole (398); the flight over the unexplored and the door's hidden targets as **fix-optional
flags**, the compromise their fixed side, a failed door onto a hidden tile revealing it (399);
Summon Boat's choice a fix-optional flag, its fixed side by straight distance on the hero's level
(400); the door's range 9 × 8, the extra row fixed (401); Scuttle Boat's hidden boats by the same
compromise behind a flag (402); A-49 accepted, A-51 and A-52 fix-optional (403); a book needed
until row E15-15 (404); the mastery on a ground in `h3-heroes`' derivation (405); the four
information spells to E12 (406).

## Part A — the content

### 1. The Prison kind (`ObjectKind::Prison`; ADV-PRI-001 to -003)

The base pack's type `prison` (class 62, subclass 0) of the kind `prison` (no parameters).
**Visited from a neighbour** (the prisoner stands on the entrance: the freed hero is put there,
survey §2.2 point 3; T139a confirms the class against `object_classes_not_entered` 0x63AA58, E08
decision 317's rule), every time, by every player. The object holds its prisoner's `HeroId` in the
static map (§7). No new parameter: the eight heroes are the town part's most heroes (TWN-TAV-002's
8), which the game reads (§15).

### 2. The Market of Time (`ObjectKind::Empty`; ADV-OBJ-008)

The base pack's type `market_of_time` (class 50) of a new kind `empty`: **entered, never visited**
(a step onto its entrance at a step's price ends the move there, as every entrance; nothing
happens, no event). *Agent's call:* a kind of its own rather than the inert blocker plus an
entrance, so E12's other classes without a visit (`visit_object`'s out-of-range case, survey §3:
18, 19, 21, 26, 36, 40, 46, 65 to 77 …) take it by data. Row E15-3 records the step's price.

### 3. The spells' data

- **Which spells act on the map** is the adventure part's: `[[spell]] key = "base:summon_boat",
  effect = "summon_boat"` and so on for the five, a closed `AdventureSpell` enum (`summon_boat`,
  `scuttle_boat`, `fly`, `water_walk`, `dimension_door`) keyed by the core's spell keys (ADR-0006
  tier 2: the behaviour closed, the binding data). A spell of the core's adventure category
  without an entry is not castable (the four of §9 and Town Portal until E12 and E09 increment 7:
  ADV-STUB-19, ADV-STUB-20, §18).
- **The values already in the core** (harvested, E04): each spell's cost by mastery and its effect
  column by mastery: Summon Boat's and Scuttle Boat's chance (50, 50, 75, 100), Dimension Door's
  casts a day (1, 2, 3, 4). The adventure reads them through `h3_content` (no copy).
- **New parameters** (`[params.spells]`, adventure ruleset part v25): `door_points = [300, 300,
  300, 200]` (by mastery; ADV-SPL-009), `target_range = [9, 8]` (dx, dy: the door's and Scuttle
  Boat's window, D-76 and A-54, decision 401), `boats_most = 64` (Summon Boat's creation, §10.1).
  The fly and water-walk prices are increment 5's `fly_cost`, `water_walk_cost` (levels 0 to 3).

### 4. The quirk flags (decisions 399, 400, 402, 403)

| Id | Candidate | Class | Flag | `faithful` | `strict-fixed` |
|---|---|---|---|---|---|
| ADV-QUIRK-15 (reclassified) | A-13 | fix-optional | `quirk.adventure.flight_reads_hidden` | on: the original's reach over hidden tiles | off: the compromise (§11.1) |
| ADV-QUIRK-51 | A-53 | fix-optional | `quirk.adventure.door_reads_hidden` | on: a hidden tile a target when free | off: every hidden tile a target, the truth at the cast (§11.2) |
| ADV-QUIRK-52 | A-50 | fix-optional | `quirk.adventure.scuttle_reads_hidden` | on: a hidden empty boat a target | off: every hidden tile a target (§11.3) |
| ADV-QUIRK-53 | D-75 | fix-optional | `quirk.adventure.summon_boat_by_steps` | on: the reading's choice | off: by straight distance on the hero's level (§10.1) |
| ADV-QUIRK-54 | A-51 | fix-optional | `quirk.adventure.spell_recast_refused` | on: a re-cast refused while on | off: a re-cast at a higher mastery raises the level (§10.3) |
| ADV-QUIRK-55 | A-52 | fix-optional | `quirk.adventure.door_failure_charged` | on: a wrong water or land target fails, charged | off: an explored wrong target not offered (§10.5) |
| ADV-QUIRK-56 | A-54 | bug-fix | none | — | — (the window 9 × 8 always) |
| ADV-QUIRK-57 | A-49 | harmless, accepted | none | — | — (Water Walk cast in a boat) |

Each entry is written into `docs/rules/quirks-adventure.md` by the task that builds it, after
batch E15 confirms its reading (the ledger's rule: a code reading confirmed by a second look; the
rows of §19). The flags join `packs/base/core.toml`'s two profiles (the core ruleset hash moves:
shared, `cross-lane`, at the `shared/` task, §20). **The AI trains under the profile played**
(E08 decision 7): with every new flag on in `faithful`, its legal moves read hidden tiles (the
original's leaks); `strict-fixed` keeps ADR-0016 point 4's rule that the legal moves never depend
on a hidden tile.

## Part B — the hero (`h3-heroes`; ADR-0013 draft)

### 5. The mastery on a ground (HER-SPL-003 amended; decision 405)

`h3_heroes::spell_masteries_on(details, core, raise: SchoolRaise)` beside `spell_masteries` (which
becomes `spell_masteries_on(.., SchoolRaise::None)`): `SchoolRaise` is `None`, `All` or
`Schools(SchoolSet)`; a raised school gives Expert to every spell among its schools (the original's
`hero_school_mastery` 0x4E5100 with the field: survey §4.2), the rest as today. The caller derives
the raise from the ground: the special ground's battlefield of the battle part
(`Battlefield::expert_schools` and `raises`, BAT-TER-005's data: Magic Plains all, Lucid Pools water, Fiery Fields fire,
Rocklands earth, Magic Clouds air). On the map the **game** does it (it holds the battle part,
ADR-0015 point 2), from `AdventureState::special_ground` of the hero's tile, and hands the mastery
to the adventure's cast (§14), so `h3-adventure` and `h3-heroes` need no battle content. The
battle may call the same function for its hero casts (BAT-TER-005's hero half; the battle lane's
inbox, its choice of when: `cross-lane` only when it does), so one rule serves both.

### 6. The cost on the map (HER-SPL-008, new)

`h3_heroes::map_spell_cost(details, core, spell, raise)`: the core's cost at the mastery on that
ground, at least 1; **no army's ±2** (Pegasi, Mages: battle-only, 0x4E52DA). The spell points are
spent by the existing `HeroChange::SetSpellPoints` (at least 0). The book (D-79, decision 404):
`h3_heroes::can_cast_on_map(details, core, spell)`: the spell among `available_spells` and a spell
book worn (HER-BAT-011's test), until row E15-15 decides; a hero without a book then has no cast.

## Part C — the map (`h3-map`, `h3-adventure`)

### 7. The import and the writer (`h3-map`; map import v29)

- **Prisoners:** each class-62 body becomes a `PrisonerSetup` in `GameSetup` (today its type only):
  the type, the hero body's customisations read as a placed hero's (experience, army, skills,
  artifacts, spells; name and portrait the clients'), the prison's footprint. The game builds its
  details as a placed hero's (§15). The census (§1.1: none on water, no type twice, none allowed or
  restricted) makes these refusals unreachable on the shipped maps; they stay typed
  (`Unsupported::PrisonerOnWater`, the existing duplicate-type error).
- **Class 50** as the `market_of_time` type (subclass 0; another subclass refused, no body).
- **The writer** (`h3_map::writer`): the prison with a hero body (type, experience, army, skills,
  artifacts; for batch E15 and fixtures), class 50 among `PLAIN_OBJECTS`, a placed hero's learned
  spells and worn scrolls (if not yet written), boats with a last captain are not a map field (the
  harness sails first, §19).

### 8. The state (`h3-adventure`; adventure state hash v28)

- **The prison** is an object of the static map whose kind is `Prison` and whose drawn content is
  its `HeroId` (beside every object's drawn contents, ADR-0016 point 1); the freeing removes the
  object (increment 2's occupancy). No new block.
- **The hero's day spells** (`HeroOnMap::spells: DaySpells`, plain): `fly: Option<u8>`,
  `water_walk: Option<u8>` (the spell's level 0 to 3, none when off; the original's +0x112,
  +0x116), `doors: u8` (the day's Dimension Door casts, +0x10D). The hero's flying and
  water-walking levels for its steps become the best of `h3_heroes::movement`'s (the artifacts', 3)
  and the spell's: every reader of `Movement` (the passage, the price, the drop to 0) takes the
  combined level, so increment 5's paths serve the spells unchanged (survey §7.2); `ADV-QUIRK-17`
  stays the artifacts' alone (the drop prices with the spell's level when the level is the spell's).
- **A boat's last captain** already exists (increment 8's `Boat::captain`); its owner too.

### 9. The Prison's visit (ADV-PRI-002; the adventure's half)

`AdventureOutcome::prison { object, hero }` to the game, which decides (the eight, the store;
§15); then the adventure's infallible `free_prisoner(object, hero, owner, points)`: the object
removed, the hero placed on its entrance with the army of its setup, its points and the day's
maximum the given land points (`day_points` over its details, as a tavern hero's, TWN-TAV-005),
no boat, its tactics flag on, its formation the setup's, nothing revealed (the original reveals
nothing, survey §2.2 point 3; T139a checks `hero_move_record`'s callers once more), not visited.
The visitor's move ends at its neighbour's tile as every visit from a neighbour; its state is
untouched. The facing (right) is the clients'.

### 10. The cast's effects (ADV-SPL-003 to -009)

The adventure's half of `GameCommand::Cast` (§14): `check_cast(hero, spell, target, mastery)` then
the infallible `apply_cast`; its refusals are typed (`CastError`) and change nothing; a **charged
failure** is an accepted cast with `CastFailed` (the mana spent, §14).

1. **Summon Boat** (ADV-SPL-003): refused on water (in a boat; GENRLTXT 334) and without a free
   water neighbour (the first of the eight clockwise from up, on the map, water, no object: 335).
   Accepted: the chance draw (one draw of 1 to 100 from the actor's stream above the chance fails,
   charged: 337; the draw at 100 too); then the boat: the hero's last empty boat (its `captain`
   this hero, nobody aboard; any owner, any level), else with `ADV-QUIRK-53` on the least |dx| + |dy|
   among the empty boats of the player or of nobody, levels ignored, a tie to the later index; off,
   the least dx² + dy² **on the hero's level**, a tie to the lower index (decision 400). Found: moved
   to the neighbour (its facing kept, its owner unchanged: boarding writes it, increment 8). None:
   at Advanced or Expert a new boat there of the hero's owner (fails, charged, at `boats_most` in
   use: 336); at None or Basic fails, charged (336).
2. **Scuttle Boat** (ADV-SPL-004): the target a tile in the window (§3) whose boat is empty
   (a boat with a hero is the hero's: refused), §11.3 for hidden tiles. Accepted: the chance draw
   (fails, charged: 338); else the boat removed (its record out of use). A cancel is no command.
3. **Fly** (ADV-SPL-005): refused in a boat (193); with `ADV-QUIRK-54` on refused while `fly` is
   set or Angel Wings are worn (the original's silent "nothing"); off, refused only when the new
   level would not exceed the current one (the artifacts' 3 included). Accepted: `fly` = the
   mastery.
4. **Water Walk** (ADV-SPL-006): on land refused while `water_walk` is set or Boots of Levitation
   are worn (the flag as Fly's); **in a boat both tests skipped and the cast accepted** (A-49
   accepted, ADV-QUIRK-57): `water_walk` = the mastery; the level is ignored at sea and holds
   after landing the same day.
5. **Dimension Door** (ADV-SPL-007 to -009): refused with points not above 0 (126), with `doors` at
   the core's casts for the mastery (339), with a target outside the window or off the map, with an
   explored target that is blocked or an entrance (a hero's tile included); with `ADV-QUIRK-55`
   off also an explored target of the wrong kind (land from a boat, water from land). Accepted:
   a target of the wrong kind (explored with the flag on, or hidden: §11.2) fails, charged
   (GENRLTXT 71: the mana, the points, a cast); else the hero (and its boat at sea) relocated:
   no guard's battle, no visit, the reveal around it by its scouting radius (increment 11's
   jump, `hero_relocate`); then in every accepted case the points less `door_points[mastery]` (at
   least 0) and `doors` + 1.

### 11. The hidden tiles (decisions 399, 402)

1. **The flight** (ADV-MOV-008, ADV-MOV-009 amended; `AdventureState::passage`, the one row
   ADV-QUIRK-15 named). **On** (faithful): an unexplored tile is passable only flying, never a stop,
   and refused when it is truly rock (the original's order: the blocked test before the fog's), so
   the reach reads the hidden truth. **Off** (strict-fixed, the compromise): an unexplored tile is
   **assumed flyable**, priced as a pass over a blocked tile, never a stop, whatever lies there, so
   the reach and the path depend only on what the player knows. The move walks the path step by
   step as today, each step revealing by the scouting radius; **a step whose tile, once revealed,
   the hero may not pass** (rock, or land a pass cannot cross) is not made: the move ends on the
   last stop node it reached, charged the steps made (the passes after that node included: their
   reveals are kept, so a probe into the unknown is never free), the rest dropped
   (`AdventureOutcome::dropped`, as an event's stop). *Agent's call:* charging the passes made;
   the alternative (only up to the stop node) makes a free scout. A walker and a water-walker keep
   ADV-MOV-008 (no unexplored tile) in both settings; D-78 (row E15-14) only records the original.
2. **The door** (ADV-SPL-008). **On** (faithful): a hidden tile is a target when free (neither
   blocked nor an entrance), so the legal targets read the hidden truth. **Off:** **every hidden
   tile in the window is a target**; at the cast a hidden tile that is blocked, an entrance or of
   the wrong kind fails, charged, and **the target tile is revealed** for the caster's team
   (decision 399: that tile alone); a hidden free tile of the right kind is the arrival.
3. **Scuttle Boat** (ADV-SPL-004). **On:** a hidden empty boat in the window is a target (the
   original's cursor). **Off:** a target is an explored tile with an empty boat **or any hidden tile
   in the window**; at the cast a hidden tile holding an empty boat takes the chance draw, any other
   hidden tile fails, charged; **the hidden target is revealed in every outcome** (decision 402).

### 12. The day's end and boarding (ADV-SPL-002)

The adventure's start of the day (the game's day step, ADR-0017 point 3) clears every hero's
`DaySpells` (all heroes on the map; garrison and away heroes have none to clear). Boarding clears
`fly` and `water_walk` (increment 8's +0x112, +0x116 −1, survey §4.3); landing keeps
`water_walk`. Nothing else writes them but a cast.

### 13. Information (ADV-PRI-003, ADV-SPL-010; ADR-0016 point 6)

A viewer that sees a prison sees its type only, never its prisoner (the original's popup: the
name). A freed prisoner is a hero arriving on a tile (`PrisonerFreed` to its owner and every player
that sees the tile, as `HeroHired`), the prison's removal as any object's. A hero's `DaySpells` are
its owner's (the own hero view); another viewer sees the effects only: a door's relocation as a
jump (`HeroTeleported` to the viewers of each end, increment 11's projection), a summoned boat's
leave and arrival by tile, a scuttled boat's removal, a created boat's arrival. A failed cast and
its reason are the caster's alone. The reveals of §11 are the caster team's explored tiles
(ADR-0016 point 5). `docs/06-security.md` gains the lines at the `shared/` task.

## Part D — the game, the wire, the env (`h3-game`, `h3-protocol`, `h3-sim`)

### 14. The cast (`GameCommand::Cast`; GAM-SPL-001, new area GAM-SPL)

`GameCommand::Cast { hero, spell, target: Option<Pos> }` (the target for the door and Scuttle Boat
only; another spell with a target, or those without, refused). In the player's turn with no
decision open (GAM-DEC-001). Checked in order, each refusal typed, changing nothing, drawing
nothing: the actor's hero on the map (not a garrison's); `can_cast_on_map` (§6: the spell known,
the book); the spell one of the adventure part's five (§3); the ground: the special ground's
battlefield's `max_spell_level` (BAT-TER-004's data, read by the game as the raise of §5: Cursed
Ground's 1) below the spell's level, an anti-magic garrison's tile
(`AdventureState::anti_magic`, increment 12: the hero standing in or on it);
`map_spell_cost` at the mastery of §5 against the spell points; then the adventure's
`check_cast` (§10). *The original's order* puts the cost (the book's click) before the ground's
refusals (survey §4.1); both refuse free, so only the reported error differs; the engine follows
the original's. Then the adventure applies, the spell points are set to the points less the cost
(at least 0), and the events go out. The draws come from the actor's stream (ADR-0017 point 4), in
the order of §10. A computer player's casts are E31's.

### 15. The prisoner freed (GAM-SET-001 amended; ADV-PRI-001, -002: the game's half)

- **At the build:** each `PrisonerSetup` becomes a hero in the store (its details as a placed
  hero's: GAM-SET-001's army, the levels at load from its custom experience, its skills, artifacts,
  spells, then its spell points full), in the setup's object order after the placed heroes (the
  original's loop over every type, `placed_heroes_setup`: by type; T139b compares the two orders'
  draws and takes the original's), **in no player's list**, its pool status `Prisoner` (E09's,
  TWN-SET-005 kept). The new day regains a stored hero's spell points as every hero's (already full).
- **At the visit** (`AdventureOutcome::prison`): the prisoner's status no longer `Prisoner` (no
  path reaches it without campaigns): the prison removed, `PrisonEmptied` to the visitor's owner.
  The visitor's player at the town part's most heroes on the map (garrison heroes not counted, as
  TWN-TAV-002): `PrisonFull` to the owner, nothing changes. Else freed: the pool's status taken by
  the visitor's colour (its mask's bit added; no mask test, survey §2.2), the hero appended last to
  the player's list, `free_prisoner` with its day's land points, `PrisonerFreed { hero, player,
  tile }`. No decision, no choice: the visit's outcome (ADR-0017 point 10).

### 16. Events and the wire (`h3-protocol` v79, the server lane's, `cross-lane` at each task)

New `GameEvent`s: `SpellCast { hero, spell, mastery, cost }` (the caster's owner),
`CastFailed { hero, spell, reason }` (the owner), `BoatSummoned { boat, from, to }`, `BoatCreated
{ boat, tile }`, `BoatScuttled { boat, tile }` (by the tiles each viewer sees), `HeroTeleported`
reused (increment 11's), `DaySpellsChanged { hero, spells }` (the owner; also at the day's start
and at boarding), `PrisonerFreed`, `PrisonFull`, `PrisonEmptied`. The own hero view gains the day
spells; the wire's command `Cast`. Whether a `GameEvent` stays `Copy` (E09 T47b's 104 bytes) is the
event's size: none of these holds a `Vec`.

### 17. The env, replays, maps, properties, benchmarks

- **`legal_commands`** (GAM-SPL-001): per own hero on the map, after its moves, each castable spell
  without a target once, the door's and Scuttle Boat's every offered target in the window in row
  order (so up to 323 each; the random workloads' weight for them is the bots' call, §21).
- **The env's encoding v18** (the AI lane's, `cross-lane`, decision 372's precedent): three entries
  without a target (Summon Boat, Fly, Water Walk) and two windows of 19 × 17 (the door, Scuttle
  Boat) relative to the hero, the hero by its slot head; the masks from `legal_commands`. The
  observation's own hero part gains the day spells and the casts left.
- **Replays (format v24):** `cast <hero> <spell> [x y z]`.
- **Maps:** a generated map (the writer) with a prison, the Market of Time, special grounds, boats
  and water, a hero with the five spells and a book: the golden replays (each spell cast, the
  failures, the prison freed and full, both flag settings for §11), the plans and oracles of the
  hygiene property (the door's and the flight's legal moves under `strict-fixed` never depend on a
  hidden tile: the property's existing comparison over forks that differ only under the fog),
  `--with-original-data`'s census (89 prisons importing as prisoners with their bodies, one Market
  of Time).
- **Properties:** a rejected cast changes nothing; the day spells cleared at every new day; a
  door's points never negative; the flag's off side passes the hygiene property.
- **Benchmarks:** the passage's new row and the combined level sit on the reach loop: measured
  right after the first build (`homm3-bench-regression`); the random workload gains casts.

### 18. Versions, rule ids, quirks, stubs

- **Versions** (one per task, decision 384): adventure ruleset part v25 (§1 to §3), map import v29
  (§7), adventure state hash v28 (§8), game state hash v39 (§15, §14's command needs no state),
  protocol v79, replay format v24, encoding v18; the core ruleset hash at the `shared/` task (the
  flags). The town lane's and the server lane's plans may take a number first: rebase and bump.
- **Rule ids:** ADV-PRI-001 to -003, ADV-OBJ-008, ADV-SPL-001 to -010 (survey §16), GAM-SPL-001,
  HER-SPL-003 amended, HER-SPL-008, ADV-MOV-008 and -009 amended (§11.1), ADV-MOV-010 amended (the
  spell's levels), GAM-SET-001 amended (prisoners).
- **Quirks:** §4 (ADV-QUIRK-15 reclassified, 51 to 57). **Stubs:** ADV-STUB-19 (the four
  information spells' casts, E12: decision 406), ADV-STUB-20 (Town Portal's cast on the map, E09
  increment 7), ADV-STUB-21 (a bookless hero's scroll, until row E15-15: lifted or closed by its
  verdict).

## Part E — batch E15 and the cut

### 19. Batch E15 (survey §15, amended)

The survey's 16 rows, with the checkpoint's changes: **E15-9** records the original's choice (the
flag's on side), the off side is the engine's; **E15-16** records only (A-54 fixed whatever it
shows); **E15-8** and **E15-10** record the cursors (the flags' on sides) and add a cast at a hidden
blocked tile, the door's failure and what the original reveals (its off side's reveal is the
engine's own); E15-15 decides ADV-STUB-21. The spec's fields (`h3-experiment`, battle-owned,
`cross-lane`): prisons with a hero body, the Market of Time, special grounds under a hero, learned
spells and worn scrolls, hidden tiles by design (a start without a reveal around a hero: the
harness's map with its fog). The harness (`clients/capture/`, common): the cast's flow (the
button, the book's spell click, the target click), the hero's +0x10D, +0x112, +0x116 and +0x18
reads, the cursor's frame, the boats' records (game +0x4E3BC); a boat's last captain by sailing
first in the recipe.

### 20. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared:** one `shared/` task (`cross-lane`): the three drafts applied with batch E15's outcome
(ADR-0013 points 2, 17; ADR-0016 points 1, 3 to 6; ADR-0017 points 1, 4, 8 to 10); the quirk
flags in the core's profiles (the core ruleset hash); `docs/03-domain.md`'s terms (prisoner, a
cast on the map, the day spells, a spell's raise by a ground); `docs/06-security.md`'s lines (§13;
the cast's target validated, the draws server-side). **`cross-lane` elsewhere:** batch E15's spec
(`h3-experiment`), the harness (common), `h3-protocol` at each wire task, the env (`h3-sim`), the
battle's optional use of `spell_masteries_on` (its inbox). **The inboxes:** battle (the spec's
fields; `spell_masteries_on`), AI (the encoding v18's entries), server (the wire per task), map
client (the prison's and the Market of Time's types, the cast's events and targets, the cursors'
rule under each flag), town (the prisoners in the store and the pool, the eight shared with the
hire; prisoners now carry details: E09's pool code reads them).

**The building tasks**, in rounds of one or two (each task's cut confirmed at its start). **Batch
E15 runs before the build and the shared task** (decision 368's precedent):

| Round | Tasks | Content |
|---|---|---|
| 2 | **T138** | Batch E15's maps: the writer's prison body and class 50, the spec's fields (`cross-lane`), the specs and their maps (§19) |
| 3 | **T139** | T139a: the harness's cast flow and reads, the recipe, the run, scored (rows 1 to 16; the quirks' confirmations). T139b: the order of the prisoners' level draws (§15) read and recorded, if T139a leaves it open |
| 4 | **T140** | `shared/` (`cross-lane`): the three drafts with T139's outcome, the flags in the profiles, the glossary, `docs/06-security.md` |
| 5 | **T141** | T141a: the Prison and the Market of Time (§1, §2, §7, §9, §15; part v25's kinds, map import v29, game state hash v39) with their events and wire. T141b: `h3-heroes`' `spell_masteries_on`, `map_spell_cost`, `can_cast_on_map` (§5, §6) |
| 6 | **T142** | The cast's frame (§3's data, §8's `DaySpells`, §12, §14: adventure state hash v28) with Fly and Water Walk (§10.3, .4) and the flight's flag (§11.1, measured against `master` at once) |
| 7 | **T143** | T143a: Summon Boat and Scuttle Boat (§10.1, .2, §11.3). T143b: Dimension Door (§10.5, §11.2). Their events and wire (protocol v79 at the first) |
| 8 | **T144** | T144a: `legal_commands`, the env's entries (v18, `cross-lane`), the replay words (v24). T144b: the maps, the replays, the plans and oracles, the benchmarks, `--with-original-data` (§17) |
| 9 | **T145** | The audit against the exit criteria; increment 15 and E08 closed (decision 396); E10's first round queued |

*Agent's call:* the Prison first (independent of the spells, the store's change the town lane
reads), then `h3-heroes`' derivations the cast needs, the frame with the two simplest spells and
the riskiest change (the passage) measured early, the boats and the door last; the env and replays
once every cast exists, so the encoding and the format move once.

**Increment 15's exit criteria:**
1. The 89 shipped prisons import as prisoners with their bodies (levels at load, army, skills,
   artifacts, spell points), in no list, out of the pool; the Market of Time as `empty`; the census
   holds (ADV-PRI-001, ADV-OBJ-008).
2. A prison's visit frees its hero for the visitor's player below the eight on the map (garrisons
   not counted): on the entrance, a full day's land points, last in the list, the prison gone; at
   eight nothing changes; a viewer never sees the prisoner (ADV-PRI-002, -003).
3. A cast on the map is refused, changing nothing, without the spell or a book, on Cursed Ground
   above level 1, in or on an anti-magic garrison, above the spell points, or by the spell's own
   test; the mastery is the hero's on its tile's ground; the cost the table's at least 1 without
   the armies' (ADV-SPL-001, HER-SPL-003, -008, GAM-SPL-001).
4. Every effect lasts until the new day; boarding clears Fly and Water Walk (ADV-SPL-002).
5. Summon Boat and Scuttle Boat draw their chance from the actor's stream, a failure charged;
   the boat chosen, moved, created or sunk as §10.1, §10.2 under each flag (ADV-SPL-003, -004).
6. Fly and Water Walk set the levels 0 to 3 the paths use; their refusals and A-49 as §10.3, .4
   under each flag (ADV-SPL-005, -006, ADV-MOV-009, -010).
7. Dimension Door: points above 0, 1 to 4 casts a day, the 9 × 8 window, no guard, no visit, the
   reveal, 300 or 200 points; its failures as §10.5 and §11.2 under each flag (ADV-SPL-007 to -009).
8. Under `strict-fixed` the legal moves never depend on a hidden tile: the flight's compromise,
   the door's and Scuttle Boat's hidden targets with their reveals (§11; the hygiene property);
   under `faithful` the original's reach and targets.
9. A viewer sees what §13 says; the events and the wire as §16.
10. Every new rule has its id, source and tests; the gates pass (coverage, determinism with the new
    and rehashed replays, the benchmarks at their baselines, the new ones baselined); batch E15 run
    and scored (T139), the consensus rows `verified`.

### 21. Risks and open points

- **The passage's change sits on the hottest loop** (the reach): the combined level and the
  flag's branch are measured at T142's first build; a regression above the gate is settled there.
- **The compromise's stop** (§11.1): a path offered and cut short is new behaviour for the env and
  the bots (an accepted move that ends early, its rest dropped); the env's step reports it as an
  event's stop does today.
- **The door's and Scuttle Boat's 323 targets** make a random game cast often; the step cap and the
  weights are the AI lane's (Revisit's and the market's precedent).
- **The prisoners' level draws' order** (§15): the setup stream's order by type (the original's
  loop) or by object; a wrong order changes every prisoner's levels on the shipped maps but not a
  rule; T139 reads it.
- **D-78, D-79** rest on rows E15-14, E15-15; a verdict against the reading moves ADV-MOV-010 or
  lifts ADV-STUB-21 in the building task.
- **Not read:** the computer's casts (E31), network play's messages, the four information spells'
  readers (E12), Town Portal (E09 increment 7), the pictures, texts and sounds.
