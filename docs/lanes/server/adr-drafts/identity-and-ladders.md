# ADR-NNNN: Player identity, lobbies, ranked ladders and matchmaking

- Status: Proposed (E16 refinement draft, 2026-10-08; direction confirmed by the user on
  2026-10-09: the password-derived keypair (SRV-Q8), plain files (SRV-Q9), ladders before the
  whole game (SRV-Q10); to be decided at the M3 refinement, the first milestone that needs it)
- Date: 2026-10-09
- Deciders: user + server refinement session

<!-- Decisions only. Reasoning and options: docs/epics/E16-server.md; detail: E16-design.md. -->

## Context

Anyone can host a server and anyone can connect to the servers of others; a server may run open
lobbies and one or more ranked ladders with leaderboards, matchmaking and player ratings. Every
server is independent: nothing is synchronized between servers and there is no global ladder
(the user, 2026-10-08). A ladder needs accounts that survive a change of device and results
that cannot be forged, and running a server must stay simple: one binary, a data directory, no
database system to install.

## Decision

1. **An account is a name and a password, and the password never leaves the device.** The
   client derives a signing keypair (Ed25519) from name and password with a memory-hard key
   derivation (Argon2id, the name as salt); a server stores the name and the public key, never
   a password or a hash of one; a login is a signed challenge; any device given the name and
   the password has the account; a password change re-keys the account on each server at the
   next login, signed by both keys. Public keys are shown to their owner only, never to other
   clients, so a weak password can be attacked only by whoever holds a server's data directory,
   as with server-side hashes. Anonymous connections are allowed where the server's
   configuration permits (open lobbies), never on a ladder. One `Authenticator` seam in
   `h3-server` holds this. The crates (`ed25519-dalek`, `argon2`; pure Rust, BSD-3-Clause and
   MIT/Apache-2.0) are a user decision; classic server-side password hashes are the fallback.
2. **Lobbies.** A server lists its open games. A host creates a game from a *game
   configuration* (packs, profile, scenario or map, seats with people or bots, options such as
   timers); players take seats or spectate; the configuration is visible before joining and
   fixed when the game starts. The server's own content applies (ADR-0018, point 10).
3. **Servers are independent.** A ladder, its ratings and its accounts belong to one server;
   nothing is synchronized, there is no global ladder and none is planned. The same name and
   password give the same key on every server, which is a convenience, not a link: each
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

A player signs up by choosing a name and a password in the client and is known on every server
by them; a new device needs nothing but the two. Servers hold no secrets: a leaked data
directory exposes names, public keys, ratings, records and finished replays, nothing a login
can be forged from. Ladders stay honest by the verified journal and by seat binding, not by
trusting clients. Costs: two crypto crates in server and client; a forgotten password is a lost
account unless the operator re-binds the name by hand; no shared leaderboard across servers,
by design.

## Alternatives considered

A keypair generated per device (the first draft): no password anywhere, but a new device needs
the key file, which the user found insufficient. Classic password accounts hashed on the
server: the fallback; the password reaches the server (TLS needed for every login), every host
keeps a hash, and the client needs a sign-up flow per server. An external identity provider
(OAuth): needs an HTTP stack, TLS and a central party. Elo instead of Glicko-2: simpler, but no
notion of rating uncertainty for new or inactive players. A database system from the start:
more to install for everyone who hosts, and the journals make it unnecessary for correctness.
