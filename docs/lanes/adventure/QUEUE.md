# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T76 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30; increment 8, boats and the sea, by T76's audit). **Increment 9, mines, the week and owned objects**, refined with the user at T76 (ADV-Q36; decisions 192, 193): the survey and the ADR amendment drafts first (T77a, T77b), each task's cut confirmed with the user at its start. T77a in two (decision 194): T77a1 done (`adventure/e08-t77a1`: survey `E08-economy.md` §0 to §6), T77a2 done whole (decision 195; `adventure/e08-t77a2`: §7 to §18, the checkpoint's `ADV-Q37`). T77b done whole (decision 200; `adventure/e08-t77b`: the drafts, `E08-design.md`, the cut in rounds 2 to 8, decision 202; the checkpoint's `ADV-Q38`, decisions 201–204). T78 done whole (decision 205; `adventure/e08-t78`, `cross-lane`, decision 206: the writer's ownable objects, weekly objects and timed events, the spec's fields, the specs `E08-batch-9a` and `-9b`). T79 in two by batch (decision 207): T79a
done (`adventure/e08-t79a`: the harness's mine records, tile words, boat dialog, Leave Guards,
battle sides and `auto again`; the recipe `E08-batch-9a` and its runs); T79b done whole (decision
208; `adventure/e08-t79b`: the harness's income, Garden mask, weekly objects' and monsters' words,
end-turn messages; the recipe `E08-batch-9b` and its runs). T80 done (decisions 209, 210: whole
with A-33's re-read; `adventure/e08-t80`, `cross-lane`: batch E9 scored, survey §19, A-33
withdrawn). T81 in two, the ADRs first (decision 211): T81a done (`shared/e08-t81a`: the three
drafts applied, ADR-0015's core row, the index, the glossary); T81b done (decision 212: whole;
`shared/e08-t81b`, `cross-lane`: the four kinds, the two parameters, the creature's abilities
outside battle, the packs; core ruleset hash v51, pack format v50). Round 5: T82 and T83a done
together (decision 213; `adventure/e08-t82-t83a`, `cross-lane`, decision 214: the day's income,
spell points and the Lighthouses' input; the seven kinds and parameters, the import), T83b in two by what reads it (decision 215): T83b1 done
(`adventure/e08-t83b1`, `cross-lane`, decision 217: the owners, guards, weekly contents, draws,
the start's reveal, the flag radius), T83b2 in two, the state first (decision 218): T83b2a done
(`adventure/e08-t83b2a`: the fractions, the added objects, `check_added`, adventure state hash
v19), T83b2b done (decision 219: whole, the `shared/` part first, a mine's guards as the
original's hint; `adventure/e08-t83b2b` with `shared/e08-t83b2b`: the views). Round 6: T84 in two
halves, the mines first (decision 220): T84a done (`adventure/e08-t84a` with `shared/e08-t84a`:
the flags, the mines' visits, Leave Guards, their battles; game state hash v21); T84b in two,
the Lighthouses and the weekly visits first (decision 221): T84b1 done (`adventure/e08-t84b1`
with `shared/e08-t84b1`: the Lighthouses' flag and points, the weekly visits); T84b2 done
(`adventure/e08-t84b2` with `shared/e08-t84b2`: the Shipyards, their boat offer and `BuildBoat`,
adventure replay format v8; the user's decision 222: a Garden marks the team). Round 7: T85 in
two halves, T85a whole (decision 223): T85a done (`adventure/e08-t85a` with `shared/e08-t85a`:
the week's step, a creature month's step, the audit of the static reads). T85b on one branch
over two sessions (decision 224): T85b1 done on `adventure/e08-t85b` (pushed, no pull request);
T85b2 in two (decision 225): T85b2a done on the same branch (the timed events, `events_today`;
the security lines on `shared/e08-t85b`); T85b2b done on it (the env's `BuildBoat`, action
encoding v6, `cross-lane`; the verify with the original data, the bench; the pull requests,
`shared/` first: the lane HANDOFF names them).
Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T86 — the economy maps, their replays, properties and benchmarks**

- **Scope:** round 8's first task (design §15, §18; decision 202), its cut confirmed with the user
  at its start: `e08-economy-36.h3m` and `e08-economy-144.h3m` (`h3_map::writer`, design §15's
  objects); the golden replays on them (random play over 60 days with a creature month by a seed
  search; scripted: a flag and Leave Guards, a neutral and an owned mine fought, a boat built at a
  visit and by `BuildBoat`, a weekly object visited twice in a week); the plans' `ECONOMY` classes
  and `Step::{Garrison, BuildBoat}` with design §15's checks; the workloads `day-start-xl-144`,
  `month-start-xl-144`, `random-play-economy-36`, `fork-economy-xl-144`, the existing ones
  re-baselined where the growth moved them (T85b1's and T85b2b's records in `E08-log.md`); the
  `--with-original-data` counts (60 owned mines, one Shipyard, the boat tiles, the abandoned
  mines' sets, 666 timed events); the seven duel replays' feature lists (T85b1).
- **Inputs:** design §15, §18; `E08-log.md` T85b1, T85b2b; `docs/lanes/adventure/TRAPS.md` (the
  plans, `lay_out`, objects added in play).
- **Depends on:** T85b's pull requests merged. **Status:** ready after them.

## Inbox

- 2026-11-02 (the town lane, E09 T2b, `TWN-Q4`; **for your merge order, nothing to build**):
  E09's build bumps your versions `cross-lane` (`docs/epics/E09-design.md` §19, §21): adventure
  replay format v9 (T5a: the town part's hash in the header; `GameRuleset::new` gains the town
  part), map import v16 (T7a), game state hash v23 after your T85b's v22 (T7b), adventure ruleset
  part v16 and state hash v20 (T8a: the kind `Town`, towns ownable); the towns' income joins your
  T85b's day advance (§16). None starts before your T85b's pull requests merge; the second to
  merge rebases and bumps again.

- 2026-11-02 (the town lane, E09 T1b2b, `TWN-Q2`; **nothing to build, for your import and
  setup**): E09's increment 1 builds **the random hero** (class 70, drawn as the main town's hero;
  E09 decision 8) and **the starting-hero pick per colour in the setup** beside the faction
  (decision 9), `cross-lane` in your import and setup, merged in order with yours; leave both to
  it (your 2026-10-20 note's 18 random heroes with a custom experience come with it). An allied
  town is entered and never captured (decision 16), as your owned objects' team test.

- 2026-11-02 (the battle lane, E26 T5; for your information, nothing to do now): the fair
  system actor `h3_battle_ai::choose_fair` (ADR-0027 point 4 as amended) is to be the computer's
  default battle policy everywhere, the all-seeing `choose` a game's option, "player option too"
  (the user, AI-Q10). When the game's setup gets options, the sight is one of them; the neutral
  seat (`h3-sim`'s adventure env) is the AI lane's switch. `crates/h3-cli/tests/
  adventure_replays.rs`' uncommitted both-sides battles may use either.

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

- 2026-11-02 (the town lane, E09 T1a1; **for increment 10's random dwellings**): of the shipped
  maps' random dwellings tied to a town by its identifier (classes 216 and 217), all 1 397 (87
  maps) are tied to a random town (class 77), none to a fixed one; the original resolves the
  random towns' factions as the map's objects are read and the dwellings after them
  (`random_dwellings_resolve` 0x502B60). So these dwellings wait for E09's random towns (increment
  1, survey `docs/rules/surveys/E09-towns.md` §1.6, §2.1, §2.3); 205 take a faction set instead.

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

- 2026-10-31 (the client lane, E17 M5 T8b2, review R7-8; for information, nothing to do): the
  harness checks the starting bonus Gold it sets. `h3drive.gold_bonus` (a recipe's `gold = true`,
  `h3adv.to_hot_seat(..., gold=True)`) reads the scenario list's rows in memory
  (`h3mem.starting_bonuses`: H3API's `H3SelectScenarioDialog` pointer 0x69FBE8, `bonusType` +0x6C,
  3 Random, 0 Artifact, 1 Gold; observed) and stops the run unless exactly its players read Gold.
  Recipes without `gold` are unchanged.

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

- 2026-10-30 (the map client lane, E24 M3 T6a2b; the user's standing consent for the writer, E24
  T0; for your information, nothing to do): **the writer writes quests**: `Extras::quests` maps
  an index of `Extras::objects` (a Seer's Hut, 83, or a Quest Guard, 215) to a
  `model::Mission`, written in the SoD layout with no deadline, no texts and a seer's reward
  nothing (`writer::extras::write_quest`; refused on another class or with a list past 255).
  A literal `Extras { tiles, objects }` now needs `..Extras::default()`.

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

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
