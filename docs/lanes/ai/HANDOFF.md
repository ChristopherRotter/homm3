# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20** (T1 to T9 in six increments, each with the user's checkpoint):
  the epic file, its record and the kept design note in `E05-log.md` (§1 to §9, cited as
  `E05-log.md` §n), the environment as built in `docs/07-simulation-and-ai.md` §1.1. Criteria
  6 to 9 on the user's machine: the bots and `h3 ai eval`, the smoke run's 0.800 against Random,
  the sweep (about 50 000 steps/s a thread, 497 000 on 24), the verifies with original data on
  Linux and Windows.
- **E06 refined 2026-10-20** (AI-Q8; `docs/epics/E06-battle-ai.md`): it starts now beside E23
  with the kinds a player plays on both sides; the neutral share (T7) waits for E23's policy of a
  side without a hero (its T2 to T4). PPO and a league first, search last; runs of a few hours,
  interruptible and resumable. **Increment 1 done:** T1, T2a (#310; ADR-0026 in #306: the design
  note `docs/epics/E06-design.md`, the run's config and record), T2b1 (#315: the full checkpoint,
  Ctrl+C, `--resume`), T2c (#319: C ABI v31). **Increment 2 done:** T3a (#324: the held-out sets
  v1, the bots' table), T3b (the SPRT gate, promotions, ladders, `--init-from`: the pull request
  of `claude/pensive-maxwell-16n287`). **Next:** the user's hour with gates
  (`configs/e06-t3b.toml`), then increment 3 (T4, the league) at the user's checkpoint.
- **Versions:** action encoding v13, observation encoding v21, replay format v29 (taken over from the battle lane), C ABI v31 (E06 T2c: the record's identity, N capped) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q9, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

**Ownership**
- The battle lane still writes rules the encodings show: a new observation feature or action entry comes as a `cross-lane` pull request from the battle lane with this lane's version bumps (`docs/08-work-process.md`, "Lanes"). This lane's own changes to `h3-battle` (an engine query the env needs) are `cross-lane` the other way.
- `docs/07-simulation-and-ai.md` is shared: only `--scope ai` checks it (T3a's edit slipped in).
- `h3-cli` is the battle lane's except `src/ai/`; its golden replays (`fixtures/`) and `benches/battle/` stay with battle, and change when the encodings or the replay format change: regenerate them deliberately (`fixtures/README.md`) in the same pull request.

**Environment**
- An agent sees the projected observation only (AI-Q1 decision 10); `fork` and `state_hash` read the full state, for tools and training (the C ABI is for trusted local consumers, ADR-0011; never forward the hash to a client).
- `BattleSetup.human` and `.difficulty` change the computer's automatic choices (the turrets, the automatic Ballista, the moat avoidance; ADR-0023 point 7): the batch sets `human = [true, true]` for every episode (ADR-0025 point 6) and masks Retreat and Surrender out (point 7); `BattleEnv` and the single-env C functions stay as they are (replays and the battle benchmarks use escapes).
- `verify --scope ai` skips the golden replays (their crate `h3-cli` is the battle lane's) and the battle benchmarks: after a change to `h3-sim` run `cargo xtask determinism` and `cargo xtask bench-check --workload random-rollouts-full` too.
- `legal_commands` lists no casts: the cast heads come from `cast_targets` × `second_targets` (the battle lane's trap; `h3_sim::action`).
- The batch (`h3_sim::batch`) skips an episode that ends before an external seat could act and gives up after 64 in a row (no side to move until a reset): a scenario whose internal seat always acts first and ends it (a step limit of one) does that. Its per-env values come back as `EnvReport`s, which `h3-ffi` converts for the C arrays.
- `batch::pool` holds this lane's only `unsafe` outside `h3-ffi`: `Pool::run` erases a job's lifetime and is sound only because it blocks until every worker is done (a worker's panic too). Keep any new call through `run`, never a stored job.
- The batch workloads count inside `*BattleBatch>::step` (v0 symbol names; `BattleBatch::step` is `#[inline(never)]`) with `--threads 0`: callgrind toggles per thread, and a pattern ending in `*` also matches the closures nested in `step` and turns the count off inside them.
- The census counts through `h3_map::import_map`, a `GameState` and `battle_ground`: a change to
  the map import or the adventure's battle place changes it; rerun `h3 ai census
  original-data/install/Maps --out crates/h3-sim/data/census-v1.toml` (bump `CENSUS_VERSION` when
  a number changes meaning). The maps arrive as `original-data/maps.tar` in one staging call.
- A batch entry is a fixed battle or the encounter distribution (`Scenario::encounters`): its
  `setup()` and `seats()` are `None`, each episode's come from `Scenario::episode` (stream
  `ENCOUNTER` of the episode's seed, never `SETUP`, which draws the entry).
- The encounter sampler (`h3_sim::encounter`) uses the engine's rules (ADV-MON-003, -004, -009,
  `derive_battle_hero`): a change to them changes the encounters; `encounter::tests` checks every
  dimension against its weight (5 σ). By default the neutral kinds weigh 0 (A12, until E23);
  tests draw them with `with_shares`. Everything is an integer (quantile tables in ‰).
- Scenario texts are parsed and packs compiled in `h3-ffi` (`src/scenario.rs`), never in `h3-sim`: `arch.toml` gives `h3-sim` `h3-pack-battle` for tests only. Each `h3_env_create_scenario` compiles its packs (about 18 ms for `base`, `ab`, `sod` in release, ten times that in debug).

- The bots (`h3_sim::bots`) read the engine's preview (`h3_battle::preview`, BAT-DMG-011): a
  change to the strike stages (or any rule) moves them and the bots' table
  `python/records/bots-report-v1.json`: regenerate it (`python/records/README.md`; a test
  catches stale ruleset hashes, not moved code). The held-out sets' seeds live in `evaluate.py`
  and `src/ai/eval.rs`, pinned by an `h3-cli` test.
- Reward v1 reads the war machines from the outcome's records (what the battle placed: a
  siege's Catapult in, a Ballista BAT-QUIRK-23 leaves out not), never `setup.war_machines`.
- `h3env.BattleBatch`'s output arrays are read-only attributes (the engine holds their
  addresses): copy or write into them, never rebind; its head arrays must be N long.

**Python**
- The trainer (`h3env.train`, ADR-0026): its tests are in the lane command (`uv run --group
  train pytest`); a run is a TOML config (`python/configs/`, `--set`), its budget and cadences in
  samples; its files go into `python/runs/` (ignored). A piece of training state goes into
  `Trainer.state` and `restore` together, and into `test_a_stopped_run_resumes_to_its_budget`'s
  comparisons (checkpoint format 2, T3b; the promoted networks stay in `promoted/`, which a
  resume reads); `resume` checks before `Run(…, resume=…)` cuts the metrics (a refused resume
  changes nothing). `h3env.train.run` stays free of torch
  (the package loads its torch modules lazily); `tests/test_run.py` runs without torch
  (to block it: a `torch.py` raising `ModuleNotFoundError` first on `PYTHONPATH`; pytest 9). In a background shell `cd`
  may not hold: run `uv --directory python run …` (a plain `uv run` from the root takes the
  system Python). A run in the sandbox competes with a verify for the 4 cores: run one at a time.
- `python/` is a uv project (`uv sync`, `uv run pytest`, `uv run python -m h3env.bench`); its
  tests are a lane command, not a gate (the user, 2026-10-18), and need the release library
  (`cargo build --release -p h3-ffi`, else `$H3_LIBRARY`). A C ABI bump changes `h3env`'s
  `ABI_VERSION` and its layouts (`_ffi.py`) in the same change: the binding refuses any other
  version. Its action constants (Retreat, Surrender, the casts) are action encoding v13's.
- PyTorch 2.14 (CUDA 12.6 build) is the last prebuilt release for the user's GTX 1070: pinned in
  the opt-in group `train` (`uv sync --group train`), torch from `download.pytorch.org/whl/cu126`
  only. PyPI's torch 2.14 is not it (Linux: CUDA 13; Windows: CPU only). The user works with
  Anaconda (system Python older than 3.11): uv in a conda env, `UV_PYTHON` on its Python
  (`python/README.md`); runs on the user's machine are given as those commands.
- The sandbox reaches PyTorch's index (`download.pytorch.org`, `download-r2.pytorch.org`) only
  while the environment's network access is **Custom** with `*.pytorch.org`: **Trusted** ignores
  the Allowed domains. The proxy's 403 says why in its body, which curl hides: send a raw
  `CONNECT host:443` to `$HTTPS_PROXY` ("no rule or allowlist entry allows host", 2026-10-19).
  `uv --directory python sync --group train`: 3 to 25 minutes.

## Recent sessions

### 2026-10-20 — E06 T3b (the evaluation gate)

- #324 merged; the user: "no need to cut" T3b. The SPRT gate, promotions, ladders, record schema
  4, checkpoint format 2, `--init-from`. 87 Python tests with torch (CPU), 69 without; the scoped
  verify (Python only); a real CLI run in the sandbox (`E06-log.md`).

### 2026-10-20 — E06 T3a (the held-out sets v1, the bots' table)

- Offered T3a in two pieces; the user chose one pull request. `h3 ai eval --set`, `--json`, the
  report set in Python, the bots' table, the bots' fallbacks tested; the full verify.
