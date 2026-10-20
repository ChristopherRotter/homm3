# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05** (`docs/epics/E05-battle-env.md`): in progress. Increments 1 (T1, T2), 2 (T3a–T3d,
  T4; #252) and 3 (T5a #253, T5b1 and T5b2 #255, T6a #257, T6b and the head calls #258) done.
  **Increment 4** done 2026-10-19: T8 (design §8, AI-Q5; `python/h3env/train/`, C ABI v29,
  #260; the opponent pool, #264): exit criterion 7 met on the user's GTX 1070 (0.800 against
  Random after 40 minutes, best 0.811). **Increment 5** done 2026-10-19 (T7, design §9;
  AI-Q6): the preview (#274), ADR-0025 point 5 (#271), the bots, `h3 ai eval`, C ABI v30
  (#276), the user's network at Aggressor's level: exit criterion 6 met. **Increment 6**
  (T9, the closure) cut 2026-10-19: T9a (the audit) and T9b (the review) done 2026-10-20 on
  `claude/festive-brown-igukig`, its pull request open (watch it). **Next:** T9c on the user's
  machine (QUEUE: the commands), then T9d.
- **Order** (BACKLOG): E05, then E23 (the battle lane: the system actor's policy), then E06.
- **Versions:** action encoding v13, observation encoding v21, replay format v29 (taken over from the battle lane), C ABI v30 (T7: the bots' seats) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q8, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

**Ownership**
- The battle lane still writes rules the encodings show: a new observation feature or action entry comes as a `cross-lane` pull request from the battle lane with this lane's version bumps (`docs/08-work-process.md`, "Lanes"). This lane's own changes to `h3-battle` (an engine query the env needs) are `cross-lane` the other way.
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
  change to the battle's strike stages moves them and `h3 ai eval`'s table; its default episodes
  are the Python evaluation's (`EVAL_SEED`, both literals pinned by an `h3-cli` test).
- Reward v1 reads the war machines from the outcome's records (what the battle placed: a
  siege's Catapult in, a Ballista BAT-QUIRK-23 leaves out not), never `setup.war_machines`.
- `h3env.BattleBatch`'s output arrays are read-only attributes (the engine holds their
  addresses): copy or write into them, never rebind; its head arrays must be N long.

**Python**
- The trainer (`h3env.train`, design §8): its tests are in the lane command (`uv run --group
  train pytest`); a run's files go into `python/runs/` (ignored). In a background shell `cd`
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

## Recent sessions

### 2026-10-19/20 — Increment 6's cut; T9a, the audit, and T9b, the review

- The user cut increment 6 (T9a to T9d; four reviewers; AI-Q7 to E06; the Linux verify with
  original data in a later staged sandbox). T9a: criteria 1 to 7 met, the inbox by destination.
- T9b: four fresh reviewers, 21 findings (`E05-log.md`); the user chose the Catapult as a placed
  war machine and one Glicko-2 period per run. A tried fix of A3's growth broke A4's premise:
  reverted, noted for E23.
- Python's `h3env.train` tests need torch (`--group train`), absent in the sandbox: the
  evaluation's skip count was checked ad hoc by loading `evaluate.py` without the package.

### 2026-10-19 — Increment 5: T7, the baselines (AI-Q6), and the network's row

- #270 merged. The user cut increment 5 (T7) and decided AI-Q6 (the bots read the engine's
  preview of their own attacks, never the retaliation; Glicko-2 from the server). Opened #271
  (`shared/`, ADR-0025 point 5) and #274 (`cross-lane`, `h3_battle::preview`).
- Built the bots, `h3 ai eval` (Greedy 0.828, Aggressor 0.800 against Random), C ABI v30 and
  `h3env.train.ladder` (`E05-log.md`); #271, #274, #276 merged. The user's network: 0.802
  against Random, 0.498 against Aggressor, 0.479 against Greedy (T7d).
- For the global HANDOFF (a `shared/` change, kept from T8b): `cargo xtask verify` outside
  `scripts/sandbox-verify.sh` needs `LLVM_COV`, `LLVM_PROFDATA` on `/usr/lib/llvm-20/bin`.

### 2026-10-19 — T8c: two hours on the user's machine; the opponent pool

- Naive self-play regressed on the user's GTX 1070 (0.779 peak, 0.721 final); with the opponent
  pool (#264) 0.800 final, 0.811 best: criterion 7 met (`E05-log.md`).
