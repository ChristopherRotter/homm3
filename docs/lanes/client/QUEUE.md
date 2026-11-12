# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E27, the game shell** (refined 2026-11-12, M0, with the user: `docs/epics/E27-game-shell.md`,
decisions 4 to 8, CLT-Q79 to CLT-Q83; the clips: `E27-design.md` §1). M1 (reading and captures)
below; M2 to M4 are refined at their T0s. The original files: stage them from the user's
`original-data/install/` (one large file per staging call; the user's computer must be online);
never from the installation. The captures need the five DLLs, `Heroes3.snd`, `H3ab_ahd.snd`
and `H3ab_ahd.vid` too; the harness's runbook: `clients/capture/README.md`. The client's gates:
`clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate
`cargo run -p h3-battle-client -- gate` from `clients/`.

#### E27 M1 T1b — the catalogue, part 2: a game's setup and the screens around play

- **Scope:** as T1a for scenario selection (the list, its filters and sort, the information
  panel, the difficulty, the players' setup: colour, human or computer, town, hero, starting
  bonus, handicap; the advanced options; the random map's tab, decided as a `CLT-Q`), Load's and
  Save's dialogs (the list, the file name's entry), the system options (every setting, where the
  original reaches the dialog, where it keeps a setting), the system menu in play, and the end
  of a scenario (victory, defeat, their clips, messages and the high score's entry). Per setup
  choice: the `GameSetup` field it fills, or the E10 gap it waits for (decision 6). Into
  `E27-design.md` §3. From T1a2 (design §2.7, §2.8): the hot seat's names (`hot_seat_names`
  0x69CA50) in the players' setup; a won scenario's screen `scenario_win_screen` 0x4F3F60
  (`HSANIM`, `HSLOOP`, the score) and the new entry `high_score_entry` 0x4E91D0 (`HighName.pcx`).
- **Acceptance:** as T1a's; the setup's choices mapped to `GameSetup` or to a named gap.
- **Depends on:** T1a2 (done). **Status:** ready.

#### E27 M1 T2 — the shell's clips, the harness on every screen, the reference captures

- **Scope:** the shell's clips: `original-data/shell-videos.vid` exists (made on the user's
  computer at T1a1, design §1: `VIDEO.VID`'s ten shell entries; a clip T1b places beyond them is
  added the same way, reading the installation only); the harness's `VIDEO.VID` holding both
  archives' clips (`setup.sh`; `h3ab_ahd.vid` stays whole beside it); the harness's steps to every
  catalogued screen and its probes (the runbook, a section of its own); the reference captures of
  each screen (stills; a clip's frames where a screen plays one), each recorded twice; the start
  with its clips under Wine; the traps into `TRAPS.md`. Cut at its start with the user if it is
  more than half a context.
- **Acceptance:** every catalogued screen reached by a recipe and captured, two recordings the
  same or each difference listed; the harness check passes; the Definition of Done.
- **Depends on:** T1b (the user's computer only if T1b adds a clip). **Status:** ready after T1b.

#### E27 M1 T3 — M1's closure and M2's refinement

- **Scope:** a fresh reviewer on the catalogue and the recipes, its findings fixed or listed;
  M1's review list and the open questions with the user; M2 T0: M2's tasks into this queue (the
  Smacker decoder, the start, the menus, the shell's state machine and its headless driver,
  CLT-Q2's loading; where the shell's code lives, CLT-Q79).
- **Acceptance:** the review list accepted by the user; M2's tasks queued; the Definition of
  Done (docs).
- **Depends on:** T2. **Status:** ready after T2.

## Inbox

Other lanes add entries here. The client lane turns each into a task, or moves its detail into
[`NOTES.md`](NOTES.md) and keeps one line here while the work waits.

- 2026-11-12 (the server lane, E16 M5 T7a1, branch `claude/keen-tesla-q5uxid`;
  informational for `h3_view::game_record`): protocol **v44**: a game's player may get a fresh
  `GameSnapshot` after a command's messages (before its `GameTurn`, never while a battle is open
  in its view), which replaces its view; a bounded record begins with its newest kept snapshot.
  `GameReveal`'s tiles are packed (a palette and per level a rectangle); your reader's wildcard
  still skips it.

- 2026-11-12 (the server lane, E16 M5 T5c, branch `claude/optimistic-meitner-j73r89`; **your
  rule followed, your docs to update**): a game's battle's last `Snapshot` is sent as your note
  of 2026-11-12 asked (protocol v43): the player's very next message after the `GameEvents`
  holding `BattleEnded`, none when those events open another battle for it; `SRV-STUB-14` is
  lifted, so `h3_view::game_record`'s doc ("the game sends no last snapshot yet") is stale. A new
  message `GameReveal` (what the player newly sees on the map) comes after it when the player's
  explored tiles grew; your reader's wildcard skips it.

- 2026-11-12 (the adventure lane, E08 T110, branch `adventure/e08-t110`; the capture harness, informational): `h3adv` gains a garrison's exchange as a screen (`garrison`, vtable 0x643890; `garrison_dialogs`, `bar:selected`, `bar:clicks`, `creature_windows`), the battle plan `{ book = "<spell>", then }` (the key C at each side's first stack's turn; `book:<side>`; `h3mem`'s stacks gain `spells`, `battlefield` gains `anti_magic`), a step's `cast` (the adventure book), a list of `hint` tiles (`hints`; an unexplored tile's hint says so), an end-turn step's `to_day`, `skip_quietly`, the hero screen's `morale` and `luck` (items 116, 117) with `popup:lines`, and `garrison:x,y`, `anti_magic`, `fountain:x,y`, `spring:x,y`, `hero:bit:<n>`, `hero:word:<offset>`, `dialog:items|boxes` (`h3mem.garrisons`, `battle_hero`). **Behaviour changed:** `h3drive.gold_bonus` clicks each row's arrow until the memory reads Gold (at most four: a heroless town player has no Artifact); `hint_at` parks the mouse first and reads until two reads agree (at most 3 s); `play_turns` gives no order once the battle is decided; an action's `row:` and `+` fields; a recipe's `+anything odd` kept beside the notes. The harness check passed. `clients/capture/README.md`, "Garrisons and the bonus visitables".

**E17's leftovers** (placed at its closure, M7 T4, with the user): each waits for the work named;
the battle client's code is this lane's, so the lane builds it then.

- **With E24 M4 T5** (the map client's battles; its inbox told, 2026-11-12): M6 T5e, the captured
  artifacts' message after Eagle Eye's (deferred by the user, 2026-11-07;
  `captured_artifacts_message` 0x4772B0, design §12.21: GENRLTXT 31, eight artifact pictures a
  box through `h3_view::message::pictures`, a pickup sound each), its list the adventure's
  capture (HER-ART-004) told to the winner's owner, a server's seat with it; CLT-Q4, the
  background variants 0 and 1 (read from the wire since M7 T3b1; recorded once a battle has trees,
  mountains or dead vegetation around it; NOTES, "Battlefields and boats"); `CLT-STUB-4` (a live
  game's battle takes no Auto Combat switch: the computer's choices for the human's side need the
  host's state).
- **With E24 M5** (hot seat): `CLT-STUB-5`, a hot-seat battle between two of the host's humans
  (design §13.4).
- **Waiting on other lanes** (their inboxes, 2026-11-05 and 2026-11-10): CLT-Q63 (a) (the
  automatic war machines' turns, the battle lane), CLT-Q65 (b) (a whole game's learned spells),
  CLT-Q69 (a) (the start split, the battle and server lanes), `CLT-STUB-1` (a server's seat
  lacks the other hero's Orb and Cloak: the battle and server lanes); the client's side follows
  each.
- **The battle screen's unbuilt gaps** (read in part, not recorded; NOTES, "The battle screen"):
  the other pending spells of a melee strike, regeneration, the mana drain, Death Blow and
  Poison's damage, the Faerie Dragon's draw line, Fire Wall's burn and Land Mine's blast, the
  shot cursor's half damage, the "no free hex" message (an engine event first); taken when a
  recording needs one.
- **E28** (the backlog): ranked play in the native client, refined after E27 (the first ranked
  play through the website, E21); NOTES, "The live client". **E29** (the backlog): HD layouts and
  graphics options for both clients; NOTES, "Options and layouts".

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
