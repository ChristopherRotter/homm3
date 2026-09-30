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
