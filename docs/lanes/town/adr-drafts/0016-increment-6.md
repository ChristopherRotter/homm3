# Draft: ADR-0016 amended for E09 increment 6 (the town's trades)

- Status: **Draft**, written by E09 T53 (2026-11-25), **accepted** at its checkpoint (`TWN-Q37`,
  decisions 214 to 217, each as recommended) before anything is built; then moved into `docs/adr/0016-adventure-state.md` by the increment's
  `shared/` task (T55a) with batch TWN-6's verdicts.
- Decisions: the epic's 206 (TD-31: the spot the original's), 207 (T-21 by ADV-SHY-003's rule,
  T-24, T-27), 209 and 210 (a town as the market's and the University's place), 212 (the stock and
  the pool's marks in the adventure state), 213 (T53's agent's calls), 214 to 217 (`TWN-Q37`).
- Working detail: survey `docs/rules/surveys/E09-trades.md` §2, §3, §5, §6; design note
  `docs/epics/E09-design.md` Part H §104, §107, §108, §111, §112, §114.

## Point 1, after the storage list's last item ("- **Increment 15's objects** (E08 decision 394): **no new block.** …")

- **E09's increment 6** (TWN-STK-001, -002; E09 decision 212): the adventure state holds **the
  merchants' stock** (one entry per class of the town part's `merchants_stock`, each an artifact or
  empty; at most `MAX_STOCK`, 8) and **the artifact pool's marks** (ADV-OBJ-006's bans and given
  marks by artifact, two bits each), kept from the build instead of dropped with
  `resolve_rewards`' local pool; both in the adventure state hash, both shared until written (the
  marks change only at the build and at each month's draw, the stock also at a purchase). The town
  lane's code lives in `h3-adventure`'s `src/town*`; the fields and the hash are the adventure
  lane's (`cross-lane`). A loaded game keeps them (the state is the save).

## Point 4, E09's kind `Town`, after increment 5's item ("- **E09's increment 5** (TWN-GLD-005; …", ending "… a turn's start never).")

- **E09's increment 6** (the town's trades; E09 decisions 209, 210, 212; `cross-lane`, the town
  lane's changes in the adventure lane's paths) adds no visit rule: every trade is a town command
  (ADR-0017 point 1). A town is **a place** of two of E08's visits: `MarketVisit` (a map object as
  today or the town by `TownId`; its hero none possible at a town) and `UniversityVisit` (the same
  place, the hero, the skills and the price as the game names them; no mark at a town). **The
  market's checks** (`markets.rs`) take the window as the game resolves it, the screens, the count
  and the stock: from the map object as today (its kind's screens, the count 5, its own stock and
  sold bits) or from the town (ADR-0028 point 2's screens and count, the merchants' stock of point
  1); every refusal of E08 stands (`Screen` for a trade the town does not offer), and the map's
  markets keep every answer and view byte for byte (their replays are the oracle). **The sale's
  rate** (ADV-MKT-002 amended): `MarketParams::artifact_sale(count, cost, take)` beside the other
  three rates (ADR-0017 point 10); `MarketOffers` gains the sale's places with their prices. **The
  pool kept** (ADV-OBJ-006 amended): the picker reads and writes point 1's marks; in this increment
  only the merchants' draws read them after the build. **The town's boat** (TWN-SHY-001;
  TWN-SET-011 amended): the adventure gives the town's boat spot (E09's increment 2's, fixed at the
  build; TD-31 the original; point 1's "increment 5's town boat" is this one) and refuses it while
  an empty boat or a hero stands on it (`BoatSpotTaken`; T-21 fixed by ADV-SHY-003's rule), a
  missing spot defensive (a standing Shipyard always has one); E08's `launch` puts an empty boat of
  the town part's type there, its builder the actor. The docked ship (building 20) is the spot's
  occupancy, derived from the boats and heroes there, never stored (T-27).

## Point 6, after the point's last paragraph ("From increment 15 a viewer that sees a prison …", ending "… lists it (E08 T140).")

**From E09's increment 6** (TWN-VEW-005; Part H §101 call 6) **the merchants' stock** is seen only
in its window: the market view (GAM-DEC-011's `MarketView`) of a town's window shows it with its
prices to the window's owner alone, as a Black Market's, beside the artifact sale's places; no event
tells anyone else that a purchase emptied an entry. The boat at a town's spot is the map's view (fog
as ever). `docs/06-security.md` lists it (E09 T55a: the stock's projection, the town commands'
validation lines).

## Revision history, a new item (when moved)

- 2026-11-xx (E09 increment 6's `shared/` task T55a): points 1, 4 and 6 (the merchants' stock and
  the pool's marks in the state; a town as the market's and the University's place, the checks with
  a resolved window, the sale's rate, the pool kept, the town's boat; the stock seen only in its
  window).
