# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** closed 2026-10-20, **E06** closed 2026-10-27 (`t7-1` the battle AI v1 and the baseline); their tasks' history and E06's closing numbers in `QUEUE-ARCHIVE.md`.

**E25** (`docs/epics/E25-battle-ai-v2.md`, AI-Q10; record `E25-log.md`): increments 1
to 10 closed (`E25-archive.md`; increment 10, the ceiling, 2026-11-23: against the fair actor
+0.085 on the side's worlds, +0.115 on the true battle; #2 +0.026, the rollouts +0.059 over it;
`E25-log.md` T42 to T44); **`e25-structure-1` #2 the current best** (AI-Q17, T29: 0.511 and
0.507 against the fair and the all-seeing system actor, the neutral set +0.302). **A long run
stops early** (the user, 2026-11-18, T36): `[gate] futility` at least 1, gates at least every
quarter of the budget, the check named in its rule (`TRAPS.md`, Python). **Paused after
increment 10** (the user at T45, 2026-11-23, AI-Q23): increment 11, expert iteration, when it
resumes (the epic file); its tasks are cut then.

**E31** (`docs/epics/E31-original-computer-player.md`, AI-Q22; refined 2026-11-21): next (since
T45, AI-Q23).

- **E31 T1a1 — the frame, in three parts** (AI-Q24): **done** 2026-11-23 (`E31-log.md` T1a1a
  to T1a1c; the survey's §1).
- **E31 T1a2 — the values, in four parts** (AI-Q25: T1a2b to T1a2d read by sub-agents):
  **done** 2026-11-23 (`E31-log.md` T1a2; the survey's §2).
- **E31 T1a3 — the towns and the answers, in four parts** (AI-Q26): **T1a3a** (the build choice)
  and **T1a3b** (the hires) **done** 2026-11-23 (`E31-log.md` T1a3a, T1a3b; the survey §3.1, §3.2).
- **E31 T1a3c — the town visit** (`Heroes3.exe` staged, `pip install capstone pefile`, a cloud
  sandbox): `ai_town_visit` 0x5253D0 whole with the nine unnamed helpers it calls (the garrison
  exchange, the recruits, the buys, by their calls an artifact merchant, the Blacksmith and the
  artifacts' arrangement; `scripts/exe.py callees ai_town_visit`) and `ai_university` 0x524ED0,
  and the army plan's open pieces (0x42C3B0, 0x42D1B0, 0x42D870, 0x42D8E0; where
  `ai_army_plan_run`'s purchases go; `ai_town_defence_recruit`'s recruits); about 2 700
  instructions: confirm a further cut at its start. Into the survey's §3.3.
- **E31 T1a3d — the answers**: the 43 functions of the computer's ranges that the game's code
  calls (about 25 of them answers to visits: `ai_garrison_visit`, `ai_dwelling_visit`,
  `ai_join_without_room`, `ai_artifact_received`, `ai_meeting`, …), how `ai_fight_answer`'s 17
  callers use the object value, the Seer's reward (0x573A70), the market visits (a map Tavern
  included), the allies' shares (`ai_ally_resource_share` 0x429110); every `ai_*` call classified
  a choice or a rule, with T1a2's (the survey §2.5) and §3.1 to §3.3's, into the survey's §3.4
  table.
- Then T1a4 (it maps the rest of the third range, 0x423EE0 to 0x42A000: the survey's §0), T1b.

## Inbox

- 2026-11-23 (the town lane, E09 T49b, `town/e09-t49b`, `cross-lane` with the user's consent,
  E09 decision 194; **in your paths, nothing to do**): the adventure env's **action encoding v19**
  appends `SPELL_BOOK` (104) after `TOWN_OPEN` (`ENTRY_COUNT` 105): a spell book's purchase at a
  town's Mage Guild (`TownCommand::BuySpellBook`, TWN-GLD-007) by its town's entrance in the tile
  head, building 0, one action per listed purchase (a standing guild, a bookless visitor else
  garrison hero, 500 gold). `tileless`, `tile_mask` (with `TOWN_OPEN` through `guild_command`),
  `decode`, `encode`, `Action::spell_book`, the layout test and the mask property's
  `check_town_heroes`. The random games on the town map buy books, so `tests/towns.rs`' seeds
  moved (its `Played` counts the purchases) and the `*-towns-36` workloads were re-baselined.
  The adventure lane's v19 (E08 T144a) becomes **v20**, its entries from 105.

Sorted by destination at E06's audit (T9a, 2026-10-27; E05's at its T9a, 2026-10-19); the
origin stays in brackets. Moved to `QUEUE-ARCHIVE.md` as done or informational: the notices of
E08 T45, T51b, T52, T58 (folded into T55b's below), T70, E23 T4c and T6's last part, the battle
lane's turret preview (its open point below), the baselines, AI-Q7 and E26 T5's fair system actor
(folded into T14); at E25 T37 (2026-11-18) every other lane's informational notice of E08 T63a
to T127b, E09 T8c to T39a and E16 M5 T14.

**From the adventure lane** (E08 T137b, `adventure/e08-t137b`, docs only; **planned, nothing to do now**; E08 T140, `shared/e08-t140`, `cross-lane`; **in your paths, nothing to do**):

- 2026-11-21: E08 increment 15 (design note `docs/epics/E08-design.md` §17, decision 407) gives
  the adventure env's action encoding **v18** at T144a, `cross-lane` in your paths: three casts
  without a target (Summon Boat, Fly, Water Walk) and two 19 × 17 target windows around the hero
  (Dimension Door, Scuttle Boat), masked by `legal_commands` (up to 323 targets each: the random
  workloads' weights and step cap are yours); the own hero's observation gains its day spells.
  Under `strict-fixed` a flight's move may end early at hidden rock (its rest dropped, as an
  event's stop); four new fix-optional flags make `faithful`'s legal moves read hidden tiles.
- 2026-11-22 (E08 T140): increment 15's six fix-optional flags joined the core's profiles
  (`ADV-QUIRK-15`, `-51` to `-55`, `QuirkFlag(45)` to `(50)`, on in `faithful`), so the base
  faithful ruleset hash moved `0x36b98ea152b4db95` to `0x0ee6066a4c802530`: `bots-report-v1.json`
  and `bots-neutral-v1.json` regenerated (only `engine.core` moved). The E25 ceiling records
  (`python/records/ceiling/*.json`) keep the old `core` as the record of their runs (no test reads
  it); nothing reads the flags until E08 T142 and T143.

**From the town lane** (E09 T43, `town/e09-t43`, docs only; **planned, nothing to do now**):

- 2026-11-22 (E09 T49a2, `town/e09-t49a2`, `cross-lane` in your paths with the user's consent,
  E09 decision 193; **adventure action encoding v18 taken, for your review**): `TOWN_OPEN` = 103,
  appended after E08 T135a's entries rather than beside the taverns' (so that no entry moves;
  `ENTRY_COUNT` 104): a town screen's opening by its town's entrance in the tile head, building
  0, one action per `Open` `legal_commands` lists, which it lists only where the opening would
  give the Mana Vortex or a spell (the random games on the town map met none: `h3-sim`'s
  town-map seeds and their counts unchanged). The module's table and docs, `tileless` (its
  answers' range now ends before it), `tile_mask`, `decode`, `encode`, `Action::town_open`, the
  layout test and the mask property's `check_town_heroes`. The adventure lane's v18 above
  (E08 T144a) becomes **v19**, its entries from 104. `SPELL_BOOK` (T49b) is still to come.

- 2026-11-20: E09 increment 5 (the mage guild; design note `docs/epics/E09-design.md` §95, accepted
  at `TWN-Q33`) appends two entries to the adventure env's action encoding, `cross-lane` in your
  paths: `TOWN_OPEN` (T49a; the town by its entrance in the tile head; one action per `Open` the
  legal commands list, only where it would change something) and `SPELL_BOOK` (T49b; the same head;
  listed where the town's guild stands, its hero lacks a book and the treasury holds the price),
  each the next version at merge (one bump when they land together). `h3-sim`'s town-map tests and
  the `*-towns-36` workloads move with them; the guild's draw from the towns' stream (T47a, decision
  179) moves every later towns' draw, so the town-map seeds may move there too.

**From the battle lane** (CLT-Q40, BAT-Q64; `claude/keen-cori-oaahgi`, `cross-lane`; for your
information and one choice):

- 2026-11-03: the battle state hash v55 and the replay format v32 (a hero's morale and luck effects
  name their sources, `Siege.bonuses` replaces `morale` and `luck`); the golden battle and
  adventure replays rehashed, their commands unchanged; `encounter.rs`'s siege gives the table's
  town morale as one stand-in building (0), `h3-ffi`'s `scenario/town.toml` as building 5. The
  observation gains public fields no rule reads (`morale_and_luck`, `unclamped_morale`,
  `unclamped_luck`); **your choice**: whether the observation encoding takes the unclamped luck
  (the computer's Fortune value reads it, E23 T4c) or nothing.

**The adventure env** (yours since E08 T45):

- 2026-11-21 (the adventure lane, E08 T135a, `adventure/e08-t135a`, `cross-lane` with the
  user's consent, E08 decisions 372, 389; **your paths changed, for your review; adventure action
  encoding v17 taken**): 66 entries appended after the taverns' (`ENTRY_COUNT` 37 to 103): a
  market's `MARKET_TRADE` (one lot of each of 42 pairs, `trade_entry`, `market_pair`),
  `MARKET_SELL` (one lot of a slot's stack for the cost resource) and `MARKET_BUY` (an entry in
  it), an Altar's `ALTAR_ARTIFACTS` and `ALTAR_CREATURES` (its two alls), a Hill Fort's
  `HILL_FORT_UPGRADE` (by slot) and `HILL_FORT_ALL`, each legal where `legal_commands` now lists
  the answer (the module's docs and table, `answer_entry`, `encode`'s listed-only sale, purchase
  and sacrifice, `decode`'s `service_answer`). The decoder's revisit and hire places are bounded by
  their counts (they read every later entry as theirs). Tests: `adventure/tests.rs` (the layout,
  the mask's literal now built from `ENTRY_COUNT`, the property test's ranges), two in
  `adventure/tests/decisions.rs`. An amount head (more lots, a transfer) stays yours. The bots
  tables did not move; random games on maps with a market, an Altar or a Hill Fort now trade,
  sacrifice and upgrade (no fixture map holds one until E08 T135b).
- 2026-11-20 (the adventure lane, E08 T134b1, `adventure/e08-t134b1`, `cross-lane` with the
  user's consent, E08 decision 387; **your paths changed, for your review; adventure action
  encoding v16 taken**): a Hill Fort's decision (GAM-DEC-013) is a new
  `h3_game::Answer::HillFort`, so `answer_entry` maps its `Leave` onto the closing role
  (`DECLINE`) and its `Upgrade` and `All` onto none, no new entry (the module's docs, the test in
  `adventure/tests.rs`); `legal_commands` lists its `Leave` alone until E08 T135a. Also the
  adventure replay format v23 (`adventure/replay.rs`, `answer hill-fort …`); the bots tables did
  not move. Random games now open a Hill Fort's decision on the maps with one and leave it.
- 2026-11-20 (the adventure lane, E08 T134a, `adventure/e08-t134a`, `cross-lane` with the user's
  consent, E08 decision 385; **your paths changed, for your review; adventure action encoding
  v15 taken**): an Altar of Sacrifice's decision (GAM-DEC-012) is a new `h3_game::Answer::Altar`,
  so `answer_entry` maps its `Leave` onto the closing role (`DECLINE`) and its two sacrifices onto
  none, no new entry (the module's docs, the test in `adventure/tests.rs`); `legal_commands` lists
  its `Leave` alone until E08 T135a (encoding the next free then). Also the adventure replay format
  v22 (`adventure/replay.rs`, `answer altar …`); the bots tables did not move. Random games now
  open an Altar's decision on the maps with one and leave it.
- 2026-11-20 (the adventure lane, E08 T133b, `adventure/e08-t133b`, `cross-lane` with the user's
  consent, E08 decision 384; **your paths changed, for your review; adventure action encoding
  v14 taken**): a market's decision (GAM-DEC-011) is a new `h3_game::Answer::Market`, so
  `adventure/action.rs`' `answer_entry` maps its `Leave` onto the closing role (`DECLINE`) and
  its trade, transfer, sale and purchase onto none, no new entry (the module's docs, the test in
  `adventure/tests.rs`); `legal_commands` lists a market's `Leave` alone until E08 T135a appends
  the 67 entries (design note §18, decision 372: v15 then). Also the adventure replay format v21
  (`adventure/replay.rs`, `answer market …`; after the client lane's v20) and the adventure ruleset part v24 (ADV-Q60 (a):
  `[params.market]`'s `cost`): the bots tables' `engine.adventure` lines (`python/records/
  bots-report-v1.json`, `bots-neutral-v1.json`) regenerated. Random games now open a market's
  decision on the maps with one and answer it with its `Leave`.

- 2026-11-20 (the client lane, E27 M4 T3a, `client/e27-m4-t3a`, `cross-lane` with the user's
  consent; **your tests touched, nothing to do**): the adventure replay format v20
  (`Replay::players`, the players' choices) gave four `Replay` literals of `h3-sim`'s
  `adventure/tests` `players: Vec::new()`, `tests/hiring.rs`' version assertion 20, and
  `tests/towns.rs` a test of the choices on the town map
  (`a_replay_keeps_the_players_choices_over_the_import`).
- 2026-11-07 (the adventure lane, E08 T103b, `adventure/e08-t103b`; **for when you regenerate
  `census-v1.toml`**): `h3 ai census original-data/install/Maps` counts 1 of the 159 maps on
  `master` and on this branch: every other map stops at `GameState::new` with "object N has an
  owner it may not have" (`SetupError`), as `src/ai/census.rs`' setup plays colour 0 alone while
  the maps' owned objects and towns (typed since E08 T83a and E09 T8a1) belong to other colours.
  `h3-map`'s `original_data` builds each map in a game of its own players (the map's colours whose
  human or computer flag is set); the same would let the census count all 159 again. The committed
  `census-v1.toml` is untouched (no gate regenerates it). T103b's change (the teleporters typed,
  map import v21) should move none of its numbers (their tiles block and are visited as before),
  unchecked while the census stops.
- 2026-10-29 (the adventure lane, E08 T74b, `adventure/e08-t74b`; decision 187, replacing T66b's
  notice of decision 172; **for you: one point to decide, the census below**): increment 8 (boats
  and the sea) moves **no encoding of yours**. The adventure env has no flat observation encoding:
  `AdventureEnv::observation` returns `h3_game::GameObservation` as it is, and its views carry the
  boats since E08 T72 (`AdventureObservation::boats`: tile, type, facing of the empty boats where
  the viewer sees heroes; `HeroView::boat`, a seen hero's boat's type; `OwnHeroView::max_movement`);
  the action encoding keeps v5 (a boarding or a landing is a tile among `legal_moves`). New since
  T73 and T74: the events `Boarded`, `Landed`, `MessageRead`, `BoatRemoved`; battles with a hero in
  a boat on water's terrain and the `deck` battlefield, both in boats on `boats`
  (`AdventureState::sea_battle_ground`). Your census (`h3-cli/src/ai/census.rs`) places every battle
  by `battle_ground` of the defender's tile: only a boat reaches a monster on water (D-46), so such
  a monster's battle is on the deck (`sea_battle_ground(…, 1)`), which the census counts at the
  water tile's own ground; whether it should is yours. When you encode the adventure for learning,
  the boats are in the views. Design: `docs/epics/E08-design.md` §12, §13, §15.

**For E25's closure** (a `shared/` pull request): `docs/07-simulation-and-ai.md` §1.2, the
pipeline as built, gains E25's training levers: shaping (T2a), the tactics bound (T4a), the
hex networks and the MLP's depth (T7a), the distillation bench (T7b), the networks' keys of
increment 7 (T24's `norm`, `feed_forward`; T25's `mask_input`, `order`, `attack_hex`), PPO's
`[ppo] accumulate` (T28), `chunk_align` and `[device] cudnn_autotune` (T31; the speed tool
`h3env.train.speed`), `imitation_floor` and the gate's `--fixed` head-to-head (T33); the GPU's
memory cap (T28); the two system actors (T14: `system` the fair
one, `system-all-seeing`, C ABI v36).

**For E25's later increments** (the epic's candidates; E06's leftovers, T9a; increment 1's
parts are its tasks above):

- **The levers E06 left** (its increment 7 checkpoint's candidates): `[network] order` (T6g)
  and `preview` (T6e), no gain on the MLP, off by default, worth a second look with a network
  that reads the battle's structure; shaping (T2a) also against the wandering T6e found: the
  tactics phase's placements and moves cost nothing at γ 1; the 60 M run at T7's mix
  (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours; without the faithful system
  actor now, E25 decision 2); the mana-price run `t5-4` (`configs/e06-mana.toml`, about 75
  minutes; T5's experiment, prepared, never run).
- **The search** (E06 T8, ADR-0025 point 12a; its terminal and T8c again were T1a to T1c): a
  world scores a cut at the step limit 0 where the trainer charges `-truncation_penalty`, so the
  tree's losing side stalls (T1c: up to 1.5 % of the true worlds' battles cut); a search that
  trains or plays scores it as the trainer does (`E25-log.md` T1c). AI-STUB-1
  (`docs/stubs.toml`: the hidden obstacles' number, and two of T9b's three gaps of a world: the
  other hero's morale and luck modifiers, a commanded war machine without a guess; the magic
  limits closed 2026-10-28, `E25-log.md`);
  the cast heads at the root unperturbed (design §14's Gumbel heads not built); T8c's optional
  Greedy rows on the user's machine (`uv run --group train python -m h3env.search --report
  --best runs/t7-1/best.pt --device cuda --opponent greedy --simulations 64 --json
  runs/t8c-greedy-64.json`). Evaluation: greedy play (the most likely heads) repeats itself
  against Random until the step limit (up to 17 % of the battles in E05 T8b's sandbox run); the
  sampled policy never. A search agent that plays its most visited action needs a rule for it (a
  tie-break or repetition rule); the 2 000-step limit is only a guard (E05 T8b). The VCMI oracle
  harness was never built (E04 refinement, 2026-10-01): revisit if E25's evaluation needs it.
- **Throughput** (T5's open parts; E05 T9c's table): the batch size and thread count (on the
  user's Ryzen 9 3900X the step scales to 9.9 × at 1 024 envs on 24 threads but only 3.8 × at 64
  envs, `E05-log.md` T9c; E06's runs kept the default 256 envs on every core); the observations written in place (the
  trainer copies each call's observations, 9 MB at 256 envs, into the rollout; the batch could
  write them per call into `H3BatchBuffers.obs`: E06 T5, no ABI change, design §10). Each worth
  a look when a profile of training shows it: the kill preview (AI-Q7 (b), E06 T5) validates
  each legal strike again (a path search each: 29 µs a step, as much as the step), one
  `h3-battle` function previewing every attack over one reach would cut it (the battle lane's);
  the flat observation costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈
  3.5 µs; the buffer is zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB,
  too slow for per-node hashing (E04b T11a); an encounter sample costs 50 µs (two heroes
  derived, each artifact through `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 %
  of an episode (T5b2). The battle lane's `full` workload is lopsided (8 attacker victories in
  2 399 agent seeds; E04b T11a): the encounter workload (`batch-random-encounters-v1`) is the
  realistic one.
- **The behaviour analysis** (the battle lane, 2026-10-24, its turret preview): `inspect` may
  count a commanded turret's shots again (it leaves them out of the strikes).

**Encounters v2** (with the adventure's logged encounters, decision 7; E05 T9b):

- A7's "+0.5 a week" in weeks 9 to 16 is rounded down in `encounters-v1.toml` (odd weeks half a
  level low); untested: A4's cap and redraws in weeks 1 to 6, the forts of weeks 2 to 6, A7's
  war-machine chance and class split, A6's upgraded share, the guards' anti-magic share.
- A3 grows a wandering stack by a tenth of its count, VCMI 1.7.5's `CGCreature::newTurn` a tenth
  of its power (a thousand per creature: 5 creatures are 9 after seven weeks, the count's tenth
  leaves them 5); VCMI's growth makes late-week stacks pass A4's cap and be drawn again, so
  decide both together (E05 T9b; meant for E23, which closed 2026-10-24 without it: the
  sampler's, so here, T9a).

**For E13** (the adventure env's ABI): reading both of the adventure env's mask heads doubles
its step; the current state's queries belong in the env (the adventure lane, E08 T9b).

**Any time** (small, no epic):

- **Encodings:** `h3_sim::action::encode` answers without a viewer (the same root as F-sec-2's
  leak, which the sim's `step` closes); it leaks nothing today (E04b T11b, the battle lane).
- **The C ABI** (E05 T9b): `h3env.tools.Env` keeps its handle after `H3_ERR_PANIC` (tests only;
  `h3_batch_create`'s unchecked N closed by E06 T2c).
- **The engine's identity** (E06 T9b, r3): a run's record takes `commit` from the Python tree,
  while the library names only its ABI version, so a stale `target/release` build or an
  `$H3_LIBRARY` at the same ABI is recorded under the wrong code (a build identity read back
  from the library would close it); `test_the_layouts_match_h3_h` compares sizes, not offsets.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
