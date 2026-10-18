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

## Increment 2's cut (2026-10-18)

- With the user: increment 2 is T3 and T4, with the C ABI last (T3d) so that it lands once with
  its reward; this session T3a and T3b; the dispatch line `h3 ai` in `h3-cli`'s `main.rs` (T3c) as
  a `cross-lane` change. Tasks in the lane's `QUEUE.md`.

## T3a — Kernel streams (2026-10-18)

- `StreamId::seat(i)` (`0x200 + i`) and `StreamId::env(i)` (`2^32 + i`) in `h3-kernel` with a test
  that they differ from every other stream; KER-RNG-001's list. Additive: `RNG_VERSION` stays 1.
  A `shared/` pull request, merged before T3b's.

## T3b — The batch core (2026-10-18)

- **Built:** `h3_sim::batch` (design §1, "Built (T3b)"): `BattleBatch::new` / `add_scenario` /
  `reset` / `step` / `spell_mask` / `target_mask` / `second_mask` / `env`; `Scenario::new` forces
  `human = [true, true]` and checks the scenario; `Policy::Random` plays `rollout::pick` (which
  never escapes) from the seat's stream; the masks never allow Retreat or Surrender and `step`
  refuses them as invalid actions; an episode that ends before an external seat acts is skipped
  (64 in a row: the env gives up).
- **Tests** (`src/batch/tests.rs`): the determinism property (proptest, 8 cases, and a fixed
  case of 4 envs over the four synthetic scenarios with mixed seats, 400 calls, 19 episodes
  ended): a batch steps as N single envs built from the public pieces (`episode_seed`,
  `draw_scenario`, `Scenario::env`, the seats' policies on their streams), output by output and
  state hash by state hash, and every action its masks allow applies; a reset replays the same
  episodes; escapes masked and refused (the env unchanged); a hero's cast through the head calls;
  a truncation's final observations and the next episode's seed; skipped episodes and an env
  that gives up (Drakes before Spearmen with a step limit of one); refusals per env; every error.
