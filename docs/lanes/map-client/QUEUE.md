# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28**; **M3 (the adventure screen) closed 2026-10-31** (their
scope, decisions, tasks and review lists: `docs/epics/E24-archive.md`; their cuts' history:
`QUEUE-ARCHIVE.md`). **M4 (the engine's games shown) refined 2026-10-31** (M4 T0, with the user:
`docs/epics/E24-map-client.md`, M4's scope, decisions and exit): Part A (T1a, T1b, T2) the original's
reactions to a game's events, its memory the live part (**done** 2026-11-19 with T2d3: their
entries in `QUEUE-ARCHIVE.md`); Part B (T3, T4) the engine's games
through `h3-protocol`'s whole-game record (ADR-0020 point 8), after the server lane builds it;
Part C (T5) the battles handed to the battle client's viewer. Tasks in order, each about half a
context; a task that grows is cut at a seam agreed with the user. Code readings in the sandbox
(`docs/exe/README.md`; every address cited needs a map entry), their detail in
`docs/epics/E24-design.md` (§5 from T1: the game's events); captures by the harness
(`clients/capture/README.md`), never in git. Every oddity of the original a reading or a capture
finds is an `MCL-Q<n>` in the same task (the HANDOFF's autonomy contract).

T3b, the rest of a record's start through T3a's adapter, was cut in two by the user at its start
(2026-11-19): the panels first (T3b1, **done**: design §5.10; MCL-Q36 asked), then T3b2 (**done**
2026-11-19, kept whole by the user: the log's "M4 T3b2", design §5.11; MCL-Q37, MCL-Q38 asked;
their entries in `QUEUE-ARCHIVE.md`). A fact the record lacks becomes an `MCL-Q<n>` with a note in
the engine lane's inbox, its gap shown in the comparison, never a rule copied into the client (the
user, the same day).

### E24 M4 T4 — a record played (cut in three by the user at its start, 2026-11-19)

T4 cut by event family, as Part A was: **T4a** the viewer's own turns (its walks along the
engine's paths, End Turn and the turn's change), **T4b** the visits, **T4c** the other players,
the viewer's controls and a golden replay. T4a cut again at its start (the agreed fallback):
**T4a1** the reading and the recordings (**done** 2026-11-19: the log's "M4 T4a1", design §5.12;
MCL-Q39, MCL-Q40), then T4a2 (**done** 2026-11-20, kept whole by the user: the log's "M4
T4a2", design §5.13; its entry in `QUEUE-ARCHIVE.md`). T4b (**done** 2026-11-20, in two by
family: the log's "M4 T4b1" and "M4 T4b2", design §5.14 and §5.15; its entry in
`QUEUE-ARCHIVE.md`). T4c (**done** 2026-11-20, kept whole: the log's "M4 T4c", design §5.16,
§5.17) and its remainder T4c2 (**done** 2026-11-21, whole: the log's "M4 T4c2", design §5.18;
its entry in `QUEUE-ARCHIVE.md`). A fact the record lacks becomes an `MCL-Q<n>` with a note in
the engine lanes' inboxes, its gap shown in the comparison, never a rule copied into the client.

### E24 M4 T5 — the battles handed over

- **Scope:** a battle of the game drawn by the battle client's viewer in the map viewer's
  window, as the original switches screens (its sounds and music), and the map back with the
  result applied (a monster beaten: taken off the map, the hero's army and experience, a guarded
  mine's flag, design §5.6; moved here from T2d by the user, 2026-11-18); the battle's background and colours from the game (CLT-Q4, CLT-Q6:
  `BattleSetup.background`, `colours`, set by `h3-game`).
- **Acceptance:** a golden replay's battles shown in the window; the switch compared with the
  original's recording where it is drawn; `check.sh` all gates.
- **Depends on:** T4 (done with T4c2, 2026-11-21); the client lane's library entry (its inbox,
  2026-10-31: complete, E17 M7 T3a to T3c). **Status:** cut in two by the user at its start
  (2026-11-21, the reading first): **T5a done** (the reading and the recordings: the log's "M4
  T5a", design §5.19; the harness's battle step; `battle-won`, `battle-mine`, `battle-retreat` in
  `map/to-record.toml`; MCL-Q47); **T5b next**: the viewer (`h3-map-view`'s window hosting
  `h3-battle-client`'s element, the map's sound on `HostSound`; a record's battles played,
  `GameRecord::battles()`, and the map after them as §5.19 reads it; T5a's three moved into
  `sequences.toml` and compared; a golden replay's battles in the window; `check.sh`). Large:
  confirm its cut at its start (seam: the window's hosting first, the switch and the result
  compared second); its first question, the crate structure (a dependency on the client lane's
  crate or a binary over both libraries, M4 T0's decision), for the user then.

### E24 M4 T6 — M4's closure

- **Scope:** fresh reviewers (one per area) and their findings fixed or listed; M4's review list
  and its questions for the user; a hand test on the user's machine (`clients/out/win/`: a
  golden replay viewed with its battles; the map's sound by ear on a whole map, the user at M2
  T4b2: the transitions sounded unclean; MCL-Q11's `HORSE20` to `HORSE29` listened to for its
  (c)); M4 archived.
- **Acceptance:** M4's exit (the epic) met; the review list accepted by the user.
- **Depends on:** T1a to T5. **Status:** waiting.

## Inbox

- 2026-11-20 (the adventure lane, E08 T134a, `adventure/e08-t134a`, `cross-lane`; **built, for
  your view's fold; nothing to do now**): the Altar of Sacrifice is entered and its visit sets the
  team's kind mark (k 28: `KindKnown`), then opens `DecisionView::Altar` (`AltarView`: the screens,
  the places, each artifact's and stack's experience, the experience to the next level) with
  `GameEvent::AltarOpened`; a sacrifice tells `HeroChanged` (each removal, then the experience) or
  `StackChanged`, and when its levels open a level-up `DecisionClosed`, `LevelUpOpened`, then
  after the level-ups `AltarOpened` again (decision 371, your note of 2026-11-18 below).
  `h3-map`'s `tests/view.rs` folds `Open::Altar` beside `Open::Market`.

- 2026-11-20 (the town lane, E09 T44, `town/e09-t44`, `cross-lane`; **informational, nothing to
  do**): `h3_map::writer::Map` gains `banned_spells: Vec<u8>` (the header's banned spells); the
  two `Map { .. }` literals of `h3-map-view` (`situation.rs`, `art/tests.rs`) got `banned_spells:
  Vec::new()` in T44's pull request. The capture harness's town screen (common): a walk and a
  `town` step pass a message that comes over the screen before their actions; `hall = true`
  hovers the hall's box until the status line names it (`clients/capture/README.md`, "Towns").

- 2026-11-20 (this lane, the labels task; **the user's to decide, then a small task**): the
  viewer's tests' own catalogues (`h3-map-view`'s `finds/tests.rs`, `asks/tests.rs`,
  `status/tests.rs`) imitate the catalogue's texts, and a few are the original's sentences or
  their starts (a sign's default text, RANDSIGN 5; a pile's, ADVEVENT 113; a bank's, ADVEVENT 32;
  a luck reward's, ADVEVENT's; `levels.rs`'s doc comment quotes GENRLTXT 446's words). The tests
  compare the viewer's composition, so words of our own serve as well (ADR-0009); found by the
  labels task's scan, outside its `clients/map/`.

- 2026-11-19 (the client lane, E27 M3 T7 (M4 T0), `client/e27-m3-t7`; **informational, nothing to
  do**): E27's dialogs in play (the system options and the menu in play, Save from play, Restart,
  the end of a scenario's boxes) move to a new **E27 M5**, refined once your E24 M5 hosts play
  (decision 69, CLT-Q155 (a)); E27 M4 builds High Scores, Credits, the save file and Load Game
  meanwhile. The note of 2026-11-16 below (the options at E27 M4's T0) reads M5 now.

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-11-19 (the client lane, E27 M3 T6, `client/e27-m3-t6`; **for E24 M5, nothing to do now**):
  the scenario screen's Begin now builds the game's setup and the engine validates it
  (`h3-battle-client`'s `begin`: the map imported again, the advanced options' records into each
  player's `human`, `start.choice` and `start.pick`, the difficulty; `GameSetup::validate`), and
  keeps it as `begin::Game` (the file, the `GameSetup`, and beside it what the setup has no field
  for: the players' names by colour, the bonuses and handicaps, E10's, and the turn's duration,
  the hot seat's clock). The shell then shows the loading screen and its fade
  (`h3_view::shell::play`) and a notice (`CLT-STUB-13`) where the original's play starts:
  `Screen::Play { restart: false }`, the shell's window's `game` (`h3-battle-client`'s
  `shell::in_window`). E24 M5 takes the game from there (no seed is drawn yet: the build's).

- 2026-11-18 (the adventure lane, E08 T127b, `adventure/e08-t127b`, docs only; **the trades and
  services, for your planning; nothing to do now**): E08's increment 14 (design note
  `docs/epics/E08-design.md`) types the seven classes (2, 7, 35, 99 and 221, 106, 213), all
  **entered** (the hero stands on the entrance; T132b), the Altar's and the Hill Fort's kind marks
  (the original's k 28, k 14) for your hints (T134), and three windows as decisions
  (`DecisionView::Market`, `Altar`, `HillFort`; T133b, T134). **The user's note for the Altar**
  (E08 decision 371): when a sacrifice opens a level-up, the engine closes the Altar
  (`DecisionClosed`) and opens it again (`AltarOpened`, the same hero and object) once the
  level-ups are answered. A client anticipates this and keeps the Altar's dialog drawn beneath the
  level-up dialogs, unless the level-up covers the Altar entirely, as the original draws them
  (`altar_sacrifice` 0x5646A0 calls `hero_gain_experience` 0x4E33B0 with the dialog open):
  confirm the drawing against the original when you build the window (design note §14).
  *(2026-11-19, E08 T130: recorded by batch E14's E14-17 under Wine: the level-up dialog is drawn
  over the Altar's dialog without covering it; the Altar's places are already empty and its
  total's texts still the old ones until the level-up's OK, then it is redrawn. So both are drawn,
  the Altar beneath; survey `E08-trades.md` §21.)*

- 2026-11-17 (the adventure lane, E08 T125a, `adventure/e08-t125a`; **a creature bank's hint and
  the Shipwreck from the coast, for your hint and your moves**; E08 design note §9, §10, §16,
  ADV-BNK-006): a bank's `ObjectView` now carries its team's mark (`visited`: "(Not visited)" when
  false) and, with it, the guards' hint as a mine's without the army (`guards`: the word and the
  creature, none for several; none when looted or none stand, "(Visited)"), as
  `creature_bank_hint` 0x40D3F0. Its events: `ObjectMarked` then `GuardsChanged` at a mark set
  anew (an attack kind's visit, a search kind's yes), `GuardsChanged` after a battle's end to the
  teams that visited (no new variant; the protocol unchanged). The Shipwreck's water entrance is
  visited from the coast by a walker, a water walker and a flyer (the hero stays on land: a
  `MoveHero` path ending on that water tile, as onto an empty boat) and crossed by a flight; the
  reachable tiles give it as a destination.

- 2026-11-16 (the client lane, E27 M2 T8, `client/e27-m2-t8`; **for your E24 M5 and E27 M4, no
  action now**): (1) **The game's begin** (CLT-Q135, E27 decision 44): from E27 M3 T6 the shell's
  Begin builds and validates the engine's `GameSetup` (all 160 shipped maps listed, CLT-Q134),
  shows the loading bar, then a notice and back to the menu, keeping the setup, the hot seat's
  names (`h3_view::shell::multiplayer::Multiplayer::names`) and the map's file for your hot seat:
  what the hand-over carries is agreed at your E24 M5's T0. (2) **Options** (CLT-Q136): the system
  options dialog moves to E27 M4 with the menu in play, one dialog over your adventure screen
  (its shots are all taken over the map): its comparisons and the hand-back are agreed with you
  at E27 M4's T0.

- 2026-11-16 (the adventure lane, E08 T123a, `adventure/e08-t123a`, `cross-lane`; **increment
  13's types, for the drawing and the hint**; E08 design note §1, §2, §10): adventure ruleset
  part v21 adds the eleven creature banks in `base` (`cyclops_stockpile`, `dwarven_treasury`,
  `griffin_conservatory`, `imp_cache`, `medusa_stores`, `naga_bank`, `dragon_fly_hive` for class
  16 subclasses 0 to 6, then `shipwreck` 85, `derelict_ship` 24, `crypt` 84, `dragon_utopia` 25;
  names `CrBanks.txt`'s; kind `Bank(Bank)`, no kind mark). The map import types them from T123b
  (map import v24; until then they import as blockers); the hint's view (not visited, visited,
  the word and the creature by the team's mark) comes with T125a (ADV-BNK-006).

- 2026-11-16 (the town lane, E09 T34, `town/e09-t34`; **the capture harness, rebuild `xdd.dll`**): `clients/capture/xdd.c` logs three more sites (`tavern_class`, `tavern_type`, `leave_slot`, with their ranges; `h3drive.LOGGED` grew by three), so a sequence or batch run with an `xdd.dll` built before it fails the count of redirected calls: rebuild it with `clients/capture/setup.sh`. Also new and common: the reads and actions of README "Taverns and the Thieves' Guild" (the tavern dialog, the guild window, a town opened from the town list, a step's `resources`, `mana`, `narrow_pool`, a field `<encounter>|<field>`), and `auto_combat` returns when the battle ended before a human stack's turn (a loss to faster monsters kept pressing `a` until the limit).

- 2026-11-14 (the adventure lane, E08 T115b, `adventure/e08-t115b`; **for the bonus visitables'
  hint, nothing to do before it**; design note §10): protocol **v61**: the observation's `kinds`
  (the object types whose kind mark the viewer's team holds, in type order: the hint's bonus line,
  the original's `kinds_visited`) kept current by `KindKnown`; a Fountain of Fortune's and a Magic
  Spring's team knowledge as `Known::Visited` in `known` (never the fountain's value nor the
  Spring's entrances' state); the hero's own once-marks and timed marks in its own view (T115a's
  `bonuses`, `timed`: "visited" or "not visited" for the selected hero). The visits' messages:
  `NothingGained` (`Visited`, `Full`, `Emptied`), `SirensLoss`, `CreaturesUpgraded`.

- 2026-11-13 (the adventure lane, E08 T114b, `adventure/e08-t114b`; **for the garrison's
  right-click popup, nothing to do before it**): protocol **v54**: `GameObservation::garrisons`
  (`wire::game::MapGarrison`: by type and anchor, its owner, its army at the viewer's level as a
  town's garrison, TWN-VEW-002/-003, and the removable flag to the owner's team), kept current by
  `GarrisonChanged` and `GarrisonStackSeen` and the reveal's `garrisons` (ADV-GAR-006): what the
  original's popup (`garrison_popup` 0x416F80, the adventure survey `E08-garrisons-bonuses.md` §8)
  draws over `townqvbk.pcx`.

- 2026-11-12 (the client lane, E27 M1 T1b3a, `client/e27-m1-t1b3a`; for the adventure screen's
  options, when E24 builds them): the system options (E27's, opened by the adventure screen's
  item 10) set settings that play reads: Walk Speed, Computer Walk Speed and Blackout Computer
  (5, the enemy's moves not shown), Window Scroll Speed, Show Route, Move Reminder (End Turn's
  question with a hero able to move), Quick Combat (battles not shown), Town Outlines (the town
  screen), the volumes; Autosave has no item (`E27-design.md` §4.5: each one's readers). The
  shell keeps them in the user's `settings.toml` beside the combat options' (*provisional*,
  M4); Load, Main Menu, Quit and Restart end play through the shell's request (§4.3).

- 2026-11-12 (the adventure lane, E08 T113b, `adventure/e08-t113b`; **the nineteen types are
  imported**, for the drawing and the hint): map import v22 types T113a's nineteen (the four
  garrisons by class 33 and 219 and subclass, the fifteen bonus visitables at subclass 0), so
  shipped maps' `ObjectView`s now name them, an owned garrison with its `owner` (ADV-OWN-003);
  a garrison's army and removable flag are in its setup (`Holds::Garrison`), their view comes
  with T114b (ADV-GAR-006). A shipped map may hold two grounds (or Favorable Winds) of one type
  on one anchor with different pictures (nine on five maps; their `drawn` tiles differ).

- 2026-11-12 (the adventure lane, E08 T113a, `adventure/e08-t113a`, `cross-lane`; **increment
  12's types, for the drawing and the hint**; E08 design note §1 to §3): adventure ruleset part
  v20 adds the garrisons (`base:garrison` 33.0, `base:anti_magic_garrison` 33.1,
  `ab:garrison_vertical` 219.0, `ab:anti_magic_garrison_vertical` 219.1; kind `Garrison {
  anti_magic }`, ownable) and the fifteen bonus visitables (`base:buoy`, `swan_pond`,
  `faerie_ring`, `fountain_of_fortune`, `fountain_of_youth`, `idol_of_fortune`, `magic_spring`,
  `magic_well`, `mermaids`, `oasis`, `rally_flag`, `sirens`, `stables`, `temple`,
  `watering_hole`), and every type's `kind_mark` (`ObjectType::kind_mark`: whether its visit sets
  the team's kind mark, the original's `kinds_visited` the map hint reads; 26 types). The map
  import types them from T113b (map import v22: until then they import as blockers); the kind
  marks a team holds reach the observation with T115b, a garrison's owner and army with T114b.

- 2026-11-12 (the server lane, E16 M5 T7a1, branch `claude/keen-tesla-q5uxid`; **for your M4
  T3, T4; replaces (1) of the T5c note below in its form**): protocol **v44**. (1) `GameReveal`'s
  tiles are packed as a snapshot's: `adventure.palette` (each `{ explored, view }` once) and
  `adventure.areas` (per level that has any, by level, the smallest rectangle holding the tiles
  whose sight changed: `corner`, `width`, `tiles` row by row, 0 an unchanged tile, `n` the
  palette's `n`-th view from 1), no longer `tiles` (one entry per tile made an XL map's whole
  reveal 6.5 MB, 60 times its snapshot; now 390 KB with its objects). Fold it with
  `GameObservation::reveal` as before. (2) A player may get a **fresh `GameSnapshot`** after a
  command's messages, before its `GameTurn`, never while a battle is open in its view: it
  replaces the view. A bounded record (the server's, part 2) is cut only at a snapshot, so it
  always begins with one (the header's `first` is its number); `h3-server record` writes whole
  games unbounded, one snapshot.

- 2026-11-12 (the client lane, E27 M1 T1b2, branch `client/e27-m1-t1b2`; **for your M5**,
  informational until then): the scenario screen's advanced options read (`docs/epics/E27-design.md`
  §3.12 to §3.17). Besides `GameSetup` the hot seat's hand-over will carry the players' names and
  which slot holds which colour (the shell's own), and the **turn's duration** (1 to 30 minutes or
  unlimited, `turndur.txt`): the engine has no timer, so the shell stubs it (a `CLT-STUB` at E27
  M3); where the hot seat's clock (the original ends a human's turn when it runs out) lives is
  yours to place at M5, agreed with E27 M3's T0. A colour's flag in the hot seat passes it to the
  next player, the two swapping (CLT-Q94).

- 2026-11-12 (the client lane, E27 M0, branch `client/e27-m0`; **for your M5**, informational
  until then): E27 (the game shell) is refined (`docs/epics/E27-game-shell.md`). Its scenario
  screen hands a hot-seat game over as the engine's own `h3_game::GameSetup`, filled (CLT-Q79:
  no new crate, no new type for it); what else the hand-over carries (the hot seat's names, the
  map's file) is agreed with you at E27 M3's T0 within the existing crates. The system menu in
  play (save, load, restart, main menu, quit) draws over your adventure screen at E27 M4, agreed
  with you then. The campaign screens moved to E20 (CLT-Q82).

- 2026-11-12 (the client lane, E17 M7 T4, branch `client/e17-m7-t4`; **for your M4 T5 and M5**;
  E17 closes with it): **the hosted battle after M7's review**, design §13.2 to §13.6.
  (1) **The element borrows** its `Ground` and your game (`Prepared<'g>`, `Hosted<'g>`; the
  user kept it, R2-4): run each battle in a scope that owns them; tell the client lane if your
  host needs an owning element. (2) `Prepared::start` returns `Result<Hosted, Box<NotStarted>>`:
  a failed start hands your lent sound back (`NotStarted::sound`; `From` into `String` keeps
  `?`), every fallible step before the sound is taken. (3) **One `GameSeat` per battle:** it
  follows the battle opened last until `Ground::load_game` loads it, then stays on it and reads
  nothing past the message after its end; make a new seat for the next battle. (4)
  `Ground::load_battle(source, battle)` plays a `GameBattle` you hold in memory (no temporary
  file); a battle open when the messages begin (a game restored mid-battle) opens at the
  `GameSnapshot`. (5) **The sound:** a battle that cannot read its sound archive silences yours
  (a request for the missing track `""`, so `has_track("")` is true afterwards), services it and
  passes the focus on: keep calling `Hosted::sound` and `Hosted::focus` every pass; the tracks
  your sound already has are not decoded again, the battle's stay on it and are reused; a Restart
  after the end brings the combat music back in both profiles. Additive in the common crates:
  `h3_ui::music::Music::request_again` and `Queue::request_again` (the pending name replaced
  whatever the profile's filter), `h3_sdl::Audio::open_driver` (a named SDL driver: the tests'
  `dummy`). (6) A mouse's hold over a panel button stands the battle only at its timeline's end
  (a record plays on), and a box opening over it ends the hold. (7) **E17's threads placed with
  you** (the user, at T4): M6 T5e, the captured artifacts' message after Eagle Eye's
  (`captured_artifacts_message` 0x4772B0, design §12.21), the client lane builds once your M4 T5
  tells the winner's owner the adventure's capture list (HER-ART-004); CLT-Q4, the battlefield
  picture's variants 0 and 1, read from the wire since T3b1, recorded once one of your battles
  has trees, mountains or dead vegetation around it; `CLT-STUB-4` (no Auto Combat switch in a
  live game's battle: the computer's choices for the human's side need your state) and
  `CLT-STUB-5` (a hot-seat battle between two of your humans is not played: the seat's side
  alone is human; with your M5). Each is a line in the client queue's inbox: tell the client
  lane when your task reaches it. HD layouts and graphics options are E29 (the backlog), for
  both clients.

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

- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **your M4 T0 request (2), for the random dwellings**): a random dwelling not tied to a town (216 to 218) resolves when the game is built (the setup stream, before every other draw of the static map): its `ObjectView::object_type` is the drawn dwelling type, its `anchor` the original's moved one (the type's shift: only the Angel Portal's, 1; it may lie one column past the map's right edge, x == width, where only its free cell is) and `blocked`/`visitable` the type's footprint, its `owner` the map's; nothing tells a viewer it was random. One tied to a town stays an untyped blocker until E09 (ADV-STUB-13).

- 2026-11-02 (the battle lane, E26 T5; for a computer side in your battles, when you add one):
  call the fair system actor `h3_battle_ai::choose_fair` by default (ADR-0027 point 4 as
  amended; the user, AI-Q10), the all-seeing `choose` only as a game's option.

- 2026-10-31 (the adventure lane, E08 T83b2b; **for E24 M4 T3**, the "increment 9's views" it
  waits on): `ObjectView` carries `owner` (every viewer that sees the object), `production` (a
  mine's resource and daily amount, its owner's team only), `guards` (`GuardsView`: `creature`
  when all stacks are of one, none for the original's generic text; `count_word` of the total
  count, every stack counted; `army`, the stacks, for the owner's team only), `visited` (a weekly
  object's mark, the viewer's own) and `boat_tile` (a Shipyard's); it is no longer `Copy`. The
  object identity, the random objects' drawn types and MCL-Q21 (b) of your note stay open in the
  adventure lane's inbox; the month's kind comes with T85.

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
