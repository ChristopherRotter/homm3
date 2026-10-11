# ADR-0018: Server and protocol — hosted games over a sans-I/O protocol core

- Status: Accepted (user, 2026-10-10, the E16 M0 checkpoint; directions from SRV-Q1 to SRV-Q11)
- Date: 2026-10-10
- Deciders: user; server lane sessions (E16 refinement, M0)
- Working detail (flows, messages, manifest, journal, integrity): [`docs/epics/E16-design.md`](../epics/E16-design.md); options and measurements: [`docs/epics/E16-server.md`](../epics/E16-server.md)

## Context

The engine is server-authoritative: untrusted clients send commands, a trusted session names the
actor, every client sees only its own projection (ADR-0004 point 10, ADR-0008). Training never
crosses a network (ADR-0011), so the server is no hot path: it serves play against bots and people,
the client lane's viewers and game records. Anyone should be able to run one (user, 2026-10-08):
one binary, few GPLv2-compatible dependencies, browser clients welcome. *Sans-I/O*: pure functions
from inputs to outputs, with sockets, clock and files outside.

## Decision

1. **Two adapter crates, split sans-I/O.** `h3-protocol` holds the *published language* (wire
   messages, their encoding and bounds, the mapping to and from engine types) and the *session
   core* (seats, opaque tokens, sequence numbers, the authoritative loop over one game), pure
   functions of their inputs and a `now` the caller passes. It is a logic crate (`no_std` +
   `alloc`, `check-arch`, the 100 % gate), so the decoder at the trust boundary is panic-free.
   `h3-server` is the shell: sockets, clock, OS randomness, files, configuration, bot seats.
2. **Transport: WebSocket** (`tokio-tungstenite`), one connection per client, one message per text
   frame. No HTTP API (lobby and ladder are messages too); no TLS in the server (a reverse proxy).
3. **`tokio` for the connections, an OS thread per game** (SRV-Q5). A task per connection waits on
   socket, outbound queue and idle timer; each game's engine runs single-threaded on its own OS
   thread, never on the runtime's workers; channels are bounded, a connection whose outbound queue
   is full is dropped. Features `rt-multi-thread`, `net`, `time`, `sync`, `macros` and, for the
   graceful shutdown, `signal` (SRV-Q12 (c), amended at E16 M2 T5a) only. The transport is one
   internal trait; the session core knows no runtime.
4. **JSON in serde's default, externally tagged enum form** (`{"Move":{...}}`): struct variants,
   `deny_unknown_fields`, no `tag`, `untagged` or `flatten`, so the same types encode in a
   non-self-describing format (`postcard`, negotiable later). Integers only; 64-bit values as
   strings; bulk data as packed arrays. One `PROTOCOL_VERSION` in the hello, bumped on every
   change; before 1.0 a server accepts exactly its own. The sim's encodings are not the client
   format (an *agent seat* may request them later).
5. **The wire types are the protocol's own.** Only `h3-protocol`'s mapping module knows both
   sides; its matches on the engine's enums are exhaustive, so a new engine variant fails
   `h3-protocol`'s build until its wire variant and arm exist (amended 2026-10-10). Messages
   carry dense handles; keys are in the manifest.
6. **Identity is the seat of the connection**, never a payload field (commands name no side or
   player). A seat is taken, and re-taken after a disconnect, with a server-issued token (128 bits
   of OS randomness); accounts bind seats later (identity ADR); bot seats are the server's.
7. **Everything outbound is projected per viewer** with the engine's `observe` and `project`.
   Spectators come with the battle's spectator viewer (before M2, SRV-Q4). Sequence numbers are
   per viewer. A command is acknowledged by the client's own sequence number: `Accepted` or
   `Rejected { code }` (the engine's error variants; text is the client's).
8. **Hidden information** (into `docs/06-security.md` in M1): the game seed is 64 bits of OS
   randomness, never derived from or revealed by anything a client sees; random obstacles come
   from a separate throwaway seed and enter the setup as pieces; seeds, RNG state, state hashes and
   tokens never reach a client or a log; the replay (ADR-0004 point 9) reaches the participants
   only after the game; before, a viewer may fetch its *record* (its own message log).
9. **Bounds, all configured:** frame size, messages per second per connection, one command in
   flight per seat, connections in total and per address, idle timeout. Decoder and session core
   are property-tested on arbitrary input, fuzzed once the harness exists (`05-testing.md`).
10. **One ruleset per game: the server's.** Clients run no rules in a hosted game. The *manifest*
    sent at join names the content (the replay header's fields) and carries the handle-to-key
    tables; a client previews with its own ruleset only when its hash equals the manifest's. The
    server never trusts a client-computed value.
11. **The server adds no rules.** It loads packs and scenarios as the CLI does and calls `apply`,
    `legal_commands`, `observe` and `project`; rules, rewards and encodings stay where they are.
12. **Journal, restore, integrity.** Every accepted command is appended to the game's journal and
    synced before it is acknowledged; the journal (header, seed, setup, commands, a state hash
    every N commands) is the replay in progress. Restart and resume re-run it (exact by ADR-0003);
    a journal whose engine or content identity changed ends the game void. A result counts only
    after a fresh re-run matched every recorded hash; a shadow instance is an option.

## Consequences

Clients speak documented JSON, and engine churn reaches the wire only through the mapping; the session core
is deterministic, scripted-client testable and gets a workload under `benches/server/`; "same
content for everyone" holds by construction; a crash costs a re-run, no state serialization exists.
Costs: a mapping to maintain, in step with every new engine variant; an async shell (cancellation, back-pressure); hundreds of games per
server (a thread each); a sync per command; long journals re-run on restore (state saves later); a
handle-to-key table for every content kind.

## Alternatives considered

Std threads with blocking I/O (a `tungstenite` socket has one reader-writer thread, so every
connection polls; `tokio` costs 11 crates and no build time, SRV-Q5). An HTTP framework (heavy,
not needed). Plain TCP or QUIC (no browsers; QUIC brings TLS). A binary format first (premature).
Internally tagged JSON (no binary codec can decode it). Serde on the engine's types (the wire
follows every internal change). Wildcard arms to an `Other` variant (the first point 5): arms no
test can reach while every variant is mapped, so a coverage exclusion, and clients that must
handle an unknown kind. Lockstep clients (excluded by ADR-0004; lets
different content diverge). Serializing live state (a second format, when the journal suffices).

## Revision history

- 2026-10-10: point 5 amended (E16 M1 T1; the user): exhaustive matches instead of wildcard arms
  to `Unsupported` and `Other`, which could never run while every variant is mapped and needed a
  coverage exclusion. A new engine variant now fails the build until it is mapped; the
  consequences and alternatives follow.
