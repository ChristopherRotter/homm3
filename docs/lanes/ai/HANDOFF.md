# AI lane — HANDOFF

The AI lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Opened 2026-10-18** with E05's refinement (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` stays the adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05** (`docs/epics/E05-battle-env.md`): in progress. **T1 drafted** (2026-10-18): the ADR draft `adr-drafts/battle-env.md` (13 points), the design note `docs/epics/E05-design.md` (the C sketch, reward v1 with worked examples, the distribution's shape, the Python layout), AI-Q2 (the reward's defaults) and AI-Q3 (an agent's `human` flag), on branch `ai/e05-t1`, **waiting for the user's review**. **Next:** apply the review, number the ADR (a `shared/` pull request), then T2 (an env from a scenario, C ABI v26).
- **Order** (BACKLOG): E05, then E23 (the battle lane: the system actor's policy), then E06.
- **Versions** (taken over from the battle lane, unchanged): action encoding v13, observation encoding v21, replay format v29, C ABI v25 (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q2, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

**Ownership**
- The battle lane still writes rules the encodings show: a new observation feature or action entry comes as a `cross-lane` pull request from the battle lane with this lane's version bumps (`docs/08-work-process.md`, "Lanes"). This lane's own changes to `h3-battle` (an engine query the env needs) are `cross-lane` the other way.
- `h3-cli` is the battle lane's except `src/ai/`; its golden replays (`fixtures/`) and `benches/battle/` stay with battle, and change when the encodings or the replay format change: regenerate them deliberately (`fixtures/README.md`) in the same pull request.

**Environment**
- An agent sees the projected observation only (AI-Q1 decision 10); `fork` and `state_hash` read the full state, for tools and training (the C ABI is for trusted local consumers, ADR-0011; never forward the hash to a client).
- `BattleSetup.human` and `.difficulty` change the computer's automatic choices (the turrets, the automatic Ballista, the moat avoidance; ADR-0023 point 7): set them per seat as T1 decides.
- `verify --scope ai` skips the golden replays (their crate `h3-cli` is the battle lane's) and the battle benchmarks: after a change to `h3-sim` run `cargo xtask determinism` and `cargo xtask bench-check --workload random-rollouts-full` too.
- `legal_commands` lists no casts: the cast heads come from `cast_targets` × `second_targets` (the battle lane's trap; `h3_sim::action`).

**Python**
- PyTorch 2.14 (CUDA 12.6 build) is the last prebuilt release for the user's GTX 1070: pin it. The user's system Python is older than 3.11: use a venv.

## Recent sessions

### 2026-10-18 — E05 T1, the design and the ADR draft

- Drafted the batch API (same-step automatic reset, per-env status, head calls per batch, std-thread shards in `h3-sim`, seeds per env stream), seats per scenario side (internal policies a closed enum; E23's function shared with `h3-game`), reward v1 as destroyed plus transferred value (transfers counted once, stakes from the episode, scaled by the starting armies), the scenario TOML unchanged, the encounter distribution's shape and census, ctypes with zero-copy numpy, uv and `uv.lock` for PyTorch 2.14, the benchmarks.
- The user settled (f): every per-player observation already is a human player's information; no check of the original's screens. The user also noted that `verify` cleans up after itself (the coverage gate's clean, `08-work-process.md` "Cloud sandbox" 5).
- Nothing built; stop for the review.

### 2026-10-18 — E05 refined, the lane opened

- With the user (AI-Q1): battle only; the AI lane opens; PyTorch 2.14 over the C ABI (Q5); opponents are self-play and the system actor, heuristic bots only baselines; casts by one call per head; the flat observation first; an encounter distribution weighted by approximate frequency, from the shipped maps until real games give it; the reward as a zero-sum value balance; the human player's view only; exit by a smoke run that learns. Recorded in the epic file, BACKLOG, `07-simulation-and-ai.md`, OPEN-QUESTIONS AI-Q1 and Q5.
- The user's four own maps were removed from `original-data/maps.tar` and `original-data/install/Maps` (`experiment.h3m`, `Untitled.h3m`, `_modified_All for One.h3m`, `_modified_Terrible Rumor.h3m`): 160 shipped maps remain (the adventure lane's inbox has the note on its counts of 164).
