# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increments 1 and 2 done;
increment 3 done 2026-10-19 (#253, #255, #257, #258); increment 4 cut 2026-10-19: T8 before T7
(T8a, T8b done 2026-10-19).

1. **T8c The smoke run** on the user's machine (the GTX 1070; epic, increment 4; design §8):
   `in progress` 2026-10-19. The first hour (naive self-play) peaked at 0.779 and ended at
   0.721 (`E05-log.md`); the opponent pool is built. Next: the hour again with the pool, after
   its pull request merged: `cargo build --release -p h3-ffi`, `uv sync --group train`, `uv run
   --group train python -m h3env.train --minutes 60 --out runs/pool` in `python/`. Exit
   criterion 7: the final evaluation's sampled `win_rate` at least 0.8 (design §8.2). Record the
   evaluations' lines in `E05-log.md`. Then increment 4's checkpoint (T7 next).

## Inbox

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
