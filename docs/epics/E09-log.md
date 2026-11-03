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
