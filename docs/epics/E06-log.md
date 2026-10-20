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

## T2b1 — The full checkpoint and `--resume` (2026-10-20)

- **The seam:** offered T2b cut at the gate (the user chose "T2b1: resume core" over T2c and a
  torch-free-only part): the pending gate or ladder at the resume and `--init-from` need T3's
  gate and promoted list, so they moved to T3b (design §3, §10).
- **Built** (design §1.3, §3; ADR-0026 point 3; decision 4): `h3env.train.run` (torch-free):
  `[checkpoint]` (`checkpoint_minutes`, 5), `RESUMABLE` and `resumed` (the checkpoint's config
  with `--set`s; only `samples`, `env.threads`, `[device]` and `checkpoint_minutes` change, any
  other key refused with its path named; `lr_horizon` stays the start's), `segment_seed` (segment
  0: `seed`; segment s: SplitMix64's s-th output from `seed`), `Stop` (the first SIGINT a flag,
  the second `KeyboardInterrupt`; installed in the main thread only), `Run(…, resume=(length,
  seconds))` (stale `*.tmp` removed, `metrics.jsonl` cut back to the checkpoint's length, the
  records after it counted as dropped, the record rebuilt from the rest, the clock continued),
  `Run.sync` (fsync before a checkpoint names the length), the `start` record with the identity,
  `segment` and `stop` records, `record.json` schema 2 (`segments`: seed, start time, commit,
  `dirty`, device, changed keys, start and end, how it ended: `budget`, `stop`, `lost`, dropped).
- **The trainer** (`ppo.py`): `Trainer.state` (format 1: config, segment, counters, best rate,
  the cadences' next samples, network, optimizer, the pool's networks, the generators: sampling,
  the pool's numpy one, torch's CPU and CUDA states; the run's seconds, the metrics' length,
  `finished`) and `restore` (the sampling generator restored on a device of the same type, else
  seeded from the segment's batch seed); checkpoints at the start, every `checkpoint_minutes` at
  the next update boundary, after each evaluation, at a stop and at the end; `best.pt` the network
  alone. A stop discards a rollout in progress (its episodes no longer counted) and lets an update
  or an evaluation finish. `resume` refuses a missing checkpoint, another format (E05's and
  T2a's `checkpoint.pt` held the network and the optimizer only; the ladder still reads them) and
  a finished run without a larger budget; it builds the trainer before it touches the directory,
  so a refused resume changes nothing. `python -m h3env.train --resume <dir> [--set …]`
  (`--config` and `--out` refused with it). Fixed in passing (T2a's): the update record's
  `episodes` was the rollout's self-play count (`**roll` after the counters), so the record's
  `progress.episodes` was the last rollout's; the rollout's count is `self_play_episodes` now.
- **Tests:** `test_run.py` (+4, torch-free: the allowed and refused changes, SplitMix64's
  reference outputs and 50 distinct segment seeds, Ctrl+C by `signal.raise_signal`, the cut-back
  with a half-written line and a stale temporary, the record's segments); `test_train.py` (+2: a
  run stopped by SIGINT after its third update, the next segment's trainer equal to the saved
  state (network, optimizer, the pool's three networks, the generators), a refused `ppo.lr`
  leaving the metrics untouched, the resume to an extended budget with update numbers strictly
  increasing; a subprocess killed hard (SIGKILL, TerminateProcess on Windows) while writing its
  fourth checkpoint, resumed from the third, the `.tmp` gone, one record dropped, its segment
  `lost`). 61 tests pass with torch 2.14 on the CPU (the GPU test skipped), 48 with torch
  blocked. Through the command line: a tiny run stopped by `kill -INT` after 16 updates, a
  refused `ppo.lr`, the resume with a cut budget to its end, the record valid.

## T2c — C ABI v31 (2026-10-20)

- **Built** (design §4; ADR-0026 point 9; exit criterion 1): `h3_sim::MAX_ENVS` (65 536) and
  `BatchError::TooManyEnvs` (`h3_batch_create` aborted on the allocation of an absurd N, which
  no `catch_unwind` catches; now it returns the message), `h3_sim::ENCOUNTERS_VERSION` (1, the
  table's parse checks it). C ABI v31: `h3_observation_encoding_version`,
  `h3_action_encoding_version`, `h3_encounters_version`, `h3_batch_scenario_count`,
  `h3_batch_scenario_info` (`H3ScenarioInfo`: the core and battle rulesets' content hashes, a
  replay header's two, the encounter sampler's adventure ruleset's or 0, the weight, whether
  the entry is the encounter distribution), `H3_BATCH_MAX_ENVS`; the encodings unchanged.
- **The binding:** `ABI_VERSION` 31, `_ffi.versions()`, `BattleBatch.entries()` (hashes as 16
  hex digits, so JSON readers keep them whole); `ppo.identity` puts both into the record:
  schema 3, `check_record` requires the new keys.
- **Tests:** `h3-ffi` (`each_entry_names_its_rulesets`: every function's null and range errors,
  the hashes against the Rust batch's, the town and the encounters sharing the original packs'
  ruleset; an N of `u32::MAX` refused with its message), `h3-sim` (the new error and its
  message), the C smoke test (the versions, the entries of its batch, the cap); Python:
  `test_abi.py` (the version, the versions, the layout's 32 bytes), `test_batch.py` (the
  entries, the cap through the binding). 62 Python tests with torch on the CPU, 49 without.

## T3a — The held-out sets v1, the ladder on them, the bots' table (2026-10-20)

- **Cut:** the user asked for a small increment; offered T3a in two pieces (Rust, then Python),
  the user chose all of T3a in one pull request.
- **Built** (design §5, §7; ADR-0026 points 4, 5, 7): `h3 ai eval --set report-v1|v0` (the
  report set v1 the default; `--episodes` per kind, at most 500; `--envs`, `--seed` and
  `--encounters` for v0 only) and `--json <file>`. A set is a list of strata (`eval::Stratum`):
  the report set v1 one per kind with a player on both sides (hero, siege a hero defends, a
  garrison's siege), batch seed `REPORT_V1_SEED + k` (`0x5EED_E7A1_0001_0000 + k`), the kind
  one-hot, the first episode of each env; v0 E05's single stratum. Per pair: the overall win rate
  (the kinds at the distribution's weights 4 : 2 : 1, the sides equally) with its 95 % interval
  (the stratified variance), each kind's rates per side, the value efficiency, truncations; the
  JSON adds the engine's identity (`ENCOUNTERS_VERSION`, the core, battle and adventure rulesets'
  content hashes). `bench::sampler` and `encounters_of` share one compiled sampler across strata.
- **Python** (`h3env.train.evaluate`): `REPORT_V1_SEED`, `REPORT_V1_ENVS`, `REPORT_V1_WEIGHTS`,
  `GATE_V1_SEED` (`0x5EED_E7A1_0002_0000`, 500 envs × 8 episodes, for T3b), `report` (each
  (kind, side) its own batch and its own generator seed: common random numbers per (kind, side)),
  `stratified`, `bots_table`; `play` takes `sides`; `ladder(held_out="report-v1"|"v0")`;
  `held_out_seeds` (v0's, the report set's 1 500 and the gate set's 4 000) in the trainer's
  held-out check; `HELD_OUT_BATCH_SEEDS`: `run.resolve` refuses a run's `seed` among them, the
  ladder a network trained on one. `python -m h3env.train.ladder --set`, `--json`; on the whole
  report set each line ends with the bots' rates against the same opponent from their table.
- **The bots' table** (`python/records/bots-report-v1.json`, `h3 ai eval --set report-v1 --json`,
  13.5 s in release in the sandbox; 9 000 games, none truncated): Greedy 0.832 ± 0.013 and
  Aggressor 0.810 ± 0.014 against Random, Greedy 0.533 ± 0.018 against Aggressor (v0: 0.828,
  0.800, 0.519); value Greedy +0.275, Aggressor +0.249 against Random, Greedy +0.023 against
  Aggressor; Glicko-2 Greedy 1659, Aggressor 1622, Random 1219 (± 13). The garrison's siege
  favours the attacker strongly (Random attacking wins 0.796 against Aggressor's garrison, 0.756
  against Greedy's); a hero battle the attacker too (Greedy attacking Aggressor 0.746, defending
  0.330).
- **Tests:** `h3-cli` (the seeds and the report set's size pinned to Python's; the runner on v0
  as before plus its interval; the report set's strata, the 4 : 2 : 1 overall rate, design §5's
  ±1.8 points at 0.75, the JSON, the command line's refusals and `--json`); `h3-sim` (the bots'
  fallbacks, E05 T9b's inbox: both bots end the tactics phase, both shoot the walls with a
  commanded Catapult); Python `test_evaluate.py` (new, no torch: the report set's batches and
  seeds, every held-out set in the trainer's check, the stratified rates and interval, Random
  against a bot on the report set reproducibly, the committed table's strata, engine identity
  and rates recomputed by `stratified`), `test_run.py` (the seeds refused, the records' test skips
  `bots-*.json`), `test_train.py` (the ladder on both sets, common random numbers, the command
  line, the refusals). 69 Python tests with torch on the CPU (the GPU test skipped), 56 without.
