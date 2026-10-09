# E16 — Design note (working detail; not binding; pruned by the tasks that implement it)

Companion to `docs/epics/E16-server.md` and the two ADR drafts in
`docs/lanes/server/adr-drafts/`. Everything here is a sketch for the draft's review.

## 1. A hosted battle, step by step

1. The operator starts `h3-server` with a configuration: bind address, packs directory, the
   scenario file (as `h3 run` takes it), profile, which seats are bots, the limits.
2. The server draws a 64-bit *game seed* and a separate *obstacle seed* from the OS, generates
   the obstacles from the obstacle seed and stores them in the setup as pieces, and creates the
   battle from the setup with the game seed. The obstacle seed is then forgotten. The journal
   is opened with the header, the seed and the setup (M2; in M1 the same data is kept in
   memory and written as the replay at the end).
3. A client opens a WebSocket and sends `Hello { protocol, client, name }`. A protocol mismatch
   gets `Goodbye { reason: "protocol" }`. Otherwise `Welcome { protocol, server, manifest }`.
4. The client sends `Join { game, seat: Attacker | Defender | Spectator }` and gets
   `Joined { viewer, token }` (no token for a spectator), then `Snapshot { n, observation }`
   with the projected observation for its viewer.
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
8. `GameEnded { n, result }` to every viewer. Participants may send `FetchReplay` and get the
   full replay; any viewer may send `FetchRecord` and get its own message log. The server
   finalizes the journal as the replay file (the CLI's TOML).

Sequence numbers: `n` counts the messages sent to *this* viewer; `seq` is the client's own
counter echoed in acknowledgements. Neither reveals anything about other viewers.

## 2. Message catalogue (M1; M2–M4 additions marked)

Client to server: `Hello`, `Join`, `Command`, `Query`, `FetchReplay`, `FetchRecord`, `Leave`,
`Ping`. M2: `CreateGame { config }`, `ListGames`, `StartGame`, `Pause`, `Resume`. M3:
`Login { public_key, signature, name }`, `Rekey { new_public_key, signatures }`. M4:
`Queue { ladder }`, `Dequeue`, `AcceptMatch`, `Leaderboard { ladder, page }`,
`MyRating { ladder }`.

Server to client: `Welcome`, `Goodbye`, `Joined`, `Snapshot`, `YourTurn`, `Accepted`,
`Rejected`, `QueryResult`, `Events`, `GameEnded`, `Replay`, `Record`, `Pong`. M2: `Games`,
`GameCreated`, `GameStarted`, `Paused`, `Resumed`. M3: `Challenge { nonce }`,
`LoggedIn { player }`. M4: `Queued`, `MatchFound { game, seat, deadline }`,
`LeaderboardPage`, `Rating`.

Every message is a struct variant of one enum per direction in serde's default, externally
tagged form (`{"Command":{"seq":7,"command":{...}}}`, `deny_unknown_fields`); no `tag`,
`untagged` or `flatten`, so the types also encode in `postcard` should a binary codec be
negotiated later (SRV-Q5; a round-trip test in both formats keeps that true). Unit variants are
avoided. Integers only; 64-bit values (the ruleset hash) travel as strings; bulk data as packed
arrays.

## 3. The manifest

```
Manifest {
  protocol: u32, engine: String,
  packs: [(id, version)], profile: String, ruleset_hash: String (hex),
  rng: (algorithm, version), hash: (algorithm, version), battle_state_hash_version: u32,
  keys: { creatures: [key by handle], spells, hero_types, terrains, artifacts, factions, skills }
}
```

The first block is the replay header's content (`h3_sim::ReplayHeader::current`). The key
tables come from the ruleset (`h3-content` has `creature_key` and the `*_by_key` lookups; M1
checks that every kind has the handle-to-key direction, else a small battle-lane ask).

A client's content check: compile the manifest's packs and profile locally if it has them,
compare the ruleset hash; equal means local previews are safe; different or absent means
server queries only and a visible note. The server never reads anything from this check.

## 4. The observation on the wire (protocol's own type)

Round, tactics phase, who may retreat or surrender, the active stack, the last side that
acted, the result; per stack: id, side, origin (army slot, war machine, created, clone),
creature handle, count, top creature's hit points, effective attack, defense, speed, damage
range, shots, retaliations left, flags (acted, waited, defending, two-hex, flying, shooter,
war machine), position, the timed effects (spell handle, rounds left), casts left; natural
obstacles and removable ones with anchors; the spell obstacles the viewer sees; both heroes'
public part; the viewer's own hero's private part (Tactics, damage bonuses, war-machine
control, surrender discount and price, funds, spell book, masteries, spell bonuses). This mirrors
what the sim's observation encoding exposes, in named fields.

## 5. Events on the wire

One wire variant per engine event kind, same fields, stack ids and hexes as integers, spells
and creatures as handles; `Other { kind: String }` for an engine kind the mapping lacks.
Visibility is applied before mapping by the engine's `project`.

## 6. The record

A JSON-lines file: the manifest, then every message the viewer received, in order, each with
its `n`. A viewer client replays a game by feeding the record through the same code path as a
live connection. Format version `RECORD_FORMAT_VERSION` in `h3-protocol`.

## 7. Journals: restore, pause, resume, upgrades (M2)

- **Content.** A journal file per game: the replay header (packs, profile, ruleset hash,
  engine and algorithm versions), the game seed, the setup, the seats (account ids or token
  hashes), then one line per accepted command with its side, and every N commands a line with
  the live state's hash. Finalized, it is the replay (`h3_sim::Replay`); the CLI's verifier
  reads it.
- **Write-ahead.** The command line is appended and synced before `Accepted` is sent. A crash
  between apply and sync loses at most that command: on restore the client sees the state
  before it and may resend (its `seq` tells it which).
- **Restore at start.** For every open journal the server checks the header against its own
  engine and content (`ReplayHeader::check`), re-runs the commands through a fresh engine
  instance with a no-op sink, compares the recorded hashes on the way, and reopens the game
  with its seats empty; participants rejoin by account or token and get a `Snapshot`. A header
  that does not match, or a hash that does not, closes the game as void: its replay stays
  available, no rating changes.
- **Pause and resume.** A paused game is a journal without a running engine; `Resume` by its
  participants restores it as above. Unranked: the host or all seats may pause; ranked: both
  players agree, within the ladder's limit.
- **Upgrades.** A drain mode refuses new games and lets open ones finish; the operator then
  upgrades. Games left open across an upgrade whose engine or content identity changed are
  void. When state saves exist (ADR-0004 point 9, an adventure-lane topic for long games), a
  snapshot every M commands can shorten restores; the journal stays the source of truth.

## 8. Integrity: how a corrupted game is caught

- **Checkpoints.** The live state's canonical hash (ADR-0003 point 11) goes into the journal
  every N commands (the replay's `checkpoint_every`) and at the end.
- **Verification before use.** When a game ends, and before any rating or leaderboard change,
  the server re-runs the journal in a fresh engine instance and compares every checkpoint and
  the final hash, as `Replay::verify` does. A single flipped bit in the live state after any
  checkpoint changes the hashes that follow it; a flipped bit in a command before it was
  journaled makes the re-run diverge from the live state at the next checkpoint; a flip that
  turns a command illegal makes the re-run reject it. Any mismatch marks the game *corrupt*:
  void for ladders, the replay kept with the diagnosis (step, expected, actual), a log line.
- **The ruleset.** The live game and the re-run share the compiled ruleset in memory, so a
  corruption there would affect both alike; at game end the server recomputes the ruleset's
  content hash from memory and compares it with the manifest's. Hardware faults remain the
  operator's to prevent (ECC memory); the checks above make them visible, not impossible.
- **Shadow instance (option).** A second engine instance applies every command in step with
  the live one; the hashes are compared after each command. A mismatch stops the game at once;
  the server restores the correct state from the journal (the journal, not either instance,
  is the truth), tells the players, and continues. Costs one more engine step per command
  (microseconds in battle). Off by default; a ladder may require it.
- **What the client sees.** `Accepted` echoes the applied command, and the events describe
  what happened, so a player whose intent was altered before the journal would notice; the
  server cannot catch that case itself.

## 9. Server configuration and data directory (sketch)

```
[server]      bind = "127.0.0.1:7777"   data_dir = "server-data"   anonymous = true
              shadow_instance = false   checkpoint_every = 25
[limits]      frame_bytes = 65536   messages_per_second = 20   connections = 256
              connections_per_address = 8   idle_seconds = 120   pause_hours = 72
[content]     packs_dir = "packs"   packs = ["base", "ab", "sod"]   profile = "faithful"
[[game]]      scenario = "fixtures/scenarios/skirmish.toml"   bots = ["defender"]   # M1 only
[[ladder]]    id = "battle-arena"  scenario_pool = ["…"]  season = 1  pause = false  # M4
```

`server-data/games/<game>.journal` (open) and `server-data/replays/<game>.toml` (finished),
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
