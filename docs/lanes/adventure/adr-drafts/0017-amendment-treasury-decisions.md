# ADR-0017 amendment: the treasury, gains and costs, pending decisions, the env's answers

- Status: Proposed (E08 T11b, 2026-10-12; for the user at the T11a + T11b checkpoint)
- Amends: ADR-0017 points 1, 4, 6, 7 and 8 (sentences added or replaced), a new point 10
- Deciders: user + adventure lane (E08 decisions 14–17; survey `E08-objects.md` §7, §10)

<!-- Decisions only. Detail: docs/epics/E08-design.md, "Increment 2". On acceptance a shared/
     pull request applies the text below to ADR-0017 and adds the revision line. -->

## Context

Players get a treasury (decision 14: in `h3-game`, so towns spend from it without the
adventure). The Cartographer's offer is the first pending decision (decision 15); the survey
found a second question of the same shape, the border guard's (D-7). The adventure must open
an offer only when it can be paid, yet must not own or read the treasury.

## Decision

**Point 1 gains:** "`GameState` holds each player's treasury: an amount per resource kind of
the core ruleset (dense by `ResourceId`; kinds stay data, ADR-0006), never negative; a gain
saturates at 2³¹ − 1 (the original wraps to 0, `A-5`, a state-corrupting bug fixed without a
flag). The starting treasury is the row of the setup's difficulty (`GameSetup`), every player
the human row (decision 16); the rows are data compiled into `GameRuleset`."

**Point 10 (new): gains and costs cross by outcome.** A context's `apply` returns an outcome:
the resources its command gains for the actor and the offers it opens, each with its price.
The context never sees a treasury. The game adds the gains after the context applied (gains
cannot fail, so atomicity holds). An offer whose price the payer cannot pay is not opened (an
event says so to the payer); otherwise it becomes a pending decision. Accepting an offer: the
context checks it still stands, the game checks the price against the treasury, then pays and
calls the context's infallible acceptance; a rejected answer changes nothing. Towns (E09) use
the same outcome and the same check.

**Point 6's last sentence becomes:** "Increment 2 has two kinds, both offers answered yes or
no: the Cartographer's (a price, decision 15) and the border guard's passage (no price; ADV-Q10,
D-7). A player's open decisions form a queue (at most one in increment 2); `GameCommand::Answer`
answers the first (`Accept` or `Decline`, additive variants for later kinds), and a decline only
closes it. While one is open, its owner's legal commands are its answers only: no move, no
`EndTurn`. The system actor's default answer is `Decline`. Treasure chests wait for increment
3."

**Point 4 gains:** "Increment 2 resolves random resources, random pile amounts and campfires
from the setup stream (ADR-0016 point 4); no command of increment 2 draws, so the streams'
ownership isolation is not yet exercised by commands."

**Point 7 gains:** "Increment 2 adds to the inventory: an object two players can remove (a pile,
a border guard) and the per-player object state (purchases, keys), which only its owner's
commands write."

**Point 8 gains:** "Action encoding v2: the entry head gains the answers after `EndTurn` and the
eight hero slots (`Accept`, `Decline`; the tile head is ignored for them); while a decision is
open the mask allows its answers only. The observation (`h3_game`'s view) shows its owner the
treasury and the open decision (kind, price)."

**Point 9 gains:** "Increment 2 bumps the game state hash to v2 (treasuries, decisions), the
action encoding to v2 and, if the answer command needs it, the adventure replay format (T14)."

## Consequences

- `legal_commands`, `apply` and the env's mask share one pending-decision check
  (legality consistency, property-tested with decisions open).
- Conservation becomes: an army changes by no accepted command of increment 2; a treasury only
  by a visit's gain or an accepted offer's price.
- Observation hygiene covers treasuries and decisions: another player's are never shown.
- The env's action space grows by two entries; agents trained on v1 need retraining (none yet).

## Alternatives considered

- The game lends the treasury to the context's `apply` (`&mut`): simpler, but the adventure would
  own a game concept and towns would need the same loan; decision 14 puts the check in the game.
- The game checks a command's cost before passing it on: the game would have to know each
  object's prices; the outcome keeps that knowledge in the context.
- Opening every offer and refusing an unpayable acceptance: the original refuses at the visit
  (survey §4); a decision that can only be declined is noise for an agent.
- A separate decision head in the env: the answers fit the entry head's categorical choice; a
  head of their own comes with decisions that choose among many (level-ups).
- No question at the border guard (always pass): declining keeps a road closed to every other
  player, a real choice (D-7).

## Revision line (for ADR-0017)

- 2026-10-1x: points 1, 4, 6–9 extended and point 10 added for the treasury, gains and costs,
  the first pending decisions and the env's answers (E08 T11b; survey `E08-objects.md`).
