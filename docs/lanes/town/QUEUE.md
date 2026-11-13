# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 3: growth, recruitment and the army rows, refined at
T20, `TWN-Q11`, decisions 64 to 67); done tasks in `E09-log.md`; increments 1 and 2 (closed by T10
and T20) in `E09-archive.md`. Increment 3's first round is done: the survey T21 (`TWN-Q12`), the
drafts T22a and T22b with Part E §37 to §58 and the cut (`TWN-Q13`, `TWN-Q14`; decisions 74 to 83;
the drafts accepted), batch TWN-3 (T23, decision 84), the `shared/` task (T24, decision 85: the
ADRs, the stream, the effects and flags); the build's T25a (decision 86), T25b1 (decision 87),
T25b2 (decision 88), T26a1 (decision 89) and T26a2 (decision 90). The building rounds follow the cut of design note §56
(decision 83): the build, E08's increment 12 merging in
parallel (each `cross-lane` number the next at merge). Each task's cut confirmed at its start;
every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T26 — growth in the game** (decision 83; design note §56 round 5, `cross-lane`; T26a in two,
decision 89; **T26a1** and **T26a2** done, decision 90, `E09-log.md`): **T26b** the neutral
garrison week (T25b2's `garrison_week`, written by `AdventureState::set_town_garrison` in
`h3-adventure`'s `src/town.rs`, the adventure part's army rules passed), the Portal's draws
(T25b2's `summon`: each week, at its build, a flagged dwelling, a capture, decisions 71 and 82,
each from its stream: the week's from the towns' stream `GameState` holds since T26a2), the views
(§47: the owner's counts and Portal; `TownChanged` told after the week, the month and an event's
creatures; the week's kind is told since T26a2) and the protocol (`cross-lane`, the server lane's
fold gate; v53 or the next at merge), the replays re-recorded (skill `homm3-rerecord-replays`),
the bench (skill `homm3-bench-regression`). Each part's rule ids, sources and tests; the merge
order with E08's increment 12 (each `cross-lane` number the next at merge). **At its start:**
confirm the cut. **Depends on:** T26a2 (its pull request merged). **Status:** next.

**E09 T27 to T30 — the rest of the build** (decision 83; design note §56 rounds 6 to 8, each cut
confirmed at its start, the agent queues each in full when the one before it ends): **T27**
recruitment (`cross-lane`; §49 to §51, the env's v10); **T28** the rows (`cross-lane`, after E08's
T114a and T115a merged; T28a §52, §53's record and moves, the env's v11; T28b §53's day and
income, §54); **T29** the gates (§55, §47); **T30** the audit, the closure, increment 4 refined.
**Depends on:** T26. **Status:** waiting.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-13 (the adventure lane, E08 T114b, `adventure/e08-t114b`, `cross-lane` with the user's
  consent; **your paths changed, for your review**): `h3-game`'s `town/view.rs`: `visions`,
  `in_visions`, `tavern_level` and `garrison_view` are `pub(crate)` (the map garrisons' views use
  the same levels, ADV-GAR-006, `crate::garrisons`), and `Told` holds the map garrisons' views of
  `every`'s players too (`crate::garrisons::Told::before` with `every`, told after the towns'),
  and its `tell` returns at once for a sink that keeps nothing (`bench-check`'s
  `day-start-builds-36`; `day-start-builds-36` and `-towns-36` re-baselined, `master` +1.68 % and
  +1.85 % on their files after your T26a2):
  a Tavern's build and a town's capture tell the raised or lowered views of the map garrisons as
  of the towns. `town/tests/view.rs` gains
  `a_map_garrison_shows_at_a_towns_levels_and_a_new_tavern_tells_its_raised_level` (its helper
  `map_garrison`). `TWN-STUB-13`'s text names the map garrisons too (Visions' reach untold for
  them as for towns). `h3_adventure::AdventureOutcome` gains `garrison_flagged` (default false).
  No version of yours moved; protocol v54 (the server lane's, after your v53).

- 2026-11-12 (the adventure lane, E08 T114a2, `adventure/e08-t114a2`, `cross-lane`; **a version
  taken and the hook for your T28a**): game state hash **v29** (a map garrison's exchange among the
  decisions, tag 8), so your T26a2's bump is v30 (done, #899); adventure replay format **v13** (`m<index>:<slot>`,
  a map garrison's army) and protocol **v52** (the server lane's, `cross-lane`); every adventure
  golden replay re-recorded (versions and hash lines only). The one owner rule is in
  `h3-adventure`'s `state/manage.rs`: `held` gives each army with its bar ownership (a hero's and
  the group the player's, a map garrison its owner's when removable, nobody's when not), so your
  `ArmyRef::Garrison(town)` gets its arm there (its town's owner) and `Held::keeps_last` stays false
  for it; `h3-game`'s `Together::Exchange { hero, army }` holds a stationary `ArmyRef` (a town's
  garrison may be its `army`), and `Together::admits` refuses a map garrison its decision does not
  hold (`DecisionOpen`; alone `NotTogether`). Design note §13's *As built* (`E08-design.md`).

- 2026-11-12 (the server lane, E16 M5 part 2's refinement, SRV-Q29; **the user's decision,
  informational**): in increment 2c the server lane builds `Viewer::Spectator` (public facts only)
  through `h3-towns`' projections too (a town's view at a spectator's level), in a `cross-lane`
  pull request with the user's consent, asking your lane's review; nothing for you to build. Your
  two notes of 2026-11-10 (`town_offer`, `town_events_at_start`) go to E16 M5 T11.

- 2026-11-12 (the adventure lane, E08 T110, `adventure/e08-t110`; the capture harness, informational, for your next batch): `h3drive.gold_bonus` now clicks each row's arrow until the memory reads Gold (a heroless town player has no Artifact in its row); `hint_at` parks the mouse and reads until two reads agree; `play_turns` gives no order once the battle is decided; a recipe's `+anything odd` is kept beside the notes. New steps and reads: `clients/capture/README.md`, "Garrisons and the bonus visitables".

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
- 2026-11-05 (the adventure lane, E08 T95b, `adventure/e08-t95b`; **for a town's recruitment,
  E09 decision 4**): the hire dialog is built to be reused (increment 10's design note §13,
  "E09's reuse"). `h3_adventure::HireDialog { target: HireTarget, source: HireSource }`: extend
  `HireSource` with a town's dwelling and `HireTarget` with the garrison; the map's purchase rule
  stays one (`AdventureState::hire_stock`, `hire_room`, `check_hire` → `HirePurchase`, `hire`;
  ADV-HIR-002), and so does the game's decision (`h3-game`'s `state/hire.rs`: `Decision::Hire`,
  `Answer::Hire(HireAnswer::{Buy, Close})`, the legal set of each creature's most, `hire_range`,
  `DecisionView::Hire`, `GameState::hire_dialog`; GAM-DEC-008). The env's four purchases (action
  encoding v7) take the source's order, so a town's dwelling of up to four creatures needs no
  new entry. Game state hash v25 is T95b's.
- 2026-11-04 (the adventure lane, E08 T94b, `adventure/e08-t94b`; **for the towns' external
  dwellings' growth**; its versions half done by E09 T8a1, v21): A dwelling's counts and guards as
  they stand:
  `AdventureState::dwelling_counts` and `dwelling_guards` (by object index; none of a block while
  every dwelling is as its week began); the week's refill of every dwelling is the first step of
  `AdventureState::start_week`, before the weekly objects and the camps, reading no week's or
  month's kind (ADV-DWL-003; your towns' growth is separate, survey `E08-dwellings.md` §6.1).
- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **for the towns' external
  dwellings' growth**, E09 survey §8.3; adventure design note §11): the static map lists the
  dwellings in object order (`StaticMap::dwellings`, `dwelling_index`; a random dwelling not tied
  to a town is resolved into its drawn faction's and level's dwelling when the game is built, so
  it is listed as a placed one; since E09 T11 one tied to a town too), their owners are `AdventureState::owner` (ownable since T93a, the
  setup's owner at the start) and their creatures the kind's (`ObjectKind::Dwelling::creatures`,
  each creature's faction the core's). Their counts and guards come with T94b (adventure state hash
  v20). A random dwelling tied to a town resolves since E09 T11, after the towns' draws (ADR-0017
  point 4).
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
- 2026-11-03 (the adventure lane, E08 T87c, ADV-Q41; **for your increment 3's recruitment, nothing
  to build now**): E08's increment 10 is dwellings and hiring (decision 231): the map's creature
  dwellings (17, 20), the random dwellings with a faction set and the Refugee Camp, their weekly
  creatures and **the hire dialog**, built in the adventure lane first (E09 decision 4); the random
  dwellings tied to a random town resolve since E09 T11. Its survey (T88a,
  `docs/rules/surveys/E08-dwellings.md`) and design note (T88b) come first; this lane's inbox is
  told when the hire dialog lands.
- (adventure archive, 2026-10-03, E04a T3a) Not harvested: the spells' town chances and the hero
  classes' tavern weights (RoE lacks the Conflux column: an override kind or a per-faction table).
  Increment 4, the Tavern (weights: only the tavern's draw reads them, survey `E09-towns.md`
  §4.4), increment 5 (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 4, the Tavern.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 4, the Tavern, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 5.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 5.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 5.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 5.
