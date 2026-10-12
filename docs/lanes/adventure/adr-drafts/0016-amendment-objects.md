# ADR-0016 amendment: object types, visits, per-object state

- Status: Proposed (E08 T11b, 2026-10-12; for the user at the T11a + T11b checkpoint)
- Amends: ADR-0016 point 4 (rewritten), point 1 (one sentence), point 6 (one sentence)
- Deciders: user + adventure lane (E08 decisions 13 and 17; survey `E08-objects.md`)

<!-- Decisions only. Detail: docs/epics/E08-design.md, "Increment 2". On acceptance a shared/
     pull request applies the text below to ADR-0016 and adds the revision line. -->

## Context

Increment 2 types the objects of E08 decision 13. The survey (`E08-objects.md` §2) found that
the original decides per class whether an object is entered or visited from a neighbouring
tile, whether it may be entered from above, and that a visit ends the move; VCMI agrees on all
three for these classes. Random contents are drawn at setup (§3.4). Pickups and border guards
leave the map; Cartographer purchases and keys are per player (§4, §6).

## Decision

**Point 4 becomes:**

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
   - A tile holding a hero still blocks other heroes (meetings and battles come later), and a
     hero's own tile is always leavable.
   - **Increment 2's kinds** (decision 13): `Blocker`; `Pickup` (resources gained, then the
     object leaves: piles, random resources, campfires); `RevealTerrain` (a price, a set of
     terrains on every level, once per player and type: the Cartographers); `RevealArea` (a
     radius around the visited tile, every visit: the Redwood Observatory, the Pillar of
     Fire); `KeyTent`, `KeyGuard`, `KeyGate` (a key colour: the tent gives the key, the guard
     leaves the map for everyone when a key holder accepts, the gate lets key holders pass).

**Point 1, per-object state, gains:** "Per-object state of increment 2 is whether the object is
still on the map; per-player state of object types (a Cartographer kind bought, the keys held)
is a plain array per player. A removal clears the object's tiles in a persistent occupancy
layer (a grid as the fog's) that the movement rules read; the static map keeps every footprint."

**Point 6, information hiding, gains:** "An object is shown whole (its type, its tiles, the
viewer's own state of its type) once the viewer has explored any tile it covers (VCMI's
`isVisibleFor`), or on the fringe under `ADV-QUIRK-1`; tiles show live occupancy. Contents
(amounts, a campfire's resource) and other players' purchases and keys are never shown. An
object's removal is seen by the players that have explored a tile it covered."

## Consequences

- Versions: `MAP_IMPORT_VERSION` 2, the adventure ruleset-part hash (object types), the
  adventure state hash v4 (removed objects, per-player object state, the resolved contents).
- `MoveHero` and the reachable-tiles query learn the visit modes, the "from above" rule and
  the occupancy layer (T13); the env's destination action is unchanged: a visit target's tile
  means "visit it".
- A guarded pile needs a battle (increment 3): until then it is an `Unsupported` import error or
  a `Blocker` (ADV-Q10, D-6).
- Two players' commands can now write the same state (the same pile, guard): the epic file's
  inventory for simultaneous play (ADR-0017 point 7) lists it.

## Alternatives considered

- Visit modes as code per kind: the original keeps them per class as data (its settings
  table); a type-level property keeps a modded object's mode in data.
- Drawing contents at the first visit (lazy): the original and VCMI draw at setup; a visit
  draw would make pickups advance players' streams for no rule.
- Removing an object by rewriting the static map: a fork would copy the map-sized block.
- Objects shown tile by tile only: an agent then reassembles objects from tiles; the list costs
  little and matches VCMI.

## Revision line (for ADR-0016)

- 2026-10-1x: point 4 rewritten, points 1 and 6 extended for increment 2's object types,
  visits and per-object state (E08 T11b; survey `E08-objects.md`).
