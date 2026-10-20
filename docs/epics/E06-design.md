# E06 — Design note (working detail; not binding; pruned by the tasks that implement it)

Companion to `docs/epics/E06-battle-ai.md` and the ADR draft
`docs/lanes/ai/adr-drafts/training-pipeline.md` (E06 T1, 2026-10-20). The training pipeline on the
user's machine: a run, its config and records (§1, §4), its budget and schedule (§2), the
resumable state (§3), the held-out sets (§5), the evaluation gate (§6), the ladder (§7), the
league (§8), the costs (§9), the increments (§10) and the points for the user's review (§11). Each
task turns its section into code and docs and deletes it here. Starting point: E05's trainer
(`python/h3env/train/`, `E05-log.md` §8 and §9).

## 1. A run

### 1.1 Its directory

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
removed at the next start.

### 1.2 The config file

TOML (stdlib `tomllib`, no dependency; the repo's data are TOML), read once. Defaults stay in code
(one dataclass per section, as E05's `Config`); the file states what differs, `--set
ppo.lr=1e-4` overrides one key, an unknown key is an error. The resolved config (every value)
goes into the record, so a run never depends on the defaults of a later commit. Sections:

```toml
name = "e06-league-1"   # the record's id; the directory's name by default
seed = 1
samples = 200_000_000   # the budget (§2)

[env]        # envs, threads (0: the cores), max_steps, kinds ("default" or six weights),
             # currency, mana_price
[network]    # kind = "mlp" (T5 adds "entity"), hidden, embed
[ppo]        # rollout, gamma, lam, lr, lr_schedule ("linear" | "constant"), lr_floor,
             # lr_horizon (samples; 0: the budget at the start), epochs, minibatches, clip,
             # value_coef, entropy_coef, max_grad_norm
[league]     # self_play, snapshot_every (samples), size, live, pfsp_power, decay (§8)
[gate]       # every (samples), elo0, elo1, alpha, beta, max_pairs (§6)
[ladder]     # opponents, episodes (per kind), promoted (§7)
[checkpoint] # minutes
[device]     # device ("auto" | "cuda" | "cpu"), torch_threads
```

Committed configs live in `python/configs/` (the runs the epic plans, so a command line names a
file, not twenty options).

### 1.3 Commands

```
uv run --group train python -m h3env.train --config configs/e06-t3.toml --out runs/t3-1
uv run --group train python -m h3env.train --resume runs/t3-1 [--samples 400_000_000]
uv run --group train python -m h3env.train --config configs/e06-t4.toml --out runs/t4-1 \
    --init-from runs/t3-1/best.pt
```

- `--resume` reads the config from the checkpoint. It may change only what does not change the
  training: the budget (`samples`, extended or cut), `env.threads`, `[device]`,
  `checkpoint.minutes`. Any other change is refused with the keys named: that is a new run.
- `--init-from` starts a new run (fresh optimizer, counters, league) from another run's network;
  the record names the parent (its run id and promoted index), and the parent's network is the
  new run's best #0, so its gates continue the lineage (§6).

## 2. The budget and the schedule, in samples

A sample is a learner decision that trained (E05: an update holds about 24 000 of a rollout's
32 768 rows with the pool). Every cadence that changes the training counts samples: the budget,
the learning rate, the league's snapshots, the gates. Wall time decides only the checkpoints and
the console, which change nothing (§3.2). The run stops at the first update boundary at or past
the budget.

- **The learning rate:** `linear` decays from `lr` to `lr · lr_floor` over `lr_horizon` samples,
  then stays; `lr_horizon = 0` resolves to the budget at the run's start and is frozen there
  (extending the budget on resume continues at the floor; a new horizon is a new run).
  `constant` for the long league runs if the decay proves unhelpful (T4).
- E05's `minutes`, `updates` and `eval_every` go; its periodic evaluation against Random is
  replaced by the gate (§6) and the ladder (§7).

## 3. The resumable state

### 3.1 What the checkpoint holds

`checkpoint.pt`, `format = 1`, a dict of:

- the resolved config; the run's id, its parent, its segments (§3.4);
- the counters: samples, updates, episodes, decisions, seconds trained;
- the network's and the optimizer's state (Adam's moments: about 72 MB at 6 M parameters);
- the random generators: the sampling generator (torch, on the device), the league's (numpy),
  torch's global CPU and CUDA states; the update's minibatch order is drawn from
  `default_rng(updates)` as in E05, so it needs none;
- the league: each member's id, kind (snapshot, promoted, later the system actor), file, birth
  in samples and its decayed score counts (§8); the next snapshot in samples;
- the gate: the best's promoted index, the promoted list, the next gate in samples, `pending`
  (a gate began after this save and has no result yet, or a promoted network's ladder has none);
- the records' position: `metrics.jsonl`'s length in bytes at the save.

Not saved: the batch (its envs' battles live in the engine; the episodes in flight are discarded,
decision 4), `SeatBook` and the open transitions, the rollout's partial calls. A checkpoint is
taken at an update boundary only, where no transition is part of a pending update.

### 3.2 When, and how

- Every `checkpoint.minutes` (5) of wall time at the next update boundary, before each gate
  (`pending = true`), after a gate's result, at a requested stop and at the end. At E05's rate an
  update takes about 4 s on the user's machine, so a crash loses at most about 5 minutes.
- **Atomic writes:** into `<file>.tmp` in the same directory, flush, `os.fsync`, close, then
  `os.replace` (atomic on one volume on Linux and Windows). Windows refuses the replace while
  another process holds the target open (a ladder reading `best.pt`): retry five times over a
  second, then keep the old file, report it and go on; the run never dies of it.
- **Ctrl+C:** the first SIGINT (Windows delivers it to Python too) sets a flag. A rollout in
  progress stops at its next call and is discarded (it changed no weights); an update in progress
  finishes (seconds; a half-applied update would be saved as no state the run ever had); a gate or
  a ladder stops at its next batch call, the save before it standing (`pending`). The run then
  saves and exits with a `stop` record. A second SIGINT raises at once (losing what the first
  would have saved). A closed console, a shutdown or a crash fall back on the last periodic save.

### 3.3 Resume

`--resume` loads the checkpoint, truncates `metrics.jsonl` to the saved length (the dropped
records were of updates the resumed run plays again; their count goes into the `segment` record),
restores the network, the optimizer, the generators and the league, opens a **new segment** and
creates a fresh batch:

- **Segment seeds:** segment 0's batch seed is `seed`; segment s's a 64-bit mix of (`seed`, s)
  (SplitMix64, written in the record), so a resumed run does not replay the episodes it already
  trained on. Every segment's batch seed is checked against the held-out sets' (§5.3).
- **A pending gate or ladder** (the stop fell inside one) runs first, again, on the saved
  candidate or promoted network.
- The run is reproducible from its records (config, segments, seeds), not bitwise equal to an
  uninterrupted one (decision 4; torch on a GPU is not bitwise anyway).

### 3.4 Tests (T2b)

The lifecycle (config, records, atomic writes, segments, the truncation, the disjointness check)
is a torch-free module, `h3env.train.run`, tested in the sandbox; the trainer's state needs torch
(the `train` group, absent in the sandbox while its network is not **Custom** with
`*.pytorch.org`: the lane HANDOFF's trap), tested where torch is. Cases: a tiny run (8 envs,
rollout 8, hidden 16, CPU) stopped by the flag after its third update resumes to its budget, the
network, the optimizer and the generators equal to the saved ones at the load, the metrics'
update numbers strictly increasing and the segment record present; a run killed hard
(subprocess, SIGKILL) during an update resumes from the last save and ignores the `.tmp`; a
resume that changes `ppo.lr` is refused; a pending gate or ladder runs at the resume.

## 4. The record

### 4.1 `metrics.jsonl`

One JSON object per line with a `type`: `start`, `segment` (index, batch seed, the dropped
records), `update` (E05's fields: samples, episodes, rates, losses, entropy, KL, clip fraction,
explained variance, samples per second; plus the league's live set), `snapshot`, `gate` (§6),
`promotion`, `ladder` (§7), `stop` (requested, budget), `end`.

### 4.2 `record.json` and the committed records

The summary, rebuilt from the state and `metrics.jsonl` at each gate, ladder, stop and end:

- **Identity:** the run id, parent, the resolved config, the commit and whether the tree was
  dirty, the C ABI version, the observation and action encodings' versions, the encounter
  table's version, the content hash of each of the batch's rulesets (T2c), torch and CUDA
  versions, the device's and the CPU's names, the OS.
- **Progress:** the segments (batch seed, samples at start and end, wall time, how it ended);
  per gate interval, the means of the update records (losses, entropy, KL, explained variance,
  samples per second, the training rates) as a short curve; the totals.
- **Evaluation:** every gate (candidate, pairs, LLR, the result), every promotion, every ladder
  (§7) with its intervals; the league's member table at the end.

About 20 to 50 KB for a run of hours. **Committed** (decision 11) as `python/records/<run>.json`,
a copy of the final `record.json`; `python/records/README.md` gives the schema and the lineage
(which promoted network is the current best). A test (torch-free) checks every committed record
against the schema version. Checkpoints and networks stay on the user's machine.

## 5. The held-out sets

### 5.1 The report set v1 (the ladder and the targets)

Stratified by kind and side, so every kind shows with a known count and the overall rate weighs
them as the distribution does: for each player kind k (hero, siege a hero defends, siege of a
garrison; `H3_ENCOUNTER_*` 2, 3, 4) a batch of 500 envs, batch seed `REPORT_V1_SEED + k`, the
kind's weight one-hot, the first episode of each env; each episode played twice, the agent
attacking and defending (E05's two batches). So 500 games per (kind, side), 3 000 per opponent.

- **The overall rate** at the default weights: w = (4, 2, 1)/7 over the kinds, the sides equal:
  `r = ½ Σ_side Σ_k w_k · r_k,side`. Its 95 % interval (normal, stratified variance
  `¼ Σ Σ w_k² r(1 − r)/n`):

  | Rate | Overall | One kind (both sides) | One kind and side |
  |---|---|---|---|
  | 0.95 | ±0.9 points | ±1.4 | ±1.9 |
  | 0.75 | ±1.8 | ±2.7 | ±3.8 |
  | 0.50 | ±2.0 | ±3.1 | ±4.4 |

  The targets' intervals (decision 7) as E05's evaluation had them (±1.8 at 0.8). A prefix (the
  first n envs of each kind) is itself a stratified set: 250 per (kind, side) gives ±2.5 at 0.75,
  for the intermediate ladders (§7).
- The sampled policy (decision 8), its generator seeded per (set, kind, side) alike for every
  checkpoint: two networks meet the same episodes and, as far as their play agrees, the same
  draws (common random numbers).
- The neutral kinds join as report set v2 with T7 (the player's seat only).

### 5.2 The gate set v1 (the SPRT)

The default distribution (kinds at their weights), batch seed `GATE_V1_SEED`, 500 envs × 8
episodes: pair j is episode ⌊j / 500⌋ of env j mod 500, 4 000 pairs at most, each played twice
(the candidate attacking, then the best). Separate from the report set, so the networks the gate
selects are not measured on the episodes that selected them.

### 5.3 Disjointness without E05's growing set

E05's trainer kept every training episode's seed in a set and compared it with the evaluation's;
over hours it grows by millions and would have to be checkpointed. Instead the held-out side is
fixed and small: the run computes the episode seeds of every held-out set (`episode_seed(batch
seed, env, episode)`: v1's 1 500 + 4 000, E05's 1 000) once, refuses a segment whose batch seed is
a held-out set's, and checks each training episode's seed (`episode_seed[done]`, and the reset's)
against them: a hit stops the run with an error. No state; the same guarantee (disjoint unless
two 64-bit hashes meet). The ladder refuses a network trained on a held-out batch seed, as E05's
does for `EVAL_SEED`.

E05's set (`EVAL_SEED`, 100 envs × 10 episodes, the default kinds) stays as **v0**, selectable in
`h3 ai eval` and the Python ladder, so E05's numbers can be compared; v1 becomes both tools'
default (T3a). The seeds' constants are defined in Python and in `h3-cli`'s `src/ai/eval.rs`,
pinned equal by a test as `EVAL_SEED` is.

## 6. The evaluation gate

- **When:** every `gate.every` samples (10 M, about 20 minutes at E05's rate), the learner's
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
  a guard follows only if it happens (§11).

## 7. The ladder

- **When:** for best #0 at the start, for every promoted network, and at the run's end for the
  final best on the full set. In the run's process, between updates (the GPU is the run's).
- **Opponents:** Random, Aggressor, Greedy (the engine's seats), E23's policy once its seat
  exists (T7), and the previous `ladder.promoted` (1) promoted networks of the lineage (both seats
  external). Earlier ones are compared through the records, not replayed.
- **Set:** the report set v1, the first `ladder.episodes` (250) per kind for an intermediate
  ladder, all 500 at the end and for any claim of a target.
- **Per opponent:** the win rate per (kind, side), per kind and overall with the 95 % intervals
  of §5.1, the value efficiency (mean reward v1, weighed alike), truncations, skipped episodes,
  games and seconds. One `ladder` record and the record's ladder table.
- **The bots' table** on the report set: `h3 ai eval --set report-v1 --json <file>` (T3a) plays
  every pair of bots on the same episodes; the bots are deterministic and Random draws from the
  episode's seat stream, so the table is a function of the engine: committed once as
  `python/records/bots-report-v1.json`, regenerated when the bots move (the lane's trap: the
  battle's strike stages).
- **Ratings:** Glicko-2 stays in one place, `h3 ai eval` (`src/ai/rating.rs`). It is a report,
  not part of the promotion or the targets; when one is wanted, `h3 ai eval --games <file>`
  merges the network games Python wrote (`ladder` records) with the bots' (optional, T3a).
- **The targets** (decision 7) are judged on the lineage's latest promoted network's full ladder:
  each point estimate at or above its target, the interval reported. Never the best of many
  checkpoints on the same set (that selects on its noise).

## 8. The league (T4)

Prioritized fictitious self-play (AlphaStar's PFSP) over the learner's past, grown from E05's pool:

- **Members:** a snapshot of the learner every `league.snapshot_every` samples (1 M; E05: about
  600 000), every promoted network, and in T7 the system actor (a member that plays only the side
  its encounter gives it). Heuristic bots never train (decision 9).
- **An episode's opponent:** self-play with probability `self_play` (0.5), else a member on a
  uniformly drawn side (E05's rule: only the learner's decisions train).
- **Which member:** each member m carries the learner's score against it, `p_m = (w_m + 1) /
  (g_m + 2)` (wins and games of the learner's training episodes against m, a draw half, both
  decayed by `decay` (0.99) per update: recent form; a new member at ½). At each rollout's start
  `live` (4) members are drawn without replacement with weights `(1 − p_m)^pfsp_power` (power 2:
  "hard", the opponents the learner still loses to); the rollout's pooled episodes draw among them
  by the same weights. So a call makes at most 1 + `live` forward passes, however large the
  league (E05's pool of 8 could make 9 and cost a third of the updates per minute).
- **Size:** at most `size` (16) members on the device (24 MB each); a new one replaces the member
  with the highest `p_m` among those with at least 200 games (the most beaten). A retired
  snapshot's file is deleted; promoted networks' files stay (the ladder and the lineage).
- **Exploiters** (decision 9) only if a check shows the agent exploitable: a fresh learner
  trained against the frozen best alone for a fixed budget; if it beats the best at least 0.70 on
  the gate set, main exploiters join (designed then, T6).
- Recorded: per update the live set; per gate the member table (id, kind, birth, `p_m`, games).

## 9. Costs on the user's machine (estimates; T2 to T4 measure them)

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
  third of a run's time while most gates promote (early). T3b measures it and sets `gate.every`
  and `ladder.episodes` so evaluation stays under about 20 %.

## 10. The increments

Each with the user's checkpoint; only the next one refined (Q41's pattern).

- **Increment 1** (now): T1, the user's review, then T2 in three small pull requests:
  - **T2a — Config and record:** `h3env.train.run` (torch-free: the TOML config with defaults in
    code and `--set`, the run directory, typed `metrics.jsonl`, `record.json`, atomic writes,
    the disjointness check of §5.3 against v0), the budget and the learning rate in samples
    (§2), `python/records/` with its README and the schema test, `python/configs/` with the
    first config. E05's `--minutes` options go.
  - **T2b — The full checkpoint and `--resume`:** §3 (the state, the cadence, Ctrl+C, segments,
    the truncation, `--init-from`) and its tests (§3.4).
  - **T2c — C ABI v31:** the record's engine identity (the batch's rulesets' content hashes, the
    observation and action encodings' versions, the encounter table's version) and
    `h3_batch_create`'s N range-checked (the binding already checks it); `h3env`'s
    `ABI_VERSION`.
  - Moved to T5: the observations written in place. It needs no ABI change: `h3_batch_step`
    takes its buffers per call, so the binding can pass the rollout's row (the read-only
    attributes stay); worth its code only if a profile shows the copy.
- **Increment 2:** T3a (the held-out sets v1 in Python and `h3 ai eval --set`, the ladder on
  them, the bots' table, the bots' fallback branches tested), T3b (the gate, the promotion, the
  ladder per promotion; an hour on the user's machine with gates).
- **Increment 3:** T4a (the league of §8), T4b (the first multi-hour run, its resume used for
  real).
- **Increment 4:** T5's experiments, each a recorded run: the entity network against the MLP,
  AI-Q7 (b), the mana price, the batch size and threads (T9c's table), the observations in place.
- **Then** T6, T7 (after E23's T2 to T4), T8 (only if PPO plateaus), T9.

## 11. Points for the user's review

1. TOML configs in, JSON records out; committed records in `python/records/`, configs in
   `python/configs/` (§1, §4).
2. On resume only the budget, threads, device and checkpoint cadence may change; anything else
   is a new run (`--init-from`) (§1.3).
3. The learning rate's horizon fixed at the start; an extended budget runs at the floor (§2).
4. Checkpoints every 5 minutes of wall time (they change no training); every training cadence in
   samples (§2, §3.2).
5. The report set v1: 500 per (kind, side), stratified; 250 per kind for intermediate ladders
   (§5.1). A separate gate set (§5.2).
6. The SPRT: elo0 0, elo1 20, α = β = 0.05, at most 4 000 pairs, no promotion at the cap, no
   second condition against the bots (§6).
7. The targets judged on the latest promoted network, point estimates against the targets
   (§7).
8. Ratings only in `h3 ai eval`, outside the promotion (§7).
9. The league: self-play 0.5, PFSP "hard" (power 2), 16 members, 4 live per rollout, a
   snapshot every 1 M samples (§8).
10. T2 in three pull requests (T2a to T2c); the observations in place moved to T5 (§10).
