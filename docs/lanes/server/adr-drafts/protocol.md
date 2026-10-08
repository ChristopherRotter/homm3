# ADR-NNNN: Server and protocol: hosted games over a sans-I/O protocol core

- Status: Proposed (E16 refinement draft, 2026-10-08; under the user's review; to be decided at
  the M0 checkpoint, the first milestone that needs it)
- Date: 2026-10-08
- Deciders: user + server refinement session

<!-- Decisions only. The reasoning, the concepts and the options are in docs/epics/E16-server.md;
     the working detail (messages, flows) in docs/epics/E16-design.md. Longer than 80 lines
     because it covers the whole hosting surface at once, as the refinement brief asked. -->

## Context

The engine is server-authoritative: untrusted clients send commands, a trusted session says who
the actor is, and every client sees only its own projection of the game (ADR-0004 point 10,
ADR-0008). Training never crosses a network (ADR-0011: the C ABI), so the server is not a hot
path; it exists for play against bots and other people, for the client lane's viewers and for
game records. Anyone should be able to run a server (the user, 2026-10-08), so it must be one
binary with few dependencies, each GPLv2-compatible (`deny.toml`), and a browser client must be
able to connect (web/WASM, `docs/01-vision.md`). *Sans-I/O* below means code that turns inputs
into outputs as pure functions, with sockets, clock and files kept outside it.

## Decision

1. **Two adapter crates, split sans-I/O.** `h3-protocol` holds the *published language* (the
   wire messages, their encoding and bounds, the mapping to and from engine types) and the
   *session core* (seats, tokens as opaque values, sequence numbers, the authoritative command
   loop over one game), all as pure functions of their inputs and a `now` the caller passes.
   It is a logic crate (`no_std` + `alloc`, `check-arch` purity, the 100 % coverage gate), so
   the decoder at the trust boundary is panic-free by construction. `h3-server` is the shell:
   threads, sockets, clock, OS randomness, files, configuration, the binary, the bot seats.
2. **Transport: WebSocket** (RFC 6455) over TCP, one connection per client, one message per
   text frame, through the blocking `tungstenite` API. No HTTP API: lobby and ladder
   operations are messages on the same connection. No TLS inside the server: loopback or LAN
   first; on the internet a reverse proxy terminates TLS (the ADR on identity, point 8).
3. **Concurrency: std threads, blocking I/O, no async runtime.** One reader thread per
   connection, one thread per hosted game owning its engine instance (logic stays
   single-threaded per game, ADR-0003), channels between them. The transport is one small
   internal trait, so another transport or a runtime never touches the session core.
4. **Wire format: JSON** (`serde` + `serde_json`, already in the workspace), internally tagged
   enums with struct variants and `deny_unknown_fields`. One `PROTOCOL_VERSION` in
   `h3-protocol`, sent in the hello and bumped on every change; before 1.0 a server accepts
   exactly its own version. A binary encoding, if ever needed, is negotiated in the hello. The
   sim's action and observation encodings are not the client format; an *agent seat* may
   request them later (additive).
5. **The wire types are the protocol's own, never the engine's.** `h3-protocol`'s mapping
   module is the only code that knows both. Its matches on engine commands, events and legal
   commands carry a wildcard arm to `Unsupported` or `Other { kind }`, so a new battle command
   or event compiles without the server lane and shows as "other" until mapped (the lane's one
   reviewed coverage exclusion). Messages carry the ruleset's dense handles; keys are in the
   manifest (point 10).
6. **Identity is the seat of the connection.** The actor the engine sees is the seat the
   connection holds, never a payload field (commands carry no side or player). A seat is taken
   with a server-issued token (128 bits of OS randomness) and re-taken with it after a
   disconnect. Accounts bind seats to persistent identities (the ADR on identity); a bot seat
   is run by the server.
7. **Everything outbound is projected per viewer** with the engine's `observe` and `project`.
   Spectators get public events only until the battle context offers a spectator viewer.
   Sequence numbers are per viewer (a global event index would show hidden events as gaps).
   Commands are acknowledged by the client's own sequence number with a typed code
   (`Accepted`, `Rejected { code }`); the engine's error variants are the codes, text is the
   client's.
8. **Hidden-information rules of the server** (to be written into `docs/06-security.md`):
   the game seed is 64 bits of OS randomness, never derived from or revealed by anything a
   client sees; random obstacles come from a separate throwaway seed and enter the setup as
   pieces; seeds, RNG state, state hashes and tokens never reach a client or the log; the full
   replay (ADR-0004 point 9) goes to a game's participants only after it ends; before that a
   viewer may fetch its *record* (its own projected message log, not re-runnable).
9. **Bounds, all configured:** frame size, messages per second per connection, one command in
   flight per seat, connections in total and per address, idle timeout. Decoding is bounded
   and panic-free; decoder and session core are property-tested on arbitrary input and fuzzed
   once the harness exists (`docs/05-testing.md`, level 6).
10. **One ruleset per game: the server's.** Clients run no rules in a hosted game, so a
    client's version, local configuration or mode cannot change it. The *manifest* sent at
    join identifies the content (pack ids and versions, profile, ruleset hash, engine and
    protocol versions: the replay header's fields) and carries the handle-to-key tables, so
    clients find assets by key. A client may use its own compiled ruleset for previews only
    when its hash equals the manifest's; otherwise it asks the server (query messages). The
    server never trusts a client-computed value.
11. **The server adds no rules.** It loads packs and scenarios as the CLI does, calls the
    engine's `apply`, `legal_commands`, `observe` and `project`, and keeps the command log,
    which is the replay. Rules, rewards and encodings stay where they are.
12. **Journal, restore and integrity.** Every accepted command is appended to the game's
    *journal* on disk and synced before it is acknowledged; the journal is the replay in
    progress (header, seed, setup, commands, the live state's hash every N commands). After a
    restart the server restores every open game by re-running its journal through the engine,
    which determinism makes exact (ADR-0003); a paused game resumes the same way. A journal
    whose header no longer matches the server's engine and content is not restored: the game
    ends void with its replay available, so operators drain open games before an upgrade.
    Before a game's result is used anywhere (a ladder, a leaderboard) the server re-runs the
    journal in a fresh instance and compares every recorded hash; a mismatch, whatever its
    cause (corrupted memory, a bug), marks the game corrupt and void, with the diagnosis kept.
    A *shadow instance* compared after every command is a configuration option: it detects
    corruption at once and lets the server restore the correct state from the journal and
    continue.

## Consequences

Clients speak documented JSON over WebSocket; the battle lane's internal churn (33 state-hash
versions so far) never changes the wire by itself; the session core is deterministic, testable
with scripted clients and a fake clock, and gets an instruction-count workload under
`benches/server/`; "same content for everyone" holds by construction; a crash or a pause costs
nothing but a re-run of the journal, and no state serialization exists on the server. Costs: a
mapping to maintain (new engine variants are "other" until mapped; the lane HANDOFF tracks
them); blocking threads scale to hundreds of connections, not tens of thousands (enough); a
sync per command (milliseconds, fine for turn-based play); long adventure games re-run long
journals on restore (state saves, ADR-0004 point 9, can shorten that later); the manifest needs
the ruleset's key tables (`h3-content` has `creature_key` and the `*_by_key` lookups; the
reverse direction for every kind is checked in M1).

## Alternatives considered

An async runtime (`tokio`, `axum`): the ecosystem default, but a large dependency tree and an
async session core for a load that does not need it; the transport trait keeps the door open.
Plain TCP with own framing, or QUIC: no browser clients; QUIC pulls in a TLS stack. A binary
wire format first: premature, the hot path is not here. `serde` derives on the engine's types:
couples the wire to internal layouts and every version bump, with no place for `Unsupported`.
Clients running the rules themselves (lockstep): excluded by ADR-0004, and it is exactly what
lets players with different content or versions diverge. Serializing live state for crash
recovery: a second format to version and test, when the journal plus determinism already gives
the exact state.
