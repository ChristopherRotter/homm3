# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increment 1: T1 done (ADR-0025 accepted 2026-10-18), then T2.

### E05 T2 — Scenarios through the C ABI

- **Scope:** the epic's T2 (ADR-0025 point 9, `docs/epics/E05-design.md` §4.1): `h3_env_create_scenario` from a scenario TOML with the built-in packs, every place including sieges; C ABI v26; the C smoke test.
- **Acceptance:** the Definition of Done (`docs/05-testing.md`); a test per place (field, special battlefield, garrison, town) through the ABI; F-robust-2 closed.
- **Depends on:** nothing (T1 accepted).

## Inbox

Taken over from the battle lane's inbox (2026-10-18; the items stay there too until the battle lane prunes them):

- 2026-09-30 (E01): batch stepping over many environments, and loading packs and scenarios through the ABI (E05 T2, T3).
- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33's open point for E05 is answered (AI-Q3): the batch plays both sides as human players' (ADR-0025 point 6); the second morale roll stays the battle lane's question.
- 2026-10-18 (T1): the battle lane's proptest `walks_are_reported_hex_by_hex` found a state that differs with and without an event sink (its inbox, 2026-10-18). The env steps with `NoopSink`; once fixed, nothing to do here unless the fix changes `apply`'s signature.
