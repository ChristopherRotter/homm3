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

## T2 — Scenarios through the C ABI (2026-10-18)

- **Built:** `h3_env_create_scenario(toml, len, seed, max_steps, error, error_len)` (C ABI v26,
  `h3-ffi`'s `src/scenario.rs`): the battle lane's scenario TOML unchanged, its packs by id from
  the five built-in ones (`synthetic`, `synthetic-ext`, `base`, `ab`, `sod`), random obstacles per
  episode where the scenario asks for them, `difficulty` and `human` as written. Null on error,
  the message in the caller's buffer prefixed by its stage (`scenario:`, `unknown pack`,
  `packs:`, `setup:`), cut at a character boundary and NUL-terminated; untouched on success.
  `h3_env_create_synthetic` takes the same path (its texts built in; unchanged results).
- **Where:** in `h3-ffi`, not `h3-sim`: `arch.toml` lets `h3-sim` use `h3-pack-battle` only in its
  tests. The batch (T3) parses its scenarios the same way (design §4.1, "Built").
- **Tests** (`src/scenario/tests.rs`, through the ABI on `base`, `ab` and `sod`, random legal
  actions with every head to the end, both viewers observed): a field (a Knight against a
  Barbarian, an `ab` creature, war machines); all twelve special battlefields of the packs, each
  on a terrain it lies on; an anti-magic garrison (no hero's cast is ever legal; legal in the same
  battle without it, BAT-MAG-061); a Castle under siege (the hero's aimed Catapult shots become
  legal, BAT-SIE-011; never without the siege); the synthetic siege by text; the seed (same seed
  same state, a reset back to it, another seed another battle); the errors and the buffer. In the
  sandbox's runs every battle was decided (none truncated).
- **C smoke test:** also plays a siege from a scenario's text (aimed wall shots) and checks a
  syntax error's message.
- **Cost:** one call compiles its packs: about 18 ms for `base`, `ab` and `sod` in a release build
  (2.6 ms the synthetic skirmish), ten times that in debug; the batch shares its rulesets.
- **Closed:** F-robust-2 (no siege reached the C ABI) and the E01 inbox item's half on loading
  packs and scenarios (batch stepping stays T3); a line in the battle lane's inbox.
