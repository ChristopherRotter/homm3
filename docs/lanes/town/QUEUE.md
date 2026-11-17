# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 4: the Tavern, refined at T30, `TWN-Q15`, decisions 98
to 128; its drafts accepted at T33b, `TWN-Q18`; batch TWN-4 scored by T34; the drafts in the ADRs
by T35a, `TWN-Q19`; its code by T35b, decisions 129 to 131; the town part v7 by T36, decision
132); done tasks in `E09-log.md`; increments 1 to 3
(closed by T10, T20 and T30) in `E09-archive.md`. The building tasks are the design note's
`E09-design.md` §79 (round by round, with the order against E08's increment 13); each task's cut
confirmed at its start; every entry's acceptance includes the Definition of Done
(`docs/05-testing.md`) and the exit criteria it serves (the epic's list, decision 121).

**E09 T37 — the pool in the game** (round 5; `cross-lane`; design §61 to §63, §68, §70, §75):
T37a the pool's `Offered`, the records at a first draw, the away records, the players' factions
and slots (game state hash), the draw and the fills from the taverns' stream (TWN-TAV-001, -002;
GAM-CAL-002), the slots over a turn and every day's clear of the marks (TWN-TAV-005; decision 126; the
day's spell points of every record already built, decision 127: told as an own hero's,
`SRV-STUB-6`, §75), `GameObservation::tavern` and `TavernOffered`,
the protocol, the replays re-recorded, the week's bench; T37b the map's customised types
(GAM-HER-001, TWN-SET-005; `GameSetup`, the import after E08's T123). TWN-TAV-001's catalog entry
is T37a's (T36 built its data, decision 132). **Status:** next.

**E09 T38 — the hire and the leaves** (round 6; `cross-lane`; design §64, §65, §67, §69, §75,
§76): T38a `TownCommand::Hire` (TWN-TAV-003), the adventure's placement, the replacement
(TWN-TAV-004), `HeroHired`, the legal commands, `town hire`, the env's `HIRE` (v12), the protocol;
T38b a hero that leaves the game (TWN-TAV-006; GAM-BAT-003, the capture's leave;
`TWN-QUIRK-12`), the returns (TD-18, TD-19), the dismissal (GAM-MAN-003, TD-22, `manage dismiss`;
no env entry), the protocol; in merge order with E08's T124b. **Status:** queued.

**E09 T39 — the map's objects and the information** (round 7; `cross-lane`, after E08's T123;
design §66, §73 to §76): T39a the kinds `Tavern` and `DenOfThieves` (the adventure part, classes
95 and 97 imported and written), the tavern decision (GAM-DEC-010, `TavernOpened`, `answer
tavern`, the env's answers by role v13); T39b the Thieves' Guild (TWN-TAV-007 to -009: the query,
the Den's `ThievesGuildSeen`, TD-23, `TWN-QUIRK-13`, `TWN-STUB-17`, `-18`; the adventure's power
and the day's gold made public), the protocol's request. **Status:** queued.

**E09 T40 — the gates** (round 8; design §78): the taverns map and its scripted replays, the
census of the 160 maps' first offers and of the maps classes 95 and 97 unblock, the pool's
properties, the benchmarks, `--with-original-data`. **Status:** queued.

**E09 T41 — the audit** (round 8): against the exit criteria; increment 4 closed (the archive,
the README's epic row in the session's `shared/` pull request); increment 5 (the mage guild and
the services) refined. **Status:** queued.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-16 (the adventure lane, E08 T123b, `adventure/e08-t123b`; **for your T37b and T39a**):
  E08's T123 is done, **map import v24** is taken (the creature banks typed: `h3-map`'s
  `codes::object_type`, class 16 by subclass, 85, 24, 84, 25; `CREATURE_BANKS` beside
  `BONUS_VISITABLES` in `import/codes.rs`). Your import changes take v25 (bump
  `MAP_IMPORT_VERSION` and its doc line); the goldens then move their `import_version` line, and
  `h3-server`'s `tests/record.rs` its `import` case (the adventure lane's `TRAPS.md`).

- 2026-11-16 (the adventure lane, E08 T122, `shared/e08-t122`, `cross-lane`; **for your T35's
  merge order**): E08 increment 13's drafts are in the ADRs: ADR-0016 points 1, 4 and 6 and
  ADR-0017 points 1, 2, 4, 6, 9, 10 and 11 each gained a part **Increment 13** after its increment
  12 part (ADR-0016 point 4 also a batch E13 paragraph), their consequences, alternatives and
  revision histories a line each; ADR-0022 point 8 is new (the creature bank battle). Your T35
  rebases its increment 4 text on them (ADR-0017 point 9's line names the merge order with E09's
  increment 4). The glossary (`docs/03-domain.md`) and `docs/06-security.md` gained increment
  13's lines after increment 12's.

- 2026-11-13 (the adventure lane, E08 T115b, `adventure/e08-t115b`; **a signature in your
  callers, mechanical**): `AdventureState::apply` takes the day of the week beside the visitor's
  stream, `(player, weekday, draws): (PlayerId, Weekday, &mut Rng)`, and `visit_held` (with the
  revisit, which runs through `apply`) a pair `(weekday, draws)`; `h3_adventure::Weekday` (from 1,
  `Weekday::FIRST`), the game's `GameState::weekday(rules)` (`CalendarParams::weekday(day)`: the
  calendar's (day − 1) mod `week_days` + 1). The Idol of Fortune's and the Temple's amounts are by
  it (ADV-BON-002); nothing of yours reads it. A new call of yours passes `state.weekday(rules)`,
  a test `Weekday::FIRST`. No version moved by it.

- 2026-11-12 (the server lane, E16 M5 part 2's refinement, SRV-Q29; **the user's decision,
  informational**): in increment 2c the server lane builds `Viewer::Spectator` (public facts only)
  through `h3-towns`' projections too (a town's view at a spectator's level), in a `cross-lane`
  pull request with the user's consent, asking your lane's review; nothing for you to build. Your
  two notes of 2026-11-10 (`town_offer`, `town_events_at_start`) go to E16 M5 T11.

- 2026-11-11 (the adventure lane, `adventure/srv-q26-reveal`, `cross-lane` with the user's consent;
  **in your paths, for information**): SRV-Q26 (a)'s query `h3_game::reveal` names the towns a
  viewer newly explored with their views through `town::view::views_of(state, rules, viewer,
  towns)`, a new `pub(crate)` function beside `town_views` (the `Seer`'s view of each, in order;
  the viewer's explored tiles read from the state). Your `Told`'s "a town the viewer comes to see
  by exploring" is that query's now. Tested in `town/tests/view.rs`
  (`a_town_explored_since_a_grid_is_revealed_with_its_view`); `town/tests/capture.rs`' `step`
  drops the `MovementChanged` events a command now tells last (SRV-Q26 (b)). Keep `views_of` in
  step with `town_views` when the views change.

- 2026-11-10 (the adventure lane, E08 T105b2, `adventure/e08-t105b2`, `cross-lane`; **a town's
  revisit, for your visit**; increment 11's design note §13): `AdventureCommand::Revisit { hero }`
  (ADV-TEL-008) visits the entered object under a hero again as a step's last step onto it would,
  no points, no step, no guard test: for a hero standing on its own town's entrance that is
  `visit_town` with `Visit::Enter` (`town_visit` as it stands), at 0 points too and as often as
  the player likes. `legal_commands` lists it for every such hero after the `BuildBoat`s (and
  before a town's builds once T18b2 lists them: `revisitable`'s place is after `buildable_shipyards`
  in `h3-game`'s `legal.rs`). Whatever a town's visit opens later (its screen, a garrison
  exchange) opens at a revisit too; nothing moved in your hashes. The replay format is v12
  (`revisit <hero>`): a town bump of the format is v13.

- 2026-11-10 (the server lane, E16 M5 T3d; **for when `TWN-STUB-13` is lifted**): your
  `TownChanged`, `TownStackSeen` and `TownIncomeSeen` (E09 T18b2a) are mapped and folded in
  `h3-protocol` (the fold replaces the viewer's town view); `SRV-STUB-12` and `-13` are lifted.
  The fold gate (`h3-protocol`'s `tests/game_fold.rs`, the 55 golden replays) now exempts the
  garrisons and the town owner facts only for a reveal and for `TWN-STUB-13` (3 viewer-commands:
  another team's town falling from level 3 to 2 as a Visions holder's reach is lost). Once
  Visions' reach is told, the gate fails with "lift [...] from it": drop `VISIONS` from those two
  parts in the same pull request (`cross-lane`), as a new event field or variant is mapped there
  (the global trap). This note replaces the server lane's of 2026-11-09 (T3c2).
- 2026-11-09 (the adventure lane, E08 T105a1, `adventure/e08-t105a1`; **a signature, for your
  callers**; increment 11's design note §13, §18): `AdventureState::apply` takes `(player, draws)`
  in place of `player` (`draws: &mut Rng`, the acting player's stream: a visit's draw, a
  monolith's exit, comes from it) and `visit_held` takes `draws` before the sink; the game passes
  them from `GameState::adventure_for(player)` (the adventure state, the store of heroes and that
  player's stream). Every caller in the adventure lane's paths is updated, none of yours called
  them; a new caller in `h3-game`'s `src/town*` (a town command that visits) uses
  `adventure_for`, and the adventure crate's tests `testing::stream()`. Nothing moved in a hash.
- 2026-11-04 (the client lane, E17 M6 T3c3, `client/e17-m6-t3c3`; **when you change
  `h3-towns`' or `h3-pack-towns`' public API**): the battle client's binary now compiles the packs'
  town part (`h3_pack_towns::compile`) and reads `TownRuleset::town`, `TownRecord::building` and
  `buildings`, `Building::key` and `slot` and `BuildingId::from_index` (a siege's buildings and a
  Grail building by key in the stats dialog, the original's map by slot; design E17 §12.11). CI's
  clients job watches both crates (#704): a change there runs `clients/scripts/check.sh`, locally
  `scripts/verify-once.sh clients --base origin/master`; a dependency change updates
  `clients/Cargo.lock`.
- 2026-11-03 (the battle lane, CLT-Q40; its siege half done by T6b, `QUEUE-ARCHIVE.md`): the
  owner's Grail building that adds a hero's morale or luck (the Colossus, the Spirit Guardian)
  names itself as `EffectSource::Building { faction, index }` on that hero's `BattleEffect`, when
  the game gives it.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 4, the Tavern (weights: only the tavern's draw reads them, survey `E09-towns.md`
  §4.4; read by T32: `E09-tavern.md` §2.2, the Complete's file has the nine columns), increment 5
  (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 4, the Tavern (read by T32:
  `E09-tavern.md` §5, the customised types applied before the first draw).
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 4, the Tavern, with E10 (read by T32: `E09-tavern.md` §4, the owner's
  tavern at once).
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 5.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 5.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 5.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 5.
