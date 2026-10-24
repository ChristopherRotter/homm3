# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md`; M0 to M4 and M6 are closed (checkpoints SRV-Q12, -Q16, -Q18, -Q22, -Q24; M6 is discovery, ADR-0021, detail in `E16-design.md` §12); the done tasks are in `E16-log.md`, the milestone chronicle in `QUEUE-ARCHIVE.md`. Work under the autonomy contract (`HANDOFF.md`). Wire changes bump `PROTOCOL_VERSION`, journal changes `JOURNAL_FORMAT_VERSION` (current values: `HANDOFF.md`). The inbox after M6 was done 2026-10-19 (`E16-log.md`).

**Next:** what the ecosystem around the server needs, talked through with the user (a refinement; no task is ready until then). M5 waits for E08's increment 3 (refined, not built); mDNS for the client lane's browser.

## Inbox

- 2026-10-23 (the client lane, E17 M4 T2a, CLT-Q20; the user's choice): **record the start's
  events.** `Session::new` starts a battle with `BattleState::new`, which drops the events of its
  start (`BattleState::start` reports them): the tactics phase, a bad-morale freeze and, at a
  town, the turrets' and the Catapult's own acts of round 1 (BAT-SIE-016, BAT-SIE-011). The
  client lane's M4 T2b needs them in the record, as its first message before any command, to
  play the opening's automatic shots (`docs/epics/E17-design.md` §10.2). A record format change
  (the client's reader too: one `Events` message per command today); the client lane's T2b
  records the commanded shots meanwhile. Update (T2b done, 2026-10-23): the Catapult's own shot
  was recorded without it (with the defender commanding its turrets the record opens on the
  keep's turn); the opening's automatic turret shots and the start's other events still wait.

- 2026-10-21 (the adventure lane, E08 T30, decision 65, ADR-0017 point 6 amended in #353; for
  M5, when the server hosts whole games): **a level-up holds the game.** While a hero's level-up
  is open the engine rejects every other actor's command (`GameCommandError::WaitingForLevelUp
  (owner)`) and `h3_game::GameState::waiting_for` names the owner (a battle's defender that won
  answers outside its turn). The user's requirement online: pause the turn timer of the player
  who waits for as long as the level-up is open, give the owner a limited time to answer, then
  answer for it with `h3_game::default_answer` (the original's choice; its network game does the
  same when its timer runs out). Other players learn only that the game waits, never the offers.

- 2026-10-13 (the user, after M6; **with E21**, the web portal): **the operator role.** The user's concern: on a public web surface anyone could try to log in. The lane's proposal for E21's refinement (an ADR then): (1) operator powers come from a random Ed25519 key (`h3-server operator-key`, made as `list-key` makes a list's) that the configuration lists (`[[operator]] key`), never from a password, so there is nothing to guess; (2) each admin request is signed over the connection's challenge and the action after its own purpose (`h3-admin-v1`), checked strictly, never replayable nor relayable to another server; (3) admin requests are accepted only from configured networks, loopback by default (`[admin] from`), so the internet cannot even try (an SSH tunnel or a VPN reaches it); (4) the key stays in the operator's browser as a non-extractable WebCrypto key, and the server checks the handshake's `Origin` for admin requests; (5) every admin action goes to an audit log in the data directory. A player's web login is today's: the browser derives the key from the password (Argon2id), the password never leaves it, the server holds public keys only, failed logins are rate-limited per address.

- 2026-10-13 (the user, after M6; **with E21**): **starting the native client into a match.** The portal hands over the server's address, its pinned key and the game's id (a link such as `h3://play?…`); the client logs in with its own cached account key and joins by account (a seat is its account's since M3), so the link carries no secret another program could steal. A client without a cached key asks for the password once.

- 2026-10-13 (the user, after M6; **later**): signed game results (E16 "Later"): the server's key signs each verified result and its replay's hash.

- 2026-10-13 (the battle lane, E04c T2b; its `cross-lane` pull request): protocol v28 maps the observation's special battlefield and anti-magic flag and lists the battlefields in the key tables (done there). The server draws a hosted scenario's obstacles with `generate_obstacles(terrain)` (`game.rs`); a scenario that names a battlefield needs `h3_battle::generate_setup_obstacles` (none does yet).

- 2026-10-13 (M6 T3; **later**, with 1.0's version rules): a peer that runs another protocol version answers the fetch's hello with `Goodbye { Protocol }` and is skipped, logged; its card says its version, but the server cannot read it. Fetch another version's card once hellos negotiate versions.

- 2026-10-12 (after T8; **later**): a ranked game over whose result did not write (its ladder's log failed) keeps its journal, but its players may play another match on the ladder before the next start rates it, after the newer game. Keep its players out of the ladder until then if logs ever fail in practice.

- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).

- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.

- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load.

- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
