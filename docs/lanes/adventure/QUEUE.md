# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T126 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 13 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11, -11-15, -11-18; increment 13, creature banks, by T126's audit). **Increment 14, the trades and services**, refined with the user at T126 (ADV-Q56; decisions 361, 362): round 1 done: the survey (T127a1, T127a2; decision 363) and the drafts with the design note and the cut (T127b, whole: decision 369); the checkpoints ADV-Q57 and ADV-Q58 answered (decisions 365 to 368, 370 to 373). The building tasks T128 to T136 (decision 373, design note §21): T128 done (decision 374), T129a and T129b done (decisions 376, 377), T130 done (batch E14 scored; the checkpoint ADV-Q59: decisions 378, 379), T131 done (the `shared/` task), T132a1 and T132a2 done (the rates, the kinds; decisions 380, 381), T132b done (the import; decision 382), T133 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T133 — the market** (round 6; decisions 373, 382; design note §8 to §13, §17, §21;
T132b done)

- **Scope (§21's plan):** **T133a** `h3-adventure`: the Black Market's stock in the static map
  (`Resolved::Stock`) and its sold entries in the visits' block (§8; adventure state hash v27),
  the draw when the game is built (§9, ADV-MKT-006: three treasures, three minors, one major from
  the one pool, the map's bans out, never restocked), the visits' outcome of the four markets
  (§10: entered, the window's screens) and the adventure's checks and changes of a trade, a
  transfer, a sale and a purchase (§11; ADV-MKT-003 to -005, -007; A-46 fixed as ADV-QUIRK-48);
  **T133b** `h3-game`: the market's decision (§13, GAM-DEC-011; game state hash v35), what a
  viewer sees (§12, ADV-MKT-008: the stock only in its visitor's open decision) and the events
  and the wire (§17; `cross-lane`, the server lane's `h3-protocol`).
- **Inputs:** design note §8 to §13, §17, §19; survey `E08-trades.md` §2 to §8; `h3-adventure`'s
  `market.rs` (`MarketParams`, T132a1) and `ruleset/trades.rs` (T132a2); T124b1's and T124b2's
  records in `E08-log.md` as the form (a draw in the static map, a visit); `TRAPS.md` (the
  codegen units, the visits' block, the plans' oracles).
- **Depends on:** T132b's pull request. **Status:** ready; its cut confirmed at its start.

## Inbox

- 2026-11-19 (the town lane, E09 T41, `town/e09-t41`, docs only; **the town's services renumbered,
  and two of your rules a town batch played; nothing to build**): (1) at T41 (`TWN-Q30`, E09
  decision 162) the user made E09's increment 5 **the mage guild alone**; the services are
  **increment 6** (the town's Marketplace, Artifact Merchants and Freelancer's Guild on your
  market of increment 14, the Blacksmith, the town's Shipyard and the rest) and travel and the
  Grail increment 7. Where your docs and ADR-0016's Consequences say "E09 increment 5" for the
  Marketplace or the artifact sale (screen 3), read increment 6; nothing of yours waits on it.
  (2) Batch TWN-4 (survey `docs/rules/surveys/E09-tavern.md` §14, both runs) played two rules of
  your catalogs as they read: **GAM-BAT-003** (rows TWN-4-3 and TWN-4-4: a retreating, a
  surrendering and a beaten hero each left the map, its owner none) and **ADV-VIS-008** (row
  TWN-4-7: the Den of Thieves' visit showed the guild at everything, its message ADVEVENT 142).
  Both stay `implemented`; marking them `verified` with the observation is yours if you agree.

- 2026-11-19 (the map client lane, E24 M4 T3b2, `map-client/e24-m4-t3b2`; **MCL-Q37, for your
  planning; the user's answer first**): a record's hover (E24 design §5.11) lacks the cursor's
  facts the view does not hold: the selected hero's costs over several days (the cursor's day
  groups; `legal_moves` answers today's), the tiles a monster guards (ADV-MON-006: the reach knows
  them), whether another team's town or map garrison holds an army at the viewer's level 0
  (TWN-VEW-002 shows nothing), and a visited creature bank's guards (the view shows a mine's
  only). Recommended (MCL-Q37 (a)): the view (or a `Moves` answer extended to days and attacks)
  carries them, the server lane's record keeps them. Also seen: the views show no Seer's Hut,
  Quest Guard or Obelisk yet (their hint is their name). Nothing to build before the user answers
  MCL-Q37.

- 2026-11-19 (the map client lane, E24 M4 T3b1, `map-client/e24-m4-t3b1`; **MCL-Q36, for your
  planning; the user's answer first**): a whole game's record shows the panels of the original's
  adventure screen (E24 design §5.10), and two facts of a viewer's own hero are not in the view:
  its morale and luck as the status window shows them (`H3Hero::GetMoraleBonus` 0x4E39B0,
  `GetLuckBonus` 0x4E36C0: its skills, artifacts, specialty and army's alignments, outside battle;
  today derived only into a battle, `h3_heroes::derive_battle_hero`) and whether it can still move
  today (`hero_can_move` 0x4E5F30: its points against the cheapest step from its tile). The map
  client lane recommends (MCL-Q36 (a)) that the engine derives them once for the adventure (an
  `h3-heroes` or `h3-game` reader the server lane's view then carries on `OwnMapHero`); the client
  copies no rule meanwhile (it draws 0, 0 and "has points"). Nothing to build before the user
  answers MCL-Q36.

- 2026-11-19 (the client lane, E27 M3 T7 (M4 T0), `client/e27-m3-t7`; **a format extension
  for you, before E27 M4 T3**): the user's CLT-Q156 (a), E27 decision 69: the client's saves are
  the engine's adventure replay (`h3-files`' format, `h3_sim::adventure::Replay`) extended with the
  setup's choices it lacks today: per colour the human flag, the town's `FactionChoice` and the
  starting hero's pick (`PlayerStart.pick`), as `h3-battle-client`'s `begin::setup` sets them over
  the map's import; verifying builds the game from the import with them applied (absent: the
  import's, so every v19 file reads the same). The client's own fields (names, bonuses, handicaps,
  the turn's duration) stay in the client's wrapper, not in your format. The server's whole-game
  journals carry the replay too, so a lobby's choices (E28) would ride on it. E27 M4 T3 (the save
  file) needs it; should it start first, the client lane asks the user to do it cross-lane.

- 2026-11-19 (the client lane, E27 M3 T7, `client/e27-m3-t7`; **for E10's starting bonus,
  nothing to do now**): the scenario screen's starting bonus is kept as the original keeps it
  across a map change and a flag's press (CLT-Q153 (a)): only the bonus's arrows skip Artifact,
  so a colour can begin with the Artifact bonus and no hero to take it (Arrogance's single player
  on Artifact, then Rebellion, whose human row has none). The shell hands each colour's bonus
  beside the setup (`h3-battle-client`'s `begin::Game::bonuses`, `CLT-STUB-9`); when E10 builds
  the bonus, decide what Artifact gives a colour with no hero (the original's effect is not read:
  `docs/epics/E27-design.md` §3.17).

- 2026-11-19 (the map client lane, E24 M4 T3a, `map-client/e24-m4-t3a`; **your inbox's entry of
  2026-10-31 from this lane, answered in part**): (1) **the object identity is withdrawn** (the
  user, 2026-11-19): the map viewer finds each object of a view in the map's file by its type and
  anchor (the server lane's note of 2026-11-12: a view's objects come in tile order without an
  index), a random one by its anchor among the file's random objects (a random dwelling's moved
  anchor allowed), a town by its `TownId`; an object added in play (a creature month's monster)
  is drawn after the file's, its order checked against the original's at E24 M4 T4. Nothing to
  build for it. (2) **Still asked: a random artifact's drawn artifact** (classes 65 to 69):
  `ObjectView` names no artifact, so the viewer cannot draw the picture the original draws
  (`ava%04d.def` of its id, E24 design §2.7; MCL-Q3: the client the artifact's own). Its picture
  shows it to every viewer that sees the object, so the view could carry it as it carries a
  random resource's drawn pile type (an artifact's id, or a type per artifact): your call. Until
  then the viewer draws a random artifact as its random template's own picture (`MCL-STUB` to
  come with T3a). The random resources (the pile's type), monsters (`MonsterView::creature`),
  dwellings (your T94a) and towns (the town's view) are covered. (3) **MCL-Q21 (b)** stays asked:
  tell this lane which of the cases you refuse at the import (the viewer's load refuses what the
  import refuses, from T3a on). (4) **A random dwelling's drawn picture** needs its class and
  subclass (17 or 20 and the original's subclass, design §2.9): the view gives the drawn
  dwelling's type, and the table from (class, subclass) to type is `h3_map::import::codes`'
  (crate-private). A public lookup the other way (a type's key to the original's class and
  subclass, `h3_map`) would serve; or say which you prefer. For E24 M4 T3b (`MCL-STUB-1`).

- 2026-11-19 (the town lane, E09 T39b2, `town/e09-t39b2`, `cross-lane` with the user's consent, E09
  decisions 151, 154; **your paths changed, for your review; no version moved**): a Den of
  Thieves' visit sets `AdventureOutcome::thieves_guild` (a flag, `command.rs`; `state.rs`' `visit`,
  `TWN-STUB-20` lifted; `Effect::DenOfThieves`' doc in `ruleset/objects.rs`); `h3-game`'s
  `settle_outcome` tells it last through the town lane's `town::thieves::tell_den` (`state.rs`),
  as `GameEvent::ThievesGuildSeen { player, level }` then `GuildPartSeen { player, part }` per
  part (`command.rs`; the owner's alone in `observe.rs`' `project`); `h3-map`'s told views
  (`tests/view.rs`) take both as the viewer's own; ADV-VIS-008's summary. Nothing is stored (a
  sink that keeps nothing is told nothing): no hash moved. The census: the 98 goldens' 12 maps
  lay out no Den, nothing re-recorded.

- 2026-11-19 (the town lane, E09 T39b1, `town/e09-t39b1`, `cross-lane` with the user's consent, E09
  decisions 151, 152; **your paths changed, for your review; the town part v8 moved your goldens'
  `town_ruleset_hash`**): `h3_adventure::army_value` (split out of `power`, `answer.rs`) and
  `power` are public, exported from `lib.rs` (ADV-MON-007's summary a sentence); `h3_heroes::
  primary_skills` (the four effective primary skills beside `attack_defense`, HER-ADV-001's
  summary a sentence, a test in `tests/derive.rs`); `h3-game`'s `GameState::day_income`
  (`pub(crate)`, factored out of `pay_income` in `state/day.rs`, the same sums: the Thieves'
  Guild's income row reads its gold). The town part v8 (`guild_resources`) moved all 98 goldens'
  `town_ruleset_hash` 0xabc9756b93a29a55 to 0xab3ee3f1cb243f30 (nothing else: the guild changes
  no state); a replay recorded on a branch from before the merge carries the old line. Next, E09
  T39b2 adds the Den of Thieves' outcome in your `visit` (`TWN-STUB-20`), as T39a planned.

- 2026-11-18 (the town lane, E09 T39a, `town/e09-t39a`, `cross-lane` with the user's consent, E09
  decision 149; **your paths changed, for your review; map import v26, part v22, game state hash
  v34 and replay format v19 taken**): two kinds, `ObjectKind::Tavern` (ADV-VIS-007: visited from
  a neighbour, its visit `AdventureOutcome::tavern`, an `h3_adventure::TavernVisit`) and
  `ObjectKind::DenOfThieves` (ADV-VIS-008: entered, its visit nothing until E09 T39b,
  `TWN-STUB-20` in `state.rs`' `visit`), their `Effect`s, defs (`tavern = {}`,
  `den_of_thieves = {}`), the visit-mode checks and hash tags 45, 46 (`ruleset/objects.rs`,
  **adventure part v22**); the base pack's `base:tavern` and `base:den_of_thieves` appended last
  among base's types, so the AB and SoD types' ids moved by 2 (the duel's and the sea's replays'
  hashes with them); `h3-map`'s `codes.rs` types classes 95 and 97 (**map import v26**; the
  typed-classes test's cases, `tests/taverns.rs`). `h3-game`: `Decision::Tavern` (hash kind 9,
  **game state hash v34**), `Answer::Tavern(TavernAnswer)`, `GameCommandError::Tavern`,
  `GameEvent::TavernOpened`, `DecisionView::Tavern`, `GameState::tavern`, the answer's arm in
  `state.rs` (the hire is the town lane's `town::hire::map_hire`), the listing and the default
  answer in `legal.rs`. `check_stand`'s doc: a map Tavern's hire places a hero on a
  neighbour-visited tile. The replay format **v19** (`answer tavern hire <slot>|close`): the 98
  goldens re-recorded from their commands (only the version lines, the part hash, the
  checkpoints and `final_hash` moved), `h3-cli`'s literals, `h3-server`'s `tests/record.rs`; the
  bots tables' `engine.adventure` line. The plans name the kind in `settle` and `hiring` only.
  Your next bumps: import v27, part v23, game state hash v35, replay format v20.

- 2026-11-18 (the town lane, E09 T38b2, `town/e09-t38b2`, `cross-lane` with the user's consent;
  **your paths changed, for your review; replay format v18 taken**): `h3_game::Manage::Dismiss
  { hero }` (GAM-MAN-003) with `ManageError::{NotOnMap, InTown, LastHero}` (`manage.rs`),
  applied in `state/manage.rs` (`DecisionOpen` unless no decision is open) through the town
  lane's `town::leave::dismiss`; `legal_management` lists it last, outside a decision
  (`legal.rs`), so `tests/manage.rs`' and `tests/exchanges.rs`' candidates hold it.
  `AdventureState::remove_hero((player, hero))` (`state.rs`): `end_battle`'s removal (the marks,
  `HeroRemoved`, the boat) is one private `remove_at` that both use. The adventure replay format
  is **v18** (`dismiss <hero>`, a bare management word; `h3-files`' word and doc, `h3-sim`'s
  `REPLAY_FORMAT_VERSION`, the 98 goldens' `format` line only, `h3-cli`'s and `h3-sim`'s
  literals): your next bump is v19. The plans (`h3-map`'s `tests/play.rs`): `managing` leaves
  the dismissals out, `dismissal` plays one in one `Step::Manage` of sixteen, and
  `check_management`, `check_conservation` and `check_isolation` know it. No hash version moved.

- 2026-11-17 (the client lane, E27 M3 T2a, `client/e27-m3-t2ab`; **for E10's victory conditions, nothing to do now**): the original rewrites a special victory's "standard victory also" flag (`h3_map`'s `Victory::Special::allow_standard`) after reading it: `victory_standard_set` 0x4BF780 (from the conditions' read 0x4C3555 with 0, from `new_game_heroes` 0x4C0139 with 1) sets it for goal 5 (a hero's defeat) always, and for goals 0, 4 and 10 (an artifact, the Grail, a transport) clears it when exactly one colour is left undefeated (game +0x1F636), else sets it outside a campaign (a campaign by its ids); the other goals keep the file's flag. The shipped maps: goals 0, 4, 10 all carry the flag, the five maps of goal 5 all lack it, and the scenario screen shows them "or Defeat All Enemies" (recorded, `scn-info`). VCMI 1.7.5 (`MapFormatH3M.cpp`) forces goal 5 too ("H3 behavior") and clears the flag for any goal on a map of one player. The client applies the screen's rule to its list only (`h3-battle-client`'s `maps::STANDARD_ALWAYS`); the game's rule is E10's (design note `docs/epics/E27-design.md` §21.1).

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
