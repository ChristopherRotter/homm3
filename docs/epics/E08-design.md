# E08 design note: increment 2

A handover, not binding (`docs/adr/README.md`): the working detail behind the ADR-0016 and
ADR-0017 amendment drafts (`docs/lanes/adventure/adr-drafts/`, E08 T11b). The task that builds a
section deletes it; the file goes when the epic closes. Facts and sources: survey
`docs/rules/surveys/E08-objects.md`.

## Object types (T12)

`adventure.toml` gains `[[object_type]]` entries: key, kind, the kind's parameters, `visit =
"enter" | "neighbour"`, `from_above = true | false`, sources. The import maps (class, subclass)
to a key:

| Classes | Type keys (sketch) | Kind and type parameters | Visit, from above |
|---|---|---|---|
| 79 / 0–6 | `base:pile_wood` … `base:pile_gold` | `Pickup`: the resource, a random amount range (5–10 or 3–6, D-5), a multiplier (gold 100) | neighbour, yes |
| 76 | `base:random_resource` | `Pickup` drawn into a pile type at setup (seven equal chances) | neighbour, yes |
| 12 | `base:campfire` | `Pickup`: amount 4–6 of one of six resources plus 100 × amount gold | neighbour, yes |
| 13 / 0–2 | `base:cartographer_water`, `_land`, `_subterranean` | `RevealTerrain`: terrains (water; dirt, sand, grass, snow, swamp, rough, lava; subterranean), price 1 000 gold | enter, no |
| 58, 60 | `base:redwood_observatory`, `base:pillar_of_fire` | `RevealArea`: radius 20 | enter, no |
| 10 / 0–7 | `base:keymaster_<colour>` | `KeyTent`: key 0–7 | enter, no |
| 9 / 0–7 | `base:border_guard_<colour>` | `KeyGuard`: key 0–7 | neighbour, yes |
| 212 / 0–7 | `ab:border_gate_<colour>` | `KeyGate`: key 0–7 | neighbour without the key, passable with it; yes |

Instance data: a pile's fixed amount (≤ 0x7FFFF, survey §1), its guards (ADV-Q10, D-6).
Resolved at the game's build in object order from the setup stream: random resources first,
then random amounts and campfires (the original's two walks, survey §3.2). Pickup contents go
into the static map as resolved `(ResourceId, amount)` lists.

## State and observation (T12, T13, T15)

- Removed objects: a bit per object in the persistent object table; an occupancy grid (blocked,
  visitable, removed objects cleared) read by `MoveHero` and the query.
- Per player: `bought: set of RevealTerrain types`, `keys: u8` (bit per key colour).
- Observation: the tiles' live occupancy; `objects: Vec<ObjectView>` (id, type, anchor, and for
  the viewer's own per-player state: bought, key held), sorted by id; no amounts.
- Events (viewers resolved at emission): `ObjectRemoved { object }` (players that explored a
  tile it covered), `ResourcesGained` (game event, the owner), `OfferOpened`, `OfferRefused`
  (cannot pay), `OfferAnswered` (the owner), `TilesRevealed { count }` (the owner), `KeyGained`
  (the owner).

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
| `MAP_IMPORT_VERSION` | 1 | 2 | T12 |
| adventure ruleset-part hash | v3 | v4 | T12 |
| adventure state hash | v3 | v4 | T12, T13 |
| game state hash | v1 | v2 | T13 |
| action encoding | v1 | v2 | T14 |
| adventure replay format | v1 | v2 if the answer needs it | T14 |
