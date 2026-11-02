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
