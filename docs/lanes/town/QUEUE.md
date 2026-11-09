# Town lane — QUEUE

Pre-refined tasks of the town lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E09-towns.md` (increment 2: construction, refined at T10, `TWN-Q7`); done tasks
in `E09-log.md`; increment 1 (closed by T10) in `E09-archive.md`. Increment 2's first round
(decision 38) is done: the survey T12 (T12a, T12b) and the drafts with the design note and the cut
T13 (accepted at `TWN-Q10`, decisions 48–51). The build's rounds (design §35, decision 49): T14
batch TWN-2 (T14a to T14c done), T15 `shared/` (done), T16a, T16b (done), T17a (done), T17b (next), T18a to T18c, T19, T20 the audit; each
queued here as the round before it is done. Each task's cut confirmed at its start; every entry's
acceptance includes the Definition of Done (`docs/05-testing.md`).

**E09 T17b — `h3-towns`: the rules that decide a build, `TownCommand` and its errors** (design
§27, §34, §35's round 5; decisions 45, 49, 51; queued 2026-11-09 by T17a, which built the mark,
`built_with` and the store's writes; not `cross-lane`): in `h3-towns`' `build.rs` beside
`built_with`, the rules `buildable` (the hall's offer, a horde offered as its upgrade; empty while
`built_today`; TWN-BLD-002, -003), `possible` (TWN-BLD-002), `check_build` (§27's order of the
refusals, the day's mark last: TWN-BLD-004's refusal) and `event_buildings` (TWN-EVT-003);
`TownCommand::Build { town, building }` and `TownCommandError` (`#[non_exhaustive]`, with
`UnknownTown` and `NotOwner` for the game); `docs/rules/catalog/town.toml`'s TWN-BLD-001 to -003,
-006 and -007 (-006 and -007 sourced from T16b's harvest and survey `E09-construction.md` §5,
§6), TWN-EVT-003, TWN-BLD-004's summary amended with the refusal; `TWN-QUIRK-7` (T-9) where design
§34 places it. Checked by `h3-towns`' tests on its synthetic part (every refusal in its order, the
hall's offer with the hordes, the Capitol's limit, the Grail never by the hall, the event's
buildings with and without water). No version moves (the part and the store are unchanged).
**Status:** next.

## Inbox

Notes from earlier work for E09, gathered at the lane's opening (2026-10-31, T0); each points to
where the detail is (grep the date or id in the adventure lane's `QUEUE-ARCHIVE.md` or `QUEUE.md`).

- 2026-11-09 (the server lane, E16 M5 T1, SRV-Q26; **the user's decision**): **a town's garrison
  change is told to every player who sees the town.** In `towns-s173-d21` command 205 player 1's
  battle empties neutral town 4's garrison; player 0 sees the town and its `TownView::garrison`
  changes, but every event of the command is player 1's alone. A server game keeps a viewer's view
  current by events only (`docs/epics/E16-design.md` §14), so the change needs an event projected
  to every player whose view of the town changes (a `TownChanged`'s `seen_by`, or a garrison
  event). The server's fold test over the golden replays (M5 T3) checks it, a stub there until it
  lands.

- 2026-11-09 (the server lane, CI of #800, a random find; **an invariant of yours broken**):
  `h3-map`'s property test `tests::play::every_step_keeps_the_commands_invariants` failed in CI's
  coverage run on code that touches neither `h3-map` nor the game crates (`master` 2e5e08f plus
  `h3-protocol`'s session): E09 T9b1's check (TWN-VIS-002, `play.rs`, "no hero stands on the
  entrance of a town of another team or of none") reported `HeroId(2) on town 18 of
  Some(PlayerId(0))` on a 16×16 RoE plan, one level. The test draws a new seed each run (48
  cases), so it shows only now and then. To replay the shrunk case, add proptest's line
  `cc e663c03d407272cf82960aa4edf2492dcbec39f73811da59da67c47f238f4f46` to
  `crates/h3-map/proptest-regressions/tests/play.txt` (the adventure lane's file: `cross-lane`, or
  ask it) and run the test; the full plan is in the CI log of run 42665928151.

- 2026-11-05 (the adventure lane, E08 T97a, `adventure/e08-t97a`; **for the week's creature,
  increment 3**): the map's dwellings and Refugee Camps ignore the week's and the month's kind
  (ADV-DWL-003: the original's `dwelling_week_refill` 0x4B8A60 reads neither; survey
  `E08-dwellings.md` §6.1). The month's side is tested (`h3-game`'s
  `a_month_s_kind_never_reaches_a_dwelling_or_a_camp`: a Plague and a creature month); the week's
  creature is not drawn yet. When it lands, extend that test with a week of the hut's creature:
  the hut's and the camp's counts stay at their growth.
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
  Increment 3 (weights: only the tavern's draw reads them, survey `E09-towns.md` §4.4), increment
  4 (chances).
- (adventure queue, 2026-10-20, T24a2b1) `Marshland Menace` gives 15 hero types a map-wide
  experience of 2 000 that only its taverns reach. Increment 3.
- (ADR-0017 point 4; E08 decision 197) The week's creature, drawn from a stream of its own, and the
  Plague's effect on towns. Increment 3.
- (`ADV-STUB-9`, E10's) A Tavern town to hire a retreated hero again: a fact of the battle's
  retreat setup. Increment 3, with E10.
- (ADR-0013 point 13.6; E08 decision 199, A-30) The Castle's Lighthouse building (sea movement).
  Increment 4.
- (adventure archive, 2026-10-14, T19a2b) The Necropolis buildings' +10/+20 % Necromancy
  (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68). Increment 4.
- (adventure archive, 2026-10-03, E04a T3b) Combination artifacts' costs (blank in the original):
  the Artifact Merchants. Increment 4.
- (adventure archive, 2026-10-11, T8a; T34b) The new day's reveal around owned objects after a
  Cover of Darkness hid tiles (0x4C7BA0). Increment 4.
