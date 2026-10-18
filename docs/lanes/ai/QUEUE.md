# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increment 1 done (T1, T2).
Increment 2 (cut with the user 2026-10-18): T3a and T3b done, then the tasks below in order;
the user's checkpoint after T3d.

### E05 T3c — Threads and benchmarks

- **Scope:** `h3_sim::batch::pool` (ADR-0025 point 3, design §1): T std threads, env i in shard
  `i * T / N`, parked between calls, T = 0 the caller's thread; `BattleBatch` steps, resets and
  answers the head calls through it. `h3 ai bench --envs N --threads T --calls K` (`h3-cli`'s
  `src/ai/`, one dispatch line in `main.rs`: `cross-lane`, the user's consent 2026-10-18) prints
  steps/s; the gated workloads `benches/ai/batch-random-full.toml` and `batch-random-siege.toml`
  (design §7: 64 envs, one thread, 1 000 calls of random legal actions from the batch's masks,
  counted inside `BattleBatch::step`, `lane_only`).
- **Acceptance:** the determinism property for random T (a batch steps as N single envs whatever
  its thread count); wall-time steps/s on the user's Ryzen 9 3900X for T = 1, 6, 12, 24 and
  N = 64, 256, 1 024 in the epic log; the single-env gates unchanged; the Definition of Done.
- **Depends on:** T3b.

### E05 T4 — Reward v1

- **Scope:** ADR-0025 point 8, design §3: `r_s = (K_o − K_s − p·(m_s − m_o)) / S` from the
  episode's `BattleOutcome` (K the value a side lost: its army and raised stacks at the start minus
  the survivors, plus destroyed war machines; created stacks count nothing; m the mana spent; S
  both sides' starting armies and war machines), terminal, 0 at a truncation; the currency
  (`ai_value` or the creatures' cost) and the price p (default 10 per spell point) in the batch's
  config; `EnvReport::reward` carries it.
- **Acceptance:** tests against the hand-computed outcomes of design §3.2; the zero-sum property
  in self-play; the Definition of Done.
- **Depends on:** T3b.

### E05 T3d — The C ABI v27

- **Scope:** design §1's functions and structs (`h3_batch_create` with the config and its reward
  settings, `h3_batch_add_scenario` with a scenario's text and seats, `h3_batch_reset`, the three
  head calls, `h3_batch_step`, `h3_batch_episode_seed`, `h3_batch_destroy`; `H3BatchBuffers` with
  the per-env arrays converted from `EnvReport`); C ABI v27; the C smoke test runs a batch.
- **Acceptance:** Rust-side ABI tests of every function and error, a batch through the ABI equal
  to the Rust batch; the Definition of Done.
- **Depends on:** T3c, T4.

## Inbox

Taken over from the battle lane's inbox (2026-10-18; the items stay there too until the battle lane prunes them):

- 2026-09-30 (E01): batch stepping over many environments (E05 T3; loading packs and scenarios through the ABI is done, T2).
- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33's open point for E05 is answered (AI-Q3): the batch plays both sides as human players' (ADR-0025 point 6); the second morale roll stays the battle lane's question.
- 2026-10-18 (T1): the battle lane's proptest `walks_are_reported_hex_by_hex` found a state that differs with and without an event sink (its inbox, 2026-10-18). The env steps with `NoopSink`; once fixed, nothing to do here unless the fix changes `apply`'s signature.
