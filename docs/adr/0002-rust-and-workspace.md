# ADR-0002: Rust, Cargo workspace, unsafe policy

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

The logic core is the hot path of RL simulation: it needs C-class speed, predictable memory behavior, strong types for domain modelling, and portability (native + WASM). Clients and the training side may use other technologies.

## Decision

- The core and its tooling are written in **Rust**, organized as a **Cargo workspace** with one crate per bounded context/adapter (ADR-0005).
- Toolchain pinned via `rust-toolchain.toml`; MSRV policy: latest stable at the time of pinning, bumped deliberately (record bumps in HANDOFF).
- `#![forbid(unsafe_code)]` in all logic crates. `unsafe` is allowed only in `h3-ffi` (and any adapter where it is unavoidable), is minimized, documented with safety comments, and covered by tests. Performance-motivated `unsafe` in logic crates requires an ADR with benchmark evidence.
- Domain modelling uses newtypes, enums and exhaustive matching; invariants live in types where practical.
- Dependencies are kept few in logic crates (see ADR-0009 for license constraints); each new dependency in a logic crate is justified in the commit or ADR.

## Consequences

Safe-by-default logic, excellent performance, good DDD expressiveness, WASM support. Rust's compile times and learning curve are accepted. RL training code may live in another language (ADR-0011).

## Alternatives considered

C++ (VCMI's language; weaker safety, harder to enforce boundaries mechanically); Go/Java/C# (GC and layout control are worse for the hot path; poorer WASM/C-ABI story); Zig (immature ecosystem for this scale).
