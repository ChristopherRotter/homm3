# Benchmark baselines

Instruction-count baselines for `cargo xtask bench-check` (`docs/05-testing.md`, "Gates"), one
file per workload: `benches/<lane>/<workload>.toml`. The directory is the owning lane
(`lanes.toml`), so each lane's +2 % gate is its own and two lanes never re-baseline the same file.

- `cargo xtask bench-check` measures every workload; `--lane <lane>` or `--workload <name>`
  narrows it; `cargo xtask verify --scope <lane>` measures the lane's workloads only.
- `cargo xtask bench-check --update --workload <name>` re-baselines one workload; give the reason
  in the commit message (`docs/lanes/battle/HANDOFF.md` has the battle lane's traps about it).
- A new lane adds its workloads under its own directory. Workloads run the release `h3` binary
  (`crates/h3-cli`) under callgrind and count only inside the function matched by `collect`.

| Lane | Workload | Measures |
|---|---|---|
| battle | `random-rollouts-full` | 20 random-vs-random battles on `packs/synthetic/scenarios/full.toml`, inside `h3_sim::rollout::random_rollout` |
