# E05 — Battle environment: design note (T1)

Working detail behind ADR-0025 (E05 T1, accepted 2026-10-18 with the user's review: AI-Q2,
AI-Q3). A handover, not binding: each task that builds a section deletes its detail here
(`docs/adr/README.md`, "Keep ADRs short"). Sections follow T1's points (a) to (g).

## 1. The batch API (a) — built (T3b, T3c, T3d)

ADR-0025 points 1 to 7 as code: `h3_sim::batch` (`BattleBatch`, `Scenario`, `Seat`, `Policy`,
`EnvReport`, `Outputs`, `episode_seed`, `draw_scenario`; the threads in `batch::pool`) and C ABI
v27 in `h3-ffi`'s `src/batch.rs`, declared in `include/h3.h` (T1's sketch with its names, plus
`H3Seats`, `H3_SEAT_*`, `H3_CURRENCY_*` and `H3_ERR_NOT_READY`; the encounter source came with
T5b2 as `h3_batch_add_encounters`, C ABI v28, §4.1). Decisions of the build beyond the ADR (the
agent's, provisional):

- **Outputs:** the big arrays (observations, entry masks, final observations) are the caller's,
  written in place by the workers; the small per-env values come back as one `EnvReport` per
  env, which the C layer converts into `H3BatchBuffers`' arrays after the call. A null array
  skips its output.
- **Skipped episodes:** an episode that ends before an external seat could act holds no decision
  and is skipped; after 64 in a row (`MAX_SKIPPED_EPISODES`) the env gives up (no side to move,
  status `H3_ERR_EPISODE_OVER`) until the next reset. `reset` restarts every env's seed stream.
- **Threads (T3c):** T std threads (at most N), env i in shard `i * T / N`, parked on a condition
  variable between calls; reset, step and each head call split the envs and the caller's rows
  into contiguous shards and return when every worker is done. The workers write into the
  caller's arrays through one `unsafe` lifetime erasure (`Pool::run`), sound because the call
  blocks until every worker finished; a worker's panic is caught and resumed in the caller after
  that. T = 0 runs in the caller's thread.
- **Head calls (2026-10-19):** each env keeps the entry mask of the last call (the single env's
  without the escapes: E05 T9b). A non-cast entry's later heads ("0 only" while it is legal) are
  answered from it; only casts ask the env. The answers equal the single env's but for the
  escapes, refused in every head (a test over every legal entry, the escapes, illegal ones and
  non-zero earlier heads); the three calls cost 0.11–0.15 ms for 64 envs on `full` instead of 1.4 ms.
- **Scenarios (T3d):** parsed and compiled in `h3-ffi` (`src/scenario.rs`, as T2's); one ruleset
  per packs and profile per batch, reused through the first scenario that compiled it
  (`Scenario::ruleset`), so `h3-ffi` needs no new dependency.
- **Errors:** a refused call changes nothing: `H3_ERR_NULL`, `H3_ERR_ARGUMENT` (with a message for
  create and add) or `H3_ERR_NOT_READY` (no scenario yet, not reset); a step returns
  `H3_ERR_REJECTED` when an env refused its action, its `status` saying why.
- **Properties:** a batch steps as N single envs for random N, T (0 to 7), scenarios, seats, step
  limits, currencies and prices (`h3-sim`'s `batch/tests.rs`); the ABI batch equals the Rust
  batch output by output (`h3-ffi`'s `batch/tests.rs`).

## 2. Seats (b)

- A scenario names a seat per side: `External` (the caller acts: every acting seat the batch
  reports is external) or `Internal(policy)`, played inside the step. At least one seat of each
  scenario is external (checked when the scenario is added or sampled).
- Internal policies are a closed enum in `h3-sim`, so no trait objects and no allocation per move:
  `Random` (uniform over legal commands, the casts' heads included; tests and benchmarks only),
  `System` (E23's policy of the original's computer: E23 provides the function
  `fn(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` in the crate it chooses,
  shared with the adventure's neutral seat in `h3-game`; `h3-sim` calls it), and the baseline bots
  of T7. They choose a `Command`, not an encoded action: an internal seat needs no encoding.
- Internal seats are engine code, not agents: `System` reads what E23's reading says the
  original's computer reads; the bots (T7) read only their side's `Observation` (fair baselines).
- Seats from the encounter distribution (T5): a neutral side (a wandering monster, guards, a
  neutral town's garrison) is `Internal(System)`; a player's side is external. Until E23 the
  neutral share samples `Internal(Random)` and is marked as not for training (BACKLOG order).
- **A level playing field** (the user, AI-Q3): the batch builds every episode's setup with
  `human = [true, true]`, whatever the scenario or the sampler says; `human = false` remains for
  compatibility with the original's computer player (ADR-0023 point 7). `difficulty` stays
  scenario data. This also
  answers BAT-Q33's open point for E05: an agent's side counts as a human player's.
- **No escapes** (the user, AI-Q2): the batch's masks never allow Retreat (entry 1392) or
  Surrender (1393), for external and internal seats alike, and `step` refuses them as an action
  that does not decode. The encoding keeps the entries; a batch option (`escapes`, default off)
  turns them back on when the adventure's view of a battle comes (E13). `BattleEnv` and the
  single-env C functions keep them (tools, replays and the battle lane's benchmarks use them).

## 3. Reward v1 (c) — built (T4)

`h3_sim::reward` (its module documentation states the formula, ADR-0025 point 8):
`r_s = (K_o − K_s − p · (m_s − m_o)) / S`, the currency `ai_value` (default) or the creatures'
cost in the ruleset's money (`Currency::Gold`: the resource surrenders are paid in, gold in the
original's packs, coin in `synthetic`), p 10 by default (`DEFAULT_MANA_PRICE`, provisional, tuned
in E06). K counts the army and raised stacks (`BattleOutcome::survivors`) and the war machines;
created stacks count nothing; a hero that gains mana in the battle is credited at the same price.
Integer sums in `i128` (no overflow for any setup), one division in `f64`, reported as `f32`:
zero-sum exactly. Terminal: only the call that decides a battle carries it, a truncation gets 0
(with both final observations). T1's six worked examples are tests in
`crates/h3-sim/src/reward/tests.rs`, one per example. The Python side accumulates each seat's
rewards between its own moves (§5).

## 4. Scenarios and the encounter distribution (d)

### 4.1 What the ABI loads (T2)

- **A scenario is the battle lane's scenario TOML** (`h3-pack-battle`'s `parse_scenario`, the
  format of `packs/synthetic/scenarios/*.toml` and `h3 run`): packs by id, profile, armies,
  heroes (as `BattleHero` values), terrain, battlefield (special grounds, the deck, the boats),
  `anti_magic` (a garrison), `siege`, war machines, formations, funds, `difficulty`, `human`,
  `random_obstacles`. It already describes every battle the engine plays; E05 changes nothing in
  it.
- **Packs:** the five built-in packs (`synthetic`, `synthetic-ext`, `base`, `ab`, `sod`, compiled
  into the library as today) by id; a ruleset is compiled once per distinct (packs, profile) of a
  batch and shared (`Arc`). Packs from files or texts stay out until a mod needs them.
- T2's single-env entry point: `H3Env *h3_env_create_scenario(const char *toml, size_t len,
  uint64_t seed, uint32_t max_steps, char *error, size_t error_len)` (C ABI v26): null on error
  with the parse or setup error's message in `error`; random obstacles as the scenario says (as
  `h3_env_create_synthetic`). Tests per place: a field, a special battlefield, a garrison, a town
  (closes F-robust-2 and the E01 inbox item).
- **Built (T2):** the scenario's text is parsed and its packs compiled in `h3-ffi` (its module
  `scenario`): `arch.toml` lets `h3-sim` use `h3-pack-battle` only in its tests, so `h3-sim`
  takes a resolved `BattleSetup` and the C ABI layer turns texts into setups (T3's
  `h3_batch_add_scenario` the same way). On error `error` gets the stage and the message
  (`scenario:`, `unknown pack`, `packs:`, `setup:`), on success it is left untouched;
  `h3_env_create_synthetic` takes the same path with its built-in texts. Each call compiles its
  packs: about 18 ms for `base`, `ab` and `sod` (2.6 ms for the synthetic skirmish) in a release
  build on the cloud sandbox, about ten times that in a debug build; the batch's shared rulesets
  (above) avoid it per env.
- **Episode sources of a batch:** a weighted list of entries, each reset drawing one by weight:
  (i) a fixed scenario (`h3_batch_add_scenario`: text, integer weight, seats), or (ii) the
  encounter distribution (**built, T5b2**): `Scenario::encounters(sampler, weight)` in Rust,
  `h3_batch_add_encounters(batch, weight, kinds, error, error_len)` in C ABI v28 (`kinds` null for
  the distribution's weights, else six in `H3_ENCOUNTER_*` order: a curriculum; on the original
  packs, `base:faithful`, sharing their ruleset with the batch's scenarios on them; the adventure
  part compiled per call). `Scenario::episode(seed, max_steps)` gives an episode's env, seats and
  kind: the sampler's encounter of the episode's seed, drawn from stream `ENCOUNTER` (a kernel
  stream, #254) so that it is independent of the entry's draw from `SETUP`; the neutral kinds'
  defender is played inside the call. Each env reports its episode's kind (`EnvReport.encounter`,
  `H3BatchBuffers.encounter`: `H3_ENCOUNTER_*`, -1 for a fixed scenario). Cost in a release build
  on the cloud sandbox: a sample 50 µs at the default weights (26 µs at A1's shares), an
  episode's start with its env 160 µs, about 2 % of an episode of some 300 steps.

### 4.2 The encounter distribution v1 (built, T5b1)

- **Files:** `crates/h3-sim/data/encounters-v1.toml` (the assumptions of §4.4, each table with
  its `why`) names the census it reads (`census-v1.toml`, §4.3); both are compiled into `h3-sim`
  (`h3_sim::encounter::{ENCOUNTERS_V1, CENSUS_V1}`) and checked by `EncounterTable::parse` (each
  table's length, non-decreasing quantiles, drawable weights, no unknown field). Integers only
  (the sim layer has no float arithmetic, ADR-0003): shares in ‰, a continuous distribution as a
  quantile table of 21 points (p = 0.001, 0.05, …, 0.95, 0.999; a draw takes one of the 20 bins
  and a point in it evenly, linear between), computed from the stated normal or log-normal when
  the file was written.
- **`Sampler`** (`h3_sim::encounter`): built on the battle and adventure parts of one core
  (`GameRuleset`'s check); `sample(seed)` draws every choice from stream `ENCOUNTER` of the seed and
  returns an `Encounter`: kind, week, a `BattleSetup` (obstacles left to the episode,
  `random_obstacles`; both sides human), the seats (A12) and the `Facts` drawn (place, level,
  count, redraws, ratio, k, factions, hero levels, values, fort). Its default weights are the
  file's `weight` (the neutral kinds 0); `with_shares` draws at A1's shares, `with_weights` by a
  curriculum's.
- **The engine's rules it uses**, never copies: ADV-MON-003 (`AdventureRuleset::random_monsters`),
  ADV-MON-004 (the adventure-map range), ADV-MON-009 (`h3_adventure::split`), and a hero's battle
  values as GAM-BAT-001 derives them (`derive_battle_hero`, `max_mana`, `war_machines`); the
  Catapult is the battle's own (BAT-SIE-009). **Approximations**, labelled in the code: a hero's
  level-ups (the class's `primary_gain` and `skill_weights`), spells and artifacts; an army one
  stack per built level, its value split by growth value, what a level's share cannot buy carried
  to the level below; guards a stack of A3 divided evenly.
- **The check** (`encounter::tests`): 30 000 encounters at A1's shares against their weights:
  kinds, weeks per kind, the neutral levels from week 7 (no stack is drawn again there) and the
  ramp's bounds, places, the bins of the ratio, k and the attacker ratio, factions, hero levels by
  week, forts. 1 500 samples are valid battles (`Scenario::new`), every 50th plays out; every
  sampled hero passes `h3_heroes::validate`.
- **What it shows** (20 000 samples): in week 1 A4's cap draws 16 % of the neutral stacks again
  (level 4 falls from 10 % to 4 % of them), in week 2 1 %, from week 4 none; under random play the
  attacker wins 98 % of the neutral battles, 80 % of the hero battles, 63 % of the sieges a hero
  defends.
- Not drawn yet: the difficulty (A11: Normal), two boats (A10), creature banks (A1). T5b2 made the
  sampler a batch's episode source (§4.1).

### 4.3 The census v1 (T5a, built)

- **`h3 ai census <maps folder or files> [--out <file>]`** (`h3-cli`'s `src/ai/census.rs`) reads
  each `.h3m` through `h3-map` (the raw model and the static layer's `import_map`, which every
  shipped map passes) into a `GameState` of one player on the original packs (`base:faithful`)
  and adds up aggregate numbers only (ADR-0009): the maps by version, size and levels; the tiles
  by passage (land, water, impassable; underground, coast, *open*: no object blocks it or is
  visited from it); the monsters by level (fixed or drawn creature) and how many fix their count;
  guarded piles, special grounds, placed and random heroes, prisons, garrisons (anti-magic apart:
  subclass 1), creature banks, the four banks with their own class, boats; the towns by faction
  (a random town apart) and starting fort (the standard set's flag, or the custom set's fort,
  citadel or castle bit); and **every battle place** with its open land tiles, monsters and
  guarded piles. A place is the engine's own `AdventureState::battle_ground` of the defender's
  tile (the monster's or the pile's visited tile; GAM-BAT-001, ADV-GRD-001): underground the
  subterranean field, a coast tile the sand and the shore, a special ground on top; the census
  and the adventure map cannot disagree. Every placement and tile counts once, so a larger map
  weighs more (it holds more battles). A map that does not read or import is skipped and named
  on standard error.
- **`crates/h3-sim/data/census-v1.toml`:** the 159 maps of the user's installation
  (`original-data/install/Maps`, 2026-10-18; the tutorial left out, not an `.h3m`). The earlier
  count of 164 (E08's survey) included maps that are not the game's (`experiment`); 160 in the
  epic counted the tutorial. 25 s in a debug build.
- **What it says** (derived; T5b reads the file, these are for the review): 48 RoE, 55 AB, 56 SoD
  maps; 31 S, 71 M, 38 L, 19 XL; 103 with an underground. 13 863 monsters (87 a map): level 1
  17.7 %, 2 26.1 %, 3 25.5 %, 4 16.0 %, 5 8.3 %, 6 3.5 %, 7 2.4 %, any 0.5 %; 1 483 name their
  creature, 740 their count. Their places: grass 24.0 %, dirt 18.8 %, subterranean 17.0 %, sand
  11.5 % (the shore included), rough 9.4 %, snow 7.6 %, swamp 5.8 %, lava 5.4 %, water 0.4 % (55
  monsters a hero meets from a boat); the plain field 88.2 %, the shore 7.5 %, a special ground
  4.3 % (of open land 2.8 %: map makers put monsters on grounds). 335 457 open land tiles, their
  places within a point of the monsters'. 1 465 towns (9 a map; 875 random; Castle the most
  named, 118, Necropolis the fewest, 32); at the start 20.9 % without a fort, 74.3 % a fort,
  1.8 % a citadel, 3.1 % a castle. 36 guarded piles, 238 garrisons (45 anti-magic), 2 137
  creature banks (13 a map), 371 heroes placed, random or in prisons, 118 boats, 1 857 special
  grounds.
- **Twins:** 35 maps share their terrain with an earlier one (the allied versions). Leaving the
  twins out moves no terrain or level share by more than 1.6 points, so every shipped map counts
  once (proposed, A9 below).

### 4.4 The assumptions v1 (decided 2026-10-19, AI-Q4)

What the maps cannot say, for the encounter distribution's `why` fields (`encounters-v1.toml`
holds every number, written from the census and these in T5b1). Every number is provisional; the
adventure's logged battles replace them (decision 7). Proposed at T5a; **the user (2026-10-19):**
A3 keeps in mind that the maps' stacks differ by level from the start (a horde of Pikemen in
week 1, a few Black Dragons) and that no hero faces level 7 early; A4's ratio is a distribution
around 1.8 with a standard deviation of 0.5, at least 0.5, for occasional tough battles; A5 as the
review changed it; A12's neutral kinds fully built but weighted 0 until E23 (after E05); the rest
as proposed. How T5b1 pins down what a row leaves open is marked *(T5b1)*.

| # | Dimension | Decided | Why |
|---|---|---|---|
| A1 | Encounter kind (of battles fought) | wandering 0.78, guards 0.02, hero 0.10, sieges 0.10: a hero defends 0.05, an owned town's garrison 0.025, a neutral town's 0.025 *(T5b1: A8's split as kinds, so A12 can weigh the neutral one 0)* | 87 monsters a map against 9 towns and a few heroes: a player clears most monsters on the way; each neutral or enemy town taken is a siege. Guards are the 274 piles and garrisons against 13 863 monsters (2 %). Creature banks (13 a map) wait for their battle layouts (the adventure lane's queue) |
| A2 | Game week (1 to 16), per kind | wandering and guards ∝ 0.8^(w−1); hero rising to week 4, flat to 8, falling to 16 (1/9 of the peak); sieges ∝ 0.9^(w−1), a neutral town ∝ 0.7^(w−1) *(T5b1: tabulated in ‰)* | neutrals fall early (median week 3), enemy heroes meet mid-game, towns are taken throughout, neutral ones early |
| A3 | A neutral stack | level: the census's (fixed and random; "any" spread evenly) × a ramp by week: 1–3 always, 4 half in week 1, 5 half in week 2, 6 and 7 from weeks 3 and 4 in quarters (full from weeks 6 and 7); the creature drawn among the level's (ADV-MON-003), the count from its adventure range (ADV-MON-004), grown each week by a tenth (integer) while below 4 000 (VCMI 1.7.5 `CGCreature::newTurn`: `weeklyGrowthPercent` 10, `weeklyGrowthCap` 4 000; *T9b: VCMI grows the stack's power, a thousand per creature, so that 5 creatures are 9 after seven weeks where the count's tenth leaves them 5; the neutral kinds weigh 0 until E23, which decides*); split by the original's rule against the hero's army (`h3_adventure::split`, ADV-MON-009) | the map's own levels and the game's own draws; the adventure ranges already make low levels hordes and high ones few; strong stacks are fought later; growth makes late neutrals larger |
| A4 | The army facing a neutral | value = the stack's × r, r normal (mean 1.8, standard deviation 0.5), at least 0.5; a stack whose army would exceed 2 × A5's value of the week is drawn again (r kept) | players attack what they beat, now and then a tough fight; the split's own table expects ratios 0.5 to 2 |
| A5 | A hero's army (hero and siege kinds) | value = k × the sum over the weeks u ≤ w of the faction's growth value (growth × `ai_value`) of the levels A6 makes available in week u; k log-normal (median 1.0, ×/÷ 1.6); the attacker's over the defender's log-normal (median 1.3, ×/÷ 1.5). Castle's median: 6 500 in week 1, 43 000 in week 4, 111 000 in week 8, 246 000 in week 16 | one main army carries a kingdom's production (a castle's double growth and more towns against its losses); consistent with A6, where G × w × 0.7 gave week 1 1.8 weeks of levels 1–4 |
| A6 | An army's creatures | the hero's faction; levels 1–4 from week 1, 5 from week 2, 6 from week 3, 7 from week 4; value split by each level's growth value; one stack a level, upgraded with the share (w − 1) / 6, at most 1 | a usual build order |
| A7 | A hero | faction by the census's towns (named; random ones spread evenly over the nine factions *(T5b1: else no Conflux)*), might or magic class 1 : 1, the hero type even among the class's; level by week 4, 7, 9, 11 (weeks 1–4), then +1 a week to 15 (week 8), then +0.5 a week, ± 2. *(T5b1: `h3-heroes` has no level-up rule; an approximation, not a rule:)* each level-up adds a primary skill by the class's chances (`primary_gain`, levels 2–9 and 10+) and raises a known skill or learns one by the class's `skill_weights` (at most 8, Expert at most); a spell book; per spell level up to the guild (levels 1, 2, 3, 3, 4, 4, then 5 from week 7; at most the faction's: 4 Castle, 3 Stronghold and Fortress, else 5, VCMI 1.7.5 `mageGuild`) and its learn level, 6 − level spells (VCMI `spellsAtLevel`, no Library) drawn among the level's hero spells; level / 2 artifacts drawn by class by level (treasure from 1, minor from 6, major from 11, a relic from 16; no combinations, scrolls or war machines), worn where a slot is free; Ballista, Ammo Cart and First Aid Tent each at 0.5; full mana | a human main hero's usual pace |
| A8 | A siege | the faction by the census's towns (as A7); the fort from the census in week 1, then citadel and castle more likely each week (castle at most 0.7; a neutral town keeps the census's); a hero defends the kind of A1, else the garrison: one week's growth (A6's levels) *(T5b1: the moat damaging from a Citadel (SoD rules), structures 6 + 2 (w − 1) at most 30 (a neutral town 6), morale +1 at an owned town (a Tavern), no Escape Tunnel; the terrain the faction's native one)* | neutral towns early, owned towns later |
| A9 | Twins | every shipped map once | §4.3 |
| A10 | Places | neutral stacks and guards: the census's monster places (the 55 on water: the deck, the hero in a boat); hero battles: open land; sieges: the town's own field; two boats left out | where battles stand on the maps |
| A11 | Difficulty | left out (normal) | the batch plays both sides as human players' (ADR-0025 point 6); the difficulty shapes only the computer's automatic choices (ADR-0023 point 7) |
| A12 | Seats | neutral kinds (wandering, guards, a neutral town) the system actor's (internal), the others external on both sides; the neutral kinds built and tested at A1's shares, weighted 0 in what the batch samples until E23's policy (after E05) | random neutrals teach the wrong lessons (the epic's risk) |

## 5. The Python side (e) — built on Linux (T6a), PyTorch locked (T6b)

`python/` (package `h3env`, `python/README.md`): ctypes over C ABI v28 (no compile step, the same
on Linux and Windows; `ctypes.CDLL` releases the GIL in each call), numpy arrays allocated once
and written in place by the engine (zero copy). `_ffi.py` (the library from `$H3_LIBRARY`, else
`../target/release/`; every signature; a library of another `h3_abi_version` is refused),
`batch.py` (`BattleBatch`), `seats.py` (`SeatBook`), `tools.py` (single envs, fork, hash: §6),
`bench.py`, `tests/` (pytest; a lane command, not a gate). A uv-managed CPython 3.12 in
`python/.venv` (`.python-version`); `uv.lock` for Linux and Windows on x86-64 (`tool.uv
.environments`), numpy and pytest, and torch in the group `train`. Decisions of the build (the agent's, provisional):

- **The group `train`** (T6b, 2026-10-19): `torch==2.14.*` from PyTorch's index
  `https://download.pytorch.org/whl/cu126` only (`tool.uv.sources`, an `explicit` index), its
  dependencies from PyPI; locked as 2.14.1+cu126 for Python 3.10 to 3.15 on both platforms, with
  the CUDA 12.6 runtime (`cuda-toolkit` 12.6.3, `nvidia-*-cu12`) on Linux (the Windows wheel
  carries its own). Opt-in (`uv sync --group train`, about 4 GB on Linux): the default groups
  stay `dev`, and `tests/test_torch.py` skips without torch (the locked build, tensors over the
  engine's arrays, a network's masked choices for every head, the GPU's kernels where there is
  one). Licences: torch is BSD-3; NVIDIA's CUDA libraries are proprietary, fetched by uv on the
  machine that trains, never distributed with or linked into the engine (ADR-0009 point 3 covers
  the engine's dependencies).
- **`BattleBatch.step`** takes the heads as arrays (or reads `actions`, an `H3Action` record
  array); with `check` (default) it raises when an env with a side to move refused its action,
  while an env that gave up refuses silently (`status`).
- **`SeatBook`:** a side's transition opens when it acts and closes at its next decision or at
  the episode's end (both sides' then), with the sum of its rewards in between; a side that never
  acted has none. A Gymnasium-style wrapper stays out until a library needs one (T8).
- **The random choice** (`sample`, `random_actions`, as `h3 ai bench`): one draw per row over the
  legal indices' flat positions (82 µs for 64 × 1 404 entries; a cumulative sum cost 658 µs).

## 6. The information boundary (f)

- The user (2026-10-18): every per-player observation coming out of a battle is already exactly
  the information a human player receives. No check against the original's screens is needed;
  E05 relies on the projection (`h3_battle::observe`, ADR-0008, `docs/06-security.md`).
- The batch API serves only the acting seat's `observe` (and, at a truncation, both sides'), never
  the state, its hash or a fork. Full-state access stays in the single-env functions
  (`h3_env_state_hash`, `h3_env_fork`, `BattleEnv::state`), marked trusted tooling in `h3.h` and
  in `h3env.tools` (exit criterion 3). The batch's episode seeds (an output for bookkeeping and
  replays, ADR-0025 point 1) decide the battle's draws: marked trusted tooling too, never an
  agent's input (the trainer reads them only to keep its held-out episodes apart; E05 T9b).
- The flat observation is the visible state now; a human also sees the battle's log. A fact that
  only history tells and that matters for play belongs in the observation (BAT-Q35 is one such
  corner, the battle lane's).

## 7. Benchmarks (g)

- **Gated** (instruction counts, `benches/ai/`, `lane_only = true`: the battle lane's changes move
  them): `batch-random-full` — a batch of 64 envs of `packs/synthetic/scenarios/full.toml`, one
  thread, 1 000 calls of random legal actions chosen from the batch's own masks (casts through the
  heads), observations and masks written, counted inside `h3_sim::batch::BattleBatch::step`;
  `batch-random-siege` the same on the siege; `batch-random-encounters-v1` (T5b2) the same on the
  encounter distribution at its default weights (`h3 ai bench --encounters default`): 11.86 G
  instructions against 11.45 G for `full`, the automatic resets' sampling included (on the
  sandbox about 39 700 steps/s inside `step` against 50 700 for `full`, a run beside a verify).
- **Recorded** (wall time, no gate): `h3 ai bench --envs N --threads T --calls K` prints steps/s
  inside `BattleBatch::step` and with the head calls and the random choice; on the user's Ryzen 9
  3900X for T = 1, 6, 12, 24 and N = 64, 256, 1 024 at T9 (moved from T3 by the user, 2026-10-18;
  the sandbox's indication in the log, T3c), and the Python binding's steps/s against Rust's
  (T6), in the epic log.
- **Built (T3c):** both gated workloads (`benches/ai/`, lane-only, threads 0 so that every count
  is in the caller's thread; collect `*BattleBatch>::step`, never inlined).
- The single-env gates (the battle lane's `random-rollouts-*`) stay as they are.

## 8. Smoke training (T8) — built (T8a–T8c, 2026-10-19; AI-Q5); exit criterion 7 met

Exit criterion 7: a smoke run, within an hour on the user's machine, wins at least 80 % against
Random on held-out seeds of its encounter family. Decided at T8a's start (the user, 2026-10-19):
an own compact PPO, not a library (§8.3); at the review (AI-Q5) the rest as proposed here:
self-play, the default distribution, C ABI v29's seats, the criterion and the settings.

### 8.1 Random against Random (measured, T8a)

1 000 episodes per kind (sandbox, batch seed 7, 64 envs, a 3 000-step limit that no episode
reached), both seats external playing `random_actions`, which chooses as the internal `Random`
does (`rollout::choose`: a uniform entry, then uniform heads):

| Kind | Attacker wins | Defender wins | Decisions, median (attacker, defender) | Both sides, p99 | Casts |
|---|---|---|---|---|---|
| hero | 72.4 % | 27.4 % | 148, 108 | 856 | 1.5 % |
| siege, a hero defends | 60.1 % | 39.8 % | 134, 110 | 650 | 1.7 % |
| siege, a garrison | 92.3 % | 7.7 % | 88, 42 | 465 | 1.3 % |

So the agent must play both sides: as the attacker alone Random already wins most of these
battles. Over the family below (§8.2), the agent in each seat half of the time, Random's own rate
against Random is about 50 %. The observation's values reach 6 · 10^5 (counts, hit points,
values): 1 731 of its 8 882 features exceed 1 somewhere, 293 exceed 100. Python played 16 000
steps/s on the sandbox's 4 cores (`threads = 4`).

### 8.2 The family, the opponent, the held-out seeds

- **The family:** the encounter distribution v1 at its default weights (`add_encounters()`):
  the three kinds with a player on both sides, hero 4/7, a siege a hero defends 2/7, a siege
  of a garrison 1/7 (the neutral kinds weigh 0 until E23, A12). Every faction, week, hero and
  army the sampler draws; sieges included. Fallback, if an hour does not suffice: the hero kind
  alone (`kinds = [0, 0, 1, 0, 0, 0]`), recorded as such.
- **Training is self-play** (decision 8: self-play and the system actor only; Random is for tests
  and measurement, decision 4): one network plays both sides of each battle, both external, and
  both sides' decisions are its samples. **With an opponent pool** (the user, 2026-10-19, after
  T8c's first run: naive self-play's defender regressed against Random, 0.63 → 0.52, while the
  attacker held at 0.93): in `pool_share` (0.5) of the episodes a frozen earlier network plays
  one side, drawn per episode from `pool_size` (8) snapshots taken every `pool_every` (25)
  updates (when full, a new one replaces a random one, so the ages stay spread; the untrained
  network is never one); only the learner's decisions train. A league and its ratings stay
  E06's.
- **Evaluation against the internal Random:** two evaluation batches, the agent attacking in one
  (seats External, Random) and defending in the other (Random, External), so every reported
  decision is the agent's. It needs **seats for the encounter source** (C ABI v29, built in
  T8b; `BattleBatch.add_encounters(attacker=, defender=)` in Python):
  `h3_batch_add_encounters` gains `const H3Seats *seats` (null: the encounter's own seats, as
  now; else these for every episode of the entry, at least one external), and
  `Scenario::encounters` an `Option<[Seat; 2]>`. ADR-0025's point 5 is kept (seats per entry, not
  per batch). The alternative, Random played from Python in the external rows, needs no ABI
  change but is a second Random beside T7's `h3 ai eval`'s.
- **Held-out seeds:** the evaluation batches have their own batch seeds (fixed constants), the
  training batch the run's seed; every evaluation plays the same episodes: the first
  `ceil(E / N)` episodes of each env, the episode set independent of how long battles last. The
  run checks that no evaluation episode seed (`episode_seed`) is a training one (a set of the
  training episodes' seeds; disjoint unless two 64-bit hashes meet). E = 1 000 per side (the 95 %
  interval at 80 % is ±2.5 points per side, ±1.8 overall).
- **The criterion:** the final evaluation's overall win rate (`best.pt` keeps the best
  evaluation's network, reported beside it), both sides weighed equally, at least 80 %, of the
  **sampled** policy (its draws from a generator seeded alike for every evaluation). Decided at
  T8b (the user, 2026-10-19), replacing AI-Q5's greedy choice: in T8b's sandbox run greedy play
  (the most likely entry, then the most likely heads) repeated itself against Random until the
  step limit in up to 17 % of its battles, while the sampled policy never did. A draw or a
  truncation is no win. Reported per side and kind beside Random's own rates and the untrained
  network's, with the mean reward v1 and the truncations; greedy play's rate too.

### 8.3 The trainer: an own PPO over the factorized heads

- **Why own:** a library's PPO (Stable-Baselines3's MaskablePPO) masks each action dimension on
  its own, so a cast's conditioned heads would need decision 5's fallback (casts as sub-steps, a
  new action encoding, the partial choice in the observation, a C ABI change) and a Gym wrapper
  over a batch that already steps vectors. An own PPO (CleanRL's single-file pattern) uses the
  head calls as they are and adds no dependency beyond torch.
- **The action:** the entry from the entry mask; only for the rows that chose a cast (about 1.5 %
  of the decisions), the spell from `spell_mask(entries)`, the target from `target_mask`, the
  second from `second_mask`, one forward pass per head, each conditioned on the earlier choices.
  A row's log-probability is the sum of its chosen heads' (a non-cast row's later heads are "0
  only", log-probability 0); its entropy the entry's plus the cast's heads along the chosen path.
  Masked logits are set to the dtype's lowest value, never `-inf` (no NaN when a row has one
  legal choice).
- **The network** over the flat observation v21 (8 882 values): `symlog(x) = sign(x) ·
  ln(1 + |x|)` on the input (stateless: no running statistics to keep in a checkpoint, the same
  for every kind), a shared trunk of two 512-unit layers (ReLU), the value head (one unit) and the
  entry head (1 404 logits) on it; the spell head (128) takes the trunk and which cast was
  chosen, the target head (298) also the spell (an embedding of 32), the second head (298) also
  the target (an embedding of 32). About 6 M parameters, three quarters of them in the first layer.
- **Self-play's transitions** (`SeatBook`, §5): one trajectory per (env, side). Each call's
  sample is the acting side's (observation, masks, chosen heads, log-probability, value); a
  sample's reward is what `SeatBook` closes for it (reward v1, terminal), and its next value the
  value of the same side's next sample, 0 after a decided battle, the value of `final_obs[env,
  side]` after a truncation. GAE runs backwards over the rollout's calls with a carry per (env,
  side). A transition still open when the rollout ends has no next value yet: it is left out of
  that update (at most two per env); the one before it bootstraps from its value.
- **Settings** (provisional, in the run's config and its report): 256 envs, `threads` = the
  machine's cores, rollouts of 128 calls (32 768 samples), γ 0.995, λ 0.95, Adam at 2.5 · 10^-4
  (linear decay), 4 epochs of 8 minibatches, clip 0.2, value coefficient 0.5, entropy 0.01, the
  gradient's norm at most 0.5, advantages normalized per minibatch; reward v1 at its defaults
  (`ai_value`, mana price 10); a 2 000-step limit per battle. The rollout's observations stay on
  the CPU (32 768 × 8 882 float32, 1.2 GB) and go to the device per minibatch.
- **Budget:** wall time is the limit (`--minutes 60`), evaluations every 10 minutes and at the
  end, and one at the start (the untrained network, and Random's own rates on the same episodes:
  the baselines). Measured in T8b on the sandbox's CPU (4 cores, the default network): 900 samples
  per second, an update of 32 768 samples in about 36 s (the rollout about a third). The
  minibatches are gathered into tensors allocated once (`ppo.Staging`): a fresh 145 MB array per
  minibatch paid its page faults every time (the sandbox's first touch costs 93 ms per 9 MB).

### 8.4 Where the code lives (built, T8b)

- `python/h3env/train/` (needs the group `train`; `import h3env` stays free of torch):
  `policy.py` (the network, the masked heads, sampling and log-probabilities), `ppo.py` (the
  rollout over a `BattleBatch` with `SeatBook`, GAE, the update), `evaluate.py` (the held-out
  evaluation against Random, §8.2), `__main__.py` (`uv run python -m h3env.train --minutes 60
  --out runs/<name>`).
- **A run's output** in `python/runs/<name>/` (git-ignored): `config.json` (every setting, the
  ABI version, torch's, the device, the commit), `metrics.jsonl` (per update: steps,
  episodes, steps/s, the losses, entropy, approximate KL, clip fraction, explained variance;
  per evaluation: the rates of §8.2), `checkpoint.pt` (the network and the optimizer at each
  evaluation), `best.pt` (at the best sampled rate). The results go into `E05-log.md` by hand.
- **Tests** (`tests/test_train.py`, skipped without torch, a lane command): sampling never picks
  a masked entry or head, a cast's heads conditioned row by row; the log-probability of a chosen
  action recomputed equal; GAE on a hand-computed two-sided trajectory (a side acting twice in a
  row, a truncation, an open transition left out); two updates on a small batch with finite
  losses and no refused action; the evaluation's episode set fixed and disjoint from training's;
  a checkpoint's round trip.
- **T8b's sandbox run** (CPU, 4 cores): a short run whose evaluation rate rises clearly above the
  untrained network's on the same held-out episodes, its numbers in the log; T8c is the hour on
  the user's machine with the GTX 1070.
- **Determinism:** the env is deterministic; torch on a GPU is not bit for bit. Seeds are set and
  recorded; a run is reproduced in its numbers, not bitwise. Nothing here is a gate.

## 9. Baselines (T7) — built 2026-10-19 (AI-Q6)

Increment 5, cut at the user's checkpoint (2026-10-19): the bots as internal seats, the runner,
the trained network's row. The user's decisions: two bots (Aggressor, Greedy value); their
damage estimate is the engine's preview of their own attack (AI-Q6, ADR-0025 point 5 amended),
never of the retaliation; the ratings are the server's Glicko-2, copied.

- **The preview** (`h3_battle::preview`, BAT-DMG-011; the AI lane's `cross-lane` change, #274):
  for an attack or shot that validates, the first strike's least and greatest damage and kills,
  every stage of the real strike, none of the rolls. Leaves out the retaliation, a second strike,
  secondary targets and reflection (the user: the original shows none of them); the agents'
  encodings may carry it later (AI-Q7).
- **The bots** (`h3_sim::bots`, `Policy::Aggressor`, `Policy::Greedy`): read their side's
  observation, the creature data and the preview; never cast; deterministic (ties to the command
  listed first). Aggressor: shoot the nearest enemy it can shoot, else attack the nearest it can
  reach, else walk toward the nearest. Greedy: the shot or attack with the greatest previewed
  kills × `ai_value`, less its own estimate of the retaliation (the survivors' mean damage scaled
  by attack against defense in per mille, as BAT-DMG-002's first stage: integers only), else walk
  toward the most valuable enemy (count × `ai_value`).
- **Seats** (C ABI v30): `H3_SEAT_AGGRESSOR` (2), `H3_SEAT_GREEDY` (3); Python's `"aggressor"`,
  `"greedy"` for `add_scenario` and `add_encounters`.
- **`h3 ai eval`** (`h3-cli`'s `src/ai/eval.rs`): every ordered pair of policies on the same
  held-out encounters, the episode seeds of the Python evaluation (`EVAL_SEED`, 100 envs × 10);
  per pair the win rate (both sides weighed equally, a draw or truncation no win) and value
  efficiency (the mean reward v1); Glicko-2 ratings (`src/ai/rating.rs`, copied from
  `h3-server`), each game one rating period, episode by episode in a fixed order of the pairs.
- **The network's row:** `python -m h3env.train.ladder --checkpoint <run>/best.pt` plays the
  sampled policy against `random`, `aggressor` and `greedy` on the same episodes.
