# E10 teams and the game's end survey (E10 T1a)

Date: 2026-11-27. Scope: E10 increment 1 (E10 decisions 4, 5; the epic's increment 1 sketch),
input for T1b's ADR amendment drafts, the design note and the cut. **Whole** (E10 decision 8, the
user at T1a's start after the reading was measured: the end about 4 to 5 KB still unread, the
teams about 2 KB, with the gathering of the read team tests and the census). Input, not rules
(`README.md`): a fact becomes binding when a task writes it into the catalog with its sources and
tests.

**Sources** (retrieved 2026-11-27):

- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml` or `structs.toml`. The
  reading follows the human player's path of a single scenario or a hot-seat game; the network's
  branches are named, not followed. **Read whole:** `player_defeated` 0x4F11A0,
  `network_player_drop` 0x556430; the conditions' small tests `victory_resources_test` 0x5F1C80,
  `victory_mines_test` 0x5F2720, `victory_days_test` 0x5F2810, `loss_days_test` 0x5F2F20; the two
  dialogs' team flags `dialog_team_flags_a` 0x458010, `dialog_team_flags_b` 0x5698C0. **Read
  again** (whole before, by E27 M1 T1b3b): `scenario_condition_met` 0x4F2CE0.
  **Read in part:** `next_player_turn` 0x4C6FE0 (its start to the next colour, 0x4C6FE0 to
  0x4C7240; the network's re-check 0x4C7640 to 0x4C76DF; its end), `player_turn_start` 0x4CC7D0
  (the human branch's tests and the days' box, 0x4CC7D0 to 0x4CCAC0), `new_day` 0x4C7FE0 (its
  check, 0x4C8221), `adventure_manager_procedure` 0x4087B0 (the computer's turn, 0x408B9A to
  0x408C34), every reader of `scenario_ended` 0x6972B8 at its site, every call of
  `scenario_condition_met` by its caller, the network's dropped player (`network_player_left`
  0x47A500, `network_player_drop` 0x556430), and the team tests (§6).
- Earlier readings this survey gathers (not read again): `docs/epics/E27-design.md` §5 (the end's
  boxes, the screens, the score), `E24-design.md` §3 (the turn's boxes, recorded), the E08 and E09
  surveys' team tests (each named in §6 with its survey), `E31-computer-player.md` (the computer's
  reads of the teams).
- Text tables (`original-data/extracted/H3bitmap`): GENRLTXT 6 to 9, 96, 416, 417, 470, 528, 659
  to 661; ArrayTxt 128, 129 (indices from 0, as `docs/epics/E27-design.md`).
- The map client's scripted game `game-week` (`docs/epics/E24-design.md` §3, M4 T6: Blue, a hero
  and no town, banished on day 8, then Red's victory).
- VCMI 1.7.5 and the wiki: §8.

## 0. Findings in short

- **The census** (§1): 51 of the 160 maps define teams (43 are "(Allies)" twins), mostly 2v2 and
  2v2v2; 97 teams hold two or more human colours; three maps start Red with a hero and no town.
- **Elimination** (§2): at each check, a colour with no hero and no town is defeated; one with
  heroes and no town gets seven days (+0x3D: 7 set at the first check, lowered at each of its own
  turns' ends, defeated at the first check at 0); a town back resets it. GENRLTXT 7 and 8 need the
  colour to be the current player, so the boxes the original shows are fewer than the rule's
  (A-55, A-56).
- **A defeated player** (§3): its flag; its Shipyards, mines, Lighthouses, dwellings and garrisons
  neutral; its heroes into the pool from the last (one creature, their artifacts kept); its tavern
  offers freed.
- **The end** (§4): checked after every action that can change an owner or a count (37 calls) and
  at each turn's end and a human's turn start (not at the game's first turn); the standard victory
  is the viewing human's team with no enemy left; with no human left the game is lost; once ended
  everything stops at its next reader (no town screen after the capture).
- **A removal** (§5): no original rule; a dropped network seat stays, the computer plays it.
- **Teams** (§6): fixed by the map; they share the fog, the marks (visited, kinds, banks,
  obelisks), the ownables (not flagged, a garrison passed), the towns (entered, the screen, the
  spells; the owner builds), Town Portal's list, the popups' top level; an ally's hero is met with
  give-only gifts (a stack into an empty slot, an artifact into an empty place).
- **The engine** (§7): reads the teams and drops them; 22 `allied` sites (the player alone), no
  end. **VCMI** (§8) agrees but for D-80 to D-84. **Batch GAM-1** (§10): six rows. **T1b** (§13):
  nine questions.

## 1. The census (the 160 shipped maps, `h3_map::read`)

A throwaway program over `h3_map::read` (not committed, as E08 T137a's), run on
`original-data/install/Maps`: every map reads.

- **Teams:** 51 maps define them (`h3_map::Teams`), 109 do not (every colour its own team, E10
  decision 2). 43 of the 51 are the "(Allies)" twins of a map without teams; the other eight are
  Barbarian Breakout, Dawn of War, For Sale, Good to Go, The Five Rings, The Gauntlet, The Mandate
  of Heaven and Warmongers. The shapes over the playable colours: 2v2 17, 2v2v2 9, 2v2v1 4,
  2v2v2v2 4, 2v1 3, 2v2v2v1 3, 2v1v1 2, 5v1 2, and 2v1v1v1v1, 3v1v1, 3v1v1v1, 3v2, 3v2v2, 3v3v2,
  4v1 one each. No map puts every playable colour on one team; every map's `count` equals its
  teams in use; every team code is below the count.
- **Who is on a team:** 97 teams on 43 maps hold two or more colours a human may play (a hot seat's
  allies); 23 teams on 22 maps are computers only; 3 teams mix a computer-only colour with human
  ones.
- **The conditions on the 51:** the standard victory 42, capture a town 3 (+1 with the standard
  too), defeat a monster 2 (+1), build the Grail with the standard 1, defeat a hero 1; the
  standard loss 46, the time 4, a town 1. Over all 160: the standard victory alone 108; the standard loss
  141, the time 15, a town 2, a hero 2 (the special ones increment 2's).
- **A start without a town:** three maps give Red (human or computer) a hero and no town:
  Adventures of Jared Haret, Knee Deep in the Dead, Pirates! (§2's seven days from the first End
  Turn). No playable colour starts with neither a hero nor a town. 28 colours start with towns
  and no hero (no main town's hero generated), never eliminated by §2's rule while they keep a
  town.

## 2. Elimination (`scenario_condition_met` 0x4F2CE0, the colours' loop 0x4F2D87 to 0x4F2FA2)

Each check (§4.1 says when) walks the colours 0 to 7 that are not defeated (game +0x1F636 + colour
clear) and looks at two counts of the player record (game +0x20AD0 + 0x168 × colour): the heroes
on the map (+0x1, `hero_count`: a prison's hero is nobody's; a town's garrison hero is not counted,
`town_visitor_to_garrison` 0x4BA106 lowers it, but its owner holds that town) and the towns
(+0x3E, `town_count`). The record's +0x3D (`days_without_town`, i8) is the count-down.

| Heroes | Towns | +0x3D | What the check does |
|---|---|---|---|
| any | ≥ 1 | any | +0x3D = −1 (0x4F2F84) |
| 0 | 0 | any | **defeated** (`player_defeated`, §3); the box: GENRLTXT 96 ("You have been eliminated from the game!") for the viewing player (no picture), else GENRLTXT 6 ("%s has been vanquished!", the colour's player name, `colour_player_name` 0x4CEB60) with the colour's crest (picture type 10, subtype the colour) and the network's 5 s (0x1388) |
| ≥ 1 | 0 | −1 | +0x3D = 7; the box GENRLTXT 7 ("…you have lost your last town. If you do not conquer another town in the next week, you will be eliminated.", the name, the crest) **only** when the colour is a human here (`player_id_is_human_here` 0x4CE970) and the current player |
| ≥ 1 | 0 | 0 | **defeated**; GENRLTXT 8 ("…your heroes abandon you, and you are banished from this land.") for a human here who is the current player, else GENRLTXT 9 ("%s's heroes have abandoned him…"), both with the name and the crest |
| ≥ 1 | 0 | 1 to 7 | nothing |

- **The count-down** is not the check's: `next_player_turn` 0x4C6FE0, after its own check at the
  turn's end (0x4C7019) and the autosave, lowers the **outgoing** player's +0x3D by one when it is
  above 0 (0x4C7133 to 0x4C7147). Every player's turn ends there, a computer's too
  (`adventure_manager_procedure` 0x408BD8 runs `ai_player_turn`, then 0x408C24
  `next_player_turn`). So the seven days are the player's own seven turns, not the calendar's.
- **A town taken** in time sets +0x3D back to −1 at the next check (the first row); losing it again
  starts a new seven.
- **The last hero lost** while the days run (no hero, no town) defeats the player at the next
  check, whatever the days.
- **A player with towns and no hero** is never eliminated by this rule.
- **The days' box** (`player_turn_start` 0x4CC7D0, a human here only: +0xE2 and +0xE1 set): after
  the turn's tests, events and screen, with +0x3D at 0 or more: ArrayTxt 129 ("…this is your last
  day to capture a town or you will be banished…") at 1, else ArrayTxt 128 ("…you only have %d
  days left…" with +0x3D) — an OK box with the crest (0x4CC8DA to 0x4CCA87). Its name: a human
  named "Player" or a computer named GENRLTXT 469 ("Computer") gets its colour's name written
  into the record (+0xCC, kept), the first letter raised; E24 M4 recorded ", you only have 6
  days left" (the single scenario's empty name, CLT-Q100).
- **The timeline**, as the code reads. (a) A colour that starts without a town (the first turn's
  start runs no check, §4.1): its first End Turn's check sets 7 (GENRLTXT 7 for a human), that
  turn's end 6; its next turns' starts show 6, 5, …, 2, then 1 ("last day", its seventh turn);
  that turn's end 0; the first check after it defeats it, which is always another colour's: the
  next colour's turn end or a human's turn start, whichever comes first. (b) A town is captured
  only in an enemy's turn: the capture's check sets 7, silently; the owner's next turns' starts
  show 7, 6, …, 1, and the seventh turn's end brings 0. Either way seven of its own turns.
  `game-week` (recorded, case (a)): Blue, a computer with a hero and no town, set 7 at Red's
  first End Turn, counted down at Blue's seven turns' ends, defeated at Red's day-8 turn start
  (GENRLTXT 9 to Red), then Red's standard victory.
- **Two oddities** of the boxes (quirk candidates, §9): (A-55) GENRLTXT 7 needs its colour to be
  the current player, and a town is captured only in an enemy's turn: only a colour that starts
  without a town sees it (at its first End Turn); one whose last town is captured learns of the
  days from its next turn start's ArrayTxt 128 ("you only have 7 days left"). (A-56) GENRLTXT 8
  ("…your heroes abandon you…") needs the same, but the count reaches 0 at the colour's own turn's
  end, after that turn's check, and the next check is another colour's: in a single scenario or the
  hot seat the defeated human always gets GENRLTXT 9 ("%s's heroes have abandoned him…") with its
  own name, as any other player would (network play may reach 8).

## 3. A defeated player (`player_defeated` 0x4F11A0, read whole)

In this order:

1. `battle_retreated` 0x6985A3 and `battle_surrendered` 0x697744 cleared (so step 7's heroes do not
   go to a tavern as a retreat or a surrender would send them).
2. Its defeated flag (game +0x1F636 + colour) set.
3. Every map Shipyard of its colour (a tile with the entrance bit, class 87, the word's low byte the
   colour; both levels): `set_shipyard_owner` 0x4C6A30 with −1 (neutral).
4. Every mine and Lighthouse record (game +0x4E38C, 0x40 bytes) of its colour: `set_mine_owner`
   0x4C66E0 (the index, −1, how 1: its message record, the guards kept; E08 `E08-economy.md`
   §2.5). Its flag is gone from the map.
5. Every dwelling record (game +0x4E39C, 0x5C bytes, owner +0x57) of its colour:
   `set_dwelling_owner` 0x4C67B0 with −1 (the towns' dwelling counts follow; E08 T88a).
6. Every garrison record (game +0x4E3AC, 0x40 bytes) of its colour: `garrison_set_owner` 0x4C6960
   with −1 (its army stays; E08 T108a1).
7. Its heroes, from the last of its list (+0x8, count +0x1) to the first: `hero_leave_game` 0x4D9EC0
   (1, 0): off the map (a town's garrison or visitor slot cleared; a boat's record freed, so the
   boat disappears: `E08-sea.md` §2), the army cleared, out of the list, the type free (+0x4DF18
   −1), its spell points and movement refilled and one creature of its first starting stack for a
   later hire (E09 `E09-tavern.md` §4, §12); no tavern slot (step 1). Its artifacts stay with the
   hero record, as a dismissed hero's (batch TWN-4's TWN-4-5, recorded: worn and backpack kept).
8. Its two tavern offers (+0x28, +0x2C): a type still reserved for it (+0x4DF18 = 0x40) freed (−1).
9. A network game: a human `next_player_turn_network` 0x4F4C00, else the map message 0x425 (not
   followed).

Not touched: its towns (it has none by §2's rule; a removed player's, §5, would keep them), its
resources and its records' marks (the visited bits, the keymasters' tents, the obelisks, the
puzzle), its artifacts on the map. Nothing tells the other players of steps 3 to 6 beyond the
objects' own message records (the map messages are the network's).

## 4. The game's end

### 4.1 When the game checks (`scenario_condition_met` 0x4F2CE0's 37 calls)

The call returns at once (E27 design §5.1) outside a network seat that checks (0x697788 clear),
during a game's setup, once the scenario ended, or re-entered. With ecx 0 (a check) it is called:

| Caller | When |
|---|---|
| `next_player_turn` 0x4C6FE0 | at every player's turn's end (0x4C7019), after the victory tests (0x5F1610, 0x5F1B10, 0x5F1C80, 0x5F1D40, 0x5F1EF0) and before the count-down; in a network game also after the new day (0x4C71E7) and after a remote turn (0x4C76C9) |
| `player_turn_start` 0x4CC7D0 | at a **human** turn's start (0x4CC831), after the loss's days and the victory's resources and days tests; a computer's turn start runs none |
| `new_day` 0x4C7FE0 | only when the resources victory is met (0x4C8227, `victory_resources_test`) |
| `adventure_battle` 0x4AD470, `battle_aftermath` 0x4AC930, `monster_fight` 0x4A6C60, `monster_flee_offer` 0x4A6DF0, `monster_visit_decision` 0x4A7740 | after a battle on the map (six sites), a monster's fight, flight or joining |
| `town_visit` 0x4AAFD0 (five), `town_change_owner` 0x4C61E0, `ai_town_visit` 0x5253D0 | a town's capture and its visit (the loss's town test first, `loss_town_test` 0x5F2E40) |
| `set_mine_owner` 0x4C66E0, `set_dwelling_owner` 0x4C67B0 | a mine's, Lighthouse's or dwelling's flag |
| `visit_and_update` 0x4AAAA0 (two), `hero_visit_here` 0x4ACCA0 (two), `event_fire` 0x4A0ED0 (two) | after a visit, an event |
| `hero_give_artifact` 0x4E3070, `town_build_effects` 0x5BEDE0, `hero_relocate_spell` 0x41D360 | an artifact gained, a building built, a Town Portal or Dimension Door's arrival |
| `battle_flag_toggles` 0x4F4350 (ecx 1, 2), `hide_whole_map` 0x402450 (ecx 2) | the typed codes' win and loss (not rules) |

So, outside the cheats, the game is checked after everything that can change a hero's or town's
count, an owner or a condition's object, and at each turn's end and human turn's start. **The
first turn of a game is not checked at its start** (`play_run` 0x4EFC58 runs the first day's
events, not `player_turn_start`; E24 M4 recorded a map's first End Turn as the first GENRLTXT 7).

### 4.2 The verdict (the check's second half, 0x4F2FF4 to 0x4F3299)

- **The enemies:** the colours not defeated and not on the viewing player's team (`viewing_player`
  0x4CEA20: in the hot seat the current human, or the last human while a computer plays; a
  viewing player outside 0 to 7 makes every colour an enemy).
- **The special victory, then the loss** (`scenario_victory_test` 0x4F15E0; when it finds nothing,
  `scenario_loss_test` 0x4F2960: increment 2's, E27 design §5.2): a loss sets the end with the
  result 0, a win the end with 1.
- **The standard victory** counts when the map has no special one (game +0x1F89C = 0xFF) or allows
  the standard one too (+0x1F89D): no enemy left → won (`victory_by_conquest` 0x69951C, the end,
  the result 1; GENRLTXT 660 "Congratulations! All your enemies have been defeated! Victory is
  yours!"; a network game sends 0x436). It follows the special tests, so a loss met in the same
  check is overwritten by the conquest's win.
- **No human left:** otherwise, when no colour that is not defeated is a human here (in the hot
  seat: every human defeated), the end with the result 0 (no box beyond the eliminations').
- **A computer's turn** cannot win or lose for itself: the verdict is the viewing player's (the
  human's). A game of computers only (every human defeated) has ended by the rule above.
- GENRLTXT 661 ("All your forces have been defeated, and you are banished from this land!") is the
  network's (`network_loss_message` 0x5569F0, with 660), never shown in a single game.

### 4.3 After the end (`scenario_ended` 0x6972B8's 36 readers)

Set, it stops: the adventure manager at its next message (the executive's exit, E27 design §5.1,
§5.3); the town visit after a capture's check (0x4AB1DC, 0x4AB389: no town screen); `new_day`'s
week and month (0x4C7FF2; E08 `E08-economy.md` §7.2); `next_player_turn` after each of its checks
(0x4C701E, 0x4C71EC, 0x4C76CE: no next player); the computer's turn before its next hero
(`ai_player_turn` 0x525EA0, 0x525FA4, 0x526199; `ai_hero_turn` 0x526284); a message box's
automatic close (0x4F6A5D: the battle result's deadline cleared, so a box opened after the end
waits for its OK); the checks themselves. The rest of the action in progress runs to its next
reader: a battle's own results, the boxes already queued (the eliminations', the conquest's), the
experience and the artifacts are given before the check.

## 5. A player removed

The original has **no resignation** in a single scenario or the hot seat: the system menu's Load,
Restart, Main Menu and Quit end the whole game for everyone (E27 design §4.3). In a **network
game** a seat that drops (`network_player_drop` 0x556430) is announced in the chat (GENRLTXT 470
"%s left the game."; the screen GENRLTXT 659 "Recovering from player drop…" is the host's,
`network_drop_recover` 0x5565E0) and, when the dropped seat's colour was the one in turn and not
defeated, the turn passes to the previous human colour (0x5564CC to 0x5564F2); the colour is
**not** defeated there. In a battle with the dropped seat's side (`network_player_left`
0x47A500): GENRLTXT 416 ("…Player turn will be replayed by computer player.") when it was the side
in turn, else GENRLTXT 417 ("Other player left the game! You won!") and the battle won for the side
that stayed. So a
seat's colour stays in the game, played by the computer; the network's further handling (the
remote turn's flag 0x691209, `remote_turn`) is not followed. The engine's "removing a player"
(SRV-Q29) has no original rule to copy: §13 asks.

## 6. What allies share (the team bytes, game +0x1F879)

The team of a colour is a byte (`player_team` 0x4A5960; `players_same_team` 0x5296D0: both
colours valid and one byte; a colour with itself counts). The hot seat keeps the map's teams
(batch E10's E10-6, E08 T89a; `E08-dwellings.md` §7). No code writes the bytes during play: **no
diplomacy** (E10 decision 2). 85 functions read them; by what a team shares:

| Shared | The original (the reading) | Read by |
|---|---|---|
| **The fog** | every reveal goes to the player and every player of its team: a step's (0x48106B), a placement's, a town's, the Cartographer's (`reveal_terrain_mask` 0x4CA240); the Cover of Darkness hides from every player but the town owner's team (`hide_disk` 0x49D3D0, inferred) | `reveal_disk` 0x49D160 (E08 T34b, E24 M3 T3b) |
| **Marks** | a visit's marks are the team's: the tiles' visited bits 5-12 (`tile_mark_visited_by_team` 0x4FBF90: Tree of Knowledge, Shrines, Witch Hut, Wagon, Warrior's Tomb, Water Wheel, Windmill…), the kind marks (game +0x4E344 + k, every bonus visitable, the Hill Fort, the Altar, the University, the schools), the banks' and Dragon Utopia's, the obelisks' (game +0x4E3E9 + index, the team's bits at a visit, 0x4A98A5) | the visits (E08 T55a2, T108a2, T118a, T127a1) |
| **Ownables** | a mine, Lighthouse, dwelling, Shipyard or garrison of the visitor's team is not flagged: a mine's owner alone gets Leave Guards (an ally's: nothing); an ally's dwelling: no guards, the hire offer (`dwelling_offer_hire`; the dialog read-only, batch E10-6: Max, Recruit and the slider disabled); an ally's Shipyard: the boat offer; a garrison of the team: the exchange, and a human passing it short of its destination takes no visit (`step_garrison_pass` 0x48154B) | `visit_mine` 0x4A39A0, `visit_lighthouse` 0x4A9479, `visit_dwelling` 0x4A18B0, `visit_shipyard` 0x49E2E0, `visit_garrison` 0x4A9061 (E08 T77a1, T88a, T108a1) |
| **Heroes** | a step toward a hero of the team meets it (`visit_hero` 0x4A2800: the meeting; an enemy's in a town: the town's visit; else the battle); in the exchange (`E08-army.md` §2.2) only the local player's stacks are picked up (`swap_army_click` 0x5AF2B0); one goes to an ally's hero **into an empty slot only** (`swap_army_move` 0x5B0DA0: no merge, no swap); an artifact into an empty position or the backpack (`swap_worn_click` 0x5AF590, `swap_backpack_click` 0x5AF990); nothing is taken from the ally; the Scholar's teaching as between one's own (`meeting_scholar` 0x4A2940) | E08 T44a |
| **Towns** | a step onto a gate of the team's town is no siege (`hero_move_step` 0x480E73, case 98); `town_visit` 0x4AAFD0 opens its screen for a human (the computer's visit for a computer), then the town's spells to its heroes (0x5BE030); in the screen the owner alone builds (TD-11, batch TWN-2: the build dialog's owner test 0x5D5AE2) and an ally's visitor buys at the merchants at the current player's count (TD-29, TWN-6-2); the siege of an ally's visitor and the rest are the town lane's (E09 design §101, `TWN-STUB-21`) | E09 T1b1, T14, T54 |
| **Town Portal** | it counts and lists the towns of the hero's team (`hero_relocate_spell` 0x41D360: 0x41D47C none of the team: GENRLTXT 125; 0x41D5B9 the Expert list, the towns of the team with no visitor; 0x41D6CB the nearest of the team), the list's room sized by the team's colours (`town_gate_dialog_create` 0x5C1AB0, 0x5C1B6F); the Castle Gate lists the owner's own Inferno towns only (`town_castle_gate` 0x5D8480) | this survey; the HD mod's notes agree (allies' towns listed below the player's) |
| **Teleporters** | a monolith's exit holding a hero not of the team is a candidate (a battle there); one of the team's is not | `teleport_exit_draw` 0x4CDB80 (E08 T98a1) |
| **Views** | the popups at their top level for the viewer's team: a hero's (`hero_popup` 0x416590, 0x4166BB: level 2, the exact army, as Visions 2), a town's (`town_popup_level` 0x416E63: 3), a garrison's (`garrison_popup` 0x416F80: 3); a mine's resource in its hint (`mine_hint_text` 0x40D670); the cursors (`adventure_mouse_move` 0x40E360: a town or Shipyard of the team, a hero of the team the meeting's); the kingdom panel's and the scenario dialogs' allies and enemies flags (`status_kingdom_create` 0x452B80, `scenario_team_flags` 0x585300, `dialog_team_flags_a` 0x458010, `dialog_team_flags_b` 0x5698C0) | E09 T1b2a, E08 T108a1, E24 M3 |
| **The end** | the enemies are the colours not on the viewing player's team (§4.2); the special conditions' colour is the team's (`victory_resources_test` 0x5F1C80, `victory_mines_test` 0x5F2720, `victory_grail_test` 0x5F1EF0: increment 2's) | §4 |
| **Not shared** | treasuries (a gift is the Trading Post's transfer, any player, ADV-MKT-004), the heroes' armies and artifacts (beyond the exchange's gifts), the border gates' keys (the player's own key bits, `pathfinder_pass_through` 0x56AB40), the turn (each colour its own), towns' recruitment and buildings (the owner's) | |

The computer's reads of the teams (`ai_*`: an ally's dwelling worth 0, an ally's town its visit,
the danger and reach of enemies only, the resource gifts `ai_ally_resource_share` 0x429110) are
E31's (`AI-STUB-18`), with the teams the engine will hold.

## 7. The engine today (`h3-adventure`, `h3-game`)

- **Teams:** `h3_map` reads them (`Teams`), the import drops them; `h3_adventure::allied`
  (`state/economy.rs`) is "the same player" and every team test calls it (22 sites: the views
  `observe.rs` and `h3-game`'s `garrisons.rs`, `town/view.rs`; the mines, Shipyards, garrisons,
  dwellings, banks, weekly and kind marks; the towns' entering and capture `town.rs`; the
  computer's start `computer/start.rs`). 72 comments or docs name E10 as the place teams or the
  end arrive (the market's recipients, the tavern's and thieves' players, the day's income, the
  server's "a whole game never ends before E10", `h3-protocol`'s lobby).
- **Meetings:** a step toward an own hero meets it (GAM-MET-001, GAM-DEC-003); another player's
  is a battle (GAM-BAT-004). No give-only exchange exists.
- **The fog:** one explored grid per player (ADV-FOG-003): a reveal goes to its player alone.
- **The end:** none. Every player plays until the game is dropped; `h3-game`'s system commands
  (ADR-0017 point 2) have no removal ("Removing a player comes with E10", `system.rs`).
- **Stubs that wait for this increment:** `TWN-STUB-21` (an ally's town's trades), `AI-STUB-18`
  (the computer's ally values), `MCL-STUB-4` (the map client's seven days' boxes); for increment
  2 `TWN-STUB-16`, `AI-STUB-9`, `AI-STUB-16`; `ADV-STUB-20` (Town Portal, E09 increment 7) will
  need §6's team list.

## 8. VCMI 1.7.5 and the wiki

**VCMI 1.7.5** (`lib/mapping/MapFormatH3M.cpp`, `lib/gameState/CGameState.cpp`,
`lib/gameState/GameStatePackVisitor.cpp`, `server/CGameHandler.cpp`,
`server/processors/NewTurnProcessor.cpp`; read 2026-11-27):

- **Teams:** the file's team bytes, else every playable colour a team of its own
  (`readTeamInfo`). The fog is kept per team (as the original's reveals). Allies' heroes meet in
  the exchange (`heroExchange`: any non-enemy); taking another player's stacks and touching an
  occupied slot on another player's hero are refused, but merging into an ally's stack of the same
  creature is allowed (`arrangeStacks`): D-82.
- **The standard loss:** no hero and no town (`checkVanquished`), or `daysWithoutCastle` ≥ 7: the
  count is raised by one at each of the player's turn ends without a town and the check runs right
  after it (`NewTurnProcessor::onPlayerTurnEnded`), so the seventh turn's end defeats; a town
  gained clears it, the last town lost sets 0 (D-80).
- **The standard victory:** the players still in the game all of one team; the winner's allies
  win too, every other loses.
- **A defeated player:** its heroes removed (back to the pool, the Grail dropped, a boat with
  them), then **every object it owns** made neutral, towns included (D-81); the others told; the
  check repeated for them; the server plays on while any player is in it and stops when no client
  is connected, so a single game ends once its human's client leaves (D-84).
- Not checked: VCMI's Town Portal and its dropped network players.

**The wiki** ([Victory and Loss Conditions](https://heroes.thelazy.net/index.php/Victory_conditions),
retrieved 2026-11-27): the standard victory is to conquer every enemy town and defeat every enemy
hero; the standard loss is losing every town for seven days, or the last hero while holding no
town. Nothing on teams or a defeated player's objects. The original's manual (E08 `E08-army.md`
§2.2) on the exchange only. The HD mod's notes: Town Portal's list shows the allies' towns below
the player's (the original's list read in §6).

## 9. Disputes, quirk candidates and open points

**Disputes** (`D-<n>`, next 80):

- **D-80** (the seventh day's moment): the original defeats at the first check after the player's
  seventh turn's end (the count lowered after that turn's own check), VCMI at that turn's end.
  The same seven turns either way; the original's moment is the next human's turn start or any
  checked action between. Low impact (the events' order, the env's terminal step).
- **D-81** (a defeated player's objects): the original frees its Shipyards, mines, Lighthouses,
  dwellings and garrisons (it holds no town by the rule); VCMI every owned object. The same for
  the standard defeat; they differ only for a removed player (D-83).
- **D-82** (the allies' exchange): the original's gifts go into an empty slot only; VCMI also
  merges into an ally's stack of the creature. Recommended: the original's.
- **D-83** (a removed player): the original has no rule (a dropped network seat stays in the game,
  the computer plays it, §5); VCMI's resignation not checked. §13 asks.
- **D-84** (no human left): the original ends the game, lost, once no colour a human plays is in
  it, whatever computer teams remain (§4.2); VCMI's server plays on while any player is in it and
  ends when the last client leaves (in a single game the same, by the client).

**Quirk candidates** (`A-<n>`, next 55):

- **A-55** (§2): GENRLTXT 7 shows only to a colour that loses its last town in its own turn (the
  check's "current player" test), which in practice is a start without a town; a town taken in an
  enemy's turn sets the seven days silently and its owner learns of them from the next turn
  start's ArrayTxt 128 ("7 days left"). Harmless (a message's moment), a rule of the events.
- **A-56** (§2): GENRLTXT 8 is not reached in a single scenario or the hot seat: a human banished by
  the days is told GENRLTXT 9 in the third person with its own name. Harmless (the text), a rule of
  the events.
- **A-57** (§4.1): the first turn's start runs no check: a colour that starts without a town is
  warned at the first End Turn, not at the start (Knee Deep in the Dead, Pirates!, Adventures of
  Jared Haret). Harmless; the same seven turns.

**Open points** (not this increment's): the network's remote turn (0x691209) and its messages;
the cheats' win and loss; the conditions' tests (increment 2: `victory_resources_test`,
`victory_days_test`, `victory_mines_test`, `loss_days_test`, `loss_town_test` and the rest).

## 10. Batch GAM-1's plan (the capture harness; rows `GAM-1-<n>`)

Named GAM-1, not E10-1: E08's batch 10 already owns the rows `E10-1` to `E10-6`. Hot seat, Easy,
on maps written by `h3-experiment` (its `teams` field since E08 T89a); each row its prediction from
§2 to §6, two runs as E09's batches. The readings at risk first:

| Row | What | Predicted (the reading) | Settles |
|---|---|---|---|
| GAM-1-1 | Red human with a hero and no town from the start, Blue human with a town, Tan computer with a town; End Turn seven days | GENRLTXT 7 at Red's first End Turn; ArrayTxt 128 at Red's day-2 to day-6 starts with 6 to 2, 129 on day 7; at Blue's day-7 start GENRLTXT 9 naming Red, Red's hero gone, the game goes on | §2's timeline, A-56, A-57, D-80 |
| GAM-1-2 | Red human with one town and a hero, Blue human captures the town in its day-1 turn; then Red takes a neutral town on day 4 and loses it on Blue's day-5 turn | no box at the captures; Red's day-2 start "7 days left", day 3 "6"; day 5 no box (the count reset); Red's day-6 start "7 days left" again | the reset, A-55 |
| GAM-1-3 | Tan (computer, no town) owns a mine, a Lighthouse, a dwelling, a garrison and a Shipyard; Red kills Tan's last hero in a battle | GENRLTXT 6 with Tan's crest after the battle (its check); all five neutral, the guards and the garrison's army kept; later Tan's hero offered in a tavern with one creature and its artifacts | §3, §4.1 |
| GAM-1-4 | Red and Blue allied humans, Tan computer: Red's step beside Blue's hero, the exchange; a step into Blue's town; Blue's hero's popup for Red; Red's walk; Red's Town Portal at Expert | the meeting: a stack into Blue's empty slot, a merge and a take refused, an artifact given; Blue's town's screen, its guild's spells learned, Build refused; the popup exact; Blue's explored tiles grown by Red's walk; Blue's town listed | §6, D-82 |
| GAM-1-5 | as GAM-1-4, Tan's last hero and town taken by Blue | GENRLTXT 6 for Tan, then GENRLTXT 660 for the viewing human, the won screen | §4.2's team verdict |
| GAM-1-6 | Red and Blue humans on two teams, Tan computer; both humans eliminated in turn | the game goes on after the first, ends lost (the lost screen) once the second is out, Tan still playing | D-84 |

## 11. The interfaces' sketch (for T1b)

- **The setup:** `GameSetup` gains the map's teams (a team per colour; without the map's, each
  colour its own), from `h3_map`'s `Teams` by the import (map import v30); checked at the build.
- **The state:** the teams in the adventure state (or passed to it), so `allied(a, b)` reads them
  (an `h3_adventure` change at every one of §7's 22 sites, by signature); per player its defeat
  (`defeated`, the day and why: no hero and no town, the seven days, removed) and its days
  without a town (`Option<u8>`, the original's −1 and 0 to 7); the game's result once over (the
  winning team or none, why: the conquest, no human left, increment 2's conditions). The fog per
  team or every reveal to every member (ADR-0016's view of the explored grid). Game state hash
  v41, adventure state hash v31.
- **The check** (`GameState::check_end`, *agent's call* for the name): after every command's and
  system command's settle, and at the two turn points (§4.1: the outgoing turn's end, before the
  count-down; a human's turn start); the eliminations in colour order (§2's table), then the
  standard victory, then no human left (§4.2). Idempotent, so a check after each command stands
  for the original's 37 calls.
- **A defeat** (`player_defeated`'s steps, §3): the ownables freed (their own events, as their
  flags' changes are told now), the heroes into the pool from the last (the dismissal's path,
  E09's `leave`), the tavern offers freed; the turn order skips the player; the market's
  recipients and the thieves' and tavern's players leave it out (the inbox notes of T136 and E09).
- **Events and views:** `PlayerWarned { player, days }` (GENRLTXT 7 at 7, ArrayTxt 128 or 129 at a
  turn start; the owner's), `PlayerDefeated { player, why }` (every player; the shell picks §2's
  box: 96, 6, 8 or 9), `GameOver { result }` (every viewer);
  each player's `defeated` public in the observation, the days left the owner's.
- **After the end:** every command refused (`GameCommandError::GameOver`, nothing changed); the
  command that ended it keeps its told events up to the check; a capture's town screen not opened
  (§4.3).
- **Allies:** the meeting of an ally's hero as a decision (GAM-DEC-003's) with the give-only rule
  (§6), the town of an ally entered (the town lane's screen), the views at the top level for the
  team (ADV-FOG-006, ADV-GAR-006, the town's view).
- **A removal:** a system command (`SystemCommand::RemovePlayer { player }`, the server's,
  SRV-Q29) by §13's answer; `h3-protocol` maps the new commands, events and errors (the global
  trap: engine types on the wire, the server lane's, `cross-lane`).
- **The hand-overs:** the server lane's end of a hosted game (`h3-server`'s "never ends before
  E10"); the env's terminal (`h3-sim`, the AI lane: the result and each player's defeat); the end
  screens (E27 M5: the result, each colour's defeat and why, the day; the score is increment 2's).

## 12. The proposed rule ids

New groups (E10 decision 7): **`GAM-TEM`** (teams) and **`GAM-END`** (the game's end).

- `GAM-TEM-001` the teams from the map (each colour its own without); no diplomacy.
- `GAM-TEM-002` a reveal for the team (amends ADV-FOG-003).
- `GAM-TEM-003` the meeting of an ally's hero and its gifts (amends GAM-MET-001, GAM-DEC-003).
- `GAM-TEM-004` what a viewer sees of the team's heroes, towns and garrisons (amends ADV-FOG-006,
  ADV-GAR-006 and the town lane's view).
- The marks and the ownables keep their rules (ADV-VIS-006, ADV-BON-009, ADV-OWN-001, ADV-DWL-004,
  ADV-GAR-003, -004, ADV-SHY-003, ADV-MIN-002, ADV-BNK-003, ADV-TEL-003 name the team already):
  their "the player alone until E10" goes.
- `GAM-END-001` the eliminations (no hero and no town), `-002` the seven days and their warnings,
  `-003` a defeated player's leftovers, `-004` the standard victory by team, `-005` no human left,
  `-006` the check's moments, `-007` after the end, `-008` a removed player (by §13's answer).
- Quirks (`ADV-QUIRK-59` on): A-55 to A-57 by the checkpoint's answers.

## 13. For T1b: the checkpoint's questions

1. **The cut:** the design note with both parts and the build in rounds: (a) teams first (the
   setup, `allied`, the fog, the meeting, the views), (b) the end (the check, the defeat, the
   result, after the end, the events), (c) the removal and the hand-overs, then the batch and the
   audit (recommended); or the end first (the env's terminal sooner).
2. **The check's moments** (D-80): the original's (after every command, at the outgoing turn's end
   before the count-down, at a human's turn start), recommended; or VCMI's (the count raised at
   the turn's end and the defeat there).
3. **A-55 and A-56** (the warning only at a start without a town; GENRLTXT 9 to the banished
   human): the original's events, harmless and accepted (recommended: the shell shows the boxes
   the original shows), or always warn the owner and tell it in the second person.
4. **A-57** (no check at the first turn's start): the original's (recommended: the same seven
   turns, the warning at the first End Turn) or a check when the game is built.
5. **No human left** (D-84): the original's end, lost, by `PlayerSetup::human` (recommended; an env
   marks its seats), or play on to one team.
6. **A removed player** (D-83, SRV-Q29): (a) defeated by §3's steps and its towns made neutral too
   (recommended: VCMI's for the towns, the original's for the rest); (b) its colour to the
   system's computer player, as the original's network (needs E31); (c) defeated with its towns
   kept, owned by nobody's turn.
7. **The allies' exchange** (D-82): give-only into empty slots, as the original (recommended), or
   VCMI's merge too.
8. **The town lane's part** (an ally's town: the screen, the trades `TWN-STUB-21`, the siege of an
   ally's visitor): built by this lane `cross-lane` with the user's consent, or handed to the town
   lane's inbox once `allied` reads the teams (recommended). And Town Portal's team list into
   `ADV-STUB-20`'s reason for E09 increment 7.
9. **Batch GAM-1** (§10): its six rows, run before the build's last round (recommended) or after
   T1b's acceptance at once.
