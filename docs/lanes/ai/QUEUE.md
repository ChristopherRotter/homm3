# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increments 1 and 2 done;
increment 3 cut 2026-10-18 (T5a done, #253; T5b1 and T5b2 done, #255; T6a done 2026-10-19):

1. **T6b Windows and PyTorch** — the binding's tests on the user's Windows machine (`uv sync`,
   `uv run pytest` in `python/`) and the group `train`, torch 2.14 from
   `https://download.pytorch.org/whl/cu126`, in `uv.lock` (design §5). The sandbox's proxy
   refused `download.pytorch.org`, `download-r2.pytorch.org` and `pytorch.org` again on
   2026-10-19 in a new session with `*.pytorch.org` allowed; the user was asked to add the hosts
   by name. Else the lock is made on the user's machine.

## Inbox

Taken over from the battle lane's inbox (2026-10-18; the items stay there too until the battle lane prunes them):

- 2026-10-11 (E04b T11a): the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node hashing; the `full` workload is lopsided (8 attacker victories in 2 399 agent seeds).
- 2026-10-01 (E04 refinement): the VCMI oracle harness was never built; revisit with E06's evaluation needs.
- 2026-10-11 (the adventure lane, E08 T9b): reading both of the adventure env's mask heads doubles its step; the current state's queries belong in the env (E13 with the adventure's ABI).
- BAT-Q33's open point for E05 is answered (AI-Q3): the batch plays both sides as human players' (ADR-0025 point 6); the second morale roll stays the battle lane's question.
- 2026-10-18 (T1): the battle lane's proptest `walks_are_reported_hex_by_hex` found a state that differs with and without an event sink (its inbox, 2026-10-18). The env steps with `NoopSink`; once fixed, nothing to do here unless the fix changes `apply`'s signature.
- 2026-10-19 (T5b2): a sample costs 50 µs (two heroes derived, each artifact through `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 % of an episode: worth a look if a profile of training shows it.
- 2026-10-19 (T5a's verify): the battle lane's proptest `every_step_preserves_the_invariants`
  (`h3-battle`) failed once under coverage: `state.is_over() || state.active() == Some(mover) ||
  killed || enraged` broke in a siege (faction 0, fort 1, no moat); its regression line is in
  #253's description. `h3-battle` is untouched by T5a; the battle lane's to fix, like
  `walks_are_reported_hex_by_hex` (the gate fails only when proptest finds such a case).
- 2026-10-19 (T5b1's baseline): `master`'s coverage gate fails on the adventure lane's
  `crates/h3-game/src/state.rs:569` (`OfferKind::Pass` in the open decisions' hash: no fixed
  test hashes a pending Pass offer; 0 hits under `cargo llvm-cov -p h3-game`); #254's full verify
  covered it, so a property test reaches it only now and then. The adventure lane's to fix with a
  fixed test (suggested to the user as a task).
- 2026-10-19 (T6a): a head call (`BattleBatch::spell_mask`, `target_mask`, `second_mask`) answers
  every env and costs about 5.7 µs per env (0.37 ms for 64, measured from Python beside a verify):
  almost all of it the non-cast envs' `action::legal_entry`, which regenerates the legal commands
  to answer "0 only". The batch could answer those from the entry mask it wrote at the last call;
  a policy that casts in any env of a call pays three head calls.
