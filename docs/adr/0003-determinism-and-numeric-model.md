# ADR-0003: Determinism and numeric model

- Status: Accepted (amended 2026-09-30, see revision history)
- Date: 2026-09-30
- Deciders: user + planning session

## Context

RL needs reproducible simulations; debugging fidelity needs replays; any future multiplayer, save/load or cross-platform use needs bit-identical behavior on Linux, Windows, macOS and WASM.

Floating point is not inherently nondeterministic. Factorio runs floating-point game logic deterministically across platforms: they report that the feared cross-machine float discrepancies "hasn't been a big problem", that they replaced platform math with **their own trigonometric functions**, that they built **integration tests that check determinism**, and that such testing exposed an unrelated desync source, an ambiguous sort comparator (equal elements ordered differently by different standard libraries) ([FFF-52](https://factorio.com/blog/post/fff-52)). They also detect desyncs with a CRC of the whole game state per tick, compared between runs ([FFF-47](https://factorio.com/blog/post/fff-47)). (Their posts do not describe compiler flags, so the flag discipline below is our own.)

There is a second reason to allow floats: the original game may compute some rules with floating-point and then truncate/round. Reproducing such a rule **exactly** is easier and more faithful with the same IEEE-754 operations than with hand-rolled fixed-point emulation.

## Decision

**Given (ruleset, initial state, seed, command sequence) the engine produces bit-identical state and events on every supported target.** Determinism is achieved by discipline and continuous testing, not by banning a number type.

### Numeric policy

1. **Two sanctioned numeric domains in logic crates:** fixed-width integers (default) and IEEE-754 `f32`/`f64` through the `kernel::num` surface (only where a rule's original semantics or a documented need calls for it). Each rule that uses floats says so in its catalog entry (`numeric = "float"`, with the exact rounding/truncation behavior); rules default to `numeric = "integer"`.
2. **Allowed float operations:** `+ − × ÷`, `sqrt`, comparisons, and explicitly-rounded conversions (floor/ceil/trunc/round-half-rule as specified per rule). These are exactly specified by IEEE-754 and deterministic on our supported targets.
3. **Transcendentals** (`sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `pow`, …) are **never** taken from `std`/platform libm. If a rule needs them, `kernel::num` provides in-repo (or vetted pure-software, e.g. a `libm`-style) implementations with published test vectors and cross-platform golden tests. Lint: `disallowed_methods` for the `std` versions.
4. **Forbidden:** fast-math or unsafe float intrinsics; `mul_add` unless `kernel::num` wraps it with a software fallback; relying on NaN payloads; NaN/±inf in state (conversions and results are checked; encountering them is a bug that returns an error or fails a test); direct use of `f32`/`f64` in logic crates outside `kernel::num` call sites that the lint policy allows per module (module-level `allow` needs a rule ID).
5. **Supported targets:** `x86_64`, `aarch64`, `wasm32`. 32-bit x86 (x87 extended precision) is **unsupported**. Codegen-affecting flags (`target-cpu`, LTO, opt level, `-C` flags) are pinned in the workspace config, and CI runs fixtures in debug and release, with and without `target-cpu=native`, to catch flag sensitivity.
6. **Integer hygiene:** fixed-width integers in anything serialized or hashed; no `usize` in persistent/hashed representations; checked/saturating ops where overflow is possible, with defined behavior.

### Other determinism rules

7. **In-repo PRNG** with documented algorithm and test vectors (chosen in E01). No `rand` thread/OS RNGs. **Named streams** (battle, map events, AI/tie-breaking, …) derived by documented splitting so one consumer cannot perturb another. All randomness goes through an explicit `Rng` parameter. Replay headers record PRNG algorithm + version.
8. **No unordered iteration dependence.** `HashMap`/`HashSet` are disallowed in logic crates; use dense indexed storage, `Vec`, `BTreeMap`, or deterministic-order maps.
9. **Sorting is total and stable by construction:** every sort key ends in a unique tie-breaker (e.g., entity ID), so equal elements cannot be ordered differently by different implementations or library versions (the Factorio lesson). `sort_unstable*` is allowed only with a total order.
10. **No ambient inputs:** no clock, no threads, no I/O, no environment, no global/mutable statics.
11. **State hash:** canonical, versioned, platform-independent, computed server-side; replays may record hashes every N steps. `verify` replays fixtures and compares golden hashes on each CI platform (Linux, Windows; WASM when available). This is our equivalent of Factorio's per-tick CRC.
12. **Desync diagnostics:** state has a human-readable debug serialization so the first divergent step can be diffed between two runs.
13. **Determinism is tested, not assumed:** dedicated determinism integration tests (replay twice; replay across platforms/profiles; clone-and-continue equivalence) are part of `verify` (ADR-0010), and any new use of floats adds a fixture that exercises it.

## Consequences

Faithful reproduction of float-based original rules without emulation gymnastics; integer math remains the default and the cheapest to verify; a single audited place (`kernel::num`) for anything numerically subtle. Cost: lint configuration, a wider CI matrix, and discipline around transcendentals and sorting. AI/observation layers may use floats freely as long as they never feed back into game state.

## Alternatives considered

Integer/fixed-point only (the original version of this ADR: simplest to verify, but forces emulation of float-based original rules and is stricter than the evidence requires); floats with no special handling (platform libm and sort differences would break replays); soft-float everywhere (slow, unnecessary given Factorio's experience and IEEE-754 basics being deterministic on our targets).

## Revision history

- 2026-09-30: Initial version, integer/fixed-point only.
- 2026-09-30: Amended after user input and FFF-52: floats allowed under the `kernel::num` discipline; added sorting rule, supported-target list and CI matrix.
