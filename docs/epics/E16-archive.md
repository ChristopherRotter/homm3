# E16 — archive: closed milestones M1 to M4 and M6

Moved out of `E16-server.md` on 2026-10-24, verbatim, so that the epic file holds the goals, the decisions, the milestone outline, the autonomy contract and the risks. Order as in the epic: task lists, exit criteria, review lists.

## M1 tasks

- **T1 Messages and codec** (done 2026-10-10; `E16-log.md`): `wire`, `codec` and `map` in
  `h3-protocol`, 100 % covered with no exclusion; ADR-0018 point 5 amended.
- **T2 Session core** (done 2026-10-10; `E16-log.md`): `connection` (the hello gate) and
  `session` (seats, tokens, the loop, the bot hook, records, the journal) in `h3-protocol`, the
  hygiene and rejection properties over the wire; `PROTOCOL_VERSION` 2.
- **T3 The server binary** (done 2026-10-10; `E16-log.md`): `h3-server` hosts the configured
  battle (TOML configuration, OS seeds and tokens, a thread for the game, a `tokio` task per
  connection behind the transport trait, the random bot, the replay file); end-to-end tests over
  loopback with the headless client; the replays it writes verify with `h3 verify`.
- **T4 Closure** (done 2026-10-10; `E16-log.md`): the lane-only workload
  `session-random-play-full` (scripted clients through the session core, 273 M instructions);
  `docs/06-security.md`'s "Hosted games"; the reviewer agent; the proposed cut of M2
  (`docs/lanes/server/QUEUE.md`), decided at the checkpoint.

## M2 tasks

Cut at the M1 checkpoint (SRV-Q12); the tasks in full: `docs/lanes/server/QUEUE.md`.

- **T1 Many games on one server** (done 2026-10-11; `E16-log.md`): the lobby (`ListGames`,
  `CreateGame` from the server's offers; wire version 4), a sans-I/O `lobby` in `h3-protocol`,
  the sectioned configuration with `[[offer]]`s, a registry and a thread per game in `h3-server`.
- **T2 The write-ahead journal and restore** (done 2026-10-11; `E16-log.md`): the session
  core's journal entries and `Session::restore`; a journal per game in `<data>/games/`, synced
  before the frames that depend on it; restore at start, void journals moved aside; the replay
  as the finished journal.
- **T3 Verification, pause and resume** (done 2026-10-11; `E16-log.md`): a finished game's journal
  read back and re-run before its replay is published (void otherwise, `Unverified`), the ruleset
  hash recomputed from memory; the shadow-instance option; `Pause` by the host, a paused game as
  its journal, resumed by a seat's join with its token (wire version 5, journal format 2).
- **T4 Spectators** (done 2026-10-11; `E16-log.md`): `Join` as a spectator, up to
  `[limits] spectators_per_game`, with the battle's spectator viewer (`observe_as`, `project_as`):
  its snapshot, the public events, its own numbers and record; the observation's `spectator` flag
  (wire version 6).
- **T4b The observation's completeness on the wire** (done 2026-10-11; `E16-log.md`): the
  fields E04b T9a added (the wait phase, the per-stack clone links, raised creatures, corpses,
  binders, poison and start hit points, the summoned creature and the cost change per side, the
  cleared hexes, the own hero's hidden magic), the effects' values; every field of the engine's
  observation and its parts named in a pattern, so a new one fails the build until it is mapped or
  left out with its reason (wire version 7).
- **T4c The host's spectator setting** (done 2026-10-11; `E16-log.md`; the user asked for it):
  `CreateGame`'s `spectators` (open when left out), the host's `SetSpectators` until the start
  (closing ends the watch), `GameInfo::spectators`, the setting journaled (wire version 8,
  journal format 3).
- **T5a Shutdown, drain, the deadlines of abandoned games** (done 2026-10-11; `E16-log.md`): the
  first signal drains (no new game, a waiting one ends, the server stops once none waits or
  runs), a second stops at once, the journals kept; a running game whose seats all stay empty
  pauses, a paused one nobody resumes is void (`[limits] empty_game_ms`, `paused_game_ms`);
  `tokio`'s `signal` (ADR-0018 point 3 amended); wire version 9 (`Refused { Draining }`).
- **T5b The connection's bounds** (done 2026-10-11; `E16-log.md`): a rate per address across its
  connections (`[limits] messages_per_address`), WebSocket pings counted, a deadline for the hello
  and one for being in a game (`hello_timeout_ms`, `lobby_timeout_ms`), a refused join leaves the
  game (`Session::holds`); no wire change.
- **T5c Memory bounded in bytes** (done 2026-10-11; `E16-log.md`): a viewer's record keeps its
  latest messages within `[limits] record_bytes` (the header's `first`), the outbound queue counts
  its bytes (`outbound_bytes`); wire version 10, record format 2.
- **T5d The decoder's mutation tests, the public archive of finished games** (done 2026-10-11;
  `E16-log.md`; SRV-Q14, SRV-Q15): mutated valid messages never make the decoders or a running
  session panic; any client lists finished games' replays and downloads one (`ListReplays`,
  `FetchArchived`), a game closed to spectators archived unlisted, `[limits] replays_kept`; wire
  version 11; ADR-0018 points 8 and 9 amended.
- **T5e M2's closure** (done 2026-10-11; `E16-log.md`): the exit criteria below; the client
  lane's note (the content check, a viewer after a restart); the inbox placed; the reviewer agent
  (1 major, 8 minor): no file I/O under the registry's lock, a new or resumed game's own join
  first, a refused join keeps the lobby's deadline, a paused game's deadline across restarts, no
  journal text in the log, the replay written whole, a panicking game thread leaves the lobby; the
  rest in the inbox.

## M3 tasks

Cut at the M3 refinement (2026-10-11, SRV-Q17); the tasks in full: `docs/lanes/server/QUEUE.md`.
ADR-0019 (accepted at the refinement; points 1 and 2): a permanent
login name that salts the key and a changeable display name, the server's id in the signed
login, `ed25519-dalek` and `argon2`, anonymous play by configuration, any seated account opting
out of its replay's listing.

- **T1 The login on the wire, sans-I/O** (done 2026-10-11; `E16-log.md`): `account` (the names'
  rules, hex, the signed bytes), `Login`, `Rekey`, `Rename`, `SetListed` answered `Account`, the
  welcome's id and challenge, the hello gate's login step; wire version 12; the server refuses
  every login until T2a (SRV-STUB-1).
- **T2a The Authenticator and the player log** (done 2026-10-11; `E16-log.md`): signatures
  checked (Ed25519, strict), registration and login, `<data>/players.log`, the derivation and its
  test vector in the test client, `[server] public_url` and `anonymous`, failed logins limited
  per address; re-keys, renames and the preference refused until T2b (SRV-STUB-2).
- **T2b Re-keying, renames, the listing preference, the operator's rebind** (done 2026-10-11;
  `E16-log.md`): the three requests answered, each a line in the player log; `h3-server rebind`
  while the server is stopped, `<data>/lock` keeping one server or command at a time.
- **T3 Seats bound to accounts** (done 2026-10-11; `E16-log.md`): a seat a logged-in connection
  takes is its account's, re-taken by it alone without a token (`session::refusal`), also paused
  and after a restart (journal format 4); the lobby's display names (`GameInfo::seated`); the
  replay listed by the seated accounts' preferences; a re-key closes the account's other
  connections (`Rekeyed`); wire version 13.
- **T4a A game's bounds** (done 2026-10-11; `E16-log.md`): `[limits] commands_per_game` (a game
  reaching it without an end void, `Goodbye { CommandLimit }`; its replay sized to fit
  `outbound_bytes`); a waiting game without a held seat leaves, its spectators told
  (`Abandoned`).
- **T4b Behind a reverse proxy** (done 2026-10-11; `E16-log.md`): `[server] trusted_proxies`
  (the client the last `X-Forwarded-For` entry names), every limit per address by IPv6 /64, the
  operator's setup with TLS (`crates/h3-server/README.md`).
- **T5 Closure** (done 2026-10-12, the checkpoint passed the same day, SRV-Q18; `E16-log.md`):
  `06-security.md`'s accounts, ADR-0018 points 6 and 8 amended, the client lane's note, the exit
  criteria checked, the reviewer agent (run before #105's pull request).

## M4 tasks

Cut at the M4 refinement (2026-10-12, SRV-Q19); the tasks in full: `docs/lanes/server/QUEUE.md`,
the working detail: `E16-design.md` §11. ADR-0019 points 4–9 (decided at the refinement): a
season follows the ladder's ruleset (ratings carried, deviations reset); one queue at a time;
the ratings written to the ladder's results log, never recomputed; a disconnect's grace and a
total per player and game (5 and 10 minutes by default), beyond either a forfeit; one unfinished
ranked game per player per ladder, bots exempt; no turn timer in M4 (a game rule, `h3-game`).

- **T1 Ladders, ratings and the results log** (done 2026-10-12; `E16-log.md`): `[[ladder]]`,
  Glicko-2 (`rating`) with Glickman's example as its test vector, `<data>/ladders/<id>.log` read
  back at start, seasons by the ruleset hash; no wire change.
- **T2 The queue and the match** (done 2026-10-12; `E16-log.md`): `h3_protocol::ladder` (the
  queue, the matcher, the accept step, one queue per account, one unfinished ranked game per
  ladder), the messages, the matched game created with both seats bound to their accounts; wire
  14; until T3a and T3b SRV-STUB-3 and -4.
- **T3a A ranked game's seats and forfeits** (done 2026-10-12; `E16-log.md`): no host; the
  session's clock, the disconnect grace and total, forfeits and no-shows (`Outcome::Forfeit`,
  `NoShow`) in the journal (format 5), across a restart; wire 15; SRV-STUB-3 resolved.
- **T3b The rated end** (done 2026-10-12; `E16-log.md`): a ranked game's end held until it
  verified (`Outcome::Unverified` otherwise), the result rated and logged (`Rated`, `NoShow`),
  `Rating` to the seats, game ids unique for the server's life (`<data>/next-game`); wire 16;
  SRV-STUB-4 resolved.
- **SRV-Q20 Server-side pings** (done 2026-10-12, the user's choice (a); `E16-log.md`): a ping
  every `ping_every_ms`, a connection silent for `ping_timeout_ms` closes; a dropped seat is
  noticed in seconds, an outage ends void for both seats alike.
- **T4 Leaderboards** (done 2026-10-12; `E16-log.md`): `Leaderboard`/`LeaderboardPage` by
  season, pages of 50 by rating (ties by login name), `Standing`/`OwnStanding` with its rank,
  display names as they stand; wire 17.
- **T5 Bots on ladders** (done 2026-10-12; `E16-log.md`): `[[ladder.bot]]`, bot accounts
  without a key, the bot for a player who waited `bot_after_ms`, its seat its account's, rated,
  flagged in the ladder list, the lobby and the leaderboard; wire 18.
- **T6 The pause by agreement** (done 2026-10-12; `E16-log.md`): `ProposePause`/`PauseProposed`,
  `AnswerPause`/`PauseAnswered` within the ladder's `pauses` and `pause_ms`, the agreed length
  journaled (format 6), the absences held until the due time, the game woken then; one ask per
  seat until the game moves on, for unranked games' `Pause` too; wire 19.
- **T5b A player's bot opt-out** (done 2026-10-12, SRV-Q21; `E16-log.md`): `Queue { …, bots }`
  (`false`: people only, however long the wait), `MatchFound::bot`; wire 20.
- **T7 Closure** (done 2026-10-12; `E16-log.md`): a rated battle through a restart end to end,
  `06-security.md`'s ladders and pings, the client lane's note, the reviewer agent (no major
  finding; five minor fixed, the bar after a match given up among them), the user's checkpoint
  (passed, SRV-Q22).
- **T8 An agreed pause's hold across a restart** (done 2026-10-12, SRV-Q22 (b); `E16-log.md`):
  the hold of a pause resumed early journaled (`Held`) and re-applied at a restore, over at its
  due time; journal format 7, no wire change.
- **M4's review items** (done 2026-10-12, the user's request; `E16-log.md`): a match that stops
  with its journal kept keeps its players until the next start; a start re-runs the journals one
  by one by game id; and from the inbox, a drain or a stop a full inbox missed, a rename racing a
  re-key.

## M6 tasks

Cut at the M6 refinement (2026-10-13, SRV-Q23); the tasks in full: `docs/lanes/server/QUEUE.md`,
the working detail: `E16-design.md` §12. ADR-0021: a server key and a signed welcome; a signed
card; peers by key and address, fetched over TLS (`rustls` with `graviola`), never an address
learnt from a card; the vouching chain checkable from the cards; lists of servers signed by a
list's key; mDNS later.

- **T1 The server key and the signed welcome** (done 2026-10-13; `E16-log.md`): `<data>/server.key`,
  the hello's nonce and dialed address, `[server] addresses`, the welcome's key and signature, the
  test client's check and pin; wire 24.
- **T2 The card and `Describe`** (done 2026-10-13; `E16-log.md`): `ServerCard`, `SignedCard` and
  their bounds, `Describe`/`Described`, `[server] name`, `description`, `operator`, `[discovery]`,
  the wall clock, `h3-server card`; wire 25.
- **T3 Peers, the fetch and `ListPeers`** (done 2026-10-13; `E16-log.md`): `[[peer]]`,
  `[discovery] refresh_ms` and `timeout_ms`, the TLS client (a `shared:` commit), the fetch and its
  bounds, the cache, `ListPeers`/`Peers`, `card::chain`; wire 26.
- **T4 Beyond the peers** (done 2026-10-13; `E16-log.md`): the cards the peers relay, to
  `[discovery] depth` hops and at most `cards`, each with its `via` checked against the cards at
  the fetch and again against what the server serves; the test client checks each chain; no wire
  change.
- **T5 Lists** (done 2026-10-13; `E16-log.md`): `ServerList`, `SignedList` and their bounds,
  `<data>/lists/` read each round, the listed servers fetched and relayed marked with the list's
  key, `Lists`/`ListsAre`, `h3-server list-key` and `sign-list`; wire 27.
- **T6 Closure** (done 2026-10-13; `E16-log.md`): `06-security.md`'s discovery
  and the client lane's inbox (`cross-lane`), the threat table's rows, the exit criteria shown, the
  reviewer agent (two major findings and four minor fixed: the roots for a listed `wss://` server,
  a listed card naming its address); the user's checkpoint (passed, SRV-Q24).
- **The inbox after M6** (done 2026-10-19; `E16-log.md`): `h3-server`'s three Windows test
  failures, the query `CreatureCastTargets`, a side's `may_retreat` as its viewer knows it and
  `last_side_acted`'s doc (wire 34), a siege's frames over the wire, the example's ladder on
  `strict-fixed` (BAT-QUIRK-4, the user's choice).

## Exit criteria (M6)

The full `cargo xtask verify` green (in CI, or the full local verify while CI cannot run) with
`h3-protocol` under the 100 % gate, and shown in tests over loopback: a client verifies the
welcome's signature at each of the server's addresses and refuses a changed key at an address it
pinned; the key survives a restart, a key file that does not parse stops the start; a card
verifies, and a tampered, expired or foreign card fails; three servers A → B → C: A serves B
(vouched by A) and C (via B), each link checkable from the cards, and A never dials C; a card a
peer serves outside its own vouching, or forged, is dropped and logged; the depth and count
bounds hold; a peer fetched at `wss://` (a test certificate and a test root); a list's servers
appear marked with its key, an expired or forged list is skipped. And: `06-security.md` carries
discovery; the client lane's inbox has what a server browser needs; the operator's notes carry the
key, peers, lists and the TLS client's platforms; a fresh reviewer agent's major findings are
fixed; the lane HANDOFF, the epic log and `QUEUE.md` are current.

## Review list for the M6 checkpoint (passed 2026-10-13, SRV-Q24)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: `<data>/server.key` (the seed as hex, mode 0600, never replaced); the hello's `nonce` and
  `address`, the welcome's `key` and `signature` over `h3-welcome-v1`, the id and the challenge;
  `[server] addresses`, the id per connection.
- T2: `Describe`/`Described` after the hello, without a login; `ServerCard`'s fields (the counts
  `Option`al, `null` under `[discovery] counts = false`) and `SignedCard` (the text as signed after
  `h3-card-v1`); the bounds (8 KiB, 8 addresses, 64 peers, 16 rulesets of at most 16 packs, 16
  ladders, names, ids and versions of 64 bytes, a description of 1024 and an operator contact of
  256, no control characters but the description's `\n`, 10 minutes' clock skew); `[server] name`
  (the server's id by default), `description`, `operator`, `[discovery] card_ms` (a day; 10 minutes
  to 30 days) and `counts` (on); a card re-signed when asked and its last one a minute old; what
  the counts count (the open connections, the games waiting or running, each ladder's current
  leaderboard); `h3-server card` asking the running server on loopback; the wall clock behind one
  local `allow`.
- T3: `tokio-rustls` (no default features) as `h3-server`'s dev-dependency for the tests' TLS
  endpoint (in the tree already); `[[peer]] key address` (at most 64, no key or address twice,
  not the server's own key); `[discovery] refresh_ms` (an hour; a minute to a day, from a round's
  start) and `timeout_ms` (10 s; a second to two minutes); one fetch at a time in the
  configuration's order, each a task of its own (graviola's panic on an older processor fails the
  fetch, logged); the welcome's id must be the address dialed; a fetch's bounds (a frame of 2 × 8
  KiB + 1 KiB, two in all); the hello's name (the server's id); a peer of another protocol version
  skipped; the last good card kept until it expires; `RelayedCard { card, via, list }`, pages of
  50, `card::chain`; the OS roots read only when a peer is at `wss://` (`SSL_CERT_FILE` and
  `SSL_CERT_DIR` honoured); the test certificates in hex in the test, valid until 2126.
- T4: `[discovery] depth` (2; 1 to 4) and `cards` (200; 1 to 1000); a fetch reads at most `cards`
  of a peer's cards, its frames up to a page of 50 cards (851 KiB) and its bytes bounded by the
  pages; the chain checked at the fetch against the peer's cards and again against what the server
  serves (a link's card may come from a shorter chain); a card a peer relays for a list not
  relayed on (until T5); a card served until the first card of its chain expires; memory up to
  `[[peer]]` × `cards` cards; one log line per dropped card.
- T5: `Lists { seq }` → `ListsAre { seq, lists }` (all at once, at most 16); a list's bounds (32
  KiB of text, 128 servers, no key twice, a name of 64 bytes), `list::names` (the list names the
  card's key at one of the card's addresses); one reader behind cards and lists
  (`card::read_signed`, the error `CardError`); `<data>/lists/*.json` in their names' order, a
  second list of one key skipped; the listed servers after the peers' own cards and before the
  peers' relays, each once, never a peer, within `cards` less the peers, at depth 1; a card a peer
  relays for its list not relayed on; the TOML list (`name`, `valid_days` 1 to 366, `[[server]]`);
  `list-key` reading an existing key file; the key file's error text now "not a key" for both.
- T6 (the reviewer's fixes): the OS roots read at the first fetch at `wss://`, a peer's or a listed
  server's (no longer at start, for peers only); a listed server's card taken only if it names the
  address its list names (a peer's card is checked by its chain alone); the card's default name the
  id's first 64 bytes; a list's file opened only if it is a regular file, read within its bound.

## Exit criteria (M4)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests over loopback: two clients queue, accept and play a rated battle through a server
restart, their ratings as Glicko-2's test vector computes them and the same after another
restart; a declined or expired match puts the other player back in the queue; a second queue,
or a second ranked game on the same ladder, is refused; an anonymous connection never queues;
a player away within the grace re-takes the seat by account, beyond the grace or the total
forfeits; both away: void, no rating change; a ranked game that does not verify is unrated and
its players are told; a content change starts a season with the deviations reset; the
leaderboard pages by season; a bot account plays a rated game, flagged; a pause by agreement
within the ladder's bounds, and a forfeit after its due time. And: `06-security.md` carries the
ladders; the client lane's inbox has the ladder's messages; a fresh reviewer agent's major
findings are fixed; the lane HANDOFF, the epic log and `QUEUE.md` are current.

## Exit criteria (M3)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests over loopback: a client registers and logs in with the key its login name and password
derive (a test vector); a login signed for another server's id, with another connection's
challenge or by another key is refused; a seat taken by an account is re-taken by a new
connection logged in as it, without a token, also after a restart; a re-keyed account logs in
with its new key only; a display name changes and stays unique; anonymous play as configured;
a game beyond its command limit is void; a waiting game without a held seat leaves; behind a
trusted proxy the limits count the client's address. And: `06-security.md` carries the
accounts; the reverse-proxy setup is documented; the client lane's inbox has the login; a fresh
reviewer agent's major findings are fixed; the lane HANDOFF, the epic log and `QUEUE.md` are
current.

## Exit criteria (M2)

The full `cargo xtask verify` green in CI with `h3-protocol` under the 100 % gate, and shown in
tests, over loopback where the server is involved:

- many games on one server through the lobby: listed with their offers before joining, created
  only from the server's offers, joined and played at once, within the limits per address and in
  total;
- the write-ahead journal: a game stopped mid-battle restores at the next start to what its
  players were told and plays on to a replay that verifies; a journal of other content, of a
  missing offer or that does not re-run restores nothing and is kept aside, void;
- every finished game re-run from its journal on disk before its replay is published, a game that
  does not verify void (`Unverified`); the shadow instance finds and repairs a divergence before
  anything is sent;
- the host's pause, the resume by a seat's token (also after a restart), the deadlines that pause
  an abandoned running game and void an abandoned paused one;
- spectators through the battle's spectator viewer, within the host's setting and the server's
  limit, seeing nothing a seat's viewer hides;
- a first signal drains and a second stops, the journals kept;
- the connection's bounds: the frame size, the rates per connection and per address with the
  WebSocket pings, the connections, the deadlines for the hello, the lobby and idling, the
  outbound queue and the record in bytes;
- the archive: finished, verified games listed and downloaded without a game, by ids it names,
  unlisted when closed to spectators, at most `replays_kept`, read back at a start;
- M1's security properties (identity, rejection, projection, secrets) held over the lobby,
  restored games and spectators; the decoders and a running session never panic on mutated valid
  messages.

And: `docs/06-security.md` carries M2's guarantees and names what is not covered yet; the
workload has a current baseline; the client lane's inbox has the content check and what a
restart does to a viewer's numbers and record; the inbox's M2 items are decided; a fresh reviewer
agent's major findings are fixed or recorded with the user's decision; the lane HANDOFF, the epic
log and `QUEUE.md` are current.

## Review list for the M4 checkpoint (passed 2026-10-12, SRV-Q22)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: `[[ladder]]`'s fields and defaults (`accept_ms` 30 s, `pauses` 0); a ladder's id as its
  log's file name, its name under a display name's rules; the results log's lines (`Season`,
  `Rated` with the ratings after the game as `f64`, `NoShow`), checked against the player log; a
  season started only at a start whose ruleset is new; `rating` public, Glicko-2 with no
  deviation growth for inactivity; `exp` and `ln` behind a local `allow` of the workspace's ban
  (an adapter, ADR-0003 §3's reason does not apply: no rating is computed twice).
- T2: the messages (`ListLadders`/`Ladders` with each ladder's season and waiting count, `Queue`/
  `Queued`, `Dequeue` and `DeclineMatch` answered `Dequeued`, `AcceptMatch` answered by
  `MatchReady` once both accepted, `MatchFound { ladder, accept_ms }`, `MatchCancelled`,
  `Unqueued { Expired | Draining | Full }`, five refusals, `GameInfo::ladder`); the window (100 +
  50 per 30 s of the shorter wait) and the greedy pairing by rating; the queues in memory only;
  the matcher every second; a match's game created as the server's own (the limit in total, none
  per address); a pool's scenarios as offers `<ladder>.<n>`, an `[[offer]]` so named refused; a
  match's waiting game never leaving the lobby; the lobby's deadline waiting while a connection
  is queued.
- T3a: `Outcome::Forfeit { side }` and `Outcome::NoShow {}` (only in `GameEnded`) and `NotHost`
  for a ranked game's `Pause` and `SetSpectators`; a void game when the other seat is empty as
  one's grace runs out; "beyond" as strictly more; the clock moved by messages, closed
  connections and the sweep (a quarter of the shortest grace, at most a minute: a forfeit comes
  that much late at most); a stretch running at a stop not journaled (the part before the stop
  is not counted either); a forfeited or void game ended as any other (verified, its undecided
  replay archived by the usual rule; a no-show of a game that never started gets an empty one);
  `Entry::Ranked` a mark, the grace the ladder's configuration at each start; a waiting match's
  game still ends at a drain or a stop (gone, unrated); the restored matches back in the
  registry's ranked games at start (T3b's item, taken here).
- T3b: only a ranked game's end waits for the verification (an unranked game's is told before,
  as since M2 T3); a void ranked game told by `GameEnded { Unverified }` (not a goodbye: its
  connections stay, as after any end); `Rating { n, ladder, before, after }` numbered and in the
  record, to the seats only, `Points { rating, deviation }` as whole numbers (`i32`, `u32`);
  the order verify, archive, rate, then the journal goes (a result that does not write keeps the
  journal, a re-run's second copy of the replay accepted); a game named twice in a results log
  stops the start; `<data>/next-game` written whole on each game's thread (one sync per game),
  at start raised by every journal's id; a seat without an account in a ranked game left unrated
  (none occurs).
- SRV-Q20: `ping_every_ms` 15 s and `ping_timeout_ms` 30 s (0 turns pings and the timeout off),
  for every connection, the lobby's too; any frame from the client counts as heard (a message, a
  ping, a pong), the server's own writes excused from the silence; the silent connection told
  `Goodbye { IdleTimeout }` within a moment (no new reason, no wire change); (b), the clocks
  standing still on a mass drop, not built.
- T4: the messages (`Leaderboard { seq, ladder, season, page }` answered `LeaderboardPage { seq,
  ladder, season, total, rows }`, `Standing { seq, ladder }` answered `OwnStanding { seq, ladder,
  season, total, row }`, `LeaderboardRow`, `Refusal::NoSuchSeason`; pages from 0); one's own row
  only for the current season and only logged in; a page needing no login; the order by the
  rating itself (not the rating less twice its deviation), exact, ties by login name; listed from
  the first rated game (no-shows alone do not list an account); no bound beyond the page of 50
  and the message rate; `bot` false until T5.
- T5: `[[ladder.bot]]` (`login`, `display`, `kind` with `random` the only one, not journaled: a
  restart's bot is the random one) and `bot_after_ms` (60 s); the bot for whoever waited that
  long, nearest in rating, outside the window; the bot accepting at once; the player log's `Bot`
  line and `Account::key` an `Option`; a bot's display name from the configuration (renamed at
  start), a player's account under a bot's login stopping the start; `LadderInfo::bots` as
  display names, `Seated::bot`; the bots' ratings passed to the queues before each matcher's
  turn; the lobby naming a match's accounts at creation (a fix).
- T6: the messages (`ProposePause { seq, ms }`, `AnswerPause { seq, accept }`, `PauseProposed {
  n, seat, ms }` and `PauseAnswered { n, seat, accept }` to every viewer, spectators too), five
  rejections (`NotRanked`, `AlreadyAsked`, `NoPausesLeft`, `BadPauseLength`, `NoProposal`); one
  proposal open at a time, lapsing when the game moves on (a command applied), no
  counter-proposal; a pause counted against its proposer's `pauses`; the due time from the
  journal's last write (as `paused_game_ms`), a restart's downtime counted (a due time passed
  while the server was down wakes the game at its first sweep: the grace from the start); a woken
  game runs with its players away; an agreed pause never voided by `paused_game_ms`; `ms` a
  plain JSON number (at most a year); a bot never answers a proposal (it lapses).
- T5b: `Queue::bots` an `Option<bool>` (left out or `None`: bots welcome), sent with each `Queue`
  and kept through a match given up, not stored; `MatchFound::bot` told to the player of a bot
  match (a person's match: `false` to both); an opted-out player still pairs with people only by
  the window, which widens as before.
- T7 (the reviewer's findings): `ladder::BARRED_MS` (60 s) for an account that gave a match up
  (a decline, an expiry, leaving it), per ladder, the player not told (it queues as ever and
  waits); a match's game out of the ranked games before its end is told; game ids beyond every
  ladder log's games at start; the ladders read off the async tasks; `MatchReady` told as the
  game is created, its seats journaled before any join reaches it (the note corrected, not the
  code). Left for the checkpoint: the hold of an agreed pause resumed early, lost at a restart;
  the rating order of journals re-run after a failed log write; a game whose journal stops
  writing leaving the ranked games.

## Review list for the M3 checkpoint (passed 2026-10-12, SRV-Q18)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: the messages (`Login { seq, login, key, signature, display }`, a registration by its
  `display`; `Rekey { seq, key, by_old, by_new }`; `Rename`; `SetListed`; one answer `Account
  { seq, display, listed }`; `Refused` with `BadLogin`, `BadName`, `NameTaken`, `NotLoggedIn`,
  `AlreadyLoggedIn`, the connection kept open); the welcome's `id` and `challenge`; keys and
  signatures as lowercase hex; the signed bytes (a purpose with its version, then the id, the
  challenge, the login name and the display name, texts after their 4-byte length; a re-key's
  over the new key); a login name only in lowercase; display names of 3–24 ASCII letters,
  digits, `_ - .` and single inner spaces (no lookalike letters on leaderboards); without
  anonymous play only `Join` and `CreateGame` wait for the login (the lists, the archive and
  queries do not); the server's id the address it listens on until T2a's `public_url`.
- T2a: the `Authenticator` as the signature check alone (`verify(key, message, signature)`),
  the rules in `accounts`; Ed25519's strict check; the player log's line (`Registered { login,
  key, display }`) and its reading (a torn last line cut, any other bad line stops the start;
  after a failed write registrations are refused); a display name sent with a known account's
  login ignored; an unknown login name without a display name `BadLogin`; `[server] public_url`
  (`ws://` or `wss://`, at most 200 visible characters after it) and `anonymous`; `[limits]
  failed_logins_per_address` (10 a minute, kept beyond the address's connections; beyond it a
  login closes its connection `RateLimited`, the address's other requests pass); a login answered
  off the connection's task (`spawn_blocking`, the player log under its own lock); `argon2` a
  normal dependency of `h3-server` for its `client` module (tools may log in); the shared lines
  for the two crates (`Cargo.toml`, `arch.toml`) in a commit of their own on the lane's branch.
- T2b: the player log's lines as typed events (`Rekeyed`, `Rebound` apart from `Rekeyed` for the
  operator's audit, `Renamed`, `Listed`), checked against the accounts on reading and before
  writing; a request that changes nothing answered without a line; a re-key's connections logged
  in already stay so; a failed re-key not counted as a failed login; `h3-server rebind
  <config.toml> <login> <public-key>` (the key checked as an Ed25519 key that is not weak);
  `<data>/lock` (`File::try_lock`), held by a running server for its life and by the command, so
  a second server on the same data directory does not start either.
- T3: an account's seat taken by no token (`SeatTaken`, no new refusal) and its `Joined` without
  one; the seat's account fixed at its first take (a login after opening the game does not bind
  it); one account in both seats allowed; `Entry::TakenBy` beside `Entry::Taken`, the journal's
  `Taken { side, account }`; `GameInfo::seated` (`Seated { seat, display }`, anonymous and bot
  seats unnamed), its names as they stand at the seats' last change (a rename shows later); an
  account the player log does not know keeps a replay unlisted; the replay's listing decided when
  the game ends; `GoodbyeReason::Rekeyed` and the connections by account (`Service::sessions`).
- T4a: `commands_per_game` (10 000) and the replay's size estimate (160 bytes a command, 64 KiB
  besides) the configuration checks against `outbound_bytes`; the limit checked by the game's
  thread after each step (a few bot commands past it possible); void at the limit (no draw, no
  replay), `Goodbye { CommandLimit }` and `Abandoned`; a waiting game without a held seat leaves at
  once, as one without connections did.
- T4b: `trusted_proxies` as exact addresses (no ranges); `X-Forwarded-For` alone, its last entry
  (not RFC 7239's `Forwarded`); a proxied connection counted in the total only until its
  handshake; the proxy's own address without a header that reads; the /64 for every limit per
  address, the lobby's included; the operator's page as the crate's README.
- M3's reviewer (run before T5): `account_changes_per_address` (20 an hour; every request that
  may write the player log, closing its connection beyond); the re-key's kick taken before a
  client message and only for another key; a refused line truncated off the log; a warning
  without `public_url` on an unspecified address or behind proxies.

## Review list for the M2 checkpoint (passed 2026-10-11, SRV-Q16)

Choices queued under the autonomy contract (details in `E16-log.md`):

- T1: the lobby's messages (`CreateGame` names an offer and the creator's seat and joins it at
  once, so a game never waits without a connection; no `StartGame`: a game starts when every
  seat is held; `Games` lists the offers with the games); the refusals `NoSuchOffer` and
  `TooManyGames` (also for a creation the OS randomness failed), `SeatTaken` for the bot's
  seat; the limits counted per address and over games that are not over; the configuration's
  sections; one connection in one game at a time (joining another leaves the first, its seat
  waiting for its token); a game leaves when it is not running and no connection is bound to
  it (a waiting game whose creator leaves is gone, a running one waits for its players); the
  replay file named by the offer.
- T2: the entries (`Taken`, `Started`, `Command`, `Checkpoint`) and the restore's rules (the
  journal must begin the re-run's entries; a journal cut short is completed, the bot acting if it
  is its turn; a viewer's numbers and record start afresh); `JOURNAL_FORMAT_VERSION` 1 and the
  file's layout (JSON lines; the first line with the creator's address, the tokens and the
  replay before its first command as the CLI's TOML); `serde_json` in `h3-server` (already in the
  workspace); a journal that does not write stops its game with `Goodbye { Shutdown }`; the void
  directory; owner-only directories and journals on Unix; ids of ended games reusable after a
  restart.
- T3: the verification reads the journal back from disk (not the session's copy) and publishes
  the replay it re-ran; a game that does not verify is void: `Rejected { Unverified }` for
  `FetchReplay`, the journal in `<data>/void/` and the replay as played beside it, the diagnosis
  (with hashes) as its first line, the log line without hashes; `GameEnded` is still sent before
  the check. Who pauses: the host (the seat taken first, a created game's creator) alone; another
  seat's ask goes to every viewer as `PauseAsked` (the design note's "or all seats" adds nothing in
  a two-seat battle, the host being one of them). No `Resume` message: a seat's `Join` with its
  token resumes (the registry checks the token before it restores the journal); no `Paused` or
  `Resumed` message: `Goodbye { Paused }` closes every connection of the game. The wire's
  additions (version 5: `Pause`, `PauseAsked`, `Phase::Paused`, `GoodbyeReason::Paused`,
  `RejectCode::Unverified`) and the journal's lines `Paused`, `Resumed` (format 2; a format-1
  journal is void after the upgrade). A paused game counts against the limits like a running one;
  its deadline (`pause_hours`) moves to T5 with the deadline of a game nobody returns to. The
  shadow instance's repair is silent for the players (the command is re-applied to the rebuilt
  battle before anything is sent), a log line names the step.
- T4: a spectator is a viewer per connection, numbered from its join, its record since then;
  nothing of it is journaled (after a drop or a restart it joins again); its messages for a seat
  are rejected `NotSeated` *numbered* (it is a viewer, unlike a connection in no game); it gets
  `PauseAsked` and `Goodbye { Paused }` like the seats, and a paused game takes no spectator (the
  registry refuses before it restores); the limit `[limits] spectators_per_game` (8, 0 for none,
  at most 1024) refused as `NoSpectators` (its meaning widened: no more spectators); spectators
  count against the connection limits only. The wire: `Observation::spectator` (version 6).
- T4b: the new fields' shapes (per-stack values on the wire's `Stack`, per-side ones on
  `SideView`; binders as the binding stacks' ids, spell sets as handles, rates as basis points);
  left out on purpose: a rate's single-precision value (quirk Q-0005), the stacks' base attack,
  defense and speed (their values now and the effects' shares are on the wire) and the values
  materialised from the effects (protections, status, …; the effects carry their spells,
  masteries and values); the completeness as patterns in the mapping, a build failure like a new
  variant's (the queue asked for a test); `h3-contracts` a normal dependency of `h3-protocol`
  (an edge `arch.toml` allowed already).
- T4c: the messages (`CreateGame`'s optional `spectators`; `SetSpectators { seq, open }` answered
  `SpectatorsSet { n, seq, open }`, numbered to the host's seat; the reject codes `NotHost` and
  `AlreadyStarted`; `Goodbye { SpectatorsClosed }` for the spectators a closing ends;
  `GameInfo::spectators`, false for a paused game and on a server without spectators); the
  setting journaled once, as the game started (a `Spectators` line before `Started`, format 3); the
  other seat is not told of a change (in a two-seat battle it can join only a waiting game, which
  starts then; the lobby showed it the setting). Not built: an offer's own setting for the
  ladders' games, which have no host (M4).
- T5a: the signals (the first SIGINT or SIGTERM drains, a second stops; no drain without a
  signal, no other trigger); a drain ends waiting games (`Goodbye { Shutdown }`, their journals
  removed) and lets paused ones resume; `Refusal::Draining`; a stopped game's journal stays (a
  waiting one's goes), its connections hear `Goodbye { Shutdown }`; the deadlines in milliseconds
  like `idle_timeout_ms` (defaults 30 minutes and 72 hours, 0 for none, at most a year), the
  sweep every quarter of the shortest (10 ms to a minute); an abandoned running game pauses
  as if its host paused it (its spectators hear `Goodbye { Paused }`), an abandoned paused one
  goes to `<data>/void/` with the reason; after a restart a paused game's clock counts from its
  journal's last write.
- T5b: the address's rate (40 a second by default, twice a connection's) closes the connection
  whose message passes it (`RateLimited`), and starts afresh once the address has no connection;
  every WebSocket control frame counts (a ping, an unasked pong), none keeps the connection from
  idling; the two deadlines (10 s for the hello from the handshake, 60 s in no game from the hello
  or from leaving a game) say `Goodbye { IdleTimeout }` (no new reason, no wire change); a game's
  answer marks the connection's message after which it holds nothing (`Frame::left`), and the task
  leaves only if it passed the game no later message (a pipelined second join may take a seat).
  Not built: a separate rate for automated seats (T3's note): an operator cannot know a client is
  an agent before accounts, so it waits for M3's bot accounts; until then an operator raises
  `messages_per_second` and `messages_per_address`.
- T5c: the record as a window of the latest messages (the oldest go, the newest always stays,
  numbering goes on; `RecordHeader::first`, format 2, wire version 10) rather than refusing
  what would grow it; the bounds 512 KiB per record and 2 MiB per outbound queue, a record to fit
  twice in a queue (its frame escapes its text once more); the bytes counted by the game as it
  queues and by the task as it takes out, a goodbye queued whatever the bytes. Not built as its
  own rule: one `FetchRecord` in flight. The queue's bytes bound what queued records hold, and a
  client that asks for them faster than it reads is dropped as `TooSlow`.
- T5d: the mutations (a flipped byte, a cut, an insertion, a repeat, a splice of two messages, a
  number or a string replaced by a plausible or an extreme one) as proptests in `h3-protocol`,
  no fuzzing dependency (SRV-Q14). The archive's messages (`ListReplays { seq, from }` answered
  `Replays { seq, total, replays }`, 50 a page, newest first; `FetchArchived { seq, replay }`
  answered `Replay` numbered 0 or `Refused { NoSuchReplay }`), served by the connection's task
  without a game; the replay's id is its file's stem (offer, game, a random suffix: unique across
  restarts); the opt-out is the host's spectator setting as the game started, whatever the
  server's spectator limit (an unlisted replay in `<data>/replays/closed/`); `replays_kept` (10
  000 in each part), the oldest file deleted, also at a start with a lower bound; the order the
  files' last writes.
- T5e: the inbox's placement (`QUEUE.md`: the commands-per-game limit in M3, before the internet,
  rather than now; ids and the verification before `GameEnded` in M4; a viewer's record across a
  restart documented for the client lane, not kept). The review's fixes: a game prepares on its
  own thread (`game::launch`), so it is listed before its journal exists; a creation whose secrets
  or journal fail, and a resume whose journal no longer reads, end with `Goodbye { Shutdown }` (no
  new wire reason); `Frame::held` for the lobby's deadline; `.part` files for replays. Left for
  the checkpoint (inbox): a waiting game's deadline, the archived replay's bytes, repeated
  `PauseAsked`, the per-address limits behind a proxy, lost `Drain` and `Tick`. **For the user:** a
  finished game's replay is public unless its host closed it to spectators; the other seat has no
  say (M3's accounts could give it one).

## Review list for the M1 checkpoint (passed 2026-10-11)

Choices queued under the autonomy contract (details in `E16-log.md`):

- M0: `futures-util` for the socket's `StreamExt`/`SinkExt`; `h3-server` → `h3-cli` for the
  pack, scenario and replay-file functions (provisional; the alternative is a battle-lane move
  into `h3-sim`).
- T1: the message schema in `wire.rs`, with three additions to the design note: `Refused` for
  a refused join (the connection stays open), the session's reject codes `NotSeated` and
  `NotStarted`, and the manifest in `Joined` (ADR-0018 point 10), not in `Welcome`; the
  observation as the sim encoding's values in named fields (the own hero's spell bonuses are
  not on the wire yet); hexes as indices, the field's size in the manifest; flag sets as named
  booleans; `DEFAULT_MAX_CLIENT_FRAME` 4096 bytes; `postcard` as a dev-dependency for the
  round trips.
- T2: the wire amended (`PROTOCOL_VERSION` 2): `seq` in `FetchReplay`, `FetchRecord`, `Replay`
  and `Record`, the reject code `NotOver`; the record's format (a `RecordHeader` line, then the
  viewer's numbered frames; `Replay` and `Record` take no number); the start rule (every seat held
  at once; two bot seats never start); a seat taken without a token only the first time; an
  `Events` message for every accepted command, even an empty one; the bot hook as a trait object
  given the state, a rejected choice replaced by the first legal command.
- T3: the configuration's format and defaults (`h3-server/src/config.rs`); the thread and
  channel layout (a game inbox of 1 024, `try_send` into per-connection queues); idle meaning no
  message from the client; the lingering close; the replay file's name; `getrandom` 0.3 beside
  the 0.4 that `tungstenite` brings (a `cargo deny` warning).
- T4: the workload's shape (`serve_frame` counted, scripted clients that strike when they can,
  20 games of `full`); **lane-only workloads** (a shared `xtask` change: the server's count is
  55 % battle engine and follows the battle lane's packs, so the full verify does not measure
  it; the server lane re-baselines a drift at its sessions' start); the shared changes ride
  this branch's pull request with the label `cross-lane` (the user, 2026-10-10).

## Exit criteria (M1)

The full `cargo xtask verify` green in CI with `h3-protocol` under
the 100 % gate; the end-to-end tests pass over loopback; the security properties above hold in
tests; the workload has a baseline; `docs/06-security.md` carries the server rules; a replay
written by the server verifies with `h3 replay verify`; the lane HANDOFF, the epic log and
`QUEUE.md` are current.
