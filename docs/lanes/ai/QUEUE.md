# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (record: `E05-log.md`; the environment as built: `docs/07-simulation-and-ai.md` §1.1). **E06** (`docs/epics/E06-battle-ai.md`, AI-Q8; design `E06-design.md`, record `E06-log.md`): increments 1 to 5 and T6a (the behaviour analysis; rerun `python -m h3env.train.inspect` at every checkpoint) done by 2026-10-23. Best so far, `t7-1` (20 M samples, a third neutral): 0.828, 0.551, 0.528 against Random, Aggressor, Greedy, the system actor 0.374; the neutral set's value +0.245 against the target +0.285. The tasks' history is in `QUEUE-ARCHIVE.md`.

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

- 2026-10-24 (the battle lane, `battle/inbox-leftovers`; **for you**, your E06 T6a request):
  **a commanded turret's shot is previewed as it is made** (BAT-DMG-011, BAT-SIE-018):
  `h3_battle::preview` gives its creature's least and greatest damage times its count through the
  target's protections alone (no attack, defense, hero's bonus or penalties), tested against the
  shot's own range. Your measures may count a turret's shots again; the Greedy bot's choices at a
  defended siege with Artillery can change, so regenerate the bot reports when convenient. Also:
  a side without a hero brings no war machines (BAT-WAR-001, the original's 0x463600); `h3-sim`'s
  encounters give machines only with a hero, so nothing of yours moves.

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

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for you: the neutral side's policy is complete**, E23 decision 1 and exit criterion 5): `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` now plays a side without a hero as the original's computer does: shots, melee attacks, moves, waits and defends, and the creature casts (the raise, the Faerie Dragon, the Ogre Mage, the Master Genie). It reads the whole state, draws only from the stream you give it, and validates its choice (a refused one becomes Defend; none is known). `POLICY_VERSION` is 9 (name it in a run record's engine identity). `Policy::System` and the neutral seat can call it now; A12's weights for the neutral share are yours (`E05-log.md` §4.4). A side with a hero still casts nothing and never retreats (BAT-STUB-5, E23 T5 and T6). Cost: within ADR-0027 point 11's budget (≤ 1 ms for the slowest decision; `benches/battle/computer-decisions.toml`).

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

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
