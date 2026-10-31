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
   - **Increment 8's boats** (E08 decision 164: no original bound; the original's 64 records are
     not reproduced, `ADV-QUIRK-32`): boats are no objects of the static map, because they move.
     Each boat is a record (its type, which is appearance only; its facing; its owner and last
     captain, kept and read by no rule until Summon Boat; whether a hero is aboard) in a
     persistent table by boat index, and the tiles where an empty boat lies are a persistent
     per-tile layer (a bitset grid as the fog's) that the movement rules and the reachable-tiles
     query read; a boat with a hero aboard lies on its hero's tile, which the hero on the map
     holds. A fork costs two reference counts; a boarding, a landing or a boat leaving the map
     writes one record and one leaf.
   - **Increment 9's owned objects, the week and the month's monsters** (E08 decision 192):
     - **Owners**: every ownable object (the mines, the abandoned mines, the Lighthouses, the
       Shipyards; the dwellings come with increment 10) has an owner or none, held in one block
       by ownable index (the static map lists the ownable objects in object order), shared by
       forks and copied at a flag. The owners at load come from the setup.
     - **A mine's guards** are the armies on objects (the table increment 3 made for the armies
       battles leave): what a Leave Guards dialog leaves there and what a battle leaves of them;
       an abandoned mine's Troglodytes drawn when the game is built are what the build drew for
       it (as increment 7's draws), the table overriding them once written.
     - **The weekly objects' contents** (the Windmill's resource and amount, the Water Wheel's
       gold, the Mystical Garden's gift and whether it is full): one block by weekly index (the
       static map's list of them), written at a visit (emptied) and at every new week (all of
       them).
     - **A monster's fraction** (the original's sixteenths, ADV-MON-013): a persistent table by
       object index beside the armies on objects, none for a fraction of 0; the count as it
       stands stays the armies' table's or the static map's.
     - **Objects added in play** (the month's monsters, ADV-MON-014): records appended after the
       static map's objects, so an added object's index follows theirs and every table by object
       index serves it unchanged; the layer of objects that left the map (increment 2) becomes
       the layer of the objects' **live presence**: the objects that left and the objects added,
       with the tiles each blocks, is visited from or guards. A fork costs a reference count; a
       month's spawn writes one block and the tiles' leaves it touches.
     - **A Shipyard's boat tile** is computed once when the game is built (ADV-SHY-001) and kept
       with the static map: the original never moves it.

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
   From increment 8 a hero on the map may be **in a boat** (it holds its boat's index) and keeps
   its **day's maximum**, the points its turn started with on its surface. The turn start sets the
   day's points and the maximum by the hero's surface: on land the land points (ADR-0013 point
   13.1), in a boat the sea points (ADR-0013 point 13.6). **Boarding** (a step from land onto an
   empty boat's tile) and **landing** (a step from a boat onto a free coastal tile) end the move;
   at the visit that follows the step the hero's maximum becomes the new surface's, computed then
   from its details and army, and its points become 0, or with free boarding (ADR-0013 point
   13.7; the Admiral's Hat) the points left × the new maximum ÷ the old one, truncated, the points
   left being those after the step's own drop to 0 (D-41, the original's order, confirmed by
   batch E8). The original keeps the old maximum without the Hat (A-28, `ADV-QUIRK-31`): not
   reproduced, no flag (E08 decision 166). Boarding clears nothing the engine holds yet (the
   original clears the Fly and Water Walk spells' levels; the spells come later); a hero in a
   boat neither flies nor water-walks, whatever it wears. A hero that leaves the map takes its
   boat (the boat leaves with it).
   From increment 9 an owned mine's visit by its owner opens the **Leave Guards** dialog: the
   army dialog of increment 6 between the hero's army and the mine's guards (merge, swap, split;
   the hero's last stack stays), whose group is kept as the mine's guards when it closes, never
   lost. Every player's hero gets it (the original shows it to a human only; a computer player's
   handling is E10's). A hero standing on an owned object's entrance hides it: a step onto that
   tile meets or fights the hero, the object is not visited and its owner does not change. A
   hero does not revisit the object it stands on (the original has no revisit command).
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
     guarded tiles and the visitable tiles of objects and heroes (on water, from increment 8,
     only an empty boat's and a boat's hero's: D-46, increment 8's kinds), never over rock and
     never over an **unexplored** tile: moves stay on explored tiles for every hero (A-13 fixed
     for now, `ADV-QUIRK-15`). **Provisional** (the user at ADV-Q20 and ADV-Q21): "probably fix, but
     gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps
     deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD
     later anyway". The compromise, and with it `ADV-QUIRK-15`'s class, is decided later; until
     then nothing hidden enters the legal moves. A flyer is attacked by a guard and visits an
     object or a hero only at its move's end, and a step down onto the visitable tile of an
     object not visited from above stays impossible. **A water-walking hero** also passes over
     water that holds no object; on land, and for the guards, it is a walking hero.
   - **A move ends only on a tile a hero may stop on** (D-25): a free land tile, or a visit made
     from such a tile (an entered object, or a neighbour's object or hero visited from it); from
     increment 8 also an empty boat's tile (a boarding) and, for a hero in a boat, every water
     tile and a landing's free coastal tile (increment 8's kinds). A
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
   - **Increment 8's kinds and the sea** (the import maps classes 8, 29, 59, 82, 86, 124 and
     225):
     - **Boats** (class 8) become boats (point 1), never objects; the import takes the template's
       subclass as the type, no owner, the original's facing 2, nobody aboard, and refuses two
       boats on one tile, a boat off water, on a tile an object blocks or is visited from, or
       under a hero. **Any hero boards any empty boat.** **A hero placed on water** is refused by
       the import (`Unsupported`, D-44, E08 decisions 163 and 178: batch E8's E8-5 showed the
       original keeping it on foot without a boat; no shipped map has one).
     - **The sea's pickups** are visited from the neighbouring tile and leave the map at their
       visit, without a question: **Flotsam** (29: nothing, 5 wood, 5 wood and 200 gold, or 10
       wood and 500 gold, a quarter each), the **Sea Chest** (82: nothing 20 %, 1 500 gold 70 %,
       1 000 gold and a treasure artifact 10 %), the **Shipwreck Survivor** (86: a treasure,
       minor, major or relic artifact at 55, 20, 20 and 5 %) are finds that leave, their contents
       drawn when the game is built from the setup stream in object order (the random artifacts'
       pool of increment 7); a hero that cannot receive the artifact gets the chest's 1 500 gold
       instead of both its parts, and loses the survivor's (`ADV-QUIRK-22`'s test). The **Ocean
       Bottle** (59) is a **message**: its visit tells the visitor's owner which object's message
       it read (the map's text, or the original's default by the bottle's column, are the
       client's: the engine holds no text) and it leaves; every visitor reads it, a computer
       player's too (A-27, `ADV-QUIRK-30`, not reproduced, no flag; E08 decision 166).
     - **Favorable Winds** (225) is a **ground of the sea**: it lies on its picture's tiles, is
       never visited and is no battlefield; a boat's step from a tile it tops costs two thirds of
       the step's price, truncated (66, 93 diagonally), as does the drop's cheapest step from
       such a tile. **Holes** (124) lie on their picture's tiles, are never visited and only
       forbid a landing there.
     - **A hero's passage in a boat** (the fourth mode beside walking, flying and water walking):
       water steps only, at water's price from the source tile (100, 141 diagonally, Favorable
       Winds as above), with neither roads, nor native terrain, nor Pathfinding; a diagonal step
       from water to water only when both side tiles are water (a diagonal landing needs no such
       test: E08 T66b's reading of 0x4B2867 and 0x481CDB); never onto an empty boat's tile nor a
       blocked water tile; onto land only by a **landing** onto a **free coastal tile** (the
       map's coast flag; no object blocking it or visited from it, no event, no hole, no hero:
       D-42, the original's class 3, with the wiki). The diagonal exception (ADV-MOV-005)
       compares the points left with water's price, never the winds' (D-43). The drop to 0 after
       a step prices the cheapest step from the new tile as the original's `cheapest_step_from`
       (100 at sea, 66 on Favorable Winds; on the coast its road's price, else the native price).
     - **From land onto water:** an empty boat's tile is a step for every mode from a stop node,
       so from land (boarding: the move ends there; no guard attacks on it; a flight or a water
       walk over water boards no boat, the boat being on the original's list of classes not
       entered, 0x63AA58, and a flight passes over it from land or water, batch E8c's E8-17d and
       E8-17e); a hero in a boat is visited from a stop node (a battle or a meeting); an object's
       visitable tile on water (a pickup, a monster, any class but a hero, a boat and the
       Shipwreck) is visited only from a boat (D-45 reverted by batch E8's E8-7), and a flight
       neither ends on nor passes over it (D-46, the original's code, confirmed by batch E8c's
       E8-17a and E8-17b, E08 decisions 179, 180); plain water and an object's blocked cell stay
       a flight's pass, plain water a water walk's. Batch E8 confirmed D-41 to D-43 (E08 decision
       162).
     - **At sea**, an object visited from the neighbouring tile is visited from the boat's water
       tile, an entered one is entered; another hero in a boat is met or attacked from the
       neighbouring water tile; a boat never reaches a hero on land; a step onto a guarded water
       tile is attacked at once; an event on water fires for a boat's step and ends the move
       there. A landing's guards attack after the landing, at the monster's tile as any guards
       (batch E8's E8-15).
     - **The reachable-tiles query** searches a hero in a boat over one node per tile (every water
       tile a stop; the landing a leaf from which no path goes on, as boarding is from land); a
       walking hero's search tests a boat only on a water neighbour, so its cost stays
       (`bench-check`). Today's reach only: the multi-day path's next day at the sea's maximum is
       not the engine's.
   - **Increment 9's kinds** (the import maps classes 42, 53, 55, 87, 109, 112 and 220; 220 as
     53's subclass 7, as the original's `object_class_values`):
     - **Ownable kinds** are entered (the visit ends the walk on the entrance). **Flagging**
       (ADV-OWN-001): a visit flags a neutral object or another team's for its visitor's owner,
       never an own or allied one (allies come with E10's teams: until then a team is one
       player); nothing is paid. The owners at load are the map's; the import refuses an owner
       colour no player of the scenario plays (an invalid map, ADV-OWN-002), and an ownable
       object whose entrance lies on water (`Unsupported`: how a hero reaches it was not read;
       none of the shipped maps has one). A defeated player's objects become neutral, a mine
       keeping its guards (ADV-OWN-004), with E10's defeat.
     - **Mines** (`mine`: its resource and daily amount, the type's battle terrain if any;
       ADV-MIN-001, -002): an own mine's visit opens the Leave Guards dialog (point 3), an ally's
       does nothing; another's with guards asks first (a guarded pile's question, ADR-0017 point
       6), then fights them: a **neutral mine's single stack** as a wandering monster (split by
       the army ratio table, its draws by the profile, `ADV-QUIRK-6`; D-47: batch E9 showed the
       original splitting an abandoned mine's Troglodytes in every run, VCMI's single stack
       wrong), any other guards as an army without a hero, their owner's or none (an owned
       mine's played by its owner, batch E9); a win clears the guards and flags the mine; a
       loss, a retreat or a surrender leaves the guards' survivors on the mine and its owner
       unchanged. A battle on a crystal, gold or abandoned mine's entrance is fought on its
       type's battle terrain (the subterranean, GAM-BAT-001 amended), whatever lies under it.
       The guards' slot is the slot the stack holds (A-29 fixed: the original reads slot 0,
       which only E10's defeat can leave empty).
     - **The abandoned mine** (`abandoned_mine`; ADV-MIN-003): its resource drawn when the game
       is built among the map's set (equal chances; the original draws at the map's load), its
       guards drawn then too: the kind's creature (Troglodytes), 100 to 200 inclusive (D-48 as
       read); its daily amount its drawn resource's mine's. The import refuses an empty set or a
       bit past the seven resources (A-31: an input bound, no flag).
     - **Lighthouses** (`lighthouse`, the points it adds; ADV-OWN-005): flagged by a visit; each
       Lighthouse a player owns adds the kind's points (500) to the day's sea points of that
       player's heroes (ADR-0013 point 13.6), read where the day's sea maximum is computed.
     - **Shipyards** (`shipyard`: the boat's price and type, the offsets; ADV-SHY-001 to -003):
       the boat tile is the first of the kind's twelve offsets around the entrance, on its level,
       that is water no object blocks and no object is visited from, when the game is built; none
       found, the Shipyard never builds. Its visit flags it, then makes the boat offer for the
       price (ADR-0017 point 6) unless the tile is missing or holds a boat or a hero (the visit
       tells its owner why); an accepted offer pays and puts an empty boat of the kind's type on
       the tile, owned by the builder (ADV-SEA-001's record). An own or allied Shipyard also
       builds without a visit (`BuildBoat`, ADR-0017 point 1), under the same refusals.
     - **The weekly objects** (`windmill`, `water_wheel`, `garden`; ADV-WEK-001 to -003):
       entered, as the mines (the user at ADV-Q39, E08 decision 214: the original's classes 55,
       109 and 112 are on neither its "never entered" nor its "from above" list, and batch E9's
       hero stood on a Windmill's entrance), owned by nobody, never asking. **Their contents are
       the object's, one a week:** the week's first visit, by any player's hero, takes them and
       empties the object for every player; every later visit that week, by any player, finds it
       empty and is told so (the user at ADV-Q38, E08 decision 201: never once per player;
       batch E9 confirmed it). A visit also marks the visitor's team (the Garden's mark the
       visitor's owner alone), a mark no week clears and that never gates the gain: the original
       keeps it for its hint only (ADV-VIS-006). The contents when the game is built and at every
       new week, visited or not, drawn in object order: the Windmill one of its resources
       (mercury to gems) and an amount of 3 to 6, both drawn; the Water Wheel its first week's
       gold (500) at the build, its later gold (1 000) every week; the Garden full with one of
       its two gifts (5 gems or 500 gold), drawn.
     - **The week's step** (ADR-0017 point 3): the weekly objects' contents as above, then
       **every wandering monster grows** unless it does not grow (ADV-MON-013): its count kept in
       the original's sixteenths (the count × 16 plus the fraction), a tenth of that value added,
       truncated, the result capped at 4 000 × 16 (A-32: a larger stack from the map becomes
       4 000 at its first week; no flag; batch E9's E9-10); a battle's survivors keep the
       fraction (D-49, confirmed by batch E9's E9-9). A monster's fraction is kept apart from its
       count: a battle writes the count as before, the week reads both.
     - **The month's step** (ADR-0017 point 3), in a month of a creature (ADV-MON-014): every
       land tile of both levels in level, row and column order that no object blocks or is
       visited from, with no event and no hero, gets a monster with one chance in 200: the
       month's creature, a count of twice a draw from the creature's adventure-map range, an
       aggression drawn 1 to 10, growing, fleeing as any, without treasure (D-50 as read). It is
       an object added in play (point 1) and guards as any monster from then on.
5. **Fog of war.** Each player has an explored layer (a bitset grid). An explored tile stays
   visible with its current content until a rule clears it (Cover of Darkness): no line-of-sight
   memory (VCMI, survey `E08-adventure-core.md` §2). Reveals set disk-shaped areas;
   clearing is supported. A hero reveals with its radius (ADR-0013 point 13.4) when the game is
   built and around each step's tile; a radius grown in the day shows at the hero's next step
   (the original's, `ADV-QUIRK-18`, fix-optional): `strict-fixed` also reveals around every hero
   for its owner at each new day's start (D-26). The original's own new-day reveal around
   everything a player owns follows a Cover of Darkness's hide and comes with it (E09). From
   increment 9 a **flag** reveals the tiles within 3 of the object's entrance to its new owner
   (its team from E10; ADV-FOG-002's disk), and the start of the game reveals the same around
   every object owned at load, a Lighthouse too, as the original does (batch E9's E9-4 and E9-6;
   its `reveal_owned` from `adventure_manager_start`, as VCMI: A-33 withdrawn). The **fringe** is the hidden tiles touching an explored tile (the
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
   nothing of one before it fires. From increment 8 a viewer sees on its explored tiles every
   **empty boat**: its tile, type and facing, never its owner or captain; of every hero it sees
   whether it is in a boat (its boat's type: the original draws the hero's flag on it); an own
   hero's day's maximum is its owner's. The sea's pickups look alike to every viewer until a
   visit, as the chests; their contents stay hidden. A boarding, a landing and a boat leaving
   with its hero are seen where their tiles are; a bottle's message is told to its visitor's
   owner alone. From increment 9 every viewer sees an **owned object's owner** where it sees the
   object. A mine's daily production, and so an abandoned mine's drawn resource, is its owner's
   team's knowledge, never another's; a mine's guards are seen as a guarded pile's (their
   creatures and words), their counts by the owner's team. A Shipyard's boat tile is static and
   public with it. A weekly object's **visited mark** is its visitor's team's (the Garden's its
   visitor's owner's), kept through the weeks (the original's hint reads "visited" from it,
   never "emptied"); its contents are never shown, before a visit or after. A month's monster is
   seen as any monster where its tile is; a monster's fraction is never shown (its word follows
   its count). A flag, a built boat and an added monster are seen where their tiles are; a
   mine's income, a weekly object's gain and the guards' exchange are told to their owner alone.

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
- Increment 8: the adventure state grows by the boats (a table and a grid), the hero's boat and
  its day's maximum (the state hash and its version); `fork-*` moves by two reference counts.
  The reachable-tiles query gains a search for a hero in a boat and the boat rows in the
  others; its callers (`legal_moves`, the env's masks, the properties' plans) see boarding and
  landing tiles among the destinations. The import maps seven more classes; the battle's
  battlefield reads the heroes' boats (ADR-0017 point 11). The env's observation encoding gains
  the boats and the in-boat flag (the AI lane's, ADR-0017 point 8). D-46 narrows increment 5's
  flight (ADV-MOV-009): a flight no longer crosses a water tile an object other than a boat or a
  boat's hero is visited from.
- Increment 9: the adventure state gains the owners, the weekly contents, the monsters'
  fractions and the objects added in play (state hash); the part gains six kinds and the
  week's, the month's and the Shipyard's parameters (part version); the import types seven
  classes and refuses three inputs (map import version). A day's start writes every growing
  monster and every weekly object once a week: a fork after it shares nothing of those tables
  until the next fork copies them again (the `day-start-xl-144` workload measures it).

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
- Increment 8 (ADV-Q32, ADV-Q34, ADV-Q35): boats as objects of the static map (they move) or as
  a plain array (a fork copies every record, and decision 164 allows a boat on every water tile,
  the XL map's 41 472); the original's 64 boats (`ADV-QUIRK-32`); a hero placed on water given a
  boat (VCMI; batch E8's E8-5 showed the original's hero on foot, D-44); VCMI's flyer or water
  walker taking a pickup or fighting a monster on water (D-45, reverted by E8-7) and its flight
  over an object on water (D-46: batch E8c showed the code's side).
- Increment 9 (ADV-Q36 to ADV-Q38): the owners as a per-tile layer (an XL map has at most a
  few hundred ownable objects, so a flag copying the block costs less than a grid's leaf, and
  the views read every owner); the month's monsters in a table of their own as the boats (a
  monster is the same kind wherever it came from; the boats' table exists because a boat
  moves); the weekly contents drawn in the original's tile walk (no draw is the original's
  anyway, ADR-0003's streams; object order is the setup's); a weekly object's contents once per
  player (VCMI's; decision 201); VCMI's neutral mine guards as one stack (D-47: batch E9 showed
  the split) and its whole-number growth (D-49: batch E9 showed the sixteenths).

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
- 2026-10-29 (E08 T70 `shared/`; the user at ADV-Q32 to ADV-Q35): points 1, 3, 4 and 6 amended
  for E08 increment 8, boats and the sea: boats as records with a tile layer (no original bound),
  a hero in a boat with its day's maximum, boarding and landing with the Admiral's Hat's
  conversion, the boat's passage, the free coastal tile, Favorable Winds and holes, the sea's
  four pickups visited only from a boat, no flight over an object's tile on water but a boat's, a
  hero placed on water refused; what a viewer sees of boats. Drafted in E08 T66b (survey
  `E08-sea.md`); batches E8 and E8c's outcome (T69, T69b: §15, §16) folded in.
- 2026-10-30 (E08 T81a `shared/`; the user at ADV-Q36 to ADV-Q38, E08 decisions 192–204):
  points 1, 3, 4, 5 and 6 amended for E08 increment 9, mines, the week and owned objects: the
  owners, a mine's guards, the weekly contents, the monsters' fractions and the objects added in
  play; Leave Guards; the kinds mine, abandoned mine, Lighthouse, Shipyard, Windmill, Water Wheel
  and Garden with flagging, the boat tile and the week's and month's steps; the flag's and the
  start's reveal; owners public, a mine's production its owner's. Drafted in E08 T77b (survey
  `E08-economy.md`); batch E9's outcome (T80, §19: D-47, D-49 and A-32 confirmed, A-33
  withdrawn) folded in.
- 2026-10-31 (E08 T83a; the user at ADV-Q39, E08 decision 214): point 4's weekly objects are
  entered, not visited from the neighbouring tile (the original's class lists and batch E9's
  E9-8); a wording correction, no rule changed elsewhere.
