# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05** (`docs/epics/E05-battle-env.md`): in progress. Increments 1 (T1, T2), 2 (T3a–T3d,
  T4; #252) and 3 (T5a #253, T5b1 and T5b2 #255, T6a #257, T6b and the head calls #258) done.
  **Increment 4** done 2026-10-19: T8 (design §8, AI-Q5; `python/h3env/train/`, C ABI v29,
  #260; the opponent pool, #264): exit criterion 7 met on the user's GTX 1070 (0.800 against
  Random after 40 minutes, best 0.811). **Next:** the user's checkpoint, increment 5 (T7).
- **Order** (BACKLOG): E05, then E23 (the battle lane: the system actor's policy), then E06.
- **Versions:** action encoding v13, observation encoding v21, replay format v29 (taken over from the battle lane), C ABI v29 (T8b: seats for the encounter source) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q6, AI-QUIRK-1, AI-STUB-1.
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

### 2026-10-19 — T8c: two hours on the user's machine; the opponent pool

- The user's hour (GTX 1070) with naive self-play: 0.779 at 12 minutes, 0.721 at the end; the
  defender regressed (0.63 → 0.52), the attacker held at 0.93 (`E05-log.md`). The user chose an
  opponent pool (design §8.2); built with `best.pt` and the double final evaluation fixed
  (#264). The second run (40 minutes): 0.800 final, 0.811 best: criterion 7 met (the user).
- #260 merged and its remote branch was deleted: the follow-up restarted the branch from
  `master` (a plain push recreates it; `--force-with-lease` fails on the stale ref).

### 2026-10-19 — T8a, the smoke training's design; T8b, the trainer and C ABI v29

- #259 had merged. The user chose T8a then T8b, an own PPO; reviewed design §8 (AI-Q5): all
  five as recommended. Built C ABI v29 (seats for the encounter source) and `h3env.train`
  (`E05-log.md`); the sandbox's 30-minute CPU run learns (sampled 0.51 → 0.73 against Random).
- Greedy play loops against Random (up to 17 % truncated): the user moved the criterion to the
  sampled policy (design §8.2). T8c on the user's machine is next (QUEUE).
- For the global HANDOFF (a shared change): `cargo xtask verify` outside
  `scripts/sandbox-verify.sh` needs `LLVM_COV`, `LLVM_PROFDATA` on `/usr/lib/llvm-20/bin`, else
  coverage fails ("LLVM tools not found").

### 2026-10-19 — T6b; the head calls from the kept entry mask; checkpoint of increment 3

- #256 and #257 had merged. PyTorch's index was refused because the environment's network access
  was Trusted (the proxy's 403 body); the user set Custom, and the session reached it at once.
- The user chose T6b with the inbox's head-call cost. Built: the batch keeps the single env's
  entry mask and answers non-cast heads from it (three calls 1.4 → 0.11–0.15 ms for 64 envs);
  the group `train` (torch 2.14.1+cu126) in `uv.lock`, `tests/test_torch.py` (`E05-log.md`).
- #258 merged; on the user's Windows machine 23 passed, the GPU test included (a conda env
  with uv, `python/README.md`). Checkpoint: increment 4 = T8 before T7. Trap for the global
  HANDOFF: a cargo call before `sandbox-setup.sh` makes rustup fetch the pinned toolchain
  without `llvm-tools` (coverage fails; `rustup component add llvm-tools-preview`).
