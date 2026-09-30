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

## Amendment — E01 walking skeleton (2026-09-30)

- **ABI v1** (`crates/h3-ffi`, header `crates/h3-ffi/include/h3.h`): `h3_abi_version`, `h3_action_count`, `h3_observation_len`, `h3_env_create_synthetic(scenario, seed, max_steps)`, `h3_env_destroy`, `h3_env_fork`, `h3_env_reset`, `h3_env_to_move`, `h3_env_legal_mask`, `h3_env_observe`, `h3_env_step` (→ `H3StepResult`), `h3_env_state_hash`. Error codes are negative `i32`; every entry point catches panics (`H3_ERR_PANIC`); pointer-taking functions are `unsafe extern "C"` with documented contracts.
- The crate builds `rlib`, `cdylib` and `staticlib`. `cargo xtask ffi-smoke` compiles `crates/h3-ffi/c/smoke.c` against the static library and runs it (Linux/macOS; N/A on Windows, where the Rust-side ABI tests still run).
- Batch stepping over many environments and loading packs/scenarios through the ABI are not there yet (E05).
- Action and observation encodings are owned and versioned by `h3-sim` (`ACTION_ENCODING_VERSION`, `OBSERVATION_ENCODING_VERSION`), and covered by the ABI version.
- The ABI exposes the full-state hash and is meant for trusted local consumers (training, tools); a server must not forward it to untrusted clients.

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (ABI v1 surface and smoke test).
