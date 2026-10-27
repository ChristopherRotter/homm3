# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (record: `E05-log.md`; the environment as built: `docs/07-simulation-and-ai.md` §1.1). **E06** (`docs/epics/E06-battle-ai.md`, AI-Q8; design `E06-design.md`, record `E06-log.md`): increments 1 to 7 done by 2026-10-27; the battle AI v1 is `t7-1` (20 M samples, a third neutral): 0.828, 0.551, 0.528 against Random, Aggressor, Greedy at +0.273, +0.029, +0.010, the neutral set's value +0.245 (decision 7's targets as revised 2026-10-27, met), the system actor 0.374 (E25's target). The tasks' history is in `QUEUE-ARCHIVE.md`.

**Increment 7** closed 2026-10-27 at the user's checkpoint (its task list in `E06-archive.md`):
T8a to T8c, T6f and T6g, the turn order as an input and the search over determinized worlds,
both negative; T8d not built. The user revised decision 7's targets to "beat every heuristic
bot", which `t7-1` meets, and chose E06's closure with `t7-1` as the battle AI v1; the stronger
agent is E25's (BACKLOG, a sketch; AI-STUB-1 moved there).

**Increment 8, T9 the closure** (refined 2026-10-27; the epic file's task list, E05's T9 as the
pattern). **Merge `shared/e06-search-docs` first** (ADR-0025 point 12a, the env doc, BACKLOG).

- **T9a The audit** (the sandbox; next): exit criteria 1 to 5 against the code, tests and
  committed records; E06's leftovers sorted by destination (E25, encounters v2, E13, another
  lane's inbox) or dropped: T5's open parts (`t5-4`, the batch size and threads, the
  observations in place), the candidates below, this inbox; the Python READMEs current.
- **T9b The independent review** (the sandbox): at most three fresh read-only reviewers, one
  slice each (the trainer; the search and its worlds; the batch since E05, C ABI v31 to v34),
  each finding with a failure path, verified, then fixed.
- **T9c The verifies** (the user's machine; exit criterion 6): `cargo xtask verify
  --with-original-data` and `uv run --group train pytest` on Windows; on Linux CI's full verify
  and a sandbox with the user's files staged.
- **T9d The closure:** `shared/e06-closure` first (BACKLOG E06 done, the env doc's training
  pipeline as built, ADR-0026's detail line), the design note at closure, the epic file done.

- **Candidates for E25** (T9a moves them; nothing runs in E06): `[network] order` (T6g: no
  gain, off by default); potential-based shaping (also against the wandering T6e found: the
  tactics phase's placements and moves cost nothing at γ 1); the 60 M run at T7's mix
  (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours); the mana run `t5-4`
  (`configs/e06-mana.toml`, about 75 minutes); T8c's optional Greedy rows on the user's machine
  (`uv run --group train python -m h3env.search --report --best runs/t7-1/best.pt --device cuda
  --opponent greedy --simulations 64 --json runs/t8c-greedy-64.json`).

## Inbox

Sorted by destination at E05's audit (T9a, 2026-10-19); the origin stays in brackets. Dropped as
done: BAT-Q33's point for E05 (AI-Q3, ADR-0025 point 6), the `h3-game` coverage gap of a pending
Pass offer (fixed by the adventure lane's fixed tests, #266).

- 2026-10-27 (the adventure lane, E08 T58, `adventure/e08-t58`; **for you**, correcting the
  T55b entry below): **A-23 is withdrawn.** Batch E7's row E7-23 showed the original's path
  crossing a hidden event, and the re-read found an event's tile no stop for its pathfinder. So
  `legal_moves` never reads events in any profile, and your tile masks reveal nothing of them;
  a step into a firing event ends the move there (the move's outcome). Survey
  `docs/rules/surveys/E08-rewards.md` §15; design §12.

- 2026-10-26 (the adventure lane, E08 T55b, `adventure/e08-t55b`; decisions 138–141; **for you,
  when the adventure lane's T63 lands**): **answers typed by decision.** `h3_game::Answer` becomes
  an enum by decision kind, each variant with its own type: `Offer(Consent)` (`Accept`,
  `Decline`: every yes-or-no offer), `LevelUp(SkillOffer)` (`First`, `Second`), `Close` (a
  meeting, an army dialog; today's `Decline` there), and increment 7's `Chest` (`Gold`,
  `Experience`), `Arena` (`Attack`, `Defense`), `School` (`First`, `Second`, `Leave`) and
  `University` (`Buy(slot)`, `Leave`). T63 migrates every caller, `h3-sim`'s
  `src/adventure/action.rs` mapping of your entries included (`cross-lane`, the user's consent):
  accept and decline onto `Offer` or `Close` by the open decision, the first and second entries
  onto `LevelUp` and the new two-way choices. Yours to decide after it: whether a University's
  purchase gets a head (until then the masks allow its `Leave`), and whether the new decisions'
  views enter the observation. Your masks must allow at least one answer per decision (the system
  actor's defaults: `Gold`, `Attack`, `Leave`, `Decline`, `Close`). Under `faithful`, if batch
  E7's row E7-23 confirms A-23, `legal_moves` treats an event's tile as an entrance for every hero
  (your tile masks then reveal hidden events, as the original's path does; `docs/06-security.md`
  will say so). Design: `docs/epics/E08-design.md` §12, §14, §16; ADR-0017's draft point 6, 8.

- 2026-10-26 (the adventure lane, E08 T52, `adventure/e08-t52`; **for you**, the entry below):
  **the env plays on through a meeting and an army dialog** in the adventure lane's own test
  (`h3-cli`'s `adventure::tests::the_env_plays_on_through_a_meeting_and_an_army_dialog`: your
  `random_rollout` plays every seat to the day limit after either opened, its first action the
  listed `Decline`). A map for your own test: `fixtures/maps/e08-army-12.h3m` (new), where red's
  first hero (slot 0, tile (3, 5)) meets its mate by one step east and is offered six goblins'
  join without room by one step west (`Action::accept()` then opens the army dialog). The
  adventure lane changed nothing of yours; the management commands still reach agents only
  through `h3_game::legal_management` (the env's encoding has no entry for them).

- 2026-10-26 (the adventure lane, E08 T51b, `adventure/e08-t51b`; **for you**, ADR-0017
  point 8, `E08-design.md` §12): **a meeting and an army dialog are decisions now.** A step
  toward an own hero opens a meeting (`GameState::meeting`), an accepted join without room the
  army dialog (`GameState::army_dialog`; the offer to join is now made whatever room the army
  has); while either is open `legal_commands` lists `Decline` alone (your entry 10, so your masks
  allow it with no change and the rollout never stalls), `default_answer` is `Decline`, no move is
  legal, and the management commands (`legal_management`, no entry of yours) name the two heroes
  or the hero and the group. `GameState::decision` stays the offer's alone: code that declines
  "an open decision" by `decision().is_some()` should also test `meeting()` and `army_dialog()`
  (the adventure lane did so in `h3-cli`'s bench `seek`). New `GameEvent`s (`MeetingOpened`,
  `ArmyDialogOpened`, `DecisionClosed`, `GroupLost`) and `DecisionView::{Meeting, ArmyDialog}`;
  game state hash v13. Your test of the env with a meeting open is T52's (ask: "the env plays on
  through meetings and army dialogs", exit criterion 6).

- 2026-10-24 (the battle lane, `battle/inbox-leftovers`; **for you**, your E06 T6a request):
  **a commanded turret's shot is previewed as it is made** (BAT-DMG-011, BAT-SIE-018):
  `h3_battle::preview` gives its creature's least and greatest damage times its count through the
  target's protections alone (no attack, defense, hero's bonus or penalties), tested against the
  shot's own range. Your measures may count a turret's shots again; the Greedy bot's choices at a
  defended siege with Artillery can change, so regenerate the bot reports when convenient. Also:
  a side without a hero brings no war machines (BAT-WAR-001, the original's 0x463600); `h3-sim`'s
  encounters give machines only with a hero, so nothing of yours moves.
  **The bots' tables: current** (the battle lane regenerated them since, 5b4464f; E06 T6b
  reproduced both byte for byte, 2026-10-26). Open: `inspect` may count a turret's shots
  again (it leaves them out of the strikes).

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
  moved there (T7c; the user, 2026-10-19); AI-Q7 closed at E06 T6e (not kept, off by default).

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
