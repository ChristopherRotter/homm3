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

## Amendment — E01 walking skeleton (2026-09-30)

How the model turned out in the battle context:

- **Entry point:** `h3_battle::apply(&mut BattleState, &Ruleset, BattleSide, Command, &mut impl Sink)`; the actor is the side the trusted caller acts for. Validation (`plan`) completes before any mutation; after it, every stack id is known valid, so the mutation phase uses infallible accessors. `validate` exposes the same check without mutating.
- **Commands (intent only):** `Move { to }`, `Attack { target, from }`, `Wait`, `Defend`. The acting stack is implied by the state (the active stack), so a command cannot even name a stack to act with.
- **Events:** `RoundStarted`, `TurnStarted`, `Moved`, `Waited`, `Defended`, `Struck` (damage, kills, retaliation flag), `BattleEnded`, each with a `Visibility` (`Public` or `Side`); `project(event, viewer)` filters. All battle events are public so far. `NoopSink` compiles to nothing; `Vec<Event>` records.
- **Observation:** the full battlefield minus the RNG state (the only hidden information so far); property-tested for hygiene.
- **Legal commands:** enumerated into a fixed-capacity buffer (`LegalCommands`, capacity 449) from the same reachability/adjacency functions `apply` uses; consistency is property-tested in both directions.
- **State layout:** `BattleState` is 944 bytes (32 inline stacks with creature stats copied in, round/turn bookkeeping, RNG) and clones in ≈ 90 ns; no heap allocation in `apply`/`legal_commands`.
- **Geometry seam:** the battle context uses only `Hex` and the `Field` API (standable, neighbours, distance). It is a concrete type rather than a trait: a trait would have spread a generic parameter (or dynamic dispatch) through the hot path for no current benefit. E03 can change `Field`'s internals (obstacles, two-hex units) behind the same API.
- **Episode limits are not game rules:** the battle has no round limit (as in the original); step limits live in the sim layer (`BattleEnv` truncation).
- **Replays:** header (packs + versions, profile, ruleset hash, RNG/hash/encoding versions) + seed + setup + command log + a state hash every N commands + final hash; model in `h3-sim::replay`, TOML format in `h3-cli`.

## Amendment — E03 battle core (2026-10-01, T3)

- **Geometry seam extended** (still a concrete `Field`): footprints of one- and two-hex stacks
  (`Footprint`, head hex + tail behind it by side), `HexSet` (256-bit set) for obstacles and
  occupancy, `Field::attack_positions` (the 24 canonical slots from which an attacker's
  footprint touches a target's; at most 10 are distinct). Movement and attacks are validated on
  footprints. `MAX_LEGAL_COMMANDS` = 2 + 255 + 24 · 32.
- **Seam type change (`BattleSetup`):** `obstacles: Vec<u8>` (blocked hexes, validated by the
  battle context). Further E03 fields (hero presence per side Q17, battlefield/terrain, war
  machines, formation) are added the same way, each defaulting to "absent".
- **Observation:** includes the obstacles (public). State hash encoding v2 (stack traits,
  obstacles).
- **Sim encodings:** action encoding v2 (attacks by target and canonical slot); replay file
  format v2 (`[setup]` table).

## Amendment — E03 battle core (2026-10-01, T4–T5)

- **Commands:** `Shoot { target, at }` (T4; `at` is the target's head, or for area shots any hex
  of the target).
- **Events:** `Struck` carries `StrikeFlags` (retaliation, ranged, secondary target, death blow)
  instead of a retaliation bool (T4). T5 adds `LifeDrained { stack, hit_points, resurrected }`,
  `DeathStared { attacker, target, kills }` and `Regenerated { stack, hit_points }`; strike and
  return emits a second `Moved`. All public.
- **Attack structure (T5):** an attack (`combat::attack`) rolls the death blow once, strikes the
  target, then the secondary targets of its pattern in stack-id order, then applies life drain
  and death stare; melee = attack, retaliation, second attack, return. Secondary targets are
  computed from the positions at the start of each attack.
- **State:** stacks carry `base_count` (resurrection limit) and retaliations per round / left;
  state hash encoding v4.

## Revision history

- 2026-09-30: Accepted.
- 2026-09-30: Amended by E01 (battle entry point, command set, events, geometry seam, replay contents).
- 2026-10-01: Amended by E03 T3 (footprints, obstacles, attack positions, setup/encoding changes).
- 2026-10-01: Amended by E03 T4–T5 (shooting, strike flags, ability events, attack structure).
