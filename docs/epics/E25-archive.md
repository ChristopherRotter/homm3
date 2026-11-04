# E25 — archive: closed increments

Moved out of `E25-battle-ai-v2.md` verbatim when an increment closes (increment 1 on
2026-10-28, increment 2 on 2026-10-29, increments 3 and 4 on 2026-11-02, increment 5 on 2026-11-03, at their checkpoints), so that the epic file holds what
is open and the decisions that apply to the whole epic. Each task's full record is in `E25-log.md` (grep its id).

## Increment 1 (closed 2026-10-28)

**Increment 1: the diagnostics and shaping** (refined 2026-10-27, AI-Q10; the AI queue's tasks):

- [x] **T1a The search's terminal by reward v1** (2026-10-28; `E25-log.md` T1a): a world scores
  a decided battle by its batch's reward v1 (`BattleEnv::with_reward`; the agent's choice: the
  world, no new function), its setup keeps the other hero's starting mana (reward v1's mana
  term); C ABI v35.
- [x] **T1b The true-world search** (2026-10-28; `E25-log.md` T1b): `BattleBatch::true_world`,
  `h3_batch_true_world` (C ABI v35 with T1a; trusted tooling, a diagnostic), `h3env.search
  --true-worlds` (the flag spelled so, `--worlds` stays the count).
- [x] **T1c T8c again** (2026-10-28; `E25-log.md` T1c; `t7-1`'s `best.pt`): no gain at N 16 or
  64 against the system actor or on the neutral set, with guessed or true worlds (the guess
  costs at most +0.012 in value, within the intervals): the network's evaluation limits the
  search, not the terminal or the hidden information.
- [x] **T2a Shaping and the training mix** (2026-10-28; `E25-log.md` T2a): `[ppo] shaping` (off
  by default; Φ the batch's potential, `H3BatchBuffers.potential`, C ABI v35), the faithful
  system actor's mixes refused (`run.trainable`: no `system_share`, no neutral kind; the user:
  no neutral battles for now), T2b's configs `e25-control.toml`, `e25-shaping.toml`.
- [x] **T2b The A/B runs** (2026-10-28; `E25-log.md` T2b; `e25-control-1`, `e25-shaping-1`, 20 M
  each): the control matches `t7-1` against the player kinds, the neutral set's value 0.015
  lower; shaping gains about +0.04 against Aggressor, Greedy and the system actor (0.409, −0.066)
  and learns faster, but loops in the tactics phase (its placements 24 to 28 % of its decisions,
  the control's 10 to 12 %; 0.2 % of training battles cut).
- [x] **T3 Increment 1's checkpoint** (2026-10-28; `E25-log.md` T3): the user chose increment 2,
  shaping bounded, then scale; `e25-shaping-1` is the lineage's current best.

## Increment 2 (closed 2026-10-29)

**Increment 2: shaping, bounded, then scale** (refined 2026-10-28; the AI queue's tasks):

- [x] **T4a The tactics bound** (2026-10-28; `E25-log.md` T4a): `[network] tactics_rounds`
  (off by default; T4b's `configs/e25-bound.toml` 1): past that many rounds of its own phase End
  tactics is the agent's only entry, wherever the network plays; the binding counts the phase's
  turns from the observations (Python only, no ABI change).
- [x] **T4b The A/B** (2026-10-29; `E25-log.md` T4b): `e25-bound-1` at or above
  `e25-shaping-1` against every engine seat (inside the intervals), the loop gone (the phase 2.5 %
  of its decisions, none cut), 73 minutes against 90; the user: it is the current best.
- [x] **T5 The scale run** (2026-10-29; `E25-log.md` T5): `e25-scale-1`
  (`configs/e25-scale.toml`, three tactics rounds), stopped by the user at 41.3 M of 60 M, the
  curve flat from 20 M; best #3 (30 M) at `e25-bound-1`'s level against every engine seat
  (0.418 against the system actor; the differences inside the intervals), the phase 4 % of its
  decisions, no loop. Scale gains nothing at these settings.
- [x] **T6 Increment 2's checkpoint** (2026-10-29; `E25-log.md` T6, AI-Q11): the current best
  `e25-scale-1` #3; the head-to-heads skipped; increment 3 is the network's architecture, a
  distillation bench first (the user).

## Increment 3 (closed 2026-11-02)

**Increment 3: the network's architecture** (refined 2026-10-29, AI-Q11; the AI queue's tasks).
Every run since E05 is the 2 × 512 MLP over the flat observation; the search (T1c) and scale
(T5) stopped at its limit. Candidates, all on observation v22 (no encoding change): a deeper and
a wider MLP; a **hex convolution** network (the field in axial coordinates, a 3 × 3 kernel
without two opposite corners: a hex's six neighbours); a **hex transformer** (the hexes as
tokens, a bias per axial offset); E06's entity network as it is. A distillation bench ranks them
against the current MLP before any RL run.

- [x] **T7a The network kinds** (2026-10-29; `E25-log.md` T7a): `[network] layers` for the MLP,
  `kind = "hexconv"` and `"hexformer"` (`h3env.train.hexnet`: the stacks drawn on their hexes,
  the entity network's heads, `entity.Pointed`), `field_width`, `field_height`; tests; the costs
  on the sandbox's CPU (`hexconv` 64 × 6 15 times the MLP's operations).
- [x] **T7b The distillation bench** (2026-10-29; `E25-log.md` T7b): `h3env.train.distill`,
  every student trained on the same rollouts of the teacher (`e25-scale-1` #3) to its heads and
  its λ-returns, scored on held-out rollouts per head and category; `configs/distill/e25.toml`
  (six students); a CPU smoke (minibatches of 512 rows: PPO's do not fit the hex networks).
- [x] **T7c The bench run** (2026-10-30; `E25-log.md` T7c; `records/distill/distill-1.json`):
  the structured networks fit the teacher ten times worse than any MLP (agreement 0.62 to 0.64
  against 0.87 to 0.88) at 5 to 25 times its cost; the wide MLP leads everywhere, its value past
  the teacher's (0.612 against 0.575). The user: the wide MLP alone to RL, from scratch.
- [x] **T7d The A/B** (2026-10-30; `E25-log.md` T7d; `records/e25-wide-1.json`): the wide MLP
  from scratch at 20 M ends at `e25-scale-1` #3's ladder (every point estimate within 0.006;
  0.414 against the system actor) at 0.7 times its samples a second; its value no better in RL.
  The decisions' mix not read (its `best.pt`, 49.5 MB, is above the upload limit).
- [x] **T8 Increment 3's checkpoint** (2026-11-02; `E25-log.md` T8, AI-Q12): `e25-scale-1` #3 stays
  the current best; increment 4 is the policy's sharpness and PPO's settings (the user); T7c's
  verdict on the structured networks holds for those configurations only (the user's caveat).

## Increment 4 (closed 2026-11-02)

**Increment 4: the policy's sharpness and PPO's settings** (refined 2026-11-02, AI-Q12; the AI
queue's tasks). Every run since E05 used PPO's settings as E05 T8 set them (the entropy bonus
0.01 throughout, 4 epochs, 8 minibatches, the learning rate 2.5e-4 falling linearly, rollouts of
128 steps × 256 envs); the entropy ends at 1.5 (`e25-scale-1`), the sampled policy's spread. A
diagnostic without training first, then 20 M A/Bs on the 2 × 512 MLP at `e25-scale.toml`'s
settings. Decision 1's rules unchanged: the sampled policy is judged, only trained sharper.

- [x] **T9a The temperature** (2026-11-02; `E25-log.md` T9a): `Policy.act`'s logits of every head
  divided by a temperature (1 the default, bit-identical; 0 the most likely choice), through
  `evaluate.ladder` and `h3env.train.ladder --temperature 1,0.7,0.5,0` (a line per temperature
  and opponent, the same episodes and draws); training untouched.
- [x] **T9b The diagnostic** (2026-11-02; `E25-log.md` T9b): `e25-scale-1` #3 at temperatures 1,
  0.7, 0.5, 0: 0.418, 0.428, 0.428, 0.425 against the system actor, +0.002 to +0.010 against
  every other seat at 0.7, all inside the intervals. The spread costs about 0.01 of the 0.08 gap;
  against its own base rates #3 falls short on both sides alike (about 0.09 in the hero kinds).
- [x] **T10a The entropy's schedule** (2026-11-02; `E25-log.md` T10a): `[ppo] entropy_floor` (the
  bonus falling with the learning rate's progress to that fraction of `entropy_coef`; 1 the
  default, every run before) and `configs/e25-sharp.toml` (the bonus to 0 at 20 M); the second
  lever (fewer epochs or a larger rollout) not run (the user at T11).
- [x] **T10b The A/B** (2026-11-02; `E25-log.md` T10b; `records/e25-sharp-1.json`): the bonus to
  0 over 20 M ends at #3's ladder at temperature 0.7 (0.422 against the system actor; every
  point estimate 0.001 to 0.009 above #3's, inside the intervals): training sharper buys the
  sharpening, no more.
- [x] **T11 Increment 4's checkpoint** (2026-11-02; `E25-log.md` T11, AI-Q13): no second A/B;
  `e25-sharp-1` #2 the current best after the head-to-head against #3; increment 5 waits for
  the fair system actor v1 (the user: the battle lane pushes it soon).

## Increment 5 (closed 2026-11-03)

**Increment 5: the fair system actor** (the user at T11, AI-Q13; refined at T13, 2026-11-02,
AI-Q14). The battle lane's fair system actor v1 (E26, #647: `h3_battle_ai::choose_fair`, the
original's computer on its side's world; 0.500 against the all-seeing one on random armies with
heroes, +0.8 % Ir per decision) becomes the AI lane's system actor, may train and is imitated.

*Decisions* (the user, 2026-11-02, AI-Q14):

1. **`system` is the fair actor everywhere:** `Policy::System` (`choose_fair`), `H3_SEAT_SYSTEM`,
   `h3 ai eval`'s `system`, the neutral kinds' defender. The all-seeing one stays as
   `system-all-seeing` (`Policy::SystemAllSeeing`, a new seat, C ABI v36). Records tell the two
   apart; a record before the switch read as all-seeing. The adventure env has no system actor
   to switch (its neutral seat is the caller's): a computer there is E13's.
2. **The target** (decision 1 stands): exit criterion 1 against the all-seeing actor on the
   report set; the fair one a reported column beside it. **The neutral set's defender is the
   fair actor**, its target the fair actor's own value in the player's seat (the bots' table
   regenerated).
3. **Training** (decision 2): the neutral kinds back at E06 T7's third, the fair actor their
   defender; no `system_share`. `run.trainable` refuses the all-seeing actor only.
4. **Imitation** (decision 3): a warm start by behaviour cloning on the fair actor's choices,
   then PPO from it, an A/B against the same mix from scratch (decision 5: the user's machine).

| Task | Scope | Status |
|---|---|---|
| T12 | The head-to-head: `e25-sharp-1` #2 against `e25-scale-1` #3 promoted by the gate, 0.523 over 332 pairs (+16 Elo): #2 the current best | done 2026-11-02 (`E25-log.md` T12) |
| T13 | Increment 5's refinement (AI-Q14) | done 2026-11-02 (`E25-log.md` T13) |
| T14 | The switch: `system` the fair actor, `system-all-seeing` beside it (C ABI v36), records, the bots' tables, the neutral bench, `run.trainable` | done 2026-11-02 (`E25-log.md` T14: the fair actor 0.500 against the all-seeing one, the neutral target stays +0.285) |
| T15 | The ladder at the switch: `e25-sharp-1` #2 against both system actors and on the neutral set with the fair defender | done 2026-11-02 (`E25-log.md` T15: 0.424 against the fair actor, 0.425 against the all-seeing one; the neutral set +0.245 against the target +0.285) |
| T16 | The cloning bench: the system actor's choice as an encoded action (C ABI v37), the fair actor's choices cloned into a run's network, its agreement and ladder | done 2026-11-02 (`E25-log.md` T16: 4 M samples, agreement 0.531; the clone 0.396 against the fair actor, the neutral set +0.244) |
| T17 | The A/B: the neutral kinds back from scratch against the same from T16's clone | done 2026-11-03 (`E25-log.md` T17: `e25-clone-1` #2 0.454 against both system actors, the neutral set +0.282; the control `e25-neutral-1` 0.411, +0.269) |
| T18 | The checkpoint: T17 read, the current best, increment 6 | done 2026-11-03 (`E25-log.md` T18, AI-Q15: `e25-clone-1` #2 the current best, 0.564 head to head against `e25-sharp-1` #2; increment 6 better imitation) |

## Increment 6 (closed 2026-11-04)

**Increment 6: better imitation** (the user at T18, 2026-11-03, AI-Q15). T17's warm start
by cloning the fair actor was the first lever past 0.41 to 0.43 against the system actor
(`e25-clone-1` #2: 0.454, the neutral set +0.282), but the clone copies the fair actor loosely
(T16: agreement 0.531, its cross-entropy still falling at 4 M samples) and PPO from it added
0.06. Increment 6 copies the teacher better and longer.

*Decisions* (the user, 2026-11-03, AI-Q15; the agent's details, change at a checkpoint):

1. **The teacher stays the fair actor** (decision 3), **the student the run's network** (the
   2 × 512 MLP over observation v22, so PPO starts from it); no new encoding.
2. **The ceiling first:** the fair actor draws among near-equal choices, so its agreement with
   its own second draw on the same states bounds any clone's; T16's 0.531 is read against it
   and the longer clone's budget set by it.
3. **The imitation loss** is decision 3's second form: a cross-entropy to the fair actor's
   choice on the learner's own decisions, its weight decaying to 0, off by default; reward v1
   and the evaluation unchanged (ADR-0026's gate and ladders).
4. **The control is `e25-clone-1`** (T17): the A/B's arms run its mix, length and seed.

| Task | Scope | Status |
|---|---|---|
| T19 | The teacher's ceiling: the fair actor's agreement with its own draws, per category | done 2026-11-03 (`E25-log.md` T19: the fair actor repeats itself 0.972, 0.044 nats; T16's student 0.531, `e25-clone-1` #2 0.270: the gap is the network's) |
| T20 | The longer clone: the clone bench at a larger budget, read against T19's ceiling | done 2026-11-03 (`E25-log.md` T20: 16 M samples, agreement 0.576 (T16 0.531, the ceiling 0.972); the student 0.411 against both system actors, the neutral set +0.251) |
| T21 | The imitation loss: PPO's auxiliary cross-entropy to the fair actor's choice, decaying to 0 | done 2026-11-03 (`E25-log.md` T21: `[ppo] imitation_coef`, `imitation_horizon`; the teacher's calls change no episode, weight 0 is `master`'s training bit for bit; the CPU check: agreement held, −0.015 over 0.4 M against −0.044, the entropy kept up; the teacher 14 % of the sandbox's time) |
| T22 | The A/B: PPO from T20's student, with and without T21's loss, against `e25-clone-1` | done 2026-11-04 (`E25-log.md` T22: `e25-imitate-1` #2 (the loss, weight 0.5 to 0 at 10 M) 0.458 and 0.462 against the system actors, the neutral set +0.287, 0.534 head to head against `e25-clone-1` #2; `e25-clone-2` #2 (without) 0.451, +0.284, 0.508) |
| T23 | The checkpoint: T22 read, the current best, increment 7 | done 2026-11-04 (`E25-log.md` T23, AI-Q16: `e25-imitate-1` #2 the current best; increment 7 a structured network) |

