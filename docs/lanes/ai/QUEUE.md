# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increment 1: T1, then T2 after the user's review.

### E05 T1 — Design and ADR draft (drafted 2026-10-18; in the user's review, PR on `ai/e05-t1`)

- **Scope:** the epic's T1 (a) to (g): the batch API, the seats, the reward against `BattleOutcome`, the scenario description and the encounter distribution's config shape, the Python side, the information boundary against the original's battle screen, the benchmark workloads. Output: `docs/lanes/ai/adr-drafts/battle-env.md` with worked reward examples, and questions as `AI-Q<n>` in OPEN-QUESTIONS.
- **Acceptance:** the draft decides each of (a) to (g) or names the question that blocks it; no code. Then **stop** for the user's review.
- **State:** `docs/lanes/ai/adr-drafts/battle-env.md`, `docs/epics/E05-design.md`, AI-Q2 and AI-Q3. After the review: apply the user's changes, then the ADR's numbering (a `shared/` pull request) and T2.
- **Depends on:** nothing.

### E05 T2 — Scenarios through the C ABI

- **Scope:** the epic's T2: an env from packs and a scenario, every place including sieges; C ABI v26; the C smoke test.
- **Acceptance:** the Definition of Done (`docs/05-testing.md`); a test per place (field, special battlefield, garrison, town) through the ABI; F-robust-2 closed.
- **Depends on:** T1 accepted by the user.

## Inbox

Taken over from the battle lane's inbox (2026-10-18; the items stay there too until the battle lane prunes them):

- 2026-09-30 (E01): batch stepping over many environments, and loading packs and scenarios through the ABI (E05 T2, T3).
- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33 (open): who commands a side; with E05 T1's seats.
