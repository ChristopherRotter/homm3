# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T97c (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 10 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05; increment 10, dwellings and hiring, by T97's audit). **Increment 11, the teleporters**, refined with the user at T97c (ADV-Q44; decisions 260, 261): round 1, the survey and the ADR amendment drafts first: T98a in two (decision 262), T98a1 done, T98a2 next, then T98b. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T98a2 — the teleporters survey, its second part** (decisions 261, 262; survey
`docs/rules/surveys/E08-teleporters.md` §9 to §14)

- **Scope:** the reach and the paths through a teleporter (the original's: `pathfinder_pass_through`
  reads the monolith lists at 0x56AE39 and 0x56AE81, whose paths those are; the engine's reach and
  passage, the hot path); the space key's visit (`key_space_visit` 0x408E65: its full gate, which
  classes, RoE) and whether the increment takes it as a command; what a viewer sees of a hero
  passing (`hero_relocate_inner` 0x49CF50, the network message, the wiki's exit shown in the
  shroud); the interfaces' sketch (commands, decisions, events, the observation, replays,
  benchmarks); batch E11's plan on the client lane's harness (decision 261's agent's call), with
  rows for the points resting on the reading alone (survey §8's open points); the proposed rule ids;
  the checkpoint's questions (D-53 to D-57, A-37's fix, A-38, A-39, A-40), asked at its end.
- **Inputs:** survey §0 to §8 (T98a1); decisions 260 to 262; ADR-0015 to -0017 (ADR-0016 points 3
  and 4, ADR-0017's streams); the lane's `TRAPS.md` (a new read in the reach's or the passage's
  loops; a test's blocker of a class that gets a kind); the inbox's performance note of 2026-10-23
  (a lifted hero's reach); surveys `E08-hero-movement.md` (the reach, flying, A-13) and `E08-sea.md`
  (a boat's entrances).
- **Depends on:** T98a1 (done). **Status:** ready (the original data staged: `Heroes3.exe` for the
  reading, the shipped maps for the batch's plan).

**E08 T98b — the ADR amendment drafts, the design note and the cut** (decision 261; as T88b)

- **Scope:** the drafts in `docs/lanes/adventure/adr-drafts/`, `E08-design.md` written for increment
  11 (its pointer replaced; increment 10's note in `E08-log.md` as the model), the reach's and the
  passage's cost planned against the gated workloads (the hot path, `homm3-bench-regression`), the
  building rounds cut, batch E11 first; the checkpoint's question: the user accepts the drafts
  before anything is built (decision 261).
- **Inputs:** the survey `docs/rules/surveys/E08-teleporters.md` (its interfaces' sketch, the
  batch's plan, the proposed ids, its checkpoint's answers); decisions 260, 261.
- **Depends on:** T98a. **Status:** after T98a; its cut confirmed with the user at its start.

## Inbox

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

- 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
  unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
  `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
  `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
  30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
  in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
  its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
  side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
  --map` fills the template beside the map when its stamp matches (no cargo build). A town's
  visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
  after. `clients/capture/README.md`, "Towns (E09 T3b)".
- 2026-11-03 (the client lane's CLT-Q40, carried by the battle lane, BAT-Q64; `cross-lane`): the
  morale and luck sources ride on the effects (`h3_contracts::EffectSource`). `derive_battle_hero`
  now names the skill and level, the artifact and the specialty (HER-BAT-003) and gives the battle
  bonuses as the temporary source (HER-ADV-008), so battles from the map carry them already. Left
  for you: the visits by once-mark (`EffectSource::Visit { mark }`, the original's +0x105 lines)
  once the derivation knows each mark's amounts (your content), the rest staying temporary; the
  owner's Grail building is the town lane's (its inbox).

- 2026-11-02 (the town lane, E09 T1b2b, `TWN-Q2`; **nothing to build, for your import and
  setup**): E09's increment 1 builds **the random hero** (class 70, drawn as the main town's hero;
  E09 decision 8) and **the starting-hero pick per colour in the setup** beside the faction
  (decision 9), `cross-lane` in your import and setup, merged in order with yours; leave both to
  it (your 2026-10-20 note's 18 random heroes with a custom experience come with it). An allied
  town is entered and never captured (decision 16), as your owned objects' team test.

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

- 2026-11-03 (E08 T87b's audit; **for the next import bump**, increment 10's or the town lane's
  T7a): the parser reads class 220 below subclass 7 as a mine body (`parse/objects.rs`) while the
  class table types every 220 as the abandoned mine (`import/codes.rs`), so a crafted 220.0 is
  refused at the build (`SetupError::Holds`) where the original (220 loaded as 53, the subclass
  kept, `object_class_values` 0x63A6E4) and VCMI make a sawmill. No shipped map has one (all 94
  are 220.7). Type 220 by `MINES` like 53, test 220.0 as a sawmill, amend ADV-MIN-001 and
  ADV-OBJ-002 and `docs/formats/h3m.md`, with the bump that moves `MAP_IMPORT_VERSION` anyway.

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
