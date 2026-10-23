# E06 — Design note (working detail; not binding; pruned by the tasks that implement it)

Companion to `docs/epics/E06-battle-ai.md` and ADR-0026 (E06 T1; accepted by the user 2026-10-20,
every point as proposed: `E06-log.md`, T1). The training pipeline on the user's machine: a run, its
config and records (§1, §4), its budget and schedule (§2), the resumable state (§3), the held-out
sets (§5), the evaluation gate (§6), the ladder (§7), the league (§8), the costs (§9) and the
increments (§10). Each task turns its section into code and docs and deletes it here. Starting
point: E05's trainer (`python/h3env/train/`, `E05-log.md` §8 and §9).

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
removed at the next start. Built (T2a, T2b1, T3b, T4a): `config.toml`, `metrics.jsonl`,
`record.json`, the atomic writes, `checkpoint.pt` (the whole state, `Trainer.state`),
`promoted/<n>.pt` and `best.pt` (each promotion's network with its index, how, run and parent),
`league/<id>.pt` (each snapshot's network; a retired one's deleted after the next checkpoint).

### 1.2 The config file — built (T2a)

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

### 1.3 Commands

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

## 2. The budget and the schedule, in samples — built (T2a)

`samples` (the budget: the first update boundary at or past it), the learning rate (`linear` to
`lr · lr_floor` over `lr_horizon`, fixed at the start; or `constant`), the league's snapshots and
live draws (T4a) and the gates (`Cadence`, T3b; E05's evaluation against Random is gone) count
samples (`run.py`). Wall time sets only the checkpoints (T2b1).

## 3. The resumable state — built (T2b1, T3b, T4a)

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

## 4. The record — built (T2a, T2b1, T2c, T3b, T4a)

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

## 5. The held-out sets — built (T3a)

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

## 6. The evaluation gate — built (T3b)

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

## 7. The ladder — built (T3a, T3b)

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

## 8. The league — built (T4a)

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
neutral episode draws none (§12). Still to come: exploiters (decision 9) only if a check shows the agent exploitable: a fresh learner trained
against the frozen best alone for a fixed budget; if it beats the best at least 0.70 on the gate
set, main exploiters join (designed then, T6).

## 9. Costs on the user's machine (estimates; T2 to T4 measure them)

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

## 10. The increments

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

## 11. The entity network — built (T5a)

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

## 12. The neutral share (T7) — T7b, T7c built

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
