# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** closed 2026-10-20, **E06** closed 2026-10-27 (`t7-1` the battle AI v1 and the baseline); their tasks' history and E06's closing numbers in `QUEUE-ARCHIVE.md`.

**E25** (`docs/epics/E25-battle-ai-v2.md`, AI-Q10; record `E25-log.md`): increments 1
to 8 closed (`E25-archive.md`; increment 8, probes, then one long run, 2026-11-18: no probe nor
the long run passed #2, `E25-log.md` T30 to T36); **`e25-structure-1` #2 the current best**
(AI-Q17, T29: 0.511 and 0.507 against the fair and the all-seeing system actor, the neutral set
+0.302). **A long run stops early** (the user, 2026-11-18, T36): `[gate] futility` at least 1,
gates at least every quarter of the budget, the check named in its rule (`TRAPS.md`, Python).
**Increment 9, a richer observation (v23)** (the user at T36, 2026-11-18, AI-Q18; its decisions
in the epic file). **T37 closed** 2026-11-18 (`E25-log.md` T37, the user's review AI-Q19): #2's
moves are blind to the enemies' reach (0.468, chance 0.455), its strikes read the kills; **v23 is
the reach block** (the design's points 1 to 5), the kill preview for `hexconv` a bench arm
(point 6), T39's rule the clone's ladder +0.02 (point 7). **T38a done** 2026-11-19
(`E25-log.md` T38a): observation v23, the reach block (`layout.REACH`), from
`h3_battle::reach_seen_by`; C ABI v38. **T38b done** 2026-11-19 (`E25-log.md` T38b): `[network]
reach` and `preview` for the hex kinds, the cloning bench's preview rows and `same_start`, T39's
config `configs/distill/e25-reach.toml` (its CPU smoke stopped and resumed).

- **T39 — the bench probe** (the user's GPU, about 6 hours in parts under 2 hours; the design's
  point 7; `configs/distill/e25-reach.toml`, its header the commands: `--out runs/reach-1`, then
  `--resume runs/reach-1` for each later part): the rule stated before it runs: an arm passes
  when its clone's ladder against the fair actor (500 a kind, the same episodes) leads the
  control's by at least 0.02 and its agreement is not below the control's; T40 starts from the
  passing arm with the better ladder, the preview's only when it also leads the `reach` arm's by
  0.02; none passing closes increment 9 at T41. The user uploads the bench's record.

## Inbox

Sorted by destination at E06's audit (T9a, 2026-10-27; E05's at its T9a, 2026-10-19); the
origin stays in brackets. Moved to `QUEUE-ARCHIVE.md` as done or informational: the notices of
E08 T45, T51b, T52, T58 (folded into T55b's below), T70, E23 T4c and T6's last part, the battle
lane's turret preview (its open point below), the baselines, AI-Q7 and E26 T5's fair system actor
(folded into T14); at E25 T37 (2026-11-18) every other lane's informational notice of E08 T63a
to T127b, E09 T8c to T39a and E16 M5 T14.

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

- 2026-11-20 (the adventure lane, E08 T133b, `adventure/e08-t133b`, `cross-lane` with the user's
  consent, E08 decision 384; **your paths changed, for your review; adventure action encoding
  v14 taken**): a market's decision (GAM-DEC-011) is a new `h3_game::Answer::Market`, so
  `adventure/action.rs`' `answer_entry` maps its `Leave` onto the closing role (`DECLINE`) and
  its trade, transfer, sale and purchase onto none, no new entry (the module's docs, the test in
  `adventure/tests.rs`); `legal_commands` lists a market's `Leave` alone until E08 T135a appends
  the 67 entries (design note §18, decision 372: v15 then). Also the adventure replay format v20
  (`adventure/replay.rs`, `answer market …`) and the adventure ruleset part v24 (ADV-Q60 (a):
  `[params.market]`'s `cost`): the bots tables' `engine.adventure` lines (`python/records/
  bots-report-v1.json`, `bots-neutral-v1.json`) regenerated. Random games now open a market's
  decision on the maps with one and answer it with its `Leave`.

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
