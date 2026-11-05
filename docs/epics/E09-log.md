# E09 log: done tasks in full

The full record of E09's done tasks (what was built, agent decisions marked "change freely",
measurements). **Not start-of-session reading:** the epic file keeps one line per done task; read
the matching section here when a task builds on an earlier one or a decision is questioned.

## Done

### T0 — E09 refined, the town lane opened (2026-10-31)

One session, the user present; branch `shared/town-lane` (`cross-lane`). Starting point: `master`
at 11f4a137 (E08 T84a merged), the adventure lane next at E08 T84b. The user's answers (TWN-Q1,
two rounds of `AskUserQuestion`) are the epic's decisions 1–6; the agent's calls are decision 7.

- `lanes.toml`: the lane `town` (`TWN`) with `crates/h3-towns/`, the town modules of the adventure
  lane's crates (`h3-adventure` and `h3-game` `src/town*`, `h3-map` `src/import/town*` and
  `src/writer/town*`), the town catalog and quirks ledger, E09's epic, surveys and experiments
  (moved from the adventure lane), its folder; its queue common; `verify_with = ["adventure"]`.
- `xtask`: `Lane::verify_with` (validated: another lane), `Config::verified_lanes`;
  `verify::LaneScope::lanes`, the scope's packages and replays the union of the covered lanes',
  the benchmark gate's workloads by `bench::Selection::Lanes`; tests for each. *Agent's call
  (change freely):* the scoped verify of a lane owning sub-paths of another lane's crates runs that
  lane's crates, rather than `lane_packages` counting any owned sub-path (which would have widened
  the adventure and AI scopes to `h3-sim` and `h3-cli`).
- Docs: `AGENTS.md` (the lane, its reading list), `docs/08-work-process.md` ("Lanes": ownership,
  ids, versions), the global HANDOFF's table, `docs/README.md`, `docs/03-domain.md` (the Town
  crate decided), BACKLOG (E09's row, the ordering rationale), the registries' TWN blocks,
  `scripts/sandbox-setup.sh`'s comment; the town folder (HANDOFF, QUEUE with T1a–T2 and an inbox
  of the earlier notes for E09, TRAPS, QUESTIONS-ANSWERED); the adventure lane's inbox note.

### T1a1 — the towns survey, part 1a: the census and the factions (2026-11-02)

One session, the user present; branch `town/e09-t1a1`. Starting point: `master` at f949c899
(E08 T84b2 merged; CI green; no open pull request of either lane). The user cut T1a in two at its
start (`AskUserQuestion`, the recommended option): **T1a1** the census and the players' factions
with the random towns, **T1a2** the main town's hero, the pool and the rest of the start.

- **Survey** `docs/rules/surveys/E09-towns.md` §0 to §3 (index in `surveys/README.md`): the census
  of the 160 shipped maps (a throwaway example over `h3_map::read` and `h3_map::import`, not
  committed: towns by class, faction, owner and format, the alignment bytes, the one footprint of
  the twenty templates, the main towns at their entrance tiles, allowed factions against the main
  towns, buildings, garrisons with 12 random stacks, events, heroes on a town's blocked tile (all
  right of the entrance), the import re-run with the town refusals and then the random heroes
  cleared: all 160 import; 1 397 random dwellings tied to random towns); the code reading of the
  players' factions (`scenario_dialog_begin` 0x588330, `random_allowed_faction` 0x5132D0) and of
  a random town's faction (`load_town_body` 0x5019F0: the alignment's colour, the owner's, else
  eight factions on an RoE map and nine on an AB or SoD map), the conversion in `setup_towns`;
  VCMI 1.7.5 and the wiki.
- **Disputes and candidates:** TD-1 (VCMI takes the owner before the alignment; the reading and
  the wiki the alignment), TD-2 (VCMI draws Conflux on RoE maps), T-1 (an alignment byte past the
  colours). *Agent's call (change freely):* E09's disputes are `TD-<n>`, not the adventure
  surveys' running `D-<n>`, so the two lanes' surveys never take the same id in parallel.
- **The exe map:** `load_town_body`, the site `random_town_neutral_factions` 0x5020C5,
  `count_allowed_factions`, `scenario_dialog_begin`, `faction_choice_offered`;
  `random_allowed_faction` read; `setup_towns`' names; `structs.toml`: `town_setup`, the game's
  `town_setups`, `chosen_factions`, `player_attributes`.
- **Notes:** the adventure lane's inbox (random dwellings tied to random towns wait for E09's
  resolution). For `TWN-Q2`: the random hero (class 70, 36 maps) shares the pool; with it
  increment 1 lets every shipped map import.

### T1a2 — the towns survey, part 1b: the heroes and the towns at the start (2026-11-02)

One session, the user present; branch `town/e09-t1a2`. Starting point: `master` at d5c70795
(#641 merged). Asked at the start whether to cut T1a2 again at the buildings (`AskUserQuestion`):
the user chose **the whole of T1a2 at once**.

- **Survey** `docs/rules/surveys/E09-towns.md` §4 to §6 and §0's findings 9 to 16: the order of
  a new game's hero steps (`game_reset`, the dialog's choices, `new_game_heroes`, the map read with
  `hero_pool_init`, the random heroes at the objects' read, the placed heroes, `main_town_heroes`);
  the pool (RoE maps allow the Conflux heroes besides; SoD's disposed heroes' masks); the
  starting-hero choice (the dialog's list of up to 16, the colour's first random hero or a human's
  main town takes it); one draw for the random hero and the main town's hero
  (`hero_draw_of_faction` 0x4BB400, uniform, no tavern weights: those are the tavern's,
  `tavern_hero_draw` 0x4BB5E0, increment 3); the generated hero on the entrance as the visitor
  (`hero_show` sets a town's visiting hero); a hero right of a town's entrance moved onto it
  (`placed_heroes_setup`); the town's setup (`town_setup`): a custom garrison's random stacks, a
  neutral town's drawn garrison (33/33/20/13 in 100); the buildings (`town_buildings_setup`, its
  earlier summary corrected: an owned standard town gets dwellings only with a Fort). A census
  (a throwaway crate outside the workspace over `h3_map::read`, not committed): no placed hero or
  prisoner is in its map's allowed set; 176 random heroes on 36 maps (T1a1's 166 corrected);
  every owner of one has the random-hero flag; 23 heroes in towns, none in a generating main town;
  garrisons and standard sets by owner and fort.
- **Disputes and candidates:** TD-3 (standard dwellings, VCMI 100/50 for every town), TD-4 (level
  4's 13 against VCMI's 14), TD-5 (an explicitly empty neutral garrison), TD-6 (a hero on a town's
  other blocked tiles); T-2 (the starting-hero choice not reserved: a hero twice; fixed as
  state-corrupting), T-3 (a random hero before a placed one of its type; none on the shipped
  maps), T-4 (the draws' empty cases). For T1b's checkpoint: TD-3 and TD-5 as batch TWN-1 rows.
- **The exe map:** 26 new entries (`hero_draw_of_faction`, `tavern_hero_draw`, `game_reset`,
  `new_game_load`, `map_header_read`, `hero_pool_init`, `town_change_owner`, `main_town_heroes`,
  `hero_place`, `new_game_start_other`, `building_tables_init`, `load_hero_body`,
  `scenario_hero_lists`, `new_game_start`, `town_heroes_learn_spells`, `town_setup_spells`,
  `town_build`, `town_boat_spot`, `town_hero_place`, `grail_victory_town`, the neutral garrison's
  three tables, `faction_buildings`, `chosen_heroes`, `superseded_buildings`) and three sites;
  `hero_show`, `town_setup` read; `new_game_heroes`, `new_game_bans`, `placed_heroes_setup`,
  `town_buildings_setup` amended; `structs.toml`: the game's `first_heroes`, `allowed_heroes`,
  `hero_owner`, `hero_masks`, `player_attributes`' hero fields, `hero_class`' faction and tavern
  weights, `town_setup`'s buildings and garrison, a new `town`.
- **Content:** none for increment 1 from T1a2 (the epic's "tavern weights" move to increment 3;
  the neutral garrison's table and the 30 are executable constants).

### T1b1 — the towns survey, part 2a: the town on the map (2026-11-02)

One session, the user present; branch `town/e09-t1b1`. Starting point: `master` at ad0c9805
(#644 merged; CI green on 34fc55ef, the last code run). Asked at the start how to cut T1b
(`AskUserQuestion`: two halves, three parts, or whole): the user chose **two halves**, T1b1 the
visit, capture and the battle at a town, T1b2 the income and the Capitol, the views, batch
TWN-1's rows and the checkpoint.

- **Survey** `docs/rules/surveys/E09-towns.md` §7 to §10 and §0's findings 17 to 23: the step
  onto a town's gate (`hero_move_step`'s case 98: an own or allied town or an undefended one
  entered, a defended one visited from the neighbouring tile; a hero on the gate met first); the
  town's visit (`town_visit` 0x4AAFD0, read whole); the visitor as the hero on the gate
  (`hero_show`, `hero_hide`), no garrison hero before increment 3's swap; capture of an undefended
  town (the screen opens) and after a siege won (the attacker stays outside); the change of owner
  (`town_change_owner` read whole: the garrison emptied, the lists, the external dwellings'
  counts, the reveal 5/20/the Tower's Grail, the scenario's tests); one Capitol per player
  (`capitol_limit`); who defends (a visitor beside a garrison hero in the open, a lone visitor
  with the garrison merged in by AI value, `garrison_merge` read whole; the town's owner as the
  side); the battle's outcome paths; ADR-0023's facts from the town as one table, and the game's
  version at a new game (the scenario dialog's version, a campaign's index; whether an RoE map
  plays under RoE rules in the Complete edition left to batch TWN-1). VCMI 1.7.5
  (`CGTownInstance`'s visit, merge, capture, Capitols), coyot's guide, the wiki's Tutorial, a
  reader's comment (a garrisoned hero without units).
- **Disputes and candidates:** TD-7 (the merge's order and ties against VCMI's), T-5 (a garrison
  hero without creatures: the town falls without a battle; the original's proposed, increment 3).
  Batch TWN-1 rows proposed: the merge with a full army, the siege winner's tile, the leftover
  garrison lost, the moat on an RoE-format map.
- **The exe map:** 15 new entries (`garrison_merge`, `town_owner_message`, `town_at_entrance`,
  `dwelling_creature_faction`, `ai_town_visit`, `scenario_version_from_slots`,
  `scenario_version_highest`, `town_screen_open`, `town_remove_from_owner`, `capitol_limit`,
  `town_dwelling_count_add`, `victory_capture_town_is`, `victory_capture_town_met`,
  `victory_town_visit_test`, `loss_town_test`); `town_visit` (was `hero_battle_on_map`),
  `town_change_owner`, `army_reset`, `hero_loss_effect` (was `hero_off_map`) read whole;
  `adventure_battle`, `hero_hide`, `new_game_heroes`, `new_game_start` amended; `structs.toml`:
  the town's +0x2, +0x32, `dwelling_counts` (was `second_army`), the game's `map_kind` setter.
- **Content:** none new (the creatures' AI values are in the packs).

### T1b2a — the towns survey, part 2b1: income, views and batch TWN-1's rows (2026-11-02)

One session, the user present; branch `town/e09-t1b2a`. Starting point: `master` at 43b1c3ad
(#648 merged; CI green on it). The views needed a reading the queue had not foreseen (the
right-click popup: E24's readings cover the own town's status window, the hint and the cursor
only), so T1b2 was cut at its start (`AskUserQuestion`: after the readings, after the income, or
whole): the user chose **after the readings**, T1b2a the income, the views and the rows, T1b2b
the checkpoint (`TWN-Q2`).

- **Survey** `docs/rules/surveys/E09-towns.md` §11 to §13 and §0's findings 24 to 27: a town's
  gold (`town_gold_income` read whole: 500 to 4 000 by the highest hall, the Grail +5 000, the
  silo no gold), who is paid (the owner, from day 2, E08's path; a neutral town nobody), one
  Capitol per player only after a capture (the start keeps every one the map gives); the census
  (23 Capitols, 10 owned, none twice for a player; the Grail in 10 towns, 2 owned; the silo in
  26, 7 owned); what another player sees (the picture, the hint, the cursor, and the right-click
  read whole: the fort level for all, the army by the viewer's taverns 0/1/2, the counts with the
  hall, income, silo and garrison hero for the owner's team or expert Visions or a Rogue within
  reach); the census of the levels at the start (63, 600, 78 players); batch TWN-1's seven rows
  with the harness's and the spec's gaps; the points open for T1b2b. VCMI 1.7.5 (the income, the
  town info's two levels, Visions and the Rogue), the wiki (Halls, Grail, Tavern, Thieves' Guild,
  Visions), the RoE manual.
- **Disputes and candidates:** none new. VCMI's two-level town info against the tavern levels is
  recorded in §12.4 (the wiki agrees with the reading), no batch row.
- **The exe map:** 10 new entries (`town_popup`, `town_popup_level`, `town_quick_view_create`,
  `quick_view_army`, `quick_view_army_places`, `player_tavern_count`, `hero_visions_level`,
  `hero_visions_reach`, `visions_cast`, `town_silo_resources`);
  `town_gold_income` and `town_silo_income` read whole; `tile_click_path` amended (its right
  button).
- **Content:** none new; the silo's table is a constant of the executable (increment 4).

### T1b2b — the checkpoint of increment 1's survey (2026-11-02)

One session, the user present; branch `town/e09-t1b2b`, docs only. Starting point: `master` at
b6c6020c (#649 merged). The queue's list and survey §13's asked interactively (`TWN-Q2`, three
rounds of `AskUserQuestion`, a recommendation each); the record is `QUESTIONS-ANSWERED.md`.

- **Decisions 8 to 16** (the epic): the random hero in increment 1 (8); the starting-hero pick in
  the setup as the original's offer and use, reserved before any draw (9); the Resource Silo paid
  from increment 1 (10); the views by the original's levels (11); T-1 to T-5 as proposed (12);
  TD-1, TD-4, TD-6 by the reading, TD-3, TD-5, TD-7 by the reading with the batch (13); TD-2 by
  the enabled expansions (14: the user's own answer, E08 decision 104 applied; a third round asked
  the player's side, Conflux only with all eight allowed, and dropped row 5); batch TWN-1's rows 1
  to 4 (15: rows 6 and 7 dropped by the user over the recommendation, row 5 by decision 14);
  the allied town entered, never captured, the ally's defence a stub until E10 (16).
- **Survey** `E09-towns.md`: §14 (each open point and its decision), finding 28, rows 5 to 7
  struck in §13's table. The epic's scope bullets and T2's and T3's lines follow the answers; the
  queue's T2 entry rewritten with them (a seam to offer: the ADR drafts first).
- **The adventure lane's inbox:** the random hero and the starting-hero pick are built by the
  town lane in increment 1, `cross-lane` in its import and setup.

### T2a — the ADR drafts (2026-11-02)

One session, the user present; branch `town/e09-t2a`, docs only. Starting point: `master` at
4f457aaf (#652 merged). T2 was cut in two at its start (`AskUserQuestion`: the ADR drafts first,
`h3-towns`' draft alone, or T2 whole): the user chose **T2a the ADR drafts**, T2b the design note,
the cut and the checkpoint. The ADR-level choices asked before writing (`TWN-Q3`, one round of
four, each as recommended): decisions 17 to 20.

- **The draft ADR-0028** `docs/lanes/town/adr-drafts/0028-town-context.md` (the number checked
  when it moves into `docs/adr/`): the crates `h3-towns` (domain) and `h3-pack-towns` (adapter);
  a town split once between `h3-towns` (faction, buildings built and forbidden, events, name; the
  derivations: hall, income, fort, siege, reveal radius, Tavern, Capitol's limit) and the
  adventure (owner, garrison, visiting hero), composed by `h3-game`; `TownId` in `h3-contracts`;
  the store of towns in `GameState` (the store of heroes' shape, its own hash and version); the
  town part `TownRuleset` and `packs/*/towns.toml`; no other seam; the moat and the siege's
  natural obstacles per faction by the packs (a Tower's kind keeps deciding its land mines'
  damage, as `h3-battle` reads it); the pool the game's; the lane's versions.
- **The amendments** beside it: ADR-0015 (the crates, the third part, the import's towns, players'
  choices, the random and the main town's hero as placeholders, a hero right of a town's
  entrance moved, the allowed heroes with Conflux by the packs, T-1 an invalid map, `TownId`);
  ADR-0016 (the town object: `TownId`, owners with the ownable objects, the garrison as the armies
  on objects, the visitor as the hero on the entrance; the kind `Town` entered and not from above;
  the step's three cases with decision 16's ally stub; capture never a flag, the attacker outside
  after a siege; the reveal by the town's radius; the views by the levels); ADR-0017 (the store
  and the pool in `GameState`, the pool's life with the picks reserved and T-2/T-3 fixed; the
  setup's per-player fields and their checks, the original's offer; the expansions' line closed;
  the setup stream's new draws before every existing one, in four steps; the draw with T-4 a typed
  failure; the towns' income from day 2; the versions; no new command or decision kind; capture's
  application in both contexts; sieges' setup and outcome; the viewer's level); ADR-0023 (the
  derivation by survey §9.4; the moat and the natural obstacles by the packs; the version line
  withdrawn).
- **Agent's calls** (change freely at the checkpoint): the setup stream's order of the new draws
  (ADR-0017's amendment point 4); the faction choice's default from the map (the one allowed
  faction, else Random); a stored visiting hero rejected (the hero on the entrance is it); the
  pool as one block by hero type.

### T2b — the design note, the cut and the checkpoint (2026-11-02)

One session, the user present; branch `town/e09-t2b`, docs only. Starting point: `master` at
c1a6f68 (#653 merged; CI on `master` green at 9b8b7c6, the docs-only merge's push run under way).
The user kept T2b whole (`AskUserQuestion`: two parts by side recommended, the note then the cut
with the checkpoint, or whole). The sandbox's VM restarted once after the checkpoint (nothing
lost: the work-in-progress commit was pushed before it).

- **The design note** `docs/epics/E09-design.md`, from the code the build touches (the import's
  object loop and `Unsupported`, `GameSetup`, `PlayerSetup`, `GameState::new`'s stream, the
  adventure's ownables, `army_left` and `mine_guards`, `Defender`, `BattleStart`, `Siege`,
  `generate_setup_obstacles`, `GameRuleset::new` and its callers, the fixtures, the spec's towns,
  the versions): part A §1 the crates and T4's list, §2 the town part and `towns.toml`'s shape
  (the parameters, the faction record, the buildings' roles, the compiler's checks,
  `GameRuleset`'s fourth part), §3 the content (the readings, the names, VCMI, the gap-fill), §4
  the record (the ever-built set, the standing set derived) and the store, §5 the build's town
  setup (buildings, garrison), §6 the derivations; part B §7 the import (players' starts, towns,
  the random hero, the moved hero, the pool's inputs, the census), §8 the kind `Town` and the
  static map's town list, §9 the step's three cases, §10 the merge and the defender, §11 capture
  in both contexts, §12 the views' words for `docs/06-security.md`; part C §13 the setup's
  fields and checks, §14 the pool and the four steps of draws, §15 the siege's battle (the
  faction's native terrain, the Stronghold's obstacles from a Citadel, the defender's bonuses and
  mana), §16 the income in T85b's day advance, §17 the observation, §18 the gates, §19 versions,
  rule ids (TWN-SET-001 to -010, TWN-VIS, -CAP, -SIE, -INC, -VEW), quirks `TWN-QUIRK-1` to `-4`
  (T-1 to T-4), stubs `TWN-STUB-1` (an ally's visitor, E10) and `-2` (the start's forbidden Grail
  and Shipyard), §20 batch TWN-1, §21 the cut, §22 risks.
- **Read for it, not new:** the E04c survey's S-29 (the Stronghold's natural obstacles under RoE
  and AB come with a Citadel or Castle, so the town part's flag applies from a Citadel) and its
  "the town's native terrain" for a battle at a town; ADR-0023 point 6 (the battle gives the
  attacker its Catapult) settles the queue's inbox note on VCMI's catapult.
- **The checkpoint** (`TWN-Q4`, one round of four, the note sent to the user first; decisions
  21–24): the drafts accepted as written; the cut as proposed; the content read and harvested
  through `h3-harvest`'s gap-fill; the agent's calls kept.
- **The queue:** T3a, T3b and T3c written (round 2 and 3); six inbox notes the note now carries
  moved to the new `QUEUE-ARCHIVE.md`; a note in the adventure lane's inbox on the planned
  `cross-lane` bumps and their order.
- **Agent's calls** (kept at the checkpoint, decision 24): `towns.toml`'s shape; the record's
  ever-built set; the static map's town list instead of a stored id; `Holds::Town { formation }`;
  the garrison written into `army_left` by the build; no event for entering an own town; the
  merge told as the army dialog's changes; a neutral garrison as `Defender::Guards`; the siege's
  terrain and no special battlefield; an unused pick freed after the draws; the resolved choices
  not kept; `GameRuleset::new`'s fourth argument with `TownRuleset::empty` for encounters; the
  Visions factor a part parameter until Visions is placed.

### T3a — batch TWN-1's maps and spec (2026-11-02)

One session, the user present; branch `town/e09-t3a`, `cross-lane` (the user's consent at the
start: `h3-experiment` and `docs/experiments/README.md` are the battle lane's). Starting point:
`master` at 75cfdd7 (#656 merged; CI on `master` green at c1a6f68). The user kept T3a whole. No
open pull request touched `h3-experiment` (#654, #655 the adventure lane's).

- **`h3-experiment`:** `TownSpec` and `MapTownSpec` gain `fort` (the editor's standard set with a
  Fort, the map's has-fort flag; refused beside `buildings`, which name the Fort) and a
  `garrison` that is absent (the map's default) or a list, `[]` an explicitly empty custom
  garrison (seven empty slots written); the template's line says "the standard buildings with a
  fort" and "an empty custom garrison". The test
  `a_town_may_have_the_standard_set_with_a_fort_and_an_empty_custom_garrison` builds both for an
  encounter's town and two map towns and reads them back through `h3_map::parse` (the has-fort
  flag, a custom army of seven `None`). The spec format in `docs/experiments/README.md`.
- **The spec** `experiments/specs/E09-batch-1.toml` (stamp `7902bb9e`; its map writes, 34 map
  towns and two sieges; the template `experiments/results/E09-batch-1.md` committed unfilled):
  row 1 sixteen owned towns with a Fort, eight without, eight neutral with and without (three new
  games: 48 draws of the level-2 dwelling, the reading's 30 in 100 against VCMI's 50); row 2 two
  neutral towns with `garrison = []` beside the eight controls; row 3 Blue's Castle with a lone
  visitor of seven stacks (two equal weakest, two Pikeman stacks) and a garrison of three (two
  equal), played twice from one save (3a Red retreats: the garrison's leftovers; 3b Red wins:
  the leftovers lost, the attacker's tile and points, no screen); row 4 a neutral Rampart with a
  Fort and 5 Centaurs won, then the later step onto the gate. Each row's prediction from survey
  §5.2, §5.3, §8.2, §8.3, §9.2, with VCMI's.
- **Row 3's design** (agent's call): a scratch simulation of both merges (the reading's §9.2 and
  VCMI's §9.5) chose armies where each of the reading's three points (the last equal weakest, the
  first equal gain, the rounds until nothing gains) moves a stack: the reading's opening
  [Swordsman 4, Halberdier 16, Marksman 10, Griffin 5, Cavalier 1, Monk 3, Pikeman 15], the
  garrison left [—, Centaur 4, Archer 5]; VCMI's Pikeman 10 with the 5 Pikemen left in the
  garrison under either of its tie rules; each tie flipped alone gives a third and a fourth
  outcome.
- **For T3b** (the harness): the town record's reads (game +0x21614, 0x168 bytes each: owner +0x1,
  visitor +0x10, garrison +0xE0, built +0x150, every-built +0x158) are not in
  `docs/exe/structs.toml` yet; three new games for rows 1 and 2, rows 3 and 4 in the first.
- **`check-exe-map` on `master`:** T2b's design note (#656, docs only, so CI's full verify never
  ran it) cited an address of `building_tables_init`'s lists that no map entry covered; the town
  scope's verify failed on it. The AI lane's #657 merged the map entry first, so the note stays as
  it was.
- **CI's first run on #659** also failed a random case of `h3-map`'s
  `every_step_keeps_the_commands_invariants` (the adventure lane's; reproduced on `master`'s code
  with its seed): a note in the adventure lane's inbox, the seed not committed. Locally: `check-lane --cross-lane`, `cargo check --workspace
  --all-targets`, `h3-experiment`'s tests and clippy, the town scoped verify with the original data
  (every step but `check-lane`, by design, after the fix).

### T3b — batch TWN-1's harness and runs (2026-11-03)

One session, the user present at the start; branch `town/e09-t3b`. Starting point: `master` at
e02c8e5 (#659 merged; CI on `master` green there; the sandbox's full engine run green on the same
code). The user kept T3b whole and asked for narrow sub-agents (one wrote the harness's reads and
the recipe offline in a worktree, one learned the town screen and played the runs); 3b is played
in a new game instead of loading save 1 (the user: the merge has no random part; the record of 3b's
opening shows any difference). The installation's files staged from the user's computer at the
start. The VM restarted once (nothing lost).

- **The harness** (`clients/capture/`, common; README "Towns (E09 T3b)"): `h3mem` reads a town's
  record by its entrance (`town_at`: owner +0x1, faction +0x4, garrison hero +0xC, visitor +0x10,
  garrison +0xE0, built +0x150, every-built +0x158; the towns from game +0x21614 to the vector's
  end +0x21618, `structs.toml` gains `towns_end`, the town fields their readers). `h3adv`: the keys
  `town:x,y:owner|visitor|built|every|garrison|slot:N`, `towns:every_vs_built`,
  `towns:owned_garrisons`, `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`; a `new_game = true` step (`start_game`, the first start's path); the town
  screen known by its vtable 0x64372C (`town_dialog_create` 0x5C34D0, map entry, inferred) and
  closed by its exit button (item 30720), the level-up dialog (0x63FE60) as `level_up`; the table
  battle plan's `attack` a list of creatures; fixes: `play_turns` reads the turn again after its
  wait (3b's first run sent the Archangels' order into the Swordsmen's turn), a retreat waits out
  the retreating side's Catapult. The harness check passed before and after; the client,
  map-client and adventure lanes' inboxes told.
- **A town's visitor (+0x10)** reads −1 while its hero is the selected one and holds it otherwise
  (a probe: TWN-1-4 on the gate after entering, −1 while selected, 18 once another hero was
  selected and on Blue's turn): the recipe reads TWN-1-4's town with TWN-1-1 selected; the
  README's and `structs.toml`'s notes.
- **The recipe** `clients/capture/batches/E09-batch-1.toml` (15 steps): game 1 rows 1 and 2's
  reads and 3a; game 2 the reads, 3b and row 4; game 3 the reads; all on Red's day 1.
- **The runs** (Easy, hot seat, the starting bonus Gold, under Wine): the first stopped in 3b
  (`play_turns`), four probes of 3b, then two whole runs, recorded: `experiments/results/
  E09-batch-1.md` (the committed recipe) and `E09-batch-1-run2.md` (played first; its row 4's
  visitor read with the hero still selected, `none`). One more start crashed on the new-game screen
  (the known rumour fault 0x4CCF8C, README "Traps") and was run again.

**What the runs showed** (for T3c; not scored here): **TWN-1-1** the level-2 dwelling (31) in
4, 4, 2 (run 2) and 3, 4, 4 (the main record) of the 16 owned towns with a Fort a game: 21 of 96;
every other set as the reading's ({5, 10}; {5, 7, 10, 30}; neutral {10, 30, 31}, {7, 10, 30, 31})
but **building 23 in every Rampart, Necropolis and Conflux town of the standard set** (owned and
neutral, with and without a Fort; not predicted, not in the survey); every-built equal to built.
**TWN-1-2** NEF and NE empty in all six games; the drawn garrisons in level order, some empty.
**TWN-1-3** both runs alike: the opening [Swordsman 4, Halberdier 16, Marksman 10, Griffin 5,
Cavalier 1, Monk 3, Pikeman 15], the garrison left [—, Centaur 4, Archer 5] (the reading's
prediction exactly); 3a Red retreated, Blue won, Blue's army as the opening; 3b Red's hero on
(20, 62) with 1 900 of 2 000, the owner Red, no town screen, the garrison empty; the Catapult acted
by itself. **TWN-1-4** before (40, 62) 2 000; after the siege (40, 62) 1 900, Red, no screen, the
garrison empty; the later step (40, 61) 1 800, the town screen, the visitor TWN-1-4.

### T3c — batch TWN-1 scored (2026-11-03)

One session, the user present; branch `claude/peaceful-curie-q7wjex` (the session's), `cross-lane`
(decision 26: `docs/experiments/README.md` is the battle lane's). Starting point: `master` at
dd99747 (#662 merged). Docs only; no game files (VCMI 1.7.5 cloned into the sandbox's scratch).

- **Scored** (survey §15, both records; counted by a throwaway script, not committed): TWN-1-1
  the reading's sets (building 23 aside) in all 204 readings, the level-2 dwelling 21 of 96 (95 % interval 0.14 to
  0.32; VCMI's 50 in 100 at p ≈ 10⁻⁸), none in an owned town without a Fort (VCMI's level 1
  always refuted); TWN-1-2 the empty custom garrisons 12 of 12 empty, the controls' 48 draws as
  the reading's table (15 empty, 31 %); TWN-1-3 the merge's three points each moving a stack as
  T3a designed (VCMI's merge refuted); the siege's aftermath as §7.1, §8.2, §8.3. TD-3, TD-5, TD-7
  confirmed; ADR-0017's amendment step 4 unchanged.
- **Building 23** in every Rampart, Necropolis and Conflux town of the standard set (66 of 66),
  in no other faction's: not built by §5.3's reading (its source a part of the setup not read);
  VCMI has no building 23 there (`"special4" : null`) and drops undefined ids; the keep counts it
  if the faction's buildable set holds it. `TWN-Q5` (one round of two): **re-read in T5b**
  (decision 25), the README's row `cross-lane` (decision 26).
- **Docs:** the survey's §15 and finding 29; the design note's §3, §5, §10, §20, §21, §22; the
  ADR-0017 draft's consequence; the epic's decisions 25, 26; `docs/experiments/README.md`'s row;
  the queue: T3c out, round 4's T4 in.

### T4a — the ADRs and the glossary (2026-11-03)

One session, the user present; branch `shared/e09-t4a`, `cross-lane` (the town lane's docs ride
along, as E08 T81a's #612). Starting point: `master` at 971da9e (#666 merged). T4 cut in two at
its start (the user: the ADRs and docs first, T4b the skeletons). Docs only: no code, no version.

- **ADR-0028** moved from the draft (the next free number, 0028, checked: 0027 the last, no other
  lane's draft numbered), status Accepted (TWN-Q4), its Consequences' `shared/` work split into
  T4a and T4b, a revision history; its "the amendment" pointers named by point.
- **The amendments folded in**, each as its ADR's increments are (a block per point, a revision
  history line; ADR-0023 in its status line): ADR-0015 points 1, 2, 4, 5 (and R01 point 3's
  folder list); ADR-0016 points 1, 3, 4, 5, 6; ADR-0017 points 1, 2, 3, 4, 9, 10, 11 ("towns
  later", the pool's and "the moat, E09"'s phrases closed); ADR-0023 points 2 and 5 rewritten
  (the packs decide the moat; the natural obstacles the town record's flag) and its Consequences
  (the version question withdrawn). Batch TWN-1's verdicts (survey §15) as "confirmed" beside
  TD-3, TD-5, TD-7 and the attacker outside; building 23 changes no point (decision 25). Each
  draft's sentences checked against its target by a throwaway script (every one found but the
  headings and the reworded decision references).
- **Also:** the ADR index (0028's row, the four Read-when cells); `docs/03-domain.md` (the Town
  row's crate and the hero model, a town's terms, a town among the ownable objects);
  `AGENTS.md`'s town reading list gains ADR-0028; the battle lane's inbox: `siege::setup`'s
  comment (ADR-0023's consequence). Left to T4b: `Moat`'s doc comment (it is code in
  `h3-contracts`), `docs/02` §3, the versions rows, ADR-0005's crate list and ADR-0006's pack
  files.


### T4b — the crate skeletons, `TownId`, the gates (2026-11-03)

One session, the user present; branch `shared/e09-t4b`, `cross-lane` (the town lane's docs and
`packs/README.md`, the battle lane's, ride along). Starting point: `master` at 8ea2e4a (#665
merged, CI green; the sandbox's baseline the light gates on CI's fingerprint). Taken whole (a
small task, no seam needed). No version moves: the two new constants start at 1.

- **`h3-towns`** (domain, logic: the lint header, `no_std`): `TownRuleset` and `Towns`, empty
  `Default` structs whose doc comments say what T5a and T6 add; `TOWN_STATE_HASH_VERSION` and
  `TOWN_RULESET_HASH_VERSION`, both 1. No dependency and no function yet, so nothing to cover
  (the gate skips a crate of declarations only, as E08 T2's skeletons).
- **`h3-pack-towns`** (adapter): doc only; the parser and the format come with T5a.
- **`h3-contracts`:** `TownId(u16)` beside `HeroId` (`new`, `raw`, `index`; a test), its doc
  ADR-0028 point 3's. *Agent's call (change freely):* the design note's "serde as a number" read
  as `HeroId`'s rule: `h3-contracts` has no serde (none of its types derives it), so files and the
  wire write `raw()`; a derive would be a new external edge of a shared logic crate. `Moat`'s doc
  comment: the packs decide (ADR-0023 point 2 as E09 amended it), its variants by the packs'
  records instead of the game's version.
- **`arch.toml`:** both crates (`h3-towns` → kernel, content, contracts, heroes; `h3-pack-towns`
  → kernel, content, `h3-towns`, `h3-pack`, serde, toml), `h3-game` → `h3-towns`; and, *agent's
  call*, the edges T5a's callers need (design §2): `h3-sim` → `h3-towns` (dev `h3-pack-towns`),
  `h3-cli` → both, `h3-map`'s dev → both, so T5a needs no `shared/` change (as E08 T2 did for its
  T9). `h3-ffi` gets none: it calls no `GameRuleset::new`.
- **`lanes.toml`:** `crates/h3-pack-towns/` and each pack folder's `towns.toml` (the five folders,
  as the adventure lane's files) to the town lane. **`clients.yml`** watches `crates/h3-towns/**`
  in both lists (the clients build it through `h3-game` from T5a; `check.sh`'s "ci paths" step
  checks only missing crates).
- **Docs:** the versions table's two rows and the town lane's paragraph (`docs/08`); `docs/02` §3
  (the two crates, the lanes' list, "E09 adds"); ADR-0005's amendment (the crates, `TownId`) and
  ADR-0006's pack file line (`towns.toml`), each a revision history line (presentation); the
  `README.md` layout table; `packs/README.md`'s file table row. The design note's §1 shrunk to
  what T5a and T6 still read. The queue: round 5's T5a and T5b written (T5a's likely cut noted).

### T5a1 — the town part, its compiler and the adapter (2026-11-03)

One session, the user present; branch `town/e09-t5a1` (the lane's own crates, not `cross-lane`).
Starting point: `master` at 90e8638 (#671 merged, CI green; the sandbox's baseline the light gates
on CI's fingerprint). **T5a cut in two at its start, the order changed** (the user, asked with the
reason: all 39 adventure golden replays use the base pack, so a `GameRuleset` before the content
would move their header's new line twice; `h3-game`'s and `h3-map`'s test files are also in the
adventure lane's open stack #669 to #674): **T5a1** (this), then **T5b** (the content), then
**T5a2** (`GameRuleset`, its callers, replay format v9; `cross-lane`). No version moves:
`TOWN_RULESET_HASH_VERSION` stays 1 with its first encoding.

- **`h3-towns`** (deps: kernel, content, contracts): `building` (`BuildingId`, below
  `MAX_BUILDINGS` 64; `BuildingSet`, a `u64`: `EMPTY`, `bits`, `contains`, `with`, `union`, the
  rest with T6); `ruleset` (the pack definitions `TownPack`, `TownParamsDef`, `TownDef`,
  `CreatureLevelDef`, `BuildingDef`, `BuildingRoles`, `TownOverride`, `TownPatch`; the compiled
  `TownParams`, `TownRecord` with `building` and `building_by_key`, `CreatureLevel`, `Building`;
  `TownRuleset::compile`, `empty(core)`, `core_hash`, `content_hash`, `params`, `towns` in the
  core's faction order, `town(faction)`); `ruleset/compile.rs` (the load order as
  `h3-adventure`'s, each pack's parameters, records and overrides in load order, then the checks
  across packs); `Moat` re-exported (the adapter has no `h3-contracts` edge). Tests on cores built
  in code: every check of the design §2 refused (the pack set, the references and the closure,
  an override of a record outside the closure, the parameters, a record's levels, upgrades and
  silo, the buildings' count, keys, slots, role values, `supersedes` and `forbidden_with` unknown
  or cyclic, the roles across a record, the demoted Capitol), the closure of `supersedes`, the
  overrides applied, the empty part; the hash pinned (fixture and empty part). 100 % line and
  region coverage.
- **`h3-pack-towns`** (deps: content, `h3-towns`, `h3-pack`, serde, toml): `parse_towns(manifest,
  text)`, `compile(packs, profile)` (the core compiled from the same texts, as
  `h3-pack-battle`'s; `""` an empty file), `TownPackError` (`Syntax`, `Key`, `CreatureLevel`,
  `Core`, `Compile`); the format and each role's set in its `lib.rs`. Tests with inline files on
  the synthetic core packs (a record of `vale`, the expansion's override) and the original core
  packs without files (the empty part).
- *Agent's calls (change freely), the design §2's shape changed where it is built:* `gold` names
  the resource the halls and the Grail pay; `[town_params]` comes from exactly one pack (as
  `adventure.toml`'s `[params]`; the design said a later pack's replace), and a part without
  records may lack it (packs without `towns.toml` compile to `TownRuleset::empty`); `creatures`
  holds **one to seven** levels, each the base and optionally one upgrade (the design said seven
  with upgrades: the synthetic factions have two levels and no upgrades, and the synthetic packs
  are the battle lane's; T5b checks the original nine have seven with upgrades, and a draw of a
  level a faction lacks is T6's typed failure, T-4's kind); the siege's side facts are three
  roles (`morale` over the standing set, `luck` and `escape_tunnel` over the ever-built set, as
  the survey §9.4 reads them) instead of one `siege` table; `dwelling` checked at the record's
  levels, each once, `[1, false]` and `[2, false]` present; morale and luck -3 to 3; building
  keys `[a-z0-9_]+`; the override changes `moat`, `siege_obstacles` and `silo` (no `fixes` gate:
  no data quirk needs one).
- **Not here:** the pack files (T5b, which also writes the synthetic packs' records and
  `packs/README.md`'s row, the battle lane's); `GameRuleset` and the replay header (T5a2).

### T5b1 — the buildings' tables read, building 23 found (2026-11-03)

One session, the user present; branch `town/e09-t5b1` (docs and the common address map only).
Starting point: `master` at 0e37885 (#675 T5a1 and #677 merged; CI on `master` green at 1642a86,
the docs-only merge after it runs none). **T5b cut in two at its start** (the user, asked with the
reason: the reading can raise disputes that change what the harvest writes): **T5b1** (this, the
reading and the checkpoint), **T5b2** (the harvest, `cross-lane`). `Heroes3.exe` and the text
files (`extracted.tar`) staged from the user's repo folder; VCMI 1.7.5 sparse-cloned (`config`,
`lib`).

- **Read whole:** `building_tables_init` 0x4EB730 and its helpers (0x4EBC50, 0x4EBB10, 0x4EBB70,
  0x5BDF60), `town_build` 0x5BE930, `town_turret_count` 0x5BF4F0 with the setup's mask writes.
  The tables dumped by a throwaway script: the nine faction lists with their direct requirements,
  the common and faction supersede lists, `map_building_ids`, `town_creatures`, the hordes, the
  silo and the garrison. Survey §16; the address map's twelve new entries and five updated, the
  struct `town`'s `recruits` and `buildable`, `creature_record`'s `growth`.
- **Building 23** (decision 25): `town_build` builds each hall's town-screen companion in
  Rampart, Necropolis and Conflux (10 → 23, 11 → 27, 12 → 28, 13 → 29); the Village Hall is built
  through it for every town before the setup branches, so a custom set has 23 too; none is in the
  faction's list (not buildable, not in the mask +0x160, not counted by the keep), none is
  superseded, no rule reads them (the only loads of bit 23 are faction-tested: the Tower's Wall of
  Knowledge, the computer's visit values).
- **Also found:** no faction's list holds the Village Hall, so the keep never counts it (design
  §6's `structures` counted it); a horde supersedes its level's dwelling and `town_build` turns it
  into the upgraded horde when the level's dwelling is upgraded; a custom set built in id order
  leaves a horde's dwelling standing (T-6); the dwellings' available creatures set at the build
  (increment 3); `installation.md` §7.27 names the Fortress' 21 and 22 the wrong way round (the
  battle lane's inbox).
- **VCMI 1.7.5:** the same lists, requirements and supersedes, but the Village Hall and the ship
  as buildings, 27 to 29 as automatic buildings without 23, the upgraded hordes' requirements and
  supersedes, the Necropolis' horde and the Conflux's upgraded Altar of Fire (TD-9, increment 2);
  its turrets another model. **TD-8:** the code and VCMI give the Glyphs of Fear +2 attack and the
  Blood Obelisk +2 defense, the game's descriptions and the fandom wiki the reverse.
- **The checkpoint** (`TWN-Q6`, one round of four, `AskUserQuestion`): decisions 27 to 30. The
  companions left out of the part (recommended); the keep skips the hall-1 building **in the
  derivation** (over the recommended building flag); the standing set **derived, T-6 fixed** with
  the hordes' upgrade (over the recommended both sets stored, T-6 kept); TD-8 by the code
  (recommended). The design note's §3 to §6, §19 and §22 amended; T-6 is `TWN-QUIRK-5`.
- **For T5b2:** the queue entry (rewritten from T5b's): the records as the original's lists plus
  the Village Hall, the hordes' upgrade as data (the field is T5b2's call), the keys (the hordes
  share their upgrades' names), the roles by survey §16.6.

### T5b2a — the hordes' upgrade as data, the synthetic town records (2026-11-03)

One session, the user present; branch `town/e09-t5b2a` (the town lane's paths only). Starting
point: `master` at 9e94c8f (#678 T5b1 merged; CI on `master` green at 1642a86, the later merges
docs and the client only; the sandbox's baseline the light gates). **T5b2 cut in two at its
start** (the user, asked with the reason: the harvest is a new reader of nine tables in
`h3-formats`, a module in `h3-harvest` and three generated packs; the town lane's own format work
needs no original data): **T5b2a** (this) and **T5b2b** (the reader, the harvest and the three
packs, `cross-lane`, the original data staged).

- **The hordes' upgrade** (decision 29; survey §16.2): a building's `auto_with`, the buildings
  whose being ever built builds it (*agent's call*: an explicit list, not VCMI's `mode: auto`
  with the requirements, since the original's upgraded horde requires only the upgraded dwelling,
  survey §16.8, and requirements wait for increment 2). `BuildingDef::auto_with` (keys),
  `Building::auto_with` (a `BuildingSet`, empty for none); the compiler refuses a key the record
  lacks and the building itself (two tests); no cycle check (a cycle only keeps both unbuilt). T6's
  setup applies it over the ever-built set (design §4), increment 2's construction too.
- **The part's hash v2** (`TOWN_RULESET_HASH_VERSION`): each building's automatic set after its
  forbidden-with building; both pinned hashes re-pinned, a test that the set moves the hash.
- **The synthetic packs' `towns.toml`** (hand-written, invented): `synthetic` the parameters and
  `vale`'s record (levels 1 and 2 without upgrades, every role once, a horde superseding the
  level-2 dwelling and its upgrade forbidden and built with it), `synthetic-ext` the override of
  the moat, the obstacles and the silo. `h3-pack-towns`' `SYNTHETIC_TOWNS`, `SYNTHETIC_EXT_TOWNS`;
  its tests now read the files (T5a1's inline texts, the same records, moved there).
- **Left for T5b2b:** `packs/README.md`'s row (the battle lane's file; it still says no pack has a
  `towns.toml`), the original packs' constants and `ORIGINAL_PACKS`.

### T5b2b1 — the town tables' reader (2026-11-03)

One session, the user present; branch `town/e09-t5b2b1` (`cross-lane`: the battle lane's
`h3-formats`, `h3-harvest` and `installation.md`, as E08 T81b). Starting point: `master` at
2c73790 (#681 T5b2a merged; CI on `master` green there; the sandbox's baseline the light gates);
the original data staged (`Heroes3.exe` SHA-256 `aac39e8c…`, both LODs, the extracted texts, the
maps). **T5b2b cut in two at its start** (the user, asked with the reason: the reader of nine
tables and the harvest of three packs are each a session's work): **T5b2b1** (this) and
**T5b2b2** (the harvest module, the three packs' `towns.toml`, `h3-pack-towns`' constants and
test, `packs/README.md`'s row).

- **The reader** (`h3_formats::exe::towns`, re-exported `TownTables` and `town_data`; the build's
  `KnownBuild::towns`): each faction's building list (a building and the ids it requires) and
  supersede list at its own address, the common supersede list, `map_building_ids` (44 read as
  none), `town_creatures`, the hordes (the base hordes' creature and growth as the file holds
  them; the levels and upgrades are filled at start-up, so not read), `horde_buildings`, the silo,
  the neutral garrison's three rows, the 30 (`cmp eax, 30` at 0x5C0C60). *Agent's calls:* the
  lists as read, direct (the closure is the harvest's or the compiler's, as the part's
  `supersedes`); the requirements kept in the result for increment 2, not harvested now; a list at
  most 64 entries and an entry at most 64 ids (the faction mask's bits).
- **Checked on the user's executable** (a throwaway example, not committed): every value as
  survey §16 dumped it (the counts 35, 37, 36, 37, 36, 36, 34, 34, 35; the supersedes; the slots;
  the hordes; the silo; 33/33/20/13, 8–15, 5–7, 3–5, 1–3; 30). The nine faction lists and the ten
  supersede lists are contiguous (`installation.md` §7.61).
- **Tests:** a synthetic image of two factions read whole; each malformed value refused (a
  building id 44 or negative, a map slot 45, a negative creature, silo or garrison value, a horde
  creature below −1, a growth above `i16::MAX`, a chance above 100, another opcode); a list of 65
  entries, an entry of 65 ids, an unterminated list, unmapped and truncated tables, the supersede
  lists' count; the reader in `robustness.rs`' property over random images.
- **`h3-harvest`:** `Inputs::town_tables` read from the executable (`TownType.txt` is already
  `Inputs::towns`); nothing harvested from it yet, so the packs are unchanged and the harvest's
  `--check` still matches.
- **For T5b2b2:** the arch edge `h3-harvest` → `h3-pack-towns` does not exist (`arch.toml`, a
  `shared/` change) if the harvester is to compile the town files itself as it does the battle
  packs; otherwise `h3-pack-towns`' test of the original packs is the compile check (the queue's
  acceptance).


### T5b2b2 — the content harvested (2026-11-03)

One session, the user present; branch `town/e09-t5b2b2` (`cross-lane`: the battle lane's
`h3-harvest`, its gap-fill and `packs/README.md`, as T5b2b1), stacked on
`shared/e09-harvest-towns-edge`. Starting point: `master` at 3fd3665 (#684 T5b2b1 merged; CI on
`master` green there; the sandbox's baseline the light gates); the original data staged
(`Heroes3.exe`, both LODs, `extracted.tar`, `maps.tar`). **At its start** (the user, asked): the
task whole, one pull request (not base's eight factions first, then AB and SoD); **the harvester
compiles the town files itself**, so a `shared/` pull request first adds `arch.toml`'s edge
`h3-harvest` → `h3-pack-towns` (not `h3-pack-towns`' test as the only compile check).

- **`h3-harvest`'s `towns` module** (`harvest_towns`, run by `main.rs` after the core harvest on
  its core files; separate from `harvest` so the crate's fixture keeps its one-level creatures):
  per faction of `TownType.txt` (base's or, by the gap-fill's `[ab]`, ab's) a record of the
  faction's listed buildings and the Village Hall; the names from `BldgNeut.txt` (ids 0 to 14,
  the Blacksmith's row 17), `BldgSpec.txt` (11 rows a faction: 17 to 26, then the silo 15) and
  `Dwelling.txt` (14 rows a faction), keys `key_of` the names, an upgraded horde `upg_` its
  horde's (*agent's call*, as the upgraded dwellings' "Upg." names); `slot` from
  `map_building_ids` (a slot naming a building the faction lacks, a Castle's fifth guild level,
  gives none); `supersedes` the common list's entries of its buildings and the faction's, direct,
  in the file's order; an upgraded horde `forbidden_with` its horde and `auto_with` its horde and
  its level's upgraded dwelling (the level the horde creature's index among the seven, checked
  against the dwelling the horde supersedes); the dwellings' roles from their ids, the others
  from the gap-fill; the seven levels with their upgrades from `town_creatures` (a creature of
  another pack written `pack:key`, `base` refusing one outside its closure); the silo's nonzero
  amounts; the moat. `[town_params]` in `base`; `sod` the overrides where SoD's siege differs.
  Refused: a count, a name or a horde that does not fit, a key or a slot twice, a supersede of a
  building the faction lacks, a role twice or on a building the faction lacks. The three files
  compile with `h3-pack-towns` on the harvested core files in every allowed combination and
  both profiles before they are returned. A report line: requirements, costs and the hordes'
  growth not harvested.
- **The gap-fill's `[towns]`** (`gapfill.rs`' `Towns`, optional elsewhere, three sources
  `exe-town-income`, `exe-town-siege`, `exe-town-roles` citing the survey's §8.3, §8.4, §9.4,
  §11.1, §16.6): `params` (gold, the halls' 500/1 000/2 000/4 000, the Grail's 5 000, the reveal's
  5, the demoted Capitol building 12); `moat` (RoE and AB harmless, SoD damaging, the Stronghold
  absent with natural obstacles under RoE and AB, damaging without under SoD; design §3); `role`
  (21 entries: the halls, forts, Tavern with morale +1, silo and Grail of every faction; the
  Castle's Brotherhood, the Rampart's Fountain, the Tower's Lookout and Grail, the Inferno's
  Stormclouds, the Dungeon's, Stronghold's and Fortress' Grails, the escape tunnel, the Fortress'
  21 and 22 by TD-8's code).
- **The packs:** `base` 1 958 lines (the parameters, eight records), `ab` Conflux's (its
  elementals `base:` and their upgrades `ab`'s, the silo `"base:mercury"`), `sod` nine overrides;
  each record the survey's count + 1 (36, 38, 37, 38, 37, 37, 35, 35, 36). `core.toml` and
  `battle.toml` unchanged.
- **`h3-pack-towns`:** `BASE_TOWNS`, `AB_TOWNS`, `SOD_TOWNS`, `ORIGINAL_PACKS`; the test
  `the_original_packs_compile_with_every_faction` (RoE, AB and SoD; both profiles; the nine
  records' levels, upgrades and counts, the Village Hall, the demoted Capitol, the moats by
  edition, the parameters, a role of each kind, the Castle's upgraded horde).
- **Tests of the harvest:** three synthetic factions (two in `base`, one in `ab` with its creatures
  in `ab`) with seven levels each: the files' shape and determinism; every refusal above; the
  gap-fill's errors; a file that does not compile.
- **Docs:** `packs/README.md` (the file's row, the harvest writes three files).

### T5a2 — `GameRuleset` takes the town part, replay format v9 (2026-11-03)

One session, the user present; branch `town/e09-t5a2` (`cross-lane`: `h3-game`'s `ruleset.rs`
and tests, `h3-sim`'s encounter and adventure replay, `h3-cli`'s adventure loader, census and
file format, `h3-map`'s tests and examples, the adventure replays, `clients/Cargo.lock`).
Starting point: `master` at 2030ef5 (#689 T5b2b2 merged; the sandbox's baseline the full engine
verify, green; CI's full run there failed one `h3-heroes` property case, fixed on `master` by
1981203, merged in); the original data staged (`Heroes3.exe`, both LODs, `extracted.tar`,
`maps.tar`). **At its start** (the user, asked): the task whole, one pull request (not
`GameRuleset` and its callers first, then the replay format).

- **`GameRuleset::new(core, adventure, battle, towns)`** and `towns()`; `RulesetMismatch` also
  when the town part was compiled on another core or a faction with a town record has no
  `[[siege]]` record in the battle part (every town can be besieged). `h3-game` depends on
  `h3-towns` (`arch.toml` allowed it since T4b).
- **The callers:** `h3-sim`'s encounter passes `TownRuleset::empty` (an encounter's game has no
  towns); `h3-sim`'s adventure tests, `h3-cli`'s census and `h3-map`'s tests and both examples
  the original part (`h3_pack_towns::ORIGINAL_PACKS`; RoE alone for `h3-map`'s one-pack case);
  `h3-cli`'s `load_rules` reads each folder's `towns.toml` beside `adventure.toml` and
  `battle.toml` (an absent file an empty text). `h3-game`'s own tests take the empty part
  (`no_towns`).
- **Adventure replay format v9:** `ReplayHeader::town_ruleset_hash`, checked after the adventure
  part's; the file's `town_ruleset_hash` line (*agent's call*: optional when read, so a v8 file
  reads with 0 and is refused by its version, as a v1 file without a difficulty; a v9 file
  without it is refused by the hash). The 39 replays re-recorded from their own commands with
  `h3 adventure run --script` (a scratch script asserting that only `format` and the new line
  moved); their checkpoints unchanged: the state hash does not read the parts.
- **Tests:** `h3-game`'s `the_parts_must_belong_to_the_core` (the town part on another core) and
  `a_town_needs_its_siege` (a town record built in code: the imp and the gog `t:town`'s levels 1
  and 2, the roles every record needs; no siege record); `h3-sim`'s tampered-header case
  `town_ruleset_hash`; `h3-cli`'s loader test (the part from the folders is the embedded one, nine
  records) and its format test (a v8 file without the line, a v9 file without it).
- **Docs:** the queue's T6 entry (round 6), the adventure lane's versions line (replay format v9).

### T6a — the town's record and the store of towns, town state hash v1 (2026-11-03)

One session, the user present; branch `town/e09-t6a` (the lane's own crate only). Starting point:
`master` at 9b07bea (#694 T5a2 merged; CI's full run there green; the sandbox's baseline the
light gates); the original data not staged (not read). **At its start** (the user, asked): T6 cut
in three, not in two (the record, store and derivations; the setup) nor kept whole: **T6a** the
record, the store and its hash; **T6b** the derivations (§6); **T6c** the setup's functions (§5).

- **`Town`** (`record.rs`; ADR-0028 point 2, design §4): the faction, `built` (every building
  ever built, the original's +0x158) and `forbidden`, private behind `new` and three reads;
  `standing(record)` derives the standing set (`built` less the union of each ever-built
  building's closed `supersedes`, the original's +0x150; E09 decision 29). Not `Copy`: the record
  grows with later increments.
- **`TownStatic`** (the name if the map gives one, the events in the map's order) and
  **`TownEvent`** (the timed part as `TimedEventSetup`'s fields, the buildings as a `BuildingSet`
  of the town's faction, the seven creature counts). *Agent's calls (change freely):* the
  **alignment is not kept** (only the build's faction draw reads it, from `TownSetup`, T6c/T7b;
  a later reader adds it with the hash); the event has **no message field** (its town and its
  index in the town's list name it; the text is the client's); its buildings are resolved to the
  faction at the build (T6c), not kept as the map's slots.
- **`Towns`** (`store.rs`; ADR-0028 point 4): `Option<Arc<Store>>`, the store's
  `Vec<Arc<Town>>` and `Arc<[TownStatic]>`: a fork costs one reference count, a map without
  towns none (*agent's call*, as the game's timed events: E08 T85b's `bench-check` note, an
  `Arc` always costs a fork 19 Ir); a write (T6b's) copies the list of references and the one
  town, the static parts stay shared. `new` (`None` above `MAX_TOWNS`, 65 536: one per `TownId`;
  an empty list is the empty store), `len`, `is_empty`, `get`, `town_static`. **Not here**
  (*agent's call*): `set_built` and `capitol` (design §4's) come with T6b, their first callers
  (the Capitol's limit, the hall level); a crate-internal write nobody calls would be dead code.
- **Town state hash v1's first encoding** (`TOWN_STATE_HASH_VERSION` stays 1: nothing recorded
  one): the version, the count, each town's faction handle and both sets' bits, then each static
  part (the name as an option, the events: players, the two flags, first day, interval,
  resources, buildings' bits, the seven counts), in `TownId` order. Pinned in its test
  (`0xA02D_E3DA_4142_AB03`), each field shown to move it.
- **`BuildingSet`** gains `difference`, `is_empty` and `iter` (index order).
- **No rule id:** the record and the store are state; the rules over them (`town.toml`) come
  with T6b. 100 % lines and regions on `h3-towns`.

### T6b — the derivations, `town.toml` (2026-11-03)

One session, the user present; branch `town/e09-t6b` (the lane's own crate and catalog only).
Starting point: `master` at 4a46251 (#698 T6a merged; the sandbox's baseline the full engine run,
CI's on `master` still running at the start); the original data not staged (not read). **At its
start** (the user, asked whole or cut in two): whole, the work split between the agent and one
sub-agent in a worktree of its own, merged into one pull request. The agent wrote the common
reading, the test part and the income and siege half; the sub-agent the reveal, the Tavern and the
Capitol's limit with the store's write (its branch `town/e09-t6b-cap`, local, merged as 693caaa).

- **The common reading** (`derive.rs`, crate-internal `Roles`): a town with its faction's record
  in the part, its standing and ever-built sets and their roles in index order. Every derivation
  takes `(&Town, &TownRuleset)` (*agent's call*, design §6's note); a faction without a record
  reads as a town without buildings (no dead branch: the setup never makes one).
- **Income** (`income.rs`; TWN-INC-001, -002): `hall_level` (the highest standing `hall`, 1 with
  none, as the original pays 500 untested), `daily_income` (a list of resources in the core's
  order, none 0, the sums saturating: the hall's and the Grail's gold, a standing silo's
  resources; a silo paying gold summed with the hall's; nothing without the parameters).
- **The siege** (`siege.rs`; TWN-SIE-003 to -005): `fort_level`, `siege` (the moat from a
  Citadel; `structures` the standing buildings not forbidden less the hall-1 building, decision
  28; the Escape Tunnel ever built; `bonuses` one `TownBonus` per building ever built with a
  change, its **morale while it stands, its luck once ever built**: the format doc's readings and
  survey §9.4, the code reading's 0x44AE60 and 0x44B2D0; the inbox note's "standing for both"
  differs only for a superseded luck building, which no pack has), `siege_obstacles` (the
  record's flag from a Citadel), `defender_bonus` (the `defender` values ever built, summed
  saturating). **The compiler refuses a record with more than 8 buildings of a morale or luck
  change** (`TownBonuses`' bound; the part's hash unchanged: no field moved), and
  `h3-pack-towns`' format doc says so.
- **Capture and the views** (the sub-agent's; `capture.rs`, `view.rs`, `store.rs`; TWN-CAP-001's
  reveal, TWN-CAP-002, TWN-VEW-002's Tavern): `reveal` (the part's radius, a larger building's
  ever built, `All` over any; 0 without parameters), `capitol_limit` (the ever-built set after the
  limit: the standing hall-4 building out, `capitol_demoted_to` in), `has_tavern` (a standing
  `tavern`), `Towns::capitol` (a standing hall-4 building), `Towns::apply_capitol_limit` (the
  public write for T8a, whether the town changed; nothing written without a change) over the
  crate-internal `set_built` (copy-on-write: the list of references copied once a fork, the one
  town rebuilt, the others and the static parts shared; an id outside the store copies nothing).
  *Its calls:* the Capitol found as the standing `hall = 4` building; a custom set's lone Capitol
  limited puts the City Hall into the ever-built set (the original sets only its built bit;
  nothing reads the halls over the ever-built set).
- **`docs/rules/catalog/town.toml`** created: TWN-INC-001, -002, TWN-SIE-003 to -005,
  TWN-CAP-001 (the reveal; capture is T8a's), -002, TWN-VEW-002 (the Tavern; the levels T8c's),
  all `implemented`, sourced from the survey's readings, VCMI and the wiki as it cites them, E04c's
  batch 3g for the siege's morale and luck.
- **Test part** (`fixture.rs`, cfg(test)): a core and a part built in code, the Keep's record
  with every role of increment 1 (a luck building superseded, so the ever-built reading shows),
  the Marsh's small one (a silo paying gold, a Citadel), no record for the Wild. 100 % lines and
  regions on `h3-towns`; no version moved.

### T6c — the town's setup at the build (2026-11-04)

One session, the user present; branch `town/e09-t6c` (the lane's own crate, catalog, quirks file;
the common `docs/stubs.toml`). Starting point: `master` at ec2ceb7 (#700, #701, #702, #703
merged; the sandbox's baseline the full engine run; the original data not staged, not read). **At
its start** (the user, asked whole or cut in two): whole, the code by one sub-agent in a worktree,
the docs and the review by the agent, one pull request; the agent's four calls below confirmed.

- **The types** (`setup.rs`): `SlotSet` (the map editor's 41 slots; `buildings(record)` resolves
  each slot to the record's building at it, a slot without one nothing), `TownFaction` (`Fixed`,
  `Random { alignment }`, resolved by the game in T7b), `BuildingsSetup` (`Standard { fort }`,
  `Custom { built, forbidden }`), `GarrisonStack` with `GarrisonCreature` (`Creature`,
  `Random { level, upgraded }`), `TownEventSetup` (`TownEvent`'s fields, the buildings by slots),
  `TownSetup`, `BuiltTown` (the `Town`, its `TownStatic`, the garrison as `ArmyStack`s) and
  `TownSetupError::NoTownRecord`.
- **The buildings** (`town_buildings_setup`, TWN-SET-003): the record's Village Hall always ever
  built (a forbidden one too, as the original builds it first: *the sub-agent's call*); a custom
  set's forbidden slots closed over `forbidden_with` to a fixpoint, its built slots with what they
  supersede, less the forbidden; a standard set's Fort with the map's flag, an owned town's Tavern
  (**the confirmed call:** the `tavern` building that supersedes no other, not the Castle's
  Brotherhood) and, only with the flag, the level-1 dwelling and the level-2 one on one draw
  `1..=100 <= level_2_dwelling_chance` (no draw without the flag), a neutral town's both
  dwellings without a draw; then the hordes' upgrade (`auto_with` all ever built, not forbidden)
  to a fixpoint. Several buildings of one role: the first by index. **T-6 fixed** as
  `TWN-QUIRK-5` in the new `docs/rules/quirks-town.md` (the standing set is derived, so no code;
  its regression test `twn_quirk_5_a_custom_sets_horde_never_leaves_its_dwelling_standing`).
  `TWN-STUB-2` registered (the start's Grail and Shipyard forbids, an empty set for now).
- **The garrison** (`town_garrison`, TWN-SET-004): a custom one in the map's order, a random
  stack the record's creature of its level (**the confirmed call:** a level the record lacks, or
  0, leaves the slot empty; an upgrade at a level without one gives the base), a count of 0
  empty, no draw (TD-5); none: an owned town's empty, a neutral town's drawn row by row for the
  levels the record has (the chance, then the count), the stacks from slot 0. The import (T7a)
  gives each slot once.
- **The whole town** (`town_setup`): the record and the parameters (else the typed error, nothing
  drawn), **the garrison's draws first, then the buildings'** (the confirmed call: the original's
  `town_setup` order), the name and the events with their slots resolved to the faction's set.
- **`town.toml`**: TWN-SET-003, -004 (`implemented`; the code reading, batch TWN-1's rows 1 and
  2, VCMI 1.7.5, the wiki for the garrison, a `design` source for the confirmed calls). The test
  part's Keep has map slots, four creature levels, a horde with its upgraded horde and a shrine of
  its own (two-step fixpoints); 16 tests, 100 % lines and regions on `h3-towns`; no version moved.

### T7a1 — the setup's new fields and the import, map import v17 (2026-11-04)

One session, the user present; branch `town/e09-t7a1` (`cross-lane`: `h3-game`'s `setup.rs` and
`lib.rs`, `h3-map`'s import and tests, the replays' headers, `h3-cli`'s census literal (the ai
lane's), `docs/formats/h3m.md`, the adventure lane's HANDOFF and inbox). Starting point: `master`
at 03b6601 (#707 merged; CI green there; the sandbox's baseline lost to a VM restart; the original
data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`); #708
(E08 T93a, adventure part v16) merged in during the task, cleanly. **At its start** (the user): T7a
cut in two at the queue's seam, T7a1 (the setup's fields, the import, map import v17, the census)
now and T7a2 (`validate`'s checks, §13) next; until T7b draws them, the build refuses a setup that
needs a draw (`TWN-STUB-3`). The code by one sub-agent (resumed once after a VM restart), the
review and the docs by the agent.

- **`h3-game`** (`src/town.rs`, the lane's): `PlayerStart` (`FactionSet`, a `u64` bit set;
  `FactionChoice`; `MainTown`), `PlayerSetup::start`, `GameSetup::towns`, `allowed_heroes`,
  `hero_masks`, `prisoners`, `GameHeroSetup::hero_type: Option<_>`; `h3-towns`' setup types
  re-exported, so `h3-map` needs no new crate edge. `GameSetupError::HeroDraw` (`RandomHero(hero)`,
  `MainTown(player)`; *the sub-agent's call*: the player, not the colour, as the game's other
  errors), checked in `validate` after the heroes' count and before any hero's type is read
  (`town::hero_types`, the only place a type is taken out of the option); everything else new is
  ignored by the build: no hash moved.
- **`h3-map`** (`src/import/town.rs`, the lane's; `import/mod.rs` wires it): the players' starts
  (TWN-SET-001's import half: factions, choice, main town, the flag), the towns (classes 77, 98;
  the faction table `FACTIONS` here, not in `codes.rs` as §7 said; `ImportError::Alignment`,
  `TWN-QUIRK-1`; `ImportError::MainTown`, `TWN-QUIRK-4`'s import half), class 70 without a type,
  TWN-SET-010 (a placed hero right of an entrance moved onto it), the pool's inputs.
  `Unsupported::HeroAtMainTown` and `RandomHero` removed. **Found by the census:** a shipped town's
  entrance is blocked and visitable, so the hero-in-a-town test leaves the entrance out (a moved
  hero stands there; the old writer test that wanted `HeroInTown` there changed). *The sub-agent's
  calls:* the import's order (the static layer, the main towns, the towns, the heroes, the banned
  sets, the events, the pool's inputs); no faction allowed gives `Random` (T7a2 refuses it); a hero
  type the ruleset lacks leaves the pool's inputs; every disposed hero's mask kept (0xFF too); the
  town's name as Windows-1252 (a copy of `h3-formats`' table, no new edge); a town event's first
  day + 1 as the timed events'; a random hero takes no custom-hero entry, and an AB spell byte
  naming a spell gives that spell alone. Survey §4.2 checked by the agent: RoE maps never set 144
  to 155; AB and SoD maps take their 156 bits as given.
- **Versions:** map import v17 (`MAP_IMPORT_VERSION`; E08 T93b took v16 first, #711 merged
  before this pull request, so it bumped again); the 39 adventure replays' `import_version` lines
  (nothing else moved). Game state hash v22, the adventure's hashes and the replay format
  unchanged.
- **Docs:** `town.toml` TWN-SET-001, -010; `quirks-town.md` TWN-QUIRK-1, -4; `docs/stubs.toml`
  TWN-STUB-3; `h3m.md`'s "Import" (the towns and the census); the design note's §7 "As built" and
  §19 (T8a takes the adventure part after T93a's v16).
- **The census** (`original_data`, 160 shipped maps): every map imports (v15: 8); 10 build whole,
  128 refused by the build for a main town's hero and 22 for a random hero; 1 468 towns (592 fixed,
  876 random, 47 of them aligned), 574 custom building sets, 113 custom garrisons; 176 random
  heroes; 23 heroes moved; 89 prisoners (`h3m.md`). After #711 (E08 T93b) merged in: 156 maps
  import, 4 are refused for an ownable object entered from water (T93b's dwellings), 9 build
  whole, 125 and 22 refused by the build; 1 437 towns (577 fixed, 860 random), 726 players, 86
  prisoners.
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` and `cargo check --workspace --all-targets` pass.

### T7a2 — the setup's checks (2026-11-04)

One session, the user present; branch `town/e09-t7a2` (`cross-lane`: `h3-game`'s `setup.rs` and
its tests' helpers, `h3-map`'s census example). Starting point: `master` at 0387933 (#710 merged;
CI green there; the sandbox's baseline passed with only the light gates, CI having verified the
code; the original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`). **At its start** (the user): `TownCount` and `MainTown`'s owner half move to T8a
(the towns have no owner in the setup and the static map no town list before it); `TownAlignment`
refuses a colour no player has, as §13 says (the original falls through to the owner, then the
draw, as for no alignment; none on the shipped maps), not the import's resolution to none (a map
import bump). The code by one sub-agent in a worktree, the review and the docs by the agent.

- **`h3-game`** (`src/town.rs`, the lane's): `check_starts`, called by `adventure_setup` after the
  heroes' count and before `hero_types` (the `HeroDraw` refusal, `TWN-STUB-3`), one pass per
  invariant in §13's order: `TownFaction(TownId)` (a fixed faction without a town record),
  `TownAlignment(TownId)`, `Choice(PlayerId)`, `MainTown(PlayerId)` (its town none of the setup's),
  `Pick(PlayerId)`, `PickTaken(PlayerId)` (the later player), `RandomHeroOwner(HeroId)`.
  `GameSetup::offer(rules, player)` and `GameSetup::OFFERED_HEROES` (16, the dialog's list) are
  public: none unless the choice is fixed and the main town generates the hero or the random-hero
  flag is set; then the first 16 types in the core's order that the map allows, no placed hero
  and no prisoner has, of a class of the faction, every colour mask of the type with the colour's
  bit. *The agent's calls:* `Choice` also refuses a set holding a faction without a town record
  (`TWN-QUIRK-4`: a draw always gives one) and no choice where the start needs a faction (a main
  town, the random-hero flag, a random hero of its own, a random town aligned to its colour).
  *The sub-agent's calls:* a colour mask test past bit 7 offers nothing (`checked_shr`; the offer
  may be asked before `validate`); towns past a `TownId`'s range are unchecked until T8a's
  `TownCount`; the tests build a ruleset of their own (a second faction and class, a town record,
  the battle tests' siege field), so six helpers of `src/tests/mod.rs` (the adventure lane's)
  became `pub(crate)`; the two T7a1 tests of the build rewritten on it.
- **`h3-map`** (`examples/original_data.rs`, the census): a map a new check refuses is reported by
  the check's name; nothing in the import changed.
- **Versions:** none moved (no hash, no import, no replay).
- **Docs:** `town.toml` TWN-SET-001 (the checks), TWN-SET-002, -006, -008 (new); `quirks-town.md`
  (`TWN-QUIRK-2`, `-3` with T7b's pool: the checks fix neither; `-4`'s faction now checked); the
  design note's §8 (T8a takes the two checks), §13 "As built", §19; `h3m.md`'s census.
- **The census** (`original_data`, 160 maps): as T7a1's after #711: 156 import, 4 refused for an
  ownable object entered from water, 9 build whole, 125 and 22 refused by the build
  (`TWN-STUB-3`); no map refused by a new check (the import sets no pick, so `Pick` is not
  exercised by the shipped maps).
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` and `cargo check --workspace --all-targets` pass. Its
  first coverage run met a failing random case of `h3-map`'s
  `tests::view::a_viewer_learns_nothing_it_cannot_see` (the import refuses its laid-out map,
  `OwnedOnWater`; not T7a2's code): its persisted seed left out, the adventure lane's inbox told;
  the re-run passed.
- **#712** (E08 T94a) merged first: `master` merged in, the census example's conflict resolved
  (a new check's name first, else T94a's `refusal`); the census unchanged.

### T7b1 — the pool and the heroes' draws (2026-11-04)

One session, the user present; branch `town/e09-t7b1` (`cross-lane`: `h3-game`'s `setup.rs`,
`state.rs`, `state/battle.rs`, `lib.rs` and tests, `h3-map`'s census example and a test, the 39
adventure replays, the adventure lane's HANDOFF versions line). Starting point: `master` at
e654837 (#713 merged; CI green there; the sandbox's baseline the light gates; the original data
staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`). **At its
start** (the user): T7b cut in two at the towns, whose owners and objects come with T8a (a random
town's faction falls back to its owner's, a town's setup differs owned or neutral, a main town's
hero stands on its entrance): T7b1 the hero half now, T7b2 the towns' half after T8a's town list
and owners. The code, the bench, the replays and the census by one sub-agent; the review and the
docs by the agent.

- **`h3-game`** (`src/town/pool.rs`, new, the lane's): `HeroStatus` (`Free`, `Taken(colour)`,
  `Prisoner`, `Out`), `HeroPool` (per core hero type its status and colour mask; `new`, `free`,
  `draw`), `draw_heroes` (the placed heroes' types checked, the pool built, the players' Random
  choices, the random heroes' types, the unused picks freed). `src/town.rs`: `needs_faction`
  (factored out of `Choice`'s test), `main_town_heroes` (the narrowed `TWN-STUB-3` refusal);
  `HeroDraw::RandomHero` gone. `setup.rs`: `adventure_setup` returns the pool;
  `GameSetupError::NoFreeHero(PlayerId)`; `validate`'s doc states the two seed-dependent cases.
  `state.rs`: `pool: Arc<HeroPool>`, hashed after the store of heroes; `free_hero_types`.
  `state/battle.rs`: `end_battle` frees the type of every hero it removes from the map (the only
  path a hero leaves the map). Tests: `src/town/tests/pool.rs` (8), the T7a1/T7a2 refusal tests
  rewritten, a beaten hero's type freed and the pool's copy on write (`tests/battles.rs`).
- **The order of the draws** (design §14 "As built"): the players' Random choices a start reads
  (player order), then the random heroes (hero order: the owner's pick for its first, else the
  draw), then the unused picks freed, all before the starting armies. *Agent's calls* (change
  freely): a Random choice nothing reads draws nothing (`needs_faction`), so the fixture maps of 12
  replays (`e08-duel-36`, `e08-army-12`, `e08-xl-144`: two Random players, no town) keep their
  stream; step 2's random towns follow the random heroes (T7b2 appends them); a placed hero's and a
  prisoner's type taken whatever the allowed set (a prisoner's wins a tie), a type two placed
  heroes share held by the last; a type that leaves the map freed even if the map did not allow
  it (0x4D9EC0); unknown types in the allowed list, masks and prisoners ignored by the pool;
  `UnknownHeroType` for a placed hero now before any draw; `NoFreeHero`'s payload the drawing
  player (T7b2's main towns draw too).
- **Versions:** game state hash v23 (the pool). The 39 replays rehashed from their own commands:
  only the hash lines, `final_hash` and `game_state_hash_version` moved; the bot reports untouched
  (they record the adventure part's hash).
- **Bench** (master against the branch, the +2 % gate): fork-boxes-xl-144 +0.41 %, fork-xl-144
  +0.27 %, fork-rewards-36 +0.37 % (about 10 Ir a fork, the extra `Arc`); every adventure workload
  within −0.60 % and +0.69 % of its baseline; nothing re-baselined.
- **The census** (`original_data`, 160 maps): 156 import, 4 refused (`OwnedOnWater`); 31 build
  whole (9 before: the 22 refused for a random hero now build), 125 refused for a main town's hero
  (`TWN-STUB-3`); none refused by `NoFreeHero`.
- **Docs:** `town.toml` TWN-SET-005, -007 (new), -001, -006, -008 (the draws); `quirks-town.md`
  `TWN-QUIRK-2`, `-3` (new), `-4`'s test; `stubs.toml` `TWN-STUB-3` narrowed; the design note's
  §14 (the cut, "As built"), §19; `h3m.md`'s census; the stale "T7b" lines of `h3-towns` and the
  import's module doc now say T7b2.
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` and `cargo check --workspace --all-targets` pass.
  `master` (#714, the client lane) merged in before the run.

### T8a1 — the town as a map object (2026-11-04)

One session, the user present; branch `town/e09-t8a1` (`cross-lane`: `h3-adventure`'s
`lib.rs`, `setup.rs`, `state.rs`, `state/economy.rs`, `state/holds.rs`, `ruleset/objects.rs`,
`testing.rs` and tests, `h3-game`'s `setup.rs`, `state.rs`, `town.rs` and tests, `h3-map`'s
import, `lib.rs`, census example and tests, `h3-cli`'s `src/ai/census.rs` (the ai lane's), `h3-pack-adventure`, `packs/base/adventure.toml`,
`adventure.toml`'s catalog, the 39 adventure replays, the ai lane's two bot reports, the adventure
lane's HANDOFF versions line and `OVERVIEW.md`'s import version). Starting point: `master` at
5dac0d2 (#715 merged; CI green there; the sandbox's baseline the light gates; the original data
staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`). **At its
start** (the user): T8a cut in two and round 8's order changed (T8a1, T7b2, T8a2, then T8b; the
design note §8's cut), the branch stacked on the adventure lane's #716 (E08 T94b, adventure state
hash v20). #716 was behind `master` (both had rehashed the 39 replays: 41 files in conflict), so
`master` was merged into the stacked base in a commit of its own (the replays' lines taken from
T94b with game state hash v23, the stale hashes re-recorded once by this task); once #716 merged
(its own merge of `master` the same code), the branch was rebased onto `master` at 990e07e and the
re-recorded replays kept. The code, the census, the bench and the replays by one sub-agent; the
review and the docs by the agent.

- **`h3-adventure`:** `ObjectKind::Town` and `ObjectKindDef::Town` (ownable, no parameter; the
  part's hash tag 33), `Holds::Town { formation }` (the state hash's tag 9). `StaticMap::towns`
  (the town objects in object order), `town_id(index) -> Option<TownId>`,
  `town_object(id) -> Option<usize>` (*agent's call*: an option, so a foreign id cannot panic);
  `SetupError::Towns` (more towns than a `TownId` names). `AdventureSetup::garrisons` (one army
  per `TownId`, written on the town's object as `army_left`, an empty one written empty;
  `SetupError::Garrisons` for another count, `Garrison { town, error }` for an invalid army);
  `AdventureState::town_garrison(index)`. `start_game` takes `town_radii` by `TownId`
  (`reveal_owned`: a town past the slice the flag radius).
- **The entrance not entered** (`TWN-STUB-4`, *agent's call*): `StaticMap::place` sets no typed
  visit on a town's entrance, so the step, the reach and the legal moves meet a blocker's entrance
  as before the town was typed (the from-above test included) and their loops test nothing more; a
  test compares the reach and the step with the blocker's.
- **The import** (`h3-map`): `codes` maps classes 77 and 98 to `base:town` whatever the subclass;
  `import/town.rs::object` gives the owner (`object_owner`: 0xFF neutral, a colour nobody plays
  refused as every ownable's) and the formation (the map's byte, 0 loose, any other tight).
- **`h3-game`:** `GameSetupError::TownCount` (the setup's towns against the objects of kind
  `Town`, `check_starts`' first check); `MainTown` widened (*agent's call*, no new variant): a main
  town the player does not own at the start (another player's or neutral) is refused as one none
  of the setup's ("not a town of its own"). The build passes empty garrisons and the flag radius
  for every town (`TWN-STUB-5`, lifted by T7b2).
- **Versions:** adventure ruleset part v17, map import v18, adventure state hash v21. The 39
  replays re-recorded from their own commands: only the checkpoint and final hashes,
  `adventure_ruleset_hash`, `adventure_state_hash_version` and `import_version` moved; both bot
  reports regenerated (only the part's hash moved).
- **The census** (`original_data`, 160 maps): 156 import, 4 refused (`OwnedOnWater`); 31 build,
  125 refused for a main town's hero (`TWN-STUB-3`), as before; none refused by the new checks;
  1 468 towns typed, 889 owned at load, 3 with a tight formation. The census's static-layer game
  gives each town a neutral town record (else `TownCount` refuses it).
- **Bench** (the stored baselines, the +2 % gate): the fork workloads +1.13 % to +1.71 %, the
  highest on maps without a town (Learning Stones, boxes), so most likely the base's own creep
  (T7b1's `Arc`, T94b's blocks; the base itself not measured); the rest −0.19 % to +0.71 %;
  nothing re-baselined.
- **Docs:** `adventure.toml` ADV-OBJ-002 (77, 98 typed), ADV-OWN-002 (towns' owners, the
  start's reveal by the game's radii; a source); `stubs.toml` `TWN-STUB-4`, `-5`; `h3m.md`'s
  import and census; the design note's §8 (the cut, "As built"), §19; the queue's T8a1, T7b2 and
  T8a2 entries.
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` and `cargo check --workspace --all-targets` pass. CI's
  coverage failed on #719: the ai lane's census test (`h3-cli`'s
  `ai::tests::the_census_counts_a_known_map`) built its game with no town records, refused by
  `TownCount`; the census now gives each town a neutral record as `h3-map`'s does, and the
  workspace's tests pass (a trap in `TRAPS.md`).

### T7b2a — the towns resolved and stored, game state hash v24 (2026-11-04)

One session, the user present; branch `town/e09-t7b2a` (`cross-lane`: `h3-game`'s `lib.rs`,
`setup.rs`, `state.rs` and tests, `h3-map`'s census example, `h3-cli`'s `src/ai/census.rs` (the
ai lane's), two adventure bench workloads, the 39 adventure replays, the adventure lane's HANDOFF
versions line). Starting point: `master` at ae59f3b (#719 merged; CI green there; the sandbox's
baseline the light gates; the original data staged: `Heroes3.exe`, `H3bitmap.lod`,
`H3ab_bmp.lod`, `maps.tar`, `extracted.tar`); `master` merged in before the pull request (#720,
#721, #722: no version, replay or workload moved). **At its start** (the user): T7b2 cut in two at
the store (design note §14): T7b2a now, T7b2b (the main towns' heroes) next. The code, the census,
the bench and the replays by one sub-agent; the review and the docs by the agent.

- **`h3-game`:** `town::build` (`draw_towns`: step 2's random towns after the random heroes,
  TWN-SET-002; `build_towns`: step 4, `h3_towns::town_setup` by `TownId`, owned or neutral,
  TWN-SET-003, -004, before the starting armies; `start_radius`: TWN-CAP-001's start).
  `check_starts` returns the towns' owners; `draw_heroes` takes them and returns the players'
  resolved factions; `needs_faction` also reads a random town the player owns. `GameState`'s store
  of towns (`Towns`; `GameState::towns`), hashed after the pool; `AdventureSetup::garrisons` the
  built garrisons, `start_game`'s radii each town's reveal. No new error variant: `TownFaction`
  widened (a random town when no faction has a town record), `Choice` widened (a random town the
  player owns).
- **Agent's calls:** step 4 cannot fail after the checks (a town it could not build is left out
  and the adventure would refuse the garrisons' count; no `?` on an error never reached, which the
  coverage gate counted as a region); `Towns::new(..).unwrap_or_default()` (at most `MAX_TOWNS`
  built); an owner that is no player draws as a neutral town (the adventure refuses it later,
  `ObjectOwner`); `Reveal::All` the radius 255 (the town's level on any map up to 181 a side, not
  the other level: `TWN-STUB-5` narrowed, not lifted); the censuses' static-layer games give each
  town the first faction with a town record, fixed (else `Choice` refuses the owned ones); the test
  part gained a Lookout Tower (radius 20) and a Grail that reveals every tile.
- **Versions:** game state hash v24. The 39 replays re-recorded from their own commands: only the
  checkpoint and final hashes and `game_state_hash_version` moved; the bot reports unchanged (the
  part did not move).
- **The census** (`original_data`, 160 maps): 156 import, 4 refused (`OwnedOnWater`); 31 build,
  125 refused for a main town's hero (`TWN-STUB-3`), as before; none refused by a new check. The
  31 build 294 towns: 139 random towns by their owner, 52 drawn, none aligned; 79 neutral towns
  with a drawn garrison (16 drawn empty); no start radius other than 5. The 156 imported: 47
  random towns aligned, 813 not (553 owned at the start); 9 custom sets with a Grail revealing
  every tile (an upper bound: a random town's Grail slot counts), 4 with a Lookout Tower. The
  dwellings' seed-0 counts moved (691 dwellings with guards, 695 before): step 4's draws come
  before them in the setup stream.
- **Bench** (`master` built in a worktree, per function): every fork +0.24 % to +0.36 % (only
  `AdventureEnv::fork` +6 Ir and the state's drop +3 Ir a fork: the field's clone and drop); boxes
  and stones re-baselined (+2.07 % against their stored counts, `master` already +1.71 %), the
  other forks +1.37 % to +1.91 % against theirs (fork-rewards-36 the next to reach the gate); the
  rest −0.19 % to +0.68 %.
- **Docs:** the design note's §14 (the cut, "As built"), §19; `stubs.toml` `TWN-STUB-5` narrowed;
  `town.toml` TWN-SET-002, -003, -004, TWN-CAP-001; the queue's T7b2b and T8a2 entries; the
  inboxes of the battle and adventure lanes (two failing property cases on `master`'s code).
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` passes; `cargo test --workspace --exclude xtask
  --all-targets` passes but two random property cases independent of this task: `h3-map`'s
  `tests::view::a_viewer_learns_nothing_it_cannot_see` (fails the same way on `master`) and
  `h3-battle`'s `a_world_shows_its_viewer_the_battle_and_reads_nothing_hidden` (a crate that
  depends on nothing this task changed); both seeds in the lanes' inboxes.

### T7b2b — the main towns' heroes (2026-11-05)

One session, the user present; branch `town/e09-t7b2b` (`cross-lane`: `h3-game`'s `lib.rs`,
`setup.rs`, `state.rs` and tests, `h3-map`'s census example and `tests/towns.rs`). Starting point:
`master` at f720585 (#724 merged; the sandbox's baseline the light gates on 1c906d6, CI's
fingerprint there; the original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`,
`maps.tar`, `extracted.tar`, and the editor `h3maped.exe` with its help file for the reading; the
user's computer's shell unavailable, staging worked). **At its start** (the user): T7b2b whole;
the agent's two calls (a second hero on an entrance refused; the guild's spells at the start in
increment 4) proposed, and the user asked whether a hero right of a town's entrance is not rather
the town's garrison hero, with another visiting. Two readings by sub-agents (the executable;
VCMI 1.7.5, the format's docs and the wiki), then the code, census and gates by one sub-agent; the
review and the docs by the agent.

- **The reading** (`docs/exe/map.toml`, `structs.toml`; survey §4.5, §4.6, TD-10): the visitor,
  never the garrison hero: `load_hero_body` stores a hero's start tile as its template's
  visitable tile, one column left of the object's anchor, and the 23 shipped heroes lie on their
  town's anchor; `placed_heroes_setup` moves such a hero onto the entrance, `hero_show` makes it
  the visitor; nothing at a new game writes a town's garrison hero (`town_init` 0x5BDE80 clears
  it; the swap 0x5BE450 and the moves 0x4B9FC0, 0x5BE390 are the town screen's and the computer's,
  not reached from `new_game_load`); the map's town record has no hero field. VCMI 1.7.5 the same
  (`placeHeroesInTowns`, `initVisitingAndGarrisonedHeroes`; its `placeStartingHero` asserts on an
  entrance already taken). `main_town_heroes` never tests the entrance: the new hero is drawn over
  a visitor, which stays the visitor and then learns the guild's spells (T-7). The guild's spells
  are drawn in `setup_towns` (`town_setup_spells` 0x5BE600) before the main towns' heroes;
  `town_heroes_learn_spells` 0x5BE030 (read whole) teaches the visitor, then the garrison hero,
  with a spell book in a town with its Mage Guild, the levels up to Wisdom's + 2 and the highest
  built; VCMI teaches none at the start (TD-10). The editor's code and help file not read (the
  help file is compressed).
- **`h3-game`:** `town::heroes::main_town_heroes` (TWN-SET-009; design note §14 "As built
  (T7b2b)"); `check_starts` returns the towns' owners and each generating main town's entrance
  (`MainTown` also refuses one without an entrance); `draw_heroes` returns `Drawn` (the pool, the
  types, the factions, the unused picks) and frees nothing; `HeroPool::take`; the main towns'
  heroes chained after the setup's in `adventure_setup`'s loops and `GameState::new`'s sexes and
  level seeds; the heroes' count includes them. `HeroDraw`, `GameSetupError::HeroDraw` and the
  stub gone (`TWN-STUB-3` lifted).
- **Rules, quirks, stubs:** TWN-SET-009 in `town.toml` (TWN-SET-005 to -008's forward references
  refreshed); `TWN-QUIRK-6` (T-7: a main town's hero over a visitor, refused through the
  adventure's `SetupError::HeroTile` with `Occupied`); `TWN-STUB-6` (the guild's spells at the
  start, increment 4); `TWN-STUB-5`'s census over the 156 maps.
- **Tests:** 10 in `town/tests/heroes.rs` (a human's pick; the pick used by a random hero; a
  computer's pick ignored and freed after step 3; the draw; `NoFreeHero`; the heroes' ids,
  tiles and draw order; the first entrance on the map; two heroes on an entrance; no generating
  town keeps the stream).
- **The census** (`h3-map`'s `original_data`, seed 0; `h3m.md`): all 156 imported maps build
  (31 before; 125 refused for a main town's hero), none refused by a new check; 523 main towns'
  heroes, all drawn (the import picks none), 84 on maps of 36 tiles a side; 33 with a spell book
  in a main town with its Mage Guild; 1 437 towns; 54 heroes levelled at load (304 levels).
- **Replay:** none added: the replays play committed maps of `fixtures/maps/` written by the
  engine; the shipped maps are original files (ADR-0009) and CI has none.
- **Bench:** inside its gate, nothing re-baselined: forks +1.38 % to +1.91 % against the stored
  counts (the same count on the base f720585: `master`'s, not this task's), day-start +0.75 %,
  the rest within ±0.4 %.
- **Gates:** the scoped verify with `--with-original-data` passes but `check-lane` (by design,
  `cross-lane`); `check-lane --cross-lane` and `cargo test --workspace --exclude xtask
  --all-targets` pass. No version moved; the 39 replays unchanged.

### T8a2 — the step onto a town and an undefended capture (2026-11-05)

One session, the user present; branch `town/e09-t8a2` (`cross-lane`: `h3-adventure`'s
`state.rs`, `reach.rs`, `command.rs`, `observe.rs`, `lib.rs`, `ruleset/objects.rs`,
`state/passage.rs`, `state/economy.rs` and tests; `h3-game`'s `state.rs`, `command.rs`,
`observe.rs`, `legal.rs`; `h3-map`'s `tests/view.rs`; the adventure lane's `TRAPS.md`). Starting
point: `master` at a415b1d (#727 merged; the sandbox's baseline the light gates there; the
original data staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`,
`extracted.tar`); `master` (#728, #729, #730) merged in on the way. **At its start:** the agent
proposed a cut in two (entering with the reach and the bench, then the capture and the game's
composition); the user chose T8a2 whole. The adventure side with the bench, then the game's
composition with the gates, by two sub-agents in turn; an independent review by a third, its
fixes by a fourth; the docs by the agent.

- **`h3-adventure`** (design note §9 and §11 "As built (T8a2)"): `src/town.rs` (`TownReveal`,
  `StaticMap::town_at`, `town_visitor`, `besieges`, `town_defended`, `visit_town`,
  `capture_town`); the entrance a typed visit (`Effect::Town`; ADV-OBJ-004 now reads the town);
  `visit_at`'s town branch; `check_step` and `passage` refuse another team's visitor (a flight
  passes over it); `AdventureOutcome::capture`; `AdventureEventKind::TownCaptured`;
  `set_owner`; `start_game` takes `&[TownReveal]` (`TWN-STUB-5` lifted).
- **`h3-game`:** `town::capture::capture` first in `settle_outcome`'s loop (`adventure_reveal`,
  which was `start_radius`; the Capitol's limit with `holds_another`); `GameEvent::TownChanged`.
- **The reach:** the walking loop is `master`'s. The first cut tested the visitor in its occupied
  branch: +7.7 % on `reach-xl-144`; then a `#[cold]` `unmark_visitors` pass before the loop on a
  map with towns, `visit_at` `#[inline(never)]` (the trap in the adventure lane's `TRAPS.md`).
  Bench against a built `master` worktree (the same toolchain), nothing re-baselined:
  `reach-xl-144` −0.03 %, `reach-lifted-xl-144` +0.35 %, `reach-sea-xl-144` −0.79 %,
  `random-play-duel-36` +0.31 %, `random-play-sea-36` −0.42 %, `random-play-economy-36`
  −0.45 %, `random-play-rewards-36` +0.02 %.
- **Rules, stubs:** TWN-VIS-001 (the step's three cases), TWN-VIS-002 (the visitor) in
  `town.toml`; TWN-CAP-001's and -002's summaries amended; `TWN-STUB-4` and `-5` lifted;
  `TWN-STUB-7` (a defended town and another team's visitor refused until T8b's siege) added;
  case 2's `AllyInTown` not built (`TWN-STUB-1` stays E10's).
- **Tests:** `h3-adventure` six in `state/tests/towns.rs` (an own town entered and ending the
  walk; an undefended town of another player or none entered for its capture; a defended one
  refused until the siege; another player's visitor refused, an allied one met; a capture's owner,
  garrison, reveal and events; a reveal of all on two levels), the reach's property test with up
  to three towns; `h3-game` seven in `town/tests/capture.rs` (a neutral and a blue town captured;
  a Capitol demoted when the new owner holds another; kept when not, or when the other is
  neutral's; a town the store lacks; an own town entered; a defended one refused; the legal
  moves), the Grail's start test on both levels.
- **The review** (independent, no behaviour bug): the visitor's test and `visit_at` read
  different objects of a shared entrance tile (fixed); `TownChanged` told to every player (given
  the capture's viewers); the limit could run after no capture (only after one); the game's
  reveal untested (tested); two texts (`TWN-STUB-7`'s reason, `legal_moves`' doc).
- **The fixes** (the sandbox restarted during them; the worker resumed from its transcript):
  `besieges` through `AdventureState::tile_index` (a first version shared `tile_object`'s lookup
  and failed `bench-check`: `reach-lifted-xl-144` +2.76 %, `random-play-duel-36` +2.01 %; with its
  own copy +0.38 % and +0.29 %), the test `the_visitors_town_is_the_tiles_object_the_step_meets`
  (a town and a sawmill on one entrance, both orders); `capture_town` returns the `Viewers` it
  told (`None`: nothing captured, no limit); `TownChanged { town, capitol, seen_by }` projected
  as `BattleEnded`; `a_capture_reveals_the_towns_radius_and_tells_its_change_where_it_is_seen`
  (48×48, a neutral town with a Capitol and a Lookout: the radius 20; blue told nothing).
- **Gates** (after the fixes): `cargo fmt`, workspace clippy; the scoped verify with the
  original data every step passed (coverage 100 %, determinism, `bench-check`, `original-data`)
  but `check-lane` (by design on a `cross-lane` branch); `check-lane --cross-lane`; the workspace
  tests. No version moved; no replay touched.

### T8b1 — the siege's battle: its setup from the town and its end (2026-11-05)

One session, the user present; branch `town/e09-t8b1` (`cross-lane`: `h3-adventure`'s
`command.rs`, `state.rs`, `state/piles.rs` and tests; `h3-game`'s `state/battle.rs` and tests;
`docs/rules/catalog/game.toml`). Starting point: `master` at 9d65268 (#733 merged; CI green
there, the sandbox's baseline the light gates; the original data staged: `Heroes3.exe`,
`H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`). **At its start** (the user): T8b
cut in two, T8b1 the siege's battle (its setup and its end, testable by starting such a battle
directly, `TWN-STUB-7` kept), T8b2 the step onto a defended town (the reach and the bench, the
merge, the defender's three kinds). The code, tests and docs by the agent; an independent review.

- **`h3-adventure`:** `BattleStart::town: Option<TownId>` (`battle_at` gives none);
  `src/town.rs`' `end_town_battle` (a garrison's siege without a hero: the garrison written,
  empty after the win, else the survivors; the capture held by `reward_after`) and `town_won`
  (the capture for the hero's owner unless allied by then; nothing for a hero off the map), the
  arms of `end_unowned_battle` (`ObjectKind::Town`) and `give_won` (first, by
  `StaticMap::town_at`).
- **`h3-game`:** `src/town/siege.rs`: `siege_setup` (the facts, the defending hero `raise`d, the
  native terrain, the special ground on the entrance), `mana_after`, `siege_experience` and
  `SIEGE_EXPERIENCE`; `start_battle` calls `siege_setup` before the obstacles' draw;
  `settle` derives an ended battle's town (its state's siege, its tile's town) and passes it to
  `end_battle` (a new parameter; the tests' calls pass `None`), which caps the defender's spell
  points, adds the attacker's 500 and, against a visitor, holds `Rest::Won(entrance)` for the
  winning attacker. No version moved (the town is derived, `OpenBattle` unchanged); no replay
  touched; no hot path (the reach and the step are T8b2's).
- **Readings and calls:** the special ground stays in a siege (`installation.md` §7.26: "in every
  case", against the design's agent's call "no special battlefield"); the attacker's +500
  (ADR-0023 point 1) re-read at 0x46A416, not in the design note: TWN-SIE-006; the defender's
  skills raised at most `primary_max` (the original raises the hero's own skills, which its
  effective skills hold within the highest). **A gap found:** the base pack's walled Stronghold
  draws the usual obstacles (TWN-SIE-005), but the original's placement keeps them off the walls
  by a branch for town type 6 not read (0x466147 to 0x466194) and the battle's generator knows no
  walls (the battle refuses an obstacle on a wall): `TWN-STUB-8`, none drawn, a note in the battle
  lane's inbox. The full pack set (with SoD) never meets it.
- **Rules, stubs:** TWN-SIE-006 and TWN-CAP-003 new; TWN-SIE-003 to -005 and GAM-BAT-001
  amended; `TWN-STUB-8` registered; `TWN-STUB-7`'s reason names T8b2.
- **Tests:** `h3-game`'s `town/tests/siege.rs` (nine: a siege's state equal to the expected setup
  with and without a fort and on a holy ground; a faction without a native terrain, a battle
  without a town and one naming a town the store lacks; the defending hero raised; `raise`'s
  limits; the defender's spell points after it; an attacker's win over the garrison captured
  after the rests, a loss and a draw; over the visitor after the winner's level-up, its type
  free, the leftovers lost; the +500; two sieges played to their ends, a retreat behind the walls
  and a win without a fort); `h3-adventure`'s `state/tests/towns.rs` (the garrison's end, the
  capture's outcome, an allied town and a hero off the map). The town tests' fixtures take a core
  and a town pack (`towns_core_pack`, `rules_of`).
- **The review** (independent, no behaviour bug in the reachable cases): a start whose tile is not
  the town's entrance would be built as a siege but end as a field battle (`siege_setup` now
  fights no siege there, tested; T8b2's queue entry says so); an attacker's win whose hero leaves
  the map beats the garrison without a capture (as a mine's; TWN-CAP-003 says so, the original's
  path not read); tests added for a defender's win without the +500 and the spell points after an
  attacker's win; five texts (`AdventureOutcome::capture` and `reward_after`, the game's
  `capture` module, `Rest::Won` and `run_rest` with TWN-CAP-003, TWN-SIE-005's terrain without a
  native one).
- **Gates:** `cargo fmt`; the scoped verify with the original data passed every step (clippy,
  coverage 100 %, determinism, `bench-check`, `original-data`) but `check-lane` (by design on a
  `cross-lane` branch); `check-lane --cross-lane`; `cargo check --workspace --all-targets`.
