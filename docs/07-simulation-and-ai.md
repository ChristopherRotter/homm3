# 07 — Simulation environment and AI

The AI is the main motivator of the project. It is not the first thing built, but it shapes upfront decisions (determinism, cheap fork/clone, action masks, batch-friendly API, observation projection).

## 1. Simulation environment (`h3-sim`)

A thin, allocation-conscious wrapper over the game logic:

- `reset(scenario, seed) -> Observation`
- `step(actions) -> (Observation, reward, done, info)` for each environment in a batch
- `legal_actions(env) -> mask` (from `legal_commands`; the action encoding is defined by the sim layer, not the domain): dense entries for every command but a cast, and a factorized cast whose spell and target heads have masks conditioned on the earlier choices (ADR-0014 point 9; `h3_sim::action`)
- `fork(env) -> env` (cheap clone for search/rollouts) and `state_hash(env)`
- Two environment families: **BattleEnv** (from a `BattleSetup`; first) and **AdventureEnv** (E08 T9a: a game on a map through `h3-game`, `h3-sim`'s `src/adventure/`; its replays record the game commands, not the env's actions, and `h3 adventure run/verify/bench` drives it, T9b; action encoding v2 adds the answers to an open decision, an entry that takes no tile has tile 0 alone, and a replay records the game's difficulty, T14b; encoding v3 adds the battle phase, in which a player in a battle acts with the battle's own encoding for its side and `to_move` yields the owner of the side to act, and replay format v3 records each command's actor and battle commands, T19b1)
- Vectorized/batched stepping in Rust, parallel across instances; single-threaded per instance
- Observations are **per-player projections** (same function as the security layer), so agents never see hidden information
- **Choices the battle makes for a side as the original's computer** (ADR-0023 point 7; E04c): the automatic turrets' and Ballista's targets read `BattleSetup.difficulty` and `.human`, which never decide who commands a side. The defaults (Easy, no human side) are the original's battle between two computer players, in which those targets count a computer side's walker of few hit points as keeping out of the moat, and that walker's berserk walk keeps out of it too (BAT-SIE-022, BAT-Q50): a `BattleEnv` for a human player's battles sets `human`
- Reward is computed in the sim layer from events/state; the domain knows nothing about rewards
- Exposed to other languages via the C ABI (`h3-ffi`), batch-oriented to amortize call overhead

The scenario distribution matters as much as the engine: training on few fixed setups overfits. Battle setups are sampled from generators (armies, terrain, heroes, artifacts, spells); adventure training needs generated maps (epic: map generator, seeded by the user's existing project, optimizing zone balance and minimal teleport connections among other objectives).

## 2. Baselines (needed before RL)

Scripted bots are required for bootstrapping, evaluation and the self-play opponent pool:

- random legal agent (also the benchmark workload),
- simple heuristic battle bot (e.g., greedy damage/value),
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
- Prefer established libraries/frameworks over bespoke RL code where they fit; the choice of training language/framework is deferred (the C ABI keeps it open).

## 5. Compute assumptions (initial)

Local PC: Ryzen 9 3900-class CPU (12 cores / 24 threads), GTX 1070 (8 GB, Pascal; no tensor cores, weak half-precision). Implications:

- Simulation speed is the lever: CPU-side self-play is cheap if the engine is fast; GPU is used for batched inference/training of small-to-medium networks.
- Design networks/observations compactly; do not rely on fp16/tensor cores.
- Verify framework support for Pascal (compute capability 6.1): newer CUDA toolkit releases have been dropping older architectures, which can constrain framework versions.
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

RL algorithm selection, network architectures, training framework/language, reward design details, the exact opponent ladder, league management, multi-agent (>2 players) handling.
