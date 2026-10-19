# ADR-0025: The battle environment for training

- Status: Accepted (E05 T1; the user, 2026-10-18, AI-Q2, AI-Q3)
- Date: 2026-10-18
- Deciders: user + AI lane session

Detail (the C sketch, the reward's worked examples, the distribution's shape, the package
layout): `docs/epics/E05-design.md`.

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
   cast heads are one call each for the whole batch, conditioned on each env's earlier choices.
   An env whose action is refused stays unchanged and reports it; the others step.
2. **Automatic reset in the same step.** An env whose episode ends resets at once; the outputs show
   the new episode's first observation and mask, and the ended one's reward, result and, after a
   truncation, both sides' final observations.
3. **Threads only in `h3-sim`'s pool:** std threads (no new crate), T chosen at creation (0 = the
   caller's thread), N envs split into T contiguous shards. An env's play never depends on T.
4. **Seeds:** env i draws its episode seeds from stream `env(i)` of the batch seed (a kernel stream
   id, T3); `h3_batch_episode_seed` exposes them. **A batch steps bit for bit as N single envs**
   with those seeds (a property test). An episode drawn from the encounter distribution draws its
   battle from stream `ENCOUNTER` of its seed (T5b), apart from the scenario's draw (`SETUP`).
5. **Seats per scenario side:** `External` (the caller acts; every acting side the batch reports
   is external) or `Internal(policy)`, played inside the step until an external side must act or
   the episode ends. Policies are a closed enum in `h3-sim` choosing a `Command`: `Random` (tests
   and benchmarks), `System` (E23's function, shared with the adventure's neutral seat), the
   baseline bots (T7). Every scenario has an external side. Bots read their side's observation
   and the engine's damage preview of their own attacks (AI-Q6): `h3-battle`'s preview of a
   shot or melee attack gives the first strike's least and greatest damage and kills on its
   target, counting every modifier the strike reveals (the defender's Armorer, its protections),
   without the rolls (luck, death blows, double damage). Never the retaliation, which the
   original does not show either: a bot estimates it itself.
6. **A level playing field: the batch plays every side as a human player's** (AI-Q3): each
   episode's setup has `human = [true, true]` whatever the scenario says. `human = false` remains
   for compatibility with the original's computer player (ADR-0023 point 7); `difficulty` stays
   scenario data.
7. **No escapes for now** (AI-Q2): the batch masks Retreat and Surrender out for every seat and
   refuses them. They belong to the adventure's view of a battle; the encoding keeps their entries
   and a batch option turns them back on later. The single env is unchanged.
8. **Reward v1: kill the most, lose the least, spend mana sparingly** (AI-Q2): in one currency
   (creature `ai_value` by default, the creature's gold cost as the alternative),
   `r_s = (K_o − K_s − p · (m_s − m_o)) / S`. K is the value a side lost: its army and raised
   stacks' value at the start minus that of its survivors, plus its war machines destroyed;
   created stacks (summons, clones) count nothing. m is the mana spent, p its price per spell
   point. S is both sides' army and war-machine value at the start. Nothing from the adventure
   counts: no hero, artifacts, treasure, town or gold. Zero-sum by construction; terminal; a
   truncated episode gets 0. Computed in the sim layer from the setup and the outcome; the
   currency and p are configuration, reported with every run.
9. **Scenarios are the battle lane's scenario TOML,** unchanged: it already names every place
   (terrain, special battlefield, garrison, siege, sea). Packs come by id from the five built-in
   packs; one ruleset per distinct (packs, profile) per batch, shared. A batch's episodes come from
   a weighted list of scenarios (with their seats) or from the encounter distribution.
10. **The encounter distribution v1** is a versioned TOML in `h3-sim` (dimensions: kind of
    encounter, game week, place, armies and their balance, heroes through `h3-heroes`,
    difficulty, seats), each weight from a census of the shipped maps or a stated assumption with
    its reason. The census holds aggregate numbers only (ADR-0009), made by `h3 ai census` from the
    user's maps.
11. **Python:** `ctypes` over the C ABI, the caller's numpy arrays passed by pointer (zero copy), a
    package `python/h3env` that refuses a library of another ABI version; a venv with a
    uv-managed CPython 3.12 and `python/uv.lock` pinning PyTorch 2.14 (cu126) for Linux and
    Windows in a separate training group.
12. **Information:** only projected observations cross the batch API (the acting side's; both
    sides' at a truncation). The state, its hash and forks stay single-env functions, marked
    trusted tooling (ADR-0011 point 4). The projection is a human player's view (the user,
    2026-10-18); no separate check of the original's screens.
13. **Benchmarks:** instruction-count workloads in `benches/ai/` (lane-only): a one-thread batch on
    `full` and on the siege, random legal actions from its own masks; wall-time throughput over
    threads and batch sizes recorded on the user's machine, not gated.
14. **Versions:** no encoding changes. C ABI v26 (T2: an env from a scenario), v27 (T3: the batch).

## Consequences

- Shared pull requests ahead of the tasks: kernel stream ids `env(i)`, `seat(side)` (T3); `h3-sim`'s
  dependencies `h3-heroes`, `serde`, `toml` in `arch.toml` (T5). No contract type changes.
- `h3-sim` grows modules `batch`, `reward`, `source`, `encounter`, `bots`; `h3-cli`'s `src/ai/` the
  commands `h3 ai bench`, `h3 ai census`, `h3 ai eval`.
- The neutral share of training waits for E23: until then it samples `Internal(Random)`, for tests.
- The agent learns to fight battles well, not which battles to fight or when to flee: that is the
  adventure's (E13), which may add escapes and stakes to the reward then.

## Alternatives considered

- **Next-step reset** (Gymnasium's other mode): one more call per episode and a dead step in every
  batch; same-step with final observations is cheaper.
- **A thread pool crate** (rayon): a dependency for what a few std threads do with fixed shards.
- **Per-batch seats:** a distribution mixes self-play and neutral battles, so seats belong to the
  episode.
- **Adventure stakes in the reward** (a hero's and its artifacts' value, a town, a treasure,
  priced escapes; the T1 draft): they decide whether to fight, not how; rejected for v1 (AI-Q2).
- **cffi:** a dependency (ABI mode) or a build per platform (API mode) for what ctypes does.

## Revision history

- 2026-10-18: Accepted (E05 T1 review: the reward without adventure stakes, escapes disabled, both
  sides human).
- 2026-10-19: Point 5: bots also read the engine's damage preview of their own attacks, never
  the retaliation (E05 T7's cut, AI-Q6).
