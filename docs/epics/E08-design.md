# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendments (E08 T11b; accepted at ADV-Q10, applied by #115). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`.

## State and observation (T13, T15)

T12 built the types, the contents drawn at the build, the removed objects with the occupancy
layer (`AdventureState::remove_object`) and `ObjectView` (type, anchor, masks; no index).

- Per player: `bought: set of RevealTerrain types`, `keys: u8` (bit per key colour); the
  observation shows the viewer's own (bought, key held).
- Events (viewers resolved at emission): `ObjectRemoved { object }` (players that explored a
  tile it covered; the object by its tile, not its index), `ResourcesGained` (game event, the
  owner), `OfferOpened`, `OfferRefused` (cannot pay), `OfferAnswered` (the owner),
  `TilesRevealed { count }` (the owner), `KeyGained` (the owner).

## Outcome and decisions (T13, T14)

- `AdventureState::apply(..) -> Result<AdventureOutcome, AdventureCommandError>`;
  `AdventureOutcome { gains: Vec<(ResourceId, u32)>, offer: Option<Offer> }`;
  `Offer { hero, object, price: Vec<(ResourceId, u32)> }` (the kind follows from the object).
- `AdventureState::check_offer(&Offer) -> Result<(), _>` and `accept_offer(&Offer, sink)`
  (infallible): the reveal and the bought flag, or the guard's removal.
- `GameState`: `treasuries: Vec<Treasury>` by player; `decisions: Vec<PendingDecision>`
  (owner, offer) in opening order. `GameCommand::Answer(Answer::{Accept, Decline})`;
  `GameCommandError::{NoDecision, DecisionOpen, CannotPay}`.
- Starting rows: `[[difficulty]]` in `adventure.toml` (the human and the computer row each,
  sources), compiled into `GameRuleset`; `GameSetup.difficulty` (default 1, normal).

## The env (T14)

Action encoding v2: entry 0 `EndTurn`, 1–8 hero slots, 9 `Accept`, 10 `Decline`; the tile head
as in v1 for the hero entries, ignored otherwise. With a decision open the entry mask is 9 and
10 only and the tile mask empty.

## Versions

| Version | From | To | Task |
|---|---|---|---|
| `MAP_IMPORT_VERSION` | 1 | 2 | T12 (done) |
| adventure ruleset-part hash | v3 | v4 | T12 (done) |
| adventure state hash | v3 | v4 | T12 (done); T13 bumps again if it adds state |
| game state hash | v1 | v2 | T13 |
| action encoding | v1 | v2 | T14 |
| adventure replay format | v1 | v2 if the answer needs it | T14 |
