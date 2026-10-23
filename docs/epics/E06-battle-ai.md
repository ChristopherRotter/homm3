# E06 — Battle AI v1 and the local training pipeline v0

Status: **in progress**, refined 2026-10-20 (AI-Q8); increment 1 (T1, T2a, T2b1, T2c) done; increment 2 (T3a, T3b) done; increment 3 (T4a, T4b) done; increment 4 (T5a to T5d) done 2026-10-23; increment 5 (T7) refined 2026-10-23, T7a to T7c done · Lane: **ai** · Follows E05 (closed
2026-10-20) · Runs beside E23 (the battle lane: the original's battle AI, the system actor's
policy), whose neutral side opens E06's neutral share (decision 1)

## Goal

A battle agent that plays proficiently on every battlefield and in sieges, on the encounters the
game produces, trained on the user's machine by a pipeline that runs, records, evaluates and
promotes on its own: config-driven runs that survive an interruption, an evaluation gate with
automatic promotion, a league of past checkpoints, and proficiency measured against a fixed ladder
on held-out encounters.

## Decisions at the refinement (the user, 2026-10-20, AI-Q8)

1. **Start now, the neutral share later.** E06 starts with the kinds that have a player on both
   sides (a hero battle, a siege a hero defends, a garrison's siege: today's whole default
   distribution). The neutral kinds join in a late increment, once E23's policy plays a side
   without a hero (E23 decision 1, its T2 to T4); until then they weigh 0 (A12).
2. **PPO and a league first, search last.** E05's own PPO over the factorized heads is scaled up
   (longer runs, a league, a larger or structured network). An AlphaZero-style search (the fork
   with its random streams re-seeded, the hidden part guessed: the opponent hero's private part,
   hidden obstacles) is a final, experimental increment, **only if PPO plateaus below the
   targets** (decision 7).
3. **Compute: runs of a few hours** on the user's machine (GTX 1070, Ryzen 9 3900X), longer once
   required. The agent prepares configs and commands and reads the logs over the device link; the
   sandbox runs only short CPU checks.
4. **Runs can be interrupted and resumed** (the user): a run checkpoints its whole training state
   (network, optimizer, schedule, counters, the league, the random generators, the records'
   position) often enough that an unplanned stop (a shutdown, a crash, Ctrl+C) loses at most the
   time since the last checkpoint, and `--resume` continues it. A run's budget can be extended on
   resume. Episodes in flight at the stop are discarded; a resumed run is reproducible from its
   records, not bitwise equal to an uninterrupted one (torch on a GPU is not bitwise anyway).
   Nothing in the schedule depends on wall time alone (E05's learning rate decayed over wall-time
   progress: it moves to a budget in samples).
5. **Network:** observation v21 stays as it is. Besides E05's MLP, an entity network reads v21's
   stack slots and hex planes as entities (attention), in Python, with no encoding change.
6. **AI-Q7, option (b) as an experiment:** the kills of each attack and shot entry (the engine's
   preview, BAT-DMG-011) as features beside the entry mask, a new output of the batch, the
   encodings unchanged; kept only if it measurably helps.
7. **Proficiency targets** (exit criterion 5), on held-out encounters at the default weights,
   both sides weighed equally, the sampled policy: at least **0.95 against Random**, at least
   **0.75 against Aggressor and against Greedy**, a positive value efficiency (mean reward v1)
   against every ladder member. In the neutral share: in the player's seat against the system
   actor, a value efficiency better than every ladder bot's in the same seat on the same episodes,
   and better than the system actor's own in that seat once E23's hero part plays a hero side.
8. **Evaluation:** the sampled policy on a fixed held-out set (greedy play repeats itself, E05
   T8b). Promotion to the current best is automatic by a sequential test (SPRT, as chess engines
   gate their patches) of the new checkpoint against the current best, its bounds declared in the
   run's config.
9. **League:** prioritized fictitious self-play over a pool of past checkpoints (E05's pool grown:
   opponents drawn by how often they still beat the learner), self-play and, in the neutral share,
   the system actor. Exploiter agents only if a check shows the main agent exploitable. Heuristic
   bots never train (E05 decision 8).
10. **Reward v1 unchanged**, one experiment on the mana price (10 per spell point, provisional);
    retreat and surrender stay masked (ADR-0025 point 7). **Amended 2026-10-22** (the user, T5c):
    in training a battle cut at the step limit costs each learning side
    `ppo.truncation_penalty` (1.0) instead of a bootstrap, against stalling (`t5-1`); reward v1
    and the evaluation unchanged.
11. **Records:** every run's summary (config, commit, ruleset hash, ABI version, metrics, the
    evaluations) is committed; checkpoints stay outside git, on the user's machine.
12. **No new scripted bot:** the ladder is Random, Aggressor, Greedy, E23's policy as it arrives,
    and earlier promoted checkpoints.

Unchanged: increments with a user checkpoint each, only the current increment refined (Q41's
pattern); the user reviews ADRs and seams; token efficiency beats speed.

## Starting point (from E05)

- The batch (`h3_sim::batch`, C ABI v30), the encounter distribution v1, reward v1, the Python
  binding `h3env` (PyTorch 2.14, CUDA 12.6) and the bots: `docs/07-simulation-and-ai.md` §1.1.
- The trainer `h3env.train` (`E05-log.md` §8): an own PPO, an MLP over `symlog` of v21 (about 6 M
  parameters), self-play with a pool of 8 frozen snapshots, wall-time budget, evaluation of the
  sampled policy against the internal Random on 1 000 held-out episodes per side, `checkpoint.pt`
  and `best.pt` (network and optimizer only), records in the ignored `python/runs/`.
- The ladder `h3 ai eval` (Glicko-2, one period per run) and `python -m h3env.train.ladder`. On
  1 000 held-out encounters per pair: Greedy 0.828 and Aggressor 0.800 against Random, Greedy
  0.519 against Aggressor; the smoke network 0.802, 0.498, 0.479 (`E05-log.md` T7).
- Throughput on the user's machine: about 50 000 steps/s a thread, 497 000 on 24 threads at 1 024
  envs inside the step, 3.8 × only at 64 envs (`E05-log.md` T9c); the smoke run sampled about
  8 500 steps/s end to end (the network's passes and updates dominate).
- The AI queue's inbox for E06: the evaluation's repetition rule, the oracle harness, AI-Q7, the
  bots' untested fallbacks, the trainer's copy of each call's observations (9 MB at 256 envs),
  the observation's cost, `h3_batch_create`'s unchecked N.

## Non-goals

The adventure env and its AI (E13); encounters v2 (the adventure's logged battles); E23's policy
itself (the battle lane); a trained network running inside the engine, the server or a client
(Python only; an export format later); new scripted bots; cloud compute; escapes in the reward; a
new observation or action encoding, unless an increment's evidence asks for one (then a
`cross-lane` change with the battle lane, after the user's review).

## Tasks (outline; refine at the start of each increment; keep status current)

**Increment 1:** T1, then the user's review, then T2 in three small pull requests (design §10;
the user, 2026-10-20: T1 and T2a in one).

- [x] **T1 Design and ADR draft** — done 2026-10-20: `E06-design.md`, ADR-0026 accepted (every
  point as proposed); the record in `E06-log.md`.
- [x] **T2a Config and record** — done 2026-10-20: `h3env.train.run` (TOML configs, the
  schedule in samples, typed metrics, `record.json`, the held-out check without a growing set),
  `python/configs/`, `python/records/`; the record in `E06-log.md`.
- [x] **T2b1 The full checkpoint and `--resume`** — done 2026-10-20: `checkpoint.pt` with the
  whole state, Ctrl+C, `--resume` in segments; the record in `E06-log.md`. The pending gate or
  ladder and `--init-from` moved to T3b (the user's cut: they need the gate).
- [x] **T2c C ABI v31** — done 2026-10-20: the engine's identity in the record (the encodings'
  and encounters' versions, each entry's rulesets' content hashes), N capped at `MAX_ENVS`;
  the record in `E06-log.md`. The observations written in place moved to T5.

**Increment 2:** T3a, then T3b (the user, 2026-10-20: each in one pull request).

- [x] **T3a The held-out sets v1** — done 2026-10-20: `h3 ai eval --set report-v1|v0` and
  `--json`, the report set's stratified rates and intervals in Python and the ladder, the gate
  set's seeds, every held-out seed in the trainer's check, the bots' table
  `python/records/bots-report-v1.json`, the bots' fallbacks tested; the record in `E06-log.md`.
- [x] **T3b The evaluation gate** — done 2026-10-20: the SPRT gate on the gate set
  (`h3env.train.gate`), promotions (`promoted/<n>.pt`, `best.pt`), the ladder per promotion and
  the final best's, `[gate]` and `[ladder]` replacing `[evaluation]`, record schema 4, checkpoint
  format 2 with a pending gate or ladder at the resume, `--init-from`; the record in
  `E06-log.md`. The hour with gates on the user's machine (`t3b-1`, 2026-10-20): evaluation
  21.4 % of 124 minutes, the agent at the bots' level from 10 M samples on, stalling suspected
  (AI-Q9).

**Increment 3:** T4a, then T4b (refined 2026-10-20, AI-Q9: T4a in one pull request; T4b two
league runs, γ 1.0 beside 0.995, 60 M samples each).

- [x] **T4a The league** — done 2026-10-20: `h3env.train.league` (PFSP over snapshots and
  promoted networks, a live set drawn every `live_every` samples) replacing E05's pool,
  `[league]`, the members' files in `league/`, checkpoint format 3, record schema 5,
  `configs/e06-t4.toml`; the record in `E06-log.md`.
- [x] **T4b The first league runs** — done 2026-10-22 (`t4-1`, `t4-2`): γ 1.0 ends the stalling
  (truncations 13 % to 0) and plays better (final 0.839, 0.578, 0.559 against Random, Aggressor,
  Greedy); the stopped `t4-2` resumed for real; the record in `E06-log.md`.

**Increment 4:** T5a, then T5b (refined 2026-10-22, the user: the entity network first, design
§11; a brief comparison of architectures in `E06-log.md`; the visualization in T5a); after
T5b's stalling, T5c and T5d (the user, 2026-10-22).

- [x] **T5a The entity network** — done 2026-10-22: `h3env.train.entity` behind one interface
  with the MLP, `[network] kind`, the v21 layout pinned by `h3-sim`'s tests, γ 1.0 the default,
  `python -m h3env.train.network`, `configs/e06-t5.toml`; the record in `E06-log.md`.
- [x] **T5b The A/B run** — done 2026-10-22, negative: `t5-1` stopped at 26.6 M (1 600
  samples/s, 45 % of self-play stalled to the step limit, both sides hiding at their edges); the
  record and the diagnosis in `E06-log.md`.
- [x] **T5c The truncation penalty** — done 2026-10-22 (the user's amendment of decision 10):
  `[ppo] truncation_penalty` 1.0; `configs/e06-t5c.toml`.
- [x] **T5d The runs with the penalty** — done 2026-10-23 (`t5-2`, `t5-3`): the entity network
  parked (behind the MLP per sample from 10 M, 3.4 times slower); the MLP with the penalty
  0.832, 0.557, 0.533 in 20 M samples, near `t4-2`'s 60 M; the record in `E06-log.md`.

**Increment 5:** T7 (refined 2026-10-23, the user: T7 before the rest of T5, no run meanwhile;
the weights in the run's config, design §12): T7a and T7b, then the checkpoint, then T7c and T7d.

- [x] **T7a `h3-sim` may depend on `h3-battle-ai`** — done 2026-10-23 (#420):
  `arch.toml`, the sim doc's neutral kinds.
- [x] **T7b The system actor's seat** — done 2026-10-23: `Policy::System`, the neutral kinds'
  defender, C ABI v32 (`H3_SEAT_SYSTEM`, `h3_system_policy_version`), `h3env`'s `"system"`,
  `h3 ai eval --policies system`; the record in `E06-log.md`.
- [x] **T7c The trainer's neutral share** — done 2026-10-23: no league member in a neutral
  episode, the rates by kind, the neutral set v1 (`h3 ai eval --set neutral-v1`, the ladder's
  `neutral`), the system actor on the ladder, both bots' tables (`--json` schema 2),
  `configs/e06-t7.toml`; the record in `E06-log.md`.
- [ ] **T7d The run** — the user's, its mix chosen with T7c's numbers.

**Later increments** (cut at each checkpoint; proposed in design §10):

- [ ] **T5 The rest** — AI-Q7 (b); the mana-price experiment; batch size and threads from T9c's
  table; the observations in place.
- [ ] **T6 Toward the targets** — runs on the best configuration until decision 7's targets hold,
  or the evidence says PPO plateaus (then T8).
- [ ] **T8 Search** (only if PPO plateaus below the targets) — determinized search over the fork
  with re-seeded streams, the learned network as its prior and value.
- [ ] **T9 Closure** — the audit, an independent review, the verifies, the docs.

## Exit criteria

1. Runs are config-driven and recorded (config, commit, ruleset hash, ABI version, metrics); every
   run the epic's results cite has its record committed.
2. A run interrupted at any moment resumes from its last checkpoint with `--resume`, losing at
   most the time since that checkpoint; tested.
3. The evaluation gate promotes automatically by the declared SPRT on the held-out set; the
   ladder is played for every promoted checkpoint.
4. The league (prioritized fictitious self-play over past checkpoints) trains the promoted agent.
5. The promoted agent meets decision 7's targets on the player-against-player kinds, and, once
   E23's neutral side exists, the neutral target.
6. `cargo xtask verify` passes on Linux and Windows, including `--with-original-data`; the Python
   tests pass on both (a lane command).

## Risks

- **The targets may be out of reach for PPO on v21:** 0.75 against Greedy from today's 0.479.
  The entity network and longer runs first; search (T8) is the fallback; the targets can be
  revised at a checkpoint with evidence.
- **E23's timing:** the neutral share waits for its T2 to T4, the neutral target's last clause
  for its T5 and T6.
- **The GTX 1070:** 8 GB, no tensor cores; the network's passes already dominate the run. A
  larger network costs samples per second; measure before choosing.
- **League cost:** frozen opponents cost forward passes (E05: the pool cost a third of the
  updates per minute).
- **The encounter approximation** (E05's risk) carries over: the targets hold on v1's
  distribution, not yet on real games.
