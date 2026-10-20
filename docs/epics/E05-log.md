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

## T8c — The first hour on the user's machine; the opponent pool (2026-10-19)

- #260 merged. **The user's run** (GTX 1070, Ryzen 9 3900X, the defaults: 256 envs, 60 minutes):
  936 updates, about 30 M samples (8 500 per second over the hour, its 12 minutes of
  evaluations included), against Random on the 1 000 held-out
  episodes per side (sampled; attacker, defender; greedy):

  | Minutes | Sampled | Attacker | Defender | Greedy |
  |---|---|---|---|---|
  | 0 (untrained) | 0.500 | 0.717 | 0.284 | 0.317 |
  | 11.7 | **0.779** | 0.930 | 0.628 | 0.701 |
  | 23.8 | 0.726 | 0.932 | 0.519 | 0.681 |
  | 35.9 | 0.725 | 0.936 | 0.514 | 0.695 |
  | 48.0 | 0.724 | 0.930 | 0.517 | 0.692 |
  | 60.5 | 0.721 | 0.906 | 0.536 | 0.669 |

  Random itself 0.502. Exit criterion 7 not met (0.721; the peak 0.779). The attacker saturated
  near 0.93; the defender peaked and lost ten points, then stayed flat: naive self-play drift
  (the defender learns against its own, ever stronger attacker, which Random is not). A longer
  run would not help (flat from minute 24). The final evaluation ran twice (one fell due just
  before the budget): fixed.
- **The user** chose an opponent pool over the hero kind alone, a revised criterion or training
  against Random (design §8.2). Built: `Config.pool_share` (0.5), `pool_size` (8),
  `pool_every` (25); `Trainer.assign` draws each new episode's opponent (a frozen snapshot on a
  random side, else self-play), `Trainer.act` routes the frozen side's rows to its network
  (`Policy.act(rows=...)`), the rollout keeps only the learner's decisions (`Rollout.store`'s
  `side`; the learner's transition spans the frozen moves, `SeatBook` unchanged), the rollout's
  last value only where the learner is to move; per rollout `pool`, `pool_episodes`,
  `pool_won`; `best.pt` at the best sampled rate; the final evaluation reuses the last one when
  no update came between. Tests: a subset of rows chooses as the whole batch's same rows; a
  frozen network plays its side and only the learner's decisions are samples; a run's pool and
  `best.pt`; the reused final evaluation. 29 Python tests pass.
- **The sandbox's check** (CPU, 20 minutes, 66 updates, 1.65 M samples, `--pool-every 5`, 200
  held-out episodes per side): sampled 0.51 → 0.67 (10 minutes) → **0.76** (attacker 0.91,
  defender 0.61: hero 0.70, siege with a hero 0.59, a garrison 0.31); the naive run of T8b had
  0.68 at the same point. The pool filled (8), the learner beat its frozen past selves 0.54 →
  0.64 of the pool episodes; updates hold about 24 000 samples (the frozen side's moves do not
  train). Too short to show whether the drift is gone: that is the next hour on the GTX 1070.
- **The second hour** (#264 merged; the pool at its defaults; the user capped it at 40 minutes:
  440 updates; against Random on the 1 000 held-out episodes per side):

  | Minutes | Sampled | Attacker | Defender | Greedy | First hour (sampled) |
  |---|---|---|---|---|---|
  | 0 (untrained) | 0.500 | 0.717 | 0.284 | 0.317 | 0.500 |
  | 11.5 | 0.775 | 0.932 | 0.618 | 0.746 | 0.779 |
  | 23.3 | **0.811** | 0.943 | 0.679 | 0.720 | 0.726 |
  | 35.7 | 0.790 | 0.945 | 0.636 | 0.700 | 0.725 |
  | 42.2 (final) | **0.800** | 0.943 | 0.658 | 0.714 | 0.724 (48 min) |

  The pool held the defender at 0.62–0.68 where naive self-play fell to 0.52. **Exit criterion 7
  met** (the user, 2026-10-19: the final 0.800, the best 0.811 in `best.pt`; ±1.8 points at
  95 %). The pool cost about a third of the updates per minute (440 in 42 minutes against 744
  in 48): the frozen networks' forward passes per call, and longer evaluations (82–140 s) as the
  agent's defence lasts. Increment 4 done; T7 next (increment 5, the user's checkpoint).

## Increment 5's cut and T7 — Baselines (2026-10-19)

- **The cut** (the user, at increment 4's checkpoint): T7 with the bots as internal seats and the
  trained network's row; two bots, Aggressor and Greedy value; the estimate from the engine (a
  preview of one's own attack, counting everything the strike reveals such as the defender's
  Armorer, never the retaliation: AI-Q6; the estimates in the agents' encodings may follow,
  AI-Q7); ratings "what the server already built": its Glicko-2, copied. Separate branches for
  the `shared/` ADR amendment (#271) and the `cross-lane` preview (#274), the user's consent.
- **T7a** (#274): `h3_battle::preview` and `combat::strike_preview` on the real strike's stages
  (BAT-DMG-011; VCMI 1.7.5's `battleEstimateDamage` gives its shape). Tests: real strikes land
  within their previews (melee and shots, 24 seeds), only valid attacks and shots preview, Bless
  fixes it, jousting's charge counts; `h3-battle`'s coverage holds; the full verify passed.
- **T7b** `h3_sim::bots` (`Policy::Aggressor`, `Policy::Greedy`; design §9): the observation, the
  creature data and the preview; no casts; deterministic. Tests: both win at least 70 % against
  Random from either side over 30 encounters, only legal commands, every battle to its end.
- **T7c** `h3 ai eval` and `src/ai/rating.rs` (Glicko-2 from `h3-server`, its two worked tests).
  On the 1 000 held-out episodes per ordered pair (`EVAL_SEED`, the default kinds; 8.8 s):

  | Row against column | Random | Aggressor | Greedy |
  |---|---|---|---|
  | Random | – | 0.200 | 0.172 |
  | Aggressor | 0.800 | – | 0.481 |
  | Greedy | 0.828 | 0.519 | – |

  Value efficiency against Random: Aggressor +0.247, Greedy +0.277; Greedy against Aggressor
  +0.021. Glicko-2: Greedy 1718, Aggressor 1714, Random 1509 (± about 130 at two deviations).
  Greedy's preview buys little over Aggressor's nearest target (AI-Q7's question).
- **T7d** C ABI v30 (`H3_SEAT_AGGRESSOR`, `H3_SEAT_GREEDY`), `h3env`'s seats and opponents,
  `python -m h3env.train.ladder`. The sandbox's 20-minute pool checkpoint (0.76 against Random
  at its training): 0.751 against Random, 0.434 against Aggressor, 0.420 against Greedy on the
  same episodes. Left: the user's `best.pt` (0.811) on the GTX 1070.
- **T7d, the network's row** (#276 merged; the user's machine): a new pool run of 11 minutes
  (145 updates; its last evaluation 0.792 sampled, 0.817 greedy against Random), its `best.pt`
  through `python -m h3env.train.ladder` on the same 1 000 held-out episodes per side:

  | Against | The network | Aggressor | Greedy |
  |---|---|---|---|
  | Random | 0.802 (value +0.247) | 0.800 (+0.247) | 0.828 (+0.277) |
  | Aggressor | 0.498 (−0.010) | – | 0.519 (+0.021) |
  | Greedy | 0.479 (−0.025) | 0.481 (−0.021) | – |

  The network plays at Aggressor's level: even against it, Greedy's margin over both alike.
  Greedy play beat the sampled policy in this run (0.817 against 0.792), unlike the first runs.
  **Exit criterion 6 met**: the bots and the runner exist and are recorded as baselines.
  Increment 5 done; the user's checkpoint next (T9).

## Increment 6's cut, T9a and T9b — The audit and the independent review (2026-10-19, 2026-10-20)

- **The cut** (the user, at increment 5's checkpoint): T9 in four parts, T9a and T9b in the
  sandbox now with four reviewers, T9c (the bench sweep, the verifies with original data) on the
  user's machine and in a sandbox with the user's files staged, T9d the closure; AI-Q7 moves to
  E06.
- **T9a, exit criteria 1 to 7 against the code and tests:** 1 met (`h3-ffi`'s scenario tests: a
  field on the shipped packs, the 12 special battlefields, the siege, the garrison;
  `encounters-v1.toml` and `census-v1.toml` versioned, design §4; `encounter::tests`: 1 500
  samples valid, every 50th played). 2 met (`batch::tests`: `a_batch_is_n_single_envs` over 1 to
  6 envs and 0 to 7 threads, every masked action applies; the head calls; resets, truncations,
  skipped episodes, refusals; `h3-ffi`'s `the_abi_batch_is_the_rust_batch`). 3 met after
  review 2's F2 (below: `h3.h` marked nothing). 4 met (`reward::tests`, T1's examples, exactly
  zero-sum; review 3's F1 and F2 fixed the war machines). 5, 6, 7 met (T6b, T7, T8c). The inbox
  sorted by destination (E06, encounters v2, E23, E13; two items dropped as done); the Python
  README gained the opponent pool and `best.pt`.
- **T9b, four fresh reviewers** (each one slice, read-only, every finding with a failure path;
  21 findings, 18 test gaps, 9 doubts), verified one by one, then fixed:
  - **r1, the batch and its pool** (5, all nits but one): the head calls answered an escape's
    heads that `step` refuses (ADR-0025 point 7; the kept entry mask now drops them, the
    head-call test runs on 0 and 2 threads); the skip loop tried 65 episodes, not 64; a failed
    setup left a stale encounter kind; two doc notes (a reset replays while no scenario was
    added; what a panic leaves). Sound: `Pool::run`'s `unsafe`, no thread-count dependence,
    distinct streams, refusals leave state, the same-step reset's outputs.
  - **r2, the C ABI and the binding** (7): `h3env`'s head calls and `step` passed short arrays
    the engine reads N values from (major; now refused), `h3.h` marked no trusted tooling
    (major, criterion 3; the fork, the hash and the episode seeds, which decide a battle's draws,
    are marked now), the binding's output arrays could be replaced under the engine's pointers
    (read-only now), ctypes wrapped negative settings (range-checked), a panic left the handle
    usable (closed), the seed's "same battle" claim needs both sides human, an ABI test
    overclaimed. Sound: the layouts and the 30 signatures against `h3.h` and `_ffi.py`, the
    error buffers, `catch_unwind` everywhere, the actor-scoped masks.
  - **r3, reward, encounters, census** (5): the reward credited the attacker of every walled
    siege with a hero with the Catapult's 500 (the battle places it, no setup lists it: about a
    third of the default episodes), and charged a defender's Ballista the battle leaves out at a
    fort (BAT-QUIRK-23) as destroyed (major, both): the war machines now come from the outcome's
    records, so the Catapult counts at the start and when destroyed (the user: ADR-0025 point 8
    read literally); tests by hand and on a sampled walled siege played to its end. The table
    refused all-zero shares but not all-zero kind weights (fixed). Recorded, not changed: A3's
    growth is the count's tenth where VCMI 1.7.5 grows a ×1000 power (5 creatures stay 5 over
    seven weeks, VCMI's 9; the fix makes late stacks pass A4's cap: E23, with the neutral
    share), A7's "+0.5 a week" rounded down in the table (encounters v2). Sound: the reward's
    formula and zero-sum, the tables against A1 to A12 (recomputed), the census's sums.
  - **r4, the bots, the evaluation, the trainer** (4): the Python evaluation dropped episodes an
    engine seat ends before the agent acts and then ran to its call bound (major; none among
    the default episodes: 0 of 1 000 per side and opponent, so T7d's and T8c's rates stand; now
    counted as `skipped`, the ladder prints them), the ladder did not check its episodes against
    the checkpoint's training seed (refused now), `h3 ai eval` looped on `--max-steps 0` or a
    refusal (refused, an error), and one Glicko-2 period per game held the deviation near 64 and
    remembered only the last ~40 games (the user: the whole run one period); a test pins the Rust
    and Python `EVAL_SEED`. Sound: no bot can hang the batch (Defend is always listed but for the
    Catapult, which acts only with a wall to shoot), no leak, PPO's masks, log-probabilities,
    GAE and the pool's exclusion, Glicko-2 as the server's.
- **`h3 ai eval` again** (the reward and the ratings fixed; 1 000 held-out episodes per pair,
  8.6 s): the win rates unchanged (Greedy 0.828 and Aggressor 0.800 against Random, Greedy
  0.519 against Aggressor); value against Random Greedy +0.276, Aggressor +0.247; Glicko-2 Greedy
  1680, Aggressor 1646, Random 1174 (± 16 at two deviations).
- **Left for later** (the lane's inbox): an N cap in `h3_batch_create`, `h3env.tools.Env` after a
  panic, the sampler's untested dimensions (encounters v2), the bots' fallback branches without
  a targeted test.

## T9c — The user's machine and the verifies with original data (2026-10-20)

- **The code:** `master` at b2d2e56 (#288 after #286, docs only: the same code, fingerprint
  b4487432). The user's Ryzen 9 3900X (12 cores, 24 threads, 64 GB), Windows 10 Pro 19045,
  rustc 1.97.0, release; QUEUE's commands as one script in the user's repo (`target\t9c\t9c.ps1`,
  its logs read over the device link), the machine otherwise idle.
- **The sweep** (`h3 ai bench --encounters default`, 1 048 576 steps per row, seed 1; thousands
  of steps/s inside `BattleBatch::step` / with the head calls and the bench's random choice):

  | N \ T | 1 | 6 | 12 | 24 |
  |---|---|---|---|---|
  | 64 | 53 / 49 | 177 / 127 | 203 / 121 | 199 / 95 |
  | 256 | 50 / 46 | 228 / 162 | 335 / 194 | 375 / 177 |
  | 1 024 | 50 / 47 | 233 / 172 | 387 / 240 | 497 / 264 |

  One thread gives about 50 000 steps/s whatever N (T3c's sandbox indication on `full`: 46 000 to
  49 000). Inside the step 12 threads give 3.8 to 7.7 × and 24 (SMT) 3.7 to 9.9 ×, more the
  larger the batch: at N = 64 a call (64 steps, 0.3 ms at 12 threads) is too short for more than
  12 workers. The bench's choice (a scan of every mask in the caller's thread) is serial and caps
  the second figure, which falls beyond 12 threads at N = 64 and 256: a learner's policy
  replaces it.
- **Rust against Python** (N = 256, T = 12, 4 096 calls, twice in turn): Rust 336 964 and 333 152
  / 194 542 and 193 298; Python 314 319 and 327 436 / 158 107 and 163 212 (3 771 episodes against
  Rust's 3 773: the choices differ). Inside the step Python is 2 to 7 % behind Rust, with its
  numpy choice 16 to 19 %.
- **The Windows verify** (`cargo xtask verify --with-original-data --summary`, 6 min 40 s): every
  step passed (coverage 288 s, determinism 72 s); `ffi-smoke` and `bench-check` N/A on Windows;
  the original-data step's harvest check passed (the committed packs match the installation), its
  executable-map check skipped (`exe.py` needs Python 3.11: the script ran outside the conda env,
  `CONDA_PREFIX` unset, so `uv` used `python/.venv` as it stands and the system Python ran the
  check).
- **The Linux verify** (the cloud sandbox, 2 vCPUs; the user's executable, `H3bitmap.lod`,
  `H3ab_bmp.lod`, `Objects.txt`, `maps.tar` and `extracted.tar` staged from the device link):
  LINUX_VERIFY_RESULT
- Exit criteria 8 (the sweep recorded; the single-env gates in the Linux run's `bench-check`) and
  9 met.

## T9d — The closure (2026-10-20)

- **Shared first** (`shared/e05-closure`): BACKLOG (E05 done), ADR-0025's working-detail line,
  `docs/07-simulation-and-ai.md` §1.1 (the environment as built: the batch, seats, episode
  sources, reward v1, the information boundary, Python, the smoke training, the baselines, the
  throughput) and its baselines and deferred items, the global HANDOFF's verify line (the
  script's LLVM variables, the lane's note since T8b). Docs only: the light gates.
- **The design note at closure** (below): every section had stayed after its task built it, as
  the record of the build's decisions, and the code and the Python package cited it by section.
  Its text is kept below with its numbering, the note deleted, and the references (26 in
  `h3-sim`, `h3-ffi`, `h3-cli`'s `src/ai/`, the two data files' headers, `python/`, the epic
  file and OPEN-QUESTIONS) repointed to `E05-log.md` §n; comments only, so the branch's code
  fingerprint changed and its full verify ran again (LANE_VERIFY_RESULT). E23's epic file (the
  battle lane's) cites the note too: the battle queue's inbox says where to.
- **Exit criteria:** 1 to 7 (T9a, after T9b's fixes) and 8 and 9 (T9c) met.
- **What later epics inherit:** the AI queue's inbox by destination (E06, encounters v2, E23,
  E13), AI-Q7 (E06), the neutral kinds weighed 0 until E23 (A12).

## The design note at closure

`docs/epics/E05-design.md` was deleted when E05 closed (T9d, the ADR index's rule). Its sections had
stayed after their tasks built them, as the record of the build's decisions; its text at the closure
follows, its numbering kept: "`E05-log.md` §n" in the code and the Python package, and "design §n"
in the epic file, earlier entries of this log and OPEN-QUESTIONS, point here. The lasting parts are
in `docs/07-simulation-and-ai.md` §1.1 (the environment as built); the decisions are ADR-0025's.

### 1. The batch API (a) — built (T3b, T3c, T3d)

ADR-0025 points 1 to 7 as code: `h3_sim::batch` (`BattleBatch`, `Scenario`, `Seat`, `Policy`,
`EnvReport`, `Outputs`, `episode_seed`, `draw_scenario`; the threads in `batch::pool`) and C ABI
v27 in `h3-ffi`'s `src/batch.rs`, declared in `include/h3.h` (T1's sketch with its names, plus
`H3Seats`, `H3_SEAT_*`, `H3_CURRENCY_*` and `H3_ERR_NOT_READY`; the encounter source came with
T5b2 as `h3_batch_add_encounters`, C ABI v28, §4.1). Decisions of the build beyond the ADR (the
agent's, provisional):

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
- **Head calls (2026-10-19):** each env keeps the entry mask of the last call (the single env's
  without the escapes: E05 T9b). A non-cast entry's later heads ("0 only" while it is legal) are
  answered from it; only casts ask the env. The answers equal the single env's but for the
  escapes, refused in every head (a test over every legal entry, the escapes, illegal ones and
  non-zero earlier heads); the three calls cost 0.11–0.15 ms for 64 envs on `full` instead of 1.4 ms.
- **Scenarios (T3d):** parsed and compiled in `h3-ffi` (`src/scenario.rs`, as T2's); one ruleset
  per packs and profile per batch, reused through the first scenario that compiled it
  (`Scenario::ruleset`), so `h3-ffi` needs no new dependency.
- **Errors:** a refused call changes nothing: `H3_ERR_NULL`, `H3_ERR_ARGUMENT` (with a message for
  create and add) or `H3_ERR_NOT_READY` (no scenario yet, not reset); a step returns
  `H3_ERR_REJECTED` when an env refused its action, its `status` saying why.
- **Properties:** a batch steps as N single envs for random N, T (0 to 7), scenarios, seats, step
  limits, currencies and prices (`h3-sim`'s `batch/tests.rs`); the ABI batch equals the Rust
  batch output by output (`h3-ffi`'s `batch/tests.rs`).

### 2. Seats (b)

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

### 3. Reward v1 (c) — built (T4)

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

### 4. Scenarios and the encounter distribution (d)

#### 4.1 What the ABI loads (T2)

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
- **Episode sources of a batch:** a weighted list of entries, each reset drawing one by weight:
  (i) a fixed scenario (`h3_batch_add_scenario`: text, integer weight, seats), or (ii) the
  encounter distribution (**built, T5b2**): `Scenario::encounters(sampler, weight)` in Rust,
  `h3_batch_add_encounters(batch, weight, kinds, error, error_len)` in C ABI v28 (`kinds` null for
  the distribution's weights, else six in `H3_ENCOUNTER_*` order: a curriculum; on the original
  packs, `base:faithful`, sharing their ruleset with the batch's scenarios on them; the adventure
  part compiled per call). `Scenario::episode(seed, max_steps)` gives an episode's env, seats and
  kind: the sampler's encounter of the episode's seed, drawn from stream `ENCOUNTER` (a kernel
  stream, #254) so that it is independent of the entry's draw from `SETUP`; the neutral kinds'
  defender is played inside the call. Each env reports its episode's kind (`EnvReport.encounter`,
  `H3BatchBuffers.encounter`: `H3_ENCOUNTER_*`, -1 for a fixed scenario). Cost in a release build
  on the cloud sandbox: a sample 50 µs at the default weights (26 µs at A1's shares), an
  episode's start with its env 160 µs, about 2 % of an episode of some 300 steps.

#### 4.2 The encounter distribution v1 (built, T5b1)

- **Files:** `crates/h3-sim/data/encounters-v1.toml` (the assumptions of §4.4, each table with
  its `why`) names the census it reads (`census-v1.toml`, §4.3); both are compiled into `h3-sim`
  (`h3_sim::encounter::{ENCOUNTERS_V1, CENSUS_V1}`) and checked by `EncounterTable::parse` (each
  table's length, non-decreasing quantiles, drawable weights, no unknown field). Integers only
  (the sim layer has no float arithmetic, ADR-0003): shares in ‰, a continuous distribution as a
  quantile table of 21 points (p = 0.001, 0.05, …, 0.95, 0.999; a draw takes one of the 20 bins
  and a point in it evenly, linear between), computed from the stated normal or log-normal when
  the file was written.
- **`Sampler`** (`h3_sim::encounter`): built on the battle and adventure parts of one core
  (`GameRuleset`'s check); `sample(seed)` draws every choice from stream `ENCOUNTER` of the seed and
  returns an `Encounter`: kind, week, a `BattleSetup` (obstacles left to the episode,
  `random_obstacles`; both sides human), the seats (A12) and the `Facts` drawn (place, level,
  count, redraws, ratio, k, factions, hero levels, values, fort). Its default weights are the
  file's `weight` (the neutral kinds 0); `with_shares` draws at A1's shares, `with_weights` by a
  curriculum's.
- **The engine's rules it uses**, never copies: ADV-MON-003 (`AdventureRuleset::random_monsters`),
  ADV-MON-004 (the adventure-map range), ADV-MON-009 (`h3_adventure::split`), and a hero's battle
  values as GAM-BAT-001 derives them (`derive_battle_hero`, `max_mana`, `war_machines`); the
  Catapult is the battle's own (BAT-SIE-009). **Approximations**, labelled in the code: a hero's
  level-ups (the class's `primary_gain` and `skill_weights`), spells and artifacts; an army one
  stack per built level, its value split by growth value, what a level's share cannot buy carried
  to the level below; guards a stack of A3 divided evenly.
- **The check** (`encounter::tests`): 30 000 encounters at A1's shares against their weights:
  kinds, weeks per kind, the neutral levels from week 7 (no stack is drawn again there) and the
  ramp's bounds, places, the bins of the ratio, k and the attacker ratio, factions, hero levels by
  week, forts. 1 500 samples are valid battles (`Scenario::new`), every 50th plays out; every
  sampled hero passes `h3_heroes::validate`.
- **What it shows** (20 000 samples): in week 1 A4's cap draws 16 % of the neutral stacks again
  (level 4 falls from 10 % to 4 % of them), in week 2 1 %, from week 4 none; under random play the
  attacker wins 98 % of the neutral battles, 80 % of the hero battles, 63 % of the sieges a hero
  defends.
- Not drawn yet: the difficulty (A11: Normal), two boats (A10), creature banks (A1). T5b2 made the
  sampler a batch's episode source (§4.1).

#### 4.3 The census v1 (T5a, built)

- **`h3 ai census <maps folder or files> [--out <file>]`** (`h3-cli`'s `src/ai/census.rs`) reads
  each `.h3m` through `h3-map` (the raw model and the static layer's `import_map`, which every
  shipped map passes) into a `GameState` of one player on the original packs (`base:faithful`)
  and adds up aggregate numbers only (ADR-0009): the maps by version, size and levels; the tiles
  by passage (land, water, impassable; underground, coast, *open*: no object blocks it or is
  visited from it); the monsters by level (fixed or drawn creature) and how many fix their count;
  guarded piles, special grounds, placed and random heroes, prisons, garrisons (anti-magic apart:
  subclass 1), creature banks, the four banks with their own class, boats; the towns by faction
  (a random town apart) and starting fort (the standard set's flag, or the custom set's fort,
  citadel or castle bit); and **every battle place** with its open land tiles, monsters and
  guarded piles. A place is the engine's own `AdventureState::battle_ground` of the defender's
  tile (the monster's or the pile's visited tile; GAM-BAT-001, ADV-GRD-001): underground the
  subterranean field, a coast tile the sand and the shore, a special ground on top; the census
  and the adventure map cannot disagree. Every placement and tile counts once, so a larger map
  weighs more (it holds more battles). A map that does not read or import is skipped and named
  on standard error.
- **`crates/h3-sim/data/census-v1.toml`:** the 159 maps of the user's installation
  (`original-data/install/Maps`, 2026-10-18; the tutorial left out, not an `.h3m`). The earlier
  count of 164 (E08's survey) included maps that are not the game's (`experiment`); 160 in the
  epic counted the tutorial. 25 s in a debug build.
- **What it says** (derived; T5b reads the file, these are for the review): 48 RoE, 55 AB, 56 SoD
  maps; 31 S, 71 M, 38 L, 19 XL; 103 with an underground. 13 863 monsters (87 a map): level 1
  17.7 %, 2 26.1 %, 3 25.5 %, 4 16.0 %, 5 8.3 %, 6 3.5 %, 7 2.4 %, any 0.5 %; 1 483 name their
  creature, 740 their count. Their places: grass 24.0 %, dirt 18.8 %, subterranean 17.0 %, sand
  11.5 % (the shore included), rough 9.4 %, snow 7.6 %, swamp 5.8 %, lava 5.4 %, water 0.4 % (55
  monsters a hero meets from a boat); the plain field 88.2 %, the shore 7.5 %, a special ground
  4.3 % (of open land 2.8 %: map makers put monsters on grounds). 335 457 open land tiles, their
  places within a point of the monsters'. 1 465 towns (9 a map; 875 random; Castle the most
  named, 118, Necropolis the fewest, 32); at the start 20.9 % without a fort, 74.3 % a fort,
  1.8 % a citadel, 3.1 % a castle. 36 guarded piles, 238 garrisons (45 anti-magic), 2 137
  creature banks (13 a map), 371 heroes placed, random or in prisons, 118 boats, 1 857 special
  grounds.
- **Twins:** 35 maps share their terrain with an earlier one (the allied versions). Leaving the
  twins out moves no terrain or level share by more than 1.6 points, so every shipped map counts
  once (proposed, A9 below).

#### 4.4 The assumptions v1 (decided 2026-10-19, AI-Q4)

What the maps cannot say, for the encounter distribution's `why` fields (`encounters-v1.toml`
holds every number, written from the census and these in T5b1). Every number is provisional; the
adventure's logged battles replace them (decision 7). Proposed at T5a; **the user (2026-10-19):**
A3 keeps in mind that the maps' stacks differ by level from the start (a horde of Pikemen in
week 1, a few Black Dragons) and that no hero faces level 7 early; A4's ratio is a distribution
around 1.8 with a standard deviation of 0.5, at least 0.5, for occasional tough battles; A5 as the
review changed it; A12's neutral kinds fully built but weighted 0 until E23 (after E05); the rest
as proposed. How T5b1 pins down what a row leaves open is marked *(T5b1)*.

| # | Dimension | Decided | Why |
|---|---|---|---|
| A1 | Encounter kind (of battles fought) | wandering 0.78, guards 0.02, hero 0.10, sieges 0.10: a hero defends 0.05, an owned town's garrison 0.025, a neutral town's 0.025 *(T5b1: A8's split as kinds, so A12 can weigh the neutral one 0)* | 87 monsters a map against 9 towns and a few heroes: a player clears most monsters on the way; each neutral or enemy town taken is a siege. Guards are the 274 piles and garrisons against 13 863 monsters (2 %). Creature banks (13 a map) wait for their battle layouts (the adventure lane's queue) |
| A2 | Game week (1 to 16), per kind | wandering and guards ∝ 0.8^(w−1); hero rising to week 4, flat to 8, falling to 16 (1/9 of the peak); sieges ∝ 0.9^(w−1), a neutral town ∝ 0.7^(w−1) *(T5b1: tabulated in ‰)* | neutrals fall early (median week 3), enemy heroes meet mid-game, towns are taken throughout, neutral ones early |
| A3 | A neutral stack | level: the census's (fixed and random; "any" spread evenly) × a ramp by week: 1–3 always, 4 half in week 1, 5 half in week 2, 6 and 7 from weeks 3 and 4 in quarters (full from weeks 6 and 7); the creature drawn among the level's (ADV-MON-003), the count from its adventure range (ADV-MON-004), grown each week by a tenth (integer) while below 4 000 (VCMI 1.7.5 `CGCreature::newTurn`: `weeklyGrowthPercent` 10, `weeklyGrowthCap` 4 000; *T9b: VCMI grows the stack's power, a thousand per creature, so that 5 creatures are 9 after seven weeks where the count's tenth leaves them 5; the neutral kinds weigh 0 until E23, which decides*); split by the original's rule against the hero's army (`h3_adventure::split`, ADV-MON-009) | the map's own levels and the game's own draws; the adventure ranges already make low levels hordes and high ones few; strong stacks are fought later; growth makes late neutrals larger |
| A4 | The army facing a neutral | value = the stack's × r, r normal (mean 1.8, standard deviation 0.5), at least 0.5; a stack whose army would exceed 2 × A5's value of the week is drawn again (r kept) | players attack what they beat, now and then a tough fight; the split's own table expects ratios 0.5 to 2 |
| A5 | A hero's army (hero and siege kinds) | value = k × the sum over the weeks u ≤ w of the faction's growth value (growth × `ai_value`) of the levels A6 makes available in week u; k log-normal (median 1.0, ×/÷ 1.6); the attacker's over the defender's log-normal (median 1.3, ×/÷ 1.5). Castle's median: 6 500 in week 1, 43 000 in week 4, 111 000 in week 8, 246 000 in week 16 | one main army carries a kingdom's production (a castle's double growth and more towns against its losses); consistent with A6, where G × w × 0.7 gave week 1 1.8 weeks of levels 1–4 |
| A6 | An army's creatures | the hero's faction; levels 1–4 from week 1, 5 from week 2, 6 from week 3, 7 from week 4; value split by each level's growth value; one stack a level, upgraded with the share (w − 1) / 6, at most 1 | a usual build order |
| A7 | A hero | faction by the census's towns (named; random ones spread evenly over the nine factions *(T5b1: else no Conflux)*), might or magic class 1 : 1, the hero type even among the class's; level by week 4, 7, 9, 11 (weeks 1–4), then +1 a week to 15 (week 8), then +0.5 a week, ± 2. *(T5b1: `h3-heroes` has no level-up rule; an approximation, not a rule:)* each level-up adds a primary skill by the class's chances (`primary_gain`, levels 2–9 and 10+) and raises a known skill or learns one by the class's `skill_weights` (at most 8, Expert at most); a spell book; per spell level up to the guild (levels 1, 2, 3, 3, 4, 4, then 5 from week 7; at most the faction's: 4 Castle, 3 Stronghold and Fortress, else 5, VCMI 1.7.5 `mageGuild`) and its learn level, 6 − level spells (VCMI `spellsAtLevel`, no Library) drawn among the level's hero spells; level / 2 artifacts drawn by class by level (treasure from 1, minor from 6, major from 11, a relic from 16; no combinations, scrolls or war machines), worn where a slot is free; Ballista, Ammo Cart and First Aid Tent each at 0.5; full mana | a human main hero's usual pace |
| A8 | A siege | the faction by the census's towns (as A7); the fort from the census in week 1, then citadel and castle more likely each week (castle at most 0.7; a neutral town keeps the census's); a hero defends the kind of A1, else the garrison: one week's growth (A6's levels) *(T5b1: the moat damaging from a Citadel (SoD rules), structures 6 + 2 (w − 1) at most 30 (a neutral town 6), morale +1 at an owned town (a Tavern), no Escape Tunnel; the terrain the faction's native one)* | neutral towns early, owned towns later |
| A9 | Twins | every shipped map once | §4.3 |
| A10 | Places | neutral stacks and guards: the census's monster places (the 55 on water: the deck, the hero in a boat); hero battles: open land; sieges: the town's own field; two boats left out | where battles stand on the maps |
| A11 | Difficulty | left out (normal) | the batch plays both sides as human players' (ADR-0025 point 6); the difficulty shapes only the computer's automatic choices (ADR-0023 point 7) |
| A12 | Seats | neutral kinds (wandering, guards, a neutral town) the system actor's (internal), the others external on both sides; the neutral kinds built and tested at A1's shares, weighted 0 in what the batch samples until E23's policy (after E05) | random neutrals teach the wrong lessons (the epic's risk) |

### 5. The Python side (e) — built on Linux (T6a), PyTorch locked (T6b)

`python/` (package `h3env`, `python/README.md`): ctypes over C ABI v28 (no compile step, the same
on Linux and Windows; `ctypes.CDLL` releases the GIL in each call), numpy arrays allocated once
and written in place by the engine (zero copy). `_ffi.py` (the library from `$H3_LIBRARY`, else
`../target/release/`; every signature; a library of another `h3_abi_version` is refused),
`batch.py` (`BattleBatch`), `seats.py` (`SeatBook`), `tools.py` (single envs, fork, hash: §6),
`bench.py`, `tests/` (pytest; a lane command, not a gate). A uv-managed CPython 3.12 in
`python/.venv` (`.python-version`); `uv.lock` for Linux and Windows on x86-64 (`tool.uv
.environments`), numpy and pytest, and torch in the group `train`. Decisions of the build (the agent's, provisional):

- **The group `train`** (T6b, 2026-10-19): `torch==2.14.*` from PyTorch's index
  `https://download.pytorch.org/whl/cu126` only (`tool.uv.sources`, an `explicit` index), its
  dependencies from PyPI; locked as 2.14.1+cu126 for Python 3.10 to 3.15 on both platforms, with
  the CUDA 12.6 runtime (`cuda-toolkit` 12.6.3, `nvidia-*-cu12`) on Linux (the Windows wheel
  carries its own). Opt-in (`uv sync --group train`, about 4 GB on Linux): the default groups
  stay `dev`, and `tests/test_torch.py` skips without torch (the locked build, tensors over the
  engine's arrays, a network's masked choices for every head, the GPU's kernels where there is
  one). Licences: torch is BSD-3; NVIDIA's CUDA libraries are proprietary, fetched by uv on the
  machine that trains, never distributed with or linked into the engine (ADR-0009 point 3 covers
  the engine's dependencies).
- **`BattleBatch.step`** takes the heads as arrays (or reads `actions`, an `H3Action` record
  array); with `check` (default) it raises when an env with a side to move refused its action,
  while an env that gave up refuses silently (`status`).
- **`SeatBook`:** a side's transition opens when it acts and closes at its next decision or at
  the episode's end (both sides' then), with the sum of its rewards in between; a side that never
  acted has none. A Gymnasium-style wrapper stays out until a library needs one (T8).
- **The random choice** (`sample`, `random_actions`, as `h3 ai bench`): one draw per row over the
  legal indices' flat positions (82 µs for 64 × 1 404 entries; a cumulative sum cost 658 µs).

### 6. The information boundary (f)

- The user (2026-10-18): every per-player observation coming out of a battle is already exactly
  the information a human player receives. No check against the original's screens is needed;
  E05 relies on the projection (`h3_battle::observe`, ADR-0008, `docs/06-security.md`).
- The batch API serves only the acting seat's `observe` (and, at a truncation, both sides'), never
  the state, its hash or a fork. Full-state access stays in the single-env functions
  (`h3_env_state_hash`, `h3_env_fork`, `BattleEnv::state`), marked trusted tooling in `h3.h` and
  in `h3env.tools` (exit criterion 3). The batch's episode seeds (an output for bookkeeping and
  replays, ADR-0025 point 1) decide the battle's draws: marked trusted tooling too, never an
  agent's input (the trainer reads them only to keep its held-out episodes apart; E05 T9b).
- The flat observation is the visible state now; a human also sees the battle's log. A fact that
  only history tells and that matters for play belongs in the observation (BAT-Q35 is one such
  corner, the battle lane's).

### 7. Benchmarks (g)

- **Gated** (instruction counts, `benches/ai/`, `lane_only = true`: the battle lane's changes move
  them): `batch-random-full` — a batch of 64 envs of `packs/synthetic/scenarios/full.toml`, one
  thread, 1 000 calls of random legal actions chosen from the batch's own masks (casts through the
  heads), observations and masks written, counted inside `h3_sim::batch::BattleBatch::step`;
  `batch-random-siege` the same on the siege; `batch-random-encounters-v1` (T5b2) the same on the
  encounter distribution at its default weights (`h3 ai bench --encounters default`): 11.86 G
  instructions against 11.45 G for `full`, the automatic resets' sampling included (on the
  sandbox about 39 700 steps/s inside `step` against 50 700 for `full`, a run beside a verify).
- **Recorded** (wall time, no gate): `h3 ai bench --envs N --threads T --calls K` prints steps/s
  inside `BattleBatch::step` and with the head calls and the random choice; on the user's Ryzen 9
  3900X for T = 1, 6, 12, 24 and N = 64, 256, 1 024 at T9 (moved from T3 by the user, 2026-10-18;
  the sandbox's indication in the log, T3c), and the Python binding's steps/s against Rust's
  (T6), in the epic log.
- **Built (T3c):** both gated workloads (`benches/ai/`, lane-only, threads 0 so that every count
  is in the caller's thread; collect `*BattleBatch>::step`, never inlined).
- The single-env gates (the battle lane's `random-rollouts-*`) stay as they are.

### 8. Smoke training (T8) — built (T8a–T8c, 2026-10-19; AI-Q5); exit criterion 7 met

Exit criterion 7: a smoke run, within an hour on the user's machine, wins at least 80 % against
Random on held-out seeds of its encounter family. Decided at T8a's start (the user, 2026-10-19):
an own compact PPO, not a library (§8.3); at the review (AI-Q5) the rest as proposed here:
self-play, the default distribution, C ABI v29's seats, the criterion and the settings.

#### 8.1 Random against Random (measured, T8a)

1 000 episodes per kind (sandbox, batch seed 7, 64 envs, a 3 000-step limit that no episode
reached), both seats external playing `random_actions`, which chooses as the internal `Random`
does (`rollout::choose`: a uniform entry, then uniform heads):

| Kind | Attacker wins | Defender wins | Decisions, median (attacker, defender) | Both sides, p99 | Casts |
|---|---|---|---|---|---|
| hero | 72.4 % | 27.4 % | 148, 108 | 856 | 1.5 % |
| siege, a hero defends | 60.1 % | 39.8 % | 134, 110 | 650 | 1.7 % |
| siege, a garrison | 92.3 % | 7.7 % | 88, 42 | 465 | 1.3 % |

So the agent must play both sides: as the attacker alone Random already wins most of these
battles. Over the family below (§8.2), the agent in each seat half of the time, Random's own rate
against Random is about 50 %. The observation's values reach 6 · 10^5 (counts, hit points,
values): 1 731 of its 8 882 features exceed 1 somewhere, 293 exceed 100. Python played 16 000
steps/s on the sandbox's 4 cores (`threads = 4`).

#### 8.2 The family, the opponent, the held-out seeds

- **The family:** the encounter distribution v1 at its default weights (`add_encounters()`):
  the three kinds with a player on both sides, hero 4/7, a siege a hero defends 2/7, a siege
  of a garrison 1/7 (the neutral kinds weigh 0 until E23, A12). Every faction, week, hero and
  army the sampler draws; sieges included. Fallback, if an hour does not suffice: the hero kind
  alone (`kinds = [0, 0, 1, 0, 0, 0]`), recorded as such.
- **Training is self-play** (decision 8: self-play and the system actor only; Random is for tests
  and measurement, decision 4): one network plays both sides of each battle, both external, and
  both sides' decisions are its samples. **With an opponent pool** (the user, 2026-10-19, after
  T8c's first run: naive self-play's defender regressed against Random, 0.63 → 0.52, while the
  attacker held at 0.93): in `pool_share` (0.5) of the episodes a frozen earlier network plays
  one side, drawn per episode from `pool_size` (8) snapshots taken every `pool_every` (25)
  updates (when full, a new one replaces a random one, so the ages stay spread; the untrained
  network is never one); only the learner's decisions train. A league and its ratings stay
  E06's.
- **Evaluation against the internal Random:** two evaluation batches, the agent attacking in one
  (seats External, Random) and defending in the other (Random, External), so every reported
  decision is the agent's. It needs **seats for the encounter source** (C ABI v29, built in
  T8b; `BattleBatch.add_encounters(attacker=, defender=)` in Python):
  `h3_batch_add_encounters` gains `const H3Seats *seats` (null: the encounter's own seats, as
  now; else these for every episode of the entry, at least one external), and
  `Scenario::encounters` an `Option<[Seat; 2]>`. ADR-0025's point 5 is kept (seats per entry, not
  per batch). The alternative, Random played from Python in the external rows, needs no ABI
  change but is a second Random beside T7's `h3 ai eval`'s.
- **Held-out seeds:** the evaluation batches have their own batch seeds (fixed constants), the
  training batch the run's seed; every evaluation plays the same episodes: the first
  `ceil(E / N)` episodes of each env, the episode set independent of how long battles last. The
  run checks that no evaluation episode seed (`episode_seed`) is a training one (a set of the
  training episodes' seeds; disjoint unless two 64-bit hashes meet). E = 1 000 per side (the 95 %
  interval at 80 % is ±2.5 points per side, ±1.8 overall).
- **The criterion:** the final evaluation's overall win rate (`best.pt` keeps the best
  evaluation's network, reported beside it), both sides weighed equally, at least 80 %, of the
  **sampled** policy (its draws from a generator seeded alike for every evaluation). Decided at
  T8b (the user, 2026-10-19), replacing AI-Q5's greedy choice: in T8b's sandbox run greedy play
  (the most likely entry, then the most likely heads) repeated itself against Random until the
  step limit in up to 17 % of its battles, while the sampled policy never did. A draw or a
  truncation is no win. Reported per side and kind beside Random's own rates and the untrained
  network's, with the mean reward v1 and the truncations; greedy play's rate too.

#### 8.3 The trainer: an own PPO over the factorized heads

- **Why own:** a library's PPO (Stable-Baselines3's MaskablePPO) masks each action dimension on
  its own, so a cast's conditioned heads would need decision 5's fallback (casts as sub-steps, a
  new action encoding, the partial choice in the observation, a C ABI change) and a Gym wrapper
  over a batch that already steps vectors. An own PPO (CleanRL's single-file pattern) uses the
  head calls as they are and adds no dependency beyond torch.
- **The action:** the entry from the entry mask; only for the rows that chose a cast (about 1.5 %
  of the decisions), the spell from `spell_mask(entries)`, the target from `target_mask`, the
  second from `second_mask`, one forward pass per head, each conditioned on the earlier choices.
  A row's log-probability is the sum of its chosen heads' (a non-cast row's later heads are "0
  only", log-probability 0); its entropy the entry's plus the cast's heads along the chosen path.
  Masked logits are set to the dtype's lowest value, never `-inf` (no NaN when a row has one
  legal choice).
- **The network** over the flat observation v21 (8 882 values): `symlog(x) = sign(x) ·
  ln(1 + |x|)` on the input (stateless: no running statistics to keep in a checkpoint, the same
  for every kind), a shared trunk of two 512-unit layers (ReLU), the value head (one unit) and the
  entry head (1 404 logits) on it; the spell head (128) takes the trunk and which cast was
  chosen, the target head (298) also the spell (an embedding of 32), the second head (298) also
  the target (an embedding of 32). About 6 M parameters, three quarters of them in the first layer.
- **Self-play's transitions** (`SeatBook`, §5): one trajectory per (env, side). Each call's
  sample is the acting side's (observation, masks, chosen heads, log-probability, value); a
  sample's reward is what `SeatBook` closes for it (reward v1, terminal), and its next value the
  value of the same side's next sample, 0 after a decided battle, the value of `final_obs[env,
  side]` after a truncation. GAE runs backwards over the rollout's calls with a carry per (env,
  side). A transition still open when the rollout ends has no next value yet: it is left out of
  that update (at most two per env); the one before it bootstraps from its value.
- **Settings** (provisional, in the run's config and its report): 256 envs, `threads` = the
  machine's cores, rollouts of 128 calls (32 768 samples), γ 0.995, λ 0.95, Adam at 2.5 · 10^-4
  (linear decay), 4 epochs of 8 minibatches, clip 0.2, value coefficient 0.5, entropy 0.01, the
  gradient's norm at most 0.5, advantages normalized per minibatch; reward v1 at its defaults
  (`ai_value`, mana price 10); a 2 000-step limit per battle. The rollout's observations stay on
  the CPU (32 768 × 8 882 float32, 1.2 GB) and go to the device per minibatch.
- **Budget:** wall time is the limit (`--minutes 60`), evaluations every 10 minutes and at the
  end, and one at the start (the untrained network, and Random's own rates on the same episodes:
  the baselines). Measured in T8b on the sandbox's CPU (4 cores, the default network): 900 samples
  per second, an update of 32 768 samples in about 36 s (the rollout about a third). The
  minibatches are gathered into tensors allocated once (`ppo.Staging`): a fresh 145 MB array per
  minibatch paid its page faults every time (the sandbox's first touch costs 93 ms per 9 MB).

#### 8.4 Where the code lives (built, T8b)

- `python/h3env/train/` (needs the group `train`; `import h3env` stays free of torch):
  `policy.py` (the network, the masked heads, sampling and log-probabilities), `ppo.py` (the
  rollout over a `BattleBatch` with `SeatBook`, GAE, the update), `evaluate.py` (the held-out
  evaluation against Random, §8.2), `__main__.py` (`uv run python -m h3env.train --minutes 60
  --out runs/<name>`).
- **A run's output** in `python/runs/<name>/` (git-ignored): `config.json` (every setting, the
  ABI version, torch's, the device, the commit), `metrics.jsonl` (per update: steps,
  episodes, steps/s, the losses, entropy, approximate KL, clip fraction, explained variance;
  per evaluation: the rates of §8.2), `checkpoint.pt` (the network and the optimizer at each
  evaluation), `best.pt` (at the best sampled rate). The results go into `E05-log.md` by hand.
- **Tests** (`tests/test_train.py`, skipped without torch, a lane command): sampling never picks
  a masked entry or head, a cast's heads conditioned row by row; the log-probability of a chosen
  action recomputed equal; GAE on a hand-computed two-sided trajectory (a side acting twice in a
  row, a truncation, an open transition left out); two updates on a small batch with finite
  losses and no refused action; the evaluation's episode set fixed and disjoint from training's;
  a checkpoint's round trip.
- **T8b's sandbox run** (CPU, 4 cores): a short run whose evaluation rate rises clearly above the
  untrained network's on the same held-out episodes, its numbers in the log; T8c is the hour on
  the user's machine with the GTX 1070.
- **Determinism:** the env is deterministic; torch on a GPU is not bit for bit. Seeds are set and
  recorded; a run is reproduced in its numbers, not bitwise. Nothing here is a gate.

### 9. Baselines (T7) — built 2026-10-19 (AI-Q6)

Increment 5, cut at the user's checkpoint (2026-10-19): the bots as internal seats, the runner,
the trained network's row. The user's decisions: two bots (Aggressor, Greedy value); their
damage estimate is the engine's preview of their own attack (AI-Q6, ADR-0025 point 5 amended),
never of the retaliation; the ratings are the server's Glicko-2, copied.

- **The preview** (`h3_battle::preview`, BAT-DMG-011; the AI lane's `cross-lane` change, #274):
  for an attack or shot that validates, the first strike's least and greatest damage and kills,
  every stage of the real strike, none of the rolls. Leaves out the retaliation, a second strike,
  secondary targets and reflection (the user: the original shows none of them); the agents'
  encodings may carry it later (AI-Q7).
- **The bots** (`h3_sim::bots`, `Policy::Aggressor`, `Policy::Greedy`): read their side's
  observation, the creature data and the preview; never cast; deterministic (ties to the command
  listed first). Aggressor: shoot the nearest enemy it can shoot, else attack the nearest it can
  reach, else walk toward the nearest. Greedy: the shot or attack with the greatest previewed
  kills × `ai_value`, less its own estimate of the retaliation (the survivors' mean damage scaled
  by attack against defense in per mille, as BAT-DMG-002's first stage: integers only), else walk
  toward the most valuable enemy (count × `ai_value`).
- **Seats** (C ABI v30): `H3_SEAT_AGGRESSOR` (2), `H3_SEAT_GREEDY` (3); Python's `"aggressor"`,
  `"greedy"` for `add_scenario` and `add_encounters`.
- **`h3 ai eval`** (`h3-cli`'s `src/ai/eval.rs`): every ordered pair of policies on the same
  held-out encounters, the episode seeds of the Python evaluation (`EVAL_SEED`, 100 envs × 10);
  per pair the win rate (both sides weighed equally, a draw or truncation no win) and value
  efficiency (the mean reward v1); Glicko-2 ratings (`src/ai/rating.rs`, copied from
  `h3-server`), the whole run one rating period against the defaults (E05 T9b, the user: one
  period per game held the deviation near 64 and remembered only the last ~40 games).
- **The network's row:** `python -m h3env.train.ladder --checkpoint <run>/best.pt` plays the
  sampled policy against `random`, `aggressor` and `greedy` on the same episodes.
