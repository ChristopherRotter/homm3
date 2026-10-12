# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendments (E08 T11b; accepted at ADV-Q10, applied by #115). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`.

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

## The env (T14b, second half)

Action encoding v2: entry 0 `EndTurn`, 1–8 hero slots, 9 `Accept`, 10 `Decline`; the tile head
as in v1 for the hero entries, ignored otherwise. With a decision open the entry mask is 9 and
10 only and the tile mask empty (v1's masks are then empty: `legal_commands` lists the two
answers, `legal_moves` gives nothing). The open decision: `GameState::decision(player)` or the
observation's `decision`; the commands `GameCommand::Answer(Answer::Accept | Answer::Decline)`.
The replay's text commands `<player> accept` and `<player> decline` exist (`h3-cli`); the
format's version and the difficulty option come with the env.

## Versions

| Version | From | To | Task |
|---|---|---|---|
| `MAP_IMPORT_VERSION` | 1 | 3 | T12 (v2), T13a (v3, the setup's difficulty; done) |
| adventure ruleset-part hash | v3 | v5 | T12 (v4), T13a (v5, the starting rows; done) |
| adventure state hash | v3 | v5 | T12 (v4), T14b (v5, the purchases; done) |
| game state hash | v1 | v3 | T13a (v2), T14b (v3, the decisions; done) |
| action encoding | v1 | v2 | T14b, second half |
| adventure replay format | v1 | v2 (the difficulty option) | T14b, second half |
