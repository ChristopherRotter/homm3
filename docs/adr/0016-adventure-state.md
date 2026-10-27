# ADR-0016: Adventure state: persistent grids, object kinds and fog of war

- Status: Accepted (2026-10-10)
- Date: 2026-10-09
- Deciders: user + adventure lane (refinement 2026-10-07/08, decisions 1, 6 and 7 of the epic)

<!-- Decisions only. The reasoning is in docs/epics/E08-adventure-map.md. Longer than 80 lines
     because it covers decisions 1, 6 and 7 at once. -->

## Context

Search and batched self-play fork the state all the time (`docs/07` §6). A SoD map has up to
144×144 tiles on two levels plus a layer per player, so a fork must not copy map-sized data
(E08 decision 6, a hard requirement). The original has about 200 object classes; `docs/03`
left open whether objects are a closed enum or data-driven primitives. No logic crate holds an
`Arc` yet.

## Decision

1. **Three kinds of storage in `AdventureState`.**
   - **The static map**, one block behind an `Arc` per game, never written after setup:
     dimensions and levels, terrain, roads and rivers per tile, the static data of every object
     (position, template masks, kind and parameters), the tile masks of objects that can never
     change.
   - **Persistent structures** for map-sized data that changes. A per-tile layer (fog per
     player, tile overlays) is a wide, shallow grid: an `Arc` to a table of leaves, a leaf being
     a block of tiles (a bitset or a value block, for example 8×8). Uniform leaves share one leaf
     through an `Option` or a sentinel held by the grid, never a static. Per-object state is a
     chunked table of the same shape in one dimension. A write copies at most the table and one
     leaf (`Arc::make_mut`, `alloc::sync::Arc`; `Arc`, not `Rc`, so a game stays `Send`).
     Per-object state of increment 2 is whether the object is still on the map; increment 3
     adds a guarding object's **army left** (a monster's count, an object's guards), written when
     a battle it survives ends (its static part keeps what setup drew; a monster keeps the
     survivors of all its stacks, an upgraded stack's as its own creature: A-6 fixed). Per-player
     state of object types (a Cartographer kind bought, the keys held) is a plain array per
     player. A removal clears the object's tiles in a persistent occupancy layer (a grid as the
     fog's) that the movement rules read; the static map keeps every footprint.
   - **Plain arrays** for the small, hot parts: heroes on the map, their armies with the army's
     formation, movement points.
   - **Increment 7's block** (E08 decision 133: bounded only by the map's objects, never by the
     original's 32 marks per class, A-17, or 400 reward records, A-19), behind one `Arc`, none
     until the first visit that writes one: **per object**, an object **emptied** (a Corpse,
     Wagon, Lean To or Warrior's Tomb searched, a Pyramid looted) beside the removal, and the
     **army left** by a battle (a box's, an event's, a guarded artifact's or scroll's guards,
     besides a monster's) in a table of leaves whose copy on a write is bounded, not the whole
     map's (one ordered map behind an `Arc` copied every entry: 16.5 M instructions with 41 472
     entries, measured); **per hero, by hero id**, the **once-per-hero marks** (which objects of
     the once-per-hero classes the hero has visited), a bit set by object index kept beside the
     heroes on the map, not in them, so a hero that leaves the map keeps its marks (the original's
     hero record) and a fork does not copy one set per hero; **per player**, its **knowledge of
     objects** (the original's team marks: which objects a hero of the player has visited), a bit
     set by object index. A fork costs one reference count more; a write copies the block's
     handles and one table and leaf of the set it changes (1 to 3 thousand instructions with eight
     heroes and players on the XL map's 41 472 objects, measured in E08 T55b). The **reward
     records** are static (the static map), one per box or event, never shared (A-19 not
     reproduced).

   A fork is `Clone`: a few reference increments plus the plain arrays. The structures are
   hand-rolled in `h3-adventure` (no persistent-collections dependency) and move to the kernel
   only if a second context needs them. The leaf size is chosen by benchmark in T5, against a
   flat per-player copy as the baseline.
2. **Nothing sees the sharing.** Equality, the canonical state hash, `Clone`, `Debug` and the
   debug serialization (ADR-0003 point 12) see content only. The hash walks tiles and objects in
   canonical order, so equal content hashes equal however it is shared. Pointer identity
   (`ptr_eq`, `as_ptr`) never enters logic or the hash; the persistent types may use it only as
   a shortcut that cannot change a result (equal pointers, equal content), and `check-arch`
   rejects the tokens elsewhere. Two property tests prove copy-on-write (T5): a write through a
   fork never reaches its parent or a sibling; equal content compares and hashes equal whatever
   the sharing.
3. **Heroes on the map** are an adventure type (ADR-0013 point 1): `HeroId`, owner `PlayerId`,
   tile and level, army and its formation, movement points. A hero's details are `h3-heroes`'
   model (ADR-0013 point 2), kept in the game's store: the adventure reads them by reference (the
   game passes the store to its commands and queries) through `h3-heroes`' derivations, and
   returns what a visit changes in a hero as typed hero changes in its outcome, which the game
   applies (ADR-0017 points 1 and 10; no adventure profile, E08 decision 25). The adventure owns
   the hero on the map and its army; a battle's outcome reaches them through the game's
   infallible application (ADR-0017 point 11). From increment 5 a hero's **day's points** (the
   land table at its army's slowest speed, with the hero's bonuses) and **scouting radius** come
   from its details through `h3-heroes` (ADR-0013 point 13), set when the game builds the state,
   after the levels at load (ADR-0017 point 1), and the points again at each turn start of its
   owner (ADV-MOV-002); a gain in the day changes neither the points left nor the explored tiles.
   From increment 6 a hero on the map holds its army (at most the ruleset's seven slots, at least
   one stack), its **formation** (loose or tight) and its **tactics flag** (on at the start, the
   original's +0x48 = 2), besides its tile and points. Its owner **manages the army**: a stack
   moved onto another slot merges into one of its creature, fills an empty slot or swaps; a split
   leaves both stacks non-empty (the original's dialog's bounds, A-16's included, refuse only a
   split's 0 or full count, which a move covers where the original's click allows it: E08
   decision 98); a stack is dismissed while its army keeps another. Between two armies a hero's
   last stack never merges away nor moves to an empty slot (it may swap). The armies are the
   hero's own, the two heroes of a meeting, or the hero and the group of an army dialog (a small
   army of the decision, seven slots, no hero, none of whose stacks is protected); counts keep
   their full width (A-15 fixed). A stack's count never passes the ruleset's cap (an adventure
   parameter, 1 000 000; E08 decisions 109, 114): a merge past it is refused, a split's count
   keeps both stacks within it, a group that joins or is raised takes a free slot when its
   creature's stack cannot take it whole, else finds no room (E08 decision 115); a setup's
   stacks and groups above it are invalid, and later rules that add creatures keep it. The
   formation and the flag change at no cost. The adventure
   validates and applies these commands; the game routes them and checks the turn and the
   decision (ADR-0017 points 1 and 6). From increment 7 a visit's changes to the hero come back
   as hero changes (ADR-0013 points 2 and 15); its morale and luck bonuses are the details'. The
   once-per-hero marks are the adventure's (point 1).
4. **Map objects are closed behaviour kinds with data parameters** (as abilities, ADR-0012;
   answers the `docs/03` question). An object is its static part (position, template masks of
   blocked and visitable tiles, object type), a kind from a closed enum in `h3-adventure` with
   typed parameters, and per-kind state (point 1). The import maps the original's class and
   subclass numbers to object-type keys; it is the only place that knows them. An object type
   (adventure ruleset part, `adventure.toml`, sources per type) names its kind, the kind's
   type-level parameters and two visit properties: **entered** or **visited from a neighbouring
   tile**, and **visitable from above** or not. Instance parameters come from the map and are
   validated at setup. A rule with a rule id reads the kind; a new behaviour is a new variant
   plus its rule; no scripting, no code keyed by names or class numbers outside the import's
   mapping. A class without a type is a `Blocker`: its blocked and visitable tiles block.
   - **Visits.** A blocked tile cannot be entered. An entered object's visitable tile is
     entered by a step, which visits it and ends the move. A step toward a neighbour-visited
     object's visitable tile visits it from the tile the hero stands on: the step's price is
     paid as if taken (ADV-MOV-006's rule included), the hero does not move, the move ends. A
     path continues past no visitable tile, except where the object lets its owner's or key
     holder's heroes pass (the border gate). An object not visitable from above is neither
     entered from nor left toward the three tiles above its visitable tile.
   - **Random contents are resolved when the game is built**, from the setup stream in object
     order (ADR-0017 point 4), into the static map: a type gives the draw (a random
     resource's resource, a pile's amount, a campfire's content). A rule that draws at a visit
     draws from the visitor's stream (ADR-0017 point 4); increment 2 has none.
   - A tile holding another player's hero is visited from a neighbouring tile, and the visit is
     a battle at once; an own hero is met (increment 6, below); a hero's own tile is always
     leavable.
   - **Increment 2's kinds** (E08 decision 13): `Blocker`; `Pickup` (resources gained, then the
     object leaves: piles, random resources, campfires); `RevealTerrain` (a price, a set of
     terrains on every level, once per player and type: the Cartographers); `RevealArea` (a
     radius around the visited tile, every visit: the Redwood Observatory, the Pillar of
     Fire); `KeyTent`, `KeyGuard`, `KeyGate` (a key colour: the tent gives the key, the guard
     leaves the map for everyone when a key holder accepts, the gate lets key holders pass).
   - **Increment 3's kinds** (E08 decisions 19, 22, 23, 26): **`Monster`** (classes 54, 71–75,
     162–164): a level range for a random monster (the type's), else the creature as instance
     data; a count (drawn at setup from the creature's adventure-map range when the map gives
     none); a disposition, whose aggression is drawn at setup; never flees; no growth
     (increment 5); a treasure of resources. Visited from a neighbouring tile, also from above.
     It **guards** the eight tiles around its visitable tile on its level, of its own water
     status (the original's 3 × 3 scan and its order): a step onto a guarded tile ends the move
     and visits the guard; a path continues past no guarded tile but its start. **The
     monster's answer** (ADV-MON-007) is the adventure's rule, from the visiting hero's army and
     details (Attack, Defense, Diplomacy's join bonus, through `h3-heroes`), the monster and
     the game's difficulty (a static input of the adventure's setup): a battle or an offer
     (join, join for gold, pursue); the monster leaves after a declined pursuit or when it
     joins. A joining group becomes one stack of the hero's army (with a stack of its
     creature, else in a free slot) and its treasure's resources go to the owner; with
     neither, the join offer was skipped as declined until increment 6 (below). **The
     split** (ADV-MON-009): a monster's stacks in its battle follow from its count and the
     visitor's army by the original's table and two draws, in `faithful` the original's
     generator seeded from the tile, written as the rule (`ADV-QUIRK-6`, fix-optional), in
     `strict-fixed` from the attacking player's stream. **Guarded objects** (ADV-PIK-004): a
     pile's guards from the map (up to seven stacks, an optional question); a question is an
     offer, none a battle at once; the guards fight unsplit; a won battle picks the pile up.
     **`Ground`** (Cursed Ground, Magic Plains, SoD's special grounds): no visit; a battle on
     one of its tiles is fought on its battlefield (ADR-0017 point 11). Random contents of
     increment 3 are drawn at setup, in object order, from the setup stream; the import refuses
     what increment 3 cannot represent (a treasure with an artifact, a count above 4 095).
   - **A hero's passage** (increment 5) depends on how it moves: walking, flying (a flying level
     from its details) or water walking (a water-walking level); a hero that does both flies.
     **A walking hero** moves as above. **A flying hero** also passes over blocked tiles, water,
     guarded tiles and the visitable tiles of objects and heroes, never over rock and never over
     an **unexplored** tile: moves stay on explored tiles for every hero (A-13 fixed for now,
     `ADV-QUIRK-15`). **Provisional** (the user at ADV-Q20 and ADV-Q21): "probably fix, but
     gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps
     deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD
     later anyway". The compromise, and with it `ADV-QUIRK-15`'s class, is decided later; until
     then nothing hidden enters the legal moves. A flyer is attacked by a guard and visits an
     object or a hero only at its move's end, and a step down onto the visitable tile of an
     object not visited from above stays impossible. **A water-walking hero** also passes over
     water that holds no object; on land, and for the guards, it is a walking hero.
   - **A move ends only on a tile a hero may stop on** (D-25): a free land tile, or a visit made
     from such a tile (an entered object, or a neighbour's object or hero visited from it). A
     tile only passed over (blocked, water, a visitable tile flown over, a tile a flight enters by
     a step up off the visitable tile of an object not visited from above) is never a move's end
     nor the tile a visit is made from. The **reachable-tiles query** keeps two kinds of node per
     tile for a flying or water-walking hero, one it may stop on and one it passes, and one kind
     for a walking hero (its search unchanged); a path follows each node's predecessor of its
     kind. Its destinations, and the env's mask (ADR-0017 point 8), are the tiles a hero may stop
     on.
   - **The step's price** follows the hero: its Pathfinding level, Nomads in its army, flying and
     water walking (the original's `step_cost` 0x4B1740). A flying step costs the lesser of the
     fly and the walking price from a land tile no object blocks, and the fly price from water and
     from a blocked tile, in every profile (D-24: the original's rule, batch E5's E5-4 and ADV-Q22;
     `ADV-QUIRK-16` withdrawn). One fix-optional quirk: the drop to 0 after a step prices the
     cheapest step without the artifacts' flying and water walking (A-12, `ADV-QUIRK-17`). Cursed
     Ground takes the native price away (D-23, confirmed in SoD by batch E5's E5-6).
   - **Increment 6's visits.** A hero visited from the neighbouring tile (a step toward it, at the
     step's price, the move ending) is attacked when another player's (GAM-BAT-004) and **met**
     when its owner's: the outcome reports the meeting, which the game opens as its owner's
     decision (ADR-0017 point 6). The cases of an object under the visited hero and of tops not
     visited from above stay as for the battle's visit. A join that finds no room is offered as
     any join; accepted, it reports the group, which the game holds as the army dialog's
     decision (ADV-MON-008 amended).
   - **Increment 7's kinds** (rewards on the map; the import maps classes 4, 5, 6, 22, 23, 26,
     32, 39, 41, 47, 51, 61, 63, 65 to 69, 81, 88 to 90, 93, 100 to 102, 104, 105, 107, 108 and
     113):
     - **Pickups that go to the hero:** a **treasure chest** (gold drawn, or a treasure artifact;
       gold asks gold or experience), a **placed or random artifact** (a random one drawn at the
       build by its classes), a **Spell Scroll** (its spell from the map). A guarded artifact
       always asks (D-29), a guarded scroll only with a text; a win picks it up. A **monster's
       treasure artifact** goes to the hero after a win or a join (the import takes it: the 8
       refused maps move on).
     - **Teachers:** the once-per-hero gifts (Marletto Tower, the Garden of Revelation, the
       Mercenary Camp, the Star Axis, the Arena, the Library of Enlightenment, the Schools of
       Magic and War, the Learning Stone, the Tree of Knowledge), the **Shrines**, the **Witch
       Hut**, the **Scholar** and the **University**. **Finds** emptied by a visit: the Corpse,
       the Wagon, the Lean To and the Warrior's Tomb. **Pandora's Box** (a reward record, a
       question, guards announced) and the **Pyramid** (a drawn spell, a question at every visit,
       a monster battle with a second creature, a looted Pyramid's luck).
     - **Events:** a reward record with its colours, computer flag and removal flag. An event is
       no visitable object: nothing is drawn or observed of it (point 6); a hero that **enters
       its tile by a step** fires it when the hero's owner's colour is in its colours (a player
       that is not human, ADR-0017 point 1's flag, only with the computer flag), and **the move
       ends there**: the rest of the path is dropped, the move's outcome, never a rejection. Its
       guards fight at once (no question); without the removal flag it fires again at every later
       entry, never for a hero that starts its move on it. **The reachable-tiles query never
       reads events** (blind, in every profile, no flag): batch E7's row E7-23 showed the
       original's path crossing a hidden event, and the original's placement clears an event
       tile's entrance bit, so its pathfinder never stops there (E08 decision 131: A-23
       withdrawn, T58). A step into a firing event mid-path ends the move there, as at the path's
       end.
     - **Visits.** A question is a pending decision of the visitor's owner (ADR-0017 point 6). A
       refusal (a full backpack, no spell book, too little Wisdom, a known spell or skill, eight
       skills, a price the owner cannot pay, a mark already set, an emptied find) is the visit's
       outcome, never a rejected step, and changes nothing but the visitor's owner's knowledge,
       which the original records first. When the hero cannot receive an artifact (ADR-0013
       point 15.4; under A-25's flag a full backpack, whatever the free positions), a placed
       artifact or a scroll refuses the visit, a chest's artifact turns into its gold choice, a
       Corpse's into gold and a Wagon's into its resources, a Tomb's is left, and a box's, an
       event's or a monster's is lost (survey `E08-rewards.md` §8.2). After a won guards' battle
       the visit's rest (the pickup, the rewards, the spell) comes after the battle's levels (the
       original's order).
     - **Random contents of increment 7 are drawn when the game is built**, from the setup
       stream in object order, the random artifacts first: the random artifacts and a chest's, a
       Corpse's, a Wagon's and a Tomb's artifact from one pool of the ruleset's artifacts that
       are placed at random (never one banned, placed or drawn before, until the pool is
       exhausted: D-32), a chest's gold, the Tree's price, a Shrine's spell (no spell of its level
       repeated until each is out: D-34), a Witch Hut's skill, a Scholar's kind and reward, a
       University's four skills, the finds' contents, the Pyramid's spell. The scenario's bans
       (skills, spells, artifacts) are static inputs. Nothing a visit does draws.
     - **No original bound** (E08 decision 133): a mark per object (A-17), a record per box or
       event (A-19); the import refuses a box's or an event's creature list longer than an army
       (A-24). An RoE map's Witch Hut without a mask takes the original's fixed 14 skills (mask
       0xEFBF) under `faithful` and every skill but Leadership and Necromancy under
       `strict-fixed` (A-21, `ADV-QUIRK-21`, a quirk with a flag).
5. **Fog of war.** Each player has an explored layer (a bitset grid). An explored tile stays
   visible with its current content until a rule clears it (Cover of Darkness): no line-of-sight
   memory (VCMI, survey `E08-adventure-core.md` §2). Reveals set disk-shaped areas;
   clearing is supported. A hero reveals with its radius (ADR-0013 point 13.4) when the game is
   built and around each step's tile; a radius grown in the day shows at the hero's next step
   (the original's, `ADV-QUIRK-18`, fix-optional): `strict-fixed` also reveals around every hero
   for its owner at each new day's start (D-26). The original's own new-day reveal around
   everything a player owns follows a Cover of Darkness's hide and comes with it (E09). The **fringe** is the hidden tiles touching an explored tile (the
   reveal's shape: ADV-Q5). A viewer sees on explored tiles the public part of their content; on the
   fringe terrain, roads and rivers, plus object appearance under the `faithful` profile once
   `ADV-QUIRK-1` is confirmed (Q30's rule: until then the clean behaviour only); nothing else of
   a hidden tile. Shared vision of allies comes with teams (E10).
6. **Information hiding.** `observe` and `project` take the viewer (ADR-0004 point 6). An
   event's visibility is resolved when it is emitted, with the state at hand: it carries the
   players who may see it (a move per step, since the fog changes during it), so `project`
   needs no state and a viewer learns of a move only where it can see it.
   Object contents beyond their appearance, other players' private state and all RNG state stay
   hidden. An object is shown whole (its type, its tiles, the viewer's own state of its type)
   once the viewer has explored any tile it covers (VCMI's `isVisibleFor`), or on the fringe
   under `ADV-QUIRK-1`; tiles show live occupancy. Contents (amounts, a campfire's resource) and
   other players' purchases and keys are never shown; an object's removal is seen by the players
   that have explored a tile it covered. Of a monster a viewer sees its creature and its count
   as the original's word (few … legion), never the count, aggression, flags or treasure; of
   another player's hero its identity, owner, class and its army's creatures with their words;
   of a battle nothing but its owners' (each its side's observation, ADR-0013 point 9), whose
   effects reach other viewers as removals where they see the tiles. The observation is materialised per viewer as flat planes; the env's encoding (T9)
   reads those, never the grids. T8 writes the viewer knowledge list into `docs/06-security.md`
   (the adventure epic owns it), including the fringe case of a public appearance with hidden
   contents. From increment 6 a hero's army, formation, flag and details are its owner's only,
   so every management change, a meeting and an army dialog are told to the owner alone; another
   player learns of a meeting only the step it sees (a step is a step), and of the group only the
   monster's leaving where it sees the tiles. From increment 7 chests, scrolls and boxes look
   alike; a placed artifact's picture names it; every content and every draw stays hidden until a
   visit. A visit tells its visitor's owner what the original's dialog shows and records the
   owner's **knowledge** of the object (a Shrine's spell, a hut's skill, a Tree's price, a
   University's skills, a find emptied, a Pyramid looted), which that player's observation shows
   from then on. A hero's once-per-hero marks and bonuses are its owner's. An **event is never
   observed**, not even by its visitor's owner after it fired; its firing is told to the visitor's
   owner alone, its removal to no one. Removals of picked-up objects are seen where the tiles
   are, as a pile's. The legal moves never depend on an event (A-23 withdrawn), so they reveal
   nothing of one before it fires.

## Consequences

- A fork barely grows with map size; T5 measures it on 36×36 and 144×144 (E08 decision 11).
  Hot reads pay an indirection per leaf; T5 measures that against the flat copy, and a grid
  that loses on hot reads goes back to the user.
- `arch.toml` forbids the pointer-identity tokens in logic crates outside the persistent module
  `crates/h3-adventure/src/persistent.rs` (E08 T2). On acceptance `docs/03` gains the terms static map, explored, fringe, object kind,
  blocked and visitable tile.
- Object behaviour grows by variants with rule ids; the import's class mapping is the only place
  that knows the original's class numbers; a monster's split generator lives in its rule, not
  the kernel. In `faithful` a split needs no player's dice (the same tile and armies split the
  same way, as the original's); in `strict-fixed` it costs the attacker two draws.
- Increment 5 reaches every caller of the query and of `MoveHero` (the env's mask,
  `legal_moves`, the properties' plans); a walking hero's search keeps its cost (`bench-check`), a
  flyer's or water walker's holds a second node layer. The state keeps no new field: the passage
  and the prices are rules over the hero's details.
- The legal moves stay a function of the explored tiles alone (ADV-MOV-008, `docs/06-security.md`)
  at A-13's price: a flight over a wide range of obstacles stops at its unexplored edge until it
  is explored. The compromise (a route to the closest unexplored but free tile) would let a flight
  end there and must then settle what that tells the player of the hidden tiles it crosses.
- Increment 6: `HeroOnMap` gains the flag (the adventure state hash and its version). The army
  commands touch only armies; a meeting's artifact moves are the store's (ADR-0013 point 14), so
  the game checks both before applying either. The original's human-only reading of the flags
  (the formation and the tactics phase for a human player's side) has no counterpart: the engine
  has no computer player (E08 decision 16); E10 decides for its computer players.
- Increment 7: the adventure state grows by one block (the state hash and its version); the army
  left changes its storage; `fork-xl-144` moves by a reference count at most once a reward was
  visited. The callers of the reachable-tiles query (`legal_moves`, the env's masks, the
  properties' plans) do not change for events (A-23 withdrawn). The monster battle gains a second
  creature (the Pyramid's), which `split` lays out by the original's order table (ADV-MON-012).
  The import maps 31 more classes and reads the scenario's spell and artifact bans.

## Alternatives considered

- A quadtree: pointer chasing on hot reads, disk-shaped reveals fragment it, the observation
  needs flat planes.
- An undo log instead of forks: may still serve tree search later; batched RL needs
  independent copies.
- Flat per-player copies: a fork costs the whole map; kept as T5's baseline.
- A persistent-collections crate: a dependency whose sharing, ordering and hashing we must
  control anyway.
- Objects as data-driven predicates: Tier 4 by the back door (ADR-0012). Code per original
  class: 200 match arms keyed by numbers.
- The split from the tile in every profile: predictable, so an attacker tunes its army to it
  (the user, ADV-Q13); monsters as their own context: they are map objects with a visit, the
  battle is the game's; a monster's exact count shown: the original shows the word only.
- Increment 5 (ADV-Q21): VCMI's layered pathfinder (a node per tile and layer: more than the
  original's two nodes per tile, which suffice until boats bring the sea's layer); the original's
  flight over unexplored tiles (A-13: the legal moves would depend on hidden terrain; kept as the
  basis of a later compromise); a flyer's visit from its tile refused inland (A-14, withdrawn by
  the user); a reveal at once when the radius grows (VCMI's for a skill: it rewards swapping
  radius artifacts between heroes).
- Increment 6 (ADV-Q24): exchanges whenever two heroes stand side by side, without a meeting (the
  original exchanges only in the meeting a step opens, which costs the step's points); one answer
  naming what to dismiss and where the group goes (a fraction of the original's dialog); VCMI's
  split bounds, or a flag for A-16.
- Increment 7 (ADV-Q28, ADV-Q29): the original's bounds (32 marks per class, 400 records) or a
  smaller bound of the engine's (E08 decision 133); the marks on each hero on the map (a fork
  copies a set per hero, 453 instructions with eight heroes against 19 for one block, measured,
  and a hero that leaves the map loses them); events as visitable objects (the original never
  draws them); A-23, an event's tile as a stop (withdrawn: the original's pathfinder never stops
  there, batch E7's E7-23 and the re-read, E08 T58); VCMI's side of the disputes (batch E7
  showed the code's side on every row, E08 T58).

## Revision history

- 2026-10-10: Accepted (E08 T1a draft; the T1b survey's findings on fog folded into point 5).
- 2026-10-11: point 1: the leaf size, measured against a flat per-player copy (E08 T5b,
  `02-architecture.md` §8): 32 × 32 tiles (the user). Full leaves share one leaf held by the
  grid, empty leaves are none.
- 2026-10-12: point 4 rewritten (object types with their kinds and visit properties, visits,
  random contents at setup, increment 2's kinds), points 1 and 6 extended (per-object and
  per-player object state, the occupancy layer; what a viewer sees of objects) (E08 T11b, the
  user at ADV-Q10; survey `E08-objects.md`).
- 2026-10-14 (E08 T18a; the user at ADV-Q12 and ADV-Q13): points 1, 4 and 6 extended and point 3
  amended for increment 3: a guarding object's army left, the heroes' formation, a hero's
  details read by reference and changed through hero changes, enemy heroes visited from a
  neighbour as a battle, the monster and ground kinds, guarded tiles, the monster's answer, the
  split by the profile (`ADV-QUIRK-6`), guarded piles, what a viewer sees of monsters, heroes and
  battles (drafted in E08 T17b, #181; survey `E08-battles.md`).
- 2026-10-22 (E08 T35a; the user at ADV-Q20 and ADV-Q21): points 3, 4 and 5 amended for E08
  increment 5, the hero's movement and scouting: the day's points and the radius from the details
  through `h3-heroes`, set after the levels at load and at each turn start; a hero's passage by
  walking, flying or water walking (A-13 fixed for now and provisional, with the user's words), a
  move's end only where a hero may stop (D-25), the query's two kinds of node, the step's price
  by the hero (`ADV-QUIRK-16`, `-17`; D-23 by batch E5); the radius's reveals (`ADV-QUIRK-18`,
  D-26). Drafted in E08 T34b (#375; survey `E08-hero-movement.md`).
- 2026-10-22 (E08 T41's shared part; ADV-Q22, E08 decision 85): point 4's step price corrected:
  a flying step costs the lesser of the fly and walking prices from every land tile no object
  blocks, in every profile (`ADV-QUIRK-16` withdrawn, its flag retired); Cursed Ground confirmed
  by batch E5.
- 2026-10-23 (E08 T45; the user at ADV-Q24 and ADV-Q25): points 3, 4 and 6 amended for E08
  increment 6, army and hero management: the hero's formation and tactics flag, the owner's army
  commands within a hero, between two met heroes and with an army dialog's group (a hero's last
  stack, the split's bounds with A-16 by E08 decision 98, A-15 fixed); an own hero met by a step
  toward it, a join without room offered and held as the army dialog; management, meetings and
  the group's army told to the owner alone. Drafted in E08 T44b (#412; survey `E08-army.md` §1–§4).
- 2026-10-24 (E08 T50's shared part; the user at T50's start, E08 decisions 109, 114, 115):
  point 3: a stack's count cap, a ruleset parameter of 1 000 000 (the original's 16-bit limits
  do not bind): merges refused past it, the split's count within it, a joining or raised group
  to a free slot else no room, setups above it invalid.
- 2026-10-27 (E08 T59; the user at ADV-Q28, ADV-Q29 and ADV-Q30): points 1, 3, 4 and 6 amended
  for E08 increment 7, rewards on the map: one block for objects emptied, the army left, the
  once-per-hero marks by hero id and each player's knowledge, bounded by the map alone (E08
  decision 133); visits' hero changes; the 31 classes' kinds (pickups, teachers, finds, Pandora's
  Box, the Pyramid, events that stop a move and that the reachable-tiles query never reads), their
  refusals (A-25's full backpack), their draws at the build, A-21's Witch Hut mask; the knowledge
  in the observation, events never observed. Drafted in E08 T55b (#484; survey `E08-rewards.md`
  §1–§9, §13); batch E7's outcome (T58, §15: A-23 withdrawn) and decision 145 folded in.
