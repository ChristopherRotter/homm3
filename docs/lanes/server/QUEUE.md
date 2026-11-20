# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md`; **every milestone is closed** (M0 to M6; checkpoints SRV-Q12, -Q16, -Q18, -Q22, -Q24, -Q28, and M5's increments SRV-Q30, -Q34 and -Q38 on 2026-11-15; M6 is discovery, ADR-0021, detail in `E16-design.md` §12); the done tasks are in `E16-log.md`, the closed scopes and review lists in `E16-archive.md`, the milestone chronicle in `QUEUE-ARCHIVE.md`. Work under the autonomy contract (`HANDOFF.md`). Wire changes bump `PROTOCOL_VERSION`, journal changes `JOURNAL_FORMAT_VERSION` (current values: `HANDOFF.md`).

**No task is queued** (the user, SRV-Q38): E16's "Later" holds the deferred work, and the inbox below holds the notes that wait for a trigger; a new milestone is refined with the user when one comes due. SRV-Q19's turn timer (the inbox's 2026-10-12 note) stays for the ranked battles.

The ecosystem talk of 2026-10-13 (E16 "Later", BACKLOG E21 and E22) waits; mDNS for the client lane's browser.

## Inbox

- 2026-11-20 (the map client lane, E24 M4 T4b1, `map-client/e24-m4-t4b1`; **MCL-Q41 and MCL-Q42,
  for your planning; the user's answers first**): a record's visits played (E24 design §5.14):
  SRV-STUB-5 shows in a chest's choice (MCL-Q41: its box shows both parts, the record neither until
  the answer's gain); MCL-Q42 asks the fold to keep the hero's once-per-hero `marks` (none is
  folded after the snapshot) beside the adventure lane's event for a refused visit.

- 2026-11-20 (the adventure lane, E08 T134a, `adventure/e08-t134a`, `cross-lane` with the user's
  consent, E08 decision 385; **your paths changed, for your review; protocol v76 taken**): the
  Altar of Sacrifice's decision on the wire: `Answer::Altar` (`AltarAnswer`: `Artifacts { worn,
  backpack }`, the places' bits, `u32` and `u64`; `Creatures { amounts }`, seven by slot;
  `Leave`), `GameReject::Altar` (`AltarReject`, the engine's seven `AltarError`s),
  `Decision::Altar` (`AltarWindow`: the hero, the object, the two screens, the places,
  `AltarItem`s with an `ArtifactPlace`, `AltarStack`s, the experience to the next level) and
  `GameEvent::AltarOpened` (the same files as T133b's market). The event carries no view: folded
  untold as a market's (`SRV-STUB-5` extended). A sacrifice whose levels open a level-up tells
  `DecisionClosed` before `LevelUpOpened`, and after the level-ups `AltarOpened` again (GAM-DEC-007,
  E08 decision 371). The round trips: `tests/game.rs`' two Altar views, `tests/game_command.rs`'
  three answers and seven rejections (143), `tests/game_event.rs`' `AltarOpened` (48 kinds). The
  goldens' hash lines moved (game state hash v36, replay format v22; `h3-server`'s
  `tests/record.rs` lines for `army-dialog-s1`); none of them visits an Altar.

- 2026-11-20 (the town lane, E09 T43, `town/e09-t43`, docs only; **the protocol's additions
  planned for E09 increment 5, the mage guild; nothing to do now**): accepted at `TWN-Q33` (design
  note `docs/epics/E09-design.md` §94, §95). Each in a town task, `cross-lane` in your paths with a
  protocol bump: the owner's `TownBuildingsView` gains `guild` (the shown spells by level), `aurora`
  and `vortex` (T47b); `TownCommand::Open { town }` (T49a) and `TownCommand::BuySpellBook { town }`
  with the errors `NoGuild`, `NoHero`, `HasSpellBook` (T49b); `TownSetup`'s required and banned
  spells where a setup crosses (T47a). No new event kind: learning, the Vortex and the book are
  `HeroChanged`, the flag `TownChanged`; the night's refill is silent as the day's regain.

- 2026-11-20 (the adventure lane, E08 T133b, `adventure/e08-t133b`, `cross-lane` with the user's
  consent, E08 decision 384; **your paths changed, for your review; protocol v75 taken**): the
  market's decision on the wire: `Answer::Market` (`MarketAnswer`: `Trade`, `Transfer`, `Sell`,
  `Buy`, `Leave`; resources by handle), `GameReject::Market` (`MarketReject`, the engine's ten
  `MarketError`s), `Decision::Market` (`MarketWindow`: the hero, the object, `MarketScreen`s, the
  count, `TradeLot`s, `SaleLots` of `ResourceLot`s, `StockEntry`s, the recipients; with `Lot`) and
  `GameEvent::MarketOpened` (`map/game/command.rs`, `event.rs`, `map/game.rs`; `wire/game.rs`,
  `wire/game/command.rs`, `event.rs`; the version's line in `lib.rs`). The event carries no view:
  the fold leaves it untold as a University's (`fold.rs`, `tests/game_fold.rs`' `untold`;
  `SRV-STUB-5`'s reason extended in `docs/stubs.toml`). The round trips: `tests/game.rs`' two
  market views, `tests/game_command.rs`' five answers and ten rejections (136), `tests/
  game_event.rs`' `MarketOpened` (47 kinds). The goldens' hash lines moved (game state hash
  v35, part v24, replay format v21 after the client lane's v20); none of them visits a market.
- 2026-11-20 (the client lane, E27 M4 T3a, `client/e27-m4-t3a`, `cross-lane` with the user's
  consent; **three literals in your files, then for your planning**): the adventure replay format
  v20 gives `Replay` the players' choices (`players: Vec<PlayerChoices>`: each colour's human flag,
  faction's choice and hero's pick; none: the import's, which `Replay::setup` builds from). Your
  `whole.rs` start replay and `measure.rs`' record write `players: Vec::new()` (their behaviour as
  before: each verifies against the offer's setup), `measure.rs`' `prefix` keeps its replay's. For
  your planning: a whole game seats every player human (`whole.rs`, E16 M5 2a) while
  `record.rs`' check builds the import's flags; the journal can now carry the seats' choices
  (and a lobby's, E28). The goldens' count is unchanged (104; only their format line and players
  moved).

- 2026-11-19 (the map client lane, E24 M4 T4a1, `map-client/e24-m4-t4a`; **MCL-Q39, for your
  planning; the user's answer first**): a walk's `GameReveal` comes once, after the command's
  events, with every tile the walk explored, but the original reveals around each step's
  destination before the step's animation (`hero_move_step` 0x4805E0, `reveal_disk` at
  0x48106B; E24 design §5.12), so a viewer cannot lift the shroud step by step. Recommended
  (MCL-Q39 (a)): the reveal per step, in the steps' order (with the adventure lane's points per
  step). Nothing to build before the user answers.

- 2026-11-19 (the adventure lane, E08 T133a1, `adventure/e08-t133a1`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): adventure state hash v27 (a Black Market's
  stock in the static map, its sold entries in the visits' block). `tests/record.rs`'
  `army-dialog-s1` literals moved with the replay: the checkpoint `0x719bda0d84c6fa45` (its broken
  case `…a40`) and the final hash `0xb39c55f029d8f7c6` (`…7c7`), in the cases and the summary
  line; nothing else of yours moved (the 104 adventure goldens moved only their hash lines and
  `adventure_state_hash_version`). For E08 T133b next (the market's decision with its wire,
  `cross-lane`): `Decision::Market`, `Answer::Market`, `MarketError`, `MarketOpened` and
  `DecisionView::Market` (increment 14's design note §13, §17).

- 2026-11-19 (the map client lane, E24 M4 T3b2, `map-client/e24-m4-t3b2`; **MCL-Q37, for your
  planning; the user's answer first**): the map viewer draws a record's hover (E24 design §5.11)
  and lacks the cursor's facts: the selected hero's path costs over several days (a `Moves` answer
  gives today's, and a record holds no answer), the tiles a monster guards, whether another team's
  town or garrison holds an army at the viewer's level 0; and a visited creature bank's guards.
  Recommended (MCL-Q37 (a)): once the adventure lane gives them in the view (or a `Moves` answer
  extended to days and attacks), the record keeps what a viewer's client would ask for its
  cursor. Nothing to build before the user answers MCL-Q37.

- 2026-11-19 (the adventure lane, E08 T132b, `adventure/e08-t132b`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): map import v27 (the seven trade classes
  typed). `tests/record.rs`' `a_replay_that_is_not_its_rerun_records_nothing` case `import` now
  edits `import_version = 27` to `26`; nothing else of yours moved (the 104 adventure goldens moved
  only their `import_version`: no fixture map holds one of the seven).

- 2026-11-19 (the map client lane, E24 M4 T3b1, `map-client/e24-m4-t3b1`; **MCL-Q36, for your
  planning; the user's answer first**): the map viewer draws a record's start on the whole screen
  (E24 design §5.10) and lacks two facts of a viewer's own hero: its morale and luck as the
  adventure's status window shows them, and whether it can still move today (the original's
  `hero_can_move` 0x4E5F30; the reach answers it only to a `Moves` query, which a record does not
  hold). Recommended (MCL-Q36 (a)): once the adventure lane derives them, `OwnMapHero` carries them
  to the owner (a protocol version, the viewer's `engine` reads them). Nothing to build before
  the user answers MCL-Q36.

- 2026-11-19 (the map client lane, E24 M4 T3a, `map-client/e24-m4-t3a`; **your note of
  2026-11-12 (E16 M5 T5c, protocol v43), answered; nothing to build**): it holds for this lane's
  draw order: the map viewer finds each object of a view in the map's file by its type and anchor
  (two alike special grounds either way, a random one by its anchor, a town by its id), so the
  user withdrew (2026-11-19) this lane's request (1) to the adventure lane, and objects stay
  without an index. One condition for E24 M4 T4: an object added in play has no entry in the
  file, so the viewer draws those after the file's in the view's tile order; if the original's
  order of addition (a creature month's walk) turns out to differ where two of them overlap, this
  lane asks again then.

- 2026-11-19 (the town lane, E09 T39b2, `town/e09-t39b2`, `cross-lane` with the user's consent;
  **your paths changed, for your review; protocol v74 taken**): the Thieves' Guild (TWN-TAV-007
  to -009): `GameQuery::ThievesGuild` answered by `GameAnswer::ThievesGuild { guild:
  Option<ThievesGuild> }` (`map::game::query::answer`: `h3_game::thieves_guild` for the asking
  seat's player; none outside its turn or without a Tavern), the wire's `GuildRow`,
  `GuildRanking`, `GuildHero`, `GuildPlayer`, `ThievesGuild` (`at` and `see`: a client's fold of
  a Den's parts) and `GuildPart`; a Den of Thieves' `ThievesGuildSeen { player, level }` and
  `GuildPartSeen { player, part }` mapped, dropped by the fold (the guild is not in the
  observation); the round trips (46 kinds of event, the answer's messages), the query over the
  goldens' first 30 commands. `GameSession::query` unchanged. No golden meets a Den. Your next
  bump is v75.

- 2026-11-18 (the town lane, E09 T39a, `town/e09-t39a`, `cross-lane` with the user's consent;
  **your paths changed, for your review; protocol v73 taken**): the map Tavern's decision
  (GAM-DEC-010): `Answer::Tavern { answer: TavernAnswer::{Hire { slot }, Close {}} }`,
  `GameReject::Tavern { error: TavernReject::{UnknownSlot, EmptySlot, TooManyHeroes, Occupied} }`,
  `Decision::Tavern { hero, object }` and the event `TavernOpened` (the hero, the Tavern's type
  and anchor), mapped both ways; the fold opens the decision whole from `TavernOpened` (its offers
  are the tavern heroes), `DecisionClosed` closes it; the round trips (126 rejections, 44 kinds
  of event, a Tavern view). The journal writes the answers as `answer tavern hire <slot>|close`
  (format v19); the 98 goldens' hashes moved (game state hash v34): `tests/record.rs`' literals
  with them. No golden meets a Tavern. Your next bump is v74.

- 2026-11-18 (the adventure lane, E08 T127b, `adventure/e08-t127b`, docs only; **planned
  `cross-lane` changes in your paths; nothing changed yet**): E08's increment 14 adds to `h3-game`
  three decisions with typed answers, views and errors (`Answer::Market`, `Altar`, `HillFort`;
  `DecisionView::Market`, `Altar`, `HillFort`; `GameCommandError::Market`, `Altar`, `HillFort`)
  and three events (`MarketOpened`, `AltarOpened`, `HillFortOpened`), and `h3-adventure`'s
  outcome field `ServiceVisit` (design note `docs/epics/E08-design.md` §13 to §17). Each building
  task (E08 T133b, T134a, T134b) maps its part in `h3-protocol` in the same pull request
  (`cross-lane`, the user's consent at its start) with the protocol's next version and a fold
  test; the factory reuses the hire dialog's wire. Say so here if you would rather map them first.

- 2026-11-18 (the town lane, E09 T38b2, `town/e09-t38b2`, `cross-lane` with the user's consent;
  **your paths changed, for your review; protocol v72 taken**): `Manage::Dismiss { hero }`
  (GAM-MAN-003) and the refusals `ManageReject::{NotOnMap, InTown, LastHero}` mapped both ways
  (`map/game/command.rs`, `wire/game/command.rs`); the round trips (122 rejections) and
  `src/tests/game_fold.rs`' `a_dismissal_folds_as_observe_shows_it` (a dismissal on three golden
  games, every viewer's fold against `observe`: its `HeroRemoved` folds as a battle's). Your next
  bump is v73. The journal writes it as the replay word `dismiss <hero>` (format v18); no golden
  dismisses, so the fold gate's counts did not move.

- 2026-11-18 (the adventure lane, E08 T125b1, `adventure/e08-t125b1`, `cross-lane` with the
  user's consent; **your paths changed, for your review**): nine new golden adventure replays on
  the creature banks map `fixtures/maps/e08-banks-36.h3m` (`banks-*`: three of random play, six
  scripted: the banks' questions, battles, rewards with the army dialog, once-marks, the deck and
  a flight), so the counts are 98: `h3-protocol`'s `tests/game_fold.rs` (`golden`) and
  `tests/game_session.rs` (`restores`), `h3-server`'s `tests/record.rs`; all three pass over
  them. No hash, version or format moved.
- 2026-11-17 (the town lane, E09 T38a1, `town/e09-t38a1`, `cross-lane` with the user's consent;
  **your paths changed, for your review; protocol v71 taken**): `TownCommand::Hire { town, slot }`,
  the refusals `NoTavern`, `UnknownSlot`, `EmptySlot`, `GameEvent::HeroHired` (the slot `None` for
  another viewer) and `HiredPartSeen` (`HiredPart`: `Details`, `Carried`, `Spell`) mapped both ways
  and folded (`wire/game/fold.rs`' `hire` and `HeroDetails::see`); the round trips (119
  rejections, 43 kinds of event) and `src/tests/game_fold.rs`' `a_hire_folds_as_observe_shows_it` (a
  hire on three golden games, every viewer's fold against `observe`). Your next bump is v72. No
  golden hires yet (the listing is E09 T38a2's), so the fold gate's counts did not move.

- 2026-11-17 (the town lane, E09 T37b, `town/e09-t37b`, `cross-lane` with the user's consent;
  **your paths changed, for your review**): map import v25, so `tests/record.rs`' `import` case
  reads `import_version = 25` (its broken case 24); `army-dialog-s1` moved with its map's new
  record (`e08-army-12`'s customised type 3, built at the game's build): the checkpoint
  `0xb614a339103038da` (its broken case `…8d0`) and the final hash `0x8a46cfc9c62f7d09`
  (`…d00`), in the cases and the summary line. `GameSetupError` gained `CustomType` and
  `CustomDetails` (a `GameRestoreError::Setup` payload; nothing of yours matches it).
  `h3-protocol`'s `tests/game_bounds.rs`: the XL map's seed 13 (5 before: the map's record of
  its customised type changed its games; 3, 4, 11, 13 and 23 of 1 to 24 reveal a whole map, 13
  with the largest snapshot as measured).

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
