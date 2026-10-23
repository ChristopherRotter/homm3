# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`):
  increments 1 to 5 done (#306 to #432). Best so far `t7-1` (20 M, a third neutral): 0.828,
  0.551, 0.528 against Random, Aggressor, Greedy, the system actor 0.374; the neutral set's value
  +0.245 below the target +0.285. **Now:** T6a, the behaviour analysis (QUEUE), then increment
  6's lever.
- **Versions:** battle action encoding v13, observation encoding v21, replay format v30 (E23 T6), the adventure env's action encoding v4 (E08 T45), C ABI v33 (E06 T5: the kill preview) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

**Ownership**
- The battle lane still writes rules the encodings show: a new observation feature or action entry comes as a `cross-lane` pull request from the battle lane with this lane's version bumps (`docs/08-work-process.md`, "Lanes"). This lane's own changes to `h3-battle` (an engine query the env needs) are `cross-lane` the other way. A new encoding also changes `python/h3env/train/layout.py` (the entity network's; `h3-sim`'s `the_python_layout_is_the_encodings` tests fail otherwise).
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
  them changes it; rerun `h3 ai census original-data/install/Maps --out
  crates/h3-sim/data/census-v1.toml` (`CENSUS_VERSION` when a number changes meaning).
- A batch entry is a fixed battle or the encounter distribution (`Scenario::encounters`): its
  `setup()` and `seats()` are `None`, each episode's come from `Scenario::episode` (stream
  `ENCOUNTER` of the episode's seed, never `SETUP`, which draws the entry).
- The encounter sampler (`h3_sim::encounter`) uses the engine's rules (ADV-MON-003, -004, -009,
  `derive_battle_hero`): a change to them changes the encounters (`encounter::tests`: 5 σ). The
  neutral kinds weigh 0 by default (the held-out sets; a run's `[env] kinds` draws them, T7);
  tests use `with_shares`. Their defender is the system actor (`Policy::System`): a
  `POLICY_VERSION` bump moves their battles (records name `system_policy`; design §12), and the
  trainer gives their episodes no league member (`evaluate.NEUTRAL_KINDS`).
- Scenario texts are parsed and packs compiled in `h3-ffi` (`src/scenario.rs`), never in `h3-sim`: `arch.toml` gives `h3-sim` `h3-pack-battle` for tests only. Each `h3_env_create_scenario` compiles its packs (about 18 ms for `base`, `ab`, `sod` in release, ten times that in debug).

- The bots (`h3_sim::bots`) read the engine's preview (`h3_battle::preview`, BAT-DMG-011): a
  change to the strike stages (or any rule), or a `POLICY_VERSION` bump (the system actor is a
  bot of both tables since T7c), moves them and the bots' tables
  `python/records/bots-report-v1.json`, `bots-neutral-v1.json`: regenerate both
  (`python/records/README.md`; a test catches stale ruleset hashes and policy versions, not
  moved code; the battle lane's inbox says so). The held-out sets' seeds live in `evaluate.py`
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
  comparisons (checkpoint format 3, T4a; the promoted networks and the league's stay in their
  files, which a resume reads: a file the checkpoint names is deleted only after the next one); `resume` checks before `Run(…, resume=…)` cuts the metrics (a refused resume
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
- A network is made only by `policy.build` from a config (a saved file's: without `kind` an
  MLP); `Mlp`'s parameter names stay, so older files load. Time a kind with `python -m
  h3env.train.network --time` (the sandbox's CPU is noisy: `--repeats 30`).
- A user's run: they upload `record.json`, `metrics.jsonl`, `best.pt`; `evaluate.load` builds
  a network on the CPU.
- The sandbox reaches PyTorch's index only while the environment's network access allows
  `*.pytorch.org` (**Custom**; **Trusted** ignores the Allowed domains). `uv --directory python
  sync --group train`: 3 to 25 minutes.

## Recent sessions

### 2026-10-23 — E06 T6a, the behaviour analysis (in progress)

- Built and pushed (`claude/brave-wright-msoutb`): `python -m h3env.train.inspect` (E06-log,
  T6a); found the turret shot's preview (battle lane's inbox). **Next:** the user's
  `runs/t7-1/best.pt`, the analysis against the system actor and Greedy, the findings in
  E06-log, the pull request, increment 6's lever put to the user.

### 2026-10-23 — T7d's mix; T5's rest beside it

- The user: T7d at a third neutral (their machine); here the mana price's preparation (#429),
  then AI-Q7 (b) part 1 (#432): the preview of every entry halves the engine's step (29 µs),
  still far above training's 5 000 samples/s. `t7-1` recorded; the user: diagnose first (T6a),
  in a fresh session.
