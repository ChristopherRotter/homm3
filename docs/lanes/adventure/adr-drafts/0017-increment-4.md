# ADR-0017 amendment (draft): hero progression in the game

- Status: Accepted by the user (2026-10-20, ADV-Q16; the epic's decision 58), drafted in E08
  T24b2 from the answers at ADV-Q15 (decision 55); goes into ADR-0017 through T26's `shared/`
  pull request.
- Amends: ADR-0017 points 1, 4, 5, 6, 8, 9 and 11.
- Detail: `docs/epics/E08-design.md` part B (§11–§17); the hero rules: part A and
  `adr-drafts/0013-increment-4.md`; facts: survey `docs/rules/surveys/E08-progression.md`.

## Context

Increment 4 turns experience into levels (decision 48). A level-up's draws follow the hero's
seed byte and the level in the original (D-14), a human answers each two-offer level in a dialog
one level at a time, the battle's winner levels after the battle, and a placed hero with a custom
experience is levelled while the new game is set up, the computer's evaluation choosing (D-20).
The rules themselves are `h3-heroes`' (the ADR-0013 draft); this draft says what the game keeps,
who answers, where the draws come from and how agents and replays see it.

## Decision

1. *(amended: the store of heroes)* Beside each hero's details the store keeps its **level
   seed** (1–255), drawn from the setup stream when the store is built, never changed. The game
   keeps the scenario's **level limit** and **banned skills** from the setup (static). A setup's
   custom experience is no longer refused (`Unsupported`): the build gives a placed hero its
   **levels at load**, after its custom skills, army, spells and artifacts and before its spell
   points, from experience 0 with the limit, each two-offer level chosen by the original's choice
   (ADR-0013 point 12) with the setup's defaults for the army.
4. *(amended: randomness)* **A level's draws** come from the original's source, the hero's level
   seed and the new level, in `faithful` (`ADV-QUIRK-9`, a fix-optional flag, E08 decision 55);
   in `strict-fixed` from the owner's stream in play and from the setup stream at load. A level
   is drawn when it is given or its decision opens, never earlier.
5. *(amended: server-only)* The level seeds are server-only state, as the streams; a level's
   offers are its owner's.
6. *(amended: pending decisions)* **A level-up** with two offers is a pending decision of the
   hero's owner, answered **`First`** or **`Second`** (the original's left and right offer;
   additive `Answer` variants). A hero's pending levels are given in order, one at a time: a level
   with one offer learns it and one with none gives the primary skill only, at once, with an event
   to the owner; a level with two opens the decision, and its answer gives the level and draws
   the next. The system actor's default answer to a level-up is the original's choice (ADR-0013
   point 12) with the hero's army (the original's answer when the dialog's time runs out).
8. *(amended: the env)* Action encoding v4 appends two entries, the first and the second offer,
   legal only while a level-up is the player's open decision (`Accept` and `Decline` only while an
   offer is). The observation shows its owner the open level-up: the hero, the new level, the
   primary skill gained and the two offers.
9. *(amended: versions)* Increment 4: the game state hash v9, the map import v8, the action
   encoding v4, the adventure replay format v5 (the new answers); the adventure state hash and
   ruleset part unchanged.
11. *(amended: battles)* The winner's experience reaches levels: after the outcome is applied
    (Necromancy and Eagle Eye included), the winner's owner gets its hero's level-ups as above,
    as the original gives a human player's levels after the battle's screen. "No levels until
    increment 4" is dropped.

## Consequences

- Faithful level-ups are predictable from the seed byte, which stays server-only, so an agent
  learns no more than a reloading player; `strict-fixed` hides them behind the owner's stream.
- A level-up is the first decision with an answer other than yes or no; the observation's open
  decision becomes a choice of kinds (an offer or a level-up), which the env and its clients map.
- Level-ups write only their owner's hero: nothing for simultaneous play's inventory (point 7).
- The golden replays, the fixture heroes and the property plans change with the decision (a
  battle's winner now answers its levels); the replays are re-recorded once.
- The computer player's own choice at a level-up stays E10's; it calls the same rule.

## Alternatives considered

- **`Accept` and `Decline` as left and right**: no encoding change, but an answer whose meaning
  depends on the decision's kind; agents and replays read clearer with their own entries.
- **The owner's stream at load in `strict-fixed`**: the setup's randomness is the setup
  stream's (point 4); a player's stream would move with the number of heroes it owns.
- **All of a battle's levels drawn at once**: the original draws each level after the previous
  answer, whose skill enters the next draw (survey §2.1).
- **The first offer as the system actor's default** (VCMI's for a hero without an owner): not
  the original's; kept as the fallback if the evaluation's reading finds the army's path costly.
