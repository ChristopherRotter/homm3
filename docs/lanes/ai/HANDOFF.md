# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05** (`docs/epics/E05-battle-env.md`): in progress. **T1 done** (2026-10-18): **ADR-0025** accepted (the batch API, seats, both sides human, no escapes, reward v1 without the adventure, scenarios, the encounter distribution, Python, benchmarks); detail in `docs/epics/E05-design.md`, record in `E05-log.md`. Open: **#248** (`shared/adr-0025-battle-env`: the ADR, its index row, `07-simulation-and-ai.md`; merge first) and **#247** (`ai/e05-t1`, `cross-lane`: the epic, the design note, OPEN-QUESTIONS, the battle inbox's two entries). **Next:** T2 (an env from a scenario, C ABI v26; `QUEUE.md`).
- **Order** (BACKLOG): E05, then E23 (the battle lane: the system actor's policy), then E06.
- **Versions** (taken over from the battle lane, unchanged): action encoding v13, observation encoding v21, replay format v29, C ABI v25 (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q4, AI-QUIRK-1, AI-STUB-1.
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

**Python**
- PyTorch 2.14 (CUDA 12.6 build) is the last prebuilt release for the user's GTX 1070: pin it. The user's system Python is older than 3.11: use a venv.

## Recent sessions

### 2026-10-18 — E05 T1, the design; ADR-0025 accepted

- Drafted the batch API (same-step automatic reset, per-env status, head calls per batch, std-thread shards in `h3-sim`, seeds per env stream), seats per scenario side (internal policies a closed enum; E23's function shared with `h3-game`), a first reward with the adventure's stakes, the scenario TOML unchanged, the encounter distribution's shape and census, ctypes with zero-copy numpy, uv and `uv.lock` for PyTorch 2.14, the benchmarks.
- The user: (f) needs no check (every per-player observation is a human player's information); `verify` cleans up after itself (the coverage gate's clean). At the review (AI-Q2, AI-Q3): the reward kills the most and loses the least by `ai_value` (or cost) with a mana price, nothing from the adventure; no retreat or surrender for now; `human` true for both sides. Accepted as ADR-0025 with these changes; the draft removed.
- **Found on `master`:** `h3-battle`'s proptest `walks_are_reported_hex_by_hex` (properties.rs:1924) found a state that differs with and without an event sink: master's full verify fails coverage until the battle lane fixes it; filed in its inbox with the case.

### 2026-10-18 — E05 refined, the lane opened

- With the user (AI-Q1): battle only; the AI lane opens; PyTorch 2.14 over the C ABI (Q5); opponents are self-play and the system actor, heuristic bots only baselines; casts by one call per head; the flat observation first; an encounter distribution weighted by approximate frequency, from the shipped maps until real games give it; the reward as a zero-sum value balance; the human player's view only; exit by a smoke run that learns. Recorded in the epic file, BACKLOG, `07-simulation-and-ai.md`, OPEN-QUESTIONS AI-Q1 and Q5.
- The user's four own maps were removed from `original-data/maps.tar` and `original-data/install/Maps` (`experiment.h3m`, `Untitled.h3m`, `_modified_All for One.h3m`, `_modified_Terrible Rumor.h3m`): 160 shipped maps remain (the adventure lane's inbox has the note on its counts of 164).
