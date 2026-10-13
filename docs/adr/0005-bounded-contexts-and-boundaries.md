# ADR-0005: Bounded contexts and enforced boundaries

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

The user wants clear separation of concerns, sub-domains for reasoning about rules, and architectural boundaries that prevent rot. Documentation alone does not prevent rot with many agents working in parallel over time.

## Decision

1. Organize the domain into bounded contexts per `docs/03-domain.md` (initial: kernel, content, contracts, battle, adventure, town, heroes & army, magic, game). Start with only the contexts an epic needs.
2. **One Cargo crate per context/adapter.** Layers and dependency direction as in `02-architecture.md` §2: adapters → application (game) → domain contexts → shared (contracts, content, kernel). Domain contexts **do not depend on each other**; they communicate through `contracts` types and the `game` orchestrator.
3. **No user-facing concerns in the core:** graphics, audio, i18n, asset formats, file/network I/O live in adapters or external clients.
4. **Mechanical enforcement** via `cargo xtask check-arch`:
   - the allowed dependency graph is declared in `arch.toml`; any other edge fails,
   - logic crates may not use forbidden APIs/crates (I/O, time, threads, ad-hoc floats outside `kernel::num`, platform transcendentals, `HashMap`/`HashSet`, random sources),
   - `pub` surface is minimal (`pub(crate)` by default); public API changes of seams require an ADR or a noted decision,
   - external crates allowed in logic crates are listed explicitly.
5. Splitting/merging contexts is allowed, recorded by ADR (it changes boundaries).

## Consequences

Compile-time and CI-time protection against coupling; the adventure↔battle seam makes battle trainable in isolation. Cost: some upfront ceremony for contracts and orchestration.

## Alternatives considered

Single crate with modules (boundaries erode, no compile-time enforcement); fine-grained crates from day one (premature, slows the skeleton) — crates are created when the context exists.

## Amendment — E01 walking skeleton (2026-09-30)

- **Crates created:** logic: `h3-kernel`, `h3-content`, `h3-contracts` (shared), `h3-battle` (domain); adapters: `h3-pack` (new: parses TOML content packs and scenarios into content definitions; the "TOML loader (adapter side)" of the E01 scope), `h3-sim`, `h3-ffi`, `h3-cli`; tool: `xtask`. `h3-game` is not needed yet (the sim drives the battle context directly). The graph is declared in `arch.toml`.
- **Logic crates are `#![no_std]` + `alloc`.** This makes I/O, clocks, threads, environment access and `std`'s `HashMap`/`HashSet` unavailable at compile time rather than only by lint, and keeps WASM natural. Tests may use `std` (`#[cfg(test)] extern crate std`).
- **Enforcement (`cargo xtask check-arch`):** every workspace member must be declared; every normal/build/dev dependency edge must be listed; layer rule (strictly downward, except shared→shared and adapter→adapter; domain↛domain; nothing depends on tools). Logic crates: the crate root must carry `#![no_std]`, `#![forbid(unsafe_code)]` and deny the policy lints (`unwrap`/`expect`/`panic`/`todo`/`unimplemented`/`unreachable`/`float_arithmetic`/print/exit) — parsed from the attribute tokens, not by substring; relaxing a policy lint (`allow`/`expect`/`warn`, also in `cfg_attr`) in non-test logic code fails unless allow-listed in `arch.toml` (only `kernel::num` may allow `float_arithmetic`); a token scan rejects `HashMap`, `HashSet`, `std`, `static mut`, `unsafe`, float types and float literals outside `kernel::num`, and any `sort*` method outside `kernel::order`. Test code is exempt: `#[cfg(test)]` items, files reached through a `#[cfg(test)] mod x;` declaration (module tree from the crate root), and Cargo's `tests/`/`benches/`/`examples/`. The `// arch-ok: <RULE-ID|ADR-NNNN> reason` escape hatch requires a reference. Tests prove that violating edges and code are rejected.
- The minimal-`pub`-surface point is not machine-checked yet (backlog).

## Amendment — E02 content pipeline (2026-10-01)

- **`h3-formats`** (adapter, no workspace dependencies; external `miniz_oxide` for zlib): bounded readers for the original installation's files (LOD archives, text tables, the executable's static tables). It replaces the planned `h3-import` crate: with Q13 the original data is harvested once into committed packs, so there is no run-time import.
- **`h3-harvest`** (tool layer, like `xtask`): the harvester binary that writes `packs/{base,ab,sod}`; nothing may depend on it.

## Amendment — E08 adventure and game crates (2026-10-10, ADR-0015)

- **Crates created (E08 T2):** `h3-adventure` (domain, logic), `h3-game` (application, logic:
  `no_std`, the purity scan and the 100 % coverage gate like a domain crate), `h3-map` and
  `h3-pack-adventure` (adapters). `h3-game` is the first application crate: it depends on the
  domain contexts it wires (`h3-adventure` now; `h3-battle` from E08 increment 3) and on the
  hero model `h3-heroes`, in the shared layer since E08 T18a (ADR-0013 point 2); no domain crate
  depends on it. ADR-0015 holds the crate shape, ADR-0016 and ADR-0017 what
  the two logic crates decide.
- **Cross-context ids** `PlayerId` and `HeroId` are contract types (`h3-contracts`).
- **Pointer identity** (`ptr_eq`, `as_ptr`) is a forbidden token in logic crates outside the
  adventure's persistent module (ADR-0016 point 2).

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (crates created, `no_std` logic crates, what `check-arch` enforces).
- 2026-10-01: Amended by E02 (`h3-formats` adapter, `h3-harvest` tool).
- 2026-10-10: Amended by E08 (adventure and game crates, the first application crate, ids in the contracts; ADR-0015).
- 2026-10-14: Amended by E08 T18a (ADR-0013 point 2): `h3-heroes` moves to the shared layer;
  `check-arch` gains **forbidden edges** (`[[policy.forbidden_edge]]` in `arch.toml`): an edge the
  layers would allow but a decision rules out, rejected whether declared or not (`h3-battle` →
  `h3-heroes`).
