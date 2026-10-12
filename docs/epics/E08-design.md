# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendments (E08 T11b; accepted at ADV-Q10, applied by #115). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`.

## State and observation (T14, T15)

T12 built the types, the contents drawn at the build, the removed objects with the occupancy
layer and `ObjectView` (type, anchor, masks; no index); T13 the treasuries, the pickups' visits
from a neighbour, `ObjectRemoved` (by type and anchor) and the game's `ResourceGained`.

- Per player: `bought: set of RevealTerrain types`, `keys: u8` (bit per key colour); the
  observation shows the viewer's own (bought, key held).
- Events (viewers resolved at emission): `OfferOpened`, `OfferRefused` (cannot pay),
  `OfferAnswered` (the owner), `TilesRevealed { count }` (the owner), `KeyGained` (the owner).

## Outcome and decisions (T14)

- `AdventureOutcome` (T13b: `gains`) gains `offer: Option<Offer>`;
  `Offer { hero, object, price: Vec<(ResourceId, u32)> }` (the kind follows from the object; the
  object named by its tile in what a player sees).
- `AdventureState::check_offer(&Offer) -> Result<(), _>` and `accept_offer(&Offer, sink)`
  (infallible): the reveal and the bought flag, or the guard's removal.
- `GameState`: `decisions: Vec<PendingDecision>` (owner, offer) in opening order beside the
  treasuries (T13a). `GameCommand::Answer(Answer::{Accept, Decline})`;
  `GameCommandError::{NoDecision, DecisionOpen, CannotPay}`.

## The env (T14)

Action encoding v2: entry 0 `EndTurn`, 1–8 hero slots, 9 `Accept`, 10 `Decline`; the tile head
as in v1 for the hero entries, ignored otherwise. With a decision open the entry mask is 9 and
10 only and the tile mask empty.

## Versions

| Version | From | To | Task |
|---|---|---|---|
| `MAP_IMPORT_VERSION` | 1 | 3 | T12 (v2), T13a (v3, the setup's difficulty; done) |
| adventure ruleset-part hash | v3 | v5 | T12 (v4), T13a (v5, the starting rows; done) |
| adventure state hash | v3 | v4 | T12 (done; T13 added no state); T14 bumps if it adds state |
| game state hash | v1 | v2 | T13a (done); T14 bumps again with the decisions |
| action encoding | v1 | v2 | T14 |
| adventure replay format | v1 | v2 if the answer needs it | T14 |
