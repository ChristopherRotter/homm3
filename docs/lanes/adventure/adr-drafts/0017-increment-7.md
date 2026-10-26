# ADR-0017 amendment (draft): the rewards' decisions, the bans, the battle's setup facts

- Status: Proposed (E08 T55b, 2026-10-26), from T55a's survey and the answers at ADV-Q28 (the
  epic's decisions 129–136), with T54's facts (decision 124). **Accepted by the user** (ADV-Q29,
  2026-10-26; decisions 138–141), with the answers typed by decision (decision 138: point 6
  rewritten). The round-3 `shared/` task (T59) applies it to ADR-0017.
- Amends: ADR-0017 points 1, 4, 6, 8, 9, 10 and 11.
- Detail: `docs/epics/E08-design.md` part C (§14–§19); facts: survey
  `docs/rules/surveys/E08-rewards.md` §8.1, §8.6; T54's record in `docs/epics/E08-log.md`.

## Context

The engine has no blocking prompts (point 6): every question a reward asks is a pending decision.
Increment 7 adds yes-or-no questions, two-way choices, a dialog of purchases (the University) and
an army dialog over several leftover stacks after which the visit goes on (a box's experience,
D-39). Today one `Answer` (`Accept`, `Decline`, `First`, `Second`) serves every decision; the user
(ADV-Q29): "there should be specific types tailored to the choice to be made". Its draws need the scenario's spell and artifact bans. T54 (decision 124) filled the
battle seam's facts from the game; point 11 does not name them yet.

## Decision

1. *(amended, the setup)* `GameSetup` holds each player's **human flag** (T54: a map's import sets
   it from the map's player kinds, a colour a human may play is human; a session may change it
   before the game is built; an env's agent counts as whatever the setup says), and beside the
   banned skills the scenario's **banned spells and artifacts** (static, from the map), which the
   game passes to the adventure's build.
4. *(amended, randomness)* Increment 7's random contents are setup placeholders drawn from the
   setup stream when the state is built (ADR-0016 point 4); no visit of increment 7 draws. The
   Pyramid's monster battle splits its guards as a monster's (`ADV-QUIRK-6`: the tile under
   `faithful`, the attacking player's stream under `strict-fixed`).
6. *(amended, decisions and their answers)* **An answer is typed by its decision** (decision
   138): `GameCommand::Answer` answers the actor's first open decision with an `Answer` whose
   variant names the kind of decision and carries that kind's own type; an answer of another kind
   is rejected (`WrongAnswer`) and changes nothing. Every decision's answers, increments 2 to 6's
   migrated with increment 7's (in T63, one replay format bump), the system actor's default in
   brackets:
   - **`Offer(Consent)`** (`Accept`, `Decline`) [`Decline`]: every yes-or-no offer, each the same
     question with an optional price (point 10): the Cartographer's, the border guard's, a
     monster's join, join for gold and pursuit, a guarded object's fight (a guarded artifact
     always, a guarded scroll with a text), and increment 7's: open a Pandora's Box, search a
     Pyramid (a looted one too) or a Warrior's Tomb, pay a Tree of Knowledge (its drawn price, not
     opened when the owner cannot pay).
   - **`LevelUp(SkillOffer)`** (`First`, `Second`: the original's left and right offer) [the
     original's choice, ADR-0013 point 12].
   - **`Chest(ChestChoice)`** (`Gold`, `Experience`) [`Gold`, the original's plain OK]; the view
     shows both amounts.
   - **`Arena(ArenaChoice)`** (`Attack`, `Defense`; no leaving) [`Attack`, the original's Cancel].
   - **`School(SchoolChoice)`** (the first or the second of the School's two skills, or `Leave`
     without paying) [`Leave`]; opened only when the owner can pay its price, which the view shows.
   - **`University(UniversityAnswer)`** (`Buy(slot)`, `Leave`) [`Leave`]: the University is a
     decision like a meeting, open until `Leave`; each `Buy` buys an open slot's skill for the
     type's price (checked against the slot's state and the treasury) and keeps it open. Its view
     shows the four skills and each slot's state.
   - **`Close`** [`Close`]: ends a meeting or an army dialog (increment 6's `Decline` for them).
     The army dialog also opens over **up to seven leftover stacks** of a box's or an event's
     creatures, with the same management commands; the visit's rest (the experience, then its
     level-ups) comes after it closes.
   - **The visit's rest after a won guards' battle** (a pickup, the rewards, the Pyramid's spell)
     is held by the game and given once the winner's level-ups are answered (at once when there
     are none), as the original gives the levels first (decision 141).
   Each of increment 7's questions is its owner's alone, holds no one else and blocks its owner's
   `EndTurn` like an offer.
8. *(amended, the env)* The new decisions reach agents through `legal_commands`; the env's
   action encoding (the AI lane's) maps its entries onto the typed answers (its `Accept`,
   `Decline`, first and second offer entries onto `Offer`, `LevelUp`, `Close` and the new kinds'
   two-way choices), and its masks must allow at least one answer of each decision, so a rollout
   never stalls (the system actor's defaults above). Whether a University's purchase gets a head of
   the encoding is the AI lane's call (its inbox); the mapping's change in `h3-sim` is T63's,
   `cross-lane` with the user's consent (decision 138).
9. *(amended, versions)* Increment 7: the core ruleset hash and the pack format (shared: the mana
   ceiling, the quirk flags), the adventure ruleset part (the kinds), the map import (the classes,
   their bodies, the bans), the adventure state hash (the visits' block, the army left), the game
   state hash (the decisions, the details' bonuses), the adventure replay format v7 (the typed
   answers); the action encoding's version is the AI lane's.
10. *(amended, outcomes)* A visit's outcome may also name: the reward's hero changes (ADR-0013
    point 15), **resources lost** (a reward record's negative amounts: the treasury goes to 0 at
    the least, never refusing), a choice with its amounts, a group of up to seven stacks without
    room, the visit's rest after a battle or an army dialog, an event's firing (to its visitor's
    owner) and the move's stop. A price (a School's, a Tree's, a University's) is the treasury's
    and the object type's, never the command's.
11. *(amended, battles)* **The setup's facts from the game** (T54, decision 124; GAM-BAT-010 to
    -013): the game's difficulty; each side's `human` flag from its player's setup flag (false for
    an unowned side), which the battle's bad-morale let-off reads (BAT-QUIRK-27); the sides'
    colours; each hero's sex (a placed hero's from the map); the background (the scenery of the
    tiles around the battle's, a tile's topmost object the last in map order, E08 decision 124's
    agent's call); the attacker's ground (the special ground of the attacking hero's own tile).
    The retreat facts stay at their defaults (`ADV-STUB-9`: a Tavern to hire the hero again, E09;
    the victory condition and the computer player's reading, E10). **Increment 7's battles:** a
    box's, an event's, a guarded artifact's and a guarded scroll's guards fight as a guarded
    pile's (unsplit; their survivors stay with the object); the Pyramid's as a monster battle with
    a second creature (ADR-0016 point 4), whose losses are not kept (the next fight is against the
    full guards). When a battle ends every hero in it loses its battle bonuses (ADR-0013 point
    15.1), as the original's `battle_end`.

## Consequences

- `Answer` becomes an enum by decision kind: every caller that builds one changes (the env's
  action mapping, the CLI's replay files, the properties' plans, the tests); the replay format
  bumps once (v7); `h3-protocol` maps no adventure command yet (the server lane's, when it does).
- A wrong answer is a type error at the caller where it can be (a level-up answered with gold is
  not writable), and `WrongAnswer` where the decision is only known at run time.
- The game holds a visit's rest across a battle and its level-ups, and across an army dialog: one
  more kind of pending state, its owner's, in the game state hash.
- Simultaneous play's inventory (point 7) grows by what two players' commands can both write: a
  find emptied, a chest, artifact, scroll, Scholar, box or event removed, an event's guards (the
  objects of increment 2's kind); a player's knowledge and a hero's marks are written by their
  owner's commands only.

## Alternatives considered

- **One answer type for every decision** (today's `Accept`, `Decline`, `First`, `Second`, with
  a `Buy(u8)` added): not chosen (the user, ADV-Q29); nor typed answers for increment 7's
  decisions alone, beside the old ones.
- **A command of its own for the University's purchase** (`GameCommand`): a purchase answers the
  open decision and keeps it open, as a meeting's management does; an answer needs no new
  routing (decision 140).
- **The rewards given at the battle's end, before the level-ups:** simpler, but a level-up's
  Wisdom or Learning would not count for a box's spells and experience, nor Expert Wisdom for the
  Pyramid's spell, unlike the original.
- **The army dialog's rest given before it opens** (VCMI's order, D-39): decision 129, the code's
  side.
