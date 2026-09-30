# ADR-0011: Portability and C ABI

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

Clients may be a web app and several desktop variants (Windows, Linux, maybe macOS); the RL side may not be written in Rust or Python; the core must be reusable from anywhere.

## Decision

1. Logic crates compile for native targets (Windows, Linux, macOS) and WASM: no threads, I/O, time or OS dependencies in them (also required by ADR-0003).
2. The stable integration surface for other languages is a **C ABI** in `h3-ffi`: opaque handles, explicit create/destroy, batch-oriented stepping, error codes instead of panics, versioned structs/encodings. Language bindings (Python, C#, JS/WASM glue) are thin layers on top and live outside the logic crates.
3. `unsafe` is permitted in `h3-ffi` only (ADR-0002), minimized and tested.
4. The FFI layer never exposes unprojected state to untrusted consumers in server mode (ADR-0008).
5. The RL/training implementation language is decided later (OPEN-QUESTIONS Q5); nothing in the core assumes a specific one.

## Consequences

Any language can drive the engine; web clients via WASM are feasible; FFI design effort is concentrated in one crate.

## Alternatives considered

Language-specific bindings only (locks out other ecosystems); a network protocol for everything (too slow for training hot loops); exposing Rust types directly (unstable ABI).
