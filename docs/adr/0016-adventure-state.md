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
   infallible application (ADR-0017 point 11).
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
     a battle at once; an own hero's tile still blocks (meetings and the army exchange come with
     increment 4's army management); a hero's own tile is always leavable.
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
     neither, the join offer is skipped as declined (a feature to-do, decision 22). **The
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
5. **Fog of war.** Each player has an explored layer (a bitset grid). An explored tile stays
   visible with its current content until a rule clears it (Cover of Darkness): no line-of-sight
   memory (VCMI, survey `E08-adventure-core.md` §2). Reveals set disk-shaped areas;
   clearing is supported. The **fringe** is the hidden tiles touching an explored tile (the
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
   contents.

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
