# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28**; **M3 (the adventure screen) closed 2026-10-31** (their
scope, decisions, tasks and review lists: `docs/epics/E24-archive.md`; their cuts' history:
`QUEUE-ARCHIVE.md`). **M4 (the engine's games shown) refined 2026-10-31** (M4 T0, with the user:
`docs/epics/E24-map-client.md`, M4's scope, decisions and exit): Part A (T1, T2) the original's
reactions to a game's events, its memory the live part; Part B (T3, T4) the engine's games
through `h3-protocol`'s whole-game record (ADR-0020 point 8), after the server lane builds it;
Part C (T5) the battles handed to the battle client's viewer. Tasks in order, each about half a
context; a task that grows is cut at a seam agreed with the user. Code readings in the sandbox
(`docs/exe/README.md`; every address cited needs a map entry), their detail in
`docs/epics/E24-design.md` (§5 from T1: the game's events); captures by the harness
(`clients/capture/README.md`), never in git. Every oddity of the original a reading or a capture
finds is an `MCL-Q<n>` in the same task (the HANDOFF's autonomy contract).

### E24 M4 T1 — the turn's change

- **Scope:** read and build what the original shows when a turn changes, the live part from its
  memory (`h3mem.py`, additive): End Turn's click (the button's action, its question when a hero
  can still move), the turn's end and the next player's start; the new day (the date's step,
  built at M3 T5b2, from its cause), the week's and the month's start and their messages (the
  week's and month's names are the client's to draw: the engine sends `WeekStarted` and
  `MonthStarted` with the kind only; a comparison reads the original's from its memory); another
  player's turn: the status window's kind 5 (`AdStatNx.pcx`, `crest58.def`, `HourSand.def`,
  `HourGlas.def`, 0x453250; design §4.8) with its tile, hint and cursors (0x40DEB0, §4.10), and
  a computer player's moves where the viewer sees them (the view's scroll to them, their walk at
  the computer's speed, `enemyMoveTime` in VCMI's terms; read, not assumed). New sequences
  recorded in the original (a day's end, a week's and a month's start, a computer player's turn;
  the inbox's harness change first: `setup.sh` again).
- **Acceptance:** the readings in a new `E24-design.md` §5; the new sequences exact on the whole
  screen and `h3map.py drawn` all the same; `clients/scripts/check.sh` all gates; every oddity an
  `MCL-Q<n>`.
- **Depends on:** T0 (done). **Status:** next (likely cut at its start with the user).

### E24 M4 T2 — a visit's results

- **Scope:** read and build what the original shows when a visit or an event changes the game,
  the live part from its memory: the adventure's message box (its dialog: the text through the
  catalogue, the pictures of what was found, OK and Yes/No, its sounds, its place) for the finds
  and questions the engine's visits make (a pile, a chest's or a School's choice, a guard's
  question, an event's or a sign's text, creatures joining, a level-up's choice); the status
  window's kind 6 (a resource found, 0x451220, 5 seconds) and kind 7 (a text, 0x451820); an
  object taken or a monster beaten (the map, the minimap and the panels redrawn); a flag raised
  (the owner's colour on the object and the minimap); a boat boarded and left; a spell cast on
  the map recorded (MCL-Q14: if the stale mana bar shows, the client shows the slot whole, the
  original's behind `--bugs original`). New sequences recorded in the original, the harness
  giving the answers.
- **Acceptance:** as T1's.
- **Depends on:** T0 (independent of T1). **Status:** waiting (likely cut at its start with the
  user, by dialog family).

### E24 M4 T3 — a record's start

- **Scope:** a game's record (the protocol's whole-game messages for one player) read: the
  static art from the map it names; `h3_map_view::live` from its first observation through an
  adapter from the protocol's types (*provisional*): each object by its identity, a random
  object's type, the owners, the heroes (class, owner, tile, boat; their facing from their
  steps), the empty boats, the explored tiles, the kingdom (the heroes' portraits, movement,
  mana and whether each can move, the towns, the resources, the date), a hero's status, the
  hover's facts (increment 9's views: the owners, visited marks, a mine's production); a town's
  own name from the map. MCL-Q15 (b): an amount past the bar's seven digits written so that it
  fits (its form a review item), the original's behind `--bugs original`. MCL-Q21 (b): the
  viewer's load refuses a map the engine's import refuses for it. The check: the engine's start
  drawn as the original's on the maps both play (the writer's without random objects, the 8
  shipped maps that import whole), a situation from each.
- **Acceptance:** the starts equal to the original's; `drawn` all the same; `check.sh` all gates.
- **Depends on:** the server lane's whole-game record (E16 M5's start: its inbox, 2026-10-31);
  the adventure lane's object identity and increment 9's views (its inbox, 2026-10-31; T83b2b,
  T85). **Status:** waiting.

### E24 M4 T4 — a record played

- **Scope:** a record's messages played over time: each event as T1 and T2 built it (the steps
  along the engine's paths as M2's walk, the reveals, the removals, the owners, the turns and
  days, the message boxes with the record's answers, the other players' turns as a computer
  player's); the viewer's controls (pause, step, restart, a speed: the window's keys, new design,
  each a review item). The check: scripted games (the same commands, no random draw between
  them) played in the engine and in the original, compared frame by frame, the original's
  visual moment from its recording; a golden adventure replay as a record viewed to its end, its
  battles skipped to their results until T5.
- **Acceptance:** the scripted games exact; `drawn` all the same; `check.sh` all gates.
- **Depends on:** T1, T2, T3. **Status:** waiting.

### E24 M4 T5 — the battles handed over

- **Scope:** a battle of the game drawn by the battle client's viewer in the map viewer's
  window, as the original switches screens (its sounds and music), and the map back with the
  result applied; the battle's background and colours from the game (CLT-Q4, CLT-Q6:
  `BattleSetup.background`, `colours`, set by `h3-game`).
- **Acceptance:** a golden replay's battles shown in the window; the switch compared with the
  original's recording where it is drawn; `check.sh` all gates.
- **Depends on:** T4; the client lane's library entry (its inbox, 2026-10-31). **Status:**
  waiting.

### E24 M4 T6 — M4's closure

- **Scope:** fresh reviewers (one per area) and their findings fixed or listed; M4's review list
  and its questions for the user; a hand test on the user's machine (`clients/out/win/`: a
  golden replay viewed with its battles; the map's sound by ear on a whole map, the user at M2
  T4b2: the transitions sounded unclean; MCL-Q11's `HORSE20` to `HORSE29` listened to for its
  (c)); M4 archived.
- **Acceptance:** M4's exit (the epic) met; the review list accepted by the user.
- **Depends on:** T1 to T5. **Status:** waiting.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

- 2026-11-11 (the town lane, E09 T19c2, branch `town/e09-t19c2`; **for the town screen's
  builds**, when your viewer offers them): construction is in the game (E09 increment 2). The
  owner builds with `GameCommand::Town(TownCommand::Build { town, building })`, one a town a day
  (TWN-BLD-001 to -007; a refusal is a typed `TownCommandError`, the state unchanged). The hall's
  offer for a client is `GameState::town_offer(rules, player, town)`: `TownOffer { possible,
  buildable, payable }` (the hall's never against not now, now, and within the treasury; the
  owner's town only). Each build, the owner's or a town event's, is told as `GameEvent::TownBuilt
  { player, town, building, event }` (`event`: the town event's place in the town's list, `None`
  for the command), then `TownChanged` to each viewer whose view moved (the owner's buildings;
  everyone's day's mark, fort and Capitol; TWN-VEW-004). The town events fire at their owner's
  turn start (`TownEventFired`; day 1's of the first group in
  `GameState::town_events_at_start`). The golden replays `builds-*` on
  `fixtures/maps/e09-builds-36.h3m` play all of it. Nothing to act on before your town screen
  builds.

- 2026-11-11 (the client lane, E17 M7 T3c, branch `client/e17-m7-t3c`; **for your M4 T5**, and
  two additive changes in the common crates): **the battle's sound on your device, one music for
  both**. Open an `h3_battle_client::audio::HostSound::open(bugs)` (the device and a 44.1 kHz
  stereo music stream played by `h3_ui::music::Queue`), play your effects on its device
  (`audio()`, your own streams) and your tracks through it (`add_track(name, decoded)` once
  decoded: a name not added is waited for; `request(name, once)`, `service(volume)` each pass,
  `focus(on)`, `reset()`); lend it to the battle in `Setup::sound` (it replaced `mute`; none, no
  sound), tell the element each focus change (`Hosted::focus`), and once it closed take it back
  with `Hosted::take_sound()` (the battle's streams removed, the device running). The combat music
  then fades in after your track's fade-out, the end's tune follows the combat music's 900 ms
  fade-out, and your track asked for again fades the tune out and comes back where its fade
  began. Your window's own `Player` would move onto it (its music, decoding and focus are the
  same model). Additive, nothing of yours changed: `h3_ui::music::Music::request_once` and
  `Queue::request_once` (a track played once: from its start at its volume, closed at its end;
  your callers' requests unchanged); `h3_sdl::Audio::remove_stream` (an index freed, reused by
  the next `add_stream`; no index changes while nothing is removed). The battle's frame is
  widened by `Widening` now. Design §13.6; names *provisional*. That ends E17's T3: the entry's
  API is complete for your M4 T5.

- 2026-11-11 (the client lane, E17 M7 T3b2, branch `client/e17-m7-t3b2` after #851, not merged
  yet; **for your M4 T5**): **a whole game's battle played live through your game**. Make a
  `h3_battle_client::game_seat::GameSeat::new(&header)` (your player's `GameRecordHeader`: the
  game's manifest, its id, the player) and hand it your player's frames as they come
  (`receive`, decoded `ServerMessage`s, from the game's start or from the `GameEvents` whose
  `BattleStarted` begins the battle); once `opened()`, `battle::Ground::load_game(&source,
  seat)` (`source`: `packs`, `install`, `speed`, `seed`), then `Prepared`/`Hosted` as before.
  At each pass of your loop, after the element's: `ground.game_seat().unwrap().borrow_mut()`,
  `take_outgoing()`: each `ToHost::Command { seq, command }` you play as the player's
  (`GameSession::play(player, seq, command, ..)`), each `ToHost::Query { seq, query }` you answer
  with `answer(seq, h3_protocol::map::query(battle.state(), rules.battle(), side, query))` from
  `GameState::battle(player)`; play the other side's turns (another player's, or the system
  actor's for a monster), then hand the new frames over. At its end the result window opens; once
  closed, `Hosted::outcome()` and `Ground::in_game()` (now an `Option<InGame>` by value) as
  T3b1's. No Auto Combat switch there yet (`CLT-STUB-4`). The hero's battle picture follows a
  map's sex now (CLT-Q78). Design §13.4; the test host in `game_seat/tests.rs` is a worked
  example; names *provisional*; still to come: T3c (the sound on your device).

- 2026-11-11 (the client lane, E17 M7 T3b1, branch `client/e17-m7-t3b1` after #849, not merged
  yet; **for your M4 T5**): **a whole game's battle from its record**, handed back. Read a
  game record with `h3_view::game_record::parse` (`is_game` tells it from a battle's own);
  `GameRecord::battles()` gives each battle its player saw, in the order they open, as a
  `GameBattle { record, at }`: `record` the battle's own (what a seat's record of it holds,
  numbered afresh), `at: InGame { heroes, tile, opened, ended }` where it stands in the game
  (`opened` the game record's number of the battle's `Snapshot`, where you switch to it;
  `ended` the number of the `GameEvents` holding `BattleEnded` and the result, where you switch
  back and go on folding: that message's adventure events come after the battle's). To play it:
  `battle::Source { input: <the game record's path>, battle: k (from 1), packs, install, .. }`,
  `Ground::load`, then `Prepared`/`Hosted` as before; its result window opens at its end and
  once its OK closed the screen (`Hosted::closed`) `Hosted::outcome()` gives the wire's
  `Outcome` and `Ground::in_game()` the `InGame`. `h3-view window <record> --battle k` plays one
  (`--check 60 --close-result` unattended). Also read now: the battlefield picture's variant
  and each side's colour (CLT-Q4, CLT-Q6); the hero's picture by a map's sex comes with T3b2. The
  battle lacks its own last snapshot (`SRV-STUB-14`: no Eagle Eye picks). Design §13.3; names
  *provisional*; still to come: T3b2 (live), T3c (the sound on your device).

- 2026-11-11 (the client lane, E17 M7 T3a1 and T3a2, branch `claude/ecstatic-thompson-cjowg2`,
  not merged yet; **for your M4 T5**): the battle entry's shape, agreed by the user for both
  lanes at T3's start (your session was not running): **the host owns the window, its loop and
  the frame; the battle is an element of it, drawn natively as any other**, so that a later
  layout (HD's) shows it as a dialog over the adventure screen. Built so far: the crate
  `clients/crates/h3-battle-client` (h3-view's binary's adapters as a library, the `h3-view`
  command line its binary; `h3-view` stays the pure scene model) and in it `window::Prepared`
  (loaded before your window opens: `frame_size`, the first cursor), `Prepared::start(Setup,
  ticks_ns, lines)` and `window::Hosted`, whose pass your loop runs in order: `timeout`, `before`
  (`Flow`), each event to `event` (`Took`: quit, redraw, or fullscreen, vsync and the cursor for
  you to do), `take_moved`, `closed`, `after`, the cursor (`cursor_at`, `cursor_image`), `sound`,
  `due` and `showed`; everything in the battle's frame's coordinates (draw `Frame::screen` at its
  place, move the mouse's events by it). `h3-view window` is its first host (the record:
  `docs/epics/E17-design.md` §13.2). **Still to come**, before your T5 can start: T3b, a battle
  started from a whole game's battle messages and its outcome handed back (the wire's
  background, colours and sex read); T3c, its sound on your audio device and its music through
  `h3_ui::music` (until then it opens a device of its own: do not play beside it). The names are
  *provisional*; tell this lane's inbox what your host needs changed.

- 2026-11-10 (the server lane, E16 M5 T6, branch `claude/vigilant-brown-m2nolc`; **for M4 T3
  and T4: the whole game's record is there**): `h3-server record [--packs-dir packs] [--root .]
  [--out .] [--game 0] <replay.toml>...`, run from the repository's root, runs an adventure replay
  through the in-process game session (`h3_protocol::session::game`; checked to its checkpoints
  and final hash) and writes each player's record to `<out>/<stem>.player-<k>.jsonl`
  (`h3_server::record`). The form (record format v4): JSON lines; the first a
  `wire::game::GameRecordHeader { format, game, player, manifest, first }` (the manifest's `map`
  names the map by its file's content hash, `h3_sim::adventure::replay::map_content_hash`, and
  import version, not by its path: draw it from the client's own copy when both match; its key
  tables give each handle's content key), then one `wire::ServerMessage` per line numbered `n`
  from `first` (1): `GameSnapshot { observation }` first, then after each command
  `GameAccepted` (the player's own), `GameEvents { events }` (every player, also empty), a
  battle's own `Snapshot` when one opens for the player (its events inside `GameEvents`) and
  `GameTurn` while it may act. `wire::game::GameObservation::fold` keeps the view from the
  events; what it cannot fold yet is stubbed (`SRV-STUB-1` the tiles a reveal shows, `-2` to
  `-11`, `-14` a battle's last snapshot; the fold's gate, `h3-protocol`'s
  `tests/game_fold.rs`). The 55 golden replays make 26 MB of records, the largest 1.4 MB.
- 2026-11-10 (the client lane, E17 M6 T7b, branch `client/e17-m6-t7b`; for you to take or
  leave): `clients/crates/h3-map-view/tests/purity.rs` has two more holes (its reviewer's R11-3),
  besides the two of T6a's entry below: `dependencies` reads only the `[dependencies]` table, so
  a `[dependencies.<k>]` or `[target.<cfg>.dependencies]` table goes unchecked, and the deny
  check matches each lint as a substring (`clippy::panic_in_result_fn` passes for
  `clippy::panic`). `h3-view`'s and `h3-ui`'s copies have the fix (`parts`, `dependency`,
  `dependencies`, `names`) and its test lines.

- 2026-11-10 (the adventure lane, E08 T105b2, branch `adventure/e08-t105b2`; **one new command,
  for the space key**): `h3_adventure::AdventureCommand::Revisit { hero }` visits the entered
  object under the hero again (the original's space key, ADV-TEL-008; no points, no step): offer
  the key where `h3_game::legal_commands` lists the hero's `Revisit` (after the `BuildBoat`s);
  its events are the object's visit's, `NothingToRevisit` its rejection. No new event or view.

- 2026-11-09 (the adventure lane, E08 T105a2, branch `adventure/e08-t105a2`; **one new event,
  for the views**): `h3_adventure::AdventureEventKind` gains `WhirlpoolLoss { hero, slot,
  creature, count }`, told before a Whirlpool's jump to the hero's owner alone: the weakest
  stack's slot and creature and how many it lost (0 for a lone creature kept, when the original's
  ADVEVENT 168 shows too; the whole count for a slot emptied); none for a hero the Sea Captain's
  Hat spares. The jump itself is `HeroTeleported`, the hero in its boat. A boat's step onto a
  Whirlpool's tile is now a visit (it was refused). Nothing to act on before E24 shows a turn's
  moves.
- 2026-11-09 (the adventure lane, E08 T105a1, branch `adventure/e08-t105a1`; **two new events,
  for the views**): `h3_adventure::AdventureEventKind` gains `HeroTeleported { hero, from, to,
  seen_from, seen_to }` (a monolith's or a gate's jump: `HeroStepped`'s fields and projection, a
  viewer told an end only where it sees the hero, so `from` or `to` may be `None`; the two tiles are
  not neighbours: draw it as a jump, not a step) and `LeadsNowhere { hero, object_type, anchor }`
  (a one-way exit or a gate without a pair, the hero's owner's alone: the original's ADVEVENT 70
  for a monolith, 153 for a gate, chosen by the object's kind). The Whirlpool's loss event comes
  with T105a2, the battle at an exit with T105b. Nothing to act on before E24 shows a turn's moves.
- 2026-11-09 (the adventure lane, E08 T104, branch `adventure/e08-t104`; **a refusal, for the
  viewer's map list**): the game's build now refuses a teleporter with a visitable tile off its
  surface (`h3_adventure::SetupError::TeleporterSurface(object index)`, A-42, `ADV-QUIRK-44`): a
  monolith's or a Subterranean Gate's on water or rock, a Whirlpool's on land or rock. None of
  the 160 shipped maps is refused; a crafted map the original may load is. The channels and the
  gates' pairs are the static map's (`StaticMap::channels`), the server's alone: never a view's
  (ADR-0016 point 6). Nothing to act on before the jump's events (T105a).
- 2026-11-07 (the adventure lane, E08 T103a, branch `adventure/e08-t103a`; **the teleporters'
  types, for when E24 draws them**): `h3_adventure::ObjectKind` has three new variants,
  `Monolith { channel: u8, way: MonolithWay }` (`Entrance`, `Exit`, `TwoWay`; the original's classes
  43 to 45, the channel its subclass), `SubterraneanGate` (103) and `Whirlpool { divisor: u32 }`
  (111); the packs' types `base:monolith_{entrance,exit,two_way}_0` to `_2`, `sod:…_3` to `_7`,
  `base:subterranean_gate`, `base:whirlpool` (adventure ruleset part v18). The import maps the
  classes to them in T103b (map import v21); until then they import as blockers. No view changes:
  a viewer sees the type as for any object (ADV-FOG-004); a jump's events come with T105a.
- 2026-11-07 (the town lane, E09 T14b, branch `town/e09-t14`; the capture harness, informational):
  `h3adv` plays a walk's `actions` on the town screen when it opens (then closes what they left
  open), with the new actions `hall = true` (a click on the shown hall building, items 10 to 13: the
  hall, vtable 0x6437A0) and `slot = <building id>` (the hall's slot showing it: the building's
  dialog, 0x643944; `click = ["build"]` its Build 30722, `cancel` 30721), and the reads
  `town:x,y:mark`, `hall:slot:<id>`, `build:button`, `build:text`, `screen`, `town_screen_closed`;
  `h3mem`'s town record gains `mark` (+0x2), `h3lod` gains `building_names`. A walk without
  `actions` behaves as before. Learned: the panel's hall picture opens nothing; a map whose players
  are all allies ends at a visit's close (an enemy is needed). The harness check passed after.
  `clients/capture/README.md`, "The hall and the build".
- 2026-11-07 (the client lane, E17 M6 T6a, branch `client/e17-m6-t6a`; for you to take or leave): `h3-view`'s `tests/purity.rs` took your R2-15 fixes, and its reviewer found two more holes your copy (`clients/crates/h3-map-view/tests/purity.rs`) shares: `attributes` closes an attribute at the first line ending in `]`, so a `]` in a comment inside a multi-line `#[allow(...)]` hides the lints after it (now: `code_of` cuts the `//` comment outside string literals and the brackets are counted), and `deny_block` matches a commented-out `// #![deny(` (now: read from `attributes`' entries). `h3-view`'s copy has both fixes and their tests.
- 2026-11-07 (the adventure lane, E08 T100b, branch `adventure/e08-t100b`; the capture harness, informational): `h3adv` gains a `key` step's `repeat = { most, first, until = { with } }` (the key pressed again and again, a row each: `presses`, `presses:<columns>`, `presses:changed`), a recipe's `[places]` (`hero:place`, `hero:at`), a step `look = "<hero>"` (the view centred by the minimap, nobody selected; its `before:` reads before the scroll), `walk = "off"` (the first open neighbour), `only_if`'s `empty` and `occupied`, and the observations `whirlpools` (`h3mem.whirlpools`: game +0x4E77C's vector, begin and end at +0x4E780), `tile:x0,y0..x1,y1`, `tile:from`, `tile:here`, `tile:hero[<name>]`, `answers`, `selected`, `target`. No behaviour of an existing step changes. Observed: the selected hero is lifted off its tile's record after its move and at its turn's start, and a scroll of the view puts it back (a tile read depends on the last scroll). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".
- 2026-11-06 (the adventure lane, E08 T100a, branch `adventure/e08-t100a`; the capture harness, informational): `h3adv` gains a step's `replay` (Replay Opponent Turn: the panel's Adventure Options button, the map dialog's item 9; its dialog vtable 0x63A610, item 5; played on the frozen clock, stepped 20 ms, the flag 0x696A04; `dumps = "every"`) and `at_opening` (reads as a step's first battle opens), `only_if = { at = [x, y] }`, `again_if = { marks = "any" }`, and the observations `replay`, `replay:views`, `replay:watched`, `replay:options`, `view:before`, `view:after`, `view:size`, `explored:x0,y0..x1,y1`, `at_opening:`, `battles`, `exchange`, `army:rows`, `hero:in_boat`. Behaviour changes: a `key` step plays what the key brings by the step's `answer`, `battle` and `level_up` (it stopped at a battle before); a skipped step leaves a field another step filled (it wrote "not played" over it). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".
- 2026-11-06 (the town lane, E09 T8c1 and T8c2, `town/e09-t8c1` (#750, merged) and
  `town/e09-t8c2`; **for the map's towns, when E24 draws them**): the observation carries them.
  `h3_adventure::ObjectView::town: Option<TownId>` links a seen object to its town;
  `h3_game::GameObservation::towns: Vec<TownView>` (in `TownId` order: the towns a tile of which
  the viewer has explored; one only on the fringe keeps its object's `town` but has no view)
  holds the hint's and the right-click popup's facts: `owner`, `faction`, `name` (none: the
  client's default, `h3_towns::TownStatic::name`), `fort`, `capitol`, `garrison: GarrisonView
  { level, stacks, army }` (the popup's level 0 to 3; the stacks packed in slot order, the word
  from level 2, the count at 3, as `quick_view_army` 0x530990 places them; `army` by slot to the
  owner's team only) and `owner_facts: Option<TownOwnerView { hall, income }>` at level 3 (the
  owner's team, or a hero of the viewer's with a Rogue whose Visions reach the town's entrance,
  T8c2). What each viewer sees: `docs/06-security.md`, "Towns"; the popup: survey
  `E09-towns.md` §12.3; design note `E09-design.md` §12, §17.

- 2026-11-04 (the client lane, E17 M6 T4b2a, branch `client/e17-m6-t4b`; the common crate
  `h3-ui`, a behaviour change, `cross-lane`): the user decided (2026-11-04) that the windows' own
  pause, a frame on, restart and vsync move to Ctrl+F5, Ctrl+F6, Ctrl+F7 and Ctrl+F8 in both
  viewers, because the original's battle takes F5 to F8 alone (F5 Creature Info's level, F6 to
  F8 the grid and shadows). `h3_ui::keys::Window` gains `ctrl` and `chord` (the default chord;
  `key` stays the key's name); `h3_map_view::keys`' defaults take `w.chord()`, its tests and the
  usage name the new chords (built on that branch). The music, the effects, the cursor's kind,
  fullscreen and Esc are unchanged; a user's `keys.toml` still overrides any. Informational
  unless `docs/epics/E24-design.md` §3's keys bullet (F5 pause, F6 the next frame, F7 restart, F8
  vsync) should say so: the map lane's file.

- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **your M4 T0 request (2), for the random dwellings**): a random dwelling not tied to a town (216 to 218) resolves when the game is built (the setup stream, before every other draw of the static map): its `ObjectView::object_type` is the drawn dwelling type, its `anchor` the original's moved one (the type's shift: only the Angel Portal's, 1; it may lie one column past the map's right edge, x == width, where only its free cell is) and `blocked`/`visitable` the type's footprint, its `owner` the map's; nothing tells a viewer it was random. One tied to a town stays an untyped blocker until E09 (ADV-STUB-13).
- 2026-11-03 (the adventure lane, E08 T90, branch `adventure/e08-t90`; the capture harness, informational): `h3adv` and `h3mem` gain the dwellings' records (`dwelling:x,y`, game +0x4E39C), `teams`, `objects:` and `access:` (a rectangle's map objects with class and subclass, its tiles' blocked and visitable bits), `camp:x,y`, the hire dialog as a screen (vtable 0x640C4C; `hire`, `hire:opening`, `hire:after`; actions `click = ["max", "recruit"]`, `amount = <n>`), a step's `hint = [x, y]` and `popup = [x, y]`, `opening:defender`, `ai_turns`. Behaviour change: `end_turn` now waits out a computer player's turn (it returned during it); nothing changes on maps with humans only. `clients/capture/README.md`, "Dwellings, the hire dialog and a computer player's turn".

- 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
  unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
  `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
  `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
  30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
  in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
  its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
  side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
  --map` fills the template beside the map when its stamp matches (no cargo build). A town's
  visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
  after. `clients/capture/README.md`, "Towns (E09 T3b)".
- 2026-11-02 (the battle lane, E26 T5; for a computer side in your battles, when you add one):
  call the fair system actor `h3_battle_ai::choose_fair` by default (ADR-0027 point 4 as
  amended; the user, AI-Q10), the all-seeing `choose` only as a game's option.

- 2026-11-02 (the client lane, E17 M6 T2a1; for information, nothing to do): additions to the
  common crates for the battle's stack stats dialog (`docs/epics/E17-design.md` §12.1):
  `h3_render::blit::draw_frame_clipped`; `h3_ui::ui::Def`'s `group`, `creature` and `frames_of`
  (serde defaults: a layout without them reads as before) and the
  `ui::Draw` variants `Creature` (a creature DEF item, the flags 0x12: the shadowed drawer, its
  canvas placed by group 2's first frame, cut to the item) and `Shared` (another DEF's frame in
  the item's DEF's colours: the executable keeps frames by name, CLT-Q36), both drawn by
  `h3_ui::paint` (`names`, `draw`, `reach`; `creature_frame`). Every existing draw is the same;
  a `match` on `ui::Draw` needs the two arms.

- 2026-11-02 (the battle lane; for your information): `h3_formats::text::TextTable` now makes
  each run of quotes in a quoted field one, as the executable's text-table readers do
  (`installation.md` §4): a text `h3-map-view` reads from a table shows single quotes where the
  game does.

- 2026-10-31 (the adventure lane, E08 T83b2b; **for E24 M4 T3**, the "increment 9's views" it
  waits on): `ObjectView` carries `owner` (every viewer that sees the object), `production` (a
  mine's resource and daily amount, its owner's team only), `guards` (`GuardsView`: `creature`
  when all stacks are of one, none for the original's generic text; `count_word` of the total
  count, every stack counted; `army`, the stacks, for the owner's team only), `visited` (a weekly
  object's mark, the viewer's own) and `boat_tile` (a Shipyard's); it is no longer `Copy`. The
  object identity, the random objects' drawn types and MCL-Q21 (b) of your note stay open in the
  adventure lane's inbox; the month's kind comes with T85.

- 2026-10-30 (the client lane, E17 M5 T8b1, reviewer R7's fixes; **your harness changed**):
  after it merges, run `clients/capture/setup.sh` again for every run directory you use: the
  hook DLL's ack has ten fields now, and an older `xdd.dll` stops a run with "stale xdd.dll".
  `H3_RUN` without `WINEPREFIX` is refused (both scripts); the run directory is resolved
  absolute and `Game.start` fails unless the prefix's `d:` points at it. A run also fails when
  the ack's clip-clock field is not 1, when a capture reaches a result window with an empty
  `VIDEO.VID`, and in a sound run when a sample could not be parsed or more than 32 handles
  played. Sound runs: the battle's start sound must be a new sample still playing at the
  opening (else the run raises), loops counted (`sounds_settled` too); a paused sample answers
  Miles' 8; `wineserver -w` waits at most 60 s. Nothing else drives the original otherwise
  (runbook, "Sound", "The clips", "Traps").

- 2026-10-23 (the client lane, E17 M3 T9c, at the split): moved here from the client lane's
  inbox, the adventure lane's entry of 2026-10-21 (E08 T30, decision 65, ADR-0017 point 6 amended
  in #353; for the hot-seat client, E24 M5): **a level-up holds the game.**
  While a hero's level-up with two offers is open, the engine accepts no command but its owner's
  answer (`h3_game::GameState::waiting_for` names the owner; `level_up(owner)`, or the owner's
  `GameObservation.decision` as `DecisionView::LevelUp`, gives the hero, level, primary skill and
  the two offers; `Answer::First`/`Second` answer it). A battle's defender that won answers it
  outside its turn. The user's requirement for hot seat: when the player who must act changes,
  show a handover screen that hides everything (map, hero list, the dialog) until that player
  confirms it is in control, then the level-up dialog alone, and hand back to the player whose
  turn runs the same way, so no one sees another player's view.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
