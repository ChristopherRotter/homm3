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
- Rate limiting, connection handling and authentication are the server's job ("Hosted games" below); the core supplies bounded, panic-free primitives.

## Hosted games (the server, E16, ADR-0018)

The server (`h3-server`, the I/O shell around `h3-protocol`'s sans-I/O session core) is the first surface where untrusted clients send commands. It adds no rules; it guarantees:

- **Identity is the connection's seat** (ADR-0018 point 6). A connection becomes a seat's actor by taking it: the first time without a token (the server hands the seat's token out in `Joined`), afterwards only with that token, 128 bits of OS randomness compared in a time independent of where it differs. Commands name no side or player (a command with such a field does not decode); the session applies them with the seat's side. A connection without a seat is no actor (`NotSeated`). Accounts bind seats from E16 M3.
- **A rejected command changes nothing**: neither the battle nor the journal, and the other seat hears nothing; while the battle runs, the seat not to act gets `NotYourTurn`. Property-tested over the wire in `h3-protocol`.
- **Everything outbound is projected per viewer by construction** (point 7): the mapping's observation and events take the viewer and call the engine's `observe` and `project`; the unprojected conversion is private to it. Every accepted command gives every viewer an `Events` message, even an empty one, so the shape of a viewer's stream does not depend on what it may not see. Property-tested: a hidden cast leaves the other viewer's bytes equal for any two seeds.
- **Secrets stay on the server** (point 8). The tools derive the visible random obstacles and the hidden battle RNG from one seed (different streams), so a seed a client could guess from the layout would predict every roll. The server therefore draws the battle's seed as 64 bits of OS randomness, independent of anything a client sees, and the obstacles from a separate seed that is used once and forgotten; the obstacles enter the setup as pieces, so no journal or replay holds an obstacle seed. Seeds, RNG state, state hashes and tokens never reach a client or a log line (a token reaches only the client that takes its seat, in `Joined`). The replay (seed, setup, every command) reaches the seats only after the battle (`NotOver` before); until then a viewer may fetch its *record*, the log of what it was sent.
- **Bounds, all configured** (point 9; `[limits]` in the configuration, defaults in parentheses): the frame size (4 096 bytes, checked before parsing; a frame far over it is dropped unread), messages per second per connection (20, then `RateLimited`), connections in total (64) and per address (8, refused before the handshake), the idle timeout (no message from the client for 300 s; it also bounds a stalled write and the handshake), the outbound queue per connection (256 frames: a client too slow to read is dropped, `TooSlow`, never buffered for). One command in flight per seat holds by construction: the game's thread handles one message at a time. A malformed, binary, oversized or out-of-place message closes the connection with its reason; decoding never panics (property-tested on arbitrary bytes; the fuzz harness comes with E16 M2).
- **One ruleset, the server's** (point 10): clients run no rules in a hosted game, and the server trusts no client-computed value; the manifest at join names the content.
- **Exposure**: the server binds loopback by default and serves the LAN until E16 M3; on the internet only behind a TLS reverse proxy, once accounts exist.
- **Not covered yet** (E16 M2 unless noted): a viewer's record, which every `FetchRecord` sends whole, and the outbound queue are bounded in messages, not bytes, so a seated client can make the server hold and queue much memory within the rate limit; a connection that never joins keeps its counted place until its idle timeout; a client that loses its `Joined` loses the seat, since only that message carries the token; the journal, restore and the re-run that verifies every result before it counts (M2, M4); timing and message-size side channels (bounded by projection and the stream's fixed shape; timing is out of scope for now).

## Untrusted files

Maps, archives, and saves are attacker-controlled input. `h3-formats` parsers: no `unsafe`, strict bounds checking, length limits, graceful errors, fuzzed continuously (`05-testing.md`). Anything that becomes domain state passes the same validation as if commands had built it (e.g., a map cannot start with negative resources or overlapping stacks beyond the limits).

## Security regression policy

- Every known exploit class gets a quirks-ledger entry (`exploit-fix`) and a named regression test that attempts the exploit and asserts rejection.
- Property tests for ownership isolation and observation hygiene (see `05-testing.md`) run on every context.
- Adding a command requires writing its validation, rejection tests, and visibility scope in the same change. The DoD checklist enforces this.

## Out of scope for the core

Transport encryption, authentication protocols, account management, matchmaking, client integrity/anti-tamper on the client side. These belong to server and client layers; the core only guarantees that a correct server cannot be tricked via valid-looking commands and that it can avoid leaking hidden state.
