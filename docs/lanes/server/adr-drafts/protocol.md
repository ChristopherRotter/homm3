# ADR-NNNN: Server and protocol: hosted games over a sans-I/O protocol core

- Status: Proposed (E16 refinement draft, 2026-10-08; corrected with the user's answers to
  SRV-Q1 to SRV-Q11 on 2026-10-09; to be accepted at the M0 checkpoint)
- Date: 2026-10-09
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
   text frame, through `tokio-tungstenite`. No HTTP API: lobby and ladder
   operations are messages on the same connection. No TLS inside the server: loopback or LAN
   first; on the internet a reverse proxy terminates TLS (the ADR on identity, point 8).
3. **Concurrency: `tokio` for the connections, an OS thread per game** (SRV-Q5). The accept
   loop and one task per connection, which waits on its socket, its outbound queue and its
   idle timer at once; every hosted game on its own OS thread owning its engine instance
   (logic stays single-threaded per game, ADR-0003); bounded channels between them, and a
   connection whose outbound queue is full is dropped. Engine work never runs on the
   runtime's worker threads. Only the runtime features the shell needs (`rt-multi-thread`,
   `net`, `time`, `sync`, `macros`); no HTTP framework. The transport is one small internal
   trait, so another transport never touches the session core, which knows no runtime.
4. **Wire format: JSON** (`serde` + `serde_json`, already in the workspace) in serde's default,
   externally tagged enum form (`{"Move":{...}}`) with struct variants and
   `deny_unknown_fields`; no `tag`, `untagged` or `flatten` attributes, so the same types also
   encode in a format that is not self-describing (SRV-Q5). Integers only; a 64-bit value
   travels as a string; bulk data (terrain, fog, masks) as packed arrays. One
   `PROTOCOL_VERSION` in `h3-protocol`, sent in the hello and bumped on every change; before
   1.0 a server accepts exactly its own version. A binary encoding (`postcard`), if whole-game
   snapshots ever need it, is negotiated in the hello on the same types. The sim's action and
   observation encodings are not the client format; an *agent seat* may request them later
   (additive).
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
   A spectator is a viewer of its own: the battle context gains a spectator viewer for
   `observe` and `project` before M2 (SRV-Q4, a battle-lane task); no spectators before it.
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

Clients speak documented JSON over WebSocket; the battle lane's internal churn (the state hash
version is bumped by most increments) never changes the wire by itself; the session core is deterministic, testable
with scripted clients and a fake clock, and gets an instruction-count workload under
`benches/server/`; "same content for everyone" holds by construction; a crash or a pause costs
nothing but a re-run of the journal, and no state serialization exists on the server. Costs: a
mapping to maintain (new engine variants are "other" until mapped; the lane HANDOFF tracks
them); an async shell (cancellation and back-pressure need care; the session core stays synchronous);
a thread per hosted game bounds a server to hundreds of concurrent games (enough); a
sync per command (milliseconds, fine for turn-based play); long adventure games re-run long
journals on restore (state saves, ADR-0004 point 9, can shorten that later); the manifest needs
the ruleset's key tables (`h3-content` has `creature_key` and the `*_by_key` lookups; the
reverse direction for every kind is checked in M1).

## Alternatives considered

Std threads with blocking I/O: no runtime, but a `tungstenite` socket is read and written by one
thread only, so every connection thread polls (20–50 ms per wake-up); measured, `tokio` adds 11
crates and no build time to the skeleton (SRV-Q5). An HTTP framework (`axum`, `hyper`): heavy, and
lobby and ladder operations need none.
Plain TCP with own framing, or QUIC: no browser clients; QUIC pulls in a TLS stack. A binary
wire format first: premature, the hot path is not here; the default enum form keeps it one codec
away. Internally tagged JSON (`{"type":"Move",...}`): slightly more readable, but no
non-self-describing format can decode it. `serde` derives on the engine's types:
couples the wire to internal layouts and every version bump, with no place for `Unsupported`.
Clients running the rules themselves (lockstep): excluded by ADR-0004, and it is exactly what
lets players with different content or versions diverge. Serializing live state for crash
recovery: a second format to version and test, when the journal plus determinism already gives
the exact state.
