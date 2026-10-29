# 07 — Simulation environment and AI

The AI is the main motivator of the project. It is not the first thing built, but it shapes upfront decisions (determinism, cheap fork/clone, action masks, batch-friendly API, observation projection).

## 1. Simulation environment (`h3-sim`)

A thin, allocation-conscious wrapper over the game logic:

- `reset(scenario, seed) -> Observation`
- `step(actions) -> (Observation, reward, done, info)` for each environment in a batch
- `legal_actions(env) -> mask` (from `legal_commands`; the action encoding is defined by the sim layer, not the domain): dense entries for every command but a cast, and a factorized cast whose spell and target heads have masks conditioned on the earlier choices (ADR-0014 point 9; `h3_sim::action`)
- `fork(env) -> env` (cheap clone for search/rollouts) and `state_hash(env)`
- Two environment families: **BattleEnv** (from a `BattleSetup`; first) and **AdventureEnv** (E08 T9a: a game on a map through `h3-game`, `h3-sim`'s `src/adventure/`, the AI lane's since E08 T45 but for the replay model `replay.rs`, ADR-0017 point 8; its replays record the game commands, not the env's actions, and `h3 adventure run/verify/bench` drives it, T9b; action encoding v2 adds the answers to an open decision, an entry that takes no tile has tile 0 alone, and a replay records the game's difficulty, T14b; encoding v3 adds the battle phase, in which a player in a battle acts with the battle's own encoding for its side and `to_move` yields the owner of the side to act, and replay format v3 records each command's actor and battle commands, T19b1; since T20b3a `to_move` yields seats, `h3_sim::adventure::Seat`: a player, or the neutral seat for a battle's unowned side, played as the system actor with `neutral_observation`, and the random agent and the recorder play it too; encoding v4 adds a level-up's answers, `First` and `Second`, and replay format v5 records them, E08 T30: while a level-up is open the game waits for it and `to_move` yields its owner alone)
- Vectorized/batched stepping in Rust, parallel across instances; single-threaded per instance
- Observations are **per-player projections** (same function as the security layer), so agents never see hidden information: an agent sees exactly what a human player would, as data rather than pixels (AI-Q1). `fork` and `state_hash` see the full state: tools and training only, never an agent's move
- **Choices the battle makes for a side as the original's computer** (ADR-0023 point 7; E04c): the automatic turrets' and Ballista's targets read `BattleSetup.difficulty` and `.human`, which never decide who commands a side. The defaults (Easy, no human side) are the original's battle between two computer players, in which those targets count a computer side's walker of few hit points as keeping out of the moat, and that walker's berserk walk keeps out of it too (BAT-SIE-022, BAT-Q50): a `BattleEnv` for a human player's battles sets `human`; the training batch sets it for both sides (ADR-0025 point 6: a level playing field for human players and agents)
- Reward is computed in the sim layer from events/state; the domain knows nothing about rewards. Reward v1 (ADR-0025 point 8): kill the most, lose the least, spend mana sparingly, in one currency (`ai_value`, or the creature's cost), zero-sum in self-play; nothing from the adventure counts (no hero, artifacts, treasure, town or gold), and the training batch disables retreat and surrender (point 7): whether to fight or flee is the adventure's question
- **Opponents** (AI-Q1): agents face each other (a hero against a hero) and the system actor (a neutral side; E23's policy of the original's computer), whose seat the engine plays inside a step
- Exposed to other languages via the C ABI (`h3-ffi`), batch-oriented to amortize call overhead

The scenario distribution matters as much as the engine: training on few fixed setups overfits. Battle setups are sampled from generators (armies, terrain, heroes, artifacts, spells); adventure training needs generated maps (epic: map generator, seeded by the user's existing project, optimizing zone balance and minimal teleport connections among other objectives).

### 1.1 The battle environment as built (E05, ADR-0025)

E05 (closed 2026-10-20) made every battle the engine plays trainable from Python. The decisions
are ADR-0025's; the build's detail is in the code documentation (`h3_sim`'s `batch`, `reward`,
`encounter`, `bots`; `h3-ffi`'s `batch`; `h3-cli`'s `src/ai/`), `python/README.md` and the
design note kept in `docs/epics/E05-log.md` (§1 to §9).

- **The batch** (`h3_sim::batch`, C ABI `h3_batch_*`): N envs per handle stepped by one call;
  observations and entry masks are written into the caller's arrays, and per env the acting side,
  both sides' rewards, done, truncated, the result, the episode's seed and kind and a status. The
  three cast heads are one call each for the whole batch. An ended episode resets in the same
  call (its reward, result and, after a truncation, both final observations reported). T std
  threads in contiguous shards; play never depends on T, and a batch steps bit for bit as N
  single envs with their episode seeds (a property test). At most `MAX_ENVS` (65 536) envs; each
  entry's rulesets' content hashes and the encodings' and encounters' versions read back for a
  training run's record (C ABI v31, E06 T2c). Like both sides' final observations, the acting
  side's previewed kills of each legal attack and shot entry are an option (`Outputs::preview`,
  C ABI v33, E06 T5): the engine's preview (BAT-DMG-011), what the original shows a player
  before a strike, never the retaliation.
- **The observation** (`h3_sim::observation`, encoding v22): v21's flat layout, then per stack
  slot its places in the engine's turn order (`h3_battle::turn_order`: in the rest of the round,
  among its side's, in the next round; E06 T6f). A network may read v21's prefix alone (the
  trainer's `[network] order` off, its default).
- **Seats** per scenario side: `External` (the caller acts) or `Internal(policy)`, played inside
  the call: `Random` (tests and benchmarks), `System` (E23's policy of the original's computer),
  the bots `Aggressor` and `Greedy`. Every side plays as a human player's (`human`), and Retreat
  and Surrender are masked out: whether to fight or flee is the adventure's (E13).
- **Episode sources:** a weighted list of the battle lane's scenario files and the **encounter
  distribution v1** (`crates/h3-sim/data/encounters-v1.toml`): the census of the 159 shipped maps
  (`census-v1.toml`, aggregate counts only, written by `h3 ai census`) and the assumptions A1 to
  A12 decided with the user (AI-Q4): kind, week, place, the armies and their balance, heroes and
  sieges, integers only. Creatures, counts and splits come from the engine's own rules; a hero's
  level-ups, spells and artifacts are approximations. The default weights draw the kinds with a
  player on both sides (a hero battle 4/7, a siege a hero defends 2/7, a garrison's 1/7); the
  neutral kinds, whose defender the system actor plays (E06 T7), weigh 0 by default and join a
  training run through its own weights (`[env] kinds`), so the held-out sets keep their episodes.
  Encounters v2 will come from the adventure's logged battles.
- **Reward v1:** `r_s = (K_o − K_s − p · (m_s − m_o)) / S` in `ai_value` (or the creatures' cost),
  K a side's losses including the war machines the battle placed, m its mana spent, p 10 per
  spell point (provisional), S both sides' value at the start; terminal, 0 after a truncation,
  zero-sum exactly.
- **Information:** only projected observations cross the batch (the acting side's, both sides' at
  a truncation). The fork, the state hash and the episode seeds are trusted tooling (`h3.h`,
  `h3env.tools`), never an agent's input. A search agent also reads *worlds* (ADR-0025 point
  12a; C ABI v34, E06 T8b; scored by the batch's reward v1 since v35, E25 T1a): copies of the
  battle in which what its side may not know is replaced by a guess drawn from the world's seed
  (`h3_batch_determinize`, the engine's `BattleState::determinized`; the stream re-seeded,
  `h3_env_reseed`), which it may fork and step, for its side to move only (E06 T9b); the true
  state's fork and hash stay tooling, as does the true world of E25's diagnostic
  (`h3_batch_true_world`, v35, T1b: nothing guessed). Its known gaps are AI-STUB-1's (E25).
- **Python** (`python/`, package `h3env`): ctypes over the C ABI, the caller's numpy arrays
  written in place, the library's ABI version checked, short arrays refused and the outputs read-only; uv with
  CPython 3.12 (on the user's machine a conda Python); PyTorch 2.14 (CUDA 12.6) in the opt-in
  group `train`.
- **The smoke training** (`h3env.train`): an own PPO over the factorized heads, since a library's
  masked PPO masks each dimension alone and the cast's heads are conditioned on the earlier
  choices; an MLP over `symlog` of the flat observation v21; self-play with a pool of frozen
  earlier networks (naive self-play let the defender regress); evaluation of the sampled policy
  on held-out episode seeds (greedy play repeats itself until the step limit). After 40 minutes
  on the GTX 1070 it won 0.800 against Random.
- **Baselines** (`h3_sim::bots`, `h3 ai eval`): Aggressor and Greedy read their side's
  observation and the engine's preview of their own attacks, never the retaliation (AI-Q6); one
  Glicko-2 period per run. On 1 000 held-out encounters per pair (v0): Greedy 0.828 and Aggressor
  0.800 against Random, Greedy 0.519 against Aggressor; the smoke network 0.802 against Random,
  0.498 against Aggressor, 0.479 against Greedy.
- **Held-out sets** (E06 T3a, `docs/epics/E06-log.md`, the design note's §5): `h3 ai eval --set` and the Python
  ladder play the report set v1 (by default: per kind with a player on both sides 500 battles
  from both sides, the kinds weighed 4 : 2 : 1, the win rate with its 95 % interval) or E05's v0;
  the trainer checks every training episode against both and the gate set v1. The bots' table on
  the report set is committed (`python/records/bots-report-v1.json`): Greedy 0.832 ± 0.013 and
  Aggressor 0.810 ± 0.014 against Random, Greedy 0.533 ± 0.018 against Aggressor.
- **Throughput** (wall time, not gated; the gates count instructions in `benches/ai/`):
  on the user's Ryzen 9 3900X (Windows, release, the encounter distribution) about 50 000 steps/s
  on one thread whatever the batch size, and inside the step 387 000 on 12 threads and 497 000 on
  24 at 1 024 envs (233 000 on 6); Python's binding is within 2 to 7 % of Rust inside the step
  (`E05-log.md`, T9c).

### 1.2 The training pipeline as built (E06, ADR-0026)

E06 (closed 2026-10-27) trained the battle AI v1 on the user's machine (GTX 1070, Ryzen 9 3900X)
with a pipeline that runs, records, evaluates and promotes on its own. The decisions are
ADR-0026's (the batch's ADR-0025's); the detail is in `python/h3env/train/`'s documentation,
`python/README.md` and the design note kept in `docs/epics/E06-log.md` (§1 to §15).

- **Runs** (`python -m h3env.train`): a TOML config (`python/configs/`, `--set table.key=value`);
  the budget and every cadence in samples (the learner's decisions), wall time only for
  checkpoints. A run's directory holds its config, `metrics.jsonl`, `record.json`, the
  checkpoint, the promoted networks and the league's snapshots; a record a result cites is
  committed (`python/records/`: the config, the commit, the engine's identity read back from
  the library, the curve, the gates, promotions and ladders).
- **Interruption** (decision 4): the checkpoint holds the whole training state (the network, the
  optimizer, the counters and schedule, the league, the random generators, what is pending, the
  metrics' length), written atomically every 5 minutes and around each gate and ladder; a stop,
  a crash or a kill loses at most the time since; `--resume` continues in a new segment with its
  own batch seed, the budget extensible: reproducible from the record, not bitwise.
- **PPO** (E05's own, over the factorized heads): an MLP over `symlog` of the observation (the
  default; its depth `[network] layers` since E25 T7a) or an entity network (attention over the
  stack slots, the hexes and the heroes; parked, slower and no better), or since E25 T7a a hex
  network (the stacks drawn on their hexes; convolutions on the axial grid, `hexconv`, or a
  transformer over the hexes, `hexformer`; `h3env.train.hexnet`); γ 1, and a battle cut at the step limit costs each learning
  side 1.0 (against stalling). Off by default after A/B runs that showed no gain: the kill
  preview as a bias of the strike entries (`[network] preview`, AI-Q7 closed) and the turn order
  in the trunk (`[network] order`).
- **The league** (decision 9): prioritized fictitious self-play; half the episodes self-play,
  the rest against one of 4 live members drawn by (1 − p)², p the learner's decayed score
  against it, among at most 16 (a snapshot every 1 M samples and every promoted network; the
  most beaten retires). In the neutral kinds (`[env] kinds`) the system actor defends;
  `[league] system_share` gives it a share of the player kinds too (off by default).
- **The gate** (decision 8): every 10 M samples the learner plays the current best on the gate
  set v1, in pairs of one battle from both sides, until an SPRT decides (Elo 0 against 20, α
  and β 0.05, at most 4 000 pairs); a promoted learner becomes the next best and a league
  member. A training episode on a held-out seed stops the run.
- **The ladder:** each promoted network, playing its sampled policy, against Random, Aggressor,
  Greedy, the system actor and the promoted network before it on the report set v1, and
  attacking the system actor on the neutral set v1, scored at reward v1's settings; the final
  best's on 500 battles per kind. `h3env.train.inspect` sorts a network's battles by kind, side
  and outcome and reads its play (E06 T6a).
- **The battle AI v1** is `t7-1`'s best (20 M samples, a third neutral, 79 minutes; its networks
  on the user's machine). On the closing engine (E06 T9c): 0.830, 0.550, 0.530 against Random,
  Aggressor, Greedy at values +0.273, +0.029, +0.010; on the neutral set 0.970 at +0.244 (Greedy
  +0.224, Aggressor +0.217): decision 7's targets as revised 2026-10-27, "beat every heuristic
  bot". Six levers (γ, the league, the entity network, the system actor as an opponent, the kill
  preview, the turn order) left PPO on the flat observation at the bots' level; the system
  actor, 0.628 against `t7-1`, is E25's target.
- **The search** (`h3env.search`, E06 T8): Gumbel AlphaZero at the root over determinized worlds
  (ADR-0025 point 12a), a saved network's heads as its prior and its value at a leaf. Measured at
  T8c with no gain over the network, a verdict E06's review made inconclusive (a terminal
  scored ±1 against the value head's reward v1; escapes reachable below the root, fixed): E25
  measures again.

## 2. Baselines (needed before RL)

Scripted bots are baselines for evaluation only; training uses self-play and the system actor alone (AI-Q1):

- random legal agent (also the benchmark workload; built, `Policy::Random`),
- simple heuristic battle bots (built in E05: Aggressor and Greedy value, §1.1),
- later: stronger scripted bots, search-based agents (MCTS with the forkable env; E06 T8 built a determinized Gumbel search over worlds, `h3env.search`, whose measurement, no gain over its network, E06's review made inconclusive, `docs/epics/E06-log.md` T8c and T9b; E25 measures again), and references such as the original game's AI and other open-source AIs where comparison is feasible.

## 3. Strategy per problem

**Battle** — well-bounded, short horizon, mostly perfect information. First RL target. Approaches to try in order of expected value: search + learned value/policy (AlphaZero-style with forkable env and action masks), policy-gradient self-play with league opponents, imitation bootstrapped from scripted bots (DAgger-style) since human data is scarce.

**Adventure map** — very long horizon, huge action space, sparse rewards, partial observability, up to 8 players. Do not attempt end-to-end tile-level RL. Use a **hierarchical design**: a learned top-level policy chooses macro-actions (go to object, recruit, build, merge/split armies, engage, …); a deterministic executor (pathing, sequencing) carries them out; battles are delegated to the battle agent. Start with 1v1 on small maps; scale players and map size later.

**Heterogeneous approaches are welcome**: RL where it wins, search/heuristics where they are better.

## 4. Training automation (user gives guidance, not every decision)

Principles:

- The training stack is itself built by coding agents, reproducibly: config-driven runs, seeded, every run recorded (config, git commit, ruleset hash, metrics).
- **Evaluation gate:** new checkpoints are evaluated against a fixed opponent ladder and previous best; promotion to "current best" is automatic when statistically better under pre-declared criteria.
- **League/self-play pool** with past checkpoints to avoid strategy collapse.
- **Automated sweeps** over a small, agreed set of hyperparameters with early stopping; reports summarizing what changed and what improved.
- **Human guidance points** (user decides, rarely): objective/reward definition, evaluation criteria and opponent ladder, compute budget, scenario distribution, which approach family to pursue next. Agents propose options with evidence; the user picks.
- Prefer established libraries/frameworks over bespoke RL code where they fit. Training is Python with PyTorch over the C ABI (Q5, AI-Q1); Rust-native frameworks (burn, candle) stay an option, mainly for running a trained network inside the engine's own processes.

## 5. Compute assumptions (initial)

Local PC: Ryzen 9 3900-class CPU (12 cores / 24 threads), GTX 1070 (8 GB, Pascal; no tensor cores, weak half-precision). Implications:

- Simulation speed is the lever: CPU-side self-play is cheap if the engine is fast; GPU is used for batched inference/training of small-to-medium networks.
- Design networks/observations compactly; do not rely on fp16/tensor cores.
- Pascal (compute capability 6.1): PyTorch 2.14 is the last release with prebuilt wheels for it (the CUDA 12.6 build; PyTorch's notice of 2026-09-04); later releases need a source build. Pin 2.14 until the GPU changes (Q5).
- Long training times are acceptable; cloud scale-out remains possible later because runs are config-driven and the engine is portable.

## 6. Upfront constraints the AI imposes on the engine

1. Determinism and seeds everywhere (reproducible experiments).
2. Cheap `clone`/`fork` of game state; no per-step heap allocation in battle.
3. `legal_commands` as a mask, consistent with `apply` (property-tested).
4. Observation projection per player (no hidden info leaks into agents).
5. Batch-friendly, panic-free C ABI.
6. Stable, versioned observation/action encodings owned by the sim layer, not the domain.
7. Performance benchmarks tracked from the walking skeleton onward.

## 7. Deferred

RL algorithm selection, network architectures, the reward's weights beyond v1 (its mana price is provisional), the exact opponent ladder, league management, multi-agent (>2 players) handling. E05's smoke run chose nothing here for good: its PPO, network and settings only show the environment trainable (E06 decides). E06 (refined 2026-10-20, AI-Q8) chose PPO with a league of past checkpoints (prioritized fictitious self-play), search only if PPO plateaus, and its targets and evaluation gate (`docs/epics/E06-battle-ai.md`). PPO plateaued at the heuristic bots' level whatever the lever, and the search gained nothing over the network (E06 T6, T8): the user revised E06's targets to the evidence and closes it with `t7-1` as the battle AI v1 (2026-10-27); a stronger battle agent is E25's.
