# E25 — archive: closed increments

Moved out of `E25-battle-ai-v2.md` verbatim when an increment closes (increment 1 on
2026-10-28, increment 2 on 2026-10-29, increments 3 and 4 on 2026-11-02, increment 5 on 2026-11-03, 6 on 2026-11-04, 7 on 2026-11-10, 8 on 2026-11-18, 9 on 2026-11-20, at their checkpoints), so that the epic file holds what
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

## Increment 7 (closed 2026-11-10)

**Increment 7: a network that reads the battle's structure** (the user at T23, 2026-11-04,
AI-Q16). Increments 5 and 6 found the gap in the network: the 2 × 512 MLP copies the fair actor
0.576 of the time (T20; the teacher's ceiling 0.972), its moves 0.38 of 0.985 and its melee hexes
0.40 of 0.941, and samples gain about 0.02 a doubling. Increment 3's structured networks were
measured in one configuration each (no normalisation, no warm-up, one learning rate) and against
an MLP teacher (T7c). Increment 7 builds them properly and measures them on the cloning bench
against the fair actor before any RL run.

*Decisions* (the user, 2026-11-04, AI-Q16; the agent's details, change at a checkpoint):

1. **The bench is T16's cloning bench** (the fair actor, `e25-neutral.toml`'s mix, 4 M samples,
   the held-out set of seed 8), the 2 × 512 MLP its control in every run. A student is scored by
   its agreement (overall, moves, melee), its cross-entropy and its value's explained variance;
   its cost by the GPU's µs a sample-pass.
2. **The candidates:** `hexconv` with normalisation (64 to 128 channels, 6 to 10 blocks),
   `hexformer` (96 to 128 wide, 4 to 6 blocks), a wider `entity` (192 to 256, 2 to 4 blocks); each
   with a warm-up, AdamW and its own learning rate. The grid is the agent's, from T24's costs.
3. **The cost cap** (the user): a sample-pass on the GTX 1070 at most hexconv 64 × 6's (466 µs,
   about 10 hours of network time for 20 M samples of PPO); fp32 only; among the candidates within
   0.02 of the best agreement the cheapest wins. **Raised at T25** (the user, 2026-11-04, on the
   GPU's grid, `E25-log.md` T24): about 700 µs on `network --time`'s measure (where `hexconv` 64 ×
   6 is 301), one candidate of each family.
4. **The heads read the geometry** (the user: the legal mask as an input is no encoding change):
   a melee entry scores the hex its attack is made from; the legal mask (action encoding v13's,
   handed with every decision) is an input of every kind, an MLP with it a second control.
   Observation v22 and action v13 stay.
5. **Gates:** the long clone runs only if a candidate reaches 0.60 agreement at 4 M within the cap
   (the MLP: 0.531 at 4 M, 0.576 at 16 M); PPO only if the long clone's ladder beats T20's student
   (0.411) beyond its interval. A failed gate goes to the checkpoint.
6. **The RL arm:** `e25-imitate.toml`'s settings with the candidate's network, from its clone,
   20 M samples, seed 1, against `e25-imitate-1` #2 (the current best).

| Task | Scope | Status |
|---|---|---|
| T24 | The kinds and the bench built properly: normalisation, warm-up, AdamW, the cost in the bench's report | done 2026-11-04 (`hexconv`'s `norm`, `feed_forward`, the bench's `warmup` and `weight_decay`, the cost per student, `network --bench` and the grid's CPU table) |
| T25 | The heads read the geometry: a melee entry's attack hex, the legal mask as an input, `order` for the structured kinds | done 2026-11-04 (`geometry`'s port checked on 18 823 real attacks; `encode(obs, mask)`; `[network] mask_input`, `order`, `attack_hex`; T26's bench prepared at the user's cap of about 700 µs) |
| T26a | The resumable bench: the distillation bench and its clone mode resume to their budget (the user, 2026-11-05: every run over 2 hours resumes) | done 2026-11-05 (the shared loop `distill.go`: `checkpoint.pt`, Ctrl-C at a batch boundary, `--resume DIR`, the clone's ladders pending across a stop) |
| T26b | The structure sweep on the cloning bench, against decision 5's first gate (a first segment stopped at 0.45 M, `E25-log.md` T26) | done 2026-11-05 (the gate passed: `hexconv` normalised 96 × 6 at 1e-3, 0.755 at 4 M, 683 µs; the MLP 0.533; `E25-log.md` T26b) |
| T27 | The long clone of the best candidate, its ladder against decision 5's second gate | done 2026-11-06 (the gate passed: `hexconv` 0.809 agreement at 16 M against the MLP's 0.574, its ladder 0.481 ± 0.019 against the fair actor against 0.411; `records/distill/clone-3.json`; T19's tool 0.810 on its states, the casts 0.748 the one category below the MLP's) |
| T28 | The A/B: PPO from T27's student with the imitation loss, against `e25-imitate-1` #2 | done 2026-11-10 (the structure wins: `e25-structure-1` #2 0.511 ± 0.019, +0.027 against the fair actor, 0.507 ± 0.019, +0.026 against the all-seeing one, the neutral set +0.302: exit criterion 1 met on its point estimates; 217 samples a second, 22 times the MLP's time; the GPU's memory capped at 90 % of the card; the head-to-head left to T29) |
| T29 | The checkpoint: structure against the MLP (clone, RL, cost), the current best, increment 8 | done 2026-11-10 (`E25-log.md` T29, AI-Q17: the head-to-head 0.552 after 77 pairs; `e25-structure-1` #2 the current best; increment 8 probes, then one long run) |

## Increment 8 (closed 2026-11-18)

**Increment 8: probes, then one long run** (the user at T29, 2026-11-10, AI-Q17).
`e25-structure-1` #2 meets exit criterion 1 on its point estimates, but its lead over the
faithful actor (0.507 ± 0.019) lies inside its interval. Increment 8 analyses #2 and probes
architectures, PPO's settings and training regimes at a small cost each before one long run,
which aims at a margin beyond the interval.

*Decisions* (the user, 2026-11-10, AI-Q17; the agent's details, change at a checkpoint):

1. **The aim:** the long run's promoted network beats the faithful actor on the report set v1
   beyond its interval (the final ladder, 500 a kind), decision 1's other parts kept. E25 stays
   open until then, or until a checkpoint decides otherwise.
2. **Analysis first** (the sandbox's CPU): where #2 wins and loses against the fair actor by kind
   and side, beside the fair actor's own mirror on the same episodes; the temperature diagnostic
   (T9a); where it leaves its teacher (T19's states by category). Its read ranks the probes.
3. **The probes** (the user's GPU, about 20 hours in all, parts under 2 hours): **the update's
   speed first** (a profile of PPO's passes, cuDNN's autotuner on fixed chunk shapes, the
   epochs), then **architectures** on the cloning bench with its ladder (the clone's ladder
   predicted PPO's endpoint at T22 and T28, +0.03 to +0.05 above it), **PPO's settings** by short
   continuations from #2 (a few M samples each, judged by the gate against #2 and a ladder against
   the fair actor), **training regimes** (#2 distilled into another network, a KL anchor to #2, a
   search at play or expert iteration on ADR-0025 point 12a's worlds). Each probe's measure and
   budget is stated before it runs; the agent's split about 1, 8, 8 and 3 hours, rebalanced at
   T30's read.
4. **The long run** follows the probes' checkpoint (T35): its start (#2 or a new clone), network,
   settings and length are agreed there. **Agreed** (the user, 2026-11-16, T35): `e25-anchor-1`
   resumed to 20 M (T27's clone, #2 the imitation loss's teacher to 10 M; reward v1).
5. **Unchanged:** decision 1's evaluation; the fair actor the only system actor that teaches
   (decision 3's KL anchor has #2 teach, T34), the faithful one training no agent (decision 2); observation v22 and action v13 (an encoding change only at a
   checkpoint, on the analysis's evidence); fp32; the GPU's memory cap (T28).

| Task | Scope | Status |
|---|---|---|
| T30 | The analysis of #2 (decision 2) | done 2026-11-10 (`E25-log.md` T30: the margin lies in the hero battles, where #2 keeps more army but wins as often; temperature 0.5 +0.011; it keeps the teacher's kind of action, not its hex; no encoding change asked) |
| T31 | The update's speed: a profile on the user's GPU, then the change that pays | done 2026-11-10 (`E25-log.md` T31: cuDNN's autotuner in the update, `[ppo] accumulate` 4 and `chunk_align` 32 for `hexconv`: 958 to 649 µs a sample-pass, 98 to 67 s an update; `gpu.cap`'s card index fixed) |
| T32 | Architecture probes on the cloning bench with its ladder, `hexconv` 96 × 6 the control | done 2026-11-15 (`E25-log.md` T32: at 4 M the control 0.752, 2e-3 0.760, 128 × 6 0.761, 64 × 10 0.753, the ladders 0.449 to 0.461 inside the control's interval: no arm leads by 0.02, so `hexconv` 96 × 6 stays and T35's long run starts from T27's 16 M clone) |
| T33 | PPO's settings by short continuations from #2: the entropy bonus at 0, the imitation loss held at a small weight, then the learning rate and the epochs | done 2026-11-13 (`E25-log.md` T33: no arm gains; head to head against #2 the bonus at 0 0.494, a quarter of the rate 0.493, the loss held 0.478, that rate annealed to #2's floor 0.503, ± 0.002: #2 sits at this setup's plateau, so T35's long run starts from T32's best clone) |
| T34 | Training regimes: the search at play with #2 (the sandbox's CPU) and the KL anchor to #2 (PPO from T27's clone, #2 the imitation loss's teacher) | done 2026-11-15 (`E25-log.md` T34: the search gains nothing on #2, 0.508 against 0.511 at N 16, +0.003 at N 64 on the hero battles; the anchor reaches #2's level in 2 M, 0.496 ± 0.002 head to head (−1.97 SE), 0.503 against the fair actor) |
| T35 | The probes' checkpoint: the long run's start, network, settings and length | done 2026-11-16 (`E25-log.md` T35: the outcome's weight probed first, decision 6 amended; `e25-win-bonus-1`, T33's annealed arm with `win_bonus` 0.1, 0.5007 ± 0.0019 head to head against #2, the annealed arm's 0.5029: it moves nothing, reward v1 stays; the user: the long run is the anchor resumed to 20 M) |
| T36 | The long run and its checkpoint | done 2026-11-18 (`E25-log.md` T36: `e25-anchor-1` resumed from 2 M to 20 M, #2 teaching T27's clone to 10 M; both gates rejected the learner against its 2 M network; at 20 M 0.511 ± 0.018 against the faithful actor, every row within 0.007 of #2's: the aim not met, #2 stays; the user: a long run stops early, `[gate] futility`; increment 9, a richer observation) |

## Increment 9 (closed 2026-11-20)

**Increment 9: a richer observation (v23)** (the user at T36, 2026-11-18, AI-Q18). Increment
8's probes and its long run ended at #2's level (T32 to T36): the network's size, PPO's
settings, the search at play, the KL anchor, the outcome's weight and 18 M of a taught regime.
What is left is the epic's risk: information observation v22 does not show. v22's hex planes hold
the obstacles, the spell obstacles, the walls and the moat, each stack its stats, position and
state, the legal mask (`[network] mask_input`) the acting stack's reach and attacks: no other
stack's reach, no distance, no threat (what the opponent's stacks reach next), no shooter's
blocked state; the kill preview (C ABI v33, E06 T6d) only the MLP reads.

*Decisions* (the agent's proposal, 2026-11-18; the user reviews the design at T37's end):

1. **The aim:** increment 8's decision 1 (the long run's promoted network beats the faithful
   actor beyond its interval), decision 1's other parts kept.
2. **Design first** (T37, the sandbox): what the fair actor's choice reads that v22 hides from
   the network, where #2 and the fair actor choose differently (T30's states by category), and
   the candidate features with their cost a decision. The smallest set that covers the gap; the
   kill preview for `hexconv` (built, no engine change) the first candidate. The design, its cost
   and its versions are the user's review: an engine query in `h3-battle` and the golden replays
   make a `cross-lane` change (`TRAPS.md`, Ownership).
3. **A feature's worth on the cloning bench before RL** (the clone's ladder predicted PPO's
   endpoint, T22 and T28): `hexconv` 96 × 6 on v23 against T32's control (v22, 0.752 at 4 M, the
   same seed and budget); the rule stated before it runs.
4. **The long run only on a bench gain**, from the v23 clone, with the early stop (T36, the
   user's directive): `[gate] futility` at least 1, gates at least every quarter of its budget.
5. **Unchanged:** reward v1 and decision 1's evaluation; the fair actor the only system actor
   that teaches; action encoding v13; `hexconv` 96 × 6 (T32); fp32; the GPU's memory cap.

| Task | Scope | Status |
|---|---|---|
| T37 | v23's design: nothing the fair actor reads is hidden, the reach is left to a pathfinder; #2's moves are blind to it (0.468, chance 0.455), its strikes read the kills; v23 the reach block (AI-Q19) | done 2026-11-18 |
| T38 | v23 built: T38a the reach block from a viewer-safe query in `h3-battle` (`cross-lane`), C ABI v38 (the batch's step +140 % on the encounters); T38b `[network] reach` and `preview` for the hex kinds, the cloning bench's preview rows and `same_start`, T39's config | done 2026-11-19 |
| T39 | The bench probe: the control, v23 and v23 with the preview at 4 M (`configs/distill/e25-reach.toml`): no arm passed (0.457, 0.460, 0.459 ± 0.018 against the fair actor) | done 2026-11-20 |
| T40 | The long run from the v23 clone, on T39's gain, with the early stop | not run (T39: no gain) |
| T41 | Increment 9's checkpoint: increment 10, the ceiling (AI-Q20) | done 2026-11-20 |

## Increment 10 (closed 2026-11-23)

**Increment 10: the ceiling** (the user at T41, 2026-11-20, AI-Q20). Increments 8 and 9 ended
at #2's level whatever the lever (size, PPO's settings, the search at play, a KL anchor, the
outcome's weight, a taught regime, the reach, the kill preview). Before another lever: how much
better play against the fair system actor exists at all, what hidden information costs, and
where #2's choices lose value, so that the next lever aims at them.

*Decisions* (the user, 2026-11-20, AI-Q20; the agent's details, change at a checkpoint):

1. **The aim:** a lower bound on the ceiling against the fair actor (the result of a policy that
   provably does at least as well as the fair actor, up to sampling noise), the cost of hidden
   information, and the value #2's choices leave by category. No training in the increment.
2. **The method: rollout policy improvement** (one step of policy iteration against the fixed
   fair actor): at each of the agent's decisions every legal command, N rollouts of each to the
   battle's end, **the fair actor on both sides** (the user: first; rollouts with #2 inside them
   later, if their cost is worth it), the stream re-seeded per rollout and shared by the commands
   (common random numbers); the command of the best mean reward v1 is played. Randomness is
   averaged over, so the ceiling is in expectation.
3. **Both worlds** (the user): the true battle (`true_world`, a trusted diagnostic: the other
   hero's spells and the hidden obstacles seen) and the side's worlds (`determinize`: only what
   the side may know; a legitimate agent's); their gap the information's cost.
4. **A measurement, not a run with a rule:** the budget and the episodes fixed by a check on a
   subset before the full measurement; the report set v1 and the neutral set v1 beside it. The
   budget is spent by sequential halving (T42, the agent's call), the rollouts full-length.
5. **Unchanged:** #2 the current best; reward v1 and the evaluation sets; no engine or C ABI change
   (a new `h3 ai` tool in this lane's `h3-cli/src/ai/`, its rollouts in `h3-sim` where pure).

| Task | Scope | Status |
|---|---|---|
| T42 | The rollout tool: `h3 ai ceiling` (the improved fair actor against the system actors, both worlds, sequential halving with common random numbers, a resumable games file, a JSON record), its tests and timing (about 190 thread-seconds a game at budget 8) | done 2026-11-20 |
| T43 | The ceiling measured (the user's CPU): a budget check (4, 8, 16), then the report set v1 and the neutral set v1, both worlds | done 2026-11-21 (budget 4; the report set over the fair actor's play, paired: +0.085 on the side's worlds, +0.115 on the true battle, the information +0.030; the neutral set +0.047; one stalled game of 600 left out, the fair actor's, BAT-STUB-19's family) |
| T44 | Where #2 diverges: #2's decisions against the fair actor replayed, every command valued by the same rollouts; the value its choice leaves by category, beside the fair actor's own | done 2026-11-23 (the tool: `h3 ai ceiling --commands`, `inspect --commands`, a confirmation on fresh rollouts; #2's 598 games: +0.026 over the fair actor, the rollouts +0.059 over #2; its regret 0.28 a game, casts 0.024 a decision, the missed moves 38 %, its edge melee and waiting) |
| T45 | Increment 10's checkpoint: the headroom, the information's cost, the divergence; the next lever | done 2026-11-23 (AI-Q23: increment 11 is expert iteration; E25 paused after increment 10 for E31) |
