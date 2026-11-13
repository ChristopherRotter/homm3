# E27 — archive: M1 (reading and captures) and its review list

Moved out of `E27-game-shell.md` on 2026-11-13 (M1 T3, M1's closure), verbatim: milestone M1's
task list and its review list, accepted by the user at T3 (decision 22). The build record:
`E27-log.md`; the readings and captures: `E27-design.md` §1 to §6.

### M1 — reading and captures (refined 2026-11-12, M0)

| Task | Scope | Status |
|---|---|---|
| T1a1 | The catalogue, part 1a: the start (the clips' order and skipping, the loading bar), the main menu, New Game's and Load Game's menus, Quit | done 2026-11-12 (design §2; CLT-Q84 to CLT-Q86) |
| T1a2 | The catalogue, part 1b: Multiplayer (its modes, decided as a `CLT-Q`) and the hot seat's names, High Scores, Credits | done 2026-11-12 (design §2.6 to §2.9; CLT-Q87 to CLT-Q89) |
| T1b1 | The catalogue, part 2a: the scenario screen's frame, its list and information panel in its three modes (new, load, save), the difficulty, Begin and Back (decision 11) | done 2026-11-12 (design §3; CLT-Q90, CLT-Q91) |
| T1b2 | The catalogue, part 2b: the advanced options (the players' setup to `GameSetup` or an E10 gap), the random map's tab, the game's begin | done 2026-11-12 (design §3.12 to §3.18; CLT-Q92 to CLT-Q95) |
| T1b3a | The catalogue, part 2c: the system options and the menu in play (decision 14) | done 2026-11-12 (design §4; CLT-Q96 to CLT-Q98) |
| T1b3b | The catalogue, part 2d: the end of a scenario (victory, defeat, their clips and messages, the win screen and its score, the high score's entry) | done 2026-11-12 (design §5; CLT-Q99, decisions 16, 17) |
| T2 | The shell's clips cut from the user's `VIDEO.VID` (on the user's computer, as `battle-videos.vid` was; done at T1a1 and T1b3a, design §1); the harness on every catalogued screen; the reference captures; the traps | done 2026-11-13 (design §6: 84 recipes, each recorded twice; CLT-Q100 to CLT-Q102; decision 18) |
| T3 | M1's closure: a fresh reviewer, the review list and the questions with the user, M2's refinement (M2 T0) | done 2026-11-13 (two fresh reviewers, 57 findings fixed or listed, design §6.4; every recipe but the two full starts recorded again, all as kept; CLT-Q100 to CLT-Q118; decisions 19 to 22; M2 refined) |

**M1's review list** (the agent's calls, *provisional*; rewritten at T3 after the fresh review so
that each item can be decided from its text; **accepted by the user, 2026-11-13**, decision 22):

1. Not built (T1a1, design §2.10): the CD status, its notices and the menus without Single,
   Campaign and Tutorial (the client has no CD; no installation of the user shows them); the free
   disk space check (GENRLTXT 708 below 5 MB, the menu then ends); a DirectPlay lobby's start; the
   single instance (a second client may start); F4's window mode (E29). A player sees none of them
   on a working installation.
2. T1a2 (design §2.10): the high scores in the client's own file, never the installation's
   `HiScore.dat` (not imported: the original's scores do not show), the defaults read from
   `CampHigh.txt`; **the player's name a client setting** (the original's registry value `Network
   Default Name`), empty by default, at most 21 characters (the hot seat's edits; the high score's
   entry takes 40): CLT-Q100 takes it for a single scenario's player, the colour's name when it is
   empty (without the setting, the colour's name always). The leftover `RMT*SC.BIN`'s deletion not
   built (the client leaves no such file).
3. T1b1 (design §3.10): not built: the scenario screen's network paths (the host's relays, a guest's
   wait, the chat: network play is E28's, decision 10); the busy cursor's helper thread (the client
   draws the same busy cursor from its own loop); the edition's format check (one edition,
   Complete, so every listed map is the edition's). The original's `.GM?` saves not listed
   (decision 8: a player's saves from the original do not show; the client lists its own records);
   the autosave keeps the original's name, `AUTOSAVE`, as its record's file name.
4. T1b2 (design §3.10): the advanced options' network paths and the random map's generator not
   built; the bonus, the handicap and the turn's duration drawn and chosen with their effects
   stubbed (a `CLT-STUB` each at M3); the hot seat's swap of CLT-Q94 in its detail (a taker without
   a colour, the computer after the last player) provisional until M3 builds it.
5. T1b3a (design §4.7): the system options' settings in the client's `settings.toml` beside the
   combat options' (one set: the volumes and Animate SpellBook shared by both dialogs, as in the
   original's registry), written whole at the close when anything changed. **Autosave:** the
   original has no item for it (its registry value alone); kept on, with no item (else: off; or an
   item in the client's own settings later, E29). The network paths and the tutorial's disabled
   boxes not built; the step sounds' reload and the second hero lift not copied (nothing visible).
6. T1b3b (design §5.8): the scores in the client's own file; GENRLTXT 116's unread text, the cheat
   flag, the typed codes and their "Cheater" land, the victory codes 11 and 12 (no map) and the
   map's animations under the score screen not built (nothing a player sees); the network's texts
   and messages (0x3FB, 0x3FC, 0x436) not built.
7. T2 (design §6, the runbook's "The shell's screens"): **the method**: the recipes on a clock
   frozen from the game's first read, each recorded twice, their digests kept per group
   (`clients/capture/shell/kept/`), the shots only in a sandbox (a session records them again:
   T3's runs gave the kept digests back, every shot); the ends reached by writing the game's memory
   (`poke`), the boxes and screens after it the game's own.
8. T2: **TCP/IP's stage 2 shots** show the sandbox's address, so their digests are the machine's:
   (a) accept (a `CHANGED` there expected and listed); (b) mask the address's box in their digests
   when M2's comparison is built (T4); (c) drop those shots. The agent's call: (a) until M2 T4,
   then (b).
9. T2: **`start-smacker`'s second run** shows its frames 10 ms earlier from `H3Intro`'s first frame
   (the same frames in the same order): (a) accept, both times kept; (b) record it again at 1 ms
   steps (about an hour more, a slower recipe). The agent's call: (a).
10. T3 (design §6.4): **no sound recorded**: the shell's recipes run without a sound driver
    (`Sound Volume` 0); the screens' sound sequences and the waits a real driver makes are recorded
    by the task that builds each screen, after h3shell gains h3drive's `sound` (M2 T4); else: all
    groups' sound recorded now, a harness task before M2's screens.
11. T3 (design §5.8, §6.4): **the boxes in play** (§5.1's eliminations, §5.2's conditions' texts)
    and the hand-over at a scenario's end are drawn over the map client's screen: which lane draws
    them is settled with the map-client lane at M4's T0.
12. T3 (design §6.4): the readings and recordings M2 to M4 still need (the fades, the edit field,
    the names' length, play's requests, the lost screen's first frame, `difficulty_names`, the
    High Scores' creatures, the score's boundaries, the states not reached) are each task's, as
    §6.4 lists them, not a reading task of their own.
