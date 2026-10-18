# E05 — Battle environment: design note (T1)

Working detail behind ADR-0025 (E05 T1, accepted 2026-10-18 with the user's review: AI-Q2,
AI-Q3). A handover, not binding: each task that builds a section deletes its detail here
(`docs/adr/README.md`, "Keep ADRs short"). Sections follow T1's points (a) to (g).

## 1. The batch API (a) — built (T3b, T3c, T3d)

ADR-0025 points 1 to 7 as code: `h3_sim::batch` (`BattleBatch`, `Scenario`, `Seat`, `Policy`,
`EnvReport`, `Outputs`, `episode_seed`, `draw_scenario`; the threads in `batch::pool`) and C ABI
v27 in `h3-ffi`'s `src/batch.rs`, declared in `include/h3.h` (T1's sketch with its names, plus
`H3Seats`, `H3_SEAT_*`, `H3_CURRENCY_*` and `H3_ERR_NOT_READY`; `h3_batch_set_encounters` comes
with T5). Decisions of the build beyond the ADR (the agent's, provisional):

- **Outputs:** the big arrays (observations, entry masks, final observations) are the caller's,
  written in place by the workers; the small per-env values come back as one `EnvReport` per
  env, which the C layer converts into `H3BatchBuffers`' arrays after the call. A null array
  skips its output.
- **Skipped episodes:** an episode that ends before an external seat could act holds no decision
  and is skipped; after 64 in a row (`MAX_SKIPPED_EPISODES`) the env gives up (no side to move,
  status `H3_ERR_EPISODE_OVER`) until the next reset. `reset` restarts every env's seed stream.
- **Threads (T3c):** T std threads (at most N), env i in shard `i * T / N`, parked on a condition
  variable between calls; reset, step and each head call split the envs and the caller's rows
  into contiguous shards and return when every worker is done. The workers write into the
  caller's arrays through one `unsafe` lifetime erasure (`Pool::run`), sound because the call
  blocks until every worker finished; a worker's panic is caught and resumed in the caller after
  that. T = 0 runs in the caller's thread.
- **Scenarios (T3d):** parsed and compiled in `h3-ffi` (`src/scenario.rs`, as T2's); one ruleset
  per packs and profile per batch, reused through the first scenario that compiled it
  (`Scenario::ruleset`), so `h3-ffi` needs no new dependency.
- **Errors:** a refused call changes nothing: `H3_ERR_NULL`, `H3_ERR_ARGUMENT` (with a message for
  create and add) or `H3_ERR_NOT_READY` (no scenario yet, not reset); a step returns
  `H3_ERR_REJECTED` when an env refused its action, its `status` saying why.
- **Properties:** a batch steps as N single envs for random N, T (0 to 7), scenarios, seats, step
  limits, currencies and prices (`h3-sim`'s `batch/tests.rs`); the ABI batch equals the Rust
  batch output by output (`h3-ffi`'s `batch/tests.rs`).

## 2. Seats (b)

- A scenario names a seat per side: `External` (the caller acts: every acting seat the batch
  reports is external) or `Internal(policy)`, played inside the step. At least one seat of each
  scenario is external (checked when the scenario is added or sampled).
- Internal policies are a closed enum in `h3-sim`, so no trait objects and no allocation per move:
  `Random` (uniform over legal commands, the casts' heads included; tests and benchmarks only),
  `System` (E23's policy of the original's computer: E23 provides the function
  `fn(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` in the crate it chooses,
  shared with the adventure's neutral seat in `h3-game`; `h3-sim` calls it), and the baseline bots
  of T7. They choose a `Command`, not an encoded action: an internal seat needs no encoding.
- Internal seats are engine code, not agents: `System` reads what E23's reading says the
  original's computer reads; the bots (T7) read only their side's `Observation` (fair baselines).
- Seats from the encounter distribution (T5): a neutral side (a wandering monster, guards, a
  neutral town's garrison) is `Internal(System)`; a player's side is external. Until E23 the
  neutral share samples `Internal(Random)` and is marked as not for training (BACKLOG order).
- **A level playing field** (the user, AI-Q3): the batch builds every episode's setup with
  `human = [true, true]`, whatever the scenario or the sampler says; `human = false` remains for
  compatibility with the original's computer player (ADR-0023 point 7). `difficulty` stays
  scenario data. This also
  answers BAT-Q33's open point for E05: an agent's side counts as a human player's.
- **No escapes** (the user, AI-Q2): the batch's masks never allow Retreat (entry 1392) or
  Surrender (1393), for external and internal seats alike, and `step` refuses them as an action
  that does not decode. The encoding keeps the entries; a batch option (`escapes`, default off)
  turns them back on when the adventure's view of a battle comes (E13). `BattleEnv` and the
  single-env C functions keep them (tools, replays and the battle lane's benchmarks use them).

## 3. Reward v1 (c) — built (T4)

`h3_sim::reward` (its module documentation states the formula, ADR-0025 point 8):
`r_s = (K_o − K_s − p · (m_s − m_o)) / S`, the currency `ai_value` (default) or the creatures'
cost in the ruleset's money (`Currency::Gold`: the resource surrenders are paid in, gold in the
original's packs, coin in `synthetic`), p 10 by default (`DEFAULT_MANA_PRICE`, provisional, tuned
in E06). K counts the army and raised stacks (`BattleOutcome::survivors`) and the war machines;
created stacks count nothing; a hero that gains mana in the battle is credited at the same price.
Integer sums in `i128` (no overflow for any setup), one division in `f64`, reported as `f32`:
zero-sum exactly. Terminal: only the call that decides a battle carries it, a truncation gets 0
(with both final observations). T1's six worked examples are tests in
`crates/h3-sim/src/reward/tests.rs`, one per example. The Python side accumulates each seat's
rewards between its own moves (§5).

## 4. Scenarios and the encounter distribution (d)

### 4.1 What the ABI loads (T2)

- **A scenario is the battle lane's scenario TOML** (`h3-pack-battle`'s `parse_scenario`, the
  format of `packs/synthetic/scenarios/*.toml` and `h3 run`): packs by id, profile, armies,
  heroes (as `BattleHero` values), terrain, battlefield (special grounds, the deck, the boats),
  `anti_magic` (a garrison), `siege`, war machines, formations, funds, `difficulty`, `human`,
  `random_obstacles`. It already describes every battle the engine plays; E05 changes nothing in
  it.
- **Packs:** the five built-in packs (`synthetic`, `synthetic-ext`, `base`, `ab`, `sod`, compiled
  into the library as today) by id; a ruleset is compiled once per distinct (packs, profile) of a
  batch and shared (`Arc`). Packs from files or texts stay out until a mod needs them.
- T2's single-env entry point: `H3Env *h3_env_create_scenario(const char *toml, size_t len,
  uint64_t seed, uint32_t max_steps, char *error, size_t error_len)` (C ABI v26): null on error
  with the parse or setup error's message in `error`; random obstacles as the scenario says (as
  `h3_env_create_synthetic`). Tests per place: a field, a special battlefield, a garrison, a town
  (closes F-robust-2 and the E01 inbox item).
- **Built (T2):** the scenario's text is parsed and its packs compiled in `h3-ffi` (its module
  `scenario`): `arch.toml` lets `h3-sim` use `h3-pack-battle` only in its tests, so `h3-sim`
  takes a resolved `BattleSetup` and the C ABI layer turns texts into setups (T3's
  `h3_batch_add_scenario` the same way). On error `error` gets the stage and the message
  (`scenario:`, `unknown pack`, `packs:`, `setup:`), on success it is left untouched;
  `h3_env_create_synthetic` takes the same path with its built-in texts. Each call compiles its
  packs: about 18 ms for `base`, `ab` and `sod` (2.6 ms for the synthetic skirmish) in a release
  build on the cloud sandbox, about ten times that in a debug build; the batch's shared rulesets
  (above) avoid it per env.
- **Episode sources of a batch:** (i) a weighted list of fixed scenarios (`h3_batch_add_scenario`:
  text, integer weight, seats); each reset draws one by weight; or (ii) the encounter
  distribution (T5).

### 4.2 The encounter distribution v1 (shape; T5 fills it)

A versioned TOML in the AI lane (`crates/h3-sim/data/encounters-v1.toml`; parsed in Rust, which
needs `serde`/`toml` in `h3-sim` and `h3-heroes` for the heroes: T5's shared pull request), its
weights drawn from a census or stated as assumptions, each with `why`:

```toml
# The shape only: every number here is a placeholder that T5 replaces with a derived or stated one.
[distribution]
id = "encounters"
version = 1
packs = ["base", "ab", "sod"]
profile = "base:faithful"
census = "census-v1"                 # crates/h3-sim/data/census-v1.toml: derived numbers only

[kind]                               # who fights: weights (assumption: per game week, below)
wandering = { weight = 0.50, why = "…" }
guards    = { weight = 0.10, why = "…" }     # guarded piles, later creature banks
hero      = { weight = 0.20, why = "…" }     # hero against hero in the field
siege     = { weight = 0.20, why = "…" }     # a town, with or without a fort or a hero

[week]                               # the game week: army sizes, hero levels, the mage guild
weights = { 1 = 0.20, 2 = 0.16, 3 = 0.13 }  # … (assumption)

[terrain.wandering]                  # census: monster placements per terrain
from = "census.monster_terrain"
[terrain.hero]                       # census: passable land tiles per terrain
from = "census.land_terrain"
[battlefield]                        # census: the share of battles on each special ground
from = "census.special_ground"
[monster]                            # census: placements per level class and fixed creature;
from = "census.monster_level"        # counts: the original's random counts by level and week
[army]                               # value by week (median, spread) and the attacker/defender
value = { … }                        # value ratio (assumption: attackers choose fights)
balance = { … }
[hero]                               # level by week, class from the census's town factions,
level = { … }                        # skills, artifacts by level, spells by the mage guild
[siege]                              # fort by week, a defending hero's share (assumption)
[difficulty]                         # assumption
weights = { 0 = 0.1, 1 = 0.4, 2 = 0.3, 3 = 0.15, 4 = 0.05 }
[seats]                              # §2
neutral = "internal:system"
player = "external"
```

- **Dimensions:** encounter kind; game week; place (terrain, special ground, sea, siege with fort
  level and moat); per side the faction, the army's value and composition by level, a hero or
  none (level, class, primary and secondary skills, artifacts, spells and their masteries, war
  machines, through `h3-heroes`' `derive_battle_hero`); the attacker/defender value ratio;
  difficulty; seats.
- **The census** (T5a): `h3 ai census <maps>` (`h3-cli`'s `src/ai/`, run locally with the user's
  maps through `h3-map`) writes aggregate numbers only (ADR-0009: counts and shares, no map
  content, no per-map rows): land tiles per terrain, monster placements per terrain and level
  class, special-ground tiles per kind, towns per faction, guarded piles, water share, map sizes.
  Each placement counts once (a larger map holds more battles). Its derivation is recorded with
  it; the adventure lane's census (`docs/rules/surveys/E08-battles.md` §1.1) is the precedent.
- A curriculum samples the same dimensions with other weights (an override table of the same
  shape). T5b's check samples many scenarios and compares each dimension's frequencies with its
  weights.

## 5. The Python side (e)

- **ctypes** over the C ABI: in the standard library, no compile step, the same on Linux and
  Windows; `ctypes.CDLL` releases the GIL during each call. cffi's ABI mode would add a
  dependency for the same thing; its API mode a build step per platform.
- **Zero copy:** the binding allocates the numpy arrays of `H3BatchBuffers` once and passes their
  data pointers each call; the engine writes into them. Python never copies an observation.
- **Layout** (`python/`, the AI lane's):

  ```
  python/pyproject.toml   package h3env; requires-python >= 3.10; numpy; group "train": torch 2.14
  python/uv.lock          Linux and Windows, torch from the cu126 index
  python/h3env/_ffi.py    loads the library ($H3_LIBRARY, else ../target/release/), signatures,
                          refuses a library whose h3_abi_version differs from the binding's
  python/h3env/batch.py   BattleBatch: config, scenarios, reset/step, the head calls, numpy views
  python/h3env/seats.py   per-seat bookkeeping: the acting seat per env, each seat's accumulated
                          reward between its moves, episode ends for the seat that did not move
  python/h3env/tools.py   single envs, fork, state hash: trusted tooling (§6)
  python/tests/           pytest: the ABI version, batch vs single envs, masks, a random run
  python/train/           T8's PPO
  ```

- **Environment:** a venv in `python/.venv` with a uv-managed CPython 3.12 (the user's system
  Python is older than 3.11 and stays untouched); `uv sync` installs it; `uv sync --group train`
  adds PyTorch 2.14 from `https://download.pytorch.org/whl/cu126` on Linux and Windows (x86-64),
  pinned in `uv.lock`. The binding's tests need numpy only, so they run in the sandbox (PyPI is
  reachable there; the PyTorch index may not be: T6 locks it on the user's machine if so).
- A Gymnasium-style vector wrapper is a thin layer over `BattleBatch` if a library needs one (T6
  or T8 decides); the factorized, conditioned heads mean most libraries need adapting anyway.

## 6. The information boundary (f)

- The user (2026-10-18): every per-player observation coming out of a battle is already exactly
  the information a human player receives. No check against the original's screens is needed;
  E05 relies on the projection (`h3_battle::observe`, ADR-0008, `docs/06-security.md`).
- The batch API serves only the acting seat's `observe` (and, at a truncation, both sides'), never
  the state, its hash or a fork. Full-state access stays in the single-env functions
  (`h3_env_state_hash`, `h3_env_fork`, `BattleEnv::state`), marked trusted tooling in `h3.h` and
  in `h3env.tools` (exit criterion 3).
- The flat observation is the visible state now; a human also sees the battle's log. A fact that
  only history tells and that matters for play belongs in the observation (BAT-Q35 is one such
  corner, the battle lane's).

## 7. Benchmarks (g)

- **Gated** (instruction counts, `benches/ai/`, `lane_only = true`: the battle lane's changes move
  them): `batch-random-full` — a batch of 64 envs of `packs/synthetic/scenarios/full.toml`, one
  thread, 1 000 calls of random legal actions chosen from the batch's own masks (casts through the
  heads), observations and masks written, counted inside `h3_sim::batch::BattleBatch::step`;
  `batch-random-siege` the same on the siege. T5 adds `batch-random-encounters-v1`.
- **Recorded** (wall time, no gate): `h3 ai bench --envs N --threads T --calls K` prints steps/s
  inside `BattleBatch::step` and with the head calls and the random choice; on the user's Ryzen 9
  3900X for T = 1, 6, 12, 24 and N = 64, 256, 1 024 at T9 (moved from T3 by the user, 2026-10-18;
  the sandbox's indication in the log, T3c), and the Python binding's steps/s against Rust's
  (T6), in the epic log.
- **Built (T3c):** both gated workloads (`benches/ai/`, lane-only, threads 0 so that every count
  is in the caller's thread; collect `*BattleBatch>::step`, never inlined).
- The single-env gates (the battle lane's `random-rollouts-*`) stay as they are.
