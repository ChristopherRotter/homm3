# E16 — Design note (working detail; not binding; pruned by the tasks that implement it)

Companion to `docs/epics/E16-server.md`, ADR-0018 (protocol and hosting; this note holds the
detail its trim left out) and the ADR drafts in `docs/lanes/server/adr-drafts/`. A sketch: each
M1–M6 task turns its section into code and docs and deletes it here.

## 1. A hosted battle, step by step

1. The operator starts `h3-server` with a configuration: bind address, packs directory, the
   scenario file (as `h3 run` takes it), profile, which seats are bots, the limits.
2. The server draws a 64-bit *game seed* and a separate *obstacle seed* from the OS, generates
   the obstacles from the obstacle seed and stores them in the setup as pieces, and creates the
   battle from the setup with the game seed. The obstacle seed is then forgotten. The journal
   is opened with the header, the seed and the setup (M2; in M1 the same data is kept in
   memory and written as the replay at the end).
3. A client opens a WebSocket and sends `Hello { protocol, client, name }`. A protocol mismatch
   gets `Goodbye { reason: Protocol }`. Otherwise `Welcome { protocol, server }`.
4. The client sends `Join { game, seat: Attacker | Defender | Spectator, token }` and gets
   `Joined { game, seat, token, manifest }` (no token for a spectator) or `Refused { reason }`,
   then `Snapshot { n, observation }` with the projected observation for its viewer.
5. When both seats are taken (a bot counts), the game starts. The seat to act gets
   `YourTurn { n, legal, castable }` (the legal commands and the castable spells of its hero);
   it may ask `Query { seq, what: CastTargets { spell } | SecondTargets { spell, target } }`.
6. The seat sends `Command { seq, command }`. The session core calls the engine's `apply` with
   the seat's side. Either `Rejected { seq, code }` (nothing changed) or, after the command is
   in the journal, `Accepted { seq, command }` (echoed, so the client can see what was applied)
   followed by `Events { n, events }` to every viewer, each list projected for its viewer, then
   the next `YourTurn`. A bot seat's turn is answered by the host's bot.
7. A client that drops reconnects with `Join { game, seat, token }` and receives a fresh
   `Snapshot`; it discards its own state. Spectators reconnect without a token.
8. `GameEnded { n, result }` to every viewer. Participants may send `FetchReplay { seq }` and
   get the full replay (`Rejected { NotOver }` before the end); any viewer may send
   `FetchRecord { seq }` and get its own message log, the *record*. The server finalizes the
   journal as the replay file (the CLI's TOML).

Sequence numbers: `n` counts the messages sent to *this* viewer; `seq` is the client's own
counter echoed in acknowledgements. Neither reveals anything about other viewers.

All eight steps are code since M1 T3: steps 3 to 8 in `h3-protocol` (`connection.rs`,
`session.rs`; the record's format there and in `wire.rs`), the host's parts (sockets, the OS
seeds and tokens, the replay file) in `h3-server` (`game.rs`, `net.rs`); the spectators of steps
4 and 7 since M2 T4 (the battle's spectator viewer; `E16-log.md`).

## 2. Message catalogue (M2–M4 additions)

The M1 messages, the manifest, the observation and the events are code since M1 T1
(`crates/h3-protocol/src/wire.rs`, in the same form: struct variants in serde's default
enum form). Since M2 T4b the observation carries every field of the engine's observation, or
the mapping names it as left out and why (`map/mod.rs`: a pattern per engine type, so a new
field fails the build until it is mapped). Later milestones add:

Client to server: M2: `ListGames { seq }` and `CreateGame { seq, offer, seat, spectators }`
(code since M2 T1: a game is created from one of the server's offers and its creator takes
`seat` in it at once; `spectators` since T4c, open when left out), `Pause { seq }` (code since
M2 T3; no `Resume`: a seat's `Join` with its token resumes), `SetSpectators { seq, open }` (code
since M2 T4c: the host's, until the game starts).
M3: `Login { public_key, signature, name }`, `Rekey { new_public_key, signatures }`. M4:
`Queue { ladder }`, `Dequeue`, `AcceptMatch`, `Leaderboard { ladder, page }`,
`MyRating { ladder }`.

Server to client: M2: `Games { seq, offers, games }` and `GameCreated { seq, game }` (code since
M2 T1; the refusals `NoSuchOffer` and `TooManyGames`), `PauseAsked { n, seat }` and
`Goodbye { Paused }` (code since M2 T3, instead of `Paused` and `Resumed`), `SpectatorsSet { n,
seq, open }`, `Goodbye { SpectatorsClosed }`, the reject codes `NotHost` and `AlreadyStarted`
and `GameInfo::spectators` (code since M2 T4c). No `StartGame`: a
game starts when every seat is held at once (M1 T2's rule). M3:
`Challenge { nonce }`, `LoggedIn { player }`. M4: `Queued`, `MatchFound { game, seat,
deadline }`, `LeaderboardPage`, `Rating`.

## 3. The client's content check (for the client lane, M2)

Compile the manifest's packs and profile locally if the client has them and compare the
ruleset hash: equal means local previews are safe; different or absent means server queries
only and a visible note. The server never reads anything from this check.

## 7. Journals: restore, pause, resume, upgrades (M2)

- **Content, write-ahead, restore at start** are code since M2 T2 (`h3-server/src/journal.rs`,
  `registry::restore`; `Session::restore` in `h3-protocol`; the record in `E16-log.md`): a journal
  per game in `<data>/games/`, its entries synced before the frames that depend on them, re-run
  at start, a journal that does not match or re-run moved to `<data>/void/`.
- **Pause and resume** are code since M2 T3 (the record in `E16-log.md`): unranked, the host's
  `Pause` pauses, a paused game is its journal, a seat's join with its token restores it. Since
  M2 T5a a running game whose seats all stay empty for `[limits] empty_game_ms` pauses, and a
  paused game nobody resumes within `paused_game_ms` is void. Still to come: ranked, both players
  agree, within the ladder's limit (M4).
- **Upgrades.** A drain mode refuses new games and lets open ones finish; the operator then
  upgrades. Code since M2 T5a: the first SIGINT or SIGTERM drains (`Refused { Draining }` for a
  creation, a waiting game ends, running and paused ones go on), the server stops once none
  waits or runs, a second signal stops it at once; stopped, the connections hear
  `Goodbye { Shutdown }` and the journals stay for the next start. Games left open across an upgrade whose engine or content identity changed are
  void. When state saves exist (ADR-0004 point 9, an adventure-lane topic for long games), a
  snapshot every M commands can shorten restores; the journal stays the source of truth.

## 8. Integrity: how a corrupted game is caught

Code since M2 T3 (`h3-server/src/game.rs`, `Session`'s shadow instance; the record in
`E16-log.md`): checkpoints in the journal; every finished game's journal read back and re-run
before its replay is published, a mismatch voiding it (the replay withheld, the journal and the
diagnosis kept); the ruleset hash recomputed from memory at the end; the shadow-instance option
(off by default), whose mismatch rebuilds both battles from the journal and goes on. Still to come:
results count only after the verification (M4), and a ladder may require the shadow instance.
Hardware faults remain the operator's to prevent (ECC memory); the checks make them visible, not
impossible. A player whose intent was altered before the journal would notice from `Accepted`'s
echo and the events; the server cannot catch that case itself.

## 9. Server configuration and data directory (sketch)

The configuration is code since M2 T1 (`crates/h3-server/src/config.rs`, the example
`server.example.toml`): `[server]` (`bind`, `data`, `checkpoint_every`), `[content]` (`packs`, the
packs directory: a scenario names its packs and profile), `[limits]` (M1's six and `games`,
`games_per_address`), one `[[offer]]` per game configuration (`id`, `scenario`, `profile`,
`bots`); the replay goes to `<data>/replays/<offer>-game<id>-<random>.toml`; `shadow_instance`
in `[server]` since M2 T3, `spectators_per_game` in `[limits]` since M2 T4, `empty_game_ms` and
`paused_game_ms` since M2 T5a. Still to come:

```
[server]      anonymous = true                                       # M3
[[ladder]]    id = "battle-arena"  scenario_pool = ["…"]  season = 1  pause = false  # M4
```

The journals (`<data>/games/<game>.journal`, open games), the replays (`<data>/replays/`,
finished) and `<data>/void/` are code since M2 T2. Still to come:
`server-data/records/<game>/<viewer>.jsonl`, `server-data/players.jsonl`,
`server-data/ladders/<id>/results.jsonl` (M3, M4), snapshots beside the logs. The log never
contains seeds, tokens, nonces or signatures.

## 10. Accounts (M3)

- **Derivation (client side).** `seed = argon2id(password, salt = "h3:" + name_lowercase,
  memory 64 MiB, 3 passes)`, `keypair = ed25519_from_seed(seed)`. The same name and password
  give the same key on every device and every server. The client caches the key locally so the
  derivation runs once per device; the password is never stored.
- **Login.** `Hello` then `Challenge { nonce }` (32 random bytes, valid for a minute, one at a
  time per connection) then `Login { public_key, signature over (server id, nonce), name }`
  then `LoggedIn { player }`. A new public key with a free name registers; a known key logs
  in; a free name with a known key renames; a taken name with another key is refused.
- **Re-keying.** After a password change the client keeps the old key until each server it
  knows has accepted `Rekey { new_public_key, signature_by_old, signature_by_new }` at the
  next login; the server rebinds the account. A forgotten password is a lost account; the
  operator's admin command can rebind a name to a new key by hand.
- **Privacy.** A public key is shown to its owner only (a fingerprint in the client); other
  clients see names. Anonymous connections skip `Login` when the configuration allows and may
  never queue for a ladder.

## 11. Ladders and matchmaking (M4)

- Servers are independent: a ladder and its accounts belong to one server; nothing is
  synchronized between servers; there is no global ladder.
- A ladder: id, game configuration (packs, profile, options), scenario pool (scenario files or
  a generator with seeds), rating system constants, season, pause policy. The ruleset hash of
  the configuration is part of the ladder's identity; a change starts a new season.
- The queue: one entry per player per ladder. Every few seconds the matcher sorts the queue by
  rating and pairs neighbours whose distance is within `window(waited)` (for example 100 points
  plus 50 per 30 seconds waited); a pair gets `MatchFound` with a deadline; both accept, or the
  decliner leaves the queue and the other stays at the front.
- The game: a scenario from the pool, sides assigned at random, both seats bound to the
  players' accounts; a disconnect gives a grace period (configured), after which the absent
  player forfeits.
- Rating: Glicko-2 with the published defaults (rating 1500, deviation 350, volatility 0.06,
  system constant 0.5), one rating period per game end, applied in the order games end; a draw
  is a half point. Only a game whose journal verified (§8) and whose seats were held by their
  accounts counts. Bot accounts are flagged; a server may exclude them from the leaderboard or
  list them apart.
- The leaderboard: rating, deviation, games, wins, losses, draws; pages of 50; a player's own
  row and rating history on request.

## 12. Server identity and discovery (M6)

- **Server key.** Generated at first start into `server-data/server.key`; the public key is
  the server's identity. `Hello` carries a client nonce; `Welcome` carries the server's key and
  `sign(server_key, nonce || key || address)`. The client pins the key per address on first
  contact and refuses a changed key until the player confirms.
- **Card.** `ServerCard { key, name, description, addresses, protocol, engine, content:
  [(packs, profile, ruleset_hash)], ladders: [(id, name, season, players)], players_online,
  games_open, anonymous, operator, issued, expires, peers: [(key, address)] }` with a
  signature over the canonical JSON; expiry a day by default; `Describe` returns it.
- **Peers.** `[[peer]] key = "…" address = "…"` in the configuration. Every hour (and at
  start) the server connects to each peer as a client (`tokio-tungstenite`'s client side), sends
  `Hello` and `Describe`, checks that the signature verifies and that the key is the configured
  one, caches the card until it expires, and also takes the peer's own cached cards up to a
  configured depth (default 2 hops) and count (default 200). `ListPeers` returns the cache,
  each card with the chain of keys it came through. A card that fails verification is dropped
  and logged; a peer that keeps serving bad cards is reported to the operator.
- **Lists.** `ServerList { key, name, issued, expires, cards }` signed by the list key; files
  the operator places in `server-data/lists/`, served in `ListPeers` marked with the list's
  key. How a list reaches the server (download, cron, a git checkout) is the operator's. A
  client may fetch lists itself where it has HTTPS (web clients do).
- **Client side** (the client lane): bookmarks with pinned keys; a browser view that merges
  the cards from every known server and list, groups them by content the client has, shows the
  vouching chain and the age of each card, and connects only on the player's choice.
- **LAN.** With the feature enabled, the server announces `_h3._tcp.local` with TXT records
  `key=<fingerprint> name=<name> protocol=<n>`; a client browses the same (the `mdns-sd`
  crate, MIT/Apache-2.0; a UDP multicast socket).
- **Deferred: resolution by key.** If addresses change often or vouching does not scale,
  publish `(key → addresses)` as a signed mutable record in the BitTorrent mainline DHT
  (BEP 44, keyed by the server's Ed25519 key, as Pkarr does); clients resolve a pinned key
  without DNS. Needs a DHT crate, a UDP socket reachable through NAT, and bootstrap nodes.

## 13. Threats the server answers, and how

| Threat | Answer |
|---|---|
| A client acts for another seat | impossible to express: commands carry no seat; the seat is the connection's |
| Out-of-turn or invalid command | the engine rejects it; state and journal unchanged (atomicity, property-tested) |
| Learning hidden state | projection per viewer; per-viewer sequence numbers; no hashes, seeds or RNG state on the wire; typed error codes only |
| Guessing the seed from the obstacles | the seed is 64 random bits; obstacles come from a separate seed |
| Malformed or oversized input | bounded decoder, panic-free (logic crate lints), property-tested, fuzzed |
| Flooding | per-connection rate limits, one command in flight per seat, connection limits |
| Taking over a seat | the seat token (128 random bits); with accounts (M3) the seat is bound to the account |
| Forged ladder results | results come from the server's own games, after the journal verified |
| A corrupted game (memory fault, a bug) | checkpoint hashes in the journal, verification before any result counts, the shadow-instance option |
| A crash or a pause | the write-ahead journal and a deterministic re-run restore the exact state |
| A leaked data directory | holds public keys, names, ratings, records and finished replays; no password, no secret |
| A stolen password | the same as anywhere: the operator can rebind the name by hand; no server-side reset exists |
| A forged or replayed server card | signed by the server's key, with issue and expiry times; relayers cannot alter it |
| Connecting to an impostor | the key pinned on first contact and the signed welcome; TLS for the channel on the internet |
| Discovery spam | only vouched cards are relayed, with hop and count bounds; the chain is shown; the player chooses |
| Browsing reveals the player's address | cards are relayed by servers the player already knows; a new server is contacted only on the player's choice |
