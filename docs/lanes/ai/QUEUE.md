# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (T1 to T9 in six increments; the
record in `E05-log.md`, the environment as built in `docs/07-simulation-and-ai.md` §1.1).

**E06** (`docs/epics/E06-battle-ai.md`) refined 2026-10-20 (AI-Q8): it starts beside E23, the
neutral share waits for E23's side without a hero.

1. **E06 T1 Design and ADR draft** — **Scope:** the epic's T1: `docs/epics/E06-design.md` and an
   ADR draft in `docs/lanes/ai/adr-drafts/` for the training pipeline (decisions 3, 4, 8, 9, 11):
   the run's config file and record (where committed records live), the resumable training state
   and its save cadence (atomic writes, the schedule in samples not wall time, the held-out set's
   disjointness without E05's set of every training seed), the held-out set v1 (per kind and
   side, sized for the targets' 95 % intervals), the SPRT's bounds, the league's sampling, the
   ladder from Python and `h3 ai eval`, the cut of the next increments. Read first: the epic file,
   `E05-log.md` §8 and §9, `python/h3env/train/`. **Acceptance:** the note and the draft
   committed; T2 refined in the epic file; stops for the user's review (no code). **Depends on:**
   nothing. **Status:** waiting (the user's review of `docs/epics/E06-design.md`, its §11, and the
   ADR draft `docs/lanes/ai/adr-drafts/training-pipeline.md`; drafted 2026-10-20 on
   `claude/nice-knuth-ekooly`). After the review: the answers into the note and the draft, the
   draft to a `shared/` pull request for its number, T1 into `E06-log.md`.
2. **E06 T2a Config and record** — **Scope:** the epic's T2a (design §1, §2, §4, §5.3 against
   v0). **Acceptance:** a run from a TOML config writes its typed metrics and `record.json`
   atomically; the held-out check needs no growing set; `python/records/` with the schema test;
   the torch-free tests pass in the sandbox, the torch ones on the user's machine (a lane
   command). **Depends on:** T1's review. **Status:** waiting (T1's review).
3. **E06 T2b The full checkpoint and `--resume`** — **Scope:** the epic's T2b (design §3).
   **Acceptance:** §3.4's tests. **Depends on:** T2a. **Status:** waiting (T1's review).
4. **E06 T2c C ABI v31** — **Scope:** the epic's T2c (the record's engine identity, N
   range-checked). **Acceptance:** the ABI's tests and the C smoke test, `h3env`'s
   `ABI_VERSION` 31, the full verify. **Depends on:** T1's review (independent of T2a, T2b).
   **Status:** waiting (T1's review).

## Inbox

Sorted by destination at E05's audit (T9a, 2026-10-19); the origin stays in brackets. Dropped as
done: BAT-Q33's point for E05 (AI-Q3, ADR-0025 point 6), the `h3-game` coverage gap of a pending
Pass offer (fixed by the adventure lane's fixed tests, #266).

**For E06** (training at scale, evaluation, search; the epic's T2 to T5 take them up):
- **Evaluation:** greedy play (the most likely heads) repeats itself against Random until the
  step limit (up to 17 % of the battles in T8b's sandbox run); the sampled policy never. E06's
  evaluation and any search agent need a rule for it (sampled play, or a tie-break or repetition
  rule); the 2 000-step limit is only a guard (T8b). The VCMI oracle harness was never built:
  revisit with E06's evaluation needs (E04 refinement, 2026-10-01).
- **Baselines and AI-Q7:** Greedy value beats Aggressor only 0.519 on the held-out episodes: the
  preview of its own kills buys little without spells or a plan; a stronger baseline (casts, a
  one-ply look at the opponent's reply) is E06's, and AI-Q7 (whether the agents get the preview)
  moved there (T7c; the user, 2026-10-19). The bots' fallback branches (the tactics phase, a
  commanded Catapult) have no targeted test (E05 T9b).
- **Encodings:** `h3_sim::action::encode` answers without a viewer (the same root as F-sec-2's
  leak, which the sim's `step` closes); it leaks nothing today (E04b T11b, the battle lane).
- **Performance**, each worth a look when a profile of training shows it: the flat observation
  costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is
  zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node
  hashing (E04b T11a); an encounter sample costs 50 µs (two heroes derived, each artifact through
  `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 % of an episode (T5b2); the
  trainer copies each call's observations (9 MB at 256 envs) into the rollout, which the batch
  could write in place (`H3BatchBuffers.obs` per call) (T8b; E06 T5, no ABI change: design §10). The battle lane's `full` workload
  is lopsided (8 attacker victories in 2 399 agent seeds; E04b T11a): the encounter workload
  (`batch-random-encounters-v1`) is the realistic one. On the user's Ryzen 9 3900X the step
  scales to 9.9 × at 1 024 envs on 24 threads but only 3.8 × at 64 envs (`E05-log.md`, T9c's
  table): E06's batch size and thread count from there.

- **Encounters v2** (with the adventure's logged encounters, decision 7; E05 T9b): A7's "+0.5 a
  week" in weeks 9 to 16 is rounded down in `encounters-v1.toml` (odd weeks half a level low);
  untested: A4's cap and redraws in weeks 1 to 6, the forts of weeks 2 to 6, A7's war-machine
  chance and class split, A6's upgraded share, the guards' anti-magic share.
- **The C ABI** (E05 T9b): `h3_batch_create` takes any N (an absurd one aborts on its
  allocation, which no `catch_unwind` catches; the binding range-checks it, a trusted local
  consumer, ADR-0011; E06 T2c); `h3env.tools.Env` keeps its handle after `H3_ERR_PANIC` (tests only).

**For E23** (the system actor; the neutral kinds' share): A3 grows a wandering stack by a tenth
of its count, VCMI 1.7.5's `CGCreature::newTurn` a tenth of its power (a thousand per creature:
5 creatures are 9 after seven weeks, the count's tenth leaves them 5); VCMI's growth makes
late-week stacks pass A4's cap and be drawn again, so decide both together (E05 T9b).

**For E13** (the adventure env's ABI): reading both of the adventure env's mask heads doubles
its step; the current state's queries belong in the env (the adventure lane, E08 T9b).
