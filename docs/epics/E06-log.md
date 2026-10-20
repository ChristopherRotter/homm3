# E06 — Log

The full record of E06's done tasks (`docs/epics/E06-battle-ai.md` keeps one line each). Read an
entry only where a task builds on it.

## T1 — Design and ADR draft (2026-10-20)

- **The seam:** offered T1 narrowed to the run's lifecycle (config, record, resume) with the gate,
  the ladder and the league designed at their own increments; the user chose T1 as queued, the
  whole pipeline's design in one note.
- **Written:** `E06-design.md` (§1 the run and its config, §2 the schedule in samples, §3 the
  resumable state, §4 the records, §5 the held-out sets, §6 the gate, §7 the ladder, §8 the
  league, §9 the costs, §10 the increments) and the ADR draft, nine decisions.
- **The user's review** (2026-10-20): every point as proposed, the ADR accepted as **ADR-0026**
  (#306, `shared/`). The points reviewed:
  1. TOML configs in, JSON records out; committed records in `python/records/`, configs in
     `python/configs/`.
  2. On resume only the budget, threads, device and checkpoint cadence may change; anything else
     is a new run (`--init-from`).
  3. The learning rate's horizon fixed at the start; an extended budget runs at the floor.
  4. Checkpoints every 5 minutes of wall time (they change no training); every training cadence
     in samples.
  5. The report set v1: 500 per (kind, side), stratified; 250 per kind for intermediate
     ladders. A separate gate set.
  6. The SPRT: elo0 0, elo1 20, α = β = 0.05, at most 4 000 pairs, no promotion at the cap, no
     second condition against the bots.
  7. The targets judged on the latest promoted network, point estimates against the targets.
  8. Ratings only in `h3 ai eval`, outside the promotion.
  9. The league: self-play 0.5, PFSP "hard" (power 2), 16 members, 4 live per rollout, a
     snapshot every 1 M samples.
  10. T2 in three pull requests (T2a to T2c); the observations in place moved to T5.
- **The cut:** increment 1 continues with T2a, then T2b and T2c (design §10); T1 and T2a go to
  `master` in one pull request (the user).
- **Verify:** the sandbox's full verify on engine code `d95c95dd` (no original data) green; the
  scoped verify of the docs change with no failing step. CI's runs on `master` never started
  (minutes).

## T2a — Config and record (2026-10-20)

- **Built** (design §1.2, §2, §4, §5.3; ADR-0026 points 1, 2, 4, 9): `h3env.train.run`, free of
  torch: `Config` (E05's settings, its keys grouped in tables: `[env]`, `[network]`, `[ppo]`,
  `[pool]`, `[evaluation]`, `[device]`), `load_config` (TOML, `--set table.key=value`, unknown
  tables and keys refused with the table a key belongs to, wrong types refused), `resolve`
  (`lr_horizon` 0 fixed at the budget, `seed == eval_seed` refused), `learning_rate` (linear to
  the floor over the horizon, or constant), `Cadence` (due once per multiple of so many samples),
  `atomic_write` (a temporary file, fsync, replace; five tries on Windows), `HeldOut` (the
  evaluation's fixed seeds, every training episode's checked: E05's growing set gone), `Run` and
  `Record` (typed `metrics.jsonl`: `start`, `update`, `snapshot`, `eval`, `end`; `record.json`
  schema 1: identity, config by table, progress, the curve per evaluation interval, the
  evaluations, the best; `check_record`); a directory that already holds a run's metrics is
  refused (a new run never appends to another's records; resuming is T2b's).
- **The trainer** (`ppo.py`): the budget in samples (E05's `minutes` and `updates` go), the
  snapshots (`pool_every` 600 000) and the evaluations (`eval_every` 5 M) in samples, checkpoints
  written atomically, the identity (commit and whether tracked files differed, ABI, sizes,
  parameters, torch, CUDA, device and GPU, OS, CPU, cores, Python, numpy). `python -m
  h3env.train --config <file> --set … --out <dir>`; `config.json` gave way to `config.toml` (the
  file as given) and `record.json`. The package loads its torch modules lazily (PEP 562), so
  `h3env.train.run` needs no torch.
- **Files:** `python/configs/pool-smoke.toml` (E05's pool run in samples, about 40 minutes on the
  user's machine), `python/records/README.md` (the schema, how to commit a record, the lineage).
  `requires-python >= 3.11` (`tomllib`): the lock dropped only 3.10's entries.
- **Tests:** `tests/test_run.py` (23 cases, torch-free: the file and `--set`, thirteen refused
  configs, tables and the record's config, the command line, the learning rate, the cadence,
  atomic writes and a stale temporary, the held-out check, a run's metrics and record from
  synthetic records, `check_record`, every committed record and config); `test_train.py` on the
  new config (a budget just past one update's samples: two updates, the learning rate decaying,
  a snapshot per update, the record valid; a held-out hit at the reset stops the trainer). The
  sandbox reached PyTorch's index this session: 55 tests pass with torch 2.14.1+cu126 on the CPU
  (the GPU test skipped); with torch blocked the lifecycle tests pass and the trainer's skip. A
  tiny run through the command line (`--config configs/pool-smoke.toml` with `--set`s, 490
  samples) printed its lines and wrote a valid record.
