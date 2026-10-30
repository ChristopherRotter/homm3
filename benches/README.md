# Benchmark baselines

Instruction-count baselines for `cargo xtask bench-check` (`docs/05-testing.md`, "Gates"), one
file per workload: `benches/<lane>/<workload>.toml`. The directory is the owning lane
(`lanes.toml`), so each lane's +2 % gate is its own and two lanes never re-baseline the same file.

- `cargo xtask bench-check` measures every workload but the lane-only ones (below); `--lane <lane>` or `--workload <name>`
  narrows it; `cargo xtask verify --scope <lane>` measures the lane's workloads only.
- `cargo xtask bench-check --update --workload <name>` re-baselines one workload; give the reason
  in the commit message (`docs/lanes/battle/HANDOFF.md` has the battle lane's traps about it).
- A new lane adds its workloads under its own directory. Workloads run a release binary under
  callgrind (`h3` of `crates/h3-cli` unless the file names `package` and `bin`) and count only
  inside the function matched by `collect`.
- A workload whose count depends on another lane's code is `lane_only = true`: measured by
  `--lane <lane>`, `--workload <name>` and `verify --scope <lane>`, never by the full verify (CI),
  so that the other lane's changes cannot fail it there. Its lane checks it at the start of each
  session and re-baselines a drift that `master` brought in, naming the cause in the commit.

| Lane | Workload | Measures |
|---|---|---|
| battle | `random-rollouts-full` | 20 random-vs-random battles on `packs/synthetic/scenarios/full.toml`, inside `h3_sim::rollout::random_rollout` |
| battle | `random-rollouts-siege` | 60 random-vs-random battles on `packs/synthetic/scenarios/siege.toml` (a Citadel: walls, gate, damaging moat, keep, the Catapult), inside `h3_sim::rollout::random_rollout` (E04c T9b) |
| battle | `computer-decisions` | the original's battle AI: 40 battles of random harvested SoD armies, the defender the computer against random commands on the open field, both sides the computer at a Fort, inside `h3_battle_ai::choose` (`h3 bench-computer`; E23 T2b, ADR-0027 point 11) |
| adventure | `random-play-duel-36` | 5 random-vs-random games of 28 days on `fixtures/maps/e08-duel-36.h3m`, inside `h3_sim::adventure::random_rollout` (`h3 adventure bench --workload play`) |
| adventure | `reach-xl-144` | every walking hero's reachable-tiles query of the acting player at each step of 2 random games of 28 days on `fixtures/maps/e08-xl-144.h3m`, inside `h3_cli::adventure::bench::reach_queries` (E08 T42: the walkers only) |
| adventure | `reach-lifted-xl-144` | the same games' queries of the flying and water-walking heroes, inside `h3_cli::adventure::bench::lifted_queries` (E08 T42) |
| adventure | `fork-xl-144` | 100 000 forks of the env after 7 days of random play on the same map, inside `h3_cli::adventure::bench::fork_workload` |
| adventure | `fork-meeting-xl-144` | the same forks with a meeting open (a player's heroes seek each other until one opens: `--meeting 1`; E08 T52), inside `h3_cli::adventure::bench::fork_workload` |
| adventure | `manage-army-12` | 250 times both heroes' management listing in a meeting on `fixtures/maps/e08-army-12.h3m` (`h3_game::legal_management` and every split range between the two armies; `--workload manage`, E08 T52), inside `h3_cli::adventure::bench::management_queries` |
| adventure | `random-play-rewards-36` | 5 random-vs-random games of 28 days on `fixtures/maps/e08-rewards-36.h3m` (every object class of increment 7; E08 T64a), inside `h3_sim::adventure::random_rollout` |
| adventure | `fork-rewards-36` | 100 000 forks of the env after 7 days of random play on the same map, the forks carrying the visits block (E08 T64a, decision 133), inside `h3_cli::adventure::bench::fork_workload` |
| adventure | `fork-boxes-xl-144` | 100 000 forks of the env after 7 days of random play on `fixtures/maps/e08-boxes-144.h3m` (decision 133: 144 × 144 with an underground, a Pandora's Box on every free tile; E08 T64b), inside `h3_cli::adventure::bench::fork_workload` |
| adventure | `fork-stones-xl-144` | the same on `fixtures/maps/e08-stones-144.h3m` (a Learning Stone on every free tile), the forks carrying the visits block (E08 T64b), inside `h3_cli::adventure::bench::fork_workload` |
| adventure | `random-play-sea-36` | 5 random-vs-random games of 28 days on `fixtures/maps/e08-sea-36.h3m` (a strait with boats, the sea's pickups, events and a monster on water; E08 T75a), inside `h3_sim::adventure::random_rollout` |
| adventure | `reach-sea-xl-144` | every hero in a boat's reachable-tiles query of the acting player at each step of 2 random games of 56 days on `fixtures/maps/e08-sea-144.h3m` (144 × 144 of sea, sixteen islands, fifteen boats; E08 T75b), inside `h3_cli::adventure::bench::sail_queries` |
| adventure | `fork-sea-xl-144` | 100 000 forks of the env after 7 days of random play on the same map, the forks carrying the boats with a hero aboard (E08 T75b), inside `h3_cli::adventure::bench::fork_workload` |
| server | `session-random-play-full` (lane-only) | 20 hosted battles on the same scenario, two scripted clients through `h3-server-bench`, inside `h3_server::bench::serve_frame` (decode, validate, apply, project, encode; the battle engine included) |
| ai | `batch-random-full` (lane-only) | a training batch of 64 envs of `packs/synthetic/scenarios/full.toml`, both seats external, one thread (`--threads 0`), 1 000 calls of random legal actions from the batch's own masks (casts through the head calls), inside `h3_sim::batch::BattleBatch::step` (`h3 ai bench`; E05 T3c) |
| ai | `batch-random-siege` (lane-only) | the same on `packs/synthetic/scenarios/siege.toml` (E05 T3c) |
| ai | `batch-random-encounters-v1` (lane-only) | the same on the encounter distribution v1 at its default weights, each automatic reset's sampled encounter included (`h3 ai bench --encounters default`; E05 T5b2) |
