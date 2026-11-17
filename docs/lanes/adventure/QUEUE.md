# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T117 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 12 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11, -11-15; increment 12, garrisons and the bonus visitables, by T117's audit). **Increment 13, creature banks**, refined with the user at T117 (ADV-Q52; decisions 333, 334): round 1, the survey (T118a, done 2026-11-15: decisions 335 to 339, its checkpoint ADV-Q53) and the ADR amendment drafts (T118b1 and T118b2 done 2026-11-15, decisions 340 to 342; the checkpoint ADV-Q54, decisions 343 to 345: the note, the drafts and the cut T119 to T126 accepted). Round 2: T119 done 2026-11-15 (`adventure/e08-t119`, `cross-lane`, whole: decision 346; the writer's banks, the spec's pictures and `slots`, batch E13's spec). Round 3: T120 done 2026-11-16 (`adventure/e08-t120`, alone: decision 347; the harness's reads, the recipe, three runs, the template filled); T121 done 2026-11-16 (`adventure/e08-t121`, whole, docs only: survey `E08-banks.md` §14, every row as predicted; the checkpoint ADV-Q55, decisions 348, 349). Round 4: T122 done 2026-11-16 (`shared/e08-t122`, `cross-lane`, whole: decision 350; the three drafts in ADR-0013, -0016, -0017, -0022 with batch E13's rows, the glossary, the security lines, `BattleSetup::creature_bank`). Round 5: T123a done 2026-11-16 (`adventure/e08-t123a`, `cross-lane`, alone: decision 351; the kind, the packs, the seventh check; part v21); T123b done 2026-11-16 (`adventure/e08-t123b`, `cross-lane`, whole: decision 352; the import, the census, map import v24). Round 6: T124 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T124 — the bank battle; the state, the draw, the visit, the battle and the reward** (round 6;
design `E08-design.md` §4 to §8, §12 to §15, §18, §20)

- **Scope:** T124a (`adventure/e08-t124a`, `cross-lane`, first, the battle lane reviewing): the
  bank battle in `h3-battle` with `BattleSetup::creature_bank` (§13: no tactics phase, no
  obstacles and a setup refusing them or a siege beside the flag, no war machines on the field and
  the Ammo Cart's supply kept, the layout by `bank_positions`; BAT-SET-001, BAT-TAC-001,
  BAT-GEO-004, BAT-WAR-001, BAT-WAR-004 amended), the battle pack's `bank_positions` (pack format
  v53, shared: likely a `shared/` part first), battle state hash v56, battle part v8, the battle's
  properties, golden replays and bench. T124b (`adventure/e08-t124b`): the state and the draw (§4,
  §5; adventure state hash v26, ADV-BNK-002), the visit and its offers (§6, ADV-BNK-003, D-66),
  the battle's two halves (§7, §14: `Defender::Bank`, the flag, the deck; ADV-BNK-004,
  GAM-BAT-001), the reward as a won battle's rest (§8, §15; ADV-BNK-005, GAM-DEC-007), the
  once-marks (§12, HER-ADV-008). Each part's cut confirmed at its start; T124a merged before
  T124b's tests.
- **Inputs:** design §4 to §8, §12 to §15, §18's versions (the battle lane's E26 and E11 may have
  taken the battle's numbers: bump past theirs); survey `E08-banks.md` §3 to §6, §14 (batch E13's
  rows); ADR-0022 point 8, ADR-0016 and -0017's increment 13 parts; the battle lane's `TRAPS.md`
  and HANDOFF (its open pull requests) before touching `h3-battle`.
- **Depends on:** T123b (`adventure/e08-t123b`). **Status:** ready once T123b merges; T124a needs
  the battle lane's state checked first (its queue's "War machines", r3).

## Inbox

- 2026-11-17 (the town lane, E09 T37a, `town/e09-t37a`, `cross-lane` with the user's consent;
  **your paths changed, for your review**; design note `E09-design.md` §61 to §63, §68, §75 As
  built): **game state hash v33** (the taverns' stream after the towns'; the pool's offered status,
  records, away records, factions and tavern slots). `state.rs`: `taverns_rng`, `pool()` and
  `taverns()` (the tavern parts' borrows), the start's fill in `GameState::new` before the first
  group's turn, `town::tavern::start_turn` after the adventure's turn start, the marks' clear at
  `EndTurn`; `state/day.rs`: the week's fill after `grow_week`, before your week step, the marks'
  clear before the towns' day, `regain_mana` skipping a tavern record the pool marks full
  (`Away::full`: `day_mana` never changes points at the maximum); `state/levels.rs`: the seeds and
  sexes a `Vec`, `Progression::record`; `setup.rs`: `hero_details` takes a `HeroDetailsSetup`,
  `starting_army` and `wear_machines` split out, the records and factions kept after the main towns'
  heroes; `command.rs`/`observe.rs`: `GameEvent::TavernOffered`, `TavernPartSeen`,
  `GameObservation::tavern`; `h3-heroes`: `backpack_equipped` (ADR-0013 point 14.5), `first_free`
  and `fits` without allocating the positions; `h3-map`'s told-views property folds the tavern
  heroes. The 89 golden replays re-recorded (only hash lines moved); the week-start benches
  re-baselined (the week's fill). **When you write a tavern hero's record** (none of yours does
  today): clear its `Away::full` and set `Away::stale`, or the day's regain and the turn's points
  skip it.

- 2026-11-17 (the town lane, E09 T36, `town/e09-t36`, `cross-lane` with the user's consent; **your replays' header, mechanical**): the town ruleset part v7 (the records' tavern weights, the hire's price) moved `town_ruleset_hash` 0x0d7be940b64312a6 to 0xabc9756b93a29a55 in all 89 replays (re-recorded from their commands, nothing else moved). As with T35b's note below: a replay recorded on a branch from before the merge carries the old line. `TownRecord::tavern` and `TownParams::hire_price` are read by no rule yet (E09 T37a, T38a).

- 2026-11-17 (the town lane, E09 T35b, `shared/e09-t35b`, `cross-lane` with the user's consent; **your replays' header, mechanical**): two `quirk.town` flags (`QuirkFlag(42)`, `(43)`, on in `faithful`) moved the base faithful ruleset hash 0xb5c01c8736eb73f9 to 0x6ba8109889b34a25; your 86 replays on it keep commands and state hashes, only `ruleset_hash` moved (the three strict-fixed ones unchanged). A replay you record on a branch from before the merge carries the old line: re-record it or patch the line after merging `master`. `StreamId::TAVERNS` (8) is new, no draw yet (E09 T37a draws from it).

- 2026-11-16 (the town lane, E09 T35a, `shared/e09-t35a`, docs only; **ADRs in your paths, for your planning**): E09 increment 4's drafts are in the ADRs, each part after E09 increment 3's (so before your increment 10 to 13 parts where the points interleave): ADR-0016 points 3 (a hire's placement with the game's away record, the leave's hand-back, the dismissal's removal), 4 (the kinds `Tavern` and `DenOfThieves`, after E09's kind `Town`) and 6; ADR-0017 points 1 to 4, 6, 8 to 11; ADR-0013 points 14.5 and 16.5; each Consequences and Alternatives a line. Two decisions at T35's start (`TWN-Q19`, E09 decisions 126, 127), both for the town lane's T37a in your `state/day.rs` (`cross-lane` then): every day's start clears every player's tavern returning marks after the taverns' week (the original's `new_day` 0x4C80CF to 0x4C80FD), and `GameState::regain_mana` stays over every record of the store, tavern heroes and heroes out of the game included (HER-MAN-002 as you built it; the design's planned exception dropped). ADR-0017 point 9 and ADR-0028 point 9 name no adventure state hash bump for increment 4 (design §77: the kinds hold no state). Nothing for you to build now.

- 2026-11-16 (the town lane, E09 T34, `town/e09-t34`, `cross-lane` with the user's consent; **the capture harness and your writer**): `clients/capture/xdd.c` logs three more sites (`tavern_class`, `tavern_type`, `leave_slot`, with their ranges; `h3drive.LOGGED` grew by three), so a sequence or batch run with an `xdd.dll` built before it fails the count of redirected calls: rebuild it with `clients/capture/setup.sh`. Also new and common: the reads and actions of README "Taverns and the Thieves' Guild" (the tavern dialog, the guild window, a town opened from the town list, a step's `resources`, `mana`, `narrow_pool`, a field `<encounter>|<field>`), and `auto_combat` returns when the battle ended before a human stack's turn (a loss to faster monsters kept pressing `a` until the limit). In your `h3_map::writer`: `TAVERN_OBJECT` (95) and `DEN_OF_THIEVES_OBJECT` (97) joined `PLAIN_OBJECTS` (no body; `tests/taverns.rs`): merge in order if you touch the list (the import of both stays the town lane's T39a).

- 2026-11-16 (the town lane, E09 T33b, `town/e09-t33b`, docs only; **planned `cross-lane` changes
  in your paths, for your planning; nothing changed yet**): E09 increment 4's drafts are accepted
  (`TWN-Q18`; `docs/lanes/town/adr-drafts/0016-`, `0017-increment-4.md`; the building tasks T34 to
  T41 in `docs/epics/E09-design.md` §79, the order against your increment 13 there). In your
  paths, each `cross-lane` with the user's consent at its task: T35 amends ADR-0016 and -0017
  (merge order with your T122); T37a the game state hash (the pool's `Offered`, records and away
  records, the players' factions and tavern slots, `StreamId::TAVERNS`) and the week's step (the
  taverns' week after the towns'); T37b `GameSetup`'s customised hero types and the import (after
  your T123's map import v24); T38a the adventure's placement of a hired hero (ADR-0016 point 3:
  a town's entrance or a map Tavern's visited tile, with its away record; `HeroHired`), `legal.rs`,
  the replay format (`town hire`); T38b a hero that leaves the game composed after your battle's
  end in `state/battle.rs` (merge order with your T124b), the dismissal `Manage::Dismiss`
  (`manage dismiss`); T39a the kinds `Tavern` (class 95, visited from a neighbour) and
  `DenOfThieves` (class 97, entered; every visit tells the game's Thieves' Guild) in the adventure
  part, the import and the writer (after your T123); T39b `h3_adventure`'s `answer::power`
  (ADV-MON-007) made public for the guild's army row and best hero, and the day's gold factored
  out of `GameState::pay_income`. T34's fixture needs the writer's classes 95 and 97 (as your
  T119's banks).

- 2026-11-15 (the town lane, E09 T31, `town/e09-t31`, `cross-lane` with the user's consent, E09
  decision 101; **your paths changed, for your review; no version moved**): `command.rs`'s
  `AdventureEventKind::GarrisonEntered` and `GarrisonLeft` have a field `named: Viewers`, the
  viewers told the move by name (the owner's team and, from the game, a viewer whose view of the
  town is at level 3 by Visions); `observe.rs`' `AdventureEvent::project` tells any other viewer
  `HeroRemoved` at the entrance and `HeroStepped { from: None, to: Some(entrance), .. }` (a hero
  coming into sight) in their place, as a step's unseen ends are left out. `h3-map`'s
  `tests/view.rs`: `views_move_as_the_events_say` checks each viewer's projection of every garrison
  move (`check_garrison_moves`, beside `check_jump_ends`). No hash, replay or format moved (events
  are not hashed). A new constructor of either event passes `named` (`Viewers::NONE` names the
  owner alone: `move_town_hero` adds it).

- 2026-11-15 (the town lane, E09 T29b, `town/e09-t29b`, `cross-lane` with the user's consent, E09
  decision 97; **your paths changed, for your review; no version moved, no engine code**):
  `h3-map`'s `play.rs`: `Step::Town` (`visit_town`) gives a listed `TownCommand::Heroes` move at
  an own town in five steps of eight, so every property's and cover test's games with town runs
  moved (all of `h3-map`'s tests pass, the properties also at 240 cases); `town_events` gives
  65 535 of a level where it drew 20; `check_invariants` bounds the towns' counts;
  `check_towns` takes the command (the Plague, a purchase's amount); a new property
  `a_forks_week_is_its_parents`; `hero_anywhere` (a hero on the map or a garrison hero) in
  `check_battle_end`, `check_heroes_after_battle`, `check_conservation` and `check_progression`,
  and `check_isolation` exempting the battle's garrison hero, which met a garrison hero
  defending, raising and levelling up. `view.rs`' `vary` keeps a map garrison's entrance land (a case
  of `master`'s, `OwnedOnWater`; your E08 T116b1's ownable garrisons were not in its
  `entrances`). A new workload `benches/adventure/month-start-builds-36.toml` (the step that
  starts day 29 on the builds map), its README row by #942.

- 2026-11-13 (the client lane, E27 M1 T3, `client/e27-m1-t3`; **for E10's victory conditions,
  nothing to do before it**): the original's "defeat a hero" victory (code 5,
  `scenario_victory_test` 0x4F15E0; `docs/epics/E27-design.md` §5.2) counts only when the viewing
  player's team defeats the hero: another team's defeat of it shows nothing and ends nothing, and
  the condition can then never be met (the hero is gone). E10 decides the engine's rule (copy, or
  a fix such as a loss or a fallback to the standard victory); the shell shows whatever the engine
  reports. **And the won scenario's score is the engine's** (the user, CLT-Q105, E27 decision 21):
  E10 computes it (a rule id and its test; design §5.5: the base 200 − (days + 10) / (towns + 5)
  by integer division, + 25 when the player dug up the Grail, + 25 for a victory by conquest, times
  the difficulty's `f32` factor 0.8, 1, 1.3, 1.6 or 2, truncated; the rank's creature is the
  shell's) and reports it with the victory; the shell needs the day, the
  difficulty and the score, and stubs it until then.

- 2026-11-12 (the client lane, E27 M1 T1b3b, `client/e27-m1-t1b3b`; **a correction for your
  survey, and E10's needs**): 0x6972B8 is the scenario's end flag (`scenario_ended`, read: set
  with `scenario_result` 0x699510 by `scenario_condition_met` and the conditions' tests; the
  adventure manager's procedure then ends play), not a network client's: `E08-economy.md` §7.2's
  "a network client skips both" is `new_day` skipping the week and the month once the scenario
  has ended (`docs/exe/map.toml` renamed). For E10's refinement: the shell's messages and score
  (`E27-design.md` §5.1, §5.2, §5.5) need from the engine per colour its elimination and why (no
  hero and no town; the week without a town and its warning), the shell player's result with the
  condition's code (or the standard victory) and the values its text takes (the artifact, the
  count and creature, the amount and resource, the hero, the town), and for the score the day,
  the player's towns, the difficulty and whether that player dug up the Grail.
  *(E08 T117: the survey's correction made, `E08-economy.md` §7.2; E10's needs stay.)*

- 2026-11-12 (the client lane, E27 M0, branch `client/e27-m0`; **for E10's refinement**, the
  user's decision CLT-Q81): the game shell (E27) fills `h3_game::GameSetup` from the scenario
  screen and builds the original's setup and end screens before the engine has their rules; the
  user (CLT-Q79): **E10 adds what is missing to `GameSetup`**, the shell stubs it until then (no
  engine change ahead of E10). What the shell will need: a player's **starting bonus** (the
  original's gold, resource or artifact choice, or random) and its draw; the **handicap**; the
  **victory and defeat** conditions and a scenario's end (the end screens, the high score's
  numbers); later the computer players' turns (E13). Its readings of the original's setup (E27
  M1 T1b, `docs/epics/E27-design.md` §3) map each choice to a `GameSetup` field or to one of
  these gaps (T1b2, §3.9, §3.14, §3.15: the bonus's offer, no Artifact for a colour without a
  hero, no Resource under a Random town, the amounts the popups state, the handicap's three
  levels into game +0x1F6A8); nothing to do before E10, the order is yours. Also (CLT-Q83): a saved game is your
  adventure record (`h3-files`' replay format); what it lacks of a setup's choices is settled
  with you at E27 M4's T0.

- 2026-11-12 (the client lane, E17 M7 T4, at E17's closure; **the harness check's growth, yours
  to take when a batch needs it**, placed by the user): the harness's own check
  (`clients/capture/checks/harness-check.toml`, its recipe `capture/batches/harness-check.toml`)
  can gain what your tools placed since E08 T22a1: a monster's offers (join, join for gold, flee:
  `[encounter.opponent]`'s `disposition`, `flees`), a guarded pile's question (`[[map.pile]]`'s
  `guards`, `question`) and a walk on level 1 (`[map.underground]`); H1 checked those only on a
  local map, H2 the underground on the shipped `Arrogance`. The client lane's notes:
  `docs/lanes/client/NOTES.md`, "The adventure lane's harness".

- 2026-11-09 (the server lane, E16 M5 T3c2, SRV-Q26 and SRV-Q27; **the user's decisions**, for
  the whole game on the wire): **the fold's gaps are the stubs `SRV-STUB-1` to `-11`**
  (`docs/stubs.toml`; `-12` and `-13` are the town lane's). The server lane's fold
  (`h3-protocol`'s `wire/game/fold.rs`) keeps a viewer's view current by its projected events; its
  gate (`tests/game_fold.rs`, the 55 golden replays) asserts it equals `observe` after every
  command for every viewer but for the parts a stub names. The SRV-Q26 note below: its (1) is
  `SRV-STUB-1` (the query; landed 2026-11-11, `h3_game::reveal`, the server lane's T5c lifts the
  stub), ~~(2) `-2`~~ (lifted 2026-11-11: `MovementChanged`, protocol v42), (3) `-3` (its heroes; in the golden replays objects and boats
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

- 2026-11-09 (the server lane, E16 M5 T1, SRV-Q26; **the user's decision**, for the whole game on
  the wire): **make the events foldable.** A server game keeps a viewer's view current by events
  only (the server lane's measurement, `docs/epics/E16-design.md` §14: diffing every viewer's
  observation costs 730 to 860 times a command's `apply` on an XL map with eight players), so a
  client must fold its snapshot with its projected events into what `h3_game::observe` gives after
  every command. What it cannot fold today (the golden replays and two eight-player XL games):
  ~~(1) **a query for what a viewer sees on the tiles it explored since a given explored grid** (a
  `BitGrid` the server keeps per viewer): their `TileView`s, explored and fringe, and the objects,
  heroes and towns it now sees there as `observe` shows them; the server asks when the viewer's
  `explored(..).count()` grew (a step's reveal has no event, `TilesRevealed` only a count)~~ (done
  2026-11-11, `adventure/srv-q26-reveal`: `h3_game::reveal`, the server lane's inbox told);
  ~~(2) **a hero's movement points left** after a step or a visit (no event or `HeroChange` carries
  them; `OwnHeroView::movement` changes with every move)~~ (done 2026-11-11: `MovementChanged`,
  told last to the owner for every hero a command changed, mapped at protocol v42); (3) **what comes into view without a
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
  *(E08 T117: a flight passes over a garrison's entrance, its own team's or another's, as over any entered object's tile, ADV-MOV-009, ADV-GAR-004; batch E12 flew over none: the same increment checks it.)*

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
