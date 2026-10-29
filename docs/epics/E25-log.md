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

## T1c — T8c again, with the true world (2026-10-28)

- **The runs** (`python/records/search/t7-1-*.json`; `t7-1`'s `best.pt`, the user's upload;
  T8c's settings: W 8 worlds, k 16, seed 1; T1a's terminal, T1b's true worlds; one library
  pinned for all six). The search with guessed and with true worlds beside the sampled policy
  on the same episodes; win rate (both sides weighed equally) and value:

  | Against the system actor | Battles | Sampled | Guessed worlds | True worlds |
  |---|---|---|---|---|
  | N 16, the report set | 2 998 | 0.374, −0.101 | 0.373 ± 0.018, −0.100 | 0.371 ± 0.018, −0.098 |
  | N 64, its first 200 per kind | 1 199 | 0.380, −0.102 | 0.371 ± 0.027, −0.100 | 0.385 ± 0.027, −0.088 |
  | N 64, the neutral set (attacking) | 1 500 | 0.976, +0.243 | 0.970, +0.243 | 0.970, +0.243 |

  Per kind and side every search row lies within its interval of the sampled one (the hero
  battles' defender 0.15 to 0.17 throughout). Costs: the guessed worlds 42 and 52 ms a decision
  (N 16, 64; a world's guess draws 8 heroes), the true 16 and 42 ms, the neutral set 27 and
  31 ms; the first run shared the sandbox's 4 cores with T2a's tests, so its times are high. 220
  and 68 of the guessed worlds' choices fell back to the network's (a cast the guess made legal),
  none of the true worlds'.
- **The read:** with reward v1 at a world's terminal (T1a) the search still gains nothing over
  its network, at N 16 or 64, against the system actor or on the neutral set: E06 T8c's negative
  verdict holds, no longer inconclusive. **The guess costs nothing measurable:** the search over
  the true battle, everything known, gains over the guessed worlds at most +0.014 in win rate and
  +0.012 in value at N 64 (1 199 battles, within the interval) and nothing at N 16 or on the
  neutral set. So neither the terminal's scale
  nor the hidden information limits the search: the network's evaluation does (its values tell
  the actions apart by a few hundredths, E06 T8c), which a deeper or perfectly informed search on
  it does not repair. For increment 2 this argues against expert iteration on this network
  (design §14's T8d) and for a better evaluation first (a network that reads the battle's
  structure, imitation of the fair system actor once built, or scale).
- **The truncations** (T1a's open point): the search stalls a little, the more the more it
  knows: 0.10 % of its battles cut at the step limit at N 16 and 0.50 % at N 64 with guessed
  worlds, 0.53 % and 1.50 % with true ones, the sampled policy none. A world scores a cut 0 where
  the trainer charges `-truncation_penalty` (1.0), so in the tree a losing side prefers stalling
  to its value. Too rare to move the rows (1.5 % of games scored 0 instead of about −0.3 is about
  +0.005 of value), but a search that trains or plays should score a world's cut as the trainer
  does (the AI queue's inbox, the search).

## T2b — The A/B runs (2026-10-28)

- **The runs** (the user's machine; `python/records/e25-control-1.json`, `e25-shaping-1.json`;
  commit `e5cfa46`, #534's merge, clean, C ABI v35; seed 1, 20 M samples each, the configs'
  length): the control (`configs/e25-control.toml`) 73 minutes, 9 of them the gates and ladders;
  shaping (`e25-shaping.toml`) 90, 25 of them evaluation. Both promoted at both gates, best #2
  at 20 M. The final ladders (the full report set and the neutral set; win rate, value; the
  intervals ±0.013 against Random, ±0.018 to 0.019 otherwise):

  | Run | Random | Aggressor | Greedy | System actor | Neutral set |
  |---|---|---|---|---|---|
  | `t7-1` (E06) | 0.828, +0.273 | 0.551, +0.029 | 0.528, +0.010 | 0.374, −0.101 | 0.972, +0.245 |
  | `e25-control-1` | 0.829, +0.271 | 0.548, +0.026 | 0.529, +0.008 | 0.371, −0.104 | 0.968, +0.230 |
  | `e25-shaping-1` | 0.835, +0.283 | **0.589, +0.063** | **0.566, +0.045** | **0.409, −0.066** | 0.973, +0.238 |

  E06's five seed-1 runs at 20 M (`t5-3`, `t7-1`, `t6b-1`, `t6d-1`, `order-1`) and the control
  lie within 0.539 to 0.557 against Aggressor, 0.526 to 0.533 against Greedy and 0.365 to 0.374
  against the system actor: shaping is 0.03 to 0.035 above each range's top, its intervals clear
  of the control's. Per side (`metrics.jsonl`'s final ladder, attacker / defender) the gain is on
  both sides of the hero kinds and none in the garrison siege: against the system actor the hero
  battles 0.541 / 0.174 → 0.585 / 0.210, the siege with a hero 0.416 / 0.272 → 0.472 / 0.316,
  the garrison 0.918 / 0.044 → 0.922 / 0.046. It learns faster too: at 10 M (250 per kind)
  shaping's ladder read 0.829, 0.559, 0.549, 0.393, the control's 0.815, 0.522, 0.489, 0.333.
- **The neutral set without neutral battles** (T2a's mix; the epic's risk): the control's
  +0.230 is E06's runs' +0.245 to +0.253 less about 0.015, at 0.968; shaping's +0.238. Both
  below the system actor's own +0.285, as `t7-1` was.
- **Shaping's cost: a tail of long battles.** In the second half its self-play battles ran 70
  and 63 samples an episode (10 to 15 M, 15 to 20 M) against the control's 53 and 54, and 0.24 %
  and 0.20 % of them were cut at the step limit against 0.01 % (the first half: 0.05 % at most
  for either). Its ladders took three to four times the control's (the final: 83 s against
  Aggressor where the control took 23, 192 s against the system actor where 41): a kind's ladder
  plays its 500 episodes as one batch, so its time is its longest battle's; the ladders cut one
  battle in 1 500 per opponent (the control none). Only the training batch computes the
  potential (`potential=c.shaping`), so the time is the policy's, not the code's. The value
  head's explained variance also fell, 0.51 to 0.68 against 0.77 to 0.79.
- **The long battles are the tactics phase** (both `best.pt`s, the user's uploads;
  `records/inspect/battles.py`, a one-off script: every battle of the report set against the
  system actor, Greedy and Aggressor as `h3env.train.inspect` records it, on the sandbox's CPU,
  whose ladder rows lie within 0.01 of the GPU's). In the 616 battles of 3 000 where the agent
  has a tactics phase (a hero with Tactics), the control placed its stacks 17 to 19 times on
  average (median 4, the 99th percentile 209 to 262, none cut), shaping 42 to 47 times (median
  5, the 99th percentile 723 to 928; 25 to 28 battles above 200 placements, 1 to 4 cut at the
  step limit inside the phase); the phase is 10 to 12 % of the control's decisions (`t7-1`'s 11 %,
  `t6d-1`'s 25 %, E06 T6e) and 24 to 28 % of shaping's. Battles of 50 rounds or more are as rare
  in both (1 to 5 in 3 000). Outside the phase shaping fights more directly: it strikes at 55 to
  58 % of its decisions (melee and shots; the control 45 to 48 %), moves at 24 to 27 % (31 to
  34 %), waits at 1 % (3 %), and its battles take 25 to 28 such decisions (27 to 32). The
  phase's rounds have no limit (BAT-TAC-002) and its
  placements cost nothing at γ 1 (E06 T6e): its potential does not move, so shaping adds nothing
  there while it sharpens the advantages elsewhere; the entropy bonus keeps the placement heads
  broad, and ending the phase is one entry among many. Not stalling: a placement loop.
- **The read:** shaping is the first lever since the truncation penalty (E06 T5d) to move the
  ladder past the runs' spread, against every scripted opponent and the system actor (+0.04 in
  win rate and value; 0.409 against the target's 0.5), and it learns faster. Dropping the
  faithful system actor and the neutral battles costs nothing against the player kinds and about
  0.015 of value on the neutral set. Its cost is the tactics phase's placement loop, which a
  longer run would carry (training time, truncations): a bound on the phase in training is the
  candidate fix before one builds on shaping.

## AI-STUB-1's magic limits: `master`'s red verify (2026-10-28)

- **Found:** CI's full verify on `master` failed at #534's merge (`e5cfa46`; #534 changed no
  `h3-battle` code): `h3-battle`'s `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden`
  drew a case whose attacker wore a hidden Orb of Inhibition, and the defender's world showed its
  own random caster four candidate spells where the battle showed none. A world recomputed the
  magic limits from the guessed hero (`MagicLimits::of`), AI-STUB-1's gap from E06 T9b; the
  property's random cases reach it only now and then (it failed in the sandbox's coverage run and
  passed in its test run and CI's).
- **The fix** (the user: on this branch, `cross-lane`): a world keeps what the battle forbids
  every caster, no casting and the highest spell level (BAT-MAG-061, -062), which each side meets
  in its own casts and legal mask; only `guards`, whether a stack's guard is looked at, follows
  the guessed hero's private defences (`h3-battle`'s `world.rs`). Tests: the failing case pinned
  in `proptest-regressions/tests/properties.txt`;
  `a_world_keeps_what_the_other_hero_forbids_every_caster` (an Orb and a Cloak the guess lacks;
  `guards` from the guess); both fail without the fix; the property clean in nine further runs.
  AI-STUB-1 keeps the other hero's morale and luck modifiers and a commanded war machine without
  a guess.

## T3 — Increment 1's checkpoint (2026-10-28)

- **Put to the user:** T1c (the search gains nothing over its network, even over the true
  battle: the network's evaluation limits it) and T2b (shaping about +0.04 against Aggressor,
  Greedy and the system actor and faster, at the cost of a placement loop in the tactics phase;
  the faithful actor's and the neutral battles' absence costs about 0.015 of neutral value).
- **The user's choices** (the agent's recommendations): increment 2 is **shaping, bounded,
  then scale**: the agent bounds its own tactics phase (T4a; its mask, the rules unchanged,
  wherever the network plays), an A/B against `e25-shaping-1` (T4b), then a long run with the
  better settings (T5, 60 M proposed). **`e25-shaping-1` is the lineage's current best** (#2,
  20 M; the network the exit criteria are judged on), `t7-1` stays the battle AI v1 and the
  baseline. Increment 1 closed (its task list in `E25-archive.md`).

## T4a — The tactics bound (2026-10-28)

- **The form** (the agent's choice, change freely; the user's proposal): `[network]
  tactics_rounds` (0, off, by default; `configs/e25-bound.toml` 1) is the rounds of its own
  tactics phase the agent plays: once its active stack would start a turn of a later round, End
  tactics is its only entry. A round of the phase hands each stack of the army one turn
  (BAT-TAC-002): it places, or waits and is called again within the same round (then places or
  passes), so one round is at most 2 n + 1 decisions for n stacks, End tactics included. The
  rules are unchanged; the original's computer stops after three rounds or a round without a
  placement (its tactics rule, BAT-AI): `tactics_rounds = 3` is its cap, not its early stop.
- **Python only:** no observation shows the phase's round (the battle's `TacticsRounds` is the
  computer's and is not observed), so the binding counts it from the observations it shows,
  without an encoding or ABI change: `BattleBatch.tactics_turns` [N, 42], the turns each stack
  started in the env's current phase (a turn starts where the active stack has not waited), and
  `tactics_round` [N], its active stack's (0 outside a phase), after every `reset` and `step`
  (`count_tactics_turns`; a refused env unchanged, a new episode from zero). About 50 µs a call
  at 256 envs (a step about 10 ms in the sandbox). Not taken: the agent's decisions in the phase
  (no army size), or the round in observation v23 (every network's input).
- **The agent's:** `policy.build` sets `Policy.tactics_rounds` from a file's config (a file
  without it, `e25-shaping-1`'s and every older one, plays unbounded), and `Policy.act` draws
  from `bounded(entry_mask, …)`, so rollouts, league members (each its own file's bound), gates
  (`duel`), ladders and `inspect` (`sampled`) play it. `Choice.entry_mask` is the mask drawn
  from; the rollout stores it, so the update recomputes the same certain probability (log 0) for
  an entry the bound forced. `h3env.search`: the root takes the batch's count, each node counts
  its observation onto its parent's (`Node.turns`; the phase draws nothing, so every world's
  path counts the same), and every node past the bound holds End tactics alone.
- **Tests:** `test_batch.py` (the count against the army's turns over waits, a cut battle's next
  episode, a refused action, a reset), `test_train.py` (the bound leaves End tactics alone past
  it and every other choice as unbounded on the same draws; a bounded attacker's `duel` ends its
  phase at its first turn of round 2; a rollout stores the bounded masks and updates),
  `test_search.py` (the tree's nodes past the bound, and the root, bounded or not by the
  network's config; it fails with the tree unbounded), `test_run.py` (`e25-bound.toml` is
  `e25-shaping.toml` with the bound; negative refused; a resume keeps it).

## T4b — The A/B: the bound against `e25-shaping-1` (2026-10-29)

- **The run** (the user's machine; `python/records/e25-bound-1.json`; commit `43c8cdd`, clean,
  C ABI v35, the core ruleset at v50, no battle rule moved since T2b; seed 1, 20 M samples,
  `configs/e25-bound.toml`): 73 minutes, 6 of them the gates and ladders (shaping 90 and 25, the
  control 73 and 9). Promoted at both gates (10 M: 0.870 in 23 pairs; 20 M: 0.561 in 82), best #2
  at 20 M. The final ladders (win rate, value; the intervals ±0.013 against Random, ±0.018
  otherwise, ±0.015 on the neutral set):

  | Run | Random | Aggressor | Greedy | System actor | Neutral set |
  |---|---|---|---|---|---|
  | `e25-shaping-1` | 0.835, +0.283 | 0.589, +0.063 | 0.566, +0.045 | 0.409, −0.066 | 0.973, +0.238 |
  | `e25-bound-1` | 0.840, +0.282 | 0.591, +0.067 | 0.573, +0.051 | 0.416, −0.062 | 0.963, +0.233 |

  At or above shaping's point estimates against every engine seat, each difference inside the
  intervals; the neutral set 0.010 and 0.005 lower, inside its interval. Against `promoted-1`
  0.525. No ladder battle cut (shaping one in 1 500 per opponent).
- **The loop is gone.** `records/inspect/battles.py` on its `best.pt` (the sandbox's CPU, the
  ladder rows within 0.007 of the GPU's; shaping's and the control's numbers are T2b's, same
  script and battles): in the 616 battles of 3 000 per opponent where the agent has a tactics
  phase it decides there 3.2 to 3.3 times on average (median 2, the 90th and 99th percentiles 8
  and 9, at most 11; shaping 42 to 47, its 99th percentile 723 to 928; the control 17 to 19), the
  phase 2.4 to 2.6 % of its decisions (shaping 24 to 28 %, the control 10 to 12 %), none cut.
  The median of 2 is a placement and End tactics: it mostly ends the phase itself, well before
  the bound. Outside the phase it plays as shaping does: strikes at 55 to 60 % of those
  decisions (shaping 55 to 58 %, the control 45 to 48 %), moves at 21 to 25 % (24 to 27 %), waits
  at 2 %, 25 to 28 such decisions a battle; battles of 50 rounds or more 2 in 3 000.
- **Training:** no self-play battle cut in any quarter (shaping 0.24 % and 0.20 % in its second
  half); 48 samples an episode in the second half (shaping 70 and 63, the control 53 and 54), so
  386 k episodes in the 20 M samples (shaping 307 k, the control 336 k). Samples per second as
  shaping's (5 380 and 4 800 per gate interval; shaping 5 400 and 4 980, the control 5 790 and
  4 910: a sample costs the same, the run's time moved out of evaluation). The value head's
  explained variance 0.48 and 0.49 (shaping 0.52 and 0.67, the control 0.77 and 0.79) and the
  entropy 1.60 at the end (shaping 1.98, the broad placement decisions gone): the lower variance
  explained is not examined further; the ladders show no cost.
- **The read:** the bound costs nothing measurable and removes the placement loop in training
  and play; with it shaping's run takes the control's time. T4b's better settings are
  `e25-bound.toml`'s, the base of T5's scale run.
- **The user's choices** (2026-10-29): T5 runs **60 M samples** from scratch on T4b's settings
  with the bound at **the original's computer's three rounds** (`configs/e25-scale.toml`,
  `tactics_rounds = 3`, BAT-AI-023's cap; its early stop after a round without a placement is
  left to the agent, which ends the phase itself). So T5 changes the length and the bound
  together: its read against `e25-bound-1` cannot part them. **`e25-bound-1` is the lineage's
  current best** (#2, 20 M; the network the exit criteria are judged on) until T5 reports.
