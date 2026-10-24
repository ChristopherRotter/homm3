# AI lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/ai/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Completed tasks (the "Tasks" text as it stood on 2026-10-24)

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

## Notices from other lanes and finished items, newest first

- 2026-10-24 (the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; for your information, done in your paths): **replay
  format v31** (`h3_sim::REPLAY_FORMAT_VERSION`: the setup names the battlefield picture's
  variant, each side's player colour and a hero's sex, for viewers; the CLI's file writes
  `background`, `attacker_colour`, `defender_colour` and a hero's `sex`, each omitted when not
  known); your HANDOFF's versions say so. `h3_battle::Observation` gained `background` and
  `colours`, `HeroView` `sex` (your observation test's literal: `sex: None`): read by no rule, so
  the observation encoding stays v21.

- 2026-10-24 (the battle lane, `battle/e04b-batch-2g` on `shared/batch-2g-quirks`, both
  `cross-lane` with the user's consent; for your information): **batch 2g's rules.** In the wait
  phase every stack not done is ordered alike, slower first, and none may Wait (`legal_commands`
  lists no Wait there: your masks follow it); a freeze no longer counts as its side's action in
  the speed tie-break; a hypnotized stack's morale and luck are its controller's; the surrender
  price leaves Basic Resurrection's temporary creatures out; three fix-optional quirks on in
  `faithful` (flags 34 to 36: a breath never reaches a side column, a retaliation spent on a
  striker the Fire Shield killed, a raise finding a two-hex corpse by its rear hex). The shared
  pull request regenerated your two bot reports (their hash lines).

- 2026-10-24 (the adventure lane, E08 T51a, `adventure/e08-t51a`, `cross-lane` with the user's
  consent; for your information, nothing to do now): **`GameCommand::Manage`** (GAM-MAN-001: a
  player's army, formation, flag and artifact commands on its heroes, in its turn). In your
  paths, one line: `action.rs`' `tileless_entry` maps it to no entry, beside `Adventure` and
  `Battle` (decision 99: the encoding's growth is yours). **`legal_commands` does not list the
  management commands** (ADR-0017 point 6 amended, E08 decision 117: listed per hero by
  `h3_game::legal_management`, a split's counts by `h3_game::split_range`), so the masks, the
  random rollout, the replays' commands and the benchmarks stay as they were. T51b opens the
  meeting and the army dialog as decisions: `legal_commands` then gives `Decline` alone while one
  is open (the masks' `Decline`, T45's note). Also: the adventure replay format v6 (the
  management commands' text; the ten replays' headers only).

- 2026-10-24 (the adventure lane, E08 T50, `adventure/e08-t50`; for your information, nothing to
  do now): **a move toward an own hero is legal now** (GAM-MET-001): the step meets it from the
  tile before at the step's price and ends the move, so the reachable-tiles query, `legal_moves`
  and the env's masks list an own hero's tile as a destination, as another player's (a battle).
  The game opens nothing until T51 (`Decision::Meeting`, where the masks need `Decline`: T45's
  note). Also **adventure state hash v11** (each hero's tactics flag) and **adventure ruleset part
  v11** (`max_stack_count`, a stack's most creatures: 1 000 000 in the base pack): the ten
  adventure replays re-recorded with their options, their commands unchanged; `h3-sim`'s tests
  pass unchanged.

- 2026-10-24 (the adventure lane, E08 T49a, `adventure/e08-t49a`, `cross-lane` with the user's
  consent; for your information, done in your paths): **`HeroDetails.worn` is by position**
  (`Vec<Option<ArtifactInstance>>`, one entry per position of `h3_heroes::positions`; ADR-0013
  point 1): `h3-sim`'s encounter hero starts from `wearing(core, &[])` (every position empty)
  before its `AddArtifact`s, which wear each at its first position as before. **Game state hash
  v12** and **map import v9**: the fixture maps `e08-duel-36` and `e08-xl-144` were regenerated
  (a hero's Spell Book on the map's book entry) and the ten adventure replays re-recorded, their
  commands unchanged; the env's tests pass on them. `HeroChange::PlaceArtifact` is new (a move's
  target, T49b); a battle never tells one.

- 2026-10-23 (the battle lane, E23 T6, `battle/e23-t6`, `cross-lane` with the user's consent; for
  your information, nothing to do): `h3_battle_ai::POLICY_VERSION` is 17 (16: a commanded Catapult
  aims as the original's computer with Ballistics, BAT-AI-021; 17: a commanded First Aid Tent heals
  the stack that lost the most, BAT-AI-022, and a computer side plays its tactics phase, BAT-AI-023:
  its shooters and walkers place themselves and it ends the phase after a round without a
  placement or after three, where it ended the phase at once). `h3_battle::BATTLE_STATE_HASH_VERSION`
  is 53 (`TacticsRounds`, the phase's rounds the computer counts: public, not observed, no rule
  reads it: no observation, encoding or replay-format change). Name both in a run record's engine
  identity.

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

