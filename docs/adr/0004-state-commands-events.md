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

## Amendments (decisions only)

What E01 and E03 added to the decisions above. The command, event, state and encoding details of
each task live in the crate docs (`h3-battle`, `h3-contracts`, `h3-sim`) and the epic files; the
full amendment text of E01–E03 is in git history (this file at commit `22b6bc0`).

- **Battle entry point (E01):** `h3_battle::apply(&mut BattleState, &Ruleset, BattleSide,
  Command, &mut impl Sink)`; the actor is the side the trusted caller acts for. The acting stack
  is implied by the state, so a command cannot name a stack to act with. Validation completes
  before any mutation; `validate` runs the same check without mutating.
- **Geometry seam (E01, E03):** a concrete `Field` type (hexes, footprints, obstacles, attack
  positions), not a trait, so the hot path carries no generic parameter or dynamic dispatch.
- **Episode limits are not game rules (E01):** the battle has no round limit, as in the original;
  step limits live in the sim layer (truncation).
- **Replays (E01):** a header (packs, versions, profile, ruleset hash, encoding versions), the
  seed, the setup, the command log, a state hash every N commands and the final hash.
- **Seam growth (E03):** `BattleSetup` and `BattleOutcome` grow by fields that default to
  "absent"; each change is a user decision (Q16, Q17, Q24, Q25; heroes: ADR-0013).
- **Setup randomness before the seam (E03 T7):** random battle inputs such as obstacles are drawn
  before the seam (`h3_battle::generate_obstacles`, stream `OBSTACLES`) and passed in; the battle
  draws only in-battle randomness.
- **Observation reads the ruleset (E04a T11a, Q37):** `h3_battle::observe(&BattleState,
  &Ruleset, BattleSide)`, with the same precondition as `apply` (the ruleset the battle was built
  with); the viewer's own hero's private part, such as its surrender price, is derived from it.

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30 – 2026-10-01: Amended by E01 and E03 (T3–T9).
- 2026-10-04: Amended by E04a T11a (the observation reads the ruleset, Q37).
- 2026-10-03: Amendments cut to their decisions (ADR size rule, `docs/adr/README.md`); the
  implementation detail is in the crate docs and in git history.
