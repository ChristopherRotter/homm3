# E16 — Design note (working detail; not binding; pruned by the tasks that implement it)

Companion to `docs/epics/E16-server.md` and the two ADR drafts in
`docs/lanes/server/adr-drafts/`. Everything here is a sketch for the first draft's review.

## 1. A hosted battle, step by step

1. The operator starts `h3-server` with a configuration: bind address, packs directory, the
   scenario file (as `h3 run` takes it), profile, which seats are bots, the limits.
2. The server draws a 64-bit *game seed* and a separate *obstacle seed* from the OS, generates
   the obstacles from the obstacle seed and stores them in the setup as pieces, and creates the
   battle from the setup with the game seed. The obstacle seed is then forgotten.
3. A client opens a WebSocket and sends `Hello { protocol, client, name }`. A protocol mismatch
   gets `Goodbye { reason: "protocol" }`. Otherwise `Welcome { protocol, server, manifest }`.
4. The client sends `Join { game, seat: Attacker | Defender | Spectator }` and gets
   `Joined { viewer, token }` (no token for a spectator), then `Snapshot { n, observation }`
   with the projected observation for its viewer.
5. When both seats are taken (a bot counts), the game starts. The seat to act gets
   `YourTurn { n, legal, castable }` (the legal commands and the castable spells of its hero);
   it may ask `Query { seq, what: CastTargets { spell } | SecondTargets { spell, target } }`.
6. The seat sends `Command { seq, command }`. The session core calls the engine's `apply` with
   the seat's side. Either `Rejected { seq, code }` (nothing changed) or `Accepted { seq }`
   followed by `Events { n, events }` to every viewer, each list projected for its viewer, then
   the next `YourTurn`. A bot seat's turn is answered by the host's bot.
7. A client that drops reconnects with `Join { game, seat, token }` and receives a fresh
   `Snapshot`; it discards its own state. Spectators reconnect without a token.
8. `GameEnded { n, result }` to every viewer. Participants may send `FetchReplay` and get the
   full replay; any viewer may send `FetchRecord` and get its own message log. The server also
   writes the replay to its data directory as the CLI's TOML.

Sequence numbers: `n` counts the messages sent to *this* viewer; `seq` is the client's own
counter echoed in acknowledgements. Neither reveals anything about other viewers.

## 2. Message catalogue (M1; M2–M4 additions marked)

Client to server: `Hello`, `Join`, `Command`, `Query`, `FetchReplay`, `FetchRecord`, `Leave`,
`Ping`. M2: `CreateGame { config }`, `ListGames`, `StartGame`. M3: `Login { public_key,
signature, name }`. M4: `Queue { ladder }`, `Dequeue`, `AcceptMatch`, `Leaderboard { ladder,
page }`, `MyRating { ladder }`.

Server to client: `Welcome`, `Goodbye`, `Joined`, `Snapshot`, `YourTurn`, `Accepted`,
`Rejected`, `QueryResult`, `Events`, `GameEnded`, `Replay`, `Record`, `Pong`. M2: `Games`,
`GameCreated`, `GameStarted`. M3: `Challenge { nonce }`, `LoggedIn { player }`. M4: `Queued`,
`MatchFound { game, seat, deadline }`, `LeaderboardPage`, `Rating`.

Every message is a struct variant of one internally tagged enum per direction
(`#[serde(tag = "type")]`, `deny_unknown_fields`); unit variants are avoided (the serde trap in
`docs/HANDOFF.md`).

## 3. The manifest

```
Manifest {
  protocol: u32, engine: String,
  packs: [(id, version)], profile: String, ruleset_hash: u64,
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

## 7. Server configuration and data directory (sketch)

```
[server]      bind = "127.0.0.1:7777"   data_dir = "server-data"   anonymous = true
[limits]      frame_bytes = 65536   messages_per_second = 20   connections = 256
              connections_per_address = 8   idle_seconds = 120
[content]     packs_dir = "packs"   packs = ["base", "ab", "sod"]   profile = "faithful"
[[game]]      scenario = "fixtures/scenarios/skirmish.toml"   bots = ["defender"]   # M1 only
[[ladder]]    id = "battle-arena"  scenario_pool = ["…"]  season = 1  # M4
```

`server-data/replays/<game>.toml`, `server-data/records/<game>/<viewer>.jsonl`,
`server-data/players.jsonl`, `server-data/ladders/<id>/results.jsonl` (M3, M4). The log never
contains seeds, tokens, nonces or signatures.

## 8. Identity (M3)

Client side: a key file generated once (`~/.config/h3/identity`), the public key shown as a
short fingerprint. Login: `Hello` then `Challenge { nonce }` (32 random bytes, valid for a
minute, one at a time per connection) then `Login { public_key, signature over (server id,
nonce), name }` then `LoggedIn { player }`. The server stores `(public key, name, first seen)`;
a name is unique per server; a known key may change its name. Anonymous connections skip
`Login` when the configuration allows and may never queue for a ladder.

## 9. Ladders and matchmaking (M4)

- A ladder: id, game configuration (packs, profile, options), scenario pool (scenario files or
  a generator with seeds), rating system constants, season. The ruleset hash of the
  configuration is part of the ladder's identity; a change starts a new season.
- The queue: one entry per player per ladder. Every few seconds the matcher sorts the queue by
  rating and pairs neighbours whose distance is within `window(waited)` (for example 100 points
  plus 50 per 30 seconds waited); a pair gets `MatchFound` with a deadline; both accept, or the
  decliner leaves the queue and the other stays at the front.
- The game: a scenario from the pool, sides assigned at random, both seats bound to the
  players' identities; a disconnect gives a grace period (configured), after which the absent
  player forfeits.
- Rating: Glicko-2 with the published defaults (rating 1500, deviation 350, volatility 0.06,
  system constant 0.5), one rating period per game end, applied in the order games end; a draw
  is a half point. Only a game whose replay verifies on the server and whose seats were held by
  their players counts. Bot identities are flagged; a server may exclude them from the
  leaderboard or list them apart.
- The leaderboard: rating, deviation, games, wins, losses, draws; pages of 50; a player's own
  row and rating history on request.

## 10. Threats the server answers, and how

| Threat | Answer |
|---|---|
| A client acts for another seat | impossible to express: commands carry no seat; the seat is the connection's |
| Out-of-turn or invalid command | the engine rejects it; state and log unchanged (atomicity, property-tested) |
| Learning hidden state | projection per viewer; per-viewer sequence numbers; no hashes, seeds or RNG state on the wire; typed error codes only |
| Guessing the seed from the obstacles | the seed is 64 random bits; obstacles come from a separate seed |
| Malformed or oversized input | bounded decoder, panic-free (logic crate lints), property-tested, fuzzed |
| Flooding | per-connection rate limits, one command in flight per seat, connection limits |
| Taking over a seat | the seat token (128 random bits); with identity (M3) the seat is bound to the player |
| Forged ladder results | results are computed from the server's own games and verified replays only |
| A leaked data directory | holds public keys, names, ratings, records and post-game replays; no secrets |
