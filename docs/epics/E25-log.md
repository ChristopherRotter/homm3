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

## T2a — Shaping and the training mix (2026-10-28)

- **The mix** (the user, 2026-10-28: "no neutral battles for now"): the faithful system actor
  trains no agent (decision 2). `run.trainable` refuses a `system_share` above 0 and any neutral
  kind's weight (wandering, guards, a neutral town: its defender is the system actor);
  `ppo.train` and `ppo.resume` call it. A config may still name them, so E06's configs
  (`e06-t6b`, `e06-t7`, `e06-mana`, `e06-order`) and records stay valid documents; the trainer
  starts or resumes none of them. The mechanisms (`system_share`'s entries, the neutral share)
  stay in the code for a fair system actor later. The agent's proposal, the neutral side in
  self-play, was not taken: it needs a batch seat for it, a later choice.
- **Φ from the batch** (the agent's choice, change freely; the queue left it open): Python cannot
  compute it (the observation holds no creature values), so `h3_sim::reward::potential`
  (`Balance::standing`: each side's losses by its stacks' current counts, its hero's mana spent
  so far, reward v1's scale) is a batch output, `Outputs::potential` / `H3BatchBuffers.potential`
  ([N, 2], written only when asked, as the kill preview; C ABI v35 with T1a, T1b, since v35 is
  not merged yet), `BattleBatch(potential=True)`. It reads the whole state, as the reward does,
  and is never an agent's input. At a decided battle's end it is reward v1's balance but for the
  end's corrections (a temporary resurrection, BAT-MAG-035, -038; none of the test's 40 sampled
  battles meets one).
- **The shaping** (`[ppo] shaping`, off by default; `Trainer.shape`): a learning side's
  transition gains Φ(s′) − Φ(s), Φ the batch's potential at the side's next decision, 0 at the
  episode's end (decided or cut), and for its first transition the episode's start (Φ 0, not its
  first decision's: the opponent may have struck first). So each side's shaping sums to 0 and an
  episode's shaped return is its reward v1 (with a cut's `-truncation_penalty`) exactly; the part
  before a side's first decision only shifts its first value target. Refused with
  `truncation_penalty` 0: the batch reports no potential of a truncated battle's last state to
  bootstrap beside. Its per-(env, side) state lives with the episodes in flight (a resume starts
  it at 0), so the checkpoint format is unchanged.
- **T2b's configs:** `configs/e25-control.toml` (`t7-1`'s settings with the player kinds alone at
  its 4 : 2 : 1, `kinds = [0, 0, 100, 50, 25, 0]`; the ladders still play the neutral set and the
  system actor) and `configs/e25-shaping.toml` (the same with `shaping = true`), 20 M samples
  each until the length is agreed (decision 5).
- **Tests:** `h3-sim` `the_potential_is_the_balance_so_far_and_reward_v1s_at_the_end` (40
  sampled encounters of every kind played at random: 0 at the start, zero-sum at every step,
  `Balance::of`'s at the end), `the_batch_writes_each_envs_potential`; `h3-ffi` (the ABI's
  potential is the Rust batch's), the C smoke test (zero-sum); Python `test_abi.py` (the
  buffers' layout), `test_batch.py` (written on request, 0 at an episode's start, zero-sum),
  `test_run.py` (the refused mixes, E06's configs refused and T2b's trainable, `shaping` without
  the penalty refused), `test_train.py` (`Trainer.shape` by hand; two trainers alike but for
  `shaping` play the same rollout, and each side's ended episode returns the same). 148 Python
  tests with torch on the CPU (the CUDA one skipped).
