# ADR-0016 amendment draft: E08 increment 8, boats and the sea

- Status: Proposed (E08 T66b, 2026-10-28); accepted as written by the user at ADV-Q33 (E08
  decision 170), applied by T70 with batch E8's outcome
- Amends: ADR-0016 points 1, 3, 4 and 6, and its consequences
- Sources: survey `docs/rules/surveys/E08-sea.md` (T66a1 §0 to §6, T66a2 §7 to §14); the answers at
  ADV-Q32 (the epic's decisions 162–168); the working detail in `docs/epics/E08-design.md` parts B
  and C.

Applied by the increment's `shared/` task with batch E8's outcome (design §18): a dispute batch E8
reverts (decision 162) is rewritten there before the ADR is amended. *Agent's call* marks a choice
the user may change at the checkpoint.

## Point 1 (three kinds of storage): a fourth bullet after increment 7's block

> - **Increment 8's boats** (E08 decision 164: no original bound; the original's 64 records are
>   not reproduced): boats are no objects of the static map, because they move. Each boat is a
>   record (its type, which is appearance only; its facing; its owner and last captain, kept and
>   read by no rule until Summon Boat; whether a hero is aboard) in a persistent table by boat
>   index, and the tiles where an empty boat lies are a persistent per-tile layer (a bitset grid as
>   the fog's) that the movement rules and the reachable-tiles query read; a boat with a hero
>   aboard lies on its hero's tile, which the hero on the map holds. A fork costs two reference
>   counts; a boarding, a landing or a boat leaving the map writes one record and one leaf.

*Agent's call:* the table and the grid, not a plain array of boats: a plain array copies every
record at a fork, and decision 164 allows a boat on every water tile (the XL map's 41 472).

## Point 3 (heroes on the map): appended after increment 7's sentence

> From increment 8 a hero on the map may be **in a boat** (it holds its boat's index) and keeps
> its **day's maximum**, the points its turn started with on its surface. The turn start sets the
> day's points and the maximum by the hero's surface: on land the land points (ADR-0013 point
> 13.1), in a boat the sea points (ADR-0013 point 13.6). **Boarding** (a step from land onto an
> empty boat's tile) and **landing** (a step from a boat onto a free coastal tile) end the move;
> at the visit that follows the step the hero's maximum becomes the new surface's, computed then
> from its details and army, and its points become 0, or with free boarding (ADR-0013 point 13.7;
> the Admiral's Hat) the points left × the new maximum ÷ the old one, truncated, the points left
> being those after the step's own drop to 0 (D-41, the original's order). The original keeps the
> old maximum without the Hat (A-28, `ADV-QUIRK-31`): not reproduced, no flag (E08 decision 166).
> Boarding clears nothing the engine holds yet (the original clears the Fly and Water Walk
> spells' levels; the spells come later); a hero in a boat neither flies nor water-walks,
> whatever it wears. A hero that leaves the map takes its boat (the boat leaves with it).

## Point 4 (map objects): a sub-list after increment 7's kinds

> - **Increment 8's kinds and the sea** (the import maps classes 8, 29, 59, 82, 86, 124 and
>   225):
>   - **Boats** (class 8) become boats (point 1), never objects; the import takes the template's
>     subclass as the type, no owner, the original's facing 2, nobody aboard, and refuses two boats
>     on one tile, a boat off water, on a tile an object blocks or is visited from, or under a
>     hero. **Any hero boards any empty boat.** **A hero placed on water** is refused by the import
>     (`Unsupported`, D-44, E08 decision 163) until batch E8's row E8-5 shows what the original
>     does with it.
>   - **The sea's pickups** are visited from the neighbouring tile and leave the map at their
>     visit, without a question: **Flotsam** (29: nothing, 5 wood, 5 wood and 200 gold, or 10 wood
>     and 500 gold, a quarter each), the **Sea Chest** (82: nothing 20 %, 1 500 gold 70 %, 1 000
>     gold and a treasure artifact 10 %), the **Shipwreck Survivor** (86: a treasure, minor, major
>     or relic artifact at 55, 20, 20 and 5 %) are finds that leave, their contents drawn when the
>     game is built from the setup stream in object order (the random artifacts' pool of
>     increment 7); a hero that cannot receive the artifact gets the chest's 1 500 gold instead of
>     both its parts, and loses the survivor's (`ADV-QUIRK-22`'s test, survey §8.3, §8.4). The
>     **Ocean Bottle** (59) is a **message**: its visit tells the visitor's owner which object's
>     message it read (the map's text, or the original's default by the bottle's column, are the
>     client's: the engine holds no text) and it leaves; every visitor reads it, a computer
>     player's too (A-27, `ADV-QUIRK-30`, not reproduced, no flag; E08 decision 166).
>   - **Favorable Winds** (225) is a **ground of the sea**: it lies on its picture's tiles, is
>     never visited and is no battlefield; a boat's step from a tile it tops costs two thirds of
>     the step's price, truncated (66, 93 diagonally), as does the drop's cheapest step from such
>     a tile. **Holes** (124) lie on their picture's tiles, are never visited and only forbid a
>     landing there.
>   - **A hero's passage in a boat** (the fourth mode beside walking, flying and water walking):
>     water steps only, at water's price from the source tile (100, 141 diagonally, Favorable
>     Winds as above), with neither roads, nor native terrain, nor Pathfinding; a diagonal step
>     from water to water only when both side tiles are water (a diagonal landing needs no such
>     test: E08 T66b's reading of 0x4B2867 and 0x481CDB); never onto an empty boat's tile nor a blocked water
>     tile; onto land only by a **landing** onto a **free coastal tile** (the map's coast flag; no
>     object blocking it or visited from it, no event, no hole, no hero: D-42, the original's
>     class 3, with the wiki). The diagonal exception (ADV-MOV-005) compares the points left with
>     water's price, never the winds' (D-43). The drop to 0 after a step prices the cheapest step
>     from the new tile as the original's `cheapest_step_from` (100 at sea, 66 on Favorable Winds;
>     on the coast its road's price, else the native price).
>   - **From land onto water:** an empty boat's tile is a step for every mode from a stop node,
>     so from land (boarding: the move ends there; no guard attacks on it; a flight or a water walk
>     over water passes no boat it could board, the boat being on the original's list of classes
>     not entered, 0x63AA58); a hero in a boat is visited from a stop node (a battle or a
>     meeting); an object visited from the neighbouring tile on water is
>     visited by a **flying** hero from a stop node (D-45, the original's click; a flight still
>     passes it), never by a walking or water-walking one; plain water stays a flight's or a water
>     walk's pass. Disputes D-41 to D-43 and D-45 take the code's side until batch E8's rows
>     confirm or revert them (E08 decision 162).
>   - **At sea**, an object visited from the neighbouring tile is visited from the boat's water
>     tile, an entered one is entered; another hero in a boat is met or attacked from the
>     neighbouring water tile; a boat never reaches a hero on land; a step onto a guarded water
>     tile is attacked at once; an event on water fires for a boat's step and ends the move there.
>     A landing's guards attack after the landing, on the shore.
>   - **The reachable-tiles query** searches a hero in a boat over one node per tile (every water
>     tile a stop; the landing a leaf from which no path goes on, as boarding is from land); a
>     walking hero's search tests a boat only on a water neighbour, so its cost stays
>     (`bench-check`). Today's reach only: the multi-day path's next day at the sea's maximum is
>     not the engine's.

## Point 6 (information hiding): appended

> From increment 8 a viewer sees on its explored tiles every **empty boat**: its tile, type and
> facing, never its owner or captain; of every hero it sees whether it is in a boat (its boat's
> type: the original draws the hero's flag on it); an own hero's day's maximum is its owner's. The
> sea's pickups look alike to every viewer until a visit, as the chests; their contents stay
> hidden. A boarding, a landing and a boat leaving with its hero are seen where their tiles are;
> a bottle's message is told to its visitor's owner alone.

## Consequences: appended

> - Increment 8: the adventure state grows by the boats (a table and a grid), the hero's boat and
>   its day's maximum (the state hash and its version); `fork-*` moves by two reference counts.
>   The reachable-tiles query gains a search for a hero in a boat and the boat rows in the
>   others; its callers (`legal_moves`, the env's masks, the properties' plans) see boarding and
>   landing tiles among the destinations. The import maps seven more classes; the battle's
>   battlefield reads the heroes' boats (ADR-0017 point 11). The env's observation encoding gains
>   the boats and the in-boat flag (the AI lane's, ADR-0017 point 8).

## Revision history line

> - 2026-10-?? (E08 T?? `shared/`; the user at ADV-Q32 and ADV-Q33): points 1, 3, 4 and 6 amended
>   for E08 increment 8, boats and the sea: boats as records with a tile layer (no original bound),
>   a hero in a boat with its day's maximum, boarding and landing with the Admiral's Hat's
>   conversion, the boat's passage, the free coastal tile, Favorable Winds and holes, the sea's
>   four pickups, a flyer's visit from the shore, a hero placed on water refused; what a viewer
>   sees of boats. Drafted in E08 T66b (survey `E08-sea.md`), batch E8's outcome folded in.
