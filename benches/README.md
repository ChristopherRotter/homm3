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
| server | `session-random-play-full` (lane-only) | 20 hosted battles on the same scenario, two scripted clients through `h3-server-bench`, inside `h3_server::bench::serve_frame` (decode, validate, apply, project, encode; the battle engine included) |
