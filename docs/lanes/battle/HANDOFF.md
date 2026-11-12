# Battle lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E26, the fair system actor** (refined 2026-11-02, BAT-Q63; `docs/epics/E26-fair-system-actor.md`, record `E26-log.md`): the original's computer unchanged on its side's world (`BattleState::determinized`; ADR-0027 point 4 amended). **Increment 1 built** (T1 to T5): `h3_battle_ai::choose_fair` (BAT-AI-025), its twin properties (no leak), `h3 bench-computer --sight fair` and `--duel 1` (+0.8 % Ir per decision, 0.500 against the all-seeing one), the other lanes' inbox notes. Pull request #647 (`claude/vibrant-dirac-udew18`, label `cross-lane`) open: watch it until it merges. **Next:** increment 1's checkpoint with the user (the queue: BAT-STUB-18, increment 2's refinement, the closure). Then **E11** (`docs/epics/E11-historical-rules.md`, Q58): T1 with the user's RoE and AB media. E23 closed 2026-10-24 (`POLICY_VERSION` 19).
- **CLT-Q40 carried** (BAT-Q64, 2026-11-03; branch `claude/keen-cori-oaahgi`, `cross-lane`): `EffectSource` on the morale and luck effects, `Siege.bonuses`, derived in `h3-heroes`, kept and public in the observation with each stack's unclamped values, on the wire. Pull request #685 (label `cross-lane`) open: watch it until it merges (merge `master` into it after #647).
- E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28 (rows 1, 3, 5, 6, 11, 12), BAT-Q32, BAT-Q35, BAT-Q58, BAT-STUB-1, BAT-STUB-18 (E26's checkpoint), the inbox (by kind since 2026-10-19; what is left: BAT-MAG-010's Acid Breath reading, which needs the executable, and the consolidated readings), Q43.
- **Versions:** battle state hash v55 (CLT-Q40), battle part hash v7 (E23 T3-0; the encodings, replay format v32 and C ABI are the AI lane's); shared: core ruleset hash v53, pack format v52 (E09 T24), protocol v37, quirk flags 41. Next ids: BAT-Q65, BAT-QUIRK-31, BAT-STUB-19, BAT-AI-026, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (2026-10-19).

## Standing traps

Moved to `TRAPS.md`, in four groups: **Battle code** (paths, stack ids, two-hex stacks, relations, abilities, hero spells, dead stacks' effects, hidden information, and the computer's walker, scores, odds, casts and predictions), **Tests**, **Packs, replays, benchmarks** (a behaviour change or a quirk flag re-hashes the golden replays and baselines) and **Process**. Area traps live where they are used: experiments `docs/experiments/README.md`, replays `fixtures/README.md`, packs `packs/README.md`, code reading `docs/exe/README.md`.

## Recent sessions

### 2026-11-03 (115th) — CLT-Q40 carried, seam to wire

The user chose the source on the effects, `h3-heroes`' derivation with it and every source kind
(BAT-Q64). Built across the lanes: contracts, battle (state v55, public observation, unclamped
values computed in `materialize`: on demand they cost the AI batch +4 %), scenario and replay files
(v32), wire (v37), the clients' call sites; replays rehashed, benches within 0.2 % of `master`.
ADR-0013 points 3 and 9, ADR-0023 point 2 amended. Nothing half-done.

### 2026-11-02 (114th) — E26 refined and increment 1 built

The user chose the port on its side's world (BAT-Q63) and one cross-lane branch. T2
`choose_fair`; T3 (a worker) the twin properties on both rulesets: no leak, one fallback the
all-seeing computer shares (BAT-STUB-18: a hypnotized shooter aims at itself); T4 the cost and
the duel; T5 the inbox notes. No version moved. Pull request opened; nothing half-done.

### 2026-11-02 (113th) — the inbox

Settled 15 entries (now in `QUEUE-ARCHIVE.md`). The client lane's `summon-turn` round 2 is no
engine bug: the scenario loads `base` alone (elementals neutral). `TextTable::parse` makes
quote runs in quoted fields one, as the executable does. Batch E8b's sea battles are now sources
of BAT-GEO-004 and BAT-TER-002. Two traps went to `TRAPS.md`. Doc fixes: `packs/README.md`'s
profiles, §7.23's order. No version moved.
