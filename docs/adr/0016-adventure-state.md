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
   - **Plain arrays** for the small, hot parts: heroes on the map, their armies, movement points.

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
   tile and level, army, movement points. The hero details stay in `h3-heroes`, kept by
   `h3-game`; what movement needs from them arrives through the adventure-profile seam (E08
   decision 5, increment 3).
4. **Map objects are closed behaviour kinds with data parameters** (as abilities, ADR-0012;
   answers the `docs/03` question). An object is its static part (position, template masks of
   blocked and visitable tiles, object type), a kind from a closed enum in `h3-adventure` with
   typed parameters, and per-kind state in the persistent object table. The import maps the
   original's class and subclass numbers to object-type keys; a type's kind and type-level
   parameters come from the adventure ruleset part (data), instance-level ones from the map,
   validated at setup. A rule with a rule id reads the kind; a new behaviour is a new variant
   plus its rule. No scripting, and no code keyed by object names or class numbers outside the
   import's mapping. A blocked tile cannot be entered. A visitable tile is visited by kind
   either by entering it, which ends the move, or from a neighbouring tile without entering
   (in VCMI, for example, resources, artifacts and monsters); increment 2's survey lists the
   modes. **Increment 1 has one kind, `Blocker`, and no object types:** every
   object blocks its blocked *and* its visitable tiles, a tile holding a hero blocks too
   (meetings and battles come later), and a hero's own tile is always leavable.
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
   hidden. The observation is materialised per viewer as flat planes; the env's encoding (T9)
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
  that knows the original's class numbers.

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

## Revision history

- 2026-10-10: Accepted (E08 T1a draft; the T1b survey's findings on fog folded into point 5).
- 2026-10-11: point 1: the leaf size, measured against a flat per-player copy (E08 T5b,
  `02-architecture.md` §8): 32 × 32 tiles (the user). Full leaves share one leaf held by the
  grid, empty leaves are none.
