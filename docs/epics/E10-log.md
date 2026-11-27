# E10 — the log of done tasks

The record of each E10 task, newest last (`docs/08-work-process.md`, "Keeping the start-of-session
reading small": grep the task's id, never read whole). The epic file keeps one line per task.

## Increment 1

### Increment 1, round 1: T1a, the teams and the game's end survey (2026-11-27, `adventure/e10-t1a`)

**The cut** (decision 8, the user at T1a's start after the reading was measured: the end about 4
to 5 KB still unread, the teams about 2 KB new, the gathering of about 60 team tests read by E08
and E09, the census): **whole**; not chosen: two parts with the end first (recommended), two parts
with the teams first. **Baseline:** `master` 003fca0 (#1210 merged; T145's #1207, #1209 merged);
the setup's light gates passed (`verify-once`: the sandbox's fingerprint). Staged the originals
from the PC at the start (17 files, SHA-256 against the PC's `staged-sums-2026-11-25.txt`: all
equal). Survey `docs/rules/surveys/E10-teams.md` §0 to §13:

- **The census** (§1, a throwaway program over `h3_map::read`, not committed): 51 maps with teams
  (43 "(Allies)" twins; 2v2 17, 2v2v2 9, …), 97 teams of two or more human colours, 23 teams of
  computers only; three maps start Red with a hero and no town; no colour starts with neither.
- **Elimination** (§2): the check's colour loop re-read; the count-down read at
  `next_player_turn` 0x4C7133 (the outgoing player's, at every turn's end, after that turn's
  check) and the days' box at `player_turn_start` 0x4CC8DA (ArrayTxt 128, 129); seven of the
  colour's own turns either way; GENRLTXT 7 and 8 need the colour current (A-55, A-56).
- **A defeated player** (§3, `player_defeated` 0x4F11A0 read whole): the flag, the Shipyards,
  mines, Lighthouses, dwellings and garrisons neutral, the heroes into the pool from the last, the
  tavern offers freed.
- **The end** (§4): the 37 calls by caller; the turn points (every turn's end; a human's turn
  start only; `new_day`'s check only with the resources victory met; not at the first turn's
  start, A-57); the verdict re-read (the viewing human's team; no human left: lost); the 36
  readers of `scenario_ended`.
- **A removal** (§5): none in the original; the network's drop (`network_player_drop` 0x556430
  read whole): the colour stays, the computer plays it.
- **Teams** (§6): the 85 readers of the team bytes sorted by what a team shares: the fog, the
  marks (the obelisks' too), the ownables, the meeting's give-only gifts (E08 T44a), the towns,
  Town Portal's team list (`hero_relocate_spell`'s three team tests, `town_gate_dialog_create`),
  the teleporters' exits, the views (`hero_popup`'s level 2 for the team); the engine's 22
  `allied` sites and 72 E10 marks (§7).
- **VCMI 1.7.5 and the wiki** (§8), the disputes D-80 to D-84 and the candidates A-55 to A-57
  (§9), batch GAM-1's six rows (§10; named GAM, E08's batch 10 owns the rows `E10-n`), the
  interfaces' sketch (§11), the ids `GAM-TEM` and `GAM-END` (§12), T1b's nine questions (§13).

`docs/exe/map.toml`: twelve entries new (`dialog_team_flags_a`, `step_team_town_gate`,
`days_without_town_countdown`, `days_left_box`, `network_player_drop`, `network_drop_recover`
inferred, `network_loss_message` inferred, `dialog_team_flags_b` inferred, `town_gate_dialog_create`
inferred, `victory_mines_test`, `victory_days_test`, `loss_days_test`), `scenario_condition_test_2`
read and renamed `victory_resources_test`, seven amended (`player_defeated` read whole,
`next_player_turn`, `player_turn_start`, `hero_relocate_spell`, `hero_popup`, `new_day`,
`scenario_condition_met`); `structs.toml`: the player's +0x3D. The epic file: decisions 1 and 6
point to E31 (the AI lane's inbox note of 2026-11-21). **Versions:** none (docs only). Next: T1b.

### Increment 1, round 1: T1b, the design note, the drafts, the cut and the checkpoint (2026-11-27, `adventure/e10-t1b`)

**The cut** (decision 9, the user at T1b's start): **whole**; not chosen: the checkpoint and the
note first, the drafts and the cut in a next session. **Baseline:** `master` 5994188 (#1214, T1a,
merged); docs only. Staged at the start while the PC was up: `Heroes3.exe` (SHA-256 `aac39e8c…`
checked), `maps.tar`, `extracted.tar`; the repo cloned from GitHub. One narrow reading with
`scripts/exe.py`: the readers of the defeated bytes (`grep '0x1f636\]'`, 60 sites), so the note's
§11 names the loops a defeated player leaves (the turn order, the income, the week's tavern
heroes, the Thieves' Guild, the market's recipients); and the check's colour loop around
`player_defeated` (no turn ended there: §11's own-turn defeat is an agent's call, batch row
GAM-1-7).

**The checkpoint** (ADV-Q65, three rounds before the note, one after; decisions 9 to 19): the
survey §13's nine questions. The build teams first (10); the original's moments (11, D-80); A-55,
A-56 accepted as the clients' boxes (12); A-57 accepted (13); no human left ends the game with the
team left named as the winner (14, D-84); a removed player's colour to the computer player, the
user against the recommendation, and the engine's half (`SystemCommand::HandOver`) now (15, D-83;
D-81 moot); give-only gifts (16, D-82); an ally's town built by this lane, `cross-lane`, the user
against the recommendation (17); batch GAM-1 before the last round (18). After the note: the cut,
the drafts and the agent's calls as written (19).

**The design note** `E10-design.md` (§1 to §21): **A** the teams (§1 `PlayerSetup::team`, map
import v30; §2 `AdventureState::allied` at the 22 sites and the 72 E10 marks swept; §3 one
explored layer per team, adventure state hash v31; §4 the shared ownables and marks; §5 the
allies' meeting with give-only gifts; §6 the team's views; §7 an ally's town: the entering, the
trades, `TWN-STUB-21` lifted, the siege of an ally's visitor read at T3c); **B** the end (§8 the
defeat, the days and the result, game state hash v41; §9 the check's three moments; §10 the
eliminations and the seven days; §11 a defeated player's steps and the loops it leaves; §12 the
verdict; §13 after the end; §14 the hand-over); **C** (§15 the events and the clients' boxes;
§16 the wire v86, the env, the server, the clients; §17 replays v28, maps, properties,
benchmarks; §18 versions, ids `GAM-TEM-001` to `-004` and `GAM-END-001` to `-008`, quirks
`ADV-QUIRK-59` to `-61`, stubs); **D** (§19 batch GAM-1 with rows 7 and 8 added; §20 the shared and
`cross-lane` work, the cut T2 to T7 and ten exit criteria; §21 risks).

**The drafts** `docs/lanes/adventure/adr-drafts/0016-e10-increment-1.md` (points 1, 4, 5, 6),
`0017-e10-increment-1.md` (points 1, 2, 3, 6, 8, 9) and `0028-e10-increment-1.md` (point 6, an
ally's town), accepted at the checkpoint for T2. ADR-0015 needs none.

**Docs:** decisions 9 to 19 and the next tasks in the epic; ADV-Q65 in `QUESTIONS-ANSWERED.md`;
T1b's entry to `QUEUE-ARCHIVE.md`, T2 and T3 queued; the inboxes of the town lane (decision 17),
the server lane (the end, the hand-over, v86), the AI lane (the terminal, the seats' `human`
marks), the client lane (E27 M5's facts, the boxes) and the map client lane (`MCL-STUB-4`).
**Verify:** docs only: `check-docs`, `check-lane`. No code, version or replay moved. **Next:** T2.

### Increment 1, round 2: T2, the `shared/` task (2026-11-27, `shared/e10-t2`, `cross-lane`)

**Baseline:** `master` be38c3e (#1216, E09 T58b, merged after T1b). Nothing staged (no folder of
the user's PC shared with the session; not needed). **The ADRs** (the three drafts accepted at
ADV-Q65, decision 19, applied and deleted with their folder): **ADR-0016** points 1 (the teams in
the static part), 4 (what allies share, the meeting, a defeated player's objects), 5 (one
explored layer a team), 6 (the team's views), an alternative, the history; **ADR-0017** points 1
(the setup's team, the defeat, the days, the result, the check's moments, `GameOver`), 2
(`HandOver`, the views), 3 (a defeated player out of the schedule, the count-down), 6 (the gifts,
an ally's dwelling read-only), 8 (the env's terminal), 9 (the versions), a consequence, an
alternative, the history; **ADR-0028** point 6 (an ally's town), the history. One change from the
drafts: E09 T58b took encoding v23, replay format v28 and protocol v86 after them, so point 8's
encoding stays v23 and point 9's next numbers are map import v30, adventure state hash v31, game
state hash v41, replay format v29, protocol v87. The index's status and Read-when cells.
**`docs/03-domain.md`:** team, ally, defeated, the seven days, the game's result, hand-over.
**`docs/06-security.md`:** increment 1's section (the teams fixed; an ally's things validated
against the trusted actor; the hand-over the host's alone; a player's days its own; a defeated
player never acts again; every command refused after the end; the events' audiences).
**Verify:** docs only: `check-docs` ok. Pull request #1218. **Next:** T3a.

### Increment 1, round 3: T3a, the teams' setup, `allied` by state and the fog per team (2026-11-27, `adventure/e10-t3a`, `cross-lane`)

**The cut** (the user at its start: as far as the session gets without the original data, with
sub-agents). **Baseline:** `master` be38c3e (the setup's light gates passing); T2's #1218 open.
Nothing staged (no folder of the user's PC shared; the shipped maps' import is covered by T1a's
census, not re-run).

- **The setup** (GAM-TEM-001, design note §1): `PlayerSetup::team` (h3-game), set by the import
  from `Teams::of_player[colour]`, else the player's index (map import v30; the experiment
  example's too); the adventure's `Seat::team`; the teams checked dense by the adventure's build
  (`SetupError::Team`, through `GameSetupError::Adventure`: *agent's call*, the check where the
  layers are made; the note's §1 amended "as built").
- **The state** (§2): `StaticMap::teams` by player (the entries past the players `NO_TEAM |
  index`, allied with nobody), set by the state's build, out of the content hash, written by the
  state hash after the player count (adventure state hash v31); `StaticMap::allied`, `team`;
  `AdventureState::allied` replaces the free `allied` at its 22 sites (the town lane's
  `town.rs` and `town/view.rs`, `cross-lane`, decision 17; `h3-game`'s garrisons, the computer's
  start and objects); the teleporters' exits leave a hero of the team out (ADV-TEL-003); the
  game's players hold their team (`GameState::team`; game state hash v41, so T4a takes v42).
- **The fog** (GAM-TEM-002, §3): one explored layer a team (`explored` by team; every reveal
  writes its player's team's, the viewers' tests walk `layers()`).
- **Measured at its first build** (a sub-agent, `homm3-bench-regression`, against `master` in a
  worktree): +2.3 to +6.7 % on all 19 reach and random-play workloads (the team table read at
  every tile test, not hoisted). Fixed by `passage::Side` (the mover's player and its team's
  layer, one 16-bit word in the mover tuple: no new argument, TRAPS line 38), set once a search
  (`Search::side`) or move (`check_steps`), read by `explored_by`; the passage's `visit` closure,
  out of line on the branch, became the always-inline `visit_entry`. Final: every workload within
  ±0.5 % of `master` but `reach-sea-xl-144` −2.97 % (the same calls, the sail path cheaper),
  re-baselined with its reason; `fork-boxes`/`fork-stones` +1.98 % against the committed
  baselines (`master` itself +1.75 %).
- **§4's rules re-tested with allies** (a sub-agent): ADV-VIS-006, ADV-BON-009, ADV-OWN-001,
  ADV-DWL-004, ADV-GAR-003, -004, ADV-SHY-003, ADV-MIN-002, ADV-BNK-003, ADV-TEL-003 in the
  adventure's unit tests, and Red playing Blue's dwelling as an ally's on batch E10's map through
  the game; the plans' oracles of ADV-SHY-003 and ADV-TEL-003 test the team. **Found:**
  ADV-VIS-006's knowledge (`Visits::known`: a Shrine's spell, a hut's skill, a find emptied, the
  Pyramid, a Fountain's and a Spring's visit) is still the visitor's alone, while the original's
  tile bits are the team's (survey §6; the Corpse and Lean To the player's own): telling an ally
  the content needs an event of its own, so it moves to **T3b** with the team's views
  (ADV-VIS-006's summary says so).
- **The sweep:** 32 marks naming E10 as the teams' place rewritten as built (17 in code, 15 in the
  catalog, the town lane's three `cross-lane`); the end's left for T4a, the meeting's and an
  ally's dwelling's for T3b, an ally's town's for T3c. `ADV-STUB-20`'s reason gains Town
  Portal's team list. GAM-TEM-001, -002 written; ADV-FOG-002, -003 amended.
- **Not yet (T3b):** a step toward an ally's hero is still a battle (GAM-BAT-004), an ally's
  dwelling's dialog still buys; T3b builds the meeting and the read-only dialog.
