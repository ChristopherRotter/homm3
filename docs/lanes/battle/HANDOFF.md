# Battle lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E23 closed 2026-10-24** (`docs/epics/E23-original-battle-ai.md`; `POLICY_VERSION` 19). **The inbox's leftovers** done; CLT-Q4/Q6's seam #465 merged, its carry's pull request (`battle/clt-q4-q6-carry`, `cross-lane`) open: watch it. **Next (the user, 2026-10-24):** **E11** (`docs/epics/E11-historical-rules.md`, Q58): T1 with the user's RoE and AB media.
- E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28 (rows 1, 3, 5, 6, 11, 12), BAT-Q32, BAT-Q35, BAT-Q58, BAT-STUB-1, the inbox (by kind since 2026-10-19), Q43.
- **Versions:** battle state hash v54 (CLT-Q4/Q6), battle part hash v7 (E23 T3-0; the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v48, pack format v47 (E08 T45), protocol v35, quirk flags 37. Next ids: BAT-Q63, BAT-QUIRK-31, BAT-STUB-18, BAT-AI-025, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (2026-10-19).

## Standing traps

Moved to `TRAPS.md`, in four groups: **Battle code** (paths, stack ids, two-hex stacks, relations, abilities, hero spells, dead stacks' effects, hidden information, and the computer's walker, scores, odds, casts and predictions), **Tests**, **Packs, replays, benchmarks** (a behaviour change or a quirk flag re-hashes the golden replays and baselines) and **Process**. Area traps live where they are used: experiments `docs/experiments/README.md`, replays `fixtures/README.md`, packs `packs/README.md`, code reading `docs/exe/README.md`.

## Recent sessions

### 2026-10-24 (112th) — CLT-Q4/Q6, seam to wire

The viewers' facts (`BattleSetup.background`, `.colours`, `BattleHero.sex`; no rule reads
them): hashed state (v54), setup errors out of range or one colour for both sides, observed, on
the wire (v35) and in the replay file (v31); replays rehashed, commands unchanged. Inbox notes:
adventure (`h3-game` sets them), client, server, AI.

### 2026-10-24 (111th) — the inbox's leftovers

#458: the heroless side's war machines, the turret-shot preview, batch 2d's replays, the
`verified` clauses. Batch 2g and 2h played by an agent (`ai.py`'s rolls and surrender price):
the surrender price, the hypnotized stack's morale, the wait phase, a freeze aligned; S-23, S-24
and BAT-Q62 fix-optional quirks (BAT-QUIRK-28 to 30, flags 34 to 36, the user); three golden
replays re-recorded. CLT-Q4/Q6 left to the 112th.

### 2026-10-24 (110th) — E23 closed

The user: close E23 now, then the inbox, then E11. Every exit criterion met (the log); CI cannot start (billing), so criterion 6 rests on #441's full local verify and the user's Windows run.
