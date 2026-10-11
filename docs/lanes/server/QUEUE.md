# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, its checkpoint passed, SRV-Q16). M3 refined 2026-10-11 (SRV-Q17): ADR-0019, accepted at the refinement (points 1 and 2 for M3; the design note §10), cut below, each task about half a context; T1 done 2026-10-11 (the login on the wire, wire 12); T2a done 2026-10-11 (logins checked, the player log); T2b done 2026-10-11 (re-keys, renames, the preference, the operator's rebind). Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`.

### E16 M3 T3 — Seats bound to accounts

- **Scope:** a logged-in connection's seat belongs to its account: re-taken by any connection logged in as it, without a token; an anonymous seat keeps its token. The journal names a seat's account (`JOURNAL_FORMAT_VERSION` 4); a restore binds the seats again. The lobby shows the seated display names. A finished game's replay is listed only if its host left it open and no seated account opted out (ADR-0018 point 8 amended in a shared change). Reconnect by account after a drop or a restart.
- **Acceptance:** the Definition of Done; end to end: a seat re-taken by account from a new connection and after a restart; another account refused; the replay's listing by preference.
- **Depends on:** T2b (done 2026-10-11): an account's `listed` is in `players::Account`; the crash tests' `crashed()` copies only the journals (copy `players.log` too once seats name accounts).
- **Status:** ready.

### E16 M3 T4a — A game's bounds

- **Scope:** the commands-per-game limit (`[limits]`; the game void beyond it, a goodbye reason), sized so a replay fits `outbound_bytes`; a waiting game whose seats are all empty leaves, its spectators told (the M2 review: a spectator kept such a game listed). The M2 inbox's two items.
- **Acceptance:** the Definition of Done; end to end: a game past its limit is void; a waiting game left by its creator leaves with a spectator bound.
- **Depends on:** T1 (done 2026-10-11).
- **Status:** ready.

### E16 M3 T4b — Behind a reverse proxy

- **Scope:** `[server] trusted_proxies`: a connection from one of them is counted by the client's address its forwarded header names (the WebSocket handshake's `X-Forwarded-For`, its last entry; *provisional*); the per-address limits by IPv6 /64 prefix; the documented setup (a TLS-terminating reverse proxy in front of the server, an example configuration, `public_url`). The M2 inbox's item.
- **Acceptance:** the Definition of Done; end to end: two clients behind a trusted proxy count as their own addresses, a forwarded header from an untrusted peer is ignored; the setup in the server's docs.
- **Depends on:** T2a (done 2026-10-11; `public_url`).
- **Status:** ready.

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
- 2026-10-11 (M3 T2b; **T3**, which tracks connections by account): after a re-key the account's other connections, logged in with the old key, stay logged in until they close: a player who re-keys after a leaked password keeps an attacker's open session alive. Close them (a goodbye reason) once the registry knows an account's connections.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
