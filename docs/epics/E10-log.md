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
