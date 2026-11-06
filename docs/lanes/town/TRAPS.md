# Town lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Working in the adventure lane's crates

- **Ownership is by the longest match** (`lanes.toml`): only files whose path starts with `crates/h3-adventure/src/town`, `crates/h3-game/src/town`, `crates/h3-map/src/import/town` or `crates/h3-map/src/writer/town` are this lane's. A `mod town;` line in a `lib.rs`, a field in a state struct, a hash, a version constant, a replay or a bot report is the adventure lane's: such a pull request is `cross-lane` (the label at once), and `cargo xtask check-lane --cross-lane` is its local check.
- **Merge in order:** before a task starts and before a pull request opens, look at the adventure lane's open pull requests (its HANDOFF). When both lanes bump the same version, the second rebases and bumps again, and re-records the replays a second time (skill `homm3-rerecord-replays`).
- **The adventure lane's traps apply** to its crates (forks without deep copies, server-only RNG seeds, template masks, staging the shipped maps): its `TRAPS.md`.
- **The scoped verify** `cargo xtask verify --scope town --summary` also runs the adventure lane's crates, replays and benchmark workloads (`verify_with`); its `check-lane` step fails by design on a `cross-lane` branch. It does not run the other lanes' tests (`h3-cli`'s ai census, E09 T8a1's CI failure): a change to the setup's checks or the static map also runs `cargo test --workspace --exclude xtask --all-targets`.
- **The adventure benchmarks follow the heap:** a change that only adds a few small allocations
  at the build (the town part's content) moved `day-start-xl-144` by +1.6 % through glibc's
  `_int_malloc` alone (E09 T8c2). Compare per function with `master` built in the same sandbox
  first (skill `homm3-bench-regression`); with every counted function the same, re-baseline the
  workload with the reason in its description.

## The import (`h3-map` `src/import/town.rs`)

- **A town's entrance is blocked and visitable** in the shipped maps' templates: a test "does a town block this tile" also hits the entrance, where a moved hero (TWN-SET-010) and a town's visitor stand. Use `town::in_town` (a blocked tile that is not an entrance), not `blocks` (E09 T7a1, found by the census).
- **Every struct literal of `GameSetup`, `PlayerSetup` and `GameHeroSetup`** lives in other lanes' crates too (`h3-cli`'s `src/ai/census.rs`, the ai lane's; `h3-map`'s examples): a new field is a `cross-lane` change there.

## The game's setup (`h3-game` `src/town*`, `setup.rs`)

- **A setup's towns match its map's** (E09 T8a1, `GameSetupError::TownCount`): a `GameSetup` whose static map has objects of the type `town` needs one `TownSetup` per such object, in object order; a test or tool that builds a game from a static layer alone gives each a neutral town record (`h3-map`'s census example and `h3-cli`'s `src/ai/census.rs` do).
