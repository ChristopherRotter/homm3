# E05 — Battle environment v1

Status: **in progress** 2026-10-18 (AI-Q1; increment 1: T1 drafted, waiting for the user's review; T2 next) · Lane: **ai**
(opened with this epic) · Follows E04c (closed 2026-10-16) · Followed by E23 (the original's battle
AI, the system actor) and E06 (training and proficiency)

## Goal

The battle agent's environment, complete enough to train on: every battle the engine plays, built
and sampled the way battles occur in games, stepped in batches through the C ABI from Python, with
a reward that values what a real battle is about, and evidence that an agent learns from it. The
engine keeps its architecture: determinism, validated commands, per-viewer projections, the
benchmarks.

## Decisions at the refinement (the user, 2026-10-18, AI-Q1)

1. **Battle only.** The adventure env's C ABI and encodings stay out (E13).
2. **The AI lane opens now** and takes `h3-sim` and `h3-ffi` (`h3-sim`'s `src/adventure/` stays the
   adventure lane's), `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
3. **Training stack** (Q5): Python with PyTorch, over the C ABI through a thin cffi or ctypes layer
   (ADR-0011 stays). PyTorch **2.14** with its CUDA 12.6 build is the last release with prebuilt
   wheels for the GTX 1070's Pascal GPU (PyTorch's notice of 2026-09-04): pin it. Considered: JAX
   (Pascal wheels on CUDA 12, but no native Windows GPU support), tch-rs (libtorch, the same limit),
   burn and candle (Rust-native: no ABI boundary, a thin RL ecosystem; an option for running a
   trained network inside the engine later).
4. **Opponents.** Agents face each other (a hero against a hero) and the **system actor** (a neutral
   side; the original's computer, E23). The batch API has two seats per env: an *external* seat is
   played by the caller, an *internal* one by a policy inside the engine within the step. Until
   E23's policy exists, the internal seat plays at random, for tests only.
5. **Casts in the batch API:** one call per head (the choice of entry, then the spell, the target,
   the second target), each answered for every env of the batch at once, conditioned on the earlier
   choices (ADR-0014 point 9). No encoding change; it matches a network that chooses its heads one
   after another, and a search sees a cast as one action. Fallback if a library needs one discrete
   choice per step: a cast as sub-steps (a new action encoding, the partial choice in the
   observation). Rejected: precomputed joint masks (spell × target, mostly wasted, costly).
6. **Observation: start simple.** The flat encoding (v21) as it is. Alternatives noted for later:
   stacks as a set of entities plus hex planes (for attention networks), encoding only what changed
   (the encoding costs about a third of a step today).
7. **Scenario distribution.** An agent must master every battlefield and sieges, every army size and
   balance, wandering stacks and heroes, each **sampled by its approximate encounter probability**.
   An encounter distribution v1 approximates them until full games run; then the adventure's
   logged encounters give the real types and frequencies. v1 is a versioned config whose weights
   are derived from the 160 maps shipped with the game (terrain shares, neutral stacks, towns, read
   with `h3-map`) and from stated assumptions where maps say nothing (game week, hero levels, army
   ratios). A curriculum may sample it with other weights.
8. **Heuristic bots are baselines, never training opponents.** Training uses self-play and the
   system actor only.
9. **Reward: the least cost for the greatest gain.** One currency (creature `ai_value`, harvested);
   a side's reward is the value it destroys minus the value it loses, minus the mana it spends at a
   price, with escapes priced: an enemy hero who retreats or surrenders keeps his hero, artifacts
   and (on surrender) army, which counts against the winner, so blocking escapes or hurting the
   enemy before he flees pays. Capturing the loser's artifacts and the hero's own value make
   winning worth something without a separate bonus. **Zero-sum in self-play** (one side's reward
   is the other's negative); against the system actor only the agent's side is scored. Terminal by
   default, from the battle's outcome; optional shaping only in potential-based form (it cannot
   change the best play). The weights (mana price, hero value, escape weight) are configurable; the
   adventure AI may later pass them as an input (one agent for different situations).
10. **Information:** an agent sees exactly what a human player sees in the original's battle, as
    data (the projected observation), never the full state. `fork` and `state_hash` see everything:
    tools and training only (an agent's search would have to guess the hidden part: E06).
11. **Performance** is measured and experimented with, no fixed target: E05 adds batched workloads
    (steps/s over 12 cores with observations and masks); the single env's gates stay.
12. **Exit:** the environment is shown to be trainable (a smoke run learns); proficiency targets
    belong to E06 (a held-out encounter set, a ladder of the random and heuristic bots, the system
    actor and earlier checkpoints, rated by win rate and value efficiency).

Unchanged: one epic in increments with a user checkpoint each, only the current increment refined
(Q41's pattern); the seam and ADRs are reviewed by the user before work builds on them; token
efficiency beats speed.

## Starting point (from E01–E04c)

- `BattleEnv` (`h3-sim`): `reset`, `step`, `fork`, `state_hash`, per-viewer `observation` and its
  flat encoding v21, the factorized action (entry, spell, target, second; action encoding v13) with
  conditioned masks, reward ±1 per side, random rollouts, replays (format v29), benchmarks
  (≈ 131 k steps/s on the user's Ryzen 9 3900X, `full`; the flat observation costs about a third of
  a step).
- C ABI v25 (`h3-ffi`): one env per handle, only the synthetic scenarios 0 to 2, no siege
  (F-robust-2); no batch, no pack or scenario loading.
- `BattleOutcome` reports survivors, every stack's record, war machines, the surrender cost and each
  hero's mana; `ai_value` and `fight_value` are harvested for every creature.
- `BattleSetup.human` and `.difficulty` (ADR-0023 point 7) shape the computer's automatic choices;
  BAT-Q33 (who commands a side) is open.
- Inherited inbox items: the observation's cost, `state_hash` too slow for per-node hashing, the
  lopsided `full` workload, batch stepping and pack loading through the ABI (E01), the VCMI oracle
  harness (revisit with evaluation).

## Non-goals

The adventure env's C ABI and encodings (E13). E23's policy (E05 provides the internal seat and a
random placeholder). Training at scale, the league, the evaluation gate and proficiency targets
(E06). A structured observation (noted alternative). Search agents and determinization (E06). A
trained network running inside the engine or the server. A faster state hash, unless a measurement
in E05 needs it.

## Tasks (outline; refine at the start of each increment; keep status current)

**Increment 1:** T1, then T2 after the user's review.

- [ ] **T1 Design and ADR draft** — **drafted 2026-10-18, in the user's review:** the ADR draft
  `docs/lanes/ai/adr-drafts/battle-env.md`, the design note `E05-design.md` (worked reward
  examples), AI-Q2 (the reward's defaults), AI-Q3 (an agent's `human` flag); (f) settled by the
  user (the projection is a human's view). The task as refined: the draft for the user's review,
  then **stop**. It decides: (a) the batch API (a handle of N envs; buffers per call
  for observations, masks, rewards, dones and the acting seat; the head calls of decision 5;
  automatic reset; threads only in the adapter, where and how many; a batch must step bit for bit
  as N single envs); (b) the seats (external, internal; how the internal policy plugs in, so E23's
  policy and the random placeholder share one interface; `human` and `difficulty` per seat and
  BAT-Q33's open point: what a self-play agent's side counts as); (c) the reward (decision 9 worked
  out against `BattleOutcome`: the currency for war machines, created, raised and summoned stacks;
  the mana price; the hero's and the artifacts' values; what the outcome does not report yet, if
  anything; worked examples); (d) the scenario description the ABI loads (packs plus a scenario
  file, every place including sieges) and the encounter distribution's config shape (its
  dimensions, how weights are derived, how the maps' statistics are kept: derived numbers only,
  ADR-0009); (e) the Python side (cffi or ctypes, zero-copy numpy buffers, the package layout in
  `python/`, a lock file pinning PyTorch 2.14 cu126 for Linux and Windows; the user's Python is
  older than 3.11: a venv); (f) the information boundary (decision 10: check the projection against
  what the original's battle screen shows a human, the hero and stack windows, and list
  differences); (g) the benchmark workloads. Readings it needs go into the draft; questions as
  `AI-Q<n>`.
- [ ] **T2 Scenarios through the C ABI** — after the review: create an env from packs (`base`, `ab`,
  `sod`, the synthetic ones) and a scenario as T1 decides, any battle the engine plays including
  sieges (closes F-robust-2 and the E01 inbox item); C ABI v26; the C smoke test and tests per place.

**Later increments** (coarse; cut at each checkpoint):

- **T3 The batch API** in Rust and the C ABI (seats, head calls, automatic reset, the random internal
  seat), its determinism property against single envs, `benches/ai/` workloads.
- **T4 Reward v1** in the sim layer, configurable weights, tests against hand-computed outcomes;
  the zero-sum property in self-play.
- **T5 Encounter distribution v1:** T5a the statistics of the 160 shipped maps and the stated
  assumptions, committed as derived numbers with their derivation; T5b the setup sampler (armies,
  heroes with levels, skills, artifacts and spells through `h3-heroes`, battlefields, sieges,
  neutral stacks) and a sampled-scenario check that every place occurs at its weight.
- **T6 The Python binding** (`python/`): the batch API as numpy arrays, a vector interface for
  self-play (the acting seat per env), tests on Linux and on the user's Windows machine.
- **T7 Baselines:** heuristic bots (greedy damage or value) and an evaluation runner (match series,
  win rate, value efficiency, Elo between policies); where bots live is decided in T1 (a new crate
  would be a shared change).
- **T8 Smoke training:** a small PPO in PyTorch 2.14 on a small encounter family; exit criterion 7.
- **T9 Closure:** an independent review, measurements on the user's machine, the docs.

## Exit criteria

1. Every battle the engine plays (every battlefield, sieges) can be created through the C ABI from
   packs and a scenario, and sampled from the encounter distribution v1 (versioned, its derivation
   recorded).
2. The batch API: N envs per handle, the acting seat per env, the head masks for every env in one
   call, automatic reset, internal seats; a property that a batch steps bit for bit as N single
   envs, and that every legal action applies.
3. Only projected observations cross the batch API; full-state access is marked trusted tooling.
4. Reward v1 as decided, zero-sum in self-play, tested on hand-computed outcomes.
5. The Python binding runs on Linux and on the user's Windows machine, PyTorch 2.14 pinned.
6. The heuristic bots and the evaluation runner exist and are recorded as baselines.
7. A smoke training run (proposed, provisional: within an hour on the user's machine) wins at least
   80 % against the random policy on held-out seeds of its encounter family.
8. Batched benchmarks recorded on the user's machine; no single-env gate regresses.
9. `cargo xtask verify` passes on Linux and Windows, including `--with-original-data`.

## Risks

- **Pascal:** PyTorch 2.14 is the last prebuilt release for the GPU; pinned, a later GPU lifts it.
- **The encounter approximation** may be wrong in ways that matter; retrain when the adventure's
  logged encounters exist (decision 7).
- **The reward's weights** shape the play strongly (a cheap mana price makes a caster spend
  freely): configurable, reported with every run, the user decides the defaults at T1's review.
- **Ownership:** battle rules that change the encodings now cross lanes (`cross-lane` pull requests
  from the battle lane, `docs/08-work-process.md`).
- **The internal seat before E23:** training on random neutrals would teach the wrong lessons; the
  neutral share of training waits for E23 (BACKLOG ordering, 2026-10-18).
