# Run records

Every training run the epics' results cite has its record here (ADR-0026 point 9; E06 exit
criterion 1): `<run>.json`, a copy of the run directory's final `record.json`
(`python/runs/<run>/`, git-ignored). Checkpoints and networks stay on the machine that trained
them.

A record (schema 5, `h3env.train.run.Record`; `docs/epics/E06-log.md` §4; `check_record`
also accepts schema 4, before the league: `t3b-1`, its config's `[pool]`):

- `id`: the run's name (its config's `name`, else its directory's).
- `identity`: when it started, the commit and whether tracked files differed from it (`dirty`),
  the C ABI version, the observation and action encodings' versions and sizes, the encounter
  distribution's version, the batch's `entries` (each one's kind, weight and the content hashes
  of its core, battle and adventure rulesets; schema 3, E06 T2c), the network's parameters,
  torch, CUDA, the device and GPU, the OS, CPU, cores, Python and numpy.
- `parent` (schema 4, E06 T3b): the network the run started from (`--init-from`: its file, run,
  promoted index, samples and updates), or null.
- `config`: every setting, resolved, by table (as the config file groups them).
- `progress`: samples, updates, episodes, minutes, the gates' and ladders' minutes
  (`evaluation_minutes`, their share of the run) and how it ended (`budget`, `stop`).
- `segments` (schema 2, E06 T2b1): the run's first part and each `--resume`'s, in order: its batch
  seed, when it started, the commit and `dirty`, the device, the config keys it changed, its
  samples, updates and minutes at its start and end, how it ended (`budget`, `stop`, or `lost`: no
  record of its end, a kill or a crash) and its records dropped after its last checkpoint.
- `curve`: per gate interval, the means of its updates' losses, entropy, KL, clip fraction,
  explained variance, samples per second and training rates (schema 5: the learner's score
  against the league's members, `league_score`, and the frozen networks a call passed,
  `frozen_passes`, instead of the pool's win rate; from E06 T7c, once a run's updates have
  them, `by_kind`: per encounter kind the self-play episodes, the attacker's wins and the
  truncations, summed over the interval; a neutral kind's attacker is the learner against the
  system actor; from E06 T6b, with `[league] system_share`, `system`: per side of the learner
  its episodes against the system actor in the player kinds and its mean score there, a draw or
  a truncation ½).
- `gates` (schema 4): each gate's samples, update, the best it played, its result (`promoted`,
  `rejected`, `inconclusive`), pairs, the pairs' counts per score, LLR, mean score, Elo, games,
  seconds and (schema 5, E06 T4a) the league's member table at the gate (each member's id, kind,
  birth in samples, `p`, decayed and plain games, whether live; the last gate's is the end's); `promotions`: each promoted network's index, how (`start`, `init`, `gate`), samples
  and update; `ladders`: each ladder's promoted network, whether final, episodes per kind, and per
  opponent the win rate, its 95 % interval, the value efficiency (at reward v1's currency and
  price whatever the run trains with, from E06 T5's mana-price preparation; no run before it
  trained at others), each kind's rate and (after
  `t3b-1`) its truncations' share, and (E06 T7c, null before or without `ladder_neutral`)
  `neutral`, the same on the neutral set v1, attacking the system actor; `best`: the latest
  promotion's index and samples.

`tests/test_run.py` checks every record here against its schema (and every config in
`python/configs/`). To commit a run's record: copy `runs/<run>/record.json` to
`records/<run>.json`.

## The bots' tables

`bots-report-v1.json` (E06 T3a; design §7): Random, Aggressor, Greedy and (E06 T7c) the system
actor against each other on the report set v1, every ordered pair on its 1 500 battles: per kind
and side the games, wins, truncations and mean rewards, per pair the overall win rate with its
95 % interval and the value efficiency, the Glicko-2 ratings, and the engine's identity (the
encounter distribution's version, the rulesets' content hashes, the system actor's policy
version). `bots-neutral-v1.json` (E06 T7c; design §12): the same four, each attacking the system
actor on the neutral set v1's 1 500 battles (the attacker's rates alone, no ratings; `defender`
names the fixed seat). Both are `h3 ai eval --json`'s schema 2 and a function of the engine;
`tests/test_evaluate.py` checks their strata, their identity against the engine's and their
rates. Regenerate both when the bots, the system actor's policy (`POLICY_VERSION`) or the rules
move (from the repository's root, in release about 85 s and 15 s):

```sh
cargo run --release -p h3-cli -- ai eval --set report-v1 --json python/records/bots-report-v1.json
cargo run --release -p h3-cli -- ai eval --set neutral-v1 --json python/records/bots-neutral-v1.json
```

## The behaviour analyses

`inspect/<run>-<set>.json` (E06 T6a): `python -m h3env.train.inspect --checkpoint
runs/<run>/best.pt --set <set> --json …` on the run's best, per opponent the ladder's row and
the battles' groups (`h3env.train.inspect`; the findings in `docs/epics/E06-log.md`). The draws
are the device's: a CPU's differ from the GPU ladder's in a few battles. `t7-1`'s on both
stratified sets (the sandbox's CPU, 4 threads).

`inspect/<run>-order-probe.json` (E06, the turn-order probe, 2026-10-26): whether the run's
trunk knows which stack acts next, from `inspect/probe_order.py` (a one-off script, not a module:
`uv run --group train python records/inspect/probe_order.py runs/<run>/best.pt 600` from
`python/`; the method and the findings in `docs/epics/E06-log.md`).

## The search's measurements

`search/<run>-<set>-<N>[-<per kind>].json` (E06 T8c): `python -m h3env.search --report --best
runs/<run>/best.pt --opponent <seat> --set <set> --simulations <N> [--per-kind <n>] --json …`,
the search and the sampled policy on the same held-out episodes (the settings, both rows, the
search's costs; the findings in `docs/epics/E06-log.md`). `-fixedscale`: before the search's σ
rescaled the completed values (T8c), when the search played as the sampled policy. From E25 T1c
(`t7-1-*`, the findings in `docs/epics/E25-log.md`) a world scores a decided battle by reward v1
(T1a), and `-true` is the search over the true battle's copies (`--true-worlds --no-sampled`,
T1b; its sampled row is the run without the suffix, on the same episodes).

## Lineage

The promoted networks (from E06 T3b: a run's `promoted/<n>.pt`, its record's `promotions`) and
the run that holds the current best; a run started `--init-from` another's network names it as
its `parent`.

- `t3b-1` (E06 T3b's hour, 2026-10-20): promoted #0 to #4; its best #4 (25 M samples) at the
  bots' level (`docs/epics/E06-log.md`). Its networks are on the user's machine
  (`python/runs/t3b-1/`).
- `t4-1` and `t4-2` (E06 T4b, 2026-10-21 and 22; the league, 60 M samples each, γ 0.995 and γ
  1.0): promoted #0 to #4 and #0 to #5; `t4-2`'s best #5 (50 M samples) is the lineage's
  strongest (Random 0.839, Aggressor 0.578, Greedy 0.559 on the full report set). Their networks
  are on the user's machine (`python/runs/t4-1/`, `t4-2/`).
- `t5-1` (E06 T5b, 2026-10-22; the entity network at `t4-2`'s settings, stopped at 26.6 M of 60
  M): promoted #0 and #1 (10 M samples; Random 0.607, Aggressor 0.383, Greedy 0.372 on 250 per
  kind); 45 % of its self-play battles stalled to the step limit (`docs/epics/E06-log.md`, T5b).
  Its networks are on the user's machine (`python/runs/t5-1/`).
- `t5-2` and `t5-3` (E06 T5d, 2026-10-22 and 23; `configs/e06-t5c.toml`, the truncation penalty
  1.0, 20 M samples each): the entity network (best #1 from 10 M: 0.770, 0.470, 0.468 on the full
  report set; 250 minutes) and the MLP (best #2, 20 M: 0.832, 0.557, 0.533; 70 minutes). Their
  networks are on the user's machine (`python/runs/t5-2/`, `t5-3/`).
- `t7-1` (E06 T7d, 2026-10-23; `configs/e06-t7.toml`, a third neutral, 20 M samples, 79
  minutes): best #2 at 20 M, on the full report set 0.828, 0.551, 0.528 against Random,
  Aggressor, Greedy and 0.374 against the system actor; on the neutral set 0.972, value +0.245.
  Its networks are on the user's machine (`python/runs/t7-1/`). **The battle AI v1** (E06's
  closure, decision 7 as revised 2026-10-27) and the current best.
- `t6b-1`, `t6d-1`, `order-1` (E06 T6c, T6e, T6g, 2026-10-26 and 27; `configs/e06-t6b.toml`,
  `e06-t6d.toml`, `e06-order.toml`): A/B runs against `t7-1`'s settings, each from scratch with
  one lever (the system actor as an opponent in a quarter of the player kinds; the kill preview
  in the network; the turn order in the trunk), 20 M samples, best #2 each; none measurably
  better (`docs/epics/E06-log.md`), so none continues the lineage. Their networks are on the
  user's machine; `order-1`'s best played the search's measurements (`search/`).
- `e25-control-1`, `e25-shaping-1` (E25 T2b, 2026-10-28; `configs/e25-control.toml`,
  `e25-shaping.toml`: `t7-1`'s settings with the player kinds alone, the second with `[ppo]
  shaping`; 20 M samples, 73 and 90 minutes): best #2 each. The control on the full report set
  0.829, 0.548, 0.529 against Random, Aggressor, Greedy, 0.371 against the system actor, the
  neutral set +0.230; shaping 0.835, 0.589, 0.566, 0.409, +0.238, ahead of `t7-1` against every
  opponent, a little behind on the neutral set (`docs/epics/E25-log.md` T2b; whether it
  continues the lineage is E25 T3's). Their networks
  are on the user's machine.
