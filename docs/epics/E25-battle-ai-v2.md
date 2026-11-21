# E25 — Battle AI v2

Status: **in progress**, refined 2026-10-27 (AI-Q10); increments 1 to 7 closed 2026-10-28, 29, 11-02, 11-03, 11-04 and 11-10, increment 8 (probes, then one long run) refined 2026-11-10 (AI-Q17) · Lane: **ai** ·
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
   *Amended 2026-11-15* (the user at T35, the probes' checkpoint): a training signal may weigh the
   outcome beside reward v1 (`[ppo] win_bonus`, its probe T35's); the evaluation stays reward v1.

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
- **A network that reads the battle's structure, built and tuned properly** (the user at T8;
  chosen for increment 7, AI-Q16):
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
| 6 | 2026-11-04 | T19–T23 | better imitation: the teacher's ceiling (0.972: the gap is the network's), a 16 M clone (0.576), the imitation loss; PPO from the clone with the loss (`e25-imitate-1` #2: 0.458 and 0.462 against the system actors, the neutral set +0.287) (AI-Q15, AI-Q16) |
| 7 | 2026-11-10 | T24–T29 | a network that reads the battle's structure: `hexconv` 96 × 6 with the legal mask, its clone 0.809 (the MLP's 0.574), PPO from it with the imitation loss (`e25-structure-1` #2: 0.511 and 0.507 against the system actors, the neutral set +0.302; 0.552 head to head against `e25-imitate-1` #2) at 22 times the MLP's time (AI-Q16, AI-Q17) |
| 8 | 2026-11-18 | T30–T36 | probes, then one long run: the update's speed (98 to 67 s), no gain from size, PPO's settings, the search at play, a KL anchor to #2 or the outcome's weight; the anchor's regime to 20 M level with #2 (0.511 against the faithful actor): #2 stays, the setup's plateau; the early stop (`[gate] futility`) (AI-Q17, AI-Q18) |
| 9 | 2026-11-20 | T37–T41 | a richer observation: v23's reach block from a viewer-safe query (C ABI v38), `[network] reach` and `preview` for the hex kinds, the bench's preview rows and `same_start`; T39's bench: neither arm passed (0.457, 0.460, 0.459 ± 0.018 against the fair actor), T40 not run (AI-Q19, AI-Q20) |

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
head-to-head against `e25-sharp-1` #2 0.564; increment 6 is better imitation. **At increment 6's
checkpoint** (the user, 2026-11-04, T23, AI-Q16): **`e25-imitate-1` #2** (20 M of PPO from T20's
clone with the imitation loss) the current best, its head-to-head against `e25-clone-1` #2 0.534;
increment 7 is a network that reads the battle's structure. **At increment 7's checkpoint** (the
user, 2026-11-10, T29, AI-Q17): **`e25-structure-1` #2** (20 M of PPO from T27's `hexconv`
clone with the imitation loss) the current best, its head-to-head against `e25-imitate-1` #2
0.552; increment 8 is probes, then one long run. **At increment 8's checkpoint** (the user,
2026-11-18, T36, AI-Q18): **#2 stays the current best** (no probe nor the long run passed it);
increment 9 is a richer observation; a long run stops early (`[gate] futility`). **At T37's
review** (the user, 2026-11-18, AI-Q19): v23 is the reach block (the rules' reach as the viewer
may know it, this lane's `cross-lane` query), the kill preview for `hexconv` a second bench arm,
T39's rule the clone's ladder +0.02 (`E25-log.md` T37). **At increment 9's checkpoint** (the user, 2026-11-20,
T41, AI-Q20): neither v23 arm passed the bench's rule; **#2 stays the current best**; increment
10 is the ceiling (rollouts of the fair actor, both worlds).

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
| T44 | Where #2 diverges: #2's decisions against the fair actor replayed, every command valued by the same rollouts; the value its choice leaves by category, beside the fair actor's own | next |
| T45 | Increment 10's checkpoint: the headroom, the information's cost, the divergence; the next lever | after T43, T44 |

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
- **The gap may be information observation v22 lacks** (no reachability or distances in its hex
  planes): if the structured networks and the MLP with the mask all stall well below the
  teacher, that is evidence for an encoding change (a `cross-lane` change after the user's
  review, T29).
- **The encounter approximation** (E05's risk) carries over.
