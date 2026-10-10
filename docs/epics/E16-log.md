# E16 log: done tasks in full

The full record of E16's finished milestones and tasks (what was built, agent decisions marked
provisional, measurements, the review lists). **Not start-of-session reading:** the epic file keeps
one line per done milestone; read the matching section here when a task builds on an earlier one
or a decision is questioned. Decisions live in ADR-0018 and the ADR drafts; this log only records
how and why the work went.

## Done

### M0 — The shared skeleton (2026-10-10)

- **The ADR.** The user accepted the protocol ADR draft at the checkpoint, asking for a trim
  first: 136 lines became 85, the twelve decisions unchanged; the cut text was already in the
  epic file (the measurements, the options) or the design note (§3 the manifest, §7 journals,
  §8 integrity). Numbered ADR-0018, moved to `docs/adr/` with its index row; the server lane's
  reading list in `AGENTS.md` gained it.
- **The crates.** `h3-protocol`: adapter layer, `logic = true`, the logic header, only
  `PROTOCOL_VERSION` (0 until M1 T1's first messages). `check-arch` accepted a logic crate in the
  adapter layer without a tool change (it checks purity by the `logic` flag and layers apart).
  `h3-server`: the binary `h3-server`, which prints its versions and exits 1 (SRV-STUB-1).
- **The dependencies.** The workspace gained `tokio` (`rt-multi-thread`, `net`, `time`, `sync`,
  `macros`), `tokio-tungstenite` 0.30 (`connect` only: no TLS), `futures-util` (`sink`, `std`;
  already in the graph through `tokio-tungstenite`; needed for `StreamExt`/`SinkExt` on the
  socket; *queued for the M1 review* as a light dependency), `getrandom` 0.3 (the version
  already in the lock), `postcard` (test only; *provisional*) and the path entries `h3-cli`,
  `h3-protocol`. `arch.toml` allows every edge M1 needs, so T1–T3 need no shared pull request;
  the crates declare a dependency when they use it. Checked in a throwaway worktree with all of
  them added: the build, `cargo deny check` (licences, bans, advisories, sources ok), 77 crates
  under `h3-server`, `serde_json` resolved with `alloc` only for `h3-protocol`. *Provisional:*
  `h3-server` → `h3-cli` for `load_ruleset`, `load_scenario` and `replay_to_toml`; the
  alternative is a battle-lane move of those into `h3-sim` (queued for the M1 review).
- **Docs.** The versions table (`PROTOCOL_VERSION`, `RECORD_FORMAT_VERSION` from M1,
  `JOURNAL_FORMAT_VERSION` from M2), `BACKLOG.md` E16 in progress, the global HANDOFF's server
  row, the README's crate table, `02-architecture.md`'s state paragraph, `benches/server/README.md`.
- **Found on the way.** The baseline verify on `master` failed `check-exe-map`: the adventure
  lane's E08 survey (CR-1 to CR-3) and ADV-Q3 to ADV-Q5 cite the H3API addresses of
  `GetMovementCost`, `CalcMaxMovement`, `MaxLandMovement` and `GetScoutingRadius`, which
  `docs/exe/map.toml` does not cover (the `test` step fails with it: xtask's
  `the_real_map_passes`); the merge was docs-only, so CI never ran on it. A map entry needs the function's size,
  which needs the executable (not staged in this session). The user left the fix to the adventure
  lane, which added the four map entries on `master` (`8eb412c`). Everything else passed on the
  M0 commit (coverage, determinism, ffi-smoke, bench-check, deny).
- **Also found.** The baseline's `test` step failed the battle property test
  `every_step_preserves_the_invariants` (a good-morale step without `HAD_MORALE` on the mover). Its
  inputs are random per run, so CI on `master` passed; the shrunk case reproduces in 5 s. Handed to
  the battle lane as a suggested task with the regression line; not committed here. The battle
  lane's `65a7a75` (the invariant allows a round's end after a lost turn) makes that case pass.
- **The merge with `master`** (E08 T2 and T3, E04b T8b) before the pull request merged: the
  adventure lane had accepted ADR-0015 to ADR-0017 first, so the protocol ADR became ADR-0018
  (only the references this branch added were renumbered); conflicts in the workspace path
  entries, the versions table, `02-architecture.md`, the ADR index and the global HANDOFF's rows.

### M1 T1 — Messages and codec (2026-10-10)

- **What was built** (`crates/h3-protocol`, all *provisional* in names and layout): `wire` (the
  published language: `ClientMessage` and `ServerMessage`, the battle `Command`, `Event`,
  `Observation` with `Stack`, `Hero`, `OwnHero` and `SideView`, the `Manifest` with `KeyTables`
  and `Field`, `RejectCode`, `Refusal`, `GoodbyeReason`, `WireU64` for 64-bit values as decimal
  strings), `codec` (one JSON document per frame, the size checked before parsing,
  `DEFAULT_MAX_CLIENT_FRAME` 4096 bytes, decode errors `TooLarge` and `Malformed`) and `map`
  (wire ↔ engine). `PROTOCOL_VERSION` is 1.
- **Projection by construction.** `map::observe` and `map::events` take the viewer and call the
  engine's `observe` and `project`; the raw observation conversion is crate-private, so the
  session core (T2) cannot send an unprojected observation or event.
- **The mapping is exhaustive (the user, ADR-0018 point 5 amended).** The first build had the
  ADR's wildcard arms to `Unsupported`/`Other`; llvm-cov counted each as an uncovered region
  (measured on a scratch crate and on the real one: 6 arms, 9 regions, 14 lines), so they
  needed a file exclusion in the shared `coverage-exclusions.toml` (`#[coverage(off)]` is
  still unstable on 1.97). Asked where the exclusion should land, the user chose no exclusion:
  exhaustive matches, a new engine variant fails the build until mapped; the `Unsupported` and
  `Other` wire variants went with the arms. The global HANDOFF names the trap for the battle
  lane. Engine enums matched: `Command`, `EventKind` (47 kinds), `CommandError` (29),
  `CastTarget`, `DamageRoll`, `ObstacleKind` (`h3-content`); the contract enums `BattleSide` and
  `BattleResult` are a seam anyway.
- **Choices for the review** (the epic's review list): `Refused` for a refused join, the
  session's codes `NotSeated` and `NotStarted`, the manifest in `Joined` rather than `Welcome`
  (ADR-0018 point 10 says "at join"), flag sets as named booleans, hexes as indices with the
  field's size in the manifest, the observation as the sim encoding's values (attack without
  the melee bonus, as the encoding; the own hero's spell bonuses not yet on the wire), `postcard`
  as a dev-dependency for the round trips.
- **Key tables.** Every kind has the handle-to-key direction without a battle-lane ask:
  creatures through `creature_count` and `creature_key`, the others through their slices'
  `key` fields in handle order.
- **Tests** (18): every message round-trips in JSON and `postcard`; the exact JSON forms; 64-bit
  values as strings and their refusals; unknown fields refused (a command naming a side does
  not decode); the size limit before parsing; malformed frames, deep nesting; arbitrary bytes
  and JSON-like text never panic (proptest); every engine variant maps to the wire variant of
  the same name; commands round-trip; events projected; the observation's every part;
  the legal list validates in the engine; castable spells and their targets; key tables.
  `h3-protocol` at 100 % lines and regions; `verify --scope server` with no failing step.
