# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1** (stills of the map view, closed 2026-10-24), **M2**
(the map moving, closed 2026-10-28), **M3** (the adventure screen, closed 2026-10-31) and **M4**
(the engine's games shown, closed 2026-11-23 at T6, its review list accepted at M5 T0) are
archived with their review lists in `docs/epics/E24-archive.md`; their queue history in
`QUEUE-ARCHIVE.md`. **M5** (hot seat on the adventure map) was refined with the user at T0
(2026-11-24): its scope, decisions and exit in the epic's M5 section; M6 to M8 sketched in its
"Later". Every task's cut is confirmed with the user at its start. `MCL-STUB-5` (a computer
colour's turn ended at once) since T3a, `MCL-STUB-7` (a screen of M6 or M7 or a decision the view does not ask: its notice)
since T4c, `MCL-STUB-9` to `-11` (a battle no find tells, an answer's battle's boxes, the track's
kept place at a battle's return) since T6a. T1 was cut in three at its start
(T1a to T1c done 2026-11-24 and 25), T2 too (T2a to T2c done 2026-11-25), T3 in two (T3a done
2026-11-25, T3b 2026-11-26), T4 in three (T4a done 2026-11-26, T4b 2026-11-26 in two sessions, T4c 2026-11-27 kept whole,
in four parts), T5 kept whole (done 2026-11-28 in five parts), T6 in four by the user at its start
(2026-11-28: T6a, then T6b to T6d; the battle client's parts of T6c and T6d this lane's, in
`cross-lane` pull requests; T6e cut out of T6a at its end). T6a done 2026-11-28 in three parts.
Next: T6b (Wine) or T6e (none), the user's pick.

### E24 M5 T6e — the guards' fights in play

- **Scope:** the guards' questions T4c left notices (`MCL-STUB-7`'s fights: a mine's, a
  dwelling's, a bank's, a guarded pile's question asked in play, Part A's questions in
  `visits::opened`, an OK that fights in `wire_answer`), their battles hosted as T6a's, each
  object's boxes and flag after the battle (Pandora's Box's rewards, `MCL-STUB-10`); an event's
  guards told as a find (`MCL-STUB-9`); the record player's walks' and answers' battles by the
  switch as play's (design §5.33: `battle_start` fades every shown battle; §5.20's plain cut was
  no reading).
- **Acceptance:** each kind played in a hot-seat game under Xvfb, `--check` exact; the record
  player's battles as recorded (`drawn`); the scoped verify.
- **Depends on:** T6a. Cut out of T6a at its end (2026-11-28: the runner's, the guards' questions
  were T4c's notices). **Status:** queued.

### E24 M5 T6b — the defended and the two humans' battles read and recorded

- **Scope:** another human's hero attacking a human's hero and town in a hot seat, read and
  recorded under Wine (`h3map.py`'s `hot_seat`): the cover before it (`adventure_battle` 0x4AE194
  to 0x4AE233: GENRLTXT 48 with 49, 50 or 430, the cover with the defender's colour; design §5.31,
  "A defended battle's cover"), the one combat screen with both sides by mouse, the return to the
  attacker's view; what `MCL-STUB-3` and `CLT-STUB-5` need; `seat-*` sequences.
- **Acceptance:** each recorded twice the same; the readings in the design.
- **Depends on:** T6a. **Status:** queued.

### E24 M5 T6c — the defended and the two humans' battles built

- **Scope:** `MCL-STUB-3` (play and the record player hand a defended battle over, with its
  cover) and `CLT-STUB-5` (one battle seat over both humans' sides, each command its side's
  player's: this lane's change in `h3-battle-client`, a `cross-lane` pull request with the user's
  consent of 2026-11-28, the client lane's inbox told); the result in each human's view.
- **Acceptance:** both played in a hot-seat game as T6b recorded them; the scoped verify and the
  client lane's check.
- **Depends on:** T6b. **Status:** queued.

### E24 M5 T6d — Auto Combat in play

- **Scope:** `CLT-STUB-4`: the battle panel's Auto Combat in a live battle, the host playing the
  human's side by the battle AI (the seat's request: this lane's change in `h3-battle-client`, a
  `cross-lane` pull request with the user's consent of 2026-11-28).
- **Acceptance:** a battle played under Auto Combat to its end in a hot-seat game; the scoped
  verify.
- **Depends on:** T6a. **Status:** queued.

### E24 M5 T7 — a whole game

- **Scope:** a two-human hot-seat game on a shipped map played from the shell's Begin to a day
  limit: headless by a script of commands (its records kept) and in the window by hand; the
  gaps found placed (a question, a stub or a fix).
- **Acceptance:** the game played both ways with every command the engine's; no rules table or
  map text read in play.
- **Depends on:** T6a to T6e. **Status:** queued.

### E24 M5 T8 — M5's closure

- **Scope:** fresh reviewers (one per area), their findings fixed or asked, M5's review list for
  the user, the epic's M5 archived, a hand-test kit on the user's machine (`clients/out/win/`).
- **Acceptance:** the exit (the epic's M5); the user's checkpoint.
- **Depends on:** T7. **Status:** queued.

## Inbox

- 2026-11-27 (this lane, E24 M5 T5; **for the user to place**): MCL-Q11 (c) is settled by ear
  (the M4 hand test's part 5: each `HORSE2x` is its terrain's `HORSE0x` with more footsteps, as on a
  road). The option it asked for: a step on a road of terrain x plays `HORSE2x` (no 28: water), the
  original's set by default and for the comparisons. A small task (the sound's settings, the step's
  road read from the view's tile); not in M5's scope.

- 2026-11-28 (the town lane, E09 T59b1, `town/e09-t59b1`, `cross-lane` with the user's consent;
  **in your paths, two test literals; for your review**): protocol **v88**'s `UniversityVisit`
  names its place (`place: UniversitySeen`, `Object { object: ObjectRef }` or `Town { town }`) in
  place of `object_type` and `anchor`, as does `GameEvent::UniversityOpened`: a town's Magic
  University (TWN-UNI-001) opens the same decision. Your `h3-map-client` `window.rs` test literal
  and `h3-map-view` `notice/tests.rs`' (now a town's University) follow so that the workspace
  builds; the notice of a University is unchanged.
- 2026-11-27 (the town lane, E09 T58b, `town/e09-t58b`, `cross-lane`; **in your paths, one line;
  for your review**): protocol **v86** adds `MarketWindow::artifact_sales` (a town's artifact sale:
  each place of the hero's that sells, its artifact and price per resource), `MarketScreen::SellArtifact`,
  `MarketAnswer::SellArtifact { place, take }`, `MarketReject::Place` and `NotTradable`; your
  `h3-map-view` `notice/tests.rs`' window literal gained `artifact_sales: Vec::new()` so that the
  workspace builds. The notice of a market window is unchanged.
- 2026-11-27 (the adventure lane, E10 T1b, `adventure/e10-t1b`; **for `MCL-STUB-4`, nothing to
  do now**): E10 increment 1 designs the seven days (`docs/epics/E10-design.md` §10, §15): the
  engine will tell `LastTownLost { player }` (to the player, at the check that sets the days),
  `DaysLeft { player, days }` (at its turn start) and `PlayerDefeated { player, why }` (every
  player), with the original's boxes the clients' rule (E10 decision 12: GENRLTXT 7 only when the
  player is in turn; a banished human told GENRLTXT 9 in the third person). Your record's
  `game-week` (Blue banished on day 8) is the rule's timeline. Built at E10 T4 (protocol v86).
- 2026-11-25 (the adventure lane, E08 T143b; **for your planning, nothing blocks**): protocol
  v84 casts Dimension Door. Its arrival is `AdventureEvent::HeroTeleported` (a teleporter's: each
  end `None` where the viewer does not see it; a hero in a boat takes the boat), after the
  owner's `SpellCast`; a wrong target is the owner's `CastFailed { reason: Target }` (GENRLTXT 71,
  the hero stays, its points and the cast spent). Refusals: `SpellReject::NoPoints` (126),
  `NoCastsLeft` (339), `OutOfWindow` and `NotFree` (no cursor), `WrongKind` (an explored water or
  land target of the wrong kind under `strict-fixed`: no cursor there; under `faithful`,
  `ADV-QUIRK-55`, the original's cursor and 71). The door's cursor: under `faithful`
  (`ADV-QUIRK-51`) the original's on any free tile of the 19 × 17 window, hidden ones too; under
  `strict-fixed` on a seen free tile of the hero's kind or any hidden tile, the hidden target
  revealed alone when it fails. `SpellReject::NotBuilt` is gone. `legal_commands` lists no cast
  until T144a.

- 2026-11-25 (the ai lane, E31 T2a, `ai/e31-t2a`; **the entry point for `MCL-STUB-5`, for your
  planning**): the original's computer player is `h3_game::computer::choose(&GameState,
  &GameRuleset, PlayerId, &mut Memory, &mut Rng) -> Option<GameCommand>` (ADR-0030 point 2):
  one command a call, which the host applies as `Actor::Player(player)` and then calls again,
  for every computer colour each time the game changes (it answers that colour's open decisions
  in any player's turn; `None` outside its turn, while another player's level-up holds the game,
  and for a battle). A battle's computer side is the host's: `h3_battle_ai::choose(battle,
  rules.battle(), side, rng)` as the colour's `GameCommand::Battle`, for the side it owns while
  that side is to act (`OpenBattle::owners`, `side_to_act`). The host keeps one
  `h3_game::computer::Memory` per computer colour (a plain value: `Memory::encode` / `decode` for
  a saved game, ADR-0030 point 5) and a stream of its own per colour (never the game's; e.g.
  `Rng::for_stream(seed, StreamId::seat(index))`). A turn ends with the policy's `EndTurn`.
  `h3_sim::computer::play` is a host of every colour to read (`crates/h3-sim/src/computer.rs`).
  As of T2a a computer hero does nothing but its town's garrison moves (each hero done at once,
  `AI-STUB-2` until E31 T2i and T2j), and its decisions take `default_answer`; the commands are
  legal on every shipped map. For M4's review item 6 (how the original shows a computer's turn):
  `ai_hero_turn` 0x5261F0 selects and centres the hero when the viewer sees its tile or a
  neighbour and neither Blackout Computer nor a network game hides it (survey
  `E31-computer-player.md` §1.3 step 3).

- 2026-11-25 (this lane, E24 M5 T2a; **for T7's whole game**): of the 236 player records the 118
  golden adventure replays now give, 205 play to their end in `h3-map-view record`, the same
  on `master`'s binary (every timeline identical): 24 are the duel's and the army's maps (no
  original template), and 7 of the replays added since M4 T6 stop in the record player: the
  `teleporters-s1711-d28`, `-s2002-d28` and `-s2647-d28` walks (a path through a teleporter:
  "no path"), `taverns-hires-s1` player 0 (a hired hero's walk: "no path") and `builds-rows-s1`
  player 0 ("a record's walk … of a hero not listed"). The record player's (M4 T4c), not play's.

- 2026-11-25 (the adventure lane, E08 T143a; **for your planning, nothing blocks**): protocol
  v83 casts Summon Boat and Scuttle Boat. A cast's `SpellCast` now comes before its effect's
  events; a charged failure follows it as the owner's `GameEvent::CastFailed { hero, spell,
  reason }` (`Chance`: GENRLTXT 337 Summon Boat, 338 Scuttle Boat; `NoBoat`: 336; `Target`: a
  hidden target revealed empty under `strict-fixed`). The boats: `AdventureEvent::BoatSummoned
  { boat, from, to, facing }` (the original's dissolve at each end the viewer sees; an end it
  does not see is `None`, as a jump's), `BoatCreated { tile, boat_type }` (facing 2, as
  `BoatBuilt`), `BoatScuttled { tile, boat }`; refusals `SpellReject::NoWater` (335),
  `OutOfWindow`, `NoBoat` (no cursor; 334 aboard is `InBoat`, as Fly's 193). Scuttle Boat's
  cursor: under `faithful` (`ADV-QUIRK-52`) the original's, on any empty boat of the 19 × 17
  window, hidden ones too; under `strict-fixed` on a seen empty boat or any hidden tile of the
  window, the hidden target revealed alone at the cast. `legal_commands` lists no cast until T144a.

- 2026-11-24 (the adventure lane, E08 T142b; **for your planning, nothing blocks; no version
  moved**): a flying hero's paths now cross tiles its player has not explored, by
  `ADV-QUIRK-15` (increment 15's design note `E08-design.md` §11.1): under `faithful` as the
  original (rock refused, priced as what lies there), under `strict-fixed` every hidden tile an
  assumed pass. A hidden tile is never a path's end in either. Under `strict-fixed` an accepted
  flight can **end short**: on the last stop node before a step whose revealed tile forbids it
  (or on its start), its `HeroStepped` events only up to there, with more tiles explored than
  those steps reveal (the undone passes'), as an event's stop leaves the rest of the path
  unwalked. The client's path drawing and its arrival need nothing but the engine's reach and
  events; the original's cursor over hidden tiles is the faithful side's.

- 2026-11-24 (the adventure lane, E08 T142a; **for your planning, nothing blocks**): protocol
  v82 carries the cast on the map: `GameCommand::Cast { hero, spell, target }`, the owner's
  `GameEvent::SpellCast { hero, spell, mastery, cost }` and `AdventureEvent::DaySpellsChanged
  { hero, spells }` (Fly's and Water Walk's levels, the day's Dimension Doors), and
  `OwnMapHero::spells`; the refusals `GameReject::Cast` mirror the original's boxes (GENRLTXT 207
  the points, 748 Cursed Ground, 685 an anti-magic garrison, 193 Fly aboard; a re-cast is the
  original's silent nothing). Fly and Water Walk only so far (T143 the boats and the door), and
  `legal_commands` lists no cast until T144a.

- 2026-11-23 (the adventure lane, E08 T141a, `adventure/e08-t141a`; **for your planning**): the
  Prison and the Market of Time are typed (map import v29); a visit's events (protocol v81): the
  Prison's `ObjectRemoved`, then `PrisonerFreed` (the hero on its tile, as a hire's
  `HeroHired`), `PrisonFull` (ADVEVENT 103), `PrisonEmptied` (104). Each player's list of heroes
  in the original's order is new (GAM-HER-003): the observation's `hero_list` (the start list by
  hero type, then each hero that joins the map at the end; a hero that leaves closes it up), the
  order of the original's hero list panel.

- 2026-11-21 (the adventure lane, E08 T137b, `adventure/e08-t137b`; **for your planning**):
  increment 15 types the Prison (62, visited from a neighbour; the prisoner never shown) and the
  Market of Time (50, kind `empty`: entered, nothing happens), and adds the cast on the map
  (`GameCommand::Cast`; the door's and Scuttle Boat's cursors by the legal targets: under
  `strict-fixed` every hidden tile in the 9 × 8 window is offered, under `faithful` the original's;
  the events of design note `docs/epics/E08-design.md` §16).

- 2026-11-21 (the client lane, E27 M4 T6, `client/e27-m4-t6`; **for E24 M5**): a load's
  hand-over. Load Game's Load keeps the loaded game (`save::Loaded`) and the names by colour
  (`Scenario::seated`) for E24 M5, which makes exactly those colours human (CLT-Q160 (a): fewer
  names leave saved humans to the computer, more take a computer's colour); a hot-seat save loaded
  from Load Game single names the loaded game's current player's colour alone (CLT-Q170 (a)). The
  engine step that changes a loaded game's human flags is asked of the adventure lane (its inbox).
  E27 M5 (the system menu in play, Options, Save from play, Restart, the end of a scenario) is
  refined once E24 M5 hosts play.

- 2026-11-20 (the adventure lane, E08 T134a, `adventure/e08-t134a`, `cross-lane`; **built, for
  your view's fold; nothing to do now**): the Altar of Sacrifice is entered and its visit sets the
  team's kind mark (k 28: `KindKnown`), then opens `DecisionView::Altar` (`AltarView`: the screens,
  the places, each artifact's and stack's experience, the experience to the next level) with
  `GameEvent::AltarOpened`; a sacrifice tells `HeroChanged` (each removal, then the experience) or
  `StackChanged`, and when its levels open a level-up `DecisionClosed`, `LevelUpOpened`, then
  after the level-ups `AltarOpened` again (decision 371, your note of 2026-11-18 below).
  `h3-map`'s `tests/view.rs` folds `Open::Altar` beside `Open::Market`.

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
