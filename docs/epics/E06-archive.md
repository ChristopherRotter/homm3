# E06 — archive: closed increments 1 to 6

Moved out of `E06-battle-ai.md` on 2026-10-26 (increments 1 to 5 at increment 6's refinement,
increment 6 at its checkpoint), verbatim, so that the epic file holds what is open and the
decisions that apply to the whole epic (decisions 1 to 12 stay there). Each task's full record
is in `E06-log.md` (grep its id).

## Tasks of increments 1 to 6

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
- [x] **T7d The run** — done 2026-10-23 (`t7-1`, a third neutral, 20 M samples): the report
  set as `t5-3`'s (0.828, 0.551, 0.528), the system actor 0.374, the neutral set 0.972 at value
  +0.245 (the target +0.285 not met); the record in `E06-log.md`.

**Increment 6:** T6b, then T6c (refined 2026-10-26, the user: the system actor as a training
opponent in the player kinds, decision 9 amended; the System side stays human); after T6c's
negative result T6d, then T6e (the user, 2026-10-26: AI-Q7 (b) part 2).

- [x] **T6b The system actor as an opponent** — done 2026-10-26: `[league] system_share` (the
  batch's entries split by `ppo.episode_entries`, no ABI change), the update record's and the
  curve's `system` (the learner's score by side), `configs/e06-t6b.toml`; the record in
  `E06-log.md`.
- [x] **T6c The A/B run** — done 2026-10-26, negative: `t6b-1` (the user's machine, 20 M
  samples) ends at `t7-1`'s ladder (0.844, 0.539, 0.526, the system actor 0.371); its play
  against the system actor unchanged (T6a's analysis); the record in `E06-log.md`.
- [x] **T6d The kill preview in the network** (AI-Q7 (b) part 2) — done 2026-10-26: `[network]
  preview` (each legal strike's logit gains a bias from its previewed kills, design §13), every
  batch a reading network plays writes the preview, `configs/e06-t6d.toml`; the record in
  `E06-log.md`.
- [x] **T6e The A/B run** — done 2026-10-26, negative: `t6d-1` (the user's machine, 20 M
  samples) ends at `t7-1`'s ladder (0.821, 0.553, 0.528, the system actor 0.365, the neutral
  set's value +0.253); its targeting better, its battles longer (it strikes less, wanders
  more); AI-Q7 closed, the preview off by default; the record in `E06-log.md`.
