# Town lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Working in the adventure lane's crates

- **Never build `xtask` of another worktree into the main clone's target dir** (E09 T26a1): the
  binary keeps its worktree's root, so a later `cargo xtask` in the clone runs every step
  (verify, bench-check, check-docs) on the other tree without a word. Run a worktree's `xtask`
  with its own target dir; if it happened, `cargo clean -p xtask` in the clone.

- **A worktree's `original-data` link gets committed** (E09 T32): `.gitignore`'s `/original-data/`
  matches a directory, not the symlink a worktree uses to share the main clone's staged files, so
  `git add -A` adds the link. Put `original-data` (no slash) into `.git/info/exclude` (shared by
  every worktree) before linking, and check `git ls-files original-data` before a push.

- **Ownership is by the longest match** (`lanes.toml`): only files whose path starts with `crates/h3-adventure/src/town`, `crates/h3-game/src/town`, `crates/h3-map/src/import/town` or `crates/h3-map/src/writer/town` are this lane's. A `mod town;` line in a `lib.rs`, a field in a state struct, a hash, a version constant, a replay or a bot report is the adventure lane's: such a pull request is `cross-lane` (the label at once), and `cargo xtask check-lane --cross-lane` is its local check.
- **Merge in order:** before a task starts and before a pull request opens, look at the adventure lane's open pull requests (its HANDOFF). When both lanes bump the same version, the second rebases and bumps again, and re-records the replays a second time (skill `homm3-rerecord-replays`).
- **The adventure lane's traps apply** to its crates (forks without deep copies, server-only RNG seeds, template masks, staging the shipped maps): its `TRAPS.md`.
- **The scoped verify** `cargo xtask verify --scope town --summary` also runs the adventure lane's crates, replays and benchmark workloads (`verify_with`); its `check-lane` step fails by design on a `cross-lane` branch. It does not run the other lanes' tests (`h3-cli`'s ai census, E09 T8a1's CI failure): a change to the setup's checks or the static map also runs `cargo test --workspace --exclude xtask --all-targets`.
- **A new game command, answer or event variant breaks matches in other lanes' crates** (E09
  T18b1): `h3-map`'s property tests (`play.rs`' `as_move`, `view.rs`' told views), `h3-sim`'s
  action entries, `h3-files`' replay words; the scoped verify builds `h3-map`'s tests only in its
  clippy and coverage steps. Run `cargo check --workspace --all-targets` right after adding one.
- **A new command on the wire takes its replay word in the same pull request** (E09 T38a1):
  `h3-server` journals every command the wire accepts in the replay text
  (`h3_files::adventure::format_command`, `journal/whole.rs`), so the word (`h3-files`) and the
  adventure replay format's bump (`h3-sim`'s `REPLAY_FORMAT_VERSION`, the goldens' `format` line,
  `h3-cli`'s and `h3-sim`'s literals) go with the command's variant, whether or not it is listed.
- **The town scope's coverage does not gate `h3-protocol`** (E09 T38a1's CI: two regions of a
  new fold, an `if let`'s untaken path and an `if` that never failed): after mapping or folding a
  new wire type, run `target/debug/xtask verify-step coverage --scope server` (after `source
  scripts/llvm-tools.sh && llvm_tools_env`; about 10 minutes) before the pull request.
- **A new town command in the legal set changes every random game with an owned town** (E09
  T19c1, T27): the env's agent draws it, so `h3-sim`'s town-map tests (`tests/towns.rs`: the
  play's seed, the fork's siege seed) and the `*-towns-36` workloads move. Search seeds with a
  throwaway `#[ignore]` test in release, its `CARGO_TARGET_DIR` outside the worktree: only
  `/target` is ignored, and `git add -A` took a `target-rel/` (E09 T27). Since the hires (E09
  T38a2, encoding v12) every game hires 10 to 16 heroes and spends its gold on them: a siege
  against a visitor with the garrison merged met 8 games of 399; search a few thousand seeds
  (400 games of 21 days about 50 s in release, without the masks' checks).
- **A new golden adventure replay moves three counts in the server lane's tests** (E09 T29a's
  CI): `h3-protocol`'s `tests/game_fold.rs` (`golden`) and `tests/game_session.rs` (the
  restores), `h3-server`'s `tests/record.rs`; their fold, query and session gates also play every
  replay. The scoped verify runs neither crate: after adding a replay run `cargo test
  --no-fail-fast -p h3-protocol -p h3-server` (CI's test run stops at the first failing binary).
- **The scoped verify's clippy skips `h3-sim`** (E09 T27's CI: a one-element `for` in a town-map
  test): after a change there run `cargo clippy --workspace --all-targets -- -D warnings`.
- **A hire dialog's creatures for the env come from `GameState::hire_stock`** (E09 T27): a new
  source served elsewhere leaves its `Buy`s without an entry, and the random agent falls back on
  `EndTurn`, which the game refuses (`DecisionOpen`).
- **A listing is folded, not collected** (E09 T38a2): `legal_commands` pushes its chain into a
  list sized 16 with `for_each`, which runs each part's own fold; `collect()` went through the
  nested `Chain`'s `next` and the list's regrowths, which the hires' longer lists made cost
  `legal-towns-36` +17 % on the same games (+4.6 % folded; a size of 8 was worse). A test that
  does not depend on the town (the price, the room on the map) runs once a listing
  (`town::hire::hire_slots`), the town's own (the entrance, the Tavern) after it.
- **A town's entrance from its template is dear in a listing** (E09 T28a): `MapObject::entrance`
  walks the visitable mask's offsets; three checks a town a listing cost `legal-towns-36` +56 %.
  Read `StaticMap::town_entrance` (cached at the build) and find a visitor by comparing the
  player's heroes' tiles with it; `StaticMap::town_at` for every hero costs too (+26 %).
- **A walk over the towns' armies at the day's start is dear** (E09 T28b1): the week's
  resources over the upper bars' stacks as a `Chain` of `Flatten`s into a `BTreeSet` cost
  `day-start-builds-36` +7.9 % and `day-start-towns-36` +5 %; noting only the creature types
  with an ability in a short `Vec` (most stacks pass over with one lookup) left +0.2 %.
- **A hero's maximum spell points are dear at the day's start** (E09 T49a1): the night's refill
  as its own pass after `regain_mana` (`max_mana` again for every refilled hero) cost
  `day-start-builds-36` +5.4 %; the regain now takes the refilled heroes
  (`town::guild::refilled`) and computes their points once. A new day-start rule on a hero's
  points goes into that walk too.
- **A listing's test of a town's guild is dear** (E09 T49a2, the opening): asked of every owned
  town a hero stands in at every listing, `vortex` or `teaches` hero by hero cost
  `legal-towns-36` +49 % on `master`'s games (no opening listed): `guild_level` walked the
  standing set once a level, `can_learn_spell` built and dropped a `BTreeSet` of the hero's
  effects per spell, `vortex` derived the standing set in every town (the week's flag is set in
  every town). `h3_towns::opening_gives` answers the listing alone: the part's derived
  `GuildRoles` per record (unhashed, as the Portals: the roles' sets and the buildings that can
  supersede a guild or a Vortex, the only ones walked), the shown spells as one `SpellSet`, each
  hero's known spells taken first, its book and Wisdom read once (`h3_heroes::spell_learner`);
  and `legal_builds` finds the hero on a town's entrance once for the hires, the opening and the
  moves (`AdventureState::town_entrance_hero`). Left: +5.3 %, the rule's own cost (about 470 Ir a
  listing). A new listed town command shares that lookup and reads derived roles.

- **CI's verify job stops at 55 minutes** (`ci.yml`'s `timeout-minutes`, 40 until #1009; the
  user, 2026-11-18): `master`'s full run about 36, the runner's apt step 6 to 10 of them,
  `bench-check` about 6. A workload whose random games grow (E09 T38a2: the hires tripled
  `reach-towns-36`'s queries, 107 s under callgrind for 17 s) adds to every pull request's run:
  time a re-baselined workload against `master`'s and shorten its games in the same task when it
  grew by more than a few seconds.
- **The adventure benchmarks follow the heap:** a change that only adds a few small allocations
  at the build (the town part's content) moved `day-start-xl-144` by +1.6 % through glibc's
  `_int_malloc` alone (E09 T8c2). Compare per function with `master` built in the same sandbox
  first (skill `homm3-bench-regression`); with every counted function the same, re-baseline the
  workload with the reason in its description.

## The property plans (`h3-map` `src/tests/play.rs`, `view.rs`; the adventure lane's files)

- **A town's counts and Portal are its owner's view** (E09 T26a1, T26b): `check_towns` lets the
  counts change only with a week's start, a build or a town event of the town, the Portal only
  with a week's start, a build, a capture or a dwelling's flag for its owner, requires the
  owner's `TownChanged` for either, and compares the rest of the record as before. A new writer
  of them (T27's hire) is told and taught there.

- **A silent write of a hero's details meets two oracles** (E09 T49a1, the night's refill and a
  hire's Mana Vortex): `play.rs`' conservation (`check_conservation`) folds each hero's details
  from its told changes, with the day's regain and the towns' night modelled at `DayStarted`
  (`night_refill`, read from the state before the command), and `view.rs`' hire fold
  (`check_hired`) builds a hired hero from its slot's view and parts; a hire's check
  (`check_town_hire`) compares the town's record whole. A new silent writer is modelled there.

- **A plan's town has no `KINDS` entry** (E09 T9b1): `towns_laid_out` lays the plan's towns
  (`TownPlan`) out after every other object but the boats, each with a template appended after
  the objects'. Code that reads a plan object's kind from its template (`(template - 1) /
  BLOCKERS.len()`: the cover tests' `kinds`) skips `Body::Town` first, or it indexes past `KINDS`
  or misreads a kind; `view.rs` has `kind` and `footprint` (E09 T9b2b), which do.
- **The thin cover tests move with the steps:** a new run kind in `steps()` changes every
  property's and cover test's games. `Step::Town` on a map without towns plays as `Step::Hire`
  (before its own settling, which declines every question), so that the dwellings' cover test
  keeps its counts; raise a cover test's cases, not its draw (the adventure lane's `TRAPS.md`).
  Either lane's new plan field or step kind moves the other's cover tests too: after merging the
  adventure lane's plans, run all of `h3-map`'s tests again (E09 T19b's merge of #830 raised its
  teleporters' cover test to 512 cases).

- **The turn start is compared whole** (`check_timed`, E09 T19b): every event after a command's
  first `TurnStarted` must be what the map's timed events and then the owned towns' events give,
  in order (a town event's views after its `TownBuilt` or its `TownEventFired` are left out,
  the told-views property's; E08 T115a and E09 T27 met an event's creatures told without a
  building). A new event at a turn start (increment 3's creatures, a week's growth told) is
  taught there, or every property fails at the first such turn.

- **A public draw under the hygiene property** (E09 T9b2a's cut; T9b2b's `hygienic`): the
  hygiene property (`a_viewer_learns_nothing_it_cannot_see`) builds its other game with another
  seed, so a setup draw a viewer sees (a random town's faction, a neutral garrison, a random
  dwelling's type, since E09 T18b2a an owned town's buildings: a standard set with a Fort draws
  its level-2 dwelling) must have one outcome in its plans; and the towns' draws come first in the
  setup stream (design §14), so an unseen town's change (owner, garrison, removal) moves every
  later draw. A new plan field that draws at the build goes through `hygienic_town`. The told
  views (`views_move_as_the_events_say`) vary nothing and play the plans' towns as drawn.
- **What the build picks from an object's surroundings shows later** (E09 T9b2b, a gap of
  `master`'s): `vary` keeps the inputs of such a pick, a Shipyard's boat tile (ADV-SHY-001: its
  offsets' terrain and objects) and a random dwelling's resolved footprint (`object_reach`); a
  seen town's `TownId` counts the towns before it, so a town is removed only after every seen one
  and added after every other.

- **A guard's battle comes before the town's visit** (E08 T103d, decision 279): a step onto an
  undefended town's entrance that a guard watches leaves the hero on the entrance while that
  battle is open, or the guard's question (a monster's offer to join); the capture follows the
  answer. A check over a town's visitor (the hero on its entrance, TWN-VIS-002) exempts a hero
  whose player has an open decision (`GameState::has_decision`), as `check_invariants` does
  (the battle since the server lane's CI, 2026-11-09; the question since E09 T19c1's CI).

- **A garrison hero is off the map but still a hero** (E09 T28a): every oracle over the heroes on
  the map (`check_isolation`, `armies` for conservation, `check_invariants`) counts
  `AdventureState::town_heroes()` too, or a hero entering or leaving its garrison reads as a hero
  lost or made; the town's moves are probed among the town commands (`town_builds`). Since T29b
  the plans' `Step::Town` enters garrisons, so the battle's oracles meet one that defends and
  wins (it stays in the garrison, its army even empty), raises and levels up: `hero_anywhere`
  reads a battle's hero from either (`check_battle_end`, `check_heroes_after_battle`,
  `check_progression`, `check_conservation`'s levels), and `check_isolation` lets the battle's
  garrison hero change.
- **A new plan step meets oracles no game reached before** (E09 T29b: the hero moves met three):
  after changing what a step plays, run the properties of `play.rs` and `view.rs` at five times
  their cases once, `every_step_keeps_the_commands_invariants` at ten (raise the two blocks'
  `cases`, then put 48 back): 240 cases missed two that CI's seeds met.
- **A command listed for every hero outside a decision floods the plans** (E09 T38b2): `pick`
  draws a kind uniformly, so a dismissal in `managing` would be played in about one
  `Step::Manage` of five and empty the maps of heroes. `managing` leaves the dismissals out and
  `dismissal` plays one in one step of sixteen; a new command of that kind goes the same way,
  and `check_isolation`, `check_conservation` and `check_management` learn what it takes away.
- **A tavern hero shown ends the hygiene's comparison** (E09 T38a2): the slots' heroes are draws
  of the taverns' stream, which the other game's seed and types move (`without_tavern` hides the
  viewer's slots), so a hire the viewer is told (its own, or another player's on a tile it sees)
  ends it, as a battle does. A new way a tavern hero reaches the map (T38b's returns, T39a's
  map Tavern) ends it too.
- **The hygiene's other game keeps every ownable object's entrance land** (`view.rs`' `vary`,
  its `entrances`): the import refuses one entered from water (`OwnedOnWater`). A new ownable
  body (E08 T116b1's map garrisons, missed until T29b) joins that list.

- **A decision kind is known to four oracles** (E09 T40b, the plans' map Tavern): `check_legal`
  (its listing's branch and its probe, else it falls to the turn's listing), `check_decisions`
  (its part of the open set, opened and closed as told), `managed` (no army while it is open) and
  `check_conservation` (a price it pays); `settle` and `hiring` name it too. A new decision kind
  (increment 5's services, E08's markets) teaches all of them, as `check_map_hire` does.
- **A draw added to `plan()` shifts every deterministic cover test's cases** (E09 T40b): the
  cover tests draw their cases from one runner in sequence, so a value appended to `plan()`
  changes every case after the first. Map Taverns drawn there failed three cover tests, then
  three others at another frequency (thin counts gone, two oracle gaps met). Draw a new object
  apart, as `tavern_plan()` (`plan()`'s, then the taverns) for the properties and the object's
  own cover test: every other cover test keeps its cases.

## The views (`h3-game` `src/town/view.rs`)

- **Every change of a viewer's town view is told** (E09 T18b2a, SRV-Q26): a new write of a town's
  record, garrison or owner, or of what a viewer's level counts (its Taverns), takes
  `Told::before` of the touched towns (and of every town by the players whose Taverns it may
  count) before the write and `Told::tell` after it; the plans' told-views property folds the
  views strictly and fails on an untold change (Visions' reach is `TWN-STUB-13`'s, exempt).
  `Told` reads nothing for a sink that keeps nothing: test a hook with a `Vec` sink. Since E09
  T28b2 a town's garrison view is its upper bar: a write of a garrison hero's army (an army
  command naming it, its battle's end, its Necromancy) is a write of its town's view
  (`state/manage.rs`' `town_told`, `state/battle.rs`' `bar_told`), and the plans' fold takes a
  told view's garrison by slot whole (`views_move_as_the_events_say`, found by a stress of 300).

## The import (`h3-map` `src/import/town.rs`)

- **A town's entrance is blocked and visitable** in the shipped maps' templates: a test "does a town block this tile" also hits the entrance, where a moved hero (TWN-SET-010) and a town's visitor stand. Use `town::in_town` (a blocked tile that is not an entrance), not `blocks` (E09 T7a1, found by the census).
- **Every struct literal of `GameSetup`, `PlayerSetup` and `GameHeroSetup`** lives in other lanes' crates too (`h3-cli`'s `src/ai/census.rs`, the ai lane's; `h3-map`'s examples): a new field is a `cross-lane` change there.

## Experiments and the original's records (`experiments/`, the capture harness)

- **The original's sets hold bits the engine never has** (E09 T14c, survey `E09-construction.md`
  §13; towns survey §16.3): the hall companions (Rampart, Necropolis, Conflux: 23, 27 to 29 with
  the halls) and the town screen's display pieces (Castle 23, Stronghold 27, Fortress 23, Conflux
  22), the pieces in the standing set from the first time a town screen opens and in the
  every-built set after the town's next build. Drop them before comparing a record with the
  engine's sets or predicting a set; a prediction from the reading names them.
- **A spec's hordes are by dwelling level, and a batch that plays weeks needs every player a town**
  (E09 T23): `h3-experiment`'s "horde n" is the horde of the level-n dwelling (the Castle's Griffin
  Bastion 18 is "horde 3"; "horde 1" builds nothing there): read a town's every-built set on day 1.
  A player without a town is out of the game after seven days of play, and the game ends when no
  enemy is left. The week's and the month's kinds cannot be written before they start (the draw
  writes them): force the draws (`clients/capture/README.md`, "Forced rolls").

## The game's setup (`h3-game` `src/town*`, `setup.rs`)

- **A setup that allows hero types builds tavern heroes** (E09 T37a): the start's fill offers up to
  two types per player and makes their records after the setup's heroes, so a test reading
  `heroes().len()` or a type's `Free` status after `GameState::new` meets them (`Offered`); count
  `adventure().heroes()` instead. `crate::tests::setup()` allows none (no fill); `towns_setup`
  allows every type.

- **A customised type still free has its record from the build** (E09 T37b,
  `GameSetup::custom_types`): its id after the build's heroes, before any tavern record, so a test
  counting `heroes()` meets it. The fixture maps built from `h3-map`'s `tests::sample`
  (`fixtures.rs`) carry its map-wide entry for type 3 (a skill, a backpack artifact): a fixture map
  that does not place type 3 (`e08-army-12`, `e08-xl-144`) gets that record, its default
  experience moving the setup stream's later draws, and a new fixture map does too.

- **A setup's towns match its map's** (E09 T8a1, `GameSetupError::TownCount`): a `GameSetup` whose static map has objects of the type `town` needs one `TownSetup` per such object, in object order; a test or tool that builds a game from a static layer alone gives each a neutral town record (`h3-map`'s census example and `h3-cli`'s `src/ai/census.rs` do).
- **An `AdventureSetup` with towns gives one garrison and one faction per town** (`SetupError::Garrisons`, since E09 T11 `TownFactions`, checked first): a test that builds the adventure alone with towns sets both (`state/tests/towns.rs`' `town_setup`, `reach/tests.rs`); the game gives step 2's factions.
- **The game's town tests give the part's dwellings and forts no map slot** (`town/tests.rs`'
  `town_pack`; E09 T26a1): a `BuildingsSetup::Custom` cannot build them; build them after
  `GameState::new` with `towns_mut().build(town, building, part, core, false)` (the first
  creatures come with it). An adventure object type a test needs (a dwelling, the months) goes
  through `rules_with_adventure`'s edit.

## The taverns (`h3-game` `src/town/tavern.rs`, `pool.rs`)

- **A write of a tavern hero's record or away army** (a fill; from T38 a return, a hire's
  replacement) sets `Away::stale` and clears `Away::full` unless its spell points end at their
  maximum: the turn start derives only stale points and the day's regain skips full records (E09
  T37a: both cost the week-start benches tens of per cent). A placement on the map (T38a) hands
  the away record to the adventure: drop it, or the regain skips a hero on the map.
- **The week's fill is the week-start benches' largest part** (E09 T37a, re-baselined:
  `day-start-builds-36` +99 % on `master`, the XL day starts +13 to +17 %): the draws' counting
  pass, about 4 Ir a type per draw (an `Entry` without the records changed no count: callgrind
  counts instructions, not cache misses), the first week's new records (`setup::hero_details`)
  and the new tavern heroes' first points. A design change there: compare per function with
  `master` (`homm3-bench-regression`).
- **Every status write of the pool is checked after every command** (E09 T40a: `h3-map`'s
  `check_pool` in the plans' `check_invariants`, the census's `pool_holds` after every `EndTurn`
  of the 160 maps): a type offered exactly when one slot holds it, taken by a colour exactly when
  one hero of it stands on the map or in a garrison of that colour and is its record, every other
  status neither. A new path that puts a hero on the map or takes one off (E10's defeated
  players, a prison's hero; the map Tavern's hire does, T40b's plans) writes the type's status in
  the same command, or both fail; the census's fills assume `EndTurn` alone (no hire, no return: every slot drawn by
  the fill).
