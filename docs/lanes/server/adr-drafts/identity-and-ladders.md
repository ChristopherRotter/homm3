# ADR-NNNN: Player identity, lobbies, ranked ladders and matchmaking

- Status: Proposed (E16 refinement draft, 2026-10-08; under the user's review)
- Date: 2026-10-08
- Deciders: user + server refinement session

<!-- Decisions only. Reasoning and options: docs/epics/E16-server.md; detail: E16-design.md. -->

## Context

Anyone can host a server and anyone can connect to the servers of others; a server may run open
lobbies and one or more ranked ladders with leaderboards, matchmaking and player ratings (the
user, 2026-10-08). A ladder needs identities that persist across connections and results that
cannot be forged. There is no central authority in a self-hosted world, so identity must work
without one, and running a server must stay simple: one binary, a data directory, no database.

## Decision

1. **A player's identity is a keypair** (Ed25519). The public key is the identity; a server
   stores it with the display name chosen on that server; a login is a signed challenge; no
   password exists anywhere; one identity works on every server. Anonymous connections are
   allowed where the server's configuration permits (open lobbies), never on a ladder. This sits
   behind one `Authenticator` seam in `h3-server`; a central identity provider is a later
   option behind the same seam. The signature crate is a user decision (`ed25519-dalek`,
   BSD-3-Clause, is the candidate).
2. **Lobbies.** A server lists its open games. A host creates a game from a *game
   configuration* (packs, profile, scenario or map, seats with people or bots, options such as
   timers); players take seats or spectate; the configuration is visible before joining and
   fixed when the game starts. The server's own content applies (protocol ADR, point 10).
3. **A ladder** is a fixed game configuration (its ruleset hash included), a scenario pool, a
   rating system, a leaderboard and a season number. A content change on the server starts a
   new season. Ladders belong to their server; there is no cross-server rating.
4. **Matchmaking:** one queue per ladder; the matcher pairs players by rating distance with a
   window that widens with waiting time; a match creates the game with a scenario from the pool
   and random sides; both players accept within a time limit or return to the queue.
5. **Rating: Glicko-2** with the published defaults, computed in `h3-server` (not game logic:
   floats are allowed in adapters), updated in the order games end. A result counts only when
   the server's own replay of the game verifies and both seats were held by their players. A
   disconnect beyond the grace period is a forfeit; an abandoned game is a loss for the absent
   player.
6. **Persistence without a database:** append-only JSON-lines logs (players, games, results)
   in the server's data directory, rebuilt at start, with snapshots for speed. An embedded
   store is a later option when a server outgrows it (a heavy dependency: a user decision then).
7. **Bots on ladders.** A server may enter its own bot seats (a scripted or trained agent)
   under a bot identity, flagged as such on the leaderboard: the AI plays people, and their
   games become records.
8. **Internet hosting.** Session integrity over the internet needs TLS; identity does not (a
   signed challenge is not a secret). The server does not implement TLS: a reverse proxy in
   front of it is the documented setup; native TLS is a later option behind the transport
   trait (protocol ADR, point 3).

## Consequences

Players sign up nowhere: a client generates a key once and is known on every server it joins by
its chosen name there. Servers hold no secrets, so a leaked data directory exposes names,
ratings and records only. Ladders stay honest by the verified replay and by seat binding, not
by trusting clients. Costs: a lost key is a lost identity unless the server's operator re-binds
a name by hand; one crypto dependency; no shared leaderboard across servers (an aggregator over
signed results could come later without changing this).

## Alternatives considered

Password accounts per server: passwords get reused across many hosts, every host stores a hash,
and the client needs a sign-up flow per server. An external identity provider (OAuth): needs an
HTTP stack, TLS and a central party. Bearer tokens issued by each server: dependency-free and
simple, but a per-server secret that TLS must protect and no cross-server identity; it stays the
fallback if the user rejects the crypto dependency. Elo instead of Glicko-2: simpler, but no
notion of rating uncertainty for new or inactive players. A database from the start: more to
install for everyone who hosts.
