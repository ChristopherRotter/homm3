# Documentation index

Working project name: **h3** (provisional; see OPEN-QUESTIONS Q2). Last structural update: 2026-09-30.

## How to read these docs

| Doc | Purpose |
|---|---|
| [01-vision.md](01-vision.md) | Goals, non-goals, success criteria, decisions already taken |
| [02-architecture.md](02-architecture.md) | Principles, layers, crates, state/command/event model, performance, extension model |
| [03-domain.md](03-domain.md) | Bounded contexts, seams, ubiquitous language |
| [04-rules-and-sources.md](04-rules-and-sources.md) | Source of truth, provenance, rule catalog, quirks and profiles |
| [05-testing.md](05-testing.md) | Test strategy, gates, and the mechanisms that keep agents from skipping tests |
| [06-security.md](06-security.md) | Threat model, trust boundaries, information hiding, anti-exploit design |
| [07-simulation-and-ai.md](07-simulation-and-ai.md) | Simulation environment, baselines, RL strategy, training automation |
| [08-work-process.md](08-work-process.md) | How agents work in increments, handoff protocol, when to ask |
| [BACKLOG.md](BACKLOG.md) | Coarse-grained epics, ordered; only the next ones are refined |
| [HANDOFF.md](HANDOFF.md) | Living state: what was done, what is next, traps |
| [OPEN-QUESTIONS.md](OPEN-QUESTIONS.md) | Decisions waiting for the user |
| [epics/](epics/) | Refined epics (only those about to be worked on) |
| [adr/](adr/) | Architecture Decision Records (binding) |
| [rules/](rules/) | Rule catalog (machine-checked), quirks ledger |

## Planning philosophy

Decide upfront only what is expensive to change later: determinism, state/command model, context boundaries, security model, the compiled-ruleset approach, licensing/IP stance, and the provenance/testing machinery. Everything else is deferred and refined just in time. The first increment is a **walking skeleton** (a minimal battle end to end, including command validation, observation, determinism replay, benchmark) so those upfront decisions get tested early.

## Status of upfront decisions

| Topic | Status | Where |
|---|---|---|
| Rust core, Cargo workspace, no unsafe in logic | Accepted | ADR-0002 |
| Determinism & numeric model | Accepted | ADR-0003 |
| Server-authoritative state/command/event model | Accepted | ADR-0004 |
| Bounded contexts & enforced boundaries | Accepted | ADR-0005 |
| Config tiers 1–3, compiled ruleset | Accepted | ADR-0006 |
| Provenance, quirks ledger, profiles | Accepted | ADR-0007 |
| Security model | Accepted | ADR-0008 |
| License & reference-material policy | **Proposed — needs user decision** | ADR-0009, Q1 |
| Testing gates & rule traceability | Accepted | ADR-0010 |
| Portability & C ABI | Accepted | ADR-0011 |
