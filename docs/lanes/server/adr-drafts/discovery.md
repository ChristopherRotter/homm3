# ADR-NNNN: Server identity and decentralized discovery

- Status: Proposed (E16 refinement draft, 2026-10-08; under the user's review; to be decided
  with the identity ADR at the M3 refinement, built after M4)
- Date: 2026-10-08
- Deciders: user + server refinement session

<!-- Decisions only. Reasoning and options: docs/epics/E16-server.md; detail: E16-design.md. -->

## Context

Players need to find servers without a central registry, and need to know that the server they
reach is the one that was described to them (the user, 2026-10-08: a trusted, decentralized
mechanism). Servers are independent (the identity ADR); operators are hobbyists on home
connections with changing addresses; the ecosystem is small. "Trusted" has two parts: a
description cannot be forged or replayed, and a client reaches the server the description
names. No decentralized mechanism can say whether a server is *good*; vouching and the
player's own choice are the honest answer to that.

## Decision

1. **Every server has a keypair** (Ed25519, generated at first start, kept in the data
   directory). Its public key is the server's identity across address changes. In `Welcome`
   the server sends its key and a signature over the client's hello nonce, so a client knows
   it talks to the key's holder; clients pin a server's key on first contact and warn loudly
   when it changes (as SSH does). Channel integrity on the internet still needs TLS (the
   identity ADR, point 10): the signature proves who answered, not that nobody sits between.
2. **A server describes itself with a signed card:** name, description, addresses, key,
   protocol and engine versions, content (packs, profile, ruleset hash), ladders with their
   seasons and player counts, players online, whether anonymous play is allowed, an operator
   contact, issue and expiry times, and its *peers* (point 3). Any client gets it with
   `Describe`; the card is also a file the operator may put on any web page.
3. **Discovery is a web of trust between servers.** An operator lists in the configuration
   the servers they vouch for, by key and address. A server fetches its peers' cards over the
   same protocol (a WebSocket client in `h3-server`, no new dependency), verifies the
   signatures and the keys, caches them until they expire, and serves them with `ListPeers`,
   together with what those peers serve, bounded in hops and count. A client browses from the
   servers it already knows (its own, bookmarks, a few in the client's configuration),
   receives cards relayed by them, and shows each server with the chain that vouches for it.
   The client contacts a server only when the player picks it, so browsing reveals the
   player's address to no unknown server.
4. **Lists are cards signed by a list key.** Anyone may publish a signed list of server cards
   (a community list, a tournament's list); a server serves the lists it was given as files,
   a client may fetch lists itself where it has an HTTP client (the client lane's choice).
   Clients choose which list keys to trust, as they choose peers.
5. **On the LAN, servers announce themselves by mDNS** (DNS-SD, `_h3._tcp.local`, the key
   fingerprint in the record), an optional feature with one small crate, so a LAN game needs
   no addresses typed.
6. **Not built:** a central registry (a party to trust and to run), a DHT (a resolver of
   addresses by key would be the next step if the ecosystem outgrows vouching: signed mutable
   records in the BitTorrent mainline DHT keyed by the server's public key, as Pkarr does; a
   DHT crate, UDP and NAT traversal), server reputation (vouching chains are what a client
   shows; the judgement is the player's).

## Consequences

Discovery needs no infrastructure beyond the servers themselves; a card cannot be forged,
replayed after its expiry or attributed to another server; an operator who vouches for junk
loses their peers' trust, which bounds spam; players see why a server appears in their list.
Costs: one more keypair to protect in the data directory (a lost server key is a new identity
and every pin breaks); peer cards go stale between refreshes; a server's player count and
ladders are public to anyone who asks; mDNS is a crate and a UDP socket when enabled.

## Alternatives considered

A central master server: simplest, but a party everyone must trust and someone must run; the
first thing this proposal rejects. DNS SRV records per operator: decentralized and cheap, but
browsing needs a list anyway, and resolving SRV needs a resolver crate; kept as something
operators may do on top (an address is an address). A DHT from the start: real decentralized
resolution, but a heavy dependency and NAT trouble for a small ecosystem; deferred, and
compatible with the keys chosen here. Unsigned lists on a wiki: how communities start, and
still possible, but nothing stops a forged entry.
