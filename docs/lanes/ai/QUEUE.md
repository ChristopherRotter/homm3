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

**Increment 4** done 2026-10-23 (`E06-log.md`): T5a (the entity network), T5b (`t5-1`
stalled: both sides hid), T5c (the truncation penalty, the user's amendment of decision 10), T5d
(`t5-2`, `t5-3`: the entity network parked; the MLP with the penalty reached 0.832, 0.557,
0.533 in 20 M samples and 70 minutes, near `t4-2`'s 60 M).

**Increment 5** done 2026-10-23 (`E06-log.md`): T7a (#420), T7b (the system actor's seat, C
ABI v32; #421), T7c (the trainer's neutral share, the neutral set v1, the system actor on the
ladder; #426), T7d (`t7-1` at a third neutral: the report set as `t5-3`'s, 0.828, 0.551, 0.528;
the system actor 0.374; the neutral set 0.972, value +0.245 against the target +0.285). Beside
it T5's rest: the mana price's preparation (#429) and AI-Q7 (b) part 1 (C ABI v33, #432).

**T6a** (the behaviour analysis) done 2026-10-23 (`E06-log.md`, T6a): `python -m
h3env.train.inspect` (rerun it at every checkpoint); on `t7-1`'s best the hero casts as the
system actor's does, the agent plays alike against the system actor and Greedy, waits (3 %)
and defends (1 %) less than the system actor (7 %, 3 %), loses mostly defending; targeting
leaves a quarter of the previewed kills.

**Next: increment 6's lever** (the user's choice at T6a's checkpoint):

- **Increment 6's choices** (put to the user at T7d; T6a's evidence beside each):
  - Code: the system actor as a training opponent in the hero kinds (a change of decision 9:
    the agent never met it there and does not adapt to it; its ladder row then measures a
    training opponent; **the agent's recommendation**, A/B against `t7-1`'s settings); **AI-Q7
    (b) part 2** (the network reads `batch.preview` beside the entry mask, an opt-in
    `[network]` key, the rollout keeps it; an A/B config against `t7-1`'s settings; the
    targeting slack says yes, the turret's preview first, the battle lane's inbox);
    potential-based shaping. Not the casts' exploration or the mana price (the hero casts as
    much as the system actor's).
  - Runs on the user's machine: 60 M at T7's mix (`configs/e06-t7.toml --set
    samples=60_000_000`, about 4 hours: whether PPO plateaus, T8's trigger); the mana run
    `t5-4` (`configs/e06-mana.toml`, about 75 minutes).
- Later (design §10): T5's rest (throughput, the observations in place), T6, T8, T9.

## Inbox

Sorted by destination at E05's audit (T9a, 2026-10-19); the origin stays in brackets. Dropped as
done: BAT-Q33's point for E05 (AI-Q3, ADR-0025 point 6), the `h3-game` coverage gap of a pending
Pass offer (fixed by the adventure lane's fixed tests, #266).

- 2026-10-24 (the adventure lane, E08 T49a, `adventure/e08-t49a`, `cross-lane` with the user's
  consent; for your information, done in your paths): **`HeroDetails.worn` is by position**
  (`Vec<Option<ArtifactInstance>>`, one entry per position of `h3_heroes::positions`; ADR-0013
  point 1): `h3-sim`'s encounter hero starts from `wearing(core, &[])` (every position empty)
  before its `AddArtifact`s, which wear each at its first position as before. **Game state hash
  v12** and **map import v9**: the fixture maps `e08-duel-36` and `e08-xl-144` were regenerated
  (a hero's Spell Book on the map's book entry) and the ten adventure replays re-recorded, their
  commands unchanged; the env's tests pass on them. `HeroChange::PlaceArtifact` is new (a move's
  target, T49b); a battle never tells one.
- 2026-10-23 (the battle lane, E23 T7, `battle/e23-t7-rest` on `shared/e23-t7-human` #436, both
  `cross-lane` with the user's consent; **for you**): **`POLICY_VERSION` 19** (the computer's
  neighbour table without the side columns, batch 4b H1; slip 33 under BAT-QUIRK-25) and two
  rules: BAT-WAR-003's automatic tent heals the most hurt stack (no draw any more), and the new
  fix-optional **BAT-QUIRK-27** (`quirk.battle.computer_morale_let_off`, registry position 33, on
  in `faithful`): a frozen stack of a side whose `BattleSetup.human` is false escapes one freeze in
  four (BAT-MOR-004; ADR-0023 point 7 widened). Your batch plays both sides as human players'
  (AI-Q3), so a `System` neutral side in training gets no let-off: set that side's `human` false if
  the original's handicap is wanted (a question for E06). Both bot reports regenerated in the two
  pull requests (#436: only `engine.core`; the battle PR: version 19; the system actor beats Greedy
  0.655, Aggressor 0.681, attacks best on the neutral set, value +0.285). **E23's exit criterion 4
  wants a gated batch workload with `System` seats** (`benches/ai/`; `h3 ai bench` has no `System`
  seat yet): yours to add when it suits; the computer's own cost is `h3 bench-computer` (now with
  `--heroes 1`: the slowest decision 3.0 to 4.1 ms, the 5 ms budget). **The workload: done**
  (E06 T6a's pull request, `benches/ai/batch-system-neutral-v1.toml`); the let-off for a
  `System` side in training is open (put to the user at T6a's checkpoint).
- 2026-10-23 (this lane, E06 T5; for the next `shared/` change): `docs/07-simulation-and-ai.md`
  (shared) should name the batch's optional kill preview (C ABI v33: the acting side's previewed
  kills of each legal attack and shot entry, the engine's BAT-DMG-011 preview, what the original
  shows a player) under "The batch" and "Information".

- 2026-10-23 (the battle lane, E23 T6's last part, `battle/e23-t6`, `cross-lane` with the user's
  consent; **for you**): the replay format is **v30** (`h3_sim::REPLAY_FORMAT_VERSION`: the setup
  names each side's `RetreatFacts` and `attacker_ground`, ADR-0027 point 14; the CLI's file writes
  `[setup.attacker_retreat]`, `[setup.defender_retreat]` and `attacker_ground`, each omitted at
  its default); please take the number into your HANDOFF's versions (it says v29; done, T7c). The battle keeps
  both as hashed state (still v53, not observed). `h3_battle_ai::POLICY_VERSION` is 18: a hero's
  side may now retreat (BAT-AI-024, the original's retreat test; never on Easy, on Normal half of
  its draws; a hero's cast in retreat takes its best damage spell), but only with the facts set: the
  default facts (`RetreatFacts::default()`, no Tavern) never retreat, so the batch's `System` seats
  and the encounters play on as before unless they fill them; the Normal draw moves the policy's
  stream at every hero cast of a side with a hero.
- 2026-10-23 (the battle lane, E23 T6, `battle/e23-t6`, `cross-lane` with the user's consent; for
  your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 17 (16: a commanded Catapult
  aims as the original's computer with Ballistics, BAT-AI-021; 17: a commanded First Aid Tent heals
  the stack that lost the most, BAT-AI-022, and a computer side plays its tactics phase, BAT-AI-023:
  its shooters and walkers place themselves and it ends the phase after a round without a
  placement or after three, where it ended the phase at once). `h3_battle::BATTLE_STATE_HASH_VERSION`
  is 53 (`TacticsRounds`, the phase's rounds the computer counts: public, not observed, no rule
  reads it: no observation, encoding or replay-format change). Name both in a run record's engine
  identity.
- 2026-10-23 (the adventure lane, E08 T45, `shared/e08-t45`, `cross-lane` with the user's
  consent; **for you**): **the adventure env is yours** (the user at ADV-Q24 and ADV-Q25, E08
  decisions 99 and 103; ADR-0017 point 8): `crates/h3-sim/src/adventure/` (`AdventureEnv`, the
  action encoding v4 and its masks, the rollout, `tests.rs` and `tests/`) moved to you in
  `lanes.toml`, with `ACTION_ENCODING_VERSION` (`docs/08-work-process.md`'s versions table); the
  adventure lane keeps `replay.rs` (the replay model of game commands, `REPLAY_FORMAT_VERSION`),
  `h3-cli`'s `src/adventure/`, the golden replays and `benches/adventure/`. The adventure lane's
  increment 6 (army and hero management) adds no entry to the encoding: its management commands
  reach agents through `h3_game::legal_commands` only. **One check is due by the adventure lane's
  T52** (round 5; `docs/epics/E08-design.md` §12): once T51 opens a meeting or an army dialog as a
  pending decision (each blocks its owner's `EndTurn`, as an offer does), the masks must allow
  `Decline` while one is open, so the rollout and the replays' recorder close it and play on.
  `legal_mask` builds the answers from `legal_commands`, which will list `Decline` there, so this
  may need only a test; the adventure lane tells this inbox when T51 lands. The core ruleset hash moved to
  v48 in the same pull request (two hero-effect kinds the adventure reads): `bots-report-v1.json`
  regenerated (`h3 ai eval --set report-v1 --json`).
- 2026-10-23 (the battle lane, E23 T5b2b, `battle/e23-t5b2b`, `cross-lane` with the user's consent;
  for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 15: in a computer hero's
  Teleport value every stack's first turn asks its creature cast entry (a Faerie Dragon's spell
  from its last activation; with BAT-QUIRK-25 an Archangel's raise test at the active stack's hit
  points), so the hero may teleport another stack or none. A side without a hero is unchanged. `h3_battle::BATTLE_STATE_HASH_VERSION` is 52 (each stack's last drawn spell is
  state, hidden: no observation, encoding or replay-format change; `h3-sim`'s replay header names
  the version). Name both in a run record's engine identity.
- 2026-10-22 (the adventure lane, E08 T41's shared part, `shared/e08-t41`, `cross-lane` with the
  user's consent; for your information, nothing to do): `python/records/bots-report-v1.json`
  regenerated (`h3 ai eval --set report-v1 --json`): flag 30 retired moved `engine.core`
  (`de583eb06ef4dd73`), and `engine.adventure` catches up with T40's adventure part v10
  (`483fb763b1a0ebbf`; #399 left it at v9's). Every rate as before.
- 2026-10-22 (the battle lane, E23 T5b2a, `battle/e23-t5b2a`; for your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 14: a computer side's hero now also casts Teleport (the original's value, BAT-AI-020; it still never retreats). A side without a hero is unchanged; a hero's decision stays within the 5 ms budget (the slowest 3.5 to 4.3 ms with every spell in the book). Name it in a run record's engine identity; the function's signature is unchanged.
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
- **Performance**, each worth a look when a profile of training shows it: the kill preview
  (AI-Q7 (b), E06 T5) validates each legal strike again (a path search each: 29 µs a step, as
  much as the step); one `h3-battle` function previewing every attack over one reach would cut
  it (the battle lane's); the flat observation
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
