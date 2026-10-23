# ADR-0017 amendment (draft): management commands, meetings and the army dialog as decisions, the tactics flag in battle, the encoding to the AI lane

- Status: Proposed (E08 T44b, 2026-10-23), from the answers at ADV-Q24 (the epic's decisions
  93–100); for the user's acceptance at ADV-Q25 (its questions 2 and 3 settle the bracketed
  parts). The round-1 `shared/` task applies it to ADR-0017.
- Amends: ADR-0017 points 1, 6, 8, 9 and 11.
- Detail: `docs/epics/E08-design.md` part B (§8–§13); facts: survey
  `docs/rules/surveys/E08-army.md` §1, §2, §4, §6.

## Context

The original's hero screen, exchange screen and army dialog are interactive; the engine has no
blocking prompts (point 6). Increment 6 turns what they allow into commands (ADR-0013's and
ADR-0016's drafts) and two of them into pending decisions: the meeting a step toward an own hero
opens (decision 93) and the army dialog of a join that finds no room (decision 94). The tactics
flag reaches the battle through the heroes' Tactics (decision 96). The user handed the adventure's
RL encoding to the AI lane (decision 99), which point 9 already foresaw ("to the AI lane at E05").

## Decision

1. *(amended)* A **management command** (`GameCommand`'s new variant) names its player's heroes
   and places: a stack's move, split or dismissal; the formation; the tactics flag; an artifact's
   move; an assembly; a disassembly. The game checks the actor, the turn and the open decision,
   then the armies' rules in the adventure and the artifacts' in `h3-heroes` through the store,
   both before either applies; Scholar's spells are hero changes the game applies when it opens
   a meeting. [ADV-Q25 question 3:] The setup carries the map's **edition** (RoE, AB, SoD), a
   static input the rules that differ by edition read (the combinations now; the misc positions'
   count, the moat at E09).
6. *(amended)* Two more kinds of pending decision, each its player's alone, holding no one else
   and blocking its owner's `EndTurn` like an offer:
   - **A meeting** of two of its heroes, opened by the step toward the visited one after Scholar:
     while it is open its owner's legal commands are the management commands on either hero
     (between them or within one) and `Decline`, which closes it. The next meeting needs another
     step.
   - **An army dialog** of a hero and a joining group, opened by an accepted join that finds no
     room (after the price and the treasure, the monster gone): the management commands between
     the hero and the group's small army (ADR-0016's draft) and on the hero alone, and `Decline`,
     which closes it; the group's rest is lost (`ADV-STUB-2` closed).
   Outside a decision the management commands are legal in the owner's turn on any of its heroes
   on the map, but for the moves between two heroes, which need the meeting. `legal_commands`
   lists every management command but the splits; a query gives a split's bounds.
8. *(amended)* **The adventure's RL encoding belongs to the AI lane** (decision 99): the env's
   action and observation encodings and masks, the rollout and their tests; the adventure lane
   keeps the game's commands, `legal_commands` and `legal_moves`, the decisions' views, the
   replay model of game commands and the CLI's replays and benchmarks. [ADV-Q25 question 2: the
   paths, `crates/h3-sim/src/adventure/` but its replay model to the AI lane in `lanes.toml`.]
   Increment 6 adds no entry to the action encoding (v4); its masks allow `Decline` while a
   meeting or an army dialog is open, so an agent that meets a hero or fills its army closes the
   decision and plays on.
9. *(amended)* Increment 6: the game state hash (the decisions, the edition), the adventure state
   hash (the flag), the details' hash (positions), the core ruleset hash and the pack format
   (shared: two effect kinds), the map import (positions, the edition), the adventure replay
   format v6 (the management commands); the action encoding's version is the AI lane's.
11. *(amended)* **The tactics flag in battle.** When the game builds a battle's setup it gives
    each hero's `BattleHero` its Tactics as `h3-heroes` derives it, unless the side whose hero
    has the greater Tactics level belongs to a hero on the map whose flag is off: then **both**
    heroes' Tactics go as 0, so no side gets the tactics phase and the two levels' cancelling
    (BAT-TAC-001) stays as the experiments confirmed it (the user's condition at ADV-Q24). No
    seam changes.

## Consequences

- Clients and servers send the new commands; the server lane maps them when it serves
  adventure games.
- An open meeting or army dialog is a state a fork shares; its views are the owner's.
- The AI lane takes the env with an encoding that ignores increment 6's commands until it
  extends it; the replays record them from v6 on.
- A battle's observation shows the zeroed Tactics of both heroes in a battle the flag cancelled.

## Alternatives considered

- **A field of the battle's setup** for the flag (`cross-lane`, mapped on the wire): not chosen
  at ADV-Q24; the zeroing gives the same battle.
- **Zeroing only the flagged hero's Tactics**: would hand the other side the phase when its level
  is lower but above 0, which the original never does.
- **An answer of its own to close a meeting or an army dialog**: `Decline` already closes a
  decision without carrying anything out.
- **A subset of entries in the adventure lane's encoding now** (not chosen at ADV-Q24).
