# 06 — Security and anti-cheat design

## Scope

The original game had exploits such as spending another player's resources in your town, and in networked play the client held far more information than the player should see. The engine must make that class of bug structurally hard.

Protections apply to the **authoritative (server) mode**: a trusted engine instance with untrusted clients. In local single-player or hotseat with an embedded engine, the local user is trusted (hotseat hiding is a UI concern). The design works in both because the same API is used.

## Trust boundaries

```
untrusted client ──bytes──▶ [transport/decoder adapter] ──typed Command──▶ [session: authenticates → Actor] ──▶ game.apply(actor, command)
untrusted files  ──bytes──▶ [formats adapter: validates, bounds, converts] ──▶ domain types
                                                              game state ──▶ [observe/project per viewer] ──▶ client
```

1. **Identity comes from the session, never from the payload.** The `Actor` passed to `apply` is created by the trusted session layer from the authenticated connection. Commands never contain "who I am" or "whose resources to use".
2. **Commands carry intent only.** No prices, resource amounts, ownership claims, derived stats or results. Everything derivable is computed from state. Example: `BuildStructure { town, structure }` — cost and payer come from the ruleset and the actor's own treasury.
3. **Validation is total and centralized.** `apply` checks: it is the actor's turn/phase, the actor owns every referenced entity (or the interaction is explicitly modelled), all references exist, preconditions hold, resource checks use the *actor's* stock, bounds hold. The validator runs before any mutation; on failure state is unchanged (atomicity is property-tested).
4. **No reach-through.** Domain functions receive the minimal capability they need (e.g., the actor's own treasury handle), not the whole state plus an id, which makes cross-player access impossible to express by accident.
5. **Legal-command consistency.** `legal_commands` and `apply` share one precondition implementation (or are tested equivalent), so UI hints can't diverge from enforcement.

## Information hiding

- The server never sends full state to a client. It sends `Observation`s and events **projected per viewer**: every event has a visibility scope; `project(event, viewer)` drops or redacts what the viewer may not know.
- Fog of war, unexplored tiles, opponents' unseen armies/heroes/resources, hidden object contents, and opponents' private decisions are excluded from projections. Define per context what each viewer may know (adventure context epic owns the list).
- **Lockstep-with-full-state multiplayer is excluded** by design: every client would hold the complete state and could read everything.
- **Clients therefore need a view model** that applies projected events to partial state. Keeping client code out of the core (ADR-0005) makes this a client concern, but projections are a core contract.
- **Side channels to consider:** RNG seed/stream state must not reach clients (it predicts rolls); saves and replays are exported *per viewer* (redacted) unless the requester is authorized (e.g., post-game); state hashes must not be computed over, or exposed in a form that leaks, hidden data; error messages must not reveal hidden facts ("no such hidden object at tile X" vs. "invalid").
- **Hotseat** hiding is UI-level and out of scope for the core.

## Denial of service and resource limits

- Bounded command size and count per turn; bounded work per command (path search limits, list lengths).
- Deterministic termination: loops in rules must have ruleset-defined bounds.
- Memory bounds for map/archive parsing; reject dimensions or counts above configured limits.
- Arithmetic: checked/saturating operations wherever overflow is possible; no panics on external input (enforced via the lint policy in `05-testing.md`).
- Rate limiting, connection handling and authentication are the server's job (later epic); the core supplies bounded, panic-free primitives.

## Untrusted files

Maps, archives, and saves are attacker-controlled input. `h3-formats` parsers: no `unsafe`, strict bounds checking, length limits, graceful errors, fuzzed continuously (`05-testing.md`). Anything that becomes domain state passes the same validation as if commands had built it (e.g., a map cannot start with negative resources or overlapping stacks beyond the limits).

## Security regression policy

- Every known exploit class gets a quirks-ledger entry (`exploit-fix`) and a named regression test that attempts the exploit and asserts rejection.
- Property tests for ownership isolation and observation hygiene (see `05-testing.md`) run on every context.
- Adding a command requires writing its validation, rejection tests, and visibility scope in the same change. The DoD checklist enforces this.

## Out of scope for the core

Transport encryption, authentication protocols, account management, matchmaking, client integrity/anti-tamper on the client side. These belong to server and client layers; the core only guarantees that a correct server cannot be tricked via valid-looking commands and that it can avoid leaking hidden state.
