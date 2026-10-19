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

## T3c — Threads and benchmarks (2026-10-18)

- **Built:** `h3_sim::batch::pool` (design §1): T std threads (at most N), env i in shard
  `i * T / N`, parked on a condition variable between calls, T = 0 the caller's thread; reset,
  step and the three head calls run shard by shard and write into the caller's arrays (one
  `unsafe` lifetime erasure in `Pool::run`, sound because the call blocks until every worker is
  done; a worker's panic is resumed in the caller after that). `BatchConfig.threads`,
  `BatchError::Threads`. `h3 ai bench --envs N --threads T --calls K [--scenario] [--seed]
  [--max-steps]` (`h3-cli`'s `src/ai/`; `pub mod ai;` in `lib.rs` and the dispatch in `main.rs`
  are the battle lane's lines: `cross-lane`, the user's consent).
- **Tests:** the determinism property draws T (0 to 7) with N (1 to 6); the pool's shards, its
  in-place writes over repeated calls and a worker's panic; the bench's steps equal whatever T.
- **Gated** (lane-only, `--threads 0`): `batch-random-full` 11.45 G Ir and `batch-random-siege`
  12.30 G Ir for 64 000 steps each (64 envs, 1 000 calls), counted inside `*BattleBatch>::step`.
- **Wall time on the user's machine:** moved to T9 (the user, 2026-10-18: the sandbox has four
  cores). The sandbox's indication (4 vCPUs of a shared Xeon at 2.1 GHz, `full`, 64 000 steps per
  row, release; thousands of steps/s inside `BattleBatch::step` / with the head calls and the
  bench's random choice, which runs in the caller's thread):

  | N \ T | 0 | 1 | 2 | 4 |
  |---|---|---|---|---|
  | 64 | 55 / 36 | 46 / 28 | 77 / 43 | 77 / 41 |
  | 256 | 44 / 25 | 49 / 28 | 85 / 44 | 97 / 49 |
  | 1 024 | 53 / 31 | 46 / 27 | 86 / 47 | 118 / 58 |

  One worker costs about 10 % against the caller's thread; four give 2.2 × at N = 1 024. The
  bench's choice (a scan of every mask) is serial and caps the second figure: a learner's policy
  replaces it.

## T4 — Reward v1 (2026-10-18)

- **Built:** `h3_sim::reward` (design §3): `Balance::of` (K, m and S per side from the setup and
  the `BattleOutcome`) and `rewards`, `Currency::AiValue` (default) or `Gold` (the creatures' cost
  in the ruleset's surrender resource), `RewardConfig { currency, mana_price }` with
  `DEFAULT_MANA_PRICE` 10, in `BatchConfig.reward`; `EnvReport::reward` carries it on the call that
  decides a battle, 0 otherwise. `BattleEnv::setup()` lets a single env compute the same.
- **Decisions of the build:** sums in `i128` instead of the sketch's `i64` (a scenario's counts
  and values are `u32`: no overflow and no panic for any setup); a hero that gains mana (spent
  < 0) is credited at the same price.
- **Tests:** the six worked examples of T1's design, one test each (a wandering monster 0.102,
  hero against hero 0.329 and 0.285 without the Ballista, summons, a siege 0.190, a draw, raised
  Demons); the gold currency, a ruleset handle outside the ruleset, empty armies (S = 1); the
  determinism property draws the currency and the price and checks the zero sum on every call.
- **Benchmarks:** the reward at each episode's end moved the batch workloads by −0.02 % and
  +0.01 % (re-baselined).

## T3d — The C ABI v27 (2026-10-18)

- **Built:** `h3-ffi`'s `src/batch.rs` (design §1): `h3_batch_create` (`H3BatchConfig` with
  `H3RewardConfig`), `h3_batch_add_scenario` (a scenario's text, its weight and `H3Seats`:
  `H3_SEAT_EXTERNAL`, `H3_SEAT_RANDOM`), `h3_batch_reset`, `h3_batch_spell_mask`, `_target_mask`,
  `_second_mask`, `h3_batch_step`, `h3_batch_episode_seed`, `h3_batch_destroy`; `H3BatchBuffers`
  (twelve arrays, each optional); `H3_ERR_NOT_READY`; `include/h3.h`. Rulesets shared per packs
  and profile (ADR-0025 point 9) through `h3_sim::Scenario::ruleset`.
- **Tests:** the ABI batch equals the Rust batch output by output over 300 calls (the duel, the
  skirmish against a random defender, the siege with a random attacker; two threads; gold at 7
  per spell point; decided and truncated episodes); shared rulesets and the `base` siege of T2
  through a batch; every function's null, argument and not-ready errors, refusals per env. The C
  smoke test plays a batch of 4 envs on two threads, 300 calls (21 episodes, all decided; the
  rewards zero-sum, every status `H3_OK`).

## Increment 3 cut (2026-10-18)

- The user merged increment 2 (#252) and chose the census (T5a) over the Python binding (T6a) for
  the session, while the device link held the maps; the Python tests stay a lane command
  (`uv run pytest`), not a `verify` step, until a later shared change.
- `download.pytorch.org` is refused (403) from the cloud sandbox and from the device link's VM:
  T6b locks PyTorch on the user's Windows machine.

## T5a — The census and the assumptions (2026-10-18)

- **Built:** `h3 ai census <maps folder or files> [--out <file>]` (`h3-cli`'s `src/ai/census.rs`,
  design §4.3): each map through `h3_map::read` and `import_map` into a `GameState` of one player
  on the original packs; aggregate counts only; every battle place from
  `AdventureState::battle_ground` of the defender's tile (the monster's or the pile's visited
  tile). `crates/h3-sim/data/census-v1.toml`: the 159 maps of the user's
  `original-data/install/Maps` (copied from `maps.tar` through the device link), none skipped,
  25 s in a debug build.
- **Decisions of the build:** the tutorial (`Tutorial.tut`) is left out (a folder's `.h3m` files
  only); towns, heroes, garrisons, banks and boats come from the raw model's classes (the
  static layer imports no town or hero), a town's faction from its subclass in the original's
  order, the anti-magic garrison as subclass 1 (VCMI's `Obj::GARRISON`), the fort from the
  standard set's flag or the custom set's bits 3 to 5 (`docs/formats/h3m.md`); an *open* tile is
  one no object blocks or is visited from; the twins (35 maps whose terrain equals an earlier
  one's) are counted, as leaving them out moves no share by more than 1.6 points (a one-off run
  over the 117 maps without an allied name).
- **Checked against E08's census** (164 maps then, `docs/rules/surveys/E08-battles.md` §1.1):
  random monsters by level within 25 placements of it (level 1 2 204 against 2 215, any 63
  against 88), fixed creatures 1 483 against 1 570, fixed counts 740 against 782; E08's five
  more files included maps that are not the game's (`experiment`).
- **Assumptions:** A1 to A12 in design §4.4 (AI-Q4), with the facts behind them: the factions'
  weekly growth values from the packs, the wandering monsters' weekly growth from VCMI 1.7.5
  (`CGCreature::newTurn`, `gameConfig.json`: 10 %, at most 4 000), the split's ratio table
  (`h3_adventure::split`).
- **Tests:** `h3-cli`'s `ai::tests`: a map written by `h3_map::writer` (water, a coast, snow, a
  Clover Field, three fixed creatures, a Stronghold with a citadel, a Castle without a fort)
  counted twice (its twin) with a broken file skipped: tiles, places, levels, forts, the TOML
  read back, the command line; the fort levels of custom buildings.

## T5b1 — The sampler (2026-10-19)

- **AI-Q4 answered** (the user, 2026-10-19; design §4.4): A3 ramps the neutral levels by week
  (stacks differ by level from the start; no level 7 early), A4's ratio normal around 1.8 (sd 0.5,
  at least 0.5), A5 changed at the review (the growth of the levels A6 makes available, summed
  over the weeks, k median 1.0), A12's neutral kinds built but weighted 0 until E23 (after E05);
  the rest as proposed. T5b split into T5b1 (this) and T5b2 (the batch's source, C ABI v28).
- **Shared first:** #254 (`arch.toml`): `h3-sim` may use `h3-heroes`, `serde`, `toml`; `h3-ffi`
  `h3-game` and `h3-pack-adventure` for T5b2, so that one shared merge serves both.
- **Built:** `crates/h3-sim/data/encounters-v1.toml`, `h3_sim::encounter` (`EncounterTable`,
  `Sampler`, `Encounter`, `Facts`, `Kind`), design §4.2. Decisions of the build, each in §4.4's
  *(T5b1)* notes: sieges as three kinds (a hero defends, an owned town's garrison, a neutral
  town), so A12 weighs the neutral one 0; the random towns spread over the nine factions (no
  shipped map names a Conflux town); spells per guild level 6 − level and the guilds' maxima
  from VCMI 1.7.5 (`spellsAtLevel`, the factions' `mageGuild`); the moat, structures, morale and
  terrain of a town; quantile tables for every continuous draw (integers only).
- **Found while testing:** an army split by growth value lost the value of the levels it could
  not buy one creature of (a week-4 army worth 6 730 came out at 2 736): the remainder now goes to
  the level below.
- **Tests:** `encounter::tests` (design §4.2, "The check"); 10 tests, 13 s in a debug build.
- **Verify:** the baseline full verify of `master` (1f32f10) failed only coverage, on the
  adventure lane's `h3-game/src/state.rs:569` (a pending Pass offer's hash tag; 0 hits in
  `cargo llvm-cov -p h3-game`), which #254's full verify covered: flaky, a property test reaches
  it now and then; queued for the adventure lane. #254's full verify passed every step; the
  lane's scoped verify every step but `check-lane` (the shared commits, until #254 merges).

## T5b2 — The batch's encounter source (2026-10-19)

- In the session of T5b1, as its cut allowed when T5b1 finished early (the user, 2026-10-19).
- **Found before building:** the batch draws an episode's entry from stream `SETUP` of the
  episode's seed, the sampler drew from the same stream: mixed with fixed scenarios, the entry's
  draw and the encounter's kind would have been correlated. A kernel stream `ENCOUNTER` (id 6)
  went into the shared #254 with a clause in ADR-0025 point 4; the sampler draws from it.
- **Built:** `Scenario` holds a fixed battle or the encounter distribution
  (`Scenario::encounters`); `Scenario::episode` gives an episode's env, seats and kind;
  `setup()` and `seats()` are `None` for the distribution; `EnvReport.encounter`. C ABI v28:
  `h3_batch_add_encounters` (`h3-ffi` compiles the adventure part, `h3-pack-adventure`, #254),
  `H3BatchBuffers.encounter` (appended), `H3_ENCOUNTER_*`; `h3.h` and the C smoke test.
  `h3 ai bench --encounters default|a,b,c,d,e,f`; the workload `batch-random-encounters-v1`.
- **Measured** (release, the sandbox): a sample 50 µs at the default weights, 26 µs at A1's
  shares (the hero kinds derive two heroes); an episode's start with its env 160 µs; the
  workload 11 858 763 279 instructions (`full`: 11 457 336 633).
- **Tests:** `h3-sim`'s `encounters_play_as_single_envs` (the encounter source mixed with a fixed
  scenario, threads 0 and 2, bit for bit as single envs of `Scenario::episode`; the reports'
  kinds), `h3-ffi`'s `encounters_are_an_episode_source` (errors, shared ruleset, curricula, the
  kinds per entry, a neutral defender played inside the call), `h3-cli`'s bench and command-line
  tests, the C smoke test.

## T6a — The Python binding on Linux (2026-10-19)

- The user chose T6a for the session, with a `shared/` pull request for the batch workloads'
  rows in `benches/README.md` (`shared/bench-ai-workloads`).
- **Built:** `python/` (package `h3env`, design §5): `_ffi.py` (every C ABI v28 signature, the
  layouts, `ABI_VERSION`, refusal of another version), `BattleBatch` (`batch.py`: scenarios and
  the encounter source, reset, step, the head calls, numpy arrays allocated once and written in
  place, `random_actions`), `SeatBook` (`seats.py`), `tools.Env` (single envs, fork, hash),
  `bench.py`; `pyproject.toml` (uv, `uv_build`, requires-python ≥ 3.10, numpy, the group `dev`
  with pytest and ruff), `.python-version` 3.12, `uv.lock` (Linux and Windows on x86-64; numpy
  2.2.6 for Python 3.10, 2.4.6 for 3.11, 2.5.4 from 3.12). The group `train` waits for T6b.
- **Tests** (`uv run pytest`, 19, 2.5 s): the ABI version and a refused one, the layouts against
  `h3.h`; the batch bit for bit as single envs (`tools.Env` from each episode's seed) on `full`
  and `siege` with both sides human (observations, entry masks without the escapes, the head
  masks, `to_move`, done, results, truncations' final observations, automatic resets), the
  thread count changing nothing, a reset replaying, escapes refused without change, the
  encounter source (default kinds, the neutral kinds' internal defender, mixed with a
  scenario), errors with the engine's messages; `SeatBook` (every transition closes once,
  terminal rewards zero-sum, an internal seat without transitions); fork and hash.
- **Measured** (release, the sandbox's four cores, beside the baseline's benchmark gate on one;
  `h3 ai bench` and `python -m h3env.bench` alternated, `full`, both seats external; steps/s
  inside the step call / with the head calls and the random choice; the choices differ, so do
  the episodes): 64 envs, threads 0, 1 000 calls, seeds 1–3: Rust 42 000–50 000 / 27 500–32 500,
  Python 37 000–42 000 / 21 800–24 400; 256 envs, threads 2, 300 calls, seeds 1–2: Rust 42 000–
  45 200 / 23 000–24 900, Python 43 500–53 400 / 22 800–27 700; the encounter distribution, 64
  envs, threads 0: Rust 38 400 / 27 400, Python 31 500 / 20 200. ctypes costs microseconds per
  call (a call is about 1.5 ms here): with threads the two match within the noise; single-threaded
  Python is 10–20 % slower inside the call too, perhaps the Python loop's memory traffic between
  calls (not measured further). The random choice in numpy costs about 0.3 ms per call of 64 envs
  (a cumulative-sum sampler cost 0.66 ms for the entries alone); the head calls are the engine's
  (QUEUE inbox).

## The head calls from the kept entry mask (2026-10-19, T6a's inbox)

- The user chose it with T6b. `BattleBatch`'s spell, target and second masks regenerated each
  env's legal commands to answer "0 only" for an entry that is no cast. The slot now keeps the
  single env's entry mask of the last call (escapes included; only the output row masks them
  out) and answers those entries from it (`plain_head`); casts still ask the env (design §1).
- **Test:** `the_head_calls_answer_as_the_single_envs` compares all three heads with the single
  env's on `full` and `siege` for every legal entry, the escapes, the casts, illegal and
  out-of-range entries and non-zero earlier heads (it fails when the earlier heads or the
  escapes are ignored); a given-up env answers nothing for a cast too.
- **Measured** from Python (release, 64 envs, `full`, 1 000 calls, sandbox): the three head calls
  1.40–1.45 → 0.11–0.15 ms per call; `python -m h3env.bench` with the head calls and the random
  choice 15 300–16 200 → 24 100–24 400 steps/s (the same episodes; inside the step unchanged
  within the noise).

## T6b — PyTorch in the lock (2026-10-19)

- **The index:** refused since 2026-10-18 although `*.pytorch.org` was in the environment's
  Allowed domains. The proxy's 403 body (hidden by curl; a raw `CONNECT` shows it): "no rule or
  allowlist entry allows host". The reachable hosts were exactly the Trusted defaults (conda and
  githubusercontent yes, huggingface.co no): the network access was **Trusted**, which ignores
  the Allowed domains (only **Custom** uses them). The user set Custom; the running session
  reached the index within the minute. PyPI's torch 2.14 (CUDA 13 on Linux) and conda-forge were
  reachable but are not the cu126 build.
- **Built:** the group `train` (`torch==2.14.*`, `tool.uv.sources` from the `explicit` index
  `pytorch-cu126`), locked as 2.14.1+cu126 (Python 3.10–3.15, Linux x86-64 and win_amd64; the
  Linux CUDA 12.6 runtime `cuda-toolkit` 12.6.3 and `nvidia-*-cu12`, triton 3.8.0, from PyPI);
  `tests/test_torch.py` (skipped without the group): the locked build (2.14, CUDA 12.6), tensors
  sharing the engine's arrays, a linear network's masked choices for every head over 200 calls
  of 8 envs on `full` and `siege` (bounded logits, casts preferred: 133 casts; every action
  accepted), and the GPU's kernels where CUDA finds a device. Licences noted in design §5.
- **Ran** in the sandbox: `uv sync --group train` (about 4 GB), `uv run pytest` 22 passed, the GPU
  test skipped (no device; the build's archs `sm_50`–`sm_90`, Pascal's 6.1 through `sm_60`).
  Left: the run on the user's Windows machine with the GTX 1070 (QUEUE).

## T6b on Windows; the checkpoint of increment 3 (2026-10-19)

- #258 merged. The user ran the binding's tests on their Windows machine: 23 passed, the GPU
  test on the GTX 1070 included (the cu126 build's kernels and cuBLAS run on Pascal). uv was not
  installed there; the user's route kept to Anaconda: a conda env with Python 3.12 and uv from
  conda-forge, `UV_PYTHON` set to the env's `python.exe` and `UV_PYTHON_DOWNLOADS=never`, so
  `python/.venv` sits on conda's Python (`python/README.md`). Never point `uv sync` at a conda
  env itself: it removes what the lock does not list.
- **Checkpoint:** increment 3 done. The user cut increment 4 as T8 before T7 (the agent's
  recommendation: trainability is the epic's largest open risk, and the win rate against Random
  needs no T7 runner); T8a, the design, goes to the user for review before code. T7 and T9 follow.

## T8a — The smoke training's design (2026-10-19)

- **The user, at the session's start:** T8a, then T8b in the same session after the review; an
  own PPO over the factorized heads rather than a library's (a library's MaskablePPO masks each
  dimension on its own and would force decision 5's fallback: casts as sub-steps, a new action
  encoding, a C ABI change).
- **Measured** (design §8.1): Random against Random per kind with both seats external, 1 000
  episodes each: the attacker wins 72.4 % of the hero battles, 60.1 % of the sieges a hero
  defends, 92.3 % of a garrison's; a battle takes about 260 decisions (p99 856); 1.5 % of the
  decisions are casts. The observation's values reach 6 · 10^5 (hence `symlog`). So the agent
  plays both sides, and Random's own rate with sides balanced is about 50 %.
- **Proposed** (design §8, AI-Q5): self-play training, evaluation against the engine's Random,
  the default distribution as the family, seats for the encounter source (C ABI v29), the
  criterion (1 000 held-out episodes per side, greedy, at least 80 % overall) and PPO's settings.
  **The user:** all five as recommended (AI-Q5 answered).

## T8b — The trainer (2026-10-19)

- **C ABI v29:** `h3_batch_add_encounters(batch, weight, kinds, seats, error, error_len)`, `seats`
  null for each encounter's own, else for every episode of the entry (at least one external);
  `Scenario::encounters(sampler, weight, Option<[Seat; 2]>)` (`seats()` returns the override).
  Tests: the batch as single envs with either seat replaced, a neutral kind's defender included,
  both internal refused (`h3-sim`); through the ABI an unknown seat, both random, the caller
  always the defender against a random attacker (`h3-ffi`); the C smoke test adds an entry
  against a random defender; `h3env`'s `ABI_VERSION` 29, `add_encounters(attacker=, defender=)`.
  The batch workloads moved by at most +0.04 %.
- **Built** `python/h3env/train/` (design §8.3, §8.4): `policy.py` (`symlog`, the trunk, the value
  and entry heads, the cast heads conditioned through embeddings, Gumbel-max draws, masked logits
  at the dtype's lowest value), `ppo.py` (`Config`, `Rollout`, `gae` per (env, side), `Staging`,
  `Trainer`, `train`), `evaluate.py` (the held-out play against the engine's Random per side, the
  untrained network and Random's own rates at the start), `__main__.py` (every `Config` field an
  option). 5.9 M parameters at the defaults. `tests/test_train.py`: masks and conditioned heads
  over 60 calls with casts preferred, log-probabilities and values recomputed equal, GAE by hand
  over two envs (a side twice in a row, a decided battle, a truncation, a transition open at the
  end, the last value), the evaluation's fixed and held-out episodes, a short run's files and its
  checkpoint's round trip. 27 Python tests pass.
- **Speed** on the sandbox's CPU (4 cores, 256 envs, the default network): 900 samples per
  second, an update of 32 768 samples in about 36 s. A fresh array per minibatch (145 MB) paid its
  page faults every time (the sandbox's first touch costs 93 ms per 9 MB): `Staging` gathers
  into tensors allocated once (680 → 900 samples/s, the same losses).
- **The sandbox run** (30 minutes, 46 updates, 1.5 M samples; 200 held-out episodes per side,
  ±4.5 points overall at 95 %): against Random, the sampled policy 0.51 (untrained) → 0.60 (10
  minutes) → 0.68 → **0.73** (attacker 0.90, defender 0.56); greedy play 0.35 → 0.745 → 0.71 →
  0.675; Random itself 0.507 on the same episodes. Battles shortened (episodes per rollout 25 →
  310); no training battle truncated. Greedy play repeated itself until the step limit in up to
  17 % of its battles as the attacker (11 % as the defender); the sampled policy never. **The
  user:** the criterion is the sampled policy's (design §8.2). Per kind at the end (sampled):
  attacking 0.94 hero, 0.84 siege with a hero, 0.86 a garrison; defending 0.65, 0.52 and 0.31 (a
  garrison is one week's growth against a hero's army: Random defends it 0.28). 80 % overall
  needs the defender at about 0.6 to 0.65 with the attacker near 0.97.
- **Left:** T8c, the hour on the user's machine (QUEUE).
