# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** closed 2026-10-20, **E06** closed 2026-10-27 (`t7-1` the battle AI v1 and the baseline); their tasks' history and E06's closing numbers in `QUEUE-ARCHIVE.md`.

**E25** (`docs/epics/E25-battle-ai-v2.md`, AI-Q10; record `E25-log.md`): increments 1
to 7 closed (`E25-archive.md`; increment 7, the structured network, 2026-11-10); **`e25-structure-1`
#2 the current best** (AI-Q17, T29: 0.511 and 0.507 against the fair and the all-seeing system
actor, the neutral set +0.302, 0.552 head to head against `e25-imitate-1` #2). **Increment 8,
probes, then one long run** (refined at T29, 2026-11-10, AI-Q17; its decisions in the epic file;
the probes about 20 hours of the user's GPU; **T30 done** 2026-11-10, the analysis of #2,
`E25-log.md` T30):

- **T31 The update's speed.** *Scope:* a profile of one PPO update of #2's network on the user's
  GPU (the forward and backward passes, the heads, the optimizer's step; cuDNN's autotuner off
  and on with fixed chunk shapes), a few minutes of the user's machine; then the change that pays,
  with a test that the update's losses and weights stay those of the same rollout, and a CPU
  smoke. *Acceptance:* the seconds an update before and after on the user's GPU, in `E25-log.md`
  T31. *Depends on:* T30 (done). *Status:* next.
- **T32 to T36** (the epic file's table, ranked by T30's read in `E25-log.md` T30): the
  architectures by their clone's ladder (`hexconv` 96 × 6 the control); PPO's settings by short
  continuations from #2 (first the entropy bonus at 0 and the imitation loss held at a small
  weight); the regimes (#2 distilled, a KL anchor, the search at play on the hero battles); the
  probes' checkpoint (with the outcome's weight in training as its question); the long run.
  Each is refined here when it is next.

## Inbox

Sorted by destination at E06's audit (T9a, 2026-10-27; E05's at its T9a, 2026-10-19); the
origin stays in brackets. Moved to `QUEUE-ARCHIVE.md` as done or informational: the notices of
E08 T45, T51b, T52, T58 (folded into T55b's below), T70, E23 T4c and T6's last part, the battle
lane's turret preview (its open point below), the baselines, AI-Q7 and E26 T5's fair system actor
(folded into T14).

**From the town lane** (E09 T8c1, T8c2, T18b2b; for your information, no encoding of yours moved):

- 2026-11-10 (the town lane, E09 T18b2b, `town/e09-t18b2b`, `cross-lane`; **for E09 T19's build
  entry**, decision 56): `h3_game::legal_commands` lists a town's builds
  (`GameCommand::Town(TownCommand::Build { town, building })`, after the boats: each owned town in
  `TownId` order, each payable building of its hall in record order). The env's mask gives them
  no entry (as before), and `adventure::rollout::choose` now draws from `drawn_commands`, the
  listed commands but the builds, so no random game, replay or bot report moved. E09 T19 adds the
  build entry (design §32: the town's entrance as the tile, a building head over
  `h3_towns::MAX_BUILDINGS`) in an encoding version after E08's v8, with your layout, and drops
  the skip; its random games then move.

- 2026-11-06 (the town lane, E09 T8c1 `town/e09-t8c1` (#750, merged) and T8c2 `town/e09-t8c2`,
  `cross-lane`): `h3_game::GameObservation` gains `towns: Vec<TownView>` (each seen town's public
  facts, its garrison by the viewer's level, the owner's facts at the top level; design note
  `E09-design.md` §17, `docs/06-security.md` "Towns") and `h3_adventure::ObjectView::town`. The
  day's advance pays each owned town's `h3_towns::daily_income` (its hall's gold, the Grail's, its
  silo's resources) from day 2, in the same `ResourceGained` as the mines' and the heroes'
  (TWN-INC-001, -002). Since T8c2 a Rogue in a hero's army shows another team's town within its
  Visions radius at the top level (TWN-VEW-003). The town part's hash (v3 since T8c2) is in the
  adventure replays' headers only. E09 T9 (the env's town fixtures, design §18) comes next.

**From the adventure lane** (E08 T95b, T105b2; the adventure agents' action space grows):

- 2026-11-10 (the adventure lane, E08 T105b2, `adventure/e08-t105b2`, `cross-lane` with the
  user's consent, E08 decision 268; **the env's action encoding v8, for the adventure agents, and
  a step cap**): `crates/h3-sim/src/adventure/action.rs` appends eight entries `REVISIT_BASE + s`
  (22 to 29, one per hero slot; `ENTRY_COUNT` 30): the hero in slot `s` revisits the entered
  object it stands on (`AdventureCommand::Revisit`, ADV-TEL-008), tile 0 alone as the answers',
  masked by `legal_commands` (which lists one per such hero after the `BuildBoat`s). A revisit
  costs no points, so a turn may revisit without end (the original's space key: a two-way
  monolith re-drawn, a Whirlpool's loss again; increment 11's design note §18): **an agent's
  rollout needs a step cap per turn**, your call (`random_rollout` ends only when another entry is
  drawn). Random play changed course on every map with an entered object (the `random-play-*`
  workloads re-baselined, the town map's env tests on new seeds 36 and 330); a policy trained on
  v7 sees eight new entries.

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

- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`, `cross-lane` with the user's
  consent, E08 decision 243; **the env's action encoding v7, for the adventure agents**):
  `crates/h3-sim/src/adventure/action.rs` appends four entries `HIRE_BASE + c` (18 to 21,
  `HIRE_SLOTS = 4`; `ENTRY_COUNT` 22): buy the most of creature `c`, in its source's order, of
  the player's open hire dialog (masked by `legal_commands`: a creature of none to buy is
  masked); the dialog's `Close` is entry 10. The most only, as the original's Max then
  Recruit: an amount head is the AI lane's call. A policy trained on v6 sees four new entries,
  masked on every map without a dwelling or a Refugee Camp (no committed map has one before
  E08 T96).

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

- 2026-11-02 (the adventure lane, E08 T85b2b, `adventure/e08-t85b`, `cross-lane`, decision 203;
  for your information, nothing to do): the adventure env's **action encoding v6** appends
  `BUILD_BOAT` (entry 17; `ENTRY_COUNT` 18): its tile head names a Shipyard by its anchor's tile,
  masked by the `BuildBoat`s `h3_game::legal_commands` lists (ADV-SHY-003: the player's or an
  ally's Shipyard, its boat tile free, the price in the treasury); `decode` builds the command at a
  Shipyard the adventure lets the player use (`AdventureState::buildable_shipyards`), the game
  refusing it out of its turn or without the price. The random agent's draw of a listed
  `BuildBoat` now builds (it ended the turn before); no committed map has a Shipyard, so no
  replay, bench or bots record moves. The same branch brings game state hash v22 (T85b1: the
  month's kind, the timed events) and `GameObservation::month` and `events_today`.

- 2026-10-31 (the adventure lane, E08 T83a, `adventure/e08-t82-t83a`, `cross-lane`; for your
  information, nothing to do): adventure ruleset part v14 (increment 9's seven kinds, the
  calendar, the growth and the months), map import v15, adventure state hash v17.
  `python/records/bots-report-v1.json` and `bots-neutral-v1.json` regenerated: only
  `engine.adventure` moved (`7576c6249341c03a`). Two setup fields reached your test literals:
  `owner: None` in `crates/h3-sim/src/adventure/tests.rs` and `tests/monsters.rs`'
  `ObjectSetup`s, `events: Vec::new()` in `tests/decisions.rs`' and `crates/h3-cli/src/ai/
  census.rs`' `GameSetup`s; no encoding of yours moves.

- 2026-10-30 (the adventure lane, E08 T81b, `shared/e08-t81b`, `cross-lane`; for your
  information, nothing to do): core ruleset hash v51 (four hero effect kinds, two hero parameters,
  a creature's abilities outside battle; pack format v50). `python/records/bots-report-v1.json`
  and `bots-neutral-v1.json` regenerated: only `engine.core` moved (`962e87e7a01056f6`); the
  `e25-*` run records keep the hash they ran with. Nothing reaches the battle's observation or
  your encodings.

- 2026-10-30 (the adventure lane, E08 T77b, `adventure/e08-t77b`; decisions 202, 203; **for
  you: a notice, nothing to do now**): increment 9 (mines, the week and owned objects) adds one
  command, `AdventureCommand::BuildBoat { object_type, anchor }` (a boat from an own or allied
  Shipyard without a visit, no hero named). The user chose (decision 203, as decision 172) that the
  adventure lane makes the **action encoding v6** change `cross-lane` in its task T85b: one entry
  per Shipyard the viewer may use, by its anchor's tile index, masked by `legal_commands`. The
  observation gains the owners, the weekly objects' visited marks, the month's kind and
  `PlayerView::events_today`; no flat observation encoding exists (decision 187), so nothing of
  yours moves there. The army dialog (Leave Guards) and the new offer kind (a boat for a price)
  are typed answers already. Design: `docs/epics/E08-design.md` §8, §14, §15.

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
- 2026-10-27 (the adventure lane, E08 T63a, `adventure/e08-t63a`, `cross-lane` with the user's
  consent; decision 153; **for you**): **the answers are typed and your action mapping moved with
  them (encoding v5).** `h3_game::Answer` is an enum by decision kind: `Offer(Consent)`
  (`Accept`, `Decline`), `LevelUp(SkillOffer)`, `Close` (a meeting, an army dialog), `Chest`
  (`Gold`, `Experience`), `Arena` (`Attack`, `Defense`), `School` (`First`, `Second`, `Leave`),
  `University` (`Buy(slot)`, `Leave`). `src/adventure/action.rs` maps answers onto entries by role
  (`answer_entry`): 9 an offer's acceptance; 10 a declined offer, a closed meeting or army dialog,
  a School or University left; 11 and 12 the first and second of a level-up's, a chest's, the
  Arena's and a School's answers; **13 to 16 (new) a University's purchase of slot 0 to 3**
  (`BUY_BASE`, `UNIVERSITY_SLOTS`; the user chose the entries now). `decode` takes the listed
  answer of the entry's role, else the offer's or level-up's answer or the purchase, which the game
  rejects; the masks come from `legal_commands` as before, so every decision has a legal entry.
  The new decisions' views (`DecisionView::Choice`, `DecisionView::University { visit, slots }`)
  are in the observation; whether your encoding reads them is yours (this replaces T55b's notice
  of 2026-10-26). The rollout's legal-command order keeps the old games' draws: the twelve golden
  replays recorded the same commands. **A-23 withdrawn** (E08 T58: `legal_moves` never reads events). Design:
  `docs/epics/E08-design.md` §14, §16; ADR-0017 points 6, 8.

**For E25's closure** (a `shared/` pull request): `docs/07-simulation-and-ai.md` §1.2, the
pipeline as built, gains E25's training levers: shaping (T2a), the tactics bound (T4a), the
hex networks and the MLP's depth (T7a), the distillation bench (T7b), the networks' keys of
increment 7 (T24's `norm`, `feed_forward`; T25's `mask_input`, `order`, `attack_hex`), PPO's `[ppo] accumulate` (T28); the two system actors (T14:
`system` the fair one, `system-all-seeing`, C ABI v36).

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
