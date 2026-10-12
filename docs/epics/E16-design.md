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
8. `GameEnded { n, result }` to every viewer (a ranked game's once it verified, M4 T3b, then
   `Rating` to its seats). Participants may send `FetchReplay { seq }` and
   get the full replay (`Rejected { NotOver }` before the end); any viewer may send
   `FetchRecord { seq }` and get its own message log, the *record* (since M2 T5c its latest
   messages within `[limits] record_bytes`, the header naming the first one kept). The server
   finalizes the journal as the replay file (the CLI's TOML), public since M2 T5d (SRV-Q15): any
   client lists the archive (`ListReplays`) and downloads a replay (`FetchArchived`) without a
   game; a game closed to spectators is archived unlisted.

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
M3: `Login`, `Rekey`, `Rename`, `SetListed` (code since M3 T1). M4 (§11): `ListLadders`,
`Queue { seq, ladder, bots }`, `Dequeue`, `AcceptMatch`, `DeclineMatch` (code since M4 T2;
`bots` since T5b); `Leaderboard { ladder, season, page }`, `Standing { ladder }`,
`ProposePause { ms }`, `AnswerPause { accept }`.

Server to client: M2: `Games { seq, offers, games }` and `GameCreated { seq, game }` (code since
M2 T1; the refusals `NoSuchOffer` and `TooManyGames`), `PauseAsked { n, seat }` and
`Goodbye { Paused }` (code since M2 T3, instead of `Paused` and `Resumed`), `SpectatorsSet { n,
seq, open }`, `Goodbye { SpectatorsClosed }`, the reject codes `NotHost` and `AlreadyStarted`
and `GameInfo::spectators` (code since M2 T4c). No `StartGame`: a
game starts when every seat is held at once (M1 T2's rule). M3:
the welcome's challenge, `Account` (code since M3 T1). M4 (§11): `Ladders`, `Queued`,
`Dequeued`, `MatchFound { ladder, accept_ms, bot }`, `MatchCancelled`, `MatchReady { ladder,
game, seat }`, `Unqueued { reason }`, `GameInfo::ladder`, the refusals `NoSuchLadder`,
`AlreadyQueued`, `InRankedGame`, `NotQueued`, `NoMatch` (code since M4 T2; `bot` since T5b);
`LeaderboardPage`, `Rating`,
`PauseProposed`, a forfeit's result.

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
  `Pause` pauses, a paused game is its journal, a seat's join with its token restores it (or as
  the seat's account, M3 T3). Since
  M2 T5a a running game whose seats all stay empty for `[limits] empty_game_ms` pauses, and a
  paused game nobody resumes within `paused_game_ms` is void. Still to come: ranked, both players
  agree, within the ladder's limit (M4 T6, §11).
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
`paused_game_ms` since M2 T5a, `messages_per_address`, `hello_timeout_ms` and `lobby_timeout_ms`
since M2 T5b, `outbound_bytes` and `record_bytes` since M2 T5c, `replays_kept` since M2 T5d,
`public_url` and `anonymous` in `[server]` and `failed_logins_per_address` in `[limits]` since M3
T2a, `commands_per_game` (a game reaching it without an end is void) since M3 T4a,
`trusted_proxies` in `[server]` (the operator's setup behind a TLS proxy:
`crates/h3-server/README.md`) since M3 T4b. Still to come:

```
[[ladder]]    id = "arena"  name = "Arena"  scenarios = ["…"]  grace_ms = 300000  # M4, §11
```

The journals (`<data>/games/<game>.journal`, open games), the replays (`<data>/replays/`,
finished) and `<data>/void/` are code since M2 T2, `<data>/players.log` since M3 T2a, `<data>/lock`
(held by a running server and by the operator's `rebind`) since M3 T2b, `<data>/ladders/<id>.log`
since M4 T1 and `<data>/next-game` since M4 T3b (§11). Still to come:
`server-data/records/<game>/<viewer>.jsonl`, snapshots beside the logs. The log never
contains seeds, tokens, nonces or signatures.

## 10. Accounts (M3)

- **Names (the user, SRV-Q17).** The *login name* is permanent: 3–20 of `a–z 0–9 _ -`, typed in
  any case and lowercased by the client (the server takes no other case); it salts the
  derivation and is shown to its owner and the operator only. The *display name* is what other
  clients see: unique on its server (ignoring case), changeable (`Rename`), chosen at
  registration; 3–24 ASCII letters, digits, `_ - .` and single inner spaces (built in M3 T1,
  `h3_protocol::account`).
- **Derivation (client side).** `seed = argon2id(password, salt = "h3-account-v1:" + login,
  memory 64 MiB, 3 passes, 1 lane, 32 bytes)`, `keypair = ed25519_from_seed(seed)` (the salt is
  at least 17 bytes, Argon2 wants 8). The same login name and password give the same key on
  every device and every server. The client caches the key locally so the derivation runs once
  per device; the password is never stored. A test vector fixes the derivation for every client
  (M3 T2a; Argon2 version 0x13, the password and the salt as UTF-8, the seed as Ed25519's secret
  key): login `alice`, password `correct horse battery staple`, seed
  `a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69`, public key
  `b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14` (computed with Python's
  `argon2-cffi` and PyNaCl; `h3-server`'s `client::Key::derive` matches it).
- **Login** (the wire and the hello gate built in M3 T1: `h3_protocol::account`,
  `connection`; checked since M3 T2a: `h3-server`'s `accounts`). `Welcome` carries the server's
  id (`[server] public_url`, what its clients dial, `ws://` or `wss://`; without one `ws://` and
  the address it listens on) and a challenge (32 random bytes from the host, one per
  connection). `Login { seq, login, key, signature, display }` (hex; `display` only to register)
  signs `account::login_message`: its purpose, the id, the challenge, the login name and the
  display name. Every account request is answered `Account { seq, display, listed }` or
  `Refused`. A client signs only if the id is the address it dialed, and the server checks the
  id it signed is its own, so a hostile server cannot relay a login. A new key with a free login
  name registers (its display name free); a known pair logs in; a known login name with another
  key is refused as a bad signature is (`BadLogin`), and so is an unknown one without a display
  name; a known pair that names a display name logs in, the name ignored (*provisional*). The
  `Authenticator` checks the signature, Ed25519 in strict mode (a small-order key or a
  non-canonical signature never holds). Failed logins count per address, beyond its connections'
  lifetimes: beyond `[limits] failed_logins_per_address` a minute (10), a login closes its
  connection `RateLimited`, as does any login from the address until the minute is over. The
  requests that may write the player log (a registration, a re-key, a rename, a preference)
  count per address too: beyond `[limits] account_changes_per_address` an hour (20), one closes
  its connection `RateLimited` (M3's review). Without
  a login a connection plays anonymously where `[server] anonymous` (default true)
  allows, with seat tokens as in M2; otherwise `Join` and `CreateGame` wait for the login.
- **Re-keying** (built in M3 T2b). After a password change the client keeps the old key until
  each server it knows has accepted `Rekey { seq, key, by_old, by_new }` at the next login: both
  keys sign `account::rekey_message` (the server's id, the connection's challenge, the login name,
  the new key); the server rebinds the account, and from then on only the new key logs in (its
  other connections, logged in with the old key, close `Goodbye { Rekeyed }`, since M3 T3). A
  re-key to the key it has changes nothing. A forgotten password is a lost account, unless the operator binds the login name to the key of the player's
  new password: `h3-server rebind <config.toml> <login> <public-key>` (*provisional*) while the
  server is stopped (`<data>/lock`: a running server holds it, and the command refuses), checked
  as an Ed25519 key, a `Rebound` line.
- **Renames and the preference** (M3 T2b). `Rename` takes a display name no other account has,
  ignoring case (one's own in another case is free; else `NameTaken`); `SetListed` sets
  `listed`. Each answered `Account`; a request that changes nothing writes no line.
- **Privacy.** A public key and a login name are shown to their owner only (a fingerprint in the
  client); other clients see display names. Anonymous connections may never queue for a ladder.
- **Seats bound to accounts** (built in M3 T3). A connection opens a game with the account it is
  logged in as. A seat it takes the first time belongs to the account: any connection logged in
  as it re-takes the seat without a token (after a drop, to resume a paused game, after a
  restart), no token and no other account takes it (`SeatTaken`), and its `Joined` carries no
  token; a seat taken anonymously keeps its token. The journal's `Taken` line names the account
  (format 4). The lobby names the seats' accounts by display name (`GameInfo::seated`).
- **A replay's listing (the user, SRV-Q17; built in M3 T3).** An account's preference (`listed`,
  default true); a finished game is listed only if its host left it open to spectators and no
  seated account opted out when it ended, else archived unlisted.
- **The player log** (M3 T2a, T2b). `<data>/players.log`, JSON lines (`Registered { login, key,
  display }`, `Rekeyed { login, key }`, `Rebound { login, key }`, `Renamed { login, display }`,
  `Listed { login, listed }`), each checked against the accounts as they stand, owner-read-write,
  each synced before its answer, read back at start: a torn last line is dropped and cut, any
  other line that does not read or contradicts the lines before it stops the start (the error
  names the line, never its text). After a failed write nothing more is written and changes are
  refused; the accounts as they stand still log in. Snapshots once a start takes too long.

## 11. Ladders and matchmaking (M4)

Decided at the M4 refinement (ADR-0019 points 4–9, SRV-Q19); the names, numbers and line shapes
below are the tasks' starting point, *provisional* until M4's review.

- **A ladder** (`[[ladder]]`, code since M4 T1): `id` (1–32 of `a–z 0–9 _ -`), `name` (a display
  name), `scenarios` (the pool: scenario files on one ruleset; the start refuses a pool whose
  ruleset hashes differ), `profile` (as an
  offer's), `accept_ms` (30 s), `grace_ms` (5 min a disconnect), `grace_total_ms` (10 min per
  player and game), `pauses` (per player and game, 0 by default: none) and `pause_ms` (the
  longest), `spectators` (whether its games take them, with `[limits] spectators_per_game`).
  Servers are independent: a ladder and its accounts belong to one server.
- **Seasons.** The ladder's results log opens each season with a `Season { season, ruleset }`
  line. At start a ruleset hash other than the last season's starts the next one (a log line
  and a `Season` line): every account keeps its rating and volatility, its deviation reset to
  350; earlier seasons stay in the log and their tables readable.
- **The results log**, `<data>/ladders/<id>.log` (read and `Season` written since M4 T1, the
  rest since T3b), JSON lines as the player log's
  (owner-only, each synced before anything depends on it, read back at start, a torn last line
  cut, any other bad line stops the start): `Season`, `Rated { game, replay, accounts: [a, d],
  outcome, forfeit, ratings: [a, d] }` (each rating `{ rating, deviation, volatility }` after
  the game, as `f64`, which `serde_json` writes and reads back exactly), `NoShow { game, replay,
  accounts }`. The tables are rebuilt from the lines: a start never recomputes a rating, so a
  server moved to another platform keeps its ladders bit for bit.
- **Rating** (`h3-server`'s `rating`, code since M4 T1): Glicko-2 as Glickman's paper gives it (rating
  1500, deviation 350, volatility 0.06, τ = 0.5, the volatility by the Illinois iteration to
  ε = 10⁻⁶), the paper's worked example its test vector; one rating period per game for both
  players, applied in the order games end; a draw scores a half. No deviation growth for
  inactivity in M4 (the server has no wall clock: the workspace bans `SystemTime`); a season's
  reset raises it.
- **The queue** (code since M4 T2: `h3_protocol::ladder`, sans-I/O; the registry in
  `h3-server`): a logged-in account waits in one ladder's queue at a time; it is refused while
  its match's game on that ladder is not over (`InRankedGame`; T3b counts restored games too).
  The matcher runs every second (the host passes the time): each ladder's players by rating
  (ties by the order they came in), neighbours paired when their distance is within the window
  of the shorter wait, 100 points plus 50 per 30 s waited (*provisional*). A pair gets
  `MatchFound { ladder, accept_ms, bot }`; both `AcceptMatch` (`MatchReady` then), or the one who
  did not (a `DeclineMatch` or `Dequeue`, the deadline, a closed connection) leaves the queue
  (`Dequeued`, or `Unqueued { Expired }`) and the other waits again with its waiting time
  (`MatchCancelled`). Who gave the match up may queue again at once, but is paired with no one
  on that ladder for 60 s (`ladder::BARRED_MS`, since M4 T7, *provisional*; M4's review: a
  player declining again and again kept another from every match). A drain empties the queues
  (`Unqueued { Draining }`). The queues live in
  memory only: a restart empties them. A queued connection hears its queue through a channel
  of its own, and the lobby's deadline waits while it is queued.
- **The ranked game** (code since M4 T2 and T3a): created when both
  accept, as the server's own (the limit in total applies, none per address; no room:
  `Unqueued { Full }`), from a scenario of the pool drawn with OS randomness, the sides at
  random, both seats the accounts' from the start (`Session::reserve`: `TakenBy` lines naming
  them, journaled by the game's thread as it opens, before any join reaches it: the players hear
  `MatchReady` as the game is created, and their joins wait behind its opening; M4's review: a
  game whose journal cannot be created ends, and its players find it gone); the clients `Join`
  it as their seat. A
  pool's scenario is an offer of its own, `<ladder>.<n>`, so a restart restores the game as any
  other. Listed with its ladder (`GameInfo::ladder`); while it waits for its players it does not
  leave the lobby, whatever their joins did (its grace ends it). Since T3a: no host (`Pause` and
  `SetSpectators` rejected `NotHost`), `Entry::Ranked` its journal's first entry;
  `empty_game_ms` and `paused_game_ms` do not apply; a restart lists it with its ladder and keeps
  its players from another match there.
- **Disconnects and forfeits** (code since M4 T3a: `Session::elapse`, the host's clock moved by
  every message, closed connection and tick of the sweep, whose period counts the ladders'
  grace). A seat whose connections are all gone is absent (a connection dropped without a close
  is gone once it answers no ping for `[limits] ping_timeout_ms`, SRV-Q20): beyond
  `grace_ms` in one stretch, or `grace_total_ms` over the game, its player forfeits (a loss;
  both absent: void, `NoShow`, no rating changes). A game not yet started counts the same way
  from its creation. The journal keeps each finished stretch (`Entry::Absent(side, ms)`) and the
  forfeit (`Entry::Forfeit(side)`), so a restart knows the totals; a stretch running when the
  server stopped restarts at the next start (the downtime never counts). A forfeit ends the
  game as a result of its own on the wire (`GameEnded` with `Outcome::Forfeit { side }`, or
  `Outcome::NoShow {}` when the other seat is empty too as one's grace runs out); the replay
  holds the commands up to it, its battle undecided (`winner = "none"`).
- **The end of a ranked game** (code since M4 T3b): the session holds a ranked game's end,
  by the battle or by a forfeit, until the host's verdict. Verified first (§8, the journal re-run
  from disk); then `GameEnded` (or `Outcome::Unverified`: void, unrated, the replay withheld);
  the replay archived; then the result written to the ladder's log (`Rated`, Glicko-2 over the
  ratings as they stood, a forfeit with `forfeit`; both forfeited: `NoShow`, unrated) and
  `Rating { n, ladder, before, after }` to each seat (numbered, in the viewer's record; whole
  points: `Points { rating, deviation }`); only then does the journal go. A replay or a result
  that does not write keeps the journal: the next start re-runs the game and writes them (a
  replay a crash wrote already is written again under another name). The log names a game once
  (`a game named twice` stops a start), so a re-run rates nothing twice. Game ids are unique for
  the server's life: `<data>/next-game` holds an id beyond every one given, recorded on the
  game's thread before its journal exists; at start the lobby follows it and every journal's id
  (an id handed out just before a crash, with no journal yet, may come again: nothing names it).
- **Leaderboards** (code since M4 T4): `Leaderboard { seq, ladder, season, page }` (the current
  season when left out, pages from 0; any connection, no login) answered by `LeaderboardPage {
  seq, ladder, season, total, rows }`, a page of 50 rows (`LeaderboardRow`: rank, display name as
  it stands, rating and deviation as whole numbers, games, wins, losses, draws, a bot flag), none
  past the last; rows by the rating itself, the highest first, ties by login name; an account
  appears after its first rated game of the season (a no-show does not list it). `Standing {
  seq, ladder }`, logged in, gives the asker's own row in the current season with its rank
  (`OwnStanding`, `None` before its first rated game). `Refused { NoSuchLadder | NoSuchSeason |
  NotLoggedIn }`. Each season's order is kept beside its table, rebuilt from the log at start.
  Ratings travel as whole numbers: `h3-protocol` holds no floats.
- **Bots** (code since M4 T5): `[[ladder.bot]]` entries (`login`, `display`, `kind`: `random`,
  the only bot so far), registered at start in the player log as bot accounts with no key
  (`Bot { login, display }`: no one logs in as them, `rebind` refuses them; a changed display
  name renamed), each a standing entry outside the queue: a player the matcher left unpaired
  after the ladder's `bot_after_ms` (60 s, *provisional*: people first) plays the bot nearest in
  rating, whatever the window; the bot accepts at once and plays any number of games at once.
  Its seat in the match's game is its account's (journaled as a player's), played by the
  server's bot; the game is rated for both. Flagged on the wire: `LadderInfo::bots`,
  `Seated::bot`, `LeaderboardRow::bot`. A player opts out per queue (code since M4 T5b,
  SRV-Q21): `Queue { …, bots: false }` waits for people only, whatever its wait (left out:
  bots welcome; not stored, the client sends it with each `Queue`), and `MatchFound::bot` tells
  a player whether its opponent is a bot, so it may decline knowingly.
- **The pause by agreement** (code since M4 T6): `ProposePause { seq, ms }` within the
  ladder's `pause_ms` and the seat's remaining `pauses`, once per seat until the game moves on (a
  command applied), while no other proposal is open; `PauseProposed { n, seat, ms }` to every
  viewer; the other seat answers `AnswerPause { seq, accept }` (`PauseAnswered` to every viewer)
  before the game moves on, or the proposal lapses. Agreed, the game pauses as a host's pause does,
  `PauseAgreed { side, ms }` in the journal (format 6). Its due time is the pause's length after
  the journal's last write (as `paused_game_ms` reads it): a game resumed before it counts no
  absence until then (`Session::hold`); at it, the registry's sweep wakes a game no one took back
  (`Session::wake`): it runs with its players away, and the grace ends it as any absence. A due
  time that passed while the server was down wakes the game at the first sweep, so the grace runs
  from the start. `paused_game_ms` never voids a game in an agreed pause. Unranked games keep the
  host's `Pause`; another seat asks once until the game moves on.
- **Stalling** is not answered in M4: a connected player may sit on a turn until the game's
  rules bring a turn timer (`h3-game`, M5; SRV-Q19).

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
