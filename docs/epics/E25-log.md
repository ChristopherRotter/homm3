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

## T5 — The scale run (2026-10-29)

- **The run** (the user's machine; `python/records/e25-scale-1.json`; commit `4395eed`, #576's
  merge, clean, C ABI v35; seed 1, `configs/e25-scale.toml`: 60 M samples, three tactics
  rounds). **Stopped by the user at 41.3 M** (1 697 updates, 137 minutes, 4.6 of them the gates
  and ladders): the curve was flat, so it has **no final ladder**. Promoted at 10, 20 and 30 M
  (0.804 in 28 pairs, 0.528 in 198, 0.523 in 301), rejected at 40 M (0.507 in 354 pairs, +5
  Elo, LLR −2.95): best #3 at 30 M. The ladders (win rate, value; 250 per kind during the run,
  intervals about ±0.025; the last row `records/inspect/battles.py` and `h3env.train.inspect
  --set neutral-v1` on its `best.pt`, the sandbox's CPU, the full sets, 500 per kind):

  | Network | Random | Aggressor | Greedy | System actor | Neutral set |
  |---|---|---|---|---|---|
  | `e25-bound-1` #2 (20 M, final) | 0.840, +0.282 | 0.591, +0.067 | 0.573, +0.051 | 0.416, −0.062 | 0.963, +0.233 |
  | `e25-scale-1` #1 (10 M) | 0.825, +0.270 | 0.564, +0.044 | 0.544, +0.029 | 0.399, −0.079 | 0.968, +0.228 |
  | `e25-scale-1` #2 (20 M) | 0.834, +0.279 | 0.583, +0.060 | 0.555, +0.042 | 0.412, −0.067 | 0.979, +0.241 |
  | `e25-scale-1` #3 (30 M) | 0.834, +0.285 | 0.592, +0.069 | 0.574, +0.050 | 0.416, −0.060 | 0.972, +0.239 |
  | #3, full sets (CPU) | 0.840, +0.284 | 0.593, +0.073 | 0.580, +0.057 | 0.418, −0.061 | 0.969, +0.241 |

  On the full sets #3 is 0.000 to 0.007 above `e25-bound-1` in win rate and 0.001 to 0.008 in
  value against every seat, each inside the intervals (±0.013 against Random, ±0.018 otherwise,
  ±0.014 on the neutral set); the neutral value still below the system actor's own +0.285. Per
  side against the system actor: the hero battles 0.593 / 0.224, the siege with a hero 0.486 /
  0.322, the garrison 0.918 / 0.052 (shaping's T2b: 0.585 / 0.210, 0.472 / 0.316, 0.922 / 0.046).
  The run's own head-to-heads: #3 against #2 0.510, the 40 M network against #3 0.507. The
  head-to-heads against `e25-bound-1` and `t7-1` were not played (their `best.pt`s are on the
  user's machine; `evaluate.ladder`'s `networks` plays them).
- **Training:** one self-play battle cut in 818 k episodes (35 to 41 M); 48 to 50 samples an
  episode after the first 5 M (`e25-bound-1` 48). Samples per second 5 300 falling to 4 840
  (`e25-bound-1` 5 380 and 4 800). The league score 0.524, 0.519, 0.509, 0.509, 0.506, 0.504 per
  5 M from 10 M; the entropy 1.79 to 1.50; the value head's explained variance rose with length,
  0.48, 0.54, 0.59, 0.61 per gate interval (`e25-bound-1` 0.48 and 0.49), with no trace in the
  ladders. The learning rate falls linearly to 5 % over the 60 M: at the stop it was 7.8e-5, 31 %
  of the start, so the run's last third (the low rates) never ran; `e25-bound-1`'s 20 M schedule
  reached the floor and the same ladder, so the missing anneal is unlikely to move it (not tested).
- **Three rounds:** in the 616 battles of 3 000 per opponent with a tactics phase it decides
  there 5.1 to 5.5 times on average (median 2, the 90th and 99th percentiles 17 to 18 and 23 to
  25, at most 25 to 32; `e25-bound-1` 3.2 to 3.3, at most 11), the phase 3.6 to 4.1 % of its
  decisions against the system actor, Greedy and Aggressor (`e25-bound-1` 2.4 to 2.6 %), 2.5 %
  against Random; none above 50, none cut. No loop: the bound holds the phase to its three
  rounds, and the median battle is still a placement and End tactics. Outside the phase, against
  the same three seats, it strikes at 52 to 55 % of its decisions, moves at 27 to 31 %, casts at
  11 to 12 % and waits at 1 %, 25 to 29 such decisions a battle (T4b's `e25-bound-1`: 55 to 60 %,
  21 to 25 %, waits 2 %, 25 to 28); battles of 50 rounds or more 3 in 9 000 (against Random 15 in
  3 000, none cut).
- **The read:** scale gains nothing at these settings. From 20 M the MLP with shaping and the
  bound is at `e25-bound-1`'s ladder and stays there to 41 M (the 40 M gate +5 Elo); three rounds
  instead of one cost and gain nothing measurable (the two changes together, T4b's caveat). The
  system actor's 0.418 is the lineage's best and still 0.08 short of the target's 0.5. Where E06's
  `t4-2` rose to 50 M (no truncation penalty, no shaping), shaping passes that level by 20 M
  and samples add nothing more: the limit is the agent (its network or what it reads), not the
  training's length.

## T6 — Increment 2's checkpoint (2026-10-29)

- **The user's choices:** `e25-scale-1` #3 (30 M, three tactics rounds) is **the lineage's
  current best**, the network the exit criteria are judged on (at or above `e25-bound-1` on every
  point estimate; later runs use three rounds). The head-to-heads against `e25-bound-1` and
  `t7-1` are skipped (the engine seats compare them on the same battles).
- **Increment 3, the network's architecture (AI-Q11):** asked to choose among the candidates
  (the agent recommended the entity network revived), the user asked which architectures are
  possible: one that correlates hexes properly and efficiently, a larger or deeper network
  (never tried: every run since E05 is the 2 × 512 MLP), else the entity network, scrutinised.
  The agent's options and the user's answer: AI-Q11 (`lanes/ai/QUESTIONS-ANSWERED.md`). **The
  user:** a distillation bench over a hex convolution network, a hex transformer, a deeper and a
  wider MLP; the agent adds the current MLP (the baseline) and E06's entity network as it is.
- **Why the entity network underperformed** (E06 T5b, T5d; what the bench can tell): its move
  logits read a hex's own 16 features and the context, not where the stacks stand, so "toward
  that stack" was hard (T5b's diagnosis of its stalling); with the truncation penalty it stayed
  behind per sample (`t5-2` 0.770 against `t5-3`'s 0.832 at 20 M) at 3.4 times the cost, its
  entropy barely falling (3.44 to 3.11; the MLP 2.74 to 1.89), its optimiser the MLP's. The
  bench separates the two causes: a network that cannot fit the teacher's move head lacks the
  representation; one that fits it but learns slowly in RL points at the optimisation.

## T7a — The network kinds (2026-10-29)

- **The MLP's depth:** `[network] layers` is the MLP's trunk's layers too (2 the default; a
  saved MLP without the key has two, every MLP before had: the parameter names unchanged).
- **The hex networks** (`h3env.train.hexnet`; observation v22 and action v13, no encoding
  change): the entity network's heads moved into a base class (`entity.Pointed`, names and
  creation order kept: an E06 entity network built from the same seed is the same, tested
  once); both new kinds draw the battle on the field (each hex's planes and a per-hex
  embedding, each present stack's token on its position and a two-hex stack's on its front hex,
  the global vector on every hex; a turret only keeps its token) and read out a context, the
  stack vectors (token plus the trunk's vector under it) and the hex vectors to those heads.
  `hexconv`: residual blocks of two 3 × 3 convolutions on the axial grid (22 × 11 cells, 187
  real) with the kernel's corners (+1, +1) and (−1, −1) zero, a global-pooling bias in each
  block, the branch initialised at 0, the padding cells kept at 0, no normalisation (agent's
  choices, change freely). `hexformer`: a global token and the 187 hexes through pre-norm blocks
  with a learned bias per head and axial offset (one table per block). The field's size is
  `[network] field_width`, `field_height` (17, 11, the base ruleset's; the C ABI does not name
  it), checked on real battles; `layout.py` gains `TWO_HEX` and `VIEWER_ATTACKER`, pinned by
  `h3-sim`'s layout test.
- **Tests:** `tests/test_hexnet.py` (the kernel's reach against `Field::neighbors`'s rule on
  every hex, a stack drawn on its hexes per side and seat, a turret drawn nowhere, an empty slot
  unseen and the slots' order free, the field on real battles, the MLP's depth, the cost
  table's convolutions); `test_train.py`'s masks and log-probabilities and a run stopped,
  resumed and laddered, for each kind. 167 Python tests pass (the GPU's skipped).
- **Costs** (`python -m h3env.train.network --time`, 256 observations, the sandbox's 4 CPU
  threads, the median of 10; the GPU decides a run's speed):

  | Network | Parameters | Multiply-adds a decision | Forward µs | Training pass µs |
  |---|---:|---:|---:|---:|
  | MLP 2 × 512 (the current) | 5.95 M | 5.5 M | 62 | 65 |
  | MLP 2 × 1 024 (wider) | 12.4 M | 11.6 M | 78 | 166 |
  | MLP 2 × 2 048 | 26.8 M | 25.3 M | 130 | 328 |
  | MLP 4 × 512 (deeper) | 6.47 M | 6.1 M | 71 | 114 |
  | `hexconv` 64 × 6 blocks | 0.69 M | 85 M | 675 | 1 984 |
  | `hexconv` 32 × 6 blocks | 0.21 M | 21 M | 305 | 728 |
  | `hexformer` 64 × 4 blocks | 0.34 M | 45 M | 1 027 | 6 091 |
  | `entity` 128 × 2 (E06's) | 1.19 M | 15.5 M | 339 | 724 |

  The hex networks share their weights over the hexes: few parameters, many operations; the
  bench measures them on the user's GPU.
- **Docs:** `python/README.md` names the kinds; `docs/07-simulation-and-ai.md` §1.2 (a shared
  path) gains them with E25's other levers (shaping, the tactics bound) at the epic's closure.

## T7b — The distillation bench (2026-10-29)

- **Built** (`h3env.train.distill`, `configs/distill/e25.toml`; `python/README.md`): the teacher
  (a run's network file; here `e25-scale-1` #3) plays itself in `ppo.Trainer`'s rollouts on its
  run's training distribution (its kinds, tactics bound and shaping, the sampled policy,
  self-play only, the bench's seed 7); every student learns on the same minibatches: the KL
  divergence from the teacher's entry head over the legal entries, plus a cast row's spell,
  target and second heads along the teacher's choices, plus 0.5 times half the squared error
  against the teacher's λ-returns (`ppo.gae`: γ 1, λ 0.95, the shaped rewards, so the targets
  PPO's value head regresses on). Adam at 3e-4 falling linearly to a tenth over the samples, 2
  epochs a rollout in minibatches of 512 rows, the gradient's norm clipped at 0.5 (agent's
  choices, change freely). A held-out set (16 384 samples of rollouts on seed 8, collected once)
  scores every student every 500 k samples: the KL per head, the agreement of its most likely
  entry with the teacher's (overall and per category of the teacher's: tactics phase, wait or
  defend, move, melee, shot, heal, cast, other), its value's mean squared error and explained
  variance against the returns (the teacher's own beside it) and its training seconds (the
  GPU synchronised around each student's steps). `report.json`, `metrics.jsonl`,
  `students/<name>.pt` (run network files with the teacher's other settings: `--init-from`
  starts T7d from one). The students: the MLP 2 × 512 (the teacher's own architecture, from
  scratch), 2 × 1 024, 4 × 512, `hexconv` 64 × 6, `hexformer` 64 × 4 (4 heads), `entity` 128 × 2.
- **Memory:** PPO's minibatches (a rollout over 8, about 3 000 rows) killed the first CPU smoke
  (the sandbox's 15 GB): the hex networks' activations at that size take several GB each (and
  would not fit the user's 8 GB GPU either); at 512 rows six students on the CPU used 6.8 GB of
  RAM. A hex network's RL run needs `[ppo] minibatches` raised (`TRAPS.md`, T7d's scope).
- **The CPU smoke** (the sandbox, 65 k samples, 4 096 held out; a check of the code, not a
  result): the teacher's value explains 0.608 of its held-out returns' variance (its run's 0.61:
  the targets are its own); after 65 k samples the entry's KL is 0.51 to 0.65 for the MLPs and
  1.01 to 1.07 for the structured networks, which learn more slowly this early; training seconds
  for the 130 k sample passes: 16, 28, 18 (the MLPs), 256 (`hexconv`), 936 (`hexformer`), 72
  (`entity`).
- **Tests** (`tests/test_distill.py`): the config's refusals and `--set`, every bench config valid,
  the categories, a student that is the teacher scores KL 0 and agreement 1 (another one does
  not), a bench end to end (its students learn, its evaluations, report, metrics and files,
  a second run into the same directory refused).

## T7c — The bench run (2026-10-30)

- **The run** (the user's machine, GTX 1070; `python/records/distill/distill-1.json`; commit
  `62c5bf1`, clean; `configs/distill/e25.toml`: the teacher `e25-scale-1` #3, 4 M samples, two
  passes in minibatches of 512, the held-out set 16 384 samples, evaluated every 500 k). Training
  seconds 9 628 in all (the teacher's rollouts and the evaluations beside them). The held-out
  decisions: 958 of the tactics phase, 3 592 moves, 6 281 melee, 2 846 shots, 2 157 casts. At 4 M
  (KL from the teacher's entry head and summed over its cast heads; the agreement of the most
  likely entry, overall and per the teacher's category; the value's explained variance against
  the λ-returns, the teacher's own 0.575; the GPU's microseconds per sample and pass):

  | Student | Parameters | KL entry | KL casts | Agreement | Moves | Melee | Shots | Tactics | Value | µs |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | MLP 2 × 512 | 5.95 M | 0.039 | 0.039 | 0.868 | 0.760 | 0.889 | 0.906 | 0.793 | 0.567 | 22 |
  | MLP 2 × 1 024 | 12.4 M | **0.029** | **0.030** | **0.884** | **0.787** | **0.908** | **0.920** | **0.807** | **0.612** | 33 |
  | MLP 4 × 512 | 6.47 M | 0.039 | 0.040 | 0.868 | 0.761 | 0.890 | 0.908 | 0.772 | 0.588 | 23 |
  | `hexconv` 64 × 6 | 0.69 M | 0.380 | 0.391 | 0.619 | 0.448 | 0.588 | 0.697 | 0.487 | 0.485 | 466 |
  | `hexformer` 64 × 4 | 0.34 M | 0.363 | 0.393 | 0.625 | 0.476 | 0.580 | 0.702 | 0.494 | 0.470 | 553 |
  | `entity` 128 × 2 | 1.19 M | 0.334 | 0.296 | 0.639 | 0.361 | 0.666 | 0.739 | 0.449 | 0.497 | 106 |

  Over the last 2 M samples every KL still falls slowly (the wide MLP 0.039 to 0.029, the
  structured networks by about a fifth), the order never changes. The values: the wide MLP
  0.530, 0.566, 0.589, 0.601, 0.612 (past the teacher's from 3 M on), the deep 0.365 to 0.588,
  the MLP 0.405 to 0.567, the structured networks 0.40 to 0.50, flattening.
- **The read:** the structured networks fit the teacher about ten times worse than any MLP
  (agreement 0.62 to 0.64 against 0.87 to 0.88), at 5 to 25 times its cost a sample (`hexconv`'s
  466 µs would make a 20 M PPO run about ten hours of network time alone). Part of it is the
  bench's bias: the teacher is an MLP, so its choices hold an MLP's quirks (a slot's own
  weights), which a network free of the slots' order cannot copy. But the value, scored
  against the returns rather than the teacher's outputs, keeps the order too (0.47 to 0.50
  against 0.57 to 0.61): at these sizes and settings they predict the battles worse. Where the
  entity network fails (T6's question): the moves first (0.361, the lowest of all; the hex
  networks 0.45 to 0.48 with the stacks on their hexes), its strikes and shots the best of the
  structured three and still 0.22 and 0.17 below the MLP's; what E06 T5b found (a move logit
  reads a hex's own features and the context) holds, and drawing the stacks on the hexes
  repairs only part of it. The width is the one lever: the wide MLP leads on every head and
  category and its value explains more of the returns than the teacher's own, while depth
  alone gains little (the value only).
- **The user's choices** (2026-10-30): T7d runs the wide MLP alone, from scratch: `e25-scale.toml`'s
  settings at 20 M samples with `[network] hidden = 1024` (`configs/e25-wide.toml`), against
  `e25-scale-1`.

## T7d — The A/B: the wide MLP against `e25-scale-1` (2026-10-30)

- **The run** (the user's machine; `python/records/e25-wide-1.json`; commit `71be686`, #604's
  merge, clean, C ABI v35; seed 1, `configs/e25-wide.toml`: `e25-scale.toml`'s settings, three
  tactics rounds, 20 M samples, `[network] hidden = 1024`, 12.37 M parameters, from scratch):
  94 minutes, 5.9 of them the gates and ladders. Promoted at both gates (10 M: 0.827 in 26 pairs;
  20 M: 0.527 in 263, +19 Elo), best #2 at 20 M. The final ladder (win rate, value; the GPU, 500
  per kind; the intervals ±0.013 against Random, ±0.018 otherwise, ±0.013 on the neutral set)
  beside `e25-scale-1` #3 on the same full sets (T5: the sandbox's CPU) and `e25-bound-1`'s final:

  | Network | Random | Aggressor | Greedy | System actor | Neutral set |
  |---|---|---|---|---|---|
  | `e25-bound-1` #2 (2 × 512, 20 M) | 0.840, +0.282 | 0.591, +0.067 | 0.573, +0.051 | 0.416, −0.062 | 0.963, +0.233 |
  | `e25-scale-1` #3 (2 × 512, 30 M) | 0.840, +0.284 | 0.593, +0.073 | 0.580, +0.057 | 0.418, −0.061 | 0.969, +0.241 |
  | `e25-wide-1` #2 (2 × 1 024, 20 M) | 0.843, +0.285 | 0.593, +0.073 | 0.574, +0.055 | 0.414, −0.059 | 0.973, +0.240 |

  Every point estimate within 0.006 of `e25-scale-1` #3's, each far inside its interval. Per
  side against the system actor: the hero battles 0.591 / 0.214, the siege with a hero 0.484 /
  0.326, the garrison 0.922 / 0.040 (#3: 0.593 / 0.224, 0.486 / 0.322, 0.918 / 0.052). Against
  `promoted-1` (its own 10 M network) 0.523. No ladder battle cut. The head-to-head against #3
  was not played (both networks are on the user's machine: `h3env.train.gate --candidate
  runs/e25-wide-1/best.pt --best runs/e25-scale-1/best.pt --device cuda` plays it).
- **Training:** no self-play battle cut; 71, 52, 47 and 46 samples an episode per 5 M (`e25-scale-1`
  48 to 50 after its first 5 M), 381 k episodes. Samples per second 3 940 and 3 730 per gate
  interval, about 0.7 times `e25-scale-1`'s 5 700 and 5 290 over the same intervals (T7c's GPU
  cost a pass, 33 against 22 µs). The value head's explained variance 0.49 and 0.54
  (`e25-scale-1` 0.48 and 0.54): the bench's lead (0.612 against 0.567 fitted to the same
  returns) does not show in RL. The entropy 2.39 and 1.52 (`e25-scale-1` 2.39 and 1.71 with its
  60 M schedule, `e25-bound-1` 2.37 and 1.60), the league score 0.572 and 0.525 (0.564, 0.521).
- **Not read:** the tactics share and the decisions' mix (`records/inspect/battles.py`): the
  wide network's `best.pt` (49.5 MB of weights) is above the user's upload limit (30 MB). The
  script on the user's machine writes its lines (`TRAPS.md`, Python).
- **The read:** width gains nothing in RL. The wide MLP fits the teacher better (T7c) but
  learns, from scratch and at the same settings, the policy and value the 2 × 512 MLP learns,
  at 1.4 times the run's time. With T5 (samples flat from 20 M) and T7c (the structured
  networks fit worse at many times the cost), the network's size, depth, structure and the
  training's length all end at one ladder, 0.41 to 0.42 against the system actor: the limit is
  not the network's capacity, but what PPO's self-play at these settings drives it to (or what
  the agent may see, the risk the epic names).

## T8 — Increment 3's checkpoint (2026-11-02)

- **The user's choices** (AI-Q12): increment 4 is (a), the policy's sharpness and PPO's settings
  (the agent's recommendation): a temperature diagnostic without training, then 20 M A/Bs on the
  2 × 512 MLP (the entropy bonus decayed to 0; fewer epochs or a larger rollout).
  **`e25-scale-1` #3 stays the current best** (the wide MLP is as strong at twice the parameters
  and 1.5 times a decision's cost). T7d's decisions' mix on the wide network is not read (it is
  not the current best).
- **The user's caveat on T7c:** the convolution's and the transformer's results are to be taken
  with a large grain of salt; they fitted the teacher far worse, and modern convolutional and
  transformer networks rely on size and on training aids (skip connections, normalisation,
  dropout) to train properly. What T7a built: `hexconv` had residual blocks (the branch at 0) and
  a global-pooling bias but no normalisation, 64 channels; `hexformer` pre-norm residual blocks,
  64 wide, 4 heads; 0.69 and 0.34 M parameters against the MLP's 5.95 M. Every student shared one
  optimiser setting (Adam 3e-4 falling to a tenth, no warm-up, minibatches of 512) and 4 M
  samples, and the teacher is an MLP (T7c's bias). So T7c ranks those configurations at those
  settings, not the architectures: a structured network wider, normalised, warmed up and tuned on
  its own learning rate stays a candidate (the epic's candidates), with the GTX 1070's cost (21
  times the MLP's a sample for `hexconv` 64 × 6) the constraint.
- **Why (a) first:** the runs' entropy ends at 1.5 (`e25-scale-1`, 40 M; about four and a half
  equally likely choices) while its KL per update falls from 0.013 to 0.007 and the clip fraction
  from 0.17 to 0.09: the policy keeps much of its spread under the fixed entropy bonus 0.01, and
  the temperature diagnostic measures what that spread costs before any run.

## T9a — The temperature (2026-11-02)

- **Built:** `Policy.act(temperature=…)` divides every head's logits (the entry's, after the kill
  preview's bias, and a cast's spell, target and second heads) before the mask and the draw; 1
  skips the division (the network's own policy, bit-identical: training and every caller
  unchanged), 0 is `greedy`, a negative, infinite or NaN temperature is refused. The
  log-probabilities are then the tempered policy's (an evaluation's only). `evaluate.sampled`,
  `duel` (the learner's side; the opponent network at its own policy) and `ladder` take it;
  `h3env.train.ladder --temperature 1,0.7,0.5,0` plays the ladder once per temperature on the same
  episodes from the same generators (common random numbers), each line prefixed `at <t>:`,
  `--json` a table per temperature keyed by it; `--temperature 1` (the default) prints and writes
  as before. `python/README.md`.
- **Tests** (`tests/test_train.py`): a network whose four heads' weights and biases are doubled
  plays at temperature 2 exactly as the network at 1 from the same generator (a power of two
  scales exactly), every head and the log-probabilities over 30 steps with casts, 1 as the plain
  call, 0 as `greedy`, the refusals; the CLI's lines and tables per temperature, its table at 1
  the plain ladder's, its refusals (a non-number, a negative, a duplicate, infinity, NaN).

## T10a — The entropy's schedule (2026-11-02; the second lever open)

- **Built:** `[ppo] entropy_floor` (1 the default: the bonus constant at `entropy_coef`, every run
  before; a saved config without it reads as 1): `run.entropy_bonus` falls linearly over
  `lr_horizon` as the learning rate does, `entropy_coef × max(1 − progress, entropy_floor)`;
  `Trainer.update(lr, entropy_coef)` takes it (the config's when not given) and each update
  record carries `entropy_coef` beside `lr`. A function of the samples, so no resumable state.
  `configs/e25-sharp.toml`: `e25-scale.toml`'s settings at 20 M samples with `entropy_floor = 0`
  (0.01 at the start, 0 at 20 M), the 2 × 512 MLP from scratch: T10b's first A/B, ready for the
  user's machine before T9b's read (both A/Bs were agreed at T8).
- **Tests:** `test_run.py` (the schedule at floors 1, 0 and 0.25 and on a horizon of its own; a
  floor above 1 refused; `e25-sharp.toml` is `e25-scale.toml` at 20 M with the floor 0);
  `test_train.py` (the same rollout from the same weights: no bonus given is the config's, a
  bonus of its own changes the step; a run's update records carry the constant bonus). 177
  Python tests pass (the GPU's skipped).
- **Open:** the second A/B's config (fewer epochs or a larger rollout), proposed from T9b's read
  and the runs' KL and clip fraction (T8), agreed with the user.

## T9b — The temperature diagnostic (2026-11-02)

- **The run** (the sandbox's CPU, 4 threads; `e25-scale-1` #3's `best.pt`, 30.0 M samples, uploaded
  by the user; `python/records/inspect/e25-scale-1-temperature.json`): `h3env.train.ladder
  --temperature` on the report set v1 (500 per kind, both sides), the same episodes and
  generators at every temperature. At 1 it replays T5's CPU ladder exactly (0.418, −0.061). Two
  minutes a temperature against the system actor (217 s greedy).

  | Temperature | System actor | Hero att. / def. | Siege att. / def. | Garrison att. / def. |
  |---|---|---|---|---|
  | 1 (the policy) | 0.418 ± 0.018, −0.061 | 0.593 / 0.224 | 0.486 / 0.322 | 0.918 / 0.052 |
  | 0.7 | **0.428**, −0.053 | 0.621 / 0.226 | 0.460 / 0.352 | 0.920 / 0.052 |
  | 0.5 | **0.428**, −0.053 | 0.613 / 0.234 | 0.488 / 0.332 | 0.920 / 0.048 |
  | 0 (most likely) | 0.425, −0.052 | 0.611 / 0.226 | 0.484 / 0.342 | 0.912 / 0.042 |

  At 0.7 against the other seats (beside #3's T5 ladder at 1): Random 0.848, +0.290 (0.840,
  +0.284); Aggressor 0.597, +0.077 (0.593, +0.073); Greedy 0.584, +0.061 (0.580, +0.057). The
  neutral set at 1 and 0.7: 0.969, +0.241 and 0.971, +0.243 (the system actor's own +0.285).
  Greedy play cut 1 battle of 2 998 at the step limit (E05 T8b's repetitions gone); no other
  temperature cut one.
- **The read:** the sampled policy's spread costs about 0.01 against every opponent (the same
  sign everywhere, each inside its interval; a cell of 500 battles is ±0.04): a sharper policy
  reaches 0.428 of the 0.5 target, its value still negative. The gap (0.08) is not the spread:
  the policy's most likely choices are about as good as its draws, and they lose where the
  samples lose. The low defending rates are the distribution's base rate, not a defending
  weakness: against itself (`e25-scale-1`'s training at 30 M, `attacker_won` per kind) the
  attacker wins 0.695 of the hero battles, 0.581 of the sieges with a hero, 0.942 of the
  garrisons; against the system actor #3 falls short of those on both sides alike (attacking
  and defending: −0.10 and −0.08 in the hero battles, −0.10 and −0.10 in the sieges with a hero,
  −0.02 and −0.01 at a garrison). For increment 4: the entropy A/B (`e25-sharp-1`, the user's machine, running)
  can gain only if a smaller bonus changes what PPO learns, not by sharpening alone; the
  diagnostic bounds sharpening's share at about +0.01.

## T10b — The first A/B: the entropy bonus to 0 (2026-11-02)

- **The run** (the user's machine; `python/records/e25-sharp-1.json`; commit `d5c7079`, #640's
  merge, clean, C ABI v35; seed 1, `configs/e25-sharp.toml`: `e25-scale.toml`'s settings, three
  tactics rounds, 20 M samples, the 2 × 512 MLP from scratch, `[ppo] entropy_floor = 0`): 72
  minutes, 7.8 of them the gates and ladders. Promoted at both gates (10 M: 0.827 in 26 pairs;
  20 M: 0.528 in 197, +19 Elo), best #2 at 20 M. The final ladder (the GPU, 500 per kind; the
  intervals ±0.012 against Random, ±0.018 otherwise, ±0.012 on the neutral set) beside #3's
  (T5, the sandbox's CPU) and #3's at temperature 0.7 (T9b):

  | Network | Random | Aggressor | Greedy | System actor | Neutral set |
  |---|---|---|---|---|---|
  | `e25-scale-1` #3 (30 M) | 0.840, +0.284 | 0.593, +0.073 | 0.580, +0.057 | 0.418, −0.061 | 0.969, +0.241 |
  | #3 at temperature 0.7 | 0.848, +0.290 | 0.597, +0.077 | 0.584, +0.061 | 0.428, −0.053 | 0.971, +0.243 |
  | `e25-sharp-1` #2 (20 M) | 0.849, +0.287 | 0.602, +0.076 | 0.581, +0.061 | 0.422, −0.055 | 0.976, +0.243 |

  Against the system actor per kind: the hero battles 0.409, the sieges with a hero 0.414, the
  garrison 0.489 (#3: 0.409, 0.404, 0.485). Against `promoted-1` (its own 10 M network) 0.521.
  The head-to-head against #3 was not played (`e25-sharp-1`'s `best.pt` is on the user's
  machine; 24 MB, uploadable).
- **Training** (`metrics.jsonl`, uploaded, outside git): the bonus 0.0095 at 1 M, 0.005 at 10 M,
  0 at 20 M; the entropy 3.40, 1.93, 1.37, 1.07, 0.84 at 1, 5, 10, 15, 20 M (per gate interval
  2.15 and 1.08; `e25-scale-1` 2.39 and 1.71, `e25-wide-1` 2.39 and 1.52); the KL per update
  0.0135 to 0.0012 and the clip fraction 0.187 to 0.009 (the learning rate falls with it); the
  value's explained variance 0.47 and 0.53 per interval (`e25-scale-1` 0.48 and 0.54); 5 430 and
  5 050 samples a second (`e25-scale-1` 5 700 and 5 290); the league score 0.566 and 0.527; 0.005 %
  of the training battles cut in the second interval. The attacker's share per kind in training
  as `e25-scale-1`'s (0.692, 0.590, 0.946 at 20 M).
- **The read:** the entropy ends at half `e25-scale-1`'s, and the ladder moves by what T9b's
  temperature predicted, no more: every point estimate 0.001 to 0.009 above #3's, five of five
  (each inside its interval), against the system actor 0.422 (#3 at 0.7: 0.428). Training
  sharper puts evaluation's sharpening into the sampled policy; the bonus did not hold back what
  PPO learns. The value head is unchanged.

## T11 — Increment 4's checkpoint (2026-11-02)

- **The user's choices** (AI-Q13): no second A/B (fewer epochs or a larger rollout): increment 4
  closes. **`e25-sharp-1` #2 is the current best** once its head-to-head against `e25-scale-1`
  #3 (T12, the sandbox on its uploaded `best.pt`) is not clearly below 0.5; else #3 stays.
  Increment 5: "the battle lane will push the fair system actor v1 soon"; the agent's reading:
  increment 5 is refined around it when it lands (decision 2: it may train; decision 3: its
  imitation; a fair target beside the faithful one), and E25 waits at the current best until
  then.
- **Increment 4 in one line:** the sampled policy's spread costs about 0.01 against every
  opponent (T9b), and training with the entropy bonus falling to 0 buys exactly that (T10b); with
  increments 1 to 3, every lever tried ends at 0.41 to 0.43 against the faithful system actor.

## T12 — The head-to-head: `e25-sharp-1` #2 against `e25-scale-1` #3 (2026-11-02)

- **The gate** (`h3env.train.gate --candidate runs/e25-sharp-1/best.pt --best
  runs/e25-scale-1/best.pt`, the sandbox's CPU, both `best.pt` uploaded by the user; the gate
  set v1, both seats external, common random numbers): **promoted after 332 pairs**, score
  0.523 (+16 Elo), LLR +2.97 against the bound 2.94 (elo0 0, elo1 20), 61 seconds.
- **The read:** the network trained sharper beats the scale run's best head to head, beside its
  ladder at or above #3's on all five sets (T10b). **`e25-sharp-1` #2 is the current best**
  (the user at T11: its head-to-head not clearly below 0.5), at 20 M samples against #3's 30 M.

## T13 — Increment 5's refinement (2026-11-02)

- **The starting point:** the battle lane's fair system actor v1 (E26 increment 1, #647;
  `E26-log.md` T4): `h3_battle_ai::choose_fair` runs the original's computer on its side's world
  (`BattleState::determinized` without a guess), 0.500 against the all-seeing `choose` on 2 000
  battles of random armies with heroes, +0.8 % Ir per decision without heroes. Where the other
  side has no hero and no hidden pit it is `choose`. In the AI lane `Policy::System` is the
  all-seeing one: the neutral kinds' defender (`encounter.rs`), seat 4 of the batch
  (`H3_SEAT_SYSTEM`), `h3 ai eval`'s `system` and the neutral set's fixed defender, a bot of
  both bots' tables, `batch-system-neutral-v1`'s defender; `run.trainable` refuses any mix with
  it.
- **Found:** the adventure env has no system actor: its neutral seat is played by the caller
  (`Seat::Neutral`; the random rollout plays it), so the battle lane's "the adventure env's
  neutral seat switches" has nothing to switch here; a computer playing it is new work, E13's.
  `e25-sharp-1` #2's `best.pt` is not in this sandbox (a fresh container): T15 needs it uploaded
  again.
- **The user's choices** (AI-Q14, the agent's recommendation on all four): `system` the fair
  actor everywhere and `system-all-seeing` beside it (C ABI v36); exit criterion 1 stays the
  all-seeing actor, the fair one reported; the neutral set's defender the fair actor; the
  neutral kinds back in training at E06 T7's third with the fair defender, no `system_share`;
  a warm start by behaviour cloning on the fair actor's choices, then PPO from it against the
  same mix from scratch. Increment 5's decisions and tasks T14 to T18 in the epic file, their
  scope in the AI queue.
- **Expectation** (the agent's): the fair actor is as strong as the all-seeing one on random
  armies, so `e25-sharp-1` #2 should score about 0.42 against it too (T15 measures). The clone
  starts from an actor that beats the lineage at about 0.58; whether PPO keeps its strength
  from there is T17's question.

## T14 — The switch (2026-11-02)

- **Built** (C ABI v36): `Policy::System` plays `h3_battle_ai::choose_fair` (the neutral kinds'
  defender with it), the new `Policy::SystemAllSeeing` plays `choose`; `H3_SEAT_SYSTEM` (4) the
  fair actor, the new `H3_SEAT_SYSTEM_ALL_SEEING` (5); `batch.py`'s seats `system` and
  `system-all-seeing`; `h3 ai eval`'s policies and default both, the neutral set's defender the
  fair one, the bots' tables' schema 3 (`BOTS_TABLE_SCHEMA`, Rust and Python); the ladder's
  opponents (`evaluate.OPPONENTS`, a run's default `ladder_opponents`) both system actors.
  Tests: `the_system_seats_play_their_named_actor` (h3-sim: each seat plays its actor decision
  by decision on hero encounters, where they differ), the all-seeing seat in the batch's
  determinism property and the C seats' test.
- **Agent decisions** (change freely): a record tells the two apart by what it already names, the
  C ABI version in a run's identity (`system` before v36 the all-seeing one) and the bots'
  table's schema (2 before), rather than a new identity field. `run.trainable(config,
  started_abi)` refuses only `ppo.resume` of a run started before v36 whose mix had a system
  seat (its opponent would change under it); `ppo.train` no longer calls it. No all-seeing twin
  of `batch-system-neutral-v1` (the all-seeing computer's cost is the battle lane's
  `computer-decisions` workload). The adventure env's neutral seat unchanged (T13).
- **The bots' tables regenerated** (release, the sandbox: neutral 26 s, report 241 s against
  about 15 s and 85 s before: 25 ordered pairs, two system actors). Report set: the fair actor
  against the all-seeing one 0.500 ± 0.019, value −0.001; Glicko-2 1658.0 and 1657.2 (± 9.5);
  each bot's rate against either within 0.002 of the other (Random 0.123 and 0.121, Aggressor
  0.320 and 0.318, Greedy 0.347 and 0.345). Neutral set with the fair defender (old, all-seeing
  defender, in brackets): the system actor attacking 0.991 at +0.285 (0.991, +0.285: **the
  neutral target stays +0.285**), Aggressor 0.963 (0.967), Greedy 0.957 (0.956), Random 0.741
  (0.751); the all-seeing actor attacking the fair defender 0.991, +0.285.
- **The neutral bench** (`batch-system-neutral-v1`, callgrind): 6 755 060 962 Ir against
  6 808 294 645, −0.78 %, re-baselined (its defender is the fair actor). `random-rollouts-full`
  +0.01 %, `cargo xtask determinism` passes.
- **For T17's runs:** a run's ladder plays one more opponent, the slowest kind: about a fifth
  more evaluation time (T2b's shaping run: 25 of its 90 minutes).

## T15 — The ladder at the switch (2026-11-02)

- **The run** (the sandbox's CPU, C ABI v36, `e25-sharp-1` #2's `best.pt` uploaded again by the
  user; `python -m h3env.train.ladder --checkpoint runs/e25-sharp-1/best.pt --set report-v1
  --opponents system,system-all-seeing`, then `--set neutral-v1`; the sampled policy;
  `python/records/inspect/e25-sharp-1-fair-report-v1.json`, `-fair-neutral-v1.json`): 157 s
  and 22 s.
- **Report set:** against the fair system actor **0.424 ± 0.018, value −0.055** (hero 0.417,
  siege_hero 0.408, siege_garrison 0.486); against the all-seeing one 0.425 ± 0.018, −0.055
  (its record's final ladder on the user's GPU: 0.422, −0.055). Two episodes skipped of 3 000
  in each (as `h3 ai eval` does not). **The fair actor is no easier for the lineage:** E25's
  gap stays about 0.075 win rate and 0.055 value whichever system actor is the target.
- **Neutral set with the fair defender:** 0.969 ± 0.014, **value +0.245** (wandering 0.972,
  guards 0.960, siege_neutral 0.878); its record against the all-seeing defender 0.976, +0.243.
  **The target is +0.285** (the fair system actor in the player's seat, T14): a gap of 0.040,
  as before the switch.
- **The read:** increment 5's starting point is the old one under the new names; the A/B's
  (T17) control is `e25-neutral-1`, the same config with the neutral kinds back.

## T16 — The cloning bench (2026-11-02)

- **Built** (C ABI v37): `BattleBatch::choice` (`h3_batch_choice`, `h3env.BattleBatch.choice`):
  an internal seat's choice for each env's side to move, encoded as the action an external seat
  plays (`action::encode`, the entry and the cast heads), drawn from that side's seat stream as
  the seat inside the call would; it returns the choices no batch action plays (the encoding
  cannot express them, or an escape). Tests: `external_seats_playing_a_policys_choices_play_its_
  episodes` (h3-sim: every policy's choices, played by both external seats, replay the episodes
  it plays from its seat streams bit for bit, on every encounter kind), the ABI's against the
  Rust batch's, `smoke.c`, and Python's (a seat's choice played outside gives the episodes it
  plays inside). The clone mode of the distillation bench (`h3env.train.clone`, `distill
  --teacher system`, `configs/distill/e25-clone.toml`; `python/README.md`): the fair actor plays
  every external seat of a run config's distribution, kept by whole episodes; a student learns
  the teacher's entry and cast heads by cross-entropy and its value on the returns by Monte Carlo
  at the run's settings (reward v1, the truncation penalty, shaping: PPO's targets at λ 1);
  held-out cross-entropy, agreement per category and of the cast heads, the value; the student
  saved as a run's network file (`--init-from`), its ladder on the report and neutral sets.
  `configs/e25-neutral.toml`: T17's mix (`e25-sharp.toml` with `e06-t7.toml`'s kinds), the
  bench's distribution. Tests: `tests/test_clone.py`.
- **Agent decisions** (change freely): the call takes any internal seat, not only the system
  actors (the same code; the all-seeing one is refused as a teacher, decision 3); the seat stream
  rather than a caller's seed, so the clone's episodes are the fair actor's own and the Rust
  test can compare them with it. The samples are whole episodes (no value bootstrap), the
  student's file carries the bench's seed (the ladder's held-out check), the run config's
  `system_share` is set to 0 (the teacher plays every seat anyway). The tactics bound is not
  applied to the teacher's rows; none of its decisions fell past it (`past_bound` 0).
- **The run** (the user's choice: 4 M samples on the sandbox's CPU; `python/records/distill/
  clone-1.json`; commit `c04ad91`, clean): 40 minutes (the teacher 9, the student's 2 passes
  22, the ladder 5, the held-out set and the evaluations the rest). The teacher: 4.03 M decisions, **none lost**,
  77 439 episodes, 13 truncated; its attacker won 0.708 of the hero kind, 0.591 siege_hero,
  0.946 siege_garrison, 0.993 wandering, 0.988 guards, 0.889 siege_neutral. The held-out set
  (25 600 decisions of 446 episodes, seed 8): its value targets' variance 0.030.

  | Samples | CE entry | CE casts | Agreement | Tactics | Moves | Melee | Shots | Casts | Value |
  |---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | 0.53 M | 1.783 | 2.313 | 0.467 | | 0.225 | 0.287 | 0.546 | | 0.315 |
  | 2.0 M | 1.600 | 2.087 | 0.506 | | 0.273 | 0.325 | 0.588 | | 0.554 |
  | 4.0 M | 1.514 | 1.973 | 0.531 | 0.788 | 0.298 | 0.360 | 0.608 | 0.841 | 0.594 |

  (Cross-entropy in nats per decision, the casts' summed per cast; the value's explained
  variance against the returns.) The cast heads' agreement along the teacher's: spell 0.679,
  target 0.609, second 0.997. Still falling slowly at 4 M (the entry's cross-entropy −0.016 over
  the last 0.5 M).
- **Its ladder** (the sampled policy): against the fair system actor **0.396 ± 0.018, value
  −0.084** (hero 0.386, siege_hero 0.376, siege_garrison 0.476); against the all-seeing one 0.392
  ± 0.018, −0.085; the neutral set **+0.244** (0.974; wandering 0.978, guards 0.974,
  siege_neutral 0.850; the target +0.285).
- **The read:** the clone plays between `t7-1` (0.372, −0.103) and the current best
  `e25-sharp-1` #2 (0.424, −0.055, +0.245, T15) without any RL, and meets #2 on the neutral set:
  a warm start well above a scratch network. It copies the fair actor only loosely: about half
  its entries, the moves (0.30) and the melee positions (0.36) least, where the original's
  computer picks one of many near-equal hexes by rules (and draws) an MLP over observation v22
  does not see; the tactics phase (0.79) and the casts (0.84) best. A clone that plays the fair
  actor exactly would score 0.5 against it. How much of the gap is the teacher's own draws is
  not measured (a diagnostic for T18: the fair actor's agreement with itself on the same
  states). T17 starts `e25-clone-1` from this student (`runs/clone-1/students/mlp.pt`, sent to
  the user).

## T17 — The A/B: the neutral kinds back, from scratch and from the clone (2026-11-03)

- **The runs** (the user's machine, the GTX 1070; `configs/e25-neutral.toml`, seed 1, 20 M
  samples, one segment each, ended by the budget; C ABI v37, the same engine in both: the
  rulesets' hashes, the system policy 19 and the encodings equal): `e25-neutral-1` from scratch
  (`python/records/e25-neutral-1.json`; commit `dd99747`, clean; 74 minutes, 8.3 of them the
  gates and ladders) and `e25-clone-1` from T16's student (`--init-from runs/clone-1/students/
  mlp.pt`; `python/records/e25-clone-1.json`; commit `a3cbd7d`, clean; 73 minutes, 7.0 of them
  evaluation). Both promoted at both gates, best #2: the control at 10 M 0.870 in 23 pairs
  (against its random start), at 20 M 0.531 in 162 (+21 Elo); the clone at 10 M 0.572 in 87
  (against the student, +50 Elo), at 20 M 0.541 in 115 (+29 Elo).
- **The final ladders** (the GPU, 500 per kind: ±0.013 against Random, ±0.018 otherwise, ±0.008
  on the neutral set), beside the current best's (`e25-sharp-1` #2: T10b, its fair columns and
  neutral set T15) and the student's (the clone's ladder #0, 250 per kind, ±0.025):

  | Network | Random | Aggressor | Greedy | Fair system | All-seeing | Neutral set |
  |---|---|---|---|---|---|---|
  | T16's student | 0.811, +0.261 | 0.566, +0.047 | 0.549, +0.027 | 0.383, −0.087 | 0.394, −0.086 | 0.979, +0.243 |
  | `e25-sharp-1` #2 | 0.849, +0.287 | 0.602, +0.076 | 0.581, +0.061 | 0.424, −0.055 | 0.422, −0.055 | 0.969, +0.245 |
  | `e25-neutral-1` #2 | 0.843, +0.286 | 0.589, +0.069 | 0.565, +0.051 | 0.411, −0.064 | 0.410, −0.064 | 0.984, +0.269 |
  | `e25-clone-1` #2 | 0.848, +0.298 | 0.635, +0.112 | 0.609, +0.088 | **0.454, −0.029** | **0.454, −0.030** | 0.986, **+0.282** |

  Against the fair actor per kind: the clone run 0.452, 0.439, 0.491 (hero, siege_hero,
  siege_garrison), the control 0.395, 0.409, 0.481, `e25-sharp-1` #2 0.417, 0.408, 0.486. The
  neutral set per kind: 0.990, 0.982, 0.868 (wandering, guards, siege_neutral), the control
  0.988, 0.976, 0.862, #2 0.972, 0.960, 0.878. Against the fair actor at each ladder (250 per
  kind): the clone run 0.383, 0.431, 0.440 at 0, 10, 20 M; the control 0.134, 0.389, 0.422.
- **Training** (`metrics.jsonl`, uploaded, outside git): the entropy at 1, 5, 10, 15, 20 M 2.01,
  1.72, 1.33, 1.05, 0.76 from the student, 3.52, 1.93, 1.46, 1.21, 0.84 from scratch
  (`e25-sharp-1` 3.40, 1.93, 1.37, 1.07, 0.84); the value's explained variance per gate interval
  0.67 and 0.70 from the student, 0.55 and 0.63 from scratch (`e25-sharp-1` 0.47 and 0.53: the
  neutral episodes are easier to predict, the student's value head a start); the KL per
  interval 0.0135 and 0.0053 against 0.0112 and 0.0051; 5 520 and 4 880 samples a second against
  5 600 and 4 800 (the fair defender costs nothing visible); truncations 0.004 % in the clone
  run's second interval, none in the control's. The attacker's share in training at 20 M per kind
  (hero, siege_hero, siege_garrison, wandering, guards, siege_neutral): 0.682, 0.559, 0.933,
  0.986, 0.981, 0.874 from the student, 0.692, 0.591, 0.941, 0.980, 0.977, 0.863 from scratch.
- **The head-to-heads** (T18's; `h3env.train.gate` on the uploaded `best.pt`, the sandbox's CPU;
  the gate set v1, elo0 0, elo1 20): `e25-clone-1` #2 against `e25-sharp-1` #2 **promoted after
  70 pairs, 0.564 (+45 Elo)**, LLR +3.05, 25 s; against `e25-neutral-1` #2 promoted after 122
  pairs, 0.541 (+29 Elo), 53 s; `e25-neutral-1` #2 against `e25-sharp-1` #2 rejected after 48
  pairs, 0.479 (−14 Elo), 20 s.
- **The read:**
  - *The neutral kinds back* (the control against `e25-sharp-1` #2): the neutral set +0.269
    (+0.024), the report set lower against every opponent by 0.006 to 0.016 (each inside its
    interval), head to head 0.479: T2b's trade, the other way round.
  - *The warm start* (the clone run against the control, the same mix and length): +0.043
    against both system actors, +0.046 and +0.044 against Aggressor and Greedy, the same against
    Random, +0.013 on the neutral set; head to head 0.541. PPO kept the student's strength and
    added 0.06 to it; the student's lead holds at 10 M and at 20 M.
  - **`e25-clone-1` #2 is the first network past the 0.41 to 0.43** every earlier lever of E25
    ended at: 0.454 against both system actors (+0.030 over `e25-sharp-1` #2), its value −0.030
    (−0.055), and on the neutral set +0.282 against the target +0.285 (inside its ±0.008).
    Exit criterion 1 is not met: 0.046 short of 0.5 against the all-seeing actor and the value
    still negative; the neutral target within its interval. The fair and the all-seeing
    columns stay equal (T15).

## T18 — Increment 5's checkpoint (2026-11-03)

- **The user's choices** (AI-Q15, the agent's recommendation on both): **`e25-clone-1` #2 is
  the current best** (T11's rule: its head-to-head against `e25-sharp-1` #2 0.564, promoted);
  **increment 6 is better imitation**: the teacher's ceiling first (how often the fair actor
  repeats its own choice), a longer clone, an imitation loss decaying during PPO, then PPO from
  them against `e25-clone-1`. Increment 5 closes (`E25-archive.md`); increment 6's decisions
  and tasks T19 to T23 in the epic file, their scope in the AI queue.
- **Increment 5 in one line:** the fair system actor plays as strongly as the all-seeing one
  against the lineage (T15), the neutral kinds back in training buy +0.024 on the neutral set
  for a little on the report set (T17's control), and a warm start by cloning the fair actor
  moves every column past E25's earlier ceiling (T17: 0.454, +0.282).

## T19 — The teacher's ceiling (2026-11-03)

- **Built:** `h3env.train.ceiling` (`python/README.md`; tests `tests/test_ceiling.py`): on the
  cloning bench's held-out distribution (`configs/distill/e25-clone.toml`: `e25-neutral.toml`'s
  kinds, the fair actor in every external seat, seed 8) each state's choice drawn k times and the
  last played; by the category of each draw: `mode` (the other draws' most frequent entry is it, a
  tie credited by its share: a clone of the most likely entry), `pair` (the share of the other
  draws equal to it: a clone that samples the teacher), the state's entropy (plug-in and
  Miller-Madow: the clone's cross-entropy floor), the cast heads' `mode` along its choice; each
  `--network`'s most likely entry and cross-entropy against every draw. No Rust change:
  `BattleBatch::choice` draws from the side's `seat_rngs` and advances it, so a second call on the
  unchanged state draws again. Cross-checked against the bench's own `clone.evaluate` on its
  held-out episodes: `e25-clone-1` #2's agreement 0.263 and cross-entropy 7.72 there, 0.270 and
  7.75 here.
- **The run** (the sandbox's CPU; `python/records/distill/ceiling/ceiling-1.json`; commit
  `22ffc03`, clean, C ABI v37, system policy 19): 25 600 states of 620 episodes, 8 draws each
  (204 800 scored), none lost, 37 s; run twice, identical. The categories' shares as T16's
  held-out set's (melee 36 %, tactics 20 %, moves 14 %, shots 13 %, casts 10 %). Agreement with
  the teacher's draw (T16's student: its record's last evaluation, the same distribution):

  | Category | Draws | Mode | Pair | Entropy | T16's student | `e25-clone-1` #2 | `e25-neutral-1` #2 | `e25-sharp-1` #2 |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|
  | tactics | 40 016 | 0.995 | 0.994 | 0.008 | 0.788 | 0.142 | 0.034 | 0.025 |
  | wait, defend | 9 886 | 0.992 | 0.988 | 0.016 | 0.485 | 0.100 | 0.049 | 0.011 |
  | move | 28 330 | 0.985 | 0.984 | 0.021 | 0.298 | 0.041 | 0.013 | 0.014 |
  | melee | 74 456 | 0.941 | 0.930 | 0.094 | 0.360 | 0.213 | 0.182 | 0.173 |
  | shot | 27 569 | 0.998 | 0.998 | 0.003 | 0.608 | 0.384 | 0.287 | 0.315 |
  | heal | 1 784 | 1.000 | 1.000 | 0.000 | 0.699 | 0.655 | 0.516 | 0.628 |
  | cast | 21 159 | 0.977 | 0.970 | 0.042 | 0.841 | 0.908 | 0.886 | 0.934 |
  | other | 1 600 | 0.979 | 0.981 | 0.026 | 0.924 | 0.460 | 0.058 | 0.032 |
  | **all** | 204 800 | **0.972** | 0.967 | **0.044** | **0.531** | 0.270 | 0.212 | 0.215 |

  (Entropy in nats, plug-in; Miller-Madow 0.049 overall.) The cast heads' mode along the
  teacher's choice: spell 0.944, target 1.000, second 0.999 (T16's student 0.679, 0.609, 0.997).
  The cross-entropy on the teacher's entry: T16's student 1.514; the PPO networks 7.7 to 8.8.
- **The read:**
  - **The fair actor is almost deterministic:** its most likely entry is its draw 0.972 of the
    time, 0.94 or more in every category (melee the lowest: near-equal hexes), at 0.044 nats.
    T16's question is answered: **the clone's gap is the network's, not the teacher's draws.**
    The student reached 0.531 of 0.972 (moves 0.30 of 0.985, melee 0.36 of 0.941, shots 0.61 of
    0.998), its cross-entropy 1.514 against a floor near 0.05.
  - **PPO leaves the teacher at once:** `e25-clone-1` #2 agrees 0.270 with it, the scratch
    networks 0.21; PPO kept the start's strength (T17) but not its choices (the tactics phase
    0.79 to 0.14, moves 0.30 to 0.04; only the casts stay, 0.91).
  - **For T20:** the room is large (0.44 of agreement), but T16's curve gains slowly (the entry's
    cross-entropy 1.783, 1.600, 1.514 at 0.5, 2, 4 M: about −0.09 a doubling), so 16 M at T16's
    settings may reach about 0.57; the MLP over observation v22 may be what limits it (the moves
    and melee hexes the original's rules pick). **For T21:** an imitation loss holds PPO nearer a
    teacher it leaves; whether that nearness is worth anything is T22's question.

## T20 — The longer clone (2026-11-03)

- **The run** (the user's machine, the GTX 1070, the teacher on its 24 threads;
  `python/records/distill/clone-2.json`; commit `b163d20`, #682's merge, clean, C ABI v37, system
  policy 19; `configs/distill/e25-clone-2.toml`: T16's bench at 16 M samples, the learning rate
  falling over them): the teacher 16.0 M decisions in 307 852 episodes, none lost, 66 truncated,
  814 s; the student's passes 710 s; the held-out set T16's (16 384 decisions, seed 8).

  | Samples | CE entry | Agreement | Tactics | Wait, defend | Moves | Melee | Shots | Casts | Value |
  |---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | 2 M | 1.603 | 0.506 | 0.773 | 0.479 | 0.280 | 0.336 | 0.582 | 0.776 | 0.543 |
  | 4 M | 1.522 | 0.527 | 0.794 | 0.475 | 0.305 | 0.361 | 0.593 | 0.811 | 0.565 |
  | 8 M | 1.431 | 0.552 | 0.816 | 0.499 | 0.375 | 0.381 | 0.630 | 0.777 | 0.588 |
  | 12 M | 1.379 | 0.565 | 0.829 | 0.519 | 0.380 | 0.388 | 0.646 | 0.807 | 0.639 |
  | 16 M | 1.344 | **0.576** | 0.836 | 0.542 | 0.378 | 0.401 | 0.660 | 0.831 | 0.633 |
  | T19's ceiling | ≈0.05 | 0.972 | 0.995 | 0.992 | 0.985 | 0.941 | 0.998 | 0.977 | |

  (T16 at 4 M: 1.514, 0.531, its learning rate at its floor by then.) The cast heads at 16 M:
  spell 0.694, target 0.640, second 0.996 (the ceiling 0.944, 1.000, 0.999).
- **Its ladder** (the user's GPU, 500 per kind): against the fair system actor **0.411 ± 0.018,
  value −0.069** (hero 0.404, siege_hero 0.392, siege_garrison 0.481); against the all-seeing one
  0.411, −0.071; the neutral set **+0.251** (0.982; wandering 0.986, guards 0.972, siege_neutral
  0.864). T16's student: 0.396, −0.084; 0.392, −0.085; +0.244.
- **The read:** four times the samples buy +0.045 of agreement (0.531 to 0.576, as T19's slope
  said) and +0.015 against both system actors (inside the intervals): the student alone plays at
  the control's level (`e25-neutral-1` #2, 20 M of PPO: 0.411, −0.064), below the current best
  (0.454). The curve still rises, but by about 0.02 a doubling: the ceiling (0.972) is out of
  this MLP's reach by samples. The gap stays where T16 left it, the hexes (moves 0.38 of 0.985,
  melee 0.40 of 0.941) and the casts' targets (0.64 of 1.000); the tactics phase and the shots
  gain most. T22's student is this one (`runs/clone-2/students/mlp.pt`, on the user's machine):
  a start +0.015 above `e25-clone-1`'s.

## T21 — The imitation loss (2026-11-03)

- **Built:** `[ppo] imitation_coef` (0, off, by default) and `imitation_horizon` (0: the
  learning rate's horizon): the weight falls linearly to 0 (`run.imitation_weight`). While it is
  above 0, each rollout call asks the fair system actor (`ppo.TEACHER`, moved from `clone`) for
  every env's choice before the step (`BattleBatch.choice`, C ABI v37), and `Trainer.label`
  keeps it on the learning rows (a league member's rows and the internal defender's get none)
  whose choice a batch action plays and whose entry the learner's mask (its tactics bound's)
  holds, with the cast heads' masks along the teacher's choice. The update adds the weight times
  the mean over a minibatch's labelled rows of −log π(the teacher's action) (the entry, a cast's
  heads), through the same pass of the trunk as PPO's terms (`Policy.evaluate` split into
  `entries` and `along`). The update records gain `imitation_coef`, and while it is above 0
  `imitation_loss`, `teacher_rows`, `teacher_lost`, `teacher_agreement` (the learner's sampled
  entry the teacher's) and `teacher_s`; the curve averages the last three over the updates that
  have them; the update line shows the weight, the cross-entropy and the agreement.
  `configs/e25-imitate.toml`: T22's arm. Tests: three in `test_train.py`, four in `test_run.py`;
  the Python suite 197 passed.
- **Decision 3's question** (do the teacher's calls consume draws the learner's episode would
  see?): **no.** `BattleBatch::choice` draws from the side to move's seat stream; at a call the
  side to move is always an external seat (the internal ones play inside `step`), whose stream
  nothing else reads, and `begin` seeds both streams afresh per episode. Tested: a rollout with
  labels is the one without, bit for bit (observations, actions, sides, rewards), on a mix with a
  league member and the internal defender, and its first call's labels are the fair actor's
  choices on a twin batch. At weight 0 no teacher is asked: three rollouts and updates of a fixed
  config give the same digest (weights, statistics, rollouts) on `master`'s code and the branch.
- **Agent decisions** (change freely): the horizon defaults to the learning rate's; the call
  takes every env (the C call selects no rows: a member's rows cost the teacher too); the loss
  rides on PPO's minibatches (the closed transitions) and averages over their labelled rows; an
  entry past the tactics bound labels nothing; the label is the teacher's choice (a one-hot
  cross-entropy, as T16's clone: the fair actor gives a choice, not a distribution); neither key
  resumes changed.
- **The CPU check** (the sandbox's 4 cores; commit `3dd8055`, clean, C ABI v37, system policy
  19; `python/records/imitate/t21-check-*.json`): a clone of 0.53 M samples (T16's bench, its
  held-out agreement 0.475), then 0.41 M samples of PPO from it at `e25-neutral.toml`'s settings
  with the weight 0, 0.5 and 2 (the horizon 10 M: nearly constant), the four networks scored by
  T19's tool against the fair actor's draws (25 600 states, 8 draws each; the learners from their
  `checkpoint.pt`):

  | Network | All | Tactics | Wait, defend | Moves | Melee | Shots | Casts | CE | Entropy | Samples/s |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | the clone | 0.466 | 0.731 | 0.424 | 0.192 | 0.295 | 0.515 | 0.831 | 1.806 | | |
  | weight 0 | 0.422 | 0.711 | 0.367 | 0.128 | 0.257 | 0.437 | 0.808 | 2.011 | 1.89 | 1 475 |
  | weight 0.5 | 0.450 | 0.660 | 0.244 | 0.219 | 0.286 | 0.508 | 0.893 | 1.975 | 2.32 | 1 139 |
  | weight 2 | 0.453 | 0.650 | 0.242 | 0.227 | 0.294 | 0.512 | 0.905 | 1.972 | 2.30 | 1 185 |

  (Agreement of the most likely entry with the teacher's draws; CE on them in nats; the entropy
  the last update's; samples a second from the second update on.) On the labels (the learner's
  own states) the cross-entropy fell 2.53 to 2.32 (weight 0.5) and 2.46 to 2.26 (2); the
  sampled agreement 0.24 to 0.26 and 0.27; no choice lost, every learning row labelled. **The
  teacher's cost:** 120 to 127 µs a decision on the batch's 4 threads, 14 % of a run's time;
  the update 10 % longer (the labels' pass of the heads); throughput −20 to −23 %. On the user's
  machine (T20: the teacher with its step 51 µs a decision on 24 threads, PPO about 200 µs a
  sample) an estimated +10 to 25 % over the horizon only (`teacher_s` measures it in T22's
  record).
- **The read:** the loss works as designed and holds the learner nearer the teacher (agreement
  −0.013 to −0.016 over 0.4 M of PPO, −0.044 without; moves, melee, shots and casts kept or
  gained), but on the teacher's own states the cross-entropy still rises (1.81 to 1.97 with it,
  2.01 without): the labels sit on the learner's states, which drift from the teacher's. Weight
  2 is weight 0.5: the gradient's norm clip (0.5) bounds the step, the weight sets only the mix
  of its direction. Two side effects for T22's read: the entropy stays up (2.3 against 1.9; a
  cross-entropy to labels the network cannot fit spreads its policy, where PPO and the entropy
  bonus's floor 0 sharpen it; T9b: sharpening is worth about 0.01), and the tactics phase and
  wait or defend lose more agreement with the loss than without (0.73 to 0.65 and 0.42 to 0.24,
  against 0.71 and 0.37), unexplained at this length. A 0.5 M clone and 0.4 M of PPO on the CPU
  say nothing of strength.
- **T22's arms** (proposed; fixed with the user when prepared): `e25-clone-2`
  (`e25-neutral.toml`) and `e25-imitate-1` (`e25-imitate.toml`: weight 0.5, horizon 10 M, half
  the run; the teacher no longer asked in the second half), both `--init-from` T20's student
  (`runs/clone-2/students/mlp.pt`), the commands in the config's header; T19's tool scores both
  arms' final networks against the teacher beside their ladders.

## T22 — The A/B: from T20's student, with and without the imitation loss (2026-11-04)

- **The runs** (the user's machine, the GTX 1070; `configs/e25-neutral.toml`'s mix, seed 1,
  20 M samples, one segment each, ended by the budget; both `--init-from` T20's student
  `runs/clone-2/students/mlp.pt`; commit `783e1af` (#712's merge), clean; C ABI v37, the system
  policy 19, the encodings and the battle ruleset's hash T17's, only the adventure ruleset's moved,
  which no battle reads; the battle lane's `049cbb8`, merged after T17's runs, gives a siege's town
  morale as one building's bonus, the same value, the bots' tables unchanged): `e25-clone-2` without the loss
  (`python/records/e25-clone-2.json`; 72 minutes, 6.8 of them evaluation) and `e25-imitate-1` with
  it (`configs/e25-imitate.toml`: weight 0.5 falling to 0 at 10 M, fixed with the user at T22's
  preparation; `python/records/e25-imitate-1.json`; 77 minutes, 5.9 evaluation). Both promoted at
  both gates, best #2: `e25-clone-2` at 10 M 0.535 in 201 pairs (+24 Elo), at 20 M 0.524 in 248
  (+17); `e25-imitate-1` 0.568 in 81 (+47), 0.528 in 232 (+19).
- **The final ladders** (the GPU, 500 per kind: ±0.012 against Random, ±0.018 otherwise, ±0.019
  against the system actors, ±0.010 on the neutral set), beside the control (T17):

  | Network | Random | Aggressor | Greedy | Fair system | All-seeing | Neutral set |
  |---|---|---|---|---|---|---|
  | `e25-clone-1` #2 (control) | 0.848, +0.298 | 0.635, +0.112 | 0.609, +0.088 | 0.454, −0.029 | 0.454, −0.030 | 0.986, +0.282 |
  | `e25-clone-2` #2 | 0.858, +0.303 | 0.650, +0.121 | 0.623, +0.096 | 0.451, −0.026 | 0.450, −0.028 | 0.984, +0.284 |
  | `e25-imitate-1` #2 | 0.858, +0.305 | 0.651, +0.124 | 0.626, +0.099 | **0.458, −0.020** | **0.462, −0.022** | 0.983, **+0.287** |

  Against the fair actor per kind (hero, siege_hero, siege_garrison): `e25-clone-2` 0.439, 0.451,
  0.497, `e25-imitate-1` 0.447, 0.462, 0.493 (the control 0.452, 0.439, 0.491). The neutral set
  per kind (wandering, guards, siege_neutral): 0.988, 0.978, 0.880 and 0.986, 0.982, 0.884 (0.990,
  0.982, 0.868). Against the fair actor at each ladder (250 per kind) at 0, 10, 20 M:
  `e25-clone-2` 0.409, 0.439, 0.458; `e25-imitate-1` 0.409, 0.441, 0.467; the control 0.383,
  0.431, 0.440.
- **Training** (`metrics.jsonl`, uploaded, outside git): the entropy at 1, 5, 10, 15, 20 M 1.71,
  1.53, 1.29, 1.07, 0.80 without the loss, 1.78, 1.78, 1.61, 1.10, 0.88 with it (the control 2.01,
  1.72, 1.33, 1.05, 0.76); the value's explained variance per gate interval 0.69 and 0.71, 0.74
  and 0.74 (0.67, 0.70); the KL 0.0148 and 0.0055, 0.0137 and 0.0060; 5 150 and 5 120 samples a
  second without the loss, 4 390 and 5 090 with it (the teacher 0.32 of the rollout's time at
  1 M, 0.21 to 0.23 from 5 M; the run 5.7 minutes, 9 %, longer); truncations 0.002 % and 0.013 %
  without the loss, none with it. The imitation terms (weight 0.45, 0.25, near 0 at 1, 5, 10 M):
  the cross-entropy on the learner's own states 1.82, 1.87, 2.27, the learner's sampled entry the
  teacher's 0.38, 0.38, 0.35; no label lost.
- **T19's tool** (the sandbox's CPU; `python/records/distill/ceiling/ceiling-2.json`; commit
  `5dac0d2`, clean; T19's states, 25 600 with 8 draws each, its ceiling reproduced: 0.972, 0.044
  nats; the uploaded control's `best.pt` identified by it, 0.2705 and 7.745 as T19's):

  | Network | All | Tactics | Wait, defend | Moves | Melee | Shots | Heal | Casts | Other | CE |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | `e25-clone-1` #2 | 0.270 | 0.142 | 0.100 | 0.041 | 0.213 | 0.384 | 0.655 | 0.908 | 0.460 | 7.745 |
  | `e25-clone-2` #2 | 0.276 | 0.119 | 0.098 | 0.063 | 0.221 | 0.397 | 0.686 | 0.912 | 0.732 | 7.252 |
  | `e25-imitate-1` #2 | **0.317** | 0.190 | 0.128 | 0.093 | 0.260 | 0.469 | 0.668 | 0.868 | 0.916 | **4.566** |

  (T20's student, the arms' start: 0.576 on its bench's held-out states.)
- **The head-to-heads** (`h3env.train.gate` on the uploaded `best.pt`, the sandbox's CPU; the
  gate set v1, elo0 0, elo1 20): `e25-imitate-1` #2 against `e25-clone-1` #2 **promoted after
  171 pairs, 0.534 (+23 Elo)**, LLR +2.95, 24 s; `e25-clone-2` #2 against it rejected after 446
  pairs, 0.508 (+6 Elo), LLR −3.43, 91 s; `e25-imitate-1` #2 against `e25-clone-2` #2 rejected
  after 738 pairs, 0.512 (+8 Elo), LLR −2.97, 215 s.
- **The read:**
  - *The longer clone* (`e25-clone-2` against the control, the loss off in both): its start's
    lead (T20: +0.015; the ladder at 0 +0.026) is gone at 20 M, every column inside its interval
    and head to head 0.508, rejected. PPO leaves the better student as it left T16's (agreement
    0.276 against 0.270).
  - *The loss* (`e25-imitate-1` against `e25-clone-2`, the same start): it keeps the network
    nearer the teacher (0.317 against 0.276, the cross-entropy 4.57 against 7.25, every category
    but the heals and casts) and the entropy higher over its horizon; the ladders +0.007 and +0.012
    against the fair and all-seeing actors, +0.003 on the neutral set (inside the intervals), head
    to head +8 Elo, rejected. It is the only arm that promoted against the control (0.534). Its
    cost: 9 % of the run's time.
  - **Increment 6 in one line:** better imitation moved the lineage by about 0.01
    (`e25-imitate-1` #2: 0.458 and 0.462 against the system actors, value −0.020; the neutral set
    +0.287 against the target +0.285, inside its ±0.010): a better clone is capped by the network
    (T20), and PPO drifts from the teacher unless the loss holds it. Exit criterion 1 is not met:
    0.038 to 0.042 short of 0.5 against the system actors and the value still negative.

## T23 — Increment 6's checkpoint (2026-11-04)

- **The user's choices** (AI-Q16, the agent's recommendation on both): **`e25-imitate-1` #2 is
  the current best** (T11's rule: its head-to-head against `e25-clone-1` #2 0.534, promoted);
  **increment 7 is a network that reads the battle's structure, built and tuned properly**,
  measured first on the cloning bench by its agreement with the fair actor (T19 and T20 put the gap
  there: moves 0.38 of 0.985, melee 0.40 of 0.941). The other candidates offered: scale with the
  loss, expert iteration, an exploiter league. Increment 6 closes (`E25-archive.md`); increment
  7's decisions and tasks in the epic file, their scope in the AI queue.

## T24 — The kinds and the bench, built properly (2026-11-04)

- **Built:** `[network] norm` (`"none"`, every network before; `"layer"`, `hexconv`'s only): a
  pre-activation LayerNorm over each field cell's channels before each convolution (then the
  ReLU, the cells off the field set back to 0, as the norm's bias moves them) and once on the
  trunk's output (`hexnet.CellNorm`, `activate`), the residual branch still initialised at 0.
  `[network] feed_forward` (2): the transformer blocks' feed-forward width in multiples of
  `width` (`entity.Block`: the entity network and the hex transformer). The bench (`distill`
  and its clone mode) gains `warmup` (samples; the rate rises linearly from 0, a batch's samples
  counted pro rata over its minibatches, then falls as before) and `weight_decay` (AdamW,
  decoupled, on every parameter of two dimensions or more: the layers' weights, the embeddings,
  the attention's offset bias; not on biases or the norms' gains), for the bench and per student
  beside `lr` (`STUDENT_KEYS`); the training step is one function for both modes (`step`,
  `learning_rate`, `make_student`). Each evaluation holds each student's cost
  (`us_per_sample_pass`: its step seconds over the rows it trained on, each pass counted;
  `peak_memory_mb`: the most CUDA memory a step took above what was allocated before it, none on
  the CPU), the table two columns; the report's students their `warmup` and `weight_decay`;
  cuDNN's autotuner on CUDA (`bench_device`, also `network`'s). `network --time` adds the
  sample-pass (the training pass, the clipping and an AdamW step at rate 0; on CUDA its peak
  memory); `network --bench FILE` puts every student of a bench's file in one table.
  `configs/distill/e25-grid.toml`: decision 2's candidates and the two references, a timing grid
  (not a bench to run). **The defaults keep the past:** a file without the new keys builds the
  same parameters and the hex convolution's block computes T7a's bit for bit (tested against
  its code verbatim); at weight decay 0 the steps are Adam's bit for bit, without a warm-up the
  rate is T7b's exactly (tested); T16's and T20's configs set neither (tested). The Python suite:
  205 passed, 1 skipped.
- **The grid on the CPU** (the sandbox's 4 threads, 128 rows, the median of 15, still noisy by
  about 15 %; `network --bench configs/distill/e25-grid.toml --time --rows 128 --repeats 15`; the
  ratio is the sample-pass's to `hexconv` 64 × 6's, the cap's network):

  | Student | Parameters | Multiply-adds | Forward µs | Training µs | Sample-pass µs | Ratio |
  |---|---:|---:|---:|---:|---:|---:|
  | MLP 2 × 512 | 5.95 M | 5.5 M | 35 | 81 | 259 | 0.12 |
  | `hexconv` 64 × 6 (the cap) | 0.69 M | 85 M | 534 | 2 377 | 2 099 | 1 |
  | `hexconv` normalised 64 × 6 | 0.69 M | 85 M | 749 | 2 965 | 2 368 | 1.13 |
  | … 64 × 8 | 0.85 M | 113 M | 1 100 | 4 081 | 3 733 | 1.78 |
  | … 64 × 10 | 1.02 M | 141 M | 1 114 | 4 800 | 4 619 | 2.20 |
  | … 96 × 6 | 1.45 M | 192 M | 1 463 | 4 975 | 4 749 | 2.26 |
  | … 96 × 8 | 1.82 M | 254 M | 1 555 | 6 326 | 7 732 | 3.68 |
  | … 96 × 10 | 2.19 M | 317 M | 2 466 | 8 539 | 7 321 | 3.49 |
  | … 128 × 6 | 2.50 M | 340 M | 2 139 | 6 681 | 7 157 | 3.41 |
  | … 128 × 8 | 3.16 M | 451 M | 2 823 | 9 353 | 9 050 | 4.31 |
  | … 128 × 10 | 3.82 M | 562 M | 3 050 | 11 165 | 11 391 | 5.43 |
  | `hexformer` 96 × 4 | 0.65 M | 87 M | 903 | 3 352 | 3 551 | 1.69 |
  | … 96 × 4, feed-forward 4 | 0.79 M | 114 M | 1 502 | 4 013 | 4 151 | 1.98 |
  | … 96 × 6 | 0.80 M | 128 M | 1 273 | 6 357 | 6 727 | 3.20 |
  | … 128 × 4 | 1.06 M | 142 M | 1 297 | 4 452 | 5 153 | 2.45 |
  | … 128 × 6 | 1.33 M | 209 M | 1 750 | 8 776 | 9 349 | 4.45 |
  | `entity` 192 × 2 | 2.12 M | 33 M | 243 | 788 | 873 | 0.42 |
  | … 192 × 2, feed-forward 4 | 2.42 M | 46 M | 308 | 932 | 1 071 | 0.51 |
  | … 192 × 4 | 2.72 M | 61 M | 402 | 1 411 | 1 782 | 0.85 |
  | … 256 × 2 | 3.28 M | 56 M | 442 | 1 260 | 1 256 | 0.60 |
  | … 256 × 4 | 4.34 M | 105 M | 1 062 | 2 254 | 2 719 | 1.30 |

  The MLP's sample-pass is mostly its AdamW step over 5.95 M parameters, a step's cost shared by
  128 rows here (512 at the bench). The normalisation's own cost (the trunk of 64 × 6 forward
  and backward, 128 rows, interleaved, the median of 30): none 1 690 µs a row, the permuted
  LayerNorm 1 993 (+18 %), with a contiguous copy after it 2 533, a plain NCHW one 2 848: the
  permuted one is kept (on the GPU it hands the convolution a channels-last tensor; T26's table
  shows whether that costs on the GTX 1070).
- **The read: decision 2's ranges against decision 3's cap.** On the CPU only the entity network
  up to 256 × 2 or 192 × 4 lies below the cap's network; the normalisation puts `hexconv` 64 × 6
  13 to 18 % above it, and every larger `hexconv` and every `hexformer` of decision 2's ranges is
  1.7 to 5.4 times it. The GPU's ratios differ (T7c on the GTX 1070: `entity` 128 × 2 106 µs
  against `hexconv`'s 466, 0.23, where the CPU says about 0.4; `hexformer` 64 × 4 553, already
  above the cap), and T26's GPU table decides (`network --bench configs/distill/e25-grid.toml
  --time --rows 512 --device cuda`, a few minutes). If the GPU agrees, the cap admits the MLP
  and the MLP with the mask, the entity network up to 256 × 4 and `hexconv` normalised at 64 × 6
  only if its normalisation costs little there; no `hexformer` and no larger `hexconv`. That is
  the user's call at T26 (keep the cap, or raise it for a candidate whose agreement pays for
  it).
- **The CPU smoke** (the clone mode on `e25-neutral.toml`'s distribution, 65 640 samples, the
  held-out set 4 096, 2 106 s; the teacher 75 008 decisions, none lost, 1 405 episodes): the MLP
  (as T16), `hexconv` normalised 64 × 6 (lr 1e-3), `hexformer` 96 × 4 with feed-forward 4 (5e-4)
  and `entity` 192 × 2 (5e-4), the three with a warm-up of 16 384 samples and decay 0.01. Every
  student trained, scored and saved; agreement at 65 k 0.379, 0.438, 0.401, 0.423 (too early to
  rank); the µs a sample-pass at the bench's 512 rows on the CPU 119, 2 787, 11 557, 1 097 (the
  hex transformer's attention costs three times its 128-row figure at 512 rows).
- **The grid on the GTX 1070** (the user, 2026-11-04, after T24: T26's first step run early;
  `network --bench configs/distill/e25-grid.toml --time --rows 512 --device cuda`; the ratio is
  the sample-pass's to `hexconv` 64 × 6's, 301 µs here, where T7c's bench measured 466):

  | Student | Forward µs | Training µs | Sample-pass µs | Peak MB | Ratio |
  |---|---:|---:|---:|---:|---:|
  | MLP 2 × 512 | 5.4 | 12.6 | 17.7 | 87 | 0.06 |
  | `hexconv` 64 × 6 (the cap) | 113 | 301 | 301 | 903 | 1 |
  | `hexconv` normalised 64 × 6 | 179 | 413 | 448 | 1 641 | 1.49 |
  | … 64 × 8 | 237 | 539 | 539 | 2 069 | 1.79 |
  | … 64 × 10 | 265 | 649 | 659 | 2 498 | 2.19 |
  | … 96 × 6 | 226 | 595 | 595 | 2 450 | 1.97 |
  | … 96 × 8 | 292 | 766 | 774 | 3 091 | 2.57 |
  | … 96 × 10 | 360 | 945 | 947 | 3 732 | 3.14 |
  | … 128 × 6 | 303 | 796 | 799 | 3 242 | 2.65 |
  | … 128 × 8 | 394 | 1 036 | 1 040 | 4 098 | 3.45 |
  | … 128 × 10 | 484 | 1 289 | 1 291 | 4 952 | 4.28 |
  | `hexformer` 96 × 4 | 145 | 681 | 682 | 1 823 | 2.26 |
  | … 96 × 4, feed-forward 4 | 166 | 1 165 | 766 | 2 067 | 2.54 |
  | … 96 × 6 | 248 | 1 041 | 1 003 | 2 536 | 3.33 |
  | … 128 × 4 | 183 | 982 | 868 | 2 332 | 2.88 |
  | … 128 × 6 | 272 | 1 312 | 1 325 | 3 278 | 4.40 |
  | `entity` 192 × 2 | 43 | 134 | 135 | 724 | 0.45 |
  | … 192 × 2, feed-forward 4 | 52 | 154 | 156 | 792 | 0.52 |
  | … 192 × 4 | 67 | 222 | 221 | 1 064 | 0.73 |
  | … 256 × 2 | 60 | 181 | 177 | 949 | 0.59 |
  | … 256 × 4 | 94 | 287 | 293 | 1 401 | 0.97 |

  The GPU agrees with the CPU's read: within the cap's network only the MLP and the entity
  network up to 256 × 4; no `hexformer` (the smallest 2.3 times) and no larger `hexconv`. The
  normalisation costs more here than on the CPU (+49 % a sample-pass against +13 %, +82 % peak
  memory: the permuted LayerNorm's channels-last tensors). Decision 3's two readings now differ:
  "at most `hexconv` 64 × 6's" on this measure is 301 µs, "466 µs" (T7c's bench, about ten hours of
  network time for 20 M samples of PPO) also admits `hexconv` normalised 64 × 6 (448). The user's
  call at T26, after T25's cost change.

## T25 — The heads read the geometry (2026-11-04)

- **Built:** `h3env.train.geometry`, a port of `Field::attack_positions` and the footprints
  (`h3-battle`'s `geometry.rs`, BAT-GEO-001, -002; `Stack::footprint`'s fallback): `attack_table`
  (each position, shape and side's 24 attack hexes) and `attack_hexes` (an observation's [rows,
  42, 24] attack hexes of the active stack, a stack's side by its owner as in `h3-battle`, so a
  hypnotised one keeps its owner's shape). Its test plays real battles (`field`, `full`, `siege`)
  and checks each legal attack entry in a fork: 18 823 attacks, every category above 7 900 (one-
  and two-hex attackers and targets, both sides), none without a hex, each one's hex the
  attacker's or a legal move's; 8 ended short on the opponent's Quicksand the attacker could not
  see (BAT-MAG-032), the one exclusion (at most 0.5 %). `encode(obs, mask)` and `values(obs,
  mask)` take the rows' entry mask, the one the logits are masked with (`act`'s bounded one; the
  search's; the bench's stored one; PPO's last value the bounded batch mask, the bootstrap of a
  cut battle an empty mask, unused at `truncation_penalty` 1, every E25 config). Three `[network]`
  keys, off by default (older files build and play bit for bit, tested per kind): `mask_input`
  (the MLP: the 1 404 entries as inputs after the observation; the structured kinds: the legal
  moves and the legal attacks' hexes as two hex planes, each slot's 27 entries on its token, the
  15 fixed entries on the global one), `order` for the structured kinds (the slot's three turn
  order columns on its token), `attack_hex` (the structured kinds: a legal melee entry adds a
  score of its target's vector, its attack hex's vector and the context, computed for the legal
  entries only). The Python suite: 238 passed, 1 skipped.
- **The cost on the CPU** (4 threads, 128 rows, the median of 15, noisy by about 15 %; µs a
  sample-pass): `entity` 192 × 2 1 330 without the keys, 1 210 with all three (a rerun in the other
  order 1 121 and 1 137); `hexconv` normalised 64 × 6 2 784, 2 624; `hexformer` 96 × 4 4 896, 5 160;
  the MLP 193, with the mask 311 (+0.72 M parameters, its AdamW step). The keys add 0.2 to 1.3 % of
  the structured kinds' multiply-adds.
- **The smokes** (the sandbox's CPU): a PPO run per kind from `e25-imitate.toml` (2 048 samples,
  three updates, the gate, a promotion and the ladder; the MLP with the mask, the three structured
  kinds with all keys at `entity` 192 × 2, `hexconv` normalised 64 × 6, `hexformer` 96 × 4) 23, 55,
  105 and 179 s; a stopped run resumed to its budget; every network file loads and plays. The
  clone (`e25-clone.toml` with those four students, 49 152 samples, the held-out set 4 096; 1 306
  s): agreement 0.401, 0.451, 0.466, 0.432, too early to rank.
- **T26 prepared:** the user's cap (after the GPU's grid above): **a sample-pass of about 700 µs**
  on the GTX 1070 (`network --time`'s measure), which admits one candidate of each family at the
  largest size of decision 2's range within it. `configs/distill/e25-structure.toml`: the MLP,
  the MLP with the mask, `entity` 256 × 4 (293 µs without the keys), `hexconv` normalised 96 × 6
  (595) and `hexformer` 96 × 4 (682), each with the three keys at two learning rates (5e-4 and
  2e-4; `hexconv` 1e-3 and 3e-4), a warm-up of 131 072 samples, weight decay 0.01. By the grid,
  about 7 to 8 hours (4 M samples, two passes each).

## T26 — The structure sweep: a first segment, stopped (2026-11-05)

- **The user's run** of `configs/distill/e25-structure.toml` on the GTX 1070, stopped after about
  1.5 hours at its first evaluation (`eval_every` 500 k; by the students' training seconds over
  their µs about 0.45 M samples of 4 M). The bench does not resume, so the run is lost but for
  this table; **the user's rule: every run over 2 hours resumes** (T26a makes the bench do so):

  | Student | CE entry | CE cast | Agreement | Moves | Melee | Shots | Value | Seconds | µs | MB |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | MLP 2 × 512 | 1.805 | 2.336 | 0.464 | 0.239 | 0.292 | 0.530 | 0.247 | 21 | 24 | 100 |
  | MLP with the mask | 1.642 | 2.343 | 0.495 | 0.345 | 0.310 | 0.519 | 0.270 | 21 | 24 | 102 |
  | `entity` 256 × 4, 5e-4 | 1.395 | 1.980 | 0.594 | 0.296 | 0.541 | 0.672 | 0.318 | 302 | 340 | 1 451 |
  | … 2e-4 | 1.564 | 2.138 | 0.550 | 0.105 | 0.502 | 0.622 | 0.231 | 301 | 339 | 1 453 |
  | `hexconv` normalised 96 × 6, 1e-3 | **1.136** | **1.884** | **0.619** | 0.414 | 0.513 | 0.593 | 0.330 | 583 | 658 | 3 071 |
  | … 3e-4 | 1.263 | 2.199 | 0.590 | **0.428** | 0.486 | 0.565 | **0.370** | 583 | 658 | 2 455 |
  | `hexformer` 96 × 4, 5e-4 | 1.320 | 2.103 | 0.578 | 0.402 | 0.446 | 0.595 | 0.295 | 657 | 742 | 1 829 |
  | … 2e-4 | 1.475 | 2.346 | 0.541 | 0.384 | 0.418 | 0.555 | 0.370 | 659 | 743 | 1 829 |

- **The read (early, one evaluation):** every structured student is ahead of both MLPs by 0.05
  to 0.15 agreement at about an eighth of the budget (the MLP 0.464 here, 0.531 at 4 M in T16);
  `hexconv` at 1e-3 already passes decision 5's gate value (0.60, due at 4 M). The melee hexes
  gain most (0.42 to 0.54 against 0.29 to 0.31: T25's attack hex), the moves least (`entity` at
  2e-4 0.105). The higher of each family's two rates leads in agreement. The mask alone gains the
  MLP 0.03 (its moves 0.24 to 0.35). The keys' cost on the GPU: `entity` 340 µs against the
  grid's 293, `hexconv` 658 against 595, `hexformer` 742 against 682 (the cap about 700).
- **The time:** about 1.5 hours for the first 0.5 M (the teacher's play, the held-out set and
  the evaluation beside 52 minutes of the students' steps), so the whole bench is about 10 to 12
  hours, not T25's 7 to 8 (which counted the students only).

## T26a — The resumable bench (2026-11-05)

- **The user's rule** (2026-11-05, after T26's lost first segment): every run over 2 hours
  resumes. Both bench modes now run one loop, `distill.go`, which a mode hands its parts
  (`distill_parts`, `clone.parts`: the teacher's play of the segment and of the held-out set,
  the loss, the scores, the table, the clone's ladders as `finish`). A new bench draws as before
  (the same seeds, minibatch orders and schedule; the old tests pass unchanged but for the
  refusal's wording).
- **The checkpoint** (`checkpoint.pt`, `BENCH_FORMAT` 1, not the trainer's): at a batch boundary
  every `checkpoint_minutes` (a new key, 10 by default), after each evaluation, at a stop and at
  the end; each student's network, AdamW state, seconds, rows and peak memory, the samples, the
  next evaluation, the learning rate's horizon (the budget at the start, kept as the trainer's),
  the run config, the report so far (JSON) and the teacher's statistics (JSON), the metrics'
  length, torch's generators. Written once: the held-out set (`heldout.npz`, compressed) and a
  network teacher (`teacher.pt`). `report.json` is rewritten at every checkpoint, with
  `segments` (each one's seed, start, commit, device, samples at its start, `--set`s, end).
- **The stop:** Ctrl-C (`run.Stop`) is taken at the next minibatch or teacher's batch call: a
  batch the students were training on counts its samples pro rata of the steps taken (as the
  warm-up's `seen`), one the teacher was still playing is dropped; then the evaluation, the
  students' files and the checkpoint. A second Ctrl-C aborts at once. A stop before or during the
  clone's ladders leaves the unplayed ones pending (each saved as it ends).
- **The resume:** `python -m h3env.train.distill --resume DIR` (`distill.resume`), the settings and
  teacher the checkpoint's; `--set` changes only `RESUMABLE` (`samples`, `threads`, `device`,
  `checkpoint_minutes`; others refused before anything changes); a finished bench only with a
  larger budget. The segment's teacher plays on `run.segment_seed(seed, segment)` (the open
  episodes of the stop dropped), the metrics are cut back to the checkpoint, the clone's
  statistics continue.
- **Tests:** `test_a_stopped_bench_resumes_to_its_budget` (distill: a stop at an evaluation, a
  refused `--set lr`, a resume stopped at once saving the students as restored, a resume to a
  larger budget, three segments), `test_a_stopped_clone_bench_resumes_its_training_and_its_ladder`
  (a stop in training, one at the ladder, the ladder played at the last resume),
  `test_a_stop_inside_a_batch_counts_its_samples_pro_rata`. The Python suite 241 passed.
- **The smoke** (the sandbox's CPU): T25's clone smoke (`e25-clone.toml`, its four students, 49 152
  samples, the held-out set 4 096) with `rollout` 32 (batches of about 8 192), an evaluation every
  16 384 and a checkpoint every 2 minutes, by the command line: SIGINT at 576 s, 20 s into a
  batch; stopped 16 s later at 17 615 samples (785 of that batch pro rata), resumed with
  `--resume` to 51 334 in segment 1; 1 614 s in all. Agreement 0.393, 0.438, 0.419, 0.426 (T25's
  0.401, 0.451, 0.466, 0.432 at its own batch size; too early to rank). The checkpoint 124 MB;
  the held-out set 156 MB as written then, 2.5 MB compressed (1.2 s; compressed since).
- **For T26b:** the structure sweep's eight students hold 23.9 M parameters, a checkpoint about
  300 MB; its commands in parts in `QUEUE.md`. Dropping a student needs a new bench (a resume
  keeps the students), so the lower rates' call comes before the first part.

## T26b — The structure sweep (2026-11-05)

- **The user's run** of `configs/distill/e25-structure.toml` on the GTX 1070 to its budget (4 M
  samples, two passes; `records/distill/structure-1.json`): two segments, a stop at 1 266 342
  samples (started 11:01 UTC) and one resumed part to 4 032 126 (from 15:06); the fair actor's
  play 215 s in all. The segments hold no wall-clock time, so the bench's segments record their
  seconds since (`segments[].seconds`, updated at each checkpoint):

  | Student | CE entry | CE cast | Agreement | Moves | Melee | Shots | Value | Seconds | µs | MB |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | MLP 2 × 512 | 1.510 | 1.996 | 0.533 | 0.285 | 0.364 | 0.617 | 0.624 | 188 | 23 | 100 |
  | MLP with the mask | 1.354 | 2.023 | 0.561 | 0.409 | 0.388 | 0.606 | 0.603 | 200 | 25 | 102 |
  | `entity` 256 × 4, 5e-4 | 0.867 | 1.397 | 0.719 | 0.481 | 0.685 | **0.783** | **0.679** | 2 836 | 353 | 1 457 |
  | … 2e-4 | 1.055 | 1.512 | 0.673 | 0.346 | 0.652 | 0.768 | 0.648 | 2 825 | 352 | 1 456 |
  | `hexconv` normalised 96 × 6, 1e-3 | **0.662** | **1.351** | **0.755** | **0.669** | **0.718** | 0.760 | 0.590 | 5 481 | 683 | 3 076 |
  | … 3e-4 | 0.785 | 1.581 | 0.721 | 0.598 | 0.670 | 0.719 | 0.545 | 5 487 | 683 | 2 460 |
  | `hexformer` 96 × 4, 5e-4 | 0.864 | 1.469 | 0.704 | 0.511 | 0.656 | 0.733 | 0.633 | 6 205 | 773 | 1 831 |
  | … 2e-4 | 1.078 | 1.729 | 0.642 | 0.462 | 0.560 | 0.691 | 0.540 | 6 216 | 774 | 1 831 |

- **The read:** every structured student passes the MLP by 0.11 to 0.22 agreement at the same
  samples (the MLP 0.533 here, T16's 0.531: the bench reproduces); `hexconv` at 1e-3 leads overall
  (0.755), in the moves (0.669 against the MLP's 0.285 and T20's 16 M MLP's 0.378) and in melee
  (0.718), and in both cross-entropies; `entity` at 5e-4 is second (0.719) at half the cost (353
  µs) and leads the shots (0.783) and the value (explained variance 0.679 against `hexconv`'s
  0.590, below the MLP's 0.624). The higher rate leads in every family, by 0.03 to 0.06 (as at
  T26's 0.45 M). The mask alone gains the MLP 0.028 (its moves 0.285 to 0.409). Ranked against
  the cost: `hexconv` 1e-3 (0.755, 683 µs), `entity` 5e-4 (0.719, 353), `hexformer` 5e-4 (0.704,
  773), the MLP with the mask (0.561, 25).
- **Decision 5's first gate passed:** `hexconv` normalised 96 × 6 at 1e-3, 0.755 at 4 M (0.60
  asked) within the cap (about 700 µs on `network --time`'s measure, where it was 595 without
  T25's keys; 683 on the bench's with them). By decision 3 it alone goes to T27: the
  next best (`hexconv` at 3e-4, 0.721; `entity` at 5e-4, 0.719) is 0.034 behind, beyond 0.02. Its
  lower value (0.590) is T28's to watch (PPO's critic starts from it); T27's clone measures it at
  16 M.
- **T27 prepared:** `configs/distill/e25-clone-3.toml`, T20's bench (16 M, the same seed and
  held-out set, the ladder against both system actors at 500 a kind) with the winner and the 2 ×
  512 MLP as the control (decision 1; T20's student again on the same episodes, 12 minutes of
  steps). About 6 hours of the winner's steps by T26b's seconds, in parts under 2 hours; its
  file about 6 MB (1.49 M parameters).

## T27 — The long clone (2026-11-06)

- **The run** (the user's machine, the GTX 1070, the teacher on its 24 threads;
  `python/records/distill/clone-3.json`; `configs/distill/e25-clone-3.toml`; C ABI v37, system
  policy 19): two segments, the first on `fa04f04` (#748's merge, clean) stopped at 6 599 926
  samples after 9 478 s, the second resumed on `bdbd6c6` (#757's merge, clean; between them only
  two synthetic packs' `towns.toml`, which no battle reads) to the budget at 16 020 741 after
  14 744 s. The teacher 16.06 M decisions in 308 353 episodes, none lost, 70 truncated, 805 s;
  the students' passes 21 734 s (`hexconv`, 681 µs a sample-pass, peak 3 076 MB) and 729 s (the
  MLP, 23 µs); the held-out set T20's (16 384 decisions, seed 8).

  | Samples | Student | CE entry | Agreement | Tactics | Wait, defend | Moves | Melee | Shots | Casts | Value |
  |---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | 2 M | `hexconv` | 0.787 | 0.720 | 0.891 | 0.601 | 0.586 | 0.666 | 0.730 | 0.794 | 0.507 |
  | 4 M | `hexconv` | 0.690 | 0.750 | 0.904 | 0.609 | 0.674 | 0.703 | 0.745 | 0.775 | 0.562 |
  | 8 M | `hexconv` | 0.596 | 0.781 | 0.912 | 0.675 | 0.740 | 0.742 | 0.784 | 0.756 | 0.618 |
  | 12 M | `hexconv` | 0.539 | 0.795 | 0.920 | 0.734 | 0.748 | 0.759 | 0.800 | 0.757 | 0.651 |
  | 16 M | `hexconv` | **0.497** | **0.809** | **0.921** | **0.774** | **0.780** | **0.776** | **0.810** | 0.747 | **0.658** |
  | 16 M | MLP 2 × 512 | 1.343 | 0.574 | 0.830 | 0.521 | 0.394 | 0.399 | 0.663 | **0.819** | 0.626 |
  | 16 M | T20's MLP | 1.344 | 0.576 | 0.836 | 0.542 | 0.378 | 0.401 | 0.660 | 0.831 | 0.633 |
  | | T19's ceiling | ≈0.05 | 0.972 | 0.995 | 0.992 | 0.985 | 0.941 | 0.998 | 0.977 | |

  The cast heads at 16 M: `hexconv` spell 0.716, target 0.828, second 0.998; the MLP 0.696,
  0.626, 0.996 (the ceiling 0.944, 1.000, 0.999).
- **The ladders** (500 per kind, both system actors on the report set v1, the neutral set v1):

  | Student | Fair actor | Hero | Siege hero | Siege garrison | All-seeing | Neutral set |
  |---|---:|---:|---:|---:|---:|---:|
  | `hexconv` | **0.481 ± 0.019, −0.020** | 0.481 | 0.471 | 0.500 | 0.473 ± 0.018, −0.021 | 0.989, **+0.274** |
  | MLP 2 × 512 | 0.414 ± 0.018, −0.065 | 0.404 | 0.400 | 0.484 | 0.407 ± 0.018, −0.068 | 0.978, +0.251 |
  | T20's MLP | 0.411 ± 0.018, −0.069 | 0.404 | 0.392 | 0.481 | 0.411, −0.071 | 0.982, +0.251 |
  | `e25-imitate-1` #2 (T22, PPO) | 0.458, −0.020 | | | | 0.462, −0.022 | 0.983, +0.287 |

  The neutral set's kinds: `hexconv` wandering 0.992, guards 0.988, siege_neutral 0.882; the MLP
  0.982, 0.972, 0.864.
- **The read:** the structure holds to 16 M: `hexconv` 0.809 against the MLP's 0.574 (+0.235;
  the bench reproduces T20, 0.574 against 0.576), its moves 0.780 and melee 0.776 (the MLP's 0.39
  and 0.40), the gap to the ceiling 0.163 (the MLP's 0.398). Four times the samples buy it +0.059
  (0.750 to 0.809, about 0.03 a doubling, the MLP's 0.02), still rising. Its value's explained
  variance passes the MLP's by 16 M (0.658 against 0.626; T26b's 0.590 at 4 M was early), so
  PPO's critic starts no worse. One category falls: the cast entry, 0.794 at 2 M to 0.747 at 16
  M, below the MLP's 0.819 (its targets 0.828 against 0.626): T19's tool below confirms it on
  T19's states. The agreement buys play: +0.067 win rate
  and +0.045 value against the fair actor over the MLP, in every kind (hero +0.077, siege hero
  +0.071, siege garrison +0.016).
- **Decision 5's second gate passed:** 0.481 ± 0.019 against the fair actor, T20's 0.411 ± 0.018
  (the intervals 0.462 and 0.429 apart). The clone alone plays at the current best's level
  (`e25-imitate-1` #2, 20 M of PPO: 0.458, the same value −0.020; inside the intervals), the
  neutral set 0.013 below it (+0.274 against +0.287, the target +0.285). Exit criterion 1 (above
  0.5 and a positive value) is not met by it. T28 next.
- **T19's tool** (the sandbox's CPU, the uploaded student; `python/records/distill/ceiling/
  ceiling-3.json`; commit `fe58dfc`, clean; T19's 25 600 states with 8 draws each, its ceiling
  reproduced, 0.972 and 0.044 nats; 2 minutes):

  | Network | All | Tactics | Wait, defend | Moves | Melee | Shots | Heal | Casts | Other | CE |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | T27's `hexconv` | **0.810** | 0.923 | 0.786 | 0.770 | 0.776 | 0.819 | 0.960 | 0.748 | 0.976 | **0.496** |
  | T16's student (MLP, 4 M) | 0.531 | 0.788 | 0.485 | 0.298 | 0.360 | 0.608 | 0.699 | 0.841 | 0.924 | 1.514 |
  | the ceiling (mode) | 0.972 | 0.995 | 0.992 | 0.985 | 0.941 | 0.998 | 1.000 | 0.977 | 0.979 | ≈0.05 |

  The bench's held-out set and T19's states agree (0.809 and 0.810; casts 0.747 and 0.748): at a
  quarter of the teacher's casts the student's most likely entry is another one. Whether it casts
  later, another spell or not at all is T28's `inspect` (the tool scores the entry only).
- **T28 prepared** (the sandbox's CPU): `configs/e25-imitate-structure.toml`, `e25-imitate.toml`
  with T27's student's network. A new `[ppo] accumulate` (the chunks a minibatch's gradient is
  summed over before its one step; 1 the update as before) keeps PPO's 8 minibatches of 4 096
  rows and their 32 steps an update at 8 chunks of 512 rows, so only the network differs from the
  current best's run (`minibatches` 64 would have made 256 smaller steps); its test updates the
  same rollout both ways to the same weights and losses (and fails with the chunks' own advantage
  normalisation). The smoke (`--init-from` an 8 192-sample CPU clone of the same network, 72 s;
  then 2 841 samples, 16 envs, chunks of 16 rows, a gate of 8 pairs, ladders of 2 a kind; 2.3
  minutes): the network loads strictly, the imitation loss and the gate run, the ladder plays.
  The Python suite 243 passed, 1 skipped. No warm-up or weight decay:
  PPO's Adam at 2.5e-4 starts from a network the bench took down to 1e-4.

## T28 — The A/B (2026-11-10)

- **The run** (the user's machine, the GTX 1070, 24 threads; `python/records/e25-structure-1.json`;
  `configs/e25-imitate-structure.toml` `--init-from` T27's student `hexconv-n-96x6-a.pt`, 16.02 M
  samples of cloning; C ABI v37, observation v22, action v13, system policy 19, the battle
  ruleset `c14c573e9d2ee26f` in every segment): 20 022 456 samples in 780 updates, 484 513
  episodes, 1 676 minutes in 13 segments on four clean commits (`305725f`, `36c9028`, `2e5e08f`,
  `a14e30f`; between them only towns' and adventure data and tests); four segments ended without
  a save (4 records dropped). **217 samples a second**, the current best's run 4 400 to 5 100: 22
  times its 77 minutes.
- **The GPU's memory:** in segments 3, 5 and 11 single updates took 650 to 1 950 s instead of
  about 100 (`update_s`; the rollouts unchanged at 16 s), from update 584 (14.98 M samples) on,
  until a restart; the user saw the card's 8 GB full, about 6 GB after a restart. The trainer's
  code was the same in every segment. The cause: PyTorch's caching allocator keeps its freed
  blocks (the tensors change size from call to call) and gives memory back only when an
  allocation fails, and on Windows the driver's sysmem fallback puts the next allocation into
  shared system memory instead of failing it. Fixed (the user, 2026-11-10: never past 90 % of the
  card): `h3env.train.gpu.cap` caps the allocator so that the card stays at 90 %, what it holds
  outside the run counted against it (the trainer, the benches, `evaluate.load`); the update
  records and lines show the reserved memory, its peak and the card's share (`TRAPS.md`,
  Python). Tests `tests/test_gpu.py` (stubbed CUDA) and the update line's in `test_run.py`.
- **Gates:** at 10 M #1 promoted after 115 pairs (0.541, +29 Elo), at 20 M #2 after 713 (0.517,
  +12 Elo). The imitation loss's weight 0.5 fell to 0 at 10 M: the policy's entropy held about
  0.70 while the weight was above 0.25, rose to 1.34 by 12.5 M and fell to 1.00 by 20 M; the
  sampled agreement with the teacher 0.71 to 0.58 at the loss's end; the value's explained
  variance 0.75 to 0.80.
- **The final ladder** (500 per kind; the report set v1, the neutral set v1 attacking the fair
  actor):

  | Network | Random | Aggressor | Greedy | Fair actor | All-seeing | Neutral set |
  |---|---:|---:|---:|---:|---:|---:|
  | `e25-structure-1` #2 | 0.887, +0.343 | 0.707, +0.179 | 0.673, +0.149 | **0.511 ± 0.019, +0.027** | **0.507 ± 0.019, +0.026** | 0.991, **+0.302** |
  | its #0 (T27's student, 250 a kind) | 0.853, +0.308 | 0.648, +0.117 | 0.615, +0.092 | 0.473, −0.022 | 0.472, −0.021 | 0.995, +0.278 |
  | `e25-imitate-1` #2 (the current best) | 0.858, +0.305 | 0.651, +0.124 | 0.626, +0.099 | 0.458 ± 0.019, −0.020 | 0.462, −0.022 | 0.983, +0.287 |

  Against the fair actor by kind: hero 0.509, siege hero 0.519, siege garrison 0.505 (the
  current best's 0.447, 0.462, 0.493); the neutral set's kinds wandering 0.994, guards 0.990,
  siege_neutral 0.910 (0.986, 0.982, 0.884).
- **T19's tool after RL** (the sandbox's CPU, the uploaded `best.pt`;
  `python/records/distill/ceiling/ceiling-4.json`; T19's 25 600 states, 8 draws each, its ceiling
  reproduced; 66 s):

  | Network | All | Tactics | Wait, defend | Moves | Melee | Shots | Heal | Casts | Other | CE |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | `e25-structure-1` #2 | 0.454 | 0.399 | 0.373 | 0.107 | 0.491 | 0.575 | 0.812 | 0.715 | 0.885 | 3.319 |
  | T27's student (its #0) | 0.810 | 0.923 | 0.786 | 0.770 | 0.776 | 0.819 | 0.960 | 0.748 | 0.976 | 0.496 |

- **`inspect`** (the sandbox's CPU, 4 threads, 30 minutes; the sampled policy, the CPU's draws;
  `python/records/inspect/e25-structure-1-report-v1.json`): against the fair actor 0.511 ± 0.019,
  value +0.027 (2 998 games), Greedy 0.677, +0.151, as on the GPU. **The casts:** the hero casts
  in 0.918 of the rounds it can (`t7-1` 0.874, `t6d-1` 0.788), 4.0 such rounds a battle, 32.9
  mana spent (`t7-1` 33.5; the fair actor's hero 31), the spells Magic Arrow, Lightning Bolt, Ice
  Bolt, then Haste, Cure, Blind, Slow (`t7-1`'s with Haste and Cure moved up). The decisions'
  mix: melee 0.37, moves 0.19, shots 0.11, hero casts 0.10, tactics 0.10, waits 0.07.
- **The read:** the structure wins the A/B. From its clone PPO adds +0.030 win rate and +0.047
  value against the fair actor (0.481, −0.020 at T27 to 0.511, +0.027), as PPO added to the MLP's
  clone at T22 (+0.047, +0.049), so the structure's lead of T27 carries through RL: +0.053 over
  the current best against the fair actor (the intervals 0.492 and 0.477 apart), +0.045 against
  the all-seeing one, every opponent's row and every kind up, the neutral set +0.015. The agent
  leaves its teacher once the loss ends (agreement 0.810 to 0.454, its moves 0.770 to 0.107)
  and plays better than it, so the clone's agreement was the start, not the bound. The clone's
  cast agreement (0.748) is not a casting gap: it casts more often than any earlier network.
  **Exit criterion 1 on decision 1's point estimates:** against the faithful (all-seeing) actor
  0.507 and +0.026, the neutral set +0.302 above +0.285, Random, Aggressor and Greedy above 0.5
  with positive values: every part met, the faithful actor's win rate inside its interval of 0.5
  (0.488 to 0.526). The cost: 22 times the MLP's wall time (28 hours a 20 M on the GTX 1070).
- **Not done here:** the head-to-head against `e25-imitate-1` #2 (`h3env.train.gate`; both
  networks on the user's machine): T29's first step. The checkpoint's question: AI-Q17.

## T29 — Increment 7's checkpoint (2026-11-10)

- **The head-to-head** (the user's machine, `h3env.train.gate --candidate
  runs/e25-structure-1/best.pt --best runs/e25-imitate-1/best.pt --device cuda`): `e25-structure-1`
  #2 **promoted after 77 pairs** (2 559 games), score **0.552 (+36 Elo)**, LLR +3.00 in [−2.94,
  2.94] (elo0 0, elo1 20), 39 s. The lineage's earlier steps: 0.534 (`e25-imitate-1` #2 against
  `e25-clone-1` #2, T22), 0.564 (`e25-clone-1` #2 against `e25-sharp-1` #2, T17).
- **The user** (AI-Q17): **`e25-structure-1` #2 the current best**; **one more increment**, but
  analysis first and probes of architectures, PPO's settings and training regimes before one long
  run at the end; the probes' budget about 20 hours of the GPU, all four families (the update's
  speed, architectures, PPO's settings, training regimes). Increment 8's decisions in the epic
  file.
- **The read of increment 7:** the structure was the lever increments 5 and 6 pointed to. The
  clone's agreement 0.576 to 0.809 (T27), its ladder against the fair actor 0.411 to 0.481, PPO's
  endpoint 0.458 to 0.511 (T28), at 22 times the MLP's wall time.

## T30 — The analysis of #2 (2026-11-10)

- **By kind and side** (the sandbox's CPU; `python/records/inspect/e25-structure-1-report-v1.json`,
  T28's `inspect` against the fair actor; the near-mirror `python/records/bots-report-v1.json`'s
  fair and all-seeing actors in both seats, 0.4999 against each other; the battle ruleset
  `c14c573e9d2ee26f` in both):

  | Kind (weight) | Mirror's attacker | #2 attacking | Edge | #2 defending | Edge | Value edge (attacking, defending) |
  |---|---:|---:|---:|---:|---:|---:|
  | hero (100) | 0.694 | 0.707 | +0.013 | 0.306 | +0.000 | +0.022, +0.029 |
  | siege hero (50) | 0.549 | 0.580 | +0.031 | 0.470 | +0.019 | +0.029, +0.045 |
  | siege garrison (25) | 0.941 | 0.940 | −0.001 | 0.060 | +0.001 | +0.006, +0.013 |

- **The temperature** (T9a's diagnostic; `python/records/inspect/e25-structure-1-temperature.json`:
  `h3env.train.ladder --checkpoint runs/e25-structure-1/best.pt --opponents system --temperature
  1,0.7,0.5,0`, the sandbox's CPU, 76 minutes; the same episodes and draws at each):

  | Temperature | Against the fair actor | Hero | Siege hero | Siege garrison | Value |
  |---:|---:|---:|---:|---:|---:|
  | 1 (the policy) | 0.511 ± 0.019 | 0.507 | 0.525 | 0.500 | +0.027 |
  | 0.7 | 0.513 ± 0.019 | 0.512 | 0.523 | 0.501 | +0.030 |
  | 0.5 | **0.522 ± 0.018** | **0.525** | 0.526 | 0.501 | +0.030 |
  | 0 (the most likely) | 0.514 ± 0.018 | 0.515 | 0.519 | 0.504 | +0.031 |

- **Where it leaves the teacher** (`python/records/distill/ceiling/ceiling-5.json`: T28's run of
  T19's tool again, its agreements unchanged, with the tool's new confusion table: by the
  category of the fair actor's draw, the categories of #2's most likely entry; `ceiling`'s
  `confusion`, test in `tests/test_ceiling.py`):

  | The teacher's | #2's: the same category (the same entry) | then |
  |---|---|---|
  | wait, defend | 0.410 (0.373) | move 0.328, cast 0.216, melee 0.040 |
  | move | 0.596 (0.107) | wait, defend 0.254, cast 0.110, melee 0.040 |
  | melee | 0.814 (0.491) | cast 0.090, move 0.051, wait, defend 0.044 |
  | shot | 0.897 (0.575) | cast 0.062, move 0.038 |
  | cast | 0.726 (0.715) | melee 0.179, wait, defend 0.036, move 0.030, shot 0.029 |
  | tactics, heal, other | 1.000, 1.000, 0.990 | |

- **The read, the probes ranked:**
  1. **The margin lies in the hero battles**, the report set's largest kind (100 of 175): there
     #2 keeps more of its army than the fair actor in its seat (+0.022 and +0.029) but wins as
     often (+0.013 attacking, 0.000 defending); the sieges with a hero carry the edge (+0.031,
     +0.019), the garrison's is the kind's (0.94 to the attacker either way). Reward v1 prices
     what each side keeps and loses (E05 T4), the win rate counts the outcome alone: a training
     signal that weighs the outcome more is a candidate for the checkpoint (decision 6 keeps
     reward v1 and allows shaping only), not a probe.
  2. **Sharpening gains +0.011** (temperature 0.5; the hero battles +0.018), as increment 4 found
     for the MLP (about 0.01; `e25-sharp-1`'s entropy bonus at 0 kept it): the entropy rose from
     0.70 to 1.34 once the imitation loss ended (T28), so the bonus at 0, or the loss held at a
     small weight, is T33's first setting.
  3. **No capability is missing:** #2 keeps the teacher's kind of action and changes its hex or
     target (the moves 0.596 and 0.107); where the teacher moves it waits 0.254, where the
     teacher waits it moves or casts; it casts 0.92 of the rounds it can (T28). The analysis asks
     no encoding change (decision 5).
  So: T31 (the speed) first, as planned; T33 the entropy bonus at 0 and the loss held, each a
  short continuation from #2, then the learning rate and the epochs; T32 the architectures by
  their clone's ladder, `hexconv` 96 × 6 the control; T34 a distillation of #2 and the search at
  play on the hero battles; the outcome's weight a question for T35.

## T31 — The update's speed (2026-11-10)

- **The suspects** (the code, before any profile): the trainer runs cuDNN without its autotuner
  (`torch.backends.cudnn.benchmark` off), while the benches turn it on (`distill.bench_device`):
  T27's 683 µs a sample-pass of `hexconv` 96 × 6 was the bench's (minibatch 512), T28's updates
  of about 100 s over about 25 700 rows and 4 epochs are about 970 µs. And the hex trunk's
  convolutions get channels-last tensors: the drawing's grid is a transposed view
  (`HexConv.trunk`) and each `CellNorm`'s permutes keep that layout, so cuDNN runs on NHWC
  tensors (`tests/test_speed.py` checks the layouts).
- **The tool** (`h3env.train.speed`, tests `tests/test_speed.py`): the config's trainer plays one
  rollout (the imitation loss's labels with it), then each variant updates from the same weights,
  optimizer state and counters (`VARIANTS`: the autotuner, the trunk in NCHW, `accumulate`,
  `chunk_align`, combinations; the baseline first, again last for the card's drift), twice, over
  one epoch: the seconds (the first a warm-up: the autotuner's search for the update's chunk
  shapes, the allocator's growth), µs a sample-pass, the peak memory, the losses and the weights
  against the baseline's (and the baseline's repeat against it: cuDNN is not deterministic);
  then the baseline's phases timed with the device synchronised around each, and torch's
  profiler over one optimizer step. A CPU smoke (16 envs, 32 calls, 501 rows): the losses equal
  in every variant, the weights within 7e-7.
- **The cap's index** (the user's first run): `gpu.cap` handed torch's
  `set_per_process_memory_fraction` a plain `cuda` device (what `ppo.device_of` gives for `auto`
  and `cuda`), which torch refuses (`ValueError`: no index): every CUDA entry point since T28
  stopped there, never run on a card (`tests/test_gpu.py` stubs CUDA). Fixed: the current card's
  index; the stub now refuses a device without one, as torch's does.
- **The first run** (the user's GTX 1070, torch 2.14.1+cu126; #2's network, `configs/e25-imitate-
  structure.toml`; the rollout 12.0 s; 32 575 rows an epoch, a fresh trainer's self-play; each
  update one epoch, twice; `python -m h3env.train.speed --config configs/e25-imitate-structure.toml
  --init-from runs/e25-structure-1/best.pt --device cuda --trace`):

  | Variant | Chunks' rows | Seconds | µs a sample-pass | Against the baseline | Peak MB |
  |---|---|---|---:|---:|---:|
  | baseline (`accumulate` 8) | 508, 509 | 29.56, 28.92 | 888 | 1.000 | 3 189 |
  | autotune | 508, 509 | 31.15, 31.48 | 967 | 1.089 | 3 189 |
  | nchw | 508, 509 | 31.53, 31.45 | 966 | 1.087 | 3 144 |
  | nchw + autotune | 508, 509 | 31.46, 30.60 | 939 | 1.058 | 3 144 |
  | autotune, `accumulate` 4 | 1 017, 1 018 | 61.66, 22.56 | 692 | 0.780 | 5 448 |
  | nchw + autotune, `accumulate` 4 | 1 017, 1 018 | 24.49, 22.51 | 691 | 0.778 | 5 358 |

  The losses agree to about 0.5 % (the policy loss 0.1734 to 0.1743, the imitation 3.115 to
  3.118), the weights within 0.0013 of the baseline's, which an update moved by 0.002 at most
  (Adam's steps on near-zero gradients; cuDNN's FFT kernels are not deterministic). The
  baseline's phases (synchronised, 29.38 s): **the backward pass 0.665, the trunk's forward
  0.277**, the rest of the encoding 0.022, the gather 0.009, the entry head and the value 0.009,
  PPO's heads 0.006, the teacher's 0.005, the clipping and the step 0.001, the rest 0.007. The
  profiler (one step, 8 chunks, 3.49 s of the device): the convolutions' backward 45 % (cuDNN's
  `maxwell_gcgemm_32x32` 19 %, FFT tiles of 32 × 32 over the 11 × 22 grid 17 %, `flip_filter`
  3 %), their forward 14 % (Winograd 10 %), cuDNN's own NHWC ↔ NCHW conversions 8 %, the
  LayerNorms 16 % forward and backward, the element-wise products and sums the rest.
- **Its read:** neither the autotuner nor the layout pays at the run's chunks (the later variants
  0.94 to 1.09, the card warming up over the minutes); **twice the chunks' rows pays 0.78**, the
  bench's speed (T27's 683 µs at 512 rows with the autotuner): the chunks' size, or their
  shapes (508 and 509 against the bench's 512), not the autotuner itself. The first big-chunk
  variant's 39 s warm-up is mostly the allocator's growth from 3.2 to 5.4 GB (the next one's
  search for the same rows in NCHW took 2 s). Not separated yet: the size alone, without the
  autotuner, and an aligned shape at today's size. So `[ppo] chunk_align` (each chunk padded to
  a multiple of rows by repeating its first, the pad in no loss; 0 the default; test
  `test_a_padded_update_steps_as_the_unpadded_one_does`) and a second run.
- **The second run** (the same machine and settings, `--no-phases`; the rollout 11.5 s):

  | Variant | Chunks' rows | Seconds | µs a sample-pass | Against the baseline | Peak MB |
  |---|---|---|---:|---:|---:|
  | baseline (`accumulate` 8) | 508, 509 | 30.55, 30.22 | 928 | 1.000 | 3 189 |
  | `accumulate` 4 | 1 017, 1 018 | 28.36, 26.79 | 822 | 0.887 | 5 453 |
  | autotune, `accumulate` 4 | 1 017, 1 018 | 25.74, 25.29 | 776 | 0.837 | 5 285 |
  | `accumulate` 5 | 814, 815 | 31.46, 32.29 | 991 | 1.069 | 5 040 |
  | `chunk_align` 32 | 512 | 30.00, 28.63 | 879 | 0.948 | 3 190 |
  | autotune, `chunk_align` 32 | 512 | 29.36, 30.74 | 944 | 1.017 | 3 190 |
  | autotune, `accumulate` 4, `chunk_align` 32 | 1 024 | 43.40, 22.36 | **686** | **0.740** | 5 480 |
  | baseline again | 508, 509 | 31.55, 31.53 | 968 | 1.044 | 3 172 |

  The baseline's repeat lies 0.00106 from its first update (cuDNN's nondeterminism), the
  variants 0.0009 to 0.0013: rounding's; the losses within 0.5 %.
- **Its read:** the card drifts +4 % over the minutes, and the same variant measures 692 in one
  run and 776 in the next (the autotuner times the kernels it picks): about 10 % is noise. The
  cost is the chunk's **shape**, not its size: 814 rows cost more than 508, 1 017 less, exactly
  1 024 with the autotuner least (0.74, the bench's 683 µs; its search 21 s, once per shape),
  while 512 gains nothing either way. A run's chunks are not the tool's: with the league's
  members a run trains about 25 700 rows an update (T28), about 800 a chunk at `accumulate` 4,
  so the chunking that pays is chosen on the cost by rows. Hence `--sweep`: one chunk's pass
  (`Trainer._loss`, the update's own; `Trainer.samples_of` its arrays, split out of `update`) at
  each of a list of rows, the autotuner off and on.
- **The sweep** (the same machine; `--variants baseline --repeats 1 --no-phases --sweep
  256,…,1280`; the baseline's update 885 µs a sample-pass): one chunk's pass, µs a row by cuDNN's
  heuristics (the plain pass, first at each size):

  | Rows | 256 | 384 | 448 | 512 | 576 | 640 | 768 | 800 | 832 | 896 | 960 | 1 024 | 1 056 | 1 088 | 1 152 | 1 280 |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | µs a row | 894 | 896 | 886 | 841 | 957 | 952 | 1 172 | 829 | 811 | 938 | 879 | 765 | **671** | **671** | 737 | out of memory |
  | Peak MB | 1 609 | 2 382 | 2 775 | 3 180 | 3 559 | 3 953 | 4 737 | 4 300 | 4 467 | 5 520 | 5 139 | 5 292 | 5 227 | 5 386 | 5 431 | |

  256 again at the end: 933 (the drift). **The autotuner's column was void:** its warm-up 0.00 s at
  every size, the same µs as the plain pass. cuDNN caches a convolution's kernels by its shapes
  and flags, not by the autotuner's, so the autotuned pass after the plain one reused the plain
  one's kernels, its search never ran. The same explains the variants: the autotuner acted only
  where its shape was new (1 017 and 1 018 rows in the first run, 692 µs against the
  heuristics' 822 in the second; 1 024 in the second, 686 against the sweep's 765): **it pays 10
  to 16 % there**, the "autotune" rows at the baseline's shapes reused the baseline's kernels.
  The tool now gives each mode its own cache entries (`speed.cudnn_mode`: the plain passes with
  TF32 off, which is in the key and changes nothing on a card before Ampere) and the sweep runs
  the autotuner first at each size.
- **Its read:** the heuristics' cost by rows is not monotonic; a run's chunks today (about 3 200
  rows a minibatch over 8: about 400) cost about 890 µs, `accumulate` 3 with `chunk_align` 32
  (about 1 070 rows, to 1 088) 671: about 0.75 of today's update before the autotuner.
- **The last sweep** (the autotuner first at each size, `cudnn_mode`; the baseline's update 888
  µs a sample-pass):

  | Rows | 400 | 512 | 832 | 1 024 | 1 056 | 1 088 | 1 120 | 1 152 | 1 216 | 400 again |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | µs a row, heuristics | 910 | 839 | 964 | 802 | 747 | 747 | 729 | 730 | 786 | 942 |
  | **with the autotuner** | **684** | **656** | **638** | **615** | **615** | **613** | **614** | 677 | 718 | 688 |
  | its search s | 0.49 | 0.57 | 0.87 | 18.55 | 19.06 | 19.70 | 20.32 | 24.07 | 25.28 | 0.01 |
  | Peak MB | 2 491 | 3 180 | 5 133 | 5 475 | 5 647 | 5 819 | 5 994 | 5 711 | 6 020 | 2 480 |

- **The read, the change:** **the autotuner is the lever**: 0.75 of the heuristics' time at a
  run's chunks today (about 400 rows), 0.66 to 0.82 at every size, its search under a second a
  shape up to 832 rows and about 20 s from 1 024; larger chunks add a little (about 614 µs from
  1 024 to 1 120 rows, 638 at 832). So:
  1. **`[device] cudnn_autotune`** (default on, resumable; `ppo.autotuned` around the update's
     steps, the flag as it was after; the rollout, the gate and the ladders unchanged, as
     measured): test `test_the_update_runs_with_cudnns_autotuner_as_its_config_says`.
  2. **For `hexconv` 96 × 6 on the GTX 1070: `[ppo] accumulate = 4`, `chunk_align = 32`** (a
     run's about 3 200 rows a minibatch: chunks of about 800, 638 µs, about 0.70 of today's
     910; the alignment keeps the shapes few, each searched once a segment). Not 3 (1 088 rows,
     613 µs): a run's start trains every row (no league member yet, about 32 500 rows: chunks of
     about 1 360), past 1 280 rows, which the cap refused.
  3. The trunk in NCHW measured no gain: dropped from the tool (the finding above).
  `Trainer.update` keeps its losses: the autotuner changes kernels only, `chunk_align` is tested
  against the unpadded update, `accumulate` since T28.
- **The confirmation** (the code above; `python -m h3env.train.speed --config
  configs/e25-imitate-structure.toml --init-from runs/e25-structure-1/best.pt --device cuda
  --no-phases`; the rollout 11.5 s, 32 575 rows an epoch, each update one epoch, twice):

  | Variant | Chunks' rows | Seconds | µs a sample-pass | Against the baseline | Peak MB |
  |---|---|---|---:|---:|---:|
  | baseline (the autotuner, `accumulate` 8) | 508, 509 | 26.73, 25.23 | 775 | 1.000 | 3 174 |
  | heuristics (the update before T31) | 508, 509 | 31.31, 31.20 | 958 | 1.236 | 3 189 |
  | `accumulate` 4, `chunk_align` 32 | 1 024 | 43.22, 22.54 | 692 | 0.893 | 5 479 |
  | `accumulate` 5, `chunk_align` 32 | 832 | 21.91, 21.15 | **649** | **0.838** | 5 138 |
  | baseline again | 508, 509 | 23.54, 23.45 | 720 | 0.930 | 2 557 |

  The baseline's repeat lies 0.00093 from it, the variants 0.0010 to 0.0013; the losses within
  0.5 %. `accumulate` 5 here gives the chunks `accumulate` 4 gives a run (about 3 200 rows a
  minibatch), `accumulate` 4 here those of a run's start (every row trains before the league's
  first member: 1 024 rows, the search's 21 s once).
- **The acceptance, an update before and after on the user's GPU** (`hexconv` 96 × 6, the
  GTX 1070; a run's update T28's mean 25 670 rows over 4 epochs): **before T31 958 µs a
  sample-pass, 98 s an update** (T28 measured about 100); **after it** (the autotuner,
  `accumulate` 4, `chunk_align` 32) **649 µs, 67 s: 0.68**. A 20 M run like T28's (780
  updates): about 14 hours of updates instead of 21, about 21 hours in all instead of 28 (its
  rollouts, gates and ladders unchanged). The autotuner alone, at `accumulate` 8, gives 0.75 to
  0.81; T33's continuations from #2 run with all three.
