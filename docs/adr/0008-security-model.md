# ADR-0008: Security model

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

The original game allows exploits such as spending other players' resources, and clients hold information players shouldn't see. The engine must make those classes structurally hard and protect against invalid commands from cheating clients and against reading hidden data.

## Decision

Adopt the model in `docs/06-security.md`:

1. Identity from the trusted session (`Actor`), never from payloads.
2. Commands carry intent only; all derived values come from state.
3. Centralized, total, pre-mutation validation; atomic failure.
4. Domain functions receive minimal capabilities (no reach-through by id).
5. Per-viewer projections of observations and events; no full state to clients; RNG state and hidden data never exposed; redacted replays/saves; no lockstep-with-full-state multiplayer.
6. Bounded work, bounded inputs, panic-free on external input, checked arithmetic; untrusted-file parsers fuzzed.
7. Every known exploit class has an `exploit-fix` quirk entry and a regression test; ownership-isolation and observation-hygiene property tests exist per context.

## Consequences

Security requirements are part of each command's Definition of Done. Some design freedom is reduced (capability-style APIs, per-event visibility) in exchange for structural safety.

## Alternatives considered

Client-side trust (what the original effectively did); validation only at the transport edge (bypassable by internal callers); full-state clients with obfuscation (not a real defense).
