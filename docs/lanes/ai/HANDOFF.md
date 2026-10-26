# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`,
  closed increments in `E06-archive.md`): increments 1 to 6 done (#306 to #481 and this
  branch), T6a (the behaviour analysis) 2026-10-23. Best so far `t7-1` (20 M, a third neutral):
  0.828, 0.551, 0.528 against Random, Aggressor, Greedy, the system actor 0.374; the neutral
  set's value +0.245 below the target +0.285. Increment 6's two levers were negative: the system
  actor as an opponent (`t6b-1`) and the kill preview in the network (`t6d-1`; AI-Q7 closed, off
  by default). Increment 7, T8 the search (the user, 2026-10-26: the plateau taken as shown;
  design §14): T8a done (#487); the user accepted ADR-0025's amendment draft (`adr-drafts/`, into
  ADR-0025 with the next `shared/` change). **Next:** T8b, the plumbing (C ABI v34, `h3-battle`'s
  `determinized` and `reseed` `cross-lane`, the user's consent; QUEUE).
- **Versions:** battle action encoding v13, observation encoding v21, replay format v31 (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v33 (E06 T5: the kill preview) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-26 (evening) — T6e, the A/B run `t6d-1`; increment 6 closed; T8a, the search's design

- `t6d-1` (the user's run, the kill preview in the network) ends at `t7-1`'s ladder; `inspect`
  shows better targeting but fewer strikes and long wandering (E06-log T6e). The user closed
  AI-Q7 (the preview off by default) and chose T8 next, determinized from the start, the tree in
  Python first: T8a, the design (E06 design §14) and ADR-0025 point 12's amendment as a draft.

### 2026-10-26 (later) — T6c's run recorded; T6d, the kill preview in the network

- `t6b-1` (the user's run) ends at `t7-1`'s ladder; its behaviour analysis shows the same play
  against the system actor (E06-log T6c). The user chose AI-Q7 (b) part 2: built `[network]
  preview` (E06 design §13), `configs/e06-t6d.toml`, on a branch restarted from `master` after
  #478 merged.

### 2026-10-26 — E06 increment 6 refined; T6b, the system actor as an opponent

- The user chose increment 6's lever (the system actor in the player kinds) and answered the
  let-off: every side stays human. Built `[league] system_share` (the batch's entries split,
  no ABI change), `configs/e06-t6b.toml`; E06's increments 1 to 5 moved to `E06-archive.md`.
  The system actor's hero decisions make a batch call at a quarter 1.7 times as long on the
  sandbox (E06-log T6b). The bots' tables were current (reproduced byte for byte).
