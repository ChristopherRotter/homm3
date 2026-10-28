# E25 — Log

The full record of E25's done tasks (`docs/epics/E25-battle-ai-v2.md` keeps one line each). Read
an entry only where a task builds on it.

## T1a — The search's terminal by reward v1 (2026-10-28)

- **The choice** (the agent's, change freely): **a world scores itself**. `BattleEnv` gains an
  optional reward config (`with_reward`); `BattleBatch::determinize` gives each world the
  batch's, so a world's `step` (its forks' too) reports reward v1 when it decides the battle
  (`reward::rewards` from its setup and the outcome, as the batch's `advance`), 0 otherwise;
  every other single env keeps ±1 by the winner (replays, the battle benchmarks, `h3env.tools`).
  The alternative, a C function scoring any single env by a config the caller passes, was not
  taken: every caller would have to carry the batch's config and know when to call it, while a
  world already belongs to one batch; a search in Rust later (expert iteration) gets the same
  value from `BattleEnv::step`. `h3env.search` is unchanged but its docstring: its terminal
  already backs up the step's rewards per side.
- **Found while building, fixed with it:** a world's setup held the guess as the other side's
  hero, with the guess's own starting mana, and reward v1's mana term reads
  `setup.heroes[o].magic.mana` (spent = start − left). Each world's terminals were off by
  `p · (guessed − true start) / S`, a different amount per world, where actions differ by a few
  hundredths (E06-log T8c). `BattleEnv::world` now keeps the battle's starting mana in the
  guess: the other hero's mana is public (its view; `world.rs`'s `public`) and the view showed
  it at the battle's start, so the world reads nothing its side did not see (ADR-0025 point
  12a). The world's other private parts in the setup stay the guess's.
- **Left as it is:** a truncated world scores 0 (the batch's reward at a truncation); the
  trainer gives a truncated episode `-truncation_penalty` (`ppo.py`, E06 T5c), so a tree that
  reaches the step limit values it above the value head's scale. A tree reaches it only within
  its depth of the 2 000 steps: T1c counts the truncations if they matter.
- **C ABI v35:** the meaning of a world's `H3StepResult` rewards (no function or layout change;
  `h3.h`, `lib.rs`'s history, the smoke test's version check); `h3env.ABI_VERSION` 35.
- **Tests:** `h3-sim` `a_world_scores_a_decided_battle_by_the_batchs_reward_v1` (six duels, a
  world per env on its episode's seed, equal to the env by its hash, played as the batch plays
  it: each ends at the batch's step with the batch's reward bit for bit, at mana price 3, below
  1 in magnitude; 0 before), `a_world_of_an_env_shows_its_viewer_the_battle_and_plays_to_its_end`
  (a guess's and a bare hero's starting mana are the battle's); Python
  `test_batch.py::test_a_world_scores_a_decided_battle_by_the_batchs_reward_v1` (the same through
  the binding, the world's hash against a single env of the episode), `test_abi.py` (v35).

## T1b — The true-world search (2026-10-28)

- **Built:** `BattleBatch::true_world(index, seed)`: a fork of the env's battle with its stream
  re-seeded (stream `BATTLE` of `seed`) and nothing guessed, scored as a world (T1a's reward
  v1); `h3_batch_true_world` (C ABI v35 with T1a: one bump, the queue's suggestion), null for a
  null or not reset batch or an env past it or not playing; `h3env.BattleBatch.true_world`. All
  three documented as trusted tooling, a diagnostic that reads the whole state (ADR-0025 point
  12a, E25 decision 6).
- **The search** (the agent's choices, change freely): `Searcher(..., true_worlds=False)` and
  `search(..., true_worlds=False)` pick the kind of world (`Searcher.world`); the CLI's flag is
  `--true-worlds` where the queue said `--worlds true|guessed`, since `--worlds` already counts
  the worlds (E06 T8c's commands and records use it); a report's settings name `true_worlds`.
  Nothing else reaches it: the trainer, the evaluation and the ladder never build a `Searcher`.
- **Tests:** `h3-ffi` `a_true_world_through_the_abi_is_the_rust_batchs` (each env's true world's
  hash is the env's fork re-seeded, the Rust batch's the same; refused arguments null); Python
  `test_batch.py::test_a_true_world_is_the_battle_with_its_stream_reseeded` (the full armies:
  the true world equals a single env of the episode re-seeded, both sides' observations
  included, where a determinized world guesses the other hero), `test_search.py`
  (`test_a_search_takes_true_worlds_only_when_asked`: without the flag no true world is read,
  with it none is guessed; the report with `--true-worlds`). 140 Python tests with torch on the
  CPU (the CUDA one skipped); the scoped verify, `determinism` and `bench-check --workload
  random-rollouts-full` (−0.01 %) green.
