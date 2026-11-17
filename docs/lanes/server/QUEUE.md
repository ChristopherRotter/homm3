# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md`; **every milestone is closed** (M0 to M6; checkpoints SRV-Q12, -Q16, -Q18, -Q22, -Q24, -Q28, and M5's increments SRV-Q30, -Q34 and -Q38 on 2026-11-15; M6 is discovery, ADR-0021, detail in `E16-design.md` §12); the done tasks are in `E16-log.md`, the closed scopes and review lists in `E16-archive.md`, the milestone chronicle in `QUEUE-ARCHIVE.md`. Work under the autonomy contract (`HANDOFF.md`). Wire changes bump `PROTOCOL_VERSION`, journal changes `JOURNAL_FORMAT_VERSION` (current values: `HANDOFF.md`).

**No task is queued** (the user, SRV-Q38): E16's "Later" holds the deferred work, and the inbox below holds the notes that wait for a trigger; a new milestone is refined with the user when one comes due. SRV-Q19's turn timer (the inbox's 2026-10-12 note) stays for the ranked battles.

The ecosystem talk of 2026-10-13 (E16 "Later", BACKLOG E21 and E22) waits; mDNS for the client lane's browser.

## Inbox

- 2026-11-17 (the adventure lane, E08 T124b1, `adventure/e08-t124b1`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): adventure state hash v26 (the creature banks'
  draw in the static map). `tests/record.rs`' `army-dialog-s1` literals moved with the replay:
  the checkpoint `0x0e5362e1ae4496f8` (its broken case `…6f0`) and the final hash
  `0x396cfaa7f922de73` (`…e70`), in the cases and the summary line (after your T37a's v33 ones); nothing else of yours moved
  (the 89 adventure goldens moved only their hash lines and `adventure_state_hash_version`).

- 2026-11-17 (the town lane, E09 T37a, `town/e09-t37a`, `cross-lane` with the user's consent;
  **your paths changed, for your review**): **protocol v70**: `GameObservation::tavern` (the
  viewer's two tavern heroes, `TavernHero { hero: TavernHeroHead, army, skills, artifacts }`: the
  tavern's held hero screen, no learned spells, no offered levels, the backpack's first five places
  and its count; E09 decision 134), `GameEvent::TavernOffered { player, slot, hero }` then
  `TavernPartSeen { player, slot, part }` per stack, skill and artifact (decision 135), mapped and
  folded (`wire/game/fold.rs`, `TavernHero::see`). The fold gate gained the part **tavern heroes**
  (exact) and its **spell points** part covers the tavern views' too (`SRV-STUB-6`: a day's regain
  tells nothing). `tests/record.rs`' `army-dialog-s1` checkpoint and final hash moved (game state
  hash v33). For T38a: a hire's owner learns the hired hero's spells and offered levels from no
  tavern view; its events will tell them (design §75's As built).

- 2026-11-16 (the adventure lane, E08 T123b, `adventure/e08-t123b`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): map import v24 (the creature banks typed).
  `tests/record.rs`' `a_replay_that_is_not_its_rerun_records_nothing` case `import` now edits
  `import_version = 24` to `23`; nothing else of yours moved (the 89 adventure goldens moved only
  their `import_version`: no fixture map holds a bank, so no checkpoint or final hash changed).

- 2026-11-16 (the town lane, E09 T33b, `town/e09-t33b`, docs only; **planned `cross-lane` changes
  in your paths; nothing changed yet**): E09 increment 4 (the Tavern; `TWN-Q18`, decision 119;
  `docs/epics/E09-design.md` §75, §76) adds to the protocol, each at its task with your consent:
  `GameObservation::tavern` (the viewer's two tavern heroes: id, type, details, army, formation,
  tactics) and `TavernOffered` (T37a); `TownCommand::Hire` and `HeroHired` (the hero, its type,
  owner and tile to every viewer of the tile, the slot to the owner: no `SRV-STUB-3` gap) (T38a);
  `Manage::Dismiss` (T38b); `DecisionView::Tavern`, `TavernOpened` (whole: no `SRV-STUB-5` gap) and
  the tavern answers (T39a); a request for the Thieves' Guild answered from the current state for
  the asking seat, and a Den of Thieves' `ThievesGuildSeen` (T39b). The fold gate should need no
  new exemption.

- 2026-11-15 (the town lane, E09 T31, `town/e09-t31`, `cross-lane` with the user's consent, E09
  decision 101; **your question of 2026-11-14 answered, your paths touched, no version moved**):
  the user chose the projection. `GarrisonEntered` and `GarrisonLeft` carry `named: Viewers`
  (the viewers that see the town's garrison hero: its owner's team and a viewer whose view of the
  town is at TWN-VEW-003's level 3, `h3-game`'s `town::view::named`); `AdventureEvent::project`
  tells any other viewer of the entrance `HeroRemoved` at the entrance in place of
  `GarrisonEntered`, and `HeroStepped { from: None, to: Some(entrance) }` (a hero coming into
  sight, SRV-Q26 (c), still `SRV-STUB-3` in your fold) in place of `GarrisonLeft`. In your paths:
  `map/game/event.rs` ignores `named` (a projected event names the viewer alone); in
  `tests/game_event.rs` the map back sets `named` to the viewer alone and `every_event` has each
  event named to both players and to player 0 alone. The wire, the fold and the protocol version
  are unchanged; the fold gate passes on the 55 replays.
- 2026-11-15 (the town lane, E09 T31's sandbox; **a test that fails under load, for your
  TRAPS**): `h3-server`'s `tests::clock::a_spectator_watches_a_timed_game_through_a_restart_to_a_stalled_battles_void`
  failed once (`clock.rs:801`, a `Timer` unwrapped on `None`) in the setup's coverage run on
  `master` at db8406b, two CPUs busy with another build; alone it passes in 9 s, and CI on that
  commit is green. Likely a wall-clock margin in the instrumented build. **Again on CI**
  (2026-11-17, the town lane's #980, the coverage step: `clock.rs:798`, player 0's last watched
  clock `turn: None` against the told `Timer { left_ms: 3000, running: true }`); it passed on the
  re-run of the failed job.

- 2026-11-15 (the battle lane's inbox session, `battle/inbox-2026-11-15`, `cross-lane`; **for you
  when you take CLT-Q69 (a)**): (1) **the engine's half of CLT-Q69 (a) is built.**
  `BattleState::setup(ruleset, setup, seed, sink)` sets a battle up (the obstacles, the armies, a
  town's hidden land mines, reported to its sink); `begin(&mut self, ruleset, sink)` plays the
  start (the tactics phase or the first activation, with everything before the first command;
  a second call does nothing); `has_begun()` tells them apart. `start` is the two in a row, with
  the same events. A battle only set up refuses every command with `NotYourTurn` and is left
  unchanged; its observation has no active stack and no result. The session's hold, the wire's
  mark of the start and the record's step are yours (your 2026-11-10 entry). (2) **G1's wire part
  is done** in the same pull request (your 2026-11-05 entry, to archive): `SideView`'s
  `forbids_casting` and `spell_level_limit`, mapped from the observation's new arrays; protocol
  **v69** (after your v68).

- 2026-11-10 (the client lane, E17 M6 T7c; **the user's answers, for you when you take them**):
  (1) **CLT-Q69 (a)** with T7b's R12-3: once the battle lane splits `BattleState::start` into the
  setup and its first activation (its inbox), the session holds the battle before its start
  until every seat is held: a join's `Snapshot` before the game starts shows the battle before
  the start, the start's `Events` then plays on it (a late joiner's snapshot after it, as
  today), and the start is marked on the wire, so a reader need not find it by the kind of its
  first event (`h3_view::record::opens` does now); a protocol step and a record format step. (2)
  T7b's R6-2 (low priority): the army info windows list a stack's active spells in the
  observation's order (by spell handle), the original in cast order (the stack's +0x420): the
  cast order on the wire, a field or the list's order (the battle lane's first if the engine
  keeps no order).

- 2026-11-05 (the client lane, E17 M6 T5a, gap G1; **the user's requirement**: a server game
  behaves exactly as a local one; after the battle lane's `SideView` fields): **carry each side's casting limits on the wire** (whether its hero
  forbids casting, its spell level limit; public, as the original's book names the Orb's wearer
  and the Cloak's limit to both sides, `docs/epics/E17-design.md` §12.17). The battle lane's
  inbox holds the request for the observation; map it once it is there (a version). The client
  reads it then (`CLT-STUB-1`).

- 2026-11-04 (the client lane, E17 M6 T4b2a, CLT-Q46; **the user's request**, when online play
  needs it): **the attack and shot previews on a seat's wire.** With Creature Info on, the battle
  client's status line shows an attack's or a shot's damage range (`docs/epics/E17-design.md`
  §12.14); a local battle takes it from `h3_battle::preview` (BAT-DMG-011), but a server's seat
  holds no battle state (ADR-0018 point 10), so it shows the hint without the damage. Send the
  acting seat, with each turn's legal commands, the preview (least and greatest damage) of each
  attack and shot it may give (a protocol version step; the client lane then reads it).

- 2026-10-13 (the user, after M6; **with E21**, the web portal): **the operator role.** The user's concern: on a public web surface anyone could try to log in. The lane's proposal for E21's refinement (an ADR then): (1) operator powers come from a random Ed25519 key (`h3-server operator-key`, made as `list-key` makes a list's) that the configuration lists (`[[operator]] key`), never from a password, so there is nothing to guess; (2) each admin request is signed over the connection's challenge and the action after its own purpose (`h3-admin-v1`), checked strictly, never replayable nor relayable to another server; (3) admin requests are accepted only from configured networks, loopback by default (`[admin] from`), so the internet cannot even try (an SSH tunnel or a VPN reaches it); (4) the key stays in the operator's browser as a non-extractable WebCrypto key, and the server checks the handshake's `Origin` for admin requests; (5) every admin action goes to an audit log in the data directory. A player's web login is today's: the browser derives the key from the password (Argon2id), the password never leaves it, the server holds public keys only, failed logins are rate-limited per address.

- 2026-10-13 (the user, after M6; **with E21**): **starting the native client into a match.** The portal hands over the server's address, its pinned key and the game's id (a link such as `h3://play?…`); the client logs in with its own cached account key and joins by account (a seat is its account's since M3), so the link carries no secret another program could steal. A client without a cached key asks for the password once.

- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
