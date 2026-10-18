# E05 — Battle environment: design note (T1)

Working detail behind the ADR draft `docs/lanes/ai/adr-drafts/battle-env.md` (E05 T1, 2026-10-18).
A handover, not binding: each task that builds a section deletes it here (`docs/adr/README.md`,
"Keep ADRs short"). Sections follow T1's points (a) to (g); open points are OPEN-QUESTIONS AI-Q2
and AI-Q3.

## 1. The batch API (a)

**Rust** (`h3-sim`, module `batch`): `BattleBatch` owns N `BattleEnv`s, each with its episode
source entry, its seats and its reward state. `step` takes N actions and writes into the caller's
buffers; a single-threaded `BattleBatch` is `Send`, and a `Pool` of std threads (module
`batch::pool`, no external crate) splits it into T contiguous shards (env i in shard
`i * T / N`), one worker each, parked between calls. T = 0 runs in the caller's thread. Shards are
independent: what an env does never depends on T or on the shard it is in.

**C ABI** (`h3-ffi`, v27 in T3; names provisional):

```c
typedef struct H3BatchConfig {
    uint32_t envs;        /* N */
    uint32_t threads;     /* T; 0 = the caller's thread */
    uint64_t seed;        /* the batch's seed: every env's episode seeds derive from it */
    uint32_t max_steps;   /* accepted steps of both seats per episode; 0 = unlimited */
    H3RewardConfig reward;
} H3BatchConfig;

/* The caller's arrays, passed with every call (numpy arrays owned by the binding). A null
 * pointer skips that output. Shapes: [N] unless noted. */
typedef struct H3BatchBuffers {
    float   *obs;          /* [N, OBS]: the acting seat's projected observation */
    uint8_t *entry_mask;   /* [N, ACTIONS] */
    int8_t  *to_move;      /* the acting side (0 attacker, 1 defender) */
    float   *reward;       /* [N, 2]: this call's reward per side */
    uint8_t *done;         /* the episode ended in this call (decided or truncated) */
    uint8_t *truncated;
    int8_t  *result;       /* H3_RESULT_* of the episode that ended, else H3_RESULT_NONE */
    float   *final_obs;    /* [N, 2, OBS]: both sides' last observations, written only for
                              envs truncated in this call (bootstrapping) */
    uint32_t *episode;     /* the env's episode counter (it grows at each automatic reset) */
    uint32_t *scenario;    /* the index of the episode's scenario in the source (fixed lists) */
    uint64_t *episode_seed;
    int32_t *status;       /* per env: H3_OK or the step's error; an env with an error is unchanged */
} H3BatchBuffers;

H3Batch *h3_batch_create(const H3BatchConfig *config, char *error, size_t error_len);
int32_t h3_batch_add_scenario(H3Batch *b, const char *toml, size_t len, uint32_t weight,
                              H3Seats seats, const H3Stakes *stakes, char *error, size_t error_len);
int32_t h3_batch_set_encounters(H3Batch *b, const char *toml, size_t len, char *error,
                                size_t error_len);                                   /* T5 */
int32_t h3_batch_reset(H3Batch *b, const H3BatchBuffers *out);   /* every env's first episode */
int32_t h3_batch_spell_mask(H3Batch *b, const uint32_t *entries, uint8_t *out, size_t len);
int32_t h3_batch_target_mask(H3Batch *b, const uint32_t *entries, const uint16_t *spells,
                             uint8_t *out, size_t len);
int32_t h3_batch_second_mask(H3Batch *b, const uint32_t *entries, const uint16_t *spells,
                             const uint16_t *targets, uint8_t *out, size_t len);
int32_t h3_batch_step(H3Batch *b, const H3Action *actions, const H3BatchBuffers *out);
uint64_t h3_batch_episode_seed(uint64_t batch_seed, uint32_t env, uint32_t episode);
void h3_batch_destroy(H3Batch *b);
```

- **One call per head** (decision 5): the entry masks come with every reset and step; the three
  head calls answer for every env at once, conditioned on that env's earlier choices. An env whose
  entry is no cast gets the mask "0 only", as the single env's heads do. The heads run on the pool
  too.
- **Step:** for each env, decode and apply the acting seat's action; then the internal seats play
  (§2) until an external seat must act or the episode ends; then, if it ended, the reward is
  final and the env resets at once (**same-step automatic reset**): its next episode's first
  observation, entry mask and acting seat are what `out` shows; `done`, `result`, `reward` and
  `final_obs` belong to the episode that ended. Observations and masks are written by the workers,
  so the encoding (about a third of a step) runs in parallel too.
- **Errors per env:** an action that does not decode or is rejected leaves that env unchanged
  (`status` < 0) and the others step; the call returns `H3_ERR_REJECTED` if any env failed. A
  refused action is never retried or replaced by the engine.
- **Seeds:** env i draws its episode seeds from its own stream of the batch seed (`StreamId::env(i)`,
  a kernel registry entry added by T3's shared pull request); episode k's seed is that stream's
  k-th `u64`. `h3_batch_episode_seed` gives it, so a single `BattleEnv` reproduces any episode of
  a batch. Scenario choice and the sampler draw from the episode seed's `StreamId::SETUP`; random
  obstacles from `OBSTACLES` as today; internal seats from `StreamId::seat(side)` (T3).
- **The determinism property** (exit criterion 2, T3): for random batches (N, T, sources, seats),
  stepping the batch with random legal actions equals stepping N single envs with the derived
  seeds, state hash by state hash and output by output; and every action the masks allow applies.

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
- **`human` and `difficulty`** (ADR-0023 point 7) are scenario data, not seat properties: the
  game the battle belongs to says them. What an agent's own side counts as is AI-Q3 (BAT-Q33's
  open point); the proposal there is `human = false`.

## 3. Reward v1 (c)

### 3.1 The balance

One currency: creature `ai_value` (harvested, `packs/*/core.toml`). For side s and its opponent
o, from the battle's start (the setup and the stakes) and its `BattleOutcome`:

    r_s = (D_o − D_s) + (T_o→s − T_s→o) − p · (m_s − m_o)

- **D (destroyed):** value a side loses that nobody gains.
  - Army: `A(0) − A(end)`, `A` = Σ count × `ai_value` over the army and raised stacks
    (`StackOrigin::Army`, `Raised`). `A(0)` from the setup's army, `A(end)` from what the side
    keeps after the battle (below). Raised Demons and permanent resurrections are in `A(end)`;
    creatures raised only for the battle are not (`StackRecord::end` already leaves them out).
  - War machines: the `ai_value` of each of the setup's machines that is not in
    `BattleOutcome::war_machines` (Ballista 600, Catapult 500, Ammo Cart 400, First Aid Tent 300
    in `base`); a defeated side's surviving machines count as destroyed too (v1). Turrets are no
    side's (the outcome leaves them out) and count nothing; walls and towers are repaired after a
    siege and count nothing.
  - Created stacks (summons, clones; `StackOrigin::Created`) count nothing on either side: they
    end with the battle. Their cost is the mana that made them.
  - The hero `H`: lost when its side is defeated (a victory over it) or in a draw; kept after a
    win, a retreat or a surrender.
  - Artifacts `R`: lost in a draw (nobody captures them).
- **T (transferred):** value that moves from the loser to the winner, counted once.
  - Artifacts `R` of a hero defeated by a victory (GAM-BAT-004; v1 counts them all, whether or not
    the winner has room, HER-CHG-002).
  - The prize `P` of the losing side: a town (a siege lost, also by the defender's retreat through
    the Escape Tunnel or its surrender), a wandering monster's treasure, a guarded pile's contents.
  - A surrender's price: `BattleOutcome::surrender_cost` × g.
- **What each side keeps (`A(end)`),** as the orchestrator applies the outcome (ADR-0013 point 5,
  `h3-game`'s `end_battle`): the winner its survivors; a side that retreats nothing (its army is
  lost: destroyed); a side that surrenders its survivors; a neutral side (no hero) its survivors
  whatever the result (they stay on the map); a defeated side nothing.
- **Mana:** `m` = the hero's mana at the start (`BattleHero` in the setup) minus
  `HeroOutcome::mana`; price p per spell point.
- **Zero-sum:** `r_o = −r_s` by construction (every term is antisymmetric). In self-play both are
  used; against the system actor only the external side's is (the internal one is reported too).
- **Scale:** r is divided by `S` = `A_a(0) + M_a(0) + A_d(0) + M_d(0)` (both sides' armies and war
  machines at the start, at least 1), so a battle's size does not set the reward's size. Stakes
  can make |r| exceed 1 (example 2).
- Computed in the sim layer (`h3-sim`, module `reward`) from integers (sums in `i64`), scaled once
  in `f64`, reported as `f32`; it never feeds back into the battle (ADR-0003).

### 3.2 Weights and stakes

- **p**, the mana price (ai_value per spell point): AI-Q2.
- **g**, the gold price (ai_value per gold): default the median `ai_value / gold cost` of the
  ruleset's faction creatures of levels 1–7 (computed from the ruleset at T4; 1.57 for `base`'s
  112 such creatures). Used for the surrender price, artifacts and the hero.
- **Stakes** are not battle state: the outcome does not report them, so the episode source gives
  them per side: `H` (hero), `R` (artifacts), `P` (prize), in ai_value. A fixed scenario carries
  them in `h3_batch_add_scenario` (default 0); the encounter sampler (T5) computes them from what
  it sampled: `H = (2 500 + 500 · (level − 1)) gold · g` (a proposal, AI-Q2), `R` = Σ the
  artifacts' gold cost (`packs/*/core.toml`) · g, `P` = a town's buildings' cost · g or a
  treasure's resources at their market value · g.
- No separate escape weight: an escape is priced by what the escaping side keeps (`H`, `R`, and on
  a surrender its army), which a kill would have destroyed or captured. Each weight is reported
  with every run (E05 risks).
- **Left out in v1** (the orchestrator's consequences after the battle): the winner's experience,
  Necromancy's raised creatures, Eagle Eye's learned spells. They grow a hero's later value; E06
  may add them as stakes.

### 3.3 Terminal by default; optional shaping

- The reward is 0 on every step but the one that ends the battle. A truncated episode gets 0 and
  `truncated = 1`, with both sides' final observations, so the learner bootstraps.
- **Optional potential-based shaping** (off by default): `Φ_s(t)` = `(K_o(t) − K_s(t) −
  p · (m_s(t) − m_o(t))) / S`, where `K(t)` is the value a side's army and raised stacks have lost
  so far plus its war machines destroyed so far, and `m(t)` the mana spent so far (no stakes, no
  retreat or surrender); `Φ = 0` at the start and after the end; each step adds
  `γ · Φ(t+1) − Φ(t)` (γ from the config, default 1). With γ = 1 the shaped rewards of an episode
  sum to its terminal reward: shaping moves credit earlier, it cannot change the best play.
- **Per seat:** the batch reports both sides' rewards every call; the Python side accumulates each
  seat's rewards between its own moves (a seat that did not make the last move still gets the
  episode's end, §5).

### 3.4 Worked examples (`base` values, g = 1.57, p = 10; S as above)

1. **A wandering monster.** A level-5 hero (`H` = 4 500 gold · g = 7 065; artifacts 6 000 gold,
   `R` = 9 420) with 20 Swordsmen (445) and 10 Griffins (351), `A_a(0)` = 12 410, attacks 50
   Skeletons (60), `A_d(0)` = 3 000, no treasure. The hero casts 10 mana; the attacker loses 3
   Swordsmen and wins. `D_d` = 3 000, `D_a` = 1 335, T = 0, mana −100:
   `r_a` = 3 000 − 1 335 − 100 = **1 565**, S = 15 410, **0.102**. With a treasure of 1 000 gold
   (`P_d` = 1 570): **3 135, 0.203**.
2. **Hero against hero: retreat, surrender, kill.** Attacker: level 8 (`H_a` = 9 420), 40 Orcs
   (192) and a Ballista (600): 8 280. Defender: level 6 (`H_d` = 7 850), artifacts 4 000 gold
   (`R_d` = 6 280), 60 Goblins (60) and 15 Wolf Riders (130): 5 550; S = 13 830. The defender
   loses 30 Goblins and 5 Wolf Riders (2 450) and casts 15 mana; the attacker loses 6 Orcs
   (1 152), no mana. Then:
   - **the defender retreats:** its 3 100 of survivors are lost too: `D_d` = 5 550, its hero and
     artifacts are kept: `r_a` = 5 550 − 1 152 + 150 = **4 548, 0.329**;
   - **it surrenders** for a price of 1 450 gold (T = 2 277) and keeps its survivors: `D_d` =
     2 450: `r_a` = 2 450 + 2 277 − 1 152 + 150 = **3 725, 0.269** (surrender beats retreat for the
     defender by 823, as in the game when the price is below the army's value);
   - **the attacker kills the army instead** (the same losses of its own): `D_d` = 5 550 + 7 850,
     T = 6 280: `r_a` = **18 678, 1.351**. Blocking the escape is worth `H_d + R_d` = 14 130
     (1.02 of S) here.
3. **Summons.** The attacker's hero summons Earth Elementals (25 mana at Basic): −250. The
   elementals that die cost nothing more; what they kill counts as any kill.
4. **A siege.** A garrison without a hero (`A_d(0)` = 6 000) loses to an attacker of 15 000 that
   loses 2 000; the town's buildings cost 20 000 gold (`P_d` = 31 400): `r_a` = 6 000 − 2 000 +
   31 400 = **35 400**, S = 21 000, **1.686**. A town is the largest stake of most battles, as in
   the game.
5. **A draw in self-play.** Both armies die by the same action: each side destroys the other's
   army, hero and artifacts: `r_a = (A_d + M_d + H_d + R_d) − (A_a + M_a + H_a + R_a) + p · (m_d − m_a)`
   (losses of machines that survive left out).
6. **Raised stacks.** A Pit Lord raises 5 Demons (445) from its side's dead stack: the dead stack
   counts fully in `A(0) − A(end)`, the Demons it keeps count in `A(end)`: +2 225 against the loss.

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
- **Episode sources of a batch:** (i) a weighted list of fixed scenarios (`h3_batch_add_scenario`:
  text, integer weight, seats, stakes); each reset draws one by weight; or (ii) the encounter
  distribution (T5).

### 4.2 The encounter distribution v1 (shape; T5 fills it)

A versioned TOML in the AI lane (`crates/h3-sim/data/encounters-v1.toml`; parsed in Rust, which
needs `serde`/`toml` in `h3-sim` and `h3-heroes` for the heroes: T5's shared pull request), its
weights drawn from a census or stated as assumptions, each with `why`:

```toml
# The shape only: every number here is a placeholder that T5 replaces with a derived or stated one.
[distribution]
id = "encounters"
version = 1
packs = ["base", "ab", "sod"]
profile = "base:faithful"
census = "census-v1"                 # crates/h3-sim/data/census-v1.toml: derived numbers only

[kind]                               # who fights: weights (assumption: per game week, below)
wandering = { weight = 0.50, why = "…" }
guards    = { weight = 0.10, why = "…" }     # guarded piles, later creature banks
hero      = { weight = 0.20, why = "…" }     # hero against hero in the field
siege     = { weight = 0.20, why = "…" }     # a town, with or without a fort or a hero

[week]                               # the game week: army sizes, hero levels, the mage guild
weights = { 1 = 0.20, 2 = 0.16, 3 = 0.13 }  # … (assumption)

[terrain.wandering]                  # census: monster placements per terrain
from = "census.monster_terrain"
[terrain.hero]                       # census: passable land tiles per terrain
from = "census.land_terrain"
[battlefield]                        # census: the share of battles on each special ground
from = "census.special_ground"
[monster]                            # census: placements per level class and fixed creature;
from = "census.monster_level"        # counts: the original's random counts by level and week
[army]                               # value by week (median, spread) and the attacker/defender
value = { … }                        # value ratio (assumption: attackers choose fights)
balance = { … }
[hero]                               # level by week, class from the census's town factions,
level = { … }                        # skills, artifacts by level, spells by the mage guild
[siege]                              # fort by week, a defending hero's share (assumption)
[difficulty]                         # assumption
weights = { 0 = 0.1, 1 = 0.4, 2 = 0.3, 3 = 0.15, 4 = 0.05 }
[stakes]                             # §3.2
hero = { base_gold = 2500, per_level_gold = 500 }
[seats]                              # §2
neutral = "internal:system"
player = "external"
```

- **Dimensions:** encounter kind; game week; place (terrain, special ground, sea, siege with fort
  level and moat); per side the faction, the army's value and composition by level, a hero or
  none (level, class, primary and secondary skills, artifacts, spells and their masteries, war
  machines, through `h3-heroes`' `derive_battle_hero`); the attacker/defender value ratio;
  difficulty; seats and stakes.
- **The census** (T5a): `h3 ai census <maps>` (`h3-cli`'s `src/ai/`, run locally with the user's
  maps through `h3-map`) writes aggregate numbers only (ADR-0009: counts and shares, no map
  content, no per-map rows): land tiles per terrain, monster placements per terrain and level
  class, special-ground tiles per kind, towns per faction, guarded piles, water share, map sizes.
  Each placement counts once (a larger map holds more battles). Its derivation is recorded with
  it; the adventure lane's census (`docs/rules/surveys/E08-battles.md` §1.1) is the precedent.
- A curriculum samples the same dimensions with other weights (an override table of the same
  shape). T5b's check samples many scenarios and compares each dimension's frequencies with its
  weights.

## 5. The Python side (e)

- **ctypes** over the C ABI: in the standard library, no compile step, the same on Linux and
  Windows; `ctypes.CDLL` releases the GIL during each call. cffi's ABI mode would add a
  dependency for the same thing; its API mode a build step per platform.
- **Zero copy:** the binding allocates the numpy arrays of `H3BatchBuffers` once and passes their
  data pointers each call; the engine writes into them. Python never copies an observation.
- **Layout** (`python/`, the AI lane's):

  ```
  python/pyproject.toml   package h3env; requires-python >= 3.10; numpy; group "train": torch 2.14
  python/uv.lock          Linux and Windows, torch from the cu126 index
  python/h3env/_ffi.py    loads the library ($H3_LIBRARY, else ../target/release/), signatures,
                          refuses a library whose h3_abi_version differs from the binding's
  python/h3env/batch.py   BattleBatch: config, scenarios, reset/step, the head calls, numpy views
  python/h3env/seats.py   per-seat bookkeeping: the acting seat per env, each seat's accumulated
                          reward between its moves, episode ends for the seat that did not move
  python/h3env/tools.py   single envs, fork, state hash: trusted tooling (§6)
  python/tests/           pytest: the ABI version, batch vs single envs, masks, a random run
  python/train/           T8's PPO
  ```

- **Environment:** a venv in `python/.venv` with a uv-managed CPython 3.12 (the user's system
  Python is older than 3.11 and stays untouched); `uv sync` installs it; `uv sync --group train`
  adds PyTorch 2.14 from `https://download.pytorch.org/whl/cu126` on Linux and Windows (x86-64),
  pinned in `uv.lock`. The binding's tests need numpy only, so they run in the sandbox (PyPI is
  reachable there; the PyTorch index may not be: T6 locks it on the user's machine if so).
- A Gymnasium-style vector wrapper is a thin layer over `BattleBatch` if a library needs one (T6
  or T8 decides); the factorized, conditioned heads mean most libraries need adapting anyway.

## 6. The information boundary (f)

- The user (2026-10-18): every per-player observation coming out of a battle is already exactly
  the information a human player receives. No check against the original's screens is needed;
  E05 relies on the projection (`h3_battle::observe`, ADR-0008, `docs/06-security.md`).
- The batch API serves only the acting seat's `observe` (and, at a truncation, both sides'), never
  the state, its hash or a fork. Full-state access stays in the single-env functions
  (`h3_env_state_hash`, `h3_env_fork`, `BattleEnv::state`), marked trusted tooling in `h3.h` and
  in `h3env.tools` (exit criterion 3).
- The flat observation is the visible state now; a human also sees the battle's log. A fact that
  only history tells and that matters for play belongs in the observation (BAT-Q35 is one such
  corner, the battle lane's).

## 7. Benchmarks (g)

- **Gated** (instruction counts, `benches/ai/`, `lane_only = true`: the battle lane's changes move
  them): `batch-random-full` — a batch of 64 envs of `packs/synthetic/scenarios/full.toml`, one
  thread, 1 000 calls of random legal actions chosen from the batch's own masks (casts through the
  heads), observations and masks written, counted inside `h3_sim::batch::BattleBatch::step`;
  `batch-random-siege` the same on the siege. T5 adds `batch-random-encounters-v1`.
- **Recorded** (wall time, no gate): `h3 ai bench --envs N --threads T --calls K` prints steps/s;
  on the user's Ryzen 9 3900X for T = 1, 6, 12, 24 and N = 64, 256, 1 024 (T3), and the Python
  binding's steps/s against Rust's (T6), in the epic log.
- The single-env gates (the battle lane's `random-rollouts-*`) stay as they are.
