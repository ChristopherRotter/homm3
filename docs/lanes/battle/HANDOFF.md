# Battle lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E26, the fair system actor** (refined 2026-11-02, BAT-Q63; `docs/epics/E26-fair-system-actor.md`, record `E26-log.md`): the original's computer unchanged on its side's world (`BattleState::determinized`; ADR-0027 point 4 amended). T1 (the design) and T2 (`h3_battle_ai::choose_fair`, BAT-AI-025) done on `claude/vibrant-dirac-udew18` (cross-lane by the user's choice: ADR, `BACKLOG.md`, `lanes.toml`; no pull request yet). **Next:** T3 (the properties), T4 (cost and strength), T5 (the hand-over, the checkpoint), in the queue. Then **E11** (`docs/epics/E11-historical-rules.md`, Q58): T1 with the user's RoE and AB media. E23 closed 2026-10-24 (`POLICY_VERSION` 19).
- E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28 (rows 1, 3, 5, 6, 11, 12), BAT-Q32, BAT-Q35, BAT-Q58, BAT-STUB-1, the inbox (by kind since 2026-10-19; what is left: BAT-MAG-010's Acid Breath reading, which needs the executable, and the consolidated readings), Q43.
- **Versions:** battle state hash v54 (CLT-Q4/Q6), battle part hash v7 (E23 T3-0; the encodings, replay format and C ABI are the AI lane's); shared: core ruleset hash v51, pack format v50 (E08 T81b), protocol v35, quirk flags 39. Next ids: BAT-Q64, BAT-QUIRK-31, BAT-STUB-18, BAT-AI-026, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (2026-10-19).

## Standing traps

Moved to `TRAPS.md`, in four groups: **Battle code** (paths, stack ids, two-hex stacks, relations, abilities, hero spells, dead stacks' effects, hidden information, and the computer's walker, scores, odds, casts and predictions), **Tests**, **Packs, replays, benchmarks** (a behaviour change or a quirk flag re-hashes the golden replays and baselines) and **Process**. Area traps live where they are used: experiments `docs/experiments/README.md`, replays `fixtures/README.md`, packs `packs/README.md`, code reading `docs/exe/README.md`.

## Recent sessions

### 2026-11-02 (114th) — E26 refined, T1 and T2

The user chose the port on its side's world (BAT-Q63) and one cross-lane branch. E26's epic,
log, the queue's T3 to T5. `choose_fair` built: its twins (the other hero's book, masteries and
damage skills) never change its command where they change `choose`'s 12 times in 70; with no
other hero it is `choose`. No version moved. **In progress:** nothing half-done; T3 next.

### 2026-11-02 (113th) — the inbox

Settled 15 entries (now in `QUEUE-ARCHIVE.md`). The client lane's `summon-turn` round 2 is no
engine bug: the scenario loads `base` alone (elementals neutral). `TextTable::parse` makes
quote runs in quoted fields one, as the executable does. Batch E8b's sea battles are now sources
of BAT-GEO-004 and BAT-TER-002. Two traps went to `TRAPS.md`. Doc fixes: `packs/README.md`'s
profiles, §7.23's order. No version moved.

### 2026-10-24 (112th) — CLT-Q4/Q6, seam to wire

The viewers' facts (`BattleSetup.background`, `.colours`, `BattleHero.sex`; no rule reads
them): hashed state (v54), setup errors out of range or one colour for both sides, observed, on
the wire (v35) and in the replay file (v31); replays rehashed, commands unchanged. Inbox notes:
adventure (`h3-game` sets them), client, server, AI.
