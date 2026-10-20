# 07 — Simulation environment and AI

The AI is the main motivator of the project. It is not the first thing built, but it shapes upfront decisions (determinism, cheap fork/clone, action masks, batch-friendly API, observation projection).

## 1. Simulation environment (`h3-sim`)

A thin, allocation-conscious wrapper over the game logic:

- `reset(scenario, seed) -> Observation`
- `step(actions) -> (Observation, reward, done, info)` for each environment in a batch
- `legal_actions(env) -> mask` (from `legal_commands`; the action encoding is defined by the sim layer, not the domain): dense entries for every command but a cast, and a factorized cast whose spell and target heads have masks conditioned on the earlier choices (ADR-0014 point 9; `h3_sim::action`)
- `fork(env) -> env` (cheap clone for search/rollouts) and `state_hash(env)`
- Two environment families: **BattleEnv** (from a `BattleSetup`; first) and **AdventureEnv** (E08 T9a: a game on a map through `h3-game`, `h3-sim`'s `src/adventure/`; its replays record the game commands, not the env's actions, and `h3 adventure run/verify/bench` drives it, T9b; action encoding v2 adds the answers to an open decision, an entry that takes no tile has tile 0 alone, and a replay records the game's difficulty, T14b; encoding v3 adds the battle phase, in which a player in a battle acts with the battle's own encoding for its side and `to_move` yields the owner of the side to act, and replay format v3 records each command's actor and battle commands, T19b1; since T20b3a `to_move` yields seats, `h3_sim::adventure::Seat`: a player, or the neutral seat for a battle's unowned side, played as the system actor with `neutral_observation`, and the random agent and the recorder play it too)
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
  training run's record (C ABI v31, E06 T2c).
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
  neutral kinds are built and weigh 0 until E23's policy plays them. Encounters v2 will come from
  the adventure's logged battles.
- **Reward v1:** `r_s = (K_o − K_s − p · (m_s − m_o)) / S` in `ai_value` (or the creatures' cost),
  K a side's losses including the war machines the battle placed, m its mana spent, p 10 per
  spell point (provisional), S both sides' value at the start; terminal, 0 after a truncation,
  zero-sum exactly.
- **Information:** only projected observations cross the batch (the acting side's, both sides' at
  a truncation). The fork, the state hash and the episode seeds are trusted tooling (`h3.h`,
  `h3env.tools`), never an agent's input.
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
- **Held-out sets** (E06 T3a, `docs/epics/E06-design.md` §5): `h3 ai eval --set` and the Python
  ladder play the report set v1 (by default: per kind with a player on both sides 500 battles
  from both sides, the kinds weighed 4 : 2 : 1, the win rate with its 95 % interval) or E05's v0;
  the trainer checks every training episode against both and the gate set v1. The bots' table on
  the report set is committed (`python/records/bots-report-v1.json`): Greedy 0.832 ± 0.013 and
  Aggressor 0.810 ± 0.014 against Random, Greedy 0.533 ± 0.018 against Aggressor.
- **Training runs** (E06, ADR-0026; `python/README.md`, "Training runs"): TOML configs, budgets
  and cadences in samples, records committed in `python/records/`, checkpoints that resume after
  any stop; an SPRT gate on the gate set promotes the learner over its best, and each promoted
  network's ladder plays the bots and the network before it on the report set (E06 T3b).
- **Throughput** (wall time, not gated; the gates count instructions in `benches/ai/`):
  on the user's Ryzen 9 3900X (Windows, release, the encounter distribution) about 50 000 steps/s
  on one thread whatever the batch size, and inside the step 387 000 on 12 threads and 497 000 on
  24 at 1 024 envs (233 000 on 6); Python's binding is within 2 to 7 % of Rust inside the step
  (`E05-log.md`, T9c).

## 2. Baselines (needed before RL)

Scripted bots are baselines for evaluation only; training uses self-play and the system actor alone (AI-Q1):

- random legal agent (also the benchmark workload; built, `Policy::Random`),
- simple heuristic battle bots (built in E05: Aggressor and Greedy value, §1.1),
- later: stronger scripted bots, search-based agents (MCTS with the forkable env), and references such as the original game's AI and other open-source AIs where comparison is feasible.

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

RL algorithm selection, network architectures, the reward's weights beyond v1 (its mana price is provisional), the exact opponent ladder, league management, multi-agent (>2 players) handling. E05's smoke run chose nothing here for good: its PPO, network and settings only show the environment trainable (E06 decides). E06 (refined 2026-10-20, AI-Q8) chose PPO with a league of past checkpoints (prioritized fictitious self-play), search only if PPO plateaus, and its targets and evaluation gate (`docs/epics/E06-battle-ai.md`).
