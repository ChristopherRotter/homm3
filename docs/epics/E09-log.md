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

### T8b2a — the step onto a town its garrison defends: the siege's start (2026-11-05)

One session, the user present; branch `town/e09-t8b2a` (`cross-lane`: `h3-adventure`'s
`state.rs`, `state/passage.rs`, `reach.rs`, `ruleset/objects.rs` and tests; `h3-game`'s
`legal.rs`). Starting point: `master` at 0e2f186 (#738 merged; CI green on 618a232; the sandbox
restarted during the setup's full baseline, so CI's green run stood for it; the original data
staged: `Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`; the user's
computer's shell unavailable, staging worked; `master`'s #740 merged in before the pull request).
**At its start** (the user): T8b2 cut in two at
the visitor: T8b2a the town its garrison alone defends (the step, the reach with the bench, the
defender's two kinds without a hero, end-to-end tests), T8b2b the visitor (the merge and its
events, `Defender::Hero`, a step onto the visitor, `TWN-STUB-7` lifted). The code, tests and docs
by the agent; an independent review.

- **`h3-adventure`:** `src/town.rs`' `town_visit(index, player) -> Option<Visit>` (was
  `town_defended`): `Enter` for a town of the player's team or an undefended one, `Neighbour` for
  one of another team or none with a creature in its garrison, none while a hero of another team
  stands on the entrance (`TWN-STUB-7`); `visit_at` takes a town's mode from it, so the walking
  step, the passage (a flight passes over it; from a pass node no visit, `NoStop`) and the reach
  (a destination, as a monster's) read one answer; `visit_town(part, (player, hero), (index,
  mode))` gives a neighbour visit's `siege_start` (`Defender::Garrison { owner, army }` for an
  owned town, `Defender::Guards { army }` for a neutral one, `battle_at` the entrance with
  `town: Some(TownId)`); `StaticMap::town_formation` and `holds_formation`. The step pays, drops
  (ADV-MOV-006) and stays as for a monster; nothing else changes until T8b1's end.
- **`h3-game`:** `town::siege_setup` puts the town's formation on a garrison without a hero
  (design §8's byte); `legal_moves`' doc. No version moved; no replay touched (no fixture map
  has a town).
- **The reach:** its walking loop untouched (`visit_at` stays out of line). Bench against a built
  `master` worktree (the same toolchain), nothing re-baselined: `reach-xl-144` −0.008 %,
  `reach-lifted-xl-144` −0.004 %, `reach-sea-xl-144` −0.000 %, `random-play-duel-36` −0.005 %,
  `random-play-sea-36` +0.001 %, `random-play-economy-36` −0.003 %, `random-play-rewards-36`
  −0.010 %, `random-play-dwellings-36` −0.003 %; after the coverage fix (`visit_at`'s
  combinator) `reach-xl-144` −0.013 %, `reach-lifted-xl-144` −0.007 %, `random-play-duel-36`
  −0.008 %.
- **Rules, stubs:** TWN-SIE-001 (who defends: the garrison alone now, the visitor with T8b2b);
  TWN-VIS-001 (the defended case besieged) and TWN-VIS-002 (T8b2b's) amended; `TWN-STUB-7`
  narrowed to a visitor of another team.
- **Tests:** `h3-adventure`'s `state/tests/towns.rs`
  (`a_town_its_garrison_defends_is_besieged_from_the_tile_before`: blue's and a neutral town, a
  walker, a water walker and a flyer: the reach's destination, the outcome's battle, the hero
  outside with the reach's points, no event, a flyer's pass and its `NoStop` from a pass node;
  an own garrisoned town entered; the formation), the reach's property test (its doc: a
  garrisoned town now a neighbour's visit); `h3-game`'s `town/tests/siege.rs` (a step's siege
  equal to the expected setup in the town's tight formation, blue's side or the neutral seat's,
  the hero outside with the legal moves' points; played to a capture with the attacker outside
  and the next step entering, blue's and neutral; a retreat behind a neutral town's walls, the
  system actor playing it) and `town/tests/capture.rs` (a defended neutral town besieged; the
  legal moves reach a defended entrance).
- **The review** (independent, no behaviour bug: the step, the passage, the reach, a boat, a hero
  starting on the entrance, a shared entrance tile, the battle's end all agree): a walled neutral
  siege never played to its end (tested: a retreat); a flight's visit from a pass node untested
  (tested); a `None` from `siege_start` would leave the step paid without a battle (unreachable,
  said in its doc); two texts (the batch's "with its points", the T8a2 "As built" pointing to
  §10).
- **Gates:** `cargo fmt`; the workspace tests; the scoped verify with the original data passed
  every step (clippy, check-rules, coverage 100 %, determinism, `bench-check`, `original-data`)
  but `check-lane` (by design on a `cross-lane` branch); `check-lane --cross-lane`; `cargo check
  --workspace --all-targets`. A first verify found two regions never run (`visit_at`'s `?` on a
  kind without a visit, `siege_setup`'s `if let` without an `else`): written as a combinator and
  an `unwrap_or`.

### T8b2b — the siege against a town's visitor and the garrison's merge (2026-11-05)

One session, the user present; branch `town/e09-t8b2b` (`cross-lane`: `h3-adventure`'s
`army.rs`, `state.rs`, `state/manage.rs`, `state/monsters.rs`, `state/passage.rs`, `reach.rs`,
`ruleset/objects.rs` and tests; `h3-game`'s `command.rs`, `state.rs`, `state/manage.rs` and
tests; `h3-cli`'s `src/adventure/`; `h3-map`'s `tests/play.rs`). Starting point: `master` at
7f8c8c5 (#741 and #742 merged; the sandbox restarted during the setup's full baseline, so CI's
green run on `master` stood for it; the original data staged: `Heroes3.exe`, `H3bitmap.lod`,
`H3ab_bmp.lod`, `maps.tar`, `extracted.tar`). **At its start** (the user, asked): T8b2b kept
whole (the agent had proposed a cut at the merge); the merge's events by a new
`ArmyRef::Garrison(TownId)` and `StackChanged` per slot (of three: that, one summary event, the
hero's slots plus a new garrison event). The code, tests and docs by the agent.

- **`h3-adventure`:** `town_visit` gives `Visit::Neighbour` also when a hero of another team
  stands on the entrance, whoever owns the town (`TWN-STUB-7` lifted); `check_step`'s and the
  passage's hero branch, meeting such a visitor (`besieges`), visit the town (`visit_at`; a
  flight's from a stop node only, passing otherwise; a boat's step stays `sail_entry`'s landing
  refusal); the reach's cold `unmark_visitors` pass kept (its doc), the walking loop untouched.
  `siege_start` gives the visitor `Defender::Hero` (its army as it stands). The merge:
  `town::merge(core, (slots, most), army, garrison)`, pure (survey §9.2's rounds; the room the
  joins' `room` with a stack's most, ADV-ARM-007, so `room`, `join_at` and `Room` became
  `pub(crate)`); `AdventureState::merge_garrison(core, part, town) -> Vec<ArmySlot>` writes both
  armies (the garrison only when changed; `hero_mut` new) and returns the changed slots, the
  visitor's then the garrison's. `ArmyRef::Garrison(TownId)` and `ArmyCommandError::Garrison`
  (the adventure's `held` refuses it).
- **`h3-game`:** `town::merge` (`src/town/siege.rs`), called by `settle_outcome` before
  `start_battle` for a siege whose defender is a hero: the merge, the battle's defending army the
  merged one, each changed slot told as `StackChanged` to its army's owner (the visitor's to its
  owner, the garrison's to the town's, a neutral town's to nobody; `stack_at` reads a
  garrison's slot) before `BattleStarted`. The manage check `admits` refuses a command naming a
  garrison on either end. `StackChanged`'s doc.
- **Other crates:** `h3-cli`'s slot text `t<town>:<slot>` (format and parse; the round-trip
  test); `h3-map`'s plans' two matches on `ArmyRef`.
- **Rules, stubs:** TWN-SIE-002 new (the merge, with the batch's row 3 and VCMI's refuted order);
  TWN-SIE-001 (the visitor defends) and TWN-VIS-002 (the step onto the visitor the siege)
  amended; `TWN-STUB-7` lifted.
- **Tests:** `h3-adventure`'s `state/tests/towns.rs`
  (`another_teams_visitor_defends_its_town_besieged_from_the_tile_before`: every owner, with a
  garrison or none, a walker, a water walker and a flyer: the reach's destination, the siege's
  `Defender::Hero`, the hero outside with the reach's points, a flight's pass and its `NoStop`;
  `the_visitors_town_is_the_tiles_object_the_step_meets` now a siege;
  `the_garrison_merges_by_the_greatest_gain_until_none_gains`: batch TWN-1's row 3 exactly;
  `a_garrison_stack_joins_where_the_army_has_room_and_a_worthless_one_never`: a free slot, a
  stack of its creature, a creature worth nothing or unknown, a stack past the most, no gain, an
  empty army; `the_state_merges_a_towns_garrison_into_its_visitor_and_names_the_changed_slots`),
  the army tests (the refusal, its text), the reach's property test's doc; `h3-game`'s
  `town/tests/siege.rs` (`a_step_onto_a_towns_visitor_merges_the_garrison_into_its_army_and_besieges_it`:
  the legal moves, the three `StackChanged` to blue alone before `BattleStarted`, the battle's
  setup with the merged army; `a_siege_against_a_visitor_keeps_its_merged_army_or_captures_the_town_with_the_attacker_outside`:
  red's retreat, then a horde's win with the levels answered;
  `a_neutral_towns_garrison_slots_are_told_to_nobody`;
  `a_visitors_siege_with_nothing_to_merge_tells_no_stack`) and `tests/manage.rs` (the refusal
  on either end); `h3-cli`'s round trip.
- **The bench** against a built `master` worktree (the same toolchain; nothing re-baselined):
  `reach-lifted-xl-144` +0.24 % and `random-play-duel-36` +0.16 %, both the passage's own count
  (+0.7 %, its hero branch's new test; every other function equal), `reach-xl-144` −0.03 %; the
  fork workloads equal to `master`'s (both +1.38 % over their baselines in this sandbox: the
  heap's state, not the change). Every workload inside the gate.
- **The review** (independent, read only): one behaviour bug, fixed: the garrison's slots were
  told to the visitor's owner; now to the town's (a neutral town's to nobody; tested with a
  visitor placed on a neutral town). Its test gaps: a tie between a gain with room and a swap's
  (tested); survivors in merged slots after a defender's win with losses (the battle's own
  write-back, not new here). The first verify's coverage found two regions never run (`visit_at`'s
  `?` on `town_visit`, now total; the passage's own `NoStop` arm, now the shared object path).
- **Gates:** `cargo fmt`; the scoped verify with the original data passed every step (clippy,
  check-rules, check-stubs, coverage 100 %, determinism, `bench-check`, `original-data`) but
  `check-lane` (by design on a `cross-lane` branch); `check-lane --cross-lane`; `cargo check
  --workspace --all-targets`; `h3-cli`'s tests.

### T8c1 — the towns' income and the observation's towns (2026-11-05)

One session, the user present; branch `town/e09-t8c1` (`cross-lane`: `h3-adventure`'s
`lib.rs`, `observe.rs`, `state.rs`, `state/economy.rs` and the observe tests; `h3-game`'s
`lib.rs`, `observe.rs`, `state/day.rs`; `h3-map`'s `tests/view.rs`; `game.toml`). Starting
point: `master` at c584f7e (#746 merged; CI green there; the setup's full baseline passed in the
sandbox; the original data staged). **At its start** (the user, asked): T8c cut in two at the
Rogue: T8c1 the income and the observation's towns, T8c2 the Rogue's Visions with its part
parameter, `docs/06-security.md` and the inbox notes. The code, tests and docs by the agent.

- **The income:** `h3_game`'s `town::add_income` (`src/town/income.rs`) adds each town the
  player owns (the adventure's owners by `TownId`) its `h3_towns::daily_income` into
  `pay_income`'s vector after the heroes', so the towns join the mines and the heroes in one
  `ResourceGained` per resource, from day 2; a neutral town pays nobody, a town captured today
  its new owner from the next day (and its old owner no more).
- **The views:** `ObjectView::town: Option<TownId>` (`StaticMap::town_id` of every seen object;
  an object added in play is never a town). `GameObservation::towns` from `town::view::town_views`
  over the adventure observation's objects (so in `TownId` order): a town a tile of which the
  viewer has explored; one only on the fringe, which `faithful` shows as an object, keeps its
  object's `town` but gets no view (the popup reads an explored tile only, *agent's call*).
  `TownView` (anchor, owner, faction, name, fort, capitol, `GarrisonView`, `owner_facts`);
  `GarrisonView { level, stacks, army }`: the stacks packed in slot order (the creature from
  level 1, the word from 2, the count at 3), `army` the garrison by slot to the owner's team only
  (as a mine's `GuardsView`; T8b2b's garrison `StackChanged` change it); `TownOwnerView { hall,
  income }`. The level: `OWNER_LEVEL` for the owner's team (`h3_adventure::allied`, made public),
  else the viewer's own towns with `has_tavern`, at most `TAVERN_LEVELS`, counted once an
  observation. `project` was §12's already (T8a2, T8b1).
- **Rules, stubs:** TWN-VEW-001 and -003 new; TWN-VEW-002 (packed stacks, the view), TWN-INC-001
  and -002 (where they are paid) amended; GAM-RES-003's and -004's "Not here" lines; `TWN-STUB-9`
  registered (a Rogue's Visions, T8c2).
- **Tests:** `h3-game`'s `town/tests/income.rs` (the hall, the Grail and the silo from day 2
  and every day; several Capitols at the start; a town beside a mine in one gain; a neutral town
  pays nobody, a captured one its new owner the next day; blue's town captured: its income and
  its view change hands) and `town/tests/view.rs` (each town's public facts and the owner's,
  linked from its objects; the garrison's levels by the viewer's own taverns, at most two, the
  stacks packed; a neutral town nobody's team; a town unexplored, and one only on the fringe of
  a profile with ADV-QUIRK-1, not shown).
- **The bench:** `day-start-xl-144` against `master` built in this sandbox: 1 295 034 → 1 295 278
  Ir (+0.02 %, the towns' loop over a map without towns); the scoped verify's `bench-check`
  inside the gate on every workload (its +0.64 % on `day-start-xl-144` is `master`'s own in this
  sandbox, the fork workloads' +1.38 % as T8b2b's). No version moved: the state is unchanged
  and no fixture map has a town, so no replay moved.
- **The review** (independent, read only): no behaviour bug or leak; its test gaps fixed (the
  fringe guard untested, a town beside a mine, a capture from another player with the view's
  change of hands), its docs (`ObjectView::town`'s fringe case, §12's "by slot" now packed) and
  nits (the words' level its own constant); its note that a seen town's `TownId` counts the
  towns before it (ADR-0028 point 3) goes to T8c2's security section. The first verify's
  coverage found four regions never run (a lookup's `?` each, now one; `add_income`'s index).

### T8c2 — the Rogue's Visions, the security doc, the inbox notes (2026-11-06)

One session, the user present; branch `town/e09-t8c2` (`cross-lane`, the user's consent at the
start: `h3-heroes`' `derive.rs`, `lib.rs` and its test; `h3-game`'s `observe.rs` and
`tests/mod.rs`; `h3-harvest`'s gap-fill, `gapfill.rs`, `towns.rs` and its test; the 47 adventure
replays' `town_ruleset_hash` line) and `shared/e09-security-towns` (#752, `docs/06-security.md`).
Starting point: `master` at a425d00 (#750 merged; CI green there; the setup's baseline lost to a
VM restart, CI's run stood for it; the original data staged with the file tool, the user's
computer's shell unavailable). **At its start** (the user, asked): T8c2 kept whole, with
`h3_heroes::power` and the replays' header in the town PR. The code, tests and docs by the agent.

- **The reading** (`hero_visions_reach` 0x4E5E10 read whole, `town_popup` 0x4167A0's loop): the
  level +0x129, or 3 with creature 0x8F in the army; the Power +0x478 held to 1–99; the radius the
  spell table's expert effect (3) times it, at least 3; true on the hero's level with
  `dx² + dy²` strictly below the radius's square; `town_popup` asks each hero of the shown player
  at the town's **entrance** (+0x5). The other three callers are the hero, garrison and monster
  popups (`docs/exe/map.toml`, inferred; the adventure lane's inbox).
- **The part** (v3; decision 31): `[town_params]`' `visions_factor`, `visions_min_radius`; any
  pack's `[town_visions] creatures` (each of the naming pack's closure, once), compiled into
  `TownParams::visions_creatures` in handle order; creatures without parameters `MissingParams`.
  The harvester: `[towns.visions]` in the gap-fill (source `exe-town-visions`), the factor and
  radius into base's parameters, each creature into its own pack's file (the AB pack's Rogue).
  The synthetic pack 2 and 4, the synthetic expansion's `lurker`.
- **The view:** `town_views` takes the game state; `visions` lists the viewer's own heroes on the
  map with a Visions creature and their squared radii (`h3_heroes::power`, new, HER-ADV-001),
  taken once an observation and only when a town of another team needs them; `in_visions` tests
  the entrance (`StaticMap` object's `entrance`). Such a viewer gets `OWNER_LEVEL`'s counts and
  `owner_facts`, never `GarrisonView::army`. `TWN-STUB-9` lifted; TWN-VEW-003 and HER-ADV-001
  amended.
- **Tests:** `h3-towns`' compile (handle order, closure, unknown, twice, without parameters, each
  field in the hash, v3 pinned), `h3-pack-towns`' (the synthetic and the original packs: RoE
  without the Rogue, AB and SoD with it; strictness), `h3-harvest`'s (both packs' tables, an
  unknown creature, one twice), `h3-heroes`' `power`, `h3-game`'s `town/tests/view.rs` (the
  radius's edges, strict; the creature needed; a neutral town; the least radius; from the
  entrance of a three-tile town, not its anchor; only the viewer's own heroes; `in_visions`'
  level).
- **The replays:** the 47 adventure replays' `town_ruleset_hash` line (0x3003da8dd5ff664f), nothing
  else; all verify.
- **The bench:** the scoped verify's `bench-check` failed `day-start-xl-144` at +2.26 % on its
  baseline; `master` built here at the same path length gave 1 295 278 Ir (+0.66 %, T8c1's
  figure), the branch 1 315 827 (+1.6 %), with every counted function the same: glibc's
  `_int_malloc` alone +12 % (the heap's state after the town part's few new allocations at the
  build; the observation is not in the step). Re-baselined with the reason in the workload's
  description; the trap in the lane's `TRAPS.md`. Every other workload inside its gate.
- **The review** (independent, read only): no behaviour bug or leak; its test gaps fixed (the
  entrance against the anchor, a neutral town by Visions), its docs (the catalog's rules, the
  `OWNER_LEVEL` doc, a long doc line); its nit on the harvester's pack (a creature's pack is base
  or ab by construction) left.

### T9a — the writer's town objects, the town map, its replays (2026-11-06)

One session, the user present; branch `town/e09-t9a` (`cross-lane`: `h3-map`'s `writer.rs` and
`tests/` (`mod.rs`, `towns_map.rs`), `fixtures/maps/`, `fixtures/replays/adventure/`, `h3-cli`'s
`tests/adventure_replays.rs`, `docs/formats/h3m.md`; two match arms in the battle lane's
`h3-experiment`). Starting point: `master` at 2f7b9c8 (#752, #753 merged; CI green there; the
setup's `verify-once` found CI's fingerprint). **At its start** (the user, asked): T9 cut in three
as E08's T96 (decision 32). The writer and the map by the agent; the replays' seed search and the
scripted replays by two narrow sub-agents in turn (scratch probes, not committed).

- **The writer** (`h3_map::writer`, the town lane's `src/writer/town.rs`; `h3m.md`, "The town
  lane's additions"): `MapObject::AnyTown` (a town of a fixed faction or random, class 77
  `AVCranx0.def`; a random town's SoD alignment; a player's `MainTown` with its generate-hero flag,
  at most one, owned; a name; garrison codes that may be `random_stack`'s, level 1's `0xFFFE` to
  level 7's upgraded `0xFFF1`) and `MapObject::RandomHero` (class 70, `ahrandom.def` with a hero's
  masks: all 36 of the shipped maps' random heroes; a placed hero's body of type `0xFF`, left out
  of the header's hero list). The header's main town: the flag, SoD's generate-hero flag and
  town-type byte (the faction's subclass, `0xFF` random: 382 of the 160 maps' 553 main towns, the
  other 171 hold 0), the entrance's position (the template's visitable tile, finding 3).
  E04c's `Town` is written as an `AnyTown` without these (its bytes unchanged; `write_town` kept
  for the extras). Refused: an alignment of a fixed town or to a colour nobody plays, a main town
  unowned, its entrance off the map or a player's second, a random stack of count 0, both objects
  in RoE, a name the format cannot hold. *Agent's call:* new variants rather than new fields of
  `Player`, `Map` or `Town` (their literals are in the battle, AI and client lanes' crates; the
  variants touch two exhaustive matches in `h3-experiment` only); the player's `random_town` and
  `random_hero` header flags stay false (the import reads `random_hero`, no fixture needs it).
- **The map** `fixtures/maps/e09-towns-36.h3m` (`tests/towns_map.rs`, `fixtures/maps/README.md`):
  SoD, 36 × 36 of grass; red (Castle) and blue (Stronghold); red's Orrin (ten archangels), Valeska
  (five halberdiers), Sorsha (eight Rogues, Power 1: Visions' radius 3 reaches blue's Tower) and a
  random hero; blue's Yog (fifteen Gorgons) right of the Tower's entrance, moved onto it
  (TWN-SET-010), and Gurnisson. Eight towns: red's main Castle "Haven" (the halls to the Capitol,
  a Fort, a Tavern, a Resource Silo, the Grail building; it generates red's starting hero), red's
  random town (random stacks of level 1 and upgraded level 2), a neutral Inferno (garrison drawn)
  and Necropolis (empty), neutral random towns aligned to blue and drawn, blue's Tower (Gremlins,
  Stone Gargoyles) and blue's main Stronghold (a Capitol, empty, no hero generated). The test reads
  the file back, imports it (the starts' choices and main towns, the heroes, each town's faction,
  buildings, garrison, name and owner) and builds it for seed 1: the random towns red's Castle,
  blue's Stronghold and a faction with a town record; the main town's hero (id 6) on its entrance,
  it and the random hero of a Castle class; the random stacks drawn of their levels; the Visions'
  top level for blue's Tower and the neutral Inferno at red's two Taverns' level. Seed 1 draws
  Ingham (the random hero), Adelaide (the main town's), five Hell Hounds in the Inferno.
- **The golden replays** (`adventure_replays.rs` lists what each meets; 55, the test's least):
  random play `towns-s173-d21`, `-s1702-d21`, `-s3435-d21` (`--seed N --agent-seed N --days 21`),
  the union of a search over the seeds 1 to 5 000 that meets all twelve features counted (an
  undefended capture by each player, a siege against a garrison alone won and lost, against a
  visitor with the merge won (5 of 5 000 games) and lost (46), the Capitol's limit (94), an own
  town entered, a neutral town and the other player's captured, hero battles, heroes 3 and 6
  taking part). Scripted (`--seed 1 --checkpoint-every 5 --script`): `towns-capture-s1` (the
  empty Necropolis captured, its Village Hall in red's next day's 10 000 gold, entered again),
  `towns-siege-s1` (Orrin wins against the Inferno's Hell Hounds, the capture after the battle, the
  town entered the same day), `towns-visitor-s1` (Sorsha against Yog, the Tower's garrison merged
  into his army first, she loses and leaves the map), `towns-capitol-s1` (Sorsha takes blue's
  Stronghold: `TownChanged`, the Capitol limited; red's next day's gold 11 500), `towns-income-s1`
  (seven day starts: red 9 500 gold, 1 wood, 1 ore; blue 4 500).
- *Found on the way* (nothing wrong in the rules): a move stops where the explored area ends, so a
  day's walk may take several moves; the merge fills Yog's slot of each creature, else his first
  free one (TWN-SIE-002, not the garrison's order); no game of the search has a monster battle
  (the map has none); 3 of 5 000 games besiege a visitor without a merge, each after the garrison
  had merged or been emptied by a capture. The map's `GORGON` (code 102) was first named an Orc
  (88); the map kept its Gorgons, the names fixed.
- *Agent's calls (change freely):* the map's heroes, armies, towns and positions; 21 days and the
  seeds; the scripts' paths.
- **Gates:** the scoped verify (without the original data: only `maps.tar`, `extracted.tar` and
  `Objects.txt` staged, for the templates and the main towns' census) passed every step it runs
  but `check-lane`, which fails by design on a `cross-lane` branch (`check-lane --cross-lane`
  passed): coverage, determinism with the 55 replays, bench-check (no engine code changed).
- **Versions:** none (the writer is not read by the game; no import, state or replay format
  change).

### T9b1 — the plans play towns (2026-11-06)

One session, the user present; branch `town/e09-t9b1` (`cross-lane`: `h3-map`'s `tests/`
(`play.rs`, `view.rs`, `fixtures.rs`, `towns.rs`), its `Cargo.toml` (`h3-towns` a dev-dependency,
which `arch.toml` already allowed) and `proptest-regressions/tests/play.txt`). Starting point:
`master` at bdbd6c6 (#756, #757 merged; CI green on 3b18baa, the same engine code; the setup's
baseline lost to a VM restart, replaced by that run). **At its start** (the user, asked): T9b cut
in two (decision 33), the plans first. Built by the agent alone; no original data needed.

- **The plans' towns** (`TownPlan`, `Plan::towns`, drawn last by `towns()`, so that every earlier
  draw and a plan without towns stay as they were; the committed fixtures unchanged): in three
  plans of four one to four towns, each of a fixed faction (98, modulo the version's factions) in
  three of four or random (77, a SoD alignment to a playing colour in one of two), owned in two of
  three, its garrison the editor's (one of six), none at all (two), one weak stack of a level-1
  creature (two) or seven slots of any base creature (one), the standard set with or without a
  Fort or a custom set of the halls to the Capitol with or without one, a tight formation in one
  of two; in one of three visited at load (the entrance on a hero's tile, the town its owner's,
  so that sieges meet visitors with garrisons to merge). `towns_laid_out` (after every other
  object but the boats, a template each: `towns.rs`' `town_template`, now `pub(super)`): the
  entrance one to four rows above a hero, two columns either side, its anchor on the map, the
  entrance on land, no tile a hero's (but a visitor's entrance) or another object's; the body as
  the reader gives it back (the identifier and required spells from AB on, nine bytes of banned
  spells), so the round trip holds.
- **The steps:** `Step::Town` and its runs of 16 to 64 in `steps()`: in a battle as `Fight` (a
  siege's either side, the system's for a neutral garrison), a decision settled as `seek` does,
  else an option that ends on a town's entrance (another's in three steps of four when it has
  one), else one that gets nearer to a town not its own, else `EndTurn`; on a map without towns
  as `Step::Hire` (first, before its settling, which would decline every hire question and moved
  the dwellings' cover test: `camps_deserted` 0).
- **The checks:** `check_towns` after every command (each town's one owner, the adventure's,
  changes only by one `TownCaptured` naming it, never to its old owner, the captured garrison
  empty; the store's towns and factions kept, a record rebuilt only at a capture, told by
  `TownChanged` with the store's Capitol); in `check_invariants` no hero on the entrance of a town
  of another team or none (`h3_adventure::allied`); `check_town_battle_end` for a siege against a
  garrison without a hero (`check_unowned_battle_end`'s town arm: won, the garrison empty and the
  capture told in the battle's last command unless the attacker left the map or its owner's
  decision holds it; else the owner kept and the survivors by slot, none after a draw; no
  removal, no treasury change); `check_income` adds each owned town's `h3_towns::daily_income`;
  `check_isolation` lets a siege's merge change the besieged visitor's army as its
  `StackChanged`s tell (`told_army`; the failing seed committed in `play.txt`) and a capture's
  reveal grow its captor's explored tiles in another player's command; `landing` takes a
  capture's `TownCaptured` and `TilesRevealed`; the forks' property compares the parent's store
  of towns after a fork played other steps. `the_plans_visit_dwellings_and_hire`'s kinds skip the
  towns (a town's template is no `KINDS` entry: the lane's `TRAPS.md`).
- **The cover test** `the_plans_capture_and_besiege_towns` (512 plans, three to six towns each):
  360 towns built; 357 own towns entered; undefended captures 17 neutral, 59 of another player;
  sieges against a garrison alone 50 (won 32, lost 16) and against a visitor 73 (won 25, lost 43);
  11 merges; 133 captures in all; 5 Capitols limited. Each asserted above zero.
- **`view.rs`** draws its plans without towns (`townless`, and the boat test's strategy cleared):
  its `check_told` refuses a town's events today ("a plan without towns told …"), and `vary`
  reads kinds from templates; both are T9b2's.
- *Found on the way* (nothing wrong in the engine): every failure was the tests' own (the landing
  pattern of a capture, the income without towns, the isolation without the merge, the unowned
  battle's end without a town arm). Stressed once with 400 cases of the two properties and 200
  of the invariants with three to six towns per plan, all passing (then set back to 48). CI's
  first run on the pull request found one more with its own seed: a town's owner, playing its
  garrison's side, ended the siege by its own command with the attacker's win, and the capture
  revealed around the town to the attacker in another player's command; `check_isolation` lets a
  captor's explored tiles grow (only grow) in a command with its `TownCaptured`. The case was
  replayed here from CI's log (failing before the fix, passing after; too large to commit), then
  700 more cases with three to six towns per plan passed.
- *Agent's calls (change freely):* the draws' weights, the four building sets, the weak garrisons'
  creatures, the offsets, the runs' length, the cover test's 512 plans and its town-heavy draw.
- **Test time** (`h3-map`'s lib tests, debug, two threads here): 121 s with the towns, the cover
  test 28 s of it.
- **Gates:** the scoped verify (no original data staged) passed every step it runs but
  `check-lane`, which fails by design on a `cross-lane` branch (`check-lane --cross-lane`
  passed).
- **Versions:** none (tests only).

### T9b2a — the told views meet the plans' towns (2026-11-06)

One session, the user present; branch `town/e09-t9b2a` (`cross-lane`: `h3-map`'s `tests/view.rs`,
`tests/play.rs`, whose `town()` is now `pub(super)`, and `proptest-regressions/tests/view.txt`). Starting point: `master` at 1b1792b
(#758 merged; CI green there; the setup's `verify-once` found CI's fingerprint). **At its start**
(the user, asked): T9b2 cut in two, the told views first (decision 34), with the agent's call for
T9b2b told in the question (the hygiene property's towns draw nothing public; a town removed only
after every seen one). Built by the agent alone; no original data needed.

- **`check_told`** follows the town events (it refused them before): `TownCaptured` is told to its
  new owner or to a viewer that saw the town's object, whose owner it changes (the object loop's
  `flagged`); `TownChanged` only after the town's capture in the same batch; a garrison's
  `StackChanged` (`ArmyRef::Garrison`, the merge) only to a viewer that saw that garrison by slot,
  applied to it as a client would. A town's view stays once shown, its anchor, faction, name and
  fort as they were, its owner and Capitol as it is told, its garrison by slot as the merge's
  slots say unless a battle of the viewer's ended or the town was captured; a town comes into
  view only where the viewer's explored tiles grew. A siege's effects on a garrison another
  viewer sees are not followed there: they show untold (`docs/06-security.md`), and the state
  check below holds the view to the state.
- **`check_town_views`** (for every player after every command, and at the start; as
  `check_hire_sources` for the dwellings): each object a viewer sees names its town if it is one;
  a town's object shows its type, anchor, footprint, owner and `TownId` and nothing else; each
  town a tile of which the viewer explored has a view in `TownId` order with the state's anchor,
  owner, faction, name, fort and Capitol and its garrison by the viewer's level, read apart from
  `town_views` (the owner's team or a Visions hero's entrance in reach: the top level; else the
  viewer's own towns with a standing Tavern, at most 2), the slots to the owner's team alone, the
  hall and income at the top alone; a town only on the fringe has an object and no view.
- **The properties:** `views_move_as_the_events_say` draws `plan()` with its towns;
  `boat_visits_are_told_as_they_happen` no longer clears them (its doc's 117th case is the one it
  was without towns); `a_viewer_learns_nothing_it_cannot_see` keeps `townless` until T9b2b.
- **The cover test** `town_views_move_as_the_events_say` (256 cases of plans with three to six
  towns, the first two the first player's and the first of the standard set, so that it holds two
  Taverns or a Capitol): views by level 45 163, 3 994, 232 and 35 324 (0 to 3), 930 of a neutral
  town with its stacks, captures told to their captor 66 and to another viewer 106, 2 Capitols'
  changes, 8 merges' garrison slots, 25 towns coming into view; each asserted above zero. A
  siege's effects seen untold by a third viewer at level 1 or more met none (a lost siege at a
  town such a viewer sees is rare), so it is not asserted. Without the bias 256 such cases met no
  level 2 and no Capitol's change.
- **Mutations** (the engine changed here and back, the cover test run): the taverns' level capped
  at 1 (`check_town_views`), a capture told to its captor alone (`check_told`'s owner of a
  town's view), the merge's garrison slots untold (`check_told`'s garrison by slot): each
  failed.
- *Found on the way* (nothing wrong in the engine): the first draft passed at once, and the 48
  cases of the property met captures 13 times and one merge, hence the cover test. Stressed once
  with 400 cases of plans with three to six towns: a viewer's explored tiles grew in another's
  command on the same day, a capture's reveal to its captor when the siege's last command is the
  system's or the defender's (T9b1 met it in `check_isolation`); `check_told` now lets a capture
  told to the viewer as the new owner widen its sight as a new day does (the explored tiles only
  grow; heroes and boats may show in them). Then 800 such cases passed (then set back to 48).
- *Found on `master`* (the full lib tests here, a random seed): the hygiene property (without
  towns) failed where another player's hero left guards at a mine the viewer saw, of another
  creature in the other game (`vary` changes the other players' armies; the guards' hint is
  public, ADV-OWN-003). The same seed fails on `master`'s `view.rs`. `outcome` now compares the
  guards' hints another player's command changed (`GuardsChanged`'s kind, not its viewers), so
  that the comparison ends there as at a battle; the seed committed in `view.txt`; 200 cases of
  the property passed (then set back).
- *Agent's calls (change freely):* the cover test's cases and bias, its counts.
- **Test time** (`h3-map`'s lib tests, debug, two threads here): 218 s; `master`'s 185 s in the
  same sandbox (T9b1's 121 s were another machine's); the cover test 42 s of it.
- **Gates:** the scoped verify (no original data staged) passed every step it runs but clippy
  (one collapsible `if`, fixed; clippy on `h3-map`'s targets then clean) and `check-lane`, which
  fails by design on a `cross-lane` branch (`check-lane --cross-lane` passed); after the two fixes
  above the lib tests, clippy and fmt again.
- **Versions:** none (tests only).

### T9b2b — the hygiene property meets the plans' towns (2026-11-06)

One session, the user present; branch `town/e09-t9b2b` (`cross-lane`: `h3-map`'s `tests/view.rs`,
`tests/play.rs`, whose `LEVEL_1`, `TOWN_BUILDINGS`, `TOWN_BLOCKED` and `TOWN_ENTRANCE` are now
`pub(super)` and whose town object is `add_town`, shared with `view.rs`, and
`proptest-regressions/tests/view.txt`). Starting point: `master` at 0178a2c (#761 merged; CI green
there; the setup's `verify-once` ran the full local verify, green). **At its start** (the agent,
told to the user): kept whole, the seam offered (the plans' towns with the checks, then `vary`'s
changes); the user's computer's shell did not start, so a sandbox cloned from GitHub. Built by the
agent alone; no original data needed.

- **`hygienic`** (the property's plans; `townless` gone): a random town made a town of a fixed
  faction, a neutral town's editor garrison given (ten of a level-1 creature in its first slot), so
  that the towns draw nothing a viewer may see (decision 34): only an owned standard set's level-2
  dwelling with a Fort, which no view shows; no main town, so the viewer's heroes keep the armies
  and experience the property gives them.
- **`vary` learns towns** (`footprint`: a town's tiles by `TOWN_BLOCKED`, since its template is no
  `KINDS` entry; `kind` skips heroes and towns): a town's entrance stays land, its tiles take no
  hero moved and no blocker added; where the viewer sees none of its tiles, unless it owns the town,
  another owner (after every other ownable object in `owners`' cycle; kept where a hero stands on
  its entrance, its visitor at load), another garrison and buildings (`Unseen::towns`); removed as
  a blocker, only after every town the viewer sees (a seen town's `TownId` counts the towns before
  it), never one it owns; towns added after every other object (`Unseen::added_towns`, a plan's
  town made `hygienic_town`, its owner a playing colour but the viewer's or none) where the viewer
  sees none of its tiles, on no tile of a hero, a boat, another object or a seen monster, the
  entrance on land.
- **`outcome`** also compares the Capitols another player's command changed (`TownChanged`'s town
  and Capitol): a capture takes the town's Capitol away when the captor holds another
  (TWN-CAP-002), maybe in a town the viewer never sees, and the change is told to every viewer that
  sees the town (the agent's call, told to the user at the start).
- **The property:** `a_viewer_learns_nothing_it_cannot_see` draws `hygienic()` and checks the town
  views in both games at the start and after every compared command (`check_town_views`, T9b2a's).
- **Unit tests:** `vary_changes_the_towns_the_viewer_never_sees_only` (four towns: the viewer's own
  kept; the others' owners, garrisons and buildings changed where unseen, the visited one's owner
  kept; removal only after the seen town; a town added where unseen and not over another; the game
  builds on each map); `vary_keeps_a_shipyard_s_boat_tiles` (below).
- *Found on the way, a gap of `master`'s* (the first stress run, 400 cases; the case shrank to a
  plan without towns): another player's boat built at a Shipyard landed on a tile the viewer saw in
  the other game, as an earlier offset it never saw (ADV-SHY-001: the boat tile is the first free
  water offset from the entrance, fixed at the build) had become land there. `vary` now keeps the
  terrain and the objects of every Shipyard's twelve offsets (from each tile it is visited from;
  the offsets read from the packs); the seed committed in `view.txt`.
- **Stressed once** (then set back to 48 cases): 400 cases; 2 000 cases (433 with towns laid out,
  311 seen at the start, 49 with towns changed, 23 removed, 154 added; 9 238 compared steps with
  a town in view, 20 captures told); 2 000 cases of plans with three to six towns (923 with towns,
  699 seen at the start, 117 changed, 43 removed, 173 added; 20 602 steps with a town in view, 49
  captures told, 2 Capitols' changes told). All passed.
- **Mutations of the test** (400 cases of plans with three to six towns each): a town removed
  before a seen one, and random towns left random, each failed; a neutral town's editor garrison
  left drawn passed, also with the viewer holding a Tavern town: a neutral town seen at a level
  of 1 or more with an editor garrison under another seed is rare (one case of 400 saw one at the
  start, with the same seed), so the call stands by its reasoning, not by a failing case.
- *Agent's calls (change freely):* the given garrison; the added towns' share (none to two); the
  owners' cycle shared with the other ownable objects; no cover test (the default 48 cases meet
  towns in about one case of five).
- **Test time:** the property's 48 cases 8 s here.
- **Gates:** the scoped verify (no original data staged) passed every step it runs but
  `check-lane`, which fails by design on a `cross-lane` branch (`check-lane --cross-lane` passed).
- **Versions:** none (tests only).

### T9c — the benchmarks, the env and the census on the town map (2026-11-06)

One session, the user present; branch `town/e09-t9c` (`cross-lane`: the adventure lane's
`benches/adventure/` and `benches/README.md`, `h3-cli`'s `src/adventure/bench.rs`, `cli.rs` and
`tests.rs`, `h3-map`'s census example, `docs/formats/h3m.md`, `fixtures/maps/README.md`; the ai
lane's `h3-sim` tests, consent as decision 7). Starting point: `master` at cf8325e (#764 merged;
CI green there; the setup's `verify-once` found CI's fingerprint: the light gates only). **At its
start** (the user, asked): kept whole (decision 35). The user's computer's shell did not start: a
sandbox cloned from GitHub, the shipped maps and the text tables staged with the file tool
(`maps.tar`, `extracted.tar`).

- **The siege's workload** (`h3 adventure bench --workload siege`; `bench::play_to_siege`,
  `seek_town`, `siege_step`): the acting player's heroes seek the entrances of the towns it does
  not own (as `seek_dwelling` the dwellings'), the random agent answers the rest; each sought move
  is first tried on a fork dropped before the count (as `starts_day`), and the one that opens a
  battle on a town's entrance (every battle there a siege, TWN-SIE-003) is decoded, then played
  inside `siege_step`, the game's step (`step_command`). `BenchResult::visitors` counts the sieges
  a visitor defends. A first version counted the env's whole step: two thirds of it the move's
  decode (its path's reachable-tiles query), which the reach workloads measure; the decode now
  runs before the count. The CLI's test: on the town map 8 sieges in 8 games, 3 of a visitor; on
  the duel (no towns) none.
- **The workloads** (`benches/adventure/`, the README's rows; rustc 1.97.0, this sandbox):

  | Workload | Args | Ir | A unit |
  |---|---|---|---|
  | `siege-towns-36` | 16 games of 7 days | 3 026 337 | about 189 000 a siege (11 of a garrison alone, 5 of a visitor with the merge) |
  | `day-start-towns-36` | 16 games of 7 days | 832 167 | about 52 000 a day start |
  | `reach-towns-36` | 2 games of 28 days | 719 414 631 | about 391 000 a query (1 840) |
  | `fork-towns-36` | 100 000 forks after 7 days | 354 800 055 | 3 548 a fork |
  | `fork-xl-144` (re-set) | as before | 375 500 024 | 3 755 a fork (+1.38 % against T86b2's 370 400 024) |

- **Per function** (callgrind, for the descriptions): a siege's step is about 80 % the battle's
  start (the stacks placed: `h3_battle::state::closest_free` alone 46 %; the battle heroes 16 %),
  20 % the walk and its reveal; the siege's setup and the merge are each under 2 %. A reach query
  on the town map costs 2.7 times one on the XL map: its heroes (archangels, Gorgons) reach most of
  its 1 296 tiles, `Reach::search` itself 71 %; the towns' entrance test
  (`AdventureState::town_visit`) is under 0.3 %. `fork-xl-144` re-set at `master`'s count (inside
  its gate; this change touches no engine code): 51 Ir more a fork since T86b2, E08's increment 10
  and E09's store of towns (an empty `Option<Arc>` there, about 9 Ir as T7b2a measured).
- **The env** (`h3-sim`'s `adventure/tests/towns.rs`; exit criterion 7):
  `the_env_plays_the_town_map_to_its_end_of_days` plays the golden replays' three seeds (173,
  1702, 3435, the agent seed the same, as `h3 adventure run`) to day 22: each chosen action legal
  by the masks and accepted, a captured town its new owner's in its observation; the union meets 4
  undefended captures, 4 sieges (2 of a visitor with the merge), 2 won and 2 lost.
  `town_map_rollouts_and_forks_are_deterministic` forks while Orrin's siege of the Tower is open.
  The main property `steps_agree_with_the_masks_and_the_game` gets a fifth kind, the town map, its
  `Chart` aimed at the towns' entrances. **Test time:** 8 s for the two town tests in a debug
  build: the map's masks checked at every step took 30 s, so they are checked before a move onto a
  town's entrance, after a step that changed a town and at every eighth step, every seat's at
  every fortieth; the battle masks at every step (*agent's call*). **A mutation:** the env's tile
  mask without the towns' entrances fails the test at the first one.
- **The bot reports** (`h3 ai eval --set report-v1`, `--set neutral-v1`, release): regenerated
  byte-identical (no part version moved).
- **The census** (`original_data`, `docs/formats/h3m.md`'s "Import"): all 160 shipped maps import,
  validate and build (T7b2b: 156; the 4 water refusals lifted by E08 T97b1), none refused by a
  town's check: exit criterion 1. New counts (`built_town_facts`): each built town's fort level,
  moat and hall level (`h3_towns::siege`, `hall_level`) and the owned towns' day's income
  (`daily_income`: 643 000 gold a day over the towns owned at the start, 7 with a silo's resource).
- **Versions:** none (no engine code changed).

### T10 — the audit; increment 1 closed; increment 2 refined (2026-11-06)

One session, the user present; branch `town/e09-t10`, docs only (the epic, its archive, log and
design note's header, `town.toml`'s statuses, the lane's HANDOFF, QUEUE and answered questions).
Starting point: `master` at 798b2e2 (#767 and #768 merged; CI green there; the setup's
`verify-once` found CI's fingerprint: the light gates only). The user's computer's shell did not
start: a sandbox cloned from GitHub, no original data needed. **At its start** (the user, asked
with a first look at the audit, `TWN-Q7`): T10 whole (decision 36) and increment 2's three
questions (decisions 37 to 39).

**The audit of increment 1's exit criteria** (`E09-archive.md`), on `master`'s code:

1. **The import.** T9c's census (`h3-map`'s `original_data`; `docs/formats/h3m.md`, "Import"):
   all 160 shipped maps import, validate and build, none refused by a town's or a start's check;
   the 128 once refused for a main town's hero build (538 main towns' heroes drawn, TWN-SET-009),
   the 3 for a hero in a town too (TWN-SET-010), the random heroes drawn (TWN-SET-008). No typed
   refusal is left, so none names a gap. **Holds.**
2. **Entered and captured, the siege on the battle's rules.** TWN-VIS-001, -002, TWN-CAP-001 to
   -003, TWN-SIE-001 to -006: `h3-game`'s `town/tests/capture.rs`
   (`an_undefended_town_of_another_player_or_none_is_captured_by_the_step`,
   `an_own_town_is_entered_without_a_capture`, `a_defended_town_is_besieged_not_captured`,
   `a_captured_capitol_is_demoted_when_its_new_owner_holds_another`) and `town/tests/siege.rs`
   (`a_siege_is_fought_on_its_towns_facts_and_native_terrain`,
   `a_step_onto_a_town_its_garrison_defends_starts_its_siege_from_the_tile_before`,
   `a_siege_a_step_started_is_played_and_captures_the_town_with_the_attacker_outside`,
   `a_step_onto_a_towns_visitor_merges_the_garrison_into_its_army_and_besieges_it`,
   `a_siege_played_to_its_end_is_settled_as_a_siege`): the siege's facts derived by
   `h3_towns::siege` into `BattleStart::town`, the battle the battle lane's siege (ADR-0023).
   Golden replays `towns-capture-s1`, `towns-siege-s1`, `towns-visitor-s1`, `towns-capitol-s1`
   and the three seeds' `towns-s*-d21`. Observed: batch TWN-1's rows 3 and 4. **Holds.**
3. **The hall pays.** TWN-INC-001, -002: `town/tests/income.rs`
   (`a_towns_income_joins_its_owners_mines_in_one_gain_per_resource`,
   `several_capitols_at_the_start_each_pay_theirs`,
   `a_neutral_town_pays_nobody_and_a_captured_one_its_new_owner_from_the_next_day`), the golden
   `towns-income-s1`, `check_income` in the plans; the census's 643 000 gold a day. **Holds.**
4. **Ids, sources, tests; the readings at risk.** `check-rules` passes (438 rules): the 26
   `TWN-*` rules each with sources and tests, the amended `ADV-`/`GAM-` rules with E09's lines.
   TD-3, TD-5, TD-7 and the siege's aftermath confirmed by batch TWN-1 (survey `E09-towns.md`
   §15); TD-1, -2, -4, -6 by the reading (decisions 13, 14), TD-8 by the code (decision 30); TD-9
   (increment 2) and TD-10 (increment 4, `TWN-STUB-6`) recorded as disputes (survey §16.9's table).
   Building 23, the batch's one surprise, read and settled (decisions 25, 27). The quirks ledger
   `quirks-town.md` holds `TWN-QUIRK-1` to `-6`, each with its regression test. **Holds.**
5. **The gates.** `h3-towns`, `h3-adventure` and `h3-game` are logic crates under the 100 %
   coverage gate with no town file in `coverage-exclusions.toml`; `h3-map`'s town modules sit in
   an adapter crate the gate does not cover, tested by its tests and the census. Golden replays
   with a town captured (above). The benchmarks: T9c's `siege-towns-36`, `day-start-towns-36`,
   `reach-towns-36`, `fork-towns-36`, `fork-xl-144` re-set inside its gate. The property plans
   play towns: `the_plans_capture_and_besiege_towns` (T9b1), `town_views_move_as_the_events_say`
   (T9b2a), the hygiene property's `vary` with towns (T9b2b). CI's full verify green on `master`
   at 798b2e2 (#768). **Holds.**
6. **The views.** `docs/06-security.md`, "What a viewer may know: a town (E09 increment 1)";
   TWN-VEW-001 to -003: `town/tests/view.rs`
   (`a_viewer_sees_each_town_with_its_public_facts_and_its_own_with_the_owners`,
   `the_garrisons_level_is_the_viewers_towns_with_a_tavern_at_most_two`,
   `a_heros_visions_creature_shows_another_teams_town_at_the_top_level_within_its_radius`,
   `only_the_viewers_own_heroes_see_by_visions`), `h3-map`'s `town_views_move_as_the_events_say`
   and `vary_changes_the_towns_the_viewer_never_sees_only`. **Holds.**
7. **The env.** `h3-sim`'s `the_env_plays_the_town_map_to_its_end_of_days`,
   `town_map_rollouts_and_forks_are_deterministic` and the main property's town map (T9c).
   **Holds.**

**The gaps fixed:** five rules cite batch TWN-1's rows and matched them in every point but stayed
`implemented`: now `verified` (TWN-SIE-001, rows 3 and 4: a visitor with the garrison merged, a
garrison alone, the attacker on the tile before, its source now naming both rows; TWN-SIE-002, row
3; TWN-CAP-003, rows 3b and 4; TWN-SET-003, row 1; TWN-SET-004, row 2). Stay `implemented`:
TWN-SIE-003 (its observation, E04c's batch 3g, shows the morale and luck bonuses only) and
TWN-VIS-001 (no row steps onto an undefended town of another player or none). The other nineteen
rest on the reading with VCMI and the wiki. **Left open, by design:** `TWN-STUB-2` (increments 2
and 4), `TWN-STUB-6` (increment 4), `TWN-STUB-8` (the battle lane's inbox), the teams'
`TWN-STUB-1` (E10's, reserved), and `ADV-STUB-13`, E09's since E08 decision 231: the random
dwellings tied to a town wait for the random towns increment 1 draws; T11 (decision 39). Noticed
for T11: `docs/formats/h3m.md`'s "Import" heading still names `MAP_IMPORT_VERSION` 18 (19 since
E08 T97b1).

**Increment 1 closed** (2026-11-06): its starting point, scope, decisions 8 to 35, task list and
exit criteria moved verbatim to `E09-archive.md`; its design note stays in `E09-design.md`
(decision 40, *agent's call*: about 85 lines of code comments in 37 files cite its sections).

**Increment 2 refined** (`TWN-Q7`; decisions 37 to 39): construction with the town events (the
build command; the requirements, costs and the day's limit; the forbidden buildings; the map's
town events' buildings and resources, their creatures with increment 3), from survey §1.3's
census (574 custom sets, 173 forbidding something; 369 town events on 110 towns: 70 build, 105
give resources, 56 add creatures). First T11 (`ADV-STUB-13`), then the survey T12 and the drafts
T13; the queue holds T11 and T12.

**Verify:** no fingerprinted path changed (`scripts/verify-once.sh engine --base origin/master`:
the light gates, check-rules, check-stubs, check-docs and check-exe-map, passed), `check-lane`
passed (the town lane's paths and one common); the engine's code is `master`'s, which CI's full
verify passed at 798b2e2. CI on the pull request is the record.

### T11 — the random dwellings tied to a town (2026-11-06)

`ADV-STUB-13` (decision 39; `cross-lane` in the adventure lane's import, build and game, consent
given at the refinement). Baseline e5a5840 (#769 to #771 merged; CI's fingerprint on `master`'s
engine code, the light gates only). The user kept the task whole at its start
(`AskUserQuestion`; recommended over a cut between the build and the import). The user's
computer's shell did not start: a sandbox cloned from GitHub, `maps.tar` and `extracted.tar`
staged with the file tool.

**Built** (the original's `random_dwellings_resolve` 0x502B60, survey `E08-dwellings.md` §3.2):
- *The import* (`h3-map`, map import v20): a random dwelling of 216 or 217 whose identifier a town
  of the map has is typed (`base:random_dwelling`) with its owner and `Holds::LinkedDwelling {
  town, levels }`: the `TownId` (the place among the towns, 77 and 98, in object order) of the
  **last** town with the identifier, as the original searches its town list (in the objects'
  order, survey `E09-towns.md` §2.1) from the end. A-35's `DwellingTown`, the levels' and the
  owner's checks as they stood.
- *The build* (`h3-adventure`): `AdventureSetup::town_factions`, one faction per town by `TownId`
  (`SetupError::TownFactions` otherwise, after the walk); in the placement walk a tied dwelling
  takes its town's faction, drawing none, then draws its level as one with a set and becomes the
  table's dwelling (`dwellings::resolve`); a town past the factions or a faction without a table
  is `SetupError::Holds`. Its tiles and names are checked against every table's outcomes, so the
  validity does not depend on the town's faction. The holds' hash tag 10, which no built state
  holds (resolved or refused): no adventure state hash bump, as E08 T93b's tag 8.
- *The game* (`h3-game`): step 2's factions (`draw_towns`) into `town_factions`; the setup's
  checks refuse a tied dwelling's town that may take a faction without a random-dwelling table
  (`GameSetupError::LinkedTown`, after `TownFaction`: a fixed town's own, a random town's any with
  a town record), so `validate` does not depend on the draw. *Changed from the plan shown to the
  user:* the plan put this as a `GameRuleset` check (a town record's faction without a table, as
  the siege record's `RulesetMismatch`); the `synthetic` pack has a town record and no tables, and
  the game's town tests' parts too, so the refusal moved to the setups that hold a tied dwelling.
- *Tests:* the adventure's resolution by the town's faction and its draws, the checks, the hash
  tag; the game's refusal; the import's last town and 217's level; an end-to-end game from the
  packs where a neutral random town's draw moves its dwelling's faction.

**Versions:** map import v20 (the 55 adventure replays' `import_version` alone moved, confirmed by
`h3-cli`'s replay test); no state, part or replay-format version moved.

**The census** (`h3-map`'s `original_data`, seed 0; diffed with `master`'s run in a worktree): all
160 maps import, validate and build; the 1 397 tied dwellings typed (216: 79 to 276 typed, 217:
126 to 1 326) and resolved, in the static layer's games (every town of the first faction with a
record: 1 603 random dwellings resolved, 3 664 dwellings, 65 Angel Portal shifts, 3 anchors one
column past the right edge, on Darwin's Prize, its Allied version and King of Pain) and in the
whole games by their town's faction (owner 1 101, alignment 43, drawn 253); no hero or boat
refused; no identifier shared by two towns (the last town's rule picks nothing on these maps); 2
more typed dwellings entered from water (11). The static layer's later draws moved with the new
level draws (the guards and the camps' spreads), as expected. `h3m.md`'s "Import" heading now
names v20 (T10's finding).

**Docs:** `h3m.md` (the import, the census), ADV-DWL-005's summary, the stub lifted
(`docs/stubs.toml`), the adventure lane's HANDOFF versions and inbox, `OVERVIEW.md`'s import line,
the lane's `TRAPS.md` (one faction per town in an `AdventureSetup`). ADR-0016 point 4's sentence
("a blocker until E09's random towns resolve it", ADV-STUB-13) is amended with increment 2's
drafts (T13, a shared path).

### T12a — the construction survey: the build command, the costs and requirements (2026-11-07)

Decision 38's survey, cut in two at its start (the user, `AskUserQuestion`, recommended over three
parts and the whole): **T12a** the build command's checks, what a build does, the cost and
requirement tables, VCMI and the wiki compared; **T12b** the town events, the census of the
shipped maps and the checkpoint. Branch `town/e09-t12a`, docs only (the survey, the code map, the
surveys' README, the lane's bookkeeping). Starting point: `master` at cf3e701 (#771 to #773 merged;
CI green there; the setup's baseline lost to a VM restart, replaced by CI's run as the sandbox
skill says). The user's computer's shell did not start: a sandbox cloned from GitHub,
`Heroes3.exe`, both LODs, `maps.tar` and `extracted.tar` staged with the file tool.

**Written:** `docs/rules/surveys/E09-construction.md` §0 to §7: the human's build through the
town screen's hall (its slots and chains, a slot's state, the Build button, the payment from the
current player's treasury), the tests (buildable now, ever buildable, the computer's buy, one
Capitol, the Shipyard's spot fixed at the start, the Grail's building), one a town a day (the
town's +0x2; a computer-only team every other day on Easy and Normal), what a build does
(`town_build_effects`), the costs (`Building.txt`'s three tables), the requirements (the nine
lists' direct requirements, tested against the every-built set), VCMI 1.7.5 and the wiki compared.
The code map gained 34 entries (the hall, the tests, the costs, the effects; three cost tables with
H3API's names confirmed); `town_boat_spot` and `grail_victory_town` are now read whole, `new_day`
and `ai_town_visit` extended, and the town's +0x2 is named `built_today`.

**Disputes:** TD-9 (the Conflux's upgraded Altar of Fire: the reading the Altar of Fire and the
upgraded Altar of Air; VCMI and the fandom wiki the Altar of Fire) and the new **TD-11** (who
builds and who pays: the reading finds no owner test on the hall's path and pays from the current
player's treasury, so a visitor might build in an ally's town at its own cost; VCMI the owner
only). No quirk candidate. Both disputes, the computer's every other day and the other open points
go to T12b's checkpoint (survey §7). Ids: disputes next 12, quirk candidates next 8 (unchanged).

### T12b — the construction survey: the town events, the census, the checkpoint (2026-11-07)

Decision 38's survey, its second part (T12 cut in two at its start); kept whole at its start (the
user, `AskUserQuestion`, recommended over a cut after the census or after the reading). Branch
`town/e09-t12b`, docs only (the survey, the code map, the open and answered questions, the epic,
the lane's bookkeeping). Starting point: `master` at d383d40 (#775, #776 merged; CI green on
708dd41; the setup's baseline lost to a VM restart while the cut was asked, replaced by CI's run
as the sandbox skill says). The user's computer's shell did not start: a sandbox cloned from
GitHub, `Heroes3.exe`, `h3maped.exe`, the four LODs, the DLLs, `maps.tar` and `extracted.tar`
staged with the file tool before the computer went offline.

**Written:** `docs/rules/surveys/E09-construction.md` §8 to §12 and §0's T12b findings: the town
events' record and the map's one list (`load_town_body`'s events), when they fire
(`town_timed_events`: at the owner's turn start after the map's timed events, the map's day test,
the town's owner the current player), what they build (`town_events_apply`: the bits through the
faction, less what stands, what the faction lacks, what the event's own buildings supersede and
the Shipyard without its spot; no requirement, forbidden, Capitol, Grail, cost or mark test; with
the visit) and add (the creatures to the level's upgraded dwelling, else its dwelling, else lost);
VCMI 1.7.5 (`handleTownEvents`, `buildStructure` with `force`, `occursToday`) and the wiki's copy
compared; the census of the 160 maps (a throwaway example over `h3_map::read`, not committed: 173
custom sets forbid something, 83 fixed towns where a forbidden requirement blocks 225 buildings;
369 events, 70 build, 27 of their buildings without a requirement in three maps, six Capitols, 56
add creatures, 105 give resources, 199 a message only, correcting `E09-towns.md` §1.3's 189; 98
repeat, 18 every day). The code map: `town_timed_events` and `town_events_apply` read whole, two
message functions named (0x5C0220 read in part, 0x5C0400 inferred), `load_town_body` and
`player_turn_start` extended; `structs.toml`'s `town_event` (H3API's H3CastleEvent confirmed) and
the game's `date` and `town_events`.

**Disputes:** TD-12 new (an every-day event: VCMI one day early); quirk candidate T-8 new (an
event's second Capitol). **Checkpoint** (`TWN-Q8`, one round of four, each as recommended;
decisions 41–44, survey §12): TD-9 the executable's list and TD-11 the owner, each with a row of
batch TWN-2; one a town a day for every player; the town events as the original plays them.
Ids: disputes next 13, quirk candidates next 9, questions next 9, decisions next 45.

### T13 — increment 2's drafts, the design note's Part D and the cut (2026-11-07)

Decision 38's second task; kept whole at its start (the user, `AskUserQuestion`, over T13a the
drafts then T13b the note with the checkpoint, recommended as increment 1's T2a and T2b, and over
the note first). Branch `town/e09-t13`, docs only (the drafts, the design note, the epic, the
answered questions, the lane's and the adventure lane's queues, the HANDOFF). Starting point:
`master` at 98b2c14 (#779 merged; CI green there; the setup's baseline lost to a VM restart,
replaced by CI's run as the sandbox skill says). The user's computer's shell did not start: a
sandbox cloned from GitHub, `Heroes3.exe`, `h3maped.exe`, the four LODs, the two `.snd`, the
DLLs, `H3ab_ahd.vid`, `battle-videos.vid`, `maps.tar` and `extracted.tar` staged with the file
tool at the start.

**The ADR-level choices** (`TWN-Q9`, one round of three before the drafts; decisions 45–47): the
build a town command `GameCommand::Town(TownCommand::Build)` (recommended); the Shipyard's water
now (recommended; `TWN-STUB-2`'s Shipyard half from increment 4); the views as the original shows
them (the user's own answer; read for it: the owner's town screen from the town list, an ally's at
its visit, the popup's portrait showing a town built in today at every level, no buildings in the
popup).

**Written:**
- **The drafts** `docs/lanes/town/adr-drafts/0028-increment-2.md` (the record's day's mark, the
  rules: buildable now with the hall's offer, the build, an event's buildings, the mark's life;
  the part's requirements, costs and the role `shipyard`; `TownCommand`; part v4, state hash v2),
  `0016-increment-2.md` (the town's boat spot by the kind `Town`'s offsets; point 4's sentence on
  the tied random dwellings, T11's; the views of the buildings and the public day's mark),
  `0017-increment-2.md` (the command's checks and payment, the observation, the day's start
  clearing the marks, the town events after the map's timed events, the draw-free Shipyard forbid
  at the build, no decision kind, the inventory, the env's build entry, the versions, the capture's
  mark). ADR-0015 and ADR-0023 are not amended.
- **The design note** `docs/epics/E09-design.md` Part D, §23 to §36: scope (§23), the part (§24),
  the harvest (§25), the record and the store (§26), the rules (§27), the Shipyard's water (§28),
  the command (§29), the day and the turn with the town events (§30), information (§31), the env and
  the gates (§32), batch TWN-2's two rows (§33), versions, rule ids (TWN-BLD-001 to -007,
  TWN-SET-011, TWN-EVT-001 to -004, TWN-VEW-004), the quirk T-9 (new: the event loop's bits 41 to
  43 past the table), the stubs `TWN-STUB-10` to `-12` planned (§34), the cut and the exit criteria
  (§35), risks (§36).
- **Read for it, not new:** `Building.txt`'s horde rows (each horde and its upgrade cost the same
  in every faction: the hall's offer changes no price); the original's town popup at every level
  shows the day's mark (survey `E09-towns.md` §12.3).

**The checkpoint** (`TWN-Q10`, one round of four after a summary was sent; decisions 48–51, each
as recommended): the drafts accepted as written (T15 applies them after batch TWN-2); the cut as
proposed (T14 to T20 in rounds 2 to 8); the eight exit criteria; the agent's calls kept, T-9 fixed
without a flag. **The queue:** T13 out, T14 (batch TWN-2) written; a note in the adventure lane's
inbox on the planned `cross-lane` bumps and their order with E08's increment 11. No version moved.
Ids: questions next 11, decisions next 52, quirk candidates next 10 (T-9), disputes next 13.

### T14a, T14b — batch TWN-2's spec, the harness's hall path and two runs (2026-11-07)

One session, the user present at the start; branch `town/e09-t14`, `cross-lane` (the user's
consent at the start: `docs/experiments/README.md` is the battle lane's; `h3-experiment` needed no
change). Starting point: `master` at 2efcb26 (#782 merged; CI green on efaae07, the same code; the
setup's baseline lost to a VM restart). T14 cut in three at its start (the user, asked;
recommended): T14a and T14b in this session, the scoring T14c next. The user's computer's shell
did not start: a sandbox cloned from GitHub, the original data staged with the file tool at the
start.

- **T14a, the spec** `experiments/specs/E09-batch-2a.toml` (stamp `1128298c`): TWN-2-1, Red's
  Conflux C1 (7, 7) with the Fort, the Mage Guild, the level-1, -2 and -4 dwellings (the closure
  of the Altar of Fire 33, without the upgraded Altar of Air 38), and the control C2 (17, 7) with
  38 too, each with a Red hero on the tile south of its entrance; TWN-2-2, Blue's Castle B1
  (27, 7) with a Fort only and Red's hero south of it, Red and Blue one team (`teams`), both
  human; the predictions of the reading, VCMI and the wiki. Two changes found by T14b's probe: a
  hero of Blue's far off (a player without a hero is offered no Artifact as its starting bonus,
  which moved the harness's choice of Gold off its place) and Tan's Castle T1 (7, 30), a computer
  player alone on its team (without an enemy the game declared Red's victory at the first visit's
  end). The batch's row in `docs/experiments/README.md`.
- **T14b, the harness** (`clients/capture/`, common; README "The hall and the build"), learned
  live under Wine with a throwaway probe: the town screen's buildings are its items (id the
  building id); a click on the shown hall building (10 to 13) opens the hall (vtable 0x6437A0),
  the panel's hall picture does not; the hall's slot k is item 700 + k (its frame the building),
  its bar 400 + k (frame 0 built, 1 buildable, 2 not now, 3 never); a slot's click opens the
  building's dialog (0x643944): Build 30722 (state 0x6 when it builds; 0x8 or 0x20 set, greyed),
  Cancel 30721, the requirements' text item 5. New: a walk's `actions` on the town screen, the
  actions `hall` and `slot`, the reads `town:x,y:mark`, `hall:slot:<id>`, `build:button`,
  `build:text` (the record and the building ids it names, `h3lod.building_names`), `screen`,
  `town_screen_closed`; `h3mem`'s town record gains the day's mark. `docs/exe/map.toml` gains
  the two vtables. The recipe `clients/capture/batches/E09-batch-2a.toml` (5 steps). The harness
  check ran after the change; the client, map-client and adventure lanes' inboxes told.
- **The runs** (Easy, hot seat, the starting bonus Gold, under Wine; about 4 minutes each): two,
  recorded (`experiments/results/E09-batch-2a.md`, `-run2.md`), the same but for the random
  bonus's gold.

**What the runs showed** (for T14c; not scored here): **TWN-2-1** C1's level-4 slot the upgraded
Altar of Fire, its look not now; its dialog's Build greyed (state 0xE), the text GENRLTXT:53
naming 38; Build pressed: nothing (the dialog stays; the sets, the mark 0 and Red's resources
unchanged). C2's slot buildable, Build enabled; pressed: the town screen at once, the standing set
40 in and 33 out, every-built 40 added, the mark 1, Red's 5 mercury, 5 ore and 2 000 gold less.
**TWN-2-2** Red's hero in Blue's town: the town screen, the hall, the Tavern's slot buildable;
its dialog's Build greyed (state 0x26), the text GENRLTXT:220 (every requirement met); pressed:
nothing; both treasuries, B1's sets and its mark unchanged. **Not predicted:** a hero's visit sets
a bit of the town's standing set that no building of its faction has (22 in each Conflux, 23 in
the Castle) as the town screen opens; it stays after the screen closes, and C2's build copied it
into the every-built set (recomputed from the standing set).

### T14c — batch TWN-2 scored (2026-11-07)

One session, the user present at the start; branch `town/e09-t14c`, `cross-lane` (the user's
consent of T14: `docs/experiments/README.md`'s row is the battle lane's). Starting point: `master`
at 74c42c4 (#783 merged; T14a's and T14b's). Kept whole at its start (the user, asked; the cut in
two, the verdicts and the stray bit, recommended). The user's computer's shell did not start: a
sandbox cloned from GitHub, the original data staged with the file tool at the start. Docs only;
code reading with `scripts/exe.py`, no probe under Wine needed.

- **The scoring** (survey `E09-construction.md` §13; §0, §1.1, §1.4 and the sources amended):
  both records the same but the starting bonus's gold. **TWN-2-1 (TD-9) as the reading**: C1's
  upgraded Altar of Fire not now, Build greyed (0xE), the text naming the upgraded Altar of Air
  alone, nothing on Build; C2 built it (40 in, 33 out of the standing set, 40 into the every-built
  set, the mark 1, 5 mercury, 5 ore, 2 000 gold); VCMI's and the wiki's requirements refuted.
  Decision 41 stands. **TWN-2-2 (TD-11) refused**, against the reading's prediction: the hall
  opened for the ally's visitor and showed the Tavern buildable, Build greyed (0x26) with no
  requirement missing; nothing paid by either player. The re-read: the hall's own test passes
  here; the dialog's constructor `build_dialog_create` 0x5D55C0 (not read by T12a) greys Build
  when the town's owner is not the current player (`build_dialog_owner_test` 0x5D5AE2; in a
  network game also for a current player without the human flags); the two greyings show as the
  button's 0x8 (the hall's) and 0x20 (the owner's). Decision 42 stands, now as the original's
  rule; no question.
- **The stray bit** (not predicted): `town_screen_pieces` 0x5C33F0 (read whole; the map's
  `town_build_cheat` renamed), called by the town screen's init and where the screen rebuilds its
  buildings, builds the faction's display piece into the standing set (`town_build`): Castle 23,
  Stronghold 27, Fortress 23, Conflux 22; with the build-all cheat's flag 0x6AAA5C every building
  instead. The every-built set takes it at the town's next build (`town_build_effects` recomputes
  it from the standing set). None of the four is in its faction's list (dumped again); every
  reader of 22, 23 and 27 tests the faction first but the Castle Gate's town list (filled with
  Infernos) and the computer's pathfinder's (`ai_pathfinder_gate_bit` 0x56A947). Display pieces, as the hall companions: **nothing for the
  engine**, no `T-10` (the next quirk candidate stays T-10). A trap for comparing the original's
  sets: the lane's `TRAPS.md`, "Experiments and the original's records".
- **Applied:** design §33 (the verdicts), §29 step 2, §36 (TD-11 settled); the drafts' status
  lines and TD-9's, TD-11's (`0016`, `0017`, `0028`; accepted as written, T15 applies them); the
  experiments README's row (evaluated); `docs/exe/map.toml`: `build_dialog_create`,
  `build_dialog_owner_test`, `town_screen_init` (inferred), `town_screen_pieces`,
  `town_castle_gate` (inferred), `build_all_cheat_flag`, `build_all_cheat_toggle`,
  `ai_pathfinder_gate_bit` (inferred). T15 queued.
- **Agent's calls (change freely):** the names above; the trap's wording; the pieces left out of
  the part with no rule id (as decision 27's companions).

### T15 — `shared/`: increment 2's drafts into `docs/adr/` (2026-11-07)

One session, the user present at the start; branch `claude/zealous-euler-msupe1` (the session's
given branch; `shared/e09-t15` by the convention), `cross-lane` (the town lane's docs ride along,
as T4a's). Starting point: `master` at c22c25c (#786 merged). Kept whole at its start (the user,
asked; recommended). The user's computer was not connected (no folder, no file tool): not
needed, docs only. No code, no version.

- **ADR-0028** points 2 (the day's mark; buildable now with the hall's offer, the build, an
  event's buildings, the mark's life; the boat spot the adventure's), 5 (the requirements, the
  costs, the role `shipyard`; TD-9 as TWN-2 observed it), 6 (`TownCommand`), 9 (part v4, state
  hash v2); its header's amendments and detail, consequences, alternatives, revision history.
- **ADR-0016** points 1 (the town's boat spot, in its E09 item), 4 (the random dwelling tied to a
  town typed, `ADV-STUB-13`'s sentence replaced; the kind `Town` gains no visit rule), 6 (a
  town's buildings its owner's and an ally visitor's, built-today public); consequences,
  alternatives, revision history.
- **ADR-0017** points 1 (the town command, its owner's: TD-11 with TWN-2's verdict), 2, 3 (the
  day's start's marks, the town events at the turn start), 4 (no draw; the Shipyard without
  water), 6, 7 (the inventory), 8 (the env's build entry), 9, 10 (a build's and a town event's
  outcome, the capture's mark); consequences, alternatives, revision history. Each text after
  its point's E09 increment 1 text, or at the point's end where the point has none (6, 7) or the
  draft named a later one (8, 10).
- **Also:** the ADR index (the three rows' status and Read-when cells); `docs/03-domain.md`'s
  town terms (build, hall's offer, day's build, requirements and cost, boat spot, town event);
  `docs/06-security.md`'s section "A town's construction (E09 increment 2)" (design §31's words)
  and the trust boundaries' example (`TownCommand::Build`); the design note's Part D header and
  §28 (the ADRs, not the drafts); the adventure lane's inbox note of T13 (the pointer). The drafts
  deleted. Each draft sentence checked against its target by a throwaway script (every one found
  but the headings and the reworded decision references). T16a queued.

### T16a — the town part's requirements, costs and the role `shipyard` (2026-11-07)

One session, the user present at the start; branch `town/e09-t16a`, `cross-lane` (the adventure
replays' header line and one test literal in `h3-game`'s `src/tests/mod.rs`, the adventure
lane's; as T8c2's). Starting point: `master` at 02b01bd (#788, T15's, merged; CI green; the
setup's baseline found CI's fingerprint, the light gates only). Kept whole at its start (the
agent's call under the user's standing instruction: a cut only for a big task; none was needed).
The user's computer was connected: the original data staged with the file tool at the start
(not needed by this task).

- **`h3-towns`** (design §24 as built): `BuildingDef::requires` (direct keys) and `cost` (core
  resource keys and amounts), `BuildingRoles::shipyard`; the compiled `Building::requires` (the
  closure by the same `close` as the superseded sets, never itself) and `Building::cost` (the
  core's resource order, through the silo's check shared as `Scope::amounts`). Refusals, in the
  existing variants: `InvalidBuilding` "it requires a building the faction lacks", "it requires
  itself", "it requires itself through others (a cycle)", "a cost names each resource once, at
  least 1"; a cost's `UnknownResource` and `ReferenceOutsideDependencies`; `InvalidTown` "has
  more than one Shipyard". `TOWN_RULESET_HASH_VERSION` 4: after `auto_with` the required set's
  bits and the cost, `shipyard` after `grail` among the roles; pinned (0xcb0d02cf9d6eb150, the
  empty part 0x985b2cc3d2245173). The tests' record has requirements on the halls, forts,
  dwellings and hordes, costs on three buildings (one given out of the core's order) and a
  Shipyard; every refusal tested.
- **`h3-pack-towns`:** the format's `requires`, `cost` (a table, as the silo) and `shipyard`
  (the format doc and the roles' table); the synthetic pack's requirements and costs on every
  building but the Village Hall and the Grail, and a Shipyard; tests of the closures, the costs'
  order, a cost's key of another pack, the strict parser's new fields, and the shipped packs
  without requirements, costs or Shipyard (until T16b).
- **`cross-lane`:** the 55 adventure replays re-hashed from their own commands (skill
  `homm3-rerecord-replays`): only `town_ruleset_hash` moved (0x3003da8dd5ff664f to
  0x929e9a26ed0bdf34); the bot reports record no town part hash. The four `BuildingDef` literals
  of `h3-game`'s tests gained the two fields (one in the adventure lane's `src/tests/mod.rs`).
  The adventure lane's inbox told.
- **Agent's calls (change freely):** `shipyard` after `grail` in the struct and the hash; the
  refusals' wording and the self-requirement told apart from a cycle; no `// covers:` line (the
  rule ids TWN-BLD-006 and -007 come with the rules, T17, and the content's sources, T16b).

### T16b — the harvest of the requirements, the costs and the role `shipyard` (2026-11-07)

One session, the user present at the start; branch `town/e09-t16b`, `cross-lane` (`h3-harvest`,
the battle lane's crate, and the adventure replays' header line, the adventure lane's).
Starting point: `master` at a65f303 (#791, T16a's, and #792, #793 merged; the setup's baseline at
f28200c found CI's fingerprint, the light gates only). Kept whole at its start (the agent's call
under the user's standing instruction: a cut only for a big task). The user's computer was
connected: the original data staged with the file tool at the start (the four LODs, the
executables, the five DLLs, both `.snd`, the videos, both tars, the 40 MP3s), then `cp -rn` into
the clone; the harvest needed `Heroes3.exe` and `H3bitmap.lod`.

- **`h3-harvest`** (design §25 as built): `towns::Tables::costs` (`Building.txt` of
  `H3bitmap.lod`, read by `Inputs::read`); `cost_row` picks a building's row as
  `building_costs_load` 0x5C14C0 fills its tables (checked against its code: rows 4 + 11 f, 104,
  125 + 16 f; the ids 26 to 29 none); each building's `requires` (the list's direct ids as keys,
  in its order), `cost` (the zero columns dropped; base's resources prefixed in the AB file) and,
  for the building 6, the role `shipyard` (after `grail` in the roles' order; refused from the
  gap-fill as `dwelling`). New refusals: a requirement outside the record, a listed building's
  blank cost row, a cost cell that is not a number. The report's line is the hordes' growth
  alone; the files' header names survey `E09-construction.md` §5 and §6.
- **Tests:** the fixture's `Building.txt` in the original's layout (each neutral, special and
  dwelling row told apart by its gold), a few requirements and Alpha's Shipyard; the outputs of
  the Village Hall, a guild, the City Hall, a dwelling, a horde's upgrade, Beta's special row,
  Gamma's dwelling and Town Hall (prefixed), the Grail (no cost) and the Shipyard; every refusal.
  `h3-pack-towns`' test of the original packs: T16a's "none" assertion replaced by
  `the_original_requirements_costs_and_shipyards` (every closure inside its record and never
  itself, the Capitol's holding the Town Hall and as §6 gives it in every faction, the upgraded
  level-7 dwelling's closure of every faction from §6's dump, TD-9's two; §5's neutral costs, the
  Castle's hordes, Rampart's second horde, the Conflux's four upgraded altars, the Tower's fifth
  guild; the Grail alone free; the Shipyards of Castle, Necropolis, Fortress and Conflux). A
  throwaway script also matched every generated `requires` (315 buildings) with §6's dump.
- **Packs:** `packs/base` (231 requirements, 285 costs, three Shipyards) and `packs/ab` (28, 35,
  one) re-harvested; `packs/sod` only the header's sources line; `--check` passes.
- **`cross-lane`:** the 55 adventure replays re-recorded from their own commands (skill
  `homm3-rerecord-replays`, a scratch script, 2 s in release): only `town_ruleset_hash` moved
  (0x929e9a26ed0bdf34 to 0x8477f2008d862e55). No version moved. The adventure lane's inbox note
  of T16a amended.
- **Agent's calls (change freely):** the checks in the harvester and not a gap-fill; the Grail
  building without a cost (the original asks none); the rule ids TWN-BLD-006 and -007 with the
  rules (T17), their sources this harvest.

### T17a — the day's mark, what a build does and the store's writes (2026-11-09)

One session, the user present at the start; branch `town/e09-t17a`, `cross-lane` (the game state
hash's version in `h3-game`'s `lib.rs` and the adventure replays, the adventure lane's).
Starting point: `master` at 2e5e08f (#794, T16b's, and #795 merged; CI green; the setup's
baseline found CI's fingerprint, the light gates only). **T17 cut in two at its start** (the
user, `AskUserQuestion`; recommended, the last session's proposal with `built_with` moved in,
since the store's build needs it): **T17a** what a build does to a town (the mark, `built_with`,
the store's three writes, the town state hash v2, the game state hash v26), **T17b** the rules
that decide a build (`buildable`, `possible`, `check_build`, `event_buildings`), `TownCommand`
and its errors, TWN-BLD-001 to -003, -006, -007, TWN-EVT-003 and `TWN-QUIRK-7`; T17a this
session, T17b queued. The user's computer was connected: the original data staged with the file
tool at the start (the four LODs, the executables, the five DLLs, both `.snd`, the videos, both
tars, the 40 MP3s), then `cp -rn` into the clone and both tars unpacked; the scoped verify ran
with them.

- **`h3-towns`** (design §26 and §27's `built_with` as built): `Town::built_today` (a flag,
  false from `Town::new`; `Town::rebuilt`, crate-private, for the store's writes);
  `built_with(town, part, building) -> Option<BuildingSet>` (new module `build.rs`, which T17b
  fills): the building with what it supersedes (setup's `closed`), then the setup's
  `upgrade_hordes` with no forbidden set, as `town_build` 0x5BE930 tests none (a town event's
  forbidden horde still upgrades); `None` for a faction without a record or a building the record
  lacks. `TownFacts { fort, capitol, built_today }` and `Towns::facts`; `Towns::build(town,
  building, part, mark) -> Option<TownFacts>` (the facts before; the mark set with `mark`, else
  kept: an event's build neither sets nor clears it), `Towns::start_day()` (writes only the
  marked towns; a store without one copies nothing), `Towns::captured(town, clear_mark) -> bool`
  (whether the mark was cleared); the old `set_built` became `write(town, change)`, which
  `apply_capitol_limit` uses too (the mark kept). The hash v2: each town's mark after its forbidden
  set (pinned 0x1F7C_E05E_64B8_2246).
- **Tests** on the synthetic part: `built_with`'s closure (a Capitol over a custom City Hall, the
  Brotherhood's Tavern), its fixed point both ways (the horde first or the upgraded dwelling first,
  the fixture's shrine a second step; no forbidden test), its `None`s; the store's build copying
  one town of a fork and no list twice, an event's build keeping either mark and building a
  forbidden Fort, the `None`s copying nothing, the day's start writing only the marked towns, the
  capture's two cases (the Capitol's limit keeps the mark), increment 1's reads at once (a Town
  Hall's income, a Citadel's moat, a Tavern's view, a silo's resources, the Capitol public), the
  hash's mark.
- **Catalog:** TWN-BLD-004 (the mark's writes; the refusal comes with T17b, the game's calls with
  T18b) and TWN-BLD-005 (what a build does), both `implemented`.
- **`cross-lane`:** `GAME_STATE_HASH_VERSION` 25 to 26 (the store's v2 folded in; E08 T105b's
  planned v26 becomes v27: the adventure lane's HANDOFF versions line and an inbox note); the 55
  adventure replays re-recorded from their own commands (skill `homm3-rerecord-replays`, a scratch
  script, 3 s in release): only `game_state_hash_version`, the checkpoints and the final hashes
  moved, in every replay (the empty store's hash moves with its version too). The bot reports
  record the part hashes only: untouched.
- **Agent's calls (change freely):** `built_with` in `h3-towns`' new `build.rs` beside setup's
  helpers made crate-public (not moved); `TownFacts` in the store; `captured` returning whether it
  wrote, as `apply_capitol_limit`.

### T17b — the rules that decide a build, `TownCommand` and its errors (2026-11-09)

One session, the user present at the start; branch `town/e09-t17b`, not `cross-lane` (only
`h3-towns`, `h3-pack-towns`' test marker and the lane's docs). Starting point: `master` at 75299e8
(#796, T17a's, and #798 merged; the setup's baseline found CI's fingerprint, the light gates
only). Kept whole (the agent's call: one crate, no version). The user's computer was connected:
the original data staged with the file tool at the start, then `cp -rn` into the clone and both
tars unpacked; the scoped verify ran with them, and `scripts/exe.py` read the event loop again.

- **`h3-towns`** (design §27 as built): in `build.rs` beside `built_with`, `buildable`,
  `possible`, `check_build` (the cost as the part's slice or the first refusal; one private
  `refusal` decides both it and `buildable`, so the offer is exactly what it allows) and
  `event_buildings`; the private `offered_as` (a horde's upgrade: superseding it, automatic with
  it, the rest of its `auto_with` ever built, itself not; T17b's refinement of design §27's call,
  the supersedes test); new `command.rs`: `TownCommand::Build { town, building }` and
  `TownCommandError` (both `#[non_exhaustive]`; `UnknownTown`, `NotOwner` for the game, then the
  eight refusals in their order; `Display`, `Error`); setup's `indexed` made crate-public. No
  version moved (the part and the store unchanged).
- **The fixture** (test-only): the Keep's requirements (the closures by hand: the halls through
  the Tavern and the Castle, the forts, the dwellings from the Fort, the hordes from their
  dwellings, the upgraded horde from the upgraded dwelling alone), its costs (all but the Village
  Hall and the Grail) and a Shipyard (23, slot 6, no requirement); `part_with_keep` changes the
  Keep's record. Every earlier test passed unchanged.
- **Tests:** the offer at the start and after the Fort and the Tavern (a Castle keeps its Fort,
  the forbidden out, the Marsh's free buildings, no record), a horde offered as its upgraded horde
  (the same outcome either way, the shrine no upgrade, a forbidden upgrade leaving neither, a custom
  set's upgrade without its horde), the Capitol's limit, the Grail never by the hall (possible as
  the original's), the day's mark, every refusal in its order (the horde made a Grail and a Capitol,
  one cause taken away a step), the offer equal to `check_build`'s Ok and inside `possible` over
  six towns, `possible`'s never (a forbidden Fort), the event's buildings with and without water
  (no requirement, forbidden, Capitol, Grail or mark test; built in order as builds), TWN-QUIRK-7.
- **Catalog:** TWN-BLD-001 (the command; the game's half with T18b) and -003, TWN-EVT-003
  `implemented`; TWN-BLD-002, -006, -007 `verified` (batch TWN-2's rows; -006 and -007's markers in
  the compiler, covered by its test and `h3-pack-towns`' test of the original packs); TWN-BLD-004's
  summary with the refusal. **Quirk:** `TWN-QUIRK-7` (T-9, fixed without a flag), its loop read
  again (0x5BFEF9 to 0x5BFF4C: 44 bits of `building_bits`, `map_building_ids` at faction × 41 +
  bit).
- **Agent's calls (change freely):** the supersedes test of the hall's offer; the errors without a
  building but `OfferedAs`'s; `possible` holding the Grail's building as the original's
  `town_build_possible` does; the rules in `build.rs`, the command in `command.rs`.


### T18a — the Shipyard's water (2026-11-09)

One session, the user present at the start; branch `town/e09-t18a`, `cross-lane` (the adventure
part, the static map, `h3-game`'s `state.rs`, the replays, the bot reports and `h3-map`'s census
are the adventure and AI lanes'). Starting point: `master` at ae7d260 (#801, #802, #804 merged;
CI's `CI` green on it; the setup's baseline was lost to a restart of the sandbox, so CI's run
stands for it). Kept whole (the user, at its start). The user's computer was connected: the
original data staged with the file tool at the start, then `cp -rn` into the clone and both tars
unpacked; the census and the scoped verify ran with them, and `scripts/exe.py` read the setup's
order.

- **The reading** (design §28's open point, settled): `new_game_heroes` 0x4BFE70 calls, in one
  straight line, the map's read (`new_game_bans` at 0x4BFFB9, whose `map_objects_place_all`
  resolves the random dwellings at 0x4FD90B and fixes the Shipyards' boat tiles at 0x4FD935),
  `resolve_random_objects`, `setup_map_objects`, `gates_pair`, then `setup_towns` at 0x4C0013,
  whose `town_buildings_setup` calls `town_boat_spot` 0x5C0C90 (read by T12a: water, not blocked,
  no object or a boat). So a town's spot sees the resolved dwellings: the agent's call "after"
  kept, the spot fixed in ADV-SHY-001's step.
- **The adventure** (design §28 as built): `ObjectKind::Town { boat_offsets }` and its def
  (distinct, never (0, 0), possibly none: no spot; hashed as a Shipyard's offsets), adventure
  ruleset part **v19**; the pack's `town = { boat_offsets = [[-1, 2], [1, 2]] }` in the base pack
  (optional, `town = {}` none); `fix_boat_tiles` takes a town's offsets with the Shipyard's test
  (`Resolved::BoatTile` on the town object, in the static map's hash);
  `StaticMap::town_boat_spot(TownId)`; `ObjectView::boat_tile` stays the Shipyard's (a town's spot
  is the town's Shipyard's, its view T18b's, §31). Every match on the kind updated (`h3-adventure`,
  `h3-game`, `h3-map`'s play tests and census, `h3-cli`'s ai census).
- **The town and the game:** `h3_towns::without_water` (the record's buildings of role `shipyard`
  forbidden and out of the ever-built set, the rest and the day's mark kept; `Town::reset`);
  `h3-game`'s `town::without_water` by `TownId` after `AdventureState::new` and before the
  reveals and the store (one line in `state.rs`); no draw. `TWN-STUB-2` narrowed to the Grail
  building. The game's town fixture gained a Shipyard (`b12`, slot 6; the siege tests' defender
  renamed `defender`), its adventure part the original's two offsets.
- **Tests:** the part's offsets (refused twice, at the entrance; hashed by value, order and
  count), the static map's spots (first offset, land, a blocker, a visited tile, owned and
  neutral, the owner's view without it, the hash), `without_water` (a custom set's Shipyard out, a
  standard set's forbidden, a requiring building kept, the mark kept, a faction without a Shipyard
  or a record unchanged), the game's start (no water, either offset, water elsewhere); the two
  tests comparing the store with `town_setup` now apply `without_water` (their maps have no
  water).
- **The census** (the original data, `h3-map`'s `original_data`, checked): 1 468 towns, 67 with a
  spot; 625 of a faction with a Shipyard, 591 of them without a spot (their Shipyard forbidden);
  the 9 custom sets that build a Shipyard (survey §5.3's 9) all have a spot, none loses it.
- **Versions:** adventure ruleset part v19 (after T103a's v18); no other. The 55 adventure
  replays re-recorded from their commands: only hash lines moved (the part's hash in each; the
  checkpoints and final hashes of the eight `towns-*` replays, whose towns without water now
  forbid their Shipyard). The bot reports' `engine` line only.
- **Catalog:** TWN-SET-011 `implemented` (the reading, VCMI's `initTowns`, the design);
  TWN-SET-003's "not yet" narrowed; ADV-SHY-001 amended (the towns' spots in the same step).
- **Agent's calls (change freely):** the offsets optional in the pack (none: no spot); the town's
  spot out of `ObjectView::boat_tile`; `without_water` a transform of the built town rather than
  an input of `town_buildings_setup` (the spots come after the towns' draws); the census in
  `built_town_water`.

### T18b1 — the build command, the day's start, the capture's mark (2026-11-09)

One session, the user present at the start; branch `town/e09-t18b1`, `cross-lane` (`h3-game`'s
`command.rs`, `lib.rs`, `observe.rs`, `legal.rs`, `state.rs`, `state/day.rs`, `h3-sim`'s replay
and action, `h3-files`' replay words (moved there from `h3-cli` by #808, merged in), `h3-cli`'s
replay tests, the replays and `game.toml` are the adventure and AI lanes').
Starting point: `master` at 31e0ea3 (#806, #807 merged; #808 merged in before the pull request; the setup's baseline lost to a restart of
the sandbox, CI's run on `master` stands for it). **T18b cut in two at its start** (the user):
T18b1 this session; T18b2 queued (the legal builds and `town_offer`, the views and events of §31,
`h3-protocol`'s mapping, SRV-Q26's place decided at its start). The seam: as soon as
`legal_commands` lists builds, the adventure property plans build (`Step::Legal` picks any listed
command), so the told-views property needs §31's events with them; the command itself goes
first. The user's computer was connected: the original data staged at the start (the scoped
verify).

- **The command** (design §29 as built): `GameCommand::Town(TownCommand)`, run as an adventure
  command (`check_ready`), then the town lane's new `town/command.rs`: the store (`UnknownTown`),
  the owner by the adventure's owner of the town's object (`NotOwner`), `holds_capitol` (the
  capture's test moved to a function the build shares), `h3_towns::check_build`, the treasury
  (`CannotPay`), then `pay_price` (`ResourcePaid` per resource, as `BuildBoat`'s: the design's
  `ResourceLost` was a slip), `Towns::build(…, mark: true)` and `GameEvent::TownBuilt { player,
  town, building, event: None }` (the owner's; `event` T18c's). `GameCommandError::Town` wraps the
  town's refusals; `GameState::towns_mut` (crate-private) the store's write. `legal_commands` lists
  no build yet (its doc says so; T18b2).
- **`TownCommand` and `TownCommandError` exhaustive** (`#[non_exhaustive]` dropped, the only two
  in the workspace): the game's match needs no wildcard arm coverage cannot reach, and the wire's
  mapping is exhaustive (SRV-Q25). Re-exported by `h3-game` with `BuildingId`.
- **The day's start:** `Towns::start_day()` in `advance_day` right before the income (GAM-CAL-002
  amended; the original's towns' loop 0x4C8067 before its income).
- **The capture's mark** (decision 43): `town::capture` reads the old owner before the adventure's
  capture and calls `Towns::captured(town, clear)`, `clear` when the old owner is none or not
  human and the new owner human (a team its player alone until E10), before the Capitol's limit;
  the game passes its players' `human` flags.
- **The replays:** adventure replay format **v11** (`town build <town> <building>`; T105b's
  bump becomes v12); the 55 adventure replays re-recorded from their commands, only the `format`
  line moved in each. The env's entry for a build is none until T19 (`h3-sim`'s `tileless_entry`
  arm). The bot reports did not move (no part or behaviour change: no build is listed).
- **Tests** (`h3-game` `town/tests/command.rs`, `capture.rs`): a build paid and told to the owner
  alone, a free one paying nothing; the mark refusing a second build, the day's start clearing
  every town's, a requirement met the next day; every refusal (unknown town, not the owner,
  built, unknown building, requirements, the treasury, another player's turn, the system actor)
  changing no hash and telling nothing; the Capitol refused while another is held, built while
  the other Capitol is another player's; the capture's mark in five owner and human cases. The
  replay words round trip and nine malformed `town` lines are refused (`h3-files`).
- **Catalog:** TWN-BLD-001's and TWN-BLD-004's game halves, TWN-CAP-001 and GAM-CAL-002 amended.
- **Agent's calls (change freely):** the capture's `human` a closure over the game's players; the
  build's facts before (`Towns::build`'s return) kept for T18b2's `TownChanged`; `TownBuilt`'s
  `event` field now, so T18c moves no event shape.

### T18b2a — the views and events, every viewer's changed view told (2026-11-10)

One session, the user present at the start; branch `town/e09-t18b2a`, `cross-lane` (`h3-game`'s
`command.rs`, `lib.rs`, `observe.rs`, `state.rs`, `state/battle.rs`, `state/day.rs`, `h3-map`'s
property tests `play.rs` and `view.rs` are the adventure lane's; `h3-protocol`'s `map::game`,
`wire::game` and their tests the server lane's). Starting point: `master` at d1b90fb (#809, #810
merged; the setup's full engine baseline green, CI's run on `master` green). **T18b2 cut in two at
its start** (the user, decision 52): T18b2a this session, T18b2b (the legal builds) queued.
SRV-Q26's town gap decided at the start (decisions 53, 54). The user's computer was connected
only at the start: the original data staged then.

- **The views** (design §31 as built): `TownView::built_today` (public), `TownView::buildings`
  (`TownBuildingsView { built, forbidden }`, the owner's alone; `BuildingSet` re-exported by
  `h3-game`). One per-town view, `Seer::view` (the viewer's Taverns and Visions read once, when a
  view needs them), serves `town_views` (`observe`) and the told changes alike.
- **The events** (decision 53): `TownChanged` per viewer (`player`, `town`, `owner`, `fort`,
  `capitol`, `built_today`, `level`, `slots`, `hall`, `buildings`), then `TownStackSeen` per shown
  stack (with its slot for the owner's team, so the garrison by slot folds) and `TownIncomeSeen`
  per income line; `project` gives each to its `player`. `town::view::Told::before` takes every
  viewer's views of the touched towns (and the views of every town by the players whose Taverns
  the change may count), `Told::tell` tells the changed ones, both nothing for a sink that keeps
  nothing (`Sink::RECORDS`). Hooked at the build (`town/command.rs`), the capture (`town/capture.rs`,
  now taking the game state; the old `TownChanged { capitol, seen_by }` gone), the siege's merge
  (`settle_outcome`), the garrison's battle (`end_battle`, around `end_unowned_battle`) and the
  day's start (`town/day.rs`, `town::start_day` in `advance_day`; the marks listed only when the
  sink records). `TWN-STUB-13` for Visions' reach (decision 54).
- **The wire:** `h3-protocol`'s `Town::built_today` and `buildings: Option<TownBuildings { built,
  forbidden }>` (`WireU64`s), mapped and mapped back; no `PROTOCOL_VERSION` step (E16 M5 T2's
  rule: no message carries the game's observation before T3). The events are T3's to map.
- **The plans** (`h3-map`): the told-views property folds the told views strictly (each told town
  shows the folded view; every other view as it was, a Visions view's public facts only), the
  hygiene property compares the viewer's told Capitols, `check_towns` reads every viewer's
  `TownChanged`, the sieges' cover counts the Capitol's limit by the store; the towns' cover
  counts a told change of a garrison the viewer does not own (69 in its run) beside the Capitols
  (2). The hygiene property found that the owner's sets show an owned standard set's drawn
  level-2 dwelling (with a Fort, TWN-SET-003), which differs with the other game's seed:
  `hygienic_town` gives an owned town the standard set without a Fort; the case is kept in
  `proptest-regressions/tests/view.txt`.
- **Tests** (`h3-game` `town/tests/view.rs`, `capture.rs`, `command.rs`; the helper `telling` in
  `town/tests.rs`): the new fields of every view; a build told to each viewer, a new Tavern
  telling its owner the raised level of another town, a Tavern-less town's owner's unchanged views
  not told; the day's start telling the cleared marks before the income; a viewer that does not
  see the town told nothing; a sink that keeps nothing reaching the same state; the captures told
  per viewer, the mark included.
- **Measured:** the benches' and the env's sinks keep nothing (`NoopSink`), so nothing is read for
  them; no bench workload moved (the scoped verify's bench-check).
- **Catalog:** TWN-VEW-001 amended (the told changes), TWN-VEW-004 new (the mark public, the
  buildings the owner's; the code reading of `town_list_frame` 0x5BD700).
- **Agent's calls (change freely):** the capture no longer reads the adventure capture's viewers
  (the store and the map hold the same towns, so it always captures); a view's change told whole
  rather than field by field (the
  fold replaces the view); the owner in `TownChanged` though `TownCaptured` tells it too; the day
  start's views told although a fold could clear the marks itself; a build in a town without a
  standing Tavern reads its owner's views of every town (not only when the building is a Tavern);
  the told events after the change's own events.

### T18b2b — the legal builds and the hall's offer (2026-11-10)

One session, the user present at the start; branch `town/e09-t18b2b`, `cross-lane` (`h3-game`'s
`legal.rs` and `lib.rs`, `h3-map`'s property tests `play.rs` are the adventure lane's; `h3-sim`'s
`adventure/rollout.rs` and its town tests the AI lane's). Starting point: `master` at 4701369
(#812, #814, #815 merged). **`master` does not build:** the server lane's #814 maps T18b1's
`TownChanged { town, capitol, seen_by }` onto the wire and #815 (T18b2a) merged after it with the
event reshaped and `TownStackSeen`, `TownIncomeSeen` new, no CI run having seen both; `h3-protocol`
fails, CI's full verify and the clients job on `master` with it. Left to the server lane (the
user; its inbox has T18b2a's note); nothing here touches the wire. Fixed by its #816 (E16 M5 T3d:
T18b2a's events mapped and folded, `SRV-STUB-12` and `-13` lifted), merged into the branch after
the pull request #817 opened (a conflict in the server lane's inbox only). **Kept whole at its start**
(the user, decision 55); the rollout's builds skipped (decision 56). The user's computer was
connected at the start: the original data staged then (the staging folder given to the setup with
`extracted.tar` unpacked, so its baseline would run with the data); the setup's baseline was lost
to a VM restart, and could not have passed (`master` red).

- **The listing** (design §29 as built): `h3_game`'s `town::offer::legal_builds`, chained after
  the boats by `legal_commands`: each town the player owns, by the map's town list (`TownId`
  order), a town built in today skipped before its Capitol test, each building of
  `h3_towns::buildable` (with `holds_capitol`) whose price the treasury holds, in the record's
  order. `legal_commands`' doc and rule line (TWN-BLD-001) updated.
- **The offer:** `GameState::town_offer(rules, player, town) -> Option<TownOffer>` with
  `possible`, `buildable` and `payable` (each holding the next), `None` unless `player` owns the
  town (the trusted viewer rather than the design's `town_offer(town)`: the buildings are the
  owner's, TWN-VEW-004), in its turn or not; `TownOffer` exported by `h3-game`.
- **The rollout** (decision 56): `h3_sim::adventure::rollout::drawn_commands`, the listed commands
  but `GameCommand::Town`, which `choose` draws from; the mask's `command_entry` already gave the
  builds no entry. No random game, replay or bot report moved.
- **The plans** (`h3-map` `play.rs`): the consistency property expects, after the boats, the
  builds `apply` accepts (`accepted_builds`: a probe of every building of every town of the store
  and one past each); `check_town_builds`: each listed build applies, adds `built_with`'s set with
  the mark, is told as `TownBuilt` and pays its building's cost, and the owner's offer has it
  payable; every unlisted probe is refused with the state unchanged and no event; the offer is
  the owner's alone and its payable set the town's listed builds. The move loop skips the builds;
  `check_rewards` explains a build's `ResourcePaid` by its cost; `check_towns` lets a record
  change by one `TownBuilt` (its set and the mark, no capture) or the day's start (every mark
  cleared after a `DayStarted`), each told. The plans build through `Step::Legal`: the towns' cover
  test counts 70 builds and 61 marks cleared in its 512 cases, its other counts all still met.
- **Tests** (`h3-game` `town/tests/offer.rs`, T18b1's fixtures shared from `command.rs`): the
  listing is the payable part of the hall (the Citadel's price refused), each listed build
  accepted; the offer's three sets on the priced record; the offer none for a non-owner, another
  player's town and an unknown town; a town built in today lists none until the day's start; two
  owned towns' builds in town order with one Capitol. `h3-sim`: the rollout draws no build on red's
  first turn of the town map (where `legal_commands` lists some).
- **Measured** (design §36): `bench-check` of the five `random-play-*` workloads (maps without
  towns): duel +0.44 %, dwellings −0.00 %, economy −0.46 %, rewards −0.07 %, sea +0.13 % (gate
  2 %; no re-baseline). Callgrind of random play on the town map (three games of 21 days, seed 173,
  469 steps): `legal_commands` 0.65 % of the rollout, `buildable` 0.34 %, `holds_capitol` 0.06 %,
  `Reach::search` 97 %: no cached offer.
- **Catalog:** TWN-BLD-001 amended (the listing and the offer).
- **Agent's calls (change freely):** `town_offer` takes the viewer; the listing walks the map's
  town list rather than the store; a marked town skips its Capitol test; the plans' probe covers
  one town and one building past the last; the rollout's skip a named function the test reads.

### T18c — the town events at the turn start (2026-11-10)

One session, the user present at the start; branch `town/e09-t18c`, `cross-lane` (`h3-game`'s
`state.rs`, `setup.rs`, `command.rs`, `observe.rs` and `h3-map`'s `tests/view.rs` are the
adventure lane's; `h3-protocol` the server lane's, with its version). Starting point: `master` at
3e1026f (#817, #820 merged); the setup's baseline, the full engine verify with the original data,
green. The user's computer was connected at the start: the original data staged then. **Kept
whole at its start, the plans' checks moved to T19** (the user, decision 57). Before it, at the
user's word, the inbox's hygiene counterexample (#822, `cross-lane`): CI's seed replayed on the
code CI ran (780fbcc); another player's hero accepted a monster's offer to join beside a mine the
viewer saw, its army (varied where the viewer never sees it) full in one game (an army dialog held
the visit) and not in the other (the mine flagged at once); the property's `outcome` compares
every decision kind now, the seed kept.

- **The day's test, once** (`h3-game` `setup.rs`): `setup::Timing { players, human, computer,
  first_day, interval }::fires(player, human, day)`, the map's timed events' test moved there;
  `TimedEventSetup::fires` builds one.
- **The firing** (`h3-game` `town/events.rs`, new): `town::fire_events(state, rules, player,
  sink)`, called by `GameState::start_group` right after `fire_events`: the events that fire are
  listed first (towns by `TownId` whose adventure owner is the player, a town without events
  skipped before its owner is read; each event by `Timing`), no allocation when none fires; per
  event its gains (`GameState::gain`), its losses (`GameState::lose`, down to 0),
  `TownEventFired { player, town, event }`, then each building of `h3_towns::event_buildings`
  (water: `StaticMap::town_boat_spot`) not standing by its turn (an earlier building of the event
  through `auto_with`, or an earlier event of the turn start, may have brought it) built and told
  by `town::command::build_told`;
  its creatures stubbed (`TWN-STUB-10`).
- **The build and its tell, shared** (`town/command.rs`): `build_told(state, rules, player, town,
  building, event, sink)`: the Tavern's every-town views (TWN-VEW-002), `Told::before`,
  `Towns::build` with the mark only for the command (`event` none), `TownBuilt { event }`,
  `Told::tell`. The command pays before it now (the treasury is in no town view, so the told
  views and the events' order are as before).
- **Day 1:** `GameState::new` starts the first group's turn with `StartSink`, which keeps only
  `TownEventFired` (`RECORDS` false: no view computed at the build); kept in the state's calendar
  block (`Calendar { events, town_events, start_events }` behind the former timed events' field,
  now `calendar: Option<Arc<Calendar>>`; the start's events not hashed: no rule reads them, they
  follow from the setup) and `GameState::town_events_at_start(player) -> Vec<(TownId, u16)>`. Not in the observation (the server lane's inbox: a session may tell a seat
  of them as the observation's `events_today` names the map's).
- **The event** (`command.rs`): `GameEvent::TownEventFired { player, town, event: u16 }`, the
  owner's alone (`observe.rs`' projection). **The wire** (`h3-protocol`): mapped,
  `wire::game::GameEvent::TownEventFired`, the fold ignores it (what it gave and built comes as
  the events around it), the round-trip test's 36 kinds, the fold test's unknown-town case;
  **protocol v41** (`master` merged in before the pull request: the adventure lane's #819 took
  v40 for `Revisit`). `h3-map`'s told-events check: the owner's.
- **Tests** (`h3-game` `town/tests/events.rs`): the order at the turn start (`TurnStarted`, the
  map's event and its gain, the town's gain, `TownEventFired`) and the treasury; the owner alone
  sees it; an event fires once without an interval, every second day with one, never for a player
  it does not name, of the other kind, or in a neutral town, and for its town's owner at the start
  (a town given to red); the losses down to 0; the buildings without the hall's tests or the mark
  (the Capitol, the Grail's building, a forbidden silo; a Shipyard without water skipped; each
  `TownBuilt` with the event's place followed by its views; the day's build still free after),
  a creatures-only event firing alone; an event's second Capitol beside its owner's other (T-8);
  day 1's inside the build (gold and building at the start, `town_events_at_start` for red only,
  blue's day-1 event told at red's end of turn, kept by a fork, none for a later event).
- **Catalog:** TWN-EVT-001, -002, -004 new; TWN-EVT-003 the game's half; GAM-CAL-004 amended (the
  towns' events after the map's, one test). **Stub:** `TWN-STUB-10` (the events' creatures,
  increment 3). No quirk; TD-12 (the original's day) and T-8 as decided.
- **Measured** (`bench-check`, all 28 workloads against `master` built in the same sandbox, the
  setup's run): first build, the start's events in a block of their own (`Option<Arc<[..]>>`
  beside the timed events'): every fork +13 Ir (fork-economy +0.67 %, fork-rewards +0.48 % on
  `master`, which already stood +1.4 % and +1.9 % over their baselines: +2.05 % and +2.40 %, past
  the gate). Settled once: the timed events' slice, a town-events flag and the start's events in
  one `Calendar` block behind a thin `Option<Arc<Calendar>>`: every fork 3 Ir cheaper than
  `master` (−0.08 % to −0.12 %), `day-start-towns-36` +0.13 % (the flag skips the town loop on maps
  whose towns have no events), the random-play and reach workloads within 0.01 %; no re-baseline.
- **Agent's calls (change freely):** `Timing` as a borrowed view of either event; the events listed
  before any is applied (the store is read while the state is written); a building standing by its
  turn skipped rather than told again; the command pays before `Told::before`; day 1's kept in the
  calendar block and out of the hash; `town_events_at_start` by (town, place), not in the
  observation (the server lane's choice).

### T19a — the writer's town events, the builds map and its replays (2026-11-10)

One session, the user present at the start; branch `town/e09-t19a`, `cross-lane` (`h3-map`'s
`writer.rs`, `writer/economy.rs` and `tests/`, the fixtures, the replays and `h3-cli`'s replay
test are the adventure lane's). Starting point: `master` at 3f60bf4 (#824, then #825 and #826
merged during the setup); the setup's baseline lost to a VM restart and run again on 3f60bf4:
every step green but coverage, where `h3-map`'s `every_step_keeps_the_commands_invariants` found
a counterexample of `master`'s (`check_rewards` expects a guarded Mystical Garden's gain before
the joining monster's treasure; the adventure lane's inbox, the user's word). The user's computer
was connected at the start only: the original data staged then (57 files). **T19 cut in three at
its start, increment 2's own map** (the user, decisions 58, 59).

- **The writer** (`h3-map` `src/writer/town.rs`, the town lane's): `writer::TownEvent { event:
  TimedEvent, buildings, creatures }`, `AnyTown::events` (in the town's order); a town's body
  writes them after its spells (each timed event as the map's, `economy::write_event` factored out
  of `write_events`, then the six bytes of buildings, the seven counts, four unused bytes). The
  check refuses an event's building bit past the 41 slots as a custom set's (`TWN-QUIRK-7`: the
  writer writes none). Tests: an event and an empty one read back as written; the bit refused; an
  event's text the format cannot hold fails as written.
- **The builds map** (`fixtures/maps/e09-builds-36.h3m`, `h3-map` `tests/builds_map.rs`; decision
  59): grass with the sea on rows 31 to 35; red's main Castle "Harbour" by the sea (its boat tile
  water) with four events (a day-1 Marketplace and Blacksmith, an every-day and a weekly gift,
  creatures on day 3), red's Castle "Keep" inland (the Capitol's requirements, the Guardhouse, the
  Barracks and the Upg. Griffin Tower; an empty garrison; blue's every-day gift, red's day-1
  Shipyard and Resource Silo), blue's main Conflux "Nexus" (a Capitol; the Altar of Water
  forbidden, so the Altar of Earth is never possible), blue's Tower "Spire" (the Capitol's
  requirements; blue's day-2 Capitol). Tests: the file as written, the import's sets and events,
  the build's Shipyards (only the Harbour's boat tile is water; the Keep's and the Nexus' Shipyards
  forbidden), red's four day-1 events with their builds (the Keep's Shipyard skipped) and gains
  (treasury 23 wood, 23 ore, 26 000 gold); each refusal kind once on the first turns with the
  state hash unchanged (blue out of its turn, an unknown town, another's town, a building past
  the record, ever built, forbidden for want of water, the Grail's, a requirement never built,
  the horde offered as its upgrade, the mark after the Harbour's Shipyard, the treasury, the
  Capitol while the Nexus' stands), the Altar of Earth not possible, the Spire's day-2 Capitol
  beside the Nexus' (T-8); a town built in today and taken keeps its mark (both players human),
  its Capitol limited to the City Hall, its event of blue's firing for blue on day 2.
- **Golden replays** (`fixtures/replays/adventure/`, 69 now): `builds-week-s1` (a build a day in
  the Harbour: the Shipyard, the Lighthouse, the Resource Silo, the guild's first level, the Town
  Hall, the City Hall, the Citadel), `builds-capture-s1` (the Keep's Capitol, Yog's capture the
  same day, blue's build there the next), `builds-events-s1` (both end their turns to day 8: every
  event fires on its days; blue's two Capitols pay 8 000 gold a day from day 3). A replay holds no
  refused command (`Replay::record_commands` and `verify` stop at one), so the refusals and the
  Capitol refused while another stands are the map's test, not replays (design §32 listed them as
  replays). Documented in `adventure_replays.rs` and `fixtures/maps/README.md`.
- **Other lanes' gates over every replay** (the server lane's files, `cross-lane`): `h3-protocol`'s
  fold test (`tests/game_fold.rs`: the fold against `observe`, no exemption needed) and game
  session test (`tests/game_session.rs`: one restore a replay) and `h3-server`'s record test
  (`tests/record.rs`) pin the replays' count: 66 to 69; both crates' suites pass.
- **No version moved** (the writer's change writes what the import reads since T7a1); no rule, stub
  or quirk new. The plans, the env and the census are T19b and T19c.
- **Agent's calls (change freely):** the map's names and positions; the Keep's dwellings for the
  horde's refusal (the Griffin Tower's requirements' closure needs the Barracks and the
  Guardhouse); the checkpoint intervals (a day, a command, a day); Yog's two moves (the reach stops
  where blue's explored area ends).

### T19b — the plans build (2026-11-10)

One session, the user present at the start; branch `town/e09-t19b`, `cross-lane` (`h3-map`'s
property tests `play.rs` and `view.rs` and their regressions file are the adventure lane's).
Starting point: `master` at 1769e23 (#827, #828 merged); the setup's baseline, the full engine
verify with the original data, green. The user's computer was connected at the start: the
original data staged then (each LOD a call, 13 files in one, the 40 MP3s in one). **Kept whole at
its start, `master`'s counterexample of the plans' gains fixed first** (the user, decisions 60,
61).

- **The counterexample** (T19a's, in the adventure lane's inbox): reproduced with its seed on
  `master`'s strategy (6 s). `check_conservation` expected a visit's own gains (`visit_gains`: the
  guarded Mystical Garden's 500 gold) before what the objects that left gave (the joining
  monster's 50-gold treasure); the engine's order (the join, then the visit it held, E08 decision
  279) is right, so the visit's gains came last. The adventure lane's #830 (E08 T106b1) fixed it
  on its side the same day (a guard's treasure inserted before the visit's gains, the seed kept):
  at T19b's merge of `master` (the pull request in conflict) its fix was kept and this one
  dropped, and the inbox note names only the hygiene gap below.
- **The plans' towns' events** (`play.rs`): `TownPlan::events` (`town_events`: none in half the
  towns, else one or two, each a timed event as the map's, `timed_event` factored out of `timed`,
  with none to three of the 41 map slots as buildings and creatures by level), written by
  `add_town` (before SoD with the human flag set, as the map's events: the reader's).
- **`Step::Build(n)`**: the acting player's `n`-th listed build, else as `Legal(n)` (`nth_listed`,
  shared with `Step::BuildBoat`); `steps()` gains runs of builds (one to eight times a `Build` and
  none to three ends of turn).
- **`check_timed`** (decision 57's "`check_rewards`": the turn start's gains are compared here, the
  treasury in `check_conservation`): after the map's events, each owned town's events that fire
  (the same day test), in `TownId` and list order: their gains (saturating), their losses down to
  0, `TownEventFired`, then a `TownBuilt` with the event's place for each building of
  `h3_towns::event_buildings` of the town as the turn starts (with its Shipyard's water) not
  standing by its turn; the views a build tells after its `TownBuilt` are left out (the told-views
  property's). `count_timed` leaves the towns' gains and losses out of the map events' counts.
- **`check_towns`**: a command's build alone with its mark, or a turn start's event builds, any
  number, in their order, none standing before it, the set the fold of `built_with`, without the
  mark (cleared if the day started, else as it was).
- **Kept from T18b2b:** the legal builds are those `apply` accepts and every unlisted probe is
  refused with the state equal (its hash too) and no event (`check_town_builds`); the day's start
  clears every mark (`check_towns`).
- **Cover tests:** `the_plans_capture_and_besiege_towns` counts the builds by command and by event
  (154, 62 in its 512 cases) and the events fired, giving, taking and naming a building standing
  (1 224, 752, 902, 1 093), its other counts met. `town_views_move_as_the_events_say` counts a
  day's mark coming into its owner's view and another viewer's (233, 241: the mark public,
  TWN-VEW-004), the owner's buildings told after a build (297) and the town events told (1 039);
  raised from 256 to 512 cases (the new draws met no view at the taverns' second level in the
  first 256). The hygiene and told-views properties pass with both, unchanged: the events draw
  nothing, and a town the viewer sees has the same events in both games. `h3-map`'s 156 tests pass
  (530 s in debug, one job).
- **A gap of the hygiene property's** (`master`'s, found by the scoped verify's coverage run on
  the new draws; the agent's call, as decision 61's): another player's hero declined a monster's
  offer to join, and the monster fled in one game (removed, which the viewer saw) and asked again
  in the other, its answer following from the visitor's army and the seed the test changes;
  `outcome` compares the objects a command removed (`ObjectRemoved`'s kind, beside the guards'
  hints) now; the seed kept in `proptest-regressions/tests/view.txt`. The three properties of
  `play.rs` passed three more runs, the hygiene and told-views ones ten.
- **`master` merged in** (#830's teleporters plans: `Step::Jump` and its runs beside
  `Step::Build`'s, every game's draws moved again; #831; the pull request was in conflict):
  `Step::Build`'s helper renamed `nth_listed` (#830 has a `listed` of its own); `h3-map`'s 157
  tests pass on the merged draws but #830's `the_plans_jump_through_teleporters`, whose own plans
  the town field moved: no battle at a Whirlpool's exit in its 256 cases, raised to 512 (met
  twice); the towns' cover test there met everything again (builds 177, 51; events 523, 427, 460,
  300), the town views' too (marks 212, 263).
- No version moved; no rule, stub or quirk new; no replay, workload or bot report moved.
- **Agent's calls (change freely):** the hygiene gap fixed here rather than sent to the adventure
  lane (its pull request's CI would meet it); the events' draw (half the towns, up to three slots, the
  map's days); the views after a build left out of `check_timed` rather than predicted; the
  standing-building count by the ever-built set before the command; 512 cases rather than another
  draw.

### T19c1 — the env's build entry, random play with builds (2026-11-10)

One session, the user present at the start; branch `town/e09-t19c1`, `cross-lane` (`h3-sim`'s
`src/adventure/` is the AI lane's; `h3-cli`'s `src/adventure/`, its replay test, the replays and
the workloads the adventure lane's; the replay counts in `h3-protocol` and `h3-server` the server
lane's). Starting point: `master` at 49cc2d5 (#832, #833 merged); the setup's baseline only the
light gates (CI's full verify green on 102b81b, the same engine code). The user's computer was
connected at the start: the original data staged then (each LOD a call, 13 files in one, the 40
MP3s in one). **T19c cut in two at its start, the env first** (the user, decision 62).

- **Action encoding v9** (`h3-sim` `src/adventure/action.rs`): the entry `BUILD` (30, appended;
  `ENTRY_COUNT` 31) and the building head (`MapAction::building`, a `u8`; `BUILDING_COUNT`, 64,
  `h3_towns::MAX_BUILDINGS`). `legal_mask` sets `BUILD` when `legal_commands` lists a town's
  build; `tile_mask` after `BUILD` allows the entrances of the listed builds' towns (`town_tile`:
  the town object's entrance, its visitor's tile); `building_mask` (new, also on `AdventureEnv`)
  after `BUILD` allows the listed builds' buildings of the town whose entrance is the tile (by
  their record index), after another entry 0 alone at a tile its tile mask allows, else nothing
  (the battle encoding's "0 for another legal prefix"). `decode` of `BUILD` gives
  `TownCommand::Build` of the town at the entrance (`StaticMap::town_at`) and the building at the
  index (the owner, the price and the mark `apply`'s); with another entry a building other than 0
  decodes to nothing; `encode` gives a build back by its town's entrance and its building's raw
  index. ADR-0017 point 8 already said it (T15).
- **The rollout** (`rollout.rs`): `choose` draws from every listed command, each build one choice
  as each Shipyard's `BuildBoat` (decision 56's `drawn_commands` gone).
- **Tests** (`tests.rs`, `tests/towns.rs`): `check_masks` checks `BUILD` (`check_build`: the tile
  mask the listed builds' entrances, at each the building mask those builds' buildings, each
  decoding to its command and encoding back, an entrance-less tile's mask empty) and the building
  head after every other entry; `legal_by_masks` reads the building head; the main property's free
  picks draw a building. The town map's env test plays seeds 142 and 244 (a release scratch search
  over 1 to 299, the shortest pair whose union meets its features: 425 steps, 66 builds; 36 and
  330 had changed course), counts the builds and checks after each that its town leaves `BUILD`'s
  tiles for the day (TWN-BLD-004); the fork test's siege seed is 244 (a siege on day 2);
  `the_rollout_draws_the_builds` replaces T18b2b's skip test.
- **Random replays with builds** on `e09-builds-36.h3m` (decision 59), 14 days, recorded with the
  rollout: `builds-s262-d14`, `-s271-d14`, `-s290-d14` (a search over 1 to 399: every game builds
  in all four towns 25 to 34 times; 7 capture a town; none builds a boat or opens a siege; the
  three each with the Keep's Capitol by command and blue's capture of the Keep, two of them on a
  day red built there, two with red's second Capitol in the Harbour after the loss); 72 replays
  (the counts in `h3-protocol`'s fold and session tests and `h3-server`'s record test). The town
  map's `towns-s*-d21` keep their games of before as their commands (a replay is its commands).
- **The benchmarks** (master built in `target/wt-master`, both measured in this sandbox): the
  maps without towns the same within ±0.01 % (the draw's filter gone); the town map's random games
  changed course, every counted function the same code (`callgrind_annotate` per function):
  `day-start-towns-36` +3.29 % (the week's builds: more halls' and silos' income,
  `h3_towns::daily_income` 25 000 to 35 400 Ir, and the marks cleared), `fork-towns-36` -6.12 %
  (another state forked), `reach-towns-36` +24.84 % (473 steps and 1 850 queries for 425 and
  1 584, about 469 000 Ir a query; `Reach::search` 71.6 % in both), re-baselined with the reason;
  `siege-towns-36` the same (its heroes seek the towns). `h3-cli`'s `agent_rollout` (the wall-time
  variant `--masks`) reads the building head too.
- **CI's counterexample of `master`'s** (the pull request's first full verify, coverage step:
  `h3-map`'s `every_step_keeps_the_commands_invariants`, reproduced on `master` 4bd174e with its
  seed): a hero stepped onto another player's town's entrance that a monster guards and its
  question (an offer to join) opened; the town's visit waits for the answer (decision 279), but
  `check_invariants` exempted only a hero whose player is in a battle. A gap of the test, not of
  the engine: it exempts a hero whose player has any open decision (`GameState::has_decision`);
  the seed kept in `proptest-regressions/tests/play.txt`; the town lane's trap amended.
- The AI lane's inbox told (v9; it replaces T18b2b's note). No game version moved; no rule, stub
  or quirk new; no bot report moved (they play battles).
- **Agent's calls (change freely):** the building head a `u8` in `MapAction`; each listed build
  one choice of the random agent (as `BuildBoat`'s), not the entry first; the building head after
  another entry by its tile's legality (a reach query for a move) rather than the entry's alone;
  14 days and three replays on the builds map; the invariant's gap fixed here (the adventure
  lane's test file, already in this `cross-lane` pull request) rather than sent to its inbox.

### T19c2 — the new workloads and the census of the 160 maps (2026-11-11)

One session, the user present at the start; branch `town/e09-t19c2`, `cross-lane` (`h3-map`'s
`examples/original_data.rs`, `h3-cli`'s `src/adventure/`, the workloads and `docs/formats/h3m.md`
the adventure lane's; the map client's inbox), the benches README's rows in
`shared/e09-t19c2-benches` (#841). Starting point: `master` at 81024ba (#839, #840 merged); the
setup's full baseline was lost to a VM restart during its coverage step, so CI's full verify on
`master`'s engine code (5613da6) stands for it. The user's computer was connected at the start:
the original data staged then (each LOD a call, 13 files in one, the 40 MP3s in one; 57 files).
**T19c2 kept whole at its start** (the user, decision 63; the agent had recommended the census
first and the workloads in a later session).

- **The census** (`h3-map`'s `original_data`, exit criterion 6; `played_town_events`, `fire`,
  `survey_order`, `town_event_differences`): every one of the 160 games that build (seed 0) is
  played by `EndTurn` alone, a week (`PLAYED_DAYS`, 8: the week's step), a map whose owned towns'
  events fire later on to the last first day of such an event (a repeating one to its second
  firing): 141 maps 8 days, the longest 122 and 168 days; the whole check takes 16 s in debug.
  The census walks TWN-EVT-001 and -003 itself from each town's record before its events (the
  same game built again without the towns' events, its factions checked the same): the owner at
  the start, the kind by its `human` (the game's `TimedEventSetup::fires`), each event in order,
  `h3_towns::event_buildings` with the town's water, each building through `built_with` unless
  standing by its turn. Checked against the play (an error fails the map): the events fired by
  day, town and place (day 1's of the first group from `GameState::town_events_at_start`), the
  buildings built after the build by day, town, event and building (`TownBuilt`), the first
  group's towns' ever-built sets after the build. All 160 hold. Counted: of the 369 events 299
  fire, 23 are a neutral town's, 8 do not name their town's owner, 39 name it but not its kind;
  496 firings (236 with resources, 38 with buildings, 59 with creatures, `TWN-STUB-10`, 190 a
  message alone); 34 buildings built (2 inside the build), the fixed towns' 18 (9 without a
  requirement: the `Back For Revenge` pair's Rampart day-11 dwellings, `Undead Unrest`'s day-14
  upgraded Hall of Darkness), 18 skipped as standing, 5 folded into the building that supersedes
  them; the random towns' 16 and 4; no Shipyard is an event's. T18a's counts stand: 591 of the
  625 towns of a faction with a Shipyard have no spot, no custom set loses a built one.
- **The survey's census reproduced** (survey `E09-construction.md` §10, "Checked by play"): the
  fixed towns' events in its order (every event once, day order, owned or not) give 57 built, 30
  skipped as standing and 27 without a requirement (`SURVEY_TOWN_EVENTS`, checked) once its terms
  are T12b's: each of an event's buildings built one by one in id order, the superseded one too
  (the game builds the superseding one alone: 38 of the 57, 19 folded), and the executable's
  direct requirement lists (the part's lists are closed: 23 of the 38 without one). The ever-built
  sets after each event are the same both ways (checked). A first count with the game's terms
  gave (38, 30, 23); the two conventions were found by trying the variants.
- **The workloads** (`benches/adventure/`, baselined at their first count; `master` built in
  `target/wt-master`, measured in this sandbox): `legal-towns-36`, the new workload kind `legal`
  (`h3 adventure bench --workload legal`: `h3_game::legal_commands` before each step of a player's
  seat in the map's phase, inside `legal_queries`; `BenchResult::builds` counts the builds
  listed), 16 random games of 7 days on the town map: 1 389 steps, 1 172 listings, 3 467 builds
  listed, 8 129 621 Ir (about 6 900 a listing: `h3_towns::buildable` 27 %, `offered_as` 18 %,
  `payable` 7 %, `holds_capitol` 6 %). `day-start-builds-36`, the `day-start` kind on the builds
  map (16 games of 7 days): 359 758 Ir, about 22 500 a day start, the town events 15 %
  (`fire_events` 9 %, `firing` 6 %); `master`'s 359 566 (+0.05 %, the same code).
  `day-start-towns-36` unchanged: 854 090 against its 853 501 (+0.07 %, `GameState::end_turn`'s
  inlining in the rebuilt binary, every other function the same); its day starts clear the marks
  since T19c1 (`Towns::start_day` 128 times). `h3-cli`'s bench test runs `legal` on the town map
  (builds listed) and the duel (none).
- The map client's inbox told (the town screen's builds: `TownCommand::Build`,
  `GameState::town_offer`, `TownBuilt`, `TownChanged`, `TownEventFired`). No version moved; no
  rule, stub or quirk new; no replay or bot report moved.
- **Agent's calls (change freely):** the census in the existing check rather than a test (it
  needs the original data; 16 s in debug); a week for every map and the later first days for the
  29 with events, a repeating event to its second firing, rather than a fixed horizon; the
  census's own walk of the rules rather than a copy of the survey's throwaway; the survey's
  convention reproduced in `survey_order` beside the game's terms; `legal` counting the whole
  listing but the moves (`legal_commands`) rather than the builds alone, on the town map; the
  builds map's day 8 for its turn start (the week and two of the Harbour's events).

### T20 — the audit; increment 2 closed; increment 3 refined (2026-11-11)

One session, the user present at the start; branch `town/e09-t20`, docs only but for four
comments in the town modules (the renumbered increments), with the ADRs' citations in
`shared/e09-t20-increments`. Starting point: `master` at 1e54a50 (#841 to #845 merged; CI green
there; the setup's `verify-once` found CI's fingerprint of the engine code: the light gates
only). The user's computer was connected at the start: the original data staged then (each LOD
a call, 13 files in one, the 40 MP3s in one; 57 files). **At its start** (the user, asked with a
first look at the audit, `TWN-Q11`): T20 whole (decision 64) and increment 3's two questions
(decisions 65, 66).

**The audit of increment 2's exit criteria** (`E09-archive.md`), on `master`'s code:

1. **The build.** TWN-BLD-001 to -007, TWN-SET-011: `h3-towns`' `build.rs` tests
   (`the_hall_offers_what_the_ever_built_set_allows`, `the_hall_offers_a_horde_as_its_upgraded_horde`,
   `a_capitol_is_offered_only_to_a_player_without_one`, `the_hall_never_builds_the_grail`,
   `a_town_built_in_today_offers_nothing`, `every_refusal_comes_in_its_order`,
   `the_offer_is_what_check_build_allows_and_possible_holds_it`), the part's
   `the_buildings_close_what_they_require_and_list_their_costs`; `h3-game`'s
   `town/tests/command.rs` (`an_owner_builds_in_its_turn_and_pays_the_price`,
   `a_town_is_built_in_once_a_day_and_the_day_s_start_clears_the_mark`,
   `every_refused_build_changes_nothing`, `a_capitol_is_refused_while_its_player_holds_another`),
   `town/tests/offer.rs` (the legal builds the payable part of the hall, in town order, one Capitol),
   `town/tests/build.rs`' `a_town_without_water_loses_its_shipyard_at_the_start`; `h3-map`'s
   `each_refused_build_on_the_builds_map_changes_nothing` (every refusal kind, the state hash
   unchanged). The costs and the closed requirement lists by the harvest (`--with-original-data`'s
   byte-for-byte packs). **Holds.**
2. **A build changes increment 1's reads at once.** TWN-BLD-005: `h3-towns`' store test
   `increment_1_s_reads_follow_a_build_at_once` (the Town Hall's gold, a Citadel's fort and moat in
   the siege's facts, the Tavern's view level, the silo's resources, the Capitol public with its
   gold), `a_build_adds_the_building_and_what_it_supersedes`,
   `the_automatic_buildings_follow_to_a_fixed_point`; the views' `a_build_tells_each_viewer_its_
   changed_view_and_a_new_tavern_its_owners_raised_levels`. **Holds.**
3. **The town events.** TWN-EVT-001 to -004: `town/tests/events.rs` (at the owner's turn start
   after the map's events; only for the town's owner, named and of its kind; losses floored at
   zero; built without the hall's tests or the mark; a building two events name built once; T-8's
   Capitol beside the owner's other one; day 1's inside the build), `h3-towns`'
   `an_event_builds_without_the_halls_tests` and `TWN-QUIRK-7`'s test; TD-12 through the map's
   events' own `TimedEventSetup::fires` (the first day, then every interval); the creatures
   registered as `TWN-STUB-10`. **Holds.**
4. **The views.** TWN-VEW-004, `docs/06-security.md`, "A town's construction (E09 increment 2)":
   `town/tests/view.rs` (a build's changed view to each viewer, the day's start's cleared marks,
   a viewer that does not see the town told nothing), the told-views property
   `town_views_move_as_the_events_say` and the hygiene property's `vary` with the builds and the
   events (T19b), `vary_changes_the_towns_the_viewer_never_sees_only`. **Holds.**
5. **The env, the plans, the replays, determinism.** `h3-sim`'s `the_rollout_draws_the_builds`,
   `a_build_action_names_its_town_by_the_entrance`, `the_env_plays_the_town_map_to_its_end_of_days`,
   `town_map_rollouts_and_forks_are_deterministic` (T19c1, action encoding v9); the plans'
   `check_town_builds`, `accepted_builds` and `check_towns` (T19b); the golden replays
   `builds-week-s1`, `builds-capture-s1`, `builds-events-s1` and the random `builds-s262-d14`,
   `-s271-d14`, `-s290-d14`; `h3-towns`' `a_build_writes_one_town_of_a_fork_and_sets_its_mark`.
   **Wording:** "golden replays cover each refusal" no longer matched (a replay holds no refused
   command, T19a: the refusals are the builds map's test): worded as built. **Holds.**
6. **The census.** `h3-map`'s `original_data` run again here on `master`'s code with the staged
   data: all 160 maps import, build and play their town events (141 a week, the 29 with later
   events to their last first day, the longest 168 days); 369 events, 299 fire, 496 firings (236
   with resources, 38 with buildings, 59 with creatures, 190 a message alone), 34 buildings built
   (2 inside the build), 22 skipped as standing, 5 superseded; no Shipyard an event's; 591 of the
   625 towns of a faction with a Shipyard without a spot, no custom set losing a built one; survey
   §10's 57 / 30 / 27 in its order: T19c2's counts, unchanged. **Holds.**
7. **Batch TWN-2.** Scored by T14c (survey `E09-construction.md` §13: TD-9 the reading, TD-11
   refused by the build dialog's owner test), applied by T15 (ADR-0028, -0016, -0017). **Holds.**
8. **The gates, the versions, the inboxes.** `h3-towns`, `h3-adventure` and `h3-game` under the
   100 % coverage gate with no town file excluded; CI's full verify green on `master` at 1e54a50;
   `check-rules` passes (459 rules). The versions in the lane HANDOFF (town state hash v2, part v4,
   game state hash v26 by T17a, adventure part v19 by T18a, replay format v11 by T18b1, protocol
   v41 by T18c, action encoding v9 by T19c1). Told: the AI lane's inbox (T19c1), the map client's
   (T19c2), the server lane's (T18b1, T18b2a, T18b2b, T18c). **Wording:** the server's manifest
   of the part's hash is the server lane's own (`GameManifest::town_ruleset_hash`, its E16 T4 of
   2026-11-10, after the part's v4): worded as built. **Holds.**

**The gap fixed:** TWN-BLD-001 cites batch TWN-2's rows 1 and 2 and matched them in every point it
decides (the owner alone in its turn, the price from its treasury, the mark; nothing changed by a
refused build) but stayed `implemented`: now `verified`, with a note. Stay `implemented`:
TWN-BLD-003 (TWN-2 does not observe the hordes, design §36), -004 and -005 (the mark and the sets
observed in part: TWN-2-1 shows one build's mark and sets, not the next day's or the automatic
buildings), the TWN-EVT rules and TWN-SET-011 (the reading with VCMI and the wiki; the census is
the engine's own walk) and TWN-VEW-004. **Left open, by design:** `TWN-STUB-2` (the Grail,
increment 6 and E10), `TWN-STUB-6` (increment 5), `TWN-STUB-8` (the battle lane's inbox),
`TWN-STUB-10` (increment 3), `TWN-STUB-13` (increment 5), `TWN-STUB-1` (E10's, reserved). No code
gap queued.

**Increment 2 closed** (2026-11-11): its section (the refinement's note, decisions 36 to 63, the
task list and the exit criteria with criteria 5 and 8 worded as built) moved verbatim to
`E09-archive.md` (checked by script: every line of the old epic in the new epic or the archive);
its design note's Part D stays in `E09-design.md`, as Parts A to C (decision 40).

**Increment 3 refined** (`TWN-Q11`; decisions 64 to 67): growth, recruitment and the army rows;
the Tavern a new increment 4, the guild and the services 5, travel and the Grail 6. The live
citations renumbered (agent's call, decision 67): `town.toml` (TWN-SET-005, -007, -009,
TWN-BLD-002, -005), `docs/stubs.toml` (`TWN-STUB-6`, `-13`), `h3-towns`' `Cargo.toml` and
`command.rs`, `h3-game`'s `town/heroes.rs`, the queue's inbox, and in `shared/e09-t20-increments`
ADR-0016's town boat spot, ADR-0017 points 1 and 4 and its consequences, ADR-0028 points 1, 6 and
8 and `docs/06-security.md`'s construction list; the log, the archive, the surveys and the design note
keep their words. The adventure lane's inbox told of the army exchange (its increment 12's
garrisons, decision 295): one army reference agreed before either lane drafts. The inbox's
2026-11-11 note (the intermediate `TownChanged` at a capture under the Capitol's limit) kept as
built and archived. The queue holds T21 (the survey) and T22 (the drafts).

**Verify:** the engine's code is `master`'s but for four comments. `cargo xtask verify --scope
town --summary --with-original-data` with no failing step (coverage 8.5 minutes, determinism 8,
bench-check 6, original-data 0.6; `deny` and `ffi-smoke` skipped by the scope); `check-lane`
passed (the town lane's paths, the common `docs/stubs.toml` and the adventure lane's
`QUEUE.md`); `shared/e09-t20-increments` (#846) `check-lane` and `check-docs`. CI on the pull
requests is the record.

### T21 — increment 3's survey (2026-11-11)

Baseline d3bf756 (#846 to #850 merged; the setup's baseline the light gates, CI's fingerprint on
`master`). The user's computer was connected only at the start: the original data staged then
(the 57 files, each call first time). **The cut** (decision 68): the agent measured the unread
code (growth about 3.5 to 5.5 KB, recruitment and the rows about 2 to 3 KB, much of the latter
read by T1 and E08) and recommended two parts; the user kept T21 whole.

**The survey** `docs/rules/surveys/E09-growth.md` (§0 to §16): a dwelling's growth
(`town_dwelling_growth` 0x5BFB60: the castle on the base, the Statue on base and castle, the
Legion pieces of the town's two heroes, one horde, the external dwellings, the Grail on the
whole), the week (its creature the exact one, the Deity of Fire every week, a neutral town's
half), the month (the exact creature doubled after the week's growth; the Plague), a neutral
town's garrison week (`neutral_town_garrison_week` 0x4BF570), the Portal of Summoning (a
dwelling then a creature, the count set, drawn too when the town screen shows an empty Portal), a
dwelling's first creatures, recruitment (one dialog over a town's index, an upgraded dwelling's
two creatures from one count, always into the garrison army), the two rows (the army dialog's
moves, both heroes' last stacks kept; the visitor in, all or nothing by `army_absorb` 0x44B620;
the garrison hero out, refused at eight heroes; the swap), what a garrison hero changes. VCMI
1.7.5 and the wiki compared (§11), the census of the 160 maps by a throwaway function in
`original_data` (not committed, §12), disputes TD-13 to TD-17, quirk candidates T-10 to T-13,
batch TWN-3's six rows, the army reference proposed (§15). The address map: 22 new entries (the
growth's functions and helpers, the town's hire, the hero moves, three tables) and 9 summaries
extended; `structs.toml`'s town +0x33, +0x3C, +0x40.

**The checkpoint** (`TWN-Q12`, one round of four; decisions 69 to 72; the agent's calls 73): TD-13
and TD-14/T-12 `fix-optional` (the user's own answers: faithful as read, a flag for VCMI's
rule); T-11 the Portal drawn at its build and when a dwelling is flagged (recommended); TD-15 the
reading (recommended).

**The adventure lane's inbox** told the army reference proposal (the town's upper bar
`ArmyRef::Garrison` or the garrison hero's, a map garrison a sibling reference, one owner rule;
survey §15), to agree before T22 and T108b draft.

**Verify:** docs only (the survey, the epic, the log, the lane's docs, the address map, the
adventure lane's `QUEUE.md`). `cargo xtask verify --scope town --summary --with-original-data`
with no failing step; CI on the pull request is the record.

### T22a — growth's drafts and the design note's Part E (2026-11-11)

Decision 66's second task, cut in two at its start (the user, `TWN-Q13`'s first round, decision 74:
growth now, recruitment and the rows next, batch TWN-3 its own task T23; recommended over T22
whole without the batch and over the batch first). Branch `town/e09-t22a`, docs only (the drafts,
the design note, the epic, the answered questions, the lane's queue and HANDOFF, the adventure
lane's inbox). Starting point: `master` at ea6d390 (#852, #853 merged); the setup's full baseline
lost to a VM restart about 25 minutes in, CI's green run on `master` standing for it. The
user's computer was connected at the start only: the original data staged then (57 files, all
first time).

**The ADR-level choices** (`TWN-Q13`'s second round, one of three; decisions 75–77, each as
recommended): one towns' stream for every weekly town draw; the Legion's growth as two hero
effects in the core; the week's kind in the game state beside the month's.

**Read for it** (the code, besides the survey): `new_month` 0x4C8F40's head (the month's kind is
drawn first; after the week of the Imp it is the Imp's month, 0x4C8F63 to 0x4C8F82, with no
second draw, so the map spawns Imps and the towns' doubling is skipped); `town_change_owner`
0x4C62B5 (the external dwellings count by the dwelling record's first creature, +4); the base and
AB packs (the core's `horde_growth` equals the executable's horde table for all twenty-two
creatures with a horde; 112 creatures with a faction in the base pack, 126 with AB: the week's
eligible sets).

**Written:**
- **The drafts**, growth's half (T22b adds the rest): `docs/lanes/town/adr-drafts/0028-increment-3.md`
  (the record's counts and Portal, the seven rules, the part's roles `horde`, `portal`,
  `week_of`, a record's `roe`, the parameters, the seams, the versions), `0017-increment-3.md` (the
  week's kind and the towns' stream in the state, the day's order, the draws, the viewers, the
  versions, the garrison week's outcome), `0016-increment-3.md` (a neutral town's garrison written
  by the game; the dwellings read), `0013-increment-3.md` (point 16.4: a hero's town growth).
- **The design note** Part E, §37 to §47: scope and the halves (§37), the part (§38), the core's
  effects, flags and stream (§39), the record and the store (§40), a dwelling's growth and its
  inputs (§41), the week (§42), the month (§43), a neutral garrison's week (§44), the Portal
  (§45), first creatures and the events' creatures (§46), the day's order, views, versions, ids
  (TWN-GRO-001 to -007; TWN-EVT-004, TWN-VEW-004, GAM-CAL-002, -003 amended), quirks
  (`TWN-QUIRK-8` to `-10` planned), gates, batch TWN-3's growth rows and the open points for T22b
  (§47).
- **The agent's calls** (decision 78), for `TWN-Q14`.

**The queue:** T22 out, T22b and T23 in. The adventure lane's inbox told the growth's planned
`cross-lane` changes in its paths (the week's kind, GAM-CAL-002 and -003, the setup's `roe`,
`h3-heroes`' derivations, the hashes and the pack format), for T108b's versions. No version moved.
Ids: questions next 14, decisions next 79.

**Verify:** docs only: `scripts/verify-once.sh engine --base origin/master` ran the light gates
(no fingerprinted path changed: `check-rules`, `check-stubs`, `check-docs`, `check-exe-map`, all
passing), `cargo xtask check-lane` ok; CI on the pull request is the record.

### T22b — recruitment's and the rows' drafts, the cut, the exit criteria (2026-11-11)

Decision 74's second half, kept whole at its start (the user, decision 79; the agent recommended
the design first, then the cut and the checkpoint). The army reference it waited on was settled
by the adventure lane's T108b while T22b started (#856, E08 decision 305: T21's proposal as
written, `ArmyRef::MapGarrison` its sibling, one owner rule, the `Exchange` decision; its note in
this lane's inbox). Branch `town/e09-t22b`, docs only (the drafts, the design note, the epic, the
log, the answered questions, the lane's queue and HANDOFF, the adventure lane's inbox). Starting
point: `master` at 73e5e91 (#855, #856 merged; CI green on `master`'s last code); the light gates
passed before any change. The user's computer was connected at the start only: the executable,
`maps.tar` and `extracted.tar` staged then (for the code reading below).

**Read for it** (the address map's entries and two slices): `next_player_turn` 0x4C73AA to
0x4C74AE sets the points of the player's heroes, its two tavern heroes, then each of its towns'
garrison heroes (+0xC); `new_day` and the week's Stables loop reach every hero record, so a
garrison hero's marks drop and its spell points return as any hero's; `town_visitor_to_garrison`
takes the hero off its owner's list (the economy survey's §14 open point: nothing counts twice).

**Written:**
- **The drafts**, now whole and accepted: `0028-increment-3.md` (recruitment's rules: the source,
  the stock, the purchase; the seam across the store, the adventure and the treasury;
  `TownCommand::Recruit`), `0017-increment-3.md` (the commands, the garrison heroes at the turn's
  start and in the income, the hire dialog's town sources, the rows without a decision, the env's
  two entries' groups, the versions, the purchase's application, the battles beside and of a
  garrison hero; the Portal at a capture), `0016-increment-3.md` (the garrison hero's record in
  the town, its moves, the references and the owner rule, the hire's sources and target, the visit
  of a town with a garrison hero, T-5, the views), `0013-increment-3.md` (status only), the new
  `0023-increment-3.md` (the battle beside a garrison hero not a siege; the garrison hero's town
  bonuses).
- **The design note** Part E §48 to §58: scope and sources (§48), the command and the dialog (§49),
  the purchase (§50), the Portal's hire and a capture's empty Portal (§51), the town's two bars
  (§52), the garrison hero (§53), what it changes (§54), the views, versions, ids (TWN-REC-001,
  -002, TWN-ROW-001 to -005; amended TWN-SIE-001, -002, TWN-CAP-003, TWN-VEW-001, TWN-VIS-002,
  GAM-DEC-008, GAM-MAN-001, -002, GAM-BAT-001, GAM-RES-003, -005, ADV-MOV-002), `TWN-QUIRK-11`
  (T-5) and gates (§55), the building tasks T23 to T30 with the order against E08's increment 12
  (§56), the exit criteria (§57), the risks (§58).
- **The checkpoint** `TWN-Q14` (four questions; decisions 80–83, each as recommended): the rows
  without a decision, the garrison hero's record in the town, a capture's empty Portal drawn, the
  cut, the exit criteria and the agent's calls as written.

**The queue:** T22b out; T23 next, T24 to T30 queued. The adventure lane's inbox told the rows'
and the garrison hero's plan in its paths (for its T114a and T115a). No version moved. Ids:
questions next 15, decisions next 84, quirks planned `TWN-QUIRK-8` to `-11`, rules planned
TWN-GRO-001 to -007, TWN-REC-001, -002, TWN-ROW-001 to -005.

**Verify:** docs only: `scripts/verify-once.sh engine --base origin/master` ran the light gates
(`check-rules`, `check-stubs`, `check-docs`, `check-exe-map`), `cargo xtask check-lane` ok; CI on
the pull request is the record.

### T23 — batch TWN-3: the specs, the harness's draws and town screen actions, the runs, the scoring (2026-11-11)

One session, the user present at the start; branch `town/e09-t23`, `cross-lane` (the user's consent
at the start: `docs/experiments/README.md` is the battle lane's; the capture harness is common).
Starting point: `master` at 5b2e761 (#858 merged; the setup's baseline lost to a VM restart, CI's
run on `master` stands for it). The user's computer was connected at the start only: the original
data staged then (20 files, the four LODs one per call). **Kept whole** (decision 84; the agent
recommended three parts). Found at the start: §14's plan to write the month's kind before the month
was void (`new_month` draws it into 0x698834 and 0x697748 and reads them straight after), so the
harness forces the draws.

- **The harness** (`clients/capture/`, common): xdd.dll's sites gain the week's chance and
  creature (0x4C87CE, 0x4C883E) and the month's kind and creature (0x4C8F5E, 0x4C8FA5), forced, and
  logs the neutral garrisons' (0x4C8E09, 0x4C8E1C, 0x4BF5E5, 0x4BF5F0, 0x4BF5FB, 0x4BF74D) and the
  Portal's draws (0x5BD825, 0x5BD86E) (`h3drive.ROLLS`, `LOGGED`; the address map's twelve
  `*_draw` sites); `h3adv` gains a recipe's and a step's `rolls` and `roll`, a step's `date`, the
  reads `draws[:<prefix>]`, `turn_draws[:<prefix>]`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero` (`h3mem`'s town record +0x16, +0x3C, +0x40, +0x118),
  `hire:creatures`, and the town screen actions `hero_in` (the two portraits, each click after the
  hover ran: a click right after one move was lost) and `building = <id>` (a dwelling's hire); the
  walk closes a hire dialog left open. README: "Forced rolls" and "Growth, the garrison hero and a
  town's hire". xdd.dll redirected all 30 calls; the harness check passed (its first run stopped
  at the known `attack` flake, the adventure lane's TRAPS).
- **The specs and recipes:** `experiments/specs/E09-batch-3a.toml` (stamp `d5f72a08`; rows 1 to 5:
  six towns on row 7, six neutral ones on row 15, three dwellings flagged on day 1, Blue's town so
  that Blue stays in the game; eight weeks by the date written to each week's last day, the second
  month the Griffin's, the third the Plague) and `-3b.toml` (stamp `a20d41e4`; row 6: T with the
  upgraded Guardhouse, U with four stacks); the recipes `clients/capture/batches/E09-batch-3a.toml`
  (38 steps, about 8 minutes) and `-3b.toml` (5 steps, about 3 minutes).
- **The runs:** two of each, recorded (`experiments/results/E09-batch-3a.md`, `-3a-run2.md`,
  `-3b.md`, `-3b-run2.md`). Probes (not recorded): the town screen's items; a 3a run whose map's
  horde was the level-1 dwelling's ("horde 1"; the Castle's Griffin Bastion is "horde 3") and whose
  Blue, without a town, was out of the game after seven days; a 3b run whose portrait clicks were
  lost (the purchase then into the town's own garrison).
- **The scoring** (survey §17): every row as the reading in every read of both runs: TD-13 (only
  the exact creature doubles: K1 47, K2 94), §1's sum (80, 57, 43), TD-14 (O1 53, 30; N1 17, 10),
  TD-15 (a replay of §4 over the logged draws predicted all 56 reads of each run), TD-16 and T-11
  (the Portal's first creature at the screen, the count set, no Plague), row 6 (the purchase into
  the garrison hero's army, one count for two creatures, the move refused with GENRLTXT 276, the
  points refilled). Not predicted, nothing for the engine: the Portal's stale count while it has no
  creature, the town's hire dialog closing after a Recruit. No question; decisions 69 to 83 stand.

**The queue:** T23 out; T24 next. The client, map-client and adventure lanes' inboxes told the
harness's change (xdd.dll's sites: a stale `xdd.dll` fails the sequences' count of redirected
calls; rebuild it with `setup.sh`). No version moved. Ids: questions next 15, decisions next 85.

**Verify:** no engine code: `scripts/verify-once.sh engine --base origin/master` (the light gates),
`cargo xtask check-lane --cross-lane`; CI on the pull request is the record.

### T24 — `shared/`: increment 3's ADRs, the towns' stream, the Legion's effects and the town flags (2026-11-12)

One session, the user present at the start; branch `shared/e09-t24`, `cross-lane` (decision 83's
plan: the packs and the harvester are the battle lane's, the golden replays the battle and
adventure lanes', the bot reports the AI lane's, one test literal the server lane's). Starting
point: `master` at 8679604 (#860, T23's, merged; CI's green run on it stands for the setup's
baseline, which a shallow clone could not match to CI's fingerprint). The user's computer was
connected at the start only: the original data staged then (21 files, the four LODs one per call).
**Kept whole** (decision 85; the agent recommended two parts, code first).

- **`h3-kernel`:** `StreamId::TOWNS = Self(7)` in the closed list (decision 75), the two tests'
  named lists extended.
- **`h3-content`:** `HeroEffect::TownGrowth { level, amount }` (code 61, `town_growth`; level
  1..=`TOWN_GROWTH_LEVELS` (7), amount ≥ 1, one per level in a list) and
  `HeroEffect::OwnerTownGrowth { bp }` (62, `owner_town_growth`; 1..=10 000) with `try_map`, the
  parameter errors, the collision, the hash and the every-kind tests; `quirk.town.creature_month_exact`
  (`QuirkFlag(39)`, `CREATURE_MONTH_EXACT`, T-13, planned `TWN-QUIRK-10`) and
  `quirk.town.neutral_plague_full_growth` (40, `NEUTRAL_PLAGUE_FULL_GROWTH`, T-12, planned
  `TWN-QUIRK-9`), read from T25b; **core ruleset hash v53** (golden base `0x9649_783d_4639_ded0`).
- **`h3-pack`:** **pack format v52** (the two kinds); the tests
  `the_legion_growth_parses_and_is_harvested` (the syntax, the refusals, the five pieces by level
  in every expansion, the SoD Statue's share alone, its five parts) and
  `the_harvested_flags_of_e09_increment_3` (on in `faithful`, off in `strict-fixed`).
- **The harvester** (battle-owned): three sources (`exe-legion-growth`: 0x5BFB60, 0x5BF900,
  0x4BACB0, 0x4D91F0 and batch TWN-3's TWN-3-1, -2; `vcmi-legion-growth`: VCMI 1.7.5's
  `config/artifacts.json`, as survey §11 read it; `wiki-legion-growth`: the growth page's shoutwiki
  copy, search snippets), the six artifacts' effects, the pending list down to the Grail; both
  profiles name the flags. `base` and `sod` re-harvested from the staged installation: the five
  pieces' and the Statue's lines (their "pending" comments gone) and the profiles' lines only.
  `synthetic-ext`'s Storm Regalia carries both kinds (the every-kind check: the Wayfarer Sash's
  list is full); `packs/README.md`'s two rows.
- **Replays and records:** the 82 golden replays (10 battle, 72 adventure) keep commands and state
  hashes; only `ruleset_hash` moved (synthetic faithful `0x876dc0d92f71fae2`, base faithful
  `0x437388440d9286a3`, strict-fixed `0xb0c378a75ff70a5b`; read from a recording of each with the
  CLI, every replay then verified with `h3 verify` and `h3 adventure verify`). The AI lane's
  `bots-report-v1.json` and `bots-neutral-v1.json` regenerated: only `engine.core` moved.
  `h3-server`'s tamper test's `ruleset_hash` line (the server lane's).
- **The ADRs:** ADR-0013 point 16.4; ADR-0016 points 1 (a neutral garrison's week, growth's inputs,
  the garrison hero's record), 3 (the garrison hero, the army references, the hire dialog's town
  sources and target; at the point's end, before E08's increment 12 text once T112 adds it), 4 (a
  town with a garrison hero, T-5), 6 (its views); ADR-0017 points 1 (the commands, the week's
  kind, the towns' stream, the setup's RoE flag), 2 (the views), 3 (the day's order; the garrison
  heroes' turn start, income and weekly resources: the draft's anchor sentence is not in the
  ADR, so after the town events), 4 (the towns' stream and the Portal's draws), 6
  (the hire dialog's town source, the rows without a decision), 8, 9, 10, 11; ADR-0023 points 1
  and 3 (the status line, the ADR has no history section); ADR-0028 points 2, 5, 6, 9, the
  Consequences and the header's amendments; each history entry; the index's status and Read-when
  cells. Each with the batch row that confirmed it (survey §17); a throwaway script found every
  draft sentence in its target but the reworded ones (the decisions' references, the verdicts
  added, the anchors). The drafts deleted; the design note's §37 and §48 heads and §39's *As
  built*.
- **Also:** `docs/03-domain.md`'s terms (available creatures, the week's kind, the Portal of
  Summoning, the garrison hero, the upper bar); `docs/06-security.md`'s section "A town's growth,
  recruitment and the army rows (E09 increment 3)"; the battle and adventure lanes' HANDOFF
  version lines; the adventure lane's inbox (T112's merge order).

**The queue:** T24 out; T25 queued in full (design §56 round 4). Ids: decisions next 86.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`; CI on the pull request is the
record.

### T25a — the town part v5, the store v3, a dwelling's growth and first creatures (2026-11-12)

One session, the user present at the start; branch `town/e09-t25a`, `cross-lane` (`h3-harvest`
and the `ab` and `base` packs, the battle lane's; `GAME_STATE_HASH_VERSION`, `h3-game`'s
`src/tests/mod.rs` literal and the 72 adventure replays, the adventure lane's). Starting point:
`master` at 4a70238 (#864, T24's, and #865 merged; the setup's baseline found CI's fingerprint
on fa98206, the light gates). The user's computer was connected at the start only: the original
data staged then (17 files: the four LODs one per call, the executables, the five DLLs, both
`.snd`, both videos, both tars); the harvest and the scoped verify used them. **T25a kept whole**
(decision 86; the agent recommended T25a in two, the part v5 with its harvest first).

- **The town part v5** (`h3-towns`' `ruleset.rs`, `compile.rs`; design §38): the roles `horde:
  Option<Horde { level, upgraded }>`, `portal`, `week_of: Option<u8>`; `TownDef::roe` and
  `TownRecord::roe`; `TownParams::week: WeekParams { creature_chance, creature_bonus }` and
  `garrison_week: GarrisonWeekParams { odds, out_of, weights, weeks, upgrade }` (in the Def too).
  Refusals: a horde's or a week's level outside the record's, an upgraded horde without the
  level's upgrade, two hordes of one index, a second Portal or week's building, a week's chance
  not `1 <= n <= d`, odds above `out_of` or `out_of` 0, weights not one to six, an upgrade chance
  above 100. `TOWN_RULESET_HASH_VERSION` 5 (pinned `0x84b4_0449_24d2_2388`, the empty part
  `0x04f0_d766_3d89_5b60`); every new field moves it (a test each).
- **`h3-pack-towns`:** the format's `horde`, `portal`, `week_of`, `roe` (default true),
  `week_creature_chance`, `week_creature_bonus`, `garrison_week` (the doc and the roles' table);
  the synthetic pack's parameters (invented: 1 in 3, 4, 30 and 70 in 100, three weights), its
  horde `[2, false]`, the Lookout's Portal, the Grail's `week_of = 1` (`horde_up` without a
  role: `vale` has no upgrades; no new building). Tests: the roles and parameters parse, `roe =
  false`, the strict parser's new fields, and `the_original_growth_roles` (the eleven hordes and
  their upgrades at their levels with their creatures' horde growth 3, 4, 2, 4, 8, 3, 6, 7, 8, 6,
  10; the Dungeon's Portal and the Inferno's Deity of Fire alone; every record RoE's but
  Conflux's; the parameters).
- **The harvest** (battle-owned, design §38): each horde's `horde` role from the horde table (its
  level by its creature's, the upgrades' buildings 19 and 25 upgraded), its record's growth
  checked equal to its creature's `CRTRAITS` horde growth (a new refusal; all match); the
  gap-fill's `portal` (Dungeon 22) and `week_of = 1` (Inferno 26) with the new source
  `exe-town-growth`, `horde` refused from the gap-fill; `roe = false` on the `ab` pack's records;
  `[towns.growth]` written into `[town_params]`. The report's line now says the growth is checked.
  `packs/base` and `packs/ab` re-harvested from the staged installation: the 22 horde roles, the
  Portal, the Deity, Conflux's `roe`, the three parameter lines only.
- **The store v3** (`record.rs`, `store.rs`; design §40): `CREATURE_INDEXES` (14),
  `MAX_AVAILABLE` (32 767), `Summoned { creature, count }`; `Town::available`, `available_all`,
  `portal`, `with_available`, `with_portal` (saturating: `TWN-QUIRK-8`, T-10 fixed); the
  store's `set_available`, `set_portal`, `write(town, Town)` (each copies only its town;
  `false` for a town the store lacks or an index past the fourteen), the closure write now
  `update`. `TOWN_STATE_HASH_VERSION` 3 (pinned `0xFA14_A109_A1EC_2745`; each count and the
  Portal's creature and count move it).
- **TWN-GRO-001** (`growth.rs`, design §41): `growth(town, part, core, index, &GrowthInputs)` and
  `index_creature`, `GrowthInputs { owned, share_bp, legion, dwellings }` with `NEUTRAL`; the
  fixture's core gains 52 creatures (the Keep's levels growing 14, 9, 7, 4, its level-1 horde 8,
  the Marsh's 6), the Keep's hordes their roles, its shrine the Portal, its Grail `week_of = 1`,
  the Marsh `roe = false`. Tests: the dwelling and its upgrade, the castle's and the citadel's
  (truncated), the Statue's share and the Legion only for an owned town, the horde at its index,
  the map's dwellings at a growing index only, the Grail's half, batch TWN-3's row 2 (66 and 48),
  the bound, a town without a record or a creature the core lacks.
- **TWN-GRO-005** (design §46, §40): the first creatures (crate-private `first_creatures`): the
  base dwellings first set their index to their creature's growth, then the upgraded ones move
  the base count; `Towns::build` (now with the core) applies them to the buildings it makes ever
  built, `town_setup` (now with the core) to the start's ever-built set. Tests in `growth`,
  the store, the setup (a custom set's upgraded dwelling: 0 and 14) and the game (`h3-game`'s
  `a_built_dwelling_gives_its_first_creatures_in_the_game`; the town tests' imp and gog grow 15
  and 8).
- **Catalog and ledger:** TWN-GRO-001 and -005 `implemented` (code reading, batch TWN-3 row 2,
  VCMI 1.7.5, the design); `TWN-QUIRK-8` in `quirks-town.md` (its regression test
  `twn_quirk_8_a_town_s_counts_saturate_at_the_bound`).
- **`cross-lane`:** `GAME_STATE_HASH_VERSION` 28 (the store's v3 moves every game hash: design
  §47 planned v28 for T26a, whose week's kind becomes v29); the 72 adventure replays re-recorded
  from their own commands (a scratch script, 3 s in release): only `town_ruleset_hash`
  (`0x8477f2008d862e55` to `0x4ded73a0d357e245`), `game_state_hash_version`, the checkpoints and
  `final_hash` moved; each verified with `h3 adventure verify`. The battle replays and the bot
  reports record neither hash: untouched. `h3-server`'s record tests' two literals (the server
  lane's: `army-dialog-s1`'s checkpoint and final hash, found by `cargo test --workspace`; the
  scoped verify does not run them). `h3-game`'s two test literals (one the adventure lane's
  `src/tests/mod.rs`) gained the parameters and `roe`; its town modules pass the core to the
  store's build and the setup. The adventure lane's inbox and HANDOFF versions line told.
- **Agent's calls (change freely):** decision 86's list; the refusals' wording; the parameters'
  integer types; `growth` and `index_creature` public, `first_creatures` crate-private.

**The queue:** T25a out of T25's entry; T25b next (unchanged scope).

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` (every step but
`check-lane`, by design, and coverage's one region, fixed and run again alone; the bench gate
passed), `cargo xtask check-lane --cross-lane`, `cargo test --workspace --exclude xtask
--all-targets`; `master` merged in (#868, #869), `h3-protocol`, `h3-server` and `h3-game` again;
CI on the pull request is the record.

### T25b1 — the week's kind, a town's week and month of growth (2026-11-12)

One session, the user present at the start; branch `town/e09-t25b1`, the town lane's paths only
(`h3-towns`, the catalog's `town.toml`, `quirks-town.md`, E09's docs). Starting point: `master`
at a396c3f (#873, T25a's, merged; the setup's baseline found CI's fingerprint, the light gates).
The user's computer was connected at the start only: the original data staged then (17 files, as
T25a's), used for the scoped verify. **T25b in two** (decision 87, recommended): T25b1 the week
and the month here, T25b2 the garrison week and the Portal next.

- **`WeekKind`** (`h3-towns`' new `week.rs`; design §42): `Normal` (the default), `Creature(c)`,
  `Of { creatures: [Option<c>; 2], bonus }`; `names(c)` (a creature's week its creature alone, a
  building's either of its level's) and `bonus(part)` (the part's `creature_bonus`, the
  building's own, none in a normal week or without parameters); its `CanonicalHash` (0; 1 and
  the creature; 2, both options and the bonus), read by the game state hash from T26a.
- **TWN-GRO-003** (`week_growth(town, part, core, &kind, &inputs) -> Town`): each index whose
  growth (TWN-GRO-001) is above 0 gains it, + the bonus for a named creature, halved after the
  bonus without an owner, held at the bound; the other indexes and the Portal kept; a faction
  without a record unchanged.
- **TWN-GRO-004** (`month(town, part, core, MonthGrowth, &week, &inputs) -> Town`; design §43):
  `MonthGrowth { Normal, Creature(c), Plague }`, the town crate's (the game's `MonthKind` stays
  in `h3-game`; T26a maps it). A creature's month doubles (held at the bound) the growing
  indexes whose creature is the month's, with `quirk.town.creature_month_exact` off either of
  the level's; nothing in a `week_of` week. The Plague: the count less the growth, at least 0,
  halved; with `quirk.town.neutral_plague_full_growth` off a town without an owner loses half the
  growth. The flags read from `core.quirks()`.
- **Tests** (`src/week/tests.rs`, ten): the kind's names, bonus and hash; the week owned and
  neutral, with the Statue's inputs, a creature's and a building's week (the bonus before the
  halving), the bound, a town without a record; the month: batch TWN-3's row 1 (K1 47 kept, K2
  94) and row 3 (O1 53 and 30, N1 17 and 10, after the week computed by `week_growth` from 106,
  61, 42, 25: the fixture's growths 14 and 9 are the rows' towns'), both flags off (K1 94; N1 21
  and 12), a `week_of` week and a normal month, the bound, the Plague's floor and an index that
  grows nothing. The fixture's `core_with(&[flags])`.
- **Catalog and ledger:** TWN-GRO-003 and -004 `implemented` (code reading, batch TWN-3 rows 1
  to 3, VCMI 1.7.5, the design); `TWN-QUIRK-9` and `-10` (`fix-optional`, faithful on,
  strict-fixed off) in `quirks-town.md`; `TWN-QUIRK-8`'s regression tests gain the week's bound.
  The `(planned)` marks of the two quirks in `h3-content`'s flag docs, the harvester's profile
  comment and the packs' `core.toml` stay (shared and battle-owned paths): for T26a's `shared/`
  or `cross-lane` change.
- **No version moved** (the game calls these from T26a); the crate's doc names increment 3.
- **Agent's calls (change freely):** decision 87's list.

**The queue:** T25b's entry is T25b2 (TWN-GRO-006, -007), next.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T25b2 — a neutral town's garrison week and the Portal's draw (2026-11-12)

One session, the user present at the start; branch `town/e09-t25b2`, the town lane's paths only
(`h3-towns`, the catalog's `town.toml`, E09's docs). Starting point: `master` at 21d6ba0 (#882
merged; T25b1's #879 at 17:05 before it); the setup's local baseline was stopped once `master`
moved past it twice, CI's run on `master` stands for it. The user's computer was connected at
the start only: the original data staged then (17 files, as T25b1's), used for the scoped verify.
**T25b2 kept whole** (decision 88, recommended).

- **TWN-GRO-006** (`h3-towns`' new `garrison.rs`; design §44, survey §4): `garrison_week(town,
  part, core, weeks, garrison, rules, draws) -> Option<GarrisonWeek { stacks, town }>`: the
  chance `below(out_of)` under the odds by a standing fort level, n = min(`weeks`, the
  parameters'), three `range_inclusive(0, n + 1)` summed, the level by the weights (a private
  `level`), the base creature or the upgrade the garrison holds, its core growth; the room (a
  stack of the creature within a stack's most, a free slot of `rules`' slots, else the least AI
  value × count below the new stack's cleared, the first in slot order, the new stack in its
  slot; else none), the level's two counts lowered (at least 0), the upgrade draw for a base
  creature only turning every stack of it. None with no draw without parameters or a record;
  none after the sums for a level the record lacks, a creature growing none or a growth above a
  stack's most.
- **TWN-GRO-007** (`h3-towns`' new `portal.rs`; design §45, survey §5): `summon(town, part,
  core, offer, draws) -> Town`: no standing `portal` building unchanged (no draw); no dwelling
  empties the Portal (no draw); else a dwelling `below(n)`, one of its creatures `below(k)`, the
  count its core growth, set; a dwelling without creatures keeps the Portal.
- **Tests** (`src/garrison/tests.rs` nine, `src/portal/tests.rs` four): the weights' table (the
  survey's S ranges, the most S by n), the odds with and without a fort (forced, and the
  original's 40 and 80 against the chance draw over 40 seeds), the first empty slot and the
  counts, the held upgrade with no upgrade draw, the upgrade of every base stack (forced both
  ways, the original's 5 against its draw over 200 seeds, a level without an upgrade), batch
  TWN-3 row 4's room (the least cleared, the first of equal ones; no stack below the new one:
  nothing, no draw after the sums; a held stack merges), the army's slots and a stack's most, the
  weeks past the parameters' 8, the rules missing; the Portal without its building, without a
  dwelling, the draw over 30 seeds (each creature drawn, set not added, with a castle, a horde and
  a Grail standing), a dwelling without creatures, a creature the core lacks. Each test checks
  the stream after against a reference of the draws expected. The fixture's core gains AI values
  (ten times a handle) and `part_changed` (the parameters and the Keep at once). Coverage of both
  modules 100 % (lines and regions).
- **Catalog:** TWN-GRO-006 and -007 `implemented` (the code reading, batch TWN-3 rows 4 and 5,
  VCMI 1.7.5 refuted, the Heroes 3 wiki's Dungeon page and the RoE manual for the Portal, the
  design). The design note's §44 and §45 carry *As built (T25b2)* lines.
- **No version moved** (the game calls them from T26b); the crate's doc names them.
- **Agent's calls (change freely):** decision 88's list.

**The queue:** T26 queued in full (T26a, T26b; its cut confirmed at its start).

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T26a1 — the towns' growth in the day (2026-11-12)

One session, the user present at the start; branch `town/e09-t26a1`, `cross-lane` (the adventure
lane's `h3-heroes`, `h3-game`'s `state/day.rs`, the catalog's `game.toml`, nine adventure
replays, comments in `h3-map`'s and `h3-cli`'s files; the battle lane's `packs/base/core.toml` and
harvester comment), with the `shared/` pull request `shared/e09-t26a1-quirk-marks` (#886: the
quirk flags' doc comments in `h3-content`, a test comment in `h3-pack`). Starting point: `master`
at 4724477 (#883 merged; the setup's baseline found CI's fingerprint, the light gates). The
user's computer was connected at the start only: the original data staged then (17 files, as
T25b2's), used for the scoped verify. **T26a in two** (decision 89, recommended): this part the
growth with every week a normal one; T26a2 the week's kind.

- **The Legion's effects** (`h3-heroes`' `day.rs`; ADR-0013 point 16.4): `town_growth(details,
  core) -> [u16; 7]` (each level's `town_growth` summed over the hero's sources, each worn type
  once, a combination's parts counting, saturating) and `owner_town_growth(details, core) -> u16`
  (the highest `owner_town_growth`, 0 without). A test on the day pack: a piece, two copies of a
  type, two types at a level, a combination with its part worn beside it and a part needing a
  skill, the saturation, a specialty's share.
- **The inputs** (TWN-GRO-001; `h3-game`'s new `town/growth.rs`, design §41): `inputs(state,
  rules) -> Vec<GrowthInputs>` by `TownId`: a neutral town's `NEUTRAL`; an owned town's owner's
  share (the highest over its heroes on the map), the visitor's Legion, per index the owner's
  dwellings on the map whose kind's first creature is the index's (a base one at its upgrade's
  index too, only where the level has an upgrade), a per-player table built at its first town.
- **The week and the month in the day** (TWN-GRO-003, -004; GAM-CAL-002): `grow_week` after
  `WeekStarted` and before the adventure's week step, `grow_month` after `MonthStarted` and before
  the adventure's month step with the week's inputs (`MonthKind` mapped to `MonthGrowth`, a
  normal month writing nothing), each town written only where it changes; `advance_day`'s month
  block now inside its week block. Every week `WeekKind::Normal` (TWN-GRO-002 is T26a2's).
- **The events' creatures** (TWN-EVT-004; design §46): `h3_towns::event_creatures(town, part,
  &creatures)` (`growth.rs`: the upgraded dwelling's index when it is ever built, else the
  dwelling's, else lost, held at 32 767, `TWN-QUIRK-8`), applied by `town/events.rs` after the
  event's buildings; no event tells it (the counts join the owner's view with T26b).
  `TWN-STUB-10` lifted (`docs/stubs.toml`; the comments in `h3-map`'s plans, its census example's
  label and `h3-cli`'s replay doc).
- **Tests:** `h3-towns`' event creatures (no dwelling, a dwelling, the upgraded one, the bound,
  a faction without a record); `h3-game`'s `town/tests/growth.rs` on four towns (red's with its
  visitor wearing the Legs and the Statue, blue's, red's without a visitor, a neutral one) and
  three huts (red's and two neutral ones): the inputs whole; the week on days 8, 15, 22 and 29
  (43 and 20, 30, 38, 22 on day 8), nothing between; a gogs' month (T0's gogs 56 to 112) and the
  Plague (each count less its growth, halved; the neutral town less half, the profile's flag
  off); a town event's imps added on day 2 and its gogs lost. The town tests' `rules_with_adventure`
  (an adventure pack edited before it compiles).
- **The quirk marks:** `TWN-QUIRK-9` and `-10` no longer "planned" in `h3-content`'s flag docs and
  `h3-pack`'s test (#886, `shared/`) and in the harvester's profile comment and the base pack's
  `core.toml` (this pull request; comments only, no hash moved).
- **Catalog:** TWN-GRO-001, -003, -004 and TWN-EVT-004 say where the game runs them; GAM-CAL-002
  the towns' week and month in the order. The design note's §41 to §43 and §46 carry *As built
  (T26a1)* lines.
- **No version moved.** Nine adventure replays re-recorded (`builds-events-s1`, `builds-s262-d14`,
  `-s271-d14`, `-s290-d14`, `builds-week-s1`, `towns-income-s1`, `towns-s173-d21`, `-s1702-d21`,
  `-s3435-d21`): only checkpoints and final hashes, each first differing at day 8's start; the
  other 63 unchanged. The bot reports unchanged. `master` merged in after E08's T113b (map
  import v22) and the server's T9b: the nine taken from `master` and re-recorded again, the same
  nine moving the same way; `h3-protocol`'s and `h3-server`'s golden-replay tests pass.
- **Bench** (callgrind, `master` built in the same sandbox): the towns' week is new work in the
  two town day-start workloads. Its first build, one `growth` call per index (each reading the
  town's buildings again), cost `day-start-builds-36` +199 % and `day-start-towns-36` +89 %. So
  `h3-towns` gained `TownGrowth` (crate-private: one pass over a town's ever-built roles, its fort
  level once, then `at(index)`; `growth` and the week's and month's loops read it), and
  `event_creatures` reads the dwellings once and returns at once without creatures. In the game,
  an owner without dwellings skips the index table. After that, `day-start-builds-36` is 567 462
  Ir against `master`'s 361 745 (+56.9 %: `grow_week` 23 %, of it `TownGrowth::of` 9 %, `inputs`
  11 %, the visitors' lookup 4 %) and `day-start-towns-36` is 1 206 371 against 854 709 (+41.1 %).
  Both were re-baselined with the reason in their descriptions. Every other workload passes its
  gate (`random-play-*`, `day-start-xl-144`, `day-start-dwellings-xl-144`, `month-start-xl-144`,
  the forks).
- **The plans' property** (`h3-map`'s `play.rs`, `check_towns`, the adventure lane's file): a
  town's record changing without a capture, a build or the day's start failed it once the counts
  grew. Now the counts may move only with a week's start, a build or a town event of the town
  (told by nothing until T26b), and the rest of the record is compared as before.
- **Agent's calls (change freely):** decision 89's list.

**The queue:** T26a2 and T26b queued (T26a2 next, its cut confirmed at its start).

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T26a2 — the week's kind (2026-11-12)

One session, the user present at the start; branch `town/e09-t26a2`, `cross-lane` (the adventure
lane's `h3-game` state, day, command and observation files, `h3-map`'s import and tests, the 72
adventure replays and four fork workloads; the server lane's `h3-protocol` and `h3-server`; a
client test literal in `h3-view`). Starting point: `master` at 7f9107f (#892, #893, #894 merged);
the setup's full baseline was stopped in its coverage step for the disk (12 GB in
`target/llvm-cov-target`, 13 GB left): CI's run on `master` stands for it. The user's computer was
connected at the start only: the original data staged then (17 files, SHA-256 checked against the
PC's), used for the scoped verify. **T26a2 whole** (decision 90, the user; the engine-now cut was
recommended).

- **The inbox's 2026-11-11 hygiene note, classified first** (the user's choice): the property
  plan's fault, no leak. A neutral plan's town visited at load is its visitor's owner's
  (`towns_laid_out`), so `hygienic_town`, which made only an owned plan's standard set with a Fort
  the one without, let it draw its level-2 dwelling (TWN-SET-003) that its new owner sees, and the
  other game's seed drew another. `hygienic_town` counts a visited town as owned; the case's
  `cc` line is committed (`h3-map`'s `proptest-regressions/tests/view.txt`). The adventure lane's
  file, in this pull request.
- **The draw** (TWN-GRO-002; `h3-towns`' `week.rs`, design §42): `draw_week(towns, part, core,
  roe, month_first, draws)`: unless a month's first, `below(denominator) < numerator` of the
  part's `creature_chance`, then `below(n)` of `week_creatures(part, roe)`; then the first town in
  `TownId` order whose `building_week` is some overwrites it. The part derives the two creature
  lists and the records' `week_of` buildings when it is built (`WeekDraw`, not hashed): a walk
  each week cost `day-start-builds-36` +7.8 % against `master`.
- **The game** (`state.rs`, `state/day.rs`): `GameState`'s `week`, `roe` and `towns_rng`
  (`StreamId::TOWNS`), game state hash **v30** (after E08 T114a2's v29, merged before this pull request; the week and `roe` after the month, the stream
  after the game's); the draw before `WeekStarted`, its kind into `grow_week` and `grow_month`;
  `draw_month` after a building's week gives its base creature's month, the kind's draw made,
  the creature's not (GAM-CAL-003). `GameSetup::roe` (every literal in `h3-game`'s, `h3-map`'s
  and `h3-cli`'s files) from the import's format, map import **v23**. `h3-game` re-exports
  `WeekKind`.
- **The telling:** `GameEvent::WeekStarted { week, kind }`, `GameObservation::week`; the wire's
  `Week` (`Normal`, `Creature`, `Of`), the mapping, the fold (`self.week = kind`), protocol
  **v53** (the server lane's E16 M5 T11 and T12 and E08 T114a2 took v50 to v52, merged during this
  task: `master` merged in three times, the replays re-recorded again after T114a2's);
  the fold gate's part "day, week and month".
- **Tests:** `h3-towns`: the lists, `building_week`, the draw's chance and stream positions over
  400 seeds, no draw on a month's first, `roe`, the building's week over the draw. `h3-game`'s
  `town/tests/growth.rs`: a creature's week drawn on day 8 as the towns' stream of seed 7 says,
  told to both players and grown (+5, halved in the neutral town), the map's huts at their growth
  (the adventure lane's E08 T97a note: here, as `hiring.rs`' rules have no town part); the stream's
  isolation (eight seeds: the month and the adventure's state the same whether the weeks draw a
  creature); a Grail as level 1's `week_of`: every week the imps' (+15), day 29's month theirs by
  a normal month's odds and by a creature month's, nothing doubled, their monsters on the map.
  `h3-map`: the import's `roe`; the hygiene property ends its comparison at a week's start whose
  kinds differ (the other game's seed), `check_told` follows the observation's week.
  `h3-protocol`: every week kind mapped and inverted, the fold. The calendar tests name the kind.
- **Replays:** the 72 adventure replays re-recorded from their commands: only the version lines
  (game state hash, map import), checkpoints and final hashes moved. `h3-server`'s record test
  literals follow (`army-dialog-s1`'s hashes, the import version). The bot reports unchanged (no
  part hash moved).
- **Bench** (`master` built in the same sandbox): `day-start-builds-36` 574 701 against 572 026
  (+0.5 %), `day-start-towns-36` 1 222 360 against 1 219 378 (+0.2 %), both inside their gates.
  The forks copy the three new fields (`AdventureEnv::fork` +19 Ir, memcpy +12 Ir a fork):
  `fork-economy-xl-144`, `fork-meeting-xl-144`, `fork-sea-xl-144` +0.8 % and `fork-rewards-36`
  +1.1 % against `master`, which measures 1.3 % to 1.8 % above their old baselines here; the four
  re-baselined with the reason in their descriptions. Every other workload passes.
- **Agent's calls (change freely):** decision 90's list.
- **Not built here:** the clients' `h3-battle-client` could not be checked (its SDL build fails in
  the sandbox; it uses the import's setup with a spread, no wire week); CI's Clients run checks it.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; `h3-protocol`'s and
`h3-server`'s tests run on their own; CI on the pull request is the record.

### T26b — the neutral garrison week, the Portal's draws, the owner's counts and Portal (2026-11-13)

One session, the user present at the start; branch `town/e09-t26b`, `cross-lane` (the adventure
lane's `h3-adventure` outcome and dwelling visit, `h3-game`'s state, day and lib files, `h3-map`'s
property plans, four adventure replays and two day-start workloads; the server lane's
`h3-protocol`). Starting point: `master` at effc66d (#899 merged); the setup's baseline found CI's
fingerprint (the light gates). The user's computer was connected at the start only: the original
data staged then (17 files, SHA-256 checked against the PC's), used for the scoped verify. **T26b
whole** (decision 91, the user; two parts were recommended).

- **The neutral garrison week** (TWN-GRO-006; design §44): `h3-game`'s `town/garrison.rs`
  (`garrison_weeks`) after the adventure's week step, `weeks` the day less one over the week's
  days, each neutral town's `h3_towns::garrison_week` with the adventure part's army slots and
  most from the towns' stream; the garrison written by `AdventureState::set_town_garrison` (the
  town lane's `src/town.rs`), the town into the store. The state's `town_draws(Draws)` gives the
  adventure, the store and the stream a draw takes at once.
- **The Portal's draws** (TWN-GRO-007; design §45, decisions 71, 82): `town/portal.rs`: each week
  after the garrisons (the towns' stream); a build that makes the Portal stand (`build_told`: a
  command's from the builder's stream, a town event's from the towns' stream); the setup's after
  the levels at load (the setup stream's last draws); a visit that flags a dwelling
  (`AdventureOutcome::flagged_dwelling`, set by the adventure's `enter_dwelling`, read by
  `settle_outcome` after the capture) draws each of the player's towns' Portals that hold nothing,
  in `TownId` order, from its stream; a capture one that holds nothing, from the capturer's.
  `h3_towns::has_portal` reads the part's derived Portal set before the standing set
  (`TownRuleset`'s `portals`, not hashed), `BuildingSet::intersects`; the offer is
  `town::growth::owned_dwellings` (an iterator; the growth's inputs read its first creatures).
- **The views** (TWN-VEW-004; design §47): `TownBuildingsView::available` and `portal` (the owner's
  alone; `h3-game` re-exports `CREATURE_INDEXES`, `Summoned`); the day takes every viewer's views
  of every town before the week's growth (a recording sink only) and the towns' day tells the
  changed ones after the marks; an event's creatures with their own `Told`; a build's and a
  capture's draws inside their `Told`, a flag's with one of its own. On the wire
  `TownBuildings`' `available` and `portal` (`Summoned`), protocol **v55** (E08 T114b's #902 took
  v54 while this task ran: `master` merged in twice); the fold unchanged
  (the whole set replaced).
- **Tests:** `h3-towns`: `has_portal`, `intersects`. `h3-adventure`: the flag named in the
  outcome (`dwelling_visits.rs`), `set_town_garrison` (`towns.rs`). `h3-game`'s
  `town/tests/garrison.rs` (16 seeds: the game's garrison and counts as the pure function over a
  replica of the towns' stream, told to a Tavern's level where the creatures change; the odds'
  draw alone) and `town/tests/portal.rs` (each moment against a replica of its stream: the week,
  a command's build, an event's, the setup's as the last draws, a flag of two empty Portals in
  `TownId` order, a capture; a held Portal kept; blue's and a neutral town's never drawn); the
  growth tests' rules without the garrison week (odds 0). `h3-map`'s plans: `check_towns` lets
  the Portal move only with a week, a build, a capture or a dwelling's flag, and wants the
  owner's `TownChanged` for a moved count or Portal; the view model shows them; the hygiene
  property ends at every week's start (the garrisons' and Portals' draws follow the week's, at
  positions an unseen town moves). `h3-protocol`: the two fields mapped, inverted and folded.
- **Replays:** of the 72 adventure replays the four on the towns map (`towns-income-s1`,
  `towns-s173-d21`, `towns-s1702-d21`, `towns-s3435-d21`) re-recorded from their commands, only
  checkpoints and final hashes moved (a neutral garrison's week); no version moved but the
  protocol's. The bot reports unchanged (no part hash moved).
- **Bench** (`master` built in the same sandbox): `day-start-towns-36` 1 328 051 against 1 228 566
  (+8.10 %: `garrison_week` and its draws and writes), re-baselined with the reason;
  `day-start-builds-36` 587 597, +1.60 % on the baseline E08 T114b set (it was 587 773 against
  577 013, +1.86 %, before #902's merge, whose `Told::tell` returns at once for a `NoopSink`).
  First build: +6.8 % and +10.0 %, of it `has_portal`'s standing set for every owned town each
  week (cut by the part's Portal set) and the offer collected for the growth's inputs (an
  iterator again). Every other workload passes.
- **The merges:** `master` merged in twice, after the server lane's #901 (two inbox notes) and
  after E08 T114b's #902 (protocol v54 taken first: T26b takes v55; the two bench files and the
  inboxes; T114b's `Told` holds the map garrisons, so the week's views take theirs too). The 72
  replays checked again from their commands after the second: all as recorded.
- **Agent's calls (change freely):** decision 91's list.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; `h3-protocol`'s and
`h3-server`'s tests run on their own; CI on the pull request is the record.

### T27 — recruitment (2026-11-13)

One session, the user present at the start; branch `town/e09-t27`, `cross-lane` (the adventure
lane's `h3-adventure` hire types and purchase, `h3-game`'s hire decision, state hash, observation
and events, `h3-map`'s property plans, `h3-files`' replay line, the 72 adventure replays and two
town-map workloads; the server lane's `h3-protocol` and `h3-server`'s record literals; the AI
lane's `h3-sim` env, with the user's consent). Starting point: `master` at 202aa3f (#903 merged);
the setup's full baseline was lost to a VM restart, CI's green run on `master` (999bb0e, #904)
stands for it. The user's computer was connected at the start only: the original data staged
then (17 files, SHA-256 checked against the PC's), used for the scoped verify. **T27 whole**
(decision 92, the user; two parts were recommended, the env apart). `master` merged in twice,
after the server lane's #904 (protocol v56, replay format v14 taken first) and after E08 T115a's
#906 (game state hash v31, protocol v57; the replays re-recorded a second time); the server
lane's #907 (protocol v58) merged in after the pull request opened.

- **The town's rules** (`h3-towns`' `recruit.rs`; design §49 to §51): `RecruitSource::{Index,
  Portal}`; `recruit_source` (UnknownBuilding, NotRecruiting, NotStanding with a base index whose
  level's upgraded dwelling was ever built, NothingSummoned for an empty Portal), `recruiting`
  (the legal set, one pass over the standing buildings), `recruit_stock` (an upgraded index's two
  creatures from one count, the upgrade first), `check_recruit` (`RecruitError::{NotOffered,
  Amount}`), `recruit` (saturating; the Portal's creature kept). `TownCommand::Recruit` and the
  three refusals in `command.rs`.
- **The adventure's side** (E08 T95b's reuse): `HireTarget::Garrison(TownId)`,
  `HireSource::{Town { town, index }, Portal { town }}`; `hire_stock` none for a town's source,
  `hire_room` over a garrison; `check_join` (the target's room and the price) under `check_hire`;
  `HirePurchase` names its army (`Joining`) and the map's count to lower (none for a town's);
  `hire` returns the garrison's changed slot.
- **The game** (`town/recruit.rs`; TWN-REC-001, -002, GAM-DEC-008 amended): `open` behind
  `town::command` (the store, the owner, `recruit_source`; `HireOpened` to the owner alone),
  `buy` (the town's tests, the room, the treasury, then the price, the store's write, the join,
  `StackChanged` of the garrison's slot to the owner, the town's changed views by `Told`);
  `GameState::hire` hands a town's source to it and closes a one-creature dialog after either;
  `GameState::hire_stock` public (any source; the env's). `HireSeen` replaces `HireView`'s and
  `HireOpened`'s type and anchor. The legal recruitments after each own town's builds
  (`town/offer.rs`). Game state hash **v32** (the target's tag 1, the sources' tags 1 and 2;
  E08 T115a's #906 took v31 while this task ran).
- **The wire** (protocol **v59**; #904 took v56, #906 v57, #907 v58): `TownCommand::Recruit`, three `TownReject`s,
  `HireTarget::Garrison`, `HireSeen` in `Hire` and `HireOpened`; mapped, inverted, round-tripped.
  Replay format **v15** (`town recruit <town> <building>`). The env's encoding **v10**: `RECRUIT`
  (entry 31) with `BUILD`'s tile and building heads; a dialog's creatures from
  `GameState::hire_stock` (from the adventure's stock alone, a listed town `Buy` encoded to
  nothing and the random agent fell back on `EndTurn`, which the game refused: the protocol's
  played games and the town map's env tests found it).
- **Tests:** `h3-towns` (11: every source, refusal and stock, the one count, the Portal's,
  `recruiting` against `recruit_source` over all 64 buildings); `h3-adventure` (a garrison's
  purchase, room and slot); `h3-game`'s `town/tests/recruit.rs` (7: the dialog the owner's alone
  and its answers, every refused recruitment and purchase changing nothing, a purchase's events
  and counts, the Portal bought out and still listed, an upgraded index's two creatures from one
  count and the dialog kept open, the hash naming the source, the legal order); the offer tests
  leave recruitments out. `h3-map`'s plans: `hire_stock`, `hire_most` and `check_hire` read a
  town's source and garrison (the town's told views between the slot's `StackChanged` and the
  close), `town_builds` probes each town's recruitments after its builds against `recruited`'s
  independent reading, `check_towns` lets a purchase lower one count or the Portal's, the open
  dialogs tracked by `HireSeen` in `view.rs`; `check_timed` leaves out a town event's views
  after its `TownEventFired` too (T26b's gap: an event's creatures told without a building, met
  by the scoped coverage's run; E08 T115a's #906 made the same fix, merged in; both `cc` lines
  kept); `check_legal` lets a move's battle lost within the command remove the mover (CI's case on
  #911: the defender's war machine firing first killed the mover's one stack; its `cc` line kept). `h3-sim`: the town map's play on seed 70 alone
  (a search over 1 to 399: the first seed meeting every feature, recruitments and purchases now
  among them), the fork test on seed 46 (a siege on day 1).
- **Replays:** the 72 adventure replays re-recorded from their commands (twice: after each merge),
  only the format, the hash version and the hash lines moved; `h3-server`'s record literals and
  `h3-cli`'s adventure tests' format literals. The bot reports unchanged.
- **Bench** (`master` built in the same sandbox): `legal-towns-36` 12 594 903 against 8 153 291
  (+54.48 %: the agent's recruitments, 1 755 listings for 1 172; `recruiting` with its
  `source_of` 12 %, about 850 Ir a listing) and `reach-towns-36` 1 291 602 865 against
  868 496 571 (+48.72 %: 4 031 queries for 1 850, the same code) re-baselined with the reasons; a
  single pass for the upgraded levels measured worse and was dropped. Every other workload within
  its gate (`day-start-builds-36` +1.41 %, `day-start-dwellings-xl-144` +1.63 %,
  `month-start-xl-144` +1.61 %).
- **Agent's calls (change freely):** decision 92's list.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; `h3-protocol`'s,
`h3-server`'s and `h3-sim`'s tests run on their own; CI on the pull request is the record.

### T28a — the town's two bars and the garrison hero (2026-11-14)

One session, the user present at the start; branch `town/e09-t28a`, `cross-lane` (the adventure
lane's `h3-adventure` state, army commands, hire, events and observation, `h3-game`'s management,
listing and observation, `h3-map`'s property plans, `h3-files`' replay line, the 72 adventure
replays and five workloads; `h3-harvest`'s town gap-fill; the server lane's `h3-protocol` and
`h3-server`'s record literals; the AI lane's `h3-sim` env, with the user's consent). Starting
point: `master` at 50b0935 (#911 merged; the setup found CI's fingerprint, the light gates
green). The user's computer was connected at the start only: the original data staged then (17
files and the four manuals, SHA-256 checked against the PC's). T28 cut in two at its start; **T28a
whole** (decision 93, the user; the agent recommended the bars first, then the garrison hero).
`master` merged in three times: after the server lane's #913 (protocol v60); after E08 T115b's
#914 (protocol v61, adventure state hash v24), the server lane's #916 (protocol v62) and #917, so
T28a took protocol v64 (the server lane's #919 claimed v63) and adventure state hash v25; after
#918 (fourteen new replays, re-recorded here) and #919; the replays re-recorded and the benches
re-measured after the second.

- **The town's rules** (`h3-towns`): `TownCommand::Heroes { town, change: TownHeroChange }`
  (`Enter`, `Leave`, `Swap`) and eight `TownCommandError`s (`GarrisonHeld` to `EmptyArmy`); the
  part's parameter `most_heroes` (8; town part **v6**; `[towns.rows]` in `h3-harvest`'s gap-fill
  with its source `exe-town-rows`, the packs' `towns.toml`).
- **The adventure** (the town lane's `src/town/rows.rs`; design §53): the record in a state block
  `AdventureState::town_heroes` (by town; none while no town holds one), adventure state hash
  **v25** (after the heroes on the map, each as a hero's: `hash_hero` shared); `town_hero`,
  `garrisoned`, `move_town_hero` (`TownHeroMove`; each check whole, then the move),
  `check_town_hero`, `town_hero_moves` (the listing's cheap reading), `absorb` (the reading's
  steps, all or nothing), `release_town_hero` (`TWN-STUB-14`); the events `GarrisonEntered` and
  `GarrisonLeft`; the owner's `AdventureObservation::garrison_heroes` (`GarrisonHeroView`);
  `OwnHeroes` over the garrison heroes; the static map's cached town entrances
  (`StaticMap::town_entrance`, `town_visitor` reads it). The one owner rule's `held` (adventure
  lane's `state/manage.rs`): `Held::Town` (an own town's garrison without a garrison hero, may be
  emptied) and `Held::TownHero` (keeps its last stack), both out of line; `set_formation`,
  `set_tactics` for a garrison hero; the hire's `Joining::TownHero` (design §49).
- **The game** (`town/rows.rs`, `town/command.rs`, `town/offer.rs`, `town/capture.rs`): the
  command after the store and the owner, `Enter`'s slots told, the town's `Told`; the legal moves
  after each own town's recruitments; a capture releases a garrison hero first and frees its type
  (`TWN-STUB-14`); the adventure lane's `state/manage.rs` `admits_army` (a town's two bars outside
  a decision, `town_bars`) and `town_told` (an army command on a town's garrison tells its views:
  the plans' told-views property found it), `stack_at` and `own_details` for a garrison hero;
  `legal.rs`' `town_partner` and `legal_town_management`; `observe.rs`' hero infos with the
  owner's garrison heroes and their details.
- **The wire** (protocol **v64**): `TownCommand::Heroes`, `TownHeroChange`, eight `TownReject`s,
  `GarrisonEntered`, `GarrisonLeft`, `Adventure::garrison_heroes` (`GarrisonHero`); the fold moves
  the owner's own view between `heroes` and `garrison_heroes`. Replay format **v16** (`town heroes
  <town> enter|leave|swap`). The env's encoding **v11**: `TOWN_HEROES` 32 to 34 with the tile head.
- **Tests:** `h3-towns` (the refusals' messages, the part's hash and refusal); `h3-adventure`'s
  `town/rows/tests.rs` (10: each move and refusal changing nothing, the listing's query against
  each check, the absorb's steps and refusals, the bars and a garrison hero under the one owner
  rule, a purchase into the garrison hero, the block's order, the capture's release);
  `h3-game`'s `town/tests/rows.rs` (4: the command's refusals, events and listing, the bars'
  management and listings, a purchase, the capture to the pool and the views); `h3-protocol`'s
  round trips (116 rejections); `h3-sim`'s layout and its mask property's `check_town_heroes`, the
  town map's fork test on seed 70 (a search over 1 to 199). `h3-map`'s plans: the moves probed after
  each town's recruitments with `check_town_heroes`, `check_invariants` (no hero both on the map
  and in a garrison, the garrison empty), `check_isolation` and `armies` over the garrison heroes,
  `view.rs`' `check_told` folding the two events.
- **Replays:** the 86 re-recorded from their commands (a scratch script; the format, the town
  part's hash, the adventure state hash version and the hash lines only); `h3-server`'s record
  literals, `h3-cli`'s and `h3-sim`'s format literals; the client lane's `h3-view` test literal
  of the wire's `Adventure`. The bot reports unchanged (no part they record moved).
- **Bench** (`master` built in the same sandbox): `manage-army-12` +0.87 % on its baseline; `legal-towns-36`
  +19.04 % (the agent's moves: 1 959 listings for 1 755; the query about 210 Ir a listing after
  two dropped designs, +56 % and +26 %), `fork-towns-36` +6.74 % (the forked game), `reach-towns-36`
  −6.01 % (the games), and, after the second merge, `fork-xl-144` +2.02 % (+0.34 % against
  master's 381 800 024: the block's pointer, 13 Ir a fork; `fork-boxes-xl-144` and
  `fork-stones-xl-144` +0.5 %, inside their gate) re-baselined with the reasons, and `day-start-builds-36`
  −3.83 % (the day's start reads the cached entrances through `town_visitor`); every other
  workload within its gate.
- **Agent's calls (change freely):** decision 93's list.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; `h3-protocol`'s,
`h3-server`'s and `h3-sim`'s tests run on their own; CI on the pull request is the record.

### T28b1 — a garrison hero's day, income and growth's inputs (2026-11-14)

One session, the user present at the start; branch `town/e09-t28b1`, `cross-lane` (the adventure
lane's `h3-adventure` `state.rs`' turn start, `state/bonuses.rs`' drops, `state/economy.rs`'
income; `h3-game`'s `state/day.rs` income; `h3-map`'s property plans). Starting point: `master` at
29f658e (#920 merged; the setup found CI's fingerprint, the light gates green). The user's
computer was connected at the start: the original data staged then (23 files and the four
manuals, SHA-256 checked against the PC's). T28b cut in two at its start; **T28b1 now**
(decision 94, the user; the agent recommended three parts, the visits and the view apart).

- **Its day** (TWN-ROW-005; ADV-MOV-002 amended): `AdventureState::start_town_heroes` (the town
  lane's `src/town/rows.rs`), called last by `start_turn`: each of the player's garrison heroes in
  `TownId` order gets `day_points` on land with its week marks' points (the Stables' 400), its
  maximum and points, written only where they change. **A gap found:** the day's and the week's
  mark drops (`drop_timed_marks`, through `change_marks`) skipped every hero not on the map, so a
  garrison hero kept a Magic Well's or a Stables' mark; the drops now change the block by hero id
  directly (one `make_mut`, the emptied entries and the block dropped as before). Its spell points
  were the store's loop already (HER-MAN-002).
- **Its income** (GAM-RES-004, -005 amended): `GameState::pay_income` pays the garrison heroes'
  `daily_income` with the heroes on the map's; `AdventureState::income`'s week reads
  `upper_bars` (`src/town.rs`: each owned town's garrison hero's army, else its garrison;
  GAM-RES-005's "a town's garrison is E09's" lifted). VCMI 1.7.5's `NewTurnProcessor` read for the
  source (its garrison hero stays in the player's list; the crystal over the heroes' armies and
  the towns' garrisons).
- **Growth's inputs** (TWN-GRO-001 amended): the share over the owner's heroes on the map and its
  garrison heroes, the Legion the visitor's plus the garrison hero's, summed by level saturating.
- **Rules:** TWN-ROW-005 (the code reading of design §48, batch TWN-3's row TWN-3-6: 1 630 points
  on Red's day 2 after 100 spent, VCMI, the design); ADV-MOV-002, GAM-RES-004, -005, TWN-GRO-001
  amended.
- **Tests:** `h3-adventure`'s `town/rows/tests.rs` (2: the turn start after the heroes on the map,
  unchanged at another's, the Stables' 400, the day's and the week's drops, the points kept out on
  the entrance; the upper bars' weekly resource once with a hero's, another owner's and a neutral
  town's); `h3-game`'s `town/tests/rows.rs` (2: the Statue's share at every own town and the two
  heroes' Legion summed; the income and the turn's points as on the map, after spent points).
  `h3-map`'s plans: `check_income` over the garrison heroes and the owned towns' garrisons,
  `check_isolation` over another player's garrison hero (its record kept, its points full at its
  owner's turn start). **CI on #924** found the view property's gap (`views_move_as_the_events_say`,
  `cc 2c6c6565…`, kept in `proptest-regressions/tests/view.txt`): T28a's `OwnHeroes::tell` tells
  a garrison hero's owner its points and marks, which T28b1 first changes in the garrison, and the
  checks of `MovementChanged` and the marks' events looked for the hero on the map only; they read
  `own_view` now (the map's hero, else `garrison_heroes`). The engine was right.
- **Replays and versions:** none moved (no replay holds a garrison hero; the replay step passed
  unchanged); no hash, format, encoding or protocol version.
- **Bench** (`master` 421b41b built in the same sandbox): the first build cost
  `day-start-builds-36` +7.86 % and `day-start-towns-36` +3.74 % on their baselines, the week's
  resources' `Chain` of `Flatten`s over the upper bars' stacks into a `BTreeSet` (21 368 and
  35 634 Ir of one iterator's `next`); a short list of the creature types with an ability left
  +0.17 % (564 984 to 565 941 Ir) and +0.11 % (1 311 465 to 1 312 852) against `master`; no
  workload re-baselined (`TRAPS.md`).
- **Agent's calls (change freely):** decision 94's list.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T28b2 — a garrison hero defends; T-5; the town's views (2026-11-14)

One session, the user present at the start; branch `town/e09-t28b2`, `cross-lane` (the adventure
lane's `h3-adventure` `state.rs`' battle's end and Necromancy, `state/bonuses.rs`' marks;
`h3-game`'s `state/battle.rs` and `state/manage.rs`; `h3-map`'s view property; `h3-protocol`).
Starting point: `master` at 0872fc6 (#924 merged; the setup's baseline lost to a VM restart, CI's
green run on `master` stands for it). The user's computer was connected at the start: the original
data and the four manuals staged then (28 files, SHA-256 checked against the PC's). T28b2 **whole**
(decision 95, the user; the agent recommended two parts, the visits with T-5 first).

- **The visits** (survey §9.1's rows 1 and 3; TWN-SIE-001, TWN-VIS-001 amended):
  `AdventureState::town_visit` finds a town of another team defended by its garrison hero with a
  stack; `siege_start`'s defender is the visitor, else the garrison hero (`Defender::Hero` off the
  map, its owner's side), else the garrison; beside a garrison hero the visitor's battle is a
  `BattleStart` without a town (no siege: the gate tile's battle, no merge, no town's facts), the
  town not taken. `town::merge` skips a defender off the map (TWN-SIE-002 only for a visitor).
- **The battle's end** (TWN-CAP-003, GAM-BAT-003 amended): `AdventureState::end_battle` hands a hero
  not on the map to `end_town_hero_battle` (the town lane's `src/town/rows.rs`): the survivors by
  slot, an empty army kept for a winner (survey §9.3, the original's); else
  `release_town_hero` (`HeroRemoved` to its owner at the entrance). The game's `end_battle` keeps a
  garrison winner's survivors when none is left, frees a beaten one's type, and tells its town's
  views after the heroes' part; Necromancy reaches it (`raise` through `hero_or_held_mut`), its
  bonuses are cleared (`change_marks` admits a garrison hero); the capture after the attacker's
  win as against a visitor.
- **T-5** (`TWN-QUIRK-11`, preserve without a flag, decision 12): a garrison hero without a stack
  does not defend, the step enters and captures, the hero to the pool; reached by a defence won
  with creatures the battle made alone. Sources: `town_visit` 0x4AB0EE (E09 T1b1) and
  `town_defended` 0x5BDFE0 (E24 M3 T6a1, T7b2), disassembled again here (the visitor +0x10, else
  the garrison hero's +0x91, else the town's +0xE0, `army_has_creature` 0x4496A0); VCMI 1.7.5's
  `armedGarrison` counts any garrison hero (it never makes one without a stack). `TWN-STUB-14`
  lifted.
- **The views** (TWN-VEW-001, -002 amended): a town's garrison at every level is its upper bar
  (`AdventureState::upper_bar`: the garrison hero's army with one, `town_garrison_army`
  0x5C1460), the owner's slots included; the top level's mark `TownOwnerView::garrison_hero` and
  `TownChanged::garrison_hero` (protocol **v67**, `TownOwnerFacts::garrison_hero`). Told: the
  moves', the recruits' and the capture's `Told`s as built; new, `state/manage.rs`' `town_told`
  for a command naming a garrison hero's army and `state/battle.rs`' around a garrison hero's
  battle's end. No `GarrisonHeroChanged` (the whole view is told).
- **Rules:** TWN-SIE-001, -002, TWN-VIS-001, TWN-CAP-003, TWN-VEW-001, -002, GAM-BAT-001, -003
  amended; the ledger's `TWN-QUIRK-11`.
- **Tests:** `h3-adventure`'s `town/rows/tests.rs` (3: row 3 and T-5's visit, row 1's battle at
  the gate, the battle's end with Necromancy, the bonuses and the release); `h3-game`'s
  `town/tests/siege/held.rs` (3: the siege against the garrison hero as built from the town, its
  win and its loss with the capture and the town's view told before it, row 1 played to its end
  with the town standing) and `town/tests/rows.rs` (T-5's regression test, the views with the
  mark and their tells). `h3-map`'s plans: `check_town_views` expects the upper bar and the mark;
  **a stress of 300 cases** found the told views' fold keeping a town's garrison by slot from the
  `StackChanged`s alone (after `Enter` the hero's army is told by `TownChanged`): the fold takes a
  told view's slots whole now (`cc 38e88291…`, kept). A second gap, the adventure lane's,
  failing on `master` 0872fc6 too: the hygiene property's map garrison index in
  `ExchangeOpened` (two seeds); the adventure lane's #929 (E08 T116b2) fixed it while this pull
  request's CI ran, and both seeds pass after `master` was merged in.
- **Replays and versions:** none moved but protocol v67 (no replay holds a garrison hero).
- **Agent's calls (change freely):** decision 95's list.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T29a — the scripted replays on the builds map and the census of growth to day 29 (2026-11-15)

One session, the user present at the start; branch `town/e09-t29a`, `cross-lane` (the adventure
lane's `h3-map` `src/tests/builds_map.rs` and `examples/original_data.rs`, the replays, their notes
in `h3-cli`'s `tests/adventure_replays.rs`, `fixtures/maps/`, the workload); the benches README's
row in `shared/e09-t29a-benches`. Starting point: `master` at 6b3087d (#930 to #933 merged; the
setup found CI's fingerprint, the light gates). The user's computer was connected at the start
only: the original data and the four manuals staged then (28 files, SHA-256 checked against the
PC's), used for the census and the scoped verify. T29 **in two, T29a now** (decision 96, the
user; the agent recommended three parts, the replays first).

- **The builds map** (`fixtures/maps/e09-builds-36.h3m`, `builds_map.rs`), every existing id kept:
  red's Dungeon "Pit" (town 4: a Fort, the Warren and the Mushroom Rings, the Portal of Summoning,
  the Guardian of Earth, an empty garrison), the neutral Rampart "Grove" (town 5: a Fort, the
  Centaur Stables and the Dwarf Cottage, six Centaurs), red's Valeska (hero 2, four pikemen) on
  (3, 10), off every path of the six replays (on (4, 8) `builds-s262-d14`'s Orrin walked onto her),
  red's Guardhouse owned at the start (the Pit's Portal holds its fourteen Pikemen from the setup,
  checked). The map test's "town not in the store" is town 6 now.
- **Replays:** the six `builds-*` re-recorded from their commands (only the map hash, the
  checkpoints and the final hashes moved). Three new, scripted (`crates/h3-cli/tests/
  adventure_replays.rs`' notes): `builds-month-s1` (the Keep's horde built, four weeks to day 29:
  week 3 a creature's, the Grove's garrison week in weeks 2, 3 and 5, the Pit's Portal each week,
  month 2 a Plague; every count checked by hand against §41 to §46 first: the Pit's Troglodytes
  14, 45, 76, 107, then (138 − 31) ÷ 2 = 53), `builds-rows-s1` (three Royal Griffins and four
  Griffins from the Upg. Griffin Tower's one count, `Enter` with the absorb, the Portal's fourteen
  Pikemen, a week in the garrison, `Leave` with full points, Valeska's `Enter`, `Swap`),
  `builds-siege-s1` (Valeska at the gate beside Orrin in the garrison: Yog's battle outside, then
  his siege of Orrin alone, the walls shot, his level-up answered, the Keep captured). The
  siege's battles are `h3_sim::bots::aggressor`'s commands for each side, written by a scratch
  generator (a throwaway `h3-cli` test that also expands `goto` lines into paths; not committed).
- **Bench:** `day-start-builds-36` 858 034 Ir against `master`'s 565 557 measured in the same
  sandbox (+51.7 %, re-baselined): no code changed; the same functions with six towns' growth
  (`TownGrowth::of`, `week_growth`), the Grove's `garrison_week` and the Pit's `summon` reached.
- **The census** (`h3-map`'s `original_data`, exit criterion 11; `played_growth`, `growth_in`,
  `growth_at`, `first_creatures`, `portals`, `inputs_seen`): every one of the 160 games that build
  (seed 0) is played by `EndTurn` alone to day 29, each town's fourteen counts checked after every
  `EndTurn` against the census's own walk: the setup's first creatures (1 464 towns, those no
  day-1 event fired in), each week's growth by its own sum of §41 and the week's bonus, halved
  for a neutral town; each neutral town's garrison week by the level whose stacks grew (a
  creature's growth, else an error; its two counts lose it); month 2's kind (a creature's
  doubling by `quirk.town.creature_month_exact`, the Plague by
  `quirk.town.neutral_plague_full_growth`); each town event's buildings (T19c2's `fire`) with their
  first creatures and its creatures. All 160 hold, in 17 s in debug: 1 468 towns, 5 872 town weeks
  (562 normal, 74 a creature's, 4 a building's), 1 571 garrison weeks grown and 745 not, 77
  creature months, 12 Plagues, 83 normal; 28 events' builds' first creatures, 85 events' creatures
  added and 128 lost; the largest count on day 29 222. What the towns read: 889 owned, 77 a Citadel
  or Castle, 51 a horde, 10 a Grail building, 6 the owner's dwellings; none a share, a Legion or a
  standing Portal of an owner (the builds map's replays and the unit tests hold those). Two
  mutations of the walk (the Castle's bonus dropped, the neutral halving dropped) fail it on the
  first map.
- **CI on #937** failed in the server lane's tests, which the scoped verify does not run: they
  count the golden adventure replays (`h3-protocol`'s fold and session gates, `h3-server`'s
  record test: 86). Now 89 (`cross-lane`, mechanical); their gates play the three new replays
  through the fold, the queries and the session, all passing (`TRAPS.md`).
- **Versions:** none moved.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T29b — the properties and the month workload (2026-11-15)

One session, the user present at the start; branch `town/e09-t29b`, `cross-lane` (the adventure
lane's `h3-map` `src/tests/play.rs`, `view.rs` and their `proptest-regressions`, the workload in
`benches/adventure/`); the benches README's row in `shared/e09-t29b-benches` (#942). Starting
point: `master` at 8026f16 (#934 to #937 merged; the setup's full baseline passed every step but
coverage, whose test run met the hygiene case below). The user's computer was connected at the
start only: the original data and the four manuals staged then (28 files, SHA-256 checked against
the PC's), used for the scoped verify. T29b **whole** (decision 97, the user; the agent recommended
two parts, the properties first), with a new month workload in place of counting
`day-start-builds-36`.

- **The properties** (`h3-map`'s plans, design §47 and §55): `check_invariants` holds every
  town's fourteen counts and its Portal's at most `MAX_AVAILABLE` (`TWN-QUIRK-8`), the plans'
  town events giving the file's most (65 535) of a level in one level of 21 (`town_events`: a
  drawn 20 becomes 65 535, the draws unchanged), so that counts meet the bound and grow, double,
  halve and sell there; `check_towns` holds that the Plague never raises a count of a town no
  event of the command reached (`MonthStarted`'s Plague: the week's growth taken again, then
  halved) and that a purchase lowers what it sold by its `Buy`'s amount (an index's count or the
  Portal's), never more than it held; a new property `a_forks_week_is_its_parents` ends the turns
  (a battle played as `Step::Fight`, a decision, meeting or dialog answered by a legal command)
  until a step starts a week, crosses it first on a fork, which leaves the parent's hash and its
  towns' counts and Portals as read out before, then on the parent: the same result, events,
  state and hash (every case reaches a week within 512 steps, asserted). T28a's invariants stand.
- **A garrison hero's siege reached:** the plans entered a garrison twice in the 512 games of
  `the_plans_capture_and_besiege_towns` and never besieged a garrison hero. `Step::Town`
  (`visit_town`) now gives a listed hero move at an own town in five steps of eight (`Enter` in
  three, `Leave` and `Swap` in one each); its cover test counts the sieges by the defence (the
  garrison alone, a visitor, a garrison hero alone, a visitor beside a garrison hero), each won and
  lost, the hero moves, the Plagues and the weeks that leave a count at its bound, all above 0:
  `sieges: [34, 30, 16, 9]`, `won: [17, 9, 3, 5]`, `lost: [16, 18, 10, 4]`, `rows: [144, 95, 31]`,
  `plagues: 41`, `saturated: 14`.
- **Oracle gaps the new games met** (the plans' oracles, no engine change): `check_progression`
  took a level-up's hero from the map only (a garrison hero that wins its defence levels up: its
  owner's level-up, as T28b2's `held.rs` plays); `check_battle_end` and `check_heroes_after_battle`
  read the battle's heroes from the map only (a garrison hero that wins stays in the garrison, its
  army kept even empty, decision 95; found by a stress of 240 cases); CI's first run on #946 met
  two more: a garrison hero's Necromancy after its defence and its level-up answered later
  (`check_heroes_after_battle`'s raise, `check_conservation`'s levels), and `check_isolation`
  holding another player's garrison hero unchanged through the siege that player's hero fought.
  `hero_anywhere` reads a hero on the map or in a garrison; the cases are in
  `proptest-regressions/tests/play.txt`.
- **A gap of `master`'s** (the setup's baseline, the coverage step's test run on 8026f16): the
  hygiene property's other game (`view.rs`' `vary`) flooded a map garrison's entrance (an ownable
  object since E08 T116b1), which the import refuses (`OwnedOnWater`); its `entrances` keep a
  garrison's land as a mine's (the case in `proptest-regressions/tests/view.txt`).
- **Stress:** the properties of `play.rs` and `view.rs` at 240 cases (five times their 48) pass;
  after CI's case, `every_step_keeps_the_commands_invariants` at 600.
- **Bench:** `month-start-builds-36` (new, `benches/adventure/`) baselined at 2 620 845 Ir: the
  step that starts day 29 in each of 16 random games of 28 days on the builds map (seed 1's
  months: 2 Plagues, 6 a creature's with 36 monsters, 8 normal), 61 % the creature months' walk
  (`AdventureState::start_month`), the towns' week 8.5 %, their month 3.7 % (`h3_towns::week::
  month` 3.3 %). No code changed: every other workload as on `master` (the setup's bench-check
  on 8026f16: all inside their gates, `fork-dwellings-xl-144` +1.99 % as the adventure lane noted).
- **Versions:** none moved.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T30 — the audit; increment 3 closed; increment 4 refined (2026-11-15)

One session, the user present at the start; branch `town/e09-t30`, docs only, with the README's
row in `shared/e09-t30-readme`. Starting point: `master` at 396f6f8 (#942, #946, #949, #952 and
#953 merged; CI green there). The user's computer was connected at the start only: the original
data and the four manuals staged then (28 files, SHA-256 checked against the PC's). The VM
restarted during the setup's baseline (`uptime`); CI's green runs on `master` stand for it, and
`cargo xtask verify --scope town --summary --with-original-data` ran on `master`'s code for the
audit. **At its start** (the user, asked with a first look at the audit, `TWN-Q15`): T30 whole
(decision 98), increment 4's scope and first round (decisions 99, 100) and the server lane's
question on the garrison moves (decision 101).

**The audit of increment 3's exit criteria** (`E09-archive.md`; design note §57), on `master`'s
code:

1. **The part, the counts, the first creatures.** `h3-towns`' `the_growth_roles_compile_as_given`
   (the roles `horde`, `portal`, `week_of`, a record's `roe` and the growth's parameters; part v5,
   v6 by T28a), `twn_quirk_8_a_town_s_counts_saturate_at_the_bound`,
   `the_counts_the_portal_and_a_record_are_written_to_one_town` (store v3);
   `a_built_dwelling_gives_its_first_creatures` and the setup's custom set (TWN-GRO-005), the game's
   `a_built_dwelling_gives_its_first_creatures_in_the_game` (a command's build) and
   `a_town_event_s_creatures_join_its_dwelling_s_count_after_its_buildings` (an event's); the plans'
   `check_invariants` holds every count at most `MAX_AVAILABLE` with events giving 65 535 (T29b).
   **Holds.**
2. **A dwelling's growth and the week.** `h3-towns`' growth tests (the castle and the Citadel's
   half, the Statue's share and the Legion, the first horde, the map's dwellings at a growing
   index, the Grail on the whole, the bound) and the week's (`a_week_adds_each_growing_index_s_
   growth_halved_without_an_owner`, the creature's bonus before the halving); the game's
   `a_towns_inputs_are_its_owners_share_its_visitors_legion_and_its_owners_dwellings`,
   `every_town_grows_its_week_on_the_week_s_first_day_a_neutral_one_half`, the garrison hero's
   `a_garrison_hero_s_legion_and_statue_count_in_its_owner_s_towns_growth` (T28b1). **Holds.**
3. **The week's kind and the month.** `h3-towns`' `a_week_s_draw_is_a_creature_s_one_time_in_four_
   never_on_a_month_s_first`, `on_roe_a_week_never_names_a_record_without_roe`,
   `the_first_week_of_building_overwrites_the_draw_every_week`, the month's
   `a_creature_s_month_doubles_its_creature_s_counts_after_the_week`, `the_plague_takes_the_growth_
   then_halves` and the two flags' (`twn_quirk_10_off_…`, `twn_quirk_9_off_…`); the game's
   `a_creature_s_week_is_drawn_from_the_towns_stream_told_and_grown` (`WeekStarted`, the
   observation's week), `a_town_s_week_leaves_the_game_s_stream_where_it_was`,
   `a_building_s_week_is_every_week_and_makes_the_month_its_creature_s_without_doubling`,
   `a_month_doubles_its_creature_s_counts_and_the_plague_halves_after_the_week`. **Holds.**
4. **The garrison week, the Portal, the events' creatures.** `h3-towns`' `garrison` (nine) and
   `portal` (five) tests; the game's `a_neutral_town_s_garrison_grows_each_week_from_the_towns_
   stream_and_is_told`, the Portal's seven (each week in an owned town, a command's and an event's
   build, the setup's, a flagged dwelling, a capture that finds it empty, a capture that keeps
   it), the events' creatures (above); `TWN-STUB-10` lifted (T26a1). **Holds.**
5. **Recruitment.** `h3-towns`' `recruit` (eleven: every source, refusal and stock, the one count,
   the Portal's); the game's `town/tests/recruit.rs` (the dialog the owner's alone, every refused
   recruitment and purchase changing nothing, the purchase into the garrison, the Portal's, an
   upgraded index's two creatures from one count, the legal recruitments following the builds)
   and `a_town_s_purchase_joins_its_garrison_hero_s_army`; GAM-DEC-008 amended. **Holds.**
6. **The rows.** `h3-adventure`'s `the_town_s_bars_follow_the_one_owner_rule` (`town/rows/tests.rs`)
   and the game's `the_owner_manages_its_town_s_bars_outside_a_decision`; E08's army reference
   and owner rule as agreed (T114a). **Holds.**
7. **The garrison hero's moves and day.** `h3-adventure`'s `the_visitor_enters_the_garrison_with_
   its_whole_record_and_the_garrison_s_creatures`, `entering_is_refused_with_a_garrison_hero_
   another_s_visitor_or_no_room`, `the_garrison_hero_leaves_onto_the_entrance_with_its_points_
   kept`, `leaving_is_refused_without_a_garrison_hero_a_free_entrance_room_or_an_army`,
   `the_visitor_and_the_garrison_hero_swap_with_their_armies`, `a_garrison_hero_gets_its_day_s_
   points_at_its_owner_s_turn_and_drops_its_marks`, `a_town_s_upper_bar_gives_its_owner_the_week_
   s_resources_once_with_its_heroes`; the game's `a_garrison_hero_s_income_and_points_are_those_it_
   had_on_the_map`, `each_refusal_of_a_move_is_the_town_command_s_own`. **Holds.**
8. **What a garrison hero changes.** The game's siege tests (`a_garrison_hero_alone_defends_its_
   town_s_siege_with_its_own_army`, `a_garrison_hero_keeps_its_town_as_the_winner_and_leaves_the_
   game_beaten`, `beside_a_garrison_hero_its_visitor_fights_at_the_gate_and_the_town_stands`,
   `beside_a_garrison_hero_the_visitor_s_battle_at_the_gate_is_no_siege`),
   `twn_quirk_11_a_garrison_hero_without_a_stack_does_not_defend`, `a_town_s_views_show_its_
   garrison_hero_s_army_and_its_mark`; the plans' cover test counts garrison heroes defending
   alone (16) and beside a visitor (9), won and lost (T29b). **Holds** for the views; **the gap:**
   the moves' events (below).
9. **The views.** The game's `town/tests/view.rs` and the views' tests of T26b, T27 and T28b2 (the
   owner's counts, Portal and hire dialog; the week's kind public in `WeekStarted` and the
   observation); the plans' `town_views_move_as_the_events_say` and the hygiene's
   `vary_changes_the_towns_the_viewer_never_sees_only` with growth, recruits and garrison heroes;
   `h3-protocol`'s fold gate over the 89 golden adventure replays (the server lane's). **Holds.**
10. **The env and the replays.** Action encoding v11 (`RECRUIT`, v10 by T27; `TOWN_HEROES`, v11 by
    T28a): `h3-sim`'s town map tests (`the_env_plays_the_town_map_to_its_end_of_days`,
    `town_map_rollouts_and_forks_are_deterministic`), `a_hire_dialog_is_answered_through_the_env`,
    the mask property's `check_town_heroes`; replay format v16 (`town recruit`, `town heroes`): the
    builds map's `builds-month-s1`, `builds-rows-s1` and `builds-siege-s1` (T29a) play growth
    through a month, recruitment and the three moves. **Holds.**
11. **Rules, quirks, gates, batch TWN-3.** TWN-GRO-001 to -007, TWN-REC-001, -002, TWN-ROW-001 to
    -005 with their ids, sources and tests (`check-rules` passes); `TWN-QUIRK-8` to `-11` in
    `quirks-town.md`, each with its regression test. The gates on `master`'s code here: coverage (111 gated
    files in the four logic crates, no exclusion); determinism (debug, release and native, the 89
    golden adventure replays among them); the census of `original_data` (all 160 maps to day 29,
    T29a's counts unchanged: 1 468 towns, 5 872 town weeks, 1 571 garrison weeks grown and 745
    not, 28 events' builds' first creatures, 85 events' creatures added and 128 lost, the largest
    count 222); the benchmarks (all 34 workloads inside their gates, from −1.14 %
    `day-start-towns-36` to +0.98 % `siege-towns-36`), the month
    workload `month-start-builds-36` baselined by T29b (decision 97: the user's pick of a month
    workload, so the criterion's "week workload" is **worded as built**). Batch TWN-3 run and
    scored (T23, survey `E09-growth.md` §17: every row as the reading). **The gap fixed** (below).
    **Holds.**

**The gaps.** (1) Criterion 11 wants the consensus rows `verified` (E08's audits' wording), but
every rule behind batch TWN-3's rows stayed `implemented`: TWN-GRO-001 (row 2), -003 (rows 1 to 3),
-004 (rows 1 and 3), -006 (row 4), -007 (row 5) and TWN-ROW-005 (row 6) cite their rows and match
them in every read of both runs: now `verified`, each with a note naming what no row played (the
code reading holds those). TWN-GRO-005 (the setup's first creatures, rows 1 and 2 on day 1: T25a's
record named row 2, the catalog did not), TWN-REC-001 (row 6's hire: two creatures from one
count, the price, the garrison hero's army, never the visitor's) and TWN-ROW-002 (row 6's move and
its refusal, GENRLTXT 276) cited no row though row 6 or rows 1 and 2 played them: the observation
added, `verified`. Stay `implemented`: TWN-GRO-002 (the batch forced the draws), TWN-REC-002 (no
Portal purchase played), TWN-ROW-001, -003, -004 (no bar move, leave or swap played), TWN-EVT-004
and TWN-VEW-004. (2) The server lane's inbox note of 2026-11-14: `GarrisonEntered` and
`GarrisonLeft` name the hero to every viewer of the town's entrance, below level 3 too, where
`docs/06-security.md`'s town part says the moves are told as heroes leaving and entering the map:
decided by the user (`TWN-Q15` (4), decision 101: projected below level 3), queued as **T31**
(code). **Left open, by design:** `TWN-STUB-2` (the Grail, increment 6 and E10), `TWN-STUB-6` and
`-13` (increment 5), `TWN-STUB-8` (the battle lane's inbox).

**Increment 3 closed** (2026-11-15): its section (the refinement's note, decisions 64 to 97, the
task list with T30's line and the exit criteria with criterion 11 worded as built) moved verbatim
to `E09-archive.md` (checked by script: every line of the old epic in the new epic or the archive,
but the six edited ones); its design note's Part E stays in `E09-design.md`, as Parts A to D
(decision 40). The README's row (3 of 6) in `shared/e09-t30-readme`.

**Increment 4 refined** (`TWN-Q15`; decisions 98 to 102): the Tavern as sketched (decision 99);
the survey with its checkpoint, then the drafts with Part F and the cut (decision 100); T31 first
(decision 101). The inbox's seven notes that increment 3 acted on or answered (the server lane's
question, E08 T115a's marks, T95b's hire dialog, T94a's and T94b's dwellings, ADV-Q41's
recruitment, the week's creature) archived with where their work went (`QUEUE-ARCHIVE.md`); the
Tavern's three (the tavern weights, `Marshland Menace`'s heroes, `ADV-STUB-9`) kept for T32. The
queue holds T31, T32 and T33.

**Verify:** the engine's code is `master`'s. `cargo xtask verify --scope town --summary
--with-original-data` on `master`'s code with no failing step (coverage 14.5 minutes, determinism
9.6, bench-check 9.8, original-data 0.9; `deny` and `ffi-smoke` skipped by the scope); on the
branch the docs gates (`check-docs`, `check-rules`, `check-stubs`, `doc-drift --base
origin/master`) and `check-lane --base origin/master` (the town lane's paths and the common ones);
`shared/e09-t30-readme` `check-lane` and `check-docs`. CI on the pull requests is the record.

### T31 — the garrison moves below level 3 (2026-11-15)

One session, the user present at the start; branch `town/e09-t31`, `cross-lane` with the user's
consent (the adventure lane's `h3-adventure` `command.rs` and `observe.rs` and `h3-map`'s
`src/tests/view.rs`; the server lane's `h3-protocol` `src/map/game/event.rs` and
`src/tests/game_event.rs`). Starting point: `master` at db8406b (#956, #957 merged). The user's
computer was connected at the start only: the original data and the four manuals staged then (28
files, SHA-256 checked against the PC's) with `H3MAPED.HLP`, for the scoped verify and T32. The
user chose the design at the start (`AskUserQuestion`; recommended): **the events carry who is
named, the projection decides**, over the town code emitting two events per move (the raw
stream would then split by other players' hidden Visions, which the hygiene property compares).

- **The events:** `GarrisonEntered` and `GarrisonLeft` carry `named: Viewers`, the viewers told
  the move by name. `AdventureState::move_town_hero` takes it from the game, adds the moving
  player (the owner is always named) and keeps those of the entrance's viewers.
  `AdventureEvent::project` tells a viewer in `named` the event with itself alone, any other
  viewer of the entrance `HeroRemoved { hero, pos }` for `GarrisonEntered` and
  `HeroStepped { hero, from: None, to: Some(pos) }` (a hero coming into sight, the shape a step
  from an unseen tile already has; SRV-Q26 (c)) for `GarrisonLeft`; a swap's two events alike.
- **The level-3 test** (`h3-game`'s `town::view::named`): the players whose view of the town
  (`Seer::view`, the observation's) is at TWN-VEW-003's top level: its owner's team and a player
  one of whose heroes sees its entrance by Visions, among those that explored a tile of it (so a
  player with the entrance only on its fringe under `ADV-QUIRK-1` is not named, as it has no view
  of the town). Nobody for a sink that keeps nothing (`Sink::RECORDS`), as `Told::before`. A move
  changes no viewer's level of its own town (the moving hero is the owner's).
- **The wire:** unchanged. `h3-protocol`'s mapping ignores `named` (a projected event names the
  viewer alone); the fold already read both forms (a hero coming into sight stays `SRV-STUB-3`).
  No protocol version.
- **Tests:** the adventure's `a_garrison_move_names_its_hero_only_to_the_viewers_that_see_the_garrison_hero`
  (the four branches, a viewer that does not see the entrance told nothing); the game's
  `a_garrison_move_names_the_owners_team_and_the_heroes_whose_visions_reach_it` (Visions' reach,
  a neutral town, a missing town, the no-op sink); `h3-map`'s `views_move_as_the_events_say` now
  checks each viewer's projection of every garrison move against its observation after the
  command (`check_garrison_moves`: by name exactly when its view of the town is at the top
  level); `h3-protocol`'s `every_event` names each garrison event to both players and to player 0
  alone. A stress of `views_move_as_the_events_say` at 240 cases (five times its 48) passed, the
  plans telling 58 moves by name and 72 as heroes leaving or coming into sight.
- **Docs:** TWN-ROW-002 to -004's summaries, the events' doc comments, `docs/06-security.md`'s
  town "Events" line (by name only to the viewers that see the garrison hero); notes in the
  adventure and server lanes' inboxes (the server lane's question of 2026-11-14 answered).
- **Versions:** none moved (events are not hashed; the wire is unchanged).

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T32 — increment 4's survey and its checkpoint (2026-11-15)

One session with T31, the user present at the start; branch `town/e09-t32` (docs only, stacked on
`town/e09-t31`). T32 **whole** (the user at T31's start; the agent had proposed T32a, the hire,
then T32b, the information). The original data, the four manuals and `H3MAPED.HLP` staged at the
start (29 files, SHA-256 checked against the PC's). Survey `docs/rules/surveys/E09-tavern.md`.

- **The code reading** (§2 to §6): `tavern_hero_draw` 0x4BB5E0, `tavern_new_heroes` 0x4C8450,
  `tavern_slot_replace` 0x4C86A0 (renamed from an inferred name), `hero_leave_game` 0x4D9EC0
  (likewise), the two hires `town_tavern_hire` 0x5C12E0 and `map_tavern_hire` 0x4D7890, the
  tavern dialog and its hints, the week's and the start's release and fill, bit 17 of +0x105 (a
  returning offer: set at a retreat or a surrender, cleared at its owner's turn end and at a hire),
  the customised types' reader and their application to every type before the first draw, the
  banned skills' readers (none on the hire's path), the Thieves' Guild's rows and parts by the
  tavern count and the Den of Thieves' 99. Fifteen map entries added, four rewritten.
- **The SoD Conflux rule** is dead in this build (edition 3, never written): a lead for E11 in
  the battle lane's inbox.
- **Compared:** VCMI 1.7.5 (`HeroPoolProcessor`, `TavernHeroesPool`, the removed hero,
  `obtainPlayersStats`, the weights from `HCTRAITS.TXT`), the wiki's Tavern page, the RoE manual.
  The shape agrees; disputes TD-18 to TD-21; quirk candidates T-14, T-15.
- **The census** (159 `.h3m` maps, a throwaway example over `h3-map`'s reader, not committed):
  versions, the allowed and disposed heroes, the customised types (`Marshland Menace`'s 15 at
  2 000 experience the only experiences), the bans (110 hired types start with a banned skill on
  19 maps), towns and taverns at the start (742 players: 228 none, 462 one, 43 two, 6 three, 3
  four), the map's Taverns (93 on 47) and Dens (227 on 107), the Conflux heroes.
- **Batch TWN-4's rows** (§9): eight, for T33's cut.
- **The checkpoint** (`TWN-Q16`, one round of four, each as recommended): decisions 103 (TD-18 the
  reading), 104 (TD-19, TD-20 the reading), 105 (T-14 the original's, not a quirk), 106 (the map's
  Tavern in increment 4), 107 (the agent's calls). The survey's §11.
- **Versions:** none (docs only).

**Verify:** the docs gates (`check-docs`, `check-rules`, `check-stubs`, `check-exe-map`,
`check-lane --base origin/master`, `doc-drift --base origin/master`).

### T33a — the hire's drafts and the design note's Part F, §59 to §71 (2026-11-16)

Decision 100's second task, cut in two at its start (the user, `TWN-Q17`'s first round, decision
108: the hire now, the information with the cut and the checkpoint next; recommended over T33
whole and over every draft first). Branch `town/e09-t33a`, docs only (the drafts, the design note,
the survey's §12, the address map, the epic, the log, the answered questions, the lane's queue and
HANDOFF). Starting point: `master` at f6dd6d3 (#964; #961 and #962 merged; CI green on
`master`'s run). The user's computer was connected at the start: the executable, `h3maped.exe`,
the four LODs, `maps.tar`, `extracted.tar`, the saves, the four manuals and the two help files
staged (18 files, SHA-256 checked, one digest of the sorted lists).

**The ADR-level choices** (`TWN-Q17`'s second round, four; decisions 109–112, each as
recommended): a hero type's record made at its first offer and kept by type; the tavern weights in
the town part's faction records; a town command for a town's tavern, a decision for the map's; the
dismissal in this increment.

**Read for it** (survey §12; three map entries added, one extended): the hero screen's dismissal
(`hero_screen_refresh` 0x4E1A50's button 129: refused from the town screen and the tavern dialog,
for a town's visitor, for the loss condition's hero, view-only, and for a player's one hero without
a town; `hero_mission_critical` 0x5F2A40, `hero_screen_no_dismiss` 0x698A90, `hero_screen_show`'s
callers' flag); `hero_place` and `hero_show` have no reveal; `hero_leave_game`'s refill of the
points comes after the army is cleared, so `hero_max_land_movement` starts from speed 20 (T-16).
VCMI 1.7.5's hero window (its dismissal button's tests: no visitor test, TD-22) and server pack
(ownership only); VCMI's object config (the Tavern visited from a neighbour, `blockVisit`).

**Written:**
- **The drafts**, the hire's half (T33b adds the information's): `docs/lanes/town/adr-drafts/`
  `0028-increment-4.md` (the tavern weights and the price, `TownCommand::Hire`, the pool's tavern
  heroes, the versions), `0017-increment-4.md` (the commands, the pool's records and away records,
  the players' factions and slots, the draw, the taverns' week and the start's fill, the slots over
  a turn, the taverns' stream, the tavern decision, the hire's application, a hero that leaves the
  game, the versions), `0016-increment-4.md` (a hero placed by a hire and its away record, the
  dismissal's removal, the kind `Tavern`), `0013-increment-4.md` (a tavern hero's backpack
  equipped at a fill, its spell points off the map).
- **The design note** Part F, §59 to §71: scope and choices (§59), the part v7 (§60), the pool's
  records and the slots (§61), the draw (§62), the fills (§63), the town's hire (§64), the
  replacement (§65), the map's Tavern (§66), a hero that leaves the game (§67), the slots over a
  turn (§68), the dismissal (§69), the map's hero settings (§70), ids, versions and the open points
  for T33b (§71).
- **New ids:** dispute TD-22 (the dismissal's visitor test); quirk candidate T-16 (a leave's points
  over the cleared army); stubs planned `TWN-STUB-15`, `-16`; rules planned TWN-TAV-001 to -006,
  GAM-DEC-010, GAM-MAN-003.
- **The agent's calls** (decision 113), for `TWN-Q18`.

**The queue:** T33a out, T33b in. No version moved. Ids: questions next 18, decisions next 114,
disputes next TD-23, quirk candidates next T-17.

**Verify:** docs only: the docs gates (`check-docs`, `check-rules`, `check-stubs`,
`check-exe-map`, `check-lane --base origin/master`, `doc-drift --base origin/master`); CI on the
pull request is the record.

### T33b — the information's drafts, Part F §72 to §81, the cut, the checkpoint (2026-11-16)

Decision 108's second half, kept whole at its start (the user, decision 114; the agent had
proposed T33b1, the information, the views, the env and the protocol, then T33b2, the gates, the
cut and the checkpoint). Branch `town/e09-t33b`, docs only (the drafts, the design note, the
survey's §13, the address map, the epic, the log, the answered questions, the lane's queue and
HANDOFF, three lanes' inboxes). Starting point: `master` at fcc19e7 (#966 merged; CI green on
`master`'s run). The user's computer was connected at the start: the executable, `h3maped.exe`,
the four LODs, `maps.tar`, `extracted.tar`, the saves, the four manuals and the two help files
staged (18 files, SHA-256 checked, one digest of the sorted lists).

**Read for it** (survey §13; five address-map entries added, one extended):
`thieves_guild_fill` 0x5DDA10 to its end (the per-player parts: 1 the best hero by
`hero_army_power`, the first of equals; 2 its four primary skills clamped to 0–99; 3 the player's
`ai_type` as text; 4 the best creature by AI value over the towns' shown armies then the heroes'),
`thieves_guild_row_value` 0x5DEE70 whole (the towns by owner, the heroes on the map, the
treasury's sums, the backpacks alone, the heroes' powers plus the defended towns' plain army
values, the day's gold), `thieves_guild_rank_sort` 0x5DF160 (an exchange sort, ties sharing a
place), `hero_worn_count` 0x4D9070 (its argument 1 counts nothing: T-17), `army_power` 0x427690,
`thieves_guild_proc` 0x5C9930 (a portrait opens a screen only in the viewer's column). VCMI 1.7.5's
`GameStatistics.cpp` and `obtainPlayersStats` (the best hero by experience, worn and backpack
artifacts, the heroes' armies only: TD-23, T-17), its object config (the Den of Thieves entered,
not `blockVisit`); the RoE manual (the report named only).

**Written:**
- **The drafts**, now whole and accepted: `0028-increment-4.md` (the Thieves' Guild's level by
  `has_tavern`, the versions), `0017-increment-4.md` (point 2: the tavern heroes' view and
  `TavernOffered`, `HeroHired`, the guild's query and its view, a Den's `ThievesGuildSeen`; point
  6: `TavernOpened` and the decision's view whole; point 8: `HIRE` + 2 and the tavern answers by
  role, no `Dismiss` entry; points 9 to 11: the versions, a Den's outcome, `TWN-QUIRK-12`),
  `0016-increment-4.md` (the kind `DenOfThieves`, a hired hero's appearance and a dismissal told),
  `0013-increment-4.md` (status only).
- **The design note** Part F §72 to §81: scope and choices (§72), the Thieves' Guild (§73), the
  Den of Thieves (§74), the views and events (§75), the env and the protocol (§76), the
  increment's ids, versions, quirks and stubs (§77), the gates and batch TWN-4 (§78), the building
  tasks T34 to T41 with the order against E08's increment 13 (§79), the exit criteria (§80), the
  risks (§81).
- **The checkpoint** `TWN-Q18` (two rounds, seven questions; decisions 115–121, each as
  recommended but T-17: a `fix-optional` flag over the agent's `preserve`): the taverns' stream,
  T-16 and T-17 (`TWN-QUIRK-12`, `-13`, both `fix-optional`, on in `faithful`), TD-22 and TD-23 the
  reading, the views, the env, the cut and the exit criteria; the agent's calls of 122.
- **New ids:** dispute TD-23; quirk candidate T-17; quirks planned `TWN-QUIRK-12`, `-13`; stubs
  planned `TWN-STUB-17` (the obelisks row), `-18` (the guild's personality); rules planned
  TWN-TAV-007 to -009.

**The queue:** T33b out; T34 next, T35 to T41 queued. The adventure lane's inbox told the planned
`cross-lane` changes in its paths, the AI lane's and the server lane's theirs. No version moved.
Ids: questions next 19, decisions next 123, disputes next TD-24, quirk candidates next T-18.

**Verify:** docs only: the docs gates (`check-docs`, `check-rules`, `check-stubs`,
`check-exe-map`, `check-lane --base origin/master`, `doc-drift --base origin/master`); CI on the
pull request is the record.

### T34 — batch TWN-4: the writer's Tavern and Den, the harness's taverns and guild, three maps run twice, the scoring (2026-11-16)

One session, the user present at the start; branch `town/e09-t34`, `cross-lane` (the user's consent
at the start: `h3-map`'s writer is the adventure lane's, `h3-experiment` the battle lane's; the
capture harness is common). Starting point: `master` at 77c2eec (#969 merged; the setup's baseline
ran only the light gates: CI's fingerprint on `master`'s code). The user's computer was connected
at the start only: the client lane's 64 files and the four manuals and two help files staged (70,
SHA-256 checked, one digest of the sorted lists). **Kept whole** (decision 123; the agent
recommended three parts as E08 T119 to T121).

- **The writer** (`h3_map::writer`, `cross-lane`): `TAVERN_OBJECT` (95) and
  `DEN_OF_THIEVES_OBJECT` (97) among `PLAIN_OBJECTS`, no body; `tests/taverns.rs` reads them back
  in SoD and RoE (the masks as the game's: the Tavern entered west of its anchor, the Den at it)
  and imports them as blockers until T39a. `h3-experiment` (`cross-lane`): the pictures
  `AVXtvrn0`, `AVXdend0` in `ObjectSpec`'s docs, the tables' fixture, a build test. The
  experiments README's paragraph and its batch row.
- **The harness** (`clients/capture/`, common; README "Taverns and the Thieves' Guild"): the tavern
  dialog (vtable 0x643980: offers 5, 6; Recruit 12; the guild 11; close 30720; the status line 14,
  its texts TvrnInfo.txt's) and the Thieves' Guild window (0x643764: the flags by place and row,
  the columns' colours at +0x60, the parts' items 750 + c, 1/10/20/30 + c, the tactic's text,
  850 + c), both read live under Wine by two probes; the reads `offers:`, `offer:`, `pool`,
  `heroes:<colour>`, `tavern`, `guild`, `hovered`, `hero[<name>]:offer`, `:class`, `:returning`,
  `:owner`, `town:x,y:visitor:<detail>`, `:guild_spells`, `start_draws`; the actions `tavern`,
  `hover`, the hero screen's `dismiss`; a step `town = [x, y]` (a town from the town list, no
  hero); a step's `resources`, `mana`, `narrow_pool`; a field `<encounter>|<field>`. xdd.dll logs
  `tavern_class`, `tavern_type` and `leave_slot` with their ranges (the address map's three
  `*_draw` sites). Fixes: auto combat that ends before a human stack's turn (the stale active stack
  kept it pressing `a`), a portrait's selection frame read 0x7 after a dialog closed. A stale
  `xdd.dll` now fails the sequences' count of redirected calls: rebuild it with `setup.sh` (the
  other lanes' inboxes told).
- **The specs and recipes:** `experiments/specs/E09-batch-4a.toml` (stamp in its template; rows 1,
  2, 6, 8: four Castles of Red's with the Tavern and the Mage Guild, eight weeks by the written
  date, then three hires and the Recruit button), `-4b.toml` (rows 3, 4, 5: the date written to day
  6, a retreat in Red's turn and one in Blue's, a surrender, a loss to 30 Black Dragons, a
  dismissal from the narrowed pool, two hires), `-4c.toml` (row 7: R1's Tavern from the list, four
  neutral towns with Taverns taken, the Den of Thieves; the resources written); recipes
  `clients/capture/batches/E09-batch-4a.toml` (19 steps, about 2 minutes), `-4b.toml` (13 steps,
  about 1.5), `-4c.toml` (6 steps, about 1.5).
- **The runs:** two of each, recorded (`experiments/results/E09-batch-4a.md`, `-4a-run2.md`,
  `-4b.md`, `-4b-run2.md`, `-4c.md`, `-4c-run2.md`); 4b's and 4c's runs the same record for
  record (the game's generator reseeded alike), 4a's offers drawn anew. Probes (not recorded): the
  tavern dialog's and the guild window's items, the hero_in after the tavern, 4a's eighth week
  alone.
- **The scoring** (survey §14): every row as the reading in every read of both runs: slot 0 of the
  faction over 1-12, slot 1 another class over 1-94 (all 32 slot-1 draws: the Conflux classes
  weigh; TWN-4-8), full armies; the hire on the entrance, the guild's spells with a book, the
  replacement at once with one creature over 94 or 95; Recruit refused with a visitor (TvrnInfo 2)
  and at 8 on the map (TvrnInfo 1), the garrison hero counted in neither; TD-18 (the day-7
  fugitive kept, the day-6 one released and here drawn again with a full army); the surrender's
  army kept, the loss's type free with 1 creature, 2 000 points over the cleared army and 10
  spell points (T-16's reading); T-14 (a dismissed hero's artifacts back with it); TD-23 and T-17
  (the best hero by army power, the artifacts row by the backpacks, the army row and the best
  creature with R1's garrison). **Not predicted, read in the code:** `new_day` clears bit 17 of
  every hero record after the week (0x4C80CF) and gives every record its day's spell points
  (0x4C8237): for T35's checkpoint (design §63, §67, §68; recommended as the original).

**The queue:** T34 out; T35 next (with the two points). The adventure, client and map-client lanes'
inboxes told the harness's change (rebuild `xdd.dll`), the adventure lane's the writer's two
classes. No version moved. Ids: questions next 19, decisions next 125, disputes next TD-24, quirk
candidates next T-18.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; `cargo xtask
check-lane --cross-lane`; `cargo test -p h3-experiment`; CI on the pull request is the record.

### T35a — `shared/`: increment 4's drafts into the ADRs, the glossary and the security section (2026-11-16)

One session, the user present at the start; branch `shared/e09-t35a`, `cross-lane` (the ADRs and
`docs/03`, `docs/06` shared; the adventure lane's inbox common; T35 planned `shared/` and
`cross-lane` at `TWN-Q18`). Starting point: `master` at a1d2c71 (#973, T34's, merged; the setup's
baseline verified the code, 9ccb68f4). The user's computer was connected at the start only: the
executable, the editor, both bitmap LODs, the two tars and the three saves staged (9 files, SHA-256
checked against the PC's), for T35b and T36. **Cut in two** at the start (`TWN-Q19`, decision 125:
T35a the docs, T35b the code), and batch TWN-4's two points the reading missed answered as the
original (decisions 126, 127).

- **ADR-0013:** points 14.5 (a tavern hero's backpack equipped at a fill; TWN-4-5's replacement
  with its Clover still in the backpack) and 16.5 (a hero off the map regains its day's spell
  points with every record: decision 127; the draft's "no spell point by the day" replaced).
- **ADR-0016:** point 3 (a hire's placement, the away record, the leave's hand-back, the
  dismissal's removal; TWN-4-2), point 4 (the kinds `Tavern` and `DenOfThieves` after E09's kind
  `Town`; TWN-4-7's Den), point 6 (after E09 increment 3's view); a Consequences line (the kinds'
  bumps, the adventure state hash unmoved) and an Alternatives line (decision 106).
- **ADR-0017:** points 1 (the two commands, the pool's records, away records, factions, slots and
  marks, the draw; TWN-4-1, -2, -5, -6, -8), 2 (the tavern heroes' view and events, the guild's
  query and view; TWN-4-7), 3 (the taverns' week and the start's fill; **every day's start clears
  the marks** and **the day's spell points reach every record**, decisions 126, 127; the turn's
  points and the turn end's clear; TWN-4-1, -3), 4 (the taverns' stream, the replacement's and a
  return's draws; TWN-4-2, -4), 6 (the tavern decision), 8 (the env), 9 (the versions), 10 (a
  Den's outcome, a hire's application; TWN-4-2's guild spells), 11 (a hero that leaves the game;
  TWN-4-3 to -5); a Consequences and an Alternatives line. Each part after E09 increment 3's, so
  before E08's increment 10 to 13 parts where the points interleave (T122 merged first).
- **ADR-0028:** points 5 (the weights and the price; TWN-4-1, -2, -8), 6 (`TownCommand::Hire`;
  TWN-4-6), 8 (replaced: the pool's tavern heroes the game's, the guild's level by the taverns;
  TWN-4-7), 9 (the versions); the header's amendments, a Consequences and an Alternatives line.
- **Corrected against the design note:** the drafts' ADR-0017 and ADR-0028 point 9 named an
  adventure state hash bump for the kind `Tavern`; §77's table has none (the kinds hold no state),
  so both points say so (decision 128). The shared change of T35b moves no version (a flag moves
  the profiles' ruleset hashes, not `RULESET_HASH_VERSION` nor the pack format: E08 T115b's Sirens
  flag); the cut's question had said "v54", corrected in `TWN-Q19`'s record, decision 125 and the
  queue.
- **A throwaway check** found every draft sentence verbatim in its ADR but those that cite a batch
  row, carry decision 126 or 127, or name a moved anchor ("below" → "point 4", ADR-0016 point 4
  for the Tavern's hiding). The drafts deleted.
- **Also:** `docs/adr/README.md`'s status and Read-when cells (0013, 0016, 0017, 0028);
  `docs/03-domain.md`'s Hero pool amended and five terms (tavern heroes, away record, returning
  mark, Thieves' Guild, Den of Thieves); `docs/06-security.md`'s section "The Tavern and the
  Thieves' Guild (E09 increment 4)"; the design note's §61, §68 (the day's clear, the regain), §75
  (the tavern views' spell points told as an own hero's: `SRV-STUB-6`'s gap, for T37a), §77, §79,
  §80 item 5 and the epic's exit criterion 5; the survey's TD-20 and §14; the adventure lane's
  inbox (the ADRs in its paths, the two decisions for T37a's `state/day.rs`).

**The queue:** T35 → T35b (the code: `StreamId::TAVERNS`, the two flags, the packs' profiles, the
replays' `ruleset_hash`), T37a's entry gains the day's clear. Ids: questions next 20, decisions
next 129.

**Verify:** `cargo xtask check-docs`, `cargo xtask doc-drift`, `cargo xtask check-lane
--cross-lane`, `cargo xtask verify --scope town --summary`; docs only, so CI runs `check-lane` and
the docs gates; the pull request is the record.

### T35b — `shared/`: the taverns' stream, the flags of `TWN-QUIRK-12` and `-13`, the profiles' hashes (2026-11-17)

One session, the user present at the start; branch `shared/e09-t35b`, `cross-lane` (`h3-kernel`
and `h3-content` shared; the harvester and the packs the battle lane's, the golden replays the
adventure lane's, the bot reports the AI lane's, one test literal the server lane's). Starting
point: `master` at 4f4fbbc (#976, T35a's, merged; the setup's baseline verified the code,
e3cedff5, after a VM restart). The user's computer was connected at the start only: the four
LODs, both executables, the two tars and the three saves staged (11 files, SHA-256 checked against
the PC's), for T35b and T36. **Kept whole** with the `cross-lane` consent (decision 129); T36 after
it in the same session (decision 130).

- **`h3-kernel`:** `StreamId::TAVERNS = Self(8)` in the closed list (decision 115), the two tests'
  named lists extended; KER-RNG-001's summary names the streams 6 to 8 (it stopped at 5).
- **`h3-content`:** `quirk.town.leave_points_over_cleared_army` (`QuirkFlag(42)`,
  `LEAVE_POINTS_OVER_CLEARED_ARMY`, T-16) and `quirk.town.guild_artifacts_by_backpacks` (43,
  `GUILD_ARTIFACTS_BY_BACKPACKS`, T-17): the registry's 44 positions, its two tests; read from
  T38b and T39b. No encoding changed: `RULESET_HASH_VERSION` 53 and the pack format v52 kept.
- **`h3-pack`:** `the_harvested_flags_of_e09_increment_4` (on in `faithful`, off in `strict-fixed`,
  every expansion). **The harvester's** two profiles and `packs/base/core.toml`'s two profile
  lines; the harvester re-run on the staged installation wrote the same packs (the synthetic packs
  name no town flag, as before).
- **The ledger:** `TWN-QUIRK-12` and `-13` entered (`quirks-town.md`; confirmed by batch TWN-4's
  TWN-4-4 and TWN-4-7; their regression tests with T38b and T39b; decision 131).
- **Replays and records:** the base faithful ruleset hash moved 0xb5c01c8736eb73f9 to
  **0x6ba8109889b34a25** (the flags' bits; `strict-fixed`'s 0xb0c378a75ff70a5b and the synthetic
  0x876dc0d92f71fae2 kept: the bits off and no town flag). The 86 golden adventure replays on it
  re-recorded from their commands by a scratch script: only `ruleset_hash` moved; the three
  strict-fixed ones and the ten battle replays unchanged; all 99 verified (`h3 adventure verify`,
  `h3 verify`). `h3-server`'s record literal; the AI lane's `bots-report-v1.json` and
  `bots-neutral-v1.json` regenerated: only `engine.core` moved.
- **Also:** the battle and adventure lanes' HANDOFF version lines; the adventure lane's inbox (the
  replays' header in its paths).

**The queue:** T35b out; T36 next (its cut confirmed, decision 130). Ids: decisions next 132.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`, `cargo test -p h3-kernel -p
h3-content -p h3-pack -p h3-harvest -p h3-server -p h3-protocol`; CI on the pull request is the
record.

### T36 — the town part v7: the tavern weights and the hire's price (2026-11-17)

The same session as T35b, the user present at the start; branch `town/e09-t36` on T35b's
(`shared/e09-t35b`, #979), `cross-lane` (the harvester and `packs/README.md` the battle lane's, the
golden replays the adventure lane's). **Kept whole** (decision 130); the original data staged at
T35b's start.

- **`h3-towns`** (design §60): `TownDef::tavern` (class key, weight), compiled to
  `TownRecord::tavern` (a `u8` per core class by `HeroClassId`, a class not given 0);
  `TownPatch::tavern` changes or adds the classes it lists; `TownParamsDef::hire_price` and
  `TownParams::hire_price` (core resources as a building's cost: each once, at least 1, empty for
  free); the refusals `UnknownHeroClass`, a class outside the pack's closure, a class given twice
  (a record's or a patch's), a record whose weights sum to 0 after the overrides, the price's
  resource twice or 0. **Town ruleset part v7** (`TOWN_RULESET_HASH_VERSION`; the price after
  `most_heroes`, each record's weights after its demoted Capitol), pins `0x0f14_27ea_a4ad_9c09` and
  the empty part's `0x2bc5_8221_66bf_4786`.
- **`h3-pack-towns`:** `hire_price = { gold = 2500 }` in `[town_params]`, a record's
  `[town.tavern]` (class = weight), an override's `tavern = { … }`; the format's doc. The synthetic
  packs: `hire_price = { coin = 2000 }`, `vale`'s Warden 7 and Sage 3, the expansion's override
  the Sage's 4.
- **The harvest** (`h3-harvest`, `cross-lane`): `heroes::class_taverns` (the columns after the
  skills' weights, one per town); each record's faction column over its pack's closure (class i is
  faction i / 2's; a 0 left out); the AB pack's overrides of the eight base records with the
  Planeswalker's and the Elementalist's 5 (the base pack cannot name them); the gap-fill's
  `[towns.tavern]` (`hire_price = 2500`, source `exe-hero-price`: `hero_price` 0x67814C). The
  harvester re-run on the staged installation wrote `base` and `ab`; `sod` unchanged.
- **Tests:** `h3-towns`' compile, refusal, message and hash tests; `h3-pack-towns`' synthetic record
  and `the_original_taverns` (every SoD record 18 classes of 5 or 6 summing to 100, Castle's column
  as survey §2.2, the base pack alone 16 classes summing to 90, the price 2 500 gold in both);
  `h3-harvest`'s town test (the fixture's columns 4 to 6 with a 0: the records, the overrides).
- **Replays:** the 89 golden adventure replays re-recorded from their commands (only
  `town_ruleset_hash` moved, 0x0d7be940b64312a6 to 0xabc9756b93a29a55; all verify). The bot reports
  record no town part: unchanged.
- *Agent's calls* (decision 132): the overrides by class; the harvest from `H3bitmap.lod`'s file
  (`H3ab_bmp.lod`'s byte-identical in this build); the price in the gap-fill; a free price
  allowed; TWN-TAV-001's entry left to T37a. Design §60's *As built*.

**The queue:** T36 out; T37 next (T37a the pool, the draw and the fills). Ids: decisions next 133.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test -p h3-harvest -p h3-protocol -p h3-server -p h3-cli -p
h3-sim -p h3-game`; CI on the pull request is the record.

### T37a — the pool in the game: the draw, the fills, the turn, the views (2026-11-17)

The user present at the start; branch `town/e09-t37a`, `cross-lane` (the adventure lane's
`h3-game` state, setup, commands, observation, `h3-heroes`, `h3-map`'s told-views property and
the golden replays; the server lane's `h3-protocol` and `h3-server`'s record test; the client
lane's `h3-view` test literal), with `shared/e09-t37a-tavern-view` (#984: ADR-0017 point 2,
`docs/06-security.md`) merged first. Starting point: `master` at 2f6ed77 (#980, T36's, merged; CI's
green run stood in for the setup's baseline, lost to a VM restart). The user's computer was
connected at the start only: the four LODs, both executables, both tars, the three saves and the
manuals and help file staged (15 files, the 11 data files SHA-256 checked). **Kept whole**
(decision 133, over the recommended two parts). Two questions on the way (`TWN-Q20`): what the
tavern shows (the user asked for a check first: the held hero screen, decision 134) and the
events' shape (decision 135).

- **The pool** (`town/pool.rs`, design §61): `HeroStatus::Offered`; per type its record and away
  record (`Record` beside the small `Entry`: status, mask, class); per player a `Tavern` (faction,
  two slots naming types, their returning marks); the build's records and factions kept
  (`record_heroes`, `set_factions`); `Away`'s `stale` and `full` flags. **Game state hash v33**
  (the pool's encoding, the taverns' stream after the towns').
- **The draw** (`town/tavern.rs`, TWN-TAV-001, design §62): `draw` by the weights over the
  classes with a count, the avoided class, own faction only; `HeroPool::class_counts`,
  `nth_free`.
- **The fills** (TWN-TAV-002, design §63): `at_start` (in `GameState::new` before the first
  group's turn, each `Random` choice no start read drawn first) and `weekly` (after `grow_week`,
  before the adventure's week step); per slot the type offered, a new record (`hero_details`' defaults,
  `Progression::record`'s seed), the backpack equipped (`h3_heroes::backpack_equipped`, ADR-0013
  point 14.5), the spell points full, the full starting army (`starting_army`, `wear_machines`).
- **The turn** (TWN-TAV-005, design §68): `start_turn` derives a stale tavern hero's points;
  `clear_marks` at `EndTurn` and every day's start; `regain_mana` skips the full records.
- **The views and events** (design §75, decisions 134, 135): `GameObservation::tavern`,
  `TavernHeroView` (`TavernHero`, army, skills, artifacts: the worn ones and the backpack's first
  `TAVERN_BACKPACK_SHOWN`), `GameEvent::TavernOffered` and `TavernPartSeen`; **protocol v70** (the
  wire's types, the mapping, the fold; the fold gate's part *tavern heroes*, its *spell points*
  part over the tavern views under `SRV-STUB-6`); `h3-map`'s told-views property folds them.
- **Also:** `h3_heroes::positions`' `first_free` and `fits` iterate the positions without
  collecting them; `setup::hero_details` takes a `HeroDetailsSetup`; the level seeds a `Vec`.
- **Tests:** `town/tests/tavern.rs` (the draw's weights, avoidance, own faction, masks, two draws
  and none; the start's fill and its records; a `Random` choice drawn at the fill; the week's
  release and the records' ids; TD-18's spared slot and the marks' clears; the turn's points and a
  fork's shared pool; the backpack equipped and the spell points; the views, the week's events
  folded, an emptied slot; the backpack's shown places; a fork's week); `h3-heroes`' backpack test;
  `h3-protocol`'s round trips of the observation and every event; four town tests' expectations
  (a type offered at the start, the store's tavern records).
- **Catalog:** TWN-TAV-001, -002, -005 `verified` (batch TWN-4's TWN-4-1, -3, -8); TWN-SET-001,
  -005, GAM-CAL-002, GAM-HER-001, -002 amended.
- **Replays and benches:** the 89 golden adventure replays re-recorded from their commands (only
  `game_state_hash_version`, the checkpoints and `final_hash` moved); `h3-server`'s record literals;
  the bot reports unchanged (no part moved). `bench-check`: measured right after the first build,
  the week-start workloads +25 to +186 %; one design pass (the regain's skip, the stale points, the
  positions without allocation, a new record's spell points once, the seeds a `Vec`) brought them
  to +99 % (`day-start-builds-36`), +63 % (`day-start-towns-36`), +17 % and +13 % (the XL day
  starts), +34 % (`month-start-builds-36`) on `master` measured in the same sandbox, every other
  counted function the same code: re-baselined with the reasons; `fork-towns-36` -5.9 % (the
  heap's state) re-baselined; the other 28 inside the gate.
- *Agent's calls* (decision 136): design §61 to §63, §68 and §75's As built.
- **CI:** its first run met two things of `master`'s: `check-exe-map` (and xtask's
  `the_real_map_passes`) on 14 addresses E24 M4 T1a cites without a map entry (`master`'s CI had
  skipped the full verify for that code): twelve `docs/exe/map.toml` entries from E24's §5.1, the
  map-client lane's inbox told; the Clients job's `h3-ui` build error, red on `master` too. And one
  of this branch's: the fold of `TavernOffered`'s implicit `else` uncovered (fixed).

**The queue:** T37a out; T37b next. Ids: decisions next 137, questions `TWN-Q21`.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test -p h3-protocol -p h3-server -p h3-sim -p h3-cli`, the
clients' check; CI on the pull request is the record.


### T37b — the map's customised hero types (2026-11-17)

The user present at the start; branch `town/e09-t37b`, `cross-lane` (the adventure lane's
`h3-game` setup, state and test modules, `h3-map`'s import, version and tests, the 89 golden
replays; the server lane's `h3-server` record test; the ai lane's census literal). Starting point:
`master` at 8944673 (#991, E08 T124b1, merged; CI's fingerprint on `master`, so the setup ran the
light gates). The user's computer was connected at the start only: the four LODs, both
executables, both tars, the three saves, the three manuals and two help files staged (16 files,
SHA-256 checked). A census of the shipped maps first (a throwaway example over `h3-map`'s reader):
`Marshland Menace`'s 15 customised Fortress types, its five players' main towns all generating a
hero, its Fortress-only player 5 owning its one random hero; `Last Chance` and its Allies copy
Elleshar's sex. A short code reading (`load_hero_body` 0x5022C0 to 0x502420; the order in
`new_game_bans` and `new_game_heroes`) and VCMI 1.7.5's `CGameState::placeStartingHero`,
`initHeroes`, `CMapLoaderH3M::readHero` and `CGHeroInstance::pickRandomObject` before the
questions (`TWN-Q21`): every hero of a customised type takes its entry (decision 137; VCMI
differs for a random hero, TD-24, the original followed), a record per such free type at the
build (decision 138: the open order settled as the build's, the seed first), T37b whole (139).

- **The setup** (`setup.rs`): `GameSetup::custom_types`, `HeroDetailsSetup::over` (field by
  field, idempotent), `GameSetupError::CustomType` (not the ruleset's, or named twice) and
  `CustomDetails` (the entry on its own at experience 0 breaks the hero model), checked after the
  banned sets and before any draw; the heroes' count bound counts one per customised type. Each
  hero's details its own over its type's entry (a random hero's by its drawn type, a main town's
  hero's); `town::custom_records` (each customised type still free after the build's heroes, in
  type order) gives the records after them, their default experience in the build's pass, no
  army; `Prepared`'s last part each store hero's customisations as built.
- **The game** (`state.rs`): the level seeds, the sexes and the levels at load read the built
  customisations (the records last, each seed before its levels). No new state: the game state
  hash's encoding is unchanged (v33); a draw finds a record made.
- **The import** (`h3-map`, **map import v25**): the map-wide entries that set a field the game
  keeps (a biography alone left out), as a placed hero's customisations over a body that sets
  none; a type code past the table refused; `hero_type_of` factored out of `hero`. The placed
  heroes' bodies still merged over their entries.
- **Tests:** `town/tests/custom.rs` (a free customised type's record at the build and its offer
  as set, the not-allowed and the prisoner's types without one, the placed heroes' merge; a main
  town's hero and a random hero by their drawn types; `faithful`'s levels at load from the
  record's seed by the levels oracle, `tests::levels::oracle` made `pub(crate)`; a banned starting
  skill kept by a tavern hero, design §70's pin; the checks); `h3-map`'s
  `the_map_wide_entries_that_set_a_field_are_the_setups_customised_types`.
- **The census, after:** `Marshland Menace` builds 13 or 14 records at level 3 (2 000), its
  Fortress player's main town's hero and random hero at 2 000 whenever they draw a customised
  type (three of four seeds).
- **Catalog:** GAM-HER-001 (a code reading and a VCMI source added), GAM-HER-002, GAM-SET-002,
  TWN-SET-005, -008, -009 amended. The survey's §5 and TD-24, `docs/exe/map.toml`'s
  `load_hero_body` and `map_custom_heroes_read`, `h3m.md`'s import.
- **Replays and reports:** the 89 goldens re-recorded from their commands: `import_version` 25 in
  each, and `e08-army-12`'s `army-meeting-s1` and `army-dialog-s1` moved their checkpoints and
  final hashes (the fixture maps' engine-written entry of type 3, which that map does not place,
  now makes a record at the build; `TRAPS.md`); `h3-server`'s record literals; the bot reports
  unchanged.
- **Benches:** the scoped verify's `bench-check` found `reach-xl-144` -3.23 % and
  `reach-lifted-xl-144` -5.40 %: the XL map does not place type 3 either, so its record's default
  experience moves the setup stream's later draws (the map objects' contents) and the random games
  changed (3 007 steps and 6 200 queries for 2 633 and 5 656). Against `master` measured in the
  same sandbox (`target/wt-master`): -3.27 % and -5.05 %, every counted function the same shape a
  few per cent lower (`Reach::search`, `passage`, `expand_layered`, `tile_object`): re-baselined
  with the reason; the other workloads inside the gate (the build is not counted). The same
  change of the XL map's games moved `h3-protocol`'s `game_bounds` test off its whole map's reveal:
  its XL seed 13 (5 before; a throwaway search over seeds 1 to 24), the server lane told.

### T38a1 — the town's hire, the replacement, `HeroHired` and its parts (2026-11-17)

One session, the user present; branches `town/e09-t38a1` (`cross-lane`: the adventure lane's
`h3-game` `command.rs`, `observe.rs`, `state.rs`, `lib.rs`, `h3-files`' replay word, `h3-sim`'s
replay version, the 89 goldens' format line, `h3-map`'s told-views arms, `h3-cli`'s literals; the
server lane's `h3-protocol`; with the user's consent) and `shared/e09-t38a1` (#999: ADR-0016
point 6, ADR-0017 point 2, `06-security.md`, a sentence each). Starting point: `master` at
7a0da76 (#995, #996, #997 merged; the setup found CI's fingerprint, the light gates green). The
user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the three saves, the three manuals, two help files; SHA-256 checked
against the PC's). **T38a cut in two at its start** (decision 140, `TWN-Q22`): T38a1 the rule,
T38a2 the listing; the agent then moved the replay word into T38a1 (the server journals every
command the wire takes). **The owner told what the slot's view lacked** (decision 141). The
agent's calls: decision 142.

- **The town's half** (`h3-towns`): `TownCommand::Hire { town, slot }`, `TownCommandError::NoTavern`,
  `UnknownSlot`, `EmptySlot`; `check_hire` (`src/tavern.rs`: a standing Tavern, then the part's
  `hire_price`).
- **The adventure's half** (the town lane's `src/town/hire.rs`): `check_tavern_hire` (the town,
  the room below the most heroes with the garrison heroes not counted, then any hero on the
  entrance), `place_hired` (a hero's arrival on the entrance, the tile's viewers); `rows.rs`'
  `town_entrance` and `arrive` made `pub(super)`.
- **The game** (`town/hire.rs`): `hire` (design §64's checks and application), `replace`
  (TWN-TAV-004: the draw with the other slot's class avoided and any faction from the hirer's
  stream, `GameState::taverns_for`; a new record by `tavern::record_of`, factored out of the
  fill; spell points full; one creature of the first stack; no equip, no points), `HiredPart`
  (`Details { offered, positions }`, `Carried`, `Spell`; `HiredPart::of`); the pool's `hire` (the
  slot emptied, its mark cleared, the type `Taken` by the colour with its bit in the mask, the
  away record handed over); `GameEvent::HeroHired` (seen by the tile's viewers, the slot `None`
  to another viewer by `project`) and `HiredPartSeen` (the owner alone); then `tavern::tell` of
  the changed slot. `TWN-STUB-15` (the guild's spells) and `TWN-STUB-19` (not listed) registered.
- **The wire** (protocol **v71**): the command both ways, the three refusals, both events and
  `HiredPart`; the fold's `hire` (the hero among every viewer's heroes on the map; the owner's view
  from the slot's, its worn positions grown to the told count) and `HeroDetails::see`.
- **The replays:** the adventure replay format **v17** (`town hire <town> <slot>`); the 89 goldens'
  `format` line only (no golden hires); `h3-cli`'s and `h3-sim`'s literals.
- **Tests:** `h3-towns`' `a_hire_needs_a_standing_tavern_and_costs_the_parts_price` and the
  refusals' messages; `h3-adventure`'s `town/hire/tests.rs` (2: the checks in order, the placement
  in id order with nothing revealed); `h3-game`'s `town/tests/hire.rs` (5: the placement with the
  away record, the pool's statuses and the observation; the replacement against the draw on red's
  stream with one imp of a 2 to 3 stack; the events, the parts and who sees them; each refusal
  changing nothing, `NotYourTurn` included; no type free leaving the slot empty, a sink keeping
  nothing reaching the same state); `h3-protocol`'s round trips (119 rejections, 43 kinds of
  event), `a_hire_folds_as_observe_shows_it` (a hire on three golden games, every viewer's fold
  equal to `observe`) and `a_hire_moves_the_owners_slot_view_into_its_heroes` (by hand: a worn
  artifact, the backpack's first places and its rest, a spell, another viewer). The first scoped
  verify's coverage found the `Spell` part's closure unrun (no test hero had a spell):
  `GameState::set_spells` (tests) and the parts test's spell.
- **Catalog:** TWN-TAV-003 and -004 entered `verified` (the code reading, batch TWN-4's rows
  TWN-4-2, -5 and -6, VCMI, the manual).

`master` merged in once, after #998 (E08 T125a, no version moved) and #999 (the `shared/` half).
CI's first full verify on #1001 found two regions of the new fold unrun (`h3-protocol`, which the
town scope's coverage does not gate: an `if let`'s untaken path, the worn positions' resize test):
both written without a branch, `verify-step coverage --scope server` clean (`TRAPS.md`).

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` (every step but
`check-lane`, as on a `cross-lane` branch, and the coverage miss above, then the coverage step
alone clean; `bench-check` inside every gate, `original-data` passed); the tests of
`h3-cli`, `h3-protocol`, `h3-server`, `h3-sim`, `h3-map` and `cargo clippy --workspace
--all-targets` on their own; CI on the pull requests is the record.

### T38a2 — the hire listed, the env's `TAVERN_HIRE` v12, the plans hire (2026-11-18)

One session, the user present; branch `town/e09-t38a2` (`cross-lane`: the adventure lane's
`h3-game` `legal.rs`, `h3-map`'s `tests/play.rs` and `tests/view.rs`, `h3-sim`'s
`src/adventure/` (action encoding v12, the rollout's docs, the tests, the town-map seeds), the
adventure workloads in `benches/adventure/`; with the user's consent). Starting point: `master` at
52a3cb5 (#1001 merged; the setup found CI's fingerprint, the light gates green). The user's
computer was connected at the start only: 16 files staged (the four LODs, both executables,
both tars, the three saves, the three manuals, two help files; SHA-256 checked against the PC's).
The agent recommended a cut in two (the listing with the plans and every oracle, the random agent
skipping the listed hires as decision 56 did; then the env); **the user kept T38a2 whole**
(decision 143). The agent's calls: decision 144.

- **The listing** (`h3-game`'s `town/offer.rs`, `town/hire.rs`): `legal_builds` gives each own
  town's builds, recruitments, **hires**, then hero moves; `hire::check` (the hire's checks in
  order, no change, no draw) serves `hire`; the listing's `hire_slots` (once a listing: the part's
  price in the treasury, room for another hero on the map, the slots holding a hero) and
  `legal_hires` (a town: its entrance free, then its standing Tavern) test the same conditions,
  the cheapest first. The adventure's `has_room` and `entrance_free` (`src/town/hire.rs`) are
  `check_tavern_hire`'s room and entrance. `legal_commands` folds its chain into a list sized 16
  (the bench below). `TWN-STUB-19` lifted; TWN-TAV-003's catalog entry names the listing.
- **The env** (action encoding **v12**): `TAVERN_HIRE` (35, 36), `TAVERN_SLOTS` (2),
  `ENTRY_COUNT` 37, `Action::tavern_hire`; the tile mask the entrances of the towns of the listed
  hires of the entry's slot, the decoding by the town at the tile (building 0), the encoding back.
  The mask property (`check_masks`' `check_town_heroes`) covers both entries; the layout test the
  numbers. ADR-0017 point 8 already held v12's entries (the design note's `HIRE + s`).
- **The town-map seeds** (`h3-sim`'s `tests/towns.rs`, a throwaway search in release): every game
  now hires 10 to 16 heroes; a siege against a visitor with the garrison merged met 8 games of
  the first 399, none with every other feature; seeds 1 to 2 400 gave ten, **2369** the shortest
  (505 steps, 24 builds, 39 dialogs, 11 hires) for the end of days; **6** for the forks (a siege
  by day 3). `Played` counts the hires.
- **The plans** (`h3-map`): `town_builds` probes the hires of slots 0 to 2 after the
  recruitments; `check_town_hire` reads a listed hire apart (the actor's tavern view's hero on
  the town's entrance as its visitor with the view's army, formation and tactics; the room and
  the entrance before; the part's price; `HeroHired` of the slot; the town's record unchanged);
  `Step::Town` hires a listed hero in one step of eight. Oracles taught: `check_isolation` (a
  hired hero joins the map), `check_conservation` (the price paid, the hero's army from its
  slot's view, the refill's record and points), the told views (`HeroHired` shows the hero on its
  tile to every viewer told; the owner's slot view folded with its `HiredPartSeen`s equals its
  details, `check_hired`), the hygiene (a hire the viewer is told ends the comparison: the hired
  hero is a draw of the taverns' stream). All of `h3-map`'s tests passed at once after the
  isolation and conservation changes; then the play and view blocks once at 240 cases.
- **Tests:** `h3-game`'s
  `the_legal_commands_list_each_hire_that_passes_after_the_towns_recruitments` (both slots after
  the recruitments and before the hero moves, each accepted; none in another player's turn, none
  without a Tavern, below the price, with no room or the entrance held; an empty slot's alone).
- **The bench** (`master` built in `target/wt-master`, per function): the first build cost
  `legal-towns-36` +106 % with the changed games; on master's games (the agent made to skip the
  listed hires, an experiment) the listing cost +17.1 %: `has_tavern` three times a town, the
  nested `Chain`'s `next` and the list's regrowths. One design pass: the player-wide tests once a
  listing, the Tavern last, the chain folded into a list sized 16 (8 measured worse): +4.6 % on
  master's games (`has_tavern` about 280 Ir a listing, the rule's own cost). Re-baselined with the
  random agent hiring (encoding v12; the same counted code, no new function): `legal-towns-36`
  +60.1 % (master 14 906 102 in the same sandbox), `reach-towns-36` +194.2 % (11 186 queries for
  3 554), `day-start-builds-36` +104.7 %, `day-start-towns-36` +51.1 %, `month-start-builds-36`
  +68.9 %, `fork-towns-36` +38.8 % (more heroes in the states); the other adventure workloads
  within their gates (`random-play-bonuses-36` +1.67 % the largest: the fold, the listing).
  CI's first full verify on #1006 was cancelled at its 40-minute limit in `bench-check`
  (`master`'s run takes 36): `reach-towns-36`'s 28 days ran 107 s under callgrind for 17 s
  with the hired heroes; its games cut to 10 days (2 418 queries, 10 s, 887 907 696 Ir). The
  second run was cancelled at 40 minutes too (the runner's apt step 10 minutes, coverage 21 for
  `master`'s 19 with the hiring plans): the user raised the job's limit to 55 (#1009, `shared/`).
- **Versions:** action encoding v12 alone; no protocol, replay format or hash moved (the 89
  goldens replay unchanged).

The play and view blocks at 240 cases found one hygiene case `master`'s oracles had not met:
another player's hire out of the viewer's sight in both games, the same record of another type in
the other game (the taverns' draw), its hero walking into sight later. `outcome` gained the
heroes a command hired as they joined the map (type, army, details): a hire that differs ends
the comparison (the case kept in `proptest-regressions/tests/view.txt`). Then both blocks at 480
cases: all 177 of `h3-map`'s tests passed. `master` merged in once, after #1002 (E08 T125b1's
nine goldens), #1003 and #1004: no conflict.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` (every step but
`check-lane`, as on a `cross-lane` branch, `fmt` (the new `h3-sim` test) and one coverage miss
(`hire_slots`' `?` on a player without a tavern entry: written without the branch, as `check`
reads the slots); then `fmt` and `verify-step coverage --scope town` clean; `determinism`,
`bench-check`, `original-data` passed); `cargo test -p h3-cli -p h3-protocol -p h3-server`,
`-p h3-sim`, `-p h3-map`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask
check-lane --cross-lane`; CI on the pull request is the record.

### T38b1 — a hero that leaves the game: the away record back, the returns (2026-11-18)

One session, the user present; branch `town/e09-t38b1` (`cross-lane`: the adventure lane's
`h3-adventure` `state.rs` (`end_battle`), `h3-game`'s `state.rs`, `state/battle.rs`,
`tests/aftermath.rs`, `tests/battles.rs`, `h3-map`'s `tests/play.rs`, `h3-sim`'s town-map test,
the golden adventure replays and four adventure workloads; with the user's consent). Starting
point: `master` at d5e5742 (#1006 merged; the setup found CI's fingerprint, the light gates
green). The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the three saves, the three manuals, two help files; SHA-256 checked
against the PC's; its shell dropped soon after, nothing more needed). **The user cut T38b in two**
(decision 145, `TWN-Q23`, as recommended): T38b1 the leaves and the returns, T38b2 the dismissal.
The agent's calls: decision 146.

- **The adventure's half** (`cross-lane`): `AdventureState::end_battle` returns the heroes that
  left (`Vec<HeroOnMap>`, in the order given, as each stood before the end); `release_town_hero`
  and `end_town_hero_battle` the garrison hero (`Option<HeroOnMap>`, the town lane's
  `src/town/rows.rs`).
- **The leave** (`h3-game`'s `town/leave.rs`, new; design §67's As built): `leave` by
  `Leave::{Retreated, Surrendered, Gone}` and `after_battle`, last in the game's `end_battle`
  (after the winner's levels, the original's order); the capture's empty garrison hero `Gone`;
  `GameState::free_hero_types` removed. The pool's `leave` and `return_to` (`pool.rs`), its
  `is_offered`. A `Gone` leave: one creature, the spell points full, the land maximum over the
  cleared army under `TWN-QUIRK-12` (the test pack's new `t:leave` profile) or over the one
  creature. A return: the slot from the owner's stream, the marked slot's other, the slot's hero
  free, the mark set, the survivors or one creature, the points kept and stale; told to the owner
  last (`tavern::tell`). Design §67 point 5 (a new record's replacement takes the land points of
  its one creature) built in `hire::replace`.
- **Catalog:** TWN-TAV-006 new (`verified`: batch TWN-4's TWN-4-3, -4 as the reading);
  GAM-BAT-003, TWN-TAV-004 and TWN-SET-005 amended; `TWN-QUIRK-12`'s regression test named;
  `ADV-STUB-9`'s reason (the Tavern fact ready).
- **Tests:** `town/tests/hire/leave.rs` (five: a loss under both flag settings, a draw and a
  created-stacks winner, a retreat with its slot, view, events, turn start and rehire, a
  surrender with the marks and without survivors, a repeated type's other hero beside an offered
  record); the capture's away record in `rows.rs`'s
  `TWN-QUIRK-11` test; the replacement's points; the older tests' beaten heroes refilled (the
  aftermath's and the siege's spell points, the surrender's events). The town tests' setups place
  two heroes of type 0: a leave of the type's second hero makes it the record (decision 146).
- **The plans:** `check_heroes_after_battle` lets a hero that leaves without returning end at its
  full spell points (untold); a return is named by its owner's `TavernOffered`. Every `h3-map`
  test passed (the hygiene ends at the viewer's battle, before any return it could see).
- **The bench** (`master` in `target/wt-master`, all 34 adventure workloads in both trees, one
  CPU each): the same shape per function, `town::leave` in none; the town map's random games
  changed course: `reach-towns-36` +3.82 % (master 887 794 829), `day-start-towns-36` +2.11 %,
  `month-start-builds-36` −2.39 %, `legal-towns-36` −1.73 %, re-baselined; every other workload
  within 0.8 % of master's.
- **Replays and seeds:** 33 of the 98 golden adventure replays re-recorded from their own
  commands (hash lines only); the bot reports unchanged; `h3-sim`'s town-map end-of-days seed
  **2541** (a release search over 1 to 3 000: ten met every feature, none returns a hero; 543
  steps, 28 builds, 58 dialogs, 13 hires). No version moved (the pool's hashed fields were there).
- **Verify** (`--scope town --with-original-data`): every step but `check-lane` (`cross-lane`:
  `check-lane --cross-lane` ok) passed once the leave's unreachable `None` paths were indexed and
  the repeated type's offered record got its test (coverage's two rounds); the protocol's fold,
  session and record tests over the new goldens passed.

### T38b2 — the dismissal: `Manage::Dismiss`, its refusals, the listing, the plans (2026-11-18)

One session, the user present; branch `town/e09-t38b2` (`cross-lane`: the adventure lane's
`h3-adventure` `state.rs` and `state/tests/sea_battles.rs`, `h3-game`'s `manage.rs`, `legal.rs`,
`state/manage.rs`, `tests/manage.rs`, `tests/exchanges.rs`, `h3-files`' replay word, `h3-sim`'s
replay format, `h3-cli`'s literals, `h3-map`'s `tests/play.rs`, the 98 golden adventure replays'
`format` line; the server lane's `h3-protocol`; with the user's consent). Starting point:
`master` at 8625fed (#1013 and #1014 merged; the setup found CI's fingerprint, the light gates
green). The user's computer was connected at the start only: 16 files staged (the four LODs,
both executables, both tars, the three saves, the three manuals, two help files; SHA-256 checked
against the PC's). **The user kept T38b2 whole** (decision 147, `TWN-Q24`, against the agent's
cut in two as T38a's). The agent's calls: decision 148.

- **The command** (`cross-lane`): `Manage::Dismiss { hero }` and `ManageError::{NotOnMap, InTown,
  LastHero}` (`h3-game`'s `manage.rs`, with their messages); `GameState::manage` refuses it with
  `DecisionOpen` unless no decision is open and hands it to the town lane's
  `town::leave::dismiss`. The adventure's `AdventureState::remove_hero((player, hero))`:
  `end_battle`'s removal (the marks, `HeroRemoved` to the owner and the tile's viewers, the boat
  freed) made one private `remove_at`, shared; `None` for a hero not the player's on the map.
- **The dismissal** (`h3-game`'s `town/leave.rs`; design §69's As built): a garrison hero of the
  player's `NotOnMap`; for its hero on the map `check_dismissal` (`InTown` by `StaticMap::town_at`
  on its tile, `LastHero` without another own hero on the map and without an owned town;
  `TWN-STUB-16` marks it: the loss condition's hero); then the removal (`NotYourHero` by its
  `None`) and `leave(.., Leave::Gone)`: one creature, the spell points full, T-16's points, the
  type free with every artifact, no draw. Batch TWN-4's row TWN-4-5 saw exactly that.
- **The listing:** `legal_management` gives the dismissal last, outside a decision, for a hero on
  the map that `check_dismissal` passes (a garrison hero's listing none).
- **The replay word and the protocol:** `dismiss <hero>` (format **v18**; decision 148), the 98
  goldens' `format` line, `h3-cli`'s and `h3-sim`'s literals; protocol **v72**: `Manage::Dismiss`
  and `ManageReject::{NotOnMap, InTown, LastHero}` both ways, the round trips (122 rejections),
  `a_dismissal_folds_as_observe_shows_it` (three golden games, every viewer's fold of the first
  dismissal that passes against `observe`).
- **Catalog:** GAM-MAN-003 new (`verified`: TWN-4-5 as the reading, the refusals by the code
  reading, TD-22's dispute in its VCMI source); TWN-TAV-006 amended; `TWN-STUB-16` registered.
- **Tests:** `town/tests/hire/dismiss.rs` (three: the leave with every artifact, its type free,
  one creature, full spell points, no draw, told as the removal; every refusal with the listing
  beside it and the messages; a meeting holding the turn), the adventure's
  `a_removed_hero_leaves_the_map_with_its_boat_as_a_beaten_one`, `h3-files`' word and its
  malformed lines; the management listings' candidates (`tests/manage.rs`, `tests/exchanges.rs`)
  hold the dismissal.
- **The plans:** `managing` leaves the dismissals out (the other steps play as before);
  `dismissal` plays one in one `Step::Manage` of sixteen; `check_management` (its removal and its
  boat's, nothing else told, its record and artifacts kept), `check_conservation` (its record's
  spell points full untold, its army off the map) and `check_isolation` (the actor's hero off the
  map by its `HeroRemoved`) know it; `the_plans_meet_and_manage` counts the heroes dismissed. The
  properties of `play.rs` and `view.rs` passed at five times their cases and
  `every_step_keeps_the_commands_invariants` at ten (release), then at 48 again.
- **The bench** (the scoped verify's `bench-check`): every workload inside its gate, none
  re-baselined; `manage-army-12` +1.37 % (the listing's dismissal test, outside a meeting only:
  in a meeting one comparison), `random-play-bonuses-36` +1.67 % (as before this task, T125b2b's
  +1.66 %), every other within 1.3 %.
- **Verify** (`--scope town --with-original-data`): every step but `check-lane` (`cross-lane`:
  `check-lane --cross-lane` ok) passed once the three new refusals' messages got their test
  (coverage's two rounds); `h3-protocol`'s coverage (`verify-step coverage --scope server`) ok;
  `h3-protocol`'s and `h3-server`'s tests, `cargo clippy --workspace --all-targets` clean.

### T39a — the map's Tavern and the Den of Thieves, the tavern decision (2026-11-18)

One session, the user present at the start; branch `town/e09-t39a` (`cross-lane`: the adventure
lane's `h3-adventure` kinds (`ruleset/objects.rs`), `command.rs`, `state.rs`, `state/revisits.rs`,
`lib.rs`, `h3-pack-adventure`, `packs/base/adventure.toml`, `h3-map`'s `import/codes.rs`, `lib.rs`
and tests, `h3-game`'s `command.rs`, `state.rs`, `observe.rs`, `legal.rs`, `lib.rs`, `h3-files`'
replay words, `h3-sim`'s replay format, `h3-cli`'s literals, the 98 golden adventure replays; the
AI lane's env (`h3-sim`'s `adventure/action.rs`) and bots tables; the server lane's `h3-protocol`
and `h3-server`'s `tests/record.rs`; with the user's consent). Starting point: `master` at
34dddd8 (#1018 merged; the setup found CI's fingerprint, the light gates green). The user's
computer was connected at the start only: 28 files staged (the four LODs, both executables, the
five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, both tars, the three saves, the
three manuals, two help files, `README.TXT`, the two `goggame` files; SHA-256 checked against the
PC's). **The user kept T39a whole as queued** (decision 149, `TWN-Q25`, against the agent's
Tavern now with the Den moved to T39b) and consented to the `cross-lane` pull request with the
game state hash v34. The agent's calls: decision 150.

- **The kinds** (the adventure part **v22**): `ObjectKind::Tavern` (ADV-VIS-007; visited from a
  neighbour, not from above; hash tag 45) and `ObjectKind::DenOfThieves` (ADV-VIS-008; entered;
  tag 46), their `Effect`s and defs (`tavern = {}`, `den_of_thieves = {}`), a pack's Tavern
  entered or Den visited from a neighbour refused; the base pack's `base:tavern` and
  `base:den_of_thieves`, appended last among base's types (the AB and SoD types' ids moved by 2).
  Classes 95 and 97 typed by the import (**map import v26**).
- **The visits** (`h3-adventure`'s `visit`): the Tavern's outcome `AdventureOutcome::tavern`, an
  `h3_adventure::TavernVisit` (the hero, the type, the anchor, the visitable tile); the Den's
  nothing until T39b (`TWN-STUB-20`). The town lane's `town/hire.rs` in the adventure:
  `check_map_tavern_hire` (the room, then the tile, `TownHeroError`'s) and `tile_free`.
- **The decision** (GAM-DEC-010): `Decision::Tavern { owner, visit }` (hash kind 9 with the tile,
  **game state hash v34**), opened by `settle_outcome`'s `open_tavern` (`TavernOpened` to the
  owner) whatever the gold and room; `Answer::Tavern(TavernAnswer::{Hire { slot }, Close})`; the
  hire `town::hire::map_hire` (the slot, the room and the tile, the price:
  `GameCommandError::Tavern(TavernError::{UnknownSlot, EmptySlot, TooManyHeroes, Occupied})`,
  then `CannotPay`), applied by the town hire's own `place` (factored out of `hire`, the price,
  the pool, `HeroHired` and its parts, the replacement, the slot told) with the Tavern's tile and
  no guild spells, then `DecisionClosed`; `Close` only closes and is the default answer;
  `GameState::tavern`, `DecisionView::Tavern { hero, object_type, anchor }`; the listing
  `map_hires` (`hire_slots`' part, the tile free) then `Close`.
- **The wire, the env, the replays:** protocol **v73** (`Answer::Tavern`, `TavernAnswer`,
  `GameReject::Tavern`, `TavernReject`, `Decision::Tavern`, `TavernOpened` both ways, the fold
  opening the decision whole); the env's action encoding **v13** (`answer_entry`: slot 0 `FIRST`,
  slot 1 `SECOND`, `Close` `DECLINE`; no new entry); the replay format **v19** (`answer tavern hire
  <slot>`, `answer tavern close`). The 98 goldens re-recorded from their commands (only the
  version lines, the part hash, the checkpoints and `final_hash` moved), `h3-server`'s
  `tests/record.rs` literals with them; the bots tables' `engine.adventure` line (both
  regenerated, nothing else moved).
- **Catalog:** GAM-DEC-010, ADV-VIS-007 and ADV-VIS-008 written `implemented` (the code reading,
  the census, VCMI; batch TWN-4 had no map Tavern row); `TWN-STUB-20` registered.
- **Tests:** the ruleset's `ruleset/tests/taverns.rs` (the kinds, their effects, the refused
  visit modes, the hash), the pack's `the_base_pack_types_the_tavern_and_the_den_of_thieves` and
  the type count, the adventure's `a_map_tavern_s_hire_needs_room_on_the_map_and_its_tile_free`,
  `h3-game`'s `town/tests/hire/map.rs` (four: the decision opened beside the Tavern, told and
  projected, its view, its listing and default, `EndTurn` and another kind's answer refused, the
  close changing nothing else; the hire on the Tavern's tile with its events, the decision closed
  after them, no revisit, a later step meeting the hired hero; every refusal with its listing and
  the messages, `Occupied` with a hero placed on the tile while it is open; the Den entered and
  revisited), `h3-map`'s import cases and `tests/taverns.rs` (the writer's two classes import
  typed), `h3-files`' two words, `h3-sim`'s answer entries and version, `h3-protocol`'s round
  trips (126 rejections, 44 kinds of event, a Tavern view). The plans name the kind in `settle`
  and `hiring` (their maps lay out no Tavern).
- **The bench** (`bench-check --lane adventure` right after the first build): every workload
  inside its gate, none re-baselined; `random-play-bonuses-36` +1.65 % and `manage-army-12`
  +1.36 % (as at T38b2), every other within 1.3 %.
- **Verify** (`--scope town --with-original-data`): every step passed (coverage, determinism,
  bench-check, original-data) but `fmt` and `check-rules` (the VCMI sources' URLs and dates),
  both fixed after, and `check-lane` (`cross-lane`: `check-lane --cross-lane` ok). `h3-protocol`'s
  coverage (`verify-step coverage --scope server`) found the tavern part's fold for an empty slot
  unrun (T37a's `if let`, run on `master` by a played game this task's changes no longer reach;
  which was not traced): a fold test for it and the decision (`a_map_taverns_decision_folds_whole_and_a_stray_part_changes_nothing`),
  then clean. The tests of `h3-cli`, `h3-files`, `h3-sim`, `h3-protocol` and `h3-server` and
  `cargo clippy --workspace --all-targets` on their own. `master` merged in once (docs only:
  the lanes' inboxes, E08 T127b's planned versions, which this task takes first).

### T39b1 — the Thieves' Guild as a query, its resource rows the town part's (2026-11-19)

Baseline 450e47e (#1030, #1031, #1032 merged; CI's runs on `master` standing in: the setup's
baseline died with a VM restart before its first step). The user's computer was connected at the
start only: 28 files staged (the four LODs, both executables, the five DLLs, both `.snd`,
`H3ab_ahd.vid`, `battle-videos.vid`, both tars, the saves, the three manuals, two help files;
SHA-256 checked). At the start the user cut T39b in two (decision 151; the agent's
recommendation) and consented to `cross-lane` for both halves; during the build the user put the
guild's resource rows in the town part (decision 152, over the agent's recommendation).

- **The guild** (design §73's As built; TWN-TAV-007 to -009): `h3-game`'s `town/thieves.rs`,
  `thieves_guild` (in the viewer's turn by its towns' Taverns, counted to `GUILD_LEVELS` 5, or at
  a map Tavern's open decision at that count, 0 without one) and `thieves_guild_at` (the rows the
  level shows as places by player id; the parts by the level). The rows' values: the towns by the
  adventure's owners, the heroes on the map, the three resource rows of the part's
  `guild_resources`, the obelisks 0 (`TWN-STUB-17`), the artifacts under `TWN-QUIRK-13`, the army
  (`h3_adventure::power` of each hero on the map plus `army_value` of each own town's upper bar,
  TD-23), the income (`GameState::day_income`'s gold without the week's resources). The parts:
  the best hero by power from 0, its `h3_heroes::primary_skills` clamped to 99 from level 2, the
  best creature from level 4 over the towns' upper bars then the heroes' armies from 0 (read at
  T39b1: 0x5DE992's best starts at 0, `jle`; a stack of count 0 skipped); no tactic
  (`TWN-STUB-18`). Read again: `thieves_guild_row_value`'s resource cases (player +0x9C the
  treasury: gold index 6, wood and ore 0 and 2, the rest 1 and 3 to 5).
- **The town part v8** (decision 152): `TownParamsDef::guild_resources` /
  `TownParams::guild_resources`, three rows of core resources, each resource in at most one row,
  each row in the core's order (a `BTreeSet`: logic crates sort nothing); hashed after the hire's
  price; the pins moved. `h3-pack-towns` parses `guild_resources = [[…], […], […]]`; the
  synthetic pack's coin, shard and none; the base pack's harvested (`h3-harvest`, `cross-lane`:
  the gap-fill's `[towns.guild]` with `RESTYPES.TXT`'s indexes and its source
  `exe-thieves-guild`, written as the keys; a harvest test for an unknown index and a resource
  given twice). The regenerated packs moved by that one line.
- **`cross-lane`:** `h3_adventure::army_value` (split out of `power`) and `power` public
  (`answer.rs`, `lib.rs`); `h3_heroes::primary_skills` (`derive.rs`, with a test); the game's
  `day_income` factored out of `pay_income` (`state/day.rs`); the catalog's ADV-MON-007 and
  HER-ADV-001 a sentence each; `packs/README.md`'s towns row; the 98 goldens'
  `town_ruleset_hash` 0xabc9756b93a29a55 to 0xab3ee3f1cb243f30 (one re-recorded from its
  commands to read the value, then the line replaced in all; nothing else moves: the guild
  changes no state). `AdventureState::upper_bars` public (the town lane's `town.rs`).
- **Catalog:** TWN-TAV-007 to -009 written `verified` (the code reading and batch TWN-4's
  TWN-4-7; the map Tavern's level 0 on the reading alone); `TWN-QUIRK-13`'s regression test;
  `TWN-STUB-17`, `-18` registered, `TWN-STUB-20`'s reason pointing at T39b2.
- **Tests:** `town/tests/hire/thieves.rs` (nine, on the hire tests' game with the core edited by
  the hire tests' new `rules_in_core`: the query's turn, Tavern and count to five; the map
  Tavern's level 0 and its close; each level's rows and parts and the Den's; the places; each
  row's value, the income against the next day's payment, the resource rows by moving the one
  test resource between them; a garrison hero by its town's army; the first of two equal heroes,
  a greater replacing, Attack 110 shown as 99, every creature worth 0; the best creature from the
  towns, then the heroes; `TWN-QUIRK-13` both ways), `h3-towns`' rows and hash, `h3-pack-towns`'
  synthetic and original rows.
- **Verify** (`--scope town --with-original-data`): every step passed (fmt, clippy, the checks,
  coverage, determinism, bench-check, doc-drift, original-data) but `check-lane` (`cross-lane`:
  `check-lane --cross-lane` ok). The bench: every workload inside its gate, none re-baselined;
  `random-play-bonuses-36` +1.66 % and `manage-army-12` +1.36 % (as at T38b2 and T39a),
  `day-start-builds-36` +1.03 % (`pay_income` through `day_income`), every other within 1.3 %.
  `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets`, `h3-cli`'s
  golden replays and the tests of `h3-protocol` and `h3-server` on their own; the base pack
  regenerated by `h3-harvest` (one line moved).

### T39b2 — the Den of Thieves' visit, the guild on the wire (2026-11-19)

Baseline 8930a1b (#1034 merged; CI's run on `master` standing in: the VM restarted about 30
minutes in and the setup's baseline died mid-build). The user's computer was connected at the
start only: the client lane's 64 files (the four LODs, both executables, the five DLLs, the two
`goggame` files, `README.TXT`, both `.snd`, three `.vid`, both tars, the saves, the 40
non-campaign MP3s) and the four manuals with three help files staged, SHA-256 checked; its tools
went away about 75 minutes in. At the start the user kept T39b2 whole (decision 154, `TWN-Q27`,
as recommended).

- **The visit** (design §74's As built; ADV-VIS-008): `AdventureOutcome::thieves_guild`, set by
  `h3-adventure`'s `visit` for `Effect::DenOfThieves` (`TWN-STUB-20` lifted, `cross-lane`);
  `h3-game`'s `settle_outcome` tells it after the visit's other effects through
  `town::thieves::tell_den`: `thieves_guild_at(state, rules, DEN_LEVEL)` whatever the visitor's
  Taverns, nothing told or computed for a sink that keeps nothing (`Sink::RECORDS`).
- **The events** (design §75's As built; decision 155): `GameEvent::ThievesGuildSeen { player,
  level }` then `GuildPartSeen { player, part }` per `GuildPart` (`Place { row, of, place }` per
  shown row's player, the rows in order, then `Player { of, shown }` per player;
  `ThievesGuildView::parts`), seen by `player` alone (`project`); `h3-map`'s told views take both
  as the viewer's own (`cross-lane`).
- **The protocol v74** (`cross-lane`, the server lane's): the wire's `GuildRow`, `GuildRanking`,
  `GuildHero`, `GuildPlayer`, `ThievesGuild` (`at`, `see`: a client's fold of the parts),
  `GuildPart`; both events mapped (`map/game/event.rs`, `map/game.rs`' `guild_part`,
  `thieves_guild`) and dropped by the fold; `GameQuery::ThievesGuild` answered by
  `GameAnswer::ThievesGuild { guild }` (`map/game/query.rs`: `h3_game::thieves_guild` for the
  asking seat's player).
- **The census:** the 98 golden replays play 12 maps, none with a Den or a map Tavern (a
  throwaway test over `game_fold::golden`'s setups): no golden steps onto one, nothing
  re-recorded, no hash moved.
- **Docs:** ADV-VIS-008 and TWN-TAV-007 amended; ADR-0017 point 2's Den sentence (the parts) and
  `docs/06-security.md`'s guild line by `shared/e09-t39b2` (#1039, a revision-history line).
- **Tests:** `town/tests/hire/map.rs` (three: the guild at everything to red alone after the
  step, the parts in order, blue told nothing; a `Revisit` and a second step alike; a `NoopSink`
  visit's state the same); `h3-protocol`'s `game_event` (every event, 46 kinds; a level-5 view's
  parts mapped back, round-tripped and folded by `see` to the query's mapping; the row inverse a
  table, test code being gated too), the fold's no-change list, `game_query`'s guild over every
  golden's first 30 commands, each player and one beyond, `wire`'s message round trips.
- **Verify** (`--scope town --with-original-data`): every step passed (fmt, clippy, the checks,
  coverage, determinism, bench-check, doc-drift, original-data) but `check-lane` (`cross-lane`).
  The bench: every workload inside its gate, none re-baselined; `random-play-bonuses-36` +1.66 %,
  `manage-army-12` +1.36 %, `fork-economy-xl-144` +1.28 %, `day-start-builds-36` +1.04 % (as at
  T39b1: nothing of the visit runs on those paths), every other within 1 %.

### T40a — the census of the taverns, the pool's properties (2026-11-19)

Baseline 38d422b (#1040 merged; CI's runs on `master` standing in: the VM restarted minutes in and
the setup's baseline died while compiling). The user's computer was connected at the start only:
the client lane's 64 files and the four manuals with three help files staged, SHA-256 checked;
its shell went away minutes in, after the staging. At the start the user cut T40 in two, the
census first (decision 156, `TWN-Q28`, as recommended), consented to `cross-lane` (decision 157)
and, once the pool proved crate-private, chose public read-only readers (decision 158, as
recommended).

- **The readers** (`h3-game`'s `town/pool.rs`, the town lane's; `lib.rs`' re-export
  `cross-lane`): `HeroStatus` public (its `Taken` doc names the hire), the player's tavern part
  public as `PlayerTavern` (renamed from `Tavern`: `ObjectKind::Tavern` and the decision's
  `Tavern` were names already), `GameState::hero_status`, `player_tavern` and `hero_record`;
  `HeroPool::status` no longer test-only. Nothing on the wire, no hash or version moved. Tested
  in `town/tests/tavern.rs`' start fill (every type, one past the core, both players, a third).
- **The census** (`h3-map`'s `original_data`, design §78's As built): classes 95 and 97 typed as
  `ObjectKind::Tavern` and `DenOfThieves`, nothing held, no owner: 93 on 47 maps and 227 on 107,
  the survey's (exit criterion 9; every one of the 160 maps already imported whole). Every game
  played by `EndTurn` alone to day 29: the start's fill and the four weeks' by the census's own
  walk of TWN-TAV-001 (what was free at each draw, the setup's masks, the class weights, slot 1's
  avoidance, slot 0's own faction while one of its classes weighs; an empty slot only with
  nothing to draw; no type twice), each slot's hero as its owner sees it (its type's record,
  full spell points, a starting army by GAM-SET-001's shape); the pool after every `EndTurn`
  (`pool_holds`); the customised types' records at the build. All 160 hold, the census 35 s in
  debug (21 s before): 800 fills, 7 440 slots filled and checked, none empty; 3 694 slot 0s
  among the faction's classes (the other 26 a player without a faction); no slot 1 of slot 0's
  class; 17 customised types (2 on the map, 1 offered at the start), 13 off the map at level 3
  with their 2 000 experience (`Marshland Menace`, T37b's 13 or 14). Two mutations of the engine
  fail it on the first map: slot 0's own faction dropped (`A Viking We Shall Go Allied`: a class
  7 hero for a weight 0) and the week's release leaving the type offered (day 8, `Offered` in no
  slot).
- **The properties** (`h3-map`'s plans, `cross-lane`): `check_invariants` ends with
  `check_pool` (a slot's type offered with its record, none on the map; a returning mark only on
  a held slot; each hero on the map or in a garrison its type's record, taken by its owner's
  colour, one hero a taken type; every other status neither held nor offered);
  `check_dismissals` before every step of `every_step_keeps_the_commands_invariants` (the heroes
  on the map and in the garrisons, any owner's, the first record off them and an id past the
  store: accepted exactly when `legal_management` lists it, a refused one leaving the state, its
  hash and the events); probing every record of the store cost the property +75 % (155 to
  271 s), the narrowed probe +7 % (166 s). `a_forks_week_is_its_parents` asserts the week's fill
  (`TavernOffered` told: the plans' maps allow every type) and every player's tavern part before
  and after. All 184 of `h3-map`'s tests passed at once (673 s); both changed properties at 240
  cases (five times their 48) passed. A refused hire leaving the state was T38a2's
  (`check_town_builds`), the room by a town's hire `check_town_hire`'s.
- **Versions:** none moved.
- **Verify** (`--scope town --with-original-data`): every step passed (fmt, clippy, the checks,
  coverage, determinism, bench-check, doc-drift, original-data) but `check-lane` (`cross-lane`:
  `cargo xtask check-lane --cross-lane` ok, `cargo check --workspace --all-targets` clean). The
  bench: every workload inside its gate, none re-baselined; `random-play-bonuses-36` +1.66 %,
  `manage-army-12` +1.36 %, `fork-economy-xl-144` +1.28 %, `day-start-builds-36` +1.04 % (as at
  T39b1 and T39b2: nothing of this task runs on those paths), every other within 1 %.

**The queue:** T40a out; T40b next. Ids: decisions next 159, questions `TWN-Q29`.

### T40b — the taverns map, its replays, the plans' map Tavern and Den, the benchmarks (2026-11-19)

Baseline dc92305 (#1048 merged; master's code verified by CI, the setup's light gates passed).
The user's computer was connected at the start: 63 files of `original-data/` (the client lane's
list without `shell-videos.vid`) and the four manuals with three help files staged, SHA-256
checked against the PC's. At the start the user kept T40b whole (decision 159, over the agent's
cut in two with the plans first and the queue's order) and consented to `cross-lane` (decision
160, `TWN-Q29`).

- **The plans** (`h3-map`'s `play.rs`, `cross-lane`): two kinds appended to `KINDS` (95, 97;
  `TAVERNS`, `TAVERN_OBJECT`, `DEN_OBJECT`, the original's masks), `Plan::taverns` laid out with
  the banks, drawn by `tavern_plan()` (`plan()`'s, then the taverns: one or two near the heroes
  in one plan of three, a Tavern in two cases of three), which the properties of `play.rs` and
  `view.rs` and the taverns' cover test draw; the cover tests keep `plan()`'s. The
  oracles: `check_legal`'s listing and probe of the tavern decision (`tavern_answers`: the hires
  `apply` accepts, then `Close`; `check_map_hire`: a listed hire's hero as its owner's view
  showed it on the Tavern's free tile within the most heroes, the price paid, `HeroHired` of the
  slot and tile, the visitor unmoved, the decision closed; an unlisted slot refused with the
  state and its hash; `Close` alone), `check_decisions` (a fifth part of its open set, opened at
  `TavernOpened` and closed by `DecisionClosed`), `managed` (no army while the decision is open:
  the first failure, `check_managing`'s), `check_conservation` (the map hire's price). Drawn
  first inside `plan()` (the innermost tuple's fourth), the taverns shifted every deterministic
  cover test's cases: at one plan of two the dwellings' boat deck (3 to 0) and the town views'
  Capitols (9 to 0: the hires spent the gold) failed and the garrisons' met an oracle gap of the
  adventure's (a Whirlpool taking skeletons a guard's battle raised: `check_heroes_after_battle`
  now subtracts the loss told after); at one of three the verify's coverage run failed three
  others (the teleporters' `at_exits` and the bonuses' Sirens after a battle at 0, the
  garrisons' turn start compared whole). Hence `tavern_plan()` apart: the cover tests' cases are
  `master`'s again. New cover test `the_plans_hire_at_map_taverns` (256 cases:
  68 Taverns and 36 Dens laid out, 80 decisions opened, 32 with a hire listed and 56 without, 15
  hires, 65 closes, 11 hired heroes off the Tavern's tile, 58 guilds seen at a Den). The
  properties of `play.rs` and `view.rs` at 240 cases (five times their 48) passed,
  `every_step_keeps_the_commands_invariants` at 480 too.
- **The taverns map** (`fixtures/maps/e09-taverns-36.h3m`, `h3-map`'s `taverns_map.rs`): written
  by the writer, read back, its allowed heroes set in the raw model and encoded (the writer
  writes the default set; no writer change): red's Castle "Bastion" (Fort, Tavern, Brotherhood),
  blue's Rampart "Glade" (Fort, Tavern), five red heroes with five Royal Griffins (speed 9: they
  act first, so each retreat or surrender is the battle's first command), blue's three slow
  armies, the map Tavern, the Den, forty savage Stone Golems; six free types.
- **The replays** (scripted at seed 1, each with its test): `taverns-weeks-s1` (56 commands:
  five fills, slot 0 of the faction's classes), `taverns-hires-s1` (the town's and the map
  Tavern's hires with their replacements, blue's slot emptied, the hired hero off the tile, a
  second visit closed), `taverns-retreats-s1` (Orrin's from the golems in red's turn back at once
  with one Pikeman, the mark cleared at the turn's end; Sorsha's on day 6 and Cuthbert's on day 7
  in blue's turns, Cuthbert kept through day 8's release, TD-18), `taverns-surrenders-s1` (Edric
  to Ufretin in red's turn, Sorsha to Mephala in blue's, 600 gold each, both back with their
  armies; Sorsha hired back on day 2), `taverns-dismiss-s1` (Valeska dismissed, drawn again into
  red's slot 0 by week 2's fill, hired back with the Charm of Mana, which the fill equipped),
  `taverns-den-s1` (the guild at level 5 and its parts). The golden counts to 104
  (`h3-protocol`'s fold and session, `h3-server`'s record; the server lane's, `cross-lane`);
  `h3-protocol`'s and `h3-server`'s suites pass over them.
- **The benchmarks:** `week-start-taverns-36` (new; the step that starts day 8 on the taverns
  map, 16 random games: 1 833 609 Ir, master's code); no engine code changed, so no baseline
  moved. The guild's query timed once (release, a scratch test not committed): 1.87 µs on the
  taverns map (level 1, 6 parts), 7.30 µs on `Myth and Legend` (8 players, 21 towns, 16 heroes,
  40 parts): far off any step.
- **Versions:** none moved. Ids: none new.
- **Verify** (`--scope town --with-original-data`): every step passed (fmt, clippy, the checks,
  coverage 1 181 s, determinism, bench-check, doc-drift, original-data) but `check-lane`
  (`cross-lane`: `cargo xtask check-lane --cross-lane` ok, `cargo check --workspace
  --all-targets` clean). The bench: every workload inside its gate, as at T40a
  (`random-play-bonuses-36` +1.66 %, `manage-army-12` +1.36 %, `fork-economy-xl-144` +1.28 %,
  `day-start-builds-36` +1.04 %, every other within 1 %). `h3-protocol`'s and `h3-server`'s
  suites passed over the 104 goldens.

**The queue:** T40b out, T40 done; T41 next (the audit, increment 4's closure, increment 5
refined). Ids: decisions next 161, questions `TWN-Q30`.

### T41 — the audit; increment 4 closed; increment 5 refined (2026-11-19)

One session, the user present at the start; branch `town/e09-t41`, docs only, with the README's
row in `shared/e09-t41-readme`. Starting point: `master` at 9404617 (#1053 merged; CI green
there, the setup's light gates passed). The user's computer was connected at the start only: 63
files of `original-data/` (the client lane's list without `shell-videos.vid`) and the four
manuals with three help files staged, SHA-256 checked against the PC's. **At its start** (the
user, asked with a first look at the audit, `TWN-Q30`): T41 whole (decision 161), increment 5
the mage guild alone with the services 6 and travel and the Grail 7 (decision 162), the first
round as before (decision 163).

**The audit of increment 4's exit criteria** (`E09-archive.md`; design note §80), on `master`'s
code:

1. **The part, the slots, the pool's records.** `h3-pack-towns`' `the_original_taverns` (the
   tavern weights of both editions and the price, part v7), `h3-towns`'
   `a_hire_needs_a_standing_tavern_and_costs_the_parts_price`; the game's
   `the_start_fills_each_players_slots_with_new_records`, `the_week_releases_and_refills_and_a_
   record_keeps_its_id`, `a_hire_places_the_slots_hero_on_the_entrance_with_its_away_record`,
   `a_repeated_types_other_hero_leaves_an_offered_record_as_it_is`; T40a's public readers and the
   plans' `check_pool`. **Holds.**
2. **The draw and the fills.** `the_draw_weighs_the_classes_with_a_free_type_by_the_players_
   faction`, `the_draw_falls_back_to_every_free_type_then_fails`, `no_type_free_leaves_the_slot_
   empty`, `a_fill_equips_the_backpack_and_fills_the_spell_points`, `a_returning_hero_stays_
   through_the_weeks_release_and_the_marks_clear`, `a_random_choice_no_start_reads_is_drawn_at_
   the_first_fill`; the census (T40a: 800 fills, 7 440 slots by its own walk of TWN-TAV-001, two
   mutations caught); `taverns-weeks-s1` (`four_weeks_of_fills`). **Holds.**
3. **The hires.** The game's `a_hire_places_the_slots_hero_on_the_entrance_with_its_away_record`,
   `each_refusal_changes_nothing`, `the_slot_is_refilled_from_the_hirers_stream_with_one_creature_
   of_the_first_stack`, the map Tavern's `a_step_toward_the_map_tavern_opens_its_decision_for_the_
   visitors_owner`, `a_map_taverns_hire_places_the_hero_on_its_tile_and_closes_it`,
   `each_refusal_of_a_map_taverns_hire_changes_nothing_and_leaves_it_open`; `h3-adventure`'s
   `a_hire_needs_room_on_the_map_and_a_free_entrance`, `a_map_tavern_s_hire_needs_room_on_the_map_
   and_its_tile_free`; the plans' `check_town_hire` and `check_map_hire` (T40b's
   `the_plans_hire_at_map_taverns`: 15 hires); `taverns-hires-s1`. **Holds.**
4. **The leaves and the returns.** `town/tests/hire/leave.rs`' five (a retreat into a slot from
   the owner's stream, a surrender with its survivors sparing a marked slot, the beaten hero under
   `TWN-QUIRK-12` in both settings, a draw and a winner of created stacks only, a repeated type's
   other hero), `a_garrison_hero_keeps_its_town_as_the_winner_and_leaves_the_game_beaten`;
   `taverns-retreats-s1` and `taverns-surrenders-s1` (TD-18 on day 8). **Holds.**
5. **The turn.** `a_tavern_heros_points_are_set_at_its_owners_turn_start`,
   `every_hero_regains_its_spell_points_at_a_new_day` (decision 127), the marks' clear in
   `a_returning_hero_stays_through_the_weeks_release_and_the_marks_clear` (decision 126). **Holds.**
6. **The dismissal.** `a_dismissed_hero_leaves_the_game_its_type_free_with_every_artifact`,
   `a_dismissal_is_refused_for_a_hero_not_on_the_map_in_a_town_or_the_last`,
   `a_dismissal_waits_for_an_open_meeting`; the plans' `check_dismissals` before every step;
   `taverns-dismiss-s1` (`a_dismissed_type_is_drawn_again`). **Holds.**
7. **The customised types.** `town/tests/custom.rs`' `a_customised_type_still_free_has_its_
   record_at_the_build_and_is_offered_as_set`, `a_customised_type_is_the_rulesets_named_once_and_
   valid_on_its_own`, `a_tavern_hero_keeps_a_starting_skill_its_map_bans`; the census (T40a: 17
   customised types, 13 off the map at level 3 with `Marshland Menace`'s 2 000). **Holds.**
8. **The Thieves' Guild.** `town/tests/hire/thieves.rs` (`the_guild_answers_in_its_viewers_turn_
   by_its_towns_taverns`, `each_level_shows_its_rows_and_parts`, `each_row_counts_the_originals_
   value`, `a_rows_places_go_from_the_highest_equal_values_sharing_one`, the best hero and the
   best creature by TD-23, `the_artifacts_row_counts_the_backpacks_alone_under_twn_quirk_13`),
   `a_map_taverns_decision_opens_the_guild_at_its_owners_taverns`, the Den's three (everything,
   no recording sink, every visit alike); `TWN-STUB-17`, `-18` in place; `taverns-den-s1`. **Holds.**
9. **Classes 95 and 97.** `h3-map`'s `the_import_types_the_tavern_and_the_den`,
   `the_tavern_and_the_den_read_back_as_written`, `h3-adventure`'s `the_tavern_is_visited_from_a_
   neighbour_and_the_den_entered`, `h3-pack-adventure`'s `the_base_pack_types_the_tavern_and_the_
   den_of_thieves`; the census: 93 Taverns on 47 maps, 227 Dens on 107, all 160 maps imported
   whole (T40a). **Holds.**
10. **The views and the protocol.** `the_owner_sees_its_tavern_heroes_and_is_told_each_change`,
    `the_owner_is_told_the_hire_its_parts_and_the_slots_new_view`,
    `a_build_tells_each_viewer_its_changed_view_and_a_new_tavern_its_owners_raised_levels`; the
    plans' `check_hired` and `check_hire_sources` (`view.rs`); protocol v70 to v74 (T37a to
    T39b2) with the fold gate over the 104 goldens without a new gap (`h3-protocol`'s, CI's full
    verify on `master` the record). **Holds.**
11. **The env and the replays.** Action encoding v12 (`TAVERN_HIRE`,
    `a_hire_action_names_its_town_by_the_entrance_and_the_slot_by_its_entry`) and v13 (the
    tavern answers by role, `tiles_and_actions_encode_as_documented`); replay format v17 to v19
    with the words `town hire`, `manage dismiss`, `answer tavern`; the six `taverns-*-s1`.
    **Holds.**
12. **Rules, quirks, gates, batch TWN-4.** TWN-TAV-001 to -009, GAM-DEC-010, GAM-MAN-003 and
    ADV-VIS-007, -008 with their ids, sources and tests (`check-rules` passes); `TWN-QUIRK-12`
    and `-13` in `quirks-town.md`, each with its regression test in both settings. The gates on
    `master`'s code here: coverage (121 gated files in the four logic crates, no exclusion); determinism (debug, release and native, the 104 golden adventure replays among them); the census of `original_data` (all 160 maps import; 800 fills and 7 440 slots checked against its own walk of TWN-TAV-001 with the pool after every `EndTurn`, 3 694 slot 0s among the faction's classes; classes 95 and 97 and the customised types as criteria 9 and 7 say); the benchmarks (all 38 workloads inside their gates, from −0.99 % `reach-sea-xl-144` to +1.37 % `manage-army-12`; the day-start and month-start workloads re-baselined by T37a with their reasons, `week-start-taverns-36` baselined by T40b). Batch TWN-4 run and scored (T34, survey `E09-tavern.md` §14:
    every row as the reading). **The gap fixed** (below). **Holds.**

**The gap.** Criterion 12 wants batch TWN-4's consensus rows `verified`: the rules its rows settle
(TWN-TAV-001 to -009, GAM-MAN-003) were marked by T37a to T39b1. One more town rule cited no row
though rows played it: TWN-SET-005 (the pool), whose statuses rows TWN-4-1, -3, -4 and -5 read at
game +0x4DF18 (each placed hero's type its owner's colour, the type draws over the class's free
types only, a beaten or dismissed hero's type free and drawn again): the observation added,
`verified`, with a note naming what no row played (a prisoner, SoD's masks, the picks). Two of
the adventure lane's rules were played as well, GAM-BAT-003 (TWN-4-3, -4: each hero left the map)
and ADV-VIS-008 (TWN-4-7: the Den's guild at everything): their status is that lane's, told in
its inbox. TWN-SET-001 stays `implemented` (no row played a faction's choice: the batch's maps
fix them). **Left open, by design:** `TWN-STUB-2`, `-16` (E10), `-6`, `-15` (increment 5), `-8`
(the battle lane's inbox), `-13` (Visions' reach), `-17` (increment 7), `-18` (E13).

**Increment 4 closed** (2026-11-19): its section (the refinement's note, decisions 98 to 160, the
exit criteria and the task list with T41's line) moved verbatim to `E09-archive.md` (checked by
script: every line of the old epic in the new epic or the archive, but the seven edited ones);
its design note's Part F stays in `E09-design.md`, as Parts A to E (decision 40). The README's row
(4 of 7) in `shared/e09-t41-readme`.

**Increment 5 refined** (`TWN-Q30`; decisions 161 to 164): the mage guild alone (decision 162;
the epic's sketch now seven increments, with the line for older citations); the survey with its
checkpoint, then the drafts with Part G and the cut (decision 163). The inbox's four notes that
increment 4 acted on (E08 T122's and T123b's, `Marshland Menace`'s, `ADV-STUB-9`'s) archived with
where their work went (`QUEUE-ARCHIVE.md`); the market's six and the four service notes kept for
increment 6 (their labels now say so), the town chances' for T42. `TWN-STUB-17`'s and
TWN-BLD-002's "increment 6" now 7. The adventure lane's inbox told the services' number and its
two rules. The queue holds T42 and T43.

**Verify:** the engine's code is `master`'s. `cargo xtask verify --scope town --summary
--with-original-data` on `master`'s code with no failing step (coverage 17.8 minutes, determinism
8.2, bench-check 12.9, original-data 0.8; `deny` and `ffi-smoke` skipped by the scope); the setup
had copied only `extracted/H3bitmap/Objects.txt`, so `h3-pack-adventure`'s example skipped itself: run
apart after the staged `original-data/.` was copied in (`cp -r --update=none`), it passed. On the
branch the docs gates (`check-docs`, `check-rules`, `check-stubs`, `doc-drift --base origin/master`)
and `check-lane --base origin/master`; `shared/e09-t41-readme` `check-lane` and `check-docs`. CI on the
pull requests is the record.

**The queue:** T41 out; T42 next, T43 queued. Ids: decisions next 165, questions `TWN-Q31`.

### T42a — the survey's first part: what the guild holds (2026-11-19)

One session, the user present at the start; branch `town/e09-t42a`, docs only. Starting point:
`master` at 7dbe195 (CI green there). The setup's baseline had run on 0d91fe8, whose coverage
and determinism steps failed only on the six taverns replays' `import_version` (#1053 recorded at
v26 while #1054 took v27); the adventure lane's 7dbe195 re-recorded them, and the session moved
its branch onto it. The user's computer was connected at the start only: 68 files staged (the
four LODs, the executables, the DLLs, sounds and videos, `maps.tar`, `extracted.tar`, the 40 MP3s,
the three saved games, the four manuals and three help files), SHA-256 checked against the PC's.
**At its start** (the user, `AskUserQuestion`): T42 cut in two, the guild first (decision 165).

**Read** (survey `docs/rules/surveys/E09-guild.md` §2 to §6; `docs/exe/map.toml`): the spell
table's load (`spell_table_load` 0x59E090, `spell_row_load` 0x59E150: the nine town chances at
the record's +0x44), the draw again at its details (`town_setup_spells` 0x5BE600), the map's two
spell sets (`load_town_body`, its spells part), the guild's part of `town_build_effects`, the
Aurora's branch of `town_heroes_learn_spells` again, the guild's window (`mage_guild_open`
0x5CE560 with the spell book's offer, `mage_guild_dialog_fill` 0x5D6EF0). Five new map entries,
five changed; `structs.toml`: the spell record's +0x44, the town record's +0x70 read and +0x7C.

**Found** (§0): the weights are `SPTRAITS.TXT`'s columns, summing to 100 at each level a faction
reaches; every town draws all five levels' slots and the spare at the new game, required spells
first (whatever their weight), banned ones never; the counts follow every build; the exact
chances match the wiki's table to the percent; the Aurora teaches every spell the town does not
ban up to Wisdom's + 2 whatever the guild's level, the map-wide bans not tested (T-18). VCMI: the
same in law, but a required spell even when banned (TD-25) and the Aurora by the map's bans, not
the town's (TD-26). Batch TWN-5's three rows from this part for T42b (§9).

**The census** (§1, a throwaway example over `h3_map::read` and a script, not committed): 32 towns
with required spells (never more than a level shows), one requiring a spell it bans (`Faeries`);
396 towns banning spells, the 14 random towns of `Heroes of Might not Magic` and its Allied
version the only short pools; 249 guilds at the start (34 at level 5), 5 Libraries, no fixed
Conflux, 5 random towns on AB or SoD maps with the Grail building.

**Verify:** the engine's code is `master`'s. `cargo xtask verify --scope town --summary
--with-original-data` on the branch with no failing step (coverage 15.1 minutes, determinism 4.0,
bench-check 9.9, original-data 0.5; `deny` and `ffi-smoke` skipped by the scope); `check-exe-map`,
`scripts/exe.py check` and `cited` (every address covered), `doc-drift` and `check-lane` against
`origin/master`. CI on the pull request is the record. **The queue:** T42 becomes T42b, then T43. Ids: decisions next 166; disputes `TD-27`;
quirk candidates `T-19`; questions `TWN-Q31` (T42b's checkpoint).

### T42b — the survey's second part: what a hero gets; the checkpoint TWN-Q31 (2026-11-20)

One session, the user present at the start and at the checkpoint; branch `town/e09-t42b`, docs
only. Starting point: `master` at 4c19fbb (#1061 merged; CI green there). The user's computer was
connected at the start only: 68 files staged (the four LODs, the executables, the DLLs, sounds
and videos but `shell-videos.vid`, `maps.tar`, `extracted.tar`, the 40 MP3s, the three saved
games, the four manuals and three help files), SHA-256 checked against the PC's. The VM
restarted during the setup's baseline (killed at its test step; files kept): CI's green run on
4c19fbb stands for it. **At its start** (the user, `AskUserQuestion`): T42b whole (decision 166).

**Read** (survey `docs/rules/surveys/E09-guild.md` §11 to §15; `docs/exe/map.toml`):
`town_heroes_learn_spells` 0x5BE030 a third time and each of its nine callers at its call (the
ninth named `town_screen_bars` 0x5C6E10: the town screen's hero bars, so every opening of the
screen teaches with a visitor there); `town_screen_open` 0x5BE210 whole (its callers: a visit, the
adventure screen's town button and two clicks: an opening without a visit); the computer's visit
(`ai_town_visit` 0x5253D0: its spell book bought without a question at 500 gold, its learning, its
building visits) and `ai_hero_turn`'s start (a computer hero on an entrance re-visits each turn);
the new day's town refill (`new_day` 0x4C82E4 to 0x4C8434); the Mana Vortex (`town_visit_buildings`
0x5BD8E0's first branch, `hero_primary_clamped` 0x5BDE40, the week flag +0x33) with every caller's
heroes in order; the hero type's spell book (`hero_reset` 0x4D889F to 0x4D890A). Three new map
entries (`manager_run`, `hero_primary_clamped`, `town_screen_bars`), eleven changed; `structs.toml`'s
town `week_flag` read.

**Found** (§0's items 9 to 16): learning by a worn book and Wisdom's bound (HER-SPL-005) at the
guild's shown slots, no owner or map-ban test; its triggers (the visit's end, a capture and a siege,
every build, a hire, the start, the guild's window, the screen's every opening with a visitor, the
garrison moves, Town Portal, Castle Gate, the computer's visits), none at the new day or a human's
turn start; the book bought by a human at the guild with a question, by the computer without; the
night's refill to the maximum in a town with a guild, no book needed; the Vortex once a week a town,
twice the maximum, given at the screen's opening and a build with the visit (the garrison hero
first), a human's hire, Castle Gate and the computer's visit, not Town Portal. VCMI 1.7.5: the same
set, book, refill and Vortex, but its triggers are every turn start (each town's garrison hero then
visitor), a skill changed in town, a visit (the visitor only), a player's build (both; a town
event's only for a guild, the Library or the Aurora) and a hire; nothing at the start. The wiki: the Vortex goes to the garrison hero "once the player opens the town menu".
New dispute TD-27 (when the heroes learn and get the Vortex). Batch TWN-5: five rows (§18).

**The census** (§10; the census function added to `h3-map`'s `original_data` for the run, not
committed; seed 0): 561 heroes on an entrance at the start (538 main towns' heroes, 23 placed),
none in a garrison; in the 74 guild towns with a hero 33 main towns' heroes and 8 placed heroes with
a book (5 maps); Wisdom's bound bites for none; one Mana Vortex at the start (a neutral Dungeon,
`Caught in the Middle`), no hero there.

**The checkpoint** (`TWN-Q31`, survey §20; `AskUserQuestion`, one round of four; the Aurora's
question asked again in other words after the user found "excluded" and "ignored" ambiguous): TD-10
the original (decision 167), TD-25 the original (168), TD-26 the original's set with T-18
`fix-optional` (169), TD-27 the original's triggers with a free opening command (170), the agent's
calls (171); each as recommended.

**Verify:** the engine's code is `master`'s. `cargo xtask verify --scope town --summary
--with-original-data` on the branch with no failing step (coverage 18.2 minutes, determinism 8.1,
bench-check 13.7, original-data 0.9; `deny` and `ffi-smoke` skipped by the scope); `check-exe-map`
and `scripts/exe.py check` (every cited address covered, the sizes the executable's), `check-docs`,
`doc-drift` and `check-lane` against `origin/master`. CI on the pull request is the record.

**The queue:** T42b leaves it; T43 next (the drafts, Part G from §82, the cut). Ids: decisions
next 172; disputes `TD-28`; quirk candidates `T-19`; questions `TWN-Q32`.

### T43 — increment 5's drafts, the design note's Part G, the cut, the checkpoint (2026-11-20)

Decision 163's second task, kept whole at its start (the user, `TWN-Q32`'s first round, decision
172; the agent had proposed T43a, what the guild holds, then T43b, what a hero gets with the env,
the cut and the checkpoint, as T33a and T33b). Branch `town/e09-t43`, docs only (the four drafts,
the design note, the epic, the log, the answered questions, the lane's queue and HANDOFF, four
lanes' inboxes). Starting point: `master` at 67b86dd (#1067 merged; the setup's baseline passed its
light gates on `master`'s code, CI's fingerprint). The user's computer was connected at the start
only: 61 files of `original-data/` (the four LODs, the executables, the five DLLs, both `.snd`,
`H3ab_ahd.vid`, `battle-videos.vid`, both archives, the 40 non-campaign MP3s, the three saves) and
the four manuals with three help files staged, SHA-256 checked against the PC's sums.

**The ADR-level choices** (`TWN-Q32`'s second round, four; decisions 173–176, each as
recommended): the guild weights in the town records; the Aurora in this increment; the opening as
`TownCommand::Open`, listed only where it would change something, a visit's end, a capture and a
siege's end applying it; the spell book as `TownCommand::BuySpellBook`.

**Read for it:** nothing new in the executable (the survey's §3 to §6 and §11 to §15 suffice); the
engine's paths each trigger lands in (`settle_outcome`'s capture, `visit_town`'s outcome for an own
town, `build_told` for the command's and the events' builds, `hire::hire`, `main_town_heroes`,
`rows::heroes`, `advance_day`'s `regain_mana`), `HeroChanged` as the meeting's learning tells it,
the towns' stream's place in the state, the store's hash with its statics.

**Written:**
- **The drafts** (`docs/lanes/town/adr-drafts/`): `0028-increment-5.md` (point 2 the guild's
  slots, the Vortex's week flag, the Aurora set and their rules; point 5 the guild weights, the
  roles `guild`, `library`, `aurora`, `mana_vortex`, the slots, the book's price, the Vortex's
  factor; point 6 `Open` and `BuySpellBook`; point 9 the versions), `0017-increment-5.md` (points
  1, 2, 3, 4, 8, 9, 10: the commands, the state and the setup's town spells, the build's draw and
  the start's learning, the owner's guild view and the events, the week's flags and the night's
  refill, the towns' stream, the env's two entries, the versions, the learning's triggers),
  `0016-increment-5.md` (the outcome's town entered), `0013-increment-5.md` (a guild's teaching by
  `can_learn_spell`; the refill's and the Vortex's points).
- **The design note** Part G, §82 to §100: scope and choices (§82, the agent's twelve calls), the
  part v9 (§83), the map's town spell sets (§84), the draw (§85), the store v4 (§86), what the
  guild shows (§87), what a hero learns (§88), the triggers (§89), the opening (§90), the book
  (§91), the night's refill (§92), the Vortex (§93), the views and events (§94), the env, the
  protocol and the replays (§95), ids, versions, quirks and stubs (§96), the gates and batch TWN-5
  (§97), the building tasks T44 to T51 with the order against E08's increment 14 (§98), the exit
  criteria (§99), the risks (§100).
- **The checkpoint** `TWN-Q33` (one round of four; decisions 178–181, each as recommended but the
  stream): the opening teaches both heroes whenever a guild stands; **the towns' stream** for the
  draw (the user's choice over the agent's new `GUILDS` stream: no `shared/` stream, the week
  replays' outcomes re-recorded at T47a; the drafts and Part G amended for it); the views and
  events as proposed; the cut and the exit criteria as written. The agent's calls of 177.
- **New ids** (planned): rules TWN-GLD-001 to -009, TWN-SET-012; quirk `TWN-QUIRK-14` (T-18, its
  flag `quirk.town.aurora_ignores_map_bans`); `TWN-STUB-6` and `-15` to be lifted at T48a; no new
  dispute or quirk candidate.

**The queue:** T43 out; T44 next, T45 to T51 queued. The adventure lane's inbox told the planned
`cross-lane` changes in its paths and the merge order, the AI lane's the env's two entries, the
server lane's the protocol's additions, the battle lane's the harvest's spell columns. No version
moved. Ids: questions next `TWN-Q34`; decisions next 182; disputes `TD-28`; quirk candidates
`T-19`.

**Verify:** docs only: the docs gates (`check-docs`, `check-rules`, `check-stubs`,
`check-exe-map`, `check-lane --base origin/master`, `doc-drift --base origin/master`) and the
scoped verify; CI on the pull request is the record.

### T44 — batch TWN-5: the maps, the harness's reads, the runs, the scoring (2026-11-20)

One session, the user present at the start; branch `town/e09-t44`, `cross-lane` (the user's
consent at the start: `h3-map`'s header writer and the writer's `Map` and `AnyTown` literals in
the adventure lane's tests and the clients, `h3-experiment` and `docs/experiments/README.md` the
battle lane's). Starting point: `master` at 40c60b8 (#1071 merged; CI's run on `master` green
there; the setup's full baseline stopped for disk, CI's run standing). The user's computer was
connected at the start only: 64 files of `original-data/` (the client lane's list) and the four
manuals with three help files staged, SHA-256 checked. The user kept T44 whole over the agent's
T44a/T44b (decision 182).

- **The writer** (`h3-map`): `AnyTown::spells` (`TownSpells`: required and banned codes, the
  town's two nine-byte sets; `writer/town.rs`, the town lane's), `Map::banned_spells` (the
  header's nine bytes; refused past 70 and in RoE; `writer.rs`, `cross-lane`); every `Map` and
  `AnyTown` literal given the field (47 and 21, mechanical). Tests: the sets read back through
  `parse`, the refusals.
- **The spec** (`h3-experiment`, `cross-lane`): `[map] banned_spells`; `[[map.town]]`'s
  `required_spells`, `banned_spells`, `main` and `generates_hero` (a town of any kind then, the
  same bytes as a `Town` without them; a main town's owner allowed its faction alone; refused: a
  neutral or second main town, `generates_hero` without `main`, an unknown or non-hero spell).
  Test `a_map_town_may_have_spells_and_be_its_owner_s_main_town`. The spec format in
  `docs/experiments/README.md`.
- **The harness** (common): `town:x,y:guild` (each level's count and drawn slots, 6 to 2: the
  record's six past them hold other data), `:guild_level`, `:week_flag`, `:standing` (the hero on
  the entrance by the heroes' tiles); a walk's and a `town` step's town screen pass the message
  that comes over it a moment after it shows (the Mana Vortex's); `hall = true` hovers the hall's
  box until the status line names it (a Tower with Mage Guild 5 covers its centre). The
  harness's own check (`harness-check.toml`) run after. `clients/capture/README.md`, "Towns".
- **The specs** `E09-batch-5a.toml` (stamp `fe8d6a6b`: rows 1 to 4, day 1) and `-5b.toml`
  (`1ca79538`: row 5 over three weeks), the recipes
  `clients/capture/batches/E09-batch-5a.toml`, `-5b.toml`; two runs each, the filled templates in
  `experiments/results/`. **The agent's prediction of TWN-5-3's counts was wrong at first**
  (5, 4, 4, 3, 2 from memory, not the survey's 5, 4, 3, 2, 1): corrected in the spec before the
  runs that count.
- **Scored** (survey `E09-guild.md` §21): every row as the code reading in both runs: TD-25, TD-26,
  TD-10, TD-27 the original's (decisions 167 to 170 stand), VCMI's refuted on each; **T-18
  confirmed** (the Aurora teaches the map's ban), so T45b may add `TWN-QUIRK-14`'s flag.

### T45 — `shared/`: increment 5's drafts into the ADRs, the glossary, the security lines, the flag of `TWN-QUIRK-14` (2026-11-20)

One session, the user present at the start; branch `shared/e09-t45`, `cross-lane` (the ADRs, the
glossary, `docs/06-security.md` and `h3-content` shared; the harvester and `packs/base` the
battle lane's, the golden adventure replays the adventure lane's, the bot reports the AI lane's,
one test literal the server lane's). Starting point: `master` at 4771688 (#1076, T44's, merged);
E08 T134a (#1077) merged during the session, the branch rebased onto it (b0efed6) before T45b, so
the re-record is on top of its v22 goldens. The setup's full baseline was stopped at its coverage
step for disk (its tests passed on 4771688); CI's run on `master` stands for it. The user's
computer was connected at the start only: 64 files of `original-data/` (the client lane's list)
and the four manuals with three help files staged, SHA-256 checked. **Kept whole** with the
`cross-lane` consent (decision 183).

- **T45a, the ADRs** (the four drafts of T43, accepted at `TWN-Q33`, deleted from
  `adr-drafts/`): ADR-0013 points **15.5** (a guild's teaching by `can_learn_spell`) and **16.6**
  (the town's refill, the Vortex's points); ADR-0016 point 4's kind `Town` (the town entered in
  the visit's outcome); ADR-0017 points 1 (`Open`, `BuySpellBook`, the state, the setup's town
  spells, the build's draw and the start's learning), 2 (the owner's guild view and the events), 3
  (the week's flags, the night's refill, no turn start), 4 (the guilds from the towns' stream), 8
  (the env's two entries), 9 (the versions, E08 T134a's numbers as merged), 10 (the learning's
  triggers); ADR-0028 points 2, 5, 6, 9, the header's amendments line, **the Consequences and
  Alternatives for increment 5** (from decisions 173 to 179); each revision history, the ADR
  index's rows. Each block cites batch TWN-5's rows that confirmed it (survey `E09-guild.md`
  §21: every row as read).
- **T45a, the docs:** `docs/03-domain.md`'s terms (the guild's slots and spare, the guild level,
  the Aurora set, the Mana Vortex's week flag, the night's refill, the opening, the town's heroes'
  learning); `docs/06-security.md`'s section "The mage guild (E09 increment 5)" (design §94: the
  owner's alone, the unshown slots never told, the stream server-only, `Open` and `BuySpellBook`
  the owner's, the events).
- **T45b, `h3-content`:** `quirk.town.aurora_ignores_map_bans` (`QuirkFlag(44)`,
  `AURORA_IGNORES_MAP_BANS`, T-18): the registry's 45 positions, its two tests; read from T48a.
  No encoding changed: `RULESET_HASH_VERSION` 53 and the pack format kept. **`h3-pack`:**
  `the_harvested_flag_of_e09_increment_5` (on in `faithful`, off in `strict-fixed`, every
  expansion). **The harvester's** two profiles and `packs/base/core.toml`'s two profile lines (the
  synthetic packs name no town flag).
- **The ledger:** `TWN-QUIRK-14` entered (`quirks-town.md`; confirmed by TWN-5-2; its rule's
  regression test with T48a, the profiles' now).
- **Replays and records:** the base faithful ruleset hash moved 0x6ba8109889b34a25 to
  **0x36b98ea152b4db95** (`strict-fixed`'s and the synthetic's kept). The 101 golden adventure
  replays on it re-recorded from their commands by a scratch script (`h3 adventure run` in
  release, 4 s): only `ruleset_hash` moved; the three `strict-fixed` ones and the ten battle
  replays unchanged; all 114 verified (`h3 adventure verify`, `h3 verify`). `h3-server`'s record
  literal; the AI lane's `bots-report-v1.json` and `bots-neutral-v1.json` regenerated (2.5
  minutes): only `engine.core` moved.
- **Also:** the battle lane's HANDOFF version line (quirk flags 44, the hash); the adventure
  lane's inbox (the replays' header, mechanical at a merge).
- *Agent's calls:* ADR-0013's two draft sentences as new items 15.5 and 16.6 (increments 3 and 4
  added 16.4, 14.5 and 16.5 so); ADR-0028's Consequences and Alternatives for increment 5 (the
  drafts had none; written from decisions 173 to 179); the ledger entry made with the flag, as
  T35b's (decision 131's pattern).

**The queue:** T45 out; T46 next (the town part v9). Ids: decisions next 184; quirks next
`TWN-QUIRK-15`.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`, `cargo test -p h3-content -p
h3-pack -p h3-harvest -p h3-server -p h3-protocol`; CI on the pull request is the record.

### T46 — the town part v9: the guild weights, the four roles, the guild's parameters (2026-11-20)

One session, the user present at the start; branch `town/e09-t46`, `cross-lane` (the harvester,
its gap-fill and `packs/README.md` the battle lane's, the golden adventure replays and
`h3-game`'s `tests/mod.rs` the adventure lane's). Starting point: `master` at a8460e1 (#1082, E08
T134b1; #1080, T45's, merged before it); the setup's light gates on `master`'s code (CI's
fingerprint). The user's computer was connected at the start only: 71 files staged (the four
LODs, the executables and DLLs, both `.snd`, the three videos, `maps.tar`, `extracted.tar`, the
three saves, 40 MP3s, the four manuals and three help files), SHA-256 checked (one hash of each
side's sorted list). **Kept whole** with the `cross-lane` consent (decision 184).

- **`h3-towns`** (design §83): `TownDef::guild` (spell key, weight) compiled to
  `TownRecord::guild` (a `u16` per core spell by `SpellId`, 0 not given) with
  `TownRecord::guild_weight`; `BuildingRoles::guild` (1 to 5), `library`, `aurora`,
  `mana_vortex`; `TownParamsDef`/`TownParams::guild_slots`, `spell_book_price`, `vortex_factor`.
  The refusals: `TownCompileError::UnknownSpell` (new), a spell outside the closure, a creature
  ability, a spell twice, a weight outside 1 to 10 000; a guild level outside 1 to 5, a level's
  second building, a second Library, Aurora Borealis or Mana Vortex; a level's slots outside 1 to
  5, the book's price a resource twice or 0 (empty: free), the factor outside 1 to 10. **Town
  ruleset part v9** (`TOWN_RULESET_HASH_VERSION`: the three parameters after the Thieves' Guild's
  rows, each record's weights after its tavern weights, the four roles after the reveal); pins
  `0x67ee_ef1c_7b56_fda6` and the empty part's `0xedec_d7c9_34cc_e444`. TWN-GLD-001 `verified`
  (its catalog entry new).
- **`h3-pack-towns`:** `[town.guild]` (spell = weight), the four role fields, `guild_slots`,
  `spell_book_price`, `vortex_factor` in `[town_params]`; the format's doc. The synthetic pack:
  slots 4, 3, 3, 2, 1, a book of 400 coin, a factor of 3, five weights, two Mage Guild levels, a
  Library, a Mana Vortex and the Grail's Aurora.
- **The harvest** (`h3-harvest`, `cross-lane`): `heroes::spell_guilds` (fields 16 to 24 of the
  hero spells' rows, each spell's pack as the spell harvest's); each record's `[town.guild]` (its
  faction's column, a 0 left out; a `sod` spell weighing anywhere refused: Titan's Lightning Bolt
  weighs 0); the gap-fill's `[towns.mage_guild]` (source `exe-mage-guild`) and eight roles (the
  Mage Guilds by their ids 0 to 4, the Tower's Library 22, the Dungeon's Mana Vortex 21, the
  Conflux's Aurora 26); a role of every faction now goes to the factions whose list holds the
  building (an error when none does). The harvester re-run on the staged installation wrote
  `base` and `ab` (the weights, the roles, the parameters) and `sod` (its header only); the core
  and battle files unchanged.
- **Tests:** `h3-towns`' compile, refusal, message and hash tests (the test core's spells and
  school new); `h3-pack-towns`' synthetic record and roles, the parser's strictness and
  `the_original_guilds` (each record's weights by level summing to 100 at every level its Mage
  Guilds reach, 0 above: Castle 4, Stronghold and Fortress 3, the others 5; no ability and not
  Titan's Lightning Bolt weighing; Armageddon only in the Inferno, Necropolis and Dungeon, Death
  Ripple and Animate Dead only in Necropolis; the RoE pack's weights the same; the roles on their
  four buildings alone; the parameters); `h3-harvest`'s town test (the fixture's columns 16 to 18,
  Zap's 0, Bolt's SoD-only 0s; the roles; the errors).
- **Replays:** the 104 golden adventure replays re-recorded from their commands (only
  `town_ruleset_hash` moved, 0xab3ee3f1cb243f30 to **0xcf8db1aa2bb768d2**). The bot reports record
  no town part: unchanged.
- **Also:** `packs/README.md` (the town files' guild data; the town chances no longer "not
  harvested"); the battle lane's inbox (T43's plan note replaced by what changed in its crate).
- *Agent's calls* (decision 185): the slots bounded 1 to 5 (design §83's "1 to 6" against its own
  reason: the spare makes at most six, the original's slots a level, survey §3.2); the guild roles
  by building id for every faction that holds it (the Mage Guilds 3 to 5 are not every faction's);
  no override of the weights (no pack needs one: the AB copy's columns are equal, RoE's are E11's);
  a 0 weight refused (0s are omitted); the §2.2 checks in `h3-pack-towns`' original-packs test, as
  T36's taverns (`h3-harvest` has no original-data test: `verify --with-original-data`
  re-harvests and compares); the Conflux's key `aurora_borealias` (the packs' spelling, from
  `BldgSpec.txt`; §83 wrote `aurora_borealis`). Design §83's *As built*.

**The queue:** T46 out; T47 next (T47a the map's town spell sets, the draw, the store v4). Ids:
decisions next 186.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`, `cargo xtask
check-lane --cross-lane`, `cargo check --workspace --all-targets`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test -p h3-harvest -p h3-protocol -p h3-server -p h3-cli -p
h3-sim -p h3-game -p h3-towns -p h3-pack-towns`; CI on the pull request is the record.

### T47a1 — `h3-towns`' guild module: the draw, what the guild shows, the Aurora set (2026-11-20)

One session, the user present at the start; branch `town/e09-t47a1`, the town lane's paths only.
Starting point: `master` at 9413c3f (#1087, E08 T134b2; #1086, T46's, merged before it); CI's run
on `master` still running at the start, so the setup ran the full baseline: its tests passed, it
was stopped at coverage (the disk at 9 GB free). The user's computer was connected at the start
only: 71 files staged (the four LODs, the executables and DLLs, both `.snd`, the three videos,
`maps.tar`, `extracted.tar`, the three saves, 40 MP3s, the four manuals and three help files),
SHA-256 checked (one hash of each side's sorted list). **T47a cut in two** at its start (decision
186): this is T47a1.

- **`h3-towns`' `guild` module** (new; design §85 to §87): `SpellSet` (a `u128` by spell handle:
  `of`, `bits`, `contains`, `with`, `union`, `iter`), `GuildSets` (the town's required and banned
  spells and the scenario's bans), `GuildSlots` (six slots at each of the five levels, a byte a
  slot; `slot`, `level`), `MAX_GUILD_SLOTS`; `draw_guild` (TWN-GLD-002: the excluded set, the
  first required spell in spell order, else a draw of 1 to the level's open weights and the first
  running weight to reach it, a total of 0 leaving the slot empty without a draw), `guild_level`
  and `shown` (TWN-GLD-003: the highest standing `guild` building; each built level's part slots,
  one more with a `library` ever built, cut at the first empty slot), `aurora_set` (TWN-GLD-004's
  set: every hero spell not granted by name only that the town does not ban, less the scenario's
  bans with `quirk.town.aurora_ignores_map_bans` off). Exported from the crate's root.
- **The fixture** (`fixture.rs`): `guild_core` (the core with a school and twelve spells of every
  level: a weight of 0 at levels 1 and 4, a creature ability, Titan's Lightning Bolt's flag) and
  `guild_keep` (the Keep's Mage Guilds 1 to 5, Library and Aurora Borealis at 24 to 30, its guild
  weights), for T48's and T49's tests too; `core_with` now over `core_edited`.
- **Tests** (`guild/tests.rs`, 13): each level's weighted spells once and the rest empty over 32
  seeds; the law by the first slot's roll replayed over 64 seeds (each of its three spells met);
  the required spells first in spell order whatever their weight, a guild whose every slot is
  required or empty drawing nothing; TD-25; the scenario's ban, the bolt and the ability never,
  required or not; an empty level; one seed alike, seeds differing; the slots' bounds; the guild
  level; the counts with and without the Library and a level or the Library built later; an empty
  slot ends its level; `twn_quirk_14_…` (the set under both settings); the spell set. `h3-towns`
  at 100 % (`cargo llvm-cov -p h3-towns`).
- **Docs:** the catalog's TWN-GLD-002, -003, -004 (`implemented`; -004's learning planned for
  T48a); `TWN-QUIRK-14`'s regression test; design §85 to §87's *As built*; decisions 186, 187.
- *Agent's calls* (decision 187): six slots at every level (§86's twenty were the base part's:
  any level's spare may make six); the parameters and a `GuildSets` as `draw_guild`'s inputs;
  `shown` over the slots beside the town until the store v4; `SpellSet` in `h3-towns`; the Aurora
  set under TWN-GLD-004 with the quirk's rule test.

No version moved, no replay re-recorded, nothing `cross-lane`: nothing calls the module yet.

**The queue:** T47a1 out; T47a2 next (TWN-SET-012 with import v28, the store v4, the game's draw,
every golden re-recorded). Ids: decisions next 188.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T47a2 — the map's town spell sets (import v28), the store v4, the guilds drawn at the build (2026-11-21)

One session, the user present at the start; branch `town/e09-t47a2`, `cross-lane` with the user's
consent (the adventure lane's `h3-map` `lib.rs`, `tests/towns.rs`, `examples/original_data.rs`,
`h3-game`'s `setup.rs`, `state.rs`, `lib.rs`, `h3-cli`'s `tests/adventure_replays.rs`, the 104
golden adventure replays; the AI lane's `h3-cli` `src/ai/census.rs`; the server lane's `h3-server`
`tests/record.rs` and `h3-protocol`'s `tests/game_mutation.rs`). Starting point: `master` at
87c83f6 (#1092; T47a1's #1090 merged before it); CI's fingerprint not yet on `master`, so the
setup ran the full baseline: its tests passed, stopped at coverage for the disk. The user's
computer was connected at the start only: 71 files staged (as T47a1's); its shell did not start
("Workspace unavailable"), so the sizes were compared with the listing and the LODs and
executables with `docs/formats/installation.md`'s SHA-256. **T47a2 kept whole** (decision 188).

- **TWN-SET-012** (design §84): `h3_towns::TownSetup::required_spells`, `banned_spells` (sorted,
  each once); `h3_map`'s import `town_spells` (bits 0 to 69, 70 and 71 dropped, an RoE map's
  required set empty, a spell the ruleset lacks left out); **map import v28**;
  `GameSetupError::TownSpell(TownId, SpellId)` (a spell the ruleset lacks or a creature ability,
  after the scenario's bans; `town::town_spell_refused`). Catalog entry new; GAM-SET-003 amended.
- **The store v4** (design §86): `Town::guild` (`with_guild`), `Town::vortex` (set from the build),
  `TownStatic::aurora`; `Towns::start_week`, `take_vortex` (tested, first called by T49a);
  `GuildSlots::all`; **town state hash v4** (the slots and flag after the Portal, the Aurora set's
  bytes after the events; the pinned store hash 0x79c07378d0e04452).
- **The game's draw** (design §85): `h3_game`'s `town/guild.rs` `draw_guilds` in `GameState::new`
  after the Shipyards without water: each town's `GuildSets`, its slots by `draw_guild`, its Aurora
  set by `aurora_set`, the towns' stream's first draws (decision 179); **game state hash v38**.
  Tests: `town/tests/guild.rs` (three: the draw and the stream replayed, two games alike and a
  fork; the scenario's ban and the quirk; the refusal's order), `tests/build.rs`' Aurora set, the
  store's two, the import's one; the game's town test part gives each creature ability a battle row.
- **Benchmarks:** the 38 adventure workloads under the gate against their baselines, none
  re-baselined (the largest `random-play-bonuses-36` +1.67 %, `day-start-builds-36` +1.47 %;
  `build-*` −0.17 % and −0.01 %).
- **Replays:** the 104 goldens re-recorded from their commands by a scratch script: 103 moved only
  `final_hash`, the checkpoints, `game_state_hash_version` and `import_version`; no week replay's
  outcome line moved. **`towns-s173-d21`** was refused at command 140 (the neutral garrison of its
  day-16 siege, grown otherwise by the moved towns' stream, now acts first): a scratch probe played
  that battle again with `h3_battle_ai::choose` for both sides (11 commands for the 67), Yog still
  wins, levels up and takes the town for blue; the other commands kept (189 for 244, day 22).
  E08 T135b1's seven `trades-*` goldens (#1095, merged meanwhile) re-recorded after the merge
  with `master`: hash lines and versions only. All 111 verified; the bot reports unchanged. `h3-server`'s record literals and `h3-protocol`'s
  sample bound (`+ 150`) moved; `h3-protocol`, `h3-server`, `h3-sim`, `h3-files`, `h3-cli` tested.
- **Inbox:** the adventure lane (v28 and v38 taken, the goldens), the server lane (its literals).
- *Agent's calls* (decision 188): a spell the ruleset lacks left out by the import; the refusal in
  the game's town module; the Aurora set in `TownStatic`; the Vortex's writes before their caller;
  the s173 siege re-played rather than the replay replaced (the protocol's tests use it).

**The queue:** T47a2 out; T47b next (the owner's view, the protocol). Ids: decisions next 189.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T47b — the guild in the owner's town view, protocol v78, the hygiene oracle's guild fix (2026-11-21)

One session, the user present at the start; branch `town/e09-t47b`, `cross-lane` with the user's
consent (the server lane's `h3-protocol`: `wire/game.rs`, `wire/game/fold.rs`, `map/game.rs`,
`lib.rs`, `tests/game.rs`, `tests/game_event.rs`, `tests/game_fold.rs`; the adventure lane's
`h3-map` `src/tests/view.rs` and `proptest-regressions/tests/view.txt`). Starting point: `master`
at cfd2f0c (T47a2's #1097 merged); CI's fingerprint not yet on `master`, so the setup's full
baseline ran (its tests passed; stopped at coverage for disk). The user's computer was connected
at the start only: 71 files staged; its shell did not start ("Workspace unavailable"), so the sizes
were compared with the listing and the LODs and executables with `docs/formats/installation.md`'s
SHA-256. **T47b kept whole, with the inbox's guild fix** (decision 189).

- **The view** (design §94, TWN-VEW-004 amended): `h3_game::TownBuildingsView` gains `guild` (an
  `h3_towns::GuildSlots` of the shown slots: `h3_towns::shown_slots`, each level's `shown` spells
  in its first slots, in slot order), `aurora` (`h3_towns::has_aurora`: a building of role
  `aurora` ever built) and `vortex` (`Town::vortex`, raw in every town); `GuildSlots::with` public,
  `h3_game` re-exports `GuildSlots`. Not the design's `[Vec<SpellId>; 5]`: `GameEvent` is `Copy`
  and `TownChanged` carries the view; `GameEvent` grows from 72 to 104 bytes (`TownChanged` its
  largest variant). No `Told` change: a build, an event and a capture tell the owner's view already.
- **The protocol v78** (design §95): `wire::game::TownBuildings`' `guild: [Vec<u16>; 5]` (the
  handles by level), `aurora`, `vortex`; no longer `Copy` (the fold's `TownChanged` arm binds it by
  `ref`, `clone_from`); the round trips (`guild_back`) and the fold tests with a guild.
- **The hygiene oracle** (`h3-map`'s `a_viewer_learns_nothing_it_cannot_see`): its
  `TownBuildingsView` with the three fields; its comparison ends when the viewer is shown a Thieves'
  Guild (`ThievesGuildSeen`, a Den of Thieves' visit), as at a hire (the adventure lane's report of
  E08 T135a: the guild's rows show a hero whose experience the hidden difference varied). The
  report's seed `cc 9b8b2dc9…` added: it fails without the break and passes with it.
- **Tests:** `h3-towns`' `the_shown_slots_hold_each_level_s_shown_spells_in_order_and_nothing_else`
  and `a_town_has_the_aurora_once_its_building_was_built`; the game's
  `the_owner_sees_what_its_guild_shows_told_at_each_build_and_another_player_nothing` (a level 1
  guild shows its slot, the Library its spare, the Aurora its mark; each told once to the owner,
  blue the public mark alone); the first view test's expected value; `h3-towns`, `h3-game`,
  `h3-protocol`, `h3-map`'s view tests, `h3-server`, `h3-sim`, `h3-files`, `h3-cli` tested.
- **Benchmarks:** the 38 adventure workloads under the gate, none re-baselined, each as T47a2's
  (the largest `random-play-bonuses-36` +1.67 %, `legal-towns-36` and `day-start-builds-36`
  +1.45 %): a sink that keeps nothing reads no view, so the larger event costs nothing there.
- **Verify:** the scoped verify with original data, every step passing but `check-lane` (by
  design; `--cross-lane` ok): coverage 20.5 minutes, determinism 10.3, bench-check 16. The
  setup copies four original files only: `h3-pack-adventure`'s check skipped until
  `extracted.tar` was untarred into the clone, then passed with the harvest check.
- **Inbox:** the server lane (v78, `TownBuildings` not `Copy`), the adventure lane (the oracle's
  break and fields); the town lane's own note of 2026-11-21 archived (`QUEUE-ARCHIVE.md`).
- *Agent's calls* (decision 189): the view's guild as the record's `GuildSlots`; the wire's as
  handles by level; the Vortex's flag raw in every town.

**The queue:** T47b out; T48a next (learning: the Aurora, T-18, the town's own triggers). Ids:
decisions next 190.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data`; CI on the pull
request is the record.

### T48a — heroes learn the guild's spells at the town's own triggers (2026-11-21)

One session, the user present at the start; branch `town/e09-t48a`, `cross-lane` with the user's
consent (the adventure lane's `h3-game` `src/state.rs`, the start's call, and
`src/state/battle.rs`, `GameState::change_hero` made `pub(crate)`; its `h3-map` `src/tests/play.rs`
and `src/tests/view.rs`). Starting point: `master` at
c0ce729 (T47b's #1102 merged; CI green, so the setup ran the light gates only). The user's
computer was connected at the start only: 71 files staged (the four LODs and both executables
checked by SHA-256 against `docs/formats/installation.md`, the rest by size). **T48a kept whole**
(decision 190).

- **What a hero learns** (design §88, TWN-GLD-004 complete): `h3_towns::teaches(town, statics,
  part, core, details)` in the guild module: nothing without a building of role `guild` ever built
  (or without the faction's record); with one of role `aurora` ever built the town's Aurora set
  (`TownStatic::aurora`, T-18 read into it at the build) in spell order, else `shown` level after
  level in slot order; each spell through `h3_heroes::can_learn_spell` with the known spells
  counting (the worn book, Wisdom + 2, a spell not learned). `h3-towns` depends on `h3-heroes`
  (allowed by `arch.toml` already; both lockfiles). The fixture's `guild_core` gained Wisdom, a
  class, the type `SAGE` and a spell book (`fixture::sage`).
- **When** (design §89, TWN-GLD-005 new): `h3_game`'s `town::guild::learn(state, rules, town,
  sink)` teaches the visitor, then the garrison hero, each spell `HeroChanged` to the hero's owner
  through `GameState::change_hero`. Called at the end of `town::command::build_told` (a command's
  build and each building of a town event, after `TownBuilt` and the views), after
  `town::hire::hire`'s placement (the map Tavern's hire none), at the end of `town::rows::heroes`
  while a visitor stands after the move (`Leave`, `Swap`), and by `town::guild::at_start` in
  `GameState::new` once the state is assembled (each generating main town in player order, before
  the start's tavern fill; `NoopSink`). `TWN-STUB-6` and `TWN-STUB-15` lifted.
- **Catalog:** TWN-GLD-004 (its learning), TWN-GLD-005 (new); TWN-SET-009, TWN-TAV-003 (its stub
  sentences replaced), TWN-BLD-005, TWN-EVT-003, TWN-ROW-002 to -004 amended.
- **Tests:** `h3-towns`' three (`a_hero_with_a_book_learns_what_the_guild_shows_up_to_its_wisdom_level_then_slot`,
  `nothing_is_taught_without_a_guild_ever_built_or_the_faction_s_record`,
  `the_aurora_teaches_its_set_in_spell_order_at_any_guild_level_up_to_wisdom`); the game's six in
  `town/tests/learn.rs` (a build's and the Library's, the Aurora past the guild's level and a hero
  without a book, a town event's, the bars with a new day teaching nothing, a hire's two heroes in
  order, TD-10's start with a placed hero not taught); the workspace's tests but `h3-map`'s outside
  the verify.
- **The plans** (`h3-map`, found by the verify's coverage run): `check_timed` expects, after each
  town event's `TownBuilt`, the visitor's then the garrison hero's `LearnSpell`s as `teaches`
  gives them (the town as built so far with its guild, the spells learned so far in the command);
  the details' check takes the owner of a hero the command hired from the state after it; the
  views' fold takes such a hero as the viewer's own and folds its learned spells after its parts
  (`check_hired`). The 199 lib tests pass in release.
- **Nothing moved:** the 111 golden adventure replays reproduce their hashes (none meets one of
  these triggers with a guild built and a hero with a book there), so no replay, bot report or
  version; `h3-sim`'s town-map tests unchanged.
- **Found, pre-existing:** `a_viewer_learns_nothing_it_cannot_see` failed on a random case
  (`cc cc8693ab…95e5`) that fails with `master`'s `h3-game`, `h3-towns` and `h3-map` too (and with
  the learning switched off): a captured town's garrison stacks told differently in the two games.
  Not committed to `view.txt`; the lane's inbox holds it for triage.
- *Agent's calls* (decision 190): `learn` without a hero list; the build's learning after its
  events; the hire's after the placement's; the bars' only with a visitor after the move; the
  start's before the tavern fill, telling nothing. `h3-map`'s census example still names
  `TWN-STUB-6` in two comments (the adventure lane's file): T50's census of the start's learning
  rewrites them.
- **Benchmarks:** the 38 adventure workloads inside the gate, none re-baselined (the largest
  `day-start-builds-36` +1.68 %, `random-play-bonuses-36` +1.67 %, `legal-towns-36` +1.45 %, as
  T47b's).
- **Verify:** the scoped verify with original data: its first run failed `check-rules` (a VCMI
  source without its URL, a `covers:` of a quirk id), `clients-lock` (`h3-towns`' new dependency)
  and coverage (the plans above); determinism (5 minutes) and bench-check (12) passed. After the
  fixes `check-rules`, `check-lane --cross-lane` and the formatting pass, and coverage again
  alone (18 minutes; 130 gated files at 100 %, `h3-map`'s tests passing in it); the workspace's
  other tests (`cargo test --workspace --exclude xtask --exclude h3-map`) passed before.

**The queue:** T48a out; T48b next (the visit's end, a capture, a siege's end). Ids: decisions
next 191.

### T48b — the visit's end and a siege's end teach the guild's spells (2026-11-21)

One session, the user present at the start; branch `town/e09-t48b`, `cross-lane` with the user's
consent (the adventure lane's `h3-adventure` `src/command.rs`, `AdventureOutcome::town_entered`,
and `src/state/tests/towns.rs`; its `h3-game` `src/state.rs`, `settle_outcome`, and
`src/state/battle.rs`, `end_battle`). Starting point: `master` at 35bedb9 (T48a's #1115 merged;
CI green, so the setup ran the light gates only). The user's computer was connected at the start
only: 71 files staged, its shell down ("Workspace unavailable"); the LODs' and executables'
SHA-256 checked against `docs/formats/installation.md`. The draft was measured before the cut was
asked (no golden replay moved); **T48b kept whole** (decision 191).

- **The visit's end** (design §89's first row, TWN-GLD-005): `visit_town` (the town lane's
  `h3-adventure` `town.rs`) sets the new `AdventureOutcome::town_entered` for every entered town
  (`Visit::Enter`: a step or a `Revisit` into a town of the team, the undefended capture);
  `GameState::settle_outcome` calls `town::guild::learn` for it right after `town::capture`.
- **A siege's end** (§89's second row): `GameState::end_battle` calls `learn` for the battle's
  town (a siege only) after the outcome, the unowned battle's end and the attacker's held capture,
  before `give_levels` (the original's `town_visit` 0x4AB3C3 before `hero_level_up` 0x4AB3CB): a
  defending visitor or garrison hero that won learns; an attacker's win teaches nobody (the
  winner outside, TWN-CAP-003); the capture after the levels (`town_won`) sets no `town_entered`.
- **Catalog:** TWN-GLD-005 (the two triggers in, out of "Planned").
- **Tests:** the game's three in `town/tests/learn.rs` (a step into an own town and a `Revisit`, a
  second `Revisit` teaching nothing more; an undefended capture after the capture's events and
  views; a siege's end: the defender's win teaches the visitor, no siege nothing, the attacker's
  win nobody); `h3-adventure`'s three town tests (`state/tests/towns.rs`) expect the new field.
- **Nothing moved:** the 111 golden adventure replays reproduce their hashes (measured on the
  draft, in release), `h3-map`'s 202 lib tests pass in release; no version, no bot report.
- **Found, pre-existing** (not this task's): with `Heroes3.exe` staged, `check-exe-map` fails on
  five addresses E31's refinement cites (CI skips it without the executable); the AI lane's inbox.
- *Agent's calls* (decision 191): the learning after the capture; the siege's for the battle's
  town before the winner's levels, whoever won (design §89's level-up paragraph had guessed a
  Wisdom counts at once: the engine's level-ups are decisions, so it counts at the next trigger,
  as the original's).

### T49a1 — the night's refill and the Mana Vortex at the visit's end, the builds and the hire (2026-11-21)

One session, the user present at the start; branch `town/e09-t49a1`, `cross-lane` with the
user's consent (the adventure lane's `h3-game` `src/state/day.rs`, `advance_day`, and
`src/state.rs`, `settle_outcome`; `docs/rules/catalog/game.toml`'s GAM-CAL-002; `h3-map`'s
`src/tests/play.rs` and `view.rs`). Starting point: `master` at a9d593d (T48b's #1120 merged;
the setup's full baseline passed every step up to coverage, stopped there for time and disk; CI
on `master` running). The user's computer was connected at the start only: 71 files staged, its
shell down ("Workspace unavailable"); the LODs' and executables' SHA-256 checked against
`docs/formats/installation.md`. **T49a cut in two** at its start (decision 192): this task the
refill and the Vortex, T49a2 `Open` whole.

- **The rules** (`h3-towns`' guild module): `refills(town, part)` (TWN-GLD-008: a building of
  role `guild` ever built) and
  `vortex(town, part, core, details)` (TWN-GLD-009: the parameters, the week flag and a standing
  `mana_vortex` before the maximum; `vortex_factor × max_mana`, saturating); the fixture's
  `vortex_keep` (the Mana Vortex at 31). `Town::with_vortex` public (the oracles').
- **The game** (`town::guild`): `refilled` (every town in `TownId` order whose night refills,
  its visitor then its garrison hero), read by `GameState::regain_mana`, which gives each its
  points or its maximum, whichever is more, in place of the regain (the same result; silent).
  The first draft's separate pass after the regain failed the bench gate (`day-start-builds-36`
  +5.37 %, `day-start-towns-36` +3.25 %, `month-start-builds-36` +2.83 %: `max_mana` twice per
  refilled hero); `vortex_to` (the first of the given heroes it raises:
  `HeroChanged`, `take_vortex`, the town's `Told`, taken only then), `vortex` (the garrison hero,
  then the visitor) and `visit` (the Vortex, then the learning). Callers: `build_told` after its
  learning; `hire` after its learning, the hired visitor alone; `settle_outcome`'s
  `town_entered` through `visit`. `advance_day` sets every town's flag
  (`Towns::start_week`) on a week's first day after the towns' growth, before the taverns' week:
  `Told::before` was taken before the growth, so the week's views tell the set flags.
- **Catalog:** TWN-GLD-008 and -009 new (`implemented`; TWN-5-5's observation among their
  sources); TWN-GLD-005 (the visit's end runs the Vortex first), TWN-BLD-005, TWN-EVT-003,
  TWN-TAV-003 and GAM-CAL-002 amended.
- **Tests:** `h3-towns`' two (a superseded guild counts; the factor, the flag, a superseded or
  unbuilt Vortex, no record, no parameters); the game's five in
  `town/tests/mana.rs` (the night raises a garrison hero and an allied town's visitor, a turn's
  end no night, above the maximum kept; no guild and a hero beside the town nothing; the visit's
  end over two weeks: the points, the told flag before the learning, nothing until day 8, the
  week's view to red alone, the threshold; a build's Vortex to the garrison hero first, at its
  threshold the visitor; a hire's to the hired hero alone, a move between the bars nothing).
- **The oracles** (`h3-map`, three properties failed on the draft): the conservation's
  `DayStarted` arm applies `night_refill`; a hire's check expects the flag cleared when the
  hired hero took the Vortex; the hire fold sets the hired view's points at a `SetSpellPoints`.
  The lane's `TRAPS.md` ("A silent write of a hero's details meets two oracles").
- **Nothing moved:** the 111 golden adventure replays reproduce their hashes (measured on the
  draft, in release); no version, no bot report.

### T49a2 — the town screen's opening, `TownCommand::Open`: the rule, its listing, the wire, the word, the env's entry (2026-11-22)

One session, the user present at the start; branch `town/e09-t49a2` (`cross-lane`: the adventure
lane's `h3-heroes` `spells.rs`, `h3-files`' replay word, `h3-sim`'s replay version and
`src/adventure/` (action encoding v18, the AI lane's paths), the 111 goldens' `format` line,
`h3-map`'s `tests/play.rs`, `h3-cli`'s literals; the server lane's `h3-protocol`; the adventure
workload `legal-towns-36`; with the user's consent). Starting point: `master` at e478f97 (#1124
merged; the setup's full baseline passed every step up to coverage, stopped there by a watcher;
CI on `master` was running). The user's computer was connected at the start only: its shell did
not start ("Workspace unavailable"), 71 files staged (SHA-256 checked against
`docs/formats/installation.md`), the repo cloned from GitHub. The agent recommended a cut in two
(the rule with the wire and the word, then the listing with the env and the benchmarks); **the
user kept T49a2 whole** (decision 193).

- **The town's half** (`h3-towns`): `TownCommand::Open { town }` (no new refusal);
  `opening_gives` (the listing's question for a town and its heroes: `vortex` or `teaches` for
  one of them), read through the part's derived `GuildRoles` per record (`ruleset.rs`, not
  hashed, as the Portals: the roles `guild`, `mana_vortex`, `aurora`, `library` and the
  buildings that supersede a guild or a Vortex, `town_with_guild_roles`); `SpellSet::difference`,
  `is_empty`, `BuildingSet::intersection`; `teaches` takes the known spells first and reads the
  book and Wisdom once (`learnable`), the guild's counts once (`shown_counts`, `counts_of`,
  `shown_at`, `shown_levels`; `shown` and `shown_slots` on them); `vortex` tests ever-built
  before the standing set. The same answers (the guild tests pass unchanged).
- **The heroes** (`h3-heroes`, the adventure lane's): `spell_learner` (the book and the
  learnable level once; `can_learn_spell` delegates); `wears_book` counts per copy (the same
  `any`, no set of types).
- **The game** (`h3-game`'s `town/guild.rs`): `open` (the store's town, then the owner, then
  `visit`: the Vortex with its told flag, then the learning) dispatched from `town::command`;
  `opens` (the garrison hero, the listing's visitor, then `opening_gives`), chained in
  `legal_builds` after the hires and before the moves. `legal_builds` finds the hero on each
  town's entrance once (`AdventureState::town_entrance_hero`, the town lane's `rows.rs` in
  `h3-adventure`) for the hires (`legal_hires`' `free`), the opening and the moves
  (`town_hero_moves_at`; `town_hero_moves` on it).
- **The wire** (protocol **v79**): the command both ways. **The replays:** format **v24**
  (`town open <town>`), the 111 goldens' `format` line only, `h3-cli`'s and `h3-sim`'s literals.
  **The env** (encoding **v18**): `TOWN_OPEN` = 103, appended after E08 T135a's entries
  (design §95's *As built*), `tileless` ending before it, `tile_mask`, `decode`, `encode`,
  `Action::town_open`; the layout test and the mask property (`check_town_heroes`).
- **The plans** (`h3-map`'s `play.rs`): `town_builds` probes every town's opening after its
  hires; `accepted_builds` keeps an accepted opening only where the probe changed the state; an
  unlisted opening may be accepted changing nothing; `check_town_open` reads a listed one apart;
  `Step::Town` plays a listed opening in one step of eight (`n % 8 == 6`). All of `h3-map`'s
  tests passed; the play and view blocks once at 240 cases (24 minutes).
- **Tests:** the game's four (`town/tests/open.rs`); `h3-towns`'
  `an_opening_gives_exactly_where_the_vortex_or_the_guild_would` (4 parts, 9 built sets, both
  flags, 18 heroes alone and in pairs against `vortex` and `teaches`); `h3-files`' round trip
  and four refusals; the protocol's round trip; the env's layout. `h3-sim`'s 130 tests passed
  unchanged: the random games on the town map list no opening, so their seeds and counts hold.
- **Catalog:** TWN-GLD-006 entered `implemented` (the code reading, batch TWN-5's TWN-5-5, VCMI,
  the wiki); TWN-GLD-005 and -009 name the opening.
- **The bench** (`master` built in `target/wt-master`, per function): the first build cost
  `legal-towns-36` +49 % on the same games (3 360 steps, 2 653 listings, 12 720 town commands,
  no opening listed): `vortex` and `teaches` hero by hero (`guild_level` once a level,
  `can_learn_spell`'s set of the hero's effects per spell, `vortex`'s standing set in every town).
  The design pass, measured after each step: the counts once and the book once (+27 %), known
  spells first and the shared entrance lookup (+14 %), `opening_gives` in one walk (+8 %), the
  part's derived roles (+5.25 %: master 23 792 510 in the same sandbox, 25 042 654; about 470 Ir a
  listing, the rule's own cost); re-baselined with the reason. Every other workload within its
  gate (`random-play-bonuses-36` +1.67 % the largest, as at T38a2).
- **Versions:** protocol v79, adventure replay format v24, adventure action encoding v18; the
  adventure lane's planned v79, v24 and v18 (E08 T143, T144) become v80, v25 and v19 (its
  inbox, the AI lane's and the server lane's). No hash moved.
- **Found:** a stray `<<<<<<< HEAD` line in the AI lane's `QUEUE.md` on `master` (no other
  marker): removed with this task's inbox note there.

### T49b — a spell book bought at a town's Mage Guild, `TownCommand::BuySpellBook`: the rule, its listing, the wire, the word, the env's entry (2026-11-23)

One session, the user present at the start; branch `town/e09-t49b` (`cross-lane`: the adventure
lane's `h3-heroes` `spells.rs`, `h3-files`' replay word, `h3-sim`'s replay version and
`src/adventure/` (action encoding v19, the AI lane's paths; the town map's seed), the 111 goldens'
`format` line, `h3-map`'s `tests/play.rs`, `h3-cli`'s literals; the server lane's
`h3-protocol`; the adventure workload `day-start-builds-36`; with the user's consent). Starting
point: `master` at f1c4a53 (#1128 merged; the setup's light gates only, CI's fingerprint on
`master`'s code). The user's computer was connected at the start: its shell worked this time;
71 files staged (SHA-256 checked against `docs/formats/installation.md`), the repo cloned from
GitHub. The agent recommended a cut in two (the rule with the wire and the word, nothing listed;
then the listing with the env, the seeds and the benchmarks); **the user kept T49b whole**
(decision 194).

- **The town's half** (`h3-towns`): `TownCommand::BuySpellBook { town }`, the refusals
  `NoGuild`, `NoHero`, `HasSpellBook`; `check_spell_book` (a standing guild through the part's
  derived roles, the part's `spell_book_price`); `guild_offers` with `GuildOffers` (the
  listing's two questions, the opening's gift and the book's price, the derived roles and what
  supersedes the guild read once: the private `GuildRead`, `Gift`; `opening_gives` on them, the
  same answers). The fixture's `building` and `roles` crate-public for a test.
- **The heroes** (`h3-heroes`, the adventure lane's): `wears_spell_book` (the private
  `wears_book`) and `spell_book` (the ruleset's first artifact with the `spell_book` effect).
- **The game** (`h3-game`'s `town/guild.rs`): `buy_book` (the store's town, the owner,
  `check_spell_book`, the ruleset's book, the visitor else the garrison hero, its book, the
  treasury; then the price, `AddArtifact`, the learning) dispatched from `town::command`;
  `offers` (both heroes, then `guild_offers`, then the treasury, the ruleset's book and the
  hero's own book last) replacing T49a2's `opens`, chained in `legal_builds` after the hires and
  before the moves, the opening and the book carried as town commands and mapped once.
- **The wire** (protocol **v80**): the command both ways, the three refusals. **The replays:**
  format **v25** (`town book <town>`), the 111 goldens' `format` line only, `h3-cli`'s and
  `h3-sim`'s literals. **The env** (encoding **v19**): `SPELL_BOOK` = 104 after `TOWN_OPEN`,
  `tileless` ending before both, `tile_mask` (both through `guild_command`), `decode`, `encode`,
  `Action::spell_book`; the layout test and the mask property (`check_town_heroes`).
- **The town map's seed** (`h3-sim`'s `tests/towns.rs`): seed 2541 met no siege won once the
  agent bought books; a search over the seeds 1 to 3 000 (a throwaway release test without the
  masks, 510 s) found four meeting every feature, 277 the shortest (657 steps, 30 builds, 29
  dialogs, 9 hires, a book); `Played` counts the purchases and the test asks for one. The fork
  test's seed 6 held.
- **The plans** (`h3-map`'s `play.rs`): `town_builds` probes every town's purchase after its
  opening; `check_town_book` reads a listed one apart; `check_conservation` takes its price;
  `Step::Town` plays a listed purchase in one step of eight (`n % 8 == 7`).
- **Tests:** the game's four (`town/tests/book.rs`); `h3-towns'`
  `a_spell_book_is_sold_where_a_mage_guild_stands_for_the_part_s_price` and the opening's test
  checking `guild_offers` against `opening_gives` and `check_spell_book` (4 parts, 9 built sets,
  18 heroes alone and in pairs); `h3-heroes'` Titan's Thunder test with the two queries;
  `h3-files`' round trip and four refusals; the protocol's round trip and rejections (149).
- **Catalog:** TWN-GLD-007 entered `implemented` (the code reading, VCMI, the wiki and the
  manual); TWN-GLD-005 names the purchase.
- **The bench** (`master` built in `target/wt-master`, per function; `legal-towns-36` measured on
  `master`'s games with the agent made to skip the purchase): +5.54 % at the first build
  (`sells_book` and `check_spell_book` for every owned town, the book's price once a listing,
  the chain carrying two `Option<GameCommand>`s and `legal_builds` no longer inlined), +4.75 %
  with the commands carried as town commands, +3.63 % with the price asked after the guild,
  +2.77 % with the opening and the book reading the derived roles once (`guild_offers`), +1.58 %
  with the hero's own book (an effects walk, about 170 Ir) asked last: inside the gate, no
  re-baseline. With the purchase in the agent's draw every workload within its gate but
  `day-start-builds-36` (-2.54 %, the games changed course: master 3 528 759 in the same
  sandbox, the same functions in the same shares), re-baselined with the reason;
  `random-play-bonuses-36` +1.65 % the largest, as at T49a2.
- **Versions:** protocol v80, adventure replay format v25, adventure action encoding v19; the
  adventure lane's planned v80, v25 and v19 (E08 T143, T144) become v81, v26 and v20 (its
  inbox, the AI lane's and the server lane's). No hash moved.
- **CI's first run** (#1136) failed in `every_step_keeps_the_commands_invariants` (seed `cc
  8b2ce4c0…`, an AB plan of side 16): a town hire in a Dungeon town took the Mana Vortex (T49a1)
  and cleared the week flag, which `check_towns` (the oracle over every command) did not know;
  the plans' games met it once the books changed their course. `check_towns` sets the flag aside,
  allowing it to clear with a `SetSpellPoints` told for the town's hero and to be set at a
  week's start, told to the owner; the seed added to `proptest-regressions/tests/play.txt`.

### T50 — the guild map and its replays, the census, the properties, the benchmarks; the hygiene failure (2026-11-24)

One session, the user present at the start; branch `town/e09-t50` (`cross-lane`: the adventure
lane's `h3-map` plans (`tests/play.rs`, `view.rs`), census (`examples/original_data.rs`) and test
modules, `h3-cli`'s replay list, the adventure workloads' directory; the server lane's
`h3-protocol` and `h3-server` golden counts; with the user's consent). Starting point: `master`
at d202566 (#1136 merged; the setup's light gates, CI's fingerprint on `master`'s code). The
user's computer was connected: the five files the town lane's gates read staged (`Heroes3.exe`,
`H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`; SHA-256 checked against the PC's).
The agent recommended a cut in two (the census and the properties first, as T40a); **the user
kept T50 whole, folded the inbox's hygiene failure in and asked for sub-agents** (decision 195):
four, one after another (the failure, the properties, the census, the map with its replays and
workload), each with a narrow brief. The sandbox VM restarted twice and the session's server
once; nothing was lost (each part committed and pushed as it came).

- **The hygiene failure** (the lane's inbox, 2026-11-21; `view.rs`'
  `a_viewer_learns_nothing_it_cannot_see`): the property's, not the engine's. Hero 4 is red's
  (the inbox read it as blue's), entering the garrison of the town 1 red had captured; blue sees
  the entrance (`HeroRemoved`) and owns a Tavern, so it is told the town at level 1, whose
  garrison is the garrison hero's army (TWN-VEW-002; `docs/06-security.md`'s Towns lines). `vary`
  changes every other player's hero army (ADV-FOG-006 shows none on the map), so the two games
  told five stacks and two, both by the rules. The comparison now ends where the told towns'
  upper bars differ between the games, the events the same but `TownStackSeen` (the lane's
  `TRAPS.md`); the seed added to `proptest-regressions/tests/view.txt`. *For the next `shared/`
  task:* `docs/06-security.md`'s "never its army (ADV-FOG-006)" may add "but a garrison hero's,
  as its town's garrison at the viewer's level (TWN-VEW-002)".
- **The properties** (`play.rs`): `check_guilds` from `check_invariants` (the slots: at each level
  the filled ones first, at most the part's count and the spare, each a hero spell of that level,
  no ability, none granted by name only, none twice, each in the town's Aurora set, where the
  town's bans are read; what the guild shows by an independent reading of the guild level and
  the Library, against `h3_towns::shown` and `shown_slots`); `check_guild_step` after
  `check_towns` (every hero record of the store keeps its spells, first and in order: learning
  never forgets; the Mana Vortex's gifts counted per town and week, a Magic Spring's entrance set
  aside); `check_guild_refusals` (the system, a player past the last and every player who cannot
  act, each on one town per state: the state, its hash and the events kept), and the refused
  probes of `check_town_builds` now hash one probe per state; the forks' guild slots and Aurora
  sets a fresh build's (the forks compared the whole state and its hash already). The plans'
  maps ban and require no spells, so TD-25's cases stay the unit tests' and the census's.
  `every_step_keeps_the_commands_invariants` 345 s before, 348 s after (a first draft probing
  every town for every actor: 444 s, +29 %); the three properties at 240 cases passed (927 s).
- **The census** (`original_data.rs`: `map_guilds`, `original_spells`, `built_guilds`,
  `guild_drawn`, `guild_shown`, `aurora_of`, `start_learning`, `initial_spells`,
  `guild_differences` with 25 of the survey's counts): each map's raw spell sets against the
  import's; the draw per level (no ban of the town's or the scenario's, none never drawn:
  abilities and id 57, none of another level, none twice; the required spells first in spell
  order, then by weight; an empty slot only with nothing left); what each guild shows; the Aurora
  sets; the start's learning. All 160 hold, every count as survey §1 and §10 (210 scenario bans
  on 26 maps; required spells in 32 towns, 34 bits; banned in 396 towns, 1 779 bits; `Faeries`'
  town requiring and banning one, never drawn; 249 guilds standing, 142 owned; 5 Libraries; one
  Vortex; 561 heroes on an entrance, the 33 main towns' heroes with a book learning 188 spells,
  the 8 placed ones nothing) but one: the 14 short towns are all empty, where §1 kept four spells
  for the two neutral ones (seed 0 draws both as Strongholds, which weigh those 0): the 12 with
  every spell excluded checked, the 14 printed. 37.6 s before, 36 to 39 s after (debug). Three
  mutations fail it: the draw ignoring the town's bans (54 maps), the start's learning skipped
  (20), the learning given to every town's heroes (exactly the 5 maps of the placed heroes).
- **The guild map** (`fixtures/maps/e09-guild-36.h3m`, `h3-map`'s `guild_map.rs`; SoD, 36 × 36,
  the scenario bans Haste): red's Tower "Athenaeum" (Mage Guilds 1 to 4, a Tavern; requires Town
  Portal, Shield and Bless, bans Magic Arrow and Shield), red's Dungeon "Abyss" (Mage Guild 1,
  the Mana Vortex), red's Conflux "Prism" (Mage Guild 1, the Aurora Borealis; bans Magic Arrow
  and Lightning Bolt: an Aurora set of 67, Haste in it by `TWN-QUIRK-14`), blue's Necropolis
  "Crypt"; red's Geon (a book), Lorelei and Valeska (none), Serena; blue's Sorsha with a Pikeman.
- **The replays** (seed 1, each with its test): `guild-start-s1` (the start's learning, TD-10),
  `guild-visit-s1` (the Aurora's set to level 3), `guild-builds-s1` (the Library's slot at levels
  1 to 4, the Abyss' Mage Guild 2 and its Vortex), `guild-hire-s1` (the hired hero learns after
  `HeroHired`), `guild-book-s1` (`NoHero`, the book for 500 gold before the learning,
  `HasSpellBook`), `guild-vortex-s1` (20 commands: the opening's Vortex, the flag set again on day
  8 only, a second opening changing nothing), `guild-siege-s1` (a siege's end teaches, the flag
  kept, the night's refill past the day's regain). The goldens to 118 (`h3-protocol`'s fold and
  session, `h3-server`'s record, the server lane's); their suites pass.
- **The benchmarks:** `day-start-guild-36` (new: the step that starts day 4 on the guild map in
  16 random games, seed 1; 1 326 586 Ir; the refill 8.1 %, the day's regain 34 %); the random
  agent spends no spell points, so there the refill raises no one and the workload measures its
  walk and the maxima (the replays show its effect). No engine code changed: every other workload
  inside its gate (`random-play-bonuses-36` +1.65 %, `fork-economy-xl-144` +1.28 %,
  `manage-army-12` +1.00 %, as at T40a); the build's draws and the listing's guild commands were
  measured by T47a2 to T49b.
- **Versions:** none moved. Ids: none new.
- **Verify** (`--scope town --with-original-data`): every step passed (fmt, clippy, the checks,
  test, coverage 1 252 s, determinism 382 s, bench-check 799 s, doc-drift, original-data 70 s) but
  `check-lane` (`cross-lane`: `cargo xtask check-lane --cross-lane` ok, `cargo check --workspace
  --all-targets` clean).
- **After merging `master`** (2026-11-24; E08 T141a, #1132: the Prison, game state hash v39,
  adventure state hash v28, import v29, protocol v81, encoding v20): one textual conflict in
  `original_data.rs` (the census beside T141a's Prisons; both kept); the seven `guild-*-s1`
  rehashed from their own commands (only the hash and version lines moved); `day-start-guild-36`
  re-baselined, 1 326 586 to 1 286 173 Ir (-3.05 %: the random games changed course, as T141a's
  three day-start re-baselines); the map's tests, the replays, `h3-protocol`, `h3-server` and the
  census pass.

**The queue:** T50 out; T51 next (the audit against §99's criteria, increment 5 closed, increment
6 refined). Ids: decisions next 196.

### T51 — the audit; increment 5 closed; increment 6 refined (2026-11-24)

One session, the user present at the start; branch `town/e09-t51`, docs only, with the README's
row in `shared/e09-t51-readme`. Starting point: `master` at 25397e7 (T50's #1144 merged, then
E08 T141b's #1147). The user's computer was connected: the five files the town lane's gates read
staged (`Heroes3.exe`, `H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`; SHA-256
checked against the PC's). The sandbox VM restarted once while the setup's baseline ran (its
coverage step lost); the scoped verify ran again (below). **At its start** (the user, asked with
a first look at the audit, `TWN-Q34`): T51 whole (decision 196), the services in two increments,
6 the town's trades and 7 the buildings for heroes and the map, travel and the Grail 8 (decision
197), the first round as before (decision 198).

**The audit of increment 5's exit criteria** (design note §99), on `master`'s code:

1. **The part.** `h3-towns`' `the_guild_weights_are_the_cores_hero_spells_by_index`,
   `the_guilds_parameters_are_checked`, `the_guilds_roles_are_checked`, `the_guilds_resource_
   rows_are_the_cores_each_in_at_most_one_row`; `h3-pack-towns`' `the_original_guilds` (in
   `the_original_packs_compile_with_every_faction`: the weights' sums, the roles, the slots, the
   price and the factor, both editions); part v9, TWN-GLD-001 `verified` by T46. **Holds.**
2. **The draw.** `h3-towns`' `each_level_draws_its_weighted_spells_once_then_leaves_its_slots_
   empty`, `a_slot_takes_the_first_spell_whose_running_weight_reaches_the_roll`, `a_required_
   spell_is_taken_first_in_spell_order_whatever_its_weight`, `td_25_a_spell_the_town_requires_
   and_bans_is_never_in_its_guild`, `the_scenario_s_bans_and_a_spell_granted_by_name_only_are_
   never_drawn`, `a_level_without_a_spell_left_is_empty`, `a_seed_draws_alike_and_seeds_differ`;
   the game's `every_town_s_guild_is_drawn_at_the_build_from_the_towns_stream`, `the_scenario_s_
   bans_leave_every_guild_and_the_aurora_set_by_the_quirk`, `a_town_spell_is_a_hero_spell_of_the_
   ruleset`; `h3-map`'s `a_towns_required_and_banned_spells_are_the_maps` and `a_town_s_spells_
   and_the_map_s_banned_spells_read_back_as_written` (TWN-SET-012); the plans' `check_guilds`;
   the census (T50: no excluded spell in the 160 maps' guilds, the required spells first,
   `Faeries`' required and banned spell never drawn). **Holds.**
3. **What the guild shows.** `the_guild_level_is_the_highest_standing_mage_guild`,
   `each_built_level_shows_the_part_s_slots_and_the_library_one_more`, `an_empty_slot_ends_its_
   level`, `the_shown_slots_hold_each_level_s_shown_spells_in_order_and_nothing_else`; the guild
   map's `a_library_and_a_guild_level_built_with_visitors` (`guild-builds-s1`); `check_guilds`'
   independent reading of the guild level and the Library. **Holds.**
4. **Learning.** `a_hero_with_a_book_learns_what_the_guild_shows_up_to_its_wisdom_level_then_
   slot`, `nothing_is_taught_without_a_guild_ever_built_or_the_faction_s_record`, `the_aurora_
   teaches_its_set_in_spell_order_at_any_guild_level_up_to_wisdom`, `twn_quirk_14_the_aurora_
   set_holds_the_scenario_s_bans_only_with_the_flag_on`; the game's `the_aurora_teaches_past_the_
   guild_s_level_and_a_hero_without_a_book_nothing`; `a_visit_learns_the_aurora_set`
   (`guild-visit-s1`); `check_guild_step` (learning never forgets). **Holds.**
5. **The triggers.** `town/tests/learn.rs`' nine (each build, a town event's build, the bars'
   moves only with a visitor and a new day nothing, a hire, TD-10's start, a step and a
   `Revisit`, an undefended capture, a siege's end whoever won), `open.rs`' and `book.rs`'
   learning; the guild map's `the_starting_heroes_learn_at_the_start`, `a_hire_at_the_tower_
   learns`, `a_siege_won_by_the_defender_teaches`; the census's start (the 33 main towns' heroes
   with a book learn 188 spells, the 8 placed ones nothing). **Holds.**
6. **The commands.** `an_opening_gives_the_vortex_to_the_garrison_hero_first_then_teaches_the_
   visitor_then_it`, `an_opening_teaches_a_garrison_hero_alone_and_a_town_without_a_guild_
   nothing`, `each_refusal_of_an_opening_changes_nothing`, `the_legal_commands_list_an_opening_
   only_where_it_would_change_something`; `a_purchase_gives_the_visitor_the_book_for_the_price_
   then_teaches_it_at_once`, `the_visitor_is_offered_the_book_before_the_garrison_hero`,
   `each_refusal_of_a_purchase_changes_nothing` (`NoGuild`, `NoHero`, `HasSpellBook`),
   `the_legal_commands_list_a_purchase_where_it_is_accepted_after_the_opening`; `h3-towns`'
   `an_opening_gives_exactly_where_the_vortex_or_the_guild_would`, `a_spell_book_is_sold_where_a_
   mage_guild_stands_for_the_part_s_price`; the plans' `check_town_open`, `check_town_book`,
   `check_guild_refusals`; `guild-book-s1`. **Holds.**
7. **The night's refill.** `a_night_raises_a_town_s_visitor_and_garrison_hero_to_their_maximum_
   silently`, `a_town_without_a_guild_refills_nothing_and_a_hero_outside_a_town_neither`,
   `a_night_refills_in_a_town_with_a_guild_ever_built`; `guild-siege-s1` (the refill past the
   day's regain); GAM-CAL-002 amended. **Holds.**
8. **The Mana Vortex.** `a_standing_mana_vortex_with_the_week_s_flag_gives_the_factor_times_the_
   maximum`, `the_visit_s_end_gives_the_vortex_before_the_learning_once_a_week`, `a_build_gives_
   the_vortex_after_its_learning_to_the_garrison_hero_first`, `a_hire_gives_the_vortex_to_the_
   hired_hero_alone_a_move_between_the_bars_nothing`; `the_vortex_over_two_weeks`
   (`guild-vortex-s1`: the flag set again on day 8 only, a second opening changing nothing);
   `check_guild_step` (the gifts counted per town and week). **Holds.**
9. **The views and the protocol.** `the_owner_sees_what_its_guild_shows_told_at_each_build_and_
   another_player_nothing`, `a_town_has_the_aurora_once_its_building_was_built`; `h3-map`'s
   `a_viewer_learns_nothing_it_cannot_see` (its garrison case the property's, fixed by T50);
   `h3-protocol`'s `every_command_maps_to_the_wire_and_back_and_round_trips` and `every_
   rejection_maps_back_and_round_trips` with `Open`, `BuySpellBook` and the three refusals, the
   town view's `guild`, `aurora` and `vortex` (protocol v78 to v80); the fold gate over the 118
   goldens without a new gap (`h3-protocol`'s; CI's full verify on `master` the record).
   **Holds.**
10. **The env and the replays.** Action encoding v18 and v19 (`TOWN_OPEN` 103 and `SPELL_BOOK`
    104: `h3-sim`'s layout test, the mask property's listed openings and purchases,
    `the_env_plays_the_town_map_to_its_end_of_days` buying books on seed 264); replay format v24
    and v25 with the words `town open` (`guild-vortex-s1`, three) and `town book`
    (`guild-book-s1`). **Holds.**
11. **The census.** `h3-map`'s `original_data` (T50: the 160 maps' guilds drawn without an
    excluded spell, 25 of the survey's counts, the main towns' heroes learning at the start),
    run again by the original-data step here. **Holds.**
12. **Rules, quirks, stubs, gates, batch TWN-5.** TWN-GLD-001 to -009, TWN-SET-012, TWN-VEW-004
    and GAM-CAL-002 amended, each with its id, sources and tests (`check-rules` passes);
    `TWN-QUIRK-14` in `quirks-town.md` with its flag `quirk.town.aurora_ignores_map_bans` and its
    regression tests in both settings; `TWN-STUB-6` and `-15` gone from `docs/stubs.toml`. The
    gates on `master`'s code: coverage (132 gated files in the four logic crates, no exclusion); determinism (debug,
    release and native, the 118 golden adventure replays among them); the census of
    `original_data` (all 160 maps import, build and play to day 29; the guilds as criterion 11
    says, 26 775 slots drawn on 158 maps, 249 guilds standing, the 33 main towns' heroes with a
    book learning 188 spells); the properties at their default cases in the test and coverage
    steps (T50's three at 240 cases, its record); the benchmarks (all 41 workloads inside their
    gates, from −0.92 % `random-play-sea-36` to +1.69 % `fork-economy-xl-144`;
    `day-start-guild-36` baselined by T50). Batch TWN-5 run and scored (T44, survey
    `E09-guild.md` §21: every row as the reading, in both runs). **The gap fixed** (below).
    **Holds.**

**The gap.** Criterion 12 wants batch TWN-5's consensus rows `verified`: of the rules they settle
only TWN-GLD-001 was (T46). Each row matches its rule as built (the checkpoint chose the
original's reading every time, decisions 167 to 170), so TWN-SET-012 (TWN-5-1), TWN-GLD-002
(TWN-5-1), -003 (TWN-5-3), -004 (TWN-5-2, -3), -005 (TWN-4-2, TWN-5-3, -4, -5), -006 (TWN-5-5,
-3), -008 and -009 (TWN-5-5) are now `verified`, each observation with a note naming what no row
played. TWN-GLD-007 stays `implemented` (no row bought a book), TWN-VEW-004 too (no row read a
view). `TWN-QUIRK-14`'s context still called its learning "planned": built by T48a. **Left open,
by design:** `TWN-STUB-2`, `-16` (E10), `-8` (the battle lane's inbox), `-13` (Visions' reach),
`-17` (now increment 8), `-18` (E13).

**Increment 5 closed** (2026-11-24): its section (the refinement's note, decisions 161 to 195 and
the task list with T51's line) moved verbatim to `E09-archive.md` (checked by script: every line
of the old epic in the new epic or the archive, but the seven edited ones); its design note's
Part G stays in `E09-design.md` (decision 40), its exit criteria in §99. The README's row (5 of
8) in `shared/e09-t51-readme`.

**Increment 6 refined** (`TWN-Q34`; decisions 196 to 199): the town's trades (decision 197; the
epic's sketch now eight increments, with the line for older citations); the survey with its
checkpoint, then the drafts with Part H and the cut (decision 198). The inbox's four notes that
increment 5 acted on (E08 T140's header, T132b's import version, T131's ADR parts, E04a T3a's
town chances) archived with where their work went (`QUEUE-ARCHIVE.md`); the market's notes and
the combination artifacts' costs kept for increment 6, the Lighthouse's, the Necromancy's and
the Cover of Darkness's now labelled increment 7, the Grail buildings' 8. `TWN-STUB-17`'s and
TWN-BLD-002's, TWN-GLD-005's and -009's "increment 7" now 8; `TRAPS.md`'s hygiene trap names
increment 6's trades. The adventure lane's and the AI lane's inboxes told the new numbers. The
queue holds T52 and T53.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` on the branch's first
state (the engine's code `master`'s at 25397e7, the eight rules already `verified`) with no
failing step (coverage 22.2 minutes, determinism 9.1, bench-check 15.8, original-data 1.0;
`deny` and `ffi-smoke` skipped by the scope). After the docs, on the branch: `check-docs`,
`check-rules`, `check-stubs`, `doc-drift --base origin/master` and `check-lane --base
origin/master`; on `shared/e09-t51-readme` `check-lane` and `check-docs`. CI on the pull
requests is the record.

**The queue:** T51 out; T52 next, T53 queued. Ids: decisions next 200, questions `TWN-Q35`.

### T52 — increment 6's survey and its checkpoint TWN-Q35 (2026-11-24)

One session, the user present at the start and at the checkpoint; branch `town/e09-t52`, docs
only. Starting point: `master` at 7782735 (#1149 and #1148 merged; the engine's code 25397e7's,
CI green there). The user's computer was connected: five files staged (`Heroes3.exe`,
`H3bitmap.lod`, `H3ab_bmp.lod`, `maps.tar`, `extracted.tar`), SHA-256 checked against the PC's.
The setup's full baseline on `master`'s code passed every step (coverage 57 minutes beside the
sub-agents, determinism 12, bench-check 20). **At its start** (the user, `AskUserQuestion`): T52
whole, the narrow jobs to sub-agents (decision 200). Seven ran, at most two at a time and only a
short one beside a reading: the market window at a town, the Artifact Merchants' stock, the
Blacksmith and the Shipyard (with a follow-up on the ship 20), the University and the
Transformer, VCMI 1.7.5 (with a follow-up), the wiki, the census (with a follow-up); each wrote
its sections and map entries to a notes file outside the repository, the session merged them.

**Read** (survey `docs/rules/surveys/E09-trades.md` §2 to §7; `docs/exe/map.toml`):
`town_screen_proc` 0x5D3240's cases 6, 14 to 17 and 21 to 23 with their faction sub-switches; the
market window's town entries, its screens' creates, redraws and procedures for screens 1, 3 and 4
(screen 3 whole), the buttons' tests; the stock's draw in `new_game_heroes` and `new_month`,
`random_artifact_of_level` and `artifacts_given_init` whole, the bans, `artifact_table_load`'s
combination pass; `town_blacksmith_buy` and the Blacksmith's dialog, the Ballista Yard's case;
the Shipyard's case, `create_boat` again, the ship's writers and the town screen's pieces and hit
map (the masks measured in the staged `H3bitmap.lod`); `town_university_open`, the University's
slots and learn again; `town_skeleton_transformer_open` and the Transformer's dialog with its
target table. 46 new map entries, 35 changed (one renamed: 0x5D2DA0 `town_university_open`);
`structs.toml`: the game's stock, given marks and bans, the artifact record's class, combination
fields, disabled and spell bytes.

**Found** (§0): the Marketplace for a human at an own town with or without a visitor, the
merchants and the guild for the visitor only and without an owner test; the count the current
player's Marketplaces, 0 at an ally's town without one, where artifacts are free (T-19); the
screens a town reaches by faction; the transfer to any undefeated player; the sale's positions,
Misc 5 never offered (T-22), the backpack's scroll pricing one place and selling another (T-20);
the stock one for the game, drawn last at the new game and whole each month, by the random
artifacts' picker with its marks; the combinations priced at the sum of their parts; the
Blacksmith's machine by faction at its creature's gold cost to the visitor, the Ballista Yard a
Castle's Blacksmith; the boat spot fixed at the new game, the occupied spot never tested (T-21, a
few uncovered pixels in a Fortress and a Conflux), the ship refreshed only with the screen (T-27);
the University's fixed four, a banned school taught (T-23); the Transformer's whole stacks, the
dragons and hydras to Bone Dragons, the rest but Skeletons to Skeletons, the stacks lost at a
timed game's expiry (T-26); the picker's endless loop when everything is banned (T-24). VCMI 1.7.5
agrees on most; it differs on D-70's rounding, the hero (TD-28), the count at an ally's town
(TD-29), the University's bans (TD-30) and the boat spot (TD-31). The wiki: five of eight pages
(three unreachable); its rate table agrees with the code's D-70 rows. Batch TWN-6: eight rows
(§11).

**The census** (§1, a throwaway example over `h3_map::read` and `h3_map::import`, not committed):
1 468 towns (592 fixed, no fixed Conflux, 876 random); the trades' buildings at the start; every
merchants' or guild's town with its Marketplace; at most 2 Marketplaces a player; 18 + 5 + 27 boat
spots, none occupied; 51 maps with teams; 2 maps banning the four schools.

**The checkpoint** (`TWN-Q35`, survey §13; `AskUserQuestion`, a round of four, one of three, then
the agent's calls one by one at the user's asking): D-70 the original (decision 201), TD-28
`fix-optional` as the new T-28 (202, the user's: the rule arbitrary), TD-29 the trader's count at
least 1 (203, the user's), TD-30 with T-23 `fix-optional` (204), T-22 fixed (205, the user's),
TD-31 the original (206), the agent's calls each accepted (207).

**Verify:** the engine's code is `master`'s. On the branch: `scripts/exe.py check --fix-sizes`,
`check` and `cited` (every cited address covered); a fresh sub-agent's second look at the
deciding paths of seven claims (the visitor-only entries, the University's and the
Transformer's fallbacks, the count and the free price, the University's bans, the sale's
positions, the spot written once, the month's seven): each confirmed. After merging `master`
(d54bdd8): `cargo xtask verify --scope town --summary --with-original-data` with no failing step
(coverage 19.6 minutes, determinism 7.0, bench-check 11.9, original-data 0.8; `deny` and
`ffi-smoke` skipped by the scope). CI on the pull request is the record.

**The queue:** T52 out; T53 next (the drafts, Part H from §101, the cut). Ids: decisions next 208;
disputes `TD-32`; quirk candidates `T-29`; questions `TWN-Q36`.

### T53 — increment 6's drafts, the design note's Part H, the cut, the checkpoint (2026-11-25)

Decision 198's second task, kept whole at its start (the user, `TWN-Q36`'s first round, decision
208: "no cut, use a sub-agent where it makes sense"; the agent had proposed T53a, the market and
the stock, then T53b, the other buildings, the views, the cut and the checkpoint, as T33a and
T33b). Branch `town/e09-t53`, docs only (the three drafts, the design note, the epic, the log, the
answered questions, the lane's queue and HANDOFF, four lanes' inboxes). Starting point: `master` at
4f2cc71 (#1161 merged; CI green on `master`'s run), `master`'s e2e15c7 (#1159, E14 T2a) merged in
before the first push. No original data staged: the survey suffices.

**Sub-agents** (decision 208), one at a time: (1) a report of the E08 paths the trades reuse (the
market's decision, checks, offers and view; its env entries, wire and replay words; the map's
University; the War Machine Factory through the hire dialog; `BuildBoat` and the town's boat spot;
`ArtifactPool`'s lifetime and picker; `HeroChange`; the state's hash order; `Recruit`'s opening of
the hire dialog), which framed the ADR-level round; (2) the three drafts written from Part H on
increment 5's drafts as the model; its four gaps folded in (a Black Market's stock is static, only
its sold entries state; a war machine is never in the backpack; `TownArmy` is `h3-towns`';
ADR-0016 point 1's "increment 5's town boat" is increment 6's).

**The ADR-level choices** (`TWN-Q36`'s second round, four; decisions 209–212, each as
recommended): the market window as E08's market decision opened by `TownCommand::Market` with the
town as its place; the University as E08's University decision opened by
`TownCommand::University`; the Blacksmith and the Ballista Yard as the one-shot
`TownCommand::BuyWarMachine`; the stock and the artifact pool's marks in the adventure state.

**Written:**
- **The drafts** (`docs/lanes/town/adr-drafts/`): `0028-increment-6.md` (point 2 nothing new in a
  town's record and the rules: the count, the window's offer, the machine, the boat, the University,
  the Transformer's target; point 5 the six building fields, the parameters, the core's combination
  costs; point 6 the five commands and their refusals; point 9 the versions), `0017-increment-6.md`
  (points 1, 3, 4, 6, 8, 9, 10: the commands and `SellArtifact`, the month's draw in the day's
  order, the streams, the decisions with a town as their place and the listing, the env's 42
  entries, the versions, the gains and costs), `0016-increment-6.md` (points 1, 4, 6: the stock
  and the marks in the state, a town as the market's and the University's place with the checks
  on a resolved window, the sale's rate, the pool kept, the town's boat; the stock seen only in its
  window).
- **The design note** Part H, §101 to §120: scope and choices (§101, sixteen agent's calls), the
  part v10 (§102), the count (§103), the window (§104), the trades, the transfer and the guild
  (§105), the merchants' purchase (§106), the sale (§107), the stock (§108), the combinations'
  costs (§109), the Blacksmith (§110), the boat (§111), the University (§112), the Transformer
  (§113), the views and events (§114), the env, the protocol and the replays (§115), ids, versions,
  quirks and stubs (§116), the gates and batch TWN-6 (§117), the building tasks T54 to T61 (§118),
  the exit criteria (§119), the risks (§120).
- **The checkpoint** `TWN-Q37` (one round of four; decisions 214–217, each as recommended): the
  month's draw from the towns' stream; the combinations' costs in the core's compile; the env's 42
  entries; the cut and the exit criteria as written. The agent's calls of 213.
- **New ids** (planned): rules TWN-TRD-001, TWN-MKT-001 to -005, TWN-STK-001, -002, TWN-SMT-001,
  TWN-SHY-001, TWN-UNI-001, TWN-TRF-001, TWN-VEW-005, the core's HER-ART-008; quirks
  `TWN-QUIRK-15` (T-28) and `-16` (T-23) with their flags, `-17` to `-23` the fixes (T-19, T-20,
  T-21, T-22, T-24, T-25, T-26); `TWN-STUB-21` (an ally's town, T58a); no new dispute or quirk
  candidate.

**The queue:** T53 out; T54 next, T55 to T61 queued. The adventure lane's inbox told the planned
`cross-lane` changes in its paths and the merge order, the AI lane's the env's 42 entries, the
server lane's the protocol's additions, the battle lane's the harvest's fields; the E08 market
notes of the lane's inbox (T127a1 to T133b), folded into Part H, archived. No version moved. Ids:
questions next `TWN-Q38`; decisions next 218; disputes `TD-32`; quirk candidates `T-29`.

**Verify:** docs only. The setup's baseline was lost to a VM restart mid-run (CI's green run on
`master` e2e15c7 stands for it). On the branch (d225af0): `check-docs`, `check-rules`,
`check-stubs`, `check-exe-map`, `check-lane --base origin/master` and `doc-drift --base
origin/master` ok; `cargo xtask verify --scope town --summary` with no failing step (coverage 24.9
minutes, determinism 10.0, bench-check 16.8; `deny` and `ffi-smoke` skipped by the scope; no
original data staged). CI on the pull request is the record.

### T54 — batch TWN-6: the specs, the harness's reads, the runs, the scoring (2026-11-25)

Decision 198's third round (§118 round 2), kept whole at its start (the user, decision 218: "no
cut, use subagents efficiently as needed"; the agent had proposed T54a the Marketplace and the
merchants, rows 1 to 5 and 8, then T54b the Shipyard and the University, rows 6 and 7); `cross-lane`
consented (`h3-experiment` and `docs/experiments/README.md`, the battle lane's; the harness is
common). Branch `town/e09-t54`. Starting point: `master` at 6498273 (#1168 merged; CI green on
`master`'s run there); the setup's full baseline was lost to a VM restart mid-run, CI's run stands
for it. The user's computer was offline for the first hour (the specs, the recipes and the
harness code written meanwhile), then connected: 17 files of `original-data/` staged (the four
LODs, the executable, the five DLLs, both `.snd`, `H3ab_ahd.vid`, `battle-videos.vid`, `maps.tar`,
`extracted.tar`), the sizes checked.

- **The spec** (`h3-experiment`, `cross-lane`): a Conflux's `artifact merchants` (bit 10) and
  `magic university` (bit 18), the AB pack's slots (VCMI's table has none); test
  `a_conflux_town_may_have_its_merchants_and_university`; the format in
  `docs/experiments/README.md`.
- **The specs** `E09-batch-6a.toml` (stamp `1655a41f`: rows 1, 3, 4, 5; Red's eight towns, Blue's
  Tower), `-6b.toml` (`011092b2`: row 2; Red and Blue one team, Tan the computer's), `-6c.toml`
  (`182c5f36`: row 8; 18 of the 21 minors placed, day 1 to day 29), `-6d.toml` (`d22528b3`: rows 6
  and 7; Fire Magic banned, a Fortress's coast); their templates and recipes
  (`clients/capture/batches/E09-batch-6a.toml` to `-6d.toml`).
- **The harness** (common; `clients/capture/README.md`, "The town's trades"): the town screen's
  `building = <id>` with `opens` (`hire`, `market`, `message`, `boat`, `university`, `nothing`;
  clicked where the status line names the building) and `pixel = [x, y]`; the trade window's
  screen 3 (vtable 0x643AAC, read in `market_sell_artifact_create`; `sell`, `position:<n>`, the
  arrows 130 and 131, 131 moving the offset +1; `market:places`, `market:offset`); `stock` and
  `stock:classes` (game +0x1F664); `town:x,y:spot`; the University's dialog (0x643BD8: the slots'
  pictures 104 + 3k, the purchase box 117 to 125 in the same dialog, kind `university_buy`;
  `university`, `university:buy`); the town's boat paid after the ship's build shows (awaited).
  The live work, the University's items and the runs by one sub-agent (decision 218); the harness
  check and its unit tests pass.
- **Run** twice each, the filled templates in `experiments/results/` (`E09-batch-6a.md` to `-6d.md`,
  `-run2.md`). **Scored** (survey `E09-trades.md` §14): every row as the code reading in both
  runs: D-70 (37, 62, 112), TD-28 (GENRLTXT 274 at the four buildings), TD-29 (count 0, free),
  TD-30 (Fire Magic taught), TD-31 (a second boat on the one spot), §3.4 (600), §3.1 and §3.2 (the
  unplaced minors, all seven redrawn), D-32 the code's; **T-19, T-20, T-21, T-23 and T-28
  confirmed**, so T55b may add `TWN-QUIRK-15`'s and `-16`'s flags. Two slips of the agent's own
  predictions, not of the game: TWN-6-8's purchase at 2.5 times the cost (the five Marketplaces'
  factor; count 1 gives the observed 5 times) and its minors' order (a draw).
- **Address map:** `market_sell_artifact_vtable` 0x643AAC. No version moved; no rule's status
  changed (increment 6's rules are written from T55 on). Ids: decisions next 219; experiment batch
  next TWN-7.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` on the branch (with
`master`'s 92fb75c merged in): every step passes but `check-lane` (by design for a `cross-lane`
branch; `cargo xtask check-lane --cross-lane --base origin/master` ok): coverage 26.2 minutes
(the first run, before a `rustfmt` of the new test's assertion; the second run with `--skip
coverage`), test 17.1, determinism 10.4, bench-check 19.6, original-data 1.1; `deny` and
`ffi-smoke` skipped by the scope. The harness's own check and its unit tests pass. CI on the pull
request is the record.

### T55 — `shared/`: increment 6's drafts into the ADRs, the glossary, the security lines, the flags of `TWN-QUIRK-15` and `-16`, the combinations' costs (2026-11-25)

One session, the user present at the start; branch `shared/e09-t55`, `cross-lane` (the ADRs, the
glossary, `docs/06-security.md`, `h3-content` and `h3-pack` shared; the harvester and
`packs/base` the battle lane's, `heroes.toml` and the golden adventure replays the adventure
lane's, the bot reports the AI lane's, one test literal the server lane's). Starting point:
`master` at 8abf1f4 (#1173, T54's, merged); the setup's full baseline passed on its code
(`verify-once: verified (sandbox) code a3676e3…`), CI green on it. The user's computer was
connected: 5 files staged at the end for the original-data step, SHA-256 checked. **Kept
whole** with the `cross-lane` consent (decision 219; the recommendation was T55a alone); edits
made in worktrees while the baseline ran, then moved over.

- **T55a, the ADRs** (the three drafts of T53, accepted at `TWN-Q37`, deleted from
  `adr-drafts/`; applied by a sub-agent): ADR-0028 points 2 (the count, the window's offer and
  hero, the machine, the boat, the University, the Transformer's target), 5 (the six building
  fields, the parameters, the core's combination costs), 6 (`Market`, `University`,
  `BuyWarMachine`, `BuildBoat`, `Transform`), 9 (the versions), the header's amendments line,
  **the Consequences and Alternatives for increment 6** (the drafts had none; written from the
  drafts' facts); ADR-0017 points 1, 3, 4, 6, 8, 9, 10; ADR-0016 points 1, 4, 6; each revision
  history, the ADR index's three rows. Each block cites batch TWN-6's rows that confirmed it
  (survey `E09-trades.md` §14: every row as read).
- **T55a, the docs:** `docs/03-domain.md`'s terms (the count, a town's window, the window's hero,
  the merchants' stock, the pool's marks, the artifact sale, the town's boat, the Transformer's
  target); `docs/06-security.md`'s section "The town's trades (E09 increment 6)" (design §114: the
  stock seen only in its window, its draws and the pool's marks server-only, the market and
  University views their owner's, the five commands' checks, nothing priced from the payload, the
  events) and the "Another player's hero" line's garrison hero (T50's record).
- **T55b, `h3-content`:** `quirk.town.visitor_only_trades` (`QuirkFlag(54)`,
  `VISITOR_ONLY_TRADES`, T-28; read from T58a) and `quirk.town.university_ignores_map_bans`
  (`QuirkFlag(55)`, `UNIVERSITY_IGNORES_MAP_BANS`, T-23; read from T59b; 52 and 53 taken by E14 T2c, #1174, merged meanwhile): the registry's 56
  positions, its two tests. **HER-ART-008** (`heroes.toml`, `verified`: the code reading of
  `artifact_table_load` and TWN-6-4): a combination whose pack names no cost costs its parts'
  costs summed per resource, in resource order, derived in the artifact compile after the pack's
  artifacts (a sum past `u32` refused); `her_art_008_a_combination_without_a_cost_costs_its_parts_sum`
  (the pack's cost kept, the sum, the overflow). No encoding changed: `RULESET_HASH_VERSION` 53
  and the pack format kept. **`h3-pack`:** `the_harvested_flags_of_e09_increment_6` (on in
  `faithful`, off in `strict-fixed`, every expansion) and
  `her_art_008_the_sod_combinations_cost_their_parts_sum` (the twelve: 84 000 down to 3 000, the
  survey's §3.4 table). **The harvester's** two profiles and `packs/base/core.toml`'s two profile
  lines (the synthetic packs name no town flag; `synthetic-ext`'s Storm Regalia now costs its
  parts' 9 500 coin).
- **The ledger:** `TWN-QUIRK-15` and `-16` entered (`quirks-town.md`; confirmed by TWN-6-5 and
  TWN-6-7; the rules' regression tests with T58a and T59b, the profiles' now).
- **Replays and records** (a sub-agent, then the session after the merge): the base faithful
  ruleset hash moved (on `master`'s 529bb61, #1174 merged in) 0x9078e1983efcc013 to
  **0x3f1f68842dab4e0c**, `strict-fixed`'s 0xc67d550d834033d8 to **0x0bd901a144728cc7** (the
  combinations' costs; the synthetic's kept). The 118 golden adventure replays re-recorded from
  their commands by a scratch script (`h3 adventure run` in release, 10 s), once before the merge
  and again after it (the merge took `master`'s files): only `ruleset_hash` moved; all 118 verified (`h3 adventure verify`), the ten battle replays unchanged and verified
  (`h3 verify`). `h3-server`'s record literal; the AI lane's `bots-report-v1.json` and
  `bots-neutral-v1.json` regenerated (3.7 minutes): only `engine.core` moved.
- **The merge:** E14 T2c (#1174, `shared/`) merged into `master` during the session with the rmg
  lane's flags at registry positions 52 and 53 (and the registry at 128 bits): the town's two moved
  to 54 and 55; the conflicts in `quirks.rs`, `lib.rs`, the harvester's profiles and
  `packs/base/core.toml` resolved by keeping both lanes' flags, `master`'s first.
- **Also:** the battle lane's HANDOFF version line (quirk flags 55, the hash); the adventure lane's
  inbox (the replays' header, HER-ART-008); the lane's inbox note on CI's jobs done (`TRAPS.md`:
  the `bench` job's 60 minutes).
- *Agent's calls:* the two ledger entries made with the flags, as T45's; the sum in resource order
  with a typed refusal past `u32` (no shipped sum comes near it); HER-ART-008's status
  `verified` (a code reading and an observation).

**The queue:** T55 out; T56 next (the town part v10). Ids: decisions next 220; quirks next
`TWN-QUIRK-17`.

**Verify:** `cargo xtask verify --scope town --summary` on the branch (with `master`'s 8fa2711
merged in): every step passes but `check-lane` (by design for a `cross-lane` branch; `cargo xtask
check-lane --cross-lane --base origin/master` ok) and clippy (`type_complexity` on the compile's
new tuple, fixed by inference; then `--only clippy` and `cargo clippy --workspace --all-targets --
-D warnings` pass): coverage 24.6 minutes, determinism 7.4, bench-check 15.3; `deny` and
`ffi-smoke` skipped by the scope; `--only original-data` with the staged data 2.5 (the harvest
matches the hand-edited profile lines). `cargo check --workspace --all-targets`; `cargo test -p
h3-content -p h3-pack -p h3-harvest -p h3-server`; `h3 adventure verify` and `h3 verify` on every
golden replay. CI on the pull request (#1181) is the record: green at the second attempt, the
first's coverage job failing on `h3-server`'s known clock test under load
(`a_spectator_watches_a_timed_game_through_a_restart_to_a_stalled_battles_void`, `clock.rs:801`;
the server lane's inbox has it), its failed job re-run.

### T56 — the town part v10: the trade roles, the war machines, the trades' parameters, the Transformer's table (2026-11-26)

One session, the user present at the start; branch `town/e09-t56`, `cross-lane` (the battle
lane's `h3-formats`, `h3-harvest`, `packs/README.md` and `docs/formats/installation.md`, the
adventure lane's golden replays and `h3-game`'s `tests/mod.rs`). Starting point: `master` at
b1e7a44 (#1182; T55's #1181 merged before it); the setup's light gates on `master`'s code (CI's
fingerprint), CI green on `master`. Five files staged (both bitmap LODs, the executable,
`maps.tar`, `extracted.tar`). **Kept whole** with the `cross-lane` consent (decision 220).

- **`h3-towns`** (design §102): `BuildingRoles::marketplace`, `artifact_merchants`,
  `freelancers`, `university`, `transformer` (each at most one building of a record);
  `BuildingDef::war_machine` (a key) compiled to `Building::war_machine` (an `ArtifactTypeId` of
  the core with a `war_machine` creature); `TownParamsDef`/`TownParams::merchants_stock` (at most
  `MAX_MERCHANTS_STOCK` 8, none special), `boat_price`, `boat_type`, `university_skills` (at least
  one, each once), `university_price`, `transformer_into`, `transformer_dragons_into`;
  `TownPack::transformer_dragons` into `TownParams::transformer_dragons` (handle order, each named
  once, `MissingParams` without the parameters). New refusals `TownCompileError::UnknownArtifact`
  and `UnknownSkill`. **Town ruleset part v10** (`TOWN_RULESET_HASH_VERSION`: the seven parameters
  and the dragons after the Vortex's factor, the five roles after the Mana Vortex, a building's war
  machine after its roles); pins `0x08b1_fe5c_b1f7_09a6` and the empty part's
  `0xa82b_d7e1_f29d_c67d`. TWN-TRD-001 `verified` (its catalog entry new: the code reading, batch
  TWN-6's rows 5 to 8, VCMI and the wiki agreeing).
- **`h3-pack-towns`:** the five role fields, `war_machine`, the seven `[town_params]` fields and
  `[town_transformer] dragons`; the format's doc. The synthetic pack: a stock of four, a boat of
  800 coin and 5 shards (type 2), three University skills, the Transformer's Shade and Wisp, two
  dragons (the expansion's Lurker a third), six new buildings (one of each role, the Blacksmith and
  the Transformer selling `bolt_thrower_kit`).
- **The harvest** (`cross-lane`): `h3-formats`' `TownTables`/`TownData::blacksmith_machines`
  (`blacksmith_machine_creatures` 0x642E90) and `transformer_targets`
  (`skeleton_transformer_targets` 0x6412FC, `TRANSFORMER_CREATURES` 145), installation.md §7.61's
  rows; `h3-harvest`'s gap-fill: the five roles (source `exe-town-trades`) and `[towns.trades]`
  (the Blacksmith's id, the Ballista Yard `machine_of` the Castle, the stock, the boat, the
  University); the town harvest: each Blacksmith's machine as its table creature's artifact of
  `[[artifacts.war_machines]]`, the Transformer's two targets (its table's one fixed point, the
  Skeleton, and the other, the Bone Dragon) and each pack's dragons (base six, ab four). Re-run on
  the staged installation: only the three `towns.toml` changed (the header's sources line in
  `sod`'s).
- **Tests:** `h3-towns`' compile, refusal, message and hash tests (the test core's war machine, its
  ring and two skills new); `h3-pack-towns`' `the_trades_parse`, the parser's strictness and keys,
  `the_original_trades` (the roles on their buildings alone, the nine Blacksmiths' machines and the
  Ballista Yard's, the parameters, the targets, each edition's dragons); `h3-formats`' reader and
  refusals; `h3-harvest`'s town test (the roles, the war machines, the parameters, the dragons by
  pack; the errors).
- **Replays:** the 118 golden adventure replays' `town_ruleset_hash` 0xcf8db1aa2bb768d2 to
  **0xc830faa47b612bfc** (the same in both profiles; nothing else records the town part: the bot
  reports, the server's literal unchanged).
- *Agent's calls* (decision 220): `[town_transformer]` over design §102's `[[transformer_dragon]]`;
  `war_machine` beside the roles; the University at least one skill; the boat's type unbounded;
  the Blacksmith's machine from 0x642E90 through the war machine artifacts. Design §102's *As
  built*.

**The queue:** T56 out; T57 next (the stock, `cross-lane`). Ids: decisions next 221.

**Verify:** `cargo xtask verify --scope town --summary --with-original-data` on the branch (with
`master`'s abf981f merged in): every step passes but `check-lane` (by design; `cargo xtask
check-lane --cross-lane --base origin/master` ok) and coverage, whose test run hit a random case
of `h3-map`'s `tests::view::views_move_as_the_events_say` that fails the same way on `master`
abf981f (the adventure lane's inbox, its `cc` line not committed); `verify-step coverage --scope
town` again: the tests passed, two regions of the compile's trade parameters uncovered (a price's
unknown resource), their tests added, `h3-towns`' ruleset files at 100 % (`cargo llvm-cov -p
h3-towns`). Determinism 11.6 minutes, bench-check 17.6 (inside its gate), original-data 2.5 (the
harvest matches the packs). `cargo check --workspace --all-targets`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --release -p h3-cli -p h3-sim -p h3-protocol -p
h3-server -p h3-game -p h3-map -p h3-towns -p h3-pack-towns -p h3-files` (every golden replay
replayed), `cargo test -p h3-harvest -p h3-formats`. CI on the pull request is the record.

### T57 — the Artifact Merchants' stock: the pool's marks kept, the new game's and the month's draws (2026-11-26)

One session, the user present; branch `town/e09-t57`, `cross-lane` (the adventure lane's
`h3-adventure` `state.rs`, `state/draws.rs`, `state/tests/draws.rs`, `lib.rs`, `h3-game`'s
`state.rs` and `state/day.rs`, the golden replays, the catalog's ADV-OBJ-006 and GAM-CAL-002;
the server lane's `h3-server` record test). Starting point: `master` at 6035ed8 (#1190), **red**:
E09 T56's `BuildingDef::war_machine` (#1189) met E31 T2d's new literal without it in the AI
lane's `h3-game/src/computer/start/tests.rs` (#1190), so CI's clippy and coverage failed to
compile `h3-game`'s tests. Fixed first in #1193 (`town/e09-war-machine-literal`, one line,
`cross-lane`, the AI lane's inbox told; decision 222); T57's branch stacks on it. The user's
computer went offline at the start, so no original data were staged (the scoped verify without
`--with-original-data`); the setup's baseline died with a VM restart, CI's run on #1193 served.
**Kept whole** with the `cross-lane` consent (decision 221).

- **`h3-adventure`** (design §108): `state::ArtifactMarks` (`state/pool.rs`: the bans and the given marks, a
  `bool` per artifact; its hash packs them 64 a word), held by `ArtifactPool` (`resume`,
  `into_marks`); `resolve_rewards` returns the marks, `StaticMap::new` gives them beside the map.
  The town lane's `town/merchants.rs`: `Merchants` (the marks and the stock, a
  `Vec<Option<ArtifactTypeId>>`, empty until the game's first draw) as `AdventureState::merchants`
  (an `Arc`, the state's last field; a fork shares it), `merchants_stock()`, `draw_merchants(core,
  classes, draws)` (each class in order by ADV-OBJ-006's picker on a copy of the marks, the block
  replaced; `TWN-QUIRK-21` an empty entry where nothing can be drawn), `hash_merchants`.
  **Adventure state hash v30** (the block last: the stock, then the marks).
- **ADV-OBJ-006's restart by class** (decision 223, found writing the stock's tests): the
  picker restarted every artifact's marks when one class ran out; now `ArtifactPool::restart`
  resets the drawn classes' records alone, and the widening to the four classes resets theirs when
  none of them is left (the original's `random_artifact_of_level` 0x4C94D0: the reset 0x4C950E
  covers the level's records, E09 T52's reading). The catalog entry amended (E09 T52's reading a
  source of its own); the adventure lane's test asserts the first three widened draws distinct.
- **`h3-game`:** `town/merchants.rs`' `at_setup` (from `GameState::new` after the Portals of
  Summoning: the setup stream's last draws) and `monthly` (from `advance_day` after a creature
  month's step, before the towns' day: the towns' stream, decision 214); the classes the town
  part's `merchants_stock`, none without its parameters. GAM-CAL-002's summary amended.
- **Rules:** TWN-STK-001 and TWN-STK-002 `verified` (the code reading, batch TWN-6's row TWN-6-8
  on day 1 and day 29, VCMI and the wiki on the shape and the timing; D-32's exclusions the code's).
  `TWN-QUIRK-21` (T-24, state-corrupting: an empty entry for the hang; no flag).
- **Tests:** `h3-adventure`'s `town/merchants/tests.rs` (the draws on the build's marks, every
  entry anew with the old ones marked, the widening and an empty entry, the hash and a fork:
  the same stock on other marks hashes otherwise); `h3-game`'s `town/tests/merchants.rs` (the new
  game's stock against a game without one: the towns, the heroes, the towns' stream and every
  earlier draw alike; the month's draws on days 29 and 57, the stock unchanged between, three
  months giving the three minors, the towns' stream one draw ahead after day 29). Two tests of
  the setup stream's and the game stream's order (`portal.rs`' Portals last, `growth.rs`' week
  leaving the game's stream) now run without a stock.
- **Replays:** the 118 golden adventure replays re-recorded from their commands (a scratch script
  over `h3 adventure run --script`): only `adventure_state_hash_version`, 1 182 checkpoint
  entries (in 19 lists) and the 118 final hashes moved; every replay replayed its commands
  (the three of 60 days too). `h3-server`'s `tests/record.rs` literals (`army-dialog-s1`'s
  checkpoint and final hash). The bot reports record the adventure part's hash only: unchanged.
- **Speed** (skill `homm3-bench-regression`, `master`'s worktree in the sandbox): the first
  build's scoped verify failed bench-check on 30 workloads, the reach and random-play ones +3 to
  +9 %: the same games (every callee's count equal) with `Reach::search` +10 % and
  `walking_step` +10 %, the codegen units moved by the pool's new code in `state::draws`. The
  pool, its marks and its picker moved to their own module `state/pool.rs`: `reach-xl-144` and
  `random-play-duel-36` flat against `master`. The month's draws first cost
  `month-start-builds-36` +7 % on `master` (each draw collecting a list over the 144 artifacts):
  the pick now counts the unmarked artifacts and finds the drawn one without a list, over the
  class's own list (`ArtifactMarks`' artifacts by class, derived at the build) when one class is
  drawn, and the block is drawn in place (`Arc::make_mut`): +2.1 % on `master` in the sandbox,
  +1.29 % on its baseline (inside the gate); `build-trades-xl-144` -1.6 % (the build's draws by
  class too). The forks pay one reference count more (`fork-economy-xl-144` +0.26 % on `master`).
- *Agent's calls* (change freely): the game state hash's version kept at 39 (its encoding
  unchanged; E08 T142a's adventure v29 the precedent; design §116's table had a bump at T57); the
  block never `None` (the marks exist in every game); the marks as `bool`s, packed only in the
  hash; the stock's length the last draw's classes'; the no-parameters case drawing nothing
  through an empty class list (no branch).

**The queue:** T57 out; T58 next (the market at a town). Ids: decisions next 224; quirks
`TWN-QUIRK-17` (planned `-17` to `-20`, `-22`, `-23`; `-21` entered).

**Verify:** `cargo xtask verify --scope town --summary` (no original data staged) on the branch
with `master`'s afb3250 (#1191) merged in: every step passes but `check-lane` (by design; `cargo
xtask check-lane --cross-lane --base origin/master` ok) and bench-check's two workloads
re-baselined after it (`build-banks-xl-144` -2.70 %: the pool's draws; `fork-economy-xl-144`
+0.26 % on `master`'s 390 800 055, which CI counts too); the first run's coverage gap (an empty
entry's hash) closed by a test. Coverage 23 minutes, determinism 8, bench-check 14. `cargo clippy
--workspace --all-targets -- -D warnings`; `cargo test --release -p h3-cli -p h3-sim -p
h3-protocol -p h3-server -p h3-files` (every golden replay replayed). **Owed:** the scoped verify
`--with-original-data` (the user's computer offline all session); CI on the pull request is the
record.
