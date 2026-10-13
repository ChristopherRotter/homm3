# ADR-0021: Server identity and decentralized discovery

- Status: Accepted (user, 2026-10-13, the E16 M6 refinement, SRV-Q23; direction from SRV-Q11)
- Date: 2026-10-13
- Deciders: user; server lane sessions (E16 refinement, M6 refinement)
- Working detail (keys, the welcome, cards, peers, lists, bounds): [`docs/epics/E16-design.md`](../epics/E16-design.md) §12; options: [`docs/epics/E16-server.md`](../epics/E16-server.md), SRV-Q23

## Context

Players need to find servers without a central registry, and need to know that the server they
reach is the one that was described to them (the user, 2026-10-08: a trusted, decentralized
mechanism). Servers are independent (ADR-0019 point 3); operators are hobbyists on home
connections with changing addresses; the ecosystem is small. "Trusted" has two parts: a
description cannot be forged or replayed, and a client reaches the server the description
names. No decentralized mechanism can say whether a server is *good*; vouching and the
player's own choice are the honest answer to that.

## Decision

1. **Every server has a keypair** (Ed25519, generated at first start, kept in the data
   directory; a start never replaces a key file it cannot read). Its public key is the server's
   identity across address changes. The hello carries a client nonce and the address the client
   dialed; the welcome carries the server's key and a signature over the nonce, the server's id
   (one of its own addresses, ADR-0019 point 1) and the connection's challenge, so a client knows
   the key's holder answered at the address it dialed.
   Clients pin a server's key on first contact and warn loudly when it changes (as SSH does).
   Each signed message starts with its purpose and version, so no signature is good for another
   purpose. Channel integrity on the internet still needs TLS (ADR-0019 point 10): the signature
   proves who answered, not that nobody sits between.
2. **A server describes itself with a signed card:** name, description, addresses, key,
   protocol and engine versions, content (packs, profile, ruleset hashes), ladders with their
   seasons and player counts, players online, whether anonymous play is allowed, an operator
   contact, issue and expiry times, and its *peers* (point 3). The signature covers the card's
   text exactly as it travels, so relayers pass it on verbatim. Any client gets it with
   `Describe`; the card is also a file the operator may put on any web page.
3. **Discovery is a web of trust between servers.** An operator lists in the configuration
   the servers they vouch for, by key and address. A server fetches its peers' cards over the
   same protocol, verifies the signatures and the keys, caches them until they expire, and
   serves them with `ListPeers`, together with what those peers serve, bounded in hops and count;
   each card comes with the chain of keys that vouches for it, which a client checks against the
   cards themselves. A server connects only to the addresses its operator configured (peers, and
   the servers of the lists it was given), never to an address it learnt from a card. A client
   browses from the servers it already knows (its own, bookmarks, a few in the client's
   configuration), receives cards relayed by them, and shows each server with its chain. The
   client contacts a server only when the player picks it, so browsing reveals the player's
   address to no unknown server.
4. **The fetch has a TLS client** (the user, SRV-Q23 (1)): internet servers sit behind a TLS
   proxy, so a server dials its peers at `wss://` (`ws://` on a LAN) with `rustls` and the
   pure-Rust `graviola` crypto backend (`rustls-graviola`), checking certificates against the
   operating system's roots (`rustls-native-certs`), through `tokio-tungstenite`'s rustls
   feature; `ring` and `aws-lc` are not GPLv2-compatible and stay out (`deny.toml`). Graviola
   runs on x86_64 and aarch64 with their crypto extensions only; the backend is one line to swap.
   Serving stays without TLS, behind the reverse proxy (ADR-0018 point 2, ADR-0019 point 10).
5. **A list names servers by key and address, signed by the list's key** (the user, SRV-Q23
   (3)): anyone may publish one (a community list, a tournament's list), valid until its own
   expiry. A server serves the lists its operator placed in its data directory and fetches the
   listed servers' cards as it fetches its peers', each marked with the list's key. A client may
   fetch lists itself where it has an HTTP client (the client lane's choice). Clients choose
   which list keys to trust, as they choose peers.
6. **Not built:** mDNS on the LAN (later, with the client lane's server browser: DNS-SD,
   `_h3._tcp.local`, the key's fingerprint in the record; the user, SRV-Q23 (2)); a central
   registry (a party to trust and to run); a DHT (a resolver of addresses by key would be the
   next step if the ecosystem outgrows vouching: signed mutable records in the BitTorrent
   mainline DHT keyed by the server's public key, as Pkarr does; a DHT crate, UDP and NAT
   traversal); server reputation (vouching chains are what a client shows; the judgement is the
   player's).

## Consequences

Discovery needs no infrastructure beyond the servers themselves; a card cannot be forged,
replayed after its expiry or attributed to another server; an operator who vouches for junk
loses their peers' trust, which bounds spam; players see why a server appears in their list.
Costs: one secret in the data directory (a leaked server key lets its holder speak as the
server; a lost one is a new identity and every pin breaks); peer cards go stale between
refreshes; a server's player count and ladders are public to anyone who asks; 13 more crates in
`h3-server` for the TLS client, its backend young and limited to two architectures; outbound
connections from every server each refresh, bounded by its configuration.

## Alternatives considered

A central master server: simplest, but a party everyone must trust and someone must run; the
first thing this proposal rejects. DNS SRV records per operator: decentralized and cheap, but
browsing needs a list anyway, and resolving SRV needs a resolver crate; kept as something
operators may do on top (an address is an address). A DHT from the start: real decentralized
resolution, but a heavy dependency and NAT trouble for a small ecosystem; deferred, and
compatible with the keys chosen here. Unsigned lists on a wiki: how communities start, and
still possible, but nothing stops a forged entry. A plain `ws://` peering address instead of a
TLS client (cards are signed, so a fetch needs no TLS for integrity): no dependency, but every
operator would expose a second, unencrypted endpoint (SRV-Q23 (1)). `ring` or `aws-lc-rs` as the
TLS backend: mainstream, but `Apache-2.0` binds them, which GPL-2.0 does not accept (ADR-0009).
Lists of signed cards (the first draft): served without any fetch, but cards expire in a day, so
a list's maker would collect and re-sign them daily (SRV-Q23 (3)).
