# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increments 1 and 2 done;
increment 3 done 2026-10-19 (#253, #255, #257, #258); increment 4 (T8) done 2026-10-19 (#260,
#264; exit criterion 7 met).

1. **Increment 6's cut** at the user's checkpoint of increment 5 (T7 done 2026-10-19, #271,
   #274, #276): T9, the closure (the epic's outline: an independent review, the measurements
   on the user's machine, the docs), refined then; AI-Q7 open, not blocking.

## Inbox

- 2026-10-19 (the battle lane, its inbox's consolidation; for your information, nothing to do now): E04b's review (T11b, 2026-10-12) noted that `h3_sim::action::encode` answers without a viewer (the same root as F-sec-2's leak, which the sim's `step` closes); it leaks nothing today.

Taken over from the battle lane's inbox (2026-10-18; pruned there 2026-10-19):

- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33's open point for E05 is answered (AI-Q3): the batch plays both sides as human players' (ADR-0025 point 6); the second morale roll stays the battle lane's question.
- 2026-10-19 (T5b2): a sample costs 50 µs (two heroes derived, each artifact through `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 % of an episode: worth a look if a profile of training shows it.
- 2026-10-19 (T5b1's baseline): `master`'s coverage gate fails on the adventure lane's
  `crates/h3-game/src/state.rs:569` (`OfferKind::Pass` in the open decisions' hash: no fixed
  test hashes a pending Pass offer; 0 hits under `cargo llvm-cov -p h3-game`); #254's full verify
  covered it, so a property test reaches it only now and then. The adventure lane's to fix with a
  fixed test (suggested to the user as a task).
- 2026-10-19 (T8b): greedy play (the most likely heads) repeats itself against Random until the
  step limit (up to 17 % of the battles in the sandbox run); the sampled policy never. E06's
  evaluation and any search agent need a rule for it (sampled play, or a tie-break / repetition
  rule), and the 2 000-step limit is only a guard.
- 2026-10-19 (T8b): the trainer copies each call's observations (9 MB at 256 envs) into the
  rollout; the batch could write them there in place (`H3BatchBuffers.obs` per call) if a
  profile on the user's machine shows the copy.
- 2026-10-19 (T7c): Greedy value beats Aggressor only 0.519 on the held-out episodes: the
  preview of its own kills buys little without spells or a plan; a stronger baseline (casts, a
  one-ply look at the opponent's reply) is E06's, AI-Q7 decides whether the agents get the
  preview.
