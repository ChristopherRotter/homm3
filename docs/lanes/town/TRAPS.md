# Town lane — standing traps

Grouped by area. Read the group for the area your task touches; delete a trap when it stops applying, merge duplicates, add a new one under its area (a trap of every lane goes into `docs/HANDOFF.md` or the topic doc).

## Working in the adventure lane's crates

- **Ownership is by the longest match** (`lanes.toml`): only files whose path starts with `crates/h3-adventure/src/town`, `crates/h3-game/src/town`, `crates/h3-map/src/import/town` or `crates/h3-map/src/writer/town` are this lane's. A `mod town;` line in a `lib.rs`, a field in a state struct, a hash, a version constant, a replay or a bot report is the adventure lane's: such a pull request is `cross-lane` (the label at once), and `cargo xtask check-lane --cross-lane` is its local check.
- **Merge in order:** before a task starts and before a pull request opens, look at the adventure lane's open pull requests (its HANDOFF). When both lanes bump the same version, the second rebases and bumps again, and re-records the replays a second time (skill `homm3-rerecord-replays`).
- **The adventure lane's traps apply** to its crates (forks without deep copies, server-only RNG seeds, template masks, staging the shipped maps): its `TRAPS.md`.
- **The scoped verify** `cargo xtask verify --scope town --summary` also runs the adventure lane's crates, replays and benchmark workloads (`verify_with`); its `check-lane` step fails by design on a `cross-lane` branch. It does not run the other lanes' tests (`h3-cli`'s ai census, E09 T8a1's CI failure): a change to the setup's checks or the static map also runs `cargo test --workspace --exclude xtask --all-targets`.
- **A new game command, answer or event variant breaks matches in other lanes' crates** (E09
  T18b1): `h3-map`'s property tests (`play.rs`' `as_move`, `view.rs`' told views), `h3-sim`'s
  action entries, `h3-files`' replay words; the scoped verify builds `h3-map`'s tests only in its
  clippy and coverage steps. Run `cargo check --workspace --all-targets` right after adding one.
- **The adventure benchmarks follow the heap:** a change that only adds a few small allocations
  at the build (the town part's content) moved `day-start-xl-144` by +1.6 % through glibc's
  `_int_malloc` alone (E09 T8c2). Compare per function with `master` built in the same sandbox
  first (skill `homm3-bench-regression`); with every counted function the same, re-baseline the
  workload with the reason in its description.

## The property plans (`h3-map` `src/tests/play.rs`, `view.rs`; the adventure lane's files)

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
  in order (a town event's build's views after its `TownBuilt` are left out, the told-views
  property's). A new event at a turn start (increment 3's creatures, a week's growth told) is
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

## The views (`h3-game` `src/town/view.rs`)

- **Every change of a viewer's town view is told** (E09 T18b2a, SRV-Q26): a new write of a town's
  record, garrison or owner, or of what a viewer's level counts (its Taverns), takes
  `Told::before` of the touched towns (and of every town by the players whose Taverns it may
  count) before the write and `Told::tell` after it; the plans' told-views property folds the
  views strictly and fails on an untold change (Visions' reach is `TWN-STUB-13`'s, exempt).
  `Told` reads nothing for a sink that keeps nothing: test a hook with a `Vec` sink.

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

- **A setup's towns match its map's** (E09 T8a1, `GameSetupError::TownCount`): a `GameSetup` whose static map has objects of the type `town` needs one `TownSetup` per such object, in object order; a test or tool that builds a game from a static layer alone gives each a neutral town record (`h3-map`'s census example and `h3-cli`'s `src/ai/census.rs` do).
- **An `AdventureSetup` with towns gives one garrison and one faction per town** (`SetupError::Garrisons`, since E09 T11 `TownFactions`, checked first): a test that builds the adventure alone with towns sets both (`state/tests/towns.rs`' `town_setup`, `reach/tests.rs`); the game gives step 2's factions.
- **The game's town tests give the part's dwellings and forts no map slot** (`town/tests.rs`'
  `town_pack`; E09 T26a1): a `BuildingsSetup::Custom` cannot build them; build them after
  `GameState::new` with `towns_mut().build(town, building, part, core, false)` (the first
  creatures come with it). An adventure object type a test needs (a dwelling, the months) goes
  through `rules_with_adventure`'s edit.
