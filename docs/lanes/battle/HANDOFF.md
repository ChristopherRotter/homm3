# Battle lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **E26, the fair system actor** (refined 2026-11-02, BAT-Q63; `docs/epics/E26-fair-system-actor.md`, record `E26-log.md`): the original's computer unchanged on its side's world (`BattleState::determinized`; ADR-0027 point 4 amended). **Increment 1 built and merged** (#647: `h3_battle_ai::choose_fair`, BAT-AI-025, its twin properties, the cost and the duel). **Next:** increment 1's checkpoint with the user (the queue: BAT-STUB-18; increment 2's refinement, which now also takes BAT-STUB-19, the world's guessed defences; the closure). Then **E11** (`docs/epics/E11-historical-rules.md`, Q58): T1 with the user's RoE and AB media. E23 closed 2026-10-24 (`POLICY_VERSION` 19). CLT-Q40 carried and merged (#685).
- **The inbox session of 2026-11-15** (`battle/inbox-2026-11-15`, `cross-lane`; pull request **#949**, open: watch it until it merges): CLT-Q69 (a)'s split (`BattleState::setup`, `begin`, `has_begun`), G1 (each side's `forbids_casting` and `spell_level_limit`, public; the wire's `SideView`, protocol v69), the world property's siege case (BAT-STUB-19), readings (Acid Breath, the Angelic Alliance, the Stronghold's obstacle branch), BAT-Q65 and BAT-Q66 written. ADR-0013 point 9's amendment for G1 is in `shared/battle-g1-adr` (#947, merges first).
- E05, `h3-sim` and `h3-ffi` are the AI lane's (AI-Q1), `h3-heroes` the adventure lane's.
- **Open:** BAT-Q28 (rows 1, 3, 5, 6, 11, 12), BAT-Q32, BAT-Q35, BAT-Q58, **BAT-Q65** (CLT-Q63 (a): the automatic war machines' turns handed over; the user's design choice), **BAT-Q66** (the Angelic Alliance on any of the player's heroes; an experiment row first), BAT-STUB-1, BAT-STUB-18, BAT-STUB-19, the inbox (the walker's hex 62 against 45, E11's Stronghold draw, the consolidated readings), Q43.
- **Versions:** battle state hash v56 (E08 T124a, `cross-lane`: a creature bank's flag and supply; v55 CLT-Q40), battle part hash v8 (E08 T124a: `bank_positions`; v7 E23 T3-0; the encodings, replay format v32 and C ABI are the AI lane's); shared: core ruleset hash v53, pack format v53 (E08 T124a: `bank_positions`; v52 E09 T24), protocol v69 (this session's G1), quirk flags 50 (E08 T140: increment 15's six, 45 `quirk.adventure.flight_reads_hidden` to 50 `door_failure_charged`; the base faithful hash 0x0ee6066a4c802530; 44 E09 T45's). Next ids: BAT-Q67, BAT-QUIRK-31, BAT-STUB-20, BAT-AI-027, BAT-TER-007, BAT-SIE-030, BAT-WAR-009; survey D-24, S-30.
- Working agreement: increments with a user checkpoint (Q41); the user confirms seams and ADRs; batches by the user or an agent; small increments (2026-10-19).

## Standing traps

Moved to `TRAPS.md`, in four groups: **Battle code** (paths, stack ids, two-hex stacks, relations, abilities, hero spells, dead stacks' effects, hidden information, and the computer's walker, scores, odds, casts and predictions), **Tests**, **Packs, replays, benchmarks** (a behaviour change or a quirk flag re-hashes the golden replays and baselines) and **Process**. Area traps live where they are used: experiments `docs/experiments/README.md`, replays `fixtures/README.md`, packs `packs/README.md`, code reading `docs/exe/README.md`.

## Recent sessions

### 2026-11-15 (116th) — the inbox

The user chose the whole inbox but CLT-Q63 (a), which became a question (BAT-Q65). Built: the
start split (CLT-Q69 (a)); G1 across the wire (v69, ADR-0013 point 9 in a `shared/` PR); the
world property's siege case (a hypnotized stack under the other hero's guessed defences: a known
gap, BAT-STUB-19). Read: Acid Breath has nothing a dispel could end; the Angelic Alliance joins
for any of the player's heroes (against VCMI and the wiki: BAT-Q66); the Stronghold's obstacle
branch (the draw queued for E11). Kept: the Catapult's draw (stream only). No battle hash moved.

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
