# E05 — Battle environment v1: log

The full record of done tasks (the epic file keeps one line each).

## T1 — Design and ADR draft (2026-10-18)

- **Drafted:** the ADR draft `docs/lanes/ai/adr-drafts/battle-env.md` (13 points) and the design
  note `E05-design.md`; questions AI-Q2 (the reward's defaults) and AI-Q3 (an agent's `human`
  flag). The first draft's reward priced the adventure's stakes: a hero's and its artifacts'
  value, a town, a treasure, a surrender's gold, escapes (transfers counted once).
- **The user, before the draft:** (f) needs no check of the original's screens: every per-player
  observation already is a human player's information.
- **The user's review:** the reward stays simple, without the adventure: kill the most, lose the
  least by `ai_value` (or the creatures' cost), a price on spell points against spamming spells;
  retreat and surrender disabled for now (they belong to the adventure's view and rarely decide
  good play in a typical battle); no treasure, no reward for defeating a hero. `human` is true for
  both sides (a level playing field for human players and agents; `false` is for compatibility).
  Accepted as ADR-0025 (points 6 to 8 changed accordingly; 14 points) in a `shared/` pull request;
  the draft removed.
- **Found on `master`:** the full verify of code 30ecdd57 failed coverage on `h3-battle`'s proptest
  `walks_are_reported_hex_by_hex` (properties.rs:1924: `apply` with an event sink and with
  `NoopSink` end in different state hashes; a town, a Clone); filed in the battle lane's inbox.
