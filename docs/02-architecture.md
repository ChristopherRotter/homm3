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
  h3-kernel/       ids, fixed-width numerics, PRNG, state hashing, errors, bonus/modifier primitives (design deferred)
  h3-contracts/    seam types: BattleSetup, BattleOutcome, ...
  h3-content/      definition model (creatures, spells, artifacts, ...), content-pack manifest, Ruleset compiler (pure: takes parsed data)
  h3-battle/       battle context
  h3-adventure/    adventure-map context            (later)
  h3-town/ h3-heroes/ h3-magic/                     (later, split out when they earn it)
  h3-game/         orchestration, session, projections/visibility
  h3-formats/      LOD/DEF/H3M/... readers (I/O, untrusted input)            (later)
  h3-import/       extracts game data from the user's install into content packs   (later)
  h3-sim/          RL environment API (batched, fork/clone, masks, observations)
  h3-ffi/          C ABI
  h3-cli/          headless runner, replay tool
xtask/             verify, check-arch, check-rules, coverage, bench gates
```

Start with the smallest set the walking skeleton needs (kernel, content, battle, sim, cli, xtask, plus minimal game/contracts if required). Split crates when a context actually exists — but the dependency rules apply from day one.

## 4. State, commands, events

Detailed in ADR-0004. In short:

- `apply(state, ruleset, actor, command, sink) -> Result<(), CommandError>` is the only mutation path. `actor` is supplied by the trusted session layer, never by the payload. Commands carry **intent only**; all derived values (prices, amounts, ownership) come from state. On error the state is unchanged.
- Events are an **output** (for clients, logs, animation, replays). Events carry a visibility scope. The state, not the event log, is the source of truth.
- `observe(state, viewer) -> Observation` projects hidden information away. `legal_commands(state, actor)` enumerates/masks legal commands and must agree with `apply` (property-tested).
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
- **Provisional targets** (calibrated in the walking skeleton, then enforced as regression thresholds): battle `apply` including legal-command mask ≥ 100k steps/s/core; state clone of a battle ≪ 1 µs-range per KB of state; adventure targets set when that context exists. These numbers are placeholders to be replaced by measurements.
- Benchmarks: `criterion` for local wall-time, plus an instruction-count-based tool for deterministic CI regression checks (tool choice in E01).

## 9. Portability

The logic core compiles to native (Windows, Linux, macOS) and WASM (no threads/I/O/time in logic crates makes this natural). The stable integration surface for other languages is a **C ABI** (`h3-ffi`); Python/other bindings are thin layers on it (ADR-0011).

## 10. Observability for fidelity work

A `Trace` sink can record every rule application (rule ID, inputs, RNG draws, result). This powers debugging, "explain this damage number", and differential testing against reference implementations (e.g., VCMI as a black-box oracle, ADR-0009).

## 11. Error handling

Logic crates return typed errors (`CommandError` variants are part of the domain language, e.g., `NotEnoughResources`, `NotYourTurn`). No panics on any path reachable from external input; `unwrap`/`expect` are denied in logic crates except in tests and in proven-invariant spots annotated with a justification.
