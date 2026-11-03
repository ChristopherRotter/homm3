# E25 — Battle AI v2

Status: **in progress**, refined 2026-10-27 (AI-Q10); increments 1 to 5 closed 2026-10-28, 29, 11-02 and 11-03, increment 6 (better imitation) refined 2026-11-03 (AI-Q15) · Lane: **ai** ·
Follows E06 (closed 2026-10-27: `t7-1` the battle AI v1, record `E06-log.md`)

## Goal

A battle agent that beats the faithful system actor (the original's battle AI, ADR-0027) on the
held-out encounters, where E06's battle AI v1 beats the heuristic bots but loses to it. It plays
on every battlefield and in sieges, is trained by E06's pipeline (ADR-0026), and is legitimate:
it reads only its side's observation, and its search too (ADR-0025 point 12a).

## Decisions at the refinement (the user, 2026-10-27, AI-Q10)

1. **The target: beat the faithful system actor** (exit criterion 1). On the report set v1
   against the system actor (the faithful profile, its `POLICY_VERSION` recorded), both sides
   weighed equally, the sampled policy: a win rate above 0.5 and a positive value efficiency. On
   the neutral set v1, in the player's seat against it: a value efficiency above the system
   actor's own in that seat on the same episodes (the bots' table, `bots-neutral-v1.json`:
   +0.285 at E06's revision). Against Random, Aggressor and Greedy still a win rate above 0.5 and
   a positive value (E06's decision 7 as revised). Judged as E06's: the lineage's latest promoted
   network, each point estimate (ADR-0026 point 7). It can be revised at a checkpoint with
   evidence.
2. **The faithful system actor trains no agent** (ADR-0026 point 8 amended). It reads the whole
   state (ADR-0027 point 4 lists what a player may not see). From E25 on it is no league member,
   no defender of the neutral kinds in training, and `system_share` stays 0. It is the target, a
   ladder opponent and a diagnostic's opponent only. `t7-1`, trained with it in E06, stays the
   baseline. A **fair variant** that decides on its side's observation is the battle lane's to
   build later (ADR-0027 point 4; the battle lane's inbox). Once it exists it may train, and it
   becomes the default in games, with the all-seeing one kept as a game option.
3. **Imitation** of the fair variant is a candidate once it exists (a warm start, or an
   auxiliary loss decaying to 0). Never of the faithful one, and never of the heuristic bots,
   which never train (E05 decision 8).
4. **Increment 1: the diagnostics and shaping.** The search's terminal scored by reward v1, T8c
   measured again, and the search on the true battle as the bound on what the guess costs. Beside
   them, potential-based shaping as an A/B run on the user's machine. Their outcome chooses
   increment 2 among the candidates below.
5. **Compute:** runs on the user's machine (GTX 1070, Ryzen 9 3900X), each run's length agreed
   with the user when it is prepared. The sandbox runs short CPU checks and the search's
   measurements.
6. **Unchanged from E06:** reward v1 and the evaluation (shaping changes the training signal
   only); retreat and surrender masked; the truncation penalty; the sampled policy on the
   held-out sets; SPRT promotion; committed records, checkpoints outside git; no new scripted
   bot. A search agent's worlds come from its side's knowledge (ADR-0025 point 12a); the
   true-world search is a diagnostic, never an agent. Increments with a user checkpoint each,
   only the current one refined; the user reviews ADRs and seams; token efficiency beats speed.

## Starting point (from E06)

- **`t7-1`** (`python/records/t7-1.json`; `configs/e06-t7.toml`): the MLP over `symlog` of
  observation v22 (5.95 M parameters), PPO with the PFSP league, γ 1, the truncation penalty, 20 M
  samples with a third neutral (the faithful system actor defending). On the closing engine (E06
  T9c): 0.830, 0.550, 0.530 against Random, Aggressor, Greedy; 0.372 at −0.103 against the system
  actor; the neutral set +0.244 (the system actor's own +0.285).
- **What E06 ruled out** (`E06-log.md`): six levers ended at one ladder. The truncation penalty
  was the gain (T5d); the system actor as an opponent (T6c), the kill preview (T6e) and the turn
  order (T6g) gained nothing; the entity network was parked (T5d: behind per sample, at 3.4 times
  the MLP's cost; its moves read a hex, not the stacks). The only rising curve was samples:
  `t4-2` at 50 M (no penalty) 0.839, 0.578, 0.559. The behaviour analysis (T6a): it loses mostly
  defending (0.17 in a hero battle), its targeting leaves a quarter of the previewed kills, and
  it plays the same against every opponent.
- **The search** (E06 T8; design note §14 in `E06-log.md`): Gumbel AlphaZero in Python
  (`h3env.search`) over worlds from C ABI v34 (`h3_batch_determinize`). T8c found no gain, but its
  verdict is inconclusive (T9b): a world's step scores a decided battle ±1 where the value head
  learned reward v1, and nodes below the root could take an escape (fixed). AI-STUB-1 holds the
  world's gaps.
- **The pipeline:** ADR-0026, `docs/07-simulation-and-ai.md` §1.2; about 5 000 samples/s on the
  user's machine, the network's passes dominate.

## Candidates for later increments (chosen at each checkpoint)

- **Expert iteration:** a search in Rust over determinized worlds, trained on its own choices
  (E06 design §14's T8d). Needs AI-STUB-1's gaps, the cast heads perturbed at the root, and a rule
  for the most visited action's repetitions (greedy play repeats itself, E05 T8b).
- **A network that reads the battle's structure, built and tuned properly** (the user at T8):
  T7c's verdict holds for its configurations only (`hexconv` 64 × 6 without normalisation,
  `hexformer` 64 × 4, E06's entity network; 0.3 to 1.2 M parameters, one optimiser setting, no
  warm-up). Wider ones with normalisation, warm-up and a learning rate of their own, on the bench
  first; the GTX 1070's cost is the constraint (`hexconv` 64 × 6: 21 times the MLP's a sample).
  With it `[network] order` and `preview` again.
- **A more diverse league:** exploiters (a second learner trained only against the main agent's
  frozen copies, its snapshots joining the league), aimed at self-play's narrowness (AI-Q12 (b)).
- **Imitation** of the fair system actor (decision 3): chosen for increment 5 (a warm start, AI-Q14)
  and continued in increment 6 (a better clone, the imitation loss; AI-Q15).
- **Scale:** the 60 M run at T7's mix (`configs/e06-t7.toml --set samples=60_000_000`, about 4
  hours; without the faithful actor now); the mana-price run (`configs/e06-mana.toml`).
- **Throughput** (E06 T5's open parts, the AI queue's inbox): the batch size and threads, the
  observations written in place, the kill preview's path searches.
- **The evaluation:** the VCMI oracle harness, if the evaluation needs it (E04's refinement).

## Non-goals

The fair system actor itself (the battle lane's); the adventure env and its AI (E13);
encounters v2; a trained network inside the engine, the server or a client; new scripted bots;
cloud compute; escapes in the reward; a new observation or action encoding, unless an increment's
evidence asks for one (then a `cross-lane` change with the battle lane, after the user's review).

## Tasks (outline; refine at the start of each increment; keep status current)

**Closed increments** (their task lists verbatim in `E25-archive.md`; the records in
`E25-log.md`):

| Increment | Closed | Tasks | Content |
|---|---|---|---|
| 1 | 2026-10-28 | T1a–T1c, T2a, T2b, T3 | the search's terminal by reward v1 and the true world (no gain: the evaluation limits it), shaping (+0.04, a tactics-phase loop), the faithful actor out of training; C ABI v35 |
| 2 | 2026-10-29 | T4a, T4b, T5, T6 | the tactics bound (the loop gone, no cost), the scale run (flat from 20 M; `e25-scale-1` #3 the current best) |
| 3 | 2026-11-02 | T7a–T7d, T8 | the network's architecture: a distillation bench (the wide MLP leads, the structured networks as configured fit ten times worse), the wide MLP in RL at #3's ladder (AI-Q12) |
| 4 | 2026-11-02 | T9a, T9b, T10a, T10b, T11 | the policy's sharpness: a temperature diagnostic (sharpening gains about 0.01) and the entropy bonus to 0 in training (`e25-sharp-1`: the same 0.01, no more) (AI-Q13) |
| 5 | 2026-11-03 | T12–T18 | the fair system actor: `system` the fair one everywhere (C ABI v36), the cloning bench (C ABI v37), the neutral kinds back and PPO from the clone (`e25-clone-1` #2: 0.454 against both system actors, the neutral set +0.282) (AI-Q14, AI-Q15) |

**At increment 1's checkpoint** (the user, 2026-10-28, T3): increment 2 is shaping, bounded,
then scale; `e25-shaping-1` (#2, 20 M) was the lineage's current best; `t7-1` stays the battle
AI v1 and the baseline. **Since T4b** (the user, 2026-10-29) **`e25-bound-1`** (#2, 20 M) is the
current best; **since T6** (the user, 2026-10-29) **`e25-scale-1` #3** (30 M, three tactics
rounds), the network the exit criteria are judged on. **At increment 3's checkpoint** (the user,
2026-11-02, T8): #3 stays the current best; increment 4 is the policy's sharpness and PPO's
settings. **At increment 4's checkpoint** (the user, 2026-11-02,
T11): **`e25-sharp-1` #2** (20 M, the entropy bonus to 0) the current best, its head-to-head
against #3 0.523 (T12). **At increment 5's checkpoint** (the user, 2026-11-03, T18, AI-Q15):
**`e25-clone-1` #2** (20 M of PPO from T16's clone of the fair actor) the current best, its
head-to-head against `e25-sharp-1` #2 0.564; increment 6 is better imitation.

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
| T19 | The teacher's ceiling: the fair actor's agreement with its own draws, per category | next (the sandbox) |
| T20 | The longer clone: the clone bench at a larger budget, read against T19's ceiling | after T19 |
| T21 | The imitation loss: PPO's auxiliary cross-entropy to the fair actor's choice, decaying to 0 | after T19 |
| T22 | The A/B: PPO from T20's student, with and without T21's loss, against `e25-clone-1` | after T20, T21 (the user's machine) |
| T23 | The checkpoint: T22 read, the current best, increment 7 | after T22 |

Scope and acceptance: the AI queue's entries.

## Exit criteria

1. The promoted agent meets decision 1's target: against the faithful system actor on the report
   set a win rate above 0.5 and a positive value efficiency; on the neutral set a value above the
   system actor's own in the player's seat; against Random, Aggressor and Greedy a win rate above
   0.5 and a positive value. Since increment 5 the fair system actor is reported beside it
   and defends the neutral set (AI-Q14).
2. It plays legitimately: it reads only its side's observation (a search through ADR-0025 point
   12a's worlds), and no faithful system actor took part in its training (its run records show
   the mix).
3. Every run the epic's results cite has its record committed; E06's guarantees (config-driven,
   resumable, gated, laddered) hold for whatever E25 adds, a search in training included.
4. `cargo xtask verify` passes: CI's full verify on Linux, `--with-original-data` on Windows; the
   Python tests on both (E06's criterion 6 as amended).

## Risks

- **The target may be out of reach:** the system actor reads the whole state, an information
  advantage the agent never has (in the hero kinds; a neutral side has no hidden hero), and E06's
  six levers ended at one ladder. Revisable at a checkpoint with evidence.
- **Training without the faithful actor** changes the neutral share's signal (`t7-1` met the
  neutral target with it); T2b's control measures what it costs.
- **The search's cost:** 17 to 38 decisions a second in Python in the sandbox; expert iteration
  needs the tree in Rust.
- **The GTX 1070** (8 GB, no tensor cores): a structured network costs samples per second (E06:
  3.4 times).
- **The encounter approximation** (E05's risk) carries over.
