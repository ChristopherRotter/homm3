# h3env — the battle environment from Python

The AI lane's Python binding of the engine (E05 T6a; `docs/epics/E05-log.md` §5): ctypes over
the C ABI (`crates/h3-ffi/include/h3.h`, ADR-0011, ADR-0025), numpy arrays the engine writes in
place.

- `h3env.BattleBatch` — the training batch: N envs per call, scenarios and the encounter
  distribution v1 as episode sources, the head calls for casts, automatic reset.
- `h3env.SeatBook` — each seat's transitions in self-play (a side's rewards between its moves).
- `h3env.tools.Env` — single envs, `fork`, `state_hash`: trusted tooling, never an agent's input.
- `h3env.train` — the training (E05 T8, `E05-log.md` §8; E06, ADR-0026): an own PPO, self-play on
  the encounter distribution, promoted by an SPRT gate and measured by ladders against the
  engine's bots on held-out episodes (group `train`); `h3env.train.run`, the run's config and
  records, and the gate's arithmetic (`h3env.train.gate.Sprt`) need no torch.

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
encounter distribution's versions, since v32 the system actor's policy's) and
`BattleBatch.entries()` (each episode source's kind, weight and its rulesets' content hashes).

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
uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-1
uv run --group train python -m h3env.train --out runs/x --set samples=2_000_000 --set device.device=cpu
uv run --group train python -m h3env.train --resume runs/t4-1 --set samples=80_000_000
uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-3 --init-from runs/t4-1/best.pt
```

A run's settings come from a TOML config (E06 T2a, ADR-0026; `docs/epics/E06-design.md` §1): the
file states only what differs from `h3env.train.run.Config`'s defaults, grouped in tables (`[env]`,
`[network]`, `[ppo]`, `[league]`, `[gate]`, `[ladder]`, `[checkpoint]`, `[device]`); `--set
table.key=value` overrides one key; an unknown key is an error. Committed configs live in
`configs/`. The budget and every cadence that changes the training count **samples** (learner
decisions that trained): `samples`, the learning rate's linear decay over `lr_horizon` (the budget
by default), the league's snapshots and live draws (`snapshot_every`, `live_every`), the gates
(`gate_every`); wall time sets only the checkpoints.

**The networks** (E06 T5a, `docs/epics/E06-design.md` §11): `network.kind` is `"mlp"` (E05's,
the default: `hidden` 512) or `"entity"`, observation v21 read as entities by `h3env.train.layout`
(a token per stack slot, the globals, the own hero, the field; `layers` 2 transformer blocks of
`width` 128 with `heads` 4; pointer heads at the hexes and stacks); `embed` (32) sizes both
kinds' spell and target embeddings. A saved network's config names its kind (a file from before
T5a is an MLP); `--init-from` a network of another kind is refused. A network at a glance:

```sh
uv run --group train python -m h3env.train.network --config configs/e06-t5.toml --time --mermaid net.md
uv run --group train python -m h3env.train.network --from runs/t4-2/best.pt
```

prints each part's parameters, output shape and multiply-adds per decision (a cast's heads
apart), with `--time` the microseconds per decision of a forward and a training pass (`--device
cuda` on the GPU), and `--mermaid` writes the table with the data flow as a Mermaid diagram.

**A battle cut at `env.max_steps`** (2 000) costs each learning side `ppo.truncation_penalty`
(1.0) and ends its trajectory there (E06 T5c; reward v1 lies in about (−1, 1), so stalling to
the limit never pays); with 0 the trainer bootstraps from the final observation's value instead,
as before T5c. Reward v1 and the evaluation (a truncation is no win) are unchanged.

Each update prints one line (samples, samples/s, self-play's attacker wins, truncations, the
league's size, the learner's score against its members, the frozen networks a call passed, the
losses). Every training episode's seed is checked against the held-out sets' (a hit stops the
run).

**The league** (E06 T4a, ADR-0026 point 8, design §8; `h3env.train.league`): prioritized
fictitious self-play over the learner's past. Its members are a snapshot of the learner every
`league.snapshot_every` samples (1 M; `league/<id>.pt`) and every promoted network (best #0 from
the start), at most `max_members` (16). An episode is self-play with `self_play` (0.5), else a
member of the live set plays one side (only the learner's decisions train): `live` (4) members
drawn every `live_every` samples (250 000) by `(1 − p)^pfsp_power` (2), `p` the learner's score
against the member (a draw or truncation ½, decayed by `decay` 0.99 per update), so the members
it still loses to play most. A new member retires the most beaten (the highest `p` among those
with `retire_games` 200 games); a retired snapshot's file is deleted.

**The gate, the promotions, the ladders** (E06 T3b, ADR-0026 points 6 and 7, design §6 and §7):
the run starts with best #0 (the untrained network, or the parent's with `--init-from`). Every
`gate.gate_every` samples (10 M; 0: only at the end), and at the end if it trained since its last
gate, the learner plays the best on the gate set v1: pairs of one battle played from both sides,
in order, until an SPRT decides (`elo0` 0, `elo1` 20, `alpha` and `beta` 0.05, at most
`max_pairs` 4 000, which reject as inconclusive). A promoted learner becomes `promoted/<n>.pt` and
`best.pt`, and its ladder plays it against `ladder.ladder_opponents` (Random, Aggressor, Greedy)
and the `ladder_promoted` (1) promoted networks before it, on the report set v1's first
`ladder_episodes` (250) per kind; at the end the final best's ladder plays all
`ladder_final_episodes` (500). Each gate, promotion and ladder prints one line;
`python -m h3env.train.gate --candidate b.pt --best a.pt` plays the same gate between any two saved
networks. `--init-from <file>` starts a new run (fresh optimizer, counters and league) from a saved
network (a `best.pt`, `promoted/<n>.pt` or `checkpoint.pt` of the same kind and size), its parent.

The run's directory (`runs/` is git-ignored) gets `config.toml` (the file as given),
`metrics.jsonl` (one typed record per line: `start`, `update`, `snapshot`, `gate`, `promotion`,
`ladder`, `stop`, `segment`, `end`), `record.json` (the run's summary: identity, parent, the
resolved config, progress with the gates' and ladders' minutes, its segments, a curve per gate
interval, the gates with the league's member table, promotions and ladders; rewritten
atomically), `checkpoint.pt` (the whole training state, E06 T2b1), `promoted/<n>.pt` (each
promoted network, kept), `best.pt` (the latest) and `league/<id>.pt` (the league's snapshots).
A record a result cites is committed as `records/<run>.json` (`records/README.md`).

**Interrupting and resuming** (E06 T2b1, T3b, T4a; `docs/epics/E06-design.md` §3): the
checkpoint (format 3: the network, the optimizer, the league's members (their networks stay in
their files), the counters, the schedule, the promoted networks' entries, what is pending, the
random generators, the metrics' length) is written atomically at the start, every `checkpoint.checkpoint_minutes` (5) of wall time at the next
update, before and after each gate and ladder, at a stop and at the end. Ctrl+C stops the run at
its next batch call (the rollout, gate or ladder in progress is discarded) or after the update in
progress, saved, with a `stop` record; a second Ctrl+C aborts at once (back to the last save). A
crash, a kill or a shutdown lose the time since the last save. `--resume <dir>` continues from
the checkpoint in a new **segment**: `metrics.jsonl` is cut back to the save, a gate or ladder the
stop cut short runs first (the gate on the checkpoint's learner, the ladder on its promoted
network, read from `promoted/`; the league's networks from their files, those of `league/` it
does not name removed), the batch starts new episodes on the segment's own seed
(SplitMix64 of `seed`), and the config is the checkpoint's; its `--set`s may change only the
budget (`samples`, extended or cut; the learning rate keeps the horizon fixed at the start),
`env.threads`, `[device]` and `checkpoint.checkpoint_minutes`, any other key is refused (a new
run, or `--init-from`). A finished run resumes with a larger budget only. A resumed run is
reproducible from its record (the segments' seeds), not bitwise equal to an uninterrupted one.
Older checkpoints (formats 1 and 2, before T4a's league) do not resume: start a new run
`--init-from` their `best.pt`.
On Windows a file another process holds open (a ladder reading `best.pt`) is kept as it was after
five refused replaces, with a warning; the run goes on.

A checkpoint against Random and the baseline bots (E05 T7), on a held-out set (E06 T3a): the
report set v1 by default (500 battles per kind with a player on both sides, each played from both
sides; the overall win rate weighs the kinds 4 : 2 : 1, with its 95 % interval; `--episodes n`
plays the first n per kind), or E05's v0 (`--set v0`). The episodes are `h3 ai eval`'s
(`cargo run --release -p h3-cli -- ai eval [--set v0]`); on the whole report set each line ends
with the bots' rates against the same opponent from their table
(`records/bots-report-v1.json`). `--json <file>` keeps the results.

```sh
uv run --group train python -m h3env.train.ladder --checkpoint runs/t4-1/best.pt [--episodes 250]
```
