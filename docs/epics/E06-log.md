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
  and lost, melee into a target with retaliations left, who struck first. `layout.py` names the
  columns it reads; `h3-sim`'s layout test pins them (after a Wait and a Defend; a swapped pair
  fails it).
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
