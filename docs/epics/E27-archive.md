# E27 — archive: M1 (reading and captures), M2 (the start and the menus) and their review lists

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
