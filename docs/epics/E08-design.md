# E08 design note

Increment 11's note: the teleporters (E08 decision 260). Increment 10's note (dwellings and hiring)
is in `E08-log.md` ("Increment 10's design note at its closure", moved by T97c), increment 9's
beside it ("Increment 9's design note at its closure", moved by T87c); the code's comments cite
these notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 11's
amendments, drafted in the lane's `adr-drafts/` (`0016-increment-11.md` by T98b1,
`0017-increment-11.md` by T98b2); written from T98a's survey and the answers at ADV-Q44 and
ADV-Q45 (the epic's decisions 260, 264–269). A building task adds an *As built* line to the section
it builds; the note moves to `E08-log.md` when the increment closes.
Facts and sources: survey `docs/rules/surveys/E08-teleporters.md` (§ numbers are the survey's
unless they name this note); the proposed rule ids are the survey's §13.

Written in two parts by side (decision 270): **parts A and B** by T98b1, the content and the map
(`h3-adventure`, `h3-pack-adventure`, `h3-map`, and the one effect kind note §2 asks of `h3-content`
with its `h3-heroes` derivation); **part C** by T98b2, the game, the env, batch E11 and the cut
(§12 on). ADR-0015 needs no amendment: the import still draws nothing, and the channels are the
static map's, built with the game (ADR-0016 point 1). *Agent's call (change freely)* marks a choice
the user may still change at the checkpoint (T98b2), where the note and both drafts are accepted
before anything is built (decision 261).

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

###### 3. The packs

- **`base`**: the monoliths of channels 0 to 2 in each way (nine types: RoE's and AB's
  `Objects.txt` list subclasses 0 to 2), the Subterranean Gate and the Whirlpool (both RoE).
  **`sod`**: the monoliths of channels 3 to 7 (fifteen types: only SoD's `Objects.txt` lists them;
  subclasses 3 to 7 stand on SoD maps only, §1). An RoE or AB ruleset meets an SoD map's channel 3
  as `MissingContent`, as any SoD object. `ab` adds none.
- Keys and names (*agent's call*; the building task fixes them): `base:monolith_entrance_0` to
  `sod:two_way_monolith_7`; the names the original's class names (`ObjNames.txt`: Monolith One Way
  Entrance, Monolith One Way Exit, Two-Way Monolith, Subterranean Gate, Whirlpool), a monolith's
  with its channel's number.
- **The harvest's check** (with the original data, `h3-map`'s tests): each monolith type's channel
  and way equal its class and subclass in the import's tables (note §4); the subclasses each
  expansion's `Objects.txt` lists (RoE and AB 0 to 2, SoD 0 to 7) are defined by the pack of that
  expansion; the names are `ObjNames.txt`'s. The types are hand-written source with their sources,
  as the mines'.
- **The synthetic packs** gain a monolith of each way in two channels, a gate and a Whirlpool, for
  the crates' own tests.

##### Part B — the map (`h3-map`, `h3-adventure`)

###### 4. The import (`h3-map`; map import v20)

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

**The entrance's guards** (an open point: not read; part C §16 adds a batch row). ADV-MON-006 as
built visits the guard of the hero's tile once the tile's object's visit returns (`guard_after`;
`visit_guard` reads the hero's tile as it is then), so after a jump it would meet the exit's guard
when the entrance had one, a combination nothing supports. *Agent's call until batch E11 says:* a
jump drops the entrance's held guard (the hero has left the entrance, and the arrival tests no
guard, §5); a failed jump leaves the hero on the entrance, whose guard visits as after any visit.
The original's order (the guard tested before the visit, after it at the step's tile, or after it
at the hero's new tile) is unread; the row: a monster beside A, the hero steps onto A with B free
and unguarded, then with B guarded too.

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
- **Versions** (parts A and B): map import v20, adventure ruleset part v18; the adventure state hash
  unmoved; shared: pack format v51, core ruleset hash v52 (note §2). A town-lane bump of the same
  rows in between: the second to merge bumps again (part C §15 orders them).

##### Part C — the game, the env, batch E11 and the cut (T98b2)

Named here for T98b2 (decision 270): §12 the battle and the meeting at an exit (GAM-BAT-004 and
GAM-MET-001 amended: the defender on the exit, where the battle is fought, the radius-1 reveal's
place), §13 the game (the jump held as a rest, GAM-DEC-007; the visitor's stream, ADR-0017 point 4;
`Revisit`: its name, validation and the env's action, with its map-side lines added to
`0016-increment-11.md`), §14 the env, replays, fixtures, properties and benchmarks (the reach's and
the passage's measure, a teleporters' map and workload, the gates' pairing cost), §15 versions, rule
ids, quirks, §16 batch E11 (its spec and recipe the first building round, decision 269; the row for
note §7's entrance guards), §17 shared and `cross-lane` work, the building tasks and the exit
criteria, §18 risks and open points. ADR-0017's amendment is drafted there.
