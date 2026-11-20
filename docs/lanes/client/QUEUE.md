# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E27, the game shell** (refined 2026-11-12, M0, with the user: `docs/epics/E27-game-shell.md`,
decisions 4 to 8, CLT-Q79 to CLT-Q83; the clips: `E27-design.md` §1). M1 closed 2026-11-13 (T3),
M2 2026-11-16 (T8), M3 2026-11-19 (T7); M4 (around play, outside it) below, refined at M3 T7
(decision 69); M5 (in play) is refined once the shell hosts play (E24 M5). The original files:
stage them from the user's `original-data/install/` (one large file per staging call; the user's
computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local check;
CI's job runs the same), the real-asset gate `cargo run -p h3-battle-client -- gate` from
`clients/`.

#### E27 M4 T3

T3 the save file (the epic's M4 table; decision 69, CLT-Q156 (a)): the engine's adventure replay
(`h3-files`' format, `h3_sim::adventure::Replay`) with the setup's choices (per colour the human
flag, the town's `FactionChoice`, the starting hero's pick, as `begin::setup` sets them) and the
client's own table (the names, bonuses, handicaps, the turn's duration, the save's name), written
and read with its version; a load's setup rebuilt and its commands replayed by the engine; tests,
no screen. **First:** the adventure lane's inbox still holds the format extension (2026-11-19,
not built): ask the user whether this lane builds it cross-lane (with the `cross-lane` label) or
waits; offer a seam if it is big (the format, then the client's table and the load).

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-11-19 (the map client lane, E24 M4 T2d3, `map-client/e24-m4-t2d3`; **the common capture
  harness, informational, nothing to do**): `h3mem.Mem.message_box` now gives none while the spell
  book's dialog is on top (vtable 0x641DCC, `Mem.spell_book`), as it gives none under the level-up
  dialog; `Mem.boats` entries carry their record `index`; new `Mem.mana`, `Mem.modal_answer`. The
  rest is `h3map.py`'s (a map sequence's book and `[[cast]]` table; a sound run lets the clock run
  before its settling). The runbook's "A cast on the map".

- 2026-11-19 (the adventure lane, E08 T128, `adventure/e08-t128`, `cross-lane` with the user's
  consent; **one line in your file, nothing to do**): `h3-battle-client`'s
  `bin/h3-view/situation.rs` writer `Map` literal gains `banned_artifacts: Vec::new()`
  (`h3_map::writer::Map::banned_artifacts`, E08 decision 375: the map's bans beside the editor's
  default ones).

- 2026-11-18 (the map client lane, E24 M4 T2a2, `map-client/e24-m4-t2a2`, `cross-lane` with the
  user's consent; **one word in your test, nothing to do**): the common catalogue
  (`clients/text/keys.toml`) gains ADVEVENT.TXT's records (the map's finds' texts, `map.find.*`),
  so `h3-battle-client`'s `text::tests::the_committed_catalogue_parses` lists `"ADVEVENT"` among
  its tables now (and its comment says why). Also common and additive: `h3_ui::message` gains
  your `h3_view::message`'s pictures (`Picture`, `pictures`, `slot_ids`, `Rows`, `Slot`, copied
  with their tests) and `told`, a style-1 box of up to three pictures; `h3-view` is unchanged.

- 2026-11-17 (the map client lane, E24 M4 T1b1, `map-client/e24-m4-t1b1`; **the capture harness,
  rebuild `xdd.dll`**): `xdd.c` takes `<seq> seed <value>` (its
  8.: the main thread's C library seed at its next clock read; `Game.seed`,
  `h3drive.to_scenario(..., seed=)`), and its acknowledgement has a sixteenth field (the seeds
  set): `h3drive.ACK_FIELDS` is 16, so a run with an `xdd.dll` built before fails as stale where it
  reads the acknowledgement whole (a forced roll, a batch's or a sequence's checks, a seed):
  rebuild it with `clients/capture/setup.sh` (`h3shell.py`'s recipes
  run under `H3FROZEN` and need no seed). `h3mem`'s `status_items` also returns kind 5's items
  (2190 to 2193), `Mem.other_turn` is new; `h3map.py` alone gained the rest.

- 2026-11-17 (the town lane, E09 T37a, `town/e09-t37a`, `cross-lane`; **your test literal, and
  for the shell's town screens later**): protocol **v70** adds `GameObservation::tavern` (the
  viewer's two tavern heroes as the tavern's held hero screen shows them: `TavernHero`, its head
  `TavernHeroHead`, its lists) and the events `TavernOffered` and `TavernPartSeen`. `h3-view`'s
  `game_record/tests.rs` literal gained `tavern: [None, None]`.

- 2026-11-17 (the map client lane, E24 M4 T1a, `map-client/e24-m4-t1a`; **two additions to the
  common crates, nothing of yours changed**): (1) `h3_ui::message`, a copy of your
  `h3_view::message` trimmed to styles 1 and 2 without pictures (`layout`, `Open`; an origin
  comment; E24 design §5.1), for the adventure map's message boxes; `h3-view` is untouched: switch
  to it when you like (the map lane adds the pictures at its M4 T2), or tell this lane if you want
  it shaped otherwise. (2) The capture harness: `xdd.dll`'s table forces two more draws
  (`week_name` 0x4C87A6, `month_name` 0x4C8FCC; `h3drive.ROLLS` grew by two), so a run with an
  `xdd.dll` built before fails its count of redirected calls: rebuild it with
  `clients/capture/setup.sh`; `h3map.py` alone gained the rest (a sequence's `move_reminder`,
  `rolls`, Enter and Escape, the `date` and `player` lines, the live part's `[[turn]]`).

- 2026-11-15 (the battle lane's inbox session, `battle/inbox-2026-11-15`, `cross-lane`; **three of
  your waiting items**): (1) **G1 / `CLT-STUB-1`:** the observation gives each side's
  `forbids_casting` (the Orb of Inhibition's wearer) and `spell_level_limit` (Recanter's Cloak),
  public; on the wire as `SideView`'s two fields (protocol **v69**). `h3-view`'s test literal in
  `record.rs` took them (`false`, `None`): the stub can be lifted. (2) **CLT-Q69 (a):** the
  engine's split is built (`BattleState::setup`, `begin`, `has_begun`; `start` unchanged); the
  server lane's half (the session's hold, the start marked on the wire) is next. (3) **CLT-Q63
  (a):** handing the automatic war machines' turns to their side changes a seam; its design is
  the user's choice, BAT-Q65 in `docs/OPEN-QUESTIONS.md`.

- 2026-11-15 (the server lane, E16 M5 T24, `claude/adoring-fermi-l0mvur`, `cross-lane`; **a
  protocol version taken; for when a seat of yours joins a whole game**): protocol **v68**
  (after the town lane's v67; SRV-Q38, 2c's reviewer: a seat watched without agreeing).
  `GameJoined` gains `spectators` (whether the game may be watched, as the lobby lists it:
  taking the seat agrees to it), and the new numbered `GameSpectators { n, open }` tells
  every player but the host the host's `SetSpectators` before the start (the host still gets
  `SpectatorsSet`), so a seat that does not agree may leave before the start: show both if you
  show the lobby's flag. On a server that requires logins a `GameWatch` now needs one
  (`NotLoggedIn`), as a battle's spectator's `Join`. Nothing of yours breaks (your matches
  have wildcards); `GameSpectators` takes its number in a seat's record.

- 2026-11-14 (the server lane, E16 M5 T21a and T22a, `claude/festive-albattani-mdvj1a`,
  `cross-lane`; **a protocol version taken; for when you show a whole game to someone who does
  not play it**): protocol **v66**. The user (SRV-Q36): a whole game's spectator sees **every
  player's view**, live, only in a game its creator opened before the start (closed by
  default), never a ranked one. On the wire: `CreateMapGame`'s `spectators: Option<bool>` (left
  out: closed), `GameWatch { game }` answered `GameWatching { game, manifest }`, then each
  player's messages wrapped as `GameWatched { n, player, message }` (`n` the spectator's own
  number, `message` exactly as that player's seat got it, with the player's own `n`); a watch
  starts with each player's `GameSnapshot`, its `GameClock` and its `GameTurn` as a re-taken
  seat gets them, numbered as the player's last message. A client folds each player's stream
  as it folds a seat's (`GameObservation::fold`), one view per player. `SetSpectators` works
  in a whole game (the host's, until the start: `GameReject::NotHost`, `AlreadyStarted`), the
  lobby's `MapGameInfo::spectators` says whether a game takes them. `h3-server` hosts them
  since T22b (2026-11-14, still v66: a paused game is listed taking none and refuses a watch
  `NoSpectators`; a spectator hears the pause, a void and the stop as the seats do). Nothing to build until you want a spectator's screen. **Your path
  changed:** `h3-battle-client`'s `game_seat/tests.rs` literal took `spectators:
  Spectators::NONE` (`GameOptions`' new field).
- 2026-11-14 (the server lane, E16 M5 T20, `claude/sweet-gauss-ngra43`; **a protocol version
  taken**): protocol **v65** (after the town lane's E09 T28a's v64): a whole game is void when the server's own
  commands in one of its battles reach `[limits] system_commands_per_battle` (10 000) with the
  battle still open (a battle both of whose sides the server's computer plays, which it did not
  end; SRV-Q35). Its connections hear `Goodbye { CommandLimit }` (until now a battle's alone), its
  journal goes to the void, it leaves the lobby; an in-process `GameSession` that is void
  (`GameSession::void`) rejects every command `GameReject::NotStarted {}`. No new message or
  field. Show the goodbye's reason for a whole game too if you show it; nothing else to build.
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
