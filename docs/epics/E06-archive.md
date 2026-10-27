# E06 — archive: closed increments 1 to 7

Moved out of `E06-battle-ai.md` on 2026-10-26 (increments 1 to 5 at increment 6's refinement,
increment 6 at its checkpoint) and 2026-10-27 (increment 7 and the later tasks T5 and T6 at its
checkpoint), verbatim, so that the epic file holds what is open and the decisions that apply to
the whole epic (decisions 1 to 12 stay there). Each task's full record is in `E06-log.md` (grep
its id).

## Tasks of increments 1 to 7

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

**Increment 7, T8 the search** (refined 2026-10-26, the user: the plateau taken as shown after
T6e; measure a decision-time search before any training change; determinized from the start;
the tree in Python first; `h3-battle`'s part `cross-lane` in T8b; design §14):

- [x] **T8a The design** — done 2026-10-26: design §14, ADR-0025 point 12's amendment as a
  draft (`docs/lanes/ai/adr-drafts/0025-amendment-search.md`, accepted by the user the same day);
  the record in `E06-log.md`.
- [x] **T6f The turn order as an input** (the user, 2026-10-26, before T8b; after the
  turn-order probe) — done 2026-10-26: `h3_battle::turn_order` (`cross-lane`), observation
  v22, `[network] order`, `configs/e06-order.toml`; the record in `E06-log.md`.
- [x] **T6g The A/B run `order-1`** — done 2026-10-27 (the user's run): negative, the ladder
  `t7-1`'s and the play alike; the trunk still drops the order (the probe again); the record in
  `E06-log.md`.
- [x] **T8b The plumbing** — done 2026-10-27: C ABI v34 (`h3_batch_determinize`,
  `h3_env_reseed`), `h3-battle`'s `BattleState::determinized` and `reseed` (`cross-lane`),
  `h3env.search` with `--time` and `--report`; one gap (AI-STUB-1); the record in `E06-log.md`.
- [x] **T8c The measurement** — done 2026-10-27 in the sandbox with `order-1`'s best (at
  `t7-1`'s level): the search's σ fixed (a fixed scale left the prior alone); no gain against
  the system actor at N 16 and 64 nor on the neutral set at N 64; the record in `E06-log.md`.
- [ ] **T8d Expert iteration** — only if T8c gains: not indicated (T8c); the user's checkpoint.

Closed 2026-10-27 at the checkpoint (the user): T8d not built (a search in Rust trained on its
own choices is E25's candidate), decision 7's targets revised, T9 next.

**Later increments, closed with increment 7** (2026-10-27; the user: E06 ends with T9):

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

T5's open parts (the mana-price run `t5-4`, the batch size and threads, the observations in
place) were not done: T9a sorts them to their destinations. T6 ended with PPO's plateau shown
(increments 6 and 7) and decision 7's targets revised, which `t7-1` meets.
