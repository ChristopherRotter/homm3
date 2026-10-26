# E06 — Battle AI v1 and the local training pipeline v0

Status: **in progress**, refined 2026-10-20 (AI-Q8); increment 1 (T1, T2a, T2b1, T2c) done; increment 2 (T3a, T3b) done; increment 3 (T4a, T4b) done; increment 4 (T5a to T5d) done 2026-10-23; increment 5 (T7a to T7d) done 2026-10-23, with T5's mana preparation and AI-Q7 (b) part 1; T6a (the behaviour analysis) done 2026-10-23; increment 6 (T6b to T6e) done 2026-10-26; increment 7 (T8, the search) refined 2026-10-26, T8a done · Lane: **ai** · Follows E05 (closed
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
   encodings unchanged; kept only if it measurably helps. **Closed 2026-10-26** (T6e, the
   user): no measurable gain, `[network] preview` stays in the code, off by default.
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
   bots never train (E05 decision 8). **Amended 2026-10-26** (the user, increment 6's lever): a
   share of the player kinds' episodes against the system actor too (`[league] system_share`,
   design §12); every side still plays as a human player's (ADR-0025 point 6).
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

**Closed increments** (their task lists verbatim in `E06-archive.md`; the records in
`E06-log.md`):

| Increment | Closed | Tasks | Content |
|---|---|---|---|
| 1 | 2026-10-20 | T1, T2a, T2b1, T2c | the design and ADR-0026, configs and records, the full checkpoint and `--resume`, C ABI v31 |
| 2 | 2026-10-20 | T3a, T3b | the held-out sets v1, the bots' table, the SPRT gate and promotions |
| 3 | 2026-10-22 | T4a, T4b | the league (PFSP), the first league runs (γ 1.0 ends the stalling) |
| 4 | 2026-10-23 | T5a–T5d | the entity network (parked), the truncation penalty |
| 5 | 2026-10-23 | T7a–T7d | the system actor's seat, the neutral share, `t7-1` |
| 6 | 2026-10-26 | T6b–T6e | the system actor as an opponent, the kill preview in the network (AI-Q7): both negative |

**Increment 7, T8 the search** (refined 2026-10-26, the user: the plateau taken as shown after
T6e; measure a decision-time search before any training change; determinized from the start;
the tree in Python first; `h3-battle`'s part `cross-lane` in T8b; design §14):

- [x] **T8a The design** — done 2026-10-26: design §14, ADR-0025 point 12's amendment as a
  draft (`docs/lanes/ai/adr-drafts/0025-amendment-search.md`); the record in `E06-log.md`.
- [ ] **T8b The plumbing** — C ABI v34 (`h3_batch_determinize`, `h3_env_reseed`), `h3-battle`'s
  `BattleState::determinized` and `reseed` (`cross-lane`), `h3env.search`; the tests of §14.
- [ ] **T8c The measurement** — the search agent with `t7-1`'s best at N 16 and 64 against the
  system actor and Greedy (report set) and on the neutral set, beside the sampled policy.
- [ ] **T8d Expert iteration** — only if T8c gains: the search's targets in training.

**Later increments** (cut at each checkpoint; proposed in design §10):

- [ ] **T5 The rest** — AI-Q7 (b); the mana-price experiment; batch size and threads from T9c's
  table; the observations in place. Beside T7d (the user, 2026-10-23): the mana price's
  preparation done 2026-10-23 (the ladders at reward v1's settings, `configs/e06-mana.toml`; the
  record in `E06-log.md`); AI-Q7 (b) part 1 done 2026-10-23 (the batch's previewed kills, C ABI
  v33); part 2 (the network reads them), increment 6's T6d, negative at T6e (AI-Q7 closed).
- [ ] **T6 Toward the targets** — runs on the best configuration until decision 7's targets hold,
  or the evidence says PPO plateaus (then T8). T6a, the behaviour analysis (the user,
  2026-10-23), done 2026-10-23: `python -m h3env.train.inspect`; `t7-1` casts as the system
  actor does, plays alike against it and Greedy, waits and defends less; the record in
  `E06-log.md`. Then increment 6 (T6b to T6e), closed 2026-10-26.
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
