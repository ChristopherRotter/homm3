# E25 — archive: closed increments

Moved out of `E25-battle-ai-v2.md` verbatim when an increment closes (increment 1 on
2026-10-28, increment 2 on 2026-10-29, at their checkpoints), so that the epic file holds what
is open and the decisions that apply to the whole epic. Each task's full record is in `E25-log.md` (grep its id).

## Increment 1 (closed 2026-10-28)

**Increment 1: the diagnostics and shaping** (refined 2026-10-27, AI-Q10; the AI queue's tasks):

- [x] **T1a The search's terminal by reward v1** (2026-10-28; `E25-log.md` T1a): a world scores
  a decided battle by its batch's reward v1 (`BattleEnv::with_reward`; the agent's choice: the
  world, no new function), its setup keeps the other hero's starting mana (reward v1's mana
  term); C ABI v35.
- [x] **T1b The true-world search** (2026-10-28; `E25-log.md` T1b): `BattleBatch::true_world`,
  `h3_batch_true_world` (C ABI v35 with T1a; trusted tooling, a diagnostic), `h3env.search
  --true-worlds` (the flag spelled so, `--worlds` stays the count).
- [x] **T1c T8c again** (2026-10-28; `E25-log.md` T1c; `t7-1`'s `best.pt`): no gain at N 16 or
  64 against the system actor or on the neutral set, with guessed or true worlds (the guess
  costs at most +0.012 in value, within the intervals): the network's evaluation limits the
  search, not the terminal or the hidden information.
- [x] **T2a Shaping and the training mix** (2026-10-28; `E25-log.md` T2a): `[ppo] shaping` (off
  by default; Φ the batch's potential, `H3BatchBuffers.potential`, C ABI v35), the faithful
  system actor's mixes refused (`run.trainable`: no `system_share`, no neutral kind; the user:
  no neutral battles for now), T2b's configs `e25-control.toml`, `e25-shaping.toml`.
- [x] **T2b The A/B runs** (2026-10-28; `E25-log.md` T2b; `e25-control-1`, `e25-shaping-1`, 20 M
  each): the control matches `t7-1` against the player kinds, the neutral set's value 0.015
  lower; shaping gains about +0.04 against Aggressor, Greedy and the system actor (0.409, −0.066)
  and learns faster, but loops in the tactics phase (its placements 24 to 28 % of its decisions,
  the control's 10 to 12 %; 0.2 % of training battles cut).
- [x] **T3 Increment 1's checkpoint** (2026-10-28; `E25-log.md` T3): the user chose increment 2,
  shaping bounded, then scale; `e25-shaping-1` is the lineage's current best.

## Increment 2 (closed 2026-10-29)

**Increment 2: shaping, bounded, then scale** (refined 2026-10-28; the AI queue's tasks):

- [x] **T4a The tactics bound** (2026-10-28; `E25-log.md` T4a): `[network] tactics_rounds`
  (off by default; T4b's `configs/e25-bound.toml` 1): past that many rounds of its own phase End
  tactics is the agent's only entry, wherever the network plays; the binding counts the phase's
  turns from the observations (Python only, no ABI change).
- [x] **T4b The A/B** (2026-10-29; `E25-log.md` T4b): `e25-bound-1` at or above
  `e25-shaping-1` against every engine seat (inside the intervals), the loop gone (the phase 2.5 %
  of its decisions, none cut), 73 minutes against 90; the user: it is the current best.
- [x] **T5 The scale run** (2026-10-29; `E25-log.md` T5): `e25-scale-1`
  (`configs/e25-scale.toml`, three tactics rounds), stopped by the user at 41.3 M of 60 M, the
  curve flat from 20 M; best #3 (30 M) at `e25-bound-1`'s level against every engine seat
  (0.418 against the system actor; the differences inside the intervals), the phase 4 % of its
  decisions, no loop. Scale gains nothing at these settings.
- [x] **T6 Increment 2's checkpoint** (2026-10-29; `E25-log.md` T6, AI-Q11): the current best
  `e25-scale-1` #3; the head-to-heads skipped; increment 3 is the network's architecture, a
  distillation bench first (the user).
