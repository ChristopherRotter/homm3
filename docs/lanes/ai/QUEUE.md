# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`) closed 2026-10-20 (record: `E05-log.md`; the environment as built: `docs/07-simulation-and-ai.md` §1.1). **E06** (`docs/epics/E06-battle-ai.md`, AI-Q8) closed 2026-10-27 (T1 to T9 in 8 increments; record `E06-log.md`, the design note kept there; the training pipeline as built: `docs/07-simulation-and-ai.md` §1.2): the battle AI v1 is `t7-1`, on the closing engine 0.830, 0.550, 0.530 against Random, Aggressor, Greedy at +0.273, +0.029, +0.010 and the neutral set's value +0.244 (decision 7's targets as revised, met), the system actor 0.628 against it (E25's target). The tasks' history is in `QUEUE-ARCHIVE.md`.

**Next:** no task queued. E25 (Battle AI v2; BACKLOG, a sketch) waits for its refinement with
the user; E06's leftovers for it are the inbox's "For E25", the search's terminal by reward v1
and T8c again first. The adventure env's typed answers wait for the adventure lane's T63.

## Inbox

Sorted by destination at E06's audit (T9a, 2026-10-27; E05's at its T9a, 2026-10-19); the
origin stays in brackets. Moved to `QUEUE-ARCHIVE.md` as done or informational: the notices of
E08 T45, T51b, T52, T58 (folded into T55b's below), E23 T4c and T6's last part, the battle
lane's turret preview (its open point below), the baselines and AI-Q7.

**The adventure env** (yours since E08 T45; when the adventure lane's T63 lands):

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
  actor's defaults: `Gold`, `Attack`, `Leave`, `Decline`, `Close`). **A-23 withdrawn** (E08 T58, 2026-10-27: an event's tile is no stop for the
  original's pathfinder, so `legal_moves` never reads events and your tile masks reveal nothing
  of them; survey `docs/rules/surveys/E08-rewards.md` §15). Design: `docs/epics/E08-design.md`
  §12, §14, §16; ADR-0017's draft point 6, 8.

**For E25** (Battle AI v2, BACKLOG; E06's leftovers, T9a; nothing runs before E25 is refined):

- **The levers E06 left** (its increment 7 checkpoint's candidates): `[network] order` (T6g)
  and `preview` (T6e), no gain on the MLP, off by default, worth a second look with a network
  that reads the battle's structure; potential-based shaping (also against the wandering T6e
  found: the tactics phase's placements and moves cost nothing at γ 1); the 60 M run at T7's mix
  (`configs/e06-t7.toml --set samples=60_000_000`, about 4 hours); the mana-price run `t5-4`
  (`configs/e06-mana.toml`, about 75 minutes; T5's experiment, prepared, never run).
- **The search** (E06 T8, ADR-0025 point 12a): T8c's verdict is **inconclusive** (T9b, the
  user, 2026-10-27): a world's step scores a decided battle ±1 (a single env's reward) where the
  value head learned reward v1, and nodes below the root could take an escape (fixed in T9b).
  First, before any search work: a terminal scored by reward v1 (the single env has none: a C ABI
  addition, or the search in Rust reads `h3_sim::reward`), then T8c measured again. AI-STUB-1
  (`docs/stubs.toml`: the hidden obstacles' number, and T9b's three gaps of a world: the other
  hero's morale and luck modifiers, the magic limits, a commanded war machine without a guess);
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
