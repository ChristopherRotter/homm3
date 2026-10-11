# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, its checkpoint passed, SRV-Q16). M3 refined 2026-10-11 (SRV-Q17): ADR-0019, accepted at the refinement (points 1 and 2 for M3; the design note §10), cut below, each task about half a context. Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`.

### E16 M3 T1 — The login on the wire, sans-I/O

- **Scope:** `h3-protocol`: `Welcome` gains the server's id and the connection's challenge (given by the host); `Login { login, public_key, signature, display }` answered `LoggedIn { display }` or a refusal; `Rekey`, `Rename`, the account's listing preference (their handling comes in T2b); keys, signatures and challenges as hex strings. The hello gate's login step, sans-I/O: the gate hands the host the login to check (the `Authenticator` seam is `h3-server`'s, T2a) and is told the outcome; a login before the hello, a second login, a malformed key or signature refused. The rules of a login name (3–20 of `a–z 0–9 _ -`, lowercase) and of a display name (provisional), as pure functions. `E16-design.md` §10 is the reference.
- **Acceptance:** the Definition of Done; `h3-protocol` 100 % covered; the round trips and JSON forms of the new messages; properties: a connection is logged in only after the host's yes, a refused login changes nothing; `PROTOCOL_VERSION` bumped.
- **Depends on:** the M3 refinement (SRV-Q17).
- **Status:** in progress (`claude/epic-bell-ot2nsa`, 2026-10-11 20:00 UTC).

### E16 M3 T2a — The Authenticator and the player log

- **Scope:** `h3-server`: `ed25519-dalek` (3) behind an `Authenticator` trait: the signature checked over `"h3-login-v1"`, `[server] public_url` and the connection's challenge (32 bytes of OS randomness); a new key with a free login name and a free display name registers, a known pair logs in, anything else is refused alike; failed logins count against the address's rate. `<data>/players.log` (JSON lines, owner-only, synced before the answer, read back at start, a torn line dropped). `[server] anonymous` (default true): without it, a connection must log in before it joins or creates a game. The test client derives its key with `argon2` (0.6; `E16-design.md` §10) and a test vector fixes the derivation. `deny.toml` passes with the new crates.
- **Acceptance:** the Definition of Done; end to end: registration, login, a restart keeps the account, a login signed for another `public_url`, with another connection's challenge or by another key refused, anonymous play on and off.
- **Depends on:** T1.
- **Status:** ready once T1 is done.

### E16 M3 T2b — Re-keying, renames, the listing preference, the operator's rebind

- **Scope:** `Rekey` (signed by the old and the new key) rebinds the account; `Rename` changes the display name (unique on the server, ignoring case); the account's `listed` preference; each a line in the player log. The operator's rebind of a login name to a new key (a subcommand of the server's binary that appends to the player log while the server is stopped, *provisional*).
- **Acceptance:** the Definition of Done; end to end: after a re-key only the new key logs in, also after a restart; a taken display name refused; the rebind.
- **Depends on:** T2a.
- **Status:** ready once T2a is done.

### E16 M3 T3 — Seats bound to accounts

- **Scope:** a logged-in connection's seat belongs to its account: re-taken by any connection logged in as it, without a token; an anonymous seat keeps its token. The journal names a seat's account (`JOURNAL_FORMAT_VERSION` 4); a restore binds the seats again. The lobby shows the seated display names. A finished game's replay is listed only if its host left it open and no seated account opted out (ADR-0018 point 8 amended in a shared change). Reconnect by account after a drop or a restart.
- **Acceptance:** the Definition of Done; end to end: a seat re-taken by account from a new connection and after a restart; another account refused; the replay's listing by preference.
- **Depends on:** T2b.
- **Status:** ready once T2b is done.

### E16 M3 T4a — A game's bounds

- **Scope:** the commands-per-game limit (`[limits]`; the game void beyond it, a goodbye reason), sized so a replay fits `outbound_bytes`; a waiting game whose seats are all empty leaves, its spectators told (the M2 review: a spectator kept such a game listed). The M2 inbox's two items.
- **Acceptance:** the Definition of Done; end to end: a game past its limit is void; a waiting game left by its creator leaves with a spectator bound.
- **Depends on:** T1 (the wire version).
- **Status:** ready once T1 is done.

### E16 M3 T4b — Behind a reverse proxy

- **Scope:** `[server] trusted_proxies`: a connection from one of them is counted by the client's address its forwarded header names (the WebSocket handshake's `X-Forwarded-For`, its last entry; *provisional*); the per-address limits by IPv6 /64 prefix; the documented setup (a TLS-terminating reverse proxy in front of the server, an example configuration, `public_url`). The M2 inbox's item.
- **Acceptance:** the Definition of Done; end to end: two clients behind a trusted proxy count as their own addresses, a forwarded header from an untrusted peer is ignored; the setup in the server's docs.
- **Depends on:** T2a (`public_url`).
- **Status:** ready once T2a is done.

### E16 M3 T5 — M3's closure

- **Scope:** `06-security.md`'s accounts (a shared change); the client lane's note (the derivation and its test vector, checking the server's id before signing, caching the key; an inbox line, cross-lane); M3's exit criteria checked; a fresh reviewer agent over M3; the user's checkpoint.
- **Acceptance:** M3's exit (epic file).
- **Depends on:** T1–T4b.
- **Status:** waiting (T1–T4b).

## Inbox

- 2026-10-11 (M2 T2; **M4**): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) when the ladders' results log needs ids unique for a server's life. Replay ids are unique already (a random part).
- 2026-10-11 (M2 T3; **M4**): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count, verify before the end is told, or tell a void game's players.
- 2026-10-11 (M2 T5b; **M4**, with the bot accounts on ladders): a separate rate for automated seats (T3's note): an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (the user, SRV-Q13; **M4's refinement**): ranked games one at a time per player (confirm the reading), a pause only by agreement, its length negotiated within the ladder's bounds, the absent player forfeits at the due time.
- 2026-10-11 (M2 T5e's review; **M4**, with the pause by agreement): a seat that is not the host can repeat `Pause`, and each `PauseAsked` fills every viewer's record window: one ask per seat until the game moves on.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
