# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (record: `E05-log.md`; the environment as built: `docs/07-simulation-and-ai.md` §1.1). **E06** (`docs/epics/E06-battle-ai.md`, AI-Q8) closed 2026-10-27 (T1 to T9 in 8 increments; record `E06-log.md`, the design note kept there; the training pipeline as built: `docs/07-simulation-and-ai.md` §1.2): the battle AI v1 is `t7-1`, on the closing engine 0.830, 0.550, 0.530 against Random, Aggressor, Greedy at +0.273, +0.029, +0.010 and the neutral set's value +0.244 (decision 7's targets as revised, met), the system actor 0.628 against it (E25's target). The tasks' history is in `QUEUE-ARCHIVE.md`.

**E25** (`docs/epics/E25-battle-ai-v2.md`, AI-Q10; record `E25-log.md`), refined
2026-10-27; increments 1 and 2 closed 2026-10-28 and 29 (`E25-archive.md`).
**`e25-scale-1` #3 is the lineage's current best** (the user at T6, 2026-10-29). **Increment 3, the network's architecture** (AI-Q11, the user):
a distillation bench ranks a deeper and a wider MLP, a hex convolution network, a hex
transformer and E06's entity network against the current MLP, then the best go to RL. The
faithful system actor still trains no agent and no neutral battles train (decision 2); each
run's length is agreed with the user (decision 5). All on observation v22 and action v13: no
encoding or ABI change.

- **T7a The network kinds.** *Scope:* (1) `[network] layers` for the MLP (its trunk's layers,
  2 by default, so every saved MLP loads; `hidden` is the width). (2) Two kinds that read the
  field as hexes, in a new module, sharing the entity network's heads (moved into a base class,
  their parameter names kept so E06's files load) and one input: each stack slot's token
  (`symlog` of its features, its creature's embedding) drawn on its hex and, for a two-hex
  stack, on its front hex too (right of its position for the attacker's, left for the
  defender's: `h3_battle::geometry::footprint`; side = own flag against the viewer-is-attacker
  global), added to the hex planes' projection, plus the globals' and heroes' projection on
  every hex; an off-field stack (a turret) keeps its token only. `kind = "hexconv"`: the field
  in axial coordinates (column `x − ⌈y/2⌉ + 5`: 17 × 11 → 22 × 11 cells, 187 real), residual
  blocks of two 3 × 3 convolutions whose kernel's two corners (+1, +1) and (−1, −1) are zero (a
  hex's six neighbours), each block with a global-pooling bias (the real cells' mean and max),
  the residual branch initialised at 0 and the padding cells kept at 0, no batch statistics;
  `width` channels, `layers` blocks. `kind = "hexformer"`: the 187 hexes and a global token,
  pre-norm blocks (E06's `Block`) with a learned bias per head and axial offset (shared by the
  rows); `width`, `layers`, `heads`. Both: the hex vectors are the trunk's per hex, a stack's
  vector its token plus the trunk's at its hex, the context the global and pooled vectors and
  the active stack's. The field's size from the config (`field_width` 17, `field_height` 11,
  the base ruleset's; checked on real battles). `layout.py` gains the columns read (two-hex,
  viewer is attacker), pinned by `h3-sim`'s layout test. (3) Tests as `test_entity.py`'s (the
  logits in the encodings' order, an empty slot unseen, the kernel's neighbours against
  `Field::neighbors`, a run stopped, resumed and laddered per kind), `--time` and parameters per
  kind in the bench's sizes. *Depends on:* T6 (done 2026-10-29). *Status:* next.
- **T7b The distillation bench.** *Scope:* `python -m h3env.train.distill --teacher <best.pt>
  --config configs/e25-distill.toml --out runs/<bench>`: the teacher plays itself on the training
  distribution (its config's kinds and tactics bound, the sampled policy, a seed apart from its
  run's) in rollouts as PPO's (`rollout` steps of `envs`); every student trains on the same
  rollouts, `epochs` passes in minibatches: the KL from the teacher's entry head over the legal
  entries, for a cast row its spell, target and second heads along the teacher's choices, and
  the value against the teacher's λ-returns (PPO's targets, shaping as the teacher trained);
  Adam, one learning-rate schedule for all (a student's own `lr` allowed). A held-out set
  (rollouts on other seeds, collected once) scores each student every so many samples: the KL
  per head, the entry's agreement with the teacher's most likely entry per category of the
  teacher's choice (move, strike, shot, cast, wait or defend, tactics), the value's mean squared
  error and explained variance, the teacher's own beside them, the time per sample; a JSON
  report and a table; the students' weights saved (T7d may start from them). The config: the
  students as named `[network]` tables (MLP 2 × 512, 2 × 1 024, 4 × 512, `hexconv`,
  `hexformer`, `entity`), the samples per student, the held-out size. A CPU smoke in the
  sandbox and tests. *Depends on:* T7a. *Status:* waiting.
- **T7c The bench run.** *Scope:* on the user's machine (the GPU; its length agreed with the
  user); the report committed (`python/records/distill/`) and read in `E25-log.md`: which
  kinds fit the teacher best per sample and per second, and where the entity network fails
  (the move head or not). *Depends on:* T7b; the user. *Status:* waiting.
- **T7d The A/B.** *Scope:* the best one or two kinds at 20 M on `e25-scale.toml`'s settings
  (from scratch, or from the bench's weights: imitation of the lineage's own network, not of
  the system actor or a bot), against `e25-scale-1`; the user's machine. *Depends on:* T7c.
  *Status:* waiting.
- **T8 Increment 3's checkpoint.** *Scope:* T7c's and T7d's reads put to the user; increment 4
  chosen. *Depends on:* T7d. *Status:* waiting.

## Inbox

Sorted by destination at E06's audit (T9a, 2026-10-27; E05's at its T9a, 2026-10-19); the
origin stays in brackets. Moved to `QUEUE-ARCHIVE.md` as done or informational: the notices of
E08 T45, T51b, T52, T58 (folded into T55b's below), T70, E23 T4c and T6's last part, the battle
lane's turret preview (its open point below), the baselines and AI-Q7.

**The adventure env** (yours since E08 T45):

- 2026-10-28 (the adventure lane, E08 T66b, `adventure/e08-t66b`; decision 172; **for you, no
  action yet**): increment 8 (boats and the sea) changes the adventure observation: the views gain
  the empty boats on the viewer's explored tiles (tile, kind, facing), each seen hero's boat
  (`HeroView::boat`) and an own hero's day's maximum (`OwnHeroView::max_movement`); the action
  encoding keeps v5 (a boarding or a landing is a tile among `legal_moves`). The user chose that
  the adventure lane makes your observation encoding's change `cross-lane` in its task T74, with
  the user's consent, as T63a's; the notice of that pull request will name the version. Design:
  `docs/epics/E08-design.md` §12, §15; ADR-0017 point 8 (amended by E08 T70, `shared/`).
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
