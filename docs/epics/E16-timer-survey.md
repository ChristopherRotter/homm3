# E16 survey: the original's turn timer (E16 M5 T15)

Date: 2026-11-13. Scope: E16 M5 2b (SRV-Q31, SRV-Q32): what the original does with a player's turn
duration, at its end, in a battle and in a level-up's dialog, for T17 (the clock hosted) and the
user's check of SRV-Q31's decisions (SRV-Q33). Input, not rules: the clock is the host's policy,
outside the engine's state (T16), so nothing here becomes a catalog rule; a fact binds once a task
builds it and its epic says so.

**Sources** (retrieved 2026-11-13):

- `original-data`: a code reading of the user's `Heroes3.exe` (SoD 4.0 Complete, SHA-256
  `aac39e8c…`); every address cited has an entry in `docs/exe/map.toml` or `structs.toml`. The
  reading follows the human player's path; the network branches are named where they differ and
  not followed further. **Read whole:** the timer's methods (`turn_timer_on` 0x557A80,
  `turn_timer_expired` 0x557AA0, `turn_timer_poll` 0x557AF0, `turn_timer_ends_within` 0x557D00,
  `turn_timer_stop` 0x557D60, `turn_timer_set_limit` 0x557D70, `turn_timer_start` 0x557D90,
  `turn_timer_pause` 0x557DD0, `turn_timer_resume` 0x557DF0, `turn_timer_clear` 0x5522B0),
  `chat_system_message` 0x553960, `dialog_proc_base` 0x41B1C0, `dialog_end_on_time_expired`
  0x41B190, `dialog_construct` 0x41B040, `message_box_proc` 0x4F08D0, `dialog_default_proc`
  0x4F0FC0, `level_up_dialog_proc` 0x4F9780, `network_turn_window` 0x419C80, `game_is_multiplayer`
  0x4CEC90. **Read at the timer's sites:** `adventure_manager_procedure` 0x4087B0 (0x4087F3),
  `message_box_build` 0x4F6990 (0x4F69B0 to 0x4F6A65), `level_up_dialog` 0x4F8880 (0x4F895A,
  0x4F96A1), `hero_level_up` 0x4DA720 (its three dialogs), `adventure_battle` 0x4AD470 (0x4AD4A4,
  0x4AD633, 0x4AE324, 0x4AE930), `next_player_turn` 0x4C6FE0 (0x4C7086, 0x4C77F2 to 0x4C78F0),
  `turn_start_screen` 0x419E00, `game_main` 0x4EE3E0 (0x4EFE3E to 0x4EFE85), the duration's
  setters (`scenario_load_begin` 0x58C290, `new_game_start` 0x58C570, 0x589B20), `town_screen_proc`
  0x5D3240, `recruit_dialog_proc` 0x550940, `swap_manager_proc` 0x5AFDF0, `town_screen_load`
  0x5C6870 (its two calls), `message_with_pictures` 0x429110 (0x429741). **Table dumped:**
  `turn_duration_minutes` 0x641AE8. The timer's whole caller list (34 references to 0x69D630 in 25
  functions) was walked; the move (`hero_move_loop` 0x407B80) is not among them.
- `original-data`: `GENRLTXT.TXT` (records counted from 0 by CRLF), `turndur.txt`.
- The manual (`Heroes3_Manual.pdf`, the user's install): p. 11, the slider "controls how long
  human player turns can last (when not in combat)"; p. 62, at a network turn "you are also
  prompted with a window and a sound". The AB and SoD manuals and `HEROES3.HLP` add nothing (the
  help repeats p. 11).
- `vcmi` 1.7.5 (`server/TurnTimerHandler.cpp`, `lib/TurnTimerInfo.{h,cpp}`), a reference only.

## 1. The timer and its duration

- **One record** for the game, `turn_timer` 0x69D630 (0x14 bytes, `structs.toml`; H3API's
  `H3TurnTimer`, its names agree): +0 the last warning's clock time, +4 the turn's start (0: not
  running), +8 the limit (0: unlimited), +0xC the time from +0 to the next warning (0: no more),
  +0x10 the clock time a pause began (0: not paused). Every time is `clock_ms`, WINMM's
  `timeGetTime`: wall time in milliseconds.
- **The limit** is the game's duration index (`game` +0x1F843, 0 to 10, the scenario screen's
  bar 338) through `turn_duration_minutes` 0x641AE8 (1, 2, 4, 6, 8, 10, 15, 20, 25, 30, 0) × 60 000
  ms, set by `turn_timer_set_limit` 0x557D70 (which also clears the start) at a new game
  (`new_game_start`), a load (`scenario_load_begin`) and 0x589B20 (a network game's setup,
  inferred). Index 10, "Unlimited", gives 0: no timer at all. The bar is enabled in a single
  player game, the hot seat and for a network host (`E27-design.md` §3.13), so a single player's
  turn against the computer can be timed too.
- **Who is timed:** every test needs the current player to be human (both human flags of its
  record, `player_is_human_both` 0x4BAD80); a computer player's turn is never timed. **Never in a
  campaign** (`campaign_game` 0x69774C: every method returns at once).
- `turn_timer_on` 0x557A80: a limit and no campaign. `turn_timer_expired` 0x557AA0: on, the
  current player human (or none), not paused, started, and more than the limit since the start.
  `turn_timer_ends_within(m)` 0x557D00: on, started, not paused, and now + m past the end.

## 2. When it runs

- **Started** (`turn_timer_start` 0x557D90: start and last warning now; the first warning after
  a quarter of the limit, after half of it for a limit of a minute or less) at each human turn
  (`next_player_turn` 0x4C78F0, for a player with both human flags), at a network turn
  (`turn_start_screen` 0x419F43), and when play begins (`game_main` 0x4EFE85, after the first
  day's events): **a loaded game restarts the current turn whole** (a save keeps only the
  duration's index, `scenario_info` +0x4A7).
- **Stopped** (`turn_timer_stop` 0x557D60: start, last and next 0) at the turn's end (0x408835),
  at the next player's turn (0x4C708B) and at a reload (`game_reload_reset` 0x4CED0D).
- **Paused** (`turn_timer_pause` 0x557DD0 notes the clock in +0x10; `turn_timer_resume` 0x557DF0
  moves the start and the last warning on by the pause's length, so the paused time is given
  back): **every battle on the map** (`adventure_battle` 0x4AD4AC to 0x4AD63F or 0x4AE942, and its
  unwind 0x4AE9B0), and a town screen's (re)build (`town_screen_load` 0x5C6870, from the screen's
  opening, three places of its procedure and the Castle Gate: the graphics' load not counted).
  While paused nothing expires, no warning shows and no dialog gets the timer's deadline.

## 3. The warnings

`turn_timer_poll` 0x557AF0 runs from the tick every 60 ms (`adventure_tick` 0x4ED635; in every
modal loop, also during a move and in a dialog). When the next warning is due it writes "<SYSTEM>:"
(GENRLTXT 54) and a text into the screen chat (`chat_system_message` 0x553960, the chat line of
the adventure screen, with the chat's sound): with *r* the time left, above a minute GENRLTXT 630
("You have %0.1f minutes remaining in your turn.", *r* / 60 000 as an f32) or 629 ("1 minute",
for 1 to 1.2 minutes); else the whole seconds *r* / 1 000 (29 shown as 30): 627 ("1 second") or
628 ("%d seconds"). The next warning (+0xC, from now): at half the limit while *r* is above it and
the limit above 2 minutes; else at a minute left while *r* is above a minute; else at 10 s left
while above 10 s; else every second, none once *r* is below 2 s. The poll fires just after each
point, so the whole seconds read one less: per duration (a reconstruction of the schedule, the
texts by *r*):

| Duration | The warnings |
|---|---|
| 1 min | 30 seconds; 9, 8, …, 2 seconds; 1 second |
| 2, 4 min | 1.5 (3.0) minutes; 59 seconds; 9 … 1 |
| 6 to 30 min | at a quarter elapsed (4.5, 6.0, 7.5, 11.2, 15.0, 18.7, 22.5 minutes), at a quarter left (1.5, 2.0, 2.5, 3.7, 5.0, 6.2, 7.5 minutes); 59 seconds; 9 … 1 |

GENRLTXT 629 never shows on these schedules. In the last 59 s the chat's text is left out when
the adventure manager closes (its +0x34 2) or a network flag is set (the sound still plays).

## 4. The turn's end

- **The test** is in the adventure manager's procedure (0x4087F3), at every message it handles,
  before anything else but a scenario's end: once `turn_timer_expired`, a message box with
  GENRLTXT 202 ("Your time has expired. Moving on to next turn.", OK, closing itself after 15 s),
  then `turn_timer_stop` and `next_player_turn`. Nothing else ends a turn by time.
- **A moving hero finishes its move.** The walk (`hero_move_loop` 0x407B80) runs inside the
  procedure's handling of the click or key that started it and tests nothing of the timer (the
  warnings still show); its steps, visits and battles run as without a timer (their dialogs as in
  §5); the turn ends at the procedure's next message.
- **The screens and dialogs open at the end close themselves** (each tests `turn_timer_expired`
  in its procedure, then control returns to the adventure manager, which ends the turn): the town
  screen (0x5D3281, its exit 0x7800), the recruit dialog (0x550968, its Close, nothing bought),
  the exchange screen (0x5AFE20, its exit; a network game's branch not read), every dialog built
  on `dialog_construct` 0x41B040 through `dialog_proc_base` 0x41B1DA (H3API's
  `vEndDialogOnTimeExpired`, 0x41B190: the result the dialog's +0x58, 0x7801 Cancel unless the
  dialog sets another), the message boxes and the level-up (§5, §7).

## 5. Message boxes at the end

`message_box_build` 0x4F6990 keeps a box's own timeout (a caller's 2 to 19 999 ms, its deadline
then that far ahead) only in a network game, while the turn timer is on or on a remote turn; then, with no
deadline yet, **a box opened with less than 15 s left** (`turn_timer_ends_within(15 000)`, true
also after the end) gets a deadline 15 s ahead (`battle_result_deadline` 0x697784, the modal
dialogs' deadline; a network flag gives one too); it notes its opening time (0x699588). At its
deadline a box closes with the result 9999 (`dialog_default_proc` 0x4F0FC0), which its caller
takes as its "else": a question's No, the treasure chest's gold, the Arena's Attack (survey
`E08-rewards.md` §2, §6.2). A box with no deadline open at the turn's end (`message_box_proc`
0x4F0906, opened at least 15 s before) is answered by its style (0x4F09D3): 1 and 5 OK (0x7802),
2 Cancel (0x7806: a question's No), **7 and 10 one of the two pictures at random** (`crt_rand`
0x617E5C & 1: the C library's rand, not the game's seeded draws; style 7 the chest's gold or
experience, 10 the Arena's Attack or Defense), the rest 0x7801. (Opened less than 15 s before
with no deadline, a case the build's test leaves to a paused turn only, the box stores the time
left as the deadline, so it closes at once with 9999.) `message_with_pictures` 0x429110 passes a
15-s timeout while the timer is on. The box of the end itself (§4) passes 15 s.

## 6. Battles

- **The turn's timer stops for every battle** (§2): a wandering monster's, a guarded object's, a
  town's or another hero's, against a computer or another human; the battle's time is given back
  (the start moves on), so a turn's limit counts only the time outside battles, as the manual
  says. A turn that expired just before a battle (a move's last step into a monster) is ended
  after it.
- **A battle has no clock of its own.** No battle code reads the record; the only timeouts in a
  battle are a network game's (the result window's 15 s, `battle_end` 0x47785F, and the network
  boxes' 15 s). A network player who leaves during a battle: GENRLTXT 416 ("The current player
  left the game! Player turn will be replayed by computer player.", `network_player_left`
  0x47A500 from the battle loop; not read further).
- **The battle's aftermath is inside the pause:** the winner's level-up (0x4AE324, 0x4AE37C,
  0x4AE405, 0x4AE42C) runs before the resume (0x4AE942), so it is never timed in a single game or
  the hot seat (§7).

## 7. The level-up's dialog

`hero_level_up` 0x4DA720 shows the dialog to a human owner at its own computer (survey
`E08-progression.md` §2.5); its time:

- **No time of its own:** while the turn runs, the dialog is open on the turn's time; at the end
  it closes with 9999 (`level_up_dialog_proc` 0x4F9780: `dialog_proc_base`'s close, the result
  then 9999), which hands the choice to the computer's (`ai_level_up_choice` 0x52BBD0; one offer is
  learned anyway). Our `h3_game::default_answer` takes that choice already (HER-LVL-006).
- **15 s:** a dialog opened with less than 15 s left (`level_up_dialog` 0x4F96A1); in a network
  game one whose owner is not the current player (0x4F895A: a battle's other side, its level-up
  shown at its own computer during the attacker's turn); 2 s while a remote turn runs (0x4F96D1,
  not read further). At the deadline 9999.
- **At once:** in a multiplayer game (`game_is_multiplayer` 0x4CEC90: a network game or the hot
  seat), a dialog opened after the turn's end gets the raw 15 000 as its deadline (0x4DAA7F,
  0x4DAAF6, 0x4DAC0A; a duration where a clock time is meant), so it closes at its first pass with
  the computer's choice.
- **Unlimited after a battle** in a single game or the hot seat (the pause, §6); the network
  game's other side keeps its 15 s.

## 8. The "your turn" window

- **The hot seat:** the next player's covered view with "%s's turn." (GENRLTXT 14,
  `covered_view_new_week` 0x4CA840, a modal box) comes before `turn_timer_start` (0x4C781E, then
  0x4C78F0): not counted.
- **A network game:** the timer starts first (0x419F43), then `network_turn_window` 0x419C80: the
  window restored and brought to the front, `SysMsg.wav`, "%s's turn." in a box with a 15-s
  timeout; unanswered, again (the window flashed, 0.5 s, the sound) until it is answered or the
  turn has expired. The window counts against the turn.

## 9. VCMI compared (reference)

VCMI 1.7.5's timer is not the original's: a turn timer, a base (reserve) timer, a battle and a
unit timer per player (`TurnTimerInfo`); on the map the turn timer, then the base, count down;
in a battle the unit, battle, turn and base timers count down in that order, so **a battle eats
the turn's time** once its own run out; a timed-out side Defends in a battle between players, and
a battle against neutrals is let run to its end, the turn ending after it on the map. The turn
ends only once no query (dialog) is pending ("wait for replies"): VCMI answers no dialog itself.

## 10. Compared with SRV-Q31, SRV-Q32 and T16

| Point | The original | SRV-Q31/Q32, as built in T16 | |
|---|---|---|---|
| The durations | eleven, a human's turns only, never a campaign | one of the eleven per offer | same |
| The end | the open dialogs closed or answered, then the turn ended | each open decision's default, a battle played out, then the turn ended | same in kind |
| A moving hero | finishes its move, then the end | a move is one command, the end between commands | same |
| A battle | **the turn's timer stops, the battle's time given back; no battle clock** (also between two humans) | the turn's timer runs on against a monster or a garrison, the computer plays the side at its end; between players' heroes Fischer's chess clocks, the attacker's turn timer stopped | **differs** (SRV-Q33 a) |
| The level-up's time | the turn's (at least 15 s); 15 s for a network game's other side; unlimited after a battle in the hot seat | an offer's setting, 60 s by default, every timer stopped meanwhile | **differs** (SRV-Q33 b) |
| The level-up's default | the computer's choice | the original's choice (`default_answer`) | same |
| A two-picture box (chest, Arena) | at random (`crt_rand`) if open 15 s, else gold or Attack | gold, Attack | **differs** (SRV-Q33 c) |
| A question, OK, Cancel | No, OK, Cancel | an offer's Decline, Close or Leave | same |
| A disconnected player | the computer replays its turn (network, inferred) | the turn timer only, no player removed | differs; SRV-Q31's (3), not asked again |
| A restore | the turn whole again (a load) | the time left (the journal) | differs; T17's, not asked (a server restart is no load) |
| The "your turn" window | hot seat outside the turn, network inside | the turn starts at its day; the window the client's | network's |
| The warnings | the chat line's schedule (§3) | `GameClock` tells the time left; the client's | client's |

**The user's answers** (SRV-Q33, 2026-11-13): (a) SRV-Q31's battles kept as built, a deliberate
deviation; (b) the level-up's default kept at 60 s; (c) `default_answer`'s gold and Attack kept.

## 11. For T17 and the clients

- T17's offer: the eleven values from `turn_duration_minutes`; "unlimited" none. The level-up's
  default stays T16's 60 s (SRV-Q33 b; 15 s is the original's dialog time).
- The clients (the map client's adventure screen): the warnings' schedule and texts (§3), the
  end's box GENRLTXT 202 (15 s), the hot seat's covered "your turn" box outside the time and the
  network one inside it (§8), all drawn from `GameClock` and the system's end-turn command.
