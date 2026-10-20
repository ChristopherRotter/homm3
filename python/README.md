# h3env — the battle environment from Python

The AI lane's Python binding of the engine (E05 T6a; `docs/epics/E05-log.md` §5): ctypes over
the C ABI (`crates/h3-ffi/include/h3.h`, ADR-0011, ADR-0025), numpy arrays the engine writes in
place.

- `h3env.BattleBatch` — the training batch: N envs per call, scenarios and the encounter
  distribution v1 as episode sources, the head calls for casts, automatic reset.
- `h3env.SeatBook` — each seat's transitions in self-play (a side's rewards between its moves).
- `h3env.tools.Env` — single envs, `fork`, `state_hash`: trusted tooling, never an agent's input.
- `h3env.train` — the training (E05 T8, `E05-log.md` §8; E06, ADR-0026): an own PPO, self-play on
  the encounter distribution, evaluated against the engine's Random on held-out episodes (group
  `train`); `h3env.train.run`, the run's config and records, needs no torch.

## Use

```sh
cargo build --release -p h3-ffi     # the library: target/release/libh3_ffi.so (h3_ffi.dll)
cd python
uv sync                             # .venv with a uv-managed CPython 3.12 and numpy
uv run pytest                       # the binding's tests (a lane command, not a gate)
uv run ruff check; uv run ruff format --check
uv run python -m h3env.bench        # Python's steps/s, the counterpart of `h3 ai bench`
uv sync --group train               # adds PyTorch 2.14 (cu126)
uv run --group train pytest         # with its tests
```

uv: `winget install --id=astral-sh.uv -e` on Windows, else https://docs.astral.sh/uv/. It brings
its own CPython 3.12; to build the venv on an Anaconda Python instead (the user's machine):

```powershell
conda create -n h3 -c conda-forge python=3.12 uv
conda activate h3
$env:UV_PYTHON = "$env:CONDA_PREFIX\python.exe"; $env:UV_PYTHON_DOWNLOADS = "never"
```

`python/.venv` then sits on conda's Python, and the conda env stays as it is. Never point `uv sync`
at a conda env itself (`UV_PROJECT_ENVIRONMENT`): it removes every package the lock does not list.

The binding loads `$H3_LIBRARY`, else `../target/release/`, and refuses a library whose
`h3_abi_version()` differs from `h3env.ABI_VERSION`: bump both together. A run's record names the
engine it trained on (C ABI v31): `h3env._ffi.versions()` (the ABI's, the encodings' and the
encounter distribution's versions) and `BattleBatch.entries()` (each episode source's kind, weight
and its rulesets' content hashes).

```python
import numpy as np
import h3env

batch = h3env.BattleBatch(64, threads=4, seed=1)
batch.add_encounters()  # or add_scenario(text, attacker="external", defender="random")
batch.reset()
rng = np.random.default_rng(1)
for _ in range(1000):
    batch.random_actions(rng)  # a policy fills batch.actions from batch.obs and the masks
    batch.step()
```

The group `train` holds PyTorch 2.14 with its CUDA 12.6 build, the last release with kernels
for the GTX 1070's Pascal GPU (ADR-0025 point 11), from PyTorch's own index
(`download.pytorch.org/whl/cu126`; its dependencies from PyPI). It is opt-in, about 4 GB on
Linux; without it `tests/test_torch.py` is skipped, and its GPU test runs only where CUDA finds
a device. `torch.from_numpy(batch.obs)` and the other arrays share the engine's memory: the
tensors show each call's results without a copy.

## Training runs

```sh
uv sync --group train
uv run --group train python -m h3env.train --config configs/pool-smoke.toml --out runs/smoke
uv run --group train python -m h3env.train --out runs/x --set samples=2_000_000 --set device.device=cpu
uv run --group train python -m h3env.train --resume runs/smoke --set samples=40_000_000
```

A run's settings come from a TOML config (E06 T2a, ADR-0026; `docs/epics/E06-design.md` §1): the
file states only what differs from `h3env.train.run.Config`'s defaults, grouped in tables (`[env]`,
`[network]`, `[ppo]`, `[pool]`, `[evaluation]`, `[checkpoint]`, `[device]`); `--set
table.key=value` overrides one key; an unknown key is an error. Committed configs live in
`configs/`. The budget and every cadence that changes the training count **samples** (learner
decisions that trained): `samples`, the learning rate's linear decay over `lr_horizon` (the budget
by default), the pool's snapshots (`pool_every`), the evaluations (`eval_every`); wall time sets
only the checkpoints.

Each update prints one line (samples, samples/s, self-play's attacker wins, truncations, the
losses), each evaluation its win rates against Random on the held-out set v0 (sampled and greedy,
overall and per side). Self-play keeps an opponent pool (`E05-log.md` §8.2): in `pool_share` of
the episodes a frozen earlier network plays one side. Every training episode's seed is checked
against the held-out episodes' (a hit stops the run). The run's directory (`runs/` is
git-ignored) gets `config.toml` (the file as given), `metrics.jsonl` (one typed record per line:
`start`, `update`, `snapshot`, `eval`, `stop`, `segment`, `end`), `record.json` (the run's summary:
identity, the resolved config, progress, its segments, a curve, the evaluations; rewritten
atomically), `checkpoint.pt` (the whole training state, E06 T2b1) and `best.pt` (the network at
the best sampled win rate). A record a result cites is committed as `records/<run>.json`
(`records/README.md`). E05's criterion of exit 7: the final evaluation's sampled `win_rate` at
least 0.8 (greedy play can repeat itself against Random until the step limit).

**Interrupting and resuming** (E06 T2b1, `docs/epics/E06-design.md` §3): the checkpoint (the
network, the optimizer, the pool, the counters, the schedule, the random generators, the
metrics' length) is written atomically at the start, every `checkpoint.checkpoint_minutes` (5)
of wall time at the next update, after each evaluation, at a stop and at the end. Ctrl+C stops
the run at its next batch call (the rollout in progress is discarded) or after the update or
evaluation in progress, saved, with a `stop` record; a second Ctrl+C aborts at once (back to the
last save). A crash, a kill or a shutdown lose the time since the last save. `--resume <dir>`
continues from the checkpoint in a new **segment**: `metrics.jsonl` is cut back to the save, the
batch starts new episodes on the segment's own seed (SplitMix64 of `seed`), and the config is the
checkpoint's; its `--set`s may change only the budget (`samples`, extended or cut; the learning
rate keeps the horizon fixed at the start), `env.threads`, `[device]` and
`checkpoint.checkpoint_minutes`, any other key is refused (a new run). A finished run resumes
with a larger budget only. A resumed run is reproducible from its record (the segments' seeds),
not bitwise equal to an uninterrupted one.

A checkpoint against Random and the baseline bots (E05 T7), on the episodes of `h3 ai eval`'s
table (`cargo run --release -p h3-cli -- ai eval`):

```sh
uv run --group train python -m h3env.train.ladder --checkpoint runs/pool/best.pt
```
