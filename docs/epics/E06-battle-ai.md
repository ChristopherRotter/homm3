# E06 — Battle AI v1 and the local training pipeline v0

Status: **closed** 2026-10-27 (refined 2026-10-20, AI-Q8; T1 to T9 in 8 increments, each with a user checkpoint; the battle AI v1 is `t7-1`, decision 7's targets as revised 2026-10-27 met and played again on the closing engine; verified on Linux by CI and on the user's Windows machine incl. `--with-original-data`; the search's measurement inconclusive, a stronger agent E25's) · Lane: **ai** · Follows E05 (closed
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
   targets** (decision 7). **Closed 2026-10-27** (the user, increment 7's checkpoint): the
   search over determinized worlds, measured at T8c, gained nothing over its network; expert
   iteration (T8d) not built, a search in Rust trained on its own choices is E25's candidate.
   **T9b** (the user, 2026-10-27): T8c's verdict inconclusive, not negative (a terminal scored
   ±1 against the value head's reward v1, escapes reachable below the root); measured again in
   E25.
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
   **Amended 2026-10-26** (the user, after the turn-order probe): observation v22 appends the
   engine's turn order (T6f); a network reads it only with `[network] order` (T6g: no
   measurable gain).
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
   **Revised 2026-10-27** (the user, increment 7's checkpoint; six levers left PPO at one ladder
   and the search gained nothing, E06-log T8c): **the agent beats every heuristic bot.**
   Against Random, Aggressor and Greedy a win rate above 0.5 and a positive value efficiency; in
   the neutral share, in the player's seat against the system actor, a value efficiency better
   than every heuristic bot's in the same seat on the same episodes. Judged as before (the
   lineage's latest promoted network, each point estimate, ADR-0026 point 7). `t7-1` meets them:
   0.828, 0.551, 0.528 at +0.273, +0.029, +0.010 (against Greedy within its interval, ±0.018, of
   0.5); the neutral set +0.245 against Greedy's +0.223 and Aggressor's +0.216. Not E06's: 0.95
   and 0.75, and the system actor, which beats `t7-1` 0.626 (+0.101) and attacks the neutral set
   better (+0.285): E25's target.
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
| 7 | 2026-10-27 | T8a–T8c, T6f, T6g | the turn order as an input (negative), the search over determinized worlds (inconclusive, T9b); T8d not built; with it T5's and T6's open parts (decision 7 revised) |

**Increment 8, T9 the closure** (refined 2026-10-27, the user at increment 7's checkpoint: E06
closes with `t7-1` as the battle AI v1, decision 7 revised; the pattern E05's T9, `E05-log.md`):

- [x] **T9a The audit** (the sandbox) — done 2026-10-27 (`E06-log.md`): exit criteria 1 to 5
  met (5's neutral comparison spans the system actor's `POLICY_VERSION` 18 and 19: T9c plays
  `t7-1`'s ladder again); the leftovers sorted in the AI queue's inbox (E25, encounters v2, E13,
  the adventure env); the Python READMEs current.
- [x] **T9b The independent review** (the sandbox) — done 2026-10-27 (`E06-log.md`): three
  reviewers (the trainer; the search and its worlds; the batch since E05), 14 findings verified;
  fixed: a refused checkpoint's files, a refused `best.pt`, one-sample minibatches, escapes in
  the search's tree, a world of the side not to move, the system actor's retreat in a batch, the
  distribution's data pinned to its version; the bots' tables regenerated (the adventure hash).
  The user: T8c inconclusive and a terminal by reward v1 E25's, the world's three gaps to
  AI-STUB-1 (E25).
- [x] **T9c The verifies** (exit criterion 6, as amended) — done 2026-10-27 (`E06-log.md`): on
  the user's Windows machine `cargo xtask verify --with-original-data` green and the Python tests
  passed; on Linux CI's full verify (#519) and the sandbox's Python tests (T9b); `t7-1`'s final
  ladder on the closing engine: 0.830, 0.550, 0.530 at +0.273, +0.029, +0.010, the neutral set
  0.970 at +0.244 (Greedy's +0.224): the targets hold.
- [x] **T9d The closure** — done 2026-10-27: #522 first (BACKLOG E06 done; 
  `docs/07-simulation-and-ai.md` §1.2, the training pipeline as built; ADR-0026's working-detail line; ADR-0025 point
  12a after the review); the design note's text kept in `E06-log.md` ("The design note at
  closure", its numbering kept), the note deleted, `python/`'s references repointed; the lane's
  HANDOFF and QUEUE.

## Exit criteria

1. Runs are config-driven and recorded (config, commit, ruleset hash, ABI version, metrics); every
   run the epic's results cite has its record committed.
2. A run interrupted at any moment resumes from its last checkpoint with `--resume`, losing at
   most the time since that checkpoint; tested.
3. The evaluation gate promotes automatically by the declared SPRT on the held-out set; the
   ladder is played for every promoted checkpoint.
4. The league (prioritized fictitious self-play over past checkpoints) trains the promoted agent.
5. The promoted agent meets decision 7's targets on the player-against-player kinds, and, once
   E23's neutral side exists, the neutral target (revised 2026-10-27: it beats every heuristic
   bot in both).
6. `cargo xtask verify` passes on Linux and Windows, including `--with-original-data`; the Python
   tests pass on both (a lane command). **Amended 2026-10-27** (the user, T9b): on Linux CI's
   full verify (no original data, ADR-0009), `--with-original-data` on Windows alone.

## Risks

- **The targets may be out of reach for PPO on v21:** 0.75 against Greedy from today's 0.479.
  The entity network and longer runs first; search (T8) is the fallback; the targets can be
  revised at a checkpoint with evidence. **Realized** (2026-10-27): PPO plateaued at Greedy's
  level and the search gained nothing (T9b: inconclusive, measured again in E25); decision 7
  revised, the gap E25's.
- **E23's timing:** the neutral share waits for its T2 to T4, the neutral target's last clause
  for its T5 and T6.
- **The GTX 1070:** 8 GB, no tensor cores; the network's passes already dominate the run. A
  larger network costs samples per second; measure before choosing.
- **League cost:** frozen opponents cost forward passes (E05: the pool cost a third of the
  updates per minute).
- **The encounter approximation** (E05's risk) carries over: the targets hold on v1's
  distribution, not yet on real games.
