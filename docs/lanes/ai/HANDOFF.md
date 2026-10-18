# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05** (`docs/epics/E05-battle-env.md`): in progress. **Increment 1 done** (2026-10-18): T1
  (ADR-0025; detail in `docs/epics/E05-design.md`, record in `E05-log.md`) and T2
  (`h3_env_create_scenario`, C ABI v26). **Increment 2** (T3 and T4, cut with the user): T3a
  (kernel streams, **#250** `shared/kernel-batch-streams`: merge first) and T3b
  (`h3_sim::batch`, one thread; `ai/e05-t3b`, stacked on #250) done. **In progress
  (2026-10-18, branch `claude/e05-continuation-4tjnq2`, one pull request at the end):** T3c
  committed (`batch::pool`, `h3 ai bench`, `benches/ai/`); next T4, T3d (`QUEUE.md`), then the
  docs and the user's checkpoint.
- **Order** (BACKLOG): E05, then E23 (the battle lane: the system actor's policy), then E06.
- **Versions:** action encoding v13, observation encoding v21, replay format v29 (taken over from the battle lane), C ABI v26 (T2) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q4, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

**Ownership**
- The battle lane still writes rules the encodings show: a new observation feature or action entry comes as a `cross-lane` pull request from the battle lane with this lane's version bumps (`docs/08-work-process.md`, "Lanes"). This lane's own changes to `h3-battle` (an engine query the env needs) are `cross-lane` the other way.
- `h3-cli` is the battle lane's except `src/ai/`; its golden replays (`fixtures/`) and `benches/battle/` stay with battle, and change when the encodings or the replay format change: regenerate them deliberately (`fixtures/README.md`) in the same pull request.

**Environment**
- An agent sees the projected observation only (AI-Q1 decision 10); `fork` and `state_hash` read the full state, for tools and training (the C ABI is for trusted local consumers, ADR-0011; never forward the hash to a client).
- `BattleSetup.human` and `.difficulty` change the computer's automatic choices (the turrets, the automatic Ballista, the moat avoidance; ADR-0023 point 7): the batch sets `human = [true, true]` for every episode (ADR-0025 point 6) and masks Retreat and Surrender out (point 7); `BattleEnv` and the single-env C functions stay as they are (replays and the battle benchmarks use escapes).
- `verify --scope ai` skips the golden replays (their crate `h3-cli` is the battle lane's) and the battle benchmarks: after a change to `h3-sim` run `cargo xtask determinism` and `cargo xtask bench-check --workload random-rollouts-full` too.
- `legal_commands` lists no casts: the cast heads come from `cast_targets` × `second_targets` (the battle lane's trap; `h3_sim::action`).
- The batch (`h3_sim::batch`) skips an episode that ends before an external seat could act and gives up after 64 in a row (no side to move until a reset): a scenario whose internal seat always acts first and ends it (a step limit of one) does that. Its per-env values come back as `EnvReport`s; T3d converts them for the C arrays.
- Scenario texts are parsed and packs compiled in `h3-ffi` (`src/scenario.rs`), never in `h3-sim`: `arch.toml` gives `h3-sim` `h3-pack-battle` for tests only. Each `h3_env_create_scenario` compiles its packs (about 18 ms for `base`, `ab`, `sod` in release, ten times that in debug).

**Python**
- PyTorch 2.14 (CUDA 12.6 build) is the last prebuilt release for the user's GTX 1070: pin it. The user's system Python is older than 3.11: use a venv.

## Recent sessions

### 2026-10-18 — E05 increment 2 cut; T3a and T3b

- With the user: increment 2 is T3 and T4 (T3a–T3d, T4; the C ABI last so that it lands once with its reward), this session T3a and T3b, the `h3 ai` dispatch line in `h3-cli`'s `main.rs` a `cross-lane` change in T3c.
- T3a: kernel streams `seat(i)` and `env(i)` (`shared/kernel-batch-streams`). T3b: `BattleBatch` on one thread with the determinism property against single envs (decisions in design §1, "Built (T3b)"). Full local verifies stand in for CI.

### 2026-10-18 — E05 T2, scenarios through the C ABI

- The user confirmed T2 alone as the session's increment, and one line in the battle lane's inbox (`cross-lane`) for F-robust-2's closure.
- Built `h3_env_create_scenario` (C ABI v26) in `h3-ffi`; `h3_env_create_synthetic` on the same path. Tests through the ABI on `base`, `ab`, `sod`: a field, all twelve special battlefields, an anti-magic garrison (no hero's cast legal), a Castle under siege (aimed Catapult shots legal); the seed; the errors. The C smoke test plays a siege from text.
- CI cannot start (the account's billing; GitHub's annotation on `master`'s last run): the full local verify stands in, recorded in the pull request. The baseline's full verify of `master` (code 30ecdd57) passed every gate: its proptest cases missed the `h3-battle` sink defect, which is still open (the battle lane's inbox; the gate fails only when proptest finds such a case).

### 2026-10-18 — E05 T1, the design; ADR-0025 accepted

- Drafted the batch API (same-step automatic reset, per-env status, head calls per batch, std-thread shards in `h3-sim`, seeds per env stream), seats per scenario side (internal policies a closed enum; E23's function shared with `h3-game`), a first reward with the adventure's stakes, the scenario TOML unchanged, the encounter distribution's shape and census, ctypes with zero-copy numpy, uv and `uv.lock` for PyTorch 2.14, the benchmarks.
- The user: (f) needs no check (every per-player observation is a human player's information); `verify` cleans up after itself (the coverage gate's clean). At the review (AI-Q2, AI-Q3): the reward kills the most and loses the least by `ai_value` (or cost) with a mana price, nothing from the adventure; no retreat or surrender for now; `human` true for both sides. Accepted as ADR-0025 with these changes; the draft removed.
- **Found on `master`:** `h3-battle`'s proptest `walks_are_reported_hex_by_hex` (properties.rs:1924) found a state that differs with and without an event sink: master's full verify fails coverage until the battle lane fixes it; filed in its inbox with the case.
