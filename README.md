# h3 — a Heroes of Might and Magic III rules engine

A headless, deterministic, high-performance rules engine for **Heroes of Might and Magic III**
(Restoration of Erathia, Armageddon's Blade, Shadow of Death), written in Rust. It is built as the
simulation backend for **reinforcement-learning self-play**: millions of battles and adventure-map
games, reproducible bit for bit, with a state-of-the-art AI trained on top. Playable clients,
a game server and a training pipeline are being built around the engine, outside the core.

Licensed **GPL-2.0-or-later** ([LICENSE](LICENSE), [ADR-0009](docs/adr/0009-license-and-reference-material.md)).
`h3` is a working name ([OPEN-QUESTIONS Q2](docs/OPEN-QUESTIONS.md)).

> **Not affiliated with or endorsed by** Ubisoft or any other owner of the *Heroes of Might and
> Magic* trademarks and copyrights. The repository contains no game assets, graphics,
> sound, maps or flavour text. Parts of the toolchain (the harvester, the code-reading tools, the
> clients) work on **your own legitimate copy of the game**, which stays outside the repository
> (`original-data/` is git-ignored and guarded by a gate). The committed `packs/` hold gameplay data
> (numbers and names) harvested from the game's tables and labelled public sources; the tests use
> invented, synthetic content.

## Why this engine is different

- **Faithful to the original.** Every game rule has a rule ID, a labelled source (original data, a
  reading of the executable, the wiki, VCMI, an observation of the original game) and a test.
  Known bugs of the original are tracked in a quirks ledger and selectable as profiles
  (`faithful`, `strict-fixed`); exploits and state-corrupting bugs are always fixed.
- **Deterministic.** Same ruleset, initial state, seed and command sequence give bit-identical
  state and events on every platform. No `HashMap`, no clocks, no threads, no ad-hoc floating point
  in the logic crates; golden replays are checked in debug, release and native builds.
- **Fast and cheap to fork.** The logic core is allocation-light and built for training loops: the
  first walking skeleton measured about 245 000 random battle steps per second per core; current
  instruction-count baselines are gated in [`benches/`](benches/README.md).
- **Server-authoritative and validated.** Every command is checked against the state and the
  trusted actor; observations are per-viewer projections that hide what a player must not see.
- **Data-driven.** Creatures, heroes, spells, artifacts and rules parameters come from content
  packs (`base`, `ab`, `sod`), which toggle like the original's expansions.
- **Mechanically enforced architecture.** Bounded contexts (battle, adventure map, game
  orchestration, …), pure logic crates and adapters are checked by `cargo xtask check-arch`;
  100 % line and region coverage on the logic crates; property tests and determinism replays.

## Status

The project is developed incrementally, in parallel work lanes ([docs/BACKLOG.md](docs/BACKLOG.md)
has the epics in order, each lane's state is in `docs/lanes/<lane>/HANDOFF.md`). As of 2026-10-24:

| Area | State |
|---|---|
| Battle: heroes, secondary skills, artifacts, specialties, battle magic, sieges, special battlefields | done (E03, E04a–c) |
| Content pipeline: packs `base`/`ab`/`sod` harvested from the game's tables, synthetic test packs | done (E02) |
| The original's battle AI (the computer's tactical policy), read from the executable | done (E23) |
| Battle environment for RL: batch API, C ABI, Python binding (`h3env`), encounter distribution | done (E05) |
| Training: PPO self-play, league, evaluation gate, ladders against scripted bots | in progress (E06) |
| Adventure map core on real `.h3m` maps: movement, fog of war, objects, monsters, heroes, army management | in progress (E08) |
| Server: protocol, lobbies, accounts, ranked ladders, discovery | in progress (E16; M0–M4, M6 done) |
| Battle client (SDL 3, assets from your own copy; frame-exact comparison against the original) | in progress (E17) |
| Adventure map client | in progress (E24; stills done) |
| Next: historical rules of RoE and AB (E11); then towns and economy, adventure AI, map generator, campaigns | planned |

## Quick start

Requirements: Rust (the pinned toolchain in `rust-toolchain.toml` installs itself via `rustup`), a C
compiler, and for the full gate `cargo install cargo-deny cargo-llvm-cov --locked`,
`rustup component add llvm-tools-preview` and valgrind (Linux). No game files are needed for the
default build and tests.

```sh
cargo xtask verify                 # all gates (docs/05-testing.md); --quick for the inner loop
cargo xtask verify --scope battle --summary   # one lane's loop, one line per gate
cargo run -p h3-cli -- run --scenario packs/synthetic/scenarios/skirmish.toml --seed 7 --record /tmp/r.toml
cargo run -p h3-cli -- verify /tmp/r.toml fixtures/replays/*.toml
cargo run --release -p h3-cli -- bench
```

The Python side (training, evaluation) is a [`uv`](https://docs.astral.sh/uv/) project in
[`python/`](python/README.md); the clients are a separate Cargo workspace in
[`clients/`](clients/README.md). Steps that need the original game (harvesting packs,
`cargo xtask verify --with-original-data`, the code-reading tool `scripts/exe.py`, the clients and
the capture harness) read your installation from a copy in `original-data/`
([docs/formats/installation.md](docs/formats/installation.md), [docs/exe/README.md](docs/exe/README.md)).

## Layout

| Path | What |
|---|---|
| `crates/h3-kernel` | ids/handles, fixed-capacity storage, PRNG streams, canonical hashing, `kernel::num` |
| `crates/h3-content`, `h3-content-battle` | content definitions and the pure `Ruleset` compiler: the core, and the battle's part |
| `crates/h3-contracts` | seam types between contexts (`BattleSetup`, `BattleOutcome`, …) |
| `crates/h3-battle`, `h3-battle-ai` | the battle context; the original's battle AI as a policy |
| `crates/h3-heroes` | the hero model beneath the contexts (levels, skills, artifacts, derived battle heroes) |
| `crates/h3-adventure`, `h3-game` | the adventure map context; game orchestration (players, turn groups, pending decisions, battles on the map) |
| `crates/h3-map`, `h3-pack`, `h3-pack-battle`, `h3-pack-adventure` | the `.h3m` reader, writer and import; pack adapters |
| `crates/h3-sim`, `h3-ffi` | RL environments, action/observation encodings, replays; the C ABI (`include/h3.h`) |
| `crates/h3-protocol`, `h3-server` | the wire protocol (sans-I/O core) and the WebSocket game server |
| `crates/h3-formats`, `h3-harvest`, `h3-experiment` | readers for the original's files; the pack harvester; experiment compiler (staged battles to confirm rules in the original) |
| `crates/h3-cli` | the `h3` command-line tool (runs, replays, benchmarks, evaluation) |
| `xtask` | `cargo xtask verify` and the individual gates (`check-arch`, `check-rules`, `check-lane`, …) |
| `lanes.toml` | which paths each work lane owns ([docs/08-work-process.md](docs/08-work-process.md), "Lanes") |
| `python/` | `h3env`: the batch environment from Python and the training pipeline |
| `clients/` | playable clients, a separate workspace outside the core |
| `packs/`, `fixtures/`, `benches/`, `experiments/` | content packs, golden replays and maps, benchmark baselines, experiment specs and results |
| `scripts/` | cloud-sandbox setup and verification helpers; `exe.py`, the code-reading tool for `Heroes3.exe` |
| `docs/` | vision, architecture, ADRs, rule catalog, backlog; [`docs/README.md`](docs/README.md) is the index |

## Documentation

Start with [docs/01-vision.md](docs/01-vision.md) (goals and non-goals),
[docs/02-architecture.md](docs/02-architecture.md) and [docs/03-domain.md](docs/03-domain.md)
(bounded contexts and the ubiquitous language). Decisions are recorded as
[ADRs](docs/adr/README.md); rules and their sources in [docs/rules/](docs/rules/) and
[docs/04-rules-and-sources.md](docs/04-rules-and-sources.md); the testing gates in
[docs/05-testing.md](docs/05-testing.md).

## How the project is developed

Most of the code is written by AI coding agents working in parallel *lanes* (battle, adventure,
AI, server, client, map-client) under a human maintainer who confirms seams, ADRs and rule
decisions. The agents' entry point is [AGENTS.md](AGENTS.md); the process is described in
[docs/08-work-process.md](docs/08-work-process.md). Human contributors are welcome too: see
**[CONTRIBUTING.md](CONTRIBUTING.md)**; security issues are reported privately
([SECURITY.md](SECURITY.md)).

## Acknowledgements

[VCMI](https://github.com/vcmi/vcmi) (GPL-2.0-or-later) is read as a reference and may be ported
with an origin comment; the HoMM3 wiki and other community documentation are used as labelled sources. Determinism follows the discipline described in Factorio's
[FFF-52](https://factorio.com/blog/post/fff-52).
