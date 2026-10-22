# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (T1 to T9 in six increments; the
record in `E05-log.md`, the environment as built in `docs/07-simulation-and-ai.md` §1.1).

**E06** (`docs/epics/E06-battle-ai.md`) refined 2026-10-20 (AI-Q8): it starts beside E23, the
neutral share waits for E23's side without a hero. Increment 1 done 2026-10-20: T1 and T2a
(ADR-0026, design `E06-design.md`), T2b1 (the full checkpoint and `--resume`; its pending gate and
`--init-from` moved to T3b), T2c (C ABI v31). Increment 2 done 2026-10-20: T3a (the held-out
sets v1, the bots' table), T3b (the SPRT gate, promotions, ladders, `--init-from`; the user's
hour `t3b-1`: evaluation 21.4 %, stalling suspected, AI-Q9). Increment 3 done 2026-10-22: T4a
(the league), T4b (the user's runs `t4-1`, `t4-2`: γ 1.0 ends the stalling, AI-Q9; the best
plays 0.839, 0.578, 0.559 against Random, Aggressor, Greedy; `E06-log.md`).

**Increment 4** refined 2026-10-22 (the user, `E06-log.md`): the entity network first, T5a then
T5b; no spare MLP run. T5a done 2026-10-22 (`E06-log.md`). Still open for later increments: AI-Q7 (b), throughput (`t4-2`'s dip to
about 4 000 samples/s from 20 M to 40 M), the mana price, T7 (E23's neutral side is complete).

### T5b — The A/B run

- **Scope:** the user runs `configs/e06-t5.toml` (the entity network, `t4-2`'s config and seed,
  60 M samples) on their machine; the record committed, the comparison with `t4-2` in
  `E06-log.md`.
- **Acceptance:** the record passes `check_record`; the ladders compared per opponent.
- **Depends on:** T5a's pull request merged; the user's machine. **Status:** waiting (the
  merge, then the user). First `python -m h3env.train.network --config configs/e06-t5.toml
  --time --device cuda` (the GPU's cost; T5a measured the entity network at about five times
  the MLP's time on the sandbox's CPU), then the run (the config's comment).

## Inbox

Sorted by destination at E05's audit (T9a, 2026-10-19); the origin stays in brackets. Dropped as
done: BAT-Q33's point for E05 (AI-Q3, ADR-0025 point 6), the `h3-game` coverage gap of a pending
Pass offer (fixed by the adventure lane's fixed tests, #266).

- 2026-10-22 (the battle lane, E23 T5b1, `battle/e23-t5b1`; for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 13: a computer side's hero now also casts Resurrection, Animate Dead, Sacrifice and Earthquake (the original's values, BAT-AI-019; not yet Teleport, nor retreats). A side without a hero is unchanged; a hero's decision stays within the 5 ms budget. Name it in a run record's engine identity; the function's signature is unchanged.
- 2026-10-22 (the battle lane, E23 T5a, `battle/e23-t5a`; for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 12: a computer side's hero now casts spells (the original's choice, BAT-AI-017, BAT-AI-018; not yet the raises, Sacrifice, Earthquake, Teleport, nor retreats). A side without a hero is unchanged. A decision with a hero's cast takes up to 5 ms (the user's budget; mean 0.43 ms with every spell in the book). Name it in a run record's engine identity; the function's signature is unchanged.
- 2026-10-22 (the battle lane, E23 slip 25, `battle/e23-slip25`; for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 11: in `strict-fixed` (BAT-QUIRK-25 off) the computer's Ice, Magma, Storm and Energy Elementals cast their protection when it beats their attack or shot; `faithful` choices are unchanged. Name it in a run record's engine identity; the function's signature is unchanged.
- 2026-10-22 (the adventure lane, E08 T36b, `adventure/e08-t36b`, `cross-lane` with the user's
  consent; for your information, nothing to do): `h3_map::writer::Map` gained `banned_skills`;
  the census map in `crates/h3-cli/src/ai/tests.rs` says none (one line).
- 2026-10-22 (the battle lane, E23 T4d, `battle/e23-t4d`; for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 10: the Master Genie values Cure and Anti-Magic as the original does (a removal's copy, slip 24 under BAT-QUIRK-25; the copy's turns and Hypnotize's map for a shooter in both profiles). Name it in a run record's engine identity; the function's signature is unchanged.
- 2026-10-22 (the adventure lane, E08 T35b, `shared/e08-t35b`, `cross-lane` with the user's consent,
  E08 decision 79; for your information, nothing to do): `python/records/bots-report-v1.json`
  regenerated (`h3 ai eval --set report-v1 --json`) for the core ruleset hash v47 (six adventure
  hero-effect kinds and three adventure flags; nothing the battle reads). Besides `engine.core`,
  two pairs' rewards and two values moved in the fourth decimal (wins unchanged): the battle lane's
  creature-cast change below, not T35b (a regeneration on `master` 25c01bd gives the same table
  but for `engine.core`).
- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for you: the neutral side's policy is complete**, E23 decision 1 and exit criterion 5): `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` now plays a side without a hero as the original's computer does: shots, melee attacks, moves, waits and defends, and the creature casts (the raise, the Faerie Dragon, the Ogre Mage, the Master Genie). It reads the whole state, draws only from the stream you give it, and validates its choice (a refused one becomes Defend; none is known). `POLICY_VERSION` is 9 (name it in a run record's engine identity). `Policy::System` and the neutral seat can call it now; A12's weights for the neutral share are yours (`E05-log.md` §4.4). A side with a hero still casts nothing and never retreats (BAT-STUB-5, E23 T5 and T6). Cost: within ADR-0027 point 11's budget (≤ 1 ms for the slowest decision; `benches/battle/computer-decisions.toml`).
- 2026-10-21 (the battle lane, `battle/creature-cast-mass-target`, `cross-lane` with the user's consent; for your information, nothing to do): `h3_battle::creature_cast_targets` now lists `CastTarget::None` first when a creature's cast takes no target: on Magic Plains the Ogre Mage casts Bloodlust at Expert, its mass version, which `apply` accepts only as `CreatureCast { target: CastTarget::None }` (BAT-MAG-048, BAT-TER-005). `h3-sim`'s action mask (`action.rs`, `rollout.rs`) reads the listing, so it now offers that cast (target index 0) without a change of yours; such a cast lists no stack. The computer's policy cast it already (BAT-AI-014).
- 2026-10-21 (the adventure lane, E08 T32a, `adventure/e08-t32a`, `cross-lane` with the user's
  consent, E08 decision 67; for your information, nothing to do): `h3_map::writer::Map` gained
  `level_limit` (the scenario's, 0 none) and `MapObject::Plain` (a Cartographer), so the test map
  in `crates/h3-cli/src/ai/tests.rs` sets `level_limit: 0`; nothing it counts moves.
- 2026-10-21 (the adventure lane, E08 T29, `adventure/e08-t29`, `cross-lane` with the user's
  consent, E08 decision 63; for your information, nothing to do): `h3_game::GameSetup` gained
  `level_limit` and `banned_skills` (the scenario's), so `crates/h3-cli/src/ai/census.rs` sets
  both to none (`0`, `Vec::new()`) where it builds its one-player setup; nothing it counts moves.
  The adventure part's hash moved (v9: the terrains' costs by Pathfinding level), so
  `python/records/bots-report-v1.json` was regenerated in that pull request (`h3 ai eval --set
  report-v1`): only `engine.adventure` moved (`5db17335c993551f`), every rate as before. The game
  state hash (v10) and the map import (v8) moved too; the core and battle hashes did not.
- 2026-10-21 (the battle lane, E23 T2b, `shared/e23-t2b`, `cross-lane` with the user's consent;
  for your information, nothing to do): BAT-QUIRK-25 (`quirk.battle.original_ai_slips`, registry
  position 29) is on in the faithful profiles, so the faithful core ruleset hashes moved
  (`base:faithful` `0xe6839bfb42ebe118`, `synthetic:faithful` `0x203012933e60e190`; strict-fixed
  unchanged) and `python/records/bots-report-v1.json` was regenerated (`h3 ai eval --set
  report-v1`): only `engine.core` moved, every rate is the same (the flag changes only the
  original's battle AI's estimates, which no bot plays). `h3_battle_ai::POLICY_VERSION` is 2 (the
  Magogs' and Liches' area value, the Cyclops's wall shot); the function is yours at E23 T4.
- 2026-10-20 (the adventure lane, E08 T27, `adventure/e08-t27`, `cross-lane` with the user's
  consent, E08 decision 62; for your information, nothing to do): `h3_heroes::HeroDetails` gained
  `offered` (the levels of the last Wisdom and school offers) and its validation refuses a level
  above the experience's level (HER-EXP-002), so the encounter sampler's heroes now carry their
  level's experience (`crates/h3-sim/src/encounter/hero.rs`: `experience_for`; the module comment
  names `h3_heroes::draw_level_up`, which the sampler's approximation does not use). The battle
  reads no experience: every encounter plays as before and the bots' engine identity (the
  ruleset hashes) did not move. The adventure env's replays moved with game state hash v9.
- 2026-10-20 (the adventure lane, E08 T26b, `shared/e08-t26b`, `cross-lane` with the user's
  consent; for your information, nothing to do): the core ruleset hash moved (v46: the hero
  parameters' level data, the classes' offer intervals, two quirk flags), so
  `python/records/bots-report-v1.json` was regenerated in that pull request (`h3 ai eval --set
  report-v1`, `python/records/README.md`): only `engine.core` moved (`6384653ed1e4c038`), every
  rate is the same. A run record made before it names the old core hash.

**For E06** (training at scale, evaluation, search; the epic's T2 to T5 take them up):
- **Evaluation:** greedy play (the most likely heads) repeats itself against Random until the
  step limit (up to 17 % of the battles in T8b's sandbox run); the sampled policy never. E06's
  evaluation and any search agent need a rule for it (sampled play, or a tie-break or repetition
  rule); the 2 000-step limit is only a guard (T8b). The VCMI oracle harness was never built:
  revisit with E06's evaluation needs (E04 refinement, 2026-10-01).
- **Baselines and AI-Q7:** Greedy value beats Aggressor only 0.519 on the held-out episodes: the
  preview of its own kills buys little without spells or a plan; a stronger baseline (casts, a
  one-ply look at the opponent's reply) is E06's, and AI-Q7 (whether the agents get the preview)
  moved there (T7c; the user, 2026-10-19).
- **Encodings:** `h3_sim::action::encode` answers without a viewer (the same root as F-sec-2's
  leak, which the sim's `step` closes); it leaks nothing today (E04b T11b, the battle lane).
- **Performance**, each worth a look when a profile of training shows it: the flat observation
  costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is
  zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node
  hashing (E04b T11a); an encounter sample costs 50 µs (two heroes derived, each artifact through
  `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 % of an episode (T5b2); the
  trainer copies each call's observations (9 MB at 256 envs) into the rollout, which the batch
  could write in place (`H3BatchBuffers.obs` per call) (T8b; E06 T5, no ABI change: design §10). The battle lane's `full` workload
  is lopsided (8 attacker victories in 2 399 agent seeds; E04b T11a): the encounter workload
  (`batch-random-encounters-v1`) is the realistic one. On the user's Ryzen 9 3900X the step
  scales to 9.9 × at 1 024 envs on 24 threads but only 3.8 × at 64 envs (`E05-log.md`, T9c's
  table): E06's batch size and thread count from there.

- **Encounters v2** (with the adventure's logged encounters, decision 7; E05 T9b): A7's "+0.5 a
  week" in weeks 9 to 16 is rounded down in `encounters-v1.toml` (odd weeks half a level low);
  untested: A4's cap and redraws in weeks 1 to 6, the forts of weeks 2 to 6, A7's war-machine
  chance and class split, A6's upgraded share, the guards' anti-magic share.
- **The C ABI** (E05 T9b): `h3env.tools.Env` keeps its handle after `H3_ERR_PANIC` (tests only;
  `h3_batch_create`'s unchecked N closed by E06 T2c).

**For E23** (the system actor; the neutral kinds' share): A3 grows a wandering stack by a tenth
of its count, VCMI 1.7.5's `CGCreature::newTurn` a tenth of its power (a thousand per creature:
5 creatures are 9 after seven weeks, the count's tenth leaves them 5); VCMI's growth makes
late-week stacks pass A4's cap and be drawn again, so decide both together (E05 T9b).

**For E13** (the adventure env's ABI): reading both of the adventure env's mask heads doubles
its step; the current state's queries belong in the env (the adventure lane, E08 T9b).
