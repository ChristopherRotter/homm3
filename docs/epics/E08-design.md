# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendments (E08 T11b; accepted at ADV-Q10, applied by #115). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`. The versions increment 2 bumped through T14 are listed in
`E08-log.md` (T14b's second half).

## State and observation (T15)

T12 built the types, the contents drawn at the build, the removed objects with the occupancy
layer and `ObjectView` (type, anchor, masks; no index); T13 the treasuries, the pickups' visits
from a neighbour, `ObjectRemoved` (by type and anchor) and the game's `ResourceGained`; T14b the
Cartographer's purchases (`AdventureState::bought`, a byte of player bits per type), the offer as
a pending decision (`h3_game::GameState::decision`, `GameCommand::Answer`), the events
`OfferOpened`, `OfferRefused`, `OfferAnswered`, `ResourcePaid` and `TilesRevealed { object_type,
count }` (the owner alone), and the viewer's purchases and open decision in the observation.

- Per player: `keys: u8` (bit per key colour); the observation shows the viewer's own.
- Events (viewers resolved at emission): `KeyGained` (the owner); an observatory's reveal as
  `TilesRevealed` (the owner).
- The guard's offer (D-7) is a second kind of `Offer` (no price): `accept_offer` removes the guard
  for everyone, and the check that the offer still stands (another player's key holder may have
  removed it) becomes reachable once two players act in one group.
