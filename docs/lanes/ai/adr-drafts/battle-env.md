# ADR draft: The battle environment for training (E05)

- Status: Proposed (E05 T1, 2026-10-18; for the user's review)
- Date: 2026-10-18
- Deciders: user + AI lane session

Detail (the C sketch, the reward's formula and worked examples, the distribution's shape, the
package layout): `docs/epics/E05-design.md`. Open points: OPEN-QUESTIONS AI-Q2 (the reward's
defaults) and AI-Q3 (what an agent's side counts as).

## Context

E05 (AI-Q1) makes every battle the engine plays trainable from Python through the C ABI: in
batches, against itself and the system actor, sampled as battles occur in games, with a reward in
one currency. `BattleEnv` (`h3-sim`) and C ABI v25 (`h3-ffi`) hold one env per handle on three
synthetic scenarios and reward ±1. ADR-0011 (the C ABI), ADR-0014 point 9 (the factorized cast)
and ADR-0023 point 7 (`human`, `difficulty`) stay as they are.

## Decision

1. **A batch API.** `h3-sim` gains `BattleBatch`: N envs, stepped with one call; outputs go into
   the caller's buffers (observations, entry masks, acting side, rewards of both sides, done,
   truncated, result, episode counter, scenario index, episode seed, a status per env). The three
   cast heads are one call each for the whole batch, conditioned on each env's earlier choices
   (decision 5). An env whose action is refused stays unchanged and reports it; the others step.
2. **Automatic reset in the same step.** An env whose episode ends resets at once; the outputs show
   the new episode's first observation and mask, and the ended one's reward, result and, after a
   truncation, both sides' final observations.
3. **Threads only in `h3-sim`'s pool:** std threads (no new crate), T chosen at creation (0 = the
   caller's thread), N envs split into T contiguous shards. An env's play never depends on T.
4. **Seeds:** env i draws its episode seeds from stream `env(i)` of the batch seed (a kernel stream
   id, T3); `h3_batch_episode_seed` exposes them. **A batch steps bit for bit as N single envs**
   with those seeds (a property test).
5. **Seats per scenario side:** `External` (the caller acts; every acting side the batch reports
   is external) or `Internal(policy)`, played inside the step until an external side must act or
   the episode ends. Policies are a closed enum in `h3-sim` choosing a `Command`: `Random` (tests
   and benchmarks), `System` (E23's function, shared with the adventure's neutral seat), the
   baseline bots (T7). Every scenario has an external side. Bots read their side's observation
   only.
6. **`human` and `difficulty` are scenario data,** set by the game a battle belongs to, never by a
   seat (ADR-0023 point 7). An agent's own side: AI-Q3.
7. **Reward v1** (decision 9), computed in the sim layer from the setup, the outcome and the
   episode's stakes: `r_s = (D_o − D_s) + (T_o→s − T_s→o) − p · (m_s − m_o)`, divided by both
   sides' army and war-machine value at the start. D is value destroyed (creatures by
   `ai_value`, war machines, a defeated hero, a retreating side's army, artifacts in a draw), T
   value moved to the winner once (a defeated hero's artifacts, the loser's prize: a town, a
   treasure; a surrender's price at the gold price g), m the mana spent at price p. Created stacks
   count nothing; raised and permanently resurrected creatures count as the side's. Zero-sum by
   construction. Stakes (hero, artifacts, prize) come with the episode, not from the battle.
   Terminal by default; potential-based shaping optional (it sums to the terminal reward). The
   weights are configuration and reported with every run; their defaults: AI-Q2.
8. **Scenarios are the battle lane's scenario TOML,** unchanged: it already names every place
   (terrain, special battlefield, garrison, siege, sea). Packs come by id from the five built-in
   packs; one ruleset per distinct (packs, profile) per batch, shared. A batch's episodes come from
   a weighted list of scenarios (with their seats and stakes) or from the encounter distribution.
9. **The encounter distribution v1** is a versioned TOML in `h3-sim` (dimensions: kind of
   encounter, game week, place, armies and their balance, heroes through `h3-heroes`, difficulty,
   seats, stakes), each weight from a census of the shipped maps or a stated assumption with its
   reason. The census holds aggregate numbers only (ADR-0009), made by `h3 ai census` from the
   user's maps.
10. **Python:** `ctypes` over the C ABI, the caller's numpy arrays passed by pointer (zero copy), a
    package `python/h3env` that refuses a library of another ABI version; a venv with a
    uv-managed CPython 3.12 and `python/uv.lock` pinning PyTorch 2.14 (cu126) for Linux and
    Windows in a separate training group.
11. **Information:** only projected observations cross the batch API (the acting side's; both
    sides' at a truncation). The state, its hash and forks stay single-env functions, marked
    trusted tooling (ADR-0011 point 4). The projection is a human player's view (the user,
    2026-10-18); no separate check of the original's screens.
12. **Benchmarks:** instruction-count workloads in `benches/ai/` (lane-only): a one-thread batch on
    `full` and on the siege, random legal actions from its own masks; wall-time throughput over
    threads and batch sizes recorded on the user's machine, not gated.
13. **Versions:** no encoding changes. C ABI v26 (T2: an env from a scenario), v27 (T3: the batch).

## Consequences

- Shared pull requests ahead of the tasks: kernel stream ids `env(i)`, `seat(side)` (T3); `h3-sim`'s
  dependencies `h3-heroes`, `serde`, `toml` in `arch.toml` (T5). No contract type changes.
- `h3-sim` grows modules `batch`, `reward`, `source`, `encounter`, `bots`; `h3-cli`'s `src/ai/` the
  commands `h3 ai bench`, `h3 ai census`, `h3 ai eval`.
- The neutral share of training waits for E23: until then it samples `Internal(Random)`, for tests.
- The reward is only as good as the stakes the sampler states; experience and Necromancy are left
  out in v1.

## Alternatives considered

- **Next-step reset** (Gymnasium's other mode): one more call per episode and a dead step in every
  batch; same-step with final observations is cheaper.
- **A thread pool crate** (rayon): a dependency for what a few std threads do with fixed shards.
- **Per-batch seats:** a distribution mixes self-play and neutral battles, so seats belong to the
  episode.
- **Value differences counted twice** (each side's total asset change): a captured artifact
  would weigh twice a destroyed one.
- **Stakes in the scenario file:** a change to the battle lane's format for what is not battle state.
- **cffi:** a dependency (ABI mode) or a build per platform (API mode) for what ctypes does.
