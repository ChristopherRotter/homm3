# Run records

Every training run the epics' results cite has its record here (ADR-0026 point 9; E06 exit
criterion 1): `<run>.json`, a copy of the run directory's final `record.json`
(`python/runs/<run>/`, git-ignored). Checkpoints and networks stay on the machine that trained
them.

A record (schema 3, `h3env.train.run.Record`; `docs/epics/E06-design.md` §4):

- `id`: the run's name (its config's `name`, else its directory's).
- `identity`: when it started, the commit and whether tracked files differed from it (`dirty`),
  the C ABI version, the observation and action encodings' versions and sizes, the encounter
  distribution's version, the batch's `entries` (each one's kind, weight and the content hashes
  of its core, battle and adventure rulesets; schema 3, E06 T2c), the network's parameters,
  torch, CUDA, the device and GPU, the OS, CPU, cores, Python and numpy.
- `config`: every setting, resolved, by table (as the config file groups them).
- `progress`: samples, updates, episodes, minutes and how it ended (`budget`, `stop`).
- `segments` (schema 2, E06 T2b1): the run's first part and each `--resume`'s, in order: its batch
  seed, when it started, the commit and `dirty`, the device, the config keys it changed, its
  samples, updates and minutes at its start and end, how it ended (`budget`, `stop`, or `lost`: no
  record of its end, a kill or a crash) and its records dropped after its last checkpoint.
- `curve`: per evaluation interval, the means of its updates' losses, entropy, KL, clip
  fraction, explained variance, samples per second and training rates.
- `evaluations`: each evaluation against Random on the held-out set v0 (the sampled policy's
  rates per side and kind; greedy play's and Random's own), and `best`.

`tests/test_run.py` checks every record here against its schema (and every config in
`python/configs/`). To commit a run's record: copy `runs/<run>/record.json` to
`records/<run>.json`.

## The bots' table

`bots-report-v1.json` (E06 T3a; design §7): Random, Aggressor and Greedy against each other on
the report set v1, every ordered pair on its 1 500 battles: per kind and side the games, wins,
truncations and mean rewards, per pair the overall win rate with its 95 % interval and the value
efficiency, the Glicko-2 ratings, and the engine's identity (the encounter distribution's
version, the rulesets' content hashes). It is a function of the engine; `tests/test_evaluate.py`
checks its strata, its identity against the engine's and its rates. Regenerate it when the bots
or the rules move (from the repository's root, about 15 s in release):

```sh
cargo run --release -p h3-cli -- ai eval --set report-v1 --json python/records/bots-report-v1.json
```

## Lineage

The promoted networks (from E06 T3) and the run that holds the current best. None yet.
