# Run records

Every training run the epics' results cite has its record here (ADR-0026 point 9; E06 exit
criterion 1): `<run>.json`, a copy of the run directory's final `record.json`
(`python/runs/<run>/`, git-ignored). Checkpoints and networks stay on the machine that trained
them.

A record (schema 1, `h3env.train.run.Record`; `docs/epics/E06-design.md` §4.2):

- `id`: the run's name (its config's `name`, else its directory's).
- `identity`: when it started, the commit and whether tracked files differed from it (`dirty`),
  the C ABI version, the observation and action sizes, the network's parameters, torch, CUDA,
  the device and GPU, the OS, CPU, cores, Python and numpy.
- `config`: every setting, resolved, by table (as the config file groups them).
- `progress`: samples, updates, episodes, minutes and how it ended (`budget`).
- `curve`: per evaluation interval, the means of its updates' losses, entropy, KL, clip
  fraction, explained variance, samples per second and training rates.
- `evaluations`: each evaluation against Random on the held-out set v0 (the sampled policy's
  rates per side and kind; greedy play's and Random's own), and `best`.

`tests/test_run.py` checks every record here against its schema (and every config in
`python/configs/`). To commit a run's record: copy `runs/<run>/record.json` to
`records/<run>.json`.

## Lineage

The promoted networks (from E06 T3) and the run that holds the current best. None yet.
