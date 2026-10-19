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
```

The binding loads `$H3_LIBRARY`, else `../target/release/`, and refuses a library whose
`h3_abi_version()` differs from `h3env.ABI_VERSION`: bump both together.

```python
import numpy as np
import h3env

batch = h3env.BattleBatch(64, threads=4, seed=1)
batch.add_encounters()              # or add_scenario(text, attacker="external", defender="random")
batch.reset()
rng = np.random.default_rng(1)
for _ in range(1000):
    batch.random_actions(rng)       # a policy fills batch.actions from batch.obs and the masks
    batch.step()
```

PyTorch 2.14 (cu126) comes with T6b as the group `train`.
