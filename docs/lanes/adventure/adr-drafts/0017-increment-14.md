# ADR-0017 amendment (draft): increment 14, the trades and services

- Status: Draft (E08 T127b, 2026-11-18), for the checkpoint at T127b's end (ADV-Q58); applied to
  ADR-0017 by increment 14's `shared/` task (T131) with batch E14's outcome (T130), then deleted.
- Amends: ADR-0017 point 1 (no command), point 6 (three decision kinds; the factory's question and
  dialog; the Altar's return after its levels), point 8 (the env's entries), point 9 (versions),
  point 10 (the trades' gains and costs, a transfer to another player); the Consequences, the
  Alternatives and the revision history.
- Detail: increment 14's design note `docs/epics/E08-design.md` §13 to §19; survey
  `docs/rules/surveys/E08-trades.md`; E08 decisions 361 to 369 and the checkpoint's.

## Point 1 (after "Increment 13 (creature banks …) adds no command")

   **Increment 14** (the trades and services; E08 decision 361) adds **no command**: the seven
   classes are visited by `MoveHero`'s last step onto their entrance and by `Revisit`; their
   windows are pending decisions answered with `Answer`'s new kinds (point 6), the War Machine
   Factory's question with the offers' answers and its dialog with the hire dialog's.

## Point 6 (after increment 13's decisions)

   Increment 14's decisions: three new kinds, each its owner's alone, holding no one else and
   blocking its owner's `EndTurn` like an offer, open until its `Leave` (every other answer keeps it
   open, as the original's windows stay open for more trades), the system actor's default
   `Leave` (nothing is lost: a sacrifice names everything it takes in one answer, A-48 fixed,
   E08 decision 366):
   - **A market** (GAM-DEC-011), opened by a Trading Post's, a Freelancer's Guild's or a Black
     Market's visit, answered **`Market(MarketAnswer)`**: `Trade { give, take, lots }` (whole lots
     at the rate, from 1 to what the treasury holds), `Transfer { resource, amount, to }` (1 : 1 to
     another player still in the game, allies and enemies alike), `Sell { slot, take, lots }` (the
     army keeping one creature), `Buy { entry, pay }` (an unsold entry at its price; refused
     before paying when the hero cannot receive the artifact, A-46 fixed), `Leave`; an answer of a
     screen the object lacks is refused. Its view shows the screens with their rates, the stock
     and the recipients.
   - **An Altar of Sacrifice** (GAM-DEC-012), answered **`Altar(AltarAnswer)`**: `Artifacts(places)`
     (at most 23, none kept), `Creatures(amounts by slot)` (one creature kept), each with a total
     above 0 and only where the hero's class's faction allows it, and `Leave`. The total is gained
     as experience with its levels: levels without a decision at once, the Altar staying open;
     when a level-up opens, **the Altar closes before it and its visit is the visit's rest**
     (GAM-DEC-007, `Rest::Visit`): once the level-ups are answered the Altar opens again and a
     later sacrifice reads the new level (the original's dialog beneath its level-up dialogs). A
     player keeps one open decision. Its view shows each artifact's and each stack's experience
     and the experience to the next level.
   - **A Hill Fort** (GAM-DEC-013), answered **`HillFort(HillFortAnswer)`**: `Upgrade(slot)` a row
     whose price the treasury holds, `All` when some row upgrades and the treasury holds every
     row's price (each paid and upgraded in slot order), `Leave`. Its view shows each row's
     upgrade, price and state and the whole's.
   - **The War Machine Factory's question** is the hire question (`Offer(Consent)`, [`Decline`]);
     accepted, **the hire dialog** (GAM-DEC-008 amended) with the factory as its source: each
     machine with a count of 1 while the hero lacks its artifact, else 0; `Buy { creature, amount:
     1 }` pays the creature's cost and gives the artifact into its position (no army change), the
     dialog of three staying open; `Close`.
   `legal_commands` lists a representative answer per action, as the hire dialog lists each
   creature's most: a market's trade of one lot per pair it can pay, its sale of one lot for gold
   per slot, its purchase in gold per unsold entry it can pay; the Altar's two "all" sacrifices
   (the original's buttons); the Hill Fort's upgrade per affordable row and its `All`; each
   `Leave`. No transfer is listed (as the management commands, E08 decision 99). Any other answer
   the view allows is still legal for `apply`.

## Point 8 (after increment 11's action encoding v8 and E09's v9 to v12)

   Increment 14's **action encoding** (the next version at merge: v13 after E09's v12) appends 67
   entries, each legal where `legal_commands` lists its answer, tile 0 alone: a market's trade of
   one lot per pair (42, given then wanted in the core's order), its sale of one lot for gold per
   slot (7), its purchase in gold per entry (7); the Altar's two "all" sacrifices (2); the Hill
   Fort's upgrade per slot (7) and `All` (1); one `Leave` for the three kinds. The factory uses the
   offer's and the hire dialog's entries. An amount head stays the AI lane's.

## Point 9 (after increment 13's versions)

   **Increment 14** bumps the adventure ruleset part (v22: the five kinds, the market
   parameters), the map import (v26: the seven classes, the RoE Hill Fort's mark), the adventure
   state hash (v27: the stock's draw and sold entries), the game state hash (v34: the three
   decisions, the rest's name), the adventure replay format (v19: the answers' words) and the env's
   action encoding (v13, the AI lane's), and `h3-protocol`'s protocol once per wire task (the
   server lane's). The core ruleset hash and the pack format stay. The numbers are the next at
   merge, in merge order with E09's increment 4 (its T39a plans map import v26, replay format v19,
   protocol v73 and env v13).

## Point 10 (after increment 11's outcomes and E09's)

   **Increment 14's outcomes:** a visit names the window to open (the market, the Altar, the Hill
   Fort) or the factory's question. **A trade:** the adventure computes the rate (the lot) from the
   part's parameters; the game checks the lots against the treasury, pays the given resource and
   adds the wanted one (saturating). **A transfer** is the first cost and gain of two players: the
   actor pays, the recipient gains 1 : 1, each told its own change; nothing else of it is seen.
   **A sale:** the adventure checks the slot, the lot and the army's last creature and names the
   gain; the game adds it after the adventure's infallible change. **A purchase** (a Black
   Market's, a factory's): the adventure checks the entry or the machine and the hero's room and
   names the price; the game checks the treasury, pays, then calls the adventure's infallible
   change. **A sacrifice:** the adventure checks the places or amounts and names the total; the
   game applies the changes and gives the experience with its levels. **An upgrade:** the
   adventure names each row's price; the game checks the treasury, pays, then calls the upgrade.
   A rejection leaves everything untouched. Every rate, price and most is the state's answer,
   never the command's.

## Consequences (append)

- Increment 14 (E08 T127b): three decision kinds with typed answers and views carrying the rates
  (so neither a client nor the env needs a copy of the computation); the Altar's levels reuse the
  visit's rest; the factory reuses the hire question and dialog with a source of war machines;
  the first gain of a player other than the actor (a transfer); `h3-protocol` maps the new
  answers, views, events and errors at each wire task (`cross-lane`), and the env appends 67
  entries once every answer exists.

## Alternatives considered (append)

- **Increment 14's windows as commands valid while the hero stands on the entrance** (no
  decision): not chosen; every trade would re-prove the object and nothing would end the window.
  **The Altar's levels held until it is left:** not chosen; a Learning specialist's later
  sacrifices would read the old level. **The level-ups queued in front of the open Altar:** not
  chosen; two open decisions of one player. **An event naming a transfer's giver to its
  recipient:** not chosen; the original's hot seat shows the treasury only. **Every trade amount
  in `legal_commands`:** not chosen; thousands per pair, as the hire dialog's amounts.

## Revision history (append)

- 2026-11-?? (E08 T131's `shared/` part; drafted in E08 T127b, the checkpoints ADV-Q56 to
  ADV-Q58): point 1 (increment 14 adds no command), point 6 (the market, the Altar and the Hill
  Fort as decisions; the factory's question and hire dialog; the Altar's return after its
  level-ups as the visit's rest; the listed answers), point 8 (the env's 67 entries), point 9
  (the versions), point 10 (the trades' outcomes; a transfer to another player). Batch E14's
  outcome (T130) folded in.
