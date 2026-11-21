# E27 — archive: M1 (reading and captures), M2 (the start and the menus), M3 (a game's setup) and their review lists

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

## M2 (moved 2026-11-17, M2 T8)

Moved out of `E27-game-shell.md` on 2026-11-17 (M2 T8, M2's closure), verbatim: milestone M2's
task list and its review list, accepted by the user at T8 with the fresh reviewers' changes
(decisions 42, 43): items 2, 10, 15, 23, 39, 43, 57, 60 and 65 changed (fixed at T8, the log's
"M2 T8"), the rest kept, the stale wording corrected in place before the move.

### M2 — the start and the menus (refined 2026-11-13, M2 T0 at M1 T3)

Decisions 20 to 22 bind it: the dialogs' input shared with the battle's (20); the shell's code as
the battle's, `h3-smacker`, the clips' sound exact against the DLLs (21); the menus' buttons
redrawn whole, Enter pressing the names' OK (22). Each screen is exact against its group's shots,
recorded again in the sandbox (`h3shell.py run`), with the sound sequences its task records
(design §6.4). The loading bar is no part of the start (design §2.1): M3's begin builds it.

| Task | Scope | Status |
|---|---|---|
| T1 | The Smacker decoder: `h3-smacker` (decision 5), FFmpeg 6.1.1's `smacker.c` video ported (integer only), the `video` command reading `.smk`; checked frame for frame against `SMACKW32.DLL` under Wine (`smacker-check.sh`, as `bink-check.sh`) on the shell's seven Smacker clips and synthetic samples | **done** 2026-11-13 (decisions 23, 24; CLT-Q119): `h3-smacker` (`Clip`, `Reader`, `Decoder`, `Frame::written`), `video --smacker`, `smackprobe.exe`, `smacker-check.sh`; the shell's eight clips, AB's `Credits.smk` and the samples the DLL's byte for byte, 8-bit and 5-6-5; log "M2 T1", design §7 |
| T2 | The clips' sound (CLT-Q104): Bink's audio in `h3-bink`, Smacker's in `h3-smacker`, the floating point emulated as `h3-bink`'s inverse DCT; checked sample for sample against the DLLs under Wine where the harness reaches them (else against FFmpeg's, a `CLT-Q`); the samples into the window's audio | **done** 2026-11-14 (decisions 25 to 27; CLT-Q120, CLT-Q121): `h3_smacker::sound`, `h3_bink::audio` (Ooura's FFT, `fft`; `trig`), `h3_view::clip_sound`, `HostSound::play_clip`, `video` writing each clip's sound and `--play`; a stand-in Miles, `smackprobe audio`, `binkprobe sound`; every track the DLLs' byte for byte; log "M2 T2", design §8 |
| T2b | The shell's Bink clips' video as `BINKW32.DLL`'s (found at T2, design §8.4): `h3-bink`'s frames of `NWCLOGO`, `H3INTRO`, `LOSEGAME` and `H3X1intr` differ from the DLL's in the chroma (at most 3 levels at the first differences) while the battle's clips and the samples match; find the cause (a block type, a coding path or the DCT on these clips), fix it in `h3-bink`, every frame of the shell's Bink clips exact in `bink-check.sh` | **done** 2026-11-14 (decision 28; CLT-Q122): the decoding was right; the DLL's `BinkCopyToBuffer` copies only the cells its change mask marks, the chroma's misfiled: `h3_bink::Target`, `Decoder`'s mask, the samples' still cells, `Reading::Original` copied; every frame of the fourteen game clips and the samples the DLL's; log "M2 T2b", design §9 |
| T3a | The shell's frame (CLT-Q103, decision 29): the screens' state machine and the menu loop's requests (design §2.2) in `h3-view`, the `h3-view shell` command with a headless driver (a text client first, E17's pattern); the dialogs' input (a DEF button's own loop, a box's and a bar's press and release, the right press's help) moved from `h3_view::options` into `h3-ui`, the combat options on it unchanged (decision 20) | **done** 2026-11-14: `h3_ui::input` (`Input`, `Held`, `Did`), `h3_view::options` on it (its tests unchanged); `h3_view::shell` (`Shell`, `Screen`, `Ended`, `Request`, `Choice`, `Kind`, `PlayEnd`; the switches of §2.2 to §2.4, §4.3, §5.3), `shell::text` (the words), `h3-view shell [--script] [--no-intro]`; log "M2 T3a" |
| T3b | Start-up loading by the LODs' tables (CLT-Q2, decision 29): the archive set read by the tables at start and a batch's entries by their ranges (`h3_formats::lod::LodTable`), across `h3-battle-client`'s readers (`render`, `battle`, `sequences`, `text`, `gate`), `h3-view gate` measuring it (`lanes/client/NOTES.md`, "Assets and archives"); the four LODs staged | **done** 2026-11-14: `h3_battle_client::lods` (`Lod`, `Reader`, `Lods`: the tables from the heads, an entry by its range, a batch archive by archive in the ranges' order), every LOD reader of the crate on it; `h3-view gate`: the start 1.3 ms for the tables alone (1 250 KiB; 145 ms for the 212 MiB read whole before), a battle still's peak memory 243 → 28 MiB; log "M2 T3b" |
| T4 | The comparison: `h3shell.py drawn` (the shell plays a recipe's inputs at its times; its frames compared with the original's shots, as `h3drive.py drawn`); h3shell's sound (h3drive's `sound`); TCP/IP's address masked in its digests (review item 8); the main menu's first shot exact | **done** 2026-11-14 (decision 30): `h3_view::shell::replay` (a recipe's steps on the shell's own clock; `Stop`), `shell::look` and `clients/layouts/shell.toml` (the main menu at rest), `Built`; `h3-view shell --steps --frames` and `--window`; `h3shell.py drawn` (`exact` against the kept lines, `shell/drawn.txt`), `sound`, `mask`; `start-plain`'s and `menu-main`'s first shots exact; `menu-sound` recorded, `mp-tcp` again masked; log "M2 T4" |
| T5a | The start's pictures (design §2.1, decision 31): the clips in order, `H3Intro` framed in `introrim.pcx`, skipping, `Show Intro`, Video Quality's Smacker copies (CLT-Q96); what T2 left unsettled (§6.3: `NWCLogo`'s placement, the end-of-clip call, the screens between the clips) read; exact against `start`; every pixel of a Smacker clip through its palette (decision 24), every Bink clip's frame whole (decision 28; the comparisons through `h3_bink::Target`, what the game's buffer holds before a clip read, design §9) | **done** 2026-11-14 (decision 32; CLT-Q123, CLT-Q124): `h3_view::shell::start` (`Start`, `Picture`, `smack_wait`, `shown_union`), `h3_bink::rects` (`BinkGetRects`) and `copies`, the replay's start, `h3-view shell --video-quality`, `Videos`' shell stand-in; 11 of the `start` group's 12 lines exact (`start-bink`'s 2 519 frames among them), `start-smacker`'s clip 23 frames short of it (T5a2); log "M2 T5a", design §10 |
| T5a2 | The Smacker clips' rectangles (decision 32; design §10): `SMACKW32.DLL`'s `SmackToBufferRect` read (its list, 0x10007410, 0x10007680, the iterator 0x10012760 and the state `SmackDoFrame` fills) and ported into `h3-smacker`, the game's union over them for the comparisons (`shown_union`); `start-smacker`'s clip exact (its 23 frames of `H3INTRO.SMK`); checked against the DLL where the probe reaches it (`smackprobe`) | **done** 2026-11-15: `h3_smacker::Rects` (`SmackToBufferRect`, its filler, list and iterator) and `Frame::spans` (the decoder's map), as the DLL's on every call of every frame (`smackprobe rects`, `smacker-check.sh`); `Picture` shows their union under `Bugs::Original`; `drawn --group start` 12 of 12 exact; log "M2 T5a2", design §10 |
| T5b1 | The start's sound without the game's run (decisions 31, 33): the readings (the 3DO logo's sound path and volume, `game_samples_load`, a Smacker clip's end, the frames with sound), a stand-in DirectSound in the probes, the client's pacing with sound checked against the DLL; the clips' sound at the effects setting through `HostSound::play_clip` in the window | **done** 2026-11-15 (decision 34; CLT-Q125): `h3_view::shell::start::SmackSound`, `Start::with_sound`, `Op::Sound`, `Quiet`, `Closed`, `Replay::op_times`; `clip_sound::volume` the DirectSound path's; `h3-view shell --effects` and the window's sound; `smackprobe wait`, `capture/vsound.h`, `smacker-check.sh`'s `wait` lines: the six game clips with a track as the DLL's at 1, 5, 7, 10 and 16 ms steps; `game_init` read; log "M2 T5b1", design §11 |
| T5b2 | The start's sound on the game's run (decision 33; design §11.7): the stand-in DirectSound in `xdd.dll` for `SMACKW32` (its `timeSetEvent` and Miles' timer kept from firing), Bink's Miles sample calls on the harness's clock, each clip sound's Play, stop and volume logged; `binkprobe` on the same Miles stand-in; the `start` group's sound recipes recorded and compared (`drawn`: the replay's `sound`, `quiet` and `closed` lines; the frames exact) | **done** 2026-11-15: `xdd.dll`'s 7. (`clock/clipsound`, the ack's fields 14 and 15), `binkprobe start` and the stand-in Miles' `MssMinimum`, `MssHold`, `MssLoads`; `h3shell.py`'s voices, the replay's volume and `closed` lines; Bink's sound from Miles' first buffer read (its first frame), `bink-check.sh`'s `start` lines; four sound recipes recorded, `drawn --group start` 10 recipes, 30 lines exact; log "M2 T5b2", design §12 |
| T6a | The main menu and Quit's question (§2.3, §2.5; decision 35): hover, press, release off, the help popups, hotkeys, `button.wav`, `MAINMENU.MP3`; exact against `menu-main`, `menu-quit` and `menu-sound` up to New Game's menu, its sample compared | **done** 2026-11-15 (decision 36; CLT-Q126): `h3_view::shell::menu` (`MainMenu`, `Boxes`, `Did`), `shell::keys` (the table `[shell]`), `shell::samples` (category 3's handles as the harness numbers them); the replay's main menu and `with_sound`, `look` with the menu's state; the window's mouse, hotkeys, samples and music; `h3shell.py drawn` compares the samples; `menu-main` 26 of 26, `menu-quit` 7 of 7, `menu-sound`'s shot and `sound1` exact; log "M2 T6a", design §13 |
| T6b1 | New Game's and Load Game's menus (§2.4; decision 37): their hover, press, release off, popups, hotkeys and Back, the buttons redrawn whole (CLT-Q118); exact against `menu-new`, `menu-load` and `menu-sound` whole (`sound2`) | **done** 2026-11-15: `h3_view::shell::menu::Menu` for both kinds (`Kind`, `Menus`, `Stale`, `outside`; `MainMenu` gone), the keys' `Command::Choice` and `Context::GameType`, `look::MenuLayout` (`redrawn`, `titles`, `dialog_for`) and `Look`'s `stale` and `over`; the replay and the window on whichever menu shows; `menu-new` 27 of 27, `menu-load` 27 of 27, `menu-sound` 5 of 5 exact at the first run; log "M2 T6b1", design §14 |
| T6b2 | The stubs (decision 37): New Game's Campaign and Tutorial (decisions 7, 9), the main menu's High Scores and Credits until M4, each a message box in the client's own words, back to its menu; the client's language pack (`clients/text/en.json`, E17 design §9.2) and its loading | **done** 2026-11-16: `Built::Stub`; `shell::menu`'s `STUBS`, `Menu::screen`, the notice over the menu (`Menus::notices`, opened by the stub's screen), its OK the screen's end; the replay stops at a stub; `clients/text/en.json` (`shell.stub.*`), `text::own_pack` (`OWN_PACK`, `Pack::lacks`), the catalogue's order (the player's, the client's, the original's) in `log::catalogue` and `h3-view text`, the gate's check; every recipe as recorded, the four notices checked in the window; log "M2 T6b2", design §15 |
| T7 | Multiplayer's first stage (decision 10's stub, CLT-Q101) and the hot seat's names (§2.6, §2.7: the edit field read first, two 21-letter names recorded, Enter pressing OK, CLT-Q107), Load Game's multiplayer; the scenario screen a stub (M3); exact against `multiplayer` | **done** 2026-11-16 (decisions 38 to 40; CLT-Q127, CLT-Q128; CLT-STUB-6): `h3_ui::edit` (the edit's keys by scan code through the original's US table), `Draw::Edit`, `Draw::Fill`, `h3_render::text::draw_text_caret`, `h3_sdl::Window::scans`; `h3_view::shell::multiplayer` (`Multiplayer`, `HotSeat`, `Texts`), `menu::Front`, `menu::Hovers` (decision 40: the original's profile only), the scenario screen's stub, `Stop::Stub`; `edit-keys` and `hotseat-long` recorded (20 letters a name, never joined); the `multiplayer` group's 113 drawn shots exact; log "M2 T7", design §16 |
| T7b | The edits' typing as the keyboard prints (decision 39, CLT-Q127): SDL's text input in the window (`h3_sdl`: started while an edit has the focus, its text events), the characters kept to the font's code page; the edits' keys (Home, End, Left, Right, Delete, BackSpace; the hot seat's Tab, Shift and Tab, Up, Down, Enter) as commands of the shell's key table (Shift chords in `h3_ui::keys`: a `Shift+` chord now parses, a change for the map's table, its inbox told; the window's keys for Tab, BackSpace, Delete and the Shifts); the replay typing characters in the client's profile, the original's scan codes in its own; `drawn` as recorded, the window checked with a layout other than the US one | **done** 2026-11-16: `h3_ui::edit` (`Caret`, `Edit::caret_key`, `Edit::typed`, `typable`: Windows-1252's printable characters but the braces, counted in characters), `h3_ui::keys` (`Chord::shift`, `Shift+`, `Keys::chord_command`, `SHIFT`); `shell::keys` (`Context::Edit`, `Command::Edit`, `Command::Ring`, `command_for`, `shifted`), `Multiplayer::typing`, `contexts`, `typed`, `Front`'s, the replay's typed characters; `h3_sdl` (`Window::set_text_input`, `Window::typed`, `Key::Tab`, `BackSpace`, `Delete`, `LeftShift`, `RightShift`), the shell window's text input; `drawn` as recorded; the window typed under a German layout; log "M2 T7b", design §17 |
| T8 | M2's closure: a fresh reviewer, the review list, M3's refinement (M3 T0: the loading bar, the scenario list's maps, decision 8; §6.4's recipes for M3) | **done** 2026-11-17 (decisions 41 to 44; CLT-Q129 to CLT-Q136): two fresh reviewers over the 71 items (62 kept, 9 changed, stale wording corrected), their 17 findings fixed (part A: `Frame::copy_16`, `start::key_skips`, `message::Open` on `h3_ui::input`, `smack_wait`'s profile, `Start::keeps_input`; part B: the shell window's key file and music, `KeyScan` without `code`, the replay under the network notice, `Command::asked_together`, `Look::over`, `Did`'s constructors, the glyphs measured), 88 of the 91 recipes played again on the new `xdd.dll` as kept (`start-bink` in one of its two runs, the two full Smacker starts not played), `drawn` 91 recipes all exact; M3 refined (its table below); log "M2 T8" |

**M2's review list** (the agent's calls, *provisional*, for T8): (1) `h3-smacker`'s shape (T1):
the frames read in order by a `Reader` (the palette is cumulative), a frame's written blocks in
`Frame::written`, the trees walked bit by bit (H3INTRO's 2 426 frames in 4 s in release); (2) the
DLL's 16-bit copy kept in `h3_smacker::check` (`original_16`, the comparisons' only, decision 24)
rather than in `h3-view`; (3) the `video --smacker` outputs `<stem>.pal8` (indices, then the
palette) and `<stem>.565`; (4) the samples left within what the DLL reads as FFmpeg does (both
byte trees, no key frame mark: design §7.3), `MAX_WIDTH` × `MAX_HEIGHT` 800 × 600 as `h3-bink`'s. T2: (5) the
stand-in Miles (`capture/mss32.c`) answering preference 15 with 1, the double buffers' path; (6)
Bink's sound checked through the DLL's decoder (`binkprobe sound`), not its playback; (7) a clip's
whole track handed to the host at once (`HostSound::play_clip`), the DLLs' rings not modelled; (8)
an 8-bit Bink track's samples given with the low byte 0 (`ClipSound::bink`); (9) `h3_bink::fft`'s
functions generated from the DLL's reading (kept as generated: the DLL's order of operations).
T3a: (10) `h3_ui::input`'s shape: an `Input` beside the dialog's `Shown` (passed in with its
`Dialog`), the dialog's own effects left to it (`Did::pressed` names a picture's press for its
sample, `Did::clicked` the click; a button drawn up before its click is reported);
`h3_view::message::Open` and `stats::Open` keep their own loops (buttons and keys only) until a
task touches them; (11) the shell's screens one enum, its endings one enum checked against the
screen (`Refused`, the shell unchanged), the original's ids the discriminants of `Request` and
`Choice`; the lost and won screens and the High Scores after a won one in the frame; play's
request of High Scores or Credits refused (no writer of them); `Show Intro` the only start
setting; (12) the driver's words and the screens' names (`shell::text`), a script's refused word
exit 1, each screen naming the task that builds it. T3b: (13) the LOD set's shape
(`h3_battle_client::lods`): paths and tables held, no file handle kept (a file opened per lookup
and per archive of a batch, as `videos::Videos`); each head read up to `lod::MAX_TABLE_BYTES`
(312 KiB an archive, as `h3-formats` advises) rather than the used table alone; an asset a
checkpoint's batch lacks still loaded as it is drawn (`Battle::compose`, the cursors' DEFs), now a
read of its range; `sprite` needs all four archives; (14) `h3_assets::archive::inventory` takes a
`LodTable` (the common crate's one caller the gate). T4: (15) the replay's shape
(`h3_view::shell::replay`): the harness's step words read in Rust, the masks left to Python; the
probes' names mapped to the shell's screens (`replay::shown`), a name it lacks, a shot, a press,
a release or a key on a screen not `Built::Whole` (a stub's too since T6b2; `Built::AtRest` gone
at T6a, item 41) and the harness's own steps stop the replay (no shot drawn from a screen the
recipe did not reach); (16) the shell's layout
(`clients/layouts/shell.toml`): each built screen its background copied whole and an `h3_ui`
dialog, the cursor `crdeflt.def`'s frame 0 with its hot spot at the corner; (17) `drawn`'s two
comparisons: `exact` against the original's kept line (pixels and time), regressions against
the viewer's own record `shell/drawn.txt` (`CHANGED`, `new`, `GONE` fail), frames as raw
`.565`; (18) the window: SDL's hardware cursor when it makes one, the screens not built black,
the words from standard input, no key of its own; (19) TCP/IP's mask (180, 96) to (620, 124),
the dialog's width; (20) a sound recipe's samples kept as lines (`sound<k> t<ms> ch<c>
<length>ms x<loops>`), their names unknown (`xdd.dll` logs none). T5a: (21) the start's shape
(`h3_view::shell::start`): a timing (`Start`: the game's readings of its clock give `Op`s) and a
picture (`Picture`: the back buffer and the screen, the decoders streamed, `Source` borrowing the
caller's parsed clips), the replay keeping the ops' log for its caller (`Replay::ops`); (22) the
start's clips read from the video archives and `introrim.pcx` from the LODs at the command's
start, a clip missing or unreadable ending the sequence where it would play, as the original's;
`Videos` taking `shell-videos.vid` after `battle-videos.vid` (no `VIDEO.VID`); (23) the window's
start on `win.ticks_ns()` from the window's opening, a key, a left or a right press skipping
(the right one `h3_sdl`'s `Key::RightButton`; F4 too, unlike the original and the replay: T8's
review); (24) `--video-quality high|low` for the setting Bink Video, `drawn`
passing `low` for a recipe's `Bink Video = 0`; (25) a release with nothing held passes on the main
menu (`start-skip-right`'s right release; at rest until T6a, its menu's input since); (26) DirectDraw's stretch of
`NWCLogo` as Wine's nearest pixel in 16.16 fixed point (the captures' only reference). T5a2:
(27) `h3_smacker::Rects` the handle's fields as the DLL keeps them, one call a `next`, the
decoder's map in `Frame::spans` (kept from frame to frame); only the game's buffer types (not 1
or 2) and no interlaced or doubled clip ported; the DLL's faulting third join stopped instead;
(28) `smacker-check.sh`'s rectangles at two places, (0, 0) and (83, 187), compared line by line
(`<stem>.rects`, `<stem>.shifted.rects`, `smackprobe rects`' text); (29) `Picture` keeping a
`Rects` for each part (a Bink clip's unused). T5b1: (30) `SmackSound` the DLL's service, position
and wait on an ideal device (the cursor moves with the clock, no lead, whole samples; the write
cursor the play cursor), the ring, its refills and its underrun not modelled (no game clip runs
the buffer dry; the samples' wait lines left uncompared); (31) the probe's stand-in DirectSound
(`vsound.h`) given straight to `SmackSoundUseDirectSound`, not through a Miles stand-in, its sound
timer made and never fired; (32) a Bink clip's sound from its first frame (read at T5b2: Miles'
first buffer loads there, item 37); (33) `--effects` (the window's 5, the replay's 0), the window's tracks decoded on
threads at its start and taken at each clip's `Op::Sound`; with no sound device the clips play
silent and their Smacker frames keep `SmackWait`'s clock (the game opens no track without a
driver). T5b2: (34) `xdd.dll`'s Smacker device the probe's (`vsound.h`, an ideal device), handed
straight to `SmackSoundUseDirectSound` in place of `SmackSoundUseMSS`, so the DLL never loads
Miles' functions (its timer `timeSetEvent`'s, never fired; no `AIL_lock` around its calls: the
main thread alone makes them); (35) Bink's Miles sample on the clock as two buffers played back
to back from the first load (no device latency, no underrun's silence), the real Miles never
given a buffer; (36) a clip's sound compared as its voice (the codec, its start and end, its
volume), not its bytes (the tracks are the DLLs' since T2, the Smacker device's cursor since
T5b1); (37) Bink's first load in the first frame checked against the harness's Miles buffer
size alone (36 864 bytes), the client keeping the first frame. T6a: (38) `MainMenu`'s shape
(`h3_view::shell::menu`; `Menu` since T6b1, item 45): the boxes laid out by the caller once
(`Boxes`: their texts fixed at the start), what is open over the menu one enum (none, a help
popup, the question; a stub's notice since T6b2), `Did`'s three
fields (drawn again, the sample, the end); the hover taken again at a hold's end and at the
question's close (recorded, not read); a left press or release closing a help popup without
reaching the menu; (39) the shell's key table `[shell]` (`h3_ui::keys::VIEWERS` gains it): the
main menu's hotkeys and the box's OK and Cancel, no window keys of its own yet (their Escape is
Quit's here); the window takes the defaults alone, no user's key file read (`Keys::original`;
T8's review); (40) `samples::Samples`: category 3's eight handles, the next in turn's start and
step *inferred*, the effects' stop by a menu's music not modelled (a request for the track
playing is dropped), the press sample's length given by the adapter (`Replay::with_sound`); (41)
the replay's main menu (`Menus` and a `Front` since T6b1, T7) kept for the run and opened anew
each time it shows; `Built::AtRest` and
`Stop::AtRest` gone (a screen not built takes the pointer's moves only); (42) the shell's layout
gains `player_palette` and `[message]` (the battle's literals), the boxes framed in red
(`menu::BOX_PLAYER`); (43) the window: `MAINMENU.MP3` decoded on a thread, asked for each time the
main menu shows (each menu since T6b1, item 50), at the first run's music setting (no `--music` flag); each press's sample on a
stream of its own, removed once played; the host's sound opened whenever the effects are above
0 (with the intro only before); the hardware cursor hidden under a help popup; (44) `drawn`
compares a sound recipe's samples (`sound<k>`) with the viewer's `sample` lines. T6b1: (45) one
`Menu` for the main menu and the game type menus (`menu::Kind`; `Menus` the three as they open,
by screen; the stubs' notices and the Multiplayer dialog too since T6b2, T7), `Did::ended` the shell's `Ended`, `Boxes::question` an option (a main menu without it
quits at once, as a lobby's start); (46) the hover by `Dialog::item_at` (the title picture an item
too, as `dialog_item_id_at`), the five buttons' hover cleared by the kind's list; (47) CLT-Q118's
rows as layers: each button's state at its last whole show kept, the rows outside the layout's
`redrawn` rectangles drawn from it over the background by the adapter (`Look::stale`, between the
dialog and the box, `Look::over`), the button's box its layout rectangle; (48) the layout's
`[game_type]` (its items in the creation order, `redrawn`, `titles` by game; the main menu's
`redrawn` too), `MenuLayout::dialog_for` setting the title; (49) the keys' `Command::Choice`
(`single`, `multiplayer`, `campaign`, `tutorial`, `back` in `[shell]`) in `Context::GameType`;
(50) the window opens a menu at each screen change (a word or a menu's end) and asks for
"MainMenu" at each, the host dropping a request for the track playing. T6b2: (51) the stubs as
screens of their own (`Built::Stub`), each the menu it was chosen from opened anew under its
notice (`Menu::screen`, the box `Over::Notice`, `STUBS`, `Menus::notices`), not a box the menu
opens over itself (the shell's frame keeps the original's switches; the hover under the notice
lost); (52) the replay stops at a stub as at a screen not built (no menu opened on it); (53) the
notices' words (a title in the highlight braces, one sentence) and their keys by the driver's
screen names (`shell.stub.<screen>`); (54) the client's own pack beside the key catalogue
(`text::OWN_PACK`, English only), every key without an original form required in it
(`Pack::lacks`: `<key>` or a plural key's `<key>_other`), a fault an error at a client's start
and in the gate; (55) no player's pack in the windows yet (only `h3-view text --pack`). T7: (56) the
edit field in `h3-ui` (`h3_ui::edit`: `Edit`, its keys by scan code through the original's US
table, `scan_of` its inverse for the harness's typing), drawn as `Draw::Edit` over the dialog's
label of its box (`Edit::place`), not an item kind of its own (`Item` is matched whole in the map
client's crate); (57) the window's keys as the original reads them a side channel of
`h3_sdl::Window` (`scans`, one an event of a wait; `KeyScan`, `set1`), not a field of
`Event::Key` (matched whole in the map client's window; since T7b the window, always the
client's profile, reads only its Shift: T8's review); (58) the status lines' fill a
`Draw::Fill` before their label; (59) `menu::Front` (a menu or the Multiplayer dialog) the one
thing the replay and the window give the inputs to; the dialog holding the game type menu as it
ended (`Multiplayer::under`), the hot seat a part of it, the probe `hotseat_names` by it; (60) the
network modes' notice and the scenario screen's stub in the client's words
(`shell.stub.network`, `shell.stub.scenario`), the replay stopping under the notice
(`Stop::Stub`), the scenario's stub over the game type menu it is reached from (six `STUBS`, one
key); (61) the hot seat's names kept in the dialog after its OK (`Multiplayer::names`) for M3; the
dialog's name not kept as a setting (no `Network Default Name` yet), empty at each opening; (62)
the Multiplayer dialog's shadow in its layout (recorded; its constructor's flags are 0, how it
comes not read); (63) the hover taken again after a hold's end in both dialogs (recorded, as the
menus'); Enter with OK disabled moving on as the original's (decision 22 names an enabled OK);
(64) the menus' last items carried from menu to menu (`Hovers`), read by the original's profile
alone (decision 40); (65) the edits' own keys in a context of their own (`Context::Edit`, asked
before the dialog's: `Front::contexts`, `keys::command_for`), named `edit-*`, the ring
`hot-seat-next` and `hot-seat-previous` (T7b); (66) a Shift chord bound nowhere gives the key
alone (Shift and a hotkey still press its button); Shift chords in `h3_ui::keys` with `Shift+`
after `Ctrl+`, the chord parser's error text unchanged (the map client's tests read it); the
battle's and the map's windows read no Shift, so a `Shift+` chord in their tables parses but never
matches (the map-client lane's inbox told); (67)
`hot-seat-ok` with OK disabled moves on, whatever key it is bound to; (68) the typed text kept to
Windows-1252's printable characters but the braces, the text, caret and room counted in
characters; (69) a character's key down still reaching the key table while an edit types; (70)
the window's text beside its events (`Window::typed`, each character with the number of events
before it) rather than an `Event` of its own (the map client matches `Event` exhaustively), a
repeat's text left out with its key down; (71) the keypad's navigation keys unbound.

## M3 (moved 2026-11-19, M3 T7)

Moved out of `E27-game-shell.md` on 2026-11-19 (M3 T7, M3's closure), verbatim: milestone M3's
task list and its review list, accepted by the user at T7 with the fresh reviewers' changes
(decisions 67, 68): items 1, 4 (into 12), 9, 13, 16, 20, 21 and 22 reworded in place before the
move, items 23 and 24 and the "Not built" note added (fixed at T7, the log's "M3 T7", design §33),
the rest kept.

### M3 — a game's setup (refined 2026-11-16, M3 T0 at M2 T8)

Decisions 4, 6 and 44 bind it: the scenario screen fills the engine's `GameSetup` through the
engine's own functions (4); what the engine lacks is drawn and chosen, its effect a `CLT-STUB`
(6); every shipped map listed, each read whole, Begin's hand-over a notice until E24 M5, Options
M4's (44). Each screen is exact against its group's shots (`scenario`, `advanced`; recorded again
at M2 T8), with the sound sequences its task records; each of design §6.4's recipe fixes is
recorded by the task that builds its screen (`h3shell.py run --only <id> --record`). The
catalogue: design §3; the shell's frame, menus, edits and the hot seat's names: M2's
(`E27-archive.md`).

| Task | Scope | Status |
|---|---|---|
| T1a | The scenario screen's frame and its information panel (§3.1, §3.2, §3.5's drawing, §3.8; decision 45): the new-game mode of a single scenario, the list hidden; both backgrounds; the kept map's name, description, victory and loss, the flags, the size, the map's difficulty and the rating; the panel buttons (caption buttons), Begin and Back, their press, hold and popups; Back and Escape; the maps read whole; exact against `scn-new` but `list-shown`; M2's leftover (found met: `Key::Other`) | **done** 2026-11-17: `h3_view::shell::scenario` (`Scenario`, `Entry`, `Texts`, `sort`), `Front::Scenario`, `Menus::scenario`, `menu::Arrow` (CLT-Q137 (b), decision 46: the original's arrow for the comparisons only), `look::ScenarioLayout`, the replay's `with_backgrounds` (a recipe's `backgrounds`); `h3_ui::ui::Caption`; `h3-battle-client`'s `maps` and the shell's `--backgrounds`; the layout's `[scenario]`, `[adventure_cursor]`; the keys `shell.scenario.*`; `scn-new` 30 of 31 exact; log "M3 T1a", design §18 |
| T1b1 | The list (§3.3, §3.4; decision 47): its panel and the toggle (128), the rows and columns, the sort, the size filters, the list's keys and scroll bar, a row's press and double press; exact against `scn-new`'s `list-shown`, `scn-list-keys`, `scn-sort`, `scn-filter` (§6.4: `up-past-top` with Page Up first) | **done** 2026-11-17: `h3_ui::scrollbar` (`H3DlgScrollbar` read whole); the screen's list (`ListLook`, `sort_by`, `list_names`, `Scenario::clock`, `Front::clock`), the keys `Command::List`, the replay's Prior and Next, the viewer's `--maps`; CLT-Q138 (b), decision 48 (the original's partial redraw, for the comparisons); `scn-list-keys` recorded again; 59 of 67 exact, the 8 others the ties' order (decision 12); log "M3 T1b1", design §19 |
| T1b2 | The list's right press (§3.8; decision 47): the popups on the columns, the filters, the bar, a row (nothing), the labels and the texts, the flags' strip's teams dialog (387, 0x576540); GENRLTXT 503 in a shot (design §19.6); exact against `scn-right` | **done** 2026-11-17 (decision 49: whole): `HELPED` (the labels, the filters, the columns, the bar `BAR`), the teams' dialog (`TEAMS`, `TeamsLook`, `Teams`, the layout's `[scenario.teams]`, `h3_view::message::frame` public); `scn-right` 30 of 30 exact; GENRLTXT 503 found a network game's alone (design §20.3: to `CLT-STUB-6`, no shot); log "M3 T1b2", design §20 |
| T2a | The difficulty (§3.5; decision 50): the presses (107 to 111) and the rating, the opening's difficulty, the victory's text of goals 0, 4, 5 and 10; exact against `scn-difficulty` (`reopened` added), `scn-info` | **done** 2026-11-17: `OPENING_DIFFICULTY` (Normal at each opening: the record's +0x33C), `choose`, `maps::STANDARD_ALWAYS` (`victory_standard_set` 0x4BF780); `scn-difficulty` 9 of 9, `scn-info` 21 of 21 exact; log "M3 T2a, T2b", design §21.1 |
| T2b | The empty list and Begin's messages (§3.5, §3.6; decision 50): the empty list's panel, its teams' dialog, a difficulty's press over it, GENRLTXT 530; exact against `scn-begin-none` (its strip and a press added), `scn-begin-filter` up to its load, `scn-empty-filter` (new) | **done** 2026-11-17 (CLT-Q139 (b), decision 51): `show_empty`, `begin`, `Over::Notice` (the box's default arrow), `Texts::no_maps`, the key `shell.scenario.no-maps`; `scn-begin-none` 7 of 7, `scn-begin-filter` 6 of 6 up to its load, `scn-empty-filter` 5 of 5 exact; log "M3 T2a, T2b", design §21.2, §21.3 |
| T2c | The description's scroll bar (§3.5; design §18.1, §21.5; decision 50): `H3DlgScrollableText`'s path for a text longer than its box (0x5BA19E) read and built; no shipped map reaches it, so a map of the harness's own with a longer description (made from a shipped one) and a recipe of it; exact against that recipe | **done** 2026-11-17: `scenario::description` (`Description`, `DescriptionLook`, `[scenario.description]`), `h3_render::text::split_lines` (0x4B5B90), `h3_ui::scrollbar`'s keys (0x596230), `Scroll::Left`/`Right`; the kept map carried (`Menus::front`); the harness's `made` maps; `scn-long-text` 36 of 36 exact; CLT-Q140 asked; log "M3 T2c", design §22 |
| T3a | The advanced options' panel and rows, the town's and the bonus's arrows and popups (§3.12 to §3.15; decision 53: T3 cut in three by recipe): the panel, the headers, the duration's text and bar drawn; a row per colour that plays; the records (`GameSetup`'s choices at T6: `PlayerStart.choice`), the bonus a `CLT-STUB`; exact against `adv-single-town` | **done** 2026-11-18: `scenario::advanced` (`Advanced`, `AdvancedLook`, `AdvancedTexts`, `Record`, `Bonus`), `Scenario::records`, the layout's `[scenario.advanced]`, the seats' factions and flags, `h3_ui::scrollbar`'s horizontal draw, `CLT-STUB-9`; CLT-Q140 (b) built (decision 52); `adv-single-town` 27 of 27 exact, `adv-single-hero` and `adv-single-maps` exact but the hero's box; log "M3 T3a", design §23 |
| T3b1 | The hero's arrows and a hero's portrait and name (§3.14, §3.15; decision 54): the engine's offer from each map's import, the pick into the records, the heroes' small portraits, the main heroes' and the map's names, Random's popup, CLT-Q112's rows; exact against `adv-single-hero`, `adv-single-maps` but a hero's popup | **done** 2026-11-18: `scenario::Heroes`, `CustomHero`, `Entry::heroes`, `MainHero::portrait` (CLT-Q141 (b), decision 55), the options' hero list, step, show and name (CLT-Q142 (b), decision 56), Random's popup; `h3-battle-client`'s `maps::rules`, `maps::heroes` (`h3-files`, `h3-game` dependencies); the asset tables' `small_portrait` (the harvest's +0x30); `adv-single-hero` 26 of 28, `adv-single-maps` 11 of 13 exact (the four hero popups T3b2's); log "M3 T3b1", design §24 |
| T3b2 | A hero's own popup (§3.15, design §24.3: `scenario_hero_popup` 0x575A90 read whole: the name, the portrait, the class and level, GENRLTXT 78 and 79's headings, the specialty's picture and text); exact against `adv-single-hero`, `adv-single-maps` | **done** 2026-11-18: `advanced::own_hero_popup`, `HeroPopupLook` (`[scenario.advanced.hero_popup]`: 320 × 256 at (250, 172)), `AdvancedTexts`' `hero_popup`, `hero_line`, `hero_classes`, `specialties`; the keys `shell.advanced.hero-popup.*`, the family `hero-specialty`, `hero-class` listed by the battle client; the classes from the game's rules; `adv-single-hero` 28 of 28, `adv-single-maps` 13 of 13 exact; the harness without `TMPDIR` for Wine; log "M3 T3b2", design §25 |
| T3c1 | The flags' hover and press (single: `scenario_flag_click` 0x58B790, CLT-Q93), the handicap (single: shown, no press), the name's edit (353, `scenario_name_click` 0x58B620) (§3.13, §3.14; decision 57; the handicap a `CLT-STUB`, decision 6); exact against `adv-single-rest`'s first ten shots (`flag-hover-own` to `name-after-flag`) | **done** 2026-11-18: the flags' hover and press (`Advanced::hover`, `flag_click`; the screen's flags from the moved colour, `Scenario::team_flags`), the name's edit (`EDIT`, the layout's `edit`, `name_click`, its keys by scan code and by the key table and text input, `Scenario::typing`, `contexts`, `typed`), the handicap drawn up (`CLT-STUB-10`); the original's strip after a flag's press and the edit's piece as stale rectangles (CLT-Q143 (b), decision 58); `adv-single-rest`'s first twelve shots exact; log "M3 T3c1", design §26 |
| T3c2 | The duration's bar's input and texts (`scenario_duration_scrolled` 0x57C7F0), the items' hints (`HELPED`) (§3.13 to §3.15; decision 57; the timer a `CLT-STUB`, decision 6); exact against the rest of `adv-single-rest` (§6.4: `popup-hero-arrow` recorded again on row 2's arrow) | **done** 2026-11-18: `h3_ui::scrollbar`'s horizontal input (the press's thumb 16 wide), the duration's bar (`Advanced::bar_press`, `bar_moved`, `bar_release`, `scrolled`; no page: the layout's `step` 0), `Scenario::duration`, `CLT-STUB-11` (no timer); `HELPED`'s headers, label and bar, `COLOUR_HELPED`, the right press past an item not enabled; the keys `shell.scenario.help.*`; `popup-hero-arrow` after the row's town (its hero's arrows hide under Random); `adv-single-rest` 39 of 39 exact; log "M3 T3c2", design §27 |
| T4a | The hot seat's screen and advanced options as they open (§3.12 to §3.14; decision 59): the entry from the Multiplayer dialog's names (`Multiplayer::names`, M2's item 61), two to eight human records, the list of the maps two humans may play, each colour a human may play to the next name, the rows (the names, the handicap a caption button, the arrows by the hot seat's rules), the flags' strip from the first player's colour, Back; exact against `adv-hotseat`, `adv-hotseat-rows`' and `adv-hotseat-two`'s `panel` | **done** 2026-11-18: `Scenario::opened_hot_seat`, `hot_seat`, `Entry::first_human`, `Menus::front`'s hot seat; `Advanced::seated`, its human records (`Human`, the single player's the first), `seat_humans`, `row_items` (the handicap's caption), `takes` by the mode; `Screen::Scenario(New, Multiplayer)` built whole (no longer a stub); the layout's `handicap_caption`, the keys `shell.advanced.handicap.*`; `adv-hotseat`'s `backgrounds`; `adv-hotseat` 8 of 8, both panels exact (also `handicap-3`, `handicap-popup`, `flag-hover-green`; `hotseat-names`' and `hotseat-long`'s screen shots); log "M3 T4a", design §28 |
| T4b | The hot seat's presses (§3.6, §3.14, §3.15; decision 59): the handicap's (+1 modulo 3, `handicap_names`), the flags' cycle (CLT-Q94, decision 13: the client's; the original's `hot_seat_colour_cycle` for the comparisons), a hero another human record holds passed over and stepped on (`scenario_hero_step`, `scenario_heroes_unclash`), the name's press taking nothing, Begin's GENRLTXT 83 and 531; exact against the rest of `adv-hotseat-rows` and `adv-hotseat-two` | **done** 2026-11-18 (CLT-Q144 (b), CLT-Q145 (c): decisions 60, 61): `Advanced::cycle` (the original's, as recorded: the holder's index + 1, the colour pressed last and the kept one), `pass` (CLT-Q94's exchange), `step_handicap` (the client's caption the record's), `held_elsewhere`, `unclash`, `refusal` (`Refusal`), `seating`; the screen's Begin over a map (`Texts::unplaced`, `too_few`; the keys `shell.scenario.unplaced`, `too-few`), the box's close keeping the screen as it was; `adv-hotseat-rows` 12 of 12, `adv-hotseat-two` 7 of 7, `adv-hotseat-begin` 7 of 7 up to its begin exact; log "M3 T4b", design §29 |
| T5a | The random map's tab drawn and its left presses (§3.16; decision 62): the toggle (130) with the list and the options hidden, `RanMapBk.pcx` and items 280 to 335, the titles; every choice, the caps and disabled counts, the levels; the own record in the panel (GENRLTXT 741, 742, the flags), built again by the counts; the size mark (CLT-Q115 (b)); exact against `adv-random` from `tab` to `monsters-random` | **done** 2026-11-19: `scenario::random` (`RandomMap`, `Choices`, `Pressed`), the screen's `toggle_random`, `random_built`, `random_click`, `random_size_mark`, `shown_entry` (the panel, the game and the advanced options read a chosen random map's record), `Texts::random`, `random_showing`, `random_map`; the layout's 103, 280 to 335, 1040, 1041; the keys `shell.scenario.random-*`; the tab's hidden items out of the mouse's dialog; the record built at each opening in the client's profile (CLT-Q111 (b)); CLT-Q146 asked (built as the original's); `adv-random` 45 of 52 exact (the 44 and `show-hover`); log "M3 T5a", design §30 |
| T5b | The rest of the random map's tab (§3.16; decision 62; CLT-Q92): the items' right press and hints, Show Random Maps (335: the random maps' list, kind 1, empty), the tab toggled back after it (CLT-Q111 (b)), the list's showing over a chosen random map (design §30.4), Begin with a random map chosen (a stub's message); CLT-Q146's answer; exact against the rest of `adv-random` (§6.4: Begin after `tab-again`, recorded by this task with `h3shell.py run --only adv-random --record`) | **done** 2026-11-19 (kept whole; CLT-Q146 (b), CLT-Q147 (b): decisions 63, 64): `random::HELPED` (the keys `shell.scenario.help.random-*`), `RandomMap::heights`; the screen's list kind (`random_list`, `toggle_list(random)`), the panel over an empty list (`blank_panel`, `show_with`; `give_to_game` and `select` of none as the original's), `random_built` setting the description and flags, Begin's stub (`Texts::random_stub`, `stubbed`, `CLT-STUB-12`, `shell.stub.random-map`), the replay stopping under it; `adv-random` recorded with Begin (53), `adv-random-list` new (12); `adv-random` 52 of 53 exact (the replay stops at `begin` under the stub), `adv-random-list` 12 of 12; log "M3 T5b", design §31 |
| T6 | The game's begin (§3.17, §2.1; CLT-Q95, CLT-Q135): the setup built and validated by the engine (a refusal shown), the loading bar (`loadbar.pcx`, `loadprog.def`; the client's steps), the begin's notice (a `CLT-STUB`) and back to the menu, the setup kept for E24 M5 (the map-client lane's inbox); exact against `adv-begin` and `adv-hotseat-begin` up to the bar | **done** 2026-11-19 (kept whole: decision 65): Begin, Enter and a row's double press end the screen, `Scenario::begun` (`Begun`, `Advanced::begun`); `h3-battle-client`'s `begin` (`GameSetup` from the import and the records, `validate`, `begin::Game` kept for E24 M5); `h3_view::shell::play` (`Play`, `Front::Play`, `LoadingLook`, the layout's `[loading]`; `Screen::Play` built), `start::faded`; the keys `shell.stub.play`, `play-refused` (`CLT-STUB-13`); both recipes' `bar` clip (new: the full bar fading, 2 frames) exact, their shots up to it as before; log "M3 T6", design §32 |
| T7 | M3's closure: a fresh reviewer, the review list, M4's refinement (M4 T0: the saves' record and its extension, decision 8; Options with the menu in play, CLT-Q136; the load and save modes with §6.4's recipes for them) | **done** 2026-11-19 (kept whole: decision 66): three fresh reviewers (one per area, the shots as contact sheets); the review list accepted, 8 items reworded and 2 added (decision 67); the fixes (design §33.2: a held row or name under a box, the list's keys under a held button, the double press on the map pressed, Begin taking the open edit, CLT-Q93 (b), the begin's guard, a random map's seats and offers); CLT-Q148 to CLT-Q154 (decision 68, §33.3); every recipe played again under Wine (92, all as kept, three after a rerun alone); `adv-random-options` new (9 of 9 exact); M4 refined (decision 69: a new M5 in play, the save the engine's replay extended); log "M3 T7", design §33 |

**M3's review list** (the agent's calls, *provisional*, for T7): (1) the maps read whole
(`h3_map::read`) and, since T3b1, imported for the heroes' offers: the 159 `.h3m` files of `Maps`
(the 160th file is `Tutorial.tut`) in about 0.3 s at the shell's start (T7: the release shell's start 0.42 to 0.57 s with
them, 0.17 to 0.23 s with an empty `Maps`), no
header-only read in `h3-map`; read once, at the shell's start (the original at each opening); (2) T1a: what
the redraw draws itself (the name, the texts and marks) and the original's unnamed items (its
100s) as layout items of the client's own ids (1000 on, no hint or right press); (3) T1a: the
background's draw given to the replay by a recipe's `backgrounds` (the original's under the
harness), the window's from the system's randomness; (4) T1a: the description as one label of
width − 3 while it fits (design §18.1; merged into (12) at T7's review); (5) T1a: the caption buttons as a `caption` of
`h3_ui::ui::Button` (the foundation, additive); (6) T1b1: the list's rows drawn by the screen
from a row template (`[scenario.list]`), the bar a `h3_ui::scrollbar` beside the dialog (not a
dialog item: a new `ui::Item` variant would change the map-client lane's code), drawn after the
items; (7) T1b1: a row's double press timed on the shell's clock given by the window and the
replay (`Front::clock`, the screen's opening its first time); (8) T1b1: the original's partial
redraw reproduced from the items' order and a stale rectangle (design §19.4), not a screen kept
between frames; (9) T1b2: the teams' dialog's items under the client's ids (the original's −1:
the title 1, the names 2 on, the flags 10 on), over an empty list none in the client's profile (decision 51; the original's shows the game's
players: with no map ever shown, one team of the eight colours), the
list's bar's right press tested before the items' (design §20.1, §20.2); (10) T2a: the
header's read's standard victory (`victory_standard_set` 0x4BF780) applied by the client's map
reading (`maps::STANDARD_ALWAYS`), the game's rule left to E10; (11) T2b: the game's map kept as
the last entry shown (`Scenario::game`, a copy) for the original's leftovers over an empty list;
Begin's message a box of the screen's own (`Over::Notice`), its default arrow through `hovers`;
(12) T2c: the description as one label of the lines shown (a highlight closed at a line's end),
the fitting case through the split too (one
label 316 wide; the raw text one label only without the description's font); its bar drawn after the items; the key's step at once
(the original's 60 ms with the arrow pressed not drawn); a description cut at 300 bytes (the record's next
bytes not read) in the original's profile, whole in the client's (CLT-Q151 (b), T7; the name's
60 bytes likewise); the harness's own maps by path in `--maps`. (13) T3a: the
advanced options' items made by the code from a row template (`[scenario.advanced]`, the DEFs'
`{}` the colour's letter) and moved to their rows at each showing; what a row draws itself after
the dialog's items, its items drawn again over its bar (the original's order); the screen handing
the mouse a dialog without the options' hidden items (`mouse_dialog`) rather than the shared input
passing over hidden items; the records kept in the options and handed on by `Scenario::begun`
(`Advanced::begun`: as the next showing would make them; T7's review); the popups' pictures centred by the layout (`picture = [104, 60]`: 48-pixel
pictures); the horizontal scroll bar as read at T3c2 (§27.1; T7's review).
(14) T3b1: the offers computed at the shell's start from each map's import with the repository's
packs (`maps::rules`: base, ab, sod, `base:faithful`; a map that does not import offers none) and
kept in the entry (`Heroes`: 8 × 9 lists), not the setup itself (T6 imports the chosen map again);
the record keeps the hero type, not the original's index; the small portraits in the asset tables
by hero type (`small_portrait`), the campaign heroes' seven portraits not tabled (not drawn).
(15) T3b2: a hero type's class taken from the game's rules (`rules.core().hero_types()`, the
engine's class number into the catalogue's `hero-class`), not the asset tables; the popup's
"%s - %s" the catalogue's format resolved once with U+0001 and U+0002 for the name and the class
(`HERO_NAME`, `HERO_CLASS`), filled at the popup (the screen holds no catalogue); `un44.def`'s frame
the hero type (the original's modulo of its 156 frames not done: no type past them).
(16) T3c1: the name's edit an item of every colour (the original's constructor adds it single only:
the hot seat's T4 leaves it hidden); in the client's profile, while the edit is open, Begin's
command closes it as the original's Enter, Back's is the screen's, every other key the edit's; the
original's piece (CLT-Q143) and the flag press's strip as stale rectangles; the name kept in the
screen, not carried to the next opening (no settings: M4); the screen's flags read the options'
single player's colour while its rows are as made, else the map's; Begin pressed by the mouse
while the edit is open takes the edit's text in the client's profile (the original's not read,
§26.5; T7's review). (17) T3c2: one key a group
for the eight colours' hints (`COLOUR_HELPED`: the first colour's `Help.txt` record; the eight
records one text in both LODs), not one a record; the duration's bar drawn and pressed by the
options beside the dialog's items (as the list's, item (6)); the right press passing over an item
not enabled through `Advanced::takes` (a single player's handicap) rather than a state bit.
(18) T4a: the human records a list in the options (`Advanced`'s `humans`: the single player's the
first, the hot seat's the names'), the mode a flag of the options and of the screen; the hot
seat's list filtered at its opening from the list read at the shell's start (the original reads
the files at each opening); the handicap's caption the item's text, set at the showing where
none is (the press sets it: T4b); the hot seat's records new at each opening (as single's).
(19) T4b: the original's cycle reads the holder's +0x74 as its index (recorded; its writer not
found); its restore keeps the holder's town and hero type (the original's index into the new
colour's list not modelled: no recording reaches it); the client's exchange gives both movers a
Random town and hero; the clash's step built though unreachable; Begin's check before the first
showing reads the colours the showing will give; the client's caption set at each refresh of
the rows (`Advanced::refresh`); Begin's box closed without the screen drawn again whole.
(20) T5a: the tab's items in the layout one by one (`[scenario.dialog]`, not made by code: no
row template); the random map's record an `Entry` the screen builds (`RandomMap::build`), not a
header read; the screen reading it before the selection (`shown_entry`) wherever the original's
`scenario_to_game` and `scenario_entry_name` read +0x38C; the size mark of the client's profile
set from the choice at each press (no record built by a size's press); the tab's hidden items out
of the mouse's dialog (as the options', item (13)); the advanced options' records made again at
their next showing after a build (the original's `scenario_players_init` at once); the record's main hero none (*provisional*), its seats' generate flag (+0xB) set as read
(§30.2, *inferred*: a row shows Random for the hero) and each faction's free heroes offered
(recorded, `adv-random-options`; T7's review).
(21) T5b: the random maps' list a flag of the screen (`random_list`), the client's own random maps
none (no folder read); the panel's header and flags passed apart to `show_with` for the original's
list showing (CLT-Q147) rather than a game record kept apart from the entry shown; the client's
profile over any empty list showing no description (decision 51 extended: the random maps' list
leaves the random map's); Begin's stub over the screen as Begin's other messages (`Over::Notice`),
told from them by a random map chosen; the tab's 56 records one key each; the files' list shown
again gives the game its selection as a select does, the advanced options' rows made again at
their next showing (CLT-Q152 (b); T7's review).
(22) T6: the setup built by the adapter at play's start (`h3-battle-client`'s `begin`: the map read
and imported again, not kept from the list's read) in three steps of the bar, then the bar filled
at once; the map's read or import failing, or the engine's refusal (none on the shipped maps the
recipes play), shown after the fade as
play's notice in the client's words, its OK the main menu (the original has none: its load
failures' message, the tutorial's GENRLTXT 743, leads to its menu); play's stub the main menu
under its notice (`STUBS`), `Screen::Play { restart: false }` built whole and the replay stopping
under the notice (`Play::noticing`, as the random map's Begin); no busy cursor on the loading
screen; the kept game (`begin::Game`) in the window's state alone, printed as play's stub ends;
the replay builds no setup (its frames do not depend on it); a human record on a colour the
import makes the computer's refused by the adapter (the setup's `human` is the engine's only
may-be-human mark; T7's review). (23) T1b1 (added at T7's review): Page Up with the selection
above the top selects nothing in both profiles (the original selects a negative index;
*provisional*). (24) T1b1 (added at T7's review): the list's name order folds ASCII alone
(`caseless`, as `kept_row`; the original's compare not read; *provisional*). Not built (T7's
review, M4's): a file `h3_map::read` refuses is left out of the list (the original lists one of
an unknown format, GENRLTXT 431 and 506, Begin disabled; a map with no players 507; no shipped
file reaches it); the list's held keys' repeats not recorded (M4's load mode records them).

## M4 (moved 2026-11-21, M4 T6)

Moved out of `E27-game-shell.md` on 2026-11-21 (M4 T6, M4's closure), verbatim: milestone M4's
scope, task list and its review list, accepted by the user at T6 with the fresh reviewers'
changes (decision 78): items 7, 10, 19, 20, 38 and 41 changed and 9 and 36 reworded in place
before the move (fixed at T6, the log's "M4 T6", design §44), the rest kept.

### M4 — around play, outside it (refined 2026-11-19, M4 T0 at M3 T7)

Decisions 8 and 69 bind it: the screens around play that need no play (High Scores, Credits, Load
Game), a save the engine's adventure replay with the setup's choices and the client's own table
(CLT-Q156 (a)); what play reaches (the options, Save, Restart, the end of a scenario) is M5's
(CLT-Q155 (a)). Each screen exact against its group's shots, each of design §6.4's recipe fixes
recorded by the task that builds its screen. The readings: design §2.8, §2.9, §3.1 to §3.6 (the
load mode), §3.12 to §3.14 (a load's rows).

| Task | Scope | Status |
|---|---|---|
| T1a | High Scores (§2.8, §34; decision 70): the dialog, both tabs and their buttons and keys, the rows from `CampHigh.txt`'s defaults, the eleven creatures stepping (the adventure map's DEFs by `high_score_creatures`), M2's stub lifted, Reset a client's notice (CLT-STUB-14); exact against `scores-tabs` | done 2026-11-19 (`scores-tabs` 28 of 28 exact; CLT-Q157 (a)) |
| T1b | High Scores' Reset (§2.8, §35): its question (GENRLTXT 667) resetting both tables to the defaults, the client's own scores file (read at the start, written at Reset; never `HiScore.dat`), CLT-STUB-14 lifted; exact against `scores-reset` | done 2026-11-19 (`scores-reset` 10 of 10 exact, `scores-tabs` 28 of 28; CLT-Q158 asked) |
| T2 | Credits (§2.9): `Credits.fnt`, `Credits.txt`'s two records wrapped to 328 px, the scroll of 2 px ten times a second (CLT-Q88), the closing text's moment read from the kept frames, a key (but F4) or a press ending it, M2's stub lifted; exact against `credits-scroll`, `credits-end` | done 2026-11-19 (`credits-scroll` 5 of 5 exact, its scroll's 2 611 frames; `credits-end` 8 of 8; design §36) |
| T3a | The save file's engine part (decisions 69, 71; `cross-lane`): the adventure replay format v20, `Replay::players` (each player's human flag, faction's choice and hero's pick, recorded from the setup, set over the import by `Replay::setup` when verifying, none the import's), its `[[players]]` tables in `h3-files`' file, the 104 golden replays recorded again (only the version and the players' lines moved) | done 2026-11-20 (the adventure lane's inbox: its increment 14 takes v21) |
| T3b | The save file (decisions 69, 71): the client's own file, its table (the names, bonuses, handicaps, the turn's duration, the save's name) beside the engine's replay of T3a, written and read with its version; a load's setup rebuilt (`Replay::setup` over the map's import) and its commands replayed by the engine; tests, no screen | done 2026-11-20 (`h3-battle-client`'s `save`; design §37) |
| T4a | Load Game single's reading and recordings (§38; decision 72): the date's order, the load mode's name, Delete's end read; the original's saves of shipped maps made by maker recipes (the user's are a random map's), a pinned time zone, a held key's repeats (`h3shell.py`); `scn-load` recorded again (§6.4's fixes: a day above 12, three sizes, the held key), `scn-load-last` new (Delete of the last save) | done 2026-11-20 (25 and 9 shots, twice the same; `scn-load-none` as kept) |
| T4b1 | Load Game single's screen (§3.1, §3.3 to §3.6, §38, §39; decision 73): the load mode (title, no panel buttons or flags, Load, the save's difficulty on the disabled buttons and its rating), the client's saves listed (the folder `games`, `<name>.GM<n>`, the map's header from the map the replay names, a save it cannot list as a broken file, the file's local time), the keys and a held key's repeats (the viewer's `key-hold`, the window's), the sorts with CLT-Q138's stale buttons and rating, Load's and a disabled difficulty's popups, a right press without a popup redrawing nothing; the viewer given client saves made as the makers' (`h3-view save`, the makers' `viewer`) and `games-time`; Load and Delete the client's notice (CLT-STUB-15) | done 2026-11-20 (`scn-load` 19 of 25 exact: all but Delete's four, the load's clip and the map) |
| T4b2 | Delete's question (688) both ways and its end (the version sort in the last direction, the first selected, the stale buttons and rating), the notice with no saves (686, Load disabled), Load's end at play's stub (as M3's begin: `save::load`'s `Loaded` kept for E24 M5; a save the load refuses its call), CLT-STUB-15 lifted; exact against `scn-load`'s Delete shots, `scn-load-last`, `scn-load-none` (`scn-load`'s `load` clip and `map` are play's, M5's) | done 2026-11-20 (`scn-load` 23 of 25 exact, `scn-load-last` 9 of 9, `scn-load-none` 5 of 5; design §40; CLT-Q159) |
| T5a | Load Game hot seat's reading and recordings (§41; decision 75): the slots, the names' placing (the save's humans first; CLT-Q160), the begin after a load, the save's extension, the list's filter, the options in a load (the duration's bar and the handicap disabled), Random Map in a load (§6.4; CLT-Q161); two hot-seat makers (`Pair.GM2`, `Four.GM4`: every colour human with its choices), `adv-load` recorded again over them, `adv-load-more` new | done 2026-11-21 (19 and 2 shots, twice the same; CLT-Q160, CLT-Q161 asked) |
| T5b1 | Load Game hot seat's screen (§3.12 to §3.14, §41.4; decision 76): the hot seat's load mode over the Multiplayer dialog (the notice with no saves), the list's filter, the panel's buttons and the flags' strip (no Random Map in the client's profile: CLT-Q161 (b)), the advanced options from the save (the saved choices, no arrows, the handicap and the duration's bar disabled, the names placed: CLT-Q160 (a)); `h3-view save` for a hot seat, the makers' `viewer` tables; exact against `mp-load`, `adv-load`'s screen, options and list shots, `adv-load-more` | done 2026-11-21 (`mp-load` 6 of 6 exact, `adv-load` 14 of 19: all but the flags' two and the three after them, `adv-load-more` 2 of 2; design §42) |
| T5b2 | The hot-seat load's flags (decision 76; the presses that change nothing and the popups exact since T5b1): a flag's press among the save's humans (CLT-Q110 (b)), the name moving onto the colour's saved choices; exact against the rest of `adv-load` | done 2026-11-21 (`adv-load` 19 of 19 exact; design §43) |
| T6a | M4's reviews (decision 77): fresh reviewers by area (High Scores and Credits; the save file and the single load; the hot-seat load), every recipe played again under Wine against `kept/`, the findings and M4's review list to the user | done 2026-11-21 (35 items kept, 7 changed, 28 findings, CLT-Q162 to CLT-Q173; decision 78) |
| T6b | M4's closure (decision 77): the user's answers and their fixes, M4 to the archive, the README's row (`shared/`), M5's refinement once the shell hosts play (else its prerequisites recorded) | done 2026-11-21 (every finding fixed or handed on; design §44; M5's prerequisites recorded) |

**M4's review list** (the agent's calls, *provisional*, for T6a):

1. (T1a) High Scores' layout `[high_scores]`: the creatures' pairs as the executable's numbers,
   the columns by tab, `drawn` (the buttons' canvas for the original's stale edges).
2. (T1a) A press's stale edges (§34.2) taken as CLT-Q118's class (decision 22: the client draws
   the frame whole, the original's profile for the comparisons), not asked again.
3. (T1a) Exit's command (`scores-exit`: Enter and Escape) ends the dialog at its key down whatever
   key the user binds; C and S press their buttons as hotkeys.
4. (T1a) The creatures' DEFs loaded at the start for the tables' rows alone (T1b's file and M5's
   entry load what theirs need); a creature without a template shows none.
5. (T1a) Reset shows the client's notice until T1b (CLT-STUB-14), the replay stopping under it
   (lifted at T1b).
6. (T1b) The scores file (§35.3): `scores.toml` in the working directory (`--scores` names
   another), TOML with a version, both tables of 11 rows; a file the window cannot read stops it
   with the path and the reason, as the settings file; a name longer than the original's 40 bytes
   kept whole.
7. (T1b; changed at T6) A failed write of the scores file told on standard error and, in the
   window, by the original's `file_error_box` (GENRLTXT 11) over the dialog, its OK closing it.
8. (T1b) The close of Reset's question by a key runs the step due as the mouse's release does
   (§35.2; only the mouse recorded).
9. (T1b) The replay takes `press`, `point` and `until-button` on a front's original box alone
   (`Front::question`: High Scores' question, since T4b2 the scenario screen's boxes too), at the
   probes' point by the box's layout, not by a search of the frame.
10. (T2; changed at T6) The credits' clock (§36.4): one step a reading, the first 100 ms after the
    opening; in the original's profile each next 100 ms after the reading that ran the last (the
    original's on the harness's frozen clock), in the client's 100 ms after the last step's due
    time (CLT-Q129 (b)'s pattern: no drift in the window); the window waits 5 ms at a turn while
    they show (High Scores too since T6).
11. (T2) The background's key pixel (§36.3, (463, 553) black during the credits) taken as a
    drawing bug (E17's binding decisions, CLT-Q138's class), not asked: the client's profile
    draws the background whole, the original's for the comparisons only.
12. (T2) `h3_ui`'s two new draws, `Draw::Part` (the text's picture made whole at each draw, 328 ×
    4 774: no cache) and `Draw::Keyed` (the original's profile); a part's source rows past its
    picture not copied (the original reads past it: no caller does).
13. (T2) The text driver names the lost and won screens' task E27 M5 (decision 69's M5).
14. (T3a) A replay records every player's choices from the setup it was recorded from (the
    golden replays' import's included), not only the ones a scenario screen changed: the import a
    verifier is given then decides nothing of them; a replay naming none of them (the server's
    whole-game journals, a file before v20) is the import's.
15. (T3a) A choice of a colour the import has no player of, or of a colour twice, is refused as
    `ReplayError::Incompatible("players")` before the game is built; the game itself checks the
    rest (an unoffered pick, a faction outside the player's).
16. (T3a) The file names a faction's choice by the faction's index or the word `random`, a pick by
    the hero type's index; the server's journals and its measure write none (the server lane's
    inbox: its seats' choices are its to carry).
17. (T3b) The save's file (design §37.1): one file, the engine's replay text whole, then the
    client's table from the line `[save]` on (its version 1; eight `[[save.colours]]`, a name for
    a human's, the bonus by its word); the part before that line is an engine replay file. The
    date not kept (the file's last-write time, as the original's list), nor the map's header (the
    map the replay names, T4's).
18. (T3b) The replay names its map by the file's name in the installation's `Maps`: a save loads
    only over the same map's bytes and import version and under the same rules (the replay's
    header: the packs, the profile, the rules' hashes, the state hash versions), so every engine
    change of the rules or of a hash version leaves the saves before it refused, the field named
    (T4 lists them; the original lists a save of another version as a broken file, §3.3).
19. (T3b) The load applies the commands itself through `h3_game::apply` with `Replay::verify`'s
    checks in its order (the engine's `verify` keeps no state; no engine change: a `Replay`
    function returning the state, the adventure lane's file, would replace the client's loop;
    asked of the adventure lane at T6, the review's change).
20. (T3b) The load refuses a save making a colour human where the map lets none play (the
    import's mark, which the engine cannot check; the begin's rule of M3 T7) and a name for a
    colour no human holds; a save file is read whole up to 64 MiB, a larger one listed broken
    and refused before it is read (changed at T6; the settings and scores files unlimited).
21. (T4a) The load's recipes use saves the original made of shipped maps (three maker recipes,
    design §38.2: Middletown at Hard, Arrogance at Easy, Dawn of War's autosave at Expert on its
    second day) in place of the user's (a random map's, which no client's save can name); the
    makers run when their saves are missing or their recipe changed, no digest of a save kept
    (its bytes differ by a few heap pointers from run to run).
22. (T4a) After a sort in the load mode the partial redraw leaves the difficulty buttons and the
    rating stale too (the rating's text drawn over the last one; design §38.3), taken as
    CLT-Q138's class (decision 48: the client draws whole, the original's for the comparisons),
    not asked again.
23. (T4a) Every shell run's time zone pinned (`tz`, UTC by default; the load's recipes Asia/Kolkata,
    a zone without daylight saving time: the original shows a file's time by the present
    offset); the display's auto-repeat off at each run's start, on only for `key-hold`, whose
    repeats' count follows real time (a recipe holds a key until the list's end, so it does not
    matter).
24. (T4b1) The client's saves (design §39.1): the folder `games` in the working directory (the
    window's `--games` names another), a save's file `<name>.GM<n>` as the original's (the rows
    show the same names), up to 5 000 files of `*.gm?` read in their names' order; the folder read
    again at each opening of the load mode (the window), a replay's once.
25. (T4b1) A save the client cannot list (it does not read, its replay's header the rules
    refuse, its map missing or moved) shows as the original's broken file (GENRLTXT 431 and 505,
    no map's columns, difficulty Easy); the full load (the commands replayed) is Load's, T4b2's.
26. (T4b1) A save's date at the zone's present offset from SDL (`h3_sdl::local_offset`:
    `localtime_r`'s `TZ` on Linux, Windows' zone), as the original's `FileTimeToLocalFileTime`;
    no time-zone crate.
27. (T4b1) A right press without a popup redraws nothing (`dialog_right_click` 0x5FF790 read:
    the redraw follows a popup alone), but on the description's box (its scrolling text,
    `scn-long-text` right-bar) and on the players' items 362 to 385 (their own cases redraw);
    M3's "redrawn whole" for the others was an inference no recording told apart.
28. (T4b1) The shell's window takes a held key's repeats on the scenario screen alone (the
    original queues every key down everywhere; the other screens' keys were not recorded held);
    the viewer gives `key-hold`'s presses by Xvfb's rate (1 under 660 ms, then one each 40 ms).
29. (T4b1) The autosave known by its file's name: the panel's name GENRLTXT 77 where the file
    begins with that text, the name's sort's lead where it begins `AUTOSAVE.`, both any case (the
    original's compares' case not read).
30. (T4b1) The made saves (`h3-view save`, a maker's `viewer` table, left out of its digest): the
    single player's colour the map's, the advanced options untouched, seed 1, every player's End
    Turn until the day; the computers make no moves (the list shows no more of them).
31. (T4b2) Load's end: the save read and loaded by the engine at play's start (`save::load`, the
    loading screen's bar stepped by the read and the load), the game kept for E24 M5; play's
    notice in the client's words (`shell.stub.loaded`, or `shell.stub.load-refused` with the error
    on standard error) where the original plays. A save the list shows broken is loaded and
    refused there too (the original's load of a broken file not read).
32. (T4b2) Delete deletes a listed file of the client's own folder alone (`saves::delete`: a plain
    name of the pattern, a file); a failure is told on standard error and the entry leaves the
    list all the same, as the original's unchecked `DeleteFileA` (CLT-Q159).
33. (T4b2) A disabled button takes no hotkey on the scenario screen (recorded for Load: `scn-load-last`
    and `scn-load-none` enter; the other screens' disabled buttons not recorded).
34. (T4b2) The viewer's replay keeps the saves Delete left for the next opening (the original reads
    its folder again; the window lists its folder anew at each opening).
35. (T5a) The hot-seat load's saves made by the original from shipped maps with every colour
    human and its town, hero and bonus chosen by the arrows (`Pair.GM2`, `Four.GM4`), so the
    rows hold no random draw a client's save cannot repeat; a single game's save under two names
    (`adv-load-more`) with its rows' drawn cells masked instead.
36. (T5a; lifted at T5b1 by item 40) The hot-seat makers' `viewer` tables give the difficulty
    alone until T5b's `h3-view save` makes a hot seat's (`drawn` stops at the hot seat's load,
    before any shot, meanwhile).
37. (T5b1) A hot-seat load's row reads its colour's saved choices whoever holds it: a human record
    seated on a colour takes that colour's record (the original's rows read the game's player of
    the colour, inferred from `adv-load` and `adv-load-more`).
38. (T5b1; changed at T6) A Random choice of town or hero (a computer's or a human's) shows the
    game's draw, as the original's rows: the client's save keeps each colour's drawn town and
    first hero beside its choices (its table version 2, written from the game's state at the
    save); the bonus as chosen (the engine draws none yet, E10); `adv-load-more` masks the drawn
    cells (the client's draws are not the original's).
39. (T5b1) The names by colour handed on at Load (`Scenario::seated`), kept beside the loaded game
    for E24 M5, which makes the humans those (no engine command changes them yet).
40. (T5b1) `h3-view save`'s hot seat: `--player <colour>:<name>:<town>:<hero>:<bonus>` and
    `--duration`; the makers' `viewer` tables name the players by indexes.
41. (T5b2) CLT-Q110 (b)'s detail in the client's profile: in a load a flag's press is taken only on
    a colour the save holds as human (else nothing, as a colour no human may play), then passed
    on as CLT-Q94's exchange, past the last name on to the first (changed at T6, the review's C1:
    always an exchange, never to the computer); a colour a name holds through the seating's second
    pass (a computer's in the save, CLT-Q160 (a)) moves with that exchange.
42. (T5b2) In a load a record that moves by a flag's press takes its new colour's saved record
    whole (the town, the first hero, the bonus and the handicap), as item 37's rows; the handicap
    is not read from the original (a load's handicap takes no press, §42.1).
