# 02 — Architecture

Status: upfront decisions only. Everything not stated here is deliberately left open and will be refined per epic. The binding decisions are in `adr/`; this document explains how they fit together.

## 1. Principles

1. **Pure, deterministic logic core** (ADR-0003). Logic crates contain no I/O, time, threads, globals, unordered iteration or ad-hoc floating point.
2. **Domain first** (ADR-0005). Code is organized by bounded context with the ubiquitous language from `03-domain.md`. Technical layers serve the domain, not the other way round.
3. **Commands in, events out, state is truth** (ADR-0004). Every mutation goes through one validated entry point.
4. **Data-driven within limits** (ADR-0006). Content is data compiled into an immutable, dense `Ruleset`; behavior variation is parameters and a closed set of composable primitives.
5. **Performance by construction.** Data layout and allocation behavior are design inputs, not afterthoughts. Observability (tracing) is zero-cost when disabled.
6. **Security by construction** (ADR-0008). Trust boundaries are part of the type structure; validation cannot be bypassed.
7. **Architecture is mechanically enforced**, not merely documented (`cargo xtask check-arch`).
8. **Grow organically.** Smallest design that satisfies the current epic, inside the guardrails above.

## 2. Layers and dependency direction

```
 adapters (I/O, untrusted input, other languages)
   formats/import   sim (RL env)   ffi (C ABI)   cli   server (later)   clients (later, external)
        │               │            │            │
        ▼               ▼            ▼            ▼
 application
   game  — session/orchestration: turns, command routing, contexts wiring, visibility/projection
        │
        ▼
 domain contexts (each independent of the others)
   battle   adventure   town   heroes(+army)   magic   …
        │
        ▼
 shared
   contracts (seam types between contexts)   content (definitions + ruleset compiler)   kernel (ids, numerics, rng, hashing, bonus primitives)
```

Rules:

- Dependencies point **downward only**. Adapters depend on application/domain, never the reverse.
- **Domain contexts do not depend on each other.** They interact through types in `contracts` and through the `game` orchestrator (e.g., adventure emits a `BattleRequest`; `game` runs the battle context; the `BattleOutcome` is applied back to adventure state).
- Logic crates (kernel, content model, contexts, contracts, game) are I/O-free. Reading files, parsing original formats, networking, FFI and anything user-facing are adapters.
- Untrusted input (map files, archives, network messages) enters only through adapters that validate and convert into domain types.

## 3. Crate layout (provisional names; create crates only when the context exists)

```
crates/
  h3-kernel/       handles, FixedVec, PRNG + streams, canonical state hashing, kernel::num, order, arith; modifier primitives (ADR-0012)
  h3-contracts/    seam types: BattleSide, Army, BattleSetup, BattleOutcome, ...
  h3-content/      definition model (creatures, ...), content-pack model, quirk registry, Ruleset compiler (pure: takes parsed data)
  h3-battle/       battle context
  h3-heroes/       heroes context: HeroDetails, their validation, war machines; the translation to BattleHero (E04a T4)
  h3-pack/         content-pack and scenario text format (TOML) → content definitions / BattleSetup   (adapter, E01)
  h3-adventure/    adventure context: map, heroes on the map, movement, objects, fog of war; its ruleset part (E08, ADR-0015/0016)
  h3-town/ h3-magic/                                (later, split out when they earn it)
  h3-game/         application: the whole game's setup and state, players, turn groups, calendar, RNG streams, routing, projections (E08, ADR-0017)
  h3-map/          .h3m reader and the import into setup types (adapter, E08); the writer moves here after E04b
  h3-pack-adventure/  packs/*/adventure.toml → the adventure's definitions (adapter, E08)
  h3-formats/      readers for original files: LOD archives, TXT tables, exe static tables (adapter, E02); H3M writer for experiment maps (E04a T2; reader later)
  h3-harvest/      one-off harvester: original files → committed TOML packs base/ab/sod (tool, E02, Q13)
  h3-experiment/   experiment compiler: spec → SoD .h3m + results template (tool, E04a T2, Q28)
  h3-sim/          RL environment API (batched, fork/clone, masks, observations)
  h3-ffi/          C ABI
  h3-cli/          headless runner, replay tool
  h3-protocol/ h3-server/   wire protocol and hosting (adapters, server lane, E16)
xtask/             verify, check-arch, check-rules, coverage, bench gates, check-lane
clients/           playable clients: a separate Cargo workspace (own lockfile, own CI job), asset decoding and rendering; never a dependency of the core (clients/README.md)
```

Work on the crates is split into lanes (battle, adventure, server, client) that own paths (`lanes.toml`, `docs/08-work-process.md`, "Lanes"); the layers above are unchanged by it.

Start with the smallest set the walking skeleton needs (kernel, content, battle, sim, cli, xtask, plus minimal game/contracts if required). Split crates when a context actually exists — but the dependency rules apply from day one.

**State after E01:** `h3-kernel`, `h3-content`, `h3-contracts`, `h3-battle` (logic, `no_std` + `alloc`, 100 % coverage gate); `h3-pack`, `h3-sim`, `h3-ffi`, `h3-cli` (adapters); `xtask`. No `h3-game` yet: the sim drives the battle context directly through `BattleSetup`. **E02 adds** `h3-formats` (adapter) and the `h3-harvest` tool; the engine never reads original files at run time — it loads the committed packs (Q13). **E04a adds** `h3-heroes` (domain, logic; T4a: hero details and their validation, never a dependency of `h3-battle`, ADR-0013). **E08 adds** `h3-adventure` (domain, logic), `h3-game` (the first application crate, a logic crate), `h3-map` and `h3-pack-adventure` (adapters); a game's rules are the core ruleset plus one compiled part per context (ADR-0015; ADR-0006's amendment). **E16 M0 adds** `h3-protocol` (adapter, logic: the wire and the sans-I/O session core) and `h3-server` (adapter, the binary; ADR-0018). The allowed graph is `arch.toml`; ADR-0005's amendments list what `check-arch` enforces.

## 4. State, commands, events

Detailed in ADR-0004. In short:

- `apply(state, ruleset, actor, command, sink) -> Result<(), CommandError>` is the only mutation path. `actor` is supplied by the trusted session layer, never by the payload. Commands carry **intent only**; all derived values (prices, amounts, ownership) come from state. On error the state is unchanged.
- Events are an **output** (for clients, logs, animation, replays). Events carry a visibility scope. The state, not the event log, is the source of truth.
- `observe(state, ruleset, viewer) -> Observation` projects hidden information away. `legal_commands(state, actor)` enumerates/masks legal commands and must agree with `apply` (property-tested).
- **State layout for cheap clone/fork** (needed for search and RL): flat, index-addressed storage with stable IDs; immutable bulk data (terrain, map layout, ruleset) shared via `Arc`; mutable parts small and contiguous; battle state sized with fixed-capacity arrays where possible (no per-step heap allocation).
- **Replays** = ruleset id + initial scenario + seed + command log (+ optional expected state hashes). **Saves** = our own state serialization (versioned). Original save import is a separate future adapter.

## 5. Determinism (summary of ADR-0003)

Integers by default; IEEE-754 floats allowed only through the audited `kernel::num` surface (basic ops and `sqrt`; transcendentals only via in-repo implementations; no fast-math; supported targets x86_64/aarch64/wasm32); in-repo PRNG with documented algorithm and stream-splitting; no `HashMap`/`HashSet` in logic crates; sorts always have a unique tie-breaker; fixed-width integers in anything serialized or hashed; state hash that is canonical and versioned; CI replays fixture games on each supported platform and build configuration and compares golden hashes. Model: Factorio's deterministic float logic with custom trig and determinism integration tests ([FFF-52](https://factorio.com/blog/post/fff-52)) and per-tick whole-state CRC for desync detection ([FFF-47](https://factorio.com/blog/post/fff-47)).

## 6. Configuration and content (summary of ADR-0006)

- **Tier 1 — data:** stats, costs, buildings, town/hero/creature definitions, which content packs are active.
- **Tier 2 — parameters:** formula constants, limits, quirk toggles, profile selection.
- **Tier 3 — composable behavior:** abilities/effects built from a closed set of typed primitives (extendable in code by adding variants, not by runtime scripting).
- Content is authored as human-editable text files, validated, then compiled once into an immutable `Ruleset` with dense integer handles. The hot path never parses or looks up strings.
- **Architecture wins:** if a configurability feature costs measurable hot-path performance or needless complexity without an immediate gain, it is dropped or deferred and the decision is recorded.

## 7. Extension model (direction only; details refined in a later epic)

An *extension* (AB, SoD, later WoG/HotA) is a **content pack** (Tier 1–3 data) and optionally a **rule module** (Rust code implementing well-defined hook points, compiled in via Cargo features/registration; no dynamic loading initially). The base game is itself a content pack + the core rules. Guardrails so later extensions don't force a rewrite are in ADR-0006 (no magic numbers for map size, slot counts, levels, town counts, resources; IDs are namespaced data).

## 8. Performance approach

- Hot-path rules: no allocation in `apply`/`legal_commands` for battle; preallocated/fixed-capacity structures; dense handles instead of strings/maps; branch-light integer math.
- Tracing/explain mode is a generic `Sink`/`Trace` parameter monomorphized to a no-op when unused.
- Logic is single-threaded per game instance and `Send`; parallelism comes from running many instances.
- **Targets (measured in E01, 2026-09-30, and again at E04b T9b, 2026-10-11, on the casting workload: heroes with every kind of spell, creature magic, opening spells, Eagle Eye)** — hardware: cloud vCPU, Intel Xeon @ 2.80 GHz, 1 core used; release profile; synthetic `full` scenario (7 stacks per side):

  | Measure | E01 | E04b T9b | Target / threshold |
  |---|---|---|---|
  | Random legal play, `legal_commands` + agent pick + `apply` (`h3 bench --games 2000`) | ≈ 245 k steps/s (≈ 4.1 µs/step) | ≈ 104 k steps/s (≈ 9.6 µs/step); ≈ 69 k with the flat observation encoded before each step (`--observe 1`) | ≥ 100 k steps/s/core (met) |
  | criterion `step = legal_commands + pick + apply` (mid-game states) | 4.7 µs | 5.3 µs | — |
  | criterion `legal_commands` alone | 2.6 µs | 3.2 µs | — |
  | criterion `BattleEnv` step via action mask (mask 449 actions + decode + apply) | 6.8 µs | 2.4 µs (from a fresh reset) | — |
  | criterion `observe` / `observe` + flat encoding (8 308 floats, encoding v13) | — | 1.0 µs / 3.5 µs | — |
  | `BattleState` size / clone | 944 bytes / ≈ 90 ns (≈ 95 ns per KB) | 6 464 bytes / ≈ 0.67 µs (≈ 0.10 µs per KB) | clone ≪ 1 µs per KB (met) |
  | `BattleEnv::fork` (clone + two `Arc` increments) | ≈ 260 ns | ≈ 2.25 µs | — |
  | `state_hash` (FNV-1a over the canonical encoding) | ≈ 0.5 µs | ≈ 4.3 µs | fine for per-N-step checkpoints; revisit if search needs per-node hashing |
  | Instruction count, `random-rollouts-full` workload | 387 614 575 Ir for 10 402 steps (≈ 37 k Ir/step) | 571 597 279 Ir for 7 388 steps (≈ 77 k Ir/step) | **regression gate: +2 %** (`benches/battle/random-rollouts-full.toml`) |

  Wall-time numbers are indicative (shared cloud CPU, ±5 %); the deterministic gate is the instruction count. Re-measure on the user's Ryzen when convenient.
- **Adventure layers (measured in E08 T5b, 2026-10-11)**: a player's explored layer as a persistent grid (`h3_adventure::BitGrid`, ADR-0016 point 1) against a flat bitset copied by each fork, callgrind instructions per operation with twenty revealed disks (`crates/h3-adventure/examples/grid_bench.rs`; all leaf sizes in `E08-log.md`):

  | Operation | 36×36 flat | 36×36 grid | 144×144×2 flat | 144×144×2 grid |
  |---|---|---|---|---|
  | fork | 209 | 22 | 906 | 22 |
  | fork + first write (bytes copied at 144: 5 184 flat, 528 grid) | 258 | 722 | 955 | 1 550 |
  | reveal of radius 5, row by row | 1 165 | 1 814 | 1 194 | 2 443 |
  | read every tile | 41 772 | 52 469 | 1 329 426 | 1 500 214 |

  Chosen: 32 × 32 leaves (the user): a fork costs the same on every map, a write copies the table and one 128-byte leaf; per-tile reads lose 13–25 %, so observation planes are built word by word (T8). The gated fork workload came with E08 T9b (below).
- **Adventure env (measured in E08 T9b, 2026-10-11)** — same hardware and profile; the generated maps `fixtures/maps/e08-duel-36.h3m` (36 × 36 × 2, two heroes per player) and `e08-xl-144.h3m` (144 × 144 × 2, four per player), the env's random agent (`h3 adventure bench`):

  | Measure | Duel | XL | Target / threshold |
  |---|---|---|---|
  | Random legal play, one step: agent + reachable-tiles queries + `apply` (`--workload play`) | ≈ 14.1 k steps/s (≈ 71 µs, ≈ 745 k Ir) | ≈ 5.5 k steps/s (≈ 183 µs, ≈ 1.6 M Ir) | ≥ 10 k steps/s/core on the duel (met); **gate +2 %**: `random-play-duel-36`, 510 319 658 Ir for 685 steps |
  | + the acting player's observation before each step (`--observe 1`) | +15 µs | +99 µs | — (cached static planes with E13's encoding) |
  | + the entry and tile masks before each step (`--masks 1`) | +70 µs (×2.0) | +131 µs (×1.7) | — (the env keeps the current state's queries with E05's batch API) |
  | One reachable-tiles query (`--workload reach`) | ≈ 27 µs (≈ 263 k Ir) | ≈ 33 µs (≈ 342 k Ir) | < 50 µs on XL (met); **gate +2 %**: `reach-xl-144`, 657 378 985 Ir for 1 920 queries |
  | `AdventureEnv::fork` (`--workload fork`) | ≈ 0.22 µs (≈ 2 200 Ir, 4 heroes) | ≈ 0.37 µs (≈ 3 400 Ir, 8 heroes) | < 1 µs on every map, growing with the heroes, not the tiles (met: each hero's army is a `Vec`); **gate +2 %**: `fork-xl-144`, 343 600 056 Ir for 100 000 forks |
  | `GameState` / `AdventureEnv` size | 168 / 200 bytes | same | — |

  The reachable-tiles query is 96 % of a random step (`Reach::search` itself, the step price, the heap, zeroed map-sized arrays): the first lever when the adventure needs speed. The reveal is 0.8 %.
- Benchmarks: `criterion` for local wall time (`cargo xtask bench`), plus instruction counts under valgrind/callgrind for the deterministic regression gate (`cargo xtask bench-check`, Linux only; counts are compared only for the toolchain and target recorded in the baseline). Chosen over `iai-callgrind` to avoid a runner binary that must match the library version; see ADR-0002.

## 9. Portability

The logic core compiles to native (Windows, Linux, macOS) and WASM (no threads/I/O/time in logic crates makes this natural). The stable integration surface for other languages is a **C ABI** (`h3-ffi`); Python/other bindings are thin layers on it (ADR-0011).

## 10. Observability for fidelity work

A `Trace` sink can record every rule application (rule ID, inputs, RNG draws, result). This powers debugging, "explain this damage number", and differential testing against reference implementations (e.g., VCMI run as an oracle, ADR-0009).

## 11. Error handling

Logic crates return typed errors (`CommandError` variants are part of the domain language, e.g., `NotEnoughResources`, `NotYourTurn`). No panics on any path reachable from external input; `unwrap`/`expect` are denied in logic crates except in tests and in proven-invariant spots annotated with a justification.
