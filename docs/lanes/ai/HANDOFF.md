# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`,
  closed increments in `E06-archive.md`): increments 1 to 6 done. Best so far `t7-1` (20 M, a
  third neutral): 0.828, 0.551, 0.528 against Random, Aggressor, Greedy, the system actor 0.374;
  the neutral set's value +0.245 below the target +0.285. Increment 7, T8 the search (design
  §14): T8a, T6f (the turn order, observation v22) and T8b (the plumbing, #493) done; T6g (the
  user's run `order-1`) done 2026-10-27: negative, `t7-1`'s ladder, the trunk drops the order.
  **In progress:** T8c in the sandbox with `order-1`'s best (the user's upload; `t7-1`'s is not
  here), N 16 on the report set against the system actor and Greedy and on the neutral set
  (`python/runs/t8c/`, ignored). At T8b's checkpoint the user reviews AI-STUB-1.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v34 (T6f and T8b: the turn
  order, the worlds) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1,
  AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-26 to 27 — T6f, the turn order as an input; T8b, the search's plumbing

- The user (before leaving): the turn order first, as observation v22 with the engine's order
  (`h3_battle::turn_order`, `cross-lane`), then T8b, one pull request with one C ABI bump (v34),
  and a 20 M A/B against `t7-1` prepared (`configs/e06-order.toml`). Built both (E06-log T6f,
  T8b); `h3env.search` also has T8c's `--report`. One gap against ADR-0025 point 12a
  registered (AI-STUB-1); the battle lane's inbox names the two additions to `h3-battle`.

### 2026-10-26 (evening) — T6e, the A/B run `t6d-1`; increment 6 closed; T8a, the search's design

- `t6d-1` (the user's run, the kill preview in the network) ends at `t7-1`'s ladder; `inspect`
  shows better targeting but fewer strikes and long wandering (E06-log T6e). The user closed
  AI-Q7 (the preview off by default) and chose T8 next, determinized from the start, the tree in
  Python first: T8a, the design (E06 design §14) and ADR-0025 point 12's amendment as a draft.
- #487 merged; the user accepted the ADR draft and both `h3-battle` functions `cross-lane`. The
  user's question on the turn order: a probe shows the trunk does not know which stack acts
  next (E06-log "The turn-order probe"); a candidate lever in QUEUE.

### 2026-10-26 (later) — T6c's run recorded; T6d, the kill preview in the network

- `t6b-1` (the user's run) ends at `t7-1`'s ladder; its behaviour analysis shows the same play
  against the system actor (E06-log T6c). The user chose AI-Q7 (b) part 2: built `[network]
  preview` (E06 design §13), `configs/e06-t6d.toml`, on a branch restarted from `master` after
  #478 merged.
