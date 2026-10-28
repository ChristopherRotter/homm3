# AI lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/ai/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Completed tasks (the "Tasks" text as it stood on 2026-10-24)

**E25 increment 1, T3** (moved 2026-10-28, done: `E25-log.md` T3):

- **T3 Increment 1's checkpoint.** *Scope:* T1c's read (`E25-log.md` T1c: no gain, the guess
  costs nothing measurable, the network's evaluation limits the search) and T2b's put to the
  user; increment 2 chosen among the epic's candidates and refined. *Depends on:* T2b (done
  2026-10-28: shaping +0.04 against Aggressor, Greedy and the system actor, a tail of long
  battles; `E25-log.md` T2b). *Status:* next (the user).

**E25 increment 1, T2b** (moved 2026-10-28, done: `E25-log.md` T2b):

- **T2b The A/B runs.** *Scope:* on the user's machine, the control (`configs/e25-control.toml`:
  `t7-1`'s settings with the player kinds alone, no neutral battles, the user at T2a) and the
  same with shaping (`configs/e25-shaping.toml`), their length agreed when prepared (20 M
  samples in the files, `t7-1`'s); the records committed, the ladders (the system actor and the
  neutral set included) against `t7-1`'s. *Acceptance:* both records committed and read in the
  log. *Depends on:* T2a (done 2026-10-28); the user. *Status:* waiting (the user's machine).

**E06 increment 8, T9 the closure** (moved 2026-10-27 at T9d, E06 closed):

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (record: `E05-log.md`; the environment as built: `docs/07-simulation-and-ai.md` §1.1). **E06** (`docs/epics/E06-battle-ai.md`, AI-Q8; design `E06-design.md`, record `E06-log.md`): increments 1 to 7 done by 2026-10-27; the battle AI v1 is `t7-1` (20 M samples, a third neutral): 0.828, 0.551, 0.528 against Random, Aggressor, Greedy at +0.273, +0.029, +0.010, the neutral set's value +0.245 (decision 7's targets as revised 2026-10-27, met), the system actor 0.374 (E25's target). The tasks' history is in `QUEUE-ARCHIVE.md`.

**Increment 8, T9 the closure** (refined 2026-10-27; the epic file's task list, E05's T9 as the
pattern). T9a, the audit, done 2026-10-27 (`E06-log.md`): exit criteria 1 to 5 met, E06's
leftovers sorted into the inbox below.

T9b, the review, done 2026-10-27 (`E06-log.md`): 14 findings verified, the fixable ones
fixed; the user: T8c inconclusive (E25 measures again), the world's gaps to AI-STUB-1.

- **T9c The verifies** (the user's machine, next; exit criterion 6 as amended 2026-10-27):
  `cargo xtask verify --with-original-data` and `uv run --group train pytest` on Windows (Linux:
  CI's full verify and the sandbox's Python tests, T9b), and `t7-1`'s final ladder again on
  today's engine (T9a: it played the system actor's `POLICY_VERSION` 18, the bots' tables 19),
  one PowerShell script once this pull request merged; the user uploads its summary, logs and the
  two ladder files.
- **T9d The closure:** `shared/e06-closure` first (BACKLOG E06 done and its E25 row and note:
  T8c inconclusive; the env doc's training pipeline as built; ADR-0026's detail line; ADR-0025
  point 12a: "one known gap" becomes AI-STUB-1's gaps, and the C ABI refuses a world of the side
  not to move), the design note at closure, the epic file done.

**E06 increment 7, at its checkpoint** (moved 2026-10-27, the user: E06 closes, T9 next):

**Increment 6** closed 2026-10-26 (T6b to T6e; its task list in `E06-archive.md`): both levers
negative, the system actor as an opponent (T6c, `t6b-1`) and the kill preview in the network
(T6e, `t6d-1`: the targeting better, the ladder unchanged; AI-Q7 closed, the preview off by
default).

**Increment 7, T8 the search** (refined 2026-10-26, the user: the plateau taken as shown; design
§14): T8a (the design, ADR-0025 point 12's amendment as a draft) done 2026-10-26; the user
accepted the draft (`docs/lanes/ai/adr-drafts/0025-amendment-search.md`) the same day, its move
into ADR-0025 due with the next `shared/` change. T6f (the turn order as an input, observation
v22) and T8b (the plumbing, C ABI v34) done 2026-10-27 (#493); T6g (the user's run `order-1`)
and T8c (the search measured in the sandbox with `order-1`'s best, its σ fixed) done 2026-10-27,
both negative (E06-log). **Next: the user's checkpoint** (E06-log T8c, "Put to the user"): close
increment 7 and decide E06's end (the agent's recommendation: decision 7's targets revised to the
evidence, then T9 with `t7-1` as the battle AI v1), or T8d with the tree in Rust, or the
true-world search as a diagnostic; and AI-STUB-1 (a world's gap against ADR-0025 point 12a),
which matters only for T8d.

- **Optional, the user's machine** (T8c's rows the sandbox left out): `uv run --group train
  python -m h3env.search --report --best runs/t7-1/best.pt --device cuda --opponent greedy
  --simulations 64 --json runs/t8c-greedy-64.json`; with `--opponent system` for the report
  set's whole 500 per kind.

- **Candidates kept** (not now): `[network] order` (T6g: no gain, off by default);
  potential-based shaping (also against the wandering T6e found: the tactics phase's placements
  and moves cost nothing at γ 1); the 60 M run at T7's mix
  (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours); the mana run `t5-4`
  (`configs/e06-mana.toml`, about 75 minutes).
- Later (design §10): T5's rest (throughput, the observations in place), T9.

**E06 increment 6, T6d's stage** (moved 2026-10-26 at T6e's checkpoint):

**Increment 6** (refined 2026-10-26, the user: the system actor as a training opponent in the
player kinds; every side stays human): T6b and its run T6c done 2026-10-26, negative (`t6b-1`
at `t7-1`'s ladder). The user's next lever: AI-Q7 (b) part 2, T6d (`[network] preview`,
`configs/e06-t6d.toml`) done 2026-10-26. **Next: T6e, the A/B run on the user's machine**
(`uv run --group train python -m h3env.train --config configs/e06-t6d.toml --out runs/t6d-1`,
20 M samples; the user uploads `record.json`, `best.pt`), then `python -m h3env.train.inspect`
on its best and the checkpoint against `t7-1` (AI-Q7: keep the preview only if it measurably
helps).

- **Candidates at T6e's checkpoint:** potential-based shaping; runs: 60 M at T7's mix
  (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours: whether PPO plateaus, T8's
  trigger: three 20 M runs, `t5-3`, `t7-1`, `t6b-1`, end at the same ladder); the mana run
  `t5-4` (`configs/e06-mana.toml`, about 75 minutes). Not the casts' exploration or the mana
  price (T6a), nor more of the system actor as an opponent (T6c).
- Later (design §10): T5's rest (throughput, the observations in place), T6, T8, T9.

**E06 increment 6, T6b's stage** (moved 2026-10-26 at T6c's checkpoint):

**Increment 6** (refined 2026-10-26, the user: the system actor as a training opponent in the
player kinds, decision 9 amended; every side stays human, the let-off question answered): T6b
(`[league] system_share`, `configs/e06-t6b.toml`) done 2026-10-26. **Next: T6c, the A/B run on
the user's machine** (`uv run --group train python -m h3env.train --config configs/e06-t6b.toml
--out runs/t6b-1`, 20 M samples; the user uploads `record.json`, `metrics.jsonl`, `best.pt`),
then `python -m h3env.train.inspect` on its best and the checkpoint against `t7-1`.

- **Still candidates at T6c's checkpoint:** AI-Q7 (b) part 2 (the network reads
  `batch.preview` beside the entry mask, an opt-in `[network]` key, the rollout keeps it; an
  A/B config; unblocked since the battle lane's #458 previews a turret's shot as it is made);
  potential-based shaping. Not the casts' exploration or the mana price (T6a). Runs: 60 M at
  T7's mix (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours: whether PPO
  plateaus, T8's trigger); the mana run `t5-4` (`configs/e06-mana.toml`, about 75 minutes).

**E06 increment 6's choices** (moved 2026-10-26, the user chose the system actor as an opponent):

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

**Moved at E06's audit** (T9a, 2026-10-27; verbatim, the inbox's other entries sorted by
destination in `QUEUE.md`): done or informational. T58's withdrawal is folded into T55b's entry,
which stays; the battle lane's turret note's open point (`inspect`) went to "For E25"; the
baselines' stronger bot is the system actor in the ladder (E06 decision 12), AI-Q7 closed at T6e.

- 2026-10-27 (the adventure lane, E08 T58, `adventure/e08-t58`; **for you**, correcting the
  T55b entry below): **A-23 is withdrawn.** Batch E7's row E7-23 showed the original's path
  crossing a hidden event, and the re-read found an event's tile no stop for its pathfinder. So
  `legal_moves` never reads events in any profile, and your tile masks reveal nothing of them;
  a step into a firing event ends the move there (the move's outcome). Survey
  `docs/rules/surveys/E08-rewards.md` §15; design §12.

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

- **Baselines and AI-Q7:** Greedy value beats Aggressor only 0.519 on the held-out episodes: the
  preview of its own kills buys little without spells or a plan; a stronger baseline (casts, a
  one-ply look at the opponent's reply) is E06's, and AI-Q7 (whether the agents get the preview)
  moved there (T7c; the user, 2026-10-19); AI-Q7 closed at E06 T6e (not kept, off by default).

- 2026-10-27 (the adventure lane, E08 T61b, `adventure/e08-t61b`, `cross-lane` with the user's
  consent; informational, nothing to act on unless a literal of yours is new): `GameSetup` gained
  `banned_spells` and `banned_artifacts` (GAM-SET-003, map import v13); your one full literal,
  `crates/h3-cli/src/ai/census.rs`'s, got `banned_spells: Vec::new()` and
  `banned_artifacts: Vec::new()`. The adventure state hash is v14
  (the build's draws on increment 7's objects, `MapObject::resolved`, and the visits' block,
  `AdventureState::visits`), so a saved state hash of yours moves; no env encoding changed. The
  shipped maps that import whole are 8 now (`Myth and Legend`, `Pandora's Box `, `Unholy Quest`
  join the 5), should a census of yours list them.
- 2026-10-27 (the adventure lane, E08 T61a, `adventure/e08-t61a`; for the next census, `h3 ai
  census`): `ObjectSetup::guards` is no longer a pile's only. Since map import v12 a placed or
  random artifact, a Spell Scroll, a Pandora's Box and an event carry the guards of their message
  records too, so `census.rs`'s `objects()` counts every guarded object as a guarded pile; filter
  by the type's kind (`ObjectKind::Pickup`) to keep `guarded_piles` a pile's count. The shipped
  `census-v1.toml` is unaffected until a re-run. Increment 7's 31 classes are typed now
  (`ObjectKind::Chest`, `Artifact`, …); the census's `_ => {}` arm ignores them.

- 2026-10-27 (this lane, E06 T6f and T8b; for the next `shared/` change):
  `docs/07-simulation-and-ai.md` (shared) should name observation v22 (the turn order block
  after v21's layout, `[network] order`) and C ABI v34's worlds (`h3_batch_determinize`,
  `h3_env_reseed`, a search agent's under ADR-0025 point 12a) under "The batch" and
  "Information"; ADR-0025 point 12a's text with the draft's "As built" (AI-STUB-1).
  Done 2026-10-27 in `shared/e06-search-docs` (E06 increment 7's checkpoint).

- 2026-10-26 (this lane, E06 T8a; for the next `shared/` change): **ADR-0025 point 12a** from
  the draft the user accepted (`docs/lanes/ai/adr-drafts/0025-amendment-search.md`: a search
  agent's worlds), then the draft deleted; T8b builds on it either way.
  Done 2026-10-27 in `shared/e06-search-docs` (E06 increment 7's checkpoint).

- 2026-10-23 (this lane, E06 T5; for the next `shared/` change): `docs/07-simulation-and-ai.md`
  (shared) should name the batch's optional kill preview (C ABI v33: the acting side's previewed
  kills of each legal attack and shot entry, the engine's BAT-DMG-011 preview, what the original
  shows a player) under "The batch" and "Information".
  Done 2026-10-27 in `shared/e06-search-docs` (E06 increment 7's checkpoint).

- 2026-10-27 (the adventure lane, E08 T60a, `adventure/e08-t60a`, `cross-lane` with the user's
  consent; for your information, nothing to do): **`HeroDetails` gained `bonuses`**
  (`h3_heroes::BattleBonuses`, HER-ADV-008: morale and luck until the hero's next battle's end, and
  once-marks). `h3-sim`'s encounter (`encounter/hero.rs`) got `bonuses: Default::default()`; a new
  literal needs it too. Game state hash v15 (the bonuses in a hero's details): the twelve adventure
  replays re-recorded, their commands unchanged. `HeroChange` gained `GainPrimary`, `RaiseSkill`,
  `Bonus` and `ClearBonuses`: a `GameEvent::HeroChanged` may carry them from T62 and T63.

- 2026-10-27 (the adventure lane, E08 T59, `shared/e08-t59`, `cross-lane`; for your information,
  nothing to do): core ruleset hash v49 (the mana ceiling, two adventure flags; no battle rule
  moved): `bots-report-v1.json` and `bots-neutral-v1.json` regenerated in that pull request
  (`h3 ai eval --set report-v1|neutral-v1 --json`); the run records keep the hash they ran on.
  ADR-0017 point 8 now says the env's mapping onto the answers typed by decision (E08 T63,
  `cross-lane`) and that a University's purchase head is your call.

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
  `System` side in training is open (put to the user at T6a's checkpoint). **Answered 2026-10-26** (the user, E06 increment 6's refinement): every side stays human, the system actor's too (ADR-0025 point 6; E06 design §12).

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

