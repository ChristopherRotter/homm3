# h3 — a Heroes of Might and Magic III rules engine

A headless, deterministic, high-performance rules engine for Heroes of Might and Magic III (RoE,
AB, SoD), written in Rust, built as a simulation backend for reinforcement-learning self-play.
Working name `h3` (OPEN-QUESTIONS Q2). Licensed GPL-2.0-or-later (`LICENSE`, ADR-0009). No
original game assets or text are included; tests use invented, synthetic content.

**Agents and contributors: start with [AGENTS.md](AGENTS.md) and [docs/HANDOFF.md](docs/HANDOFF.md).**

## Status

E01 (walking skeleton) is done and verified on Linux and Windows: a small battle runs end to end — content pack → ruleset →
validated commands → events/observations → simulation environment / C ABI / CLI replays —
deterministically, at ≈ 245 k steps/s per core, behind a single quality gate.

## Quick start

```sh
cargo xtask verify                 # all gates (see docs/05-testing.md); --quick for inner loop
cargo xtask verify --scope battle --summary   # a lane's local loop, one line per gate
cargo run -p h3-cli -- run --scenario packs/synthetic/scenarios/skirmish.toml --seed 7 --record /tmp/r.toml
cargo run -p h3-cli -- verify /tmp/r.toml fixtures/replays/*.toml
cargo run --release -p h3-cli -- bench
```

Tools for the full gate: `cargo install cargo-deny cargo-llvm-cov --locked`,
`rustup component add llvm-tools-preview`, a C compiler, and valgrind (Linux).

## Layout

| Path | What |
|---|---|
| `crates/h3-kernel` | ids/handles, fixed-capacity storage, PRNG streams, canonical hashing, `kernel::num` |
| `crates/h3-content` | content definitions and the pure `Ruleset` compiler |
| `crates/h3-contracts` | seam types between contexts (`BattleSetup`, `BattleOutcome`) |
| `crates/h3-battle` | the battle context |
| `crates/h3-pack` | TOML content packs and scenarios (adapter) |
| `crates/h3-sim` | RL environment, action/observation encodings, replays (adapter) |
| `crates/h3-ffi` | C ABI (`include/h3.h`) |
| `crates/h3-cli` | `h3` command-line tool |
| `crates/h3-formats` | readers for the original game's files (LOD, TXT, exe tables), used by the harvester, and the `.h3m` writer for experiment maps (adapter) |
| `crates/h3-harvest` | harvester: original files → committed packs `base`/`ab`/`sod` (tool) |
| `crates/h3-experiment` | experiment compiler: spec → `.h3m` map + results template for the user (tool) |
| `xtask` | `cargo xtask verify` and the individual gates, `check-lane` |
| `lanes.toml` | which paths each work lane owns (`docs/08-work-process.md`, "Lanes") |
| `clients/` | playable clients: a separate Cargo workspace, outside the core (`clients/README.md`) |
| `scripts/` | cloud-sandbox setup and background verify for agent sessions (`docs/08-work-process.md`) |
| `packs/`, `fixtures/`, `benches/` | harvested and synthetic content packs, golden replays, benchmark baselines |
| `docs/` | vision, architecture, ADRs, rule catalog, backlog, handoff; `docs/lanes/` per lane |
