# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T97c (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 10 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05; increment 10, dwellings and hiring, by T97's audit). **Increment 11, the teleporters**, refined with the user at T97c (ADV-Q44; decisions 260, 261): round 1, the survey and the ADR amendment drafts first: T98a in two (decision 262), both done (T98a2 with the checkpoint ADV-Q45, decisions 263–269); T98b in two by side (decision 270): T98b1 done (`adventure/e08-t98b1`: design parts A and B, ADR-0016's draft), T98b2 done (whole: decision 271; part C, ADR-0017's draft, the checkpoint ADV-Q46, decisions 272–274: the note, both drafts and the cut accepted). Rounds 2 to 8 (design §17): T99 in two at the code's seam (decision 275): T99a done (`adventure/e08-t99a`, `cross-lane`: the writer's five classes, the spec's `level`, the gates' pairs, the boats on Whirlpools' tiles); T99b in two by land and water (decision 276): T99b1 done (`adventure/e08-t99b1`: 11a's land rows), T99b2 in two by spec (decision 277): T99b2a done (`adventure/e08-t99b2a`: 11a's Whirlpool rows), T99b2b done (`adventure/e08-t99b2b`, `cross-lane`: the spec E11b, the batch's README row); round 2 done. Round 3: T100 in two by spec (decision 278): T100a done (`adventure/e08-t100a`: the harness's replay, branch and box reads, the recipe `E08-batch-11a`, three runs, the template filled), T100b done (`adventure/e08-t100b`: the key repeated, the Whirlpools' list and the tiles' reads, the recipe `E08-batch-11b`, three runs, the template filled); T101 done (`claude/epic-brahmagupta-rp3lje`, whole, docs only: survey §15, every row as read but E11-19; the checkpoint ADV-Q47, decisions 279, 280); round 3 done. Round 4: T102 done (`shared/e08-t102`, `cross-lane`, whole: decision 281; the ADRs, the glossary, the security lines, the effect kind `whirlpool_protection`); round 4 done. Round 5: T103 in four (decision 282): T103a done (`adventure/e08-t103a`, `cross-lane`: the kinds, the packs and their check, `whirlpool_protection`; part v18; the bot reports' line, decision 283); T103b done (`adventure/e08-t103b`: the import of the five classes, A-39 refused, `ADV-QUIRK-41`; map import v21, with the inbox's class 220 by the mines' table; the census of the shipped maps); T103c done (`adventure/e08-t103c`, `cross-lane` for the experiments README's row, whole: decision 284; batch E11c, E11-20 to E11-23 every row as read, survey §15.1); T103d done (`adventure/e08-t103d`, whole: the guard first for every entered kind, `Rest::Visit`; no version moved); round 5 done. Round 6: T104 done (`adventure/e08-t104`, whole: decision 285; A-42's refusal, the channels and the gates' pairs, `build-gates-xl-144`; the shipped maps' census brought forward, decision 286); round 6 done. Round 7: T105, T105a in two by land and water (decision 287): T105a1 done (`adventure/e08-t105a1`: the stream in `apply`, the monoliths' and the gates' jumps, `ADV-STUB-16` for an occupied exit; no version moved); T105a2 done (`adventure/e08-t105a2`: the Whirlpool's loss and jump with the boat, `WhirlpoolLoss`; no version moved); T105b in two (decision 288): T105b1 done (`adventure/e08-t105b1`: the battle and the meeting at an exit, `Rest::Jump`; game state hash v27), T105b2 done (`adventure/e08-t105b2`, `cross-lane`, whole: decision 289; `Revisit`, replay format v12, the env's action encoding v8; seven workloads re-baselined); round 7 done. Round 8: T106 in three (decision 290, as T86 and T96): T106a done (`adventure/e08-t106a`: the map `e08-teleporters-36`, three random and the scripted replays); T106b in two (decision 291): T106b1 done (`adventure/e08-t106b1`: the plans' teleporters, `Step::Jump`, the `play.rs` checks, a cover test); T106b2, T106c and T107 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T106b2 — the teleporters' view checks** (round 8; design §10, §14; decision 291)

- **Scope:** `view.rs`: no viewer told an end of a jump it does not see, nor a channel's exits or
  a gate's pair, for every player after every command (`views_move_as_the_events_say`) and in both
  games of `a_viewer_learns_nothing_it_cannot_see`; the hygiene comparison's `vary` for the five
  classes (what the viewer never sees varied: unseen exits, members and gates, so that a hidden
  jump's draw or pair may differ); a pin test of `vary`'s new rules.
- **Inputs:** design §10, §14; T106b1's plans (`hygienic()` clears `Plan::teleporters` until
  then; `check_told`'s `jumper` already lets a held jump reveal in another's command).
  **Depends on:** T106b1 (done). **Status:** ready.

**E08 T106c — the teleporters XL, its benchmarks, the census** (round 8; design §14; decision
290)

- **Scope:** the map `e08-teleporters-144.h3m` (both levels and a sea for the Whirlpools,
  `e08-sea-144.h3m`'s generator the model: the shipped maps' most of each kind on one map, each
  channel's exits spread over the map, a monster on every hundredth free tile); the workloads
  `reach-teleporters-xl-144` (the walkers' queries meeting the entrances as stop nodes) and
  `random-play-teleporters-36` (on T106a's map) baselined at their first count;
  `--with-original-data`'s census rows not yet checked (T104 brought the channels and pairs
  forward, decision 286).
- **Inputs:** design §14; survey §1 (the census). **Depends on:** T106a (done).
  **Status:** ready (independent of T106b2).

**E08 T107 — increment 11's audit** (round 8; design §17's exit criteria)

- **Scope:** the audit against the ten exit criteria with a fresh reviewer; increment 11 closed
  (its scope, decisions 260 to 291 and tasks to `E08-archive.md`); increment 12 refined with the
  user. **Depends on:** T106b2, T106c. **Status:** queued.

## Inbox

- 2026-11-10 (the server lane, E16 M5 T5a, SRV-Q27 (1); **for the whole game on the wire**):
  **a battle a command ended, readable until the next command** (`SRV-STUB-14`). The game
  session (`h3-protocol`'s `session/game.rs`) sends each owner its battle's `Snapshot` when the
  battle opens, but cannot send the last one when it ends, as the battle session does (v38,
  CLT-STUB-3: the own hero's Eagle Eye picks, which no event shows): the ending command closes the
  battle in `h3_game`, so `GameState::battle` has none. Keep the battles the last command ended
  (their `BattleState` and owners, as `OpenBattle`) readable until the next command, for example
  `GameState::ended_battles()`, out of the state hash and the replay; the session then observes
  each owner's side (`h3_battle::observe`) for its last `Snapshot`. Tell the server lane's inbox
  when it lands.
- 2026-11-09 (the server lane, E16 M5 T3c2, SRV-Q26 and SRV-Q27; **the user's decisions**, for
  the whole game on the wire): **the fold's gaps are the stubs `SRV-STUB-1` to `-11`**
  (`docs/stubs.toml`; `-12` and `-13` are the town lane's). The server lane's fold
  (`h3-protocol`'s `wire/game/fold.rs`) keeps a viewer's view current by its projected events; its
  gate (`tests/game_fold.rs`, the 55 golden replays) asserts it equals `observe` after every
  command for every viewer but for the parts a stub names. The SRV-Q26 note below: its (1) is
  `SRV-STUB-1` (the query), (2) `-2`, (3) `-3` (its heroes; in the golden replays objects and boats
  appear with a reveal only), (4) `-4`. Beyond it (SRV-Q27 (2): what the rules derive):
  `-4` also what any visit tells (`AdventureObservation::known`, ADV-VIS-006, and
  `OwnHeroView::marks`: no event carries them); `-5` the views of an offer, a choice, a University
  and a hire dialog (their `…Opened` events open them without their view, nor tell a University's
  slots or a hire's counts after a purchase); `-6` the spell points a new day restores (no
  `SetSpellPoints` at the day's start); `-7` an added artifact's place (`HeroChange::AddArtifact`:
  a pickup, a reward, a battle's spoils); `-8` `GameEvent::LevelGained`'s levels at which Wisdom
  and a school were last offered (as `HeroChange::GainLevel` carries them); `-9` the slot where
  joined, hired or raised creatures land (`CreaturesJoined`, `CreaturesRaised`) and each hero's
  survivors at a battle's end (`BattleEnded`); `-10` what an object's new owner's team sees at
  `ObjectFlagged` (a mine's production, a Shipyard's boat tile); `-11` a Cartographer's type at
  `OfferAnswered`. Each lands as an event field or variant, which breaks `h3-protocol`'s build
  until mapped (the global trap: map it in the same `cross-lane` pull request, or ask the server
  lane first); once the fold folds it the gate fails until the stub is lifted from its part, and
  from the registry when no part and no marker names it.

- 2026-11-09 (the server lane, before E16 M5 T3c; **the user's decision**; informational): **the
  adventure's files moved from `h3-cli`'s `src/adventure/` into `h3-files`' `src/adventure/`**, yours
  (`lanes.toml`; ADR-0005's amendment): `load_rules`, `read_map`, `load_map`, the replay file
  format v8 (`replay_to_toml`, `parse_replay`), the commands' text form and `verify_file`, verbatim
  (`AdventureCliError` is `h3_files::adventure::AdventureFileError`). `h3-cli`'s `src/adventure/`
  keeps the runs, the scripts, the benchmarks and the commands and re-exports the rest; a replay
  file format change goes to `h3-files` now.

- 2026-11-09 (the town lane, E09 T18a, `town/e09-t18a`, `cross-lane`; **a kind's field and a
  version, for your matches and bumps**): `ObjectKind::Town` and `ObjectKindDef::Town` carry
  `boat_offsets: Vec<(i8, i8)>` (match `Town { .. }`; the base pack's `town = { boat_offsets =
  [[-1, 2], [1, 2]] }`, optional), **adventure ruleset part v19**; `fix_boat_tiles` fixes each
  town's spot in ADV-SHY-001's step (the town object's `Resolved::BoatTile`, in the static map's
  hash; `StaticMap::town_boat_spot(TownId)`; not in `ObjectView::boat_tile`), and the game forbids
  a town's Shipyard without one (TWN-SET-011). The 55 replays were re-recorded (only hash lines:
  the part's hash, the `towns-*` checkpoints); a pull request of yours that bumps the part again
  merges after it and re-records again. Nothing else moved.

- 2026-11-09 (the server lane, E16 M5 T1, SRV-Q26; **the user's decision**, for the whole game on
  the wire): **make the events foldable.** A server game keeps a viewer's view current by events
  only (the server lane's measurement, `docs/epics/E16-design.md` §14: diffing every viewer's
  observation costs 730 to 860 times a command's `apply` on an XL map with eight players), so a
  client must fold its snapshot with its projected events into what `h3_game::observe` gives after
  every command. What it cannot fold today (the golden replays and two eight-player XL games):
  (1) **a query for what a viewer sees on the tiles it explored since a given explored grid** (a
  `BitGrid` the server keeps per viewer): their `TileView`s, explored and fringe, and the objects,
  heroes and towns it now sees there as `observe` shows them; the server asks when the viewer's
  `explored(..).count()` grew (a step's reveal has no event, `TilesRevealed` only a count);
  (2) **a hero's movement points left** after a step or a visit (no event or `HeroChange` carries
  them; `OwnHeroView::movement` changes with every move); (3) **what comes into view without a
  reveal, with its public view**: another player's hero stepping into sight (`HeroStepped` gives
  its id and tiles, not its owner, type or boat), an object or a boat that appears (63 objects and
  9 boats in the golden replays); (4) **a visit that changes only the hero's points and the
  viewer's knowledge emits nothing**: `rewards-s162-d28` command 35, player 1's hero 3 visits the
  object of type 65 at (29, 23, 0), learns it is emptied and spends 141 points. Each lands as an
  event field or variant the server lane maps (the global trap from M5 T2 on); the server's fold
  test over the golden replays (M5 T3) checks them, each gap a stub there until it lands.

- 2026-11-09 (E08 T104, decision 285; **a performance task**): the gates' pairing
  (`state/channels.rs`' `pair_gates`) is 91.6 % of `build-gates-xl-144`'s build (898.7 M of
  981.3 M Ir for two builds of 8 116 gates, 63 ms each in release; about 17 M comparisons a build,
  about 26 Ir each: the scan visits the paired entries too). The levers: a compacted list of the
  unpaired entries per level (half the scan, no `is_some` test) or design §18's row-band search;
  both must keep the original's walk order and first-on-tie. Harmless on the shipped maps (a few
  dozen gates a map). Measure with `homm3-bench-regression`; re-baseline the workload down.

- 2026-11-07 (the town lane, E09 T13, `town/e09-t13`, docs only; **the planned `cross-lane` changes
  of E09 increment 2, construction**; design `docs/epics/E09-design.md` §34, §35, the drafts
  `docs/lanes/town/adr-drafts/*-increment-2.md`, accepted at `TWN-Q10`, in ADR-0016 and -0017 since
  E09 T15): in your paths, by task:
  T17 the game state hash (the store of towns' v2 folded in); T18a the kind `Town`'s
  `boat_offsets` in the adventure part (ruleset part bump) and each town's boat spot in
  `fix_boat_tiles` (as the town object's `Resolved::BoatTile`, the encoding unchanged); T18b
  `GameCommand::Town`, its error and events in `h3-game`'s `command.rs`, one call in `state/day.rs`
  at the day's start (the marks), the adventure replay format (the town command); T18c one call in
  `state.rs`' `start_group` after the map's timed events (the town events); T19 the fixtures, the
  golden replays re-recorded, `h3-map`'s plans (`Step::Build`). Your increment 11 plans part v18
  (T103a), game state hash v26, replay format v11 and action encoding v8 (T105b): whichever lane
  merges second bumps again and re-records (the town lane's `TRAPS.md`). Nothing to act on now.

- 2026-11-06 (the town lane, E09 T8c2, `town/e09-t8c2`, `cross-lane` with the user's consent;
  **for when E08 places Visions**): new `h3_heroes::power` (HER-ADV-001 amended: the effective
  Power beside `attack_defense`); the 47 adventure replays' `town_ruleset_hash` line moved (town
  part v3), nothing else. A Rogue's Visions for the towns is town part data (`TownParams::
  visions_factor`, `visions_min_radius`, `visions_creatures`; the packs' `[town_visions]`, the AB
  pack's Rogue; E09 decision 31). The original's same test (`hero_visions_level` 0x4E5DE0: the
  day's cast, or expert with a Rogue; `hero_visions_reach` 0x4E5E10) also feeds the hero, garrison
  and monster popups (`hero_popup` 0x416590, `garrison_popup` 0x416F80, `monster_popup` 0x417150,
  inferred in `docs/exe/map.toml`): when you place Visions or the Rogue's effect on those views,
  read the same parameters, or move them to the spell's record and the core with us (a town part
  version to bump).

- 2026-11-03 (the client lane's CLT-Q40, carried by the battle lane, BAT-Q64; `cross-lane`): the
  morale and luck sources ride on the effects (`h3_contracts::EffectSource`). `derive_battle_hero`
  now names the skill and level, the artifact and the specialty (HER-BAT-003) and gives the battle
  bonuses as the temporary source (HER-ADV-008), so battles from the map carry them already. Left
  for you: the visits by once-mark (`EffectSource::Visit { mark }`, the original's +0x105 lines)
  once the derivation knows each mark's amounts (your content), the rest staying temporary; the
  owner's Grail building is the town lane's (its inbox).

- 2026-10-31 (the town lane, E09 T0, `shared/town-lane`; **read before your next task**): the user
  opened a lane `town` (`TWN`) for E09 in parallel with E08 (TWN-Q1). It owns the crate `h3-towns`
  (to come) and the town modules inside your crates by the longest match: `h3-adventure`'s and
  `h3-game`'s `src/town*`, `h3-map`'s `src/import/town*` and `src/writer/town*`; E09's epic,
  surveys and experiments and `docs/rules/catalog/town.toml` moved to it. Your crates' common
  parts (`lib.rs`, the state structs and hashes, the versions, the replays and bot reports) stay
  yours; the town lane changes them in `cross-lane` pull requests merged in order with yours (the
  second rebases and bumps again): name your open pull requests in your HANDOFF as you do. E08's
  increments 10 to 12 stay yours (decision 4): the hire dialog and the weekly growth are built
  once, by whichever lane needs them first; tell the town lane's inbox when yours land. The
  E09 notes of your inbox and archive are gathered in the town queue's inbox; yours can leave
  them as they are. Increment 1 of E09 builds towns on the map (the import's main-town hero and
  `HeroInTown`, entering, capture by siege, the hall's income).

- 2026-10-31 (the map client lane, E24 M4 T0; **for the views, before E24 M4 T3**; your archive's
  note (3) of 2026-10-23, "refined with you at M4"): the map viewer takes the static art from the
  raw `.h3m` and the live part from the game's observation, through `h3-protocol` (the user, M4
  T0: ADR-0020 point 8 kept; the server lane maps it, its inbox the same day). It needs from
  you: (1) **an object identity:** each `ObjectView`, and each event that names an object
  (`ObjectRemoved` and the rest), naming its object in the map's list (`RawMap`'s objects, the
  order the viewer draws them in; the map is public, so its index reveals nothing the file does
  not), and an object added in play (T83b2a) an identity of its own (what it may reveal is your
  call, `docs/06-security.md`); (2) a random object's type as drawn, for every random kind the
  import supports (a random resource's is `object_type` today); (3) **MCL-Q21 (b)** (the user,
  2026-10-31): the import refuses a map whose data names nothing a hint can show, as A-31
  refuses an empty resource set: a Border Guard's, a Keymaster's Tent's or a Border Gate's
  colour past the eight, a resource's subclass past the seven, a dwelling's kind past its table,
  a seer's name past its table, a defeat-hero quest whose hero is −1 (the editor never writes
  them, a crafted map can; E24 design §4.12); tell this lane which you refuse already. Nothing
  is needed before the server lane's whole-game record exists; increment 9's views (your T83b2b,
  T85) are read by the same task.

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

- 2026-11-03 (E08 T87, decision 230; **a performance task**): the month's walk's tile test
  from the static map's flags, without a lookup of the events per tile (increment 9's design
  note §19): `month-start-xl-144` costs 12.7 M Ir a creature month's start, mostly the walk
  (`E08-log.md` T86b2). Measure against `master` (`homm3-bench-regression`).

- 2026-10-23 (T42, measured; for a performance task): **a flyer's or water walker's reach costs
  about 1.9 times a walker's query** on the XL map (`reach-lifted-xl-144`: 279 k Ir a query
  against `reach-xl-144`'s 148 k on the same games), and random play on the duel with one of each
  costs 56 % more a step (`random-play-duel-36`): `Reach::search_layered`'s second layer is the
  lever (`crates/h3-adventure/src/reach.rs`). E08 T75a: on the sea map (`random-play-sea-36`,
  one flyer and one water walker beside a 12-column sea) `Reach::expand_layered` is 52 % of the
  workload's Ir, the boats' search 4 %. E08 T75b: a boat's query on the open sea costs about
  2.29 M Ir (`reach-sea-xl-144`), 91 % in `Reach::expand_sail`; 8.7 % is `Search::landed`
  computing the land's day points (`movement::day_points`) at every landing tile, the same for
  all of a query's landings: computing it once a query is an easy cut.

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for the neutral seat**, E23 exit criterion 5): the original's battle AI for a side without a hero is complete: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (shots, melee, moves, waits, defends, the creature casts), pure, deterministic, its draws from the stream given, its choice validated (Defend if refused). The neutral seat (decision 27, your random stand-in) can call it for a monster battle's unowned side; `POLICY_VERSION` 9. A side with a hero (a computer player's) casts nothing and never retreats until E23 T5 and T6 (BAT-STUB-5).

- 2026-10-22 (ADV-Q20, decision 75; restated at ADV-Q21, decision 78, in ADR-0016's draft point 4; for the increment that revisits flying): **A-13's trade-off.** Flying stays on explored tiles (A-13 fixed for now, provisional), so wide obstacle ranges with unexplored tiles beyond them stop a flyer; the user: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway". Decide the compromise, what it tells the player of the hidden tiles, and `ADV-QUIRK-15`'s class (built by T41: the unexplored tile's row of `AdventureState::passage`, `crates/h3-adventure/src/state/passage.rs`). ~~**A-14's gap**~~ (closed by T38: the tile bit +0xC 0x40 means "no object blocks the tile", not the coast, so `hero_flight_tile_test` passes on every free land tile and a flyer visits as a walker). ~~**D-23**: Cursed Ground's movement effect is built only if batch E5's row E5-6 confirms it.~~ (E5-6 confirmed it for both grounds, T38: built by T40.)

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
