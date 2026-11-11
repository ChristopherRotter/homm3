# E08 design note

Increment 11's note: the teleporters (E08 decision 260). Increment 10's note (dwellings and hiring)
is in `E08-log.md` ("Increment 10's design note at its closure", moved by T97c), increment 9's
beside it ("Increment 9's design note at its closure", moved by T87c); the code's comments cite
these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 11's
amendments, drafted in the lane's `adr-drafts/` (`0016-increment-11.md` by T98b1,
`0017-increment-11.md` by T98b2; applied to ADR-0016 and ADR-0017 by T102, then deleted); written from T98a's survey and the answers at ADV-Q44 and
ADV-Q45 (the epic's decisions 260, 264–269). A building task adds an *As built* line to the section
it builds; the note moves to `E08-log.md` when the increment closes.
Facts and sources: survey `docs/rules/surveys/E08-teleporters.md` (§ numbers are the survey's
unless they name this note); the proposed rule ids are the survey's §13.

Written in two parts by side (decision 270): **parts A and B** by T98b1, the content and the map
(`h3-adventure`, `h3-pack-adventure`, `h3-map`, and the one effect kind note §2 asks of `h3-content`
with its `h3-heroes` derivation); **part C** by T98b2, the game, the env, batch E11 and the cut
(§12 on). ADR-0015 needs no amendment: the import still draws nothing, and the channels are the
static map's, built with the game (ADR-0016 point 1). *Agent's call (change freely)* marks a choice
the user could change at the checkpoint; **the checkpoint** (ADV-Q46, T98b2; decisions 272–274)
accepted the note, both drafts, every agent's call and the cut (§17) as written.

**The checkpoint's answers** (ADV-Q45, decisions 264–269), which this note builds on: every exit
the original's (D-53 drawn, never named by a command; D-54 the weakest stack by fight value ×
count, the first slot on a tie; D-55 half kept rounded down, a lone creature kept; D-56 the gates
paired by truncated distance; D-57 one draw over the other Whirlpools' tiles; D-58 no path through
a teleporter); A-37 fixed by the battle; A-38 no bound; A-39 refused at the import; A-40 and A-41
not reproduced; `Revisit` in this increment (part C); batch E11 before the build (part C).

##### Part A — the content (the adventure part, its packs, one effect kind)

###### 1. The kinds (`h3-adventure`'s `ObjectKind`; ADV-TEL-001)

Three kinds, every one **entered** (none of the five classes is in `object_classes_not_entered`
0x63AA58), none ownable or weekly; only the Whirlpool **visitable from above** (the only one in
`object_classes_from_above` 0x63AB00; survey §2):

- **`monolith`** (classes 43, 44, 45): `channel`, a number (the original's subclass: the picture's
  colour), and `way`: `entrance` (43, one-way: its exits are the channel's `exit` monoliths), `exit`
  (44: leads nowhere) or `two_way` (45: its exits are the channel's other `two_way` monoliths). A
  one-way channel and a two-way channel of the same number are distinct (the original's two sets of
  eight lists, §3.1). *Agent's call:* one kind with the way, not three kinds, the channel's list
  keyed by the way's group and the number (note §5); no bound on the number in the kind (the
  original's eight lists are the import's table, A-39, note §4).
- **`subterranean_gate`** (103): no parameters; its exit is its pair (note §5). Named apart from the
  border gate (`KeyGate`, `Effect::Gate`), which keeps its name.
- **`whirlpool`** (111): `divisor` (the original's 2; at least 2): the weakest stack keeps its count
  divided by it, rounded down, a lone stack at least one (note §8). *Agent's call:* data, not a
  constant (ADR-0006), as increment 10's dwelling parameters.

`ObjectKind::effect` gains the three kinds' visit (*agent's call:* one effect, `Effect::Teleport`,
the visit reading the kind; the building task names it); the visit is note §7. Hash tags follow the
part's last (the building task's); **ruleset part v18**.

*As built* (T103a, `adventure/e08-t103a`): `ObjectKind::Monolith { channel: u8, way: MonolithWay }`
(`Entrance`, `Exit`, `TwoWay`), `SubterraneanGate`, `Whirlpool { divisor: u32 }` (hash tags 34 to
36; the way 0 to 2), their `Def`s; a teleporter not entered and a divisor below 2 refused;
`effect` none for the three until T105a, so a step onto an entrance is refused (`Visit`) and the
reach ends before it (no `Effect::Teleport` yet); adventure ruleset part v18.

###### 2. The Whirlpool's protection (`h3-content`'s effect kinds; HER-ADV-009 amended)

The original spares a hero wearing the Sea Captain's Hat (123), alone or in a combination
(`hero_wears_artifact` 0x4D91F0 looks into the Admiral's Hat, 136; survey §4.4). *Agent's call:* a
core **artifact effect kind `whirlpool_protection`**, without parameters, on the Sea Captain's Hat,
read through a new `h3_heroes::whirlpool_protection(details, core) -> bool` (the effect from any
source, `Count::OncePerType`: a worn type or a part of a worn combination, so the Admiral's Hat
protects through its part), as increment 8's `free_boarding` (HER-ADV-009; E08 T70, T71) and as
VCMI's `WHIRLPOOL_PROTECTION` bonus. A shared change: `h3-content` and `h3-pack` (**pack format
v51, core ruleset hash v52**) and the harvester's gap-fill (battle-owned, `cross-lane`: the Hat's
effect beside its spells and sea points), in the round's `shared/` task with the ADRs (part C §17).
Not chosen: the protecting artifacts as the Whirlpool kind's parameter (no shared change, but an
adventure kind naming artifacts, and a combination's parts read outside `h3-heroes`).

The weakest stack's measure (note §8) reads the core creature's fight value (`fight_value`,
harvested; the original's creature record +0x3C): no new content.

*As built* (T102, `shared/e08-t102`): `HeroEffect::WhirlpoolProtection` (code 60), pack format
v51, core ruleset hash v52; the gap-fill's Sea Captain's Hat with three sources
(`exe-whirlpool-protection`, `vcmi-whirlpool-protection`: VCMI 1.7.5's `seaCaptainsHat`
`WHIRLPOOL_PROTECTION`, read for it; `wiki-whirlpool-protection`); `h3_heroes::whirlpool_protection`
is T103a's.

###### 3. The packs

- **`base`**: the monoliths of channels 0 to 2 in each way (nine types: RoE's and AB's
  `Objects.txt` list subclasses 0 to 2), the Subterranean Gate and the Whirlpool (both RoE).
  **`sod`**: the monoliths of channels 3 to 7 (fifteen types: only SoD's `Objects.txt` lists them;
  subclasses 3 to 7 stand on SoD maps only, §1). An RoE or AB ruleset meets an SoD map's channel 3
  as `MissingContent`, as any SoD object. `ab` adds none.
- Keys and names (*agent's call*; the building task fixes them): `base:monolith_entrance_0` to
  `sod:two_way_monolith_7`; the names the original's class names (`ObjNames.txt`: Monolith One Way
  Entrance, Monolith One Way Exit, Two-Way Monolith, Subterranean Gate, Whirlpool), a monolith's
  with its channel's number. *As built* (T103a): the keys `monolith_entrance_N`,
  `monolith_exit_N`, `monolith_two_way_N`, `subterranean_gate`, `whirlpool`; `ObjNames.txt`'s row
  45 is "Monolith Two Way" (not "Two-Way Monolith"), so the names "Monolith One Way Entrance N",
  "Monolith One Way Exit N", "Monolith Two Way N" with the channel N, "Subterranean Gate",
  "Whirlpool".
- **The harvest's check** (with the original data, `h3-map`'s tests): each monolith type's channel
  and way equal its class and subclass in the import's tables (note §4); the subclasses each
  expansion's `Objects.txt` lists (RoE and AB 0 to 2, SoD 0 to 7) are defined by the pack of that
  expansion; the names are `ObjNames.txt`'s. The types are hand-written source with their sources,
  as the mines'.
- **The synthetic packs** gain a monolith of each way in two channels, a gate and a Whirlpool, for
  the crates' own tests.
- *As built* (T103a): base's 11 types after the town, sod's 15 after Favorable Winds (base's new
  types move ab's and sod's type ids: the duel and sea replays' checkpoints moved), `synthetic`'s
  eight (divisor 3). The check against the original data is `h3-pack-adventure`'s
  `original_data` (its check 5): the 26 types against H3bitmap.lod's (SoD's) and H3ab_bmp.lod's
  (AB's) `Objects.txt` (base when AB lists the subclass), `ObjNames.txt` and the executable's
  `object_classes_not_entered` 0x63AA58 and `object_classes_from_above` 0x63AB00; the import's
  tables join it in T103b. *As built* (T103b): `h3-map`'s `original_data` checks every
  teleporter of the 160 shipped maps to import as its class and subclass's type (every channel of
  the three classes stands on them, so each table entry is met), its tests every entry of the three
  tables against the packs' kinds without the original data.

##### Part B — the map (`h3-map`, `h3-adventure`)

###### 4. The import (`h3-map`; map import v21: E09 T11 took v20)

- **Classes** (ADV-OBJ-002 amended): 43, 44 and 45 by subclass (three tables of eight: the
  channel's number is the subclass), 103 `base:subterranean_gate` and 111 `base:whirlpool`
  whatever the subclass (the original's setup ignores 111's; every shipped 103 and 111 is
  subclass 0, §1). No body (`h3m.md`'s plain objects; VCMI's `readGeneric`); the maps' templates
  rule, as for every object (42 SoD monoliths block one tile where `Objects.txt` blocks two, §2).
- **Refused:** a monolith subclass past 7 (**A-39**, decision 267: the table's bound,
  `ImportError::Subclass`, as A-36's; the original writes past its eight lists). An input bound,
  no flag; none on the shipped maps, and the editor offers 0 to 7.
- Nothing else: the census found no entrance shared with another object, none off the map, none with
  a body (§1). The surfaces are the build's check (note §5), so a setup from any source is held to
  them.
- *As built* (T103b, `adventure/e08-t103b`): `codes.rs`' `MONOLITH_ENTRANCES`, `MONOLITH_EXITS`,
  `TWO_WAY_MONOLITHS` (eight keys each, `base:` 0 to 2, `sod:` 3 to 7) in `object_type`'s classes
  43 to 45, 103 and 111 one key each; A-39 is the table's bound (`Subclass`, `ADV-QUIRK-41`). Map
  import v21.

###### 5. The build: the surfaces and the channels (`h3-adventure`'s `StaticMap`; ADV-TEL-001, -002)

Checked and built when the game is built, after the static map's tiles and objects (the random
dwellings resolved, every type final), from the typed objects alone, **without a draw**: no setup
placeholder's order moves (ADR-0017 point 4 unchanged). Static, behind the static map's `Arc`,
shared by forks (decision 6); empty for a map without teleporters.

- **The surfaces** (a new candidate **A-42**, *agent's call*): a monolith's or a gate's visitable
  tile on water, or a Whirlpool's on land, is refused (`SetupError`, named by the building task;
  the import passes it on). A jump between surfaces would put a hero in a boat on land or a walker
  on water, which no rule handles (state-corrupting); the original's case is unread (six monolith
  templates allow water, none stands on it; every gate on land, every Whirlpool tile on water,
  §1), so none of the shipped maps is refused. Not chosen: an exit on the other surface no
  candidate (every map loads, but a test per candidate and a gate's pair to drop).
- **Entries** are visitable tiles, as the original fills a word per tile (§3.1): one per object
  on every shipped map (a monolith's and a gate's one tile, a Whirlpool's six). Every list is **in
  the walk's order**, level, row, column: the static map's tile index (`StaticMap::index`), not
  `Pos`'s derived order (column first).
- **The lists:** for each `exit` channel its exits' tiles; for each `two_way` channel its members'
  tiles; one list of every Whirlpool's tiles, each with its Whirlpool (its object index: the
  original's word shared by the six, §3.1). An `entrance` is in no list (the original's case 43
  has none).
- **The gates' pairs** (ADV-TEL-002; D-56 as the original, `gates_pair` 0x4C0B60): over the gates'
  entries in walk order, each one still unpaired takes the unpaired entry after it on another level
  with the least truncated distance `⌊√(dx² + dy²)⌋` (`dx`, `dy` the entries' columns and rows; an
  integer square root, exact, so no float, ADR-0003), the first in walk order on a tie, and both are
  paired; one left over has none. The original's maps have two levels at most, so the levels' gates
  pair while both have one (every shipped gate paired, 301 pairs; 18 gates on 8 maps not with their
  nearest, the greedy order, §1). Cost: quadratic in the gates, at most about 8 300 on the largest
  original map (a gate covers five tiles), about 17 M comparisons in a build; part C §14 measures
  it.
- *Agent's call:* the representation is the building task's (for example a table sorted by the
  way's group and number, the Whirlpools' list, the pairs by gate entry), read by the visit alone
  (note §7), never by the reach (note §9).
- *As built* (T104, `adventure/e08-t104`; decisions 285, 286): `state/channels.rs`'
  `StaticMap::channels`, a `Channels` of the one-way exits and the two-way members by channel
  number (`BTreeMap<u8, Vec<Pos>>`), the Whirlpools' tiles with their object (`WhirlpoolTile`) and
  the gates' entries with their pairs (`GateEntry`, `gate_pair` by a binary search in walk
  order); built after `list_economy`, empty without a teleporter, out of the hash. A tile is listed
  for the last object in map order visited from it (the original's tile class), so a teleporter
  under a later object is in no list. A-42 is `SetupError::TeleporterSurface`, the first
  teleporter in object order with a visitable tile off its surface; *agent's call:* rock is off
  both surfaces (nothing walks or sails it). The pairing compares squares against the best
  distance's square (`⌊√s⌋ < d` exactly when `s < d²`) and scans the levels after the entry's own
  (the walk's order puts them after it). `build-gates-xl-144` (8 116 gates): 981 M Ir for two
  builds, the channels 91.6 % of it, 63 ms a build: the lever (§18) is queued. Checked against the
  160 shipped maps with T106's census brought forward (`h3-map`'s `original_data`): §1's counts
  exactly, no map refused.

###### 6. The state (`h3-adventure`; adventure state hash v21, unmoved)

No new block. A jump writes the hero's tile (nothing of an aboard boat: its record keeps the tile
it was boarded on while the hero's tile is the boat's, ADR-0016 point 1 since increment 8), a
Whirlpool the hero's army, a reveal its owner's fog. The channels follow from the static map's
objects, which its hash covers (their types, anchors and masks), as the per-tile object flags do.
The jump held as a won battle's rest is the game's state (part C §13).

###### 7. The jump: the visits (ADV-TEL-003 to -007)

The step onto an entrance is priced, paid and revealed as any step (ADV-MOV; its drop to 0 below
the cheapest next step is tested there and never at the exit, §6), and its visit is the jump: the
walk ends at the entrance and nothing goes on past the exit (§6). The same visit runs for a
`Revisit` (part C §13).

**The candidates** (`teleport_exit_draw` 0x4CDB80, §4.1): over the channel's list in walk order, a
tile is a candidate when **a hero of another team stands on it**, whatever its object (a team is
its player until E10's teams add the allies), or when **no hero and no empty boat stand on
it** and the visit does not exclude it: a two-way monolith excludes its own tile, a Whirlpool its
own six (the shared word). The mover stands on the entrance, which is so never a candidate.
*Agent's call:* a tile with an empty boat (a hero landed from a Whirlpool's tile) is no candidate:
the reading tests the tile's top class, which would be the boat's (not read), and an arrival there
would make two vessels on one tile.

**The draw** (ADV-TEL-003; D-53, D-57): none without a candidate; **none for one** (the original's
`random_between` returns equal bounds without a draw, §4.1); else one uniform draw of the
candidate's place among `n`. A Whirlpool's exit object and tile come from that one draw. The
stream is the visitor's (ADR-0016 point 4: "a rule that draws at a visit draws from the visitor's
stream"); the jump is the first visit that draws, so `AdventureState::apply` gains a stream
(`draws: &mut Rng`) the game passes (part C §13, ADR-0017 point 4's draft). It draws after the
whole command is checked, so a rejected command leaves the stream untouched as the rest of the
state.

By kind:

1. **A one-way entrance** (`monolith`, `entrance`; ADV-TEL-004): the candidates among its channel's
   exits. None: nothing at all, no event, the hero on the entrance (the original's silence; a
   channel whose exits all hold heroes of the team). The drawn tile free: **the arrival**. An enemy
   hero on it: **the battle at an exit**.
2. **A two-way monolith** (`two_way`; ADV-TEL-004): the same over its channel's other members.
3. **A one-way exit** (`exit`; ADV-TEL-004): it leads nowhere, told to the visitor's owner (the
   original's ADVEVENT 70, a human's only; every owner's here, decision 16); nothing else.
4. **A Subterranean Gate** (ADV-TEL-005; `gate_exit` 0x4CDE40, §4.3): its pair, **no draw**.
   None: told to the visitor's owner that it leads nowhere (ADVEVENT 153), the hero stays. A hero
   of the mover's team on the pair: the tiles within 1 of the pair revealed to the mover's owner,
   then **the meeting** (GAM-MET-001's outcome, the mover the visitor), the mover staying on the
   entrance (part C §12). An enemy hero on it: the battle at an exit. Free: the arrival.
5. **A Whirlpool** (ADV-TEL-006; §4.4): first **the loss** (note §8), then the draw over the other
   Whirlpools' tiles. None: the hero stays, the loss taken (the wiki: a lone Whirlpool only takes
   troops). Free: the arrival, the boat with the hero. An enemy hero on it (**A-37, fixed by the
   battle**, decision 265: the original moves the mover onto the other hero's tile): the battle at
   an exit.

**The battle at an exit** (the original's own for 43, 45 and 103, §4.2, §4.3; decision 265 for the
Whirlpool): the tiles within 1 of the exit revealed to the mover's owner (the original's
`reveal_disk` with radius 1), then the outcome's battle with the exit's hero as the defender
(GAM-BAT-004's, the mover attacking from the entrance; where it is fought, at the defender's tile
as every hero's battle under GAM-BAT-001, at sea with a boat on either side as increment 8's:
part C §12), and **the jump to that exit held as the visit's rest** (a new outcome field naming the
hero and the exit, *agent's call*; the game holds it and gives it after the winner's levels,
GAM-DEC-007: part C §13). The mover lost, retreated, surrendered or died in a draw: it is gone and
the rest dropped. Won: the arrival at the exit, its hero gone (the gate's test that the exit is a
gate again, 0x4AA4A6, holds after every win).

**The arrival** (ADV-TEL-007; `hero_relocate` 0x41D930, §5): the hero's tile becomes the exit; its
points and the day's maximum are kept; **no guard test and no visit** at the exit (a two-way exit
is not re-entered), no event fired; the tiles within the hero's scouting radius of the exit are
revealed to its owner (ADV-FOG-002's disk, as a step's); the move ends. The jump's event: note §10.

**The entrance's guards** (decision 279, batch E11's E11-19; survey §15). ADV-MON-006 as built
until T103d visited the guard of the hero's tile once the tile's object's visit returned
(`guard_after`), so after a jump it would have met the exit's guard. **The original's order is the guard first**: the step
onto a guarded entrance meets its guard (its answer, its battle and levels) and the jump follows
for a hero still on the entrance (a win, a join, a flight); the arrival tests no guard (E11-3,
E11-19's N), nor does a revisit (E11-19's Space; the user: a hero on a two-way pair cycles without a
fight). Decision 279 takes this order for **every entered object** (ADV-MON-006, GAM-DEC-007: the
tile's visit held as a rest after its guard's, as an event's), built by its own task before the
jump (round 7's cut, §17); the jump then needs nothing of its own for the entrance's guard. A
one-way exit beside a guard: no fight on arrival by §5's reading, unobserved; batch E11c (§16).
*(Decision 272's agent's call, a jump dropping the entrance's held guard, is replaced.)*

*As built* (T105a1, `adventure/e08-t105a1`; decision 287, the cut by land and water):
`state/jumps.rs`, the kinds' `Effect::Monolith { channel, way }` and `Effect::SubterraneanGate`
(the Whirlpool's effect still none: a boat's step onto it refused until T105a2). `draw_exit` counts the
candidates over the channel's list, then takes the drawn one (two passes, no allocation); `jump`
arrives when no hero stands on the exit, else explores radius 1 and leaves the mover on the
entrance (`ADV-STUB-16`: the battle and the meeting are T105b's; lifted by T105b1, §12); `arrive` writes the tile, keeps
the points, explores the scouting radius and emits `HeroTeleported`. The entrance's guard needed
nothing: T103d's held visit is the jump, `visit_held` passing the stream.
*As built* (T105a2, `adventure/e08-t105a2`): `Effect::Whirlpool(divisor)`, `visit_whirlpool` (the
loss unless protected, then `draw_exit` over `Channels::whirlpools`, each tile excluded when its
Whirlpool is the entrance's object); `draw_exit` takes the list's tiles with what the visit
excludes, so one pass serves the three kinds; the arrival with the boat needed nothing (the hero's
tile is the boat's). An enemy hero on a drawn Whirlpool tile: `ADV-STUB-16` as the monoliths' (lifted by
T105b1).

###### 8. The Whirlpool's loss (ADV-TEL-006; `whirlpool_losses` 0x4ACAA0, §4.4)

- **Whose:** every hero's (**A-40 not reproduced**, decision 267: the original spares a computer
  player's, `player_id_is_human` 0x4CE940), unless `h3_heroes::whirlpool_protection` (note §2).
- **The weakest stack** (D-54; **A-38: no bound**, decision 266): the occupied slot with the least
  fight value × count, a 64-bit product, the first slot on a tie (strictly less). The original's
  start at 99 999 999 and its `i32` product are not reproduced: a lone stack of 1 776 Azure Dragons
  is halved (the wiki's "bug"), and several stacks never corrupt the seventh slot.
- **The loss** (D-55): with more than one stack the weakest's count becomes `count / divisor`,
  rounded down, 0 emptying the slot; a lone stack keeps `max(1, count / divisor)` (with the
  original's 2: a lone creature kept, a lone stack of 3 keeps 1).
- **The event** (*agent's call*, new): the hero, the slot, its creature and the count lost (0 for
  a lone creature kept: the original's ADVEVENT 168 shows then too), told to the hero's owner
  alone, as its army is.
- The army is never left empty (a lone stack keeps one); its slowest speed may change, the day's
  points do not (neither the loss nor `hero_relocate` touches them).
- *As built* (T105a2): `jumps.rs`' `whirlpool_loss` (the least `(fight value × count, slot)`, the
  stack's count set, an emptied slot dropped), `AdventureEventKind::WhirlpoolLoss { hero, slot,
  creature, count }`; a hero without details is unprotected; `ADV-QUIRK-40` (A-38),
  `ADV-QUIRK-42` (A-40).

###### 9. The reach and the paths (ADV-OBJ-003 amended; D-58 as the original)

- Typed and entered, an entrance is a visit's tile (`visit_at` gives the kind's effect), so the
  walking search and `AdventureState::passage` meet it as any entered object: a **destination**
  with the points left after the step (the jump costs none, §5), never a tile on the way and never
  its exits (the original's mode 0, §9.1). `MoveHero`'s last step may be onto it; a step past it is
  `PastVisit`. On land a flight passes over an entrance as over every entered object's tile
  (ADV-MOV-009); a Whirlpool's tiles are a water object's, entered by a hero in a boat only (D-46's
  row, ADV-SEA-005), closed to a flight and a water walk. From above (ADV-OBJ-004's rows): the
  monoliths and gates closed, the Whirlpool open.
- A hero standing on an entrance (a failed jump, a jump's exit that is itself an entrance, a
  meeting at a gate) starts its reach there as on any entered object; a step off and back on visits
  again (§6).
- **The hot path:** no new read in the reach's or the passage's loops. The entrance's row comes
  from the static tile's typed visit, which the types set (`StaticMap`'s `typed_visits`); the
  channels are read by the visit alone. A map with teleporters gains destinations (each entrance a
  stop node, the search going no further there). Part C §14 plans the measure (`reach-xl-144` and
  the `random-play-*` workloads against `master`; TRAPS: a fixture's or test's blocker of one of the
  five classes becomes typed, the generators use 118, the building task checks the committed maps)
  and a teleporters' workload.
- The computer's planning through a channel (every exit relaxed, its price, the guards weighed) is
  E10's (§9.1).

###### 10. Information (ADV-FOG-004 amended; ADR-0016 point 6)

- **Each kind is public** where a viewer sees the object: its type, so a monolith's channel (the
  picture's colour). No channel's exits and no gate's pair are listed: the map's pictures show what
  the original shows.
- **The jump** (ADV-FOG-004 amended; *agent's call:* a new event `HeroTeleported` with
  `HeroStepped`'s fields and projection; not chosen: `HeroStepped` itself, whose two tiles a client
  takes for neighbours): seen by the players that see the hero on the entrance or on the exit
  (ADV-FOG-004, -005, as a step's); a viewer that sees one end only is told that end, the other
  `None`. **A-41 not reproduced** (decision 267): no viewer learns a hidden exit (the original's
  record for the Replay Opponent Turn is a display, §10).
- **Reveals:** the arrival's (the scouting radius) and an occupied exit's (radius 1) change the
  owner's explored tiles as a step's reveal does, with no event of their own (*agent's call*,
  against §11's sketch: `TilesRevealed` stays an object's counted reveal, the Cartographer's and an
  observatory's, which the original reports in a message).
- **To the visitor's owner alone:** the Whirlpool's loss (note §8); "leads nowhere" for a one-way
  exit and a gate without a pair (*agent's call:* one new event naming the object by type and
  anchor, the client choosing the original's text by the kind); the meeting (part C). A monolith
  without a candidate tells nothing.
- **To the viewers of the tiles:** a battle at an exit as any battle (ADR-0016 point 6).
- `docs/06-security.md` gains the lines with the build (the round's `shared/` task, part C §17).
- *As built* (T105a1): `AdventureEventKind::HeroTeleported` (`HeroStepped`'s fields, its
  projection) and `LeadsNowhere { hero, object_type, anchor }` (the owner's alone); the radius-1
  reveal and the arrival's tell nothing beyond the explored tiles; `ADV-QUIRK-43` (A-41).

###### 11. Ids, quirks and seams (parts A and B)

- **Rule ids** (survey §13; written by the building tasks): ADV-TEL-001 (the kinds, the import's
  classes, A-39's and A-42's refusals), -002 (the channels and the gates' pairs), -003 (the
  candidates and the draw), -004 (the monoliths), -005 (the gate), -006 (the Whirlpool with its
  loss), -007 (the arrival); ADV-OBJ-002, ADV-OBJ-003 and ADV-FOG-004 amended; HER-ADV-009 amended
  (the protection, note §2). Part C adds ADV-TEL-008 (`Revisit`) and amends GAM-BAT-004, GAM-MET-001
  and GAM-DEC-007.
- **Quirks** (ledger ids proposed here, the entries written by the building tasks as increment
  10's): A-37 `ADV-QUIRK-39` (state-corrupting, always fixed: the battle; no flag), A-38 `-40`
  (fixed: no bound; no flag), A-39 `-41` (exploit-fix: refused at the import), A-40 `-42`
  (harmless, not reproduced; revisited with E10), A-41 `-43` (a display's leak, not reproduced),
  A-42 `-44` (refused when the game is built; the agent's candidate, for the checkpoint). D-53 to
  D-58 are the original's rules (decision 264), without an entry.
- **Stubs:** none.
- **Shared needs:** the effect kind `whirlpool_protection` (note §2); nothing in `h3-contracts`.
- **Other lanes:** the battle lane's harvester (note §2, `cross-lane`); the AI lane's action
  encoding (part C, `Revisit`); the map client: the five kinds' types and the new events (its inbox
  when built); the town lane: none (Town Portal is a spell, the Castle Gate a town building, neither
  here).
- **Versions** (parts A and B): map import v21 (E09 T11 took v20), adventure ruleset part v18; the adventure state hash
  unmoved; shared: pack format v51, core ruleset hash v52 (note §2). A town-lane bump of the same
  rows in between: the second to merge bumps again (part C §15 orders them).

##### Part C — the game, the env, batch E11 and the cut

Written by T98b2 (decision 270; whole: decision 271): §12 the battle and the meeting at an exit,
§13 the game (the visitor's stream, the jump held as a rest, `Revisit`), §14 the env, replays,
fixtures, properties and benchmarks, §15 versions, rule ids, quirks, §16 batch E11, §17 shared and
`cross-lane` work, the building tasks and the exit criteria, §18 risks. ADR-0017's amendment
(points 1, 4, 6, 8, 10, 11) was drafted in `adr-drafts/0017-increment-11.md`; `Revisit`'s map-side
lines were added to `0016-increment-11.md` (point 4); both applied by T102.

###### 12. The battle and the meeting at an exit (GAM-BAT-004, GAM-MET-001 amended)

- **The defender** (note §7's battle at an exit): the hero on the exit, of another team by the
  candidates' test (a gate's pair by its own, note §7 item 4), as `Defender::Hero`, the mover the
  attacker; the outcome's `battle`, opened by the game at once and without a question as every
  hero battle (ADR-0017 point 11). The defender's owner plays outside its turn, as any defender.
- **Where it is fought:** at the **exit**, the defender's tile (`BattleStart::tile`, GAM-BAT-001):
  its battle terrain in the original's order (the sea's with a boat on either side, the
  underground's, the coast's, the tile's own), its special ground, the background from the tiles
  around it (GAM-BAT-011), the obstacles from its tile seed under `ADV-QUIRK-5`; the attacker's
  ground (GAM-BAT-013) is the special ground under the mover's own tile, the entrance. A
  Whirlpool's (A-37, decision 265): both heroes in boats, so on the two boats (increment 8's,
  `sea_battle_ground`); a monolith's or a gate's on land (A-42 keeps both ends there). *Agent's
  call:* nothing new in the setup; the original's `visit_hero` 0x4A2800 from the entrance (§4.2)
  is read as GAM-BAT-001's battle at the defender's tile (E08 T17a's reading); batch E11's E11-7
  records the battlefield (§16).
- **The radius-1 reveal's place:** in the visit, the adventure's own, before the outcome returns:
  the tiles within 1 of the exit (ADV-FOG-002's disk) explored by the mover's owner, no event (note
  §10), so its observation shows the defender before the battle opens, as the original reveals
  before `visit_hero`. The same before a gate's meeting.
- **What a viewer sees:** the step onto the entrance as any step (`HeroStepped`); the battle's
  start, events and end as any battle's, to its two owners (GAM-BAT-001): `BattleStarted` names
  the exit, where the defender's owner sees its own hero, and the entrance is named to no one but
  through the step. The jump's event comes with the held jump (§13), not before.
- **Its end** (GAM-BAT-002, -003, -004 as any hero battle's): won, the defender leaves the map (in
  a boat with its boat, ADV-SEA-006), the winner gains the battle's experience with the 500 and
  the loser's artifacts, then its levels, then **the held jump** (§13); lost, retreated or
  surrendered, the mover leaves the map (with its boat) and the held jump finds no hero; a draw,
  both leave.
- **The meeting at a gate** (GAM-MET-001 amended; note §7 item 4): a hero of the mover's team on
  the pair: the radius-1 reveal at the pair, then the outcome's **`meeting`** (the mover the
  visitor, the pair's hero the visited), opened by the game as increment 6's meeting (Scholar,
  HER-SPL-007; the exchange, GAM-DEC-003) although the two heroes are not neighbours: neither
  reads a distance (the decision names both heroes). The mover stays on the entrance; the
  meeting's events are its owner's alone, so no other viewer learns the pair through them. Once
  it closes, the entrance's guard visits as after any visit (note §7, ADV-MON-006). *Agent's
  call:* the meeting as the original's `visit_hero` gives it (batch E11-6 records the dialog and
  that the mover stays); an ally's hero on the pair (E10's teams) is E10's.
- *As built* (T105b1): `jumps.rs`' `jump` returns the outcome (the three visits pass it on): a
  hero of the player's own on the exit (a gate's pair alone) gives `meeting`, an enemy `battle`
  (the existing `battle_start`, so the sea's ground with two boats needs nothing new) and
  `jump_after`; `ADV-STUB-16` lifted, `ADV-QUIRK-39` written.

###### 13. The game (`h3-game`; GAM-DEC-007 amended, ADV-TEL-008; game state hash v26)

- **The visitor's stream** (ADV-TEL-003; ADR-0017 point 4's draft): `AdventureState::apply` gains
  `draws: &mut Rng`, the acting player's stream (its record in `GameState`), which the game passes
  for every adventure command; only `MoveHero` and `Revisit` can draw. The adventure draws only
  after the whole command is checked, so a rejected command leaves the stream untouched as the rest
  of the state (a property, §14). Within one command the jump's draw comes first, then the game's
  draws for the battle it opens (its seed; under `strict-fixed` its obstacles: point 4), in that
  order from the same stream. The answers and the rests take no stream: none draws (a held jump's
  exit was drawn at the visit). *Agent's call:* a parameter, not a stream inside `AdventureState`
  (the streams are the game's, ADR-0017 point 4; forks and the adventure state hash stay as they
  are). **Both profiles alike:** the original's draw is the C runtime's one `rand()` stream (§4.1),
  which no profile can reproduce (its position depends on every earlier call in the session), so
  the player's stream replaces it as for every draw a command causes; no quirk.
  *As built* (T105a1): `apply`'s `(player, draws)` (a tuple: the argument count's lint) and
  `visit_held`'s `draws`, from `GameState::adventure_for`, the acting player's (a rest's owner's).
- **The jump held as the visit's rest** (GAM-DEC-007 amended, a fifth rest): the adventure's
  outcome gains **`jump_after: Option<(HeroId, Pos)>`** (the mover and the exit), set beside
  `battle` when the exit holds an enemy hero (§12). The game holds it at once as `Rest::Jump(exit)`
  (the held rest's hash: the tag 4, then the tile; **game state hash v26**), so it waits for the
  battle and the winner's levels as every rest waits for its owner's open decisions, and gives it
  with **`AdventureState::arrive(context, (hero, exit), sink)`**: nothing when the hero left the map
  or the exit holds a hero or an empty boat (unreachable after a win, as the original's test that
  the exit is a gate again, 0x4AA4A6; tested so that a rest never corrupts the map), else note §7's
  arrival with its event (note §10). Its outcome opens nothing. *Agent's call:* the outcome's field
  and the rest, as `reward_after` and `Rest::Won`. Not chosen: the exit kept in the open battle and
  given at its end, which would move the hero before its levels (the original gives the levels
  first, 0x4DA720, then `hero_relocate`).
  *As built* (T105b1): `AdventureState::arrive(context, (player, hero), exit, sink)` (the owner
  beside the hero, as `visit_held`; the entrance is the hero's tile), the arrival proper
  `arrival`; `Rest::Jump` hashed with the tag 4 and the exit, **game state hash v27**; run as `Rest::Won`
  is, its arrival opening nothing.
- **The entrance's guard** (ADV-MON-006 amended; note §7, decision 279): the guard first, for every
  entered object; the tile's visit (the jump, a meeting, "leads nowhere") is the guard's rest, given
  to a hero still on the tile. Built before T105a; the jump's visit sets no `guard_after`.
  *As built (T103d):* the step's outcome is the guard's visit with `AdventureOutcome::visit_after`
  (the hero), held by the game as `Rest::Visit` (the hash's tag 0 as `Rest::Guard`'s before: no
  version moves, the hero being on the tile it entered while its owner has an open decision) and
  given by `AdventureState::visit_held`, which visits the object at the hero's tile as it stands
  (nothing for a hero off the map); the jump will be that visit. Batch E11c's E11-21 and E11-22
  are its tests (a mine, a Learning Stone).
- **`Revisit { hero }`** (ADV-TEL-008; decision 268; the original's space key, §11): an adventure
  command, **`AdventureCommand::Revisit { hero }`**, the entered object under the hero visited
  again.
  - **Its checks**, in order, each typed and leaving the state and the stream untouched: the
    game's (the player's turn, no open decision, `DecisionOpen`; no level-up holding the game);
    then the adventure's: the hero not the player's or not on the map (`NotYourHero`); the hero's
    tile not the visitable tile of an entered object whose kind has a built visit
    (**`NothingToRevisit`**, new; *agent's call:* one error for every case, naming nothing but the
    player's own tile).
  - **What it does:** that object's visit as a step's last step onto it gives it (the kind's visit
    at the hero's tile), with its decisions, events and outcome; **no points spent or required**
    (the original's visit tests none: a revisit at 0 points), the points and the day's maximum
    unchanged; **no step**, so no step event and no reveal of its own; **no guard test** (*agent's
    call:* the original tests a guard only at a step's end, `after_step_guard_check` 0x4800A5,
    §5, and the key calls `visit_and_update` alone; E11-19's last part checks it). A hero in a boat
    on a Whirlpool's tile revisits it (the loss again, the draw again).
  - **Which kinds:** every entered kind with a built visit (the original's gate: any entered class
    but 3, the Anchor Point, which the engine does not type): the teleporters, the Cartographer,
    the observatories, the Keymaster's tents, the weekly objects, the mines, the dwellings and the
    camps, the entered teachers and finds, a town (the town lane's visit; its inbox told: a revisit
    of the hero's own town). Not: a neighbour-visited object (the hero never stands on it), a boat
    (boarding is a step's), a tile without an object.
  - **The legal set** (`legal_commands`): a `Revisit` of each of the player's heroes on such a tile,
    in hero order after the `BuildBoat`s; `legal_moves` unchanged. Its replay line `revisit
    <hero>` (**adventure replay format v11**; v12 since E09 T18b1 took v11).
  - **Every player alike** (decision 16): the original's key works on a human's turn only and a
    computer never presses it; a computer player's use is E10's.
  - *As built* (T105b2): `state/revisits.rs`' `revisit` (the hero, then `revisit_at`: the live
    tile's `visit_at` when entered and not passed, then `visit`) and `revisitable` (the player's
    heroes in id order); the game's path as `MoveHero`'s (no price); the replay line `revisit
    <hero>`, **adventure replay format v12**.
- **No new decision kind and no new view:** the jump's battle is a battle (ADR-0017 point 6), a
  gate's meeting increment 6's, the rest no decision; the observation shows the hero on the object,
  and a client offers the key where `legal_commands` lists it.
- **Events:** the adventure's new ones (note §8's loss, note §10's `HeroTeleported` and "leads
  nowhere") reach viewers through `GameEvent::Adventure` with their projections; the game adds
  none.

###### 14. The env, replays, fixtures, properties, benchmarks

- **The env** (`h3-sim`, the AI lane's; `cross-lane` in the game's task with the user's consent,
  as decisions 138, 203 and 243): **action encoding v8** appends eight entries, `22 + h`, `h < 8`:
  `Revisit` of the hero in slot `h` (the move entries' slots), masked by `legal_commands`, the tile
  head ignored. *Agent's call:* one entry per hero slot; a jump needs none (a destination among
  the reach's tiles, ADR-0017 point 8). *As built* (T105b2): `REVISIT_BASE` 22, `ENTRY_COUNT` 30;
  a revisit entry takes no tile (tile 0 alone, as the answers': the head is not ignored but
  fixed), decodes to the `Revisit` of the hero in its slot (none for an empty slot). The observation the env returns is `GameObservation`
  (decision 187): the kinds' types in the objects' views, nothing else. A revisit at 0 points can
  repeat within a turn (§18).
- **Replays:** adventure replay format **v11** (`revisit`; v12 since E09 T18b1 took v11); every golden replay rehashed (the
  versions of §15; no recorded map holds a teleporter: the generators use 118, the lane's
  `TRAPS.md`). New, on **`e08-teleporters-36.h3m`** (`h3-map`'s `src/tests/teleporters_map.rs`,
  `h3_map::writer`), two levels and two players: a one-way channel of one entrance and three
  exits, a two-way channel of three, a lone one-way exit, two gate pairs and a gate without a pair,
  three Whirlpools in a sea with a boat for each player, the Sea Captain's Hat on one hero, a
  monster beside one entrance, a weekly object and a dwelling to revisit. Random play over 28 days
  (`teleporters-s<seed>-d28`, seeds by the search for jumps, losses and revisits); scripted ones
  (`teleporters-*-s1`): each kind's jump (a one-way's exits drawn, a two-way of three, a gate), a
  one-way exit's and an unpaired gate's "leads nowhere", a monolith without a candidate, the
  meeting at a gate, a battle at a monolith's exit won (the jump after the levels) and lost, a
  Whirlpool's losses (several stacks, a tie, a lone creature, the Hat) and A-37's battle, the
  entrance's guard, a revisit of a gate, a Whirlpool and the dwelling at 0 points.
- **Properties** (`h3-map`'s `play.rs`, `view.rs`): the plans place the five classes
  (`TELEPORTERS`) and play `Revisit`; checks after every command: the map's invariants (a hero in a
  boat on water, a walker on land, one hero a tile, no boat on land); a jump keeps the points and
  the day's maximum; the exit was a candidate (never a tile of a hero of the team, never the
  entrance, a Whirlpool's never one of its own six); a fork's channels are a fresh build's; a
  Whirlpool's loss only in the weakest slot by fight value × count, the army never empty; a
  rejected `MoveHero` or `Revisit` leaves the state and the player's stream untouched (the stream's
  position compared); no viewer is told an end of a jump it does not see, nor a channel's exits or
  a gate's pair.
  *As built* (T106b1, `adventure/e08-t106b1`; decision 291, the `play.rs` half): `TELEPORTERS`
  (the entrances of channels 0 and 1, channel 0's exit, the two-way monoliths of channels 0 and 1,
  the gate, the Whirlpool; a gate the original's four tiles, a Whirlpool two of a column),
  `Plan::teleporters` laid out after every other object but the towns and boats, on their surface
  (A-42 never met) and alone on their tiles; `Step::Jump` (runs of them), `Free::Revisit`; the
  checks `check_channels` (the lists against the raw map, at the start), `check_jumps` (every jump
  to one of `candidates`, an oracle of §7; every visit's outcome), `check_whirlpool_losses` (the
  army at the visit rebuilt from the event), the map's invariants in `check_invariants`, a jump's
  points in `check_legal`; the stream compared through the state's equality (its accessor is the
  game's own). *Agent's reading:* an enemy hero on another tile of the entrance's own Whirlpool is
  a candidate (survey §4.1: the hero's class before the excluded word; the engine's
  `draw_exit`), so "never one of its own six" holds for a free tile; its battle won, the held jump
  lands there.
  *As built* (T106b2, `adventure/e08-t106b2`; decision 292, the `view.rs` half): `check_jump_ends`
  (every `HeroTeleported` told to its hero's owner with both ends, to another viewer exactly when
  it sees an end, that end alone) and `check_teleporter_views` (a teleporter's view its type and
  tiles, nothing else) after every command for every player; in the hygiene property the plans'
  teleporters, `vary` removing those the viewer never sees and adding others there (on their
  surface, alone on their tiles; every teleporter's visited tiles keep their surface, A-42),
  `check_jump_ends` in both games, `outcome` leaving out the move of a hero that jumped where the
  viewer does not see the exit (the comparison goes on past another player's hidden jump), and the
  viewer's own visit of a teleporter ending the comparison when the two games tell it other events
  (the same up to the visit's).
- **Benchmarks** (`benches/adventure/`): **measured first** against `master` right after the first
  build that makes the entrances visits (T105a; `homm3-bench-regression`): `reach-xl-144`,
  `reach-lifted-xl-144`, `reach-sea-xl-144` and the `random-play-*` workloads hold no teleporter
  (the generators use 118), so a change is the dispatch's or the new parameter's and is fixed, not
  re-baselined, unless its reason is written. **New**, on **`e08-teleporters-144.h3m`** (both levels
  and a sea for the Whirlpools, `e08-sea-144.h3m`'s generator the model: the shipped maps' most of
  each kind on one map, each channel's exits spread over the map, a monster on every hundredth free
  tile):
  `reach-teleporters-xl-144` (the walkers' queries meeting the entrances as stop nodes),
  `random-play-teleporters-36` (jumps, losses and revisits in random play) and
  **`build-gates-xl-144`** (*agent's call:* a new bench workload `build`, the game's build of an XL
  map with a gate on every fifth free tile of both levels, about 8 000, the pairing's worst case,
  note §5), each baselined at its first count. If the pairing dominates the build, the lever is a
  nearest search by row bands that keeps the original's walk order and tie (§18).
  *As built* (T106c, `adventure/e08-t106c`; decision 293): the map's numbers are the census's most
  on one map (`--with-original-data` below): 8 one-way entrances and 8 exits over channels 0 to 3,
  17 two-way monoliths over channels 0 to 5, 26 gates, 19 Whirlpools, each with the original's
  template of its class and subclass (`teleporters_xl_map.rs`); the sea on the surface's rows 60 to
  83; each land teleporter in a 12 × 12 cell of its own, the cells drawn by a stride of 101
  through both levels' 264 land cells (a one-way channel's exits on both levels), the Whirlpools'
  by a stride of 5 through the sea's 24; ten walkers (two beside boats), 378 gnoll stacks.
  *Agent's call:* each workload's seed by a short search, as T106a's replays (the XL's seed 11 of
  1 to 24 for the most jumps: 7; the 36's seed 3 of 1 to 3 for the first with a Whirlpool's loss).
  `reach-teleporters-xl-144` 1 184 099 078 Ir (9 350 walkers' queries, 885 reaching a
  teleporter's tile), `random-play-teleporters-36` 6 009 640 722 Ir (2 195 steps, 19 jumps, 3
  losses, 23 revisits drawn); no engine code changed, so no other workload moved.
- **`--with-original-data`** (`crates/h3-map/examples/original_data.rs`): every shipped map's
  static layer with the five classes typed (the census's 1 671 objects: 602 gates, 845 monoliths
  by way and channel, 224 Whirlpools; §1), the channels, the gates' 301 pairs as the census found
  them (the 18 gates not with their nearest), no shipped map refused by A-39 or A-42; the harvest's
  check (note §3).
  *As built* (T104, T106c): the channels and the pairs since T104 (decision 286); since T106c the
  rest of survey §1 checked too (the counts by class, subclass and level, the maps with each class,
  the channels by ends and size, the Whirlpools' tiles and the maps by their Whirlpools) and each
  class's and way's most on one map recorded and checked (`SURVEY_MOST`).

###### 15. Versions, rule ids, quirks

| Version | Change | Task |
|---|---|---|
| pack format v51, core ruleset hash v52 (shared) | the effect kind `whirlpool_protection` (note §2) | T102: done |
| adventure ruleset part v18 | the three kinds, their packs (note §1, §3) | T103a: done |
| map import v21 (E09 T11 took v20 first) | classes 43 to 45, 103, 111; A-39's refusal (note §4) | T103b: done |
| game state hash v27 (E09 T17a took v26) | the held jump `Rest::Jump` (§13) | T105b1: done |
| adventure replay format v12 (E09 T18b1 took v11) | `revisit` (§13) | T105b2: done |
| env action encoding v8 (the AI lane's) | `Revisit`'s eight entries (§14) | T105b2 (`cross-lane`): done |

The adventure state hash stays v21 (note §6). The town lane's E09 bumps the same rows: **the second
to merge bumps again**, each change once (the table's numbers are this increment's if it merges
first).

Rule ids (survey §13): ADV-TEL-001 to -008 new; amended: ADV-OBJ-002, ADV-OBJ-003, ADV-FOG-004,
ADV-MON-006 (decision 279: the guard first, for every entered kind), HER-ADV-009 (note §2), GAM-BAT-004,
GAM-MET-001 (§12), GAM-DEC-007 (§13). Quirks (note §11): `ADV-QUIRK-39` A-37, `-40` A-38, `-41`
A-39, `-42` A-40, `-43` A-41, `-44` A-42, `-45` A-43 (decision 280), none with a flag; each entry written by the task that
builds its rule (§17). Stubs: none (a computer player's revisit and its paths through a channel
are E10's; no player meets them).

###### 16. Batch E11

Survey §12's eighteen rows and one added, in two runs (decision 269), on the harness
(`clients/capture/`, common) in SoD on Easy, hot seat, from maps `h3-experiment` writes: **E11a**
the jumps (E11-1 to E11-13, E11-18, E11-19; days 1 and 2), **E11b** the repetitions with the space
key (E11-14 to E11-17; one day).

| Row | Settles | Setup | Record | The reading predicts |
|---|---|---|---|---|
| E11-19 | note §7's entrance guards, §13's revisit | a weak monster M beside a two-way monolith A, another N beside its pair B | Red steps onto A: the battles, in order, and the tile after; then, on B, Red presses Space | unread; *the agent's call:* no battle (not M: the hero jumped; not N: the arrival tests no guard, E11-3), on B; at Space no battle (the guard test is a step's), on A |

One setup for note §7's two (M alone, then N too): M fought first tells a guard tested before the
visit or at the step's tile; N fought, one tested at the hero's new tile. **Predictions added to
the survey's rows:** E11-6 and E11-7 record the tiles explored within 1 of the pair before the
meeting or the battle (§12), and E11-7 the battlefield (the exit's terrain, §12); E11-17 adds an
empty boat on one of W3's tiles, never drawn (note §7's empty boat: the reading tests the tile's
top class, not read for a boat).

- **The writer and the spec's fields** (T99; `h3-experiment` is the battle lane's, `cross-lane`;
  the writer this lane's): the five classes by picture and subclass on either level
  (`[[map.object]]`; written without a body, the writer's class table checked; T89a the model), a
  hero in a boat on or beside a Whirlpool's tile and an empty boat on one (batch E8's boats), the
  worn Hats (batch E8's artifacts); the spec's checks of E11-9's walk order and truncated
  distances and of E11-16's and E11-17's tiles. **The specs:** `E08-batch-11a.toml`, `-11b.toml`
  and their templates; E11-18 needs Blue's turn and Red's next (day 2).
- **The harness** (T100; common, CLT-Q18: built with the harness check, the client and map-client
  lanes' inboxes told): Space repeated with a read after each press (the hero's tile, points and
  army; the step-by-step table's rows, E08 T68b), the Adventure Options' Replay Opponent Turn and
  the view's corner during it (E11-18), the meeting dialog closed unchanged (E11-6), the messages'
  texts (ADVEVENT 70, 153, 168); the recipes, the runs.
- **The scoring** (T101): survey §15 (new), each row against its prediction; a row against the
  reading opens an `ADV-Q` before the shared task; the rows each new rule cites (`verified` when
  built). *Done:* every row as read but E11-19 (ADV-Q47, decisions 279, 280).
- **Batch E11c** (decision 279's investigation, the agent's call): a hero arriving on a one-way
  exit beside a guard (no fight predicted, §5's reading) and its step off onto the guarded tile (the
  fight); a guarded entered object of another kind (a mine or a Learning Stone beside a monster:
  the guard first, the visit after a win); Space on a two-way monolith whose own tile is guarded (no
  fight, the user's play). Spec, run and scoring before T105a, their cut at round 5's start. *Done:*
  T103c, whole (decision 284): E11-20 to E11-23, every row as read (survey §15.1): the guard
  before a mine's flag and a Learning Stone's experience; no fight at a one-way exit's arrival.
- **Not in the batch** (survey §12): A-39 (refused at the import), A-40 and the computer's paths
  (E10), RoE's space key, network play; A-42 (refused when the game is built, whatever the original
  does; none on the shipped maps).

###### 17. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** the `shared/` task (T102, `cross-lane`, as increment 8's T70):
ADR-0016's and -0017's drafts applied with batch E11's outcome; `docs/03-domain.md`'s terms
(teleporter, channel, entrance, exit, jump, monolith, Subterranean Gate, Whirlpool, a Whirlpool's
loss, revisit); `docs/06-security.md`'s lines (note §10, §13); the effect kind
**`whirlpool_protection`** (`h3-content`'s artifact effect kinds: its code, name and hash;
`h3-pack`'s syntax; pack format v51, core ruleset hash v52; note §2) with the harvester's gap-fill
(the battle lane's: the Sea Captain's Hat's effect). `cross-lane` elsewhere: batch E11's spec
(`h3-experiment`, T99), the harness (common, T100), the env's encoding (`h3-sim`, T105b). The
inboxes: the AI lane's (v8, T105b), the map client's (the five kinds' types, T103a; the new events,
T105a), the town lane's (the stream in `AdventureState::apply` for its callers, T105a; a town's
revisit, T105b), the client lane's (the harness, T100).

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut confirmed
at its start). **Batch E11 runs before the build and before the shared task** (decision 269), so the
ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T99** | Batch E11's maps: the writer's five classes, the spec's fields (`cross-lane`), the two specs and templates (§16) |
| 3 | **T100** + **T101** | T100: the harness's reads and keys, the recipes, the runs E11a and E11b. T101: scored (survey §15) |
| 4 | **T102** | `shared/` (`cross-lane`): the two ADRs with T101's outcome, the glossary, `docs/06-security.md`, the effect kind with the harvester's gap-fill (pack format v51, core ruleset hash v52) |
| 5 | **T103** | T103a: the kinds, the packs, the harvest's check, `h3_heroes::whirlpool_protection` (note §1 to §3; part v18). T103b: the import, A-39's refusal and its entry (note §4; map import v21); a step onto an entrance refused as any unbuilt visit's until T105a. *Cut at its start* (decision 282): T103c batch E11c, T103d decision 279's order, after T103c |
| 6 | **T104** | The build: the surfaces (A-42 and its entry), the channels, the gates' pairs (note §5); `build-gates-xl-144` |
| 7 | **T105** | T105a, in two by land and water (decision 287): T105a1 the stream in `apply`, the monoliths' and the gates' jumps, the events and views, the entrance's guard, the reach's measure against `master`, A-41's entry (done); T105a2 the Whirlpool, its loss and jump (A-38's, A-40's entries; done). T105b: the battle and the meeting at an exit, the held jump, `Revisit` and its legal set, the env (§12, §13, §14's env; game state hash v27, E09 T17a having taken v26; replay format v11, `cross-lane` v8; A-37's entry) |
| 8 | **T106** + **T107** | T106: the teleporters' maps, their replays and properties, the benchmarks, `--with-original-data` (§14). T107: the audit against the exit criteria; increment 11 closed; increment 12 refined (both done: T106a to T106c, T107 whole) |

*Added at T101* (decisions 279, 280): batch E11c (§16) and decision 279's order for every entered
kind (ADV-MON-006, GAM-DEC-007), both before T105a; their ids and cut at round 5's start.

**Increment 11's exit criteria:**
1. The five classes import typed (a monolith's channel and way by its class and subclass, the gate,
   the Whirlpool) by the packs of their expansions; A-39 is refused and recorded (ADV-TEL-001,
   ADV-OBJ-002).
2. The game's build makes the channels and the gates' pairs in the walk's order as the original
   (D-56) without a draw, and refuses A-42 (ADV-TEL-002).
3. A step onto an entrance ends the move and jumps: a monolith to a candidate of its channel (one
   uniform draw from the visitor's stream for two or more, none for one), a gate to its pair, a
   Whirlpool after its loss to another's tile; a one-way exit and an unpaired gate lead nowhere, a
   monolith without a candidate does nothing (ADV-TEL-003 to -006).
4. An enemy hero on the exit is fought first (radius 1 revealed), the mover jumping after its
   levels if it wins, A-37 so fixed; a hero of the team on a gate's pair is met, the mover staying
   (GAM-BAT-004, GAM-MET-001, GAM-DEC-007).
5. The arrival keeps the points, fights no guard, visits nothing and reveals the scouting radius;
   the Whirlpool's loss takes D-54 and D-55 without the bound (A-38), every hero alike (A-40)
   unless protected by the Hats (ADV-TEL-006, -007, HER-ADV-009); a guard beside an entered
   object, an entrance too, is met before its visit (decision 279; ADV-MON-006, GAM-DEC-007).
6. No path goes through a teleporter: the reach and `MoveHero` meet an entrance as a destination;
   the reach's workloads inside their gate (re-baselined by T105b2 for the random games' course;
   `reach-teleporters-xl-144` the new one) (ADV-OBJ-003, D-58; worded as built by T107).
7. `Revisit` visits the entered object under the hero again without points, every entered kind
   with a built visit; the env plays it (ADV-TEL-008).
8. A viewer sees a teleporter's type, never its exits or its pair, and a jump as a step's two ends,
   never a hidden exit (A-41; ADV-FOG-004); the loss and "leads nowhere" are the visitor's owner's
   alone (ADV-TEL-004 to -006; the citation corrected by T107).
9. Every new rule has its id, source and tests; `ADV-QUIRK-39` to `-45` recorded (`-45` A-43's,
   T104); the gates pass (coverage, determinism with the new and rehashed replays, the benchmarks
   at their baselines and the new ones baselined).
10. Batch E11 run and scored (T101), batch E11c too; the consensus rows `verified`.

###### 18. Risks and open points

- **Two lanes in the same crates:** the town lane's E09 tasks bump the versions of §15 and touch
  `h3-game`'s outcome and rests and `AdventureState::apply`'s callers; each merges in order, the
  second rebasing and bumping again (the town lane's `TRAPS.md`). The new parameter of `apply`
  (T105a) touches every caller (the game, the crates' tests, the CLI's benches, the properties): a
  wide, mechanical change, its own commit, the town lane told first.
- **Revisit at 0 points repeats** (the original's "unlimited cycling", decision 268): an agent or
  random play can revisit without end within a turn (a two-way channel re-drawn, a Whirlpool's
  losses again). The property plans and the random-play workloads bound their commands a turn;
  the env's rollout needs a step cap (the AI lane's call, its inbox told with v8).
- **The reach's stop nodes:** a map with many entrances gains destinations; §14's measure decides,
  with `reach-teleporters-xl-144` as the new baseline.
- **The gates' pairing is quadratic:** harmless on the shipped maps (at most a few dozen gates a
  map), about 17 M comparisons on a crafted XL (note §5); `build-gates-xl-144` measures it, the
  row-band search is the lever (§14).
- **The guard's order** (decision 279) changes increment 3's built rule for every entered kind:
  golden replays with a guarded entered object are re-recorded, the game state hash bumps with its
  task (the second of it and T105b bumps again), the town lane's inbox told (a guarded town's visit
  after the guard).
- **A-42 refuses** a crafted map the original loads (none shipped); the map client's inbox hears of
  the refusal with T104.
- **Not read** (survey §5, §9.1, §10): `hero_relocate`'s network message, the Replay Opponent
  Turn's display (A-41 not reproduced either way), the computer's paths through a channel
  (`pathfinder_teleport_exits` 0x56A400 in part, `ai_fight_answer`; E10), RoE's space key.
- **Not in increment 11** (decision 260): garrisons and the other visitables, the adventure spells
  (Dimension Door, Fly, Town Portal), creature banks, quests, simultaneous play, towns (E09), a
  computer player's routes and revisits (E10).
