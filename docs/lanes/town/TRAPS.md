# Town lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Working in the adventure lane's crates

- **Ownership is by the longest match** (`lanes.toml`): only files whose path starts with `crates/h3-adventure/src/town`, `crates/h3-game/src/town`, `crates/h3-map/src/import/town` or `crates/h3-map/src/writer/town` are this lane's. A `mod town;` line in a `lib.rs`, a field in a state struct, a hash, a version constant, a replay or a bot report is the adventure lane's: such a pull request is `cross-lane` (the label at once), and `cargo xtask check-lane --cross-lane` is its local check.
- **Merge in order:** before a task starts and before a pull request opens, look at the adventure lane's open pull requests (its HANDOFF). When both lanes bump the same version, the second rebases and bumps again, and re-records the replays a second time (skill `homm3-rerecord-replays`).
- **The adventure lane's traps apply** to its crates (forks without deep copies, server-only RNG seeds, template masks, staging the shipped maps): its `TRAPS.md`.
- **The scoped verify** `cargo xtask verify --scope town --summary` also runs the adventure lane's crates, replays and benchmark workloads (`verify_with`); its `check-lane` step fails by design on a `cross-lane` branch.
