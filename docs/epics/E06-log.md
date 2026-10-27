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

## T3b — The evaluation gate (2026-10-20)

- **Cut:** offered T3b in three pieces (the gate as a tool first); the user: "no need to cut it".
- **Built** (design §1 to §4, §6, §7; ADR-0026 points 6, 7, 9): `h3env.train.gate` (`Sprt`: the
  generalized SPRT of design §6 on the five pair scores; `play_gate`: two batches of the gate set
  v1, the candidate attacking in one and the best in the other, both seats external, each network
  on its own rows (`evaluate.duel`, its draws seeded alike at every gate), pairs in order until
  the test decides, the first min(500, `max_pairs`) envs only; `gate`; `python -m
  h3env.train.gate --candidate --best`). In the run (`ppo._go`): best #0 (the untrained network,
  or the parent's with `--init-from`), a gate every `gate.gate_every` samples (10 M; 0: only at
  the end) and at the end if the learner trained since its last, a promotion's
  `promoted/<n>.pt` and `best.pt` (the network with its index, how, run and parent), its ladder
  against `ladder.ladder_opponents` and the `ladder_promoted` networks before it (both seats
  external) on `ladder_episodes` (250) per kind, the final best's on `ladder_final_episodes`
  (500). `[gate]` and `[ladder]` replace `[evaluation]` (E05's evaluation against Random on v0 is
  gone; v0 stays a held-out set and a ladder option).
- **Resumable** (design §3): checkpoint format 2 (the promoted entries, the next gate, the last
  gate and final ladder, `pending`), saved before and after each gate and ladder; Ctrl+C stops
  either at its next batch call (`evaluate.Stopped`), and a resume runs it first (the gate on the
  saved learner, the ladder on its network read from `promoted/`; the lineage stays outside the
  checkpoint). Format 1 is refused with a pointer to `--init-from`. A replace Windows refuses
  five times keeps the old file and warns (`run.atomic_write`, a ladder reading `best.pt`).
- **Record** schema 4: `parent`, `gates`, `promotions`, `ladders`, `best` (the latest
  promotion), the curve per gate interval, `progress.evaluation_minutes` (the gates' and
  ladders' share of the run). `python/configs/e06-t3b.toml`: the user's hour with gates (30 M
  samples, a gate every 5 M).
- **Tests:** `test_gate.py` (new: the scores and Elo, the SPRT's bounds and formula, decisions in
  pair order, the cap; its error rates by simulation, 200 gates at either hypothesis, at least
  180 right, 300 to 900 pairs on average, +100 Elo under 150; a network against itself on the
  gate set not promoted, reproducible); `test_run.py` (the new keys and their refusals, the
  record of gates, promotions and ladders, `--init-from`'s command line, the printed lines, a
  refused replace keeping its file); `test_train.py` (a run's gate and ladders, a stand-in gate
  that promotes each time: every promoted network kept, each ladder against the one before; the
  end gating only after training; a gate and a ladder cut short by Ctrl+C run first at the
  resume; `--init-from`, and a network of another size refused). 87 Python tests with torch on
  the CPU (the GPU test skipped), 69 without; the scoped verify (Python only: the engine code is
  #324's, verified there).
- **A real run in the sandbox** (the CLI, the default network, `samples=1`, 64 envs, ladders of
  10 per kind, CPU): best #0's ladder (16 s), one update, the end gate rejected the learner after
  150 pairs (140 of them ½: each side's attacker won its game; LLR −3.36) in 353 s, the final
  ladder repeated best #0's rates exactly (common random numbers). The gate played 5 006 games
  for its 150 pairs: while it waits for the slowest battle among the pairs it needs, the other
  envs of its 500-env batches play on into later episodes. On the CPU the network's passes over
  those rows dominate; the user's hour on the GPU measures what it costs there (the lane's
  inbox).

## T3b — The hour with gates on the user's machine (2026-10-20)

- **The run** `t3b-1` (`python/records/t3b-1.json`; `configs/e06-t3b.toml`, commit `c6797b7`,
  GTX 1070, Windows, 24 threads, 256 envs): 30 M samples in 124.3 minutes, 1 164 updates.
- **Evaluation share 21.4 %** (26.6 minutes): 6 gates 7.7 minutes (18 s to 162 s, 25 to 874
  pairs, 4 153 to 7 836 games each), 6 ladders 18.9 minutes (50 s to 391 s, the later ones
  slower for the same games: longer battles). Training 97.7 minutes, about 5 100 samples/s (the
  curve 4 400 to 5 900), below E05's 8 500 (T5's batch size and threads).
- **Gates:** promoted at 5 M (25 pairs, +288 Elo), 10 M (175, +20), 15 M (322, +16), 25 M (874,
  +12); rejected at 20 M (207 pairs, +2) and at the end (193, +1). Between 85 and 92 % of the
  pairs score ½ (each side's attacker won its game, or truncations), so a gate decides in a few
  hundred pairs; its uncounted games cost little on the GPU (the inbox item closes).
- **Ladders** (the report set; final: 500 per kind): Random 0.503 (#0), 0.800, 0.811, 0.794,
  0.783 (#4), final 0.792 ± 0.014; Aggressor 0.195, 0.500, 0.521, 0.528, 0.524, final 0.527 ±
  0.018; Greedy 0.175, 0.465, 0.494, 0.497, 0.507, final 0.505 ± 0.018; the promoted network
  before: 0.800 (#1 against #0), 0.521, 0.499, 0.485, final 0.477 ± 0.019 (#4 against #3, value
  +0.014). The agent reaches the bots' level by 10 M samples and stays there (the bots' table:
  Greedy 0.832, Aggressor 0.810 against Random; Greedy 0.533 against Aggressor).
- **Stalling** (AI-Q9): self-play's truncations rose from 0 to 11.9 % of the episodes, the
  attacker's wins from 0.73 to 0.62 down, the learner won 0.46 against its own frozen past, the
  entropy fell to 2.29 by 10 M and rose to 2.93. Reward v1 at the end only and discounted (γ
  0.995) lets the losing side gain by delaying; a truncation counts as no win for either side in
  the ladders, so #4's 0.477 against #3 can be even. The record keeps each ladder opponent's
  truncations since (`Record`). Decided at T4's refinement.


## Increment 3's refinement (2026-10-20, AI-Q9)

- The agent proposed T4a (the league) in one pull request, then T4b, and for AI-Q9 an A/B hour
  on T3b's config at γ 1.0 against `t3b-1` while T4a was built (GAE checked first: a truncation
  bootstraps from the final value, no advantage crosses an episode, so the stalling is the
  incentive, not a bug). **User:** two league runs in T4b, γ 1.0 beside 0.995; T4a in one pull
  request; 60 M samples a run (extendable on resume).

## T4a — The league (2026-10-20)

- **`h3env.train.league`** (torch-free): `Member` (id, kind `snapshot` or `promoted`, file,
  birth, the decayed score and games, the plain games), `League` (`join` with the retirement,
  `draw`, `opponent`, `side`, `settle`, `table`, `state`, `restore`). The trainer holds the
  members' networks (`Trainer.nets`; a promoted member shares the lineage's copy) and per env its
  episode's member (`frozen_id`); `join` replaces `snapshot`, every promotion joins (best #0 from
  the start). An episode's result counts for its member by the gate's `game_score` (a draw or a
  truncation ½), settled once per update; a retired member's results are dropped.
- **One change from design §8's outline:** the live set is drawn every `live_every` samples
  (250 000) and when it is stale (a retired member, fewer than it could hold), not at every
  rollout: an episode outlasts a rollout (`t3b-1`: about 147 calls against 128) and keeps its
  opponent, so the sandbox run below, drawing at every rollout with `live` 2, passed up to 3.9
  frozen networks a call (a rollout's mean; retired members' too, while their episodes ran). The update record's `frozen_passes` measures it for T4b.
- **Config:** `[league]` (`self_play` 0.5, `snapshot_every` 1 M, `max_members` 16, `live` 4,
  `live_every` 250 000, `pfsp_power` 2, `decay` 0.99, `retire_games` 200) replaces `[pool]`;
  `configs/e06-t4.toml` (60 M samples, a gate every 10 M, ladders of 250 per kind) replaces
  `pool-smoke.toml` and `e06-t3b.toml` (their keys are gone; `t3b-1`'s record keeps its resolved
  config and commit).
- **Checkpoint format 3:** the league's state (members, next id, live set, next draw, its
  generator) instead of the pool's networks; the networks come from the members' files at the
  resume (`league/<id>.pt`, `promoted/<n>.pt`). A retired snapshot's file is deleted after the
  next checkpoint (the last one still names it); a resume removes the files of `league/` its
  checkpoint does not name. Formats 1 and 2 are refused (`--init-from` their `best.pt`).
- **Record schema 5:** each `update` record's `league`, `live`, `drawn`, `league_episodes`,
  `league_score` (the learner's mean score against members, a truncation ½: the pool's
  `pool_won` counted wins only, which hid even matches in AI-Q9's reading), `frozen_passes`;
  `snapshot` and `promotion` records name the member and the one it retired; each gate's member
  table (the record's `gates[i].league`); the curve's `league_score` and `frozen_passes`.
  `check_record` still accepts schema 4 (`OLDER_TABLES`: `t3b-1`'s `[pool]`).
- **Tests:** `test_run.py` (the league's weights, scores, decay, draws and their cadence, the
  episodes' opponents and sides, the retirement with and without enough games, the state round
  trip, the refused `[pool]`); `test_train.py` (the league in a short run with its files, the
  checkpoint's league and its networks restored from the files, a member playing its side,
  retirement with the files deleted after the checkpoint, a stray file removed at the resume).
  90 Python tests with torch on the CPU (the GPU test skipped), 71 without.
- **A real run in the sandbox** (the CLI, `configs/e06-t4.toml` with the default network, 64
  envs, a snapshot every 5 000 samples, 3 members, `live` 2 drawn at every rollout, no games
  needed to retire, a 50-pair gate at the end, ladders of 4 per kind; CPU): 5 updates (about
  950 samples/s), Ctrl+C, saved; `--resume` to the budget (10 updates, 64 487 samples) in a
  second segment. Snapshots 1 to 10 joined, eight members retired in turn (best #0 among them:
  no games were needed), `league/` kept only the members' files (8, 9, 10), the record passed `check_record` with both segments; the end gate was
  inconclusive at its cap (50 pairs, +63 Elo).

## T4b — The first league runs on the user's machine (2026-10-21 to 22)

- **The runs** (`python/records/t4-1.json`, `t4-2.json`; `configs/e06-t4.toml`, seed 1, 60 M
  samples, GTX 1070, Windows, 24 threads, 256 envs): `t4-1` at γ 0.995 (commit `3b0d79f`, 222.4
  minutes, one segment), `t4-2` at γ 1.0 (`--set ppo.gamma=1.0`, commits `a6b6c55` and
  `357f7c6`, 232.8 minutes; stopped by Ctrl+C at 52.9 M samples after 209 minutes, resumed the
  next morning to the budget, 0 records dropped: exit criterion 2 used for real). Between the two
  commits the trainer, the encounters and the rulesets' content hashes are the same; the engine
  moved by the battle lane's E23 T3c to T4b (the computer's choices, which `human = [true,
  true]` leaves out, and an Ogre Mage's mass Bloodlust on Magic Plains offered without a target).
- **AI-Q9's answer: γ 1.0 ends the stalling.** Self-play's truncations per 10 M samples: γ 0.995
  1.8, 9.3, 12.4, 12.6, 13.2, 13.2 % (as `t3b-1`); γ 1.0 11.7, 10.1, 2.2, then 0 (the winner had
  no reason to hurry while the policy was loose; the truncations went once it firmed up, entropy
  3.3 to 1.5, γ 0.995's stayed at 3.0). The ladders' truncations: 4 % against the bots and 9 %
  against the promoted network at γ 0.995, none from 30 M on at γ 1.0. A battle took 253 batch
  calls at γ 0.995, 108 at γ 1.0 (`t3b-1`: 147); the attacker won 0.61 and 0.69 of self-play.
- **Ladders** (final, 500 per kind): γ 0.995's #4 Random 0.788 ± 0.015 (value +0.251),
  Aggressor 0.540 ± 0.018 (+0.037), Greedy 0.526 ± 0.018 (+0.022), #3 0.465 (+0.018); γ 1.0's
  #5 0.839 ± 0.013 (+0.281), 0.578 ± 0.018 (+0.054), 0.559 ± 0.018 (+0.039), #4 0.524 (+0.016).
  With γ 0.995's truncations counted ½, γ 1.0 still leads by 3, 2 and 1 points, and on every
  value. Both rose slowly to 50 M (γ 1.0 against Greedy: 0.422, 0.487, 0.517, 0.533, 0.561); both
  last gates rejected (−7 and −2 Elo) while the learning rate decayed to its floor (5 %) at the
  budget. Promotions: 4 of 6 gates at γ 0.995, 5 of 6 at γ 1.0. The targets (decision 7): 0.95,
  0.75, 0.75 not met; a positive value against every ladder member met by both.
- **The league:** at γ 1.0 a call passed 4.0 to 7.0 frozen networks, 4.4 at the end (the live set
  4); at γ 0.995 up to 8.1 (stalled episodes keep earlier sets' members). Every member's `p` lay
  between 0.45 and 0.58 after the first gate (the learner scores about ½ against its recent past),
  so PFSP's weights were nearly uniform; the untrained best #0 and the first snapshots retired by
  20 M, the members spanned the last 15 to 30 M samples.
- **Costs:** evaluation 18.7 % of `t4-1` (ladders of 321 s to 518 s: long battles) and 11.6 % of
  `t4-2` (late gates 14 s to 19 s, ladders 50 s to 76 s), as design §9 estimated for a gate every
  10 M. Training rate 5 530 samples/s at γ 0.995, 4 860 at γ 1.0 (about 4 000 from 20 M to 40 M,
  then 5 200; not explained: T5's profile).
- **Increment 3 closed** at the checkpoint (2026-10-22): the user ended the session before
  increment 4's refinement; the next session puts it to the user (the QUEUE's note).

## Increment 4's refinement (2026-10-22)

- The agent put T5's start to the user: the entity network (its recommendation), AI-Q7 (b),
  throughput, or T7 (E23's neutral side being complete); and a spare MLP run from `t4-2`'s best.
  **User:** the entity network, with a brief comparison of architectures; T5a now (the network,
  its config, tests, a CPU cost check; one pull request), T5b later (the user's 60 M A/B run
  against `t4-2`); no spare run; network visualization tooling soon.
- **The comparison** (forward multiply-adds per sample on v21, the agent's estimates): the MLP
  (≈ 6 M; each stack slot and hex learned apart, a creature's handle a number, 1 404 entry
  outputs each its own); an entity transformer over the stacks with the globals, the own hero
  and the field as tokens (≈ 14 M; shared weights, order-free slots, relations between stacks,
  pointer heads); the same with the 255 hexes as tokens (≈ 125 M: too slow on the GTX 1070); a
  CNN on the hex grid (20 to 60 M; the offset rows, the stacks' 99 features as planes, the
  turrets off the field); a graph network on the hexes' adjacency (the engine's geometry in
  Python); a recurrent memory (v21 is complete; sequence minibatches in PPO).
- **User:** the entity transformer as proposed (design §11: two layers of width 128, four heads,
  pointer heads; `[network] kind`, the MLP the default kind, γ 1.0 the default), the
  visualization in T5a (dependency-free: the summary table, the cost, a Mermaid diagram; no
  torchview or Graphviz, whose EPL is a licence doubt).

## T5a — The entity network (2026-10-22)

- **One interface** (`h3env.train.policy.Policy`): a kind implements `encode` (a tuple of tensors,
  rows first), `value` and the four heads' logits; `act`, `evaluate`, `values` and `frozen` are
  shared. `Mlp` is E05's network unchanged (its weights' names, so `t4-2`'s files load), `Entity`
  the network of design §11 (`h3env.train.entity`), `build(config, batch)` the one place a
  network is made: the trainer, `evaluate.load` (the league's and lineage's files, the ladder, the
  gate's command) and `--init-from` build the kind a saved file's config names (none: an MLP).
- **The layout** (`h3env.train.layout`): v21's sizes, the stack columns read (present, creature,
  x, y, active) and action v13's bases. `h3-sim`'s two tests `the_python_layout_is_the_encodings`
  read its literals (`python_layout`, test-only) against `observation.rs` and `action.rs` (a
  wrong column fails them); `test_entity.py` checks the derived values against the library's
  sizes and on real battles (the legal attacks, shots and heals name present stacks, at most one
  active). One change from §11's outline: a stack counts as active only where it is present.
- **Config:** `[network] kind` (`"mlp"` the default), `width`, `layers`, `heads` (`embed` both
  kinds'); γ 1.0 the default (AI-Q9); `--init-from` a network of another kind is refused before
  its weights are read. No checkpoint format or record schema change: a file's config names its
  kind, a record gains only keys. `configs/e06-t5.toml`: `t4-2`'s settings with the entity
  network.
- **The visualization** (`python -m h3env.train.network`; no new dependency): parameters, output
  shape and multiply-adds per part (a cast's heads apart), `--time` per decision, `--mermaid`
  writes the table and the data flow.
- **Costs** (the defaults, 256 observations of the encounter distribution; the sandbox's 4 CPU
  threads, the median of 30): the MLP 5.95 M parameters, 5.53 M multiply-adds per decision (a
  cast's heads 0.40 M more), forward 82 µs, a training pass 159 µs; the entity network 1.19 M
  parameters, 15.5 M (2.6 M more a cast; the two blocks 12.8 M), 398 µs and 873 µs, about five
  times the MLP on this CPU (many small kernels). The GPU decides the run's samples per second:
  T5b starts with `--time --device cuda` on the user's machine.
- **Tests:** `test_entity.py` (the layout against the library, the logits in the encodings'
  order, an empty slot unseen, the context free of the slots' order, other encodings refused, the
  visualization's numbers and files); `test_train.py` (the choices' masks and recomputed
  log-probabilities for both kinds, an entity run stopped, resumed and laddered, `--init-from`
  across kinds refused). 97 Python tests with torch on the CPU (the GPU test skipped), 71
  without.

## T5b — The entity network's run, stopped (2026-10-22)

- **`t5-1`** (`python/records/t5-1.json`; `configs/e06-t5.toml`: `t4-2`'s settings with the
  entity network; GTX 1070; the user's GPU timing first: forward 37.6 µs, a training pass 119 µs
  per decision at 256 rows): stopped by the user at 26.6 M of 60 M samples (275 minutes).
- **Against `t4-2` at 10 M and 20 M:** 1 831 and 1 643 samples/s (`t4-2`: 6 673 and 5 328);
  self-play truncations 42.8 % and 44.9 % (11.7 %, 10.1 %); explained variance 0.82, 0.46, then
  0.02 from 20 M on (0.87); gates promoted (+137 Elo) and rejected (−14) (`t4-2`: +233, +43); the
  10 M ladder 0.607, 0.383, 0.372 with 20 % truncated (0.719, 0.440, 0.422, 8 %); the learner's
  score against its league 0.39 at the end.
- **The diagnosis** (the run's `metrics.jsonl`, `best.pt` and `checkpoint.pt` in the sandbox):
  truncations rose from 18 % in the first million samples to 45 % in the second and stayed. In
  self-play with either network 62 of 128 battles truncated (hero battles 39 of 74, sieges with a
  hero 19 of 36, garrisons' 4 of 18); in them 86 to 95 % of the choices were moves, the rest
  waits, no attack, shot, defence or cast in 2 000 steps; at the cut the attackers stood at column
  1.5 on average, the defenders at 14.5, the closest opposing stacks 11 columns apart, 8 % of the
  moves ending closer to the enemy. Both sides hide at their own edge: who approaches is struck
  first, and a truncation bootstrapped from a value of about 0 costs nothing. Such a battle lasts
  2 000 steps against about 100, so about 94 % of the samples came from battles where nothing
  happens (their advantages about 0: explained variance 0, the regression); and their long
  episodes kept members live (8 frozen passes a call). The context layer was alive (126 to 128
  of 128 units on); the cost: a PPO update about 10 s and a rollout about 5 s for 25 000 samples.
  The MLP escaped the same incentive (`t4-2`'s truncations went to 0): its move logits read
  every stack's position, the entity network's a hex's own 16 features and the context, which
  makes "to my edge" easy and "toward that stack" hard.

## Increment 4's revision, T5c (2026-10-22)

- The agent proposed relative geometry for the hexes (the stacks on their hexes, distances).
  **User:** exploration should find the fights (the agent: a random step forward is undone by the
  next; an approach exploration finds is mostly punished, as the approacher is struck first; the
  entropy of 3.3 nats spreads over the safe hexes, not toward the enemy); no feature engineering
  for so simple a problem, but a penalty for stalling to the step limit (or a cost per step); the
  neutral share (T7) will force fights anyway, as the system actor does not cower. The agent
  chose the penalty at the limit: a cost per step would weigh on every decided battle and could
  drive a losing side to attack only to end it sooner.
- **T5c:** `[ppo] truncation_penalty` (1.0, the default; decision 10 amended): at a truncation
  each learning side's last transition gets −1.0 and ends there, no bootstrap from
  `final_obs`. Reward v1 lies in about (−1, 1), so stalling to the limit is worse than any
  decided battle. Reward v1 itself, the engine and the evaluation (a truncation is no win) are
  unchanged. `configs/e06-t4.toml` and `e06-t5.toml` state 0.0 (their runs reproducible),
  `configs/e06-t5c.toml` the next runs. Tests: both cases in a rollout of battles cut after 6
  steps.

## T5d — The runs with the truncation penalty (2026-10-22 to 23)

- **The runs** (`python/records/t5-2.json`, `t5-3.json`; `configs/e06-t5c.toml`: `t4-2`'s
  settings with the penalty 1.0 and 20 M samples, the learning rate's horizon 20 M): `t5-2`, the
  entity network (250 minutes in three segments, two of them resumed; 1 463 to 1 502 samples/s),
  and `t5-3`, the MLP (70 minutes; 4 914 to 5 433 samples/s).
- **The penalty ends the stalling:** self-play truncations 2.8 % and 0.4 % (`t5-2`, the two
  halves; `t5-1`: 45 %), 0.001 % (`t5-3`; `t4-2`'s first 20 M: 11.7 %, 10.1 %).
- **Results** (ladders on the report set; at 10 M 250 per kind, the finals 500):

  | Random / Aggressor / Greedy | 10 M | 20 M (final) |
  |---|---|---|
  | `t4-2` (MLP, no penalty, 60 M horizon) | 0.719 / 0.440 / 0.422 | 0.790 / 0.500 / 0.487 (250 per kind) |
  | `t5-2` (entity + penalty) | 0.771 / 0.461 / 0.452 | 0.770 / 0.470 / 0.468 (best #1, from 10 M) |
  | `t5-3` (MLP + penalty) | 0.821 / 0.535 / 0.511 | **0.832 / 0.557 / 0.533** (best #2) |

  Gates at 10 M and 20 M: `t5-2` +243 Elo promoted, −8 rejected; `t5-3` +330 and +14, both
  promoted. `t4-2`'s final (best #5, 50 M, 233 minutes): 0.839, 0.578, 0.559.
- **The entity network is parked:** with the same penalty and schedule the MLP led from 10 M on
  and kept improving while the entity network's gate rejected, at 3.4 times the MLP's speed; the
  entity network's entropy barely fell (3.44 to 3.11; the MLP's 2.74 to 1.89), its explained
  variance stayed at 0.52 (0.78). Its lead over `t4-2` at 10 M was the penalty's.
- **The penalty (with the 20 M schedule) is the gain:** `t5-3` came near `t4-2`'s 60 M level in a
  third of the samples and of the time; the shorter horizon's faster decay may share in it (a 60 M
  run with the penalty separates the two). The targets (0.95, 0.75, 0.75) are not met; a positive
  value against every ladder member: `t5-3`'s best yes (+0.272, +0.030, +0.008, +0.022 against
  #1), `t5-2`'s no (−0.032, −0.043 against the bots).
- **Increment 4 closed** at the checkpoint (2026-10-23): the user ran nothing further and closed
  the session; increment 5's refinement is the next session's (the QUEUE's choices).


## Increment 5's refinement (2026-10-23)

- The agent put the choices to the user (the QUEUE's): a run (the MLP with the penalty for 60 M,
  its recommendation, or one from `t5-3`'s best) and the code (T7, its recommendation; AI-Q7
  (b); throughput; the mana price). **User:** T7, no run meanwhile; cut at a seam: T7a (the
  `shared/` dependency) and T7b (the system actor's seat) now, T7c (the trainer, the neutral
  held-out set, the ladder) and T7d (the run) after the checkpoint.
- **User:** the neutral kinds' weights live in the run's config (`[env] kinds`), not in the
  distribution's defaults: the held-out sets keep their episodes and the records stay
  comparable (design §12). The `shared/` change gets its own branch (`shared/e06-t7a`).

## T7a, T7b — The system actor's seat (2026-10-23)

- **T7a** (#420): `arch.toml` lets `h3-sim` depend on `h3-battle-ai` (ADR-0025
  point 5 named `System` among the policies; ADR-0027 point 11 the batch with neutral `System`
  seats); the sim doc's neutral kinds (`docs/07-simulation-and-ai.md`).
- **T7b:** `Policy::System` (`h3_battle_ai::choose` from the seat's stream); the neutral kinds'
  defender `Internal(System)` (was `Random`); `h3_sim::SYSTEM_POLICY_VERSION` (15); C ABI v32:
  `H3_SEAT_SYSTEM` (4), `h3_system_policy_version()`; `h3env`: the seat `"system"`,
  `versions()["system_policy"]` in a new record's identity (the schema stays 5); `h3 ai eval
  --policies` names `system`, its default list and the bots' table unchanged (T7c). The
  distribution's defaults and the held-out sets are unchanged, so is every gated workload (the
  encounter workload draws the default kinds).
- **Tests:** the batch's determinism property plays the system actor on either side of the four
  synthetic scenarios (a hero's side with its spells, a siege); the encounter test checks each
  kind's seats (the neutral kinds' defender the system actor); the C ABI's internal seats
  (`H3_SEAT_SYSTEM` as a defender, refused as the second internal seat, the version); `h3 ai
  eval --policies random,system`; Python: the seat on either side of a scenario.
- **Cost** (`h3 ai bench`, 64 envs, one thread, 2 000 calls of random legal actions, the
  sandbox's CPU): the neutral kinds alone (`--encounters 780,20,0,0,0,25`) 56 154 caller steps/s
  inside the step with the random defender, 15 349 with the system actor (2 089 episodes against
  979: it beats a random attacker fast); about 47 µs per system decision. The default kinds
  unchanged (40 186 and 43 887, noise). The engine stays far above the trainer's 5 000 samples/s.

## T7c — The trainer's neutral share (2026-10-23)

- **The user:** T7c in one pull request (as queued; the agent had proposed the evaluation and
  the trainer as two).
- **The trainer:** `Trainer.assign` draws no league member for a neutral kind's episode (its
  defender is the system actor inside the batch; before, a member took one side: the defender's
  never moved but scored, the attacker's left nothing to train). The update record's `by_kind`
  (per kind the self-play episodes, the attacker's wins, the truncations), the record's curve
  summed per gate interval, the update line's per-kind rates. `[ladder] ladder_neutral` (true):
  every ladder also plays the neutral set v1, the ladder record's `neutral`. `ladder_opponents`'
  default gains `system`. The gate stays on the gate set v1. `configs/e06-t7.toml`: `t5-3`'s
  settings at the census shares.
- **The neutral set v1:** strata wandering, guards, siege_neutral (`NEUTRAL_V1_SEED + k`,
  `0x5EED_E7A1_0003_0000`), 500 envs each, weighed 780 : 20 : 25 (the census's `share`, pinned
  by a Python test against `encounters-v1.toml`), the agent attacking the system actor; in
  `evaluate.py` (`report(held_out=…)`, `stratified`, `ladder`, the held-out seeds: 8 batch
  seeds, 8 000 episodes), `h3 ai eval --set neutral-v1` (a fixed defender: every policy
  attacks it, itself too; no ratings) and `python -m h3env.train.ladder --set neutral-v1`.
- **`h3 ai eval`:** the system actor in the default list; `--json` schema 2 (`defender`, the
  engine's `system_policy`); both tables regenerated (`bots-report-v1.json`, new
  `bots-neutral-v1.json`); the old bots' rows did not move (only the ratings, with more games).
- **The bots' tables** (release, the sandbox: 84 s and 14.5 s; `POLICY_VERSION` 18, E23 T6
  merged first: the tactics phase, the tent, the Catapult; its retreat needs facts the encounters
  leave at their defaults, so the system actor never retreats here). Report set v1, row against
  column:

  | | Random | Aggressor | Greedy | System |
  |---|---|---|---|---|
  | Aggressor | 0.810 | – | 0.467 | 0.322 |
  | Greedy | 0.832 | 0.533 | – | 0.340 |
  | System | 0.875 | 0.681 | 0.653 | – |

  The system actor's value efficiency +0.324, +0.140, +0.115 against Random, Aggressor, Greedy;
  Glicko-2 1709 (Greedy 1561, Aggressor 1529, Random 1201). Decision 7's 0.75 targets against
  Aggressor and Greedy now stand beside a bot at 0.68 and 0.65 (`t5-3`: 0.557, 0.533).
  Neutral set v1, attacking the system actor: Random 0.743 ± 0.036 (value −0.056), Aggressor
  0.967 ± 0.014 (+0.216), Greedy 0.956 ± 0.016 (+0.223), the system actor 0.991 ± 0.006
  (+0.285). The neutral target is a value efficiency above +0.285, the system actor's own (its hero
  casts since E23 T5a; it retreats only with the adventure's facts, which the encounters leave at
  their defaults).
- **Throughput** (`h3 ai bench`, 64 envs, one thread, 2 000 calls, the sandbox): the default
  kinds 43 111 caller steps/s inside the step, the census shares 20 872, a third neutral
  (`78,2,100,50,25,3`) 33 888; every mix far above the trainer's 5 000 samples/s. The user's
  machine: `cargo run --release -p h3-cli -- ai bench --encounters 780,20,100,50,25,25 --envs 256
  --threads 24` and the run itself (the update line's samples/s).
- **Tests:** `h3-cli`: the neutral strata, the fixed defender's matchups, rates, interval, JSON
  and command line, the constants pinned to Python; Python: the neutral set's batches and
  shares, its stratified rates, Random against the system actor, both tables against the engine,
  every held-out seed, the record's `by_kind` and `neutral`, the console lines; with torch (the
  sandbox reached PyTorch's index): a neutral episode draws no member while a hero battle does,
  the rates by kind, the ladder's neutral row, the ladder command on `--set neutral-v1`.

## T7d's mix; T5's rest beside it: the mana price's preparation (2026-10-23)

- **The user:** T7d at a third neutral (`[78, 2, 100, 50, 25, 3]`: the census's neutral shares a
  tenth beside the default 4 : 2 : 1; `configs/e06-t7.toml` now says so); in the sandbox
  meanwhile the mana price's preparation, then AI-Q7 (b)'s batch output (its first part).
- **The evaluation at reward v1's settings:** a run's `[env] currency` and `mana_price` now shape
  only the training's reward. The ladders score their value efficiency at reward v1's
  (`evaluate.CURRENCY`, `MANA_PRICE`: `ai_value`, 10), as `h3 ai eval` scores the bots' tables;
  the gate reads results only (`play_gate` takes no reward settings). Before, the ladders scored
  at the run's own settings, so a run at another price could not compare its values with the
  others' (every record so far trained at v1's: nothing recorded moves).
- **`configs/e06-mana.toml`:** `t5-3`'s settings with the training's price at 0 (every cast free),
  the run `t5-4`; another price by `--set env.mana_price=…`. The user's, after T7d.
- **Test:** a trainer at `gold` and price 1 000 ladders as a run at v1's settings (and the
  ladder at its own settings scores differently).

## T5's rest: AI-Q7 (b) part 1, the batch's kill preview (2026-10-23)

- **The output:** `Outputs::preview` (`h3_sim::batch`), an option like `final_obs`: per env
  `PREVIEW_LEN` (2 184) values, the least and the greatest kills of each attack and shot entry
  from `PREVIEW_BASE` (257, the first attack entry; 1 092 entries), the engine's preview of the
  entry's first strike (`h3_battle::preview`, BAT-DMG-011: what the original shows a player
  before the strike; never the retaliation), 0 for an entry not legal or without a side to move
  (`action::preview_kills`, `BattleEnv::preview_kills`). The encodings are unchanged.
- **C ABI v33:** `H3BatchBuffers.preview` (the struct grows by one pointer at its end),
  `h3_preview_base`, `h3_preview_len`; `h3env.BattleBatch(…, preview=True)` writes
  `batch.preview` [N, 1 092, 2] (`preview_base`, `preview_entries`; `_ffi.preview()` pinned to
  `layout.py`'s attack and heal bases); without it `None`, nothing computed.
- **Cost** (`h3 ai bench --preview true`, 64 envs, one thread, 2 000 calls, the sandbox): the
  default kinds 39 623 caller steps/s inside the step without, 18 487 with the preview; T7d's mix
  (`78,2,100,50,25,3`) 31 497 and 16 905: about 29 µs a step more, as many as the step itself
  (`preview` validates each command again, a path search each). Still far above the trainer's
  5 000 samples/s; a preview of every attack that shares the reach (a `h3-battle` function, the
  battle lane's) only if a profile of training shows it.
- **Tests:** the batch's preview against the engine's of every legal attack and shot entry in the
  batch-against-single-envs property (any thread count), both kinds of strike shown, the short
  array refused; the ABI's preview against the Rust batch's; the C smoke test (the legal
  strikes only, least ≤ greatest); `h3 ai bench --preview` plays as without; Python: the binding's
  preview on the legal strikes only, the same play with and without.

## T7d — The run at a third neutral (2026-10-23)

- **The run** (`python/records/t7-1.json`; `configs/e06-t7.toml`: `t5-3`'s settings with the
  kinds `[78, 2, 100, 50, 25, 3]`; commit 639ab77, C ABI v33, the system actor's policy v18; 20 M
  samples in 79 minutes, one segment, 4 846 to 5 205 samples/s): gates at 10 M (+245 Elo) and
  20 M (+24 Elo), both promoted; truncations 0.4 % and 0.001 % of self-play.
- **The mix in episodes:** the neutral kinds 32 % of the draws but 49 % of the training episodes
  (shorter battles, one learning side); their attackers won 0.974 (wandering), 0.972 (guards),
  0.857 (a neutral town) in the second half, self-play's attacker 0.711 (hero), 0.615 (a siege a
  hero defends), 0.940 (a garrison).
- **Results** (the ladders; at 10 M and 20 M 250 per kind, the final 500):

  | | Random | Aggressor | Greedy | System | Neutral set (value) |
  |---|---|---|---|---|---|
  | 10 M (#1) | 0.809 | 0.512 | 0.499 | 0.359 | 0.975 (+0.241) |
  | 20 M (#2), final | 0.828 | 0.551 | 0.528 | 0.374 | 0.972 (+0.245) |
  | `t5-3` final | 0.832 | 0.557 | 0.533 | – | – |

  The value efficiency against Random, Aggressor, Greedy, the system actor: +0.273, +0.029,
  +0.010, −0.101 (positive against every ladder member but the system actor); on the neutral
  set the strata 0.976 (wandering), 0.966 (guards), 0.852 (a neutral town).
- **Read:** a third neutral costs the player-against-player kinds nothing at 20 M (within the
  intervals, ±0.013 to ±0.018, of `t5-3`; a little behind at 10 M). On the neutral set the agent
  attacks better than every heuristic bot (Greedy +0.223, Aggressor +0.216) but not as well as
  the system actor (+0.285): decision 7's neutral target is not met, and the value stayed flat
  from 10 M. The system actor is the ladder's gap: it beats this agent 0.626; the 0.75 targets
  against Aggressor and Greedy are as far as before.
- **Evaluation:** 12.2 of 79.3 minutes (15.3 %; `t5-3` 6.8 %): the system actor's row and the
  neutral set (ladders 93 s, 137 s, 138 s, the final 220 s; gates 13 s, 128 s).
- **Increment 5 closed** at the checkpoint (2026-10-23). Put to the user for increment 6: AI-Q7
  (b) part 2 (the agent's recommendation) or the system actor as a league opponent in the hero
  kinds (a change of decision 9); a run of 60 M at T7's mix (the recommendation: whether PPO
  plateaus) or the mana run `t5-4`. **User:** `t7-1` recorded, nothing built and no run for
  now; then (asked what lies on the critical path to beating the system actor): diagnose first,
  a behaviour analysis as T5b's (T6a, in a fresh session; the lane's QUEUE has its plan). Known
  so far: the agent plays at Greedy's level, the system actor beats Greedy 0.653 with an edge
  alike in every kind and seat (its hero casts, Greedy never; its waits, defends and targets);
  the agent never met the system actor in a hero battle (decision 9); more samples barely help
  (`t4-2` at 60 M against `t5-3` at 20 M).

## T6a — The behaviour analysis (2026-10-23)

- **The command:** `python -m h3env.train.inspect --checkpoint …` (`h3env.train.inspect`; Python
  only, any checkpoint): the sampled policy plays a stratified set as on the ladder
  (`evaluate.report` with `evaluate.sampled`, now the ladder's own agents: the same episodes and,
  on the same device, the same draws) against the system actor and Greedy (`--opponents`; the
  neutral set: the system actor) with the batch's kill preview on (the play is the same), and
  sorts every battle by kind, the agent's side and its outcome (lost: a truncation too). Per group
  (`--json` keeps them all, the console two tables per opponent): the decision mix (wait, defend,
  move, melee, shot, the hero's and a stack's casts, a commanded turret's, the tactics phase's,
  the rest), decisions and rounds per battle; the rounds the hero could cast in and the share it
  cast in, its spells (named by the packs' load order), both heroes' mana spent (the maximum less
  the last observation's; the encounters start each hero full); the choices when a shot was
  legal; each melee strike's and shot's previewed kills in hit points against the best legal
  strike's, strikes killing nothing while one could, strikes when one was legal; per *clean*
  strike (no opponent turn and no new round before the next observation) the hit points dealt
  and lost, melee into a target with retaliations left (and of them those while a target without
  was in reach), who struck first; the share of the stack turns that waited and defended, the
  agent's (its decisions) and the opponent's (its stacks' flags, the most seen per round).
  `layout.py` names the columns it reads; `h3-sim`'s layout test pins them (after a Wait and a
  Defend; a swapped pair fails it). The results: `python/records/inspect/t7-1-*.json`.
- **The reading checked:** a clean strike's target lost at least the least kills previewed in
  98.9 % (Random against Aggressor, 60 battles per kind and side; 350 clean strikes) and 100 %
  (against the system actor); the four misses were melee strikes on Vampire Lords, whose
  retaliation drains some back. The first misses were a commanded turret's shots (a hero with
  Artillery commands its towers, BAT-SIE-016): **the engine's preview of a turret's shot is an
  ordinary strike's, the shot itself BAT-SIE-018's** (`turret_shot`; previewed 3 to 6 kills, 1
  killed): in the battle lane's inbox; a turret's decisions are a category of their own here and
  stay out of the targeting and the trades. It matters for AI-Q7 (b) part 2 (the network would
  read it) and the Greedy bot (reads it for its turrets).
- **Tests:** without torch (`tests/test_inspect.py`): the battles, wins and rates are `report`'s,
  each decision in one category, the clean strikes against the preview, the neutral set, the
  observation's reading by hand, the spells' names; with torch: a checkpoint's analysis plays its
  ladder's draws, the command's tables and JSON, a held-out seed refused.
- **`t7-1`'s best** (the user's `best.pt`; the sandbox's CPU, 4 threads, about 8 minutes per
  set; report set 3 000 battles per opponent, neutral set 1 500): the rates as the record's
  within the device's draws (the system actor 0.371, Greedy 0.531, the neutral set 0.968 at
  +0.236; recorded 0.374, 0.528, 0.972 at +0.245). Against the system actor, Greedy in brackets:

  | | hero | siege, a hero defends | a garrison's siege |
  |---|---|---|---|
  | attacking | 0.55 (0.74) | 0.43 (0.64) | 0.92 (0.94) |
  | defending | 0.17 (0.32) | 0.25 (0.42) | 0.04 (0.10) |

- **The casts are not the gap:** the hero casts in 87 % of the rounds it could (88 against
  Greedy) and spends 33 mana a battle, 44 % of its maximum; the system actor's hero 31, 41 %.
  Magic Arrow, Ice Bolt and Lightning Bolt are 60 % of its casts, then Blind, Implosion, Curse,
  Weakness, Chain Lightning, Slow, Resurrection. On the neutral set it casts in 67 % (17 mana).
- **It plays the same against both opponents:** the mix (moves 27 and 24 %, melee 33 and 34,
  shots 12 and 14, the hero's casts 10 and 10, waits 2, defends 1, the tactics phase 11),
  shooting (it shoots at 88 % of the decisions with a legal shot, moves at 7 %), targeting and
  trades within 1 to 3 points; only the outcomes differ. It never met the system actor in these
  kinds (decision 9).
- **The system actor waits and defends more:** its creature stacks wait in 7 % of their turns
  (sieges 9 to 16 %) and defend in 3 % (up to 8 %); the agent's 3 % and 1 %; Greedy's none.
- **Targeting leaves kills:** 53 % of its strikes are the best previewed strike (kills in hit
  points; against Greedy 55 %); their previewed kills are 77 % of the best's; 12 % kill nothing
  while a strike could (18 to 27 % in the battles it lost attacking); it strikes at 73 % of the decisions with a legal
  strike. On the neutral set 72 %, 80 %, 9 %.
- **Trades:** 77 % of its melee strikes meet a retaliation, 11 % while a target without one was in
  reach; a clean melee strike deals 151 hit points and costs 53 (against Greedy 155 and 47), a
  shot deals 70 and costs 1. It strikes first in 41 % of the battles (Greedy 44 %; about half in
  a hero battle, 21 to 30 % attacking a siege).
- **The system actor's benchmark** (the battle lane's ask, ADR-0027 point 11, E23 exit criterion
  4; the user: in this pull request): `benches/ai/batch-system-neutral-v1.toml`, 64 envs of the
  neutral kinds at the census's shares, the system actor defending inside the call, 200 calls of
  random legal actions, counted inside `BattleBatch::step`: 6.81 G instructions for 12 800 caller
  steps (`POLICY_VERSION` 19; 6.60 G on 18), about 532 000 a step with the system actor's replies
  (the encounter workload's default kinds: about 185 000). Lane-only: `h3_battle_ai` moves it.
- **Read for increment 6** (put to the user): the casts' exploration and the mana price are not
  indicated. The agent loses mostly as the defender and does not adapt to the opponent that beats
  it, which waits and defends more than anything it trained against: the direct lever is meeting
  the system actor in training, in the hero kinds (a change of decision 9; the agent's
  recommendation, A/B against `t7-1`'s settings). Targeting leaves about a quarter of the
  previewed kills and an eighth of the strikes kill nothing: AI-Q7 (b) part 2 addresses it, once
  a turret's preview is right (the battle lane's inbox) or the turrets' entries are left out of
  the feature.

## Increment 6's refinement (2026-10-26)

- **The user's choice** (asked at T6a's checkpoint, the agent's recommendation): the system
  actor as a training opponent in the player kinds, as an A/B against `t7-1`'s settings
  (decision 9 amended); AI-Q7 (b) part 2 (now unblocked: the battle lane's #458 previews a
  commanded turret's shot as it is made) and potential-based shaping stay candidates. T6b the
  code, T6c the run on the user's machine.
- **The let-off** (the battle lane's question, E23 T7, BAT-QUIRK-27): **every side stays
  human**, the system actor's too (ADR-0025 point 6 unchanged); training and the held-out sets
  never play the computer side's escape from a bad-morale freeze (design §12).

## T6b — The system actor as an opponent in the player kinds (2026-10-26)

- **Built** (design §12): `[league] system_share` (a float in [0, 1], used in thousandths; 0 by
  default; above 0 it needs `[env] kinds` and a player kind, `resolve` refuses otherwise).
  `ppo.episode_entries` splits the kinds into up to four encounter entries: the neutral kinds'
  (weight 2000 · their sum), the player kinds' own (both seats external, (2000 − 2s) · their
  sum) and one per side with the system actor there (`s` · their sum each, `s` the share in
  thousandths), so every kind keeps its share of the episodes. With the share 0 the batch has
  `t7-1`'s one entry. `Trainer.system_side` maps an entry (`batch.scenario`) to the system
  actor's side; `assign` draws no member for such an episode (as for a neutral kind), and
  `collect` scores the learner there by its side (`system`: episodes and mean score, a draw or
  truncation ½), outside self-play's rates (`attacker_won`, `by_kind`). The record's curve sums
  it per gate interval (schema 5 unchanged: a key added); the update line prints
  `system-attacker`, `system-defender`. `configs/e06-t6b.toml`: `t7-1`'s settings with a
  quarter.
- **Agent's choices (change freely):** a fixed share, not a PFSP member (the batch draws the
  entry, and with it the seats, at its own reset); a uniform side (T6a: the agent loses mostly
  defending, but the system actor's attacking is as strong); a quarter for the A/B (of the
  player kinds' samples about a fifth: self-play gives two learning sides an episode, a member
  or the system actor one).
- **The system actor's cost** (the sandbox, 4 threads, 256 envs, random legal entries but the
  casts, inside `step`): in the player kinds alone a call takes 2.96 ms with both seats
  external, 36.3 ms with the system actor attacking, 29.7 ms defending (its hero's decisions;
  about 0.5 ms of CPU a caller step). At `t7-1`'s kinds a call takes 3.70 ms, at a quarter
  6.41 ms (the worst of 400 calls 7.9 ms and 44.5 ms). On the user's 24 threads that is about
  half a millisecond more a call against `t7-1`'s about 50 ms (the network's passes and
  updates): a few percent of the run, more where the worst calls bunch.
- **Tests:** `test_run.py` (the refusals, the curve's `system`, the line),
  `test_train.py` (`episode_entries` at `t7-1`'s kinds; a trainer at a half: the system actor
  on both sides, never to move, no member, no self-play rates, its score by side, an update).

## T6c — The A/B run `t6b-1`: the system actor as an opponent (2026-10-26)

- **The run** (`python/records/t6b-1.json`; `configs/e06-t6b.toml`: `t7-1`'s settings with
  `system_share` 0.25; commit 9215205, C ABI v33, the system actor's policy v19; 20 M samples in
  76.0 minutes, one segment, 5 025 to 5 507 samples/s, `t7-1` 4 846 to 5 205): gates at 10 M
  (+245 Elo) and 20 M (+28 Elo), both promoted; evaluation 12.0 minutes (`t7-1` 12.2). The
  system actor's cost did not show on the user's 24 threads.
- **Results** (the final ladder, 500 per kind; `t7-1`'s in brackets): Random 0.844 (0.828),
  Aggressor 0.539 (0.551), Greedy 0.526 (0.528), the system actor 0.371 (0.374; hero 0.350,
  siege a hero defends 0.360, a garrison's siege 0.477), the neutral set 0.978 at value +0.245
  (0.972, +0.245). Every difference lies within the intervals (±0.011 to ±0.018 each).
- **Against the system actor in training** (the curve's `system`, the learner's mean score):
  attacking 0.477 at 10 M, 0.549 at 20 M; defending 0.107, 0.149. On the ladder 0.327 at 10 M
  and 0.371 at 20 M; `t7-1`, which never met it there, 0.359 and 0.374.
- **Read:** negative. Meeting the system actor in a fifth of the player kinds' samples does not
  make the agent play better against it within 20 M samples; it gains against it at the pace it
  gains without meeting it. The gap is not the opponent mix. Three 20 M runs with different
  settings (`t5-3`, `t7-1`, `t6b-1`) end at the same ladder.
- **The behaviour analysis** (`python -m h3env.train.inspect` on the user's `best.pt`, as T6a;
  `python/records/inspect/t6b-1-*.json`; against the system actor, `t7-1`'s in brackets): the
  analysis's wins 0.394 (0.394); the hero casts in 91 % of the rounds it can (87 %), 33 mana
  (33); targeting 54 % at the best previewed kill (53 %), 77 % of the best's kills (77 %), 12 %
  killing nothing while one could (12 %); melee into a retaliation 77 % (77 %); clean melee
  strikes deal 152 and cost 53 hit points (151, 53). The only shift: its stacks defend in
  2.3 % of their turns (0.8 %) and wait in 1.6 % (2.6 %), alike against Greedy (1.9 %, 1.7 %):
  not an adaptation to the system actor, which still waits in 7 % and defends in 2 %. It plays
  the same against both opponents, as `t7-1` did. On the neutral set the hero casts in 82 % of
  the rounds it can (67 %); the rest as before.
- **The user's next lever** (2026-10-26, the agent's recommendation): AI-Q7 (b) part 2, the
  kill preview in the network (T6d), an A/B against `t7-1`.

## T6d — The kill preview in the network (2026-10-26)

- **Built** (design §13; AI-Q7 (b) part 2): `[network] preview` (the MLP's; `resolve` refuses it
  with the entity network). `Policy.with_preview` adds each legal strike entry's bias `φ(f) ·
  u(h)` to the entry logits (`Mlp`: `preview_net` 4 → `embed` → `embed`, `preview_context`
  `hidden` → `embed`, gain 0.01; the features from the preview and the target slot's count and
  full hit points, `policy.preview_slots`); `act` reads `batch.preview` (refusing a batch
  without it), `evaluate` the rollout's. The trainer's batch, the gate's (`play_gate`, when
  either network reads it) and the ladder's (`ladder`, when any of its networks does) write
  the preview; `Rollout.preview` keeps it in half precision; `build` reads `preview` from a
  saved config (absent: none). `python -m h3env.train.network` shows and times the parts.
  `configs/e06-t6d.toml`: `t7-1`'s settings with the preview.
- **Agent's choices (change freely):** the bias on the entry logits only (the value and the
  cast heads, and a truncation's bootstrap from `final_obs`, which has no preview, as before);
  the four features; a bilinear bias, so the state weighs a kill; the context near 0 at the
  start; only the legal entries computed (sparse: `nonzero`, `index_put`).
- **Cost** (the sandbox's CPU, 4 threads, 256 rows, `--repeats 30`): 17 632 parameters more
  (5 964 249), 0.4 % more multiply-adds; a forward pass 57.9 → 80.4 µs a decision, a training
  pass 83.4 → 91.7 µs (the gathers and the scatter, not the arithmetic). The batch's preview
  costs about 29 µs an env step (T5's AI-Q7 (b) part 1), about 0.3 ms a call of 256 envs on
  the user's 24 threads, against `t7-1`'s about 50 ms a call.
- **Tests:** `test_train.py`: the bias touches the legal strikes alone and moves them, the
  stored half-precision preview gives the update the same log-probabilities, a batch or an
  `evaluate` without the preview is refused; a run with the preview stops, resumes, gates and
  plays its ladders, its files read the preview. `test_run.py`: the entity network refused.

## T6e — The A/B run `t6d-1`: the kill preview in the network (2026-10-26)

- **The run** (`python/records/t6d-1.json`; `configs/e06-t6d.toml`: `t7-1`'s settings with
  `[network] preview`; commit 7a8899c, C ABI v33, the system actor's policy v19; 20 M samples,
  the last gate at 85.4 minutes, the end at 105.0, one segment, 4 231 to 4 470 samples/s, 13 %
  fewer than `t7-1`'s 4 846 to 5 205): gates at 10 M (+214 Elo) and 20 M (+22 Elo), both
  promoted; evaluation 27.3 minutes (`t7-1` 12.2): the ladders 96, 270, 419 s and the final
  758 s (`t7-1` 93, 137, 138, 220). The first upload was copied while the final ladder still
  ran (`ended` null, the last save before it); the second is the run's end.
- **Results** (the final ladder, 500 per kind; `t7-1`'s in brackets): Random 0.821 (0.828),
  Aggressor 0.553 (0.551), Greedy 0.528 (0.528), the system actor 0.365 (0.374; hero 0.340,
  a siege a hero defends 0.357, a garrison's siege 0.481), the neutral set 0.978 at value
  +0.253 (0.972, +0.245; strata 0.982, 0.964, 0.862). Every win rate lies within the intervals
  (±0.011 to ±0.019). The neutral value's +0.008: a battle's reward on the neutral set has a
  standard deviation of 0.147 (the sandbox, this `best.pt`, 1 500 battles: standard error
  0.004), so about 1.5 standard errors of two independent runs (the shared episodes would make
  a paired comparison tighter; `t7-1`'s network is not in the sandbox); a fifth of the gap to
  the target +0.285.
- **The behaviour analysis** (`python -m h3env.train.inspect` on the user's `best.pt`, as T6a;
  `python/records/inspect/t6d-1-*.json`; against the system actor, `t7-1`'s in brackets):
  - **Targeting improved as intended:** 61 % of its strikes the best previewed (53 %), their
    kills 85 % of the best's (77 %), 4 % killing nothing while one could (12 %); melee into a
    retaliation while a target without one was in reach 6 % (11 %); a clean melee strike
    deals 196 hit points (151) and costs 50 (53). Against Greedy alike (64 %, 85 %, 3 %).
  - **But it strikes less and wanders:** it strikes at 42 % of the decisions with a legal
    strike (73 %), shoots at 62 % of those with a legal shot (88 %); moves are 38 % of its
    decisions (27 %), the tactics phase's 25 % (11 %); 44 decisions a battle (28), 6.0 rounds
    (4.9). Worst in the sieges: attacking a garrison it moves in 73 % of its decisions (43 %)
    and strikes at 16 % of those with a strike legal (70 %), 9.0 rounds (4.2); attacking a
    town a hero defends, half its decisions are the tactics phase's placements (36 a battle;
    8), whose rounds have no limit (BAT-TAC-002) and cost nothing at γ 1 but the step limit.
  - **The outcomes did not move:** the analysis's wins 0.395 (0.394) against the system
    actor, 0.529 (0.529) against Greedy; the hero casts in 79 % of the rounds it can (87 %),
    35 mana (33).
  - **The neutral set:** 84 % of its strikes the best (72 %), 90 % of the best's kills (80 %),
    2 % killing nothing (9 %); wandering stacks and guards a little faster (4.4 and 4.5
    rounds, 4.8 and 4.9) and better (reward +0.258 and +0.237, +0.239 and +0.228); a neutral
    town slower (7.2 rounds, 5.6; +0.136, +0.147), with the same wandering as the sieges.
- **Training:** truncations 0.14 % and 0.12 % of self-play at 10 M and 20 M (`t7-1` 0.4 %,
  0.001 %), on the final ladder 0.1 to 0.2 % (`t7-1` none); the entropy 2.81 and 2.54 (2.83,
  1.98): the policy stayed broader in the second half (KL and clip fraction alike).
- **The cost:** besides the 13 % of training, the evaluation's battles are longer and some
  truncate (a set's batch plays until its last battle ends): the final ladder 3.4 × `t7-1`'s,
  the analysis on the sandbox 821 and 431 s against 279 and 140 s.
- **Read:** negative. The preview fixes what T6a measured, the targeting, and the outcomes do
  not move: the agent spends it on wandering (moves, the tactics phase's placements) and
  strikes less. Targeting is not what limits it. Four 20 M runs with different settings
  (`t5-3`, `t7-1`, `t6b-1`, `t6d-1`) end at the same ladder. By AI-Q7's rule (kept only if it
  measurably helps) the preview stays out of the best configuration.
- **The checkpoint** (the user, 2026-10-26, the agent's recommendation on the first point):
  AI-Q7 closed: `[network] preview` stays in the code, off by default (opt-in, tested), out of
  the best configuration; increment 6 closed. Next: T8 (the search) refined now, the plateau
  taken as shown (four 20 M runs at one ladder; `t4-2` at 60 M barely above `t5-3` at 20 M),
  without the 60 M run the agent recommended first.

## Increment 7's refinement; T8a — the search's design (2026-10-26)

- **The user's choices** (asked at T6e's checkpoint): T8 now, the plateau taken as shown; the
  seam T8a (the design), T8b (the plumbing), T8c (a decision-time search measured with `t7-1`'s
  network, no training change), T8d (expert iteration) only if T8c gains; **determinized from
  the start** (the agent had recommended an oracle bound first: the true hidden part with the
  stream re-seeded, a cheap upper bound); the tree in Python first; `h3-battle`'s part
  `cross-lane` in T8b.
- **Designed** (design §14): a world per seed (the other hero's private part guessed by the
  encounter sampler from its public part, its hidden obstacles and a drawn spell placed again,
  the stream re-seeded), Gumbel AlphaZero at the root and the completed-Q selection below,
  open-loop over worlds, one network call per simulation round for the batch; ADR-0025 point
  12's amendment as a draft (`docs/lanes/ai/adr-drafts/0025-amendment-search.md`: a world reads
  nothing hidden, tested).
- **What the code has:** the battle's one stream (`BattleState.rng`, crate-private, seeded from
  the episode's seed) and `observe`'s list of what a side may not know (`h3-battle`'s
  `observe.rs`); `h3_env_fork` forks a single env only, nothing forks a batch's env; the
  encounter sampler draws a hero by faction and week (`h3-sim`'s `encounter/hero.rs`), so the
  guess fixes the hero type and draws the rest.

## The turn-order probe (2026-10-26)

- **The user's question:** how well observation v21 tells the agent the round's state: whose
  stack acts, which have acted, in which order the next go. **The encoding:** the active stack
  (`active`), the viewer to act, per stack acted, waited, defending, retaliated, good morale
  this round and its turn started, the round, the wait phase begun and the side the speed
  tie-break counts as last; the order itself is not given, only derivable (BAT-TRN-001 to 003:
  each stack's current speed, the flags, the tie side, the slot).
- **The probe** (`python/records/inspect/probe_order.py`, a one-off script; the result
  `inspect/t6d-1-order-probe.json`): `t6d-1`'s best plays itself (both seats external, the
  player kinds, 600 calls of 256 envs, the sandbox's CPU); at each decision after the tactics
  phase that is not a Wait, a sample of the trunk's output (512) and two labels from the later
  decisions of the same battle: the stack acting next (a morale repeat left out) and the
  opponent's stack acting next. Linear and MLP (256) probes masked to the candidates, trained on
  192 envs' samples, tested on the other 64's (107 131 and 123 790 labelled). Controls: the
  same probes on an untrained network's trunk and on the observation's order columns alone, an
  order rule computed from the observation, the naive "fastest stack yet to act".

  | Test accuracy | next stack | the opponent's next stack |
  |---|---|---|
  | chance (uniform over the candidates) | 0.10 | 0.24 |
  | the trained trunk, linear / MLP probe | 0.18 / 0.19 | 0.34 / 0.35 |
  | an untrained trunk, linear / MLP probe | 0.24 / 0.26 | 0.41 / 0.42 |
  | the order columns alone, linear / MLP probe | 0.65 / 0.72 | 0.76 / 0.78 |
  | the order rule from the observation | 0.89 | 0.91 |
  | the fastest stack yet to act | 0.79 | – |

- **Read:** the network does not represent the turn order. Its trunk predicts the next stack
  barely above chance and worse than an untrained network's (whose random projection keeps
  more of the input: training discarded what the order needs), while the order is in the
  observation (the rule 0.89 to 0.91; the probes learn 0.72 to 0.78 from the columns alone).
  It fits T6a's (the agent waits and defends less than the system actor) and T6e's (the
  wandering). `t6d-1`'s network, not `t7-1`'s (not in the sandbox): the same MLP and settings
  but the preview. Candidate lever (QUEUE): the order as an input, the engine's own (exact,
  re-evaluated before every activation) or computed from v21 in Python, then an A/B against
  `t7-1`; T8's search plays the true order through its forks either way.

## T6f — The turn order as an input (2026-10-26)

- **The user's choices** (before T8b, after the turn-order probe; asked at the session's start):
  the turn order first, as observation v22 with the engine's own order (`h3-battle` `cross-lane`,
  the user's consent), not a Python copy of BAT-TRN-001..003 (the probe's rule, 0.89) nor a batch
  output (the search's single envs would lack it); one pull request with T8b (one C ABI bump,
  v34); a 20 M A/B against `t7-1` prepared for the user's machine.
- **Built:** `h3_battle::turn_order(&Observation) -> TurnOrder` (`turn.rs`): the rest of the
  round's turns, the active stack first, then the next round's, as `next_active`'s picks give
  them if each stack takes its turn as it comes; it shares `next_active`'s key (`OrderKey`: the
  phase, the turrets, the Catapult, the speed, then the side tie-break by `rank(preferred)`, the
  slot, the id), so the rule stays in one place; the tie-break's side moves with each foretold
  turn (the Catapult's counts as the defender's). A stack that loses its turns (blind, petrified,
  paralysed) and an ammunition supply take no place; in the tactics phase the round is the first
  (its end starts round 1 fresh); an ended battle has none. Observation encoding v22
  (`h3-sim`): per stack slot `[its place in the rest of the round's turns (1: the active stack),
  among its side's (1: its side's next), in the next round's]`, 3 × 42 features appended after
  v21's layout (`ORDER` = 8 882, v21's length; `OBSERVATION_LEN` 9 008). C ABI v34.
  `[network] order` (the MLP's, off by default): the trunk reads the whole observation; without
  it v21's prefix, so every saved network (`t7-1`'s too) plays on. `configs/e06-order.toml`:
  `t7-1`'s settings with the order.
- **Agent's choices (change freely):** places, not one-hots (the MLP reads `symlog`); the side's
  place so that "the opponent's next stack" needs no minimum over slots; the next round's order
  with every flag cleared and nothing else changed (an effect ending at the round's start is not
  foreseen); the block appended (v21's layout as a prefix) rather than per slot. Not in v22:
  BAT-STUB-1's field (it needs an `Observation` field, a server-lane mapping).
- **Tests:** `h3-battle` `tests/turn_order.rs` (the order foretells every turn of defending
  battles with speed ties across rounds; waits go to the round's end, slowest first; a stack
  without turns; the tactics phase; an ended battle) and the property
  `the_turn_order_foretells_the_next_turn` (arbitrary games, sieges and war machines included:
  the turns a quiet Defend starts follow the order told, 146 turns over 43 games; both viewers
  alike). `h3-sim`: the block's columns, the layout pinned against `layout.py`. Python: a network
  with `order` reads the block, one without (and a saved config from before v22) does not; a
  run with it resumes and its files read the order.
- **Cost:** the batch's step counts the order block (an observation's `turn_order`: the keys
  without copying a stack, sorted by their unique key without the side tie-break, the picks a
  group scan): `batch-random-full` +7.03 %, `batch-random-encounters-v1` +5.90 % (both
  re-baselined, lane-only), `batch-random-siege` +1.80 %; the battle's rollouts unchanged
  (−0.01 %, +0.79 %). A training run is bound by the network's passes (about 5 000 samples/s
  end to end against 50 000 steps/s a thread inside the step), so this is below 1 % of a run.
- **The A/B run** (the user's machine; T6g):
  `uv run --group train python -m h3env.train --config configs/e06-order.toml --out runs/order-1`
  (about 1 h 45 min, as `t7-1`); then the ladder against `t7-1`'s and `python -m
  h3env.train.inspect` (the waits, defends and wandering T6a and T6e measured).

## T8b — The plumbing (2026-10-27)

- **Built** (design §14): `BattleState::determinized(ruleset, viewer, guess, seed)` and
  `reseed` (`h3-battle` `world.rs`, `cross-lane`, the user's consent): the stream re-seeded; the
  other side's hero rebuilt from the guess through the setup's check (BAT-HER-001) with its
  public part kept (the view, no retreat, the cost change), its limits and the templates of the
  stacks its spells would create rebuilt from it (`state::world_templates`: the battle's terrain,
  battlefield and army stacks), its opening spells due only before its first public turn; its
  hidden obstacles placed again (`obstacles::place_unseen_again`: their hexes cleared first, so
  no draw reads them); a drawn spell the viewer does not control drawn again (`casters::
  activated`); the Escape Tunnel and the retreat facts as the viewer knows them. `h3-sim`:
  `Sampler::guess` (of 8 heroes of the observed type drawn as an encounter draws one, the week
  by a hero battle's or a siege's prior, the nearest in primary skills; the skill of each war
  machine the view shows it commands learnt at Basic, room made from the skills that command
  none; `HeroData::details_of` split out with the draw order unchanged, so every encounter is
  the same), `BattleEnv::world` (its setup without what the viewer may not know, so a reset
  starts nothing hidden either), `BattleBatch::determinize` (the guess from the env's encounter
  distribution, stream `ENCOUNTER` of the seed; a type alone outside it), `BattleEnv::reseed`.
  C ABI v34: `h3_batch_determinize`, `h3_env_reseed`. Python: `BattleBatch.determinize`,
  `Env.reseed`, `h3env.search` (the module's documentation): Gumbel-top-k at the root with
  sequential halving (`considered_visits`, `mctx`'s schedule), the completed-Q improved policy
  below, open-loop over W worlds, one network call per simulation round for every env searched
  (and one per head for the leaves with a legal cast); `search(policy, simulations, worlds, k)`
  as `report`'s agents; `python -m h3env.search --time` and `--report` (T8c's measurement: the
  search and the sampled policy on the same held-out episodes, `--json`).
- **Agent's choices (change freely):** the root's candidates among the entries the true battle
  allows (a world's legal set may differ by the guessed hero's defences, BAT-Q26); a chosen cast
  the true battle refuses gives way to the network's most likely action (`stats["fallbacks"]`);
  below the root every entry legal at a node's first evaluation, a cast's heads the network's
  most likely there; an action a world refuses backs up the node's value (the schedule moves
  on); a world where another side moves after the same actions (morale, a death) evaluates its
  position without a node; values of reward v1 scaled to [0, 1] as (q + 1) / 2 for σ; the
  paper's `c_visit` 50, `c_scale` 0.1; the envs past `limit` episodes (one: `report`'s battles)
  take a random action, unsearched.
- **Gaps** (AI-STUB-1): a world keeps the number and value of the other side's hidden obstacles
  (the caster's mastery decides them, private; only their hexes are drawn again) and the last
  drawn spells of the stacks not active (read only by the computer's cast choice). The state
  keeps no list of a hero's cast spells, so a guessed book may lack one the hero cast.
- **Cost** (the sandbox's CPU, 4 threads, an untrained MLP of `t7-1`'s size, 16 envs of the
  player kinds, N 16, W 8, k 16): 17.7 ms a decision, 1.1 ms a simulation; of a decision about
  0.37 ms a world (8 hero draws), the network's passes most of the rest, the tree's walk and
  steps 0.2 ms a simulation.
- **Tests:** `h3-battle` `tests/world.rs` (twins differing in the stream, the hidden pits' hexes,
  the hero's private part and its templates give the same world; the viewer's observation of
  the world is the battle's; a seed's world, another's; re-seeding; no guess; a refused guess)
  and the property `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden` (arbitrary
  games, both viewers, every step). `h3-sim`: a batch env's world shows its viewer the battle,
  holds a guess of the observed type, a setup without the funds, plays to its end; the guess's
  type, distance and machines. `h3-ffi`: the ABI's worlds are the Rust batch's; the C smoke
  test plays a world of each env. Python `tests/test_search.py`: the binding's worlds, the
  schedule, the improved policy, legal and repeatable choices, battles to their end, refusals,
  `--report`.

## T6g — The A/B run `order-1`: the turn order as an input (2026-10-27)

- **The run** (`python/records/order-1.json`; `configs/e06-order.toml`: `t7-1`'s settings with
  `[network] order`; commit d1b263e, C ABI v34, observation v22, the system actor's policy v19;
  20 M samples in 86.5 minutes, one segment, 4 543 to 5 413 samples/s, `t7-1` 4 846 to 5 205):
  gates at 10 M (+245 Elo) and 20 M (+22 Elo), both promoted; evaluation 18.5 minutes (`t7-1`
  12.2): the 10 M ladder 458 s (137) and its gate 50 s (13), the battles longer there.
- **Results** (the final ladder, 500 per kind; `t7-1`'s in brackets): Random 0.832 (0.828),
  Aggressor 0.553 (0.551), Greedy 0.527 (0.528), the system actor 0.369 (0.374; hero 0.356, a
  siege a hero defends 0.343, a garrison's siege 0.475), the neutral set 0.978 at value +0.250
  (0.972, +0.245; strata 0.982, 0.962, 0.852). Every difference lies within the intervals
  (±0.011 to ±0.018). At 10 M it was behind: Greedy 0.479 (0.499), the system actor 0.327
  (0.359).
- **Training:** the first half shortened its battles later: 170 samples an episode (`t7-1`
  101), truncations 3.1 % of self-play (0.4 %; 12 % in the first 2 M samples); the second half
  79 (66) and 0.2 % (0.0 %); the entropy 2.97 and 2.14 (2.83, 1.98).
- **The behaviour analysis** (`python -m h3env.train.inspect` on the user's `best.pt`, the
  sandbox's CPU; `python/records/inspect/order-1-*.json`; against the system actor, `t7-1`'s in
  brackets): wins 0.399 (0.394); 26.6 decisions a battle (28.1), 4.96 rounds (4.94); its stacks
  wait in 1.3 % of their turns (2.6 %) and defend in 0.5 % (0.8 %), the system actor's 8.7 % and
  2.9 %; the tactics phase 6.6 % of its decisions (10.9 %); the hero casts in 93 % of the rounds
  it can (87 %), 34 mana (33); targeting 52 % at the best previewed (53 %), 76 % of the best's
  kills (77 %), 13 % killing nothing while one could (12 %); melee into a retaliation 77 % (77 %).
  Against Greedy alike. On the neutral set it waits in 8.1 % of its turns (4.4 %) and its hero
  casts in 95 % of the rounds it can (67 %), at the same value (+0.242; +0.236 in `t7-1`'s
  analysis).
- **The probe again** (`records/inspect/probe_order.py` on `order-1`'s best,
  `order-1-order-probe.json`; it now also reads the order block itself, `engine_order`): the
  trained trunk predicts the next stack at 0.22 / 0.25 (linear / MLP probe; `t6d-1` 0.18 / 0.19)
  and the opponent's next at 0.35 / 0.37 (0.34 / 0.35), still below an untrained trunk's (0.26 /
  0.32 and 0.40 / 0.46), while the block it reads tells them at 0.885 and 0.866. The network has
  the order in its input and does not keep it.
- **The block checked** (the same self-play, scratch scripts): outside the tactics phase the
  active stack holds place 1 in all 24 167 decisions. Of the opponent's next stacks the block
  misses and the probe's rule finds (1 290 of 16 849), 1 234 are automatic turns (a war machine
  whose skill its hero lacks, a turret, the Catapult: placed in the order, but no decision, and
  the probe's labels count decisions alone) and 56 a stack whose lost turns ended in play. No
  fault of `turn_order`; a probe of the order should label turns, not decisions.
- **Read:** negative. The turn order in the input moves neither the ladder nor the play T6a and
  T6e measured (the waits and defends went down, not up): the MLP's trunk drops the 126 order
  features among 9 008 as it dropped v21's order columns. Five 20 M runs with different settings
  (`t5-3`, `t7-1`, `t6b-1`, `t6d-1`, `order-1`) end at the same ladder. The agent's
  recommendation: `[network] order` stays off by default (opt-in, tested), out of the best
  configuration; observation v22 stays (its block costs the step about 6 %, under 1 % of a run).

## T8c — The measurement, in the sandbox with `order-1`'s network (2026-10-27)

- **The network:** `order-1`'s best (the user's upload, T6g; `t7-1`'s file is not in the
  sandbox): the same ladder as `t7-1`'s, observation v22 with `[network] order`. The search
  with it plays (`--report` on 20 per kind first: 120 battles, 14 ms a decision at N 16).
- **The search was the sampled policy** (`--report`, N 16, W 8, k 16, the report set against
  the system actor, 2 998 battles): 0.374 and value −0.101, the sampled policy 0.374 and −0.102
  on the same episodes. The cause: σ(q̂) took the completed values on a fixed scale ((q + 1) / 2,
  T8b's choice), but a battle's values differ by a few hundredths between actions (their spread
  over a root's candidates: median 0.08) while the prior's logits differ by several units
  (median 7.0): σ spread 0.2 to 0.3, and the root's choice differed from the prior's own
  Gumbel-max sample in 1.0 % of the decisions at N 16, 2.1 % at N 64 (384 roots each, the
  sandbox's self-play). **Fixed:** σ rescales the completed values by their least and greatest
  over the node's legal candidates (`search.rescaled`; the paper's normalization, `mctx`'s
  `rescale_values`), so σ spans `(c_visit + max N) · c_scale` (5.1 at one visit); the choice
  then differs from the prior's sample in 45 % (N 16) and 33 % (N 64). Test:
  `test_small_value_differences_decide_once_rescaled`.
- **N 16, rescaled** (the same set): 0.371 ± 0.018, value −0.101 (the sampled 0.374, −0.102):
  no gain. 102 000 decisions (the fixed scale's, which played as the sampled policy, 79 000:
  longer battles), 26 ms a decision, 45 minutes on the sandbox's 4 threads. At N 16 and k 16
  sequential halving gives each candidate one visit, in one world, and its one step holds the
  damage rolls: the comparison is mostly noise.
- **N 64** (rescaled; the report set's first 200 per kind against the system actor, 1 199
  battles): 0.384 ± 0.028, value −0.092 (the sampled 0.381, −0.095 on the same episodes): no
  gain beyond the intervals (per kind and side within 5 of 200 battles). 60 ms a decision, 49
  minutes.
- **The neutral set, N 64** (attacking the system actor): on its first 200 per kind (600
  battles) value +0.250 (the sampled +0.238), the same +0.012 in each kind; on the whole set
  (500 per kind, 1 500 battles) +0.247 (+0.242; wandering +0.251 and +0.246, guards +0.234 and
  +0.234, a neutral town +0.152 and +0.151), wins 0.974 both: the first 600's gain did not hold.
  36 ms a decision, 25 minutes.
- **Not run:** Greedy (the system actor's rows show no gain at either N), `t7-1`'s network (not
  in the sandbox; `order-1`'s plays at its level), N 64 on the report set's other 300 per kind.
- **Read:** negative. A decision-time search over determinized worlds with this network gains
  nothing beyond the intervals against the system actor (N 16 and 64) or on the neutral set
  (N 64), at 13 to 47 times the sampled policy's time. The network's values tell the actions
  apart by a few hundredths, less than one step's chance and the worlds' guesses vary them: its
  evaluation, not the planning on top of it, limits the agent. By design §14's rule T8d (expert
  iteration) is not indicated; it would also need the tree in Rust (the sandbox's search makes
  about 17 to 38 decisions a second, a training run some thousands).
- **Put to the user** (the checkpoint; the agent's recommendation first): (a) close increment 7
  with T8 negative and decide E06's end: decision 7's targets revised to the evidence (six
  levers, one ladder: PPO on this network plateaus at Greedy's level, 0.37 against the system
  actor) and T9, the closure, with `t7-1` the battle AI v1; a stronger agent (a network that
  reads the battle's structure, a search in Rust with expert iteration) as an epic of its own;
  (b) T8d regardless (the tree in Rust first); (c) a search diagnostic first: the true world
  (the hidden part known, the stream re-seeded; a batch fork, an ABI addition) bounds what the
  guess costs. AI-STUB-1 (T8b's gap) matters only for (b).

## Increment 7's checkpoint; increment 8's refinement, T9 the closure (2026-10-27)

- **The user's choices** (asked at the session's start, the agent's recommendations taken):
  close increment 7 with T8 negative and E06 with `t7-1` as the battle AI v1; decision 7's
  targets revised to "beat every heuristic bot" (against Random, Aggressor and Greedy a win rate
  above 0.5 and a positive value efficiency; on the neutral set a value above every heuristic
  bot's), which `t7-1` meets (T7d's final ladder: 0.828, 0.551, 0.528 at +0.273, +0.029, +0.010;
  the neutral set +0.245 against Greedy's +0.223 and Aggressor's +0.216). Against Greedy the
  rate is within its interval (±0.018) of 0.5: a point estimate, as ADR-0026 point 7 judges.
  The system actor (it beats `t7-1` 0.626; +0.285 on the neutral set) and a stronger agent
  are E25's (Battle AI v2, a sketch in BACKLOG): a network that reads the battle's structure,
  a search in Rust trained on its own choices, the true-world search as a diagnostic, shaping.
- **Done in this session:** the AI lane's part (the epic file: decisions 2 and 7, exit
  criterion 5, the risk; increment 7 and T5's and T6's open parts to `E06-archive.md`;
  AI-STUB-1's epic E25; the ADR draft retired) and the `shared/` part
  (`shared/e06-search-docs`, merged first: ADR-0025 point 12a with T8b's as-built facts,
  `docs/07-simulation-and-ai.md` with observation v22, C ABI v33's preview and v34's worlds,
  BACKLOG's E06 row and E25). The AI queue's three inbox entries for the shared change leave it.
- **Increment 8** (the epic file): T9a the audit, T9b the independent review (at most three
  reviewers), T9c the verifies (the user's machine, exit criterion 6), T9d the closure (a
  `shared/e06-closure` first), E05's T9 as the pattern.

## T9a — The audit (2026-10-27)

- **Exit criteria 1 to 5 against the code, tests and records** (on `master` at 73a0bfe):
  - **1 met.** Every run the epic's results cite has its record in `python/records/` (`t3b-1`,
    `t4-1`, `t4-2`, `t5-1` to `t5-3`, `t6b-1`, `t6d-1`, `t7-1`, `order-1`; `t5-4` was prepared,
    never run), each with its config, commit (none dirty), C ABI version, rulesets, metrics;
    `test_every_committed_record_and_config_is_valid` checks them and the configs.
  - **2 met.** `test_a_stopped_run_resumes_to_its_budget`, `…_killed_hard_resumes_from_its_last_checkpoint`,
    a gate and a ladder cut short (`…_runs_first_at_the_resume`), the league's files, the entity,
    preview and order runs' resumes (`test_train.py`); `test_run.py`'s resume rules.
  - **3 met.** The SPRT (`test_gate.py`: bounds, error rates, a network against itself not
    promoted) promotes by itself in every run; every promoted network has its ladder in its
    record (`t3b-1` to `order-1`: one per promotion, the final best's again at the end; `t5-1`,
    stopped, both of its).
  - **4 met.** `t7-1`'s league (`self_play` 0.5, 16 members, 4 live, PFSP power 2) trained it;
    `test_the_league_draws_scores_and_retires`, `…_retires_its_most_beaten_and_a_resume_keeps_its_files`.
  - **5 met** (decision 7 as revised): `t7-1`'s final ladder (500 per kind) 0.828, 0.551, 0.528
    at +0.273, +0.029, +0.010; the neutral set +0.245 against the bots' table's Greedy +0.2239,
    Aggressor +0.2170, Random −0.0502 (the epic's +0.223 and +0.216: truncated, not rounded).
    **One caveat:** `t7-1` played the system actor at `POLICY_VERSION` 18 (its record), the bots'
    tables are at 19 (E23 T7, 2026-10-23: the neutral side's neighbour table), so the neutral
    comparison is not on one engine. It holds whichever: the three A/B runs at 19 on `t7-1`'s
    settings reach +0.245, +0.253, +0.250 (`t6b-1`, `t6d-1`, `order-1`), and `t7-1`'s own
    analysis on the sandbox's CPU (`inspect/t7-1-neutral-v1.json`) +0.236, all above Greedy's.
    The player kinds' rows do not move with the policy (the bots' tables' values stayed the same
    across the core ruleset's later hashes). T9c plays `t7-1`'s final ladder again on today's
    engine (its network is on the user's machine).
- **The leftovers sorted** (the AI queue's inbox, by destination): **E25** the increment-7
  candidates (`[network] order` and `preview`, shaping, the 60 M run, `t5-4`), the search's
  (AI-STUB-1, T8c's Greedy rows, a repetition rule for a search agent, the oracle harness), T5's
  open parts (the batch size and threads, the observations in place) with the performance notes,
  `inspect`'s turret shots; **encounters v2** A3's growth too (meant for E23, closed without it);
  **E13** unchanged; **the adventure env** T55b's typed answers (T58's withdrawal of A-23 folded
  in); **any time** the viewer-less `encode`, `tools.Env` after a panic. To `QUEUE-ARCHIVE.md`
  as done or informational: E08 T45, T51b, T52, T58, E23 T4c and T6's last part, the turret
  preview, the baselines and AI-Q7.
- **The Python READMEs:** `python/README.md` gained the kill preview in the network, the system
  actor's share, the behaviour analysis, the battle AI v1 and the search's outcome;
  `records/README.md`'s lineage `t7-1` as the battle AI v1 and the three A/B runs.

## T9b — The independent review (2026-10-27)

Three fresh read-only reviewers, one slice each, run one after another; every finding with a
failure path, verified against the code by the agent, then fixed with a test that fails before
the fix, or kept with its reason.

- **r1, the trainer** (`h3env.train`; 4 findings, 4 doubts, 7 test gaps): **fixed:** on Windows a
  refused `checkpoint.pt` replace (another process holding it, `run.atomic_write`) still deleted
  the retired snapshots' files the old checkpoint names, so `--resume` failed in `restore`; a
  promotion's refused `best.pt` was never written again (a user's upload of `best.pt` then the
  network before); an update of fewer than twice `minibatches` samples normalized a single
  advantage (NaN weights: `--set env.envs=1`). `save` returns whether it wrote, the files go
  after a written checkpoint, a refused best is written at each checkpoint until it is,
  minibatches hold two samples at least (`test_a_refused_replace_keeps_what_the_old_checkpoint_names`,
  `test_an_update_of_fewer_samples_than_minibatches_stays_finite`). **Kept:** the kill preview
  stored in half precision (kills above 2 048 round, the first epoch's ratio off by that; the
  preview is off by default, T6e); the record's ladder table without `skipped` and `games`
  (`metrics.jsonl` keeps them; every set's games are fixed). Doubts not traced: promotion files
  ahead of a checkpoint and a resume that rejects; a gate looping on a skipped episode (both
  seats external: none known); a very large `pfsp_power` underflowing every weight. Sound: the
  resumable state, the SPRT, the league, PPO's masks, GAE and seats, the held-out check.
- **r2, the search and its worlds** (`h3env.search`, T8b's part of `h3-battle`, `h3-sim`,
  `h3-ffi`; 7 findings, 4 doubts): **fixed:** below the root a node took its entries from the
  world's single-env mask, which keeps Retreat and Surrender (ADR-0025 point 7 masks them in the
  batch): an opponent's node could retreat and give the viewer a win (masked now,
  `test_no_node_takes_an_escape_a_worlds_own_mask_allows`); `h3_batch_determinize` made a world
  for either side, though a world of the side not to move holds that side's own hero as it is
  (refused now, as its documentation said; `h3.h`, the binding); the docstring's "reward v1 at a
  terminal" and "Gumbel-perturbed cast heads at the root" (neither built: corrected).
  **Kept, the user's decisions (2026-10-27):** a world's step scores a decided battle ±1 (a single
  env's reward) where the value head learned reward v1 (about ±0.2 to 0.4), so a terminal inside
  the horizon outweighs every leaf and, with the escapes, distorted T8c's search: **T8c's verdict
  is inconclusive**, not negative; a terminal scored by reward v1 and T8c again are E25's first
  step (the AI queue's "For E25"; E06 closes on `t7-1`, whose targets the search does not touch).
  Three gaps of a world against ADR-0025 point 12a.2 (the other hero's morale and luck modifiers
  kept, the magic limits recomputed with the guess, a commanded war machine automatic without a
  guess) join AI-STUB-1 (E25); T9d's shared part amends point 12a's "one known gap". Doubts: a
  world copies the env's step count; re-placed obstacles avoid every occupied hex; a past summon
  the guessed book lacks; a failed `determinize` stops the evaluation. Sound: sequential halving,
  Gumbel-top-k, completed Q, σ's rescaling, the backed-up signs, determinism, `reseed`, the hidden
  obstacles cleared, the C ABI's checks.
- **r3, the batch since E05** (C ABI v31 to v34; 3 findings, 3 doubts): **fixed:** a scenario
  with retreat facts (a Tavern) let the system actor retreat inside a batch call, which point 7
  rules out for every seat (the encounters leave them unset: training unaffected):
  `Scenario::new` drops them, as it sets both sides human; a run's record names
  `ENCOUNTERS_VERSION`, not the two data files the distribution is built from: a test pins their
  bytes to the version (`encounter::tests::the_shipped_data_are_their_version`; TRAPS). **Kept:**
  r1's half-precision preview again. Doubts: `play_internal` has no progress guard if the system
  actor's command were refused; the record's commit is the Python tree's, the library names only
  its ABI version (the AI queue's "Any time"); the turn order's side by owner, not controller (a
  hypnotized stack). Sound: no thread-count dependence (threads 0 to 7, bit for bit), the pool,
  the buffers' checks, the layouts (`h3.h`, Rust, `_ffi.py`), refusals leave state, the preview
  rewritten every call, the turn order from the viewer's observation (`ORDER` 8 882 in both).
- **The Python tests** (the lane command, `uv run --group train pytest` in the sandbox): two
  failures before the fixes, the bots' tables' adventure hash stale since E08 T61 (the Python
  tests are no CI gate): both regenerated, every number the same.

## T9c — The verifies on the user's machine (2026-10-27)

- **The code:** `master` at 9263fc8 (#519 merged). The user's Windows 10 machine, release, the
  script `t9c.ps1` (the QUEUE's T9c), its summary and logs uploaded. Exit criterion 6 as the
  user amended it at T9b (Linux: CI's full verify, no run with the original data).
- **The Windows verify** (`cargo xtask verify --with-original-data --summary`): green; every step
  passed (coverage 372 s, determinism 95 s, `original-data` 34 s), `ffi-smoke` and `bench-check`
  N/A on Windows. **The Python tests** (`uv run --group train pytest`, the release library built
  first): passed. **Linux:** CI's full verify on #519 (the same code) and the Python tests in
  the sandbox at T9b (137 passed, the GPU test skipped).
- **`t7-1`'s final ladder on the closing engine** (T9a's caveat: it had played the system actor
  at `POLICY_VERSION` 18, the bots' tables at 19; `python -m h3env.train.ladder --device cuda`,
  500 battles per kind): the report set 0.830 ± 0.013 at +0.273 against Random, 0.550 ± 0.018 at
  +0.029 against Aggressor, 0.530 ± 0.018 at +0.010 against Greedy, 0.372 ± 0.018 at −0.103
  against the system actor (2 998 battles, 2 skipped); the neutral set 0.970 ± 0.013 at +0.244
  (wandering 0.974, guards 0.970, a neutral town 0.852), the bots' table's Greedy +0.224,
  Aggressor +0.217, Random −0.050. The same as `t7-1`'s record within its intervals (0.828,
  0.551, 0.528, 0.374; +0.245): decision 7's revised targets hold on the closing engine.
- Exit criterion 6 met (as amended); with T9a and T9b, criteria 1 to 6.

## T9d — The closure (2026-10-27)

- **Shared first** (#522): BACKLOG (E06 done; E25's search candidate and the note: T8c
  inconclusive), `docs/07-simulation-and-ai.md` §1.2 (the training pipeline as built) and its
  §1.1 and §2 references, ADR-0026's working-detail line, ADR-0025 point 12a after the review (a
  world through the C ABI for the side to move only; AI-STUB-1's gaps). Docs only.
- **The design note at closure** (below): its sections stayed after their tasks built them, as
  the record of the build's decisions; its text is kept with its numbering, the note deleted (the
  ADR index's rule), the references in `python/` repointed to `E06-log.md` §n (comments and
  docstrings only). `h3-battle`'s `world.rs` cites the note too: the battle queue's inbox says
  where to.
- **Exit criteria:** 1 to 5 (T9a, after T9b's fixes) and 6 (T9c, as amended) met. E06 closed.
- **What later epics inherit:** the AI queue's inbox by destination (E25: the search's terminal
  by reward v1 and T8c again first, AI-STUB-1, the levers E06 left, throughput; encounters v2;
  E13; the adventure env), `t7-1` as the battle AI v1 and the ladder's baseline.

## The design note at closure

`docs/epics/E06-design.md` was deleted when E06 closed (T9d, the ADR index's rule). Its sections
had stayed after their tasks built them, as the record of the build's decisions; its text at the
closure follows, its numbering kept: "`E06-log.md` §n" in `python/` and "design §n" in the code,
the epic file, earlier entries of this log, the queue and its archive point here. The lasting
parts are in `docs/07-simulation-and-ai.md` §1.2 (the training pipeline as built); the decisions
are ADR-0026's (the search's worlds ADR-0025 point 12a).

Companion to `docs/epics/E06-battle-ai.md` and ADR-0026 (E06 T1; accepted by the user 2026-10-20,
every point as proposed: `E06-log.md`, T1). The training pipeline on the user's machine: a run, its
config and records (§1, §4), its budget and schedule (§2), the resumable state (§3), the held-out
sets (§5), the evaluation gate (§6), the ladder (§7), the league (§8), the costs (§9) and the
increments (§10). Each task turns its section into code and docs and deletes it here. Starting
point: E05's trainer (`python/h3env/train/`, `E05-log.md` §8 and §9).

### 1. A run

#### 1.1 Its directory

A run is a directory, `python/runs/<run>/` by default (git-ignored), named by the user:

| File | Holds | Written |
|---|---|---|
| `config.toml` | the config file as given, copied | at the start |
| `checkpoint.pt` | the full training state (§3.1) | every few minutes, before a gate, at a stop |
| `best.pt` | the current best's network and its metadata | at a promotion |
| `promoted/<n>.pt` | each promoted network (n from 0, the starting one) | at a promotion, never rewritten |
| `league/<id>.pt` | each league snapshot's network (§8) | once; deleted when it retires |
| `metrics.jsonl` | one typed record per line (§4.1) | appended, flushed per line |
| `record.json` | the run's summary (§4.2) | at each gate, ladder, stop and the end |

Every `.pt` and `record.json` is written atomically (§3.2). A crash leaves at most a stale `*.tmp`,
removed at the next start. Built (T2a, T2b1, T3b, T4a): `config.toml`, `metrics.jsonl`,
`record.json`, the atomic writes, `checkpoint.pt` (the whole state, `Trainer.state`),
`promoted/<n>.pt` and `best.pt` (each promotion's network with its index, how, run and parent),
`league/<id>.pt` (each snapshot's network; a retired one's deleted after the next checkpoint).

#### 1.2 The config file — built (T2a)

`h3env.train.run` (`Config`, `load_config`; `python/README.md`, "Training runs"): TOML, the
defaults in code, `--set table.key=value`, unknown keys refused, the resolved config in the
record; configs in `python/configs/`. Its tables today: `[env]`, `[network]`, `[ppo]`, `[pool]`,
`[checkpoint]` (`checkpoint_minutes`, T2b1), `[device]`, and since T3b `[gate]` (`gate_every`
in samples, `elo0`, `elo1`, `alpha`, `beta`, `max_pairs`, §6) and `[ladder]`
(`ladder_opponents`, `ladder_episodes` and `ladder_final_episodes` per kind, `ladder_promoted`,
§7), which replaced `[evaluation]` (a key belongs to one table, hence the prefixes), and since
T4a `[league]` (`self_play`, `snapshot_every`, `max_members`, `live`, `live_every`, `pfsp_power`,
`decay`, `retire_games`, §8) replacing `[pool]`. To come: `network.kind` (`"mlp"`, `"entity"`,
T5).

#### 1.3 Commands

```
uv run --group train python -m h3env.train --config configs/e06-t3.toml --out runs/t3-1
uv run --group train python -m h3env.train --resume runs/t3-1 [--set samples=400_000_000]
uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-1 \
    --init-from runs/t3-1/best.pt
```

- `--resume` — built (T2b1; `h3env.train.run.resumed`, `ppo.resume`): the config comes from the
  checkpoint; its `--set`s may change only `run.RESUMABLE` (`samples`, extended or cut;
  `env.threads`; `[device]`; `checkpoint.checkpoint_minutes`), any other change is refused with
  its keys named (a new run). A finished run resumes only with a larger budget.
- `--init-from` — built (T3b; `Trainer.init_from`): a new run (fresh optimizer, counters,
  pool) from a saved network of the same size (`best.pt`, `promoted/<n>.pt`, `checkpoint.pt`);
  the record's `parent` names its file, run, promoted index, samples and updates, and the
  parent's network is the new run's best #0 (`how` `init`), so its gates continue the lineage.

### 2. The budget and the schedule, in samples — built (T2a)

`samples` (the budget: the first update boundary at or past it), the learning rate (`linear` to
`lr · lr_floor` over `lr_horizon`, fixed at the start; or `constant`), the league's snapshots and
live draws (T4a) and the gates (`Cadence`, T3b; E05's evaluation against Random is gone) count
samples (`run.py`). Wall time sets only the checkpoints (T2b1).

### 3. The resumable state — built (T2b1, T3b, T4a)

Built (`ppo.Trainer.state`, `restore`, `ppo.resume`; `run.Run`, `Stop`, `resumed`,
`segment_seed`; `E06-log.md`, T2b1): `checkpoint.pt` format 1 holds the resolved config, the
segment, the counters (samples, updates, episodes, the run's seconds), the best rate, the
cadences' next samples, the network, the optimizer, the pool's networks (E05's pool: inside the
checkpoint, 8 networks at most), the generators (sampling, the pool's numpy one, torch's CPU and
CUDA states), `metrics.jsonl`'s length and whether the run finished. It is taken at an update
boundary only: at the start, every `checkpoint_minutes` (5) of wall time, after each evaluation, at
a stop and at the end. Not saved: the batch's battles, `SeatBook`, the open transitions (a resume
starts new episodes, decision 4). Ctrl+C: the first stops a rollout at its next call (discarded:
no weight, no counter changed) or lets an update or an evaluation finish, then saves with a `stop`
record; the second raises. `--resume` cuts `metrics.jsonl` back to the saved length (the
`segment` record counts the records dropped), restores the state and opens a new segment on its
own batch seed (SplitMix64 of the run's `seed`); every episode's seed is still checked against the
held-out set's. Reproducible from the records, not bitwise equal to an uninterrupted run.

Built in T3b (`checkpoint.pt` format 2; format 1 is refused at a resume, its `best.pt` can seed a
new run `--init-from`): the gate's state (the promoted entries, the next gate in samples, the
last gate's update, the last final ladder, `pending`: a gate or ladder that began after the save
and has no result). A checkpoint is taken before and after each gate and ladder; a pending one
runs first at the resume, again, the gate on the saved learner, the ladder on the promoted network
read from `promoted/` (with the ones before it its ladder plays: the networks stay outside the
checkpoint); a gate or ladder stops at its next batch call on Ctrl+C (`evaluate.Stopped`). A
replace Windows refuses five times keeps the old file, warns and goes on (`run.atomic_write`).

Built in T4a (format 3; format 2 is refused at a resume): the league replaces the pool, each
member's id, kind (snapshot, promoted; later the system actor), file (`league/<id>.pt` or its
`promoted/<n>.pt`, outside the checkpoint), birth and its decayed and plain score counts, the live
set, the next draw and the league's generator (`League.state`); the members' networks come from
their files at the resume. A retired snapshot's file is deleted after the next checkpoint (the
last one still names it); a resume removes the files of `league/` its checkpoint does not name.

### 4. The record — built (T2a, T2b1, T2c, T3b, T4a)

Built: `run.Run` and `run.Record` (typed `metrics.jsonl`: `start` with the identity, `update`,
`snapshot`, `eval`, `stop`, `segment`, `end`; `record.json` schema 2, rebuilt from those records
in order: its segments with their seeds, commits, changes, spans, how each ended and the records
dropped), `python/records/` (README, `check_record` over every committed record in
`tests/test_run.py`); schema 3 (T2c, C ABI v31): the identity's encodings' and encounter
distribution's versions and the batch's `entries` (kind, weight, the core, battle and adventure
rulesets' content hashes); schema 4 (T3b): `parent`, the `gate`, `promotion` and `ladder`
records and the record's `gates`, `promotions`, `ladders`, `best` (the latest promotion), its
curve per gate interval and `progress.evaluation_minutes` (`python/records/README.md`); schema 5 (T4a): the league's
fields in each `update` record (its size, the live set, whether it was drawn, the episodes against
members and the learner's mean score in them, the frozen networks a call passed), `snapshot`
records (the member, the one it retired), the member table in each gate (id, kind, birth, `p`,
games, whether live; the last gate's is the end's), the curve's `league_score` and
`frozen_passes`. Schema 4 records stay valid (`t3b-1`).

### 5. The held-out sets — built (T3a)

Built (`h3env.train.evaluate`, `h3 ai eval --set`, `python -m h3env.train.ladder --set`;
`E06-log.md`, T3a; the constants pinned between Rust and Python by an `h3-cli` test):

- **The report set v1** (the ladder and the targets): per kind with a player on both sides (a
  hero battle, a siege a hero defends, a garrison's siege; `H3_ENCOUNTER_*` 2, 3, 4) a batch of
  500 envs, seed `REPORT_V1_SEED + k`, the kind one-hot, each env's first episode played from
  both sides: 500 games per (kind, side), 3 000 per opponent. The overall rate weighs the kinds
  as the distribution does (4 : 2 : 1), the sides equally; its 95 % interval comes from the
  stratified variance `¼ Σ Σ w² r (1 − r) / n`: ±0.9 points at 0.95, ±1.8 at 0.75, ±2.0 at
  0.50 (one kind ±1.4, ±2.7, ±3.1). The first n envs of each kind are themselves a stratified
  set: 250 give ±2.5 at 0.75 (the intermediate ladders, §7). The sampled policy's generator is
  seeded per (kind, side) alike for every network (common random numbers). The neutral kinds
  have a set of their own, the neutral set v1 (T7c, §12; the player's seat only).
- **The gate set v1** (the SPRT, T3b): the default kinds, `GATE_V1_SEED`, 500 envs × 8
  episodes; pair j is episode ⌊j / 500⌋ of env j mod 500, 4 000 pairs at most, each played
  twice (the candidate attacking, then the best). Separate from the report set, so the networks
  the gate selects are not measured on the episodes that selected them. Its seeds are defined;
  T3b plays it.
- **v0** (E05's `EVAL_SEED`, 100 envs × 10 episodes, the default kinds) stays selectable, so
  E05's numbers can be compared; the trainer's evaluation plays it until T3b's gate.
- **Disjointness without a growing set** (T2a, T3a): every held-out episode's seed (v0's, the
  report set's 1 500, the gate set's 4 000) is computed once and each training episode's seed
  checked against them, a hit stops the run; a run's `seed` among the sets' batch seeds is
  refused, and so is a ladder of a network trained on one.

### 6. The evaluation gate — built (T3b)

Built as below (`h3env.train.gate`: `Sprt`, `play_gate`, `gate`, `python -m h3env.train.gate`;
in the run `ppo._go`); the end of a run gates the learner if it trained since its last gate, and
`gate_every` 0 gates only there. The SPRT's error rates and pair counts were checked by simulation
(`tests/test_gate.py`: about 590 pairs at either hypothesis, 64 at +100 Elo). The first
min(500, `max_pairs`) envs of each batch play (a smaller cap needs fewer).

- **When:** every `gate.gate_every` samples (10 M, about 20 minutes at E05's rate), the learner's
  current network is the candidate. The run starts with best #0: the untrained network, or the
  parent's for `--init-from`.
- **Play:** the gate set's pairs in order; two batches of 500 envs, both seats external (in one
  the candidate attacks, in the other the best), each network acting on its rows (E05's
  `Trainer.act` with two networks); both batches advance until every env finished its next
  episode, then the pairs up to there are scored.
- **The score of a pair:** the candidate's mean over its two games, a win 1, a draw or a
  truncation ½, a loss 0: one of 0, ¼, ½, ¾, 1. A pair plays both sides of one battle, so the
  attacker's advantage cancels.
- **The test** (a generalized SPRT on the pair scores, as Fishtest's normal approximation):
  `H0: s = s0`, `H1: s = s1` with `s = 1/(1 + 10^(−elo/400))`; after n pairs
  `LLR = n (s1 − s0)(2 x̄ − s0 − s1) / (2 σ̂²)`, x̄ and σ̂² the pairs' mean and variance from the
  five outcomes' counts with half a pair added to each (no zero variance early). In pair order,
  the first n with `LLR ≥ ln((1 − β)/α)` promotes; `LLR ≤ ln(β/(1 − α))` rejects; `max_pairs`
  without either rejects (inconclusive). The pairs after the deciding one are not counted.
- **Bounds** (the config's `[gate]`, recorded): `elo0 = 0`, `elo1 = 20` (s1 = 0.529), α = β =
  0.05 (±2.94), `max_pairs = 4 000`. The expected pairs: about 500 to 800 at either hypothesis
  (a pair's variance 0.08 to 0.125), about 840 to 1 310 halfway, so the cap stops only a
  candidate equal to the best within a few Elo. A large gain decides early (+100 Elo: about 65 to
  100 pairs).
- **A promotion:** the candidate becomes `promoted/<n>.pt` and `best.pt`, joins the league (§8),
  and the ladder plays it (§7). A rejection changes nothing. No second condition: a candidate
  that wins the gate but loses ground against the bots is still promoted; the ladder shows it, and
  a guard follows only if it happens.

### 7. The ladder — built (T3a, T3b)

Built: the report set's ladder (`evaluate.ladder`, `report`; T3a) with network opponents (both
seats external, `evaluate.duel`; T3b), played in the run for best #0, every promotion and the
final best (`Trainer.ladder`), each a `ladder` record. Not built: `h3 ai eval --games`.

- **When:** for best #0 at the start, for every promoted network, and at the run's end for the
  final best on the full set. In the run's process, between updates (the GPU is the run's).
- **Opponents:** Random, Aggressor, Greedy and the system actor (the engine's seats; the system
  actor since T7c), and the previous `ladder.ladder_promoted` (1) promoted networks of the lineage (both seats
  external). Earlier ones are compared through the records, not replayed. With
  `ladder.ladder_neutral` (T7c) the neutral set v1 too, attacking the system actor (§12).
- **Set:** the report set v1, the first `ladder.ladder_episodes` (250) per kind for an intermediate
  ladder, all 500 at the end and for any claim of a target (built as a tool in T3a:
  `evaluate.ladder`, `report`; T3b plays it inside a run).
- **Per opponent:** the win rate per (kind, side), per kind and overall with the 95 % intervals
  of §5, the value efficiency (mean reward v1, weighed alike: at reward v1's currency and price,
  `evaluate.CURRENCY`, `MANA_PRICE`, whatever the run's `[env]` trains with; the gate reads
  results only), truncations, skipped episodes, games and seconds. One `ladder` record and the
  record's ladder table.
- **The bots' table** on the report set — built (T3a): `h3 ai eval --set report-v1 --json
  python/records/bots-report-v1.json` plays every pair of bots on the same episodes; the bots
  are deterministic and Random draws from the episode's seat stream, so the table is a function
  of the engine (its identity in the file, checked by a Python test): regenerated when the bots
  or the rules move (the lane's trap). The ladder prints the bots' rates beside the network's.
- **Ratings:** Glicko-2 stays in one place, `h3 ai eval` (`src/ai/rating.rs`). It is a report,
  not part of the promotion or the targets; when one is wanted, `h3 ai eval --games <file>`
  merges the network games Python wrote (`ladder` records) with the bots' (optional; not built
  in T3a).
- **The targets** (decision 7) are judged on the lineage's latest promoted network's full ladder:
  each point estimate at or above its target, the interval reported. Never the best of many
  checkpoints on the same set (that selects on its noise).

### 8. The league — built (T4a)

Built (`h3env.train.league`, the trainer's `join`, `assign`, `collect`; `[league]`): PFSP over a
snapshot every `snapshot_every` samples (1 M; E05's pool: 600 000) and every promoted network
(best #0 from the start), at most `max_members` (16) on the device; an episode is self-play with
`self_play` (0.5), else a live member on a uniformly drawn side (only the learner's decisions
train); `p_m = (score + 1) / (games + 2)` (a win 1, a draw or truncation ½, decayed by `decay`
0.99 per update); `live` (4) members drawn without replacement by `(1 − p_m)^pfsp_power` (2),
each new episode's member among them by the same weights; a new member retires the highest
`p_m` among those with `retire_games` (200) games, or of all, the oldest of equals; a retired
promoted network's file stays. One change from the outline: the live set is drawn again every
`live_every` samples (250 000, about ten updates at `t3b-1`'s size) and when it holds a retired
member or fewer than it could, not at every rollout: an episode lasts longer than a rollout
(`t3b-1`: about 147 calls against 128) and keeps its opponent, so a draw per rollout would have a
call pass two or three sets' networks; the update record's `frozen_passes` measures it (T4b).

The system actor is no member (T7c): it plays the neutral kinds' defender inside the batch, so a
neutral episode draws none (§12). From T6b it also plays a fixed share of the player kinds'
episodes (`system_share`, §12), still no member: the batch draws an episode's entry, and with it
the seats, itself, so its share cannot follow PFSP's weights. Still to come: exploiters (decision 9) only if a check shows the agent exploitable: a fresh learner trained
against the frozen best alone for a fixed budget; if it beats the best at least 0.70 on the gate
set, main exploiters join (designed then, T6).

### 9. Costs on the user's machine (estimates; T2 to T4 measure them)

Measured by T3b's hour (`t3b-1`, `E06-log.md`): about 5 100 samples/s; a gate 18 s to 162 s, a
ladder of 250 per kind 50 s to 302 s, the final ladder 391 s; at a gate every 5 M samples the
gates and ladders took 21.4 % of the run, the ladders three quarters of that. The estimates below
stay for what is not measured yet.

- **Training:** about 8 500 samples/s end to end with E05's pool and evaluations (T8c); the
  pool's frozen passes cost a third of the updates per minute, which §8's live set bounds.
- **Checkpoint:** about 75 MB (the network, Adam's moments, the league table), under a second on
  an SSD, every 5 minutes; snapshots 24 MB each, at most 16 kept; promoted networks kept.
- **Evaluation speed:** E05 played 2 000 games against Random in 82 to 140 s with 100 envs;
  500 envs should batch the network's passes better. Assume 15 to 30 games/s until measured.
- **A gate:** 500 to 1 300 pairs typically (1 000 to 2 600 games, 1 to 3 minutes); the cap 8 000
  games (4 to 9 minutes).
- **A ladder:** intermediate, 3 bots and 1 promoted × 1 500 games = 6 000 games (3 to 7
  minutes); the final one twice that.
- At a gate every 10 M samples (about 20 minutes), gates and ladders could take a quarter to a
  third of a run's time while most gates promote (early). The user's hour with gates
  (`configs/e06-t3b.toml`, T3b) measures it (the record's `progress.evaluation_minutes`); T4's
  configs set `gate_every` and `ladder_episodes` from it so evaluation stays under about 20 %.

### 10. The increments

Each with the user's checkpoint; only the next one refined (Q41's pattern).

- **Increment 1** (done 2026-10-20): T1, the user's review, then T2 in three small pull
  requests:
  - **T2a — Config and record:** done 2026-10-20 (`E06-log.md`).
  - **T2b1 — The full checkpoint and `--resume`:** done 2026-10-20 (`E06-log.md`; the user cut
    T2b there: the pending gate or ladder and `--init-from` wait for T3's gate, §3).
  - **T2c — C ABI v31:** done 2026-10-20 (`E06-log.md`).
  - Moved to T5: the observations written in place. It needs no ABI change: `h3_batch_step`
    takes its buffers per call, so the binding can pass the rollout's row (the read-only
    attributes stay); worth its code only if a profile shows the copy.
- **Increment 2** (done 2026-10-20, `E06-log.md`): T3a (the held-out sets v1 in Python and
  `h3 ai eval --set`, the ladder on them, the bots' table, the bots' fallback branches tested),
  T3b (the gate, the promotion, the ladder per promotion, their state in the checkpoint and a
  pending one at the resume, `--init-from`; the user one pull request). The hour on the user's
  machine with gates (`python/configs/e06-t3b.toml`) measures §9's evaluation share; its result
  sets T4's cadences.
- **Increment 3** (done 2026-10-22, `E06-log.md`; AI-Q9: γ 1.0): T4a (the league of §8; with
  `configs/e06-t4.toml`: a gate every 10 M samples and ladders of 250 per kind, about 12 % of a
  run by `t3b-1`'s costs), T4b (two league runs of 60 M samples, about 3.5 hours each, at γ 0.995
  and γ 1.0 against the stalling; one of them stopped and resumed for real).
- **Increment 4** (refined 2026-10-22; T4b's evidence: the MLP at γ 1.0 still climbs slowly, the
  learning rate at its floor at the budget; done 2026-10-23): T5a (the entity network of §11
  and the visualization command), T5b (`t5-1`: stalled), T5c (the truncation penalty), T5d
  (`t5-2`, `t5-3`: the entity network parked, the MLP with the penalty near `t4-2`'s 60 M in
  20 M).
- **Increment 5** (refined 2026-10-23, the user: T7 first, no run meanwhile; done 2026-10-23,
  `E06-log.md`): T7a (`shared/`: `h3-sim` may depend on `h3-battle-ai`), T7b (the system
  actor's seat, C ABI v32; §12), then at the checkpoint T7c (the trainer's neutral share, the
  neutral held-out set, the system actor in the ladder) and T7d (`t7-1`).
- **Then** T5's rest (AI-Q7 (b), the mana price, the batch size and threads from T9c's table,
  the observations in place), T6, T8 (only if PPO plateaus), T9. Built beside T7d (the user,
  2026-10-23): the mana price's preparation (the evaluation at reward v1's settings,
  `configs/e06-mana.toml`), then AI-Q7 (b)'s batch output (part 1, C ABI v33: the acting side's
  previewed kills per attack and shot entry, an option of the batch; part 2, the network reading
  them beside the entry mask and an A/B config, after the checkpoint).

### 11. The entity network — built (T5a)

Decision 5: v21 read as entities, in Python, with no encoding change. `[network] kind` chooses
`"mlp"` (E05's, the default: its weights and files stay as they are) or `"entity"`; both share
one interface (`h3env.train.policy.Network`: `encode`, the value, the four heads' logits; `act`,
`evaluate` and `frozen` on top), and a saved network's config names its kind (a file without the
key is an MLP). The comparison with the other architectures the user saw: `E06-log.md`,
"Increment 4's refinement".

- **Layout** (`h3env.train.layout`, pinned by an `h3-sim` test against `observation.rs`): the
  globals (38), the two public hero parts (14 each), the own hero's private part (578), 16 hex
  planes of 255, 42 stack slots of 99 (8 882); in a slot, present (0), the creature's handle
  (3), x and y (12, 13), active (18).
- **Tokens** (width `width`, 128): one per stack slot, `symlog` of its 99 features through a
  linear layer plus learned embeddings of its creature's handle (clamped to 255), x (31) and y
  (15), an empty slot masked out of attention; the global token (the globals and both public
  hero parts, 66); the own-hero token (578); the field token: each hex's 16 planes (`symlog`)
  through a shared 16 → 16 layer plus a learned per-hex embedding, ReLU, the 255 hex vectors
  flattened through a linear layer.
- **Encoder:** `layers` (2) pre-norm transformer layers, `heads` (4), feed-forward twice the
  width, over the 45 tokens; the context is the global token's output, the active stack's (zeros
  without a present active one) and the mean of the present stacks', through a linear layer and
  ReLU.
- **Heads** (the logits in action encoding v13's order): the value and the 15 fixed entries
  (Wait, Defend, End tactics, Retreat, Surrender, the two casts, the 8 wall shots) from the
  context; Move h from hex h's vector, the 24 attacks, 2 shots and the heal of stack s from its
  token, each scored against the context (an additive pointer: a linear layer of the key plus
  one of the query, ReLU, a linear layer to the logits); the spell head as the MLP's; the target
  heads point at "none", the 255 hexes and the 42 stacks the same way, their query the context,
  the cast and the spell's embedding (the second target's also the target's embedding).
- **Cost** (measured in T5a, `E06-log.md`): 1.19 M parameters against the MLP's 5.95 M, 15.5 M
  multiply-adds per decision against 5.5 M, about five times the MLP's time on the sandbox's CPU;
  on the GTX 1070 `t5-1` trained at about 1 600 samples/s (the MLP's `t4-2`: about 5 000).
- **T5b's lesson** (`t5-1`, `E06-log.md`): a move's logit reads its hex's own features and the
  context, so "to my edge" is easy and "toward that stack" hard; both sides learned to hide and
  45 % of self-play stalled to the step limit. The user's remedy is the incentive, not features:
  `ppo.truncation_penalty` (T5c).
- **Parked** (T5d, 2026-10-23): with the penalty the entity network stopped stalling but stayed
  behind the MLP per sample from 10 M on (entropy 3.4 to 3.1, the MLP's 2.7 to 1.9; explained
  variance 0.52 against 0.78) at 3.4 times its cost. It stays buildable (`kind = "entity"`);
  a revival would start from the move head's view of the stacks.
- **Visualization** (`python -m h3env.train.network`, no dependency beyond torch): a config's
  network (`--config`, `--set`) or a saved file's: each module's parameters and output shape on
  an observation, the multiply-adds of its linear layers and attention, the time of a forward
  and a training pass per sample (`--time`), and a Mermaid diagram (`--mermaid <file>`) of the
  data flow, which GitHub renders.

### 12. The system actor in training (T7, T6b) — T7b, T7c, T6b built

Decisions 1, 7 and 9: the neutral kinds (a wandering monster, a pile's or a garrison's guards, a
neutral town) join the training with the system actor as their defender (E23's policy of the
original's computer, complete for a side without a hero since E23 T4c). Refined with the user on
2026-10-23: T7a, T7b now; T7c and the run (T7d) after T7b's checkpoint.

- **The seat** (T7b; ADR-0025 point 5, ADR-0027): `Policy::System` plays
  `h3_battle_ai::choose(state, ruleset, side, rng)` from the seat's stream, inside the call as
  the bots do. It reads the whole state (a trusted host, ADR-0027 point 4) and plays its side as
  the computer's whatever `human` says (point 8); the batch's `human = [true, true]` stays
  (ADR-0025 point 6). Its commands always apply (it validates them; the batch's property test
  plays it on both sides of every synthetic scenario, a hero's side included). A hero's side
  retreats only with the setup's adventure facts (E23 T6, BAT-AI-024, ADR-0027 point 14), which
  the encounters leave at their defaults: it never retreats here. Once the sampler fills them,
  an escape would end the episode inside the call, and ADR-0025 point 7 then needs a decision.
- **The neutral kinds' defender** is `Internal(System)` (A12; it was `Random`): the caller always
  plays the attacker of a neutral encounter, and an evaluation's seats still replace it.
- **The weights stay where they are** (the user, 2026-10-23): the distribution's defaults keep
  the neutral kinds at 0, so the held-out sets (v0, the report set v1, the gate set) keep their
  episodes and every recorded result stays comparable. A run draws the neutral share by its own
  `[env] kinds` (A1's shares: 780, 20, 100, 50, 25, 25); no new encounters version.
- **C ABI v32** (T7b): the seat `H3_SEAT_SYSTEM` (4) and `h3_system_policy_version()`
  (`h3_battle_ai::POLICY_VERSION`, ADR-0027 point 12), which `h3env`'s `versions()` adds to a run
  record's identity (`system_policy`; a record gains a key, the schema stays 5). `h3env`'s seat
  name `"system"`; `h3 ai eval --policies` names `system` (not in its default list yet). The
  encodings are unchanged.
- **The trainer** (T7c): a neutral episode has one learning side, the attacker, and no league
  member (`Trainer.assign` draws none for a neutral kind; before T7c a member would have taken
  one side of a neutral battle: the defender's, which the system actor plays, so the member never
  moved but scored, or the attacker's, so nothing trained). The update record's `by_kind` splits
  the self-play episodes' rates (the attacker's wins, the truncations) by kind, and the record's
  curve sums them per gate interval. The gate stays on the gate set v1 (the default kinds).
- **The neutral set v1** (T7c; `neutral-v1` in `evaluate.py`, `h3 ai eval --set neutral-v1`,
  `python -m h3env.train.ladder --set neutral-v1`): one stratum per neutral kind (batch seed
  `NEUTRAL_V1_SEED + k`, `0x5EED_E7A1_0003_0000`; the kind one-hot; the first episode of each of
  500 envs), weighed by A1's shares (780 : 20 : 25, `share` in `encounters-v1.toml`), the
  policy in the attacker's seat against the system actor: 1 500 games, ±1.8 points at 0.95, ±3.6
  at 0.75 (the wandering stratum is 94.5 % of the weight). `h3 ai eval` plays every policy, the
  system actor too, in the same seat on the same episodes (`bots-neutral-v1.json`); decision
  7's neutral target reads off it (the value efficiency above every ladder bot's and the system
  actor's own).
  Its seeds are held out like the other sets'. Every ladder plays it (`ladder.ladder_neutral`,
  true by default; as many per kind as the report set's), its row the ladder record's `neutral`.
- **The system actor on the ladder** (T7c, decision 12): an opponent of `evaluate.OPPONENTS`, so
  `ladder.ladder_opponents` and `h3 ai eval`'s default list name it; `h3 ai eval --json` schema
  2 names the engine's `system_policy` and a set's fixed `defender`; both bots' tables
  regenerated. The bots on the report set (`bots-report-v1.json`): the system actor beats
  Greedy 0.653, Aggressor 0.681, Random 0.875; on the neutral set it attacks best (value +0.285
  against Greedy's +0.223, Aggressor's +0.216, Random's −0.056; `E06-log.md`, T7c).
- **The run** (T7d; `configs/e06-t7.toml`, `t5-3`'s settings): the census shares would put
  82.5 % of the episodes in the neutral kinds, each with one learning side; the user chose a
  third neutral with T7c's numbers (2026-10-23: `[78, 2, 100, 50, 25, 3]`, the census's neutral
  shares a tenth). `t7-1` (`E06-log.md`, T7d): the report set as `t5-3`'s, the neutral set
  0.972 at value +0.245, flat from 10 M, below the system actor's +0.285.
- **The system actor as an opponent in the player kinds** (T6b; increment 6's lever, the user,
  2026-10-26: decision 9 amended, T6a's evidence: the agent never met the opponent that beats
  it, and loses mostly defending): `[league] system_share` (0 by default, in thousandths) of the
  player kinds' episodes have the system actor on a uniformly drawn side inside the batch, the
  learner on the other, no league member. The batch draws it: `ppo.episode_entries` splits the
  config's kinds into the neutral kinds' entry, the player kinds' own (both seats external) and
  one per side with `h3_batch_add_encounters`' seats (`System` there, `External` the other),
  weighed so that every kind keeps its share of the episodes; the trainer reads an episode's
  entry (`batch.scenario`), so no ABI change. It needs `[env] kinds` (the defaults' weights are
  the sampler's). The update record's `system` (and the curve's, per gate interval) gives the
  learner's mean score against it by its side; such episodes leave self-play's rates
  (`attacker_won`, `by_kind`). Without the key a batch has the one entry as before, so `t7-1`'s
  runs are unchanged. The ladder's system row then measures a training opponent (decision 12
  keeps it on the ladder); the gate set, the other rows and the neutral set stay comparable.
  `configs/e06-t6b.toml`: `t7-1`'s settings with a quarter (the A/B, T6c).
- **Every side stays human** (the user, 2026-10-26, asked at T6a's checkpoint): the batch keeps
  `human = [true, true]` for the system actor's side too (ADR-0025 point 6 unchanged), so
  BAT-QUIRK-27's let-off of a computer side's bad-morale freeze (BAT-MOR-004, since E23 T7) is
  never played in training or on the held-out sets.

### 13. The kill preview in the network (T6d) — built, off by default (T6e: no measurable gain)

AI-Q7 (b) part 2 (decision 6; the user, 2026-10-26, after `t6b-1`): the batch's previewed kills
(part 1, C ABI v33: each legal attack and shot entry's least and greatest kills, the engine's
BAT-DMG-011 preview, what the original shows a player) reach the MLP's entry head; the
encodings are unchanged.

- **The bias:** a legal strike entry's logit gains `φ(f) · u(h)`. `f` holds four features: the
  least and greatest kills (`symlog`), the share of the target stack their mean is (clipped to
  [0, 1]), the hit points it is (`symlog` of the mean times the target's full hit points; the
  target's slot is fixed by the entry: 24 attack entries, then 2 shots, per slot). `φ` is a
  network of two layers (4 → `embed` → `embed`) shared by the entries; `u` a linear map of the
  trunk's output to `embed` (initialized near 0, so the network starts as the plain MLP's), so
  the state decides how much a kill counts. Only the legal entries' features are computed (a
  row has a few dozen of its 1 092 previewed entries; the dense form would be 0.5 GB a
  minibatch). The value and the cast heads read no preview, so a truncation's bootstrap from
  `final_obs` (no preview there) is as before.
- **The plumbing:** `[network] preview` (false by default; the MLP's: `resolve` refuses it with
  the entity network). Every batch such a network plays writes the preview: the trainer's, the
  gate's (when either network reads it), the ladder's (when any of its networks does);
  `Policy.act` refuses a batch without it. The rollout keeps it in half precision (whole kills
  up to 2 048 exact; 143 MB at 256 envs × 128 calls, beside the observations' 1.2 GB) and the
  update recomputes the entry's log-probability from it. A saved file's config names
  `preview`; one without it reads none.
- **The run** (T6e, the user's machine): `configs/e06-t6d.toml`, `t7-1`'s settings with the
  preview, against `t7-1` (one change; `t6b-1` showed the system actor as an opponent changes
  nothing measurable). **Result** (E06-log T6e): the ladder unchanged; the targeting better, but
  the agent strikes less and wanders (moves, the tactics phase's placements), its battles
  longer; AI-Q7 closed (the user, 2026-10-26).

### 14. The search (T8) — designed (T8a), its plumbing built (T8b), measured (T8c): no gain, closed

Decision 2's last increment (the user, 2026-10-26, after T6e: the plateau taken as shown). The
seam (the user): T8a this design; T8b the plumbing; T8c a decision-time search with `t7-1`'s
network measured, no training change; T8d expert iteration only if T8c gains. **Determinized
from the start** (the user): the searcher is a legitimate agent, it reads only what its side's
observation tells (ADR-0025 point 12a). The tree in Python first (the user); Rust
only if T8d needs the throughput.

- **The agent.** At each of its decisions it builds W *worlds* (8) of the battle from its side's
  knowledge, runs N simulations (16 to 64) over them and plays the root's choice. The network is
  a promoted MLP's without the preview (a single env writes none): its entry and cast heads are
  the prior, its value head a leaf's value. It plays both sides inside the tree, each from its
  own observation of the world; never the real opponent's policy (the system actor, Greedy).
- **A world** (C ABI v34, `h3_batch_determinize(batch, env, viewer, seed) -> H3Env *`): a copy
  of the env's battle in which, for the viewer, everything hidden (`h3_battle::observe`'s list)
  is replaced from the world's seed:
  1. the other side's hero's private part (its secondary skills' values, the war machines'
     levels, the spell book, masteries, spell bonuses, resistance, duration bonus, defences,
     negates, the opening spells, Eagle Eye; funds and the surrender price go unread, surrender
     masked) from the encounter sampler: a hero of the observed hero type (public), its level and
     draws by the table's week prior, of R (8) candidates the one whose primary skills are
     nearest the observed (L1, ties to the first drawn); then made consistent with the public:
     the spells the public state shows it cast added to its book (T8b: what the state keeps of
     them), no book when it carries none, a war machine's skill at least Basic where it commands
     the machine and none where it does not. Its public part (primary skills, mana and its
     maximum, the round's casts) stays as observed;
  2. the other side's hidden spell obstacles (BAT-MAG-031: the cast public, the hexes not)
     placed again by their spell's rule, and a spell a random caster the viewer does not control
     drew (BAT-MAG-049) drawn again by its rule, from the world's stream;
  3. the battle's stream re-seeded, so no roll of the true stream is known. The defender's
     Escape Tunnel stays as it is (retreat is masked).

  A world reads nothing hidden: two states that differ only in what the viewer may not know give
  the same world for a seed (T8b's test). Items 2 and 3 and writing a guessed private part live
  where the hidden part is defined, `h3-battle` (`BattleState::determinized(viewer, guess,
  seed)`, `reseed`; trusted tooling beside `observe`, `cross-lane` in T8b with the user's
  consent); the guess in `h3-sim` (the sampler's hero draws).
- **The search** (`h3env.search`): Gumbel AlphaZero (Danihelka et al., "Policy improvement by
  planning with Gumbel", ICLR 2022), fit for a pretrained prior, few simulations and many
  actions. At the root, Gumbel-top-k (16) full actions: entries by Gumbel-top-k over the entry
  logits, a cast's heads by their Gumbel-perturbed argmax in order; sequential halving over them,
  the action of the largest `g + logits + σ(q̂)`, the completed values rescaled by their least
  and greatest over the node's candidates (T8c: a fixed scale left the prior alone). Below the
  root, the deterministic selection of the completed-Q improved policy. Chance and the hidden
  part are averaged open-loop: the tree is keyed by action histories; a simulation takes a world
  (round robin over W) with a fresh stream from the agent's own, forks it (`h3_env_fork`) and
  replays its path (`h3_env_step`); a node keeps its first evaluation's prior and selects among
  the actions legal in the simulation's world, renormalized. Values are the side to move's:
  reward v1 at a terminal, the value head at a leaf.
- **Batching:** the agent searches every env it moves at once: each simulation round evaluates
  one leaf per env in one network call (the GPU's), so a decision costs N calls for the batch.
- **Cost** (measured in T8b): a simulation steps its path through ctypes (a few steps) and reads
  an observation and masks at its leaf; a world costs R hero draws (an encounter's hero about
  50 µs, E05 T5b2). T8c's report set (3 000 battles per opponent, about 28 decisions each) at
  N 32 is about 2.7 M simulations: tens of minutes on the user's machine; the sandbox plays a
  subset.
- **T8b, the plumbing — built** (E06-log T8b): C ABI v34 (with T6f's observation v22),
  `determinized`, `reseed`, the guess, `h3env.search` with `--time` and `--report`. Beyond the
  above: the root's candidates are the entries the true battle allows, a chosen cast it refuses
  gives way to the network's most likely action; a world in which another side moves after the
  same actions evaluates its position without a node. Gap: AI-STUB-1 (the hidden obstacles'
  number and value).
- **T8c, the measurement — done** (E06-log T8c; the sandbox, `order-1`'s best): the search
  beside the sampled policy on the same episodes (`python -m h3env.search --report --best
  <best.pt> --opponent system --simulations 64 --json …`; `--set neutral-v1`; `--per-kind`: the
  sets' 500 by default): no gain against the system actor at N 16 and 64 nor on the neutral set
  at N 64. T8d only if the search gains beyond the intervals: not indicated.
- **T8d, expert iteration** (designed after T8c): the search's improved policy and value as the
  trainer's targets. **Not built** (the user, 2026-10-27, increment 7's checkpoint): E06 closes
  without it; a search in Rust trained on its own choices is E25's candidate, with AI-STUB-1.

### 15. The turn order as an input (T6f) — built, off by default (T6g: no measurable gain)

The turn-order probe (E06-log) showed the trunk does not know which stack acts next. The user
(2026-10-26): the engine's own order in the observation, v22, before T8b.

- **The order:** `h3_battle::turn_order` (`turn.rs`, sharing `next_active`'s key): the rest of the
  round's turns as they come (no wait, morale or death foreseen), then the next round's.
- **The encoding:** per stack slot its place in the rest of the round (1: the active stack), its
  place among its side's turns there (1: its side's next), its place in the next round; appended
  after v21's layout, which stays its prefix.
- **The network:** `[network] order` (the MLP's): the trunk reads the whole observation, else
  v21's prefix (every network saved before v22). The entity network reads v21's slots as before.
- **The run** (T6g, `order-1`, E06-log): `t7-1`'s ladder and play; the trained trunk still
  does not represent the order it reads (the probe again).
