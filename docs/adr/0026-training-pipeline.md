# ADR-0026: The training pipeline: runs, their resumable state and records, the evaluation gate, the ladder, the league

- Status: Accepted (E06 T1; the user, 2026-10-20)
- Date: 2026-10-20
- Deciders: user + AI lane session

Detail: the code documentation (`python/h3env/train/`), `python/README.md`,
`docs/07-simulation-and-ai.md` §1.2 and [`docs/epics/E06-log.md`](../epics/E06-log.md) (the design
note `E06-design.md` was deleted when E06 closed, 2026-10-27; its text is kept there, "The design
note at closure": the run §1 to §4, the held-out sets §5, the gate §6, the ladder §7, the league
§8).

## Context

E06 (AI-Q8) scales E05's PPO (`python/h3env/train/`, ADR-0025) to runs of hours on the user's
machine that run, record, evaluate and promote on their own and survive an interruption
(decisions 3, 4, 8, 9, 11 of the epic file). E05's trainer budgets wall time, saves only the
network and the optimizer, keeps every training episode's seed to prove its evaluation held out,
evaluates against Random alone and writes its records into an ignored directory.

## Decision

1. **A run is a directory and a config file.** TOML in (defaults in code, the file states what
   differs, `--set` overrides one key, an unknown key is an error); the resolved config goes into
   the record. `--resume <run>` may change only what changes no training (the budget, threads,
   the device, the checkpoint cadence); anything else is a new run, `--init-from <network>`, its
   parent recorded.
2. **Every training cadence counts samples** (learner decisions that trained): the budget, the
   learning rate's schedule (its horizon fixed at the start), the league's snapshots, the gates.
   Wall time decides only when to checkpoint.
3. **The checkpoint is the whole training state** at an update boundary: the config, the
   counters, the network, the optimizer, every random generator, the league, the gate's state
   (the best, the promoted networks, a pending gate or ladder) and the records' position. Not
   the battles in flight: a resume discards them and starts a new segment whose batch seed is
   derived from (seed, segment). Saved every few minutes, before a gate, at Ctrl+C (after the
   update in progress) and at the end; every file written atomically (a temporary file, fsync,
   replace). A resumed run is reproducible from its records, not bitwise.
4. **Held-out sets are fixed and versioned:** the report set v1 (per player kind and side, 500
   games each, stratified so the overall rate weighs the kinds as the distribution does) for the
   ladder and the targets; the gate set v1 (the default distribution, at most 4 000 pairs) for
   promotion; E05's set stays as v0. Disjointness: each training episode's seed is checked
   against the held-out sets' (a hit stops the run), and no segment's batch seed is one of
   theirs. No growing set of training seeds.
5. **Evaluation plays the sampled policy**, its generator seeded per set and side alike for every
   network; a draw or a truncation is no win.
6. **Promotion by an SPRT** of the candidate (the learner every `gate.every` samples) against the
   current best on the gate set: each pair one battle played from both sides, the candidate's
   pair score tested by a generalized SPRT (normal approximation), `H0: elo = elo0`, `H1: elo =
   elo1`, error rates α and β, at most `max_pairs`; the cap rejects. Defaults: 0, 20, 0.05, 0.05,
   4 000. A run starts with best #0: the untrained network or the parent's.
7. **The ladder is played for every promoted network** (and #0, and the final best on the full
   set): against Random, Aggressor, Greedy, E23's policy once it exists, and the previous promoted
   network, on the report set; win rates per kind and side and overall with their 95 %
   intervals, and the value efficiency. Decision 7's targets are judged on the lineage's latest
   promoted network: each point estimate at or above its target. Glicko-2 ratings stay in `h3 ai
   eval` alone, outside promotion and targets.
8. **The league is PFSP over the learner's past:** members are snapshots every few million
   samples, the promoted networks and (T7) the system actor; an episode is self-play or against a
   member on a random side; members are weighted by how often they still beat the learner
   (`(1 − p)^2`), at most 4 of them live per rollout (bounding the forward passes per call), at
   most 16 kept. Exploiters only if a check shows the agent exploitable. Bots never train.
   **Amended 2026-10-27** (the user, E25's refinement, AI-Q10): the faithful system actor
   (ADR-0027, which reads the whole state) trains no longer: no league member, no defender of the
   neutral kinds in training, no `system_share`; it stays the target and a ladder opponent. A
   fair variant that decides on its side's observation (the battle lane's, later) may train, and
   imitating it is a candidate.
9. **Records are committed:** each run's `record.json` (identity: config, commit, ABI and
   encodings' versions, rulesets' content hashes, the encounter table's version; segments;
   metrics as a short curve; every gate, promotion and ladder) as `python/records/<run>.json`,
   checked by a test; configs in `python/configs/`. Checkpoints and networks stay outside git.

## Consequences

- `python/h3env/train/` gains a torch-free lifecycle module (config, records, atomic writes,
  segments, the held-out check), testable without torch; the trainer's state needs torch.
- C ABI v31 (E06 T2c) exposes the record's engine identity; no encoding changes.
- `h3 ai eval` gains `--set` (v0, report v1) and a JSON output; the bots' table on the report set
  is committed and regenerated when the bots move.
- Evaluation costs training time (gates and ladders, estimated a quarter to a third early in a
  run); T3b measures and sets the cadence so it stays under about 20 %.

## Alternatives considered

- **Wall-time budgets and schedules** (E05): a resumed or slower run would train differently.
- **Saving the battles in flight:** the engine's state across processes, for minutes of play.
- **A fixed-size head-to-head gate** (AlphaZero's 55 % on 400 games): no error rates; the SPRT
  decides large gains in under a hundred pairs and small ones only when they are real.
- **Promotion on the report set or against the bots:** selects on the set that measures the
  targets; head-to-head is what a league improves.
- **A second Glicko-2 in Python:** two implementations to keep equal for a report.
- **Every pool member live** (E05): forward passes grow with the league.

## Revision history

- 2026-10-20: Accepted (E06 T1 review: every point of the design note's §11 as proposed).
- 2026-10-27: The working-detail line after E06's closure (the design note deleted).
- 2026-10-27: Point 8 amended (E25's refinement, AI-Q10): the faithful system actor trains no
  longer.
