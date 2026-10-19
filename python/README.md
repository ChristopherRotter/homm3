# h3env — the battle environment from Python

The AI lane's Python binding of the engine (E05 T6a; `docs/epics/E05-design.md` §5): ctypes over
the C ABI (`crates/h3-ffi/include/h3.h`, ADR-0011, ADR-0025), numpy arrays the engine writes in
place.

- `h3env.BattleBatch` — the training batch: N envs per call, scenarios and the encounter
  distribution v1 as episode sources, the head calls for casts, automatic reset.
- `h3env.SeatBook` — each seat's transitions in self-play (a side's rewards between its moves).
- `h3env.tools.Env` — single envs, `fork`, `state_hash`: trusted tooling, never an agent's input.

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
`h3_abi_version()` differs from `h3env.ABI_VERSION`: bump both together.

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
