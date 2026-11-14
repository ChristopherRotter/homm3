# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md`; M0 to M4, M6 and M5 part 1 are closed (checkpoints SRV-Q12, -Q16, -Q18, -Q22, -Q24, -Q28, M5 2a's SRV-Q30; M6 is discovery, ADR-0021, detail in `E16-design.md` §12); the done tasks are in `E16-log.md`, the milestone chronicle in `QUEUE-ARCHIVE.md`. Work under the autonomy contract (`HANDOFF.md`). Wire changes bump `PROTOCOL_VERSION`, journal changes `JOURNAL_FORMAT_VERSION` (current values: `HANDOFF.md`). The inbox after M6 was done 2026-10-19 (`E16-log.md`).

**M5 part 1 closed 2026-11-12** (SRV-Q28: the review list as built; the tasks T0 to T7 in `QUEUE-ARCHIVE.md` and `E16-archive.md`, done in full in `E16-log.md`). **M5 part 2 refined 2026-11-12** (SRV-Q29): three increments, 2a whole games over the network (no engine change), 2b time (the server lane builds `h3-game`'s system commands, `cross-lane`), 2c spectators (the server lane builds `Viewer::Spectator`, `cross-lane`) and long journals; each with a reviewer and a checkpoint. **2a's tasks T8 to T13** are in the epic's "M5 part 2" (scope, the inbox notes each takes); T8a (the seats and connections, protocol v45) done 2026-11-12 (`E16-log.md`); T8b done 2026-11-12 (#880 merged); T9 split in two (the user, 2026-11-12): T9a live hosting done 2026-11-12 (protocol v47), T9b the lifecycle done 2026-11-12 (protocol v48); T10 the unowned battle sides done 2026-11-12 (protocol v49, after the `shared/` change #889; the inbox's 2026-11-02 note in `QUEUE-ARCHIVE.md`); T11 the game's queries done 2026-11-12 (protocol v50, record format v5, a battle's targets added by the user; the town lane's two notes of 2026-11-10 in `QUEUE-ARCHIVE.md`); T12 T7b's two findings done 2026-11-12 (protocol v51); T13 2a's closure done 2026-11-13 (SRV-Q30: **2a closed**, its tasks in `E16-archive.md`; the reviewer's major finding into 2b and 2c); 2b refined 2026-11-13 (SRV-Q31: the original's turn duration, the timer running on in battles, a chess timer between two players, the turn timer alone for a disconnected player, the level-up's time an offer's); 2b's T14 `h3-game`'s system commands done 2026-11-13 (`cross-lane`, protocol v56, replay format v14); T16 the clock in the session done 2026-11-13 (protocol v58, SRV-Q32; the inbox's 2026-10-21 note, the level-up hold, in `QUEUE-ARCHIVE.md`); T15 the survey of the original's turn timer done 2026-11-13 (`docs/epics/E16-timer-survey.md`; SRV-Q33: SRV-Q31's battles, the 60 s and `default_answer` kept); T17 the clock hosted done 2026-11-14 (protocol v60: the offer's clock in the lobby); T18 a seat's commands bounded done 2026-11-14 (protocol v62: a turn's map commands and a battle's commands, the user's counter per battle, `cross-lane`); **next: T19, 2b's closure** (the epic's "M5 part 2", T19). SRV-Q19's turn timer (the inbox's 2026-10-12 note) stays for the ranked battles.

The ecosystem talk of 2026-10-13 (E16 "Later", BACKLOG E21 and E22) waits; mDNS for the client lane's browser.

## Inbox

- 2026-11-14 (the adventure lane, E08 T116a, `adventure/e08-t116a`, `cross-lane` with the
  user's consent; **your paths changed, for your review**): the golden adventure replays are 86
  now (fourteen new on `e08-garrisons-36.h3m` and `e08-bonuses-36.h3m`: map garrisons exchanged,
  won, lost, crossed and captured, the fifteen bonus visitables with the Sirens in both
  profiles). The three counts (`tests/game_fold.rs`' `golden`, `tests/game_session.rs`'
  `restores`, `h3-server`'s `tests/record.rs`) say 86; the fold, the session and the records pass
  over all of them, no stub added. `game_fold.rs`' comment above the two map garrison parts ("no
  golden replay holds a map garrison yet") is stale now, yours to reword: both parts agree over
  the garrisons replays.
- 2026-11-14 (the adventure lane, E08 T115b, `adventure/e08-t115b` after `shared/e08-t115b-sirens`
  #909, `cross-lane` with the user's consent; **your paths changed, for your review**): protocol
  **v61** (after your T16's v58, the town lane's T27 v59 and your T17's v60): the adventure events `NothingGained` (with `wire::game::NothingWhy`: `Visited`, `Full`,
  `Emptied`), `SirensLoss`, `CreaturesUpgraded` (the owner's alone) and `KindKnown` (to each
  player of the visitor's team that gained the kind mark); the adventure observation's `kinds`
  (the types whose kind mark the viewer's team holds, in type order) and `Known::Visited` (a
  Fountain of Fortune's or a Magic Spring's team knowledge); the fold: `KindKnown` inserted into
  `kinds`, `SirensLoss` as `WhirlpoolLoss`, `CreaturesUpgraded` sets the slot's creature,
  `NothingGained` nothing (the Sirens' experience comes as the hero's `GainExperience`). No stub
  added or lifted. `h3-server`'s `tests/record.rs`: #909 moved the base faithful `ruleset_hash`
  literal, T115b `army-dialog-s1`'s checkpoint and final hash (adventure state hash v24). The
  gates `tests::game_fold`, `tests::game_session`, `tests::record` pass.
- 2026-11-13 (the town lane, E09 T27, `town/e09-t27`, `cross-lane`; **a protocol version taken,
  for your review**): protocol **v59** (after your T14's v56, E08 T115a's v57 and your T16's v58): `wire::game::TownCommand::Recruit
  { town, building }` (TWN-REC-001, -002), `TownReject`'s `NotRecruiting`, `NotStanding`,
  `NothingSummoned`; `HireTarget::Garrison { town }`; a hire dialog's source as the new
  `wire::game::HireSeen` (`Object { object_type, anchor }`, `Town { town, index }`, `Portal { town
  }`) in `Hire` and `HireOpened`, which no longer carry `object_type` and `anchor` themselves. A
  town's purchase tells the garrison slot's `StackChanged` (`ArmyRef::Garrison`) to the owner,
  then the town's changed views (`TownChanged` and its stacks) before the dialog's close. The
  fold needed nothing new (the decision's view replaced whole; the garrison's slot as a siege's
  merge). Mapped, inverted and round-tripped in `src/tests/`; the golden replays re-recorded
  (hash lines and the format's version only).
- 2026-11-13 (the adventure lane, E08 T115a, `adventure/e08-t115a`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): protocol **v57** (after the town lane's T26b's
  v55 and your E16 M5 T14's v56): the battle bonuses are the hero on the map's, `wire::game::OwnMapHero::bonuses` (`Bonuses { morale, luck, once:
  Vec<VisitBonus> }`, `wire::game::VisitBonus`) and `::timed` (object type handles, sorted);
  `HeroDetails::bonuses` and the changes `HeroChange::Bonus`, `ClearBonuses` are gone; the
  adventure events `BonusChanged`, `OnceMarkChanged`, `TimedMarkChanged` (owner alone, after
  `MovementChanged`, a command's last) mapped both ways and folded; the fold gate's new part
  "battle bonuses and timed marks" (no stub), a fold case in `tests/game_fold.rs`;
  `h3-server`'s `tests/record.rs` holds `army-dialog-s1`'s new checkpoint and final hash (the 72
  replays re-recorded: hash lines only). Nothing for you to build.

- 2026-11-13 (the town lane, E09 T26b, `town/e09-t26b`, `cross-lane`; **a protocol version
  taken**): protocol **v55** (after the adventure lane's T114b's v54): `wire::game::TownBuildings` (the owner's) gains `available`
  (`[u16; 14]`, the counts) and `portal` (`Option<Summoned { creature, count }>`) (TWN-VEW-004,
  TWN-GRO-007); the fold replaces the set whole, so nothing new there. More `TownChanged` are told:
  after the week, the month, a neutral garrison's week, each Portal's draw and a town event's
  creatures. `game_session.rs`' golden finals follow the four re-recorded towns replays.
  Informational.
- 2026-11-13 (the adventure lane, E08 T114b, `adventure/e08-t114b`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): protocol **v54** (after the town lane's v53): `wire::game::MapGarrison`
  (type, anchor, owner, the town's `Garrison` at the viewer's level, `removable` to the owner's
  team; `place()` its anchor's level, row, column, then its type) in `GameObservation::garrisons`
  and `GameReveal::garrisons`, both in their places' order (the engine's are in object order: the
  mapping sorts them); the events `GarrisonChanged { player, object_type, anchor, owner, level,
  slots, removable }` and `GarrisonStackSeen { player, object_type, anchor, stack, slot }`, each to
  its viewer alone, mapped both ways and in the round-trip lists (39 kinds); the fold puts a told
  view in its place (one it did not see yet too, `put_garrison`) and adds its stacks
  (`Garrison::see`, shared with `TownStackSeen`), the reveal's garrisons the same; `town()` maps
  its garrison through the shared `garrison_wire`. The fold gate gains the parts "map garrisons
  in sight" and "map garrisons", without a stub: no golden replay holds a garrison yet (E08 T116
  adds them; a Visions holder's reach, `TWN-STUB-13`, then joins the second part if one shows
  it). No golden replay or record literal moved. **Your review of v52** (2026-11-13): T114b's
  views and events name a map garrison by type and anchor; the exchange keeps the object index
  (`ArmyRef::MapGarrison`, `Decision::Exchange`, `ExchangeOpened`: its owner's alone, the index
  fixed at the setup), as E08 decision 324's agent's call says, so your doc's exception stands.

- 2026-11-12 (the adventure lane, E08 T114a2, `adventure/e08-t114a2`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): protocol **v52** (after your T11's v50 and T12's v51): the wire's
  `ArmyRef::MapGarrison { garrison: u32 }` (a map garrison by object index, `u32::MAX` past it),
  `ArmyReject::{NoGarrison, NotOwn, OnlyEmpty}`, `Decision::Exchange` (the hero, the index, the
  `ObjectRef`, the removable flag, the army) and `GameEvent::ExchangeOpened`, mapped both ways and
  in the round-trip tests; the fold opens the exchange at `ExchangeOpened` and keeps its army from
  the `StackChanged`s that follow (the other viewers' view of a map garrison, `GarrisonChanged`,
  comes with E08 T114b). `map::game::command::slot` and `slot_to_engine` are no longer `const`.
  The golden replays re-recorded (game state hash v29, adventure replay format v13), so
  `h3-server`'s `tests/record.rs` literals moved with them. No golden replay holds a garrison.

- 2026-11-12 (the adventure lane, E08 T113b, `adventure/e08-t113b`; **the answer to your
  question of one type on one anchor**, E16 M5 T7b's review; E08 decisions 320, 322): every
  typed object a command, an offer or an event can name is unique by its type and anchor: the
  setup refuses a second one (`SetupError::SharedAnchor`, `AdventureState::new`'s name check;
  a random dwelling by every outcome it may take). The exceptions are the special grounds, the
  grounds of the sea (Favorable Winds) and the holes, which nothing names (never visited,
  offered, flagged or removed); untyped blockers are in no view. The census of the 160 shipped
  maps: 9 such twins on 5 maps (*Back For Revenge* and its allied version, *Carpe Diem* and its
  allied version, *Deluge*), each a ground or Favorable Winds of two pictures that differ only
  in `drawn`; the user kept them, and since T113b the setup refuses a ground, a ground of the
  sea or a hole that differs from an earlier one of its type and anchor in anything else. So a
  view may hold two `ObjectView`s of one type and anchor (in tile order by masks, as yours) that
  no event names; nothing else. Also: map import v22 and adventure state hash v22 (your
  manifests read the constants); the import types the garrisons (33, 219, with their owners)
  and the fifteen bonus visitables, so shipped maps' views show those nineteen types (no golden
  replay holds one). No server change is needed beyond one made for you, with the user's
  consent (`cross-lane`): the 72 golden replays' hashes moved (the adventure state hash version
  is hashed), so `h3-server`'s `tests/record.rs` has the smallest replay's new checkpoint, final
  hash and import version (`army-dialog-s1`); nothing else of yours changed.

- 2026-11-12 (the town lane, E09 T26a2, `town/e09-t26a2`, `cross-lane`; **a protocol version
  taken**): protocol **v53** is the week's kind (after the adventure lane's v52) (TWN-GRO-002): `wire::game::Week` (`Normal`,
  `Creature`, `Of`), `GameEvent::WeekStarted`'s `kind`, `GameObservation`'s `week`, the fold
  keeping it; the fold gate's part "day and month" is "day, week and month". A bump of yours is
  v54 or the next at merge (the town lane's T26b plans the next for the owner's counts).
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

- 2026-11-10 (the adventure lane, E08 T106a, `adventure/e08-t106a`, `cross-lane` with the
  user's consent; **for your review**): the golden adventure replays are 66 now (T106a's eleven
  teleporters replays: jumps of every kind, `LeadsNowhere`, battles and a meeting at an exit, the
  Whirlpools' losses, revisits). `h3-protocol`'s `tests/game_fold.rs` (`golden`) and
  `tests/game_session.rs` (`restores`), and since E16 M5 T6 `h3-server`'s `tests/record.rs`,
  counted 55 exactly and failed CI; all three say 66 in that pull request, and the fold, the
  session and the records pass over all 66 (no new stub). The adventure lane's next replays (T106b, T106c
  add none planned) bump them the same way; loosen them to a bound if you prefer.
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

- 2026-10-31 (the map client lane, E24 M4 T0; **for E16 M5, the whole game: the user's order**):
  the map viewer shows the engine's whole games through `h3-protocol`, as ADR-0020 point 8 says
  (a record file and a live connection through the same code; an engine replay becomes a record
  by running the session in-process). The user (M4 T0) chose that the server lane builds this
  first, as the start of E16 M5, refined with the user there: the whole game's wire messages
  (the observation projected per viewer, its adventure part, players, kingdom and decisions; the
  events; the commands and answers), a session hosting one game in-process, and its per-viewer
  record, so that a golden adventure replay (`fixtures/replays/adventure/`) becomes a record as
  `h3-view render` makes one of a battle scenario. Seats over the network, timers and the rest of
  M5 can follow. What the viewer reads (E24 M4 T3, T4; `h3_map_view::live` is its model): each
  object by an identity matching the map's object list (the adventure lane's inbox, the same
  day), its type (a random object's as drawn), its owner, a monster's view, increment 9's views;
  the heroes (class, owner, tile, boat), the empty boats, the explored tiles; the viewer's
  kingdom (its heroes with their movement and mana and whether each can move, its towns, the
  resources, the date and the month); a hero's status (primary skills, morale, luck, army); the
  events as the viewer sees them (each step, the reveals, the removals, the gains, the turns,
  days, weeks and months, the decisions opened and answered, the battles with their own battle
  messages, which the battle client's viewer draws). Until then the map client lane works on what
  needs no engine (M4 T1, T2).

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
