# Run records

Every training run the epics' results cite has its record here (ADR-0026 point 9; E06 exit
criterion 1): `<run>.json`, a copy of the run directory's final `record.json`
(`python/runs/<run>/`, git-ignored). Checkpoints and networks stay on the machine that trained
them.

A record (schema 5, `h3env.train.run.Record`; `docs/epics/E06-design.md` §4; `check_record`
also accepts schema 4, before the league: `t3b-1`, its config's `[pool]`):

- `id`: the run's name (its config's `name`, else its directory's).
- `identity`: when it started, the commit and whether tracked files differed from it (`dirty`),
  the C ABI version, the observation and action encodings' versions and sizes, the encounter
  distribution's version, the batch's `entries` (each one's kind, weight and the content hashes
  of its core, battle and adventure rulesets; schema 3, E06 T2c), the network's parameters,
  torch, CUDA, the device and GPU, the OS, CPU, cores, Python and numpy.
- `parent` (schema 4, E06 T3b): the network the run started from (`--init-from`: its file, run,
  promoted index, samples and updates), or null.
- `config`: every setting, resolved, by table (as the config file groups them).
- `progress`: samples, updates, episodes, minutes, the gates' and ladders' minutes
  (`evaluation_minutes`, their share of the run) and how it ended (`budget`, `stop`).
- `segments` (schema 2, E06 T2b1): the run's first part and each `--resume`'s, in order: its batch
  seed, when it started, the commit and `dirty`, the device, the config keys it changed, its
  samples, updates and minutes at its start and end, how it ended (`budget`, `stop`, or `lost`: no
  record of its end, a kill or a crash) and its records dropped after its last checkpoint.
- `curve`: per gate interval, the means of its updates' losses, entropy, KL, clip fraction,
  explained variance, samples per second and training rates (schema 5: the learner's score
  against the league's members, `league_score`, and the frozen networks a call passed,
  `frozen_passes`, instead of the pool's win rate).
- `gates` (schema 4): each gate's samples, update, the best it played, its result (`promoted`,
  `rejected`, `inconclusive`), pairs, the pairs' counts per score, LLR, mean score, Elo, games,
  seconds and (schema 5, E06 T4a) the league's member table at the gate (each member's id, kind,
  birth in samples, `p`, decayed and plain games, whether live; the last gate's is the end's); `promotions`: each promoted network's index, how (`start`, `init`, `gate`), samples
  and update; `ladders`: each ladder's promoted network, whether final, episodes per kind, and per
  opponent the win rate, its 95 % interval, the value efficiency, each kind's rate and (after
  `t3b-1`) its truncations' share; `best`: the latest promotion's index and samples.

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

The promoted networks (from E06 T3b: a run's `promoted/<n>.pt`, its record's `promotions`) and
the run that holds the current best; a run started `--init-from` another's network names it as
its `parent`.

- `t3b-1` (E06 T3b's hour, 2026-10-20): promoted #0 to #4; its best #4 (25 M samples) at the
  bots' level (`docs/epics/E06-log.md`). Its networks are on the user's machine
  (`python/runs/t3b-1/`).
- `t4-1` and `t4-2` (E06 T4b, 2026-10-21 and 22; the league, 60 M samples each, γ 0.995 and γ
  1.0): promoted #0 to #4 and #0 to #5; `t4-2`'s best #5 (50 M samples) is the lineage's
  strongest (Random 0.839, Aggressor 0.578, Greedy 0.559 on the full report set). Their networks
  are on the user's machine (`python/runs/t4-1/`, `t4-2/`).
