# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E27, the game shell** (refined 2026-11-12, M0, with the user: `docs/epics/E27-game-shell.md`,
decisions 4 to 8, CLT-Q79 to CLT-Q83; the clips: `E27-design.md` §1). M1 closed 2026-11-13 (T3);
M2 (the start and the menus) below, refined at M1 T3; M3 and M4 are refined at their T0s. The original files: stage them from the user's
`original-data/install/` (one large file per staging call; the user's computer must be online);
never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd`
and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates:
`clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate
`cargo run -p h3-battle-client -- gate` from `clients/`.

#### E27 M2 T3 — The shell's frame

- **Scope:** the epic's M2 table, T3 (CLT-Q103, decision 20, CLT-Q2): the state machine and the
  menu loop's requests in `h3-view`, `h3-view shell` with a headless text driver, the dialogs'
  input moved from `h3_view::options` into `h3-ui` (the combat options' tests unchanged; `h3-ui`
  is common: the map-client lane's inbox told of the addition), start-up loading by the LODs'
  tables.
- **Status:** ready.

#### E27 M2 T4 to T8

T4 the comparison (`h3shell.py drawn`, h3shell's sound, TCP/IP's address masked; after T3); T5 the
start (after T4; every pixel of a Smacker clip through its palette, decision 24, every Bink frame
whole, decision 28, the comparisons through `h3_bink::Target`; the clips'
sound at the effects setting, decision 27, through `HostSound::play_clip`; how much of a Smacker
clip's queued sound plays at its end, design §8.4); T6 the main menu, New Game's and Load Game's menus, Quit (after T4); T7
Multiplayer and the hot seat's names (after T6); T8 M2's closure and M3's refinement (after T7).
Their scopes: the epic's M2 table; what each still reads or records: design §6.4.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-11-14 (the town lane, E09 T28a, `town/e09-t28a`, `cross-lane` with the user's consent;
  **your path changed, informational**): protocol v64's `wire::game::Adventure` gains
  `garrison_heroes` (a player's own garrison heroes, `wire::game::GarrisonHero`), so
  `h3-view`'s `game_record/tests.rs` literal took `garrison_heroes: Vec::new()`. New wire
  variants a client may meet: `TownCommand::Heroes`, eight `TownReject`s, the adventure events
  `GarrisonEntered` and `GarrisonLeft`.

- 2026-11-14 (the server lane, E16 M5 T19, `claude/gracious-maxwell-gu6rjb`; **a protocol
  version taken**): protocol **v63** (after T18's v62), 2b's reviewer's findings:
  `GameReject::WaitingForLevelUp {}` no longer names whose level-up holds the game (SRV-Q31:
  show only that the game waits, as `GameClock`'s `waits`); the lobby's `wire::game::MapGameInfo`
  gains `clock` (`OfferClock`, the game's own: the offer's at its creation, which a restart keeps
  whatever the offer's is now; show it for a listed game rather than its offer's); a seat whose
  turn or battle side's chess clock ran out gets no battle commands in its `GameTurn` (the
  computer plays the side at once); the join that starts a game now tells every player its clock
  as it runs (before, the first turn's timer looked stopped until the next reading). Nothing else
  to build.
- 2026-11-14 (the server lane, E16 M5 T18, `claude/awesome-sagan-lue93w`, `cross-lane` with the
  user's consent; **a protocol
  version taken**): protocol **v62** (after the adventure lane's T115b's v61): the server bounds a whole game's seat's commands
  (`session::game::CommandBounds`, `GameOptions::commands`; `[limits] commands_per_turn`, 1 000,
  and `commands_per_battle`, 4 000). Past its turn's bound a seat's map commands (an adventure's,
  a management one, a town's) are rejected `GameReject::TurnLimit {}`, while ending the turn,
  answering a decision and battle commands stay accepted; past its battle's bound the server's
  computer plays its side for the rest of the battle (the system's `SystemCommand::Battle`
  events, as for a side whose chess clock ran out) and its own battle commands are rejected
  `GameReject::BattleLimit {}`. `GameTurn` leaves out what a bound refuses (no `GameTurn` when
  nothing is left). Show the two rejections if you show rejections; nothing else to build.
  **Your path changed:** `h3-battle-client`'s `src/game_seat/tests.rs` builds its `GameOptions`
  with `commands: CommandBounds::NONE` (the in-process host vouches for its commands).

- 2026-11-14 (the adventure lane, E08 T115b, `adventure/e08-t115b`, #914, `cross-lane`; **your path
  changed, for your review**): protocol v61 adds the adventure observation's `kinds` (the kind
  marks of the viewer's team), so `h3-view`'s `game_record/tests.rs` snapshot literal gained
  `kinds: Vec::new()`; also new: `Known::Visited` and the adventure events `NothingGained`,
  `SirensLoss`, `CreaturesUpgraded`, `KindKnown` (your readers' wildcards skip them).
- 2026-11-14 (the server lane, E16 M5 T17, `claude/admiring-lovelace-jlv0hu`; **a protocol
  version taken**): protocol **v60** (after the town lane's T27's v59): a map offer shows how its games are timed,
  `wire::game::MapOffer`'s `clock` (`wire::game::OfferClock`: `turn_ms`, none for unlimited, an
  offer's one of the original's 1, 2, 4, 6, 8, 10, 15, 20, 25 or 30 minutes; `level_up_ms`, none
  when the game waits for a level-up; `battle_ms` and `increment_ms`, each side's chess clock in a
  battle between two players' heroes under a limited turn). The server's whole games are timed
  by their offer now (`[[map]] turn_minutes`, 60 s a level-up by default), so `GameClock` is told
  with running timers; T16's note above stands. Nothing to build unless your lobby shows the
  offer's clock (it decodes the new field with the crate).

- 2026-11-13 (the server lane, E16 M5 T16, `claude/charming-edison-866oew`; **a protocol version
  taken**): protocol **v58** (after the adventure lane's T115a's v57), the whole game's clock: `ServerMessage::GameClock { n, clock }`
  (`wire::game::GameClock`: the player's own `turn` timer, its battle's chess clocks `battle`
  against another player's hero (`Duel`'s attacker and defender), its own `level_up` time, and
  `waits`, the game waiting for another player's level-up), each `Timer` the milliseconds left
  when sent and whether it runs (count it down locally; a battle side at 0 is the server's
  computer's for the rest of the battle). Numbered and recorded: after a command's other
  messages and before the `GameTurn`s, alone when a timer ran out, and after a `GameSnapshot`
  that leaves something to tell. In an untimed game (every game until the server lane's T17)
  only `waits` is ever told. At a timer's end the server acts for the player with the system
  actor's commands (T14's `GameCommand::System`), told as every command's. Your readers'
  wildcards skip it; showing it is yours (E28's screens). **Your path changed** (`cross-lane`,
  the user's consent): `GameOptions` gained `clock`, so `h3-battle-client`'s
  `game_seat/tests.rs` builds its options with `clock: Clock::UNLIMITED` (one line and its
  import).

- 2026-11-13 (the adventure lane, E08 T115a, `adventure/e08-t115a`; **CLT-Q40's amounts, and the
  wire**): a hero's battle bonuses reach the battle by source: the morale and luck no once-mark
  names (a Pandora's Box's, an event's) as `EffectSource::Temporary`, each once-mark's own
  amounts as `EffectSource::Visit { mark }` (the Warrior's Tomb's mark 0: −3 morale; a looted
  Pyramid's 1: −2 luck; increment 12's from 2 with T115b), the original's +0x105 lines. On the
  wire (protocol v57) the bonuses left `HeroDetails` for `OwnMapHero::bonuses` (with each mark's
  amounts) and `::timed` (the Magic Well's and the Stables' marks), changed by the adventure
  events `BonusChanged`, `OnceMarkChanged` and `TimedMarkChanged`. No client literal moved.

- 2026-11-13 (the adventure lane, E08 T114b, `adventure/e08-t114b`, `cross-lane` with the user's
  consent; **your test changed, nothing to build**): protocol **v54**: the wire's
  `GameObservation` and `GameReveal` gain `garrisons` (`wire::game::MapGarrison`, the map
  garrisons a player sees), so `h3-view`'s `game_record/tests.rs` literal gained `garrisons:
  Vec::new()`; the events `GarrisonChanged` and `GarrisonStackSeen` are new (a record reader that
  matches every event kind needs their arms).

- 2026-11-12 (the server lane, E16 M5 T12, branch `claude/confident-knuth-subyc4`; **check
  your readers, nothing to build**): protocol **v51**. A whole game's fresh `GameSnapshot` (past
  half a record's bound) now comes inside a battle too, holding the battle's view in
  `observation.battle`, with no battle `Snapshot` after it: `h3_view::game_record` already reads
  it as a later snapshot of the open battle (R3-1); a live seat must not take it as a new battle
  or a reconnect. A battle's own `Snapshot` opens only a battle the player does not hold yet
  (unchanged on every reachable path).
- 2026-11-12 (the server lane, E16 M5 T11, branch `claude/vibrant-albattani-ml282o`; **for online
  play, nothing to do before it**): protocol **v50**, the game's queries: a seat sends `GameQuery {
  seq, query }` and gets `GameAnswer { n, seq, answer }`, unnumbered (`n` the number of its last
  message) and not recorded, so your record readers never meet it; a battle inside a whole game asks
  its cast targets as `GameQuery::Battle { query }` (the battle's `Query`), answered
  `GameAnswer::Battle { targets }`, as a battle seat's `QueryResult` (CLT-Q46's previews are still
  the battle lane's and yours, later). A whole game's start now tells a player its town events of
  the build (day 1's of the first group) in one `GameEvents` after its `GameSnapshot`, before its
  turn (record format **v5**).
- 2026-11-12 (the server lane, E16 M5 T7a1, branch `claude/keen-tesla-q5uxid`;
  informational for `h3_view::game_record`): protocol **v44**: a game's player may get a fresh
  `GameSnapshot` after a command's messages (before its `GameTurn`, never while a battle is open
  in its view), which replaces its view; a bounded record begins with its newest kept snapshot.
  `GameReveal`'s tiles are packed (a palette and per level a rectangle); your reader's wildcard
  still skips it.

- 2026-11-12 (the server lane, E16 M5 T5c, branch `claude/optimistic-meitner-j73r89`; **your
  rule followed, your docs to update**): a game's battle's last `Snapshot` is sent as your note
  of 2026-11-12 asked (protocol v43): the player's very next message after the `GameEvents`
  holding `BattleEnded`, none when those events open another battle for it; `SRV-STUB-14` is
  lifted, so `h3_view::game_record`'s doc ("the game sends no last snapshot yet") is stale. A new
  message `GameReveal` (what the player newly sees on the map) comes after it when the player's
  explored tiles grew; your reader's wildcard skips it.

- 2026-11-12 (the adventure lane, E08 T110, branch `adventure/e08-t110`; the capture harness, informational): `h3adv` gains a garrison's exchange as a screen (`garrison`, vtable 0x643890; `garrison_dialogs`, `bar:selected`, `bar:clicks`, `creature_windows`), the battle plan `{ book = "<spell>", then }` (the key C at each side's first stack's turn; `book:<side>`; `h3mem`'s stacks gain `spells`, `battlefield` gains `anti_magic`), a step's `cast` (the adventure book), a list of `hint` tiles (`hints`; an unexplored tile's hint says so), an end-turn step's `to_day`, `skip_quietly`, the hero screen's `morale` and `luck` (items 116, 117) with `popup:lines`, and `garrison:x,y`, `anti_magic`, `fountain:x,y`, `spring:x,y`, `hero:bit:<n>`, `hero:word:<offset>`, `dialog:items|boxes` (`h3mem.garrisons`, `battle_hero`). **Behaviour changed:** `h3drive.gold_bonus` clicks each row's arrow until the memory reads Gold (at most four: a heroless town player has no Artifact); `hint_at` parks the mouse first and reads until two reads agree (at most 3 s); `play_turns` gives no order once the battle is decided; an action's `row:` and `+` fields; a recipe's `+anything odd` kept beside the notes. The harness check passed. `clients/capture/README.md`, "Garrisons and the bonus visitables".

**E17's leftovers** (placed at its closure, M7 T4, with the user): each waits for the work named;
the battle client's code is this lane's, so the lane builds it then.

- **With E24 M4 T5** (the map client's battles; its inbox told, 2026-11-12): M6 T5e, the captured
  artifacts' message after Eagle Eye's (deferred by the user, 2026-11-07;
  `captured_artifacts_message` 0x4772B0, design §12.21: GENRLTXT 31, eight artifact pictures a
  box through `h3_view::message::pictures`, a pickup sound each), its list the adventure's
  capture (HER-ART-004) told to the winner's owner, a server's seat with it; CLT-Q4, the
  background variants 0 and 1 (read from the wire since M7 T3b1; recorded once a battle has trees,
  mountains or dead vegetation around it; NOTES, "Battlefields and boats"); `CLT-STUB-4` (a live
  game's battle takes no Auto Combat switch: the computer's choices for the human's side need the
  host's state).
- **With E24 M5** (hot seat): `CLT-STUB-5`, a hot-seat battle between two of the host's humans
  (design §13.4).
- **Waiting on other lanes** (their inboxes, 2026-11-05 and 2026-11-10): CLT-Q63 (a) (the
  automatic war machines' turns, the battle lane), CLT-Q65 (b) (a whole game's learned spells),
  CLT-Q69 (a) (the start split, the battle and server lanes), `CLT-STUB-1` (a server's seat
  lacks the other hero's Orb and Cloak: the battle and server lanes); the client's side follows
  each.
- **The battle screen's unbuilt gaps** (read in part, not recorded; NOTES, "The battle screen"):
  the other pending spells of a melee strike, regeneration, the mana drain, Death Blow and
  Poison's damage, the Faerie Dragon's draw line, Fire Wall's burn and Land Mine's blast, the
  shot cursor's half damage, the "no free hex" message (an engine event first); taken when a
  recording needs one.
- **E28** (the backlog): ranked play in the native client, refined after E27 (the first ranked
  play through the website, E21); NOTES, "The live client". **E29** (the backlog): HD layouts and
  graphics options for both clients; NOTES, "Options and layouts".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
