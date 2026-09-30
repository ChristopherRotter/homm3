# ADR-0004: Server-authoritative state, commands and events

- Status: Accepted
- Date: 2026-09-30
- Deciders: user + planning session

## Context

We need a single, validated mutation path (security), a replayable record (debugging, AI), cheap cloning (search/RL), per-viewer information hiding (anti-cheat), and event output for clients — without performance cost in the simulation hot path.

## Decision

1. **Single mutation entry point:** `apply(&mut State, &Ruleset, Actor, Command, &mut impl Sink) -> Result<(), CommandError>` per context/orchestrator.
2. **`Actor` is supplied by the trusted caller**, never decoded from the command payload.
3. **Commands carry intent only** (no prices, amounts, ownership or derived values).
4. **Atomic:** validation precedes mutation (or mutations are transactional); on `Err`, state is unchanged. Property-tested.
5. **State is the source of truth; events are output.** Events describe what happened, carry a visibility scope, and are generated through a generic `Sink` (no-op sink in simulation = zero cost). We are *not* event-sourced.
6. **Observation:** `observe(&State, Viewer) -> Observation` and `project(event, Viewer)`. Hidden information never leaves via these projections' complement.
7. **Legality:** `legal_commands(&State, Actor)` (enumeration and/or mask) shares preconditions with `apply`; consistency is property-tested.
8. **State layout:** flat, index-addressed, cheap to clone; bulk immutable data behind `Arc`; fixed-capacity structures in battle; no allocation in the battle hot path.
9. **Replays** = ruleset id + initial scenario + seed + command log (+ optional hashes). **Saves** = our own versioned state serialization; original save import is a separate future adapter.
10. **Multiplayer model:** authoritative server with per-viewer projections. Lockstep with full state on clients is excluded (leaks hidden information).

## Consequences

Clear security story, reproducibility, straightforward RL integration and client event streams. Cost: every feature must define its command validation, events and visibility up front (enforced by the Definition of Done).

## Alternatives considered

Event sourcing (heavier, slower, unnecessary); direct state mutation APIs (bypassable validation); lockstep multiplayer (information leak, conflicts with anti-cheat goal).
