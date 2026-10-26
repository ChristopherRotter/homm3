# Adventure lane — QUEUE

Pre-refined tasks of the adventure lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T51b (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 6 are closed (2026-10-12, -13, -20, -21, -23, -26; increment 6, army and hero management, by T53's audit). **Increment 7, rewards on the map**, refined with the user at T53 (ADV-Q26; decisions 121 to 123): first T54 (the battle seam's facts), then round 1, the survey and the ADR amendment drafts (T55a, T55b), each task's cut confirmed with the user at its start. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

### E08 T54 — The battle seam's facts from the game

- **Scope** (decision 123; the inbox's three battle-lane notes of 2026-10-23 and -24, which this
  task closes): `h3-game`'s battle setup (`state/battle.rs`, `Battles` when a battle starts on the
  map) fills what the seam gained for battles from the map, each from the game's own state:
  (1) `BattleSetup.difficulty`, the game's difficulty (the setup's, kept in the state if it is
  not); (2) `BattleSetup.human`, per side from a new per-player flag of the game's setup
  (`PlayerSetup`), false for a side without a player; with it BAT-QUIRK-27's let-off (BAT-MOR-004,
  on in `faithful`) no longer reaches a human player's side. Where the flag comes from (the
  session that seats the players, the map's player kinds for the import's default, what an env's
  agent counts as) is the user's question at the task's start; (3) `BattleSetup.colours`, each
  side's player colour (`None` for a monster's or a guard's side); (4) `BattleSetup.background`,
  0x41ADB0's variant on dirt, grass and snow: the tiles within 3 of the battle's whose object
  class is a tree-like, mountain-like or dead vegetation one (`docs/epics/E17-design.md` §5:
  classes 114 to 160; the import may have to keep a decoration's class, a map import version);
  (5) `BattleSetup.attacker_ground`, the special battlefield of the attacking hero's own tile
  (ADV-GRD-001); (6) `BattleHero.sex`, the map's sex code for a placed hero (`h3-map`'s model
  reads it; `None` the hero type's own). The retreat facts (`BattleSetup.retreat`, ADR-0027 point
  14: a Tavern to hire the hero again, E09; the victory condition, E10; read only for a computer
  player's hero, none before E10) stay at their defaults: a stub (`ADV-STUB-9`). Rule ids (a
  `GAM-BAT-*` per derived fact), tests, the ten and two golden replays re-recorded (their hashes
  inside battles move), the versions it moves (game state hash, map import) in the table.
- **Not in it:** the computer player's own choices (E10, E13), the retreat facts' derivation.
- **Depends on:** T53 (done). **Status:** ready; its cut confirmed with the user at its start.

### E08 T55a — The rewards survey

- **Scope** (decisions 121, 122): the survey `docs/rules/surveys/E08-rewards.md` (as T17a, T24a,
  T34a and T44a): the original's rules for the objects that hand a visiting hero something, by
  code reading (`docs/exe/README.md`) with VCMI 1.7.5 and the wiki, and a census of the 160
  shipped maps (which classes, how often, which bodies). (1) **Treasure chests** (the gold or
  experience choice, the artifact). (2) The objects that give **experience, spells, secondary or
  primary skills**, once per hero or once per game (for example Tree of Knowledge, Learning Stone,
  Witch Hut, the Shrines, Scholar, Arena, Library of Enlightenment, Marletto Tower, Star Axis,
  Mercenary Camp, Garden of Revelation, the Schools of Magic and War; the survey settles the
  list). (3) **Artifacts as pickups**, guarded or not, with the random artifacts and Spell
  Scrolls. (4) **The monsters' treasures with an artifact** the import refuses (E08 decisions 22
  and 48; 8 shipped maps). (5) **Rewards that give creatures** (the army dialog's other callers,
  0x4ABDC0, 0x573C80; survey `E08-army.md` §1) and where Pandora's Box, events and Seer's Huts
  belong. (6) What a reward asks (a question, a choice), refuses (a full backpack, a hero
  without a book, a skill already known), what changes in the hero's derived properties and
  when, what another player sees of a visited object (`docs/06-security.md`); the commands,
  decisions, events, the observation, replays and benchmarks. (7) **Batch E7's plan** on the
  client lane's harness (decision 122's agent's call). (8) The proposed rule ids, quirk
  candidates and disputes; a fresh agent's second reading of the code; the checkpoint's questions
  (an `ADV-Q`).
- **Not in it** (decision 121; the epic's sketch, items 8 and 9): boats, Navigation and the sea
  (Favorable Winds, the sea's chests and flotsam), the week's objects and mines; towns (E09).
- **Depends on:** T54 (decision 123's order; nothing in it is needed). **Status:** ready after
  T54; a user checkpoint at its end (decision 122).

### E08 T55b — The ADR amendment drafts, the design note and the cut

- **Scope:** from T55a's survey and the answers at its checkpoint: the amendment drafts of the
  ADRs it touches (ADR-0016 for the objects and their visits, `docs/lanes/adventure/adr-drafts/`;
  ADR-0017 for the commands and decisions; ADR-0013 if `h3-heroes` gains rules), `E08-design.md`
  rewritten for increment 7 (the hero side, the map, the game, the batch and the cut, with
  increment 7's exit criteria), the building tasks cut in rounds; the checkpoint's questions (an
  `ADV-Q`). The user accepts the drafts and the cut before anything is built (decision 122).
- **Depends on:** T55a and its checkpoint. **Status:** waiting for T55a.

## Inbox

- 2026-10-24 (**T54**, decision 123; the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; **for you**, when battles start from the adventure
  map): the seam carries three facts for viewers, read by no rule, each defaulting to "not
  known": `BattleSetup.background` (the battlefield picture's variant on dirt, grass and snow:
  0x41ADB0 counts the dead vegetation, mountains and trees within 3 tiles of the battle's tile, 0
  where dead vegetation is the most, 1 where mountains are, 2 otherwise, an empty surrounding
  included; `docs/epics/E17-design.md` §5), `BattleSetup.colours` (each side's player colour by
  `BattleSide::index`, 0 red to 7 pink; `None` for a side without a player, a monster's or a
  neutral town's; one colour for both sides is `SetupError::InvalidColour`) and `BattleHero.sex`
  (`HeroSex`, where the map gave a placed hero one; `None` the hero type's own). `h3-game` should
  set them when it starts a battle. The battle keeps them as hashed state (battle state hash v54)
  and observes them; your ten golden replays were re-recorded with their options, their commands
  unchanged (only checkpoint hashes taken inside a battle moved).

- 2026-10-23 (**T54**, decision 123; the battle lane, E23 T7, `shared/e23-t7-human` #436 and `battle/e23-t7-rest`, both
  `cross-lane` with the user's consent; **for you**): **a battle rule reads `BattleSetup.human`
  now**: with the new fix-optional BAT-QUIRK-27 (`quirk.battle.computer_morale_let_off`, registry
  position 33, on in `faithful`) a frozen stack of a side whose player is not human escapes one
  bad-morale freeze in four (BAT-MOR-004, 0x464BDB; ADR-0023 point 7 widened). `h3-game`'s battle
  setup (`state/battle.rs`) leaves `human` and `difficulty` at their defaults, so in a game's battle
  every side, a human player's too, gets the let-off: set `human` from the side's player (true for
  a human player's hero, false for a computer player's and a neutral side) and `difficulty` from
  the game. In your paths: the base faithful core ruleset hash moved (`0x1ac6c50d158057de` →
  `0x0c7e01b4dffeba0c`), your eight faithful replays' headers patched (`ruleset_hash` only; all ten
  verify). Also BAT-WAR-003: the automatic tent heals the most hurt stack (no draw).

- 2026-10-23 (**T54**, decision 123; the battle lane, E23 T6's last part, `battle/e23-t6`, `cross-lane` with the user's
  consent; **for you**, when battles start from the adventure map): the battle now keeps
  `BattleSetup.retreat` and `attacker_ground` (ADR-0027 point 14) as hashed state, and
  `h3_battle_ai::choose` reads them: a hero's side retreats as the original's computer does
  (BAT-AI-024; `POLICY_VERSION` 18) when the facts allow it, and the odds value an attacking hero's
  spells at its own ground's mastery in `faithful`. The game orchestrator (`h3-game`) should derive
  them: per side the hero's experience, its artifacts' worth by the original's count (per worn or
  carried artifact the larger of the adventure AI's value, 0x433AA0, and half its gold cost), a
  Tavern town of its player to hire it again from (another than the battle's town for its
  defender), whether the scenario's victory condition wants that hero defeated; and the special
  battlefield of the attacking hero's own tile. The defaults never retreat. Your ten golden
  replays were re-recorded again with their options, their commands unchanged (only hash lines;
  `duel-s2-d7` none): the hash covers the new fields.

- 2026-10-23 (T42, measured; for a performance task): **a flyer's or water walker's reach costs
  about 1.9 times a walker's query** on the XL map (`reach-lifted-xl-144`: 279 k Ir a query
  against `reach-xl-144`'s 148 k on the same games), and random play on the duel with one of each
  costs 56 % more a step (`random-play-duel-36`): `Reach::search_layered`'s second layer is the
  lever (`crates/h3-adventure/src/reach.rs`).

- 2026-10-22 (the battle lane, E23 T4c, `battle/e23-t4c`; **for the neutral seat**, E23 exit criterion 5): the original's battle AI for a side without a hero is complete: `h3_battle_ai::choose(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (shots, melee, moves, waits, defends, the creature casts), pure, deterministic, its draws from the stream given, its choice validated (Defend if refused). The neutral seat (decision 27, your random stand-in) can call it for a monster battle's unowned side; `POLICY_VERSION` 9. A side with a hero (a computer player's) casts nothing and never retreats until E23 T5 and T6 (BAT-STUB-5).

- 2026-10-22 (ADV-Q20, decision 75; restated at ADV-Q21, decision 78, in ADR-0016's draft point 4; for the increment that revisits flying): **A-13's trade-off.** Flying stays on explored tiles (A-13 fixed for now, provisional), so wide obstacle ranges with unexplored tiles beyond them stop a flyer; the user: "probably fix, but gameplay impacts have to be considered (wide obstacle ranges become impassable); perhaps deserves a compromise, like allowing to route to the closest unexplored, but free hex; TBD later anyway". Decide the compromise, what it tells the player of the hidden tiles, and `ADV-QUIRK-15`'s class (built by T41: the unexplored tile's row of `AdventureState::passage`, `crates/h3-adventure/src/state/passage.rs`). ~~**A-14's gap**~~ (closed by T38: the tile bit +0xC 0x40 means "no object blocks the tile", not the coast, so `hero_flight_tile_test` passes on every free land tile and a flyer visits as a walker). ~~**D-23**: Cursed Ground's movement effect is built only if batch E5's row E5-6 confirms it.~~ (E5-6 confirmed it for both grounds, T38: built by T40.)

- 2026-10-20 (T24a2b1, survey §6; for the increments that build them): **levels at load beyond placed heroes.** 61 prisoners in 28 shipped maps have a custom experience (levels 3 to 15, two at 42 in `Pandora's Box `) and 18 random heroes carry one: their levels at load come with prisons and random heroes (the wiki's "fictitious level-ups" of a prisoner, §5). `Marshland Menace` gives 15 hero types a map-wide experience of 2 000 that only its taverns reach (E09); a hired hero type may start with a skill its map bans (19 SoD maps ban skills; D-18), where the original's handling of such a skill was not read.

- 2026-10-20 (T22b, E3-9's reading; for E10's computer players or the formation command: increment 6 since decision 90, T44a's survey point 4): the battle places an army in the tight formation only for a human player's side (`battle_war_machine_stacks` 0x463679, the side's +0x54A4): a computer player's hero always fights loose, whatever its formation. The engine has no computer player and keeps the map's formation (`ADV-QUIRK-8`); decide then whether a computer player's side fights loose (a rule or a quirk; the battle lane's inbox has the tables).

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
