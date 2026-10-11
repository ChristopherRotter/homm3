# ADR-0019: Player identity, lobbies, ranked ladders and matchmaking

- Status: Accepted (user, 2026-10-11, the E16 M3 refinement, SRV-Q17; directions from SRV-Q8 to
  SRV-Q10 and SRV-Q13). Points 4 to 9 (ladders, matchmaking, rating, pauses on a ladder, bots)
  are direction, amended at the M4 refinement.
- Date: 2026-10-11 (drafted 2026-10-08)
- Deciders: user; server lane sessions (E16 refinement, M3 refinement)
- Working detail (names, derivation, login, the player log): [`docs/epics/E16-design.md`](../epics/E16-design.md) §10; options: [`docs/epics/E16-server.md`](../epics/E16-server.md)

<!-- Decisions only. Reasoning and options: docs/epics/E16-server.md; detail: E16-design.md. -->

## Context

Anyone can host a server and anyone can connect to the servers of others; a server may run open
lobbies and one or more ranked ladders with leaderboards, matchmaking and player ratings. Every
server is independent: nothing is synchronized between servers and there is no global ladder
(the user, 2026-10-08). A ladder needs accounts that survive a change of device and results
that cannot be forged, and running a server must stay simple: one binary, a data directory, no
database system to install.

## Decision

1. **An account is a login name and a password, and the password never leaves the device.** The
   client derives a signing keypair (Ed25519) from them with a memory-hard key derivation
   (Argon2id, salted by the login name); a server stores the login name and the public key,
   never a password or a hash of one. The *login name* is permanent (a new one is a new
   account); other clients see the *display name*, unique on its server and changeable; the
   login name and the public key are shown to their owner only (the user, SRV-Q17). A login is a
   signature over a fresh challenge and the server's own id (the address its clients dial), so
   it can be neither replayed nor relayed to another server. Any device given the login name and
   the password has the account; a password change re-keys the account on each server at the
   next login, signed by both keys. A weak password can be attacked only by whoever holds a
   server's data directory, as with server-side hashes. Anonymous play (seat tokens) stays
   where the server's configuration permits it, by default; never on a ladder. One
   `Authenticator` seam in `h3-server` holds this. The crates (the user, SRV-Q17):
   `ed25519-dalek` (the server checks signatures) and `argon2` (the derivation: a client's, and
   the server's test client's); pure Rust, BSD-3-Clause and MIT/Apache-2.0.
2. **Lobbies.** A server lists its open games. A host creates a game from a *game
   configuration* (packs, profile, scenario or map, seats with people or bots, options such as
   timers); players take seats or spectate; the configuration is visible before joining and
   fixed when the game starts. The server's own content applies (ADR-0018, point 10). A
   finished game's replay is public (ADR-0018, point 8) unless its host closed the game to
   spectators or a seated account opted out (a preference of the account): then it is archived
   unlisted, for the operator (the user, SRV-Q17).
3. **Servers are independent.** A ladder, its ratings and its accounts belong to one server;
   nothing is synchronized, there is no global ladder and none is planned. The same login name
   and password give the same key on every server, which is a convenience, not a link: each
   server's record of a player stands alone.
4. **A ladder** is a fixed game configuration (its ruleset hash included), a scenario pool, a
   rating system, a leaderboard and a season number. A content change on the server starts a
   new season.
5. **Matchmaking:** one queue per ladder; the matcher pairs players by rating distance with a
   window that widens with waiting time; a match creates the game with a scenario from the pool
   and random sides; both players accept within a time limit or return to the queue.
6. **Rating: Glicko-2** with the published defaults, computed in `h3-server` (not game logic:
   floats are allowed in adapters), updated in the order games end. A result counts only after
   the game's journal re-ran and verified (ADR-0018, point 12) and only when both seats
   were held by their players. A disconnect beyond the grace period is a forfeit; an abandoned
   game is a loss for the absent player.
7. **Persistence is files, not a database system.** The data directory holds the game
   journals (ADR-0018, point 12) and append-only JSON-lines logs of players, name changes,
   finished games and results, each line synced when written, rebuilt into memory at start,
   with periodic snapshots to bound the start time. Ladder results are therefore durable,
   crash-consistent (a torn last line is discarded) and readable with any text tool. An
   embedded store is a later option for a server that outgrows in-memory tables (a heavy
   dependency: a user decision then); journals and logs stay the source of truth either way.
8. **Pause and resume.** An unranked game pauses when its host or all its seats ask; a ranked
   game has no host and pauses only when both players agree (one proposes, the other accepts)
   and within the ladder's limit, else the absent player forfeits. A player whose ranked game
   on the server is not over starts no other ranked game there (both rules the user's,
   2026-10-11). The pause's length is proposed within the ladder's bounds and agreed; at its due
   time the disconnect grace runs, and a player not back forfeits (both absent: void, a no-show
   each; the user, SRV-Q13). A paused game is unloaded and listed as resumable for its
   participants; it resumes from its journal when every seat is taken again (by account on a
   ladder, by token otherwise).
9. **Bots on ladders.** A server may enter its own bot seats (a scripted or trained agent)
   under a bot account, flagged as such on the leaderboard: the AI plays people, and their
   games become records.
10. **Internet hosting.** Session integrity over the internet needs TLS; the login does not (a
    signed challenge is not a secret). The server does not implement TLS: a reverse proxy in
    front of it is the documented setup; native TLS is a later option behind the transport
    trait (ADR-0018, point 3).

## Consequences

A player signs up by choosing a login name and a password in the client, and a display name on
each server; a new device needs nothing but the first two. Servers hold no secrets: a leaked data
directory exposes names, public keys, ratings, records and finished replays, nothing a login
can be forged from. Ladders stay honest by the verified journal and by seat binding, not by
trusting clients. Costs: two crypto crates in server and client; a forgotten password is a lost
account unless the operator re-binds the name by hand; no shared leaderboard across servers,
by design.

## Alternatives considered

A keypair generated per device (the first draft): no password anywhere, but a new device needs
the key file, which the user found insufficient. One name that salts the key and can be renamed:
a renamed account could no longer log in from a new device. A random salt the server stores and
hands out before the login: renames work, but a hostile server could choose the salt. Classic password accounts hashed on the
server: the fallback; the password reaches the server (TLS needed for every login), every host
keeps a hash, and the client needs a sign-up flow per server. An external identity provider
(OAuth): needs an HTTP stack, TLS and a central party. Elo instead of Glicko-2: simpler, but no
notion of rating uncertainty for new or inactive players. A database system from the start:
more to install for everyone who hosts, and the journals make it unnecessary for correctness.
