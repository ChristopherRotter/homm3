# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increments 1 and 2 done;
increment 3 cut 2026-10-18 (T5a done, on its pull request):

1. **T5b The sampler** — waits for AI-Q4 (the assumptions, design §4.4). A `shared/` pull request
   first (`serde`, `toml`, `h3-heroes` for `h3-sim`); then `encounters-v1.toml`, the sampler, the
   batch's encounter source and C ABI v28, the frequency check, `batch-random-encounters-v1`.
2. **T6a The Python binding on Linux** (design §5) — independent of T5b; `uv run pytest` stays a
   lane command (the user, 2026-10-18).
3. **T6b Windows and PyTorch** — on the user's machine (PyTorch's index is refused elsewhere).

## Inbox

Taken over from the battle lane's inbox (2026-10-18; the items stay there too until the battle lane prunes them):

- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33's open point for E05 is answered (AI-Q3): the batch plays both sides as human players' (ADR-0025 point 6); the second morale roll stays the battle lane's question.
- 2026-10-18 (T1): the battle lane's proptest `walks_are_reported_hex_by_hex` found a state that differs with and without an event sink (its inbox, 2026-10-18). The env steps with `NoopSink`; once fixed, nothing to do here unless the fix changes `apply`'s signature.
- 2026-10-18 (T3c): `benches/README.md`'s workload table lacks `ai/batch-random-full` and `ai/batch-random-siege` (lane-only; 64 envs, threads 0, 1 000 calls, inside `h3_sim::batch::BattleBatch::step`): the file is a shared path, so a small `shared/` pull request adds the two rows.
